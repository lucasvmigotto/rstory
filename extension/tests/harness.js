// @ts-check
// A tiny test runner and a fake daemon for the gjs fixture tests. Synthetic data only; the fake
// listens on a socket inside a temp dir and never touches a real daemon or the session.

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import System from 'system';

const decoder = new TextDecoder();
const encoder = new TextEncoder();

/** @type {{name: string, fn: () => Promise<void>|void}[]} */
const tests = [];

/** @param {string} name @param {() => Promise<void>|void} fn */
export function test(name, fn) {
    tests.push({name, fn});
}

export async function runAll() {
    let failed = 0;
    for (const t of tests) {
        try {
            await t.fn();
            print(`ok - ${t.name}`);
        } catch (e) {
            failed++;
            print(`not ok - ${t.name}: ${e instanceof Error ? e.message : e}`);
        }
    }
    print(`${tests.length - failed}/${tests.length} passed`);
    if (failed > 0)
        System.exit(1);
}

/** @param {boolean} cond @param {string} msg */
export function assert(cond, msg) {
    if (!cond)
        throw new Error(msg);
}

/** @param {unknown} actual @param {unknown} expected @param {string} [msg] */
export function assertEquals(actual, expected, msg = '') {
    const a = JSON.stringify(actual);
    const b = JSON.stringify(expected);
    if (a !== b)
        throw new Error(`${msg} expected ${b}, got ${a}`);
}

/** @param {number} ms */
export function sleep(ms) {
    return new Promise(resolve => {
        GLib.timeout_add(GLib.PRIORITY_DEFAULT, ms, () => {
            resolve(undefined);
            return GLib.SOURCE_REMOVE;
        });
    });
}

/** @param {() => boolean} cond @param {string} what @param {number} [ms] */
export async function waitFor(cond, what, ms = 5000) {
    const end = GLib.get_monotonic_time() + ms * 1000;
    while (!cond()) {
        if (GLib.get_monotonic_time() > end)
            throw new Error(`timed out waiting for ${what}`);
        await sleep(10);
    }
}

/** @returns {string} a fresh temp dir */
export function tempDir() {
    return GLib.dir_make_tmp('rstory-ext-test-XXXXXX');
}

/** Root of the repository, from this file's location. */
export function repoRoot() {
    const here = Gio.File.new_for_uri(import.meta.url); // extension/tests/harness.js
    return /** @type {Gio.File} */ (here.get_parent()?.get_parent()?.get_parent()).get_path() ?? '';
}

/** @param {string} path @returns {string[]} the non-empty lines of a fixture file */
export function readLines(path) {
    const [, bytes] = Gio.File.new_for_path(path).load_contents(null);
    return decoder.decode(bytes).split('\n').filter(l => l.trim() !== '');
}

/** @param {string} dir @returns {string[]} file names, sorted */
export function listDir(dir) {
    const names = [];
    const e = Gio.File.new_for_path(dir).enumerate_children('standard::name', 0, null);
    for (let i = e.next_file(null); i !== null; i = e.next_file(null))
        names.push(i.get_name());
    return names.sort();
}

/**
 * @typedef {(req: any, reply: (obj: any) => void, conn: Gio.SocketConnection) => void} Handler
 */

/** A fake rstory daemon: NDJSON over a Unix socket, scripted by a handler. */
export class FakeDaemon {
    /** @type {Gio.SocketService|null} */
    #service = null;
    /** @type {Set<{conn: Gio.SocketConnection, out: Gio.OutputStream}>} */
    #clients = new Set();
    #path;
    #handler;
    /** every request received, in order */
    requests = /** @type {any[]} */ ([]);
    connections = 0;

    /** @param {string} path @param {Handler} handler */
    constructor(path, handler) {
        this.#path = path;
        this.#handler = handler;
    }

    get path() {
        return this.#path;
    }

    start() {
        try {
            Gio.File.new_for_path(this.#path).delete(null);
        } catch (_e) {
            // nothing to remove
        }
        const service = new Gio.SocketService();
        service.add_address(Gio.UnixSocketAddress.new(this.#path), Gio.SocketType.STREAM,
            Gio.SocketProtocol.DEFAULT, null);
        service.connect('incoming', (_s, conn) => {
            this.#accept(conn);
            return true;
        });
        service.start();
        this.#service = service;
    }

    /** Stops listening and drops every client (a daemon restart, from the client's side). */
    stop() {
        this.#service?.stop();
        this.#service?.close();
        this.#service = null;
        for (const c of this.#clients) {
            try {
                c.conn.close(null);
            } catch (_e) {
                // already closed
            }
        }
        this.#clients.clear();
        try {
            Gio.File.new_for_path(this.#path).delete(null);
        } catch (_e) {
            // already gone
        }
    }

    /** Pushes a raw line (or an object) to every connected client. @param {string|object} msg */
    push(msg) {
        const line = typeof msg === 'string' ? msg : JSON.stringify(msg);
        for (const c of this.#clients)
            c.out.write_all(encoder.encode(`${line}\n`), null);
    }

    get clientCount() {
        return this.#clients.size;
    }

    /** @param {Gio.SocketConnection} conn */
    #accept(conn) {
        this.connections++;
        const out = conn.get_output_stream();
        const client = {conn, out};
        this.#clients.add(client);
        const input = new Gio.DataInputStream({base_stream: conn.get_input_stream()});
        const loop = () => {
            input.read_line_async(GLib.PRIORITY_DEFAULT, null, (_s, res) => {
                let bytes;
                try {
                    [bytes] = input.read_line_finish(res);
                } catch (_e) {
                    this.#clients.delete(client);
                    return;
                }
                if (bytes === null) {
                    this.#clients.delete(client);
                    return;
                }
                const req = JSON.parse(decoder.decode(bytes));
                this.requests.push(req);
                const reply = (/** @type {any} */ obj) =>
                    out.write_all(encoder.encode(`${JSON.stringify(obj)}\n`), null);
                this.#handler(req, reply, conn);
                loop();
            });
        };
        loop();
    }
}

/**
 * A handler that answers from the shared protocol fixtures: for each op, the first matching
 * request/response pair in `examples.jsonl`, with the id rewritten to the client's.
 * @param {string} examplesPath
 * @returns {Handler}
 */
export function fixtureHandler(examplesPath) {
    const lines = readLines(examplesPath).map(l => JSON.parse(l));
    /** @type {Map<string, any>} */
    const responses = new Map();
    for (let i = 0; i + 1 < lines.length; i++) {
        if (lines[i].op && lines[i + 1].id === lines[i].id && !responses.has(lines[i].op))
            responses.set(lines[i].op, lines[i + 1]);
    }
    return (req, reply) => {
        const r = responses.get(req.op);
        if (r)
            reply({...r, id: req.id});
    };
}

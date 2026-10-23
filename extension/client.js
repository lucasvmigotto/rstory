// @ts-check
// Protocol v1 client for the rstory control socket (contracts/protocol.md).
// No history logic lives here: it connects, frames NDJSON, matches responses to requests by id,
// forwards events, and reconnects with backoff. It never spawns a process and never logs content.

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

/**
 * @typedef {{state: 'recording'|'paused'|'inactive', paused_until?: string|null, reason?: string}} State
 * @typedef {{id: string, short_id: string, index: number, size: number, created: string,
 *   last_copied: string, pinned: boolean, preview: string}} EntrySummary
 * @typedef {State & {backend: 'wayland'|'x11'|'none', mode: 'native'|'xwayland_bridge'|null,
 *   entries: number, pinned: number, max_entries: number, max_pinned: number,
 *   counters: {skipped_sensitive: number, skipped_oversize: number, skipped_paused: number,
 *   dropped_burst: number}, tray: 'active'|'no_host'|'no_bus'|'disabled',
 *   picker_available: boolean, version: string, protocol: 1}} Status
 * @typedef {{entries: number, top: string|null}} HistoryChangedData
 * @typedef {'history_changed'|'pause_changed'} EventName
 * @typedef {{
 *   path?: string,
 *   backoffMinMs?: number,
 *   backoffMaxMs?: number,
 *   onConnected?: () => void,
 *   onDisconnected?: () => void,
 *   onEvent?: (name: string, data: any) => void,
 * }} ClientOptions
 */

export const PROTOCOL_VERSION = 1;
export const BACKOFF_MIN_MS = 1000;
export const BACKOFF_MAX_MS = 30000;

/** The doubled delay, capped. */
export function nextBackoff(/** @type {number} */ ms, max = BACKOFF_MAX_MS) {
    return Math.min(ms * 2, max);
}

/** `$XDG_RUNTIME_DIR/rstory/control.sock` */
export function defaultSocketPath() {
    return GLib.build_filenamev([GLib.get_user_runtime_dir(), 'rstory', 'control.sock']);
}

/** A failed request: carries the protocol error `code` (never content). */
export class RequestError extends Error {
    /** @param {string} code @param {string} message */
    constructor(code, message) {
        super(message);
        this.name = 'RequestError';
        this.code = code;
    }
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export class Client {
    #path;
    #minMs;
    #maxMs;
    #opts;
    #delay;
    #generation = 0;
    #running = false;
    #timer = 0;
    #nextId = 0;
    /** @type {Gio.SocketConnection|null} */
    #conn = null;
    /** @type {Gio.Cancellable|null} */
    #cancellable = null;
    /** @type {Gio.OutputStream|null} */
    #output = null;
    /** @type {Map<number, {resolve: (v: any) => void, reject: (e: Error) => void}>} */
    #pending = new Map();
    /** @type {Promise<void>} */
    #writeChain = Promise.resolve();

    /** @param {ClientOptions} [opts] */
    constructor(opts = {}) {
        this.#opts = opts;
        this.#path = opts.path ?? defaultSocketPath();
        this.#minMs = opts.backoffMinMs ?? BACKOFF_MIN_MS;
        this.#maxMs = opts.backoffMaxMs ?? BACKOFF_MAX_MS;
        this.#delay = this.#minMs;
    }

    get connected() {
        return this.#conn !== null;
    }

    start() {
        if (this.#running)
            return;
        this.#running = true;
        this.#delay = this.#minMs;
        this.#connect();
    }

    /** Closes the socket, cancels every pending operation and removes the reconnect timer. */
    stop() {
        this.#running = false;
        this.#generation++;
        if (this.#timer) {
            GLib.source_remove(this.#timer);
            this.#timer = 0;
        }
        this.#teardown();
    }

    /**
     * @param {string} op
     * @param {Record<string, unknown>} [args]
     * @returns {Promise<any>} the `result` object; rejects with a RequestError on `ok:false`
     */
    request(op, args = {}) {
        if (!this.#conn)
            return Promise.reject(new RequestError('disconnected', 'not connected'));
        const id = ++this.#nextId;
        const line = `${JSON.stringify({v: PROTOCOL_VERSION, id, op, args})}\n`;
        return new Promise((resolve, reject) => {
            this.#pending.set(id, {resolve, reject});
            const gen = this.#generation;
            this.#write(line).catch(() => {
                this.#pending.delete(id);
                reject(new RequestError('disconnected', 'write failed'));
                this.#closed(gen);
            });
        });
    }

    #connect() {
        this.#timer = 0;
        if (!this.#running)
            return;
        const gen = ++this.#generation;
        const cancellable = new Gio.Cancellable();
        this.#cancellable = cancellable;
        const client = new Gio.SocketClient();
        const address = Gio.UnixSocketAddress.new(this.#path);
        client.connect_async(address, cancellable, (_src, res) => {
            if (gen !== this.#generation)
                return;
            let conn;
            try {
                conn = client.connect_finish(res);
            } catch {
                this.#scheduleReconnect();
                return;
            }
            this.#attach(conn, cancellable, gen);
        });
    }

    /**
     * @param {Gio.SocketConnection} conn
     * @param {Gio.Cancellable} cancellable
     * @param {number} gen
     */
    #attach(conn, cancellable, gen) {
        this.#conn = conn;
        this.#output = conn.get_output_stream();
        const input = new Gio.DataInputStream({
            base_stream: conn.get_input_stream(),
            close_base_stream: false,
        });
        this.#opts.onConnected?.();
        // onConnected may have called stop().
        if (gen !== this.#generation)
            return;
        this.#readLine(input, cancellable, gen);
    }

    /**
     * @param {Gio.DataInputStream} input
     * @param {Gio.Cancellable} cancellable
     * @param {number} gen
     */
    #readLine(input, cancellable, gen) {
        input.read_line_async(GLib.PRIORITY_DEFAULT, cancellable, (_src, res) => {
            if (gen !== this.#generation)
                return;
            let bytes;
            try {
                [bytes] = input.read_line_finish(res);
            } catch {
                this.#closed(gen);
                return;
            }
            if (bytes === null) {
                this.#closed(gen);
                return;
            }
            this.#dispatch(decoder.decode(bytes));
            if (gen === this.#generation)
                this.#readLine(input, cancellable, gen);
        });
    }

    /** @param {string} line */
    #dispatch(line) {
        /** @type {any} */
        let msg;
        try {
            msg = JSON.parse(line);
        } catch {
            return; // a malformed line is not ours to answer; unknown input is ignored
        }
        if (typeof msg !== 'object' || msg === null || msg.v !== PROTOCOL_VERSION)
            return;
        this.#delay = this.#minMs; // the daemon talks protocol v1: the link is healthy
        if (typeof msg.event === 'string') {
            this.#opts.onEvent?.(msg.event, msg.data);
            return;
        }
        const entry = this.#pending.get(msg.id);
        if (!entry)
            return;
        this.#pending.delete(msg.id);
        if (msg.ok === true) {
            entry.resolve(msg.result ?? {});
        } else {
            const e = msg.error ?? {};
            entry.reject(new RequestError(String(e.code ?? 'internal'), String(e.message ?? '')));
        }
    }

    /** @param {string} line */
    #write(line) {
        const output = this.#output;
        const cancellable = this.#cancellable;
        const bytes = encoder.encode(line);
        const next = this.#writeChain.then(() => new Promise((resolve, reject) => {
            if (!output) {
                reject(new Error('closed'));
                return;
            }
            output.write_all_async(bytes, GLib.PRIORITY_DEFAULT, cancellable, (_src, res) => {
                try {
                    output.write_all_finish(res);
                    resolve(undefined);
                } catch (e) {
                    reject(e);
                }
            });
        }));
        this.#writeChain = next.then(() => {}, () => {});
        return /** @type {Promise<void>} */ (next);
    }

    /** @param {number} gen */
    #closed(gen) {
        if (gen !== this.#generation)
            return;
        this.#generation++;
        const wasConnected = this.#conn !== null;
        this.#teardown();
        if (wasConnected)
            this.#opts.onDisconnected?.();
        this.#scheduleReconnect();
    }

    #teardown() {
        this.#cancellable?.cancel();
        this.#cancellable = null;
        try {
            this.#conn?.close(null);
        } catch {
            // already closed
        }
        this.#conn = null;
        this.#output = null;
        this.#writeChain = Promise.resolve();
        const pending = [...this.#pending.values()];
        this.#pending.clear();
        for (const p of pending)
            p.reject(new RequestError('disconnected', 'connection closed'));
    }

    #scheduleReconnect() {
        if (!this.#running || this.#timer)
            return;
        const delay = this.#delay;
        this.#delay = nextBackoff(delay, this.#maxMs);
        this.#timer = GLib.timeout_add(GLib.PRIORITY_DEFAULT, delay, () => {
            this.#timer = 0;
            this.#connect();
            return GLib.SOURCE_REMOVE;
        });
    }
}

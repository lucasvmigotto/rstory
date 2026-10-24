// @ts-check
// Protocol client tests under gjs: replay the shared fixtures (SC-002) against a fake daemon.
import {BACKOFF_MAX_MS, BACKOFF_MIN_MS, Client, RequestError, nextBackoff} from '../client.js';
import {FakeDaemon, assert, assertEquals, fixtureHandler, listDir, readLines, repoRoot, runAll, sleep,
    tempDir, test, waitFor} from './harness.js';

const FIXTURES = `${repoRoot()}/tests/fixtures/protocol`;

/** @param {string} [name] */
function socketIn(name = 'control.sock') {
    return `${tempDir()}/${name}`;
}

test('backoff doubles from 1 s and is capped at 30 s', () => {
    assertEquals([BACKOFF_MIN_MS, BACKOFF_MAX_MS], [1000, 30000]);
    let ms = BACKOFF_MIN_MS;
    const seen = [ms];
    for (let i = 0; i < 7; i++) {
        ms = nextBackoff(ms);
        seen.push(ms);
    }
    assertEquals(seen, [1000, 2000, 4000, 8000, 16000, 30000, 30000, 30000]);
});

test('every operation in examples.jsonl round-trips through the client', async () => {
    const daemon = new FakeDaemon(socketIn(), fixtureHandler(`${FIXTURES}/examples.jsonl`));
    daemon.start();
    let up = false;
    const c = new Client({path: daemon.path, onConnected: () => { up = true; }});
    c.start();
    try {
        await waitFor(() => up, 'connect');
        const lines = readLines(`${FIXTURES}/examples.jsonl`).map(l => JSON.parse(l));
        let checked = 0;
        for (let i = 0; i + 1 < lines.length; i++) {
            const req = lines[i];
            const res = lines[i + 1];
            if (!req.op || res.id !== req.id)
                continue;
            if (checked > 0 && lines.slice(0, i).some(l => l.op === req.op))
                continue; // the first pair per op is what the fake serves
            if (res.ok) {
                assertEquals(await c.request(req.op, req.args), res.result, req.op);
            } else {
                try {
                    await c.request(req.op, req.args);
                    assert(false, `${req.op} should have failed`);
                } catch (e) {
                    assert(e instanceof RequestError, 'a RequestError');
                    assertEquals(/** @type {RequestError} */ (e).code, res.error.code);
                }
            }
            checked++;
        }
        assert(checked >= 10, `replayed ${checked} operations`);
    } finally {
        c.stop();
        daemon.stop();
    }
});

test('an error response rejects with the protocol code', async () => {
    const daemon = new FakeDaemon(socketIn(), (req, reply) => reply({
        v: 1, id: req.id, ok: false, error: {code: 'pin_limit', message: 'pin limit reached (10)'},
    }));
    daemon.start();
    let up = false;
    const c = new Client({path: daemon.path, onConnected: () => { up = true; }});
    c.start();
    try {
        await waitFor(() => up, 'connect');
        let code = '';
        await c.request('pin', {ref: '0'}).catch(e => { code = e.code; });
        assertEquals(code, 'pin_limit');
    } finally {
        c.stop();
        daemon.stop();
    }
});

test('events carry no id and reach onEvent; fixture events parse', async () => {
    const daemon = new FakeDaemon(socketIn(), (req, reply) => reply({v: 1, id: req.id, ok: true, result: {}}));
    daemon.start();
    /** @type {any[]} */
    const events = [];
    const c = new Client({path: daemon.path, onEvent: (n, d) => events.push([n, d])});
    c.start();
    try {
        await waitFor(() => c.connected, 'connect');
        await c.request('subscribe', {events: ['history_changed']});
        for (const f of ['history_changed', 'pause_changed']) {
            for (const line of readLines(`${FIXTURES}/valid/${f}.jsonl`))
                daemon.push(line);
        }
        await waitFor(() => events.length >= 2, 'events');
        assert(events.some(e => e[0] === 'history_changed'), 'history_changed seen');
        assert(events.some(e => e[0] === 'pause_changed'), 'pause_changed seen');
    } finally {
        c.stop();
        daemon.stop();
    }
});

test('invalid fixture lines are ignored and the link keeps working', async () => {
    const daemon = new FakeDaemon(socketIn(), (req, reply) => reply({v: 1, id: req.id, ok: true, result: {n: 1}}));
    daemon.start();
    const c = new Client({path: daemon.path});
    c.start();
    try {
        await waitFor(() => c.connected, 'connect');
        for (const dir of ['invalid', 'invalid_entry', 'invalid_status']) {
            for (const f of listDir(`${FIXTURES}/${dir}`)) {
                for (const line of readLines(`${FIXTURES}/${dir}/${f}`)) {
                    // Responses to ids we never asked for and events with bad shapes: no crash.
                    daemon.push(unrelatedId(line));
                }
            }
        }
        daemon.push('{not json');
        daemon.push('[1,2,3]');
        assertEquals(await c.request('status'), {n: 1});
    } finally {
        c.stop();
        daemon.stop();
    }
});

test('a request while disconnected rejects without sending', async () => {
    const c = new Client({path: socketIn('absent.sock'), backoffMinMs: 20});
    let code = '';
    await c.request('status').catch(e => { code = e.code; });
    assertEquals(code, 'disconnected');
});

test('daemon restart: pending requests reject, then the client reconnects with backoff', async () => {
    const path = socketIn();
    const quiet = new FakeDaemon(path, () => {});
    quiet.start();
    let connects = 0;
    let disconnects = 0;
    const c = new Client({path, backoffMinMs: 30, backoffMaxMs: 120,
        onConnected: () => { connects++; }, onDisconnected: () => { disconnects++; }});
    c.start();
    try {
        await waitFor(() => connects === 1, 'first connect');
        const pending = c.request('status').then(() => 'ok', e => e.code);
        await waitFor(() => quiet.requests.length === 1, 'request received');
        quiet.stop();
        assertEquals(await pending, 'disconnected');
        await waitFor(() => disconnects === 1, 'disconnect callback');
        assert(!c.connected, 'not connected');
        await sleep(100); // retries fail while the daemon is down
        assertEquals(connects, 1);
        const back = new FakeDaemon(path, (req, reply) => reply({v: 1, id: req.id, ok: true, result: {}}));
        back.start();
        try {
            await waitFor(() => connects === 2, 'reconnect');
            assertEquals(await c.request('status'), {});
        } finally {
            c.stop();
            back.stop();
        }
    } finally {
        c.stop();
    }
});

test('stop() closes the socket and cancels the reconnect timer', async () => {
    const path = socketIn();
    const d = new FakeDaemon(path, () => {});
    d.start();
    let connects = 0;
    const c = new Client({path, backoffMinMs: 20, onConnected: () => { connects++; }});
    c.start();
    await waitFor(() => connects === 1, 'connect');
    c.stop();
    await waitFor(() => d.clientCount === 0, 'server sees the close');
    d.stop();
    const d2 = new FakeDaemon(path, () => {});
    d2.start();
    await sleep(150);
    assertEquals(connects, 1);
    assertEquals(d2.connections, 0);
    d2.stop();
});

/** Moves a fixture's request id out of the way of the ids this test uses. @param {string} line */
function unrelatedId(line) {
    try {
        const o = JSON.parse(line);
        if (o !== null && typeof o === 'object' && typeof o.id === 'number')
            return JSON.stringify({...o, id: 900000 + Math.abs(o.id)});
    } catch {
        // not JSON: pushed as is
    }
    return line;
}

await runAll();

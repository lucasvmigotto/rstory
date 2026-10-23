// @ts-check
// Controller tests: subscribe, fetch and keep the menu state current, against a fake daemon that
// answers from the shared protocol fixtures.
import {Controller} from '../controller.js';
import {buildMenu, iconName} from '../menu.js';
import {FakeDaemon, assert, assertEquals, fixtureHandler, repoRoot, runAll, sleep, tempDir, test,
    waitFor} from './harness.js';

const EXAMPLES = `${repoRoot()}/tests/fixtures/protocol/examples.jsonl`;

/** @param {import('./harness.js').Handler} [handler] */
function daemonWith(handler = fixtureHandler(EXAMPLES)) {
    const d = new FakeDaemon(`${tempDir()}/control.sock`, handler);
    d.start();
    return d;
}

test('on connect: subscribe, then status and the newest 10 entries', async () => {
    const d = daemonWith();
    /** @type {any[]} */
    const states = [];
    const c = new Controller({path: d.path, onChange: s => states.push(s)});
    c.start();
    try {
        await waitFor(() => states.length > 0, 'first state');
        assertEquals(d.requests.map(r => r.op).slice(0, 3), ['subscribe', 'status', 'list']);
        assertEquals(d.requests[0].args, {events: ['history_changed', 'pause_changed']});
        assertEquals(d.requests[2].args, {limit: 10});
        const state = states[0];
        assert(state.connected, 'connected');
        assertEquals(state.status.version, '0.1.0');
        assertEquals(state.entries.length, 2);
        assertEquals(iconName(state), 'recording');
        const menu = buildMenu(state, Date.now());
        assertEquals(menu.filter(i => i.type === 'entry').length, 2);
    } finally {
        c.stop();
        d.stop();
    }
});

test('history_changed re-reads status and the list', async () => {
    const d = daemonWith();
    let changes = 0;
    const c = new Controller({path: d.path, onChange: () => { changes++; }});
    c.start();
    try {
        await waitFor(() => changes === 1, 'first state');
        d.push({v: 1, event: 'history_changed', data: {entries: 9, top: null}});
        await waitFor(() => changes === 2, 'second state');
        const ops = d.requests.map(r => r.op);
        assertEquals(ops.slice(3, 5).sort(), ['list', 'status']);
    } finally {
        c.stop();
        d.stop();
    }
});

test('pause_changed switches the icon and the status label without a request', async () => {
    const d = daemonWith();
    /** @type {any} */
    let last = null;
    const c = new Controller({path: d.path, onChange: s => { last = s; }});
    c.start();
    try {
        await waitFor(() => last !== null, 'first state');
        const before = d.requests.length;
        d.push({v: 1, event: 'pause_changed', data: {state: 'paused', paused_until: null}});
        await waitFor(() => last.status.state === 'paused', 'paused');
        assertEquals(iconName(last), 'paused');
        assertEquals(buildMenu(last, 0).find(i => 'id' in i && i.id === 'status')?.label, 'Paused');
        d.push({v: 1, event: 'pause_changed', data: {state: 'recording', paused_until: null}});
        await waitFor(() => last.status.state === 'recording', 'recording');
        assertEquals(d.requests.length, before);
    } finally {
        c.stop();
        d.stop();
    }
});

test('no daemon: disconnected state; it connects once the daemon appears', async () => {
    const path = `${tempDir()}/control.sock`;
    /** @type {any} */
    let last = null;
    const c = new Controller({path, backoffMinMs: 30, backoffMaxMs: 60, onChange: s => { last = s; }});
    c.start();
    try {
        await sleep(100);
        assertEquals(iconName(c.state), 'disconnected');
        assertEquals(buildMenu(c.state, 0).length, 1);
        const d = new FakeDaemon(path, fixtureHandler(EXAMPLES));
        d.start();
        try {
            await waitFor(() => last?.connected === true, 'connected');
            assertEquals(iconName(last), 'recording');
            d.stop();
            await waitFor(() => last.connected === false, 'disconnected again');
            assertEquals(iconName(last), 'disconnected');
        } finally {
            d.stop();
        }
    } finally {
        c.stop();
    }
});

test('activate sends the item request; failures are reported by code only', async () => {
    const d = daemonWith((req, reply) => {
        if (req.op === 'restore')
            reply({v: 1, id: req.id, ok: false, error: {code: 'inactive', message: 'no backend'}});
        else
            fixtureHandler(EXAMPLES)(req, reply, /** @type {any} */ (null));
    });
    /** @type {string[]} */
    const failures = [];
    let ready = false;
    const c = new Controller({path: d.path, onChange: () => { ready = true; },
        onFailure: (op, code) => failures.push(`${op}:${code}`)});
    c.start();
    try {
        await waitFor(() => ready, 'first state');
        await c.activate({op: 'restore', args: {ref: '01JB'}});
        assertEquals(failures, ['restore:inactive']);
        await c.activate({op: 'pause', args: {for: '1m'}});
        assertEquals(d.requests.at(-1).op, 'pause');
        assertEquals(d.requests.at(-1).args, {for: '1m'});
        await c.activate(null); // Cancel: nothing is sent
        assertEquals(d.requests.at(-1).op, 'pause');
    } finally {
        c.stop();
        d.stop();
    }
});

test('stop() leaves no connection and no callbacks', async () => {
    const d = daemonWith();
    let changes = 0;
    const c = new Controller({path: d.path, backoffMinMs: 20, onChange: () => { changes++; }});
    c.start();
    await waitFor(() => changes === 1, 'first state');
    c.stop();
    await waitFor(() => d.clientCount === 0, 'socket closed');
    d.push({v: 1, event: 'history_changed', data: {entries: 1, top: null}});
    await sleep(100);
    assertEquals(changes, 1);
    d.stop();
});

await runAll();

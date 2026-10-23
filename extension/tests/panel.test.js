// @ts-check
// Panel tests: the PanelMenu.Button, the menu rendered from the model, the actions, and the
// complete cleanup in destroy() (what disable() calls). The Shell toolkit is replaced by fakes.
import GLib from 'gi://GLib';
import {RstoryPanel, failureMessage} from '../panel.js';
import {fakeUi} from './fake_shell.js';
import {FakeDaemon, assert, assertEquals, fixtureHandler, repoRoot, runAll, sleep, tempDir, test,
    waitFor} from './harness.js';

const EXAMPLES = `${repoRoot()}/tests/fixtures/protocol/examples.jsonl`;
const ICONS = `${repoRoot()}/extension/icons`;

/** @param {import('./harness.js').Handler} [handler] */
function setup(handler = fixtureHandler(EXAMPLES)) {
    const daemon = new FakeDaemon(`${tempDir()}/control.sock`, handler);
    daemon.start();
    /** @type {any[]} */
    const added = [];
    /** @type {string[]} */
    const notes = [];
    const panel = new RstoryPanel({
        ui: fakeUi,
        iconDir: ICONS,
        socketPath: daemon.path,
        addToStatusArea: b => added.push(b),
        notify: msg => notes.push(msg),
        tickMs: 40,
        backoffMinMs: 30,
        backoffMaxMs: 60,
    });
    return {daemon, panel, added, notes, button: /** @type {any} */ (panel.button)};
}

/** @param {any} button */
const iconFile = button => button.children[0].gicon.get_file().get_basename();
/** @param {any} button */
const labels = button => button.menu.items.map((/** @type {any} */ i) => i.label?.text ?? '---');

test('enable: a button in the status area, disconnected icon, one insensitive item', () => {
    const {daemon, panel, added, button} = setup();
    try {
        assertEquals(added.length, 1);
        assert(added[0] === button, 'the button is handed to the status area');
        assertEquals(iconFile(button), 'disconnected-symbolic.svg');
        assertEquals(button.accessible_name, 'rstory — daemon not running');
        assertEquals(labels(button), ['rstory daemon not running']);
        assertEquals(button.menu.items[0].sensitive, false);
    } finally {
        panel.destroy();
        daemon.stop();
    }
});

test('connected: the menu follows contracts/menu.md and the icon is the recording shape', async () => {
    const {daemon, panel, button} = setup();
    try {
        await waitFor(() => iconFile(button) === 'recording-symbolic.svg', 'recording icon');
        const l = labels(button);
        assertEquals(l.slice(0, 2).length, 2);
        const tail = l.slice(-6);
        assertEquals(tail[0], '---');
        assertEquals(l.includes('Pause recording'), true);
        assertEquals(l.includes('Pause for 1 minute'), true);
        assertEquals(l.includes('Open picker'), true);
        assertEquals(l[l.length - 1], 'Clear history…');
        assertEquals(button.accessible_name, 'rstory — Recording');
    } finally {
        panel.destroy();
        daemon.stop();
    }
});

test('clicking an entry sends restore with its id', async () => {
    const {daemon, panel, button} = setup();
    try {
        await waitFor(() => button.menu.items.length > 3, 'menu');
        button.menu.items[0].activate();
        await waitFor(() => daemon.requests.some(r => r.op === 'restore'), 'restore request');
        const req = daemon.requests.find(r => r.op === 'restore');
        assertEquals(typeof req.args.ref, 'string');
        assertEquals(req.args.ref.length, 26);
    } finally {
        panel.destroy();
        daemon.stop();
    }
});

test('pause, pause for 1 minute and open picker are plain daemon requests', async () => {
    const {daemon, panel, button} = setup();
    try {
        await waitFor(() => button.menu.items.length > 3, 'menu');
        /** @param {string} label */
        const click = label => button.menu.items.find((/** @type {any} */ i) => i.label?.text === label).activate();
        click('Pause for 1 minute');
        click('Pause recording');
        click('Open picker');
        await waitFor(() => daemon.requests.filter(r => ['pause', 'open_picker'].includes(r.op)).length === 3,
            'three requests');
        const sent = daemon.requests.filter(r => ['pause', 'open_picker'].includes(r.op))
            .map(r => [r.op, r.args]);
        assertEquals(sent, [['pause', {for: '1m'}], ['pause', {}], ['open_picker', {}]]);
    } finally {
        panel.destroy();
        daemon.stop();
    }
});

test('pause_changed switches the icon to the paused shape and the menu to Resume', async () => {
    const {daemon, panel, button} = setup();
    try {
        await waitFor(() => iconFile(button) === 'recording-symbolic.svg', 'recording');
        daemon.push({v: 1, event: 'pause_changed', data: {state: 'paused', paused_until: null}});
        await waitFor(() => iconFile(button) === 'paused-symbolic.svg', 'paused icon');
        assert(labels(button).includes('Resume recording'), 'Resume shown');
        assert(!labels(button).includes('Pause for 1 minute'), 'pause 1 min hidden');
        assertEquals(button.accessible_name, 'rstory — Paused');
    } finally {
        panel.destroy();
        daemon.stop();
    }
});

test('clear history: confirm sends clear (not all), cancel sends nothing', async () => {
    const {daemon, panel, button} = setup();
    try {
        await waitFor(() => button.menu.items.length > 3, 'menu');
        const sub = button.menu.items.at(-1);
        assertEquals(sub.menu.items.length, 2);
        assert(/^Delete \d+ unpinned entries$/.test(sub.menu.items[0].label.text), 'counted confirm');
        sub.menu.items[1].activate();
        await sleep(60);
        assert(!daemon.requests.some(r => r.op === 'clear'), 'cancel sends nothing');
        assertEquals(button.menu.closeCalls, 1);
        sub.menu.items[0].activate();
        await waitFor(() => daemon.requests.some(r => r.op === 'clear'), 'clear request');
        assertEquals(daemon.requests.find(r => r.op === 'clear').args, {all: false});
    } finally {
        panel.destroy();
        daemon.stop();
    }
});

test('a failed action notifies by code, never with entry content', async () => {
    const {daemon, panel, button, notes} = setup((req, reply, conn) => {
        if (req.op === 'restore')
            reply({v: 1, id: req.id, ok: false, error: {code: 'inactive', message: 'secret-looking text'}});
        else
            fixtureHandler(EXAMPLES)(req, reply, conn);
    });
    try {
        await waitFor(() => button.menu.items.length > 3, 'menu');
        button.menu.items[0].activate();
        await waitFor(() => notes.length === 1, 'notification');
        assertEquals(notes[0], failureMessage('restore', 'inactive'));
        assert(!notes[0].includes('secret'), 'the daemon message is not echoed');
        assertEquals(failureMessage('restore', 'not_found'), null);
    } finally {
        panel.destroy();
        daemon.stop();
    }
});

test('a timed pause counts down in place while the menu is open', async () => {
    const until = new Date(Date.now() + 90_000).toISOString();
    const {daemon, panel, button} = setup();
    try {
        await waitFor(() => iconFile(button) === 'recording-symbolic.svg', 'recording');
        daemon.push({v: 1, event: 'pause_changed', data: {state: 'paused', paused_until: until}});
        await waitFor(() => iconFile(button) === 'paused-symbolic.svg', 'paused');
        const status = () => button.menu.items.find((/** @type {any} */ i) => /^Paused/.test(i.label?.text ?? ''));
        button.menu.open();
        const item = status();
        const first = item.label.text;
        assert(/^Paused — resumes in 1:(2\d|30)$/.test(first), first);
        await waitFor(() => item.label.text !== first, 'the label ticks');
        assert(status() === item, 'the same item is updated, not rebuilt');
        button.menu.close();
        const frozen = item.label.text;
        await sleep(150);
        assertEquals(item.label.text, frozen);
    } finally {
        panel.destroy();
        daemon.stop();
    }
});

test('destroy: socket closed, timers and handlers gone, nothing renders afterwards', async () => {
    const {daemon, panel, button} = setup();
    await waitFor(() => button.menu.items.length > 3, 'menu');
    button.menu.open(); // starts the countdown timer
    assert(panel.tickSource !== 0, 'the open menu has a timer');
    const tick = panel.tickSource;
    panel.destroy();
    assert(button.destroyed, 'button destroyed');
    assertEquals(button.handlers.size + button.menu.handlers.size, 0);
    assert(GLib.MainContext.default().find_source_by_id(tick) === null, 'timer removed');
    assertEquals(panel.tickSource, 0);
    await waitFor(() => daemon.clientCount === 0, 'socket closed');
    const items = button.menu.items.length;
    daemon.push({v: 1, event: 'history_changed', data: {entries: 1, top: null}});
    await sleep(100);
    assertEquals(button.menu.items.length, items);
    panel.destroy(); // idempotent
    daemon.stop();
});

test('twenty enable/disable cycles leave nothing behind', async () => {
    const daemon = new FakeDaemon(`${tempDir()}/control.sock`, fixtureHandler(EXAMPLES));
    daemon.start();
    try {
        for (let n = 0; n < 20; n++) {
            const panel = new RstoryPanel({ui: fakeUi, iconDir: ICONS, socketPath: daemon.path,
                addToStatusArea: () => {}, notify: () => {}});
            const button = /** @type {any} */ (panel.button);
            if (n % 2 === 0)
                await waitFor(() => button.menu.items.length > 3, 'menu');
            button.menu.open();
            panel.destroy();
            assertEquals(button.handlers.size + button.menu.handlers.size, 0);
        }
        await waitFor(() => daemon.clientCount === 0, 'every socket closed');
    } finally {
        daemon.stop();
    }
});

await runAll();

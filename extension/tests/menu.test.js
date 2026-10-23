// @ts-check
// Menu model tests: one per column of the state matrix (specs/009-tray/ui.md plus Disconnected).
import {accessibleName, buildMenu, entryLabel, iconName, initialState} from '../menu.js';
import {assert, assertEquals, runAll, test} from './harness.js';

const NOW = Date.parse('2026-10-21T14:00:00Z');

/** @param {number} i @param {boolean} [pinned] @param {string} [preview] */
function entry(i, pinned = false, preview = `item ${i}`) {
    const id = `01JB${String(i).padStart(22, '0')}`;
    return {id, short_id: id.slice(0, 4), index: i, size: 8, created: '2026-10-21T13:00:00Z',
        last_copied: '2026-10-21T13:00:00Z', pinned, preview};
}

/** @param {Record<string, any>} [over] */
function status(over = {}) {
    return {state: 'recording', paused_until: null, backend: 'x11', mode: 'native', entries: 3,
        pinned: 1, max_entries: 20, max_pinned: 10,
        counters: {skipped_sensitive: 0, skipped_oversize: 0, skipped_paused: 0, dropped_burst: 0},
        tray: 'no_host', picker_available: true, version: '0.1.0', protocol: 1, ...over};
}

/** @param {Record<string, any>} [over] @param {any[]} [entries] */
function connected(over = {}, entries = [entry(0), entry(1, true), entry(2)]) {
    return /** @type {any} */ ({connected: true, status: status(over), entries});
}

/** @param {import('../menu.js').MenuItem[]} items @param {string} id @returns {any} */
const byId = (items, id) => items.find(i => 'id' in i && i.id === id);

/** @param {any[]} items */
const ids = items => items.map(i => i.type === 'separator' ? '---' : i.id);

test('disconnected: a single insensitive item and nothing else', () => {
    const items = buildMenu(initialState(), NOW);
    assertEquals(items.length, 1);
    assertEquals(items[0], {type: 'placeholder', id: 'disconnected',
        label: 'rstory daemon not running', sensitive: false});
    assertEquals(iconName(initialState()), 'disconnected');
});

test('normal: entries, separator, status, pause, pause 1 min, picker, separator, clear', () => {
    const items = buildMenu(connected(), NOW);
    assertEquals(ids(items).map(s => s.replace(/^entry:.*/, 'entry')),
        ['entry', 'entry', 'entry', '---', 'status', 'pause', 'pause_minute', 'open_picker', '---',
            'clear']);
    const labels = items.filter(i => i.type === 'entry').map(i => /** @type {any} */ (i).label);
    assertEquals(labels, ['item 0', '📌 item 1', 'item 2']);
    assertEquals(iconName(connected()), 'recording');
});

test('an entry restores by id', () => {
    const e = entry(0);
    const first = /** @type {any} */ (buildMenu(connected({}, [e]), NOW)[0]);
    assertEquals(first.request, {op: 'restore', args: {ref: e.id}});
});

test('at most 10 entries are shown', () => {
    const many = Array.from({length: 14}, (_, i) => entry(i));
    const items = buildMenu(connected({entries: 14}, many), NOW);
    assertEquals(items.filter(i => i.type === 'entry').length, 10);
});

test('empty: an insensitive placeholder instead of entries', () => {
    const items = buildMenu(connected({entries: 0, pinned: 0}, []), NOW);
    assertEquals(items[0], {type: 'placeholder', id: 'empty', label: 'No entries yet',
        sensitive: false});
});

test('status line and pause items per state', () => {
    /** @param {any} state @param {string} id */
    const find = (state, id) => byId(buildMenu(state, NOW), id);
    assertEquals(find(connected(), 'status')?.label, 'Recording');

    const paused = connected({state: 'paused', paused_until: null});
    assertEquals(find(paused, 'status')?.label, 'Paused');
    assertEquals(find(paused, 'resume')?.label, 'Resume recording');
    assert(find(paused, 'pause') === undefined, 'pause hidden while paused');
    assert(find(paused, 'pause_minute') === undefined, 'pause 1 min hidden while paused');
    assertEquals(iconName(paused), 'paused');

    const timed = connected({state: 'paused', paused_until: '2026-10-21T14:01:05Z'});
    assertEquals(find(timed, 'status')?.label, 'Paused — resumes in 1:05');
    assertEquals(find(connected({state: 'paused', paused_until: '2026-10-21T14:00:09Z'}),
        'status')?.label, 'Paused — resumes in 0:09');
    assertEquals(find(connected({state: 'paused', paused_until: '2026-10-21T13:59:00Z'}),
        'status')?.label, 'Paused');

    const inactive = connected({state: 'inactive', reason: 'no clipboard access'});
    assertEquals(find(inactive, 'status')?.label, 'Inactive — no clipboard access');
    assertEquals(find(inactive, 'pause')?.label, 'Pause recording');
    assertEquals(iconName(inactive), 'recording');
});

test('pause requests', () => {
    const items = buildMenu(connected(), NOW);
    assertEquals(byId(items, 'pause').request, {op: 'pause', args: {}});
    assertEquals(byId(items, 'pause_minute').request, {op: 'pause', args: {for: '1m'}});
    const resume = byId(buildMenu(connected({state: 'paused'}), NOW), 'resume');
    assertEquals(resume.request, {op: 'resume', args: {}});
});

test('open picker only when picker_available, and it asks the daemon', () => {
    const none = buildMenu(connected({picker_available: false}), NOW);
    assert(!ids(none).includes('open_picker'), 'hidden without a terminal');
    const item = byId(buildMenu(connected(), NOW), 'open_picker');
    assertEquals(item.request, {op: 'open_picker', args: {}});
});

test('clear is a submenu with a counted confirm and a cancel', () => {
    const items = buildMenu(connected({entries: 12, pinned: 2}), NOW);
    const clear = items[items.length - 1];
    assertEquals(clear.type, 'submenu');
    if (clear.type !== 'submenu')
        return;
    assertEquals(clear.label, 'Clear history…');
    assertEquals(clear.items.map(i => i.label), ['Delete 10 unpinned entries', 'Cancel']);
    assertEquals(clear.items[0].request, {op: 'clear', args: {all: false}});
    assertEquals(clear.items[1].request, null);
});

test('entry labels: newline glyph, 48-char cap, pin prefix', () => {
    assertEquals(entryLabel('a\\nb', false), 'a⏎b');
    assertEquals(entryLabel('x'.repeat(48), false), 'x'.repeat(48));
    assertEquals(entryLabel('x'.repeat(49), false), `${'x'.repeat(47)}…`);
    assertEquals(Array.from(entryLabel('é'.repeat(300), false)).length, 48);
    assertEquals(entryLabel('😀'.repeat(60), true), `📌 ${'😀'.repeat(47)}…`);
});

test('accessible names say the state in words', () => {
    assertEquals(accessibleName(initialState()), 'rstory — daemon not running');
    assertEquals(accessibleName(connected()), 'rstory — Recording');
    assertEquals(accessibleName(connected({state: 'paused', paused_until: '2026-10-21T14:01:05Z'})),
        'rstory — Paused');
    assertEquals(accessibleName(connected({state: 'inactive', reason: 'no backend'})),
        'rstory — Inactive — no backend');
});

await runAll();

// @ts-check
// Menu model for the panel button: a pure function of the daemon state. No Shell, Gio or GLib
// imports, so it runs anywhere. Order and labels follow specs/009-tray/contracts/menu.md; the
// Disconnected state is specific to this extension (specs/010-gnome-extension/ui.md).

/**
 * @typedef {import('./client.js').Status} Status
 * @typedef {import('./client.js').EntrySummary} EntrySummary
 * @typedef {{op: string, args: Record<string, unknown>}} MenuRequest
 *
 * @typedef {{type: 'entry', id: string, label: string, pinned: boolean, sensitive: true,
 *   request: MenuRequest}} EntryItem
 * @typedef {{type: 'placeholder', id: string, label: string, sensitive: false}} PlaceholderItem
 * @typedef {{type: 'separator'}} SeparatorItem
 * @typedef {{type: 'status', id: string, label: string, sensitive: false}} StatusItem
 * @typedef {{type: 'action', id: string, label: string, sensitive: boolean,
 *   request: MenuRequest|null}} ActionItem
 * @typedef {{type: 'submenu', id: string, label: string, sensitive: true,
 *   items: ActionItem[]}} SubmenuItem
 * @typedef {EntryItem|PlaceholderItem|SeparatorItem|StatusItem|ActionItem|SubmenuItem} MenuItem
 *
 * @typedef {'recording'|'paused'|'disconnected'} IconName
 * @typedef {{connected: boolean, status: Status|null, entries: EntrySummary[]}} DaemonState
 */

/** Copy keys of ux-vision (`menu.*`, `state.*`), verbatim. */
export const COPY = Object.freeze({
    recording: 'Recording',
    paused: 'Paused',
    inactive: 'Inactive',
    pause: 'Pause recording',
    resume: 'Resume recording',
    pauseMinute: 'Pause for 1 minute',
    openPicker: 'Open picker',
    clear: 'Clear history…',
    clearCancel: 'Cancel',
    empty: 'No entries yet',
    disconnected: 'rstory daemon not running',
});

export const MAX_ENTRIES = 10;
export const PREVIEW_MAX = 48;

/** @returns {DaemonState} */
export function initialState() {
    return {connected: false, status: null, entries: []};
}

/**
 * One menu line from a daemon preview: the escaped newline becomes `⏎`, at most 48 characters
 * (the last is `…` when cut), prefixed `📌 ` when pinned.
 * @param {string} preview
 * @param {boolean} pinned
 */
export function entryLabel(preview, pinned) {
    const flat = preview.replaceAll('\\n', '⏎');
    const chars = Array.from(flat);
    const text = chars.length > PREVIEW_MAX ? `${chars.slice(0, PREVIEW_MAX - 1).join('')}…` : flat;
    return pinned ? `📌 ${text}` : text;
}

/**
 * @param {Status} status
 * @param {number} nowMs
 */
export function statusLabel(status, nowMs) {
    switch (status.state) {
    case 'paused': {
        const until = status.paused_until ? Date.parse(status.paused_until) : NaN;
        const left = Math.ceil((until - nowMs) / 1000);
        if (!Number.isFinite(left) || left <= 0)
            return COPY.paused;
        const m = Math.floor(left / 60);
        const s = String(left % 60).padStart(2, '0');
        return `${COPY.paused} — resumes in ${m}:${s}`;
    }
    case 'inactive':
        return status.reason ? `${COPY.inactive} — ${status.reason}` : COPY.inactive;
    default:
        return COPY.recording;
    }
}

/** @param {DaemonState} state @returns {IconName} */
export function iconName(state) {
    if (!state.connected || !state.status)
        return 'disconnected';
    // Inactive keeps the recording shape; the tooltip and status line say why.
    return state.status.state === 'paused' ? 'paused' : 'recording';
}

/** The panel button's accessible name: `rstory — {state}`. @param {DaemonState} state */
export function accessibleName(state) {
    if (!state.connected || !state.status)
        return 'rstory — daemon not running';
    return `rstory — ${statusLabel(state.status, 0)}`.replace(/ — resumes in .*$/, '');
}

/**
 * The whole menu. Disconnected: a single insensitive item and nothing else.
 * @param {DaemonState} state
 * @param {number} nowMs
 * @returns {MenuItem[]}
 */
export function buildMenu(state, nowMs) {
    const {status} = state;
    if (!state.connected || !status) {
        return [{type: 'placeholder', id: 'disconnected', label: COPY.disconnected, sensitive: false}];
    }
    /** @type {MenuItem[]} */
    const items = [];
    const entries = state.entries.slice(0, MAX_ENTRIES);
    if (entries.length === 0) {
        items.push({type: 'placeholder', id: 'empty', label: COPY.empty, sensitive: false});
    } else {
        for (const e of entries) {
            items.push({
                type: 'entry',
                id: `entry:${e.id}`,
                label: entryLabel(e.preview, e.pinned),
                pinned: e.pinned,
                sensitive: true,
                request: {op: 'restore', args: {ref: e.id}},
            });
        }
    }
    items.push({type: 'separator'});
    items.push({type: 'status', id: 'status', label: statusLabel(status, nowMs), sensitive: false});

    const paused = status.state === 'paused';
    items.push(paused
        ? {type: 'action', id: 'resume', label: COPY.resume, sensitive: true,
            request: {op: 'resume', args: {}}}
        : {type: 'action', id: 'pause', label: COPY.pause, sensitive: true,
            request: {op: 'pause', args: {}}});
    if (!paused) {
        items.push({type: 'action', id: 'pause_minute', label: COPY.pauseMinute, sensitive: true,
            request: {op: 'pause', args: {for: '1m'}}});
    }
    if (status.picker_available) {
        items.push({type: 'action', id: 'open_picker', label: COPY.openPicker, sensitive: true,
            request: {op: 'open_picker', args: {}}});
    }
    items.push({type: 'separator'});
    const unpinned = Math.max(0, status.entries - status.pinned);
    items.push({
        type: 'submenu',
        id: 'clear',
        label: COPY.clear,
        sensitive: true,
        items: [
            {type: 'action', id: 'clear_confirm', label: `Delete ${unpinned} unpinned entries`,
                sensitive: unpinned > 0, request: {op: 'clear', args: {all: false}}},
            {type: 'action', id: 'clear_cancel', label: COPY.clearCancel, sensitive: true,
                request: null},
        ],
    });
    return items;
}

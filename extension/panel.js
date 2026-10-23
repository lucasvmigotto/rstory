// @ts-check
// The panel button and its menu, rendered from the menu model. The Shell toolkit (PanelMenu,
// PopupMenu, St) is passed in, so the fixture tests drive this with fakes under plain gjs.
// Everything it creates, connects or schedules is undone by destroy() (FR-008).
// It never spawns a process: "Open picker" is a daemon request.

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

import {Controller} from './controller.js';
import {accessibleName, buildMenu, statusLabel} from './menu.js';

/**
 * @typedef {import('./menu.js').MenuItem} MenuItem
 * @typedef {import('./menu.js').DaemonState} DaemonState
 * @typedef {{PanelMenu: any, PopupMenu: any, St: any}} Toolkit
 * @typedef {{
 *   ui: Toolkit,
 *   iconDir: string,
 *   addToStatusArea: (button: any) => void,
 *   notify: (message: string) => void,
 *   socketPath?: string,
 *   tickMs?: number,
 *   backoffMinMs?: number,
 *   backoffMaxMs?: number,
 * }} PanelOptions
 */

/**
 * A short message for a failed action, by error code only: the daemon's message text can quote
 * content and is never shown. `null` when the failure is not worth a notification.
 * @param {string} _op
 * @param {string} code
 * @returns {string|null}
 */
export function failureMessage(_op, code) {
    switch (code) {
    case 'inactive':
        return 'No clipboard access in this session. Run `rstory doctor` for details.';
    case 'no_terminal':
        return 'No terminal emulator found. Set picker_command in the rstory config.';
    case 'picker_command_failed':
        return 'The picker command could not be started. Check picker_command in the rstory config.';
    case 'backend_error':
        return 'Could not take clipboard ownership. Try again.';
    case 'disconnected':
        return 'The rstory daemon is not running.';
    default:
        return null;
    }
}

export class RstoryPanel {
    /** The PanelMenu.Button. */
    button;
    /** GLib source id of the countdown timer; 0 when none. */
    tickSource = 0;

    #ui;
    #iconDir;
    #tickMs;
    #controller;
    #icon;
    #menuSignal = 0;
    #destroyed = false;
    #signature = '';
    #iconName = '';
    /** @type {any} */
    #statusItem = null;

    /** @param {PanelOptions} opts */
    constructor(opts) {
        this.#ui = opts.ui;
        this.#iconDir = opts.iconDir;
        this.#tickMs = opts.tickMs ?? 1000;
        const {PanelMenu, St} = opts.ui;

        this.button = new PanelMenu.Button(0.5, 'rstory', false);
        this.#icon = new St.Icon({style_class: 'system-status-icon rstory-panel-icon'});
        this.button.add_child(this.#icon);
        this.#menuSignal = this.button.menu.connect('open-state-changed',
            (/** @type {any} */ _menu, /** @type {boolean} */ open) => this.#onOpenChanged(open));
        opts.addToStatusArea(this.button);

        this.#controller = new Controller({
            path: opts.socketPath,
            backoffMinMs: opts.backoffMinMs,
            backoffMaxMs: opts.backoffMaxMs,
            onChange: state => this.#render(state),
            onFailure: (op, code) => {
                const message = failureMessage(op, code);
                if (message)
                    opts.notify(message);
                console.warn(`rstory: ${op} failed (${code})`);
            },
        });
        this.#render(this.#controller.state);
        this.#controller.start();
    }

    /** Undoes everything: socket, reconnect timer, countdown timer, signals, actors. */
    destroy() {
        if (this.#destroyed)
            return;
        this.#destroyed = true;
        this.#stopTick();
        this.#controller.stop();
        if (this.#menuSignal) {
            this.button.menu.disconnect(this.#menuSignal);
            this.#menuSignal = 0;
        }
        this.#statusItem = null;
        this.button.destroy();
    }

    /** @param {boolean} open */
    #onOpenChanged(open) {
        if (this.#destroyed)
            return;
        this.#stopTick();
        if (!open)
            return;
        this.#tick();
        this.tickSource = GLib.timeout_add(GLib.PRIORITY_DEFAULT, this.#tickMs, () => {
            this.#tick();
            return GLib.SOURCE_CONTINUE;
        });
    }

    #stopTick() {
        if (this.tickSource) {
            GLib.source_remove(this.tickSource);
            this.tickSource = 0;
        }
    }

    /** Refreshes the countdown in the status line without rebuilding the menu. */
    #tick() {
        const {status} = this.#controller.state;
        if (this.#statusItem && status)
            this.#statusItem.label.text = statusLabel(status, Date.now());
    }

    /** @param {DaemonState} state */
    #render(state) {
        if (this.#destroyed)
            return;
        this.#setIcon(state);
        this.button.accessible_name = accessibleName(state);

        const items = buildMenu(state, Date.now());
        // The status line ticks in place; everything else rebuilds only when it changed.
        const signature = JSON.stringify(items.map(i => i.type === 'status' ? {...i, label: ''} : i));
        if (signature === this.#signature) {
            this.#tick();
            return;
        }
        this.#signature = signature;
        this.#statusItem = null;
        const menu = this.button.menu;
        menu.removeAll();
        for (const item of items)
            this.#add(menu, item);
    }

    /** @param {DaemonState} state */
    #setIcon(state) {
        const name = state.connected && state.status
            ? (state.status.state === 'paused' ? 'paused' : 'recording')
            : 'disconnected';
        if (name === this.#iconName)
            return;
        this.#iconName = name;
        this.#icon.gicon = new Gio.FileIcon({
            file: Gio.File.new_for_path(GLib.build_filenamev([this.#iconDir, `${name}-symbolic.svg`])),
        });
    }

    /** @param {any} menu @param {MenuItem} item */
    #add(menu, item) {
        const {PopupMenu} = this.#ui;
        if (item.type === 'separator') {
            menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
            return;
        }
        if (item.type === 'submenu') {
            const sub = new PopupMenu.PopupSubMenuMenuItem(item.label);
            for (const child of item.items)
                this.#add(sub.menu, child);
            menu.addMenuItem(sub);
            return;
        }
        const widget = new PopupMenu.PopupMenuItem(item.label);
        widget.setSensitive(item.sensitive);
        if (item.type === 'status')
            this.#statusItem = widget;
        if (item.type === 'entry' || item.type === 'action') {
            widget.connect('activate', () => {
                if (item.request)
                    this.#controller.activate(item.request);
                else
                    this.button.menu.close();
            });
        }
        menu.addMenuItem(widget);
    }
}

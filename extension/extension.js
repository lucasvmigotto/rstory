// @ts-check
// Entry point of the rstory GNOME Shell extension (ADR-0008). A panel icon and menu driven
// entirely by the daemon over its control socket; no history logic, no subprocess, no D-Bus.

import St from 'gi://St';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

import {RstoryPanel} from './panel.js';

export default class RstoryExtension extends Extension {
    /** @type {RstoryPanel|null} */
    _panel = null;

    enable() {
        this._panel = new RstoryPanel({
            ui: {PanelMenu, PopupMenu, St},
            iconDir: `${this.path}/icons`,
            addToStatusArea: button => Main.panel.addToStatusArea(this.uuid, button),
            notify: message => Main.notify('rstory', message),
        });
    }

    disable() {
        this._panel?.destroy();
        this._panel = null;
    }
}

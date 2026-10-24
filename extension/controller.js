// @ts-check
// Glue between the protocol client and the menu model: subscribe, fetch, keep the state current.
// Still no history logic: every value shown comes from `status`, `list` and `subscribe` events.
// No Shell imports, so the fixture tests can drive it under plain gjs.

import {Client} from './client.js';
import {MAX_ENTRIES, initialState} from './menu.js';

/**
 * @typedef {import('./menu.js').DaemonState} DaemonState
 * @typedef {import('./menu.js').MenuRequest} MenuRequest
 * @typedef {import('./client.js').ClientOptions} ClientOptions
 * @typedef {{
 *   path?: string,
 *   backoffMinMs?: number,
 *   backoffMaxMs?: number,
 *   onChange?: (state: DaemonState) => void,
 *   onFailure?: (op: string, code: string) => void,
 * }} ControllerOptions
 */

export class Controller {
    #opts;
    #client;
    /** @type {DaemonState} */
    #state = initialState();
    #stopped = false;
    #refreshing = false;
    #again = false;

    /** @param {ControllerOptions} [opts] */
    constructor(opts = {}) {
        this.#opts = opts;
        this.#client = new Client({
            path: opts.path,
            backoffMinMs: opts.backoffMinMs,
            backoffMaxMs: opts.backoffMaxMs,
            onConnected: () => this.#onConnected(),
            onDisconnected: () => {
                this.#state = {...this.#state, connected: false};
                this.#emit();
            },
            onEvent: (name, data) => this.#onEvent(name, data),
        });
    }

    get state() {
        return this.#state;
    }

    start() {
        this.#stopped = false;
        this.#client.start();
    }

    /** Closes the socket and removes every timer; no callback fires afterwards. */
    stop() {
        this.#stopped = true;
        this.#client.stop();
        this.#state = initialState();
    }

    /**
     * Sends the request behind a menu item. Failures are reported by code only, never content.
     * @param {MenuRequest|null} request
     */
    async activate(request) {
        if (!request)
            return;
        try {
            await this.#client.request(request.op, request.args);
        } catch (e) {
            this.#opts.onFailure?.(request.op, errorCode(e));
        }
    }

    #emit() {
        if (!this.#stopped)
            this.#opts.onChange?.(this.#state);
    }

    async #onConnected() {
        try {
            await this.#client.request('subscribe', {events: ['history_changed', 'pause_changed']});
            await this.#refresh();
        } catch (e) {
            this.#opts.onFailure?.('subscribe', errorCode(e));
        }
    }

    /** @param {string} name @param {any} data */
    #onEvent(name, data) {
        if (name === 'history_changed') {
            this.#refresh();
        } else if (name === 'pause_changed' && this.#state.status && data) {
            // `pause_changed` carries a State: merge it over the cached status.
            const {state, paused_until: pausedUntil, reason} = data;
            this.#state = {
                ...this.#state,
                status: {...this.#state.status, state, paused_until: pausedUntil, reason},
            };
            this.#emit();
        }
    }

    /** Re-reads status and the newest entries; coalesces bursts of events into one extra pass. */
    async #refresh() {
        if (this.#refreshing) {
            this.#again = true;
            return;
        }
        this.#refreshing = true;
        try {
            do {
                this.#again = false;
                const [status, list] = await Promise.all([
                    this.#client.request('status'),
                    this.#client.request('list', {limit: MAX_ENTRIES}),
                ]);
                this.#state = {connected: true, status, entries: list.entries ?? []};
                this.#emit();
            } while (this.#again && !this.#stopped);
        } catch (e) {
            this.#opts.onFailure?.('refresh', errorCode(e));
        } finally {
            this.#refreshing = false;
        }
    }
}

/** @param {unknown} e */
function errorCode(e) {
    return e instanceof Error && 'code' in e ? String(/** @type {any} */ (e).code) : 'internal';
}

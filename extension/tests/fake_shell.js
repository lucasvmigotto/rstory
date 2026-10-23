// @ts-check
// Minimal stand-ins for the Shell toolkit (PanelMenu, PopupMenu, St) that record what the panel
// does, so the fixture tests can check rendering and, above all, the cleanup in disable().

class Signals {
    #next = 1;
    /** @type {Map<number, {name: string, fn: (...args: any[]) => void}>} */
    handlers = new Map();

    /** @param {string} name @param {(...args: any[]) => void} fn */
    connect(name, fn) {
        const id = this.#next++;
        this.handlers.set(id, {name, fn});
        return id;
    }

    /** @param {number} id */
    disconnect(id) {
        this.handlers.delete(id);
    }

    /** @param {string} name @param {any[]} args */
    emit(name, ...args) {
        for (const h of [...this.handlers.values()]) {
            if (h.name === name)
                h.fn(this, ...args);
        }
    }
}

export class FakeItem extends Signals {
    /** @param {string} [text] */
    constructor(text = '') {
        super();
        this.label = {text};
        this.sensitive = true;
        this.destroyed = false;
    }

    /** @param {boolean} s */
    setSensitive(s) {
        this.sensitive = s;
    }

    destroy() {
        this.destroyed = true;
        this.handlers.clear();
    }

    activate() {
        this.emit('activate', null);
    }
}

export class FakeSeparator extends Signals {
    constructor() {
        super();
        this.destroyed = false;
    }

    destroy() {
        this.destroyed = true;
    }
}

export class FakeMenu extends Signals {
    constructor() {
        super();
        /** @type {any[]} */
        this.items = [];
        this.isOpen = false;
        this.closeCalls = 0;
    }

    /** @param {any} item */
    addMenuItem(item) {
        this.items.push(item);
    }

    removeAll() {
        for (const i of this.items)
            i.destroy();
        this.items = [];
    }

    open() {
        this.isOpen = true;
        this.emit('open-state-changed', true);
    }

    close() {
        this.closeCalls++;
        this.isOpen = false;
        this.emit('open-state-changed', false);
    }
}

export class FakeSubMenuItem extends FakeItem {
    /** @param {string} [text] */
    constructor(text = '') {
        super(text);
        this.menu = new FakeMenu();
    }
}

export class FakeIcon extends Signals {
    /** @param {Record<string, any>} [props] */
    constructor(props = {}) {
        super();
        Object.assign(this, props);
        /** @type {any} */
        this.gicon = props.gicon ?? null;
    }
}

export class FakeButton extends Signals {
    /** @param {number} _align @param {string} name @param {boolean} _noMenu */
    constructor(_align, name, _noMenu) {
        super();
        this.name = name;
        this.menu = new FakeMenu();
        /** @type {any[]} */
        this.children = [];
        this.accessible_name = '';
        this.destroyed = false;
    }

    /** @param {any} c */
    add_child(c) {
        this.children.push(c);
    }

    destroy() {
        this.destroyed = true;
        this.menu.handlers.clear();
        this.emit('destroy');
        this.handlers.clear();
    }
}

/** The toolkit bag the panel takes. */
export const fakeUi = /** @type {any} */ ({
    PanelMenu: {Button: FakeButton},
    PopupMenu: {
        PopupMenuItem: FakeItem,
        PopupSeparatorMenuItem: FakeSeparator,
        PopupSubMenuMenuItem: FakeSubMenuItem,
    },
    St: {Icon: FakeIcon},
});

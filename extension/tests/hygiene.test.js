// @ts-check
// Source hygiene the Shell reviewers and ADR-0008 require: every module is type-checked, the
// extension never spawns anything, and it stays away from D-Bus.
import Gio from 'gi://Gio';
import {assert, listDir, repoRoot, runAll, test} from './harness.js';

const DIR = `${repoRoot()}/extension`;
const decoder = new TextDecoder();

/** @param {string} path */
function read(path) {
    const [, bytes] = Gio.File.new_for_path(path).load_contents(null);
    return decoder.decode(bytes);
}

const modules = listDir(DIR).filter(f => f.endsWith('.js') && f !== 'eslint.config.js');
const sources = modules.map(f => ({name: f, text: read(`${DIR}/${f}`)}));

test('the extension has its modules', () => {
    for (const m of ['extension.js', 'client.js', 'controller.js', 'menu.js', 'panel.js'])
        assert(modules.includes(m), `${m} exists`);
});

test('every module starts with // @ts-check', () => {
    for (const s of [...sources, ...listDir(`${DIR}/tests`).filter(f => f.endsWith('.js'))
        .map(f => ({name: `tests/${f}`, text: read(`${DIR}/tests/${f}`)}))])
        assert(s.text.startsWith('// @ts-check\n'), `${s.name} starts with // @ts-check`);
});

test('no process spawning and no D-Bus in the extension modules', () => {
    const banned = [/\bspawn\w*\(/, /\bSubprocess\b/, /\bGtk\b/, /gi:\/\/Gdk/, /\bDBusProxy\b/,
        /\bDBus\.session\b/, /\bGio\.bus_/, /\bGLib\.spawn/];
    for (const s of sources) {
        for (const re of banned)
            assert(!re.test(s.text), `${s.name} matches ${re}`);
    }
});

test('the protocol client is the only module that opens a socket', () => {
    for (const s of sources.filter(x => x.name !== 'client.js'))
        assert(!/SocketClient|UnixSocketAddress/.test(s.text), `${s.name} opens a socket`);
});

await runAll();

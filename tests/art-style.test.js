import { test, afterEach } from 'node:test';
import assert from 'node:assert/strict';
import { get } from 'svelte/store';
import * as characters from '../src/lib/characters.js';
import { readFileSync } from 'node:fs';

const STYLE_IDS = ['anime90', 'pc98', 'msdos', 'taiwan-print', 'gameboy', 'modern-paint'];

const originalStorage = Object.getOwnPropertyDescriptor(globalThis, 'localStorage');
afterEach(() => {
  if (originalStorage) Object.defineProperty(globalThis, 'localStorage', originalStorage);
  else delete globalThis.localStorage;
});

let instance = 0;
const freshPreference = () => import(`../src/lib/art-style.js?test=${++instance}`);
function storageWith(value) {
  const data = new Map(value == null ? [] : [['mj_art_style', value]]);
  Object.defineProperty(globalThis, 'localStorage', {
    configurable: true,
    value: { getItem: (key) => data.get(key) ?? null, setItem: (key, val) => data.set(key, val) },
  });
  return data;
}

test('each style resolves a different complete set without changing identities', () => {
  assert.equal(typeof characters.getCharacterImage, 'function');
  const paths = new Set();
  for (const c of characters.CHARACTERS) {
    for (const style of STYLE_IDS) {
      for (const expression of ['normal', 'happy', 'surprised', 'sad', 'win']) {
        const expected = `/characters/${c.id}/${style === 'anime90' ? '' : style + '/'}${expression}.png`;
        assert.equal(characters.getCharacterImage(c.id, expression, style), expected);
        paths.add(expected);
      }
    }
  }
  assert.equal(paths.size, 330);
});

test('four new men are regular playable characters with dialogue and matching engine identities', () => {
  assert.equal(characters.getSelectableCharacters().length, 10);
  assert.equal(characters.getSelectableCharacters(true).length, 11);
  for (const [id, name] of [['ziheng', '子衡'], ['chengyue', '承岳'], ['zhibai', '知白'], ['youchen', '佑辰']]) {
    const character = characters.getSelectableCharacters().find(c => c.id === id);
    assert.equal(character?.name, name);
    for (const event of Object.keys(characters.EVENT_EXPRESSION)) {
      assert.ok(character.lines[event]?.every(line => typeof line === 'string' && line.trim()), `${id}/${event}`);
      assert.ok(characters.pickLine(id, event), `${id}/${event} reaction`);
    }
  }
  const core = readFileSync(new URL('../crates/mahjong-core/src/room.rs', import.meta.url), 'utf8');
  const registry = core.match(/pub const CHARACTERS:[\s\S]*?= \[([\s\S]*?)\];/)[1];
  const engineCharacters = [...registry.matchAll(/\("([a-z]+)",\s*"([^"]+)"/g)].map(match => [match[1], match[2]]);
  assert.deepEqual(engineCharacters, characters.getSelectableCharacters().map(c => [c.id, c.name]));
});

test('Ayi is hidden from new players but renderable for opponents and unlocked players', () => {
  assert.equal(characters.getCharacter('ayi').name, '阿翼');
  assert.equal(characters.getCharacter('ayi').hidden, true);
  assert.ok(!characters.getSelectableCharacters(false).some(c => c.id === 'ayi'));
  assert.ok(characters.getSelectableCharacters(true).some(c => c.id === 'ayi'));
  assert.equal(characters.getCharacterImage('ayi', 'win', 'pc98'), '/characters/ayi/pc98/win.png');
});

test('unknown style, expression and character have safe portrait fallbacks', () => {
  assert.equal(typeof characters.getCharacterImage, 'function');
  assert.equal(characters.getCharacterImage('xiaocui', 'sad', 'bad'), '/characters/xiaocui/sad.png');
  assert.equal(characters.getCharacterImage('xiaocui', 'bad', 'pc98'), '/characters/xiaocui/pc98/normal.png');
  assert.equal(characters.getCharacterImage('bad', 'win', 'msdos'), '/characters/xiaocui/msdos/win.png');
});

test('first visit defaults to anime, switching notifies subscribers and persists', async () => {
  const data = storageWith(null);
  const { artStyle, setArtStyle, ART_STYLES } = await freshPreference();
  assert.deepEqual(ART_STYLES.map(s => s.id), STYLE_IDS);
  const seen = [];
  const stop = artStyle.subscribe(s => seen.push(s));
  setArtStyle('pc98');
  setArtStyle('msdos');
  stop();
  assert.deepEqual(seen, ['anime90', 'pc98', 'msdos']);
  assert.equal(data.get('mj_art_style'), 'msdos');
});

test('a new instance restores a valid saved style', async () => {
  for (const style of STYLE_IDS) {
    const data = storageWith(null);
    const preference = await freshPreference();
    preference.setArtStyle(style);
    assert.equal(data.get('mj_art_style'), style);
    assert.equal(get((await freshPreference()).artStyle), style);
  }
});

test('only pixel-art styles request nearest-neighbor rendering', async () => {
  const { isPixelStyle } = await freshPreference();
  for (const style of STYLE_IDS) {
    assert.equal(isPixelStyle(style), ['pc98', 'msdos', 'gameboy'].includes(style));
  }
  assert.equal(isPixelStyle('unknown'), false);
});

test('every selectable style has complete PNG artwork on disk', () => {
  function png(path, dimensions) {
    const bytes = readFileSync(new URL(`../public${path}`, import.meta.url));
    assert.equal(bytes.subarray(1, 4).toString(), 'PNG', path);
    if (dimensions) assert.deepEqual([bytes.readUInt32BE(16), bytes.readUInt32BE(20)], dimensions, path);
  }
  for (const style of STYLE_IDS) {
    for (const c of characters.CHARACTERS) {
      for (const expression of ['normal', 'happy', 'surprised', 'sad', 'win']) {
        png(characters.getCharacterImage(c.id, expression, style), [512, 512]);
      }
    }
    png(`/ui/${style}/logo.png`, [1400, 420]);
    png(`/ui/${style}/table.png`, [1600, 1100]);
    png(`/ui/${style}/title-scene.png`);
    png(`/ui/${style}/parlor.png`);
  }
});

test('invalid saved preference falls back; invalid updates are ignored', async () => {
  const data = storageWith('obsolete');
  const { artStyle, setArtStyle } = await freshPreference();
  assert.equal(get(artStyle), 'anime90');
  setArtStyle('pc98');
  setArtStyle('invalid');
  assert.equal(get(artStyle), 'pc98');
  assert.equal(data.get('mj_art_style'), 'pc98');
});

test('blocked storage access does not prevent switching', async () => {
  Object.defineProperty(globalThis, 'localStorage', { configurable: true, get() { throw new Error('blocked'); } });
  const { artStyle, setArtStyle } = await freshPreference();
  assert.equal(get(artStyle), 'anime90');
  assert.doesNotThrow(() => setArtStyle('msdos'));
  assert.equal(get(artStyle), 'msdos');
});

test('a failed persistence write still updates the current screen', async () => {
  Object.defineProperty(globalThis, 'localStorage', {
    configurable: true,
    value: { getItem: () => null, setItem() { throw new Error('quota'); } },
  });
  const { artStyle, setArtStyle } = await freshPreference();
  assert.doesNotThrow(() => setArtStyle('pc98'));
  assert.equal(get(artStyle), 'pc98');
});

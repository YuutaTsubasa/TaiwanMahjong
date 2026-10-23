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
        const expected = `/characters/${c.id}/${style === 'anime90' ? '' : style + '/'}${expression}.webp`;
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
  for (const [id, name] of [['jinye', '金爺'], ['alie', '阿烈'], ['leo', '雷歐'], ['cangya', '蒼牙']]) {
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
  assert.equal(characters.getCharacterImage('ayi', 'win', 'pc98'), '/characters/ayi/pc98/win.webp');
});

test('unknown style, expression and character have safe portrait fallbacks', () => {
  assert.equal(typeof characters.getCharacterImage, 'function');
  assert.equal(characters.getCharacterImage('xiaocui', 'sad', 'bad'), '/characters/xiaocui/sad.webp');
  assert.equal(characters.getCharacterImage('xiaocui', 'bad', 'pc98'), '/characters/xiaocui/pc98/normal.webp');
  assert.equal(characters.getCharacterImage('bad', 'win', 'msdos'), '/characters/xiaocui/msdos/win.webp');
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

// WebP 的寬高：VP8X（有透明等延伸資訊）、VP8L（無損）、VP8（有損）三種格式各放在不同位置
function webpSize(bytes) {
  const kind = bytes.subarray(12, 16).toString();
  if (kind === 'VP8X') return [1 + bytes.readUIntLE(24, 3), 1 + bytes.readUIntLE(27, 3)];
  if (kind === 'VP8L') {
    const b = bytes.readUInt32LE(21);
    return [1 + (b & 0x3fff), 1 + ((b >> 14) & 0x3fff)];
  }
  return [bytes.readUInt16LE(26) & 0x3fff, bytes.readUInt16LE(28) & 0x3fff];
}

test('every selectable style has complete WebP artwork on disk', () => {
  function png(path, dimensions) {
    const bytes = readFileSync(new URL(`../public${path}`, import.meta.url));
    assert.equal(bytes.subarray(0, 4).toString() + bytes.subarray(8, 12).toString(), 'RIFFWEBP', path);
    if (dimensions) assert.deepEqual(webpSize(bytes), dimensions, path);
  }
  for (const style of STYLE_IDS) {
    for (const c of characters.CHARACTERS) {
      for (const expression of ['normal', 'happy', 'surprised', 'sad', 'win']) {
        png(characters.getCharacterImage(c.id, expression, style), [512, 512]);
      }
    }
    png(`/ui/${style}/logo.webp`, [1400, 420]);
    png(`/ui/${style}/table.webp`, [1600, 1100]);
    png(`/ui/${style}/title-scene.webp`);
    png(`/ui/${style}/parlor.webp`);
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

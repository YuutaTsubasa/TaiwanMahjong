import { test, afterEach } from 'node:test';
import assert from 'node:assert/strict';
import { get } from 'svelte/store';
import * as characters from '../src/lib/characters.js';

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
    for (const style of ['anime90', 'pc98', 'msdos']) {
      for (const expression of ['normal', 'happy', 'surprised', 'sad', 'win']) {
        const expected = `/characters/${c.id}/${style === 'anime90' ? '' : style + '/'}${expression}.png`;
        assert.equal(characters.getCharacterImage(c.id, expression, style), expected);
        paths.add(expected);
      }
    }
  }
  assert.equal(paths.size, 90);
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
  assert.deepEqual(ART_STYLES.map(s => s.id), ['anime90', 'pc98', 'msdos']);
  const seen = [];
  const stop = artStyle.subscribe(s => seen.push(s));
  setArtStyle('pc98');
  setArtStyle('msdos');
  stop();
  assert.deepEqual(seen, ['anime90', 'pc98', 'msdos']);
  assert.equal(data.get('mj_art_style'), 'msdos');
});

test('a new instance restores a valid saved style', async () => {
  storageWith('pc98');
  assert.equal(get((await freshPreference()).artStyle), 'pc98');
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

import { test, afterEach } from 'node:test';
import assert from 'node:assert/strict';
import { get } from 'svelte/store';

const originalStorage = Object.getOwnPropertyDescriptor(globalThis, 'localStorage');
afterEach(() => {
  if (originalStorage) Object.defineProperty(globalThis, 'localStorage', originalStorage);
  else delete globalThis.localStorage;
});
let instance = 0;
const fresh = () => import(`../src/lib/unlocks.js?test=${++instance}`);
function storage(value) {
  const data = new Map(value == null ? [] : [['mj_unlock_ayi', value]]);
  Object.defineProperty(globalThis, 'localStorage', {
    configurable: true,
    value: { getItem: key => data.get(key), setItem: (key, value) => data.set(key, value) },
  });
  return data;
}

test('only a completed match with a positive first-place local score wins', async () => {
  const { isWinningMatch } = await fresh();
  const state = {
    phase: 'game_over', you: 0,
    seats: [{ seat: 0, score: 300 }, { seat: 1, score: 100 }, { seat: 2, score: -200 }, { seat: 3, score: -200 }],
  };
  assert.equal(isWinningMatch(state), true);
  assert.equal(isWinningMatch({ ...state, phase: 'result' }), false);
  assert.equal(isWinningMatch({ ...state, phase: 'playing' }), false);
  assert.equal(isWinningMatch({ ...state, you: null }), false);
  assert.equal(isWinningMatch({ ...state, you: 1 }), false);
  assert.equal(isWinningMatch({ ...state, you: 4 }), false);
  assert.equal(isWinningMatch(null), false);
  assert.equal(isWinningMatch({ ...state, seats: state.seats.map(p => ({ ...p, score: 0 })) }), false);
  assert.equal(isWinningMatch({ ...state, seats: [{ seat: 0, score: 200 }, { seat: 1, score: 200 }] }), true);
});

test('unlock notifies once, persists, and restores without exposing a store setter', async () => {
  const data = storage();
  const { ayiUnlocked, unlockAyi } = await fresh();
  const seen = [];
  const stop = ayiUnlocked.subscribe(value => seen.push(value));
  assert.equal(ayiUnlocked.set, undefined);
  assert.equal(unlockAyi(), true);
  assert.equal(unlockAyi(), false);
  stop();
  assert.deepEqual(seen, [false, true]);
  assert.equal(data.get('mj_unlock_ayi'), '1');
  assert.equal(get((await fresh()).ayiUnlocked), true);
});

test('malformed preference stays locked and blocked storage does not prevent unlocking', async () => {
  storage('false');
  assert.equal(get((await fresh()).ayiUnlocked), false);
  Object.defineProperty(globalThis, 'localStorage', {
    configurable: true, get() { throw new Error('blocked storage'); },
  });
  const { ayiUnlocked, unlockAyi } = await fresh();
  assert.equal(get(ayiUnlocked), false);
  assert.equal(unlockAyi(), true);
  assert.equal(get(ayiUnlocked), true);
});

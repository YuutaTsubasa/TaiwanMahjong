import test from 'node:test';
import assert from 'node:assert/strict';
import { resolvePage } from '../src/lib/menu.js';

test('disconnected players can open each menu directly and unknown links return to title', () => {
  for (const page of ['solo', 'host', 'join', 'gallery', 'settings']) {
    assert.equal(resolvePage(`#${page}`, null), page);
  }
  for (const hash of ['', '#', '#title', '#missing', '#lobby', '#table']) {
    assert.equal(resolvePage(hash, null), 'title');
  }
});

test('an active room stays visible even when the URL points to a menu', () => {
  for (const hash of ['', '#host', '#join', '#gallery', '#settings']) {
    assert.equal(resolvePage(hash, { phase: 'lobby' }), 'lobby');
    assert.equal(resolvePage(hash, { phase: 'playing' }), 'table');
    assert.equal(resolvePage(hash, { phase: 'ended' }), 'table');
  }
});

test('leaving a room restores the requested menu without reloading', () => {
  assert.equal(resolvePage('#join', { phase: 'lobby' }), 'lobby');
  assert.equal(resolvePage('#join', null), 'join');
});

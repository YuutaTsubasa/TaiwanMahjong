import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseMML, chordNotes } from '../src/lib/audio/synth.js';
import { SONGS } from '../src/lib/audio/music.js';

test('MML basics: notes, octaves, lengths, dots, ties, loops', () => {
  const { notes, length } = parseMML('o4 l8 c d4 e4. r f&f g+ <a >[b]3');
  assert.deepEqual(
    notes.map((n) => [n.midi, n.dur]),
    [[60, 0.5], [62, 1], [64, 1.5], [65, 1], [68, 0.5], [57, 0.5], [71, 0.5], [71, 0.5], [71, 0.5]],
  );
  assert.equal(length, 0.5 + 1 + 1.5 + 0.5 + 1 + 0.5 + 0.5 + 1.5);
});

test('chord names resolve to the right pitch classes', () => {
  const pcs = (name) => chordNotes(name).map((m) => m % 12);
  assert.deepEqual(pcs('Dmaj7'), [2, 6, 9, 1]);
  assert.deepEqual(pcs('F#m7'), [6, 9, 1, 4]);
  assert.deepEqual(pcs('Bb'), [10, 2, 5]);
});

test('every looping song: all melodic tracks have the same whole number of bars', () => {
  for (const [name, song] of Object.entries(SONGS)) {
    if (!song.loop) continue;
    const lengths = song.tracks.filter((t) => t.mml).map((t) => parseMML(t.mml).length);
    for (const len of lengths) {
      assert.equal(len, lengths[0], `${name}: track lengths differ ${lengths}`);
    }
    assert.equal(lengths[0] % 4, 0, `${name}: ${lengths[0]} beats is not whole 4/4 bars`);
    const chordBeats = song.tracks
      .filter((t) => t.kind === 'chords')
      .map((t) => t.prog.split(/\s+/).reduce((s, tok) => s + (tok.includes(':') ? Number(tok.split(':')[1]) : 4), 0));
    for (const b of chordBeats) assert.equal(b, lengths[0], `${name}: chord progression is ${b} beats`);
  }
});

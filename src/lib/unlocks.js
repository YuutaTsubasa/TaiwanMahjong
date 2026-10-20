import { writable } from 'svelte/store';

const STORAGE_KEY = 'mj_unlock_ayi';
function loadUnlock() {
  try { return localStorage.getItem(STORAGE_KEY) === '1'; } catch { return false; }
}
let unlocked = loadUnlock();
const preference = writable(unlocked);
export const ayiUnlocked = { subscribe: preference.subscribe };

export function unlockAyi() {
  if (unlocked) return false;
  unlocked = true;
  preference.set(true);
  try { localStorage.setItem(STORAGE_KEY, '1'); } catch {}
  return true;
}

// A full match must end; a single winning hand does not count.
// Positive shared first place counts as a win. Spectators cannot unlock by watching.
export function isWinningMatch(state) {
  if (state?.phase !== 'game_over' || state.you == null || !Array.isArray(state.seats)) return false;
  const player = state.seats.find(seat => seat.seat === state.you);
  return !!player && Number.isFinite(player.score) && player.score > 0 &&
    state.seats.every(seat => Number.isFinite(seat.score) && seat.score <= player.score);
}

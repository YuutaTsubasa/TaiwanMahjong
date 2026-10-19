// 角色反應：伺服器送來的事件（碰、胡、放槍…）→ 表情 + 對話泡泡，維持幾秒。
import { clock } from './net.svelte.js';
import { EVENT_EXPRESSION, pickLine } from './characters.js';

export const reactions = $state({});

let lastSeq = null;

/** 每次收到新狀態時呼叫。第一次只記住進度，不重播加入前的舊事件。 */
export function processEvents(state) {
  const events = state?.events ?? [];
  const maxSeq = events.reduce((m, e) => Math.max(m, e.seq), 0);
  if (lastSeq === null || maxSeq < lastSeq) {
    // 剛連上，或伺服器重開（序號歸零）
    lastSeq = maxSeq;
    return;
  }
  for (const e of events) {
    if (e.seq <= lastSeq || !(e.kind in EVENT_EXPRESSION)) continue;
    const character = state.seats[e.seat]?.character;
    const long = e.kind === 'tsumo' || e.kind === 'ron' || e.kind === 'deal_in';
    reactions[e.seat] = {
      expression: EVENT_EXPRESSION[e.kind] ?? 'normal',
      text: pickLine(character, e.kind),
      kind: e.kind,
      until: performance.now() + (long ? 7000 : 3200),
    };
  }
  lastSeq = maxSeq;
}

export function resetReactions() {
  lastSeq = null;
  for (const k of Object.keys(reactions)) delete reactions[k];
}

/** 目前還有效的反應（沒有就回傳 null） */
export function reactionFor(seat) {
  const r = reactions[seat];
  return r && r.until > clock.now ? r : null;
}

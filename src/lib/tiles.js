import { t, isZh, displayName, usesKanji, i18n } from './i18n.svelte.js';

export const NUM_ZH = ['一', '二', '三', '四', '五', '六', '七', '八', '九'];
const HONOR = { E: '東', S: '南', W: '西', N: '北', C: '中', F: '發', P: '白' };
const FLOWERS = '春夏秋冬梅蘭竹菊';
const SUIT_ZH = { m: '萬', p: '筒', s: '條' };

export function parseCode(c) {
  if (!c || c === '?') return { kind: 'back' };
  if (/^[1-9][mps]$/.test(c)) return { kind: c[1], n: +c[0] };
  if (HONOR[c]) return { kind: 'z', ch: HONOR[c], c };
  if (/^f[1-8]$/.test(c)) return { kind: 'f', n: +c[1], ch: FLOWERS[+c[1] - 1] };
  return { kind: 'back' };
}

/** 牌的名稱（無障礙朗讀、滑鼠提示用；牌面本身維持中文） */
export function tileName(c) {
  const p = parseCode(c);
  if (p.kind === 'm' || p.kind === 'p' || p.kind === 's') {
    // 日文的條子叫「索子」：七條 → 七索
    if (usesKanji()) return NUM_ZH[p.n - 1] + (i18n.lang === 'ja' && p.kind === 's' ? '索' : SUIT_ZH[p.kind]);
    return t(`{n}${SUIT_ZH[p.kind]}`, { n: p.n });
  }
  if (p.kind === 'z' || p.kind === 'f') return t(p.ch);
  return t('牌背');
}

export const SEAT_KIND_ICON = { human: '👤', agent: '🤖', bot: '💻' };
export const SEAT_KIND_LABEL = { human: '玩家', agent: 'AI', bot: '電腦' };

/**
 * 牌桌上玩家的顯示名稱。電腦的名字是「角色名(電腦)」，角色名換成該語言的名字；
 * 玩家自己取的名字不翻。
 */
export function seatName(seat) {
  const name = seat?.name;
  if (!name) return name;
  const m = /^(.*)\(電腦\)$/.exec(name);
  if (m) return t('{name}(電腦)', { name: displayName(m[1], seat.character) });
  return displayName(name, seat.character);
}

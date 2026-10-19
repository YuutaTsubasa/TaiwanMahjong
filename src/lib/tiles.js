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

export function tileName(c) {
  const t = parseCode(c);
  if (t.kind === 'm' || t.kind === 'p' || t.kind === 's') return NUM_ZH[t.n - 1] + SUIT_ZH[t.kind];
  if (t.kind === 'z' || t.kind === 'f') return t.ch;
  return '牌背';
}

export const SEAT_KIND_ICON = { human: '👤', agent: '🤖', bot: '💻' };
export const SEAT_KIND_LABEL = { human: '玩家', agent: 'AI', bot: '電腦' };

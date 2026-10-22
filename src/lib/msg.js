// 牌局引擎送來的訊息（見 crates/mahjong-core/src/msg.rs）翻成目前的語言。
//   { k: '{name} 打出 {tile}', a: { name: { p: 2, n: '小翠(電腦)' }, tile: { tile: '3m' } } }
// k 是中文樣板，同時也是翻譯的 key；參數依種類換成該語言的玩家名字、牌名。
import { t } from './i18n.svelte.js';
import { tileName, seatName } from './tiles.js';

/** 把一個參數換成文字 */
function formatArg(v, seats) {
  if (v == null) return '';
  if (typeof v !== 'object') return String(v);
  if ('t' in v) return t(v.t);
  if ('p' in v) {
    const seat = seats?.[v.p];
    // 玩家可能已經離開或換人，名字以訊息當下的為準，角色用現在座位上的
    return seatName({ name: v.n, character: seat?.name === v.n ? seat?.character : undefined }) ?? v.n;
  }
  if ('tile' in v) return tileName(v.tile);
  if ('tiles' in v) return v.tiles.map(tileName).join(t('、'));
  if ('m' in v) return formatMsg(v.m, seats);
  if ('list' in v) return v.list.map((m) => formatMsg(m, seats)).join(v.sep ? t(v.sep) : '');
  return '';
}

/** 一則訊息 → 目前語言的句子；overrides 可以替換某些參數（例如把名字拿掉） */
export function formatMsg(m, seats, overrides = {}) {
  if (!m) return '';
  if (typeof m === 'string') return m;
  const params = {};
  for (const [k, v] of Object.entries(m.a ?? {})) params[k] = k in overrides ? overrides[k] : formatArg(v, seats);
  return t(m.k, params);
}

/** 訊息裡的主角（{name} 參數）是哪個座位 */
export const msgSeat = (m) => (m?.a?.name && typeof m.a.name === 'object' && 'p' in m.a.name ? m.a.name.p : null);

/**
 * 跟 formatMsg 一樣，但把牌拆出來：回傳 [{ text }, { tile: '3m' }, ...]，
 * 介面可以把牌畫成小牌（英文的牌名跟牌面對不起來，看圖比較直覺）。
 */
export function msgParts(m, seats) {
  if (!m) return [];
  if (typeof m === 'string') return [{ text: m }];
  const out = [];
  const push = (p) => {
    const last = out[out.length - 1];
    if (p.text != null && last?.text != null) last.text += p.text;
    else if (p.text !== '') out.push(p);
  };
  const template = t(m.k);
  const pieces = template.split(/\{(\w+)\}/);
  pieces.forEach((piece, i) => {
    if (i % 2 === 0) return push({ text: piece });
    const v = m.a?.[piece];
    if (v && typeof v === 'object' && 'tile' in v) push({ tile: v.tile });
    else if (v && typeof v === 'object' && 'tiles' in v) v.tiles.forEach((c, j) => { if (j) push({ text: t('、') }); push({ tile: c }); });
    else if (v && typeof v === 'object' && 'm' in v) msgParts(v.m, seats).forEach(push);
    else push({ text: v === undefined ? `{${piece}}` : formatArg(v, seats) });
  });
  return out;
}

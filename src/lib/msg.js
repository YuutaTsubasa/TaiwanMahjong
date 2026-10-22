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

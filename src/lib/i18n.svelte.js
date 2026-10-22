// 多國語言。原文（繁體中文）本身就是翻譯的 key：
//   t('開始遊戲')                          → 'Start game'
//   t('恭喜 {name} 拿下第一！', { name })   → 'Congratulations, {name} takes first place!'
// 翻譯放在 src/lib/i18n/<語言>/*.js（各檔匯出一個 { 原文: 譯文 } 物件，會自動合併）。
// 找不到翻譯就顯示原文，所以漏翻的地方只會顯示中文，不會壞掉。
//
// 牌局引擎送來的文字（牌局紀錄、胡牌說明、錯誤訊息）目前還是中文，之後改成代碼再翻。

import { CHARACTERS } from './characters.js';

const STORAGE_KEY = 'mj_lang';
const CHARACTER_NAMES = Object.fromEntries(CHARACTERS.map((c) => [c.id, c.name]));

export const LANGUAGES = [
  { id: 'zh-TW', name: '中文', htmlLang: 'zh-Hant-TW' },
  { id: 'en', name: 'English', htmlLang: 'en' },
  { id: 'ja', name: '日本語', htmlLang: 'ja' },
];

const modules = import.meta.glob('./i18n/*/*.js', { eager: true });
const DICTS = {};
for (const [path, mod] of Object.entries(modules)) {
  const lang = path.split('/')[2];
  DICTS[lang] = { ...(DICTS[lang] ?? {}), ...mod.default };
}

const isLang = (id) => LANGUAGES.some((l) => l.id === id);

function initial() {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (isLang(saved)) return saved;
  } catch {}
  // 沒選過就看瀏覽器：第一個認得的語言（中文任何地區都用中文），都不認得就用英文
  const nav = typeof navigator !== 'undefined' ? navigator.languages ?? [navigator.language] : [];
  if (nav.length === 0) return 'zh-TW';
  for (const l of nav) {
    if (/^zh/i.test(l ?? '')) return 'zh-TW';
    if (/^ja/i.test(l ?? '')) return 'ja';
    if (/^en/i.test(l ?? '')) return 'en';
  }
  return 'en';
}

export const i18n = $state({ lang: initial() });

export function setLang(id) {
  if (!isLang(id)) return;
  i18n.lang = id;
  try { localStorage.setItem(STORAGE_KEY, id); } catch {}
}

export const htmlLang = () => LANGUAGES.find((l) => l.id === i18n.lang)?.htmlLang ?? 'zh-Hant-TW';

/** 翻譯一段原文；{name} 之類的位置用 params 代入 */
export function t(text, params) {
  if (text == null) return '';
  const dict = DICTS[i18n.lang];
  let out = (dict && dict[text]) ?? text;
  if (params) out = out.replace(/\{(\w+)\}/g, (m, k) => (params[k] ?? m));
  return out;
}

/**
 * 玩家顯示名稱：電腦（或沒改名的玩家）用的是角色的中文名字，就換成該語言的角色名；
 * 玩家自己取的名字不翻。
 */
export function displayName(name, characterId) {
  if (!name) return name;
  // 電腦玩家的名字是「小翠(電腦)」
  const cpu = /^(.*)\(電腦\)$/.exec(name);
  if (cpu) return t('{name}(電腦)', { name: displayName(cpu[1], characterId) });
  const zh = CHARACTER_NAMES[characterId];
  return zh && zh === name ? t(name) : name;
}

/** 牌名、風向直接用漢字的語言（中文、日文）；其他語言用翻譯的名字、牌畫成小牌 */
export const usesKanji = () => i18n.lang === 'zh-TW' || i18n.lang === 'ja';

/** 現在是不是中文介面（有些地方中文、英文排法不一樣） */
export const isZh = () => i18n.lang === 'zh-TW';

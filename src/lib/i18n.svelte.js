// 多國語言。原文（繁體中文）本身就是翻譯的 key：
//   t('開始遊戲')                          → 'Start game'
//   t('恭喜 {name} 拿下第一！', { name })   → 'Congratulations, {name} takes first place!'
// 翻譯放在 src/lib/i18n/<語言>/*.js（各檔匯出一個 { 原文: 譯文 } 物件，會自動合併）。
// 找不到翻譯就顯示原文，所以漏翻的地方只會顯示中文，不會壞掉。
// 語言偵測、字典合併、代入參數這些共用的部分在 VirtualCore（virtual-core/i18n/lang.js）。
//
// 牌局引擎送來的文字（牌局紀錄、胡牌說明、錯誤訊息）目前還是中文，之後改成代碼再翻。

import { LANGUAGES, isLang, htmlLangOf, mergeDicts, loadLang, saveLang, translate } from 'virtual-core/i18n/lang.js';
import { CHARACTERS } from './characters.js';

export { LANGUAGES };

const STORAGE_KEY = 'mj_lang';
const CHARACTER_NAMES = Object.fromEntries(CHARACTERS.map((c) => [c.id, c.name]));

const DICTS = mergeDicts(import.meta.glob('./i18n/*/*.js', { eager: true }));

export const i18n = $state({ lang: loadLang(STORAGE_KEY) });

export function setLang(id) {
  if (!isLang(id)) return;
  i18n.lang = id;
  saveLang(STORAGE_KEY, id);
}

export const htmlLang = () => htmlLangOf(i18n.lang);

/** 翻譯一段原文；{name} 之類的位置用 params 代入 */
export function t(text, params) {
  return translate(DICTS[i18n.lang], text, params);
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

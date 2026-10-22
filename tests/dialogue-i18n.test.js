// 每句角色台詞都要有各語言的翻譯（新增角色或改台詞時，記得補 src/lib/i18n/<語言>/dialogue.js）
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { CHARACTERS } from '../src/lib/characters.js';
import en from '../src/lib/i18n/en/dialogue.js';
import ja from '../src/lib/i18n/ja/dialogue.js';

for (const [lang, dialogue] of Object.entries({ en, ja })) {
  test(`every character line has a ${lang} translation`, () => {
    const missing = [];
    for (const c of CHARACTERS) {
      for (const [event, lines] of Object.entries(c.lines ?? {})) {
        for (const line of lines) {
          if (!dialogue[line]?.trim()) missing.push(`${c.id}/${event}: ${line}`);
        }
      }
    }
    assert.deepEqual(missing, [], `缺少 ${lang} 台詞：\n${missing.join('\n')}`);
  });
}

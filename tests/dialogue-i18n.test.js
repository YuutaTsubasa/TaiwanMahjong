// 每句角色台詞都要有英文翻譯（新增角色或改台詞時，記得補 src/lib/i18n/en/dialogue.js）
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { CHARACTERS } from '../src/lib/characters.js';
import dialogue from '../src/lib/i18n/en/dialogue.js';

test('every character line has an English translation', () => {
  const missing = [];
  for (const c of CHARACTERS) {
    for (const [event, lines] of Object.entries(c.lines ?? {})) {
      for (const line of lines) {
        if (!dialogue[line]?.trim()) missing.push(`${c.id}/${event}: ${line}`);
      }
    }
  }
  assert.deepEqual(missing, [], `缺少英文台詞：\n${missing.join('\n')}`);
});

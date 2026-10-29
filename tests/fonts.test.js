// 附帶的字型（src/fonts）只收了原始碼用到的字：新加的介面文字有字沒收到，就要重產字型
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { missingChars } from 'virtual-core/fonts/chars.js';

test('bundled fonts cover every character in the source', () => {
  const missing = missingChars(fileURLToPath(new URL('..', import.meta.url)));
  assert.deepEqual(missing, {}, '有字不在字型裡，請跑 npm run fonts');
});

// 元件編譯時不能有無障礙或反應性警告。手把／鍵盤焦點導覽的純函式在 VirtualCore（virtual-core/input/controls.js），測試也在那邊。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { compile } from 'svelte/compiler';

test('Controls compiles without accessibility or reactivity warnings', async () => {
  const source = await readFile(new URL('../src/lib/Controls.svelte', import.meta.url), 'utf8');
  const result = compile(source, { filename: 'Controls.svelte', generate: 'client' });
  assert.deepEqual(result.warnings.map(warning => `${warning.code}: ${warning.message}`), []);
});

test('controller keyboard compiles without accessibility or reactivity warnings', async () => {
  const source = await readFile(new URL('../src/lib/ControlsKeyboard.svelte', import.meta.url), 'utf8');
  const result = compile(source, { filename: 'ControlsKeyboard.svelte', generate: 'client' });
  assert.deepEqual(result.warnings.map(warning => `${warning.code}: ${warning.message}`), []);
});

test('in-app confirmation compiles without accessibility or reactivity warnings', async () => {
  const source = await readFile(new URL('../src/lib/ConfirmDialog.svelte', import.meta.url), 'utf8');
  const result = compile(source, { filename: 'ConfirmDialog.svelte', generate: 'client' });
  assert.deepEqual(result.warnings.map(warning => `${warning.code}: ${warning.message}`), []);
});

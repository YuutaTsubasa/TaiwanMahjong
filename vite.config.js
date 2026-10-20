import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  // GitHub Pages 版放在 /TaiwanMahjong/ 子路徑；App 與 headless 伺服器維持根路徑
  base: process.env.PAGES_BASE || '/',
  // 單機版的 Web Worker 會載入 WebAssembly，用 ES module 格式比較單純
  worker: { format: 'es' },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    watch: { ignored: ['**/src-tauri/**'] },
  },
});

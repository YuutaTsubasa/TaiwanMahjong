<script>
  // 語言切換：上方工具列（畫風旁邊）用的小下拉選單
  import { LANGUAGES, i18n, setLang, t } from './i18n.svelte.js';
</script>

<label class="lang-picker" title={t('語言')}>
  <span aria-hidden="true">🌐</span>
  <select aria-label={t('語言')} value={i18n.lang} onchange={(e) => setLang(e.currentTarget.value)}>
    {#each LANGUAGES as l}<option value={l.id}>{l.name}</option>{/each}
  </select>
</label>

<style>
  .lang-picker { display: flex; align-items: center; gap: 4px; padding: 5px 10px; background: rgba(7, 26, 18, 0.92); border-bottom: 1px solid var(--line); font-size: 13px; color: var(--muted); }
  select { min-height: 30px; padding: 2px 6px; font-size: 13px; }
</style>

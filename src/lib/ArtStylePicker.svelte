<script>
  import { ART_STYLES, artStyle, setArtStyle } from './art-style.js';
  import { t } from './i18n.svelte.js';
</script>

<div class="style-picker" role="group" aria-label={t('整體風格：介面與角色')}>
  <span class="label">{t('整體風格')}</span>
  <select data-gamepad-start aria-label={t('整體風格：介面與角色')} value={$artStyle} onchange={(event) => setArtStyle(event.currentTarget.value)}>
    {#each ART_STYLES as style}<option value={style.id}>{t(style.name)}</option>{/each}
  </select>
  {#each ART_STYLES as style}
    <button
      type="button"
      class:selected={$artStyle === style.id}
      data-gamepad-start={$artStyle === style.id ? '' : undefined}
      aria-pressed={$artStyle === style.id}
      title={t(style.description)}
      onclick={() => setArtStyle(style.id)}
    >{t(style.name)}</button>
  {/each}
</div>

<style>
  .style-picker {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    padding: 7px 16px;
    background: rgba(7, 26, 18, 0.92);
    border-bottom: 1px solid var(--line);
  }
  .label { font-size: 12px; color: var(--muted); margin-right: 6px; }
  select { display: none; min-width: 0; }
  button { padding: 5px 12px; font-size: 12px; border-radius: 7px; }
  button.selected { color: var(--gold); border-color: var(--gold); background: rgba(255, 209, 102, 0.15); }
  button:focus-visible { outline: 2px solid var(--gold); outline-offset: 2px; }
  /* 按鈕放不下一行（或畫面很矮）時改成下拉選單；之後畫風變多也預留了空間 */
  @media (max-width: 1100px), (max-height: 620px) {
    .style-picker { flex-wrap: nowrap; padding: 5px 12px; }
    .label { white-space: nowrap; }
    button { display: none; }
    select { display: block; flex: 1; min-height: 36px; padding: 5px 10px; }
  }
</style>

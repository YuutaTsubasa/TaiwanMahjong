<script>
  import { ART_STYLES, artStyle, setArtStyle, isPixelStyle } from './art-style.js';
  import { asset } from './site.js';
  import { soundSettings, setVolume, toggleMute } from './audio/sound.svelte.js';
  import Avatar from './Avatar.svelte';
  import Brand from './Brand.svelte';
  import OrnateHeading from './ui/OrnateHeading.svelte';
  import SectionBanner from './ui/SectionBanner.svelte';
  import ToggleSwitch from './ui/ToggleSwitch.svelte';
  import { autoHints, setAutoHints } from './hints.js';
  import FancyButton from './ui/FancyButton.svelte';
  import { t } from './i18n.svelte.js';
</script>

<main class="concept-page settings-page">
  <header class="page-header">
    <Brand compact />
    <a class="back-link" href="#title">← {t('返回標題')}</a>
  </header>
  <div class="settings-layout">
    <section class="art-panel settings-panel" aria-labelledby="settings-heading">
      <OrnateHeading text={t('遊戲設定')} id="settings-heading" />
      <SectionBanner text={t('畫面風格')} />
      <div class="style-options" role="group" aria-label={t('畫面風格')}>
        {#each ART_STYLES as style}
          <button type="button" class="style-option" class:selected={$artStyle === style.id} aria-pressed={$artStyle === style.id} onclick={() => setArtStyle(style.id)}>
            <img src={asset(`/ui/${style.id}/parlor.png`)} alt={t('{style}麻將館預覽', { style: t(style.name) })} class:pixel={isPixelStyle(style.id)} />
            <span>{t(style.name)}</span>
            {#if $artStyle === style.id}<b class="check" aria-hidden="true">✓</b>{/if}
          </button>
        {/each}
      </div>
      <SectionBanner text={t('聲音與輔助')} />
      <div class="audio-settings">
        <label class="volume-row">
          <span>{t('音樂音量')}</span>
          <input type="range" min="0" max="1" step="0.05" value={soundSettings.bgm} oninput={(event) => setVolume('bgm', Number(event.currentTarget.value))} />
          <output>{Math.round(soundSettings.bgm * 100)}%</output>
        </label>
        <label class="volume-row">
          <span>{t('音效音量')}</span>
          <input type="range" min="0" max="1" step="0.05" value={soundSettings.sfx} oninput={(event) => setVolume('sfx', Number(event.currentTarget.value))} />
          <output>{Math.round(soundSettings.sfx * 100)}%</output>
        </label>
        <div class="toggle-pair">
          <div class="mute-row"><span>{t('靜音')}</span><ToggleSwitch label={t('靜音')} checked={soundSettings.muted} onchange={toggleMute} /></div>
          <div class="mute-row" title={t('輪到你做決定時，自動彈出提示框（建議打哪張、要不要吃碰）；牌桌上的「提示」按鈕隨時可以打開')}><span>{t('自動提示')}</span><ToggleSwitch label={t('自動彈出提示')} checked={$autoHints} onchange={setAutoHints} /></div>
        </div>
      </div>
      <p class="save-note">— {t('設定會自動儲存。')} —</p>
      <div class="done"><FancyButton variant="red" href="#title">{t('完　成')}</FancyButton></div>
    </section>
    <aside class="settings-host" aria-label={t('麻將館老闆娘蘭姐')}>
      <Avatar id="lanjie" size={540} />
      <p class="art-panel">{t('調好喜歡的風格，喝杯茶再來。')}</p>
    </aside>
  </div>
</main>

<style>
  .settings-page { padding: clamp(8px, 1.8vh, 34px) clamp(16px, 2.2vw, 34px); background-image: linear-gradient(90deg, #021b1866, transparent), var(--scene-parlor); }
  .page-header { display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-bottom: clamp(6px, 1.6vh, 20px); min-height: clamp(44px, 7vh, 58px); }
  .back-link { color: var(--gold); text-decoration: none; border: 1px solid var(--line); padding: 10px 18px; background: #071c18d9; white-space: nowrap; }
  .settings-layout { display: grid; grid-template-columns: minmax(0, 64%) minmax(0, 1fr); gap: 16px; align-items: end; }
  .settings-panel { padding: clamp(10px, 2.2vh, 40px) clamp(20px, 2.5vw, 40px); }
  h2 { display: flex; align-items: center; gap: 18px; color: var(--gold); font: 600 22px 'DFKai-SB', '標楷體', serif; letter-spacing: .18em; margin: 22px 0 16px; }
  h2::after { content: ''; flex: 1; height: 1px; background: var(--line); }
  /* 畫風：桌面版固定排成一列（畫風變多時每格自動變窄），才不會把下面的設定擠出畫面 */
  .style-options { display: grid; grid-auto-flow: column; grid-auto-columns: minmax(0, 1fr); gap: clamp(6px, 1vw, 14px); }
  .style-option { display: flex; flex-direction: column; padding: 0; position: relative; overflow: hidden; border-radius: 2px; border: 1px solid #a98745; background: #021d18; }
  .style-option img { width: 100%; aspect-ratio: 1.7; height: clamp(52px, 8vh, 160px); object-fit: cover; }
  .style-option img.pixel { image-rendering: pixelated; }
  .style-option span { padding: clamp(4px, 1vh, 9px) 2px; font: 600 clamp(15px, 1.2vw, 19px) 'DFKai-SB', '標楷體', serif; color: var(--gold); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .style-option.selected { box-shadow: 0 0 0 2px #e4be60, 0 0 14px #e8c45e77; }
  .check { position: absolute; right: 7px; top: 7px; display: grid; place-items: center; width: 28px; height: 28px; border-radius: 50%; border: 2px solid #fff0bd; background: #8e681c; color: #fff0bd; font-size: 18px; }
  .audio-settings { display: grid; gap: clamp(6px, 1.6vh, 18px); }
  .volume-row { display: grid; grid-template-columns: 100px minmax(0, 1fr) 48px; gap: 18px; align-items: center; }
  .volume-row > span, .mute-row > span { color: var(--text); font-size: 17px; letter-spacing: .08em; }
  .volume-row input { width: 100%; padding: 0; accent-color: var(--gold); min-height: 28px; }
  output { color: var(--gold); text-align: right; font-variant-numeric: tabular-nums; }
  .mute-row { display: grid; grid-template-columns: 100px 1fr; gap: 18px; align-items: center; }
  /* 靜音與新手提示排在同一行 */
  .toggle-pair { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
  .toggle-pair .mute-row { grid-template-columns: auto 1fr; gap: 12px; }
  .toggle-pair .mute-row:first-child { grid-template-columns: 100px 1fr; gap: 18px; }
  .mute-toggle { width: fit-content; min-width: 110px; }
  .mute-toggle[aria-pressed='true'] { background: #68412e; border-color: var(--gold); }
  .save-note { text-align: center; color: var(--muted); font-size: 13px; margin: clamp(6px, 1.8vh, 20px) 0 clamp(6px, 1.4vh, 16px); }
  .done { display: flex; justify-content: center; }
  .done :global(.fancy) { min-width: min(320px, 100%); }
  .done-link { display: block; width: min(280px, 100%); margin: auto; border: 1px solid #c6a256; outline: 1px solid #785d2e; outline-offset: 3px; padding: 12px 30px; background: #5f3529; color: #ffe6b2; text-decoration: none; text-align: center; font: 600 25px 'DFKai-SB', '標楷體', serif; letter-spacing: .3em; }
  .settings-host { min-width: 0; }
  .settings-host :global(.avatar) { width: min(100%, calc(100vh - 260px)) !important; height: auto !important; aspect-ratio: 1; margin: 0 auto; filter: drop-shadow(0 4px 12px #000a); }
  .settings-host :global(img) { object-fit: contain; }
  .settings-host p { padding: 18px; line-height: 1.7; margin: 0 0 10px; text-align: center; }
  @media (max-width: 900px) {
    .style-options { grid-auto-flow: row; grid-template-columns: repeat(3, minmax(0, 1fr)); }
  }
  @media (max-width: 800px) {
    .settings-layout { grid-template-columns: 1fr; }
    .settings-host { display: none; }
  }
  /* 手機橫向（畫面很矮）：畫風在左、聲音與完成在右，整頁不用捲動 */
  @media (max-height: 620px) and (orientation: landscape) {
    .settings-page { padding: 4px 10px; }
    .page-header { min-height: 34px; margin-bottom: 4px; }
    .page-header :global(.logo) { max-height: 30px; width: auto; }
    .back-link { padding: 3px 12px; font-size: 13px; }
    .settings-layout { grid-template-columns: 1fr; }
    .settings-host { display: none; }
    .settings-panel { display: grid; grid-template-columns: minmax(0, 1.3fr) minmax(0, 1fr); grid-template-rows: auto auto auto auto 1fr; column-gap: 18px; align-items: start; padding: 6px 14px; }
    .settings-panel > :global(.ornate) { grid-column: 1 / -1; margin: 0; }
    .settings-panel > :global(.ornate .text) { font-size: 20px; }
    .settings-panel > :global(.section-banner:nth-child(2)) { grid-column: 1; grid-row: 2; }
    .style-options { grid-column: 1; grid-row: 3 / span 3; grid-auto-flow: row; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 6px; }
    .settings-panel > :global(.section-banner:nth-child(4)) { grid-column: 2; grid-row: 2; }
    .audio-settings { grid-column: 2; grid-row: 3; gap: 2px; }
    .save-note { grid-column: 2; grid-row: 4; margin: 2px 0; font-size: 11px; }
    .volume-row input { min-height: 24px; }
    .settings-panel :global(.section-banner) { margin-block: 2px; }
    .done { grid-column: 2; grid-row: 5; }
    .style-option img { height: clamp(26px, 10vh, 56px); }
    .style-option span { padding: 2px; font-size: 13px; }
    .volume-row { grid-template-columns: 64px minmax(0, 1fr) 40px; gap: 8px; }
    .mute-row { grid-template-columns: 64px 1fr; gap: 8px; }
    .volume-row > span, .mute-row > span { font-size: 14px; }
    .done :global(.fancy) { min-height: 32px; min-width: 0; font-size: 16px; padding: 2px 18px; }
  }
  @media (max-height: 400px) and (orientation: landscape) {
    .page-header { min-height: 28px; margin-bottom: 2px; }
    .settings-panel { padding: 4px 12px; }
    .settings-panel > :global(.ornate .text) { font-size: 17px; }
    .style-option img { height: clamp(22px, 8vh, 40px); }
    .save-note { display: none; }
  }
  @media (max-width: 500px) {
    .page-header { flex-wrap: wrap; }
    .style-options { grid-auto-flow: row; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
    .style-option span { font-size: 15px; }
    .volume-row { grid-template-columns: 74px minmax(0, 1fr) 38px; gap: 10px; }
    .volume-row > span, .mute-row > span { font-size: 14px; }
    .mute-row { grid-template-columns: 74px 1fr; gap: 10px; }
    .settings-panel { padding: 18px; }
  }
</style>

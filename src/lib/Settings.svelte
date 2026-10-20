<script>
  import { ART_STYLES, artStyle, setArtStyle, isPixelStyle } from './art-style.js';
  import { asset } from './site.js';
  import { soundSettings, setVolume, toggleMute } from './audio/sound.svelte.js';
  import Avatar from './Avatar.svelte';
  import Brand from './Brand.svelte';
  import OrnateHeading from './ui/OrnateHeading.svelte';
  import SectionBanner from './ui/SectionBanner.svelte';
  import ToggleSwitch from './ui/ToggleSwitch.svelte';
  import FancyButton from './ui/FancyButton.svelte';
</script>

<main class="concept-page settings-page">
  <header class="page-header">
    <Brand compact />
    <a class="back-link" href="#title">← 返回標題</a>
  </header>
  <div class="settings-layout">
    <section class="art-panel settings-panel" aria-labelledby="settings-heading">
      <OrnateHeading text="遊戲設定" id="settings-heading" />
      <SectionBanner text="畫面風格" />
      <div class="style-options" role="group" aria-label="畫面風格">
        {#each ART_STYLES as style}
          <button type="button" class="style-option" class:selected={$artStyle === style.id} aria-pressed={$artStyle === style.id} onclick={() => setArtStyle(style.id)}>
            <img src={asset(`/ui/${style.id}/parlor.png`)} alt="{style.name}麻將館預覽" class:pixel={isPixelStyle(style.id)} />
            <span>{style.name}</span>
            {#if $artStyle === style.id}<b class="check" aria-hidden="true">✓</b>{/if}
          </button>
        {/each}
      </div>
      <SectionBanner text="聲音設定" />
      <div class="audio-settings">
        <label class="volume-row">
          <span>音樂音量</span>
          <input type="range" min="0" max="1" step="0.05" value={soundSettings.bgm} oninput={(event) => setVolume('bgm', Number(event.currentTarget.value))} />
          <output>{Math.round(soundSettings.bgm * 100)}%</output>
        </label>
        <label class="volume-row">
          <span>音效音量</span>
          <input type="range" min="0" max="1" step="0.05" value={soundSettings.sfx} oninput={(event) => setVolume('sfx', Number(event.currentTarget.value))} />
          <output>{Math.round(soundSettings.sfx * 100)}%</output>
        </label>
        <div class="mute-row"><span>靜音</span><ToggleSwitch label="靜音" checked={soundSettings.muted} onchange={toggleMute} /></div>
      </div>
      <p class="save-note">— 設定會自動儲存。—</p>
      <div class="done"><FancyButton variant="red" href="#title">完　成</FancyButton></div>
    </section>
    <aside class="settings-host" aria-label="麻將館老闆娘蘭姐">
      <Avatar id="lanjie" size={540} />
      <p class="art-panel">調好喜歡的風格，喝杯茶再來。</p>
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
  /* 六種畫風：寬螢幕排成一列，比較窄時三欄兩列，才不會把下面的設定擠出畫面 */
  .style-options { display: grid; grid-template-columns: repeat(6, minmax(0, 1fr)); gap: clamp(8px, 1vw, 14px); }
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
    .style-options { grid-template-columns: repeat(3, minmax(0, 1fr)); }
  }
  @media (max-width: 800px) {
    .settings-layout { grid-template-columns: 1fr; }
    .settings-host { display: none; }
  }
  @media (max-width: 500px) {
    .page-header { flex-wrap: wrap; }
    .style-options { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
    .style-option span { font-size: 15px; }
    .volume-row { grid-template-columns: 74px minmax(0, 1fr) 38px; gap: 10px; }
    .volume-row > span, .mute-row > span { font-size: 14px; }
    .mute-row { grid-template-columns: 74px 1fr; gap: 10px; }
    .settings-panel { padding: 18px; }
  }
</style>

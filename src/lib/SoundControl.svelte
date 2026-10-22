<script>
  import { soundSettings, setVolume, toggleMute } from './audio/sound.svelte.js';
  import { t } from './i18n.svelte.js';
</script>

<div class="sound" role="group" aria-label={t('聲音設定')}>
  <label title={t('背景音樂')}>
    <span>🎵 {t('音樂')}</span>
    <input
      type="range"
      min="0"
      max="1"
      step="0.05"
      value={soundSettings.bgm}
      oninput={(e) => setVolume('bgm', Number(e.currentTarget.value))}
    />
  </label>
  <label title={t('音效')}>
    <span>🔔 {t('音效')}</span>
    <input
      type="range"
      min="0"
      max="1"
      step="0.05"
      value={soundSettings.sfx}
      oninput={(e) => setVolume('sfx', Number(e.currentTarget.value))}
    />
  </label>
  <button type="button" class:muted={soundSettings.muted} onclick={toggleMute} aria-pressed={soundSettings.muted}>
    {soundSettings.muted ? `🔇 ${t('已靜音')}` : '🔊'}
  </button>
</div>

<style>
  .sound {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 7px 16px;
    background: rgba(7, 26, 18, 0.92);
    border-bottom: 1px solid var(--line);
  }
  label { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--muted); white-space: nowrap; }
  .sound { flex-wrap: wrap; }
  input[type='range'] { width: 80px; padding: 0; accent-color: var(--gold); }
  button { padding: 4px 10px; font-size: 12px; border-radius: 7px; }
  button.muted { color: var(--red); border-color: var(--red); }
</style>

<script>
  // 區段標題：緞帶底的書法字，後面接一條金線（設定頁「畫面風格」「聲音設定」）
  let { text, tag = 'h2' } = $props();
</script>

<div class="section-banner">
  <svelte:element this={tag} class="ribbon">{text}</svelte:element>
  <span class="rule" aria-hidden="true"></span>
</div>

<style>
  .section-banner { display: flex; align-items: center; gap: 0; margin: clamp(8px, 2vh, 20px) 0 clamp(6px, 1.4vh, 14px); }
  .ribbon {
    margin: 0;
    padding: clamp(2px, 0.6vh, 6px) 34px;
    font-family: var(--heading-font);
    font-size: clamp(16px, min(1.6vw, 2.8vh), 25px);
    font-weight: 700;
    letter-spacing: 0.3em;
    color: var(--gold);
    background: linear-gradient(#5a2f2a, #3b1d1b);
    border: 1px solid var(--art-line);
    box-shadow: inset 0 0 0 3px #0008, inset 0 0 0 4px color-mix(in srgb, var(--art-line) 60%, transparent);
    clip-path: polygon(10px 0, calc(100% - 10px) 0, 100% 50%, calc(100% - 10px) 100%, 10px 100%, 0 50%);
  }
  :global(html[data-ui-style='pc98']) .ribbon { background: #4b365b; }
  :global(html[data-ui-style='msdos']) .ribbon { background: #00aaaa; color: #001717; }
  :global(html[lang='en']) .ribbon { letter-spacing: .06em; }
  .rule { flex: 1; height: 1px; background: linear-gradient(90deg, var(--art-line), transparent); }
</style>

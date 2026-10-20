<script>
  // 置中的書法標題，左右有金色雲紋，可加一行副標（例如「選好角色，準備入席。」）
  let { text, sub = '', size = 'lg', tag = 'h1', id = undefined } = $props();
</script>

<div class="ornate {size}">
  <div class="row">
    <svg class="orn" viewBox="0 0 60 20" aria-hidden="true">
      <path d="M2 10 H30" />
      <path d="M30 10 C36 2 46 2 48 8 C50 14 42 16 40 11 C39 8 43 7 44 9" />
      <circle cx="55" cy="10" r="2.2" />
    </svg>
    <svelte:element this={tag} {id} class="text">{text}</svelte:element>
    <svg class="orn flip" viewBox="0 0 60 20" aria-hidden="true">
      <path d="M2 10 H30" />
      <path d="M30 10 C36 2 46 2 48 8 C50 14 42 16 40 11 C39 8 43 7 44 9" />
      <circle cx="55" cy="10" r="2.2" />
    </svg>
  </div>
  {#if sub}<p class="sub">{sub}</p>{/if}
</div>

<style>
  .ornate { text-align: center; }
  .row { display: flex; align-items: center; justify-content: center; gap: 0.5em; }
  .text {
    margin: 0;
    font-family: var(--heading-font);
    font-weight: 700;
    color: var(--gold);
    letter-spacing: 0.18em;
    text-shadow: 0 2px 0 #0009, 0 0 12px color-mix(in srgb, var(--gold) 25%, transparent);
    white-space: nowrap;
  }
  .lg .text { font-size: clamp(24px, min(3.6vw, 4.8vh), 56px); }
  .md .text { font-size: clamp(22px, min(2.6vw, 4.2vh), 40px); }
  .sm .text { font-size: clamp(18px, min(1.7vw, 2.8vh), 26px); }
  .orn { width: 2.6em; height: 0.9em; flex: none; fill: none; stroke: var(--gold); stroke-width: 1.6; stroke-linecap: round; }
  .orn circle { fill: var(--gold); stroke: none; }
  .orn { transform: scaleX(-1); }
  .orn.flip { transform: none; }
  .lg .orn { font-size: clamp(24px, min(3.6vw, 4.8vh), 56px); }
  .md .orn { font-size: clamp(22px, min(2.6vw, 4.2vh), 40px); }
  .sm .orn { font-size: clamp(18px, min(1.7vw, 2.8vh), 26px); }
  .sub {
    margin: 0.3em 0 0;
    font-family: var(--heading-font);
    font-size: clamp(13px, min(1.35vw, 2.3vh), 21px);
    letter-spacing: 0.2em;
    color: var(--text);
  }
</style>

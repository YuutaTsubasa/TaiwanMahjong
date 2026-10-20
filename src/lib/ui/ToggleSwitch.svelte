<script>
  // 開關（設定頁的「靜音」）
  let { checked = false, onchange, label } = $props();
</script>

<button type="button" class="switch" class:on={checked} role="switch" aria-checked={checked} aria-label={label} onclick={() => onchange?.(!checked)}>
  <span class="knob"></span>
</button>

<style>
  .switch {
    position: relative;
    width: 76px;
    height: 36px;
    min-height: 0;
    padding: 0;
    border-radius: 999px;
    background: #0a1512;
    border: 1px solid var(--art-line);
    box-shadow: inset 0 2px 6px #000a;
  }
  .knob {
    position: absolute;
    top: 4px;
    left: 4px;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: radial-gradient(circle at 35% 30%, #fff, #9a9a9a);
    box-shadow: 0 2px 4px #000a;
    transition: left 0.18s;
  }
  .on { background: linear-gradient(90deg, #8a5a1d, #d9a74a); }
  .on .knob { left: 44px; }
</style>

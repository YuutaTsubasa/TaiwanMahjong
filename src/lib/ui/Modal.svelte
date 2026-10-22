<script>
  // 置中的對話框（大廳的「更換角色」「調整規則」）。按背景或 Esc 關閉。
  import OrnateHeading from './OrnateHeading.svelte';

  let { open = $bindable(false), title, children, width = '720px' } = $props();

  function key(e) {
    if (open && e.key === 'Escape') open = false;
  }
</script>

<svelte:window onkeydown={key} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={(e) => e.target === e.currentTarget && (open = false)}>
    <div class="art-panel dialog" role="dialog" aria-modal="true" aria-label={title} style:width>
      <button type="button" class="close" aria-label="關閉" onclick={() => (open = false)}>✕</button>
      <OrnateHeading text={title} size="md" tag="h2" />
      <div class="content">{@render children()}</div>
    </div>
  </div>
{/if}

<style>
  /* 欄寬固定是視窗寬度，對話框指定的寬度（例如 620px）在手機上才會被 max-width 縮到放得下 */
  .backdrop { position: fixed; inset: 0; z-index: 60; display: grid; grid-template-columns: minmax(0, 1fr); place-items: center; background: #000a; padding: 16px; }
  .dialog { position: relative; max-width: 100%; max-height: 92vh; overflow: auto; }
  .content { margin-top: 16px; }
  .close { position: absolute; top: 10px; right: 10px; width: 40px; height: 40px; min-height: 0; padding: 0; z-index: 2; }
</style>

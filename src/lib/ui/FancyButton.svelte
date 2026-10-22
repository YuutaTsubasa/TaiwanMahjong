<script>
  // 概念圖風格的按鈕：gold（主要動作，金底＋麻將牌＋›）、red（次要動作，酒紅底）
  import Tile from '../Tile.svelte';

  let {
    variant = 'gold',
    tile = '',
    chevron = variant === 'gold',
    href = undefined,
    type = 'button',
    form = undefined,
    disabled = false,
    onclick = undefined,
    class: extra = '',
    /** 手把：這個畫面還沒有焦點時，第一次按方向鍵先跳到這顆 */
    gamepadStart = false,
    children,
  } = $props();
</script>

{#if href}
  <a class="fancy {variant} {extra}" {href} {onclick} data-gamepad-start={gamepadStart ? '' : undefined}>
    <span class="label">{@render children()}</span>
    {#if tile}<span class="tile-ico" aria-hidden="true"><Tile code={tile} size="sm" /></span>{/if}
    {#if chevron}<span class="chev" aria-hidden="true">›</span>{/if}
  </a>
{:else}
  <button class="fancy {variant} {extra}" {type} {form} {disabled} {onclick} data-gamepad-start={gamepadStart ? '' : undefined}>
    <span class="label">{@render children()}</span>
    {#if tile}<span class="tile-ico" aria-hidden="true"><Tile code={tile} size="sm" /></span>{/if}
    {#if chevron}<span class="chev" aria-hidden="true">›</span>{/if}
  </button>
{/if}

<style>
  /* 英文：按鈕的大字距只適合中文 */
  :global(html:not([lang^='zh'])) .fancy { letter-spacing: .04em; }
</style>

<script>
  // 顯示牌局引擎的訊息：中文照原樣是文字；其他語言把牌畫成小牌，後面括號附上名字
  import Tile from './Tile.svelte';
  import { formatMsg, msgParts } from './msg.js';
  import { isZh, usesKanji } from './i18n.svelte.js';
  import { tileName } from './tiles.js';

  let { msg, seats } = $props();
</script>

{#if usesKanji()}{formatMsg(msg, seats)}{:else}{#each msgParts(msg, seats) as p}{#if p.tile}<span class="tile-ref" title={tileName(p.tile)}><Tile code={p.tile} size="xs" /><small>({tileName(p.tile)})</small></span>{:else}{p.text}{/if}{/each}{/if}

<style>
  .tile-ref { display: inline-flex; align-items: center; gap: 2px; vertical-align: middle; white-space: nowrap; margin: 0 1px; }
  .tile-ref :global(.tile) { --w: 18px; margin-bottom: 2px; }
  .tile-ref small { font-size: 0.85em; opacity: 0.85; }
</style>

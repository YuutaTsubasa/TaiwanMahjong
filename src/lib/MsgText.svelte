<script>
  // 顯示牌局引擎的訊息：牌一律畫成小牌，比較好找；英文等不用漢字的語言再在後面括號附上名字
  import Tile from './Tile.svelte';
  import { msgParts } from './msg.js';
  import { usesKanji } from './i18n.svelte.js';
  import { tileName } from './tiles.js';

  let { msg, seats } = $props();
</script>

{#each msgParts(msg, seats) as p}{#if p.tile}<span class="tile-ref" title={tileName(p.tile)}><Tile code={p.tile} size="xs" />{#if !usesKanji()}<small>({tileName(p.tile)})</small>{/if}</span>{:else}{p.text}{/if}{/each}

<style>
  .tile-ref { display: inline-flex; align-items: center; gap: 2px; vertical-align: middle; white-space: nowrap; margin: 0 1px; }
  .tile-ref :global(.tile) { --w: 18px; margin-bottom: 2px; }
  .tile-ref small { font-size: 0.85em; opacity: 0.85; }
</style>

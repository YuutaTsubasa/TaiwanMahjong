<script>
  import Tile from './Tile.svelte';
  let { melds = [], flowers = [], size = 'md' } = $props();
</script>

<div class="melds">
  {#each melds as m}
    <div class="meld" title={{ chow: '吃', pung: '碰', kong: '明槓', concealed_kong: '暗槓' }[m.kind]}>
      {#each m.tiles as t, i}
        <Tile code={t} {size} back={t === '?' || (m.kind === 'concealed_kong' && (i === 0 || i === 3))} />
      {/each}
    </div>
  {/each}
  {#if flowers.length}
    <div class="meld flowers">
      {#each flowers as f}
        <Tile code={f} size={size === 'md' ? 'sm' : 'xs'} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .melds { display: flex; flex-wrap: wrap; gap: 8px; align-items: flex-end; }
  .meld { display: flex; gap: 1px; }
  .flowers { gap: 2px; opacity: 0.95; }
</style>

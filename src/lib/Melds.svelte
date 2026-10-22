<script>
  import Tile from './Tile.svelte';
  import { t } from './i18n.svelte.js';
  // turn：左右兩家的副露橫躺著擺（left 是上家、right 是下家），牌面朝向該家
  let { melds = [], flowers = [], size = 'md', turn = '' } = $props();
</script>

<div class="melds" class:turned={turn} class:turn-left={turn === 'left'} class:turn-right={turn === 'right'}>
  {#each melds as m}
    <div class="meld" title={t({ chow: '吃', pung: '碰', kong: '明槓', concealed_kong: '暗槓' }[m.kind])}>
      {#each m.tiles as t, i}
        {#if turn}
          <span class="lay"><Tile code={t} {size} back={t === '?' || (m.kind === 'concealed_kong' && (i === 0 || i === 3))} /></span>
        {:else}
          <Tile code={t} {size} back={t === '?' || (m.kind === 'concealed_kong' && (i === 0 || i === 3))} />
        {/if}
      {/each}
    </div>
  {/each}
  {#if flowers.length}
    <div class="meld flowers">
      {#each flowers as f}
        {#if turn}
          <span class="lay"><Tile code={f} size={size === 'md' ? 'sm' : 'xs'} /></span>
        {:else}
          <Tile code={f} size={size === 'md' ? 'sm' : 'xs'} />
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .melds { display: flex; flex-wrap: wrap; gap: 8px; align-items: flex-end; }
  .meld { display: flex; gap: 1px; }
  .flowers { gap: 2px; opacity: 0.95; }

  /* 橫躺的牌：外框佔「牌高 × 牌寬」，裡面的牌轉 90 度 */
  .turned, .turned .meld { flex-direction: column; flex-wrap: nowrap; align-items: center; }
  .turn-right, .turn-right .meld { flex-direction: column-reverse; }
  .lay { position: relative; flex: none; width: calc(var(--lay-w, 20px) * 1.36); height: var(--lay-w, 20px); }
  .lay :global(.tile) {
    --w: var(--lay-w, 20px) !important;
    position: absolute;
    left: 50%;
    top: 50%;
    margin: 0;
    transform: translate(-50%, -50%) rotate(90deg);
  }
  .turn-right .lay :global(.tile) { transform: translate(-50%, -50%) rotate(-90deg); }
</style>

<script>
  import Tile from './Tile.svelte';
  import Melds from './Melds.svelte';
  import Nameplate from './Nameplate.svelte';

  let { seat, pos, active = false, thinking = false } = $props();
  const vertical = $derived(pos === 'left' || pos === 'right');
</script>

<div class="opp {pos}">
  <Nameplate {seat} {active} {thinking} bubble="down" />
  <div class="body" class:vertical>
    {#if seat.hand}
      <div class="hand">
        {#each seat.hand as t}<Tile code={t} size="xs" />{/each}
      </div>
    {:else if seat.hand_count != null}
      <div class="hand">
        {#each Array(seat.hand_count) as _}<Tile back size="xs" />{/each}
      </div>
    {/if}
    <Melds melds={seat.melds ?? []} flowers={seat.flowers ?? []} size="sm" />
  </div>
</div>

<style>
  .opp { display: flex; flex-direction: column; gap: 8px; min-width: 0; padding: 8px; }
  .opp.top { align-items: center; }
  .opp.left { align-items: flex-start; }
  .opp.right { align-items: flex-end; }
  .body { display: flex; flex-direction: column; gap: 6px; align-items: inherit; }
  .hand { display: flex; flex-wrap: wrap; gap: 1px; }
  .vertical .hand { max-width: 150px; }
  .right .hand { justify-content: flex-end; }
</style>

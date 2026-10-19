<script>
  import { SEAT_KIND_ICON } from './tiles.js';
  let { seat, active = false, thinking = false, isMe = false } = $props();
</script>

<div class="plate" class:active class:thinking>
  <span class="wind" class:dealer={seat.is_dealer}>{seat.wind ?? ''}</span>
  <div class="info">
    <div class="nm">
      <span>{SEAT_KIND_ICON[seat.kind] ?? ''}</span>
      <b>{seat.name ?? '空位'}</b>
      {#if isMe}<span class="me">你</span>{/if}
      {#if seat.auto}<span class="auto">託管</span>{/if}
      {#if !seat.connected && seat.kind !== 'bot'}<span class="off">離線</span>{/if}
    </div>
    <div class="score" class:neg={seat.score < 0}>{seat.score > 0 ? '+' : ''}{seat.score}</div>
  </div>
  {#if seat.is_dealer}<span class="dealer-tag">莊</span>{/if}
</div>

<style>
  .plate {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 5px 12px 5px 6px;
    background: rgba(0, 0, 0, 0.42);
    border: 1px solid var(--line);
    border-radius: 999px;
    position: relative;
    transition: box-shadow 0.2s, border-color 0.2s;
    max-width: 100%;
  }
  .plate.active { border-color: var(--gold); box-shadow: 0 0 0 2px var(--gold), 0 0 18px rgba(255, 209, 102, 0.6); }
  .plate.thinking { border-color: var(--accent); box-shadow: 0 0 0 2px var(--accent); }
  .wind {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: #f6efdc;
    color: #1d3557;
    font-weight: 700;
    font-family: 'DFKai-SB', '標楷體', serif;
    font-size: 18px;
    flex: none;
  }
  .wind.dealer { background: var(--gold); color: #7a1f00; }
  .info { min-width: 0; }
  .nm { display: flex; gap: 4px; align-items: center; font-size: 14px; white-space: nowrap; overflow: hidden; }
  .nm b { overflow: hidden; text-overflow: ellipsis; }
  .score { font-size: 12px; color: var(--accent); font-variant-numeric: tabular-nums; }
  .score.neg { color: #ff8fa3; }
  .me { background: var(--gold); color: #3d2200; font-size: 10px; padding: 0 5px; border-radius: 5px; font-weight: 700; }
  .auto { background: #4cc9f0; color: #062033; font-size: 10px; padding: 0 5px; border-radius: 5px; font-weight: 700; }
  .off { background: #555; font-size: 10px; padding: 0 5px; border-radius: 5px; }
  .dealer-tag {
    position: absolute;
    top: -8px;
    right: -6px;
    background: #c1121f;
    color: white;
    font-size: 11px;
    font-weight: 700;
    border-radius: 6px;
    padding: 0 5px;
  }
</style>

<script>
  import { SEAT_KIND_ICON } from './tiles.js';
  import Avatar from './Avatar.svelte';
  import { reactionFor } from './reactions.svelte.js';

  let { seat, active = false, thinking = false, isMe = false, bubble = 'up' } = $props();

  const reaction = $derived(reactionFor(seat.seat));
</script>

<div class="plate" class:active class:thinking>
  {#if seat.character}
    <div class="av" class:pop={reaction}>
      <Avatar id={seat.character} expression={reaction?.expression ?? 'normal'} size={54} crop />
      {#if seat.wind}<span class="wind badge" class:dealer={seat.is_dealer}>{seat.wind}</span>{/if}
    </div>
  {:else}
    <span class="wind" class:dealer={seat.is_dealer}>{seat.wind ?? ''}</span>
  {/if}
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
  {#if reaction?.text}
    {#key reaction.until}
      <div class="bubble {bubble}" class:big={reaction.kind === 'tsumo' || reaction.kind === 'ron'}>{reaction.text}</div>
    {/key}
  {/if}
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
  .av { margin: -12px 0 -12px -10px; transition: transform 0.2s; }
  .av.pop { animation: pop 0.35s ease; }
  @keyframes pop { 40% { transform: scale(1.18) rotate(-4deg); } }
  .wind {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: #f6efdc;
    color: #1d3557;
    font-weight: 700;
    font-family: 'DFKai-SB', '標楷體', serif;
    font-size: 16px;
    flex: none;
  }
  .av { position: relative; }
  .wind.badge {
    position: absolute;
    left: -4px;
    bottom: -2px;
    width: 22px;
    height: 22px;
    font-size: 13px;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.4);
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
  .bubble {
    position: absolute;
    left: 6px;
    z-index: 15;
    background: #fffdf6;
    color: #2b2033;
    font-size: 14px;
    font-weight: 700;
    padding: 6px 12px;
    border-radius: 14px;
    white-space: nowrap;
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.35);
    animation: bubble 0.3s cubic-bezier(0.2, 1.6, 0.4, 1);
    pointer-events: none;
  }
  .bubble.big { font-size: 17px; background: linear-gradient(180deg, #fff6d6, #ffd166); }
  .bubble.up { bottom: calc(100% + 10px); }
  .bubble.down { top: calc(100% + 10px); }
  .bubble::after {
    content: '';
    position: absolute;
    left: 22px;
    border: 7px solid transparent;
  }
  .bubble.up::after { top: 100%; border-top-color: #fffdf6; }
  .bubble.big.up::after { border-top-color: #ffd166; }
  .bubble.down::after { bottom: 100%; border-bottom-color: #fffdf6; }
  .bubble.big.down::after { border-bottom-color: #fff6d6; }
  @keyframes bubble { from { transform: scale(0.4) translateY(8px); opacity: 0; } }
</style>

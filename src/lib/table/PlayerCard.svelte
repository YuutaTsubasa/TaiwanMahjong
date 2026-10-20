<script>
  // 對局畫面的角色名牌：有框的半身立繪＋名字、門風、分數、各種標記與對話泡泡。
  // variant：side（左右兩家，直式）、top（對家，橫式）、me（自己，橫式）、chip（手機用的小名牌）
  import Avatar from '../Avatar.svelte';
  import { reactionFor } from '../reactions.svelte.js';

  let { seat, variant = 'side', active = false, thinking = false, isMe = false, bubble = 'down' } = $props();

  const reaction = $derived(reactionFor(seat.seat));
  const offline = $derived(!seat.connected && seat.kind !== 'bot');
  // 電腦的名字是「角色名(電腦)」，名牌太窄會被切掉；改成只顯示角色名，再加一個「電腦」標籤
  const isBot = $derived(seat.kind === 'bot');
  const shownName = $derived(isBot ? seat.name?.replace(/\(電腦\)$/, '') : seat.name);
</script>

<div class="pcard v-{variant}" class:active class:thinking class:empty={!seat.name}>
  <div class="portrait" class:pop={reaction}>
    {#if seat.character}
      <Avatar id={seat.character} expression={reaction?.expression ?? 'normal'} size={240} />
    {/if}
  </div>
  <div class="meta">
    <div class="line1">
      <b class="nm" title={seat.name}>{shownName ?? '空位'}</b>
      {#if seat.wind}<span class="wind" class:dealer={seat.is_dealer}>{seat.wind}</span>{/if}
    </div>
    <div class="line2">
      <span class="score" class:neg={seat.score < 0}>{seat.score > 0 ? '+' : ''}{seat.score} 分</span>
      {#if isMe}<span class="tag me">你</span>{/if}
      {#if seat.is_dealer}<span class="tag dealer">莊</span>{/if}
      {#if isBot}<span class="tag bot">電腦</span>{/if}
      {#if seat.kind === 'agent'}<span class="tag ai">AI</span>{/if}
      {#if seat.auto}<span class="tag auto">託管</span>{/if}
      {#if offline}<span class="tag off">離線</span>{/if}
    </div>
  </div>
  {#if reaction?.text}
    {#key reaction.until}
      <div class="bubble {bubble}" class:big={reaction.kind === 'tsumo' || reaction.kind === 'ron'}>{reaction.text}</div>
    {/key}
  {/if}
</div>

<style>
  .pcard {
    position: relative;
    display: flex;
    background: var(--art-panel);
    border: 1px solid var(--art-line);
    border-radius: var(--surface-radius);
    box-shadow: inset 0 0 0 3px #0008, inset 0 0 0 4px color-mix(in srgb, var(--art-line) 55%, transparent), 0 8px 22px #0009;
    color: var(--text);
    transition: box-shadow 0.2s;
  }
  .pcard.active {
    box-shadow: inset 0 0 0 3px #0008, inset 0 0 0 4px var(--gold), 0 0 0 2px var(--gold), 0 0 24px color-mix(in srgb, var(--gold) 60%, transparent);
  }
  .pcard.thinking { box-shadow: inset 0 0 0 3px #0008, 0 0 0 2px var(--accent), 0 8px 22px #0009; }

  .portrait {
    position: relative;
    overflow: hidden;
    flex: none;
    background: radial-gradient(circle at 50% 30%, color-mix(in srgb, var(--gold) 26%, transparent), transparent 70%);
  }
  .portrait :global(.avatar) {
    position: absolute;
    left: 50%;
    top: 0;
    width: 125% !important;
    height: auto !important;
    aspect-ratio: 1;
    transform: translateX(-50%);
  }
  .portrait.pop { animation: pop 0.35s ease; }
  @keyframes pop { 40% { filter: brightness(1.25); transform: scale(1.04); } }

  .meta { min-width: 0; display: flex; flex-direction: column; justify-content: center; gap: 4px; padding: 6px 10px; }
  .line1 { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .nm {
    font-family: var(--heading-font);
    font-size: 19px;
    letter-spacing: 0.08em;
    color: var(--gold);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .wind {
    flex: none;
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    font-family: var(--heading-font);
    font-size: 19px;
    font-weight: 700;
    color: var(--tile-blue);
    background: var(--tile-face);
    border-radius: var(--small-radius);
    box-shadow: 0 2px 0 var(--tile-edge);
  }
  .wind.dealer { color: #7a1f00; background: var(--gold); }
  .line2 { display: flex; align-items: center; gap: 5px; flex-wrap: wrap; border-top: 1px solid color-mix(in srgb, var(--art-line) 60%, transparent); padding-top: 4px; }
  .score { font-size: 15px; font-variant-numeric: tabular-nums; color: var(--text); margin-right: auto; }
  .score.neg { color: var(--negative); }
  .tag { font-size: 11px; font-weight: 700; padding: 1px 6px; border-radius: var(--small-radius); line-height: 1.5; }
  .tag.me { background: var(--gold); color: #3d2200; }
  .tag.dealer { background: #b3171f; color: #fff; }
  .tag.ai { background: #5a3b8c; color: #fff; }
  .tag.bot { background: #2a6f97; color: #fff; }
  .tag.auto { background: var(--claim-bg); color: var(--action-text); }
  .tag.off { background: #555; color: #ddd; }

  /* 左右兩家：直式 */
  .v-side { flex-direction: column; width: clamp(130px, 11vw, 184px); }
  .v-side .portrait { width: 100%; aspect-ratio: 1 / 1.02; border-bottom: 1px solid var(--art-line); }
  /* 對家：橫式 */
  .v-top { width: clamp(260px, 21vw, 360px); height: clamp(100px, 12.5vh, 128px); }
  .v-top .portrait { width: 40%; height: 100%; border-right: 1px solid var(--art-line); }
  /* 橫式名牌比較矮：圖縮小一點，才看得到整個頭和肩膀 */
  .v-top .portrait :global(.avatar) { width: 104% !important; top: -3%; }
  .v-top .meta { flex: 1; }
  /* 自己 */
  .v-me { width: clamp(220px, 17vw, 300px); height: clamp(110px, 15vh, 150px); }
  .v-me .portrait { width: 48%; height: 100%; border-right: 1px solid var(--art-line); }
  .v-me .meta { flex: 1; }
  /* 手機：小名牌（小頭像＋名字、門風、分數），對話泡泡照樣會出現 */
  .v-chip { align-items: center; gap: 6px; padding: 3px 8px 3px 3px; width: max-content; max-width: 100%; }
  .v-chip .portrait { width: 40px; height: 40px; border-radius: var(--small-radius); }
  .v-chip .portrait :global(.avatar) { width: 150% !important; top: -8%; }
  .v-chip .meta { padding: 0; gap: 1px; min-width: 0; }
  .v-chip .line1 { gap: 5px; justify-content: flex-start; }
  .v-chip .nm { font-size: 13px; letter-spacing: 0.02em; max-width: 7em; }
  /* 門風疊在頭像右下角，名字才有空間 */
  .v-chip .wind { position: absolute; left: 27px; top: 25px; width: 19px; height: 19px; font-size: 12px; box-shadow: 0 1px 3px #000a; z-index: 1; }
  .v-chip .line2 { gap: 3px; border-top: 0; padding-top: 0; flex-wrap: nowrap; }
  .v-chip .score { font-size: 11px; margin-right: 2px; }
  .v-chip .tag { font-size: 9px; padding: 0 4px; line-height: 1.6; }
  .v-chip .bubble { font-size: 12px; padding: 3px 9px; }
  .v-chip .bubble.big { font-size: 14px; }

  .empty { opacity: 0.6; }

  .bubble {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    z-index: 30;
    padding: 6px 14px;
    font-size: 15px;
    font-weight: 700;
    white-space: nowrap;
    color: var(--bubble-text);
    background: var(--bubble-bg);
    border: 1px solid var(--art-line);
    border-radius: var(--surface-radius);
    box-shadow: var(--bubble-shadow);
    pointer-events: none;
    animation: bubble 0.3s cubic-bezier(0.2, 1.6, 0.4, 1);
  }
  .bubble.big { font-size: 18px; border: 2px solid var(--gold); }
  .bubble.down { top: calc(100% + 10px); }
  .bubble.up { bottom: calc(100% + 10px); }
  .bubble.right { left: calc(100% + 12px); top: 30%; transform: none; }
  .bubble.left { left: auto; right: calc(100% + 12px); top: 30%; transform: none; }
  @keyframes bubble { from { opacity: 0; scale: 0.5; } }
</style>

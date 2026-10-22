<script>
  // 整場結束的「最終排名」：四家角色卡由第一名到第四名，第一名最大、金框，第四名表情難過。
  import { net, hostCmd, disconnect } from './net.svelte.js';
  import Avatar from './Avatar.svelte';
  import OrnateHeading from './ui/OrnateHeading.svelte';
  import FancyButton from './ui/FancyButton.svelte';
  import { t } from './i18n.svelte.js';
  import { seatName } from './tiles.js';

  const s = $derived(net.state);
  const local = $derived(net.mode === 'local');

  // 依分數排名；同分同名次
  const ranked = $derived.by(() => {
    const sorted = [...s.seats].filter((p) => p.name).sort((a, b) => b.score - a.score);
    let rank = 0;
    return sorted.map((p, i) => {
      if (i === 0 || p.score !== sorted[i - 1].score) rank = i + 1;
      return { ...p, rank };
    });
  });
  const champion = $derived(ranked[0]);
  const roundsLabel = $derived.by(() => {
    const n = s.settings?.rounds;
    const fixed = { 1: '東風圈', 2: '東南圈', 4: '一將' }[n];
    return fixed ? t(fixed) : t('{n} 圈', { n: n ?? '' });
  });
  const RANK_NAME = ['', '第一名', '第二名', '第三名', '第四名'];
  const championName = $derived(champion ? seatName(champion) : '');
  const expressionFor = (p) => (p.rank === 1 ? 'win' : p.rank === ranked.length && p.score < 0 ? 'sad' : p.rank === 2 ? 'happy' : 'normal');
</script>

<div class="final">
  <OrnateHeading text={t('最終排名')} sub={t('{rounds}結束・恭喜 {name} 拿下第一！', { rounds: roundsLabel, name: championName })} />

  <ol class="podium">
    {#each ranked as p (p.seat)}
      <li class="card r{p.rank}" class:me={p.seat === net.seat} style:--delay="{(ranked.length - p.rank) * 0.12}s">
        <div class="rank-badge">{t(RANK_NAME[p.rank])}</div>
        <div class="pic">
          {#if p.character}<Avatar id={p.character} expression={expressionFor(p)} size={260} />{/if}
          {#if p.rank === 1}<span class="crown" aria-hidden="true">♛</span>{/if}
        </div>
        <div class="info">
          <b class="nm" title={seatName(p)}>{seatName(p)}</b>
          <span class="score" class:neg={p.score < 0} class:pos={p.score > 0}>{p.score > 0 ? '+' : ''}{p.score}</span>
          {#if p.seat === net.seat}<span class="you">{t('你')}</span>{/if}
        </div>
      </li>
    {/each}
  </ol>

  <div class="btns">
    {#if net.isHost}
      <FancyButton variant="red" onclick={() => hostCmd('reset')}>{local ? t('回標題') : t('回大廳')}</FancyButton>
      <FancyButton variant="gold" tile="F" onclick={() => hostCmd('start')}>{t('再來一場')}</FancyButton>
    {:else}
      <FancyButton variant="red" onclick={disconnect}>{t('回首頁')}</FancyButton>
    {/if}
  </div>
</div>

<style>
  .final {
    width: min(1040px, 96%);
    max-height: 96%;
    overflow: auto;
    padding: clamp(14px, 2.4vh, 28px) clamp(14px, 2vw, 32px);
    background: var(--art-panel);
    border: 1px solid var(--art-line);
    border-radius: var(--surface-radius);
    box-shadow: inset 0 0 0 4px #0008, inset 0 0 0 5px var(--art-line), 0 20px 50px #000c;
    animation: rise 0.35s ease;
  }
  @keyframes rise { from { transform: translateY(18px); opacity: 0; } }

  .podium {
    list-style: none;
    margin: clamp(12px, 2.4vh, 26px) 0;
    padding: 0;
    display: grid;
    grid-template-columns: 1.3fr 1fr 1fr 1fr;
    align-items: end;
    gap: clamp(8px, 1.4vw, 18px);
  }
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    min-width: 0;
    background: #0007;
    border: 1px solid var(--art-line);
    border-radius: var(--small-radius);
    box-shadow: inset 0 0 0 2px #0008, 0 8px 18px #0009;
    animation: rise 0.45s ease both;
    animation-delay: var(--delay);
  }
  .card.me { outline: 2px solid var(--accent); outline-offset: 3px; }
  .r1 { --medal: #ffd166; border-color: #ffd166; box-shadow: 0 0 0 2px #ffd166, 0 0 30px #ffd16688, 0 10px 24px #000a; }
  .r2 { --medal: #d9e2ec; }
  .r3 { --medal: #e0a46a; }
  .r4 { --medal: #8a9a90; opacity: 0.92; }

  .rank-badge {
    position: absolute;
    top: -14px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 2;
    padding: 2px 16px;
    font: 700 clamp(14px, min(1.5vw, 2.6vh), 22px) var(--heading-font);
    letter-spacing: 0.2em;
    white-space: nowrap;
    color: #2b1a08;
    background: linear-gradient(#fff6, #0000 60%), var(--medal);
    border: 1px solid #0006;
    border-radius: var(--small-radius);
    box-shadow: 0 3px 8px #000a;
  }
  .pic {
    position: relative;
    overflow: hidden;
    aspect-ratio: 1 / 1.08;
    max-height: 46vh;
    background: radial-gradient(circle at 50% 35%, color-mix(in srgb, var(--medal) 40%, transparent), #0008 75%);
    border-bottom: 1px solid var(--art-line);
  }
  .pic :global(.avatar) {
    position: absolute;
    left: 50%;
    top: 2%;
    height: 112% !important;
    width: auto !important;
    aspect-ratio: 1;
    transform: translateX(-50%);
  }
  .r4 .pic :global(.avatar) { filter: saturate(0.6); }
  .crown {
    position: absolute;
    top: 4px;
    right: 8px;
    font-size: clamp(26px, 3vw, 40px);
    color: #ffd166;
    text-shadow: 0 2px 6px #000, 0 0 12px #ffd166aa;
  }
  .info { display: flex; flex-direction: column; align-items: center; gap: 2px; padding: 8px 6px 10px; }
  .nm {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: 700 clamp(15px, min(1.5vw, 2.6vh), 22px)/1.4 var(--heading-font);
    letter-spacing: 0.08em;
    color: var(--gold);
  }
  .r1 .nm { font-size: clamp(18px, min(2vw, 3.2vh), 28px); }
  .score { font: 700 clamp(18px, min(2vw, 3.4vh), 30px) var(--heading-font); font-variant-numeric: tabular-nums; color: var(--text); }
  .r1 .score { font-size: clamp(22px, min(2.6vw, 4.2vh), 40px); }
  .neg { color: var(--negative); }
  .pos { color: var(--accent); }
  .you { font-size: 13px; font-weight: 700; padding: 1px 10px; letter-spacing: 0.1em; border-radius: var(--small-radius); background: var(--gold); color: #3d2200; }
  .btns { display: flex; justify-content: center; gap: clamp(10px, 2vw, 24px); flex-wrap: wrap; }
  .btns :global(.fancy) { min-width: min(240px, 42vw); }

  /* 手機直向：改成直排清單（頭像在左） */
  @media (max-width: 640px) {
    .podium { grid-template-columns: 1fr; gap: 14px; }
    .card { flex-direction: row; align-items: center; }
    .pic { width: 30%; max-width: 110px; aspect-ratio: 1; border-bottom: 0; border-right: 1px solid var(--art-line); }
    .info { flex: 1; flex-direction: row; justify-content: space-between; padding: 8px 14px; }
    .rank-badge { left: 12px; transform: none; top: -12px; }
  }
  /* 手機橫向：高度很矮，頭像壓扁一點，讓按鈕不用捲動就看得到 */
  @media (max-height: 520px) and (min-width: 641px) {
    .final { max-height: 100%; padding: 8px 16px 10px; }
    .podium { margin: 18px 0 10px; }
    .pic { aspect-ratio: auto; height: 28vh; max-height: none; }
    .info { padding: 4px 6px 6px; gap: 0; }
    .nm { font-size: 14px; }
    .r1 .nm { font-size: 16px; }
    .score { font-size: 16px; }
    .r1 .score { font-size: 20px; }
    .crown { font-size: 22px; }
  }
  @media (prefers-reduced-motion: reduce) {
    .final, .card { animation: none; }
  }
</style>

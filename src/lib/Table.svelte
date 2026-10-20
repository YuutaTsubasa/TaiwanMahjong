<script>
  // 對局畫面，依 docs/art/concepts/gameplay-anime90-v2.png 排版：
  //   中間是俯視牌桌（public/ui/<畫風>/table.png），四家各有半身立繪名牌；
  //   桌面上是各家的手牌牆、副露、花與牌河，中央是局數／剩餘張數／倒數；
  //   下方是自己的名牌、提示列與手牌；右側是牌桌紀錄與聊天。
  import { onMount } from 'svelte';
  import { net, act, clock, leave, hostCmd, disconnect, setAuto } from './net.svelte.js';
  import { artStyle } from './art-style.js';
  import { asset } from './site.js';
  import Tile from './Tile.svelte';
  import Melds from './Melds.svelte';
  import Result from './Result.svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import PlayerCard from './table/PlayerCard.svelte';
  import SideLog from './table/SideLog.svelte';

  let confirmation;

  const s = $derived(net.state);
  const me = $derived(s.you ?? 0);
  const at = (k) => (me + k) % 4;
  const mine = $derived(s.seats[me]);
  const isPlayer = $derived(s.you != null);
  const top = $derived(s.seats[at(2)]);
  const left = $derived(s.seats[at(3)]);
  const right = $derived(s.seats[at(1)]);

  const auto = $derived(!!mine.auto);
  // 託管時由電腦出手，介面不顯示動作按鈕、手牌也不能點
  const actions = $derived(auto ? [] : (s.actions ?? []));
  const discardById = $derived(new Map(actions.filter((a) => a.type === 'discard').map((a) => [a.tile, a])));
  const special = $derived(actions.filter((a) => a.type !== 'discard' && a.type !== 'next'));
  const canDiscard = $derived(discardById.size > 0);

  const remainingMs = $derived(s.deadline_ms == null ? null : Math.max(0, s.deadline_ms - (clock.now - net.receivedAt)));
  const remaining = $derived(remainingMs == null ? null : Math.ceil(remainingMs / 1000));

  const active = (seat) => s.phase === 'playing' && s.step === 'turn' && s.turn === seat;
  const thinking = (seat) => s.phase === 'playing' && s.step !== 'turn' && s.waiting_on.includes(seat);

  // 選牌：第一下選取、第二下打出
  let selected = $state(null);
  let submittedDecision = null;
  onMount(() => {
    const cancel = () => {
      selected = null;
    };
    window.addEventListener('mahjong-cancel-selection', cancel);
    return () => window.removeEventListener('mahjong-cancel-selection', cancel);
  });
  $effect(() => {
    s.decision_id;
    selected = null;
  });
  $effect(() => {
    if (net.status !== 'open') submittedDecision = null;
  });

  function clickTile(code, key) {
    const a = discardById.get(code);
    if (!a || submittedDecision === s.decision_id) return;
    if (selected === key) {
      if (act(a.id)) submittedDecision = s.decision_id;
      selected = null;
    } else {
      selected = key;
    }
  }

  function isLast(seat, i, river) {
    const ld = s.last_discard;
    return ld && ld.seat === seat && i === river.length - 1 && river[i] === ld.tile && s.phase === 'playing';
  }

  const btnClass = { tsumo: 'win', ron: 'win', pung: 'claim', kong: 'claim', chow: 'claim', concealed_kong: 'claim', added_kong: 'claim', pass: 'pass' };
  const btnText = { tsumo: '自摸', ron: '胡', pung: '碰', kong: '槓', chow: '吃', concealed_kong: '暗槓', added_kong: '加槓', pass: '過' };

  const waitingText = $derived.by(() => {
    if (s.phase !== 'playing') return '';
    const name = (i) => s.seats[i]?.name;
    if (s.step === 'turn') return s.turn === s.you ? '' : `${name(s.turn)} 思考中…`;
    const who = s.waiting_on.filter((i) => i !== s.you).map(name);
    return who.length ? `等待 ${who.join('、')} 決定…` : '';
  });

  const keyFor = (t, i) => `${t}-${i}`;
  const backs = (n) => Array.from({ length: n ?? 0 });

  // 手機：窄（直向）或矮（橫向）。名牌改成小名牌，直向時三家排在牌桌上方
  const COMPACT_QUERY = '(max-width: 760px), (max-height: 520px)';
  const hasWindow = typeof window !== 'undefined';
  let compact = $state(hasWindow && window.matchMedia(COMPACT_QUERY).matches);
  let portrait = $state(hasWindow && window.innerHeight > window.innerWidth);
  onMount(() => {
    const mq = window.matchMedia(COMPACT_QUERY);
    const update = () => {
      compact = mq.matches;
      portrait = window.innerHeight > window.innerWidth;
    };
    mq.addEventListener('change', update);
    window.addEventListener('resize', update);
    return () => {
      mq.removeEventListener('change', update);
      window.removeEventListener('resize', update);
    };
  });
  const stripMode = $derived(compact && portrait);

  let showSide = $state(typeof window === 'undefined' || window.innerWidth >= 1100);
  let wasWide = typeof window === 'undefined' || window.innerWidth >= 1100;
  onMount(() => {
    // 視窗從窄變寬時自動打開紀錄、從寬變窄時收起
    const onResize = () => {
      const wide = window.innerWidth >= 1100;
      if (wide !== wasWide) showSide = wide;
      wasWide = wide;
    };
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  });

  function askLeave() {
    if (!isPlayer) return disconnect();
    confirmation.ask({ message: '離開後由電腦代打，確定？', confirmLabel: '離開房局', onconfirm: leave });
  }
  function askEnd() {
    confirmation.ask({ message: '結束這場遊戲並回到大廳？', confirmLabel: '結束牌局', onconfirm: () => hostCmd('reset') });
  }
</script>

<div class="gamescreen" class:noside={!showSide} class:compact class:strip={stripMode} style:--table-img={`url('${asset(`/ui/${$artStyle}/table.png`)}')`}>
  <div class="tableboard">
    {#if stripMode}
      <!-- 手機直向：上家｜對家｜下家 排在牌桌上方 -->
      <div class="opp-strip">
        <PlayerCard seat={left} variant="chip" active={active(at(3))} thinking={thinking(at(3))} bubble="down" />
        <PlayerCard seat={top} variant="chip" active={active(at(2))} thinking={thinking(at(2))} bubble="down" />
        <PlayerCard seat={right} variant="chip" active={active(at(1))} thinking={thinking(at(1))} bubble="down" />
      </div>
    {:else}
      <!-- 左右兩家的名牌在桌子外側 -->
      <div class="seat-left"><PlayerCard seat={left} variant={compact ? 'chip' : 'side'} active={active(at(3))} thinking={thinking(at(3))} bubble="right" /></div>
      <div class="seat-right"><PlayerCard seat={right} variant={compact ? 'chip' : 'side'} active={active(at(1))} thinking={thinking(at(1))} bubble="left" /></div>
    {/if}

    <div class="table-wrap">
      <div class="table">
        <div class="felt">
          <!-- 對家 -->
          {#if !stripMode}
            <div class="seat-top"><PlayerCard seat={top} variant={compact ? 'chip' : 'top'} active={active(at(2))} thinking={thinking(at(2))} /></div>
          {/if}
          <!-- 副露擺在各家手牌的右手邊（對家的右手邊是畫面左邊），跟手牌連成一排 -->
          <div class="wall wall-top">
            <div class="melds"><Melds melds={top.melds ?? []} flowers={top.flowers ?? []} size="sm" /></div>
            {#if top.hand}
              {#each top.hand as t}<Tile code={t} size="xs" />{/each}
            {:else}
              {#each backs(top.hand_count) as _}<span class="back-up"></span>{/each}
            {/if}
          </div>

          <!-- 上家（左）、下家（右）：直排的手牌牆 -->
          <div class="wall wall-left">
            {#if left.hand}
              {#each left.hand as t}<Tile code={t} size="xs" />{/each}
            {:else}
              {#each backs(left.hand_count) as _}<span class="back-side"></span>{/each}
            {/if}
            <div class="melds"><Melds melds={left.melds ?? []} flowers={left.flowers ?? []} size="sm" turn="left" /></div>
          </div>
          <div class="wall wall-right">
            <div class="melds"><Melds melds={right.melds ?? []} flowers={right.flowers ?? []} size="sm" turn="right" /></div>
            {#if right.hand}
              {#each right.hand as t}<Tile code={t} size="xs" />{/each}
            {:else}
              {#each backs(right.hand_count) as _}<span class="back-side"></span>{/each}
            {/if}
          </div>

          <!-- 自己的副露與花放在桌面右下角 -->
          <div class="melds melds-me"><Melds melds={mine.melds ?? []} flowers={mine.flowers ?? []} size="md" /></div>

          <!-- 牌河 -->
          {#each [['r-top', 2], ['r-left', 3], ['r-right', 1], ['r-bottom', 0]] as [cls, k]}
            {@const seat = at(k)}
            {@const river = s.seats[seat].discards ?? []}
            <div class="river {cls}">
              {#each river as t, i}
                <Tile code={t} size="md" hl={isLast(seat, i, river)} />
              {/each}
            </div>
          {/each}

          <!-- 中央資訊 -->
          <div class="center-panel">
            <div class="round">{s.round_wind}風圈・第 {s.hand_no} 局{#if s.streak > 0}<span class="streak">連{s.streak}</span>{/if}</div>
            <div class="left-count"><small>剩餘</small><b>{s.wall_count ?? '-'}</b></div>
            <div class="timer" class:urgent={remaining != null && remaining <= 5}>
              {remaining != null && s.phase === 'playing' ? `${remaining} 秒` : ' '}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 自己 -->
    <div class="seat-me">
      <PlayerCard seat={mine} variant={compact ? 'chip' : 'me'} active={active(me)} thinking={thinking(me)} isMe={isPlayer} bubble="up" />
    </div>
    <div class="bottom">
      <div class="prompt-row">
        {#if auto && s.phase === 'playing'}
          <div class="banner auto-banner">
            <b>託管中</b><span class="sep"></span><span>電腦正在幫你打</span>
          </div>
        {:else if special.length}
          <div class="actions">
            {#each special as a (a.id)}
              <button class="act {btnClass[a.type]}" onclick={() => act(a.id)}>
                <span>{btnText[a.type]}</span>
                {#if a.type === 'chow'}
                  <span class="mini">{#each a.tiles as t}<Tile code={t} size="xs" />{/each}</span>
                {:else if a.tile && a.type !== 'pass'}
                  <span class="mini"><Tile code={a.tile} size="xs" /></span>
                {/if}
              </button>
            {/each}
            {#if remaining != null}<span class="cd" class:urgent={remaining <= 5}>{remaining}s</span>{/if}
          </div>
        {:else if canDiscard}
          <div class="banner turn" title="鍵盤／手把：方向鍵選牌 · Enter／A 確認 · Esc／B 取消">
            <b>輪到你出牌</b><span class="sep"></span><span>點一下選牌，再點一下打出</span>
          </div>
        {:else if waitingText}
          <div class="banner quiet"><span>{waitingText}</span></div>
        {:else if !isPlayer}
          <div class="banner quiet"><span>觀戰中</span></div>
        {/if}

        {#if s.my_waits?.length && !auto}
          <div class="waits">聽 {#each s.my_waits as w}<Tile code={w} size="xs" />{/each}</div>
        {/if}

        {#if isPlayer && s.phase === 'playing'}
          <button class="auto-btn" class:on={auto} onclick={() => setAuto(!auto)} title="讓電腦幫你打，隨時可以取消">
            {auto ? '我要自己打' : '託管'}
          </button>
        {/if}
      </div>

      <div class="hand" class:turn={canDiscard}>
        {#each mine.hand ?? [] as t, i (keyFor(t, i))}
          <Tile code={t} size="xl" clickable={canDiscard} selected={selected === keyFor(t, i)} onclick={() => clickTile(t, keyFor(t, i))} />
        {/each}
        {#if mine.drawn}
          <span class="drawn-gap"></span>
          <span class="drawn">
            <Tile code={mine.drawn} size="xl" clickable={canDiscard} selected={selected === 'drawn'} onclick={() => clickTile(mine.drawn, 'drawn')} />
          </span>
        {/if}
      </div>
    </div>

    {#if s.phase === 'hand_over' || s.phase === 'game_over'}
      <Result />
    {/if}
  </div>

  {#if showSide}
    <SideLog onclose={() => (showSide = false)} onleave={askLeave} onend={net.isHost ? askEnd : null} />
  {:else}
    <button class="open-side" onclick={() => (showSide = true)}>‹ 紀錄</button>
  {/if}
</div>

<ConfirmDialog bind:this={confirmation} />

<style>
  .gamescreen {
    position: relative;
    height: 100%;
    display: grid;
    grid-template-columns: minmax(0, 1fr) clamp(260px, 21vw, 340px);
    gap: 12px;
    padding: 10px 12px 12px;
    /* 雀館背景：模糊、壓暗，讓牌桌跳出來 */
    background:
      radial-gradient(ellipse at 45% 50%, #0000 30%, #000a 100%),
      linear-gradient(#0006, #0006),
      var(--scene-parlor) center / cover;
  }
  .gamescreen.noside { grid-template-columns: minmax(0, 1fr); }

  .tableboard {
    position: relative;
    min-width: 0;
    min-height: 0;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    grid-template-rows: minmax(0, 1fr) auto;
    grid-template-areas:
      'sl table sr'
      'me bottom bottom';
    column-gap: clamp(8px, 1.2vw, 20px);
    row-gap: 6px;
  }
  /* z-index 讓名牌的對話泡泡留在結果視窗（z-index 20）底下 */
  .seat-left { grid-area: sl; align-self: center; position: relative; z-index: 3; }
  .seat-right { grid-area: sr; align-self: center; position: relative; z-index: 3; }
  .seat-me { grid-area: me; align-self: end; z-index: 3; }

  /* ───── 牌桌：table.png 是 1600×1100，桌子在 (300,50) 起 1000×1000 ───── */
  .table-wrap { grid-area: table; min-height: 0; display: grid; place-items: center; container-type: size; }
  .table {
    position: relative;
    width: min(100cqw, 100cqh);
    aspect-ratio: 1;
    background: var(--table-img) 50% 50% / 160% 110% no-repeat;
    filter: drop-shadow(0 18px 24px #000a);
  }
  /* 絨布區（木框約占 4.6%） */
  .felt { position: absolute; inset: 4.6%; container-type: size; }

  /* 對家名牌依絨布大小縮放，手牌牆排在名牌正下方，不會被蓋住 */
  .felt { --top-card-h: clamp(62px, 14cqh, 128px); }
  .seat-top { position: absolute; top: -3%; left: 50%; transform: translateX(-50%); z-index: 4; }
  .seat-top :global(.v-top) { width: clamp(190px, 46cqw, 360px); height: var(--top-card-h); }

  .wall { position: absolute; display: flex; align-items: center; gap: 1px; }
  .wall-top { top: calc(var(--top-card-h) - 1.5cqh); left: 50%; transform: translateX(-50%); }
  .wall-left { left: 3%; top: 50%; transform: translateY(-50%); flex-direction: column; }
  .wall-right { right: 3%; top: 50%; transform: translateY(-50%); flex-direction: column; }
  .wall > :global(.tile) { --w: clamp(14px, 4.1cqw, 30px); margin-bottom: 0; }
  .wall-left > :global(.tile), .wall-right > :global(.tile) { --w: clamp(12px, 2.9cqh, 20px); margin-bottom: calc(var(--w) * 0.1); }
  /* 副露跟手牌之間留一點空隙 */
  .wall-top .melds { margin-right: clamp(4px, 1.2cqw, 10px); }
  .wall-left .melds { margin-top: clamp(4px, 1.2cqh, 10px); }
  .wall-right .melds { margin-bottom: clamp(4px, 1.2cqh, 10px); }
  .wall .melds :global(.melds) { gap: clamp(3px, 0.8cqw, 6px); flex-wrap: nowrap; }
  .wall-top .melds :global(.tile) { --w: clamp(13px, 3.3cqw, 26px); }
  .wall-left .melds, .wall-right .melds { --lay-w: clamp(11px, 2.7cqh, 20px); }
  /* 牌背：對家是立著的長方形，左右兩家是側面 */
  .back-up, .back-side {
    display: block;
    background: var(--tile-back);
    box-shadow: inset 0 -3px 0 var(--tile-face), 0 2px 0 var(--tile-edge), 1px 3px 4px #0006;
    border-radius: 2px;
  }
  .back-up { width: clamp(14px, 4.1cqw, 30px); height: clamp(20px, 5.6cqw, 42px); }
  .back-side { width: clamp(22px, 6cqw, 42px); height: clamp(13px, 3.9cqh, 27px); box-shadow: inset -3px 0 0 var(--tile-face), 0 2px 0 var(--tile-edge), 1px 3px 4px #0006; }
  .wall-right .back-side { box-shadow: inset 3px 0 0 var(--tile-face), 0 2px 0 var(--tile-edge), 1px 3px 4px #0006; }

  .melds-me { position: absolute; right: 2%; bottom: 2%; max-width: 46%; }
  .melds-me :global(.melds) { gap: 4px; justify-content: flex-end; }
  .melds-me :global(.tile) { --w: clamp(16px, 4cqw, 30px); }

  /* 牌河：上下兩家每排 8 張，左右兩家每列 6 張；往外長 */
  .river { position: absolute; display: flex; gap: 2px; }
  .river :global(.tile) { --w: clamp(20px, 5.4cqw, 42px); margin-bottom: calc(var(--w) * 0.12); }
  /* 牌河依中央資訊框的大小往外推，兩者之間固定留 --river-gap，不會互相蓋到 */
  .felt { --panel-w: 26%; --river-gap: 2%; }
  .r-top {
    bottom: calc(50% + var(--panel-w) / 2.9 + var(--river-gap));
    left: 50%;
    transform: translateX(-50%);
    width: calc(8 * (clamp(20px, 5.4cqw, 42px) + 2px));
    flex-direction: row-reverse;
    flex-wrap: wrap-reverse;
  }
  .r-bottom {
    top: calc(50% + var(--panel-w) / 2.9 + var(--river-gap));
    left: 50%;
    transform: translateX(-50%);
    width: calc(8 * (clamp(20px, 5.4cqw, 42px) + 2px));
    flex-wrap: wrap;
  }
  .r-left {
    right: calc(50% + var(--panel-w) / 2 + var(--river-gap));
    top: 50%;
    transform: translateY(-50%);
    height: calc(6 * (clamp(20px, 5.4cqw, 42px) * 1.48 + 2px));
    flex-direction: column;
    flex-wrap: wrap-reverse;
    align-content: flex-start;
  }
  .r-right {
    left: calc(50% + var(--panel-w) / 2 + var(--river-gap));
    top: 50%;
    transform: translateY(-50%);
    height: calc(6 * (clamp(20px, 5.4cqw, 42px) * 1.48 + 2px));
    flex-direction: column-reverse;
    flex-wrap: wrap;
  }

  .center-panel {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    width: var(--panel-w);
    aspect-ratio: 1.45;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2%;
    background: color-mix(in srgb, var(--table-center) 92%, transparent);
    border: 1px solid var(--art-line);
    border-radius: var(--surface-radius);
    box-shadow: inset 0 0 0 4px #0007, inset 0 0 0 5px var(--art-line), 0 8px 18px #0009;
    font-family: var(--heading-font);
    color: var(--text);
  }
  .round { font-size: clamp(11px, 2.4cqw, 19px); letter-spacing: 0.08em; }
  .streak { margin-left: 6px; padding: 0 6px; font-size: 0.8em; background: #b3171f; color: #fff; border-radius: var(--small-radius); }
  .left-count { display: flex; align-items: baseline; gap: 8px; }
  .left-count small { font-size: clamp(11px, 2.2cqw, 18px); }
  .left-count b { font-size: clamp(26px, 7.5cqw, 60px); line-height: 1; color: var(--gold); font-variant-numeric: tabular-nums; text-shadow: 0 2px 0 #0008; }
  .timer {
    width: 80%;
    text-align: center;
    border-top: 1px solid var(--art-line);
    padding-top: 2%;
    font-size: clamp(11px, 2.3cqw, 18px);
    font-variant-numeric: tabular-nums;
  }
  .timer.urgent { color: var(--red); font-weight: 700; }

  /* ───── 下方：提示列＋手牌 ───── */
  .bottom { grid-area: bottom; min-width: 0; display: flex; flex-direction: column; align-items: center; gap: 4px; container-type: inline-size; }
  .prompt-row { display: flex; align-items: center; justify-content: center; gap: 12px; min-height: 46px; width: 100%; flex-wrap: wrap; }
  .banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 26px;
    background: var(--art-panel);
    border: 1px solid var(--art-line);
    border-radius: var(--surface-radius);
    box-shadow: inset 0 0 0 3px #0008, inset 0 0 0 4px color-mix(in srgb, var(--art-line) 60%, transparent), 0 6px 14px #0008;
  }
  .banner b { font-family: var(--heading-font); font-size: clamp(18px, 1.7vw, 26px); letter-spacing: 0.18em; color: var(--gold); }
  .banner span { font-size: 14px; color: var(--text); }
  .banner .sep { width: 1px; align-self: stretch; background: var(--art-line); }
  .banner.quiet span { color: var(--muted); font-size: 15px; }
  .banner.turn { border-color: var(--gold); }
  .auto-banner b { color: var(--accent); }

  .actions { display: flex; gap: 10px; align-items: center; animation: rise 0.2s ease; }
  @keyframes rise { from { transform: translateY(8px); opacity: 0; } }
  .act { display: flex; align-items: center; gap: 8px; font-family: var(--heading-font); font-size: 24px; font-weight: 700; padding: 6px 22px; }
  .mini { display: inline-flex; gap: 1px; }
  .cd { font-variant-numeric: tabular-nums; color: var(--muted); }
  .cd.urgent { color: var(--red); font-weight: 700; }

  .waits {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px 10px 0;
    font-size: 13px;
    color: var(--accent);
    background: var(--art-panel);
    border: 1px solid var(--art-line);
    border-radius: var(--surface-radius);
  }
  .auto-btn {
    min-width: 128px;
    padding: 8px 18px;
    font-family: var(--heading-font);
    font-size: 19px;
    letter-spacing: 0.2em;
    background: var(--art-panel);
    color: var(--gold);
    border: 1px solid var(--art-line);
    box-shadow: inset 0 0 0 3px #0008, inset 0 0 0 4px color-mix(in srgb, var(--art-line) 60%, transparent);
  }
  .auto-btn.on { color: var(--accent); border-color: var(--accent); }

  .hand { display: flex; align-items: flex-end; gap: 3px; max-width: 100%; overflow-x: auto; overflow-y: hidden; padding: 16px 6px 6px; }
  /* 17 張牌（16＋摸到的那張）要剛好塞進下方區域的寬度，也不要比畫面高度允許的大 */
  .hand :global(.tile) { flex-shrink: 0; --w: clamp(26px, min(calc((100cqw - 48px) / 18.4), 7.2vh), 54px); }
  .drawn-gap { flex: none; width: clamp(10px, 1.2vw, 20px); }
  .drawn { display: inline-flex; border-radius: 6px; box-shadow: 0 0 0 2px var(--gold), 0 0 18px color-mix(in srgb, var(--gold) 70%, transparent); }

  .open-side { position: absolute; right: 12px; top: 12px; z-index: 5; }

  /* ───── 窄螢幕：名牌縮小、紀錄改成浮在上面 ───── */
  @media (max-width: 1100px) {
    .gamescreen { grid-template-columns: minmax(0, 1fr); }
    .gamescreen :global(.sidelog) {
      position: absolute;
      right: 8px;
      top: 8px;
      bottom: 8px;
      width: min(330px, calc(100% - 16px));
      z-index: 40;
    }
    .seat-left :global(.v-side), .seat-right :global(.v-side) { width: 110px; }
  }

  /* ───── 手機（.compact）：名牌都是小名牌 ───── */
  .compact { padding: 6px; gap: 6px; }
  .compact .tableboard { column-gap: 6px; row-gap: 4px; }
  .compact .felt { --top-card-h: 46px; }
  .compact .seat-top :global(.v-chip) { width: max-content; height: auto; }
  .compact .seat-left :global(.v-chip), .compact .seat-right :global(.v-chip) { flex-direction: column; padding: 4px; max-width: 86px; }
  .compact .seat-left :global(.v-chip .portrait), .compact .seat-right :global(.v-chip .portrait) { width: 46px; height: 46px; }
  .compact .seat-left :global(.v-chip .meta), .compact .seat-right :global(.v-chip .meta) { align-items: center; }
  .compact .seat-left :global(.v-chip .line1), .compact .seat-right :global(.v-chip .line1) { flex-direction: column; gap: 2px; }
  .compact .prompt-row { min-height: 34px; gap: 6px; }
  .compact .banner { padding: 4px 12px; gap: 8px; }
  .compact .banner b { font-size: 16px; letter-spacing: 0.1em; }
  .compact .banner span { font-size: 12px; }
  .compact .act { font-size: 18px; padding: 4px 14px; }
  .compact .auto-btn { min-width: 0; padding: 4px 12px; font-size: 15px; letter-spacing: 0.1em; }
  .compact .hand { padding: 10px 4px 4px; }
  .compact .hand :global(.tile) { --w: clamp(22px, min(calc((100cqw - 40px) / 18.4), 9vh), 54px); }
  .compact .open-side { top: 6px; right: 6px; padding: 4px 10px; }
  /* 牌桌小的時候，中央資訊框放寬一點、字縮小，局數才不會斷行 */
  .compact .felt { --panel-w: 32%; }
  .compact .round { font-size: clamp(9px, 3.6cqw, 15px); letter-spacing: 0; white-space: nowrap; }
  .compact .left-count small { font-size: clamp(9px, 3.2cqw, 14px); }
  .compact .timer { font-size: clamp(9px, 3.2cqw, 14px); }

  /* 直向：上方三家小名牌、牌桌滿寬、下方自己＋提示列、手牌兩排 */
  .strip .tableboard {
    grid-template-columns: auto minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr) auto;
    grid-template-areas:
      'strip strip'
      'table table'
      'me bottom';
    align-items: end;
  }
  .opp-strip { grid-area: strip; display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 4px; position: relative; z-index: 5; }
  .opp-strip :global(.v-chip) { width: 100%; }
  .opp-strip :global(.v-chip:nth-child(2)) { justify-content: center; }
  .strip .felt { --top-card-h: 4cqh; }
  /* 牌桌區沒有內容高度（它是 size container），一定要撐滿那一列 */
  .strip .table-wrap { align-self: stretch; }
  .strip .seat-me { align-self: start; padding-top: 2px; }
  .strip .bottom { grid-column: 1 / -1; grid-row: 3; }
  .strip .prompt-row { padding-left: 132px; justify-content: flex-start; }
  .strip .hand { flex-wrap: wrap; justify-content: center; overflow: visible; row-gap: 6px; }
  .strip .hand :global(.tile) { --w: clamp(26px, calc((100cqw - 40px) / 9.6), 48px); }
  .strip .drawn-gap { width: 6px; }
  /* 直向：提示列只留主要文字，跟託管按鈕排在同一行 */
  .strip .banner .sep, .strip .banner span:not(.sep) { display: none; }
  .strip .banner.quiet span { display: inline; font-size: 12px; }
  .strip .prompt-row { flex-wrap: nowrap; }
  /* 「紀錄」按鈕移到牌桌右上角，不擋到下家的名牌 */
  .strip .open-side { top: 62px; right: 10px; }
  .compact .seat-left :global(.v-chip .wind), .compact .seat-right :global(.v-chip .wind) { left: 33px; top: 31px; }

</style>

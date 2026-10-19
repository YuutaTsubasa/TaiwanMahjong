<script>
  import { net, act, clock, leave, hostCmd, disconnect, setAuto } from './net.svelte.js';
  import Tile from './Tile.svelte';
  import Melds from './Melds.svelte';
  import Nameplate from './Nameplate.svelte';
  import Opponent from './Opponent.svelte';
  import Result from './Result.svelte';
  import Chat from './Chat.svelte';

  const s = $derived(net.state);
  const me = $derived(s.you ?? 0);
  const at = (k) => (me + k) % 4;
  const mine = $derived(s.seats[me]);
  const isPlayer = $derived(s.you != null);

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

  // 選牌：第一下選取、第二下打出（或雙擊）
  let selected = $state(null);
  $effect(() => {
    s.decision_id;
    selected = null;
  });

  function clickTile(code, key) {
    const a = discardById.get(code);
    if (!a) return;
    if (selected === key) {
      act(a.id);
      selected = null;
    } else {
      selected = key;
    }
  }
  function dbl(code) {
    const a = discardById.get(code);
    if (a) act(a.id);
  }

  const lastDiscard = $derived(s.last_discard);
  function isLast(seat, i, river) {
    return (
      lastDiscard && lastDiscard.seat === seat && i === river.length - 1 && river[i] === lastDiscard.tile && s.phase === 'playing'
    );
  }

  const btnClass = { tsumo: 'win', ron: 'win', pung: 'claim', kong: 'claim', chow: 'claim', concealed_kong: 'claim', added_kong: 'claim', pass: 'pass' };
  const btnText = { tsumo: '自摸', ron: '胡', pung: '碰', kong: '槓', chow: '吃', concealed_kong: '暗槓', added_kong: '加槓', pass: '過' };

  const status = $derived.by(() => {
    if (s.phase !== 'playing') return '';
    const name = (i) => s.seats[i]?.name;
    if (s.step === 'turn') return s.turn === s.you ? '輪到你出牌' : `${name(s.turn)} 思考中…`;
    if (s.step === 'claims' || s.step === 'rob_kong') {
      const who = s.waiting_on.filter((i) => i !== s.you).map(name);
      if (s.waiting_on.includes(s.you)) return '要吃碰胡嗎？';
      return who.length ? `等待 ${who.join('、')} 決定…` : '';
    }
    return '';
  });

  const keyFor = (t, i) => `${t}-${i}`;

  let showSide = $state(true);
</script>

<div class="game" class:noside={!showSide}>
  <div class="board">
    <!-- 對家 -->
    <div class="top"><Opponent seat={s.seats[at(2)]} pos="top" active={active(at(2))} thinking={thinking(at(2))} /></div>
    <!-- 上家 -->
    <div class="left"><Opponent seat={s.seats[at(3)]} pos="left" active={active(at(3))} thinking={thinking(at(3))} /></div>
    <!-- 下家 -->
    <div class="right"><Opponent seat={s.seats[at(1)]} pos="right" active={active(at(1))} thinking={thinking(at(1))} /></div>

    <!-- 中央：牌河 -->
    <div class="center">
      {#each [['r-top', 2], ['r-left', 3], ['r-right', 1], ['r-bottom', 0]] as [cls, k]}
        {@const seat = at(k)}
        {@const river = s.seats[seat].discards ?? []}
        <div class="river {cls}">
          {#each river as t, i}
            <Tile code={t} size="sm" hl={isLast(seat, i, river)} />
          {/each}
        </div>
      {/each}
      <div class="info">
        <div class="round">{s.round_wind}風圈 · 第 {s.hand_no} 局</div>
        <div class="wall"><span>{s.wall_count ?? '-'}</span><small>剩餘</small></div>
        {#if s.streak > 0}<div class="streak">連 {s.streak}</div>{/if}
        {#if remaining != null && s.phase === 'playing'}
          <div class="timer" class:urgent={remaining <= 5}>⏱ {remaining}</div>
        {/if}
      </div>
    </div>

    <!-- 自己 -->
    <div class="bottom">
      <div class="status-row">
        <Nameplate seat={mine} active={active(me)} thinking={thinking(me)} isMe={isPlayer} />
        <div class="status" class:mine={s.waiting_on.includes(s.you)}>{status}</div>
        {#if s.my_waits?.length}
          <div class="waits">聽：{#each s.my_waits as w}<Tile code={w} size="xs" />{/each}</div>
        {/if}
        <div class="spacer"></div>
        {#if isPlayer && s.phase === 'playing'}
          <button class="small auto-btn" class:on={auto} onclick={() => setAuto(!auto)} title="讓電腦幫你打，隨時可以取消">
            🤖 {auto ? '取消託管' : '託管'}
          </button>
        {/if}
        <Melds melds={mine.melds ?? []} flowers={mine.flowers ?? []} size="md" />
      </div>

      {#if auto && s.phase === 'playing'}
        <div class="auto-bar">
          <span>🤖 託管中，電腦正在幫你打</span>
          <button class="small" onclick={() => setAuto(false)}>我要自己打</button>
        </div>
      {/if}

      {#if special.length}
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
      {/if}

      <div class="hand" class:turn={canDiscard}>
        {#each mine.hand ?? [] as t, i (keyFor(t, i))}
          <Tile
            code={t}
            size="xl"
            clickable={canDiscard}
            selected={selected === keyFor(t, i)}
            onclick={() => clickTile(t, keyFor(t, i))}
            ondblclick={() => dbl(t)}
          />
        {/each}
        {#if mine.drawn}
          <span class="drawn-gap"></span>
          <Tile
            code={mine.drawn}
            size="xl"
            clickable={canDiscard}
            selected={selected === 'drawn'}
            onclick={() => clickTile(mine.drawn, 'drawn')}
            ondblclick={() => dbl(mine.drawn)}
          />
        {/if}
        {#if !isPlayer}
          <div class="muted spect">觀戰中</div>
        {/if}
      </div>
      {#if canDiscard && !special.length}
        <div class="hint">點一下選牌、再點一下打出（或雙擊）{remaining != null ? ` · ${remaining}s` : ''}</div>
      {/if}
    </div>

    {#if s.phase === 'hand_over' || s.phase === 'game_over'}
      <Result />
    {/if}
  </div>

  {#if showSide}
    <aside class="side card">
      <div class="side-head">
        <b>牌桌紀錄</b>
        <button class="small" onclick={() => (showSide = false)}>收起 ›</button>
      </div>
      <Chat showLog />
      <div class="side-btns">
        {#if net.isHost}
          <button class="small danger" onclick={() => confirm('結束這場遊戲並回到大廳？') && hostCmd('reset')}>結束牌局</button>
        {/if}
        {#if isPlayer}
          <button class="small" onclick={() => confirm('離開後由電腦代打，確定？') && leave()}>離開（電腦代打）</button>
        {:else}
          <button class="small" onclick={disconnect}>回首頁</button>
        {/if}
      </div>
    </aside>
  {:else}
    <button class="open-side small" onclick={() => (showSide = true)}>‹ 紀錄</button>
  {/if}
</div>

<style>
  .game { position: relative; height: 100%; display: grid; grid-template-columns: 1fr 290px; gap: 10px; padding: 10px; }
  .game.noside { grid-template-columns: 1fr; }
  .board {
    position: relative;
    display: grid;
    grid-template-columns: minmax(150px, 190px) 1fr minmax(150px, 190px);
    grid-template-rows: auto 1fr auto;
    grid-template-areas:
      'left top right'
      'left center right'
      'bottom bottom bottom';
    min-height: 0;
    border-radius: 22px;
    background: radial-gradient(ellipse at 50% 45%, rgba(255, 255, 255, 0.07), transparent 65%);
    box-shadow: inset 0 0 0 10px #4a2c17, inset 0 0 0 12px #2b190c, inset 0 0 60px rgba(0, 0, 0, 0.5);
    padding: 16px;
  }
  .top { grid-area: top; display: flex; justify-content: center; }
  .left { grid-area: left; overflow: hidden; }
  .right { grid-area: right; overflow: hidden; display: flex; justify-content: flex-end; }
  .center {
    grid-area: center;
    display: grid;
    grid-template-columns: 1fr minmax(120px, 160px) 1fr;
    grid-template-rows: 1fr auto 1fr;
    grid-template-areas:
      '. rtop .'
      'rleft info rright'
      '. rbottom .';
    gap: 8px;
    min-height: 0;
    align-items: center;
    justify-items: center;
  }
  .river { display: flex; flex-wrap: wrap; gap: 1px; align-content: flex-start; }
  .r-top { grid-area: rtop; grid-column: 1 / 4; max-width: 440px; align-self: end; justify-content: center; flex-direction: row-reverse; flex-wrap: wrap-reverse; }
  .r-bottom { grid-area: rbottom; grid-column: 1 / 4; max-width: 440px; align-self: start; justify-content: center; }
  .r-left { grid-area: rleft; max-width: 150px; justify-self: end; justify-content: flex-end; }
  .r-right { grid-area: rright; max-width: 150px; justify-self: start; }
  .info {
    grid-area: info;
    width: 100%;
    aspect-ratio: 1;
    max-width: 150px;
    border-radius: 18px;
    background: rgba(0, 0, 0, 0.38);
    border: 1px solid var(--line);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
  }
  .round { font-size: 12px; color: var(--muted); }
  .wall { display: flex; flex-direction: column; align-items: center; line-height: 1; }
  .wall span { font-size: 34px; font-weight: 700; color: var(--gold); font-variant-numeric: tabular-nums; }
  .wall small { font-size: 11px; color: var(--muted); }
  .streak { font-size: 12px; background: #c1121f; padding: 1px 8px; border-radius: 6px; }
  .timer { font-size: 15px; font-variant-numeric: tabular-nums; }
  .timer.urgent, .cd.urgent { color: var(--red); font-weight: 700; }
  .bottom { grid-area: bottom; display: flex; flex-direction: column; gap: 8px; align-items: center; padding-top: 6px; }
  .status-row { width: 100%; display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
  .spacer { flex: 1; }
  .status { font-size: 15px; color: var(--muted); }
  .status.mine { color: var(--gold); font-weight: 700; }
  .waits { display: flex; align-items: center; gap: 2px; font-size: 13px; color: var(--accent); background: rgba(0, 0, 0, 0.3); padding: 2px 8px 0; border-radius: 8px; }
  .actions { display: flex; gap: 10px; align-items: center; animation: pop 0.2s ease; }
  @keyframes pop { from { transform: scale(0.9); opacity: 0; } }
  .act { display: flex; align-items: center; gap: 8px; font-size: 20px; font-weight: 700; padding: 8px 18px; border-radius: 14px; }
  .act.win { background: linear-gradient(180deg, #ff6b6b, #c1121f); border: none; box-shadow: 0 0 16px rgba(255, 80, 80, 0.6); }
  .act.claim { background: linear-gradient(180deg, #4cc9f0, #1d4e89); border: none; }
  .act.pass { background: rgba(255, 255, 255, 0.12); }
  .mini { display: inline-flex; gap: 1px; }
  .cd { font-variant-numeric: tabular-nums; color: var(--muted); }
  .hand { display: flex; align-items: flex-end; gap: 3px; min-height: 86px; padding-top: 14px; }
  .drawn-gap { width: 18px; }
  .auto-btn.on { background: rgba(76, 201, 240, 0.3); border-color: #4cc9f0; }
  .auto-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    background: rgba(76, 201, 240, 0.18);
    border: 1px solid #4cc9f0;
    border-radius: 12px;
    padding: 6px 8px 6px 14px;
    font-size: 15px;
  }
  .hint { font-size: 12px; color: var(--muted); }
  .spect { align-self: center; }
  .side { display: flex; flex-direction: column; gap: 10px; min-height: 0; padding: 14px; }
  .side-head { display: flex; justify-content: space-between; align-items: center; }
  .side-btns { display: flex; gap: 8px; flex-wrap: wrap; }
  .open-side { position: absolute; right: 12px; top: 12px; z-index: 5; }
</style>

<script>
  import { net, act, hostCmd, disconnect, clock } from './net.svelte.js';
  import Tile from './Tile.svelte';
  import Melds from './Melds.svelte';
  import Avatar from './Avatar.svelte';
  import { reactions } from './reactions.svelte.js';
  import { getCharacter, pickLine } from './characters.js';

  const s = $derived(net.state);
  const r = $derived(s.result);
  const winner = $derived(r?.winner != null ? s.seats[r.winner] : null);
  const nextAct = $derived(s.actions.find((a) => a.type === 'next'));
  const remaining = $derived(
    s.deadline_ms == null ? null : Math.max(0, Math.ceil((s.deadline_ms - (clock.now - net.receivedAt)) / 1000)),
  );
  const dealerInvolved = $derived(r && r.payments.some((p) => p.from === s.dealer || p.to === s.dealer));
  const winIdx = $derived(winner?.hand ? winner.hand.indexOf(r.win_tile) : -1);
  const loser = $derived(r?.loser != null ? s.seats[r.loser] : null);
  // 贏家的台詞：沿用剛剛泡泡講的那句，沒有就再抽一句
  const winLine = $derived(
    winner ? reactions[r.winner]?.text ?? pickLine(winner.character, r.self_draw ? 'tsumo' : 'ron') : '',
  );
  const ranking = $derived([...s.seats].sort((a, b) => b.score - a.score));
  const medals = ['🥇', '🥈', '🥉', '4'];
</script>

<div class="overlay">
  {#if s.phase === 'game_over'}
    <div class="card box">
      <h2>🏁 遊戲結束</h2>
      <div class="rank">
        {#each ranking as p, i}
          <div class="rrow" class:me={p.seat === net.seat}>
            <span class="medal">{medals[i]}</span>
            <span class="nm">{p.name}</span>
            <span class="pts" class:neg={p.score < 0}>{p.score > 0 ? '+' : ''}{p.score}</span>
          </div>
        {/each}
      </div>
      <div class="btns">
        {#if net.isHost}
          <button class="primary" onclick={() => hostCmd('start')}>再來一場</button>
          <button onclick={() => hostCmd('reset')}>回大廳</button>
        {/if}
        <button onclick={disconnect}>回首頁</button>
      </div>
    </div>
  {:else if r}
    <div class="card box">
      {#if r.draw}
        <h2>🌊 流局</h2>
        <p class="muted">牌摸完了，莊家連莊。</p>
      {:else}
        {#if winner?.character}
          <div class="cutin">
            <div class="portrait"><Avatar id={winner.character} expression="win" size={170} /></div>
            <div class="say">
              <div class="who">{getCharacter(winner.character).name}</div>
              <div class="line">「{winLine}」</div>
            </div>
            {#if loser?.character}
              <div class="loser" title="{loser.name} 放槍">
                <Avatar id={loser.character} expression="sad" size={64} crop />
              </div>
            {/if}
          </div>
        {/if}
        <h2>🀄 {r.summary}</h2>
        {#if winner}
          <div class="hand">
            {#each winner.hand ?? [] as t, i}
              <Tile code={t} size="md" hl={i === winIdx} />
            {/each}
            <span class="gap"></span>
            <Melds melds={winner.melds} flowers={winner.flowers} size="md" />
          </div>
          {#if r.win_tile}
            <p class="muted wt">胡：<Tile code={r.win_tile} size="sm" /> {r.self_draw ? '（自摸）' : ''}</p>
          {/if}
        {/if}
        <div class="items">
          {#each r.items as it}
            <div class="item"><span>{it.name}</span><b>{it.tai} 台</b></div>
          {/each}
          {#if dealerInvolved}
            {#each r.dealer_items as it}
              <div class="item dealer"><span>{it.name}<small>（莊家相關）</small></span><b>{it.tai} 台</b></div>
            {/each}
          {/if}
          {#if r.items.length === 0}
            <div class="item"><span>屁胡</span><b>0 台</b></div>
          {/if}
        </div>
        <div class="pay">
          {#each r.payments as p}
            <div>{s.seats[p.from].name} → {s.seats[p.to].name}：{p.tai} 台，{p.amount}</div>
          {/each}
        </div>
      {/if}
      <div class="deltas">
        {#each s.seats as p, i}
          <div class="d" class:me={i === net.seat}>
            <span>{p.name}</span>
            <b class:neg={r.deltas[i] < 0} class:pos={r.deltas[i] > 0}>{r.deltas[i] > 0 ? '+' : ''}{r.deltas[i]}</b>
            <small>{p.score}</small>
          </div>
        {/each}
      </div>
      <div class="btns">
        {#if nextAct}
          <button class="primary" onclick={() => act(nextAct.id)}>下一局{remaining != null ? `（${remaining}）` : ''}</button>
        {:else}
          <span class="muted">等待其他玩家…{remaining != null ? `（${remaining}）` : ''}</span>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .overlay {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.35);
    z-index: 20;
    animation: fade 0.25s ease;
  }
  @keyframes fade { from { opacity: 0; } }
  .box { min-width: min(420px, 94%); max-width: min(860px, 94%); max-height: 92%; overflow: auto; background: rgba(10, 32, 22, 0.96); }
  h2 { margin: 0 0 14px; color: var(--gold); }
  .cutin {
    display: flex;
    align-items: flex-end;
    gap: 14px;
    margin: -20px -20px 12px;
    padding: 0 20px;
    background: linear-gradient(100deg, rgba(255, 209, 102, 0.28), rgba(255, 209, 102, 0) 70%);
    border-bottom: 1px solid rgba(255, 209, 102, 0.4);
    border-radius: 16px 16px 0 0;
    overflow: hidden;
  }
  .portrait { animation: slide 0.5s cubic-bezier(0.2, 1.3, 0.4, 1); margin-bottom: -4px; }
  @keyframes slide { from { transform: translateX(-60px); opacity: 0; } }
  .say { flex: 1; padding-bottom: 26px; animation: fade 0.6s ease 0.2s both; }
  .say .who { color: var(--gold); font-weight: 700; font-size: 14px; }
  .say .line { font-size: 22px; font-weight: 700; }
  .loser { padding-bottom: 18px; opacity: 0.9; }
  .hand { display: flex; flex-wrap: wrap; gap: 2px; align-items: flex-end; margin-bottom: 8px; }
  .gap { width: 14px; }
  .wt { display: flex; align-items: center; gap: 6px; margin: 4px 0 10px; }
  .items { display: grid; grid-template-columns: repeat(auto-fill, minmax(170px, 1fr)); gap: 6px; margin: 10px 0; }
  .item { display: flex; justify-content: space-between; background: rgba(255, 255, 255, 0.06); padding: 6px 10px; border-radius: 8px; }
  .item b { color: var(--gold); }
  .item.dealer { background: rgba(193, 18, 31, 0.18); }
  .item small { color: var(--muted); font-size: 11px; }
  .pay { font-size: 13px; color: var(--muted); margin-bottom: 10px; }
  .deltas { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; margin: 12px 0; }
  .d { display: flex; flex-direction: column; align-items: center; background: rgba(0, 0, 0, 0.3); border-radius: 10px; padding: 8px; font-size: 13px; }
  .d.me { outline: 1px solid var(--gold); }
  .d b { font-size: 20px; }
  .d small { color: var(--muted); }
  .neg { color: #ff8fa3; }
  .pos { color: var(--accent); }
  .btns { display: flex; gap: 10px; justify-content: center; align-items: center; margin-top: 8px; }
  .rank { display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px; }
  .rrow { display: flex; align-items: center; gap: 12px; background: rgba(255, 255, 255, 0.06); padding: 10px 16px; border-radius: 10px; font-size: 18px; }
  .rrow.me { outline: 1px solid var(--gold); }
  .medal { width: 30px; text-align: center; }
  .rrow .nm { flex: 1; }
  .pts { font-weight: 700; color: var(--accent); }
</style>

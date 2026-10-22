<script>
  import { formatMsg } from './msg.js';
  import { net, act, hostCmd, disconnect, clock } from './net.svelte.js';
  import Tile from './Tile.svelte';
  import Melds from './Melds.svelte';
  import Avatar from './Avatar.svelte';
  import FinalRanking from './FinalRanking.svelte';
  import { reactions } from './reactions.svelte.js';
  import { getCharacter, pickLine } from './characters.js';
  import { autoHints, taiInfo, taiLabel } from './hints.js';
  import { t, isZh } from './i18n.svelte.js';
  import { seatName } from './tiles.js';

  let openTai = $state(null);
  const toggleTai = (name) => (openTai = openTai === name ? null : name);

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
  // 胡牌的一句話：中文直接用引擎的 summary；其他語言用結構化欄位自己組（台數取付款中最高的那筆，含莊家台）
  const summary = $derived.by(() => {
    if (!r || r.draw) return '';
    if (r.summary_msg) return formatMsg(r.summary_msg, s.seats);
    if (isZh()) return r.summary;
    const tai = r.payments.length ? Math.max(...r.payments.map((p) => p.tai)) : r.hand_tai;
    const name = seatName(winner);
    let text = r.self_draw
      ? t('{name} 自摸！{tai} 台', { name, tai })
      : loser
        ? t('{name} 胡 {loser} 放的槍！{tai} 台', { name, loser: seatName(loser), tai })
        : t('{name} 胡牌！{tai} 台', { name, tai });
    // 阿翼的銀之盾、阿烈的身經百戰：那家這次不用付
    for (const p of r.payments) {
      if (p.amount !== 0) continue;
      const who = seatName(s.seats[p.from]);
      if (!r.self_draw && r.summary?.includes('銀之盾')) text += t('（{name} 的銀之盾擋下了付款）', { name: who });
      else if (r.self_draw && r.summary?.includes('身經百戰')) text += t('（{name} 的身經百戰：這次不用付）', { name: who });
    }
    return text;
  });
  // 贏家的台詞：沿用剛剛泡泡講的那句，沒有就再抽一句
  const winLine = $derived(
    winner ? reactions[r.winner]?.text ?? pickLine(winner.character, r.self_draw ? 'tsumo' : 'ron') : '',
  );
</script>

<div class="overlay">
  {#if s.phase === 'game_over'}
    <FinalRanking />
  {:else if r}
    <div class="card box hand-result">
      {#if r.draw}
        <h2>🌊 {t('流局')}</h2>
        <p class="muted">{t('牌摸完了，莊家連莊。')}</p>
      {:else}
        {#if winner?.character}
          <div class="cutin">
            <div class="portrait"><Avatar id={winner.character} expression="win" size={170} /></div>
            <div class="say">
              <div class="who">{t(getCharacter(winner.character).name)}</div>
              <div class="line">{t('「{line}」', { line: t(winLine) })}</div>
            </div>
            {#if loser?.character}
              <div class="loser" title={t('{name} 放槍', { name: seatName(loser) })}>
                <Avatar id={loser.character} expression="sad" size={64} crop />
              </div>
            {/if}
          </div>
        {/if}
        <h2>🀄 {summary}</h2>
        {#if winner}
          <div class="hand">
            {#each winner.hand ?? [] as t, i}
              <Tile code={t} size="md" hl={i === winIdx} />
            {/each}
            <span class="gap"></span>
            <Melds melds={winner.melds} flowers={winner.flowers} size="md" />
          </div>
          {#if r.win_tile}
            <p class="muted wt">{t('胡：')}<Tile code={r.win_tile} size="sm" /> {r.self_draw ? t('（自摸）') : ''}</p>
          {/if}
        {/if}
        <!-- 台數：點一下看說明 -->
        <div class="items">
          {#each r.items as it}
            <button type="button" class="item" class:open={openTai === it.name} title={taiInfo(it.name)} onclick={() => toggleTai(it.name)}><span>{taiLabel(it.name)}</span><b>{t('{n} 台', { n: it.tai })}</b></button>
          {/each}
          {#if dealerInvolved}
            {#each r.dealer_items as it}
              <button type="button" class="item dealer" class:open={openTai === it.name} title={taiInfo(it.name)} onclick={() => toggleTai(it.name)}><span>{taiLabel(it.name)}<small>{t('（莊家相關）')}</small></span><b>{t('{n} 台', { n: it.tai })}</b></button>
            {/each}
          {/if}
          {#if r.items.length === 0}
            <div class="item"><span>{taiLabel('屁胡')}</span><b>{t('{n} 台', { n: 0 })}</b></div>
          {/if}
        </div>
        <div class="pay">
          {#if openTai && taiInfo(openTai)}
            <!-- 說明浮在下面的內容上，不撐高結果視窗；再點一下關掉 -->
            <button type="button" class="tai-info" onclick={() => (openTai = null)}><b>{taiLabel(openTai)}</b>{t('：')}{taiInfo(openTai)}</button>
          {/if}
          {#each r.payments as p, i}
            <div>{t('{from} → {to}：{tai} 台，{amount}', { from: seatName(s.seats[p.from]), to: seatName(s.seats[p.to]), tai: p.tai, amount: p.amount })}{#if $autoHints && i === r.payments.length - 1}<span class="tai-tip">{t('（點台數看說明）')}</span>{/if}</div>
          {/each}
        </div>
      {/if}
      <div class="deltas">
        {#each s.seats as p, i}
          <div class="d" class:me={i === net.seat}>
            <span>{seatName(p)}</span>
            <b class:neg={r.deltas[i] < 0} class:pos={r.deltas[i] > 0}>{r.deltas[i] > 0 ? '+' : ''}{r.deltas[i]}</b>
            <small>{p.score}</small>
          </div>
        {/each}
      </div>
      <div class="btns">
        {#if nextAct}
          <button class="primary" onclick={() => act(nextAct.id)}>{remaining != null ? t('下一局（{n}）', { n: remaining }) : t('下一局')}</button>
        {:else}
          <span class="muted">{remaining != null ? t('等待其他玩家…（{n}）', { n: remaining }) : t('等待其他玩家…')}</span>
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
  /* 視窗不高時立繪縮小，結果視窗才不用捲動 */
  .portrait :global(.avatar) { width: clamp(110px, 21vh, 170px) !important; height: clamp(110px, 21vh, 170px) !important; }
  @keyframes slide { from { transform: translateX(-60px); opacity: 0; } }
  .say { flex: 1; padding-bottom: 26px; animation: fade 0.6s ease 0.2s both; }
  .say .who { color: var(--gold); font-weight: 700; font-size: 14px; }
  .say .line { font-size: 22px; font-weight: 700; }
  .loser { padding-bottom: 18px; opacity: 0.9; }
  .hand { display: flex; flex-wrap: wrap; gap: 2px; align-items: flex-end; margin-bottom: 8px; }
  .gap { width: 14px; }
  .wt { display: flex; align-items: center; gap: 6px; margin: 4px 0 10px; }
  .items { display: grid; grid-template-columns: repeat(auto-fill, minmax(170px, 1fr)); gap: 6px; margin: clamp(6px, 1.2vh, 10px) 0; }
  .item { display: flex; justify-content: space-between; background: rgba(255, 255, 255, 0.06); padding: 6px 10px; border-radius: 8px; }
  .item b { color: var(--gold); }
  :global(html[data-ui-style]) button.item { width: 100%; min-height: 0; border: 1px solid transparent; border-radius: 8px; font: inherit; color: inherit; text-align: left; cursor: pointer; background: rgba(255, 255, 255, 0.06); box-shadow: none; padding: 6px 10px; letter-spacing: normal; }
  :global(html[data-ui-style]) button.item.dealer { background: rgba(193, 18, 31, 0.18); }
  :global(html[data-ui-style]) button.item:hover, :global(html[data-ui-style]) button.item.open { border-color: var(--gold); }
  .pay { position: relative; }
  :global(html[data-ui-style]) .tai-info { position: absolute; left: 0; right: 0; top: -2px; z-index: 2; min-height: 0; margin: 0; padding: 6px 10px; font: 13px/1.5 var(--ui-font, inherit); letter-spacing: normal; text-align: left; color: var(--text); background: var(--art-panel); border: 1px solid var(--gold); border-radius: 8px; box-shadow: 0 6px 16px #000a; cursor: pointer; }
  .tai-info b { color: var(--gold); }
  .tai-tip { margin-left: 6px; color: var(--muted); font-size: 11px; }
  .item.dealer { background: rgba(193, 18, 31, 0.18); }
  /* 英文：台數名稱後面加了英文說明，格子放寬一點 */
  :global(html[lang='en']) .items { grid-template-columns: repeat(auto-fill, minmax(230px, 1fr)); }
  :global(html[lang='en']) .item { gap: 8px; text-align: left; }
  :global(html[lang='en']) .item b { flex: none; }
  .item small { color: var(--muted); font-size: 11px; }
  .pay { font-size: 13px; color: var(--muted); margin-bottom: 10px; }
  .deltas { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; margin: clamp(6px, 1.4vh, 12px) 0; }
  .d { display: flex; flex-direction: column; align-items: center; background: rgba(0, 0, 0, 0.3); border-radius: 10px; padding: 8px; font-size: 13px; }
  .d.me { outline: 1px solid var(--gold); }
  .d b { font-size: 20px; }
  .d small { color: var(--muted); }
  /* Game Boy 只有四種綠，muted 在分數格上太暗，改用最亮的那種綠 */
  :global(html[data-ui-style='gameboy']) .d small { color: #9bbc0f; }
  .neg { color: #ff8fa3; }
  .pos { color: var(--accent); }
  .btns { display: flex; gap: 10px; justify-content: center; align-items: center; margin-top: 8px; }

  /* 手機橫向（畫面很矮）：左邊贏家與手牌、右邊台數與分數，不用捲動 */
  @media (max-height: 620px) and (orientation: landscape) {
    .box { max-height: 96%; padding: 10px 14px; }
    .hand-result { display: grid; grid-template-columns: minmax(0, 1.3fr) minmax(0, 1fr); grid-auto-flow: dense; column-gap: 14px; align-items: start; width: min(900px, 96%); max-width: 96%; }
    .hand-result > * { grid-column: 2; }
    .hand-result > :is(.cutin, h2, .hand, .wt) { grid-column: 1; }
    .hand-result > .cutin { margin: -10px 0 6px -14px; padding: 0 12px; }
    .portrait :global(.avatar) { width: clamp(64px, 24vh, 110px) !important; height: clamp(64px, 24vh, 110px) !important; }
    .say { padding-bottom: 10px; }
    .say .line { font-size: 16px; }
    h2 { font-size: 16px; margin: 0 0 6px; }
    .hand :global(.tile) { --w: clamp(16px, 3vw, 28px); }
    .wt { margin: 2px 0; }
    .items { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 4px; margin: 0 0 4px; }
    .item { padding: 3px 8px; font-size: 13px; }
    .pay { font-size: 12px; margin-bottom: 4px; }
    .deltas { gap: 4px; margin: 4px 0; }
    .d { padding: 4px; font-size: 12px; }
    .d b { font-size: 16px; }
    .btns { margin-top: 4px; }
  }
</style>

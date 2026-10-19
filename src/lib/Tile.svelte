<script>
  import { parseCode, NUM_ZH, tileName } from './tiles.js';

  let {
    code = '?',
    size = 'md',
    back = false,
    selected = false,
    hl = false,
    dim = false,
    clickable = false,
    onclick = undefined,
    ondblclick = undefined,
  } = $props();

  const t = $derived(back ? { kind: 'back' } : parseCode(code));

  const BLUE = '#1d4e89';
  const GREEN = '#1f7a3a';
  const RED = '#c1121f';

  // 筒子：點的位置 (viewBox 60x80)
  const DOTS = {
    1: [[30, 40]],
    2: [[30, 21], [30, 59]],
    3: [[15, 15], [30, 40], [45, 65]],
    4: [[17, 20], [43, 20], [17, 60], [43, 60]],
    5: [[16, 17], [44, 17], [30, 40], [16, 63], [44, 63]],
    6: [[18, 14], [42, 14], [18, 41], [42, 41], [18, 66], [42, 66]],
    7: [[13, 11], [30, 20], [47, 29], [18, 50], [42, 50], [18, 69], [42, 69]],
    8: [[18, 11], [42, 11], [18, 30], [42, 30], [18, 50], [42, 50], [18, 69], [42, 69]],
    9: [[13, 14], [30, 14], [47, 14], [13, 40], [30, 40], [47, 40], [13, 66], [30, 66], [47, 66]],
  };
  function dotColor(n, i) {
    switch (n) {
      case 1: return RED;
      case 2: return [GREEN, BLUE][i];
      case 3: return [BLUE, RED, GREEN][i];
      case 4: return [BLUE, GREEN, GREEN, BLUE][i];
      case 5: return i === 2 ? RED : [BLUE, GREEN, GREEN, BLUE][i > 2 ? i - 1 : i];
      case 6: return i < 2 ? GREEN : RED;
      case 7: return i < 3 ? GREEN : RED;
      case 8: return BLUE;
      case 9: return [BLUE, RED, GREEN][Math.floor(i / 3)];
    }
    return BLUE;
  }
  const dotR = (n) => (n === 1 ? 17 : n <= 2 ? 11 : n <= 4 ? 10 : n <= 6 ? 9 : 7.6);

  // 條子：竹子的位置
  const STICKS = {
    2: [[30, 21], [30, 59]],
    3: [[30, 21], [18, 59], [42, 59]],
    4: [[20, 21], [40, 21], [20, 59], [40, 59]],
    5: [[15, 21], [45, 21], [30, 40], [15, 59], [45, 59]],
    6: [[14, 21], [30, 21], [46, 21], [14, 59], [30, 59], [46, 59]],
    7: [[30, 13], [14, 40], [30, 40], [46, 40], [14, 67], [30, 67], [46, 67]],
    // 八條：上半部是 M 形（中間兩根往內靠），下半部是 W 形（中間兩根往外開）；第三個值是傾斜角度
    8: [[10, 21], [24, 21, 22], [36, 21, -22], [50, 21], [10, 59], [24, 59, -22], [36, 59, 22], [50, 59]],
    9: [[14, 13], [30, 13], [46, 13], [14, 40], [30, 40], [46, 40], [14, 67], [30, 67], [46, 67]],
  };
  function stickColor(n, i) {
    if (n === 5 && i === 2) return RED;
    if (n === 7 && i === 0) return RED;
    if (n === 9 && i % 3 === 1) return RED;
    return GREEN;
  }
  const stickH = (n) => (n === 7 || n === 9 ? 22 : n === 5 ? 24 : 32);

  const honorColor = (c) => (c === 'C' ? RED : c === 'F' ? GREEN : BLUE);
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="tile {size}"
  class:back={t.kind === 'back'}
  class:selected
  class:hl
  class:dim
  class:clickable
  title={t.kind === 'back' ? '' : tileName(code)}
  {onclick}
  {ondblclick}
>
  {#if t.kind === 'm'}
    <div class="man">
      <span class="num">{NUM_ZH[t.n - 1]}</span>
      <span class="wan">萬</span>
    </div>
  {:else if t.kind === 'p'}
    <svg viewBox="0 0 60 80">
      {#each DOTS[t.n] as [x, y], i}
        {@const r = dotR(t.n)}
        <circle cx={x} cy={y} r={r} fill={dotColor(t.n, i)} />
        <circle cx={x} cy={y} r={r * 0.62} fill="none" stroke="#fffaf0" stroke-width={r * 0.18} />
        <circle cx={x} cy={y} r={r * 0.22} fill="#fffaf0" />
        {#if t.n === 1}
          <circle cx={x} cy={y} r={r * 0.9} fill="none" stroke={GREEN} stroke-width="2" />
        {/if}
      {/each}
    </svg>
  {:else if t.kind === 's'}
    <svg viewBox="0 0 60 80">
      {#if t.n === 1}
        <g transform="translate(30 40)">
          <ellipse cx="0" cy="8" rx="13" ry="18" fill={GREEN} />
          <circle cx="0" cy="-16" r="9" fill={GREEN} />
          <circle cx="3" cy="-18" r="2.2" fill="#fffaf0" />
          <path d="M8 -16 L17 -13 L8 -11 Z" fill={RED} />
          <path d="M-10 10 Q-22 26 -6 30 M10 10 Q22 26 6 30" stroke={RED} stroke-width="3" fill="none" />
          <path d="M-5 0 Q0 12 5 0" stroke="#fffaf0" stroke-width="2" fill="none" />
        </g>
      {:else}
        {#each STICKS[t.n] as [x, y, angle = 0], i}
          {@const h = stickH(t.n)}
          {@const w = t.n === 8 ? 7 : 8}
          <g transform="rotate({angle} {x} {y})">
            <rect x={x - w / 2} y={y - h / 2} width={w} height={h} rx={w / 2} fill={stickColor(t.n, i)} />
            <line x1={x - w / 2} x2={x + w / 2} y1={y} y2={y} stroke="#fffaf0" stroke-width="1.6" />
            <line x1={x} x2={x} y1={y - h / 2 + 3} y2={y + h / 2 - 3} stroke="#fffaf0" stroke-opacity=".45" stroke-width="1.2" />
          </g>
        {/each}
      {/if}
    </svg>
  {:else if t.kind === 'z'}
    {#if t.c === 'P'}
      <div class="white"></div>
    {:else}
      <span class="honor" style:color={honorColor(t.c)}>{t.ch}</span>
    {/if}
  {:else if t.kind === 'f'}
    <div class="flower" class:season={t.n <= 4}>
      <span class="fch">{t.ch}</span>
      <span class="fno">{((t.n - 1) % 4) + 1}</span>
    </div>
  {/if}
</div>

<style>
  .tile {
    --w: 40px;
    width: var(--w);
    height: calc(var(--w) * 1.36);
    flex: none;
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: calc(var(--w) * 0.14);
    background: linear-gradient(170deg, #fffef9 0%, #f6efdc 70%, #eadfc3 100%);
    box-shadow:
      0 calc(var(--w) * 0.06) 0 #d8c89f,
      0 calc(var(--w) * 0.12) 0 #1b6b47,
      0 calc(var(--w) * 0.16) calc(var(--w) * 0.2) rgba(0, 0, 0, 0.35);
    margin-bottom: calc(var(--w) * 0.12);
    transition: transform 0.12s ease, box-shadow 0.12s ease;
    user-select: none;
    font-family: 'DFKai-SB', 'BiauKai', '標楷體', 'Kaiti TC', 'KaiTi', 'Noto Serif TC', serif;
  }
  .xl { --w: 54px; }
  .lg { --w: 46px; }
  .md { --w: 34px; }
  .sm { --w: 28px; }
  .xs { --w: 20px; }
  .back {
    background: linear-gradient(160deg, #2f9e6e, #1b6b47);
    box-shadow:
      inset 0 0 0 calc(var(--w) * 0.06) #5cc497,
      0 calc(var(--w) * 0.12) 0 #e8dcc0,
      0 calc(var(--w) * 0.16) calc(var(--w) * 0.2) rgba(0, 0, 0, 0.35);
  }
  .clickable { cursor: pointer; }
  .clickable:hover { transform: translateY(-4px); }
  .selected { transform: translateY(-14px) !important; box-shadow: 0 0 0 3px #ffd166, 0 calc(var(--w) * 0.12) 0 #1b6b47, 0 8px 16px rgba(0,0,0,.45); }
  .hl { box-shadow: 0 0 0 3px #ff6b6b, 0 calc(var(--w) * 0.12) 0 #1b6b47, 0 0 14px #ff6b6b; }
  .dim { filter: brightness(0.82) saturate(0.7); }
  svg { width: 86%; height: 86%; }
  .man {
    display: flex;
    flex-direction: column;
    align-items: center;
    line-height: 1;
  }
  .num { font-size: calc(var(--w) * 0.5); color: #1d3557; font-weight: 700; }
  .wan { font-size: calc(var(--w) * 0.5); color: #c1121f; font-weight: 700; margin-top: calc(var(--w) * 0.04); }
  .honor { font-size: calc(var(--w) * 0.78); font-weight: 700; line-height: 1; }
  .white {
    width: 62%;
    height: 72%;
    border: calc(var(--w) * 0.07) solid #1d4e89;
    border-radius: calc(var(--w) * 0.08);
    box-shadow: inset 0 0 0 calc(var(--w) * 0.05) #fffaf0, inset 0 0 0 calc(var(--w) * 0.1) #1d4e89;
  }
  .flower { display: flex; flex-direction: column; align-items: center; line-height: 1; color: #7b2cbf; }
  .flower.season { color: #d35400; }
  .fch { font-size: calc(var(--w) * 0.62); font-weight: 700; }
  .fno { font-size: calc(var(--w) * 0.3); font-family: system-ui, sans-serif; font-weight: 700; margin-top: 2px; }
</style>

<script>
  // 二次元角色頭像。內建用 SVG 畫；角色設定有 images 時改用圖片。
  import { getCharacter, getCharacterImage } from './characters.js';
  import { artStyle, isPixelStyle } from './art-style.js';

  /** expression: normal | happy | surprised | sad | win */
  let { id, expression = 'normal', size = 64, crop = false } = $props();

  const c = $derived(getCharacter(id));
  const img = $derived(getCharacterImage(id, expression, $artStyle));
  const uid = `av${Math.random().toString(36).slice(2, 9)}`;

  const SKIN = '#ffe6d6';
  const SHADE = '#f6c9b5';
  const LINE = '#3a2a3f';

  // 用 HSL 粗略算出深一點的髮色，畫陰影與髮絲
  function darker(hex, amt = 0.22) {
    const n = parseInt(hex.slice(1), 16);
    const f = (x) => Math.max(0, Math.round(x * (1 - amt)));
    return `rgb(${f(n >> 16)}, ${f((n >> 8) & 255)}, ${f(n & 255)})`;
  }
  const hairDark = $derived(darker(c.hair));

  const view = $derived(crop ? '24 16 72 72' : '0 0 120 120');

  const BANGS = {
    soft: 'M22 58 C17 12 103 12 98 58 Q93 50 90 40 Q85 52 76 55 Q77 44 71 36 Q65 51 54 55 Q56 44 50 36 Q44 50 35 53 Q34 45 31 40 Q27 50 22 58 Z',
    spiky:
      'M22 58 L18 32 L28 34 L26 16 L40 26 L46 6 L56 22 L66 4 L72 22 L86 10 L88 30 L102 28 L98 58 L92 44 L84 54 L80 38 L72 54 L66 38 L58 54 L52 38 L44 54 L40 40 L32 54 L28 44 Z',
    parted: 'M21 62 C15 10 105 10 99 62 C95 48 86 38 72 32 C66 44 52 52 36 54 C30 55 25 58 21 62 Z',
  };
</script>

<div class="avatar" class:crop style:width="{size}px" style:height="{size}px">
  {#if img}
    <img src={img} alt={c.name} class:pixel={isPixelStyle($artStyle)} />
  {:else}
    <svg viewBox={view} xmlns="http://www.w3.org/2000/svg" aria-label={c.name}>
      <defs>
        <linearGradient id="{uid}-iris" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stop-color={darker(c.eyes, 0.45)} />
          <stop offset="0.55" stop-color={c.eyes} />
          <stop offset="1" stop-color={c.eyes} stop-opacity="0.65" />
        </linearGradient>
        <linearGradient id="{uid}-hair" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stop-color={c.hair} />
          <stop offset="1" stop-color={hairDark} />
        </linearGradient>
      </defs>

      <!-- 後髮 -->
      {#if c.hairStyle === 'twintails'}
        <path d="M30 36 C4 46 0 88 12 114 C20 98 22 74 32 56 Z" fill="url(#{uid}-hair)" />
        <path d="M90 36 C116 46 120 88 108 114 C100 98 98 74 88 56 Z" fill="url(#{uid}-hair)" />
      {:else if c.hairStyle === 'ponytail'}
        <path d="M84 24 C120 22 122 78 104 108 C102 86 98 62 82 44 Z" fill="url(#{uid}-hair)" />
      {:else if c.hairStyle === 'bun'}
        <circle cx="60" cy="13" r="14" fill={c.hair} />
        <path d="M50 10 Q60 2 70 10" stroke={hairDark} stroke-width="1.5" fill="none" />
      {:else if c.hairStyle === 'doublebun'}
        <circle cx="31" cy="20" r="12" fill={c.hair} />
        <circle cx="89" cy="20" r="12" fill={c.hair} />
      {/if}
      {#if c.hairStyle === 'long'}
        <path d="M24 46 C18 80 16 100 20 120 L100 120 C104 100 102 80 96 46 Z" fill="url(#{uid}-hair)" />
      {/if}
      <ellipse cx="60" cy="50" rx="38" ry="39" fill={c.hair} />

      <!-- 身體 -->
      <path d="M12 120 C14 103 34 95 60 96 C86 95 106 103 108 120 Z" fill={c.clothes} />
      <path d="M12 120 C14 103 34 95 60 96 C86 95 106 103 108 120" fill="none" stroke="rgba(0,0,0,.15)" stroke-width="1" />
      <rect x="53" y="80" width="14" height="19" rx="5" fill={SKIN} />
      <path d="M53 88 Q60 94 67 88 L67 92 Q60 97 53 92 Z" fill={SHADE} />
      <path d="M44 97 L60 112 L76 97 L68 95 L60 104 L52 95 Z" fill={c.collar} />

      <!-- 臉 -->
      <path d="M31 46 C31 72 44 88 60 91 C76 88 89 72 89 46 Z" fill={SKIN} />
      <ellipse cx="31" cy="64" rx="4" ry="6" fill={SKIN} />
      <ellipse cx="89" cy="64" rx="4" ry="6" fill={SKIN} />

      <!-- 腮紅 -->
      <ellipse cx="41" cy="73" rx="6" ry="3" fill="#ff8fa3" opacity={expression === 'sad' ? 0.25 : 0.45} />
      <ellipse cx="79" cy="73" rx="6" ry="3" fill="#ff8fa3" opacity={expression === 'sad' ? 0.25 : 0.45} />

      <!-- 眼睛 -->
      {#each [{ x: 47, left: true }, { x: 73, left: false }] as e}
        {#if expression === 'happy' || (expression === 'win' && e.left)}
          <path d="M{e.x - 7} 64 Q{e.x} 55 {e.x + 7} 64" stroke={LINE} stroke-width="2.8" fill="none" stroke-linecap="round" />
        {:else}
          {@const big = expression === 'surprised'}
          <ellipse cx={e.x} cy="63" rx={big ? 7.6 : 7} ry={big ? 9.4 : 8.6} fill="#fff" />
          <ellipse cx={e.x} cy={expression === 'sad' ? 65 : 64} rx={big ? 4.2 : 5.8} ry={big ? 5.2 : 7.4} fill="url(#{uid}-iris)" />
          <ellipse cx={e.x} cy={expression === 'sad' ? 66 : 65} rx={big ? 1.8 : 2.6} ry={big ? 2.4 : 3.6} fill="#1b1430" />
          <circle cx={e.x - 2.2} cy="60.5" r="2.1" fill="#fff" />
          <circle cx={e.x + 2.4} cy="67.5" r="1" fill="#fff" />
          <path
            d="M{e.x - 8.5} {big ? 56 : 58} Q{e.x} {big ? 49 : 51.5} {e.x + 8.5} {big ? 56 : 57.5}"
            stroke={LINE}
            stroke-width="2.6"
            fill="none"
            stroke-linecap="round"
          />
          <path
            d={e.left ? `M${e.x - 8.5} 58 l-2.5 -2` : `M${e.x + 8.5} 57.5 l2.5 -2`}
            stroke={LINE}
            stroke-width="2"
            stroke-linecap="round"
          />
        {/if}
      {/each}
      {#if expression === 'sad'}
        <path d="M76 70 q-2 4 0 6 q2 -2 0 -6 Z" fill="#8ecae6" />
      {/if}

      <!-- 鼻子、嘴巴 -->
      <path d="M60 71 l-1 2.5" stroke={SHADE} stroke-width="1.4" stroke-linecap="round" />
      {#if expression === 'happy' || expression === 'win'}
        <path d="M53 77 Q60 87 67 77 Z" fill="#c9485b" />
        <path d="M56 81.5 Q60 84.5 64 81.5 Q60 80 56 81.5 Z" fill="#ff8fa3" />
      {:else if expression === 'surprised'}
        <ellipse cx="60" cy="80" rx="3" ry="4" fill="#c9485b" />
      {:else if expression === 'sad'}
        <path d="M55 82 Q60 78 65 82" stroke="#b5525c" stroke-width="1.8" fill="none" stroke-linecap="round" />
      {:else}
        <path d="M56 79 Q60 82 64 79" stroke="#b5525c" stroke-width="1.8" fill="none" stroke-linecap="round" />
      {/if}

      <!-- 側髮 -->
      {#if c.hairStyle !== 'short'}
        <path d="M27 44 C22 66 23 86 28 100 C32 86 33 66 36 50 Z" fill="url(#{uid}-hair)" />
        <path d="M93 44 C98 66 97 86 92 100 C88 86 87 66 84 50 Z" fill="url(#{uid}-hair)" />
      {/if}

      <!-- 瀏海 -->
      <path d={BANGS[c.bangs] ?? BANGS.soft} fill="url(#{uid}-hair)" />
      <path d="M40 20 Q52 14 64 18" stroke="#fff" stroke-opacity=".35" stroke-width="3" fill="none" stroke-linecap="round" />

      <!-- 眉毛（難過、驚訝時看得到） -->
      {#if expression === 'sad'}
        <path d="M39 55 Q46 54 53 50" stroke={hairDark} stroke-width="2.2" fill="none" stroke-linecap="round" />
        <path d="M81 55 Q74 54 67 50" stroke={hairDark} stroke-width="2.2" fill="none" stroke-linecap="round" />
      {:else if expression === 'surprised'}
        <path d="M40 49 Q47 45 54 49" stroke={hairDark} stroke-width="2.2" fill="none" stroke-linecap="round" />
        <path d="M66 49 Q73 45 80 49" stroke={hairDark} stroke-width="2.2" fill="none" stroke-linecap="round" />
      {/if}

      <!-- 飾品 -->
      {#if c.accessory === 'ribbons'}
        {#each [30, 90] as x}
          <path d="M{x} 36 l-10 -8 l1 15 Z M{x} 36 l10 -8 l-1 15 Z" fill={c.accent} />
          <circle cx={x} cy="36" r="3.2" fill={darker(c.accent, 0.2)} />
        {/each}
      {:else if c.accessory === 'ribbon'}
        <path d="M84 22 l-11 -7 l1 15 Z M84 22 l11 -7 l-1 15 Z" fill={c.accent} stroke="rgba(0,0,0,.15)" />
        <circle cx="84" cy="22" r="3.2" fill={c.accent} stroke="rgba(0,0,0,.2)" />
      {:else if c.accessory === 'moon'}
        <path d="M86 22 a9 9 0 1 0 9 12 a7 7 0 1 1 -9 -12 Z" fill={c.accent} />
      {:else if c.accessory === 'headband'}
        <path d="M22 42 Q60 22 98 42" stroke={c.accent} stroke-width="6" fill="none" />
        <path d="M98 42 l10 -2 l-4 8 Z" fill={c.accent} />
      {:else if c.accessory === 'star'}
        <path
          d="M86 24 l2.6 5.4 5.9 .8 -4.3 4.1 1 5.9 -5.2 -2.8 -5.2 2.8 1 -5.9 -4.3 -4.1 5.9 -.8 Z"
          fill={c.accent}
          stroke="rgba(0,0,0,.2)"
        />
      {:else if c.accessory === 'glasses'}
        <rect x="37" y="56" width="20" height="15" rx="5" fill="rgba(255,255,255,.12)" stroke={c.accent} stroke-width="1.8" />
        <rect x="63" y="56" width="20" height="15" rx="5" fill="rgba(255,255,255,.12)" stroke={c.accent} stroke-width="1.8" />
        <path d="M57 62 Q60 60 63 62" stroke={c.accent} stroke-width="1.8" fill="none" />
      {:else if c.accessory === 'flower'}
        <circle cx="86" cy="24" r="6" fill={c.accent} />
      {/if}

      {#if expression === 'win'}
        <path d="M100 52 l2 5 5 2 -5 2 -2 5 -2 -5 -5 -2 5 -2 Z" fill="#ffd166" />
        <path d="M16 30 l1.4 3.4 3.4 1.4 -3.4 1.4 -1.4 3.4 -1.4 -3.4 -3.4 -1.4 3.4 -1.4 Z" fill="#ffd166" />
      {/if}
    </svg>
  {/if}
</div>

<style>
  .avatar { flex: none; position: relative; }
  .avatar.crop {
    border-radius: 50%;
    overflow: hidden;
    background: radial-gradient(circle at 50% 35%, #fff6e8, #ffd6a5);
    box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.7), 0 2px 8px rgba(0, 0, 0, 0.35);
  }
  svg, img { width: 100%; height: 100%; display: block; }
  img { object-fit: cover; object-position: top; }
  img.pixel { image-rendering: pixelated; }
</style>

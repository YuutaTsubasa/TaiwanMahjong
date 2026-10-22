<script>
  // 概念圖的角色選擇：大張半身立繪方格＋名牌，選中的有金框和菱形標記。
  import { CHARACTERS, getSelectableCharacters } from '../characters.js';
  import { ayiUnlocked } from '../unlocks.js';
  import Avatar from '../Avatar.svelte';
  import { t } from '../i18n.svelte.js';

  let { value = $bindable(), onpick = undefined, taken = [], columns = undefined, label = undefined } = $props();

  // 隱藏角色一律列出：還沒解鎖時是黑色剪影、名字「???」、不能選
  const list = CHARACTERS;
  const unlocked = $derived(new Set(getSelectableCharacters($ayiUnlocked).map((c) => c.id)));
  // 沒指定欄數時固定排成兩排，角色變多時欄數跟著加，畫面才不會變長
  const cols = $derived(columns ?? Math.ceil(list.length / 2));

  function pick(id) {
    value = id;
    onpick?.(id);
  }
</script>

<div class="pgrid" role="radiogroup" aria-label={label ?? t('選擇角色')} style:--cols={cols}>
  {#each list as c}
    {@const locked = !unlocked.has(c.id)}
    {@const isTaken = taken.includes(c.id) && value !== c.id}
    <button
      type="button"
      role="radio"
      class="cell"
      class:sel={value === c.id}
      class:taken={isTaken}
      class:locked
      aria-checked={value === c.id}
      aria-disabled={locked}
      aria-label={locked ? t('隱藏角色（尚未解鎖）') : t(c.name)}
      title={locked ? t('隱藏角色：尚未解鎖') : t(isTaken ? '{name}：{intro}（已有人選）' : '{name}：{intro}', { name: t(c.name), intro: t(c.intro) })}
      onclick={() => !locked && pick(c.id)}
    >
      <span class="pic"><Avatar id={c.id} expression={value === c.id ? 'happy' : 'normal'} size={220} /></span>
      <span class="name">{locked ? '???' : t(c.name)}</span>
      {#if value === c.id}<span class="diamond" aria-hidden="true"></span>{/if}
    </button>
  {/each}
</div>

<style>
  .pgrid { display: grid; grid-template-columns: repeat(var(--cols), minmax(0, 1fr)); gap: clamp(8px, 1vw, 14px); }
  .cell {
    position: relative;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    padding: 0;
    overflow: visible;
    border-radius: var(--small-radius);
  }
  .pic {
    position: relative;
    display: block;
    aspect-ratio: 1.22;
    /* 外層可用 --pic-h 依視窗高度限制圖片高度（寬螢幕但不高的時候） */
    height: var(--pic-h, auto);
    overflow: hidden;
    background: radial-gradient(circle at 50% 30%, color-mix(in srgb, var(--gold) 22%, transparent), #0006 75%);
  }
  /* 依格子「高度」決定圖的大小：格子又寬又矮時（橫向螢幕）也一定看得到頭和肩膀 */
  .pic :global(.avatar) {
    position: absolute;
    left: 50%;
    top: -3%;
    height: 152% !important;
    width: auto !important;
    aspect-ratio: 1;
    transform: translateX(-50%);
  }
  .name {
    display: block;
    padding: 4px 2px 6px;
    font-family: var(--heading-font);
    font-size: clamp(14px, min(1.3vw, 2.4vh), 20px);
    letter-spacing: 0.2em;
    text-align: center;
    color: var(--text);
    border-top: 1px solid var(--art-line);
  }
  /* 英文名字比較長：字距縮小，放不下就省略 */
  /* 英文名字比較長：字縮小、不加字距 */
  :global(html[lang='en']) .name { letter-spacing: 0; font-size: clamp(9px, min(0.95vw, 1.8vh), 15px); line-height: 1.15; overflow-wrap: anywhere; }
  .sel .name { color: var(--gold); }
  .taken { opacity: 0.5; }
  /* 未解鎖：整張壓成黑色剪影 */
  .locked { cursor: not-allowed; }
  .locked .pic :global(.avatar) { filter: brightness(0) drop-shadow(0 0 1px #fff4); opacity: 0.85; }
  .locked .name { color: var(--muted); letter-spacing: 0.35em; }
  .diamond {
    position: absolute;
    top: -7px;
    right: -7px;
    width: 18px;
    height: 18px;
    transform: rotate(45deg);
    background: var(--gold);
    border: 2px solid #3a2a12;
    box-shadow: 0 0 8px var(--gold);
  }
</style>

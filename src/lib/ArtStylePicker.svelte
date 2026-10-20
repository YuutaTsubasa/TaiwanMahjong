<script>
  import { ART_STYLES, artStyle, setArtStyle } from './art-style.js';
</script>

<div class="style-picker" role="group" aria-label="整體風格：介面與角色">
  <span class="label">整體風格</span>
  {#each ART_STYLES as style}
    <button
      type="button"
      class:selected={$artStyle === style.id}
      aria-pressed={$artStyle === style.id}
      title={style.description}
      onclick={() => setArtStyle(style.id)}
    >{style.name}</button>
  {/each}
</div>

<style>
  .style-picker {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    padding: 7px 16px;
    background: rgba(7, 26, 18, 0.92);
    border-bottom: 1px solid var(--line);
  }
  .label { font-size: 12px; color: var(--muted); margin-right: 6px; }
  button { padding: 5px 12px; font-size: 12px; border-radius: 7px; }
  button.selected { color: var(--gold); border-color: var(--gold); background: rgba(255, 209, 102, 0.15); }
  button:focus-visible { outline: 2px solid var(--gold); outline-offset: 2px; }
</style>

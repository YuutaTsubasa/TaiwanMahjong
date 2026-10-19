<script>
  import { CHARACTERS } from './characters.js';
  import Avatar from './Avatar.svelte';

  let { value = $bindable(), onpick = undefined, size = 72, taken = [] } = $props();

  function pick(id) {
    value = id;
    onpick?.(id);
  }
</script>

<div class="picker">
  {#each CHARACTERS as c}
    <button
      class="opt"
      class:sel={value === c.id}
      class:taken={taken.includes(c.id) && value !== c.id}
      title="{c.name}：{c.intro}"
      onclick={() => pick(c.id)}
    >
      <Avatar id={c.id} expression={value === c.id ? 'happy' : 'normal'} {size} crop />
      <span>{c.name}</span>
    </button>
  {/each}
</div>

<style>
  .picker { display: flex; flex-wrap: wrap; gap: 10px; }
  .opt {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 8px 10px 6px;
    border-radius: 14px;
    background: rgba(255, 255, 255, 0.05);
  }
  .opt span { font-size: 13px; }
  .opt.sel { background: rgba(255, 209, 102, 0.22); border-color: var(--gold); }
  .opt.taken { opacity: 0.5; }
</style>

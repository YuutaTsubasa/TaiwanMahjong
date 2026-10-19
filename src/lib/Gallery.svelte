<script>
  // 角色圖鑑（網址加上 #gallery）：檢查每個角色的每種表情，換自己的圖時也能用來預覽。
  import { CHARACTERS } from './characters.js';
  import Avatar from './Avatar.svelte';

  const EXPRESSIONS = ['normal', 'happy', 'surprised', 'sad', 'win'];
</script>

<main>
  <h1>角色圖鑑</h1>
  {#each CHARACTERS as c}
    <section class="card">
      <div class="head">
        <Avatar id={c.id} size={56} crop />
        <div><b>{c.name}</b> <code>{c.id}</code><div class="muted">{c.intro}</div></div>
      </div>
      <div class="row">
        {#each EXPRESSIONS as e}
          <figure><Avatar id={c.id} expression={e} size={150} /><figcaption>{e}</figcaption></figure>
        {/each}
      </div>
      <div class="lines muted">{Object.values(c.lines).flat().join('　')}</div>
    </section>
  {/each}
</main>

<style>
  main { height: 100%; overflow: auto; padding: 24px; display: flex; flex-direction: column; gap: 16px; }
  h1 { margin: 0; }
  .head { display: flex; gap: 12px; align-items: center; margin-bottom: 10px; }
  .row { display: flex; gap: 12px; flex-wrap: wrap; }
  figure { margin: 0; text-align: center; background: rgba(255, 255, 255, 0.06); border-radius: 12px; padding: 6px; }
  figcaption { font-size: 12px; color: var(--muted); }
  .lines { font-size: 12px; margin-top: 8px; }
</style>

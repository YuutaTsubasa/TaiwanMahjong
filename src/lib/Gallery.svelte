<script>
  import { getSelectableCharacters, getCharacter } from './characters.js';
  import { ayiUnlocked } from './unlocks.js';
  import Avatar from './Avatar.svelte';
  import Brand from './Brand.svelte';
  import OrnateHeading from './ui/OrnateHeading.svelte';
  import PortraitGrid from './ui/PortraitGrid.svelte';

  const EXPRESSIONS = [
    { id: 'normal', name: '常態', line: 'chow' },
    { id: 'happy', name: '開心', line: 'pung' },
    { id: 'surprised', name: '驚訝', line: 'draw' },
    { id: 'sad', name: '失落', line: 'deal_in' },
    { id: 'win', name: '勝利', line: 'tsumo' },
  ];
  let character = $state('xiaocui');
  let expression = $state('happy');
  const selected = $derived(getCharacter(character));
  const preview = $derived(EXPRESSIONS.find((item) => item.id === expression));
  const line = $derived(selected.lines[preview.line]?.[0] ?? '');
</script>

<main class="concept-page gallery-page">
  <header class="page-header">
    <Brand compact />
    <a class="back-link" href="#title">← 返回標題</a>
  </header>
  <div class="gallery-layout">
    <section class="art-panel selection-panel" aria-labelledby="gallery-heading">
      <OrnateHeading text="角色圖鑑" size="md" id="gallery-heading" />
      <div class="characters"><PortraitGrid bind:value={character} columns={2} label="選擇角色" /></div>
    </section>
    <div class="gallery-portrait" aria-label="{selected.name}的{preview.name}表情">
      <Avatar id={character} {expression} size={540} />
    </div>
    <section class="art-panel detail-panel" aria-label="角色介紹">
      <OrnateHeading text={selected.name} tag="h2" />
      <p class="intro">{selected.intro}。</p>
      <h3>表情預覽</h3>
      <div class="expressions" role="group" aria-label="選擇表情">
        {#each EXPRESSIONS as item}
          <button type="button" class="expression-option" class:selected={expression === item.id} aria-pressed={expression === item.id} onclick={() => (expression = item.id)}>
            <Avatar id={character} expression={item.id} size={70} />
            <span>{item.name}</span>
          </button>
        {/each}
      </div>
      <h3>台詞欣賞</h3>
      <blockquote aria-live="polite">「{line}」</blockquote>
    </section>
  </div>
</main>

<style>
  .gallery-page { padding: clamp(8px, 1.8vh, 34px) clamp(16px, 2.2vw, 34px); background-image: linear-gradient(0deg, #021b1855, transparent), var(--scene-parlor); }
  .page-header { display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-bottom: clamp(6px, 1.6vh, 24px); min-height: clamp(44px, 7vh, 58px); }
  .back-link { color: var(--gold); text-decoration: none; border: 1px solid var(--line); padding: 10px 18px; background: #071c18d9; white-space: nowrap; }
  .gallery-layout { display: grid; grid-template-columns: minmax(210px, 27%) minmax(0, 1fr) minmax(320px, 36%); align-items: center; gap: 14px; }
  .selection-panel { padding: clamp(10px, 2vh, 20px); }
  .characters { margin-top: clamp(6px, 1.4vh, 16px); --pic-h: clamp(52px, 14.5vh, 170px); }
  .character-option, .expression-option { display: flex; flex-direction: column; align-items: center; overflow: hidden; padding: 0; border: 1px solid #9a7839; border-radius: 2px; background: #001f1b; min-width: 0; }
  .character-option :global(.avatar), .expression-option :global(.avatar) { width: 100% !important; height: auto !important; aspect-ratio: 1; background: #795d3c55; }
  .character-option span { display: block; padding: 6px; font: 600 19px 'DFKai-SB', '標楷體', serif; letter-spacing: .16em; color: var(--gold); }
  button.selected { border-color: #ffe995; box-shadow: 0 0 0 2px #caa344, 0 0 12px #e8c45e88; }
  .gallery-portrait { align-self: end; min-width: 0; margin: 0 -55px 0 -35px; filter: drop-shadow(0 6px 14px #0008); }
  .gallery-portrait :global(.avatar) { width: 100% !important; height: auto !important; aspect-ratio: 1; }
  .gallery-portrait :global(img) { object-fit: contain; }
  .detail-panel { padding: clamp(18px, 2.2vw, 34px); position: relative; }
  .detail-panel :global(.ornate) { margin: 4px 0 6px; }
  .intro { text-align: center; line-height: 1.8; font-size: 16px; }
  h3 { display: flex; align-items: center; gap: 14px; color: var(--gold); font: 600 20px 'DFKai-SB', '標楷體', serif; letter-spacing: .16em; text-align: center; margin: clamp(10px, 2.4vh, 26px) 0 clamp(8px, 1.8vh, 18px); }
  h3::before, h3::after { content: ''; flex: 1; height: 1px; background: var(--line); }
  .expressions { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 8px; }
  .expression-option span { font-size: 12px; padding: 6px 2px; white-space: nowrap; }
  blockquote { margin: 0; min-height: 94px; display: grid; place-items: center; border: 1px solid var(--line); background: #0005; padding: 14px; font: 20px/1.7 'DFKai-SB', '標楷體', serif; color: #ffecc5; }
  @media (max-width: 1050px) {
    .gallery-layout { grid-template-columns: minmax(220px, 32%) minmax(0, 1fr); }
    .gallery-portrait { grid-column: 2; grid-row: 1; max-width: 400px; justify-self: center; margin: 0; }
    .detail-panel { grid-column: 1 / -1; }
    .expressions { max-width: 480px; margin: auto; }
  }
  @media (max-width: 550px) {
    .page-header { flex-wrap: wrap; margin-bottom: 18px; }
    .gallery-layout { grid-template-columns: 1fr; }
    .selection-panel { padding: 16px; }
    .characters :global(.pgrid) { --cols: 3 !important; }
    .gallery-portrait { grid-column: 1; grid-row: auto; width: 75%; max-width: 320px; }
    .character-option span { font-size: 16px; }
  }
</style>

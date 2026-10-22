<script>
  import { getSelectableCharacters, getCharacter, SKILLS } from './characters.js';
  import { ayiUnlocked } from './unlocks.js';
  import Avatar from './Avatar.svelte';
  import Brand from './Brand.svelte';
  import OrnateHeading from './ui/OrnateHeading.svelte';
  import PortraitGrid from './ui/PortraitGrid.svelte';
  import { onDestroy } from 'svelte';
  import { previewTheme } from './audio/sound.svelte.js';
  import { t } from './i18n.svelte.js';

  const EXPRESSIONS = [
    { id: 'normal', name: '常態', line: 'chow' },
    { id: 'happy', name: '開心', line: 'pung' },
    { id: 'surprised', name: '驚訝', line: 'draw' },
    { id: 'sad', name: '失落', line: 'deal_in' },
    { id: 'win', name: '勝利', line: 'tsumo' },
  ];
  let character = $state('xiaocui');
  // 選到誰就播誰的主題曲；離開圖鑑回到標題曲
  $effect(() => previewTheme(character));
  onDestroy(() => previewTheme(null));
  let expression = $state('happy');
  const selected = $derived(getCharacter(character));
  const preview = $derived(EXPRESSIONS.find((item) => item.id === expression));
  const skill = $derived(SKILLS[character]);
  // 名字的讀音（日文介面才有，例如 小翠 → シャオツイ）
  const reading = $derived.by(() => {
    const key = `讀音：${selected.name}`;
    const r = t(key);
    return r === key ? '' : r;
  });
  const line = $derived(t(selected.lines[preview.line]?.[0] ?? ''));
</script>

<main class="concept-page gallery-page">
  <header class="page-header">
    <Brand compact />
    <a class="back-link" href="#title">← {t('返回標題')}</a>
  </header>
  <div class="gallery-layout">
    <section class="art-panel selection-panel" aria-labelledby="gallery-heading">
      <OrnateHeading text={t('角色圖鑑')} size="md" id="gallery-heading" />
      <div class="characters"><PortraitGrid bind:value={character} columns={3} label={t('選擇角色')} /></div>
    </section>
    <div class="gallery-portrait" aria-label={t('{name}的{expression}表情', { name: t(selected.name), expression: t(preview.name) })}>
      <Avatar id={character} {expression} size={540} />
    </div>
    <section class="art-panel detail-panel" aria-label={t('角色介紹')}>
      <OrnateHeading text={t(selected.name)} tag="h2" />
      {#if reading}<p class="reading">{reading}</p>{/if}
      <p class="intro">{t('{intro}。', { intro: t(selected.intro) })}</p>
      {#if skill}<p class="skill-line" title={t('技能模式開啟時可以使用')}><b>{t('技能「{name}」', { name: t(skill.name) })}</b>{t(skill.desc)}</p>{/if}
      <h3>{t('表情預覽')}</h3>
      <div class="expressions" role="group" aria-label={t('選擇表情')}>
        {#each EXPRESSIONS as item}
          <button type="button" class="expression-option" class:selected={expression === item.id} aria-pressed={expression === item.id} onclick={() => (expression = item.id)}>
            <Avatar id={character} expression={item.id} size={70} />
            <span>{t(item.name)}</span>
          </button>
        {/each}
      </div>
      <h3>{t('台詞欣賞')}</h3>
      <blockquote aria-live="polite">{t('「{line}」', { line })}</blockquote>
    </section>
  </div>
</main>

<style>
  .gallery-page { padding: clamp(8px, 1.8vh, 34px) clamp(16px, 2.2vw, 34px); background-image: linear-gradient(0deg, #021b1855, transparent), var(--scene-parlor); }
  .page-header { display: flex; align-items: center; justify-content: space-between; gap: 16px; margin-bottom: clamp(6px, 1.6vh, 24px); min-height: clamp(44px, 7vh, 58px); }
  .back-link { color: var(--gold); text-decoration: none; border: 1px solid var(--line); padding: 10px 18px; background: #071c18d9; white-space: nowrap; }
  .gallery-layout { display: grid; grid-template-columns: minmax(210px, 27%) minmax(0, 1fr) minmax(320px, 36%); align-items: center; gap: 14px; }
  .selection-panel { padding: clamp(10px, 2vh, 20px); }
  .characters { margin-top: clamp(6px, 1.4vh, 16px); --pic-h: clamp(52px, 10vh, 120px); }
  .character-option, .expression-option { display: flex; flex-direction: column; align-items: center; overflow: hidden; padding: 0; border: 1px solid #9a7839; border-radius: 2px; background: #001f1b; min-width: 0; }
  .character-option :global(.avatar), .expression-option :global(.avatar) { width: 100% !important; height: auto !important; aspect-ratio: 1; background: #795d3c55; }
  .character-option span { display: block; padding: 6px; font: 600 19px var(--kai-font); letter-spacing: .16em; color: var(--gold); }
  button.selected { border-color: #ffe995; box-shadow: 0 0 0 2px #caa344, 0 0 12px #e8c45e88; }
  .gallery-portrait { align-self: end; min-width: 0; margin: 0 -55px 0 -35px; filter: drop-shadow(0 6px 14px #0008); }
  /* 立繪寬度也受視窗高度限制，不然寬螢幕時會比畫面還高 */
  .gallery-portrait :global(.avatar) { width: min(100%, calc(100vh - 190px)) !important; height: auto !important; aspect-ratio: 1; margin: 0 auto; }
  .gallery-portrait :global(img) { object-fit: contain; }
  .detail-panel { padding: clamp(18px, 2.2vw, 34px); position: relative; }
  .detail-panel :global(.ornate) { margin: 4px 0 6px; }
  .intro { text-align: center; line-height: 1.8; font-size: 16px; }
  .reading { margin: -6px 0 2px; text-align: center; font-size: 14px; letter-spacing: .12em; color: var(--muted); }
  .skill-line { margin: 4px 0 0; text-align: center; font-size: 14px; line-height: 1.6; color: var(--text); }
  .skill-line b { color: var(--gold); margin-right: 6px; }
  h3 { display: flex; align-items: center; gap: 14px; color: var(--gold); font: 600 20px var(--kai-font); letter-spacing: .16em; text-align: center; margin: clamp(10px, 2.4vh, 26px) 0 clamp(8px, 1.8vh, 18px); }
  :global(html:not([lang^='zh'])) h3 { letter-spacing: .04em; }
  h3::before, h3::after { content: ''; flex: 1; height: 1px; background: var(--line); }
  .expressions { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 8px; }
  /* 文字顏色要自己指定：有些畫風（MS-DOS）的按鈕預設是深色字，會跟深色底融在一起 */
  .expression-option span { font-size: 12px; padding: 6px 2px; white-space: nowrap; color: var(--gold); }
  /* 選中的那格各畫風會換底色，字色跟著畫風的按鈕走 */
  .expression-option.selected span { color: inherit; }
  blockquote { margin: 0; min-height: 94px; display: grid; place-items: center; border: 1px solid var(--line); background: #0005; padding: 14px; font: 20px/1.7 var(--kai-font); color: #ffecc5; }
  /* 英文台詞：楷書的英文字母像打字機，改用標題字型（英文時是襯線體） */
  :global(html:not([lang^='zh'])) blockquote { font-family: var(--heading-font); font-size: 18px; line-height: 1.5; }
  @media (max-width: 900px) {
    .gallery-layout { grid-template-columns: minmax(220px, 32%) minmax(0, 1fr); }
    .gallery-portrait { grid-column: 2; grid-row: 1; max-width: 400px; justify-self: center; margin: 0; }
    .detail-panel { grid-column: 1 / -1; }
    .expressions { max-width: 480px; margin: auto; }
  }
  /* 手機橫向（畫面很矮）：角色格、立繪、介紹排成三欄，整頁不用捲動 */
  @media (max-height: 620px) and (orientation: landscape) {
    .gallery-page { padding: 4px 10px; }
    .page-header { min-height: 34px; margin-bottom: 4px; }
    .page-header :global(.logo) { max-height: 30px; width: auto; }
    .back-link { padding: 3px 12px; font-size: 13px; }
    .gallery-layout { grid-template-columns: minmax(0, 30%) minmax(0, 1fr) minmax(0, 40%); align-items: end; gap: 8px; }
    .gallery-portrait { grid-column: auto; grid-row: auto; max-width: none; justify-self: stretch; margin: 0; }
    .gallery-portrait :global(.avatar) { width: min(100%, calc(100vh - 100px)) !important; }
    .detail-panel { grid-column: auto; padding: 6px 12px; }
    .detail-panel :global(.ornate) { margin: 0; }
    .detail-panel :global(.ornate .text), .selection-panel :global(.ornate .text) { font-size: 18px; }
    .selection-panel { padding: 6px 8px; }
    .characters { margin-top: 4px; --pic-h: clamp(30px, 10vh, 56px); }
    .characters :global(.pgrid) { --cols: 4 !important; gap: 4px; }
    .characters :global(.name) { font-size: 11px; padding: 1px; letter-spacing: 0; }
    .intro { font-size: 13px; line-height: 1.45; margin: 0; }
    .skill-line { font-size: 11px; line-height: 1.4; }
    h3 { margin: 4px 0; font-size: 14px; }
    .expressions { gap: 4px; max-width: none; }
    .expression-option span { font-size: 10px; padding: 1px; }
    blockquote { min-height: 0; padding: 4px 8px; font-size: 14px; line-height: 1.5; }
  }
  @media (max-height: 400px) and (orientation: landscape) {
    .page-header { min-height: 28px; margin-bottom: 2px; }
    blockquote { font-size: 13px; }
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

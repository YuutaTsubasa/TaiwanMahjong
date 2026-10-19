// 音樂與音效的總管：設定（音量、靜音）、音效、依牌局狀態切換 BGM 與短曲。
import { audioContext, buses, fmNote, ssgNote, drum, noiseHit, setBusVolume, PATCHES } from './synth.js';
import { Player, SONGS } from './music.js';

function load(key, fallback) {
  try {
    const v = localStorage.getItem(key);
    return v == null ? fallback : JSON.parse(v);
  } catch {
    return fallback;
  }
}
function save(key, v) {
  try {
    localStorage.setItem(key, JSON.stringify(v));
  } catch {}
}

export const soundSettings = $state({
  bgm: load('mj_bgm_volume', 0.5),
  sfx: load('mj_sfx_volume', 0.7),
  muted: load('mj_muted', false),
});

function applyVolumes() {
  if (!audioContext()) return;
  const m = soundSettings.muted ? 0 : 1;
  setBusVolume('bgm', soundSettings.bgm * m);
  setBusVolume('sfx', soundSettings.sfx * m);
}

export function setVolume(which, v) {
  soundSettings[which] = v;
  save(which === 'bgm' ? 'mj_bgm_volume' : 'mj_sfx_volume', v);
  applyVolumes();
}

export function toggleMute() {
  soundSettings.muted = !soundSettings.muted;
  save('mj_muted', soundSettings.muted);
  applyVolumes();
}

// 瀏覽器規定要使用者先互動過才能出聲：第一次點擊或按鍵時啟動
let unlocked = false;
function unlock() {
  const ctx = audioContext();
  if (!ctx) return;
  applyVolumes();
  const go = () => {
    if (unlocked) return;
    unlocked = true;
    if (wantedBgm && !jingle.playing) startBgm(wantedBgm);
  };
  if (ctx.state === 'suspended') ctx.resume().then(go);
  else go();
}
if (typeof window !== 'undefined') {
  window.addEventListener('pointerdown', unlock, { capture: true });
  window.addEventListener('keydown', unlock, { capture: true });
  // 一般按鈕的輕微按鍵聲（牌桌上的動作按鈕另外有音效，所以略過 .act）
  window.addEventListener('click', (e) => {
    const b = e.target.closest?.('button');
    if (b && !b.classList.contains('act')) sfx.click();
  });
}
const ready = () => unlocked && audioContext()?.state === 'running';

// ───────── BGM ─────────
const bgm = new Player();
const jingle = new Player();
let wantedBgm = null;
let currentBgm = null;

function startBgm(name) {
  currentBgm = name;
  if (!name) return bgm.stop(0.8);
  bgm.play(SONGS[name]);
}

export function setBgm(name) {
  wantedBgm = name;
  if (!ready() || jingle.playing || name === currentBgm) return;
  startBgm(name);
}

export function playJingle(name) {
  if (!ready() || !SONGS[name]) return;
  bgm.stop(0.25);
  currentBgm = null;
  jingle.play(SONGS[name], {
    onEnd: () => {
      if (wantedBgm) startBgm(wantedBgm);
    },
  });
}

// ───────── 音效 ─────────
PATCHES.wood = {
  level: 0.3,
  env: { a: 0.001, d: 0.06, s: 0, r: 0.03 },
  mods: [{ ratio: 2.41, index: 1.8, env: { a: 0.001, d: 0.03, s: 0, r: 0.02 } }],
};

const now = () => audioContext().currentTime + 0.01;
const out = () => buses.sfx;

export const sfx = {
  /** 打牌：牌敲在桌上的「喀」 */
  discard(mine = false) {
    if (!ready()) return;
    const t = now();
    noiseHit(out(), t, { type: 'bandpass', freq: 3200, q: 1.2, vol: mine ? 0.5 : 0.35, decay: 0.035 });
    fmNote(out(), 'wood', mine ? 86 : 81, t, 0.04, 0.9);
  },
  /** 吃、碰 */
  claim() {
    if (!ready()) return;
    const t = now();
    for (const m of [62, 66, 69]) fmNote(out(), 'brass', m, t, 0.16, 0.9);
    drum(out(), 's', t);
  },
  kong() {
    if (!ready()) return;
    const t = now();
    for (const m of [57, 62, 66, 69]) fmNote(out(), 'brass', m, t, 0.3, 1);
    drum(out(), 'k', t);
    drum(out(), 'c', t);
  },
  /** 胡牌瞬間的重擊（之後接短曲） */
  winHit() {
    if (!ready()) return;
    const t = now();
    for (const m of [62, 66, 69, 74]) fmNote(out(), 'brass', m, t, 0.35, 1.1);
    drum(out(), 'k', t);
    drum(out(), 'c', t);
  },
  /** 補花：閃亮的琶音 */
  flower() {
    if (!ready()) return;
    const t = now();
    [79, 83, 86, 91].forEach((m, i) => fmNote(out(), 'bell', m, t + i * 0.06, 0.2, 0.8));
  },
  /** 輪到你 */
  turn() {
    if (!ready()) return;
    const t = now();
    ssgNote(out(), 76, t, 0.06, 1.2);
    ssgNote(out(), 83, t + 0.08, 0.1, 1.2);
  },
  /** 可以吃碰槓胡（胡的時候更急促） */
  alert(win = false) {
    if (!ready()) return;
    const t = now();
    const notes = win ? [84, 88, 91, 96] : [81, 81, 88];
    notes.forEach((m, i) => ssgNote(out(), m, t + i * 0.07, 0.05, 1.3));
  },
  /** 倒數最後幾秒 */
  tick() {
    if (!ready()) return;
    ssgNote(out(), 96, now(), 0.03, 0.9);
  },
  /** 一般按鈕 */
  click() {
    if (!ready()) return;
    noiseHit(out(), now(), { type: 'highpass', freq: 5000, vol: 0.08, decay: 0.02 });
  },
};

// ───────── 依牌局狀態決定要放什麼 ─────────
let lastSeq = null;
let lastDecision = null;
let lastPhase = null;

/** 每次收到新狀態時呼叫 */
export function onGameState(state) {
  if (!state) {
    lastSeq = null;
    lastDecision = null;
    lastPhase = null;
    setBgm('lobby');
    return;
  }
  const me = state.you;
  const auto = me != null && state.seats[me]?.auto;

  // BGM
  if (state.phase === 'lobby') setBgm('lobby');
  else if (state.phase === 'playing') setBgm(state.wall_count != null && state.wall_count <= 16 ? 'tension' : 'game');
  else if (state.phase === 'game_over') setBgm('lobby');
  else setBgm(null);

  // 事件 → 音效與短曲（第一次只記進度，不重播舊事件）
  const events = state.events ?? [];
  const maxSeq = events.reduce((m, e) => Math.max(m, e.seq), 0);
  if (lastSeq === null || maxSeq < lastSeq) {
    lastSeq = maxSeq;
  } else {
    let tune = null;
    const rank = { otherwin: 1, draw: 1, lose: 2, win: 3 };
    const want = (name) => {
      if (!tune || rank[name] > rank[tune]) tune = name;
    };
    for (const e of events) {
      if (e.seq <= lastSeq) continue;
      switch (e.kind) {
        case 'discard':
          sfx.discard(e.seat === me);
          break;
        case 'flower':
          sfx.flower();
          break;
        case 'pung':
        case 'chow':
          sfx.claim();
          break;
        case 'kong':
          sfx.kong();
          break;
        case 'tsumo':
        case 'ron':
          sfx.winHit();
          want(e.seat === me ? 'win' : 'otherwin');
          break;
        case 'deal_in':
          if (e.seat === me) want('lose');
          break;
        case 'draw':
          want('draw');
          break;
      }
    }
    lastSeq = maxSeq;
    if (tune) setTimeout(() => playJingle(tune), 350);
  }

  if (state.phase === 'game_over' && lastPhase && lastPhase !== 'game_over') {
    setTimeout(() => playJingle('gameover'), 2500);
  }
  lastPhase = state.phase;

  // 輪到你／可以吃碰胡
  if (state.decision_id !== lastDecision) {
    lastDecision = state.decision_id;
    const acts = state.actions ?? [];
    if (!auto && state.phase === 'playing' && acts.length) {
      if (acts.some((a) => a.type === 'tsumo' || a.type === 'ron')) sfx.alert(true);
      else if (acts.some((a) => ['pung', 'kong', 'chow', 'concealed_kong', 'added_kong'].includes(a.type))) sfx.alert();
      else if (acts.some((a) => a.type === 'discard')) sfx.turn();
    }
  }
}

let lastTickSecond = null;
/** 倒數提示：有自己的決定、剩 5 秒以內時每秒嗶一聲 */
export function onCountdown(state, remainingMs) {
  const me = state?.you;
  const mine = me != null && state.phase === 'playing' && (state.actions?.length ?? 0) > 0 && !state.seats[me]?.auto;
  if (!mine || remainingMs == null) {
    lastTickSecond = null;
    return;
  }
  const sec = Math.ceil(remainingMs / 1000);
  if (sec <= 5 && sec > 0 && sec !== lastTickSecond) sfx.tick();
  lastTickSecond = sec;
}

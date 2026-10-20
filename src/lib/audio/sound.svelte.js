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
  // 介面音效：游標移動、確定、返回、選角色…（見 installUiSounds）
  installUiSounds();
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
    ssgNote(out(), 81, now(), 0.03, 1.4);
  },

  // ── 介面音效（PC-98 遊戲選單的感覺：游標移動嗶一聲、確定往上、取消往下） ──
  /** 游標移到另一個選項 */
  cursor() {
    if (!ready()) return;
    ssgNote(out(), 88, now(), 0.022, 1.2);
  },
  /** 確定（主要按鈕、選單項目） */
  confirm() {
    if (!ready()) return;
    const t = now();
    ssgNote(out(), 84, t, 0.045, 2);
    ssgNote(out(), 91, t + 0.055, 0.07, 2);
  },
  /** 返回、取消 */
  back() {
    if (!ready()) return;
    const t = now();
    ssgNote(out(), 86, t, 0.045, 1.8);
    ssgNote(out(), 79, t + 0.055, 0.07, 1.8);
  },
  /** 選角色 */
  pick() {
    if (!ready()) return;
    const t = now();
    fmNote(out(), 'bell', 84, t, 0.12, 0.8);
    fmNote(out(), 'bell', 91, t + 0.07, 0.2, 0.8);
  },
  /** 不能選（例如還沒解鎖的隱藏角色） */
  deny() {
    if (!ready()) return;
    const t = now();
    ssgNote(out(), 47, t, 0.06, 2);
    ssgNote(out(), 47, t + 0.09, 0.06, 2);
  },
  /** 選起一張手牌 */
  select() {
    if (!ready()) return;
    fmNote(out(), 'wood', 91, now(), 0.03, 0.6);
  },
};

// 依元件種類決定介面音效。用事件委派，所有頁面自動套用。
const INTERACTIVE = 'a[href], button, [role="radio"], [role="switch"], input[type="range"]';
const isBack = (el) =>
  el.matches('.back-link, a[href="#title"], .fancy.red, .close, .exit, .end, .open-side') ||
  /返回|取消|離開|回首頁|回標題|關閉/.test(el.textContent || '');

function uiSoundFor(el) {
  if (el.closest('.pgrid .cell')) return el.closest('.cell').classList.contains('locked') ? 'deny' : 'pick';
  if (el.closest('.hand') && el.matches('.tile, .tile *')) return null; // 手牌另外處理
  if (el.matches('.act')) return null; // 吃碰胡的音效等伺服器回應後才播
  if (el.matches('.copy-btn, .send, .seat-btn, .small')) return 'click';
  if (isBack(el)) return 'back';
  if (el.matches('.fancy.gold, .menu-item, .tabs button, .subtabs button, .seg button, .style-option, .auto-btn')) return 'confirm';
  return 'click';
}

let lastHover = null;
let lastHoverAt = 0;
function hoverSound(el) {
  if (!el || el === lastHover) return;
  lastHover = el;
  const t = performance.now();
  if (t - lastHoverAt < 45) return; // 滑過一排按鈕時不要連珠砲
  lastHoverAt = t;
  sfx.cursor();
}

function installUiSounds() {
  window.addEventListener('pointerover', (e) => {
    if (e.pointerType === 'touch') return;
    const el = e.target.closest?.(`${INTERACTIVE}, .hand .tile.clickable`);
    if (el && !el.disabled) hoverSound(el);
    else if (!el) lastHover = null;
  });
  // 鍵盤／手把移動焦點也嗶一聲（滑鼠點擊造成的 focus 不算）
  window.addEventListener('focusin', (e) => {
    const el = e.target;
    if (el?.matches?.(':focus-visible') && el.matches(`${INTERACTIVE}, .tile`)) hoverSound(el);
  });
  window.addEventListener(
    'click',
    (e) => {
      const tile = e.target.closest?.('.hand .tile.clickable');
      if (tile) {
        // 第一下是選牌（打出去時另有打牌的聲音）
        if (!tile.classList.contains('selected')) sfx.select();
        return;
      }
      const el = e.target.closest?.(INTERACTIVE);
      if (!el || el.disabled) return;
      const kind = uiSoundFor(el);
      if (kind) sfx[kind]();
    },
    { capture: true },
  );
}

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
    setBgm('title');
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
        case 'skill':
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

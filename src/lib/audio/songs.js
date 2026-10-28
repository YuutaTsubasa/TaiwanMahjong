// 台麻的曲目（全部原創）。播放器在 VirtualCore（virtual-core/audio/music.js）。
//
// 每首曲子由幾個音軌組成：
//   { kind: 'fm',     patch, mml, pan, vel }      FM 音色彈 MML
//   { kind: 'ssg',    mml, pan, vel }             方波彈 MML
//   { kind: 'chords', patch, prog, rhythm, vel }  依和弦進行自動伴奏
//       prog：'Dmaj7 Bm7 G7:2 C7:2'（預設一個和弦一小節，:n 表示 n 拍）
//       rhythm：一小節 8 格（八分音符）；x = 短促、W = 延長整個和弦
//   { kind: 'drums',  k, s, h, o, hits }          一小節 16 格，x 表示打；hits 是 [種類, 拍] 的額外打擊
//   { kind: 'bass',   patch, prog, pattern }       依和弦自動彈貝斯：pattern 一小節 8 格（八分音符）
//       r = 根音、5 = 五度、o = 高八度根音、3 = 三度、. = 休息
//   { kind: 'arp',    prog, pattern, oct, patch? } 依和弦自動琶音（十六分音符），pattern 是和弦音的編號，
//       例如 '0121'；沒給 patch 就用 SSG 方波
// 長度（拍數）以最長的旋律音軌為準。
// 曲子可以設 gain（0～1）調整整首的音量。

export const SONGS = {
  // 標題：夜晚的雀館。慢一點、帶點 city pop 的 F 大調爵士和弦
  title: {
    bpm: 92,
    loop: true,
    gain: 0.5, // 和弦層比較厚，整首壓小一點，跟其他曲子音量一致
    tracks: [
      {
        kind: 'fm',
        patch: 'lead',
        pan: 0.12,
        mml: `o5 l8 r c f a >c4< a g | e4. d c+4 e4 | d f a >c d4. c< | b-4 a g e4 g4 |
              a4. g a >c e4< | d4 f a >d4 c4< | c4 e g f+4 a4 | g4. f e4 r4`,
      },
      {
        kind: 'fm',
        patch: 'bass',
        mml: `o2 l8 f4. f >c4< a4 | e4. e a4 a4 | d4. d a4 >c4< | g4. g >c4 c4< |
              f4. f >c4< a4 | b-4. b- >f4< b-4 | a4. a d4 d4 | g4. g >c4 c4<`,
      },
      {
        kind: 'chords',
        patch: 'epiano',
        prog: 'Fmaj7 Em7:2 A7:2 Dm7 Gm7:2 C7:2 Fmaj7 Bbmaj7 Am7:2 D7:2 Gm7:2 C7:2',
        rhythm: '..x..x.x',
        pan: -0.3,
        vel: 0.6,
      },
      {
        kind: 'chords',
        patch: 'strings',
        prog: 'Fmaj7 Em7:2 A7:2 Dm7 Gm7:2 C7:2 Fmaj7 Bbmaj7 Am7:2 D7:2 Gm7:2 C7:2',
        rhythm: 'W.......',
        vel: 0.45,
      },
      { kind: 'drums', k: 'x.......x.....x.', s: '....x.......x...', h: 'x.x.x.x.x.x.x.x.', o: '..............x.' },
    ],
  },

  // 大廳：午後的麻將館，輕快的 F 大調
  lobby: {
    bpm: 120,
    loop: true,
    tracks: [
      {
        kind: 'fm',
        patch: 'lead',
        pan: 0.1,
        mml: `o5 l8 c f a >c< a4 f4 | d4 f a >d4< a4 | b-4. a g4 d4 | e4 g b- >c4.< r |
              c f a >c f4. e< | >c+4< a4 e4 c+4 | d e f a >d4 c4< | b- a g e f2`,
      },
      {
        kind: 'fm',
        patch: 'bass',
        mml: `o2 l4 f >c< f >c< | d a d a | g >d< g >d< | c g e g |
              f >c< f >c< | a >e< a >c+< | d a >c< a | g >d< c g`,
      },
      { kind: 'chords', patch: 'epiano', prog: 'F Dm Gm7 C7 F A7 Dm7 G7:2 C7:2', rhythm: '.x..x.x.', pan: -0.3 },
      { kind: 'drums', k: 'x.......x.x.....', s: '....x.......x...', h: 'x.x.x.x.x.x.x.x.' },
    ],
  },

  // 對局：沉穩的 D 大調，五聲音階的旋律
  game: {
    bpm: 108,
    loop: true,
    tracks: [
      {
        kind: 'fm',
        patch: 'lead',
        pan: 0.15,
        mml: `o5 l8 a4 f+ e d4 e f+ | f+4. e d <b> d4 | b4 a b >d4< b a | a2 r4 e f+ |
              a4 >d< a f+4 e d | e4. f+ e c+ <a4> | b4 g e f+4 g a | a2. r4`,
      },
      {
        kind: 'fm',
        patch: 'bass',
        mml: `o2 l8 d4. d a4 f+4 | b4. b f+4 d4 | g4. g >d4< b4 | a4. a >e4< g4 |
              d4. d a4 f+4 | f+4. f+ >c+4< a4 | e4. e b4 g4 | a4 a a g4 e4`,
      },
      { kind: 'chords', patch: 'epiano', prog: 'Dmaj7 Bm7 Gmaj7 A7 Dmaj7 F#m7 Em7 A7', rhythm: '.x...x.x', pan: -0.3 },
      { kind: 'chords', patch: 'strings', prog: 'Dmaj7 Bm7 Gmaj7 A7 Dmaj7 F#m7 Em7 A7', rhythm: 'W.......', vel: 0.8 },
      {
        kind: 'ssg',
        pan: 0.4,
        vel: 0.6,
        mml: `o6 l16 r2 a8 r8 f+8 r8 | r1 | r2 d8 r8 <b8> r8 | r2 c+8 r8 e8 r8 |
              r2 a8 r8 f+8 r8 | r1 | r2 g8 r8 e8 r8 | r2 c+8 e8 a4`,
      },
      { kind: 'drums', k: 'x.....x.x.......', s: '....x.......x..x', h: 'x.x.x.x.x.x.x.x.' },
    ],
  },

  // 海底（牌快摸完）：B 小調、快板，緊張感
  tension: {
    bpm: 144,
    loop: true,
    tracks: [
      {
        kind: 'fm',
        patch: 'brass',
        pan: 0.1,
        mml: `o5 l8 f+ f+ b4 a f+ d4 | g g b4 >d4< b4 | a a >c+4 e4 c+4< | a+4 f+4 e4 c+4 |
              f+ f+ b4 >d c+< b4 | g a b4 >d4 e4< | a a >c+4 e4 g4< | a+2 >c+2<`,
      },
      {
        kind: 'fm',
        patch: 'bass',
        mml: `o2 l8 [b b >b< b]2 | [g g >g< g]2 | [a a >a< a]2 | [f+ f+ >f+< f+]2 |
              [b b >b< b]2 | [g g >g< g]2 | [a a >a< a]2 | [f+ f+ >f+< f+]2`,
      },
      {
        kind: 'ssg',
        pan: -0.35,
        vel: 0.55,
        mml: `o5 l16 [b>df+d<]4 | [gb>d<b]4 | [a>c+ec+<]4 | [a+>c+f+c+<]4 |
              [b>df+d<]4 | [gb>d<b]4 | [a>c+ec+<]4 | [a+>c+f+c+<]4`,
      },
      { kind: 'drums', k: 'x...x...x...x...', s: '....x.......x.x.', h: 'xxxxxxxxxxxxxxxx' },
    ],
  },

  // ───── 短曲 ─────
  win: {
    bpm: 150,
    loop: false,
    tracks: [
      { kind: 'fm', patch: 'brass', pan: 0.15, mml: 'o5 l8 d f+ a >d4 c+< a >d2' },
      { kind: 'fm', patch: 'lead', pan: -0.2, vel: 0.7, mml: 'o4 l8 a >d f+ a4 a e f+2' },
      { kind: 'fm', patch: 'bass', mml: 'o2 l8 d d d d4 e4 d2' },
      { kind: 'ssg', vel: 0.6, mml: 'o6 l16 r2 r8 d f+ a >d4< r8 a8 >d2' },
      { kind: 'drums', k: 'x...x...x...', s: '....x..x..x.', h: 'x.x.x.x.x.x.', hits: [['c', 3.5], ['k', 3.5]] },
    ],
  },
  otherwin: {
    bpm: 150,
    loop: false,
    tracks: [
      { kind: 'fm', patch: 'brass', mml: 'o5 l8 a >d4 c+ d2<' },
      { kind: 'fm', patch: 'bass', mml: 'o2 l8 d >d4< a d2' },
      { kind: 'drums', hits: [['k', 0], ['s', 0.5], ['c', 1.5], ['k', 1.5]] },
    ],
  },
  lose: {
    bpm: 132,
    loop: false,
    tracks: [
      { kind: 'fm', patch: 'lead', mml: 'o5 l8 e d+ d c+ c4. r <b2' },
      { kind: 'fm', patch: 'bass', mml: 'o3 l8 c c c c <b4. r a+2' },
      { kind: 'drums', hits: [['k', 0], ['s', 1], ['k', 2], ['s', 3]] },
    ],
  },
  draw: {
    bpm: 112,
    loop: false,
    tracks: [
      { kind: 'fm', patch: 'epiano', mml: 'o5 l8 a >c e4 d c< b4 a2' },
      { kind: 'fm', patch: 'bass', mml: 'o2 l2 a e a' },
      { kind: 'chords', patch: 'strings', prog: 'Am:2 Em:2 Am:2', rhythm: 'W.......' },
    ],
  },
  gameover: {
    bpm: 132,
    loop: false,
    tracks: [
      { kind: 'fm', patch: 'brass', pan: 0.1, mml: 'o5 l8 d d d f+4 a4 >d4 c+ d e4 f+1' },
      { kind: 'fm', patch: 'lead', pan: -0.2, vel: 0.7, mml: 'o4 l8 a a a >d4 f+4 a4 a b >c+4 d1' },
      { kind: 'fm', patch: 'bass', mml: 'o2 l8 d d d d4 d4 g4 g g a4 d1' },
      { kind: 'chords', patch: 'strings', prog: 'D:5 G:1.5 A:1.5 D:4', rhythm: 'W.......' },
      { kind: 'drums', k: 'x.x.x...x...x...', s: '....x.......x...', h: 'x.x.x.x.x.x.x.x.', hits: [['c', 8], ['k', 8]] },
    ],
  },
};

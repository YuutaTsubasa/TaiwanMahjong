// 角色主題曲（全部原創，PC-98 FM 音色）。
// 當莊家時播莊家的主題曲；角色圖鑑選到誰就播誰的。
//
// 每首 16 小節（A 段 8 小節＋B 段 8 小節）循環。旋律用 MML 手寫，
// 貝斯、和弦、琶音依和弦進行自動產生（見 VirtualCore 的 audio/music.js 的 bass／chords／arp）。
// 旋律裡的 | 是小節線，只是方便閱讀；tests/music.test.js 會檢查每小節剛好 4 拍。

function theme({ bpm, lead, patch = 'lead', prog, comp, bass, arp, pad, drums, gain, counter }) {
  const tracks = [{ kind: 'fm', patch, pan: 0.12, mml: lead }];
  if (counter) tracks.push({ kind: 'ssg', pan: 0.35, vel: 0.55, mml: counter });
  tracks.push({ kind: 'bass', patch: 'bass', prog, pattern: bass });
  if (comp) tracks.push({ kind: 'chords', patch: comp.patch ?? 'epiano', prog, rhythm: comp.rhythm, pan: -0.3, vel: comp.vel ?? 0.7 });
  if (pad) tracks.push({ kind: 'chords', patch: 'strings', prog, rhythm: 'W.......', vel: pad });
  if (arp) tracks.push({ kind: 'arp', prog, pattern: arp.pattern, oct: arp.oct ?? 1, pan: -0.4, vel: arp.vel ?? 0.45, patch: arp.patch });
  if (drums) tracks.push({ kind: 'drums', ...drums });
  return { bpm, loop: true, gain, tracks };
}

// 鼓的節奏型
const ROCK = { k: 'x.....x.x.......', s: '....x.......x...', h: 'x.x.x.x.x.x.x.x.' };
const POP = { k: 'x.......x.x.....', s: '....x.......x...', h: 'x.x.x.x.x.x.x.x.' };
const DRIVE = { k: 'x...x...x...x...', s: '....x.......x.x.', h: 'xxxxxxxxxxxxxxxx' };
const LIGHT = { k: 'x.......x.......', s: '....x.......x...', h: '..x...x...x...x.' };
const SWING = { k: 'x.......x.......', s: '....x.......x...', h: 'x..xx..xx..xx..x' };
const MARCH = { k: 'x.......x.......', s: 'x.xxx.x.x.xxx.x.', h: '' };
const GALLOP = { k: 'x..x..x.x..x..x.', s: '....x.......x...', h: 'x.x.x.x.x.x.x.x.' };

export const CHARACTER_THEMES = {
  // 小翠：元氣少女。明亮彈跳的 C 大調
  xiaocui: theme({
    bpm: 144,
    lead: `o5 l8 e g >c4< g e c4 | d g b4 a g d4 | c e a4 g e c4 | f a >c4< a4 g4 |
           e g >c4 d4 e4< | >d4 c< b a4 g4 | a4 f a >c4< c4 | d e f g b4 r4 |
           a4 >c4< b a e4 | g4 b4 a g e4 | f4 a >c< f4 a4 | g2 e4 r4 |
           d f a >d4< c d4 | b4 a g f4 d4 | e g >c e d4 c4< | b2. r4`,
    prog: 'C G Am F C G F G Am Em F C Dm G C G',
    comp: { rhythm: '..x..x.x' },
    bass: 'r.r5o.5.',
    arp: { pattern: '0121', oct: 1 },
    drums: POP,
    gain: 0.85,
  }),

  // 月華：冷靜的大小姐。月光下的 A 小調，鐘聲般的旋律
  yuehua: theme({
    bpm: 92,
    patch: 'bell',
    lead: `o5 l4 a2 g e | f2. e | d f a >c< | b2 g+2 |
           a >c e d< | >c2< a2 | f e d f | e1 |
           a >c e2< | >d2< b g | g b >d e< | >c2< a2 |
           a f d f | g b >d f< | e2 g b | g+2 b2`,
    prog: 'Am Fmaj7 Dm7 E7 Am Fmaj7 Dm7 E7 Fmaj7 G Em7 Am Dm7 G Cmaj7 E7',
    comp: { patch: 'epiano', rhythm: 'x...x...', vel: 0.55 },
    pad: 0.7,
    bass: 'r...5...',
    drums: LIGHT,
    gain: 1.25,
  }),

  // 阿明：熱血少年。全速前進的 E 小調搖滾
  aming: theme({
    bpm: 152,
    patch: 'brass',
    lead: `o5 l8 e4 e g a4 g4 | e4 e g >c4< b4 | a4 f+ a >d4< a4 | b2 a g e4 |
           e4 e g a4 b4 | >c4 d e d4 c4< | d4 f+ a >d4 c+4< | b2 d+4 f+4 |
           g4 e g >c4 e4< | >d4< a >d f+4 d4< | >e2< b4 g4 | e g b >e d e4.< |
           >c4< b a g4 e4 | f+4 a4 >d4 f+4< | d+4 f+4 a4 b4 | b1`,
    prog: 'Em C D Em Em C D B7 C D Em Em C D B7 B7',
    comp: { patch: 'guitar', rhythm: 'x.x.x.x.', vel: 0.45 },
    bass: 'rrrrrrrr',
    drums: DRIVE,
    gain: 0.8,
  }),

  // 糖糖：愛撒嬌。蹦蹦跳跳的 F 大調
  tangtang: theme({
    bpm: 128,
    patch: 'bell',
    lead: `o5 l8 c f a f >c4< a4 | d f a f >d4< a4 | b- >d< b- f d4 f4 | e g >c4< g4 r4 |
           a >c< a f c4 f4 | d e f a >d4< c4 | b- a g f g4 b-4 | >c2< r2 |
           b-4 >d4 f4 d4< | >c4 e4 g4 e4< | a4 >c e< a4 e4 | f4 a4 >d2< |
           b- >d< b- g f4 d4 | e g b- g >c4 e4< | f4 a4 >c4 f4< | f2. r4`,
    prog: 'F Dm Bb C F Dm Gm7 C7 Bb C Am Dm Gm7 C7 F F',
    comp: { rhythm: '.x.x.x.x', vel: 0.55 },
    bass: 'r.5.r.5.',
    arp: { pattern: '0212', oct: 1, vel: 0.35 },
    drums: LIGHT,
    gain: 1.15,
  }),

  // 小春：運動少女。衝刺感的 G 大調
  xiaochun: theme({
    bpm: 152,
    patch: 'brass',
    lead: `o5 l8 d g b >d4< b g4 | e g >c e4 d c4< | d4 f+ a >d4< a4 | b2 g4 r4 |
           g a b >d4 e d4< | >e4 d< b g4 e4 | e g >c4< b a g4 | a2 f+4 d4 |
           b4 >e4< b g e4 | >c4 e4< g e c4 | d g b >d g4 d4< | >f+4 e d< a4 f+4 |
           g4 >c4 e4 g4< | f+4 a4 >d4 f+4< | >g2 d4< b4 | g1`,
    prog: 'G C D G G Em C D Em C G D C D G G',
    comp: { rhythm: 'x..x..x.', vel: 0.6 },
    bass: 'r.ro5.o.',
    arp: { pattern: '0120', oct: 1 },
    drums: ROCK,
    gain: 0.85,
  }),

  // 蘭姐：麻將館老闆娘。帶點慵懶爵士味的 D 小調
  lanjie: theme({
    bpm: 100,
    patch: 'flute',
    lead: `o5 l8 a4. f d4 f4 | g4. a b- a g4 | f4 a4 >c4. <a | g2 e4 c+4 |
           d4. e f a >c4< | d4. c <b- >d f4 | g4 b- a g4 f4 | e2. r4 |
           f4 a4 >d4 c4< | e4 g4 >c4.< b- | a4. g e4 c4 | d2 f4 a4 |
           b-4. a g4 d4 | e4 g4 b-4 g4 | a4. g f4 e4 | c+2 e2`,
    prog: 'Dm7 Gm7 Dm7 A7 Dm7 Bbmaj7 Gm7 A7 Bbmaj7 C Am7 Dm7 Gm7 C7 Fmaj7 A7',
    comp: { patch: 'epiano', rhythm: 'x..x..x.', vel: 0.6 },
    pad: 0.5,
    bass: 'r..5r..5',
    drums: SWING,
  }),

  // 蒼牙：狼族少年。野性十足的 D 小調
  cangya: theme({
    bpm: 144,
    patch: 'guitar',
    lead: `o5 l8 d4 d f a4 >d4< | >d4 c< b- f4 d4 | e g >c4 e4 c4< | d2 r d f a |
           >d4 c d f4 e d< | b-4 >d4< b- a f4 | g4 e g >c4< b-4 | a2 c+4 e4 |
           g4 b- >d< g4 b-4 | a4 >d f< a4 f4 | b-4. a b-4 >d4< | >c4. <b- >c4 e4< |
           >d4 c< b- g4 b-4 | a4 g e c+4 e4 | d f a >d f4 e4< | >d2.< r4`,
    prog: 'Dm Bb C Dm Dm Bb C A7 Gm Dm Bb C Gm A7 Dm Dm',
    comp: { patch: 'brass', rhythm: 'x..x..x.', vel: 0.4 },
    bass: 'rrorrror',
    drums: DRIVE,
    gain: 0.8,
  }),

  // 金爺：老牌賭客。煙霧繚繞的降 B 大調爵士
  jinye: theme({
    bpm: 96,
    patch: 'sax',
    lead: `o5 l8 f4 d f a4. f | g4. f d4 b-4 | e-4 g b- >d4< b-4 | a4. g e-4 c4 |
           d4 f a >c4< a4 | b-4. a g4 d4 | e-4 g4 b-4 g4 | f2 r4 c4 |
           g4 b- >d4< b- g4 | g-4 b- >d-4< b- g-4 | f4 a >c4< a f4 | b4 >d4< b g f4 |
           e-4. d c4 <b- >c | a4 >c4< a g f4 | d4 f4 a4 >d4< | >c2.< r4`,
    prog: 'Bbmaj7 Gm7 Cm7 F7 Dm7 Gm7 Cm7 F7 Ebmaj7 Ebm7 Dm7 G7 Cm7 F7 Bbmaj7 F7',
    comp: { patch: 'epiano', rhythm: '.x...x..', vel: 0.6 },
    bass: 'r.3.5.3.',
    drums: SWING,
  }),

  // 雷歐：王子騎士。堂堂正正的 C 大調進行曲
  leo: theme({
    bpm: 116,
    patch: 'brass',
    lead: `o5 l8 g4 >c4. c e4< | >f4 e< c a4 >c4< | >e4. d c4 e4< | >d2< b4 g4 |
           g4 >c4. e g4< | >a4 g e c4 e4< | >f4 a4 g4 f4< | >d2 r4< g4 |
           a4. b >c4 e4< | >d4. c< b4 g4 | a4 >c f4 e< c4 | e2 g2 |
           a4 >c4 f4 a4< | >g4. f d4< b4 | >c4 e4 g4 >c4<< | >c1<`,
    prog: 'C F C G C Am F G Am Em F C F G C C',
    comp: { patch: 'organ', rhythm: 'x...x...', vel: 0.6 },
    pad: 0.7,
    bass: 'r.5.r.5.',
    drums: MARCH,
    gain: 0.85,
  }),

  // 阿烈：紅髮傭兵。踏上旅途的 A 大調
  alie: theme({
    bpm: 132,
    patch: 'guitar',
    lead: `o5 l8 a4 e a >c+4< a4 | b4 g b >d4< b4 | a4 f+ a >d4 f+4< | >e2 c+2< |
           a >c+ e< a >c+4 e4< | >d4 d< b g4 d4 | f+4 a4 >d4. c+< | b2 g+4 e4 |
           f+4 a >c+4< a f+4 | f+4 a >d4< a f+4 | e4 a >c+ e4 c+4< | b2. g+4 |
           a4 >c+4 f+4 e4< | >d4 c+< b a4 f+4 | g+4 b4 >d4 e4< | >c+2< a2`,
    prog: 'A G D A A G D E7 F#m D A E F#m D E7 A',
    comp: { rhythm: 'x.xx.x.x', vel: 0.5 },
    bass: 'r.rr5.rr',
    arp: { pattern: '0102', oct: 1, vel: 0.35 },
    drums: GALLOP,
    gain: 0.85,
  }),

  // 阿翼：穿銀甲的隱藏來客。神秘又壯闊的 B 小調
  ayi: theme({
    bpm: 112,
    patch: 'brass',
    lead: `o5 l8 f+2 b4 >d4< | >d4. c+< b4 g4 | a2 f+4 a4 | e2. r4 |
           f+4 b4 >d4 f+4< | >g4. f+ d4< b4 | >e4 d< b g4 e4 | f+2 a+2 |
           b4. a g4 b4 | >c+4. <b a4 e4 | a4 f+4 c+4 f+4 | b2 >d2< |
           g4. f+ e4 g4 | a4 >c+4 e4 c+4< | >d4. c+ d4 f+4< | a+2. r4`,
    prog: 'Bm G D A Bm G Em F# G A F#m Bm Em A D F#',
    pad: 0.9,
    bass: 'r..rr..5',
    arp: { pattern: '0121', oct: 1, patch: 'bell', vel: 0.5 },
    drums: ROCK,
    gain: 0.8,
  }),
};

/** 角色的主題曲名稱（給 setBgm 用）；自訂角色沒有主題曲時回傳 null */
export function themeName(characterId) {
  return characterId && CHARACTER_THEMES[characterId] ? `char:${characterId}` : null;
}

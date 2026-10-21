// 常駐角色的 id、名字要跟 crates/mahjong-core/src/room.rs 的 CHARACTERS 一致。
//
// 想換成自己的圖：在 public/characters/<id>/ 放 PNG，然後在該角色加上
//   images: { normal: '/characters/<id>/normal.png', happy: '...', surprised: '...', sad: '...', win: '...' }
// 沒給的表情會退回 normal 那張；完全沒給 images 就用內建的 SVG 畫法。
//
// 外觀參數（給內建 SVG 用）：
//   hair      髮色          hairStyle  twintails | long | short | bun | doublebun | ponytail
//   bangs     soft | spiky | parted
//   eyes      瞳色          clothes    衣服顏色     collar  領子顏色
//   accessory ribbons | ribbon | moon | headband | star | glasses | flower
//   accent    飾品顏色

import { asset } from './site.js';
import { ART_STYLES } from './art-style.js';

// 90 年代賽璐璐動畫風格，五種表情均使用獨立透明 PNG。
function characterImages(id, style = 'anime90') {
  const directory = asset(`/characters/${id}${style === 'anime90' ? '' : `/${style}`}`);
  return Object.fromEntries(
    ['normal', 'happy', 'surprised', 'sad', 'win'].map((expression) => [
      expression, `${directory}/${expression}.png`,
    ]),
  );
}

export const CHARACTERS = [
  {
    id: 'xiaocui',
    images: characterImages('xiaocui'),
    name: '小翠',
    intro: '綠色雙馬尾的元氣少女',
    hair: '#3fbf86',
    hairStyle: 'twintails',
    bangs: 'soft',
    eyes: '#1f9e89',
    clothes: '#ffffff',
    collar: '#2a9d8f',
    accessory: 'ribbons',
    accent: '#ff5d8f',
    lines: {
      pung: ['碰！這張我要了～', '嘿嘿，碰！'],
      kong: ['槓！看我的！', '四張到齊～槓！'],
      chow: ['吃一下下～', '謝啦，我吃！'],
      tsumo: ['自摸！今天運氣超好！', '摸到了！耶～！'],
      ron: ['胡啦！謝謝你的牌～', '就等這張！胡！'],
      deal_in: ['嗚…怎麼是我放槍…', '啊啊啊打錯了啦！'],
      draw: ['流局了呢～下一局加油！'],
      skill: ['元氣再來！這張不算～', '再摸一次！嘿嘿～'],
    },
  },
  {
    id: 'yuehua',
    images: characterImages('yuehua'),
    name: '月華',
    intro: '銀髮的冷靜大小姐',
    hair: '#e6e9f2',
    hairStyle: 'long',
    bangs: 'parted',
    eyes: '#7b6cf6',
    clothes: '#2b2d42',
    collar: '#e6e9f2',
    accessory: 'moon',
    accent: '#ffd166',
    lines: {
      pung: ['碰。', '這張，我收下了。'],
      kong: ['槓。', '一切都在計算之中。'],
      chow: ['吃。', '順理成章。'],
      tsumo: ['自摸。這是必然的結果。', '月光眷顧著我呢。'],
      ron: ['胡了。你的牌，太好讀了。', '失禮了——胡。'],
      deal_in: ['……失算了。', '真是不甘心。'],
      draw: ['流局……這也不壞。'],
      skill: ['看穿了。你們在等什麼，我都知道。', '一切都在預料之中。'],
    },
  },
  {
    id: 'aming',
    images: characterImages('aming'),
    name: '阿明',
    intro: '熱血的刺蝟頭少年',
    hair: '#2f2f3a',
    hairStyle: 'short',
    bangs: 'spiky',
    eyes: '#d62828',
    clothes: '#f77f00',
    collar: '#fcbf49',
    accessory: 'headband',
    accent: '#d62828',
    lines: {
      pung: ['碰！燃起來了！', '這張是我的啦！'],
      kong: ['槓！！氣勢來了！', '看我的四連擊！'],
      chow: ['吃！衝啊！', '順子組起來！'],
      tsumo: ['自摸！！這就是我的實力！', '熱血自摸——！'],
      ron: ['胡！！一擊必殺！', '等的就是這一張！'],
      deal_in: ['可惡——！下次一定贏！', '我、我只是不小心！'],
      draw: ['流局？！我還沒燃燒完啊！'],
      skill: ['逆轉之魂，燃燒吧！', '還沒結束呢！'],
    },
  },
  {
    id: 'tangtang',
    images: characterImages('tangtang'),
    name: '糖糖',
    intro: '愛撒嬌的粉紅包包頭',
    hair: '#ff9ec7',
    hairStyle: 'doublebun',
    bangs: 'soft',
    eyes: '#ff5d8f',
    clothes: '#ffd6e8',
    collar: '#ff8fab',
    accessory: 'star',
    accent: '#ffd166',
    lines: {
      pung: ['人家要碰～♪', '碰碰碰～♡'],
      kong: ['槓～糖糖好厲害吧♡', '嘿嘿，槓了喔～'],
      chow: ['吃吃～謝謝你♡', '這張給糖糖吃～'],
      tsumo: ['自摸了～♡ 大家要給糖糖糖果喔！', '耶～自摸自摸♪'],
      ron: ['胡了～♡ 謝謝你送的牌！', '抓到你了～胡♡'],
      deal_in: ['嗚嗚…欺負人家…', '不算不算！再來一次嘛～'],
      draw: ['流局了～糖糖餓了…'],
      skill: ['拜託嘛～這張不要碰啦～', '糖糖的牌，不准吃喔～'],
    },
  },
  {
    id: 'xiaochun',
    images: characterImages('xiaochun'),
    name: '小春',
    intro: '橘色馬尾的運動少女',
    hair: '#f8961e',
    hairStyle: 'ponytail',
    bangs: 'soft',
    eyes: '#2a9d8f',
    clothes: '#277da1',
    collar: '#ffffff',
    accessory: 'ribbon',
    accent: '#ffffff',
    lines: {
      pung: ['碰！好球！', '接住了，碰！'],
      kong: ['槓！全壘打！', '再加一分！槓！'],
      chow: ['吃！傳球接力～', '吃！節奏很好喔！'],
      tsumo: ['自摸！比賽結束～！', '一桿進洞！自摸！'],
      ron: ['胡！得分！', '漂亮攔截——胡！'],
      deal_in: ['唔，被逆轉了…', '失誤了！下一局扳回來！'],
      draw: ['平手！延長賽見！'],
      skill: ['快攻！這張我接走了！', '衝啊——吃！'],
    },
  },
  {
    id: 'lanjie',
    images: characterImages('lanjie'),
    name: '蘭姐',
    intro: '戴眼鏡的麻將館老闆娘',
    hair: '#6a4c93',
    hairStyle: 'bun',
    bangs: 'parted',
    eyes: '#9d4edd',
    clothes: '#c1121f',
    collar: '#ffd166',
    accessory: 'glasses',
    accent: '#ffd166',
    lines: {
      pung: ['碰。年輕人，牌要看清楚。', '這張姐收了。'],
      kong: ['槓。開店三十年不是開假的。', '呵，槓。'],
      chow: ['吃。', '謝啦，小朋友。'],
      tsumo: ['自摸，三家都拿錢來～', '呵呵，自摸。'],
      ron: ['胡。這張姐等很久了。', '放槍囉～胡。'],
      deal_in: ['哎呀，姐也有失手的時候。', '算你厲害。'],
      draw: ['流局，喝杯茶再來。'],
      skill: ['老闆娘的眼力，可不是開玩笑的。', '牌牆裡有什麼，姐一看就知道。'],
    },
  },
  {
    id: 'cangya',
    images: characterImages('cangya'),
    name: '蒼牙',
    intro: '銀髮狼耳、鼻子靈又好勝的狼族少年',
    hair: '#a7a9ad',
    hairStyle: 'short',
    bangs: 'spiky',
    eyes: '#9b4a46',
    clothes: '#2c2c34',
    collar: '#c9a24a',
    accent: '#2f6fd1',
    lines: {
      pung: ['碰！這塊肉我叼走了。', '嘿，碰！手腳太慢囉。'],
      kong: ['槓！一口氣全吃下！', '四張到齊——嗷嗚，槓！'],
      chow: ['吃！聞到就知道是我的。', '這張味道不錯，吃。'],
      tsumo: ['自摸！獵物自己送上門啦！', '嘿嘿，自摸。我的鼻子從不出錯。'],
      ron: ['胡！早就聞到你要打這張了。', '抓到了！胡！'],
      deal_in: ['嗚……居然看漏了。', '可惡，被反咬一口！'],
      draw: ['流局？下一局一定咬住你。', '嘖，獵物跑了。'],
      skill: ['鎖定了。分數最高的那個，就是我的獵物。', '你的牌味，我全聞到了。'],
    },
  },
  {
    id: 'jinye',
    images: characterImages('jinye'),
    name: '金爺',
    intro: '鬍子帥氣、見過大風大浪的老牌賭客',
    hair: '#2a2a2e',
    hairStyle: 'short',
    bangs: 'parted',
    eyes: '#5a3a22',
    clothes: '#1b2233',
    collar: '#f0ece2',
    accent: '#c9a24a',
    lines: {
      pung: ['碰。年輕人，這張讓給長輩吧。', '呵，碰。'],
      kong: ['槓。賭桌上，要懂得乘勝追擊。', '四張到手，槓。'],
      chow: ['吃。別急，牌局還長得很。', '這張不錯，吃。'],
      tsumo: ['自摸。各位，付錢吧。', '呵呵，自摸。這把我早就看好了。'],
      ron: ['胡。這張牌，我等很久了。', '承讓了。胡。'],
      deal_in: ['呵，輸得起，才贏得起。', '這把算你的，下把再說。'],
      draw: ['流局。休息一下，喝杯茶。', '不急，好牌總會來的。'],
      skill: ['加注。這把，多算一台。', '賭桌上的老規矩——敢加注的人才贏得多。'],
    },
  },
  {
    id: 'leo',
    images: characterImages('leo'),
    name: '雷歐',
    intro: '戴著王冠、一本正經的異國王子騎士',
    hair: '#f2c94c',
    hairStyle: 'short',
    bangs: 'spiky',
    eyes: '#2b59c3',
    clothes: '#c7ccd6',
    collar: '#1f3f9a',
    accent: '#d4af37',
    lines: {
      pung: ['碰。這張牌，歸於王國。', '本王子收下了。碰。'],
      kong: ['槓。騎士的四重守護。', '四張齊聚，槓！'],
      chow: ['吃。順勢而為，亦是騎士之道。', '我接下了。吃。'],
      tsumo: ['自摸。勝利屬於堂堂正正之人。', '這是王者的勝利。自摸。'],
      ron: ['胡。這一張，便是你的敗因。', '承讓。胡。'],
      deal_in: ['……是我大意了。', '騎士也有失手的時候。下一局見真章。'],
      draw: ['流局。休整之後，再決勝負。', '無妨，戰局尚未結束。'],
      skill: ['王命已下——這張牌，誰也不准胡。', '以王冠之名，此牌不可侵犯。'],
    },
  },
  {
    id: 'alie',
    images: characterImages('alie'),
    name: '阿烈',
    intro: '紅髮刀疤、豪爽愛冒險的傭兵',
    hair: '#c1272d',
    hairStyle: 'short',
    bangs: 'spiky',
    eyes: '#c07a2c',
    clothes: '#1d1d1d',
    collar: '#7a1d1d',
    accent: '#d4af37',
    lines: {
      pung: ['碰！老子就愛這張！', '哈哈，碰！搶到手才算數！'],
      kong: ['槓！痛快！', '四張全收，槓啦！'],
      chow: ['吃！冒險就是要邊走邊撿！', '這張歸我了，吃！'],
      tsumo: ['自摸！寶藏到手啦！', '哈哈哈，自摸！今晚我請客！'],
      ron: ['胡！就等你這張！', '多謝啦，胡！'],
      deal_in: ['嘖，被將了一軍。', '這點小傷，不算什麼！'],
      draw: ['流局？下一場冒險再說！', '寶藏還沒挖到，繼續！'],
      skill: ['身經百戰，這點小錢算什麼！', '自摸？這次我不付！'],
    },
  },
  {
    id: 'ayi',
    hidden: true,
    images: characterImages('ayi'),
    name: '阿翼',
    intro: '棕髮藍眼、穿著銀色盔甲的隱藏來客',
    hair: '#795038',
    hairStyle: 'short',
    bangs: 'spiky',
    eyes: '#4baff2',
    clothes: '#cbd3dc',
    collar: '#20252f',
    accent: '#87bfff',
    lines: {
      pung: ['碰！這張我收下了。', '剛好，碰！'],
      kong: ['四張到齊，槓！', '槓！再來一張。'],
      chow: ['吃！接得正好。', '這張能接上，吃！'],
      tsumo: ['自摸！今晚手氣不錯！', '摸到了，謝謝大家！'],
      ron: ['胡！等的就是這張。', '這局拿下了！'],
      deal_in: ['哎呀，被看穿了。', '這張出得太急了。'],
      draw: ['再戰一局吧！', '先喝口茶，下一局見。'],
      skill: ['銀之盾！這一擊擋下了。', '盔甲可不是穿好看的。'],
    },
  },
].map((character) => ({
  ...character,
  imageStyles: Object.fromEntries(ART_STYLES.map(({ id }) => [id, characterImages(character.id, id)])),
}));

const BY_ID = Object.fromEntries(CHARACTERS.map((c) => [c.id, c]));

export function getSelectableCharacters(ayiUnlocked = false) {
  return CHARACTERS.filter((character) => !character.hidden || ayiUnlocked);
}

export function getCharacter(id) {
  return BY_ID[id] ?? CHARACTERS[0];
}

export function getCharacterImage(id, expression = 'normal', style = 'anime90') {
  const character = getCharacter(id);
  const images = character.imageStyles?.[style] ?? character.images;
  return images?.[expression] ?? images?.normal ?? character.images?.normal ?? null;
}

/** 每種事件對應的表情 */
export const EVENT_EXPRESSION = {
  pung: 'happy',
  kong: 'happy',
  chow: 'happy',
  tsumo: 'win',
  ron: 'win',
  deal_in: 'sad',
  draw: 'surprised',
  skill: 'happy',
};

// 技能模式的技能（要跟 crates/mahjong-core/src/skills.rs 一致）
export const SKILLS = {
  xiaocui: { name: '元氣再來！', desc: '摸到不想要的牌時，放回牌尾重摸一張。每局一次。' },
  yuehua: { name: '看穿', desc: '看穿其他三家聽什麼牌，維持到自己再打出三張牌。每局一次。' },
  aming: { name: '逆轉之魂', desc: '分數最低時，開局自動把配牌換好一點。' },
  tangtang: { name: '拜託嘛～', desc: '這一回合打出的牌，別人不能吃、碰、槓（胡牌擋不住）。每局一次。' },
  xiaochun: { name: '快攻', desc: '可以吃任何人打出的牌，不限上家。每局一次。' },
  lanjie: { name: '老闆娘的眼力', desc: '偷看牌牆接下來會被摸走的 3 張牌。每局一次。' },
  ayi: { name: '銀之盾', desc: '放槍時不用付錢。一場一次。' },
  cangya: { name: '鎖定獵物', desc: '看穿分數最高那家的整副手牌，維持到自己再打出三張牌。每局一次。' },
  jinye: { name: '加注', desc: '胡牌時多算 1 台。' },
  leo: { name: '王命', desc: '這一回合打出的牌，別人不能胡（吃、碰、槓擋不住）。每局一次。' },
  alie: { name: '身經百戰', desc: '別人自摸時不用付錢。一場一次。' },
};

export function pickLine(character, kind) {
  const list = getCharacter(character).lines[kind];
  if (!list?.length) return null;
  return list[Math.floor(Math.random() * list.length)];
}

// 角色設定。id 要跟 src-tauri/src/game/room.rs 的 CHARACTERS 一致。
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

// 90 年代賽璐璐動畫風格，五種表情均使用獨立透明 PNG。
function characterImages(id, style = 'anime90') {
  const directory = `/characters/${id}${style === 'anime90' ? '' : `/${style}`}`;
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
    },
  },
].map((character) => ({
  ...character,
  imageStyles: {
    anime90: character.images,
    pc98: characterImages(character.id, 'pc98'),
    msdos: characterImages(character.id, 'msdos'),
  },
}));

const BY_ID = Object.fromEntries(CHARACTERS.map((c) => [c.id, c]));

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
};

export function pickLine(character, kind) {
  const list = getCharacter(character).lines[kind];
  if (!list?.length) return null;
  return list[Math.floor(Math.random() * list.length)];
}

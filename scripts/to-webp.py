"""把 public/characters 與 public/ui 底下的 PNG 轉成 WebP（轉完刪掉 PNG）。

  python scripts/to-webp.py            # 轉換全部
  python scripts/to-webp.py path.png   # 只轉指定的檔案（之後新增的圖也用這個）

規則：
- 像素畫風（pc98、msdos、gameboy）的角色圖、牌桌、標誌：無損（像素邊緣不能糊）
- 大張背景（parlor、title-scene）：有損 quality 90（無損還是好幾 MB，畫面上看不出差別）
- 其他：有損 quality 85
圖示（icon-*.png、apple-touch-icon.png）與分享預覽圖（og-image.jpg）不轉，平台要求那些格式。
"""
import os
import sys
from PIL import Image

ROOT = os.path.join(os.path.dirname(__file__), '..', 'public')
PIXEL_STYLES = {'pc98', 'msdos', 'gameboy'}
BACKGROUNDS = {'parlor.png', 'title-scene.png'}


def style_of(path):
    parts = os.path.normpath(path).split(os.sep)
    if 'ui' in parts:
        return parts[parts.index('ui') + 1]
    if 'characters' in parts:
        rest = parts[parts.index('characters') + 1:]
        return rest[1] if len(rest) == 3 else 'anime90'
    return None


def convert(path):
    name = os.path.basename(path)
    out = path[:-4] + '.webp'
    im = Image.open(path)
    if name in BACKGROUNDS:
        im.save(out, 'WEBP', quality=90, method=6)
    elif style_of(path) in PIXEL_STYLES:
        im.save(out, 'WEBP', lossless=True, method=6)
    else:
        im.save(out, 'WEBP', quality=85, method=6)
    before, after = os.path.getsize(path), os.path.getsize(out)
    os.remove(path)
    return before, after


def main():
    files = sys.argv[1:]
    if not files:
        for top in ('characters', 'ui'):
            for dirpath, _, names in os.walk(os.path.join(ROOT, top)):
                files += [os.path.join(dirpath, n) for n in names if n.endswith('.png')]
    total_before = total_after = 0
    for f in sorted(files):
        b, a = convert(f)
        total_before += b
        total_after += a
    print(f'{len(files)} 張：{total_before / 1048576:.1f} MB → {total_after / 1048576:.1f} MB')


if __name__ == '__main__':
    main()

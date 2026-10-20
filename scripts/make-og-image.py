"""產生社群分享預覽圖（public/og-image.jpg，1200×630）與網頁圖示（apple-touch-icon、icon-192/512）。

用法：python scripts/make-og-image.py [專案根目錄，預設 .]
需要 Pillow；字型用 Windows 內建的微軟正黑體。
"""
import sys
from PIL import Image, ImageDraw, ImageFont, ImageFilter

root = sys.argv[1] if len(sys.argv) > 1 else "."
W, H = 1200, 630

# ── 預覽圖：標題畫面＋左側壓暗＋書法標誌＋一行介紹 ──
scene = Image.open(f'{root}/public/ui/anime90/title-scene.png').convert('RGB')
scale = W / scene.width
scene = scene.resize((W, round(scene.height * scale)), Image.LANCZOS)
top = (scene.height - H) // 2
og = scene.crop((0, top, W, top + H)).convert('RGBA')

# 左側漸層壓暗，讓標誌與字看得清楚
shade = Image.new('RGBA', (W, H))
d = ImageDraw.Draw(shade)
for x in range(W):
    a = max(0, int(205 * (1 - x / (W * 0.62))))
    d.line([(x, 0), (x, H)], fill=(4, 22, 17, a))
og = Image.alpha_composite(og, shade)

logo = Image.open(f'{root}/public/ui/anime90/logo.png').convert('RGBA')
lw = 470
logo = logo.resize((lw, round(logo.height * lw / logo.width)), Image.LANCZOS)
og.alpha_composite(logo, (30, 178))

font_path = 'C:/Windows/Fonts/msjhbd.ttc'
font = ImageFont.truetype(font_path, 32)
small = ImageFont.truetype(font_path, 24)
d = ImageDraw.Draw(og)


def text(xy, s, f, fill):
    x, y = xy
    d.text((x + 2, y + 2), s, font=f, fill=(0, 0, 0, 200))
    d.text((x, y), s, font=f, fill=fill)


text((56, 362), '線上的台灣十六張麻將', font, (255, 236, 196, 255))
text((56, 410), '人類 × AI × 虛擬角色，同桌開打', font, (255, 236, 196, 255))
text((56, 470), '單機遊玩・區網連線・Claude / ChatGPT 上桌', small, (214, 230, 214, 255))

og.convert('RGB').save(f'{root}/public/og-image.jpg', quality=88, optimize=True, progressive=True)
print('og-image.jpg', og.size)

# ── 圖示 ──
icon = Image.open(f'{root}/src-tauri/icons/icon.png').convert('RGBA')
print('source icon', icon.size)
icon.resize((180, 180), Image.LANCZOS).save(f'{root}/public/apple-touch-icon.png', optimize=True)
icon.resize((192, 192), Image.LANCZOS).save(f'{root}/public/icon-192.png', optimize=True)
icon.resize((512, 512), Image.LANCZOS).save(f'{root}/public/icon-512.png', optimize=True)
print('icons done')

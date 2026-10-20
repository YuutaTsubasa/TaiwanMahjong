"""Only crop, resize and pad ImageGen output; preserve generated RGBA."""
from pathlib import Path
from PIL import Image
import hashlib
import json

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
EXPRESSIONS = ["normal", "happy", "surprised", "sad", "win"]
STYLES = {
    "anime90": ("anime90-initial.png", ROOT / "public/characters/ayi"),
    "pc98": ("pc98.png", ROOT / "public/characters/ayi/pc98"),
    "msdos": ("msdos.png", ROOT / "public/characters/ayi/msdos"),
}
report = []
preview = Image.new("RGB", (5 * 256, 3 * 256), "#e5e2da")
for row, (style, (sheet, destination)) in enumerate(STYLES.items()):
    source = Image.open(HERE / "sheets" / sheet).convert("RGBA")
    assert source.size == (1536, 1024), source.size
    destination.mkdir(parents=True, exist_ok=True)
    for index, expression in enumerate(EXPRESSIONS):
        x = index % 3 * 512
        top = index // 3 * 512
        bottom = top + 512
        # Anime's top-row breastplate ends at y=514. The next portraits
        # start below y=520, so use that transparent gutter for clean cuts.
        if style == "anime90":
            top, bottom = (0, 514) if index < 3 else (520, 1024)
        box = (x, top, x + 512, bottom)
        crop = source.crop(box)
        scale = min(480 / crop.width, 480 / crop.height)
        size = (round(crop.width * scale), round(crop.height * scale))
        resampling = Image.Resampling.LANCZOS if style == "anime90" else Image.Resampling.NEAREST
        crop = crop.resize(size, resampling)
        final = Image.new("RGBA", (512, 512))
        offset = ((512 - crop.width) // 2, (512 - crop.height) // 2)
        final.paste(crop, offset)
        path = destination / f"{expression}.png"
        final.save(path)
        alpha = final.getchannel("A")
        histogram = alpha.histogram()
        assert final.size == (512, 512)
        assert histogram[0] > 512 * 512 * .25
        assert sum(histogram[129:]) > 512 * 512 * .25
        assert alpha.getpixel((0, 0)) == 0
        bbox = alpha.getbbox()
        assert bbox[0] >= 16 and bbox[1] >= 16 and bbox[2] <= 496 and bbox[3] <= 496
        report.append({
            "style": style, "expression": expression,
            "file": str(path.relative_to(ROOT)).replace("\\", "/"),
            "size": list(final.size), "mode": final.mode,
            "source_crop": box, "padding_offset": offset,
            "nonzero_alpha_bbox": bbox,
            "fully_transparent_pixels": histogram[0],
            "alpha_max": alpha.getextrema()[1],
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        })
        thumb = final.resize((256, 256), resampling)
        preview.paste(thumb, (index * 256, row * 256), thumb)
preview.save(HERE / "contact-sheet.png")
(HERE / "asset-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"count": len(report), "passed": True, "preview": str(HERE / "contact-sheet.png")}))

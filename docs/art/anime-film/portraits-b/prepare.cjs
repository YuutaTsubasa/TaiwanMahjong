const fs = require('node:fs/promises');
const path = require('node:path');
const crypto = require('node:crypto');
const sharp = require('C:/Users/User/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
const root = path.resolve(__dirname, '../../../..');
const expressions = ['normal', 'happy', 'surprised', 'sad', 'win'];
async function bbox(input, threshold = 0) {
  const { data, info } = await sharp(input).ensureAlpha().raw().toBuffer({ resolveWithObject: true });
  let left = info.width, top = info.height, right = -1, bottom = -1;
  for (let y = 0; y < info.height; y++) for (let x = 0; x < info.width; x++) {
    if (data[(y * info.width + x) * 4 + 3] > threshold) {
      left = Math.min(left, x); top = Math.min(top, y);
      right = Math.max(right, x); bottom = Math.max(bottom, y);
    }
  }
  return { left, top, right, bottom };
}
async function main() {
  const manifest = JSON.parse(await fs.readFile(path.join(__dirname, 'prompts.json'), 'utf8'));
  const checks = [], tiles = [];
  await fs.mkdir(path.join(__dirname, 'sheets'), { recursive: true });
  for (let row = 0; row < manifest.assets.length; row++) {
    const asset = manifest.assets[row];
    const local = path.join(__dirname, 'sheets', `${asset.id}.png`);
    await fs.copyFile(asset.correction.source, local);
    const meta = await sharp(local).metadata();
    const destination = path.join(root, 'public/characters', asset.id, 'modern-paint');
    await fs.mkdir(destination, { recursive: true });
    for (let i = 0; i < 5; i++) {
      const left = Math.round(i * meta.width / 5);
      const width = Math.round((i + 1) * meta.width / 5) - left;
      const cell = await sharp(local).extract({ left, top: 0, width, height: meta.height }).png().toBuffer();
      const b = await bbox(cell, 1);
      const crop = {
        left: Math.max(0, b.left - 2), top: Math.max(0, b.top - 2),
        right: Math.min(width - 1, b.right + 2), bottom: Math.min(meta.height - 1, b.bottom + 2),
      };
      const portrait = await sharp(cell).extract({
        left: crop.left, top: crop.top, width: crop.right - crop.left + 1, height: crop.bottom - crop.top + 1,
      }).resize(480, 480, { fit: 'inside' }).png().toBuffer();
      const size = await sharp(portrait).metadata();
      const output = path.join(destination, `${expressions[i]}.png`);
      await sharp({ create: { width: 512, height: 512, channels: 4, background: { r: 0, g: 0, b: 0, alpha: 0 } } })
        .composite([{ input: portrait, left: Math.floor((512 - size.width) / 2), top: Math.floor((512 - size.height) / 2) }])
        .png().toFile(output);
      const file = await fs.readFile(output);
      const actual = await sharp(file).metadata();
      const bounds = await bbox(file);
      if (actual.width !== 512 || actual.height !== 512 || !actual.hasAlpha) throw Error(output);
      if (bounds.left < 16 || bounds.top < 16 || bounds.right > 495 || bounds.bottom > 495) throw Error(`Margin: ${output}`);
      checks.push({ file: path.relative(root, output).replaceAll('\\', '/'), width: actual.width, height: actual.height, hasAlpha: actual.hasAlpha, bounds, sha256: crypto.createHash('sha256').update(file).digest('hex') });
      tiles.push({ input: await sharp(file).resize(240, 240).png().toBuffer(), left: i * 240, top: row * 240 });
    }
  }
  if (new Set(checks.map(x => x.sha256)).size !== 15) throw Error('Duplicate images');
  await fs.writeFile(path.join(__dirname, 'verification.json'), JSON.stringify(checks, null, 2) + '\n');
  await sharp({ create: { width: 1200, height: 720, channels: 4, background: '#b7c1cf' } }).composite(tiles).png().toFile(path.join(__dirname, 'contact-sheet.png'));
  console.log('Verified 15 unique 512x512 RGBA portraits with minimum16px transparent margins.');
}
main().catch(error => { console.error(error); process.exitCode = 1; });

const fs = require('node:fs/promises');
const path = require('node:path');
const sharp = require('C:/Users/User/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
const root = path.resolve(__dirname, '../../..');
const expressions = ['normal', 'happy', 'surprised', 'sad', 'win'];
const transparent = { r: 0, g: 0, b: 0, alpha: 0 };

async function bounds(buffer, threshold = 0) {
  const { data, info } = await sharp(buffer).ensureAlpha().raw().toBuffer({ resolveWithObject: true });
  let left = info.width, top = info.height, right = -1, bottom = -1;
  let count = 0;
  for (let y = 0; y < info.height; y++) for (let x = 0; x < info.width; x++) {
    if (data[(y * info.width + x) * 4 + 3] > threshold) {
      left = Math.min(left, x); right = Math.max(right, x);
      top = Math.min(top, y); bottom = Math.max(bottom, y); count++;
    }
  }
  return { left, top, right, bottom, width: right - left + 1, height: bottom - top + 1, count };
}

async function main() {
  const manifest = JSON.parse(await fs.readFile(path.join(__dirname, 'prompts.json'), 'utf8'));
  await fs.mkdir(path.join(__dirname, 'sheets'), { recursive: true });
  const verification = [];
  for (const item of manifest.portraits) {
    const sheetPath = path.join(__dirname, 'sheets', `${item.id}.png`);
    await fs.copyFile(item.correction.source, sheetPath);
    const sheet = await sharp(sheetPath).metadata();
    const destination = path.join(root, 'public/characters', item.id, 'taiwan-print');
    await fs.mkdir(destination, { recursive: true });
    for (let index = 0; index < expressions.length; index++) {
      const output = path.join(destination, `${expressions[index]}.png`);
      if (item.id === 'ayi' && index === 0) {
        await fs.copyFile(path.join(root, 'docs/art/style-studies/2026-10-21/taiwan-print/character.png'), output);
      } else {
        const left = Math.round(index * sheet.width / 5);
        const right = Math.round((index + 1) * sheet.width / 5);
        const cell = await sharp(sheetPath).extract({ left, top: 0, width: right - left, height: sheet.height }).png().toBuffer();
        const visible = await bounds(cell, 1);
        const box = {
          left: Math.max(0, visible.left - 2),
          top: Math.max(0, visible.top - 2),
          right: Math.min(right - left - 1, visible.right + 2),
          bottom: Math.min(sheet.height - 1, visible.bottom + 2),
        };
        box.width = box.right - box.left + 1;
        box.height = box.bottom - box.top + 1;
        const portrait = await sharp(cell).extract({ left: box.left, top: box.top, width: box.width, height: box.height })
          .resize(480, 480, { fit: 'inside' }).png().toBuffer();
        const p = await sharp(portrait).metadata();
        await sharp({ create: { width: 512, height: 512, channels: 4, background: transparent } })
          .composite([{ input: portrait, left: Math.floor((512 - p.width) / 2), top: Math.floor((512 - p.height) / 2) }])
          .png().toFile(output);
      }
      const content = await fs.readFile(output);
      const meta = await sharp(content).metadata();
      const box = await bounds(content);
      if (meta.width !== 512 || meta.height !== 512 || !meta.hasAlpha) throw new Error(`Invalid format: ${output}`);
      if (box.left < 16 || box.top < 16 || box.right > 495 || box.bottom > 495) throw new Error(`Invalid margin: ${output}: ${JSON.stringify(box)}`);
      verification.push({ file: path.relative(root, output).replaceAll('\\', '/'), width: meta.width, height: meta.height, hasAlpha: meta.hasAlpha, bounds: box });
    }
  }
  for (const item of manifest.backgrounds) {
    const output = path.join(root, 'public/ui/taiwan-print', `${item.id}.png`);
    await sharp(item.source).resize(1920, 1080, { fit: 'fill' }).png().toFile(output);
    const meta = await sharp(output).metadata();
    verification.push({ file: path.relative(root, output).replaceAll('\\', '/'), width: meta.width, height: meta.height, hasAlpha: meta.hasAlpha });
  }
  for (const id of ['logo', 'table']) {
    const source = path.join(root, 'docs/art/style-studies/2026-10-21/taiwan-print', `${id}.png`);
    const output = path.join(root, 'public/ui/taiwan-print', `${id}.png`);
    await fs.copyFile(source, output);
    const meta = await sharp(output).metadata();
    verification.push({ file: path.relative(root, output).replaceAll('\\', '/'), width: meta.width, height: meta.height, hasAlpha: meta.hasAlpha, exactApprovedCopy: true });
  }
  await fs.writeFile(path.join(__dirname, 'verification.json'), JSON.stringify(verification, null, 2) + '\n');
  const thumbs = [];
  for (let row = 0; row < manifest.portraits.length; row++) {
    for (let col = 0; col < 5; col++) {
      const input = path.join(root, 'public/characters', manifest.portraits[row].id, 'taiwan-print', `${expressions[col]}.png`);
      thumbs.push({ input: await sharp(input).resize(180, 180).toBuffer(), left: col * 180, top: row * 180 });
    }
  }
  await sharp({ create: { width: 900, height: 1260, channels: 4, background: '#c4b89c' } }).composite(thumbs).png().toFile(path.join(__dirname, 'contact-sheet.png'));
  console.log(`Prepared and verified ${verification.length} production assets.`);
}
main().catch(error => { console.error(error); process.exitCode = 1; });

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const sharp = require('C:/Users/User/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
const root = path.resolve(__dirname, '../../..');
const manifest = JSON.parse(fs.readFileSync(path.join(__dirname, 'prompts.json'), 'utf8'));
const titleCorrection = JSON.parse(fs.readFileSync(path.join(__dirname, 'title-correction.json'), 'utf8'));
const titleEntry = manifest.entries.find(entry => entry.id === 'title');
titleEntry.initialGeneratedSource = titleEntry.initialGeneratedSource || titleEntry.generatedSource;
titleEntry.correction = titleCorrection;
titleEntry.generatedSource = titleCorrection.generatedSource;
const expressions = ['normal', 'happy', 'surprised', 'sad', 'win'];
const sample = path.join(root, 'docs/art/style-studies/2026-10-21/modern-paint');
const checks = [];
const hash = (b) => crypto.createHash('sha256').update(b).digest('hex');
async function inspect(file) {
  const {data, info} = await sharp(file).ensureAlpha().raw().toBuffer({resolveWithObject:true});
  let minX=info.width, minY=info.height, maxX=-1, maxY=-1, nonzero=0, edge=0;
  for(let y=0;y<info.height;y++) for(let x=0;x<info.width;x++) {
    if(data[(y*info.width+x)*4+3]) {
      minX=Math.min(minX,x);minY=Math.min(minY,y);maxX=Math.max(maxX,x);maxY=Math.max(maxY,y);nonzero++;
      if(x<16||y<16||x>=info.width-16||y>=info.height-16) edge++;
    }
  }
  return {file:path.relative(root,file).replaceAll('\\','/'),width:info.width,height:info.height,channels:info.channels,bounds:[minX,minY,maxX,maxY],transparentPixels:info.width*info.height-nonzero,outer16Nontransparent:edge,sha256:hash(fs.readFileSync(file))};
}
async function run() {
  fs.mkdirSync(path.join(__dirname,'sources'),{recursive:true});
  fs.copyFileSync(titleEntry.initialGeneratedSource,path.join(__dirname,'sources','title-initial.png'));
  for(const entry of manifest.entries) {
    const isPortrait = !['title','parlor'].includes(entry.id);
    const source = path.join(__dirname,'sources',entry.id+(isPortrait?'-expressions':'')+'.png');
    fs.copyFileSync(entry.generatedSource,source);
    entry.archivedSource=path.relative(root,source).replaceAll('\\','/');
    entry.references = isPortrait ? [
      `public/characters/${entry.id}/normal.png`,
      'docs/art/style-studies/2026-10-21/modern-paint/character.png'
    ] : [
      `public/ui/anime90/${entry.id==='title'?'title-scene':'parlor'}.png`,
      `docs/art/style-studies/2026-10-21/modern-paint/${entry.id==='title'?'character':'table'}.png`
    ];
    if(!isPortrait) {
      const output=path.join(root,'public/ui/modern-paint',entry.id==='title'?'title-scene.png':'parlor.png');
      fs.mkdirSync(path.dirname(output),{recursive:true});
      await sharp(source).resize(1920,1080,{fit:'fill'}).png().toFile(output);
      checks.push(await inspect(output));
      continue;
    }
    const {data,info}=await sharp(source).ensureAlpha().raw().toBuffer({resolveWithObject:true});
    const counts=Array(info.width).fill(0);
    for(let x=0;x<info.width;x++) for(let y=0;y<info.height;y++) if(data[(y*info.width+x)*4+3]>32) counts[x]++;
    const boundaries=[0];
    for(let k=1;k<5;k++) {
      const center=Math.round(info.width*k/5), radius=Math.round(info.width/5*.07);
      let best=center;
      for(let x=center-radius;x<=center+radius;x++) if(counts[x]<counts[best] || (counts[x]===counts[best]&&Math.abs(x-center)<Math.abs(best-center))) best=x;
      boundaries.push(best);
    }
    boundaries.push(info.width);
    entry.cropBoundaries=boundaries;
    entry.outputs=[];
    for(let i=0;i<5;i++) {
      const output=path.join(root,'public/characters',entry.id,'modern-paint',expressions[i]+'.png');
      fs.mkdirSync(path.dirname(output),{recursive:true});
      if(entry.id==='ayi'&&i===0) {
        fs.copyFileSync(path.join(sample,'character.png'),output);
      } else {
        const cell=await sharp(source).extract({left:boundaries[i],top:0,width:boundaries[i+1]-boundaries[i],height:info.height}).png().toBuffer();
        await sharp(cell).trim({threshold:1}).resize(480,480,{fit:'contain',background:{r:0,g:0,b:0,alpha:0}})
          .extend({top:16,bottom:16,left:16,right:16,background:{r:0,g:0,b:0,alpha:0}}).png().toFile(output);
      }
      entry.outputs.push(path.relative(root,output).replaceAll('\\','/'));
      checks.push(await inspect(output));
    }
  }
  for(const asset of ['logo','table']) {
    const output=path.join(root,'public/ui/modern-paint',asset+'.png');
    fs.copyFileSync(path.join(sample,asset+'.png'),output);
    checks.push(await inspect(output));
  }
  manifest.approvedCopies = ['character','logo','table'].map(asset=>({
    source:`docs/art/style-studies/2026-10-21/modern-paint/${asset}.png`,
    destination:asset==='character'?'public/characters/ayi/modern-paint/normal.png':`public/ui/modern-paint/${asset}.png`
  }));
  manifest.processing='Portraits split at minimum-alpha columns near five equal cell boundaries, alpha-trimmed, proportionally fit into 480x480 and padded 16 px to 512x512. Ayi normal, logo and table copied byte-identically from approved studies. Scene resize only to 1920x1080.';
  fs.writeFileSync(path.join(__dirname,'prompts.json'),JSON.stringify(manifest,null,2)+'\n');
  fs.writeFileSync(path.join(__dirname,'verification.json'),JSON.stringify(checks,null,2)+'\n');
  const contact=[];
  for(let row=0;row<7;row++) for(let col=0;col<5;col++) {
    const entry=manifest.entries[row];
    const png=await sharp(path.join(root,entry.outputs[col])).resize(256,256).png().toBuffer();
    contact.push({input:png,left:col*256,top:row*256});
  }
  await sharp({create:{width:1280,height:1792,channels:4,background:'#25333e'}}).composite(contact).png().toFile(path.join(__dirname,'contact-sheet.png'));
  console.log(JSON.stringify({total:checks.length,portraits:checks.filter(x=>x.width===512).length,marginFailures:checks.filter(x=>x.width===512&&x.outer16Nontransparent),duplicateHashes:checks.length-new Set(checks.map(x=>x.sha256)).size},null,2));
}
run().catch(e=>{console.error(e);process.exit(1)});

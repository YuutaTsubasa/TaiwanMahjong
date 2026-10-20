const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const sharp = require('C:/Users/User/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
const root = path.resolve(__dirname, '../../../..');
const manifest = JSON.parse(fs.readFileSync(path.join(__dirname, 'prompts.json'), 'utf8'));
const expressions = ['normal', 'happy', 'surprised', 'sad', 'win'];
const checks = [];
async function run() {
  fs.mkdirSync(path.join(__dirname,'sources'),{recursive:true});
  const contact=[];
  for(const [row,entry] of manifest.entries.entries()) {
    const source=path.join(__dirname,'sources',entry.id+'-expressions.png');
    fs.copyFileSync(entry.generatedSource,source);
    entry.archivedSource=path.relative(root,source).replaceAll('\\','/');
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
    for(let col=0;col<5;col++) {
      const output=path.join(root,'public/characters',entry.id,'modern-paint',expressions[col]+'.png');
      fs.mkdirSync(path.dirname(output),{recursive:true});
      const cell=await sharp(source).extract({left:boundaries[col],top:0,width:boundaries[col+1]-boundaries[col],height:info.height}).png().toBuffer();
      await sharp(cell).trim({threshold:1}).resize(480,480,{fit:'contain',background:{r:0,g:0,b:0,alpha:0}})
        .extend({top:16,bottom:16,left:16,right:16,background:{r:0,g:0,b:0,alpha:0}}).png().toFile(output);
      entry.outputs.push(path.relative(root,output).replaceAll('\\','/'));
      const {data: pixels,info:meta}=await sharp(output).ensureAlpha().raw().toBuffer({resolveWithObject:true});
      let edge=0,transparent=0,minX=512,minY=512,maxX=-1,maxY=-1;
      for(let y=0;y<512;y++) for(let x=0;x<512;x++) {
        if(pixels[(y*512+x)*4+3]===0) transparent++;
        else {
          minX=Math.min(minX,x);minY=Math.min(minY,y);maxX=Math.max(maxX,x);maxY=Math.max(maxY,y);
          if(x<16||y<16||x>=496||y>=496) edge++;
        }
      }
      checks.push({file:entry.outputs[col],width:meta.width,height:meta.height,channels:meta.channels,
        bounds:[minX,minY,maxX,maxY],outer16Nontransparent:edge,transparentPixels:transparent,
        sha256:crypto.createHash('sha256').update(fs.readFileSync(output)).digest('hex')});
      contact.push({input:await sharp(output).resize(256,256).png().toBuffer(),left:col*256,top:row*256});
    }
  }
  manifest.processing='Five distinct generated portrait cells split at low-alpha valleys near equal fifths; transparent trim; fit proportionally to 480x480; add 16 px transparent margin on each edge. No painting, recoloring, or facial edits by code.';
  fs.writeFileSync(path.join(__dirname,'prompts.json'),JSON.stringify(manifest,null,2)+'\n');
  fs.writeFileSync(path.join(__dirname,'verification.json'),JSON.stringify(checks,null,2)+'\n');
  await sharp({create:{width:1280,height:1024,channels:4,background:'#25333e'}}).composite(contact).png().toFile(path.join(__dirname,'contact-sheet.png'));
  console.log(JSON.stringify({portraits:checks.length,marginFailures:checks.filter(x=>x.outer16Nontransparent),duplicateHashes:checks.length-new Set(checks.map(x=>x.sha256)).size},null,2));
}
run().catch(e=>{console.error(e);process.exit(1)});

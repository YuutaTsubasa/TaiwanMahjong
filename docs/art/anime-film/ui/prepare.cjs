const fs=require('fs');
const path=require('path');
const crypto=require('crypto');
const sharp=require('C:/Users/User/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
const root=path.resolve(__dirname,'../../../..');
const output=path.join(root,'public/ui/modern-paint');
const transparent={r:0,g:0,b:0,alpha:0};
async function normalizeAlpha(input){
 const {data,info}=await sharp(input).ensureAlpha().raw().toBuffer({resolveWithObject:true});
 for(let i=3;i<data.length;i+=4){
  if(data[i]<=8)data.fill(0,i-3,i+1);
  else if(data[i]>=240)data[i]=255;
 }
 return sharp(data,{raw:info}).png().toBuffer();
}
async function bounds(input,threshold=0){
 const {data,info}=await sharp(input).ensureAlpha().raw().toBuffer({resolveWithObject:true});
 let left=info.width,top=info.height,right=-1,bottom=-1,count=0,partial=0;
 for(let y=0;y<info.height;y++)for(let x=0;x<info.width;x++){
  const a=data[(y*info.width+x)*4+3];
  if(a>threshold){left=Math.min(left,x);right=Math.max(right,x);top=Math.min(top,y);bottom=Math.max(bottom,y);count++;}
  if(a&&a<255)partial++;
 }
 return {left,top,right,bottom,width:right-left+1,height:bottom-top+1,opaqueOrPartialPixels:count,partialAlphaPixels:partial};
}
async function main(){
 const manifest=JSON.parse(fs.readFileSync(path.join(__dirname,'prompts.json'),'utf8'));
 fs.mkdirSync(path.join(__dirname,'sources'),{recursive:true});
 const checks=[];
 for(const j of manifest.assets){
  const source=path.join(__dirname,'sources',j.id+'.png');
  fs.copyFileSync(j.source,source);
  const destination=path.join(output,j.id+'.png');
  if(j.id==='title-scene'||j.id==='parlor'){
   await sharp(source).resize(1920,1080,{fit:'cover'}).flatten({background:'#213650'}).png().toFile(destination);
  }else{
   // The corrected generated table's physical outer wooden rectangle was
   // visually inspected on a solid blue backdrop. Crop to that rectangle,
   // not almost-transparent diffusion pixels elsewhere in the source.
   const b=j.id==='table'?{left:266,top:45,width:982,height:944}:await bounds(source,8);
   const cropped=sharp(source).extract({left:b.left,top:b.top,width:b.width,height:b.height});
   if(j.id==='table'){
    const art=await normalizeAlpha(await cropped.resize(1000,1000,{fit:'fill'}).png().toBuffer());
    await sharp({create:{width:1600,height:1100,channels:4,background:transparent}}).composite([{input:art,left:300,top:50}]).png().toFile(destination);
   }else{
    const art=await normalizeAlpha(await cropped.resize(1352,372,{fit:'inside'}).png().toBuffer());
    const size=await sharp(art).metadata();
    await sharp({create:{width:1400,height:420,channels:4,background:transparent}}).composite([{input:art,left:Math.floor((1400-size.width)/2),top:Math.floor((420-size.height)/2)}]).png().toFile(destination);
   }
  }
  const meta=await sharp(destination).metadata();
  checks.push({file:path.relative(root,destination).replaceAll('\\','/'),width:meta.width,height:meta.height,hasAlpha:meta.hasAlpha,bounds:await bounds(destination),sha256:crypto.createHash('sha256').update(fs.readFileSync(destination)).digest('hex')});
 }
 fs.writeFileSync(path.join(__dirname,'verification.json'),JSON.stringify(checks,null,2)+'\n');
 const panels=[];
 for(const [id,left,top,width,height] of [['title-scene',0,0,960,540],['parlor',960,0,960,540],['logo',0,640,960,288],['table',960,540,960,660]]){
  panels.push({input:await sharp(path.join(output,id+'.png')).resize(width,height,{fit:'contain',background:transparent}).png().toBuffer(),left,top});
 }
 await sharp({create:{width:1920,height:1200,channels:4,background:'#10253b'}}).composite(panels).png().toFile(path.join(__dirname,'contact-sheet.png'));
 console.log(JSON.stringify(checks,null,2));
}
main().catch(e=>{console.error(e);process.exitCode=1;});

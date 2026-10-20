const fs=require('fs');
const path=require('path');
const sharp=require('C:/Users/User/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp');
const root=path.resolve(__dirname,'../../..');
const palette=[[15,56,15],[48,98,48],[139,172,15],[155,188,15]];
const expressions=['normal','happy','surprised','sad','win'];
async function quantize(input,out,opaque=false){
 const {data,info}=await sharp(input).ensureAlpha().raw().toBuffer({resolveWithObject:true});
 for(let i=0;i<data.length;i+=4){
  if(!opaque&&data[i+3]<128){data.fill(0,i,i+4);continue;}
  let best=palette[0],distance=Infinity;
  for(const c of palette){const d=(data[i]-c[0])**2+(data[i+1]-c[1])**2+(data[i+2]-c[2])**2;if(d<distance){best=c;distance=d;}}
  data[i]=best[0];data[i+1]=best[1];data[i+2]=best[2];data[i+3]=255;
 }
 await sharp(data,{raw:info}).png().toFile(out);
}
async function sheet(id,source){
 const folder=path.join(root,'public/characters',id,'gameboy');fs.mkdirSync(folder,{recursive:true});
 const meta=await sharp(source).metadata();
 const sheetPixels=await sharp(source).ensureAlpha().raw().toBuffer();
 const occupancy=Array(meta.width).fill(0);
 for(let y=0;y<meta.height;y++)for(let x=0;x<meta.width;x++)if(sheetPixels[(y*meta.width+x)*4+3]>=128)occupancy[x]++;
 const cuts=Array.from({length:6},(_,i)=>{
  if(i===0)return 0;if(i===5)return meta.width;
  const ideal=meta.width*i/5, radius=Math.round(meta.width/5*.12);
  let best=Math.round(ideal),score=Infinity;
  for(let x=Math.round(ideal)-radius;x<=Math.round(ideal)+radius;x++){
   const candidate=occupancy[x]*1000+Math.abs(x-ideal);
   if(candidate<score){best=x;score=candidate;}
  }
  return best;
 });
 for(let i=0;i<5;i++){
  const cell=await sharp(source).extract({left:cuts[i],top:0,width:cuts[i+1]-cuts[i],height:meta.height}).ensureAlpha().raw().toBuffer({resolveWithObject:true});
  // Cropping to the main connected silhouette excludes adjacent-cell slivers
  // when a ponytail extends across an otherwise transparent sheet gutter.
  let minX=cell.info.width,minY=cell.info.height,maxX=-1,maxY=-1,largest=0,mainPixels=null;
  const seen=new Uint8Array(cell.info.width*cell.info.height),queue=new Int32Array(seen.length);
  for(let p=0;p<seen.length;p++){
   if(seen[p]||cell.data[p*4+3]<128)continue;
   let head=0,tail=1,lx=cell.info.width,ly=cell.info.height,hx=-1,hy=-1;queue[0]=p;seen[p]=1;
   while(head<tail){
    const q=queue[head++],x=q%cell.info.width,y=Math.floor(q/cell.info.width);
    lx=Math.min(lx,x);hx=Math.max(hx,x);ly=Math.min(ly,y);hy=Math.max(hy,y);
    for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){
     const nx=x+dx,ny=y+dy;if(nx<0||ny<0||nx>=cell.info.width||ny>=cell.info.height)continue;
     const n=ny*cell.info.width+nx;if(!seen[n]&&cell.data[n*4+3]>=128){seen[n]=1;queue[tail++]=n;}
    }
   }
   if(tail>largest){largest=tail;minX=lx;minY=ly;maxX=hx;maxY=hy;mainPixels=queue.slice(0,tail);}
  }
  const keep=new Uint8Array(seen.length);
  for(const p of mainPixels)keep[p]=1;
  for(let p=0;p<seen.length;p++)if(!keep[p])cell.data.fill(0,p*4,p*4+4);
  if(maxX<0)throw Error('Empty sprite '+id+'/'+i);
  const cropped=await sharp(cell.data,{raw:cell.info}).extract({left:minX,top:minY,width:maxX-minX+1,height:maxY-minY+1}).resize(480,480,{fit:'inside',kernel:'nearest'}).png().toBuffer();
  const cm=await sharp(cropped).metadata();
  const canvas=await sharp({create:{width:512,height:512,channels:4,background:{r:0,g:0,b:0,alpha:0}}}).composite([{input:cropped,left:Math.floor((512-cm.width)/2),top:496-cm.height}]).png().toBuffer();
  await quantize(canvas,path.join(folder,expressions[i]+'.png'));
 }
}
async function background(id,source){
 const folder=path.join(root,'public/ui/gameboy');fs.mkdirSync(folder,{recursive:true});
 const img=await sharp(source).resize(1672,940,{fit:'cover',kernel:'nearest'}).png().toBuffer();
 await quantize(img,path.join(folder,id+'.png'),true);
}
async function approved(){
 const src=path.join(root,'docs/art/style-studies/2026-10-21/gameboy');
 const dest=path.join(root,'public/ui/gameboy');fs.mkdirSync(dest,{recursive:true});
 for(const id of ['logo','table'])fs.copyFileSync(path.join(src,id+'.png'),path.join(dest,id+'.png'));
 const char=path.join(root,'public/characters/ayi/gameboy');fs.mkdirSync(char,{recursive:true});
 fs.copyFileSync(path.join(src,'character.png'),path.join(char,'normal.png'));
}
async function verify(){
 const results=[];
 const files=['xiaocui','yuehua','aming','tangtang','xiaochun','lanjie','ayi'].flatMap(id=>expressions.map(e=>'public/characters/'+id+'/gameboy/'+e+'.png')).concat(['logo','table','title-scene','parlor'].map(id=>'public/ui/gameboy/'+id+'.png'));
 const allowed=new Set(palette.map(c=>c.join(',')));
 for(const file of files){
  const {data,info}=await sharp(path.join(root,file)).ensureAlpha().raw().toBuffer({resolveWithObject:true});
  let badColor=0,badAlpha=0,edge=0;const colors=new Set();let minX=info.width,minY=info.height,maxX=-1,maxY=-1;
  for(let i=0;i<data.length;i+=4){
   if(data[i+3]!==0&&data[i+3]!==255)badAlpha++;
   if(data[i+3]){const c=Array.from(data.subarray(i,i+3)).join(',');colors.add(c);if(!allowed.has(c))badColor++;const p=i/4,x=p%info.width,y=Math.floor(p/info.width);minX=Math.min(minX,x);maxX=Math.max(maxX,x);minY=Math.min(minY,y);maxY=Math.max(maxY,y);if(!x||!y||x===info.width-1||y===info.height-1)edge++;}
  }
  results.push({file,width:info.width,height:info.height,badColor,badAlpha,colors:[...colors],bounds:[minX,minY,maxX,maxY],opaqueEdgePixels:edge,sha256:require('crypto').createHash('sha256').update(fs.readFileSync(path.join(root,file))).digest('hex')});
 }
 fs.writeFileSync(path.join(__dirname,'verification.json'),JSON.stringify(results,null,2)+'\n');
 console.log(JSON.stringify({assets:results.length,invalid:results.filter(r=>r.badColor||r.badAlpha),portraits:results.filter(r=>r.file.startsWith('public/characters')).length},null,2));
}
async function contact(){
 const ids=['xiaocui','yuehua','aming','tangtang','xiaochun','lanjie','ayi'];
 const layers=[];
 for(let y=0;y<ids.length;y++)for(let x=0;x<5;x++){const input=await sharp(path.join(root,'public/characters',ids[y],'gameboy',expressions[x]+'.png')).resize(160,160,{kernel:'nearest'}).png().toBuffer();layers.push({input,left:x*160,top:y*160});}
 await sharp({create:{width:800,height:1120,channels:4,background:'#26352c'}}).composite(layers).png().toFile(path.join(__dirname,'contact-sheet.png'));
}
const [task,id,source]=process.argv.slice(2);
({sheet:()=>sheet(id,source),background:()=>background(id,source),approved,verify,contact}[task]||(()=>{throw Error('unknown task');}))().catch(e=>{console.error(e);process.exitCode=1});

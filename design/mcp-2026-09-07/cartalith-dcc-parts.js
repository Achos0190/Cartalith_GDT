// Cartalith DCC Environment — heavy method bodies (map drawing, static data tables, menu rows).
// Extracted from the DC so the single-file design stays under the 256 KiB export cap.
// Each entry is called with the component as `this` by a one-line delegate in the logic class.
window.CDCC={
  draw(c,W,H,dpr){const ctx=c.getContext('2d');ctx.setTransform(dpr,0,0,dpr,0,0);const s=this.state;const v=this.view;const L=s.light;
    ctx.clearRect(0,0,W,H);
    const g=ctx.createRadialGradient(W/2,H*0.4,0,W/2,H*0.4,Math.max(W,H)*0.7);
    if(L){g.addColorStop(0,'#faf9f5');g.addColorStop(0.6,'#f1efe9');g.addColorStop(1,'#eae7e0')}else{g.addColorStop(0,'#17191a');g.addColorStop(0.6,'#101112');g.addColorStop(1,'#0d0e0f')}
    ctx.fillStyle=g;ctx.fillRect(0,0,W,H);
    const tints={biome:'rgba(110,165,95,.05)',political:'rgba(150,115,205,.055)',elevation:'rgba(224,163,74,.045)',slope:'rgba(120,140,170,.05)',flow:'rgba(90,140,190,.05)',temp:'rgba(210,110,80,.05)',rain:'rgba(80,130,200,.05)'};
    if(tints[s.layer]){ctx.fillStyle=tints[s.layer];ctx.fillRect(0,0,W,H)}
    const wx=x=>(x-v.cx)*v.s+W/2, wy=y=>(y-v.cy)*v.s+H/2;
    const ink=L?'0,0,0':'255,255,255';
    const rad=Math.hypot(W,H)/2/v.s;const x0=Math.max(0,v.cx-rad),x1=Math.min(4096,v.cx+rad),y0=Math.max(0,v.cy-rad),y1=Math.min(4096,v.cy+rad);
    ctx.lineWidth=1;
    const step=v.s>0.8?64:v.s>0.3?128:256;
    ctx.strokeStyle='rgba('+ink+',.05)';ctx.beginPath();
    for(let x=Math.ceil(x0/step)*step;x<=x1;x+=step){if(x%512){ctx.moveTo(wx(x),wy(y0));ctx.lineTo(wx(x),wy(y1))}}
    for(let y=Math.ceil(y0/step)*step;y<=y1;y+=step){if(y%512){ctx.moveTo(wx(x0),wy(y));ctx.lineTo(wx(x1),wy(y))}}
    ctx.stroke();
    ctx.strokeStyle='rgba('+ink+',.13)';ctx.beginPath();
    for(let x=Math.ceil(x0/512)*512;x<=x1;x+=512){ctx.moveTo(wx(x),wy(y0));ctx.lineTo(wx(x),wy(y1))}
    for(let y=Math.ceil(y0/512)*512;y<=y1;y+=512){ctx.moveTo(wx(x0),wy(y));ctx.lineTo(wx(x1),wy(y))}
    ctx.stroke();
    ctx.strokeStyle='rgba('+ink+',.22)';ctx.strokeRect(wx(0),wy(0),4096*v.s,4096*v.s);
    ctx.fillStyle='rgba('+ink+',.3)';ctx.font='10px IBM Plex Mono, monospace';
    for(let x=Math.ceil(x0/512)*512;x<=x1;x+=512){if(x>0&&x<4096)ctx.fillText(x.toLocaleString('en-US'),wx(x)+4,14)}
    for(let y=Math.ceil(y0/512)*512;y<=y1;y+=512){if(y>0&&y<4096)ctx.fillText(y.toLocaleString('en-US'),4,wy(y)-4)}
    const acc=L?'#a4650f':'#e0a34a';
    if(s.region){const r=s.region;ctx.strokeStyle=acc;ctx.setLineDash([5,4]);ctx.strokeRect(wx(r.x),wy(r.y),r.w*v.s,r.h*v.s);ctx.setLineDash([]);
      ctx.fillStyle=acc;[[r.x,r.y],[r.x+r.w,r.y],[r.x,r.y+r.h],[r.x+r.w,r.y+r.h]].forEach(p=>ctx.fillRect(wx(p[0])-3,wy(p[1])-3,6,6));}
    const mp=s.measure.pts;const msub=(s.measure.sub||'distance');
    if(mp.length){ctx.strokeStyle=acc;ctx.lineWidth=1.4;
      if(msub==='area'&&mp.length>2){ctx.fillStyle=acc;ctx.globalAlpha=0.1;ctx.beginPath();mp.forEach((p,i)=>i?ctx.lineTo(wx(p.x),wy(p.y)):ctx.moveTo(wx(p.x),wy(p.y)));ctx.closePath();ctx.fill();ctx.globalAlpha=1;ctx.stroke()}
      else if(msub==='radius'&&mp.length>1){const r=Math.hypot(mp[1].x-mp[0].x,mp[1].y-mp[0].y)*v.s;ctx.beginPath();ctx.arc(wx(mp[0].x),wy(mp[0].y),r,0,7);ctx.stroke();ctx.beginPath();ctx.moveTo(wx(mp[0].x),wy(mp[0].y));ctx.lineTo(wx(mp[1].x),wy(mp[1].y));ctx.stroke()}
      else{ctx.beginPath();mp.forEach((p,i)=>i?ctx.lineTo(wx(p.x),wy(p.y)):ctx.moveTo(wx(p.x),wy(p.y)));ctx.stroke()}
      ctx.fillStyle=L?'#111210':'#e8ebec';mp.forEach(p=>{ctx.beginPath();ctx.arc(wx(p.x),wy(p.y),3,0,7);ctx.fill()});
      ctx.fillStyle=acc;ctx.font='10px IBM Plex Mono, monospace';
      if(msub==='section'&&mp.length>1){ctx.fillText('A',wx(mp[0].x)+7,wy(mp[0].y)-7);ctx.fillText('B',wx(mp[1].x)+7,wy(mp[1].y)-7);
        const dx2=wx(mp[1].x)-wx(mp[0].x),dy2=wy(mp[1].y)-wy(mp[0].y);const ln=Math.hypot(dx2,dy2)||1;
        ctx.strokeStyle=acc;ctx.globalAlpha=0.55;for(let t=0.25;t<1;t+=0.25){const px2=wx(mp[0].x)+dx2*t,py2=wy(mp[0].y)+dy2*t;ctx.beginPath();ctx.moveTo(px2-dy2/ln*6,py2+dx2/ln*6);ctx.lineTo(px2+dy2/ln*6,py2-dx2/ln*6);ctx.stroke()}ctx.globalAlpha=1}
      else if(msub==='distance'||msub==='bearing'){for(let i=1;i<mp.length;i++){const a=mp[i-1],b=mp[i];const km=Math.hypot(b.x-a.x,b.y-a.y)*2.5;ctx.fillText(this.fmtKm(km),(wx(a.x)+wx(b.x))/2+6,(wy(a.y)+wy(b.y))/2-6)}}}
    if(s.sample){ctx.strokeStyle=acc;ctx.fillStyle=acc;ctx.beginPath();ctx.arc(wx(s.sample.x),wy(s.sample.y),4,0,7);ctx.fill();ctx.beginPath();ctx.arc(wx(s.sample.x),wy(s.sample.y),9,0,7);ctx.stroke()}
    if(this.drawExtra)this.drawExtra(ctx,wx,wy,acc,L);
    const an=this.an?this.an():{labels:[],icons:[],sel:-1};const cv=this.cv?this.cv():{places:[],pois:[]};
    if(this.terrTrail&&this.terrTrail.length){const fmap={};this.FACTIONS().forEach(f=>fmap[f[0]]=f[1]);
      this.terrTrail.forEach(p=>{ctx.fillStyle=p.mode==='subtract'?(L?'#f4f2ee':'#0d0e0f'):(fmap[p.f]||acc);ctx.globalAlpha=0.22;ctx.beginPath();ctx.arc(wx(p.x),wy(p.y),(this.cv().terrRadius||10)*8*v.s,0,7);ctx.fill()});ctx.globalAlpha=1}
    cv.places.forEach((p,i)=>{ctx.fillStyle=acc;ctx.save();ctx.translate(wx(p.x),wy(p.y));ctx.rotate(Math.PI/4);const r=p.cls==='city'||p.cls==='metropolis'?5:4;ctx.fillRect(-r/1.4,-r/1.4,r*1.4,r*1.4);ctx.restore();
      ctx.font='10px IBM Plex Mono, monospace';ctx.fillStyle=L?'#23241f':'#c8cbcd';ctx.fillText(p.name,wx(p.x)+8,wy(p.y)+3)});
    cv.pois.forEach(p=>{ctx.strokeStyle=acc;ctx.save();ctx.translate(wx(p.x),wy(p.y));ctx.rotate(Math.PI/4);ctx.strokeRect(-3,-3,6,6);ctx.restore()});
    an.icons.forEach(ic=>{ctx.strokeStyle=L?'#3d3f39':'#a9adb0';ctx.save();ctx.translate(wx(ic.x),wy(ic.y));ctx.rotate(Math.PI/4);const r=5*(ic.s||1);ctx.strokeRect(-r,-r,r*2,r*2);ctx.restore()});
    an.labels.forEach((lb,i)=>{ctx.font=(i===an.sel?'600 ':'')+'12px IBM Plex Mono, monospace';ctx.fillStyle=i===an.sel?acc:(L?'#111210':'#e8ebec');const tw=ctx.measureText(lb.text).width;ctx.fillText(lb.text,wx(lb.x)-tw/2,wy(lb.y));
      if(i===an.sel){ctx.strokeStyle=acc;ctx.globalAlpha=0.5;ctx.strokeRect(wx(lb.x)-tw/2-5,wy(lb.y)-13,tw+10,18);ctx.globalAlpha=1}});
    if(this.drawExtra4)this.drawExtra4(ctx,wx,wy,acc,L);
    if(this.drawExtra6)this.drawExtra6(ctx,wx,wy,acc,L);
    if(this.cursorW&&(s.tool==='biome'||s.tool==='sculpt'||s.tool==='freehand')){const r=(this.brushSize?this.brushSize():64)*v.s;ctx.strokeStyle=acc;ctx.globalAlpha=.8;ctx.beginPath();ctx.arc(wx(this.cursorW.x),wy(this.cursorW.y),r,0,7);ctx.stroke();ctx.globalAlpha=1}
  },
  drawExtra(ctx,wx,wy,acc,L){const sc=this.sc();const v=this.view;
    sc.stamps.forEach((st,i)=>{if(!st.vis)return;const seld=i===sc.sel;
      ctx.strokeStyle=acc;ctx.fillStyle=acc;
      if(st.radial){ctx.globalAlpha=seld?0.28:0.14;ctx.beginPath();ctx.arc(wx(st.pts[0].x),wy(st.pts[0].y),st.r*v.s,0,7);ctx.fill();ctx.globalAlpha=seld?1:0.55;ctx.beginPath();ctx.arc(wx(st.pts[0].x),wy(st.pts[0].y),st.r*v.s,0,7);ctx.stroke()}
      else{ctx.lineCap='round';ctx.lineJoin='round';ctx.globalAlpha=seld?0.3:0.15;ctx.lineWidth=Math.max(2,st.bs*v.s);ctx.beginPath();st.pts.forEach((p,j)=>j?ctx.lineTo(wx(p.x),wy(p.y)):ctx.moveTo(wx(p.x),wy(p.y)));ctx.stroke();
        ctx.globalAlpha=seld?1:0.6;ctx.lineWidth=1.2;ctx.beginPath();st.pts.forEach((p,j)=>j?ctx.lineTo(wx(p.x),wy(p.y)):ctx.moveTo(wx(p.x),wy(p.y)));ctx.stroke();
        if(st.type==='cliff'&&st.pts.length>1){const sgn=st.side==='right'?-1:1;ctx.lineWidth=1.2;
          for(let j=1;j<st.pts.length;j++){const ax=wx(st.pts[j-1].x),ay=wy(st.pts[j-1].y),bx=wx(st.pts[j].x),by=wy(st.pts[j].y);
            const dx2=bx-ax,dy2=by-ay;const ln=Math.hypot(dx2,dy2);if(ln<14)continue;
            const nx=dy2/ln*sgn,ny=-dx2/ln*sgn;const mx=(ax+bx)/2,my=(ay+by)/2;
            ctx.beginPath();ctx.moveTo(mx,my);ctx.lineTo(mx+nx*9,my+ny*9);ctx.stroke();
            ctx.beginPath();ctx.moveTo(mx*0.5+ax*0.5,my*0.5+ay*0.5);ctx.lineTo(mx*0.5+ax*0.5+nx*6,my*0.5+ay*0.5+ny*6);ctx.stroke();
            ctx.beginPath();ctx.moveTo(mx*0.5+bx*0.5,my*0.5+by*0.5);ctx.lineTo(mx*0.5+bx*0.5+nx*6,my*0.5+by*0.5+ny*6);ctx.stroke()}}}
      ctx.globalAlpha=1});
    if(this._liveStroke&&this._liveStroke.length>1){ctx.strokeStyle=acc;ctx.lineCap='round';ctx.lineJoin='round';ctx.globalAlpha=0.35;ctx.lineWidth=Math.max(2,this.sc().brush.size*v.s);ctx.beginPath();this._liveStroke.forEach((p,j)=>j?ctx.lineTo(wx(p.x),wy(p.y)):ctx.moveTo(wx(p.x),wy(p.y)));ctx.stroke();ctx.globalAlpha=1}
    if(this.paintTrail&&this.paintTrail.length){const col=(this.bpLegend().find(x=>x[0]===this.bp().value)||['','#5d8a5f'])[1];
      ctx.fillStyle=col;ctx.globalAlpha=0.3;const r=this.bp().radius*8*v.s;
      this.paintTrail.forEach(p=>{ctx.beginPath();ctx.arc(wx(p.x),wy(p.y),r,0,7);ctx.fill()});ctx.globalAlpha=1}},
  drawExtra4(ctx,wx,wy2,acc,L){const w=this.wy();const v=this.view;
    ctx.lineCap='round';ctx.lineJoin='round';
    w.ways.forEach(wa=>{ctx.strokeStyle=L?'#6b6f6a':'#8d9296';ctx.lineWidth=1.6;ctx.beginPath();wa.pts.forEach((p,j)=>j?ctx.lineTo(wx(p.x),wy2(p.y)):ctx.moveTo(wx(p.x),wy2(p.y)));ctx.stroke()});
    if(w.draft.length){ctx.strokeStyle=acc;ctx.lineWidth=1.4;ctx.setLineDash([6,5]);ctx.beginPath();w.draft.forEach((p,j)=>j?ctx.lineTo(wx(p.x),wy2(p.y)):ctx.moveTo(wx(p.x),wy2(p.y)));if(this.cursorW)ctx.lineTo(wx(this.cursorW.x),wy2(this.cursorW.y));ctx.stroke();ctx.setLineDash([]);
      ctx.fillStyle=acc;w.draft.forEach(p=>{ctx.beginPath();ctx.arc(wx(p.x),wy2(p.y),3,0,7);ctx.fill()})}},
  drawExtra6(ctx,wx,wy2,acc,L){const lm=this.lm();if(!lm.marks||!window.LM_GLYPHS)return;const v=this.view;
    if(!this._lmPaths)this._lmPaths={};
    ctx.strokeStyle=L?'#3d3f39':'#a9adb0';ctx.lineWidth=1.2;ctx.lineCap='round';ctx.lineJoin='round';
    const sc2=0.85;
    lm.marks.forEach(m=>{const g=window.LM_GLYPHS[m.type];if(!g)return;
      let ps=this._lmPaths[m.type];if(!ps){ps=g.ds.map(d=>new Path2D(d));this._lmPaths[m.type]=ps}
      const px=wx(m.x),py=wy2(m.y);if(px<-20||py<-20)return;
      ctx.save();ctx.translate(px-8*sc2,py-8*sc2);ctx.scale(sc2,sc2);ps.forEach(p=>ctx.stroke(p));ctx.restore()})},
  _terrPaint(w){const cv=this.cv();const cells={...cv.cells};const add=Math.round(Math.PI*cv.terrRadius*cv.terrRadius);
    cells[cv.faction]=Math.max(0,(cells[cv.faction]||0)+(cv.terrMode==='add'?add:-add));
    if(!this.terrTrail)this.terrTrail=[];this.terrTrail.push({...w,f:cv.faction,mode:cv.terrMode});this.setCv({cells},()=>{this.dirty=true})},
  sampleData(x,y){const h=(a,b)=>{const n=Math.sin(a*12.9898+b*78.233)*43758.5453;return n-Math.floor(n)};
    const elev=Math.round(-410+h(x,y)*4620);const land=elev>0;
    return{elev,slope:(h(x+1,y)*32).toFixed(1),aspect:Math.round(h(x,y+1)*360),plate:'P-'+(1+Math.floor(h(x,y)*14)),ptype:h(x+2,y)>0.5?'convergent':'divergent',bdist:Math.round(h(x,y+2)*900),resist:(0.2+h(x+3,y)*0.7).toFixed(2),lith:['granite','basalt','sediment','schist'][Math.floor(h(x,y+3)*4)],temp:Math.round(30-Math.abs(y-2048)/2048*48+h(x,y)*4),precip:Math.round(h(x+4,y)*2200),drain:['ocean','endorheic','river 3','river 5'][Math.floor(h(x,y+4)*4)],biome:land?['steppe','temperate forest','taiga','desert','tundra','rainforest'][Math.floor(h(x+5,y)*6)]:'ocean',soil:(h(x,y+5)*3.2).toFixed(1),land}},
  // ---- map pointers ----
  CAD(){if(this._cad)return this._cad;this._cad={sel:'terrain',search:'',ramp:'Earth',domain:'World',preset:'Atlas',edited:false,rampOpen:false,az:315,el:45,strength:0.62,multi:true,selStop:2,
    stops:[{e:-410,c:'#1d3140',interp:'linear'},{e:120,c:'#2e5a4a',interp:'linear'},{e:2640,c:'#B9A878',interp:'ease'},{e:3600,c:'#d8cdb0',interp:'linear'},{e:4210,c:'#efe9dd',interp:'step'}],
    layers:[{id:'labels',label:'Labels & annotation',vis:true,op:1},{id:'settlements',label:'Settlements',vis:true,op:1},{id:'ways',label:'Ways & routes',vis:true,op:0.9},{id:'political',label:'Political',vis:false,op:0.6},{id:'water',label:'Water',vis:true,op:1},{id:'veg',label:'Vegetation',vis:true,op:0.8},{id:'terrain',label:'Terrain',vis:true,op:0.78},{id:'hand',label:'Hand-drawn hillshade',vis:false,op:0.5,ind:1},{id:'hillshade',label:'Hillshade',vis:true,op:0.62,ind:1},{id:'relief',label:'Colour relief',vis:true,op:1,ind:1},{id:'land',label:'Land',vis:true,op:1},{id:'bg',label:'Background',vis:true,op:1}]};return this._cad},
  RAMPS(){return{Earth:'linear-gradient(90deg,#1d3140,#2e5a4a,#7a8a55,#b9a878,#d8cdb0,#efe9dd)',Elevation:'linear-gradient(90deg,#22304a,#2d6a58,#a8a06a,#b06a42,#8a4a3a,#f0ece4)',Atlas:'linear-gradient(90deg,#2a3140,#4a6a5a,#9aa571,#c9b789,#e5dcc4)',Mono:'linear-gradient(90deg,#14161a,#3a3f44,#7a8188,#b8bdc2,#eceff2)',Imhof:'linear-gradient(90deg,#5a6a7a,#8a9a8a,#c2b494,#e0cfa4,#f2e8cf)',Ice:'linear-gradient(90deg,#2a4a6a,#5a8aaa,#9ac2d8,#d0e5ef,#f4fafd)','Dark ice':'linear-gradient(90deg,#10202f,#26445c,#4a7690,#84aec4,#c6dfec)',Desert:'linear-gradient(90deg,#5a4432,#8a6a44,#b8905e,#d8b884,#efdcb4)','Dark atlas':'linear-gradient(90deg,#10141a,#243230,#4a5540,#7a744f,#a89a6a)'}},
  LMFAMS(){if(this._lmf)return this._lmf;
    const g=(p)=>'<svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">'+p+'</svg>';
    const T=(name,cls,o)=>({id:name.toLowerCase().replace(/[^a-z]+/g,'-'),name,cls,...o});
    this._lmf=[
      {id:'physical',label:'PHYSICAL',glyph:g('<path d="M2 12.5 6.2 4.5 9 10 11 7 14 12.5"></path><path d="M3.5 14.2 C6 13 10 13 12.5 14.2"></path>'),types:[
        T('Peak','REG',{cap:12,base:14,cand:640,noview:1}),T('Ridge','REG',{cap:0,was:8}),T('Saddle','LOC',{cap:0,was:5}),T('Cliff','LOC',{cap:0,was:20}),T('Gorge','REG',{cap:8,base:6,cand:410,fixed:'no terrain'}),T('Cave','LOC',{cap:0,was:12}),T('Waterfall','REG',{cap:40,base:11,cand:1284}),T('Spring','LOC',{cap:0,was:30}),T('Lake','REG',{cap:20,base:12,cand:520}),T('Delta','REG',{cap:0,was:3}),T('River confluence','LOC',{cap:30,base:24,cand:960}),T('Volcanic feature','CON',{cap:0,was:2,noview:1}),T('Rock formation','LOC',{cap:0,was:16}),T('Glacial feature','REG',{cap:0,was:6}),T('Ancient forest','REG',{cap:16,base:9,cand:300,fixed:'candidates'})]},
      {id:'transportation',label:'TRANSPORTATION',glyph:g('<path d="M1.5 11 C4 6.5 5.5 6.5 8 9 C10.5 11.5 12 11.5 14.5 7"></path><path d="M2.5 9 H13.5" stroke-dasharray="2 1.8"></path>'),types:[
        T('Mountain pass','REG',{cap:12,base:9,cand:88}),T('River crossing','LOC',{cap:0,was:20}),T('Ford','LOC',{cap:20,base:8,cand:340,fixed:'no terrain'}),T('Bridge site','LOC',{cap:0,was:12}),T('Road junction','LOC',{cap:0,was:8}),T('Caravan station','LOC',{cap:0,was:6}),T('Portage','LOC',{cap:0,was:4}),T('Harbour','REG',{cap:8,base:4,cand:60,fixed:'candidates'})]},
      {id:'economic',label:'ECONOMIC',glyph:g('<path d="M4.5 6 H11.5 L14 12 H2 Z"></path><path d="M7 8.5 L9 10"></path>'),types:[
        T('Mine','LOC',{cap:12,base:6,cand:520}),T('Quarry','LOC',{cap:5,base:3,cand:280,fixed:'no terrain'}),T('Salt works','LOC',{cap:0,was:4}),T('Resource extraction site','LOC',{cap:0,was:8}),T('Market site','LOC',{cap:0,was:6}),T('Trade depot','LOC',{cap:0,was:5})]},
      {id:'military',label:'MILITARY',glyph:g('<path d="M4 14 V5 H5.8 V3.2 H7 V5 H9 V3.2 H10.2 V5 H12 V14"></path><path d="M2.5 14 H13.5"></path>'),types:[
        T('Fort','REG',{cap:12,base:10,cand:240,noview:1}),T('Watchtower','LOC',{cap:30,base:15,cand:680,noview:1}),T('Fortified pass','REG',{cap:5,base:8,cand:44}),T('Fortified crossing','LOC',{cap:0,was:6}),T('Battlefield','CUL',{cap:0,was:10}),T('Border marker','LOC',{cap:8,base:3,cand:90,fixed:'candidates',noview:1})]},
      {id:'religious',label:'RELIGIOUS · CULTURAL',glyph:g('<path d="M3.5 14 V8 C3.5 5 5.5 3.5 8 3.5 C10.5 3.5 12.5 5 12.5 8 V14"></path><path d="M6 14 V9.5 C6 8 6.8 7 8 7 C9.2 7 10 8 10 9.5 V14"></path>'),types:[
        T('Shrine','CUL',{cap:50,base:18,cand:1420}),T('Temple','CUL',{cap:8,base:7,cand:120}),T('Sacred grove','CUL',{cap:12,base:8,cand:380,fixed:'no terrain'}),T('Sacred mountain','CON',{cap:3,base:5,cand:22,noview:1}),T('Pilgrimage site','CUL',{cap:0,was:6}),T('Tomb','CUL',{cap:8,base:5,cand:210,fixed:'candidates'}),T('Monument','CUL',{cap:0,was:8}),T('Ceremonial site','CUL',{cap:0,was:7})]},
      {id:'historical',label:'HISTORICAL',glyph:g('<path d="M4.5 13.5 V5 M4.5 5 H6.5 M4.5 13.5 H6.5"></path><path d="M10 13.5 V9.5 L11.5 7.5 V5 M9.5 13.5 H12"></path>'),types:[
        T('Ruin','CUL',{cap:20,base:5,cand:600}),T('Abandoned settlement','CUL',{cap:8,base:3,cand:150}),T('Ancient road','CUL',{cap:3,base:1,cand:40,fixed:'candidates'}),T('Historic battlefield','CUL',{cap:0,was:6}),T('Destroyed fortress','CUL',{cap:0,was:4}),T('Historic crossing','CUL',{cap:0,was:5})]}];
    return this._lmf},
  scFeats(){if(this._scF)return this._scF;
    const g=(p)=>'<svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">'+p+'</svg>';
    this._scF=[
      {id:'mountains',label:'Mountains',hint:'stroke · add — height, peak sharpness, ridge frequency, ruggedness',svg:g('<path d="M1.5 12.5 6 4.5 8.8 9.5"></path><path d="M6.5 12.5 10.5 5.5 14.5 12.5"></path>'),params:[{k:'h',label:'height',min:0.1,max:0.55,step:0.01,v:0.42},{k:'sharp',label:'peak sharpness',min:0.6,max:3,step:0.1,v:1.5},{k:'rfreq',label:'ridge freq',min:0.6,max:5,step:0.1,v:1.6},{k:'rug',label:'ruggedness',min:0,max:1,step:0.01,v:0.55}]},
      {id:'hills',label:'Hills',hint:'stroke · add — soft rolling amplitude',svg:g('<path d="M1.5 12 Q4 8 6.5 10.8 Q9.5 6.8 11.5 10.2 Q13 9 14.5 12"></path><path d="M1.5 12.5 H14.5"></path>'),params:[{k:'amp',label:'amplitude',min:0.02,max:0.3,step:0.01,v:0.11},{k:'freq',label:'rolling freq',min:0.5,max:4,step:0.1,v:1.4},{k:'soft',label:'softness',min:0,max:1,step:0.01,v:0.7}]},
      {id:'ridge',label:'Ridge',hint:'stroke · add — one crest along the stroke axis',svg:g('<path d="M2 12.5 8 4 14 12.5"></path><path d="M8 4 V12.5" stroke-dasharray="1.5 1.5"></path>'),params:[{k:'h',label:'height',min:0.02,max:0.35,step:0.01,v:0.15},{k:'w',label:'width frac',min:0.1,max:0.6,step:0.01,v:0.28},{k:'det',label:'detail freq',min:0.5,max:4,step:0.1,v:1.5}]},
      {id:'plateau',label:'Plateau',hint:'stroke · set — never lowers existing terrain',svg:g('<path d="M1.5 12.5 4.5 6.5 H11.5 L14.5 12.5"></path><path d="M4.5 6.5 H11.5" stroke-width="2"></path>'),params:[{k:'rise',label:'rise',min:0.03,max:0.45,step:0.01,v:0.26},{k:'terr',label:'terraces',min:1,max:8,step:1,v:4},{k:'det',label:'detail freq',min:0.4,max:3,step:0.1,v:1.1}]},
      {id:'cliff',label:'Cliff',hint:'stroke · add — direction-sensitive, high side left of the stroke',svg:g('<path d="M1.5 5.5 H7.5 V12.5 H14.5"></path>'),params:[{k:'rise',label:'rise',min:0.05,max:0.45,step:0.01,v:0.22},{k:'steep',label:'steepness',min:0.2,max:1,step:0.01,v:0.75}]},
      {id:'canyon',label:'Canyon',hint:'stroke · add negative — walls closing to a flat floor',svg:g('<path d="M1.5 4.5 H5.6 L6.6 11.5 H9.4 L10.4 4.5 H14.5"></path>'),params:[{k:'depth',label:'depth',min:0.03,max:0.35,step:0.01,v:0.18},{k:'wall',label:'wall steepness',min:0,max:1,step:0.01,v:0.7},{k:'mea',label:'meander',min:0,max:0.8,step:0.01,v:0.35}]},
      {id:'valley',label:'Valley',hint:'stroke · add negative — U-shaped trough',svg:g('<path d="M1.5 4.5 C3 11.5 6 12.5 8 12.5 C10 12.5 13 11.5 14.5 4.5"></path>'),params:[{k:'depth',label:'depth',min:0.03,max:0.3,step:0.01,v:0.14},{k:'w',label:'width frac',min:0.3,max:1,step:0.01,v:0.85},{k:'mea',label:'meander',min:0,max:0.8,step:0.01,v:0.3}]},
      {id:'river',label:'River',hint:'stroke · set — writes riverMask and riverFloor on commit',svg:g('<path d="M2 5.2 C5 3.6 7 7 10 5.4 12 4.4 13.4 5.4 14 5"></path><path d="M2 10.8 C5 9.2 7 12.6 10 11 12 10 13.4 11 14 10.6"></path>'),params:[{k:'w',label:'width',min:2,max:26,step:1,v:7,unit:' px'},{k:'depth',label:'depth',min:0.02,max:0.22,step:0.01,v:0.09},{k:'mea',label:'meander',min:0,max:0.6,step:0.01,v:0.28},{k:'branch',label:'branch noise',min:0,max:1,step:0.01,v:0.5}]},
      {id:'lake',label:'Lake',hint:'radial · set — tap places it, brush size is the radius; fills lakeMask on commit',svg:g('<path d="M3 9 C3 6.8 5.2 5.6 8 5.6 C10.8 5.6 13 6.8 13 9 C13 11 10.8 12 8 12 C5.2 12 3 11 3 9 Z"></path><path d="M4.6 7.4 6.4 6.6"></path>'),radial:1,params:[{k:'depth',label:'depth',min:0.03,max:0.3,step:0.01,v:0.13},{k:'shore',label:'shore',min:0.05,max:0.6,step:0.01,v:0.25}]},
      {id:'basin',label:'Basin',hint:'stroke · add negative — endorheic, no outlet',svg:g('<path d="M2 6 C4 11 12 11 14 6"></path><path d="M4.5 6.5 C6 9.5 10 9.5 11.5 6.5"></path>'),params:[{k:'depth',label:'depth',min:0.02,max:0.25,step:0.01,v:0.1},{k:'rough',label:'floor rough',min:0,max:1,step:0.01,v:0.4}]},
      {id:'coastline',label:'Coastline',hint:'stroke · set — pulls terrain toward sea level',svg:g('<path d="M1.5 7 3 5.5 4.5 7.5 6 5.2 8 7 9.5 5.5 11 7.8 12.5 6 14.5 7.4"></path><path d="M2 11.5 H14" stroke-dasharray="2 1.6"></path>'),params:[{k:'amt',label:'amount',min:0.1,max:1,step:0.01,v:0.85},{k:'rag',label:'raggedness',min:0.4,max:4,step:0.1,v:1.6}]},
      {id:'volcano',label:'Volcano',hint:'radial · add — tap places the cone; crater notch at the summit',svg:g('<path d="M2 12.5 6 4.5 H6.6 L8 6.4 9.4 4.5 H10 L14 12.5"></path>'),radial:1,params:[{k:'cone',label:'cone height',min:0.15,max:0.6,step:0.01,v:0.45},{k:'crat',label:'crater depth',min:0,max:0.9,step:0.01,v:0.5},{k:'rad',label:'radius',min:30,max:200,step:5,v:110,unit:' px'},{k:'flank',label:'flank rough',min:0,max:1,step:0.01,v:0.6}]},
      {id:'freehand',label:'Freehand',hint:'continuous drag or tap — sub-mode below; a one-point stroke degenerates to radial',svg:g('<path d="M3.2 12.8 3.8 10.2 11 3 13 5 5.8 12.2 Z"></path><path d="M10 4 12 6"></path>'),params:[{k:'amt',label:'amount',min:0.02,max:0.3,step:0.01,v:0.12}]}];
    return this._scF},
  menuRowsFor(id){const s=this.state;const p=s.prefs;const sub=s.sub;const R=[];const push=(...a)=>R.push(...a);
    if(id==='file'){push(this.it('New world…','mock:New world modal — name, seed, extent, working resolution',{sc:'⌘N'}),this.it('Open project…','mock:Open project — .zip archive picker',{sc:'⌘O'}),this.it('Recent worlds','sub:recent',{glyph:'▸'}));
      if(sub==='recent')push(this.it('VHAREN REACH — 129384 · 5 d ago','world:VHAREN REACH:129384',{ind:36}),this.it('KESSA — 774201 · 3 w ago','world:KESSA:774201',{ind:36}),this.it('ELDRA — 483920 · 2 d ago','world:ELDRA:483920',{ind:36}));
      push(this.sep(),this.it('Save project','save',{sc:'⌘S'}),this.it('Save as…','mock:Save as — the new path becomes the project path',{sc:'⌘⇧S'}),
      this.tog('Autosave','autosave',p.autosave),this.segRow('interval','autoInt',['off','1 min','5 min','15 min'],p.autoInt),
      this.it('Revert to last save','mock:Revert — discards in-memory changes including sculpt drafts'),this.it('Close project','pick',{sc:'⌘W'}),this.sep(),
      this.head('STORAGE LOCATIONS'),this.read('PROJECTS','~/Cartalith/Worlds'),this.read('TILE ATLAS','~/Cartalith/Cache/atlas'),this.read('ASSET PACKS','~/Cartalith/Packs'),this.read('EXPORTS','~/Cartalith/Exports'),
      this.it('Change locations…','mock:Change locations — one folder picker per root; moving the atlas root invalidates the cache'),this.it('Show project on disk','mock:Revealed in the OS file manager'),
      this.note('imports live under Data ▸ Import · asset packs under Assets'))}
    if(id==='edit'){const un=s.undoStack.length,re=s.redoStack.length;
      push(this.it('Undo'+(un?' — '+s.undoStack[un-1].label:''),'undo',{sc:'⌘Z',dim:!un}),this.it('Redo'+(re?' — '+s.redoStack[re-1].label:''),'redo',{sc:'⌘⇧Z',dim:!re}),this.it('Undo history','sub:hist',{glyph:'▸',dim:!un}));
      if(sub==='hist')s.undoStack.slice().reverse().forEach((e,i)=>push(this.it(e.label,'histjump:'+i,{ind:36})));
      push(this.sep(),this.it('Cut','mock:Cut — operates on the current selection',{sc:'⌘X',dim:!s.sample}),this.it('Copy','mock:Copy',{sc:'⌘C',dim:!s.sample}),this.it('Paste','mock:Paste',{sc:'⌘V',dim:true}),this.it('Delete','delsel',{sc:'⌫',dim:!s.sample}),this.sep(),
      this.it('Select all','mock:Select all — scoped to the active layer',{sc:'⌘A'}),this.it('Deselect','desel',{sc:'⌘D'}),this.it('Find on map…','mock:Find — places, labels, factions, routes; result pans the viewport',{sc:'⌘F'}))}
    if(id==='assets'){push(this.it('Asset library','win:assets',{glyph:'⧉',sc:'⇧A'}),this.it('Sprite sheet slicer','win:slicer',{glyph:'⧉'}),this.sep(),
      this.it('Import image…','mock:Import image — lands in Unassigned imports'),this.it('Import asset pack .zip…','mock:Import pack — loads into the library for editing'),
      this.it('Asset pack','sub:pack',{glyph:'▸'}));
      if(sub==='pack')push(this.head('ACTIVE PACK'),this.read('NAME','Eldra Atlas Pack'),this.read('AUTHOR','A. Chos'),this.read('LICENSE','CC-BY 4.0'),this.read('SCHEMA','2 · STORED zip'),this.read('FILLED','148 of 212 · 26 MB'),
        this.it('Pack metadata…','mock:Pack metadata — name, author, license',{ind:36}),this.it('Validate pack','mock:Validate — 8 warnings',{ind:36}),this.it('Export pack .zip…','mock:Export — pack.json schema 2 + PNGs, STORED zip',{ind:36,sc:'⌘⇧P'}));
      push(this.it('Icon families','sub:fam',{glyph:'▸'}));
      if(sub==='fam')push(this.it('P · Places — 10 of 12','win:assets',{ind:36}),this.it('B · Buildings — 18 of 24','win:assets',{ind:36}),this.it('T · Trees & cover — 22 of 22','win:assets',{ind:36}),this.it('C · Compass & frame — 6 of 8','win:assets',{ind:36}),this.note('24 families — full list in the Asset library'));
      push(this.it('Texture sets','win:assets',{glyph:'▸'}),this.sep(),this.it('Landmark types','sub:lmt',{glyph:'▸'}));
      if(sub&&sub.startsWith('lmt')){const lm=this.lm();const openF=sub.startsWith('lmt:')?sub.slice(4):null;
        this.LMFAMS().forEach(f=>{const armed=f.types.filter(t=>lm.types[t.id].armed).length;const placed=f.types.reduce((a,t)=>a+lm.res[t.id].placed,0);
          push(this.it(f.label,'sub:lmt:'+f.id,{ind:36,glyph:openF===f.id?'▾':'▸',sc:armed+' of '+f.types.length+' · '+placed+' placed'}));
          if(openF===f.id){f.types.forEach(t=>{const st=lm.types[t.id];const r=lm.res[t.id];
            push(this.read((st.armed?'● ':'○ ')+t.name,st.armed?st.cap+' max · '+r.placed+' placed · '+r.reason:'off · was '+st.cap))});
            push(this.it('Open in CIVIL ▸ Landmarks','lmjump:'+f.id,{ind:36}))}});
        push(this.note('leaves are read-only — the dropdown is a shortcut into the panel, never a second implementation of it'),
          this.read('LANDMARK ICONS','poi · 10 slots'),this.it('Landmark label style… → Cartography','dom:CARTO',{ind:36}))}
      push(this.sep(),this.it('Apply library to map','mock:Library compiled and loaded as the live pack'),this.it('Clear library…','mock:Clear library — destructive, confirmation required',{danger:1}))}
    if(id==='data'){push(this.head('IMPORT'),this.it('Maps · heightmaps (PNG · TIFF)','win:data:imp-maps',{glyph:'⧉'}),this.it('GIS / GeoJSON','win:data:imp-gis',{glyph:'⧉'}),this.it('World data (.zip · fields)','win:data:imp-world',{glyph:'⧉'}),
      this.head('EXPORT'),this.it('Maps (image · tiles)','win:data:exp-maps',{glyph:'⧉'}),this.it('GIS / GeoJSON','win:data:exp-gis',{glyph:'⧉'}),this.it('World data','win:data:exp-world',{glyph:'⧉'}),this.it('Assets (pack .zip)','win:data:exp-assets',{glyph:'⧉'}),
      this.head('SOURCES'),this.it('External sources','win:data:sources',{glyph:'⧉'}),this.it('Source registry','win:data:registry',{glyph:'⧉'}),
      this.head('CONVERSION'),this.it('Coordinate systems (EPSG)','win:data:crs',{glyph:'⧉'}),this.it('Format conversion','win:data:convert',{glyph:'⧉'}),
      this.head('VALIDATION'),this.it('Check data — 3 warnings','win:data:check',{glyph:'⧉'}),this.it('Repair / normalize','win:data:repair',{glyph:'⧉'}))}
    if(id==='prefs'){push(this.head('PERFORMANCE'),this.tog('GPU acceleration — WebGPU','gpu',p.gpu),this.read('DEVICES','GPU 0 · 16 GB · 71% / GPU 1 · 64%'),this.segRow('multi-gpu','mgpu',['split tiles','alt frames','single'],p.mgpu),this.read('CPU WORKERS','12 of 16'),this.read('VRAM BUDGET','12 GB · fallback CPU tile pass'),
      this.head('GRAPHICS'),this.segRow('quality','quality',['perf','balanced','quality','ultra'],p.quality),this.segRow('anti-aliasing','aa',['off','2×','4×','8×'],p.aa==='MSAA 4×'?'4×':p.aa),this.segRow('relief exaggeration','relief',['1×','2×','4×'],p.relief||'2×'),this.read('COLOUR','sRGB · anisotropy 8'),
      this.head('TILES & LOD'),this.read('TILED LOD','auto on zoom · 512 px · L0–L8'),this.read('ATLAS CACHE','6.2 of 24 GB'),this.it('Clear caches…','mock:Cleared atlas + field caches — never project data'),
      this.head('MEMORY'),this.segRow('undo depth','undoDepth',['5','15','50'],String(p.undoDepth)),this.read('WORKING SET','1.6 GB of 12 GB'),
      this.head('APPLICATION'),this.segRow('theme','theme',['dark','light'],s.light?'light':'dark'),this.segRow('units','units',['km','mi'],p.units),this.it('Keyboard shortcuts…','mock:Editable shortcut table — per context'),this.it('Storage locations…','mock:Same modal as File ▸ Change locations'))}
    if(id==='window'){push(this.tog('Left dock','win:ld',s.ldOpen),this.tog('Right dock','win:rd',s.rdOpen),this.tog('Domain rail','win:rail',s.showRail),this.tog('Status bar','win:sb',s.showSB),this.tog('Timeline (CIVIL · INFRA)','win:tl',s.tlOpen),this.sep(),
      this.it('Reset layout','resetlayout'),this.it('Save layout as…','mock:Layout saved as preset'),this.sep(),this.head('WORKSPACES'),this.it('World','dom:WORLD'),this.it('Civilization','dom:CIVIL'),this.it('Cartography','dom:CARTO'))}
    if(id==='help'){push(this.it('Documentation','mock:Documentation opens in the OS browser'),this.it('Keyboard shortcuts','mock:V M R B L I arm tools · ⌘Z undo · Esc commits or disarms'),this.it('Credits & academic principles','mock:Credits — generation follows published geomorphology'),this.it('Report an issue','mock:Issue reporter'),this.sep(),this.read('VERSION','2.11 · build 4183'))}
    return R.map((r,idx)=>({key:idx,isHead:r.t==='h',isSep:r.t==='s',isIt:r.t==='i',isTog:r.t==='t',isSeg:r.t==='g',isRead:r.t==='r',isNote:r.t==='n',
      label:r.label||'',act:r.act||'',sc:r.sc||'',glyph:r.glyph||'',ind:r.ind||14,val:r.val||'',
      col:r.danger?'var(--block)':r.dim?'var(--dis)':'var(--body)',
      togBg:r.on?'var(--acc)':'var(--ins)',togX:r.on?15:2,
      opts:(r.opts||[]).map(v=>({v,act:r.act+'='+v,col:v===r.val?'var(--acc)':'var(--dim)',bg:v===r.val?'var(--wash2)':'transparent'}))}))},
  lmCompute(types,crowd,compete){const res={};
    this.LMFAMS().forEach(f=>f.types.forEach(t=>{const st=types[t.id];
      if(!st.armed){res[t.id]={placed:0,reason:''};return}
      const base=t.base!=null?t.base:Math.max(1,Math.round((t.was||6)*0.6));
      const room=Math.max(1,Math.round(base/Math.pow(crowd,1.6)*(compete?1:1.35)));
      let placed,reason;
      if(t.fixed==='no terrain'){placed=Math.min(st.cap,base);reason=placed>=st.cap?'at cap':'no terrain'}
      else if(t.fixed==='candidates'){placed=Math.min(st.cap,base);reason=placed>=st.cap?'at cap':'candidates'}
      else{placed=Math.min(st.cap,room);reason=placed>=st.cap?'at cap':'spacing'}
      res[t.id]={placed,reason}}));
    return res},
  lmMarks(types,crowd,compete){const res=this.lmCompute(types,crowd,compete);const marks=[];const h=(a,b)=>{const n=Math.sin(a*127.1+b*311.7)*43758.5453;return n-Math.floor(n)};
    let k=0;this.LMFAMS().forEach(f=>f.types.forEach(t=>{const n=res[t.id].placed;
      for(let i=0;i<n;i++){k++;const x=200+h(k,i+1)*3700,y=200+h(i+2,k)*3700;marks.push({x,y,type:t.name,fam:f.id})}}));
    return marks},
  lmRun(){const lm=this.lm();if(lm.run)return;clearInterval(this._lmInt);let pct=0;
    this.setLm({run:{pct:0}});
    this._lmInt=setInterval(()=>{pct+=9+Math.random()*14;
      if(pct>=100){clearInterval(this._lmInt);this._lmInt=null;
        const t=new Date();const hh=String(t.getHours()).padStart(2,'0')+':'+String(t.getMinutes()).padStart(2,'0');
        const res=this.lmCompute(this.lm().types,this.lm().crowd,this.lm().compete);
        const marks=this.lmMarks(this.lm().types,this.lm().crowd,this.lm().compete);
        this.setLm({run:null,res,marks,lastRun:hh,edited:false},()=>{this.dirty=true});
        const total=Object.values(res).reduce((a,x)=>a+x.placed,0);
        this.toast('Landmark pass — '+total+' placed across '+Object.values(this.lm().types).filter(x=>x.armed).length+' armed types')}
      else this.setLm({run:{pct}})},130)},
  secSample(){const mp=this.ms().pts;if(mp.length<2)return null;const a=mp[0],b=mp[1];const n=120;const f=this.ms().field;
    const vals=[];for(let i=0;i<=n;i++){const t=i/n;const sd=this.sampleData(a.x+(b.x-a.x)*t,a.y+(b.y-a.y)*t);
      vals.push(f==='Terrain'?+sd.slope:f==='Climate'?sd.temp:f==='Hydrology'?sd.precip:f==='Geology'?Math.round(sd.resist*100):Math.max(sd.elev,0))}
    const min=Math.min(...vals),max=Math.max(...vals);const rng=Math.max(max-min,1);
    let d='M0,130 ';vals.forEach((v2,i)=>{d+='L'+(i/n*1000).toFixed(1)+','+(126-((v2-min)/rng*118)).toFixed(1)+' '});d+='L1000,130 Z';
    const unit=f==='Terrain'?'°':f==='Climate'?' °C':f==='Hydrology'?' mm':f==='Geology'?'':' m';
    return{d,min,max,unit,len:Math.hypot(b.x-a.x,b.y-a.y)*2.5,mean:Math.round(vals.reduce((x,y2)=>x+y2,0)/vals.length)}},
  PSTAGES(){return[{name:'Vhal Serai → Kess Ford',terr:'plains · steppe',km:118},{name:'Kess Ford → Thornwood',terr:'forest · temperate',km:92},{name:'Thornwood → High Saddle',terr:'mountain · alpine',km:76,closed:true},{name:'High Saddle → Grey Vale',terr:'mountain · alpine',km:64},{name:'Grey Vale → Lakemouth',terr:'hills · steppe',km:88},{name:'Lakemouth → Port Amre',terr:'river · water leg',km:102,water:true}]},
  FACTIONS(){return[['Vhal Serai Compact','#6a9bc4'],['Kessan League','#c96a5a'],['Free Marches','#6fae7d']]},
  PRESETS(){return[{id:'rolling',label:'Rolling Hills',f:'hills',p:{amp:0.09,freq:1.1,soft:0.85}},{id:'alps',label:'Alps',f:'mountains',p:{h:0.5,sharp:2.4,rfreq:2.2,rug:0.7}},{id:'rockies',label:'Rockies',f:'mountains',p:{h:0.44,sharp:1.8,rfreq:1.4,rug:0.6}},{id:'badlands',label:'Badlands',f:'canyon',p:{depth:0.22,wall:0.85,mea:0.55}},{id:'volcisle',label:'Volcanic Isle',f:'volcano',p:{cone:0.52,crat:0.6,rad:140,flank:0.7}},{id:'mesa',label:'Mesa',f:'plateau',p:{rise:0.3,terr:5,det:1.4}},{id:'karst',label:'Karst',f:'hills',p:{amp:0.16,freq:2.8,soft:0.3}},{id:'glacial',label:'Glacial Valley',f:'valley',p:{depth:0.2,w:0.95,mea:0.15}}]},
  RND(){return{tint:0.7,hs:0.62,water:0.5,atmo:0.3}},
  AND(){return{labels:[],icons:[],sel:-1,draft:'New label',sizeMode:'scale',anchor:'centre',iconFam:'Places',iconVar:'capital-star',iconScale:1}},
  _updateHud(){const v=this.view;if(this.zoomRef.current)this.zoomRef.current.textContent='zoom '+Math.round(v.s*100)+'%';
    if(this.scaleLabRef.current)this.scaleLabRef.current.textContent=this.fmtKm(120/v.s*2.5);
    if(this.coordsRef.current){const c=this.cursorW;this.coordsRef.current.textContent=c?this.fmtKm(c.x*2.5)+' E · '+this.fmtKm(c.y*2.5)+' N · '+(this.sampleData(c.x,c.y).elev.toLocaleString('en-US'))+' m':'— · —'}},
  vals5(){const s=this.state;const m=this.ms();const sc=this.sc();const bp=this.bp();const tool=s.tool;
    const isBrush=tool==='sculpt'||tool==='freehand'||tool==='biome';
    const isMeas=tool==='measure';
    const g=(p)=>'<svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">'+p+'</svg>';
    const groups=[{id:'SCULPT',on:tool==='sculpt'||tool==='freehand',svg:g('<path d="M1.5 11.5 8 5M9 2l3 3-2.6 2.6-3-3z"></path>')},{id:'PAINT',on:tool==='biome',svg:g('<path d="M2 12h4l6-6-3-3-6 6z"></path>')},{id:'MEASURE',on:isMeas,svg:g('<path d="M1.5 12.5 12.5 1.5"></path><circle cx="2" cy="12" r="1.3" fill="currentColor"></circle><circle cx="12" cy="2" r="1.3" fill="currentColor"></circle>')}];
    const mp=m.pts;
    let mTotal=0;for(let i=1;i<mp.length;i++)mTotal+=Math.hypot(mp[i].x-mp[i-1].x,mp[i].y-mp[i-1].y)*2.5;
    const bear2=mp.length>1?Math.round((Math.atan2(mp[1].x-mp[0].x,-(mp[1].y-mp[0].y))*180/Math.PI+360)%360):0;
    let areaKm=0;if(mp.length>2){let acc2=0;for(let i=0;i<mp.length;i++){const a=mp[i],b=mp[(i+1)%mp.length];acc2+=a.x*b.y-b.x*a.y}areaKm=Math.abs(acc2)/2*6.25}
    const radKm=mp.length>1?Math.hypot(mp[1].x-mp[0].x,mp[1].y-mp[0].y)*2.5:0;
    const sec=m.sub==='section'?this.secSample():null;
    const fmtA=a2=>Math.round(a2).toLocaleString('en-US')+' km²';
    const totals={distance:this.fmtKm(mTotal),bearing:mp.length>1?('00'+bear2).slice(-3)+'°':'—',area:mp.length>2?fmtA(areaKm*(m.water?0.88:1)):'—',radius:mp.length>1?this.fmtKm(radKm):'—',section:sec?this.fmtKm(sec.len):'—'};
    const bigLabels={distance:'TOTAL LENGTH',bearing:'BEARING A → B',area:'AREA · PROJECTED',radius:'RADIUS',section:'SECTION · '+m.field.toUpperCase()};
    const hints={distance:'click drops points · double-click or Esc ends',bearing:'two points · A then B',area:'click vertices · double-click closes the ring',radius:'click the centre, then the edge',section:'click A, then B — profile reads below the map'};
    const statRows=[];const SR=(k,v,col)=>statRows.push({k,v,col:col||'var(--sec)'});
    if(m.sub==='bearing'&&mp.length>1){SR('BACK-BEARING','↺ '+('00'+((bear2+180)%360)).slice(-3)+'°');SR('LENGTH',this.fmtKm(mTotal));SR('Δ ELEVATION',(this.sampleData(mp[1].x,mp[1].y).elev-this.sampleData(mp[0].x,mp[0].y).elev).toLocaleString('en-US')+' m')}
    if(m.sub==='area'&&mp.length>2){SR('TRUE SURFACE',fmtA(areaKm*1.036));if(m.water)SR('WATER SUBTRACTED','−'+fmtA(areaKm*0.12));SR('PERIMETER',this.fmtKm(mTotal+Math.hypot(mp[0].x-mp[mp.length-1].x,mp[0].y-mp[mp.length-1].y)*2.5));SR('VERTICES',String(mp.length))}
    if(m.sub==='radius'&&mp.length>1){SR('DIAMETER',this.fmtKm(radKm*2));SR('CIRCUMFERENCE',this.fmtKm(2*Math.PI*radKm));SR('ENCLOSED AREA',fmtA(Math.PI*radKm*radKm))}
    if(m.sub==='section'&&sec){SR('MIN · MAX',sec.min.toLocaleString('en-US')+' · '+sec.max.toLocaleString('en-US')+sec.unit);SR('MEAN',sec.mean.toLocaleString('en-US')+sec.unit);SR('BEARING',('00'+bear2).slice(-3)+'°');SR('SAMPLES','120 · 1 per '+this.fmtKm(sec.len/120))}
    const L=s.light;
    return{
      railShow:s.scr==='app'&&(isBrush||isMeas),
      railGroups:groups.map(x=>({id:x.id,svg:{__html:x.svg},bg:x.on?'var(--acc)':'var(--ins)',col:x.on?'var(--accInk)':'var(--sec)'})),
      hRailGroup:e=>{const id=e.currentTarget.dataset.id;this.armTool(id==='SCULPT'?'sculpt':id==='PAINT'?'biome':'measure')},
      railMeasure:isMeas,railBrush:isBrush,
      measTools:[['distance','Distance'],['bearing','Bearing'],['area','Area'],['radius','Radius']].map(([id,label])=>({id,label,bg:m.sub===id?'var(--wash2)':'var(--ins)',col:m.sub===id?'var(--acc)':'var(--sec)'})),
      hMeasSub:e=>this.setMs({sub:e.currentTarget.dataset.id,pts:[],done:false},()=>{this.dirty=true}),
      secLabCol:m.sub==='section'?'var(--acc)':'var(--faint)',
      secFields:['Elevation','Terrain','Climate','Hydrology','Geology'].map(id=>({id,bg:m.sub==='section'&&m.field===id?'var(--wash2)':'var(--ins)',col:m.sub==='section'&&m.field===id?'var(--acc)':'var(--sec)'})),
      hSecField:e=>this.setMs({sub:'section',field:e.currentTarget.dataset.id,pts:this.ms().sub==='section'?this.ms().pts:[],done:false},()=>{this.dirty=true}),
      railSizePct:tool==='biome'?Math.round((bp.radius-2)/28*100)+'%':Math.round((sc.brush.size-6)/194*100)+'%',
      railSizeDisp:tool==='biome'?bp.radius+' cells':sc.brush.size+' px · '+this.fmtKm(sc.brush.size*2.5),
      railSizeSlide:e=>{if(tool==='biome')this.startSlide(e,p=>this.setBp({radius:Math.round(2+p*28)}));else this.startSlide(e,p=>this.setSc({brush:{...this.sc().brush,size:Math.round((6+p*194)/2)*2}}))},
      railHardOn:tool!=='biome',railHardPct:Math.round(sc.brush.hard*100)+'%',railHardDisp:(+sc.brush.hard).toFixed(2),
      railBrushNote:tool==='biome'?'shape is shared with sculpt · ⇧ erases':'shape + falloff detail in the dock below',
      measIsDist:m.sub==='distance',measIsArea:m.sub==='area',measIsSection:m.sub==='section',
      measField:m.field,
      measWaterBg:m.water?'var(--wash2)':'var(--ins)',measWaterCol:m.water?'var(--acc)':'var(--dim)',
      hMeasWater:()=>this.setMs({water:!m.water}),
      measTotal:totals[m.sub],measBigLabel:bigLabels[m.sub],
      measBigSub:m.sub==='distance'?(Math.max(mp.length-1,0))+' segments · '+mp.length+' points · great circle':m.sub==='section'&&sec?'A → B locked · '+m.field.toLowerCase():m.sub==='area'?(m.water?'water subtracted':'projected'):'',
      measHint:hints[m.sub],measRdHint:hints[m.sub],
      measShowSegs:m.sub==='distance'||m.sub==='bearing',
      measStatRows:statRows,
      measDelta:(()=>{if(m.sub!=='distance'||mp.length<3)return'';const st2=Math.hypot(mp[mp.length-1].x-mp[0].x,mp[mp.length-1].y-mp[0].y)*2.5;return'straight line '+this.fmtKm(st2)+' — along path +'+Math.round((mTotal/Math.max(st2,0.01)-1)*100)+'%'})(),
      hudBottom:(m.sub==='section'&&sec&&isMeas)?'180px':'12px',
      secStrip:m.sub==='section'&&!!sec&&isMeas,
      secLen:sec?this.fmtKm(sec.len):'',secHalf:sec?this.fmtKm(sec.len/2):'',
      secField:m.field.toLowerCase(),
      secMinMax:sec?('min '+sec.min.toLocaleString('en-US')+sec.unit+' · max '+sec.max.toLocaleString('en-US')+sec.unit):'',
      secTop:sec?sec.max.toLocaleString('en-US')+sec.unit:'',secMid:sec?Math.round((sec.min+sec.max)/2).toLocaleString('en-US'):'',secBot:sec?sec.min.toLocaleString('en-US'):'',
      secArea:sec?sec.d:'M0,130 L1000,130 Z',
      secGridCol:L?'rgba(0,0,0,.07)':'rgba(255,255,255,.06)',
      secFillCol:L?'rgba(164,101,15,.12)':'rgba(224,163,74,.13)',secLineCol:L?'#a4650f':'#e0a34a',
      sfHasSide:sc.feature==='cliff'&&tool!=='freehand',
      sideLeftCol:(sc.side||'left')==='left'?'var(--acc)':'var(--dim)',sideLeftBg:(sc.side||'left')==='left'?'var(--wash2)':'transparent',
      sideRightCol:sc.side==='right'?'var(--acc)':'var(--dim)',sideRightBg:sc.side==='right'?'var(--wash2)':'transparent',
      hScSide:e=>this.setSc({side:e.currentTarget.dataset.v}),
      hStampFlip:e=>{e.stopPropagation();const i=+e.currentTarget.dataset.i;const sc2=this.sc();const stamps=sc2.stamps.map((x,j)=>j===i?{...x,side:x.side==='right'?'left':'right'}:x);this.setSc({stamps},()=>{this.dirty=true});this.toast('High side flipped — now '+(stamps[i].side)+' of the stroke')}
    };
  },
  // ---- infra: ways + planner ----
  vals6(){const s=this.state;const lm=this.lm();const LAD=this.LM_LADDER();
    const civ=s.domain==='CIVIL';const cc=this.cc();
    const armedAll=Object.values(lm.types).filter(x=>x.armed);
    const capsTotal=armedAll.reduce((a,x)=>a+x.cap,0);
    const placedTotal=Object.values(lm.res).reduce((a,x)=>a+x.placed,0);
    const room=Math.round(210/Math.pow(lm.crowd,1.6)*(lm.compete?1:1.35));
    const funnelT=lm.funnel?this.lmType(lm.funnel):null;
    const fT=funnelT?lm.types[funnelT.id]:null;
    const catDefs=[['landmarks','LANDMARKS'],['factions','FACTIONS & SETTLEMENTS'],['infra','WAYS & ROUTES'],['planner','JOURNEY PLANNER']];
    const noViewCount=this.LMFAMS().reduce((a,f)=>a+f.types.filter(t=>t.noview).length,0);
    const famGroups=this.LMFAMS().map(f=>{const armed=f.types.filter(t=>lm.types[t.id].armed);
      const placed=f.types.reduce((a,t)=>a+lm.res[t.id].placed,0);
      const open=lm.openFam===f.id;
      return{id:f.id,label:f.label,glyph:{__html:f.glyph},open,chev:open?'90deg':'0deg',
        count:armed.length+' of '+f.types.length+' armed · '+placed+' placed',
        rows:!open?[]:f.types.filter(t=>!lm.classFilter||t.cls===lm.classFilter).map(t=>{const st=lm.types[t.id];const r=lm.res[t.id];
          const idx=st.armed?LAD.reduce((best,v2,i2)=>Math.abs(v2-st.cap)<Math.abs(LAD[best]-st.cap)?i2:best,0):0;
          return{id:t.id,name:t.name,badge:t.cls,noview:!!t.noview,armed:st.armed,
            glyph:{__html:(window.LM_GLYPHS&&window.LM_GLYPHS[t.name])?'<svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">'+window.LM_GLYPHS[t.name].inner+'</svg>':''},
            dotBg:st.armed?'var(--acc)':'transparent',nameCol:st.armed?'var(--body)':'var(--dis)',
            capPct:Math.round(idx/(LAD.length-1)*100)+'%',
            capDisp:st.armed?st.cap+' max':'off',wasOff:!st.armed,wasNote:st.armed?'':'was '+st.cap,
            resolved:st.armed&&!lm.edited,placedPct:st.armed&&st.cap?Math.min(100,Math.round(r.placed/st.cap*100))+'%':'0%',
            placedLine:r.placed+' placed · '+r.reason,
            reasonCol:r.reason==='at cap'?'var(--acc)':'var(--dim)',
            funnelOpen:lm.funnel===t.id}})}});
    const setType=(id,patch)=>{const types={...lm.types,[id]:{...lm.types[id],...patch}};this.setLm({types,edited:true})};
    return{
      ldCivilDock:civ,civCats:catDefs.map(([id,label])=>({id,label,on:cc===id,col:cc===id?'var(--acc)':'var(--faint)',chev:cc===id?'90deg':'0deg'})),
      hCivCat:e=>{const id=e.currentTarget.dataset.id;this.setCc(this.cc()===id&&id!=='landmarks'?'landmarks':id)},
      lmCatCol:cc==='landmarks'?'var(--acc)':'var(--faint)',lmCatChev:cc==='landmarks'?'90deg':'0deg',
      lmCatCount:armedAll.length+' armed · '+placedTotal+' on the map',
      facCatCol:cc==='factions'?'var(--acc)':'var(--faint)',facCatChev:cc==='factions'?'90deg':'0deg',
      infCatCol:cc==='infra'?'var(--acc)':'var(--faint)',infCatChev:cc==='infra'?'90deg':'0deg',
      plnCatCol:cc==='planner'?'var(--acc)':'var(--faint)',plnCatChev:cc==='planner'?'90deg':'0deg',
      ldLandmarks:civ&&cc==='landmarks',
      lmHeadroom:'caps total '+capsTotal+' · room for about '+room+' at this spacing · last run placed '+placedTotal,
      lmCrowdPct:Math.round((lm.crowd-0.25)/1.75*100)+'%',lmCrowdDisp:'× '+lm.crowd.toFixed(2),
      lmCrowdKm:'a regional landmark keeps '+this.fmtKm(lm.radii.REG*lm.crowd),
      hLmCrowd:e=>this.startSlide(e,p=>this.setLm({crowd:+(0.25+p*1.75).toFixed(2),edited:true})),
      lmCompeteBg:lm.compete?'var(--acc)':'var(--sur)',lmCompeteX:lm.compete?15:2,
      hLmCompete:()=>this.setLm({compete:!lm.compete,edited:true}),
      lmAdv:!!lm.adv,lmAdvChev:lm.adv?'90deg':'0deg',hLmAdv:()=>this.setLm({adv:!lm.adv}),
      lmRadii:['CON','REG','LOC','CUL'].map(k=>({k,label:{CON:'Continental',REG:'Regional',LOC:'Local',CUL:'Cultural'}[k],disp:this.fmtKm(lm.radii[k]*lm.crowd),pct:Math.round(lm.radii[k]/160*100)+'%'})),
      hLmRadius:e=>{const k=e.currentTarget.dataset.k;this.startSlide(e,p=>this.setLm({radii:{...this.lm().radii,[k]:Math.round(2+p*158)},edited:true}))},
      lmClassChips:['CON','REG','LOC','CUL'].map(id=>({id,col:lm.classFilter===id?'var(--acc)':'var(--dim)',bg:lm.classFilter===id?'var(--wash2)':'var(--ins)'})),
      hLmClass:e=>{const id=e.currentTarget.dataset.id;this.setLm({classFilter:lm.classFilter===id?null:id})},
      lmNoViewNote:noViewCount+' types score without viewshed — not wired: the engine has no visibility analysis yet. Intended once it lands: score = 0.6 × prominence + 0.4 × visible land area inside 30 km, caps unchanged',
      famGroups,hLmFam:e=>{const id=e.currentTarget.dataset.id;this.setLm({openFam:lm.openFam===id?null:id})},
      hLmBulk:e=>{e.stopPropagation();const ds=e.currentTarget.dataset;const fam=this.LMFAMS().find(f=>f.id===ds.id);
        const types={...lm.types};fam.types.forEach(t=>{types[t.id]={...types[t.id],armed:ds.v==='on'}});this.setLm({types,edited:true})},
      hLmSlide:e=>{const id=e.currentTarget.dataset.id;this.startSlide(e,p=>{const i=Math.round(p*(LAD.length-1));
        const lm2=this.lm();const cur=lm2.types[id];const v=LAD[i];
        if(v===0){if(cur.armed)this.setLm({types:{...lm2.types,[id]:{...cur,armed:false}},edited:true})}
        else this.setLm({types:{...lm2.types,[id]:{armed:true,cap:v}},edited:true})})},
      hLmRowFunnel:e=>{e.stopPropagation();const id=e.currentTarget.dataset.id;this.setLm({funnel:lm.funnel===id?null:id})},
      lmRunning:!!lm.run,lmRunPct:lm.run?Math.min(100,Math.round(lm.run.pct))+'%':'0%',
      lmRunLabel:lm.run?'placing… '+Math.min(99,Math.round(lm.run.pct))+'%':'Run landmark pass',
      hLmRun:()=>this.lmRun(),
      lmStale:lm.edited,lmLastRun:'last run '+lm.lastRun+' · '+placedTotal+' placed · results below are that run',
      lmFunnelOn:!!funnelT,lmFunnelTitle:funnelT?funnelT.name.toUpperCase()+' · LAST RUN':'',
      lmFunnelRows:funnelT?(()=>{const r=lm.res[funnelT.id];const cand=funnelT.cand||400;const placed=r.placed;
        const f1=Math.round(cand*0.7),f2=Math.round((cand-f1)*0.62);const sp=Math.max(0,cand-f1-f2-placed);
        const rows=[{k:'candidates evaluated',v:cand.toLocaleString('en-US'),col:'var(--sec)'},
          {k:'failed '+(funnelT.fam==='physical'?'min flow accumulation':'physical constraints'),v:'− '+f1+'   '+(cand-f1)+' left',col:'var(--sec)'},
          {k:'failed type constraints',v:'− '+f2+'   '+(cand-f1-f2)+' left',col:'var(--sec)'},
          {k:'rejected by spacing',v:'− '+sp+'   '+placed+' left',col:'var(--sec)'},
          {k:'cap '+(fT?fT.cap:0),v:r.reason==='at cap'?'reached':'not reached',col:r.reason==='at cap'?'var(--acc)':'var(--faint)'}];
        return rows})():[],
      lmFunnelPlaced:funnelT?lm.res[funnelT.id].placed+' placed':'',
      hLmFunnelClose:()=>this.setLm({funnel:null})
    };
  },
  // ---- horizontal rail: measure subs + brush ----
  valsCarto(){const s=this.state;const carto=s.domain==='CARTO';const cc2=this.ct();
    const lab=this.lab();const ico=this.ico();
    const CL=[['continental','Continental','#e0a34a','26/2.5 · .28 em',4],['region','Region','#c8cbcd','18/2 · .20 em',11],['settlement','Settlement','#a9adb0','13/1.5 · .06 em',48],['water','Water','#6f9fb5','15/1.5 · .14 em italic',22],['landmark','Landmark','#8d9296','11/1.2 · .06 em',37]];
    const FAM={PLACES:[10,12],TREES:[22,22],'SEA MARKS':[6,8],POI:[10,12]};
    const f=FAM[ico.fam];
    const on=(b)=>b?'var(--acc)':'transparent';
    return{
      ldCartoDock:carto,
      hCartoCat:e=>{const id=e.currentTarget.dataset.id;this.setState({cartoCat:this.ct()===id&&id!=='style'?'style':id})},
      caStyleCol:cc2==='style'?'var(--acc)':'var(--faint)',caStyleChev:cc2==='style'?'90deg':'0deg',caStyleCount:this.ca().layers.filter(l=>l.vis).length+' of '+this.ca().layers.length+' visible',
      caLabCol:cc2==='labels'?'var(--acc)':'var(--faint)',caLabChev:cc2==='labels'?'90deg':'0deg',caLabCount:'122 drawn · 9 culled',
      caIcoCol:cc2==='icons'?'var(--acc)':'var(--faint)',caIcoChev:cc2==='icons'?'90deg':'0deg',caIcoCount:'48 placed',
      caTerCol:cc2==='terrain'?'var(--acc)':'var(--faint)',caTerChev:cc2==='terrain'?'90deg':'0deg',caTerCount:'mock',
      ldCarto:carto&&cc2==='style',ldLabels:carto&&cc2==='labels',ldIcons:carto&&cc2==='icons',ldRender:carto&&cc2==='terrain',
      labClasses:CL.map(([id,label,sw,spec,count])=>({id,label,swatch:sw,spec,count:String(count),col:lab.sel===id?'var(--acc)':'var(--body)'})),
      hLabClass:e=>this.setLab({sel:e.currentTarget.dataset.id}),
      labSelTitle:(CL.find(c=>c[0]===lab.sel)||CL[2])[1].toUpperCase()+' · TYPE',
      labFields:[{k:'size',label:'size',pct:Math.round((lab.size[lab.sel]-8)/26*100)+'%',disp:lab.size[lab.sel]+' px'},
        {k:'halo',label:'halo',pct:Math.round(lab.halo[lab.sel]/4*100)+'%',disp:lab.halo[lab.sel].toFixed(1)+' px'},
        {k:'track',label:'tracking',pct:Math.round(lab.track[lab.sel]/0.4*100)+'%',disp:lab.track[lab.sel].toFixed(2)+' em'}],
      hLabField:e=>{const k=e.currentTarget.dataset.k;this.startSlide(e,p=>{const l=this.lab();
        const v=k==='size'?Math.round(8+p*26):k==='halo'?+(p*4).toFixed(1):+(p*0.4).toFixed(2);
        this.setLab({[k]:{...l[k],[l.sel]:v}})})},
      labCollBg:lab.collision?'var(--acc)':'var(--sur)',labCollX:lab.collision?15:2,
      hLabColl:()=>this.setLab({collision:!lab.collision}),
      labCollNote:lab.collision?'9 labels suppressed at this zoom — zoom in to recover them':'off — labels may overlap; export will not fix it',
      icoFams:Object.keys(FAM).map(id=>({id,bg:ico.fam===id?'var(--wash2)':'var(--ins)',col:ico.fam===id?'var(--acc)':'var(--sec)'})),
      hIcoFam:e=>this.setIco({fam:e.currentTarget.dataset.id}),
      icoSlotLine:f[0]+' of '+f[1]+' slots filled · unfilled slots fall back to the family default glyph',
      icoFields:[{k:'scale',label:'icon scale',pct:Math.round((ico.scale-0.5)/1.5*100)+'%',disp:Math.round(ico.scale*100)+'%'},
        {k:'spacing',label:'min spacing',pct:Math.round(ico.spacing/40*100)+'%',disp:ico.spacing+' px'}],
      hIcoField:e=>{const k=e.currentTarget.dataset.k;this.startSlide(e,p=>this.setIco({[k]:k==='scale'?+(0.5+p*1.5).toFixed(2):Math.round(p*40)}))},
      icoRules:[['avoidLabels','avoid label boxes'],['minSpacing','enforce min spacing'],['snapCoast','snap sea marks to coast']].map(([id,label])=>({id,label,bg:on(ico.rules[id]),col:ico.rules[id]?'var(--body)':'var(--dim)'})),
      hIcoRule:e=>{const id=e.currentTarget.dataset.id;this.setIco({rules:{...this.ico().rules,[id]:!this.ico().rules[id]}})}
    }},
  // ---- landmarks (CIVIL) ----
  LABD(){return{sel:'settlement',size:{continental:26,region:18,settlement:13,water:15,landmark:11},
    halo:{continental:2.5,region:2,settlement:1.5,water:1.5,landmark:1.2},
    track:{continental:0.28,region:0.2,settlement:0.06,water:0.14,landmark:0.06},collision:true}},
  ICOD(){return{fam:'PLACES',scale:1,spacing:14,rules:{avoidLabels:true,minSpacing:true,snapCoast:false}}},
  tbLabelExtra(tb){if(tb==='sculpt')return'SCULPT · '+(this.state.tool==='freehand'?this.sc().free.toUpperCase():this.sc().feature.toUpperCase());if(tb==='biome')return'PAINT · '+this.bp().target.toUpperCase();return null}
};

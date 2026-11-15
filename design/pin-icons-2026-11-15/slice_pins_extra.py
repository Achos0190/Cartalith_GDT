## Cuts the owner's third sheet (2026-11-15, 38 painted pins on a charcoal ground,
## labelled) into transparent teardrops. Gold rim against a dark ground: fit a
## teardrop (centre, R, tip factor K) maximising inside-rim brightness minus
## outside brightness, starting from hand-read approximate centres.
import sys, numpy as np
from PIL import Image, ImageDraw
im=np.asarray(Image.open(sys.argv[1]).convert('RGB')); L=im.astype(float).mean(2); H,W=L.shape
def pts(cx,cy,R,K,n=180):
    t=K*R; phi=np.arccos(R/t); s,e=np.pi/2+phi,np.pi/2-phi+2*np.pi
    a=np.linspace(s,e,n); P=np.c_[cx+R*np.cos(a),cy+R*np.sin(a)]; N=np.c_[np.cos(a),np.sin(a)]
    T=np.array([cx,cy+t])
    for ang in (s,e):
        q=np.array([cx+R*np.cos(ang),cy+R*np.sin(ang)]); u=np.linspace(0,1,40)[:,None]
        d=T-q; nr=np.array([d[1],-d[0]])/np.linalg.norm(d)
        if np.dot(nr,q-np.array([cx,cy]))<0: nr=-nr
        P=np.r_[P,q+d*u]; N=np.r_[N,np.repeat(nr[None],40,0)]
    return P,N
def smp(Q):
    x=np.clip(Q[:,0].round().astype(int),0,W-1); y=np.clip(Q[:,1].round().astype(int),0,H-1); return L[y,x]
def score(cx,cy,R,K):
    P,N=pts(cx,cy,R,K); return smp(P-3*N).mean()-smp(P+3*N).mean()
rows=[(105,[95,252,407,558,707,853,999,1141,1287,1442]),(345,[95,252,407,558,707,853,999,1141,1287,1442]),
      (585,[95,252,407,558,707,853,999,1141,1287,1442]),(820,[98,277,452,625,793,963,1131,1300])]
names=['city','hamlet','tavern','smithy','shrine','guild','gate','well','camp','other',
'spring','river_confluence','delta','ford','portage','historic_crossing','mountain_pass','ridge','cliff','gorge',
'rock_formation','glacier','quarry','salt_works','road_junction','caravan_station','trade_depot','ancient_road','fort','watchtower',
'fortified_pass','fortified_crossing','battlefield','border_marker','sacred_grove','sacred_mountain','ceremonial_site','tomb']
i=0
for gy,xs in rows:
    for gx in xs:
        best=(-1e9,)
        for dx in range(-16,17,4):
            for dy in range(-20,21,4):
                for R in range(56,77,4):
                    for K in (1.6,1.75,1.9):
                        s=score(gx+dx,gy+dy,R,K)
                        if s>best[0]: best=(s,gx+dx,gy+dy,R,K)
        _,bx,by,bR,bK=best
        for dx in range(-3,4):
            for dy in range(-3,4):
                for R in np.arange(bR-3,bR+3.5,1):
                    for K in np.arange(bK-0.1,bK+0.11,0.025):
                        s=score(bx+dx,by+dy,R,K)
                        if s>best[0]: best=(s,bx+dx,by+dy,R,K)
        _,CX,CY,R,K=best; R+=0.5; t=K*R; S=4; pad=3
        x0,y0,x1,y1=int(CX-R-pad),int(CY-R-pad),int(CX+R+pad)+1,int(CY+t+pad)+1
        m=Image.new('L',((x1-x0)*S,(y1-y0)*S),0); dr=ImageDraw.Draw(m); ox,oy=(CX-x0)*S,(CY-y0)*S
        dr.ellipse([ox-R*S,oy-R*S,ox+R*S,oy+R*S],fill=255)
        phi=np.arccos(R/t)
        dr.polygon([(ox+R*S*np.cos(np.pi/2+phi),oy+R*S*np.sin(np.pi/2+phi)),(ox+R*S*np.cos(np.pi/2-phi),oy+R*S*np.sin(np.pi/2-phi)),(ox,oy+t*S)],fill=255)
        m=m.resize((x1-x0,y1-y0),Image.LANCZOS)
        c=Image.fromarray(im).crop((x0,y0,x1,y1)).convert('RGBA'); c.putalpha(m)
        c.save(f"p3_{i:02d}_{names[i]}.png"); print(names[i],round(CX),round(CY),R,round(K,3),round(best[0],1)); i+=1

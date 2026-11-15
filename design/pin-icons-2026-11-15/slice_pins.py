## Cuts the owner's two 5x4 teardrop sheets (2026-11-15) into transparent PNGs.
## Every pin has a dark outer outline. For each cell, a teardrop outline
## (circle radius R, tip K*R below the centre, K measured by hand on cell 0,0)
## is fitted by maximising (inside + outside brightness) - 2 * (on-line
## brightness) over centre and R, coarse then fine. Alpha = that shape at 4x.
import sys, numpy as np
from PIL import Image, ImageDraw
def outline_pts(cx,cy,R,K,n=160):
    t=K*R; a=np.arcsin(1/K)
    # circle arc from the left tangent point over the top to the right one
    th=np.linspace(np.pi/2+ (np.pi/2-a) - np.pi, np.pi/2 - (np.pi/2-a) - np.pi, n)  # placeholder
    angs=np.linspace(np.pi/2 + (np.pi/2 - a), np.pi/2 - (np.pi/2 - a) + 2*np.pi, n)
    angs=np.linspace(np.pi - a + np.pi/2 - np.pi/2, 0, n)
    # parametrise: tangent points at angle phi from +x axis measured downward = a'
    phi=np.arccos(R/t)          # angle between centre->tip (down) and centre->tangent point
    start=np.pi/2+phi; end=np.pi/2-phi+2*np.pi   # y down: angle pi/2 points to the tip
    ang=np.linspace(start,end,n)
    P=np.c_[cx+R*np.cos(ang), cy+R*np.sin(ang)]
    N=np.c_[np.cos(ang), np.sin(ang)]
    T=np.array([cx,cy+t])
    for s in (start,end):
        q=np.array([cx+R*np.cos(s), cy+R*np.sin(s)])
        u=np.linspace(0,1,24)[:,None]; seg=q+(T-q)*u
        d=T-q; nrm=np.array([d[1],-d[0]])/np.linalg.norm(d)
        if np.dot(nrm, q-np.array([cx,cy]))<0: nrm=-nrm
        P=np.r_[P,seg]; N=np.r_[N,np.repeat(nrm[None],24,0)]
    return P,N
def score(L,cx,cy,R,K):
    P,N=outline_pts(cx,cy,R,K)
    def samp(Q):
        x=np.clip(Q[:,0].round().astype(int),0,L.shape[1]-1); y=np.clip(Q[:,1].round().astype(int),0,L.shape[0]-1)
        return L[y,x]
    on=np.minimum(samp(P),np.minimum(samp(P+N),samp(P-N)))
    return samp(P+4*N).mean()+samp(P-4*N).mean()-2*on.mean()
def cut(path,tag,K,R0,RS=12,GRID=None):
    im=np.asarray(Image.open(path).convert('RGB')); L=im.astype(float).mean(2); H,W=L.shape
    for r in range(4):
        for c in range(5):
            x0,y0=int(c*W/5),int(r*H/4); cell=L[y0:y0+H//4, x0:x0+W//5]
            ys,xs=np.nonzero(cell<28); cy,cx=cell.shape[0]*0.42,cell.shape[1]*0.5
            for _ in range(6):
                k=(ys-cy)**2+(xs-cx)**2<(cell.shape[1]*0.30)**2; cy,cx=ys[k].mean(),xs[k].mean()
            gx,gy=x0+cx,y0+cy
            best=(-1e9,)
            if GRID:
                ## Painted sheet: centres sit on a regular grid (column and row
                ## medians of the cells the free fit got right); only a +-3 px
                ## refinement is searched, because its bronze rim against a
                ## brown ground misled the free fit on four cells.
                best=(score(L,GRID[0][c],GRID[1][r],R0,K),GRID[0][c],GRID[1][r],R0)
            else:
             for dx in range(-24,25,4):
                for dy in range(-12,33,4):
                    for R in range(R0-RS,R0+RS+1,max(1,RS//3) if RS else 1):
                        s=score(L,gx+dx,gy+dy,R,K)
                        if s>best[0]: best=(s,gx+dx,gy+dy,R)
            _,bx,by,bR=best
            for dx in np.arange(-3,3.5,1):
                for dy in np.arange(-3,3.5,1):
                    for R in (np.arange(bR-3,bR+3.5,1) if RS>3 else np.arange(R0-RS,R0+RS+0.5,1)):
                        s=score(L,bx+dx,by+dy,R,K)
                        if s>best[0]: best=(s,bx+dx,by+dy,R)
            _,CX,CY,R=best; R+=1.0; t=K*R
            S=4; pad=3
            x0b,y0b,x1b,y1b=int(CX-R-pad),int(CY-R-pad),int(CX+R+pad)+1,int(CY+t+pad)+1
            m=Image.new('L',((x1b-x0b)*S,(y1b-y0b)*S),0); dr=ImageDraw.Draw(m)
            ox,oy=(CX-x0b)*S,(CY-y0b)*S
            dr.ellipse([ox-R*S,oy-R*S,ox+R*S,oy+R*S],fill=255)
            phi=np.arccos(R/t)
            p1=(ox+R*S*np.cos(np.pi/2+phi),oy+R*S*np.sin(np.pi/2+phi)); p2=(ox+R*S*np.cos(np.pi/2-phi),oy+R*S*np.sin(np.pi/2-phi))
            dr.polygon([p1,p2,(ox,oy+t*S)],fill=255)
            m=m.resize((x1b-x0b,y1b-y0b),Image.LANCZOS)
            crop=Image.fromarray(im).crop((x0b,y0b,x1b,y1b)).convert('RGBA'); crop.putalpha(m)
            crop.save(f"{tag}_{r}{c}.png"); print(tag,r,c,round(CX),round(CY),round(R,1),round(best[0],1))
cut(sys.argv[1],'glyph',137/102,101)
cut(sys.argv[2],'painted',131/93,92,2,GRID=([193,481,767,1054,1342],[109,355,598,842]))

import csv,sys
from PIL import Image,ImageDraw
D=sys.argv[1]
def load(n): return list(csv.DictReader(open(f'{D}/{n}_384.csv')))
bl=load('bldg'); rg=load('rings')
def draw(seed,town,out):
    ims=[]
    for v in ('off','proto'):
        B=[(float(b['x']),float(b['y'])) for b in bl if b['seed']==seed and b['town']==town and b['variant']==v]
        R=[(float(r['x']),float(r['y'])) for r in rg if r['seed']==seed and r['town']==town and r['variant']==v]
        pts=B+R
        x0=min(p[0] for p in pts)-20;y0=min(p[1] for p in pts)-20;x1=max(p[0] for p in pts)+20;y1=max(p[1] for p in pts)+20
        sc=min(700/(x1-x0),700/(y1-y0))
        im=Image.new('RGB',(int((x1-x0)*sc),int((y1-y0)*sc)),(235,230,215));d=ImageDraw.Draw(im)
        T=lambda p:((p[0]-x0)*sc,(p[1]-y0)*sc)
        for b in B: q=T(b); d.rectangle([q[0]-1,q[1]-1,q[0]+1,q[1]+1],fill=(80,60,50))
        if R: d.line([T(p) for p in R+[R[0]]],fill=(200,30,30),width=2); [d.ellipse([T(p)[0]-3,T(p)[1]-3,T(p)[0]+3,T(p)[1]+3],outline=(0,0,200)) for p in R]
        ims.append(im)
    W=sum(i.width for i in ims)+10;H=max(i.height for i in ims)
    o=Image.new('RGB',(W,H),(255,255,255));x=0
    for i in ims: o.paste(i,(x,0));x+=i.width+10
    o.save(out)
for s,t in [('912345','32'),('246371','9'),('912345','1')]: draw(s,t,f'ring_{D}_{s}_{t}.png')

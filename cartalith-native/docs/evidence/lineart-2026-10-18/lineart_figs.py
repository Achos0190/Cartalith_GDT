import numpy as np, sys
from PIL import Image, ImageDraw
OUT=sys.argv[1]; FIG=sys.argv[2]
vis=np.load(OUT+'/vis.npy',allow_pickle=True).item()
def tile(img,H):
    img=np.asarray(img,np.uint8)
    if H==40: img=np.repeat(np.repeat(img,5,0),5,1)
    t=np.full((200,200,3),255,np.uint8); h,w=img.shape[:2]; y=(200-h)//2; x=(200-w)//2
    t[y:y+h,x:x+w]=img[:200,:200]; return t
def label(im,txt,xy):
    d=ImageDraw.Draw(im); d.rectangle([xy[0],xy[1],xy[0]+7*len(txt)+4,xy[1]+13],fill=(255,255,255)); d.text((xy[0]+2,xy[1]+1),txt,fill=(0,0,0))
ROWS=[("legacy (today)","legacy"),("coverage 1024","coverage"),("SDF (128|512) + floor","sdfmip"),("medial-axis 256","mat256")]
COLS=[(40,"x1: 40 px (shown 5x)"),(160,"x4: 160 px"),(640,"x16: 200 px crop"),(2560,"x64: 200 px crop")]
for name in vis:
    if name in ("binarise",) or name.startswith("sdfres"): continue
    V=vis[name]
    V["sdfmip"]={40:V["sdf128_floor"][40],160:V["sdf512_floor"][160],640:V["sdf512_floor"][640],2560:V["sdf512_floor"][2560]}
    im=Image.new("RGB",(130+4*204,20+4*204),(250,250,250))
    for c,(H,t) in enumerate(COLS): label(im,t,(130+c*204,3))
    for r,(t,k) in enumerate(ROWS):
        label(im,t,(2,20+r*204+90))
        for c,(H,_) in enumerate(COLS):
            im.paste(Image.fromarray(tile(V[k][H],H)),(130+c*204,20+r*204))
    im.save(f"{FIG}/grid_{name}.png",optimize=True)
# binarisation panel (scan source, left tower crop, 1:1)
rb=vis["binarise"]; src=np.asarray(Image.open(OUT+'/src_scan.png'))
y0,x0=150,90
panels=[("scan input",src[y0:y0+300,x0:x0+300])]
for k,t in (("chroma_tol40","chroma key tol 40 (today)"),("otsu_raw","Otsu, raw"),("sauvola_flat","Sauvola, flattened"),("otsu_flat_clean_close1","flatten+Otsu+despeckle+close1")):
    m=rb[k][y0:y0+300,x0:x0+300]; panels.append((t,np.where(m[...,None],np.array([20,20,20],np.uint8),np.array([255,255,255],np.uint8))))
im=Image.new("RGB",(5*304,320),(250,250,250))
for i,(t,p) in enumerate(panels):
    im.paste(Image.fromarray(p.astype(np.uint8)),(i*304,18)); label(im,t,(i*304,2))
im.save(f"{FIG}/binarise_scan.png",optimize=True)
# SDF storage resolution at x64
for name in ("castle","mountain"):
    S=vis["sdfres_"+name]
    im=Image.new("RGB",(3*204,220),(250,250,250))
    for i,R in enumerate((64,128,256)):
        im.paste(Image.fromarray(np.asarray(S[R],np.uint8)),(i*204,18)); label(im,f"SDF {R}^2, x64 crop",(i*204,2))
    im.save(f"{FIG}/sdf_resolution_x64_{name}.png",optimize=True)
# sources
im=Image.new("RGB",(4*404,420),(250,250,250))
for i,n in enumerate(("pine","castle","mountain","scan")):
    s=Image.open(f"{OUT}/src_{n}.png").resize((400,400),Image.LANCZOS); im.paste(s,(i*404,18)); label(im,f"{n} (1024^2 source)",(i*404,2))
im.save(f"{FIG}/sources.png",optimize=True)

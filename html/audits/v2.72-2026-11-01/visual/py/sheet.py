# contact sheet: crop the same box from several shots and tile them, labelled
import sys, os
from PIL import Image, ImageDraw
S='/tmp/claude-0/-home-user-Cartalith-GDT/9bb82d91-f6d5-50c4-b743-a54d60b8e9ff/scratchpad/audit-visual/shots/'
out=sys.argv[1]; box=tuple(map(int,sys.argv[2].split(','))); cols=int(sys.argv[3]); names=sys.argv[4:]
ims=[]
for n in names:
    im=Image.open(S+n+'.png').convert('RGB'); c=im.crop(box); d=ImageDraw.Draw(c); d.rectangle([0,0,len(n)*6+6,12],fill=(0,0,0)); d.text((3,1),n,fill=(255,255,0)); ims.append(c)
w,h=ims[0].size; rows=(len(ims)+cols-1)//cols
sheet=Image.new('RGB',(w*cols,h*rows),(40,40,40))
for i,c in enumerate(ims): sheet.paste(c,((i%cols)*w,(i//cols)*h))
sheet.save(out); print(out, sheet.size)

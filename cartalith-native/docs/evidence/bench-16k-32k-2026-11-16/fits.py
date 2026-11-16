# Power-law fit t = a * cells^b per stage over the CPU runs (2k, 4k, 8k, 16k), then the
# prediction at 32k = 2^31 cells... (32768 x 16384 = 536 870 912 cells).
import re,math,sys,glob
runs={'2k':(2048,'r2k_cpu'),'4k':(4096,'r4k_cpu'),'8k':(8192,'r8k_cpu'),'16k':(16384,'r16k_cpu')}
data={}
for k,(w,tag) in runs.items():
    vals={}
    for l in open(tag+'.log'):
        m=re.match(r'STAGE rep=(\d+)\s+(?:\d+|--)\s+(.+?)\s+([\d.]+) ms',l)
        if m: vals.setdefault(m.group(2).strip(),[]).append(float(m.group(3))/1000)
    data[k]=(w*(w//2),{n:sorted(v)[len(v)//2] for n,v in vals.items()})
names=['Tectonics','Volcanism & impacts','Erosion','Hydrology','Climate','terrain total','civilisation']
print('| stage | 2k s | 4k s | 8k s | 16k s | exponent b (t ~ cells^b) | ratio 16k/8k | predicted 32k s (from 4k..16k fit) |')
print('|---|---|---|---|---|---|---|---|')
for n in names:
    xs=[];ys=[]
    for k in ['4k','8k','16k']:
        c,v=data[k]; xs.append(math.log(c)); ys.append(math.log(v[n]))
    N=len(xs); mx=sum(xs)/N; my=sum(ys)/N
    b=sum((x-mx)*(y-my) for x,y in zip(xs,ys))/sum((x-mx)**2 for x in xs); a=my-b*mx
    p=math.exp(a+b*math.log(32768*16384))
    row=[data[k][1][n] for k in ['2k','4k','8k','16k']]
    print('| %s | %.2f | %.2f | %.2f | %.1f | %.2f | %.2f | %.0f |'%(n,*row,b,row[3]/row[2],p))

import sys,re,csv,statistics as S
d=sys.argv[1]; tag=sys.argv[2]
a=open(f'{d}/agg_{tag}.txt',encoding='utf8').read().split('\n')
def sect(name):
    i=[k for k,l in enumerate(a) if l.startswith('== '+name)][0]; return a[i:i+30]
s24=sect('8 largest'); s35=sect('all towns')[:16]
def grab(sec,pat,which=1):
    # nth match
    m=[re.search(pat,l) for l in sec if re.search(pat,l)]
    return m
quads=[re.search(r'quads ([\d.]+)',l).group(1) for l in s24 if l.strip().startswith('blocks outside core')]
housed=[re.search(r'housed p10 (\d+)',l).group(1) for l in s35 if 'housed p10' in l]
fs1=[re.search(r'kept >= 85 %: (\d+)/\d+ \(median ([\d.]+)',l).groups() for l in s35 if 'kept >= 85' in l]
need=[re.search(r'NEEDLES[^%]*?([\d.]+) % \((\d+)\)',l).group(1) for l in s35 if 'NEEDLES' in l]
print(tag,'quads24 off/on',quads,'housed p10 off/on',housed,'FS1 kept off/on',fs1,'needles off/on',need)
for l in a:
    if re.match(r'\s+(market_street|organic_core|river_front):',l): print(' ',l.strip()[:150])
r=list(csv.DictReader(open(f'{d}/{tag}/towns_384.csv')))
for v in ('off','proto'):
    T=[x for x in r if x['variant']==v and x['walled']=='1']
    sh=[int(x['wall_sharp_convex'])+int(x['wall_sharp_reflex']) for x in T]; lh=sorted(float(x['wall_len_over_hull']) for x in T)
    q=lambda x,f:sorted(x)[int(round((len(x)-1)*f))]
    print(' wall',v,'sharp med/p90/total',S.median(sh),q(sh,.9),sum(sh),'hull med/p90/max',round(S.median(lh),3),round(q(lh,.9),3),lh[-1])
mk=[x for x in r if x['variant']=='proto' and x['ptype']=='market_street']
print(' market phi per town',[x['phi'] for x in mk],'cap .35')

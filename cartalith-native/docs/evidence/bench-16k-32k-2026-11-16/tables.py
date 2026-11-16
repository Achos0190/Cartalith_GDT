import re,csv,sys
def stages(tag):
    d={}
    for l in open(tag+'.log'):
        m=re.match(r'STAGE rep=\d+\s+(?:\d+|--)\s+(.+?)\s+([\d.]+) ms',l)
        if m: d[m.group(1).strip()]=float(m.group(2))/1000
        m=re.match(r'TOTAL rep=\d+ ([\d.]+) ms',l)
        if m: d['TOTAL']=float(m.group(1))/1000
        m=re.match(r'DIGEST (.*)',l)
        if m: d['digest']=m.group(1)
    return d
def mem(tag):
    rows=list(csv.DictReader(open(tag+'.csv')))
    f=lambda k:[float(r.get(k) or 0) for r in rows]
    base=rows[0]
    return dict(ws=max(f('ws_mb')),commit=max(f('pagefile_peak_mb')),pf=max(f('pf_used_mb')),pf0=float(base.get('pf_used_mb') or 0),avail=min(f('avail_mb')),pin=max(f('pages_in_per_s')),avail0=float(base['avail_mb']))
names=['Tectonics','Volcanism & impacts','Erosion','Hydrology','Climate','terrain total','civilisation','TOTAL']
tags=sys.argv[1:]
S={t:stages(t) for t in tags}
print('| stage (s) | '+' | '.join(tags)+' |'); print('|---|'+'---|'*len(tags))
for n in names: print('| %s | '%n+' | '.join('%.1f'%S[t][n] for t in tags)+' |')
print(); print('| run | peak WS MB | peak commit MB | page file in use at start MB | at peak MB | min avail RAM MB | avail RAM at start MB | max pages-in/s |'); print('|---|---|---|---|---|---|---|---|')
for t in tags:
    m=mem(t); print('| %s | %.0f | %.0f | %.0f | %.0f | %.0f | %.0f | %.0f |'%(t,m['ws'],m['commit'],m['pf0'],m['pf'],m['avail'],m['avail0'],m['pin']))
for t in tags: print(t,S[t].get('digest'))

# Per-stage memory/paging from a sampler CSV and a bench log (aligned on unix_ms).
import sys,re,csv
tag=sys.argv[1]
rows=list(csv.DictReader(open(tag+'.csv')))
log=open(tag+'.log').read().splitlines()
stages=[]
for l in log:
    m=re.match(r'STAGE rep=(\d+)\s+(\d+|--) (.+?)\s+([\d.]+) ms.*unix_ms=(\d+)\.\.(\d+)',l)
    if m: stages.append((m.group(3).strip(),float(m.group(4))/1000,int(m.group(5)),int(m.group(6))))
    else:
        m=re.match(r'STAGE rep=(\d+)\s+-- (.+?)\s+([\d.]+) ms',l)
        if m: stages.append((m.group(2).strip(),float(m.group(3))/1000,None,None))
# civilisation window: from end of Climate to end of run
last=[s for s in stages if s[2]]
civ_start=last[-1][3] if last else None
end=int(rows[-1]['unix_ms'])
out=[]
for n,sec,a,b in stages:
    if n=='civilisation': a,b=civ_start,end
    if n=='terrain total': continue
    if a is None: continue
    w=[r for r in rows if a<=int(r['unix_ms'])<=b]
    if not w: continue
    g=lambda k:[float(r.get(k) or 0) for r in w]
    out.append((n,sec,max(g('ws_mb')),max(g('private_mb')),max(g('pf_used_mb')),max(g('faults_per_s')),max(g('pages_in_per_s')),sum(g('pages_in_per_s'))*1.0,min(g('avail_mb'))))
print('| stage | wall s | peak WS MB | peak private MB | page file used MB (max) | max faults/s | max pages-in/s | min avail RAM MB |')
print('|---|---|---|---|---|---|---|---|')
for o in out: print('| %s | %.1f | %d | %d | %d | %d | %d | %d |'%(o[0],o[1],o[2],o[3],o[4],o[5],o[6],o[8]))
print('overall peak WS %d MB, peak commit(pagefile bytes peak) %d MB, pf used max %d, pf%% peak %s'%(max(float(r['ws_mb']) for r in rows),max(float(r['pagefile_peak_mb']) for r in rows),max(float(r.get('pf_used_mb') or 0) for r in rows),max(float(r['pf_peak_pct']) for r in rows)))

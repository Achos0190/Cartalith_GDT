# Per-stage memory and paging table from a sampler CSV aligned to a bench log on unix_ms.
# A stage is labelled "paging" when the machine's mean pages-input/s over it exceeds 1000
# or available RAM fell below 1 GB at any sample in it; otherwise "in RAM".
import sys,re,csv
tag=sys.argv[1]
R=list(csv.DictReader(open(tag+'.csv'))); L=open(tag+'.log').read().splitlines()
st=[]
for l in L:
    m=re.match(r'STAGE rep=\d+\s+(?:\d+)\s+(.+?)\s+([\d.]+) ms.*unix_ms=(\d+)\.\.(\d+)',l)
    if m: st.append([m.group(1).strip(),float(m.group(2))/1000,int(m.group(3)),int(m.group(4))])
st.append(['civilisation',[float(re.match(r'STAGE rep=\d+\s+-- civilisation\s+([\d.]+)',l).group(1))/1000 for l in L if re.match(r'STAGE rep=\d+\s+-- civilisation',l)][0],st[-1][3],int(R[-1]['unix_ms'])])
pf0=float(R[0].get('pf_used_mb') or 0)
print('| stage | wall s | peak WS MB | commit peak to date MB | min avail RAM MB | page file in use, max MB (start %.0f) | mean pages-in/s | max pages-in/s | state |'%pf0)
print('|---|---|---|---|---|---|---|---|---|')
for n,sec,a,b in st:
    w=[r for r in R if a<=int(r['unix_ms'])<=b]
    if not w: continue
    g=lambda k:[float(r.get(k) or 0) for r in w]
    mp=sum(g('pages_in_per_s'))/len(w); av=min(g('avail_mb'))
    s='paging' if (mp>1000 or av<1000) else 'in RAM'
    print('| %s | %.1f | %.0f | %.0f | %.0f | %.0f | %.0f | %.0f | %s |'%(n,sec,max(g('ws_mb')),max(g('pagefile_peak_mb')),av,max(g('pf_used_mb')),mp,max(g('pages_in_per_s')),s))

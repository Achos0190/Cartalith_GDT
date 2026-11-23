"""Ruling HS gate: every town with a crossing (BRIDGE or FERRY) with civ.bank_sites on before keeps one after, and
its housing. python -I crossings.py BEFORE AFTER"""
import re, sys
def rows(f):
    seed=None; out={}
    for l in open(f, encoding="utf-8", errors="replace"):
        m=re.match(r"TOWN seed (\d+) at (\d+)", l)
        if m: seed=m.group(1)+"@"+m.group(2)
        m=re.match(r"TOWN     on  #(\d+) (\w+) .*?: (.*)", l)
        if m:
            kind="bridge" if "BRIDGE" in m.group(3) else ("ferry" if "FERRY" in m.group(3) else "-")
            h=re.search(r"housed (\d+)%", l)
            out[(seed,int(m.group(1)))]=(kind, int(h.group(1)) if h else None)
    return out
b,a=rows(sys.argv[1]),rows(sys.argv[2])
lost=0; n=0
for k,(kind,h) in sorted(b.items()):
    if kind=="-": continue
    n+=1
    ak,ah=a.get(k,("not listed",None))
    if ak not in ("bridge","ferry"):
        lost+=1
    if ak!=kind or (ah is not None and h is not None and ah<h-5):
        print(k, kind, h, "->", ak, ah)
gain=[k for k,(kind,_) in a.items() if kind!="-" and b.get(k,("-",0))[0]=="-"]
print(f"crossing towns before {n}, lost {lost}; gained a crossing {len(gain)} {gain}")

"""Ruling HS: per settlement, whether the town laid out by two builds is the same (market, buildings, edges).
python -I cmp_dumps.py BEFORE.jsonl AFTER.jsonl"""
import json, sys
def load(f):
    out={}
    for l in open(f):
        r=json.loads(l)
        if "i" in r and r.get("d"): d=r["d"]; out[r["i"]]=(d["market"], d["buildings"], d["edges"], d.get("pop"))
    return out
a,b=load(sys.argv[1]),load(sys.argv[2])
diff=[i for i in a if i in b and a[i]!=b[i]]
print(f"{len(a)} towns before, {len(b)} after; laid out differently: {len(diff)} {diff[:40]}")

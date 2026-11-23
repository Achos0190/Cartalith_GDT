"""fate.py BEFORE MID AFTER: per town (switch on, bank_sites on rows of _towncensus_probe --rows), its crossing (bridge,
ferry or nothing) and housing in the three builds; prints every town whose crossing or housing (by 5 points) differs
from MID in AFTER, and the counts. Bars: no town loses its crossing against BEFORE or MID; housing per town."""
import re, sys
def rows(f):
    seed = None; out = {}
    for l in open(f, encoding="utf-8", errors="replace"):
        m = re.match(r"TOWN seed (\d+) at (\d+)", l)
        if m: seed = m.group(1)
        m = re.match(r"TOWN     on  #(\d+) (\w+) \((\d+),(\d+)\) pop 0:(.*)housed (\d+)% of (\d+)", l)
        if m:
            note = m.group(5)
            kind = "bridge" if re.search(r"BRIDGE \d+", note) else ("ferry" if re.search(r"FERRY \d+", note) else "-")
            out[(seed, int(m.group(1)))] = (kind, int(m.group(6)), int(m.group(7)))
    return out
b, m, a = (rows(f) for f in sys.argv[1:4])
keys = sorted(set(b) | set(m) | set(a), key=lambda k: (k[0], k[1]))
lost = {"mid": 0, "after": 0}
for k in keys:
    fb, fm, fa = b.get(k, ("?", None, None)), m.get(k, ("?", None, None)), a.get(k, ("?", None, None))
    for name, f in (("mid", fm), ("after", fa)):
        if fb[0] in ("bridge", "ferry") and f[0] not in ("bridge", "ferry", "?"): lost[name] += 1
    if fm[0] != fa[0] or (fm[1] is not None and fa[1] is not None and abs(fm[1] - fa[1]) >= 5):
        print(f"{k[0]} #{k[1]}: before {fb[0]} {fb[1]}% | mid {fm[0]} {fm[1]}% | after {fa[0]} {fa[1]}% (target {fa[2] or fm[2]})")
print("rows before/mid/after", len(b), len(m), len(a), "| crossings lost against before: mid", lost["mid"], "after", lost["after"])
print("crossings (bridge, ferry): before", sum(1 for v in b.values() if v[0] == "bridge"), sum(1 for v in b.values() if v[0] == "ferry"),
      "mid", sum(1 for v in m.values() if v[0] == "bridge"), sum(1 for v in m.values() if v[0] == "ferry"),
      "after", sum(1 for v in a.values() if v[0] == "bridge"), sum(1 for v in a.values() if v[0] == "ferry"))

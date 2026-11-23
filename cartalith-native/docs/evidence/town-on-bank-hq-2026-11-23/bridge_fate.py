"""Ruling HQ: what became of each town that had a bridge in Ruling HN's build: still a bridge, a ferry, or nothing.
  python -I bridge_fate.py HN_CENSUS HQ_CENSUS
HN's dll reports no span over the water, so its figure is the census chain, dry approaches included. A town with
neither a bridge nor a ferry in HQ and no row of its own is "listed in neither": it houses at least four fifths of
its switch-off figure (the census prints a row only for a town with a note)."""
import re, sys
def rows(f):
    seed = None; out = {}
    for l in open(f, encoding="utf-8", errors="replace"):
        m = re.match(r"TOWN seed (\d+) at (\d+)", l)
        if m: seed = m.group(1) + "@" + m.group(2)
        m = re.match(r"TOWN     on  #(\d+) (\w+) \((\d+),(\d+)\) pop 0:(.*)housed (\d+)% of", l)
        if m:
            note = m.group(5)
            b = re.search(r"BRIDGE (\d+) \((\d+) m over the water\)", note)
            fe = re.search(r"FERRY (\d+) \((\d+) m\)", note)
            out[(seed, int(m.group(1)))] = (m.group(2), int(m.group(6)), b and int(b.group(2)), fe and int(fe.group(2)))
    return out
hn, hq = rows(sys.argv[1]), rows(sys.argv[2])
fate = {"bridge": [], "ferry": [], "nothing": []}
for k, (kind, housed, br, fe) in sorted(hn.items()):
    if br is None: continue
    a = hq.get(k)
    if a is None: f, h = "nothing", "listed in neither (at least 4/5 of off)"
    else:
        f = "bridge" if a[2] is not None else ("ferry" if a[3] is not None else "nothing"); h = "%d %%" % a[1]
    fate[f].append((k, kind, br, housed, h))
for f, L in fate.items():
    print("%s: %d" % (f, len(L)))
    for k, kind, br, housed, h in L:
        print("   %s #%d %s: HN bridge %d m (chain), housed %d %% -> HQ %s" % (k[0], k[1], kind, br, housed, h))

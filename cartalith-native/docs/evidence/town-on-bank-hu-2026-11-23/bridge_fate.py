"""Ruling HU: what became of each town that had a bridge in Ruling HN's build, in Ruling HQ's build (BEFORE) and in
this ruling's (AFTER): a bridge, a ferry, or nothing (the crossing lost), with its housing in each.
  python -I bridge_fate.py HN_CENSUS HQ_CENSUS HU_CENSUS
HN's census is Ruling HQ's `census_before_*.txt`. A town with neither a bridge nor a ferry and no row of its own is
"listed in neither": it houses at least four fifths of its switch-off figure (the census prints a row only for a town
with a note, and a bridge and a ferry are notes). Bars (Ruling HU): no town loses its crossing; every ferry town
houses at least its HN figure less 5 points."""
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
            c = re.search(r"\(chain (\d+) m\)", note)
            fe = re.search(r"FERRY (\d+) \((\d+) m\)", note)
            out[(seed, int(m.group(1)))] = (m.group(2), int(m.group(6)), b and int(b.group(2)), fe and int(fe.group(2)))
    return out
def fate(a):
    if a is None: return "nothing", None
    return ("bridge" if a[2] is not None else ("ferry" if a[3] is not None else "nothing")), a[1]
hn, hq, hu = rows(sys.argv[1]), rows(sys.argv[2]), rows(sys.argv[3])
count = {"hq": {"bridge": 0, "ferry": 0, "nothing": 0}, "hu": {"bridge": 0, "ferry": 0, "nothing": 0}}
bad = 0
for k, (kind, housed, br, fe) in sorted(hn.items()):
    if br is None: continue
    fq, hq_h = fate(hq.get(k)); fu, hu_h = fate(hu.get(k))
    count["hq"][fq] += 1; count["hu"][fu] += 1
    flag = ""
    if fu == "nothing": flag = "  << LOST"
    elif fu == "ferry" and hu_h is not None and hu_h < housed - 5: flag = "  << HOUSING %d < HN %d - 5" % (hu_h, housed)
    if flag: bad += 1
    h = lambda x: "listed in neither" if x is None else "%d %%" % x
    print("%s #%d %s: HN bridge (%d m over the water), housed %d %% | HQ %s %s | HU %s %s%s" % (k[0], k[1], kind, br, housed, fq, h(hq_h), fu, h(hu_h), flag))
print("HQ:", count["hq"]); print("HU:", count["hu"]); print("towns off a bar:", bad)
# Towns whose crossing changed kind between HQ and HU, among every town the HU census lists (not only HN's bridges).
ch = [(k, fate(hq.get(k))[0], fate(hu.get(k))[0]) for k in sorted(set(hq) | set(hu)) if fate(hq.get(k))[0] != fate(hu.get(k))[0]]
print("crossing kind changed HQ -> HU: %d" % len(ch))
for k, a, b in ch: print("   %s #%d: %s -> %s" % (k[0], k[1], a, b))

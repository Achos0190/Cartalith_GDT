"""Ruling HU: the bar table from census runs, one row per world and file.
  python -I bars.py CENSUS [CENSUS ...]"""
import re, sys
for f in sys.argv[1:]:
    seed = None; off = {}
    for l in open(f, encoding="utf-8", errors="replace"):
        m = re.match(r"TOWN seed (\d+) at (\d+)", l)
        if m: seed = m.group(1) + "@" + m.group(2)
        m = re.match(r"TOWN   (on |off): .*housed p10 ([\d.]+) %.*over drawn river: buildings (\d+) of \d+, wall runs (\d+) \(gated \d+\), street edges (\d+) of \d+ \(bridge edges \d+ in (\d+) towns, longest bridge (\d+) m.*over drawn lake: b (\d+) w (\d+) s (\d+) \| over drawn sea: b (\d+) w (\d+) s (\d+) \| other bank x1 (\d+)/\d+ x4 (\d+)/\d+ x16 (\d+)/\d+ x64 (\d+)/\d+.*ferries (\d+) in (\d+) towns \(longest (\d+) m\) \| no street near the market (\d+) \| over-housed \(over 125 %\) (\d+) \| housed under a quarter of off (\d+)", l)
        if not m: continue
        g = m.groups()
        if g[0] == "off": off[seed] = g[1]; continue
        print("%-12s %-22s p10 off %s on %s | river b%s w%s s%s | lake %s/%s/%s sea %s/%s/%s | other bank %s %s %s %s | bridge towns %s longest %s m | ferries %s in %s towns (longest %s m) | no street near mkt %s | over-housed %s | quartered %s" % (
            seed, f.split("/")[-1].split("\\")[-1], off.get(seed), g[1], g[2], g[3], g[4], g[7], g[8], g[9], g[10], g[11], g[12], g[13], g[14], g[15], g[16], g[5], g[6], g[17], g[18], g[19], g[20], g[21], g[22]))

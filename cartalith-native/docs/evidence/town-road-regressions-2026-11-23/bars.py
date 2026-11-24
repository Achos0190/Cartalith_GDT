"""The road-attachment bar table, before (e74e7312) and after, from `_roadattach_probe.gd`'s summary lines.

  python -I bars.py DIR    (DIR holds road_<head|fix>_<world>.txt)"""
import pathlib, re, sys

d = pathlib.Path(sys.argv[1])
F = [("street median", r"street median (\S+)"), ("street max m", r"street median \S+ max (\S+) m"), ("along median m", r"along median (\S+) m"),
     ("end-street x16 px", r"drawn end to street x16 max (\S+) px"), ("end-street x64 px", r"drawn end to street x64 max (\S+) px"),
     ("named", r"ends the engine names (\d+)"), ("at own bank", r"at its own bank or with no market: (\d+)"), ("market no street", r"market with no street: (\d+)"),
     ("made crossings", r"made by the attach (\d+)"), ("over 4 px", r"over 4.0 px: (\d+)"), ("empty rings", r"rings with under 5 inside (\d+)")]
print("world | state | ends | " + " | ".join(n for n, _ in F))
for w in ("384a", "384b", "2048", "owner", "4096"):
    for tag, name in (("head", "before (e74e7312, on)"), ("fix", "after (on)")):
        p = d / ("road_%s_%s.txt" % (tag, w))
        if not p.exists():
            continue
        seed = None
        for line in p.read_text(errors="replace").splitlines():
            m = re.match(r"ROAD seed (\d+) at (\S+),", line)
            if m:
                seed = "%s %s" % m.groups()
            if line.startswith("ROAD   on "):
                vals = [(re.search(rx, line) or [None, "-"])[1] for _, rx in F]
                ends = re.search(r"road ends at a town (\d+)", line)[1]
                print("%s | %s | %s | %s" % (seed, name, ends, " | ".join(vals)))

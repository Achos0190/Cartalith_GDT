"""Classifies the over-bar frames of a `_zoomcost_probe` run whose tree carried the scratch trace (TRC_FR per frame, TRC_CH per overlay chunk).
Usage: python -I analyze_trace.py trace.log. The trace lines came from scratch edits (not committed): a print in `_chunk_done`, one in
`_zoomcost_probe.gd::_frame`, and per-step timers in `_settlement_plan_step`, `_zoom_at` and `_refresh_layers`."""
import re, sys, collections
fr, ch, seq = {}, collections.defaultdict(list), []
for l in open(sys.argv[1], encoding="utf8", errors="replace"):
    m = re.match(r"TRC_FR f=(\d+) ([\d.]+)", l)
    if m: fr[int(m[1])] = float(m[2]); seq.append(int(m[1]))
    m = re.match(r"TRC_CH f=(\d+) (\w+) (\d+) (\d+)", l)
    if m: ch[int(m[1])].append((m[2], int(m[3]), int(m[4])))
cat = collections.Counter()
for f in seq:
    ms = fr[f]
    if ms <= 16.7 or ms > 5000: continue   # > 5 s is the world build in the first frame
    cs = ch.get(f, []); tot = sum(c[2] for c in cs) / 1000; st = sum(c[2] for c in cs if c[0] == "settlements") / 1000
    cat["settlement chunks > 8 ms" if st > 8 else "chunks >= 40% of frame" if tot >= ms * .4 else "no chunk drawn" if not cs else "chunks < 40% of frame"] += 1
print(dict(cat))

"""Side-by-side stretches between lane pairs in a census dump.
python find_thread.py <dump.json> [max_sep_cells=8]
For every sample (every 0.5 map cell) of lane i, the nearest point of lane j (j != i); a stretch is a run of
samples with 0 < sep <= max_sep and the two directions within 20 degrees. Prints the longest stretches: lanes,
their end ports, length km, separation min/median/max in map cells, centre."""
import sys, json, math
import numpy as np

d = json.load(open(sys.argv[1]))
mx = float(sys.argv[2]) if len(sys.argv) > 2 else 8.0
km_cell = d["km"] / d["gw"]
lanes = [np.array(l, float) for l in d["lanes"]]

def samples(l):
    out = []
    for a, b in zip(l[:-1], l[1:]):
        if abs(a[0] - b[0]) > d["gw"] / 2:
            out.append(None); continue
        L = math.hypot(*(b - a)); n = max(1, int(math.ceil(L / 0.5)))
        u = (b - a) / max(L, 1e-9)
        for k in range(n):
            out.append((a + (b - a) * k / n, u))
    return out

def nearest(p, l):
    a, b = l[:-1], l[1:]
    v = b - a; l2 = (v * v).sum(1)
    t = np.clip(((p - a) * v).sum(1) / np.maximum(l2, 1e-12), 0, 1)
    q = a + v * t[:, None]
    dd = np.hypot(*(q - p).T)
    ok = np.abs(v[:, 0]) <= d["gw"] / 2
    dd[~ok] = np.inf
    k = int(np.argmin(dd))
    return dd[k], v[k] / max(math.sqrt(l2[k]), 1e-9)

rows = []
cos20 = math.cos(math.radians(20))
for i, li in enumerate(lanes):
    S = samples(li)
    for j, lj in enumerate(lanes):
        if j == i: continue
        run = []
        def flush():
            if len(run) >= 4:
                seps = [r[0] for r in run]
                c = run[len(run) // 2][1]
                rows.append((len(run) * 0.5 * km_cell, i, j, min(seps), float(np.median(seps)), max(seps), c[0], c[1]))
            run.clear()
        for s in S:
            if s is None: flush(); continue
            p, u = s
            dd, w = nearest(p, lj)
            if 1e-6 < dd <= mx and abs(u @ w) >= cos20:
                run.append((dd, p))
            else:
                flush()
        flush()
rows.sort(reverse=True)
print(f"lanes={len(lanes)} km_cell={km_cell:.3f}")
for r in rows[:15]:
    km, i, j, a, m, b, x, y = r
    print(f"lane {i} beside {j}: {km:6.1f} km  sep min {a:.2f} med {m:.2f} max {b:.2f} cells  centre ({x:.0f},{y:.0f})  ends i {lanes[i][0].round().tolist()}->{lanes[i][-1].round().tolist()}  j {lanes[j][0].round().tolist()}->{lanes[j][-1].round().tolist()}")

import sys, subprocess, statistics as st
B, A = sys.argv[1], sys.argv[2]
out = subprocess.run(["python", "fg_ladder_stats.py", B, A, "2"], capture_output=True, text=True).stdout
rows = {}
for l in out.splitlines():
    p = l.split()
    if p and p[0] in ("x4", "x16", "x64") and "_800km_" in p[1]:
        rows.setdefault(p[0], []).append((float(p[3]), float(p[5])))
for r in ("x4", "x16", "x64"):
    v = rows.get(r, [])
    if v:
        mb, ma = st.median(x[0] for x in v), st.median(x[1] for x in v)
        print(f"800km {r}: before {mb:.2f} after {ma:.2f} ratio {ma / mb:.3f} n={len(v)}")

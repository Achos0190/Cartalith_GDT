"""Ruling FG bar B6: the Ruling CI zoom ladder's mean luminance gradient per crop, before vs after.
usage: python fg_ladder_stats.py <before_dir> <after_dir> [law]
Per frame: mean |grad luma| over the whole 950x858 frame (the Ruling CI ladder's measure).
Prints per-crop values and the median over crops at each rung, and after/before."""
import sys, glob, os
import numpy as np
from PIL import Image

def g(p):
    a = np.asarray(Image.open(p).convert("RGB"), dtype=np.float64)
    l = 0.2126 * a[..., 0] + 0.7152 * a[..., 1] + 0.0722 * a[..., 2]
    gx = np.abs(np.diff(l, axis=1))[:-1, :]
    gy = np.abs(np.diff(l, axis=0))[:, :-1]
    return float(np.hypot(gx, gy).mean())

B, A = sys.argv[1], sys.argv[2]
law = sys.argv[3] if len(sys.argv) > 3 else "2"
rows = {}
for p in sorted(glob.glob(os.path.join(B, f"*_law{law}.png"))):
    name = os.path.basename(p)
    q = os.path.join(A, name)
    if not os.path.exists(q):
        continue
    stem = name[: -len(f"_law{law}.png")]
    crop, rung = stem.rsplit("_", 1)
    rows.setdefault(rung, []).append((crop, g(p), g(q)))
for rung in ["x4", "x16", "x64"]:
    r = rows.get(rung, [])
    if not r:
        continue
    for crop, b, a in r:
        print(f"{rung:4s} {crop:45s} before {b:6.2f} after {a:6.2f}  x{a / max(b, 1e-9):.2f}")
    mb, ma = np.median([x[1] for x in r]), np.median([x[2] for x in r])
    print(f"MEDIAN {rung}: before {mb:.2f} after {ma:.2f} ratio {ma / mb:.3f} (n={len(r)})")

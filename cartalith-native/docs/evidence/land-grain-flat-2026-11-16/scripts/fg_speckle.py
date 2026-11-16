"""Ruling FG leftover bar S1 (speckle on flat desert at far zoom).
usage: python fg_speckle.py <before_dir> <after_dir> [frame-suffix ...]
Same capture format as fg_bars.py (2560x1440, map area x 420..2250, y 160..1350, 32 px blocks).
Blocks are chosen on the BEFORE frame, so before and after measure the same ground:
  land      <= 10 % water-blue pixels (b > r + 25)
  desert    mean(r) - mean(b) >= 30 (sand-coloured; forest and snow and rock fail it)
  flat      mean |grad luma| of the block <= the median of the land blocks' (the flattest half)
speckle = mean over selected blocks of the luma standard deviation inside the block after
removing the block's best-fit plane (a plane is lighting; what is left is mottle and grain).
Prints before, after and the ratio per frame."""
import sys, os, glob
import numpy as np
from PIL import Image

B, A = sys.argv[1], sys.argv[2]
X0, X1, Y0, Y1, BL = 420, 2250, 160, 1350, 32


def load(p):
    return np.asarray(Image.open(p).convert("RGB"), dtype=np.float64)[Y0:Y1, X0:X1]


def luma(a):
    return 0.2126 * a[..., 0] + 0.7152 * a[..., 1] + 0.0722 * a[..., 2]


yy, xx = np.mgrid[0:BL, 0:BL]
M = np.stack([xx.ravel(), yy.ravel(), np.ones(BL * BL)], axis=1)
PINV = np.linalg.pinv(M)


def resid_std(l):
    v = l.ravel()
    c = PINV @ v
    return float((v - M @ c).std())


def blocks(a):
    l = luma(a)
    gx = np.abs(np.diff(l, axis=1))[:-1, :]
    gy = np.abs(np.diff(l, axis=0))[:, :-1]
    g = np.hypot(gx, gy)
    h, w = l.shape
    out = []
    for y in range(0, h - BL, BL):
        for x in range(0, w - BL, BL):
            blk = a[y:y + BL, x:x + BL]
            if (blk[..., 2] > blk[..., 0] + 25).mean() > 0.1:
                continue
            out.append((y, x, g[y:y + BL - 1, x:x + BL - 1].mean(), blk[..., 0].mean() - blk[..., 2].mean()))
    return out


def measure(bp, ap):
    a_b, a_a = load(bp), load(ap)
    bl = blocks(a_b)
    if not bl:
        return None
    med = np.median([b[2] for b in bl])
    sel = [b for b in bl if b[2] <= med and b[3] >= 30]
    if len(sel) < 5:
        return (len(sel), np.nan, np.nan)
    lb, la = luma(a_b), luma(a_a)
    sb = np.mean([resid_std(lb[y:y + BL, x:x + BL]) for y, x, _, _ in sel])
    sa = np.mean([resid_std(la[y:y + BL, x:x + BL]) for y, x, _, _ in sel])
    return (len(sel), sb, sa)


print(f"{'frame':26s} {'blocks':>6s} {'before':>7s} {'after':>7s} {'ratio':>6s}")
for p in sorted(glob.glob(os.path.join(B, "*_on_*.png"))):
    n = os.path.basename(p)
    q = os.path.join(A, n)
    if not os.path.exists(q):
        continue
    r = measure(p, q)
    if r is None:
        continue
    k, sb, sa = r
    print(f"{n.replace('_on_', '_').replace('.png', ''):26s} {k:6d} {sb:7.3f} {sa:7.3f} {sa / sb if sb == sb and sb > 0 else float('nan'):6.2f}")

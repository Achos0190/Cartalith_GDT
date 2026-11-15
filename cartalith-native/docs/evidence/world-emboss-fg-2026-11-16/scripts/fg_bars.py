"""Ruling FG bars B1-B5, B7 from one capture folder of `_fg_probe.gd` (<tag>_<on|off>_<shot>.png).

Same definitions as `WORLD_EMBOSS_FINDINGS_2026-11-15.md` section 3 (`emboss_measure.py`): the map area
x 420..2250, y 160..1350 of the 2560x1440 capture; land = off-frame pixels that are not water-blue
(b > r + 25); 32 px blocks with <= 10 % water.
  added   mean |luma(on) - luma(off)| on land
  grad    mean |grad luma| on land, on / off
  bind_r  Pearson r across land blocks: std of the added luma vs the off frame's mean gradient
  flat/steep (B3)  mean |added| over the flattest decile of land blocks (by the off frame's gradient,
          an input: the grid relief) / the steepest decile's
  snow_pp (B7)  snow fraction on minus off, in percentage points, over land pixels that are not snow
          in the off frame (snow = all channels >= 225 and max-min <= 30)
usage: python fg_bars.py <dir>"""
import sys, os, glob
import numpy as np
from PIL import Image

D = sys.argv[1]
X0, X1, Y0, Y1, B = 420, 2250, 160, 1350, 32


def load(p):
    return np.asarray(Image.open(p).convert("RGB"), dtype=np.float64)[Y0:Y1, X0:X1]


def luma(a):
    return 0.2126 * a[..., 0] + 0.7152 * a[..., 1] + 0.0722 * a[..., 2]


def gmag(l):
    gx = np.abs(np.diff(l, axis=1))[:-1, :]
    gy = np.abs(np.diff(l, axis=0))[:, :-1]
    return np.hypot(gx, gy)


def snow(a):
    return (a.min(axis=2) >= 225) & (a.max(axis=2) - a.min(axis=2) <= 30)


def measure(on_p, off_p):
    a_on, a_off = load(on_p), load(off_p)
    water = a_off[..., 2] > a_off[..., 0] + 25
    land = ~water
    l_on, l_off = luma(a_on), luma(a_off)
    d = l_on - l_off
    g_on, g_off = gmag(l_on), gmag(l_off)
    h, w = d.shape
    s_add, s_rel, m_add = [], [], []
    for y in range(0, h - B, B):
        for x in range(0, w - B, B):
            if water[y:y + B, x:x + B].mean() > 0.1:
                continue
            s_add.append(d[y:y + B, x:x + B].std())
            m_add.append(np.abs(d[y:y + B, x:x + B]).mean())
            s_rel.append(g_off[y:y + B - 1, x:x + B - 1].mean())
    lm = land[:-1, :-1]
    out = dict(blocks=len(s_add), added=np.abs(d)[land].mean() if land.any() else np.nan,
               grad_on=g_on[lm].mean() if lm.any() else np.nan, grad_off=g_off[lm].mean() if lm.any() else np.nan,
               bind=np.nan, flat_steep=np.nan, snow_pp=np.nan)
    if len(s_add) >= 8:
        s_add, s_rel, m_add = np.array(s_add), np.array(s_rel), np.array(m_add)
        if s_add.std() > 0:
            out["bind"] = float(np.corrcoef(s_add, s_rel)[0, 1])
        o = np.argsort(s_rel)
        k = max(1, len(o) // 10)
        st = m_add[o[-k:]].mean()
        out["flat_steep"] = m_add[o[:k]].mean() / st if st > 0 else np.nan
    sn_off = snow(a_off)
    sel = land & ~sn_off
    if sel.any():
        out["snow_pp"] = 100.0 * snow(a_on)[sel].mean()
    return out


rows = []
for on in sorted(glob.glob(os.path.join(D, "*_on_*.png"))):
    off = on.replace("_on_", "_off_")
    if os.path.exists(off):
        rows.append((os.path.basename(on).replace("_on_", "_").replace(".png", ""), measure(on, off)))
print(f"{'frame':24s} {'blocks':>6s} {'added':>7s} {'grad_on':>8s} {'grad_off':>8s} {'bind_r':>7s} {'flat/steep':>10s} {'snow_pp':>8s}")
for t, m in rows:
    print(f"{t:24s} {m['blocks']:6d} {m['added']:7.2f} {m['grad_on']:8.2f} {m['grad_off']:8.2f} {m['bind']:7.2f} {m['flat_steep']:10.2f} {m['snow_pp']:8.2f}")

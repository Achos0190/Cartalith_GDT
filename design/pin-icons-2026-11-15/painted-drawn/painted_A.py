"""First-look ("A": dark disc, bronze frame) versions of the 38 painted pins
that so far exist only in the landscape/gold look (owner, 2026-11-15: "make
this for the rest as well", after approving temple_A).

Body: the per-pixel median of the owner's 20 first-look pins (107 x 128), the
same body temple_A uses, so outline, frame, alpha and tip are the set's own.
Picture: painted procedurally with the temple's techniques (painted_temple.py):
form by gradients lit from the upper left, value-noise texture, dark contours,
a grain-plus-median pass. Each subject is a single miniature on a small ground
patch over the vignetted dark disc.

Subjects follow the owner's landscape versions and the owner's glyph rulings:
ancient_road = road under a ruined arch; portage = boat on a cart;
caravan_station = wagon under a waystation roof with a lantern;
historic_crossing = bridge with its middle arch fallen; fort = timber stockade
(never the castle).

Coordinates in the pin functions are pin pixels from the disc centre
(51.5, 54.5), x right, y down; the disc edge is r = 42, subjects stay inside
r = 38 except ground, water and bridges, which run to the edge as the owner's do.

Must never write outside this folder or touch the repository.
"""
import math, os, sys, shutil
import numpy as np
from PIL import Image, ImageDraw, ImageFilter
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "painted_temple"))
import painted_temple as T
from painted_temple import Paint, lerp, SS

OX, OY = 51.5, 54.5
INK = (26, 20, 15)
# material = (light, mid, shade, texture strength). Sampled by eye from the
# owner's first-look pins (castle stone, village roofs and plaster, harbour
# wood and water, mountain rock and snow, desert sand).
MAT = dict(
    stone=((238, 224, 198), (194, 172, 140), (104, 84, 64), 0.30),
    rock=((196, 190, 182), (134, 128, 122), (64, 60, 58), 0.34),
    rockw=((196, 176, 148), (140, 120, 98), (70, 58, 48), 0.34),
    sand=((232, 178, 116), (184, 124, 72), (104, 62, 38), 0.28),
    wood=((196, 142, 86), (140, 94, 54), (74, 46, 28), 0.26),
    woodd=((132, 92, 58), (92, 62, 38), (46, 30, 20), 0.26),
    shingle=((150, 104, 70), (104, 68, 44), (54, 34, 24), 0.30),
    roof=((222, 116, 72), (170, 76, 46), (98, 42, 28), 0.26),
    slate=((128, 158, 196), (80, 108, 154), (40, 58, 96), 0.22),
    thatch=((220, 186, 116), (170, 134, 72), (100, 74, 40), 0.30),
    plaster=((242, 232, 210), (206, 192, 166), (128, 112, 92), 0.18),
    canvas=((244, 236, 216), (206, 194, 168), (126, 112, 92), 0.18),
    hide=((176, 128, 84), (128, 88, 54), (70, 46, 30), 0.24),
    blue=((96, 140, 208), (56, 94, 166), (28, 50, 102), 0.16),
    red=((220, 90, 58), (170, 54, 38), (96, 28, 22), 0.16),
    water=((120, 168, 212), (62, 112, 172), (30, 64, 112), 0.14),
    snow=((252, 252, 252), (214, 224, 238), (150, 168, 198), 0.10),
    ice=((222, 240, 248), (150, 198, 228), (80, 128, 176), 0.16),
    grass=((134, 156, 66), (86, 114, 46), (44, 62, 26), 0.34),
    pine=((104, 140, 62), (62, 98, 44), (30, 54, 28), 0.34),
    metal=((224, 224, 230), (146, 148, 156), (64, 66, 74), 0.14),
    gold=((252, 218, 116), (216, 162, 62), (130, 88, 30), 0.12),
    dirt=((208, 180, 134), (162, 132, 94), (96, 74, 52), 0.30),
    smoke=((170, 166, 162), (120, 116, 114), (70, 68, 68), 0.20),
    foam=((255, 251, 238), (236, 224, 198), (176, 154, 122), 0.06),
)
DARK = (22, 17, 14)


def bez(p0, p1, p2, n=16):
    return [((1 - t) ** 2 * p0[0] + 2 * (1 - t) * t * p1[0] + t * t * p2[0],
             (1 - t) ** 2 * p0[1] + 2 * (1 - t) * t * p1[1] + t * t * p2[1]) for t in np.linspace(0, 1, n)]


class Q(Paint):
    """The temple's canvas plus materials: every shape is laid as contour,
    then a three-stop colour ramp driven by a light-direction tone, times
    texture. All coordinates are relative to the disc centre."""

    def __init__(s, seed):
        super().__init__(107, 128, seed)
        s.n1 = s.fbm(1.2, 1); s.n3 = s.fbm(3, 2); s.n5 = s.fbm(5, 3)
        s.r = np.hypot(s.X - OX, s.Y - OY)
        s.put(np.ones_like(s.X), lerp((30, 29, 27), (12, 12, 11), np.clip(s.r / 42, 0, 1) ** 2))   # the vignetted dark disc
        s.disc = (s.r < 41.3).astype(np.float32)

    # ---- masks -------------------------------------------------------------
    def mp(s, pts): return s.poly([(x + OX, y + OY) for x, y in pts])
    def mr(s, x0, y0, x1, y1): return s.mp([(x0, y0), (x1, y0), (x1, y1), (x0, y1)])
    def me(s, cx, cy, rx, ry): return s.ell(cx + OX, cy + OY, rx, ry)
    def ml(s, pts, w):
        m = Image.new('L', (s.w * SS, s.h * SS), 0); d = ImageDraw.Draw(m)
        q = [((x + OX) * SS, (y + OY) * SS) for x, y in pts]
        d.line(q, fill=255, width=max(1, int(round(w * SS))), joint='curve')
        for x, y in (q[0], q[-1]): d.ellipse([x - w * SS / 2, y - w * SS / 2, x + w * SS / 2, y + w * SS / 2], fill=255)
        return np.asarray(m, np.float32) / 255
    def march(s, x, hw, top, bottom):
        """A round-headed opening."""
        return np.maximum(s.me(x, top + hw, hw, hw), s.mr(x - hw, top + hw, x + hw, bottom))

    # ---- shaded fill ---------------------------------------------------------
    def col(s, mat, t, tex=1.0):
        L, M, D, amp = MAT[mat]; t = np.clip(t, 0, 1)
        c = np.where(t[..., None] < 0.5, lerp(L, M, t * 2), lerp(M, D, t * 2 - 1))
        amp *= tex
        return c * (1 - amp / 2 + amp * (0.62 * s.n5 + 0.38 * s.n1))[..., None]

    def fill(s, m, mat, mode='flat', a=0.12, bu=0.45, bv=0.15, ink=0.5, tex=1.0, t=None):
        """Contour, then material. Tone t: 0 = fully lit, 1 = deep shade.
        'flat' ramps left to right and a little top to bottom across the
        shape's own box; 'cyl' is a cylinder; 'round' a ball lit upper left."""
        rows = np.nonzero(m.max(1) > 0.5)[0]; cols = np.nonzero(m.max(0) > 0.5)[0]
        if len(rows) == 0: return
        if ink: s.line(m, INK, ink)
        y0, y1, x0, x1 = rows[0] / SS, rows[-1] / SS + 1e-3, cols[0] / SS, cols[-1] / SS + 1e-3
        u = np.clip((s.X - x0) / (x1 - x0), 0, 1); v = np.clip((s.Y - y0) / (y1 - y0), 0, 1)
        if t is not None: tt = np.full_like(u, t) + 0.12 * (u - 0.5)
        elif mode == 'cyl':
            r = 2 * u - 1
            tt = 0.10 + 0.55 * np.clip((r + 0.35) / 0.95, 0, 1) ** 1.2 + 0.30 * np.clip((-0.35 - r) / 0.65, 0, 1) - 0.12 * np.clip((r - 0.72) / 0.28, 0, 1)
        elif mode == 'round': tt = 0.06 + 0.9 * np.clip(0.5 + 0.45 * (2 * u - 1) + 0.5 * (2 * v - 1), 0, 1)
        else: tt = a + bu * u + bv * v
        s.put(m, s.col(mat, tt, tex))

    def P(s, mat, pts, **k): s.fill(s.mp(pts), mat, **k)
    def R(s, mat, x0, y0, x1, y1, **k): s.fill(s.mr(x0, y0, x1, y1), mat, **k)
    def E(s, mat, cx, cy, rx, ry, **k): k.setdefault('mode', 'round'); s.fill(s.me(cx, cy, rx, ry), mat, **k)
    def L(s, mat, pts, w, **k): s.fill(s.ml(pts, w), mat, **k)
    def dark(s, m, c=DARK, a=1.0): s.put(m, c, a)
    def door(s, x, hw, top, bottom, arch=True, c=DARK):
        s.dark(s.march(x, hw, top, bottom) if arch else s.mr(x - hw, top, x + hw, bottom), c)
    def win(s, x0, y0, x1, y1, c=DARK):
        m = s.mr(x0, y0, x1, y1); s.line(m, INK, 0.3); s.put(m, c)
    def lines(s, segs, w=0.3, a=0.42, clip=None, c=INK):
        """Thin dark lines (joints, planks, cracks) in one pass."""
        m = Image.new('L', (s.w * SS, s.h * SS), 0); d = ImageDraw.Draw(m)
        for p0, p1 in segs: d.line([((p0[0] + OX) * SS, (p0[1] + OY) * SS), ((p1[0] + OX) * SS, (p1[1] + OY) * SS)], fill=255, width=max(1, int(round(w * SS))))
        m = np.asarray(m, np.float32) / 255
        s.put(m if clip is None else m * clip, c, a)
    def joints(s, x0, y0, x1, y1, rh=4.6, bw=7.5, a=0.38):
        """Coursed masonry joints over a rectangle."""
        segs = []; i = 0; y = y0 + rh
        while y < y1 + rh:
            if y < y1 - 0.5: segs.append(((x0, y), (x1, y)))
            x = x0 + (bw if i % 2 else bw / 2)
            while x < x1 - 1: segs.append(((x, y - rh), (x, min(y, y1)))); x += bw
            y += rh; i += 1
        s.lines(segs, a=a, clip=s.mr(x0, y0, x1, y1))
    def planks(s, x0, y0, x1, y1, step=3.2, a=0.4):
        s.lines([((x, y0), (x, y1)) for x in np.arange(x0 + step, x1 - 0.5, step)], a=a)

    # ---- grounds ----------------------------------------------------------------
    gkind = None; path = None
    def drop(s, dx):
        """How far the ground has fallen away at dx from the centre: nothing
        across the flat footprint, then a curve down to the patch's ends."""
        return s.gbulge * (np.clip(np.abs(dx) - s.gflat, 0, None) / max(s.ghw - s.gflat, 1e-3)) ** 2
    def mound(s, kind='grass', top=23.0, hw=40.0, bulge=7.0, flat=31.0):
        """The small ground patch the first-look subjects stand on. It is FLAT
        across the subject's footprint (|x| < flat) and only curves down at
        its ends (owner, 2026-11-15: a dome made the ground fall away under the
        objects, so they floated). Ragged top edge; fades into the dark."""
        s.gkind, s.gtop, s.ghw, s.gbulge, s.gflat = kind, top, hw, bulge, flat
        lo, hi = dict(grass=((50, 70, 30), (122, 146, 60)), dirt=((70, 54, 38), (156, 126, 88)),
                      snow=((150, 166, 192), (242, 246, 250)), sand=((120, 86, 50), (220, 176, 116)))[kind]
        dx = s.X - OX
        edge = OY + top + s.drop(dx) + 1.0 * (s.n3 - 0.5)
        m = ((s.Y > edge) & (np.abs(dx) < hw + 1)).astype(np.float32)
        g = lerp(lo, hi, np.clip(s.fbm(2.2) * 1.5 - 0.3, 0, 1))
        g = g * (1.15 - 0.8 * np.clip((s.Y - edge) / 14, 0, 1))[..., None] * (1 - 0.6 * np.clip((np.abs(dx) - hw * 0.55) / (hw * 0.5), 0, 1))[..., None]
        s.put(m, g); s.gmask = m; return m
    def seat(s):
        """Seat whatever stands on the mound: a contact shadow along the ground
        line, then grass blades (or clods, on bare ground) overlapping the
        bases, so nothing reads as hovering. Called once after each pin."""
        if s.gkind is None: return
        dx = s.X - OX; line = OY + s.gtop + s.drop(dx)
        band = np.clip(1 - np.abs(s.Y - (line + 1.6)) / 1.7, 0, 1) * (np.abs(dx) < s.gflat + 3)
        s.put(band, (14, 18, 10), 0.42)
        lo, hi = dict(grass=((62, 88, 36), (132, 156, 66)), dirt=((92, 72, 50), (170, 140, 100)),
                      snow=((176, 190, 212), (246, 248, 252)), sand=((150, 108, 64), (226, 184, 124)))[s.gkind]
        for i in range(54):
            x = s.rng.uniform(-s.gflat - 2, s.gflat + 2)
            if s.path and s.path[0] < x < s.path[1]: continue
            y = s.gtop + float(s.drop(np.float32(x))) + s.rng.uniform(1.6, 3.4)
            c = hi if s.rng.random() < 0.45 else lo
            if s.gkind == 'grass':
                s.put(s.mp([(x - 0.7, y), (x + s.rng.uniform(-1.0, 1.0), y - s.rng.uniform(1.8, 3.3)), (x + 0.7, y)]), c)
            else:
                s.put(s.me(x, y - 0.6, s.rng.uniform(0.7, 1.4), 0.7), c, 0.9)
    def tufts(s, n=22, top=None, hw=30.0, bulge=None, cols=((126, 150, 64), (70, 96, 38))):
        for _ in range(n):
            x = s.rng.uniform(-hw, hw); y = (s.gtop if top is None else top) + float(s.drop(np.float32(x))) + s.rng.uniform(2.0, 6)
            s.put(s.mp([(x - 0.8, y), (x + s.rng.uniform(-1.2, 1.2), y - s.rng.uniform(2, 3.6)), (x + 0.8, y)]), cols[int(s.rng.random() < 0.5)])
    def water(s, top=15.0, clip=None, rim=0.5):
        """A band of water to the disc edge, as under the owner's bridge:
        lighter at the near edge of the top, darker below, with wave strokes."""
        m = ((s.Y > OY + top + 0.5 * np.sin(s.X * 1.4)) & (s.r < 41.3)).astype(np.float32)
        if clip is not None: m = m * clip
        v = np.clip((s.Y - OY - top) / 22, 0, 1)
        c = s.col('water', 0.25 + 0.6 * v)
        wav = (np.sin(s.Y * 2.3 + 1.8 * np.sin(s.X * 0.5)) > 0.72).astype(np.float32)
        c = lerp(c, (176, 208, 232), wav * 0.45 * (1 - 0.6 * v))
        s.put(m, c * (1 - rim * np.clip((s.r - 30) / 12, 0, 1))[..., None]); return m
    def land(s, cx, cy, rx, ry):
        """A patch of grassland seen from above (for the river subjects)."""
        d = ((s.X - OX - cx) / rx) ** 2 + ((s.Y - OY - cy) / ry) ** 2 + 0.12 * (s.n3 - 0.5)
        m = (d < 1).astype(np.float32)
        s.line(m, INK, 0.5)
        g = lerp((52, 74, 30), (116, 140, 58), np.clip(s.fbm(2.5) * 1.5 - 0.3, 0, 1))
        s.put(m, g * (1.08 - 0.5 * np.clip(d, 0, 1) ** 2)[..., None]); return m
    def river(s, m):
        """Water laid over land: dark bank line, blue, wave strokes."""
        s.line(m, (30, 44, 30), 0.6)
        c = s.col('water', 0.3 + 0.25 * s.n3)
        wav = (np.sin(s.Y * 2.6 + 2.0 * np.sin(s.X * 0.6)) > 0.74).astype(np.float32)
        s.put(m, lerp(c, (180, 210, 234), wav * 0.5))

    # ---- props -----------------------------------------------------------------
    def bush(s, x, y, r, n=8, dark=(40, 62, 24), light=(124, 150, 64)):
        T.bush(s, x + OX, y + OY, r, dark, light, INK, n=n)
    def tree(s, x, base, h, r):
        s.R('woodd', x - 1.3, base - h + r, x + 1.3, base, mode='cyl', ink=0.4); s.bush(x, base - h + r * 0.6, r, n=10)
    def pine(s, x, base, h):
        w = h * 0.36
        s.R('woodd', x - h * 0.05, base - h * 0.16, x + h * 0.05, base, mode='cyl', ink=0.4)
        for f0, f1, ww in ((0.50, 0.90, 1.0), (0.24, 0.66, 0.8), (0.0, 0.42, 0.56)):
            y0, y1 = base - h + f0 * h * 0.94, base - h + f1 * h * 0.94
            s.P('pine', [(x, y0), (x - w * ww, y1), (x - w * ww * 0.45, y1 - h * 0.035), (x, y1 + h * 0.02), (x + w * ww * 0.5, y1 - h * 0.03), (x + w * ww, y1)], a=0.1, bu=0.6, bv=0.2)
    def flame(s, x, base, h, w, glow=1.0):
        gl = np.exp(-(((s.X - OX - x) / (w * 2.6)) ** 2 + ((s.Y - OY - base + h * 0.4) / (h * 1.2)) ** 2))
        s.img = s.img + np.asarray((120, 56, 14), np.float32) * (gl * 0.55 * glow)[..., None]
        outer = s.mp([(x - w, base), (x - w * 0.95, base - h * 0.35), (x - w * 0.45, base - h * 0.66), (x - w * 0.5, base - h * 0.3),
                      (x, base - h), (x + w * 0.42, base - h * 0.45), (x + w * 0.62, base - h * 0.72), (x + w, base - h * 0.3), (x + w * 0.9, base)])
        s.line(outer, (90, 30, 10), 0.35); s.put(outer, (236, 112, 38))
        s.put(s.mp([(x - w * 0.62, base), (x - w * 0.3, base - h * 0.45), (x, base - h * 0.72), (x + w * 0.35, base - h * 0.4), (x + w * 0.6, base)]), (252, 196, 72))
        s.put(s.mp([(x - w * 0.3, base), (x, base - h * 0.4), (x + w * 0.3, base)]), (255, 242, 196))
    def glow(s, x, y, r, c, k=0.6):
        s.img = s.img + np.asarray(c, np.float32) * (np.exp(-(((s.X - OX - x) ** 2 + (s.Y - OY - y) ** 2) / (r * r))) * k)[..., None]
    def boulder(s, cx, cy, rx, ry, mat='rock', n=9):
        pts = [(cx + rx * math.cos(a) * s.rng.uniform(0.82, 1.08), cy + ry * math.sin(a) * s.rng.uniform(0.82, 1.08)) for a in np.linspace(0, 2 * math.pi, n, endpoint=False)]
        pts = [(x, min(y, cy + ry * 0.75)) for x, y in pts]          # sits flat on the ground
        s.P(mat, pts, mode='round')
        s.lines([((cx - rx * 0.1, cy - ry * 0.7), (cx + rx * 0.25, cy + ry * 0.1)), ((cx + rx * 0.25, cy + ry * 0.1), (cx + rx * 0.1, cy + ry * 0.7))], a=0.3, clip=s.mp(pts))
    def peak(s, ax, ay, lx, rx, by, mat='rock', snow=0.38):
        """A mountain: lit left face, shaded right face, a jagged snow cap."""
        mx = ax + (rx - ax) * 0.22; h = by - ay
        whole = s.mp([(lx, by), (ax, ay), (rx, by)]); s.line(whole, INK, 0.55)
        s.fill(s.mp([(lx, by), (ax, ay), (mx, by)]), mat, a=0.12, bu=0.22, bv=0.25, ink=0)
        s.fill(s.mp([(mx, by), (ax, ay), (rx, by)]), mat, a=0.58, bu=0.2, bv=0.1, ink=0)
        segs = []
        for f in (0.3, 0.52, 0.74):
            px, py = ax + (mx - ax) * f, ay + h * f
            segs.append(((px, py), (px - (ax - lx) * 0.30, py + h * 0.2))); segs.append(((px, py), (px + (rx - ax) * 0.28, py + h * 0.17)))
        s.lines(segs, a=0.32, clip=whole)
        if snow:
            Lp = (ax + (lx - ax) * snow, ay + h * snow); Rp = (ax + (rx - ax) * snow, ay + h * snow); Mp = (ax + (mx - ax) * snow * 1.25, ay + h * snow * 1.25)
            d = h * 0.07; mix = lambda p, q, t: (p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t)
            s.fill(s.mp([(ax, ay), Lp, (mix(Lp, Mp, 0.3)[0], mix(Lp, Mp, 0.3)[1] - d), (mix(Lp, Mp, 0.55)[0], mix(Lp, Mp, 0.55)[1] + d), (mix(Lp, Mp, 0.8)[0], mix(Lp, Mp, 0.8)[1] - d * 0.6), Mp]), 'snow', t=0.05, ink=0)
            s.fill(s.mp([(ax, ay), Mp, (mix(Mp, Rp, 0.3)[0], mix(Mp, Rp, 0.3)[1] - d), (mix(Mp, Rp, 0.6)[0], mix(Mp, Rp, 0.6)[1] + d), Rp]), 'snow', t=0.62, ink=0)
    def spire(s, cx, base, h, w, mat='sand'):
        """A rock pinnacle: lit left, shaded right, a few strata."""
        whole = [(cx - w / 2, base), (cx - w * 0.36, base - h * 0.5), (cx - w * 0.1, base - h), (cx + w * 0.12, base - h * 0.96), (cx + w * 0.34, base - h * 0.45), (cx + w / 2, base)]
        s.P(mat, whole, a=0.08, bu=0.3, bv=0.2)
        s.fill(s.mp([(cx + w * 0.0, base - h * 0.98), (cx + w * 0.12, base - h * 0.96), (cx + w * 0.34, base - h * 0.45), (cx + w / 2, base), (cx + w * 0.14, base), (cx + w * 0.06, base - h * 0.5)]), mat, t=0.7, ink=0)
        s.lines([((cx - w * 0.4, base - h * f), (cx + w * 0.4, base - h * f + 1)) for f in (0.22, 0.45, 0.68)], a=0.3, clip=s.mp(whole))
    def facets(s, pts, mat, tones, jit=2.0):
        """Break a rock face into upright facets of differing tone with cracks
        between them, so it reads as rock and not as a slab."""
        m = s.mp(pts); xs = [p[0] for p in pts]; ys = [p[1] for p in pts]
        x0, x1, y0, y1 = min(xs), max(xs), min(ys), max(ys); n = len(tones)
        edges = [[(x0 + (x1 - x0) * i / n + (s.rng.uniform(-jit, jit) if 0 < i < n else 0), y0 + (y1 - y0) * k / 3) for k in range(4)] for i in range(n + 1)]
        edges[0] = [(x0 - 1, y) for _, y in edges[0]]; edges[-1] = [(x1 + 1, y) for _, y in edges[-1]]
        for i, t in enumerate(tones):
            s.fill(s.mp(edges[i] + edges[i + 1][::-1]) * m, mat, t=t, ink=0)
        s.lines([(e[k], e[k + 1]) for e in edges[1:-1] for k in range(3)], w=0.45, a=0.45, clip=m)
    def house(s, x0, x1, eave, base, rise, wall='plaster', roof='roof', kind='gable', door=None, wins=(), over=2.5):
        s.R(wall, x0, eave, x1, base)
        if kind == 'gable': s.P(roof, [(x0 - over, eave + 1), ((x0 + x1) / 2, eave - rise), (x1 + over, eave + 1)], a=0.1, bu=0.5, bv=0.1)
        else: s.P(roof, [(x0 - over, eave + 1), (x0 + rise * 0.45, eave - rise), (x1 - rise * 0.45, eave - rise), (x1 + over, eave + 1)], a=0.1, bu=0.45, bv=0.15)
        s.lines([((x0 - over, eave + 1), (x1 + over, eave + 1))], w=0.5, a=0.5)
        if door: s.R('woodd', door[0] - door[1], base - door[2], door[0] + door[1], base, ink=0.35)
        for w in wins: s.win(*w)
    def tower(s, x0, x1, top, base, n=3, mat='stone', over=1.4, joints=True):
        s.R(mat, x0, top, x1, base, a=0.1, bu=0.5, bv=0.1)
        if joints: s.joints(x0, top, x1, base)
        s.R(mat, x0 - over, top - 2.6, x1 + over, top + 1.2, a=0.08, bu=0.45)
        step = (x1 - x0 + 2 * over) / (2 * n - 1)
        for i in range(n): s.R(mat, x0 - over + 2 * i * step, top - 6, x0 - over + (2 * i + 1) * step, top - 2.4, a=0.08 + 0.4 * i / n, bu=0.15)
    def wall(s, x0, x1, top, base, n=5, mat='stone'):
        s.R(mat, x0, top, x1, base, a=0.14, bu=0.4, bv=0.15); s.joints(x0, top, x1, base)
        step = (x1 - x0) / (2 * n - 1)
        for i in range(n): s.R(mat, x0 + 2 * i * step, top - 3.6, x0 + (2 * i + 1) * step, top + 0.4, a=0.1 + 0.4 * i / n, bu=0.15)
    def banner(s, x, top, w, h, mat='blue'):
        s.P(mat, [(x - w / 2, top), (x + w / 2, top), (x + w / 2, top + h), (x, top + h - w * 0.4), (x - w / 2, top + h)], a=0.15, bu=0.4, bv=0.2, ink=0.4)
        s.E('gold', x, top + h * 0.4, w * 0.22, w * 0.22, ink=0.25)
    def wheel(s, x, y, r):
        s.E('woodd', x, y, r, r, t=0.45); s.dark(s.me(x, y, r * 0.66, r * 0.66), (30, 23, 18))
        s.lines([((x - r * 0.8 * math.cos(a), y - r * 0.8 * math.sin(a)), (x + r * 0.8 * math.cos(a), y + r * 0.8 * math.sin(a))) for a in np.arange(0, math.pi, math.pi / 4)], w=0.7, a=1.0, c=(120, 84, 52))
        s.E('wood', x, y, r * 0.24, r * 0.24, ink=0.25)
    def star(s, x, y, r, c=(255, 250, 220)):
        q = r * 0.24
        s.put(s.mp([(x, y - r), (x + q, y - q), (x + r, y), (x + q, y + q), (x, y + r), (x - q, y + q), (x - r, y), (x - q, y - q)]), c)


PINS = {}
def pin(name, seed=1):
    def deco(f): PINS[name] = (f, seed); return f
    return deco


# ---- settlements and buildings -------------------------------------------------
@pin('city')
def _(q):
    # A walled city: gable house, slate spire and a round tower behind a
    # crenellated wall with its gatehouse.
    q.mound('grass', 23)
    q.house(-27, -14, -5, 12, 9, wins=[(-22.5, -1, -19, 4)])
    q.R('stone', -12.5, -9, -7.5, 12); q.P('slate', [(-14, -8), (-10, -18), (-6, -8)], a=0.1, bu=0.6)
    q.R('stone', -7, -17, 7, 12, a=0.1, bu=0.5); q.joints(-7, -17, 7, 12); q.P('slate', [(-9.5, -16), (0, -33), (9.5, -16)], a=0.08, bu=0.65); q.win(-2, -11, 2, -4)
    q.R('stone', 13, -8, 25, 12, mode='cyl'); q.P('roof', [(11, -7), (19, -21), (27, -7)], a=0.1, bu=0.6); q.win(17.5, -3, 20.5, 3)
    q.wall(-31, 31, 10, 25, n=8)
    q.tower(-9, 9, 5, 25, n=3); q.door(0, 5, 13, 25); q.R('woodd', -4, 17, 4, 25, ink=0.2, t=0.5)
    for x in (-21, 21): q.win(x - 1.2, 15, x + 1.2, 20)
    q.tufts(14)

@pin('hamlet')
def _(q):
    # Three cottages and a tree.
    q.mound('grass', 23)
    q.tree(-28, 24.5, 24, 7.5)
    q.house(-6, 10, -6, 24, 11, door=None, wins=[(0, 0, 4, 6)])
    q.house(-23, -3, 6, 24, 12, door=(-13, 2.6, 9), wins=[(-20.5, 11, -17, 15.5), (-8.5, 11, -5, 15.5)])
    q.house(7, 28, 9, 24, 10, kind='slope', door=(14, 2.4, 8), wins=[(20, 13, 24, 17.5)])
    q.E('dirt', -2, 27.5, 9, 2.2, ink=0, t=0.3)
    q.tufts(18)

@pin('tavern')
def _(q):
    # A foaming tankard (owner, 2026-11-15: "The tavern can still be the beer
    # mug, it really sells it well"): wooden staves, iron hoops, a handle, and
    # a head of foam spilling down one side. It stands on the grass.
    q.mound('grass', 23)
    q.E('woodd', 15, 8, 10.5, 12.5, t=0.45); q.dark(q.me(15, 8, 6.2, 8.2), (22, 21, 19))
    body = [(-17, 25), (-16, -8), (11, -8), (12, 25)]
    q.P('wood', body, mode='cyl')
    q.lines([((x, -8), (x * 1.04, 25)) for x in (-11.5, -7, -2.5, 2, 6.5)], w=0.45, a=0.45, clip=q.mp(body))
    for y in (-2.5, 17):
        q.fill(q.mr(-17.6, y, 12.6, y + 3.4), 'metal', mode='cyl', ink=0.35)
        for x in (-12, -2.5, 7): q.put(q.me(x, y + 1.7, 0.7, 0.7), (60, 62, 70))
    foam = np.maximum.reduce([q.me(-13, -11, 7, 6), q.me(-4, -15, 8.5, 7.5), q.me(6, -12.5, 7.5, 6.5), q.me(12, -9, 4.5, 4), q.mr(-17, -10, 12, -6),
                              q.me(-17.5, -4, 3.8, 5), q.me(-18, 3, 2.6, 4.2), q.me(3, -6, 3.2, 3.6)])
    q.fill(foam, 'foam', mode='round', ink=0.55)
    for x, y, r in ((-13, -12.5, 3.2), (-5, -17, 4), (5, -14.5, 3.2), (-17.5, -5, 1.6)): q.put(q.me(x, y, r, r * 0.8), (255, 253, 244), 0.85)
    q.lines([((-8.5, -9), (-7, -6.5)), ((0.5, -10), (2, -7.5)), ((9, -8.5), (9.8, -6.5))], w=0.5, a=0.3)

@pin('inn')
def _(q):
    # A half-timbered inn with lit windows and a hanging sign (the building
    # first painted for `tavern`; the owner kept it as an inn).
    q.mound('grass', 23)
    q.R('stone', 6, -24, 11, -8); q.R('stone', 5, -26, 12, -23.5)
    q.R('plaster', -20, -3, 14, 24)
    q.P('roof', [(-24.5, -2), (-3, -27), (18.5, -2)], a=0.1, bu=0.55, bv=0.1)
    for seg in ([(-20, -2), (-20, 24)], [(14, -2), (14, 24)], [(-20, 10), (14, 10)], [(-20, -2.4), (14, -2.4)], [(-3, -2), (-3, 10)], [(-20, 10), (-12, -2)], [(14, 10), (6, -2)]):
        q.L('woodd', seg, 1.5, ink=0.2, t=0.5)
    q.R('woodd', -7, 12, 1, 24, ink=0.35); q.E('gold', -0.6, 18.5, 0.7, 0.7, ink=0)
    for w in ((-17, 13, -11, 20), (5, 13, 11, 20), (-13, 1, -7.5, 7), (1.5, 1, 7, 7)): q.win(*w, c=(250, 196, 96))
    q.glow(-14, 16, 7, (90, 50, 10), 0.4); q.glow(8, 16, 7, (90, 50, 10), 0.4)
    q.R('woodd', 21.6, -9, 24, 24, mode='cyl'); q.R('woodd', 14, -9, 24, -7)
    q.lines([((16.5, -7), (16.5, -4.5)), ((21, -7), (21, -4.5))], w=0.35, a=1)
    q.R('wood', 15, -4.5, 22.5, 3.5); q.E('gold', 18.7, -0.5, 2.0, 2.0, ink=0.25)
    q.tufts(14)

@pin('smithy')
def _(q):
    # An anvil on its stump with a smith's hammer standing on it (owner,
    # 2026-11-15: "Smithy could just be an anvil"; then "The handle from the
    # hammer should point upright and a bit thicker. It looks fragile").
    # The anvil's horn is a horizontal taper off one end of a flat face. The
    # hammer stands head-down on the face: a chunky iron head lying along it
    # (flat striking face at the heel end, wedge peen toward the horn) and a
    # thick wooden handle rising near-vertically from its eye.
    q.mound('grass', 23)
    d = 3.0                                                   # the anvil sits this much lower than centre, to leave room for the handle
    q.R('woodd', -15, 15 + d, 9, 25.5, mode='cyl'); q.E('wood', -3, 15 + d, 12, 2.6, t=0.2, ink=0.4, mode='flat')
    q.lines([((x, 17.5 + d), (x, 25.5)) for x in (-10, -5.5, -0.5, 4.5)], a=0.35, clip=q.mr(-15, 15 + d, 9, 25.5))
    q.P('metal', [(-17, 16 + d), (-13, 9.5 + d), (7, 9.5 + d), (11, 16 + d)], a=0.3, bu=0.4, bv=0.2)                 # foot
    q.R('metal', -9, 4 + d, 3, 10 + d, a=0.35, bu=0.4)                                                                # waist
    top = [(-24, -12 + d), (12, -12 + d), (33.5, -9.5 + d), (33, -7.5 + d), (13, -1 + d), (8, -1 + d), (5, 5 + d), (-11, 5 + d), (-14, -1 + d), (-24, -1 + d)]
    q.P('metal', top, a=0.3, bu=0.3, bv=0.35)
    q.fill(q.mp([(-24, -12 + d), (12, -12 + d), (33.5, -9.5 + d), (33.2, -8.6 + d), (12, -9.6 + d), (-24, -9.6 + d)]), 'metal', t=0.02, ink=0)   # the bright face
    q.lines([((-24, -9.4 + d), (12, -9.4 + d)), ((12, -9.4 + d), (33, -8.4 + d))], w=0.35, a=0.35)
    f = -12 + d                                               # the anvil face the hammer stands on
    handle = [(-5.6, f - 8.0), (-0.4, f - 8.0), (3.4, -34.6), (-2.6, -35.6)]
    q.P('wood', handle, mode='cyl', ink=0.55)
    q.E('wood', 0.4, -35.3, 3.1, 2.0, ink=0.5, mode='cyl')                                                           # the rounded end of the grip
    q.lines([((-3.6, f - 9), (-0.6, -33)), ((-1.8, f - 9), (1.4, -33)), ((-4.6, f - 12), (-2.6, -27))], w=0.3, a=0.35, clip=q.mp(handle))   # grain
    head = [(-12.5, f), (-12.5, f - 9.4), (-9.8, f - 9.4), (-9.2, f - 8.4), (3.5, f - 8.4), (10, f - 5.0), (10, f - 3.2), (3.5, f)]
    q.P('metal', head, a=0.2, bu=0.35, bv=0.35, ink=0.55)
    q.fill(q.mr(-12.5, f - 9.4, -10.2, f), 'metal', t=0.03, ink=0)                                                   # the polished striking face
    q.fill(q.mp([(-9.2, f - 8.4), (3.5, f - 8.4), (10, f - 5.0), (10, f - 4.4), (3.5, f - 7.0), (-9.2, f - 7.0)]), 'metal', t=0.08, ink=0)   # light along the top
    q.lines([((-10.1, f - 9.2), (-10.1, f)), ((3.5, f - 8.2), (3.5, f))], w=0.35, a=0.45)
    q.R('woodd', -5.9, f - 9.6, -0.1, f - 7.6, t=0.5, ink=0.35)                                                      # the handle's collar at the eye
    q.glow(24, -17, 8, (120, 70, 14), 0.4)
    for x, y, r in ((23, -16, 2.8), (29, -21, 2.0), (18, -23, 1.8)): q.star(x, y, r, (255, 226, 130))

@pin('shrine')
def _(q):
    # A small stone shrine: pointed arch between pillars, a blue light inside.
    q.mound('grass', 23)
    q.bush(-25, 20.5, 5.5); q.bush(25, 20.5, 5.5)
    q.R('stone', -19, 20, 19, 25); q.R('stone', -15, 16, 15, 20.4, a=0.08)
    q.glow(0, 2, 15, (20, 60, 110), 0.5)
    g = np.maximum(q.mr(-7, -9, 7, 16), q.mp([(-7, -9), (0, -20), (7, -9)]))
    q.put(g, lerp((236, 250, 255), (56, 128, 206), np.clip(np.abs(q.X - OX) / 7.5 + 0.25 * np.clip((q.Y - OY - 6) / 10, 0, 1), 0, 1)))
    q.P('stone', [(-18, -9), (0, -29), (18, -9), (11, -9), (0, -21), (-11, -9)], a=0.08, bu=0.5)
    q.R('stone', -13, -9, -7, 16, mode='cyl'); q.R('stone', 7, -9, 13, 16, mode='cyl')
    q.R('stone', -15, -11.5, -5.5, -8.5, a=0.06); q.R('stone', 5.5, -11.5, 15, -8.5, a=0.3)
    q.star(0, -31.5, 4.2, (214, 240, 255)); q.glow(0, -31.5, 5, (40, 80, 120), 0.5)
    q.tufts(14)

@pin('guild')
def _(q):
    # A guildhall with its banner hung over the door.
    q.mound('grass', 23)
    q.R('plaster', -24, 4, 20, 24)
    for seg in ([(-24, 4.5), (-24, 24)], [(20, 4.5), (20, 24)], [(-24, 13), (20, 13)]): q.L('woodd', seg, 1.4, ink=0.2, t=0.5)
    q.P('roof', [(-28.5, 5), (-2, -17), (24.5, 5)], a=0.1, bu=0.55)
    q.R('woodd', -6, 13, 2, 24, ink=0.35)
    for w in ((-20, 15, -14, 21), (9, 15, 15, 21), (-20, 6.5, -14, 11), (9, 6.5, 15, 11)): q.win(*w, c=(60, 70, 96))
    q.L('woodd', [(-2, -35), (-2, -27)], 1.5, ink=0.3); q.L('woodd', [(-11, -31), (7, -31)], 1.5, ink=0.3)
    q.banner(-2, -30.5, 15, 25)
    q.tufts(14)

@pin('gate')
def _(q):
    # A gatehouse: two towers, banners, the way through between them.
    q.mound('grass', 23)
    q.wall(-13, 13, -3, 24, n=4)
    q.door(0, 8, 5, 24)
    q.put(q.mp([(-4, 24), (4, 24), (6, 14), (-6, 14)]) * q.march(0, 8, 5, 24), (86, 68, 48))      # the road seen through the arch
    q.tower(-29, -13, -13, 24, n=3); q.tower(13, 29, -13, 24, n=3)
    q.banner(-21, -8, 7, 15); q.banner(21, -8, 7, 15)
    q.path = (-9, 9)
    q.P('dirt', [(-7, 24), (7, 24), (11, 31), (-11, 31)], ink=0, t=0.35)
    q.tufts(12)

@pin('well')
def _(q):
    # A roofed draw-well.
    q.mound('grass', 23)
    q.R('woodd', -14, -12, -11, 9, mode='cyl'); q.R('woodd', 11, -12, 14, 9, mode='cyl'); q.R('woodd', -14, -6.5, 14, -4.2, t=0.45)
    q.lines([((0, -4), (0, 2))], w=0.5, a=1, c=(206, 184, 140))
    q.P('wood', [(-4, 1.5), (4, 1.5), (3, 8), (-3, 8)], a=0.1, bu=0.6)
    q.R('stone', -15.5, 8, 15.5, 24.5, mode='cyl'); q.joints(-15.5, 9, 15.5, 24.5, rh=4, bw=6.5)
    q.E('stone', 0, 8, 15.5, 3.6, t=0.08); q.dark(q.me(0, 8.2, 11.5, 2.2))
    q.P('shingle', [(-21.5, -11), (0, -28), (21.5, -11)], a=0.1, bu=0.55)
    q.lines([((x, -11), (x * 0.25, -24)) for x in (-14, -7, 7, 14)], a=0.3, clip=q.mp([(-21.5, -11), (0, -28), (21.5, -11)]))
    q.tufts(16)

@pin('camp')
def _(q):
    # Tents round a campfire.
    q.mound('grass', 23)
    q.P('canvas', [(6, 24.5), (18, 2), (30, 24.5)], a=0.35, bu=0.4); q.dark(q.mp([(16, 24.5), (18, 9), (21, 24.5)]))
    q.L('woodd', [(-9, -15), (-9, -21)], 1.2, ink=0.3); q.P('red', [(-9, -21), (-2, -19), (-9, -17)], ink=0.3)
    q.fill(q.mp([(-30, 24), (-9, -15), (13, 24)]), 'canvas', t=0.6)
    q.fill(q.mp([(-30, 24), (-9, -15), (-5, 24)]), 'canvas', a=0.05, bu=0.3, bv=0.1, ink=0)
    q.dark(q.mp([(-12, 24), (-9, 0), (-3, 24)])); q.lines([((-9, -15), (-5, 24))], a=0.4)
    q.flame(20.5, 26, 13, 5)
    for x in (16, 20.5, 25): q.boulder(x, 26.5, 2.6, 1.8, n=7)
    q.tufts(12)

@pin('other')
def _(q):
    # An unmarked place: a traveller's tent with a question mark on it.
    q.mound('grass', 23)
    q.L('woodd', [(0, -22), (0, -29)], 1.3, ink=0.3); q.P('red', [(0, -29), (8, -27), (0, -25)], ink=0.3)
    q.fill(q.mp([(-28, 24), (0, -22), (28, 24)]), 'hide', t=0.62)
    q.fill(q.mp([(-28, 24), (0, -22), (5, 24)]), 'hide', a=0.08, bu=0.3, bv=0.1, ink=0)
    pts = [(-4 + 6.2 * math.cos(math.radians(a)), 2 + 6.2 * math.sin(math.radians(a))) for a in range(-190, 61, 10)]
    q.L('canvas', pts + [(-3.5, 10), (-4, 13)], 3.6, ink=0.4, t=0.08); q.E('canvas', -4, 18.5, 2.3, 2.3, ink=0.4, t=0.08)
    q.tufts(14)


# ---- water --------------------------------------------------------------------
@pin('spring')
def _(q):
    # Water breaking out between mossy rocks into a pool.
    q.mound('grass', 20)
    q.bush(-3, -16, 7, n=9)
    q.boulder(1, -3, 13, 12); q.boulder(-15, 10, 13, 14); q.boulder(15, 11, 12, 13)
    fall = q.mp([(-3.5, 2), (3.5, 2), (6, 23), (-6, 23)]); q.line(fall, (30, 50, 80), 0.45)
    q.put(fall, lerp((236, 246, 252), (112, 170, 220), np.clip(np.abs(np.sin((q.X - OX) * 2.6)) * 0.9, 0, 1)))
    pool = q.me(0, 26, 23, 5.5); q.river(pool)
    for x, rx in ((0, 7.5), (-5, 3.5), (6, 3)): q.put(q.me(x, 23.5, rx, 2.2), (244, 250, 252), 0.9)
    q.boulder(-23, 25, 5, 3.6); q.boulder(23.5, 25.5, 5, 3.6)

@pin('river_confluence')
def _(q):
    # Two rivers joining, seen from above on a patch of green country.
    land = q.land(0, 3, 37, 29)
    m = np.maximum.reduce([q.ml(bez((-32, -17), (-12, -12), (0, 6)), 7.5), q.ml(bez((31, -19), (12, -12), (0, 6)), 7.5), q.ml(bez((0, 6), (3, 18), (-2, 34)), 11)])
    q.river(m * land)
    q.pine(-22, 20, 17); q.pine(22, 18, 17); q.pine(0, -8, 14)
    q.bush(-12, 24, 3.6, n=5); q.bush(14, 26, 3.4, n=5)

@pin('delta')
def _(q):
    # A river delta: the river crosses the mainland and fans out over a lobe
    # of low ground pushed into the sea, sand at its rim.
    q.water(-12)
    main = q.land(0, -27, 41, 19)
    fan = q.land(-1, 3, 28, 20)                               # the lobe; no ring round it, and no symmetric trident on it
    both = np.maximum(main, fan)
    ch = np.maximum.reduce([q.ml(bez((3, -40), (-3, -24), (0, -9)), 7.5),
                            q.ml(bez((0, -9), (-10, -2), (-25, 12)), 4.8), q.ml(bez((-8, -3), (-10, 10), (-12, 24)), 3.8),
                            q.ml(bez((0, -9), (5, 4), (3, 25)), 4.8), q.ml(bez((3, 2), (14, 6), (24, 14)), 4.2), q.ml(bez((4, 12), (10, 18), (14, 24)), 3.2)])
    q.river(ch * both)
    for x, y in ((-30, -20), (31, -21), (-16, -24), (17, -25)): q.bush(x, y, 3.2, n=5)
    for x, y in ((-18, 12), (-4, 14), (13, 13), (16, -2), (-14, -6)): q.put(q.me(x, y, 2.4, 1.3), (214, 196, 136), 0.85)   # sand bars

@pin('ford')
def _(q):
    # A shallow river crossed by a track on stepping stones.
    land = q.land(0, 3, 37, 29)
    for pts in ([(-12, -26), (-8, -12)], [(6, 14), (12, 32)]): q.fill(q.ml(pts, 5) * land, 'dirt', ink=0.35, t=0.3)
    q.river(q.ml(bez((-40, -2), (0, -10), (40, 10)), 15) * land)
    for x, y in ((-6.5, -7), (-1.5, 0.5), (3.5, 8)): q.E('stone', x, y, 4.2, 2.8, ink=0.45)
    q.bush(-26, 12, 4.5, n=6); q.bush(24, -10, 4.5, n=6); q.pine(-24, -4, 13); q.pine(26, 26, 13)

@pin('portage')
def _(q):
    # A boat riding on a two-wheeled cart (owner's glyph pick).
    q.mound('grass', 24)
    hull = [(-31, -11), (-23, -4), (15, -4), (23, -8), (29, -16), (24, 0), (16, 7), (-19, 7), (-27, 0)]
    q.P('wood', hull, a=0.12, bu=0.4, bv=0.3)
    q.lines([((-25, -0.5), (22, -0.5)), ((-22, 3.2), (19, 3.2))], a=0.45, clip=q.mp(hull))
    q.L('wood', [(-23, -4), (15, -4), (23, -8)], 1.3, ink=0, t=0.05)
    q.R('woodd', -22, 8, 20, 11.5, t=0.4); q.L('woodd', [(20, 9.8), (34, 4.5)], 2.0, ink=0.4, t=0.4)
    q.wheel(-11, 17.5, 8.5); q.wheel(11, 17.5, 8.5)
    q.tufts(10, top=24)

@pin('historic_crossing')
def _(q):
    # An old three-arch bridge whose middle arch has fallen (owner's choice).
    # The two halves are deliberately unequal, with a lone pier stub between
    # them, so the arches do not pair up into a face.
    left = [(-42, -9), (-9, -13), (-5, -8), (-9, -4), (-6, 0), (-6, 19), (-42, 19)]
    right = [(42, -5), (21, -8), (18, -3), (21, 1), (17, 5), (17, 19), (42, 19)]
    stub = [(3, 19), (3, 6), (5.5, 3), (7.5, 7), (10, 4.5), (10.5, 19)]
    for pts in (left, right, stub): q.P('stone', pts, a=0.14, bu=0.35, bv=0.2)
    both = np.maximum.reduce([q.mp(left), q.mp(right), q.mp(stub)])
    q.lines([((-42, -4), (-6, -5)), ((17, 0), (42, 0))] + [((x, -5), (x, 19)) for x in range(-38, 40, 7)] + [((-42, y), (42, y)) for y in (3, 9, 15)], a=0.3, clip=both)
    for x, hw, top in ((-26, 9, 1), (31, 6, 6)):
        q.fill(q.march(x, hw + 1.9, top - 1.9, 19) * both, 'stone', t=0.12, ink=0)
        a0 = q.march(x, hw, top, 19); q.line(a0 * both, INK, 0.3); q.dark(a0)
    q.lines([((-42, -9.5), (-9, -13.5)), ((42, -5.5), (21, -8.5))], w=0.8, a=0.7, c=(246, 234, 210))
    q.water(14)
    q.P('stone', [(-3, 14.5), (1.5, 12.5), (2.5, 16.5), (-2, 18)]); q.P('stone', [(12, 17.5), (15.5, 16), (16.5, 19.5), (13, 20.5)], t=0.5)
    for x in (-26, 6.5, 31): q.put(q.me(x, 15.2, 5.5, 1.1), (220, 236, 246), 0.6)


# ---- land -----------------------------------------------------------------------
@pin('mountain_pass')
def _(q):
    # Two peaks and the track that climbs to the saddle between them.
    q.mound('grass', 22)
    q.peak(17, -19, -4, 38, 24); q.peak(-15, -27, -38, 9, 24)
    q.peak(1, 5, -13, 15, 24, snow=0)                        # the saddle between them
    path = bez((1, 30), (-10, 21), (0, 15), 10) + bez((0, 15), (7, 11), (1, 6), 8)
    q.L('dirt', path, 3.4, ink=0.4, t=0.2)
    q.bush(-27, 21.5, 4.5, n=6); q.bush(28, 21.5, 4.5, n=6)
    q.tufts(10)

@pin('ridge')
def _(q):
    # A long serrated ridge, a green shoulder in front of it.
    q.mound('grass', 22)
    q.peak(-27, -8, -40, -10, 24, snow=0.3); q.peak(29, -10, 14, 40, 24, snow=0.3)
    q.peak(-9, -21, -26, 8, 24); q.peak(12, -27, -6, 30, 24)
    q.P('grass', [(-38, 26), (-36, 20), (-22, 12), (-10, 18), (4, 10), (18, 17), (30, 13), (38, 20), (38, 26)], a=0.2, bu=0.4, bv=0.2)
    q.tufts(12)

@pin('cliff')
def _(q):
    # A sheer sea cliff: broken rock in upright facets, a ledge, fallen blocks
    # in the surf, a pine at the edge.
    q.water(15)
    q.pine(-22, -12, 16)
    face = [(-41, 30), (-41, -11), (-30, -13), (-18, -11), (-6, -13), (3, -11), (9, -6), (5, -1), (10, 5), (6, 11), (11, 17), (7, 30)]
    q.P('rockw', face, a=0.15, bu=0.3, bv=0.15)
    q.facets(face, 'rockw', (0.22, 0.4, 0.28, 0.5, 0.36, 0.74), jit=2.4)
    q.lines([((-41, 3), (-12, 1)), ((-30, 14), (2, 12)), ((-12, 1), (-9, 6)), ((-9, 6), (8, 5))], w=0.45, a=0.42, clip=q.mp(face))
    q.P('grass', [(-41, -14.5), (-30, -16.5), (-18, -14.5), (-6, -16.5), (3, -14.5), (9.5, -7), (3, -9.5), (-6, -11.5), (-18, -9.5), (-30, -11.5), (-41, -9.5)], a=0.1, bu=0.4)
    q.water(17, clip=q.mr(-45, 0, 45, 45))
    q.boulder(14, 20, 5, 3.6, 'rockw'); q.boulder(22, 24, 3.6, 2.6, 'rockw'); q.boulder(-2, 21, 3.4, 2.4, 'rockw')
    for x, rx in ((8, 7), (-18, 9), (-33, 5)): q.put(q.me(x, 18.3, rx, 1.2), (232, 244, 250), 0.8)

@pin('gorge')
def _(q):
    # A river in the slot between two broken rock walls.
    q.mound('grass', 24)
    q.fill(q.mp([(-12, -13), (14, -9), (9, 26), (-6, 26)]), 'rockw', t=0.95, ink=0)      # the dark back of the slot
    q.P('water', [(-1.2, 4), (2, 4), (6.5, 28), (-5.5, 28)], a=0.2, bu=0.2, bv=0.4, ink=0.35)
    lw = [(-36, 26), (-36, -11), (-28, -15), (-18, -12), (-10, -15), (-5, -4), (-9, 3), (-4, 12), (-6, 19), (-2.5, 26)]
    rw = [(36, 26), (36, -8), (28, -12), (20, -8), (12, -11), (7, -1), (11, 6), (6, 14), (8, 20), (5, 26)]
    q.P('rockw', lw, a=0.12, bu=0.3, bv=0.15); q.facets(lw, 'rockw', (0.2, 0.36, 0.26, 0.5, 0.8), jit=2.2)
    q.P('rockw', rw, a=0.3, bu=0.4, bv=0.15); q.facets(rw, 'rockw', (0.1, 0.3, 0.46, 0.36, 0.56), jit=2.2)
    q.lines([((-36, 2), (-12, 0)), ((-30, 14), (-8, 13)), ((12, 4), (36, 2)), ((10, 16), (30, 15))], w=0.45, a=0.4, clip=np.maximum(q.mp(lw), q.mp(rw)))
    q.P('grass', [(-36, -14.5), (-28, -18.5), (-18, -15.5), (-10, -18.5), (-7.5, -13), (-10, -13.5), (-18, -10.5), (-28, -13.5), (-36, -9.5)], a=0.1, bu=0.4)
    q.P('grass', [(36, -11.5), (28, -15.5), (20, -11.5), (12, -14.5), (9.5, -9), (12, -9.5), (20, -6.5), (28, -10.5), (36, -6.5)], a=0.2, bu=0.4)
    q.pine(-25, -15, 14); q.pine(26, -12, 14)
    q.put(q.me(0.5, 25.5, 5, 1.2), (226, 240, 248), 0.7)

@pin('rock_formation')
def _(q):
    # Weathered rock spires.
    q.mound('sand', 23)
    q.spire(-17, 25, 36, 14); q.spire(19, 25, 40, 14); q.spire(1, 25, 52, 15)
    q.spire(-29, 25, 16, 8); q.spire(30.5, 25, 13, 7)
    q.boulder(-8, 25, 4.4, 3, 'sand'); q.boulder(11, 26, 4, 2.8, 'sand')
    q.tufts(8, cols=((150, 150, 70), (96, 104, 44)))

@pin('glacier')
def _(q):
    # An ice front between two snow peaks, meltwater at its foot.
    q.mound('snow', 21)
    q.peak(-23, -24, -40, -4, 23.5, snow=0.62); q.peak(23, -19, 4, 40, 23.5, snow=0.62)
    top = [(-27, 4), (-24, -2), (-18, -5), (-14, -1), (-9, -8), (-3, -10), (1, -5), (6, -9), (12, -11), (16, -5), (21, -6), (25, -1), (27, 5)]
    wallp = [(-27, 23.5)] + top + [(27, 23.5)]
    q.P('ice', wallp, a=0.08, bu=0.5, bv=0.25)                 # one ice front, not a row of bars
    q.fill(q.mp(top + [(x, y + 3.2 + (1.2 if i % 2 else 0)) for i, (x, y) in enumerate(top[::-1])]), 'snow', t=0.05, ink=0)
    q.lines([((x, y + 4), (x + dx, 20)) for (x, y), dx in zip(top[1:-1:2], (1.5, -1, 1, -1.5, 1, -1))], w=0.5, a=0.5, c=(60, 110, 160), clip=q.mp(wallp))
    q.lines([((x + 2.5, y + 9), (x + 3.5, 20)) for (x, y) in top[2:-2:3]], w=0.35, a=0.4, c=(60, 110, 160), clip=q.mp(wallp))
    q.river(q.me(0, 26.5, 24, 4.6))
    for x, y in ((-10, 26), (8, 27.5), (16, 25.5)): q.P('ice', [(x - 3, y + 1), (x - 1.5, y - 2), (x + 2.5, y - 1.5), (x + 3, y + 1)], a=0.05, bu=0.5, ink=0.35)

@pin('quarry')
def _(q):
    # A quarry cut into a rocky hillside: rough rock behind, the stepped
    # working face of squared blocks in front, a timber crane lifting one.
    q.mound('dirt', 23)
    hill = [(-36, 25), (-36, -8), (-31, -20), (-22, -26), (-12, -22), (-4, -12), (3, 0), (7, 12), (9, 25)]
    q.P('rockw', hill, a=0.3, bu=0.3, bv=0.1); q.facets(hill, 'rockw', (0.45, 0.62, 0.5, 0.7, 0.58))
    q.bush(-24, -25, 4.2, n=6); q.bush(-13, -21.5, 3.2, n=5)
    for x0, top, x1 in ((-31, -11, -15), (-15, 0, 0), (0, 11, 13)):
        q.R('stone', x0, top, x1, 24.5, a=0.06, bu=0.35, bv=0.15); q.joints(x0, top, x1, 24.5, rh=6, bw=9)
        q.R('stone', x0, top, x1, top + 1.6, t=0.02, ink=0)
    q.L('woodd', [(22, 24.5), (13, -24)], 2.2, ink=0.4, t=0.4); q.L('woodd', [(7, 24.5), (13, -24)], 2.0, ink=0.4, t=0.55)
    q.L('woodd', [(13, -24), (30, -11)], 2.0, ink=0.4, t=0.35); q.L('woodd', [(17.5, 0), (30, -11)], 1.4, ink=0.3, t=0.5)
    q.lines([((30, -11), (30, 3))], w=0.5, a=1, c=(206, 184, 140))
    q.R('stone', 25.5, 3, 34, 10.5)
    q.R('stone', 23, 18, 32, 24.8, a=0.2); q.boulder(17, 25, 3, 2.2, 'stone')

@pin('salt_works')
def _(q):
    # A salt works: a thatched boiling hut, rounded heaps of white salt, and
    # shallow evaporation pans crusted with salt at their edges.
    q.mound('sand', 9, hw=40, bulge=4, flat=33)
    q.house(-31, -15, -3, 10.5, 9, wall='plaster', roof='thatch', door=(-23, 2.4, 8))
    def heap(ax, ay, lx, rx, by):
        Lc = bez((lx, by), (ax - (ax - lx) * 0.35, ay + (by - ay) * 0.1), (ax, ay), 8); Rc = bez((ax, ay), (ax + (rx - ax) * 0.35, ay + (by - ay) * 0.1), (rx, by), 8)
        q.fill(q.mp(Lc + Rc), 'snow', t=0.62)
        q.fill(q.mp(Lc + [(ax + (rx - ax) * 0.15, by)]), 'snow', a=0.02, bu=0.2, bv=0.15, ink=0)
        q.star(ax - (ax - lx) * 0.3, ay + (by - ay) * 0.45, 1.6)
    heap(22, -3, 11, 33, 10.5); heap(3, -15, -12, 18, 10.5)
    for r, (y0, y1, sh) in enumerate(((13, 18.5, 0), (20.5, 27, 2))):
        for i in range(3):
            x0 = -30 - sh + i * (20 + sh * 0.7)
            pan = q.mp([(x0 + 1.5, y0), (x0 + 18.5 + sh * 0.3, y0), (x0 + 19.5 + sh * 0.6, y1), (x0, y1)])
            q.line(pan, INK, 0.5); q.put(pan, q.col('wood', np.full_like(q.X, 0.45)))
            inner = q.mp([(x0 + 3, y0 + 1.2), (x0 + 17.2 + sh * 0.3, y0 + 1.2), (x0 + 18 + sh * 0.6, y1 - 1.2), (x0 + 1.8, y1 - 1.2)])
            q.put(inner, (244, 246, 248)); q.put(q.grow(1 - inner, 0.9) * inner * 0 + inner * (1 - q.grow(1 - inner, 1.0)), lerp((176, 204, 226), (120, 160, 198), np.clip(q.n3 + 0.15 * (i + r) - 0.2, 0, 1)))
    q.L('woodd', [(-5, 10), (-9, -9)], 1.4, ink=0.35, t=0.4); q.L('woodd', [(-12, -9.5), (-6.5, -8.5)], 1.6, ink=0.3, t=0.4)

@pin('road_junction')
def _(q):
    # A fingerpost where the road forks (approved in the glyph set as is).
    g = q.mound('grass', 23)
    fork = np.maximum.reduce([q.ml([(0, 25), (-24, 33)], 6), q.ml([(0, 25), (24, 33)], 6), q.me(0, 25.5, 6, 2.5)])
    q.fill(fork * g, 'dirt', ink=0.35, t=0.3)
    q.R('woodd', -2.2, -29, 2.2, 25.5, mode='cyl'); q.P('woodd', [(-3, -29), (0, -32), (3, -29)], t=0.3)
    q.P('wood', [(-2, -26), (17, -26), (25, -20.5), (17, -15), (-2, -15)], a=0.1, bu=0.4, bv=0.2)
    q.P('wood', [(2, -11), (-17, -11), (-25, -5.5), (-17, 0), (2, 0)], a=0.1, bu=0.4, bv=0.2)
    q.P('wood', [(-2, 4), (12, 4), (18, 8), (12, 12), (-2, 12)], a=0.2, bu=0.4, bv=0.2)
    q.lines([((2, -22.5), (15, -22.5)), ((2, -18.5), (15, -18.5)), ((-15, -7.5), (-2, -7.5)), ((-15, -3.5), (-2, -3.5)), ((2, 8), (11, 8))], a=0.45)
    q.tufts(14)

@pin('caravan_station')
def _(q):
    # A covered wagon standing under a waystation roof, a lantern at the eave.
    q.mound('dirt', 23)
    q.R('woodd', -30, -6, -26.5, 24.5, mode='cyl'); q.R('woodd', 26.5, -6, 30, 24.5, mode='cyl')
    bon = [(-15 + 14.5 - 14.5 * math.cos(a), 9 - 16 * math.sin(a)) for a in np.linspace(0, math.pi, 20)]
    q.P('canvas', bon, a=0.05, bu=0.5, bv=0.1)
    q.lines([((x, -7), (x, 9)) for x in (-9, -0.5, 8)], a=0.35, clip=q.mp(bon))
    q.R('wood', -16.5, 9, 15.5, 14.5, a=0.15, bu=0.4)
    q.wheel(-8.5, 18.5, 6.6); q.wheel(8, 18.5, 6.6)
    q.P('thatch', [(-36, -6), (0, -29), (36, -6)], a=0.08, bu=0.5, bv=0.1)
    q.lines([((x, -6), (x * 0.3, -22)) for x in (-26, -17, -8, 8, 17, 26)], a=0.28, clip=q.mp([(-36, -6), (0, -29), (36, -6)]))
    q.R('woodd', -33.5, -8, 33.5, -4.6, t=0.45)
    q.lines([((21, -4.6), (21, -1))], w=0.45, a=1)
    q.glow(21, 2, 7, (130, 84, 20), 0.7)
    q.R('woodd', 18.6, -1.6, 23.4, -0.4, ink=0.25); q.E('gold', 21, 2.2, 2.6, 3.0, ink=0.3, t=0.02); q.put(q.me(21, 2.4, 1.2, 1.6), (255, 246, 200))

@pin('trade_depot')
def _(q):
    # A timber warehouse with goods stacked outside.
    q.mound('dirt', 23)
    q.R('wood', -28, -2, 8, 24.5, a=0.12, bu=0.4); q.planks(-28, -2, 8, 24.5)
    q.P('shingle', [(-32, -1), (-10, -21), (12, -1)], a=0.1, bu=0.55)
    q.R('woodd', -20, 6, 0, 24.5, t=0.5); q.dark(q.mr(-18.4, 7.6, -1.6, 24.5)); q.win(-12.5, -12, -7.5, -6)
    q.R('wood', 10.5, 13, 21.5, 24.5, a=0.1); q.R('wood', 21.5, 16, 30, 24.5, a=0.3); q.R('wood', 13.5, 3.5, 22.5, 13, a=0.15)
    q.lines([((10.5, 13), (21.5, 24.5)), ((21.5, 13), (10.5, 24.5)), ((21.5, 16), (30, 24.5)), ((30, 16), (21.5, 24.5)), ((13.5, 3.5), (22.5, 13)), ((22.5, 3.5), (13.5, 13))], a=0.45)
    sx = 4.5                                                  # a tied sack and a barrel, standing on the ground
    q.P('thatch', [(sx - 3.8, 25), (sx - 4.3, 19.5), (sx - 1.6, 16.6), (sx - 2.2, 14.8), (sx + 2.2, 14.8), (sx + 1.6, 16.6), (sx + 4.3, 19.5), (sx + 3.8, 25)], a=0.1, bu=0.5, bv=0.2)
    q.lines([((sx - 1.7, 16.6), (sx + 1.7, 16.6))], w=0.6, a=0.7)
    q.R('woodd', -8.5, 16.5, -2.5, 25, mode='cyl'); q.lines([((-8.5, 18.6), (-2.5, 18.6)), ((-8.5, 23), (-2.5, 23))], w=0.7, a=0.9, c=(70, 70, 78))

@pin('ancient_road')
def _(q):
    # A paved road passing under a ruined archway (owner's glyph pick).
    g = q.mound('grass', 23)
    hw = lambda y: 6.5 + (y - 21) * 0.95                        # the road lies on the ground and runs from the arch to the rim
    road = q.mp([(-hw(21), 21), (hw(21), 21), (hw(42), 42), (-hw(42), 42)]) * q.disc
    q.fill(road, 'dirt', a=0.12, bu=0.3, bv=0.25, ink=0.4); q.path = (-11, 11)
    rows = [21, 24.5, 28.5, 33, 38, 42]
    segs = [((-hw(y), y), (hw(y), y)) for y in rows[1:-1]]
    for i in range(5):
        for f in ((-0.3, 0.35) if i % 2 else (0.05,)): segs.append(((f * hw(rows[i]), rows[i]), (f * hw(rows[i + 1]), rows[i + 1])))
    q.lines(segs, a=0.45, clip=road)
    ro, ri, cy = 29, 18, -3
    ring = [(ro * math.cos(math.radians(a)), cy + ro * math.sin(math.radians(a))) for a in range(-180, -49, 5)]
    inner = [(ri * math.cos(math.radians(a)), cy + ri * math.sin(math.radians(a))) for a in range(-56, -181, -5)]
    arch = ring + [(16.5, -21), (13, -19.5)] + inner
    q.P('stone', arch, a=0.08, bu=0.5, bv=0.1)
    q.lines([((math.cos(math.radians(a)) * ri, cy + math.sin(math.radians(a)) * ri), (math.cos(math.radians(a)) * ro, cy + math.sin(math.radians(a)) * ro)) for a in range(-166, -56, 14)], a=0.4, clip=q.mp(arch))
    q.R('stone', -29, cy, -18, 24.5, a=0.08, bu=0.5); q.joints(-29, cy, -18, 24.5, rh=5.5, bw=20)
    q.R('stone', -31.5, cy - 2, -16, cy + 2, t=0.08); q.R('stone', -31.5, 21, -15.5, 25, t=0.15)
    rp = [(18, 24.5), (18, 1), (21.5, -4), (24.5, 2), (29, -2), (29, 24.5)]
    q.P('stone', rp, a=0.3, bu=0.4); q.lines([((18, y), (29, y)) for y in (7, 12.5, 18)], a=0.38, clip=q.mp(rp)); q.R('stone', 16, 21, 31.5, 25, t=0.4)
    q.boulder(34, 24.5, 3.4, 2.4, 'stone'); q.boulder(-35, 25, 2.6, 2, 'stone')
    q.tufts(10)


# ---- military ---------------------------------------------------------------------
@pin('fort')
def _(q):
    # A timber stockade with a gate, a lookout and a pennant: wood and points,
    # not stone and crenels, so it cannot be read as the owner's castle.
    q.mound('grass', 23)
    for x in (-28.5, -17.5): q.L('woodd', [(x, -6), (x, 6)], 1.6, ink=0.3, t=0.5)
    q.R('wood', -30.5, -19, -15.5, -5, a=0.12, bu=0.4); q.planks(-30.5, -19, -15.5, -5); q.win(-25.5, -15, -20.5, -10)
    q.P('thatch', [(-33.5, -18), (-23, -30), (-12.5, -18)], a=0.08, bu=0.55)
    q.L('woodd', [(20, -29), (20, 0)], 1.4, ink=0.3, t=0.45); q.P('red', [(20.6, -29), (30.5, -25.5), (20.6, -22)], ink=0.35)
    for i in range(10):
        x0 = -31 + i * 6.2
        q.P('wood', [(x0 + 0.2, 25), (x0 + 0.2, 2), (x0 + 3.1, -5.5), (x0 + 6.0, 2), (x0 + 6.0, 25)], mode='cyl', ink=0.4)
    for y in (9, 18): q.lines([((-31, y), (31, y))], w=1.0, a=0.9, c=(214, 186, 128)); q.lines([((-31, y + 0.8), (31, y + 0.8))], w=0.3, a=0.5)
    q.R('woodd', -11.5, -12, -6.5, 25, mode='cyl'); q.R('woodd', 6.5, -12, 11.5, 25, mode='cyl'); q.R('woodd', -13.5, -15, 13.5, -10, t=0.35)
    q.R('wood', -6.5, -6, 6.5, 25, a=0.25, bu=0.3); q.planks(-6.5, -6, 6.5, 25, step=2.6)
    q.dark(q.mr(-6.5, -10, 6.5, -6)); q.lines([((-6.5, 4), (6.5, 4)), ((-6.5, 16), (6.5, 16))], w=1.0, a=0.9, c=(70, 70, 78)); q.lines([((0, -6), (0, 25))], w=0.5, a=0.8)
    q.tufts(12)

@pin('watchtower')
def _(q):
    # A timber lookout on braced legs.
    q.mound('grass', 23)
    for a, b in (((-10, 2), (-16, 24.5)), ((10, 2), (16, 24.5))): q.L('woodd', [a, b], 2.6, ink=0.4, t=0.45)
    for a, b in (((-11.5, 7), (14.5, 19)), ((11.5, 7), (-14.5, 19)), ((-13, 13), (13, 13))): q.L('woodd', [a, b], 1.5, ink=0.3, t=0.55)
    q.R('wood', -18, -1.5, 18, 3, a=0.1, bu=0.4)
    q.R('wood', -12.5, -15, 12.5, -1.5, a=0.12, bu=0.45); q.planks(-12.5, -15, 12.5, -1.5)
    q.win(-8, -11, -2, -5); q.win(2, -11, 8, -5)
    q.P('shingle', [(-18.5, -14), (0, -31), (18.5, -14)], a=0.1, bu=0.55)
    q.lines([((x, -14), (x * 0.25, -27)) for x in (-12, -6, 6, 12)], a=0.3, clip=q.mp([(-18.5, -14), (0, -31), (18.5, -14)]))
    q.tufts(14)

@pin('fortified_pass')
def _(q):
    # A wall and gate tower closing the gap between two peaks.
    q.mound('grass', 23)
    q.peak(22, -21, 5, 40, 24); q.peak(-22, -27, -40, -5, 24)
    q.wall(-21, 21, 9, 24.5, n=6)
    q.tower(-8, 8, -6, 24.5, n=3); q.door(0, 4.8, 11, 24.5); q.win(-1.2, -1, 1.2, 5)
    q.path = (-7, 7)
    q.P('dirt', [(-5, 24.5), (5, 24.5), (8, 31), (-8, 31)], ink=0, t=0.35)
    q.tufts(10)

@pin('fortified_crossing')
def _(q):
    # A bridge with a gate tower standing on it, banners flying.
    deck = q.mr(-42, 5, 42, 19)
    q.R('stone', -42, 5, 42, 19, a=0.14, bu=0.35, bv=0.2); q.joints(-42, 5, 42, 19, rh=4.6, bw=8)
    for x0 in (-38, -30, -22, 15, 23, 31): q.R('stone', x0, 1.8, x0 + 5, 5.4, a=0.1, bu=0.3)
    for x in (-25, 25):
        q.fill(q.march(x, 10.3, 7.7, 19) * deck, 'stone', t=0.12, ink=0); q.dark(q.march(x, 8.5, 9.5, 19))
    q.tower(-10, 10, -17, 19, n=3); q.door(0, 4.6, -4, 5); q.win(-1.2, -12, 1.2, -7)
    for x in (-7, 7):
        q.L('woodd', [(x, -23), (x, -33)], 1.1, ink=0.3, t=0.45); q.P('blue', [(x + 0.5, -33), (x + 8, -30.5), (x + 0.5, -28)], ink=0.35, a=0.15)
    q.water(15)
    for x in (-25, 0, 25): q.put(q.me(x, 16, 6.5, 1.1), (220, 236, 246), 0.6)

@pin('battlefield')
def _(q):
    # The aftermath: swords driven into a churned mound, a shield, a torn banner.
    q.mound('dirt', 21)
    q.L('woodd', [(-25, -25), (-23, 23)], 1.5, ink=0.35, t=0.45)
    q.P('red', [(-24.5, -25), (-9, -22.5), (-13, -17.5), (-8, -12), (-12, -8), (-24, -9.5)], a=0.1, bu=0.5, bv=0.2, ink=0.4)
    def sword(top, tip):
        dx, dy = tip[0] - top[0], tip[1] - top[1]; n = math.hypot(dx, dy); ux, uy = dx / n, dy / n; px, py = -uy, ux
        at = lambda t, o=0.0: (top[0] + ux * t + px * o, top[1] + uy * t + py * o)
        q.P('metal', [at(11, 2.0), at(n, 1.6), at(n, -1.6), at(11, -2.0)], a=0.05, bu=0.7, ink=0.4); q.lines([(at(12), at(n))], a=0.4)
        q.L('gold', [at(10.5, -6), at(10.5, 6)], 2.2, ink=0.35); q.L('woodd', [at(2.5), at(10)], 2.2, ink=0.3, t=0.4); q.E('gold', *at(1.2), 2.1, 2.1, ink=0.3)
    sword((0, -27), (9, 23)); sword((24, -24), (11, 23))
    q.L('woodd', [(-36, 20), (-8, 27)], 1.3, ink=0.3, t=0.45); q.P('metal', [(-8, 27), (-3.5, 26.2), (-7.2, 29.4)], ink=0.3, t=0.2)
    q.E('wood', -9, 17.5, 8, 8); q.E('woodd', -9, 17.5, 8, 8, ink=0, t=0.4); q.E('wood', -9, 17.5, 6.3, 6.3, ink=0.25); q.E('metal', -9, 17.5, 2.3, 2.3, ink=0.3)
    q.lines([((-9, 11.2), (-9, 23.8)), ((-15.3, 17.5), (-2.7, 17.5))], a=0.4, clip=q.me(-9, 17.5, 6.3, 6.3))
    q.tufts(8, top=22, cols=((140, 130, 70), (86, 84, 44)))

@pin('border_marker')
def _(q):
    # A squat boundary stone with a carved device and a painted band, smaller
    # stones marking the line to either side (not the monument's tall obelisk).
    q.mound('grass', 23)
    for x, y, r in ((-30, 24.5, 3.2), (-20, 25.5, 3.6), (22, 25.5, 3.6), (31, 24.5, 3.2)): q.boulder(x, y, r, r * 0.8, 'stone', n=7)
    q.P('stone', [(-15, 25), (-13.5, 20), (7, 20), (8.5, 25)], a=0.1, bu=0.3); q.P('stone', [(8.5, 25), (7, 20), (13, 17.5), (15, 22.5)], t=0.7)
    q.P('stone', [(-11, 20), (-10, -12), (5, -12), (6, 20)], a=0.08, bu=0.3, bv=0.15)
    q.P('stone', [(6, 20), (5, -12), (11, -15), (12, 17.5)], t=0.72)
    q.P('stone', [(-10, -12), (-1, -25), (5, -12)], a=0.03, bu=0.25); q.P('stone', [(5, -12), (-1, -25), (11, -15)], t=0.55)
    q.P('red', [(-10.7, 9), (5.6, 9), (5.8, 14), (-10.9, 14)], a=0.1, bu=0.4, ink=0.25); q.P('red', [(5.6, 9), (11.6, 6.3), (11.8, 11.3), (5.8, 14)], t=0.75, ink=0.25)
    q.lines([((-2.5, -8), (2, -2.5)), ((2, -2.5), (-2.5, 3)), ((-2.5, 3), (-7, -2.5)), ((-7, -2.5), (-2.5, -8)), ((-2.5, -5), (-2.5, 0))], w=0.8, a=0.7)
    q.tufts(14)

# ---- sacred and memorial --------------------------------------------------------
@pin('sacred_grove')
def _(q):
    # A ring of standing stones under old trees, a faint light among them.
    q.mound('grass', 23)
    q.pine(0, 18, 38); q.pine(-23, 24.5, 37); q.pine(23, 24.5, 37)
    q.glow(0, 14, 13, (40, 70, 60), 0.5)
    for sx, t in ((-1, 0.0), (1, 0.25)):
        q.P('stone', [(sx * 31, 25), (sx * 30, 15), (sx * 27.5, 12.5), (sx * 24.5, 14.5), (sx * 24, 25)][::sx], a=0.1 + t, bu=0.5)
        q.P('stone', [(sx * 20, 25), (sx * 19.5, 10), (sx * 17, 5.5), (sx * 14, 8.5), (sx * 13.5, 25)][::sx], a=0.08 + t, bu=0.5)
    q.P('stone', [(-6.5, 25), (-6, 3), (-2, -6), (4, -3), (6, 4), (6.5, 25)], a=0.08, bu=0.5)
    q.lines([((-2, 2), (1, 8)), ((1, 8), (-1.5, 14)), ((-1.5, 14), (0.5, 19))], w=0.6, a=0.6)
    q.star(-1, -11, 3.6, (236, 250, 230)); q.star(15, -2, 2.4, (236, 250, 230)); q.star(-17, 0, 2.2, (236, 250, 230))
    q.tufts(14)

@pin('sacred_mountain')
def _(q):
    # A lone snow peak with a light breaking from its summit.
    q.mound('grass', 23)
    q.glow(0, -22, 15, (60, 100, 150), 0.75)
    segs = [((9 * math.cos(math.radians(a)), -22 + 9 * math.sin(math.radians(a))), (17 * math.cos(math.radians(a)), -22 + 17 * math.sin(math.radians(a)))) for a in range(-170, -9, 20)]
    q.lines(segs, w=1.0, a=0.75, c=(206, 232, 255))
    q.peak(0, -22, -35, 35, 24, snow=0.44)
    q.star(0, -24, 5, (240, 250, 255))
    q.pine(-27, 25, 12); q.pine(28, 25, 12); q.bush(-17, 22.5, 3.6, n=5); q.bush(18, 22.5, 3.6, n=5)
    q.tufts(8)

@pin('ceremonial_site')
def _(q):
    # A trilithon between standing stones, a fire burning on the stone floor.
    q.mound('grass', 23)
    q.E('stone', 0, 24.5, 27, 4.6, mode='flat', a=0.1, bu=0.4)
    q.P('rock', [(-32, 24), (-31, 6), (-27.5, 2), (-24, 5), (-23, 24)], a=0.1, bu=0.5); q.P('rock', [(23, 24), (24, 8), (28, 4), (31, 8), (32, 24)], a=0.3, bu=0.5)
    q.glow(0, 14, 16, (120, 60, 14), 0.6)
    q.P('rock', [(-18, 24), (-17, -11), (-8.5, -11), (-9, 24)], mode='cyl'); q.P('rock', [(9, 24), (8.5, -11), (17, -11), (18, 24)], mode='cyl')
    q.P('rock', [(-22, -11), (-21, -20.5), (20, -21.5), (22, -11)], a=0.08, bu=0.45, bv=0.25)
    q.lines([((-14, -4), (-12, 6)), ((13, 0), (12, 12)), ((-6, -20), (-4, -12)), ((8, -20), (9, -12))], a=0.35)
    for x in (-5, 0, 5): q.boulder(x, 25, 2.8, 1.8, n=7)
    q.flame(0, 24, 17, 6.2)
    q.tufts(10)

@pin('tomb')
def _(q):
    # A stone mausoleum with a dark doorway, a broken pillar beside it.
    q.mound('grass', 23)
    q.P('stone', [(-33, 24.5), (-33, 10), (-31, 7), (-29, 10.5), (-27, 8), (-26, 24.5)], mode='cyl'); q.R('stone', -34.5, 22, -24.5, 25, t=0.2)
    q.R('stone', -19, -6, 19, 24.5, a=0.1, bu=0.45); q.joints(-19, -6, 19, 24.5, rh=5, bw=8)
    q.R('stone', -9, 1, 9, 24.5, t=0.1); q.R('stone', -11, -1.5, 11, 2, t=0.05)
    q.dark(q.mr(-6, 4, 6, 24.5)); q.put(q.mr(-6, 4, 6, 7), (8, 6, 5), 0.6)
    q.R('stone', -22, -10.5, 22, -5.5, a=0.05, bu=0.4)
    q.P('stone', [(-21, -10.5), (0, -22), (21, -10.5)], a=0.06, bu=0.45)
    tym = q.mp([(-13, -12), (0, -19), (13, -12)]); q.line(tym, INK, 0.3); q.put(tym, q.col('stone', np.full_like(q.X, 0.68)))
    q.E('stone', 0, -14.3, 2.2, 2.2, ink=0.3)
    q.R('stone', -23, 21, 23, 25.5, a=0.1, bu=0.4); q.lines([((-23, 23.2), (23, 23.2))], a=0.4)
    q.path = (-7, 7)
    q.P('dirt', [(-5, 25.5), (5, 25.5), (8, 31), (-8, 31)], ink=0, t=0.35)
    q.bush(27, 21.5, 4.6, n=6); q.tufts(12)


# ---- assembly ----------------------------------------------------------------------
ORDER = ("city hamlet tavern smithy shrine guild gate well camp other spring river_confluence delta ford portage historic_crossing "
         "mountain_pass ridge cliff gorge rock_formation glacier quarry salt_works road_junction caravan_station trade_depot ancient_road "
         "fort watchtower fortified_pass fortified_crossing battlefield border_marker sacred_grove sacred_mountain ceremonial_site tomb inn").split()

def body_and_mask():
    body = np.median(T.stack(T.LOOK_A), 0)
    ys, xs = np.mgrid[0:128, 0:107]
    return body, np.clip(41.6 - np.hypot(xs - OX, ys - OY), 0, 1).astype(np.float32)

def render(name, body, mask):
    f, seed = PINS[name]
    q = Q(seed + sum(map(ord, name)))
    f(q); q.seat()
    return T.compose(body, q.finish(), mask)

def contact(pins, owner, path, h, bg, cols=10):
    from PIL import ImageDraw as D
    w = round(107 * h / 128); cw, ch = max(w + 14, 104), h + 24
    ro, rp = math.ceil(len(owner) / cols), math.ceil(len(pins) / cols)
    sheet = Image.new('RGBA', (cols * cw + 16, (ro + rp) * ch + 44), bg + (255,)); d = D.Draw(sheet); f = T.font(10)
    def put(items, y0):
        for i, (nm, im) in enumerate(items):
            x, y = 8 + (i % cols) * cw, y0 + (i // cols) * ch
            t = im if h == 128 else im.resize((w, h), Image.LANCZOS)
            sheet.alpha_composite(t, (x + (cw - w) // 2, y))
            d.text((x + cw / 2, y + h + 2), nm, fill=(20, 20, 20) if sum(bg) > 300 else (235, 235, 235), font=f, anchor='ma')
    put(owner, 6); y1 = 6 + ro * ch + 4
    d.line([(8, y1 + 6), (sheet.width - 8, y1 + 6)], fill=(0, 0, 0, 120), width=1)
    put(pins, y1 + 16); sheet.save(path)

def main():
    only = sys.argv[1:]
    body, mask = body_and_mask()
    for name in ORDER:
        if only and name not in only: continue
        render(name, body, mask).save(os.path.join(HERE, name + ".png"))
    shutil.copyfile(os.path.join(HERE, "..", "painted_temple", "temple_A.png"), os.path.join(HERE, "temple.png"))
    owner = [(n, Image.open(T.PINS + n + ".png").convert("RGBA")) for n in T.LOOK_A]
    pins = [(n, Image.open(os.path.join(HERE, n + ".png")).convert("RGBA")) for n in ORDER + ["temple"] if os.path.exists(os.path.join(HERE, n + ".png"))]
    contact(pins, owner, os.path.join(HERE, "painted_A_128.png"), 128, (52, 54, 60))
    contact(pins, owner, os.path.join(HERE, "painted_A_32.png"), 32, (150, 158, 122))
    print(len(pins), "pins")

if __name__ == "__main__":
    main()

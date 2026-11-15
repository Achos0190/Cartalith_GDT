"""A coded attempt at a `temple` pin for the PAINTED set (owner, 2026-11-15:
"let's try a temple"). Two versions, one per look of that set:

    temple_A.png  dark disc + bronze frame, kin of monument / market (107 x 128)
    temple_B.png  landscape scene + gold frame, kin of shrine / tomb  (96 x 128)

Pin body (frame, outline, alpha, tip)
    As for the glyph set, the body is the per-pixel MEDIAN of existing pins of
    that look, so the silhouette and hit shape are the set's own. Look A's 20
    pins are all 107 x 128 and pixel-aligned. Look B's 38 extras come in 13
    widths (90..101); the eight that are 96 px wide (tomb among them) are
    aligned with each other, so B uses those and is 96 x 128.
    Only the picture opening is repainted: a circle for A (measured), and for B
    the region that is neither gold frame nor dark outline in most of the pins.

The picture
    Painted procedurally at SS x: gradients for form (light from the upper
    left, as in the set), value noise for stone and foliage texture, dark
    contour lines, then a grain-plus-median pass that breaks flat fills into
    dabs. It is NOT hand painting and the report says how far it falls short.

Must never write outside this folder or touch the repository.
"""
import math, os
import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
PINS = "C:/Users/Vincent/Cartalith_GDT/cartalith-native/godot-project/icons/pins/painted/"
LOOK_A = 'bridge capital castle cave desert farmland forest harbour hazard lake market mine monument mountain ruin town village volcano waterfall wildlife'.split()
LOOK_B = 'caravan_station glacier gorge portage road_junction tavern tomb trade_depot'.split()   # the 96 px wide extras
SS = 6


def lerp(a, b, t):
    """Blend colour a to b by t (scalar or HxW array)."""
    a = np.asarray(a, np.float32); b = np.asarray(b, np.float32)
    t = np.asarray(t, np.float32)
    if t.ndim == 2: t = t[..., None]
    return a + (b - a) * t


class Paint:
    """A float RGB canvas SS times the pin size; coordinates are pin pixels."""

    def __init__(s, w, h, seed):
        s.w, s.h = w, h
        s.img = np.zeros((h * SS, w * SS, 3), np.float32)
        s.Y, s.X = np.mgrid[0:h * SS, 0:w * SS].astype(np.float32) / SS
        s.rng = np.random.default_rng(seed)

    def fbm(s, cell, octaves=3):
        """Value noise in 0..1 whose largest blobs are `cell` pin pixels wide."""
        out = np.zeros((s.h * SS, s.w * SS), np.float32); amp, tot = 1.0, 0.0
        for o in range(octaves):
            c = max(cell / (2 ** o), 0.4)
            small = s.rng.random((max(2, int(s.h / c) + 2), max(2, int(s.w / c) + 2))).astype(np.float32)
            big = Image.fromarray((small * 255).astype(np.uint8)).resize((s.w * SS, s.h * SS), Image.BICUBIC)
            out += amp * np.asarray(big, np.float32) / 255; tot += amp; amp *= 0.5
        return out / tot

    def poly(s, pts):
        m = Image.new('L', (s.w * SS, s.h * SS), 0)
        ImageDraw.Draw(m).polygon([(x * SS, y * SS) for x, y in pts], fill=255)
        return np.asarray(m, np.float32) / 255

    def rect(s, x0, y0, x1, y1):
        return s.poly([(x0, y0), (x1, y0), (x1, y1), (x0, y1)])

    def ell(s, cx, cy, rx, ry):
        return (((s.X - cx) / rx) ** 2 + ((s.Y - cy) / ry) ** 2 <= 1).astype(np.float32)

    def grow(s, m, px):
        """Dilate a mask by px pin pixels (for contour lines and cast shadows)."""
        k = int(round(px * SS)) * 2 + 1
        return np.asarray(Image.fromarray((m * 255).astype(np.uint8)).filter(ImageFilter.MaxFilter(k)), np.float32) / 255

    def put(s, m, col, a=1.0):
        """Lay colour (tuple or HxWx3) through mask m at opacity a."""
        col = np.asarray(col, np.float32)
        mm = (m * a)[..., None]
        s.img = s.img * (1 - mm) + col * mm

    def line(s, m, col, px=0.55):
        """A dark contour around mask m, painted before the shape itself."""
        s.put(s.grow(m, px), col)

    def finish(s):
        """Grain, then a median: flat fills break into small dabs; then downsample."""
        g = s.img + s.rng.normal(0, 7, s.img.shape).astype(np.float32)
        im = Image.fromarray(np.clip(g, 0, 255).astype(np.uint8)).filter(ImageFilter.MedianFilter(5))
        im = im.resize((s.w, s.h), Image.LANCZOS)
        return np.asarray(im, np.float32)


def temple(p, cx, base, W, pal):
    """A classical tetrastyle temple front: three steps, four columns, a dark
    cella with a door, entablature with triglyphs, pediment with a tympanum.

    cx, base: centre and ground line in pin pixels; W: width across the columns.
    pal: light / mid / shade / deep stone colours, contour `ink`, `cella`, `door`.
    Light comes from the upper left: left faces and tops are light, the right
    side of every column and everything under a projection is in shade."""
    u = W / 62.0
    X, Y = p.X, p.Y
    tex = 0.80 + 0.28 * p.fbm(5 * u) + 0.14 * p.fbm(1.2)          # stone mottling
    stain = np.clip((p.fbm(7 * u) - 0.52) * 4, 0, 1) * 0.30       # weather stains, warm brown
    moss = np.clip((p.fbm(3 * u) - 0.45) * 4, 0, 1) * np.clip(1 - (base - Y) / (9 * u), 0, 1) * 0.45   # green at the foot
    def stone(t):                                                # t: 0 light .. 1 deep shade
        t = np.clip(t, 0, 1)
        c = np.where(t[..., None] < 0.5, lerp(pal['light'], pal['mid'], t * 2), lerp(pal['mid'], pal['shade'], t * 2 - 1))
        c = lerp(c, np.asarray(pal['stain'], np.float32) * (1.15 - 0.5 * t)[..., None], stain)
        c = lerp(c, np.asarray((96, 116, 60), np.float32) * (1.1 - 0.5 * t)[..., None], moss)
        return c * tex[..., None]
    ys = base - 10.2 * u                                         # top of the steps
    yc = ys - 25 * u                                             # top of the columns
    ye = yc - 6.5 * u                                            # top of the frieze
    hw = W / 2
    # --- silhouette contour first, so every part gets a dark edge
    sil = np.maximum.reduce([p.rect(cx - hw - 5 * u, base - 3.4 * u, cx + hw + 5 * u, base),
                             p.rect(cx - hw - 2.5 * u, ys, cx + hw + 2.5 * u, base),
                             p.rect(cx - hw + 1 * u, yc, cx + hw - 1 * u, ys),
                             p.rect(cx - hw - 1.5 * u, ye - 1.4 * u, cx + hw + 1.5 * u, yc),
                             p.poly([(cx - hw - 1.5 * u, ye - 1.2 * u), (cx, ye - 14.4 * u), (cx + hw + 1.5 * u, ye - 1.2 * u)])])
    p.line(sil, pal['ink'], 0.85)
    # --- cella (the dark room behind the columns) and its door
    cel = p.rect(cx - hw + 1 * u, yc, cx + hw - 1 * u, ys)
    p.put(cel, lerp(pal['cella'], pal['door'], np.clip((Y - yc) / (8 * u), 0, 1) * 0.0 + np.clip(1 - (Y - yc) / (7 * u), 0, 1) * 0.6))
    door = p.rect(cx - 3.6 * u, ys - 15 * u, cx + 3.6 * u, ys)
    p.put(p.grow(door, 0.9 * u), lerp(pal['mid'], pal['shade'], 0.6) * 0.8); p.put(door, pal['door'])
    # --- steps: light tread on top, shaded riser, a few joints
    for i in range(3):
        x0, x1 = cx - hw - (5 - 2.5 * i) * u, cx + hw + (5 - 2.5 * i) * u
        y1 = base - 3.4 * u * i; y0 = y1 - 3.4 * u
        m = p.rect(x0, y0, x1, y1)
        p.line(m, pal['ink'], 0.35)
        t = 0.18 + 0.5 * np.clip((Y - y0) / (3.4 * u), 0, 1) + 0.25 * np.clip((X - cx) / hw, -1, 1) * 0.5
        p.put(m, stone(t))
        p.put(p.rect(x0, y0, x1, y0 + 0.9 * u), stone(np.zeros_like(X)), 0.85)       # sunlit tread edge
        for k in range(-3, 4):
            jx = cx + (k + 0.5 * (i % 2)) * 9.5 * u
            if x0 + 2 < jx < x1 - 2: p.put(p.rect(jx - 0.25, y0 + 1 * u, jx + 0.25, y1), pal['ink'], 0.55)
    # --- columns
    cw = 6.4 * u; pitch = (W - 6 * u - cw) / 3
    for i in range(4):
        x = cx + (i - 1.5) * pitch
        shaft = p.rect(x - cw / 2, yc + 1.8 * u, x + cw / 2, ys - 1.6 * u)
        cap = p.rect(x - cw / 2 - 1.3 * u, yc, x + cw / 2 + 1.3 * u, yc + 1.9 * u)
        ech = p.poly([(x - cw / 2 - 1.0 * u, yc + 1.8 * u), (x + cw / 2 + 1.0 * u, yc + 1.8 * u), (x + cw / 2, yc + 3.4 * u), (x - cw / 2, yc + 3.4 * u)])
        foot = p.rect(x - cw / 2 - 1.1 * u, ys - 1.7 * u, x + cw / 2 + 1.1 * u, ys)
        p.line(np.maximum.reduce([shaft, cap, foot, ech]), pal['ink'], 0.55)
        r = (X - x) / (cw / 2)                                   # -1 left edge .. +1 right edge
        # a cylinder lit from the upper left: brightest at r = -0.35, a core
        # shadow at r = +0.6 and a little bounce light at the right edge
        t = 0.10 + 0.55 * np.clip((r + 0.35) / 0.95, 0, 1) ** 1.2 + 0.30 * np.clip((-0.35 - r) / 0.65, 0, 1)
        t = t - 0.14 * np.clip((r - 0.72) / 0.28, 0, 1)
        t = t + 0.20 * np.clip(1 - (Y - (yc + 1.8 * u)) / (5 * u), 0, 1)           # shadow of the entablature
        t = t + 0.07 * (np.sin(r * 9.5) > 0.55)                                    # fluting
        p.put(shaft, stone(t))
        p.put(cap, stone(0.12 + 0.35 * np.clip(r * 0.5 + 0.3, 0, 1)))
        p.put(ech, stone(0.35 + 0.35 * np.clip(r * 0.5 + 0.5, 0, 1)))
        p.put(foot, stone(0.2 + 0.4 * np.clip(r * 0.5 + 0.4, 0, 1)))
        p.put(p.rect(x - cw / 2 - 1.3 * u, yc + 1.75 * u, x + cw / 2 + 1.3 * u, yc + 2.1 * u), pal['ink'], 0.5)
    # --- entablature: architrave, frieze with triglyphs, projecting cornice
    arch = p.rect(cx - hw - 0.2 * u, yc - 3.2 * u, cx + hw + 0.2 * u, yc)
    fr = p.rect(cx - hw - 0.2 * u, ye, cx + hw + 0.2 * u, yc - 3.2 * u)
    cor = p.rect(cx - hw - 1.5 * u, ye - 1.4 * u, cx + hw + 1.5 * u, ye)
    side = np.clip((X - cx) / hw, -1, 1) * 0.12
    p.put(arch, stone(0.16 + side + 0.25 * np.clip((Y - (yc - 3.2 * u)) / (3.2 * u), 0, 1)))
    p.put(fr, stone(0.42 + side))
    for k in range(-5, 6):
        tx = cx + k * 5.4 * u
        p.put(p.rect(tx - 1.0 * u, ye + 0.3 * u, tx + 1.0 * u, yc - 3.4 * u), stone(0.2 + side))
        p.put(p.rect(tx - 0.2 * u, ye + 0.4 * u, tx + 0.2 * u, yc - 3.5 * u), pal['ink'], 0.55)
    p.put(p.rect(cx - hw - 0.2 * u, yc - 3.4 * u, cx + hw + 0.2 * u, yc - 3.0 * u), pal['ink'], 0.5)
    p.put(cor, stone(0.05 + side))
    p.put(p.rect(cx - hw - 0.2 * u, ye, cx + hw + 0.2 * u, ye + 0.7 * u), pal['ink'], 0.6)     # shadow under the cornice
    # --- pediment: raking cornice (light), recessed tympanum (shade), a boss
    apex = ye - 14.4 * u
    outer = p.poly([(cx - hw - 1.5 * u, ye - 1.2 * u), (cx, apex), (cx + hw + 1.5 * u, ye - 1.2 * u)])
    inner = p.poly([(cx - hw + 5.5 * u, ye - 2.6 * u), (cx, apex + 3.6 * u), (cx + hw - 5.5 * u, ye - 2.6 * u)])
    p.put(outer, stone(0.08 + 0.30 * np.clip((X - cx) / hw, 0, 1)))                # right rake a little darker
    p.line(inner, pal['ink'], 0.3)
    p.put(inner, stone(0.62 + 0.25 * np.clip(1 - (Y - apex) / (9 * u), 0, 1) - 0.15 * np.clip((X - cx) / hw, -1, 0) * -1))
    boss = p.ell(cx, ye - 5.6 * u, 2.3 * u, 2.3 * u)
    p.line(boss, pal['ink'], 0.3); p.put(boss, stone(0.1 + 0.5 * np.clip(((X - cx) + (Y - (ye - 5.6 * u))) / (4 * u) + 0.4, 0, 1)))
    for ax, ay, r in ((cx, apex - 0.6 * u, 1.9 * u), (cx - hw - 1.0 * u, ye - 2.4 * u, 1.5 * u), (cx + hw + 1.0 * u, ye - 2.4 * u, 1.5 * u)):
        ac = p.ell(ax, ay, r, r * 1.15)                                            # acroteria
        p.line(ac, pal['ink'], 0.4); p.put(ac, stone(0.1 + 0.5 * np.clip((X - ax) / r * 0.5 + 0.4, 0, 1)))
    # a few cracks and chips, so the stone is not factory-new
    for _ in range(7):
        x0 = cx + p.rng.uniform(-hw, hw); y0 = p.rng.uniform(apex + 6 * u, base - 2 * u)
        pts = [(x0, y0)]
        for _ in range(3): pts.append((pts[-1][0] + p.rng.uniform(-1.4, 1.4) * u, pts[-1][1] + p.rng.uniform(0.8, 2.2) * u))
        m = Image.new('L', (p.w * SS, p.h * SS), 0)
        ImageDraw.Draw(m).line([(x * SS, y * SS) for x, y in pts], fill=255, width=max(1, int(0.35 * SS)))
        p.put(np.asarray(m, np.float32) / 255 * sil, pal['ink'], 0.35)


def bush(p, cx, cy, r, dark, light, ink, n=7):
    """A clump of foliage: overlapping lit blobs with a dark contour."""
    blobs = [(cx + p.rng.uniform(-r, r), cy + p.rng.uniform(-r * 0.6, r * 0.5), p.rng.uniform(0.30, 0.55) * r) for _ in range(n)]
    allm = np.maximum.reduce([p.ell(x, y, q, q * 0.9) for x, y, q in blobs])
    p.line(allm, ink, 0.5)
    for x, y, q in sorted(blobs, key=lambda b: b[1]):
        m = p.ell(x, y, q, q * 0.9)
        t = np.clip(((p.X - x) + (p.Y - y) * 1.3) / (2.2 * q) + 0.5, 0, 1)        # lit upper left
        p.put(m, lerp(light, dark, t) * (0.85 + 0.3 * p.fbm(1.5)[..., None]))


def scene_a(seed=3):
    """Look A: the temple as a miniature on a dark disc with a grass mound."""
    p = Paint(107, 128, seed); X, Y = p.X, p.Y
    cx, cy, R = 51.5, 54.5, 42.0
    r = np.hypot(X - cx, Y - cy)
    p.put(np.ones_like(X), lerp((30, 29, 27), (12, 12, 11), np.clip(r / R, 0, 1) ** 2))   # the dark disc, vignetted
    ink = (26, 20, 15)
    # grass mound: a low dome with a ragged top edge, fading into the dark
    top = 78.5 + 8.5 * ((X - cx) / 40) ** 2 + 1.6 * (p.fbm(3, 2) - 0.5)
    mound = ((Y > top) & (np.abs(X - cx) < 41)).astype(np.float32)
    g = lerp((44, 62, 26), (104, 128, 52), np.clip(p.fbm(2.2) * 1.5 - 0.3, 0, 1))
    g = g * (1.1 - 0.85 * np.clip((Y - top) / 13, 0, 1))[..., None] * (1 - 0.6 * np.clip((np.abs(X - cx) - 22) / 20, 0, 1))[..., None]
    p.put(mound, g)
    pal = dict(light=(238, 224, 198), mid=(194, 172, 140), shade=(104, 84, 64), stain=(150, 116, 82), ink=ink, cella=(50, 38, 30), door=(20, 15, 12))
    temple(p, cx, 81, 51, pal)
    bush(p, 17.5, 77, 6.8, (40, 62, 24), (124, 150, 64), ink, n=9); bush(p, 85.5, 77.5, 6.8, (40, 62, 24), (116, 142, 60), ink, n=9)
    # loose stones and grass tufts over the foot of the steps
    for x, y, q in ((25, 82.5, 2.6), (79, 83, 2.9), (31, 85, 1.8), (72, 85.5, 1.6)):
        m = p.ell(x, y, q, q * 0.7); p.line(m, ink, 0.4)
        p.put(m, lerp((214, 200, 176), (120, 104, 86), np.clip(((X - x) + (Y - y)) / (2 * q) + 0.5, 0, 1)))
    for _ in range(26):
        x = cx + p.rng.uniform(-34, 34); y = 81.5 + 7 * ((x - cx) / 40) ** 2 + p.rng.uniform(0.5, 5)
        m = p.poly([(x - 0.8, y), (x + p.rng.uniform(-1.2, 1.2), y - p.rng.uniform(2, 3.6)), (x + 0.8, y)])
        p.put(m, (126, 150, 64) if p.rng.random() < 0.5 else (70, 96, 38))
    return p.finish()


def scene_b(seed=11):
    """Look B: the temple in a landscape: sky, clouds, far mountains, trees,
    a flagged path running down to the point of the opening."""
    p = Paint(96, 128, seed); X, Y = p.X, p.Y
    cx = 48.0; ink = (38, 34, 36)
    # sky and clouds
    p.put(np.ones_like(X), lerp((40, 92, 168), (150, 192, 224), np.clip((Y - 8) / 40, 0, 1)))
    n = p.fbm(12, 4)
    cl = np.clip((n - 0.50) * 9, 0, 1) * np.clip(1 - np.abs(Y - 25) / 14, 0, 1)
    p.put(cl, lerp((252, 252, 250), (176, 194, 220), np.clip((0.62 - n) * 6 + (Y - 24) / 14, 0, 1)))   # lit tops, grey-blue bases
    # far mountains (blue), nearer hills (green), each with a lit left flank
    def ridge(y0, amp, cell, c_light, c_dark, ph):
        f1 = ((X + ph) / cell) % 1.0; f2 = ((X + ph * 2.3) / (cell * 0.47)) % 1.0
        prof = y0 - amp * (1 - np.abs(2 * f1 - 1)) - amp * 0.4 * (1 - np.abs(2 * f2 - 1)) + 1.6 * (p.fbm(2.5, 2) - 0.5)   # sharp peaks
        m = (Y > prof).astype(np.float32)
        lit = np.clip((0.5 - f1) * 6 + 0.5 + (p.fbm(3) - 0.5) * 1.2, 0, 1)                    # the left flank of each peak is lit
        p.put(m, lerp(c_dark, c_light, lit) * (1 - 0.25 * np.clip((Y - prof) / 20, 0, 1))[..., None])
    ridge(49, 19, 34, (176, 190, 218), (74, 94, 142), 30)
    ridge(55, 8, 23, (108, 144, 104), (50, 84, 70), 3)
    # ground: grass with light and dark patches
    gm = (Y > 57 + 1.2 * (p.fbm(4, 2) - 0.5)).astype(np.float32)
    grass = lerp((58, 92, 40), (132, 164, 72), np.clip(p.fbm(3.2) * 1.6 - 0.35, 0, 1))
    p.put(gm, grass * (1.05 - 0.3 * np.clip((Y - 57) / 40, 0, 1))[..., None])
    # trees either side, behind the temple
    for x, y, r in ((15, 53, 9), (24, 58, 6.5), (81, 53, 9), (72, 58, 6.5), (9, 62, 6), (88, 62, 6)):
        bush(p, x, y, r, (24, 52, 32), (104, 142, 62), ink, n=16)
    # the path: flagstones widening toward the viewer, down to the point
    hwp = lambda y: 5.5 + (y - 76) * 0.42
    path = ((np.abs(X - cx) < hwp(Y) + 0.8 * (p.fbm(2, 2) - 0.5)) & (Y > 76)).astype(np.float32)
    p.put(path, lerp((206, 192, 160), (142, 128, 104), p.fbm(2.5)) * (1.05 - 0.25 * np.clip((Y - 76) / 20, 0, 1))[..., None])
    for y in (80.5, 85.5, 91):
        p.put(path * p.rect(0, y - 0.3, 96, y + 0.3), ink, 0.55)
    for y0, y1, fs in ((76, 80.5, (0.1,)), (80.5, 85.5, (-0.4, 0.45)), (85.5, 91, (0.0,)), (91, 97, (-0.35, 0.4))):
        for f in fs: p.put(path * p.poly([(cx + f * hwp(y0) - 0.3, y0), (cx + f * hwp(y0) + 0.3, y0), (cx + f * hwp(y1) + 0.3, y1), (cx + f * hwp(y1) - 0.3, y1)]), ink, 0.5)
    # cast shadow of the temple on the grass, to the right
    p.put(p.poly([(26, 77.5), (73, 77.5), (81, 81.5), (32, 81.5)]), (24, 44, 26), 0.45)
    # warm light, cool shade: marble in sun
    pal = dict(light=(240, 230, 206), mid=(186, 176, 160), shade=(98, 98, 116), stain=(146, 122, 96), ink=ink, cella=(50, 46, 56), door=(22, 20, 26))
    temple(p, cx, 78, 46, pal)
    bush(p, 20, 77, 4.8, (32, 64, 32), (124, 160, 70), ink, n=8); bush(p, 76.5, 77.5, 4.8, (32, 64, 32), (118, 154, 68), ink, n=8)
    for _ in range(22):
        x = p.rng.uniform(14, 82); y = p.rng.uniform(80, 92)
        if abs(x - cx) < hwp(y) + 1.5: continue
        m = p.poly([(x - 0.8, y), (x + p.rng.uniform(-1.2, 1.2), y - p.rng.uniform(2, 3.4)), (x + 0.8, y)])
        p.put(m, (150, 178, 80) if p.rng.random() < 0.5 else (60, 96, 42))
    return p.finish()


def stack(names):
    return np.stack([np.asarray(Image.open(PINS + n + ".png").convert("RGBA")) for n in names]).astype(np.float32)

def opening_b(st):
    """Look B's picture opening: inside the pin, and neither gold frame nor
    dark outline in most of the pins; one connected region, holes filled."""
    R, G, B, A = [st[..., i] for i in range(4)]
    gold = ((R - B > 70) & (R > G) & (G > B)).mean(0) > 0.6
    dark = (st[..., :3].max(-1) < 45).mean(0) > 0.6
    m = Image.fromarray((((A.min(0) > 250) & ~gold & ~dark) * 255).astype(np.uint8)).filter(ImageFilter.MinFilter(3))
    ImageDraw.floodfill(m, (48, 50), 128)
    a = Image.fromarray(((np.asarray(m) == 128) * 255).astype(np.uint8)).filter(ImageFilter.MaxFilter(3))
    inv = Image.fromarray(((np.asarray(a) == 0) * 255).astype(np.uint8)).copy(); ImageDraw.floodfill(inv, (0, 0), 128)
    a = (np.asarray(inv) != 128).astype(np.uint8) * 255
    return np.asarray(Image.fromarray(a).filter(ImageFilter.GaussianBlur(0.6)), np.float32) / 255

def compose(body, pic, mask):
    out = body.copy()
    out[..., :3] = body[..., :3] * (1 - mask[..., None]) + pic * mask[..., None]
    return Image.fromarray(np.clip(out, 0, 255).astype(np.uint8), 'RGBA')

def font(sz):
    try: return ImageFont.load_default(size=sz)
    except TypeError: return ImageFont.load_default()

def main():
    sa = stack(LOOK_A); body_a = np.median(sa, 0)
    ys, xs = np.mgrid[0:128, 0:107]
    mask_a = np.clip(41.6 - np.hypot(xs - 51.5, ys - 54.5), 0, 1).astype(np.float32)   # measured disc, 1 px feather
    A = compose(body_a, scene_a(), mask_a); A.save(os.path.join(HERE, "temple_A.png"))
    sb = stack(LOOK_B); body_b = np.median(sb, 0)
    mb = opening_b(sb)
    # the set's scenes darken a little toward the frame: an inner shadow
    edge = np.asarray(Image.fromarray((mb * 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(3.5)), np.float32) / 255
    pic_b = scene_b() * (0.62 + 0.38 * np.clip((edge - 0.5) * 2.6, 0, 1))[..., None]
    B = compose(body_b, pic_b, mb); B.save(os.path.join(HERE, "temple_B.png"))
    # options sheet: each option beside three neighbours of its look
    rows = [(['monument', 'market', 'castle'], A, 'temple_A'), (['shrine', 'tomb', 'ceremonial_site'], B, 'temple_B')]
    cw = 150; W = cw * 4 + 16; RH = 262
    sh = Image.new("RGBA", (W, RH * 2), (52, 54, 60, 255)); d = ImageDraw.Draw(sh); f = font(11)
    for r, (nbs, im, nm) in enumerate(rows):
        y0 = r * RH
        d.rectangle([0, y0 + 166, W, y0 + RH], fill=(150, 158, 122, 255))
        cells = [(n, Image.open(PINS + n + ".png").convert("RGBA")) for n in nbs]; cells.insert(2, (nm + " (coded)", im))
        for i, (label, pin) in enumerate(cells):
            x = 8 + i * cw
            sh.alpha_composite(pin, (x + (cw - pin.width) // 2, y0 + 8)); d.text((x + cw / 2, y0 + 142), label, fill=(235, 235, 235), font=f, anchor="ma")
            for h, xo in ((48, 30), (32, 96)):
                t = pin.resize((round(pin.width * h / 128), h), Image.LANCZOS)
                sh.alpha_composite(t, (x + xo, y0 + 226 - h)); d.text((x + xo + t.width / 2, y0 + 232), str(h), fill=(20, 20, 20), font=f, anchor="ma")
    sh.save(os.path.join(HERE, "temple_options.png"))

if __name__ == "__main__":
    main()

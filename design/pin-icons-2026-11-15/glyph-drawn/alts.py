"""Alternative pictograms for the three icons the owner asked to be rethought
(2026-11-15): `ancient_road`, `portage`, `historic_crossing`, and two options
for `caravan_station` that show a stop rather than only a wagon.
Adopted since: ancient_road B and portage C (now in fable_glyphs.py);
historic_crossing keeps the current drawing.

Uses fable_glyphs.py's canvas, pin body and compositor unchanged, so every
alternative obeys the same rules (the owner's rim and outline, pictogram
clipped 5 px inside the rim, 110 x 128, tip at the bottom centre).
Writes out_alt/<name>_{A,B,C}.png and <name>_options.png. It must never write
into out/: the owner chooses first.
"""
import math, os
import numpy as np
from PIL import Image, ImageDraw
import fable_glyphs as F
from fable_glyphs import icon, wheel, wagon, TAN, WATER, STONE, FLAME

# ---- ancient_road ------------------------------------------------------------
@icon('ancient_road_A', 'blue')
def _(g):
    # A: a straight paved road running to the horizon between kerb stones, with
    # a Roman milestone standing in front of it.
    vx, vy = -12, -20                                         # vanishing point
    L = lambda y: vx - 1.5 + (y - vy) * (-19.0 / 48)          # road's left / right edge at height y
    R = lambda y: vx + 1.5 + (y - vy) * (15.0 / 48)
    g.line([(-34, vy), (10, vy)], 2.2)                        # the horizon
    g.poly([(L(28), 28), (L(vy), vy), (R(vy), vy), (R(28), 28)])
    rows = [-20, -13, -5, 5, 17, 30]
    for i, y in enumerate(rows[1:-1]): g.line([(L(y), y), (R(y), y)], 1.3 + 0.3 * i, g.k)
    for i in range(1, len(rows) - 1):                         # staggered joints
        y0, y1 = rows[i], rows[i + 1]
        for f in ((0.33, 0.67) if i % 2 else (0.5,)):
            g.line([(L(y0) + f * (R(y0) - L(y0)), y0), (L(y1) + f * (R(y1) - L(y1)), y1)], 1.2 + 0.3 * i, g.k)
    for i in range(len(rows) - 1):                            # kerb stones, shrinking with distance
        y0, y1 = rows[i] + 0.8, rows[i + 1] - 0.8
        w0, w1 = 1.0 + (y0 - vy) * 0.085, 1.0 + (y1 - vy) * 0.085; gp = 1.5
        g.poly([(L(y0) - gp - w0, y0), (L(y0) - gp, y0), (L(y1) - gp, y1), (L(y1) - gp - w1, y1)])
        g.poly([(R(y0) + gp + w0, y0), (R(y0) + gp, y0), (R(y1) + gp, y1), (R(y1) + gp + w1, y1)])
    g.circ(22.5, -10, 11, g.k); g.rect(11.5, -10, 33.5, 30, g.k)   # a clear gap around the milestone
    g.circ(22.5, -10, 8.5); g.rect(14, -10, 31, 21); g.rect(12, 21, 33, 27)
    for y, hw in ((-9, 4.5), (-2, 5), (5, 3.5)): g.line([(22.5 - hw, y), (22.5 + hw, y)], 2.4, g.k)
    g.rect(-40, 27, 40, 40, g.k)

@icon('ancient_road_B', 'blue')
def _(g):
    # B: a road passing under a ruined archway (the ring is broken on the
    # right, a fallen block lies beside it); the road is tan so it reads as a way.
    ro, ri, cy = 30, 19, -4
    ring = [(ro * math.cos(math.radians(a)), cy + ro * math.sin(math.radians(a))) for a in range(-180, -49, 5)]
    inner = [(ri * math.cos(math.radians(a)), cy + ri * math.sin(math.radians(a))) for a in range(-56, -181, -5)]
    g.poly(ring + [(17, -22.5), (13.5, -21)] + inner)         # arch ring, ending in a ragged break
    g.rect(-30, cy, -19, 25); g.rect(-33, cy - 2, -17, cy + 2.5); g.rect(-33, 21, -16.5, 25)
    g.poly([(19, 25), (19, 0), (22.5, -5), (25.5, 1), (30, -3), (30, 25)]); g.rect(16.5, 21, 33, 25)
    for a in (-152, -124, -96, -70):                          # voussoir joints
        ca, sa = math.cos(math.radians(a)), math.sin(math.radians(a))
        g.line([(ca * (ri + 2), cy + sa * (ri + 2)), (ca * (ro - 2), cy + sa * (ro - 2))], 1.8, g.k)
    hw = lambda y: 3.5 + (y - 2) * 0.5                        # the road, widening toward the viewer
    g.poly([(-hw(2), 2), (hw(2), 2), (hw(30), 30), (-hw(30), 30)], TAN)
    rows = [2, 8, 16, 30]
    for i, y in enumerate(rows[1:-1]): g.line([(-hw(y), y), (hw(y), y)], 1.4 + 0.3 * i, g.k)
    for i in range(len(rows) - 1):
        for f in ((-0.3, 0.3) if i % 2 else (0.0,)):
            g.line([(f * hw(rows[i]), rows[i]), (f * hw(rows[i + 1]), rows[i + 1])], 1.3 + 0.3 * i, g.k)
    g.rect(-40, 27, 40, 40, g.k)

@icon('ancient_road_C', 'blue')
def _(g):
    # C: worn cobbles with two wheel ruts, winding away, grass at the verges.
    P0, P1, P2 = (2, 32), (-16, 6), (9, -27)
    def at(t):
        x = (1 - t) ** 2 * P0[0] + 2 * (1 - t) * t * P1[0] + t * t * P2[0]; y = (1 - t) ** 2 * P0[1] + 2 * (1 - t) * t * P1[1] + t * t * P2[1]
        dx = 2 * (1 - t) * (P1[0] - P0[0]) + 2 * t * (P2[0] - P1[0]); dy = 2 * (1 - t) * (P1[1] - P0[1]) + 2 * t * (P2[1] - P1[1])
        n = math.hypot(dx, dy); return x, y, -dy / n, dx / n, 17 - 12 * t
    pt = lambda t, f: (at(t)[0] + at(t)[2] * at(t)[4] * f, at(t)[1] + at(t)[3] * at(t)[4] * f)
    ts = np.linspace(0, 1, 30)
    g.poly([pt(t, 1) for t in ts] + [pt(t, -1) for t in ts[::-1]])
    cuts = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]
    for i, t in enumerate(cuts): g.line([pt(t, -1), pt(t, 1)], 1.5 - 0.08 * i, g.k)
    for i, (t0, t1) in enumerate(zip([0.0] + cuts, cuts + [1.0])):
        for f in ((-0.72, 0.0, 0.72) if i % 2 else (-0.36, 0.36)):
            g.line([pt(t0, f), pt(t1, f)], 1.4 - 0.08 * i, g.k)
    for f in (-0.5, 0.5):                                     # the ruts: wide near, thin far
        for t0, t1, w in ((0.0, 0.35, 4.2), (0.35, 0.7, 3.0), (0.7, 1.0, 1.8)):
            g.line([pt(t, f) for t in np.linspace(t0, t1, 10)], w, g.k)
    for x, y, s in ((-25, 22, 1.3), (25, 22, 1.3), (25, -6, 1.0)):   # grass tufts at the verge
        for dx in (-2.2, 0, 2.2): g.line([(x + dx * 0.5 * s, y), (x + dx * s, y - (8 - abs(dx) * 1.2) * s)], 2.0 * s)
    g.rect(-40, 27.5, 40, 40, g.k)

# ---- portage -------------------------------------------------------------------
@icon('portage_A', 'blue')
def _(g):
    # A: a boat on log rollers, hauled overland by a rope.
    g.poly([(-35, -16), (-27, -9), (9, -9), (16, -13), (21, -22), (18, -5), (11, 6), (-22, 6), (-31, -4)])
    g.line([(-25, -1.8), (14, -1.8)], 1.9, g.k)
    for x in (-25, -12.5, 0, 12.5): g.circ(x, 12.6, 5.8); g.circ(x, 12.6, 2.1, g.k)
    g.rect(-34, 18.6, 34, 22)
    g.line([(20.5, -20), (25.5, -6)], 1.8)                    # the rope
    g.circ(30.5, -11, 4.0); g.line([(29.5, -4.5), (26, 7)], 5.6)
    g.line([(29, -4), (25, -6)], 3.0)
    g.line([(26, 7), (22.5, 17.5)], 4.2); g.line([(26, 7), (31, 17.5)], 4.2)

@icon('portage_B', 'blue')
def _(g):
    # B: a plan view: two waters, a dotted carrying track between them, and a
    # canoe lying on the track.
    g.ell(-52, -58, -3, -14, WATER); g.ell(3, 14, 52, 58, WATER)
    g.wave(-25, -27, -11, 2.4, 1.3, 9, g.k); g.wave(25, 11, 27, 2.4, 1.3, 9, g.k)
    a = math.atan2(1, 0.78); ca, sa = math.cos(a), math.sin(a)
    for u in (-24.5, -19.5, 19.5, 24.5): g.circ(u * ca, u * sa, 2.2, TAN)
    hl, hw = 16.5, 7.2
    up = [(u, hw * (1 - abs(u / hl) ** 2.0)) for u in np.linspace(-hl, hl, 31)]
    g.poly([(u * ca - v * sa, u * sa + v * ca) for u, v in up + [(u, -v) for u, v in up[::-1]]])
    for u in (-5, 5):                                         # thwarts
        g.line([(u * ca + 4.2 * sa, u * sa - 4.2 * ca), (u * ca - 4.2 * sa, u * sa + 4.2 * ca)], 2.0, g.k)

@icon('portage_C', 'blue')
def _(g):
    # C: a boat riding on a two-wheeled cart.
    g.poly([(-33, -17), (-24, -9), (16, -9), (24, -13), (30, -21), (25, -4), (17, 3), (-20, 3), (-29, -6)])
    g.line([(-23, -3.4), (20, -3.4)], 1.7, g.k)
    g.rect(-23, 5, 21, 9); g.line([(21, 7), (34, 2.5)], 3.2)
    for x in (-21, 19): g.rect(x - 1.4, 2, x + 1.4, 6)          # chocks holding the hull
    wheel(g, -12, 15.5, 9.5); wheel(g, 11, 15.5, 9.5)
    g.rect(-30, 25.6, 30, 28)

# ---- historic_crossing ---------------------------------------------------------
@icon('historic_crossing_A', 'blue')
def _(g):
    # A: a steep single-arch packhorse bridge with a memorial stone at its
    # foot. One high arch and a pillar: not the owner's low three-arch bridge.
    top = [(x, 4 - 13 * max(0.0, 1 - (abs(x) / 27.0) ** 1.6)) for x in np.arange(-40, 40.01, 1)]
    g.poly(top + [(40, 19), (-40, 19)])
    g.arch(0, 17.5, -1.5, 20)
    g.rect(-33, -17, -25, 5); g.poly([(-33, -16.5), (-29, -25), (-25, -16.5)])
    g.line([(-31, -11), (-27, -11)], 1.9, g.k); g.line([(-31, -6), (-27, -6)], 1.9, g.k)
    g.rect(-40, 18, 40, 40, g.k)
    g.wave(23, -29, 29, 4.6, 1.8, 14.5); g.wave(31, -18, 18, 4.2, 1.6, 14.5, ph=1.6)

@icon('historic_crossing_B', 'blue')
def _(g):
    # B: seen from the bank: a carved waymarker stone, and old stepping stones
    # leading across the water to the far side.
    g.poly([(-32, 9), (-31, -20), (-26, -29), (-18.5, -26), (-16, 9)])
    g.line([(-25, -19), (-25, -1)], 2.5, g.k); g.line([(-25, -18), (-20.5, -13), (-25, -8.5)], 2.3, g.k)
    g.poly([(-40, 8), (-12, 8), (-7, 13), (-7, 30), (-40, 30)])
    for x in (1.5, 13.5, 25.5, 37.5): g.ell(x - 5.2, 4.6, x + 5.2, 12.8, STONE)
    g.wave(17.5, -3, 34, 4.6, 1.7, 13); g.wave(26, -3, 26, 4.2, 1.5, 13, ph=1.6)

@icon('historic_crossing_C', 'blue')
def _(g):
    # C: a map view: an inscribed stone at the head of a dashed track that
    # crosses the river.
    g.wave(4, -40, 40, 6.0, 2.2, 17); g.wave(15, -40, 40, 6.0, 2.2, 17, ph=1.9)
    g.rect(-5.5, -8, 5.5, 34, g.k)
    for y0 in (-5, 4.5, 14, 23.5): g.rect(-2.6, y0, 2.6, y0 + 6, TAN)
    g.circ(0, -23, 11.5, g.k); g.rect(-11.5, -23, 11.5, -5, g.k)
    g.circ(0, -23, 8.6); g.rect(-8.6, -23, 8.6, -11); g.rect(-11.5, -11.5, 11.5, -7.5)
    for y, hw in ((-24, 4.2), (-18.5, 4.8)): g.line([(-hw, y), (hw, y)], 2.3, g.k)

# ---- caravan_station ---------------------------------------------------------------
@icon('caravan_station_A', 'blue')
def _(g):
    # A: the wagon drawn up beside a small open draw-well (a windlass frame, no
    # roof, so it is not the `well` pin).
    wagon(g, -12, 5, 0.72, tongue=False)
    g.rect(13, 12, 30, 25); g.rect(11.5, 9.5, 31.5, 13)
    g.rect(13, 18.2, 30, 19.6, g.k)
    for x in (21.5,): g.rect(x - 0.7, 13, x + 0.7, 18.5, g.k)
    for x in (17, 26): g.rect(x - 0.7, 19.3, x + 0.7, 25, g.k)
    g.rect(13.5, -9, 16.3, 10); g.rect(26.7, -9, 29.5, 10); g.rect(11.5, -12, 31.5, -8)
    g.rect(20.8, -8, 22.2, -1); g.poly([(18, -2), (25, -2), (24, 5), (19, 5)])
    g.rect(-31, 25, 31, 27.5)

@icon('caravan_station_B', 'blue')
def _(g):
    # B: the wagon standing under a waystation roof, a lit lantern hung from
    # the eave: shelter and a light, so a stop for the night.
    g.poly([(-35, -9), (0, -30), (35, -9)]); g.rect(-33, -10, 33, -6); g.circ(0, -17.5, 3.8, g.k)
    g.rect(-31, -6, -27, 25); g.rect(27, -6, 31, 25)
    wagon(g, 1.5, 9.5, 0.6, tongue=False)
    g.rect(20.4, -6, 21.6, -1.5); g.circ(21, 2, 3.6, FLAME); g.rect(18.4, -2.2, 23.6, -0.8)
    g.rect(-31, 25, 31, 27.5)

# ---- sheets ----------------------------------------------------------------------
def sheet(name, cat, left, right, letters="ABC"):
    """Options sheet: neighbour, the current icon, A B C, neighbour; each at
    128 px on the dark ground and at 48 and 32 px on the map-like ground."""
    body = F.bodies()[cat]
    alts = [(n, F.render(f'{name}_{n}', body)) for n in letters]
    for n, im in alts: im.save(os.path.join(F.HERE, "out_alt", f"{name}_{n}.png"))
    op = lambda p: Image.open(p).convert("RGBA")
    mine = lambda n: (n, op(os.path.join(F.OUT, n + ".png")))
    nb = lambda n: ("owner: " + n[6:], op(F.OWNER + n[6:] + ".png")) if n.startswith("owner:") else mine(n)
    cells = [nb(left), ("current", mine(name)[1])] + [("option " + n, im) for n, im in alts] + [nb(right)]
    cw = 132; W = cw * len(cells) + 16
    sh = Image.new("RGBA", (W, 262), (52, 54, 60, 255)); d = ImageDraw.Draw(sh); f = F.font(11)
    d.rectangle([0, 166, W, 262], fill=(150, 158, 122, 255))
    for i, (nm, im) in enumerate(cells):
        x = 8 + i * cw
        sh.alpha_composite(im, (x + 11, 8)); d.text((x + 66, 142), nm, fill=(235, 235, 235), font=f, anchor="ma")
        sh.alpha_composite(im.resize((41, 48), Image.LANCZOS), (x + 16, 178))
        sh.alpha_composite(im.resize((28, 32), Image.LANCZOS), (x + 76, 194))
        d.text((x + 36, 232), "48", fill=(20, 20, 20), font=f, anchor="ma"); d.text((x + 90, 232), "32", fill=(20, 20, 20), font=f, anchor="ma")
    sh.save(os.path.join(F.HERE, f"{name}_options.png"))

if __name__ == "__main__":
    os.makedirs(os.path.join(F.HERE, "out_alt"), exist_ok=True)
    sheet('ancient_road', 'blue', 'road_junction', 'caravan_station')
    sheet('portage', 'blue', 'ford', 'caravan_station')
    sheet('historic_crossing', 'blue', 'owner:bridge', 'ford')
    sheet('caravan_station', 'blue', 'well', 'trade_depot', letters="AB")

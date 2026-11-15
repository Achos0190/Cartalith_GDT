"""Glyph-set teardrop pins for the 39 kinds the owner's sheet does not cover.

What it does
    Writes one 110 x 128 RGBA PNG per kind into ./out/, plus two contact sheets
    (contact_128.png, contact_32.png) that put the owner's 20 pins on top for
    comparison.

How the pin body is made (and why)
    The owner's 20 pins (icons/pins/glyph/*.png) are pixel-aligned: every one
    has the same alpha mask, the same rim and the same disc. So the body is not
    redrawn here: it is the per-pixel MEDIAN of the five owner pins of the same
    category colour. That gives the owner's exact outline, rim shading, inner
    dark ring, alpha edge and tip pixel, so hit-testing and tilting are
    unchanged. Only the inside of the disc (radius DISC_R about the measured
    centre) is replaced, by the pictogram drawn below.

How a pictogram is drawn
    On a square canvas SS times oversampled, in final-pixel units with the
    origin at the disc centre (x right, y down; the disc edge is at r = 45, the
    owner's pictograms fill roughly +-32). A median filter rounds the corners
    slightly, as the owner's art has, then a faint top-light gradient is
    applied and the canvas is box-downsampled.

What it must never do
    Touch a pixel outside the disc, change the canvas size, or write anywhere
    but this folder. It reads the repository's pins and writes nothing there.
"""
import math, os, sys
import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
OWNER = "C:/Users/Vincent/Cartalith_GDT/cartalith-native/godot-project/icons/pins/glyph/"
OUT = os.path.join(HERE, "out")

# The owner's pins by rim colour; the median of each group is that colour's body.
OWNER_CATS = {
    'gold':   ['capital', 'castle', 'ruin', 'town', 'village'],
    'blue':   ['bridge', 'harbour', 'market', 'mine', 'temple'],
    'green':  ['cave', 'forest', 'lake', 'mountain', 'waterfall'],
    'orange': ['desert', 'farmland', 'rockfall', 'volcano', 'wildlife'],
}
# Measured on the owner's pins: the dark disc spans x,y = 10..99, so its centre
# is (54.5, 54.5) and its edge r = 45. The pictogram is pasted inside DISC_R,
# leaving the owner's own 1 px darker inner ring untouched.
CX = CY = 54.5
DISC_R = 43.6
# No pictogram pixel may touch the rim: everything is clipped to this radius,
# which leaves an even dark margin (owner, 2026-11-15: touching the disc edge
# is an artifact).
CLIP_R = 40.0
# Disc and pictogram colours, sampled from the owner's pins (disc: the flat
# interior; pictogram: the mid-height fill of castle, temple, forest, wildlife).
DISC = {'gold': (18, 18, 18), 'blue': (14, 20, 26), 'green': (17, 22, 21), 'orange': (19, 19, 19)}
PIC = {'gold': (250, 197, 84), 'blue': (101, 179, 247), 'green': (158, 214, 102), 'orange': (245, 130, 68)}
# Secondary colours, also from the owner's sheet: water, snow, fire, bare stone
# (rockfall), and the tan of the farmland furrows / desert track.
WATER = (96, 176, 244); SNOW = (240, 244, 240); FIRE = (250, 140, 50); FLAME = (255, 214, 96)
STONE = (190, 186, 178); TAN = (214, 180, 104)

SS = 8            # oversampling factor
HALF = 48         # canvas half-size in final pixels (canvas = 96 x 96)
N = 2 * HALF * SS


class Cv:
    """One pictogram canvas. All coordinates are final pixels from the disc centre.

    `c` is the category colour, `k` the disc colour (used for cut-outs: windows,
    doors, joints, and the dark 'halo' that separates overlapping shapes)."""

    def __init__(s, cat, sc=1.0):
        s.c, s.k, s.sc = PIC[cat], DISC[cat], sc
        s.im = Image.new('RGB', (N, N), s.k)
        s.d = ImageDraw.Draw(s.im)

    def X(s, p):
        return ((HALF + p[0] * s.sc) * SS, (HALF + p[1] * s.sc) * SS)

    def poly(s, pts, col=None):
        s.d.polygon([s.X(p) for p in pts], fill=col or s.c)

    def rect(s, x0, y0, x1, y1, col=None):
        x0, x1 = min(x0, x1), max(x0, x1); y0, y1 = min(y0, y1), max(y0, y1)
        s.d.rectangle([s.X((x0, y0)), s.X((x1, y1))], fill=col or s.c)

    def ell(s, x0, y0, x1, y1, col=None):
        s.d.ellipse([s.X((x0, y0)), s.X((x1, y1))], fill=col or s.c)

    def circ(s, x, y, r, col=None):
        s.ell(x - r, y - r, x + r, y + r, col)

    def chord(s, x0, y0, x1, y1, a0, a1, col=None):
        s.d.chord([s.X((x0, y0)), s.X((x1, y1))], a0, a1, fill=col or s.c)

    def line(s, pts, w, col=None):
        """A round-capped, round-jointed stroke of width w."""
        col = col or s.c
        s.d.line([s.X(p) for p in pts], fill=col, width=max(1, int(round(w * s.sc * SS))), joint='curve')
        for p in (pts[0], pts[-1]):
            s.circ(p[0], p[1], w / 2, col)

    def hline(s, pts, w, col=None, halo=2.0):
        """A stroke with a disc-coloured halo, so it reads over another shape."""
        s.line(pts, w + 2 * halo, s.k); s.line(pts, w, col)

    def arch(s, x, w, top, bottom, col=None):
        """A round-headed opening centred on x: half-width w, crown at `top`."""
        col = col or s.k
        s.circ(x, top + w, w, col); s.rect(x - w, top + w, x + w, bottom, col)

    def wave(s, y, x0, x1, w=4.0, a=2.0, lam=12.0, col=WATER, ph=0.0):
        s.line([(x, y + a * math.sin(2 * math.pi * (x / lam) + ph)) for x in np.arange(x0, x1 + 0.01, 0.5)], w, col)

    def crenel(s, x0, x1, y, n, h=4, col=None):
        """n merlons standing on y between x0 and x1 (merlon, gap, merlon ...)."""
        step = (x1 - x0) / (2 * n - 1)
        for i in range(n):
            s.rect(x0 + 2 * i * step, y - h, x0 + (2 * i + 1) * step, y, col)

    def house(s, x0, x1, eave, base, rise, over=3):
        """A gabled house front: walls x0..x1 from eave to base, roof `rise` high."""
        s.rect(x0, eave, x1, base)
        s.poly([(x0 - over, eave + 1), ((x0 + x1) / 2, eave - rise), (x1 + over, eave + 1)])

    def pine(s, x, base, h, col=None):
        """A three-tier conifer like the owner's forest: trunk foot at `base`."""
        w = h * 0.36; t = h * 0.14
        for i, (f0, f1, ww) in enumerate(((0.0, 0.42, 0.55), (0.25, 0.68, 0.8), (0.52, 1.0, 1.0))):
            s.poly([(x, base - h + f0 * (h - t)), (x - w * ww, base - h + f1 * (h - t)), (x + w * ww, base - h + f1 * (h - t))], col)
        s.rect(x - h * 0.06, base - t - 0.5, x + h * 0.06, base, col)

    def peak(s, pts, cap=0.36, col=None):
        """A mountain (left foot, summit, right foot) with a jagged snow cap."""
        (lx, ly), (ax, ay), (rx, ry) = pts
        s.poly(pts, col)
        L = lambda t: (ax + (lx - ax) * t, ay + (ly - ay) * t)
        R = lambda t: (ax + (rx - ax) * t, ay + (ry - ay) * t)
        hy = ay + (max(ly, ry) - ay) * cap
        s.poly([(ax, ay), L(cap), (ax + (L(cap)[0] - ax) * 0.45, hy - (hy - ay) * 0.28), (ax, hy),
                (ax + (R(cap)[0] - ax) * 0.45, hy - (hy - ay) * 0.28), R(cap)], SNOW)

    def flame(s, x, base, h, w):
        """A two-tone fire: orange outer, yellow core."""
        s.poly([(x - w, base), (x - w * 0.9, base - h * 0.35), (x - w * 0.45, base - h * 0.62), (x - w * 0.5, base - h * 0.3),
                (x, base - h), (x + w * 0.45, base - h * 0.45), (x + w * 0.6, base - h * 0.7), (x + w, base - h * 0.3), (x + w * 0.9, base)], FIRE)
        s.poly([(x - w * 0.45, base), (x, base - h * 0.55), (x + w * 0.45, base)], FLAME)

    def star(s, x, y, r, col=SNOW):
        """A four-point sparkle."""
        q = r * 0.26
        s.poly([(x, y - r), (x + q, y - q), (x + r, y), (x + q, y + q), (x, y + r), (x - q, y + q), (x - r, y), (x - q, y - q)], col)

    def ground(s, hw=29, y=24, col=None):
        """The owner's lens-shaped ground line (village, ruin, forest)."""
        s.chord(-hw, y - 3.5, hw, y + 6.5, 180, 360, col); s.rect(-hw + 4, y + 1, hw - 4, y + 3, col)


def bez(p0, p1, p2, n=24):
    """Points on a quadratic Bezier, for curved strokes."""
    return [((1 - t) ** 2 * p0[0] + 2 * (1 - t) * t * p1[0] + t * t * p2[0],
             (1 - t) ** 2 * p0[1] + 2 * (1 - t) * t * p1[1] + t * t * p2[1]) for t in np.linspace(0, 1, n)]


def wheel(g, x, y, r=9.0):
    """A spoked cart wheel with a dark gap around it."""
    g.circ(x, y, r + 2, g.k); g.circ(x, y, r); g.circ(x, y, r - 3.6, g.k); g.circ(x, y, 2.3)
    for a in range(0, 180, 45):
        dx, dy = (r - 2) * math.cos(math.radians(a)), (r - 2) * math.sin(math.radians(a))
        g.line([(x - dx, y - dy), (x + dx, y + dy)], 1.5)

def wagon(g, ox=0.0, oy=0.0, k=1.0, tongue=True):
    """The covered wagon of `caravan_station`, drawn at scale k with its
    origin moved to (ox, oy), so a variant can park it beside something."""
    T = lambda x, y: (ox + x * k, oy + y * k)
    g.chord(*T(-28, -24), *T(20, 26), 180, 360); g.rect(*T(-28, 1), *T(20, 7))
    for x in (-13, 4): g.line([T(x, -23), T(x, 1)], 1.8 * k, g.k)
    g.rect(*T(-28, 4.6), *T(20, 6.2), g.k)
    g.rect(*T(-28, 6), *T(22, 13))
    if tongue: g.line([T(22, 10), T(33, 5)], 3.2 * k)
    for x in (-15, 9):
        cx, cy = T(x, 17)
        g.circ(cx, cy, 11.6 * k, g.k); g.circ(cx, cy, 9.6 * k); g.circ(cx, cy, 5.6 * k, g.k); g.circ(cx, cy, 2.4 * k)
        for a in range(0, 180, 45):
            dx, dy = 7 * k * math.cos(math.radians(a)), 7 * k * math.sin(math.radians(a))
            g.line([(cx - dx, cy - dy), (cx + dx, cy + dy)], max(1.2, 1.5 * k))

ICONS = {}
def icon(name, cat, sc=1.0):
    """Register a pictogram under its engine kind (= file name) and rim colour.

    `sc` enlarges a compact symbol about the disc centre so that every pin
    carries about the same weight of ink (the owner's fill about 70 %)."""
    def deco(f):
        ICONS[name] = (cat, f, sc); return f
    return deco


# ---- settlements and built things -----------------------------------------
@icon('city', 'gold')
def _(g):
    # A walled skyline: curtain wall with a gate, three different roofs behind
    # (gable, spire, dome). The long wall is what separates it from the owner's
    # capital (a castle) and town (a church among houses).
    g.house(-26, -13, -8, 6, 10, over=2); g.rect(-21.5, -3, -17.5, 3, g.k)
    g.rect(-7, -19, 7, 6); g.poly([(-9.5, -18), (0, -34), (9.5, -18)]); g.rect(-2.5, -13, 2.5, -5, g.k)
    g.rect(13, -5, 26, 6); g.chord(12, -16, 27, 6, 180, 360); g.rect(18.7, -21, 20.3, -15)
    g.rect(-36, 3.5, 36, 12, g.k)                           # a clear gap, so the merlons read
    g.rect(-29, 11, 29, 26); g.crenel(-29, 29, 11, 6, h=4.5)
    g.arch(0, 6.5, 14, 26)
    for x in (-19, 19): g.rect(x - 1.6, 16, x + 1.6, 22, g.k)

@icon('hamlet', 'gold')
def _(g):
    # Two cottages and one round tree: fewer and lower than the owner's village.
    g.rect(-10, -11, -5.5, 0)                                 # chimney
    g.house(-26, -4, 5, 24, 18, over=3.5); g.rect(-21, 12, -14.5, 24, g.k); g.rect(-11, 10, -6.5, 15.5, g.k)
    g.house(2, 15, 12, 24, 10, over=2.5); g.rect(6.5, 16.5, 10.5, 24, g.k)
    g.circ(24.5, 4, 7.5); g.rect(23.2, 10, 25.8, 24)
    g.ground(30, 23.5)

@icon('tavern', 'blue', 1.12)
def _(g):
    # A foaming tankard.
    g.ell(2, -3, 24, 20); g.ell(8, 3, 18, 14, g.k)
    g.rect(-20, -6, 8, 22); g.rect(-22, 22, 10, 26)
    for x in (-12, -3): g.rect(x - 1.6, 0, x + 1.6, 18, g.k)
    for x, y, r in ((-18, -9, 6.5), (-9, -13, 8.5), (2, -11, 7.5), (9, -8, 5)): g.circ(x, y, r)

@icon('smithy', 'blue', 1.08)
def _(g):
    # Anvil under a hammer, three sparks.
    g.poly([(-22, -4), (12, -4), (31, -2.5), (30.5, -0.5), (13, 6), (8, 6), (5, 12), (-11, 12), (-14, 6), (-22, 6)])
    g.rect(-9, 12, 3, 18); g.poly([(-19, 26), (-15, 18), (9, 18), (13, 26)])
    g.line([(15, -33), (-3, -17)], 4.5)                       # hammer: a square head across the handle
    g.poly([(-0.7, -7.7), (6, -13.6), (-5.3, -26.3), (-12, -20.4)])
    for x, y, r in ((-21, -11, 2.6), (-19, -29, 2.0), (-28, -20, 1.8)): g.star(x, y, r * 1.8, FLAME)

@icon('shrine', 'blue', 1.1)
def _(g):
    # A wayside shrine: roofed pillar, lit niche, stepped base.
    g.rect(-12, -9, 12, 18); g.poly([(-21, -8), (0, -27), (21, -8)]); g.circ(0, -28, 3.4)
    g.arch(0, 7, -4, 12); g.poly([(-3.4, 8), (0, -1), (3.4, 8)], FLAME); g.circ(0, 8, 3.4, FLAME)
    g.rect(-17, 18, 17, 22.2); g.rect(-23, 22, 23, 27)

@icon('guild', 'blue', 1.08)
def _(g):
    # A stepped-gable guildhall with a pennant: the stepped front is the
    # recognisable part, and no other pin in either set has it.
    g.rect(-23, -3, 23, 26); g.rect(-17, -10, 17, -3); g.rect(-11, -17, 11, -10); g.rect(-5, -24, 5, -17)
    g.rect(-1.2, -34, 1.2, -24); g.poly([(1, -34), (13, -30.5), (1, -27)])
    g.arch(0, 5.5, 10, 26); g.circ(0, -8, 4.2, g.k)
    for x in (-15, 15): g.rect(x - 3.2, 4, x + 3.2, 13, g.k)

@icon('gate', 'gold')
def _(g):
    # A gatehouse: two towers, a portcullis in the arch between them.
    for sx in (-1, 1):
        g.rect(sx * 29, -13, sx * 12, 26); g.rect(sx * 31, -18, sx * 10, -13)
        g.crenel(min(sx * 31, sx * 10), max(sx * 31, sx * 10), -18, 3, h=4.5)
        g.rect(sx * 20.5 - 1.6, -4, sx * 20.5 + 1.6, 6, g.k)
    g.rect(-12, -3, 12, 26); g.crenel(-10, 10, -3, 3, h=4)
    g.arch(0, 8.5, 5, 26)
    for x in (-4.2, 0, 4.2): g.rect(x - 1.1, 6, x + 1.1, 21); g.poly([(x - 1.6, 21), (x, 24), (x + 1.6, 21)])
    g.rect(-8, 13, 8, 15.2)

@icon('well', 'blue', 1.1)
def _(g):
    # A roofed draw-well with bucket.
    g.poly([(-25, -11), (0, -28), (25, -11)]); g.rect(-17, -11, -12.5, 10); g.rect(12.5, -11, 17, 10)
    g.rect(-1, -11, 1, -3); g.poly([(-5.5, -4), (5.5, -4), (4.2, 5), (-4.2, 5)])
    g.rect(-20, 9, 20, 26); g.ell(-20, 5, 20, 13); g.ell(-14.5, 7.2, 14.5, 11.2, g.k)
    g.rect(-20, 17.2, 20, 18.8, g.k)
    for x in (-9, 5): g.rect(x - 0.8, 12, x + 0.8, 17.5, g.k)
    for x in (-2, 12, -14): g.rect(x - 0.8, 18.5, x + 0.8, 26, g.k)

@icon('camp', 'gold')
def _(g):
    # A tent with its flap open and a campfire.
    g.line([(-13, -29), (-2, -14)], 2.4); g.line([(-1, -29), (-12, -14)], 2.4)
    g.poly([(-31, 24), (-7, -22), (14, 24)]); g.poly([(-13, 24), (-7, 0), (-1, 24)], g.k)
    g.flame(23, 23, 22, 7.5); g.line([(15, 24.5), (31, 21.5)], 2.6); g.line([(15, 21.5), (31, 24.5)], 2.6)
    g.rect(-32, 24, 12, 27)

@icon('other', 'blue', 1.12)
def _(g):
    # A question mark.
    pts = [(12 * math.cos(math.radians(a)), -12 + 12 * math.sin(math.radians(a))) for a in range(-195, 61, 5)]
    g.line(pts + bez(pts[-1], (0.5, 3), (0, 10), 8), 8.5)
    g.circ(0, 23, 5.4)


# ---- water ------------------------------------------------------------------
@icon('spring', 'green')
def _(g):
    # Water rising between two mossy rocks into a pool.
    g.poly([(0, -32), (-9, -16), (9, -16)], WATER); g.circ(0, -11, 10.4, WATER)
    g.ell(-5.5, -15, -1.5, -8, g.k)                           # highlight cut
    g.ell(-33, 2, -7, 30); g.ell(-21, 10, 1, 30); g.ell(8, 4, 33, 30); g.ell(0, 12, 18, 30)
    g.rect(-36, 19, 36, 34, g.k)
    g.wave(19, -27, 27, 4.4, 1.8, 13.5); g.wave(27.5, -18, 18, 4.0, 1.6, 13.5, ph=1.5)

@icon('river_confluence', 'green')
def _(g):
    # Two rivers joining into one, a conifer on each bank.
    g.pine(-22, 28, 27); g.pine(22, 28, 27)
    L = bez((-27, -30), (-10, -22), (0, 4)); R = bez((27, -30), (10, -22), (0, 4))
    for path in (L, R): g.line(path, 14, g.k)
    g.line([(0, 2), (0, 34)], 17, g.k)
    for path in (L, R): g.line(path, 9, WATER)
    g.line([(0, 2), (0, 34)], 12, WATER)

@icon('delta', 'green')
def _(g):
    # A river fanning out across low land into the sea.
    g.ell(-35, -27, 35, 57)
    chans = [bez((0, -10), (-7, 4), (-22, 18), 12), [(0, -10), (0, 18)], bez((0, -10), (7, 4), (22, 18), 12)]
    g.line([(0, -40), (0, -10)], 13, g.k)
    for ch in chans: g.line(ch, 10, g.k)
    g.line([(0, -40), (0, -10)], 8.5, WATER)
    for ch in chans: g.line(ch, 5.8, WATER)
    g.rect(-40, 14.5, 40, 40, g.k)
    g.wave(20, -29, 29, 4.6, 1.8, 14.5); g.wave(28.5, -21, 21, 4.2, 1.6, 14.5, ph=1.6)

@icon('ford', 'green')
def _(g):
    # A shallow river crossed by a track on stepping stones, bank to bank.
    g.ell(-40, -62, 40, -13); g.ell(-40, 15, 40, 64)
    g.wave(-5.5, -40, 40, 5.2, 1.7, 15); g.wave(6.5, -40, 40, 5.2, 1.7, 15, ph=2.0)
    g.line([(-13, -36), (-9, -19)], 5.5, TAN); g.line([(9, 21), (13, 38)], 5.5, TAN)
    for x, y in ((-7.5, -9.5), (0, 0.5), (7.5, 10.5)):
        g.ell(x - 7.2, y - 5.6, x + 7.2, y + 5.6, g.k); g.ell(x - 5.2, y - 3.7, x + 5.2, y + 3.7, STONE)

@icon('portage', 'blue')
def _(g):
    # A boat riding on a two-wheeled cart: carried overland. Owner's pick
    # (option C, 2026-11-15) over a walker carrying an upturned canoe.
    g.poly([(-33, -17), (-24, -9), (16, -9), (24, -13), (30, -21), (25, -4), (17, 3), (-20, 3), (-29, -6)])
    g.line([(-23, -3.4), (20, -3.4)], 1.7, g.k)
    g.rect(-23, 5, 21, 9); g.line([(21, 7), (34, 2.5)], 3.2)
    for x in (-21, 19): g.rect(x - 1.4, 2, x + 1.4, 6)          # chocks holding the hull
    wheel(g, -12, 15.5, 9.5); wheel(g, 11, 15.5, 9.5)
    g.rect(-30, 25.6, 30, 28)

@icon('historic_crossing', 'blue')
def _(g):
    # A three-arch bridge whose middle arch has fallen. Distinct from the
    # owner's (whole) bridge, and "old" without needing text.
    g.poly([(-40, -8), (-10, -12), (-6, -7), (-10, -3), (-7, 1), (-7, 18), (-40, 18)])
    g.poly([(40, -8), (12, -12), (10, -6), (13, -2), (9, 2), (9, 18), (40, 18)])
    g.arch(-27, 7.5, 3.5, 19); g.arch(27, 7.5, 3.5, 19); g.arch(1, 10.5, -2, 19)
    g.poly([(-3.5, 12.5), (2.5, 9.5), (4.5, 14.5), (-1.5, 17.5)])   # a fallen block
    g.rect(-40, 17, 40, 40, g.k)
    g.wave(21.5, -29, 29, 4.6, 1.8, 14.5); g.wave(29.5, -20, 20, 4.2, 1.6, 14.5, ph=1.6)


# ---- land -------------------------------------------------------------------
@icon('mountain_pass', 'green')
def _(g):
    # Two peaks and a track climbing to the saddle between them.
    g.peak([(-37, 26), (-16, -27), (8, 26)]); g.peak([(-4, 26), (17, -19), (37, 26)])
    g.poly([(-10, 26), (0.5, -1), (11, 26)])                 # fill the saddle below the notch
    path = bez((0, 29), (-9, 19), (0, 13), 10) + bez((0, 13), (6, 9), (0.5, 2), 8)
    g.line(path, 8.6, g.k); g.line(path, 4.6, (250, 232, 170))
    g.rect(-40, 26, 40, 40, g.k)

@icon('ridge', 'green')
def _(g):
    # One long serrated crest with snow along it, a lower ridge in front.
    crest = [(-40, 8), (-28, -9), (-20, -1), (-8, -22), (0, -9), (10, -27), (20, -9), (28, -15), (40, 4)]
    g.poly([(-40, 30)] + crest + [(40, 30)], SNOW)
    low = [(-40, 17)]
    for i, (x, y) in enumerate(crest[1:-1]):                  # zig-zag snow line under each summit
        low += [(x - 4.5, y + 15), (x, y + 9), (x + 4.5, y + 15)] if i % 2 == 0 else [(x, y + 13)]
    low.append((40, 13))
    g.poly([(-40, 30)] + low + [(40, 30)])
    front = [(-40, 20), (-26, 10), (-16, 17), (-3, 7), (8, 17), (19, 10), (29, 18), (40, 13)]
    g.poly([(-40, 40)] + [(x, y - 3.4) for x, y in front] + [(40, 40)], g.k)
    g.poly([(-40, 40)] + front + [(40, 40)])
    g.rect(-40, 26, 40, 40, g.k)

@icon('cliff', 'green')
def _(g):
    # A sheer face dropping to the sea, a lone tree on top.
    g.pine(-20, -16, 16)
    g.poly([(-40, 30), (-40, -17), (5, -17), (10, -12), (6, -3), (9, 8), (6, 30)])
    for a, b in (((-27, -5), (-9, -5)), ((-17, 7), (-1, 7)), ((-29, 17), (-15, 17))):
        g.line([a, b], 2.0, g.k)
    g.rect(-40, 26, 40, 40, g.k)
    g.wave(15, 15, 34, 4.4, 1.7, 12.5); g.wave(23.5, 13, 30, 4.2, 1.6, 12.5, ph=1.6)

@icon('gorge', 'green')
def _(g):
    # A plateau split by a narrow canyon, the river in the bottom of the slot.
    g.pine(-22, -15, 13); g.pine(23, -15, 13)
    g.poly([(-34, 26), (-34, -16), (34, -16), (34, 26)])
    g.poly([(-11, -17), (-6, -4), (-8, 6), (-5, 27), (5, 27), (7, 10), (4, -2), (10, -17)], g.k)
    g.poly([(-1.2, 1), (1.2, 1), (2.6, 14), (3.2, 27), (-3.2, 27), (-2.4, 14)], WATER)
    for a, b in (((-27, -5), (-14, -5)), ((-22, 8), (-12, 8)), ((14, -3), (26, -3)), ((11, 10), (21, 10))):
        g.line([a, b], 2.0, g.k)

@icon('rock_formation', 'orange')
def _(g):
    # A butte, a spire with a balanced cap, a stub: desert rock towers.
    g.poly([(-31, 24), (-28, 6), (-23, 3), (-24, -19), (-5, -19), (-6, 2), (-2, 6), (1, 24)])
    g.line([(-24, -8), (-6, -8)], 1.9, g.k); g.line([(-28, 10), (-1, 10)], 1.9, g.k)
    g.poly([(6, 24), (10, 4), (12, -8), (18, -8), (20, 4), (24, 24)]); g.ell(6, -21, 24, -9)
    g.poly([(24, 24), (28, 8), (31, 24)])
    g.rect(-33, 24, 33, 27)

@icon('glacier', 'green')
def _(g):
    # An ice tongue flowing down between two peaks, crevasses across it.
    g.peak([(-40, 30), (-19, -26), (4, 30)], cap=0.3); g.peak([(-2, 30), (19, -19), (40, 30)], cap=0.3)
    ice = [(-3, -6), (3, -6), (8, 6), (20, 30), (-22, 30), (-9, 6)]
    g.poly([(-7, -9), (7, -9), (12, 5), (25, 30), (-27, 30), (-13, 5)], g.k); g.poly(ice, SNOW)
    for y, hw in ((6, 4.5), (14, 8.5), (22, 13)):
        g.line([(-hw, y + 2), (0, y - 1.5), (hw, y + 2)], 2.4, WATER)
    g.rect(-40, 27, 40, 40, g.k)

@icon('quarry', 'blue')
def _(g):
    # A stepped working face cut into blocks, a pick above it.
    g.poly([(-31, 26), (-31, -22), (-16, -22), (-16, -10), (-2, -10), (-2, 2), (12, 2), (12, 14), (27, 14), (27, 26)])
    for a, b in (((-31, -10), (-16, -10)), ((-31, 2), (-2, 2)), ((-31, 14), (12, 14)), ((-23, -10), (-23, 2)), ((-9, 2), (-9, 14)),
                 ((-16, 14), (-16, 26)), ((0, 14), (0, 26)), ((14, 14), (14, 26)), ((-24, 2), (-24, 14))):
        g.line([a, b], 1.8, g.k)
    g.line([(7, -6), (24, -26)], 4.2)
    g.line(bez((9, -31), (25, -29), (31, -12), 14), 4.6)

@icon('salt_works', 'blue')
def _(g):
    # White salt heaps over a grid of evaporation pans.
    g.poly([(-31, 8), (-18, -12), (-5, 8)], SNOW); g.poly([(-8, 8), (11, -25), (30, 8)], g.k)
    g.poly([(-5, 8), (11, -21), (27, 8)], SNOW)
    for x0 in (-31, -10, 11):
        g.rect(x0, 11, x0 + 19, 17.5); g.rect(x0 + 2.5, 20.5, x0 + 16.5, 26.5)

@icon('road_junction', 'blue')
def _(g):
    # A fingerpost where the road forks.
    g.poly([(-30, 34), (-13, 17), (-4, 17), (-9, 34)], TAN); g.poly([(30, 34), (13, 17), (4, 17), (9, 34)], TAN)
    g.rect(-12, 16, 12, 22, TAN)
    g.rect(-3, -31, 3, 21)
    g.poly([(-1, -27), (17, -27), (26, -20), (17, -13), (-1, -13)])
    g.poly([(1, -9), (-17, -9), (-26, -2), (-17, 5), (1, 5)])
    g.rect(-40, 28, 40, 40, g.k)

@icon('caravan_station', 'blue')
def _(g):
    # A covered wagon.
    wagon(g)

@icon('trade_depot', 'blue')
def _(g):
    # A warehouse with a braced door, crates stacked outside.
    g.house(-29, 7, -5, 26, 18, over=3); g.circ(-11, -10, 3.6, g.k)
    g.rect(-22, 5, 0, 26, g.k); g.rect(-20, 7, -2, 26)
    g.line([(-20, 7), (-2, 26)], 1.8, g.k); g.line([(-2, 7), (-20, 26)], 1.8, g.k); g.rect(-11.8, 7, -10.2, 26, g.k)
    g.rect(12, 11, 31, 26); g.rect(15, -3, 28, 9)
    g.rect(12, 17.7, 31, 19.3, g.k); g.rect(20.7, 11, 22.3, 26, g.k); g.rect(15, 2.2, 28, 3.8, g.k)

@icon('ancient_road', 'blue')
def _(g):
    # A road passing under a ruined archway whose ring is broken on the right;
    # the road is tan so it reads as a way. Owner's pick (option B, 2026-11-15)
    # over a winding paved road beside a broken column.
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

# ---- military ---------------------------------------------------------------
@icon('fort', 'gold')
def _(g):
    # A timber stockade: sharpened logs, a gate, a lookout and a pennant. Wood
    # and points, not stone and crenels, so it cannot be read as the owner's
    # castle ("fort can't borrow" it).
    g.rect(-27, -19, -14, 0); g.poly([(-31, -18), (-20.5, -30), (-10, -18)]); g.rect(-23, -14, -18, -8, g.k)
    g.rect(18.9, -30, 21.1, -4); g.poly([(21, -30), (33, -26), (21, -22)])
    for i in range(10):                                       # the palisade
        x0 = -31 + i * 6.2
        g.poly([(x0 + 0.5, 26), (x0 + 0.5, 1), (x0 + 3.1, -6.5), (x0 + 5.7, 1), (x0 + 5.7, 26)])
    g.rect(-31, 9, 31, 12.5)
    g.rect(-11, -13, -5.5, 26); g.rect(5.5, -13, 11, 26); g.rect(-13, -15, 13, -9)
    g.rect(-5.5, -9, 5.5, 26, g.k)                           # the open gateway
    g.rect(-30, 24, 30, 27)

@icon('watchtower', 'gold', 1.08)
def _(g):
    # A timber lookout on braced legs.
    g.poly([(-19, -15), (0, -31), (19, -15)]); g.rect(-13, -15, 13, 1)
    g.rect(-8, -10, -2, -3, g.k); g.rect(2, -10, 8, -3, g.k)
    g.rect(-18, 0, 18, 4.5)
    g.line([(-11, 4), (-17, 25)], 4.2); g.line([(11, 4), (17, 25)], 4.2)
    g.line([(-13, 9), (15.5, 21)], 2.8); g.line([(13, 9), (-15.5, 21)], 2.8)
    g.rect(-24, 24, 24, 27)

@icon('fortified_pass', 'gold')
def _(g):
    # A gate tower and wall closing the gap between two peaks.
    g.peak([(-40, 28), (-21, -27), (-2, 28)]); g.peak([(2, 28), (21, -21), (40, 28)])
    g.rect(-21, 3, 21, 30, g.k); g.rect(-11, -17, 11, 30, g.k)
    g.rect(-19, 9, 19, 26); g.crenel(-19, 19, 9, 5, h=4)
    g.rect(-8, -8, 8, 26); g.rect(-9.5, -11, 9.5, -7); g.crenel(-9.5, 9.5, -11, 3, h=4)
    g.arch(0, 4.6, 12, 26); g.rect(-1.4, -3, 1.4, 4, g.k)
    g.rect(-40, 26, 40, 40, g.k)

@icon('fortified_crossing', 'gold')
def _(g):
    # A bridge with a gate tower standing on it.
    g.rect(-40, 4, 40, 18); g.crenel(-36, -12, 4, 4, h=3.4); g.crenel(12, 36, 4, 4, h=3.4)
    g.arch(-23, 8.5, 9.5, 19); g.arch(23, 8.5, 9.5, 19)
    g.rect(-13, -25, 13, 19, g.k)
    g.rect(-9, -15, 9, 18); g.rect(-11, -19, 11, -14); g.crenel(-11, 11, -19, 3, h=4.5)
    g.arch(0, 4.4, -4, 4.5); g.rect(-1.3, -11, 1.3, -6.5, g.k)
    g.rect(-40, 17.5, 40, 40, g.k)
    g.wave(22.5, -30, 30, 4.4, 1.7, 14.5); g.wave(30.5, -20, 20, 4.0, 1.5, 14.5, ph=1.6)

def sword(g, p0, p1, halo=False):
    """A sword from pommel p0 to point p1; halo=True cuts a gap under it."""
    dx, dy = p1[0] - p0[0], p1[1] - p0[1]; n = math.hypot(dx, dy); ux, uy = dx / n, dy / n; px, py = -uy, ux
    at = lambda t, o=0.0: (p0[0] + ux * t + px * o, p0[1] + uy * t + py * o)
    if halo:
        g.poly([at(15, 5.6), at(n - 9, 5.6), at(n + 3, 0), at(n - 9, -5.6), at(15, -5.6)], g.k)
        g.line([at(13, -11), at(13, 11)], 8.4, g.k)
    g.poly([at(13, 3.3), at(n - 8, 3.3), at(n, 0), at(n - 8, -3.3), at(13, -3.3)])
    g.line([at(13, -9), at(13, 9)], 4.4); g.line([at(3, 0), at(13, 0)], 3.8); g.circ(*at(1.5), 3.6)

@icon('battlefield', 'gold', 1.08)
def _(g):
    # Crossed swords.
    sword(g, (-23, 26), (22, -28)); sword(g, (23, 26), (-22, -28), halo=True)

@icon('border_marker', 'gold')
def _(g):
    # A banded boundary post standing on a dashed frontier line. (The old
    # drawing was a headstone with a cross, which is the tomb's picture.)
    g.rect(-9, -19, 9, 22); g.poly([(-9, -18.5), (0, -30), (9, -18.5)])
    g.poly([(-9, -9), (9, -16), (9, -8), (-9, -1)], g.k); g.poly([(-9, 7), (9, 0), (9, 8), (-9, 15)], g.k)
    g.rect(-13, 21, 13, 27)
    for x0 in (-33, -23, 17, 27): g.rect(x0, 21.5, x0 + 6, 26.5)

# ---- sacred and memorial ------------------------------------------------------
@icon('sacred_grove', 'green')
def _(g):
    # One great broadleaf tree between two standing stones, with sparkles: the
    # round crown keeps it apart from the owner's conifer forest.
    for x, y, r in ((0, -14, 13), (-12, -7, 10.5), (12, -7, 10.5), (-7, -20, 9), (7, -20, 9), (0, -3, 9)): g.circ(x, y, r)
    g.poly([(-5.5, 24), (-3, 2), (3, 2), (5.5, 24)]); g.line([(0, 8), (-8, 1)], 3); g.line([(0, 8), (8, 1)], 3)
    for sx in (-1, 1):
        g.poly([(sx * 25, 24), (sx * 24.5, 11), (sx * 21.5, 7.5), (sx * 17.5, 9), (sx * 16, 24)])
    g.rect(-30, 24, 30, 27)
    g.star(25, -22, 7.5, FLAME); g.star(-26, -16, 5, FLAME)

@icon('sacred_mountain', 'green')
def _(g):
    # A lone peak with rays breaking from its summit.
    g.peak([(-35, 28), (0, -15), (35, 28)], cap=0.34)
    for a in (-162, -126, -90, -54, -18):
        ca, sa = math.cos(math.radians(a)), math.sin(math.radians(a))
        g.line([(ca * 9, -15 + sa * 9), (ca * 17.5, -15 + sa * 17.5)], 3.6, FLAME)
    g.rect(-40, 26, 40, 40, g.k)

@icon('ceremonial_site', 'blue')
def _(g):
    # A trilithon between two standing stones, a fire burning inside.
    g.poly([(-17, 24), (-16, -12), (-8, -12), (-8.5, 24)]); g.poly([(8.5, 24), (8, -12), (16, -12), (17, 24)])
    g.poly([(-22, -12), (-21, -21), (20, -22), (22, -12)])
    g.poly([(-31, 24), (-30, 5), (-26.5, 1), (-23, 4), (-22, 24)]); g.poly([(22, 24), (23, 7), (27, 3), (30, 7), (31, 24)])
    g.flame(0, 24, 21, 6.6)
    g.rect(-33, 24, 33, 27)

@icon('tomb', 'blue')
def _(g):
    # A headstone on a grave mound, a second stone leaning behind it.
    g.poly([(17, 20), (19.5, 2), (23, -2), (28, 0), (30, 4), (29, 20)])
    g.chord(-31, 13, 31, 41, 180, 360)
    g.circ(-3, -9, 17.5, g.k); g.rect(-20.5, -9, 14.5, 17, g.k)
    g.circ(-3, -9, 14.5); g.rect(-17.5, -9, 11.5, 22)
    for y, hw in ((-9, 7.5), (-1.5, 8.5), (6, 6)): g.line([(-3 - hw, y), (-3 + hw, y)], 2.6, g.k)
    g.rect(-40, 27, 40, 40, g.k)

@icon('monument', 'blue', 1.08)
def _(g):
    # An obelisk on a stepped plinth.
    g.poly([(-9.5, 15), (-6.5, -21), (0, -32), (6.5, -21), (9.5, 15)])
    g.rect(-13, 15, 13, 19.6); g.rect(-20, 20.6, 20, 24); g.rect(-25, 25, 25, 28)


# ---- assembly ----------------------------------------------------------------
def bodies():
    """The owner's pin body per rim colour: the per-pixel median of its five pins."""
    out = {}
    for cat, names in OWNER_CATS.items():
        st = np.stack([np.asarray(Image.open(OWNER + n + ".png").convert("RGBA")) for n in names])
        out[cat] = np.median(st, axis=0).astype(np.uint8)
    return out

def render(name, body):
    """Compose one pin: the owner's body with this kind's pictogram in the disc."""
    cat, fn, sc = ICONS[name]
    g = Cv(cat, sc); fn(g)
    # Round the corners a little (the owner's shapes are soft-cornered), then a
    # faint top-light on the pictogram, as measured on the owner's castle
    # (about +5 % at the top of the symbol, -5 % at its foot).
    im = g.im.filter(ImageFilter.MedianFilter(7))
    a = np.asarray(im).astype(np.float32)
    k = np.array(g.k, np.float32)
    ink = (np.abs(a - k).sum(-1, keepdims=True) > 40).astype(np.float32)
    yy = np.linspace(-HALF, HALF, N, dtype=np.float32)[:, None, None]
    a = np.clip(a * (1 + ink * (-0.05 * yy / 30.0)), 0, 255)
    # clip to CLIP_R (antialiased at the oversampled scale)
    gy, gx = np.mgrid[0:N, 0:N]
    rr = np.hypot(gx - N / 2 + 0.5, gy - N / 2 + 0.5) / SS
    cl = np.clip((CLIP_R - rr) * SS, 0, 1)[..., None]
    a = a * cl + k * (1 - cl)
    small = np.asarray(Image.fromarray(a.astype(np.uint8)).resize((2 * HALF, 2 * HALF), Image.LANCZOS)).astype(np.float32)
    out = body.astype(np.float32).copy()
    ys, xs = np.mgrid[0:128, 0:110]
    w = np.clip(DISC_R + 0.5 - np.hypot(xs - CX, ys - CY), 0, 1)      # antialiased disc mask
    # canvas pixel (i, j) covers final pixel centre (CX - HALF + 0.5 + j, ...)
    ox, oy = int(round(CX - HALF + 0.5)), int(round(CY - HALF + 0.5))
    pic = np.zeros((128, 110, 3), np.float32) + k
    pic[max(oy, 0):oy + 96, max(ox, 0):ox + 96] = small[max(-oy, 0):, max(-ox, 0):][:128 - max(oy, 0), :110 - max(ox, 0)]
    out[..., :3] = out[..., :3] * (1 - w[..., None]) + pic * w[..., None]
    return Image.fromarray(out.astype(np.uint8), 'RGBA')

def font(sz):
    try: return ImageFont.load_default(size=sz)
    except TypeError: return ImageFont.load_default()

def contact(pins, owner, path, h, bg, cols=10):
    """A labelled sheet: the owner's 20 on top, then the 39, each `h` px tall."""
    w = round(110 * h / 128); cw, ch = max(w + 14, 104), h + 24
    rows_o = math.ceil(len(owner) / cols); rows_p = math.ceil(len(pins) / cols)
    sheet = Image.new('RGBA', (cols * cw + 16, (rows_o + rows_p) * ch + 44), bg + (255,))
    d = ImageDraw.Draw(sheet); f = font(10)
    def put(items, y0):
        for i, (nm, im) in enumerate(items):
            x, y = 8 + (i % cols) * cw, y0 + (i // cols) * ch
            t = im if h == 128 else im.resize((w, h), Image.LANCZOS)
            sheet.alpha_composite(t, (x + (cw - w) // 2, y))
            d.text((x + cw / 2, y + h + 2), nm, fill=(20, 20, 20) if sum(bg) > 300 else (235, 235, 235), font=f, anchor='ma')
    put(owner, 6); y1 = 6 + rows_o * ch + 4
    d.line([(8, y1 + 6), (sheet.width - 8, y1 + 6)], fill=(0, 0, 0, 120), width=1)
    put(pins, y1 + 16)
    sheet.save(path)

def main():
    os.makedirs(OUT, exist_ok=True)
    body = bodies()
    only = sys.argv[1:]
    pins = []
    for name in ICONS:
        im = render(name, body[ICONS[name][0]])
        if not only or name in only: im.save(os.path.join(OUT, name + ".png"))
        pins.append((name, im))
    owner = [(n, Image.open(OWNER + n + ".png").convert("RGBA")) for c in OWNER_CATS for n in OWNER_CATS[c]]
    contact(pins, owner, os.path.join(HERE, "contact_128.png"), 128, (52, 54, 60))
    # mid-tone, map-like ground: the muted green-tan of the app's lowland colours
    contact(pins, owner, os.path.join(HERE, "contact_32.png"), 32, (150, 158, 122), cols=10)
    contact(pins, owner, os.path.join(HERE, "contact_48.png"), 48, (150, 158, 122), cols=10)
    print(len(pins), "pins")

if __name__ == "__main__":
    main()

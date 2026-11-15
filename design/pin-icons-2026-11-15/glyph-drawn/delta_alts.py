"""Delta, rethought from scratch for both sets (owner, 2026-11-15: "the delta
looks like weird it needs a redesign"), plus the small sheet for the repainted
painted smithy.

A delta must read as ONE river splitting into SEVERAL mouths where it meets the
sea; not a trident, a peace sign or a hill. Three ideas, each done in both sets:
    A  map view: land above, sea below, a fan of land pushed into the sea,
       uneven distributaries reaching the coast
    B  three-quarter aerial view: the river widening toward the viewer and
       parting round islands
    C  reeds and mud flats at a river mouth

Writes out_alt/delta_glyph_{A,B,C}.png, painted_A_alt/delta_{A,B,C}.png,
delta_options.png and smithy_painted.png. Never writes into out/, painted_A/
or the repository.
"""
import math, os, sys
import numpy as np
from PIL import Image, ImageDraw
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "painted_A"))
import fable_glyphs as F
from fable_glyphs import icon, bez, WATER, TAN
import painted_A as A
from painted_A import pin, OX, OY, INK
REPO = "C:/Users/Vincent/Cartalith_GDT/cartalith-native/godot-project/icons/pins/"

# ---- glyph ---------------------------------------------------------------------
@icon('delta_glyph_A', 'green')
def _(g):
    # Map view. Land fills the top of the disc edge to edge and bulges into a
    # solid blue sea; five uneven channels run from one river to the coast.
    o = -6                                                    # the whole coast sits this far above centre, so the sea has room
    g.rect(-40, -40, 40, 6 + o); g.ell(-26, -14 + o, 26, 22 + o)
    P = lambda a, b, c: bez((a[0], a[1] + o), (b[0], b[1] + o), (c[0], c[1] + o))
    paths = [bez((2, -40), (-5, -26 + o), (0, -11 + o)), P((0, -11), (-10, -4), (-24.5, 12)), P((-9, -4), (-12, 8), (-13.5, 20.5)),
             P((0, -11), (3, 4), (0.5, 23.5)), P((2, 1), (10, 6), (14.5, 20)), P((0, -11), (12, -6), (24.5, 12))]
    ws = [7, 3.6, 3.0, 3.6, 3.0, 3.4]
    for p, w in zip(paths, ws): g.line(p, w + 2.8, g.k)
    for p, w in zip(paths, ws): g.line(p, w, WATER)
    arc = [(28.6 * math.cos(math.radians(a)), 4 + o + 20.6 * math.sin(math.radians(a))) for a in range(167, 12, -5)]
    g.poly([(-40, 8.6 + o), (-27.9, 8.6 + o)] + arc + [(27.9, 8.6 + o), (40, 8.6 + o), (40, 40), (-40, 40)], WATER)   # the sea, a dark coast line left round the land
    for p, w in zip(paths[1:], ws[1:]):                                                                # open each mouth through that line
        (x0, y0), (x1, y1) = p[-3], p[-1]; n = math.hypot(x1 - x0, y1 - y0)
        g.line([(x1, y1), (x1 + (x1 - x0) / n * 5, y1 + (y1 - y0) / n * 5)], w, WATER)
    g.wave(27.5, -20, 20, 2.2, 1.2, 9, g.k); g.wave(34, -11, 11, 2.2, 1.2, 9, g.k)

@icon('delta_glyph_B', 'green')
def _(g):
    # Aerial view from the land side: the river widens toward the viewer and
    # parts round green islands before the open water.
    g.rect(-40, -40, 40, 40)
    Lc = bez((-3.5, -40), (-7, -10), (-35, 40)); Rc = bez((3.5, -40), (7, -10), (35, 40))
    g.poly([(x - 2.4, y) for x, y in Lc] + [(x + 2.4, y) for x, y in Rc[::-1]], g.k); g.poly(Lc + Rc[::-1], WATER)
    for x0, y0, x1, y1 in ((-4.5, -10, 4.5, 12), (-16.5, 10, -7.5, 31), (7.5, 9, 16.5, 30), (-4, 19, 4, 36)):
        g.ell(x0 - 2, y0 - 2.4, x1 + 2, y1 + 2.4, g.k); g.ell(x0, y0, x1, y1)

@icon('delta_glyph_C', 'green')
def _(g):
    # A river mouth seen from the shore: mud flats, reeds, channels between.
    g.wave(-14, -30, 30, 4.2, 1.6, 13); g.wave(8.5, -6, 34, 4.2, 1.6, 13, ph=1.5); g.wave(30, -20, 20, 4.2, 1.6, 13)
    def bar(x0, y0, x1, y1): g.ell(x0 - 2, y0 - 2, x1 + 2, y1 + 2, g.k); g.ell(x0, y0, x1, y1, TAN)
    def reeds(x, base, hs):
        for i, h in enumerate(hs):
            xx = x + (i - (len(hs) - 1) / 2) * 5.5; lean = (i - (len(hs) - 1) / 2) * 1.6
            g.line([(xx, base), (xx + lean, base - h)], 2.2); g.line([(xx + lean, base - h), (xx + lean * 1.25, base - h - 5)], 4.4)
    bar(4, -7, 34, 2); reeds(20, -4, (15, 20, 14))
    bar(-34, 1, -6, 11); reeds(-20, 5, (16, 23, 17))
    bar(-14, 15, 24, 25); reeds(6, 19, (11, 15, 10))

# ---- painted ---------------------------------------------------------------------
def reed(q, x, base, h, lean=0.0):
    q.lines([((x, base), (x + lean, base - h))], w=0.7, a=1.0, c=(104, 138, 62))
    q.L('woodd', [(x + lean, base - h), (x + lean * 1.2, base - h + 3.6)], 1.7, ink=0.25, t=0.35)

@pin('delta_A')
def _(q):
    # Map view: mainland above, a sand-rimmed fan of low ground pushed into
    # the sea, uneven distributaries reaching the coast, sand bars off the mouths.
    q.water(-45)
    main = q.land(0, -42, 62, 46)
    d = ((q.X - OX) / 28.5) ** 2 + ((q.Y - OY - 3) / 21.5) ** 2 + 0.12 * (q.n3 - 0.5)
    rim = (d < 1).astype(np.float32); q.put(rim, q.col('sand', np.full_like(q.X, 0.22)))
    fan = q.land(0, 2, 25.5, 18.5)
    both = np.maximum(main, rim)
    ch = np.maximum.reduce([q.ml(bez((3, -44), (-5, -27), (0, -11)), 7.5), q.ml(bez((0, -11), (-10, -4), (-26, 13)), 4.6), q.ml(bez((-9, -4), (-12, 8), (-14, 23)), 3.6),
                            q.ml(bez((0, -11), (3, 4), (0.5, 26)), 4.6), q.ml(bez((2, 1), (10, 6), (15, 22)), 3.6), q.ml(bez((0, -11), (12, -6), (26, 13)), 4.2)])
    q.river(ch * both)
    q.water(-45, clip=1 - both)                               # the sea again, so every mouth is open to it
    edge = (q.grow(both, 0.55) - both) * (1 - q.grow(ch, 1.3)); q.put(np.clip(edge, 0, 1), (30, 44, 60), 0.85)
    for x, y, rx in ((-21, 21, 4), (-7, 27.5, 3.4), (8, 27.5, 3.4), (22, 20.5, 4), (0, 31, 2.6)):
        m = q.me(x, y, rx, 1.5); q.line(m, (30, 44, 60), 0.3); q.put(m, (226, 204, 150))
    for x, y in ((-30, -16), (31, -17), (-17, -24), (18, -25), (-27, -4), (28, -5)): q.bush(x, y, 3.2, n=5)

@pin('delta_B')
def _(q):
    # Three-quarter aerial view: the river comes through the far bank and
    # spreads toward the viewer round low green islands with sandy shores.
    q.water(-12)
    q.fill(q.ml([(-1, -30), (1, -12)], 6) * q.disc, 'water', t=0.3, ink=0)
    for sx in (-1, 1):
        q.land(sx * 25, -16, 21, 9); q.pine(sx * 30, -15, 11); q.pine(sx * 20, -17, 13); q.bush(sx * 11, -15, 3.2, n=5)
    for x, y, rx, ry in ((-1, -1, 9, 3.6), (-17, 10, 11, 4.4), (15, 8, 11, 4.2), (-2, 20, 13, 4.8), (-27, 23, 7, 3), (25, 22, 8, 3.2)):
        s = q.me(x, y + 0.5, rx + 2.0, ry + 1.4); q.line(s, (30, 44, 60), 0.4); q.put(s, q.col('sand', np.full_like(q.X, 0.2)))
        q.land(x, y - 0.3, rx, ry)
        for dx in (-rx * 0.5, 0.0, rx * 0.5): reed(q, x + dx, y + 0.5, 5 + ry * 0.6, dx * 0.08)
    for x, y, rx in ((-9, 4.5, 4), (7, 3, 4), (6, 14.5, 5), (-8, 28, 6), (12, 29, 5)): q.put(q.me(x, y, rx, 0.7), (214, 232, 246), 0.6)

@pin('delta_C')
def _(q):
    # A river mouth from the shore: a far bank, mud flats with reeds, channels.
    q.water(-10)
    q.land(-22, -13, 22, 6); q.land(26, -12, 16, 5); q.pine(-28, -13, 12); q.pine(-17, -15, 14); q.pine(27, -12, 11)
    for x, y, rx, ry, hs in ((17, -1, 15, 3.6, (12, 16, 11)), (-19, 9, 16, 4.4, (15, 21, 16, 12)), (6, 22, 19, 5.2, (11, 15, 12, 9))):
        m = q.me(x, y, rx, ry); q.line(m, (30, 44, 60), 0.45)
        q.put(m, q.col('dirt', 0.25 + 0.5 * np.clip((q.Y - OY - y) / ry * 0.5 + 0.5, 0, 1)))
        q.put(q.me(x - rx * 0.2, y - ry * 0.3, rx * 0.5, ry * 0.3), (226, 206, 160), 0.5)
        for i, h in enumerate(hs):
            k = i - (len(hs) - 1) / 2; reed(q, x + k * 3.6, y + 0.5, h, k * 1.1)
        for dx in (-rx * 0.6, rx * 0.55): q.put(q.mp([(x + dx - 1, y), (x + dx, y - 3.4), (x + dx + 1, y)]), (120, 148, 64))
    for x, y, rx in ((-2, 2.5, 6), (22, 9, 7), (-8, 15.5, 7), (-27, 21, 5)): q.put(q.me(x, y, rx, 0.7), (214, 232, 246), 0.6)

# ---- sheets ----------------------------------------------------------------------
def strip(cells, rows_at, sh, d, f, y0, cw):
    for i, (nm, im) in enumerate(cells):
        x = 8 + i * cw
        sh.alpha_composite(im, (x + (cw - im.width) // 2, y0 + 8)); d.text((x + cw / 2, y0 + 142), nm, fill=(235, 235, 235), font=f, anchor="ma")
        for h, xo in ((48, 22), (32, 82)):
            t = im.resize((round(im.width * h / 128), h), Image.LANCZOS)
            sh.alpha_composite(t, (x + xo, y0 + 226 - h)); d.text((x + xo + t.width / 2, y0 + 232), str(h), fill=(20, 20, 20), font=f, anchor="ma")

def sheet(rows, path, cw=132):
    n = max(len(r) for r in rows); W = cw * n + 16
    sh = Image.new("RGBA", (W, 262 * len(rows)), (52, 54, 60, 255)); d = ImageDraw.Draw(sh); f = F.font(11)
    for r, cells in enumerate(rows):
        d.rectangle([0, r * 262 + 166, W, r * 262 + 262], fill=(150, 158, 122, 255)); strip(cells, None, sh, d, f, r * 262, cw)
    sh.save(path)

def main():
    op = lambda p: Image.open(p).convert("RGBA")
    os.makedirs(os.path.join(HERE, "out_alt"), exist_ok=True); os.makedirs(os.path.join(HERE, "painted_A_alt"), exist_ok=True)
    gb = F.bodies()['green']; body, mask = A.body_and_mask()
    g = []; p = []
    for n in "ABC":
        im = F.render('delta_glyph_' + n, gb); im.save(os.path.join(HERE, "out_alt", f"delta_glyph_{n}.png")); g.append((f"glyph {n}", im))
        im = A.render('delta_' + n, body, mask); im.save(os.path.join(HERE, "painted_A_alt", f"delta_{n}.png")); p.append((f"painted {n}", im))
    sheet([[("lake", op(REPO + "glyph/lake.png")), ("river_confluence", op(os.path.join(HERE, "out", "river_confluence.png"))), ("harbour", op(REPO + "glyph/harbour.png"))] + g,
           [("lake", op(REPO + "painted/lake.png")), ("river_confluence", op(os.path.join(HERE, "painted_A", "river_confluence.png"))), ("harbour", op(REPO + "painted/harbour.png"))] + p],
          os.path.join(HERE, "delta_options.png"))
    sheet([[("tavern", op(os.path.join(HERE, "painted_A", "tavern.png"))), ("smithy (new)", op(os.path.join(HERE, "painted_A", "smithy.png"))), ("market", op(REPO + "painted/market.png")),
            ("glyph smithy", op(os.path.join(HERE, "out", "smithy.png")))]], os.path.join(HERE, "smithy_painted.png"), cw=150)

if __name__ == "__main__":
    main()

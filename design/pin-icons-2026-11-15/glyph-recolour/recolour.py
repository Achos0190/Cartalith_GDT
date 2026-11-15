"""Recolour the whole flat glyph pin set by pin-palette category (owner,
2026-11-15: "Do the flat set with colours that are intuitive (based of
games/real world)").

Input (read only): the installed set, icons/pins/glyph/*.png, and the category
table copied from shell/pin_palette.gd `CATEGORIES` (below; keep in step).
Output: recolour/glyph/<name>.png, recolour_sheet.png, recolour_cvd.png, and
glyph_original/ (an untouched copy of the installed files, for comparison).

Two routes, one result:
  * The owner's 20 are images. They are recoloured by a hue / saturation /
    value remap of the pixels that carry the pin's own colour (rim and
    pictogram), so the drawing, the rim gradient and highlight are kept.
    Secondary colours (snow, stone, tan, fire) have other hues and are left.
  * The other 41 are this folder's own drawings. They are re-rendered from
    their generators with the new category colour, on a body made by the same
    remap of the owner's median body, so their secondary colours stay exact.

Water-category pins get a blue rim and pictogram, so their secondary "water"
is drawn in a deeper blue (WATER_ON_BLUE) to keep reading as water.

Colours were chosen by search inside an intuitive hue range per category,
maximising the smallest pairwise CIELAB difference under normal vision,
deuteranopia and protanopia (cvd.py). Must never write into the repository.
"""
import os, sys, colorsys, itertools, shutil
import numpy as np
from PIL import Image, ImageDraw
HERE = os.path.dirname(os.path.abspath(__file__)); ROOT = os.path.join(HERE, "..")
for p in (ROOT, os.path.join(ROOT, "painted_A"), os.path.join(ROOT, "painted_temple"), os.path.join(ROOT, "inn"), os.path.join(ROOT, "notice")): sys.path.insert(0, p)
import fable_glyphs as F
import alts, delta_alts, inn as inn_mod, notice as notice_mod
import cvd
REPO = "C:/Users/Vincent/Cartalith_GDT/cartalith-native/godot-project/icons/pins/glyph/"

# shell/pin_palette.gd CATEGORIES, 2026-11-15. `hazard` is in the table but has no glyph file (painted-only).
CATEGORIES = [
    ("settlements", "Settlements", "capital city town village hamlet"),
    ("buildings", "Buildings & trade", "tavern inn market smithy guild mine quarry salt_works trade_depot caravan_station"),
    ("military", "Military", "castle fort watchtower fortified_pass fortified_crossing gate battlefield border_marker camp"),
    ("sacred", "Sacred & historic", "temple shrine monument ruin tomb sacred_grove sacred_mountain ceremonial_site ancient_road historic_crossing"),
    ("water", "Water", "harbour lake waterfall spring well river_confluence delta ford portage bridge"),
    ("terrain", "Terrain", "mountain mountain_pass ridge cliff gorge rock_formation glacier volcano cave forest"),
    ("land", "Land & hazards", "desert farmland rockfall hazard wildlife"),
    ("other", "Other", "other notice road_junction"),
]
# The chosen colours (see the report in the hand-back and choose_palette.txt for the measured separations).
PALETTE = {
    "settlements": "#F5D039",   # gold: towns and cities, as on most game maps
    "buildings":   "#D4B396",   # tan / parchment: built, man-made, trade; pale so it is not the settlements' gold
    "military":    "#E7344E",   # crimson: danger, war; a blue-leaning red, so it does not collapse onto green for red-green CVD
    "sacred":      "#844DD5",   # violet: magic, the sacred, the old
    "water":       "#64BEEA",   # sky blue
    "terrain":     "#8EE473",   # green: the natural land
    "land":        "#EF7930",   # orange: warning, wild land
    "other":       "#C8CFDA",   # neutral cool grey
}
WATER_ON_BLUE = (44, 108, 196)   # the secondary water colour on a blue (Water) pin: deeper than the rim blue
# The owner's four originals: which source colour each of the owner's 20 was drawn in.
OWNER_SRC = {n: c for c, ns in F.OWNER_CATS.items() for n in ns}
KIND_CAT = {k: cid for cid, _, ks in CATEGORIES for k in ks.split()}
# which of this folder's generators made each installed file (verified pixel-identical before recolouring)
GEN = {"caravan_station": "caravan_station_B", "delta": "delta_glyph_A"}

def rgb(h): return cvd.hexc(h)
def hsv(c): return colorsys.rgb_to_hsv(*(np.asarray(c, float) / 255))

def remap(img, src, dst, tol=30.0, only=None):
    """Move every pixel whose hue is within `tol` degrees of colour `src` (and
    that has some saturation) to colour `dst`'s hue, scaling saturation and
    value by dst/src, so gradients, highlights and antialiased edges keep
    their relative shading. img: float RGBA array. `only`: optional bool mask."""
    a = img[..., :3] / 255.0
    mx, mn = a.max(-1), a.min(-1); v = mx; s = np.where(mx > 0, (mx - mn) / np.maximum(mx, 1e-6), 0)
    d = np.maximum(mx - mn, 1e-6); r, g, b = a[..., 0], a[..., 1], a[..., 2]
    h = np.where(mx == r, ((g - b) / d) % 6, np.where(mx == g, (b - r) / d + 2, (r - g) / d + 4)) * 60.0
    hs, ss, vs = hsv(src); hd, sd, vd = hsv(dst)
    dh = (h - hs * 360 + 180) % 360 - 180
    m = (np.abs(dh) < tol) & (s > 0.16)
    if only is not None: m &= only
    h2 = (hd * 360 + dh * 0.5) % 360; s2 = np.clip(s * sd / max(ss, 1e-3), 0, 1); v2 = np.clip(v * vd / max(vs, 1e-3), 0, 1)
    c = v2 * s2; x = c * (1 - np.abs((h2 / 60) % 2 - 1)); mm = v2 - c; k = (h2 // 60).astype(int) % 6
    z = np.zeros_like(c)
    R = np.choose(k, [c, x, z, z, x, c]) + mm; G = np.choose(k, [x, c, c, x, z, z]) + mm; B = np.choose(k, [z, z, x, c, c, x]) + mm
    out = img.copy()
    out[..., :3] = np.where(m[..., None], np.stack([R, G, B], -1) * 255.0, img[..., :3])
    return out

def disc_tint(col):
    """The dark disc, tinted a little toward the pin's hue, as the owner's blue and green pins are."""
    h, s, v = hsv(col)
    return tuple(int(round(x * 255)) for x in colorsys.hsv_to_rgb(h, min(0.42, s * 0.55), 0.092))

def tint_disc(img, col, cx, cy):
    """Pull the near-black pixels of the disc toward this colour's disc tint."""
    ys, xs = np.mgrid[0:img.shape[0], 0:img.shape[1]]
    m = (np.hypot(xs - cx, ys - cy) < 44.5) & (img[..., :3].max(-1) < 34)
    out = img.copy(); out[..., :3] = np.where(m[..., None], np.asarray(disc_tint(col), float), img[..., :3]); return out

def nearest_src(col):
    """The owner body whose hue is closest, to remap into the new colour's body."""
    h = hsv(col)[0]
    return min(F.PIC, key=lambda c: min(abs(hsv(F.PIC[c])[0] - h), 1 - abs(hsv(F.PIC[c])[0] - h)))

def main():
    out_dir = os.path.join(HERE, "glyph"); orig_dir = os.path.join(HERE, "glyph_original")
    os.makedirs(out_dir, exist_ok=True); os.makedirs(orig_dir, exist_ok=True)
    base_bodies = {c: b.astype(np.float32) for c, b in F.bodies().items()}
    base_pic, base_disc = dict(F.PIC), dict(F.DISC)
    # new bodies and colours, registered beside the owner's four under the category ids
    bodies = {}
    for cid, hx in PALETTE.items():
        col = rgb(hx); src = nearest_src(col)
        b = tint_disc(remap(base_bodies[src], base_pic[src], col), col, F.CX, F.CY)
        bodies[cid] = np.clip(b, 0, 255).astype(np.uint8); F.PIC[cid] = col; F.DISC[cid] = disc_tint(col)
    wave_defaults = F.Cv.wave.__defaults__
    done = {}
    for name in sorted(os.path.basename(f)[:-4] for f in os.listdir(REPO) if f.endswith(".png")):
        shutil.copyfile(REPO + name + ".png", os.path.join(orig_dir, name + ".png"))
        cid = KIND_CAT[name]; col = rgb(PALETTE[cid])
        if name in OWNER_SRC:                                   # the owner's drawing: remap the image
            img = np.asarray(Image.open(REPO + name + ".png").convert("RGBA")).astype(np.float32)
            src = OWNER_SRC[name]
            if cid == "water" and src != "blue":                # lake, waterfall: their blue water first, to the deeper blue
                img = remap(img, base_pic["blue"], WATER_ON_BLUE, tol=24)
                ys, xs = np.mgrid[0:128, 0:110]                 # (the rim is not blue on these, so only pictogram water moves)
            img = tint_disc(remap(img, base_pic[src], col), col, F.CX, F.CY)
            im = Image.fromarray(np.clip(img, 0, 255).astype(np.uint8), "RGBA")
        else:                                                   # this folder's drawing: re-render in the new colour
            key = GEN.get(name, name); _, fn, sc = F.ICONS[key]
            F.ICONS[key] = (cid, fn, sc)
            w = WATER_ON_BLUE if cid == "water" else F.WATER
            saved = (F.WATER, alts.WATER, delta_alts.WATER)
            if cid == "water":
                F.WATER = alts.WATER = delta_alts.WATER = w; F.Cv.wave.__defaults__ = wave_defaults[:3] + (w,) + wave_defaults[4:]
            im = F.render(key, bodies[cid])
            F.WATER, alts.WATER, delta_alts.WATER = saved; F.Cv.wave.__defaults__ = wave_defaults
        im.save(os.path.join(out_dir, name + ".png")); done[name] = im
    sheets(done)
    measure()

def strip32(done, bg=(150, 158, 122)):
    """All pins at 32 px on the map-like ground, in category order, a gap between categories."""
    xs = 6; items = []
    for cid, title, ks in CATEGORIES:
        for k in ks.split():
            if k in done: items.append((xs, done[k])); xs += 31
        xs += 12
    rows = 2; per = (xs + 1) // rows + 40
    im = Image.new("RGBA", (per, 44 * rows), bg + (255,))
    for x, pin in items:
        r = x // (per - 34); t = pin.resize((28, 32), Image.LANCZOS); im.alpha_composite(t, (6 + x - r * (per - 34), 6 + r * 44))
    return im

def sheets(done):
    f = F.font(12); fs = F.font(10)
    W = 16 + 10 * 112; rows = [(cid, title, [k for k in ks.split() if k in done]) for cid, title, ks in CATEGORIES]
    strip = strip32(done)
    H = sum(26 + 150 for _ in rows) + strip.height + 40
    sh = Image.new("RGBA", (max(W, strip.width + 16), H), (52, 54, 60, 255)); d = ImageDraw.Draw(sh); y = 6
    for cid, title, ks in rows:
        d.rectangle([8, y + 4, 20, y + 16], fill=rgb(PALETTE[cid])); d.text((26, y + 3), f"{title}   {PALETTE[cid]}", fill=(240, 240, 240), font=f)
        for i, k in enumerate(ks):
            sh.alpha_composite(done[k], (8 + i * 112, y + 24)); d.text((8 + i * 112 + 55, y + 153), k, fill=(220, 220, 220), font=fs, anchor="ma")
        y += 176
    d.text((8, y + 2), "32 px on the map ground, in the same order", fill=(240, 240, 240), font=f); sh.alpha_composite(strip, (8, y + 22))
    sh.save(os.path.join(HERE, "recolour_sheet.png"))
    # CVD sheet: the same strip as seen with normal vision, deuteranopia and protanopia, and the eight swatches
    sims = [("normal vision", None), ("deuteranopia (simulated)", cvd.DEUTAN), ("protanopia (simulated)", cvd.PROTAN)]
    sw = 44 * 8 + 16
    cs = Image.new("RGBA", (max(strip.width, sw) + 16, (strip.height + 86) * 3 + 8), (52, 54, 60, 255)); d = ImageDraw.Draw(cs); y = 6
    for label, M in sims:
        d.text((8, y), label, fill=(240, 240, 240), font=f)
        a = np.asarray(strip.convert("RGB")).astype(np.float64)
        if M is not None: a = cvd.simulate(a, M)
        cs.paste(Image.fromarray(np.clip(a, 0, 255).astype(np.uint8)), (8, y + 20))
        for i, (cid, title, _) in enumerate(CATEGORIES):
            c = np.array(rgb(PALETTE[cid]), float); c = c if M is None else cvd.simulate(c, M)
            d.rectangle([8 + i * 44, y + 26 + strip.height, 8 + i * 44 + 38, y + 26 + strip.height + 22], fill=tuple(int(v) for v in np.clip(c, 0, 255)))
            d.text((8 + i * 44 + 19, y + 50 + strip.height), title.split()[0][:7], fill=(220, 220, 220), font=fs, anchor="ma")
        y += strip.height + 86
    cs.save(os.path.join(HERE, "recolour_cvd.png"))

def measure():
    """Print and save the smallest pairwise colour differences in each vision mode."""
    lines = []
    for label, M in (("normal", None), ("deuteranopia", cvd.DEUTAN), ("protanopia", cvd.PROTAN)):
        cols = {c: (np.array(rgb(h), float) if M is None else cvd.simulate(np.array(rgb(h), float), M)) for c, h in PALETTE.items()}
        pairs = sorted((cvd.dE(cols[a], cols[b]), a, b) for a, b in itertools.combinations(PALETTE, 2))
        lines.append(f"{label:13s} closest pairs (CIELAB dE): " + ", ".join(f"{a}/{b} {d_:.0f}" for d_, a, b in pairs[:4]))
    open(os.path.join(HERE, "choose_palette.txt"), "w").write("\n".join(lines) + "\n"); print("\n".join(lines))

if __name__ == "__main__":
    main()

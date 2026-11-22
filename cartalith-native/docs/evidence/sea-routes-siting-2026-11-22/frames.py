"""Schematic map frames from the census dumps (`SEA_CENSUS_DUMP`, see
`sea_site_census.rs`): the generator's own ocean/lake/land classes, lanes, ports
and towns, drawn from the real `CivData`. Not a render of the app's map -- a
data frame, so what it shows is exactly what the engine produced.
Usage: python frames.py <dump_dir> <out_dir>"""
import json, sys, os
import numpy as np
from PIL import Image, ImageDraw

# One colour per lane: threaded (side-by-side) lanes show as two colours, a shared trunk as one.
PALETTE = [(200, 30, 30), (20, 20, 20), (150, 40, 190), (230, 130, 0), (10, 130, 60), (200, 20, 130), (90, 60, 20), (0, 90, 160)]


def load(p):
    return json.load(open(p))

def base(d, scale):
    gw, gh = d["gw"], d["gh"]
    wb = np.array(d["wb"], dtype=np.uint8).reshape(gh, gw)
    img = np.zeros((gh, gw, 3), np.uint8)
    img[:] = (214, 205, 176)            # land
    img[wb == 1] = (126, 164, 196)      # ocean
    img[wb == 2] = (160, 190, 210)      # lake / inland sea
    im = Image.fromarray(img).resize((int(gw * scale), int(gh * scale)), Image.NEAREST)
    return im

def draw(d, scale, box=None, lanes=True, label=""):
    im = base(d, scale)
    dr = ImageDraw.Draw(im)
    if lanes:
        for li, pts in enumerate(d["lanes"]):
            col = PALETTE[li % len(PALETTE)]
            for a, b in zip(pts, pts[1:]):
                if abs(a[0] - b[0]) > d["gw"] / 2:
                    continue
                dr.line([(a[0] * scale, a[1] * scale), (b[0] * scale, b[1] * scale)], fill=col, width=max(2, int(scale * 1.6)))
    for t in d["towns"]:
        r = max(2, int(scale * 1.2))
        dr.ellipse([t[0] * scale - r, t[1] * scale - r, t[0] * scale + r, t[1] * scale + r], fill=(60, 60, 60))
    for p in d["ports"]:
        r = max(4, int(scale * 2.6))
        dr.ellipse([p[0] * scale - r, p[1] * scale - r, p[0] * scale + r, p[1] * scale + r], outline=(250, 220, 40), width=3)
    if box:
        im = im.crop(tuple(int(v * scale) for v in box))
    dr = ImageDraw.Draw(im)
    dr.rectangle([0, 0, im.width, 22], fill=(255, 255, 255))
    dr.text((6, 5), label, fill=(0, 0, 0))
    return im

def pair(dump, out, tag, scale, box=None):
    off = load(f"{dump}/{tag}_lanes_off.json")
    on = load(f"{dump}/{tag}_lanes_on.json")
    a = draw(off, scale, box, label=f"{tag}  civ.sea_lanes OFF (before)  lanes={len(off['lanes'])}")
    b = draw(on, scale, box, label=f"{tag}  civ.sea_lanes ON (after)  lanes={len(on['lanes'])}")
    w = Image.new("RGB", (a.width + b.width + 8, a.height), (255, 255, 255))
    w.paste(a, (0, 0)); w.paste(b, (a.width + 8, 0))
    w.save(f"{out}/{tag.replace('@', '_')}_sea_before_after{'_crop' if box else ''}.png")

if __name__ == "__main__":
    dump, out = sys.argv[1], sys.argv[2]
    os.makedirs(out, exist_ok=True)
    pair(dump, out, "owner83744", 0.9)
    for t in ["483920@2048", "912345@2048", "483920@384", "246371@384"]:
        pair(dump, out, t, 1.0 if "2048" not in t else 0.45)


def confluences(d):
    """Cells that two or more river polylines pass through (the engine's own rule)."""
    cnt = {}
    for pl in d["rivers"]:
        last = None
        for x, y in pl:
            c = (int(x), int(y))
            if c != last:
                cnt[c] = cnt.get(c, 0) + 1
                last = c
    return [c for c, n in cnt.items() if n >= 2]

def site(d, scale, box, label):
    im = base(d, scale)
    dr = ImageDraw.Draw(im)
    for pl in d["rivers"]:
        for a, b in zip(pl, pl[1:]):
            dr.line([(a[0] * scale, a[1] * scale), (b[0] * scale, b[1] * scale)], fill=(70, 120, 190), width=max(1, int(scale * 0.8)))
    for c in confluences(d):
        r = max(3, int(scale * 1.6))
        dr.ellipse([c[0] * scale - r, c[1] * scale - r, c[0] * scale + r, c[1] * scale + r], outline=(20, 150, 60), width=2)
    for t in d["towns"]:
        r = max(3, int(scale * 1.4))
        col = (200, 40, 40) if t[2] else (40, 40, 40)
        dr.ellipse([t[0] * scale - r, t[1] * scale - r, t[0] * scale + r, t[1] * scale + r], fill=col)
    if box:
        im = im.crop(tuple(int(v * scale) for v in box))
    dr = ImageDraw.Draw(im)
    dr.rectangle([0, 0, im.width, 22], fill=(255, 255, 255))
    dr.text((6, 5), label, fill=(0, 0, 0))
    return im

def site_pair(dump, out, tag, scale, box=None):
    off = load(f"{dump}/{tag}_site_off.json")
    on = load(f"{dump}/{tag}_site_on.json")
    a = site(off, scale, box, f"{tag}  civ.site_map_terms OFF (before); green ring = confluence, red = harbour, dark = other")
    b = site(on, scale, box, f"{tag}  civ.site_map_terms ON (after)")
    w = Image.new("RGB", (a.width + b.width + 8, a.height), (255, 255, 255))
    w.paste(a, (0, 0)); w.paste(b, (a.width + 8, 0))
    w.save(f"{out}/{tag.replace('@', '_')}_siting_before_after{'_crop' if box else ''}.png")

if __name__ == "__main__":
    site_pair(dump, out, "owner83744", 0.9)
    site_pair(dump, out, "483920@2048", 0.5, (0, 0, 1000, 640))
    site_pair(dump, out, "912345@384", 2.0)

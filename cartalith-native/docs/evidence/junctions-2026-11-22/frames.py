"""Schematic before/after frames from the census dumps (`JUNCTION_DUMP`, see
`junction_census.rs`): the generator's own water classes, river cells, base roads, settlements and
the hamlets founded at junctions (red rings), drawn from the real `CivData`. Not a render of the
app's map -- a data frame. Before = `civ.junction_settlements` off; after = on (founded marked).
Usage: python frames.py <dump_dir> <out_dir>"""
import json, sys, os
import numpy as np
from PIL import Image, ImageDraw

def draw(d, scale, founded, label):
    gw, gh = d["gw"], d["gh"]
    wb = np.array(d["wb"], dtype=np.uint8).reshape(gh, gw)
    img = np.zeros((gh, gw, 3), np.uint8)
    img[:] = (214, 205, 176)
    img[wb == 1] = (126, 164, 196)
    img[wb == 2] = (160, 190, 210)
    rv = np.zeros(gw * gh, bool); rv[d["river"]] = True
    img[rv.reshape(gh, gw)] = (90, 140, 200)
    im = Image.fromarray(img).resize((int(gw * scale), int(gh * scale)), Image.NEAREST)
    dr = ImageDraw.Draw(im)
    for pts in d["ways"]:
        for a, b in zip(pts, pts[1:]):
            if abs(a[0] - b[0]) > gw / 2: continue
            dr.line([(a[0] * scale, a[1] * scale), (b[0] * scale, b[1] * scale)], fill=(120, 80, 40), width=max(1, int(scale * 1.2)))
    for t in d["settlements"]:
        r = max(2, int(scale * 1.0))
        dr.ellipse([t[0] * scale - r, t[1] * scale - r, t[0] * scale + r, t[1] * scale + r], fill=(60, 60, 60))
    if founded:
        for t in d["founded"]:
            r = max(5, int(scale * 3))
            dr.ellipse([t[0] * scale - r, t[1] * scale - r, t[0] * scale + r, t[1] * scale + r], outline=(200, 20, 20), width=3)
            dr.ellipse([t[0] * scale - 2, t[1] * scale - 2, t[0] * scale + 2, t[1] * scale + 2], fill=(200, 20, 20))
    dr.rectangle([0, 0, im.width, 22], fill=(255, 255, 255))
    dr.text((6, 5), label, fill=(0, 0, 0))
    return im

if __name__ == "__main__":
    dump, out = sys.argv[1], sys.argv[2]
    os.makedirs(out, exist_ok=True)
    for f in sorted(os.listdir(dump)):
        if not f.endswith(".json"): continue
        tag = f[:-5]; d = json.load(open(f"{dump}/{f}"))
        sc = 1.8 if d["gw"] < 1000 else 0.5
        a = draw(d, sc, False, f"{tag}  civ.junction_settlements OFF (before)")
        b = draw(d, sc, True, f"{tag}  ON (after): {len(d['founded'])} hamlets founded at road junctions (red)")
        w = Image.new("RGB", (a.width + b.width + 8, a.height), (255, 255, 255))
        w.paste(a, (0, 0)); w.paste(b, (a.width + 8, 0))
        w.save(f"{out}/{tag}_junction_before_after.png")

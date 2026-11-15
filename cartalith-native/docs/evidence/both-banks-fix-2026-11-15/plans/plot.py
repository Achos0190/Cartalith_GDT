"""Quick plan plot of a town dumped by both_banks_audit (TOWN_RIVER_PLOT=<dir>).
usage: python plot.py <json> [<json> ...] <out.png>   (side by side)
A data plot of the layout, not the app's renderer."""
import json, sys
from PIL import Image, ImageDraw

COL = {"primary": (200, 60, 40), "street": (90, 90, 90), "lane": (150, 150, 150),
       "quay": (40, 120, 200), "bank": (240, 0, 200), "ringroad": (120, 60, 160)}

def frame(path, size=900):
    d = json.load(open(path))
    pts = [p for b in d["buildings"] for p in b] + d.get("ring", []) or [d["market"]]
    xs = [p[0] for p in pts]; ys = [p[1] for p in pts]
    x0, x1, y0, y1 = min(xs) - 60, max(xs) + 60, min(ys) - 60, max(ys) + 60
    s = size / max(x1 - x0, y1 - y0)
    P = lambda p: ((p[0] - x0) * s, (p[1] - y0) * s + 30)
    img = Image.new("RGB", (size, size + 30), (240, 238, 228))
    dr = ImageDraw.Draw(img)
    if len(d["water_poly"]) >= 3:
        dr.polygon([P(p) for p in d["water_poly"]], fill=(160, 200, 235))
    r = d["river"]
    if len(r) >= 2:
        dr.line([P(p) for p in r], fill=(160, 200, 235), width=max(1, int(d["river_w"] * s)))
    for cls, poly in d["parcels"]:
        if len(poly) >= 3:
            dr.polygon([P(p) for p in poly], outline=(215, 205, 185))
    for b in d["buildings"]:
        if len(b) >= 3:
            dr.polygon([P(p) for p in b], fill=(150, 90, 70))
    for cls, w, ax, ay, bx, by in d["edges"]:
        dr.line([P((ax, ay)), P((bx, by))], fill=COL.get(cls, (0, 0, 0)), width=max(1, int(w * s)))
    if len(d["ring"]) >= 3:
        dr.line([P(p) for p in d["ring"] + d["ring"][:1]], fill=(20, 20, 20), width=3)
    pz = d.get("plaza", [])
    if len(pz) >= 3:
        dr.polygon([P(p) for p in pz], outline=(255, 140, 0), width=3)
    m = P(d["market"]); dr.ellipse([m[0] - 5, m[1] - 5, m[0] + 5, m[1] + 5], fill=(255, 200, 0))
    dr.text((5, 5), f"{path.split('/')[-1]}  pop {d['pop']} / {d['target']}  ({(x1-x0):.0f} m wide)", fill=(0, 0, 0))
    return img

def main():
    *ins, out = sys.argv[1:]
    tiles = [frame(p) for p in ins]
    img = Image.new("RGB", (sum(t.width for t in tiles) + 10 * (len(tiles) - 1), max(t.height for t in tiles)), "white")
    x = 0
    for t in tiles:
        img.paste(t, (x, 0)); x += t.width + 10
    img.save(out)

main()

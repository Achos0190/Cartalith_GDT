"""Plot a town dump of `_towncensus_probe.gd --dump` (WorldGen.town_debug_json) to a PNG.

  python -I plot_town.py town_<seed>_<i>_<state>.json [out.png] [title]

The map's water under the town (blue: a drawn river stroke at its ground width; navy: sea; teal: lake; pale blue: the
town's own water raster where the map draws land), then the town: buildings (grey), streets (brown; quay green), the
wall ring (orange) with gates (white; a water gate cyan), the detected bridges (yellow), the market (red) and the
settlement's cell (magenta cross). 1 px = step / SCALE metres."""
import json, sys
from PIL import Image, ImageDraw
src = sys.argv[1]; out = sys.argv[2] if len(sys.argv) > 2 else src[:-5] + ".png"; title = sys.argv[3] if len(sys.argv) > 3 else src.split("/")[-1]
d = json.load(open(src)); S = 3; st = d["step"]; k = S / st
im = Image.new("RGB", (d["gw"] * S, d["gh"] * S), (226, 220, 196)); px = im.load()
COL = {"1": (40, 60, 120), "2": (60, 140, 150), "3": (90, 140, 220), "4": (190, 215, 240)}
g = d["grid"]
for j in range(d["gh"]):
    for i in range(d["gw"]):
        c = COL.get(g[j * d["gw"] + i])
        if c:
            for y in range(S):
                for x in range(S): px[i * S + x, j * S + y] = c
dr = ImageDraw.Draw(im)
P = lambda x, y: (x * k, y * k)
for b in d["buildings"]: dr.polygon([P(*v) for v in b], fill=(90, 90, 90))
for e in d["edges"]: dr.line([P(e[0], e[1]), P(e[2], e[3])], fill=(40, 150, 60) if e[4] == "quay" else ((150, 60, 20) if e[4] == "primary" else (140, 100, 60)), width=2 if e[4] == "primary" else 1)
if d["ring"]: dr.line([P(*v) for v in d["ring"]] + [P(*d["ring"][0])], fill=(240, 130, 0), width=2)
for gt in d["gates"]: dr.ellipse([gt[0] * k - 4, gt[1] * k - 4, gt[0] * k + 4, gt[1] * k + 4], fill=(0, 230, 230) if gt[2] else (255, 255, 255), outline=(0, 0, 0))
for b in d["bridges"]: dr.ellipse([b[0] * k - 4, b[1] * k - 4, b[0] * k + 4, b[1] * k + 4], fill=(250, 230, 0), outline=(0, 0, 0))
if len(d["river"]) > 1: dr.line([P(*v) for v in d["river"]], fill=(255, 255, 255), width=1)
m = d["market"]; dr.rectangle([m[0] * k - 5, m[1] * k - 5, m[0] * k + 5, m[1] * k + 5], fill=(230, 0, 0))
p = d["pin"]; dr.line([p[0] * k - 8, p[1] * k, p[0] * k + 8, p[1] * k], fill=(230, 0, 230), width=2); dr.line([p[0] * k, p[1] * k - 8, p[0] * k, p[1] * k + 8], fill=(230, 0, 230), width=2)
dr.text((4, 4), "%s  %s  %.0f x %.0f m  river %.0f m  housed %.0f of %.0f" % (title, d["site_kind"], d["wm"], d["hm"], d["river_w"], d["pop"], d["pop_target"]), fill=(0, 0, 0))
im.save(out); print(out, im.size)

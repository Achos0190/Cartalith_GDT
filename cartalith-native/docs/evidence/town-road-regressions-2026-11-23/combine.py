"""Combine the frames into combined.png: one row per town and state, columns map x4, x16, x64 and the City Viewer.

  python -I combine.py FRAMES_DIR OUT_PNG

FRAMES_DIR holds `<world>/desktop_t<i>_<state>_<map_x4|map_x16|map_x64|city_x1>.png` with world `f4096`/`f384`
(this fix: states `off` and `on`) and `h4096`/`h384` (the build before it, `head_on`)."""
import pathlib, sys
from PIL import Image, ImageDraw

d, out = pathlib.Path(sys.argv[1]), sys.argv[2]
W = 300
rows = []
for world, label, towns in (("4096", "seed 51203 at 4096x2048 (app defaults)", "24,130,14,1"), ("384", "seed 483920 at 384x288 (census)", "19,0,10")):
    for t in towns.split(","):
        for sub, state, name in (("h" + world, "head_on", "BEFORE (e74e7312, civ.bank_sites on)"), ("f" + world, "on", "AFTER (this fix, civ.bank_sites on)")):
            cells = []
            for col in ("map_x4", "map_x16", "map_x64", "city_x1"):
                p = d / sub / ("desktop_t%s_%s_%s.png" % (t, state, col))
                cells.append(Image.open(p).convert("RGB").resize((W, W)) if p.exists() else Image.new("RGB", (W, W), (40, 40, 40)))
            rows.append(("%s #%s %s" % (label, t, name), cells))
img = Image.new("RGB", (W * 4, len(rows) * (W + 20)), (20, 20, 24))
dr = ImageDraw.Draw(img)
for r, (label, cells) in enumerate(rows):
    y = r * (W + 20)
    dr.text((4, y + 4), label + "   |  map x4  |  map x16  |  map x64  |  City Viewer", fill=(230, 230, 230))
    for c, im in enumerate(cells):
        img.paste(im, (c * W, y + 20))
img.save(out)
print("wrote", out, img.size)

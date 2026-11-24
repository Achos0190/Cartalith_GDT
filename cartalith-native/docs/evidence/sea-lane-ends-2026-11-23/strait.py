"""Strait frames from census dumps: python strait.py <out.png> <cx> <cy> <half_w_cells> <scale> <label>=<dump.json> ...
Draws each dump's ocean/land, lanes (one colour per lane) and ports, cropped to the box, side by side."""
import sys, json
sys.path.insert(0, r"C:/cw/lanes/cartalith-native/docs/evidence/sea-routes-siting-2026-11-22")
from frames import draw
from PIL import Image

out, cx, cy, hw, scale = sys.argv[1], float(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4]), float(sys.argv[5])
ims = []
for arg in sys.argv[6:]:
    label, path = arg.split("=", 1)
    d = json.load(open(path))
    hh = hw * 0.62
    box = (max(0, cx - hw), max(0, cy - hh), min(d["gw"], cx + hw), min(d["gh"], cy + hh))
    ims.append(draw(d, scale, box, label=f"{label}  lanes={len(d['lanes'])}"))
W = sum(i.width for i in ims) + 8 * (len(ims) - 1)
H = max(i.height for i in ims)
c = Image.new("RGB", (W, H), (255, 255, 255))
x = 0
for i in ims:
    c.paste(i, (x, 0)); x += i.width + 8
c.save(out)
print(out, c.size)

"""Frame script for the town-on-its-river census (owner report, 2026-11-14).

Reads the JSON that `town_river_census` writes with TOWN_RIVER_DUMP=<dir> and
draws, in ONE frame and in the layout's local metres (the map's own town-layer
transform, so the frame is what the world map shows at that spot):
  - light blue squares: the world's channel cells (flow above the river threshold);
  - dark blue line:     the world's traced river polylines (what the map strokes);
  - red line:           the town layout's own river (what the City Viewer draws);
  - grey ring / tan blocks: the town's wall circuit and street blocks;
  - black outline:      the settlement's own grid cell; black dot: its marker.
A data plot of the engine's output, not a capture of the app's renderer.

usage: python render.py <dump_dir> <out_png> <seed>_<idx> [<seed>_<idx> ...] [--half-km N]
"""
import json, sys
from PIL import Image, ImageDraw, ImageFont

def main():
    args = sys.argv[1:]
    half_km = 3.0
    if "--half-km" in args:
        k = args.index("--half-km"); half_km = float(args[k + 1]); del args[k:k + 2]
    src, out, names = args[0], args[1], args[2:]
    tiles = [frame(json.load(open(f"{src}/{n}.json")), half_km) for n in names]
    w = sum(t.width for t in tiles) + 10 * (len(tiles) - 1)
    img = Image.new("RGB", (w, max(t.height for t in tiles)), "white")
    x = 0
    for t in tiles:
        img.paste(t, (x, 0)); x += t.width + 10
    img.save(out)

def frame(d, half_km, size=640):
    img = Image.new("RGB", (size, size + 40), (236, 236, 228))
    dr = ImageDraw.Draw(img)
    mx, my = d["market"]
    s = size / (2000.0 * half_km)  # px per metre
    P = lambda p: ((p[0] - mx) * s + size / 2, (p[1] - my) * s + size / 2 + 40)
    c = d["cell_m"]
    for (x, y) in d["channel_cells"]:
        dr.rectangle([P((x, y)), P((x + c, y + c))], fill=(170, 205, 235))
    ox, oy = d["own_cell"]
    dr.rectangle([P((ox, oy)), P((ox + c, oy + c))], outline=(0, 0, 0), width=2)
    for b in d["blocks"]:
        if len(b) >= 3:
            dr.polygon([P(p) for p in b], fill=(214, 196, 150))
    ring = d["ring"]
    if len(ring) >= 3:
        dr.line([P(p) for p in ring + ring[:1]], fill=(110, 110, 110), width=3)
    rw = max(2, int(d["river_w"] * s))
    for pl in d["world_rivers"]:
        if len(pl) >= 2:
            dr.line([P(p) for p in pl], fill=(20, 60, 160), width=max(3, rw))
    if len(d["river"]) >= 2:
        dr.line([P(p) for p in d["river"]], fill=(220, 30, 30), width=max(2, rw // 2))
    q = P((mx, my))
    dr.ellipse([q[0] - 6, q[1] - 6, q[0] + 6, q[1] + 6], fill=(0, 0, 0))
    dr.rectangle([0, 0, size, 40], fill="white")
    dr.text((6, 4), f"seed {d['seed']} #{d['idx']} ({d['site_kind']}), {2*half_km:.0f} km across, cell {c:.0f} m", fill="black")
    dr.text((6, 22), "blue: world river + channel cells   red: town layout's river   box: own cell", fill="black")
    return img

main()

import sys, pathlib
from PIL import Image, ImageDraw
src = pathlib.Path(sys.argv[1]); dst = pathlib.Path(sys.argv[2]); phone = pathlib.Path(sys.argv[3]); big = pathlib.Path(sys.argv[4]); exp = pathlib.Path(sys.argv[5])
dst.mkdir(parents=True, exist_ok=True)
MAP = (372, 118, 1320, 850)
def load(d, n): return Image.open(d / n).convert("RGB")
def tile(d, n, box, scale, label):
    im = load(d, n).crop(box); im = im.resize((int(im.width * scale), int(im.height * scale)), Image.LANCZOS)
    dr = ImageDraw.Draw(im); dr.rectangle([0, 0, im.width, 15], fill=(0, 0, 0)); dr.text((4, 2), label, fill=(255, 255, 255)); return im
def sheet(tiles, name, cols):
    w, h = tiles[0].size; rows = (len(tiles) + cols - 1) // cols
    out = Image.new("RGB", (w * cols, h * rows), (20, 20, 20))
    for i, t in enumerate(tiles): out.paste(t, ((i % cols) * w, (i // cols) * h))
    out.save(dst / name); print(name, out.size)
t = []
for z in ["x1", "x4", "x16", "x64"]:
    t += [tile(src, f"desktop_{z}_off.png", MAP, .5, f"{z}  Coastline off (before)"), tile(src, f"desktop_{z}_on.png", MAP, .5, f"{z}  Coastline on (after)")]
sheet(t, "combined_before_after_desktop.png", 2)
t = []
for z in ["x2", "x16"]:
    t += [tile(src, f"desktop_{z}_shoreborder_coastoff.png", MAP, .4, f"{z} coast off"), tile(src, f"desktop_{z}_shoreborder_borders_on.png", MAP, .4, f"{z} borders on, coast on"), tile(src, f"desktop_{z}_shoreborder_borders_off.png", MAP, .4, f"{z} borders off, coast on")]
sheet(t, "combined_shore_border.png", 3)
t = []
for k in ["coast", "national", "province"]:
    for p in ["Antique", "Blueprint", "Default"]:
        t.append(tile(src, f"desktop_ink_{k}_{p}.png", (570, 330, 1120, 640), .6, f"{k} line, {p} ink (terrain unchanged)"))
sheet(t, "combined_preset_ink.png", 3)
t = []
for k in ["coast", "national", "province"]:
    for p in ["Antique", "Blueprint", "Night"]:
        t.append(tile(src, f"desktop_look_{p}_{k}.png", (570, 330, 1120, 640), .6, f"{k} line over the {p} Look"))
sheet(t, "combined_looks.png", 3)
t = []
for i in (1, 2, 3):
    t += [tile(src, f"desktop_outlier{i}_x16_off.png", MAP, .4, f"x4 miss {i} at x16, coast off"), tile(src, f"desktop_outlier{i}_x16_on.png", MAP, .4, f"x4 miss {i} at x16, coast on")]
sheet(t, "combined_x4_misses_at_x16.png", 2)
t = []
for z in ["x1", "x4", "x16", "x64"]:
    t += [tile(phone, f"phone_{z}_off.png", (0, 0, 450, 975), .45, f"phone {z} off"), tile(phone, f"phone_{z}_on.png", (0, 0, 450, 975), .45, f"phone {z} on")]
sheet(t, "combined_phone.png", 4)
t = []
for z in ["x1", "x4"]:
    t += [tile(big, f"desktop_{z}_off.png", MAP, .5, f"2048 world {z} off"), tile(big, f"desktop_{z}_on.png", MAP, .5, f"2048 world {z} on")]
sheet(t, "combined_2048_world.png", 2)

# the 2048 px export: the same 450 x 300 px window of the file without and with the coastline (crop, not the file)
t = [tile(exp, "desktop_export_off.png", (1200, 100, 1650, 400), 1.0, "export 2048 px, coast off (crop)"), tile(exp, "desktop_export_on.png", (1200, 100, 1650, 400), 1.0, "export 2048 px, coast on (crop)")]
sheet(t, "combined_export_crop.png", 2)

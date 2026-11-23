"""Ruling HS frames: one PNG, a row a town, BEFORE (origin/main at 94064bc5) and AFTER (this build), both with
civ.bank_sites on: the map at x4 and x16 round the marker and the City Viewer's opening frame
(`tools/town_bank_drive.gd`'s `on` crops). The last rows are one town at H_SURGE 4 m and at 8 m (the sweep).

  python -I combine.py FRAMES_DIR OUT_PNG
"""
import pathlib, sys
from PIL import Image, ImageDraw

src, out = pathlib.Path(sys.argv[1]), sys.argv[2]
ROWS = [  # (before folder or None, after folder, index, label)
    ("base_w483920", "after_w483920", 25, "483920 #25 hamlet 521:214 (class d): no reason, the market comes to the water (122 -> 96 m)"),
    ("base_w483920", "after_w483920", 30, "483920 #30 capital 1063:198 (class d): the rule's layout lost a crossing, the guard kept the old one"),
    ("base_w483920", "after_w483920", 3, "483920 #3 capital 1520:293 (class c): a cliff edge at the water, set back 77 m, quay at the water"),
    ("base_w483920", "after_w483920", 143, "483920 #143 hamlet 853:559: storm surge on an exposed shore, set back 121 m"),
    ("base_owner", "after_owner", 1, "532187 #1 capital 1478:1159 (owner's town): a sheltered harbour, no surge band, unchanged"),
    ("base_owner", "after_owner", 0, "532187 #0 capital 146:1179: on its floodplain, walked back to the first ground 2 m up (551 m from the river)"),
    ("base_owner", "after_owner", 20, "532187 #20 hamlet 580:256: surge and marsh at the water, set back 117 m (was 365 m back with no reason)"),
    ("after_w483920", "after_h8", 151, "483920 #151 hamlet 824:695, the sweep: H_SURGE 4 m (left, at the water) and 8 m (right, set back 121 m)"),
]
COLS = ["map_x4", "map_x16", "city_x1"]
H = 300
def pics(folder, i):
    ims = []
    for c in COLS:
        f = src / folder / ("desktop_t%d_on_%s.png" % (i, c))
        im = Image.open(f).convert("RGB") if f.exists() else Image.new("RGB", (H, H), (200, 60, 60))
        ims.append(im.resize((max(1, int(im.width * H / im.height)), H)))
    return ims
rows = []
for before, after, i, label in ROWS:
    ims = pics(before, i) + pics(after, i)
    w = sum(im.width for im in ims) + 10 * (len(ims) - 1) + 30
    row = Image.new("RGB", (w, H + 24), (255, 255, 255))
    ImageDraw.Draw(row).text((4, 4), label + "   [BEFORE map x4 | x16 | City Viewer]  ||  [AFTER map x4 | x16 | City Viewer]", fill=(0, 0, 0))
    x = 0
    for k, im in enumerate(ims):
        if k == 3:
            x += 30
        row.paste(im, (x, 24))
        x += im.width + 10
    rows.append(row)
W = max(r.width for r in rows)
sheet = Image.new("RGB", (W, sum(r.height for r in rows) + 8 * len(rows)), (255, 255, 255))
y = 0
for r in rows:
    sheet.paste(r, (0, y))
    y += r.height + 8
sheet.save(out)
print(out, sheet.size)

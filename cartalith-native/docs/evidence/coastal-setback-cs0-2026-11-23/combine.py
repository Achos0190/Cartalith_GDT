"""CS-0 frames: one PNG of the six towns, a row each (civ.bank_sites on, today's app state): the map at x4 and x16
(`tools/town_bank_drive.gd`'s crops round the marker) and the City Viewer's opening frame.

  python -I combine.py FRAMES_DIR OUT_PNG

FRAMES_DIR holds `r2048/` (seed 483920 at 2048 x 1311) and `r384/` (seed 912345 at 384 x 288)."""
import pathlib, sys
from PIL import Image, ImageDraw

src, out = pathlib.Path(sys.argv[1]), sys.argv[2]
TOWNS = [  # (folder, index, label): two of each class, (a) (c) (d)
    ("r2048", 166, "(a) 483920 #166 1456:298 at 2048: hamlet, market 838 m back; marsh/flood at the market"),
    ("r384", 16, "(a) 912345 #16 228:124 at 384: capital, market 398 m back; surge+marsh on the whole front band"),
    ("r2048", 3, "(c) 483920 #3 1520:293 at 2048: capital, market 4 m from the sea; surge+cliff at the market"),
    ("r2048", 34, "(c) 483920 #34 1404:968 at 2048: town, market 20 m; cliff top at the market"),
    ("r2048", 30, "(d) 483920 #30 1063:198 at 2048: capital, market 318 m back, houses 19 m; reason-free shore in reach"),
    ("r2048", 7, "(d) 483920 #7 1323:81 at 2048: hamlet, market 272 m, nearest house 204 m; no reason"),
]
COLS = ["on_map_x4", "on_map_x16", "on_city_x1"]
H = 340
rows = []
for folder, i, label in TOWNS:
    ims = []
    for c in COLS:
        im = Image.open(src / folder / ("desktop_t%d_%s.png" % (i, c))).convert("RGB")
        ims.append(im.resize((max(1, int(im.width * H / im.height)), H)))
    w = sum(im.width for im in ims) + 10 * (len(ims) - 1)
    row = Image.new("RGB", (w, H + 24), (255, 255, 255))
    ImageDraw.Draw(row).text((4, 4), label + "   [map x4 | map x16 | City Viewer]", fill=(0, 0, 0))
    x = 0
    for im in ims:
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

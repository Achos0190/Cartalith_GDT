"""Ruling HU: one PNG of the crops `tools/town_bank_drive.gd` saved with the build before this ruling and with it.

  python -I combine.py OUT.png "BEFORE_DIR,AFTER_DIR,INDEX=TITLE[,INDEX=TITLE ...]" ["BEFORE_DIR,AFTER_DIR,..." ...]

Two rows per town, both with `civ.bank_sites` ON: BEFORE (the build Ruling HQ shipped, `origin/main` at `4317673c`) and AFTER (this ruling's).
Columns: the map at x16 and x64, the City Viewer at its opening frame and at x4. Every crop is fitted into a
300 x 300 cell."""
import pathlib, sys
from PIL import Image, ImageDraw
CELL, PAD, HEAD = 300, 4, 22
VIEWS = ["map_x16", "map_x64", "city_x1", "city_x4"]
rows = []
for spec in sys.argv[2:]:
    before, after, *towns = spec.split(",")
    for t in towns:
        idx, title = t.split("=", 1)
        for label, d in (("BEFORE (Ruling HQ's build)", before), ("AFTER (Ruling HU)", after)):
            rows.append(("%s  %s" % (title, label), [pathlib.Path(d) / ("desktop_t%s_on_%s.png" % (idx, v)) for v in VIEWS]))
out = Image.new("RGB", (len(VIEWS) * (CELL + PAD) + PAD, len(rows) * (CELL + HEAD + PAD) + PAD), (24, 24, 28))
dr = ImageDraw.Draw(out)
missing = 0
for j, (label, files) in enumerate(rows):
    y = PAD + j * (CELL + HEAD + PAD)
    for i, f in enumerate(files):
        x = PAD + i * (CELL + PAD)
        dr.text((x + 2, y + 4), ("%s  " % label if i == 0 else "") + VIEWS[i].replace("_", " "), fill=(235, 235, 235))
        if not f.exists():
            missing += 1; continue
        im = Image.open(f).convert("RGB"); im.thumbnail((CELL, CELL), Image.LANCZOS)
        out.paste(im, (x + (CELL - im.width) // 2, y + HEAD + (CELL - im.height) // 2))
# A 256-colour palette keeps the sheet near a megabyte; the crops are evidence, not artwork.
out.quantize(256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).save(sys.argv[1], optimize=True); print(sys.argv[1], out.size, len(rows), "rows,", missing, "crops missing")

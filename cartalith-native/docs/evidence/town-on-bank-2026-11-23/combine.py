"""Ruling HM: one PNG of the crops `tools/town_bank_drive.gd` saved.
  python -I combine.py OUT.png FRAMES_DIR=FORM[=TITLE] [FRAMES_DIR=FORM[=TITLE] ...]
One row per framed town and switch state (BEFORE = civ.bank_sites off, AFTER = on), one column per view the drive
saved (the map at each zoom, the City Viewer's opening frame and x4), every crop fitted into a 260 x 260 cell."""
import pathlib, re, sys
from PIL import Image, ImageDraw
CELL, PAD, HEAD = 260, 4, 22
rows, cols_all = [], []
for spec in sys.argv[2:]:
    d, form, *title = spec.split("=", 2)
    d = pathlib.Path(d)
    found = {}
    for f in sorted(d.glob(form + "_t*_*.png")):
        m = re.match(form + r"_t(\d+)_(off|on)_(.+)\.png", f.name)
        if m: found[(int(m[1]), m[2], m[3])] = f
    towns = sorted({k[0] for k in found})
    views = sorted({k[2] for k in found}, key=lambda v: (v.startswith("city"), int(re.search(r"x(\d+)", v)[1])))
    for t in towns:
        for st in ("off", "on"):
            rows.append(("%s %s #%d %s" % (title[0] if title else "", form, t, "BEFORE" if st == "off" else "AFTER"), [(v, found.get((t, st, v))) for v in views]))
ncol = max(len(r[1]) for r in rows)
out = Image.new("RGB", (ncol * (CELL + PAD) + PAD, len(rows) * (CELL + HEAD + PAD) + PAD), (24, 24, 28))
dr = ImageDraw.Draw(out)
for j, (label, cells) in enumerate(rows):
    y = PAD + j * (CELL + HEAD + PAD)
    for i, (view, f) in enumerate(cells):
        x = PAD + i * (CELL + PAD)
        dr.text((x + 2, y + 4), "%s  %s" % (label.strip(), view.replace("_", " ")), fill=(235, 235, 235))
        if f is None: continue
        im = Image.open(f).convert("RGB")
        im.thumbnail((CELL, CELL), Image.LANCZOS)
        out.paste(im, (x + (CELL - im.width) // 2, y + HEAD + (CELL - im.height) // 2))
# A 256-colour palette keeps a sheet of thirty-odd rows to a few megabytes; the crops are evidence, not artwork.
out.quantize(256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).save(sys.argv[1], optimize=True); print(sys.argv[1], out.size, len(rows), "rows")

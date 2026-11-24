"""The combined frame, desktop form, seed 51203 at 2 048 x 1 311 (app defaults), each crop centred on the marker, columns
x1 / x4 / x16 / x64, two lane ends per block (Grimkarcairn, and Eshtamishallows where two lanes end):
  1. BEFORE, pins hidden: the attachment removed (`before_nopin` run: `points = _attach_route_ends(...)` replaced by
     `pass` in the project copy, restored after), so the stroke ends at the engine's cell corner;
  2. AFTER, pins hidden (`SL_HIDE_PINS=1`): the stroke ends on the marker point (the town's market badge);
  3. AFTER as the app shows it, pins drawn.

  python -I combine.py BEFORE_NOPIN_DIR AFTER_NOPIN_DIR AFTER_DIR OUT_DIR"""
import pathlib, shutil, sys
from PIL import Image, ImageDraw

before, after_np, after, out = map(pathlib.Path, sys.argv[1:5])
(out / "frames").mkdir(parents=True, exist_ok=True)
ends = ["lane0_end0", "lane0_end1"]
zooms = [1, 4, 16, 64]
S, PAD, HEAD = 300, 6, 22
blocks = (("BEFORE, pins hidden: the stroke ends at the engine's cell corner", before, "before_nopin"),
          ("AFTER, pins hidden: the stroke ends on the marker point", after_np, "after_nopin"),
          ("AFTER, as the app draws it", after, "after"))
im = Image.new("RGB", (len(zooms) * (S + PAD), len(blocks) * (HEAD + len(ends) * (S + PAD))), (255, 255, 255))
dr = ImageDraw.Draw(im)
y = 0
for label, d, tag in blocks:
    dr.text((6, y + 5), label + "   (columns x1 x4 x16 x64; desktop; seed 51203 at 2048)", fill=(0, 0, 0))
    y += HEAD
    for e in ends:
        for i, z in enumerate(zooms):
            p = d / ("desktop_%s_x%d.png" % (e, z))
            if p.exists():
                im.paste(Image.open(p).convert("RGB"), (i * (S + PAD), y))
                shutil.copy(p, out / "frames" / ("%s_%s" % (tag, p.name)))
        y += S + PAD
im.save(out / "combined.png")
print(out / "combined.png", im.size)

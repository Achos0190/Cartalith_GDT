"""Ruling FG sheets: rows of labelled 1000x640 crops (centred on the pivot at screen (1335, 755))."""
import sys
from PIL import Image, ImageDraw
def crop(p, lab, scale=0.5):
    im = Image.open(p).convert("RGB").crop((835, 435, 1835, 1075))
    im = im.resize((int(1000 * scale), int(640 * scale)))
    d = ImageDraw.Draw(im); d.rectangle((0, 0, len(lab) * 6 + 8, 14), fill=(0, 0, 0)); d.text((4, 2), lab, fill=(255, 255, 0))
    return im
def sheet(out, rows, scale=0.5):
    w, h = int(1000 * scale), int(640 * scale)
    cols = max(len(r) for r in rows)
    s = Image.new("RGB", (cols * (w + 6), len(rows) * (h + 6)), (20, 20, 20))
    for i, r in enumerate(rows):
        for j, (p, lab) in enumerate(r):
            s.paste(crop(p, lab, scale), (j * (w + 6), i * (h + 6)))
    s.save(out, quality=88)
if __name__ == "__main__":
    pass

"""The `notice` pin (PIN_PALETTE_SCOPE.md PP-1): an exclamation mark, the
companion of `other` (the question mark), in both sets.

    glyph_notice.png    110 x 128, flat glyph look, blue like `other`
    painted_notice.png  107 x 128, first painted look: a notice board with "!"

Both reuse the sets' own generators (fable_glyphs.py, painted_A.py), so body,
outline, tip and grounding are the sets'. Writes only into this folder.
"""
import os, sys
import numpy as np
from PIL import Image, ImageDraw
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..")); sys.path.insert(0, os.path.join(HERE, "..", "painted_A"))
import fable_glyphs as F
import painted_A as A
REPO = "C:/Users/Vincent/Cartalith_GDT/cartalith-native/godot-project/icons/pins/"

@F.icon('notice', 'blue', 1.12)
def _(g):
    # A bold exclamation mark, sized and weighted like `other`'s question mark
    # (same scale, same dot, same top and foot).
    g.poly([(-5.2, -22), (5.2, -22), (3.3, 11), (-3.3, 11)]); g.circ(0, -22, 5.2); g.circ(0, 11, 3.3)
    g.circ(0, 23, 5.4)

@A.pin('notice')
def _(q):
    # A roofed notice board on two posts with a cream "!" painted on it: the
    # companion of `other`'s tent with its "?". Stands on the grass.
    q.mound('grass', 23)
    q.R('woodd', -17.5, -16, -14, 25, mode='cyl'); q.R('woodd', 14, -16, 17.5, 25, mode='cyl')
    q.R('wood', -19, -17, 19, 13, a=0.12, bu=0.4, bv=0.15); q.planks(-19, -17, 19, 13, step=5.4)
    q.R('woodd', -19, -17, 19, -14.6, t=0.35, ink=0.3); q.R('woodd', -19, 10.6, 19, 13, t=0.55, ink=0.3)
    q.P('shingle', [(-24, -16), (0, -29), (24, -16)], a=0.1, bu=0.55)
    q.lines([((x, -16), (x * 0.25, -26)) for x in (-16, -8, 8, 16)], a=0.3, clip=q.mp([(-24, -16), (0, -29), (24, -16)]))
    q.P('canvas', [(-4.8, -12.5), (4.8, -12.5), (2.9, 1.5), (-2.9, 1.5)], ink=0.5, t=0.04)
    q.E('canvas', 0, 6.2, 3.1, 3.1, ink=0.5, t=0.04)
    for x in (-16, 16): q.put(q.me(x, -11, 0.8, 0.8), (60, 62, 70)); q.put(q.me(x, 4, 0.8, 0.8), (60, 62, 70))
    q.tufts(12)

def main():
    g = F.render('notice', F.bodies()['blue']); g.save(os.path.join(HERE, "glyph_notice.png"))
    body, mask = A.body_and_mask(); p = A.render('notice', body, mask); p.save(os.path.join(HERE, "painted_notice.png"))
    cells = [("glyph: other", Image.open(REPO + "glyph/other.png").convert("RGBA")), ("glyph: notice", g),
             ("painted: other", Image.open(REPO + "painted/other.png").convert("RGBA")), ("painted: notice", p)]
    cw = 150; W = cw * 4 + 16
    sh = Image.new("RGBA", (W, 262), (52, 54, 60, 255)); d = ImageDraw.Draw(sh); f = F.font(11)
    d.rectangle([0, 166, W, 262], fill=(150, 158, 122, 255))
    for i, (nm, im) in enumerate(cells):
        x = 8 + i * cw
        sh.alpha_composite(im, (x + (cw - im.width) // 2, 8)); d.text((x + cw / 2, 142), nm, fill=(235, 235, 235), font=f, anchor="ma")
        for h, xo in ((48, 30), (32, 96)):
            t = im.resize((round(im.width * h / 128), h), Image.LANCZOS)
            sh.alpha_composite(t, (x + xo, 226 - h)); d.text((x + xo + t.width / 2, 232), str(h), fill=(20, 20, 20), font=f, anchor="ma")
    sh.save(os.path.join(HERE, "notice_options.png"))

if __name__ == "__main__":
    main()

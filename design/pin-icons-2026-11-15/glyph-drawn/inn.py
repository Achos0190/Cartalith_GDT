"""The glyph-set `inn` pin (PIN_PALETTE_SCOPE.md PP-7), so the flat set matches
the painted `inn.png` (the half-timbered inn building).

Reuses fable_glyphs.py, so the body is the owner's blue pin body and the
pictogram is clipped 5 px inside the rim. Writes glyph_inn.png (110 x 128) and
inn_options.png into this folder only.
"""
import os, sys
from PIL import Image, ImageDraw
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
import fable_glyphs as F

@F.icon('inn', 'blue')
def _(g):
    # A gabled, half-timbered inn with a chimney and a sign hanging from a post.
    # The timber frame (cut as dark lines) and the sign are what separate it
    # from `guild` (stepped gable, pennant) and `tavern` (the tankard).
    g.rect(-3.5, -25, 1.5, -9)                                # chimney
    g.rect(-28, -3, 4, 26)                                    # wall
    g.poly([(-32.5, -2), (-12, -28), (8.5, -2)])              # gable roof
    # the timber frame: thin dark lines kept inside the wall, so the wall stays one piece
    for seg in ([(-25.5, 10.5), (1.5, 10.5)], [(-12, 0.5), (-12, 10.5)], [(-25.5, 8.5), (-18.5, 0.5)], [(1.5, 8.5), (-5.5, 0.5)]):
        g.line(seg, 1.5, g.k)
    g.rect(-16, 14.5, -8, 26, g.k)                            # door
    for x in (-25, -4.5): g.rect(x, 14.5, x + 5.5, 21.5, g.k)  # windows
    g.rect(22.5, -15, 25.8, 26); g.rect(10.5, -15, 25.8, -12.2)   # sign post and arm
    for x in (13.2, 18.3): g.rect(x - 0.8, -12.2, x + 0.8, -6.5)  # chains
    g.rect(10.8, -6.5, 20.7, 5); g.circ(15.75, -0.7, 2.4, g.k)    # the sign board, hanging clear of the post
    g.rect(-30, 26, 29, 28.5)

def main():
    g = F.render('inn', F.bodies()['blue']); g.save(os.path.join(HERE, "glyph_inn.png"))
    op = lambda p: Image.open(p).convert("RGBA")
    cells = [("glyph tavern", op(os.path.join(HERE, "..", "out", "tavern.png"))), ("glyph inn", g),
             ("glyph guild", op(os.path.join(HERE, "..", "out", "guild.png"))), ("painted inn", op(os.path.join(HERE, "..", "painted_A", "inn.png")))]
    cw = 150; W = cw * 4 + 16
    sh = Image.new("RGBA", (W, 262), (52, 54, 60, 255)); d = ImageDraw.Draw(sh); f = F.font(11)
    d.rectangle([0, 166, W, 262], fill=(150, 158, 122, 255))
    for i, (nm, im) in enumerate(cells):
        x = 8 + i * cw
        sh.alpha_composite(im, (x + (cw - im.width) // 2, 8)); d.text((x + cw / 2, 142), nm, fill=(235, 235, 235), font=f, anchor="ma")
        for h, xo in ((48, 30), (32, 96)):
            t = im.resize((round(im.width * h / 128), h), Image.LANCZOS)
            sh.alpha_composite(t, (x + xo, 226 - h)); d.text((x + xo + t.width / 2, 232), str(h), fill=(20, 20, 20), font=f, anchor="ma")
    sh.save(os.path.join(HERE, "inn_options.png"))

if __name__ == "__main__":
    main()

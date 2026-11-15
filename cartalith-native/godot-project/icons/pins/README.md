# Teardrop pin icon sets (owner-supplied, 2026-11-15)

Two 5 x 4 sheets the owner supplied with *"Can you use these and cut them correctly as options for the teardrops?"*.
The sheets as supplied are in `design/pin-icons-2026-11-15/` (`sheet_glyph.png`, `sheet_painted.png`), with the
cutting script (`slice_pins.py`).

- `glyph/`: flat two-colour pictograms, colour-coded by row (gold settlements, blue built features, green nature,
  orange hazards and land).
- `painted/`: painted miniatures in a bronze frame.

Every file is one teardrop, **128 px tall, transparent outside the pin, with the tip at the bottom-centre pixel**
and the head a circle whose centre sits R px below the top (glyph R = 101/239 of the height, painted R = 92/222),
so a drawer can anchor it by its tip and hit-test it as a circle plus a tangent triangle.

How they were cut: per cell, a teardrop outline (circle R, tip K*R below the centre; K measured by hand on cell
(0,0): glyph 137/102, painted 131/93) was fitted to the pin's dark outer outline, then used as a 4x-supersampled
alpha mask. The painted sheet's bronze rim against its brown ground misled a free fit on four cells, so its centres
are taken from the sheet's regular grid with a +-3 px refinement. All 40 were checked by eye over a magenta ground.

| row | glyph | painted |
|---|---|---|
| 1 | capital, town, village, castle, ruin | capital, town, village, castle, ruin |
| 2 | harbour, bridge, market, mine, temple | harbour, bridge, market, mine, monument |
| 3 | mountain, forest, waterfall, lake, cave | mountain, forest, waterfall, lake, cave |
| 4 | volcano, desert, farmland, rockfall, wildlife | volcano, desert, farmland, hazard, wildlife |

Names are the coordinator's reading of each picture; mapping them onto the engine's kinds (POI kinds, landmark
kinds, settlement tiers) is the integration's job.

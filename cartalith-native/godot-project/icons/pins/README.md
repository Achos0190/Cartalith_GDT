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

## Painted set, extra sheet (owner, 2026-11-15)

The owner supplied a third sheet (`design/pin-icons-2026-11-15/sheet_painted_extra.png`, labelled by the owner,
cut by `slice_pins_extra.py`) answering the gap list: 38 pins added to `painted/` under the owner's own labels:
city, hamlet, tavern, smithy, shrine, guild, gate, well, camp, other, spring, river_confluence, delta, ford, portage,
historic_crossing, mountain_pass, ridge, cliff, gorge, rock_formation, glacier, quarry, salt_works, road_junction,
caravan_station, trade_depot, ancient_road, fort, watchtower, fortified_pass, fortified_crossing, battlefield,
border_marker, sacred_grove, sacred_mountain, ceremonial_site, tomb. Same contract (128 px tall, tip at the
bottom-centre pixel); these have a gold rim on a charcoal ground, so the fit maximises rim-inside minus
ground-outside brightness, with the tip factor K fitted per pin (1.575 to 1.875). All 38 checked by eye.
The glyph set has none of these; a kind with no glyph icon falls back to the drawn pin. The owner ruled that fort
must not borrow the castle icon.

## Glyph set, drawn in code (owner-approved, 2026-11-15)

39 more pins in `glyph/`, drawn to the owner's flat style because the owner's sheet had none for these kinds:
city, hamlet, tavern, smithy, shrine, guild, gate, well, camp, other, spring, river_confluence, delta, ford, portage,
historic_crossing, mountain_pass, ridge, cliff, gorge, rock_formation, glacier, quarry, salt_works, road_junction,
caravan_station, trade_depot, ancient_road, fort, watchtower, fortified_pass, fortified_crossing, battlefield,
border_marker, sacred_grove, sacred_mountain, ceremonial_site, tomb, monument. A first coded draft was refined by a
Fable agent at the owner's request: each pin body (outline, rim shading, disc) is the per-pixel median of the owner's
five pins of that colour, so the silhouette and hit shape match the owner's pins; only the pictogram is drawn.
The owner reviewed the contact sheets and chose: ancient_road option B (road under a ruined arch), portage option C
(boat on a cart), caravan_station option B (wagon under a waystation roof with a lantern), historic_crossing kept
as the bridge with a fallen middle arch, road_junction approved; fort is a timber stockade, never the castle.
Generators and sheets: `design/pin-icons-2026-11-15/glyph-drawn/` (`fable_glyphs.py`, `alts.py`).

# Landmark glyphs — 49 types

Drawn 2026-08-31 to `DCC_SHELL_SPEC.md` §12, for the landmark/POI types
`cartalith_civ::landmark::kinds()` declares.

| File | What it is |
|---|---|
| `Main.dc.html` | the specimen sheet — all 49 at 12 / 16 / 24 px, grouped by family |
| `Contrast.dc.html` | the eleven pairs the set is decided on |
| `canvas.json` | layout |
| `cartalith-landmark-glyphs.html` | the seeded canvas |

**The glyphs themselves ship in `shell/dcc_icons.gd`**, not here — these
artboards are the specimen, not the source. `DccIcons.landmark_glyph(key)`
resolves an engine type key to a glyph name.

## Three are reused rather than drawn

`cliff`, `lake` and `volcanic_feature` are served by the shipped `cliff`,
`lake` and `volcano` sculpt-feature glyphs. Reusing a good glyph beats drawing
a near-duplicate, and this repository prefers deletion to addition.

## Two devices carry meaning across the set

- **A ring above** marks sanctity, and *only* where sanctity is what separates
  a type from the physical one it would otherwise match: `sacred_mountain`
  against `peak`, `sacred_grove` against `ancient_forest`.
- **A broken or dashed run** marks age: `historic_crossing` is
  `river_crossing` with its way dashed; `destroyed_fortress` is `fort`
  breached.

## Verified through the real rasteriser

`_glyphsheet_probe.gd` renders all 49 through **Godot's own SVG rasteriser** —
the one that ships them — and asserts every engine type resolves, every glyph
rasterises non-blank, and the three reuses point at shipped names. A browser
renders the markup differently from Godot, and §12's real question — whether a
1.2 hairline survives at 12 px — is a property of the rasteriser, not of the
path data.

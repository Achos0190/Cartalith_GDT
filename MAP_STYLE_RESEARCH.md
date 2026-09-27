# Map graphic-style research

Owner, 2026-09-27, on `OUTSTANDING_WORK.md`'s "Research map graphic styles"
row: *"can you think research map styles? Not data overlays but actually
graphic styles. Basically to expand the graphic presets we already have."*

This is research, filed at the owner's request — not a scope document, and it
carries no status column. It does not decide anything; it inventories what
the renderer can already do, catalogues real cartographic and illustrative
styles with citations, maps each style onto that inventory, and proposes
concrete preset definitions for the owner to choose from. `STATUS.md` still
answers what, if anything, has been built from it.

## 1. What exists today — the vocabulary

Read at the symbols named, 2026-09-27.

### 1.1 The shell's style gallery

`godot-project/shell/workspaces/render_workspace.gd`'s `STYLE_PRESETS` (six
tiles today). Each preset is an **absolute bundle**: a base *look* name, plus
a dictionary of Painter (NPR) overrides applied over that look, plus
(optionally, "Village" only) an appearance-key override bundle:

| Preset | Look | NPR overrides |
|---|---|---|
| Natural Vibrant | Natural Vibrant | `multi_sun: true` |
| Default | Quality tier | — |
| Antique | Antique Parchment | `sepia: 0.35, multi_sun: true` |
| Ink | Natural Vibrant | `ink: 0.6, contours: 0.35, multi_sun: true` |
| Watercolor | Natural Vibrant | `watercolor: 0.65, multi_sun: true` |
| Print | Natural Vibrant | `risograph: 0.5, contours: 0.25` |
| Village | Quality tier | `village: true`, plus appearance overrides zeroing the three hillshade band weights and setting `relief_ambient: 1.0` |

`STYLE_MANAGED` names every key a preset resets to its own default before
applying itself: `watercolor`, `contours`, `contour_m`, `ink`, `hachure`,
`cel`, `crosshatch`, `stipple`, `sepia`, `risograph`, `pointillism` (all
`0.0`), and `waves`, `multi_sun`, `village` (all `false`). That is the whole
"Painter" (NPR) vocabulary a preset can address.

### 1.2 The engine's looks

`crates/cartalith-godot/src/render.rs`'s `LOOK_PRESETS` — three names:
`"Quality tier"` (`LOOK_TIER`), `"Natural Vibrant"` (`LOOK_VIBRANT`, the
default), `"Antique Parchment"` (`LOOK_ANTIQUE`). `TerrainAppearance::with_look`
is a struct-update over whatever the quality tier produced, and only ever
touches colour, chroma, light-shaping and grade fields — never a radius, a
light count, or anything a cheap tier switched off. `LOOK_VIBRANT` re-pitches
the four base material ramps toward saturated hue families and raises
`relief_chroma`, `ao_strength`, `crest_strength`, `tex_strength`,
`ridged_strength`, `curve_shade`, `hydro_wet_strength`, `biome_sat`,
`haze_strength`. `LOOK_ANTIQUE` pushes the four ramps toward ochre/umber, sets
`paper_strength: 1.0` with a warm `paper_tint`, raises `paper_grain`,
`paper_mottle`, `paper_wash`, sets a dark `border_ink`, and moves the colour
grade (`grade_temperature`, `grade_saturation`, `grade_contrast`,
`grade_shadow_tint`) toward warm/low-chroma/lifted-shadow.

### 1.3 `TerrainAppearance`'s full tunable surface

Everything a preset (or a future one) can set, grouped as the panel groups
them (`APPEARANCE_GROUPS`, `render_workspace.gd`):

- **Relief & light**: `relief_lights`, `relief_directionality`,
  `relief_ambient`, `relief_gain`, `relief_chroma`, `ao_strength`,
  `ao_radius_frac`, `svf_strength`, `shadow_strength`, `crest_strength`,
  `curve_shade`, `ridged_strength`. Plus, in "Map view": `exag` (vertical
  exaggeration), `sun_az_deg`, `sun_alt_deg`.
- **The sheet** (paper/texture overlay): `paper_strength`, `paper_grain`,
  `paper_mottle`, `paper_wash`, `stipple_strength`, `border_width_frac`. Plus
  `paper_tint`, `border_ink` (colours, set by a look rather than a slider
  today).
- **Materials**: `biome_sat`, `tex_strength`, `rock_slope`, `litho_strength`,
  `litho_exposure`, `geo_micro`, `hydro_wet_strength`, `wetness`, `sdf_coast`,
  `sdf_rivers`, `sdf_biomes`, `local_contrast`, `splat_strength`,
  `sea_grain_warp`.
- **Ice & snow**: `ice_strength`.
- **Atmosphere**: `haze_strength`, `atmo_desaturation`, `atmo_contrast`.
- **Multi-scale detail**: `detail_macro_weight`, `detail_meso_weight`,
  `detail_micro_weight` (the macro/meso/micro hillshade band weights —
  zeroing all three, as "Village" does, collapses the light to the ambient
  floor).
- **Colour grade**: `grade_exposure`, `grade_gamma`, `grade_contrast`,
  `grade_saturation`, `grade_temperature`, `grade_shadow_tint`,
  `grade_highlight_tint` (a presentation-only post-process, before rivers,
  labels and icons draw).
- **Colours** (Ruling L): `bio_blend` (biome-colour blend), the elevation
  ramp (`ElevationRamp`, `ramp_strength`, `RampMode` — `Linear`/`Ease`/`Step`),
  and per-biome-class colour overrides (`_biome_swatches`).
- **The base material ramps themselves**: `grass_temp`, `wood_trop`,
  `sand_desert`, `sand_red` — each a 3-stop `(low, mid, high)` RGB triple a
  look can re-pitch.

### 1.4 `RAMP_PRESETS` — nine named elevation-tint ramps

`Earth` (default, closest to the material model), `Elevation`, `Atlas`,
`Mono`, `Imhof`, `Ice`, `Dark ice`, `Desert`, `Dark atlas`. Each is a list of
`(relative-elevation, RGB)` stops with an opacity per stop
(`RampStop::a`) and a curve (`Linear`/`Ease`/`Step`). `ramp_strength` blends
this land-only, pre-lighting tint over the material colour; `0.0` (shipped
default) means the material model alone. `Imhof` is already this port's own
reading of the warm-lowland/cool-highland Swiss convention (§2.1) — it exists
today, but at `ramp_strength: 0.0` in every shipped preset.

### 1.5 The Painter (NPR) block — `Npr` struct fields

`contours` (constant-width isolines, every 5th indexed), `contour_m`
(interval in metres, `0` = reference's `0.05`-of-relief auto interval),
`ink` (pen outlines on curvature×slope edges), `hachure` (downslope
hatching, denser on steep ground — **already named as a field, cost
unmeasured**, see §3), `watercolor` (pigment pooling/granulation/edge
blooms), `cel` (posterised flat toon bands), `crosshatch` (antique-engraving
multi-direction hatch, denser the darker the cell), `stipple` (luminance-
driven pen dot-density over all land), `sepia` (warm toning matrix),
`risograph` (indigo→amber duotone with halftone screen), `pointillism`
(Seurat-style coloured dot field), `waves`/`wave_dist` (foam contours hugging
shore, water only), `multi_sun` (four-light painterly rig replacing the macro
hillshade only), `animate_water` (a presentation flag, read by a separate
Godot overlay, not by this renderer), `village` (flat limited-palette
quantiser over the already-lit colour, land and water both — the one style
flag that replaces rather than mixes in).

### 1.6 What is *not* wired to a preset yet

- **Parchment** (antique 0.6, watercolor 0.2 in the reference) has no preset
  counterpart: this port's paper ground already runs at 0.85 by default.
- **Icons** (mountain/hill/tree glyphs, `pack.rs::composite_map_icons`/
  `draw_icon_glyph`) are built but drawn only with a pack loaded, with no
  preset on/off toggle.
- `hachure`, `cel`, `crosshatch`, `stipple`, `pointillism` are real `Npr`
  fields with **no preset currently sets them** — five knobs already in the
  vocabulary and unused by any of the six tiles.

## 2. Real cartographic and illustrative styles, researched

Each entry: visual rules, a sample palette (hex, where the source gives real
colours — flagged where a figure is this document's own reading rather than
a measured or cited value), and its citation.

### 2.1 Swiss style / Eduard Imhof relief shading

Combines airbrushed shaded relief with contours and colour subtly modulated
by elevation and slope aspect; ridge/gully lines drawn first, shading built
from dark slopes toward bright illuminated peaks; an "aerial perspective"
desaturation/cooling with distance is a deliberate design component, not an
accident. Palette: warm low ground (ochre/olive) cooling to blue-grey at
altitude — this port's own `Imhof` ramp (`RAMP_PRESETS`) is already a reading
of this: `#b0ba8c → #c6c492 → #cec8b8 → #baa294 → #aaa8b2 → #eef2f8`. Type:
restrained, small serif labels; paper: none — a clean printed sheet.
Source: Imhof, *Cartographic Relief Presentation* (summarised at
[ikgrelief.ethz.ch](https://ikgrelief.ethz.ch/cartographers/imhof/),
[shadedrelief.com's Swiss method](http://www.shadedrelief.com/shading/Swiss.html),
[Eduard Imhof — Wikipedia](https://en.wikipedia.org/wiki/Eduard_Imhof)).

### 2.2 Tanaka illuminated contours

Grey background; contour lines split into two colours (white where the slope
faces the light, black where it faces away), line thickness varying with the
cosine of the angle between slope aspect and illumination azimuth — a genuine
technique, not a filter, devised by Kitiro Tanaka (1950). Water and label
treatment are conventional. Source:
[Tandfonline, "Modifications of Tanaka's Illuminated Contour Method"](https://www.tandfonline.com/doi/abs/10.1559/152304001782173709),
[the `tanaka` R package documentation](https://cran.r-project.org/web/packages/tanaka/readme/README.html),
[Terrain cartography — Wikipedia](https://en.wikipedia.org/wiki/Terrain_cartography).

### 2.3 Hachures (Lehmann system)

Short strokes drawn in the direction of steepest descent, in rows
perpendicular to that direction; stroke thickness and length proportional to
slope steepness (short+thick = steep, long+thin = gentle); standardised by
J. G. Lehmann, 1799, for military topographic sheets. Two-colour hachures on
a neutral ground (black+white on grey) can read as illuminated relief.
Water: none prescribed; typically plain outline. Source:
[Hachure map — Wikipedia](https://en.wikipedia.org/wiki/Hachure_map),
[ICA proceedings on Lehmann's system](https://icaci.org/files/documents/ICC_proceedings/ICC2013/_extendedAbstract/265_proceeding.pdf).

### 2.4 Erwin Raisz physiographic diagrams

Bird's-eye pictorial landform texture — small pen-and-ink "caricature"
strokes standing in for dune fields, karst, glaciated peaks, etc., drawn with
the "Armadillo" orthoapsidal projection; black ink strokes on white paper,
no colour. Meant to communicate landform *type* (geomorphology) rather than
precise elevation. Source:
[Raisz's physiographic method of landform mapping (SAGE/ResearchGate)](https://www.researchgate.net/publication/320284649_Raisz's_physiographic_method_of_landform_mapping),
[Erwin Raisz — Wikipedia](https://en.wikipedia.org/wiki/Erwin_Raisz).

### 2.5 Heinrich Berann panorama

Painterly oblique panorama: deliberate terrain deformation (rotating
mountains, widening valleys), vertical exaggeration (Berann doubled Denali's
height, with extra exaggeration at the painting's own focal peak), a
cylindrical-projection quasi-orthographic view giving a frontal, majestic
background, tree brush-strokes, and trademark painterly cloud formations.
Colour is naturalistic but idealised/saturated — "the world as a beautiful
place." Source:
[Heinrich C. Berann — Wikipedia](https://en.wikipedia.org/wiki/Heinrich_C._Berann),
[Seeing the "perfect world" through Berann's Panorama Map of the Alps](https://www.tandfonline.com/doi/full/10.1080/23729333.2021.1912887).

### 2.6 Portolan chart

Coastal outline only (interiors mostly blank), a dense web of rhumb lines
radiating from compass roses, colour-coded by wind class: black for the
eight main winds, green for half-winds, red for quarter-winds; place names
run perpendicular to the coast, red ink for major ports, black for secondary
harbours. No relief shading at all — a coastline-and-network aesthetic.
Source:
[Portolan chart — Wikipedia](https://en.wikipedia.org/wiki/Portolan_chart),
[Map of the Month: portolan charts (U Toronto)](https://mdl.library.utoronto.ca/mdl-blog/map-month-curious-world-portolan-charts).

### 2.7 Medieval mappa mundi (Hereford-style)

Not a survey map: T-and-O world layout, east at the top, dense figurative
illustration (cities as tiny building drawings, biblical/classical scenes,
animals, monsters) filling the continents, drawn on vellum with iron-gall
ink and pigment washes — a visual encyclopaedia over a schematic frame
rather than a metrically accurate coastline. Source:
[Hereford Mappa Mundi — Wikipedia](https://en.wikipedia.org/wiki/Hereford_Mappa_Mundi),
[UNESCO Memory of the World entry](https://www.unesco.org/en/memory-world/hereford-mappa-mundi).

### 2.8 Shan shui (Chinese ink landscape)

Monochrome ink-wash (`shuimo`): varied brush loading and water content
render form and atmosphere rather than outline; mountains (yang) and water
(yin) are the two compositional poles; empty paper functions as mist/space,
not "unfinished" area; the aim is the painter's conception of the landscape,
not an observed view. Palette: black ink values on cream/tan paper — no
hue. Source:
[Shan shui — Wikipedia](https://en.wikipedia.org/wiki/Shan_shui),
[New World Encyclopedia, "Shan shui"](https://www.newworldencyclopedia.org/entry/Shan_shui).

### 2.9 Japanese woodblock (ukiyo-e / meisho zue)

Flat colour fields from separate printing blocks (no continuous gradient),
strong black keyline, stylised wave and cloud motifs, high-contrast palette
(indigo `#1e3a5f`-class blues, vermillion, ochre — this document's own
reading; no hex values were found in the sources), figures/landmarks drawn
schematically rather than photographically; "meisho zue" (famous-places)
series specifically catalogue named landmarks in a travel-guide format.
Source:
[Utagawa Hiroshige — Artelino](https://www.artelino.com/articles/hiroshige.asp),
[Edo Meisho Zue by Hiroshige II — Artelino](https://www.artelino.com/articles/hiroshige-II-edo-meisho-zue.asp).

### 2.10 Woodcut / lithograph (generic antique print)

Covered by risograph (§2.18) and the antique/parchment look already shipped;
distinguishing marks are heavy black keylines, cross-hatched shading in lieu
of continuous tone, and a restricted ink count. No separate search was run —
this is folded into the Print/Antique family below rather than treated as an
eleventh distinct style, since no source distinguished it usefully from
§2.3/§2.18 for this port's purposes.

### 2.11 National Geographic physical-map style

Natural earth shading keyed to vegetation/landcover, hand-drawn (or
hand-drawn-emulating) shaded relief for land, detailed bathymetric relief
shading for the ocean floor (not flat blue), bright saturated but
naturalistic hue, proprietary typefaces. Source:
[NatGeo Maps, World Classic](https://www.natgeomaps.com/re-world-classic),
[Esri, "Meet the National Geographic Style Basemap"](https://www.esri.com/arcgis-blog/products/arcgis-living-atlas/mapping/meet-the-national-geographic-style-basemap),
[Tom Patterson, "Creating a National Geographic-style Physical Map"](http://shadedrelief.com/lenk/NG-Style_World_Map.pdf).

### 2.12 USGS topographic

Fixed colour-by-theme convention: black (culture/boundaries/names), blue
(water), red (highways, built-up areas, PLSS lines), green (woodland/scrub/
orchard/vineyard polygons), brown (contours and other topographic features,
heavier index contours labelled). Source:
[USGS, "Topographic Map Symbols"](https://pubs.usgs.gov/gip/TopographicMapSymbols/topomapsymbols.pdf),
[USGS educational resources page](https://www.usgs.gov/educational-resources/topographic-map-symbols).

### 2.13 Ordnance Survey (Explorer / Landranger)

House-style distinguished chiefly by cover colour and scale (Explorer =
orange, 1:25 000; Landranger = pink/magenta, 1:50 000), with road
classification colour-coded (motorway blue, primary green, A-road red,
B-road orange) and a dense, standardised symbol legend for footpaths,
access land, and antiquities. No relief-shading claim found in the sources
searched — treat this as a symbol/line-weight convention, not a shading
style. Source:
[OS, "Using OS Leisure maps in your app"](https://www.ordnancesurvey.co.uk/blog/using-os-leisure-maps-in-your-app),
[OS, "Cartographic house styles"](https://www.ordnancesurvey.co.uk/blog/os-cartographic-house-styles).

### 2.14 Tolkien / fantasy pen-and-ink

Christopher Tolkien's 1953 Middle-earth map set the modern epic-fantasy
convention: pen-and-ink profile-view mountain ranges (rows of little
triangular peaks), dense fields of individually drawn egg-shaped trees for
forests (bordered by a visible-trunk tree row), hand-lettered names, black
ink (originally black+red) on plain paper — descended directly from
Lehmann hachures and Raisz's physiographic pictograms but simplified to a
small, legible glyph vocabulary. Source:
[Tolkien's maps — Wikipedia](https://en.wikipedia.org/wiki/Tolkien%27s_maps),
["Here Dragons Abound": Lord of the Rings map style](https://heredragonsabound.blogspot.com/2018/10/lord-of-rings-map-style.html),
[Adventures in Mapping, "Middle Earth Map Style"](https://adventuresinmapping.com/2018/09/10/middle-earth-map-style/).

### 2.15 Blueprint / cyanotype

White (or light-cyan) linework on a deep Prussian-blue ground (`#0a2f5c`-class
blue is a common modern approximation; the historical process itself yields
a saturated Prussian blue, no single hex is canonical), the inverse of an
ink-on-paper print; fine technical grid lines, crisp geometric linework, no
soft shading at all — a deliberately flat, high-contrast, "engineering
drawing" look. Source:
["Blueprint Map Art" — Cartosketch](https://cartosketch.com/gallery/blueprint-map-art),
[Herreshoff Marine Museum, "Blueprints and Cyanotypes"](https://herreshoff.org/2020/07/hmco-blueprints-and-cyanotypes/).

### 2.16 Vintage school atlas (mid-century)

Warm, muted earth tones with occasional bright mid-century colour-block
accents; playful, simplified shoreline detail rather than photographic
relief. A cited representative palette: Philippine Brown `#582119`,
Charming Black `#1a1813`, Forest Brown `#906c54`, Muted Bronze `#d0a772`,
Bleach White `#fef1d7`, Bison Hide `#c4b9a3`. Source:
[SchemeColor, "Vintage Map Color Scheme"](https://www.schemecolor.com/vintage-map.php).

### 2.17 Nautical chart

Depth-keyed colour bands rather than land-keyed ones: white (deep water),
light blue (shallow), dark blue (very shallow), green (intertidal), tan/brown
(land or exposed-at-low-tide bottom); isobaths (depth contour lines) with
tightly spaced lines marking a steep underwater slope; numeric depth
soundings scattered across the water. Source:
[NOAA, "What do the numbers mean on a nautical chart?"](https://oceanservice.noaa.gov/facts/sounding.html),
[Amnautical, "Nautical Chart Symbols Explained"](https://www.amnautical.com/blogs/the-mariners-blog/nautical-chart-symbols-explained).

### 2.18 Risograph / limited-palette print

Two- or three-spot-colour printing (fluorescent pink, federal blue,
sunflower yellow are named house inks) with visible grain, imprecise layer
registration, and halftone-dot texture standing in for continuous tone;
often read as a duotone. This port's existing `risograph` `Npr` field
already targets an "indigo→amber duotone with a halftone screen," which
matches this description closely. Source:
[Design Shack, "Risograph Style Backgrounds"](https://designshack.net/articles/inspiration/risograph-style-backgrounds/),
[Design Lexicon, "Risograph Aesthetic"](https://freedesignmd.com/lexicon/risograph).

### 2.19 Night / dark-mode

Deliberately low-contrast base (near-black or deep desaturated navy ground),
with everything dimmed except the labels/data meant to stand out; thinned
line weights, lowered label density, reduced overall contrast versus a day
basemap — the goal is minimising eye strain and distraction, not depicting
night lighting realistically. Source:
[MapTiler, "Dark basemaps for navigation and data visualisation"](https://www.maptiler.com/maps/dark/),
[Mapbox, "Dark Map Style"](https://www.mapbox.com/maps/dark).

## 3. Mapping each style onto the vocabulary

Class (a): buildable today as a preset (existing knobs only). Class (b):
needs small renderer additions (a new field or two, same shape as existing
ones). Class (c): needs a new rendering technique (a genuinely new draw
pass).

| Style | Class | What it needs |
|---|---|---|
| Swiss / Imhof relief | **(a)** | `Imhof` ramp at real strength + Relief & light tuned toward gentle ambient/gain, low `haze`/`atmo` for the aerial-perspective read. Everything is an existing knob. |
| Tanaka illuminated contours | **(c)** | No field draws a two-tone, aspect-thickness-varying contour line today. `contours`/`contour_m` draw constant-width, single-colour isolines. Needs a new contour-rasterisation mode reading local aspect vs. sun azimuth per contour segment — a real new technique, not a slider. |
| Hachures (Lehmann) | **(b)/(c)** | `hachure` is already a named `Npr` field (§1.5) but **its cost and current behaviour were not measured in this pass** — `render.rs` was read for the field's doc comment only, not its draw code. If it already draws slope-aligned strokes it is (b) (tune it into a preset); if it is a stub or draws something else, promoting it to true Lehmann hachures (length/thickness keyed to slope run, rows perpendicular to descent direction) is (c). **Verify at `apply_npr`'s `hachure` branch before scheduling.** |
| Raisz physiographic | **(c)** | Landform-type pictogram stamping (dune fields, karst, glaciated cirques as small oriented glyphs) is a symbol-placement technique this renderer has no equivalent of — closer to the landmark/icon glyph system (`pack.rs::draw_icon_glyph`) than to a Painter filter. Would reuse that stamping machinery, not the NPR block. |
| Berann panorama | **(c)** | Oblique/cylindrical projection and per-feature terrain deformation are a camera/geometry change, not a shader pass; out of scope for a 2D top-down renderer without a larger projection rework. Its *palette and painterly cloud/tree brush character* alone are (b) — richer `grass_temp`/`wood_trop` saturation plus a soft `watercolor`-style edge bloom already available. |
| Portolan chart | **(b)** | Needs a rhumb-line/compass-rose overlay (new geometry, but simple — straight lines from fixed points, not a per-pixel pass) and suppressing interior terrain detail (a coast-only render mode). Moderate: new draw pass, but a cheap one. |
| Medieval mappa mundi | **(c)** | Figurative city/creature icon stamping over a schematic (non-metric) projection is well outside a colour/shading preset — effectively a different content-generation problem (icons narrating theology/legend, not terrain), not a rendering-technique problem this document can size. |
| Shan shui ink | **(b)** | Monochrome ink-wash is reachable by `sepia`-style toning generalised to a true single-hue value ramp (a new `Npr` field or a `RAMP_PRESETS` "Ink wash" mono ramp at `ramp_strength: 1.0`) plus heavy `ink`/`crest_strength` for brush-like ridgelines. Close to buildable with the `Mono` ramp already in `RAMP_PRESETS`. |
| Japanese woodblock | **(b)/(c)** | Flat colour fields from `village`'s quantiser (a) plus a strong black keyline from `ink` (a) gets partway there cheaply; the stylised wave/cloud motif linework is a new symbol pass (c). |
| Woodcut / lithograph | **(a)** | `crosshatch` + `ink` + a mono or duotone ramp — all existing fields. |
| National Geographic | **(a)** | `LOOK_VIBRANT`-family saturation with the ocean given depth-keyed shading (`sea_grain_warp`, already built) and a bathymetric-leaning ramp; no new technique, a tuning exercise. |
| USGS topographic | **(b)** | The fixed-hue-by-theme convention (brown contours, blue water, green vegetation polygons, black culture) needs each land-cover/feature class re-mapped to its own flat swatch rather than a continuous material blend — closer to `village`'s quantiser generalised with fixed per-class hues than to a new pass. |
| Ordnance Survey | **(c)** | Its distinguishing content (footpath/access-land/antiquity symbols, road classification colours) is a vector symbol layer this engine doesn't have a civilian-infrastructure-symbol equivalent of yet; out of reach without new data, not just new paint. |
| Tolkien fantasy | **(b)** | `ink` (mountain outlines) + the existing icon-glyph system generalised to draw the egg-shaped forest-block glyph at density — the tree glyph itself is new art/geometry (b), not a new technique class. |
| Blueprint | **(a)** | A hard-swap "ramp" — flat deep-blue land+water, `ink` at high strength for white/cyan linework, `contours` on, everything else (biome colour, ramp, paper) suppressed. All existing controls, just extreme values. |
| Vintage atlas | **(a)** | `Antique`-family palette re-pitched to the cited hex family, moderate `paper_wash`, low `paper_grain`/`mottle` (atlas paper is smoother than parchment), `grade_temperature` warm. |
| Nautical chart | **(b)** | Needs a depth-banded sea ramp (a `RAMP_PRESETS`-shaped table but keyed to bathymetry rather than land elevation — the mechanism exists, applied to the wrong side of the shoreline) plus a soundings/isobath overlay (new, but geometrically simple: point/line labels over existing depth data). |
| Risograph | **(a)** | `risograph` field exists and is already shipped (Print preset); a second preset at a different duotone hue pair and a coarser halftone scale is a value change, not new code — **contingent on `risograph`'s halftone scale being exposed as a tunable; not confirmed in this pass.** |
| Night / dark-mode | **(b)** | Needs an inverted/darkened base ramp (new `RAMP_PRESETS` entry, e.g. "Night") plus `grade_exposure`/`grade_gamma` pushed low and `haze`/`atmo_desaturation` tuned — mechanically a preset, but a genuinely dark ramp doesn't exist among the nine yet. |

## 4. Top preset definitions, ranked by value for effort

Ranked by (visual distinctiveness × owner-recognisable reference) ÷ (build
cost). All eight below are class (a) or (b) with a small, named addition —
none needs a new rendering technique. Every knob named is one already
inventoried in §1.

1. **"Atlas" (National Geographic-leaning)** — *class (a).* Look:
   `Natural Vibrant`. Ramp: `Atlas` at `ramp_strength: 0.35`. NPR: none.
   Appearance: `haze_strength` and `atmo_desaturation` slightly up for the
   plate-edge aerial-perspective read; `sea_grain_warp` at its shipped
   default (already on since Ruling AS). *Intended look:* a bright, legible
   reference-atlas physical map — closest to what a general audience calls
   "a proper map."

2. **"Imhof relief"** — *class (a).* Look: `Quality tier`. Ramp: `Imhof` at
   `ramp_strength: 0.5`. Appearance: lower `relief_ambient`, higher
   `relief_gain` for crisper illuminated-slope contrast; `haze_strength` up
   for depth. NPR: none. *Intended look:* the warm-lowland/cool-summit Swiss
   school-atlas relief this port's own ramp already encodes, finally turned
   on.

3. **"Blueprint"** — *class (a).* Look: `Quality tier`. Ramp: `Mono` at
   `ramp_strength: 1.0`, re-tinted toward Prussian blue rather than grey (a
   `RampStop` colour edit, not a new mechanism). NPR: `ink: 0.7`,
   `contours: 0.4`. Appearance: `paper_strength: 0`, `biome_sat: -1.0` (or
   as low as the range allows) to flatten material colour under the ramp.
   *Intended look:* white/cyan technical linework on a deep blue field.

4. **"Ink wash" (shan shui-leaning)** — *class (a)/(b).* Look:
   `Quality tier`. Ramp: `Mono` at `ramp_strength: 0.9`, tinted warm
   grey/sepia rather than blueprint's cool blue. NPR: `ink: 0.5`,
   `crest_strength` (appearance) raised for brush-like ridge emphasis,
   `crosshatch: 0.0` (kept off — shan shui is wash, not hatch). *Intended
   look:* a monochrome painterly mountain-and-water study; closest existing
   style to the reference material's own washy watercolor but desaturated
   to ink values.

5. **"Woodcut"** — *class (a).* Look: `Antique Parchment`. NPR:
   `crosshatch: 0.5`, `ink: 0.4`, `sepia: 0.0` (keep the antique palette but
   drop the extra warm cast so the engraving reads black-on-cream rather
   than sepia-toned). Appearance: `paper_grain` up, `paper_mottle` down (a
   woodcut sheet is fibrous, not blotchy). *Intended look:* heavy black
   keyline over cross-hatched shading, printed on a plain cream sheet.

6. **"Vintage atlas"** — *class (a).* Look: `Antique Parchment` with the four
   base ramps re-pitched toward the cited mid-century palette (`#582119`
   brown family, `#d0a772` bronze, `#fef1d7` cream highlights — see §2.16).
   NPR: `sepia: 0.15` (light, not the full antique cast). Appearance:
   `paper_wash` moderate, `paper_grain`/`mottle` low (smoother than
   parchment), `grade_temperature` warm. *Intended look:* a 1950s-60s school
   wall-map, warm and flat rather than aged/foxed.

7. **"Nautical"** — *class (b), needs one new ramp table.* Requires a new
   depth-banded sea ramp (§3's Nautical row) — the one genuinely new data
   table among these eight, reusing the existing `RampStop`/curve
   machinery. Land: `Atlas` ramp at low strength, `sand_desert`-family
   desaturated toward tan. NPR: `contours: 0.2` reused as isobaths once the
   sea ramp exists. *Intended look:* the blue-band-by-depth sea plus tan
   land of a real chart — deferred to (b) rather than (a) only because the
   depth-side ramp table doesn't exist yet.

8. **"Night"** — *class (b), needs one new ramp table.* Requires a new dark
   `RAMP_PRESETS` entry (the existing `Dark ice`/`Dark atlas` are close in
   spirit but keyed to ice/desert biomes, not a general dark base — check
   whether one of them already reads acceptably as "night" before building
   a tenth table). Appearance: `grade_exposure`/`grade_gamma` pushed low,
   `haze_strength` reduced (haze reads muddy in a dark scene),
   `atmo_desaturation` up. *Intended look:* a legible dark-mode map for a
   low-light viewing context, matching the shell's own future dark theme
   rather than depicting a night sky.

Every number above is this document's own proposal, not a measured or
owner-approved constant — none of these eight has been built, and none
should be scheduled without opening `render.rs`'s current default values for
every field named, since several (`biome_sat`, `paper_grain` range limits)
were read only from their doc comments in this pass, not from their declared
bounds.

## 5. Owner questions, each with a default

1. **How many presets to ship first?** *Default: the top three (Atlas,
   Imhof relief, Blueprint) — one bright reference-map style, one that turns
   on this port's own dormant Imhof ramp, one maximally distinct from the
   other five shipped tiles.*
2. **Should `hachure`'s current behaviour be verified before any hachure-
   leaning preset ships, given §3 flagged it as unmeasured in this pass?**
   *Default: yes — a five-minute check of `apply_npr`'s `hachure` branch
   before scheduling any preset that turns it on, per `MISTAKES.md`'s
   re-open-at-the-symbol rule.*
3. **Is a genuinely new bathymetric (depth-keyed) sea ramp worth building
   for one "Nautical" preset, or should it wait until export/water-body work
   touches the sea shading anyway?** *Default: wait — bundle it with the
   next sea-colour touch rather than as a standalone small ticket.*
4. **Do Raisz-style pictogram stamping, Tanaka contours, or the mappa-mundi
   figurative layer belong on this project's roadmap at all, given each is
   class (c) and none was requested by name?** *Default: no — file them here
   as researched-but-not-scoped, the way `RM-`/`EF-` items in the reconstruction
   and elevation-field research documents sit unscoped until an owner
   ruling schedules them.*
5. **Should "Village" (the existing flat-quantiser preset) gain a woodblock-
   leaning sibling that reuses its quantiser at a different band count and
   hue set, rather than building woodblock's flat-fields effect from
   scratch?** *Default: yes — `quantize_flat_palette`'s band count is
   already a chosen constant per `render.rs`'s own doc comment; a second
   named quantisation is cheaper than a new mechanism.*

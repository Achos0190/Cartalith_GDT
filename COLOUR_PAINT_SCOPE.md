# COLOUR_PAINT_SCOPE.md — the non-destructive colour-paint tool (Ruling CN)

**What this is.** The scope for owner Ruling CN (`LARGE_ITEM_RULINGS.md`,
2026-10-18): a **Paint colour** tool in `CARTO ▸ Colours` — a brush and a
colour picker whose strokes are stored in **map kilometres**, composited the
same way in the screen, the deep-zoom tiles and the export, saved with the
project as an additive optional slot, with its own undo, hideable and
clearable, and never touching world data or generation. It defines milestones
CP-1 … CP-7 with acceptance bars; it carries **no status column** — progress is
`cartalith-native/docs/STATUS.md`'s (`CLAUDE.md`, the 2026-08-31 decision).

**What it is not.** WORLD paint (Biome, Terrain, Splat and Rock paint) edits
world data and stays in WORLD (`DCC_SHELL_SPEC.md` §4.5.2). This layer is
presentation: it recolours pixels and nothing reads it but the renderer.
Shortcut rows from CARTO into the WORLD paint tools remain a separate,
optional nicety (the ruling's last sentence).

**The primary acceptance case** (owner, via the coordinator, 2026-10-18):
*"if I want to paint the inside of a volcano red I want to be able to do it"*.
Every design decision below is checked against it, and CP-1's frames measure
it (§11).

Ruling BK applies to all code this scope produces: every function, type and
constant carries a doc comment saying what it does, why, and what it must
never do; every non-obvious constant names its source or is labelled a
judgement; every test says what it protects.

---

## 1. Inventory: what already existed (read at the symbols, 2026-10-18)

| Piece | Where | Used for |
|---|---|---|
| ET-D1 stroke engine (smoothing, spacing, dynamics, scatter, symmetry; deterministic) | `cartalith-spatial/src/stroke.rs` (`StrokeEngine`, `StrokeConfig`) | CP-2 turns pointer samples into the kilometre polyline a mark stores |
| WORLD paint's categorical brush (hard disc + dithered falloff band, `EdgeShape`) | `cartalith-spatial/src/paint.rs` (`PaintStamp`) | Not reused for storage: it writes a `u8` grid, which is the resolution-bound model this ruling rules out. Its `Hardness`/`Softness` vocabulary is reused in the shell |
| The legacy paint tint (Biome/Terrain at 0.60 over the lit colour) | `render.rs::land_color`, the `paint.is_empty()` block | Precedent for "a tint inside `land_color`"; this layer goes in **before the light**, that one after (§4) |
| The terrain layer stack and its blend modes | `render.rs` (`LayerStack`, `BlendMode`) | Same alpha-fold convention (`d + (B(d,s) − d)·a`) |
| Colour picker with the print-gamut row | `shell/dcc_widgets.gd::confirmable_color` (ET-E3; the bridge's `color_print_gamut`) | CP-2's picker, reused as is |
| Look history / undo | `shell/look_history.gd` (`begin_gesture`, `commit`, `undo`, `redo`) | CP-2's undo model (§8) |
| Additive optional project slot precedent | `cartalith-io/src/project.rs` `DOCUMENT_SLOTS` (`appearance_style.json`, `appearance_overlay_adjust.json`); `SAVEFILE_COMPAT.md` | CP-3's `appearance_paint.json` (§9) |
| Banded export, look census, deep-zoom tile path | `render.rs` (`bake_export_band`, `render_biome_tile_rgba_water`), `export_raster.rs` (`adjust_census_tests`) | The three consumers; the census literals hold the absent case |
| The kilometre zoom-detail law | `render.rs::zoom_detail_km_law` | Not read: marks are kilometres already, so a mark gains nothing from the law and loses nothing to it |
| Level-set contours | `cartalith-spatial/src/contour.rs::contour_polylines` | CP-5's fill helper traces its region with it |

## 2. The stroke model — vector, in kilometres, with a spatial index

**Decision: marks are vector geometry in map kilometres, evaluated
analytically per output pixel.** Two shapes (`cartalith_spatial::colour_paint::MarkShape`):

- **Line** — a polyline of `[x_km, y_km]` points plus `radius_km`: the union of
  discs swept along it (round caps and joins). A stroke is one shape, so a
  stroke crossing itself paints once — opacity is applied once, as in a paint
  program.
- **Area** — a closed ring plus `edge_km`: the even-odd interior, its edge
  softened over `edge_km` centred on the ring. The fill helper's output (CP-5),
  and what makes "fill the crater" one mark instead of a scribble.

Each mark carries `rgb` (sRGB 0-255, the picked colour), `opacity`, `hardness`
(the solid fraction of a stroke's radius), `blend` (§3) and `land_only` (§5).

**Why not raster tiles.** A raster has a resolution, and past it the edge
turns into blocks — the exact failure the owner's 2026-10-17 requirement
forbids. A vector mark is evaluated at whatever resolution asks: the
one-pixel-per-cell screen texture, a deep-zoom tile at 30 px per cell, a 16K
export. Its edge is **one output pixel wide at every one of them**
(`coverage`: `smoothstep(0.5 + (s − half) / max(band, px))`, the half-coverage
contour fixed in kilometres, the width never below one pixel).

**Kilometre convention.** Origin at the map's top-left **edge**, `x` east, `y`
south; grid cell `i`'s centre is `(i + 0.5) · km_per_cell`,
`km_per_cell = map_width_km / gw` (the convention `civ_catchment_radius_cells`
uses; cells are square). A context with no map width paints nothing — a
guessed scale would put a mark on the wrong ground
(`RenderCtx::colour_paint_query`).

**Bounded cost.** A uniform 128 × 128 bucket grid over the marks' bounding box
(`BucketIndex`) lists, per bucket and in mark order, each mark that can reach
it and which of its segments are near; an area entry also carries whether the
bucket's centre is inside, so a pixel's inside test counts only the bucket's
own edges (exact, local). A query visits one bucket. A footprint wider than
the index's margin (a view zoomed out past the grid) falls back to visiting
every mark — never wrong, only slower where rare. **Caps**, refused whole
when exceeded: `MAX_MARKS` 5 000, `MAX_POINTS_PER_MARK` 4 096,
`MAX_TOTAL_POINTS` 250 000, `MIN_RADIUS_KM` 1 m. The measured cost model is §10.

## 3. Blend modes

Three, keyed `colour` / `replace` / `multiply` (`ColourBlend`), all evaluated
on the **material** colour before the light (§4):

| Mode | Formula (0-255, W3C Compositing Level 1) | Reads as |
|---|---|---|
| **Replace** | `s` | The picked colour, shaded by the relief. Red reads red on grey rock **and on snow**. Replaces the material's own texture, but the light (and at deep zoom the tile's micro-relief normals) still models the ground |
| **Colour** ("Tint") | `SetLum(s, Lum(d))` with `ClipColor` (the non-separable `color` mode) | The paint's hue and saturation at the map's own luminance: keeps material texture and relief. Cannot reach a saturated colour on very light or very dark ground — red over snow is pink |
| **Multiply** | `d · s / 255` | A stain: darkens toward the paint, never lightens. Red over grey rock is dark red |

**On "tint".** The brief proposed a *hue-preserving* tint as the default. A
tint that keeps the map's hue can never make grey rock red, which fails the
primary case, so "tint" here is the W3C `color` mode — the paint's hue and
saturation, the map's luminance — and is offered as the second mode.

**Default: Replace at opacity 100 %** (changed from the brief's "tint"
default, with this reason): because the paint goes in before the light,
Replace already keeps the relief — measured, the sun-flip test moves painted
Replace ground by 0.091 against the unpainted 0.094 (relative luminance), and
the frames show crater walls, gullies and snow-rim relief under Replace at x4
through x64 (evidence 01-03). It is the only mode in which the owner's red
reads as the red picked on every ground (Colour on the snow cone is pink,
Multiply nearly black on dark rock). Colour is one tap away for blends that
should keep the ground's own variation (the desert-to-grass blend, evidence 07
and 09, reads best in Colour). An **Erase** mode (CP-2) is the fourth entry.

## 4. Where it composites, and why there

| Stage (land, in `render.rs::land_color` order) | Paint relative to it | Why |
|---|---|---|
| Material mix, splat, texture, canopy, micro-tone, `biome_sat` | **after** | The colour picked is the material colour drawn |
| **Colour paint** | — | `land_color`, right after `biome_sat`, before `light` |
| Light (hillshade, toon bands, layer stack), `bio_blend`, relief chroma, curvature, atmosphere, haze, near-water tint | after paint | So every mode keeps the relief; the look still finishes the painted ground |
| Legacy WORLD paint tint (0.60 over the lit colour) | after paint | World data stays on top of presentation; in practice they are different tools |
| Painter (NPR) styles, rivers (`river_over`), AO × vignette | after paint | Rivers draw over paint; hand-drawn styles act on paint as on everything else |
| Crest, SDF coast/river bands, toon outline | after paint | Unchanged |
| Paper, plate frame, local contrast, colour grade, colour space | after paint | The paint is graded with the look, as one picture, on all three paths |
| Labels, icons, overlays (Godot nodes) | above | Never painted over |

**Water** (`land_only == false` marks only): on the finished water colour
(`sea_color_core` / `lake_color`) before paper, frame and grade, in all three
paths (`RenderCtx::paint_water`). Water colour is already lit by its own
shading, so the blend acts on it there.

**Local contrast** reads the painted picture on all three paths (the screen
pass, the tiles' detail band from the base map, the export), so the three
agree. A paint edge is therefore an edge local contrast can enhance; the
frames show no halo at the crater edges, but this was **not separately
measured** — CP-2 measures the halo at a Replace edge with local contrast at
its maximum and, if one shows, excludes paint from the detail band on all
three paths together (never on one).

## 5. Land only, per mark; across a regenerate

`land_only` is **per mark, default on**: a land-only mark leaves sea and lakes
alone; a mark with it off paints water too — the owner's crater lake
(evidence 02: the lake and the crater walls painted red by one fill plus a
grow stroke). Per mark so one stroke can include a lake and the next leave it.

**Across a regenerate** the marks stay at their kilometres. A new world with
different land under them shows them on whatever land is there; marks that now
fall on water vanish under `land_only` (default) and reappear if land returns.
Nothing is ever deleted by a regenerate. A different `map_width_km` keeps the
kilometres, so the marks keep their ground size and move relative to the
plate; CP-2 shows a one-line notice when a regenerate changes the map width
while marks exist.

## 6. Resolution independence

Same kilometres, same ground, any grid and any output size. Measured
(`tests/colour_paint.rs::the_same_kilometres_paint_the_same_ground_at_two_grid_sizes`):
a hard stroke whose edges are at 175 and 255 km is drawn at 175.20 / 254.80 km
on a 61 × 43 grid and 175.97 / 254.03 km on a 122 × 86 grid of the same 610 km
map (export 488 px wide; one export pixel is 1.25 km), and on a tile at 48 and
384 texels over the same ground at 175.85 / 254.15 and 175.10 / 254.90 km
(`a_stroke_edge_is_at_the_same_kilometre_at_every_zoom`; one coarse texel is
3.4 km). A 16K export evaluates the same function at its own pixel, so it
matches the screen by construction; the banded export equals the monolithic
one byte for byte with marks attached (§11, bar 5).

## 7. Hide, clear, export without paint; looks

- **Hide** — the layer's `visible` flag. A hidden layer is not attached
  (`RenderCtx::with_colour_paint` filters inert layers), so the picture is the
  unpainted one by control flow.
- **Clear** — one undoable step that empties the marks (§8).
- **Export without paint** — an export-dialog switch (CP-6), default **with**
  paint (what the screen shows is what exports).
- **Saved looks** do not carry paint: a look is a style applied to any map,
  paint is this map's annotation. Switching looks, including Satellite relief,
  keeps the marks; their colour is finished by the new look's light and grade
  (Replace red under Satellite relief is still red, lit by that look's sun).

## 8. Undo

Stroke-granular, on **its own stack** (`look_history.gd`'s pattern: a gesture
snapshot, a labelled commit, undo/redo), so undoing paint never undoes a look
change and the reverse. A step is a whole document (marks are small: 5 000 ×
40-point marks measure 3.98 MB as JSON, §9), held structurally shared where the
shell can; the stack shares the app's undo budget (the oldest steps drop
first). Labels: *Paint stroke*, *Fill*, *Erase stroke*, *Clear colour paint*,
*Colour paint shown/hidden*.

## 9. Save format — `appearance_paint.json`, additive and optional

A `DOCUMENT_SLOTS` member, written **only when the layer has marks** (an
untouched project stays byte-identical; older builds ignore an unknown member;
absent ⇒ no paint). Payload: `ColourPaintDoc`
`{"version":1,"visible":bool,"marks":[{"shape":{"kind":"line","points_km":[[x,y],…],"radius_km":r}|{"kind":"area","ring_km":[…],"edge_km":e},"rgb":[r,g,b],"opacity":o,"hardness":h,"blend":"colour"|"replace"|"multiply","land_only":bool}]}`.
An unknown `version` or blend key is refused, not guessed; a missing
`land_only` reads as `true`. CP-3 writes coordinates rounded to the metre (3
decimals of km). Measured: 5 000 marks × 40 points = 6.03 MB at full precision,
3.98 MB metre-rounded, before the archive's deflate; the layer builds in
31.7 ms (`colour_paint_save_size`). Added by CP-3 (2026-10-18, §20).

## 10. Cost (measured 2026-10-18, release, this machine, two independent runs)

`tests/colour_paint_frames.rs::colour_paint_tile_cost`, world 71077345 Classic
800 km at 2048 × 1311, Quality tier + Natural Vibrant, 24 tiles per arm,
alternated; marks are hand-stroke-sized (12 points, about 6-12 km long,
0.5-3 km radius, the three modes in turn):

| Case | Overdraw (marks covering a point) | Off median (min..max) | On median (min..max) | Ratio of medians |
|---|---|---|---|---|
| a. 1 000 strokes over the x4 view (6 × 4 level-4 tiles, ~300 × 200 km) | 0.91 | 5.54 ms (1.51..6.14) / 5.53 (1.64..6.21) | 5.89 (1.48..6.00) / 5.81 (1.57..6.19) | **1.06 / 1.05** |
| b. the same strokes through 24 level-6 tiles | 0.89 | 5.86 / 5.77 | 6.21 / 6.04 | 1.06 / 1.05 |
| c. base raster (`cell_color`, whole grid), 1 000 strokes over the map | — | 135.0 ms (133.8..135.2) / 134.5 | 139.7 / 139.2 | **1.035 / 1.035** |
| d. stress: the 1 000 strokes packed into one 6 × 4 level-6 block | 13.24 | 5.74 / 5.75 | 9.38 / 9.46 | 1.64 / 1.65 |

The bar (≤ 1.25× with 1 000 strokes in view) holds at realistic density.
**Cost scales with overdraw, not with the stroke count**: every mark covering
a pixel is composited there. An earlier run of the same scatter packed into a
6 × 4 level-8 block (~19 × 12 km; overdraw not measured, in the hundreds) was
13.2×. CP-2 therefore surfaces the mark count, and CP-7 (optional) adds
*flatten opaque overdraw*: dropping marks wholly covered by a later opaque
Replace mark.

## 11. Acceptance bars (each with the test that holds it)

1. **Absent = byte-identical.** No layer, a hidden layer, an empty layer and a
   layer whose marks reach nothing draw the same bytes on the screen, export
   and tile paths (`absent_hidden_empty_and_unreached_layers_draw_the_same_bytes`);
   the absent case is held against HEAD by the existing literal censuses
   (`export_raster.rs::adjust_census_tests`, `cel_toon.rs` digests,
   `golden_parity_*`), all green with CP-1. js_reference and every golden are
   untouched: the layer is absent by default.
2. **Same km at every zoom** (§6), tile edge 10 %→50 % within two texels at 384
   texels.
3. **Same km at every grid size** (§6), export edge 10 %→90 % within two export
   pixels.
4. **No tile seams**: two tiles' shared column differs painted by no more than
   unpainted, plus one level (`adjacent_tiles_agree_at_the_seam`).
5. **Banded export = monolithic** with marks, at 1, 37 and 5 rows per band
   (`banded_export_with_paint_equals_monolithic`).
6. **Relief survives every mode**: painted relative luminance spread ≥ 60 % of
   unpainted (`relief_survives_every_mode`; Colour 0.163, Replace 0.131,
   Multiply 0.177 vs 0.171), and the sun still lights painted ground
   (`the_sun_still_lights_painted_ground`: ≥ half the unpainted change under a
   315°→135° sun flip; Colour 0.068, Replace 0.091, Multiply 0.091 vs 0.094).
   Replace reads red (red ≥ 60 levels above green and blue) on every painted
   land pixel.
7. **Cost** ≤ 1.25× at realistic density (§10).
8. **Water only by choice** (`land_only_is_honoured_on_every_path`).
9. **No scale, no paint** (`no_map_width_paints_nothing`).
10. **The primary case, read on frames** at x1/x4/x16/x64 (§12).

Mutation testing (CP-1): 19 mutants, 19 killed — the land-paint block
removed; paint moved after the light (killed only after the sun-flip test was
added: the spread test alone let it survive); the kilometre origin moved half a
cell; the tile and bake footprints fixed at one cell (the bake one killed only
after the export sharpness leg was added); water unpainted on each of the
three paths; the map-width guard replaced by a guess; and in
`colour_paint.rs` the antialiasing floor, the stroke half-contour, the bucket
centre's inside state, the crossing parity, both `ClipColor` branches,
`land_only` on water, the mark cap, the Multiply formula and opacity. Harness:
exact literal, occurs-once, restored in `finally`, hashes checked.

## 12. The volcano case, measured (CP-1 frames)

`colour_paint_frames` (two worlds, 800 km, 2048 × 1311): a red
(`[200, 30, 18]`) fill of the crater in each mode plus unpainted, at x1, x4
(1.86 px/cell), x16 (7.42), x64 (29.7):

- **71077345's snow-rimmed cone** (click at cell 1578, 690; crater ~6 km
  across): Replace reads red on the snow rim and the rock floor with the
  crater relief modelled at every rung; Colour is pink on snow and red on
  rock; Multiply is dark red. Edges are one pixel wide at x64 — no blocks, no
  stair-steps (evidence 01).
- **24601's crater lake** (~5 km): the fill with `land_only` off paints the
  lake and the walls; Colour keeps the rock's texture and the lake's own
  shading in red, Replace flatter and redder, Multiply near black
  (evidence 02).
- **Default chosen from the frames: Replace** (§3).
- **x1**: a crater a few km across is a few pixels at fit; the mark is there
  and correct, but at fit it is a dot (evidence 04, 05).

The hand-drawn set, before/after at x1-x64 (evidence 06-10): a coast tint
(Colour, teal, 2.5 km, soft) follows the coast at every rung; a desert-to-grass
blend (Colour, 12 km, hardness 0) reads as grass with the ground's relief and
grain kept; a mountain tint (Multiply ochre) keeps the snow cone's shading.
No seam and no pixelation seen at x64.

## 13. The fill helper (CP-5) — feasibility measured

Prototype in `colour_paint_frames.rs` (`crater_fill`, `capped_fill`,
`ring_around`), click-seeded:

1. **Closed fill to the pour point** — grow from the click the way water
   fills: take the lowest boundary cell, raise the level, stop at the first
   boundary cell lower than the level reached; trace the level-set ring just
   under it (`contour_polylines`, so the outline is the field's own smooth
   contour, not a cell staircase). 0.2-1.0 ms.
2. **Capped fill** for breached craters — every cell below the rim level less
   a margin, connected to the click and within a radius cap (the cap closes the
   breach with an arc), box-blurred twice and traced at one half. 0.45-0.74 ms;
   rings of 52-110 points; craters measured 5-10 km across.

Findings that shape CP-5: **every crater found on these two worlds was breached
or too small at grid resolution** for the closed fill (it stopped within 20
cells), so the capped fill is the main mode; one capped fill leaked down a
breach into a flank lobe (evidence 03), so the tool needs a **live preview with
level, cap and grow controls**; the tile draws a crater lake about 1 km beyond
the grid contour, so a fill of a lake needs **grow** (a hard stroke along the
ring, 1.5 km in the frames); and an automatic crater detector mistook a river
gorge's rimmed pits for a crater on 71077345, so the tool is **click-seeded**,
not automatic. Output is an `Area` mark (plus a grow `Line`), in kilometres,
undoable like any stroke. Feasible.

## 14. Phone and tablet

- **Brush size** in kilometres with a one-thumb control: the size row's
  slider plus a two-finger pinch on the canvas while the tool is armed
  resizes the brush (not the map) with the ring shown; a long-press samples
  the colour under the finger (an eyedropper, which `spec/06-phone.md`'s
  long-press-pin conflicts with — the owner's MAP_CONTEXT F1 fork decides;
  until then the eyedropper is a button).
- **Picker as a bottom sheet** on phone (the existing picker inside the sheet,
  the gamut row under it, the swatch row above).
- **Reachability**: arming Paint colour selects CARTO, whose phone sheet must
  keep the tool-options row visible (`MISTAKES.md`: *"check that a thumb can
  reach the bar"*); proven on glass with `adb`, not by a desktop probe.

## 15. Print gamut, swatches, Session mode

- **Print gamut**: the picker's existing ET-E3 row warns when the picked colour
  is outside the print gamut. The warning is about the picked colour; the
  drawn colour is the picked one shaded by the light and graded by the look, so
  CP-2 adds no second check on the result.
- **Swatches** (a nicety, CP-2): a row of presets beside the picker — lava red
  `200,30,18`, ash grey `120,118,112`, sulphur yellow `220,200,60`, sea blue
  `40,90,150`, grass green `86,140,58`, sand `210,180,120`, ochre
  `225,160,110`, snow `240,240,236` — plus the last eight colours used.
- **Session mode** hides WORLD; CARTO stays visible, so the tool and the layer
  stay usable in Session mode (presentation, not world editing).

## 16. Milestones

| ID | What | Acceptance |
|---|---|---|
| **CP-1** | Engine: `cartalith_spatial::colour_paint` (marks, blend modes, caps, bucket index, document), composited by `render.rs` in `land_color` (land, before the light) and `RenderCtx::paint_water` (water, per-mark opt-in) at all three consumers' own pixel footprint: `cell_color` (1 cell), `BakeFields::pixel_at` (the export step), `render_biome_tile_rgba_water` (`cells_per_px`). Attached by `RenderCtx::with_colour_paint`; absent by default | Bars 1-10; frames read at x1/x4/x16/x64; mutation-tested |
| **CP-1M** | Emissive **materials** (§18): `cartalith_spatial::colour_paint::material` -- lava (and the Ice / Toxic glow ramps it gives for free) as a crust lit like ground plus a glow added after the light; merged lava bodies; ragged km-keyed margins; halo; `heat_along` | §18's bars; frames read at x1/x4/x16/x64 on two worlds; mutation-tested |
| **CP-2** | Binding and shell tool: `WorldGen` holds the layer (`Option<ColourPaint>`), `#[func]`s to set/get the document, append a mark, clear, show/hide; attached at the three producers (`build_color_texture`, `lod_worker::SnapshotInputs` with the document in `digest()`, `export_raster.rs::export_render_with` / `export_session.rs`'s parts); `CARTO ▸ Colours ▸ Paint colour`: ET-D1 stroke engine to a simplified kilometre polyline, picker with the gamut row, swatches, mode (Replace / Colour / Multiply / Erase), radius in km, hardness, opacity, land only, hide, clear, own undo. Engine addition: **Erase** mode (blend toward the unpainted colour the query was handed). Measure the local-contrast halo (§4) | Every producer moves bytes with a mark and none without (`MISTAKES.md`: attaching is not the consumer reading it); cache keys change on every edit, undo and redo; probes for the tool, and on glass for phone |
| **CP-3** | Save slot `appearance_paint.json` (§9), **after GF-8 slice 3 lands** | Round trip byte-identical; an untouched project byte-identical to HEAD; a real prior-format archive opens unchanged; size at 5 000 marks reported |
| **CP-4** | Phone and tablet forms (§14) | Thumb-reachable on glass; brush resize does not zoom the map |
| **CP-5** | Fill helper (§13): click, preview, level / cap / grow, commit as `Area` (+ grow `Line`) | The crater cases of §12 filled in one gesture; undo one step |
| **CP-6** | Export option "without paint"; export census leg with a mark | Export with paint equals the screen's composition; without equals HEAD |
| **CP-7** | Optional: flatten opaque overdraw (§10) | Stress case d within 1.25× |

## 17. Owner questions, with the defaults this scope builds unless told otherwise

1. **Default mode** — Replace (measured, §3), not the brief's tint. Keep?
2. **Saved looks carry paint?** — No (§7).
3. **Export default** — with paint (§7).
4. **Paint water by default?** — No: `land_only` on per mark, off by a toggle (§5).
5. **Map-width change with marks** — keep kilometres, notify (§5).
6. **Long-press on phone** — button eyedropper until MAP_CONTEXT F1 is ruled (§14).
7. **Lava defaults** (§18) — heat 0.75, plates medium (crack density 0.5,
   0.71 km top cell), halo 1.5 km, crust `34,33,38`, cooling along a stroke
   0.6. Keep?

## 18. Emissive materials — lava (CP-1M)

**Why.** The owner on CP-1's frames (2026-10-18): *"Currently it doesn't look
convincing at all for lava/magma."* The use case was the inside of a volcano;
a flat red, even shaded by the relief, reads as red paint. Lava is not a
colour but a surface, so a mark can now carry a **material** in place of its
flat colour (`ColourMark::emissive`, `cartalith_spatial::colour_paint::material`).

**What real lava looks like** (the standard the frames are judged against):
a chilled crust, near-black with a slight blue-grey cast, broken into plates;
the incandescent interior shows only in the cracks between plates and at
vents; colour is temperature — dull red, orange, yellow, white-yellow at the
hottest. In daylight the crust reads dark and the cracks orange; at night the
cracks dominate and the ground near the flow picks up a red glow. The crack
network is self-similar over a wide range of scales.

**Design** (per output pixel, from the mark's geometry and the pixel's km
position and footprint only — the same pure function on the screen, the tiles
and every export band):

| Part | How | Why |
|---|---|---|
| Temperature `t` | `heat` x (1 − `heat_along` x along-fraction) x a rim-to-core profile (`s / depth`, depth = a stroke's radius or half an area's equivalent radius), plus low-frequency km noise at three plate sizes | Hot core inside, cooling to the rim and down a flow; hot channels and cooler rafts |
| Crust | the mark's `rgb` (Lava preset `34,33,38`), plate-to-plate albedo ±11 %, applied **before the light** with the mark's blend | Sunlit crust is lit like ground: dark, with the crater's relief |
| Cracks | multi-octave cellular (Voronoi-edge) network; top cell 2.0 → 0.25 km by `crack_density`; each octave 2.3x finer, narrower and dimmer, and needing a hotter crust to open (+0.08 `t` per octave); finer octaves drawn over only about half their edges (a km noise mask) and every crack's width wandering along its length; down to 4 m cells | A few wide seams between big plates on cool crust, fine broken fractures where hot; without the mask the octaves nested into even polygons and read as a mesh |
| Zoom law | an octave is drawn once its cell spans 2.5 output pixels, fading in over one doubling (`CANOPY_NYQ_PX`'s rule); an octave not yet drawn contributes its **mean** open fraction | Detail arrives at every zoom; overall brightness does not jump |
| Glow | ramp colour at `t` (Fire: black → dark red → red → orange → yellow → white-yellow) x open fraction (cracks ∪ molten core, `t` > 0.8), **added after the light**, after haze and WORLD paint, before Painter styles and rivers; the `ao x vignette` step applies to the ground part only, so occlusion never dims the glow | Self-lit; hillshade and AO cannot darken it |
| Halo | outside the mark, `HALO_GAIN 0.9 x (1 − d/R)^3` of the ramp at the edge's heat, R = `glow_radius_km`; the mark's index reach widened by R | Glow spills a little onto the ground; per pixel, not a screen blur, so screen/tile/export agree |
| Ragged margin | the signed distance moved by km-keyed value-noise lobes (wavelength 1.6 depths down to 4 m, amplitude 0.16 of each wavelength, octaves Nyquist-gated) | A flow's edge is lobate, not a pen stroke's parallel offset (the first flow frames read as a pipe) |
| Lava bodies merge | consecutive emissive marks reaching a pixel form one group: coverage is the union, temperature the hottest mark's base heat; a covering flat mark flushes the group and hides its glow | A stroke over a fill no longer draws its own cool rim across a lake (seen in the first frames) |
| Presets | the ramp is the only difference: **Fire** (lava, embers at low heat), **Ice**, **Toxic** | Falls out of the mechanism; nothing else widened |

Options per mark: `heat` 0..1, `crack_density` 0..1, `glow_radius_km`,
`heat_along` 0..1 (strokes), crust colour (`rgb`), `ramp`. Static (no
animation, by the brief). A flat mark's saved JSON carries no `emissive` key,
so CP-1's document bytes are unchanged.

**Bars** (tests in `tests/colour_paint.rs` and the module's unit tests):
identity — a lava mark reaching nothing draws the same bytes on all three
paths, and flat documents keep their JSON
(`an_unreached_lava_mark_draws_the_same_bytes_and_flat_docs_keep_their_json`;
the absent case is CP-1's bar 1, unchanged); banded export = monolithic with
lava at 1, 37 and 5 rows per band (`banded_export_with_lava_equals_monolithic`);
no tile seam (`lava_opens_no_tile_seam`); same ground across grid sizes —
footprint IoU 0.906 between a 61 x 43 and a 122 x 86 grid of one map at one
export size, bar ≥ 0.9 (`lava_paints_the_same_ground_at_two_grid_sizes`);
lighting independence — the glow over a fully covered flow moves 0.25 % under
a 315°→135° sun flip and 0.26 % with AO and cast shadows at 1.0 (which darken
the ground 7.1 %), bar ≤ 3 % (`the_glow_is_independent_of_sun_and_occlusion`);
a flow cools along its length (vent 5 200 vs toe 725 glow levels,
`a_flow_cools_along_its_length`); lava on water glows (`lava_on_water_glows`);
the index equals brute force with lava (halo and margin reach), merge/order/halo
rules, the ragged edge, zoom brightness conservation with structure at zoom,
ramp order, heat and halo (unit tests).

**Cost** (`colour_paint_tile_cost` cases e and f, release, this machine, two
runs in separate processes, another lane's builds sharing the machine): 1 000
strokes of which every tenth is lava (heat 0.75, cooling 0.6, 1.5 km halo),
ratio of median tile times **1.12 / 1.14** over the x4 view (level 4) and
**1.09 / 1.09** through level 6, against **1.07 / 1.09** and **1.09 / 1.07** for
the same strokes all flat (cases a, b); bar ≤ 1.25x holds. Single tiles reach
60-70 ms outliers in both arms (the shared machine), which the medians absorb.

**Frames** (`colour_paint_lava_frames`, evidence
`cartalith-native/docs/evidence/colour-paint-lava-2026-10-18/`): 71077345's
snow-rimmed dome (click 1578,690: the crater fill + a 1 km grow ring + a 38.6 km
flow down the flank, cooling along it) and 24601's crater (detector pick), at
x1 / x4 / x16 / x64, a heat sweep (0.25 / 0.5 / 0.95) and the Ice and Toxic
ramps. Read honestly:

- **x64** reads as lava: near-black plates, orange seams, broken secondary
  fractures, a yellow-white core, a ragged margin, a faint warm halo on the
  snow and the green. The top-level seams are **still recognisably cellular**
  — at full resolution it reads closer to a well-made stylised lava texture
  than to a photograph of a lava lake, whose seams are more jagged and less
  evenly polygonal (evidence 07; evidence 08 sets it beside CP-1's flat red).
- **x16** reads as a glowing crater and a dark flow with orange seams; the
  heat sweep reads as cooled crust → active lake → white-hot lake.
- **x4** reads as a dark-red spot with a dark line — honest for its size (the
  crater is a dozen pixels), but not visibly "lava" without context.
- **x1** the crater is a few pixels; the mark is there and nothing more can be
  said at that scale.
- **heat 0.95** clips a large white core: reads as a molten lake, slightly
  blown out.

Mutation testing (CP-1M): 16 mutants, 16 killed (one, the halo showing
through a covering lava mark, survived the first run and was killed by a test
added for it) — the glow add removed; the glow put under AO; the water glow
dropped; an unresolved octave's mean set to 0; the Nyquist gate removed; the
rim-to-core profile flattened; `heat_along` ignored; the halo gain 0; `heat`
not read; a ramp stop moved out of order; the index reach forgetting the
ragged edge, then the halo; the merge taking the coolest mark; a flat mark not
flushing the lava group under it; the ragged edge removed. Harness: exact
literal, occurs-once, restored in `finally`, no residue (the files came back
with LF line endings, which the repository's own storage is).

## 19. CP-2 as built (2026-10-18)

**Engine binding** (`cartalith-godot/src/colour_paint_bridge.rs`):
`WorldGen::colour_paint` (`Option<Arc<ColourPaint>>`) and
`colour_paint_epoch`. `#[func]`s: `colour_paint_info` (`marks`, `visible`,
`emissive`, `revision`, and `km_per_cell` -- absent with no map width),
`colour_paint_document` (the v1 JSON, `""` for none), `colour_paint_set_document`
(`""` removes; a refused document changes nothing and says why),
`colour_paint_add_mark` (one `ColourMark` JSON), `colour_paint_set_visible`.
Attached at every producer: `build_color_texture` and the sculpt preview,
the tile snapshot (`SnapshotInputs::colour_paint`, hashed behind its own
separator word only when present, attached in `render_tile_conformed_cached`),
`export_render_with` and the export snapshot (`Parts::ctx`). The tile and grid
cache keys gain `;cp<epoch>` only while the layer has visible marks, so every
key without paint is the pre-CP-2 string. Marks keep their kilometres across a
regenerate; the index is rebuilt at the new grid's scale
(`colour_paint_rescale`, at each of the three source assignments).
Producer tests: `lod_worker::a_snapshot_with_colour_paint_paints_and_keys_its_tiles`,
`export_session::the_export_snapshot_draws_colour_paint`.

**Shell** (`shell/colour_paint_tool.gd`, owned by `cartography_workspace.gd`,
drawn last in CARTO ▸ Colours as "Paint colour"): "Paint on the map" arms tool
`colour_paint`; a press-drag-release through the app's map dispatch is one
stroke, sampled every quarter radius in km, simplified (RDP at a tenth of the
radius) and appended as one mark. Controls: **Material** first (Colour, Lava,
Embers, Ice glow, Toxic glow -- presets of §18's mechanism), the picker with
the print-gamut row (`DccWidgets.confirmable_color`), §15's eight swatches,
Blend (Replace first: the measured default of §3, unruled), Radius in km with a
readout in cells and screen pixels at the current zoom, Hardness, Opacity, Land
only, and for a material Heat, Crack density, Glow radius and Cool along stroke;
Undo / Redo on the tool's own stack (whole documents; drops its oldest steps to
stay inside what the app's undo budget has left, `undo_stats()`), Hide, Clear.
The brush ring follows the cursor while armed. Every bridge call is gated on
`generating`. Probe: `_colourpaint_probe.gd` (desktop, tablet and phone).

**Not done in CP-2, and why:** the fill helper (CP-5) -- still feasible per §13,
left for its own milestone; the local-contrast halo measurement at a Replace
edge (§4) -- not measured; phone reachability -- the phone run proves the tool
renders and operates through the app's dispatch, not that a thumb can find
CARTO ▸ Colours behind the bottom bar (CP-4); the export dialog's "without
paint" switch (CP-6). Session mode was not separately probed (§15 says CARTO
stays visible there).

## 20. CP-3 as built (2026-10-18)

`appearance_paint.json` is registered in `cartalith-io`'s `DOCUMENT_SLOTS` and
is **engine-owned** (`project_bridge.rs` `ENGINE_OWNED_SLOTS`). The save writes
`colour_paint_save_doc` -- the layer's document with every coordinate, radius,
edge and glow radius rounded to the metre -- only when the layer has marks;
the open reads it with `colour_paint_from_data` after `load_save` has dropped
the previous project's layer. The format, the reading rules and the forward
compatibility note are `SAVEFILE_COMPAT.md`'s "`appearance_paint.json`"
section. Tests (`project_bridge.rs`): `colour_paint_survives_a_real_archive_round_trip`
(round trip equal, re-save byte-identical, engine-owned, size delta printed:
+459 bytes for two marks), `a_project_without_colour_paint_opens_and_saves_without_the_slot`
(a real pre-slot archive opens with no paint and no warning; no layer or an
empty one writes no slot), `a_damaged_colour_paint_member_warns_and_opens_without_paint`.
Probe: `_colourpaint_probe.gd` leg 7 saves through `EngineBridge.save_project`,
clears, reopens through `load_save`, and finds both marks and the material.

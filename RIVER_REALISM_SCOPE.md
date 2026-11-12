# River realism: research and a scope

**Agent-produced 2026-11-12**, from the owner's request. Documents only: no code was changed. It
defines proposals and milestones with numeric bars; it does not track status
(`cartalith-native/docs/STATUS.md` does). Everything about this tree was read at the symbol named, on
origin/main `823a8147`; a document's claim about itself was not trusted. **Another lane is editing
`river_stroke.rs`, `river_delta.rs` and the routing concurrently (Ruling ER, from `77270a55`), so every
code point below is "as of `823a8147`": re-open it at its symbol before building.**

> Owner, 2026-11-11/12, verbatim: *"graphically I'd like an opus or fable agent think about improving how
> realistic rivers look visually. (and add. Features as waterfalls, think about where rivers start, water
> actually accumulates for a lake."* — *"For example the current lines don't look very realistic. The
> effect and visuals should be more subtle and realistic."* — *"And à width that matches the volume of the
> river."*

**Evidence tags.** CONFIRMED: read on the page, in the paper's own text, or in a PDF opened this
session. SECONDARY: read in an abstract, a citing paper or a search summary only. UNCONFIRMED: recalled,
or no source found; not to be built on. Numbers about this tree are either read at a symbol, derived by
arithmetic shown, or quoted from a named measurement in another document (never "measured" here: nothing
was run except the frame captures in §1.7).

---

## 0. Summary

1. **On a whole-world map every river is drawn at the same width.** `channel_disc`'s half-width is a
   symbolic law in cells, `(0.6 + 3·mag² + 0.45·(order − 1)) · slope_fac · width_k`, clamped to
   `[0.5, 9·width_k]`, and `width_k = clamp(800 / map_km, 1/16, 16)` is `1/16` on a 40 075 km world. The
   largest term the bracket can reach there (order 8, `mag` 0.5) is `4.5 / 16 = 0.28` cells, under the
   `0.5` floor, so **the floor is the whole network's width** (RZ-0 measured exactly that on every
   40 075 km shape: "one value for the whole network"). A river is then 1 cell wide, 19.6 km on the
   owner's world: 7.4 px at x16, 30 px at x64, Amazon or trickle alike. That is the "wide, uniform, pale
   strokes" of `RENDER_TARGET_GAP_ASSESSMENT_2026-10-17.md` and the owner's "lines don't look very
   realistic". On an 800 km map (`width_k` 1) widths vary 1.2 to 18 cells, but by **Strahler order**
   first and discharge only through a capped logarithm, and order and discharge disagree on this
   engine (RZ-0: 26 % of the owner world's top-decile discharge length is order 1).
2. **Width should follow discharge as `w ∝ Q^0.5`** (Leopold & Maddock 1953, CONFIRMED; the Mississippi
   basin fit `w = 13.4·Q^0.46`, CONFIRMED). Two things follow. The delta code's split rule
   `w_a² + w_b² = w_p²` is this law exactly (`w² ∝ Q`), so a `Q^0.5` trunk and the fans agree by
   construction. And at world scale the *true* width of the largest river is about 0.15 cells (§2.1),
   sub-pixel until x64, so "width matches volume" has to be a **relative** law with a chosen scale: the
   biggest river's width is the one knob (owner question Q1).
3. **Colour and edges carry no water information.** The colour is an 8-entry Strahler palette
   (`RIVER_ORDER_RGB`) pulled toward the preset's ink; a lake is `sea_color_core` at a fixed depth 0.30
   whatever its depth or size; the edge is solid to the geometric edge with a 1 px fringe, and the
   floodplain tint is one fixed lift at every climate. Realistic water darkens and saturates with depth
   and volume, lightens and takes the land's tint when small, goes turbid in lowland and arid country,
   and its green riparian strip is a desert feature, not a rainforest one (§2.2).
4. **Rivers start at a flow threshold and nowhere else.** Every drawn run begins at the first cell whose
   rain-weighted accumulation passes `river_flow_thresh` (0.04 % of the grid, eased by map size), with
   a slope term only when `river_density ≠ 1`. No spring, snowmelt or glacier source exists in
   `cartalith-hydrology` (zero matches), and a head is drawn at full weight from its first point. The
   channel-head literature's rule is an inverse area–slope threshold `A·S² = c` (§2.3): the engine has
   the shape of it, uncalibrated and gated on density.
5. **Lakes accumulate water topologically, not by budget.** A depression is a lake when it is deeper than
   `LAKE_DEPTH` 0.004 and the cell's rain is at least `LAKE_RAIN` 0.22; its level is the spill height
   always. No inflow, evaporation or basin area enters (`basin` and `basin_outlet` exist with no
   consumer), so a trickle-fed wet pool is as full as one fed by a great river, and no basin is ever a
   dry playa. The water-balance rule is level `= min(spill, balance)` (§2.4); the one prototype in this
   tree (`lake_candidates_harness.rs`) found that a naive balance drains real lakes, which fixes the bar.
6. **Waterfalls exist only as a landmark pick, never as something the river draws.** `pool_waterfall`
   finds channel cells with a 40 m drop, 6 % gradient and twice the channel threshold; its lithology
   term (`carve_width::knickpoints`, a strong cap over a weaker substrate) runs only with
   `geology_processes`, which is **off** in the app (`params::defaults`), so every app world uses the
   rock-blind detector. Rapids do not exist in any form. The slope classes that say where white water is
   are published (cascade > 0.065, step-pool 0.03–0.065, plane-bed 0.015–0.03, pool-riffle < 0.015;
   Montgomery & Buffington 1997, CONFIRMED via Wohl & Merritt 2005).
7. **Ranked proposals** (§3): RR-1 hydraulic width (render only, the biggest visible gain); RR-2 colour
   by volume and climate, soft banks, climate-gated floodplain (render only); RR-4 rapids and waterfalls
   drawn on the river and tied to the landmark detector (render only, plus one decoupling question);
   RR-3 heads that fade in (render only); RR-5 lake depth tone (render only); RR-8 meander wavelength
   scaled to width (render only); then two generation changes that need a ruling each, RR-6 lake water
   balance and RR-7 channel heads by area and slope.

---

## 1. What ships today, read at the symbols

### 1.1 The drawn river's geometry and width

- **Network to stroke.** `trace_river_polylines(order, recv, gw, gh, 1)` (`cartalith-engine/src/lib.rs`)
  traces every run from order 1. `WorldGen::river_draws` builds a `DrawnRun` per run
  (`river_stroke.rs`): render points through an RDP-simplified Catmull-Rom curve, per-point `widths`
  (cells), `colors`, `orders`, `discharge` (`flow_discharge`, NaN for no reading), `ranks` (RZ-2),
  `pieces` cut once per water crossing, and `reach` carrying each end into the water
  (`ShoreReach`). `RiverGeometry.runs` is in draw order, tributaries before their trunk.
- **The width law** is `cartalith_hydrology::channel_disc` (the reference's `buildRiverNetwork` disc,
  lines 4532-4537), read per traced cell by `river_half_width_profile` and made non-decreasing
  downstream by a running maximum: `half_w = (0.6 + 3·mag² + 0.45·(o − 1)) · slope_fac · width_k`,
  `mag = min(1, ln(Q/T) / ln(0.05·n))`, `slope_fac = 1 / (1 + 5·|∇h|·w)`, clamped to
  `[0.5, 9·width_k]`; `width_k = river_width_scale_k(map_km) = clamp(800 / map_km, 1/16, 16)`.
  **It reads discharge**, through `mag`, but `mag` is a logarithm normalised to `ln(0.05·n)` (11.8 on
  the owner world), so a river carrying 600× the threshold scores `mag` 0.54 and the quadratic term
  0.88 cells: the order term (`0.45` per order) dominates. On the owner world (`width_k` 1/16) the
  bracket cannot exceed `0.28` cells for any order up to 8, so the `0.5` floor binds everywhere;
  `RIVER_ZOOM_VISIBILITY_RESEARCH.md` RZ-0 measured "one value for the whole network (the floor at
  `width_k` = 1/16)" on every 40 075 km shape. `settle_join_widths` and `channel_lmax` do not change
  this. Derived, not measured: a full width of 1 cell is 19.6 km there, 0.46 px at the fit view
  (floored to `MIN_STROKE_PX` 1), 1.8 px at x4, 7.4 px at x16, 29 px at x64.
- **The seam** `river_px_width(RiverPoint, ppc, appearance)` returns `(full width px, alpha)`:
  `width_cells · ppc · wm · river_width_factor`, floored at `MIN_STROKE_PX` 1, where `wm` is the
  reference's headwater width de-emphasis (`headwater_deemphasis`, 0.55 at 1 ppc fading out by 8 ppc,
  scaled by `headwater_weight(u)` over rank 0.5..1) and alpha is `river_visibility(u, ppc)` (Ruling EN
  and its addenda: 5.5 % of the network at a world's opening view, geometric to the whole at x64;
  `hide_short_stubs` for Ruling EQ item 3). The painted field's shader (`map_shore.gdshader`) and
  `river_field::coverage`/`pixel_paint` mirror it, held equal by tests.
- **Edges.** Solid to the geometric edge, alpha falling over `EDGE_FRINGE_PX` 1 outside it (chosen by
  looking, RV-2). RIM-2's bank outline (`bank_coverage`, 1 px in the preset's river ink) runs only where
  a preset sets `river_bank` (Antique 0.40, Ink 0.60, and the engraved/atlas rows of
  `render_workspace.gd`'s `STYLE_PRESETS`; off on Natural Vibrant, Default, Watercolor, Print, Night and
  the rest).
- **Meander** (`river_meander::display_meander`, Ruling ER): a sine of amplitude 0.3 cells and wavelength
  4 cells on every run alike, clamped inside the traced cells; invisible below about x16. Not scaled to
  the channel's width.
- **Routing** (`cartalith_hydrology::rough`, Ruling ER): D8 with a slope-scaled noise perturbation, on in
  the app, off in `WorldParams::defaults`.
- **Deltas** (`river_delta::delta_fans`): render-only distributaries for a mouth of own order ≥ 3 and
  discharge ≥ 2e-4 of the grid, widths splitting as `w_a² + w_b² = w_p²`, ranked for the zoom law.

### 1.2 Colour, light and the land beside the river

- **River colour** is `RIVER_ORDER_RGB` (`cartalith-godot/src/lib.rs`), eight RGB entries from
  (156,208,228) at order 1 to (22,36,80) at order 8, blended along the course (`blended_orders`,
  `palette_at`), then `render::river_style_color` pulls it toward the preset's `river_ink_r/g/b` by
  `river_ink` and scales alpha by `river_opacity`. **Nothing in it reads discharge, depth, slope,
  climate, sediment or sun**; two rivers of the same order are the same colour whatever they carry, and
  the colour darkens with order, which RZ-0 showed is not volume. The painted path takes this colour from
  `river_color_texture` ("the map with every river at full coverage"), composited through the preset's
  paper and grade.
- **Lake colour** is `render::lake_color`: `sea_color_core(a, 0.30, …)` lifted by a constant, so a lake
  is one tone at a fixed notional depth of 0.30 regardless of `fill_level − field` (which
  `WaterBodies.fill_level` holds per cell) or of its area; the sea alone has bathymetry
  (`cell_color_river`'s `depth`) and Ruling CI's shelf term (`shelf_km`). `crater_lake_tint` is the one
  per-lake variation.
- **Floodplain** (RIM-3): the shader's `floodplain(land, w)` lifts the land toward
  `land·(0.96, 1.03, 0.94) + (0.012, 0.034, 0.004)` over a band of `1 + edge` cells
  (`FLOODPLAIN_BASE_CELLS`, `FLOODPLAIN_PER_HALF_WIDTH`), weighted by `(1 − headwater_weight)`, the
  visibility alpha and `floodplain_strength`; one lift at every climate. The reference's B3 river
  bands (`TerrainAppearance::sdf_rivers`, `apply_river_sdf`: damp bank to 2 cells, wetland green to 7,
  floodplain to 16) ship at `0.0`, off.
- **Relief.** Generation carves every traced run into the height field (`carve_channel_network`, step
  `drop` 0.0006, lakes kept above `CARVE_KEEP_LAKE_DEPTH`); the renderer shades `valley_shade_field`
  (RV-3), the carve filled back and a smooth valley cut along the drawn line, instead. Water cells are
  never shaded by it. No specular or sky term exists on river pixels.
- **Three paths, one law** (RIM-7): the screen shader reads the RGBA16F field
  (`R` distance, `G` half-width, `B` rank `u`, `A` plate-frame weight; `MAX_TEXELS` 12.6 M, all four
  channels used); deep-zoom tiles and every export evaluate `PaintSource::raster` and `pixel_paint` at
  their own pixels; a look without `rivers_as_water` strokes `river_view_mesh` through the same seam.

### 1.3 Where rivers start

- `compute_flow` seeds each cell with its rainfall (mean-normalised, floor 0.05; "rivers accumulate
  runoff, not area", Whipple & Tucker 1999 per its doc) and pushes the sum to one receiver in
  descending-height order; `build_channels_routed_with` channelises where
  `flow > channel_threshold(T, slope_n, density)`, `T = river_flow_thresh = 0.0004·n / (terrain_detail_k ·
  river_coarse_ease)` (reference 4493), `channel_threshold = (T / density) · (1 + 8·slope_n)^(−|ln density|)`.
  On the owner world `terrain_detail_k` is 1 (19.6 km cells are coarser than the 0.39 km reference
  cell), `river_coarse_ease` is 16 (capped), so `T ≈ 67` cell-units; with `river_density` 1.55 the slope
  exponent is 0.44, so steep ground channelises earlier than flat. At density 1 the slope term is the
  identity.
- A traced run begins at the first channel cell; the stroke draws it at full alpha and the 1 px floor
  from its first render point (`stroke_mesh`; RV-2's "headwaters never draw thinner than 1 px"). There is
  no taper to nothing, no spring, no snow-line or glacier source: `grep -ci "snowmelt|glacier"` is 0 in
  `cartalith-hydrology`, `cartalith-climate` and `cartalith-engine`. A **spring** exists only as a
  landmark (`pool_spring`, `SPRING_TERMS`; GF-5's `spring_line_potential` in `cartalith-erosion/src/karst.rs`
  needs `geology_karst`, off in the app).
- Discharge only grows downstream: no evaporation, infiltration or abstraction along a channel, so a
  river crossing a desert is as wide at the far side as where it entered (realistic rivers lose water
  there: the Nile below the Atbara, SECONDARY).

### 1.4 How lakes are placed and filled

`cartalith-civ::build_water_bodies` (`LAKE_FILL_RESEARCH.md` Part 2, re-opened at the symbols: `LAKE_DEPTH`
0.004, `LAKE_RAIN` 0.22, `wb_visit`, `fill_level`, `basin`, `basin_outlet`, `apply_lake_area_floor`,
`water_body_topology`):

- below-sea components are ocean or lake by edge contact (Rulings Q and T);
- above sea, a 4-neighbour priority flood from the edges and the ocean; a cell is lake when
  `filled − field > LAKE_DEPTH` **and** `rain ≥ LAKE_RAIN` (a per-cell point test, so a pool straddling
  the 0.22 contour is half dry); the level is the **spill height**; Ruling CC's area floor removes
  1–3-cell pits (4 cells at 2048×1311);
- `basin` (per cell) and `basin_outlet` (per basin) exist as data **with no consumer**;
- `water_body_topology` gives Ocean / EndoreicLake / MapBoundedWater from edge contact; inflow, outflow,
  salinity are "explicitly not computed";
- rivers route over a separate 8-neighbour fill (`build_routing_surface`) through every lake to its
  outlet; the carve keeps lakes; the stroke breaks at lake cells and `ShoreReach` carries the end in.

So "water accumulates for a lake" is true in the sense that a lake needs a closed depression and the
river network leaves it at the spill point, and false in the budget sense: nothing asks whether the
inflow could fill it, every qualifying pool is brim-full, and an arid basin is either a full lake or no
lake. No evaporation term reaches lakes anywhere (`grep -rli evaporat crates` hits climate's rainfall
model, droplet erosion, trade's salt-lake check on an already classed lake, and the ignored prototype
`cartalith-godot/tests/lake_candidates_harness.rs`, whose candidate 2 is a per-basin balance).

### 1.5 Waterfalls, rapids, knickpoints and lithology

- **No river pixel is ever drawn as a fall or a rapid.** The stroke, the field, the tiles and the export
  know width, colour, rank and the plate frame; nothing reads the slope along the run.
- **The waterfall landmark** (`cartalith-civ/src/landmark.rs::pool_waterfall`): a land channel cell with
  drop to its receiver ≥ `WATERFALL_MIN_DROP_M` 40, gradient ≥ `WATERFALL_MIN_GRADIENT` 0.06 (grid
  scale: "what this asks is that the reach be steep"), flow ≥ 2× the channel threshold; scored by river
  gradient 0.35, drop 0.30, flow 0.20, confinement 0.15 (`WATERFALL_TERMS`, Category C). With
  knickpoints present a fifth term "resistant cap over weaker rock" takes 0.30 (`WATERFALL_TERMS_ROCK`)
  and the facts name both rocks. It is drawn as a badge/glyph (`dcc_icons.gd` `lm_waterfall`), filed
  in the landmark list, linkable from the vault (Ruling EI).
- **Knickpoints** (`cartalith-erosion/src/carve_width.rs::knickpoints`, GF-6): a channel cell exposing
  its column's cap whose receiver exposes the substrate, cap stronger than substrate ("the Niagara
  form"); derived on demand from the column. `RockView::for_world` supplies them **only when the world
  was generated with `geology_processes`**, otherwise `KNICK_PROCESSES_OFF`; `params::defaults` sets
  `geology_processes = false` (GF-2's bars B1/B2/B8, coordinator 2026-09-27). `geology_model` is on, so
  app worlds **do** carry a column and an exposed-rock strength `s` (`RockView::for_world(ws, p)` "gives
  the exposed rock's `s` on any world with a column, the app's included"); the GF-6 measurement found
  knickpoints 76–103 cells a world and "few of them pass waterfall's 40 m drop and 6 % gradient at about
  0.4 km a cell".
- **Hanging valleys, fault scarps, coastal cliffs**: no detector of any of the three. Fault
  (`boundary_type`, `shear_field`) and coast (`shore_depth`, `coast_distance`) data exist for a
  detector to read.

### 1.6 What the target asks for

`design/vision-render-2026-10-17/target_map_render.webp` (opened): rivers are **thin, dark blue-green
lines** that follow valleys, widen gently toward the sea and read as part of the land (no outline, no
pale halo), with a lighter turquoise shelf where they meet the coast; lakes are **crisp dark basins**; the
only wide water is the sea. `RENDER_TARGET_GAP_ASSESSMENT_2026-10-17.md` graded today's rivers PARTLY at
x1/x4 and GAP at x16/x64 ("wide, uniform, pale strokes: about 8 px") and lakes PARTLY at every level.

### 1.7 Frames of the current look

Captured windowed on this build (origin/main `823a8147`, `cargo build -p cartalith-godot` in a detached
worktree, `_riverzoom_probe.gd --rz-frames`, 1600×1000 window, floodplain on, the view on the map centre
at x1 and on the widest trunk above), under `C:/cw/.win_lock` with owner file `owner_realism`, in
`cartalith-native/docs/evidence/river-realism-2026-11-12/`:

| World | x1 | x4 | x16 | x64 |
|---|---|---|---|---|
| owner 2048×1311, 40 075 km, seed 246371, density 1.55 | `cur_owner_z01.png` | `cur_owner_z04.png` | `cur_owner_z16.png` | `cur_owner_z64.png` |
| seed 483920, 1024×656, 1 200 km (the probe's default width) | `cur_483920_z01.png` | `cur_483920_z04.png` | `cur_483920_z16.png` | `cur_483920_z64.png` |

What they show is written under the table in §1.8 after the capture, frame by frame, so that a claim
about the look is tied to a file that can be opened.

### 1.8 Frame notes

Read by eye from the eight frames above (each opened; the ppc per frame is in the logs beside them):

- **Owner x1 (0.464 ppc).** Rivers are hairlines, a few per province (Ruling EN's 5.5 %); a long, thin
  channel-shaped lake east of the central sea reads as a river-wide strip of lake colour; every lake is
  one flat dark tone whatever its size. Nothing to fault at this zoom except that the few rivers drawn
  are the same 1 px whether they carry a continent or a valley.
- **Owner x4 (1.86 ppc).** The network is dense and uniformly about 2 px; the hierarchy is carried by
  **colour only** (dark trunk, pale headwater), not by width; edges are hard; long axis-aligned runs
  still show in the north-east; the dark lake on the coast is a flat disc.
- **Owner x16 (7.42 ppc).** The §1.1 arithmetic on screen: every river, trunk or trickle, is **7–8 px
  wide** with a 1 px halo, so a pale order-1 stream is exactly as wide as the dark trunk it joins; the
  colour steps at each junction; tributaries meet the trunk at right angles in ladder form; no bank,
  no shading, no change at a steep reach. This is the frame the owner's "lines don't look very
  realistic" describes.
- **Owner x64 (29.7 ppc).** 30 px opaque strips of flat colour over the relief; the display meander's
  soft bends are visible; the order palette changes abruptly where a tributary enters; a headwater
  (top left) is the same width as the confluence trunk. Zoom gained width and nothing else.
- **483920 x1 (1 200 km, 0.93 ppc).** With `width_k` 0.67 the widths vary a little (1 to about 3 cells
  by order), the sea-level trunk in the desert is visibly wider than the mountain streams; rivers in
  the desert and in the forest are the same blue; the glacier-fed northern streams start at full
  weight at the snow line.
- **483920 x4 (3.7 ppc).** A trunk at about 3 px against tributaries at 2 px, pale headwaters with a
  halo, a jagged dark lake polygon at the coast; the desert rivers are as blue and as opaque as the
  mountain ones.
- **483920 x16 (14.8 ppc).** Trunk 10 px dark, tributary 8 px pale, both opaque with hard edges; the
  coastal lake is an angular dark blob; the pale tributary is nearly as wide as the dark trunk.
- **483920 x64 (59 ppc).** Both channels about 45 px, flat fills, the colour switching at the junction
  from pale to dark along a short blend; no bank, no bed, no white water anywhere, and the river bears
  no relation to the relief it crosses.

In short: at x16 and x64 the drawing shows one width and one surface for every river, and the eye reads
the volume hierarchy only from the Strahler palette, which RZ-0 showed is not the volume. RR-1 and RR-2
are the answer to these eight frames; RR-4 to what none of them has.

---

## 2. Research

### 2.1 Hydraulic geometry: width against discharge

- **Leopold & Maddock 1953** (USGS Professional Paper 252): width, depth and velocity are power functions
  of discharge, `w = a·Q^b`, with **downstream** exponents `b = 0.5`, `f = 0.40` (depth), `m = 0.1`
  (velocity) at a constant discharge frequency; `b = 0.5` equals Lacey's 1930 regime value. CONFIRMED
  (USGS publication page and the Cambridge handbook chapter summary read this session). Later work finds
  `b` varies: a Mississippi-basin satellite fit gives `w = 13.4·Q^0.46` (r² 0.64, Q in m³/s, w in m),
  sub-basins 0.3 (Missouri) to 0.63 (Upper Mississippi), "close to the range of values calculated for
  world rivers by Moody and Troutman (2002)" — CONFIRMED, read from the HESS 2014 PDF (Allen & Pavelsky
  group, hess-18-4883-2014). Moody & Troutman 2002's own global coefficient (often quoted as
  `w = 7.2·Q^0.5`): UNCONFIRMED here (the paper was not opened).
- **What it means on this engine.** `flow_discharge` is a rain-weighted cell count. To translate: cell
  area × mean runoff. On the owner world a cell is 19.6 km (383 km²); at an Earth-like 800 mm/yr and a
  runoff ratio 0.35 (both SECONDARY, textbook) one cell-unit is about 3.4 m³/s, so the owner world's
  largest outlet (1.5 % of 2.68 M cells, `COMB_DISCHARGE_FRAC`'s doc) carries about 1.4×10⁵ m³/s (the
  Amazon's order, SECONDARY) and `13.4·Q^0.46` gives **3.1 km: 0.16 cells, 1.2 px at x16, 4.6 px at
  x64**. A headwater at the threshold (67 units, 230 m³/s) is 165 m wide, 0.008 cells. So a true-scale
  width is sub-pixel for the whole network until x16 and for all but the trunks at x64, and today's
  1-cell floor draws the Amazon **six times too wide** at x16 and 120× too wide for a headwater. On an
  800 km map (0.39 km cells) the same arithmetic puts a 10 000 m³/s river at 0.92 km, 2.4 cells, which
  today's law (order 6: `0.6 + 3·mag² + 2.25 ≈ 3.5` cells half-width, 7 cells full) also overdraws.
- **The delta rule is already the law.** `w_a² + w_b² = w_p²` at every split (`river_delta.rs`, bar B5)
  is `w ∝ Q^0.5` with discharge conserved. A `Q^0.5` trunk therefore hands its fans a consistent width
  without a change there.
- **Depth** `∝ Q^0.4` is the quantity that darkens water (§2.2), so one discharge reading can drive both
  width and tone.

### 2.2 How realistic and cartographic renderers draw rivers subtly

- **swisstopo's vector style** (the public style JSON for its vector tiles, CONFIRMED read): rivers and
  streams are one `water_line` layer in `rgba(138,195,229)` at **opacity 0.7**, widths by a per-feature
  width class and zoom (1 px at z7; at z13 3.5 px for the widest class down to 1 px; at z16 5 → 1; at
  z20 10 → 4), intermittent streams **dashed and grey at 0.25–0.35 opacity**, and water areas a separate
  fill `rgb(199,224,245)` with a 0.5 px outline that fades out above z14. Lessons: translucent lines
  that let the land show through, a width *class* ladder (not one width), no outline at close zoom, and
  the large river as an area.
- **OSM Carto, OpenMapTiles, Mapbox** (fetched for `RIVER_ZOOM_VISIBILITY_RESEARCH.md` §1.1, not
  re-fetched): class-gated appearance, width growing about 1.3–1.5× per zoom level, and the big river
  as a polygon with the line "biased toward the smaller end".
- **Satellite colour** (USGS remote-sensing page, CONFIRMED): clear water, sediment-laden water (often
  brown) and algae-rich water (often green) each look different; strong red reflectance means sediment.
  A 234 727-image study of US rivers found **56 % sediment-yellow and 38 % algal-green** (Scientific
  American summary, CONFIRMED as a summary); the Forel-Ule scale runs dark blue (clear) to yellow-brown
  (turbid) and is used as a turbidity index (SECONDARY). Mountain rivers over bedrock and gravel are clear
  and read dark blue-green; lowland and arid-country rivers are turbid and read olive to tan. The Nile
  "supplies a narrow strip of land" of dark green against tan desert (NASA Earth Observatory,
  CONFIRMED): the riparian strip is a **desert** signal; in humid green country it is invisible.
- **Terrain tools.** Gaea's Rivers node (CONFIRMED, docs page): headwater count, water amount ("higher
  values can create larger rivers"), width, depth, downcutting, and a separate water surface; width
  follows the water budget. World Creator's river page could not be read (redirect to an empty page).
  Azgaar's FMG: flux-based rivers drawn as tapering polygons is UNCONFIRMED (the module could not be
  fetched); Martin O'Leary's notes that it credits trace rivers only above a flux threshold (SECONDARY).
- **Relief cartography.** Imhof's books are the standard Swiss reference (SECONDARY, cited by Jenny et al.
  2014); no numeric river rule was found and none is claimed.
- **Principles the sources agree on**, in the order they matter here: (1) width is a function of volume
  (`Q^0.5`), shown as a *ladder* of a few classes at a given zoom; (2) small water is translucent and
  takes the ground's tint, big water is opaque, dark and saturated; (3) no drawn outline on painterly
  looks, a thin ink line only on inked ones (what the presets already do, RIM-2); (4) the bank is soft:
  a wet strip a shade darker than the land, then the land; (5) the riparian green belongs to dry
  country; (6) at large width the river becomes an area with the shore's own edge (RIM-1 already does
  this). Specular highlights, foam and wave texture are x64 features and come last.

### 2.3 Channel heads: where rivers start

- **Montgomery & Dietrich 1988** (*Nature* 336, "Where do channels begin?"): the source area above a
  channel head falls as local slope rises, over slopes of about 5–45°, matching a landslide threshold on
  steep humid ground and not an overland-flow prediction on gentle valleys (CONFIRMED from the DOI
  landing page summary). The threshold form is `A·S^n = c`; Jefferson & McGee 2013 (*ESPL* 38, PDF
  opened, CONFIRMED) tabulate exponents 1.7 (southern Sierra Nevada, `S = 0.35·A^−0.6`, lower limit, M&D
  1988), 1.9 (Samoeng, `A = 30975·S^−1.9`), 2.0 (Kanozan, `A = 170·S^−2.0`), and state that "Montgomery
  and Dietrich (1994) show that the exponent value of 2 derives from laminar infiltration-excess
  (Hortonian) overland flow and low gradient areas with saturation overland flow"; `c` depends on
  climate, soils and vegetation. Montgomery & Foufoula-Georgiou 1993: the slope-dependent form
  represents valley development better than channel initiation (SECONDARY).
- **Springs and snowmelt.** Contact springs where a permeable unit sits on an impermeable one are what
  GF-5's spring line models (its own doc); snow- and glacier-fed heads start at the snow line in a
  regime where discharge peaks in early summer: UNCONFIRMED here (no source opened), and the engine has
  no snow-line hydrology to read anyway (LOD-D4's snow and `build_glacier_potential` are render fields).
- **What it means here.** The engine's `channel_threshold` is `A`-only at density 1 and `A·(1+8S)^−0.44`
  at the owner's 1.55: the right shape, an uncalibrated exponent, and gated on an unrelated control. A
  calibrated `A·S² = c` moves the network (generation; a ruling). Drawing-side, the one thing to fix is
  that a head appears at full weight: real channels begin as a swale, and the map should let them begin
  faint.

### 2.4 Lakes: when a basin holds water

- **Closed basins.** Langbein 1961 (USGS PP 412, *Salinity and hydrology of closed lakes*, 20 pp.):
  closed lakes adjust to inflow mainly through surface area; a later citing source states the water
  balance "inflow = lake area × net evaporation (evaporation minus precipitation)" (SECONDARY; the PDF
  was not opened). IPCC TAR WG2 §4.3.7 (CONFIRMED summary): endorheic lakes "are very dependent on the
  balance of inflows and evaporation", "under some climatic conditions, lakes may disappear entirely",
  and such systems "include significant internal thresholds". Salar del Huasco: precipitation about
  145 mm/yr against evaporation 230 mm/yr, a terminal salt flat (HESS 2026, CONFIRMED abstract).
- **Fill-Spill-Merge** (Barnes, Callaghan & Wickert 2021, *ESurf* 9, CONFIRMED abstract): runoff fills
  depressions which overflow into their neighbours, O(N log N), **no evaporation or infiltration term**:
  it is a fills-to-spill model, as this engine is today.
- **The rule for a generator** (`LAKE_FILL_RESEARCH.md` Part 1(b), carried): a basin is a lake when
  inflow (routed runoff plus rain on the pool) exceeds evaporation over the pool's area at the spill
  level; otherwise the level falls until the two balance (`level = min(spill, balance)`), and when the
  balance height is at the pit floor the basin is a playa. A spilling lake feeds an outlet river;
  an endorheic one has none, and is saline.
- **What this tree already learned** (`lake_candidates_harness.rs`, candidate 2, read in its module doc
  and in `LAKE_FILL_RESEARCH.md`): a balance with every constant a judgement "drains or shrinks large
  lakes, cuts a lake that feeds a river below its spill, and credits a lake beside a river with no
  inflow". So the bar for any balance is: no lake that today feeds a drawn river may lose its outlet, and
  a basin's inflow must be the routed discharge at its cells, not a guess.

### 2.5 Waterfalls, rapids and how to find them

- **Formation** (Wikipedia "Waterfall", CONFIRMED read, citing National Geographic and Carreck 1982):
  the common case is "a river courses over a top layer of resistant bedrock before falling onto softer
  rock" (caprock; undercutting, collapse, plunge pool); "a large step in the rocks that may have been
  formed by a fault line"; "along the edge of a glacial trough" (hanging valleys, Yosemite); falls
  recede upstream at up to about 1.5 m/yr. Types: ledge, plunge, horsetail, cascade, punchbowl, tiered,
  cataract, segmented, ephemeral (after rain or snowmelt). Coastal cliff falls: not covered by the
  source, UNCONFIRMED as a category.
- **Rapids are a slope class.** Montgomery & Buffington 1997's reach types by bed gradient: **cascade
  > 0.065, step-pool 0.03–0.065, plane-bed 0.015–0.03, pool-riffle < 0.015** (CONFIRMED, quoted from
  Wohl & Merritt 2005, *WRR* 41, W08419, which also warns that other regions show "greater variability
  in gradient range for each channel type"). Cascade and step-pool reaches are what the eye calls
  rapids and white water.
- **Detection from a DEM**: local gradient and curvature along the profile (Gonga-Saholiariliva et al.
  2011, about 80 % field-confirmed on SRTM/ASTER, SECONDARY); χ-space segmentation (Gailleton et al.
  2019, *ESurf* 7, SECONDARY); profile breakpoint tools (Knickpoint Finder, SECONDARY); and in all of
  them a **knickpoint fixed to a rock-strength contact** is distinguished from one migrating upstream.
  The engine's `knickpoints` is the fixed kind; a gradient break along the traced run (a step much
  larger than the run's local mean step) is the migrating kind and needs no lithology.
- **What it means here.** Every ingredient is on hand per traced cell: `field` for the drop per step,
  `dh_m` for metres, `flow` for volume, the column's `s` for the contact, `boundary_type`/`shear_field`
  for faults, `shore_depth` for the coast. A **rapids** reach is a run of cells with gradient above
  0.03 (step-pool) or 0.065 (cascade); a **waterfall** is a single step meeting `pool_waterfall`'s own
  constraints (reuse them, never a second copy), ranked higher on a knickpoint, a fault contact or a
  tributary's last step into a much deeper trunk valley (the hanging-valley signature: a tributary
  whose last step exceeds `k` × its own median step). At 19.6 km a cell the owner world's "steps" are
  reaches, not lips, which is what `WATERFALL_MIN_GRADIENT`'s doc already says; the drawn mark is a
  symbol of a reach that holds a fall, as a map's is.

---

## 3. Design

Each proposal says what it changes, whether it is render-only or moves generated output
(`DECISIONS.md` §7p: a generation change ships `false` in `cartalith_engine::WorldParams::defaults()`,
`true` in `cartalith_godot::params::defaults()`, with `PARAMS` and `JS_PATHS` rows, and a change that
moves the network or the water classification needs an owner ruling and a disclosed re-baseline), its
bar at x1, x4, x16 and x64 on the owner world (fit 0.46 px per cell) and on the probe's 1024×656 worlds
(the owner's hard rule: zoom gains detail at every level), and its cost. Bars are measured by
`_riverzoom_probe.gd` legs (windowed; `ImageTexture.update()` is a no-op headless) and by `#[ignore]`d
Rust harnesses beside `river_zoom_measurement.rs`; every constant below is a **labelled judgement** until
RR-0 measures it, and the measurement, not this text, pins it.

### RR-1. Width from discharge (render only)

- **Change.** `river_half_width_profile` reads a new `river_stroke::hydraulic_half_width(Q, Q_max,
  scale)` instead of `channel_disc`'s bracket: `hw_cells = W_MAX_CELLS · sqrt(Q / Q_max)`, `Q` the
  run's running-maximum discharge (the running maximum already there), `Q_max` the largest drawn
  discharge of the world (one pass; `visibility_ranks` already sorts these), `W_MAX_CELLS` the chosen
  width of the biggest river (Q1). `slope_fac` is dropped (a bedrock river *is* narrower, Montgomery &
  Gran 2001 per `GEOLOGY_FIRST_SCOPE.md` §4.7, but `channel_disc`'s factor is a grid-slope guess; GF-6's
  `ω` is the right carrier if the owner wants it, behind `geology_carve`). The 1 px floor
  (`MIN_STROKE_PX`), `EDGE_FRINGE_PX`, the visibility law and the seam are unchanged; `channel_disc`
  stays for the intensity stamp (`stamp_river_intensity`, reference parity) and `river_entities`'
  `half_width_cells` reports the new value with its law named. Default `W_MAX_CELLS = 1.0`: the trunk
  is as wide as today's floor, so x1 and x4 move by at most the headwater de-emphasis, and every lesser
  river narrows toward the 1 px floor. A true-scale alternative, `W_MAX_CELLS = 13.4·Q_max_m3s^0.46 /
  cell_m` with the §2.1 conversion (0.16 cells on the owner world), is Q1's other option. A preset's
  `river_width` still multiplies.
- **Interactions.** The floodplain band (`FLOODPLAIN_PER_HALF_WIDTH`) and RIM-2's bank follow the half-width
  and narrow with it. The field's `G` channel carries the new half-width unchanged in meaning. Fans
  split `w²` from the new trunk width (no change in `river_delta.rs`). The comb thinning
  (`river_draw_plan`) is unchanged. `reach_limit_cells` bounds the same widest stroke.
- **Bars.** RR-0 prints, per world, the distribution of `Q/Q_max` over drawn length and the resulting
  widths in px at the four zooms. Then: (a) at **x16** on the owner world the drawn width histogram
  (ON-vs-OFF diff mask, run-length across the stroke) has at least **three** distinct classes (≥ 1 px
  apart) where today it has one, and the widest trunk is at least **4×** the 1 px floor; at **x64** at
  least **five** classes, trunk ≥ 12×; at **x4** at least two classes on the 1024×656 worlds (fit
  0.93 ppc, where `hw·ppc` clears the floor); at **x1** the frame's ink pixel count is within **±15 %**
  of today's (the floor rules there; this bar is "x1 did not get sparser", because Ruling EN's share is
  not this change's to move). (b) Width never decreases downstream on any run (running maximum, the
  existing `_stroke_stats` check). (c) World hash and every `golden_parity_*` unchanged; `river_at`,
  the planner, saves untouched. (d) Mutants of `W_MAX_CELLS` and the exponent 0.5 each turn a literal
  test red.
- **Cost.** Small: one function, two call sites, the probe leg, tests. Half a lane-day plus the windowed
  runs.

### RR-2. Colour by volume and climate, soft banks, a climate-gated floodplain (render only)

- **Change, three parts, each its own switch in `TerrainAppearance` so a look can decline it.**
  1. **Tone by volume.** The per-point colour interpolates the preset-styled palette by the **discharge
     rank** `u` (already in `DrawnRun::ranks`) instead of Strahler order: `palette_at(RIVER_ORDER_RGB,
     7·(1 − u)^γ)`, `γ` a judgement near 1.5 so that only the top of the network is dark. Alpha for small
     water: `river_opacity · (0.65 + 0.35·(1 − u))` — a headwater is 65 % opaque and takes the land's
     tint; a trunk is opaque. Depth `∝ Q^0.4` is the physical reason; the palette is the existing one.
  2. **Turbidity by climate and slope.** A per-point turbidity `t = clamp(a·(1 − slope_norm) + b·(1 −
     vegetation) , 0, 1)` from the cell's grid slope and the biome's green-ness (`material_weights`'
     forest/grass share, already computed per cell by `land_color`), pulling the colour toward the look's
     `river_silt` tint (default olive-tan `(118, 112, 78)`, a judgement from the Forel-Ule scale's warm
     end) by at most `TURBID_MAX` 0.35. Mountain and forest rivers stay clear and dark; lowland and
     desert rivers go warm and pale. One number per point, stored in the field's colour map
     (`river_color_texture` is already "the map with every river at full coverage"; the tint goes into
     it at build), so the shader is untouched by this part.
  3. **Soft banks and the riparian strip.** A wet-bank darkening of the **land** one pixel outside the
     edge, `BANK_WET` 0.12 luminance, on every preset (the ink line of RIM-2 stays preset-gated above
     it); the floodplain lift multiplied by `(1 − green)·aridity_weight`, so it is strongest where the
     land is tan and vanishes where it is already green (the Nile rule), and its band grows with the
     half-width as now. The fringe stays 1 px (the RV-2 finding stands: a wider fringe reads washed).
- **Bars.** At **x16** on the owner world, the mean colour of the widest trunk's pixels and of order-1
  headwater pixels differ by **at least 18 and at most 70 summed RGB levels** (today: read by the probe
  at RR-0; the palette's ends are 156 vs 22 in R alone, so today's spread is wider than realism wants);
  a desert river (sampled where `material_weights` is desert-dominant) and a forest river of equal rank
  differ by **at least 10 levels** in hue; at **x4** and **x1** the frames' mean river colour moves by
  under **8 levels** (subtle at the opening view: the owner's "more subtle"); at **x64** the bank strip is
  measurable as a 1 px darker ring in at least 80 % of the trunk's edge pixels (ON/OFF diff), and the
  floodplain weight over green land is **0** (pixel-identical with the lift off) while over desert it is
  the lift. Night and the inked presets keep their ink (`river_style_color` runs after). No golden moves.
- **Cost.** Medium: the colour map build, three tunables with GUI rows (`tunables!`, Render workspace),
  a probe leg sampling by biome, before/after sheets per preset. One to two lane-days.

### RR-3. Heads that begin faint (render only)

- **Change.** The first `HEAD_FADE_CELLS` (4, a judgement: one meander wavelength) of a run whose head
  meets no water and no trunk take an alpha ramp 0 → 1 along the arc, in the seam
  (`river_px_width`'s alpha) and the shader (a per-point head distance interpolated into the field would
  need a channel; instead the ramp is baked into the colour map's alpha, which the field's `A` already
  multiplies; RR-0 decides which). Width stays at the floor: a width taper below 1 px is the dotted line
  RV-2 removed, and an alpha fade of a full-weight line "reads as a lighter line" (the RZ research's own
  finding). Applies from **x4** (`ppc ≥ 1`); at x1 a head is already a sub-pixel dot and the visibility
  law hides most of them.
- **Bars.** At x16 and x64, no drawn run begins with a pixel above 50 % of its own full alpha (probe:
  sample the first 2 px of every visible head); at x1 the frame is pixel-identical with the ramp off;
  springs placed by `pool_spring` lie on a drawn head (count); hash and goldens unchanged.
- **Cost.** Small.

### RR-4. Rapids and waterfalls, drawn on the river and tied to the landmark (render only, one question)

- **Change.** A new `river_stroke::reach_classes(run, field, dh_m)` labels each drawn point by the traced
  gradient of the step it lies on: `RAPIDS` where gradient ≥ `RAPIDS_SLOPE` 0.03 (step-pool, §2.5) and
  `CASCADE` ≥ 0.065, each for runs of at least `REACH_MIN_CELLS` 2 so a single noisy step is not white
  water; `FALL` where `pool_waterfall`'s constraints hold at a step (drop ≥ 40 m, gradient ≥ 0.06, flow
  ≥ 2T — read from `cartalith_civ::landmark`'s constants through one shared function, never a second
  copy), and where available raised by the knickpoint, a fault contact (`boundary_type` at the cell) or
  the hanging-valley signature (a tributary's last step ≥ 3× its median step into a trunk). Drawing,
  from **x16** (below it a reach is under 2 px long): rapids lighten the water along the reach with a
  streak noise aligned to the flow (`WHITEWATER` 0.25 lift, strength ∝ gradient), cascades more; a fall
  draws a 1-cell-long bright lip across the channel, a 0.5-cell darker plunge pool below it and, on the
  shaded relief, nothing (the carve already steps there). The class is stored per point in the
  `DrawnRun` and reaches the field as a third use of the colour map (lightened texels), so tiles and
  export follow without a shader change; the badge of the landmark stays at every zoom; the context
  card's river row lists "rapids N · falls M".
- **The one question (Q3).** `knickpoints` is derived on demand from the column and marks no world
  field, but `RockView::for_world` refuses it without `geology_processes`, on the argument that without
  the processes "the erosion never cut weak rock back faster than strong, so no step at a contact is
  claimed". For a *drawn* fall that argument is weaker than for a landmark: the contact is still where a
  fall would be in that geology. Proposed default: compute knickpoints for the drawing on any world
  with a column, and leave the landmark's gate as it is unless the owner says otherwise. Turning
  `geology_processes` on is a generation change outside this scope.
- **Bars.** RR-0 counts candidates per world (falls, rapids reaches, cascade reaches, by cause). Then:
  every `pool_waterfall` placement lies on a `FALL` point (100 %, by construction of the shared
  constraints); no `FALL` or `RAPIDS` on a lake or sea cell; on the owner world at x16 and x64 a fall
  is a visible mark (ON/OFF diff ≥ 24 levels over at least 4 px) at every placed waterfall in frame; at
  x1 and x4 frames are pixel-identical with the classes off (nothing drawn below x16); rapids reaches
  cover between **1 % and 8 %** of drawn length on the three probe worlds (a judgement bracket: more
  than that is a world of white water, less is invisible; RR-0 sets it from the gradient distribution);
  goldens and the landmark identity hashes unchanged with Q3's default off, the landmark identity moving
  "by design" if Q3 opens the knickpoints to it.
- **Cost.** Medium: the classifier and its tests, the colour-map lightening, the lip/pool mark, the
  context-card row, a probe leg framing placed waterfalls. Two lane-days.

### RR-5. Lakes with depth and shallows (render only)

- **Change.** `lake_color` takes `depth = clamp((fill_level − field) / LAKE_DEPTH_FULL, 0, 1)` per cell
  (`WaterBodies.fill_level` exists per cell) in place of the constant 0.30, so a deep basin is dark and a
  shelf is pale; near the shore `shore_depth`'s distance gives a one-cell shallows lift, the lake's
  analogue of Ruling CI's shelf. Tiles (`TILE_LAKE`, `LAKE_MEMBERSHIP_MIN`) and export (`bake_lake_at`)
  take the same function. Crater lakes keep `crater_lake_tint`.
- **Bars.** At every zoom a lake deeper than 2·`LAKE_DEPTH` shows at least two distinguishable tones
  (≥ 12 levels between its deepest and its shore ring) and a 1–4-cell pond stays one tone within 4
  levels of today's; sea pixels pixel-identical; no classification changes (`build_water_bodies`
  untouched); goldens unchanged.
- **Cost.** Small.

### RR-6. Lake water balance (generation: a ruling, measured first)

**Built 2026-11-12 under Ruling ET** (owner: *"In regards to the lakes, make it the new default on setting"*), as `cartalith_civ::lake_balance` behind `lake_balance`; the playa is land under the climate biome and the outlet cut is not skipped. `STATUS.md` carries the measured result; the design below is the spec it was built from.

- **Change.** Per basin (`basin`, `basin_outlet` exist), inflow `I` = routed discharge summed over the
  basin's cells at the spill level plus rain on the pool; evaporation `E` = `e(T)` × pool area, with
  `e(T)` from the cell temperature (a Thornthwaite-shaped curve, SECONDARY, to be cited at build) scaled
  to the rain field's units; level `= min(spill, h: I = E(area(h)))` by a descending walk down the
  basin's hypsometry (sorted cell heights, already a by-product of the flood); below the floor plus
  `LAKE_DEPTH` the basin is a **playa** class (drawn as a salt flat; `water_body_topology` already names
  `EndoreicLake`, and the trade module's salt-lake check reads it). A lake that does not spill has no
  outlet river (`carve_channel_network`'s outlet cut skipped; the routed network still crosses the
  basin, so the planner and ways are unchanged in kind).
- **§7p.** Off in `WorldParams::defaults` (every golden unchanged), on in `params::defaults`, a
  `PARAMS`/`JS_PATHS` row, old saves load it off. It moves the water classification on app worlds, so it
  needs **an owner ruling** before any default flips (Q4), and `lake_candidates_harness.rs`'s finding is
  its first bar.
- **Bars** (the harness, 37 worlds + owner): no lake that today has a drawn outlet river loses it unless
  its basin's `I < E` at the spill (the count of such lakes printed and reviewed, not hidden); no
  below-sea lake moves; the number of above-sea lakes falls by **≤ 30 %** on the humid worlds and the
  playas appear only where the cell rain is under the median (a judgement bracket); every lake still
  passes Ruling CC's floor; settlements within 3 cells of a changed lake re-sited by the existing path
  (as VA-6 measured); `cargo test --workspace` green with the switch off.
- **Cost.** Large: the balance, the playa class through the civ, drawn, tile, export and atlas
  classifications (the VA-6 maar lake threading is the precedent), the harness, the ruling. Three to
  five lane-days.

### RR-7. Channel heads by area and slope (generation: a ruling, measured first)

- **Change.** `channel_threshold` takes `A · S^n ≥ c` with `n = 2` (M&D 1994's derivation, CONFIRMED
  via Jefferson & McGee) and `c` calibrated so the drawn network length at density 1 matches today's
  on the 37 harness worlds (so the owner's density feel is kept and only *where* heads sit moves: up the
  steep slopes, back from the flats), decoupled from `river_density`'s exponent trick. Behind a switch,
  §7p as RR-6. It moves the network, which Ruling EN item 5 reserves and Ruling ER overrode once, for
  rough routing only: **a ruling** (Q5). Rough routing's density change (traced −27 %, drawn +14 % on the
  owner world) is still open with the owner, so this should wait for that answer.
- **Bars.** Drawn length within ±10 % of today's per world at density 1; heads' median slope rises
  (printed before/after); no dry ends, no uphill steps (the routing harness's plausibility checks); the
  planner's `fresh_river_channels` count within ±10 %.
- **Cost.** Medium, after the ruling.

### RR-8. Meander wavelength scaled to the channel (render only)

- **Change.** `MEANDER_WAVELENGTH_CELLS` becomes `max(4, k_λ · 2·hw_cells)` with `k_λ` 9 (Leopold &
  Wolman's 7–11 widths, UNCONFIRMED here) and the amplitude capped as now (inside the traced cells), so a
  wide trunk wanders in long bends and a trickle in short ones, instead of every run at 4 cells.
- **Bars.** At x64 the trunk's drawn straight share (`straight_stats`) does not rise; no point leaves its
  traced cells (existing bar); at x1/x4 pixel-identical.
- **Cost.** Small; depends on RR-1's width.

### Not proposed

- A **physical water surface** (specular, reflection, waves) on the 2D map: a photoreal cue that the
  target does not show and the presets would each need to answer; the 3D viewport (`3D_VIEWPORT_SCOPE.md`)
  is where a water material belongs.
- **Braided channels and estuary funnels**: Ruling EN item 4 ("no estuary or braided-channel model")
  stands.
- **River labels**: out of scope (Ruling EN).

---

## 4. Milestones

- **RR-0 — measure before pinning (harness and probe, no app code).** `tests/river_realism_measurement.rs`
  (`#[ignore]`, release, alone, median with min..max) on the 37 RZ-0 worlds plus the owner world: the
  `Q/Q_max` distribution over drawn length and the implied `Q^0.5` widths in cells and in px at x1/x4/x16/x64;
  today's width classes (expected: one, on 40 075 km shapes); the §2.1 km conversion per world; the
  gradient distribution along traced runs and the counts of rapids, cascade and fall candidates by cause
  (knickpoint, fault, hanging valley, none); per basin `I` and the area at spill (for RR-6's bracket);
  lake depth distributions. Probe leg `--realism` capturing the colour samples RR-2's bars read (trunk
  vs headwater, desert vs forest). **Pass:** non-empty tables (silently-empty output is the known
  failure), and each proposal's constant above either confirmed or moved with the table beside it.
- **RR-1 — width** (bars in §3). Frame pairs at the four zooms on the three worlds for the owner.
- **RR-2 — colour, turbidity, banks, floodplain**, per preset sheets.
- **RR-3 — heads.**
- **RR-4 — rapids and falls**, after Q3.
- **RR-5 — lake tone.**
- **RR-8 — meander scaling** (small; folds into RR-1's lane).
- **RR-6 — lake balance** and **RR-7 — channel heads**: harness and ruling first; built only on the
  owner's answer.
- **RR-9 — close-out.** Screen, tile and export agree at equal `ppc` for every new term (RIM-7's mirror
  tests extended); pan-frame time with rivers on within the pre-change spread at the probe's zooms and
  under Ruling BP's 16.7 ms where today's passes; `cargo test --workspace`; parse-check of every `.gd`
  touched plus `shell/app.gd`; `STATUS.md` in the same change; the superseded order palette and
  `channel_disc`'s drawing use scrubbed, not kept beside the new (§7p).

---

## 5. Risks

- **The concurrent lane.** `river_stroke.rs`, `river_delta.rs`, `river_field.rs` and the routing are
  being edited under Ruling ER; RR-1 to RR-4 all touch `river_stroke.rs`. Schedule them after that lane
  lands, and re-open every symbol named here first (the backlog-row rule in `MISTAKES.md`).
- **The width floor hides the width law at x1 and x4** on whole-world maps: RR-1's visible gain is at
  x16 and x64 by arithmetic. If the owner expects a visible hierarchy at the opening view, the only
  lever is `W_MAX_CELLS` above 1 (wider trunks than today), which Q1 asks.
- **Colour bars are palette-bound** (`MISTAKES.md`): every pixel bar above names the preset it is
  measured on (Natural Vibrant / Default) and the probe must force it.
- **Field channels are full** (RGBA, `MAX_TEXELS`): RR-2's turbidity, RR-3's head fade and RR-4's classes
  all go through the colour map rather than a fifth channel; if any needs per-pixel geometry the shader
  cannot derive, it costs a second texture (about 100 MB at the cap) — RR-0 decides.
- **Rapids on a coarse world are reaches, not features**; drawn below x16 they would read as noise. The
  x16 gate is a bar, not a tuning.
- **A lake balance can drain a lake the owner likes.** RR-6's "printed and reviewed" list exists so that
  no lake disappears silently; the switch ships off until the ruling.
- **The 1 px floor and alpha**: RR-2's translucent headwaters at 65 % alpha over the floor could re-open
  the "washed band" the RV-2 fringe fix removed. The x1/x4 "under 8 levels" bar is there to catch it.

---

## 6. Owner questions, with recommended defaults

1. **How wide is the biggest river?** Default: **symbolic, `W_MAX_CELLS = 1.0`** (the trunk as wide as
   today, everything smaller narrower, hierarchy visible from x16). Alternative: true scale (0.16 cells
   on the owner world: sub-pixel until x64, the most honest and the least visible), or a larger symbolic
   value (2.0: trunks twice today's at every zoom). A per-look `river_width` still multiplies whichever.
2. **Colour by volume and climate, subtle (RR-2)?** Default **yes**, at the bars' "subtle at x1/x4"
   bracket; a preset may opt out. Alternative: tone by volume only, no turbidity.
3. **May the drawing read knickpoints on any world with a geology column** (processes off), leaving the
   landmark's gate as it is? Default **yes** for the drawing, **no change** for the landmark.
4. **Lake water balance (RR-6): build it behind a switch and measure, or not now?** Default: **build
   the harness and show the owner the list of lakes that would change** before any switch flips;
   nothing moves until a ruling.
5. **Channel heads by area and slope (RR-7)?** Default: **wait** for the owner's view on rough routing's
   density change, then decide; not scheduled by this scope.
6. **Heads fade in from x4 (RR-3)?** Default **yes**.

---

## Sources

Opened this session (CONFIRMED): USGS, *The hydraulic geometry of stream channels and some physiographic
implications* (PP 252) publication page <https://pubs.usgs.gov/publication/pp252> and the Cambridge
handbook chapter summary; HESS 18, 4883 (2014) PDF <https://hess.copernicus.org/articles/18/4883/2014/hess-18-4883-2014.pdf>
(`w = 13.4·Q^0.46`, Moody & Troutman cited); Jefferson & McGee 2013, *ESPL* 38, 601–613 PDF
<https://redlair.charlotte.edu/wp-content/uploads/sites/937/2023/11/Jefferson_et_al-2013-Earth_Surface_Processes_and_Landforms.pdf>
(channel-head threshold table); Wohl & Merritt 2005, *WRR* 41, W08419 PDF
<https://wpg.forestry.oregonstate.edu/sites/default/files/seminars/2005_Wohl%26Merritt.pdf> (Montgomery &
Buffington slope ranges); Montgomery & Dietrich 1988 DOI page <https://www.doi.org/10.1038/336232A0>;
Barnes, Callaghan & Wickert 2021 <https://esurf.copernicus.org/articles/9/105/2021/>; Wikipedia
"Waterfall" <https://en.wikipedia.org/wiki/Waterfall>; swisstopo vector style JSON
<https://gist.github.com/niccantieni/046e47b793ab58d766425e188a2b3e72>; Gaea Rivers node
<https://docs.quadspinner.com/Reference/Water/Rivers.html>; USGS remote sensing of water quality
<https://www.usgs.gov/mission-areas/water-resources/science/remote-sensing-water-quality>; IPCC TAR WG2
§4.3.7 <https://archive.ipcc.ch/ipccreports/tar/wg2/index.php?idp=170>; HESS 30, 433 (2026)
<https://hess.copernicus.org/articles/30/433/2026/>; NASA Earth Observatory Nile images (search summary
of the page text); Scientific American, "Data Show Shifting Colors of U.S. Rivers" (summary).
SECONDARY: Langbein 1961 USGS PP 412 <https://pubs.usgs.gov/publication/pp412>; Moody & Troutman 2002
*ESPL* 27; Montgomery & Buffington 1997 *GSA Bull.* 109 (not opened; quoted through Wohl & Merritt);
Gonga-Saholiariliva et al. 2011; Gailleton et al. 2019 *ESurf* 7; Montgomery & Foufoula-Georgiou 1993;
Imhof 1965/1982 via Jenny et al. 2014; Martin O'Leary's terrain notes. UNCONFIRMED: Moody & Troutman's
`7.2·Q^0.5`; Leopold & Wolman 1960's 7–11 widths per meander wavelength; Azgaar's FMG river width code;
Earth-like runoff ratio 0.35 and 800 mm/yr; the Amazon's discharge order; glacial and snowmelt channel
heads; coastal-cliff falls as a category.

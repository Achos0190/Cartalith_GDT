# LOD detail — scope

**Scale-dependent terrain detail toward the owner's Aletsch target.**

This document **defines** milestones LOD-D0 to LOD-D7. It does not track them:
status lives only in `cartalith-native/docs/STATUS.md`. It was planned on
2026-09-13 from the owner-supplied research note `docs/research/lod extra info.md`,
Ruling K in `LARGE_ITEM_RULINGS.md`, and a symbol-level map of what this workspace
already has.

The owner questions in the last section each carry the default the milestones
were written against; a ruling on any of them is recorded in
`LARGE_ITEM_RULINGS.md`, not here. **One ruling already moves this plan:**
Ruling AP (2026-09-23) authorised the main-map snow re-baseline that LOD-D4 could
not reach under the no-re-baseline rule — see *Rules* and LOD-D4. Checked
2026-09-24: `LARGE_ITEM_RULINGS.md` carries no ruling on any of the six owner
questions below, so each still stands at its default.

Where a milestone's premise was refuted when it was built, the milestone carries
a short **Found when built** note naming the symbol. Those notes are design
corrections, not progress; whether a milestone has landed is `STATUS.md`'s.

---

## Why this exists

**Three inputs, all from the owner.**

- **2026-09-12, the research note.** It covers a multiresolution pyramid, LOD chosen
  by physical rather than GUI scale, continuous transitions, macro/meso/micro
  normals, and snow, rock, glacier and hydrology derived from terrain. Its
  acceptance test is one continuous zoom with no popping.
- **The same day, the target image.** `design/owner-references-2026-09-12/lod-zoom-target-aletsch-sentinel2.jpg`,
  the Aletsch glacier seen from above.
- **Ruling K.** The target is read **top-down, with 3D still parked**. The order is
  ruled: **port `renderBiomeTileRGBA` so tiles carry colour, then draw ice from
  fields that already exist.** A glaciation model (ice extent, flow, moraines) was
  offered and not chosen.

**The measured defect the plan exists for**
([`OUTSTANDING_WORK.md`](OUTSTANDING_WORK.md), "DIAGNOSED: LOD tiles carry a SHADE
RATIO, not colour"):

- A live tile carries one scalar per pixel, the ratio of detailed to plain hillshade.
  `lod_tile.gdshader` multiplies the grid-resolution colour texture by it.
- **So a deeper level cannot show more.** Detail per screen pixel falls from
  **0.0220 to 0.0017** between zoom 1 and zoom 40.
- Hiding the whole LOD layer at zoom 16 moves mean |dL| by **0.0083**.

**The research note's crate names are not this workspace's,** and much of what it
proposes is already built. It maps as follows:

| Note | Here |
|---|---|
| `-grid` | `cartalith-spatial` |
| `-worldgen` | `cartalith-engine` plus the terrain, climate, erosion and hydrology crates |
| `-render` / `-cartograph` | `cartalith-godot/src/render.rs`, `cartalith-terrain/src/tile_render.rs`, `shell/lod_tile.gdshader` |
| `-compute` | `cartalith-gpu` plus Rayon |
| `-project` | `cartalith-io` |
| `-logistics` | `cartalith-civ` |

This document schedules only the gap.

---

## What already exists, and the gap

The table is the planning map's summary as of **2026-09-13** — the baseline the
milestones were written against, not a description of the tree today (several
of its "gap" cells are exactly what the milestones change). Its full version is
kept with the 2026-09-13 planning evidence. Symbols are named rather than
line-cited, because line numbers drift.

| Research asks | Already here | Gap |
|---|---|---|
| Detail from scale inside the existing map (§1, §7, §28) | `WorldGen::lod_synthesize_tile` reads the live height, seed and sea level; `_lod_layer` sits over `map_view` | LOD entry is still a threshold switch (`LOD_PX_PER_CELL_THRESHOLD`, `LOD_AUTO_ZOOM`) with a 0.15 s whole-layer fade |
| Multiresolution pyramid (§9) | `cartalith_spatial::pyramid` (`pyramid_dims`, `pyramid_tile_bounds`, `chunk_parent`/`chunk_children`); `cartalith_engine::bake::pyramid_tile` = `refine_tile` + `add_zoom_detail`; `TILE_PX = 256`, `MAX_LEVEL = 10`; the atlas in `cartalith-io/src/atlas.rs` | **Only height is pyramided.** It is synthesised upward (bilinear plus seeded detail), not filtered down, and there is no material, hydrology, snow or ice level |
| Screen-space LOD selection (§10) | `lod_bridge::level_for_zoom` → `pyramid_level_for_zoom`, from screen pixels per grid cell; `viewport_host.gd::_update_lod` | Levels are discrete (`js_round(log2 …)`); no device budget enters the choice |
| Continuous transitions (§11) | Shared tile edges, `edge_l/r/u/d` extrapolation, world-coordinate detail, a whole-layer fade | **Missing.** A level change frees the old tiles at once (`_apply_lod_tiles`), with no parent fallback and no morph |
| Macro/meso/micro normals (§12) | `TerrainAppearance::detail_{macro,meso,micro}_weight` (0.40/0.40/0.20) in `land_color` | Base map only, grid resolution, the same weights at every zoom; micro is noise, not relief; tiles use a single fixed sun |
| Multiscale shading (§13) | `multi_sun_from_normal`, `build_ao`, `build_crest`/`apply_crest`, atmosphere, local contrast | None of it reaches a tile |
| Material detail (§15) | `material_weights` (six fractions), `geo_exposure`, `rock_material_col`, lithology | **Tiles carry no material at all** |
| Snow (§16) | `snow_col` ramps; temperature carries the lapse rate (`cartalith_climate::compute_temperature`) | `material_weights` snow is `smoothstep(3, -5, t)`, in effect a temperature cutoff at grid resolution |
| Glaciers (§17) | `cartalith_erosion::glacial_kernel` carves troughs where `h ≥ sea + (1 − sea)·glacial_snowline` and `T < 0` | **No ice or glacier field exists anywhere**; the kernel keeps no mask |
| Hydrology at scale (§18) | `RiverInk`, `build_river_sdf`, `build_hydro_wetness`; fields `flow_discharge`, `channels`, `stream_order`, `river_mask`; `build_water_bodies` | No stream-order-by-scale gating. The reference's per-tile river SDF and lake shoreline are unported |
| Microdetail and residuals (§19–20) | `amplify::{refine_tile, add_zoom_detail}`, seeded, relief-tapered | Built, but it is the only thing a tile adds — the note's Phase 8 is live while its Phases 5–7 are absent at tile resolution |
| Cache and invalidation (§21) | Atlas `world_key`; save `source_key` + `producer`; `StageGraph` height versions | Live tiles are not dropped when a sculpt commits (its own backlog row) |
| GPU with CPU fallback (§22) | Tiles are CPU/Rayon; the GPU only composites | No GPU tile path, so the fallback question is moot |
| Android (§23–24) | `recommended_quality_tier` lowers Android one tier | Tile synthesis runs on the main thread; LOD budgets ignore the device; a 0.87 s zoom-notch stall on the 6T is recorded with its cause unfound |
| Acceptance: continuous zoom, no popping (§27, §30) | Unit test `deeper_levels_carry_strictly_finer_detail` (mask only); probes `_tiledlod_probe`, `_lodlevels_probe` | **No automated zoom test** |

**Loaded saves are thinner than generated worlds.** `cartalith_io::SaveFields`
carries height, temperature, rainfall, volcanic and impact fields and Strahler
order — no flow and no lithology — so every milestone states what it does on a
loaded save.

**The reference function to port.** `renderBiomeTileRGBA`, `reference/Cartalith
Gen1 v2.11.html` **lines 11668–11779** (the frozen snapshot, so these do not
drift), chosen by `_lodBuildTileRGBA` when `state.mode === 'biome'`.

- **Per tile** it builds:
  - the light vector and a meso step `ms = max(2, min(W,H)/64)`;
  - a tile-local AO box blur;
  - the crest field;
  - a coast SDF from the tile's own height;
  - a river SDF from world-sampled flow;
  - a biome-boundary distance;
  - the shared sea fields (the v1.29 seam fix).
- **Per pixel** it computes, in order:
  1. normal and macro shade;
  2. bilinear T and M;
  3. world-coordinate noise;
  4. ocean via `seaColorCore`, or lake via the `_lakeFill` shoreline;
  5. meso shade, slope, relative height and TWI;
  6. continuous aspect and curvature;
  7. AO;
  8. `landColorCore`;
  9. `applyCrest`;
  10. `applyCoastRiverSDFv`.
- **Most helpers already have Rust counterparts** in `render.rs` — `land_color`,
  `sea_color_core`, `material_weights`, `multi_sun_from_normal`,
  `build_crest`/`apply_crest`, `sea_shade_from`, `build_river_sdf`,
  `build_biome_boundary_dist`, `aspect_factor_f`, `curvature_at_f`. But they are
  indexed per grid cell and need tile-resolution entry points.

---

## Relation to the other LOD documents

**`LOD_TILING_BASE_SCOPE.md`** built `cartalith-spatial` standalone.
**`LOD_TILING_INTEGRATION_SCOPE.md`** owns tiers Z1–Z5 and milestones M0–M3; M1,
the interactive deep-zoom tiles, is what this document extends.

**Nothing here re-scopes Z3** (streaming the base raster), **Z4** (export) **or the
M3 atlas.** The tier definitions (Z1–Z5) live in that document and are not
restated here.

That document's *Out of scope* list excludes several things. This one touches
two of them, and both are owner questions:

- **"A GPU compute path for `build_color_texture`/`with_appearance`."** Not touched:
  every milestone stays on the CPU path. The only GPU element is a composite shader.
- **"Anything resembling … clipmaps."** LOD-D3 borrows the CDLOD *morph idea* in
  colour space on a top-down view. It builds no clipmap geometry and pages nothing
  — see question 5.

---

## Rules that apply to every milestone

- **Nothing moves an existing golden.** New stages are gated the way
  `TerrainAppearance` gated its milestones (`TERRAIN_APPEARANCE_SCOPE.md`,
  *Rules every milestone here holds*): inert under `js_reference()`, on in
  `default()`. `golden_parity_render.rs` stays unedited.
- **Parity class is stated per milestone.**
  - A CPU port of a reference function is golden-verified (`PARITY_TESTING.md`).
  - Behaviour new to this port is verified by **principled equivalence plus visual
    quality**, with its sources cited (owner decision on GPU and optimised paths).
    Per `DECISIONS.md` §7p, such behaviour is the standard on its own terms, not a
    divergence to be held against the reference's look.
- **A golden re-baseline needs an owner ruling.** None was planned in these seven
  milestones. **One has since been ruled:** Ruling AP (2026-09-23) authorises
  giving the **main map's** `material_weights` snow term an aspect
  (slope-direction) term, a visible change to every snowy mountain and not only
  to LOD tiles. It was built outside D0–D7 in `19c38d9`; see LOD-D4. The term
  is 0.0 under `js_reference()`, so `golden_parity_render.rs` and every
  JS-parity golden stayed unedited. What moved were the Rust render hashes
  (`tests/color_space.rs`'s `FINISHED_RENDER_FNV1A` and its siblings, and
  `tests/layer_stack.rs`).
- **No Rust panic crosses the gdext boundary.** Every new `#[func]` returns an
  `Option` or a result the shell can dash with a reason.
- **Android stays viable.** The generation peak on the phone (878–908 MB at
  2048×1311) must not move; new retained fields are built lazily, never inside
  `generate`.
- **Each milestone fits one lane** of at most three builders plus an adversarial
  verifier, and every probe fails when its feature is absent.

---

## Milestones

### LOD-D0 · Zoom-sweep harness — measurement only, no pixel change

**Goal:** the no-popping bar and the Aletsch comparison become numbers before
anything is built, with today's figures recorded as the baseline.

**Relation:** new. It does not reorder Ruling K, because it changes no output.

**Scope.** A windowed probe (`_lodsweep_probe.gd`) that:

- loads fixed seeds (at least three) at 2048×1311 and 512×384, with overlays hidden;
- zooms geometrically at 1.02× per frame from fit to `_zoom_max` around a fixed
  high-relief pivot, plus a constant-zoom pan and a zoom-while-pan (research Tests
  A–C);
- captures each frame after `frame_post_draw`.

It computes five metrics:

1. **Temporal discontinuity `T_i`.** Warp frame *i* into frame *i+1* by the known
   camera transform (exact for top-down), then take mean |ΔL\*| over the overlap. A
   **pop** is any frame with `T_i > 3 ×` its rolling 15-frame median and
   `T_i > 1.0` L\* units.
2. **Seam ratio.** The tile-boundary column difference over the mean of its eight
   neighbouring columns (the reference v1.29 method).
3. **Hole pixels.** Pixels inside the map rect covered by neither a live tile nor a
   fallback.
4. **Detail per screen pixel.** Reuses `_mapsharp_probe`'s statistic.
5. **Timing.** Tile synthesis time and worst frame time, as median with min..max,
   with the harness running alone.

It also writes an **Aletsch comparison sheet** for a chosen glaciated view about
25 km across:

- the owner image and the render side by side;
- material-class fractions per elevation band (ice/snow, rock, vegetation, water);
- snowline transition width;
- snow's correlation with slope and aspect.

**Non-goals:** any fix; any threshold picked by feel. The thresholds below are
proposed defaults, calibrated here.

**Parity class:** instrumentation. **Verification:** a planted defect must be
detected by every metric — for example, `_set_lod_active` without its tween must
register a pop.

**Size:** small. **Absorbs:** the measurement half of "A zoom notch costs a 0.87 s
frozen frame", recording whether the stall coincides with LOD entry rather than
assuming it does.

**Risks:**

- Headless capture is vacuous, so the probe runs windowed.
- Thresholds are palette-bound, so the palette is forced.

**Found when built** (`cartalith-godot/src/lod_sweep.rs`, `_lodsweep_probe.gd`;
figures from `OUTSTANDING_WORK.md`'s LOD-D0 row, cited not re-measured):

- **Pops alone do not discriminate.** The baseline had zero pops but a worst
  level-boundary `T_i` of 1.61×, so D3 must be graded on both of its criteria.
- **D2's detail bar needs a pinned view.** Detail per screen pixel at zoom 40
  spans 7.2%–116% of the zoom-1 value across worlds; the "about 8% today" below
  was one world's worst case, not a property of the renderer.
- **The Aletsch view must straddle the snowline.** A coldest-highest-cell chooser
  landed on ground uniformly above it in 4 of 6 worlds, where a snowline
  transition cannot be measured at all.

### LOD-D1 · Port `renderBiomeTileRGBA` as a pure engine function (Ruling K, step 1a)

**Goal:** a Rust function that colours a tile of amplified height with the full
biome look.

**Scope.** `render_biome_tile_rgba(ctx, tile, w, h, bounds, tile_fields)` in
`render.rs`, following the reference line by line.

- **From the tile's own height** (`edge_l/r/u/d`): slope; macro and meso shade; the
  tile-local AO blur; crest (`build_crest` generalised to take `cx, cy`, as the
  reference's `buildCrestField` does); the coast SDF.
- **Bilinear at world coordinates:** T, M, flow, sea height, sea shade,
  `aspect_factor_f`, `curvature_at_f`.
- **Nearest cell:** lithology, and the lake mask. The lake shore ports `_lakeFill`
  if `build_water_bodies` exposes a fill surface; otherwise it uses the reference's
  nearest-cell fallback, with the reason recorded where it happens.
- **Per-tile SDFs:** a river SDF from sampled flow; a biome-boundary distance from
  `classify_biome` of the sampled T and M.
- **Existing helpers reused:** `land_color`, `sea_color_core`, `apply_crest` and the
  coast/river SDF appliers.

The port-only per-pixel stages (paper, border, waves, stipple, grade from a sampled
influence field, colour space) are applied in `cell_color`'s own order, so the tile
matches the screen.

`apply_local_contrast` is a grid-scale neighbourhood pass. It is applied from the
grid's blurred luma sampled bilinearly, not recomputed per tile — a per-tile version
would seam.

A new `TileFields` struct holds the grid-resolution precomputes a tile needs. It is
built once per world and appearance.

**Non-goals:** Godot wiring, the shader, ice, and debug-view tiles.

**Parity class:** golden. **Three checks:**

1. **Against the reference** `renderBiomeTileRGBA` on a small fixture — for example
   a 10×10 grid, a 32×32 tile, seed 24601, both land and sea — under
   `js_reference()`, to 1e-4 per channel. A Node capture script runs the frozen
   reference.
2. **Screen identity.** With zero detail and `bounds` on integer cells at one pixel
   per cell, tile pixels equal `cell_color` to f32 tightness — the bound
   `tests/bake_raster.rs` already holds `BakeFields::pixel` to.
3. **Seams.** Two adjacent tiles agree on their shared column to within 2× the
   interior column-to-column spread.

**Acceptance:** all three green; `cargo test --workspace` green; mutating one
sampled field (T to nearest-cell, say) turns the golden red.

**Budget:** desktop ≤ 40 ms median per 256² tile, single thread (estimated from the
appearance M6 serial figure of about 16 ms per tile, plus the per-tile blur, SDF and
crest). `TileFields` holds ≤ 32 MiB at 2048×1311.

**Size:** large. **Absorbs:** the engine half of the DIAGNOSED shade-ratio row.

**Risks:**

- The reference reads many globals, so the capture harness is real work.
- The water-body fill surface may not be exposed.
- Local contrast has no reference counterpart. It is a principled-equivalence
  sub-stage and is labelled as one.

**Found when built** (`render::render_biome_tile_rgba`, `render::TileFields`;
`OUTSTANDING_WORK.md`'s LOD-D1 row):

- **Checks 1 and 2 pull apart by construction.** The reference's map colour
  (`shadeFactor2`) divides the meso shade by its sample step and its tile meso
  block does not, so a literal port of the tile cannot also be screen-identical.
  It was ported literally (the reference's errors are part of the contract); the
  gap is about 1.67 mean L\* levels at LOD entry. This is owner question 1.
- **The 40 ms budget did not price the port-only tail** (paper, stipple, local
  contrast), which the reference's tile does not have: single-thread synthesis
  measures about 51 ms, all-cores about 6 ms. Three ways out are named at the
  ignored timing test's doc comment; choosing one is an owner call.

### LOD-D2 · Colour tiles on screen, and the sharpness bar (Ruling K, step 1b)

**Goal:** a deeper level visibly shows more detail.

**Relation:** extends integration M1, replacing the shade-ratio content M1 chose on
2026-08-23.

**Scope.**

- **Bridge.** `lod_bridge::synthesize_tile_rgba` returns
  `render_biome_tile_rgba(pyramid_tile(…))`.
  - `WorldGen` caches `TileFields`, keyed by every argument of the function it
    guards: the height and climate `StageGraph` versions, the appearance, the
    quality tier, the pack and the paint.
  - `tile_producer_id` becomes `cartalith-lod/v2`. Stored v1 masks are refused by
    producer id, never decoded under the new shader.
- **Shader.** `lod_tile.gdshader` samples the tile texture directly; `base_tex`
  stays bound only as the entry fallback.
- **Sculpt.** A sculpt commit clears the live tiles.
- **Save size.** `pyramid_mask_bytes` is recomputed for RGB tiles, since the size
  Rulings 28/29 show at save grows about 3× raw.

**Non-goals:** morphing (D3), ice (D4), a worker thread (D6).

**Parity class:** engine content golden via D1; screen agreement and sharpness
probe-verified.

**Acceptance, on the D0 harness:**

- Detail per screen pixel at zoom 16 and 40 is ≥ 50% of the zoom-1 value, on a
  pinned view (about 8% at planning — one world's worst case; see D0's finding).
- Hiding the LOD layer at zoom 16 moves mean |dL| by ≥ 10× the 0.0083 baseline.
- At the LOD entry frame, mean |ΔL\*| between the layer shown and hidden is ≤ 2.0.
- Seam ratio ≤ 1.5 at every level.
- `deeper_levels_carry_strictly_finer_detail` is rewritten against colour tiles and
  stays red if `add_zoom_detail` is removed.

**Budget:**

- **Desktop:** no frame over 50 ms from synthesis; the backlog drains the rest.
- **Memory:** ≤ 36 MiB for about 68 cached 256² RGBA tiles (the reference's measured
  working set).
- **Phone:** until D6, at most one tile per frame, with progressive fill accepted;
  a frame over 100 ms from synthesis is a failure.

**Size:** medium. **Absorbs:** the rest of the DIAGNOSED row, and "The in-session
tile cache is not invalidated by a sculpt" if still open.

**Risks:**

- Colour tiles make popping *more* visible than a shade ratio did, so D3 must follow
  directly.
- A stored v1 pyramid could be silently misread.
- The phone stall could get worse.

**Found when built** (`OUTSTANDING_WORK.md`'s LOD-D2 row): caching `TileFields`
alone was not enough — a `RenderCtx` costs about 200 ms to build at 2048×1311,
so both are cached on `WorldGen` (`Cow`-backed), keyed on the world, pack and
paint epochs plus the appearance's serde fingerprint. The LOD-entry ΔL\* bar
traces to D1's meso-step disagreement (owner question 1), not to the bridge.

### LOD-D3 · Continuous transitions: parent fallback and a colour-space morph

**Goal:** the research's primary criterion — no visible popping, holes or mode
switch.

**Relation:** extends M1's compositor; question 5 covers the clipmaps boundary.

**Scope.**

- **Parent fallback.** A tile's parent (`chunk_parent`) stays alive until all four
  children are built and draws under the missing ones. This removes holes.
- **Morph.** The shader takes
  `t = smoothstep(0, 1, log2(screen_px_per_cell / level_px_per_cell(z)) + 0.5)` and
  outputs `mix(parent_sample, tile_sample, t)`, so a level fades in across its own
  half-level band instead of switching at the `js_round` boundary.
- **LOD entry.** The entry level's parent is `map_view.texture` sampled linearly.
  The 0.15 s layer tween is removed, and `t` supplies the fade.
- **Content.** The pyramid content is unchanged, so every golden holds.

**Non-goals:** geometry of any kind; morphing inside `add_zoom_detail`'s golden
octave schedule.

**Parity class:** principled equivalence. Sources: Strugar (CDLOD morph); Asirvatham
& Hoppe (transition blending).

**Acceptance, on the D0 harness** (three seeds, two grid sizes):

- **Zero pops** across the full sweep, the constant-zoom pan and zoom-while-pan.
- The worst `T_i` at any level boundary is ≤ 1.5× the median of the non-boundary
  frames on either side.
- Zero hole pixels after the first built frame.
- Seam ratio ≤ 1.5 with adjacent-level tiles on screen together.

**Budget:** parents add at most a quarter of the child count (≤ 9 MiB) and one
texture sample per fragment. Phone: at most one frame over one vsync per level change
at rest.

**Size:** medium. **Absorbs:** research §11 and Tests A–C.

**Risks:**

- Parent eviction interacting with the backlog (the M1 dropped-tile bug class).
- Mid-band blending softening detail — judged by eye as well as by metric.

**Found when built** (`lod_bridge::morph_for_zoom`; `OUTSTANDING_WORK.md`'s
LOD-D3 row): the morph is computed engine-side from `level_for_zoom`'s own
expression minus the rounding, so the fade and the level switch cannot disagree
about a boundary. **The zero-holes bar is partly structural:** the pyramid
samples `[0, gw − 1]` while the map raster covers `[0, gw]`, so the outermost half
coarse cell lies outside every tile. A parent tile must also be laid out by the
*child's* texel, or its inset overhangs the children by half a texel and reads
as a seam.

### LOD-D4 · Ice and snow from fields that already exist (Ruling K, step 2)

**Goal:** the Aletsch reading from above: ice filling troughs, snow on high ground
breaking along aspect and slope, rock on steep faces, vegetation in valleys.

**Confirmed 2026-09-13: a default new world has no glacial troughs.**
`cartalith_engine::WorldParams::defaults` sets `passes: ErosionPassParams::off()`
(glacial carving off), and the shell's `cartalith_godot::params::defaults()` does not
override it. Ice therefore appears only where the user enabled the glacial pass;
see question 3.

**Scope.** Three derived stages, all inert under `js_reference()`:

1. **Glacier potential**, at grid resolution, stored in `TileFields` and usable by
   the main map. It is `glacial_kernel`'s own gate (`h ≥ sea + (1 − sea)·glacial_snowline`
   and `T < 0`), weighted by `smoothstep` of `flow_discharge`, then blurred one cell
   so trough floors read as tongues.
2. **Tile-resolution snow.** Tile temperature is
   `T_sampled − lapse_rate·g·(h_tile − h_coarse)·height_scale`, using
   `cartalith-climate`'s own lapse relation, so the snow fraction follows sub-cell
   relief. *As planned*, "the existing aspect and curvature inputs of
   `material_weights` drive the breakup". **That premise was false when D4 was
   built** (see *Snow's aspect term* below): `material_weights`' snow term was
   `smoothstep(3, -5, t)`, temperature alone, as this document's own gap table
   (§16 row) already said. Since `19c38d9` it takes a `snow_shift_c` argument
   from `snow_aspect_shift`.
3. **Ice colour.** Where glacier potential is high, snow takes `snow_glac` plus a
   slope- and flow-aligned brightness term from the tile's own height. Rock exposure
   keeps `geo_exposure(slope, r, snow)`, so steep faces above the snowline stay rock.

**On a loaded save**, which has no flow, it falls back to snow only, and the UI
dashes the reason.

**Non-goals:** a glaciation model (not chosen in Ruling K); any change to
`glacial_kernel` or its golden.

**Parity class:** principled equivalence (the reference has no ice layer), with the
lapse rate and the kernel's gate cited.

- Existing goldens are unchanged.
- A test shows tile snow fraction rising with sub-cell height at a fixed coarse
  temperature; zeroing the lapse term turns it red.

**Acceptance, on the D0 comparison sheet** (three glaciated seeds, views about 25 km
across):

- Snow is not an elevation cutoff: the 10–90% transition spans ≥ 15% of local
  relief, and within that band snow correlates with aspect at |r| ≥ 0.2. (The
  aspect half cannot be met inside D4's own rules — see below.)
- Cells with glacier potential ≥ 0.5 render as ice or snow in ≥ 80% of their pixels.
- Pixels above the snowline with slope > 0.08 are rock-dominant in ≥ 60%.
- Pops stay at zero.
- **The owner's visual verdict against the Aletsch image is recorded as its own
  line.**

**Budget:** glacier field ≤ 10 MiB, under 50 ms to build on desktop; per-tile cost
up by at most 10%.

**GUI:** one "Ice & snow" group in the render workspace, subject to the GUI
definition of done — a canvas draws it, or the owner is asked.

**Size:** medium. **Absorbs:** "Draw ice from fields that already exist, toward the
Aletsch zoom target."

**Risks:**

- **Scale.** The Aletsch glacier is 1–1.5 km wide, while the default world is about
  0.39 km per cell, so a trough is 3–4 cells wide and tongues will read soft at deep
  zoom. This is stated, and no geometry is invented.
- If the glacial pass is off by default, most worlds will show snow and no ice.

**Snow's aspect term — owner-ruled, and outside D4.** Built as scoped, D4 gates
its stages on `TerrainAppearance::ice_strength` (`build_glacier_potential`,
`apply_ice_cover`, which rebalances the material weights off `geo_exposure`'s own
slope term). The aspect bar then measured no correlation (|r| under 0.1 on three
seeds, `OUTSTANDING_WORK.md`'s LOD-D4 row), and the cause was at the symbol: snow
read temperature alone, so no stage that left `material_weights` untouched could
make it follow aspect. Giving snow an aspect term is a **re-baseline of the main
map's own `material_weights`**, which D4's no-re-baseline rule excluded.
**Ruling AP (`LARGE_ITEM_RULINGS.md`, 2026-09-23) authorises exactly that
re-baseline**: snow gets an aspect (slope-direction) term in `material_weights`
itself — on the main map, not only in LOD tiles — and every existing snowy
mountain's rendered look changes. It was built as its own work, not as part of
D4, in `19c38d9` (2026-09-24). The term is `TerrainAppearance::snow_aspect_c`,
2.0 °C in `default()` and 0.0 under `js_reference()`. `snow_aspect_shift` feeds
it to `material_weights` as a temperature shift, and LOD tiles supply their own
facing through `tile_snow_facing`. That commit's own measurement left bar 1b
unmet. Status is in `STATUS.md`.

Two further findings from building D4, both kept in `MISTAKES.md`: a fixture
guarded by three separately-true conditions (flow, cold, height) never had all
three on one cell, so the glacier potential it certified was empty; and a
metrics probe placed inside `apply_border`'s margin compared content against the
frame. The ice-off cache key folds `glacial_snowline` in only when
`ice_strength > 0`, since nothing else reads it.

### LOD-D5 · Scale-aware shading weights, and hydrology that resolves

**Goal:** research §12–14 and §18 — the balance of shading scales and the drainage
detail change continuously with ground scale.

**Scope.**

- **Shading weights.** `detail_{macro,meso,micro}_weight` become the fit-zoom values
  of a curve over the tile's km per pixel: macro-heavy far out, meso and micro-heavy
  close in.
- **Micro shade.** `sh_micro` comes from the tile's `add_zoom_detail` residual
  instead of value noise at tile levels. The main map keeps its current formula, so
  its golden holds.
- **Radii.** Crest and AO radii are set in ground units.
- **Rivers.** The per-tile river SDF threshold drops continuously with km per pixel,
  so tributaries appear where the sampled discharge supports them and nowhere else.
- **Check at the symbols** whether `add_zoom_detail`'s frequency ignores cell size.
  If it does, record it for an owner ruling; the golden schedule does not change
  here.

**Non-goals:** new hydrology; channels the flow field does not carry; vector rivers
(a separate backlog row, reverted 2026-09-13).

**Parity class:** principled equivalence. Sources: Li et al. 2021; Terrain3D's
derivative normals. A test holds the zoom-1 weights byte-identical to today.

**Acceptance:**

- In a named river valley, the count of distinct channel components in view rises
  with zoom, and every drawn channel pixel has sampled discharge ≥ the threshold
  (research Test E).
- Detail per screen pixel is non-decreasing from zoom 4 to 40.
- Pops stay at zero.

**Size:** medium.

**Found when built** (`TerrainAppearance::detail_scale_strength`, which gates all
four stages and is the identity at `0.0`; `cartalith-terrain/src/amplify.rs`,
`add_zoom_detail`'s doc comment):

- **AO needed nothing.** `ao_radius_frac` was already a ground-scale fraction, so
  only the crest radius had to move into ground units.
- **The symbol check came back positive, three ways, all recorded at
  `add_zoom_detail` for an owner ruling rather than changed:** its frequency is
  per *coarse cell*, not per kilometre (an 800 km and a 40 000 km world on one
  grid get the same detail wavelength in cells); its octave count is keyed on the
  pyramid level, not ground scale; and its amplitude decays 0.6× per octave while
  ground per pixel halves per level. The third is not a defect, but it is why the
  "non-decreasing from zoom 4 to 40" bar is **unattainable without changing that
  schedule** — a golden re-baseline of every pyramid tile.

### LOD-D6 · Tile synthesis off the main thread, profiled per device

**Goal:** research §22–24 and Tests F–G — phone viability.

**Scope.**

- **Worker thread.** Synthesis moves off the main thread (planned as
  `WorkerThreadPool`, which cannot carry it — see *Found when built*). Rust computes
  RGBA from a snapshot of the height slice and an `Arc` of `TileFields`; the main thread uploads
  the texture; a world version checked on landing drops stale tiles. No `Gd` crosses
  a thread.
- **Quality tier.** The tier sets tile budget, maximum level and cache size — for
  example, Android Performance gets `MAX_LEVEL − 1` and a 32-tile cache.
- **Profiling** on desktop, an integrated GPU if available, and the phones.

**Non-goals:** a wgpu tile path (question 4).

**Parity class:** determinism. A worker-built tile is byte-identical to a main-thread
one.

**Acceptance:**

- **Phone, 2048×1311:** during the D0 sweep, p99 frame ≤ 33 ms and worst frame
  ≤ 100 ms.
- **The 0.87 s zoom-notch stall** is gone, or measured and attributed elsewhere.
- **Steady memory** at most 60 MiB above the recorded steady state; the generation
  peak is unchanged.
- **Thermal and battery drain** over a five-minute pan and zoom are recorded.

**Size:** medium. **Absorbs:** the device half of the zoom-notch row, if the harness
ties it to LOD.

**Found when built** (`cartalith-godot/src/lod_worker.rs`, its module doc *"Why a
Rust-side pool and not `WorkerThreadPool`"*): **`WorkerThreadPool` cannot do
this.** It takes a `Callable`, and `WorldGen` holds a `RefCell` and `Gd` handles,
neither `Sync`. Synthesis runs on a Rust-side `rayon` pool instead, over an owned
`Send + Sync` `LodSnapshot` behind an `Arc`, whose `render_tile()` is the only
colouring function both the main thread and every worker call — so the
determinism bar holds structurally. A generation counter drops tiles that land
stale. Keeping that snapshot cheap is why `WorldState`'s four large `Vec<f32>`
fields became `Arc<Vec<f32>>`.

### LOD-D7 (optional) · Debug and info views in tiles

**Scope:** port `renderAffordanceTileRGBA` so the info layers stay detailed and honour
opacity at deep zoom (reference v0.89/v0.91), with a golden per view family.

**Size:** small to medium. **Only if the owner wants it** (question 6).

---

## Parked, and out of this document

- **3D:** parked since 2026-08-31, and Ruling K confirms it.
- **Clipmap geometry and streaming:** outside `LOD_TILING_INTEGRATION_SCOPE.md`'s
  boundary.
- **A glaciation model:** not chosen.
- **Microdetail beyond `add_zoom_detail`:** the research says only after the physical
  systems work. Revisit after D5, with the harness's numbers.

---

## Dependency order and scheduling

**Order:** D0 → D1 → D2 → D3 → D4 → D5 → D6.

- **D6** can start after D2 in its own lane if a phone regression appears.
- **D4** can run beside D3 if the lanes do not share `viewport_host.gd` — D4 is
  `render.rs` only.
- **D1** splits into "port + golden" and "screen identity + seams" if the capture
  harness is heavy.

**This sits behind the GUI work, tablet first** (owner, 2026-09-12). **D0 is the one
piece a spare lane could run between GUI batches**, since it changes no pixels — but
it must not overlap a GUI verifier's tree.

---

## Owner questions — each with the default the plan assumes

1. **Match the reference tile, or this port's screen including stages the reference
   never had?**
   *Default:* both. Golden against the reference under `js_reference()`, identity
   with the port's own screen under the shipped look (D1). No re-baseline.
   *Since found:* the two disagree by construction (D1's *Found when built* — the
   reference's tile skips the meso-step division its own map applies), so "both"
   is not exactly available; screen-matching is a two-line change if chosen.
2. **Do the Rulings 28/29 stored pyramids become colour tiles** (about 3× raw), or
   does the save slot stay off until D3?
   *Default:* producer id v2, slot off by default, the new size shown at save.
3. **Glaciation.** Confirmed: the glacial pass is off in both the engine defaults
   (`ErosionPassParams::off()`) and the shipped shell defaults, so a default world has
   no troughs to draw ice into. Should new worlds turn it on? That changes generated
   output in the shell's divergence defaults only; engine defaults and goldens are
   untouched.
   *Default:* leave it as shipped. D4 draws snow and rock on every world and ice where
   the pass ran. Ask again with the D0 comparison sheet in hand.
4. **Tile synthesis on the GPU (wgpu), or CPU with a worker?**
   *Default:* CPU with a worker (D6), per appearance M6's measured verdict and to stay
   on the golden-verified path.
5. **D3's colour-space morph borrows the CDLOD blending idea**, and
   `LOD_TILING_INTEGRATION_SCOPE.md` rules out "anything resembling … clipmaps".
   *Default:* rule it inside scope — it is a 2D shader blend with no geometry or
   paging — and record the ruling against that bullet.
6. **Do info and debug views need deep-zoom tiles (D7)?**
   *Default:* after D6, only if asked.

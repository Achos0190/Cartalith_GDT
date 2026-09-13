# Reference Map Reconstruction Mode — research and design

**What this is.** Owner-requested research, 2026-09-13. It is **not a scope
document**: it records what exists, what outside tools and literature suggest,
and a proposed design with milestones. Status lives only in
`cartalith-native/docs/STATUS.md`. Build rows are filed only once the owner has
answered the questions at the end.

## The request, verbatim

> "Reference Map Reconstruction Mode sits alongside Cartalith's native heightmap
> generation and heightmap import/inference workflows. When a user has an
> existing map image but no usable heightmap, they can load the image as an
> adjustable-opacity reference and create the heightmap directly over it using
> Cartalith's already existing sculpting tools.
>
> Supplemented with an edge detection tool to quickly recognise landmasses and
> river systems. User should be able to sculpt the cartalith layer and draw in
> rough where the voronoi/tectonic plates should be. The drawn plates then should
> also have a input to make the edges less smooth much like the noise beta and
> warp functions in the base generator."

## How to read the claims

- **[V]** — verified at the symbol by the research pass (read-only; no build or
  run).
- **[P]** — proposed.

## Four findings that shape the design

1. **Beta does not roughen plate edges.** Only warp moves plate edges, and only
   at a large scale; beta adds noise to height. The request's "noise beta and
   warp" needs a ruling (question 1).
2. **Saved projects cannot be sculpted today.** An opened project cannot commit
   sculpt edits, and plate data is not saved. A tracing job that spans several
   sessions needs this fixed first (RM-0).
3. **The engine can take plates from outside; the pipeline cannot.**
   `assign_plates` accepts any list of plates, but both the generator and the
   import build their own internally, so nothing passes user plates in.
4. **No design canvas and no reference-HTML feature exists for any of this.** It
   is new capability under `DECISIONS.md` §7d, and GUI "done" needs an
   owner-approved canvas.

---

## 1 · What already exists

### 1.1 Heightmap import and inference [V]

**Entry points**

- `app.gd::open_heightmap_import()` → `EngineBridge.import_heightmap(path,
  new_world_dialog.request())` → `WorldGen::import_heightmap(path, seed, width_km,
  grid_w)` → `std::fs::read` → `cartalith_engine::import::import_heightmap`.
- The same flow is also reached from:
  - Data ▸ Import ▸ Heightmaps (`data_manager_window.gd`);
  - the welcome screen (`open_project_dialog.gd`);
  - the WORLD buttons "Load heightmap…" and "Infer tectonics from heightmap…"
    (`world_workspace.gd` `_build_terrain_head` / `_build_geology_foot`). Both
    open the import; no `#[func]` re-infers over an existing surface.

**Decoding**

- `cartalith_assets::raster::decode_png` — PNG only; the `image` crate is built
  with `features = ["png"]`.
- `infer::heightmap_to_field` box-filters the image to the grid and takes Rec.601
  luma at 8-bit precision.
- `heightmap_grid_h`: `GH = max(80, round(GW/ar))`.
- `normalize_field`.

**Inference — `import::infer_tectonics(field, p)`**

1. `build_relief_field`.
2. `pick_plate_seeds` — one seed per region, placed at the lowest relief; count is
   `clamp(round(w*h/3000), 6, 40)`.
3. `assign_plates(..., None, None)` — the Voronoi, without warp.
4. `classify_plate_crust`, then `reconstruct_boundary_stress`.
5. `build_age_field`, `compute_resistance`, `stamp_volcanic_arcs`,
   `infer_plate_velocities`.
6. Climate against a zero flow field, then `compute_flow`.

The field itself is left untouched: the test
`infer_tectonics_does_not_modify_the_imported_field` pins that. The world then
goes through `absorb(state, params, seed, ORIGIN_IMPORTED)`, giving a
`WorldSource::Generated` world with a fresh `SculptEditor` — **so an imported
world can be sculpted.**

**Parity:** the inference is bit-exact (`cartalith-terrain/tests/golden_parity_infer.rs`);
the resampling is deliberately not.

**Limits**

- PNG only, luma only, 8-bit.
- No way to supply plate seeds.
- Grid height is derived from the image.
- A colour map is read as if it were a heightmap.

### 1.2 Sculpting [V]

**Engine — `cartalith-terrain/src/sculpt.rs`**

- 13 features (`FEATURE_KEYS`): Mountains, Hills, Ridge, Plateau, Cliff, Canyon,
  Valley, **River**, **Lake**, Basin, **Coastline**, Volcano, Freehand. Freehand
  has 8 sub-modes.
- A `SculptStamp` is a recipe: seed, stroke points in grid cells, and parameters.
  It implements `cartalith_spatial::Stamp` and lives in a
  `PassBuffer<SculptStamp>` (draft, preview, commit, discard; draft undo capped at
  `HISTORY_MAX` = 30).

**Bridge** — `sculpt_bridge::SculptEditor`, exposed as `#[func]`s:
`sculpt_set_feature`, `sculpt_begin_stroke` / `sculpt_add_point` /
`sculpt_end_stroke`, `sculpt_list_stamps`, `sculpt_undo` / `sculpt_redo`,
`sculpt_commit`, `sculpt_discard`, `sculpt_set_grid_snap`.

**Commit — `WorldGen::sculpt_commit`**

- Refused while the world is finalized (`bake.check(Mutation::HeightEdit)`), and
  requires a `WorldSource::Generated` world.
- `undo.push("Sculpt commit", &ws.field)` — `MAX_STEPS` = 5, a 256 MiB budget,
  about 16 MB per snapshot at 2048².
- `sculpt_commit::commit_sculpt_pass`: bake, re-clamp river locks, carve and lock
  River stamps, dry-run Lake stamps.
- `mark_and_recompute(PipelineStage::Height, …)`.
  - `cartalith-engine/src/staleness.rs` has four stages: height → hydrology →
    climate → civ.
  - `recompute_stale` re-runs hydrology and climate and leaves civ stale.
  - The civ recompute is a manual `#[func]` (about 4.22 s at 2048²).

**GDScript** — `world_workspace.gd::_build_sculpt`; strokes via `_sculpt_click` /
`_sculpt_drag` / `_sculpt_release` in grid coordinates. A loaded save shows "a
loaded save has no draft session".

**Opened projects cannot commit sculpt edits.** `load_save` produces
`WorldSource::Loaded`, which has no substrate; `project_open` can rebuild a draft,
but committing still needs a generated world (see the comment at
`self.sculpt = None` in `lib.rs`).

**Precedent for categorical painting:** `cartalith_spatial::paint::PaintStamp` /
`PaintLayer` (u8 values, 0 means unpainted, edge falloff per `DECISIONS.md` §7k),
bridged by `paint_bridge.rs`, with `paint_commit` marking civ stale.

### 1.3 Image layers and underlays [V]

- `viewport_host.gd` layers, bottom to top:
  - `map_view`, `territory_view`, `province_view`;
  - `_preview_layer` (draft raster; bounded `set_preview_patch`);
  - `_debug_layer`, whose opacity `set_debug_opacity` drives via `modulate.a`;
  - `overlay` (`map_overlay.gd`);
  - `tool_overlay` (`ToolOverlay`: `set_grid`, `set_brush_cursor`,
    `set_path_preview`, `set_handles`, `set_region`);
  - the LOD tile sprites.
- `_map_display_rect()` maps the grid to the screen.
- **There is no user-image layer.** In the reference, the only opacity control is
  the debug layer's (`#dbgOpacity` / `#layersOpacity` → `state.debugOpacity`,
  v2.11 around 13467 and 14183).
- GDScript already decodes images: `Image.load_from_file` in
  `new_world_dialog.gd`, and `load_png_from_buffer` in
  `asset_library_window.gd`.

### 1.4 Plates, beta and warp [V]

**Generation** — `cartalith_engine::generate_terrain_inner`:

1. `compute_warp(gw, gh, tect.seed, tect.warp, world)`
2. `build_plates(seed, n, lloyd, world, ws)` — mulberry32 positions and
   velocities, 45% oceanic, Lloyd relaxation.
3. `assign_plates(gw, gh, world, &plates, warp_x, warp_y)` — a jump-flood Voronoi
   producing `Vec<u16>`.
4. `compute_stress` — boundary mask and type, stress and shear, from velocity ×
   `tect.vel`, blurred by `tect.blur_r`.
5. Flexure, base field, age, heterogeneity, resistance, optional orogeny.
6. `compute_height`, then `normalize_field`.

**Warp.**

- Amplitude is `tect.warp × 0.18 × gw` cells; below 0.5 cells the kernel returns
  `None`.
- Frequency is **fixed** at `2.5/gw` (region) or `3/gw` (world), using a 6-octave
  fbm with a domain warp inside it.
- `assign_plates` looks up the Voronoi at `(x + warpX, y + warpY)`.
- **So warp gives continent-scale bends, not fine jagged edges.**

**Beta** (`tect.beta`, "Noise β", 0–0.6) appears only in `compute_height`, as
`p.b * n_val * (0.25 + 0.75*rug)`. **It roughens height, never plate edges.**

**Uplift spread** is `tect.blur_r` (2–42).

**External plates.**

- `assign_plates` is public and takes any `&[Plate]`, but no orchestration passes
  user plates: `generate_terrain` uses the seed only, `infer_tectonics` uses relief
  only.
- There is no painted plate mask.
- Only `plate_id` is kept on `WorldState`, and the substrate is not saved.

**Debug views already present:** `plates`, `bounds`, `btype`, `stress`
(`sample_bridge.rs` `LAYER_GROUPS`).

### 1.5 Image processing in the workspace [V]

**Crates** (`Cargo.lock`): `image` 0.25.10 (PNG only), `png` 0.18.1, `fdeflate`,
`rayon` 1.12, `zip` 8.6, `serde_json`, `wgpu` 30. There is no `imageproc` and no
JPEG decoder.

**Reusable kernels:**

- `gauss_blur`
- `infer.rs::chamfer_dist` — distance to coast
- `build_age_field`
- `thin_mask` — Zhang-Suen thinning
- `trace_boundaries` — mask to polylines
- `cartalith-spatial/src/geo.rs`: `trace_mask_rings`, `ring_area`,
  `point_in_ring`, `id_mask`
- `measure::cell_km`
- the alpha trim in `slicer.rs`

**Not present** (grep over `crates/*/src`): Sobel, Canny, Otsu, k-means, and
flood-fill segmentation.

### 1.6 Reference HTML [V]

`loadImage` (v2.11 line 4940) takes the image's luma into `field`, normalises,
refreshes climate and opens the calibrate gate; inference runs on commit.

The reference has **no** reference-image overlay, tracing mode, image segmentation
or plate drawing. Everything below is **new capability** under §7d, apart from the
ported kernels it reuses.

### 1.7 Design canvases [V]

**No canvas covers this feature** (grep over `design/`). The vocabulary a design
could derive from:

- the Layers popover's `Opacity` row (`Cartalith DCC Shell.dc.html` lines 125 and
  1361);
- "Plates" as a Layers row (`proposed-2026-09-05-round2/LayersPopover.dc.html`);
- the SCULPT screen ("DCC Generate Sculpt 1920");
- the owner's re-sorted rail
  (`owner-references-2026-09-12/left_rail_tree_resorted.md`): WORLD ▸ PIPELINE |
  SCULPT, where SCULPT holds "everything done by hand on the map surface" and
  Generate ▸ Import holds Load heightmap and Infer tectonics.

### 1.8 Backlog and docs [V]

- `FUNCTIONAL_CONTRACT.md` §9 records heightmap import and inference as done and
  bit-exact.
- `GUI_GAP_REGISTER.md` MS-02 is closed (2026-08-20).
- No row in `OUTSTANDING_WORK.md`, `GUI_GAP_REGISTER.md` or `UNIFIED_TOOL_PLAN.md`
  mentions a reference image, tracing, or plate painting.
- Android memory: `ANDROID_BUILD_SCOPE.md` records 878 MB peak / 647 MB steady at
  2048×1311, with a later 1 033 / 818 MB note cited in
  `MEMORY_OPTIMIZATION_SCOPE.md`.

---

## 2 · Outside research

### Tracing tools

**Azgaar's Fantasy Map Generator** has an Image Converter.

- An "overlay opacity" slider fades the source image over the heightmap being
  built.
- Heights are assigned by luminosity, hue, a colour scheme, or by hand per colour,
  after reducing the palette (100 colours by default).
- The user then refines with brushes (Raise, Smooth, Disrupt) and the template
  editor.
- Sources: [heightmap customization](https://github.com/Azgaar/Fantasy-Map-Generator/wiki/Heightmap-customization),
  [template editor](https://github.com/Azgaar/Fantasy-Map-Generator/wiki/Heightmap-template-editor),
  [import discussion](https://github.com/Azgaar/Fantasy-Map-Generator/discussions/1249).
- **Lesson:** reduce the palette, map colour classes to height, then refine by
  brush — which maps directly onto detect → rough in → sculpt.

**Wonderdraft's Trace Tool**: opacity (0.30–0.45 suggested), an Additive blend,
and a Scale slider. [walkthrough](https://painfullyhopeful.me/2020/10/28/map-upgrade/)

**Dungeondraft's Trace Image**: opacity, scale, drag to offset, a "Center" reset,
and a **T** key to toggle. It has no rotation, and the trace image is never
exported. [guide](https://dungeondraft-encyclopaedia.gitbook.io/guide/all-the-tools/settings-tab/trace-image),
[Encounter Library](https://encounterlibrary.com/dungeondraft-basics/settings-tracing-exporting/)

**Gaea**: the Mask node takes a guide image, with Blur, Iterations and Strength.
Its docs recommend a **Warp node before combining, "for added shape randomness"**
— the same idea as warp-roughened hand-drawn plates.
[Draw to Modify](https://docs.quadspinner.com/Learning/Techniques/Draw-Modify.html),
[Mask](https://docs.quadspinner.com/Reference/Data/Mask.html),
[masks](https://docs.gaea.app/using-gaea/terrain-basics/masks). World Machine
takes bitmap masks as inputs ([forum](https://forum.world-machine.com/t/masking-terrain/6172)).

**Plate sketching.** No consumer terrain tool was found with interactive plate
drawing.

- GPlates is used for plate polylines ([Astrographer](https://astrographer.wordpress.com/2013/08/22/using-gplates-for-realistic-worldbuilding/)).
- Manual guides divide a map into 8–10 irregular plates, then assign motion per
  plate ([Worldbuilding Workshop](https://worldbuildingworkshop.com/2015/11/19/tectonic-plates/),
  [Emma Lindhagen](https://www.emmalindhagen.com/2015/11/worldbuilding-wednesdays-step-by-step-for-tectonic-plates/)).
- **Lesson:** the useful input is rough regions plus motion per plate, not exact
  lines.

### Land, water, coastline and river detection

- **Sea/land segmentation, then the boundary is the coastline.** Thresholding and
  clustering on colour are the classic methods; superpixel classification refines
  them. [MDPI](https://www.mdpi.com/2072-4292/15/19/4865),
  [T&F review](https://www.tandfonline.com/doi/full/10.1080/15481603.2023.2243671),
  [coastline edge detection](https://arxiv.org/html/2405.11494v1).
- **Scanned maps discolour mostly in the blue and green channels**; one study used
  only the red channel. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S1470160X23015054)
- **Otsu vs k-means.** Otsu is a global two-class threshold; k-means handles
  several classes but struggles with uneven colour. Morphological opening, hole
  filling, and removing small blobs are the usual cleanup.
  [comparison](https://www.researchgate.net/publication/334967194_A_Comparative_Study_of_Otsu_Thresholding_and_K-means_Algorithm_of_Image_Segmentation),
  [Microsoft ISE](https://devblogs.microsoft.com/ise/using-otsus-method-generate-data-training-deep-learning-image-segmentation-models/),
  [k-means quantisation](https://arxiv.org/pdf/1101.0395).
- **Zhang-Suen thinning** gives centred, connected skeletons for line drawings.
  [implementation](https://github.com/linbojin/Skeletonization-by-Zhang-Suen-Thinning-Algorithm),
  [variant](https://www.researchgate.net/publication/254040436_Improved_Zhang-Suen_thinning_algorithm_in_binary_line_drawing_applications).
  The port already has it (`thin_mask`).

**Handling map clutter** — the research's own synthesis, not from a source:

- Segment the image **after resampling to the grid**; the box filter is a low-pass
  filter against hatching and parchment texture.
- Flood-fill from ocean points the user taps, so blue text inside land is not
  reached.
- Stop the fill at strong edges, so coast ink acts as a wall.
- Close small gaps before filling, so hatched seas stay connected.
- Filter components by area in km², which removes labels and compass roses. Show
  the removed components so real islets are not silently deleted.
- Provide an eraser brush.

### Roughening hand-drawn boundaries

- **Domain warping** evaluates `f(p + fbm(p))` ([Inigo Quilez](https://iquilezles.org/articles/warp/)).
  Applied to a nearest-region lookup, this is exactly what `assign_plates` already
  does with `warpX` / `warpY`.
- **Noisy edges by recursive midpoint displacement** ([Red Blob Games](https://www.redblobgames.com/maps/noisy-edges/))
  is a vector method; the raster warp fits this engine better.

---

## 3 · Proposed design [P]

### 3.1 Workflow

1. **Start.** Either "New world from reference map…" (grid aspect via
   `heightmap_grid_h`, width in km from the New World form, a flat all-ocean
   field), or attach a reference to the current world.
2. **Register.** "Fit to world" by default; then drag corners, or match two image
   points to two grid points to solve scale, rotation and offset; numeric fields
   for exact values.
3. **Onion-skin.** Opacity slider; visibility toggle (T, plus a hold-to-peek button
   on touch); Normal or Difference blend.
4. **Detect** (optional).
   - Tap the ocean (and optionally land) to sample colours → Detect land → review
     the mask and the component list → erase or add by brush.
   - Tap a river colour → Detect rivers.
5. **Rough in.** "Apply land mask as base height" writes a shelf and an interior
   rise from distance to the coast, as one undoable height commit.
6. **Sculpt** with the existing tools. Detected rivers arrive as **uncommitted
   River stamps** in the sculpt draft.
7. **Draw plates.** Tap to drop sites; optionally paint regions; set crust and
   drift per plate, or leave them inferred.
8. **Roughen edges.** Edge warp, Edge detail (question 1), and the existing Uplift
   spread, with a live preview.
9. **Apply plates** — rebuild the substrate under the user's height.

**What reruns on Apply.** By default:

- **Rebuilt:** `plate_id`, boundary mask and type, stress, shear, crust, age,
  resistance, volcanic field, velocities.
- **Untouched:** height, so hydrology and climate stay current.
- **Stale:** civ, left for the existing manual Recompute.

An optional later mode (RM-11) grows tectonic relief from the drawn plates as a
separate undoable height edit.

### 3.2 Rust / GDScript split

**Engine crates** (plain Rust, tested under `cargo test`)

`cartalith-terrain/src/reference.rs`:

- `Registration { a, b, c, d, tx, ty }` — image pixel to grid cell — with
  `fit_to_grid`, `from_point_pairs` (a similarity fit), and `invert`.
- `resample_rgb_to_grid(rgba, w, h, &Registration, gw, gh) -> (rgb, covered)`, a
  box filter that generalises `heightmap_to_field`. Uncovered cells are marked,
  never faked.
- `palette_kmeans(rgb, k, seed)`, deterministic via `cartalith-rng`.
- `detect_land(rgb, covered, gw, gh, &DetectOpts) -> LandDetection { land,
  components }` — sampled-colour distance, a seeded flood fill with a Sobel
  barrier, morphological close, area filters in km² via `cell_km`.
- `detect_river_lines(rgb, land, …) -> Vec<Vec<Point>>` — colour distance plus a
  black top-hat line filter, then `thin_mask`, polyline trace and simplification.
- `base_from_land_mask(land, sea_level, shelf_km, rise, cell_km)`, using
  `infer::chamfer_dist`.

`cartalith-terrain/src/plates_drawn.rs`:

- `PlateLayout { sites: Vec<PlateSite{x, y, base: Option<f64>, vx:
  Option<f64>, vy: Option<f64>}>, paint: Option<Vec<u8>>, edge_warp,
  edge_detail, edge_seed }`. **An absent override is `None`, never a plausible
  default.**
- `assign_plates_constrained(gw, gh, world, &layout, warp_x, warp_y)`:
  - with no paint, it **delegates to `assign_plates`**, so it is bit-identical by
    control flow;
  - with paint, it runs a multi-source jump flood seeded from the labelled painted
    cells, looked up through the warp, so painted edges roughen too.

`cartalith-engine/src/import.rs`:

- Split `infer_tectonics` into seed picking plus
  `infer_tectonics_with_plates(field, p, plate_id, plates)`; the existing goldens
  guard the refactor.
- `rebuild_substrate(ws, p, layout)` runs only the substrate half and **skips
  climate**, so a saved climate is never overwritten.

**Bridge (`cartalith-godot`)**

- `reference_bridge.rs` (no Godot types).
- `plate_draw_bridge.rs`, modelled on `paint_bridge`, holding a
  `PassBuffer<PaintStamp>` for plate painting.

**`#[func]`s.** Every call returns a Dictionary with omitted keys plus a `reason`,
and never panics.

| Group | Functions |
|---|---|
| Reference | `reference_load_file(path)` (PNG, decoded in Rust), `reference_load_pixels(rgba, w, h, name)` (JPEG/WebP, decoded by Godot), `reference_set_registration(dict)`, `reference_solve_registration(pairs)`, `reference_get()`, `reference_clear()`, `reference_sample_color(gx, gy)` |
| Detection | `reference_detect_land(opts)`, `reference_detection_texture(which)`, `reference_mask_paint_at(gx, gy, erase, r)`, `reference_apply_land_base(opts)`, `reference_detect_rivers(opts)` (pushes River stamps into the sculpt draft) |
| Plates | `plates_draw_begin(from: "current"\|"empty")`, `plates_add_site`, `plates_move_site`, `plates_delete_site`, `plates_paint_at`, `plates_set_edge(dict)`, `plates_set_overrides(i, dict)`, `plates_preview_texture()`, `plates_apply()`, `plates_discard()` |
| Session | `continue_editing()` — turns an opened (Loaded) world editable via `rebuild_substrate`, keeping the saved climate |

`reference_apply_land_base` is a height commit: it pushes undo, checks the
`Mutation::HeightEdit` gate, and marks Height stale.

**Apply's bookkeeping.**

- A new `bake::Mutation` kind.
- An undo-ledger entry — a `plate_id` snapshot is 2 bytes per cell, about 5 MB at
  2048×1311.
- A Civ staleness mark with reason `"plates"`.
- **No new staleness graph node**; a test pins the graph at four stages.

**GDScript (presentation only)**

- A `_reference_layer` under the camera-transformed map node, above the LOD
  sprites and below `overlay`, placed from the Rust `Registration`.
- The opacity slider; handles via `ToolOverlay.set_handles`; river and coast
  previews via `set_path_preview`.
- JPEG and WebP decoding through Godot `Image`, with the long edge clamped
  **before** the pixels are handed to Rust.
- A new armed tool id, `"plates"`, alongside `"sculpt"` and `"paint"`.
- A display texture may be downscaled for display only; **every grid-space number
  comes from Rust.**

### 3.3 "Noise beta and warp" mapped onto existing parameters

| Control | Implementation | Status |
|---|---|---|
| **Edge warp** (0–1) | `compute_warp(gw, gh, edge_seed, edge_warp, world)` verbatim — same `×0.18×gw` amplitude and `<0.5` cutoff; large-scale bends | reuses a ported kernel |
| **Edge detail** (0–1) | a second warp octave built from the same `fbm` / `pfbm` at k× frequency and smaller amplitude, added to the first; jagged edges | new — question 1 |
| **Uplift spread** | the existing `tect.blur_r` | existing |
| Noise β | stays a height parameter; the UI hint points users to the sculpt features' own noise for surface roughness | unchanged |

### 3.4 Save format

All entries are MAY and additive: older readers ignore them (§6.3), and per the
precedent at `SAVEFILE_COMPAT.md` lines 660 and 2215 there is no `format_version`
bump.

| Entry | Holds |
|---|---|
| `annotations/reference.json` | source name, sha256, pixel size, registration, opacity, visibility, blend, detection settings, sampled colours (read by nothing downstream) |
| `annotations/reference.<png\|jpg\|webp>` | the original image bytes (question 5) |
| `drafts/reference_mask.json` | sparse mask corrections plus `gw` / `gh`; refused on grid mismatch, like `drafts/paint.json` |
| `drafts/plates.json` | the uncommitted `PlateLayout` |
| `entities/plates.json` + `rasters/plate_paint.u8` | the applied layout, so `continue_editing()` rebuilds exactly the substrate the user drew |

Spec rows go into `SAVEFILE_COMPAT.md` §5 and §8–12. A backward-compatibility test
opens a real archive saved by the prior format (`git show <sha>:…`) and
re-serialises it byte-identically.

### 3.5 Parity class and verification

| Piece | Class | Verification |
|---|---|---|
| Reference layer and registration | new, no reference ancestor | round trip and 2-point solve tests; a **windowed** opacity probe with a positive control; an alignment probe — image corner on cell (0,0) within 1 px at three zooms |
| Land and river detection | new | synthetic PNG fixtures with ground-truth masks (coast ink, blue labels on land, a compass rose, a hatched sea, parchment noise); IoU and precision/recall targets; every threshold mutation-tested; timing as median (min..max) |
| Base height from the mask | new, built on the ported `chamfer_dist` | literal sea-level assertions; undo restores byte-identically |
| Rivers into stamps | new detection feeding the golden sculpt River commit | stamps listed; discard removes them all; commit locks `river_mask` |
| Constrained plates | new; must equal `assign_plates` with no paint | byte-identity pin; `golden_parity_infer.rs` unchanged; field hash unchanged after Apply; painted cells keep their id at warp 0 |
| Edge roughness | `compute_warp` ported; the detail octave is new | identical below the cutoff; every input moves the output |
| §7d tag | `FUNCTIONAL_CONTRACT.md` row: "new capability (owner 2026-09-13), no reference ancestor" | |

### 3.6 Android, touch and memory

Arithmetic from the research pass, not measurements.

**Retained per world**

- Grid RGB: about 7.7 MiB at 2048×1311.
- Each u8 mask (land, river, coverage, plate paint): about 2.6 MiB.
- `plate_id` undo snapshot: about 5 MiB.

**Transient**

- A decoded source image at w×h×4: cap the long edge at 4096 on phone (≤ 64 MiB)
  and 8192 on desktop (≤ 256 MiB), and drop it once resampled.
- The display texture: capped at 2048 on phone and 4096 on desktop.
- Detection never runs during generation.

**Touch**

- One finger draws or paints; two fingers pan and zoom.
- Handles and plate sites are 44 dp on both axes.
- Hold-to-peek stands in for the T key.
- Point-pair registration is tap-then-tap with a magnifier loupe.
- Phone dialogs are proven only on real hardware.

### 3.7 GUI placement (needs an owner-approved canvas)

**WORLD ▸ SCULPT** gains two categories beside Terrain and Biomes:

- **Reference** — load, replace or clear; registration (Fit / Match points /
  numeric); opacity, visibility, blend; Detect land; Detect rivers; Apply base
  height.
- **Plates** — the Draw plates armed tool; a site list with crust and drift
  overrides; Edge warp; Edge detail; Uplift spread; Preview / Apply / Discard; a
  shortcut to view as Plates or Plate boundaries.

**WORLD ▸ PIPELINE ▸ Generate ▸ Import** gains "New world from reference map…".

This changes the owner's rail tree, so it needs owner approval. `command_index.gd`
EXTRAS and `shortcuts_dialog.gd` are updated in the same change as any menu move.

---

## 4 · Milestones (proposed)

| # | Size | Scope | Acceptance | Risks / non-goals |
|---|---|---|---|---|
| **RM-0** | M | `continue_editing()` — rebuild the substrate for an opened project and add a sculpt editor, keeping saved temperature and rainfall | `sculpt_commit` succeeds after open; heightmap and climate byte-identical before the commit | changes opened-world behaviour — keep it opt-in; the reference's seed replay on load is not verified as ported |
| **RM-1** | S | `reference.rs` registration and resampling; reference `#[func]`s | round trip and 2-point solve; bad bytes and zero size return Err without panicking; uncovered cells reported | non-goal: perspective or rubber-sheet warping |
| **RM-2** | M | viewport reference layer, opacity, visibility, handles | windowed opacity probe with positive control; alignment probe across zoom, pan and LOD | node placement against camera and LOD sprites; GUI done only with a canvas |
| **RM-3** | S | New world from reference (flat all-ocean import, identity registration) | aspect follows `heightmap_grid_h`; sculpt works immediately | reuses the featureless-import path |
| **RM-4** | L | land detection and mask editing | fixture IoU; area filter in km²; thresholds mutation-tested; timing median (min..max) | non-goal: OCR, roads, semantic labels |
| **RM-5** | M | land mask → base height commit | literal sea-level assertions; undo byte-identical; recompute runs | refused on a finalized world |
| **RM-6** | M | river detection → River stamps in the draft | fixture precision/recall; stamps discardable; commit locks channels | skeleton branches in `trace_boundaries`; downhill direction needs RM-5 |
| **RM-7** | M | `PlateLayout`, `assign_plates_constrained`, `infer_tectonics_with_plates`, plate `#[func]`s; Apply = substrate only | byte-identity pin without paint; goldens green; field hash unchanged; civ stale reason `"plates"` | CPU only; u8 paint caps at 255 plates (the limit is 40) |
| **RM-8** | S | Edge warp, Edge detail, Uplift spread | identical below cutoff; each input moves the output; deterministic | Edge detail needs a ruling (question 1) |
| **RM-9** | M | save and restore of every §3.4 entry, plus spec rows | prior-format archive round trips byte-identically; grid mismatch refused; each entry alone survives a round trip | image size inside the archive (question 5) |
| **RM-10** | L | GUI per the approved canvas: desktop dock, phone sheet, command index | parse checks; `cargo test --workspace`; touch-floor walk on both axes | blocked until a canvas is approved |
| **RM-11** | M, optional | "Grow relief from plates": orogeny along drawn boundaries (`trace_boundaries` → `build_orogeny_field` → `smooth_orogeny`) as an undoable height edit, off by default (§7g) | off: output unchanged; on: height rises only near convergent drawn boundaries | question 3 |

**Order.**

- RM-1 first, then RM-2, RM-3 and RM-4 in parallel.
- RM-5 follows RM-4, and RM-6 follows RM-5.
- RM-7 → RM-8 → RM-11.
- RM-0 and RM-9 once RM-1 and RM-7 exist.
- RM-10 last.
- The engine milestones can start without the canvas.

---

## 5 · Owner questions — ANSWERED 2026-09-13 (see `LARGE_ITEM_RULINGS.md`, Ruling M)

**The answers change the design:** plates are drawn as **boundary lines** (not sites); drawn plates **only inform the resources step** and never change height (RM-11 dropped); an opened project becomes **editable automatically** (RM-0 without an action); the image is **embedded** in the save; registration is scale/rotation/offset only; v1 detection is land and rivers; the reference is never exported; the placement/canvas question is the **first step when the work starts**. On edge roughness the owner suggested *"Tectonic alpha might be it"* — checked at the symbol: `tect.alpha` only scales tectonic height in `compute_height`, so it is not an edge control; with drawn lines, roughening is a warp-style displacement of the lines (open: confirm). The original questions and defaults follow for the record.

### The questions as asked

1. **"Noise beta and warp" for plate edges.** Beta roughens height, not edges, and
   warp only bends edges at a large scale.
   *Default:* Edge warp (the generator's exact warp), plus a new Edge detail octave
   built from the same noise, plus the existing Uplift spread.
2. **How are plates drawn?**
   *Default:* tap to place sites; optionally paint regions; unpainted cells go to
   the nearest site through the warp. The alternatives are boundary lines or lasso
   polygons.
3. **Do drawn plates change the sculpted height?**
   *Default:* no — they rebuild only the substrate. Growing relief is the opt-in
   RM-11.
4. **Plate crust and drift.**
   *Default:* inferred from the sculpt (the existing `classify_plate_crust` and
   `infer_plate_velocities`), with optional overrides per plate.
5. **Reference image storage.**
   *Default:* embed the original bytes in the project zip, with a warning above
   about 50 MB. The alternative, an external path, is fragile on Android.
6. **Registration depth.**
   *Default:* scale, rotation and offset via fit, corner drag or a two-point match.
   No perspective or rubber-sheet warping.
7. **Placement and canvas.**
   *Default:* WORLD ▸ SCULPT ▸ Reference and ▸ Plates, plus Import ▸ New world from
   reference map. Commission the canvas now, so the engine milestones can proceed
   alongside it.
8. **Editing opened projects (RM-0).**
   *Default:* an explicit "Continue editing" action that rebuilds the substrate and
   keeps the saved climate, rather than promoting worlds automatically on open.
9. **v1 detection scope.**
   *Default:* land/water and rivers only — no roads, text, or biome-colour
   classification.
10. **Image formats.**
    *Default:* PNG, JPEG and WebP decoded by Godot; PNG also through Rust; source
    capped at 4096 px on phone and 8192 px on desktop.
11. **Export.**
    *Default:* the reference image is never drawn into any export, as in
    Dungeondraft.

**Critical files for implementation:**

- `cartalith-native/crates/cartalith-engine/src/import.rs`
- `cartalith-native/crates/cartalith-terrain/src/lib.rs`
- `cartalith-native/crates/cartalith-godot/src/lib.rs`
- `cartalith-native/godot-project/shell/viewport_host.gd`
- `cartalith-native/godot-project/shell/workspaces/world_workspace.gd`

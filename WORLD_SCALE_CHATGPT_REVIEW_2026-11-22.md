# World-scale terrain: a review of ChatGPT's "regenerate a region from the source world" proposal (2026-11-22)

**Why this exists.** On 2026-11-22 the owner supplied a ChatGPT reply about regenerating a selected region of a
world from its seed at a higher resolution, and asked for a thorough analysis: ChatGPT knew only the legacy HTML
(v2.71), but *some of it may have merit*. This file checks each of its claims against the port's code, measures the
one claim that was cheap to measure, and proposes edits to `WORLD_SCALE_TERRAIN_PLAN.md`. Those edits are **not
applied**.

This file **defines and explains; it does not track**. Status is `cartalith-native/docs/STATUS.md`'s. It changes no
code.

**Tags.**

- `[read]`: read today at the named symbol, or in the named document.
- `[measured]`: produced in this session by the scratch run in §4. It was removed afterwards.
- `[recalled]`: from memory, not checked here.
- `[inference]`: reasoned from the above, not observed.

Numbers quoted from another document are `[read]` and name that document. They were not re-measured.

---

## 1. The proposal, condensed

The full text is the owner's paste; this is its shape.

1. Terrain is a point function: `H(x,y) = F(S, P, x, y)`. A region is the same function at offset coordinates.
2. Preserve the seed, the plate definitions and boundaries, the map dimensions, the global coordinates, the
   generation parameters and a generator version.
3. In v2.71, `buildTectonicSubstrate()` rebuilds the substrate from the saved seed. `amplifyRegion()` refines a
   region from the coarse field. "Extract as new world" then infers fresh tectonics instead of re-running the source
   simulation, and ChatGPT calls that the gap.
4. Add a "Regenerate region from source world" mode. It reconstructs the source geology and evaluates it at the
   destination resolution.
5. The same seed is not the same terrain at a new resolution, because fields are built at `GW × GH`. The fix it
   proposes is resolution-independent world coordinates in `[0,1]²`. Stages such as flexure, stress, boundary
   distance and blur need context outside the rectangle.
6. Keep a source-world manifest. It holds the seed and PRNG version, the generator version, the dimensions and
   extent, the tectonic parameters, the plate geometry, the settings, and the region bounds.
7. It offers two fidelity levels: **A**, which keeps the original heights and adds detail between them, and **B**,
   which re-runs the simulation at a higher resolution. It recommends A first. It then asks the owner which one.

---

## 2. Claim by claim

| # | Claim | Verdict | Evidence |
|---|---|---|---|
| 1 | Generation is deterministic from seed + params | **True of the port** | See §2.1. |
| 2 | Fields depend on the grid, so the same seed at a new resolution is not the same terrain | **True of the port** | See §2.2. It is measured: §4. |
| 3 | Resolution-independent `[0,1]²` world coordinates fix it | **False as a fix**. The coordinate convention it asks for is already the port's | See §2.3. |
| 4 | Context outside the rectangle matters | **Already planned** (`WORLD_SCALE_TERRAIN_PLAN.md` §6, §9). The plan's margin is too small for the substrate fields | See §2.4 for the halo of each stage. |
| 5 | Keep a source-world manifest with a generator version | **Already planned** in name ("rules version", plan §8 and §9). Not designed or built. Three members are new | See §2.5. |
| 6 | Fidelity A versus B | **Already measured**. A is built and has a measured ceiling. B moves the rivers | See §2.6 and §4. |
| 3a | v2.71: `buildTectonicSubstrate()` rebuilds the substrate from the saved seed on load | **True only of the HTML** | See §2.7. |
| 3b | v2.71: "Extract as new world" amplifies, then infers fresh tectonics | **True of the port too** | See §2.7. |

### 2.1 Claim 1: determinism

- **Measured.** `generate_terrain` run twice with the same `WorldParams` gave a bit-identical `field` in **18 of 18**
  pairs. That covers two extents, three seeds and three widths (§4). `[measured]`
- **"Params" has to mean every input.** The flat `params.json` → `cartalith` map is written from every row of `PARAMS`
  (`cartalith_godot::params::save_state`). It includes `use_gpu` and every ruled divergence switch. `[read]`
- **The CPU and GPU paths are two different worlds for one seed.** Each is the same world on every device measured
  (PC, 6T, Pad 3; `PERFORMANCE_BENCHMARKS.md`, the digest table under §12 and the 16k table in §13). `[read]`
- **Thread count.** Independence from thread count was measured for the ZR-0 tile code (`ZOOM_REFINEMENT_SCOPE.md`
  §8). It is not proven here for `generate_terrain` as a whole. `[read]`
- **The code version is part of "params", and nothing records it** (§2.5). `[read]`

### 2.2 Claim 2: the fields depend on the grid

Each of these is read at its symbol. Each makes the same seed give a different world at another width. All are
`[read]` unless marked.

| Term | Symbol | Unit | Effect of doubling the width |
|---|---|---|---|
| Tectonic blur radius | `TectonicParams::blur_r` (18.0 in `WorldParams::defaults`), consumed by `compute_stress` and `compute_flexure` (×3) through `gauss_blur`. It is also the source of `plate_base_blur_r` | **cells**. The app never rescales it with resolution (the `params.rs` `PARAMS` row is a plain dial) | The blur covers half the map fraction it did |
| Relief-noise frequency | `cartalith_terrain::terrain_detail_k(gw, km)`, which feeds `HeightParams::nf` and the heterogeneity frequency | km per cell | On a map finer than 800 km / 2048, more noise frequency per map width |
| Routing roughness | `cartalith_hydrology::rough::ROUGH_WAVELENGTH_CELLS` = 4, in `Roughness::with_params` | cells | The lattice is half the ground size, so receivers are tie-broken differently |
| Feature radius cap | `clamp_feature_radius_cells` (`FEATURE_RADIUS_MAX_FRAC` 0.12 of the shorter axis) | grid fraction | Unchanged on one world. A cut-out regenerated as its own map caps its volcanoes at 12 % of *the cut-out* |
| Volcano sites | `stamp_volcanoes_provinces_traced`: `conv[(rng * conv.len()) as usize]` | an index into a list of cells | The list is about twice as long, so the same draw lands on a different cell `[inference]` |
| Plate centres | `build_plates`: a Lloyd relaxation over every grid cell | discrete centroids | Sub-cell shifts. The plate-sign masks agree at 98.9-99.9 % across widths (`NEW_WORLD_PREVIEW_PROPOSAL.md` §4.2) |
| Iterations and passes | `StreamParams::iters` (15), `ErosionPassParams::{velo_iters 60, glacial_passes 8, coastal_passes 4, diffuse_passes 6}` | iterations per cell | A fixed count reaches half as far in map fraction |
| Depression-fill tilt | `build_routing_surface_with`: one `next_up` (ULP) per ring, or 5-7 per step for `FillTilt::Floor` | ULPs per cell step | The rise grows with path length in cells |
| Climate | `simulate_weather_with` and the moisture correctors run on a `gw.min(240)`-wide coarse grid | map fraction above 240 | **Not grid-dependent above 240 wide**: the opposite of the claim, for this stage |

**The port already knew this and corrects for it once.** `cartalith_engine::preview::preview_params` scales
`tect.blur_r` and the map width by the preview's width ratio. Its module doc records that both corrections are
load-bearing, by mutation. The preview proposal's own table (`NEW_WORLD_PREVIEW_PROPOSAL.md` §4.2) measured
land/ocean agreement against the finished world: **naive 87.6-93.9 % median** and **corrected 97.6-98.2 %**. `[read]`

### 2.3 Claim 3: `[0,1]²` world coordinates as the fix

**The coordinate system already exists.** The point-evaluable terms of the port's substrate are already written in
map fractions. Each of these is `[read]`:

- `build_plates` seeds at `rng.next_f64() * gw`;
- `compute_warp`'s frequency is `3.0 / gw` (world) or `2.5 / gw` (region);
- `compute_height`'s noise reads `wx * p.nf / gw`;
- `generate_continentality_field` runs on its own coarse grid in fractions.

The display-side elevation query is the same idea, finished: `cartalith_engine::elevation::world_elevation_tile` and
`cartalith_terrain::amplify::sample_elevation` (EF-0) evaluate at any texel in continuous coarse coordinates, with
bit-identical seams (`ZOOM_REFINEMENT_SCOPE.md` §1.4, §8). `[read]`

**Why it is not a fix** `[read]` + `[inference]`. Four kinds of term block "evaluate `F` at offset coordinates", and a
coordinate change touches none of them:

1. **Parameters in cells.** These are the rows of §2.2. Expressing them in ground units is a real step, and §4
   measures what it buys.
2. **Discrete algorithms on the grid:** Lloyd centroids, RNG draws from cell lists, D8 routing, the Priority-Flood
   fill and the stream-power solver. None of them is a function of `(x, y)`.
3. **Global normalisers.** `normalize_field` maps the whole raw field by its own global min and max. `compute_flexure`
   divides by the global maximum `|broad|`, and `build_age_field` divides by the global maximum distance. Under World
   Structure, `apply_world_structure_sea_level` re-anchors the sea on the whole field. **Every height in a region
   depends on the extremes of the whole world.** ChatGPT does not mention these.
4. **Coupled global processes:** drainage, depression filling, erosion and climate advection (§2.4).

**Which stages are point-evaluable** (the brief's list):

| Stage | Point-evaluable? |
|---|---|
| Warp, heterogeneity, the relief-noise term of `compute_height`, temperature (latitude + height), the roughness lattice | yes (once its cell-unit parameter is converted) |
| Plate assignment | yes, given the global plate list (at most 40 points) |
| Lloyd relaxation | no: global, iterative over every cell |
| Stress, flexure, plate-base blur | bounded halo plus a global normaliser |
| Boundary-distance age | no: unbounded distance plus a global normaliser |
| Stream power, Priority-Flood fill, flow accumulation | no: global and iterative, with an unbounded basin |
| Climate wind and rain sweeps | no: global, iterative on the coarse grid |

### 2.4 Claim 4: the halo each stage needs

The support follows from `gauss_blur`. It runs three box passes of half-width `pr = js_round(r / 1.6)`, so its
support is `±3·pr` cells. The figures assume the defaults:

- a 40 075 km / 2048-wide world: 19.6 km per cell;
- a 1 200 km / 1024-wide region: 1.17 km per cell.

All cell counts are `[read]` from the symbols; the km figures are arithmetic on them.

| Stage (symbol) | Halo in cells | World (km) | Region (km) | Also global? |
|---|---|---|---|---|
| Warp, height noise, heterogeneity | 0 (point) | 0 | 0 | no |
| Plate assignment (`assign_plates`) | 0, given the plate list | 0 | 0 | the plate list (`build_plates`, Lloyd over every cell) |
| Plate-base blur (`plate_base_blur_r` = max(2, 18 × 0.18) = 3.24, so `pr` 2) | ±6 | ±117 | ±7 | no |
| Stress (`compute_stress`, `gauss_blur(blur_r 18)`, so `pr` 11) | ±33 | ±646 | ±39 | yes: `normalize_by_abs_max` over the whole field |
| Flexure (`compute_flexure`, `blur_r × 3` = 54, so `pr` 34) | ±102 | ±1 996 | ±120 | yes: divided by the global maximum |
| Age (`build_age_field`, a chamfer distance to the nearest boundary) | unbounded (up to the plate's interior radius) | — | — | yes: divided by the global maximum distance |
| Normalisation (`normalize_field`) | — | — | — | **yes: the global min and max** |
| Volcano stamps (`clamp_feature_radius_cells`) | up to 12 % of the shorter axis: 123 cells on the world | ±2 400 | — | site draws from global cell lists |
| Depression fill (`build_routing_surface_with`) | unbounded: the spill path to the sea or the map edge | — | — | yes |
| Flow accumulation, stream power (`stream_power_kernel`) | unbounded: the whole upstream basin | — | — | yes |
| Climate (`simulate_weather_with`, `w_iters` 70 on the `min(gw, 240)` grid) | the whole map, in practice | — | — | yes. **On a non-wrapping map the upwind edge is reset to ocean humidity** (the `!wrap_x` edge loop) |

**What this does to the plan's §9 margin** `[inference]`. Take the plan's example cut-out of 400 km on a
40 075 km / 2048 world. Its 15-25 % margin is 60-100 km, which is **3-5 parent cells**. Stress needs ±33 parent cells
and flexure ±102. Even the 1 200 km region, at 85 cells for 100 km, is short of flexure's 120 km. **So the margin
cannot be what supplies the substrate.** The substrate fields, the normalisers and the climate have to come from the
parent world as given inputs. The margin serves only the fine-grid erosion and drainage, which is what the plan uses
it for. The climate point is sharper still. A cut-out run as its own non-wrapping map would be fed ocean-humid air at
its upwind edge. A rain shadow cast by a range outside the cut-out would then vanish. The plan's "interpolate
climate" is the right call, and this is why.

### 2.5 Claim 5: the manifest and the generator version

What a save carries today (`SAVEFILE_COMPAT.md` §7, §8.3, §13.1; all `[read]`):

- **`project.json`:**
  - `format_version`;
  - `generator` = `format!("cartalith-native {}", env!("CARGO_PKG_VERSION"))` in `cartalith_io::project`, and
    every crate is at `0.1.0`;
  - `world.{grid_width, grid_height, wrap_x, map_width_km, sea_level, seed, origin, name}`;
  - the `substrate` member, with its own `version`.
- **Rasters:** the heightmap (MUST), climate, and the fourteen substrate rasters: `plate_id`, `boundary_mask`,
  `boundary_type`, `stress_field`, `shear_field`, `age_field`, `crust_field`, `flow_discharge`, the channels and the
  river lock (Ruling AR).
- **`params.json`:** every parameter, including the 56 rows of `cartalith_godot::params::LOADS_WHEN_ABSENT` (Ruling
  EV). Those rows make an old save regenerate the world it was made with, one switch at a time.

**So most of ChatGPT's list is already stored.** The seed, the parameters, the dimensions, the extent, the plates as
grids and the switches are all there. The plate *definitions* (centres, velocities) are not stored. They do not need
to be: `tectonic_substrate` replays them exactly at the parent's grid (§2.1).

**What is missing** (`[read]` that each is absent; that each matters is `[inference]`):

1. **A version that means anything.** `generator` is provenance that "no reader may branch on". It is `0.1.0`
   forever. **The switch rows cover only switched changes.** An unswitched change of generated output reaches every
   regenerate with no record. One example: Ruling EZ item 3 changed which world the GPU path makes, for every seed,
   and kept no switch (`PERFORMANCE_BENCHMARKS.md` §12, the "Section 7p" paragraph). The ZR scope designed exactly
   this for tiles: `ZR_RULES_VERSION` beside a committed golden hash of a fixture (`ZOOM_REFINEMENT_SCOPE.md` §4.3).
   **Nothing like it exists for `generate_terrain`.**
2. **A digest of what was generated.** A rules hash already exists: `bake_bridge::world_key_signature` hashes
   `params::world_key_state`. It is computed for the atlas key and never saved. **No hash of the height field is
   saved either**; the ZR inventory notes the world key has none. So a regenerate-from-source cannot check that it
   reproduced the source.
3. **A parent link on a region world.** `region_new_world` (`ops_bridge.rs`) writes only `origin: "region"` and the
   parent's seed (`absorb(..., ORIGIN_REGION)`). It keeps none of the parent's identity, its grid, or the selection's
   bounds in it. A region world cannot be traced back to its source, regenerated from it, or snapped back into it.
   Grepping `crates/` for a parent or source-world member found none.

**A hazard this exposes.** A region world carries the parent's seed but its own grid and extent. Plate positions are
`rng * gw` in that frame (`build_plates`). A *Generate* on such a world therefore builds an unrelated world, not its
parent's geography `[inference]`. Whether the shell guards against it was not checked.

### 2.6 Claim 6: fidelity A versus B, mapped onto what the port has

| ChatGPT | What the port has | Status and measured ceiling |
|---|---|---|
| **A**: exact heights kept, detail added between them | `amplify_region` (bilinear coarse constraint plus world-space detail, relief-tapered, with Ruling O's sea guard) behind `File ▸ New world from selection` (`region_as_new_world`), and the same composition per LOD tile (`bake::pyramid_tile_padded_rough`, Ruling FG's law) | **Built.** Its limit is measured. FG met B6-B8 but not B1, B4 or B5 (plan §3): *"a 19.6 km cell holds no information about 1 km valleys. Noise can follow the terrain; it cannot create landforms."* `[read]`. Also: A does **not** keep the original heights exactly. The detail is added on top of the bilinear field (`±detail_amp/2`, tapered), so a coarse sample's own height moves too `[read]` |
| A plus local processes, per tile | ZR-0's prototype (`tests/zr0_refine.rs`) | **Measured no-go.** `bind_r` failed 5 of 6 cells (0.10-0.25 against native 0.47-0.59). Drainage, burn and erosion added 4-12 % over noise (`ZOOM_REFINEMENT_SCOPE.md` §8) `[read]` |
| **B**: re-run the simulation at a higher resolution | No code. The nearest thing is generating the same seed at a larger width, which is what §4 measures | **Measured here.** The coarse shape holds. The river network moves: about half of the major river cells are no longer within one coarse cell of where they were (§4) `[measured]` |
| *(not in ChatGPT)* a constrained regeneration | Plan §9: the parent's coarse fields over the cut-out plus a margin as the initial shape and boundary conditions, full erosion over the block, a crop, civ copied 1:1 | **Planned, not built, not measured** (plan §11 step 3) `[read]` |

ChatGPT's dichotomy leaves out the third row, and the third row is the port's plan.

**Why ZR-0's reasons bear on the choice** `[read]` + `[inference]`:

- ZR-0 failed because per-tile steps cannot grow structure at 5-50 km from a smooth cell.
- Pure B re-runs a *substrate*. At a finer grid the substrate adds noise octaves (`terrain_detail_k` rises), not
  valleys. The valleys come from erosion and drainage, which B would also re-run, so they come out different (§4).
- A block-wide constrained regeneration is the one form that can grow valleys and still keep the parent's rivers:
  plan §9.3's *"the reason this differs from ZR-0's per-tile failure"*.

### 2.7 The v2.71-specific claims

- **3a, substrate replay on load: true only of the HTML.** v2.71's load path calls `await buildTectonicSubstrate()`
  with the comment *"exactly reconstruct plates/plateId/stress/…/orogeny from the saved seed+tect params — none of
  these depend on `field`, so this is an exact replay"* (`reference/Cartalith v2.71 DCC test.html`, search the
  string). The port does not replay. Since Ruling AR it stores the substrate rasters and restores them (`SAVEFILE_COMPAT.md`
  §8.3). The port's `cartalith_terrain::infer` module doc describes the reference's replay as "a different, exact
  path". `[read]`
- **The replay is exact only at the same grid.** ChatGPT extends "exact" to a new grid. §2.2 and §4 show that it
  does not carry over. `[read]` + `[measured]`
- **3b, extract = amplify, then infer tectonics: true of the port too.** v2.71's `regionNewWorldBtn` handler runs
  `amplifyRegion` and sets `_canInvert`/`_imported`. Per the port's `region_as_new_world` doc, the reference's
  calibrate commit (`_suCalCommit`) then runs `inferTectonics()`. The port's
  `region_as_new_world` collapses that gate. It calls `amplify_region`, then `cartalith_engine::import::infer_tectonics`,
  in one call, and its doc says so. `infer_tectonics` is a proxy from the heightmap's morphology (the
  `cartalith_terrain::infer` module doc: *"A world imported this way will not match any world `generate_terrain`
  would produce from a seed"*). `[read]`

---

## 3. What has merit and is new to the port's plan

Each item names what the plan lacks today.

1. **A generation-rules version for `generate_terrain`, with a fixture-hash guard.** The plan's §8 names "rules
   version" and never designs it. Copy `ZOOM_REFINEMENT_SCOPE.md` §4.3's pattern:
   - a `GENERATION_RULES_VERSION` literal beside a committed hash of a small fixture world;
   - any change to generated output moves the hash, and the fix is to bump both;
   - mutating the version alone must also go red;
   - the save writes the version.

   This closes the hole the switch rows leave: unswitched changes such as Ruling EZ item 3.
2. **A generation digest in the save.** Store a hash of the generated `heightmap` (and of `flow_discharge`) at
   generate time, beside `world_key_signature`'s rules hash. A regenerate-from-source then checks itself before it
   trusts its output: *same rules version + same rules hash + same digest = the source world*. Without the digest,
   "deterministic" is a promise nobody verifies.
3. **A source-world manifest on region worlds.** Add a `world.region_source` object to `project.json`: MAY, additive,
   no `format_version` bump, under §6.3's unknown-entry rule. It holds:
   - the parent's seed, rules version, rules hash and digest;
   - the parent's `grid_width` × `grid_height`, `map_width_km` and `wrap_x`;
   - the selection in parent cells **and** in map fractions.

   This is ChatGPT's best point. It is what lets the plan's §9 regeneration, and its §9.5 snap-back, find their
   source.
4. **Name the regeneration a mode.** Call it "Regenerate region from source", beside today's "New world from
   selection". The two map onto A and plan §9; giving them distinct names stops the two being confused. **It is a
   world-construction mode, not a third save type.** The plan's two save types (§8) stay. The full save gains items
   1-3.
5. **Carry the global normalisers from the coarse pass** (§2.3 item 3). These are `normalize_field`'s min and max,
   the stress and flexure maxima (`normalize_by_abs_max`, `compute_flexure`), the age normaliser and the World-Structure sea anchor. They are not new as an
   idea, since the plan's §6 already gives global stages a coarse pass. They are **missing from its table**, and they
   are the cheapest global state to carry: a handful of scalars.
6. **A cell-unit census as the first step of the tiled-pipeline scope.** §2.2's table is the start of the
   "stage-by-stage locality census" in plan §11 step 4. It also bears on plan §5 direction 1 (*"generate the base at
   16k-32k"*). **A 16k generate of a seed is not a higher-resolution copy of the same seed at 2k**, because `blur_r`
   is in cells. §4 measures the cost.

---

## 4. What is wrong or naive, measured where cheap

### 4.1 The measurement

**Purpose.** What does option B do to a region? The same seed and the same parameters were run at two widths, and a
central cut-out was compared.

**Harness.** A scratch binary in `C:/cw/wsreview` (removed after). It called `cartalith_engine::generate_terrain` on
`cartalith_godot::params::defaults()`, the app's own parameters, included by path as `nwprev_cost.rs` does. The
settings:

- `use_gpu` off;
- six plates for a 1 200 km region and 36 for a 40 075 km world;
- grid heights as `nwprev_cost.rs`'s `grid_h`.

**Arms.**

- **naive:** the same parameters at both widths.
- **scaled:** `tect.blur_r` × the width ratio. This is `preview_params`' correction 1. The map width is held, so the
  cell size really changes, and nothing else is converted.
- **negative control:** the fine world of the next seed.

**Window and metrics.** The fine world is block-reduced onto the coarse grid. Metrics are taken in the central
40 % × 40 % window:

- land/ocean agreement;
- Pearson r of sea-relative height;
- the median |Δh| over the window's 5-95 % relief;
- **rivers:** the share of major-river cells (the land cells at or above the 0.98 quantile of discharge; block
  maximum for the fine world) that have a river in the other world within one coarse cell, counted both ways.

**Runs.** Seeds 1200, 42 and 51127, at 512 vs 1024, 1024 vs 2048 and 512 vs 2048. That is 18 comparisons per arm,
one run each. **Times are not reported**: the coordinator's APK build was running on the same machine.

| Arm | Land agreement | Height r | median \|Δh\| / relief | Rivers within one cell |
|---|---|---|---|---|
| naive | 95.8 % (87.9..98.8) | 0.959 (0.654..0.988) | 0.065 (0.024..0.180) | **25.5 % (0.0..49.6)** |
| scaled | 98.6 % (90.7..99.6) | 0.994 (0.720..0.999) | 0.010 (0.003..0.059) | **49.7 % (7.8..82.1)** |
| negative control | 33.7..59.0 % | -0.333..0.428 | 0.279..0.605 | 0.0..8.1 % |

Each cell is the median (min..max) of 18. `[measured]`

**Reading it** `[measured]` + `[inference]`:

- **The broad geography survives a resolution change once `blur_r` is in ground units.** The median r is 0.994 and
  median land agreement is 98.6 %. That matches the preview proposal's land-only result and extends it to heights.
- **It does not survive at the cell scale that matters for a cut-out.** Even corrected, half the major river cells
  move by more than one coarse cell, and the worst case keeps 7.8 %. The worst height case (region, seed 42, 1024 vs
  2048) falls to r 0.72. Settlements are bound to real river and coast geometry (Ruling N), and plan §9.4 copies them
  1:1. **So B, a free re-run, breaks the 1:1 copy.** The river network is decided by drainage, and drainage is
  re-decided at every resolution.
- **Converting `blur_r` closes most of the naive gap but not all of it.** The other cell-unit terms of §2.2 are still
  unconverted: roughness, iteration counts, fill ULPs and cell-list draws.

**Not measured:**

- the GPU path;
- World-Structure archetypes;
- volcano and crater positions on their own;
- any cut-out regenerated as its own map. Its plates would sit in a different frame (§2.5), so it would be an
  unrelated world `[inference]`;
- more seeds.

### 4.2 What is wrong or naive in the proposal

1. **`H(x,y) = F(S,P,x,y)` is false for this pipeline** `[read]`. Only the substrate's noise terms are point
   functions. The terms that give a map its character are global or coupled: the fill, drainage, erosion and climate,
   plus the normalisers of §2.3.
2. **"Evaluate the world-space terrain functions at the destination resolution" adds noise, not landforms**
   `[read]` + `[inference]`. A finer substrate raises `terrain_detail_k`'s frequency. The landforms the owner wants
   come from erosion and drainage (plan §3, ZR-0). Re-running those is B, and B moves the rivers (§4.1).
3. **"Reconstruct plates, stress, flexure at the higher resolution" buys nothing** `[inference]` from the §2.4 halos.
   Those fields are smooth on scales of 33-102 parent cells. Upsampling the parent's stored rasters gives the same
   fields. A re-run gives the same fields plus sub-cell boundary shifts and different volcano draws.
4. **"Build A first": A is already built, and its ceiling is measured** `[read]`. See `amplify_region`, FG, ZR-0 and
   the plan's §3.
5. **"A keeps the original heights identical" is not how `amplify_region` works** `[read]`. The detail is added over
   the bilinear field.
6. **"B: local elevation may change" understates it** `[measured]`. The drainage moves, and with it everything bound
   to rivers.
7. **The manifest's "plate definitions and geometry" is the wrong thing to add** `[read]`. The substrate replays
   exactly at the parent's grid, and the port stores its rasters anyway. What is missing is the version and the
   digest that make a replay trustworthy (§3 items 1-2).
8. **The coordinate fix misses the four blockers of §2.3.** The global normalisers are the easiest to miss and the
   cheapest to carry.

**What it got right:**

- the seed is not sufficient;
- the source world must stay the reference context;
- a versioned profile is needed;
- the extraction mode should be named and kept separate from today's "Extract".

---

## 5. Recommendation: proposed edits to `WORLD_SCALE_TERRAIN_PLAN.md`

These are **not applied**. Each one is written to be pasted in.

**Edit 1: §6, a row in the two-level table, after "Global".**

> | Global normalisers | `normalize_field`'s min/max, `compute_flexure`'s and the stress maxima, `build_age_field`'s maximum distance, the World-Structure sea anchor | computed once by the coarse whole-world pass and passed to every tile and block as constants; never recomputed over a tile, a block or a cut-out (`WORLD_SCALE_CHATGPT_REVIEW_2026-11-22.md` §2.3) |

**Edit 2: §6 "Requirements", a new bullet.**

> - Cell-unit parameters (`tect.blur_r`, `ROUGH_WAVELENGTH_CELLS`, the erosion iteration and pass counts, the fill
>   tilt's per-step rise, cell-list RNG draws) are expressed in ground units or map fraction for the new pipeline,
>   behind a switch (§7p), with the reference width 2048 so default-resolution worlds stay byte-identical. Measured
>   for `blur_r` alone (review §4.1): river agreement across a resolution change 25.5 % → 49.7 % median, height r
>   0.959 → 0.994. Until then, a 16k generate of a seed is a different world from its 2k generate.

**Edit 3: §8, a "Provenance" row in the save table, and a paragraph under it.**

> | Provenance | `generation_rules_version`, the rules hash (`world_key_signature`), a digest of `heightmap` and `flow_discharge`; on a region world, `world.region_source` (parent seed, rules version, rules hash, digest, grid, extent, wrap, bounds in parent cells and map fractions) | the same | |
>
> The rules version follows `ZOOM_REFINEMENT_SCOPE.md` §4.3's pattern for `generate_terrain`: a literal beside a
> committed hash of a fixture world, both bumped together. The switch rows (`LOADS_WHEN_ABSENT`, Ruling EV) cover
> switched changes only; the version covers the rest. A regenerate compares version, rules hash and digest before it
> claims to reproduce a world.

**Edit 4: §9, the mode named, with what is not re-run and why.** Insert after the current paragraph that opens
"Today":

> This is fidelity A (`amplify_region` + `infer_tectonics`), and it stays as **New world from selection**. The
> regeneration below is a separate mode, **Regenerate region from source**. It **does not re-run the substrate**: the
> plates, stress, flexure, age, crust and the global normalisers come from the parent's stored rasters, upsampled.
> Their supports (stress ±33, flexure ±102 parent cells at `blur_r` 18) exceed any margin, so the margin cannot
> supply them. Climate is interpolated from the parent's grids, not re-simulated: a non-wrapping block resets its
> upwind edge to ocean humidity (`simulate_weather_with`), which would erase rain shadows cast from outside it. A free
> re-run at the finer grid (fidelity B) was measured to move about half of the major river cells by more than one
> parent cell (review §4.1), which is incompatible with step 4's 1:1 copy.

**Edit 5: §9 step 1, add the manifest to the inputs.**

> … plus the parent's `region_source` manifest (§8), so the result records its parent and can be checked against it
> and snapped back (step 5).

**Edit 6: §11 step 1, a note.**

> State which cell-unit correction the 16k run applies. With defaults, a 16k world is not the 2k world of the same
> seed at a higher resolution (§6 Requirements).

**Edit 7: §11 step 4, the census seed.**

> The stage-by-stage locality census starts from `WORLD_SCALE_CHATGPT_REVIEW_2026-11-22.md` §2.2 (cell-unit terms)
> and §2.4 (halo per stage).

Nothing else of the proposal should go into the plan. The plan's §6, §8 and §9 already hold its sound core in a
stronger form.

---

## 6. Owner questions, each with a default

**Answered by Ruling GV (owner, 2026-11-22):** Q1 *"Also offer free B"* (as a proof of concept, §8); Q2-Q6 the
defaults below. The edits of §5 are applied to `WORLD_SCALE_TERRAIN_PLAN.md` with the ruling.

**Q1 (ChatGPT's own question): keep the original heights identical with detail between them (A), or re-run the
simulation at a higher resolution (B)?**

*Recommendation and default:* neither as stated. Keep **A**, already built, as **New world from selection**.
**Build plan §9's constrained regeneration** as **Regenerate region from source**:

- the parent's coarse geography is kept;
- real landforms are grown by full erosion over the block;
- rivers are pinned by the parent's inflow, outlets and water levels;
- the substrate and climate are taken from the parent, not re-simulated.

**Do not build free B.** It moved about half of the major river cells in §4.1, and that breaks the 1:1 copy of
towns. Plan §11 step 3's cut-out measurement is the go/no-go for the constrained mode.

**Q2.** Write a `world.region_source` manifest on every region world (parent seed, rules version and hash, digest,
grid, extent, bounds)?

*Default:* yes. It is MAY and additive, with no `format_version` bump.

**Q3.** Add a `generation_rules_version` with a fixture-hash guard, and a field digest, to every save?

*Default:* yes. Build it before the tiled pipeline, because every later regenerate leans on it.

**Q4.** Convert the generator's cell-unit parameters to ground units for new worlds, behind a switch, so that a seed
gives the same world at any resolution?

*Default:* yes, scheduled with the tiled-pipeline scope (plan §11 step 4), not before. The reference width is 2048,
so default-resolution worlds stay byte-identical, and any other width is a disclosed re-baseline under §7p.

**Q5.** Is "regenerate from source" a third save type?

*Default:* no. It is a world-construction mode. The two save types of plan §8 stay, and the full save gains Q2 and Q3.

**Q6.** A region world keeps its parent's seed, so a Generate on it builds an unrelated world (§2.5). Should Generate
on a region-origin world warn, or offer "Regenerate region from source" instead?

*Default:* warn now, if the shell does not already (not checked). Offer the mode once it exists.

---

## 7. Method and scope

- **Read:**
  - `MISTAKES.md`'s preflight;
  - `WORLD_SCALE_TERRAIN_PLAN.md`;
  - `ZOOM_REFINEMENT_SCOPE.md` §1, §4.3 and §8;
  - `WORLD_EMBOSS_FINDINGS_2026-11-15.md` §2;
  - `TERRAIN_COMPUTE_METHODS_RESEARCH.md` §2, §5 and its corrections;
  - `DECISIONS.md` §7p;
  - `LARGE_ITEM_RULINGS.md` (FG, FH and the region row);
  - `SAVEFILE_COMPAT.md` §7, §8 and §13.1;
  - `NEW_WORLD_PREVIEW_PROPOSAL.md` §4.2.
- **Code opened**, all in `cartalith-native/crates`:
  - `cartalith-engine`: `lib.rs` (`tectonic_substrate`, `generate_terrain_inner`, `plate_base_blur_r`, `roughness_for`, `fill_tilt_for`), `preview.rs`, `region_export.rs`, `import.rs`;
  - `cartalith-terrain`: `lib.rs` (`build_plates`, `gauss_blur`, `compute_flexure`, `normalize_field`, `terrain_detail_k`, `clamp_feature_radius_cells`, the volcano stamps), `amplify.rs`, `infer.rs`;
  - `cartalith-hydrology`: `lib.rs` (`build_routing_surface_with`) and `rough.rs`;
  - `cartalith-climate`: `lib.rs` (`simulate_weather_with`);
  - `cartalith-godot`: `ops_bridge.rs` (`region_new_world`), `params.rs` (`LOADS_WHEN_ABSENT`, `world_key_state`) and `tests/nwprev_cost.rs`;
  - `cartalith-io`: `project.rs`.
- **v2.71 reference:** `buildTectonicSubstrate`, `inferTectonics`, `amplifyRegion` and the `regionNewWorldBtn`
  handler, found by name.
- **Not done:**
  - no GPU run;
  - no check of whether the shell guards Generate on a region world;
  - no 16k run;
  - no timing.

No code was changed and no test was added.

---

## 8. Free B ("Re-simulate region") as a proof of concept: how it sits with this research

Added after Ruling GV. The owner: *"Do note for now it's proof of concept and check how it sits with your research"*.
The mode is defined in `WORLD_SCALE_TERRAIN_PLAN.md` §9a: off by default, no save-format promises, a measured PoC
bar.

### 8.1 Against each source

- **This review's measurement (§4.1)** `[measured]` + `[inference]`. A free re-run keeps the coarse geography once
  `blur_r` is in ground units (median land 98.6 %, height r 0.994) and moves about half the major river cells
  (49.7 % within one coarse cell, worst 7.8 %). Uncorrected it is clearly worse (25.5 %, r 0.959). So **the PoC must
  run with the cell-unit terms corrected**, at least `blur_r`, and say which of §2.2's other terms it left in cells.
  The measurement re-ran the whole substrate; §9a keeps the parent's plates, normalisers and climate, so its coarse
  geography should hold at least as well. That is an expectation, not a result.
- **ZR-0's no-go (`ZOOM_REFINEMENT_SCOPE.md` §8)** `[read]` + `[inference]`. ZR-0 failed because per-tile steps
  cannot grow structure at 5-50 km from a smooth cell (`bind_r` 0.10-0.25 against native 0.47-0.59). Free B is not a
  per-tile step: it erodes and drains the whole block at once, which is the form the plan's §9.3 argues can grow
  real drainage. **ZR-0 is therefore not evidence against B, and B is the first direct test of that argument.** Two
  ZR-0 findings carry over as risks:
  - physical-unit steps fought the per-map normalisation (one talus pass moved 1.6 M texels), so B must use the
    parent's normalisers (§9a item 3) and check slopes at the finer cell;
  - ZR-0's `bind_r` and land-gradient metrics are the right landform bar, and §9a item 5 uses them.
- **`WORLD_EMBOSS_FINDINGS_2026-11-15.md` §2.5** `[read]` + `[inference]`. The findings say real dendritic valleys
  at the viewed scale are out of reach for any noise law, because a native map's landforms come from the generation
  pipeline at its own cell size. Free B *is* the pipeline at the finer cell size, so it tests that statement directly.
  One trap the PoC must measure: a native 400 km map gets its own plates (six in the harnesses) across 400 km, while a
  400 km cut-out of a 36-plate planet may lie inside one plate with no boundary. A faithful B can then look *calmer*
  than a native map of the same extent. That would not be a defect of B; it is the native maps compressing tectonics
  into a small extent. B also does not close FG's B4 for the zoomed world: it makes a separate world.
- **`TERRAIN_COMPUTE_METHODS_RESEARCH.md`** `[read]`. It makes the cost plausible: Braun-Willett implicit stream power
  is O(n) and was measured at about 4× the iterative kernel (ZR-0: 20 ms per tile); Priority-Flood is exact and has a
  tiled parallel form for larger blocks. Its GPU rule binds the PoC: generation kernels stay CPU or integer, since a
  float GPU kernel makes a different world per device.

### 8.2 What a PoC can and cannot show

**Can show:**

- whether a re-simulated block grows landforms near a native map's (`bind_r`, land gradient, frames side by side);
- how far rivers, lakes and coasts move against the parent;
- whether the coarse-geography bars hold with the parent's plates, normalisers and climate;
- the cost and peak memory of a 16 M-cell block on the PC, and determinism across thread counts.

**Cannot show:**

- whether users accept moved rivers and re-sited towns: that is the owner's judgement on frames;
- the product UI, the save behaviour or snap-back, none of which the PoC promises;
- the GPU path, the phone and tablet, or worlds beyond the seeds it runs;
- a full ground-unit pipeline, unless the PoC converts every §2.2 term.

### 8.3 The cheapest PoC

Two steps, both an `#[ignore]`d harness with no shell work:

1. **Whole-parent re-run, then crop.** Parent: a 1 200 km region at 1024 wide, seed 1200. Re-simulate the whole
   parent at 4096 wide with `blur_r` × 4, crop a 400 km window, and compare it with the parent's window (§9a bars 1-4)
   and with a native 400 km map at fit (bar 5). This needs no new windowed code; it is §4.1's harness at a larger
   ratio plus ZR-0's metrics and frames. It answers "does the pipeline at a finer cell grow landforms" for minutes of
   compute.
2. **Only if step 1 earns it: the block form on a planet.** One 400 km cut-out of the 40 075 km / 2048 world at
   about 98 m (about 4 000 × 4 000 cells), with the parent's plates, substrate rasters, normalisers and climate. The
   new code is a windowed height composition evaluated in the parent's frame. Same bars, plus cost and memory.

Each step: one CPU run per seed, three seeds (1200, 42, 51127), side-by-side frames at the parent's fit and at two
deeper zooms, opened and looked at.

### 8.4 What would justify promoting it

All of these, then a separate owner ruling:

1. Bars 1-4 of §9a met on all three seeds.
2. Bar 5 met where ZR-0 failed: `bind_r` within 0.15 of the native map in at least 5 of ZR-0's 6 place-and-zoom
   cells (ZR-0 met 1), and land gradient at least 0.6 of native.
3. A 16 M-cell block in a couple of minutes on the PC, within the 4k world's peak memory, and identical at 1, 4 and
   16 threads.
4. The owner's (or Fable's) look at the frames prefers B's landforms to A's and accepts the moved rivers.
5. Measured against the constrained mode (plan §9, §11 step 3) where that exists. If the constrained mode meets bar
   5 too, B adds only the "new world inspired by the region" use, and promoting it is a choice, not a need.

If step 1 fails bar 5 the way ZR-0 did, that is a finding in itself: the native maps' richness comes from their plate
density per extent, not from the finer cell, and neither B nor the constrained mode will reach it without a
landform process at the 5-50 km scale.

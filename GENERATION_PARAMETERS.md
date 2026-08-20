# Generation parameters: the complete exposed surface

Owner directive: *"make all generation options active in the current interface
so that we have the same functional controls as the older html version."* This
document is the Rust half's contract — every generation parameter
`cartalith-godot`'s `WorldGen` now exposes to GDScript, what it is, what it
defaults to, what range it accepts, what it physically does, and whether the
HTML reference ever gave a user a control for it.

It is written to be worked *from*, not skimmed: the GUI fork builds its
Generate-menu dialogs against this surface, and every later pass (save/load of
parameter sets, presets, stale-field tracking) checks itself against this
inventory.

**Sources, all read directly rather than summarised from another doc**:
`cartalith-native/crates/cartalith-engine/src/lib.rs` (the eight parameter
structs and `WorldParams::defaults`), `reference/Cartalith Gen1 v2.10.html`
(the real `<input>` elements at lines 962-1215 and their `tparam`/`eparam`/
`cparam`/`bind` handlers at lines 12718-13012, plus `state`'s own literal
defaults at lines 2258-2310 and `syncUI` at 12648), `docs/GENERATOR_PARAMETERS.md`
(the physical-meaning reference, written against an older snapshot but with
mappings that still match v2.10 exactly for every parameter below),
`GUI_FEATURE_PARITY_SCOPE.md` Category 1, and `FUNCTIONAL_CONTRACT.md`.

## What changed

Before this pass, `WorldGen` exposed **7** of the engine's generation
parameters: `sea_level` (`set_sea_level`), four subsystem flags
(`set_experimental_flags`), and — indirectly, as five hardcoded named presets
with no path for raw values — the World-Structure block
(`generate_world_structure`). Everything else in `TectonicParams`,
`VolcanismParams`, `CraterParams`, `PlanetParams`, `ClimateInputParams`,
`StreamParams`, `WorldStructureParams` and `WorldParams`' own top level was
live in the engine and unreachable from the UI.

After it, **58** parameters are reachable, covering all eight structs. The
seven that were already reachable keep their old `#[func]`s (`main.gd` drives
them) — those are now thin sugar over the same storage, so the two surfaces
cannot disagree.

## The API

Six new `#[func]`s on `WorldGen`, plus three read-only additions. The
parameter namespace is **flat and dotted** — `"sea_level"`, `"tect.plates"`,
`"climate.lat_n"` — mirroring the `WorldParams` field path exactly, so a
reader of either side finds the other without a lookup table.

| Signature | Returns |
|---|---|
| `get_params() -> Dictionary` | Every parameter's **current** value, keyed by dotted key. `bool` for checkbox parameters, `int` for whole-number ones, `float` otherwise. |
| `get_param_defaults() -> Dictionary` | The same shape at `WorldParams::defaults` — what a "reset to default" control shows. Never affected by this instance's state. |
| `get_param_info() -> Dictionary` | key -> `{group, type, default, min, max, step, label, unit, reference_control}`. Everything a dialog needs to build a control, so no range/step/label is hardcoded twice. |
| `get_param_groups() -> PackedStringArray` | `["world", "planet", "world_structure", "tectonics", "volcanism", "erosion", "climate", "weather"]` — the section order, each matching a real panel heading in the reference's sidebar. |
| `set_params(values: Dictionary) -> Dictionary` | Applies a **partial** dictionary. Returns `{"rejected": PackedStringArray, "clamped": PackedStringArray}`. Both empty = every key applied exactly as sent. |
| `reset_params() -> void` | Restores every parameter to its engine default. |
| `get_gpu_stages_used() -> PackedStringArray` | Read-only: which GPU-eligible stages actually ran on GPU last generation. |
| `get_seed() -> int` | The seed the last generation used (`0` before the first). Seed is a `generate()` argument, not a parameter. |
| `get_villages_enabled() -> bool` | Round-trip for `set_villages_enabled` (a `cartalith-civ` toggle, not a `WorldParams` field, so not in `get_params()`). |
| `apply_archetype(name: String) -> bool` | Writes a named World-Structure preset into the **persistent** parameters and enables World Structure, so the five knobs then show real numbers and stay editable. `false` for an unknown name, changing nothing. |
| `get_archetypes() -> PackedStringArray` | `["earth", "supercontinent", "archipelago", "volcanic", "rift"]`. |

Unchanged, and still the way `main.gd` drives them:
`set_sea_level(f)`, `set_experimental_flags(bool × 4)`,
`set_villages_enabled(bool)`, `generate(seed, width_km, resolution)`,
`generate_world_structure(seed, width_km, resolution, archetype) -> bool`.

## Map dimensions and aspect ratio

**The map does not have to be square, and in the reference it never is.**

`cartalith_engine::WorldParams` has always carried independent `pub gw` and
`pub gh`, every subsystem threads both, and **every golden-parity fixture in
this workspace is non-square** (14×11, 16×12, 24×18, 20×14, 48×40, 10×8) — so
JS parity at non-square dimensions has been established since those
subsystems were ported. The squareness lived entirely in `cartalith-godot`:
`generate()` took one `resolution` and `call_params` wrote `p.gh = gw`,
discarding the capability at the boundary. The reference itself computes
`GH = gridH(GW) = round(GW × (world ? 0.5 : 0.64))` (reference HTML line
5049) — 2:1 equirectangular in world mode, a 1.5625:1 frame in region mode —
and its "Working resolution" segment (`512 / 1K / 2K / 4K / 8K`) sets the
**width** only.

| Signature | Returns |
|---|---|
| `generate_sized(seed: int, width_km: float, grid_w: int, grid_h: int) -> void` | The general entry point: full pipeline at independent grid dimensions. Each dimension clamped to ≥ 4. |
| `generate_world_structure_sized(seed, width_km, grid_w, grid_h, archetype) -> bool` | Same, with a one-call archetype applied. `false` for an unknown name. |
| `reference_grid_height(grid_w: int, world: bool) -> int` | The reference app's own `gridH`: `round(grid_w × 0.5)` when `world`, `round(grid_w × 0.64)` otherwise, floored at 4. Pure function; touches no state. |
| `get_map_width_km() -> float` | The current world's real map **width** in km. `0.0` before any generation/load. |
| `get_map_height_km() -> float` | The current world's real map **height** in km — **derived** as `map_width_km × gh / gw`. `0.0` before any generation/load. |

`get_width()` / `get_height()` already returned the real `gw` / `gh` (the
loaded-save path has always carried both), so nothing new is needed to read
the shape back.

**One caveat the GUI must respect: in `world` mode, 2:1 is not a preference,
it is the physically consistent shape.** X wraps a full 360° of longitude
over `gw` and Y spans 180° of latitude over `gh` (`lat_at`, reference
`latAt`), so any other ratio silently stretches the graticule. Generation
does not enforce it (nothing panics, and a loaded save may legitimately be
any shape), so a setup dialog should default world extent to
`reference_grid_height(grid_w, true)` rather than offering the two dimensions
as free, independent choices.

### Why height is a `generate()` argument, not a parameter

Same reasoning `resolution` already had, and stated in the same place: grid
dimensions reallocate every field in the pipeline, so they are a creation-time
decision, not a slider. Making `grid_h` a stored `set_params` key would put
it in a table whose whole contract is "set it, then generate as many times as
you like" — which is exactly what a dimension cannot support. It therefore
sits beside `seed`, `resolution` and `width_km` as a call argument.

### Why there is no `map_height_km`

Because it is not free to choose. Every kilometre↔cell conversion in this
workspace derives from **one** quotient, `map_width_km / gw`, and applies it
to both axes:

- `cartalith_terrain::terrain_detail_k(gw, map_width_km)` → `cell_km = mwk / gw`,
- `cartalith_hydrology::river_flow_thresh(gw, gh, world_gw, map_width_km)`,
- `cartalith_civ::civ_catchment_radius_cells(cat_km2, map_width_km, gw)`,
- `cartalith_civ::suppression_radius_cells(spacing_km, gw, map_width_km)`.

So the engine's real, already-shipped assumption is that **cells are square
in kilometres**. A separately-settable map height would contradict every
distance, grade, river threshold, catchment radius and settlement spacing the
world is generated from — the same class of silent rescaling the reference
cites as its reason for freezing `map_width_km` after creation. A world 2:1
in cells is 2:1 in kilometres; `get_map_height_km()` reports that, and there
is deliberately no setter.

### Backward compatibility

`generate(seed, width_km, resolution)` and `generate_world_structure(...)`
are unchanged and still square — they now delegate to the `_sized` forms with
`grid_h = grid_w`. Square output is bit-identical to before this pass; every
golden-parity fixture is unmodified and passing.

### What the plate frame does on a non-square sheet

The atlas plate border (`TERRAIN_APPEARANCE_SCOPE.md` milestone 4) is a
**uniform margin in cells on all four sides**, keyed to `gw`, which is what a
real plate margin is and what keeps `get_border_inset_frac()`'s
"fraction of texture width" contract exact under `map_overlay.gd`'s uniform
fit. One guard was added: on a plate much *wider* than it is tall, a
width-derived margin can exceed half the height and swallow the sheet, so
`border_width_cells` caps at `0.25 × gh` — **only when `gh < gw`**, so every
square and every tall grid keeps exactly the width it had before.

### Persistence

**Parameters persist on the `WorldGen` instance between generations.** Set
once, and every subsequent `generate()` / `generate_world_structure()` uses
them until `reset_params()` or another `set_params()` changes them — the same
behaviour `set_sea_level`/`set_villages_enabled` have always had. This is
stated in the doc comments on `WorldGen::params`, `set_params` and `generate`,
not left to be discovered.

### What is *not* a parameter, and why

Three values stay `generate()` arguments (four, counting `grid_h` — see
**Map dimensions and aspect ratio** above):

- **`seed`** — changes on every "New seed" click; it is the call, not a
  setting. Readable back via `get_seed()`.
- **`resolution`** (`gw`, and `gh` via `generate_sized`) — a
  working-resolution segment in the reference, not a slider, and it
  reallocates every field.
- **`width_km`** (`map_width_km`) — the reference itself refuses to make this
  editable mid-project: *"it's a creation-time decision, set only in the
  New-world / Import setup gate (never editable mid-project; changing it would
  silently rescale every derived distance, grade, route length and settlement
  spacing)"* (reference HTML, v0.83 comment at line 1015). This port follows
  that decision rather than overruling it.

Also not in `get_params()`: `set_villages_enabled` (a `cartalith-civ` concern,
with its own getter) and the render-side `TerrainAppearance` (Phase 3, its own
future surface).

### Invalid-value handling — the decision, and why

One policy, identical for every parameter, implemented once in
`params::set`:

| Input | Result |
|---|---|
| **Unknown key** | **Rejected.** Nothing written, key returned in `"rejected"`, and a line printed to the Godot console. A typo'd key in a dialog must not silently do nothing *and* look like it worked. |
| **Wrong type** | **Rejected.** A `bool` parameter takes only a real boolean (no truthy numbers — `0`/`1` do not mean `false`/`true` here); a numeric parameter takes only `int`/`float`. Strings, Arrays, `null` and Objects are never coerced: `"0"` and `null` both have plausible-looking numeric coercions that would silently write a wrong world. |
| **NaN / ±infinity** | **Rejected.** Clamping a NaN produces a NaN either way (`NaN.max(x)` is `x`, `f64::clamp` panics on NaN bounds), and one NaN in the height field propagates through every downstream stage — and NaN comparison differs between JS and Rust (`cartalith-rust-conventions`), so it would not even fail loudly. |
| **Out of range** | **Clamped, applied, and reported** in `"clamped"`. |
| **Fractional value for an `int` parameter** | **Rounded, applied, and reported** in `"clamped"` — a GDScript slider with a float step happily produces `13.999999` for "14". |

Clamping rather than rejecting out-of-range values is a real choice with real
consequences, so here is the reasoning: every one of these values feeds a
generation kernel with no meaningful behaviour outside its range (a negative
plate count, a sea level of 4.0, zero grid iterations). Clamping keeps
generation always well-defined; it matches the precedent already set in this
same file by `set_sea_level`'s own `.clamp(0.0, 1.0)` and `generate`'s
`resolution.max(4)`; and — the part that makes it safe — it is **reported**,
so a dialog reads `get_params()` back and shows the stored value rather than
assuming its widget won. Rejecting would be purer but would make an
overshooting slider do nothing at all, which reads as a broken control.

**Consequence for the GUI**: after `set_params`, if `"clamped"` is non-empty,
re-read `get_params()` for those keys and update the widget. If `"rejected"`
is non-empty, that is a bug in the caller, not user error.

## Zero behaviour change at defaults

`WorldGen::params` is initialised to `WorldParams::defaults(0, 0, 0)`, and
`generate()` overwrites only `gw`/`gh`/`tect.seed`/`map_width_km` before
calling `generate_terrain`. An instance nobody calls a setter on therefore
builds a byte-identical `WorldParams` to the one the old code built inline.
Verified: `cargo test --workspace` passes with **0 regressions** (83 test
binaries, every golden-parity fixture unmodified), and
`tests/params_mapping.rs::defaults_round_trip_through_every_key` asserts that
writing every parameter's own default back leaves `WorldParams` `PartialEq`-
identical to `WorldParams::defaults`.

## Reading the tables

- **Key** — the dotted key `set_params`/`get_params`/`get_param_info` use.
- **Field** — the `cartalith_engine::WorldParams` field path.
- **Default** — `WorldParams::defaults`' value, which is the reference's own
  `state` literal in every case.
- **Range** — the `min`..`max` (and `step`) `get_param_info` reports and
  `set_params` clamps to.
- **Reference control** — the reference HTML element id, its raw slider range,
  and the mapping its handler applies. **`—` means the reference never exposed
  this as a user control**: it is an internal tuning constant this port
  chooses to surface anyway. Per `DECISIONS.md` §7d that is a superset, not a
  violation, because the default reproduces reference behaviour exactly — but
  the distinction is recorded, not blurred.

Where a reference slider's raw range maps to a narrower float range than the
underlying field could hold (`tect.flexure` reaches only `0.36`, `tect.hetero`
only `0.16`), the table carries the **reference-reachable** range, deliberately:
matching the control the reference actually shipped is the stated goal. The
reference's own static `value=` attributes for those two sliders are stale
markup that `syncUI` corrects on load — the state defaults (`0.20`, `0.08`)
are the truth and both sit inside the reachable range.

---

## Group `world` — Source & resolution + Scale & calibration

| Key | Field | Type | Default | Range | Reference control | Meaning |
|---|---|---|---|---|---|---|
| `world` | `world` | bool | `false` | — | `extentSeg` (Region / Whole world) | Region = a framed area with user-set latitudes; Whole world = seamless equirectangular with toroidal X wrap. Changes plate/noise wrapping in every stage. |
| `sea_level` | `sea_level` | float | `0.42` | 0.0 .. 1.0, step 0.01 | `sea`, raw 0-100, `v/100` | The normalized height counted as 0 m; below is ocean. A threshold on the already-`[0,1]`-stretched field, not a metre value. **Overridden when World Structure is on** — `apply_world_structure_sea_level` re-anchors it from the archetype's land-fraction target. |
| `peak_m` | `peak_m` | float | `4000` | 1 .. 30000, step 50 | `peak` (number input, min 1 step 50) | Metres at the highest point. Sets the vertical scale (`metresPerUnit = peakM/(1-seaLevel)`), which drives temperature lapse and every grade readout. |
| `carve_rivers` | `carve_rivers` | bool | `true` | — | `carveRiversChk` | Runs the light stream-power pass plus parabolic valley stamping along the Strahler network inside `generate()`, so rivers sit in carved terrain instead of painted on a flat surface. Off → no channel topology at all. |
| `river_density` | `river_density` | float | `1.00` | 0.30 .. 3.00, step 0.05 | `riverDensR`, raw 30-300 step 5, `v/100` | Scales the channel-initiation drainage-area threshold. Higher = fewer, larger channels; lower = a denser network. (`state.viz.riverDensity` in the reference — a viz field that genuinely feeds generation.) |
| `use_gpu` | `use_gpu` | bool | `false` | — | `gpuToggle` | Runs plate assignment, domain warp, heterogeneity, the flexure/base blur, weather and flow accumulation on GPU where available, falling back to CPU **per stage** on any failure. **Not a performance-only switch**: per `DECISIONS.md` §7c the GPU noise primitive is a different hash function, so the same seed produces a different (still valid, still deterministic) world. Read `get_gpu_stages_used()` for what actually ran. |

## Group `planet`

Gravity is the one planetary parameter with terrain-wide reach: it scales
fluvial and glacial erosion (×g), the temperature lapse (×g), crater size
(×g⁻⁰·²²), wave energy (×1/g), and rescales peak altitude (~1/g).

| Key | Field | Type | Default | Range | Reference control | Meaning |
|---|---|---|---|---|---|---|
| `planet.g` | `planet.g` | float | `1.00` | 0.30 .. 2.50, step 0.05 | `pg`, raw 30-250 step 5, `v/100` | Surface gravity in Earth g. Reaches `stamp_craters`, `stream_power_kernel` and `compute_temperature`'s lapse term. Low-g worlds get taller mountains (the Olympus Mons effect). |
| `planet.rotation_hours` | `planet.rotation_hours` | float | `24.0` | 6 .. 96, step 1 | `prot`, raw 6-96 step 1 | Rotation period in hours. Sets the atmospheric circulation-cell count (`N_c ≈ 3·√((24/h)·radius/√g)`) and the Coriolis term in `build_wind`; also scales the equator-pole temperature contrast (`(24/h)^0.25`, v1.85). Fast spin → many wind belts; slow spin → one giant Hadley cell. |
| `planet.axial_tilt_deg` | `planet.axial_tilt_deg` | float | `23.4` | 0 .. 45, step 0.5 | `ptilt`, raw 0-45 step 0.5 | Obliquity. v1.85: scales the equator-pole temperature **contrast** (not the pole temperature) via `s2(tilt)/s2(23.4°)`, `s2(ε)=3sin²ε−2` — the 2nd-order energy-balance obliquity term (North & Coakley 1979). Lower tilt sharpens the gradient (≈1.31 at 0°), higher flattens it (≈0.33 at 45°). Exactly 1.0 at the default. |

## Group `world_structure`

The five raw archetype knobs. `GUI_FEATURE_PARITY_SCOPE.md` Category-1 item 8
named these as real-but-unreachable: the engine struct always took arbitrary
values, but `WorldGen` only ever reached them through five hardcoded presets.

When `world_structure.enabled` is on, `deriveFromWorldStructure()`'s own
overrides apply: plate count becomes `clamp(round(4 + fragmentation·24), 4, 40)`,
drift becomes `tectonic_energy·2`, volcano count becomes
`round(hotspot_density·60)` — so `tect.plates`, `tect.vel` and `volc.count`
below are **ignored** in that mode, exactly as in the reference. Graph-driven
orogeny also switches on with it (the reference's `tectonicGraph`, whose only
caller is this same derivation).

| Key | Field | Type | Default | Range | Reference control | Meaning |
|---|---|---|---|---|---|---|
| `world_structure.enabled` | `world_structure.enabled` | bool | `false` | — | `wsEnabled` | Generates a continentality field first: continental plates settle in high-field zones, oceanic in low, and a large-scale elevation bias anchors the land/sea split. Off = the plain random plate generation. |
| `world_structure.continentality` | `.continentality` | float | `0.30` | 0.01 .. 0.90, step 0.01 | `wsCont`, raw 1-90, `v/100` | How much land vs ocean. Also the land-fraction target `apply_world_structure_sea_level` re-anchors sea level against. |
| `world_structure.fragmentation` | `.fragmentation` | float | `0.50` | 0.0 .. 1.0, step 0.01 | `wsFrag`, raw 0-100, `v/100` | One landmass or many. Drives the derived plate count and pushes plate bases toward an archipelago distribution. |
| `world_structure.tectonic_energy` | `.tectonic_energy` | float | `0.60` | 0.0 .. 1.0, step 0.01 | `wsTect`, raw 0-100, `v/100` | Overall relief intensity; becomes the derived drift velocity (`×2`). |
| `world_structure.ocean_depth` | `.ocean_depth` | float | `0.60` | 0.0 .. 1.0, step 0.01 | `wsOcean`, raw 0-100, `v/100` | How deep ocean basins sit below sea level. |
| `world_structure.hotspot_density` | `.hotspot_density` | float | `0.20` | 0.0 .. 1.0, step 0.01 | `wsHot`, raw 0-100, `v/100` | Volcanic-province likelihood; becomes the derived volcano count (`×60`). |

**Archetype presets** (reference `ARCHETYPES`, lines 2521-2526), applied
persistently by `apply_archetype()` or for one call by
`generate_world_structure()`:

| Name | continentality | fragmentation | tectonic_energy | ocean_depth | hotspot_density |
|---|---|---|---|---|---|
| `earth` | 0.30 | 0.50 | 0.60 | 0.60 | 0.20 |
| `supercontinent` | 0.60 | 0.10 | 0.50 | 0.70 | 0.10 |
| `archipelago` | 0.15 | 0.90 | 0.80 | 0.30 | 0.50 |
| `volcanic` | 0.05 | 1.00 | 0.90 | 0.80 | 1.00 |
| `rift` | 0.40 | 0.35 | 0.75 | 0.55 | 0.30 |

## Group `tectonics`

The height formula these feed:
`field = 0.5 + α·(0.40·base + 0.50·stress) + F·flexure + C·heterogeneity + β·noise·(0.25 + 0.75·rugosity)`.

| Key | Field | Type | Default | Range | Reference control | Meaning |
|---|---|---|---|---|---|---|
| `tect.plates` | `tect.plates` | int | `14` | 4 .. 40, step 1 | `plates`, raw 4-40 step 1 | Number of tectonic plates (Voronoi cells over drifting centroids). Few = large continents and long boundaries; many = fragmented, busier coastlines. Ignored when World Structure is on. |
| `tect.vel` | `tect.vel` | float | `1.00` | 0.0 .. 2.0, step 0.02 | `vel`, raw 0-100, `v/50` | Plate-velocity multiplier: how far centroids move, and so the stress magnitude at boundaries. Ignored when World Structure is on. |
| `tect.warp` | `tect.warp` | float | `0.45` | 0.0 .. 1.0, step 0.01 | `warp`, raw 0-100, `v/100` | Domain-warp amount. Distorts the sampling grid for organic, non-circular coastlines and ridgelines. 0 = geometric and blobby. |
| `tect.blur_r` | `tect.blur_r` | float | `18.0` | 2 .. 42, step 0.4 | `sigma`, raw 0-100, `2 + (v/100)·40` px | Blur radius for plate base and stress. Small = sharp, narrow mountain belts; large = broad smooth swells. Also sets the flexural and isostatic-rebound wavelength. |
| `tect.alpha` | `tect.alpha` | float | `0.85` | 0.0 .. 1.2, step 0.012 | `alpha`, raw 0-100, `v/100·1.2` | Weight of the tectonic signal (plate base + stress) in the height formula — the master "how tectonic vs how noisy" dial. |
| `tect.beta` | `tect.beta` | float | `0.22` | 0.0 .. 0.6, step 0.006 | `beta`, raw 0-100, `v/100·0.6` | Weight of the fBm/ridged fractal detail layered on top of tectonics, concentrated near boundaries by the rugosity term. |
| `tect.age_inf` | `tect.age_inf` | float | `0.60` | 0.0 .. 1.0, step 0.01 | `age`, raw 0-100, `v/100` ("Erosion / age") | Boundary-age influence. `rugosity = exp(−age·(1 + ageInf·6))`: young crust near boundaries is rough, old interiors smooth. High = sharp arcs against flat cratons. |
| `tect.ridged` | `tect.ridged` | bool | `true` | — | `ridged` | Switches the fractal between ridged (sharp crests, mountainous) and standard fBm (rolling). |
| `tect.flexure` | `tect.flexure` | float | `0.20` | 0.0 .. 0.36, step 0.006 | `flexure`, raw 0-60, `v/100·0.6` | Lithospheric-flexure weight: broad isostatic arches around mountain loads and subsidence in rift basins — the main driver of continental shelves. |
| `tect.hetero` | `tect.hetero` | float | `0.08` | 0.0 .. 0.16, step 0.004 | `hetero`, raw 0-40, `v/100·0.4` | Within-plate crustal-diversity weight: low-frequency fBm × age, giving craton interiors and sedimentary basins their own topography. |
| `tect.resist` | `tect.resist` | float | `0.50` | 0.0 .. 1.0, step 0.01 | `resist`, raw 0-100, `v/100` | Erodibility spread by rock type in the stream-power pass: old shields resist incision (5-30% rate), young volcanic arcs erode at full rate. |
| `tect.dynamic_lithology` | `tect.dynamic_lithology` | bool | `false` | — | `dynLithChk` | Exhumation hardening: after erosion, re-derives the resistance field so stripped-down crust exposes more resistant rock. Reference default off. |
| `tect.lloyd` | `tect.lloyd` | int | `2` | 0 .. 8, step 1 | **—** | Lloyd-relaxation passes over the plate centroids before Voronoi assignment. More = more evenly sized, less clustered plates. The reference stores it in `state.tect` but never gave it a control; range is this port's own judgement. |

## Group `volcanism` — Volcanism & impacts

Stamped after the base height is built and normalized, before erosion.

| Key | Field | Type | Default | Range | Reference control | Meaning |
|---|---|---|---|---|---|---|
| `volc.count` | `volc.count` | int | `20` | 0 .. 100, step 1 | `volc`, raw 0-100 step 1 | Number of volcanic edifices stamped into `field` and recorded in `volcanic_field` for biome tinting. Ignored when World Structure is on (derived from hotspot density). |
| `volc.age` | `volc.age` | float | `0.40` | 0.0 .. 1.0, step 0.01 | `volca`, raw 0-100, `v/100` | Weathering of volcanoes. Old (high) = lower, softer, wider; young (low) = tall and sharp with a caldera notch. |
| `volc.provinces` | `volc.provinces` | bool | `true` | — | `volcProv` | Province mode: clusters volcanoes on convergent boundaries and hotspots with power-law sizes, instead of the simple boundary scatter. |
| `crater.count` | `crater.count` | int | `100` | 0 .. 200, step 2 | `crat`, raw 0-100 step 1, `v·2` | Number of impact craters recorded in `impact_field`. Sizes follow a realistic D⁻² distribution: mostly small (0.5-5 km), rare large basins (25-200 km). Crater radius scales with `g^−0.22`. |
| `crater.age` | `crater.age` | float | `0.50` | 0.0 .. 1.0, step 0.01 | `crata`, raw 0-100, `v/100` | Crater degradation. Old = shallow and infilled; young = crisp rim with a central peak. |

## Group `erosion` — the stream-power pass `carveRiverValleys` runs

These are `state.stream` as read through the reference's own `streamParams()`.
They apply to the **light** pass `generate()` itself runs at
`max(4, round(iters·0.6))` iterations, not to the manual "Stream-power carve"
button (which this port does not have). `state.stream.cycles` is deliberately
absent: only `evolveCoupled()` — the manual "Evolve" tool — reads it.

| Key | Field | Type | Default | Range | Reference control | Meaning |
|---|---|---|---|---|---|---|
| `stream.uplift` | `stream.uplift` | float | `0.00` | 0.0 .. 0.4, step 0.004 | `sUp`, raw 0-100, `v/100·0.4` | Tectonic uplift rate competing against incision. **Default 0 by design** — the pass purely carves rivers. Raise it only to grow active-orogen ranges that fight the incision. |
| `stream.k` | `stream.k` | float | `0.012` | 0.0 .. 0.03, step 0.0003 | `sK`, raw 0-100, `v/100·0.03` | The erodibility constant K in `E = K·Q^m·S^n`, **×planet gravity**. High = deep, dense valley networks. |
| `stream.iters` | `stream.iters` | int | `15` | 4 .. 40, step 1 | `sIt`, raw 4-40 step 1 | Implicit-solver steps (Braun & Willett 2013) toward equilibrium. More = closer to a mature, graded river profile. |
| `stream.deposit` | `stream.deposit` | float | `0.30` | 0.0 .. 1.0, step 0.01 | `sDep`, raw 0-100, `v/100` | Sediment deposition in low-gradient reaches and below sea level — floodplains and fans. Never raises a channel above the surrounding land. |
| `stream.climate_k` | `stream.climate_k` | float | `0.50` | 0.0 .. 1.0, step 0.01 | `sClim`, raw 0-100, `v/100` | Couples local rainfall into K (`1 + climateK·2·rain`). High = wet regions erode much faster than dry, giving climate-driven landscape asymmetry. |

## Group `climate` — Climate & biomes

| Key | Field | Type | Default | Range | Reference control | Meaning |
|---|---|---|---|---|---|---|
| `climate.lat_n` | `climate.lat_n` | float | `55` | −90 .. 90, step 1 | `latN`, raw −90..90 step 1 | Latitude of the map's top row (Region mode). Sets the direction of the cold/warm gradient. Ignored when `world` is on (the map then spans pole to pole). |
| `climate.lat_s` | `climate.lat_s` | float | `5` | −90 .. 90, step 1 | `latS`, raw −90..90 step 1 | Latitude of the bottom row. The span between the two edges is the climate range across the map. |
| `climate.equator_temp` | `climate.equator_temp` | float | `30` | 0 .. 45, step 1 | `teq`, raw 0-45 step 1 | Sea-level temperature at the warmest latitude, in °C. The *effective* value also passes through the axial-tilt and day-length contrast scaling above. |
| `climate.pole_temp` | `climate.pole_temp` | float | `−25` | −50 .. 10, step 1 | `tpo`, raw −50..10 step 1 | Sea-level temperature at the coldest latitude, in °C — the fixed anchor the tilt/rotation contrast scaling is measured from. |
| `climate.lapse_rate` | `climate.lapse_rate` | float | `6.5` | 0 .. 12, step 0.1 | `lapse`, raw 0-120 step 1, `v/10` | Temperature drop per km of elevation, in °C/km, **×planet gravity**. Higher = colder peaks, a lower snowline, more alpine zonation. |
| `climate.albedo_k` | `climate.albedo_k` | float | `0.00` | 0.0 .. 1.0, step 0.01 | `albedo`, raw 0-100, `v/100` | Ice-albedo feedback strength: ice and snow reflect sunlight and cool further, so polar caps and high massifs grow colder and broaden. 0 = off (the reference default; it also forces the CPU temperature path there). |
| `climate.currents` | `climate.currents` | bool | `true` | — | `currents` | Whether the ocean-current SST anomaly (Ekman rotation, coastal blocking, shelf friction, a western-intensification/gyre heuristic) feeds back into temperature and rainfall. |
| `climate.current_k` | `climate.current_k` | float | `1.00` | 0.0 .. 3.0, step 0.05 | **—** | Multiplier on that current strength. Stored in the reference's `state.climate` and read by `computeOceanCurrent`, but never given a control; range is this port's own judgement. |
| `climate.terrain_wind_deflection` | `climate.terrain_wind_deflection` | bool | `true` | — | **—** | Bends the prevailing wind around real mountains and coastlines. The reference had a `terrainWind` toggle in v1.77 and **deleted it in v1.78** — it is now unconditional there (owner: *"wind and current should always be coupled to terrain"*). Kept as a toggle here only so a run can be compared against the reference with it off; the default matches the reference's always-on behaviour. |

## Group `weather` — Weather · rainfall sim

Iterative moisture advection on a coarse grid: evaporate over sea → advect
along wind → precipitate (orographic + convective + supersaturation) →
deplete.

| Key | Field | Type | Default | Range | Reference control | Meaning |
|---|---|---|---|---|---|---|
| `climate.w_iters` | `climate.w_iters` | int | `70` | 20 .. 200, step 5 | `wIters`, raw 20-200 step 5 | Advection steps. More = moisture penetrates deeper inland; fewer = coastal-only wetness. |
| `climate.rain_k` | `climate.rain_k` | float | `1.00` | 0.0 .. 2.0, step 0.01 | `rainK`, raw 0-200, `v/100` | Strength of rain-on-rising-terrain. High = drenched windward slopes and stark rain shadows. |
| `climate.evap` | `climate.evap` | float | `0.12` | 0.0 .. 0.3, step 0.003 | `evap`, raw 0-100, `v/100·0.3` | Base moisture pickup over ocean. Under bulk-aerodynamic mode, wind speed and saturation deficit modulate it further. |
| `climate.rain_dep` | `climate.rain_dep` | float | `0.35` | 0.0 .. 1.0, step 0.01 | `rainDep`, raw 0-100, `v/100` | Depletion rate as air rains out. High = air dries quickly after the first ridge, giving sharp wet/dry boundaries. |
| `climate.ocean` | `climate.ocean` | float | `1.00` | 0.0 .. 2.0, step 0.01 | `ocean`, raw 0-200, `v/100` | Multiplier on the evaporation flux from sea cells — the global moisture-budget knob. |
| `climate.wind_manual` | `climate.wind_manual` | bool | `false` | — | `windModeSeg` (Planetary / Manual) | Planetary = latitude circulation belts (trades / westerlies / polar easterlies, belt count from day length) bent by thermal pressure. Manual = one fixed direction, Region mode only. |
| `climate.wind_dir_deg` | `climate.wind_dir_deg` | float | `0` | 0 .. 360, step 5 | `windDir`, raw 0-360 step 5 | Prevailing wind direction in Manual mode. Ignored in Planetary mode and in `world` mode (the reference disables the control there). |
| `climate.press_k` | `climate.press_k` | float | `0.60` | 0.0 .. 1.5, step 0.05 | `pressK`, raw 0-150 step 5, `v/100` | How strongly thermal lows and highs bend the planetary wind (Coriolis-deflected). High = monsoon-like sea→land deflection where summer land runs hot; 0 = pure zonal belts. |
| `climate.zonal_k` | `climate.zonal_k` | float | `0.50` | 0.0 .. 1.5, step 0.05 | `zonalK`, raw 0-150 step 5, `v/100` | Strength of the ITCZ-wet / subtropical-dry latitude correction applied on top of the emergent wind structure. |
| `climate.ocean_hum` | `climate.ocean_hum` | float | `1.00` | 0.0 .. 2.0, step 0.01 | **—** | Sea-surface humidity floor: the moisture level cells over water are seeded and re-topped to. Stored in the reference's `state.climate` and read throughout `simulateWeather`, but never given a control; range is this port's own judgement. |
| `climate.bulk_evap` | `climate.bulk_evap` | bool | `true` | — | **—** | Bulk-aerodynamic evaporation (`E = Ce·U·(qs−q)`) instead of a flat rate — makes evaporation respond to wind speed and saturation deficit. On in the reference, with no control. |

---

## Heightmap import — the third entry point (2026-08-20)

`generate*` builds a world from a seed and `load_save` reads one back from a
`.zip`. **`import_heightmap` is the third way in**, and the reference's own:
`Import ▸ Load heightmap…` (`#loadBtn`) followed by `Infer tectonics from
heightmap` (`#inferTectBtn`), reference HTML lines 534-535.

| Signature | Returns |
|---|---|
| `import_heightmap(path: String, seed: int, width_km: float, grid_w: int) -> bool` | Decodes a PNG, takes it as the elevation field, infers a tectonic substrate under it, then runs climate and flow. `false` (world untouched) on any read or decode error. |
| `heightmap_grid_size(grid_w: int, image_w: int, image_h: int) -> Vector2i` | What `import_heightmap` *would* resample onto, without importing — for a dialog that wants to show the working grid first. Pure; touches no state. |

**There is no `grid_h` argument, deliberately.** An imported DEM has a shape
of its own, and the reference derives the grid height from the *image's*
aspect ratio rather than the caller's: `GH = max(80, round(GW / (imgW /
imgH)))` (reference HTML line 4917). Resampling into a caller-chosen frame
would stretch the terrain. `get_grid_size()` reports what was used.

### Parameters this pass exposes, and the ones it deliberately does not

The inversion itself has four tunables in the reference, all passed as an
`opts` object that **every call site leaves empty** — `inferTectonics()`
calls `pickPlateSeeds(relief, W, H, {})`, `reconstructBoundaryStress(…,
{wrap})` and `stampVolcanicArcs(boundaryType, W, H, {})`. They are reachable
in Rust (`cartalith_terrain::infer`, each an `Option`) and **not** in the
parameter table, because adding a control the reference never had would make
this a superset rather than a port. They are listed here so the decision is
recorded rather than implied:

| Reference `opts` key | Default | What it does |
|---|---|---|
| `count` (`pickPlateSeeds`) | `clamp(round(W·H/3000), 6, 40)` | Inferred plate count. The cap of 40 is the reference's own v0.70 fix — uncapped, a 2K import produced ~900 plates and the pass became unusable. |
| `blurR` (`buildReliefField`) | `max(1, W/128)` | Smoothing on the gradient-magnitude relief proxy that decides where boundaries fall. |
| `blurR` (`reconstructBoundaryStress`) | `max(2, W/40)` | Smoothing on the synthesised stress and shear fields. |
| `updipK` / `shearK` | `6` / `8` | Sensitivity of normal stress to elevation-above-trend, and of shear to the along-strike gradient. |
| `decay` (`stampVolcanicArcs`) | `max(3, W/80)` | Exponential falloff of the volcanic-arc proxy away from subduction/arc cells. |

**The existing parameters that *do* apply.** `sea_level` decides which plates
come out oceanic and which continental, so it changes the inferred substrate
outright and is worth setting before importing rather than after. `map_width_km`
and `peak_m` are the calibrate step's own two fields — the reference reopens
its setup gate in `calibrate` mode after a heightmap loads, and that mode is
*literally the same form* as the new-world one with resolution and extent
omitted (`_suCalSync` is defined as `_suGenSync`). `tect.seed` still matters
for one downstream stage, `computeHeterogeneity`, which the reference reuses
verbatim; the inversion itself uses no RNG at all.

**The height stages are skipped, and must be.** `compute_height`,
`normalize`, volcano and crater stamping, world-structure sea-level
re-anchoring and river carving are all *forward* stages that write `field` —
running them would overwrite the imported elevation. The reference says so
directly: `inferTectonics` "leaves `field` untouched — only the tectonic/
derived layers". `impact_field` is therefore zero and `channels`/`river_mask`
are `None` on an imported world.

**Format: PNG, 8-bit.** The reference's file input is `accept="image/*"`,
decoded through the browser, so it takes PNG and JPEG and **not** TIFF (no
browser decodes TIFF natively) — PNG-only here is parity, not a shortfall. A
16-bit PNG imports fine but at 8-bit precision, because the reference reads
through a `<canvas>` and cannot see more than 8 bits either. Luma is Rec. 601
(`0.299 R + 0.587 G + 0.114 B`), the reference's own weights.

**One disclosed parity carve-out.** The resample from source pixels to the
working grid is browser-implementation-defined in the reference (`<canvas>`
`drawImage`; the HTML spec does not pin the filter and the three major
engines disagree), so there is no JS output to be bit-identical to. This port
uses a documented box-average downsample, deterministic here, under
`PARITY_TESTING.md`'s own carve-out for that case. Everything *downstream* of
the field — the whole inversion — is golden-parity tested bit-exact
(`cartalith-terrain/tests/golden_parity_infer.rs`).

## Parameters the reference exposed that this port does not

Recorded so the gap is a decision, not an omission. Every one of these belongs
to a pipeline stage `cartalith-engine` has not ported, not to a parameter that
was skipped:

- **Droplet hydraulic erosion** (`drops`, `estr`, `edep`, `ethr`, `etal`) and
  **hillslope diffusion** (`edD`, `edPas`) — the reference's manual "Erode
  (droplet)" / "Hillslope diffuse" buttons. `state.erosion` has no
  `cartalith-engine` equivalent; `generate()` never runs them.
- **Velocity (momentum) erosion** (`vIt`, `vStr`, `vMnd`) — a manual op the
  reference itself says "never auto-runs".
- **Glacial** (`gSnow`, `gKg`, `gUF`, `gPas`) and **coastal** (`cWave`,
  `cEst`, `cMar`, `cPas`) passes — not ported.
- **Evolve cycles** (`evoCyc` / `state.stream.cycles`) — read only by
  `evolveCoupled()`, the manual evolve tool.
- **Structured-orogeny tuning** (`foldI`, `trenchD`, `faultB`) — the T5 knobs.
  `generate_terrain` hardcodes them to the exact values the reference's own
  null-coalescing defaults produce (`0.16`, `1.0`, `0`), documented in the
  engine's module doc comment. Exposing them means threading three new fields
  through `OrogenyParams`' call site — real work, not a wiring gap.
- **Geoid** (`geoidChk`, `geoidAmp`) and **tides** (`tidesChk`, `tideMass`,
  `tideDist`, `tideK2`) — both default-off sub-objects of `state.planet`, not
  ported (`PlanetParams`' own doc comment says so).
- **Seasons / Köppen** (`seasons`) — `computeSeasons()` is deliberately
  deferred.
- **`radiusRel`** — read only by `circulationCells`, which `simulate_weather`
  already takes a fixed default for.
- **Min stream order** (`minOrderR`) — a render filter, not a generation
  parameter.

## Verification

- `cargo test --workspace`: 83 test binaries, all pass, 0 regressions, every
  golden-parity fixture unmodified.
- `cargo clippy -p cartalith-engine -p cartalith-godot --all-targets`: clean.
- `cartalith-native/crates/cartalith-godot/tests/params_mapping.rs`: 11 tests
  over the mapping layer — default round-trip through every key, defaults
  inside their own ranges, unique and contiguous groups, unknown-key
  rejection, wrong-type rejection in both directions, non-finite rejection,
  out-of-range clamping, int rounding, partial updates touching only their
  named keys, all eight engine structs reachable, and the two
  `GUI_FEATURE_PARITY_SCOPE.md` Category-1 items (`use_gpu`, the raw
  World-Structure knobs) reachable.
- `Godot_v4.7.1 --headless --quit main.tscn`: loads clean, extension
  initialises, `get_param_info()` returns 58 entries.

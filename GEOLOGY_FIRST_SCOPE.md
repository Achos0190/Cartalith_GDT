# GEOLOGY_FIRST_SCOPE.md — lithology, layering and age drive every landform process

**What this is.** The scope for owner **Ruling BH** (2026-09-27,
`LARGE_ITEM_RULINGS.md`, *"geology first; every landform process reads it"*).
It defines milestones **GF-0 to GF-7** and gives the reasoning for each. The
backlog row is `OUTSTANDING_WORK.md`'s *"Geology first: lithology, layering and
age drive every landform process"*.

**What this is not.** It does not track progress. Status lives only in
`cartalith-native/docs/STATUS.md`. A ruling on any owner question in §9 is
recorded in `LARGE_ITEM_RULINGS.md`, not here.

**How it was checked.** Every symbol cited in §1 was opened for this document on
2026-09-27. Symbols are named instead of line numbers, because line numbers in
this tree drift within a day. **No timing or landform figure in this document
was measured.** Figures derived by arithmetic from a constant in the code are
labelled as arithmetic. Each literature value is cited, and each value that is
a judgement is labelled as one, with the measurement that will tune it.

**Order with Ruling BG.** Ruling BG (landslips and pinnacles) builds on this
model. The owner's order is: BH scope, BH build, then BG. §10 lists what BG
gets from here.

---

## 1. Today's pipeline, read at its symbols

### 1.1 Stage order of `cartalith_engine::generate_terrain`

`generate_terrain` delegates to `generate_terrain_inner(p, false)`. That
function runs these stages, in this order. The `progress::advance` banner for
each stage is in brackets.

1. **World-structure overrides.** Plates, velocity and volcano count come from
   the archetype when `world_structure.enabled`. It is off in both
   `WorldParams::defaults` and `cartalith_godot::params::defaults`.
2. **[WORLD_STRUCTURE]** `generate_continentality_field`, only when
   world-structure is enabled.
3. **[TECTONICS]**
   - `compute_warp`
   - `build_plates`: 45 % oceanic. `Plate::base` is `±(0.55 + 0.45·u)`, so a
     negative base means oceanic crust.
   - `assign_plates`, which gives `plate_id`.
   - `compute_stress`, which gives `boundary_mask`, `boundary_type`,
     `stress_field` and `shear_field`.
   - `compute_flexure`
   - `base_raw`, the per-cell plate base. It is stored as
     `WorldState::crust_field`.
   - `base_field` (`gauss_blur` of `base_raw`)
   - `build_age_field`
   - `compute_heterogeneity`, which reads age.
   - `compute_resistance`
   - Orogeny (`trace_boundaries` → `tag_boundary_types` →
     `build_orogeny_field` → `smooth_orogeny`), only when world-structure is
     enabled.
   - `compute_height`, which reads age. Then `normalize_field`.
4. **[VOLCANISM]**
   - `stamp_volcanoes_provinces_shaped`, or `…_simple_shaped`. Each writes
     `field` and `volcanic_field`.
   - `stamp_craters`, which writes `field` and `impact_field`.
   - A clamp to 0..1.
5. **Sea level.** `apply_world_structure_sea_level`, or `p.sea_level`.
6. **Priming hydrology and climate.**
   - `routing_view` → `compute_flow`, which gives `flow_area`.
   - `compute_temperature`
   - `simulate_weather`
   - `apply_climate_moisture_correctors`
   - `apply_ocean_currents`, when `climate.currents` is on.
   - The pre-carve discharge flow is skipped when `carve_rivers` is on.
7. **[EROSION]** This runs only when `carve_rivers` is on.
   - `stream_power_kernel`, run for `max(4, round(iters·0.6))` = 9 iterations
     at the default `iters: 15`.
   - `isostatic_rebound`.
   - `recompute_resistance_after_erosion`, only when `tect.dynamic_lithology`
     is on.
8. **[HYDROLOGY]**
   - `routing_view` → `compute_flow`, which gives `flow_for_network`.
   - `build_channels_routed`
   - `strahler_from_receivers`
   - `trace_river_polylines`
   - `stamp_river_intensity`
   - `carve_channel_network` (RV-1, Ruling BD). This gives `river_mask` and
     `river_floor`.
9. **[CLIMATE]** The refresh:
   - `compute_flow`, which gives `flow_discharge`.
   - `compute_temperature`
   - `simulate_weather`
   - The correctors, and currents when on.
10. **Erosion passes** (`p.passes.any()`), in this order:
    - `velocity_erode_kernel`
    - `glacial_kernel`, then `isostatic_rebound`, then the dynamic-lithology
      recompute if on.
    - `coastal_process`
    - `hillslope_diffuse`
    - `evolve_cycles`: stream power, rebound, the recompute and
      `refresh_climate`, repeated.
    - `sediment_fill`: stream power, then `route_sediment`.
    - `apply_tidal_sedimentation`
    - A clamp, then `refresh_climate`.
11. **[ECOLOGY_BIOMES], [RESOURCES_SOILS]** These have no engine work.
    Lithology, soil and biomes are built later, outside this function
    (`cartalith_civ::compute_affordance_fields`).

**What each boundary actually runs.**
- **`WorldParams::defaults`** runs one erosion kernel: the 9-iteration light
  stream-power pass. `ErosionPassParams::off()` leaves step 10 empty.
- **The shipped app (`cartalith_godot::params::defaults`)** also runs
  **`glacial_kernel` and its rebound**, because Ruling AU sets
  `p.passes.glacial = true`. So step 10 runs in every app world, and ends in
  `refresh_climate`.

One consequence, read at the call site: the app's glacial pass modifies `field`
**after** `channels`, `stream_order`, `river_mask` and `river_floor` were
computed from the pre-glacial surface. Whether this shows is **unmeasured**.
The new stage order in §3 removes it either way.

### 1.2 Every field that encodes geology

| Field (on `WorldState` unless stated) | Computed by | What it actually is | Read by |
|---|---|---|---|
| `crust_field` | `base_raw` in `generate_terrain_inner`: `plates[plate_id].base` per cell, unblurred | Plate crust sign and magnitude. `< 0` is oceanic. Continental values are 0.55–1.0 | `build_lithology` (oceanic → basalt); Sample (`plate_oceanic`) |
| `age_field` | `build_age_field(gw, gh, boundary_mask)` | **Not an age.** It is a two-pass chamfer distance from the nearest plate-boundary cell, divided by the map's own maximum, so it runs 0 at a boundary to 1 at the cell furthest from any. It is relative per map, changes with plate count, and does not wrap in x | `compute_heterogeneity` (`low_n·(0.3 + 0.7·age)`); `compute_height` (roughness damping `rug = exp(−age·(1 + 6·age_inf))`); `compute_resistance`; `build_lithology` (`age_old = 0.6`); `build_soil_fertility` (`0.4 + 0.6·age`); `build_resource_potentials` |
| `resistance_field` | `compute_resistance`: `min(1, 0.6·max(base, 0) + 0.4·age)` | Erodibility proxy. On oceanic plates it is `0.4·age`, so 0–0.4. On continental plates it is 0.33–1.0 (arithmetic from `base` ∈ 0.55–1.0) | `stream_power_kernel` (the only shaping reader); `build_lithology` (`res_hard = 0.55`); landmark *rock formation* (`ROCK_FORMATION_MIN_RESISTANCE = 0.55`); Sample; the EF-3 tile re-erosion test (`ef3_tile_erosion.rs`) |
| `resistance_field`, mutated | `recompute_resistance_after_erosion(resist, pre, post, k = 6.0)`: `r ← min(1, r + k·(pre − post))` wherever erosion lowered the cell. Gated on `tect.dynamic_lithology`, which is **false at both boundaries** | The reference's *"exhumation exposes harder basement"* heuristic | On the default path it runs **after** the only stream-power call. So it shapes nothing, and only changes the stored field that `build_lithology` later reads. It feeds back into terrain only through `evolve_cycles` or `sediment_fill` |
| lithology (`Vec<u8>`, **not stored**) | `cartalith_civ::build_lithology(field, age, volc, crust, resist, rain, sea)`. Called after generation by `compute_affordance_fields`, `sample_bridge`, `lod_worker` (`LodSnapshot::build`), `export_raster`, the civ bridges and landmarks | Seven classes (`LITH_NAMES`). It is **derived from the finished terrain and climate.** Limestone, sandstone and shale are defined as *post-erosion* lowland (`r < 0.30`), split by *present* rainfall. Every volcano is andesite, including hotspot and rift basalt | Soil (`LITH_WEATHER`); resource potentials (copper, tin, iron and others key on class indices); render (`litho_palette`, `litho_microtexture`); landmarks (the rock name in the causal text); Sample; export; the channel atlas. **No shaping process reads it** |
| `boundary_type` | `compute_stress` → `classify_boundary` → `btype::{NONE, COLLISION, SUBDUCTION_OC, ARC_OO, RIFT, TRANSFORM}`. Set on boundary cells only | Tectonic setting at the margin | Volcano placement (`exclude_transform`); orogeny tagging; resource potentials; Sample |
| `stress_field`, `shear_field` | `compute_stress` | Blurred, normalised convergence and shear | Height; volcano placement; stream power (as the **uplift** field `u`, which is zero at the default `stream.uplift = 0.0`); resources |
| `volcanic_field` | The volcano stampers: `max((1 − t)·(1 − volc.age))` over the edifices | Volcanic intensity. **It carries no setting.** `VolcanicSetting` (Arc / Rift / Hotspot) is rolled per province, used for placement and by `EdificeModel::Morphological`, and then discarded | `build_lithology` (`> 0.35` → andesite); resources; landmark volcanic features |
| `impact_field` | `stamp_craters` | Shock and melt marker | Described by `DECISIONS.md` §7l as input *"for the lithology stage"*, but **no lithology code reads it** (§7l Ruling 3's own correction) |
| orogeny (not stored) | `build_orogeny_field` + `smooth_orogeny` | Fold and thrust relief along tagged polylines | `compute_height` only. It exists only with world-structure enabled |

### 1.3 Every shaping process: does it read rock today?

| Process | Symbol | Reads rock today? | The constant geology would replace or modulate |
|---|---|---|---|
| Stream-power incision | `stream_power_kernel` → `stream_power_kernel_bounded` | **Yes, weakly.** `Cc_i = K·g·max(0.05, 1 − 0.7·tect.resist·R_i)·(1 + 2·climate_k·rain_i)·dt·A_i^0.5/L_i` | The factor `max(0.05, 1 − 0.7·resist·R)`. At the default `tect.resist = 0.50` it is `1 − 0.35·R`: **0.65 to 1.0** over `R` ∈ 0–1. On continental land (`R` ≥ 0.33) it is at most **1.36×** between the softest and hardest rock (arithmetic, not measurement). The literature spans orders of magnitude (§2.3). The 0.05 floor is unreachable below `resist ≈ 1.36`, and the slider stops at 1.0 |
| Isostatic rebound | `isostatic_rebound` | No | `0.8·blur(max(0, pre − field))`. It does not read rock, and does not need to. It must **move the rock column with it** (§4.2) |
| Thermal (talus) | `erode_thermal`; GPU `thermal_grid_gpu_with` | No. **It is not in generation at all.** It runs only in the manual Erode op (`erode_op`) | Scalar `talus = 0.012`, a raw normalised height difference. It is extent-blind: `EROSION_GEOLOGICAL_TIME_SCOPE.md` §2 derives 7° at 800 km and 87° at 5 km |
| Hillslope diffusion | `hillslope_diffuse` | No. Off at both boundaries | `diffuse_d` (extent-scaled by `hillslope_extent_scale`, §7m) |
| Glacial | `glacial_kernel` | No. **On in the app** (Ruling AU) | `e = kg·g·Q^mg·S·0.001`. `kg = 0.15` is a scalar |
| Coastal | `coastal_process` | No. Off at both boundaries | Wave cut `wave_str·exposure·0.002`; shore debris `·0.0005`; estuary cut `0.003·…` |
| River carve (RV-1) | `cartalith_hydrology::carve_channel_network` | No | Half-width `(0.8 + 0.5·(o − 1))·width_k`, capped at `4·width_k`; per-step `drop = 0.0006`; `CARVE_LAND_MARGIN`; `CARVE_KEEP_LAKE_DEPTH` |
| Volcanism | `stamp_volcanoes_provinces_shaped` / `stamp_one_volcano` | No. It is a geology **producer**, not a reader | It writes no rock type. The setting is discarded after placement |
| Craters | `stamp_craters` | No. A producer | `impact_field` is unread |
| Sediment routing | `route_sediment`; stream power's internal deposition (`deposit = 0.3`); `apply_tidal_sedimentation`; the coastal marsh | No | Deposition writes height only. **No record of what was deposited survives** |
| Velocity erosion | `velocity_erode_kernel` | No. Off at both boundaries | `erode_k`, `capacity` |
| LOD detail | `cartalith_terrain::amplify` (`amplify_region_padded`, `add_zoom_detail_padded`, `sample_elevation`) | No | `AmplifyOpts::detail_amp = 0.14`; the relief taper `min(1, hypot(gx, gy)·8)`; `ridged` |
| Tile re-erosion (EF-3) | `cartalith_erosion::tile::tile_erode` | Yes, through the same stream-power expression. **It has no production caller.** Its only caller is `cartalith-engine/tests/ef3_tile_erosion.rs`, which passes a nearest-cell upsample of `resistance_field` | Whatever stream power reads |
| Landform classifier | `cartalith_terrain::landform::build_landform_field` | No. Morphometric only | It is **a measurement tool here** (§5), not a shaping process |

**The ruling's summary, checked.** It is correct, with three precisions:
- `dynamic_lithology` updates `resistance_field`, not the lithology label.
- On the default path it runs after the only stream-power call, so it shapes
  nothing.
- Thermal erosion is not "ignoring rock inside generation". It is not in
  generation at all.

### 1.4 What `EROSION_GEOLOGICAL_TIME_SCOPE.md` provides

Ruling BH says that scope's *"geological time is the clock they run on"*.
**There is no clock.** The owner ruled on 2026-09-02 *"let's only fix the
hillslope extent blindness"*. That fix is the extent correction (§7m) and
nothing else. The clock's prerequisites are still unruled (its §8, questions 1
and 3–5):
- reversing the reference's "not an iterated LEM" position;
- a second self-referential anchor;
- whether rebound is per pass or per op;
- uplift as a real forcing.

What it **does** provide, and this scope builds on it:

1. **Dimensional rules.** Stream power's `A^0.5/L` is correctly extent-blind
   and must not be "fixed". Anything that compares a slope with an angle, such
   as a talus threshold, must be scaled by the real cell size and `peak_m`.
   §4.3 does that for the critical slope.
2. **The stability wall.** Explicit schemes cannot take a larger rate
   constant. Duration means more passes, not bigger coefficients. So
   rock-dependent rates are **multipliers inside the stable range**, and the
   hillslope multiplier is capped by `HILLSLOPE_STABLE_D`.
3. **Equilibration.** Stream power stops responding to time past about 360
   iterations. At 9 iterations it is transient, which is exactly where a
   per-cell `K` contrast shows as differential lowering.
4. **Zero uplift.** With `stream.uplift = 0` every cell only goes down.
   Differential erosion still makes **relative** relief: weak rock goes down
   faster and strong rock stands proud. It cannot make absolute relief grow.
   That is enough for this ruling, and uplift stays that scope's question.

So BH **runs on the existing pass structure** and adds no clock. Rock
**formation age** is carried as a relative stratigraphic order (§2.4), not in
years. §9 Q4 asks the owner to confirm this reading of the ruling.

### 1.5 Where rock already reaches the product

These consumers already read rock, and will read the new model instead:
- `render.rs` draws per-rock colour (`litho_palette`, `litho_strength`,
  `litho_exposure`) and microtexture (`litho_microtexture`: granite speckle,
  limestone karst pitting, sandstone and shale strata banded by elevation).
- The Sample panel shows `lithology` and `resistance`.
- The landmark generator's *rock formation* kind is a differential-erosion
  landmark. It reads `resistance ≥ 0.55` and a contrast of ≥ 0.20.
- The *cave* kind is unbuilt, because *"nothing in it carves hollows"*.
- The sculpt presets include a "Karst" stamp (`sculpt.rs`), which is an
  ordinary hills stamp.

---

## 2. The lithology model

### 2.1 Principles

1. **Computed before any erosion, from causes only.** The inputs are:
   - plate crust (`crust_field`) and plate identity;
   - the boundary type and distance to it (a labelled distance transform, §2.4);
   - convergence stress;
   - margin distance (`age_field`, used under its real meaning);
   - volcanic intensity and **setting**;
   - the pre-erosion surface (structural lows collect sediment);
   - latitude;
   - a seeded facies field.

   **No input is a product of erosion or of present climate.** This removes
   the legacy circularity, where "limestone" meant "wet post-erosion lowland".
2. **The column has explicit layers** (§2.5). A caprock over a substrate is
   recorded as such, with the contact's elevation.
3. **Properties come from one table**, indexed by rock type (§2.3). No
   per-property raster is stored. Each process looks up the rock exposed at
   the cell.
4. **Never encode "no value" as a plausible value** (`MISTAKES.md`). A
   single-layer cell has *no* contact. It is not a contact at `0.0` or `−1`,
   because both are valid normalised elevations. Readers go through an
   accessor that returns `Option`.

### 2.2 Rock types (proposed: 11; §9 Q1)

| # | Rock | Where it comes from (setting) | Projects to legacy class (civ, render) |
|---|---|---|---|
| 0 | **Granite / granitoid** | Old continental interior (high margin distance); batholith belts behind continental arcs | 0 Granite |
| 1 | **Gneiss** | High-grade core of a collision orogen (continental `COLLISION`, high stress, small margin distance) | 6 Metamorphic |
| 2 | **Schist / slate** | Flanks of an orogen, and sheared `TRANSFORM` zones on continental crust | 6 Metamorphic |
| 3 | **Plateau (flood) basalt** | Hotspot and rift provinces on continental crust. Layered lava, often over sediment: **the Trotternish case** | 1 Basalt |
| 4 | **Oceanic basalt** | Oceanic plates; `ARC_OO` basement | 1 Basalt |
| 5 | **Andesite** | Arc volcano cores (`SUBDUCTION_OC`, `ARC_OO`, and the `Arc` setting) | 2 Andesite |
| 6 | **Tuff / volcaniclastic** | Arc volcano flanks and aprons (outer edifice radius) | 2 Andesite |
| 7 | **Limestone** | Sedimentary cover in structural lows at low latitude (warm shallow seas), by facies field | 3 Limestone |
| 8 | **Sandstone** | Sedimentary cover, coarse facies; rift-basin fill | 4 Sandstone |
| 9 | **Shale / mudstone** | Sedimentary cover, fine facies. The default **substrate** under a sedimentary or volcanic cap | 5 Shale |
| 10 | **Unconsolidated** (alluvium, talus, marine sediment) | Not a pre-erosion rock. It is **regolith** written by deposition during generation (§2.5) | 5 Shale (a lossy projection, and disclosed) |

Impact breccia (from `impact_field`), evaporite and dolomite are left out of the
first build (§9 Q1, Q10).

### 2.3 Properties per rock type

**Most per-rock numbers below are judgements.** A literature ordering or range
constrains each one, and each is tuned by a GF-0 measurement. The columns:

- **s** — normalised strength, 0–1. It is read by the relief metric, landslip
  susceptibility (BG) and the landmark rules. It is taken from the midpoint of
  the indicative Selby rock-mass-strength class: very weak < 26, weak 26–50,
  moderate 51–70, strong 71–90, very strong 91–100 (Selby 1980). The class per
  rock is a **judgement**, from typical intact strength and joint spacing.
  Selby's score is site-specific and is not a property of a rock name.
- **κ** — the stream-power `K` multiplier. The ordering follows Stock &
  Montgomery (1999), whose calibrated `K` spans several orders of magnitude
  across lithologies: granitoids and metamorphics lowest, volcaniclastics and
  mudstones highest. It also follows Sklar & Dietrich (2001), whose abrasion
  experiments give erosion rate ∝ (rock tensile strength)⁻². The values are a
  **judgement**, compressed to a 0.3–4 range (13×) so that 9 iterations stay
  stable and legible. §4.1's contrast exponent tunes the spread.
- **θc** — the critical (threshold) hillslope angle in degrees. It is a
  **judgement** anchored to Selby's strength-equilibrium relation between
  rock-mass strength and slope angle (Selby 1980, 1993). For loose debris it is
  the angle of repose, about 30–40° in the standard texts (e.g. Carson & Kirkby
  1972). Verify that figure at the source before pinning it.
- **γ** — the glacial erodibility multiplier (abrasion plus quarrying).
  Quarrying is controlled by fracture spacing: Dühnforth et al. (2010); in NW
  Scotland, Krabbendam & Glasser (2011). Abrasion follows Hallet (1979). The
  values are a **judgement**.
- **ρ** — the cliff-retreat multiplier. Sunamura (1977, 1992) makes retreat
  rate depend on wave force relative to rock compressive strength, with
  retreat rates spanning roughly four orders of magnitude from crystalline
  rock to unconsolidated deposits. The values are a **judgement**, compressed.
- **σ** — solubility class. Carbonate dissolution is the karst process (Ford &
  Williams 2007).
- **P** — permeability class, following the hydraulic-conductivity ranges in
  Freeze & Cherry (1979), Table 2.2. It lowers surface runoff and so drainage
  density (Carlston 1963).
- **J** — the jointing and layering class.

| Rock | Selby class (s) | κ | θc (°) | γ | ρ | σ | P | J (jointing / layering) | Hoek–Brown `mi` (Marinos & Hoek 2000) |
|---|---|---|---|---|---|---|---|---|---|
| Granite | strong–very strong (0.85) | 0.30 | 60 | 0.5 | 0.10 | 0 | low (fractured: moderate) | massive; widely spaced sheeting joints | 32 |
| Gneiss | strong (0.80) | 0.35 | 55 | 0.6 | 0.15 | 0 | low | foliated; moderate spacing | 28 |
| Schist / slate | moderate (0.60) | 0.80 | 40 | 1.2 | 0.50 | 0 | low | foliated; close spacing, anisotropic | 12 |
| Plateau basalt | strong (0.75) | 0.40 | 60 | 1.3 | 0.20 | 0 | high (vesicular, jointed) | columnar; flow-layered, so **bedded** | 25 |
| Oceanic basalt | strong (0.75) | 0.40 | 45 | 1.0 | 0.20 | 0 | high | pillowed, blocky | 25 |
| Andesite | moderate–strong (0.70) | 0.50 | 45 | 1.0 | 0.30 | 0 | moderate | blocky | 25 |
| Tuff | weak (0.35) | 2.0 | 35 | 1.5 | 3.0 | 0 | moderate | bedded, weak | 13 |
| Limestone | moderate–strong (0.70) | 0.60 | 55 | 1.0 | 0.30 | **1** | high (karst) | bedded; jointed | 9–10 |
| Sandstone | moderate (0.60) | 0.80 | 50 | 1.1 | 0.60 | 0 (calcareous cement: low) | moderate | bedded; widely jointed | 17 |
| Shale / mudstone | weak (0.30) | 2.5 | 30 | 1.8 | 1.5 | 0 | very low | fissile, thinly bedded | 6 (mudstone 4) |
| Unconsolidated | very weak (0.10) | 4.0 | 33 | 3.0 | 10 | 0 | high (sand, gravel); low (clay, till) | none | n/a |

Where a value is a literature figure rather than a judgement, it is the
Hoek–Brown `mi` column, and the Selby class boundaries. **Before GF-1 pins
them, GF-1's lane opens the cited table and checks each figure.** It replaces
any figure it cannot verify, and says so. The bibliography is in §11.

### 2.4 Derivation rules (GF-1)

**Inputs.** All of these exist before erosion:
- `crust_field`, `plate_id`
- `boundary_mask`, `boundary_type`, `stress_field`, `shear_field`
- `age_field`, the **margin distance**, named as such in the new code
- `volcanic_field`, plus a new per-cell `volcanic_setting` raster. It is the
  setting of the edifice that won the `max` in the stamper. Today the setting
  is discarded; storing it is plumbing, not new classification.
- the pre-erosion normalised `field`, and `sea_level`
- latitude, from `climate.lat_n` and `climate.lat_s`
- a seeded low-frequency facies noise (`fbm`, `pfbm` under world-wrap)

**Steps.**

1. **Labelled boundary distance.** A two-pass chamfer transform, like
   `build_age_field`'s, that also carries the **type** of the nearest boundary
   cell, and the crust sign on that side. It gives `(d_margin, nearest_type)`.
   It wraps in x for world maps. `build_age_field` does not wrap, and its
   golden pins that. The new transform is a separate function, so the golden
   is untouched.
2. **Basement** (the deepest unit):
   - Oceanic plate → oceanic basalt.
   - Continental, nearest type `COLLISION` and `d_margin < w_core` → gneiss.
     Between `w_core` and `w_orogen` → schist.
   - Continental, nearest `SUBDUCTION_OC` (continental side) within the arc
     belt → granite (the batholith).
   - Continental, nearest `TRANSFORM` within `w_shear` → schist.
   - Continental, nearest `RIFT` within `w_rift` → sandstone and shale basin
     fill over basement. That is a two-layer column.
   - Otherwise → granite, the old interior.

   The widths `w_*` are fractions of `tect.blur_r` in cells. That is the same
   scale the stress blur already sets, so belts scale with plate size. Their
   values are **judgements**, tuned in GF-1 by the rock-type-share report
   (§5, bar B10).
3. **Sedimentary cover.** This applies to continental cells in **structural
   lows**, where the pre-erosion height is below its own wide blur by more
   than a threshold, and more than `w_orogen` from an active margin:
   - Cover thickness is proportional to depression depth, times the facies
     noise.
   - Cap facies: limestone where |latitude| is below a carbonate band
     (**judgement**, since present latitude stands in for palaeolatitude, and
     this is disclosed); otherwise sandstone.
   - The cover's lower unit is shale.
   - So a thick cover is a two-layer column, strong cap over weak shale.
   - A thin cover leaves basement exposed.
4. **Volcanic overlay.** This applies where `volcanic_field > v_th`:
   - Hotspot or Rift setting → a **plateau basalt cap** over whatever was on
     top. The previous top becomes the substrate. Sediment under basalt is the
     Skye stratigraphy.
   - Arc setting → andesite at the edifice core, tuff at the outer radius.
   - Cap thickness is proportional to volcanic intensity × edifice height.
5. **Formation age.** A relative ordinal per unit, with no years:
   basement < cover < volcanic < regolith. It is what "age layers" means here
   until a clock is ruled (§9 Q4).

### 2.5 The column

Per cell:

| Raster | Type | Meaning |
|---|---|---|
| `rock_top` | `u8` | Uppermost bedrock unit (the cap, or the only unit) |
| `rock_sub` | `u8` or `NO_LAYER` | The unit beneath the contact. `NO_LAYER` (a named constant, never a valid rock index) means a single-layer column |
| `contact` | `f32` | **Absolute** normalised elevation of the cap/substrate contact. Meaningful only where `rock_sub != NO_LAYER`. Every reader goes through `Column::substrate(i) -> Option<(Rock, f32)>` |
| `regolith` | `f32` | Thickness of unconsolidated material on bedrock, in normalised units. `0.0` is a real value: bare rock |

**The exposure rule**, used by every process, is:

```
exposed(i) = Unconsolidated          if regolith[i] > 0
           = rock_top[i]              if single-layer, or field[i] ≥ contact[i]
           = rock_sub[i]              otherwise
```

Erosion strips regolith first, then the cap. Once the cap is breached, the
substrate is exposed. Nothing else is needed for scarps to emerge: the weak
substrate lowers faster (κ), and the cap edge keeps a steep face (θc).
Layered-landscape modelling shows the same behaviour: Forte et al. (2016);
Perne et al. (2017).

Contact geometry is horizontal, plus a gentle low-frequency warp so that the
outcrop line is irregular. Dip is §9 Q8.

**Memory**, by arithmetic: 1 + 1 + 4 + 4 = 10 B per cell, plus 1 B per cell for
`volcanic_setting`. At 2048 × 1311 that is 29.5 MB. It is stored in the save
(`substrate.rs`, `SAVEFILE_COMPAT.md`, GF-1).

---

## 3. The new stage order

### 3.1 Dependency graph

```
plates ─ assign ─ stress ─┬─ flexure ─┐
   │                      ├─ age (margin distance) ─ heterogeneity ─┐
   │                      └─ boundary_type, shear                   │
   └─ crust (base_raw) ─ base_field ────────────────────────────────┼─ height ─ normalize
                                                                    │
volcanism (+ setting kept) ─ craters ─ clamp ─ sea level ◄──────────┘
        │
        ▼
 ┌──────────────── GEOLOGY (new, GF-1) ────────────────┐
 │ labelled boundary distance → basement → cover       │  reads only the
 │ → volcanic overlay → column (top, sub, contact)     │  above; no climate,
 │ → regolith = 0                                      │  no erosion output
 └─────────────────────────────────────────────────────┘
        │
        ▼
 priming flow(area) → temperature → weather → correctors (→ currents)
        │
        ▼
 ┌──────────────── LANDFORM PROCESSES (each reads exposed rock) ──────────┐
 │ stream power (κ; contact switch in-loop) → rebound (moves the column)  │
 │ → threshold hillslope (θc, extent-scaled)                   [new, GF-3]│
 │ → karst dissolution (σ, runoff)                             [new, GF-5]│
 │ → glacial (γ) → rebound                                  [moved, GF-4] │
 │ → coastal (ρ)                                                   [GF-4] │
 │ → [velocity, hillslope diffuse, evolve cycles, sediment fill, tidal]   │
 │   deposition → regolith                                                │
 └────────────────────────────────────────────────────────────────────────┘
        │
        ▼
 routing → channels (flow uses permeability-reduced runoff, GF-5)
 → Strahler → trace → intensity → carve_channel_network (half-width ω, GF-6)
        │
        ▼
 climate refresh (flow_discharge, temperature, weather, correctors)
        │
        ▼
 outside generate_terrain: civ affordances read the stored column
 (projected to the 7 legacy classes) instead of build_lithology
```

### 3.2 What moves, and why

1. **The geology stage is inserted after volcanism, craters and sea level, and
   before the first flow.** Every input is final by then. No climate or erosion
   output is available to it, by construction.
2. **Every landform process runs before the final hydrology trace.** Today, in
   the app, glacial erosion runs after the channels are traced (§1.1). Moving
   it, and the other `passes.*`, ahead of `build_channels_routed` means the
   network and carve describe the final surface.
   - The light stream-power pass stays gated by `carve_rivers`, as today.
     Changing that gate would change `carve_rivers = false` worlds, which is
     outside this ruling.
3. **Weathering and erosion of the cap expose the substrate** through the
   exposure rule. It is re-evaluated after every process, and *inside* the
   stream-power solve (§4.1).
4. **Rebound lifts the column.** `isostatic_rebound` adds `0.8·blur(...)` to
   `field`. GF-2 adds the same increment to `contact` and to the bedrock
   surface. Otherwise rebound would re-bury an exposed substrate, or expose one
   that erosion never reached.
5. **Deposition becomes regolith.** Stream power's internal deposition,
   `route_sediment`, the tidal flats, the marsh bump and the talus moved by the
   threshold hillslope all write their net height gain to `regolith`.
6. **`dynamic_lithology` is superseded.** The explicit column replaces the
   exhumation heuristic. §9 Q9 asks the owner to confirm removing the toggle
   (§7p).
7. **No clock** (§1.4). Pass counts stay as they are, except where §4 adds a
   pass and GF-0 measures its cost.

---

## 4. Per-process changes, with formulas

**One mechanism applies throughout.** Each kernel gains an **optional**
per-cell coefficient slice. `None` takes the existing arithmetic, by control
flow. So every kernel golden stays bit-identical by construction, not by
coincidence (`MISTAKES.md`: *"identity by control flow beats identity by
arithmetic"*).

### 4.1 Stream power (GF-2)

Today:

```
Cc_i = K·g·max(0.05, 1 − 0.7·resist·R_i)·(1 + 2·ck·rain_i)·dt·A_i^m / L_i
```

It becomes:

```
Cc_i = K·g·κ(exposed_i)^c ·(1 + 2·ck·rain_i)·dt·A_i^m / L_i
```

- `c` ∈ 0–1 is the **rock contrast**. `c = 0` gives uniform rock. The existing
  "Rock resistance" control (`tect.resist`) is repurposed as `c`, so the GUI
  gains nothing new (§9 Q5).
- **The contact switch inside the implicit loop.** The kernel precomputes
  `Cc_cap` and `Cc_sub`. After it updates `fld[i]`, if the column has a
  substrate and `fld[i] < contact[i]`, it sets `Cc[i] ← Cc_sub[i]`.
  - Receivers, drainage area and the fill stay as they are: they are frozen
    before the iterations today.
  - The cost is one comparison per cell per iteration.
  - Without the switch, a cap breached in iteration 3 would erode at cap
    rates until the next process, because the default path calls the kernel
    once.
- `tile_erode` (EF-3) takes the same slice, resampled the way its test
  resamples `resist` today, whenever EF-3 gains a production caller.
- **GPU.** `GPU_STREAM_POWER_SCOPE.md` (SP-G) must take the κ slice and the
  contact switch. Its per-cell `Cc` phase already runs per cell.

### 4.2 Isostatic rebound (GF-2)

```
field += 0.8·b
contact += 0.8·b
```

The bedrock surface moves with it, where `b = blur(max(0, pre − field))`. The
kernel's own output is unchanged. The column update runs in the caller.

### 4.3 Threshold hillslope (GF-3; a new generation stage)

This is `erode_thermal`'s rule with a per-cell threshold. It is physically
scaled, which fixes the extent-blindness §1.3 notes, for this use.

```
talus_i = tan(θc(exposed_i)) · cell_m · (1 − sea) / peak_m
```

- `cell_m = map_width_km·1000/gw`.
- Excess over `talus_i` moves downslope as in `erode_thermal`: `0.5·0.25` of
  the excess, split by excess share.
- **The moved mass becomes regolith** where it lands, which is the talus
  apron.
- The number of passes, `N_h`, is a **judgement**. GF-3 measures it against
  the cap-edge bar (B4) and the cost bar (B9).
- The scalar `erode_thermal` stays the manual Erode op's kernel. Its golden
  (`golden_parity_thermal.rs`) is untouched, because the per-cell form is an
  `Option`.

**What to expect.** At 800 km over 2048 cells, a cell is about 390 m wide
(arithmetic). A cell-mean slope above about 40° is then a steep feature, so
the threshold binds mainly at cap edges and at finer extents. GF-0 measures the land slope
distribution at 80, 800 and 8 000 km before GF-3 fixes any value.

### 4.4 Glacial (GF-4)

```
e_i = kg · γ(exposed_i)^c · g · Q_i^mg · S_i · 0.001
```

The flanking `u_factor` and the cirque `0.6` terms scale with the same `e_i`.

### 4.5 Coastal (GF-4)

```
wave cut:  Δ_i = − wave_str · exposure · 0.002 · ρ(exposed_i)^c
debris:    +wave_str · exposure · 0.0005 · ρ(exposed_i)^c  to each land neighbour, written to regolith
estuary:   cut_i × ρ(exposed_i)^c
```

The coastal pass is **off at both boundaries** today. Rock-aware cliff
retreat, and BG's sea stacks, need it on in the app (§9 Q7).

### 4.6 Karst and permeability (GF-5)

```
dissolution:  Δz_i = − k_k · σ(exposed_i) · rain_i        (land cells only)
runoff:       rain_eff_i = rain_i · (1 − φ · P(exposed_i))
```

- `rain_eff` is used for the **channel network's** flow.
- `k_k` and `φ` are **judgements**, tuned by B5 and B8. The rate is
  proportional to runoff, which is the standard control on carbonate
  denudation (Ford & Williams 2007).
- The dissolution pass may leave closed depressions (dolines). With
  `integrate_drainage` on they route through the fill, which is correct for
  karst: the water goes underground.
- **Spring line.** Where a permeable cap sits over an impermeable substrate and
  the contact outcrops, GF-5 records the cell as spring-line potential. The
  landmark *spring* reads it.

### 4.7 River carve (GF-6)

```
half_w = min(4·width_k, (0.8 + 0.5·(o − 1))·width_k) · ω(run)
```

- `ω` is the median over the run of a per-rock valley-width factor: narrower
  in strong rock, wider in weak. Bedrock-channel width varies with rock
  resistance (Montgomery & Gran 2001). The values are a **judgement**.
- The per-step `drop = 0.0006`, `CARVE_LAND_MARGIN` and `CARVE_KEEP_LAKE_DEPTH`
  **do not change.** They are the hydrological guarantees RV-1 measured.
- **Knickpoints.** Where a run crosses from cap to substrate, a waterfall
  landmark candidate is recorded.
- RV-3's smooth valley cut, when built, uses the same `ω`.

### 4.8 Volcanism and craters (GF-1)

These stay producers:
- **Volcanism** additionally writes `volcanic_setting`, and the §2.4 step 4
  overlay. `volcanic_field` is unchanged, which keeps the pipeline and
  province goldens bit-identical.
- **Craters** keep `impact_field` as it is. It is not a rock type in the
  first build (§9 Q10).

### 4.9 Sediment routing and deposition (GF-2 and GF-4)

The kernels do not change. The caller adds `max(0, field_after − field_before)`
to `regolith` for each deposition step. When a deposit is eroded, regolith is
consumed before bedrock.

### 4.10 LOD detail (GF-6)

```
amp_i = detail_amp · α(exposed_i)
style = ridged        where J is massive or columnar
        terraced      (a small height quantisation) where J is bedded and
                      the cell is within one coarse cell of a cap edge
        fbm           otherwise
```

- `α` and the terrace step are **judgements**, tuned by eye against the
  owner's reference photograph.
- There is also a numeric bar: a terrace may never move the coastline. The
  existing `refinement_never_moves_the_coastline_in_either_direction` test
  applies to it.
- This ties into `LOD_DETAIL_SCOPE.md` EF-9 and BG's pinnacles.

### 4.11 Landmarks (GF-6)

- *Rock formation*: `ROCK_FORMATION_MIN_RESISTANCE` reads **s** instead of
  `resistance_field`.
- *Cave*: becomes buildable on soluble rock that karst has lowered. Its
  `not_built` reason is removed in the same change.
- *Spring*: reads the spring line.
- *Waterfall*: reads the knickpoints.

`DECISIONS.md` §7q already exempts landmarks from parity.

---

## 5. Measurement proving landforms now track rock

### 5.1 The harness (GF-0)

A Rust test-side harness, run `--ignored` and alone, prints each metric per
seed and extent.

- **Seeds:** 483920, 24601 and 71077345, which are `_riverzoom_probe`'s and
  the RV-1 measurement's. Plus 12345 and 314159, the golden seeds.
- **Extents:** 80, 800 and 8 000 km at the app's default grid.
- **Parameters:** `cartalith_godot::params::defaults`.

**Two arms, on one binary.**
- **Control.** The geology stage runs and the column is stored, but every
  process ignores it: `c = 0` and no new stages. This is the correct "before".
  The legacy `build_lithology` **cannot** serve as the baseline, because it
  defines sediment as post-erosion lowland. "Weak rock is lowland" is then
  true by construction. `MISTAKES.md`'s rule applies: never select by the
  value under test.
- **Treatment.** The processes read the column.

Both arms use the **same pre-erosion rock map**, so every difference is the
processes' doing.

**The harness's own validity checks.** A harness that cannot fail proves
nothing, so these must pass before any bar is read:
- **A positive control.** A synthetic fixture: a strong block in weak
  surroundings, uniform rain. The treatment must give B1's ρ > 0.5.
- **A negative control.** The rock map randomly permuted. B1's |ρ| must be
  below 0.05.
- **Non-emptiness.** Every metric's population is printed, and a population
  under 100 cells marks that metric "not measurable on this seed". It is never
  reported as a pass.

### 5.2 Acceptance bars

Unless a bar says otherwise, it must hold at 800 km on **all five seeds**. **All
thresholds are judgements set before measurement.** GF-0 records the control
values. If the control already meets a bar, that bar is not evidence: it is
raised, and the change is disclosed.

**B1 — relief tracks strength.**
- **Metric:** Spearman ρ between **s** (exposed rock) and local relief (max − min
  elevation in metres over a 9 × 9 window). Land cells at least 5 cells from
  the coast.
- **Bar:** treatment ρ ≥ 0.25, and ρ_treatment − ρ_control ≥ 0.15.

**B2 — slope tracks strength.**
- **Metric:** median physical slope in degrees (the Sample panel's `grade`
  formula) on strong rock (s ≥ 0.7), divided by the median on weak rock
  (s ≤ 0.4). Selected by the **input** s.
- **Bar:** the ratio ≥ 1.5, and ≥ 1.25 × the control's ratio.

**B3 — weak rock becomes lowland.**
- **Metric:** mean erosion depth (pre-erosion minus final surface, in metres)
  on weak rock, divided by the same on strong rock. The same pre-erosion
  surface is used in both arms.
- **Bar:** ≥ 2.0 in the treatment, and ≥ 1.6 × the control's ratio.

**B4 — scarps at caprock edges.**
- **Metric:** median slope at cap-edge cells (a two-layer cell whose exposed
  rock differs from a 4-neighbour's) ÷ median slope of substrate cells within
  5 cells.
- **Bar:** ≥ 2.0 in the treatment, and ≥ 40 % of cap-edge cells in the land
  slope's top decile.
- The selection depends on the output, so the metric is paired with an
  input-selected twin: all two-layer cells against single-layer cells of the
  same top rock. That twin must move the same way.
- Reported at 80 km as well.

**B5 — karst only on soluble rock.**
- 100 % of cells lowered by the dissolution pass have σ > 0. This is exact, and
  a mutation that drops the σ factor must turn the test red.
- Of the closed depressions the pass creates deeper than the lake classifier's
  0.004 threshold, at least 95 % lie on soluble rock.

**B6 — glacial reads jointing.**
- **Metric:** at matched ice discharge (Q decile bands), the median glacial
  lowering on γ ≥ 1.5 rock divided by the median on γ ≤ 0.6 rock.
- **Bar:** ≥ 1.5 in the treatment, against about 1 in the control.

**B7 — coastal retreat reads rock.**
- **Metric:** land cells lost to the sea per coastline cell, weak rock (ρ ≥ 1.5)
  ÷ strong rock (ρ ≤ 0.3), over the default passes.
- **Bar:** ≥ 3.
- It is only measurable once §9 Q7 turns the pass on.

**B8 — no hydrology regression**, against RV-1's own metrics
(`_riverzoom_probe`):
- ocean cells on channels = **0** (hard);
- river cells that are lake ≤ control + 2 percentage points;
- 1–3-cell lakes ≤ 1.25 × control.

**B9 — cost.**
- **Metric:** median generation time of at least 5 runs at 2048 × 1311, run
  alone, reported with min..max.
- **Bar:** ≤ 1.20 × the control's median.
- If the brackets overlap parity, "no difference established" is reported.

**B10 — the model is alive.**
- The same seed twice gives byte-identical output.
- At least 4 rock types present per world at 800 km.
- Two-layer cells are at least 5 % of land. Below that, the caprock design is
  inert on real worlds, and GF-1 re-tunes before GF-2 starts.
- Each rock type's share is printed.

### 5.3 What the numbers are for

- B1–B3 are the owner's question: *"how strong they are carved"*.
- B4 is the precondition BG needs.
- B5–B7 are the per-process claims of Ruling BH item 3.
- B8–B10 are the guards.

---

## 6. Re-baseline plan

**Precedent.** RV-1 (commit `dd836f6`, Ruling BD) re-recorded port-versus-port
hashes with the old and new values disclosed. It pinned the JS-parity suites to
a captured pre-change world,
`crates/cartalith-engine/tests/fixtures/pre_rv1_world.rs` and
`pre_rv1_worlds.bin`. That kept those suites from becoming snapshots of
themselves.

**Why RV-1's pin is not enough here.**
- RV-1's pin restores six arrays: `field`, `temperature`, `rainfall`,
  `flow_discharge`, `river_floor` and `river_mask`. That works because the
  carve runs **last**.
- It keys each world on the hash of `stream_order`, which is computed before
  the carve.
- BH changes stream power, which is **upstream** of the channels. So
  `stream_order`, `channels` and the river-intensity stamp move too, and
  RV-1's identity check fails for a reason that is not a defect.

### 6.1 During the build (GF-1 to GF-6)

- The model ships behind a switch. It is **false** in
  `WorldParams::defaults` and **true** in `cartalith_godot::params::defaults`.
  That is `MISTAKES.md`'s standing convention for a change to generated output.
- No JS-parity golden moves mid-build, and GF-0's two arms run on one binary.
- The switch gets **no GUI control**. There is no user-facing legacy mode
  (§7p).

### 6.2 The final disposition (GF-7); recommended route A (§9 Q6)

**Untouched by construction.** Each must be asserted bit-identical in the
milestone that could move it:
- **All kernel goldens:** `cartalith-erosion/tests/golden_parity_{streampower,
  thermal, passes, rebound, droplet}.rs`. The new coefficients are `Option`
  slices.
- **All `cartalith-terrain` goldens**, including
  `golden_parity_flex_hetero_resist.rs` (`compute_resistance`) and
  `golden_parity_age.rs`. The geology stage is new code after them, and the
  labelled transform is a separate function.
- **`cartalith-engine/tests/golden_parity_pipeline.rs`.** It uses
  `carve_rivers: false` and no passes, so no erosion runs. GF-1's done-means
  requires that the geology stage leaves `field`, `age_field`,
  `resistance_field`, `volcanic_field` and `impact_field` bit-identical.
- **Tile, amplify and zoom-detail goldens.** Rock inputs are `Option`.

**Pinned to a captured pre-BH world.** These are JS-parity suites that
generate a world and then test a civ port against a reference capture:
- The **16 `cartalith-civ/tests/golden_parity_*` suites** that use
  `pre_rv1_world` today: affordance, biome, carrying_capacity, civ_tools,
  faction_aggregates, hierarchical_network, resource_potentials,
  road_consolidation, road_network, sea_routes, settlement_naming,
  settlement_placement, settlement_prereqs, settlement_suitability,
  smelting_salt and waterbodies.
- The pin becomes `pre_bh_world`. It captures **every** `WorldState` array a
  civ suite reads, from the last pre-BH commit.
- It keys each world on a hash of `plate_id` and `age_field`, which BH never
  moves.
- It sets the stored column to absent, so `compute_affordance_fields` takes
  the reference path (`build_lithology`).
- `golden_parity_carve.rs`'s check of the capture against the reference arrays
  carries over to the new capture.

**Re-recorded with old → new disclosed in the commit:**
- `cartalith-engine/tests/golden_parity_carve.rs`'s port-versus-port field
  hashes (RV-1 already records its old and new pairs in the file).
- `cartalith-engine/tests/world_structure_orogeny.rs`'s generated-world hashes.
- Any test that runs `cartalith_godot::params::defaults` with a value
  assertion. GF-7 opens each candidate and classifies it:
  - `lod_d4_ice_and_snow.rs`
  - `pass_relief_measure.rs`
  - `appearance_ab_dump.rs`
  - `params_mapping.rs`
  - the save and project round trips, which check round-trip identity, not
    values, and so are expected not to move.
- The measured figures other documents quote from generated worlds. Each is
  re-measured, and old → new is disclosed where the figure lives:
  - RV-1's `_riverzoom_probe` numbers;
  - the LOD-D4 bars;
  - `PERFORMANCE_BENCHMARKS.md`'s generation times;
  - `GENERATION_PARAMETERS.md`'s cost of the deliberate divergences.

**New golden files** for the model itself, such as the column on fixed seeds.
There is no reference for them, so they are **regression pins, labelled as
such**, like landmarks under §7q. Each is mutation-tested: every table value,
the exposure rule and the contact switch.

**Scrubbed, route A.** The switch is deleted, and `generate_terrain` has one
path. `build_lithology` stays as the ported, golden-verified reference function.
It is the civ parity path, and the fallback for worlds with no column: saves
from before GF-1. `dynamic_lithology` and its GUI row go, pending Q9.

**Route B** is the alternative. It keeps §6.1's split permanently, as the six
existing divergences do. See §9 Q6 for the trade-off.

---

## 7. Milestones

### GF-0 — the measurement harness

- The harness of §5.1. Metric functions take any rock map, so the harness
  exists before the model does.
- The positive and negative controls.
- A first run on today's tree, using `resistance_field` quartiles as a
  stand-in strength. It is labelled as such: it is not the control arm.
- The land-slope distribution at 80, 800 and 8 000 km, measured, to settle
  §4.3's expectation.

**Done means:**
- The harness runs on all five seeds and three extents, and prints every
  metric with its population.
- The positive control passes and the negative control reads |ρ| < 0.05.
- The first run's numbers are recorded in `STATUS.md` with the command that
  produced them.

### GF-1 — the lithology model and column, read by no process

- §2.4's derivation.
- `volcanic_setting` stored.
- The column of §2.5 on `WorldState`, behind §6.1's switch.
- Save format: new `rasters/` entries, and `SAVEFILE_COMPAT.md` updated. An
  old save has no column. Every reader dashes it with the reason "saved before
  geology", and civ falls back to `build_lithology`.
- The import path (`cartalith_engine::import`) and Ruling M's substrate rebuild
  build a single-layer column from the inferred plates. A layered import is
  not in scope.
- The Sample panel and the lithology map view (§9 Q3).
- The table's literature values verified at their sources (§2.3).

**Done means:**
- With the switch on, `field` and every pre-existing array are
  **bit-identical** to the switch off. That is identity by control flow,
  asserted by hash.
- `golden_parity_pipeline.rs` is untouched.
- B10 passes. The column round-trips through a save.
- No reader shows a bare `0`, `1.0`, `"none"` or empty for an absent contact.
- `cargo test --workspace`, and `--check-only` on every touched `.gd` plus
  `shell/app.gd`.

### GF-2 — stream power and rebound read rock; deposition becomes regolith

§4.1, §4.2, §4.9.

**Done means:**
- B1, B2, B3, B8 and B9 pass.
- The kernel goldens are bit-identical.
- Mutation-killed: κ lookup, contact switch, rebound-moves-column,
  regolith-first stripping.
- The DLL and `.rs` mtimes are stated at the end of the run.

### GF-3 — the threshold hillslope stage

§4.3.

**Done means:**
- B4 passes at 800 km and is reported at 80 km. B8 and B9 still pass.
- `golden_parity_thermal.rs` is bit-identical.
- The critical-angle scaling is tested at two extents with a literal expected
  talus. It is never asserted against its own constant.

### GF-4 — glacial and coastal read rock; the passes move ahead of the trace

§4.4, §4.5, §3.2 item 2.

**Done means:**
- B6 passes. B7 passes if Q7 is ruled yes; otherwise it is reported as not
  measurable, with that reason.
- B8 still passes.
- `golden_parity_passes.rs` is bit-identical.

### GF-5 — karst and permeability

§4.6.

**Done means:**
- B5 passes, including its mutation.
- B8 passes, since permeability changes drainage density on purpose. The
  change in river-cell count per rock class is reported.
- Spring-line cells are non-empty on at least one seed that has a permeable
  cap.

### GF-6 — carve, LOD detail and landmarks read rock

§4.7, §4.10, §4.11.

**Done means:**
- RV-1's `carve_network_*` tests are still green, and B8 still passes.
- Deep-zoom tiles are unchanged with the rock inputs absent (the amplify and
  zoom goldens).
- A windowed probe shows terraced cap edges and ridged granite at deep zoom.
  Its positive control must move pixels.
- *Cave* is buildable on karst, and its `not_built` reason is removed.

### GF-7 — re-baseline, projection and scrub

- §6.2 per Q6's ruling.
- The civ projection to the 7 legacy classes, used by soil, resources, render
  and export.
- `DECISIONS.md` gains an entry recording the new stage order and the
  superseded heuristic.
- `GENERATION_PARAMETERS.md`, `SAVEFILE_COMPAT.md` and the tooltips are
  updated, and the prose describing the old behaviour is hunted down
  (`MISTAKES.md`: *"Change behaviour"*).

**Done means:**
- Every golden is classified as untouched, pinned or re-recorded, with each
  old → new pair in the commit.
- The switch is deleted (route A), or documented (route B).
- `cargo test --workspace` is green, with the result-line count checked
  against the previous floor.
- `STATUS.md` is updated.

*A possible GF-8, lithology painting, is §9 Q2's; it is not scheduled here.*

---

## 8. Constraints any lane on this scope must carry

These are from `MISTAKES.md`'s preflight table, matched to this work:

- Re-open each cited symbol before acting on it. This document is dated
  2026-09-27.
- Never encode "no value" as a plausible value. That covers the contact,
  pre-geology saves and imports.
  - An existing instance: `sample_bridge`'s `CellSample` fills `stress`,
    `resistance` and `drainage` with `unwrap_or(0.0)`.
  - The `build_lithology` call just above it indexes `f.resistance_field[i]`
    directly, so a short resistance field panics before that fallback is
    reached.
  - GF-1 replaces the fallback with a dash and a reason.
- Never assert a constant against itself. Mutation-test with Python exact
  replace, restore in `finally`, and hash-check afterwards.
- Timings: medians with min..max, run alone.
- `cargo test --workspace` cannot see a broken shell. Parse-check every
  touched `.gd`.
- A Godot probe after a `.rs` edit needs `cargo build` first. State both
  mtimes.
- No `git stash` and no `git checkout --` while lanes run. Compare against
  HEAD in a worktree.
- If the permission system denies an action, stop and report it.
- Searches stay inside `C:\Users\Vincent\Cartalith_GDT`.

---

## 9. Owner questions, each with a recommended default

`LARGE_ITEM_RULINGS.md` was searched on 2026-09-27 for each of these. None is
already ruled.

1. **How many rock types?** **Default: the 11 in §2.2.** They project to the 7
   legacy classes for civ, soil and resources until those tables are extended.
   - The alternatives are 7, reusing the legacy classes, which cannot express
     a strong cap over weak rock inside the sedimentary group, or about 16:
     adding quartzite, dolomite, chalk, evaporite, conglomerate and impactite.
2. **Can the user paint lithology?** **Default: not in this build.** A later
   GF-8 "Geology paint" tool would paint the top rock and cap thickness, with
   the processes re-run over the painted column. It needs its own short scope,
   for how painting interacts with sculpt and regeneration.
   - Until then, sculpting down exposes the substrate automatically through
     the exposure rule. Sculpting up extends the top rock.
   - Ruling M's drawn plates inform resources only, so they would not feed
     this.
3. **How does the Sample panel show lithology?** **Default:**
   - **Rock (surface)**: the exposed rock.
   - **Beneath**: the substrate, and the contact depth in metres. Dashed
     "single-layer column" where there is none.
   - **Strength**: the Selby class name.
   - **Soluble** and **Permeable** flags.
   - **Regolith**: thickness in metres.
   - The legacy *Lithology* row becomes the surface rock name.
   - The lithology map view gets an 11-colour palette.
4. **The clock.** Ruling BH names the geological-time scope's clock, and that
   scope has none (§1.4). **Default: BH builds no clock.** Formation age is
   relative, and a clock remains its own ruling, with that scope's §8
   prerequisites.
5. **Contrast control.** **Default:** repurpose the existing "Rock resistance"
   slider (`tect.resist`) as the contrast exponent `c`. Its default is set by
   GF-2's measurement. There are no per-rock sliders; the table is editable in
   code only.
6. **Final re-baseline route.** **Default: route A** (§6.2): one pipeline;
   the civ parity suites pinned to `pre_bh_world`; engine full-pipeline hashes
   re-recorded with disclosure; the switch deleted. It matches Ruling BH's
   *"re-baselines with every old → new disclosed"*, the RV-1 precedent, and
   §7p's *"scrub, don't dual-path"*.
   - Route B keeps the `WorldParams::defaults` / app split permanently, like
     the six existing divergences. It is cheaper to maintain, and it keeps a
     parity-baseline generation path in the engine.
7. **Turn the coastal pass on in the app?** It is off at both boundaries
   today. Rock-aware cliff retreat and BG's sea stacks need it. **Default: yes,
   at GF-4**, with its measured cost disclosed, as Ruling AU did for glacial.
8. **Layer dip?** **Default: horizontal with a gentle low-frequency warp.**
   Per-province dip, which gives cuestas and hogbacks, can come later.
9. **Remove `tect.dynamic_lithology`?** The explicit column supersedes the
   exhumation heuristic. **Default: yes, at GF-7** (§7p). Its GUI row goes and
   the parameter is dropped.
10. **Impact breccia as a rock type?** **Default: no.** `impact_field` stays a
    marker. It can join Q1's extended set later.

---

## 10. How Ruling BG slots in

BG's scope should take these from the GF-1 column. **Ruling BG's statement that
today's generator already makes "stepped basalt escarpments" is not verified
here.** BG's scope should check it with B4's metric on a seed that has a basalt
cap.

**Landslip susceptibility inputs** (per cell, all available after GF-3):
- **Strength contrast:** `s(cap) − s(sub)`. It is large for basalt or
  sandstone over shale.
- **Cap thickness:** `surface − contact`, in metres.
- **Cap-edge exposure and slope:** the outcrop cell, and slope relative to
  `θc(sub)`. A face steeper than the substrate can hold is undercut.
- **Permeability contrast:** a permeable cap over an impermeable substrate.
  Pore pressure at the contact, the spring line, is the classic trigger.
- **Jointing of the cap:** columnar or blocky caps shed blocks, which BG's
  pinnacles can survive on.
- **Glacial debuttressing:** the depth the glacial pass removed next to the
  scarp. This is the paraglacial rock-slope-failure setting; BG's scope owns
  its literature (e.g. Ballantyne 2002).
- **Coastal exposure:** the coastal pass's wave cut on strong rock, for sea
  stacks.

**Pinnacle survival:** s ≥ 0.7, with J columnar or massive, on slipped blocks
or cliff remnants.

**Rock names for landmark text:** the new rock names replace `LITH_NAMES` in
the causal fact.

**Mass-wasting stage:** BG's slump pass slots in after GF-3's threshold
hillslope and before the final trace (§3.1). It writes its debris to
`regolith`.

---

## 11. Bibliography

Cited as known. GF-1's lane verifies the page, table or figure for every value
before it is pinned, and replaces any reference it cannot verify.

- Ballantyne, C.K. (2002). Paraglacial geomorphology. *Quaternary Science
  Reviews* 21, 1935–2017.
- Braun, J. & Willett, S.D. (2013). A very efficient O(n), implicit and
  parallel method to solve the stream power equation governing fluvial
  incision and landscape evolution. *Geomorphology* 180–181, 170–179.
- Carlston, C.W. (1963). Drainage density and streamflow. *USGS Professional
  Paper* 422-C.
- Carson, M.A. & Kirkby, M.J. (1972). *Hillslope Form and Process.* Cambridge
  University Press.
- Dühnforth, M., Anderson, R.S., Ward, D. & Stock, G.M. (2010). Bedrock
  fracture control of glacial erosion processes and rates. *Geology* 38(5),
  423–426.
- Ford, D.C. & Williams, P.W. (2007). *Karst Hydrogeology and Geomorphology.*
  Wiley.
- Forte, A.M., Yanites, B.J. & Whipple, K.X. (2016). Complexities of landscape
  evolution during incision through layered stratigraphy with contrasts in
  rock strength. *Earth Surface Processes and Landforms* 41, 1736–1757.
- Freeze, R.A. & Cherry, J.A. (1979). *Groundwater.* Prentice-Hall (Table 2.2,
  hydraulic conductivity by material).
- Hallet, B. (1979). A theoretical model of glacial abrasion. *Journal of
  Glaciology* 23(89), 39–50.
- Hallet, B. (1996). Glacial quarrying: a simple theoretical model. *Annals of
  Glaciology* 22, 1–8.
- Harel, M.-A., Mudd, S.M. & Attal, M. (2016). Global analysis of the stream
  power law parameters based on worldwide ¹⁰Be denudation rates.
  *Geomorphology* 268, 184–196.
- Hoek, E. & Brown, E.T. (1980). Empirical strength criterion for rock masses.
  *Journal of the Geotechnical Engineering Division, ASCE* 106(GT9),
  1013–1035.
- Krabbendam, M. & Glasser, N.F. (2011). Glacial erosion and bedrock
  properties in NW Scotland: abrasion and plucking, hardness and joint
  spacing. *Geomorphology* 130, 374–383.
- Marinos, P. & Hoek, E. (2000). GSI: a geologically friendly tool for rock
  mass strength estimation. *Proc. GeoEng2000*, Melbourne (the `mi` table).
- Montgomery, D.R. & Gran, K.B. (2001). Downstream variations in the width of
  bedrock channels. *Water Resources Research* 37(6), 1841–1846.
- Perne, M., Covington, M.D., Thaler, E.A. & Myre, J.M. (2017). Steady state,
  erosional continuity, and the topography of landscapes developed in layered
  rocks. *Earth Surface Dynamics* 5, 85–100.
- Roering, J.J., Kirchner, J.W. & Dietrich, W.E. (1999). Evidence for
  nonlinear, diffusive sediment transport on hillslopes and implications for
  landscape morphology. *Water Resources Research* 35(3), 853–870.
- Schmidt, K.M. & Montgomery, D.R. (1995). Limits to relief. *Science* 270,
  617–620. It links rock-mass strength to the relief a range can hold, which
  is B1's premise.
- Selby, M.J. (1980). A rock mass strength classification for geomorphic
  purposes: with tests from Antarctica and New Zealand. *Zeitschrift für
  Geomorphologie* 24(1), 31–51.
- Selby, M.J. (1993). *Hillslope Materials and Processes*, 2nd ed. Oxford
  University Press.
- Sklar, L.S. & Dietrich, W.E. (2001). Sediment and rock strength controls on
  river incision into bedrock. *Geology* 29(12), 1087–1090.
- Stock, J.D. & Montgomery, D.R. (1999). Geologic constraints on bedrock river
  incision using the stream power law. *Journal of Geophysical Research*
  104(B3), 4983–4993.
- Sunamura, T. (1977). A relationship between wave-induced cliff erosion and
  erosive force of waves. *Journal of Geology* 85, 613–618.
- Sunamura, T. (1992). *Geomorphology of Rocky Coasts.* Wiley.
- Whipple, K.X. & Tucker, G.E. (1999). Dynamics of the stream-power river
  incision model. *Journal of Geophysical Research* 104(B8), 17661–17674.

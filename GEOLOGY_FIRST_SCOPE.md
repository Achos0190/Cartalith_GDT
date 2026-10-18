# GEOLOGY_FIRST_SCOPE.md — lithology, layering and age drive every landform process

**What this is.** The scope for owner **Ruling BH** (2026-09-27,
`LARGE_ITEM_RULINGS.md`, *"geology first; every landform process reads it"*).
It defines milestones **GF-0 to GF-13** and gives the reasoning for each. The
backlog row is `OUTSTANDING_WORK.md`'s *"Geology first: lithology, layering and
age drive every landform process"*.

**Amended 2026-09-27 for owner Ruling BJ**, which answered four of §9's
questions: 11 rock types (Q1); lithology painting **in this build** (Q2, now
§4.13 and GF-8); the coastal pass on in the app at GF-4 (Q7, confirmed); and
**a simple geological clock** (Q4, now §4.12 and GF-7), which reverses this
document's original "no clock". The milestones were renumbered: the old GF-7
(re-baseline and scrub) is now **GF-9**, and it still comes last. §5.4 records
GF-0's measured baseline.

**Amended 2026-09-27 for owner Ruling BM** (*"relief tracks rock by
construction and by uplift-driven erosion"*). GF-2, GF-3 and GF-7 were built
and measured (§5.6–§5.8): no setting of them makes relief track rock (B1, B2
and B4 fail on every world), because under zero uplift the light pass lowers
land by metres, rebound returns most of that, and the carve sets channel
depth. The owner chose **both** remedies, and this amendment scopes them:
- **§4.14, rock-aware construction.** After the column is derived, a
  layer-through lowering of the pre-erosion surface by a rock-blind budget,
  read through the column: a cap holds as a plateau until its thickness is
  spent, weak rock goes down faster, and the breach line becomes a scarp. The
  contact never moves, so the column and the surface stay consistent by
  construction.
- **§4.15, uplift-driven erosion.** An uplift field derived from the existing
  tectonics, and an implicit stream-power stage run toward the steady state
  in which relief is set by uplift ÷ erodibility (Whipple & Tucker 1999). The
  clock scales its model time rather than its iteration count, so cost does
  not grow with τ.
- **§4.16**, how the two combine and in what order.
- **§5.2's B1, B2 and B4 are revised**, and B8 and B12 extended, with the
  evidence for each change in §5.9. B13–B15 are new.
- **Milestones GF-10 to GF-13** (§7). GF-8 (painting) and GF-9 (re-baseline)
  keep their numbers; GF-9 still comes last. The first new milestone, GF-10,
  is a measurement only: a harness prototype showing that the proposed terms
  can move B1, B2 and B4 at all.
- Owner questions 12–19 (§9), each with a default.

**Amended 2026-09-27 for owner Ruling BN**, which answered three of those
questions and reversed two defaults:
- **Q14: construction may move the coastline.** Weak coastal rock may be
  lowered below sea into new bays and inlets, and hard headlands stay, but
  construction still makes no inland pit. §4.14's "land stays land" floor is
  replaced by an ocean-seeded fill (§4.14, *The coast*), and B14 is reworded
  to match (§5.2).
- **Q12: the uplift field includes rift shoulders and rift-basin
  subsidence.** §4.15 gains a signed rift term, and the stage handles
  negative uplift, with the basins it opens filled as sediment (§4.15, *Rift
  shoulders and basin subsidence*).
- **Q18: relief first**, GF-10 → GF-13, then GF-4, GF-5, GF-6 and GF-8, with
  GF-9 last. That was already §7's recommended order.

**Nothing of Ruling BM is built into the pipeline.** GF-10's harness
prototype has been measured (§5.10). No other figure in the BM sections was
measured when they were written: each is cited from §5.4–§5.8 or labelled as
arithmetic.

**What this is not.** It does not track progress. Status lives only in
`cartalith-native/docs/STATUS.md`. A ruling on any owner question in §9 is
recorded in `LARGE_ITEM_RULINGS.md`, not here.

**How it was checked.** Every symbol cited in §1 was opened for this document on
2026-09-27. Symbols are named instead of line numbers, because line numbers in
this tree drift within a day. **No timing or landform figure in §1–§4 was
measured when it was written**; the measured figures are §5.4–§5.8's, each
with its command. Figures derived by arithmetic from a constant in the code are
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
| `resistance_field` | `compute_resistance`: `min(1, 0.6·max(base, 0) + 0.4·age)` | Erodibility proxy. On oceanic plates it is `0.4·age`, so 0–0.4. On continental plates it is 0.33–1.0 (arithmetic from `base` ∈ 0.55–1.0) | `stream_power_kernel` (the only shaping reader); `build_lithology` (`res_hard = 0.55`); landmark *rock formation* (`ROCK_FORMATION_MIN_RESISTANCE = 0.55`; it reads `s` instead since GF-6, §7); Sample; the EF-3 tile re-erosion test (`ef3_tile_erosion.rs`) |
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

So BH **runs on the existing pass structure**. Rock **formation age** is
carried as a relative stratigraphic order (§2.4), not in years.

*Superseded in part by Ruling BM (2026-09-27).* Point 4's "that is enough for
this ruling" was measured false: §5.6–§5.8 found relative relief moving by
metres, not the hundreds of metres B1, B2 and B4 read. Ruling BM makes uplift a
real forcing inside this scope (§4.15), which the geological-time scope's §8
left as its question. That scope's larger design stays declined; §4.15 takes
only the uplift field and the implicit stage it needs.

*Amended for Ruling BJ (2026-09-27).* The owner answered §9 Q4: **add a simple
clock.** The ruling says the 2026-09-02 decline covered the geological-time
scope's larger design, and that the owner now wants a simple one. §4.12
specifies it. It is one dimensionless parameter that scales how many
passes and iterations each process runs, within the stability wall in point 2
above. It adds no uplift, no years and no second anchor, so that scope's §8
questions 1 and 3–5 stay unruled and are not needed here.

### 1.5 Where rock already reaches the product

These consumers already read rock, and will read the new model instead:
- `render.rs` draws per-rock colour (`litho_palette`, `litho_strength`,
  `litho_exposure`) and microtexture (`litho_microtexture`: granite speckle,
  limestone karst pitting, sandstone and shale strata banded by elevation).
- The Sample panel shows `lithology` and `resistance`.
- The landmark generator's *rock formation* kind is a differential-erosion
  landmark. It reads `resistance ≥ 0.55` and a contrast of ≥ 0.20 (the
  same thresholds on `s` since GF-6, §7).
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

### 2.2 Rock types (11; Ruling BJ answered §9 Q1)

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
   basement < cover < volcanic < regolith. It is what "age layers" means here.
   The §4.12 clock does not change it: the clock scales how long the processes
   have run, not when each unit formed.

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
exposed(i) = Unconsolidated          if regolith[i] > R_EXPOSE
           = rock_top[i]              if single-layer, or field[i] ≥ contact[i]
           = rock_sub[i]              otherwise
```

*Amended for Ruling BJ.* The threshold was `regolith > 0`. The clock (§4.12)
adds weathered mantle everywhere, a few metres thick, and at `> 0` that would
hide every bedrock cell under "unconsolidated". `R_EXPOSE` is **5 m**,
converted to normalised units by `(1 − sea) / peak_m`. It is a **judgement**:
it sits above the weathering mantle's largest value at the clock's maximum
(2.7 m, arithmetic in §4.12), so weathering alone never hides the rock, and
below the thickness of the deposits that should (alluvium, talus aprons and
marine sediment, which at map scale are tens of metres and more). GF-2 reports
the share of land where `regolith` lies between 0 and `R_EXPOSE`.

Erosion strips regolith first, then the cap. Once the cap is breached, the
substrate is exposed. Nothing else is needed for scarps to emerge: the weak
substrate lowers faster (κ), and the cap edge keeps a steep face (θc).
Layered-landscape modelling shows the same behaviour: Forte et al. (2016);
Perne et al. (2017).

Contact geometry is horizontal, plus a gentle low-frequency warp so that the
outcrop line is irregular. Dip is §9 Q8.

**Memory**, by arithmetic: 1 + 1 + 4 + 4 = 10 B per cell, plus 1 B per cell for
`volcanic_setting`. At 2048 × 1311 that is 29.5 MB. The painting provenance
mask (§4.13) adds 1 B per cell when anything has been painted, which makes 32.2
MB. It is stored in the save, as §2.6 specifies.

### 2.6 The save format (GF-1 writes it; GF-8 adds the painted mask)

Ruling BJ requires the painted layer to persist and the change to be backward
compatible, so the format is specified here rather than left to GF-1. It
follows `SAVEFILE_COMPAT.md` §8.3's substrate set in every rule. It is a
**separate set with its own manifest member**, not a new version of the
substrate member. §8.3 says a reader MUST refuse a substrate `version` it does
not know, so bumping that version would make every older build drop the flow
and channel grids of a new save. A separate member costs an older build
nothing.

| Path | Element | Meaning | Present when |
|---|---|---|---|
| `rasters/rock_top.u8` | u8 | Uppermost bedrock unit, `0 … 10` (§2.2's index). A reader MUST refuse the set if a value is above 10 | the set is written |
| `rasters/rock_sub.u8` | u8 | The unit beneath the contact, `0 … 10`, or **255 = no second layer** (`NO_LAYER`). Any other value: refuse the set | the set is written |
| `rasters/rock_contact.f32` | f32 | Absolute normalised elevation of the contact. A writer MUST write **NaN** where `rock_sub` is 255, so a reader that forgets the gate reads NaN, never a plausible elevation. A reader MUST ignore it there, and MUST refuse the set if a two-layer cell's contact is NaN | the set is written |
| `rasters/regolith.f32` | f32 | Unconsolidated thickness, in the heightmap's normalised units. `0.0` is a real value: bare rock | the set is written |
| `rasters/volcanic_setting.u8` | u8 | `0` none, `1` arc, `2` rift, `3` hotspot. Anything else: refuse the set | the set is written |
| `rasters/rock_painted.u8` | u8 | `1` where the user painted any of rock, substrate or cap (§4.13); `0` where the column is as generated | `geology.painted` is true |

**The manifest member.** `project.json`'s top-level `geology`:

```json
"geology": {
  "version": 1,
  "age": 1.0,
  "painted": true
}
```

- `version` is this set's own version. A reader MUST refuse any it does not
  know, and then opens the world as having no column.
- `age` is the clock value (§4.12) the column and the terrain were produced
  with. It belongs here for the reason `integrated_drainage` belongs in the
  substrate member: it is a property of these grids, and the parameter may
  since have moved. The parameter itself is also saved with the other
  generation parameters (`params.rs` `PARAMS` and `JS_PATHS` rows).
- `painted` says whether `rock_painted.u8` is present. **Absent is recorded
  absent**: a world nobody painted writes no mask, and a reader MUST NOT
  substitute an all-zero one on re-save.

**Writer obligations.** Write the member and every raster it promises in one
save, or neither. A world with no column (opened from an older save, or an
import before GF-1's single-layer import column) writes neither. It does not
invent a column.

**Reader obligations.** Treat the set as complete only when the member is
present, its `version` is known, and every raster it promises was read at the
right length and in range. Anything less is **no column**:
- every rock reader is dashed with the reason *"saved before geology"*;
- civ falls back to `build_lithology`, as GF-1 specifies;
- the Rock paint target (§4.13) is disabled with the same reason.

**Backward compatibility, both directions.** `format_version` stays 2, on
§8.3's own reasoning: the change is additive.
- **An old save in a new build** has no `geology` member, so it opens with no
  column, as above. Its clock is read as **1.0**. That is not a guessed
  default: τ = 1 reproduces the pre-clock pass counts by control flow (§4.12),
  so 1.0 is exactly the value that world was generated with. It is the loader
  convention `integrate_drainage` already set: an absent key reloads the world
  as it was.
- **A new save in an old build**: the old reader finds the new `rasters/`
  names, carries them as foreign entries (§6.2) and reads the terrain as
  before. Its re-save keeps the rasters but not the member. A new reader then
  sees rasters with no member and treats the world as having no column. That
  is §8.3's safe direction: a world may lose its paint in an old build, but it
  is never read wrong.
- A round-trip test is built from a **real prior-format archive**
  (`git show <sha>:` of a save written before GF-1), not from an empty fixture
  (`MISTAKES.md`, *"Write a backward-compatibility test"*). It asserts that
  the archive opens, reads as having no column, and re-saves with no
  `geology` member and no `rock_*` rasters.

`SAVEFILE_COMPAT.md` gains this table as a new §8.4 in GF-1. The
`rock_painted.u8` row is added in GF-8.

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
7. **The clock scales pass counts; it adds no stage** (§4.12, Ruling BJ). At
   its default of 1.0 every pass count is today's, by control flow. §4 adds
   passes where it says so, and GF-0's B9 measures their cost.
8. **Ruling BM adds two stages, behind their own switch** (§4.16 gives the
   full order). *Construction* (§4.14) runs right after the geology stage and
   before the priming flow, so climate, rain and the first flow already see
   the rock-shaped surface. *The uplift-driven stage* (§4.15) replaces the
   light pass's stream power and rebound, and GF-3's threshold hillslope
   still follows it before the trace.

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
retreat, and BG's sea stacks, need it on in the app. **Ruling BJ confirms the
default of §9 Q7: GF-4 turns it on in `cartalith_godot::params::defaults()`**
(`p.passes.coastal = true`), in the same change that makes cliff retreat read
rock, so hard headlands and soft bays follow the geology. It stays off in
`WorldParams::defaults`, like glacial under Ruling AU. Its measured cost is
disclosed in `GENERATION_PARAMETERS.md`'s *"The six deliberate divergences"*,
which becomes seven.

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
- Built 2026-10-18 with three departures, each read on frames (ridging
  gated to grid slopes from 4 to 12 degrees, terraces band-passed in screen
  pixels, ledges added on top of the fine relief rather than cut through
  it). See §7, "GF-6 measured: rock-styled LOD detail, §4.10".

### 4.11 Landmarks (GF-6)

- *Rock formation*: `ROCK_FORMATION_MIN_RESISTANCE` reads **s** instead of
  `resistance_field`.
- *Cave*: becomes buildable on soluble rock that karst has lowered. Its
  `not_built` reason is removed in the same change.
- *Spring*: reads the spring line.
- *Waterfall*: reads the knickpoints.

`DECISIONS.md` §7q already exempts landmarks from parity.

### 4.12 The geological clock (GF-7; Ruling BJ)

**What the owner asked for.** *"Scope a geological-time parameter that scales
how far each process runs."* Simple, and not the geological-time scope's
larger design, which the owner declined on 2026-09-02.

**The parameter.** One number, **geological age τ**, dimensionless:
- range 0.25 to 4.0, step 0.05, default **1.0**;
- `WorldParams::geo_age`, the same default at both boundaries. It is **not a
  divergence**: at 1.0 every process takes today's expressions by control
  flow (`if τ == 1.0 { today } else { scaled }`), so neither boundary's output
  moves and no golden can;
- `params.rs`: a `PARAMS` row (`geo.age`, group `erosion`, `Kind::Float`, unit
  `×`, no reference control) and a `JS_PATHS` row.

**Why dimensionless, not millions of years.** The pipeline has no calibrated
time. `EROSION_GEOLOGICAL_TIME_SCOPE.md` §8 Q2 (a second self-referential
anchor) is unruled, and the owner declined that design. A slider labelled in
Myr would claim a calibration nothing supports. τ is *relative* age: 2.0 means
"the processes have run twice as long as a default world's". The UI says so.

**How it acts: on pass and iteration counts, never on rate constants.** That
is the geological-time scope's stability wall (§1.4, point 2): explicit
kernels cannot take a larger coefficient, so duration means more passes.
Each process `p` has a response `f_p(τ)` with `f_p(1) = 1`:

```
N_p(τ) = N_p(1)                            if τ == 1.0   (by control flow)
       = max(N_min_p, round(N_p(1) · f_p(τ)))  otherwise
```

where `N_p(1)` is the count the process runs today.

**Two response shapes, chosen per process by whether the kernel already
contains the physics that slows the process down.**

- **Linear, `f(τ) = τ`**, where the kernel's own dynamics already saturate, or
  where the column bounds the process. Imposing a second saturation on top
  would count the slowdown twice.
- **Saturating, `f_sat(τ) = (1 − e^(−kτ)) / (1 − e^(−k))`, with `k = 0.5`**,
  where the real process decelerates for a reason the kernel does not model.
  - `k` is a **judgement**. It gives `f_sat(0.25) = 0.30`, against 0.25
    linear, and `f_sat(4) = 2.20`, against 4 linear (arithmetic).
  - B12 measures whether the kernels' own output still grows with τ under
    it (§5.2).

| Process | What τ scales | Today at τ = 1 | Response | Why that shape | Literature |
|---|---|---|---|---|---|
| **Incision depth** (stream power, light pass and `evolve_cycles`/`sediment_fill`) | the iteration count: `max(4, round(iters·0.6·τ))` for the light pass, `iters·τ` for the sediment fill | 9 iterations (arithmetic from `iters: 15`) | **linear** | Detachment-limited stream power with no uplift lowers channels toward base level. As they lower, their slope falls, so the lowering rate falls with it: the saturation is *in the kernel*. The geological-time scope measured it as equilibration past about 360 iterations (§1.4, point 3). A second saturation would double-count it | Whipple & Tucker (1999) on response time; Baldwin, Whipple & Tucker (2003) on post-orogenic decay timescales |
| **Threshold hillslope** (§4.3) | `N_h` passes | GF-3's measured `N_h` | **linear** | It moves only the excess over the critical slope. Once slopes fall below `θc` it moves nothing, so it saturates by construction | Roering, Kirchner & Dietrich (1999); Selby (1993) |
| **Karst dissolution fraction** (§4.6) | the dissolution budget: `Δz = −min(k_k·σ·rain·τ, soluble thickness above the next insoluble contact)` | `k_k·σ·rain` | **linear, bounded by the column** | Carbonate denudation is proportional to runoff, so it is constant under a fixed climate. The limit is how much soluble rock there is: once a limestone cap is consumed, the substrate is exposed and `σ = 0`. The column supplies that limit, so time needs none | Ford & Williams (2007) |
| **Cliff retreat** (§4.5) | coastal passes | 4 (`coastal_passes`) | **saturating** | A retreating cliff leaves a widening shore platform, and waves spend their energy crossing it before they reach the cliff. Retreat therefore slows toward an equilibrium platform width. `coastal_process` has no platform term | Sunamura (1992); Trenhaile (2000) |
| **Glacial depth** (§4.4) | glacial passes | 8 (`glacial_passes`) | **saturating** | A trough's cross-section tends toward a steady U-form, and long-profile overdeepening decelerates as the trough deepens. `glacial_kernel` runs a fixed per-pass abrasion law with neither effect | Harbor (1992); MacGregor et al. (2000) |
| **Regolith production** (new with the clock) | the weathered mantle added to `regolith` on land where bedrock is exposed | `h(1) = h₁` | **logarithmic**: `h(τ) = h₀ · ln(1 + τ·(e^(h₁/h₀) − 1))` | Soil production decays exponentially with soil depth, `P = P₀·e^(−h/h₀)`. With no erosion, integrating it gives `h(t) = h₀·ln(1 + P₀t/h₀)`: logarithmic growth. Anchoring the curve at `h(1) = h₁` absorbs the unknown `P₀·t` into a judgement | Heimsath, Dietrich, Nishiizumi & Finkel (1997) |

- **Regolith constants.** `h₀ = 0.5 m` is Heimsath et al.'s e-folding depth
  for soil production. It is a literature value and is **verified at the
  source before GF-7 pins it**, as §2.3 requires of every table value. `h₁ =
  2 m` is a **judgement**. With both, `h(0.25) = 1.33 m` and `h(4) = 2.69 m`
  (arithmetic). That is why §2.5's `R_EXPOSE` is 5 m.
- **Floors.** `N_min` is 4 for stream power (today's own `max(4, …)` floor) and
  1 for every pass. At the default `iters: 15` the light pass runs
  `max(4, round(9·τ))` iterations, so any τ below 0.5 runs 4 (arithmetic:
  `round(9·τ)` is at most 4 there, and the floor lifts anything lower). The
  rounding is `js_round`, as today's expression uses. The UI says when the
  floor binds; the clamp is not hidden.
- **Rebound** is not scaled separately. It follows each erosion call, as it
  does today, so it follows the scaled counts.
- **The clock does not touch** the priming climate, the hydrology trace, the
  river carve (RV-1's guarantees are per-geometry, not per-time), volcanism or
  craters. Crater degradation already has its own `crater_degradation_tau`,
  and the clock does not feed it. Feeding it is left as an open question
  (§9 Q11), not done silently.
- **Cost.** Linear in the stream-power iterations. At τ = 4 the light pass
  runs 36 iterations instead of 9 (arithmetic). B9's bar is set at τ = 1. GF-7
  measures and discloses the cost at 0.5, 2 and 4, as a median with min..max,
  and gates nothing on it: choosing an old world is choosing its cost.
- **Interaction with the existing counts.** τ multiplies them; it does not
  replace them. The Erosion stage's own iteration and pass controls keep
  their meaning ("iterations at τ = 1"), and the effective counts are shown
  beside them.

**The UI**, derived from the DCC vocabulary:
- Stage **06 Erosion**'s key controls (`DCC_SHELL_SPEC.md` §5.1) gain one row
  at the top of the stage, above the per-process groups, because it scales all
  of them. It is written the way that table writes its other multipliers
  ("drift ×1.00"):
  **`geological age ×1.00`** — a slider, 0.25 to 4.00, step 0.05.
- Its readout line underneath shows the effective counts, e.g. *"stream power
  9 it · glacial 8 passes · coastal 4 passes · mantle 2.0 m"*. It updates as
  the slider moves, and a floored count is marked as floored.
- Its tooltip: *"How long the landscape has been eroding, relative to a
  default world. Higher is older: deeper valleys, wider bays, deeper glacial
  troughs, thicker weathered mantle. Not in years: the generator has no
  calibrated time."*
- It is a generation parameter, so moving it marks the world stale like any
  other stage-06 control (header correction #2 of `DCC_SHELL_SPEC.md`: the
  product regenerates; it does not re-run a single stage).
- `DCC_CONTROL_INDEX.md` gains its row in GF-7. The phone and tablet sheets
  take it wherever they list stage-06 controls (`ANDROID_UI_SPEC.md`,
  `TABLET_UI_SPEC.md`).

### 4.13 Lithology painting (GF-8; Ruling BJ)

**What the owner asked for.** *"Hand-painting rock types (like biome painting)
is part of BH, not a later GF-8. Erosion responds to the painted rock."* (The
ruling's "GF-8" is the *old* deferred number; this scope's GF-8 is this
milestone, in this build.)

**What exists to build on, opened 2026-09-27:**
- `cartalith_spatial::paint`: `PaintStamp` (a categorical disc over a `u8`
  grid, with a caller's exclusion mask and the `with_falloff` edge),
  `PaintLayer` (`0` = unpainted, else a 1-based palette index), and
  `encode_sparse`/`decode_sparse`.
- `cartalith_godot::paint_bridge::PaintEditor`: one
  `PassBuffer<PaintStamp>` draft per layer, `commit_all`/`discard_all`, one
  shared `DirtyTracker`.
- `WorldGen::paint_commit`: it records a **non-reversible** ledger row
  (`EntryKind::Recorded`), then `mark_and_recompute(PipelineStage::Civ, …)`.
- `DCC_SHELL_SPEC.md` §4.5.2's Biome-paint tool, whose target selector lists
  exactly three targets (biome, terrain, splat). It says soil and lithology
  have "no override array behind them, so offering them would be inventing a
  feature". Ruling BJ makes it one. That sentence is updated in GF-8.

**What does not exist, checked at the symbols.** No edit re-runs erosion
today:
- `sculpt_commit` pushes a height undo, then `mark_and_recompute(Height)`,
  which re-runs hydrology and climate but **not** erosion
  (`DCC_SHELL_SPEC.md` header correction #1).
- Erosion after generation is the manual **Erode** op
  (`erode_bridge::run_erode_with_recompute`). It runs `recompute_stale`, then
  the op, then marks `Height` changed over the whole map, then
  `recompute_stale` again.
- `cartalith_erosion::tile::tile_erode` re-erodes a window with a pinned
  boundary ring and seeded upstream area. It **has no production caller**.

So "the same way it does after other edits" is read as **the Erode op's
pattern**, because that is the one edit path that runs erosion. GF-8 is
`tile_erode`'s first production caller.

**The tool.**
- Biome paint's target selector gains a fourth target, **Rock**.
  - The tool-options row, in §4.5.2's grammar: `PAINT · ROCK` · layer
    (**Surface** / **Beneath** / **Cap thickness**) · value swatch from the
    11-rock legend (Beneath adds **none — single layer**) · thickness *m*
    (Cap thickness only) · radius · hardness · softness · land only ·
    ✓ Commit.
  - The right dock shows the painted-cell count, the rock legend with painted
    counts per class, the re-erosion window's size, and Commit / Discard.
- **Three new `PaintLayer`s** in `PaintEditor`, drafted and committed with
  the other three by the same `commit_all`/`discard_all`, so a layer switch
  never drops a draft:
  - `rock_top` (value = rock index + 1);
  - `rock_sub` (value = rock index + 1, or **255 = none, single layer**);
  - `rock_cap`: the thickness in **10 m classes**, value `k` ∈ 1 … 255 meaning
    `10·k` m, so 10 m to 2 550 m. Reusing the `u8` categorical layer keeps one
    brush, one sparse encoding and one falloff. The 10 m step and the 2 550 m
    ceiling are **judgements**: a flood-basalt pile can reach about 2 km,
    which fits, and 10 m is well under one cell's relief at the default
    extent. Painting a cap thickness is a categorical write of a class; no
    two thicknesses are ever blended, which is `paint.rs`'s own
    categorical-blending rule.
- **Erase (⇧) is disabled for the Rock target**, with the reason on the
  control: *"a committed rock is reverted with Undo; the column has no
  unpainted state to fall back to"*. Discard still drops an uncommitted
  draft. This is because the commit **bakes** the paint into the column, as a
  sculpt commit bakes into the heightmap, rather than keeping an override over
  a stored generated column. Baking keeps one column, one save set and one
  undo mechanism.

**The commit** (`WorldGen::paint_commit`, the Rock branch). It is refused while
the world is finalized, like every height edit (`bake.check(HeightEdit)`), and
refused with the §2.6 reason when the world has no column.
1. **Undo first.** Push one undo step holding the field **and** the column:
   `rock_top`, `rock_sub`, `contact`, `regolith` and `rock_painted`. Record one
   ledger row, `subsystem: "geology"`, label *"Rock paint commit"*,
   `EntryKind::HeightSnapshot`.
   - `undo::HeightUndo` gains an optional column snapshot on a step, charged
     to the same byte budget (`DEFAULT_BUDGET_BYTES`, 256 MiB). A step is
     4 + 11 = 15 B per cell, 40.3 MB at 2048 × 1311 (arithmetic).
   - `undo_one` and `redo_one` restore both, or neither.
   - The redo tail carries the column the same way.
   - A step without a column (every existing kind) is unchanged, so the
     existing undo tests are untouched.
2. **Bake.**
   - Painted `rock_top` and `rock_sub` cells overwrite the column's.
   - A painted cap thickness `t` sets `contact = bedrock_surface − t·(1 −
     sea)/peak_m`, where `bedrock_surface = field − regolith` at commit time.
     So the contact is **absolute**, as §2.5 requires, and later lowering
     breaches it.
   - A thickness dab on a cell whose effective `rock_sub` is none is
     **skipped and counted** (`stamps_skipped`, which `CommitSummary` already
     reports). A cap needs something beneath it, and inventing a substrate
     would be a silent choice.
   - Painting `rock_sub = none` makes the cell single-layer, and its contact
     becomes NaN, per §2.6.
   - Every baked cell sets `rock_painted = 1`.
3. **Re-erode, differentially, over a padded window.** Let `W` be the
   bounding box of the committed dabs' dirty tiles, padded by `P` cells on
   each side, and clipped to the map.
   - Run the rock-reading landform chain on `W` **twice**: once with the
     pre-commit column (`E_before`) and once with the baked column
     (`E_after`).
   - Add `E_after − E_before` to `field` inside `W`.
   - The chain is stream power through `tile_erode` with the κ slice; then
     the threshold hillslope; then karst. Glacial and coastal join it when
     their passes are on and `W` holds ice or coast. Every count is the
     generation's, scaled by the world's own τ (§4.12), so the painted rock
     has eroded for the same geological age as its surroundings.
   - **Why differential.** Re-running erosion on an already-eroded surface
     would erode everything in `W` a second time. The difference cancels that
     to first order, keeps every earlier sculpt and Erode edit, and leaves
     exactly the response to the rock change.
   - A rock painted to be the same as the rock already there gives
     `E_after = E_before` bit for bit, so the field moves by zero bytes. That
     is identity by control flow, and B11 asserts it.
   - `P` is a **judgement**, set in GF-8 from `tile_erode`'s own measured ring
     lip (`ef3_tile_erosion.rs`, which holds it under 2 % of relief). It is
     reported in the right dock.
4. **Refresh.** Mark `Height` changed over `W`'s tiles, and `Civ` over the
   painted tiles (lithology feeds soil and resources). Then
   `mark_and_recompute`, as the Erode op does. Every consumer of rock reads
   the new column on its next read: the Sample panel, render, civ and
   landmarks.

**Persistence.** The baked column is the column, so it persists in §2.6's
rasters with no new mechanism. `rock_painted.u8` and `geology.painted` record
provenance, so the Sample panel can say *"Granite (painted)"* rather than pass
paint off as generated. The three draft layers persist in `drafts/paint.json`
beside the existing three, as more sparse `[index, value, …]` lists under new
keys (`rock_top`, `rock_sub`, `rock_cap`):
- An older reader ignores keys it does not know. GF-8 checks that it does,
  against `project_bridge.rs`'s reader, before relying on it.
- A document whose `gw`/`gh` are not the world's is refused, as
  `SAVEFILE_COMPAT.md` already requires of `paint.json`.

**Regeneration** replaces the column and the paint together, as it replaces a
sculpted heightmap. The shell already asks before a regenerate that would
discard hand-authored work (`_authored_inventory()`, which lists sculpt
stamps and paint among others). GF-8 adds painted rock to that inventory.

*Amended for Ruling BM.* When the relief switch (§4.16) is on, the painted
window's chain gains the §4.14 construction delta and runs the §4.15 stage
in place of the light stream-power call. The construction delta needs the
lowering budget, which is recomputable from stored fields (`stress_field`,
`boundary_mask`, `crust_field`; flexure is `compute_flexure` of the first
two), and the structural surface, which is **not** stored today. GF-11
decides whether to store the structural surface or the construction depth
(4 B per cell either way, arithmetic), and discloses the memory. GF-8 is built
after GF-12, so it inherits whichever chain exists.

*Decided at GF-11 (2026-10-12).* The **structural surface** is stored, as
`GeologyColumn::construction = Construction::Built { structural_surface }`,
only on a world construction lowered; every other world carries `Off` (or
`Skipped` with a reason) and pays nothing. The surface rather than the depth,
because the depth stops being recoverable once later passes move the field,
while the surface is what §4.14 reads. Measured by `_memlane_peak` at
2048 × 1311, seed 483920, processes on, relief off against on: 10.24 MiB
resident (the arithmetic 4 B per cell), whole-run peak 805.76 → 816.06 MiB.

### 4.14 Rock-aware construction (GF-11; Ruling BM, part 1)

**How the height is built today, read at its symbols.** In
`generate_terrain_inner`:
- `compute_warp` displaces the noise coordinates by up to
  `0.18·tect.warp·gw` cells (arithmetic at the default `warp = 0.45`: 0.081 of
  the map width).
- `compute_height` writes, per cell,
  `0.5 + α·(0.40·base + 0.50·t) + fwt·flex + hwt·hetero + β·(n − 0.5)·(0.25 + 0.75·rug)`,
  where `base` is the blurred plate base, `t` is `stress_field` (or, with
  world-structure on, `oro + min(σ, 0)`), `flex` is `compute_flexure`,
  `hetero` is `compute_heterogeneity`, `n` is ridged or fBm noise at
  frequency `5·terrain_detail_k`, and `rug = exp(−age·(1 + 6·age_inf))`.
  The defaults are α 0.85, β 0.22, `flexure` 0.20, `hetero` 0.08 and
  `age_inf` 0.6 (`WorldParams::defaults`).
- Orogeny (`build_orogeny_field`, `smooth_orogeny`) runs only with
  world-structure on, which is off at both boundaries. **The app has no
  orogeny.**
- `normalize_field` stretches the result min-max to 0–1. Volcanism and
  craters are stamped on it, it is clamped to 0–1, and sea level is fixed.
  Only then does `build_geology` derive the column from this surface.

Two properties of that formula matter here:
- **Nothing in it reads rock.** Its only lithology-like input is `age`,
  which is margin distance, and it enters as roughness damping. At the
  default `age_inf` the noise term's multiplier falls from 1.0 at a margin to
  `0.25 + 0.75·e^(−4.6)` = 0.258 at the most distant cell (arithmetic). That
  is where §5.4's negative B1 comes from: `compute_resistance` and §2.4's
  granite both mark the old interior as strong, and the formula makes the old
  interior smooth.
- **The column is derived from its output.** The cover's structural lows and
  its contact (`surface − CAP_FRACTION·cover`), the rift cap
  (`surface − RIFT_CAP_M`) and the volcanic caps (`surface − vf·h`) are all
  measured from this surface (`build_geology`). So any rock term applied
  *before* `build_geology` would change the surface the column is read from:
  the circularity the ruling warns of.

**The options, and the choice.**
1. *A multiplicative relief term inside `compute_height`*, scaling the noise
   amplitude by strength. **Rejected.** It would write "relief ∝ strength"
   into the surface directly, so B1 would pass by fiat, the same flaw §5.1
   found in using `build_lithology` as a baseline. Noise amplitude is not the
   relief a rock holds up. It would also need the column before the surface
   it is derived from, so only the basement (which `build_geology` could
   derive from plates alone) would be available, without caps.
2. *An additive term by strength*, `z −= D·f(s)`. **Accepted in a
   specific form**: as a lowering, not a raising; applied *after* the column;
   and read through the layers, which is option 3.
3. **A caprock-plateau construction through the column.** **Chosen.** It is a
   stripping model: a rock-blind budget of denudation is spent through the
   column at each unit's own erodibility. A cap is lowered slowly until it is
   gone; then the substrate beneath goes fast.

**The construction.** For each land cell `i` (structural surface
`z₀(i) > sea`), with a lowering budget `B(i)` in metres (below), `k_t =
κ(top)^c`, `k_s = κ(sub)^c` (§4.1's contrast `c`), and cap thickness
`h(i) = z₀(i) − contact(i)` in metres:

```
single-layer:                         D = B·k_t
two-layer, cap survives  (B·k_t ≤ h): D = B·k_t
two-layer, cap breached  (B·k_t > h): D = B·k_s        (the cap is gone)
z(i) = z₀(i) − m_to_norm(D)
```

- **At `c = 0` every `k` is 1 and `D = B` everywhere**, whatever the rock.
  That is the rock-blind control arm of §5.1, exactly: the same budget, the
  same surface, no rock. Every relative bar compares against it.
- **Where the cap survives, the surface is lowered uniformly and stays a
  plateau.** Where it is breached, the whole budget is spent at the
  substrate's rate. The breach line is where `h` crosses `B·k_t`, and the
  surface steps down across it by `B·(k_s − k_t)` (arithmetic): that is the
  scarp, and its height is set by the budget and the contrast, not chosen.
  With sandstone over shale at `c = 0.5`, `k_s − k_t = 1.581 − 0.894 = 0.687`
  (arithmetic from §2.3's κ), so a 100 m budget gives a 69 m step: 10° across
  one 390.6 m cell at 800 km.
- **Why "breached means all of it at the substrate's rate", a judgement.**
  The obvious vertical rule, `D = h + (B − h/k_t)·k_s` once the cap is
  spent, is **continuous at the breach line** (both branches give `D = h`
  where `h = B·k_t`, arithmetic), so it makes a slope break and no scarp. A
  real cap edge retreats laterally, undermined as the weak rock beneath it is
  removed, much faster than it is lowered from above; the cells behind the
  retreating face have spent most of their history as substrate. Treating
  that retreat as instantaneous against the budget is the modelling
  simplification. GF-10 measures both rules, so the choice is evidence, not
  assertion.
- **The contact is never written.** Construction is erosion in the past, and
  erosion does not move a contact (§4.2). So `build_geology`'s column is read
  exactly as derived, the exposure rule (§2.5) reads the new surface
  correctly, and a breached cap reads as its substrate with no further
  bookkeeping. **This is how construction and column stay consistent**: the
  column is derived once, from the structural surface; construction reads the
  column and never re-derives it; nothing downstream re-derives it either.
  The order structure → column → construction is also the geological order:
  deposit, then denude.
- **Arc edifices are exempt** (§9 Q17). Where the exposed rock is andesite
  or tuff, `D = 0`. Their relief is the stamper's, and young. Plateau basalt
  is not exempt: flood-basalt provinces are dissected (the Trotternish case
  is one).
- **The coast may move** (§9 Q14; **Ruling BN** reversed the default "land
  stays land"). There is no floor at sea level. A land cell may end below
  sea, but only where the fill below leaves it there, which is exactly where
  it is joined through below-sea cells to the ocean: a new bay or inlet. An
  inland cell lowered below sea is a closed depression like any other and is
  filled back to its spill point. So construction moves the coast and still
  makes no inland pit. A hard headland stands because strong rock is
  lowered least, with no rule of its own. The share of land that goes under
  is reported, beside the share the old 1 m floor would have caught.
  - **How it changes this section.** Under the old default the fill could
    use the routing rule, which treats every below-sea cell as an outlet.
    That rule cannot be used here: an inland cell pushed below sea would read
    as an outlet rather than a pit. So the fill is seeded only at the
    structural surface's **ocean** cells (`build_water_bodies` class 1) and
    the map's real outlets (its non-wrapping edges). It joins cells
    **4-connected**, the way `build_water_bodies` joins below-sea water, so
    a bay the fill leaves open is one the classifier calls ocean. An
    8-connected fill would pass a diagonal-only mouth that the classifier
    calls a lake.
  - **Consequences.** The land share falls where weak rock meets the sea.
    Every coastal consumer (the coastal pass at GF-4, settlement siting, the
    classifier) sees a new coast, and B8's ocean-on-paths guarantee now runs
    on a moving sea (it is re-checked on every arm, §5.10).
- **No new closed depression.** Differential lowering of a weak patch inside
  strong rock would make a pit, and pits are what B8 counts. So both
  surfaces are priority-flood filled from the ocean (the coast bullet above),
  giving a fill depth `f(z) − z` per cell. Wherever the lowered surface's fill depth
  exceeds the structural surface's, the cell is raised by the excess, and
  **the raise is added to `regolith`**: a basin that differential erosion
  opens is a basin that fills with sediment (alluvium, lake beds).
  Depressions that already existed on `z₀`, today's lakes among them, keep
  their depth. So construction creates no new depression by construction
  (B14). One pass is not guaranteed to be enough, because raising one pit can
  change another's spill point; GF-11 repeats it until no excess remains and
  reports the pass count.
- **Scarp sharpness comes from jointing, through θc, and nothing new.**
  §2.3's θc column already carries jointing (massive granite and columnar
  basalt 60°, bedded sandstone 50°, foliated schist 40°, fissile shale 30°).
  GF-3's threshold hillslope, which runs after both BM stages (§4.16), sheds
  anything steeper than the exposed rock's θc. A cap face stands at θc(cap)
  and the slope below it at θc(sub): the free face over a debris slope of a
  cliff-and-slope scarp. No second jointing constant is added; bedded
  benches below cell scale are GF-6's LOD terrace (§4.10).

**The budget `B(i)`.** Denudation follows uplift, so the budget is the same
uplift pattern §4.15 derives, spatially smoothed:

```
B(i) = τ · D₁ · clamp(Ū(i) / mean_land(Ū), w_min, w_max)
```

- `Ū` is §4.15's uplift field, blurred at `3·tect.blur_r` (the flexure
  blur's scale), so the budget varies over the plate-scale and the rock map
  supplies all the short-wavelength contrast.
- `w_min = 0.25`, `w_max = 4`: **judgements**, so that no land is exempt from
  denudation (cratons strip their covers too) and no belt gets an unbounded
  budget.
- **`D₁` is pre-registered, not tuned against the bars:** the median land
  cap thickness `h` over two-layer cells on the world's own column. At
  `c = 0` that breaches about half the caps where `Ū` is average, which
  maximises the breach lines B4 needs, and it scales itself with GF-1's cover
  constants. It is a **judgement** of the rule, not of a number. GF-10 prints
  the cap-thickness distribution; the arithmetic floor is 10 m
  (`CAP_FRACTION · COVER_MIN_M`) for cover caps and 150 m (`RIFT_CAP_M`) for
  rift caps.
- **τ scales it linearly**, bounded by base level (the sea floor above, and
  the fill): §4.12's rule for a process the kernel does not saturate but a
  limit does.

**Scale awareness (80, 800, 8 000 km).**
- `B` and `D` are **metres**, converted by `m_to_norm`, so the lowering is
  the same physical depth at every extent. A step of `D` metres across one
  cell is a slope of `atan(D / cell_m)`: at 390.6 m cells (800 km over 2 048,
  arithmetic) a 100 m step is 14°; at 39 m cells (80 km) it is 69°, which
  GF-3's threshold then spreads over several cells at θc; at 3 906 m cells
  (8 000 km) it is 1.5°. That is the right behaviour: the same scarp is a
  cliff on a local map and a gentle break on a continental one.
- **The rock map itself is extent-blind.** §2.4's belt widths are multiples
  of `tect.blur_r` in cells, so a collision belt is the same fraction of the
  map at every extent (at the default 18 cells, `W_OROGEN` is 54 cells: 2.1 km
  at 80 km, 211 km at 8 000 km, arithmetic). Construction does not change
  that; it is GF-1's property, disclosed here because it sets the wavelength
  of every rock-controlled step.
- B4 is therefore expected to pass at 800 km, is reported at 80 km, and is
  not expected at 8 000 km, where a cap edge is sub-grid (§5.2).

### 4.15 Uplift-driven erosion (GF-12; Ruling BM, part 2)

**Why zero-uplift erosion cannot do it, from the kernel.** The rock kernel's
update, per cell per iteration, is
`z' = (z + dt·u + c·z_r) / (1 + c)`, with `c = dt·C`,
`C = K·g·κ^c·(1 + 2·ck·rain)·A^m / L`, `m = 0.5` and `dt = 1.0`
(`stream_power_core`). Two consequences follow by arithmetic:
- **Its fixed point is `z* = z_r + u/C`**, whatever `dt` is: the
  steady-state gradient of the detachment-limited stream-power law with
  `n = 1`, `S = U/(K·A^m)` (Whipple & Tucker 1999). With `u = 0` the fixed
  point is `z_r`: every channel grades to base level, and **rock drops out of
  the steady state entirely**. Rock can only show in the transient, as
  §5.6–§5.8 measured, by metres.
- **Its deviation from the fixed point shrinks by `1/(1 + c)` per
  iteration** (with the receiver held). On a unit-area hillslope cell at the
  defaults, `C = 0.012·(1 + rain)` (arithmetic from `k = 0.012`,
  `climate_k = 0.5`, `g = 1`, `L = 1`), so nine iterations close 10 % of the
  gap at `rain = 0` and 19 % at `rain = 1` (`1 − 1.012^−9`, `1 − 1.024^−9`). That is why the light pass is transient everywhere off the
  channels.

With uplift, the fixed point is `S = U/(K·A^m)`: **at matched uplift and
drainage area, weak rock holds a gentler slope, and strong rock a steeper
one, in the ratio of their erodibilities**, `S_strong/S_weak =
(κ_weak/κ_strong)^c`. Relief scales with `(U/K)^(1/n)` (Whipple & Tucker
1999; the channel-steepness form is Kirby & Whipple 2012). That is the
relation B1 and B2 test, and it needs `U > 0`.

**The uplift field.** Derived from the tectonics that already exist, all of
it available before the geology stage:

```
U(i) = U₀ · ( max(σ_i, 0) + a_φ·max(φ_i, 0) + a_o·max(oro_i, 0) + a_r·R_i ) + U_bg·[crust_i ≥ 0]
```

(`R` is Ruling BN's signed rift term, defined below. Before the ruling the
bracket had no `R` and `U` was never negative.)

- `σ` is `stress_field` (convergence positive; `compute_stress` normalises it
  by its absolute maximum, so it is relative per map). The legacy kernel's own
  `u` is `max(σ, 0)/max · stream.uplift`, so this generalises the term the
  kernel already has.
- `φ` is `compute_flexure`: stress seeded at boundary cells and blurred at
  `3·blur_r`. Its positive part is the broad flexural uplift behind a
  convergent margin. `a_φ = 0.5` is a **judgement**.
- `oro` is `smooth_orogeny`'s field, present only with world-structure on,
  `a_o = 1` a **judgement**. Zero in the app.
- `U_bg` is a background epeirogenic rate on continental crust, so a craton's
  rivers keep a gradient rather than grading to sea level (the `u = 0`
  failure above). **Judgement**: `U_bg = 0.1·U₀`, measured in GF-10.
- **Rift shoulders and basin subsidence** (§9 Q12; **Ruling BN** reversed
  the default "no subsidence and no rift-shoulder uplift"). `R` is built from
  the divergence `max(−σ, 0)` on boundary cells typed `RIFT`. That source is
  blurred twice, at `3·blur_r` (the flexure's own wavelength, as
  `compute_flexure` uses it) and at `blur_r` (the stress wavelength). `R` is
  the wide blur minus the narrow one, divided by its largest magnitude.
  - **Why this shape.** Both blurs preserve mass, so their difference
    integrates to about zero. It is **negative on the axis**, where the narrow
    blur peaks higher: the subsiding basin, or graben. It is **positive on the
    flanks**, where the wide blur reaches further: the uplifted shoulders.
    That is the shape of a flexural rift profile. The two radii are this
    scope's own two blur scales, not new constants.
  - `a_r = 0.5`, a **judgement**, set equal to `a_φ`. Both terms are the same
    lithosphere's flexural response to a boundary load, so neither is given
    more weight than the other.
  - **How the stage takes negative `U`.** A subsiding cell is lowered by
    `U·dt` each step, with its contact, so the whole column goes down.
    Subsidence is burial, and the accommodation it makes fills. After the
    stage, any closed depression inside the subsiding footprint that is
    deeper than it was on the stage's input surface is filled to its spill
    point, and the fill is added to `regolith`. This is the same rule as
    construction's fill (§4.14) and uses the same ocean-seeded fill. Ocean
    and lake cells stay pinned. So subsidence can deepen an existing sea or
    lake bed only through its pinned level, which it cannot move, and it
    makes no new inland pit. Where a subsiding basin meets the sea it may
    flood, as §4.14's coast allows.
  - **Not modelled:** forearc and foreland basins. No existing field marks
    them. `φ` is positive near convergence throughout, and nothing in the
    tectonics distinguishes a flexural moat.
  - **Before Ruling BN** the default was no subsidence, on the ground that
    it lowers land under the sea and is deposition's business. The ruling
    keeps that reasoning in the basin fill above, and allows the sea where it
    connects.
- **Units: metres per unit model time**, converted with `m_to_norm`. The
  kernel's `A^0.5/L` is scale-free (§1.4 point 1), so a steady-state step of
  `U/C` metres per cell gives a physical slope `∝ 1/(A_cells^0.5 · cell_m)`,
  which is `(U/K)·A^(−0.5)` with `A` in m²: the law is dimensionally
  consistent at every extent once `U` is in metres (arithmetic).
- **`U₀` is calibrated by mean balance, not by the bars.** On the
  rock-blind arm (`c = 0`) at τ = 1, `U₀` is set so that the land-mean surface
  change over the stage is zero: at steady state denudation equals uplift, and
  this keeps the world's hypsometry and land share where the structural
  surface put them. GF-10 measures it per seed; GF-12 fixes the rule (a
  per-world solve or one constant) from that measurement.
- It is not `stream.uplift`. That slider (0–0.4, `sUp`) stays the legacy
  kernel's, in normalised units per iteration (§9 Q13).

**The stage.** It replaces the light pass's `rock.stream_power` and
`rock.rebound` when the relief switch is on (§4.16):

```
for k in 0..N_BM:
    route: fill, receivers, drainage area over the current surface
    one implicit update: dt_k = T(τ) / N_BM, uplift U·dt_k, rock κ(exposed)^c,
                         contact += U·dt_k (the existing rule), lakes and ocean pinned
                         (U < 0 where a rift basin subsides: Ruling BN)
account regolith once, on the stage's net change less the uplift U·T (§4.9)
fill new closed depressions in the subsiding footprint to spill, as regolith (Ruling BN)
```

(Accounting "less the uplift" is a correction, not a Ruling BN change. The
line read "on the stage's net change". Under uplift, a cell that rises
without being eroded would then be counted as deposit.)

- **Implicit, so `dt` may be large.** Braun & Willett (2013)'s scheme is
  unconditionally stable for `n = 1`, and the kernel is that scheme. §4.12's
  rule "on counts, never on rate constants" rests on the stability wall of
  the *explicit* kernels (§1.4 point 2), which does not bind here. So the
  clock scales model time `T`, not `N_BM`, and **the cost is the same at
  every τ**. This amends §4.12 for this stage only (§9 Q16).
- **The routing is refreshed every step.** The legacy kernel freezes the
  fill, receivers and area across its iterations (`stream_power_core`), which
  is harmless at `dt = 1`, where a surface moves by metres. With `dt ≫ 1` the
  surface moves by far more than the fill's `1e-5` epsilon, and a frozen tree
  can leave a cell draining "uphill" into a pit it has itself cut. **This is
  the leading hypothesis for §5.8's small-lake growth at τ > 1, which was not
  diagnosed**; GF-10 tests it before GF-12 relies on it. Braun & Willett
  recompute the stack at every step.
- **`N_BM`**: a **judgement**, 8, set in GF-10 from B8 against B9.
- **Lakes and the ocean are pinned** (`stream_power_kernel_bounded`'s
  `pinned` mask, which the rock entry point does not take today). A pinned
  cell is one whose depression is deeper than the lake classifier's 0.004
  (`build_water_bodies`' `lake_depth`) on the stage's input surface. The
  classifier also requires rain ≥ 0.22 to call it a lake; the pin ignores
  rain, so it holds a superset (dry pans too), deliberately. Pinning holds
  existing lakes at their level; without it, the implicit update raises a pit
  cell toward `z_r + U/C` above its receiver and fills it (the kernel already does this partly at `dt = 1`; §5.6 records the
  effect).
- **Deposition is off in the stage** (`deposit = 0`). The deposition
  sub-loop's capacity rule, `0.005·A^0.5·slope`, is per iteration and not
  scaled by `dt`, so a large step would misstate it. The steady-state analysis
  above is detachment-limited in any case. `sediment_fill` stays the
  transport pass.
- **The clamp.** The kernel clamps to 0–1 at the end. With uplift a summit
  can exceed 1 and would be silently flattened, which is "no value" (a
  clamped height) encoded as a plausible one. The stage reports the count of
  cells it clamps at 1; B15 requires 0.

**Model time `T(τ)`, pre-registered.** `T(τ) = τ·T₁`, and `T₁` is set by a
rule that does not read B1, B2 or B4: three e-folding times of a channel-head
cell, `T₁ = 3 / C_head`, where `C_head` is `C` at `κ = 1`, `rain = 0`,
`L = 1` and `A` = the channel-initiation area `build_channels_routed` uses at
the world's river density. At τ = 1 the channel network is then within about
5 % of steady state (`e^(−3)`, arithmetic) and interfluves, whose `A` is
smaller, are still transient: a mature drainage on young divides. τ = 4
brings most of the landscape near steady state; τ = 0.25 leaves it young.

**How rebound interacts: it is not applied after this stage** (§9 Q15). `U`
is **rock uplift net of isostasy**, the quantity England & Molnar (1990)
distinguish from surface uplift and exhumation. Adding `isostatic_rebound`
on top would count isostasy twice. Two further facts, one measured and one
arithmetic, say rebound is the wrong tool here:
- it returned 78–93 % of the light pass's mean lowering (§5.8);
- its blur radius is `max(tect.blur_r, 8)` **cells**, so at the default 18
  cells it spreads the removed mass back over a radius of 7.0 km at 800 km
  and 0.70 km at 80 km (arithmetic). Flexural isostasy acts over a regional
  wavelength that does not shrink with the map, so this hands much of a
  valley's own incision back to that valley. (How much of §5.8's 78–93 % is
  due to the radius was not measured.)
Rebound stays exactly as it is after the glacial pass and in `evolve_cycles`,
where it is today's behaviour and no BM bar reads it.

**Hillslopes under uplift.** At high `U` the steady-state slope `U/(K·A^m)`
on small-area cells can exceed the rock's θc. GF-3's threshold hillslope,
which follows the stage, then caps it at θc(exposed rock). That is the
threshold-hillslope regime in which relief saturates with uplift and is set
by rock-mass strength (Schmidt & Montgomery 1995; Montgomery & Brandon 2002),
so in the most active belts relief tracks θc directly. `hillslope_diffuse`
stays off, as in the app today.

**How B8 is protected**, given that τ > 1 failed it (§5.8, 1.9–7.7× at τ = 2
and 4, cause unmeasured):
1. GF-10 diagnoses §5.8's failure first, on the GF-7 τ = 4 worlds, with
   `gf2_small_lake_diag`'s method: are the new 1–3-cell lakes pits against
   the frozen receiver tree, unconsolidated-regolith feedback (the §5.6
   mechanism), or the carve?
2. The stage refreshes the routing every step, pins existing lakes, runs no
   deposition, and accounts regolith once per stage (§4.9). That is designed
   to remove the first two candidates; GF-12's B8 run at every τ is what
   shows whether it does.
3. Construction creates no new depression (§4.14, B14).
4. B8's thresholds are unchanged, and it must hold at **every** τ in
   {0.5, 1, 2, 4}, not only at τ = 1 (§5.2).

### 4.16 How the two combine, and in what order (Ruling BM)

**Why both, and not one.** The steady state forgets its starting surface: at
`T → ∞` the stage's output depends only on `U`, `κ` and base level. Where `U`
is small, which is every craton, that steady state is near base level, so
running erosion alone long enough to make rock show would erase the plate
structure the tectonic surface built. So:
- **Construction supplies the big shapes where uplift is slow**: stripped
  plateaus, breached caps, weak-rock lowlands on the cratons. Rock shows there
  as a transient that the budget, not a steady state, sets. Cuesta and
  scarpland country is this kind of landscape.
- **The uplift-driven stage supplies the detail and the active belts**:
  graded, drainage-integrated relief whose steepness follows `U/K`, with θc
  capping the slopes where uplift is fast.

**They agree by sharing their inputs.** Both use the same `U` (construction
through its smoothed `Ū`), the same `κ^c` from §2.3's table, the same
exposure rule and the same column. Construction lowers weak rock further;
the stage then keeps weak rock gentler under uplift. Both push the same way,
so the second refines the first instead of undoing it.

**The order:**

```
structure: compute_height → normalize → volcanism → craters → clamp → sea level   (unchanged)
GEOLOGY (GF-1): the column, from the structural surface                           (unchanged)
CONSTRUCTION (GF-11): layer-through lowering by B(i); fill → regolith              [BM]
priming flow → temperature → weather → correctors (→ currents)                     (reads the constructed surface)
UPLIFT-DRIVEN STAGE (GF-12): N_BM implicit steps, U, κ, routing refreshed           [BM; replaces light SP + rebound]
threshold hillslope (GF-3)                                                          (unchanged; sets scarp angles)
routing → channels → Strahler → trace → intensity → carve (RV-1)                    (unchanged)
climate refresh; passes (glacial + its rebound, …)                                  (unchanged)
```

**The switch.** `WorldParams::relief_model`, off in `WorldParams::defaults`
and in `cartalith_godot::params::defaults()` until GF-13 decides; a
`PARAMS` and `JS_PATHS` row; a save without the key reloads off; no GUI
control (like `geology_processes`). It acts only when `geology_model` **and**
`geology_processes` are on too. With it off, every call is today's by control
flow.

**The clock under BM.** τ scales the construction budget and the stage's
model time, both linearly (§4.14, §4.15). GF-3's threshold hillslope and the
glacial and coastal passes keep §4.12's count laws. So under BM, **τ = 1 is
not identity**: it is the calibrated default age of a BM world. §4.12's
identity-at-τ-1 rule, and B12's first half, still hold for every world with
the switch off.

---

## 5. Measurement proving landforms now track rock

### 5.1 The harness (GF-0)

A Rust test-side harness, run `--ignored` and alone, prints each metric per
seed and extent. It is `crates/cartalith-godot/tests/gf0_geology_harness.rs`.
It lives in `cartalith-godot` because it needs `params::defaults()`, which it
pulls in by `#[path]`, as `params_mapping.rs` does. The commands are in §5.4.

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
- ***Revised 2026-09-27 for Ruling BM*** (the evidence is §5.9). The
  thresholds are unchanged; the population and the control are not:
  - **Stratified by uplift.** ρ is computed within each of ten equal-count
    bands of §4.15's `U` (input, pre-erosion), and the reading is the median
    of the per-band ρ over bands holding ≥ 100 cells of each of the strongest
    and weakest thirds of `s`. Relief tracks uplift first and strength second
    (steady-state relief ∝ `(U/K)^(1/n)`), so only at matched uplift is the
    strength signal the thing measured.
  - **Constructional edifices excluded**: cells with `volcanic_field > V_TH`
    are left out of both groups and reported on their own line.
  - **`s` is read on the rock-blind arm's constructed surface** (§4.14 at
    `c = 0`, before its erosion stage): the rock the landscape would expose if
    rock did not matter. It is the same input map for every arm, it depends
    on no rock-dependent process, and unlike §5.6's pre-erosion map it
    contains exposed substrate (shale) wherever a uniform budget breaches a
    cap.
  - **The control for the relative half is the rock-blind BM arm** (the
    switch on, `c = 0`), not today's world. Gneiss and schist sit in the
    collision belts, where `U` is highest, so a comparison with today's world
    would credit uplift with rock's doing.
  - The legacy form (all interior land, today's world as control) is still
    printed, labelled as such, so §5.4–§5.8 stay comparable.

**B2 — slope tracks strength.**
- **Metric:** median physical slope in degrees (the Sample panel's `grade`
  formula) on strong rock (s ≥ 0.7), divided by the median on weak rock
  (s ≤ 0.4). Selected by the **input** s.
- **Bar:** the ratio ≥ 1.5, and ≥ 1.25 × the control's ratio.
- ***Revised 2026-09-27 for Ruling BM*** (evidence §5.9): the same
  thresholds, with B1's four changes (uplift bands, reading the median of
  per-band ratios; edifices excluded; `s` on the rock-blind constructed
  surface; the rock-blind arm as control).
  - **Why 1.5 is principled.** At steady state under the stage, at matched
    `U` and `A`, `S_strong/S_weak = (κ_weak/κ_strong)^c` (§4.15). The
    least-contrasting pair across B2's thresholds is limestone (s 0.70,
    κ 0.6) against tuff (s 0.35, κ 2.0): `(2.0/0.6)^0.5` = 1.83 at the
    default `c = 0.5`; the most contrasting, granite against unconsolidated,
    is `(4.0/0.3)^0.5` = 3.65 (arithmetic). So 1.5 is below every steady-state
    pair ratio and asks the landscape to be about two-thirds of the way there
    in log terms (`ln 1.5 / ln 1.83` = 0.67, arithmetic). It was not set to
    be met; it was set below what the physics predicts.

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
- ***Revised 2026-09-27 for Ruling BM*** (evidence §5.9):
  - **A cap edge is a breach line**: a land two-layer cell exposing its
    **cap**, with a 4-neighbour that is two-layer and exposes its
    **substrate**. The legacy definition (a two-layer cell whose exposed rock
    differs from *any* neighbour's) counts the lateral margin of a
    sedimentary cover against the granite around it, which is not a caprock
    scarp.
  - **The ratio bar stays ≥ 2.0.** Its basis: a breach-line cell carries the
    step `B·(k_s − k_t)` (§4.14) across one cell, against a substrate slope of
    `U/(K_sub·A^m)`. At 800 km a 69 m step is 10° (§4.14's arithmetic), while
    the control's median strong-rock slope is about 0.4° (§5.7). The bar asks
    for a small fraction of that contrast.
  - **The top-decile share stays ≥ 40 %**, read over breach-line cells whose
    input cap thickness `h` is at least the height one top-decile slope of
    the rock-blind arm needs across one cell (`tan(p90)·cell_m`); the
    population is printed. A cap thinner than that cannot, by itself, hold a
    top-decile face (for example the thinnest cover cap, 10 m, is 1.5° over a
    390.6 m cell, against a land p90 of 2.75–6.56° at 800 km, §5.4;
    arithmetic).
  - **The twin, input-selected: thin caps against thick caps.** Two-layer
    land cells in the lowest quartile of input `h` against those in the
    highest quartile, ratio of median slopes. Thin caps are where breaches
    and scarps form; thick caps stay plateaus. Bar: ≥ 1.25, and ≥ 1.2 × the
    rock-blind arm's (where a uniform budget makes no step, so it should read
    near 1). The legacy twin is unmeasurable by design (§5.5) and is dropped.
  - **Extents.** The bar applies at 800 km. It is reported at 80 km, where
    θc binds (§5.4: 14–18 % of land is above 40°) and caps the face, and at
    8 000 km, where a cap edge is sub-grid and **no pass is expected**
    (§4.14).

**B5 — karst only on soluble rock.**
- 100 % of cells lowered by the dissolution pass have σ > 0. This is exact, and
  a mutation that drops the σ factor must turn the test red.
- Of the closed depressions the pass creates deeper than the lake classifier's
  0.004 threshold, at least 95 % lie on soluble rock.

**B6 — glacial reads jointing.**
- **Metric:** at matched ice discharge (Q decile bands), the median glacial
  lowering on γ ≥ 1.5 rock divided by the median on γ ≤ 0.6 rock.
- **Bar:** ≥ 1.5 in the treatment, and ≥ 1.5 × the control's ratio.
  - *Amended 2026-09-27 from GF-0's measurement:* the bar read "against about
    1 in the control". That expectation is withdrawn. A kernel that reads no
    rock measured 1.32 to 6.58 at 800 km against the stand-in strength map
    (§5.4), because rock and ice discharge are both tied to geography.
    Only a comparison with the measured control arm can separate the
    kernel's doing from the map's.

**B7 — coastal retreat reads rock.**
- **Metric:** land cells lost to the sea per coastline cell, weak rock (ρ ≥ 1.5)
  ÷ strong rock (ρ ≤ 0.3), over the default passes.
- **Bar:** ≥ 3, and ≥ 2 × the control's ratio. *(The relative half was added
  2026-09-27: a coastal pass that reads no rock measured 0.13 to 2.10 on the
  stand-in map, §5.4, so an absolute bar alone could be met, or missed, by
  geography.)*
- ***Revised 2026-10-12 by Ruling CH, AFTER the result was seen; not a
  pre-registered bar.*** B7 is read with the tidal marsh off in both arms
  (`marsh_band` 0 in the metric's replay; the pass itself is unchanged).
  The marsh's value-noise bump reads no rock under §4.5 and sinks cells on
  its own. Disclosure: with the marsh the bar's relative half read
  1.27-1.49 × control on the 4 measurable seeds; without it 1.62-2.17 ×,
  and the revised bar passes on **2 of 4** measurable seeds (483920,
  12345), not all (§7 "GF-4 measured").
- Ruling BJ turns the pass on in the app at GF-4 (§4.5), so the bar is
  measured from GF-4 on. GF-0 measures its baseline on an extra arm with the
  pass forced on (§5.4).

**B8 — no hydrology regression**, against RV-1's own metrics
(`_riverzoom_probe`):
- ocean cells on channels = **0** (hard);
- river cells that are lake ≤ control + 2 percentage points;
- 1–3-cell lakes ≤ 1.25 × control.
- ***Extended 2026-09-27 for Ruling BM:*** with the relief switch on, all
  three hold at **every** τ in {0.5, 1, 2, 4}, against today's app world as
  control. The thresholds are unchanged. §5.8 found B8 failing at every
  τ > 1 under GF-7; under BM, τ is a first-class knob, so a pass at τ = 1
  alone is not a pass.

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

**B11 — painting is exact where nothing changed, and local where it did**
(GF-8, §4.13).
- Painting a region with the rock already there, then committing, moves the
  field by **0 bytes**, and it moves the column by 0 bytes apart from
  `rock_painted`. This is exact. A mutation that makes the two re-erosion runs
  differ, such as dropping the κ slice from one of them, must turn it red.
- Painting shale into a granite interior on a fixed seed at 800 km:
  - the painted cells' mean lowering exceeds that of the unpainted cells in
    the same window;
  - no cell outside the padded window `W` moves;
  - the step at `W`'s ring is under 2 % of `W`'s relief, which is
    `tile_erode`'s own lip bar.
- Undo restores the field and the column byte-identically, and redo
  re-applies them byte-identically.
- The painted column survives a save and a reopen byte-identically.

**B12 — the clock scales what it says it scales** (GF-7, §4.12).
- At τ = 1, every world is byte-identical to one generated without the clock
  parameter, on all five seeds. This is identity by control flow.
- At τ ∈ {0.5, 2, 4}, on all five seeds at 800 km:
  - mean incision depth (the pre-erosion surface minus the final surface, over
    channel cells) rises strictly with τ;
  - coastal land lost and mean glacial lowering each rise with τ, and each
    rises by less than τ's own ratio between τ = 2 and τ = 4. That is the
    saturation, measured rather than assumed;
  - the weathered mantle equals `h(τ)` to within 1e-6 m on bare-rock cells.
    This is asserted against the literal values 1.333 m at τ = 0.25 and
    2.686 m at τ = 4, never against the constants.
- The cost at 0.5, 2 and 4 is reported as a median with min..max. It is
  disclosed and not gated.
- ***Extended 2026-09-27 for Ruling BM*** (the switch on):
  - The channel-incision half is **not used** for BM worlds. §5.8 showed it
    fails for a reason that is not a defect: channels are re-traced on the
    new surface, and at τ = 4 only 53–58 % of the control's channel cells are
    still channels. Under uplift it also mixes uplift with incision.
  - In its place: in the top uplift band (B1's banding), median 9 × 9 relief
    rises with τ from 0.5 to 4, and B1's and B2's readings do not fall from
    τ = 1 to τ = 4. The theory predicts both: the stage approaches a steady
    state whose relief is `U/K`-set, so relief grows and its rock contrast
    sharpens with model time.
  - The cost at τ ∈ {0.5, 2, 4} must equal τ = 1's within the brackets of
    two independent runs, because τ scales `dt`, not `N_BM` (§4.15).

**B13 — the stage reaches the steady state the theory predicts** (GF-12;
the positive control through the BM stage, which §5.6 could not assert for
the zero-uplift kernel).
- A synthetic fixture: a tilted plane, uniform `U`, two rock blocks, no rain.
- After the stage at `T ≥ 10/C` for the fixture's smallest channel `C`
  (arithmetic: `e^(−10)` of the initial deviation left), at channel cells of
  matched `A`, the slope ratio equals `(κ_weak/κ_strong)^c` within 5 %, and
  equals 1 within 5 % at `c = 0`.
- Asserted against literals: at `c = 0.5`, granite against shale is
  `(2.5/0.3)^0.5` = 2.887 (arithmetic), never against the table read back.
- A mutation that drops `κ` from the stage, or `U`, turns it red.

**B14 — construction is consistent with the column and makes no pits**
(GF-11).
- The `contact` array is **bit-identical** before and after construction.
- No land cell of the structural surface ends below sea unless it is joined
  to the ocean. *Reworded for Ruling BN (§9 Q14): the coast may move, so
  the check is that every new below-sea cell is ocean by `build_water_bodies`
  (a bay), never a stranded inland depression. It read "No land cell of the
  structural surface becomes sea".*
- Construction creates **zero** new closed depressions deeper than the lake
  classifier's 0.004 (exact: §4.14's fill rule makes it so by construction).
- Every cell the fill raised has its `regolith` raised by the same amount,
  exactly.
- At `c = 0` the constructed surface equals `z₀ − m_to_norm(B)` (before the
  sea floor and the fill) on every cell, bit for bit.

**B15 — uplift headroom and balance** (GF-12).
- Cells clamped at 1.0 by the stage: **0** on all fifteen worlds (hard).
- The land-mean surface change over the stage on the rock-blind arm at τ = 1
  is within the brackets of the `U₀` calibration (§4.15), printed with its
  population. This is the calibration's own check, not a landform bar.

### 5.3 What the numbers are for

- B1–B3 are the owner's question: *"how strong they are carved"*.
- B4 is the precondition BG needs.
- B5–B7 are the per-process claims of Ruling BH item 3.
- B8–B10 are the guards.
- B11 and B12 are Ruling BJ's painting and clock.
- B13–B15 are Ruling BM's mechanism checks: the stage does what the theory
  says, construction stays consistent with the column, and uplift neither
  clips summits nor drifts the hypsometry.

### 5.4 GF-0 findings: the arm-1 baseline (measured 2026-09-27)

**What was run.** Arm 1 only: today's pipeline at `params::defaults()`,
2048 × 1311, on the five seeds and three extents of §5.1, in a release build.
Arm 2 needs GF-1's column and does not exist yet.

```text
cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf0_bars
cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf0_b9_cost
```

**The strength map is a stand-in, not the control arm.** It is
`resistance_field` (`compute_resistance`), with "strong" its top quartile and
"weak" its bottom quartile over each bar's own population. §7's GF-0 entry
prescribes this stand-in. The scope's control arm, the GF-1 column with
`c = 0`, will replace it.

**How each bar is computed.** These are the harness's definitions, where §5.2
left a choice open:
- **Populations.**
  - B1 to B3: water-body class 0 (`build_water_bodies`, the classification
    `sample_cell` reads), excluding any cell with an ocean cell within
    Chebyshev distance 4.
  - Relief is max − min over 9 × 9, clipped at the map edge, in metres by
    `peak_m / (1 − sea)`.
  - Slope is the Sample panel's `grade` formula, through
    `build_slope_field`.
- **B3** is **undefined** whenever either mean depth is ≤ 0. A non-positive
  mean is net raising (rebound, deposition), and a ratio with it is a sign,
  not a contrast. The pre-erosion surface is the same world generated with
  `carve_rivers = false` and every pass off: nothing after the volcanism clamp
  writes `field` on that path. The harness asserts that the two runs share
  `age_field` and `plate_id`.
- **B6.**
  - The lowering is `glacial_kernel` **alone**, re-run on the pre-glacial
    field and temperature (the same world with `passes.glacial = false`). So
    it excludes the rebound after it. The harness asserts that kernel +
    `isostatic_rebound` + the passes' clamp reproduces the generated world
    **bit for bit**.
  - Ice cells are land at or above the snowline and below 0 °C, the kernel's
    own gate.
  - Discharge is a stand-in: unit upstream area over the depression-filled
    surface (`compute_flow` over `build_routing_surface`). The kernel's own
    `Q` is the same quantity on its own tree, which it does not return.
  - The bands are **equal-count** deciles of the ice cells ranked by that
    discharge. The first run used value deciles, which collapsed to 1 band of
    10 on seed 24601, because unit area is heavily tied.
  - The reported value is the median of per-band ratios, over the bands with
    at least 10 cells in each group and a positive strong median.
- **B7** runs an extra arm: arm 1 with `passes.coastal = true`.
  - The coastline is arm-1 land that is 4-adjacent to ocean.
  - A cell counts as lost if it was arm-1 land and is below sea after the
    pass.
- **B8** walks every run of `river_entities` at min order 1, over the same
  classification. It counts every point of each run except the last, as
  `_riverzoom_probe` does. That probe also skips runs the Godot draw plan marks
  `parallel_of`, a plan no test can construct. So these path counts are over
  a **superset** of the probe's.

**Validity controls.** They are ordinary tests in the same file, so they run
in every `cargo test --workspace`.
- **Positive control.** A synthetic fixture: 64-cell strong and weak blocks
  in a checkerboard, with roughness proportional to strength, and weak rock
  lowered more.
  - It must give B1 ρ > 0.5, and the inverted map ρ < −0.5.
  - B2 must exceed 1.5.
  - B3 must recover the built ratio, 0.034 / 0.013.
  - It controls the **metric code**. There is no treatment to run it through
    until GF-2, which re-runs it through the rock-reading kernel.
- **Negative control.** On every real world, the stand-in is permuted over
  the population. **Measured |B1| ≤ 0.001 on all 15 worlds** (bar: < 0.05).
  B2 and B3 on the permuted map read 0.95 to 1.04.

**The baseline at 800 km**, where §5.2's bars apply:

| Seed | B1 ρ | B2 slope strong ÷ weak (medians, °) | B3 depth weak ÷ strong (means, m) | B6 glacial weak ÷ strong (ice cells; bands) | B7 coastal weak ÷ strong | B8 ocean on paths · lake % of path cells · 1–3-cell lakes |
|---|---|---|---|---|---|---|
| 483920 | −0.4611 | 0.2172 (0.290 / 1.335) | **undefined** (−0.28 / 1.86) | 3.7644 (6 343; 10/10) | 1.5055 | 0 · 1.177 % of 23 533 · 46 |
| 24601 | −0.4656 | 0.4298 (0.276 / 0.643) | 0.7874 (0.80 / 1.02) | 6.5833 (227 295; 10/10) | 0.1334 | 0 · 2.232 % of 31 368 · 65 |
| 71077345 | −0.3429 | 0.3677 (0.298 / 0.811) | **undefined** (−0.23 / 0.22) | 1.3229 (5 165; 10/10) | 1.4550 | 0 · 1.419 % of 23 746 · 26 |
| 12345 | −0.4860 | 0.1391 (0.315 / 2.266) | 1.4047 (0.64 / 0.46) | 3.1294 (10 832; 9/10) | 0.4524 | 0 · 4.661 % of 21 905 · 105 |
| 314159 | −0.3294 | 0.3244 (0.324 / 0.998) | 0.5238 (0.97 / 1.85) | 1.9380 (33 904; 10/10) | 2.1049 | 0 · 9.347 % of 27 325 · 81 |

Every B1–B3 population is at least 312 089 cells. Every B7 coastline group is
at least 1 076 cells.

**The other extents** (ranges over the five seeds):

| Extent | B1 ρ | B2 | B3 | B6 | B8 ocean · lake % · 1–3-cell lakes |
|---|---|---|---|---|---|
| 80 km | −0.3770 … −0.1025 | 0.2864 … 1.0967 | 1.9913 … 5.2360 | 0.3168 … 1.5343 | 0 · 8.776 … 25.365 · 238 … 621 |
| 8 000 km | −0.4649 … −0.3207 | 0.1685 … 0.4182 | 0.3247 … 0.8384 | 0.9517 … 4.8333 | 0 · 2.385 … 16.775 · 19 … 62 |

**The bars that cannot be measured yet**, each for its stated reason:
- B4: there is no two-layer column before GF-1, and its input-selected twin
  needs the same column.
- B5: there is no dissolution pass before GF-5.
- B10's rock-type count and two-layer share: there is no model before GF-1.
  The harness prints the legacy `build_lithology` land shares, labelled as
  post-erosion labels and not the model.

**B9 (cost).** `generate_terrain`, seed 483920, 800 km, 2048 × 1311, the app's
defaults, CPU path; one warm-up, then five timed runs:
- **median 3.115 s (3.089 … 3.124)**;
- an independent re-run in a separate process: **3.095 s (3.068 …
  3.140)**, inside the first bracket.
- Two Godot processes from another lane were resident, and idle by their
  memory, during both runs.

**B10 (determinism).** The same seed twice gave byte-identical `field`,
`temperature`, `rainfall`, `flow_discharge`, `river_mask` and
`resistance_field` on all five seeds at 800 km.

**Land-slope distribution, which settles §4.3's expectation:**

| Extent | p50 | p90 | p99 | p99.9 | max | share of land > 40° |
|---|---|---|---|---|---|---|
| 80 km | 12.4 … 13.5° | 46.8 … 53.8° | 77.2 … 80.5° | 84.6 … 86.0° | 87.9 … 88.6° | 14.1 … 18.3 % |
| 800 km | 0.44 … 0.56° | 2.75 … 6.56° | 26.8 … 33.8° | 40.6 … 45.3° | 56.9 … 70.3° | 0.13 … 0.37 % |
| 8 000 km | 0.048 … 0.061° | 0.29 … 0.61° | 2.8 … 3.9° | 4.9 … 5.8° | 11.9 … 17.8° | 0 |

**What the baseline says.**
1. **The stand-in is confounded with geography, as §5.1 predicted of any map
   not built from causes.**
   - `compute_resistance` makes rock hardest in old interiors far from any
     margin, and those are flat. So on a pipeline whose only rock-reading
     process is a 1.36× stream-power factor, relief and slope correlate
     *negatively* with strength (B1 −0.33 to −0.49, B2 0.14 to 0.43 at
     800 km).
   - §2.4's granite is also "old interior". The GF-1 control arm will
     probably inherit part of this, which would put B1's absolute half (ρ ≥
     0.25) further away than its relative half (Δρ ≥ 0.15).
   - The bars are left as they are: §5.2 sets them before measurement. GF-1
     measures the control arm, and any change is disclosed then.
2. **"About 1 in the control" was wrong for B6, and plausibly for B7.** A
   kernel that reads no rock gave 1.32 to 6.58 (B6) and 0.13 to 2.10 (B7)
   against the stand-in. Both bars now also require a multiple of the
   measured control (§5.2).
3. **B3 is undefined on 2 of 5 seeds at 800 km.** At the app's default extent,
   generation lowers weak rock by about a metre on average, and net lowering
   is negative in places, where rebound and the carve outweigh the light
   stream-power pass. At 80 km the same means are 36 to 281 m. GF-2 should
   consider measuring B3 on the stream-power call alone, before rebound, as
   B6 now measures the glacial kernel alone. The clock (§4.12) also bears on
   it: τ > 1 deepens the incision.
4. **§4.3's expectation holds at 800 km, and the other extents behave very
   differently.**
   - At 800 km, 0.13 to 0.37 % of land is steeper than 40°. The threshold
     hillslope will bind only on a small share, mostly the cap edges it
     exists for.
   - At 80 km, 14 to 18 % is. The threshold will be a dominant process there,
     and GF-3's cost and B8 need checking at that extent.
   - At 8 000 km no land cell is steeper than 17.8°, so the stage is inert at
     continental extent for every θc in §2.3's table.
5. **RV-1's hard guarantee holds everywhere measured.** There are 0 ocean
   cells on river paths on all 15 worlds.

### 5.5 GF-1 findings: the column's first values (measured 2026-09-27)

**What was run.** The same `gf0_bars` command as §5.4, in release, on the
GF-1 tree, where `params::defaults()` now runs the geology stage and stores its
column (`WorldState::geology`), and no process reads it. So every
"control arm" line the harness now prints **is** §5.1's control arm (`c = 0`,
no new stages).

**Identity.** Every §5.4 line (B1 to B3 stand-in, the negative control, B6, B7,
B8, the slope distribution, B10's array determinism and the legacy shares)
is **byte-identical** to the pre-GF-1 run on all 15 worlds (the two outputs
were diffed line for line). `cartalith-engine/tests/geology_gf1.rs` asserts
the same bit for bit on every pre-existing `WorldState` array, switch on
against switch off.

**Where the model lives.** The table (§2.3) and the derivation (§2.4) are
`cartalith-terrain/src/geology.rs` (`ROCK_PROPS`, `build_geology`); this
scope named no file. The setting is kept by `stamp_volcanoes_*_traced`, which
record the edifice that won each cell's `volcanic_field` max
(`VolcanoTrace`); the engine reads it in the geology stage and drops it.

**B10 at 800 km** (land = water-body class 0):

| Seed | Rock types exposed on land | Two-layer share of land |
|---|---|---|
| 483920 | 9 | 0.1530 |
| 24601 | 9 | 0.1431 |
| 71077345 | 10 | 0.2375 |
| 12345 | 10 | 0.2786 |
| 314159 | 10 | 0.1967 |

Both B10 bars pass on all five (≥ 4 types, ≥ 5 % two-layer), and the column
is byte-identical run to run. Granite is 0.57–0.75 of land, sandstone
0.10–0.18, oceanic basalt 0.004–0.22, limestone 0.03–0.06, schist
0.05–0.08; gneiss, plateau basalt,
andesite, tuff and shale are each under 0.03. Exposed shale is the cover's
substrate, where the pre-erosion surface already lies below the contact.

**B4 (control arm).** Cap-edge ÷ substrate median slope: 0.035–0.243 at
800 km, 0.436–1.213 at 80 km, 0.020–0.621 at 8 000 km; the share of cap-edge
cells in land's top slope decile: 0.20–0.29 at 800 km. **Its input-selected
twin is not measurable** on any of the 15 worlds, and cannot be with this
derivation: every rock type is either always two-layer (sandstone, limestone,
the volcanic caps) or always single-layer (granite, gneiss, schist, oceanic
basalt), so no top rock has both groups. GF-2 needs either a different twin
(for example two-layer cells against single-layer cells of the same
*substrate*) or a derivation that puts one rock in both groups.

**B1 to B3 on the model's own `s`** (the control arm, §5.2's split, strong
`s ≥ 0.7`, weak `s ≤ 0.4`), at 800 km:
- B1 ρ −0.46 to −0.32; B2 0.041 to 0.051.
- **B3 7.9 to 108**, already far above the treatment bar of 2.0. Weak rock
  here is mostly tuff on volcano flanks, which the existing kernels erode
  hardest because they are steep, not because they are weak. Under §5.2's own
  rule ("if the control already meets a bar, that bar is not evidence") B3's
  absolute half is not evidence at GF-2; its relative half (≥ 1.6 × control)
  is the one that can speak. Not changed here: a bar change is the owner's.

**The property table, checked at its sources for GF-1:**
- **Verified:** Hoek–Brown `mi`, Marinos & Hoek (2000) Table 2 (opened in the
  paper as Rocscience publishes it): granite 32 ± 3, gneiss 28 ± 5, schists
  12 ± 3, andesite 25 ± 5, sandstones 17 ± 4; and, as the table's own
  parenthesised estimates, basalt (25 ± 5), tuff (13 ± 5), micritic limestone
  (9 ± 2), shales (6 ± 2). **Two corrections to §2.3's column:** limestone is
  pinned at micritic 9 ± 2 (sparitic is 10 ± 2); and "mudstone 4" is not in
  the table, whose 4 ± 2 is *claystone*.
- **Cited, not verified** (the source could not be opened for GF-1): Selby's
  (1980) class boundaries, which the code uses as labels only, with each `s`
  asserted inside its quoted band; Freeze & Cherry (1979) Table 2.2's
  permeability ordering; the angle of repose (Carson & Kirkby 1972) behind
  unconsolidated θc 33°.
- **Judgements, as §2.3 labels them:** every `s`, κ, θc, γ, ρ, the jointing
  classes, and every §2.4 derivation constant (belt widths, low depth, cover
  and cap thicknesses, `V_TH`, the arc core radius, the 30° carbonate band).

**Departures from §2.4, disclosed.**
- The cover and volcanic contacts are **surface-parallel** (the pre-erosion
  surface minus a smooth thickness), not horizontal plus a warp. In a
  structural low the pre-erosion surface is near-flat, and the facies noise
  already makes the outcrop line irregular.
- An arc edifice's cap and a hotspot or rift plateau-basalt cap are both
  `vf · h` thick (the edifice's own normalised height times its intensity),
  an approximation of the stamped height.
- `stamp_volcanoes_simple` (`volc.provinces = false`, off at both
  boundaries) rolls no setting. Its cells are stored as setting **4,
  unclassified**, which §2.6's table has no code for; §2.6 needs that row
  before GF-8 writes the set. Such a volcano is read as basaltic next to a
  rift and as an arc otherwise.

**Not built in GF-1, against §7's list:** the §2.6 save set and
`SAVEFILE_COMPAT.md` §8.4 (moved to GF-8 by the brief; a reopened world has
no column and says so); the single-layer import column (an import has none
and says so); the lithology map view's 11-colour palette.

**Memory**, measured with `_memlane_peak` (2048 × 1311, seed 483920,
800 km, the app's parameters, `MEMLANE_NO_GEOLOGY=1` for the before):
`WorldState` 153.63 → 181.80 MiB (+11.0 B/cell, the §2.5 arithmetic), and
the generation peak **486.50 → 513.94 MiB (+27.44 MiB)**. The allocator count
is deterministic, so one run each.

### 5.6 GF-2 findings: control against treatment (measured 2026-09-27)

**What was built.** `cartalith_erosion::stream_power_kernel_rock` (§4.1:
`κ(exposed)^c` in place of the legacy factor, the exposed rock re-read before
every iteration, which is the contact switch), `kappa_multiplier`,
`lift_column` (§4.2) and `account_regolith` (§4.9). `generate_terrain_inner`
routes every stream-power call (light pass, evolve cycles, sediment fill),
every rebound (light pass, glacial, evolve), the sediment routing, and the
regolith stripping of the glacial pass and the RV-1 carve through one gate,
`RockContext`. It is on only when **both** `geology_model` and a second
switch, `WorldParams::geology_processes`, are on; otherwise every legacy call
runs verbatim. The control arm (§5.1) is the same parameters with
`geology_processes` off: the world GF-1 shipped.

**Gated off in the app, pending the bars** (coordinator decision, 2026-09-27,
on the measurement below). `geology_processes` is off in
`WorldParams::defaults()` **and** in `cartalith_godot::params::defaults()`,
with a `PARAMS` and `JS_PATHS` row and the same rule as `geology_model` for a
save without the key (it reloads off). The reason: B8's small-lake rise (1.28×
and 1.26× against 1.25×) is the regression RV-1 fixed, and B1 and B2 show no
gain. So the app keeps GF-1's inert column and today's worlds, and the harness
and the GF-2 tests turn the switch on explicitly. **GF-3 and GF-7 must
re-measure B1, B2 and B8 with the switch on** (`gf2_arms`) before anyone
turns it on in `params::defaults()`.

**Choices where §4 left one, each with its reason.**
- `c` is `tect.resist`, as §4.1 and §9 Q5 say; at the app default that is
  0.5, so κ spans 0.55 (granite) to 2.0 (unconsolidated). Not tuned.
- The legacy term `max(0.05, 1 − 0.7·resist·R)` is dropped, not multiplied in:
  §4.1's formula has no `resist` term.
- **Regolith is accounted by the caller, on each call's net change** (§4.9:
  "the kernels do not change. The caller adds `max(0, field_after −
  field_before)` to regolith"; a net lowering strips regolith first). The
  kernel reads regolith through §2.5's exposure rule and never writes it.
  *A first build wrote regolith inside the iterations* (every rise, including
  a pit the implicit scheme fills, became regolith at once). Measured on the
  same five seeds at 800 km it roughly doubled the 1–3-cell lakes against the
  control (46 → 99, 65 → 183, 26 → 72, 105 → 193, 81 → 159): a pit filled in
  one iteration read as unconsolidated (κ = 4) in the next and was incised
  harder. The diagnostic (`gf2_small_lake_diag`) found the new small lakes
  sitting on cells that read unconsolidated, tens of metres below the control.
  The scope's own caller-side design replaced it; no constant was changed.
- Tectonic uplift raises the contact with the field inside the kernel (the
  §4.2 rule applied to uplift). It is zero at `stream.uplift = 0`, so it moves
  nothing on either boundary today.
- The glacial pass and the carve only **strip** regolith (they read rock at
  GF-4 and GF-6). Tidal flats, the marsh, coastal debris and hillslope
  transport do not write regolith yet: §4.9 assigns them to GF-4 (and the
  threshold hillslope to GF-3), and all are off in the app.
- The harness judges both arms against **one input map**: `s` of the rock
  exposed on the pre-erosion surface (the carve-off, passes-off world), over
  the control's interior land. §5.1 requires the same pre-erosion rock map;
  exposure on each arm's own final surface would select by the output.

**Commands** (release, run alone):

```text
cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf2_arms
cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf2_b9_cost
```

**At 800 km** (control → treatment). B3 "SP alone" is §5.4 item 3's
advice: the light stream-power call replayed alone on the pre-erosion world,
before rebound, the carve and glacial.

| Seed | B1 ρ | B2 | B3 full | B3 SP alone | B8 lake % | B8 1–3-cell lakes |
|---|---|---|---|---|---|---|
| 483920 | −0.4600 → −0.4617 | 0.0501 → 0.0506 | 3.73 → 8.79 | 12.71 → 27.63 | 1.177 → 1.154 | 46 → **59** |
| 24601 | −0.3155 → −0.3141 | 0.0475 → 0.0480 | 5.94 → 13.47 | 16.25 → 33.81 | 2.232 → 2.522 | 65 → **82** |
| 71077345 | −0.3922 → −0.3900 | 0.0491 → 0.0492 | 1.22 → 3.43 | 14.14 → 29.40 | 1.419 → 1.581 | 26 → 28 |
| 12345 | −0.4232 → −0.4321 | 0.0487 → 0.0488 | 4.15 → 8.36 | 10.00 → 19.67 | 4.661 → 4.077 | 105 → 111 |
| 314159 | −0.4268 → −0.4272 | 0.0394 → 0.0395 | 5.94 → 13.98 | 15.62 → 32.65 | 9.347 → 9.312 | 81 → 59 |

Every B1–B3 population is at least 1 248 307 interior-land cells; the weak
group (`s ≤ 0.4` on the pre-erosion map) is 4 589 to 25 180 of them. Ocean
cells on river paths are 0 in both arms on all 15 worlds.

**The bars.**
- **B1 fails** on all five (ρ ≥ 0.25 and Δρ ≥ 0.15): Δρ is −0.009 to +0.002.
- **B2 fails** on all five (≥ 1.5 and ≥ 1.25 × control): the ratio moves by
  under 1.3 %.
- **B3 passes** on all five, both halves, full (2.02 to 2.80 × control) and
  on the stream-power call alone (1.97 to 2.17 × control, each ≥ 2.0 in
  absolute value on the SP-alone line too). §5.5 already said B3's absolute
  half is not evidence here; the relative half is, and it holds.
- **B8 fails on 2 of 5**: lake share holds everywhere (within +0.29 pp), but
  the 1–3-cell lakes rise 1.28× (483920) and 1.26× (24601) against the 1.25×
  bar; the other three are 1.08×, 1.06× and 0.73×. At 80 km and 8 000 km B8
  passes on all ten worlds.
- **B9 passes, no difference established.** Seed 483920, 800 km, seven
  alternating runs per arm in one process: control 3.309 s (3.246 … 3.954),
  treatment 3.434 s (3.399 … 3.880), ratio of medians 1.038. An independent
  re-run: 3.287 s (3.148 … 3.734) against 3.333 s (3.284 … 3.733), 1.014. The
  brackets overlap both times.
- **B10 passes**: deterministic on all five (field, river mask, regolith,
  contact byte-identical run to run), 10–11 rock types exposed on land,
  two-layer share 0.143–0.280.

**Why B1 and B2 do not move: the effect is metres, the relief is hundreds of
metres.** Treatment minus control on land (seeds 483920 and 314159): median
−0.01 m, 1st to 99th percentile −10 to +15 m and −7 to +11 m. By pre-erosion
rock, the mean change is +0.04 m and +0.10 m on granite (0.75 and 0.62 of
land), −4.9 m and −7.3 m on tuff, −0.6 m and −1.9 m on schist, +3.9 m and
+5.4 m on andesite. The difference map shows the change confined to channel
lines (some incised deeper, some shallower) and volcano flanks; interfluves do
not move. This is §1.4 point 3 and 4 made concrete: nine iterations of
transient incision under zero uplift lower channels by metres, and a 2.9×
κ contrast between granite and shale changes metres into more metres, which
no 9 × 9 relief statistic at 390 m cells can see. The weak class is also
tiny on the input map (tuff and andesite flanks; pre-erosion exposed shale
is 0), and it sits on volcano flanks that are steep for reasons that are not
rock, which is why B2's control ratio is 0.04–0.05 in the first place.
Neither is fixable by tuning inside §4.1 without the scope's procedure: GF-7's
clock (more passes) and GF-3's threshold hillslope are the scope's own levers
on landform scale, and B1/B2 need re-measuring after them.

**The positive control through the kernel** (`positive_control_through_the_rock_kernel`,
granite and shale checkerboard on a tilted plateau, contrast 0 against 0.5):
B3 0.998 → 2.495 (asserted ≥ 2.0 and ≥ 1.6 × control), a permuted map near 1
(asserted). B1 reads −0.020 → 0.209 and B2 0.994 → 1.062: §5.1's "the
treatment must give B1's ρ > 0.5" is **not met** by the kernel on this
fixture, so that sentence of §5.1 is not asserted. The same reason as above:
nine iterations deepen the channels in weak rock, which raises the local
relief of weak rock as much as it lowers its surface.

**The column after generation** (land, 800 km): regolith between 0 and
`R_EXPOSE` on 0.049–0.088, thicker (reads unconsolidated) on 0.005–0.014; the
substrate is exposed on 0.12–0.21 % of land, against 0 on the pre-erosion
surface -- caps are breached, in few places.

**What moved, and what did not.** No golden moved and no test was
re-recorded. The parity path is switch-off (`WorldParams::defaults`), so the
16 civ suites on `pre_rv1_world`, the kernel goldens and
`golden_parity_pipeline.rs` pass unchanged; §6.2's `pre_bh_world` capture is
a GF-9 action (it is needed only when the switch is deleted) and is not built.
With the processes gated off in the app, **the app-default world is
bit-identical to HEAD (`2cf0143`)**: a scratch test hashed `field`,
`temperature`, `rainfall`, `flow_discharge`, `river_mask`, `river_floor`,
`stream_order`, `resistance_field` and all five column arrays of
`params::defaults()` at 2048 × 1311, 800 km, seeds 483920 and 314159, in a
worktree built from HEAD and in the working tree; every hash matched.
`geology_gf1.rs`'s GF-1 identity (app-like parameters, model on against model
off, every array) runs on the app's own parameters again and passes.

### 5.7 GF-3 findings: the threshold hillslope, control against treatment (measured 2026-09-27)

**What was built.** `cartalith_erosion::threshold_hillslope` (§4.3):
`erode_thermal`'s move rule with a per-cell threshold
`critical_talus(θc) = tan(θc)·cell_m·(1 − sea)/peak_m`, `θc` from
`ROCK_PROPS` for the rock each cell exposes. `erode_thermal` and the new stage
share one body, `thermal_core`, whose `per_cell = None, wrap = false` arm is the
legacy port's statements, so `golden_parity_thermal.rs` is bit-identical by
control flow. The engine runs it through `RockContext::threshold_hillslope`,
behind the same gate as GF-2 (`geology_model` **and** `geology_processes`),
which stays **off in the app**. So the treatment arm below is GF-2 plus GF-3,
and the control is the app's own world, as in §5.6.

**Choices where §4.3 left one, each with its reason.**
- **Placement:** once, in the light pass, after stream power and its rebound
  and before the channels are traced (§3.1's order; §3.2 item 2 keeps that
  block gated by `carve_rivers`). It does not follow the `evolve_cycles` or
  `sediment_fill` calls: §3.1 lists the stage once.
- **No rebound after it.** The stage moves mass a few cells downslope, and
  `isostatic_rebound` counts only net removal, so it would lift every shedding
  cell and ignore the cells that received the same mass.
- **Whose threshold:** the shedding cell's (§4.3 indexes `talus_i` by the cell
  whose excess moves).
- **When rock is read:** the exposed rock is re-read at the start of every
  pass (a cap lowered through its contact sheds at the substrate's angle from
  the next pass), but regolith is the stage's starting thickness, and the net
  change is accounted once, afterwards, by §4.9's caller-side rule
  (`account_regolith(..., gains_are_deposit = true)`): the talus a cell
  receives becomes regolith (§4.3), and what it sheds strips regolith first.
  §5.6 measured that writing regolith inside the iterations feeds back.
- **World maps wrap in x**, as a generation stage must; the manual Erode op
  keeps the reference's no-wrap `erode_thermal`.
- **`N_h = 8`** (`THRESHOLD_HILLSLOPE_PASSES`), by §4.3's own procedure, B4
  and B9. At 800 km on the five seeds, `N_h` of 0, 8, 16 and 32 were measured
  (a scratch copy with the count read from an environment variable; 0 is GF-2
  alone):
  - **B4 fails at every count** and does not separate them (treatment
    0.035–0.261; top-decile share 0.197–0.325).
  - **B1 and B2 read the same to four decimals at every count.**
  - **B8's 1–3-cell lakes:** at 0, 59 / 82 / 28 / 111 / 59, which reproduces
    §5.6's GF-2 figures exactly; at 8, 50 / 72 / 24 / 106 / 54; at 16, 48 / 66 /
    24 / 107 / 53; at 32, 49 / 69 / 25 / 108 / 53. Control: 46 / 65 / 26 / 105
    / 81. Every count ≥ 8 passes.
  - **B9 decided it.** At 16 passes the live tree read 1.202 (control 3.220 s,
    3.182 … 3.290; treatment 3.871 s, 3.731 … 4.028) and 1.132 against the
    1.20 bar; 8 is the smallest count measured that keeps B8 passing. Eight
    passes leave `0.75^8 ≈ 0.100` of a lone step's excess over `θc`
    (arithmetic): the stage relaxes toward `θc` and does not reach it.

**Commands** (release, run alone; unchanged from §5.6):

```text
cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf2_arms
cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf2_b9_cost
```

`gf2_arms` now also prints B4 for both arms, each on its own final column
and surface.

**At 800 km** (control → treatment, `N_h = 8`):

| Seed | B1 ρ | B2 | B3 full | B3 SP alone | B4 edge ÷ substrate · top-decile share | B8 lake % | B8 1–3-cell lakes |
|---|---|---|---|---|---|---|---|
| 483920 | −0.4600 → −0.4617 | 0.0501 → 0.0506 | 3.73 → 8.84 | 12.71 → 27.63 | 0.243 → 0.253 · 0.294 → 0.325 | 1.177 → 1.146 | 46 → 50 |
| 24601 | −0.3155 → −0.3141 | 0.0475 → 0.0480 | 5.94 → 13.47 | 16.25 → 33.81 | 0.035 → 0.050 · 0.286 → 0.306 | 2.232 → 2.531 | 65 → 72 |
| 71077345 | −0.3922 → −0.3900 | 0.0491 → 0.0492 | 1.22 → 3.52 | 14.14 → 29.40 | 0.213 → 0.119 · 0.254 → 0.286 | 1.419 → 1.585 | 26 → 24 |
| 12345 | −0.4232 → −0.4320 | 0.0487 → 0.0488 | 4.15 → 8.36 | 10.00 → 19.67 | 0.036 → 0.068 · 0.200 → 0.211 | 4.661 → 4.072 | 105 → 106 |
| 314159 | −0.4268 → −0.4272 | 0.0394 → 0.0395 | 5.94 → 13.99 | 15.62 → 32.65 | 0.156 → 0.166 · 0.251 → 0.213 | 9.347 → 9.307 | 81 → 54 |

Ocean cells on river paths: 0 in both arms on all 15 worlds.

**At the other extents** (ranges over the five seeds, treatment unless stated):
- **80 km:** B1 −0.2505 … −0.0674 (Δρ +0.0003 … +0.022); B2 0.2239 … 0.3411;
  B3 full 2.60 … 4.21, **failing its relative half** on all five (0.91 …
  1.01 × control); B3 SP alone passes; B4 0.646 … 1.403, top-decile share
  0.101 … 0.137; B8 passes on all five (small lakes 284 → 320, 621 → 631,
  312 → 213, 238 → 246, 519 → 444).
- **8 000 km:** B1 −0.4472 … −0.2947; B2 0.0085 … 0.0113 where measurable
  (not measurable on 24601 and 71077345, as in §5.6); B3 passes where
  measurable; B4 0.026 … 0.694; B8 passes on all five (worst 57 → 68, 1.19×).

**The bars.**
- **B1 fails** on all five at 800 km: Δρ −0.009 … +0.002, the same as GF-2
  alone.
- **B2 fails** on all five: the ratio moves by under 1.3 %.
- **B3 passes** on all five at 800 km, both halves, full (2.01 … 2.88 ×
  control) and SP alone.
- **B4 fails** on all five at 800 km (0.050 … 0.253 against ≥ 2.0; top-decile
  share 0.211 … 0.325 against ≥ 0.40), and on all five at 80 km. Its
  input-selected twin is still unmeasurable (§5.5): no top rock has both
  groups.
- **B8 passes on all 15 worlds.** GF-2's two failures (1.28× and 1.26×) are
  now 1.09× and 1.11×. The stage is the cause of the difference, since
  `N_h = 0` reproduces §5.6's counts exactly. **Why** it removes small lakes
  was not measured.
- **B9 passes, and the cost is real.** Seed 483920, 800 km, seven alternating
  runs per arm, run alone: control 3.300 s (3.233 … 3.326), treatment 3.651 s
  (3.597 … 3.667), ratio 1.106; an independent re-run: 3.268 s (3.194 …
  3.312) against 3.619 s (3.598 … 3.645), 1.107. The brackets do not overlap,
  so this is a measured cost of about 10 %, against GF-2's 1.014–1.038.
- **B10 passes:** deterministic on all five, 10–11 rock types, two-layer
  share 0.143–0.280.

**Why B1, B2 and B4 do not move: the stage can only lower slopes, and at
800 km there are almost none to lower.** A threshold pass removes excess over
`θc`; it never steepens anything. GF-0 measured 0.13–0.37 % of land above 40°
at 800 km (§5.4), and the slopes these bars compare are far below every `θc`
in §2.3: cap-edge medians 0.76–1.13°, strong-rock medians about 0.4°. So the
stage binds on a few cells, and B1 and B2 did not change in the fourth decimal
between 0 and 32 passes. B4 asks cap edges to be *steeper* than the
substrate near them, which only a process that lowers the substrate faster
than the cap can make. That is stream power at a time scale the default
pass does not reach (§5.6), i.e. GF-7's clock. At 80 km the stage does bind
(14–18 % of land is above 40°, §5.4), and it raises B4 there (0.44–1.21 →
0.65–1.40) without reaching 2.0. No constant was tuned outside §4.3's `N_h`
procedure, and no bar was changed.

**Screenshots** (`_gf2relief_shot.tscn`, windowed, seeds 483920 and 314159,
800 km, 2048 × 1311, now with `--processes on`, which sets the switch through
`set_params` and aborts if the key is rejected). "Before" is HEAD `22ec647`'s
engine (GF-2 only), "after" is GF-3 at `N_h = 8`. 671 and 725 pixels change by
more than 8 levels of 255, out of 2 684 928. The changes lie along one river
line and a steep shoreline strip per seed. **Relief does not visibly track
rock more than it did.** Granite interiors and the sedimentary lowlands look
as they did under GF-2.

**Identity.** `WorldParams::defaults()` and the app-default world are
bit-identical to HEAD `22ec647`. A scratch test hashed `field`, `temperature`,
`rainfall`, `flow_discharge`, `river_mask`, `river_floor`, `stream_order`,
`resistance_field`, `volcanic_field` and all five column arrays, for
`params::defaults()` at 2048 × 1311, 800 km, and `WorldParams::defaults` at
512 × 328, on seeds 483920 and 314159. It ran in a `git archive` copy of HEAD
and in a copy of the working tree: 48 of 48 hashes matched. No golden moved
and none was re-recorded.

**Tests.**
- `cartalith-erosion/tests/gf3_threshold_hillslope.rs` (11 tests): the talus
  at 80 and 800 km against hand-worked literals; the refusal of a
  non-positive peak; granite holding what shale sheds, to literal heights;
  the shedding cell's own rock, both orientations; the cap and its contact,
  including a breach in the middle of the stage; regolith reading
  unconsolidated; the same angle at two extents; wrap at both edges; the
  column never written; 0.75ⁿ relaxation.
- Three `cartalith-engine` unit tests on `RockContext::threshold_hillslope`:
  off touches nothing; on relaxes by `0.75^8` and records talus as regolith;
  world maps wrap.
- `geology_gf2.rs`'s light-pass replay now replays the stage, at 800 km and at
  20 km, where it must move cells.

**Mutation testing** was done in a scratch copy, never the live tree. It used
Python exact-replace, a pattern that occurs exactly once, restore in
`finally`, and a hash check after. **20 of 20 mutants were killed.** Four
survived the first round and were killed after tests were added: the per-cell
index, the right-edge wrap, the engine gate and the engine's `world`
pass-through.

### 5.8 GF-7 findings: the geological clock, swept (measured 2026-09-27)

**What was built.** `cartalith_engine::geo_clock` (§4.12): `WorldParams::geo_age`
(τ, 1.0 at both boundaries), the response laws `f_linear`, `f_saturating`
(`k = 0.5`) and `mantle_thickness_m`, and `GeoClock`, which returns each
process's effective count with its floor recorded. `RockContext` carries the
clock, built with the same gate as GF-2 (`geology_model` **and**
`geology_processes`), so it scales nothing in the app, where the processes are
off. Every count call site reads it: the light pass and each `evolve_cycles`
cycle (`max(4, round(iters·0.6·τ))`), the sediment fill (`max(4,
round(iters·τ))`), GF-3's threshold hillslope (`max(1, round(8τ))`), glacial and
coastal (`max(1, round(N·f_sat(τ)))`). When the clock does not act (τ = 1, or
the gate off), each call site runs today's expression by control flow. The
`PARAMS` row is `geo.age` (group `erosion`, first in it, 0.25–4.00, step
0.05, unit ×, no reference control), with a `JS_PATHS` row, and a save without
the key reloads at 1.0 (§2.6).

**Choices where §4.12 left one, or could not be followed, each with its reason.**
- **The weathered mantle is not wired.** The law is built and pinned by test
  (1.3336 m at τ = 0.25, 2.6862 m at τ = 4, 2.0 m at τ = 1), but nothing writes
  it into `regolith`. §4.12 anchors it at `h(1) = h₁ = 2 m`, and that mantle does
  not exist in today's pipeline. Writing it at τ = 1 would move every
  processes-on world at the default age, which contradicts §4.12's "at 1.0
  every process takes today's expressions" and B12's τ = 1 identity. Writing
  it only at τ ≠ 1 would make τ = 0.95 and τ = 1 differ by a whole 1.9 m
  mantle. **That contradiction is §4.12's own, and it needs an owner or
  coordinator call.** The mantle also cannot move B1, B2 or B4: its maximum,
  2.69 m, is below `R_EXPOSE` (5 m), so it never changes the exposed rock on
  its own. `h₀ = 0.5 m` is **not yet verified at the source**: the search
  reached Heimsath et al.'s abstract, not the fitted coefficient.
- **Karst (§4.6) has no hook.** There is no dissolution pass before GF-5, and
  §4.12's karst term is τ inside GF-5's own budget expression, so GF-5 writes
  it with `GeoClock::age()`. A hook with no caller would be dead code.
- **Coastal is scaled now, though it reads no rock until GF-4.** How long a
  cliff has retreated does not depend on whether the pass reads rock. The pass
  is off in the app.
- **Rebound is not scaled separately** (§4.12): it follows each scaled call.
- **Non-finite τ** is refused (today's counts); `params::set` already rejects
  it, so it is reachable only from a hand-built `WorldParams`.

**Commands** (release, run alone):

```text
cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf7_clock_sweep
cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf7_b9_cost
```

The sweep's control is `params::defaults()` (processes off, where τ is inert).
Every τ is judged on the same pre-erosion rock map and population as
`gf2_arms`. The τ = 1 row reproduces §5.7's treatment figures exactly (for
example B3 8.84, 13.47, 3.52, 8.36, 13.99 and small lakes 50, 72, 24, 106, 54).

**At 800 km, 2048 × 1311, processes on** (B4 is edge ÷ substrate · top-decile
share; B8 is lake % · 1–3-cell lakes; ✗ marks a failed bar):

| Seed | τ | B1 ρ | B2 | B3 full | B4 | B8 |
|---|---|---|---|---|---|---|
| 483920 | control | −0.460 | 0.050 | 3.73 | 0.243 · 0.294 | 1.177 · 46 |
| | 0.5 | −0.469 ✗ | 0.050 ✗ | 5.82 ✗ | 0.339 · 0.338 ✗ | 1.197 · 18 |
| | 1 | −0.462 ✗ | 0.051 ✗ | 8.84 | 0.253 · 0.325 ✗ | 1.146 · 50 |
| | 2 | −0.467 ✗ | 0.050 ✗ | 20.31 | 0.060 · 0.344 ✗ | 1.219 · 119 ✗ |
| | 4 | −0.459 ✗ | 0.050 ✗ | 26.76 | 0.072 · 0.351 ✗ | 1.009 · 355 ✗ |
| 24601 | control | −0.316 | 0.048 | 5.94 | 0.035 · 0.286 | 2.232 · 65 |
| | 0.5 | −0.314 ✗ | 0.047 ✗ | 10.37 | 0.130 · 0.268 ✗ | 2.942 · 43 |
| | 1 | −0.314 ✗ | 0.048 ✗ | 13.47 | 0.050 · 0.306 ✗ | 2.531 · 72 |
| | 2 | −0.317 ✗ | 0.049 ✗ | 23.89 | 0.038 · 0.346 ✗ | 2.054 · 162 ✗ |
| | 4 | −0.318 ✗ | 0.049 ✗ | 28.48 | 0.044 · 0.362 ✗ | 1.926 · 352 ✗ |
| 71077345 | control | −0.392 | 0.049 | 1.22 | 0.213 · 0.254 | 1.419 · 26 |
| | 0.5 | −0.396 ✗ | 0.049 ✗ | 2.56 | 0.829 · 0.294 ✗ | 1.926 · 20 |
| | 1 | −0.390 ✗ | 0.049 ✗ | 3.52 | 0.119 · 0.286 ✗ | 1.585 · 24 |
| | 2 | −0.396 ✗ | 0.049 ✗ | 6.25 | 0.078 · 0.296 ✗ | 1.249 · 70 ✗ |
| | 4 | −0.401 ✗ | 0.051 ✗ | 12.32 | 0.075 · 0.299 ✗ | 0.622 · 175 ✗ |
| 12345 | control | −0.423 | 0.049 | 4.15 | 0.036 · 0.200 | 4.661 · 105 |
| | 0.5 | −0.432 ✗ | 0.049 ✗ | 5.26 ✗ | 0.156 · 0.205 ✗ | 5.028 · 87 |
| | 1 | −0.432 ✗ | 0.049 ✗ | 8.36 | 0.068 · 0.211 ✗ | 4.072 · 106 |
| | 2 | −0.432 ✗ | 0.048 ✗ | 17.35 | 0.044 · 0.238 ✗ | 5.220 · 201 ✗ |
| | 4 | −0.419 ✗ | 0.049 ✗ | 11.03 | 0.053 · 0.271 ✗ | 3.507 · 438 ✗ |
| 314159 | control | −0.427 | 0.039 | 5.94 | 0.156 · 0.251 | 9.347 · 81 |
| | 0.5 | −0.429 ✗ | 0.040 ✗ | 9.69 | 0.720 · 0.187 ✗ | 9.207 · 32 |
| | 1 | −0.427 ✗ | 0.040 ✗ | 13.99 | 0.166 · 0.213 ✗ | 9.307 · 54 |
| | 2 | −0.429 ✗ | 0.038 ✗ | 51.13 | 0.138 · 0.195 ✗ | 9.125 · 171 ✗ |
| | 4 | −0.426 ✗ | 0.038 ✗ | 73.39 | 0.093 · 0.213 ✗ | 9.005 · 498 ✗ |

Ocean cells on river paths: 0 on all 25 worlds. B10 passes on all 20
treatment worlds: each is byte-identical to a second run of itself (field,
river mask, regolith, contact), 10–11 rock types, two-layer share 0.143–0.280.

**B12.**
- **τ = 1 identity: holds** (the hash comparison below).
- **Channel incision** (pre-erosion minus final surface, mean over the
  control's channel cells, a population fixed across τ): **fails on all five**.
  It does not rise strictly with τ: 28.2 → 27.5 → 20.0 → 13.9 m on 483920, and
  on the others it peaks at τ = 1 or 2. See the reason below.
- **Glacial**, replayed alone on each control's final surface at the clock's
  counts (4, 8, 13, 18 passes): mean land lowering rises with τ on all five, and
  the τ = 2 → 4 ratio is 1.384 on every seed, under τ's own 2. That is 18/13
  exactly (arithmetic), so the kernel's lowering is linear in its pass count
  on these worlds: the saturation measured is the count law's, and the kernel
  adds none.
- **Coastal**, replayed the same way (2, 4, 6, 9 passes): land cells lost rise
  with τ on all five, and the τ = 2 → 4 ratio is 1.072–1.210.
- **Mantle**: not measurable, because it is not wired (above). The law's
  literals are asserted by unit test.

**B9 (cost; disclosed, gated only at τ = 1).** Seed 483920, five alternating
runs per arm in one process, run alone. Run 1: control 3.189 s (3.155 … 3.271);
τ 0.5 3.077 s (3.020 … 3.100), ratio 0.965; τ 1 3.454 s (3.423 … 3.555), 1.083;
τ 2 4.239 s (4.158 … 4.330), 1.329; τ 4 5.775 s (5.744 … 5.913), 1.811. An
independent re-run: control 3.123 s (3.111 … 3.161); ratios 0.969, 1.082,
1.323 and 1.824, each inside or at the edge of run 1's bracket. τ = 1 passes
B9's 1.20 bar. That is GF-3's cost, and the clock adds nothing at τ = 1.

**The verdict. No τ makes B1, B2 and B4 pass. B8 fails at every τ above 1.**
- B1's ρ moves by at most 0.013 and B2's ratio by at most 0.002 across τ = 0.5
  to 4 on any seed. Both fail on all 20 worlds.
- B4 fails on all 20 worlds and does not rise with τ. Its cap-edge ratio is
  largest at τ = 0.5 on all five seeds (0.339, 0.130, 0.829, 0.156, 0.720)
  and falls at τ ≥ 2. The top-decile share is 0.213–0.362 at τ = 4 and never
  reaches 0.40.
- B8 passes at τ = 0.5 and 1 on all five seeds. At τ = 2 the 1–3-cell lakes
  are 1.9–2.7× the control's (70 … 201), and at τ = 4 they are 4.2–7.7× (175 …
  498). That is the regression RV-1 fixed, and it is worse than GF-2's. **Why
  was not measured.**
- B3 still passes at τ ≥ 1 on all five, and fails its relative half at τ = 0.5
  on two seeds (483920, 12345).

So the recommended default stays **τ = 1**, and **`geology_processes` should
not ship on** on this evidence: no τ passes B1, B2 or B4 at 1, and B8 fails
everywhere above 1. Nothing was switched on.

**What limits it: the clock multiplies a metres-scale effect, and rebound
takes most of it back.** A scratch diagnostic replayed the light pass alone on
the pre-erosion world at the clock's counts (seeds 483920 and 314159):
- The rock stream-power kernel alone lowers land by a mean of 0.85, 1.50, 2.89
  and 5.45 m at 5, 9, 18 and 36 iterations on 483920, and by 1.18, 2.04, 3.84
  and 7.03 m on 314159. That is roughly linear in τ, as §4.12's linear
  response intends, and it is still metres against relief of hundreds of
  metres (§5.6).
- `isostatic_rebound` after it returns land to a mean lowering of 0.06, 0.11,
  0.23 and 0.45 m on 483920 (0.26 … 1.31 m on 314159), and it lifts the control's channel cells
  above their pre-erosion height on 483920 (−0.65 … −1.60 m). Measured, 78–93 %
  of the kernel's mean land lowering is returned.
- Channel depth in the final world (21 to 45 m over each world's own
  channels) is therefore set by what runs after the light pass, of which
  RV-1's carve is the step the clock does not touch (§4.12). The carve's own
  share was not isolated. The channels are re-traced on the lowered surface,
  and at τ = 4 only 53–58 % of the control's channel cells are still channels. That is why incision
  over a fixed channel set does not rise.

So more passes of the same kernel under zero uplift cannot reach the hundreds
of metres of differential relief B1, B2 and B4 read at 800 km. Nothing in
§4.12's procedure changes that without tuning a rate constant, which §4.12
forbids and which was not done. No bar was loosened.

**Screenshots** (`_gf2relief_shot.tscn`, windowed, 800 km, 2048 × 1311, seeds
483920 and 314159, `--processes on`, with a new `--age` flag that aborts if the
key is rejected or clamped). They were taken in a scratch Godot project that
loads a DLL built from a copy of this working tree, so the live `target/`
another lane was using was not touched. τ = 1 against τ = 4: 168 684 and
196 437 pixels move by more than 8 levels of 255, out of 2 684 928. Where the
changes are:
- along every channel line, which are darker and more hatched at τ = 4;
- small lakes, several of which disappear or shrink at τ = 4 (a
  lake west of the central inlet on 483920; lakes on the east of 314159);
- volcano cones and one fault ridge, whose flanks gain fine gully hatching;
- coastlines, in a thin band.

**Relief does not visibly track rock.** No scarp appears at a cap edge. No
weak-rock lowland opens up. Granite interiors and the sedimentary basins read
as they do at τ = 1.

**Identity.** A scratch test hashed `field`, `temperature`, `rainfall`,
`flow_discharge`, `river_mask`, `river_floor`, `stream_order`,
`resistance_field`, `volcanic_field` and all five column arrays, on seeds
483920 and 314159, for four arms:
- the app default (`params::defaults()`, 2048 × 1311, 800 km);
- the app default with the processes on (τ = 1);
- `WorldParams::defaults` at 512 × 328;
- the app default at 512 × 328 with the processes on and coastal, the
  sediment fill and two evolve cycles all on. That reaches every clocked call
  site.

It ran in a `git archive` copy of HEAD `ec4e078` and in a copy of the working
tree. **102 of 102 hashes matched.** In the same binary,
`geology_gf7.rs::with_the_processes_off_age_changes_nothing` asserts that τ =
0.25 and 4 equal τ = 1 bit for bit with the processes off. No golden moved and
none was re-recorded.

**The UI** (stage 06, `world_workspace.gd`). `geo.age` is the first row of the
stage's erosion group, as a `geological age ×` slider. Beneath it,
`_refresh_geo_age_row` draws a readout line from `WorldGen.geo_clock_readout`
(`erode_bridge.rs`), so the count laws have one copy. The line reads, for
example, "stream power 36 it · hillslope 32 passes · glacial 18 passes",
marks a floored count "(floor)", and lists glacial, coastal and sediment fill
only when their pass is on. With the processes off (the app) the row is
non-editable and dimmed at the World Structure override's 0.55. Its tooltip
and readout give the reason and the counts it would run. The weathered mantle
is left out of the readout and of §4.12's tooltip text, since no world has it.
**Not yet seen running:** the GDScript parse-checks clean, and the readout's
Rust side is unit-tested by literal string. But the row was not rendered in
the shell: the live project loads `target/debug`, which another lane was
building, and this lane did not replace that DLL. A windowed probe of the row
is still owed.

**Tests.**
- `geo_clock.rs` (6 unit tests): the saturating law, the mantle law, and each
  process's count and floor, against literals; τ = 1 and the gate being off
  both give today's counts; the constructor's clamp and its refusal of a
  non-finite τ; the readout's gate.
- One engine unit test: the clock reaches `RockContext::threshold_hillslope`
  (0.75³² at τ = 4, 0.75⁸ at τ = 1, untouched with the gate off).
- `cartalith-engine/tests/geology_gf7.rs` (4 tests):
  - with the processes off, τ changes nothing;
  - with them on, τ = 4 moves the world and land lowering rises with τ;
  - every clocked call site runs the clock's count. At τ = 0.25 the floors
    make different raw counts equal, so two worlds must be identical, while at
    τ = 1 each knob must be live;
  - a scaled age is deterministic.
- `params_mapping.rs`: a save without `geo.age` reloads at 1.0; the key round
  trips; it clamps to its range; it defaults to 1.0 at both boundaries.
- `erode_bridge.rs`: the readout's literal strings on the app's parameters.

**Mutation testing** ran in a scratch copy, never the live tree, with Python
exact replacement, a pattern that occurs exactly once, restore in `finally`,
and a hash check after. **31 of 31 mutants were killed.** They cover
every constant in `geo_clock.rs`, each response shape, each floor and its
flag, the clamp, the non-finite refusal, the gate, every engine call site, the
readout's clauses, reason and forced gate, and the loader's reset to 1.0. One
survived the first round and was killed after a test was added: the readout
gate dropping `geology_model`.

### 5.9 Why B1, B2 and B4 were revised for Ruling BM (2026-09-27)

No threshold was lowered. Each change is to *what is measured*, and each is
for a reason found in the code or in §5.4–§5.8, not in a wish to pass.

**B1 and B2 measured tectonic setting, not rock.**
- Evidence: the control arm reads B1 −0.32 to −0.46 at 800 km on every seed
  (§5.5, §5.6), with no rock-reading process at all. Granite is 0.57–0.75 of
  land (§5.5) and, by §2.4, the old interior; `compute_height` damps the old
  interior's roughness to 0.258 of a margin's (§4.14, arithmetic). So the
  global ρ is dominated by margin distance.
- That is not a defect of the world. In nature relief is set by uplift first:
  steady-state relief scales as `(U/K)^(1/n)` (Whipple & Tucker 1999), and
  erosion rate tracks relief in active ranges (Montgomery & Brandon 2002). A
  strong, slowly uplifting shield is low; a bar that demanded a positive
  global ρ would demand rugged cratons, which is wrong. The strength signal
  lives at matched uplift, which is what the band stratification reads.
  §5.4's B6 fix was the same move (at matched ice discharge).
- The weak class was almost all volcano flank. §5.5: "Weak rock here is
  mostly tuff on volcano flanks", and §5.6: pre-erosion exposed shale is 0.
  So B2's control ratio, 0.039–0.050 (§5.6), compared granite interiors with
  stamped cones, whose steepness is the stamper's. Excluding edifices, and
  reading `s` where the rock-blind budget has exposed the substrate, puts
  shale and breached caps into the weak group, which is what B2 is about.
- The control changes because uplift is now a process. Rock and uplift are
  correlated by construction in §2.4 (gneiss and schist are collision-belt
  rocks, `U` peaks at convergence), so only a rock-blind arm under the same
  `U` isolates the rock term.

**B4 measured the wrong edge.**
- Evidence, from the harness (`b4` in `gf0_geology_harness.rs`): an edge is a
  two-layer cell whose exposed rock differs from **any** 4-neighbour's. With
  GF-1's derivation the covers lie in structural lows
  (`build_geology`, step 3), so such a cell is, by the derivation, mostly a
  sandstone or limestone cover cell beside the granite around its basin, not
  a cap beside its own exposed substrate. (Read from the code, not measured;
  GF-10 prints the split.) The substrate population near edges is the
  0.12–0.21 % of land where a cap is breached (§5.6).
- A breach line is the scarp the scope meant (§2.5: "once the cap is
  breached, the substrate is exposed ... the cap edge keeps a steep face").
  The revised definition reads exactly that.
- The share half is restricted by an input quantity (cap thickness) and a
  control-arm quantity (the rock-blind p90), never by the treatment's own
  slope.
- The legacy twin could not be measured on any of 15 worlds, by design
  (§5.5); its replacement is measurable wherever there is any cover.

**What is not revised.** B3 (it passes, and §5.5 already recorded that its
absolute half is not evidence), B5–B7, B9's 1.20 at τ = 1, B10 and B11.

### 5.10 GF-10 findings: the BM prototype, measured (2026-09-27) — **NO-GO**

**Verdict: NO-GO. GF-11 and GF-12 should not be built as scoped.** The
pre-registered rule (§7 GF-10) needs arm C to beat arm B by half of each
relative margin on B1, B2 and B4's twin on at least 3 of 5 seeds, with B8
holding. At 800 km and the pre-registered setting, **0 of 5 seeds** meet it:
- B1's margin is met on 0 of 5 seeds;
- B4's twin margin is met on 0 of 5;
- B2's margin is met on 5 of 5;
- B8 fails for arm C on 4 of 5 seeds.

No other setting rescues it either. No setting of the grid meets B1's margin
on any seed (the table under *Every setting*). So there is no "another
setting" decision to put to anyone (§7). The scope goes back to the owner
with the numbers below.
- The rule was written into the harness as constants before the first run
  (`GO_B1_DRHO` 0.075, `GO_B2_RATIO` 1.125, `GO_B4_TWIN_RATIO` 1.1,
  `GO_SEEDS` 3). No pre-registered value was changed after a result was seen.
  No bar was loosened.
- B8's part of the rule was read as its own §5.2 condition: all five seeds.

**What was built.** Harness code only: a new "GF-10" section at the end of
`crates/cartalith-godot/tests/gf0_geology_harness.rs`. No production file
changed. The pieces:
- `uplift_shape`: §4.15's field, with Ruling BN's rift term.
- `construct`: §4.14's construction, both breach rules, with Ruling BN's
  ocean-seeded fill.
- `bm_stage`: §4.15's stage, emulated with the existing
  `stream_power_kernel_rock`.
- `solve_u0`: the mean-balance rule.
- `tail`: a replay of the rest of `generate_terrain_inner`, so each arm is a
  whole world with rivers, lakes and the glacial pass.
- The revised B1, B2 and B4, as `b1_rev`, `b2_rev` and `b4_rev`.

**How the prototype differs from §4.14–§4.15**, each difference stated in its
doc comment:
- **`dt` is emulated by scaling `k` and the uplift by `dt`, not by the step
  count.** The kernel's update `(z + dt·u + c·z_r)/(1 + c)`, with `c = dt·C`,
  is then exact at the scope's `dt` (arithmetic).
- **Pinning is restore-after-call.** The rock entry point takes no mask.
- **Subsidence is applied before each call.** This equals a signed `u` in the
  kernel's formula, except that the step's routing is computed on the
  subsided surface.
- **Regolith is accounted on the net change less `U·T`** (the §4.15
  correction above).
- **`T₁`'s channel-head area is `river_flow_thresh / river_density`**, the
  slope-free initiation threshold, read as cells. At 800 km and 2048 × 1311
  it is 1 074 cells, so `T₁` = 7.629.
- **"BM on" includes GF-3's hillslope**, which reads θc, so arm B is
  rock-blind in construction and in the stage but not in the hillslope.

**The replay is `generate_terrain`.** `replica_reproduces_generate_terrain`
is a fast test at 192 × 123: the replay (priming climate, light pass, tail)
equals `generate_terrain` bit for bit, on the app path and with the processes
on at τ = 1 and 4. `gf10_small_lake_diag` checks the same at 2048 × 1311 on
all five seeds: 15 of 15 as-built worlds read *replica == generate_terrain*.

**Commands** (release, run alone, in a `git archive` copy of HEAD `7ac41a4`
plus the harness):

```text
cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf10_bm_prototype
GF10_EXTENTS=80,8000 cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf10_bm_prototype
cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf10_small_lake_diag
cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact gf10_b9_cost
```

The 800 km run was made twice, as separate processes. Every ROW line (every
setting, every seed) is identical between the two runs.

**The pre-registered constants, per seed (800 km).** `D₁` is the median cap
thickness; `U₀` is solved on arm B at τ = 1, with a residual under 0.01 m in 3
evaluations on every seed:

| Seed | `D₁` (m) | `U₀` (m per unit time) | two-layer land cells | breached at `c = 0.5` |
|---|---|---|---|---|
| 483920 | 150.00 | 7.665 | 232 851 | 41.2 % |
| 24601 | 80.91 | 8.414 | 307 021 | 54.1 % |
| 71077345 | 104.15 | 6.112 | 335 413 | 59.6 % |
| 12345 | 150.00 | 7.216 | 329 112 | 37.7 % |
| 314159 | 150.00 | 10.242 | 370 211 | 31.7 % |

**The cap-thickness distribution explains `D₁`.**
- Rift caps are all exactly `RIFT_CAP_M`, 150 m, and they are 28–63 % of
  two-layer land. So on three seeds the median is 150 m, and the rule gives
  the rift cap's own thickness.
- Cover caps are thin: medians of 35–72 m, p95 of 82–219 m.
- Volcanic caps are thick: medians of 760–2 603 m.
- The consequence: "the median cap" is mostly one constant, not a property of
  the world's covers.

**Arm by arm at the pre-registered setting** (800 km, 2048 × 1311, step rule,
τ = 1; B → C, with the margin in brackets):

| Seed | B1 rev (Δ ≥ 0.075) | B2 rev (C/B ≥ 1.125) | B4 twin (C/B ≥ 1.1) | B4 ratio · share, C | B8 small lakes C / A | B15 clamped, C |
|---|---|---|---|---|---|---|
| 483920 | −0.031 → −0.058 (−0.027) ✗ | 1.165 → 1.818 (1.560) ✓ | 0.077 → 0.040 (0.513) ✗ | 0.802 · 0.841 | 105 / 46 ✗ | 2 ✗ |
| 24601 | −0.006 → −0.060 (−0.054) ✗ | 1.165 → 1.606 (1.379) ✓ | 0.091 → 0.067 (0.735) ✗ | 0.868 · 0.748 | 223 / 65 ✗ | 31 ✗ |
| 71077345 | 0.056 → 0.041 (−0.015) ✗ | 1.260 → 1.692 (1.343) ✓ | 0.068 → 0.050 (0.726) ✗ | 2.078 · 0.566 | 52 / 26 ✗ | 36 ✗ |
| 12345 | 0.077 → −0.035 (−0.112) ✗ | 1.279 → 1.615 (1.263) ✓ | 0.106 → 0.073 (0.694) ✗ | 1.049 · 0.467 | 348 / 105 ✗ | 72 ✗ |
| 314159 | −0.010 → −0.070 (−0.059) ✗ | 1.275 → 2.779 (2.180) ✓ | 0.152 → 0.069 (0.452) ✗ | 0.764 · 0.638 | 69 / 81 ✓ | 59 ✗ |

Against the full §5.2 bars, arm C:
- **passes B2 on all five seeds** (≥ 1.5 and ≥ 1.25 × B);
- fails B1 on all five;
- fails the B4 twin on all five;
- passes B4's ratio and share on one seed (71077345);
- fails B15 on all five.

On B8:
- **ocean cells on river paths are 0 on every arm and every world**, the
  moving coast included;
- the failures are the 1–3-cell lakes, and on some seeds the lake share of
  river cells.

**Ruling BN's coast, measured.**
- At `c = 0.5`, construction took 0.24–2.26 % of land below sea as
  ocean-connected bays. The old 1 m floor would have caught about the same
  cells (for example 28 400 cells against 28 571 on 483920).
- No construction on any world left a new closed depression deeper than
  0.004: B14 PASS on every arm printed.
- The contact was bit-identical after construction on every arm.

**The rift term, measured.** 1.2–1.6 % of land subsides (21 416–32 132 cells
per seed). The stage's basin fill raised 823–1 533 cells per arm at the
pre-registered setting.

**Every setting** (800 km; seeds meeting each go/no-go margin, out of 5; B8
for arm C):

| Setting | B1 margin | B2 margin | twin margin | B4 bar (C) | B8 (C) |
|---|---|---|---|---|---|
| pre-registered (`D₁`, `T₁`) | 0 | 5 | 0 | 1 | 1 |
| `D₁`×0.5, `T₁` | 0 | 2 | 0 | 0 | 1 |
| `D₁`×0.5, 4`T₁` | 0 | 0 | 1 | 0 | 0 |
| `D₁`, 4`T₁` | 0 | 1 | 0 | 0 | 0 |
| `D₁`×2, `T₁` | 0 | 3 | 0 | 0 | 0 |
| `D₁`×2, 4`T₁` | 0 | 0 | 0 | 0 | 0 |
| continuous breach rule | 0 | 5 | 0 | 3 | 1 |
| construction alone | 0 | 5 | 0 | 3 | 4 |
| stage alone | 0 | not measurable | 0 | 0 | 1 |

About B1 across the grid:
- ΔB1 is under 0.075 on all 45 setting-seed pairs. It is negative on 40 of
  them, and the largest value is +0.029 (construction alone, 483920).
- Stage alone has no weak class for B2 (without construction nothing but
  edifices is weak, and edifices are excluded).
- Its B4 twin is 0.98–1.01 × B: the stage alone does not read rock at the
  scale of a cap.

**Where the effect lives: construction makes scarps, and the stage takes
them away.**
- Construction alone gives a breach-line to substrate slope ratio of 3.68,
  7.27, 11.93, 8.17 and 1.50 in arm C. The B4 bar passes on 3 of 5 seeds,
  and B8 holds on 4 of 5 (small lakes 15, 51, 11, 108 and 10, against 46,
  65, 26, 105 and 81).
- Adding the stage takes the ratio to 0.76–2.08 and fails B8 on 4 of 5.
- The continuous breach rule, which §4.14 expected to make no scarp,
  measures 11.7–15.4 on three seeds under the full model. That is the only
  full-model setting where B4's ratio passes on three seeds; its B1 and twin
  fail as the step rule's do.

**τ sweep (arm C, B8, 800 km).** At τ = 0.5 B8 passes on 3 of 5 seeds; at
τ = 2 and 4 it fails on all five:

| τ | small lakes, C (A) per seed |
|---|---|
| 0.5 | 25 (46), 82 (65), 26 (26), 198 (105), 22 (81) |
| 2 | 520, 574, 226, 723, 641 |
| 4 | 1 721, 2 117, 1 086, 1 757, 3 083 |

**80 and 8 000 km** (the pre-registered setting, reported as §5.2 asks; the
rule is not applied at these extents):
- **80 km:** ΔB1 is +0.016 to +0.045, the only extent where it is positive
  on every seed, still under 0.075. B2 is 1.05–1.22 × B, and the twin is
  0.997–1.244 × B. B4's ratio is 1.20–1.78, with a share of 0.18–0.26. B8
  fails on 4 of 5 (small lakes 373/284, 830/621, 235/312, 434/238 and
  734/519).
- **8 000 km:** ΔB1 is negative on all five. B4's ratio is 0.53–1.33, with
  no pass, as §4.14 expected at this cell size. B8 fails on all five (small
  lakes 126/38, 271/57, 173/19, 133/35 and 297/62).

**What limits it.** These are readings of the numbers above; the causes
marked *hypothesis* were not isolated.
1. **The stage fails B8 and B15 on its own.** Stage alone raises the small
   lakes on 4 of 5 seeds, and the lakes grow steeply with model time. It
   clamps 2–72 cells at 1.0 at τ = 1, and 2–287 at τ = 4.
   - *Hypothesis:* each step routes once and then moves the surface by
     `dt·U`, so within a step the tree is frozen, the same mechanism the
     diagnosis below confirms for the light pass. Eight steps per `T` means a
     longer `T` freezes a larger move.
   - Refreshing more often costs more steps, and B9 is already failed.
2. **B1 does not see rock at matched uplift.** Granite is more than
   two-thirds of the B1 population on every seed, so the tertiles tie at
   0.85: "strong" is granite and "weak" is everything else.
   - C − B is negative on every seed.
   - *Hypothesis:* construction puts shale in scarp-foot positions next to
     caps, where a 9 × 9 relief window straddles the step, so weak rock gains
     window relief.
3. **B4's twin measures geography.** The thin-cap quartile is cover sitting
   in structural lows. The thick quartile is 150 m rift caps along rift
   troughs, plus volcanic caps. So the twin reads 0.04–0.15 in both arms,
   and rock makes the thin covers flatter still.
   - The not-pre-registered diagnostic twin moves the same way (C/B
     0.47–0.74 at the pre-registered setting). It leaves edifices out, and
     cuts strictly when the quartiles tie.
4. **Cost.** B9 fails by far:

   | Piece, median of 5 (min..max) | Run 1 | Run 2 |
   |---|---|---|
   | app generation | 3.244 s (3.224..3.955) | 4.391 s (3.857..11.806) |
   | light pass | 1.130 s | 1.629 s |
   | construction | 1.030 s | 1.338 s |
   | BM stage, 8 steps | 5.338 s (5.193..5.455) | 7.732 s (6.091..8.119) |

   Projected BM generation is **2.648 ×** the app in run 1 and **2.726 ×** in
   run 2, against B9's ≤ 1.20. The two runs' absolute brackets are disjoint,
   because the machine was busier in run 2, so no absolute time is claimed.
   Both ratios exceed the bar by more than 2 ×.

**B13 and B14** (fast tests, part of the ordinary suite):
- `b13_the_prototype_stage_reaches_the_stream_power_steady_state`: the
  granite/shale slope ratio at matched area is 2.887 within 5 % at
  `c = 0.5`, and 1 within 5 % at `c = 0`.
- `b14_construction_keeps_the_column_and_makes_no_inland_pits`:
  - `contact` is bit-identical;
  - at `c = 0` the unfilled surface is `z0 − m_to_norm(B)` bit for bit, and
    at `c = 0.5` it is not;
  - no new depression;
  - regolith rises by exactly the raise;
  - the breach lowerings match literals: 53.666, 94.868 and 79.510 m;
  - no land cell goes below sea without joining the ocean, and an inland
    breach below sea is refilled.

So the prototype stage reaches the steady state the theory predicts on a
fixture. The failure on real worlds is not a failure of the kernel's steady
state.

**Mutation testing** (scratch copy only; Python exact replacement, restore in
`finally`, hash checked):
- **10 of 11 mutants were killed.** Among them: dropping κ, the uplift or `dt`
  from the stage; the breach branch; the continuous rule; the fill; the
  regolith write; ocean seeding (the old "every below-sea cell" rule); a
  contact write; and the tail's glacial strip.
- The breach-branch mutant survived the first round. It was killed after the
  literal lowerings were added to B14.
- **One survivor:** dropping the ocean currents from the tail's post-carve
  climate is invisible at 192 × 123. The final `refresh_climate` overwrites
  climate, and only the glacial pass reads that temperature. The full-size
  replica check (15 of 15) covers it.

**The τ > 1 small-lake failure of §5.8: found — the frozen routing.**
`gf10_small_lake_diag` counts 1–3-cell lakes stage by stage on the GF-7
worlds. Per seed, 483920 / 24601 / 71077345 / 12345 / 314159:

| Case | final small lakes |
|---|---|
| app world (A) | 46 / 65 / 26 / 105 / 81 |
| τ = 4 as built | 355 / 352 / 175 / 438 / 498 |
| τ = 4, routing refreshed each of its 36 iterations | 62 / 99 / 43 / 53 / 88 |
| τ = 4 with no regolith accounting | 355 / 353 / 175 / 438 / 498 |

- **The excess appears in the stream-power call itself.** Right after the
  call, τ = 4 has 473 / 598 / 251 / 569 / 608 small lakes, against A's 67 /
  83 / 44 / 118 / 106. Rebound and the hillslope barely change the count.
- **The carve lowers the count** (pre-carve 453 → final 355 on 483920), so it
  is not the cause.
- **Regolith feedback is ruled out.** With no regolith accounting the counts
  are identical to as-built, bar one lake on one seed.
- **Refreshing the routing removes 88–98 % of the τ = 4 excess** over A on
  four seeds, and all of it on 12345 (53 against A's 105). It does not
  remove all of it: 3 of 5 seeds stay above 1.25 × A (62 against 57.5, 99
  against 81.3, 43 against 32.5).

So the §4.15 hypothesis holds for the light pass, and the per-step refresh
that §4.15 designs in is necessary but, on this evidence, not sufficient.

**Disclosed, in the order it happened.**
- **A 512-wide smoke run** on one seed, before the recorded run, found two
  harness problems:
  - B3 over the edifice-excluded population had no weak rock. B3 is
    unrevised, so it was moved to its own form (all interior land,
    pre-erosion map).
  - The scope's twin was unmeasurable at 512, because the quartiles tied at
    150 m. The diagnostic twin was added, labelled not pre-registered and
    kept out of the rule. At 2048 the scope's twin is measurable on every
    seed.
  - No pre-registered constant changed.
- **B15's first recorded count included cells already at 1.0 on the stage's
  input.** Those are stamped summits, 646–3 224 per seed. The definition was
  corrected to "below 1.0 on input, held at 1.0 by the stage", and the
  800 km run was made again. Every ROW line was identical; only the B15
  column changed.
- **The B14 fixture's inland patch was moved below sea**, and the literal
  lowerings added, after the first mutation round. Both changes were made so
  that Ruling BN's seeding and the breach branch are pinned.

**The images** (`GF10_DUMP`; hillshade, NW light, ×10 vertical exaggeration at
full frame and ×5 on a 512 × 512 crop at the densest breach lines; arms A, B,
C on seeds 483920 and 314159; scratch output, not committed). They were
opened and looked at:
- **B and C** are more finely gullied than A. On 314159, C has new
  elongated lakes north of centre that A lacks.
- **In C**, the rift troughs gain thin parallel step lines along both flanks.
  They are one cell wide, and they are the breach lines. B has none of them.
  So rock does control where steps sit.
- **No plateau stands behind a scarp.** The steps have no bench-and-face
  relief at this scale.
- **No weak-rock lowland opens** that is visible against B.
- The coastline differs from A in a few places, as Ruling BN allows.

**Identity (GF-10 changes no production behaviour).**
- The change is one test file, plus this document and `STATUS.md`. No file
  under `crates/*/src` and no `Cargo.toml` or `Cargo.lock` was touched, so
  `WorldParams::defaults()` and the app's defaults generate what they
  generated at HEAD by construction.
- `cargo test --workspace --no-fail-fast` in the scratch copy passed before
  and after the change. The counts are in `STATUS.md`'s line.

### 5.11 Refreshing the stream-power routing: measured on today's app worlds (2026-09-28) — **not adopted; owner call**

The `OUTSTANDING_WORK.md` row "Refresh drainage routing inside the
stream-power call (the small-lake cause)". §5.10 found that the kernel routes
once per call and that a per-iteration refresh removes most of τ = 4's excess
small lakes. The question here: does refreshing change **today's app worlds**
(`cartalith_godot::params::defaults()`, `geology_processes` off, a 9-iteration
light pass)?

**Answer: yes, on every seed and every interval below 9.** So it is a
re-baseline and is **not adopted**. Nothing in the engine calls it.

**What was built** (no production caller):
- `cartalith_erosion::stream_power_kernel_refreshed` and
  `stream_power_kernel_rock_refreshed`. They rebuild the fill, receivers,
  drainage area and `Cc` from the current surface before iterations `k`,
  `2k`, and so on. `k` is a `NonZeroUsize`.
- The ported setup moved, statement for statement, into
  `stream_power_routing`. `stream_power_core` calls it once, then again
  only when a refresh interval is given. Every existing entry point passes
  `None`, so the legacy and GF-2 kernels are unchanged by control flow.
- Harness section "§5.11" in `crates/cartalith-godot/tests/gf0_geology_harness.rs`:
  `light_pass_rr`, `rr_row`, the ignored `rr_routing_refresh` and
  `rr_routing_refresh_cost`.

**Commands** (release, alone, in a `git archive` copy of HEAD `f607262` plus
the change):

```text
cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact rr_routing_refresh
RR_TAUS=1 RR_EVERY=1 cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact rr_routing_refresh
cargo test --release -p cartalith-godot --test gf0_geology_harness -- --ignored --nocapture --test-threads=1 --exact rr_routing_refresh_cost   # twice, separate processes
```

Each as-built row was checked against `generate_terrain` in the same run: "replica ==
generate_terrain" on all 5 seeds, app path and gated path. The second run
reproduced every "refresh every 1" app row of the first exactly.

**How each measure is defined:**
- *Small lakes*, *lake % on path* and *ocean on path* are B8's (§5.2).
- *3+ pieces* is a **traced-cell proxy**: a river's maximal dry runs of at
  least 2 traced points, counted over every run `river_entities` traces. It
  is not `_riverzoom_probe`'s stroke count, which cuts on the render curve,
  and the two must not be compared.
- *Relief* is B1's 9 × 9 window and *slope* is the Sample panel's grade.
  Both are taken over interior land, with the median and p90 shown.

**App path, 800 km, 2048 × 1311, light pass 9 iterations.** Today → refresh
every 1 / 2 / 3 / 5. Refresh every 9 is today's world bit for bit on all five
seeds.

| Seed | small lakes | lake % on path | 3+ pieces (proxy) | relief m, median | slope °, p90 |
|---|---|---|---|---|---|
| 483920 | 46 → 65 / 58 / 64 / 85 | 1.18 → 0.98 / 1.33 / 1.26 / 1.11 | 1/1120 → 2/1297 / 1/1254 / 1/1145 / 1/1071 | 30.96 → 29.81 / 29.86 / 30.76 / 30.79 | 3.83 → 3.72 / 3.76 / 3.76 / 3.78 |
| 24601 | 65 → 50 / 64 / 86 / 80 | 2.23 → 3.54 / 3.36 / 2.56 / 2.63 | 1/1298 → 5/1577 / 3/1436 / 5/1312 / 6/1289 | 31.24 → 30.56 / 30.51 / 31.25 / 31.30 | 2.64 → 2.59 / 2.58 / 2.66 / 2.66 |
| 71077345 | 26 → 18 / 21 / 32 / 31 | 1.42 → 0.72 / 1.85 / 1.01 / 1.29 | 2/1044 → 3/1216 / 2/1160 / 3/1049 / 1/1061 | 29.82 → 28.96 / 29.33 / 29.35 / 29.47 | 2.84 → 2.74 / 2.78 / 2.78 / 2.80 |
| 12345 | 105 → 107 / 97 / 80 / 93 | 4.66 → 5.41 / 4.60 / 4.80 / 3.80 | 7/952 → 6/1176 / 6/1123 / 5/1015 / 6/1002 | 37.28 → 35.80 / 35.95 / 37.01 / 37.22 | 5.70 → 5.63 / 5.62 / 5.71 / 5.73 |
| 314159 | 81 → 60 / 83 / 100 / 88 | 9.35 → 9.63 / 9.75 / 9.07 / 9.99 | 2/1358 → 3/1508 / 5/1481 / 1/1334 / 2/1308 | 36.42 → 35.50 / 35.38 / 36.22 / 36.65 | 5.02 → 4.87 / 4.87 / 4.95 / 4.93 |

What the table shows:
- **Ocean on path is 0 on every row.**
- **How much moves:** at every interval below 9, 96.8–99.1 % of field cells
  change. The largest single-cell difference is 570–1 524 m.
- **More rivers are traced**, for example 1120 → 1297 on 483920 at every-1.
- **Relief and slope fall a little.** At every-1 the median relief drops
  0.7–1.5 m, and p90 slope drops 0.05–0.15°.
- **There is no small-lake benefit at 9 iterations.** At every-1 the count
  falls on 3 seeds and rises on 2. Every-3 and every-5 each raise it on 4
  of 5 seeds.
- Lake share on the path moves either way.

**The gated path (`geology_processes` on).** Small lakes, as built → every 1 /
every 4:

| τ (iterations) | 483920 | 24601 | 71077345 | 12345 | 314159 |
|---|---|---|---|---|---|
| 4 (36) | 355 → 62 / 127 | 352 → 99 / 200 | 175 → 43 / 84 | 438 → 53 / 90 | 498 → 88 / 162 |
| 1 (9) | 50 → 71 / 69 | 72 → 67 / 84 | 24 → 30 / 33 | 106 → 88 / 75 | 54 → 51 / 65 |

- **At τ = 4, the in-kernel refresh reproduces §5.10's emulation exactly:**
  62 / 99 / 43 / 53 / 88. Over the app world's counts, every-1 removes
  88–98 % of the τ = 4 excess on four seeds and all of it on 12345.
  Every-4 removes 53–81 % on four seeds and all of it on 12345.
- **At τ = 1 the count moves either way,** as on the app path. So the refresh
  pays only on long runs.
- At τ = 4 every-1 also lowers median relief by 1.5–2.8 m, and raises the
  proxy 3+ pieces count on all 5 seeds (by 2–4).

**Cost** (seed 483920, 5 runs after one warm-up, two separate processes, run
alone, median with min..max). Nothing else in the pipeline changes cost, so
the projection is the app's generation plus the difference in the call.

| | Run 1 | Run 2 |
|---|---|---|
| `generate_terrain` (app) | 3.251 s (3.219..3.302) | 3.191 s (3.153..3.228) |
| light-pass SP call, frozen | 1.081 s (1.067..1.102) | 1.017 s (1.011..1.034) |
| every 1 | 5.206 s → projected 2.27 × | 5.167 s → 2.30 × |
| every 2 | 3.089 s → 1.62 × | 3.067 s → 1.64 × |
| every 3 | 2.058 s → 1.30 × | 2.054 s → 1.33 × |
| every 5 | 1.522 s → 1.14 × | 1.526 s → 1.16 × |

Each refresh adds about 0.5 s at 2048 × 1311: (5.206 − 1.081) / 8
refreshes and (3.089 − 1.081) / 4 in run 1.

**Recommendation.**
1. **App: do not adopt.** It re-baselines every app world, with no
   small-lake benefit at the app's 9 iterations, and costs 1.14–2.30 ×
   generation.
2. **Gated path: not wired either.** §5.8 and §5.10 leave the
   `geology_processes` path off and GF-11/GF-12 at NO-GO. At its only
   shipped age (τ = 1), the refresh measures as noise. The owner may want it
   later, if a long-run path (τ > 1, or §4.15's stage) is built. Then the
   evidence favours refreshing every iteration **only when the clock
   lengthens the run**:
   - It removes 88–98 % of τ = 4's excess small lakes on four seeds, and
     all of it on 12345.
   - Its cost is about 6.5 × the frozen call at 36 iterations (27.6 s
     against 4.3 s on 483920). That is one sample inside the measurement
     run, not a timing.
3. **No golden moved.** App, app-with-processes and `WorldParams::defaults`
   worlds hash identically before and after the change (below).

**Identity.**
- `cartalith-engine` is untouched. Every existing kernel entry point passes
  `refresh_every = None`.
- `golden_parity_streampower` passes. Its two cases fail under the mutant
  that makes the frozen entry refresh, which proves they can see the
  difference.
- Whole-world hashes were compared, HEAD `f607262` against the change, in
  two `git archive` copies with separate target dirs, release. The hash
  covers field, rainfall, temperature, discharge and river mask.
  **15 of 15 were identical:**
  - 5 seeds × app defaults at 2048 × 1311;
  - 5 seeds × app defaults with `geology_processes` on;
  - 5 seeds × `WorldParams::defaults(512, 328, seed)`.
  - A first attempt's comparison was vacuous: the hash test did not compile,
    so both outputs were empty. It was caught by counting lines and redone.

**Tests** (`crates/cartalith-erosion/tests/stream_power_refresh.rs`, 5 fast
tests):
- The oracle: every-`k` over 12 iterations equals `12/k` back-to-back
  frozen calls of `k` iterations. This holds bit for bit, for `k` = 1–4
  and for the rock entry at `k` = 1 and 3.
- An interval ≥ the iteration count is the frozen kernel.
- The fixture is one the refresh changes.
- Literal pit counts at 36 iterations: 182 → 34 frozen, → 31 refreshed.
- Mutation testing in a scratch copy, 7 of 7 mutants killed:
  - the refresh never fires;
  - the wrong schedule;
  - the rebuilt routing shadowed rather than assigned;
  - the rock flag dropped on refresh;
  - each `_refreshed` entry passing `None`;
  - the frozen entry refreshing.

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

### 6.1 During the build (GF-1 to GF-8)

- The model ships behind a switch. It is **false** in
  `WorldParams::defaults` and **true** in `cartalith_godot::params::defaults`.
  That is `MISTAKES.md`'s standing convention for a change to generated output.
- No JS-parity golden moves mid-build, and GF-0's two arms run on one binary.
- The switch gets **no GUI control**. There is no user-facing legacy mode
  (§7p).
- **The clock (GF-7) needs no switch.** At τ = 1.0, its default at both
  boundaries, every process takes today's expressions by control flow, so no
  golden can move. B12 asserts it.
- **Painting (GF-8) cannot move a generated world**: it acts only on a user's
  commit.
- **The coastal default (GF-4)** is an app-boundary divergence of the Ruling
  AU kind (`p.passes.coastal = true` in `params::defaults()` only).
  `params_mapping.rs`'s
  `exactly_the_ruled_divergences_ship_at_the_app_boundary` gains it as the
  seventh divergence in the same change.

### 6.2 The final disposition (GF-9); recommended route A (§9 Q6)

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
  assertion. GF-9 opens each candidate and classifies it:
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

### 6.3 Ruling BM's re-baseline (GF-10 to GF-13, then GF-9)

Ruling BM: *"They re-baseline app worlds when switched on, with disclosure."*

- **During GF-10 to GF-12 nothing moves.** `relief_model` is off at both
  boundaries; GF-10 adds only harness code; GF-11 and GF-12 are control-flow
  gated. Each asserts, by hash, that `WorldParams::defaults()` and the
  app-default world are bit-identical to the HEAD it started from (the
  §5.6–§5.8 practice: `field`, climate, flow, rivers, stream order,
  resistance, volcanic field and every column array, two seeds, in a
  worktree built from HEAD).
- **The kernel goldens stay bit-identical by construction.** The stage's
  new inputs (an uplift slice, `dt`, a pinned mask on the rock entry point,
  per-step routing) are `Option`s or new entry points; `stream_power_kernel`
  and `stream_power_kernel_bounded` are untouched, as GF-2 left them.
- **GF-13 is the switch-on**, and it is where app worlds move. It turns
  `geology_processes` and `relief_model` on in
  `cartalith_godot::params::defaults()` only if every bar of §7's GF-13 entry
  passes. It records old → new for every figure other documents quote from an
  app world, the same list §6.2 gives (RV-1's `_riverzoom_probe` numbers, the
  LOD-D4 bars, `PERFORMANCE_BENCHMARKS.md`'s generation times,
  `GENERATION_PARAMETERS.md`'s divergence costs), and it names the change
  in `GENERATION_PARAMETERS.md`'s deliberate-divergence list.
  `params_mapping.rs`'s `exactly_the_ruled_divergences_ship_at_the_app_boundary`
  gains the two flags in the same change.
- **A regenerate warns.** An existing saved world does not change on open
  (its parameters are saved), but regenerating it with the new defaults
  does. The shell already confirms before a regenerate that discards
  authored work (`_authored_inventory()`); GF-13 checks that a save without
  `relief_model` reloads **off**, so an old world regenerates as it was
  generated unless the user asks otherwise.
- **The JS-parity suites are untouched** at GF-13 (they run
  `WorldParams::defaults`, where the switch stays off). GF-9 then does what
  §6.2 says, with BM's stages inside the one pipeline under route A, or
  behind the split under route B.

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
- The first run's numbers are recorded in §5.4 with the commands that
  produced them. `STATUS.md` records the milestone.

### GF-1 — the lithology model and column, read by no process

- §2.4's derivation.
- `volcanic_setting` stored.
- The column of §2.5 on `WorldState`, behind §6.1's switch.
- Save format: §2.6's `geology` set, its manifest member and its reader and
  writer obligations, with `SAVEFILE_COMPAT.md` gaining it as §8.4. An old
  save has no column. Every reader dashes it with the reason "saved before
  geology", and civ falls back to `build_lithology`. The backward-compatibility
  test is built from a real prior-format archive (§2.6).
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
- B6 passes. B7 passes: Ruling BJ turned the coastal pass on in the app
  (§4.5), in this milestone.
- `params::defaults()` sets `p.passes.coastal = true`. `params_mapping.rs`
  gains it as the seventh ruled divergence, and `GENERATION_PARAMETERS.md`
  discloses its measured cost as a median with min..max.
- B8 still passes, now with the coastal pass on.
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

### GF-7 — the geological clock (Ruling BJ)

§4.12.

- `WorldParams::geo_age` (τ), default 1.0 at both boundaries, with its
  `PARAMS` and `JS_PATHS` rows.
- The per-process responses: linear for stream power, the threshold
  hillslope and karst; saturating for coastal and glacial; logarithmic for
  the new weathered mantle.
- The floors, and §2.5's `R_EXPOSE`.
- The stage-06 row, `geological age ×1.00`, with its effective-count readout
  and tooltip. The `DCC_CONTROL_INDEX.md` row. The phone and tablet sheets.
- `h₀` verified at its source (Heimsath et al. 1997) before it is pinned.

**Done means:**
- B12 passes.
- At τ = 1, every golden is bit-identical and every app-default world is
  byte-identical to one generated without the parameter.
- The mantle's tests assert the literal values of §5.2's B12, never
  `h(τ)` against itself. Mutating `k`, `h₀` or `h₁` turns them red.
- The cost at τ ∈ {0.5, 2, 4} is disclosed in `GENERATION_PARAMETERS.md`.
- `--check-only` on every touched `.gd`, plus `shell/app.gd`.

### GF-8 — lithology painting (Ruling BJ)

§4.13, §2.6's `rock_painted.u8` row.

- The Rock target and its three layers in `PaintEditor`, and the tool-options
  row and right dock of §4.13.
- The commit: undo first, then bake, then the differential re-erosion over
  `W` through `tile_erode` (its first production caller), then the refresh.
- `HeightUndo`'s optional column snapshot, and the redo tail's.
- Persistence: the column rasters (GF-1), `rock_painted.u8`,
  `geology.painted`, and the three draft layers in `drafts/paint.json`.
- `DCC_SHELL_SPEC.md` §4.5.2's "soil and lithology have no override array"
  sentence, and the target list below it, are updated in the same change
  (`MISTAKES.md`: *"Change behaviour"*).
- `_authored_inventory()` lists painted rock.

**Done means:**
- B11 passes, including its mutation.
- A windowed probe paints shale into granite, commits, and shows the painted
  patch lower than its surroundings in the rendered relief. Its positive
  control must move pixels.
- Undo and redo are exact for the field and the column. The existing undo
  tests are unchanged.
- The save round trip is byte-identical for the column and the mask. The
  prior-format archive test of §2.6 still passes.
- `cargo test --workspace`, and `--check-only` on every touched `.gd` plus
  `shell/app.gd`.
- The DLL and `.rs` mtimes are stated at the end of the run.

### GF-9 — re-baseline, projection and scrub

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

*Ruling BJ moved lithology painting into this build as GF-8, above; §9 Q2
records the answer.*

**Build order after Ruling BM** (§9 Q18). The numbers are names, not an
order. The recommended order is **GF-10 → GF-11 → GF-12 → GF-13**, then
GF-4, GF-5, GF-6 and GF-8, then **GF-9 last**. BM comes first because every
later milestone's bars are measured on the relief model it settles; GF-8
comes after GF-12 because its re-erosion chain includes BM's stages (§4.13's
BM note).

### GF-10 — the BM prototype measurement (Ruling BM; measurement only)

The question it answers: **can the proposed terms move B1, B2 and B4 at
all**, before anything is built into the pipeline? It adds harness code only
(`gf0_geology_harness.rs`, `#[ignore]`d tests run `--release`, alone), and no
production code.

- **The prototype.** Test-side functions, over a generated app world with the
  processes off:
  - §4.15's uplift field from the stored `stress_field`, `boundary_mask` and
    `crust_field` (flexure by `compute_flexure`), with Ruling BN's rift term
    from `boundary_type`, and the stage's subsidence and basin fill;
  - §4.14's construction, **both** breach rules (§4.14's "judgement" note),
    with the fill-to-regolith step seeded from the ocean, so the coast may
    move (Ruling BN);
  - the uplift-driven stage approximated with the **existing**
    `stream_power_kernel_rock`: its uplift enters through the `stress` slice
    and `stream.uplift`, one call per step so the routing is refreshed each
    step, and `dt` emulated by the step count where the kernel's `dt = 1` is
    fixed. Pinning is emulated by restoring lake cells after each call. The
    prototype says which of these emulations differs from §4.15 and how.
- **Arms, on the same seeds and pre-erosion world:** A (today's app world),
  B (BM on, `c = 0`: rock-blind), C (BM on, `c = 0.5`). Construction alone,
  the stage alone, and both.
- **Settings:** `D₁` at 0.5, 1 and 2 × §4.14's pre-registered rule (the
  median cap thickness), and two model times (`T₁` by §4.15's rule, and
  4·`T₁`). Every setting is reported. GF-11 and GF-12 build the
  pre-registered rules; if GF-10 shows those cannot move the bars and another
  setting can, changing the rule is a coordinator or owner decision made on
  the printed numbers, and disclosed as such.
- **Readings:** revised B1, B2, B4 (with the twin), B3, B8 and B15's clamp
  count, on five seeds at 800 km, and B4 and B8 at 80 km; plus the
  cap-thickness distribution, the legacy-B4 edge split (cover-margin cells
  against breach-line cells), the `U₀` mean-balance solve per seed, and the
  share of land floored at sea.
- **The τ > 1 diagnosis** (§4.15, B8 item 1): §5.8's new small lakes at τ = 4,
  classified as frozen-tree pits, unconsolidated feedback or carve products.
- **Cost:** the prototype stage's time at 2048 × 1311 against the light pass,
  as a median with min..max over two independent runs, to size `N_BM`
  against B9.

**Done means:**
- A table of every reading above, per seed and arm, in a new §5.10, with the
  commands.
- **A go/no-go stated against a rule written before the run:** GF-11 and
  GF-12 proceed only if arm C beats arm B on the revised B1, B2 and B4 by at
  least half of each bar's relative margin (Δρ ≥ 0.075 for B1; ≥ 1.125 × for
  B2; ≥ 1.1 × on B4's twin) on at least three of five seeds at the
  pre-registered setting, with B8 holding there. If only another setting
  meets it, that is the decision the settings item above describes. If none
  does, the scope goes back to the owner with the numbers, and nothing is
  built.
- The τ > 1 small-lake cause is stated from measurement, or stated as not
  found.
- The positive and negative controls (§5.1) still pass. `STATUS.md` records
  the milestone.

### GF-11 — rock-aware construction (Ruling BM, part 1)

§4.14, behind `relief_model` (§4.16), off at both boundaries.

- The construction stage between the geology stage and the priming flow; the
  budget from §4.15's smoothed uplift; the breach rule GF-10 chose, and why.
- The fill-to-regolith step, seeded from the ocean so the coast may move
  and no inland pit forms (§9 Q14, Ruling BN); arc edifices exempt (§9 Q17).
- `WorldParams::relief_model` with its `PARAMS` and `JS_PATHS` rows; a save
  without the key reloads off.
- The storage decision §4.13's BM note leaves to it (the structural surface or
  the construction depth), with its memory measured by `_memlane_peak`.

*Design choices made while building it (2026-10-12), recorded here because
they are design, not status:*
- Under Ruling BY (construction alone, no GF-12) the light pass's stream
  power and rebound **do not run when construction has lowered the world**:
  that is the arm GF-11's rule measured, and the built stage reproduces it
  bit for bit. A construction that **skips** leaves them running, so a skip
  yields the processes-on world rather than one with no denudation at all.
- It **skips with a recorded reason** (`ConstructionSkip`) rather than
  inventing an input: no land; no two-layer land (`D₁` has no sample); or
  world-structure on (§4.15's orogeny term is not kept past the substrate,
  and GF-11 was measured without world-structure).
- The step breach rule only. The structural ocean the fill is seeded from is
  a copy of `build_water_bodies`' ocean rule inside `cartalith-erosion`
  (`structural_ocean`), because `cartalith-civ` depends on the engine; a
  harness test holds the two equal.

**Done means:**
- B14 passes, including the `c = 0` bit-identity and its mutation (drop the
  fill, drop the regolith write, drop the breach branch: each turns a test
  red).
- The switch-off identity of §6.3, by hash.
- Construction's share of the B1, B2 and B4 movement GF-10 measured is
  re-measured on the built stage and matches GF-10 within its spread.
- `cargo test --workspace`, with the result-line count checked against the
  previous floor. Annotated per Ruling BK.

### GF-12 — the uplift field and the uplift-driven stage (Ruling BM, part 2)

§4.15, behind the same switch.

- The uplift field, with Ruling BN's rift term (shoulders and basin
  subsidence, and the basin fill); `U₀` by the mean-balance rule GF-10 fixed;
  `U_bg`.
- The stage: `N_BM` implicit steps with per-step routing, `dt = T(τ)/N_BM`,
  lakes and ocean pinned, no deposition, contact lifted by uplift; replacing
  the light pass's stream power and rebound when the switch is on.
- The rock entry point gains the uplift slice, `dt` and the pinned mask as
  `Option`s; the legacy entry points are untouched.
- The clock scales `T` and the construction budget (§4.16); GF-7's readout
  line says so when the switch is on.

**Done means:**
- B13 and B15 pass, with their mutations.
- With GF-11: **revised B1, B2 and B4 pass at 800 km on all five seeds**;
  B3 and B8 pass at every τ in {0.5, 1, 2, 4}; B8 and B4 are reported at 80
  and 8 000 km; B9 ≤ 1.20 × control at τ = 1, and the cost at other τ equals
  τ = 1's within brackets (B12's BM extension).
- The kernel goldens and every legacy entry point are bit-identical; the
  switch-off identity of §6.3 holds by hash.
- The DLL and `.rs` mtimes are stated at the end of the run.

### GF-13 — combine, look, and decide the app switch (Ruling BM)

- Windowed screenshots (`_gf2relief_shot.tscn`, which gains a `--relief`
  flag that aborts if the key is rejected), seeds 483920 and 314159 at
  800 km, arm A against arm C at τ = 1 and 4, **opened and looked at** at fit
  and at deep zoom (`MISTAKES.md`, "Verify a visual change"). The question they
  answer: do caps stand as plateaus behind scarps, do weak-rock lowlands open,
  and do active belts read steeper on strong rock?
- If every GF-12 bar holds, `geology_processes` and `relief_model` go on in
  `cartalith_godot::params::defaults()`, with §6.3's disclosure; if any bar
  fails, they stay off and the finding goes to the owner.
- The weathered mantle (§5.8's open contradiction) is wired under the switch
  if §9 Q19 is answered that way.
- The stage-06 age row is seen rendered in a running shell (§5.8 still owes
  it), now reading the model-time wording under BM.

**Done means:**
- The screenshots are in the commit's evidence and described in §5.
- The app-boundary divergence list, `GENERATION_PARAMETERS.md` and
  `params_mapping.rs` agree.
- `cargo test --workspace` green with the line count checked, and
  `--check-only` on every touched `.gd` plus `shell/app.gd`.
- `STATUS.md` is updated.

### GF-13 under Ruling BY: the bars re-read for construction alone, the evidence, and the switch-on for the owner (2026-10-12)

*Written 2026-10-12, after GF-11 landed (`41ad59ac`). This is design and
evidence, not status; `cartalith-native/docs/STATUS.md` says how far it has
got. The GF-13 entry above was written when GF-12 was planned. Ruling BY
(2026-10-10) dropped GF-12, so its first bullet's condition ("if every GF-12
bar holds") names bars for a stage that does not exist. This section re-reads
them for a construction-only build. It was written after GF-11's numbers were
known, and says so: nothing below lowers a threshold.*

**What GF-13 asks, item by item.**
1. Windowed screenshots, arm A against arm C at τ = 1 and 4, seeds 483920 and
   314159 at 800 km, opened and looked at, at fit and at deep zoom. *Done
   below.*
2. The switch-on decision: `geology_processes` and `relief_model` go on in
   `cartalith_godot::params::defaults()` only if every bar holds, with §6.3's
   disclosure; otherwise they stay off and the finding goes to the owner.
   *Read below: a bar fails, so they stay off and this goes to the owner.*
3. The weathered mantle under the switch, if §9 Q19 is answered yes. Ruling BN
   kept Q19's default (yes). *Not wired; see owner question 4 below.*
4. The stage-06 age row seen rendered in a running shell, with the model-time
   wording under BM. *Built and probed; see below.*

**The switch-on bars, re-read for construction alone.** The switch-on bar is
§5.2's, which holds on **all five seeds** unless a bar says otherwise. GF-11's
own build rule asked for 3 of 5 on G1 and G2 because it licensed building
behind an off switch. `GF11_LANDFORM_SEEDS`' doc in `gf0_geology_harness.rs`
already says *"GF-13's switch-on still needs §5.2's five."* Each reading below
comes from GF-11's measurement. The built stage reproduces that measured arm
bit for bit (`gf11_built_stage_reproduces_the_measured_arm`, and the ignored
full-size run on all five seeds; `STATUS.md`, GF-11 entry), so it was not
re-run.

| GF-12 bar as written | Construction-only reading, and why | Measured (GF-11, 800 km) | Result |
|---|---|---|---|
| B13 (steady state of the uplift stage) | dropped: there is no stage (`GF11_RULE`, "Not gated") | — | not applicable |
| B15 (clamp count, `U₀` balance) | dropped: both are the stage's. G4's headroom check replaces the clamp | — | not applicable |
| revised B1 and B2 on 5 seeds | reported, not gated: they read the steady state under `U > 0`, and construction leaves `U = 0` (`GF11_RULE`'s reasoning, §4.15) | not gated | not applicable |
| revised B4 on **5** seeds | G1 (breach-line slope ratio ≥ 2.0 and top-decile share ≥ 0.40), now on 5 of 5 | **3 of 5**: 12345 fails on share (0.310), 314159 fails on ratio (1.503) | **FAIL** |
| B4's twin on 5 seeds | G2 (C over B slope on the predicted breach lines ≥ 1.2, population ≥ 100) | 5 of 5 | pass |
| B3 | reported (§5.9) | reported | not gated |
| B8 at every τ in {0.5, 1, 2, 4} on 5 seeds | G3 under rule v3, with Ruling BY item 4's exemption | every seed, every τ | pass |
| B8 and B4 at 80 and 8 000 km | reported in GF-12's text, never gated | not measured | open: not gated |
| B14 | G4 under Ruling BY item 3 | every seed, arm and τ | pass |
| B9 ≤ 1.20 × at τ = 1 | ≤ **1.25** × by Ruling CA item 1 | 1.211 and 1.187 | pass (relaxed bar) |

**Verdict: a bar fails.** Revised B4 holds on 3 of 5 seeds, not five. Under
GF-13's own rule and Ruling BO's standing answer (*"If any bar fails, leave
it off"*), `relief_model` and `geology_processes` **stay off in the app**,
and the decision goes to the owner (questions below). Nothing was switched
on.

**The screenshots** (`_gf2relief_shot.tscn`, which gained `--relief`,
`--seed` and `--zoom-at`; run windowed from a `git archive` copy of
`41ad59ac` with the DLL built from it). Arms: A = app world; P1 = processes
on, relief off; C1 = processes and relief on at τ = 1; C4 = the same at
τ = 4. Fit images are 2048 × 1311. The deep-zoom crops are 65 cells across
512 px (about 7.9 px per cell), drawn by `export_snapshot_png`, the export's
`bake_rect`. That is a magnified render of the grid, **not** the app's LOD
tiles, which add synthesized sub-cell detail. All images were opened and
looked at. The evidence set is in `cartalith-native/docs/evidence/gf13-2026-10-12/`:
- `fit_A_C1_<seed>.jpg`: A beside C1, at half size;
- `zoom3_<seed>_z<x>_<y>.jpg`: A, C1 and C4 at each crop;
- `state*_row.jpg`: the two age-row screen crops.
- **Pixels moved at fit** (summed RGB difference > 8). A against P1: 49 431
  and 48 089 of 2 684 928, which is the light pass. A against C1: 812 405
  (483920) and 1 079 964 (314159). C1 against C4: 1 101 405 and 1 577 783.
  Construction is a large visible change, and τ = 4 doubles it.
- **Do caps stand as plateaus behind scarps? In places.** On 483920's
  northern upland (crop at cell 384,64), C1 shows a pale block standing
  beside a darker lowered strip, with a sharp north–south edge that the rivers
  follow. That is the intended form. At τ = 4 the strip under that edge
  becomes a long lake with a stepped shoreline. Elsewhere (314159, crop at
  500,1000), a breach line reads as a narrow dark line one or two cells wide
  across a plain. At this scale it looks more like a ditch than a scarp face.
  This crop was chosen as a plain land area, not by where construction
  changed the map. The 384,64 and 768,224 crops are the blocks of largest
  A-against-C1 change: **selected by the output, so they show the strongest
  case.**
- **Do weak-rock lowlands open? Yes.** Lowered ground reads darker and
  greener in both seeds: the lowering moves the climate and biome colours.
  It is clearer at τ = 4.
- **Do active belts read steeper on strong rock? Not visible, and not
  expected.** That is the uplift stage's job (§4.16), and Ruling BY did not
  build it.
- **The coast moves visibly** (Ruling BN allows it). On 483920 the isthmus
  between the two northern seas is cut through, and the eastern coastal strip
  breaks into islands. On 314159 a shelf area changes water class at τ = 4.
- **A defect to look at before any switch-on.** In deep zoom on 314159 (crop
  768,224) the edge between the lowered land and the snow/ice field draws as
  a **one-cell staircase**: a visible pixel step at C1, stronger at C4. A
  shows a smooth edge there. The same stepping marks the lake shore in
  483920's C4 crop. It is consistent with construction's step breach rule
  lowering whole cells by `B·κ`, so neighbouring cells end at different
  heights. That reading comes from the code, not from a measurement.
- **Fit-scale look.** C reads flatter. The relief shading's contrast is
  lower, and 314159's orange highland tint and crater rim are muted. Fine
  stream lines appear on the lowered plains.
- **A control.** 483920's crop at the map centre (1024,655) is open sea. It
  moved 0 pixels between arms.

**The stage-06 age row, seen.** `_gf13agerow_probe.tscn` boots the real
shell windowed, generates a world and captures the row from the screen in two
states. With the app's defaults the row is inert and dimmed, and its note
gives the reason and *"stream power 9 it"*. With `geology_processes` and
`relief_model` set on the live `WorldGen` and the slider at 2.0, the row is
editable and reads the construction clause. The probe requires the two
screen crops to differ (positive control). The windowed run of 2026-10-12,
with the DLL rebuilt after the `.rs` change, passed 13 checks with 0 failed;
the positive control moved 3 552 px. In state 2 the row read *"construction
×2.00 (in place of stream power) · hillslope 16 passes · glacial 13
passes"*. Under the relief switch the
readout (`erode_bridge.rs` `geo_clock_readout_parts`) now writes
*"construction ×τ (in place of stream power)"* as its first clause. Before
this change it listed a stream-power count for a pass that does not run on a
constructed world (`generate_terrain`'s `if !constructed`). With
world-structure on, construction skips and the stream-power clause returns.
The two world-dependent skips (no land, no two-layer land) cannot be seen
from parameters, and the function's doc says so.

**What a switch-on moves: measured by flipping the default.** This was run in
a scratch worktree of `41ad59ac`, never the repository: `geology_processes`
and `relief_model` were set to `true` in `params::defaults()`, then
`cargo test -p cartalith-godot --no-fail-fast -j 2` ran. Result: 47 result
lines, **2 589 passed, 5 failed**, 68 ignored. Only `cartalith-godot` calls
`params::defaults()`; the other crates' mentions are comments. The five
failures:
1. `params_mapping.rs::exactly_the_ruled_divergences_ship_at_the_app_boundary`.
   An intended pin: it gains the two flags as the eighth and ninth ruled
   divergences (§6.3).
2. `params_mapping.rs::a_state_this_port_does_not_recognise_is_survivable`.
   A reference export reloads `relief_model` **off**, the §6.3 rule that an
   old world regenerates as it was, and the test expects the app defaults.
   Its expectation needs the same carve-out `geology_processes` already has.
   The new behaviour is intended.
3. `erode_bridge.rs::geo_clock_readout_tests::readout_on_the_app_parameters`.
   It asserts the row is inert on the app's parameters, and it is re-pinned
   to the active readout.
4. `gf0_geology_harness.rs::gf11_built_stage_reproduces_the_measured_arm`
   (*"arm A is the app's world"*).
5. `gf0_geology_harness.rs::replica_reproduces_generate_terrain` (*"app-path
   replay diverged on seed 483920"*). Items 4 and 5 are the harness's own
   arm A, which is defined as `params::defaults()`. Arm A must be pinned to
   the pre-switch parameters (processes and relief off) so the measured arms
   keep their meaning.

**No `golden_parity_*` test moves**: they run `WorldParams::defaults`, where
both flags stay off. No other test in the workspace pins an app-default
world's bytes. **What moves outside the tests** is every app world, which
§6.3 says must be recorded old → new:
- RV-1's `_riverzoom_probe` figures;
- the LOD-D4 bars;
- `PERFORMANCE_BENCHMARKS.md`'s generation times;
- `GENERATION_PARAMETERS.md`'s divergence costs.

`GENERATION_PARAMETERS.md`'s list says "six deliberate divergences", while
the boundary test already neutralises seven (`geology_model`, Ruling BH). It
was stale before this change, and a switch-on fixes it in the same edit.

**The re-baseline plan, if the owner switches on.** One change:
1. Flip both flags in `params::defaults()`, with the comment citing the
   ruling.
2. Apply the five test changes above, each with its old and new value in the
   commit.
3. Re-run and record old → new for the four figure sets above; each probe
   runs windowed, alone.
4. Add `GENERATION_PARAMETERS.md` rows for both flags, with B9's measured
   cost (a median with min..max over two separate runs, alone).
5. Probe a save from before GF-11: it reloads with relief off, and the
   regenerate confirmation (`_authored_inventory()`) is unchanged.
6. Run `cargo test --workspace` against the 209-line floor, and
   `--check-only` on every touched `.gd` plus `shell/app.gd`.

No `golden_parity_*` file is touched. GF-9 later decides between route A and
route B (§6.2).

**Owner questions, each with a default.**
1. **Switch construction on although revised B4 holds on 3 of 5 seeds, not
   five?** Default: **no, keep it off.** The bar is the scope's, and the
   screenshots add two things the bar does not see: a one-cell staircase at
   deep zoom, and breach lines that read as narrow lines at fit. The
   alternative is to switch on with B4's failure disclosed. That is an owner
   call, because it overrides a bar Ruling BO said to respect.
2. **Investigate the deep-zoom staircase before any switch-on?** Default:
   **yes.** Measure where it comes from first: the step breach rule, the
   fill, or the shading of a one-cell height step. Then decide whether to
   smooth construction's lowering across a breach line. Any fix moves GF-11's
   measured arm, so it re-measures G1–G5 under a rule written before the run.
3. **Ruling BN's moving coast** cuts 483920's northern isthmus and breaks a
   coastal strip into islands at τ = 1. Default: **keep BN as ruled.** The
   owner may want to see these frames first, since this is the first time
   the effect has been shown at app scale.
4. **The weathered mantle (§9 Q19, yes by Ruling BN).** Default: **leave it
   unwired until `h₀ = 0.5 m` is verified at its source.** §4.12 and §2.3
   require that before the value is pinned, and §5.8 found only Heimsath et
   al.'s abstract. The mantle cannot change the exposed rock on its own (its
   2.69 m maximum is below `R_EXPOSE`), so nothing visible waits on it.
5. **World-structure worlds skip construction**
   (`ConstructionSkip::WorldStructureOn`). Default: **accept, disclosed in
   the readout**, until the orogeny term is kept past the substrate.
6. **A GUI control for `relief_model` (§9 Q13's "no GUI control until
   GF-13").** Default: **none while it is off.** If it is switched on, a
   stage-04 checkbox row with the age row's dim-and-reason pattern.

### The deep-zoom staircase under Ruling CG item 2: cause, fix, re-measurement (2026-10-12)

*Design and evidence, not status; `cartalith-native/docs/STATUS.md` says how
far it has got. `relief_model` stays off in the app throughout.*

**Cause, measured.** `gf0_geology_harness.rs` `gf11_staircase_diag` splits
every 4-neighbour pair of land cells by what makes construction's lowering
`D = B·k` jump across it, read from inputs only. In the two GF-13 crops the
stepping pairs are **top-rock edges, not breach lines**: at 314159 (768,224),
τ 1, 184 top-rock pairs step a median 167 m in arm C against 12 m in arm A,
and its 28 breach-line pairs step 0 m. At 483920 (384,64), τ 4, 44 top-rock
pairs step a median 954 m before the hillslope (677 m after) against 1.5 m in
A. `build_geology`'s belts are chamfer-distance thresholds and its cover is a
threshold, so the factor `k = κ^c` changes along lines the grid samples at
cell centres. The scarp is intended; drawing it along the grid's staircase is
aliasing. On whole maps, at τ 1 and C final: top-rock pairs 26 925 (483920)
and 33 615 (314159), median step 47 and 48 m against A's 5.1 and 5.4 m;
breach-line pairs 773 and 881; arc-edge pairs 771 and 583. The hillslope
barely changes any of them.

**Fix (behind `relief_model`).** `cartalith_erosion::construction::subcell_edge_factor`
gives each cell on a two-valued edge the area-weighted blend of the two
factors: the edge is recovered as the mid-level of the factor field blurred
over lowered cells (`EDGE_BLUR_R` 1.6), and the cell's share comes from its
signed distance to that edge through the exact box-filtered half-plane
(`unit_cell_cover`). This is distance-field anti-aliasing (Green 2007). Never
outside `[lo, hi]`. Cells beside the coast or an arc edifice and three-value
junctions are untouched, which keeps §9 Q17's exemption and the `c = 0`
identity exact. 98.6 % of edge cells (483920) and 98.0 % (314159) are blended.
`EDGE_BLUR_R` was picked by measurement: it has the least RMS error against
the exact area cover of straight edges and discs, 0.116 against the
staircase's 0.251. Which shapes stand in for the geology's edges is a
judgement. The harness gained `Breach::StepSubCell` (`GF11_BREACH=subcell`),
and the built stage reproduces it bit for bit.

**Switch-off identity.** Parity, app, and processes-on worlds at τ 1 and 4
were hashed (seeds 483920, 314159, 12345, 512×328) in a `git archive` copy of
`f8ea9f93` and in that copy plus this change: all 12 are identical. The
relief-on control moves on all three seeds.

**Re-measurement (rule v3's thresholds, unchanged; not a new pre-registered
rule).** The rule's setting line names the step rule, so this is a variant
run, disclosed as one. Both rules were run on the same tree on the same day.
Since GF-11's measurement the app world has moved (the river chain-break fix
F1 re-baselined app worlds), so the GF-13 table's G1 readings are stale.

| Bar | Step, HEAD tree | StepSubCell |
|---|---|---|
| G1 (B4 ratio ≥ 2.0, share ≥ 0.40) | **3 of 5**. Fails 483920 (ratio 1.355) and 314159 (0.984); 12345 passes (2.004, 0.472) | **2 of 5**. Fails 483920 (1.180), 12345 (1.402, share 0.358) and 314159 (0.996) |
| B4 ratios, all seeds | 1.355, 4.198, 10.253, 2.004, 0.984 | 1.180, 2.662, 3.090, 1.402, 0.996 |
| G2 (≥ 1.2) | 5 of 5 | 5 of 5 (2.56-10.07) |
| G3, every τ | pass | pass |
| G4 | pass | pass (c = 0 identity misses 0) |
| B9 (≤ 1.25, Ruling CA) | 1.192, 1.191 | **1.224, 1.238**. Construction 1.43 → 1.56 s median |

The fix **costs B4**. B4 reads a one-cell slope at breach-line cells, and
spreading the step by area share across the cells the edge crosses lowers
that slope. That is the arithmetic of anti-aliasing. Nothing was tuned
toward B4.

**Screenshots** (windowed, `_gf2relief_shot.tscn`, before = a `git archive`
copy of `f8ea9f93`, after = the same plus this change, each with its own DLL
and its own `config/name`; every image opened, crops checked at full
resolution). Evidence: `cartalith-native/docs/evidence/cg2-staircase-2026-10-12/`.
`cmp_*` sets top row before, bottom row after, columns A, C1 and C4; `c*_*`
are 2× crops, before on the left. Arm A moved 0 px in every crop.
- **Gone:** 314159's τ 1 ice edge (`c1_768_ne`) is a smooth line, and the
  τ 1 and τ 4 breach-line "ditch" at 314159 (500,1000) loses its beads.
- **Reduced, not gone:** 483920's τ 4 lake shore (`c4_384_shore`) loses its
  notches but keeps a one-cell scallop; 483920's τ 1 scarp beside the river
  (`c1_384_scarp`) is still notched.
- **Not touched:** 314159's τ 4 NE edge (`c4_768_ne`) keeps crisp 8-px stair
  blocks in both builds. There a filled flat at 1 120 m meets the slope on an
  exact 45° diagonal (1 120 → 1 345 → 1 446 m per row), and the blocks are
  sharp-edged, so a field sampled at the nearest cell is a **suspicion, not
  checked**.
- **New:** (a) at τ 4, 314159 (`c4_768_nw`), a beaded dark diagonal line
  across a ~1 100 m-deep filled flat. The surface there is bit-identical
  between builds. What moved is `flow_discharge`: a new drainage path over
  the flat, after the edge cells' regolith changed. It is a re-routed flow
  line drawn cell by cell, not a height step. (b) At 483920 τ 1, a river
  segment beside the scarp draws much wider, and new short river strokes
  appear at 314159 (500,1000). Both are routing consequences of the moved
  heights.

**Owner questions, each with a default.**
1. **Keep the anti-aliased edge (built, off with `relief_model`)?** Default:
   **keep it.** It removes the measured τ 1 staircase, and its cost is B4
   2 of 5 instead of 3 of 5, with B9 at 1.22-1.24 against 1.25.
2. **Blend top-rock edges only, and leave breach lines one-cell?** That
   would probably give back most of B4, but it would keep the staircase on
   breach lines (773 and 881 pairs per map). It was not measured. Default:
   **no**: it is tuning toward the bar.
3. **Chase the remaining τ 4 stair** (the filled-flat margin and the nearest
   cell render) and the beaded flow line? Default: **yes, before any
   switch-on**, as its own row.

### Follow-up: what the remaining artefacts are (2026-10-12)

*Design and evidence, not status. Question 3's default taken; `relief_model`
stays off.* Method: windowed renders through `export_snapshot_png` (the
crops above), one appearance group zeroed per render with `set_appearance`,
and per-cell dumps with `sample_cell`. They were made in `git archive`
scratch copies of `ee3bcec7` (then HEAD) and `f8ea9f93` (before the edge
fix), each with its own DLL and project name. Evidence:
`cartalith-native/docs/evidence/cg2-followup-2026-10-12/`.

**(a) The τ 4 NE stair is the lithology colour sampled at the nearest
cell, not the heights.**
- **Found by ablation.** Zeroing `litho_strength`, `litho_exposure` and
  `geo_micro` removes it (`a_litho_nearest_cell.jpg`, with the difference
  map). Zeroing any of these leaves it (`a_ablation_sheet.jpg`): hillshade
  (`exag`/`relief_gain`), AO/sky/shadow, curvature, textures, `wetness`,
  SDF bands, `hydro_wet_strength`, ice, colour relief or the detail bands.
- **The sampler.** `RenderCtx::litho_at_f` (export bake and deep-zoom tiles
  alike) rounds to a cell after a coherent jitter. The jitter is smooth over
  a few cells, so it shifts a straight contact but cannot break its
  staircase.
- **The contact.** The cells (`sample_cell`) put Shale beside Limestone on
  an exact 45° line. That line is the breach line: the limestone is exposed
  where the shale cap is breached, and it is also the fill edge
  (1 120 → 1 364 → 1 456 m). The contact sits on the steep scarp, so
  `land_color`'s bedrock-exposure weight (`smoothstep(1.5, 9, slope·gw)` ×
  cover) is high there.
- **Why the edge factor did not help.** It blends the factor `k`, so the
  heights are anti-aliased. The label it leaves is per cell and
  categorical.
- **App worlds.** Relief off, the same crop is all shale. Two app-world
  contacts (314159 at (472,256) and (880,160)) were on gentle ground and
  showed no visible stair. So no visible app-world instance was found.
  But the sampler is shared, and the reference's own bake read (`lithB`)
  is nearest too.

**Proposed separately, not applied (it moves app-world output).** Filter
the category: evaluate the lithology-dependent colour terms
(`rock_material_col`, the exposure tint `ramp3(litho_palette(li))`, the
microtexture factor) for each jittered corner's rock, then blend them by
the bilinear weights. This is percentage-closer filtering applied to a
categorical map, the same idea as blending splat-map materials.
- **What it would move:** export, snapshot and deep-zoom-tile pixels near
  a lithology contact wherever `litho_strength`/`litho_exposure` > 0. The
  shipped looks set 0.62 / 0.55.
- **What should stay:** the fit-view base map (`cell_color`, one pixel per
  cell, `litho_at`). So should every render golden under `js_reference()`,
  whose lithology weights are 0. That is to be confirmed by the run, not
  assumed.
- **Tests to re-check:** those that attach `with_lithology` under a
  shipped look (`bake_raster`, `appearance_tiers`, `layer_stack`,
  `cel_toon`, `color_space`, `sdf_river_and_biome`,
  `geology_micro_and_sky_fields`).
- **What it needs:** it is a §7p divergence from the reference's nearest
  read, so it needs an owner call. Frames are needed at both a relief-on
  scarp and an app-world steep contact.

**(b) The beaded flow line is a routing consequence, drawn by the TWI
wetland term. The F1 receiver guard is not involved.**
- **Not the guard.** The guard only re-points channel receivers in
  `build_channels_core`, and this line is not a channel.
- **The flat itself did not move.** The 1 120 m flat is identical in both
  builds (6 644 cells in common). But 429 cells of the 131 × 91 window
  around it moved, including cells that joined the flat.
- **The routing did.** Priority-Flood+ε's routing across the exact flat
  reorganised. `flow_discharge`'s median on the flat went from 11.7 to
  95.6, and its maximum from 5 989 to 16 690
  (`b_flow_line_pre_vs_head.jpg`, before left).
- **Why it now shows.** Flow reaches the land colour only through
  `twi = ln(a/β)`, with β floored at 0.002 on a flat; `material_weights`
  then turns twi into a wetland weight. Flow of 16 690 gives twi ≈ 1.13,
  a weight of ≈ 0.80 under `smoothstep(-1, 2, ·)`. The old maximum gives
  0.11, a weight of ≈ 0.31.
- **Why it is straight.** On an exactly flat surface, ε-filling routes
  along straight D8 lines; that is the method's known parallel-flow
  behaviour. Only construction's fill makes dry exact flats, so this is
  relief-on only.
- **Why it is beaded.** The beads repeat every 11.3 px, against one
  diagonal cell step of 11.1 px. Sampling a one-cell diagonal path
  bilinearly gives two of four corners between centres, so the line halves
  there. That is the renderer, shared with app worlds.

**Candidates, not built.**
- A flat-resolution gradient over construction's fill. Garbrecht and
  Martz (1997) is the classic; Barnes, Lehman and Mulla (2014) give an
  efficient version for flats. Gated to the filled cells, it would stay
  relief-on only.
- A smoother flow sample in the renderer. That one is app-wide.

**(c) 483920's τ 1 notched scarp is a carved river gorge, not
construction.**
- **The gorge.** With `carve_rivers` off, a 1.2 km-deep gorge disappears
  (`c_scarp_carve_on_vs_off.jpg`). It is 2-3 cells wide, floored 1-7 m
  above sea, and runs along the shale/limestone contact. What remains of
  the scarp is one blended cell: 983 → 1 204 → 1 266 m.
- **What draws the notches.** The notches are the gorge walls stepping
  with the D8 channel. `carve_rivers` runs in every world, so a fix there
  is not confined to `relief_model`. Arm A has no gorge here, because the
  river is routed elsewhere.
- **The lithology again.** The carve-off scarp still shows crisp blocks:
  that is (a)'s lithology colour (crisp-step count 352 → 128 with it off;
  `c_scarp_carve_off_litho_on_vs_off.jpg`).
- **A question raised, not investigated:** whether a carve that deep
  through construction's exposed substrate is intended. `stream.k = 0`
  left this window bit-identical, which was also not investigated.

**(d) The τ 4 shore scallop is the anti-aliased step read at a level 30 %
up it.**
- **The step.** The shore runs along a ~1 km construction step: −330 m
  water side, one or two blended cells, then +745 m
  (`c_shore_scallop.jpg`).
- **Why it wobbles.** The edge factor's box coverage is exact for a
  cell's area. It is not exact for a contour at another level of the
  bilinear surface. A model of straight edges at 10-77° (scratch
  `wobble.py`, not committed) gives the level-0.3 contour 0.066 cells RMS
  of wobble with box coverage. Linear feathers in signed distance give
  0.033, 0.014 and 0.002 at 1.5, 2 and 3 cells wide.
- **What the model does not explain.** The dumped contour advances in
  jumps of about 0.5 cell, more than the model predicts. The excess (the
  τ 4 processes, two-cell ramps) was not isolated.
- **What a fix would cost.** A 3-cell feather would straighten it, but it
  spreads a 1 km scarp over ~1.2 km. That is a landform change, and it
  would lower B4 further, since B4 reads one-cell slopes. Building it to
  hide a rendering artefact is the tuning question 2 declined. **Not
  built.**
- **Not re-measured.** B4, G1-G4 and B9 were not re-run, because nothing
  changed.

**Owner questions, each with a default.**
4. **The categorical lithology filter in (a)?** It is the only one of
   these that removes a crisp stair, and it moves app-world exports and
   tiles near steep contacts. Default: **schedule it as its own renderer
   row**, with frames and a test re-run before any re-baseline.
5. **A flat-resolution gradient over construction's fills (b)?**
   Default: **no for now**. The line is a correct routing consequence,
   and the beading is the shared renderer.
6. **A feathered edge (≥ 3 cells) for (d), against a softer scarp and a
   lower B4?** Default: **no**.
7. **Is (c)'s 1.2 km carve through exposed limestone intended?**
   Default: **investigate as part of the GF-4…GF-9 measurements**, not
   here.


### GF-4 measured, and what of it is built (2026-10-12)

*Design and evidence, not status.* GF-4 was the first unbuilt milestone of
Ruling BN's order after GF-13 (opened at its symbols: glacial and coastal
called `glacial_kernel` / `coastal_process` with no rock input, after the
trace; `params::defaults()` left `passes.coastal` off). Construction stays
off (Ruling CG), so everything here is measured **without** it, on
`params::defaults()` with `geology_processes` on, `relief_model` off, at
800 km and 2048 × 1311, `c = tect.resist` = 0.5. Harness:
`cartalith-godot/tests/gf4_glacial_coastal.rs` (`gf4_bars`, ignored; module
doc gives the command).

**What was built.**
- **The rock kernels** (`cartalith_erosion::passes`): `glacial_kernel_rock`
  (§4.4: `e·γ(exposed)^c` on the trunk, flank and cirque terms) and
  `coastal_process_rock` (§4.5: wave cut, debris and estuary cut each ×
  the cutting cell's `ρ(exposed)^c`; marsh unchanged), behind a shared core
  that takes the reference arithmetic by control flow without rock.
  Exposed rock is read per pass at the pass-start surface. The kernel
  goldens (`golden_parity_passes.rs`) are unchanged.
- **The coastal pass is wired** through `RockContext::coastal`: rock-reading
  with `geology_processes` on, debris and marsh gain accounted as regolith
  (§4.9); the legacy pass otherwise.
- **The coastal pass runs ahead of the trace** (§3.2 item 2) whenever it is
  on and `carve_rivers` is on, with or without the processes. Reason,
  measured: after the trace it sank land the trace had already routed
  through. B8's hard guard (ocean cells on river paths) read **199, 1 933,
  645, 281, 154** on the five seeds with the pass on at its old position,
  and **0 on all five** after the move, for both the app world and the
  processes-on world. `geology_gf4.rs`'s carved-river-below-sea test reads
  43 on seed 483920 at 256 × 164 on HEAD `e80ba61a` and 0 here. No world
  that ships moves: the pass is off in the app and on the parity path, so
  parity, app and processes-on worlds (seeds 483920 and 314159, 512 × 328,
  13 arrays each) hash identical to `e80ba61a`.
- **Glacial is not wired.** Wiring it moves every processes-on world (the
  app runs glacial), and the GF-0 harness's
  `replica_reproduces_generate_terrain` must change in the same commit; that
  file belonged to a concurrent lane, so it was left alone. The kernel and
  B6 are ready for it.

**B6 — glacial reads jointing (kernels alone, on the pipeline's own
post-trace inputs; control = `c = 0`, asserted equal to the legacy kernel,
and the legacy chain replayed bit for bit against the generated world).**

| Seed | control | treatment | bar (≥ 1.5 and ≥ 1.5 × control) |
|---|---|---|---|
| 483920 | 2.033 | 3.486 | PASS (3.050) |
| 24601 | 15.965 | 26.953 | PASS (23.947) |
| 71077345 | 1.689 | 2.698 | PASS (2.534) |
| 12345 | — | — | not measurable: 62 weak-rock ice cells in the usable bands |
| 314159 | — | — | not measurable: 63 weak-rock ice cells |

Treatment ÷ control is 1.60-1.71: passing, with little margin.

**B7 — coastal retreat reads rock: FAILS** on every measurable seed (same
method, at the pass's pre-GF-4 inputs; the replay was asserted bit for bit
before the move).

| Seed | control | treatment | bar (≥ 3 and ≥ 2 × control) | marsh off: control → treatment |
|---|---|---|---|---|
| 483920 | 2.493 | 3.523 | FAIL (relative: 4.985) | 3.174 → 6.903 |
| 24601 | 0.071 | 0.090 | FAIL | 0.083 → 0.155 |
| 71077345 | 1.042 | 1.397 | FAIL | 0.919 → 1.488 |
| 12345 | 1.715 | 2.549 | FAIL (relative: 3.429) | 1.607 → 3.418 |
| 314159 | — | — | not measurable: 74 weak-rock coast cells | — |

**Why, as far as measured.** Land is "lost" only when the cut takes a cell
below sea, and at the default `wave_str` 0.5 over 4 passes the cut is at
most about 0.004 (normalised) in total, so mostly cells already within a
few metres of sea are lost, whatever their rock. The tidal marsh's
value-noise bump (±0.55 · `marsh_band`, rock-blind under §4.5) also lowers
cells below sea on its own: with the marsh off the treatment/control ratio
roughly doubles (1.62-2.17), which is the bar's relative half on 2 of 4
seeds. That the marsh is the main remaining cause is an inference from that
diagnostic, not isolated further.

**B8 with the pass on (moved):** ocean on paths 0 everywhere; lake share of
path cells within +2 pp except 24601 (1.61 → 6.70 % app, 1.97 → 7.13 %
processes-on); 1-3-cell lakes over 1.25 × on 24601 (68 → 205; 49 → 238)
and 314159 (62 → 102; 37 → 62). **So B8 fails on 2 of 5 seeds.** On 24601
the pass moves 335 000 cells and turns 44 000 land cells to water. Cause
not isolated (the marsh noise is the suspect again).

**B9 for the coastal pass on in the app:** off 4.81 s (3.94..6.60), on
4.34 s (4.09..5.67), medians of 5 interleaved runs, seed 483920: the
brackets overlap, so no difference established. The machine was not quiet
(another lane was building), so this is not a cost disclosure for
`GENERATION_PARAMETERS.md`.

**Done-means against what exists.** B6: passes where measurable (3 of 5),
not yet in the pipeline. B7: fails. B8 with the pass on: fails on 2 of 5.
So the coastal default in `params::defaults()` (Ruling BJ, Q7) is **not
flipped**: GF-4's own done-means pair it with B7 and B8 passing, and with
the processes off in the app the pass would not read rock anyway. Ruling
BJ's wording, "on in the app at GF-4, when cliff retreat starts reading
rock", assumed rock-reading would be live in the app. It is not, because
`geology_processes` stays off (§5.6-§5.8, Ruling CG).

**Owner questions, each with a default.**
8. **The coastal flip while the processes stay off?** In the app it would
   be rock-blind cliff retreat, it fails B8 on 2 of 5 seeds, and it moves
   every app world. Default: **wait** until `geology_processes` ships and
   B7 and B8 pass.
9. **B7 and the tidal marsh.** (a) Measure B7 on the wave and estuary cut
   only (marsh off in the metric), (b) make the marsh read rock (no
   physical basis found for that), or (c) keep the bar and accept that
   GF-4's coastal half fails. Default: **(a), disclosed as a revision made
   after seeing the result**. Even then the bar passes on 2 of 4 measurable
   seeds, not all.
10. **B8's small lakes with the pass on (24601, 314159).** Default:
    **investigate the marsh-noise cause before any flip**, as its own row.
11. **Wire glacial now** (rock kernel plus the move ahead of the trace,
    updating the GF-0 replica in the same change)? Default: **yes**, once
    the harness file is free. Its B6 passes where measurable.

### GF-4 under Ruling CH: B7 revised, glacial wired, the small lakes traced (2026-10-12)

*Design and evidence, not status.* Ruling CH answered questions 8-11: B7
revised (question 9, option a; §5.2's B7 carries the disclosure), glacial
wired (question 11), small lakes investigated (question 10), and no coastal
flip (question 8's default stands, since B8 still fails).

**B7, revised (marsh off in the metric; written after the result was
seen).** From the table above: 483920 3.174 → 6.903 (PASS: ≥ 3 and ≥ 6.35),
24601 0.083 → 0.155 (FAIL), 71077345 0.919 → 1.488 (FAIL), 12345
1.607 → 3.418 (PASS: ≥ 3 and ≥ 3.21), 314159 not measurable. **2 of 4
measurable seeds**, so B7 still does not pass on all seeds.

**Glacial wired** (`RockContext::glacial`). With the processes on, glacial
runs on `glacial_kernel_rock` **ahead of the trace** (§3.1: glacial,
rebound, coastal), after GF-3's hillslope, reading the priming
temperature (the only one computed by then), then the strip and the
rebound with the column. With the processes off it stays after the trace,
rock-blind, as before. The GF-0 replica's `tail` does the same (it now
takes the priming temperature, and `run_arm` passes its arm's `c` as
`tect.resist`, since glacial reads the one contrast knob).
`replica_reproduces_generate_terrain` and
`gf11_built_stage_reproduces_the_measured_arm` hold bit for bit.

- **Re-baseline, disclosed per DECISIONS §7p.** Every processes-on world
  that runs glacial moves (the app's own parameters do). Parity and app
  worlds hash identical to `cb4a7927` (two seeds, 512 × 328, 13 arrays);
  processes-on worlds differ in every array. Their B8 at 2048 × 1311, old
  → new (lake % of path cells; 1-3-cell lakes): 483920 0.977 → 0.977,
  43 → 43; 24601 1.965 → 1.966, 49 → 49; 71077345 0.660 → 0.660, 27 → 27;
  12345 3.809 → 3.855, 89 → 87; 314159 10.039 → 10.049, 37 → 37. Ocean on
  paths 0 → 0 throughout. One test changed:
  `geology_gf2.rs`'s `at_contrast_zero_the_treatment_terrain_is_the_controls`
  now runs with glacial off (the two arms now run glacial in different
  places, so no contrast can make them equal). One was retired:
  `glacial_lowering_strips_regolith_first_exactly` replayed a post-trace
  input that no longer exists. Its claims are held by the replica and by
  `lib.rs`'s `gf4_rock_context_glacial_is_the_rock_kernel_strip_and_rebound`.
- **B6 at the moved position** (`gf0_geology_harness.rs` `gf4_b6_moved`:
  the replica's own input, asserted to replay to the generated world):
  483920 2.040 → 3.503, 24601 17.950 → 29.333, 71077345 1.778 → 2.814,
  all PASS. 12345 and 314159 are not measurable (46 and 84 weak-rock ice
  cells). **3 of 5, the other 2 not measurable.**

**The small lakes (B8), traced.** The flip was re-run with one coastal
term removed at a time (`gf4_bars`' diagnostics; 1-3-cell lakes, control →
coastal on → marsh off → wave and estuary off):

| Seed | app world | processes-on world |
|---|---|---|
| 24601 | 68 → 205 → **69** → 155 | 49 → 239 → **169** → 135 |
| 314159 | 62 → 102 → **61** → 89 | 37 → 62 → **35** → 63 |

- **Cause on the app world and on processes-on 314159: the tidal marsh.**
  With the marsh off the count is back at the control's (69 vs 68, 61 vs
  62, 35 vs 37). The marsh's bump is centred value noise,
  `(vnoise − 0.5)·marsh_band·0.55·fade`, so it lowers about half the
  low-slope cells in its band by up to 0.008 (normalised). On flat
  lowland that digs one- to three-cell closed hollows, which the classifier
  calls lakes.
- **Processes-on 24601 has a second cause, not isolated.** With the marsh
  off it still reads 169 against 49, and with the cut off 135: both terms
  add lakes there. The rock-reading cut (`ρ^c` up to 3.16 on unconsolidated
  ground) and the debris written as regolith (which then reads as
  unconsolidated) are the suspects. Not measured.
- **Proposed fix, not built:** make the marsh accrete only (`max(0, bump)`).
  A marsh builds up and never digs. That is a physical reading of the
  pass, not one taken from a source at this line. It removes the measured
  cause on three of the four failing arms. It is **not applied**: in the
  app it changes the reference kernel (`coastal_process`, golden-verified
  in `golden_parity_passes.rs`), and on the processes-on path alone it
  would not fix 24601. Ruling CH allows a fix only if it is confined and
  golden-neutral, and this one is neither by itself.

**Owner questions, each with a default.**
12. **Accretion-only marsh?** In both kernels it re-baselines the
    reference marsh golden (§7p). In the rock path only, the app flip stays
    blocked. Default: **both kernels, with the golden re-recorded and
    disclosed**, since the noise-dug pits are an artefact, not a landform.
13. **The second small-lake cause on processes-on 24601?** Default:
    **investigate before any switch-on of `geology_processes`.**

### GF-4: accretion-only marsh and the second lake cause (2026-10-13, Ruling CH addendum)

*Design and evidence, not status.* The owner answered Q12 yes and Q13
yes. The coastal flip stays off.

**Accretion-only marsh, built in both kernels** (`coastal_process` and
`coastal_process_rock`: the bump is applied only when positive). The
coastal pass is off in the app and on the parity path, so app, parity and
processes-on worlds are unchanged: 13 arrays on two seeds hash identical
to `1376bc9a`. The reference golden moved, as expected:
`golden_parity_passes.rs` `coastal_process_case_0/1/2` were re-recorded
(6, 8 and 1 cells, every one higher; `RE-BASELINED 2026-10-13 under Ruling
CH`). Case 3 and every other golden, the `js_reference` ones included,
are unmoved (workspace run below). DECISIONS §7p has the entry. A new
test, `the_marsh_never_lowers_a_cell`, covers both kernels, and the mutant
that restores the dig fails it. **Frames** (app world, relief off, coastal
forced on with `_gf2relief_shot.gd --coastal on`, before and after DLLs
built from one worktree; `cartalith-native/docs/evidence/gf4-marsh-2026-10-13/`,
all opened). At fit the maps are nearly identical. In the most-changed
65-cell window of 314159, the scattered one- to three-cell blue water
specks on the coastal lowland are gone after, and the land around them is
unbroken. On 483920 a narrow isthmus that was notched and broken before is
continuous after. No new artefact was seen.

**Re-measured** (2048 × 1311, 800 km, the coastal pass on;
`gf4_glacial_coastal.rs` `gf4_bars`, from the exact commit content):
- **B7 (revised, marsh-free):** unchanged by construction, because the
  marsh can no longer sink a cell: 483920 3.174 → 6.903 PASS, 12345
  1.607 → 3.418 PASS, 24601 0.083 → 0.155 FAIL, 71077345 0.919 → 1.488
  FAIL, 314159 not measurable. **2 of 4.**
- **B8, 1-3-cell lakes, control → coastal on** (ocean on paths 0
  everywhere):

  | Seed | app world | processes-on world |
  |---|---|---|
  | 483920 | 34 → 32 | 43 → 41 |
  | 24601 | 68 → **112** (FAIL) | 49 → **169** (FAIL) |
  | 71077345 | 30 → 24 | 27 → 26 |
  | 12345 | 89 → 75 | 87 → 76 |
  | 314159 | 62 → 71 (was 102) | 37 → 39 (was 62) |

  B8 now fails on **1 of 5** seeds (24601), on both worlds. Lake share
  of path cells stays within +2 pp everywhere: 24601 is 1.61 → 2.35 % on
  the app world and 1.97 → 4.33 % processes-on, the largest rise.
- **B9** (app world, seed 483920, 5 interleaved runs): off 3.944 s
  (3.493..4.194), on 4.335 s (3.593..4.930). The brackets overlap, so no
  difference is established.

**Q13, the second cause on processes-on 24601: the estuary cut.** Each
variant was built into the engine in a scratch run and restored after
(hash-checked). These are processes-on, seed 24601, 1-3-cell lakes against
a control of 49:

| Variant | lakes |
|---|---|
| as built | 169 |
| debris not written to regolith | 169 (no effect) |
| coastal rock factor off (`c = 0` in the pass only) | 99 |
| estuary cut off (`estuary_depth` 0) | **53** |
| wave and estuary cut off | 55 |

- So the wave cut adds almost nothing. The estuary cut adds about 50 lakes
  rock-blind and about 70 more through `ρ^c`. On this world `ρ^c` is up to
  3.16 on alluvium, because stream-power deposit over `R_EXPOSE` reads as
  unconsolidated rock (ρ 10).
- **Why the estuary cut can dig.** The cut applies to every cell whose
  discharge exceeds `w·h·0.001` and whose height is under
  `sea + estuary_depth`. At the default 0.08 that is about 552 m above sea
  (0.08 × 4 000 m / 0.58, arithmetic; sea 0.42, `peak_m` 4 000). So it acts
  along every large river far inland, not only at the mouth. It scales
  with `ln(flow)`, which jumps where a tributary joins. Cells along a
  channel are therefore lowered by different amounts, and some end below
  their downstream neighbour: a closed hollow. That is inferred from the
  variant table, not traced cell by cell.
- **Not fixed.** A fix confined to processes-on worlds would have to change
  only the rock kernel's estuary term. Even rock-blind, the estuary cut
  leaves 99 against 61 (B8's 1.25 × 49), so no change to the rock factor
  alone passes. The fix that would work changes the reference estuary term
  in both kernels, which reaches beyond processes-on worlds. Ruling CH
  allows that only on a ruling, so it is reported, not built.

**Owner questions, each with a default.**
14. **The estuary cut's reach.** Limit it to cells near the sea in
    physical terms (for example a fixed height or distance band, scaled
    with `peak_m` / extent), or cap its per-cell drop to the downstream
    neighbour's height so it cannot dig, in both kernels, with the golden
    re-recorded and disclosed. Default: **the downstream cap**: it removes
    the mechanism without changing where estuaries form. Not measured.
15. **B7 on 24601 and 71077345** (0.155 and 1.488 against 3). Default:
    **leave B7 failing and the flip off**; with 2 of 4 seeds the rock
    signal is real but weak at the default `wave_str`.

### GF-4: the estuary cut capped (2026-10-13, Ruling CH addendum 2)

*Design and evidence, not status.* Q14 took its default and Q15 was
accepted (B7 may fail on 24601 and 71077345; the coastal flip stays off).
**Built** in both kernels: the estuary cut is capped at the cell's drop to
its lowest of eight neighbours on the pass-start surface (x wrapped on a
world map), so it can lower a cell to its downstream neighbour but never
below. A cell lower than all eight is not cut.

**Measured first** (`gf4_bars` with the cap removed and with it; 2048 ×
1311, 800 km, coastal on; 1-3-cell lakes, control → as built → capped):

| Seed | app world | processes-on world |
|---|---|---|
| 483920 | 34 → 32 → 33 | 43 → 41 → 40 |
| 24601 | 68 → 112 → **65** | 49 → 169 → **59** |
| 71077345 | 30 → 24 → 22 | 27 → 26 → 21 |
| 12345 | 89 → 75 → 74 | 87 → 76 → 74 |
| 314159 | 62 → 71 → 62 | 37 → 39 → 38 |

- **B8 now passes on all five seeds**, on both worlds. Ocean on paths
  stays 0 and lake share stays within +2 pp (the largest is 24601's app
  world, 1.61 → 2.18 %).
- **B7 (revised), capped:** 483920 1.027 → 3.234 (PASS), 71077345
  1.486 → 5.614 (PASS; it failed before the cap), 12345 1.104 → 3.231
  (PASS), 24601 0.104 → 0.230 (FAIL, as Q15 accepts). 314159 is not
  measurable. **3 of 4.** The controls fell because the cap removes rock-blind
  loss along rivers, which sharpens the weak/strong contrast.
- **B9** was not re-run: the cap adds an eight-cell minimum on estuary
  cells only, and the coastal pass is off in the app.
- **Goldens:** `golden_parity_passes.rs` `coastal_process_case_0` (2 of 80
  cells) and `case_2` (36 of 108; 12 higher and 24 lower, since over 4
  passes a smaller cut changes what the later passes see) were
  re-recorded. Cases 1 and 3 are unmoved. Parity, app and processes-on
  worlds hash identical to `e7948205`.
- **Tests:** `the_estuary_cut_never_digs_below_the_lowest_neighbour`
  (both kernels, literal heights); 2 mutants (cap dropped, floor dropped),
  both killed.
- **Frames (taken 2026-10-13, ~~not taken~~).** The comparison is the capped
  build with coastal forced on against the coastal-off control (no pre-cap
  DLL existed), seeds 24601 (app and processes-on worlds) and 314159, fit and
  three crops, all opened:
  `cartalith-native/docs/evidence/gf4-estuary-2026-10-13/`. No closed hollow
  or new artefact at the river mouths (314159's mouth crop is identical on
  and off; 24601's differs by a few bank cells). 314159 at fit is nearly
  unchanged. On 24601 the coastal pass itself (not the cap) fragments the
  central island's coast and ragged-edges the large south-east inland
  lake. One thing looked off and is reported, not diagnosed: the inland
  lake crop shows small round dimples on the land beside the lake when
  coastal is on, absent in the control.

### GF-5 measured and built (2026-10-17)

*Design and evidence, not status.* Opened at the symbols first: before this
change no dissolution pass and no permeability term existed
(`ROCK_PROPS` carried `soluble` and `permeability` and nothing read them),
GF-4's last change was `a8a8b1ce`, and `relief_model`, `geology_processes`
and the coastal pass were all off in `params::defaults()`. GF-5 has no owner
question of its own in §9, so nothing blocked it. Measured without
construction (Ruling CG), as GF-4 was.

**Built** (`cartalith_erosion::karst`), behind a new switch,
`WorldParams::geology_karst`, off at both boundaries, acting only with the
rock gate (`geology_model` and `geology_processes`) and on the light pass's
path (`carve_rivers`), where §3.1 puts it. Its own switch so that a
processes-on world without it stays the pre-GF-5 world. It has `PARAMS` and
`JS_PATHS` rows (port-only, no reference path), no GUI control, and a save
without the key reloads with it off.

- **Dissolution** (`karst_dissolution`), after the threshold hillslope and
  before glacial: on land whose exposed rock is soluble (limestone),
  `Δz = −min(k_k·rain·τ, soluble thickness)`, §4.12's budget. Soluble
  thickness is the surface's height above an insoluble contact when the
  limestone is the cap; otherwise unbounded. Two judgements are added:
  the pass never takes a cell to or below sea (the water table is the base
  level for solution, and it keeps the pass from making sea, which B8's hard
  guard forbids), and nothing rebounds after it. τ is
  `GeoClock::karst_budget_factor`, exactly 1 at τ = 1 or with the clock
  off. The lowering strips regolith first (§4.9).
- **Runoff** (`effective_runoff`): the channel network's flow accumulation
  is seeded with `rain·(1 − φ·P(exposed))`. Only that accumulation reads it:
  the light pass, the coastal pass and the post-carve climate refresh still
  read the rain.
- **`P`** (`permeability_fraction`), a judgement: §2.3's ordinal classes
  spaced evenly, very low 0, low 0.25, moderate 0.5, high 1. `Variable`
  (unconsolidated) is 0: the model does not know whether it is sand or clay,
  so it claims no loss there.
- **`k_k` = 15 m** at unit rain (`KARST_DEPTH_M`) and **`φ` = 0.5**
  (`PERMEABLE_RUNOFF_LOSS`). Both are judgements, chosen by B5 and B8 as
  §4.6 says, **after** seeing the results below. 20 m failed B8 on 71077345
  (1-3-cell lakes 27 -> 34 against a bar of 33.75). 10 m passed B8, but B5's
  depression half had fewer than 100 cells on two seeds, so it could not be
  read there. φ was tried at 0 and 0.5 only. The runoff term alone (k_k = 0,
  φ = 0.5) kept B8 on the three seeds tried (24601, 71077345, 12345); the
  small-lake rise at 20 m came from the dissolution term.
- **Spring line** (`spring_line_potential`, `is_spring_pair`): a land cell
  of a two-layer column whose cap is permeable (`P ≥ 0.5`) over an
  impermeable substrate (`P ≤ 0.25`), which exposes its cap and has a
  4-neighbour exposing its substrate. That is B4's breach-line definition
  limited to these pairs. It is **derived on demand, not stored**, so it
  cannot disagree with the column. No landmark read it when GF-5
  landed; GF-6's *spring* reads it (see "GF-6 measured" below).

**Measured** (`cartalith-godot/tests/gf5_karst.rs`, `gf5_bars`; 2048 × 1311,
800 km, processes on, relief off; control is the switch off, treatment the
switch on). B5 is the pass replayed alone on the control world's final
surface, column and rain, because its input inside `generate_terrain` is
not observable from a `WorldState`, as with GF-4's B6 and B7:

| Seed | B5 lowered (all soluble) | B5 new depressions > 0.004, % soluble | B8 ocean · lake % · 1-3-cell lakes, off → on | Spring-line cells |
|---|---|---|---|---|
| 483920 | 28 860 (mean 4.05 m) | 1 cell: not measurable | 0 → 0 · 0.977 → 0.956 · 43 → 45 | 4 749 |
| 24601 | 56 106 (2.96 m) | 139, 100 % | 0 → 0 · 1.966 → 1.904 · 49 → 50 | 6 645 |
| 71077345 | 37 112 (10.81 m) | 1 202, 100 % | 0 → 0 · 0.660 → 0.920 · 27 → 27 | 6 287 |
| 12345 | 58 192 (6.02 m) | 662, 100 % | 0 → 0 · 3.855 → 3.934 · 87 → 97 | 4 520 |
| 314159 | 56 327 (4.60 m) | 370, 100 % | 0 → 0 · 10.049 → 8.261 · 37 → 39 | 5 557 |

- **B5's exact half passes on all five**: every lowered cell exposes
  soluble rock (asserted in the harness, and pinned by literal in
  `karst::tests`, where the σ-drop mutant is killed). **B5's depression half
  passes on the 4 seeds where it can be read**; on 483920 the pass makes 1
  such cell, under §5.1's population of 100, so that seed is "not
  measurable", **not a pass**. The 100 % is partly by construction: lowering
  only soluble cells can only lower a fill level, so no insoluble cell can
  become newly flooded.
- **B8 passes on all five.** The largest small-lake rise is 12345's 1.11 ×;
  the largest lake-share rise is 71077345's +0.26 pp.
- **River cells per exposed rock** (`river_mask`, off → on) fall on the
  high-permeability rocks: oceanic basalt by 9 to 42 % on the four seeds
  where it carries rivers, limestone by 16 % (483920) and 30 % (71077345),
  but up 0.7-2.1 % on the other three. Granite gains on four seeds (−4.0 to
  +11.1 %), shale on three (−13.3 to +10.9 %). Why low-permeability rock
  gains river cells is **not established**: `river_flow_thresh` depends
  only on the grid and the extent, so it is not a relative threshold.
- **The spring line is non-empty on all five seeds** (4 520 to 6 645 cells).
- **B9 was not measured.** The pass is off in the app, and GF-5's done-means
  does not name B9.

**Identity.** Parity, app and processes-on worlds (seeds 483920 and 314159,
512 × 328, 13 arrays: field, rainfall, temperature, flow discharge, stream
order, river mask, river floor, channel `chan`, `recv` and intensity,
regolith, contact, rock top) hash identical between HEAD `03ab5caa` and
this change (`gf5_identity_hashes`, printed by both builds and diffed). No
golden moved, so nothing is re-baselined.

**Tests.** `karst::tests` (6, literal heights and runoffs) and
`cartalith-engine/tests/geology_gf5.rs` (4: the switch is inert with the
processes off at 256 × 164 and 512 × 328, and with the model off; it moves a
processes-on world deterministically; dissolution lowers off-river limestone
and permeable rock loses channel cells; τ in the budget, by literal), plus
`params_mapping.rs`'s save-key test. **Mutants: 12 run, 12 killed.** The
first run had one survivor, the dissolution gate made to ignore the rock
gate: the 256 × 164 fixture exposes no limestone, so a leaking gate had
nothing to dissolve. That was a real test gap. The processes-off test now
also runs at 512 × 328 and kills it.

**Not done.** Nothing moves in the app, and the processes stay off. GF-6
reads the spring line and the karst-lowered cells (*spring*, *cave*; see
"GF-6 measured" below). The
interactive re-erode (`erode_bridge`, §4.13's painting chain) does not run
karst yet; §4.13 puts it in GF-8's chain.

---

### GF-6 measured, and what of it is built (2026-10-17): the landmark half

*Design and evidence, not status.* Opened at the symbols first: before this
change `kinds()` shipped *cave* `buildable: false` ("nothing in it carves
hollows"), `pool_spring` read only the channel tree, `pool_rock_formation`
read `resistance_field`, `pool_waterfall` read no knickpoint, the river carve
(`carve_channel_network`) had no `ω`, and the LOD amplifier had no rock
style. So GF-6 was unbuilt in all three parts (§4.7, §4.10, §4.11). GF-6 has
no owner question of its own in §9, and its done-means names no numeric
measurement gate for the landmarks, so nothing blocked the landmark half.
Landmarks are exempt from parity (`DECISIONS.md` §7q).

**Built: cave and spring read GF-5** (`cartalith-civ/src/landmark/karst_view.rs`,
`landmark.rs` `pool_cave`, `pool_spring`).

- **`KarstView::for_world(ws, p)`** derives GF-5's facts on demand, through
  GF-5's own functions (`karst_dissolution`, `spring_line_potential`), so
  there is one copy of each rule. The dissolution is the pass **replayed on
  the finished surface and rain** with the engine's `k_k` and τ, as GF-5's
  B5 replayed it, because the pass's own input surface is not observable
  from a `WorldState`. So it reads where the pass acts on the world as
  finished. It is not a record of what the pass did. It **refuses**, with
  a reason, unless the world has a column and was generated with
  `geology_processes` and `geology_karst` both on, which is the engine's own
  gate. The app ships `geology_karst` off, so every app world refuses.
  The refusal is `NoTerrain` with zero candidates for cave, and the
  unchanged hydrology spring.
- **Cave** (now `buildable: true`; its `not_built` reason is removed, as
  §4.11 says): the steepest cell of its face within the fine window, on
  land the pass dissolves (`dissolution_m > 0`), with local relief of at
  least 40 m. It is scored on dissolution depth 0.45, face slope 0.30 and
  relief 0.25. The 40 m and the weights are judgements (Category C, no
  source).
- **Spring** with the view gains GF-5's spring line as a second domain. A
  spring-line cell that a fed channel runs through is excluded, because that
  is the river crossing the contact. A fourth term, the contact, takes 0.30;
  the old three are scaled by 0.70. That is a judgement. Without the view
  the domain, the facts and the terms are the old ones, by control flow.

**Measured** (`cartalith-civ/tests/gf6_karst_landmarks.rs`, `gf6_measure`;
the app's parameters at 2048 × 1311, 800 km, processes on, relief off,
karst on; default landmark settings):

| Seed | Dissolving cells (mean m) | Limestone-exposed land | Spring line | Cave cand / constraint / placed | Spring cand off → on (placed 30 both), on the line | Other kinds: placed off, lost with the view |
|---|---|---|---|---|---|---|
| 483920 | 28 606 (3.61) | 46 671 | 4 749 | 35 / 0 / 12 (at cap) | 495 → 5 110, 30 of 30 | 144, 5 |
| 24601 | 55 443 (2.61) | 68 345 | 6 645 | 53 / 0 / 12 | 603 → 7 107, 30 | 160, 3 |
| 71077345 | 38 158 (9.00) | 55 843 | 6 287 | 27 / 0 / 12 | 507 → 6 653, 30 | 138, 7 |
| 12345 | 59 483 (6.21) | 74 956 | 4 520 | 59 / 0 / 12 | 501 → 4 900, 30 | 142, 10 |
| 314159 | 56 688 (4.37) | 55 577 | 5 557 | 43 / 0 / 12 | 565 → 5 923, 30 | 152, 3 |

- **Why limestone shows more cells than dissolve:** checked on 483920 and
  24601. On 483920, 7 191 cells have no rain. Another 10 907 have rain above
  zero but lower by less than f32 resolution; the five sampled had rain
  under 1e-6. One is a cap already dissolved to its contact. On 24601,
  14 593 have no rain and 209 are caps at their contact. None is at the sea
  floor.
- **Every placed spring moves onto the spring line on all five seeds.**
  Contact cells outnumber channel heads about ten to one, and the contact
  term ranks them first. That follows from the judgement weight, and **it
  is the thing to look at when a karst world is first shown.** A lower
  contact weight, or limiting the contact domain to channel heads, would
  keep more channel-head springs.
- The cave relief minimum rejected nothing at 2048. The fixture test
  `a_cave_needs_dissolving_rock_and_a_face_with_relief` is what proves it
  is live.
- Placing caves and moving springs costs other kinds 3 to 10 placements per
  seed, through the cross-kind spacing.

**Identity.** `cartalith-civ/tests/gf6_identity_hashes.rs` runs app worlds
(seeds 483920 and 314159, 512 × 328, karst off) through the full default
landmark pass. It printed the same hash for every placement and every funnel
row except cave's, at `d718d052` and with this change (`603d785fc1edf96e`,
`936d88e055902d42`; 170 and 168 landmarks). Cave's row is `not_buildable` →
`no_terrain`, 0 candidates. No terrain code changed, so no golden can move.

**Tests:** 3 in `gf6_karst_landmarks.rs` (the refusals by gate; cave and
spring on a real karst world; the k_k and τ scaling), and 2 in
`landmark::tests` (the cave fixture; the weights by literal). The kind
table now counts 31 buildable kinds. **Mutants: 12 run, 12 killed.** They
were the dissolution domain test, the relief minimum, the fed-channel
exclusion, the contact weight, the contact read, the grid check, the
`detect` arm, the terms branch, both gates, the clock factor and `k_k`.

**App wiring.** `cartalith-godot`'s `with_landmark_inputs` passes
`KarstView::for_world(ws, &self.recompute_params())`. It refuses before it
computes anything, so app worlds pay nothing. `_gf6cave_probe.tscn`
(windowed, seed 483920, 512 × 328, 800 km) passed. With the app's
parameters, cave is `no_terrain` with 0 candidates. After `set_params`
turns `geology_processes` and `geology_karst` on, the same world gives 16
cave candidates and places 7 (limited by spacing), each chain naming
limestone the karst model dissolves. The shell's `no_terrain` tooltip
(`LM_LIMIT_WHY`) now also covers the absent-input case, which is what an
app world's cave always reports.

**Not done in GF-6:**
- §4.7 (the carve's `ω`, and knickpoints for *waterfall*), §4.10 (rock-styled
  LOD detail, and its windowed probe) and §4.11's *rock formation* reading
  `s`. None was built by that change. §4.7 and rock formation are built in
  the next section; §4.10 is still open.

### GF-6 measured: the carve and rock half (2026-10-18)

*Design and evidence, not status.* Opened at the symbols first. Before this
change the carve's half-widths (`generate_terrain`, `half_ws`) read only
Strahler order and `width_k`, and nothing computed a knickpoint.
`pool_rock_formation` read `LandmarkInputs::resistance` (`resistance_field`).
`pool_waterfall`'s `WATERFALL_TERMS` comment said §7's `R` (lithological
contrast) was dropped because no contrast field existed. So §4.7 and §4.11's
rock formation and waterfall rows were unbuilt. Nothing blocked them: GF-6
has no owner question in §9, and its done-means names B8 and RV-1's tests as
the only carve gates. **§4.10 (rock-styled LOD detail and its windowed
probe) was not touched.** It lives in the renderer's tiled-detail code, which
another lane is re-keying for zoom detail; it waits on that re-key.

**Built: the carve's `ω`** (`cartalith-erosion/src/carve_width.rs`, wired as
`RockContext::carve_widths` in `cartalith-engine`).

- `valley_width_factor(rock) = 1 + 1.0·(0.60 − s)`, read from §2.3's `s` (so
  there is no second property table). It runs from 0.75 (granite) to 1.50
  (unconsolidated): schist and sandstone 1.00, limestone and andesite 0.90,
  basalts 0.85, gneiss 0.80, tuff 1.25, shale 1.30. The pivot and the slope
  are **judgements**. Montgomery & Gran (2001) is §4.7's citation for the
  direction; no source was found for the magnitude.
- `ω(run)` is the median of the factor over the run's **land** points, read
  on the surface handed to the carve. A run with no land point keeps its
  half-width (there is no rock to read, and 1.0 would stand in for "no
  value").
- §4.7's formula is applied as written: `ω` multiplies the capped
  half-width, so a weak-rock valley can exceed `4·width_k`.
- `drop`, `CARVE_LAND_MARGIN` and `CARVE_KEEP_LAKE_DEPTH` are untouched:
  `carve_channel_network` itself did not change.
- **Behind a new switch, `WorldParams::geology_carve`, off at both
  boundaries**, acting only under the rock gate (`geology_model` and
  `geology_processes`) on the `carve_rivers` path. `PARAMS` and `JS_PATHS`
  rows; a save without the key reloads off.
- RV-3's smooth valley shade (`cartalith-godot/src/valley_shade.rs`) reads
  the drawn runs' widths, not the carve's `half_ws`, so it does **not** use
  `ω` yet. §4.7 says it should. It is not built here: it is renderer code,
  and with the switch off in the app nothing it shades has moved.

**Built: knickpoints** (`carve_width::knickpoints`). A land channel cell
that exposes its column's cap, whose receiver is land exposing its own
column's substrate, where the cap's `s` is above that substrate's. The
strength condition is a **judgement**: §4.7 says only "crosses from cap to
substrate", and a weak cap over strong rock gives no reason for a step. It
is derived on demand, like GF-5's spring line, so it cannot disagree with the
column. It does not depend on `ω`.

**Built: rock formation reads `s`, and waterfall reads the knickpoints**
(`cartalith-civ/src/landmark/rock_view.rs`, `landmark.rs` `pool_rock_formation`,
`pool_waterfall`).

- `RockView::for_world(ws, p)` gives the exposed rock's `s` on any world with
  a column, the app's included, and the knickpoints only when the world was
  generated with `geology_processes` on. Without the processes the erosion
  never cut weak rock back faster than strong, so no step at a contact is
  claimed; the reason is kept (`KNICK_PROCESSES_OFF`). With no column the
  view refuses (`RockAbsent::NoColumn`, with the column's own reason).
- **Rock formation** now reads the view only. `LandmarkInputs::resistance`
  is removed, and so is `LandmarkInputs::lithology`, whose only reader was
  rock formation's fact string. With no view it is `NoTerrain` with 0
  candidates; it does not fall back to `resistance_field`. The domain is the
  local maximum of a key that is strength first and fine TPI second, because
  `s` is categorical and a whole outcrop ties on it. The thresholds keep
  their values: `ROCK_FORMATION_MIN_RESISTANCE` 0.55 (on `s` this admits
  moderate and strong rock and refuses tuff, shale and unconsolidated) and
  contrast 0.20 (one Selby class).
- **Waterfall** keeps its domain and constraints. With knickpoints, a fifth
  term (1 on a knickpoint) takes 0.30 and the old four are scaled by 0.70
  (`WATERFALL_TERMS_ROCK`; a judgement, as GF-6's spring weight is). A
  waterfall on a knickpoint names both rocks. Without knickpoints (every app
  world) it is the old detector by control flow.
- `cartalith-godot`'s `with_landmark_inputs` passes the view
  (`recompute_params()`, as for `KarstView`).

**Measured, the carve** (`cartalith-civ/tests/gf6_carve.rs` `gf6_carve_bars`;
the app's parameters at 2048 × 1311, 800 km, processes on, relief off,
control `geology_carve` off):

| Seed | B8 ocean on paths | B8 lake % of path | B8 1–3-cell lakes | Field cells moved | Carved cells, strong rock (ω < 1) | Carved cells, weak rock (ω > 1) | Knickpoints off → on |
|---|---|---|---|---|---|---|---|
| 483920 | 0 → 0 | 0.977 → 0.952 | 43 → 45 | 56 133 | 58 749 → 40 454 (−31.1 %) | 9 073 → 8 006 (−11.8 %) | 76 → 73 |
| 24601 | 0 → 0 | 1.966 → 1.953 | 49 → 58 | 48 888 | 65 988 → 52 197 (−20.9 %) | 11 046 → 9 916 (−10.2 %) | 90 → 90 |
| 71077345 | 0 → 0 | 0.660 → 0.640 | 27 → 26 | 53 580 | 59 279 → 42 081 (−29.0 %) | 11 221 → 10 284 (−8.4 %) | 91 → 92 |
| 12345 | 0 → 0 | 3.855 → 3.891 | 87 → 88 | 43 809 | 44 221 → 31 894 (−27.9 %) | 11 464 → 12 261 (+7.0 %) | 103 → 103 |
| 314159 | 0 → 0 | 10.049 → 10.025 | 37 → 46 | 54 521 | 58 053 → 43 857 (−24.5 %) | 10 670 → 10 108 (−5.3 %) | 99 → 101 |

- **B8 passes on all five seeds.** The worst small-lake ratio is 314159's
  1.24× against the 1.25× bar, which is close. RV-1's `carve_network_*`
  tests are green (workspace run below).
- Carved cells are counted per exposed rock on the control world's surface.
  **Strong-rock valleys narrow on every seed (−21 to −31 %). Weak-rock carved
  cells grow on only 1 of 5 seeds**, and shrink 5–12 % on the other four.
  That follows from the median: the rivers here run mostly through granite
  (ω 0.75), so a shale or alluvial reach inside a granite run takes the
  run's narrow width. Per-rock widths would need a per-point factor, which
  §4.7 does not specify. At 512 × 328 on seeds 12345 and 483920 together the
  weak-rock count rises (935 → 1 215), which is what
  `geology_gf6.rs` asserts.
- The pivot at s = 0.60 therefore narrows the average valley on these worlds:
  carved cells fall overall. Re-centring the pivot on a world's own
  river-rock median would keep the average width and only redistribute it.
  That is an owner question (below); the switch is off, so nothing ships
  either way.
- B9 (cost) was not measured. The change adds one median per run.

**Measured, the landmarks** (`cartalith-civ/tests/gf6_rock_landmarks.rs`
`gf6_rock_measure`; 2048 × 1311, 800 km, default landmark settings):

| Seed | Rock formation, app world: cand / constraint / placed | Rocks it names | Knickpoints, processes on | Waterfall passing constraints | Waterfalls on a knickpoint, term off → on (placed unchanged) |
|---|---|---|---|---|---|
| 483920 | 2 972 / 2 800 / 20 (cap) | granite, oceanic basalt | 76 | 255 | 1 → 1 (10) |
| 24601 | 4 389 / 4 177 / 20 (cap) | granite, oceanic basalt | 90 | 149 | 1 → 1 (10) |
| 71077345 | 2 818 / 2 603 / 20 (cap) | granite | 91 | 106 | 0 → 1 (8) |
| 12345 | 2 243 / 2 097 / 20 (cap) | granite, oceanic basalt | 103 | 158 | 4 → 4 (9) |
| 314159 | 3 778 / 3 557 / 20 (cap) | granite | 99 | 311 | 0 → 0 (19) |

- **Rock formation now places on app worlds.** At 512 × 328 (the identity
  check below) it placed 0 before this change (all 94 and 102 candidates
  failed the resistance constraints) and places 14 and 15 after.
- **The knickpoint term barely moves waterfalls**: on one seed of five it
  moves one placement onto a knickpoint. Knickpoints are 76–103 cells a
  world, and few of them pass waterfall's 40 m drop and 6 % gradient at
  about 0.4 km a cell. That is the measured state, not a tuning target; the
  constraints were not relaxed.

**Identity.** `gf6_carve_identity_hashes` printed the same thirteen-array
hashes for the parity, app and processes-on worlds (seeds 483920 and 314159,
512 × 328) at `bfadd3fb` and with this change, so the switch off moves no
array and **no golden can move**. The landmark identity
(`gf6_identity_hashes.rs`, app worlds) moves **by design**, because rock
formation reads `s` on every app world: 483920 goes from 170 to 184
landmarks (rock formation 0 → 14), 314159 from 168 to 183 (0 → 15). Every
other kind's placed count is unchanged on both seeds; the hashes are
`5c6025b745ecfd41` and `0e7bc85117cc1e9a` (were `603d785fc1edf96e` and
`936d88e055902d42`). Landmarks are parity-exempt (`DECISIONS.md` §7q).

**A measurement hazard met on the way.** A second worktree at HEAD built into
the same `CARGO_TARGET_DIR` as the change's worktree, and cargo then reused
one test binary for both, so a run of the change printed HEAD's numbers
until the sources were touched. The figures above come from runs after a
forced rebuild. Two worktrees need two target directories.

**Tests.** `carve_width::tests` 4 (every rock's factor by literal; the
median, the land filter and the refusal; the scaling; every knickpoint
condition), `cartalith-engine/tests/geology_gf6.rs` 2 (the gate; weak
widens and strong narrows, deterministic), `params_mapping.rs` 1 (the save
key), `landmark::tests` 4 (rock formation refuses without the view;
a weak outcrop is refused; a waterfall on a knickpoint outranks one off it;
the weights and thresholds by literal), `gf6_rock_landmarks.rs` 2 (the view
by gate; formations on a real world stand on rock of `s` ≥ 0.55). The
fixture test `rock_formation_needs_contrast_not_just_high_strength` now uses
a granite-in-shale view. **Mutants: 15 run, 15 killed** (two survived first
and were killed by new tests: the same-rock receiver column and the strength
floor).

**Owner questions, each with a default:**
1. **Re-centre `ω` on each world's river-rock median?** Today's pivot
   (s = 0.60) narrows the average valley 21–31 % in strong rock on these
   worlds. Default: **leave it**; the switch is off, and the scope's own
   formula has a fixed per-rock factor.
2. **A per-point `ω` instead of the run median**, so a shale reach widens
   inside a granite run? Default: **no**; §4.7 specifies the median.
3. **Switch `geology_carve` on with the processes** when they are switched
   on? Default: **yes, together**; it acts only with them, and B8 passes.

**Not done in GF-6:** §4.10 (rock-styled LOD detail and its windowed probe),
waiting on the zoom-detail re-key; RV-3's valley shade reading `ω`.

### GF-6 measured: rock-styled LOD detail, §4.10 (2026-10-18)

*Design and evidence, not status.* Opened at the symbols first: nothing in the
deep-zoom tile path read the rock column. `render_biome_tile_rgba_water` lit
and toned a tile from the kilometre law's heights alone, and
`lod_snapshot_inputs` carried the legacy 7-class lithology (a colour input),
not the GF-1 column. The zoom re-key (`1fc9208f`), the forest/rock fix
(`6c7f5ea9`, `micro_relief_tone`) and EF-9 had landed, so nothing blocked it.
§4.10 has no owner question in §9.

**Built** (`cartalith-godot/src/render.rs`'s "GF-6 §4.10" block,
`rock_detail_bridge.rs`, `lod_worker.rs`):

- `build_rock_detail`: per cell, the rock that §2.5's exposure rule gives on
  the finished field (regolith included), plus a **cap-edge** flag. A cell is
  on a cap edge when it has a cap over a substrate and, among it and its
  eight neighbours that also have one, at least one exposes its cap and at
  least one its substrate. That is the reading of "within one coarse cell of
  a cap edge" (a judgement).
- `rock_style`, §4.10's table:
  - `α = 0.6 + 0.6·s` (granite 1.11, limestone 1.02, shale 0.78,
    unconsolidated 0.66). A judgement; Selby gives the direction.
  - **Ridged** for massive or columnar jointing.
  - **Terraced** for bedded rock on a cap edge. **Fissile rock (shale) is
    read as bedded**, with steps a quarter of the ladder's.
  - Everything else keeps the law's own relief (fbm), scaled by `α`.
  - The weights are blended bilinearly between cells (`rock_style_at`), so no
    cell boundary draws as a square (Ruling CG addendum 2's reason).
- `rock_styled_height`: `r = h − c` is the sub-cell relief the law added.
  - Ridged folds it to `−|r|`, and `α` scales it.
  - The terraces then add the offset of a step function, read at
    `c + 0.4·(h1 − c)`, on a ladder of 256, 64, 16 and 4 m.
  - The step function (`terrace_height`) is benches with a 0.3 riser.
- `rock_styled_tile` restyles every land pixel of the padded tile and clamps
  a land pixel to stay at or above sea level.
- In `render_biome_tile_rgba_water`, the styled tile is what the tile is
  **lit and toned** from: the macro and meso normals, the R2 crest and the
  micro-relief tone.
  - Everything else keeps the unstyled tile: the water height `wt`, the
    elevation colour `ht`, the cryosphere and micro-band residual, and every
    material decision.
  - So by construction a terrace **cannot move a coastline or a lake**.
    That is §4.10's numeric bar, and a test asserts it on rendered pixels.

**Departures from §4.10's formula, each made after looking at frames:**

1. **Ridging is gated by slope.** It fades in from 4° to 12° of grid-scale
   slope (`ROCK_RIDGE_SLOPE_DEG`).
   - Massive rock is 55-75 % of land on these worlds. Folding every granite
     cell drew a closed crease network over gentle lowland, like crazy
     paving.
   - Granite under gentle ground weathers to domes and grus, not crests
     (Twidale 1982).
   - Granite slopes measure p50 0.36-0.39°, p90 1.3-2.1°, p97 6.4-10.3° and
     p99 28-32°. So the ridged style acts on roughly the steepest 3-5 % of
     granite.
2. **Terraces are band-passed in screen pixels.** An octave fades in as its
   bench spacing grows from 6 to 14 px, and fades out from 40 to 120 px.
   - Benches closer than about 6 px read as hatching. Benches far apart only
     flatten a slope.
   - So each zoom step draws the finest bench scale that reads, which suits
     the detail-at-every-zoom requirement (Ruling CI item 1).
3. **The ledges are added on top of the whole fine relief**, not cut through
   it. Replacing the relief with the ledges read better on steep scarps. In
   the shell, though, it smoothed gentle cap edges at x64, which loses detail
   on zoom.

**Gate, and identity.**
- The tiles are styled only on a **generated world whose processes read
  rock**: `geology_model` and `geology_processes`, read from
  `recompute_params()`.
  - This is GF-6's knickpoint gate, for the same reason: on a rock-blind
    world the surface crosses the rock only by chance.
  - App worlds have the processes off, so they get no `RockDetail`. Their
    snapshot fingerprints and tiles are unchanged by control flow.
  - A loaded save and an import have no column.
- `WorldGen::set_rock_lod_detail` is a view switch: on by default, never
  saved. It exists so that a probe can flip the styling on one world.
- `lod_cache_key` gains `rd`, whether the tiles are styled.
- The snapshot fingerprint folds in the detail, behind a separator word, only
  when the detail is present.

**Measured: the zoom ladder** (`cartalith-godot/tests/gf6_rock_ladder.rs`,
`#[ignore]`d).

- **World:** the app's `params::defaults()` with `geology_processes` on,
  2048 × 1311, 800 km, Natural Vibrant, the shipped tile path.
- **Seeds:** 483920, 24601 and 71077345.
- **Targets:** four per seed, picked by inputs: the steepest inland granite,
  the steepest bedded cap edge, the steepest fissile cap edge, and the
  steepest unconsolidated or schist cell.
- **Rungs:** x4, x16 and x64, with styling off and on.

The statistics come from the PNGs (`stats.py` in the run's scratch, not
committed):
- the mean of |∇ luma|;
- the number of unique RGB triples;
- the **sub-cell share**: the variance of luma minus a box mean over one
  coarse cell, over the total variance. This is **not** the assessment's
  definition, so it is only compared between the two arms.

Medians over the 12 crops:

| Rung | Gradient, off → on | Unique colours, off → on | Sub-cell share, off → on |
|---|---|---|---|
| x4 | 1.97 → 1.98 | 44 874 → 45 054 | 0.24 % → 0.24 % |
| x16 | 4.34 → 4.31 | 48 845 → 49 112 | 3.87 % → 3.90 % |
| x64 | 7.77 → 8.45 | 71 274 → 69 387 | 57.36 % → 56.87 % |

- **Detail still rises at every rung on every crop with styling on.** On,
  x16/x4 is 1.69-2.47 and x64/x4 is 3.50-7.33. Off, they are 1.70-2.55 and
  3.55-7.46.
- Granite crops gain gradient at x64: 7.72 → 8.45, 6.57 → 7.00 and
  6.42 → 7.02.
- Two kinds of crop lose a little:
  - the weak crops (11.27 → 10.79; `α` is 0.66 by design);
  - one cap-edge crop (71077345: 5.33 → 5.19).
- Unique colours at x64 fall by up to 3 %.
- At x4 the styling moves 6-11 % of pixels, by a mean of 0.1 levels.
- Shares of land:
  - massive or columnar rock, before the slope gate: 55.7-75.5 %;
  - cap edge: 0.77-1.01 %;
  - terraced: 0.74-1.00 %.

**Frames read** (`cartalith-native/docs/evidence/gf6-lod-2026-10-18/`; 01-07
are from the Rust ladder):
- **Steep granite** (the walls of the river carve) has crisper crags. At x64
  on s24601 the crests form a cell-like crease network: stylised, and read
  as such.
- **Flat granite plains** are unchanged.
- **Cap-edge walls** carry thin ledge lines along the wall on s24601 and
  s483920. The s71077345 cap-edge crop barely changes.
- **Near-flat fissile ground** is a problem. The shale's fine steps draw
  closed contour rings (05, lower left). They read as a fingerprint, not as
  strata. This is recorded, not tuned away.
- **Unconsolidated ground** is visibly calmer.

**The windowed probe** (`_gf6lod_probe.gd`, in the shell, seed 24601, 2048 ×
1311) passes **15 checks with 0 failures**.
- **Phase A**, the app's world (processes off):
  - the styling is inactive;
  - flipping the switch moves 0 px at x1, x16 and x64 on both targets;
  - its six frames are pixel-identical to the same probe run on the HEAD DLL
    (`26ac5156`).
- **Phase B**, the same seed with the processes on, framing the engine's own
  picks (`rock_lod_detail_stats`):
  - the x1 base map is unmoved (0 px), since the styling acts in tiles only;
  - x16 and x64 move 58-82 % of pixels: the positive control.

Read honestly, **the effect in the shell is subtle** (08-09).
- The shell's tiles shade RV-3's valley field, which fills in the carved
  channel walls where the Rust ladder shows crags and ledges.
- So the steepest granite and the cap edges the probe frames are gentle
  ground. There the ridged style does not act (the slope gate), and the
  ledges are small next to the law's relief.

**Tests.**
- `cartalith-godot/tests/gf6_rock_detail.rs`, 11 tests:
  - the style table, by literal, for all 11 rocks;
  - the terrace law: benches, risers, continuity and monotonicity;
  - identity when neutral, and the ridged fold;
  - terraces fading in with zoom, and out when far apart;
  - monotone at every zoom;
  - the cap-edge flag on a hand-built column;
  - the ridge slope gate, worked by hand;
  - a rendered km-law tile: rock moves, water never does;
  - off without the km law: under `js_reference()` and at law 0;
  - a mismatched detail is ignored.
- `lod_worker::tests::a_snapshot_with_rock_detail_styles_and_keys_its_tiles`:
  the fingerprint tag, the tile, and the reference schedule.
- **Mutants: 19 run, 19 killed** (style table, ridge fold and slope gate, terrace riser, fade-in, fade-out and normalisation, cap-edge rule, fissile step, the styled-tile hand-over, the km-law gate, the builder's length check, the fingerprint tag, and the worker's attachment).

**Not done:**
- B9 (cost) was not measured. For each land pixel of a tile, the styling does
  a bilinear style read, a coarse sample, a coarse gradient and up to four
  terrace octaves. Only processes-on worlds pay for it.
- `RockDetail` is rebuilt per snapshot (one exposure read per cell) and is
  not cached.

**Owner questions, each with a default:**
1. **Strata colour banding at cap edges.** In the shell the ledges are faint
   on gentle cap edges, and real bedded outcrops read mostly by colour bands
   along the contour. A zero-mean tone band would make them legible without
   touching relief. Default: **no**, not without a ruling, since §4.10
   specifies relief styles only.
2. **The fingerprint rings on near-flat shale.** Default: **leave them**.
   Gate the terrace by a minimum slope only if the owner reads them as wrong.
3. **The crease network on steep granite.** Keep it, or soften the fold (for
   example half `−|r|`, half `r`)? Default: **keep it**. It is the scope's
   ridged style, gated to steep ground.


---

### GF-8 measured: the engine half (2026-10-18)

*Design and evidence, not status.* Opened at the symbols first: no Rock
target in `paint_bridge`, no `rock_painted` anywhere, `tile_erode` with no
production caller, and **no geology save set at all**: `GeologyAbsent::Restored`
still reads *"opened from a save: rock columns are not saved yet"*, so §2.6's
five column rasters and the `geology` manifest member, which GF-1 was to
write, do not exist. GF-8's persistence row therefore needs the whole set,
not only `rock_painted.u8`.

What this slice adds, and what it leaves for the next two:
- `GeologyColumn::painted: Option<Vec<u8>>`, §2.6's mask in memory. `None`
  means nobody painted (absent recorded absent); `build_geology` writes
  `None`; nothing in any process reads it.
- `cartalith_erosion::tile::tile_erode_rock`: `tile_erode`'s pinned ring and
  seeded area through the shared core's rock arm, the κ slice of §4.13.
- `cartalith_engine::rock_paint`: `bake` (§4.13 step 2, values validated
  before any write) and `reerode`/`commit` (step 3, the differential over
  `W`). The chain is stream power via `tile_erode_rock` plus its rebound, the
  threshold hillslope, karst, glacial and coastal, each through the same
  `RockContext` method generation calls, with the clock's counts.
  `cell_m` is kept the world's by scaling `map_width_km` by the window's
  share of `gw`.
- `undo::ColumnSnapshot` on `UndoStep`, `HeightUndo::push_with_column` /
  `restore_with` (both or neither), and `undo_one_with`/`redo_one_with`,
  which the three production undo/redo call sites now use with
  `ws.geology.column_mut()`. The pre-GF-8 undo tests are unchanged and pass.
- **Not built:** the shell's Rock target, the three `PaintEditor` layers,
  `WorldGen::paint_commit`'s Rock branch, `_authored_inventory()`, the
  `DCC_SHELL_SPEC.md` §4.5.2 sentence, and the save set. `rock_paint::commit`
  has no caller outside its tests.

Three decisions taken here, each labelled:
- **A substrate dab on a single-layer cell with no cap thickness in the same
  commit is skipped and counted** (`BakeReport::sub_skipped`). The scope is
  silent; inventing a contact elevation would be the silent choice §4.13
  refuses for caps. Judgement.
- **Skips are counted per cell**, not as `CommitSummary::stamps_skipped`: the
  bake runs on the committed layers, after the stamps are merged.
- **`P` = 16 cells** (`REERODE_PAD_CELLS`), twice the hillslope's 8-pass
  reach. Judgement; the ring step it leaves is measured below.
- **`W` is padded from the baked cells' bounding box**, not the dirty
  tiles': the cells are what changed, and the tile box only adds unpainted
  ground. The window is clipped at the map edge and does not wrap.

Not re-run by the chain, each named in `ReerodeReport::not_rerun`: the carve
and channel network (the recompute's), GF-11's construction delta on a
constructed world (the windowed delta is not built), and the passes block
(evolve cycles, sediment fill, velocity, diffusion). With
`geology_processes` off the bake stands and `reerode` returns
`ProcessesOff`: erosion does not read rock on that world.

**Measured** (`cartalith-engine/tests/gf8_rock_paint.rs`, release, 512 x 328
at 800 km, app params with model, processes, karst and glacial on; a radius-8
disc of shale painted into the granite interior nearest the map centre):

| seed | W | cells moved | painted mean lowering | unpainted mean | ring step / relief | commit |
|---|---|---|---|---|---|---|
| 12345 | 49 x 49 | 2401 | 6.602e-4 | -4.486e-5 | 0.0054 % of 0.5116 | 3.9 ms |
| 483920 | 49 x 49 | 2401 | 2.342e-4 | -1.510e-5 | 0.0061 % of 0.1408 | 4.0 ms |

Commit times are one run each, not a benchmark. Every cell of `W` moves,
because the rebound is a blur; the unpainted cells rise slightly for the same
reason. No cell outside `W` moved on either seed. B11's identity holds on both
seeds: repainting the rock already there moved the field by 0 bytes and the
column by 0 bytes apart from `painted`.

**Mutations** (Python, exact literal replace, restored in `finally`): the
before-run reading other rock, the before-run at contrast 0 (the κ slice
dropped from one run), `CAP_CLASS_M` 10 → 11, the `rock_sub` and `rock_top`
index off by one, and the window's index shifted by one cell: **6 of 6
killed**.

**Identity** (`cartalith-godot/tests/gf5_karst.rs` `gf5_identity_hashes`,
512 x 328, seeds 483920 and 314159, parity / app / processes-on, 13 arrays
each): the change against `531a2477`, built in the same worktree, **18 of 18
lines identical**.

### GF-8 measured: the shell half (2026-10-18)

*Design and evidence, not status.* `rock_paint::commit` now has its one app
caller, `WorldGen::paint_commit_rock` (`cartalith-godot/src/lib.rs`), reached
from the one paint Commit.

What this slice adds:
- **Three Rock layers in `PaintEditor`** (`paint_bridge::PaintTarget::RockTop`
  / `RockSub` / `RockCap`, keys `rock_top` / `rock_sub` / `rock_cap`). They are
  drafts only: `take_rock_commit` commits them into whole-world byte grids for
  the bake, so there is no committed Rock layer to render or save.
  `PaintTarget::ALL` stays the three override layers, so `get_paint_layers`,
  `drafts/paint.json` and the renderer did not move. `pending_stamps` and
  `discard_all` include the Rock drafts (one Commit / Discard). Values:
  `rock_sub` keeps 255 = none; `rock_cap` takes 1..255. **Erase is stored
  `false` on a Rock layer** in `set_brush`, which both the Erase latch and the
  Shift modifier pass through.
- **The gate** `WorldGen::rock_paint_gate`, read by `rock_paint_status`,
  `paint_set_layer` and the commit. It refuses when there is no generated world,
  no column, `geology_processes` off, `carve_rivers` off, or a finalized world.
  **App worlds have the processes off, so the Rock target is disabled by
  default**, with the engine's sentence as its reason. The column exists on app
  worlds (`geology_available()` is true), but erosion would not respond to it.
- **The commit's Rock branch.** Gate → undo step with the column
  (`push_with_column`), redo tail cleared, one `HeightSnapshot` ledger row
  (`"geology"`, *"Rock paint commit"*) → `rock_paint::commit` with `P` =
  `REERODE_PAD_CELLS` → `Civ` marked over the painted tiles and `Height` over
  `W`'s tiles, then one recompute. A refused gate keeps the Rock drafts. A
  refused bake pops the step it pushed.
- **Bindings:** `rock_paint_status` (`available`, `reason` only when
  unavailable, `layers`, `cap_class_m`, `cap_max_m`, `sub_none`, `pad_cells`,
  `painted_cells` only when a mask exists, `save_note`), `geology_available`,
  `get_rock_classes` (the `ROCK_PROPS` rows), `get_paint_palette("rock_*")`
  (each rock's label carries `s` and `κ`; entries also carry `s`, `kappa`,
  `theta_c`, `selby`), and `geology_digest`, a probe-only diagnostic. The shell
  reaches the new ones through `has_method` + `call`, so an older DLL does not
  break parsing.
- **Shell** (`rock_paint_ui.gd`, the shared wording, plus four surfaces). The
  tool bar (desktop and tablet) gains a Rock segment and a `PAINT · ROCK` options
  row. The WORLD dock's Target field gains a Rock item with Rock controls. The
  right dock gains a Rock context. The phone Paint column gains a Rock chip, layer
  chips, a thickness slider and notes. Every Rock surface does three things: it
  draws the Rock target **disabled with the engine's reason** when it is
  unavailable; it disables Erase with §4.13's reason; and it says **"not saved
  with the project yet"**, because the column is not in the save format.
  `_authored_inventory()` lists painted rock, and the regenerate prompt adds the
  save caveat when painted rock is at stake. The status line (a note on a phone)
  reports each commit: the window, the cells moved, the ring step, every skip
  count and the not-re-run list. `paint_original.gd` gives Rock an honest
  no-Original reason.
- **Not built (slice 3):** the save set (§2.6's column rasters,
  `rock_painted.u8`, `geology.painted`, the three draft layers in
  `drafts/paint.json`). Until it exists, reopening a project loses the column,
  and with it the painted rock. The UI says so.

**Decision taken here, labelled.** When a regenerate replaces the engine's
`PaintEditor`, the engine goes back to Biome but the shell keeps its selection.
`world_workspace.gd::rock_status()` drops a stale Rock selection back to Biome,
and `_paint_ensure_rock_layer()` re-selects the Rock layer in the engine before
each Rock stroke. **The override layers were left exactly as they were**, which
includes the same pre-existing mismatch after a regenerate (reported, not
fixed).

**Measured** (`_gf8rock_probe.gd`, windowed, scratch copy, debug DLL built
from this change; seed 483920, 512 x 328, 800 km). The engine's app world first,
then the same seed with `geology_processes` set on; a radius-6 shale disc at
(256, 192), with Land only off:
- Leg A (app world): the Rock target is drawn disabled with the
  processes-off reason, and the save caveat is on it; the engine and the shell
  both refuse the layer. Field and column digests are unchanged.
- Leg B: `W` = (234, 170, 45 x 45), which is the predicted disc box + 16.
  113 cells were baked; 2 025 cells moved (all of `W`); the ring step was 0.020 %
  of relief. The commit took 29.7–34.4 ms in the debug build (one run per form,
  not a benchmark). Field and column digests outside `W` were unchanged; inside,
  both moved (the positive control). The painted mask went from absent to present.
  **Undo restored both, byte for byte, and Redo reapplied both.**
- Desktop 1600 x 900: 35 of 35. Phone 393 x 852 `--force-touch`: 35 of 35.
  Tablet `--force-touch`: 35 of 35, at **1600 x 1031** (the window was clamped to
  the screen, not 1600 x 2560).
- **Tests:** 6 in `paint_bridge::tests` (literal values, Erase refused, the
  draft routing, the "none" slot, the swatch literals) and 1 end-to-end in
  `lib.rs` (`rock_paint_commit_tests`: a generated processes-on world, field
  and column outside `W` untouched, undo and redo exact). 7 mutants were run; 7
  were killed (Discard skipping the cap draft first survived, and the test now
  discards one dab per Rock layer). `gf5_identity_hashes`: 6 of 6 lines
  identical to `55fa1607`.

**Found on the way: a shell defect, fixed.** `paint_original.gd::decorate`
read the options row as "the column's last child". ET-D5 (`e05a5445`) appended
the brush row, which is an `HFlowContainer`, so the cast was `null` and the
**Original slider had not been drawn on the Paint bar since then**. That was
`_paintorig_probe.gd`'s long-failing check. The fix reads the last
`HBoxContainer`. Desktop `_paintorig` now passes 0 failures. On a phone the probe
now reaches a later check, *"returning to 0 restores the original pixels
exactly"*, which fails. Its frames show deep-zoom LOD tile detail arriving
between the two captures (the land is blocky in the first frame and detailed in
the second). That is a probe timing issue the old failure hid. It is reported and
not fixed.

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

`LARGE_ITEM_RULINGS.md` was searched on 2026-09-27 for each of these. None
was ruled when this scope was written. **Ruling BJ (2026-09-27) answered Q1,
Q2, Q4 and Q7**, marked below. The others keep their defaults, as the ruling
says. Q11 is new with the clock.

1. **How many rock types?** **Answered by Ruling BJ: 11**, the table in §2.2.
   (The recommended default was the same.) They project to the 7
   legacy classes for civ, soil and resources until those tables are extended.
   - The alternatives are 7, reusing the legacy classes, which cannot express
     a strong cap over weak rock inside the sedimentary group, or about 16:
     adding quartzite, dolomite, chalk, evaporite, conglomerate and impactite.
2. **Can the user paint lithology?** **Answered by Ruling BJ: yes, in this
   build**, reversing the recommended default ("not in this build"). §4.13
   specifies it and GF-8 builds it: the surface rock, the rock beneath, and
   cap thickness; a baked commit with an undo step; persistence in §2.6's
   set; and a differential re-erosion over the painted window, so erosion
   responds to the painted rock.
   - The interactions the old default deferred are settled there. Sculpting
     down still exposes the substrate through the exposure rule, and sculpting
     up extends the top rock. Regeneration replaces paint as it replaces
     sculpting, and the regenerate confirmation names it.
   - Ruling M's drawn plates still inform resources only, and do not feed
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
   scope has none (§1.4). **Answered by Ruling BJ: add a simple clock**,
   reversing the default ("BH builds no clock"). §4.12 specifies one
   dimensionless parameter, geological age τ, that scales pass and iteration
   counts. It is linear where the kernel or the column already limits the
   process, saturating where the real process slows for a reason the kernel
   does not model, and logarithmic for the weathered mantle. GF-7 builds it.
   Formation age stays relative. The geological-time scope's larger design
   stays declined, and its §8 prerequisites are not needed.
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
   today. Rock-aware cliff retreat and BG's sea stacks need it. **Answered by
   Ruling BJ: yes, at GF-4**, confirming the default, with its measured cost
   disclosed as Ruling AU did for glacial (§4.5, GF-4).
8. **Layer dip?** **Default: horizontal with a gentle low-frequency warp.**
   Per-province dip, which gives cuestas and hogbacks, can come later.
9. **Remove `tect.dynamic_lithology`?** The explicit column supersedes the
   exhumation heuristic. **Default: yes, at GF-9** (§7p). Its GUI row goes and
   the parameter is dropped.
10. **Impact breccia as a rock type?** **Default: no.** `impact_field` stays a
    marker. It can join Q1's extended set later.
11. **Should the clock age craters too?** `crater_degradation_tau` already
    degrades craters by the hillslope diffusivity (`DECISIONS.md` §7l-ii
    ruling 2). The clock could also scale it, so an old world's craters are
    more degraded. **Default: no, not in GF-7.** The crater model has its own
    ruling and its own calibration, and coupling it to τ changes crater
    output, which B12's identity-at-τ = 1 would not catch at other values of
    τ. It can be ruled on separately.

*Questions 12–19 are new with Ruling BM (2026-09-27). **Ruling BN
(2026-09-27) answered Q12, Q14 and Q18**, marked below, and kept the
defaults of Q13, Q15, Q16, Q17 and Q19. Each default is what GF-10 to GF-13
build.*

12. **Which tectonic settings uplift?** **Answered by Ruling BN: all of
    them**, which reverses the default below. Convergence, flexure and the
    continental background, **plus rift shoulders (uplift) and basin
    subsidence**: §4.15's signed rift term `R`, with the stage's basin fill.
    GF-10 measured it (§5.10). The default was: convergence (`stress`),
    the flexural bulge behind it (`compute_flexure`), orogeny where
    world-structure is on, and a small continental background (§4.15). **No
    subsidence and no rift-shoulder uplift** in the first build: subsidence
    would lower land under the sea, and it is deposition's business.
    - The alternative adds rift shoulders from `min(σ, 0)` near `RIFT`
      boundaries, which gives the flanking highlands of a rift valley.
13. **The existing "Uplift" slider (`stream.uplift`, reference control
    `sUp`).** **Default: leave it the legacy kernel's**, in its normalised
    per-iteration units, and give BM's field its own parameters, with no GUI
    control until GF-13. Its magnitude is calibrated (§4.15), so a slider
    would mostly let a user break the mean balance.
    - The alternative repurposes the slider as a multiplier on BM's `U₀`.
      That gives the user a "more active tectonics" control, at the cost of a
      slider whose meaning depends on a switch.
14. **May construction move the coastline?** **Answered by Ruling BN: yes**,
    which reverses the default below. Weak coastal rock may go below sea into
    bays and inlets, hard headlands stay, and there is still no inland pit:
    §4.14's coast bullet (an ocean-seeded 4-connected fill in place of the
    floor) and B14's reworded check. GF-10 measured it (§5.10). The default
    was: **no.** Land stays land
    (§4.14), so the land share and every coastal consumer keep their
    meaning, and B8's ocean-on-paths guarantee is not tested by a moving
    sea.
    - The alternative lets weak coastal rock go under, which makes drowned
      valleys and embayments in soft rock. It would move the land share and
      is better scoped with GF-4's coastal pass.
15. **Rebound after the uplift-driven stage?** **Default: none** (§4.15):
    `U` is net rock uplift, and rebound returned 78–93 % of the light pass's
    lowering over an 18-cell radius that does not scale with the map.
    Rebound stays after the glacial pass and in `evolve_cycles`.
    - The alternative keeps rebound and raises `U₀` to compensate. It would
      keep the extent-blind radius and re-open the metres-scale result of
      §5.8.
16. **The clock under BM: model time, not counts.** **Default: yes.** Under
    the switch, τ scales the stage's model time `T` and the construction
    budget, at a fixed step count, so the cost does not grow with τ
    (§4.15). This amends §4.12's "never on rate constants" for this one
    stage, because its kernel is implicit and unconditionally stable (Braun &
    Willett 2013), and §4.12's reason was the explicit kernels' stability
    wall. Every other process keeps §4.12's count laws.
17. **Arc volcanoes exempt from construction?** **Default: yes.** Their relief
    is the stamper's and young; plateau basalt is not exempt.
18. **Build BM before GF-4, GF-5, GF-6 and GF-8?** **Answered by Ruling BN:
    relief first**, GF-10 → GF-13, then GF-4, GF-5, GF-6 and GF-8, with GF-9
    last. That confirms the default. **Default: yes** (§7's
    build order). Their bars are measured on whatever relief model exists; if
    BM lands after them, each would need re-measuring.
19. **Wire the weathered mantle under BM?** §5.8 left it unwired because
    §4.12 anchors it at 2 m at τ = 1, which contradicts τ = 1 identity.
    Under the BM switch τ = 1 is not identity (§4.16), so that objection does
    not apply there. **Default: yes, under `relief_model` only, in GF-13.**
    Its maximum (2.69 m) stays below `R_EXPOSE`, so it cannot change the
    exposed rock on its own.

*Not a question, recorded so it is not mistaken for one:* B9 stays at
≤ 1.20 × control at τ = 1 under BM. If GF-10 shows that the step count B8
needs costs more, the trade comes back to the owner as a new question; the
bar is not moved to fit it.

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
before it is pinned, and replaces any reference it cannot verify. The five
added for the clock (Baldwin et al., Harbor, Heimsath et al., MacGregor et
al., Trenhaile) are verified the same way by GF-7's lane. The three added for
Ruling BM (England & Molnar, Kirby & Whipple, Montgomery & Brandon) are
verified by GF-10's lane; §4.15 relies on Whipple & Tucker (1999) and Braun &
Willett (2013), already listed, for the steady-state balance and the implicit
scheme, and GF-10's lane checks both at the source for the equations quoted.

- Ballantyne, C.K. (2002). Paraglacial geomorphology. *Quaternary Science
  Reviews* 21, 1935–2017.
- Baldwin, J.A., Whipple, K.X. & Tucker, G.E. (2003). Implications of the
  shear stress river incision model for the timescale of postorogenic decay
  of topography. *Journal of Geophysical Research* 108(B3), 2158.
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
- England, P. & Molnar, P. (1990). Surface uplift, uplift of rocks, and
  exhumation of rocks. *Geology* 18, 1173–1177. The rock-uplift versus
  surface-uplift distinction §4.15 uses for rebound. (Added for Ruling BM;
  already cited by `isostatic_rebound`'s doc comment.)
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
- Harbor, J.M. (1992). Numerical modeling of the development of U-shaped
  valleys by glacial erosion. *Geological Society of America Bulletin* 104,
  1364–1375.
- Harel, M.-A., Mudd, S.M. & Attal, M. (2016). Global analysis of the stream
  power law parameters based on worldwide ¹⁰Be denudation rates.
  *Geomorphology* 268, 184–196.
- Heimsath, A.M., Dietrich, W.E., Nishiizumi, K. & Finkel, R.C. (1997). The
  soil production function and landscape equilibrium. *Nature* 388, 358–361.
  The source of §4.12's exponential soil-production law and `h₀`.
- Hoek, E. & Brown, E.T. (1980). Empirical strength criterion for rock masses.
  *Journal of the Geotechnical Engineering Division, ASCE* 106(GT9),
  1013–1035.
- Kirby, E. & Whipple, K.X. (2012). Expression of active tectonics in
  erosional landscapes. *Journal of Structural Geology* 44, 54–75. The
  channel-steepness form of the steady-state balance, `k_s = (U/K)^(1/n)`.
  (Added for Ruling BM.)
- Krabbendam, M. & Glasser, N.F. (2011). Glacial erosion and bedrock
  properties in NW Scotland: abrasion and plucking, hardness and joint
  spacing. *Geomorphology* 130, 374–383.
- MacGregor, K.R., Anderson, R.S., Anderson, S.P. & Waddington, E.D. (2000).
  Numerical simulations of glacial-valley longitudinal profile evolution.
  *Geology* 28(11), 1031–1034.
- Marinos, P. & Hoek, E. (2000). GSI: a geologically friendly tool for rock
  mass strength estimation. *Proc. GeoEng2000*, Melbourne (the `mi` table).
- Montgomery, D.R. & Brandon, M.T. (2002). Topographic controls on erosion
  rates in tectonically active mountain ranges. *Earth and Planetary Science
  Letters* 201, 481–489. Erosion rate tracks relief, and saturates where
  hillslopes reach their threshold angle. (Added for Ruling BM.)
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
- Trenhaile, A.S. (2000). Modeling the development of wave-cut shore
  platforms. *Marine Geology* 166, 163–178.
- Whipple, K.X. & Tucker, G.E. (1999). Dynamics of the stream-power river
  incision model. *Journal of Geophysical Research* 104(B8), 17661–17674.

### The lithology corner blend (2026-10-12, Ruling CG addendum 2)

*Design and evidence, not status (`STATUS.md` has the numbers).* Built as
proposed in (a): under magnification only, `RenderCtx::litho_at_f` returns
the four jittered corner rocks with their bilinear weights, and
`land_color` blends each rock's colour terms by them.
- **Where it acts.** Export bake, snapshot and deep-zoom tiles, when a pixel
  is smaller than a cell on both axes. At one pixel per cell or coarser the
  read stays nearest, which is the OpenGL magnification/minification switch
  point. That keeps a grid-resolution export equal to the screen. The
  fit-view map (`cell_color`) is untouched.
- **Result.** The τ 4 NE stair and the carve-off scarp's blocks are gone in
  the frames. On the scarp crop the crisp-step count falls to the
  lithology-off level (1 024 → 668, against 671 with lithology off).
  Evidence: `cartalith-native/docs/evidence/litho-blend-2026-10-12/`.
- **Not addressed.** The flow line (b), the gorge (c) and the shore scallop
  (d) are unchanged. Questions 5 and 6 took their defaults, so nothing was
  built for them.

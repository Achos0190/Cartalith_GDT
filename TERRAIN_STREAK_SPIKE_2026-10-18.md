# Terrain streaks: a measurement spike (2026-10-18)

The owner asked to measure a fix first (2026-10-18). The 2026-10-18 zoom-forest
report (`STATUS.md`, Ruling CI follow-up) had found faint streaks running
downslope: radial on 71077345's ice dome and north-south on 314159's coastal
slope. It suspected the generation river carve, D8 flow routing and the rain
field's lattice. This spike looks for the stage that puts grid-aligned
structure into a world, using ablation on five seeds. It then tests three
principled de-aligned variants. Each is behind a switch that is **off at both
boundaries**, so nothing ships and no golden moves.

The governing requirement (owner): every zoom level gains believable detail with
no grid artefacts. Frames were read at x4, x16 and x64 (deep-zoom tile ladder)
and at x1 (fit view).

This is a findings document. It holds no status; status lives in `STATUS.md`.

> **Note, 2026-10-18 (after Ruling CM).** The owner answered the four
> questions below at their defaults. What that changed in the tree, so the
> rest of this document is read as the record of the spike rather than as a
> description of today's code:
>
> - **(a) shipped.** The B-spline rain up-sample is no longer a spike switch:
>   it is the named parameter `climate.rain_bspline`
>   (`cartalith_engine::ClimateInputParams::rain_bspline`, a `PARAMS` row with
>   no GUI control), **on in the app's `params::defaults()`** and off in
>   `WorldParams::defaults` / under every JS golden. A save without the key
>   reloads bilinear, so an older project regenerates as it was saved.
> - **(b), (c) and the display-only MFD flow are retired.**
>   `carve_channel_network_subcell`, `chaikin_round`, `compute_flow_mfd`,
>   `stream_power_kernel_rho8` and `rho8_factor` are removed, with their
>   tests and the `carve_subcell` / `stream_rho8` switches. So is the plain-D8
>   channel ablation (`build_channels_routed_plain_d8`, `channel_plain_d8`),
>   which measured no effect (0.995-1.001 of head). Removing them moved no
>   array of the parity, app or processes-on worlds
>   (`gf6_carve_identity_hashes`, identical to `86ae9d0a` before (a) was
>   switched on).
> - **What is left of the harness.** `StreakSpikeParams` keeps the two
>   ablations that still measure something (`skip_light_pass`,
>   `skip_channel_carve`); `terrain_streak_spike.rs` keeps `head`,
>   `carve_off`, `no_light`, `no_cut`, `no_glacial`, `edifice_off` and a new
>   `rain_bilinear` arm (the app before the ruling), and now also prints
>   river counts and land biome fractions. The tables below quote arms that
>   no longer exist (`a_rain_bs`, `b_sub*`, `c_rho8`, the combos, `plain_d8`,
>   `DISPLAY`); their numbers stand as measured on `df64f48c`.
> - **Follow-ups** are two `OUTSTANDING_WORK.md` rows: the edifice gullies
>   (kilometre spacing, sub-cell gullies at zoom, the pits; measure first)
>   and locating 314159's streaks with the river layer hidden.
> - The before/after measurements of shipping (a) are in `STATUS.md`,
>   2026-10-18 (Ruling CM).

## How it was measured

- **Harness:** `cartalith-native/crates/cartalith-godot/tests/terrain_streak_spike.rs`
  (`streak_spike_measure`, `streak_spike_render_ablation`, `streak_spike_cost`;
  all `#[ignore]`d). Frames come from `tests/zoom_detail_ladder.rs` with the new
  `ZD_STREAK` / `ZD_FLOW` switches. The ladder is the shipped deep-zoom tile path
  without the river stroke layer, under the shipped kilometre law. Its targets
  are fixed to head's picks with `ZD_PICKS`.
- **Worlds:** app parameters (`params::defaults()`), 2048 x 1311, 800 km.
  - 71077345 Classic
  - 314159 Earth archetype
  - 24601 Archipelago archetype
  - 483920 Classic
  - 12345 Classic
- **Directional metric (`dir_ratio`):** band-pass the field between 1 and 3
  cells (difference of Gaussians), take a Scharr gradient, then divide the
  gradient energy within 22.5 degrees of an axis by the energy within 22.5
  degrees of a diagonal. 1.0 means isotropic.
- **Downslope streak metric (`across_along`, plus `H_acrE` / `LUM_acrE`):** uses
  the steepest 2 % of land. That subset is picked on the `carve_rivers`-off
  world, which is an input to the carve, never the field under test. The metric
  compares band-passed gradient energy across the regional downslope direction
  (Gaussian blur, sigma 6) with energy along it. A downslope streak adds
  across-slope energy, so `acrE` (the across energy relative to head) is the
  streak number.
- **Lattice metric (`lattice_index`):** mean |d2/dx2| on the weather grid's own
  node lines over the mean between them. Bilinear creases read above 1.
- **Calibration:** `metric_calibration` checks isotropic noise (1 within 0.1),
  10-degree and 35-degree gratings, downslope versus across-slope ridges, and a
  bilinear up-sample against smooth noise (lattice above 3 against 1 ± 0.15).
- **Edge exclusion:** every population excludes 40 cells at the map edge (see
  correction 2).
- **Rivers:**
  - channel cells, carved (`river_mask`) cells and order >= 3 cells
  - RV-1's guards: ocean cells on river paths, and 1-3-cell lakes from
    `build_water_bodies`
  - basin-outlet agreement with head, exact and within 5 cells, following the
    routing surface's D8 receivers to the sea
- **Run logs:** the full per-world lines are `ARM` / `DISPLAY` / `RENDER` lines
  in the harness output. Medians below are over the five seeds, with the range
  in brackets.

## Two corrections to the earlier report

1. **"Fine structure leans diagonal (0.33 / 0.44 / 0.49)" was mostly the
   metric.** A plain high-pass keeps the square grid's corner frequencies, and
   those all lie toward the diagonals. Through this harness's own operator,
   white noise high-passed at 3 cells reads **0.857**, not 1 (measured; it is
   why `BAND_INNER` exists).
   - With the 1-3-cell band-pass, the height field is close to isotropic:
     median **0.935 (0.746-1.113)** on head.
   - Turning the carve block off does not make it more isotropic:
     **0.813 (0.740-1.336)**.
   - The carve is not what bends the world's fine structure toward the
     diagonals.
2. **The rendered map's axis bias was the plate frame.** On the first pass the
   x1 base-raster luminance read 9.99 axis over diagonal on 71077345.
   - The render ablation found one term that moved it: `border_width_frac` set
     to 0 gave **0.975**, and 0.968 on 314159.
   - The frame's straight edges are pure axis structure. This is `MISTAKES.md`'s
     "measure the picture without accounting for the frame".
   - With the frame band excluded, rendered x1 luminance is isotropic: median
     **0.977 (0.868-1.056)**.

## What makes the streaks

### The radial streaks on 71077345's dome are the volcano edifice's gullies

- They survive `carve_rivers = false` (frame 01, middle row).
- They vanish with `volc.edifice_model = false` (frame 01, bottom row).
- The dome is a volcano. `cartalith_terrain`'s edifice model, which the app
  turns on under DECISIONS §7l-ii's ruling 1, deliberately cuts radial gullies
  every `EDIFICE_GULLY_SPACING_CELLS = 4` cells at mid-flank.
- At 4 cells the gullies sit near the grid's resolution:
  - at x1 they read as a sunburst on every volcano (frame 07);
  - at x4 they read as hairline streaks;
  - at x16 and x64 the tile synthesiser magnifies them as soft radial bands
    without adding gully detail.
- Measured with the edifice off:
  - steep-land across-slope energy falls to **0.917 (0.793-0.984)** of head,
    0.880 on 71077345 and 0.793 on 24601;
  - rendered across energy falls to **0.943 (0.829-0.958)**.
- **Side finding (measured):** with the edifice off, RV-1's small-lake count
  drops from a median of **61 to 18**. On 24601 it falls from 176 to 8, and on
  314159 from 61 to 18. The gullies leave closed 1-3-cell depressions.

### The generation carve block adds a little downslope energy

Steep-land across energy relative to head, with each piece removed:

| Removed | Median | Range | 71077345 |
|---|---|---|---|
| The whole carve block (`carve_rivers` off) | 0.975 | 0.821-1.024 | 0.821 |
| The light stream-power pass (`no_light`) | 0.988 | 0.875-1.025 | 0.875 |
| The channel cut (`no_cut`) | 0.991 | 0.935-0.999 | 0.935 |

- The carve block is a real but secondary source: up to 18 % on the volcanic
  dome and a few percent elsewhere.
- Of its two parts, the D8-routed light pass contributes more than the channel
  cut.
- On 314159 the carve block **removes** across energy (1.024 with it off).
- Two candidate stages measured as **no effect** on the streak numbers:
  - `build_channels_core`'s aspect steering (`plain_d8`: 0.995-1.001 of head);
  - the glacial pass (`no_glacial`: 1.000).
- Frames 01-06 show no visible streak difference with the carve off, apart
  from the carved river trenches themselves (frame 04).

### The rain field's square lattice is real, and does not reach luminance

- `RAIN_lat` on head: **2.617 (1.076-3.519)**. The rain field creases on the
  240-wide weather grid's lines, 8.5 cells apart at 2048. Three bilinear
  up-samples cause it:
  - `finish_weather_grid`;
  - the coastal boost in `apply_climate_moisture_correctors`;
  - the river-corridor boost in the same function.
- It is an interpolation lattice, not a noise lattice. The `sea_grain_warp` /
  `land_grain_warp` pattern (warp or rotate a value-noise lattice) does not
  apply: warping bilinear coordinates would only bend the creases. A
  reconstruction kernel with no crease removes them.
- The lattice does **not** show in the rendered luminance: `LUM_lat` is
  **0.999 (0.987-1.005)** on head. Colour (hue) was not measured, so a lattice
  in the biome tint is not excluded.

### D8 flow's parallel lines are real, and do not reach luminance

- The log-flow field's axis ratio is **1.306 (1.288-1.336)** on head.
- The render reads flow through its TWI wetness, near-channel wetness and river
  SDF terms. Rendered luminance does not change measurably whether it is fed
  D8 flow, MFD flow or no flow at all. See the `DISPLAY` lines; for example,
  71077345 reads 0.966 / 0.961 / 0.958.

## The variants

Every variant is off at both boundaries. Identity: parity, app and processes-on
worlds (seeds 483920 and 314159, 512 x 328, 13 arrays,
`gf6_carve_identity_hashes`) hash **identical** before and after this change
(`3f27e383`). So no golden test can move while the switches are off.

| Variant | Switch | Removes | Steep-land across energy (rel. head) | Rendered across energy | Rain lattice | River cells / carved | Small lakes (median) | Ocean on paths | Outlets same / within 5 cells | Generation cost |
|---|---|---|---|---|---|---|---|---|---|---|
| head | -- | -- | 1.000 | 1.000 | 2.617 | 22 573 / 67 451 | 61 | 0 | -- | 10.41 s (9.80..11.77) |
| (a) C2 rain up-sample | `rain_bspline` | the rain lattice | 1.000 | 1.000 | **0.984 (0.956-1.082)** | 22 526 / 70 114 | 61 | 0 | 99.93 % / **99.996 %** | 10.88 s (10.65..11.33) |
| (b) sub-cell carve, 1.5 cells | `carve_subcell = 1.5` | the cell-centre trench | **1.033 (1.011-1.080)**, worse | 1.031 | 2.612 | 22 573 / 76 855 (+14 %) | 56 | 0 | 86.8 % / 99.89 % | 10.86 s (10.64..11.38) |
| (b) at 1.0 / 2.5 | `carve_subcell` | -- | 1.015 / 1.111 | 1.014 / 1.092 | -- | 68 921 / 102 968 carved | 59 / 52 | 0 | -- / 99.90 % and 99.66 % | -- |
| (c) display-only MFD flow | none (`compute_flow_mfd`, harness only) | D8 lines in the flow field | n/a (display only) | 0.98-1.02 of D8 render | n/a | unchanged (simulation untouched) | unchanged | unchanged | unchanged | 277 ms vs 72 ms for D8 |
| (c) Rho8 light pass | `stream_rho8` | D8's 8-direction incision | 1.010 (1.004-1.042) | 0.980 | 2.553 | 23 279 / 74 358 | **119**; 34 to 119 on 483920 | 0 | 53.7 % / **93.0 %** | 10.68 s (10.38..10.75) |
| a + b + c (Rho8) | all three | -- | 1.070 | 1.020 | 0.996 | 23 438 / 82 681 | 104 | 0 | 40.9 % / 91.9 % | 11.10 s (10.51..11.37) |

Notes on the table:

- **Cost.** Full generation was measured alone, five runs each and alternated,
  on the debug test profile (`opt-level = 1`). Every variant's range overlaps
  head's, so **no generation-cost difference is established**.
- **MFD cost.** The MFD figure is the accumulation alone over the 2048 x 1311
  routing surface, median of five.
- **MFD flow field.** The flow field's own axis ratio falls from 1.306 to
  **0.985 (0.922-1.028)**.
- **Strahler order.** Maximum order is unchanged on every variant except Rho8,
  which takes it from 4 to 5 on two seeds.

### Frames

Frames are in `cartalith-native/docs/evidence/terrain-streaks-2026-10-18/`, all
opened. They are x4 / x16 / x64 centre crops of the ladder at head's own picks,
plus x1 fit-view luminance.

- **(a) Rain B-spline:** no visible change at any rung on the four frames that carry it
  (02, 03, 05, 08). The lattice it removes did not show in luminance; see the
  rain finding above.
- **(b) Sub-cell carve:**
  - river trenches widen slightly at x4 and x16 (frame 04);
  - no streak is removed;
  - the steep-land across energy rises, because a wider trench is still a
    downslope feature.
- **(c) MFD display flow:** no visible change (frames 02, 03).
- **(c) Rho8:** no visible change in the dome's streaks (frame 02). The radial
  pattern is the edifice, not the routing.
- **Edifice off (an ablation, not a variant):** the dome's radial streaks are
  gone at x4 and x16 (frame 01). On 314159's mountain and forest frames there is
  no visible change (frames 04, 06), and those frames hold no volcano.
- **Shell (frame 09):**
  - the windowed shell (`_zoomdetail_probe`) on this spike's DLL with every
    switch off, at x1 / x4 / x16 / x64;
  - generation is hash-identical to HEAD, so this is the shipped picture.
- **Seen in every arm, unrelated to these stages:**
  - a short diagonal dash hatching at x16 on snow and relief (frame 06). This is
    the render-side "dark diagonal hatching" already filed in
    `OUTSTANDING_WORK.md`;
  - a straight north-south colour edge at x16 in 314159's forest (frame 03),
    possibly a tile or level boundary (not established);
  - a straight east-west scarp across 71077345's volcano (frames 01, 02).

### Golden tests that would move if each variant were switched on

All of these are behind the app boundary (`params::defaults()`). None is a
JS-captured reference golden, because `WorldParams::defaults()` keeps every
switch off by control flow.

- **(a) Rain B-spline:** every app-world rain, flow, river and biome digest:
  - `cel_toon.rs` shipped-look grid and tile digests;
  - the GF harnesses' processes-on bars;
  - any app-world landmark or settlement count;
  - the identity hashes' app and processes-on lines.

  The reference path (`simulate_weather`, `finish_weather_grid`,
  `apply_climate_moisture_correctors`) is the `Bilinear` arm by control flow.
  `golden_parity_weather.rs` and everything JS-captured downstream cannot move
  unless `WorldParams::defaults()` turned it on, which it must not.
- **(b) Sub-cell carve:** app-world field, `river_mask`, `river_floor` and
  everything downstream of the carve (the same app-side digests). The
  JS-captured `golden_parity_carve.rs` and the RV-1 unit tests call
  `carve_channel_network` and are untouched.
- **(c) Rho8:** app-world field and the entire drainage topology (outlets
  within 5 cells: 93 %), so every app-world river, landmark and settlement
  digest. `golden_parity_streampower.rs` calls the unchanged entry points.
- **Display-only MFD:** no golden in the simulation. It would move the shipped
  render digests (`cel_toon.rs`) only where the TWI or wetness terms read
  flow, which the measurement says is invisible.

No variant reaches a reference-verified or JS-captured golden through any
shipped path, so no variant had to be stopped on that rule. Rho8 was stopped
on hydrology instead: it nearly doubles 1-3-cell lakes, and 7 % of land
changes basin.

## Recommendation

1. **Do not ship (b) or (c) for streaks.**
   - Neither removes the observed streaks.
   - The sub-cell carve adds downslope energy.
   - Rho8 fails RV-1's small-lake guard and moves basins.
   - Display-only MFD makes the debug flow field isotropic (axis ratio 1.31 to
     0.99) but changes nothing a reader sees on the map.
2. **(a) is safe and principled, and fixes a real lattice in the rain field.**
   - It removes a measured C0 lattice: 2.6 to 0.98, while basin outlets agree
     within 5 cells on 99.996 % of land.
   - Its visible benefit is not established in luminance.
   - Switching it on is an owner call: it moves every app world.
3. **The streak the owner sees on the dome is the volcano edifice's 4-cell
   gully spacing**, a designed feature drawn at the grid's own resolution.
   - The principled fix belongs to the edifice and the zoom law, not to the
     carve: gully spacing in kilometres, not cells, with the sub-cell gullies
     synthesised by the deep-zoom tile path. That design question is new.
   - The edifice gullies also leave closed 1-3-cell depressions (small lakes
     61 to 18 with the edifice off), which is worth fixing on its own.
4. **No candidate stage was found for 314159's north-south coastal streaks.**
   - Not this spike's carve, routing or rain variants.
   - Not the edifice: 314159's across energy moves 0.984 with it off.
   - This spike's Rust-ladder targets (picked by inputs) did not frame that
     slope. The windowed shell run on this DLL did (frame 09, 314159 forest x4
     and x16).
   - There, the north-south lines include **perfectly straight vertical river
     strokes**: channels running due south down a uniform coastal ramp, the
     comb `OUTSTANDING_WORK.md` already carries. Faint terrain streaks also sit
     between them.
   - Separating the two needs the river layer hidden in the shell, which this
     spike did not do.

## Owner questions (with defaults)

1. **Switch on (a), the C2 rain up-sample, in the app?** Default: **yes**, with
   the app-side digest re-baselines it implies. It is principled, costs nothing
   measurable, and keeps the rivers. Its visible benefit is unproven.
2. **Re-scope the dome streaks as an edifice and zoom-law question?** For
   example, gully spacing in kilometres with sub-cell gullies synthesised at
   zoom, and removing the closed depressions the gullies leave. Default:
   **yes**, as a new backlog row. Nothing here is built.
3. **Retire variants (b) and Rho8, and their switches?** Default: **yes, delete
   them in the next pass that touches this code.** They are measurement-only
   and off. Keep (a) until question 1 is answered, and keep `compute_flow_mfd`
   only if the flow debug view wants it.
4. **Locate 314159's coastal streaks with the windowed probe's pick before any
   further generation change?** Default: **yes**.

## What was built (all off at both boundaries)

| Piece | What it does |
|---|---|
| `cartalith_engine::StreakSpikeParams` (`WorldParams::streak`) | `skip_light_pass`, `skip_channel_carve`, `channel_plain_d8`, `rain_bspline`, `stream_rho8`, `carve_subcell` |
| `cartalith_climate::CoarseUpsample`, `simulate_weather_with`, `finish_weather_grid_with`, `apply_climate_moisture_correctors_with` | variant (a) |
| `cartalith_hydrology::carve_channel_network_subcell`, `chaikin_round` | variant (b) |
| `cartalith_hydrology::compute_flow_mfd` | display-only Freeman / Quinn MFD |
| `cartalith_hydrology::build_channels_routed_plain_d8` | ablation |
| `cartalith_erosion::stream_power_kernel_rho8`, `rho8_factor` | Rho8 |

Notes:

- `StreakSpikeParams` has **no `PARAMS` / `JS_PATHS` row, deliberately**. See
  its doc: a row would make a measurement switch saveable and settable from
  the shell.
- Every existing entry point delegates with the off value, so the shipped
  arithmetic runs by control flow.

**Tests.**

- Unit tests:
  - climate: 2;
  - hydrology: 7 (sub-cell guarantees, sub-cell geometry, deep traced cells,
    Chaikin literal points, MFD mass and smoothness, MFD weights, plain-D8);
  - erosion: 2 (the Rho8 distribution, the Rho8 aspect);
  - engine: 1 (every switch reaches generation);
  - harness: 2 (`metric_calibration`, every switch off at both boundaries).
- **Mutants: 17 run; 16 killed, 1 survived.**
  - Two first survived and are now killed by new tests: the Chaikin
    quarter-points and the MFD weights.
  - The survivor removes the sub-cell carve's explicit step 1 (the traced cells
    cut to their floors). On every fixture tried, step 2's overlapping
    sections already cut those cells at least as deep, so the step is kept as
    the stated guarantee but is not shown to be load-bearing.

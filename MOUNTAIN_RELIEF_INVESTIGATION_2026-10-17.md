# Mountain relief investigation — 2026-10-17

Findings for **Ruling CI item 3** ("investigate mountain relief first"). This is
an investigation, not a scope and not a status record. It has no status column;
where it and `cartalith-native/docs/STATUS.md` disagree, `STATUS.md` wins.
Nothing was built, and no Rust, GDScript or generated output was changed,
until §7 (2026-10-19), which records the H1 and H2 build under Ruling CU item 1.

It follows `RENDER_TARGET_GAP_ASSESSMENT_2026-10-17.md` (commit `adb12bf5`).
That assessment found that the highest ground reads as a snow blob with no
ridges at any zoom. It called the height field the likely limit, but only as
"suggestive, not established". This document tests that claim. The target is
`design/vision-render-2026-10-17/target_map_render.webp`. Every image named
below is in `cartalith-native/docs/evidence/mountain-relief-2026-10-17/`, and
every image was opened and looked at.

## 0. Verdict in one paragraph

**The height field is the dominant limit.** None of the nine worlds measured has
a mountain range in the sense the target shows: a dendritic system of ridges,
spurs and valleys. The things that stand highest are:
- smooth, single-crest plate-boundary swells, about 1.6 km high and 17 km wide
  at half height, with flanks of about 5°;
- volcanic cones;
- plateaus, some of them clipped flat at the 4 000 m ceiling.

In the top tenth of land by height:
- the median slope is **0.3–2.8°**;
- crest cells are **no more common** than in the lowlands;
- channels are **rarer** than in the lowlands (real mountains have more).

Two further causes stack on top:
1. **The detail synthesis is switched almost off on this ground.** Both
   amplifiers scale their noise by the coarse cell-to-cell gradient × 8. That
   factor reaches its cap (1) on **0 %** of the top-decile cells, and its
   median there is **0.003–0.023**. So sub-cell detail adds about 3–24 m of
   relief, where the full amplitude is about 1 km.
2. **The material rule cannot show rock on this ground.** Snow depends on
   temperature alone and multiplies rock out at any slope. Slope-driven rock
   needs per-cell slopes of 21–69°, which these surfaces do not have. Natural
   Vibrant also ships `rock_slope = 0`.

No appearance control brings ridges back. All 21 one-at-a-time appearance arms
were tried, over 7 mountain sites and 4 zooms. The arms that move the most
pixels (light gain, AO, the detail-band weights) change tone, not structure.

## 1. Method

- **Build.** I used the existing debug `cartalith-native/target/debug/cartalith_godot.dll`,
  built 2026-10-17 21:47. It was copied into a scratch tree made by
  `git archive HEAD` (`0c768d17`), with the project renamed. The tree ran
  `--headless --import` once (exit 0), then the probe ran **windowed** in
  Godot 4.7.1.
  - **The DLL is older than HEAD.** Two commits after it touch `.rs` files:
    - `d718d052`, whose message says test and comment changes only;
    - `0c768d17` (EF-9.4), whose conformed-tile path is skipped when
      `conform == 0`, and every tile here is drawn with `conform == 0`.
  - I did not rebuild, so this is an inference from those two commits and is
    not verified.
  - Whether the DLL contains `b72b7a1e` and `295623ee` (21:44 and 21:45) is
    **not established**. Neither touches rendering or the paths measured here.
- **Harness.** A scratch probe, not committed. It creates a `WorldGen` and then:
  - calls `set_sea_level(0.42)`, as the assessment did;
  - generates with `generate_world_structure_sized` (or `generate_sized` for
    classic worlds) at 2048 × 1311, then calls `warm_render_caches()` and
    `set_look("Natural Vibrant")`;
  - exports a 16-bit `export_heightmap_png` at 2048;
  - draws the deep-zoom tiles directly with `lod_synthesize_tile(z, col, row)`.
    That is the shell's own tile producer: `pyramid_tile` amplification, then
    `render_biome_tile_rgba`.

  At each site it draws a 2 × 2 tile mosaic (512 × 328 px) at
  **z = 3, 5, 7 and 9**, which is **1, 4, 16 and 64 px per cell**. That is the
  assessment's fit / ×4 / ×16 / ×64 ladder (0.65 / 1.9 / 7.4 / 29.7 px per
  cell), one step finer at each level. `export_snapshot_png` at about
  15.5 px per cell gives the magnified grid without synthesized detail. Sea
  level and metres-per-unit were solved from two `sample_cell` reads per world.
- **Worlds.**
  - The ablations ran on four: the assessment's three (314159 earth 800 km,
    24601 archipelago 800 km, 24601 earth 3000 km), plus 71077345 classic
    800 km. That world was added because it has a visible plate-boundary belt.
  - The height statistics ran on nine. The extra five are 483920 classic 800,
    246371 earth 800, 24601 classic 800, 314159 classic 3000 and 483920
    supercontinent 3000.
  - The stronger-stream-power arms ran on three of them.
- **Sites.** There are seven, chosen without looking at any metric under test.
  - **S1** is the highest 33 × 33-cell box mean, which is the assessment's
    "highest ground".
  - **S2** is the same rule, restricted to boxes with no cell clipped at 1.0.
  - The 71077345 site is the belt crest at x = 1880. The 3000 km S2 is the
    crest of its inland ridge. Both were read off the height field's own
    maximum along a transect.
- **Generation arms.**
  - **A**: app defaults.
  - **B**: `geology_processes` on.
  - **C**: `geology_processes` plus `relief_model`, which is GF-13's arm C.
  - **D**: `tect.ridged` toggled. It is on by default, so D is off.
  - **E / F / G**: `stream.deposit` 0, then also `stream.iters` 40, then also
    `stream.k` × 2. The defaults are `k` 0.012, `iters` 15, `deposit` 0.3 and
    uplift 0.
- **Appearance arms** (on A, all with `river_opacity = 0` so strokes do not
  pollute the statistics; one `base_rivers` frame keeps them):
  - geometry: `grey` (`bio_blend` 0, the relief-only grey), `hill0` / `exag2x`
    / `exag4x` (exag 0, ×2, ×4 of 3.4);
  - light: `gain06` / `gain20` (`relief_gain` 0.6 / 2.0 against 1.16), and
    `ao0` / `ao1` (AO, SVF and shadow all 0 / all 1);
  - shading terms: `curv0` / `curv1` (curvature plus crest), `tex0` / `tex1`
    (`tex`, `geo_micro`, splat and ridged), `rock0` / `rock1` (`rock_slope` and
    `litho_exposure`), `ice0` / `ice1`;
  - the detail bands: `det0` / `det1` (macro, meso and micro weights plus
    `detail_scale_strength`), and `lc1` (`local_contrast` 1).

  **There is no appearance control for the snow line.** Snow is
  `material_weights`' temperature term, so the snow rule was read at the
  symbol, not ablated.
- **Statistics.** Each is computed on the luminance of a crop's central 70 %:
  mean gradient, its 95th percentile (p95), `hf`, the luminance standard
  deviation, the sub-cell variance share, the edge fraction, a snow fraction
  and a grey-rock fraction.
  - `hf` is the standard deviation of the image minus a 2-px Gaussian blur.
  - The sub-cell variance share is the assessment's method.
  - The edge fraction is the share of pixels with a gradient above 6.
  - The snow fraction counts pixels with L > 225 and chroma < 25.
  - The grey-rock fraction counts pixels with chroma < 22 and 60 < L < 215.
  - Height statistics are on the top decile of land by height (the "mountain"
    mask, chosen by height, not by any structure measure), against the rest of
    the land. They include:
    - slope in degrees, from real cell size and metres per unit;
    - crest and valley cells, meaning a strict profile maximum or minimum along
      at least 2 of 4 axes;
    - a D8 channel fraction, meaning accumulation of at least 32 cells.
- **What this cannot establish.**
  - The target is one image at one scale, not a rendered world. Its numbers are
    an **estimate** for the shading-side metrics only, taken from four 96-px
    crops of its mountain areas. Its real-world scale is unknown.
  - No real DEM was available, so "what a real range measures" is not measured
    here.
  - One build, deterministic: each number is one generation, not a median of
    runs.

## 2. Question 1 — where is the limit?

### 2.1 The height field (dominant)

From the nine-world survey (`hstats`, app defaults). "Top" is the top decile of
land by height; "rest" is the other 90 % of land.

| World | Land | Elev. p90 / p99 (m) | Top slope p50 / p90 (°) | Top share > 30° | Crest frac top / rest | Channel frac top / rest |
|---|---|---|---|---|---|---|
| 314159 earth 800 | 0.30 | 1 706 / 2 786 | 2.7 / 16.0 | 3.2 % | 0.046 / 0.052 | 0.067 / 0.115 |
| 24601 archipelago 800 | 0.15 | 3 084 / 3 828 | 1.0 / 16.4 | 2.6 % | 0.053 / 0.060 | 0.046 / 0.090 |
| 24601 earth 3000 | 0.30 | 1 227 / 2 551 | 0.8 / 5.4 | 0.0 % | 0.061 / 0.073 | 0.053 / 0.097 |
| 483920 classic 800 | 0.59 | 1 807 / 2 318 | 1.1 / 7.2 | 1.5 % | 0.046 / 0.045 | 0.083 / 0.136 |
| 71077345 classic 800 | 0.58 | 1 468 / 2 644 | 2.3 / 10.0 | 1.2 % | 0.045 / 0.053 | 0.094 / 0.117 |
| 246371 earth 800 | 0.30 | 1 581 / 2 670 | 2.8 / 15.7 | 0.5 % | 0.047 / 0.050 | 0.057 / 0.098 |
| 24601 classic 800 | 0.85 | 2 818 / 3 046 | 0.3 / 4.1 | 0.4 % | 0.051 / 0.051 | 0.080 / 0.119 |
| 314159 classic 3000 | 0.74 | 2 344 / 3 240 | 0.3 / 2.4 | 0.0 % | 0.050 / 0.052 | 0.079 / 0.139 |
| 483920 supercont. 3000 | 0.60 | 669 / 1 909 | 0.5 / 5.2 | 0.0 % | 0.048 / 0.071 | 0.066 / 0.087 |

What the numbers and the images say:
- **The high ground is not steep.** On all nine worlds, the median slope in the
  top decile is 0.3–2.8°. The p90 values of 10–16° come from volcano flanks and
  from river-trench walls. RV-1 carves channels as one-cell trenches with
  near-vertical walls; they are the black-and-white lines in `02` and `03`.
- **No ridge system.** Crest cells are no denser on high ground than on low
  ground: 0.045–0.061 against 0.045–0.073. The channel fraction is **lower** on
  high ground on every world: 0.046–0.094 against 0.087–0.139. A dissected
  range would show the opposite on both counts.
- **What the "mountains" are** (`02`, `03`, `04`):
  - On classic worlds, a plate-boundary belt is a single smooth swell. At
    71077345, x = 1880, it rises 0.236 units (about **1.6 km**) over a
    half-height width of **43 cells (about 17 km)**, so the flanks are about 5°.
  - Archetype worlds show the same kind of narrow single crest. The 3000 km
    inland ridge in `03` is one example; the coastal scarps are another.
  - Volcanic cones are radially gullied and are the one landform with relief
    structure.
  - There are wide plateaus. The highest box in 314159 is a volcano summit
    **clipped flat at 4 000 m**: `sample_cell` gives 4 000 m, 0.0° slope and
    −16.7 °C. Clipped land is 0–0.62 % of land, so it is rare, but it is
    exactly what "the highest ground" picks.
- **The cause, read at the symbol.** The height formula is
  `cartalith_terrain::compute_height`:
  `0.5 + a·(0.40·base + 0.50·t) + fwt·flex + hwt·hetero + b·noise·(0.25 + 0.75·rug)`.
  - `t` is the boundary stress, or the orogeny field. It is a smooth function
    of distance to the boundary, so it produces exactly one crest.
  - The only rough term is `noise`: `pridged` / `ridged` when `tect.ridged` is
    on, which it is by default. Its weight `b` is **uniform over the map**,
    damped only by age, so it does not concentrate ridges in the belt.
  - Arm D (ridged off) confirms that this noise is where what little crest
    structure exists comes from. The top-decile crest fraction falls from 0.046
    to 0.031 (314159), from 0.061 to 0.045 (3000 km) and from 0.045 to 0.037
    (71077345).
  - The orogeny stage (`build_orogeny_field`) runs only with world-structure
    on. Its v2.36 multi-sheet belt is **not built**: `OrogenyParams` has no
    sheet fields, which was checked at the symbol.
- **Erosion does not dissect it.** The shipped stream power runs
  `max(4, round(15 × 0.6)) = 9` iterations, with uplift 0 and deposit 0.3.
  Making it four times stronger (arm G: deposit 0, 40 iterations, `k` × 2)
  moves the height field by an average of only 31–75 m. The changes:

  | Top-decile measure | Shipped → arm G |
  |---|---|
  | slope p50 | 2.7 → 3.0°, 2.3 → 2.8°, 0.8 → 1.1° |
  | crest fraction | 0.046 → 0.064, 0.045 → 0.065, 0.061 → 0.072 |
  | Laplacian std | about × 1.8 |

  `13` shows the 71077345 belt before and after: faint rills appear, and the
  swell stays a swell. With uplift 0, stream power can only incise into relief
  that is already there, and these bands have little of it.
  `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` EF-3 recorded the same mechanism
  (deposit 0.3 refills valley floors).
- **`relief_model` (GF-11 construction) does nothing on archetype worlds.**
  - Arms B and C give **bit-identical** heights on all three archetype worlds:
    14.6 %, 21.4 % and 12.9 % of cells changed against A, all from B's light
    processes, for a mean change of 1.4–3.1 m.
  - The reason is at the symbol. `generate_terrain_inner` records
    `ConstructionSkip::WorldStructureOn` and skips construction when
    `world_structure.enabled`. GF-11 was measured without world-structure.
  - On classic 71077345, C changes 71.5 % of cells (mean 45 m). It raises the
    top-decile crest fraction from 0.045 to 0.053 and slope p90 from 10.0° to
    11.3°.
  - At the belt it **doubles the edge energy** at 16 px per cell: gradient
    2.59 → 4.50, edge fraction 0.10 → 0.31. But what it draws is **stepped
    terraces and scarps** (`07`, row "C base"), not ridges, spurs or a
    drainage-aligned range. That is what GF-11 was designed to make (caps
    standing behind scarps).
- **Shading-independent check.** `02` and `03` are not the product's renderer.
  They are a plain hillshade I computed straight from the exported heights. They
  show the same smooth swells, so no light model could pull ridges out of these
  surfaces.

### 2.2 The detail synthesis (a second, independent cut)

Both `amplify_region` (`refine_tile`) and `add_zoom_detail` scale their noise by
`relief = min(1, hypot(gx, gy) × 8)`. Here `gx` and `gy` are half central
differences of the **coarse field in field units per coarse cell**.

| World | `relief` p50 / p90 (top decile) | Share at 1 | Detail amplitude at p50 (`amplify`, full = 0.14 units) |
|---|---|---|---|
| 314159 earth 800 | 0.021 / 0.127 | 0 % | 21 m (full 990 m) |
| 24601 archipelago 800 | 0.004 / 0.073 | 0 % | 7 m (1 767 m) |
| 24601 earth 3000 | 0.012 / 0.076 | 0 % | 24 m (2 040 m) |
| 483920 classic 800 | 0.008 / 0.057 | 0 % | 8 m (966 m) |
| 71077345 classic 800 | 0.018 / 0.080 | 0 % | 18 m (966 m) |
| 246371 earth 800 | 0.023 / 0.133 | 0 % | 22 m (925 m) |
| 24601 classic 800 | 0.003 / 0.033 | 0 % | 3 m (966 m) |
| 314159 classic 3000 | 0.010 / 0.071 | 0 % | 9 m (966 m) |
| 483920 supercont. 3000 | 0.009 / 0.097 | 0 % | 13 m (1 532 m) |

The `× 8` is the reference's number, and it is **per coarse cell**. At 2048
wide the per-cell gradient is several times smaller than at the reference's
grids; `render.rs` documents about 6× from 512² to 2048² for the same slope
measure, on `TerrainAppearance::rock_slope`'s note. So on the shipped grid the
taper keeps detail at 0.3–2.3 % of its amplitude on mountain ground.

The detail noise is `fbm` (`AmplifyOpts::ridged` is `false` in the LOD path),
so what does arrive is isotropic "stucco", not crests. `11` compares the
magnified grid with no synthesis (`export_snapshot_png`) against the LOD tile at
16 px per cell, and against the same tile with exag × 4:
- the synthesis adds a fine grain;
- exag × 4 makes that grain visible, at gradient 3.73 against 1.48;
- it never adds a ridge or a valley.

This is separate from Ruling CI item 1's re-key, which changes the noise
**frequency and octave decay**. The **amplitude taper** is a second scale bug of
the same kind, and re-keying the frequency alone will not lift it.

### 2.3 Shading and materials (real, but downstream)

Ladder medians over the 7 sites, A base with rivers off:

| Zoom (px per cell) | grad | `hf` | lum std | sub-cell share | unique colours |
|---|---|---|---|---|---|
| z3 (1) | 3.17 | 3.98 | 25.3 | — | 15 854 |
| z5 (4) | 2.50 | 3.51 | 18.3 | 0.009 | 8 189 |
| z7 (16) | 1.48 | 2.49 | 14.5 | 0.030 | 2 357 |
| z9 (64) | 1.94 | 2.82 | 8.1 | 0.517 | 756 |

Tonal range collapses with zoom: luminance std goes 25 → 8. At z9 the sub-cell
share is high, but it is high **in a picture with almost no variance**: it is
the fbm grain above. Arms B, C (on archetype worlds) and D sit on the same
ladder within a few percent, except at the specific sites named in §2.1.

The appearance ablation, as median mean-absolute RGB change against base at
z5 / z7 (`09`, `10`):

| Arm | Change z5 / z7 | What it did to mountains |
|---|---|---|
| `det0` (all detail bands 0) | 68 / 84 | removes the light entirely: dark flat map |
| `ao1` | 34 / 39 | black blobs in every hollow; no ridges |
| `gain20` / `gain06` | 33 / 48, 33 / 40 | brighter or darker overall; `gain20` lowers the z9 gradient (1.94 → 1.32): the washout the assessment saw |
| `det1` | 18 / 24 | brighter, flatter |
| `grey` (geometry view) | 10 / 9 | the same smooth swells, in grey |
| `exag4x` | 4.2 / 4.4 | brings up the fbm grain (z7 gradient × 2.5); no structure |
| `rock1`, `curv1`, `tex1` | 2–4 | `rock1` greys the belt flanks slightly (`09`); `curv1` adds crisp bank edges |
| `ice0`, `ice1`, `lc1`, `curv0`, `tex0` | 0–1 | inert here. `lc1` moved **0** tile pixels at every level (§5) |

No arm produced a ridge on any site or zoom.

Two shading rules make the snow blob in particular, read at the symbol in
`render.rs::material_weights`:
- **Snow is temperature-only.** It is `snow = smoothstep(3, −5, t)`, and the
  budget left after it is `bud = 1 − snow`. Rock is then computed as
  `… × bud`, so full snow suppresses rock at **any** slope. There is no shedding
  of snow from steep faces.
- On the 314159 plateau (S1) the snow fraction is **0.93–0.97** at 16 px per
  cell in every arm, including `exag4x`, `ao1` and `rock1`.
- In the target, snow is a thin ridgeline accent: 1–4 % bright-neutral pixels
  by the same test (an estimate), against 23–52 % grey rock.

**Slope rock is off, and could not act if on.** Natural Vibrant ships
`rock_slope = 0.0`, together with `svf_strength 0`, `shadow_strength 0` and
`ao_strength 0.2`. The rock knee is the reference's `slope / 0.08` per cell. On
these grids that means **21–69°** of real slope. In the top decile,
`rock_slope_mix(slope, 1)` has a p90 of only 0.012–0.095. The material term's
own rock exposure (`sl^1.8`, before the vegetation factors) has a p90 of
0.004–0.047.

### 2.4 Against the target (estimate, shading-side only)

Four 96-px crops of the target's ranges (T1–T4) against 96-px crops of the
engine's fit image, resampled to the target's 803-px width, at the mountain
sites (`01`):

| Metric | Target T1–T4 (estimate) | Engine sites (fit scale) |
|---|---|---|
| mean gradient | 23.8–26.5 | 6.2–15.1 |
| `hf` | 26.7–29.1 | 7.0–20.4 |
| lum std | 42.9–48.5 | 19.4–51.5 (the high end is coast or frame contrast in the crop) |
| grey-rock fraction | 0.23–0.52 | 0.000–0.021 |
| bright snow fraction | 0.01–0.04 | 0.000–0.135 |
| edge fraction (gradient > 6) | 0.87–0.90 | 0.40–0.65 |

The engine crops that score high on gradient or std do so from coasts, river
trenches or the plate frame, not from relief. The rock and snow rows are the
clearest: the target is mostly rock with a little snow, and the engine is
almost no rock with blobs of snow.

## 3. Question 2 — rows, minimal fixes, what moves, ranking

| # | Cause | Existing row or scope | Minimal fix | Goldens and pins it moves | Effort | Impact |
|---|---|---|---|---|---|---|
| H1 | **Orogens are single smooth swells; no ridge or valley dissection** | `OUTSTANDING_WORK.md` §2.9 "Build multi-ridge orogenic belts" (Ruling AP, **scheduled, not built**; `RC_ENGINE_CHANGES.md` §6b). GF-11 / GF-13 (`relief_model`: scarps, not ranges). EF-3 (`tile_erode`, built, **not wired**). **No row** for fluvial dissection of the uplifted belt | Two steps. (a) Build the multi-sheet belt behind a flag; that gives several crests. (b) A new **belt-localised dissection** pass: stream power with uplift > 0 on the orogeny mask, and/or the ridged-noise weight `b` scaled by the orogeny field rather than uniform. Arm G shows that uniform erosion alone is too weak | Every height golden downstream if on in `WorldParams::defaults()`. So: on only in `cartalith_godot::params::defaults()`, with the JS-parity goldens kept (`MISTAKES.md` "Change generated output"). It also moves rivers, lakes, landmarks and civ placements on app worlds, which needs a disclosed app-world re-baseline | large, multi-batch | **highest**: nothing downstream can show a range that is not in the height field |
| H2 | **Volcano summits clipped flat at 1.0** | none | Let the stamp saturate smoothly (a soft knee toward 1), or rescale after stamping | Volcano goldens (`stamp_volcanoes_*`), via the same app-side flag | small | low (0–0.62 % of land) but it is exactly "the highest ground" |
| H3 | **`relief_model` skipped under world-structure** | GF-13, Ruling CG (off) | Feed construction the orogeny term, or accept the skip | GF goldens; GF-13 bars | medium | low for ranges: it makes scarps |
| S1 | **Detail amplitude taper per coarse cell (× 8)** | Ruling CI item 1 re-key (frequency and decay), LOD-D5, EF-0 / EF-2 | Key the taper to real slope (m / km) instead of per-cell units, and use `ridged` detail where the orogeny or stress field is high. Same app-side divergence; `js_reference` keeps × 8 / fbm | LOD-D1's reference tile golden (only if not gated), `pyramid_tile` / bake goldens, LOD-D5 harness numbers | small to medium | high **once H1 exists**; on today's swells it adds believable roughness, not ranges |
| R1 | **Snow overrides rock at any slope** | LOD-D4 (closed) and the §2.5 snow-aspect row; Ruling CI item 2's "Satellite relief" look | In the new look only: shed snow by real slope, e.g. `snow × (1 − smoothstep(35°, 55°, slope))`, freeing budget for rock | Look digests for the new look only; `FINISHED_RENDER_FNV1A` stays if gated by an appearance field at 0 | small | medium; needs H1 / S1 slopes to show |
| R2 | **Slope rock off (`rock_slope = 0`) and its knee per cell** | `TERRAIN_APPEARANCE_SCOPE.md` milestone 5; `MAP_STYLE_RESEARCH.md` (no satellite preset) | In the new look: `rock_slope > 0`, and a km-normalised knee as a **new** appearance field (the ported knee stays) | None if new field; new look digests | small | medium, same dependency |
| R3 | Light rig (AO / SVF / shadows off, gain) | `TERRAIN_APPEARANCE_SCOPE.md` milestone 2 | Tune inside the new look after H1 | look digests | small | low on today's terrain: measured, it changes tone only |

**Ranking (impact ÷ effort):**
1. **S1** is small, and it is the zoom requirement's own lever. Do it with Ruling
   CI item 1's re-key, as one re-baseline rather than two.
2. **H1** is large but decisive. Start with a measurement spike, not a build:
   belt-localised `b` and uplift-on stream power, measured with this document's
   height statistics on the nine worlds.
3. **R1 + R2** are small and belong to the Satellite relief look. Build them
   after S1 so they have slopes to act on.
4. **H2**, then H3 and R3.

## 4. Question 3 — owner questions, with recommended defaults

1. **Is making real mountain ranges in the height field (H1) in scope for "the
   render"?** It is generation, not rendering, and it moves app worlds.
   *Recommended:* yes, as a new row. Start with a measurement spike: localise
   the ridged-noise weight and an uplift-on stream-power pass to the orogeny or
   stress field, behind an app-side flag. Then build the scheduled multi-sheet
   belt. Bars: the top-decile slope p50 at least doubles, the crest fraction
   and the channel fraction exceed the lowland figures, and frames are looked
   at, on at least 3 of 5 seeds.
2. **Fold the amplitude taper (S1) into Ruling CI item 1's re-key**, so the
   pyramid goldens are re-baselined once? *Recommended:* yes, with `ridged`
   detail gated by the stress or orogeny field. Disclosed per DECISIONS §7p;
   `js_reference` unchanged.
3. **Snow shedding by slope and slope rock (R1 / R2) in "Satellite relief"
   only**, leaving Natural Vibrant and the reference rule alone? *Recommended:*
   yes.
4. **`relief_model` under world-structure (H3).** *Recommended:* leave it
   skipped. It makes caprock scarps, not ranges, and Ruling CG keeps it off
   anyway.
5. **Volcano clip (H2).** *Recommended:* a soft knee, app-side, in the same
   re-baseline as H1.

## 5. What I could not establish

- Whether a range built by H1 would read like the target. Only an
  implementation can show that; the stronger-erosion arms are the nearest proxy
  and they are weak.
- Real-terrain reference values. No DEM was available. Any comparison to real
  slopes or drainage density is general knowledge, not a measurement here, and
  none is used in a bar above.
- Whether the target's crops are at a scale comparable to 1 px per cell. The
  target statistics are an estimate for the shading side only.
- Whether `local_contrast` is meant to act on deep-zoom tiles. `lc1` moved 0
  tile pixels at every level, and I did not trace why.
- Whether the DLL contained `b72b7a1e` and `295623ee` (see §1). It is stale
  against HEAD by two commits; I judged both not to affect this measurement but
  did not verify it by rebuilding.

## 6. Evidence index (`cartalith-native/docs/evidence/mountain-relief-2026-10-17/`)

| File | Shows |
|---|---|
| `01_target_vs_engine_mountains_fitscale.jpg` | the target's four mountain crops above four engine mountain sites at the same 803-px scale |
| `02_height_hillshade_s314159_earth800.jpg` | a plain hillshade of the exported heights (not the product renderer): plateaus, trenches, volcanoes, no ranges |
| `03_height_hillshade_s71077345_classic_and_s24601_3000km.jpg` | the same for the classic belt world and the 3000 km world: single smooth crests |
| `04_sites_colour_vs_height.jpg` | the six archetype-world sites, colour beside height hillshade |
| `05`–`08`, `12` | z3 / z5 / z7 / z9 ladders (1 / 4 / 16 / 64 px per cell), rows A base with rivers, A grey, A exag × 4, A AO 1, C base, D base |
| `09_ablation_grid_s71077345_belt_4pxcell.jpg` | all 21 appearance arms on the belt at 4 px per cell |
| `10_ablation_grid_s314159_plateau_4pxcell.jpg` | the same on the snow plateau |
| `11_snapshot_vs_lodtile_16pxcell.jpg` | magnified grid without synthesis / LOD tile / LOD tile with exag × 4, at three sites |
| `13_streampower_default_vs_strong_s71077345_belt.jpg` | hillshade of the belt, shipped stream power above arm G (4× stronger) below |

## 7. The build (2026-10-19, Ruling CU item 1)

Ruling CU item 1 un-parked H1, H3 and H2 at §4's defaults. Each cause was
re-opened at its symbol first, and every number below was measured in this
pass (`cartalith-native/docs/evidence/orogen-relief-2026-10-19/measurements.txt`
has every line and every arm).

### 7.1 The causes, re-opened

- **H1 holds.** `cartalith_terrain::compute_height` is unchanged since §2.1:
  `t` is the stress (classic) or `oro + min(stress, 0)` (world structure), a
  smooth one-crest term, and `b·noise` is uniform. A Rust rebuild of §1's
  hstats (`cartalith-godot/tests/mountain_relief.rs`) on HEAD `747b94c3`
  reproduces §2.1 within rounding (314159 earth: p90 / p99 1 706 / 2 786 m, top
  slope p50 2.72°, crest 0.048 / 0.054, channel 0.068 / 0.117), so the two
  definitions agree. A plain hillshade of 71077345 still shows the single
  smooth swell (`evidence/orogen-relief-2026-10-19/01_...`).
- **H2 holds.** The clip is inside `stamp_one_volcano` (per stamp,
  `stored > 1.0 → 1.0`), then a whole-field `clamp(0, 1)` after the craters.
- **H3 is left as §4 Q4 recommends.** `generate_terrain_inner` still records
  `ConstructionSkip::WorldStructureOn`, and construction stays off in the app
  (Ruling CG). The new term runs before construction, so on a classic world a
  user who turns on the geology processes and `relief_model` gets construction
  over the ridged surface. Nothing was changed there.
- **S1, R1 and R2** were built earlier (`km_relief_taper`, and Satellite
  relief's snow shedding and slope rock). They were not re-opened.

### 7.2 What was built

One switch, `tect.orogen_relief` (`TectonicParams::orogen_relief`). It is
**off** in `WorldParams::defaults` (and so under `js_reference` and every JS
golden) and **on** in `params::defaults()`. A save without the key reloads off
(DECISIONS §7p's pattern). With it on, `generate_terrain` does five things:

1. **Belt** (`cartalith_terrain::orogen::belt_weight`): `compute_height`'s own
   `t`, positive part, spread by three box blurs of 30 km, normalised by its
   99.5th percentile, smoothstep 0.10..0.55.
2. **Relief** (`apply_orogen_relief`), after `normalize_field` and before
   volcanism: a 400 m lift plus a zero-mean ridged multifractal (Musgrave) of
   1 500 m amplitude over gradient noise, first wavelength 20 km, octaves down
   to two cells, faded in over 300 m above the coast. Land is never lowered
   below sea + 10 m and the sea is untouched.
3. **Dissection** (`cartalith_engine`'s `orogen_dissect`): 24 iterations of
   stream power (K 0.03, no deposition, no uplift) inside the belt, with
   everything outside it and every sea cell pinned and the routing rebuilt
   every 4 iterations (`cartalith_erosion::stream_power_kernel_pinned_refreshed`).
4. **H2:** volcano stamps keep what they add above 1 (`soft_top`), and the
   field saturates with `orogen::soft_ceiling` (identity to 0.9, then
   `0.9 + 0.1·tanh`) before the clamp.
5. **Late belt fill:** after every other stage, the belt's closed hollows are
   filled to their spill level with the Ruling CO fill's Priority-Flood+ε. This
   includes below-sea hollows that the open sea does not reach; summit calderas
   are kept.

What the first builds got wrong, measured and fixed:
- Ridges on the raw `t` sat only on the swell's crest, so the belt is spread.
- Value noise drew rectilinear crests, so the term moved to gradient noise.
- Fitting each volcano under the ceiling shrank coastal shields and turned
  4 317 land cells to sea on 71077345. It was reverted, and the soft ceiling
  replaced it.
- Without the late fill, 1-3-cell lakes rose 2.3-8× on every world (for
  example 24 → 131 and 61 → 355). Without the open-sea test, 24601
  archipelago's enclosed gorge floors still made 97 against 16.

### 7.3 §4 Q1's bars, nine worlds (before → after)

| World | Top slope p50 (°) | Crest top / rest | Channel top / rest | Flat share of top |
|---|---|---|---|---|
| 314159 earth 800 | 2.72 → 9.80 | 0.048/0.054 → 0.146/0.068 | 0.068/0.117 → 0.048/0.108 | 0.144 → 0.051 |
| 24601 archipelago 800 | 0.98 → 3.51 | 0.056/0.059 → 0.120/0.082 | 0.048/0.094 → 0.039/0.091 | 0.225 → 0.138 |
| 24601 earth 3000 | 0.83 → 2.64 | 0.064/0.076 → 0.155/0.083 | 0.053/0.097 → 0.055/0.093 | 0.433 → 0.387 |
| 483920 classic 800 | 1.07 → 5.98 | 0.049/0.047 → 0.144/0.057 | 0.083/0.137 → 0.059/0.129 | 0.340 → 0.086 |
| 71077345 classic 800 | 2.32 → 7.52 | 0.046/0.055 → 0.138/0.065 | 0.096/0.119 → 0.066/0.108 | 0.152 → 0.038 |
| 246371 earth 800 | 2.82 → 9.46 | 0.048/0.053 → 0.152/0.067 | 0.057/0.100 → 0.042/0.087 | 0.049 → 0.030 |
| 24601 classic 800 | 0.34 → 0.40 | 0.054/0.054 → 0.083/0.066 | 0.081/0.121 → 0.078/0.113 | 0.646 → 0.569 |
| 314159 classic 3000 | 0.32 → 0.47 | 0.053/0.054 → 0.131/0.062 | 0.080/0.140 → 0.070/0.132 | 0.572 → 0.505 |
| 483920 supercont. 3000 | 0.46 → 0.77 | 0.050/0.077 → 0.118/0.079 | 0.066/0.089 → 0.079/0.086 | 0.511 → 0.468 |

- **Slope at least doubles: 6 of 9** (3.2-5.6×). The three that miss have a
  top decile that is mostly plateau rather than belt (flat share 0.47-0.57
  even after): 24601 classic at 800 km (85 % land) and the two 3 000 km
  classic and supercontinent worlds.
- **Crest fraction above the lowlands: 9 of 9** (before: 2 of 9, by hairs).
- **Channel fraction above the lowlands: 0 of 9 (before: 0 of 9). This bar
  fails.** It is also the wrong proxy, measured. Inside the belt (weight ≥ 0.5,
  rebuilt from the stress field, an input, on the four classic worlds) the D8
  `A ≥ 32` fraction **falls** with dissection (71077345 0.178 → 0.091). A
  smooth flank routes every cell down long parallel lines that all pass 32
  cells, while a dissected range gathers flow into fewer, larger valleys.
  Valley cells (strict profile minima), which track dissection directly,
  exceed the lowlands on 7 of 9 (before: 0 of 9). This is an owner decision.

### 7.4 What derives from height (regression check, nine worlds, off → on)

From `cartalith-civ/tests/mountain_relief_civ.rs`:
- **Rivers.** RV-1's B8 count of ocean cells on river paths is 0 → 0 on all
  nine. The lake share of path cells moves both ways by world (5.57 → 2.36 %,
  0.74 → 1.51 %, 5.05 → 8.84 %, 1.61 → 0.38 % …). **1-3-cell lakes** go from
  24/16/21/28/27/33/61/61/32 to 30/10/11/27/24/25/57/57/34. That is within
  GF-6's 1.25× bar on all nine, with 314159 earth exactly at it.
- **Watersheds.** Each land cell's drainage terminal agrees on 97.4-99.2 % of
  land. The land mask agrees on 99.09-99.98 % of cells; archetype worlds
  re-anchor their sea level after volcanism, so their coasts move most.
- **Settlements.** The count is unchanged at 40, except 24601 archipelago
  (38 → 36). 28-68 % of sites lie within 3 cells of a site in the before world,
  and the median nearest distance is 0-31 cells. Siting is re-run on new
  terrain, so this is a re-siting, not a code change. Nothing in the siting
  code changed.
- **Landmarks.** Totals move by at most 10 per world (24601 classic 215 → 205). The kinds that move are
  waterfalls (12 → 18 and 8 → 10, but 14 → 9 on 24601 classic), river confluences, harbours (24601
  archipelago 9 → 4) and lakes (fewer lake landmarks where belt hollows were
  filled: 16 → 10 on 24601 earth 3000).

### 7.5 Zoom ladder and the target

Frames at x1/x4/x16/x64 in Satellite relief and Natural Vibrant, before and
after, beside the target, are `04_ladder_*.jpg`. Whole-map and x4 hillshades
are `01`-`03`. They were looked at:
- At x4 the belts read as ranges, with ridges, spurs and valleys and snow on
  the crests (before: a smooth grey swell).
- At x16 the valleys and spurs carry on; at x64 there is rock texture over the
  dissected form.
- At fit (x1) the belts are now textured grey bands. They are still narrower
  and less extensive than the target's ranges, which cover much of each
  continent; ours are where the plate boundaries are.
- The mean luminance gradient on the belt pick rises at every rung, in both
  looks, on all three worlds (for example 71077345 in Satellite relief, x4 /
  x16 / x64: 2.06 / 1.98 / 2.82 → 3.82 / 3.72 / 5.41). So zoom gains detail at
  every level there.

Visible defects, stated:
- Filled hollows read as flat tan basins at x4-x16.
- At 3 000 km (1.5 km cells) the ridged term draws meandering, worm-like
  crests at x16.
- The Satellite relief look's slope rock (`rock_slope_deg` 6) now greys most
  of every belt.

### 7.6 Cost and golden effects

- **Cost** (release, run alone, five alternating runs): 71077345 classic goes
  from 3.09 s (3.08..3.12) to 6.44 s (6.42..6.47), and 314159 earth from
  7.48 s (7.47..7.51) to 10.90 s (10.87..10.95). That is about +3.4 s per
  2048 × 1311 world, almost all of it the dissection's six full-grid routings.
  16 iterations (four routings, arm E) costs about a second less and draws
  slightly softer valleys.
- **Goldens.** No JS golden, `js_reference` path or pinned app digest moved
  (`cargo test --workspace`). The only failures were two fixtures that need
  the smooth-belt world, now pinned off with the reason given: `project_bridge`'s
  substrate tests, whose road-edge fixture needs a straight 4-cell stretch,
  and `gf0_geology_harness.rs`, whose GF numbers and pipeline replay predate
  the term.

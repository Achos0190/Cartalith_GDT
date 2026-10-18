# Volcano edifice gullies and 314159's streaks: measurements (2026-10-18)

The owner filed two "measure first" rows under Ruling CM
(`LARGE_ITEM_RULINGS.md`; `OUTSTANDING_WORK.md`):

1. the radial gullies of the morphological volcano edifice, which produced the
   streaks on 71077345's dome in `TERRAIN_STREAK_SPIKE_2026-10-18.md`;
2. the north-south streaks on 314159's coastal slope, to be located with the
   river layer hidden.

This document holds findings, a table per variant, a recommendation and
owner questions. It carries no status; status lives in
`cartalith-native/docs/STATUS.md`.

**Nothing here is switched on.** Every variant sits behind
`cartalith_engine::StreakSpikeParams::edifice`
(`cartalith_terrain::EdificeVariant`). That switch is off in
`WorldParams::defaults` and in `params::defaults()`, and has no `PARAMS`
row. Variant (b) is a tile-time function that nothing in the app calls. The
§7l-ii edifice ruling is the owner's, so changing the edifice at generation
waits for the owner's answer to the questions at the end.

## How it was measured

- **Harnesses** (all `#[ignore]`d):
  - `cartalith-godot/tests/terrain_streak_spike.rs` (`streak_spike_measure`,
    `streak_spike_cost`). It gains eight edifice arms, two worlds (71077345 at
    3 000 km and at 40 075 km), and these output lines:
    - `EDIFICES`: per world;
    - `EDIFICE`: per arm;
    - `WINDOW`: with `SS_WINDOW`.
  - `tests/zoom_detail_ladder.rs`. It gains:
    - `ZD_STREAK` tokens for the new arms;
    - `ZD_FURROWS=1` for variant (b);
    - `ZD_X1=1`, which renders x1 as the base raster;
    - `ZD_ONLY`;
    - volcano, volcano-flank and mountain-flank picks.
  - The windowed shell probe `_zoomdetail_probe.gd`, which gains
    `--rivers on|off` and `--only`.
- **Worlds.** The spike's five worlds at 800 km and 2048 x 1311 under app
  parameters:
  - 71077345 Classic;
  - 314159 Earth;
  - 24601 Archipelago;
  - 483920 Classic;
  - 12345 Classic.

  Two more test scale: 71077345 at 3 000 km and at 40 075 km.
- **Medians.** Medians are over the five 800 km worlds, with min-max in
  brackets.
- **Edifices are recovered from `volcanic_field`.** `EdificeFrame`, in the new
  module `cartalith_terrain::edifice_furrows`, recovers them from the stamp's
  own placement mark. That mark is identical in every arm, so every population
  below is chosen by an input, never by the field under test.
- **Pit metrics** (`depressions`, `small_lakes_split`):
  - "Pits" are closed depressions on the final height field, counted with a
    plain Priority-Flood that is 4-connected, like `build_water_bodies`.
  - "Pits on edifice" are pits of 1-3 cells that have a cell on an edifice
    footprint (`volcanic_field > 0`).
  - "Lakes on edifice" are RV-1's own 1-3-cell lakes from
    `build_water_bodies` that touch a footprint.
  - "All small lakes" is RV-1's count.
- **Dome energy (`dome acrE`).** The band-passed (1-3 cell) across-slope
  gradient energy on edifice footprint land (`volcanic_field > 0.05`). It uses
  the spike's calibrated metric (white noise reads 1.0 after the 1-cell inner
  band; the 0.857 artefact is excluded). It is given relative to head; this is
  the radial-streak number.
- **Identity.** `gf6_carve_identity_hashes` (13 arrays; parity, app and
  processes-on worlds; seeds 483920 and 314159) printed the same hashes before
  this change (origin tip `a984b2c8`) and after it, with every switch off.

## What the edifices are, by scale

The `EDIFICES` lines give the recovered edifices, their size, and each law's
gully spacing at mid-flank:

| World | km/cell | Edifices | Radius km, p10 / p50 / p90 | Shipped spacing km, p10 / p50 / p90 | 1.5 km law, in cells |
|---|---|---|---|---|---|
| 800 km (five seeds) | 0.391 | 7-25 | 2.6-7.1 / 6.3-11.8 / 16.9-28.0 | 1.36-1.55 / 1.55-1.60 / 1.61-1.83 | 3.84 (held) |
| 71077345, 3 000 km | 1.465 | 18 | 4.3 / 9.7 / 25.7 | 2.3 / 5.1 / 5.9 | 1.02 (not held, so removed at generation) |
| 71077345, 40 075 km | 19.57 | 18 | 42.7 / 49.5 / 51.9 | 22.4 / 25.9 / 27.2 | 0.08 (not held) |

- **The shipped law is ground-scale-correct only at 800 km.**
  - At 800 km, 4 cells is about 1.56 km, close to real radial-valley spacing.
  - At 3 000 km it is 5 km.
  - At Earth width every edifice is clamped to the 2-cell minimum radius
    (`stamp_one_volcano`'s `max(2)`: a 2-10 km cinder cone stamped 39 km
    wide), so it carries the 6-gully floor. That is a radius problem, not a
    gully one, and it is separate from this row.
- **71077345's "ice dome" is one large shield**, about 83 km in recovered
  radius. That puts it at the shipped law's 48-gully cap, about 5.4 km apart
  at mid-flank. This is the sunburst in frame 01.

## Where the pits and streaks come from (ablations)

| Arm | Pits on edifice | Pit depth p50 (m) | Lakes on edifice | All small lakes | Dome acrE (rel.) | Outlets within 5 cells (%) |
|---|---|---|---|---|---|---|
| head (shipped) | 618 (241-1143) | 3.1 (2.3-4.5) | 41 (5-170) | 61 (31-176) | 1.000 | -- |
| edifice off | 166 (47-583) | 1.6 | 4 (0-13) | 19 (8-50) | 0.247 (0.139-0.445) | 96.6 (73.8-99.4) |
| gully term off | 150 (45-290) | 1.2 | 4 (3-5) | 25 (10-37) | 0.503 (0.346-0.764) | 98.4 (88.8-99.9) |
| lump term off | 586 (225-1149) | 2.9 | 27 (5-143) | 47 (29-149) | 0.986 (0.972-1.002) | 99.5 (97.8-100.0) |

- **The gully term makes both the pits and the radial streaks.**
  - With it off, lakes on edifices fall 41 to 4 (24601: 170 to 4).
  - The dome's across-slope energy halves.
  - The lump term (`fbm`) does almost nothing to either.
- **Most pits are not lakes.**
  - About 600 closed hollows sit on edifices per world, with a median depth of
    about 3 m. The classifier calls only 5-170 of them lakes; the rest sit
    under `LAKE_DEPTH` or the rain gate.
  - Summit (caldera) lakes are a small share: 0-6 per world, counted within
    0.3 of the recovered radius.
  - So on the flanks the lakes are artefacts of the gully term, while the
    caldera lakes are real closed basins.
- **Where the pits form** (unit test
  `the_flank_pits_and_what_each_variant_does_to_them`, one stratocone on a
  flat plane):
  - all 96 pits sit at `t >= 0.5`, most at 0.7-1.0;
  - there the concave cone's slope is gentlest, and the flank envelope
    `4u(1-u)` shallows the gully faster than the cone falls.
  - It is a **slope ratio, not aliasing**, so band-limiting the noise does not
    remove it (variant (a) below).

## The variants

| Variant | Switch | Pits on edifice | Depth p50 (m) | Lakes on edifice | All small lakes | Dome acrE | Rendered across energy | Outlets within 5 | Rivers |
|---|---|---|---|---|---|---|---|---|---|
| head | -- | 618 | 3.1 | 41 | 61 | 1.000 | 1.000 | -- | 492 (197-527) |
| **(a)** km gullies, 1.5 km, band-limited | `gully_spacing_km = Some(1.5)` | 659 (135-1079) | 8.4 | **204 (4-450)** | **237 (32-456)** | 1.202 (0.960-1.408) | 1.004 | 98.3 (91.0-99.9) | 491 |
| (a) without the band limit (ablation) | `+ full_band` | 1611 | 7.4 | 479 (5-996) | 499 | 1.410 | 1.006 | 98.3 | 497 |
| **(c)** fill flank pits at the stamp | `fill_pits` | 290 (89-401) | 1.4 | 14 (0-22) | 28 (28-48) | 0.981 | 1.000 | 99.87 (99.37-100.0) | 492 |
| **(c-late)** fill flank pits in the final field | `fill_pits_final` | **75 (12-151)** | 4.4 | **10 (0-11)** | **27 (16-42)** | 0.993 | 1.000 | **99.93 (99.57-100.0)** | 492 |
| (a) + (c-late) | both | 114 | 5.1 | 18 (0-67) | 38 (28-73) | 1.161 | 1.004 | 98.3 | 491 |
| (a) + (c) | both | 287 | 1.4 | 25 (2-52) | 45 (22-80) | 1.129 | 1.004 | 98.3 | 500 |
| **(b)** tile-time radial furrows | `ZD_FURROWS` (ladder only) | unchanged (render only) | -- | unchanged | unchanged | -- | -- | unchanged | unchanged |

### Notes on the table

- **Cost.** Full generation on 314159, five runs each, alternated, run alone,
  measured twice:
  - head: 10.81 s (8.12..12.04);
  - (a): 12.37 s (8.18..13.95), then 10.11 s (9.23..14.33);
  - (c): 10.60 s (8.05..13.08), then 11.14 s (10.43..11.55);
  - (c-late): 8.12 s (8.06..14.08), then 10.74 s (9.71..11.64).

  Every range overlaps head's, so **no cost difference is established**.

  Variant (b)'s per-tile cost was not timed. It is at most nine `pvnoise`
  calls per land texel on an edifice flank, and zero elsewhere.
- **Variant (a) at 3 000 km:**
  - at 1.46 km/cell the gully octaves fall below the grid, so the band limit
    removes them at generation;
  - pits on edifice fall 98 to 22, and dome acrE to 0.741;
  - the gullies then exist only at tile time, through variant (b).
- **Variant (a) at 40 075 km:** no change, because every edifice is a 2-cell
  blob.
- **A first build of (a), corrected before these numbers.** It weighted each
  octave at each cell's own radius, so the weight grew outward and deepened a
  gully downslope. That is the pit mechanism itself: 71077345's pits went from
  325 to 422 under the 8-connected metric of the first run. The weights are
  now per edifice, at mid-flank. The numbers above are the corrected build's.
- **Why (c) at the stamp is not enough.** It removes most pits from the
  final field, but the lakes come back: erosion and the carve re-cut hollows
  along the gully lines. (c-late) fills after every stage.
- **Why both fills are 4-connected.** An 8-connected first build left hollows
  whose only outlet was diagonal, and the classifier still pooled them
  (24601's lakes went 170 to only 103). The fill now uses the classifier's own
  4-connected neighbourhood.
- **What (c-late) leaves.** The lakes left on edifices are summit lakes, or
  sit inside 0.4 r, which the rebuilt mask protects (`SUMMIT_T`).

### Frames (read)

Files are in `cartalith-native/docs/evidence/edifice-streaks-2026-10-18/`
(01-04, 06-09; 05 was dropped, see below). Every frame was opened.

Rust ladder, no river layer, shipped kilometre zoom law. x1 is the base raster
(`render::cell_color`), cropped around the pick and enlarged; x4, x16 and x64
are the deep-zoom tile path.

- **01 / 02 — 71077345's dome, summit and mid-flank:**
  - Head is a radial sunburst at x1 and x4. At x16 it shows soft, wide radial
    bands; at x64 blurred vertical bands with no gully inside. These are the
    5.4 km cap-spaced gullies magnified.
  - **(a)** makes the dome's gullies denser and sharper (1.5 km, about 3.5x as
    many on this shield). x64 becomes strong parallel stripes. It reads busier,
    not more believable.
  - **(c-late)** cannot be told from head at any rung (its changes are metres).
  - **(b)** keeps head's x1 and x4. At x16 and x64 it adds fine radial furrows
    inside the bands. This is the only arm in which **x64 gains gully detail**
    on the flank, though it reads a little combed (regular hairlines).
  - (a)+(b) is the busiest.
- **03 / 04 / 06 — volcano flanks on 24601, 314159 and 71077345:**
  - On the small cones, head reads as a smooth cone with a caldera lake.
  - (b) adds radial rills converging on the summit at x16 and x64 (04's x64:
    thin dark lines down the flank, crossing the rocky coastal band).
  - (a) and (c-late) are visually the same as head at x1 to x16.
- **09 — x1 whole map:** the edifice's sunburst is a minor element at fit
  view. The ablations show the dome without gullies (edifice off is a plain
  white swell).
- 24601's 3 000 km volcano pick turned out to be a seamount barely breaking
  the surface. Its frames showed nothing and were dropped; the 3 000 km
  evidence is the numbers above.

### Goldens and pins that would move if switched on

- **(a), (c) and (c-late)** move the height field on edifice ground in every
  app world. That reaches every app-world digest:
  - the app and processes-on lines of `gf6_carve_identity_hashes`;
  - the GF harness bars on app worlds;
  - app-world `cel_toon.rs` digests;
  - landmark and settlement counts that read app worlds.

  No JS-captured golden moves. `volc.edifice_model` is off in
  `WorldParams::defaults`, so the morphological edifice and every variant on
  it are unreachable under the parity parameters.
- **(b)** moves no generation output. Wired in, it would move deep-zoom tile
  bytes on edifice flanks only (LOD tile digests, if pinned on an app world).

## 314159's north-south streaks: they are the rivers

- **Frames 07 and 08.** The windowed shell (this branch's DLL; generation is
  byte-identical to origin tip) shows 314159's forest pick (996, 340) and
  coast pick (940, 652) at x1, x4, x16 and x64, with the river layer on and
  off (`set_layer_visible("rivers")`, which hides both the overlay stroke and
  the in-tile river paint).
  - With rivers on, x16 shows the perfectly straight channels running due
    south to the coast (the comb).
  - **With rivers off, no north-south line remains.** The surface is the
    forest and steppe mottle, and the short **diagonal** dash hatching already
    filed as "dark diagonal hatching".
- **Numbers.** `SS_WINDOW=996,340,64` measures the north-south band-energy
  ratio (`ns_ratio`), calibrated: north-south gratings read above 20,
  east-west below 0.05, noise 1 ± 0.1.

  | Arm | Height ns | Luminance ns (no river ink) | Flow ns |
  |---|---|---|---|
  | head | 1.479 | 1.073 | 2.943 |
  | carve off | **0.211** | 0.443 | 3.503 |
  | no light pass | 0.508 | 0.760 | 2.999 |
  | no channel cut | 0.562 | 0.423 | 3.055 |
  | no glacial | 1.479 | 1.056 | 2.883 |
  | edifice off | 1.469 | 0.990 | 2.954 |

  - The north-south structure in the height is **the generation carve's**:
    the D8 flow runs due south (flow ns 2.9) and both carve parts cut along
    it. The light pass and the channel cut each remove most of it.
  - In the rendered terrain without river ink it reads **1.07, close to
    isotropic**. Nothing visible remains with the strokes hidden.
- **Conclusion.** The streaks are the comb in "Coastal and lake-shore river
  combs that remain after Ruling BV": straight D8 channels down a uniform
  coastal ramp, drawn as strokes. No separate terrain-streak stage exists.

## Recommendation

1. **Do not ship (a) as built.** At 800 km it multiplies flank lakes by five
   (61 to 237 small lakes; 24601: 170 to 450). Two causes:
   - it drops the shipped law's 48-gully cap, so large shields (and the dome)
     get several times as many gullies;
   - the remaining resolved octaves still form closed basins.

   Its one real win is at coarse scales, where it removes the gullies the grid
   cannot hold. That is worth keeping only together with (b).
2. **(c-late) is the effective pit fix.**
   - Lakes on edifices fall 41 to 10, and all small lakes 61 to 27, close to
     edifice-off's 19.
   - It is invisible at every zoom and costs nothing measurable.
   - Outlets agree within 5 cells on 99.93 % of land.

   Switching it on is an owner call (it moves every app world's height by a
   few metres in pits) and needs a disclosed app re-baseline, like Ruling CM's.
3. **(b) is the only variant that adds gully detail as you zoom.** It does so
   at x16 and x64 without touching generation. It needs a believability pass
   before wiring into `lod_bridge`: the current furrows read slightly combed.
   - Amplitude: `FURROW_DEPTH_PER_SPACING = 0.1`, a labelled judgement.
   - Seams: an owner-boundary flip between overlapping edifices is not
     smoothed.
4. **The root cause of the pits needs a design change, not a fill.** The
   gully envelope shallows faster than a concave cone falls on the lower flank
   (`t > 0.6`).
   - A gully whose depth is capped by the cone's own fall, so its floor
     descends monotonically to the toe, would remove the pits by construction.
   - With a kind-dependent spacing (shields dissected far less than
     stratocones, as Mauna Loa is), the dome's sunburst would go too.
   - This is new design and is not measured here.
5. **314159**: route to the comb row. There is no terrain-streak stage to fix.

## Owner questions (with defaults)

1. **Switch on (c-late), the final-field flank-pit fill, in the app?**
   Default: **yes**, with a disclosed app-world re-baseline. It is invisible
   in the frames and halves RV-1's small-lake count on edifice-heavy seeds.
2. **Retire variant (a) (kilometre spacing plus band limit) as built?**
   Default: **yes, retire it**. Replace it with a design row for
   kind-dependent gully spacing in km and a monotone (cone-capped) gully
   depth. Keep (a)'s band limit only as the generation half of (b).
3. **Take (b), tile-time radial furrows, to a believability pass and then
   wire it into the deep-zoom tiles?** Default: **yes, as its own row**:
   amplitude, irregularity, and the owner-boundary seam first. It is the only
   variant that meets the zoom requirement on volcano flanks.
4. **Close the 314159 streak row into the comb row?** Default: **yes**.
5. **Edifice radius floor at coarse scales** (Earth width: every edifice
   2 cells, about 39 km). Default: **file as its own row**; this pass does not
   touch it.

## What was built (all off at both boundaries)

| Piece | What it does |
|---|---|
| `cartalith_terrain::EdificeVariant` | Fields `gully_spacing_km`, `fill_pits`, `fill_pits_final`, `drop_gullies`, `drop_lumps`, `full_band`. Reached through `stamp_volcanoes_{provinces,simple}_variant`; the old `_traced` entries delegate with the default |
| `Edifice::new_with` / `add_at_band`, `pridged_weighted`, `fbm_weighted`, `edifice_resolved_weight` | Variant (a) and the ablations. At weight 1 the noise is the shipped noise bit for bit (tested) |
| `fill_edifice_flank_pits`, `FLANK_*` | Variants (c) and (c-late): 4-connected Priority-Flood+ε, calderas protected |
| `cartalith_terrain::edifice_furrows` | `EdificeFrame` (edifices from `volcanic_field`), `edifice_furrow_at`, `add_edifice_furrows_padded` (variant (b)), `flank_mask_from_volcanic_field` (c-late's mask) |
| `cartalith_engine::StreakSpikeParams::edifice` | The switch; (c-late) applied after the erosion passes |
| Harness additions | See "How it was measured" |

### Tests and mutants

- **Tests**: 12 terrain unit tests (7 for the variants: weighted noise,
  means, km spacing, band limit, flat-plane pits, two fill fixtures; 5 for
  `edifice_furrows`: frame recovery, the c-late mask, the band rule, the seam,
  radial orientation) and 1 engine test
  (every switch reaches generation; placement unchanged).
- **Calibration**: `ns_ratio` was added to `metric_calibration`.
- **Mutants: 13 run, all 13 killed** (after two tests were added). Two first
  survived:
  - the fill's `<=`, now killed by `fill_drains_a_flat_toward_its_outlet`;
  - the furrow noise's axis order, now killed by `furrows_close_across_the_seam`.

  A 14th, which disables the stamp-time mask, was not run because its literal
  occurs twice.

# Render target gap assessment — 2026-10-17

An **assessment**, not a scope and not a status record: it carries no status
column, and where it and `cartalith-native/docs/STATUS.md` disagree, `STATUS.md`
wins. It compares the shipped renderer against the owner's second target render
(`VISION.md`, "Second target: the photoreal map render", 2026-10-17;
`design/vision-render-2026-10-17/target_map_render.webp`). The owner's hard
requirement goes with it: **zoom has to gain detail at every level.** Each feature
is therefore judged at four zooms, not only at fit.

Verdicts are **MATCHES / PARTLY / GAP / CANNOT JUDGE**. Nothing is called a
match for being plausible. Every image named below is in
`cartalith-native/docs/evidence/render-gap-2026-10-17/`.

---

## 1. Method (what was run, and its limits)

- **Build.** I used the existing debug `cartalith-native/target/debug/cartalith_godot.dll`
  (built 2026-10-13 06:33), copied into a scratch tree made by `git archive HEAD`.
  After that time only three commits touch `crates/`. Two are annotation-only, and
  the third (`cf95556d`) is the export dialog. So the DLL carries every render
  change this assessment is about, including the lithology corner blend
  (`f272311a`, 2026-10-12) and the river receiver guard (`278877ec`, 2026-10-12).
  Nothing was built, and no Rust, GDScript or generated output was changed.
- **Harness.** A scratch probe (not committed) boots the real shell
  (`shell/app.tscn`) **windowed** in Godot 4.7.1. It generates through
  `EngineBridge.generate` at 2048 × 1311 with the app defaults: 800 km width
  (the New World dialog's default), the `sea_level` parameter (0.42), look
  **Natural Vibrant**, tier **quality**. It hides every overlay except rivers and
  clears the generated labels (`MapOverlay.set_labels([])`). It grabs the map
  viewport, which is 950 × 858 px because the docks take the rest of the window.
  The zoom steps reuse `_riverzoom_probe.gd`'s `_grab` / `_settle_lod` method:
  `reset_view`, `zoom_step`, `move_view_to`, then wait until `lod_pending() == 0`.
- **Zoom ladder.** Camera zoom **1** (the shell's opening view: `reset_view()` is a
  *cover* fit, measured at 141 %, 0.65 px per cell), then **×4** (1.9 px per cell),
  **×16** (7.4 px per cell) and **×64** (29.7 px per cell). Every step above 2.2 is
  the deep-zoom tiled-LOD path, and the probe confirmed `lod_active() == true` at
  ×4, ×16 and ×64.
- **Crops.** Each world has five targets. Mountain is the highest cell from a
  `sample_cell` scan at an 8-cell stride. River is the widest drawn trunk at 70 %
  of its length (`_riverzoom_probe.gd`'s rule). Coast, forest and ice were
  picked by eye from the 1-px-per-cell colour raster. **The ×1 crops are not
  centred:** at cover zoom, `move_view_to` cannot centre near an edge, so those
  frames include the void and the parchment frame.
- **Exports.** Each world also got a 4096-wide `WorldGen.export_raster_png`
  (4096 × 2622, about 1.6-2.0 s each) and an opening view with the look set to
  **Quality tier**.
- **Worlds.** The three used:
  - seed **24601**, `archipelago` archetype, 800 km;
  - seed **314159**, `earth`, 800 km;
  - seed **24601**, `earth`, **3000 km**, because the target reads as continental scale.

  I also generated and rejected two: 483920 classic at 800 km is almost all land
  around inland seas, and 483920 archipelago at 3000 km has almost no land. That
  makes five generations in all, at 11.7-17.6 s each.
- **Statistics** are on the luminance of each crop's central 70 %, which keeps out
  the HUD chips and the frame:
  - `grad`: mean neighbour gradient;
  - `p95`: its 95th percentile;
  - `hf`: standard deviation of the image minus a 2-px Gaussian blur, i.e.
    pixel-scale detail;
  - `subcell`: the share of luminance variance that a cell-resolution copy cannot
    reproduce. The copy is the crop box-downsampled by its px-per-cell, then
    upsampled bicubic. Near 0 means the picture is a magnified grid; it rises
    only if detail finer than a cell is really there.
- **The target** is 803 × 524 and is not a rendered world, so it has no zoom
  ladder. It is compared at whole-map scale only, against the three exports
  downsampled to the same size.

**What this cannot judge.** The target is one image at one scale. Saying what it
"would" show at ×64 is invention, so the per-zoom verdicts measure the shipped
renderer against the owner's *requirement* (detail keeps arriving) and against the
target's *qualities* (sharp hillshade, canopy texture and so on), not against
pixels the target does not have. One seed is one sample (`MISTAKES.md`): every
verdict below held on all three worlds unless it says otherwise.

## 2. Whole-map comparison (fit scale)

`01_target_vs_exports_whole_map.jpg` puts the target beside the three 4096 exports
at the same size.

| Statistic (803 × 524, central 70 %) | Target | 24601 archipelago | 314159 earth | 24601 earth 3000 km |
|---|---|---|---|---|
| mean gradient | **19.3** | 5.05 | 5.93 | 5.64 |
| p95 gradient | **59.8** | 17.2 | 21.4 | 20.8 |
| pixel-scale detail (`hf`) | **17.5** | 6.3 | 7.7 | 6.6 |
| luminance std (tonal range) | **42.3** | 26.1 | 24.9 | 23.2 |
| unique colours | 88 433 | 42 658 | 50 904 | 46 032 |

At the same scale the target has about **3.5× the edge energy, 2.5× the
pixel-scale detail and 1.7× the tonal range**. That is the measured form of what
the eye sees: the shipped map is soft, low in contrast and pastel.

## 3. Feature by feature, at each zoom

Zoom columns: **Fit** = opening view and 4096 export; **×4**; **×16**; **×64**.
The ladders are `02_…archipelago`, `03_…s314159_earth` and `04_…earth_3000km`.
In each, the rows are coast, forest, ice, mountain and river, and the columns
are ×1, ×4, ×16 and ×64.

| Feature (target) | Fit | ×4 | ×16 | ×64 | What I saw |
|---|---|---|---|---|---|
| **Sea depth ramp**: pale turquoise shallows to deep navy | PARTLY | PARTLY | GAP | GAP | A shallow-to-deep ramp exists (shelf, deep, abyss), but it runs pale teal to mid blue, never navy. Its large-scale pattern is cloud-like mottle crossed by **dark sinuous lines in open ocean** (01, all three worlds), which the target does not have. From ×16 the sea is a flat smooth wash (02 / 04 coast column). |
| **Shelf shading and shore halo**: a bright turquoise band hugging every coast | GAP | GAP | PARTLY | PARTLY | No coast-hugging shelf at fit: water beside a coast is often as dark as open sea (01, 11). From ×16 a thin pale fringe and a beach band appear at the waterline (`07`, `09`), but no wide turquoise halo. |
| **Forest canopy texture**: dark, granular tree canopy | GAP | GAP | GAP | GAP | Forest is a smooth flat green at every zoom. At ×64 it is a featureless gradient with a faint brushed grain (`05`: `hf` 0.64, 11 962 colours in a 950 × 858 crop). No tree-scale grain arrives at any level. |
| **Grass/desert blending** over wide distances, no hard edges | PARTLY | GAP | PARTLY | PARTLY | Wide cross-fades do exist, but at fit and ×4 the land carries an **axis-aligned square mottle** about 14 cells across (`12_square_mottle_grid_s314159.jpg`; visible in every ×4 frame of 02-04, strongest in tan/olive and snow/grass country). The target has nothing like it. At ×16 and ×64 the mottle blurs into soft blobs. |
| **Mountains**: strong directional hillshade, rock texture, grey crags, snow on ridges, spurs and foothills | GAP | GAP | GAP | GAP | The highest ground in all three worlds reads as a **soft white snow blob or a smeared light/dark band**, never as a lit ridge system (`06`, `09`, mountain rows of 02-04). There is no visible rock texture, no crag grey, and no spur and foothill structure. `10_relief_gain_diagnostic_s314159.jpg` doubles `relief_gain` (1.16 → 2.0) through `set_appearance`: the picture only **washes out**, and no ridges appear. |
| **Ice-shelf edge**: ragged, fractal, tundra below | PARTLY | PARTLY | GAP | GAP | The 800 km earth world has polar snow country and white ice tongues (01, 03 ice row). At ×4 the edge is blobby, not fractal. At ×16 and ×64 it is a featureless cream field (314159 ice ×64: **130 unique colours**, gradient 0.03, so practically flat). |
| **Rivers**: thin, dark blue branching lines following valleys | PARTLY | PARTLY | GAP | GAP | Networks branch and reach the sea. At fit they are thin and pale. From ×16 they are **wide, uniform, pale strokes**: about 8 px, about 1.5 km at 3000 km. Many tributaries run **straight and parallel**, side by side to the coast (`07`). At ×64 on the 3000 km world a straight diagonal trunk takes **straight perpendicular tributaries like a ladder** (`08`). Some strokes stop in open ground (`06`). |
| **Lakes in basins** | PARTLY | PARTLY | PARTLY | PARTLY | Lakes exist and sit in plausible places, with smooth shores (RV-4). At ×64, small lakes are soft rounded or rectangular blobs (`05`). The target's lakes are crisp dark-blue basins. |
| **Island scatter and coast shape**: two large continents, fractal coasts, archipelagos | PARTLY | PARTLY | PARTLY | CANNOT JUDGE | Island scatter exists, but the islands are mostly **long thin arcs** (01, 24601 archipelago). Coasts are smooth rather than fractal. Land is about 20-35 % of each map against the target's roughly 45 %. No world drawn has the target's two-big-continents layout. That is world shape, not rendering. |
| **No clutter**: no labels, borders, icons or frame | PARTLY | MATCHES | MATCHES | MATCHES | With the overlays off and labels cleared, the map is clean. Two things remain. The **parchment plate border and paper tone** are part of the shipped look: `border_width_frac` is 0.014 in the shipped table, and the frame shows in every export. The shell **opens with** territory, settlements, roads and province names showing. The target is a bare satellite-style base layer. |

**Natural Vibrant vs Quality tier** (`11_look_natural_vibrant_vs_quality_tier.jpg`):
the Quality-tier look is *less* saturated and *flatter*. Neither look closes any
of the gaps above, and none of the 15 shipped style presets was tried. Swapping
looks is not the lever.

## 4. Does detail keep arriving with zoom?

Medians over the 15 crops at each zoom (3 worlds × 5 targets):

| Zoom | px / cell | unique colours | mean gradient | p95 gradient | pixel detail `hf` | sub-cell variance share |
|---|---|---|---|---|---|---|
| ×1 (opening) | 0.65 | 62 647 | 5.60 | 19.6 | 11.9 | — |
| ×4 (tiles) | 1.86 | 42 335 | 2.48 | 8.7 | 4.2 | **0.4 %** |
| ×16 (tiles) | 7.42 | 25 478 | 1.10 | 3.6 | 2.2 | **1.1 %** |
| ×64 (tiles) | 29.7 | 4 585 | 0.77 | 2.1 | 1.4 | **5.4 %** |

Every per-pixel detail measure falls at every step.

The sub-cell share does rise at ×64, but the river strokes cause that, not the
terrain. The crops with a river through them read 0.16-0.25: 314159 river 0.161,
3000 km river 0.251, 3000 km forest 0.214. The riverless ice and forest crops
mostly read **0.002-0.02**. Only the archipelago mountain crop sits at 0.10, and
it lies at a volcano rim with a river crossing it. So **about 98-99 % of what is
on screen at ×16 and ×64 is the cell-resolution field magnified.** Vector rivers
are the one thing that stays sharp at deep zoom.

**Where it stops.** The terrain is effectively finished by about ×4. From ×16 it
goes **soft and blurry, not blocky**: no stair-stepping was seen, because the
smooth shores (RV-4) and the tile morph (LOD-D3) work. Below that, nothing new
arrives: no rock, no canopy, no crags, no tributaries finer than the drawn
network. That fails the owner's hard requirement.

It matches what the project already recorded:
- **LOD-D2** closed with the zoom-40 detail bar on the 2048 world *not met*;
- **LOD-D5** closed with "detail per pixel non-decreasing from zoom 4 to 40"
  *not met*. Its fix needs `cartalith_terrain::amplify::add_zoom_detail`'s octave
  decay changed. `amplify.rs`'s own doc records the frequency as **per coarse
  cell, not per kilometre**, and calls that change an owner ruling and a golden
  re-baseline;
- **LOD-D4** has 1 of 4 bars met.

`STATUS.md`'s LOD rows give those counts; I did not re-run their harnesses.

**After (2026-10-17, Ruling CI item 1):** the kilometre-keyed zoom detail re-measured
this ladder on four worlds through the shell (`DECISIONS.md` §7p's 2026-10-17 entry;
frames in `cartalith-native/docs/evidence/zoom-detail-2026-10-17/`). Median gradient
×4 → ×64 now 1.58 → 2.89 (1.58 → 0.41 on the same crops before); sub-cell share at
×64 9.45 %. The numbers above are left as they were measured.

## 5. Root causes, as far as I verified them

| Gap | Verified cause | Evidence quality |
|---|---|---|
| No coast-hugging shelf or halo | `render.rs::sea_color_core` takes `depth` from the smoothed bathymetry alone (`cell_color_river`: `depth = (sea_level − sea_h) / sea_level`), and its only shore term is `surf = smoothstep(0.03, 0.0, depth)`. **No distance-to-coast term reaches the sea colour.** `coast_distance` exists, but its only caller is the `npr.waves` branch. `apply_coast_sdf` paints the *land* side only (beach and plain bands, `din ≥ 0`). | **Verified at the symbols** |
| Sea too pale; dark lines in open ocean | The ramp's endpoints are the appearance's `w_shelf` / `w_deep` / `w_abyss` palettes, and the lines follow the bathymetry in `sea_h`. **Which generator stage makes those lines (plate boundaries are the guess) is not isolated.** | Ramp verified; line source a hypothesis |
| Square land mottle | `render.rs::bio_jitter` adds `vnoise` at 44 and **150 cycles per map width**, which is a lattice about 13.7 cells across on a 2048 grid. It shifts temperature by up to ±3.5 °C and moisture into `material_weights`. `cartalith_noise::vnoise` is lattice value noise, and value noise shows axis-aligned square cells; the temperature thresholds then sharpen them. **The lattice size matches what is seen. No on/off test was run.** | **Likely; not isolated** |
| Mountains without ridges | Doubling `relief_gain` washes out instead of revealing ridges (`10`), so **the height field at these sites has no ridge and valley structure for the shading to light**. Shade strength is not the limit. The 800 km highest cells are a snow plateau and a volcano rim. The geology-first construction (`relief_model`, GF-11/GF-13) is off by default and was not tried. | One world, one crop: **suggestive, not established** |
| Detail stops at about ×4 | `add_zoom_detail`'s frequency is per coarse cell, and its octave count is keyed to the pyramid level (`amplify.rs` doc, LOD-D5) | Recorded in code; consistent with the §4 numbers |
| Forest has no canopy texture | Forest is `forest_col` mixed by `material_weights`. Milestone 4's forest stipple (`stipple_strength`) did not show at any zoom in Natural Vibrant. **Why was not checked** (strength, scale or look). | Observation only |
| Wide, pale, straight or parallel rivers | The stroke colours and widths come from the river layer (`river_style_color`, `RIVER_ORDER_RGB`, RIM-*). The **straight, parallel and ladder-like courses** look like D8 flow on near-flat ground, most visible at 3000 km (1.46 km per cell). **Not traced.** | Observation; cause a hypothesis |
| Frame and paper | `apply_border` / `paper_tone` (milestone 4 "atlas look"), on in the shipped table | Verified that the field is set |

## 6. Prioritised gap list

Impact is judged against the target and the zoom requirement. Effort is a rough
class. "Row" names an existing `OUTSTANDING_WORK.md` row or scope; **no row yet**
means nothing routes it today.

| # | Gap | Impact | Effort | Row |
|---|---|---|---|---|
| 1 | **Detail stops arriving past about ×4** (terrain is the magnified grid at ×16/×64) — the owner's hard requirement | very high | large; needs a golden re-baseline ruling | `LOD_DETAIL_SCOPE.md` LOD-D5's unmet bar (`add_zoom_detail` per-cell frequency and octave decay) and `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` EF-0…EF-9. **No open row:** LOD-D5 closed with the bar disclosed as unmet |
| 2 | **Mountains: no ridged relief, no rock or crag, snow as blobs** | very high | large (terrain), medium (render) | `GEOLOGY_FIRST_SCOPE.md` (`relief_model`, off by default; not measured here), EF-3 (closed); **no row** for rock texture and crag colour as rendering |
| 3 | **No coast-hugging shelf or shore halo; sea too pale; ocean lines** | high | **small to medium**: a coast-distance term into `sea_color_core` plus a darker deep ramp | **No row yet** (`MAP_STYLE_RESEARCH.md` §2.11 notes NatGeo's bathymetric shading; Ruling BI's Nautical ramp is a palette, not a shelf) |
| 4 | **Square value-noise mottle on land** | high (visible at fit and ×4 everywhere) | small to try (an on/off measurement first) | **No row yet**. Related, but not the same symptom: the open "diamond-lattice pattern in deep-zoom tiles" row in §2.5 |
| 5 | **Forest canopy texture** (none at any zoom) | high | medium | `TERRAIN_APPEARANCE_SCOPE.md` milestone 4 (forest stipple), LOD-D5; **no row** for canopy detail under zoom |
| 6 | **Rivers wide, pale, and straight or parallel at deep zoom** | medium to high | medium | `RIVERS_IN_MAP_SCOPE.md` RIM-* (painted rivers), RV-*; **no row** for the straight/parallel/ladder courses |
| 7 | **Ice edge not fractal; flat at deep zoom** | medium | medium | `LOD_DETAIL_SCOPE.md` LOD-D4 (1 of 4 bars) |
| 8 | **Overall tonal range and saturation** (1.7× tonal range, 3.5× edge energy in the target) | medium | small: a "satellite" look preset | `MAP_STYLE_RESEARCH.md` §4 presets (no satellite or photoreal one); **no row yet** |
| 9 | **Frame, paper and default overlays** against a clean base layer | low to medium | small: a look with `border_width_frac = 0`, paper off | **No row yet** |
| 10 | **World shape** (arc islands, low land fraction, no two-continent layout) | medium | medium: generation, not rendering | **No row yet**. This is the World Structure archetypes, not the renderer |

## 7. Owner questions, each with a recommended default

1. **Does the target replace or sit beside the atlas look?** *Recommended:* add it
   as a new named look ("Satellite relief"), with no frame or paper, a navy deep
   ramp and a turquoise shelf. Keep Natural Vibrant as the default until that
   look passes its own zoom ladder.
2. **May `add_zoom_detail`'s frequency be keyed to kilometres, and its octave decay
   changed, as a golden re-baseline?** That is LOD-D5's recorded open question, and
   it is the main lever for the hard requirement. *Recommended:* yes, behind the
   app-side `params::defaults()` divergence pattern (`MISTAKES.md`, "Change
   generated output"), so the JS-parity goldens stay put.
3. **A coast-distance shelf term in the sea colour?** It is new behaviour with no
   reference ancestor. *Recommended:* yes, as an appearance tunable that is 0 in
   `js_reference()` and on in the new look.
4. **Measure the square mottle (`bio_jitter`) off and on before deciding.**
   *Recommended:* measure first. If it is the cause, replace the value-noise
   lattice with a rotated or warped one (the pattern `sea_grain_warp` already
   follows) in the new look only.
5. **Should the shell open with overlays and province names on?** *Recommended:*
   leave the default; let the new look's preset turn them off.
6. **Is world shape (two big continents with archipelagos) in scope for "the
   render"?** *Recommended:* no. Treat it as a World Structure archetype question
   and keep this list to rendering.

## 8. Evidence index

| File | Shows |
|---|---|
| `01_target_vs_exports_whole_map.jpg` | the target beside the three 4096 exports, same size |
| `02_zoom_ladder_s24601_archipelago_800km.jpg` | coast, forest, ice, mountain and river at ×1 / ×4 / ×16 / ×64 |
| `03_zoom_ladder_s314159_earth_800km.jpg` | the same ladder, seed 314159 earth |
| `04_zoom_ladder_s24601_earth_3000km.jpg` | the same ladder, 3000 km world |
| `05_forest_x64_s24601_archipelago.jpg` | forest at ×64: smooth, no canopy |
| `06_mountain_x16_s314159_earth.jpg` | the highest ground at ×16: a snow blob, no relief |
| `07_rivers_x16_s24601_earth3000.jpg` | wide pale strokes, parallel straight tributaries |
| `08_river_x64_s24601_earth3000.jpg` | a straight trunk with perpendicular "ladder" tributaries |
| `09_export4096_1to1_crop_s314159.jpg` | a 1:1 crop of a 4096 export |
| `10_relief_gain_diagnostic_s314159.jpg` | `export_snapshot_png` at the mountain: shipped (left) vs `relief_gain` 2.0 (right) |
| `11_look_natural_vibrant_vs_quality_tier.jpg` | the opening view in both looks |
| `12_square_mottle_grid_s314159.jpg` | the 1-px-per-cell raster at 4× nearest, showing the square mottle |

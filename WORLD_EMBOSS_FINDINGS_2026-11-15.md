# World emboss findings (2026-11-15)

Findings only. No product code changed. This file claims no status; `cartalith-native/docs/STATUS.md` is the
source of truth for progress, and the routed row belongs in `OUTSTANDING_WORK.md`.

**The owner's report.** On a whole world, all land carries a uniform mottle at low zoom (`1.webp`). At about
760 % a crinkled, embossed bump texture covers every biome with equal strength (`2.webp`). A region instead
shows relief that follows the terrain (`3.webp`, `4.webp`). The owner's follow-up: *"The smaller the scale the
more refined this embossing is working with the underlaying terrain (eg 40k km becomes more uniform and stays
it no matter the zoom and 200km shows a lot of detail and difference per terrain type)"*. Also: *"Something
that, with the LOD scaling mechanism shouldn't make a difference."*

**Short answer.** World against region is not the variable. The cause is the map's **kilometres per base
cell**, through three terms:

1. **The deep-zoom tiles' kilometre zoom-detail law** (`cartalith_terrain::amplify::KmZoomDetail`,
   `km_zoom_detail_sum`, `km_relief_taper`; Ruling CI). It accounts for the embossing in `2.webp`. Its
   amplitude grows linearly with km per cell, and its terrain mask fades out as km per cell grows.
2. **The LOD pyramid's depth cap** (`lod_bridge::MAX_LEVEL` and `level_for_zoom`). It is counted in base
   cells, so on a coarse-celled map detail stops arriving at a ground scale set by the base grid. This is
   why the texture "stays it no matter the zoom" on a 40 075 km map.
3. **The land colour jitter** (`render::land_grain`, which feeds `tt` and the biome jitter in `land_color`).
   It accounts for the low-zoom camouflage in `1.webp`. Its lattice is per base cell, so on a world each
   blotch is hundreds of kilometres of ground, unrelated to terrain.

Each term was isolated on and off in the running shell (frames below). None of them is generation. All three
are presentation-side (§3).

---

## 1. Every layer that adds fine relief or texture to the 2D map

Each layer was read at its symbol in `origin/main` `92a63f48`. "Scale" means how its spatial frequency is
set; "amp" means how its strength is set.

| Layer (symbol) | Where it runs | Scale | Amp | Modulated by | Changes with width or world mode? |
|---|---|---|---|---|---|
| Hillshade, macro/meso/micro bands (`land_color` `sh`, `sh_m`; `flat_land_light`; `tile_shade_exag`) | base raster and tiles | per base cell (tiles: `exag · px per cell`, so the gradient is read per coarse cell) | `exag`, the band weights | true grid gradient | No direct km term. Heights are normalised, and `peak_m` fixes the metres per unit, so the same normalised map has the same per-cell gradients at every width (§2.3) |
| AO, SVF, sun shadow (`build_ao`, `build_svf`, `build_sun_shadow`, `fold_lighting_fields`) | grid precompute | per cell (`ao_radius_frac` × grid) | tunables | grid relief | `world` only switches wrap. No km term |
| Local contrast (`apply_local_contrast`, `TileFields::contrast_d`) | grid raster, tiles sample it | per cell radius | `local_contrast` | the finished raster | no km term |
| **Land colour jitter** (`land_grain` → `n_low`, `n_hi`, `n_bio`; `bio_jitter`) | every land pixel, base and tiles | **per base cell**: `vnoise(x·0.06)` (≈16.7 cells), `96/gw`, `44/gw`, `150/gw` | fixed (`tt = 0.5 + 1.1·(n_low−0.5) + 0.5·(n_hi−0.5)`; biome temperature jitter ±3.5) | nothing (`eco_k` only) | **Yes, by construction.** One blotch is ≈16.7 cells: 3 km on a 200 km/1024 map, **327 km on a 40 075 km/2048 world** |
| Forest canopy (`canopy_relief`, `apply_canopy`) | where the look enables it | **km** (`CANOPY_BASE_KM` 4 km down to 8 m, Nyquist-faded per output pixel) | `canopy_texture` × forest weight | forest fraction | km-keyed. Correct at any width |
| Rock microtexture (`litho_microtexture`, `geo_micro`) | rock | per pixel/cell | `geo_micro` × exposure | slope, lithology | not km |
| LOD-D5 scale-aware bands (`DetailScale`, `detail_scale_u`, `micro_n_from_residual`) | tiles | cells per pixel | `detail_scale_strength` | the tile residual | inherits the residual's amplitude (below) |
| **Micro-relief tone** (`micro_relief_tone`, `tile_micro_tone`) | tiles under the km law | tile Laplacian ÷ cells per px | `tanh(·/MICRO_TONE_REF 0.04)` | the residual's curvature | **inherits the km law's amplitude**, and saturates when the residual is large |
| **Cryosphere lapse on tiles** (`TileCryo::delta_t`) | tiles | per tile pixel | `lapse · g · residual · meters_per_unit` (metres = `peak_m/(1−sea)`) | the residual | **inherits the km law's amplitude**, which turns noise peaks into snow (the white flecks in `2.webp`) |
| **Km zoom-detail law** (`KmZoomDetail`, `km_zoom_detail_sum`, `km_relief_taper`; wired by `render::tile_amplify_opts` / `zoom_detail_km_law`; `zoom_detail_km = 2.0` shipped) | deep-zoom tiles and info tiles; `sample_elevation` (EF-0) shares the sum | **km lattice**: `λ_j = 0.195 km · 2^−j`, octaves inside `(λ_min/2, L)` | **`a_j = 0.14·0.6·2 · 0.5^j`, so amplitude ∝ λ in km, in normalised height** | taper `max(0.05, slope·1.5625/L·8, 0.3·land)` × shore fade | **Yes: amplitude per base cell ∝ L, and the slope mask ∝ 1/L.** The culprit (§2) |
| Reference zoom schedule (`add_zoom_detail` with `km: None`, `zoom_detail_km = 0`) | tiles, when the law is off | per cell, `min(6, z − z_base)` octaves | `0.6^o` per cell | `min(1, 8·|∇h|)` per cell | not km. Its octave count is capped per level |
| **LOD depth** (`lod_bridge::MAX_LEVEL = 10`, `level_for_zoom(px_per_cell, gw)`, `z_base()` = 4) | tile pyramid | **px per base cell** | — | — | **Yes**: the finest level is 256 px per base cell whatever the cell's size |
| Volcano furrows, lava, edifice (`add_tile_furrows` etc.) | tiles on edifices | km / edifice frame | tunables | edifice | local only |
| Paper grain/mottle (`paper_tone`) | sheet scale | sheet | `paper_*` | — | not land relief |

What world mode itself changes: `world` switches wrap in the blurs, AO and sea smoothing, and the
equirectangular frame. None of those is a detail or texture amplitude. The width sweep below confirms that
region mode at 40 075 km shows the same emboss as world mode.

## 2. Mechanism

### 2.1 The km law's amplitude grows with km per cell

`km_zoom_detail_sum` puts octave `j` at wavelength `λ_j = λ_ref·2^−j` (with `λ_ref = 0.195 km`) and amplitude
`a0·0.5^j` (with `a0 = detail_amp·0.6·zoom_detail_k·strength = 0.168`). So `a_j = a0·λ_j/λ_ref`: amplitude
proportional to ground wavelength, in **normalised height units**. That unit is a fixed number of metres
(`peak_m / (1 − sea)` = 4000/0.58 ≈ 6.9 km per unit at the default `peak_m`) **whatever the map's width**.

The top octave the law draws sits just under one base cell (`λ ∈ [L/2, L)`, where `L` is km per cell). Its
amplitude is therefore about `a0 · L/λ_ref`. Each octave's slope per base cell is
`a_j / (λ_j/L) = a0 · L / λ_ref`, which **grows linearly with L**. Computed from the law (`j_lo` is the first
octave, peak amplitude is `a0·2^−j_lo·0.5`):

| map | km/cell `L` | top octave λ | peak amp (units) | peak amp (m, before taper) | synthetic slope per cell | slope-mask gain `8·1.5625/L` |
|---|---|---|---|---|---|---|
| 200 km / 1024 | 0.195 | 0.098 km | 0.042 | ≈ 290 m | 0.17 | 64 |
| 800 km / 2048 (**Ruling CI's anchor**) | 0.391 | 0.195 km | 0.084 | ≈ 580 m | 0.34 | 32 |
| 1200 km / 512 | 2.34 | 1.56 km | 0.67 | ≈ 4.6 km | 2.0 | 5.3 |
| 5000 km / 1024 | 4.88 | 3.1 km | 1.34 | ≈ 9.3 km | 4.2 | 2.6 |
| 40 075 km / 2048 (owner's world) | 19.6 | 12.5 km | 5.4 | ≈ 37 km | 16.8 | 0.64 |
| 40 075 km / 1024 | 39.1 | 25 km | 10.8 | ≈ 74 km | 33.7 | 0.32 |

The coarse terrain's own gradient per cell does **not** grow with width. The pipeline lays out the same
geography in cells at every width; the fit frames of seed 1200 at 200, 1200 and 5000 km are the same coastline
(`sheet_equivalence_native_vs_world_zoomed.png`, left column). The median land slope per cell on the shipped
grids is of order 10⁻³ (`MOUNTAIN_RELIEF_INVESTIGATION_2026-10-17.md` §2.2, cited, not re-measured here). So
the synthetic slope against the real slope is about 100× at the anchor, and about 10⁴× on a 40 075 km world.
The light reads the gradient per coarse cell (`tile_shade_exag`), so on a world the noise is all the light
sees.

### 2.2 The terrain mask fades out as km per cell grows

`km_relief_taper = max(0.05, min(1, 8·|∇h|·1.5625/L), 0.3·land) · shore`. The slope term is multiplied by
`1.5625/L`: 4 at the anchor, 0.08 at 19.6 km/cell. On a world it never fires. The taper collapses to
`max(0.05, 0.3·(h−sea)/(1−sea))`: a floor plus a pure elevation term. Neither knows slope, drainage or biome.
That is the owner's *"applied to all terrain above sea level … types of biome and terrain look the same"*. On
a 200 km map the slope term saturates on almost any relief, so the detail follows the terrain.

The two clamps make the shape worse, not better. `clamp_toward_sea` halves any excursion toward sea level, and
the top clip flattens tops, so ±kilometre noise turns into flat-topped blobs. `TileCryo::delta_t` then cools
every noise peak by the lapse rate. The result is the white-capped crinkle of `2.webp` (`world_759_detail_on.png`).

### 2.3 The LOD depth is counted in base cells, so zoom stops adding detail on a coarse map

`level_for_zoom(px_per_cell, gw)` stops at `MAX_LEVEL = 10`, which is 256 px per **base cell**. The law's
finest full-weight octave at level `z` is `λ_min = L · 2^(z_base−z) · 2` with `z_base = 4`. At the deepest level
that is `L/32`:

- **6 m** on a 200 km/1024 map;
- **1.2 km** on a 40 075 km/1024 world.

Past that, the tile texture is only magnified. So on a world, at the 0.05 km/px view, the finest detail is
24 px across, and zooming further adds nothing. That is the owner's *"stays it no matter the zoom"*. With the
law off, the reference schedule has the same cap (`min(6, z − z_base)` octaves per level, levels capped at 10).
On the world the "off" frames go almost flat at depth: land gradient 0.16 at 0.05 km/px.

### 2.4 The zoom-0 mottle is the per-cell land colour jitter

`land_grain` samples value noise at 0.06 cycles per cell, 96/gw, 44/gw and 150/gw. Those frequencies are per
base cell. They drive `tt` (where each material sits on its three-stop ramp) and the biome temperature jitter.
On the owner's 2048-wide world one `n_low` blotch is ≈ 16.7 cells ≈ 327 km of ground. Seen at the zoom of
`1.webp`, those blotches are the 20–40 px camouflage spots. In pixels they are the same size at every width,
but on a region they are a few km of ground and sit under strong hillshade; on a world they are continent-scale
noise over weak per-pixel relief. The km law is **not** involved at that zoom: on and off at 250 % differ by
0.46 luma.

### 2.5 Why "the LOD should make it not matter" does not hold today

The owner's expectation is that a 40 075 km world zoomed until the screen shows ~200–1200 km should look like a
200–1200 km map at fit. Every term above ties detail to the base cell rather than to the viewed ground scale:

- the law's amplitude (∝ `L`);
- its mask (∝ `1/L`);
- the LOD depth (px per base cell);
- the colour jitter (per cell).

The side-by-side at equal km per pixel (`sheet_equivalence_native_vs_world_zoomed.png`) shows it. The native
1200 km map at fit (0.471 km/px) has ridges, valleys and snow on the range. The 40 075 km world zoomed to
0.471 km/px shows flat-topped noise blobs with the law on, and a nearly flat, snow-covered field with it off.

A full equivalence, with real dendritic valleys and tributaries at the viewed scale, is out of reach for any
noise law. The native map's landforms come from the generation pipeline (orogen, erosion, hydrology) at its own
cell size. What the renderer can and should guarantee is **statistical** equivalence: sub-cell relief bound to
the terrain and the biome, at a strength that does not depend on base km per cell, arriving at every zoom.
Real landforms at the viewed scale are the EF line's job (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` EF-0…EF-9:
local refinement, local hydrology re-accumulation).

## 3. Proof: renders

**Harness.** Windowed Godot 4.7.2 at 2560×1440, scratch `APPDATA`, the owner's debug DLL (built 20:43, no
render change since), seed 1200. Probe `_emboss_probe.gd`: a scratch copy of `_wrcap_probe.gd`, not committed.
`zoom_detail_km` was set with `set_appearance` before each generate: `2.0` is "on" and shipped, `0.0` is "off",
the reference schedule. The `land_grain` switch was a temporary env-gated early return in a local build
(`CARGO_TARGET_DIR=C:/cw/emboss_target`), reverted and never committed. Each pair compares two renders of the
same view.

Measures, in `metrics.txt`, over the map area only and on land blocks:

- `added`: mean |Δluma| on/off.
- `grad_on`/`grad_off`: mean luma gradient.
- `bind_r`: Pearson r, across 32 px blocks, between the std of the added detail and the grid's own relief
  (the off frame's gradient). Near 0 means unbound to the terrain.

Width sweep, grid 1024×512, region mode, at ×8 from fit, on the same pivot:

| width km | km/cell | added (on−off) | grad on / off | bind_r |
|---|---|---|---|---|
| 200 | 0.195 | 7.9 | 6.81 / 6.12 | 0.14 |
| 1200 | 1.17 | 19.7 | 9.65 / 6.04 | 0.39 |
| 5000 | 4.88 | 38.8 | 14.77 / 4.73 | 0.62 |
| 15 000 | 14.6 | 43.1 | 19.63 / 3.16 | 0.39 |
| 40 075 | 39.1 | 39.1 | 17.82 / 3.06 | 0.13 |
| 40 075 world mode | 39.1 | 60.1 | 17.45 / 1.99 | −0.03 |

At 0.05 km/px: 40 075 world `added` 96.3, `bind_r` 0.11. The 1200 km/512 region at 900 %: `added` 31.7,
`bind_r` 0.30. The added detail grows about 5–8× from 200 km to 40 075 km. At the world end it becomes
unrelated to the terrain (`bind_r` ≈ 0) and uniform (cv 0.11–0.19). Read in the frames: from 5000 km up, the
"on" frames are a maze of embossed blobs with white caps; the "off" frames are smooth (`sheet_width_sweep_x8_on_off.png`).

Owner-matched frames, world 40 075 km at 2048×1024 and region 1200 km at 512×384:

- `world_759_detail_on.png` / `_off.png`: reproduces `2.webp` (fine crinkle with snow flecks over plains,
  forest and snow alike). Off: smooth. `added` 32.7, `bind_r` 0.12.
- `world_250_detail_on.png` / `_off.png`: the `1.webp` zoom. On and off are the same (0.46 luma): the law is
  not the mottle.
- `world_250_land_grain_on.png` / `_off.png`: the mottle is `land_grain`. Off, the desert and plains lose the
  blotches. `added` 9–11 luma at every zoom, `bind_r` ≈ 0.
- `region_900_detail_on.png` / `_off.png` and `region_117_*`. At this 1200 km/512 region (2.34 km/cell, 6×
  the anchor) the law already blobs the snowy range at 900 %. The owner's terrain-bound `4.webp` is consistent
  with a smaller km per cell.
- `equiv_*`: a native map at fit against the world zoomed to the same km/px, law on and off (§2.5).

**Harness caveats.** The land-picking pivot fell back to screen point (256, 240) in every run. That point is
under the left dock, so every zoom pivots there; on and off share it, so each pair is valid. The log carries
`draw_layout` and Cinzel font-loader script errors from the copied import cache. They concern urban layouts
and title fonts, not the terrain raster.

## 4. Proposed fix (not built)

All of it is **render-only**: tile synthesis, tile colour, and the LOD level choice. It does not change
generation, world data, saves or any JS golden. It is an app-side re-baseline of Ruling CI's tile law under
`DECISIONS.md` §7p, and it needs an owner ruling, as Ruling CI did. Tile and look fingerprints move;
`tests/zoom_detail_ladder.rs`, the `lod_bridge` octave-ratio tests and EF-0's `sample_elevation` (it shares
`km_zoom_detail_sum`, so displayed elevations at deep zoom move) re-baseline.

**F1. Relief-relative amplitude** (`KmZoomDetail`, `km_zoom_detail_sum`, `km_relief_taper`). Keep the km
lattice, which keeps tiles seam-exact and the frequencies tied to ground distance. Replace the amplitude
`a0·0.5^j` with the grid's own local relief continued below the cell:

```text
a_j(x) = κ · ρ(biome, lithology, x) · R(x) · (λ_j / L)^H        H = 1 (each octave adds the same slope)
R(x)   = local per-cell relief of the coarse field (e.g. the RMS of h − lowpass₃ₓ₃(h), or |∇h| per cell),
         in normalised units, with a small floor only on non-flat biomes
```

- Every octave's slope per base cell becomes `κ·ρ·R`. That is proportional to the terrain's real per-cell
  slope and **independent of km per cell**, so a world and a region with the same normalised terrain get the
  same relative roughness.
- `R → 0` on flat land, so plains and floodplains stay smooth.
- `ρ` is the biome and rock roughness. Labelled judgements, to be set on the ladder: rock/alpine 1.0, forest
  and scrub 0.5, grassland 0.35, desert sand 0.25, wetland/floodplain/ice 0.15–0.3, lithology `geo_micro`
  terms where present.
- Drop the `1.5625/L` slope term and the `0.3·land` elevation term; `R` replaces both. Keep the shore fade.
- Cap the excursion in metres: `|Σ a_j| ≤ c·R` (`c` ≈ 0.5) and ≤ `peak_m/10`. This keeps `TileCryo` and
  `micro_relief_tone` in range, with no snow flecks and no saturated tone.
- Set `κ` so that at the Ruling CI anchor (800 km/2048) the shipped ladder's numbers are kept. The look was
  tuned there, and the owner likes the region.
- Optional follow-up, already a named row (the mountain investigation's S1): ridged octaves on orogens and
  valley-aligned octaves from `flow`. These bring the synthesis closer to "a natively finer map" than isotropic
  noise can.

**F2. LOD depth in ground units** (`lod_bridge::MAX_LEVEL`, `level_for_zoom`). Allow levels until one texel is
at most a ground floor, rather than until 256 px per base cell:
`max_level = z_base + ceil(log2(L / λ_floor))`, with `λ_floor` ≈ 10–20 m (labelled), and capped by a memory
and time budget. Then a 40 075 km world zoomed to 0.05 km/px reaches octaves down to the screen's Nyquist
limit, as a 200 km map does. The law's `max_octaves` (8) and `z_base` stay. Measure tile cost and the pyramid's
residency at the new depth before shipping (`MEMORY_OPTIMIZATION_SCOPE.md` discipline).

**F3. Land colour jitter in km** (`land_grain`, `bio_jitter`). Key its four lattices to km with a
cell-relative ceiling, `λ = min(cells·L, λ_max_km)`, so a world's blotch is tens of km, not hundreds. Fade
`n_low`/`n_hi`'s `tt` contribution where the grid relief is low and the zoom is far out, so the base map shows
terrain rather than camouflage. Keep the reference branch (`land_grain_warp <= 0`) byte-identical, as today.

### Bars (seed 1200, plus at least two more seeds; the windowed harness above, frames looked at)

- **B1 width invariance.** At 1024×512, widths 200 / 1200 / 5000 / 15 000 / 40 075 km, at ×8 and ×32: `added`
  on land within 0.7–1.3× of the 1200 km value. Today 7.9 → 60.1 at ×8, 7.6×.
- **B2 terrain binding.** `bind_r` ≥ 0.4 at every width and zoom in the sweep. Today 0.13 and −0.03 at
  40 075 km.
- **B3 flat land stays smooth.** On the flattest decile of land blocks, `added` ≤ 0.25× the steepest decile's.
- **B4 equivalence at equal km/px.** A 40 075 km world zoomed to 0.471 and 0.078 km/px against the 1200 km
  and 200 km maps at fit: land luma-gradient energy within ±30 %, and `bind_r` within 0.15 of the native
  map's. Today the world reads 12.3 / 4.8 with blobs, or 1.65 / 0.41 with the law off.
- **B5 zoom gains detail at every level** (hard requirement). On the 40 075 km world, the median land
  gradient is non-decreasing from ×4 to the 0.05 km/px view and one step beyond. The ratio of the finest drawn
  octave to the screen pixel stays within 2–4 px at every zoom (F2).
- **B6 the anchor is preserved.** At 800 km/2048, the Ruling CI ladder (16 crops, four worlds) stays within
  ±10 % of its shipped x4 → x64 median (1.58 → 2.89).
- **B7 no noise snow.** Land pixels whose grid temperature is above the snowline: snow fraction on/off
  differs by ≤ 2 percentage points at 760 %. Today the flecks are visible across the plains.
- **B8 mottle.** At the world's 250 % view, `land_grain` on/off `added` ≤ 4 luma, and blotch wavelength ≤
  the chosen `λ_max_km`. Today 10.6 luma at ≈ 327 km.

## 5. Evidence

The evidence is in `cartalith-native/docs/evidence/world-emboss-2026-11-15/`:

- `metrics.txt`: every frame pair.
- `sheet_width_sweep_x8_on_off.png`: 200 → 40 075 km at ×8, on | off.
- `sheet_equivalence_native_vs_world_zoomed.png`: a native map at fit | the world at the same km/px, on | off.
- `world_759_detail_on.png`, `world_759_detail_off.png`.
- `world_250_detail_on.png`, `world_250_detail_off.png`.
- `world_250_land_grain_on.png`, `world_250_land_grain_off.png`.
- `region_900_detail_on.png`, `region_900_detail_off.png`, `region_117_detail_on.png`, `region_117_detail_off.png`.
- `sweep_w200km_*`, `sweep_w40075km_*`, `sweep_world40075km_*` (×8 on/off).
- `equiv_*`.

Every image is a 1000×640 crop of the map area, centred on the zoom pivot, except the sheets and the
`land_grain` pair.

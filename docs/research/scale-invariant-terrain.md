# Real-km-aware relief and rivers — why a smaller region never showed more detail

*v1.60. Owner report: "when choosing a smaller region rivers dont become more visible (i think its
a scaling issue). I cant seem to find any rivers with the branching pattern or length you might
expect... check any and all information on river formation and how simulations/terrain programs
realistically generate them and apply them with proper scaling." Root-caused by direct measurement
(not guesswork) before any code changed — see §4.*

## 1. Diagnosis

Cartalith Gen1 lets a user generate either a **World** (whole-planet, 2:1 aspect, wraps
horizontally) or a **Region** (a chosen real-world extent, `state.mapWidthKm`, 1.56:1 aspect) at a
chosen grid resolution `GW` (`state.resW`; `GH=gridH(GW)`). `GW` and `mapWidthKm` are two fully
independent user-set values — nothing in the UI ties them together.

**Finding 1 — the relief pipeline never reads `mapWidthKm`.** `buildTectonicSubstrate`'s warp
(`fillWarpRows`), crustal heterogeneity (`fillHeteroRows`) and the height formula itself
(`fillHeightRows`) all sample fractal noise at a frequency defined as a fixed fraction of grid
width — e.g. the height formula's own noise term samples `fbm(x*nf/W, y*nf/W, seed)` with `nf=5.0`
a hardcoded constant, meaning **exactly 5 wavelengths of primary noise always span the grid,
regardless of resolution or real km extent**. A controlled A/B (same seed, same `GW`, only
`mapWidthKm` swept: 40000/800/200/50 km) confirmed this directly: with volcanism/craters disabled,
`field`/`temp`/`rain`/`flow` FNV hashes were **bit-identical** across every `mapWidthKm` value. A
50 km region and a 40,000 km world at the same resolution produced statistically identical relief —
a smaller region never got finer local drainage detail, only the same coarse pattern re-labelled
with smaller km values.

**Finding 2 — craters/volcanoes are the one place real km *is* used, and it's unclamped.**
`placeSizedVolcano`/`stampCraters` draw a real-km radius (`radKm`) then divide by
`cellKm=mapWidthKm/GW` to get a cell radius, with only floor clamps (`Math.max(2,...)`,
`Math.max(1.5,...)`). Measured at a fixed seed: the largest crater's radius reached **716 cells
against a `GW` of only 256 — 2.8× the entire grid width** at a 50 km region, while at world scale
all 100 craters + 20 volcanoes shrank to sub-pixel and vanished.

**Finding 3 — the channel-initiation threshold is also purely a grid-cell fraction**, independently
re-implemented at ~18 call sites (`flowThresh = GW*GH*0.0004`) with zero real-km conversion,
despite `flowField` itself carrying no inherent area unit either (`computeFlow` accumulates a
rain-weighted cell count, re-normalised to mean 1/cell — a dimensionless "N cells' worth of
drainage," not km²).

## 2. Grounding in real hydrology and procedural-terrain practice

- **Montgomery & Dietrich (1988, 1992), "Where do channels begin?" / "Channel initiation and the
  problem of landscape scale."** A real channel starts where contributing catchment area exceeds a
  roughly constant threshold (~0.1–1 km² for typical terrain, weakly slope-dependent — already the
  literature this repo's own `docs/research/natural-rivers.md` cites for the River-Density/
  Min-Stream-Order sliders' slope-area law, `channelThreshold(baseThresh, slopeN, density)`). A
  threshold expressed as a *fixed fraction of total grid cells* has no relationship to that real
  area at all — it happens to be a reasonable coarse-resolution PROXY at world scale (where a
  single cell already covers thousands of km², so no finer distinction is resolvable anyway) but is
  simply wrong once resolution is fine enough that a cell's own footprint is smaller than the real
  channel-initiation area.
- **Horton (1945) / Strahler (1957) / Hack (1957)** — real river networks are approximately
  self-similar (fractal) across scales: drainage density (channel-km per km² of land) is roughly
  constant in nature (order 0.5–3 km/km²) regardless of whether you're looking at a whole continent
  or a single watershed. A generator whose relief frequency is fixed *relative to the grid* rather
  than *relative to real km* cannot reproduce that — it shows the same absolute number of "bends"
  in the terrain no matter what real extent the grid represents, so drainage density (in real terms)
  falls as the map gets smaller instead of staying roughly constant.
- **Perlin/Musgrave multifractal terrain practice** (the standard approach in tools like World
  Machine/Gaea when working in "real-world units" mode): noise octaves are pegged to real
  wavelengths, not to a fraction of the canvas. A small area-of-interest naturally resolves more of
  the fine end of the spectrum and less of the coarse end; a large AOI is the reverse. This is
  exactly the "band-limited noise, finer octaves meaningful only once resolvable" framing this
  repo's own `docs/research/multiscale-detail-editing.md` already documents for `addZoomDetail`
  (extra octaves activated by camera zoom) — that mechanism solves "zooming into an *existing*
  generated map reveals more detail" (a render-time augmentation of the coarse `field`, which stays
  unchanged underneath); it does not, and was never meant to, address "does *generating* a small
  region in the first place produce genuinely finer relief" — a `generate()`-time question this
  version answers instead.
- **This codebase already solved exactly this class of bug once**, for a different consumer:
  `_V3D_RATIO0` (v0.67) derives 3D vertical exaggeration from "the TRUE relief:width ratio... 
  normalized so the default (800 km, 4000 m) reproduces the old look EXACTLY... bigger maps
  auto-flatten, smaller maps show a touch more relief." Same anchor-at-the-literal-default,
  one-sided-clamp discipline, now applied to the relief-*generation* frequency itself.

## 3. Design

**Stage A — crater/volcano radius ceiling.** `clampFeatureRadiusCells(radCells,gw,gh)` caps a
single feature at `FEATURE_RADIUS_MAX_FRAC=0.12` of the shorter grid axis. A universal correctness
fix (a crater covering the whole map is wrong at any scale), not gated to small regions — it applies
wherever a large/rare crater or volcano roll would otherwise exceed the ceiling, including
occasionally at the literal default scale (measured: at 800 km/`GW`=256, the largest crater
observed dropped from 44.8 to 19.7 cells once clamped).

**Stage B — relief noise frequency becomes real-km-aware.** `terrainDetailK(gw,mapWidthKm)`:
```js
const REF_CELLKM = 800/2048;      // the app's own literal untouched default (mapWidthKm:800, resW:2048)
const TERRAIN_DETAIL_MAX_K = 16;  // safety cap, same spirit as addZoomDetail's extra<=6
function terrainDetailK(gw, mapWidthKm){
  const cellKm = (mapWidthKm>0?mapWidthKm:800)/gw;
  return Math.min(TERRAIN_DETAIL_MAX_K, Math.max(1, REF_CELLKM/cellKm));
}
```
`REF_CELLKM` is not an arbitrary choice — `mapWidthKm:800` and `resW:2048` are the app's own literal
defaults (the state object literal and the pre-selected resolution button), true for both Region
*and* World mode (`_suGenSync` seeds the setup form's width from `state.mapWidthKm||800`
regardless of the extent toggle). The one-sided `Math.max(1,...)` is deliberate: at or above the
reference cell size — world scale, or any region ≥800 km at ≤2048 resolution, i.e. the
overwhelmingly common case — `k===1` exactly, so the two frequencies below are provably unchanged.
Threaded into exactly two constants:
- `heightParams().nf = 5.0 * terrainDetailK(...)` — the height formula's own noise term directly
  shapes ridges/valleys/drainage divides.
- `heteroParams().hf = 1.5 * terrainDetailK(...)` — crustal heterogeneity feeds the same height
  formula's `Hwt*hetero[i]` term, so it needs the same treatment for a coherent result.

**Warp frequency and `state.tect.blurR` (tectonic/orogeny blur radius) are deliberately untouched**
— warp is a coordinate *distortion*, not a shape-driver, and scaling its frequency without
re-deriving its amplitude risks coordinate-folding artefacts; `blurR` is an existing, intentional
"mountain-range chunkiness" user slider, not part of this bug. Macro tectonic structure keeps its
current relative size; only the fine multifractal texture that drives local drainage-divide density
changes. Both are disclosed, deliberate scope cuts, not silent omissions.

Because `fbm()`/`ridged()`/`pfbm()`/`pridged()` are always fixed at 6 internal octaves and
frequency is just a multiplier on the sampled coordinate (not an extra loop), this costs **zero**
additional compute — simpler and cheaper than an `addZoomDetail`-style extra-octave-loop design.

**Stage C — one canonical channel threshold, made real-km-aware too (informed by measurement, not
assumed).** Step 1 consolidated the ~18 independent `GW*GH*0.0004` recomputations into one
`riverFlowThresh(gw,gh)` (a pure refactor, byte-for-byte match to the old inline literal). Measuring
Stage B *alone* (before Step 2) turned up a real problem: at a 50 km region, the finer relief
measurably made the drainage network **sparser**, not richer — channel-cell count dropped from
4233 (v1.59 baseline) to 3345, and max Strahler order fell from 4 to 3. The reason: fragmenting the
terrain into more, smaller local bumps interrupts the long contiguous downhill runs flow
accumulation needs to build up magnitude, so *fewer* cells cleared a threshold still calibrated for
the old, coarser relief's typical accumulation values. This is exactly the trap flagged in advance
(see the plan this version shipped from) — a hand-derived Montgomery-Dietrich area constant either
did nothing or declared the whole map a river depending on the value picked, so instead of guessing
a constant, Step 2 reuses the **same** `terrainDetailK` that governs the relief itself:
```js
function riverFlowThresh(gw,gh){ return gw*gh*0.0004/terrainDetailK(GW,state.mapWidthKm); }
```
Dividing by the same one-sided multiplier that raised the noise frequency keeps both knobs
coherent (finer relief ⇒ proportionally less accumulated area required to channelize) and inherits
the identical bit-identity guarantee (`k===1` at/above the reference scale ⇒ the exact legacy
`gw*gh*0.0004`). The divisor always reads the **world's own** `GW`/`state.mapWidthKm` (module
globals) rather than the `gw`/`gh` parameters passed in, so a call site classifying an LOD tile's
own (smaller) grid still gets the *world's* real detail level, not a tile-local mis-estimate.

## 4. Measured results

All measurements: seed 12345, controlled sweep, `docs/research`-adjacent scratch probes (not
committed — see CHANGELOG for the exact scripts/methodology).

**Crater/volcano ceiling** (`GW`=256): largest radius as a fraction of `GW`, before → after:

| mapWidthKm | before | after |
|---|---|---|
| 40000 (world) | 0.003 | 0.003 (unchanged — floor never binds a ceiling) |
| 800 (default) | 0.175 | 0.077 (ceiling now binds occasionally) |
| 200 | 0.700 | 0.077 |
| 50 | **2.799** (bigger than the map) | 0.077 |

**River/relief scaling** (`GW`=256, before Stage C step 2 → after; world/800/200 km rows are
`terrainDetailK===1` at this resolution and are unaffected either way):

| mapWidthKm | channel cells (v1.59) | Stage B alone | Stage B+C | max Strahler order |
|---|---|---|---|---|
| 50 | 4233 | 3345 (worse) | **6001** (+42%) | 4 → 3 → **4** (recovered) |

Distinct river polylines at 50 km: 718 (v1.59) → 771 (B alone) → **1497** (B+C, +109%).

**At a more realistic resolution** (`GW`=1024/2048, isolating `terrainDetailK`'s effect directly):

| scenario | `terrainDetailK` | channel fraction | polylines |
|---|---|---|---|
| default (800 km / 2048 res) | 1 (unchanged) | 0.00216 | 503 |
| region (100 km / 1024 res) | 4 | 0.02556 (12×) | 2859 (5.7×) |
| region (25 km / 1024 res) | 16 (capped) | 0.06044 (28×) | 8945 (17.8×) |

**Bit-identity, isolated and confirmed**: with craters/volcanoes disabled (removing Stage A's
universal, scale-independent effect from the comparison), v1.59 and v1.60 produce **identical**
`field`/`temp`/`rain`/`flow` FNV hashes at the standard test scenario (seed 12345, region, 256px,
`mapWidthKm`=800) — proving Stage B/C's `terrainDetailK` mechanism is a true no-op at/above the
reference scale, exactly as designed. The *only* reason the standard `hash_gen1.js` battery shows a
difference is Stage A's crater/volcano ceiling clamp, which is not scale-gated by design (a crater
covering the whole map is wrong at any resolution) and can occasionally fire even at the default
scenario when a large/rare crater roll would otherwise have exceeded the new sane ceiling.

## 5. Known scope cuts

- **Warp frequency** stays grid-relative — only height/heterogeneity noise (the terms that actually
  shape ridges/valleys) were made real-km-aware. Scaling warp without re-deriving its amplitude
  risks coordinate-folding artefacts that would need their own tuning pass.
- **`state.tect.blurR`** (stress/flexure/orogeny blur radius) stays a pure cell-count slider —
  mountain-RANGE macro-shape doesn't get an independent real-km recalibration this pass; only the
  fine multifractal texture that drives local drainage-divide density does. Sufficient to fix the
  reported branching complaint (measured above) without touching the more invariant-sensitive
  macro-tectonic shape code.
- **Erosion kernels, coastal/glacial passes, river carving (`carveRiverValleys`) and fjord masking**
  were all audited and confirmed to already be purely resolution-relative (fixed cell counts or
  `GW`-fractions, no real-km dependence at all) — none exhibit either failure mode this version
  fixes (neither "grid-blind" nor "unclamped km-to-cell blowup"), so none needed changes.
- **`TERRAIN_DETAIL_MAX_K=16` and `FEATURE_RADIUS_MAX_FRAC=0.12`** are reasoned starting values
  confirmed against the measurements in §4, not independently calibrated against a historical
  drainage-density figure the way e.g. v1.34's 9:1 farmer ratio was — reasonable defaults, open to
  retuning if a future report asks for it.

## 6. Sources

- Montgomery, D.R. & Dietrich, W.E. (1988). Where do channels begin? *Nature*, 336, 232–234.
- Montgomery, D.R. & Dietrich, W.E. (1992). Channel initiation and the problem of landscape scale.
  *Science*, 255(5046), 826–830.
- Horton, R.E. (1945). Erosional development of streams and their drainage basins. *GSA Bulletin*,
  56(3), 275–370.
- Strahler, A.N. (1957). Quantitative analysis of watershed geomorphology. *Trans. AGU*, 38(6),
  913–920.
- Hack, J.T. (1957). Studies of longitudinal stream profiles in Virginia and Maryland. *USGS
  Professional Paper 294-B*.
- Musgrave, F.K., Kolb, C.E. & Mace, R.S. (1989). The synthesis and rendering of eroded fractal
  terrains. *ACM SIGGRAPH Computer Graphics*, 23(3), 41–50.
- Internal: `docs/research/natural-rivers.md` (slope-area channel threshold, already shipped),
  `docs/research/multiscale-rivers.md` / `docs/research/multiscale-detail-editing.md` (the LOD-tile
  render-time detail precedent this version deliberately does NOT duplicate), engine `_V3D_RATIO0`
  (v0.67, the anchor-at-the-default precedent this version's `terrainDetailK` directly follows).

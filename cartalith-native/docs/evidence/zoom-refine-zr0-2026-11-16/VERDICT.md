# ZR-0 verdict: NO-GO (2026-11-16)

The bar (`ZOOM_REFINEMENT_SCOPE.md` §5 ZR-0): an offline-refined tile reaches at least 0.6 of the native map's land
luma gradient at 0.471 and 0.078 km/px, with `bind_r` within 0.15 of the native map's, for at least one solver, at no
more than 30 ms per tile on one PC thread. The numbers are in `metrics.txt`. How they were measured is in that file's
header and in `cartalith-native/crates/cartalith-godot/tests/zr0_refine.rs`.

| km/px | place | native grad / bind_r | FG noise grad (ratio) / bind_r | ZR-BW grad (ratio) / bind_r (diff) | ZR-EF3 grad (ratio) / bind_r (diff) |
|---|---|---|---|---|---|
| 0.471 | mountain | 3.43 / 0.43 | 13.03 (3.80) / 0.21 | 14.96 (4.37) / 0.39 (0.04) **pass** | 14.78 (4.31) / 0.39 (0.04) **pass** |
| 0.471 | plain | 3.15 / 0.48 | 2.51 (0.80) / 0.13 | 3.37 (1.07) / 0.11 (0.36) **fail** | 3.28 (1.04) / 0.10 (0.38) **fail** |
| 0.471 | FG pivot | 3.11 / 0.47 | 2.48 (0.80) / 0.17 | 3.37 (1.08) / 0.13 (0.34) **fail** | 3.28 (1.05) / 0.14 (0.33) **fail** |
| 0.078 | mountain | 5.08 / 0.50 | 5.88 (1.16) / 0.48 | 14.25 (2.80) / 0.15 (0.35) **fail** | 14.25 (2.80) / 0.14 (0.35) **fail** |
| 0.078 | plain | 4.90 / 0.54 | 2.20 (0.45) / 0.18 | 3.07 (0.63) / 0.20 (0.34) **fail** | 2.90 (0.59) / 0.25 (0.29) **fail** |
| 0.078 | FG pivot | 4.88 / 0.59 | 2.24 (0.46) / 0.14 | 3.17 (0.65) / 0.17 (0.42) **fail** | 2.95 (0.60) / 0.19 (0.40) **fail** |

Ratios are against the same-terrain native window. Against the whole native map at fit (4.85 at 0.471, 6.51 at 0.078),
ZR-BW is 0.69-3.08 at 0.471 and 0.47-2.19 at 0.078.

**Cost (one PC thread, median of land tiles at levels 4-8, two runs):** ZR-BW **20.4 ms** (19.5..21.3) passes.
ZR-EF3 **37.0 ms** (35.1..38.5) fails. On the 6T (one thread) ZR-BW takes 80.7 ms and ZR-EF3 139.7 ms.

**Verdict.**

- The gradient part passes against same-terrain native windows, overshooting by 2.8-4.4x on mountains. It fails
  against the whole native map at 0.078 on flat ground (0.47-0.49).
- `bind_r` fails in 5 of the 6 place-and-scale cells, for both solvers.
- So no solver passes the bar, and ZR-1 does not start. Per the scope, ZR closes with this finding and the owner decides.

**The kernel, if the owner continues anyway:** the Braun-Willett light solve over `tile_flow`'s D8 routing. It is the
only candidate within 30 ms. EF-3 is 1.8x slower on the PC (1.7x on the 6T), and EF-3 looks no different: gradient
within 5 %, `bind_r` within 0.05, frames indistinguishable.

**Why it fails (frames looked at, every sheet in this folder):**

- **Noise dominates the refined tiles.** The detail is the one FG octave added per level. The steps that should make
  landforms (drainage, the burn and erosion) add only 4-12 % to the gradient over the steps-2-3-only control
  (`zrbase`). They lift `bind_r` by at most 0.13, and that is on the mountain at 0.078.
- **The native maps have km-scale ridge and valley systems.** These come from orogeny and erosion at their own cell
  size: dendritic valleys 10-30 km long in the 0.078 frames, combs of ridges at 0.471.
- **Nothing in the per-tile chain builds structure at that scale from a smooth 19.6 km cell.** The burn carves only
  channels 2-4 texels wide (150-300 m at level 8), and stream power over 3 passes barely moves a field this smooth.
- **The flat-ground frames read as uniform sand texture**, with a few straight facets where a D8 channel was burned
  (the 0.078 plain sheet).

**Other findings:**

- **Seams.**
  - 0 edge texels off the base, in every chain at every level (76 k to 154 k checked per chain).
  - Channel continuity across seams is 94.7-98.1 %, above ZR-1's 90 % bar.
  - The seam step ratio is 1.06-1.21, so a faint seam signature remains.
  - A first run reported about 3 % of edge texels off the base. That was a harness rounding bug, fixed before these
    numbers.
- **Determinism.**
  - Every level's digest is identical at 1, 4 and 16 threads, for both solvers.
  - The world field digest and the level-8 digest are identical on the PC and the 6T, for both solvers.
- **Physical units do not agree with the renderer's units.**
  - One 35 deg talus pass moved 1.6-1.7 M texels on the world mountain at level 8.
  - The reason: the FG law's amplitude, which is in normalised height per base cell, implies slopes steeper than 35 deg
    at 76 m texels on a 19.6 km per cell world. Any physically scaled step (a talus angle, a burn depth in metres)
    fights the per-map normalisation the renderer shades by.
- **Cold jump.**
  - One level-8 tile needs 41 ancestor tiles over levels 1-7.
  - Its cone is 404 x 404 base cells, because a level-1 tile spans 128 base cells. Cones should be counted per texel
    window, not per aligned tile.
- **The offline FG figures differ from FG's screen table.**
  - FG noise reaches 0.80 and 0.46 of the native window here, against FG's windowed 0.18-0.23. These frames have no
    shell composite and no river layer, and are compared per window.
  - The FG frame of the world mountain at 0.078 shows rectangular flat shelves at tile boundaries. This was seen in the
    offline frame only and was not investigated.
- **The colouring cost is larger than the refinement.** Colouring one 256^2 refined tile takes 61 ms on one PC thread
  and 189 ms on the 6T.
- **The band step is the most expensive refinement step** (6.7 ms PC, 23 ms 6T), because it does per-texel coarse
  relief and roughness lookups. This could be cached.

**Not measured here:**

- compression;
- the Pad 3;
- the app's own process;
- seeds other than 1200.

The scope asks for 3 seeds before ZR-1. With this verdict there is no ZR-1 to gate.

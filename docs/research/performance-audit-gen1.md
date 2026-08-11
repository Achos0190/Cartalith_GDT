# Cartalith Gen1 — Performance & Architecture Audit

**Target:** `Cartalith Gen1 v0.57.html` (13,365 lines at audit start; commit `f081fb3` = the measured baseline)
**Date:** 2026-07 · **Method:** static reverse-engineering of every subsystem + a new in-app instrumentation layer (`PERF`) + a Playwright measurement harness (`tests/perf/`)
**Measurement environment:** headless Chromium on a 4-core VM, WebGL2 via SwiftShader (software rasterizer). Figures are tagged **[measured]** (this harness, this machine) or **[estimated]** (reasoned from code, unverified). GPU-adjacent numbers are software-GL results — real-GPU machines will see the GPU-offloaded stages run faster; all-CPU stages (flow, carveRivers, the pixel loop) are representative.
**Determinism contract:** every optimization implemented in this round keeps the default world **bit-identical at a fixed seed** — FNV-1a hashes of `field`/`tempField`/`rainField`/`flowField` **and** the rendered RGBA, compared A/B against the pre-change build in the same browser binary (`tests/perf/hash_gen1.js`, 35-config matrix). Output-changing ideas appear only in the roadmap (§13).

---

## 1. Executive summary

Cartalith Gen1 is a mature, already-heavily-optimized engine: GPU fragment-shader compute with per-op CPU self-validation (R32F), a self-validating multicore worker pool for the generation noise loops, four self-contained erosion worker kernels, a tiled LOD pyramid + IndexedDB atlas as the sanctioned high-resolution path, a coarse 240-cell climate grid, ≤384px civ pathfinding, RAF render coalescing and CSS-transform pan/zoom. The audit therefore found the untreated cost concentrated in **the render/interaction path and one algorithmic hotspot**, not in the generation architecture.

Five headline findings, all fixed this round, all bit-identical:

| Finding | Before → after [measured, 2048², seed 12345] |
|---|---|
| Civ label/icon drags repainted the full 2.7-Mpx terrain per mouse move | **1,380 ms/move → 0.03 ms/move** (0 pixel-loop repaints during a 60-move drag, was 60) |
| `computeFlow`'s comparator sort — the single hottest generate() line, ≥3 runs per generate + one per terrain edit | **1,005 ms/call → 120 ms/call** (stable LSD radix sort, proven element-identical) |
| `_seaH`/`_seaShade` sea-shading fields rebuilt every biome render | renderNow prologue **248 ms → ~0** on repeated renders (cached, generation-counter invalidation) |
| `landColorCore` allocated ~10 short-lived arrays + a closure per pixel (~2.7M calls/render) | scalar accumulators; pixel loop **1,170 → 1,050 ms** wall + the GC churn removed |
| **Correctness bug**: `loadZip`/`loadImage` rendered the imported world through the previous world's derived caches (wrong content, wrong *length* after a resolution change) | fixed (`invalidateFieldCaches()` + length guard) — same bug class as the historical v0.071 warp-cache bug |

Net: `generate()` at 2048² **23.3 s → 19.6 s** (−16%), repeated `renderNow` at 2048² **1,435 → 1,066 ms** (−26%), and interactive dragging went from unusable (0.7 fps) to free. The remaining big-ticket items (§13) are the `carveRivers` pipeline (41% of generate), GPU-side rendering, and noise-field memoization.

---

## 2. Architecture overview

One self-contained HTML file; three sequential classic `<script>` blocks sharing one global scope by document order:

| Block | Lines | Contents |
|---|---|---|
| 1 | 1063–8470 | The elevation-foundation engine (~v0.147 level): state, noise, tectonics, climate, hydrology, erosion (+ 4 worker kernels), GPU compute object, renderer, LOD/atlas, exporters, most UI wiring |
| 2 | 8475–12180 | Civilization layer: factions/territory, settlements, ways/routes/journey planner, labels, manual icons, civ canvas, sync-to-`state.civ` monkey-patches |
| 3 | 12181–13363 | Asset Library page + late wiring |

Key architectural facts that shape any optimization:

- **`renderNow` is double-wrapped:** the original (line ~5320) is monkey-patched once to refresh the Shift+D overlay, and again by the civ layer (bake-cache + `drawCivLayer` append). Instrumentation and phase timing live inside the original.
- **Canvas stack:** `#view` (terrain, `putImageData` of a persistent `ImageData`, fixed GW×GH), `#civCanvas` (civ vector layer, HiDPI/zoom-scaled), `#polyOverlay` (transient overlays + water animation). Non-LOD pan/zoom is a pure CSS transform — **no terrain re-render** — only the civ layer redraws. This separation is what made the drag fix (§8) safe and trivial.
- **Cross-block hazard (established this project):** an immediate top-level call from an earlier block to a later block's function throws and silently aborts the rest of that `<script>` block. All new cross-block references added this round are runtime-only (event handlers / function bodies).
- **`file://` contract:** no SharedArrayBuffer (no COOP/COEP), workers via blob-URL stringification, IndexedDB feature-detected. This forecloses shared-memory parallelism (§10).

## 3. Dependency graph (module level)

```
state (params) ──► generate() ──► field ─┬─► computeFlow ──► flowField ─► rivers/erosion/TWI
                                          ├─► refreshClimate ──► tempField, rainField ─► biomes
                                          ├─► refreshGeoid ──► geoidField (nullable)
                                          └─► renderNow ──► vctx pixels
derived caches (lazy, module globals):
  _waterBody, _biomeRaster, _cartBiome/_cartTerrain, _lithField/_soilField/_waterField,
  _resourcePots/_carryCap/_settleSuit, _wildlife, _riverNet, _seaH/_seaShade (now cached),
  _lodCache (LRU 48) / _atlasBaked (IndexedDB)
invalidation constellation (the five sites that null _waterBody — now also bump _fieldGen):
  generate() · computeFlow() · shiftGridX/center · depositWater · sculpt
  (+ refreshGeoid and, since this round, loadZip/loadImage via invalidateFieldCaches())
civ layer reads: field, state.seaLevel, currentWaterBodies(), the ≤384px cost grids
civ layer owns:  civTerritory, civWays/civJourneys, state.places/labels/mapIcons, civCanvas
```

Invalidation is deliberately **coarse** (whole-cache null-out at a handful of sites) — appropriate for a batch-generation tool where edits are op-sized, not per-cell. The round added two *generation counters* (`_fieldGen`, `_civTerrGen`) on top of the same constellation rather than a new dependency system; a full dirty-graph would be over-engineering at this state-change granularity **[assessed, not measured]**.

## 4. Execution pipeline — generate(), with measured stage costs

Order (unchanged by this audit; the flow→climate→flow sandwich is deliberate LEM-style coupling):
invalidate+geoid → continentality → warp* → plates+stress → flexure → baseBlur → boundaryDist+age → heterogeneity* → resistance → orogeny (opt-in) → height* → normalize → volcanism+craters → **flow1 → climate → flow2** → carveRivers (default ON since v0.145) → tides → render. (* = multicore GENPOOL when ≥262,144 cells.)

**[measured]** medians of 3 (1 warmup discarded), seed 12345, default params, after this round's changes; baseline in parentheses:

| Stage (ms) | 512² | 1024² | 2048² |
|---|---|---|---|
| plates+stress† | 302 | 1,111 | 4,402 (4,541) |
| flexure† | 148 | 574 | 2,194 (2,186) |
| baseBlur† | 30 | 100 | ~390 |
| heterogeneity* | 29 | 43 | 160 |
| height* | 31 | 42 | 165 |
| flow1 / flow2 | 76 / 75 | 340 / 330 | 777 / ~760 (1,590 / 1,479) |
| — of which sort | 3.1 | 26 | **120 (1,005)** |
| climate (coarse 240 grid) | 182 | 234 | 370 |
| carveRivers | 533 | 1,894 | **8,079 (9,895)** |
| render | 178 | 575 | 2,159 (2,312) |
| **total** | **1,567 (1,740)** | **5,076 (5,914)** | **19,601 (23,252)** |

† runs through GPU `gaussBlur` — here on SwiftShader; real GPUs will do these stages substantially faster, so on real hardware **carveRivers' share is even larger** than the 41% measured here.

Notable: `warp` measures ~0 on repeat same-seed runs because `computeWarp` caches on seed+amount+length (its first-run cost is hidden by the warmup-discard policy). `carveRivers` internally runs another `computeFlow` + `buildRiverNetwork` + carve — it is the #1 CPU stage at every resolution and the top roadmap item (§13).

## 5. Computational complexity

| Algorithm | Complexity | Notes |
|---|---|---|
| computeFlow ordering | was O(n log n) comparator TimSort; **now O(n) LSD radix** (4 stable byte passes) | proven element-identical incl. −0/duplicates (§11) |
| computeFlow accumulation | O(8n) D8 scan | order-dependent float sums → tie order is part of the contract |
| gaussBlur / box blurs | O(n·passes), separable | GPU when full-grid; coarse 240-grid always CPU (Invariant 3) |
| priority-flood (erosion kernels ×3, weather) | O(n log n) binary heap | deliberately duplicated per worker kernel (Invariant 11) — **not** a cleanup target |
| pixel loop | O(n · k) — k = per-pixel shader cost, dominated by 4–5 `vnoise` calls + materialWeights + palettes | §8 |
| civ Dijkstra | O(E log V) on ≤384² grid, no decrease-key (lazy dupes) | `_civHierarchicalNetwork` ≈ 2·P runs ×≤3 passes (§11) |
| chamfer / JFA distance fields | O(n) / O(n log n) | already idx-optimized (v0.117) |
| Zhang–Suen thinning, Strahler, RDP, Catmull-Rom | O(n)–O(n log n) | debug/overlay-gated |

## 6. Memory analysis

- **Persistent:** ~23 full-grid Float32Arrays + 3 small-int grids (`allocate()`), ≈ 24 × 4 B × n. **[measured via the Shift+D overlay inventory]** ≈ 420 MB at 2048², ≈ 1.7 GB at 4096², ≈ 6.7 GB at 8192² — memory, not compute, remains the reason a full-grid 8K world is off the table; the LOD pyramid/atlas is the sanctioned 8K path (unchanged conclusion from the v0.117 audit, now with the unified file's larger cache population).
- **Per-render transients (before → after this round):** up to six full-grid geoid-difference IIFE copies → **one shared buffer, built at most once per render**; `_seaH`+`_seaShade` (2 full grids + `src.slice()`) per render → **cached across renders**; per-pixel `[r,g,b]`/closure/mix allocations in `landColorCore` (order 10⁷ small objects per 2048² render) → **scalar accumulators** (palette-fn returns remain — roadmap).
- **Per-call transients:** `computeFlow`'s 2.7M-slot index Array per call → persistent radix scratch (3 typed arrays, reused); `waterAnimFrame`'s per-RAF-frame `ImageData` → reused; `drawCivLayer`'s per-redraw territory `ImageData` + occupancy grid → key-cached / reused. Remaining: `computeFlow`'s `acc` Float32Array per call (returned to callers → kept fresh deliberately; roadmap), the erosion CPU-fallback delta arrays (GPU normally covers these), and `exportZip`'s hold-everything-then-zip peak (§13).

## 7. Raster-pass audit (per default biome render)

Before this round, one `renderNow` at defaults ran: `smoothSeaH` (slice + 4 box passes) + `seaShadeFrom` (1 hillshade pass) + the pixel loop (1 pass, heavy) + `putImageData`. With geoid/waves/AO/SVF/shadows enabled, each consumer added its own full-grid geoid-subtraction pass. **After:** the sea passes run only when `_fieldGen`/sun/exaggeration actually changed; the geoid subtraction runs at most once per render; slider-gated fields (AO/SVF/shadows/crest/SDFs) still rebuild per render when enabled **[roadmap: same counter-keyed caching applies trivially if profiling ever shows it matters]**. Derived *simulation* rasters (`_biomeRaster`, water bodies, affordances) were already lazily cached with sound invalidation — verified, credited, and reused (the new caches hook the identical constellation).

## 8. Rendering analysis

**[measured]** repeated `renderNow` at defaults (median of 20): 2048² = 1,066 ms → **prologue ~0 / pixels 1,050 / overlays 15**. The pixel loop is now >98% of a repeat render, and within it **[estimated from code]** the cost per land pixel is dominated by: 4–5 `vnoise` evaluations (nLow, nHi, two bioJitter octaves), `materialWeights`, ~7 palette-function calls (each returning a fresh array — the remaining allocation), multi-scale hillshade, and in `surfaceColor` the slope/TWI/aspect/curvature samplers.

- **Redraw policy:** there is no dirty-rectangle system; every `renderNow` repaints the full grid. RAF coalescing (`scheduleRender`) collapses same-frame requests, and the civ `_civBakeCache` (full-frame ImageData LRU) short-circuits repaints when terrain is committed. With the drag fix, the remaining full repaints are all justified (sim-data changes).
- **The interaction bug fixed:** all four label/icon drag branches called `renderNow()` per pointermove although those elements live on `civCanvas` — 1,380 ms/move at 2048² for zero visual difference. `drawCivLayerAuto()` is the documented contract for interactive quick-redraws; the handlers simply predated it.
- **Frame-loop work:** only the opt-in water animation runs per-frame (≤400k-cell gated, now allocation-free per frame).

## 9. GPU evaluation

Already on GPU (WebGL2 fragment ping-pong, R32F with RGBA32F and CPU fallbacks, per-op self-validation): thermal, hydraulic diffuse, full-grid gaussian blur, normalize, temperature (when geoid/albedo off), coastal. Correctly **not** on GPU: procedural noise (integer-hash CPU semantics can't be reproduced → would fail self-validation by design), coarse-grid climate blur (Invariant 3), D8 flow accumulation and priority-flood (sequential dependencies), plate Voronoi/Lloyd (small n).

**GPU renderNow (Track B of the compute-offload audit)** remains the biggest *display* win on paper — the 1 s pixel loop at 2048² is embarrassingly parallel — but stays roadmap-only: the per-pixel shader depends on the CPU integer-hash noise (parity impossible under the self-validation regime), it would need every gated style branch ported, and the established bit-identity discipline would have to be renegotiated per-branch. Same verdict as the prior audit, now with a measured cost to weigh it against. WebGPU: still deferred (file:// inconsistency, third backend).

## 10. Parallelization analysis

In place and verified healthy: GENPOOL (noise fills, copy-in/transfer-out, self-validated bit-identical, ≤8 workers), four erosion kernels in blob-URL workers with sync fallbacks, `_eroBusy`/`_busyChain` serialization (the v0.136 race fix). Constraints that bound further parallelism: no SharedArrayBuffer under `file://` (copy/transfer only — fan-out cost grows with grid size), Invariant 11 self-containment (kernels must stringify), and the headless suite's synchronous path.

The natural next candidates **[estimated]**: `carveRivers`' network build + carve (worker-ifiable like the erosion ops — same copied-buffer pattern; ~8 s at 2048² is easily worth one buffer round-trip), and `computeFlow` itself (the radix sort made the ordering cheap, but the accumulation is inherently sequential in height order — only the sort was parallel-friendly and it no longer needs it).

## 11. Algorithm review

- **computeFlow ordering (replaced this round):** comparator TimSort → stable LSD radix over float32 bit patterns. 8.4× **[measured]**. Bit-identity is structural: monotone sign-flip bit mapping; −0 canonicalized to +0 (comparator-equality); byte-wise counting sort is stable so ties keep ascending-index order — exactly the spec-stable `Array#sort` result. Proven element-by-element on the real field, post-erosion field, and a −0/duplicate/denormal synthetic.
- **roadDijkstra:** binary heap without decrease-key (lazy duplicate pushes) — fine at ≤384²; not worth a pairing heap **[assessed]**.
- **`_civHierarchicalNetwork`:** ~2·P full-grid Dijkstras ×≤3 passes (+ betweenness per pass in `_civIterativeAutoWorld`). P is capped (≤40) and the grid is ≤384² — acceptable today; if it ever grows, the fix is reusing pass-1 distance fields and a proper multi-source formulation (roadmap).
- **MinHeap ×3 duplication:** deliberate (Invariant 11) — re-affirmed, not a cleanup target.
- **`Math.min`/branch-min:** re-affirmed the v0.117 decline (V8 intrinsics + NaN/−0 edge risk).

## 12. Ranked hotspots (post-round)

1. **carveRivers** — 8.1 s at 2048² **[measured]**, 41% of generate, CPU-only, default-on. Top target.
2. **Pixel loop** — 1.05 s at 2048² **[measured]**; vnoise-dominated **[estimated]**; N1 memoization or GPU (§13).
3. **plates+stress / flexure** — 4.4 s / 2.2 s here, but SwiftShader-inflated; real-GPU machines see less **[measured, caveat]**.
4. **flow accumulation** — ~650 ms ×3 at 2048² after the sort fix; sequential; hard floor without algorithm change.
5. **exportZip peak memory** — all whole-grid buffers live simultaneously before `zipStore` **[estimated]**; matters at 4K+.

## 13. Optimization roadmap

**Shipped this round (all bit-identical, hash-verified):** per-stage/per-phase instrumentation + Shift+D surfacing; drag → civ-layer-only redraw; `_seaH`/`_seaShade` caching + shared geoid-effective field; importer stale-cache correctness fix (+ atlas world-key idiom in `loadZip`, LOD cache/edit invalidation); `computeFlow` radix sort with persistent scratch; `landColorCore` scalar accumulators; `waterAnimFrame`/territory-raster/occupancy-grid/`placeMapIcons` reuse & caching.

**Next, in expected value order:**
1. **carveRivers decomposition** — profile its internal flow/network/carve split (the instrumentation hooks exist); skip the redundant internal `computeFlow` when `flowField` is already fresh **[estimated: up to ~1 s]**; worker-ify the remainder like the erosion ops (responsiveness, not wall time).
2. **N1 noise-field memoization** — nLow/nHi/bioJitter are pure functions of (x, y, GW); full-frame Float64 caches (~65–90 MB at 2048²) rebuilt per resolution would be bit-identical (Float32 storage would NOT be — vnoise returns doubles) and remove ~4 vnoise calls/pixel **[estimated: 30–50% of the pixel loop]**. Memory-vs-speed knob; measure first.
3. **Palette-function scratch-ification** (C2) — the remaining per-pixel allocations; mechanical but touches ~10 functions; needs the same 35-config battery.
4. **computeFlow `acc` reuse** — requires auditing the ~20 call sites for held references (the function currently returns a fresh array by contract).
5. **exportZip streaming** — write entries incrementally instead of holding all buffers + the final Blob simultaneously.
6. **Slider-gated render fields** (AO/SVF/shadows/crest/SDF) — same `_fieldGen` caching as `_seaH` if their rebuild ever shows up in `PERF.render.prologue` with the sliders on.
7. **GPU renderNow / WebGPU** — highest ceiling, highest cost; parity regime must be renegotiated first (§9).
8. **`_civHierarchicalNetwork` Dijkstra reuse** — only if the settlement cap or grid ever grows.

**Instrumentation now available for all of the above:** `PERF.gen` (per-stage ms), `PERF.render` (prologue/pixels/overlays), `PERF.counters.renderPixelLoop`/`drawCivLayer` (repaint accounting), `PERF.flowSortMs`; `tests/perf/perf_gen1.js` (timings + drag sim + hashes, JSON out) and `tests/perf/hash_gen1.js` (A/B bit-identity battery, `--full` = 35 configs). Golden hashes are deliberately never stored — hashes are engine-version-dependent, so comparisons are always two live runs in one binary.

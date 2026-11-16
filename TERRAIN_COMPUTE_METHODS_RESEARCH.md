# Terrain compute methods: sources for cheaper calculations that keep (or beat) the same fidelity (2026-11-16)

**Research, not a scope; no status column.** Written for `WORLD_SCALE_TERRAIN_PLAN.md` §10 (cheaper methods), §10a (GPU) and
§7 (storage), from the owner's request: *"find sources for software and new (easier) calculations specifically aimed at
keeping the same results/fidelity or better"*, with the GPU available on PC (RX 7800 XT, the 9800X3D's iGPU) and on phones
(Adreno 630, Snapdragon 8 Elite), less powerful there.

**How to read it.** Every claim is tagged **[source]** (read in the document or page named, this session) or
**[inference]** (my reasoning, or recalled and not re-read; check before relying). Where a fetch failed or only an
abstract was visible, the row says so. Speed-ups are the authors' own, on their hardware, and **none of them has been
measured on this project's data**: each section ends with "what to measure first". "Exact" means the output equals the
serial method's output; "§7p" means the change makes a new world and needs the switch and re-baseline of `DECISIONS.md`
§7p.

Verified this session by opening the text: the FastFlow paper (PDF text), the WGSL specification source
(`gpuweb/gpuweb`, `wgsl/index.bs`), the RichDEM, fastscapelib, meshoptimizer and Basis Universal repositories, Rune
Skovbo Johansen's erosion-filter write-up, Terrain Diffusion (arXiv). Everything else rests on search-result abstracts.

---

## 1. Erosion and landscape evolution

| Source | Year | Claim | Speed-up vs naive | Fidelity evidence | Licence | Fit for deterministic Rust/wgpu |
|---|---|---|---|---|---|---|
| Braun & Willett, "A very efficient O(n), implicit and parallel method to solve the stream power equation", *Geomorphology* 180-181, 170-179. <https://doi.org/10.1016/j.geomorph.2012.10.008> | 2013 | Implicit stream power solved in one O(n) sweep over the stream tree, unconditionally stable [source: title and abstract via search]. | Removes the CFL limit of explicit schemes; the project measured about 4x per tile against its iterative kernel (plan §4/§10) [source: plan]. FastFlow reports 20x larger time steps than explicit [source: FastFlow PDF]. | ZR-0 judged it indistinguishable from EF-3's kernel [source: plan]. FastFlow notes explicit and implicit schemes "do not yield identical solutions" (different discretisation error) [source: FastFlow PDF]. | Paper (algorithm free to implement) | **CPU**, serial order along the stream tree; bit-identical because it is a fixed topological sweep with no atomics [inference]. Already in the tree. |
| fastscapelib (C++/Python), <https://github.com/fastscape-lem/fastscapelib> | maintained | Reusable C++ routines: flow routing, implicit erosion, depression handling (feature list not confirmed from the README page) [source: partial]. | n/a | Reference implementation of the above | **GPL-3.0** [source: repo page] | Reading it for the algorithm is fine; **linking or copying it into a closed-source app is not**. Re-implement from the paper. Note the *Python* `fastscape` package is BSD-3 [source: PyPI/docs], a different project. |
| Cordonnier, Braun, Cani, Benes, Galin, Peytavie, Guérin, "Large scale terrain generation from tectonic uplift and fluvial erosion", *CGF* 35(2). <https://diglib.eg.org/xmlui/handle/10.1111/cgf12820> | 2016 | User-painted uplift map plus implicit stream power on a stream graph generates large terrains "at a low computational cost" [source: abstract]. | No figure seen. | Plausible landforms; qualitative [source: abstract]. | Paper | CPU, as Braun-Willett. The uplift-domain authoring idea does not apply to an already generated world. |
| Cordonnier et al., "Forming terrains by glacial erosion", *ACM TOG* 42(4). <https://doi.org/10.1145/3592422> | 2023 | Glacier expansion/retreat solver; U valleys, cirques, fjords, hanging valleys [source: abstract]. | Terrains computed "in less than 5 minutes on a 16 x 16 km2 terrain" [source: abstract]. | Qualitative. | Paper | CPU/GPU float solver; an expensive **new-world** process, relevant only to Ruling BH/EROSION scopes, not to cost reduction. |
| Schott, Paris, Fournier, Guérin, Galin, "Large-scale terrain authoring through interactive erosion simulation", *ACM TOG* 42(5), art. 162. <https://aparis69.github.io/public_html/projects/schott2023_Uplift.html>, code <https://github.com/H-Schott/StreamPowerErosion> | 2023 | Works in the uplift domain; uses an **approximation** of drainage area and flow routing so erosion runs interactively [source: project page]. FastFlow describes it as explicit stepping with discharge propagated "by a single cell on each simulation step", which underestimates drainage when noise is added to uplift [source: FastFlow PDF]. | "Interactive"; no figure retrieved (HAL PDF blocked). | Approximate by design; FastFlow shows a failure case. | Code on GitHub, licence not retrieved | GPU-friendly but **approximate and path-dependent**, so unsuitable as a generation kernel that must be identical per device. |
| Tzathas, Gailleton, Steer, Cordonnier, "Physically-based analytical erosion for fast terrain generation", *CGF* 43(2). <https://diglib.eg.org/handle/10.1111/cgf15033> | 2024 | Analytical solutions to the stream power law extended from 1D to a 2D heightmap by a **multigrid-accelerated iterative process**; time becomes a function parameter (a slider from light erosion to full replacement); adds landslides/hillslope terms [source: abstract]. | Not retrieved; "fast" per title. | Physically based, compared with simulation in the paper (not read). | Paper | Multigrid on CPU is a fixed schedule, so deterministic if implemented with fixed iteration counts [inference]. Worth reading in full: "time as a parameter" fits per-level refinement and the cut-out regeneration. |
| Mei, Decaudin, Hu, "Fast hydraulic erosion simulation and visualization on GPU", PG 2007 <https://hgpu.org/?p=2256>; Štava et al., "Interactive terrain modeling using hydraulic erosion", SCA 2008 <https://www.cs.purdue.edu/homes/bbenes/papers/Stava08SCA.pdf> | 2007/2008 | Virtual-pipe shallow-water plus capacity-based dissolve/deposit, fully on the GPU; Štava adds multi-layer materials and **tiling** for GPU memory [source: search summaries]. | Interactive on 2007 hardware; no figure retrieved. | Visually plausible; not geomorphologically calibrated [inference]. | Papers | Float stencils with ping-pong buffers: **different results per device** (Ruling EZ item 3 pattern) [inference]. Display-time or sculpt-preview use only. |
| Landlab (MIT) <https://github.com/landlab/landlab>; Badlands | maintained | Python component libraries for landscape evolution; Landlab ships the FastScape stream power component [source: search]. | n/a | Peer-reviewed models | Landlab **MIT** [source: listing]; Badlands licence not retrieved | Reference oracles for a Python cross-check of the Rust kernels; not a runtime dependency. |
| Barnes, "Accelerating a fluvial incision and landscape evolution model with parallelism", arXiv 1803.02977 (*Geomorphology* 2019) <https://arxiv.org/pdf/1803.02977> | 2018 | Parallel formulation of flow accumulation + incision exploiting many-core/GPU/SIMD [source: search]. | **43x** (70 s vs 3 000 s on a 10 000 x 10 000 input) [source: search summary] | Same model, different schedule | Paper | The parallel scheme still routes flow along paths (O(path length) steps); FastFlow improves on it [source: FastFlow PDF]. |
| **Jain, Kerbl, Gain, Finley, Cordonnier, "FastFlow: GPU acceleration of flow and depression routing for landscape simulation", *CGF* 43(7), Pacific Graphics 2024.** DOI 10.1111/cgf.15243 <https://diglib.eg.org/handle/10.1111/cgf15243>; code <https://gitlab.inria.fr/landscapes/fastflow> | 2024 | **It exists and is real.** Flow accumulation as a tree scan by **rake-compress** (pointer jumping plus leaf pruning), O(log n) iterations; depression routing as a minimum spanning tree (Boruvka), O(log² n) iterations; implicit stream-power stepping on top; 4-connected grids; PyTorch/TensorFlow with CUDA kernels [source: PDF text]. | Flow routing **5x** over previous GPU work and depression routing **34x-52x** over an optimised parallel CPU method, at 1024²; "under 55 ms on terrains up to 4096²"; advantage narrows at the largest sizes [source: PDF text]. A reported 14x came from one optimisation, and measured on an RTX A6000, not on this project's GPUs [source]. | "Exact" routing for the chosen recipient tree: the paper contrasts it with approximate explicit methods [source]. Accumulated sums in float are still order-dependent [inference]. | Code on Inria GitLab (licence not retrieved); paper CC-licensed on Wiley [source] | **The best match to the plan's "parallel prefix over a flow tree".** Make it bit-identical by accumulating **integers** (cell counts or fixed-point rainfall in u32): integer addition is associative, so any schedule gives the same result [inference]. A 32k² world has 1.07 G cells, which fits u32 [inference]. Needs a WGSL port (the release is CUDA) and 4-connectivity only: **D8 or D-infinity would change the world** [inference]. |

### Best candidate for Cartalith: erosion

- **Candidate:** keep Braun-Willett implicit stream power on the CPU for generation (already built). For world-scale drainage and tile inflow at 16k/32k, port **FastFlow's rake-compress accumulation with integer weights** as a wgpu kernel.
- **Expected gain:** FastFlow's own flow routing is 5x over earlier GPU work at 1024² [source]; against this project's serial accumulation the gain is unknown. Memory-bound at 1 G cells on a 16 GB card [inference].
- **Measure first:** time of today's accumulation at 4k, 8k, 16k on one CPU thread and all cores (the baseline); then FastFlow's PyTorch reference on a 4k copy of a real world for a result comparison.
- **World identity:** accumulation of integer weights over the **same recipient field** keeps the world identical (exact) [inference]. Switching to the 4-neighbour tree or to the Boruvka depression order is **§7p** unless proven to reproduce today's recipients.

---

## 2. Depression filling and flow routing

| Source | Year | Claim | Speed-up | Exact? | Licence / fit |
|---|---|---|---|---|---|
| Barnes, Lehman, Mulla, "Priority-Flood: an optimal depression-filling and watershed-labeling algorithm for DEMs", *Comput. Geosci.* 62, 117-127. <https://arxiv.org/abs/1511.04463> | 2014 | Floods inward from the edges with a priority queue; "lowest time complexity of any known floating-point depression-filling algorithm"; the improved variant up to **37% faster** than the original [source: search]. | 1.0-1.37x over the earlier version; large over iterative fills [source: search]. | **Exact**: the filled surface is unique (the lowest surface with no depression), so any correct fill gives the same elevations [inference]. | Reference code <https://github.com/r-barnes/Barnes2013-Depressions>; RichDEM is **GPL-3.0** [source]. Re-implement (about 60 lines with a binary heap); CPU, sequential, deterministic. The tree already has a binary-heap flood in `cartalith-erosion/src/construction.rs` (grep hit; confirm it is Priority-Flood) [source: grep]. |
| Priority-Flood+ε (Algorithm 3 of the same paper) | 2014 | Adds a tiny increment so every cell has a strictly downhill path: no flats [source: search]. | Avoids a second pass | **Changes the heights** (by epsilons) and thereby ties and flow directions: **not** identical to plain fill + separate flat handling [inference] | Same. §7p only if used to *replace* today's flat rule. |
| Barnes, Lehman, Mulla, "An efficient assignment of drainage direction over flat surfaces in raster DEMs", *Comput. Geosci.* 62, 128-135 | 2014 | O(n) assignment of convergent flow over flats (towards the edge of the flat and away from higher ground) [source: RichDEM docs via search]. | Linear time | Deterministic rule; differs from other flat rules | Implemented as `ResolveFlats` in RichDEM [source]. Use as the model if the tree's flat rule is slow, but it produces *different* drainage than any other rule [inference]. |
| Barnes, "Parallel Priority-Flood depression filling for trillion cell DEMs on desktops or clusters", *Comput. Geosci.* 96. <https://arxiv.org/abs/1606.06204> | 2016 | Tile the DEM, run Priority-Flood per tile in parallel, then combine **tile offsets** with a fixed number of memory/communication events per tile [source: search]. | Linear scaling with processors; trillion-cell scale [source: search] | **Exact**, matches serial Priority-Flood [inference: stated aim of the paper, not re-read] | Code <https://github.com/r-barnes/Barnes2016-ParallelPriorityFlood> (licence not retrieved). **The best fit for the plan's two-level tiled pipeline**: per-tile local fill, then a small global solve on tile edges. |
| Barnes, Callaghan, Wickert, "Computing water flow through complex landscapes, Part 2: Finding hierarchies in depressions and morphological segmentations", *Earth Surf. Dynam.* 8, 431. <https://www.earth-surf-dynam.net/8/431/2020/> | 2020 | The **depression hierarchy**: a tree of depressions with spill elevations and volumes; use to fill or breach selectively or to speed up dynamic hydrology [source: search]. Companion Part 3, "Fill-Spill-Merge" (2021): **recalled, not retrieved** [inference]. | Large for repeated rain/lake queries [inference] | Exact for the quantities it stores [inference] | Reference code in RichDEM/EarthWorkHydro (licences not retrieved). Directly answers the plan's "fill coarse, hand spill levels down": a coarse hierarchy gives each fine tile its spill level. |
| RichDEM <https://github.com/r-barnes/richdem> | maintained (2 564 commits; CI badge points to the retired travis-ci.org) [source: repo page] | Header-only C++ with Priority-Flood fill/breach, flat resolution, D8/D∞ accumulation, slopes and curvatures [source] | "Days to hours" is its README's own claim, no numbers [source] | Algorithms exact as published | **GPL-3.0** [source]. Use as a **test oracle** (build it outside the app, compare outputs on exported DEMs). Do not link. |
| Jain et al., FastFlow depression routing (section 1) | 2024 | GPU MST-based routing: 34x-52x over a parallel CPU method at 1024² [source: PDF text]. | See above | Routes water out of depressions by the minimum-cost tree; this is a **different pour-point choice** from Priority-Flood when saddles tie, so it can differ at flats [inference]; the paper breaks ties lexicographically by elevation then basin id [source: PDF text] | GPU. Gain is largest when depressions are many (an un-eroded raw heightmap); smaller after a fill has run [inference]. |
| Barnes, Callaghan, Wickert and others, GPU flow accumulation: Barnes 2019 and Schott 2023 | 2019/2023 | Earlier GPU methods propagate along paths: O(path length) iterations [source: FastFlow PDF] | Superseded by rake-compress | Float order-dependent | n/a |

### Best candidate for Cartalith: depression filling and routing

- **Candidate:** (a) keep the exact Priority-Flood fill and add the **parallel tiled variant** (per-tile fill plus a global solve on tile borders) for 16k/32k; (b) build a **depression hierarchy** on the coarse world and pass spill levels down to fine tiles.
- **Expected gain:** (a) scales with cores, near-linear by the paper's design [source: search]; (b) removes the per-fine-cell fill, "several x" in the plan's estimate [source: plan], not measured.
- **Measure first:** the time share of fill plus flat handling in the generation profile at 4k/8k/16k; a diff of filled elevations against RichDEM's Priority-Flood on 3 worlds (it must be zero).
- **World identity:** the fill **surface is unique, so (a) is exact** (bit-identical, since fill only copies heights and takes maxima, no arithmetic) [inference]. Flat flow directions and lake spill choices must be reproduced rule for rule; (b) is exact only if the coarse hierarchy reproduces today's lake set, which is "same lakes" in the plan and must be tested. ε-flood and any new flat rule are **§7p**.

---

## 3. Climate and orographic precipitation at low cost

| Source | Year | Claim | Cost | Fidelity | Licence | Fit |
|---|---|---|---|---|---|---|
| Smith & Barstad, "A linear theory of orographic precipitation", *J. Atmos. Sci.* 61 (2004) | 2004 | Steady-state linearised airflow, condensed-water advection and downslope evaporation, solved with **Fourier transforms**; precipitation is the terrain FFT multiplied by a wavenumber-dependent transfer function [source: search of the Python package docs]. | O(n log n) per field, independent of wind distance travelled [inference] | Linear theory (no blocking, no convection); the paper compared it with a nonlinear model, **not re-read** [inference] | Paper | CPU `rustfft` in f64 is deterministic on a given build (fixed plan, no threads) [inference]. A GPU FFT would drift between devices, so keep it on the CPU at coarse resolution. |
| `orographic_precipitation` (Python) <https://github.com/fastscape-lem/orographic-precipitation> | maintained | Implements the model: elevation array plus wind speed/direction, conversion and fallout times, moist stability; pads boundaries before the FFT [source: repo/PyPI] | n/a | Reference | **MIT** [source: repo page] | An **oracle** for a Rust port; the algorithm is a page of code. |
| QGIS/R ports | - | Exist [source: search] | "Slow with insufficient RAM" for large rasters [source: QGIS plugin note] | - | - | Informational. |
| Cheap moisture advection in games/procedural generation | - | **Not found.** The searches returned no verified source; the usual game approach (a wind-direction sweep that loses moisture on uphill cells and keeps the rest) is **[inference]**, not cited. | O(n) per wind direction | Rain-shadow qualitatively, no conservation | - | Deterministic (integer/fixed order sweep) but whatever the engine already does is this. |

### Best candidate for Cartalith: climate

- **Candidate:** the plan's "coarse climate, interpolated" (about 64x fewer cells at 16k) stays the cheapest and keeps today's model. Smith-Barstad is the one literature-backed *replacement*: a linear FFT precipitation field at coarse resolution.
- **Expected gain:** replacing a full-resolution sweep by an FFT on the coarse grid is O(n log n) on 64x fewer cells; magnitude vs today's code unmeasured [inference].
- **Measure first:** share of generation time spent in climate at 16k; correlation of the coarse-then-interpolated field with the full-resolution field on 3 worlds (the plan expects "practically same").
- **World identity:** coarse-then-interpolated is **not bit-identical** (practically same) and so needs §7p unless the diff is below an agreed bar; Smith-Barstad is a **new model, §7p**.

---

## 4. Terrain amplification and detail synthesis

| Source | Year | Claim | Cost | Fidelity evidence | Licence | Fit |
|---|---|---|---|---|---|---|
| Guérin, Digne, Galin, Peytavie, "Sparse representation of terrains for procedural modeling", *CGF* 35(2), 177-187. <https://perso.liris.cnrs.fr/eguerin/download/eg2016.pdf> | 2016 | A dictionary of landform patches blended through a **Sparse Construction Tree**; learned automatically from real DEMs [source: search]. | Not retrieved | Learned from real terrain; qualitative | Paper | Deterministic once the dictionary is fixed (CPU, fixed order); dictionary learning is offline. A **new look, §7p**; no evidence that it follows the app's own drainage (the failure ZR-0 measured for noise) [inference]. |
| Guérin, Peytavie, Galin et al., "Interactive example-based terrain authoring with conditional GANs", *ACM TOG* 36(6), art. 228. <https://doi.org/10.1145/3130800.3130804> | 2017 | cGANs trained on real terrain and sketches, as synthesizers for ridge/river/amplification tasks [source: search]. | Interactive inference; no number retrieved | Qualitative; learned | Paper | Network inference in float: not bit-identical across devices; **cannot be the generation path** [inference]. |
| Goslin, "Terrain Diffusion", arXiv 2512.08309 (v3). <https://arxiv.org/html/2512.08309v3>; code <https://github.com/xandergos/terrain-diffusion> | 2025 | Hierarchical diffusion (coarse continental model, finer ridge/valley models) designed to replace Perlin noise: **seed-consistent, order-invariant, constant-time random access** (InfiniteDiffusion) [source: PDF text]. | 0.66 s per adjacent 512² tile and 1.72 s to the first, on an RTX 3090 Ti, about 2.2 GB VRAM in fp32; 0.26 s for 256² [source: PDF text]. The paper itself says noise functions are much faster [source]. | Qualitative plus statistics in the paper (not read). Determinism holds when conditioning is itself seed-consistent, **assuming one machine** [source]. | CC BY 4.0 paper; repo licence not retrieved | **Slower than the current tile build by two orders of magnitude** (60 ms colour tile vs 660 ms) on a big desktop card, impossible on a phone [inference]. Cross-device bit identity is not claimed. A research option for a learned upsampler (the plan's second "larger option"), **§7p**, not for now. |
| Other diffusion/SR terrain work: EarthGen (arXiv 2409.01491, imagery), Mars DEM inpainting (arXiv 2510.14765), ISPRS 2026 Mars DTM diffusion | 2024-2026 | Terrain imagery or inpainting/SR of heightmaps [source: search]. | - | **No source found comparing any of them with simulated erosion on landform metrics.** | - | Not usable without a measured comparison. |
| Quilez, "More noise: derivatives" <https://iquilezles.org/articles/morenoise/> (fetch failed; relied on a search summary) | 2008 onward | Sum the analytic derivatives of each fBm octave and damp later octaves by the accumulated slope; this produces erosion-like terrain at the cost of normal fBm [source: search summary]. | One noise evaluation with derivatives per octave | Looks eroded; not drainage-connected [inference] | Article/code free to implement | Pure arithmetic: deterministic on CPU, display-grade on GPU. |
| **Johansen, "Advanced terrain erosion filter"** (Rune Skovbo Johansen), builds on Clay John 2018 (Shadertoy) and Felix Westin 2023; blog <https://blog.runevision.com/2026/03/>, gist <https://gist.github.com/runevision/c867c458f06284ad4c0f8aa3b974bc59>; Bevy WGSL port <https://docs.rs/crate/bevy_erosion_filter/latest> | 2025-2026 | **Evaluated per point with no simulation**, so "fast, GPU-friendly, easy to generate in chunks"; gully stripes aligned to the terrain gradient inside random-pivot cells, blended; altitude-based fading keeps peaks and valleys crisp; adds a ridge map so drainage lines can be drawn [source: write-up]. | Author states it is an approximation; "analytical derivatives are not very accurate"; drainage lines sometimes stop part way down a slope; the fade target by altitude "is not robust in general" [source: write-up]. **No measured comparison with simulated erosion found.** | **MPL-2.0** [source: write-up] (file-level copyleft: modifications to those files stay open) | The closest thing to the plan's "erosion-like analytic noise". Chunk-independent by construction (stateless), so tile seams are exact in principle [inference]. Port to Rust CPU for any value that is stored; **WGSL for display only**, since floats differ per device. |
| Johansen (Bevy) port `bevy_erosion_filter` | 2026 | Plain WGSL plus a CPU module `cpu.rs` [source: docs.rs listing] | - | Same | See crate | A usable reference for WGSL structure. |

### Best candidate for Cartalith: detail synthesis

- **Candidate:** the **slope-damped derivative fBm / gully filter** (Quilez, Johansen) for the deepest LOD levels only, as the plan suggests; it is stateless, chunk-exact and cheap. The ZR-0 finding that noise follows terrain but cannot create landforms still holds: use it **after** real landforms exist at 16k/32k and the cut-out regeneration, not instead of them [source: plan].
- **Expected gain:** the plan estimates 10-100x cheaper than erosion at the deepest levels [source: plan]; no timing of the filter on this project's data exists.
- **Measure first:** ZR-0's own `bind_r` and gully-spacing metrics on the filter's output at x1/x4/x16/x64 against the native-map target, and its cost per 256² tile on CPU and in a wgpu shader on the 6T.
- **World identity:** as a **render-only** layer on top of the stored heights it changes no saved data (like Ruling FG). If the output is stored, written into Inspect heights or used by anything generation-time, it is **§7p**.

---

## 5. Tiled, out-of-core and LOD terrain

| Source | Year | Claim | Fit |
|---|---|---|---|
| Losasso & Hoppe, "Geometry clipmaps: terrain rendering using nested regular grids", *ACM TOG* 23(3). <https://hhoppe.com/proj/geomclipmap/> | 2004 | Nested regular grids centred on the viewer, incrementally refilled; a 40 GB US height map [source: project page via search]. GPU Gems 2 (Asirvatham & Hoppe 2005): 20 billion samples held in **355 MB**, about 90 fps [source: search]. | The classic answer to "only the viewed region is resident". The project's CDLOD-style plan (3D_VIEWPORT_SCOPE) already supersedes it for 3D; the compressed-in-memory pyramid is the part that matters here. |
| Strugar, "Continuous distance-dependent level of detail for rendering heightmaps", *JGT* 14(4), 57-74. <https://github.com/fstrugar/CDLOD> (repo named in search results) | 2009 | One LOD function from 3D distance, vertices **morph** between levels; quadtree of nodes [source: search]. | Already the 3D design. Morphing is the no-popping answer the plan wants. |
| GDAL overviews (`gdaladdo`), COG | - | **Not retrieved**: the search returned nothing on `gdaladdo`. Recalled: overviews are successively halved copies of the raster inside or beside the file, resampled (nearest, average, Lanczos) [inference]. | The same idea as the LOD pyramid; the useful lesson is a **tiled, internally compressed, overview-bearing** file (COG) [inference]. |
| Cesium quantized-mesh-1.0 <https://github.com/CesiumGS/quantized-mesh> | - | Quadtree pyramid of meshes; positions quantised to **15 bits** (0-32767) per tile; 88-byte header with height range and bounding sphere; gzip on the wire; optional oct-encoded normals and water mask [source: spec via search]. | Per-tile height range plus 15-bit quantisation is the same as the plan's "16-bit height with per-tile range" [source: plan]. |
| Mapbox Terrain-RGB / Mapzen Terrarium | - | Terrain-RGB: `h = -10000 + (R*65536 + G*256 + B) * 0.1` m; Terrarium: `h = (R*256 + G + B/256) - 32768` m [source: terrain-codec docs and nst-guide repo]. | Resolution 0.1 m and 1/256 m respectively: 24 bits, **lossy only if recompressed as PNG/WebP**; PNG deflate of the 3 bytes is the format's own cost [inference]. Not better than 16-bit plus a range. |
| Terrain Diffusion's random-access idea | 2025 | See section 4 | Relevant only as the property "any tile in any order gives the same result", which the plan already requires. |

### Best candidate for Cartalith: tiling and LOD

- **Candidate:** nothing new to adopt; the plan's tile/halo design with exact seams (ZR-0) already follows these. The one thing worth taking is the **COG/clipmap lesson**: store the pyramid as tiled, individually-decodable, compressed blocks with the per-tile range in a header, so a tile read touches only its bytes.
- **Expected gain:** I/O proportional to what is viewed instead of the file; cannot be estimated without the profile.
- **Measure first:** read time of a single tile from the current save versus a tile-indexed file at 16k.
- **World identity:** storage/layout only: identical.

---

## 6. Storage and compression

| Source | Year | Claim | Speed/size | Exact? | Licence / fit |
|---|---|---|---|---|---|
| Laplacian / residual pyramid (Burt & Adelson 1983, **recalled, not retrieved**) | 1983 | Store each level as a difference against the upsampled coarser level; total about 1.33x the finest level [source: plan §7; the 1.33 figure is the pyramid sum of 1 + 1/4 + ...]. | Plan estimates 0.5-1 B/texel vs ZR-0's measured 1.97 B [source: plan, **to be measured**] | Lossless **only if the upsample is a fixed integer/fixed-point filter**; a float upsample must be reproduced exactly at load, or the residual no longer adds back to the stored height [inference] | Fixed 2x bilinear or Catmull-Rom with integer weights makes the round-trip exact and device-independent [inference]. |
| LERC (Esri), <https://github.com/Esri/lerc>; Rust crate `lerc-rs` (seen on docs.rs, maturity unchecked) | maintained (2015-2022 copyright) | Raster codec with a user-chosen **maximum error per pixel**; error 0 gives lossless; for elevation, scientific data [source: README via search]. Esri claims better lossless ratios than LZW and deflate [source: ArcGIS blog via search]. | Larger error gives stronger compression [source]. No figure on this data. | Lossless at 0; **lossy mode guarantees a bound, not equality**, so a saved height must use 0 or a bound smaller than the sim's own quantum [inference] | **Apache-2.0**; the repo notes it implements US patent 9,002,126, licensed under Apache-2.0 [source: Debian NOTICE]. A good first test for height tiles. |
| zstd vs gzip, <https://github.com/facebook/zstd>, lzbench | 2016+ | At level 1, one blog citing zstd's benchmark: ratio 2.896 at 510 MB/s compress, 1 550 MB/s decompress vs zlib-1 at 2.743, 105 MB/s, 390 MB/s: about **5x compress, 4x decompress** [source: independent blog, unverified against official lzbench]. A 2024 Depot test found a slight ratio edge at default levels and that higher levels cost much time for modest gain [source]. | About 5x faster encode per the plan [source: plan]; to be measured on height/flow tiles | Lossless: **identical data** | zstd is BSD/GPLv2 dual (**recalled**) and the Rust `zstd` crate wraps it [inference]. Trivial swap; mind a **frozen dictionary and level**: output bytes depend on version and level, so never compare compressed bytes, only decoded data [inference]. Predictors help more than level: byte-plane split (shuffle) of 16-bit heights plus delta before zstd [inference]. |
| Skip ocean/uniform tiles | - | The plan: about 70% of a planet is ocean, about 3x less [source: plan]. | Cheapest storage win | Identical if "uniform" means exactly equal | Store a 1-bit flag and the constant. |
| Basis Universal / KTX2 / ASTC / BC7 <https://github.com/BinomialLLC/basis_universal> | maintained, copyright to 2026 | ETC1S (about 0.3-3 bpp), UASTC LDR 4x4 (8 bpp), XUASTC LDR 4x4-12x12 (0.3-5.7 bpp, supercompressed ASTC), XUBC7 (about 1.15-2.5 bpp lossy sweet spot); transcodes to ASTC, BC1-5, BC7, ETC; reads/writes KTX2 [source: README]. | The README gives no timings [source] | **Lossy**; fine for display colour tiles only | **Apache-2.0** [source]. Encoding is offline-class (slow); transcoding is fast. For tiles generated on demand, **encode cost likely exceeds the 60-189 ms redraw it replaces** [inference], so cache only if redraw is the bottleneck; or write plain ASTC (hardware format) on phones and BC7 on PC and skip the supercompression. |
| meshoptimizer <https://github.com/zeux/meshoptimizer> | maintained | Lossless vertex codec (about 2-4x on quantised data, decode 3-6 GB/s), index codec (about 1 byte/triangle), simplification, float quantisation; glTF `EXT_/KHR_meshopt_compression`; Rust crate `meshopt` [source: README]. | Decode speed is the point | Lossless codecs; simplification is lossy | **MIT** [source]. Relevant only if the 3D mesh path stores meshes; a heightmap-driven CDLOD does not need it (the mesh is a regular grid) [inference]. Its "reduce mantissa bits before compressing" idea applies to float height tiles [inference]. |
| Cesium quantized-mesh | - | See section 5 | - | 15-bit quantisation lossy | Spec only |

### Best candidate for Cartalith: storage

- **Candidate:** **zstd with byte-plane shuffle and delta** for heights and flow, plus ocean/uniform-tile skipping; then evaluate **LERC at zero error** and the **residual pyramid with an integer upsample** against that baseline.
- **Expected gain:** zstd about 5x on encode [source: blog, unverified]; ocean skip about 3x on a typical world [source: plan]; the residual pyramid 0.5-1 B/texel vs 1.97 [source: plan, unmeasured].
- **Measure first:** on 10 saved tiles from a 4k world: bytes and encode/decode ms for gzip-6, zstd-1/3/9, zstd+shuffle+delta, LERC-0, and residual+zstd; plus the 16k extrapolation.
- **World identity:** all of these are lossless and keep the world identical **provided** the decoded data is compared, not the bytes. A 16-bit store with a per-tile range is lossy relative to the f32 generation field and is **only** safe for display data, not for anything the simulation reads back [inference].

---

## 7. GPU determinism

What the specification actually says (read from `gpuweb/gpuweb`, `wgsl/index.bs`, this session) [source]:

- **Fusion is allowed**: "An implementation may fuse operations if the transformed expression is at least as accurate as the original formulation", with the example of a fused multiply-add being more accurate than a multiply then an add. So `a*b + c` may or may not be an FMA, **per compiler and per device**. WGSL has no per-expression "no contraction" switch.
- **Reassociation is allowed**: "An implementation may reassociate operations." (`(a+b)+c` may become `a+(b+c)`.)
- **Accuracy is a bound, not a value**: division is 2.5 ULP (f32), `exp(x)` and `exp2(x)` are `3 + 2*|x|` ULP, `inverseSqrt` is 2 ULP, `atan(x)` up to 4096 ULP in f32 (the table also lists many others). Two conformant GPUs may differ by up to that bound, and the same GPU may differ across driver versions.
- **The `invariant` attribute** only guarantees the same result across different *pipelines on one device* for a vertex `position` output; it is not a cross-device guarantee.
- **Integers are exact**: `i32` uses two's complement with defined overflow wrap; `u32` arithmetic is also fully specified (the spec says i32 has defined overflow behaviour; I recall the same for u32) [source: spec line on i32; inference for u32].
- **Not in WGSL (an inference from the spec's type list, not verified):** no 64-bit integers or floats in core WGSL; use u32 pairs for wide accumulators.

Outside the specification [source: search, secondary]: reductions and floating-point addition order, different vendor math libraries, FMA contraction and hardware rounding are the standard causes of NVIDIA-versus-AMD mismatches; a 2026 CppCon talk (listing only) proposes reproducing computation by controlling contraction, denormals and math-library approximations.

**How projects get cross-device reproducibility [inference except where tagged]:**

1. **Integer or fixed-point kernels.** Add, subtract, multiply, shift, compare on `u32/i32` are bit-exact on every conformant device. This is what the project's own EZ results show: plate assignment, stress and blur with a CPU-uploaded normaliser were bit-identical on PC, 6T and Pad, while float warp/heterogeneity/weather were not [source: plan §10a].
2. **Exact-rounding operations only.** IEEE `+ - * /` and `sqrt` are correctly rounded on CPUs, but WGSL allows 2.5 ULP for division and does not promise correctly-rounded `sqrt`, so even these are not safe. Use multiplication by a CPU-computed reciprocal and integer shifts.
3. **Polynomials instead of libm.** Implement `exp`, `log`, `pow` as fixed-degree polynomials in a **fixed evaluation order** using only `+ - *`, and prevent contraction by making products go through a value that cannot be fused (for example by splitting with `bitcast` round trips, or by evaluating in integers) [inference]. Even then, an implementation is permitted to fuse when "at least as accurate", so a pure-float path is **not guaranteed** in WGSL; integer arithmetic is the only reliable guard.
4. **Compute on the CPU, upload the result.** The project's blur-with-a-CPU-normaliser is the pattern.
5. **Verify, don't assume:** a CI test that runs each kernel on the RX 7800 XT, the iGPU, the 6T and the Pad and compares **bytes** against the CPU, as the plan's §11 step 1 already says.
6. **Where determinism is not needed** (display colour, shading, the analytic detail filter): floats are free [source: plan].

### Best candidate for Cartalith: determinism

- **Candidate:** the **integer accumulation** of FastFlow (section 1) and **integer fixed-point sweeps** for any generation kernel moved to the GPU; keep every transcendental (climate `exp`, warp) on the CPU.
- **Expected gain:** keeps GPU speed-ups without changing the world.
- **Measure first:** the byte-compare harness against the CPU per kernel per device, before any timing; kernel timing second.
- **World identity:** integer kernels that reproduce the CPU's integer result are exact. Any float-in-WGSL kernel on the generation path risks a different world per device (**needs §7p and a device-independence proof, or stays on the CPU**).

---

## Ranking: top candidates by gain per effort

| # | Candidate | Effort | Gain | World |
|---|---|---|---|---|
| 1 | **zstd (level 1-3) with byte-plane shuffle and delta, plus ocean/uniform-tile skip** | hours | encode about 5x; storage about 3x on typical worlds [source: blog, plan, both unmeasured] | identical |
| 2 | **GPU colour/shading tiles** (display only; floats allowed) | days | 10x+ per the plan: 60 ms PC, 189 ms 6T [source: plan] | identical for saved data (display) |
| 3 | **Exact tiled Priority-Flood** (per-tile fill, then a border solve; confirm the current fill is already Priority-Flood) | days | near-linear in cores at 16k/32k [source: paper aim] | identical (fill surface is unique) |
| 4 | **Johansen/Quilez analytic erosion detail** for the deepest LOD levels, display-only | days | 10-100x vs simulating those levels [source: plan estimate]; unmeasured quality | render-only; §7p if stored |
| 5 | **FastFlow-style rake-compress accumulation on integer weights** in wgpu | 1-2 weeks | 5x over earlier GPU flow routing at 1024² [source: paper]; unknown vs the current CPU | identical if the same recipients; else §7p |

Not ranked: Smith-Barstad (a new model, **§7p**), diffusion terrain (about 100x slower than the current tile build on a desktop GPU [source+inference]), and the learned upsampler (the plan's own "longer project").

## Findings that contradict or correct the plan

- **"Priority-Flood"-style fill is *exact* only in its elevations.** Flat flow directions and spill-point ties are rules, and different rules give different rivers. Do not swap the flat rule without a diff [inference].
- **fastscapelib and RichDEM are GPL-3.0** [source]. They are oracles to test against, not code to link.
- **FastFlow's "exact" is about the routing it implements** (4-connected, a specific MST tie-break), not about equivalence with an 8-connected engine [source: PDF text].
- **The GPU "flow accumulation (parallel prefix over a flow tree)" in plan §10a is FastFlow's rake-compress**, and it was published in 2024 with code (Inria GitLab) [source].
- **WGSL cannot forbid FMA**: fusion is explicitly permitted whenever the result is at least as accurate [source: spec], so a "float but careful" generation kernel is not a safe plan; integer kernels are.

## Not verified (state plainly)

Schott 2023 timings and licence (HAL blocked); Tzathas 2024 and Cordonnier 2016 timings; Mei 2007/Štava 2008 figures; Guérin 2016/2017 figures; Barnes 2016's statement of exactness (recalled); Barnes 2020/2021 part and title for Fill-Spill-Merge; GDAL overviews; Quilez's page itself (fetch failed); zstd's licence and the official lzbench numbers; the `lerc-rs` crate's maturity; Badlands' licence; any published comparison of analytic erosion filters with simulated erosion (none found).

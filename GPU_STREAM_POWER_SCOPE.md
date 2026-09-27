# GPU stream-power erosion — scope

**Stream-power erosion on the GPU needs a different algorithm; a direct port
would not work.**

This document **defines** milestones SP-G0 to SP-G6. It does not track them:
status lives only in `cartalith-native/docs/STATUS.md`. It was written on
2026-09-29 for owner **Ruling AZ** (2026-09-28, `LARGE_ITEM_RULINGS.md`: *"GPU
stream-power erosion (needs a scope first)"*, one of the three research tracks
to start). The backlog row is in `OUTSTANDING_WORK.md` §2.6, *"Erosion's per-cell
parts (thermal, stream-power) on GPU"*.

Each owner question in the last section carries the default the milestones were
written against. A ruling on any of them is recorded in `LARGE_ITEM_RULINGS.md`,
not here.

**Every symbol cited below was opened for this document on 2026-09-29.** Symbols
are named instead of line numbers because line numbers in this tree drift within
a day. **Every timing below is cited, not measured.** No timing was measured for
this document. Each figure names its source, and SP-G0 exists to replace
the single-sample ones.

---

## 1. Why this exists, and what the backlog row already found

The GPU layer work (`GPU_LAYER_INTEGRATION_SCOPE.md`, milestone **E**) set out to
move *"erosion's per-cell parts"* to the GPU. Thermal erosion was built: commit
`08020ee`, `gpu_thermal.wgsl` and `cartalith_gpu::thermal_grid_gpu_with`. Stream
power was not. The row's instrumented phase timings of
`stream_power_kernel_bounded`, taken inside `generate_terrain` with **one sample per
size** and the instrumentation reverted, gave these shares:

| Phase | 1024² | 2048² | Parallel today? |
|---|---|---|---|
| Per-cell: uplift normalise, receivers, `Cc` | 5.9 of 330 ms (1.8%) | 18.4 of 1 712 ms (1.1%) | mostly — see §2 |
| Priority-flood fill | 25% | 20% | no |
| MFD drainage-area accumulation | 35% | 27% | no |
| Implicit Braun–Willett solve (all iterations) | 38% | 51% | no |

*Source: `OUTSTANDING_WORK.md` §2.6, the row named above; also quoted in
`GPU_LAYER_INTEGRATION_SCOPE.md` §E. These are single samples. They are not a median.*

The row concluded that porting only the per-cell slice would save at most about
2%, and would then pay for a field upload and readback. A GPU stream-power is an
algorithm change, not a port. Checked at the symbols (§2), that conclusion
holds, with three corrections:

- **"All three per-cell phases are already `rayon`-parallel" is not quite
  true.** The uplift normalise begins with a **serial** `ss` running sum of
  `|stress|`. It is kept serial on purpose: a parallel float sum could round
  differently and flip the `ss < 1e-3` branch. That decision is recorded in
  `CPU_MULTITHREADING_SCOPE.md`. The loop is O(n) with trivial work per cell, so the
  1–2% figure is not affected.
- **The accumulation runs in *reverse* fill order**, highest first. The row says
  "in fill order". The meaning is the same, because donors must be finished before
  their receivers, but the phrase reads the wrong way round.
- **The row does not name the deposition sub-pass.** It runs inside every
  iteration whenever `deposit > 0`, which includes the default `stream.deposit`
  of `0.3` (`cartalith_engine::WorldParams::defaults`). It is serial for the
  same reason as the solve, running in the opposite direction. The row's
  "implicit solve 38–51%" figure almost certainly includes it, because both run
  in the same `for _ in 0..p.iters` loop. SP-G0 splits the two.

## 2. The current kernel, read at its symbols

`cartalith_erosion::stream_power_kernel` (`crates/cartalith-erosion/src/lib.rs`)
delegates to `stream_power_kernel_bounded` with `pinned = None` and
`area_seed = None`. It is a port of the reference's `streamPowerKernel()`, v2.10
lines 4082–4194. It combines three methods:

- implicit stream-power incision (Braun & Willett 2013), run over a
  priority-flood-filled surface;
- multiple-flow-direction drainage area (Freeman 1991);
- an optional sediment-deposition pass.

Golden parity is `tests/golden_parity_streampower.rs`, which uses two small
fixtures of 12×9 and 11×8.

**Callers.** There are four:

- `generate_terrain`'s carve block, gated by `p.carve_rivers`, which defaults to
  `true`. It runs `light_iters = max(4, round(stream.iters × 0.6))`, which is **9**
  at the default `iters: 15`.
- the `evolveCoupled` loop. It runs once per `passes.evolve_cycles` (default 0,
  panel maximum 12), with a `refresh_climate` between cycles.
- the `depositSediment` block, gated by `passes.sediment_fill`, which defaults to
  `false`. It runs the full `stream.iters`.
- `cartalith_erosion::tile::tile_erode`, which is EF-3's deep-zoom tile re-run.
  It is the **only** caller that passes `pinned` or `area_seed`, and it
  asserts `!p.world`.

`erode_op`, the Erode button, does **not** call stream power. It calls droplet,
thermal and rebound only.

### 2.1 Phases

| # | Phase | Reads | Writes | Shape |
|---|---|---|---|---|
| 1 | **Seed** | `filled` (a copy of `fld`) | `done`, heap | Pushes the top and bottom rows, and the left and right columns **only when `!wrap`** |
| 2 | **Priority-flood fill** | `filled`, `done`, heap | `filled` (raised to `parent + 1e-5` when `≤ parent`), `order[]` (pop sequence), `rdist` (D8 step length to the discovering parent), `done` | **Serial by construction** — see below |
| 3 | **Uplift normalise** | `stress`, `fld` | `u` | Serial `ss` sum. If `ss < 1e-3`, `u = max(fld − 0.3, 0)` in parallel. Parallel `u_max` reduction (exact, because max is associative), then a parallel scale by `p.uplift` |
| 4 | **Receivers** | frozen `filled` | `rcv[i]` (strict steepest D8 descent, first maximum wins, `-1` if nothing is strictly lower), `rdist[i]` | Per cell, `rayon` |
| 5 | **MFD area** | `filled`, `order`, seed `area` (`1.0`, or `area_seed`) | `area` — each cell spreads `area[i]·s^1.1/Σs^1.1` to every strictly lower neighbour | **Serial**: a scatter in reverse `order` |
| 6 | **`Cc`** | `rcv`, `rdist`, `resist`, `rain`, `area` | `cc: Vec<f64>`, `K·(1−res)·(1+2·ck·rain)·dt·A^0.5 / L` | Per cell, `rayon`, in **`f64`** |
| 7a | **Implicit incision** (× `iters`) | `fld[i]` (old), `u[i]`, `cc[i]`, `fld[rcv[i]]` (**already updated this pass**) | `fld[i] = (fld[i] + dt·u[i] + c·fld[r]) / (1 + c)` | **Serial**: runs in `order`, receivers before donors. Pinned cells and roots are skipped |
| 7b | **Deposition** (× `iters`, only if `deposit > 0`) | `old_h` (a copy of `fld` taken before 7a), `fld`, `u`, `area`, `rdist`, `sea` | `sed` (initial eroded column), then per cell a capacity clamp that deposits into `fld[i]` and a below-sea dump; `sed[r] += sed[i]` | **Serial**: runs in reverse `order`, a single-receiver carry |
| 8 | **Clamp** | `fld` | `fld ∈ [0,1]`, skipping pinned cells | Per cell, `rayon` |

Phases 2, 4, 5 and 6 run **once per call**. Only 7a and 7b repeat `iters` times.
Receivers are **never recomputed** between iterations. They come from the filled
surface as it was when the kernel started.

### 2.2 Why phases 2, 5, 7a and 7b are serial, and what their order really is

**The fill order is a topological order, and the arithmetic does not depend on
it.** This is the most important fact in this document. The reasoning:

- `MinHeap` (the field-for-field port of the reference heap) only pushes a cell
  at a priority **strictly above** the one just popped. A raised cell becomes
  `f32(parent + 1e-5)`, which is strictly greater than the parent for values in
  `[0, 1]`. An unraised cell was already greater.
- So the pops are non-decreasing in `filled`.
- Every receiver (phase 4) and every MFD donor target (phase 5) is **strictly
  lower** in `filled`, so it is popped earlier.

`order` is therefore one topological sort of both the single-receiver forest
and the MFD DAG. Any other topological sort gives:

- **Phase 7a: bit-identical results.** Each update reads only its own old
  height, its own `u` and `cc`, and its receiver's height *after this pass*. The
  f64 operations are the same. The sequential loop is one schedule of a
  dependency tree, not a fundamental serial chain.
- **Phase 7b: the same results, except the summation order of `sed[r] +=
  sed[i]`** when a receiver has several donors. That can be made reproducible
  by summing donors in their `order` position.
- **Phase 5: the same results, except the summation order of `area[j] += …`**
  across a cell's several upslope donors, each of which is rounded to `f32`. The
  kernel's doc comment names this as the same multi-writer trap as
  `erode_thermal`'s `delta[j]`.

**Phase 2 is the one genuinely order-bound phase.** Which neighbour discovers a
cell first depends on the heap's tie-breaks. That neighbour decides `filled`'s ε
pattern inside a depression, and through it the receivers and flow paths inside
filled lakes. `PROVENANCE.md` says so for priority-flood: *"Equal-priority pop
order decides the fill tie-break and therefore lake shape"*. Its row names
`buildWaterBodies`, and `MinHeap`'s doc comment applies the same rule to this
kernel. That is why `MinHeap` is ported rather than replaced with
`std::collections::BinaryHeap`.
**The ε-free fill level is not order-bound.** It is the spill surface: for each
cell, the lowest possible maximum height along any path to a seeded edge. Any
correct algorithm produces the same value, and it is computed from existing
values by `min` and `max` only, with no arithmetic.

### 2.3 The `world` X-wrap

`p.world` (`wrap`) changes two things, and they are the only two:

1. **Seeding.** With `wrap`, only the top row and the bottom row are seeded. The
   left and right columns are not an edge, so they drain like interior cells. Y
   never wraps.
2. **Every neighbour loop** in phases 2, 4 and 5 wraps x as
   `((nx % w) + w) % w`. With `!wrap`, cells outside the grid are skipped. Phases
   7a and 7b have no neighbour loop, because they follow `rcv`, which already has
   the wrap built in.

A GPU version needs the same modulo in each neighbour loop and the same seeding
rule. `gpu_flow.wgsl` already implements x-wrap for flow direction, as *"one
extra modulo"*, and is the precedent. `tile_erode` refuses `world`, so wrap and
the `pinned`/`area_seed` extras never meet.

### 2.4 Precision the GPU cannot copy

`cc` is `f64` throughout, and 7a/7b do all their arithmetic in `f64` and round
to `f32` when they store. **The GPU has no `f64`.** `cartalith-gpu/src/lib.rs`
records it: naga (wgpu's WGSL front end) implements no `enable f64;`, even though
the adapter reports `SHADER_F64`. The test is
`f64_wgsl_is_not_implemented_by_naga_even_though_the_gpu_feature_exists`. A GPU
stream-power is therefore **f32 all the way through**, and bit parity is not
possible. For the bar this sets, see §4.

## 3. A candidate parallel algorithm for each serial phase

Only citations this author can name with confidence are given. A few are marked
*verify*: the work exists, but its method details or bibliographic details
should be re-read before this document leans on them further.

### 3.1 Implicit incision (7a)

- **(a) Level sweep.** Assign each cell its depth in the receiver forest, where a
  root has depth 0 and a cell's depth is its receiver's depth plus 1. Then
  dispatch one pass per depth, in increasing order. All cells at the same depth
  are independent. This is the parallelisation of Braun & Willett's stack order
  described in **Barnes (2019)**, *"Accelerating a fluvial incision and landscape
  evolution model with parallelism"*, *Geomorphology*. It is **exact**: by
  §2.2 it is bit-identical to today's loop on the CPU in f64. **Cost: one
  dispatch per level**, and the level count is the length of the longest river
  in cells. The count is unknown until SP-G0 measures it, and it could run to
  thousands at 2048².
- **(b) Affine pointer jumping.** For fixed `c`, the update is affine in the
  receiver's new height: `h_i' = α_i + β_i·h_r'`, where `α_i = (h_i + dt·u_i)/(1+c_i)`
  and `β_i = c_i/(1+c_i) ∈ [0,1)`. Composing affine maps along the receiver pointer
  is associative, so **pointer doubling** resolves every cell against its root
  in `⌈log₂ depth⌉` rounds: 22 at most at 2048². This is the technique for linear
  recurrences in Kogge & Stone (1973), *"A parallel algorithm for the efficient
  solution of a general class of recurrence equations"*, *IEEE Trans.
  Computers*. Pointer jumping in general goes back to Wyllie (1979, PhD thesis,
  Cornell), and Hillis & Steele (1986), *"Data parallel algorithms"*, *CACM*,
  is the standard account. **This repository already has the doubling machinery**:
  `gpu_flow.wgsl` (GLI-9) doubles over the same kind of single-receiver forest.
  It is exact in real arithmetic. In f32 the composition rounds differently.
  The conditioning is good, because `|β| < 1` so the products shrink, but the
  error is measured, not assumed. Pinned cells and roots become fixed points
  (`β = 0`, `α = h`).

**Default:** (b), unless SP-G0 shows few enough levels that (a) costs fewer
dispatches than 22 rounds of (b).

### 3.2 Deposition (7b)

This phase is **not affine**. The capacity test, the ceiling clamp and the
below-sea dump are conditionals on the carried `sed`, so pointer jumping does not
apply.

- **Upstream-height level sweep.** Level a cell by the longest path *from* a
  leaf: a leaf is 0, and each other cell is one more than the highest of its
  donors. Process the levels in increasing order, and have each cell **gather**
  its donors' outgoing `sed`, in a fixed donor order, so there are no atomics
  and the result is deterministic. This matches the CPU arithmetic, up to the
  donor summation order. **Cost: one dispatch per level**, the same order of
  count as 3.1(a).
- **Fallback: keep deposition on the CPU.** That costs a readback and an upload
  per iteration. At 2048² a whole-grid transfer is a few milliseconds: thermal's
  full GPU call, including upload, pipeline build and readback, is 5.66 ms at
  2048² per `OUTSTANDING_WORK.md` §2.6. So round-trips are affordable, but the
  CPU deposition loop itself stays serial.

### 3.3 MFD drainage area (5)

This is a DAG, not a forest, so pointer doubling does not apply.

- **(a) DAG level sweep with gather.** Level each cell by its longest upslope
  path. Each cell gathers `area[j]·w_ji` from its up-to-8 upslope neighbours,
  whose weights come from the frozen `filled`. There are no atomics, and it is
  deterministic in f32.
- **(b) Iterated fixed point.** Solve `A = a + WᵀA` by Jacobi or Gauss–Seidel
  sweeps. This is **Richardson, Hill & Perron (2014)**, *"IDA: An implicit,
  parallelizable method for calculating drainage area"*, *Water Resources
  Research*. A plain Jacobi sweep needs roughly one iteration per cell of flow
  path, so it is only competitive with a **warm start**, for example from the
  previous `evolve_cycles` call's area, or with a stronger solver.
- **(c) Single-flow approximation.** Replace MFD with the D8 area that GLI-9's
  pointer doubling already computes. This is cheap, but it **changes the look**:
  MFD's divergent spreading on hillslopes goes away. That is an owner question
  (Q4), not an engineering choice.

Related work: **Qin & Zhan (2012)**, *"Parallelizing flow-accumulation
calculations on graphics processing units"*, *Computers & Geosciences*. It is
already cited by `gpu_flow.wgsl`. Also **Barnes (2017)**, *"Parallel
non-divergent flow accumulation for trillion cell digital elevation models on
desktops or clusters"*, *Environmental Modelling & Software*. Both are D8, not MFD.

**Default:** (a), with the level histogram from SP-G0 deciding whether it pays.

### 3.4 Depression handling (2)

- **Keep the CPU priority-flood.** It stays exact and ordered. Amdahl then caps
  the kernel's speed-up at about 1/0.20–0.25, so **4–5× at most**, from the row's
  single-sample shares.
- **Tile-parallel priority-flood on the CPU.** **Barnes (2016)**, *"Parallel
  priority-flood depression filling for trillion cell digital elevation models
  on desktops or clusters"*, *Computers & Geosciences*. Each tile is filled
  independently, then a small spill graph between tiles is solved. That is
  `rayon` work, not GPU work, and it keeps f32 CPU arithmetic. **It changes the ε
  tie-break pattern at tile seams**, so it is still an algorithm change inside
  filled depressions.
- **Data-parallel filling.** **Planchon & Darboux (2002)**, *"A fast, simple and
  versatile algorithm to fill the depressions of digital elevation models"*,
  *Catena*, is an iterative relaxation that suits the GPU. Its iteration count
  grows with depression size (**Barnes, Lehman & Mulla (2014)**, *"Priority-flood:
  an optimal depression-filling and watershed-labeling algorithm for digital
  elevation models"*, *Computers & Geosciences*, compares them). Once the ε-free
  spill surface is known, the flats need a drainage direction: **Garbrecht &
  Martz (1997)**, *Journal of Hydrology*, or **Barnes, Lehman & Mulla (2014)**,
  *"An efficient assignment of drainage direction over flat surfaces in raster
  digital elevation models"*, *Computers & Geosciences*. Either **replaces** the
  ε gradient that the heap order produces.
- **Basin graph instead of filling.** Route flow over the unfilled surface, find
  each basin's lowest pass, and connect basins through a spanning tree of passes.
  This is **Cordonnier, Braun, Cani, Benes, Galin, Peytavie & Guérin (2016)**,
  *"Large scale terrain generation from tectonic uplift and fluvial erosion"*,
  *Computer Graphics Forum*. A GPU implementation of flow and depression routing
  is **Jain, Kerbl, Gain, Finley & Cordonnier (2024)**, *"FastFlow: GPU
  acceleration of flow and depression routing for landscape simulation"*,
  *Computer Graphics Forum* (*verify* the method details before relying on it).
  A GPU interactive stream-power authoring system is Schott, Paris, Fournier,
  Guérin & Galin (2023), *"Large-scale terrain authoring through interactive
  erosion simulation"*, *ACM Transactions on Graphics* (*verify* which of these
  phases it solves and how). **This is the largest departure.** Flow inside a
  depression goes to the pass instead of down an ε ramp. Spill heights and lake
  extents stay the same, but paths inside lakes differ.

**Default:** keep the CPU fill first. Move it only if SP-G0 or SP-G4 shows it is
what remains (Q3).

## 4. The equivalence bar

The standing rule is `DECISIONS.md` §7a, *"Principled equivalence for
GPU/optimized paths"* (owner decision, 2026-08-16):

- GPU paths need **principled equivalence and visual quality**, not bit parity.
- The **CPU pipeline stays the parity path**, and nothing in this document
  touches its goldens.
- `WorldParams::defaults()` keeps `use_gpu: false`.

Two facts make stream power harder than the stages already on the GPU:

- **It is chaotic at ties.** A last-bit difference in `filled` can flip one
  cell's steepest receiver, and then a whole subtree's area and incision move.
  A per-cell *worst* `|Δ|` can therefore be large at a handful of cells while the
  landscape is equivalent. A tolerance that is a single maximum would be either
  meaningless or unreachable.
- **Its outputs feed rivers, lakes and settlements.** "How close is the number"
  matters less than "does the river network come out the same". GLI-9
  established this with `gpu_flow_downstream_river_network_divergence`.

**The three-way decomposition.** Every comparison runs three legs:

- **CPU-original**: today's kernel.
- **CPU-reformulated**: SP-G1's level-ordered or tile-filled variant, in f64.
- **GPU**.

CPU-original against CPU-reformulated isolates the *algorithm* change.
CPU-reformulated against GPU isolates the *precision* change. A difference that
cannot be assigned to one of the two is a defect.

**What is measured, before any constant is pinned** (the thermal precedent:
measure, then pin with stated headroom — `THERMAL_GPU_TOL = 1e-6` against a
worst element of 2.98e-7):

1. **Elevation:** the distribution of per-cell `|Δ|`. Report the worst, p99.9,
   p99 and mean. Normalise it by the kernel's own signal, which is the per-cell
   `|fld_after − fld_before|` of the CPU run. Report the count of cells whose
   receiver differs.
2. **Drainage network:** run the real `cartalith_hydrology::build_channels` and
   `strahler_from_receivers` over both results. Report the channel-mask cells
   differing, the river-cell count delta, and whether the **maximum Strahler
   order is unchanged**. This is GLI-9's assert shape.
3. **Water bodies:** run `build_water_bodies` on both. Report the lake count and
   the lake-area delta.
4. **Picture:** a windowed render diff against a **CPU-against-CPU control and
   a GPU-against-GPU control**. Until the recompute defect in §7 is fixed, compare
   renders built from the kernel's output fields directly, **not** through the
   shell's post-op recompute path. §7 shows that path repaints tens of thousands
   of pixels with no change to elevation.
5. **Determinism:** GPU against GPU must be **bit-identical**. That means no float
   atomics and gathers in a fixed order. GLI-9's fixed-point `atomic<u32>` scatter
   is acceptable only where the order-independence argument holds.

**Fields and settings:**

- real `generate_terrain` outputs, never synthetic ones only;
- 3 seeds × 512², 1024² and 2048² × `world` false and true;
- `iters` at the default 9 and the panel maximum; `deposit` 0, 0.3 and 1.0;
- `uplift` 0 (the default) and one non-zero value. At 0 the uplift term is inert
  (`EROSION_GEOLOGICAL_TIME_SCOPE.md` §4, *"The default has no uplift"*).

Each tolerance is set from the measured worst with its headroom stated, and each
is mutation-tested per `MISTAKES.md`.

## 5. The precedent to reuse

- **Device.** `init_gpu_device_set()` serves a cached set under Ruling Y, so the
  ~190 ms device handshake is paid once per session, not once per call. That
  figure is cited from `OUTSTANDING_WORK.md` §2.6's thermal record.
  `gpu_allowed_for_grid` is the VRAM-budget gate, and
  `GpuDeviceSet::supports_grid` is the binding-limit gate. A `None` at any step
  runs the untouched CPU kernel.
- **Buffers.** GLI-POOL's `BufferPool` (`cartalith-gpu/src/pool.rs`). Stream
  power needs about 10 grids at once: `fld`, `filled`, `rcv`, `rdist`, `area`, `cc`,
  `u`, `old_h`, `sed` and a level or pointer buffer. At 2048² that is roughly
  160 MB, which `vram_verdict` must see before dispatch.
- **Encoding.** `thermal_grid_gpu_with` ping-pongs every pass in **one encoder
  and one submit**. Stream power's `iters × levels` dispatches must be encoded
  the same way, because a submit per level would pay the per-submit overhead
  thousands of times.
- **Shape.** `gpu_flow.wgsl` provides the direction pass, the pointer doubling,
  the x-wrap and the fixed-point determinism argument.
- **Reporting.** `WorldState::gpu_stages_used` gets a `"stream_power"` entry,
  recorded when the stage succeeds, not when it is attempted.

## 6. Milestones

### SP-G0 · Measurement harness (measurement only; no output change)

This milestone answers the go/no-go question before any shader is written.

- **Factor, don't instrument.** Split `stream_power_kernel_bounded` into private
  phase functions: fill, uplift, receivers, area, `cc`, incise, deposit and
  clamp. The body calls them in the same order with the same arithmetic, so a
  harness can time each one with no instrumentation to revert. Every golden stays
  byte-identical, and a `generate_terrain` output hash at the default is
  unchanged for 3 seeds, `world` false and true.
- **Timings** (`#[ignore]`, release, `--test-threads=1`, harness run alone):
  - the median (min..max) of ≥5 runs per phase, at 512², 1024², 2048² and the
    app's default grid;
  - the kernel's share of a whole `generate_terrain`, and how many times it is
    called at the default and at `evolve_cycles` 12;
  - repeated in a **second process**, with the median landing inside the first
    bracket. Otherwise say that no figure is established.
- **Structure**, the numbers that decide the algorithm:
  - the maximum depth of the receiver forest, and a histogram of cells per level;
  - the longest upslope path in the MFD DAG;
  - the maximum upstream height for 7b;
  - the root count;
  - the fraction of cells in filled depressions (`filled > fld`).
- **Transfer:** measured upload and readback of the ~10 grids at 2048² through
  the pooled path. **Per-dispatch cost:** an empty dispatch chain of N levels in
  one submit.
- **Done means:** every number above is in the backlog row and in `STATUS.md`,
  with its command. A projected GPU kernel time is computed as levels × per-dispatch
  cost + per-level work + transfer, for 3.1(a), 3.1(b) and 3.3(a), and set
  against the CPU phases. A **go/no-go verdict** is stated against Q1's
  threshold. A no-go closes the track at SP-G0, or at SP-G1 if that alone pays.

### SP-G1 · CPU reformulation as oracle (bit-identical, `rayon`)

- **Build:** the level-ordered incision (3.1(a)) and deposition (3.2) on the CPU
  in f64, with deposition's donor sums ordered by `order` position so that they
  are **bit-identical to today's kernel**. This is the "CPU-reformulated" leg of §4
  and the reference every GPU milestone compares against.
- **Done means:**
  - goldens byte-identical, and the `generate_terrain` hash unchanged, as in
    SP-G0;
  - mutation-tested: swap the level order, drop a level;
  - a timing against the serial loop. **If this alone captures most of the win,
    Q2 applies.**

### SP-G2 · GPU incision (7a), with fill, receivers and area on the host

- **Build:** a shader for the solve, 3.1(b) or (a) per SP-G0, over host-supplied
  `rcv`, `cc`, `u` and `fld`, with all `iters` in one submit. Deposition is off
  (`deposit = 0`) in this milestone's comparisons. Pinned cells and `area_seed`
  are **refused**: the tile path stays CPU (Q6).
- **Done means:**
  - §4's metrics 1 and 5 measured at every size and setting, and the incision
    tolerance pinned from the measured worst with its headroom stated;
  - GPU against GPU bit-identical;
  - mutation-tested: the `β` term, the root fixed point, the x-wrap, the round
    count;
  - a timing against SP-G1.

### SP-G3 · GPU deposition (7b)

- **Build:** the upstream-height level gather from 3.2, fused into SP-G2's
  per-iteration encoder. The alternative is the measured CPU fallback, if SP-G0's
  upstream-height count makes the gather lose.
- **Done means:**
  - §4's metrics 1, 2 and 5 at `deposit` 0.3 and 1.0, including the below-sea
    dump. A fixture puts cells under `sea`, because at the default fields the
    branch may never bind; the same fixture lesson came from thermal's clamp;
  - the tolerance pinned from measurement;
  - mutation-tested: the capacity constant `0.005`, the `0.8` dump factor, the
    ceiling clamp, the donor order.

### SP-G4 · GPU MFD drainage area (5)

- **Build:** 3.3(a) as a DAG-level gather, with the `s^1.1` weights computed from
  the frozen `filled` on the device. MFD itself is kept (Q4).
- **Done means:**
  - §4's metric 2 (the network) and metric 3 (water bodies), plus the relative
    `|ΔA|/A` distribution;
  - the maximum Strahler order unchanged;
  - the river-cell count within a bound pinned from measurement;
  - GPU against GPU bit-identical;
  - mutation-tested: the `1.1` exponent, the weight normalisation, the wrap.

### SP-G5 · Depression handling (only if Q3 says so)

- **Build:** per Q3's ruling, either the tile-parallel CPU fill (Barnes 2016), or a
  GPU spill surface with a flats direction, or the basin graph (§3.4). **The ε-free
  spill surface must come out bit-identical to the CPU's.** It is a min/max
  construction, so this is achievable and it is the test.
- **Done means:**
  - spill surface identical on every §4 field;
  - lake count and extents identical;
  - flow paths inside lakes reported as their own metric, meaning changed-receiver
    cells *inside* `filled > fld` against outside, so that the change is
    attributed where it belongs;
  - SP-G0's projection re-run with the fill moved.

### SP-G6 · Wiring, end-to-end measurement, and the picture

- **Build:** a `use_gpu` branch at the three `generate_terrain` call sites, behind
  `gpu_allowed_for_grid` and `supports_grid`, with the untouched CPU kernel on any
  `None`. It is reported as `"stream_power"` in `gpu_stages_used`. `world` is
  supported. `WorldParams::defaults()` keeps `use_gpu: false`, asserted as a
  literal.
- **Done means:**
  - an end-to-end `generate_terrain` median (min..max) with GPU against CPU at
    the default grid, in two processes;
  - §4's metrics 1–4 on the full pipeline output;
  - a windowed probe that runs both paths with CPU-against-CPU and
    GPU-against-GPU controls, whose screenshots the owner can judge;
  - `cargo test --workspace` green, and every `.gd` file touched parse-checked.

**Order:** SP-G0 → SP-G1 → SP-G2 → SP-G3 → SP-G4 → SP-G6, with SP-G5 slotted
before SP-G6 only if Q3 rules for it. A no-go at SP-G0 or SP-G1 stops the chain
there, and the scope stays as the record of why.

## 7. The pre-existing recompute defect, and why it is its own row

The thermal work found this, and did not cause it (`OUTSTANDING_WORK.md` §2.6).
After an `erode_op`, the flow and climate recompute is **not a function of
elevation alone**:

- With elevation bit-identical, two CPU runs with an undo between them differ in
  drainage on about all 110 592 cells, and in precipitation on 19–36k cells.
- A **passes-0 control**, which changes no elevation at all, still repaints
  54 443 px.

**It is a separate defect, and it should be its own backlog row** (filed beside
this one on 2026-09-29):

- It is not stream power's, and not the GPU's, since the CPU against CPU shows it.
- It makes the same world repaint differently depending on history, which is a
  determinism defect in the engine.
- It confounds every visual comparison this scope (and any later GPU stage)
  needs, which is why §4 metric 4 routes around it.

The recompute is `cartalith_engine::staleness::recompute_stale`, which calls
`refresh_climate` (`cartalith-engine/src/lib.rs`).

**A suspicion, not a verified cause.** `refresh_climate` calls
`compute_flow_routed(…, Some(rainfall), …)` first. That routes flow with the
rainfall **already in the world state from before this recompute**, and
`simulate_weather` replaces it on the next line. The output would then depend on
the previous rainfall, and so on history. The reference's own `refreshClimate`
(v2.11 line 5180) routes no flow at all. Whoever takes the row should confirm
or refute this at the symbol before fixing anything. The GPU-against-GPU spread
(3–7k px) may have a second source.

## 8. Owner questions — each with the default the plan assumes

1. **The go/no-go bar.** What saving justifies a second, f32, non-parity
   stream-power path?
   *Default:* proceed past SP-G0 only if the projected GPU kernel is **≥2× faster
   than SP-G1's CPU-parallel version** *and* saves **≥10% of a default-grid
   `generate_terrain`** end to end.
2. **If SP-G1 (CPU, bit-identical) captures most of the win, stop there?**
   *Default:* yes. Take SP-G1, park the GPU milestones, and record the numbers. A
   parity-preserving speed-up beats a principled-equivalence one when their sizes
   are close.
3. **Depression handling.** Keep the ordered CPU priority-flood, and accept
   Amdahl's cap of about 4–5× on the kernel? Or allow a parallel fill that
   changes flow paths inside filled depressions, while spill heights and lake
   extents stay identical?
   *Default:* keep the CPU fill through SP-G4, and decide SP-G5 on SP-G4's
   measured remainder.
4. **MFD or single-flow area on the GPU path?** Single-flow is cheap (GLI-9's
   doubling), but it removes MFD's hillslope spreading.
   *Default:* keep MFD. The GPU path must not change the look by choice, only by
   precision.
5. **Device-dependent worlds.** With `use_gpu` on, a seed's rivers may differ in
   detail from the same seed on the CPU. Flow (GLI-9) and weather (GLI-7) already
   do this within their tolerances, but stream power moves more through ties.
   *Default:* the existing GPU policy: opt-in, CPU as the parity path, and a
   tooltip saying results may differ slightly. SP-G6's screenshots decide
   whether "slightly" is still true.
6. **The EF-3 tile path (`tile_erode`) on the GPU?**
   *Default:* no. Tiles are small, `pinned` and `area_seed` would double the
   shader's cases, and deep-zoom tiles already run off the main thread
   (LOD-D6).

## 9. Out of scope

- Any change to the CPU kernel's output, and any golden re-baseline.
- Recomputing receivers between iterations, which the reference never does.
- The droplet kernel, which has sequential per-droplet state.
- `glacial_kernel` and `velocity_erode_kernel`: other erosion passes, other rows.
- The §7 recompute defect, which is its own row.

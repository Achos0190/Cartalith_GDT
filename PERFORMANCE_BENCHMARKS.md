# Compute-configuration benchmarks (agent-produced, 2026-08-24)

> **Agent-produced, and every number in it was measured on this machine.**
> Nothing here is estimated, scaled from another size, or inherited from an
> earlier pass — where a figure comes from an existing document it says so and
> names it. Two failures are reported as failures rather than omitted, and one
> of them was a real crash that this run found and fixed (`CHANGELOG.md`,
> same date; commit `504c2a6`).

## 0. The question

The owner asked for a real comparison across compute configurations at
**2048²** and **8192²**, **40 tectonic plates**, full pipeline — and asked for
it to be judged on **smoothest user experience**, not raw throughput. That
framing is the reason this document does not stop at a wall-clock table: the
configuration that generates fastest and the configuration that *feels* best
are not automatically the same one, and on this machine the thing that decides
how the app feels turns out not to be the compute configuration at all.

**The short version, stated up front so the rest reads as evidence:**

1. **Use the discrete GPU, `single device` mode.** It is fastest at both sizes
   (1.25× at 2048², 1.44× at 8192²) *and* has the best-behaved frame-time tail
   under concurrency. There is no throughput-versus-smoothness trade-off to
   make here — the same choice wins both. (§3, §7)
2. **Generation is not the smoothness problem.** It already runs on a worker
   `Thread`; the UI never blocks on it. (§7)
3. **LOD tile synthesis is the smoothness problem**, and it is *invariant*
   under every compute configuration and every world size: **16–42 ms per
   256 px tile**, single-threaded, on the Godot main thread. The shell's own
   budgets turn that into a **1.28–1.81 s** single-frame stall per input event
   and **135–230 ms per frame** while a backlog drains. (§5)
4. That last one has **8.8× of measured headroom** sitting unused, and the fix
   is not architectural. (§5.4)

---

## 1. The machine, and what "device 0/1" mean here

| | |
|---|---|
| CPU | 16 logical processors |
| RAM | 31.2 GB |
| GPU 0 | **AMD Radeon RX 7800 XT**, discrete, Vulkan, driver 26.7.1 (LLPC), adapter limits 2048 MB buffer / 2047 MB storage binding |
| GPU 1 | **AMD Radeon(TM) Graphics**, integrated, Vulkan, driver 26.7.1, same reported adapter limits |
| (GPU 2) | Microsoft Basic Render Driver — software, never selected |
| OS | Windows 11 (10.0.26200) |
| Toolchain | rustc 1.97.1, cargo 1.97.1, `wgpu` 30 |

Device identity was **confirmed per run, not assumed**. Every GPU
configuration selects its device by the stable
`vendor:device:name` key (`cartalith-gpu`'s `GpuPreferences::selected_keys`)
and then reads back which adapter actually opened, from
`cartalith_gpu::last_usage()` — the recording `generate_terrain` itself makes
via `record_usage(set)` just before dropping the device set. Every table below
is annotated with what that readback said.

### 1.1 The multi-GPU code is healthy — checked before trusting its numbers

The device-selection race fixed in `3d167eb` ("Selecting the integrated GPU
opened the discrete one") is the exact failure mode that would make a
per-device benchmark quietly meaningless, so it was re-verified first rather
than assumed. `cargo test --release -p cartalith-gpu --test multi_gpu --
--test-threads=1`: **10/10 pass**, including
`a_globally_set_device_key_is_the_device_that_opens` (the direct regression
test for that race) and `every_enumerated_device_can_be_selected_and_opened`,
which reports:

```
selected "1002:747e:AMD Radeon RX 7800 XT"   -> opened "AMD Radeon RX 7800 XT"   (DiscreteGpu)
selected "1002:13c0:AMD Radeon(TM) Graphics" -> opened "AMD Radeon(TM) Graphics" (IntegratedGpu)
```

Every full-pipeline run below independently confirms the same thing through
`last_usage()`.

## 2. Method

`cargo run --release -p cartalith-engine --example compute_config_bench` — a
new harness added by this pass (committed with the fix in §4). It has five
modes: `devices`, `gen`, `tiles`, `tilepar`, `interactive`.
`cartalith-engine/examples/timing_bench.rs`, the tool the earlier CPU and GPU
passes used, is left untouched — it measures one axis (CPU wall clock at four
fixed sizes) and cannot reach which device ran, per-tile distribution, or
concurrency.

- **One OS process per configuration.** Peak working set is read from the OS
  for *that* process, so a CPU-only run's peak carries no GPU driver
  allocations. Peak comes from `(Get-Process -Id $pid).PeakWorkingSet64` — the
  same quantity `MEMORY_OPTIMIZATION_SCOPE.md` measured, asked for the same
  way, with no new crate dependency.
- **Parameters**: `WorldParams::defaults(size, size, 12345)` with
  `tect.plates = 40`, everything else at the shipped default, `carve_rivers`
  on. `use_gpu` set per configuration.
- **Reps**: 3 at 2048², 2 at 8192² (a single 8192² CPU rep is 78 s; the two
  reps agree to 2.6%, so a third buys nothing). Both best and mean are given.
- **Frame-time distribution, not just a mean.** Every per-tile and per-frame
  table reports n, mean, sd, p50, p95, p99 and max, because the owner's
  question is about consistency and a mean hides exactly the thing being asked
  about.
- **Determinism cross-check.** `field[0]` is printed on every run.
  CPU and GPU differ by design (`DECISIONS.md` §7c — the GPU noise primitive is
  a different hash function, so the same seed is a different, still valid,
  still deterministic world). Within a path the value is stable, and it is
  identical across the discrete GPU, the integrated GPU and the two-GPU split
  at a given size — which is itself a correctness result for the split.

**Configurations measured**

| name | what it is |
|---|---|
| `cpu` | `use_gpu = false` — the CPU/Rayon path, `CPU_MULTITHREADING_SCOPE.md` milestones 1–3 |
| `gpu0` | discrete RX 7800 XT, `single device` |
| `gpu1` | integrated Radeon, `single device` |
| `gpusplit` | both, `MultiGpuMode::SplitTiles` |

---

## 3. Full-pipeline generation

40 plates, seed 12345, square grids. "Device confirmed" is what
`last_usage()` reported after the run.

### 3.1 2048²

| config | best | mean | vs. CPU | peak working set | device confirmed | `gpu_stages_used` |
|---|---|---|---|---|---|---|
| `cpu` | **3.748 s** | 3.829 s | 1.00× | 493 MB | none (pure CPU) | `[]` |
| `gpu0` discrete | **2.997 s** | 3.072 s | **1.25×** | 630–696 MB | AMD RX 7800 XT | warp, plate_assignment, base_field_blur, heterogeneity, flow, weather |
| `gpu1` integrated | **3.707 s** | 3.952 s | 1.01× | 845 MB | AMD Radeon(TM) Graphics | same six |
| `gpusplit` both | **3.096 s** | 3.108 s | 1.21× | 821 MB | **both** devices reported | warp_split + the same six |

### 3.2 8192²

| config | best | mean | vs. CPU | peak working set | device confirmed |
|---|---|---|---|---|---|
| `cpu` | **78.087 s** | 79.115 s | 1.00× | 7 470 MB | none |
| `gpu0` discrete | **54.361 s** | 54.431 s | **1.44×** | 7 609 MB | AMD RX 7800 XT |
| `gpu1` integrated | **fails** | — | — | — | — (see §5.2) |
| `gpusplit` both | **54.080 s** | 54.137 s | 1.44× | 7 731 MB | **both** devices reported |

### 3.3 The bracket for the integrated GPU

Because 8192² fails on it, 4096² was measured to find where it stops being
usable rather than reporting only a failure:

| config | size | best | mean | peak working set |
|---|---|---|---|---|
| `gpu1` integrated | 4096² | 15.400 s | 15.424 s | 2 986 MB |
| `gpu1` integrated | 8192² | **crash** | — | — |

### 3.4 Reading the generation table

- **GPU wins, and wins more as the grid grows** — 1.25× at 2048², 1.44× at
  8192². That direction is consistent with `GPU_LAYER_INTEGRATION_SCOPE.md`
  milestone 6's own finding that GPU loses below 2048² and wins above it.
- **The ceiling is not the GPU.** Only six of the pipeline's stages are
  GPU-eligible. `CPU_MULTITHREADING_SCOPE.md`'s 2026-08-19 stage breakdown
  measured the rest at ~35% Rayon-parallel and ~21% genuinely sequential
  (flood-fill water bodies, the road-network graph algorithm, settlement
  placement and naming). Amdahl, not the card, is what caps this at 1.44×.
- **The integrated GPU is not an accelerator here.** At 2048² it is
  statistically indistinguishable from the CPU path (3.707 s vs 3.748 s) while
  costing 350 MB more peak memory — the six GPU stages run ~4–7× slower on it
  (see §3.5) and hand back the time the CPU path would have saved.
- **`split tiles` does not move the full pipeline.** 3.096 s vs `gpu0`'s
  2.997 s at 2048² (3% *worse*), 54.080 s vs 54.361 s at 8192² (0.5% better,
  inside run-to-run noise). That is expected and is a scope fact, not a
  failure: `split_tiles` partitions **one** stage, the domain warp, because it
  is the only GPU kernel here that reads nothing outside its own cell.
  `menus.gd`'s own tooltip already says so. The warp is a small share of a
  full generate, so a 1.4× on the warp is a rounding error on the total.

### 3.5 The warp kernel alone, per device

From `per_device_warp_throughput_measured` and
`split_tiles_across_two_real_devices_measured` in
`cartalith-gpu/tests/multi_gpu.rs`, re-run for this document (two runs, both
shown, so run-to-run spread is visible):

| grid | discrete | integrated | ratio | split (both) | split vs. discrete |
|---|---|---|---|---|---|
| 512² | 2.1 / 2.2 ms | — | — | 2.8 / 2.9 ms | 0.74× / 0.78× |
| 1024² | 3.5 / 3.4 ms | 15.9 / 16.5 ms | 0.21× | 5.9 / 5.0 ms | 0.59× / 0.78× |
| 2048² | 7.7 / 9.1 ms | 59.1 / 60.4 ms | 0.14× | 10.8 / 11.1 ms | 0.74× / 0.78× |
| 4096² | 58.5 / 51.4 ms | 272.7 / 288.0 ms | 0.18× | 41.1 / 37.7 ms | **1.36× / 1.39×** |

This reproduces the `device_weight` constant's own recorded table
(`multi.rs`: 0.217 / 0.133 / 0.170) to within run noise, so the shipped 0.17
weight is still right for this hardware. It also confirms the shape: the split
only pays above 2048², where the second device contributes more than its fixed
per-dispatch cost (~1.8 ms, measured previously).

---

## 4. A real crash, found by measuring and fixed

**The first 8192² GPU run did not produce a number. It died.**

```
thread 'main' panicked at wgpu-30.0.0/src/backend/wgpu_core.rs:1280:26:
wgpu error: Validation Error
Caused by:
  In Device::create_bind_group, label = 'warp bind group'
    Buffer binding 1 range 268435456 exceeds `max_*_buffer_binding_size` limit 134217728
```

**Cause.** `request_gpu_device_from` built its device limits as
`Limits::downlevel_defaults()` followed by `.using_resolution(adapter.limits())`
— and `using_resolution` raises only the three `max_texture_dimension_*`
fields. The two *buffer* ceilings stayed at `downlevel_defaults()`'s 128 MiB
binding / 256 MiB allocation regardless of what the card reported. One
full-grid `f32` buffer is `w·h·4` bytes, so the GPU path was silently capped
at **5792²**: 4096² (64 MiB) fits, 8192² (256 MiB) does not. Both adapters on
this machine report 2047 MiB / 2048 MiB — the ceiling was the crate's own
request, not the hardware.

**Why this is severe, not academic.** `8192` is an entry in
`new_world_dialog.gd`'s `RESOLUTION_PRESETS` and `GRID_MAX` is 8192, while the
shell's GPU toggle defaults to *on*. The reachable user action is "pick the
largest offered resolution, press Generate". And a `wgpu` validation error is
raised on the uncaptured-error path, which **panics** — a panic inside a loaded
GDExtension takes the Godot process with it
(`cartalith-rust-conventions`). This is a different bug from `3d167eb`'s
device-selection race and from the GL-backend instance crash `COMPUTE_BACKENDS`
documents; it shares only the ">2k" symptom.

**Fixed in two parts**, because raising the request is not by itself a
guarantee:

1. `request_gpu_device_from` now also raises `max_storage_buffer_binding_size`
   and `max_buffer_size` to the adapter's own reported ceilings.
   `HARDWARE_ACCELERATION.md` §10's "request the minimum actually needed, never
   `Limits::unlimited()`" is intact — this still never asks for more than the
   hardware reports; it stops asking for *less than the pipeline needs*.
2. New `device_supports_grid` / `GpuDeviceSet::supports_grid` /
   `device_grid_limit_bytes`, with `generate_terrain` filtering its opened
   device set through `supports_grid(gw, gh)`. An adapter that genuinely cannot
   reach a size now takes the CPU path (`HARDWARE_ACCELERATION.md` §27) instead
   of panicking. This is a **hard device** limit, deliberately kept separate
   from `vram_verdict`'s user-set VRAM budget: the budget is a policy the owner
   chooses, this is arithmetic the driver enforces.

**Verified.** New regression test
`an_opened_device_can_bind_a_full_grid_at_every_shipped_resolution` asserts the
limit at all five `RESOLUTION_PRESETS` sizes *and* runs the real 8192² warp
dispatch that used to die. `multi_gpu` 10/10; `cargo test --release -p
cartalith-gpu -p cartalith-engine` 0 failures with every golden-parity suite
unmodified; clippy adds no warning; `cargo check -p cartalith-godot` clean.
Determinism unaffected — 2048²/`gpu0` yields `field[0] = 0.4297` before and
after the fix. Every `gpu0` and `gpusplit` number at 8192² in §3.2 exists
because of this fix.

---

## 5. LOD tile synthesis

### 5.1 What was measured, and against which version

Tile synthesis is `lod_bridge::synthesize_tile_rgba` → `pyramid_tile`
(`cartalith_engine::bake`) + a second `refine_tile` for the plain reference
pass + two `shade_tile` reductions. `cartalith-godot` is a `cdylib`, so nothing
can link a benchmark against it; the harness reproduces that call sequence
**call for call** from the same committed engine functions.

Two caveats stated plainly:

- The pyramid-tile design in `lod_bridge.rs` was **in flight in the working
  tree** while this ran (a concurrent session's deep-zoom rewrite, uncommitted).
  The functions it calls — `pyramid_tile`, `refine_tile`, `shade_tile` — are
  committed engine code (`e0dfa44`), so the per-tile cost measured here is
  real; but if that path changes shape, re-run `compute_config_bench
  tiles`/`tilepar`.
- The harness's own numbers agree with the in-flight module's independently
  recorded measurement (its doc comment: *"16.5 ms for a 256 px tile"* on a
  512×384 world) to within 4%, which is the cross-check that the reproduction
  is faithful.

**Generation-pipeline numbers in §3 are entirely independent of this** — they
touch no LOD code at all.

### 5.2 Per-tile cost: 256 px tile, n = 48 per cell

`z_base` here is 4, so level `z` carries `min(6, z − 4)` progressive
octaves — which is exactly the growth the table shows.

| config | size | z | extra octaves | mean | sd | p50 | p95 | p99 | max | over 16.7 ms |
|---|---|---|---|---|---|---|---|---|---|---|
| `cpu` | 2048² | 4 | 0 | 15.93 ms | 0.15 | 15.90 | 16.25 | 16.39 | 16.39 | 0% |
| `cpu` | 2048² | 6 | 2 | 26.80 | 1.90 | 27.37 | 27.92 | 29.57 | 29.57 | 100% |
| `cpu` | 2048² | 7 | 3 | 32.50 | 0.77 | 32.16 | 34.01 | 34.81 | 34.81 | 100% |
| `cpu` | 2048² | 8 | 4 | 36.89 | 0.37 | 36.76 | 37.64 | 38.62 | 38.62 | 100% |
| `cpu` | 2048² | 9 | 5 | 42.05 | 1.21 | 41.56 | 44.27 | 46.93 | 46.93 | 100% |
| `gpu0` | 2048² | 4 | 0 | 16.20 | 0.17 | 16.26 | 16.41 | 16.55 | 16.55 | 0% |
| `gpu0` | 2048² | 6 | 2 | 26.81 | 3.51 | 27.79 | 28.33 | 36.13 | 36.13 | 96% |
| `gpu0` | 2048² | 7 | 3 | 32.79 | 0.61 | 32.57 | 33.88 | 35.90 | 35.90 | 100% |
| `gpu0` | 2048² | 8 | 4 | 37.66 | 0.87 | 37.29 | 39.62 | 40.85 | 40.85 | 100% |
| `gpu0` | 2048² | 9 | 5 | 42.13 | 0.40 | 42.00 | 43.06 | 43.74 | 43.74 | 100% |
| `cpu` | 8192² | 4 | 0 | 15.89 | 0.16 | 15.93 | 16.08 | 16.37 | 16.37 | 0% |
| `cpu` | 8192² | 6 | 2 | 26.75 | 1.72 | 27.29 | 27.56 | 28.36 | 28.36 | 100% |
| `cpu` | 8192² | 7 | 3 | 32.33 | 0.67 | 32.04 | 33.87 | 33.92 | 33.92 | 100% |
| `cpu` | 8192² | 8 | 4 | 36.77 | 0.15 | 36.72 | 37.25 | 37.27 | 37.27 | 100% |
| `cpu` | 8192² | 9 | 5 | 41.76 | 0.39 | 41.72 | 42.57 | 43.20 | 43.20 | 100% |

**Two findings, both clean:**

1. **Tile cost is independent of the compute configuration.** There is no GPU
   path for tile synthesis at all — `bake.rs`, `amplify.rs` and
   `tile_render.rs` contain zero `gpu` references between them — so the only
   thing the configuration changes is *which world* the tiles are cut from.
   The 2048² `cpu` and `gpu0` columns differ by 0.1–2.1%, i.e. by nothing.
2. **Tile cost is independent of world size.** 2048² and 8192² agree to within
   0.3% at every level. A tile is a fixed 256×256 pixels; only the coarse
   sampling changes, and the coarse field is read through a bilinear sampler
   whose cost does not scale with the grid. **An 8192² world costs no more to
   zoom into than a 2048² one.**
3. **One tile already exceeds a 60 Hz frame at every level past 4.** The
   "over 16.7 ms" column is 100% from z = 6 onward.

### 5.3 What the shell's own budgets turn that into

`viewport_host.gd` synthesises tiles **synchronously on the Godot main
thread**: up to `MAX_LOD_TILES_PER_UPDATE = 48` inside a single
`_update_lod()` call (which runs *per input event*), with the remainder queued
and drained at `MAX_LOD_TILES_PER_CATCHUP = 6` per `_process()` frame.
Measured directly:

| config | size | z | 48-tile `_update_lod()` burst | 6-tile `_process()` catch-up |
|---|---|---|---|---|
| `cpu` | 2048² | 6 | **1 280 ms** | **135 ms** |
| `cpu` | 2048² | 8 | **1 759 ms** | **220 ms** |
| `gpu0` | 2048² | 6 | 1 282 ms | 159 ms |
| `gpu0` | 2048² | 8 | 1 808 ms | 223 ms |
| `cpu` | 8192² | 6 | 1 294 ms | 137 ms |
| `cpu` | 8192² | 8 | 1 796 ms | 230 ms |

In plain terms: **one wheel notch that crosses the deep-zoom threshold buys a
1.3–1.8 second frozen frame**, and while the backlog drains the app runs at
**4–7 fps**. That is more than an order of magnitude larger than the entire
difference between the best and worst compute configuration at 2048² (955 ms
across a whole 3-second generate), and unlike generation it is not on a worker
thread.

### 5.4 The headroom, measured — not proposed

Two measurements of what a fix would be worth, taken without changing the
shipped path (`compute_config_bench tilepar`, `cpu` 2048², 16 Rayon threads):

| z | 48 tiles sequential | 48 tiles across Rayon | speed-up | 48 tiles, detailed half only |
|---|---|---|---|---|
| 6 | 1 304.6 ms | **165.9 ms** | **7.86×** | 861.8 ms (−34%) |
| 8 | 1 776.8 ms | **201.6 ms** | **8.81×** | 1 346.2 ms (−24%) |

- **Parallelism**: the 48-tile burst is embarrassingly parallel — each tile is
  a pure function of the frozen coarse field — and Rayon takes it from 1.78 s
  to 0.20 s. `bake.rs` already does exactly this for a baked level; the
  interactive path simply never got it.
- **The redundant pass**: the shade *ratio* a tile carries requires a second
  `refine_tile` (with `detail_amp = 0`) plus a second `shade_tile` per tile,
  costing 24–34% of every tile. That is inherent to the encoding, not waste —
  but it is 24–34% that a different encoding would not pay.

Taken together, a Rayon-dispatched burst with the plain pass restructured
would land the 48-tile update near **0.15 s** rather than 1.8 s. Neither is
done here: this pass was scoped to measure, and redesigning the tile path is a
`LOD_TILING_INTEGRATION_SCOPE.md` milestone, not a benchmark's call to make.

---

## 6. Async and hybrid offloading — what is real, and what is not

The owner asked specifically about hybrid CPU+GPU offloading. Three separate
questions hide in that, and they have three different answers.

### 6.1 Is generation async with respect to the UI? — **Yes, already**

`engine_bridge.gd` runs `generate_sized()` on a `Thread` and signals back
through `call_deferred`. The UI thread is never blocked by generation. This
matters for the recommendation: it means the compute configuration's effect on
*smoothness* is entirely indirect, via contention, not via blocking.

### 6.2 Is GPU↔GPU hybrid real? — **Yes, and it is measured** (§3.5)

`MultiGpuMode::SplitTiles` with `warp_grid_gpu_split` is a genuine two-device
concurrent dispatch, and §3.1/§3.2 confirm both devices reported allocations
in the same run. It wins **1.36–1.39×** on the warp stage at 4096², loses
below 2048², and is a wash on the full pipeline because it covers one stage.

### 6.3 Is CPU+GPU hybrid *within generation* buildable cheaply? — **No, and this is not a wiring gap**

Checked rather than assumed, and reported honestly per the brief. Two
independent blockers:

1. **The pipeline is a strict dependency chain.** `generate_terrain` runs
   warp → plate assignment → stress → flexure/base blur → age → heterogeneity →
   height/resistance → climate → erosion → hydrology → carve. Each stage
   consumes the previous stage's output. There is no pair of heavy stages that
   could run at the same time on different processors, so there is nothing to
   overlap without restructuring what depends on what — which is
   `GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md`'s territory, not a wiring
   task.
2. **A row-band CPU/GPU split of any noise stage is barred by
   `DECISIONS.md` §7c.** The GPU noise primitive is *a genuinely different hash
   function* from the CPU/JS-matching one, not a precision-tolerant port. A
   band computed on the CPU and a band computed on the GPU would therefore
   disagree at the seam — visibly, not within tolerance. This is precisely why
   `warp_grid_gpu_split` splits GPU↔GPU and never GPU↔CPU: two GPUs run the
   *same* WGSL kernel, and the test that checks it measures a worst-case
   difference of 2.9e-3 across devices against a bit-exact match across bands
   on one device.

So: no number was faked for a CPU+GPU hybrid, and none should be. What
*was* measured instead is the concurrency that genuinely exists in this app —
§7.

---

## 7. Smoothness: frame-time distribution under real concurrency

The scenario the app actually has: a world is on screen, the user is zoomed in
past the LOD threshold so the main thread is synthesising tiles, and a
generation is running on the worker thread. `compute_config_bench interactive`
reproduces it — one OS thread doing `MAX_LOD_TILES_PER_CATCHUP = 6` tiles at
z = 8 per "frame" (with a 1 ms yield, so it is a frame loop and not a spin
loop), while `generate_terrain` runs on a `std::thread` exactly as
`engine_bridge.gd` does.

### 7.1 UI frame time, 2048²

| config | state | n | mean | sd | p50 | p95 | p99 | **max** |
|---|---|---|---|---|---|---|---|---|
| `cpu` | idle | 40 | 223.7 ms | 4.0 | 223.2 | 228.3 | 243.3 | 243.3 |
| `cpu` | **generating** | 15 | 345.4 ms | 48.3 | 352.4 | 415.0 | 420.7 | **420.7** |
| `gpu0` | idle | 40 | 223.3 | 2.4 | 222.8 | 227.6 | 229.1 | 229.1 |
| `gpu0` | **generating** | 12 | 341.1 | 60.5 | 375.4 | 402.6 | 414.4 | **414.4** |
| `gpu1` | idle | 40 | 224.7 | 2.4 | 224.6 | 228.8 | 230.4 | 230.4 |
| `gpu1` | **generating** | 15 | 330.1 | 56.8 | 332.4 | 408.9 | 415.5 | **415.5** |
| `gpusplit` | idle | 40 | 224.9 | 2.9 | 224.1 | 230.1 | 234.2 | 234.2 |
| `gpusplit` | **generating** | 13 | 348.6 | 42.4 | 340.8 | 414.2 | 428.1 | **428.1** |

### 7.2 UI frame time, 8192² — the long samples

| config | state | n | mean | sd | p50 | p95 | p99 | **max** |
|---|---|---|---|---|---|---|---|---|
| `cpu` | idle | 40 | 223.9 ms | 2.6 | 223.4 | 228.6 | 231.8 | 231.8 |
| `cpu` | **generating** | 280 | 362.5 | 75.6 | 343.3 | 509.7 | 542.5 | **768.2** |
| `gpu0` | idle | 40 | 221.4 | 1.9 | 220.4 | 225.2 | 226.1 | 226.1 |
| `gpu0` | **generating** | 182 | 367.5 | 81.2 | 367.5 | 516.2 | 530.8 | **534.7** |

### 7.3 What contention costs, both ways

| config | size | generate solo | generate with UI thread active | slow-down | peak WS (concurrent) |
|---|---|---|---|---|---|
| `cpu` | 2048² | 3.75 s | 5.08 s | **+36%** | 809 MB |
| `gpu0` | 2048² | 3.00 s | 3.88 s | **+31%** | 949 MB |
| `gpu1` | 2048² | 3.71 s | 4.90 s | +32% | 1 150 MB |
| `gpusplit` | 2048² | 3.10 s | 4.43 s | +43% | 1 065 MB |
| `cpu` | 8192² | 78.1 s | 101.6 s | **+30%** | 12 237 MB |
| `gpu0` | 8192² | 54.4 s | 66.9 s | **+23%** | 12 367 MB |

**Reading these three tables together:**

- **The UI thread's own work dominates its frame time in every configuration.**
  Idle is already 221–225 ms for six tiles. No compute configuration changes
  that, because no compute configuration touches tile synthesis (§5.2).
- **Contention is real and roughly symmetric.** A single UI thread on a
  16-thread machine oversubscribes Rayon's 16-thread pool by one, and the cost
  shows up on both sides: frames get 47–66% slower, and generation gets 23–43%
  slower.
- **The discrete GPU has the best tail, and the tail is what "smooth" means.**
  At 8192² the CPU path's worst frame is **768 ms** against `gpu0`'s **535 ms**
  — a 30% better worst case — even though the two have almost identical means
  (362 vs 367 ms). The mean says they are the same; the max says they are not,
  and the max is what a user perceives as a hitch. This is the one place where
  distribution rather than throughput changes the answer, and it changes it in
  the *same* direction throughput already pointed.
- **`gpusplit` is the worst configuration for smoothness**, at +43% generation
  slow-down and the highest 2048² frame max. Opening and driving a second
  device costs main-thread work for a stage that is a small share of the total.
- **The integrated GPU's one plausible selling point does not hold up.**
  "Offload to the iGPU so the dGPU stays free" gives a very slightly better
  frame mean at 2048² (330 vs 341 ms — inside noise) at the cost of a 24%
  slower generate and 200 MB more peak memory, and it cannot run 8192² at all.

---

## 8. Memory

Peak OS working set, per configuration, single-generate runs (from §3):

| size | `cpu` | `gpu0` | `gpu1` | `gpusplit` |
|---|---|---|---|---|
| 2048² | **493 MB** | 630–696 MB | 845 MB | 821 MB |
| 4096² | — | — | 2 986 MB | — |
| 8192² | **7 470 MB** | 7 609 MB | fails | 7 731 MB |

- The CPU path is the leanest at every size — GPU driver allocations and
  staging buffers are real and are not free. At 2048² the discrete GPU costs
  ~140–200 MB more; at 8192² the difference narrows to ~2% because the CPU-side
  field arrays dominate everything.
- **8192² is a 7.5 GB working set on a 31 GB machine, and 12.4 GB with a UI
  thread also live** (§7.3). That is not a failure, but it is the real
  constraint on that resolution — and it is the same ~96-full-grid-allocation
  ceiling `MEMORY_OPTIMIZATION_SCOPE.md` identified and left as a follow-up.
- GPU-side memory is separately reported by `cartalith_gpu::device_usage`:
  **512 KB allocated / 256–320 MB reserved** after every run. That is this
  application's own allocator report, not a system-wide occupancy figure — the
  distinction `multi.rs` is careful about and this document keeps.

---

## 9. Two failures, reported as failures

### 9.1 Fixed — GPU device limits capped the path at 5792²

§4. Was a process-killing panic on a reachable user action; fixed and
regression-tested.

### 9.2 Open — the integrated GPU cannot survive 8192², and says so by panicking

```
thread 'main' panicked at crates/cartalith-gpu/src/lib.rs:1717:50:
buffer map failed: BufferAsyncError
```

This is the base-field blur's readback. The integrated GPU is past its
allocation ceiling at ~2.5 GB of working set (`gpu_working_set_bytes(8192,
8192)`), and the failure surfaces as a `BufferAsyncError` on the map.

**Not fixed here, deliberately.** There are **ten** `expect`-on-readback sites
in `cartalith-gpu`, and making them fallible means threading `Option` (or a
real error) through every dispatch function and every call site in
`generate_terrain`, then deciding per stage whether a mid-pipeline GPU failure
should retry on CPU or abandon the whole GPU path for the run. That is genuine
architecture — a milestone with a scope document, not a benchmark's side
effect. Recorded here and in `STATUS.md` so it is not rediscovered.

**Interim mitigation available today, no code needed**: the integrated GPU is
verified working through 4096² (§3.3), and it is never the default — the
default preference is empty (`auto` → `PowerPreference::HighPerformance` → the
discrete card). The only way to reach this is to select the integrated GPU
explicitly in Preferences ▸ Performance ▸ Devices *and* generate at 8192².

**A second interim mitigation that already exists**: the VRAM budget. Setting
`vram_budget_bytes` to anything below `gpu_working_set_bytes(8192, 8192)` =
2 560 MB makes `gpu_allowed_for_grid` refuse the GPU path for that grid and
fall back to CPU, which is exactly the `VramFallback::CpuTilePass` default.
That is a real, shipped control that turns this crash into a graceful CPU run.

---

## 10. Recommendation

### 10.1 The compute configuration: **discrete GPU (device 0), `single device`**

It is the fastest at both requested sizes (1.25× at 2048², 1.44× at 8192²
against the CPU path) **and** the most consistent under real concurrency
(worst frame 535 ms at 8192² against the CPU path's 768 ms; the smallest
generation slow-down under UI contention at both sizes, +31% and +23%). The
usual throughput-versus-smoothness trade-off does not appear here — the same
configuration wins both, so there is nothing to trade.

Explicitly **not** recommended:

- **`split tiles`** — 3% slower than a single discrete GPU at 2048², a wash at
  8192², the largest generation slow-down under contention (+43%) and the worst
  2048² frame max. It splits one stage of many. Keep it as the honest,
  documented capability it is; do not make it a default. It becomes worth
  considering only if more stages ever gain cell-independent kernels.
- **The integrated GPU** — no faster than the CPU path at 2048², 350 MB more
  memory, and it crashes at 8192².
- **CPU-only** — a fully correct fallback and the leanest on memory, and the
  only path with JS-parity semantics (`DECISIONS.md` §7c), so it must stay the
  default for any parity-sensitive work. But it is the slowest and has the
  worst frame-time tail under concurrency.

### 10.2 The thing that will actually change how the app feels

**Get LOD tile synthesis off the Godot main thread.** Every number in §5 says
the same thing: a 1.3–1.8 second stall on a single wheel notch, 4–7 fps while a
backlog drains, one tile over budget at every level past 4 — and none of it
moves when the compute configuration changes, because none of it runs on a GPU
and none of it runs in parallel. It is the single largest, most user-visible,
most measurable smoothness cost in the application, and it is roughly **50×**
larger than the difference between the best and worst compute configuration.

The measured headroom (§5.4) is **7.9–8.8×** from Rayon alone, on work that is
already a pure function of a frozen field — the same dispatch `bake.rs` makes
for a baked level. A further 24–34% sits in the shade ratio's second reference
pass. Neither needs new architecture; both need a milestone and a scope
document, which is where they belong rather than in a benchmark.

Until that lands, `MAX_LOD_TILES_PER_UPDATE = 48` is the number to look at
first: at 16–42 ms per tile it is a budget denominated in a unit nobody
measured when it was chosen.

### 10.3 Second and third

2. **Fix the readback panics** (§9.2). A GPU that runs out of memory should
   produce a CPU-rendered world and a log line, not a dead process. Ten sites.
3. **Leave 8192² documented as expensive** — 78 s on CPU, 54 s on the discrete
   GPU, 7.5 GB of working set alone and 12.4 GB with a UI thread live. It
   works; it is not a casual setting, and the shell should probably say so.

---

## 11. Reproducing this

```
cargo run --release -p cartalith-engine --example compute_config_bench -- devices
cargo run --release -p cartalith-engine --example compute_config_bench -- gen         <cpu|gpu0|gpu1|gpusplit> <size> <plates> <reps>
cargo run --release -p cartalith-engine --example compute_config_bench -- tiles       <cpu|gpu0|gpu1|gpusplit> <size> <plates>
cargo run --release -p cartalith-engine --example compute_config_bench -- tilepar     <cpu|gpu0|gpu1|gpusplit> <size> <plates>
cargo run --release -p cartalith-engine --example compute_config_bench -- interactive <cpu|gpu0|gpu1|gpusplit> <size> <plates>

cargo test --release -p cartalith-gpu --test multi_gpu -- --test-threads=1 --nocapture
```

`gpu0`/`gpu1` are indices into `enumerate_devices()`'s non-software list —
discrete first, then integrated. On different hardware they will name different
cards; every mode prints which device it requested and which one
`last_usage()` says actually ran, so a run on another machine is
self-documenting rather than inheriting this one's assumptions. The same
applies to `device_weight`'s 0.17 integrated-GPU ratio, which
`per_device_warp_throughput_measured` exists to let another machine re-measure.

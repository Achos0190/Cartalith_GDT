# Compute-configuration benchmarks (agent-produced, 2026-08-24)

> **Agent-produced, and every number in it was measured on this machine.**
> Nothing here is estimated, scaled from another size, or inherited from an
> earlier pass — where a figure comes from an existing document it says so and
> names it. Two failures are reported as failures rather than omitted, and
> **both were real process-killing crashes that this run found**; both are now
> fixed (§9, `CHANGELOG.md` same date; commits `504c2a6` and the readback-
> fallback commit that follows it).

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
3. **LOD tile synthesis was the smoothness problem** — *fixed 2026-08-25, see
   §5.5.* It was invariant under every compute configuration and every world
   size: **16–42 ms per 256 px tile**, single-threaded, on the Godot main
   thread, which the shell's own budgets turned into a **1.28–1.81 s**
   single-frame stall per input event and **135–230 ms per frame** while a
   backlog drained. (§5)
4. That last one had **8.8× of measured headroom** sitting unused, and the fix
   was not architectural. **Claimed**: 2.82–5.97 ms per tile, a **0.25 s**
   burst, 0 % of tiles over a 60 Hz frame at any level. (§5.4, §5.5)

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
| `gpu1` integrated | **81.905 s** | — | 0.95× | 7 571 MB | AMD Radeon(TM) Graphics, **for three stages** — see §9.2 |
| `gpusplit` both | **54.080 s** | 54.137 s | 1.44× | 7 731 MB | **both** devices reported |

### 3.3 The bracket for the integrated GPU

Because 8192² does not complete on it, 4096² was measured to find where it
stops being usable rather than reporting only a failure:

| config | size | best | mean | peak working set | note |
|---|---|---|---|---|---|
| `gpu1` integrated | 4096² | 15.400 s | 15.424 s | 2 986 MB | fully on GPU |
| `gpu1` integrated | 8192² | 81.905 s | — | 7 571 MB | **three GPU stages, then CPU** (§9.2) |

The 8192² row was a **crash** when this document was first measured. It is now
a graceful, correct, slightly-slower-than-CPU run: the device fails its first
whole-grid readback partway through and everything after it falls back
(`HARDWARE_ACCELERATION.md` §27). The 5% behind the pure-CPU path is the three
GPU stages that ran and were then discarded. The recommendation in §10 is
unchanged — this makes a bad configuration survivable, not good.

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

### 5.5 Claimed (2026-08-25, the `/ponytail` pass) — a level below where §5.4 proposed

§5.4 measured the headroom over the **burst**, which is `viewport_host.gd`'s
call loop and therefore a shell change. It is claimable one level down instead,
entirely inside the engine, with the shell asking for a tile exactly the way it
does today: `amplify_region` and `add_zoom_detail`
(`cartalith-terrain/src/amplify.rs`) and `shade_tile` (`tile_render.rs`) are the
*entirety* of a tile's cost, and all three are `output[i] = f(frozen input, i)`
— `CPU_MULTITHREADING_SCOPE.md`'s own bar, the shape milestones 1-3
parallelised across five other crates. Row-parallel via `par_chunks_mut`,
per-pixel arithmetic untouched and unreordered, so the output is **bit-identical**
and the goldens pass at exact equality rather than a new tolerance.

Re-run of §5.2 and §5.3, same command, same machine, same seed:

| z | per tile | 48-tile burst | 6-tile catch-up | over 16.7 ms |
|---|---|---|---|---|
| 4 | 15.94 → **2.82 ms** | | | 2 % → **0 %** |
| 6 | 27.55 → **4.25 ms** | 1 314.4 → **196.6 ms** | 164.0 → **25.9 ms** | 100 % → **0 %** |
| 7 | 32.16 → **4.62 ms** | | | 100 % → **0 %** |
| 8 | 36.71 → **5.27 ms** | 1 768.6 → **252.4 ms** | 220.1 → **31.2 ms** | 100 % → **0 %** |
| 9 | 41.54 → **5.97 ms** | | | 100 % → **0 %** |

5.7–7.0× per tile and **6.7–7.0× on the burst** — close to §5.4's 7.9–8.8×
ceiling, which is what one should expect, since the same 16 threads are doing
the same total work either way. Peak working set 498 → 492 MB. **§5.2's finding
3 is now false in the best way**: one tile fits inside a 60 Hz frame at every
level, including the deepest this port has.

Batching the burst on top is still worth roughly the remaining ~4× (the
per-dispatch overhead 48 separate row-parallel calls pay that one tile-parallel
call would not), and stays where §5.4 put it: a
`LOD_TILING_INTEGRATION_SCOPE.md` milestone, needing the shell. So is the
redundant plain pass — still 24–34 % of every tile, still inherent to the
encoding.

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
  slower generate and 200 MB more peak memory, and at 8192² it gives out
  partway through and finishes on the CPU anyway, 5% behind a pure-CPU run
  (§9.2).

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

## 9. Two failures, both now fixed

*Both were found by measuring rather than by reading, and both were
process-killing panics on a reachable user action. §9.2 was reported open when
this document was first written; it was closed on the same day, and the entry
below now records the fix and the second bug it uncovered.*

### 9.1 Fixed — GPU device limits capped the path at 5792²

§4. Was a process-killing panic on a reachable user action; fixed and
regression-tested.

### 9.2 Fixed 2026-08-24 — the integrated GPU could not survive 8192², and said so by panicking

**What it did:**

```
thread 'main' panicked at crates/cartalith-gpu/src/lib.rs:1717:50:
buffer map failed: BufferAsyncError
```

The base-field blur's readback. The integrated GPU is past its allocation
ceiling at ~2.5 GB of working set (`gpu_working_set_bytes(8192, 8192)`), and
the failure surfaced as a `BufferAsyncError` on the map — a `.expect`, i.e. a
panic, i.e. a dead Godot process.

**What it does now** (same command, same device, this document's `gen gpu1
8192 40` run, re-measured after the fix):

```
cartalith-gpu: buffer map failed (BufferAsyncError) on AMD Radeon(TM) Graphics
at 67108864 cells -- this device is done for this run; falling back to CPU
# gpu_stages_used = ["warp", "plate_assignment", "base_field_blur"]
RESULT gen cfg=gpu1 size=8192 plates=40 best=81905.1ms
RESULT peak_working_set_mb cfg=gpu1 size=8192 7571
```

**81.9 s and a correct world**, against the CPU path's 78.1 s — the 5% is the
three GPU stages that ran before the device gave out, paid and then discarded.
That is the honest cost of discovering the wall by hitting it, and it is what
§27's "transition to CPU" is supposed to look like.

**Two bugs, not one.** The first is the ten `.expect`-on-readback sites; every
dispatch in `cartalith-gpu` now returns `Option`, and every engine call site
was already shaped as `match gpu.map(…) { Some => …, None => cpu }`, so the
change there was `map` → `and_then`. The second was only visible once the
first was fixed and this run was repeated: with the readback made fallible,
the blur fell back cleanly and the **next** stage — weather, on a 240² coarse
grid, nowhere near any size limit — panicked immediately on a 32-byte uniform
buffer with `Buffer with 'weather params' label is invalid`. A `wgpu` device
that loses a `map_async` is *gone*, not merely out of room at that size. So a
readback failure now marks the live device lost, at every size, for the life
of that device; the next `generate_terrain` opens its own and starts clean.

**Regression-tested against the real device, not a mock.**
`a_full_8192_generation_on_the_integrated_gpu_completes_or_falls_back` in
`cartalith-gpu/tests/multi_gpu.rs` runs this exact generation on this exact
integrated GPU. A companion test
(`the_integrated_gpu_at_8192_falls_back_instead_of_panicking`) shows why the
whole-pipeline test is needed: an *isolated* 8192² warp on the same device
completes fine in ~1.0 s — only under the pipeline's accumulated working set
does the readback fail.

**The two mitigations that pre-dated the fix still stand**, and are now
redundancy rather than the only defence: the integrated GPU is never the
default (empty preference → `PowerPreference::HighPerformance` → the discrete
card), and setting `vram_budget_bytes` below `gpu_working_set_bytes(8192,
8192)` = 2 560 MB refuses the GPU path for that grid up front.

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

### 10.2 The thing that will actually change how the app feels — **done, 2026-08-25**

> **Claimed.** §5.5 has the after-numbers: 2.82–5.97 ms per tile, a 0.25 s
> burst, 0 % of tiles over a 60 Hz frame at every level. The recommendation
> below is kept verbatim because its *diagnosis* was right and worth preserving;
> only its framing — "get it off the main thread" — turned out to be one option
> among two, and the cheaper one was to make the tile itself 7× faster where it
> already runs. `MAX_LOD_TILES_PER_UPDATE = 48` is still the number nobody
> measured before choosing, but at 2.82–5.97 ms per tile it now buys a 0.25 s
> burst rather than a 1.8 s one.

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

---

## 12. Snapdragon handsets: per-stage generation on the OnePlus Pad 3 and 6T (2026-11-14)

**The question** (owner, 2026-11-14): *"we should also check optimisation on snapdragon cpu (like the pad3 and 6t, now
the 6t is old, but generation in the pad feels as slow."* Measured over `adb` on both handsets the same evening, then
tuned one change at a time. Routed from `OUTSTANDING_WORK.md` ("Snapdragon CPU optimisation").

### 12.1 Method

- **Harness:** `cartalith-godot/src/device_stage_bench.rs` (`device_stage_bench`, `#[ignore]`d), added by this pass. It
  generates the world the app makes, `generate_terrain` over `params::defaults()` then `compute_civilisation`, at the New
  World dialog's defaults: 2048 x 1311 cells (2K, region frame), 800 km, seed 12345, CPU path unless `SNAP_GPU=1`. Stage
  boundaries come from polling `cartalith_engine::progress` every 1 ms, so no engine code is touched. On Android it also
  reads each stage's process CPU time from `/proc/self/stat` (CPU time / wall time = average busy cores) and which core
  each thread ran on. It prints a digest of the field, temperature, rainfall, discharge, river mask, settlements and ways.
- **On the handsets** the test binary is built with `cargo ndk -t arm64-v8a test --release -p cartalith-godot --lib
  --no-run`, pushed to `/data/local/tmp` and run from `adb shell` under `nohup` (an `adb` server restart by another
  session killed two attached runs). The binary is the same code and profile as the shipped `.so`, but it runs in the
  shell's cpuset, not the app's (both are `0-7` on both handsets). **It does not measure the app's own extra steps**:
  `absorb()`, `warm_render_caches()`, the first draw, or the shell's idle-mode frame pacing.
- **Comparisons are interleaved:** each variant runs once per round in its own process, with a cooldown before every run
  (60 s on the Pad, 20 s on the 6T) and a guard that waits until the installed Cartalith app is under 10 % CPU. One 6T
  round set (A/B2) was discarded because another session launched the app mid-run (it used 103 % CPU, and CPU
  temperatures reached 89 degC); A/B3 repeated it with the guard.
- `simpleperf` is not usable from the `adb shell` user on either handset ("Event type 'cpu-clock' is not supported on the
  device"), hence the `/proc` CPU-time method.
- **The PC:** same harness, `cargo test --release`, run alone. The first baseline ran at 2 % background load; the
  LTO comparison ran at 47 % background load from another session, and its fifth round (34.5 s) is excluded below.

**Hardware, as read on the devices:** OnePlus 6T (`ONEPLUS A6013`, Snapdragon 845): cpu0-3 at 1.766 GHz and cpu4-7 at
2.803 GHz; the kernel advertises `fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp asimdhp` (no `asimddp`,
`asimdrdm` or `lrcpc`). OnePlus Pad 3 (`OPD2415`, Snapdragon 8 Elite): cpu0-5 at 3.533 GHz and cpu6-7 at 4.320 GHz, all
one core type; its feature list adds `asimdrdm lrcpc asimddp i8mm bf16` and others. Both run Android API 35/36.

### 12.2 Results: total New World generation, median (min..max)

| | OnePlus 6T | OnePlus Pad 3 | PC |
|---|---|---|---|
| **Before** (cargo defaults), 5 interleaved runs | **55.55 s** (55.22..55.69) | **20.94 s** (20.82..24.74) | 10.44 s (10.42..10.46), 5 runs alone |
| `lto = "fat"`, `codegen-units = 1` | **54.09 s** (53.98..54.44), A/B1; 53.76 s (53.72..54.27), A/B3 | 20.70 s (20.38..23.09), A/B1; 20.57 s (20.40..24.50), A/B2 | 10.63 s (10.62..10.79) vs 10.80 s (10.73..10.86) before, 4 interleaved runs |
| LTO + `+v8.2a,+dotprod,+fp16,+rcpc` | 54.71 s (54.02..54.75), A/B3 | 20.57 s (20.45..20.67), A/B2 | not built (Android-only flags) |
| LTO + `RAYON_NUM_THREADS=8` | 53.01 s (52.86..53.84), A/B3 | not run (the Pad's pool is already 8) | - |
| LTO + `use_gpu` on (the shell's boot default) | 56.65 s (56.26..56.83) vs 54.51 s (54.16..54.84) CPU, 3 interleaved | 20.74 s (20.60..20.85) vs 20.49 s (20.42..21.00) CPU, 3 interleaved | - |
| **Final (shipped): LTO only** | **54.51 s** (54.16..54.84) | **20.49 s** (20.42..21.00) | 10.63 s (10.62..10.79) |

Reading it: **LTO is a 2.6 % gain on the 6T** (A/B1's ranges do not overlap) and **no difference is established on the
Pad or the PC** (the ranges overlap). The target features and the 8-thread pool show **no difference established** on
either handset (each range overlaps LTO's), so neither shipped. The GPU path is **slower on the 6T** (about 4 %, ranges
apart) and **no different on the Pad**.

### 12.3 Per stage, before and final (median, min..max, seconds; busy cores from the final build's CPU time)

| stage | 6T before | 6T final | 6T busy cores | Pad before | Pad final | Pad busy cores | PC before |
|---|---|---|---|---|---|---|---|
| Tectonics | 21.44 (21.36..21.69) | 20.70 (20.66..20.79) | 2.15 | 8.07 (8.01..8.62) | 7.81 (7.75..8.06) | 2.13 | 4.55 (4.54..4.56) |
| Volcanism & impacts | 2.90 (2.82..3.04) | 2.90 (2.82..2.92) | 1.86 | 0.96 (0.94..1.33) | 0.92 (0.90..0.97) | 2.43 | 0.39 (0.39..0.40) |
| Erosion | 7.09 (6.95..7.25) | 7.27 (7.12..7.50) | 1.17 | 1.80 (1.76..2.02) | 1.92 (1.86..1.94) | 1.13 | 1.03 (1.02..1.04) |
| Hydrology | 2.07 (2.06..2.10) | 2.06 (2.06..2.07) | 1.11 | 0.72 (0.71..1.06) | 0.68 (0.67..0.69) | 1.11 | 0.30 (0.29..0.30) |
| Climate (to the end of terrain) | 8.24 (8.15..8.63) | 8.37 (8.33..8.56) | 1.54 | 2.92 (2.88..4.20) | 2.83 (2.79..2.94) | 1.91 | 1.26 (1.25..1.27) |
| terrain total | 41.78 (41.77..42.10) | 41.51 (41.04..41.60) | 1.78 | 14.43 (14.37..16.67) | 14.09 (14.03..14.60) | 1.92 | 7.51 (7.51..7.56) |
| civilisation | 13.60 (13.45..13.77) | 13.11 (12.99..13.24) | 2.69 | 6.57 (6.44..8.62) | 6.40 (6.33..6.45) | 2.75 | 2.93 (2.85..2.94) |

"Before" is A/B1's five runs of the default build; "final" is the shipped LTO build's three CPU runs (the CPU column of
the GPU comparison); the PC column is the first, quiet baseline. Stages 0-2 (Planet, Extent, World structure) take under
1 ms and get no row; "Climate" runs from the climate stage to the end of `generate_terrain`, because the last two stages
are advanced together at the end. The 6T's busy cores are of 6 pool threads, the Pad's of 8.

**Determinism.** Every CPU run on every machine and build printed the same digest, `terrain=632b22edf25908e1
civ=f6708d1a30982bb1` (40 settlements, 66 ways): default and LTO on the PC, the 6T and the Pad, the target-feature build
on both handsets, and the 8-thread pool. No flag moved a value, so none had to be dropped for it. The GPU path differs
by design (`DECISIONS.md` section 7c), and it also **differs between the two handsets**: `terrain=a98006fb0982d1fa
civ=c1ee4e88332f4367` (63 ways) on the Pad's Adreno, `terrain=14fa38faada65f7a civ=81260e568e825a3c` (64 ways) on the
6T's, both using the same six GPU stages (warp, plate assignment, stress, base field blur, heterogeneity, weather). With
the shell's boot default (`use_gpu` on, `engine_bridge.gd`), one seed therefore makes a different world on each handset.

### 12.4 Why the Pad is not as much faster as its CPU

1. **It is faster, by 2.6 times on the engine path:** 20.5-20.9 s against 53.8-55.6 s for the same world, cold. The
   harness does not reproduce "as slow" for generation itself; what it does not cover (above) is still open.
2. **The pipeline keeps about two cores busy.** Over the whole terrain pass the Pad averaged 1.92 busy cores of 8, and the
   6T 1.78 of 6; erosion and hydrology run at about 1.1, i.e. serially. So the 8 Elite's six extra cores mostly idle,
   and the speed-up between the handsets is roughly the single-core one. This is the same Amdahl limit section 3.4 found
   on the PC, and it is not a build setting: it needs parallel algorithms in the serial stages.
3. **The Pad throttles under sustained load; the 6T does not.** Five generations back to back in one process (before
   the cooldown method was adopted): Pad 20.90, 20.94, 26.31, 29.44, 30.94 s; 6T 55.12, 55.45, 55.14, 55.15, 55.17 s.
   After about two minutes of load the Pad lost about a third of its speed. The cause of the slowdown (thermal
   throttling) is inferred from the shape, not measured directly.
4. **Not the causes:** the thread pool is not capped (8 threads on the Pad; 6 on the 6T, where `available_parallelism()`
   reports 6 at start because two cores are offline, and 8 threads made no established difference); the build was not
   the problem (LTO 0-2.6 %, the ISA baseline nothing); the GPU path runs on both (wgpu on Vulkan, six stages) and helps
   neither.

**Core use (6T, final build, share of CPU ticks):** cpu4-7 (the 2.8 GHz cores) carried 77-78 % of the ticks and cpu0-3
22-23 %, so the work does reach the big cores. On the Pad the two 4.32 GHz cores carried 47-49 %.

### 12.5 What shipped, and reproducing it

`[profile.release] lto = "fat", codegen-units = 1` in `cartalith-native/Cargo.toml`, with this table's figures in its
comment. No `.cargo/config.toml` and no thread-pool change (a thread rule came later, with the parallel stages: 12.6). The ISA baseline tried was chosen as one both handsets'
cores implement: Armv8.2-A with dot product on the SD845's Cortex-A75/A55-derived cores (Arm, *Exploring the Arm dot
product instructions*, developer.arm.com community blog, which names the Cortex-A55 and A75 as implementing them), fp16
as the 6T's own `fphp asimdhp` features show, and RCpc. The 6T's kernel does not advertise `asimddp` or `lrcpc`, so
the only evidence that they execute there is that the build ran: its disassembly carries 16 642 `ldapr`/`ldaprb`
(RCpc) sites and inline LSE atomics, and no `sdot`/`udot` at all, so dot product was never exercised. A later ISA flag
needs that check again. The release `.so` built with it is 27 920 368 bytes; the
Android test binary went from 46 269 576 to 33 045 896 bytes and its cold build from 7 m 13 s to 10 m 57 s.

```
cargo test --release -p cartalith-godot --lib device_stage_bench -- --ignored --nocapture --test-threads 1
cargo ndk -t arm64-v8a test --release -p cartalith-godot --lib --no-run
adb push target/aarch64-linux-android/release/deps/cartalith_godot-<hash> /data/local/tmp/snap_bench
adb shell "cd /data/local/tmp && SNAP_REPS=1 SNAP_WARM=0 ./snap_bench device_stage_bench --ignored --nocapture --test-threads 1"
```

Environment: `SNAP_REPS`, `SNAP_WARM`, `SNAP_GW`, `SNAP_SEED`, `SNAP_GPU` (the module comment). Run one variant per
process with a cooldown between runs on the Pad, or its throttling becomes the measurement.

### 12.6 Parallelising the serial stages, the GPU per stage, and the thread rule (2026-11-15)

**The request** (owner, 2026-11-15): *"6t is still connected, can use it also to do test with parallelisation on
android. Same idea as the pc version. See where the gpu gives a benefit and use the multicore/multithreading. Should
provide a base of for the pad and other mobile devices."* The Pad 3 was off; everything here is the PC and the 6T.

**Method.** The same harness and world as 12.1 (2048 x 1311, seed 12345, `params::defaults()`), release with LTO, run
alone: on the 6T one variant per process, interleaved, 20 s cooldown, the app guard of 12.1; on the PC interleaved, 5 s
apart (the PC read 0-37 % background load from other sessions between runs; the spreads below include it). "Before" is
`origin/main` at `2e9fd4db`, built in its own worktree. Where the serial time went was found first with temporary
segment timers (not committed): every statement of `generate_terrain_inner` and `compute_civilisation`, run once at
16 threads and once at `RAYON_NUM_THREADS=1` on the PC, and once on the 6T with `/proc` CPU time per segment.

**Where the serial time was (one run each, PC 16 threads, before):** the orogen dissection's stream power 3.33 s and
the light pass's 1.06 s, of which the MFD drainage-area pass was 0.30 s per routing (7 routings) and the priority
flood 0.21 s per routing; the in-place jump flood of `assign_plates` 0.28 s; four D8 flow accumulations about 0.11 s
each; in civilisation the coast SDF's two jump floods 0.81 s, the capitals' Dijkstras 0.53 s, the water bodies' flood
0.31 s. On the 6T the same places, larger, plus two the PC hides: memory-latency-bound serial loops (the dissection's
incision loop took 1.74 s there against 0.13 s on the PC, in the instrumented non-LTO build: the PC's L3 holds the working set, the 6T's 2 MB does not),
and a heap allocation per cell in `compute_stress` (1.31 s at 1.71 busy cores).

**What changed, all bit-identical** (every change keeps the same operands, operations and order of every f32
addition; the digest below is unchanged on both machines, the JS goldens pass, and each change has its own
bit-identity test against the form it replaced):

- **Stream power, MFD drainage area** (`cartalith-erosion`, `mfd_area_scatter`): the two `powf` per downslope
  neighbour depend only on the fixed fill surface, so they are computed in parallel for a block of the order at a time;
  the scatter replays them serially in the reference's order. PC: 0.30 s to 0.04 s per routing.
- **Stream power, incision and deposition** (`BasinGroups`, `incise_by_basin`, `deposit_by_basin`): the receiver
  forest's trees are independent (`GPU_STREAM_POWER_SCOPE.md` 2.2: the fill order is only a topological order), so each
  tree runs on its own task, in `order` sequence; the dissection cuts trees at pinned cells (3 420 trees, the largest
  5 % of the active cells), the light pass keeps them whole (254 trees, the largest 31 %). Deposition's capacity factor
  `0.005 * area^0.5` is computed once per routing. PC: dissection incision 0.13 s to 0.01 s, light pass 1.06 s to 0.53 s.
- **`MinHeap`** (the fill's heap): paired array and a moving hole instead of pairwise swaps; the same comparisons, so
  the same pop order (`minheap_matches_the_swap_form_pop_for_pop`). About 5 % of the flood in one run on the PC, inside the noise.
- **D8 flow** (`cartalith-hydrology::compute_flow_with`): receivers in parallel beside the radix sort; the accumulation
  alone stays serial. PC: 0.11 s to 0.03 s per call.
- **Glacial pass**: the ice-flux accumulation tree by tree, the erosion term per cell in parallel, the lateral scatter
  serial in order.
- **`compute_stress`**: a fixed array instead of a `Vec` per cell, and its two blurs side by side.
- **Civilisation**: the coast SDF's two jump floods side by side; the capitals' Dijkstras in batches bounded by 64 MiB
  (six at 2048 x 1311, one at 8192^2), merged in settlement order.
- **With one worker** (`RAYON_NUM_THREADS=1`), the MFD and basin paths fall back to the serial loops: measured on the
  6T, staging the work only added memory traffic there (with the fallback the 6T at one worker takes 85.55 s (85.37..86.41) against 86.95 s before; without it, 98.32 s).

**Left serial, and why** (each needs an algorithm change, i.e. a `DECISIONS.md` section 7p ruling, to go parallel):
the priority floods (stream power's fill, 7 per world, about 0.2 s each on the PC and 0.6 s on the 6T; the routing
surface; the water bodies; the active-crater pools; the glacial fill): their pop order decides tie-breaks and so the
world. The in-place jump floods (`assign_plates`, `jfa_dist`): each pass reads cells it has already updated in scan
order, so a ping-pong version gives a different (also valid) answer. The D8 accumulation: summing donors in order
needs per-receiver donor lists, which cost about what they would save.

#### Per stage, before and after (median, min..max, seconds)

| stage | 6T before (6 thr) | 6T after (8 thr) | 6T busy cores before / after | PC before | PC after |
|---|---|---|---|---|---|
| Tectonics | 20.48 (20.36..20.59) | 14.40 (14.31..14.77) | 2.15 / 3.37 | 4.51 (4.47..4.88) | 2.71 (2.67..2.75) |
| Volcanism & impacts | 2.88 (2.81..3.07) | 2.58 (2.41..2.65) | 1.85 / 2.25 | 0.41 (0.40..0.42) | 0.32 (0.31..0.33) |
| Erosion | 7.19 (7.14..7.44) | 3.72 (3.60..4.26) | 1.17 / 2.74 | 1.09 (1.01..1.10) | 0.50 (0.48..0.53) |
| Hydrology | 2.07 (2.04..2.08) | 1.59 (1.55..1.62) | 1.11 / 1.48 | 0.30 (0.30..0.32) | 0.18 (0.17..0.18) |
| Climate (to the end of terrain) | 8.32 (8.12..8.37) | 7.10 (7.00..7.23) | 1.55 / 1.97 | 1.29 (1.27..1.30) | 1.09 (1.07..1.10) |
| terrain total | 41.02 (40.74..41.08) | 29.41 (29.14..29.84) | 1.78 / 2.74 | 7.61 (7.46..7.99) | 4.83 (4.72..4.85) |
| civilisation | 13.06 (12.99..13.17) | 10.45 (10.25..10.50) | 2.74 / 3.77 | 2.88 (2.85..3.17) | 2.34 (2.26..2.39) |
| **total** | **54.09 (53.75..54.19)** | **39.83 (39.64..40.25)** | | **10.47 (10.34..10.90)** | **7.14 (6.99..7.25)** |

5 runs each. The 6T's "after" is the new code at `RAYON_NUM_THREADS=8` (what the thread rule below picks), built before the
one-worker fallback, which does not touch the 8-worker path; the final build at its defaults took 39.19 s (39.14..39.33),
3 runs. At 6 workers the new code took 41.62 s
(41.20..42.24), 4 runs. So the 6T generates **26 % faster**, the PC **32 %**; on the 6T the terrain now keeps 2.74
cores busy on average instead of 1.78.

#### CPU serial, CPU parallel and GPU, per stage (after; median, min..max, seconds)

| stage | 6T serial (1 thr) | 6T parallel | 6T GPU (6 thr) | PC serial (1 thr) | PC parallel (16) | PC GPU (16) |
|---|---|---|---|---|---|---|
| Tectonics | 33.89 (33.78..34.07) | 14.40 (14.31..14.77) | 16.01 (15.89..16.23) | 10.65 (10.38..10.80) | 2.71 (2.67..2.75) | 2.62 (2.59..2.87) |
| Volcanism & impacts | 3.83 (3.82..3.90) | 2.58 (2.41..2.65) | 2.55 (2.55..2.57) | 0.63 (0.59..0.72) | 0.32 (0.31..0.33) | 0.31 (0.31..0.34) |
| Erosion | 7.38 (7.35..7.74) | 3.72 (3.60..4.26) | 4.36 (4.16..4.45) | 1.21 (1.15..1.21) | 0.50 (0.48..0.53) | 0.57 (0.53..0.58) |
| Hydrology | 1.89 (1.87..1.90) | 1.59 (1.55..1.62) | 1.58 (1.56..1.60) | 0.30 (0.30..0.32) | 0.18 (0.17..0.18) | 0.18 (0.18..0.19) |
| Climate | 9.99 (9.53..10.07) | 7.10 (7.00..7.23) | 7.20 (7.11..7.53) | 1.67 (1.63..1.69) | 1.09 (1.07..1.10) | 1.07 (1.05..1.09) |
| civilisation | 28.89 (28.42..29.02) | 10.45 (10.25..10.50) | 11.55 (11.45..11.76) | 8.46 (8.36..8.90) | 2.34 (2.26..2.39) | 2.52 (2.46..2.55) |
| **total** | 85.55 (85.37..86.41) | 39.83 (39.64..40.25) | 43.29 (42.97..43.88) | 23.03 (22.41..23.54) | 7.14 (6.99..7.25) | 7.33 (7.17..7.50) |

Serial: 3 runs each of the final build (with the one-worker fallback; the PC's ran at 5-49 % background load from other sessions, so read it as "no faster, no slower" than before). GPU: 3 runs on the 6T (at its then-default 6 workers; compare 41.62 s, the CPU at 6), 5 on the
PC. Before the change, serial was 86.95 s (86.84..87.05) on the 6T and 22.50 s (22.49..23.51) on the PC; the GPU path
54.57 s (54.56..55.11) and 10.47 s (10.30..10.83).

**Where the GPU helps: nowhere end to end.** On the 6T the GPU path is slower than the CPU at the same worker count
(43.29 against 41.62 s, 4 %, ranges apart), as in 12.2. On the PC (RX 7800 XT, Vulkan) the GPU path's tectonics is
within the CPU's range and its total is slower (7.33 against 7.14 s, ranges overlap at the edges: no gain established).
Per kernel (fed identical CPU inputs, median of 3; table below) the PC's GPU is 1.5 to 25 times faster, the 6T's only
for plate assignment (318 against 1 014 ms) and the base blur (189 against 356 ms) and slower or level for warp,
stress, heterogeneity and weather; but those kernels are not where the stages spend their time, which is the serial
floods above, and opening the device and moving the grids eats the rest. So **no stage is both faster and identical
to the CPU on the GPU**, and Ruling EY's default (GPU off on touch devices) stands; on the PC the GPU path is a
different world for no measured gain. One narrower option is recorded for the owner: plate assignment and the blur
are faster on the 6T's GPU **and** give the same bits on both GPUs measured, so a GPU path restricted to them (CPU
warp, heterogeneity and weather) would be the same world on the PC and the 6T; it is still a different world from the
CPU path (plate assignment differs in 4 cells), so it is a section 7p choice, and it would save about 1 s of 40 on
the 6T before the device's start-up cost. Not built.

#### Why the GPU's world differs between devices

New ignored harness `cartalith-engine/src/gpu_divergence_bench.rs`: it builds the CPU path's own substrate, feeds
every GPU kernel the handsets run **the same CPU-made inputs**, and prints each kernel's output hash, so a difference
cannot be inherited from an earlier GPU kernel. PC (RX 7800 XT) against the 6T (Adreno 630):

| kernel (CPU inputs) | GPU hash, PC | GPU hash, 6T | same on both? | cells unlike the CPU stage | GPU / CPU ms, PC | GPU / CPU ms, 6T |
|---|---|---|---|---|---|---|
| warp (x) | `b6f9c648` | `abb0cb86` | **no** | all (different noise, section 7c) | 6 / 80 | 338 / 328 |
| plate assignment | `6772947d` | `6772947d` | yes | 4 of 2 684 928 | 11 / 279 | 318 / 1 014 |
| stress field | `7b869fb9` | `7b869fb9` | yes | 723 022, at most 4.8e-7 (mask identical) | 24 / 63 | 646 / 544 |
| base-field blur | `d99f0762` | `d99f0762` | yes | 1 196 573, at most 3.0e-7 (bit-identical to its f32 CPU mirror) | 4 / 47 | 189 / 356 |
| heterogeneity | `29a576a9` | `3e7f9860` | **no** | all (different noise, section 7c) | 7 / 26 | 159 / 182 |
| weather | `effcd018` | `3bcde0dd` | **no** | 1.2 M, at most 1.1e-5 | 17 / 26 | 345 / 285 |

Hashes are the first 8 hex digits of each output's FNV-1a; median of 3; the CPU column on the 6T is at 6 workers.

Reading it: **warp and heterogeneity** use the GPU-safe f32 noise (`DECISIONS.md` section 7c), so they differ from the
CPU everywhere by design; **the three kernels that differ between the devices are exactly the three whose arithmetic is floating-point
heavy** (the value-noise interpolation `a*(1-u)*(1-v) + ...` and `f32(h) / 4294967295.0` in warp and heterogeneity;
`exp` and divisions in weather), while the three that agree use comparisons, integer arithmetic and plain sums in a
fixed per-cell order (plate assignment, stress, the blur, which also equals its CPU f32 mirror bit for bit). That
pattern points at the driver's code generation, not at any race; it is inferred from which kernels differ, not traced
instruction by instruction. The CPU path, by contrast, printed the same digest on the PC and the 6T in
every run. The mechanisms the WGSL leaves to the driver are the usual three: **FMA contraction** (`a * b + c` may be
fused or not; WGSL permits either and has no `precise` qualifier), **division and `exp` precision** (f32 division is
specified to 2.5 ULP, `exp` to 3 + 2|x| ULP, and the shaders divide in the hash-to-unit step and the weather kernel),
and **reduction order** (none here: every kernel writes its own cell, and the blur's sums are per cell). A one-ULP
difference in a warp or a stress value then moves a plate boundary cell, a fill tie-break or a channel head, and the
priority floods and erosion amplify it into a different world (63 ways on the Pad, 64 on the 6T, 67 on the PC).
**Making the GPU path identical across devices would mean integer or fixed-point arithmetic in every kernel** (the
noise lattice in integers, the interpolation in fixed point): a section 7p change that would also change every GPU
world, for a path that measured no gain. Not proposed; the CPU path is the cross-device-identical one.

#### The thread rule for big.LITTLE (`cartalith_engine::auto_thread_count`)

**Rule: "auto" builds the pool with every core the process may be scheduled on, whatever its type**, read from
`/proc/self/status`'s `Cpus_allowed_list` on Linux and Android when it is larger than `available_parallelism()`;
elsewhere, `available_parallelism()`. A user setting and `RAYON_NUM_THREADS` still win. No per-device table, no
big/little split.

- **Why not `available_parallelism()` alone:** on the 6T it read 6 in 35 of 36 harness starts on 2026-11-15 and 8 in one, although
  `Cpus_allowed_list` is `0-7` and all eight cores are online: the Snapdragon kernel parks cores at idle and
  `sched_getaffinity` leaves parked cores out. Under load the kernel unparks them.
- **Measured on the 6T with the new code:** 8 workers 39.83 s (39.64..40.25), 6 workers 41.62 s (41.20..42.24):
  4 % faster, ranges apart. (In 12.2, before the stages were parallel, 8 workers made no established difference.) The
  final build with no environment set chose 8 workers by this rule while `available_parallelism()` read 6, and took
  39.19 s (39.14..39.33), 3 runs, same digest.
- **Why no big/little weighting:** rayon's work stealing gives a slower core fewer chunks, and the serial stages run on
  the calling thread, which the scheduler keeps on a fast core (12.4: the 6T's big cores carried 77-78 % of the
  ticks). A pool smaller than the cpuset would leave cores idle in the parallel stages; a larger one would only add
  switching.
- **For the Pad 3 and other devices:** the Pad reports 8 of 8 (12.4), so its pool is unchanged at 8. A phone the
  system has put in a smaller cpuset (a background app) gets that cpuset's size. Re-measure on the Pad with the
  commands in 12.5; its sustained throttling (12.4, point 3) is the thing to watch, since more busy cores means more
  heat.
- **In the app:** `ensure_thread_pool` applies the rule when generation is the first thing to use the pool. Rayon's
  global pool is built once per process, and the shell can reach a `par_iter` (a render, a sample) before the first
  generation, in which case rayon's own default (`available_parallelism()`) is what runs. The harness itself hit this:
  its `CONFIG` line's `rayon::current_num_threads()` built the pool first, so a 6T run without `RAYON_NUM_THREADS` ran
  6 workers until the harness was changed to call `ensure_thread_pool()` first. Routed as a row in
  `OUTSTANDING_WORK.md`.

#### Block size of the MFD pass

Measured on the 6T at 8 workers with a temporary environment override (non-LTO build, not committed, 3 interleaved
runs each): 32 768 cells per block 40.40 s (40.27..40.60), 4 096 44.07 s (44.01..44.67), 1 024 51.38 s
(50.47..53.23). Smaller blocks cost more rayon dispatches than the scratch saves, so `MFD_AREA_BLOCK` stays 32 768 (its
doc comment and `mfd_area_block_scratch_is_three_point_four_megabytes` say so). At one worker the 6T ran the new code
**slower** than the old (98.32 s (97.82..99.03) against 86.95 s, tectonics 46.06 against 35.02 s): staging the MFD
weights and grouping the trees only adds memory traffic when nothing runs beside them. Both paths therefore fall back
to the serial loops when the pool has one worker; see the serial column above for the final build.

#### Reproducing

```
cargo test --release -p cartalith-godot --lib device_stage_bench -- --ignored --nocapture --test-threads 1
cargo test --release -p cartalith-engine --lib gpu_divergence_bench -- --ignored --nocapture --test-threads 1
cargo ndk -t arm64-v8a test --release -p cartalith-godot -p cartalith-engine --lib --no-run
```

On the handset, push both test binaries to `/data/local/tmp` and run each with the same arguments (`SNAP_GPU=1` for the
GPU path, `RAYON_NUM_THREADS=1` for serial). Determinism: every CPU run here, before and after, on the PC and the 6T, at
1, 6, 8 and 16 workers, printed `terrain=632b22edf25908e1 civ=65905050eba0a3e3` (40 settlements, 60 ways; the civ
digest moved from 12.3's `f6708d1a30982bb1` with `32d032fd`, Ruling EX's town placement, before this work). The GPU path
printed the same digest before and after on each device: `terrain=dcdd2bce8534e95e civ=f23e16afb93fbc19` (67 ways) on
the PC, `terrain=14fa38faada65f7a civ=a06b729d7654569a` (64 ways) on the 6T.

### 12.7 The OnePlus Pad 3 with the 2026-11-15 build (2026-11-15): cold start measured; timed by the coordinator (a reduced matrix, below)

**Status: half done, and the missing half is a blocker, not a result.** `OUTSTANDING_WORK.md`'s "Re-measure generation on
the Pad 3" asked for before/after, 5 interleaved runs per column at the default thread count and at
`RAYON_NUM_THREADS=1`, with thermal readings, the CPU digest and the app's cold-start worker count. Only the last was
measured. **There are no Pad timings, no Pad digest and no throttling data in this section, and none are inferred from
the 6T or the PC.**

**What happened.** Both test binaries were built (release with LTO, `cargo ndk -t arm64-v8a test --release -p
cartalith-godot --lib --no-run`, each in its own worktree and `CARGO_TARGET_DIR` under `C:/cw/`: "before" `2e9fd4db`,
"after" `origin/main` `5721443c`; 33 131 664 and 33 322 632 bytes). The Pad was reachable over wifi adb at
`192.168.1.22:41753` for the cold-start check, then dropped off adb while the two builds ran (about 25 minutes): from the
PC the host answered ping, port 41753 refused connections, a scan of 30000-50000 found two open ports (36631, 46888)
that both reported `offline` to `adb connect`, `adb mdns services` listed nothing, and eight retries over three minutes
did not bring it back. Wireless debugging's port rotates when it is toggled or the tablet sleeps, and re-enabling it
needs the tablet in hand, so nothing was pushed to the Pad and no benchmark process ran on it.

**Cold start: the app runs 8 Rayon workers on the Pad 3** (the one result of this pass). The installed build
(`107d5465`; `org.cartalith.walkingskeleton`) was force-stopped, launched with `monkey -p ... -c
android.intent.category.LAUNCHER 1`, and its threads read from `/proc/<pid>/task/*` 25 s after launch (no `run-as`, no
app data touched; temperatures 38-41 degC on the CPU sensors beforehand, `Thermal Status: 0`, plugged in). Two groups of
eight threads exist, and only one is Rayon's:

- `WorkerThread_0` ... `WorkerThread_7`, eight consecutive tids: **Godot's own `WorkerThreadPool`** (its threads carry a
  numbered name; Godot sizes it from the core count, 8 here whatever Rayon does).
- **Eight threads named `GLThread <n>` with one identical start tick (79409270 in `/proc/<pid>/task/<tid>/stat` field 22,
  tids 29135-29137, 29158-29161, 29173)**: Rayon's global pool. Rayon starts unnamed workers, which on Linux inherit the
  creating thread's name, and the creating thread is Godot's render thread (`GLThread`). Another `GLThread`-named thread
  (tid 29103) started 145 ticks earlier and is not part of the group. Rayon's other pool in the app, `lod_worker`'s,
  names its threads `cartalith-lod-<i>` and was not yet built.

So the thread rule of 12.6 gave 8 on the Pad (`auto_thread_count` reads `Cpus_allowed_list` `0-7`, and the pool is the
`CONFIGURED_THREADS = 0` auto case), the same as `available_parallelism()` would. The count is read from the thread layout,
not from `cpu_thread_count_active()`, which the app does not log.

**A correction to the 6T entry of the same day.** `STATUS.md`'s "App builds the thread pool from the big.LITTLE rule at
boot" counted "8 consecutive tids spawned right after Godot's setup" on the 6T. On the Pad the 8 consecutive tids are
Godot's `WorkerThread_N`, which are 8 on any 8-core device regardless of Rayon, so that evidence would not have told 6
from 8. The 6T count should be re-read by start tick and name as above before it is relied on.
**Timing, run by the coordinator 2026-11-15** (the owner re-enabled wireless debugging and asked for it quickly, so the matrix was cut to **3 interleaved rounds at the default thread count, 30 s cooldown**; the `RAYON_NUM_THREADS=1` column was not run). The same two binaries (`2e9fd4db` "before", `5721443c` "after", LTO release), `device_stage_bench`, `SNAP_REPS=1 SNAP_WARM=0`. Median (min..max), seconds:

| stage | before | after |
|---|---|---|
| Tectonics | 7.77 (7.68..7.83) | 5.78 (5.76..5.84) |
| Volcanism | 0.96 (0.90..1.34) | 0.80 (0.78..0.85) |
| Erosion | 1.85 (1.85..3.33) | 1.04 (1.04..1.05) |
| Hydrology | 0.83 (0.68..1.02) | 0.44 (0.43..0.45) |
| Climate | 3.83 (2.82..3.89) | 2.59 (2.49..2.59) |
| Civilisation | 8.33 (6.53..8.61) | 5.02 (5.00..5.02) |
| **Total** | **23.76 (20.62..25.67)** | **15.69 (15.57..15.70)** |

About 34 % faster; terrain busy cores 1.80-1.92 -> 2.71-2.84. **Digest on every run, both builds: `terrain=632b22edf25908e1 civ=65905050eba0a3e3`**, the PC's and the 6T's. The "before" column's spread (round 2 is 25 % slower than round 1) is the Pad's own variance and is why the medians, not single runs, are quoted. Thermal: the hottest `CPU*` sensor in `dumpsys thermalservice` read 41.3 C before and after every run, a flat reading over ~4 minutes of load. The owner notes the Pad 3 has a vapour chamber (2026-11-15), so the flat reading may be accurate rather than a slow sensor; sustained throttling (section 12.4 point 3) over a longer run is still unmeasured. Device files removed afterwards.

**Re-read 2026-11-15 by the coordinator (6T, pid 11140, APK `107d5465`):** the 6T's tids 11280-11287 are named `GLThread 32` (not `WorkerThread_N`) and share one start tick (3323026), separate from Godot's `WorkerThread 0..7` and from the two other `GLThread`-named threads (11178, 11191, distinct ticks). So the 6T's pool is 8 Rayon workers; the original count named the right threads, only its stated evidence (consecutive tids) was too weak.

**To finish it (the matrix is ready).** `C:/cw/pad_bins/` holds `pad_before`, `pad_after` and `pad_matrix.sh` (a
device-side driver: 5 rounds of before and after at the default thread count and at `RAYON_NUM_THREADS=1`, a 60 s
cooldown before each run, the highest CPU temperature and `Thermal Status` from `dumpsys thermalservice` before and after
each run, one `SNAP_REPS=1 SNAP_WARM=0` process per run). With the Pad on adb and the app force-stopped:
`adb push` the three files to `/data/local/tmp`, `chmod 755`, `nohup sh pad_matrix.sh &`, and read `out_*_<round>.txt` and
`thermal.txt` when `ALLDONE` is the last line (about 30 minutes). The digest to expect is `terrain=632b22edf25908e1
civ=65905050eba0a3e3` unless commits since 12.6 moved the civ half (`ed6e3b96`'s riverbank change is a 7p layout switch;
compare against a PC run of the same commit).

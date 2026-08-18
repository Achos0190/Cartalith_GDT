# Hardware Acceleration & Adaptive Compute Architecture

> **The earliest of three owner-supplied architecture documents, in lineage
> order: this one** (2026-08-16) → `TERRAIN_ARCHITECTURE_RESEARCH.md` (v1.0,
> tiling/LOD/clipmaps) → `HETEROGENEOUS_COMPUTE_RESEARCH.md` (v3.0, which
> explicitly integrates both). The later two supersede much of what follows.
> Kept because it is the owner's original stated intent and because the scope
> correction recorded below — static one-shot generation, not continuous
> simulation — is the single most load-bearing fact about all three.
>
> **What was actually built from this line of research**:
> `GPU_LAYER_INTEGRATION_SCOPE.md` (nine milestones) and
> `GPU_COMPUTE_PILOT_SCOPE.md` before it. No capability-tier classifier,
> diagnostics panel, telemetry system or adaptive scheduler exists, and none
> is scheduled.

**Status: proposed, not yet scoped or implemented.** Supplied by the owner
2026-08-16 as a `/goal` directive; the command itself failed to register
(26,845 characters against the harness's 4,000-character `/goal` limit), so
this is **not** an active, hook-enforced directive — it's preserved here
verbatim as the owner's stated intent, pending a proper scoping pass.

This project's own working discipline (`README.md`, `MVP_SCOPE.md`,
`ROADMAP.md`) is to scope a new phase of work in its own document — what it
covers, what it explicitly doesn't, how "done" is measured — before writing
implementation code against it, the same way `MVP_SCOPE.md` did for Phase 1
before any Rust was ported. This document is the owner's raw input for that
scoping pass, not the scope itself. See the assistant's response in the
conversation this was captured from for the open questions that need
resolving before implementation starts (renderer-independence of a
standalone `wgpu` compute pipeline vs. Godot's own `RenderingDevice`,
absence of a currently-measured performance problem, interaction with
`DECISIONS.md` §7's tolerance-based golden-parity discipline, and the sheer
size of this spec relative to any single work session).

**Major scope correction, owner-confirmed 2026-08-16**: Cartalith generates
a **static map from a one-shot batch simulation**, not a continuously
recomputing interactive application. `generate()` runs once per
seed/parameter change and produces a final result; there is no per-frame
simulation tick. This significantly narrows what several sections below
actually require in practice — read them as describing the general
principle, not literally the runtime shape needed here:

- §16 (async GPU work), §32-34 (UX/priority model): the concern that
  matters is "don't freeze the UI during the one `generate()` call" — this
  port already does that today (a background `Thread` in `main.gd`, no
  GPU involved yet). The elaborate continuous-scheduler/priority-queue
  machinery these sections describe (competing background jobs, frame-by-
  frame budget) is not needed for a single batch job that runs, completes,
  and hands back a static result.
- §21 (thermal/mobile adaptive scheduling): matters far less for an
  occasional multi-second batch computation than for sustained per-frame
  load — revisit only if real device testing shows it matters, don't
  build it speculatively.
- §14/§15 (GPU memory pooling, minimize CPU↔GPU transfers): **still fully
  applies, arguably more cleanly** — a one-shot pipeline should keep
  intermediate fields resident on GPU across every stage that can consume
  them (terrain → climate → erosion → hydrology → civ affordance layers,
  wherever each stage is actually GPU-resident) and read back to CPU only
  the final result(s) that Godot/`cartalith-civ`/save-export actually need
  — not because of a continuous-frame budget, but because upload/download
  bandwidth is pure overhead on a single run with no amortization across
  frames to hide it in.
- §9/§27 (GPU self-test, failure fallback): still fully applies, but runs
  *once* (at first GPU use, not every frame) — cache the result for the
  process lifetime, no need for elaborate cache-invalidation machinery
  (§30) beyond "did the GPU/driver change since last successful run."

The practical shape this implies: an efficient **one-shot GPU pipeline**
per `generate()` call — CPU orchestrates, each GPU-suitable stage runs as
a batch compute dispatch keeping data resident on GPU, CPU-only stages
(graph/sequential algorithms — flow accumulation, priority-flood, Dijkstra/
MST road networks — see the per-layer feasibility read below) interleave
via minimal, deliberate readback points, not a round-trip per stage.

---

## Cartalith — Hardware Acceleration & Adaptive Compute Architecture

### 1. Objective

Cartalith must be designed as a hardware-aware, GPU-accelerated computational application.

The application must make active use of the host device's available CPU, GPU, memory and compute capabilities to provide a smooth interactive experience while maintaining deterministic simulation correctness.

Do not implement Cartalith as a CPU-first application that happens to have optional GPU rendering.

The intended architecture is:

```
                     CARTALITH APPLICATION
                              |
                    Rust Simulation Core
                              |
                  Hardware Capability Layer
                              |
             +----------------+----------------+
             |                                 |
       GPU Compute Path                  CPU Compute Path
             |                                 |
           wgpu                             Rayon
             |                                 |
    +--------+--------+                       |
    |        |        |                       |
 Vulkan     DX12    Metal/GL              Multicore CPU
    |        |        |
    +--------+--------+
             |
       GPU Compute Device
```

The GPU path should be preferred whenever the hardware is capable and the workload benefits from GPU execution.

The CPU path must remain fully functional as a correctness-preserving fallback.

### 2. Core Technology Requirement

Use Rust + `wgpu` as the hardware abstraction layer for GPU rendering and GPU compute.

Do not make Vulkan the application's primary abstraction layer. Vulkan is a backend. `wgpu` is the portability layer.

```
Cartalith Rust code
        |
      wgpu
        |
 +------+---------------+
 |      |               |
Vulkan DX12           Metal
 |      |               |
Windows Android      Apple
Linux
```

`wgpu` currently exposes Vulkan, Metal, Direct3D 12 and OpenGL/OpenGL ES backends, among others. Vulkan is supported on Windows, Linux and Android; Direct3D 12 is supported on Windows; Metal is used on Apple platforms; and the GL backend covers OpenGL/OpenGL ES environments.

The implementation must therefore avoid platform-specific GPU code wherever `wgpu` provides an adequate abstraction. Only introduce direct Vulkan/DX12/Metal APIs if a specific Cartalith requirement cannot reasonably be implemented through `wgpu`.

### 3. Hardware Detection Must Occur at Runtime

At application startup, Cartalith must inspect the available compute hardware.

Use `wgpu::Instance` and `Instance::request_adapter()` to discover suitable GPU adapters.

A `wgpu::Adapter` represents a physical graphics/compute device and exposes adapter information, supported features, supported limits, device creation, and surface compatibility via APIs such as `adapter.get_info()`, `adapter.features()`, `adapter.limits()`, `adapter.request_device(...)`.

This capability information must be used to determine the appropriate execution tier. Do not simply assume "GPU exists" = "GPU compute is suitable" — a device must be tested and classified.

### 4. Hardware Capability Tiers

Create an explicit hardware capability model, e.g.:

```rust
enum ComputeTier {
    CpuOnly,
    GpuBasic,
    GpuStandard,
    GpuHigh,
}
```

The exact names may differ, but the concept must exist. Classification should consider at minimum: GPU availability, backend, GPU vendor, GPU device name, device type, supported compute features, maximum buffer sizes, storage-buffer capabilities, workgroup limits, texture/storage capabilities, available device limits, memory pressure, CPU core count, system memory, mobile vs desktop characteristics.

Do not classify devices using GPU model-name string matching alone — capability detection must be based primarily on actual `wgpu` adapter capabilities.

### 5. Prefer Hardware Adapters

When requesting an adapter, use an appropriate `PowerPreference`. Prefer a high-performance adapter for computationally intensive workloads where appropriate.

`wgpu::RequestAdapterOptions` provides `power_preference` and `force_fallback_adapter`. A fallback adapter generally represents a software implementation and must not accidentally become the normal execution path.

```
Normal startup -> Request hardware adapter -> GPU available?
    YES -> capability test
    NO  -> CPU/Rayon
```

Do not deliberately request a fallback/software adapter unless explicitly running in a diagnostic mode.

### 6. GPU Compute Is a First-Class Cartalith Subsystem

GPU compute must not be limited to visual rendering. Priority candidates: heightfield generation, fractal noise, plate/terrain field generation, orogeny synthesis, hydraulic erosion, flow accumulation, hydrological field generation, water-depth calculations, climate field generation, temperature calculations, rainfall calculations, moisture transport, biome classification, large raster transforms, hillshade/AO generation, terrain material classification, large-scale map processing, LOD/tile generation, other embarrassingly-parallel raster operations.

Do not blindly move every operation onto the GPU. GPU acceleration should be used where the workload has sufficient parallelism to justify dispatch overhead, buffer upload/download, synchronization, pipeline creation, and GPU memory usage. Small operations should remain on the CPU when that is demonstrably faster.

### 7. GPU/CPU Execution Must Be Workload-Aware

```
ComputeBackend
    |
    +-- CpuBackend
    |
    +-- GpuBackend
```

Higher-level simulation code should not contain platform-specific GPU logic. The simulation system must not need to know whether it is currently executing on Vulkan, DX12, Metal, OpenGL ES, or CPU/Rayon. This separation is mandatory.

### 8. CPU Fallback Must Remain Correct

The CPU implementation is not a degraded or experimental implementation — it is the reference implementation. The GPU implementation must produce results that satisfy the same numerical and semantic contract as the CPU implementation. Where floating-point execution order makes exact bit-for-bit equality impractical, define explicit numerical tolerances and document them.

GPU acceleration changes execution strategy, NOT simulation meaning. A machine without a usable GPU must still be capable of generating a valid Cartalith world.

### 9. GPU Self-Test

After selecting a GPU adapter and creating a `wgpu::Device`, execute a small GPU self-test before enabling the production GPU path: create a minimal compute pipeline, allocate a small storage buffer, execute a known deterministic calculation, read the result back, compare against a CPU reference result, verify GPU execution is stable, only then enable GPU compute.

This prevents driver/device combinations that technically expose GPU compute but are unsuitable for Cartalith from destabilising the application.

### 10. Device Creation Must Be Conservative

Do not request unnecessarily aggressive `wgpu` limits. `wgpu::Limits` documentation recommends starting with the most restrictive limits necessary and increasing only when required. Do not use `Limits::unlimited()` in production. Do not request optional features merely because the GPU supports them — only request features and limits actually required by the workload.

### 11. Android Hardware Acceleration

Android is a first-class Cartalith target. On Android, prioritize Vulkan through `wgpu` (Android's primary low-level graphics API, available from API level 24, with documented CPU-overhead/performance advantages over OpenGL ES for suitable workloads).

```
Android device -> wgpu -> Vulkan -> Adreno / Mali / other GPU -> GPU compute
```

If Vulkan is unavailable or unsuitable, fall back to an alternative `wgpu` backend if suitable, else CPU/Rayon. Do not make Android GPU acceleration dependent on a specific GPU vendor. Android also exposes `android.hardware.vulkan.compute` for apps requiring hardware-accelerated Vulkan compute.

### 12. Windows Hardware Acceleration

On Windows, allow `wgpu` to select between appropriate native backends (normally Vulkan or DX12, discrete or integrated GPU). Do not hard-code Vulkan if DX12 provides a better or more compatible execution path on the current machine. Selection should consider adapter availability, capabilities, stability, device tier, and workload requirements. The backend actually selected must be visible in diagnostics.

### 13. Desktop Integrated vs Discrete GPU

Where multiple GPUs are available, inspect the adapters rather than blindly selecting the first one returned. For compute-heavy generation tasks, prefer a high-performance physical GPU when configured for performance, but do not assume the discrete GPU is always appropriate. Consider adapter type, power preference, capabilities, available memory, workload, and device tier. The user should ultimately be able to see which adapter Cartalith selected.

### 14. GPU Memory Management

Do not continuously allocate and destroy GPU buffers during simulation. GPU resources must be pooled/reused where possible (allocate -> reuse -> rewrite -> reuse, not allocate -> compute -> destroy repeated). Large Cartalith fields (heightField, temperatureField, rainField, flowField, moistureField, erosionField, biomeField) should use persistent GPU storage buffers or textures depending on access pattern.

### 15. Avoid Unnecessary GPU <-> CPU Transfers

This is critical. Do not ping-pong CPU/GPU for every simulation stage. Prefer: upload initial data once, run terrain/erosion/hydrology/climate/biome/derived-fields on GPU in sequence, read back only required results. Keep intermediate fields on the GPU when several successive operations can use them; only synchronize/read back when the CPU genuinely needs the data.

### 16. Asynchronous GPU Work

Do not block the UI thread while waiting for GPU computation. GPU work must be scheduled asynchronously; the application must remain responsive while large terrain/simulation workloads execute. The UI must never freeze simply because a terrain generation job is running.

### 17. Workgroup and Dispatch Configuration

Compute shaders must use hardware-appropriate workgroup sizes — do not hard-code an arbitrary size without checking device limits. Provide tunable compute configurations for workgroup dimensions, tile dimensions, batch sizes, dispatch sizes, staging buffers, with sensible defaults based on detected hardware tier.

### 18. Tile-Based Computation

Large Cartalith worlds must not require the entire world to be processed simultaneously if this exceeds practical GPU memory. Implement tiled/chunked processing where appropriate, integrated with Cartalith's existing LOD, tile baking, chunk streaming, quadtree, raster storage, and rendering pipeline architecture.

### 19. CPU Parallelism

When GPU execution is unavailable or inefficient, use Rayon for embarrassingly parallel CPU workloads. Do not create uncontrolled thread pools — use a bounded scheduler respecting available cores, and avoid monopolising every core during interactive editing (interactive workloads take priority over background generation).

### 20. Dynamic Hardware Utilisation

Cartalith should not use a fixed performance profile on every machine — the runtime should adapt: high-end desktop GPU gets large batches/high-res compute/aggressive parallelism/larger tiles; mid-range laptop gets moderate batches/controlled memory/adaptive LOD; mobile GPU gets smaller batches/lower transient memory/thermal-aware scheduling/aggressive tiling; no usable GPU gets bounded Rayon CPU execution. The objective is maximum useful performance while preserving responsiveness and stability, not maximum benchmark throughput.

### 21. Thermal and Mobile Considerations

Mobile devices must not be treated as miniature desktop computers. GPU acceleration can increase power consumption, heat, thermal throttling, and battery drain. Android execution should use adaptive workloads — long-running generation divided into manageable jobs rather than enormous uninterrupted dispatches. The system should be capable of reducing dispatch size, tile size, concurrency, generation resolution, and background workload priority when necessary. Do not implement thermal APIs unless required by the target platform; first design the scheduler so it can adapt to performance feedback.

### 22. Rendering and Simulation Must Share the Hardware Strategy

Do not build two unrelated GPU systems. The same `wgpu` hardware abstraction should support rendering and compute where practical, sharing adapter selection, device, queue, resource lifetime management, GPU memory management, diagnostics, and backend selection. Avoid multiple independent GPU devices for the same application without a compelling technical reason.

### 23. Diagnostics

Cartalith must expose a hardware diagnostics panel showing at minimum: Compute Backend, GPU, Vendor, Device, Device Type, Backend, GPU Compute (enabled/disabled), VRAM/memory information, CPU Cores, Execution Tier, GPU Self-Test result, Current Compute Path. This information is essential for debugging performance reports.

### 24. Performance Telemetry

Create an internal performance monitor tracking (at minimum) GPU dispatch time, CPU compute time, GPU->CPU transfer time, CPU->GPU upload time, GPU memory allocation/reuse, queue submission time, frame time, simulation time, tile processing time. Do not expose excessive telemetry in the normal UI — provide it through a developer/debug mode.

### 25. Adaptive GPU/CPU Switching

Individual workloads should choose their own execution path (e.g. height generation/hydrology/biome classification -> GPU; small settlement calculation/pathfinding/UI logic -> CPU). Do not enforce "GPU = everything." The rule: GPU where massively parallel and worthwhile, CPU where latency, branching, synchronization, or workload size makes CPU preferable.

### 26. Do Not Sacrifice Determinism for Performance

Performance optimisation must not silently alter world generation. Maintain the existing deterministic-seed architecture — for a fixed seed/parameters/dimensions/generation version, the result must remain stable per the project's defined numerical/deterministic contract. GPU optimisation must be tested against CPU reference output (heightField, temperatureField, rainField, flowField, biomeField for regression comparison). If GPU floating-point differences are unavoidable, define explicit acceptable tolerances and ensure they cannot cascade into uncontrolled categorical differences.

### 27. GPU Failure Handling

GPU failure must never crash Cartalith. On adapter/device creation failure, missing required feature/limit, self-test failure, shader compilation failure, device loss, GPU memory allocation failure, or backend instability: record the diagnostic reason, invalidate the GPU compute backend, requeue the current operation, fall back to CPU/Rayon execution, and continue. Do not force the user to restart merely because GPU computation failed.

### 28. Shader Architecture

GPU compute shaders must be treated as part of the computational engine, not ad-hoc rendering code. Organise as e.g. `shaders/{terrain,erosion,hydrology,climate,biome,rendering,common}/`. Centralise shared mathematical functions. Avoid duplicating mathematical definitions between CPU and GPU implementations — where practical, define the algorithm once conceptually and implement equivalent CPU/GPU kernels from the same specification.

### 29. Backend Independence

Never branch application logic on `if vulkan {...} else if dx12 {...}` except for backend diagnostics or a genuine backend-specific limitation. Normal Cartalith compute code must target `wgpu`, not Vulkan/DX12/Metal/OpenGL directly. This lets the same architecture migrate across Windows/Linux/Android/macOS/future platforms without rewriting the simulation engine.

### 30. Hardware Capability Cache

After successful hardware detection, cache the resulting capability profile locally. Invalidate on application version change, GPU driver change, backend change, device change, compute self-test failure, or shader version change. Do not permanently assume a GPU is safe merely because it was safe during a previous run.

### 31. Required Startup Sequence

```
START -> Initialize Rust core -> Enumerate hardware -> Create wgpu Instance
      -> Request suitable hardware Adapter -> Inspect adapter
      -> Classify hardware tier -> Request minimal required Device
      -> Run GPU compute self-test
           PASS -> GPU path enabled
           FAIL -> CPU/Rayon path enabled
      -> Initialize scheduler -> Initialize rendering
      -> Initialize simulation compute -> START APPLICATION
```

The UI should be available as quickly as possible. Large hardware tests must not unnecessarily delay startup.

### 32. User Experience Requirement

The hardware system exists to produce a smooth application, not merely impressive benchmark numbers. The user should be able to pan/zoom/paint/edit/inspect/modify parameters/switch views/interact with the map while background generation occurs. Long computations must not block the interface. Generation jobs should be interruptible/cancellable where technically possible; when parameters change mid-generation, cancel/deprioritise the old job rather than letting it overwrite newer user state.

### 33. Priority Model

Suggested priority order: 0 UI/input, 1 visible viewport rendering, 2 interactive map edits, 3 visible simulation calculations, 4 background world generation, 5 precomputation/cache generation. A background world-generation task must never make the UI unusable merely because the GPU or CPU is technically capable of processing it faster.

### 34. Performance Goal

The target is **not** "use the GPU everywhere." The target is: use the best available hardware execution path for each workload while maintaining deterministic simulation behaviour, bounded memory usage, UI responsiveness, and reliable fallback behaviour. Powerful hardware -> use it. Weaker hardware -> adapt. No usable GPU -> multicore CPU. GPU failure -> recover automatically. High workload -> tile, schedule, stream. Interactive user action -> always remains responsive.

### 35. Implementation Constraints

Rust remains the core simulation language; `wgpu` is the primary GPU abstraction; Vulkan supported via `wgpu` where available; DX12 available on Windows via `wgpu`; Metal available for Apple targets via `wgpu`; GL/OpenGL ES as a compatibility backend where appropriate; Android prioritises Vulkan; CPU/Rayon always available as fallback; GPU availability detected at runtime; adapter capabilities inspected; features/limits requested conservatively; GPU compute undergoes a startup self-test; GPU failure automatically falls back to CPU; GPU/CPU implementations share a defined numerical/simulation contract; GPU-to-CPU transfers minimised; GPU resources reused/pool-managed; large workloads support tiling/chunking; long-running computation never blocks the UI; workloads scheduled by priority; hardware diagnostics available; performance telemetry available in developer mode; architecture remains backend-independent; no hard-coded GPU vendors; no GPU model-name string matching as the primary capability mechanism; no hard Vulkan dependency where `wgpu` suffices; no `Limits::unlimited()` in production; GPU acceleration not silently disabled for implementation simplicity; GPU hardware not silently required; simulation correctness never compromised for GPU performance.

### 36. Technical References

`wgpu::Adapter`, `wgpu::RequestAdapterOptions`, `wgpu::DeviceDescriptor`, `wgpu::Limits`, `wgpu::Backend`, Android Developers' Vulkan overview/requirements, Android Vulkan compute hardware feature declaration. Treat as implementation references, not suggestions — verify exact APIs against the `wgpu` version actually selected, since its APIs evolve between major releases.

### 37. Final Architectural Principle

Cartalith should not ask "does this computer have a GPU?" It should ask "what compute hardware is available, what can it actually do, what is the most efficient execution strategy for this workload, and how can I use it without compromising correctness or responsiveness?" That principle should govern the entire Rust compute architecture, scaling from a low-end CPU-only machine through multicore desktop CPU, integrated GPU, mobile GPU, to a high-end desktop GPU, without requiring separate simulation architectures for each class of machine.

Hardware capability determines execution strategy. The simulation model remains the same.

## Real-world findings so far (updated as GPU/CPU work actually lands)

**CPU multithreading was the real, immediate gap, not GPU.** Checked
2026-08-16 (owner's own observation: "currently on my system it doesn't
seem to fully use the cpu"): confirmed `rayon` was never added as a
dependency and every per-cell loop across the entire pipeline ran on a
single thread, on a 16-logical-core machine. `CPU_MULTITHREADING_SCOPE.md`
covers this — and unlike GPU work, it needs none of §7a's carve-out,
since parallelizing an existing per-cell loop changes nothing about the
math, only execution order across already-independent cells. Given GPU
milestone 6 found the dedicated-GPU path currently *loses* to CPU below
2048² (per-call context-creation overhead), CPU parallelism is plausibly
the higher-value near-term win, not a consolation prize.

**Using the integrated GPU alongside the dedicated one — a real idea,
not yet scoped.** Also raised 2026-08-16: this machine (and presumably
many real target machines) has both a dedicated and an integrated GPU.
`cartalith-gpu` currently only ever requests one `PowerPreference::
HighPerformance` adapter — correctly picks the dedicated GPU (confirmed
by the pilot's own results), but the integrated GPU is never enumerated
or used for anything, even as a secondary resource for smaller/latency-
tolerant work running alongside the dedicated GPU's main pipeline. This
is architecturally consistent with §13's own "desktop integrated vs
discrete GPU" section, which already anticipates inspecting multiple
adapters rather than blindly picking one — but actually building
multi-adapter workload splitting is a genuinely more complex undertaking
than anything implemented so far (adapter enumeration, capability-based
workload assignment, avoiding the two GPUs contending over the same PCIe/
memory bandwidth in ways that net out worse than just using one well).
Deliberately not scoped yet — CPU multithreading and finishing out the
single-dedicated-GPU pipeline (context reuse, per `GPU_LAYER_INTEGRATION_
SCOPE.md`'s own milestone 6 finding) are the higher-value next steps.
Revisit this once those are real and the actual remaining headroom is
clearer, not before.

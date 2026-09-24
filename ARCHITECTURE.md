# Architecture

## The split

Rust owns engine state and logic. Godot owns rendering, UI, input, and packaging.

- Rust exposes a few GDExtension classes via `gdext` — a `WorldGen` with
  `generate(seed, width_km, resolution)` and accessors returning fields. Godot
  calls in and draws what comes back.
- Godot computes nothing beyond layout. Anything you could get numerically wrong
  belongs in Rust.
- Rust never touches the scene tree. Communication runs Godot → Rust, which keeps
  the engine testable under `cargo test` with Godot absent.

The HTML app already separates its engine block from its UI blocks (root
`CLAUDE.md`). This makes that convention a compiler-enforced boundary.

**Two failure modes it prevents.** Logic drifting into GDScript reproduces the
"two functions answering one question" problem the HTML CHANGELOG records hitting
repeatedly. Engine code depending on Godot types (`Vector2`, `Color`) would make
the engine untestable without booting Godot and would foreclose a future WASM
target (`DECISIONS.md` §2). Engine crates use plain Rust types; the boundary
layer converts at the edge.

## Crate layout: one crate per subsystem

A Cargo workspace of small crates, each independently compilable and testable.
Dependencies run one way, roughly in pipeline order. The one exception is
deliberate and test-only: `cartalith-gpu`'s dev-dependencies on
`cartalith-engine` and `cartalith-civ` close a **dev-dependency cycle**
(civ → engine → gpu → civ), which Cargo permits; `cartalith-gpu`'s
`Cargo.toml` says why. Its normal `[dependencies]` stay acyclic.

```
cartalith-native/                 (workspace root, new repository)
├── crates/                       16 crates; internal [dependencies] in brackets
│   ├── cartalith-jsmath/         Math.hypot/exp/sin/cos/log/atan2/round/toFixed
│   │                             with V8's semantics — no dependencies at all
│   ├── cartalith-noise/          hash/vnoise/fbm/ridged — hand-ported, see below  []
│   ├── cartalith-rng/            mulberry32, ported exactly (PARITY_TESTING.md)   []
│   ├── cartalith-vault/          Markdown Vault: sections, links, fs provider      []
│   ├── cartalith-spatial/        tiling, dirty-tracking, draft stack, geometry [jsmath]
│   ├── cartalith-urban/          urban morphology (UME engine)        [jsmath, rng]
│   ├── cartalith-assets/         asset packs, library, PNG            [jsmath, noise]
│   ├── cartalith-io/             save/load (SAVEFILE_COMPAT.md)       [spatial]
│   ├── cartalith-terrain/        tectonics → height → normalize → volcanism → archetypes
│   │                             [jsmath, noise, rng, spatial]
│   ├── cartalith-climate/        temperature, wind, rainfall  [jsmath, noise, terrain]
│   ├── cartalith-erosion/        droplet, stream-power, thermal
│   │                             [jsmath, noise, rng, terrain]
│   ├── cartalith-hydrology/      flow accumulation, river network, channel width
│   │                             [jsmath, terrain]
│   ├── cartalith-gpu/            wgpu compute stages, CPU fallback elsewhere  [noise]
│   │                             (dev-deps: terrain, climate, hydrology, engine, civ)
│   ├── cartalith-engine/         orchestrator: owns WorldState, runs the pipeline
│   │                             [jsmath, terrain, climate, hydrology, erosion,
│   │                              spatial, io, assets, gpu]
│   ├── cartalith-civ/            civilisation layer [jsmath, engine, hydrology, rng,
│   │                             terrain, urban, noise] (dev-dep: spatial)
│   └── cartalith-godot/          the only crate that depends on gdext; depends on
│                                 every crate above except urban and erosion
│                                 (reached through civ and engine)
├── godot-project/                scenes, GDScript glue, export presets
└── docs/                         STATUS.md, the retired CHANGELOG.md,
                                  3D_TERRAIN_RENDER_RESEARCH.md
```

**`cartalith-jsmath` is a true leaf, and that is the point.** It has **no
dependencies** — not on another Cartalith crate, not on a third-party crate, not
even a dev-dependency. `JS_SEMANTICS_AUDIT.md` catalogues eight places where
Rust's standard library and V8 disagree about a floating-point operation, and by
the time the audit ran the JS-faithful replacements had been written
independently in five crates: seven copies of `js_hypot`, seven of `js_round`,
three of `js_min`/`js_max`, two of `toFixed`, and one each of `js_exp`,
`js_sin`, `js_cos`, `js_log` and `js_atan2` that nothing outside their own crate
could reach. The copies had already drifted apart in three measurable ways.

A dependency-free crate is the only shape that reaches all fifteen without
disturbing the one-way ordering above: `cartalith-urban` otherwise depends
only on `cartalith-rng`, and `cartalith-assets` only on `cartalith-noise`, so
neither can see `cartalith-spatial` or `cartalith-hydrology`, where two of those helpers
used to live. Sitting below everything, it cannot create a cycle wherever it is
added. Adding *anything* to its `[dependencies]` — including a dev-dependency —
would put that back in question, which is why its bulk goldens carry a four-line
inline `mulberry32` rather than borrowing `cartalith-rng`'s.

**`cartalith-noise` is hand-ported rather than taken from `noise-rs`.** `noise-rs`
is well maintained and would be the obvious choice for a project without a parity
requirement. It implements different hash and lattice functions, so its output
cannot match the JS engine at the same seed. Keep it in mind for later decorative
effects where matching is not the goal (`ROADMAP.md`).

**What the split buys:**

- Each crate golden-parity-verifies in isolation, which is what makes
  `PARITY_TESTING.md`'s one-stage-at-a-time structure natural rather than forced.
- Later subsystems (civ, urban morphology, assets, vault) arrive as new crates
  without touching terrain or climate. Only `cartalith-civ` depends on
  `cartalith-engine`'s public types; `-urban`, `-assets` and `-vault` depend on
  no pipeline crate at all, and `cartalith-engine` itself depends on `-assets`
  (region export's PNG/zip) and `-gpu`. The
  boundary makes accidental duplication harder: you would have to add a
  dependency and import, not just paste inline.
- Only `cartalith-godot` knows Godot exists, so a future `cartalith-wasm` swaps
  in at the same seam with no engine change.
- `cartalith-io` keeps ZIP and JSON concerns out of generation logic.

**`cartalith-engine` orchestrates; it does not compute.** It owns `WorldState`
(the Rust equivalent of the JS `state` plus the field globals) and calls stages in
order. A height-formula tweak written inside it belongs in `cartalith-terrain`.

**This is a starting shape, not a commitment.** Refine it once Phase 0 exposes
real friction. One known pressure point: climate and erosion may need a tighter
loop than a one-way dependency allows — the JS engine's `evolveCoupled()` exists
because that coupling is genuinely two-way. Read that function before assuming
the graph stays acyclic. Refining the shape is expected; collapsing it back into
one crate defeats its purpose.

## Data flow

1. Godot UI collects seed, resolution, and map width, then calls `WorldGen.generate(...)`.
2. Rust runs the pipeline (`MVP_SCOPE.md`), off the main thread so the window stays
   responsive.
3. Rust returns the generated fields as flat arrays or built `Image`s.
4. Godot wraps them in an `ImageTexture` and draws — a `TextureRect` suffices for
   MVP, no tiling (`godot-shell` skill covers the texture path and its
   `update()`-over-`create_from_image()` rule).

The JS engine guarantees `generate()` completes deterministically and callers may
rely on it. Decide the Rust equivalent of that guarantee explicitly when building
this, rather than leaving it implicit.

## Threading

The JS engine runs erosion kernels in Web Workers with a synchronous fallback
(root `CLAUDE.md` invariant 11). Rust's equivalent is `rayon`, which removes the
need for a hand-written worker pool. The decomposition was decided per kernel
and is recorded in `CPU_MULTITHREADING_SCOPE.md`'s passes and at each kernel:
`cartalith-erosion::droplet_kernel` stays sequential (each droplet's path reads
what every earlier droplet carved), and the CPU thermal pass stays sequential
because its scatter-writes race; `gpu_thermal.wgsl` is the gather version that
`erode_op` runs under `use_gpu`.

The pool is Rayon's global pool, built once by
`cartalith_engine::ensure_thread_pool` with the worker count
`set_configured_thread_count` recorded (0 = Rayon's default). A count chosen
after the pool exists takes effect at the next start; that function's doc says
why a live rebuild is not offered.

## Decided since this file was written

- The array library: plain `Vec<f32>` / slices with manual indexing. No crate
  in the workspace depends on `ndarray`.
- The GDExtension API surface: `cartalith-godot`'s `WorldGen` class and its
  bridge modules; `STATUS.md` and `GENERATION_PARAMETERS.md` describe what it
  exposes.
- Windows builds are native on this machine; `TOOLCHAIN.md` keeps the
  Linux cross-compilation routes.

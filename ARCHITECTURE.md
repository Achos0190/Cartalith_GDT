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
Dependencies run one way, in pipeline order:

```
cartalith-native/                 (workspace root, new repository)
├── crates/
│   ├── cartalith-noise/          hash/vnoise/fbm/ridged — hand-ported, see below
│   ├── cartalith-rng/            mulberry32, ported exactly (PARITY_TESTING.md)
│   ├── cartalith-terrain/        tectonics → height → normalize → volcanism → archetypes
│   ├── cartalith-climate/        temperature, wind, rainfall
│   ├── cartalith-erosion/        droplet, stream-power, thermal
│   ├── cartalith-hydrology/      flow accumulation, river network, channel width
│   ├── cartalith-engine/         orchestrator: owns WorldState, runs the pipeline
│   ├── cartalith-io/             save/load (SAVEFILE_COMPAT.md)
│   └── cartalith-godot/          the only crate that depends on gdext
├── godot-project/                scenes, GDScript glue, export presets
└── docs/                         CHANGELOG.md, HANDOFF.md
```

**`cartalith-noise` is hand-ported rather than taken from `noise-rs`.** `noise-rs`
is well maintained and would be the obvious choice for a project without a parity
requirement. It implements different hash and lattice functions, so its output
cannot match the JS engine at the same seed. Keep it in mind for later decorative
effects where matching is not the goal (`ROADMAP.md`).

**What the split buys:**

- Each crate golden-parity-verifies in isolation, which is what makes
  `PARITY_TESTING.md`'s one-stage-at-a-time structure natural rather than forced.
- Later subsystems (civ, urban morphology, assets) arrive as new crates depending
  on `cartalith-engine`'s public types, without touching terrain or climate. The
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
need for a worker pool.

Decide the decomposition when porting erosion rather than assuming one. Droplets
mutate a shared height array as they carve, which is a data race across droplets
unless bucketed or serialised — read how `dropletKernel` handles it before
reaching for `par_iter`.

## Not decided yet

- The array library (`ndarray` versus `Vec<f32>` with manual indexing) — choose
  from real ergonomics while writing `cartalith-engine`.
- The GDExtension API surface — design against Phase 0 and Phase 1's actual needs.
- Windows cross-compilation route — see `TOOLCHAIN.md`.

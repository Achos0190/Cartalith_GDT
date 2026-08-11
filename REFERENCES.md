# Reference libraries and projects

Researched via live web search rather than recalled, and worth re-checking before
adopting any of it — a starting point for evaluation, not an approved dependency
list. For which algorithms may be replaced by a crate at all, see `PROVENANCE.md`
§2.

## Godot ↔ Rust

**[`godot-rust/gdext`](https://github.com/godot-rust/gdext)** — the GDExtension
binding for Godot 4, decided in `DECISIONS.md` §3. Actively developed; recent work
cited covers Godot 4.6 API level, Rust edition 2024, and WebAssembly.

**The caveat the project states about itself: Android and WASM support is
experimental, "documentation and tooling still lacking."** This port's goal
includes an `.apk`, which makes it the first thing to de-risk in Phase 0.

If Android proves too rough, the options are a C++ boundary layer for that
platform alone (unattractive — a second language at the seam this architecture
keeps thin) or reconsidering Godot for Android. **Surface it rather than working
around it silently.**

Docs: [godot-rust book](https://godot-rust.github.io/book/),
[docs.rs/godot](https://docs.rs/godot).

## Noise

**[`noise-rs`](https://github.com/Razaekel/noise-rs)** — the standard Rust noise
library. **Not used for `cartalith-noise`**: parity needs the JS engine's own hash
and lattice functions, and this implements different ones (`ARCHITECTURE.md`).
A fair candidate for later decorative effects where matching is not the goal.

The older `noisy` crate is unmaintained; its own maintainers point to noise-rs.

## Save and load

**[`zip`](https://docs.rs/zip)** reads the HTML app's archives, DEFLATE included.
**`serde`** and **`serde_json`** parse `params.json`. See `SAVEFILE_COMPAT.md`.

## Cross-compilation

**[`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk)** builds for Android without
hand-managing NDK paths. **`cargo-xwin`** cross-compiles to Windows MSVC targets
from Linux. Both in `TOOLCHAIN.md`.

## Godot terrain plugins — Phase 3, not now

Both are C++ GDExtension terrain systems for Godot 4 doing what the JS engine's 3D
drape does. Listed so Phase 3 need not rediscover them; evaluating them during the
2D MVP would breach `MVP_SCOPE.md`.

- **[`TokisanGames/Terrain3D`](https://github.com/TokisanGames/Terrain3D)** —
  GPU-driven clipmap terrain, 64 m to 65.5 km, up to 32 textures and 10 LOD
  levels, heightmap import. The more actively promoted of the two. Evaluate as a
  dependency if its import path can take this engine's height data directly, or as
  reference for a Godot-idiomatic clipmap renderer.
- **[`Zylann/godot_heightmap_plugin`](https://github.com/Zylann/godot_heightmap_plugin)**
  — older, GDScript rather than native. Lower performance ceiling, lower
  integration risk.

## Hydraulic erosion — reading, not a dependency

Several droplet-erosion projects surfaced in research
(`csaddison/Hydraulic-Erosion-Sim`, `guydols/HydraulicErosion`,
`weigert/SimpleErosion`). They are useful for building intuition about the
inertia/capacity/deposit/erode/evaporate parameters the JS `erosion` block already
uses.

**Do not port from them.** The source of truth is the JS `dropletKernel` and
`streamPowerKernel`, carrying tuning accumulated across many verified versions
(see v1.87 and v1.89 on the `MinHeap` work). A generic implementation will not
reproduce it. Read them to understand *why* the parameters behave as they do.

The app's own credits screen also names the erosion work it studied — see
`PROVENANCE.md`.

## Not adopted

- **`bevy`** — rejected as the framework in `DECISIONS.md` §3. Its ecosystem may
  hold useful reference code; nothing found was a clear win given Godot is the
  shell.
- **Wrapping the HTML app** — rejected in `DECISIONS.md` §1.

# LOD/tiling base: standalone data structures, not yet integrated

Prompted by the owner (2026-08-17), directly after `TERRAIN_ARCHITECTURE_RESEARCH.md`
was filed as forward-looking research, not current scope: *"LOD and zoom etc
might be out of scope for the base, but they're still goals in this project.
The base should be present before integration."* Given three concrete options
(data structures only / + dirty-region+versioning scaffolding / start
threading tiles through the live pipeline now), the owner chose the middle
one — build the foundational data structures now, standalone and real, but
touch **nothing** in the live generation/rendering pipeline.

This is deliberately narrower than `TERRAIN_ARCHITECTURE_RESEARCH.md`'s full
9-phase roadmap — no camera, no quadtree-driven rendering, no clipmaps, no GPU
residency, no interactive painting. Those stay deferred to whenever Phase 3
(3D, `ROADMAP.md`) or a genuine large-world need triggers real integration,
per `ROADMAP.md`'s own "Not a phase: LOD and large worlds" section: "revisit
when a concrete need appears rather than building it speculatively." What
changed between that section being written and this doc is scope, not
philosophy: the owner wants the *foundation* laid without waiting for that
trigger, specifically so integration (whenever it comes) isn't starting from
zero or fighting a codebase that was never built to accommodate it.

## Why standalone, not wired in

A half-migrated pipeline — some functions tile-aware, most not, nothing
actually exercising the tile-aware path — is worse than either extreme: it
adds real complexity (new types, new call shapes) without any of the payoff
(nothing gets faster, nothing streams, nothing renders LOD), and it's the
exact "dead code" risk the owner flagged wanting to avoid. A standalone,
fully-tested, unintegrated crate carries none of that risk: it costs nothing
to the existing pipeline (nothing depends on it, so nothing can break), and
when real integration starts, it's a known, tested foundation instead of a
green field.

## In scope

**New crate `cartalith-spatial`** (no `gdext` dependency — a pure
data-structure library, per `ARCHITECTURE.md`'s crate-boundary rule; doesn't
touch Godot, doesn't touch generation).

1. **`TiledField<T>`** — wraps a flat `Vec<T>` (the exact same
   Structure-of-Arrays shape `WorldState`/`CivData` already use — nothing new
   invented here) with tile-addressable views:
   - `tile_size: usize` as a constructor parameter, not a hardcoded constant.
     `TERRAIN_ARCHITECTURE_RESEARCH.md` §31 flags 64/128/256 as candidates and
     says explicitly "the correct size should be benchmarked against actual
     Cartalith workloads" — there is no real workload exercising this yet, so
     don't guess at one now.
   - Views: `whole()`, `tile(x, y)`, `region(bounds)`, `row(y)`, `column(x)`
     (research §28), both read-only and mutable, zero-copy — the view indexes
     into the existing backing array, it does not duplicate data.
   - Real unit tests: tile-boundary correctness, the off-by-one case when
     world dimensions aren't an exact multiple of `tile_size`, and confirming
     a mutable view's writes land in the correct backing-array cells.

2. **Packed quadtree / spatial index** (research §12/13, `geo-index`-inspired
   — read as a design reference, not a dependency to add):
   - `Vec<Node>` with integer child indices, not `Box<Node>`/pointers.
   - Generic per-node aggregate metadata: bounds, min/max of whatever `T` is
     being indexed, and a caller-defined flag/bitmask field (research §14/15's
     "contains water"/"contains river" idea) — keep the data structure generic,
     do not bake in Cartalith-specific semantics (no `has_river: bool` field
     literally named that) since there is no real caller yet to say what the
     right semantics are.
   - A build-from-field constructor (bottom-up aggregation over a
     `TiledField<T>` or a flat array + dimensions).
   - A bounds-rejecting region query returning candidate indices without
     visiting every cell.
   - Real unit tests: aggregate min/max actually matches the source data, a
     query outside every node's bounds returns empty without full traversal
     (assert via a call counter or similar, not just "returns the right
     answer" — the whole point is *not visiting* rejected subtrees), and one
     "find cells matching a predicate within a region" test exercising the
     rejection path for real.

3. **Dirty-region tracking + versioning** (research §16/17/41):
   - A per-tile dirty flag, generic (a caller-supplied tag/reason, not
     Cartalith's specific `HEIGHT_DIRTY`/`BIOME_DIRTY` field-dependency
     semantics from research §16-17 — that dependency graph has no real
     caller yet either, so don't invent Cartalith-specific field names inside
     a generic library crate).
   - A monotonic `u64` version counter per tile, bumped on any marked change
     (research §41's reproducibility idea — seed + tile ID + version).
   - Real unit tests: mark/clear dirty, version increments correctly and only
     on real changes.

4. **Serialization**: `serde` `Serialize`/`Deserialize` on the above types (add
   `serde` as a dependency if not already in the workspace — check
   `Cargo.toml` first). This supports both `TERRAIN_ARCHITECTURE_RESEARCH.md`
   §22's out-of-core/disk-tile idea and the owner's own earlier remark this
   session about native builds removing the browser's memory ceiling and
   working "from an on-device folder" for LOD tiling (recorded in project
   memory) — both assume tiles can eventually round-trip to disk. **Round-trip
   test only** — do not build an actual disk-paging/streaming system, that is
   real integration work and stays out of scope here.

## Out of scope (the owner's own chosen boundary)

- Any change to `cartalith-engine`/`cartalith-terrain`/`cartalith-climate`/
  `cartalith-erosion`/`cartalith-hydrology`/`cartalith-civ`/`cartalith-godot`
  — zero integration, this pass.
- Camera, quadtree-driven LOD rendering, clipmaps, GPU-residency lifecycles —
  real Phase-3-or-later work, `TERRAIN_ARCHITECTURE_RESEARCH.md`'s own §37/49.
- Interactive painting, brush tools, dependency-graph invalidation propagation
  — no editor exists in this port (`MVP_SCOPE.md` excludes the sculpt editor
  outright).
- Multi-resolution *generation* (different fields at different resolutions) —
  a pipeline-wide numerical-parity change, not a data-structure question;
  stays deferred regardless of this pass.
- Disk-backed tile streaming itself (only a serialization round-trip test,
  not a paging system).
- Picking a "correct" tile size — a constructor parameter, not a hardcoded
  default, since nothing benchmarks it yet.

## Verification

- `cargo build -p cartalith-spatial`, `cargo test -p cartalith-spatial`,
  `cargo clippy -p cartalith-spatial --all-targets` clean.
- `cargo build --workspace`, `cargo test --workspace` — 0 regressions
  elsewhere (nothing else references this crate yet, so this mainly confirms
  the workspace `Cargo.toml` addition didn't disturb anything).
- Real, meaningful unit tests per component above — correctness tests, not
  "it compiles."

## Done means

`cartalith-spatial` exists, compiles, is fully unit-tested, and is added to
the workspace `Cargo.toml` members list, but is not a dependency of any other
crate. Ready to be picked up — with a known, tested foundation instead of a
green field — whenever Phase 3 or a real large-world need starts actual
integration.

## Done (2026-08-17)

Built exactly as scoped: `TiledField<T>`, `QuadTree<T>`, `DirtyTracker`, all
`serde`-round-trippable, 24 real unit tests, `cargo build/test/clippy -p
cartalith-spatial` clean, full workspace `cargo test --workspace` clean (one
unrelated, already-documented pre-existing GPU-driver test flake reproduced
and confirmed unrelated — no GPU code in this crate, nothing depends on it).
Confirmed zero references from any other crate or `.gd`/`.tscn` file. Full
record: `cartalith-native/docs/CHANGELOG.md`'s "New crate cartalith-spatial"
entry, `docs/STATUS.md`'s own "LOD/tiling base" section.

## Integrated (2026-08-18) — the trigger was the tool system, not LOD

"Whenever Phase 3 or a real large-world need actually starts integration"
turned out to be neither: it was the DCC tool system.
`UNIFIED_TOOL_PLAN.md` milestone A built `PassBuffer<S>` and `StageGraph`
*in this crate*, on top of `TiledField`/`DirtyTracker`, and
`cartalith-engine` now depends on it — the first dependent, so the "not a
dependency of any other crate" line above is history rather than current
fact. The bet this document made paid off exactly as argued: the tool
system started from a tested foundation instead of a green field, and
`DirtyTracker` needed **no** extension at all to serve a real caller — its
deliberately generic caller-supplied reason string (defended here against
baking in Cartalith field names) turned out to be right, because each
pipeline stage owns its own tracker instance rather than sharing one
field-name enum.

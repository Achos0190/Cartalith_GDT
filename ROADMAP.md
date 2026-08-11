# Roadmap

A sketch, deliberately not over-planned. Each phase gets its own scope document —
as `MVP_SCOPE.md` serves Phase 1 — written when it starts, informed by what the
previous phase actually taught.

## Phase 0 — Walking skeleton

No engine logic. Prove `gdext` + Godot + Rust builds and runs on all three targets
with placeholder content: a triangle, a button, a printed line. Steps in
`TOOLCHAIN.md`.

**The Android export is the risk.** gdext's own docs call Android support
experimental (`REFERENCES.md`). Surface trouble here immediately rather than
discovering it mid-Phase 1.

## Phase 1 — Terrain MVP

Scope in `MVP_SCOPE.md`. The full pipeline — tectonics, height, climate, erosion,
hydrology — parity-verified (`PARITY_TESTING.md`), rendered in 2D, reading HTML
saves (`SAVEFILE_COMPAT.md`), shipping as an `.exe` and `.apk` confirmed on the
owner's hardware.

Two things belong in the definition of done and are easy to forget: the **credits
screen** and a **licence check** over the crates pulled in (`PROVENANCE.md`).

## Phase 2 — Civilisation layer

Block 2: factions, settlements, territory, roads, provinces, economy. A new
`cartalith-civ` crate depending on `cartalith-engine`'s terrain without modifying
it.

It will need its own golden data. Settlement suitability is exactly the kind of
subtly-tuned scoring — soil × rainfall optimum, flood penalty, coastal preference
— that a rewrite gets plausibly wrong; read the v1.30 "one function" CHANGELOG
entry before starting.

The Journey Planner is large and largely self-contained. Consider it a sub-phase
rather than bundling it.

## Phase 3 — Rendering and 3D

Brings back the 3D drape deferred in `DECISIONS.md` §4, and the point to evaluate
`Terrain3D` and `godot_heightmap_plugin` (`REFERENCES.md`) — as a dependency or as
reference for a Godot-idiomatic clipmap renderer.

Also the natural moment to revisit 2D fidelity beyond MVP's "correct and plain":
multi-octave grain, hillshade quality, NPR styles. And the moment to install a
UI/UX skill (`SKILLS.md`), once the interface outgrows four controls.

## Phase 4 — Asset Library

Block 3, the sprite and texture pack system. Lower priority unless custom art
becomes a near-term goal — and probably better after Phase 3 establishes what art
plugs into. Confirm before starting.

## Phase 5 — Urban morphology

Block 4, procedural city layouts. Already a self-contained DOM-free engine in the
JS codebase, which suggests it ports cleanly into `cartalith-urban`, depending on
`cartalith-civ` for settlement context.

## Not a phase: LOD and large worlds

The tiled-LOD deep-zoom system matters as worlds grow. Godot's terrain plugins may
cover the 3D case; the 2D case may need its own answer if 20,000 km worlds — the
ones v2.05–v2.09 were fixing — turn out to matter natively too.

Revisit when a concrete need appears rather than building it speculatively.

## Options kept open, not scheduled

Save-file **writing** (`SAVEFILE_COMPAT.md`), store distribution
(`DECISIONS.md` §6), and a WASM target sharing `cartalith-engine`
(`DECISIONS.md` §2) are all things the architecture permits and nobody has
committed to. Raise them rather than assuming they are queued.

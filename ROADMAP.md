# Roadmap

A sketch, deliberately not over-planned. Each phase gets its own scope document —
as `MVP_SCOPE.md` serves Phase 1 — written when it starts, informed by what the
previous phase actually taught.

**This file is the plan, not the state.** Each phase below carries a one-line
status, but the authority is `cartalith-native/docs/STATUS.md`; the per-phase
scope documents carry the milestone-by-milestone record. Where a phase
description below was written before the work started and the work then
contradicted it, the correction is recorded in the phase's own section rather
than by quietly rewriting the original text.

## Phase 0 — Walking skeleton

**Status: done.**

No engine logic. Prove `gdext` + Godot + Rust builds and runs on all three targets
with placeholder content: a triangle, a button, a printed line. Steps in
`TOOLCHAIN.md`.

**The Android export is the risk.** gdext's own docs call Android support
experimental (`REFERENCES.md`). Surface trouble here immediately rather than
discovering it mid-Phase 1.

## Phase 1 — Terrain MVP

**Status: done** — all seven criteria in `MVP_SCOPE.md`, plus both closeout
items named below (credits screen, crate licence audit).

Scope in `MVP_SCOPE.md`. The full pipeline — tectonics, height, climate, erosion,
hydrology — parity-verified (`PARITY_TESTING.md`), rendered in 2D, reading HTML
saves (`SAVEFILE_COMPAT.md`), shipping as an `.exe` and `.apk` confirmed on the
owner's hardware.

Two things belong in the definition of done and are easy to forget: the **credits
screen** and a **licence check** over the crates pulled in (`PROVENANCE.md`).

## Phase 2 — Civilisation layer

**Status: done** — `PHASE2_SCOPE.md` carries the milestone record. The
Journey Planner advice below proved right and was followed: it became its own
sub-phase (`JOURNEY_PLANNER_SCOPE.md`, six milestones) and is engine-complete
at 65 of the reference's 74 `jp*` functions — 6 UI-only, 2 JS idioms with no
Rust function to write, 1 blocked on a Route-tool pathfinder since ported.
Economy aggregation (`ECONOMY_SCOPE.md`) closed the last piece.

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

**Status: partial.** The 2D-fidelity half is well underway —
`TERRAIN_APPEARANCE_SCOPE.md` milestones 1-5 (multidirectional hillshade,
ambient occlusion, hydrology tint, the atlas look, geological exposure and
local contrast). **The 3D drape is not started.** The UI/UX half went further
than this entry anticipated — a full DCC-style shell was designed and built
(`DCC_SHELL_SCOPE.md`), and the hold called on 2026-08-18 was **lifted later
the same day** ("replace the current GUI and replace it in full … including
all it's wiring and functionality"); shell work has been landing continuously
since. *Corrected 2026-08-24 (`PARITY_AUDIT.md` pass 2, F1).*

Brings back the 3D drape deferred in `DECISIONS.md` §4, and the point to evaluate
`Terrain3D` and `godot_heightmap_plugin` (`REFERENCES.md`) — as a dependency or as
reference for a Godot-idiomatic clipmap renderer.

Also the natural moment to revisit 2D fidelity beyond MVP's "correct and plain":
multi-octave grain, hillshade quality, NPR styles. And the moment to install a
UI/UX skill (`SKILLS.md`), once the interface outgrows four controls.

## Phase 4 — Asset Library

**Status: done** — all seven milestones, `ASSET_LIBRARY_SCOPE.md`. Started on
the owner's explicit direction, which satisfied the "confirm before starting"
below; it did **not** wait for Phase 3, and that turned out fine because the
dependency this entry assumed doesn't exist (see the correction below).

**Two corrections to the paragraph below, from reading the real code:** an
asset is not arbitrary art but one PNG bound to a slot in a **frozen, ordered
vocabulary**, and an asset pack is a real shipping serialization format
(PKZIP + `pack.json`/`pack.csv`), not a proposal. The renderer genuinely
draws pack sprites; the vector glyphs are the *fallback*. Also confirmed:
Phase 5's urban morphology does **not** consume asset packs, so the two are
independent.

Block 3, the sprite and texture pack system. Lower priority unless custom art
becomes a near-term goal — and probably better after Phase 3 establishes what art
plugs into. Confirm before starting.

## Phase 5 — Urban morphology

Block 4, procedural city layouts. Already a self-contained DOM-free engine in the
JS codebase, which suggests it ports cleanly into `cartalith-urban`, depending on
`cartalith-civ` for settlement context.

**Started 2026-08-18. Two corrections to the paragraph above, from actually
reading the code — see `URBAN_MORPHOLOGY_SCOPE.md`:**

- "DOM-free" is right (zero hits for any browser API in block 4's whole range),
  but **"ports cleanly" is true of the boundary and false of the effort**: 92
  engine functions / 2,937 lines plus a 28-function / 925-line civ adapter —
  ~3,860 lines, **the largest single unported subsystem left**, bigger than the
  Journey Planner and the Asset Library. ~17 milestones, not one phase-sized
  push.
- **"depending on `cartalith-civ`" is wrong for the engine.** `generate(seed,
  opts)` takes only scalars and two plain rasters; no civ types anywhere. The
  civ coupling lives one layer up in block 2's `_um*` adapter. `cartalith-urban`
  depends on `cartalith-rng` alone.

## Not a phase: LOD and large worlds

The tiled-LOD deep-zoom system matters as worlds grow. Godot's terrain plugins may
cover the 3D case; the 2D case may need its own answer if 20,000 km worlds — the
ones v2.05–v2.09 were fixing — turn out to matter natively too.

Revisit when a concrete need appears rather than building it speculatively.

## Options kept open, not scheduled

~~Save-file **writing** (`SAVEFILE_COMPAT.md`)~~ — **no longer open. Done,
2026-08-23**, authorised by the owner after five register rows (FI-01, DM-04,
JP-06, JP-08, MEA-07) had queued up behind it. `cartalith_io::write_save`
plus `WorldGen::save_project`; File ▸ Save / Save as… / Autosave / Revert /
Close project are all real controls now. `SAVEFILE_COMPAT.md`'s own "Writing
a save" section carries the format decisions and the one disclosed
limitation (`state.erosion`).

Store distribution (`DECISIONS.md` §6) and a WASM target sharing
`cartalith-engine` (`DECISIONS.md` §2) are things the architecture permits
and nobody has committed to. Raise them rather than assuming they are queued.

**Markdown Vault integration** (`MARKDOWN_VAULT_INTEGRATION.md`) — owner-
supplied full V1 design (2026-08-18): links Cartalith entities (settlements,
POIs, regions) to an external Markdown vault (Obsidian-compatible, not
Obsidian-dependent), pull-oriented with explicit, section-aware write-back.
Genuinely new feature, not a port — nothing in the reference HTML app does
this, so it sits outside `DECISIONS.md` §7d's contract entirely. Needs its
own `MARKDOWN_VAULT_SCOPE.md` (the same investigate-then-milestone discipline
every other large effort here has used) before any code — the design doc's
own V1 acceptance criteria assume entity concepts this port may not fully
have yet (POIs/regions as addressable entities with their own info panel);
verify before scoping.

**Priority and framing, owner 2026-08-18**: *"Its not a critical part."*
Stays here, at the end, and does not compete with engine or parity work. The
target is a **generic Markdown vault** — Obsidian is one compatible vault and
the owner's own, but nothing may require it and no Obsidian-specific
behaviour belongs in the core. **An Obsidian plugin is a wish, deferred
outright.** A refined spec and templates are expected from the owner, so the
document on file will be replaced or extended before anyone scopes it.

Note that `DCC_SHELL_SPEC.md` §9 puts a vault block in the Data manager that
assumes more than the V1 design allows (`obsidian://` links, note links in
exported GeoJSON, two-way sync — the last an explicit V1 non-goal). Treat
that block as deferred rather than approved; `DCC_CONTROL_INDEX.md` records
the conflict and `MARKDOWN_VAULT_INTEGRATION.md`'s header resolves it.

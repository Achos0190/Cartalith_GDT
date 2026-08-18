# Cartalith — native port

A ground-up native port of **Cartalith Gen1**, the single-file HTML
worldbuilding tool, to a Rust engine inside Godot 4 — shipping a Windows
`.exe` and an Android `.apk`.

The frozen source lives in `reference/`; it is never modified. The port's own
code is the Cargo workspace in `cartalith-native/`.

## Where things stand

| Phase | State |
|---|---|
| 0 — Walking skeleton | **Done** |
| 1 — Terrain MVP | **Done**, all seven criteria plus closeout |
| 2 — Civilisation layer | **Done** — 17 milestones; the Journey Planner sub-phase is engine-complete (65 of the reference's 74 `jp*` functions; 6 UI-only, 2 JS idioms, 1 blocked) |
| 3 — Rendering and 3D | **Partial** — terrain appearance milestones 1-5 done; the 3D drape (`DECISIONS.md` §4) not started |
| 4 — Asset Library | **Done**, all seven milestones |
| 5 — Urban morphology | **In progress** — milestones 1-5 of ~17; the largest single unported subsystem (~3,860 lines) |

Cross-cutting work, none of it a numbered phase: GPU compute (9 milestones,
including a redesigned parallel flow accumulation), CPU multithreading (3),
a measured memory-optimization pass, the standalone `cartalith-spatial`
tiling/quadtree base, and the tool system's engine layer (milestones A-E).

**Authoritative status lives in `cartalith-native/docs/STATUS.md`** — read it
before starting work. `cartalith-native/docs/CHANGELOG.md` is the detailed
per-milestone history.

**All UI work is currently on hold** at the owner's direction (2026-08-18)
while the interface is redesigned; see the notice at the top of
`DCC_SHELL_SCOPE.md`.

## The workspace

Fourteen crates under `cartalith-native/crates/`, one per subsystem, per
`ARCHITECTURE.md`. Only `cartalith-godot` depends on `gdext`; everything else
is plain Rust and testable without Godot.

`cartalith-rng` · `cartalith-noise` · `cartalith-terrain` ·
`cartalith-climate` · `cartalith-erosion` · `cartalith-hydrology` ·
`cartalith-civ` · `cartalith-engine` · `cartalith-io` · `cartalith-gpu` ·
`cartalith-spatial` · `cartalith-assets` · `cartalith-urban` ·
`cartalith-godot`

## Reading order

**Start here** — the decisions and the rules that govern everything else:

| Document | Covers |
|---|---|
| `DECISIONS.md` | every choice, what it beat, and why — including §7d, the rule that behaviour is the contract and implementation is not |
| `ARCHITECTURE.md` | the Rust↔Godot split and the crate layout |
| `ROADMAP.md` | the phases (note: `docs/ROADMAP.md` is a *different*, source-project document) |
| `PARITY_TESTING.md` | golden-value testing against the JS engine |
| `PROVENANCE.md` | academic sources and formats; what must be hand-ported |

**Then, as the work requires** — scope documents, each owning one subsystem
and carrying its own milestone-by-milestone record of what was built, what the
reference actually turned out to do, and what remains:

`MVP_SCOPE.md` · `PHASE2_SCOPE.md` · `JOURNEY_PLANNER_SCOPE.md` ·
`ECONOMY_SCOPE.md` · `ASSET_LIBRARY_SCOPE.md` · `URBAN_MORPHOLOGY_SCOPE.md` ·
`TERRAIN_APPEARANCE_SCOPE.md` · `UNIFIED_TOOL_PLAN.md` ·
`GPU_LAYER_INTEGRATION_SCOPE.md` · `GPU_COMPUTE_PILOT_SCOPE.md` ·
`CPU_MULTITHREADING_SCOPE.md` · `MEMORY_OPTIMIZATION_SCOPE.md` ·
`LOD_TILING_BASE_SCOPE.md` · `ANDROID_BUILD_SCOPE.md` ·
`GENERATION_PARAMETERS.md` · `SAVEFILE_COMPAT.md` · `TOOLCHAIN.md` ·
`REFERENCES.md` · `SKILLS.md`

**Reference and direction** — inputs, not plans: `FUNCTIONAL_CONTRACT.md`
(what the HTML app does, capability by capability), `VISION.md`,
`UI_SHELL_DESIGN.md`, `DCC_SHELL_SCOPE.md`, `GUI_FEATURE_PARITY_SCOPE.md`,
`MARKDOWN_VAULT_INTEGRATION.md`, and the three owner-supplied research
documents (`TERRAIN_ARCHITECTURE_RESEARCH.md`,
`HETEROGENEOUS_COMPUTE_RESEARCH.md`, `TERRAIN_APPEARANCE_RESEARCH.md`), each
annotated with how much of it applies to this port today.

## Other directories

| Path | What it is |
|---|---|
| `cartalith-native/` | the Cargo workspace and the Godot project |
| `reference/` | the frozen HTML snapshot + its generated function index |
| `docs/` | **the source project's own documentation**, kept as provenance — see `docs/README.md`; two filenames collide with the port's |
| `design/` | owner-supplied UI mockups and handoff specs, imported verbatim |
| `skills/` | the vendored Claude Code skills this project uses |

## Working discipline

The HTML project's discipline — measure before fixing, test everything, finish
one thing before starting the next, document the reasoning — is why it
survived 200+ versions. It carries over, and this port has added to it from
real failures:

- **Read the reference before porting.** Every milestone that assumed a scope
  document's description without checking found it wrong: line ranges wrong at
  both ends, functions that turned out to be callers rather than duplicates,
  plans describing the rarer of two code paths.
- **One subsystem at a time**, verified before the next.
- **Faithful, not literal.** Idiomatic Rust is expected; identical numbers are
  also expected. A deviation that changes the numbers is a decision to raise,
  not to make quietly (`DECISIONS.md` §7, §7a, §7d).
- **Golden-matching is necessary and not sufficient.** Mutation-test the
  constants. Fixtures must be *shaped* to reach the code under test —
  quantised where a tie-break hides in continuous values, just-below-a-boundary
  where rounding hides a constant, and built out of the geometry under test.
  Each of those three conventions came from a sweep where every golden passed
  and most mutations survived.
- **Watch for V8's libm.** `Math.hypot` and `Math.exp` both diverge from
  Rust's, and both changed real results — one altered graph topology through a
  snap threshold. Use `geom::js_hypot` and `geom::js_exp`; likewise
  `js_min`/`js_max`, because JS propagates NaN where Rust absorbs it.
- **Say what you verified.** On-device behaviour, touch input and GPU
  rendering cannot be checked from a headless session. "Compiles and passes
  tests" is not "works" — and a synthetic tap at a computed pixel is not
  evidence a person can reach the control.

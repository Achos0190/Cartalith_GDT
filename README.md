# Cartalith — native port

A ground-up native port of **Cartalith Gen1**, the single-file HTML
worldbuilding tool, to a Rust engine inside Godot 4 — shipping a Windows
`.exe` and an Android `.apk`.

The frozen source lives in `reference/`; it is never modified. The port's own
code is the Cargo workspace in `cartalith-native/`.

## Where things stand

**In `cartalith-native/docs/STATUS.md`, and only there.** Owner decision,
2026-08-31: that file is the single source of truth for progress. Read it
before starting work.

This README used to open with a phase table. It is gone on purpose. It was a
second place to look, so it drifted — the last version of it described Phase 5
as "milestones 1-7 plus 17a … of ~17" and asserted the UI hold for six days
after `DCC_SHELL_SCOPE.md` had lifted it. Both were corrected by audit rather
than by the work that outdated them, which is the failure mode a single source
exists to remove.

What the rest of this file is for, then:

| Question | Answer |
|---|---|
| How far along is anything? | **`cartalith-native/docs/STATUS.md`** |
| What is left to pick up? | `OUTSTANDING_WORK.md` — the routed backlog |
| What is a milestone, and why is it shaped that way? | the owning `*_SCOPE.md`, below |
| What happened, and when? | `git log`. **`cartalith-native/docs/CHANGELOG.md` is retired** — frozen 2026-08-31, stops at 2026-08-26, historical narrative only |
| How is the code laid out, and how do I work in it? | the rest of this file |

**Standing decisions that are not status.** The UI hold was lifted by the owner
on 2026-08-18, later the same day it was called — the exact wording and scope
are at the top of `DCC_SHELL_SCOPE.md`. 3D was parked by the owner on
2026-08-31, after the commissioned research landed
(`cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md`); `DECISIONS.md` §4
stands. A decision like these keeps its home in the document that records it.
How much has been *built* under either one is `STATUS.md`'s answer, not this
file's.

## The workspace

Sixteen crates under `cartalith-native/crates/` — one per subsystem, plus the
dependency-free `cartalith-jsmath` leaf (`ARCHITECTURE.md`). Only
`cartalith-godot` depends on `gdext`; everything else is plain Rust and
testable without Godot.

`cartalith-jsmath` · `cartalith-rng` · `cartalith-noise` ·
`cartalith-terrain` · `cartalith-climate` · `cartalith-erosion` ·
`cartalith-hydrology` · `cartalith-civ` · `cartalith-engine` ·
`cartalith-io` · `cartalith-gpu` · `cartalith-spatial` ·
`cartalith-assets` · `cartalith-urban` · `cartalith-vault` ·
`cartalith-godot`

`cartalith-vault` is the newest and the only one that is not engine work at
all: the Markdown Vault integration (`MARKDOWN_VAULT_SCOPE.md`), depending on
`serde`/`serde_json` and on nothing else in the workspace.

## Reading order

**Start here** — the decisions and the rules that govern everything else:

| Document | Covers |
|---|---|
| `DECISIONS.md` | every choice, what it beat, and why — including §7d, the rule that behaviour is the contract and implementation is not |
| `ARCHITECTURE.md` | the Rust↔Godot split and the crate layout |
| `ROADMAP.md` | the phases (note: `docs/ROADMAP.md` is a *different*, source-project document) |
| `PARITY_TESTING.md` | golden-value testing against the JS engine |
| `PROVENANCE.md` | academic sources and formats; what must be hand-ported |

**Then, as the work requires** — scope documents, each owning one subsystem:
what its milestones *are*, what the reference actually turned out to do, and
the design reasoning behind both. **They no longer carry status** — as of
2026-08-31 every status column and done/not-started marker moved to
`STATUS.md`, and a scope document that still claims one is a leftover to fix,
not to believe:

`MVP_SCOPE.md` · `PHASE2_SCOPE.md` · `JOURNEY_PLANNER_SCOPE.md` ·
`ECONOMY_SCOPE.md` · `ASSET_LIBRARY_SCOPE.md` · `URBAN_MORPHOLOGY_SCOPE.md` ·
`TERRAIN_APPEARANCE_SCOPE.md` · `UNIFIED_TOOL_PLAN.md` ·
`GPU_LAYER_INTEGRATION_SCOPE.md` · `GPU_COMPUTE_PILOT_SCOPE.md` ·
`CPU_MULTITHREADING_SCOPE.md` · `MEMORY_OPTIMIZATION_SCOPE.md` ·
`PERFORMANCE_BENCHMARKS.md` (measured CPU/per-GPU/split comparison at 2048²
and 8192², and what actually decides how the app feels) ·
`LOD_TILING_BASE_SCOPE.md` · `ANDROID_BUILD_SCOPE.md` ·
`GENERATION_PARAMETERS.md` · `SAVEFILE_COMPAT.md` · `TOOLCHAIN.md` ·
`REFERENCES.md` · `SKILLS.md` · `LANDMARK_GENERATION_SCOPE.md`
(causally-placed landmarks — the inventory of what this engine already had for
it, the binding Category A/B/C rule, nine milestones). *This entry read "no
code yet" until 2026-08-31, having survived `cartalith-civ/src/landmark.rs`
landing at 3 730 lines on 2026-08-30. It is the exact drift the single-source
rule above exists to stop.*

**Reference and direction** — inputs, not plans: `FUNCTIONAL_CONTRACT.md`
(what the HTML app does, capability by capability), `VISION.md`,
`UI_SHELL_DESIGN.md`, `DCC_SHELL_SCOPE.md`, `GUI_FEATURE_PARITY_SCOPE.md`,
`GUI_GAP_REGISTER.md` (every disconnected control in the shipped shell,
classified against the design; comparable-app research where no design
exists; the menu-naming audit),
`MARKDOWN_VAULT_INTEGRATION.md`, and the four owner-supplied research
documents (`TERRAIN_ARCHITECTURE_RESEARCH.md`,
`HETEROGENEOUS_COMPUTE_RESEARCH.md`, `TERRAIN_APPEARANCE_RESEARCH.md`,
`LANDMARK_GENERATION_RESEARCH.md`), each annotated with how much of it
applies to this port today.

## Other directories

| Path | What it is |
|---|---|
| `cartalith-native/` | the Cargo workspace and the Godot project |
| `reference/` | the frozen `Cartalith Gen1 v2.10.html` + `FUNCTION_INDEX.md` — since 2026-08-23 a full checklist: every user-facing control (with backing functions) and a one-line purpose for all 1094 functions. **Both are v2.10 while a tracked `Cartalith Gen1 v2.11.html` sits at this repository's root** — the re-freeze is open work (`OUTSTANDING_WORK.md` §2.8), and `CLAUDE.md` records what is and is not established about it |
| `docs/` | **the source project's own documentation**, kept as provenance — see `docs/README.md`; two filenames collide with the port's |
| `design/` | owner-supplied UI mockups and handoff specs, imported verbatim |
| `skills/` | the vendored Claude Code skills this project uses |

## Working discipline

The HTML project's discipline — measure before fixing, test everything, finish
one thing before starting the next, document the reasoning — is why it
survived 200+ versions. It carries over, and this port has added to it from
real failures:

- **One place holds state; put it there, in the same change.** A milestone
  moving from open to done is one row edit in
  `cartalith-native/docs/STATUS.md` plus the code. A status sentence added to a
  scope document, to this file, or to a commit message *instead* is a
  regression, however true it is on the day it is written.
- **Verify status against the code, never by copying another document.** A
  document's claim about itself is a claim. The 2026-08-31 audit found
  `STATUS.md` calling landmark generation unbuilt on the day 3 730 lines of it
  shipped, and `ROADMAP.md` filing it as "not scheduled, no code written". Say
  what you opened, and prefer naming a symbol to citing a line number — line
  numbers here have drifted inside a single day.
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

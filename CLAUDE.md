# Cartalith native port — directory instructions

This repository **is** the new repository `DECISIONS.md` §8 describes. It holds
both the port's documentation and its code: `cartalith-native/` is the Cargo
workspace plus the Godot project.

`Cartalith_RC`'s own root `CLAUDE.md` governs `Cartalith Gen1 v*.html` there and
is unrelated to this effort except as the source being ported.

## Read `README.md` first

It carries the current phase status, the crate layout, the reading order and the
working discipline. This file exists to load automatically and state the
constraints below; it does not replace reading `README.md`, `DECISIONS.md` and
`ARCHITECTURE.md` properly.

**Authoritative status is `cartalith-native/docs/STATUS.md`.** Read it before
starting work — not this file, which is a map rather than a state.

## Constraints

- **Do not edit `reference/Cartalith Gen1 v2.10.html`.** It is the frozen
  snapshot every other document was written against. Re-freezing to a newer
  version is fine — regenerate `FUNCTION_INDEX.md` in the same pass, so the two
  never drift. (Checked 2026-08-18: the live `Cartalith_RC` repo is still at
  v2.10; no drift.)
- **Do not deviate from `DECISIONS.md` silently.** Architecture decided before
  code exists sometimes needs revision. Raise it, then record the new reasoning —
  the same way the HTML CHANGELOG discloses every deliberate re-baseline. §7a,
  §7b, §7c and §7d were all added that way.
- **Expect these documents to age, and say so when they have.** Godot versions,
  gdext maturity and crate specifics all move. Re-verify rather than trusting a
  version number written here.
- **The UI hold is lifted** (owner, 2026-08-18, later the same day it was
  called — see the top of `DCC_SHELL_SCOPE.md` for the exact wording and
  scope). A full DCC-shell replacement, including tool-system milestone F, is
  underway and has landed in stages since. This line is stale on every prior
  read of it; corrected here 2026-08-23 after `PARITY_AUDIT.md` caught this
  file — which auto-loads into every session — still asserting the hold
  `DCC_SHELL_SCOPE.md`'s own notice had already lifted.

## Two naming hazards

1. **`docs/` is the *source project's* documentation, not this port's.** Its
   `UNIFIED_TOOL_PLAN.md` and `ROADMAP.md` collide by name with the port's
   root-level documents of the same name and entirely different content. An
   unqualified reference to either means **the one at the repository root**. See
   `docs/README.md`.
2. **Three locations are called "docs"**: `docs/` (source project),
   `cartalith-native/docs/` (this port's living `CHANGELOG.md`/`STATUS.md`), and
   the design project's own `docs/`-rooted convention that `UI_SHELL_DESIGN.md`
   was imported with.

## Contents

| Path | What it is |
|---|---|
| `README.md` | **start here** — status, crates, reading order, discipline |
| `DECISIONS.md` | every choice, what it beat, and why |
| `ARCHITECTURE.md` | the Rust↔Godot split and crate layout |
| `ROADMAP.md` | the phases (0-5) |
| `PARITY_TESTING.md` | golden-value testing against the JS engine |
| `PROVENANCE.md` | sources, algorithms, formats; what must be hand-ported |
| `SAVEFILE_COMPAT.md` | the `.zip` format, verified against live code |
| `TOOLCHAIN.md` | setup, in order |
| `REFERENCES.md` | external libraries and projects |
| `SKILLS.md` | which skills to install, vendored or not, and why |
| **Scope documents** — one per subsystem, each carrying its own milestone record | |
| `MVP_SCOPE.md` | Phase 1's boundary and its seven success criteria |
| `PHASE2_SCOPE.md` | the civilisation layer, 17 milestones |
| `JOURNEY_PLANNER_SCOPE.md` | Phase 2's largest sub-phase; engine-complete |
| `ECONOMY_SCOPE.md` | faction/settlement economy aggregation |
| `ASSET_LIBRARY_SCOPE.md` | Phase 4, complete |
| `URBAN_MORPHOLOGY_SCOPE.md` | Phase 5, in progress; the largest unported subsystem |
| `TERRAIN_APPEARANCE_SCOPE.md` | Phase 3's 2D fidelity milestones |
| `UNIFIED_TOOL_PLAN.md` | the tool system, milestones A-F (**root**, not `docs/`) |
| `GPU_LAYER_INTEGRATION_SCOPE.md` | per-layer GPU work, 9 milestones |
| `GPU_COMPUTE_PILOT_SCOPE.md` | the original `wgpu` feasibility pilot |
| `CPU_MULTITHREADING_SCOPE.md` | Rayon parallelisation, 3 milestones |
| `MEMORY_OPTIMIZATION_SCOPE.md` | the measured memory pass |
| `LOD_TILING_BASE_SCOPE.md` | `cartalith-spatial`'s tiling/quadtree base |
| `ANDROID_BUILD_SCOPE.md` | Android toolchain and the real device passes |
| `GENERATION_PARAMETERS.md` | every exposed generation parameter and its API |
| `MARKDOWN_VAULT_SCOPE.md` | the Markdown Vault: the entity audit that found continents did not exist, and milestones 0-1 |
| **Direction and reference** — inputs, not plans | |
| `FUNCTIONAL_CONTRACT.md` | the HTML app's capabilities vs. this port, tagged per `DECISIONS.md` §7d |
| `VISION.md` | the owner's target render, with an honest gap assessment |
| `UI_SHELL_DESIGN.md` | the DCC shell's rule set (owner-supplied) |
| `DCC_SHELL_SCOPE.md` | how that shell maps onto the port — **carries the (lifted) UI hold notice** |
| `GUI_SHELL_SCOPE.md` | the superseded panel-browser shell; history only |
| `GUI_FEATURE_PARITY_SCOPE.md` | the gap audit between engine capability and GUI |
| `GUI_GAP_REGISTER.md` | every disconnected control in the shipped shell, classified by whether a design exists; comparable-app research where none does; the menu-naming audit |
| `MARKDOWN_VAULT_INTEGRATION.md` | owner-supplied V1 design; **scheduled and started 2026-08-24** — see `MARKDOWN_VAULT_SCOPE.md` |
| `HARDWARE_ACCELERATION.md` | owner-supplied GPU architecture, annotated with a major scope correction |
| `TERRAIN_ARCHITECTURE_RESEARCH.md` | owner-supplied; tiling/LOD/clipmaps, mostly Phase-3-or-later |
| `HETEROGENEOUS_COMPUTE_RESEARCH.md` | owner-supplied; hardware-tiered scheduling, mostly not yet applicable |
| `TERRAIN_APPEARANCE_RESEARCH.md` | owner-supplied; the source for `TERRAIN_APPEARANCE_SCOPE.md` |
| **Directories** | |
| `cartalith-native/` | the Cargo workspace (16 crates) and the Godot project |
| `cartalith-native/docs/` | the port's living `CHANGELOG.md` and `STATUS.md` |
| `reference/` | the frozen HTML snapshot and its function index |
| `docs/` | **the source project's** documentation — see `docs/README.md` |
| `design/` | owner-supplied UI mockups, imported verbatim |
| `skills/` | vendored skills (also installed under `.claude/skills/`) |

## Working rules this port learned the hard way

Recorded here because they are cheap to state and expensive to rediscover; each
came from a real failure, and every one is detailed in the scope document that
found it.

- **Verify a scope document's line ranges against the real reference before
  slicing.** Four consecutive urban milestones found theirs wrong — twice at
  both ends. A start that is too *late* does not fail to parse; it silently
  omits a definition.
- **Golden-matching is necessary and not sufficient.** Mutation-test the
  constants. Shape fixtures to reach the code: quantised where a tie-break hides
  in continuous values, just-below-a-boundary where rounding hides a constant,
  and built from the geometry under test.
- **Watch for silently-empty golden output.** Four subsystems were bitten: a
  slice missing a constant whose consumer swallowed its own exception; host-side
  assignment shadowing `let`-declared reference globals (lexical bindings, not
  `vm` context properties, twice); and an apostrophe in prose defeating a
  comment scanner. Assert non-emptiness and shape explicitly.
- **V8's libm is not Rust's.** `Math.hypot` and `Math.exp` both diverge, and
  both changed real results. Use `geom::js_hypot` and `geom::js_exp`, and
  `js_min`/`js_max` because JS propagates NaN where Rust absorbs it.
- **A stale binary reports a healthy `N passed`.** Re-run every mutation
  survivor in isolation.

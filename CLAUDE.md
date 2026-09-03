# Cartalith native port — directory instructions

This repository **is** the new repository `DECISIONS.md` §8 describes. It holds
both the port's documentation and its code: `cartalith-native/` is the Cargo
workspace plus the Godot project.

`Cartalith_RC`'s own root `CLAUDE.md` governs `Cartalith Gen1 v*.html` there and
is unrelated to this effort except as the source being ported.

## One place holds state: `cartalith-native/docs/STATUS.md`

Owner decision, 2026-08-31. **`cartalith-native/docs/STATUS.md` is the single
source of truth for progress.** Read it before starting work. Not this file —
which is a map, not a state — and not the document that happens to be open.

| If you want to know | Read | Not |
|---|---|---|
| Is this built? how far along is it? what is left? | **`STATUS.md`** | anything else, ever |
| What is this milestone, and why is it shaped that way? | the owning `*_SCOPE.md` | `STATUS.md`, which carries no design |
| What happened, in what order? | **`git log`** | `CHANGELOG.md` |
| Why is this line of Rust written this way? | the retired `CHANGELOG.md`, for anything before 2026-08-26 | — |
| **What am I about to get wrong?** (before scheduling a row, pinning a constant, quoting a timing, changing behaviour, declaring green) | **`MISTAKES.md`**'s preflight table | rediscovering it at cost, as this project did eleven times with one row class alone |
| How do I hand this to a fresh session? | **`SESSION_HANDOFF.md`** | a summary written from memory, which is how a stale number enters a new context |

Three rules follow from that, and they are the point of this section:

1. **Scope documents define milestones; they do not track them.** Their status
   columns and done/not-started markers were removed on 2026-08-31 and replaced
   with a pointer here. If you find one that still claims a status, it is a
   leftover — treat it as stale and fix it rather than believe it.
2. **`cartalith-native/docs/CHANGELOG.md` is retired** — frozen and marked, not
   deleted. It stops at 2026-08-26, 51 commits behind the tree. Append nothing
   to it; trust nothing in it as current. Its own header says the rest.
3. **Update `STATUS.md` in the same change that changes its answer.** Recording
   a status anywhere else is the regression this decision exists to prevent.

**A document's claim about itself is a claim, not evidence.** This is not
theoretical here: the audit behind this decision found `STATUS.md` asserting
landmark generation was unbuilt on the day a 3 730-line implementation of it
shipped, and `ROADMAP.md` filing the same subsystem as "not scheduled, no code
written". **Verify status against the code and say what you opened.** Prefer
naming a symbol to citing a line number — line numbers in this repository have
drifted inside a single day.

## Read `MISTAKES.md` at the start of every session

**Owner instruction, 2026-09-03.** `MISTAKES.md` at the repository root is a
**preemptive** file, not a log — its point is that the mistake is not made a
second time. It opens with a **preflight table keyed to what you are about to
do**: scan the left column, and if a row matches your next action, apply its
rule before you start. The entries beneath it exist only to explain why each
rule is there, and each is a mistake that actually shipped into this tree.
Several carry a `×N`, which is the best available guide to what will go wrong
next.

Three obligations, and they are cheap:

1. **Read the preflight table when the session starts, and again whenever you
   are about to schedule a row, pin a constant, quote a timing, change
   behaviour, or declare the work green.** The table is one screen.
2. **Put its rules in every agent brief.** The recurring ones — never encode "no
   value" as a plausible value; never assert a constant against itself; re-open
   a backlog row at its symbol before believing it; `cargo test` cannot see a
   broken shell — are the ones that keep recurring precisely because a fresh
   agent does not know them.
3. **Add an entry after a confirmed mistake or a user correction**, merging into
   an existing entry rather than duplicating it. Do not record transient tool
   failures or unverified guesses.

`STATUS.md` answers *what state is this in*; `OUTSTANDING_WORK.md` answers *what
is left*; **`MISTAKES.md` answers *what goes wrong here, and how to not do it
again*.** It is the cheapest of the three to read and the most expensive to
have skipped.

## Read `README.md` next

It carries the crate layout, the reading order and the working discipline. This
file exists to load automatically and state the constraints below; it does not
replace reading `README.md`, `DECISIONS.md` and `ARCHITECTURE.md` properly.

## Constraints

- **`reference/` holds two frozen snapshots. Do not edit either.** Re-freezing
  to a newer version is fine — regenerate the index in the same pass, so the two
  never drift.

  - **`Cartalith Gen1 v2.10.html`**, indexed by `FUNCTION_INDEX.md`. **Every
    line range in every scope document resolves against this file.** Do not
    delete it: that is why the re-freeze added a file instead of replacing one.
  - **`Cartalith Gen1 v2.11.html`**, indexed by `FUNCTION_INDEX_v2.11.md` — the
    version this repository ships (also at the root, where the owner committed
    it 2026-08-26). Read this one when you are reading the reference.
  - **`REFERENCE_DRIFT_v2.10_to_v2.11.md`** maps between them: the exact
    line-offset segments, the 14 functions added, and the changes that carry
    porting consequences. Follow a scope document's citation into v2.10, or
    offset it with that table — do not guess which file a line number means.

  **Unresolved, and stated rather than guessed:** whether the v2.11 here is the
  live `Cartalith_RC` head or a copy that repository has since moved past.
  `Cartalith_RC` is not present on this machine and is not a remote of this one,
  so it could not be checked. Do not assert either way without opening it.
- **Do not deviate from `DECISIONS.md` silently.** Architecture decided before
  code exists sometimes needs revision. Raise it, then record the new reasoning —
  the same way the *HTML project's own* CHANGELOG — a different file, in
  `Cartalith_RC`, not the retired one here — discloses every re-baseline. §7a,
  §7b, §7c and §7d were all added that way.
- **Expect these documents to age, and say so when they have.** Godot versions,
  gdext maturity and crate specifics all move. Re-verify rather than trusting a
  version number written here.
- **The UI hold is lifted** (owner, 2026-08-18, later the same day it was
  called — see the top of `DCC_SHELL_SCOPE.md` for the exact wording and
  scope). That is a standing decision, so it belongs here. **How far the DCC
  shell replacement has actually got does not — read `STATUS.md`.** This bullet
  used to carry a progress claim as well, and that claim went stale twice: it
  asserted the hold for five days after `DCC_SHELL_SCOPE.md` lifted it (caught
  by `PARITY_AUDIT.md`, corrected 2026-08-23), then described the replacement as
  "underway" through several more stages. A file that auto-loads into every
  session is the worst possible place to keep a moving number.

## Three naming hazards

1. **`docs/` is the *source project's* documentation, not this port's.** Its
   `UNIFIED_TOOL_PLAN.md` and `ROADMAP.md` collide by name with the port's
   root-level documents of the same name and entirely different content. An
   unqualified reference to either means **the one at the repository root**. See
   `docs/README.md`.
2. **Three locations are called "docs"**: `docs/` (source project),
   `cartalith-native/docs/` (this port's `STATUS.md`, plus the retired
   `CHANGELOG.md`), and the design project's own `docs/`-rooted convention that
   `UI_SHELL_DESIGN.md` was imported with.
3. **Two files are called `CHANGELOG.md`**, and they retired differently. The
   source project's, in `Cartalith_RC`, is live and is what most citations in
   `docs/` and in the porting-discipline skill mean. This port's,
   `cartalith-native/docs/CHANGELOG.md`, is retired. A bare "CHANGELOG" in a
   Rust source comment here almost always means the second, cited for a
   historical disclosure — which is still a valid thing to cite.

## Contents

| Path | What it is |
|---|---|
| `MISTAKES.md` | **read at session start** — every confirmed mistake, its root cause, the rule, and how to verify. Owner instruction, 2026-09-03 |
| `SESSION_HANDOFF.md` | **paste this to start a new session** — the standing goal, the skills, the 2-lanes-plus-a-verifier method, and the commands that derive current state. Deliberately carries no counts, hash or test total |
| `README.md` | **start here** — crates, reading order, discipline (status is `STATUS.md`'s) |
| `DECISIONS.md` | every choice, what it beat, and why |
| `ARCHITECTURE.md` | the Rust↔Godot split and crate layout |
| `ROADMAP.md` | the phases (0-5) |
| `PARITY_TESTING.md` | golden-value testing against the JS engine |
| `PROVENANCE.md` | sources, algorithms, formats; what must be hand-ported |
| `SAVEFILE_COMPAT.md` | the `.zip` format, verified against live code |
| `TOOLCHAIN.md` | setup, in order |
| `REFERENCES.md` | external libraries and projects |
| `SKILLS.md` | which skills to install, vendored or not, and why |
| **Scope documents** — one per subsystem. They **define** milestones and hold the design reasoning; **status for every one of them is in `STATUS.md`** | |
| `MVP_SCOPE.md` | Phase 1's boundary and its seven success criteria |
| `PHASE2_SCOPE.md` | the civilisation layer, 17 milestones |
| `JOURNEY_PLANNER_SCOPE.md` | Phase 2's largest sub-phase — the `jp*` route planner |
| `ECONOMY_SCOPE.md` | faction/settlement economy aggregation |
| `MILITARY_MANPOWER_SCOPE.md` | standing/field/emergency armies and war duration, from five variables — carries the owner's supplied specification **verbatim**, since the reference has no model to check it against |
| `ASSET_LIBRARY_SCOPE.md` | Phase 4 — the asset pack format, library and slicer |
| `URBAN_MORPHOLOGY_SCOPE.md` | Phase 5 — settlement layout; the project's largest block of unbuilt work |
| `TERRAIN_APPEARANCE_SCOPE.md` | Phase 3's 2D fidelity milestones |
| `UNIFIED_TOOL_PLAN.md` | the tool system, milestones A-F (**root**, not `docs/`) |
| `GPU_LAYER_INTEGRATION_SCOPE.md` | per-layer GPU work, 9 milestones |
| `GPU_COMPUTE_PILOT_SCOPE.md` | the original `wgpu` feasibility pilot |
| `CPU_MULTITHREADING_SCOPE.md` | Rayon parallelisation, 3 milestones |
| `MEMORY_OPTIMIZATION_SCOPE.md` | the measured memory pass |
| `LOD_TILING_BASE_SCOPE.md` | `cartalith-spatial`'s tiling/quadtree base |
| `ANDROID_BUILD_SCOPE.md` | Android toolchain and the real device passes |
| `GENERATION_PARAMETERS.md` | every exposed generation parameter and its API |
| `MARKDOWN_VAULT_SCOPE.md` | the Markdown Vault: the entity audit that found continents did not exist, and its milestone definitions |
| `STORY_PLANNING_SCOPE.md` | settlement timelines, the conflict overlay and the Journey entity — one subsystem over the Timeline's year cursor; carries the owner's three 2026-08-25 forks |
| `LANDMARK_GENERATION_SCOPE.md` | causally-placed landmarks: the inventory of what this engine already had for it (a golden-verified mountain-pass corridor detector, a TPI-equivalent buried inside the 2D renderer's AO, 15 mineral resources), the Category A/B/C rule carried forward as binding, nine milestones, and six open questions |
| `EXPORT_SCOPE.md` | 16K/32K single-image export — **shelved 2026-08-25 by the owner**, findings only. Records that the reference's own bake draws terrain and nothing else, the four measured gaps in today's export, the render-once decision that would have to be reversed, a banded renderer that was prototyped and measured byte-identical before being reverted, and the codec survey (WebP eliminated at 16 383 px, JPEG XL at its AGPL encoder) |
| **Direction and reference** — inputs, not plans | |
| `FUNCTIONAL_CONTRACT.md` | the HTML app's capabilities vs. this port, tagged per `DECISIONS.md` §7d |
| `VISION.md` | the owner's target render, with an honest gap assessment |
| `DESIGN_HANDOFF.md` | **give this to a designer.** Everything needed to produce a buildable GUI: the resolved tokens, the frame geometry for all three shells, the widget inventory a design must map onto, what does not exist, and the six rules learned expensively |
| `UI_SHELL_DESIGN.md` | the DCC shell's rule set (owner-supplied) |
| `DCC_SHELL_SCOPE.md` | how that shell maps onto the port — **carries the (lifted) UI hold notice** |
| `GUI_SHELL_SCOPE.md` | the superseded panel-browser shell; history only |
| `GUI_FEATURE_PARITY_SCOPE.md` | the gap audit between engine capability and GUI |
| `GUI_GAP_REGISTER.md` | every disconnected control in the shipped shell, classified by whether a design exists; comparable-app research where none does; the menu-naming audit |
| `MARKDOWN_VAULT_INTEGRATION.md` | owner-supplied V1 design; scheduled by the owner 2026-08-24 — the milestones are in `MARKDOWN_VAULT_SCOPE.md` |
| `HARDWARE_ACCELERATION.md` | owner-supplied GPU architecture, annotated with a major scope correction |
| `TERRAIN_ARCHITECTURE_RESEARCH.md` | owner-supplied; tiling/LOD/clipmaps, mostly Phase-3-or-later |
| `HETEROGENEOUS_COMPUTE_RESEARCH.md` | owner-supplied; hardware-tiered scheduling, mostly not yet applicable |
| `TERRAIN_APPEARANCE_RESEARCH.md` | owner-supplied; the source for `TERRAIN_APPEARANCE_SCOPE.md` |
| `LANDMARK_GENERATION_RESEARCH.md` | owner-supplied, imported verbatim 2026-08-30; a geographic-causality framework for landmark placement (TPI, viewshed, least-cost path, Poisson-disc, spatial interaction) — see `LANDMARK_GENERATION_SCOPE.md` for what of it this port already has |
| **Directories** | |
| `cartalith-native/` | the Cargo workspace (16 crates) and the Godot project |
| `cartalith-native/docs/` | **`STATUS.md`** — the single source of truth for progress. Also the **retired** `CHANGELOG.md` (frozen 2026-08-31, history only) and `3D_TERRAIN_RENDER_RESEARCH.md` |
| `reference/` | the two frozen HTML snapshots (v2.10, v2.11) and their function indexes |
| `docs/` | **the source project's** documentation — see `docs/README.md` |
| `design/` | owner-supplied UI mockups, imported verbatim |
| `skills/` | vendored skills (also installed under `.claude/skills/`) |

## Working rules this port learned the hard way

Recorded here because they are cheap to state and expensive to rediscover; each
came from a real failure, and every one is detailed in the scope document that
found it.

- **When two design canvases disagree, the newer one wins; where none exists,
  derive from the DCC canvases' own vocabulary.** Owner ruling, 2026-08-25 —
  the full statement, and the five conflicts it already settles, are at the top
  of `DCC_SHELL_SCOPE.md`. An owner decision is newer than any canvas: `Data ▸
  Conversion` is still drawn in the canvas and was removed by decision on
  2026-08-20, so the canvas is the stale party there, not the shell.
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
- **`godot --headless --import` silently strips `project.godot`'s `;` comments**,
  including the block warning that `#`/`##` there is silently swallowed as data
  rather than a comment (see the Constraints section above). Registering a new
  `class_name` needs that import pass — diff `project.godot` after running it and
  restore the comment block if it's gone, every time, not just once.

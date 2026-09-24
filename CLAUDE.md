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
| Is this built? How far along is it? | **`STATUS.md`** | anything else, ever |
| What is left, and where do I go to do it? | **`OUTSTANDING_WORK.md`** — a router, not state. Where it and `STATUS.md` disagree about a status, `STATUS.md` wins | a scope document's own to-do list |
| What did the owner decide? | `DECISIONS.md` (architecture) and `LARGE_ITEM_RULINGS.md` (owner rulings) | re-asking the owner |
| What is this milestone, and why is it shaped that way? | the owning `*_SCOPE.md` | `STATUS.md`, which carries no design |
| What happened, in what order? | **`git log`** | `CHANGELOG.md` |
| Why is this line of Rust written this way? | the retired `CHANGELOG.md`, for anything before 2026-08-26 | — |
| **What am I about to get wrong?** (before scheduling a row, pinning a constant, quoting a timing, changing behaviour, declaring green) | **`MISTAKES.md`**'s preflight table | rediscovering it at cost — each `×N` there counts how often this project already has |
| How do I hand this to a fresh session? | **`SESSION_HANDOFF.md`** | a summary written from memory, which is how a stale number enters a new context |

Three rules follow from that, and they are the point of this section:

1. **Scope documents define milestones; they do not track them.** Their status
   columns and done/not-started markers were removed on 2026-08-31 and replaced
   with a pointer here. If you find one that still claims a status, it is a
   leftover — treat it as stale and fix it rather than believe it.
2. **`cartalith-native/docs/CHANGELOG.md` is retired** — frozen and marked, not
   deleted. Its last entry is 2026-08-26; it was retired 2026-08-31. Append
   nothing to it; trust nothing in it as current. Its own header says the rest.
3. **Update `STATUS.md` in the same change that changes its answer.** Recording
   a status anywhere else is the regression this decision exists to prevent.

**A document's claim about itself is a claim, not evidence.** This is not
theoretical here: the audit behind this decision found `STATUS.md` asserting
landmark generation was unbuilt on the day a 3 730-line implementation of it
shipped, and `ROADMAP.md` filing the same subsystem as "not scheduled, no code
written". **Verify status against the code and say what you opened.** Prefer
naming a symbol to citing a line number — line numbers in this repository have
drifted inside a single day.

**This file is not exempt, and it is the worst possible place to keep a moving
number**: it auto-loads into every session, so a stale figure here reaches every
context at once. It has been caught carrying stale state again and again — the
UI hold asserted for five days after it was lifted (caught by `PARITY_AUDIT.md`),
the DCC shell replacement described as "underway" through several more stages,
the crate count wrong, urban morphology called *"the project's largest block of
unbuilt work"* for nine days after its milestones closed (`9e79e52`,
2026-09-03), and `EXPORT_SCOPE.md` described as shelved for seventeen days after
the owner un-shelved it. So its rows route; they do not report progress.
Standing decisions belong here; how far anything has got does not.

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
   behaviour, or declare the work green.** It has outgrown one screen; scan the
   whole left column against what you are about to do anyway — choosing rows
   from memory is its own entry, and it missed the rows that then bit.
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
    delete it: that is why the v2.11 freeze added a file instead of replacing
    one.
  - **`Cartalith Gen1 v2.11.html`**, indexed by `FUNCTION_INDEX_v2.11.md` —
    read this one when you are reading the reference. It is a byte copy of the
    v2.11 the owner committed at this repository's root on 2026-08-26
    (`4b2c95a`, `b576d56`), which stays there too.
  - **`REFERENCE_DRIFT_v2.10_to_v2.11.md`** — at the repository root, not in
    `reference/` — maps between them: the exact line-offset segments, the 14
    functions added, and the changes that carry porting consequences. Follow a
    scope document's citation into v2.10, or offset it with that table — do not
    guess which file a line number means.

- **The source has moved far past both snapshots, and the line to follow is
  decided.** `Cartalith_RC` forked at v2.22 into a mainline and a DCC line;
  every engine change from v2.25 on exists only on the DCC line. **The port
  follows the DCC line** (owner, Ruling AP, 2026-09-23, `LARGE_ITEM_RULINGS.md`).
  `RC_ENGINE_CHANGES.md` specifies the interval change by change; it is routed
  as `OUTSTANDING_WORK.md` §2.9, and the re-freeze itself is §2.8. How far
  behind the snapshots are, and how much of the interval is already ported, are
  moving answers — read them there and in `STATUS.md`, do not assume either way,
  and **do not record them in this file**; that is exactly the second-source
  habit the section above forbids.
- **Do not deviate from `DECISIONS.md` silently.** Architecture decided before
  code exists sometimes needs revision. Raise it, then record the new reasoning —
  the same way the *HTML project's own* CHANGELOG — a different file, in
  `Cartalith_RC`, not the retired one here — discloses every re-baseline.
  §7a through §7q were all added that way.
- **Expect these documents — this one included — to age, and say so when they
  have.** Godot versions, gdext maturity and crate specifics all move. Re-verify
  rather than trusting a version number or a count written here.
- **The UI hold is lifted** (owner, 2026-08-18, later the same day it was
  called — see the top of `DCC_SHELL_SCOPE.md` for the exact wording and
  scope). That is a standing decision, so it belongs here. **How far the DCC
  shell replacement has actually got does not — read `STATUS.md`.**

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

A routing table: each row says what a document *is*, never how far its work has
got. Status for everything below is in `STATUS.md`.

| Path | What it is |
|---|---|
| **Start here** | |
| `MISTAKES.md` | **read at session start** — every confirmed mistake, its root cause, the rule, and how to verify. Owner instruction, 2026-09-03 |
| `SESSION_HANDOFF.md` | **paste this to start a new session** — the standing goal, the skills, the lanes-plus-a-verifier method, and the commands that derive current state. Deliberately carries no counts, hash or test total |
| `README.md` | crates, reading order, discipline (status is `STATUS.md`'s) |
| `OUTSTANDING_WORK.md` | **what is left** — the routed backlog: every row names the document that owns the work. A router, not state; it must never grow a "done" column |
| `LARGE_ITEM_RULINGS.md` | every owner ruling, dated — decisions, never status. Cite a ruling rather than re-asking the owner |
| `DECISIONS.md` | every architectural choice, what it beat, and why |
| `ARCHITECTURE.md` | the Rust↔Godot split and crate layout |
| `ROADMAP.md` | the phases (0-5) |
| **Porting method and references** | |
| `PARITY_TESTING.md` | golden-value testing against the JS engine |
| `JS_SEMANTICS_AUDIT.md` | **read before porting** — every place V8 and Rust disagree about a floating-point operation, and the `cartalith-jsmath` helper that answers each |
| `PROVENANCE.md` | sources, algorithms, formats; what must be hand-ported |
| `SAVEFILE_COMPAT.md` | the `.zip` format, verified against live code |
| `REFERENCE_DRIFT_v2.10_to_v2.11.md` | the line-offset map between the two frozen snapshots (see Constraints) |
| `ALIGNMENT_AUDIT.md` | the 2026-09-24 code-versus-documentation audit: every place a document, string or comment disagreed with the code, with evidence; findings only, routed from `OUTSTANDING_WORK.md` §2.11 |
| `PARITY_AUDIT.md` | three dated passes (2026-08-23 to 25) checking the port's progress claims against the legacy checklist and the code; findings only, kept as a trail |
| `PERFORMANCE_BENCHMARKS.md` | compute-configuration benchmarks at 2048² and 8192², measured on this machine and judged on smoothness, not throughput |
| `TOOLCHAIN.md` | setup, in order |
| `REFERENCES.md` | external libraries and projects |
| `SKILLS.md` | which skills to install, vendored or not, and why |
| **Scope documents** — one per subsystem. They **define** milestones and hold the design reasoning | |
| `MVP_SCOPE.md` | Phase 1's boundary and its seven success criteria |
| `PHASE2_SCOPE.md` | the civilisation layer, 21 milestones |
| `JOURNEY_PLANNER_SCOPE.md` | Phase 2's largest sub-phase — the `jp*` route planner |
| `ECONOMY_SCOPE.md` | faction/settlement economy aggregation |
| `MILITARY_MANPOWER_SCOPE.md` | standing/field/emergency armies and war duration, from five variables — carries the owner's supplied specification **verbatim**, since the reference has no model to check it against |
| `RELIGION_DIFFUSION_SCOPE.md` | a quantitative religion-diffusion model scoped from an owner-supplied paper, preserved verbatim for the same reason — new scope, not a port |
| `TIMELINE_SCOPE.md` | the timeline / collapse-recovery simulation, its reference function list re-verified line by line |
| `STORY_PLANNING_SCOPE.md` | settlement timelines, the conflict overlay and the Journey entity — one subsystem over the Timeline's year cursor; carries the owner's three 2026-08-25 forks |
| `ASSET_LIBRARY_SCOPE.md` | Phase 4 — the asset pack format, library and slicer |
| `URBAN_MORPHOLOGY_SCOPE.md` | Phase 5 — settlement layout, milestones 1-17 and their reasoning |
| `TERRAIN_APPEARANCE_SCOPE.md` | Phase 3's 2D fidelity milestones |
| `UNIFIED_TOOL_PLAN.md` | the tool system, milestones A-F (**root**, not `docs/`) |
| `SCULPT_LIVE_SCOPE.md` | live sculpt manipulation, tiers L1-L3 (owner ruling 2026-08-18) — replaces the reference's deliberately cheap draft overlay |
| `EROSION_GEOLOGICAL_TIME_SCOPE.md` | geological time as a forcing framework for the erosion kernels; partial by its own account, and its §7 names what is missing |
| `GPU_LAYER_INTEGRATION_SCOPE.md` | per-layer GPU work, 9 milestones |
| `GPU_COMPUTE_PILOT_SCOPE.md` | the original `wgpu` feasibility pilot |
| `CPU_MULTITHREADING_SCOPE.md` | Rayon parallelisation, 3 milestones |
| `MEMORY_OPTIMIZATION_SCOPE.md` | the measured memory pass |
| `LOD_TILING_BASE_SCOPE.md` | `cartalith-spatial`'s tiling/quadtree base |
| `LOD_TILING_INTEGRATION_SCOPE.md` | threading the tiling base through the pipeline: tiers Z1-Z5, milestones M0-M3 (interactive deep-zoom tiles, the atlas) |
| `LOD_DETAIL_SCOPE.md` | scale-dependent terrain detail toward the owner's Aletsch target (Ruling K): milestones LOD-D0 to D6 — zoom-sweep harness, the `renderBiomeTileRGBA` port, colour tiles, no-popping morph, ice and snow from existing fields, scale-aware shading and hydrology, off-main-thread synthesis — and its owner questions with defaults |
| `ANDROID_BUILD_SCOPE.md` | Android toolchain and the real device passes |
| `GENERATION_PARAMETERS.md` | every exposed generation parameter and its API |
| `MARKDOWN_VAULT_SCOPE.md` | the Markdown Vault: the entity audit that found continents did not exist, and its milestone definitions |
| `LANDMARK_GENERATION_SCOPE.md` | causally-placed landmarks: the inventory of what this engine already had for it (a golden-verified mountain-pass corridor detector, a TPI-equivalent buried inside the 2D renderer's AO, 15 mineral resources), the Category A/B/C rule carried forward as binding, nine milestones, and its owner questions |
| `EXPORT_SCOPE.md` | 16K/32K single-image export. Shelved by the owner 2026-08-25, un-shelved 2026-09-06 (ruling 15; PNG by ruling 26) and resumed by Ruling AP 2026-09-23 — its header carries the current scope. Keeps the findings: the reference's own bake draws terrain and nothing else, the measured gaps in today's export, a banded renderer prototyped and measured byte-identical before being reverted (the prose is its only surviving artefact), and the codec survey |
| **UI and design** | |
| `DESIGN_HANDOFF.md` | **give this to a designer.** Everything needed to produce a buildable GUI: the resolved tokens, the frame geometry for all three shells, the widget inventory a design must map onto, what does not exist, and the six rules learned expensively |
| `UI_SHELL_DESIGN.md` | the DCC shell's rule set (owner-supplied) |
| `DCC_SHELL_SCOPE.md` | how that shell maps onto the port — **carries the (lifted) UI hold notice and the which-canvas-wins ruling** |
| `DCC_SHELL_SPEC.md` | the DCC shell implementation spec, imported from the owner's design project; records its conflicts with the real product rather than resolving them silently |
| `DCC_CONTROL_INDEX.md` | every control in the DCC spec against what this program can do, one row per control |
| `ANDROID_UI_SPEC.md` | the phone's per-screen inventory against `design/Cartalith-Android-2026-09-07.dc.html` |
| `TABLET_UI_SPEC.md` | the tablet canvas inventoried against the shipped shell — read its 2026-09-07 correction first: its original "inventory only, adoption undecided" framing was wrong |
| `JOURNEY_PLANNER_SPEC.md` | the journey planner rebuilt on the DCC shell — every v2.10 field and computation kept, only layout changes |
| `TRAVEL_LIBRARY_SPEC.md` | `Data ▸ Travel library` — an addition to the DCC GUI, not in v2.10 |
| `SCULPT_FUNCTION_CHART.md` | the HTML's sculpt panel charted against the Rust port and the DCC spec — "similar functioning, not copies" (owner, 2026-08-18) |
| `GUI_SHELL_SCOPE.md` | the superseded panel-browser shell; history only |
| `GUI_FEATURE_PARITY_SCOPE.md` | the gap audit between engine capability and GUI |
| `GUI_GAP_REGISTER.md` | every disconnected control in the shipped shell, classified by whether a design exists; comparable-app research where none does; the menu-naming audit |
| `UNWIRED_FUNCTIONS.md` | per control: the product draws it — is anything behind it? A dated cut, not a live count |
| `SCREENS_NEEDED.md` | the surfaces with no artboard anywhere, briefed for Claude Design |
| `STRANDED_TOOLS.md` | closed 2026-08-19, when the design's §4.5 Tool palette gave every stranded tool a home; history only |
| **Direction and research** — inputs, not plans | |
| `FUNCTIONAL_CONTRACT.md` | the HTML app's capabilities vs. this port, tagged per `DECISIONS.md` §7d |
| `PORT_ONLY_FEATURES.md` | the reverse: features and refinements this port has that the legacy v2.11 HTML does not, each checked at its symbol and against the reference (2026-09-13). A list, not a status |
| `RC_ENGINE_CHANGES.md` | **the source engine's simulation/generation changes after the freeze, from v2.11 on, as a porting spec** — the function and constant each change lives in, why each number is that number, which harness verified it. Claims nothing about port status. Read first: its fork note; §7's cross-cutting calibration rules; and §8's heading before appending to it (§8.1 moves generated output, §8.2 is adjacent). It names **defects a faithful port inherits** — among them §6s.4's corner-only water test, §6r.5's unbounded retry loop, and v2.68's never-drawn `field`/`pasture` details — and **builds a port should not make**, such as §6m.4's reverted digging pass and v2.68's furrow hatching. Read those sections whatever you decide about the feature |
| `VISION.md` | the owner's target render, with an honest gap assessment |
| `MARKDOWN_VAULT_INTEGRATION.md` | owner-supplied V1 design; scheduled by the owner 2026-08-24 — the milestones are in `MARKDOWN_VAULT_SCOPE.md` |
| `HARDWARE_ACCELERATION.md` | owner-supplied GPU architecture, annotated with a major scope correction |
| `TERRAIN_ARCHITECTURE_RESEARCH.md` | owner-supplied; tiling/LOD/clipmaps, mostly Phase-3-or-later |
| `HETEROGENEOUS_COMPUTE_RESEARCH.md` | owner-supplied; hardware-tiered scheduling, mostly not yet applicable |
| `TERRAIN_APPEARANCE_RESEARCH.md` | owner-supplied; the source for `TERRAIN_APPEARANCE_SCOPE.md` |
| `GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` | **agent-produced** (2026-08-24), every claim evidence-tagged; its analysis is kept as written and its §4 says what has since been built |
| `HYDROLOGY_CLASSIFICATION_RESEARCH.md` | owner-supplied, imported verbatim 2026-09-08 — lake/sea/marine-basin classification by hydrological **topology** rather than surface area. Its import note describes the size-primary rule of that date; the owner has since ruled on it (Rulings Q and T, `LARGE_ITEM_RULINGS.md`). Any change here moves `buildWaterBodies` goldens |
| `docs/research/lod extra info.md` | owner-supplied 2026-09-12 — **left where the owner put it, inside the *source* project's `docs/` tree (naming hazard 1)**, although it targets this port's Rust renderer. Scale-dependent terrain detail: a multiresolution pyramid, LOD by physical scale rather than GUI zoom, no popping, terrain-derived snow/rock/glacier/hydrology, citing the clipmap/CDLOD literature. **Its crate names are not this workspace's** (`cartalith-grid`, `-compute`, `-worldgen`, `-cartograph`, `-render`, `-logistics`, `-project`, `-types` do not exist). `LOD_DETAIL_SCOPE.md` turns it into milestones — read that first |
| `LANDMARK_GENERATION_RESEARCH.md` | owner-supplied, imported verbatim 2026-08-30; a geographic-causality framework for landmark placement (TPI, viewshed, least-cost path, Poisson-disc, spatial interaction) — see `LANDMARK_GENERATION_SCOPE.md` for what of it this port already had |
| `REFERENCE_MAP_RECONSTRUCTION_RESEARCH.md` | owner-requested 2026-09-13 — load an existing map image as an adjustable-opacity reference and sculpt a heightmap over it, with land/river edge detection and hand-drawn tectonic plates whose edges roughen like the generator's warp. Research and a proposed design (RM-0…RM-11), **not a scope**: no reference ancestor, and owner questions gate its build rows. Finds that noise β roughens height rather than plate edges |
| `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` | owner-requested 2026-09-20 — a hard, numbered acceptance bar ("always correctly detailed, no pixelated/square artifacts, no matter the zoom") for real river and mountain detail on zoom without upsampling; raster stays the simulation substrate, deliberately not a vertex/TIN replacement. EF-0…EF-9 propose a multi-resolution refinement primitive, local hydrology re-accumulation for real tributaries, a generalised vector layer and a tile structure. **EF-7 is Ruling N** (`LARGE_ITEM_RULINGS.md`): settlement siting may change to use real river/coast geometry, gated on staying a term inside the existing suitability ranking, not a bypass. Research and a proposed design, **not a scope** |
| **Directories** | |
| `cartalith-native/` | the Cargo workspace and the Godot project (`ls cartalith-native/crates` for the crates; `ARCHITECTURE.md` for what each is) |
| `cartalith-native/docs/` | **`STATUS.md`** — the single source of truth for progress. Also the **retired** `CHANGELOG.md` (frozen 2026-08-31, history only) and `3D_TERRAIN_RENDER_RESEARCH.md` |
| `reference/` | the two frozen HTML snapshots (v2.10, v2.11) and their function indexes |
| `docs/` | **the source project's** documentation — see `docs/README.md` |
| `design/` | owner-supplied UI mockups and design canvases, imported verbatim |
| `skills/` | vendored project skills (installed, with third-party ones, under `.claude/skills/`) |

## Working rules this port learned the hard way

Recorded here because they are cheap to state and expensive to rediscover; each
came from a real failure, and every one is detailed in the scope document that
found it.

- **When two design canvases disagree, the newer one wins; where none exists,
  derive from the DCC canvases' own vocabulary.** Owner ruling, 2026-08-25 —
  the full statement, and the conflicts it already settles, are at the top of
  `DCC_SHELL_SCOPE.md`. An owner decision is newer than any canvas: `Data ▸
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
  both changed real results. Use `cartalith-jsmath`'s `js_hypot` and `js_exp`
  (re-exported as `geom::js_*` in `cartalith-urban`), and `js_min`/`js_max`
  because JS propagates NaN where Rust absorbs it. `JS_SEMANTICS_AUDIT.md` has
  the full catalogue.
- **A stale binary reports a healthy `N passed`.** Re-run every mutation
  survivor in isolation.
- **Only `;` starts a comment in `project.godot`.** A `#`/`##` line is parsed as
  data, and a quote or apostrophe inside one silently swallows every key below
  it to the end of the section. **`godot --headless --import` silently strips
  the file's `;` comments** — including the block that warns about exactly
  this. Registering a new `class_name` needs that import pass — diff
  `project.godot` after running it and restore the comment block if it's gone,
  every time, not just once.

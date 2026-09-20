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

  - `Cartalith Gen1 v2.11.html` (2 374 691 bytes) **is tracked and committed at
    this repository's root** — added 2026-08-26 in `4b2c95a`, modified again in
    `b576d56`. So a v2.11 demonstrably exists, and it is in this repository.
  - `reference/Cartalith Gen1 v2.10.html` is untouched since 2026-08-11, and
    `reference/FUNCTION_INDEX.md`'s own first line still reads *"Built against
    `reference/Cartalith Gen1 v2.10.html`"*.
  - **Resolved 2026-09-17 — the root v2.11 is a copy the source has long since
    moved past.** The previous bullet here recorded this as unresolved because
    *"`Cartalith_RC` is not present on this machine."* It is now, so it was
    opened and counted: **164** `Cartalith Gen1 v*.html`, newest **v2.22**, plus
    **49** DCC-line files, newest **v2.71**. The port is measured against a
    reference **twelve mainline versions** behind, not one — and the source
    **forked at v2.22**, so "newest" is two different files and re-freezing means
    choosing a line first. `FUNCTIONAL_CONTRACT.md`'s contradicting *"no drift,
    no re-freeze question to raise"* paragraph was corrected the same day.
  - **The interval is specified, not unknown.** `RC_ENGINE_CHANGES.md` covers
    v2.11 → v2.71 change by change. Until 2026-09-17 nothing referenced it but
    this file, so it sat outside `OUTSTANDING_WORK.md`'s count and outside
    `STATUS.md`; it is `OUTSTANDING_WORK.md` §2.9 now. **How much of it is
    already ported is not established** — do not assume either way.

  The re-freeze itself is real outstanding work, tracked in
  `OUTSTANDING_WORK.md` §2.8 — not here. **Do not record its status in this
  file**; that is exactly the second-source habit the section above forbids.
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
| `URBAN_MORPHOLOGY_SCOPE.md` | Phase 5 — settlement layout, milestones 1-17 and their reasoning. **Status is `STATUS.md`'s** — this row called it *"the project's largest block of unbuilt work"* for nine days after its milestones closed (`9e79e52`, 2026-09-03), which is the auto-loaded-file regression this file's own rules describe |
| `TERRAIN_APPEARANCE_SCOPE.md` | Phase 3's 2D fidelity milestones |
| `UNIFIED_TOOL_PLAN.md` | the tool system, milestones A-F (**root**, not `docs/`) |
| `GPU_LAYER_INTEGRATION_SCOPE.md` | per-layer GPU work, 9 milestones |
| `GPU_COMPUTE_PILOT_SCOPE.md` | the original `wgpu` feasibility pilot |
| `CPU_MULTITHREADING_SCOPE.md` | Rayon parallelisation, 3 milestones |
| `MEMORY_OPTIMIZATION_SCOPE.md` | the measured memory pass |
| `LOD_TILING_BASE_SCOPE.md` | `cartalith-spatial`'s tiling/quadtree base |
| `LOD_TILING_INTEGRATION_SCOPE.md` | threading the tiling base through the pipeline: tiers Z1–Z5, milestones M0–M3 (interactive deep-zoom tiles, the atlas) |
| `LOD_DETAIL_SCOPE.md` | scale-dependent terrain detail toward the owner’s Aletsch target (Ruling K): LOD-D0 zoom-sweep harness, D1 `renderBiomeTileRGBA` port, D2 colour tiles, D3 no-popping morph, D4 ice and snow from existing fields, D5 scale-aware shading and hydrology, D6 off-main-thread synthesis; six owner questions with defaults |
| `ANDROID_BUILD_SCOPE.md` | Android toolchain and the real device passes |
| `GENERATION_PARAMETERS.md` | every exposed generation parameter and its API |
| `MARKDOWN_VAULT_SCOPE.md` | the Markdown Vault: the entity audit that found continents did not exist, and its milestone definitions |
| `STORY_PLANNING_SCOPE.md` | settlement timelines, the conflict overlay and the Journey entity — one subsystem over the Timeline's year cursor; carries the owner's three 2026-08-25 forks |
| `LANDMARK_GENERATION_SCOPE.md` | causally-placed landmarks: the inventory of what this engine already had for it (a golden-verified mountain-pass corridor detector, a TPI-equivalent buried inside the 2D renderer's AO, 15 mineral resources), the Category A/B/C rule carried forward as binding, nine milestones, and six open questions |
| `EXPORT_SCOPE.md` | 16K/32K single-image export — **shelved 2026-08-25 by the owner**, findings only. Records that the reference's own bake draws terrain and nothing else, the four measured gaps in today's export, the render-once decision that would have to be reversed, a banded renderer that was prototyped and measured byte-identical before being reverted, and the codec survey (WebP eliminated at 16 383 px, JPEG XL at its AGPL encoder) |
| **Direction and reference** — inputs, not plans | |
| `FUNCTIONAL_CONTRACT.md` | the HTML app's capabilities vs. this port, tagged per `DECISIONS.md` §7d |
| `PORT_ONLY_FEATURES.md` | the reverse of `FUNCTIONAL_CONTRACT.md`: features and refinements this port has that the legacy v2.11 HTML does not, each checked at its symbol and against the reference (2026-09-13), with the candidates that turned out to be ports. A list, not a status — status is `STATUS.md`’s |
| `RC_ENGINE_CHANGES.md` | **the HTML's simulation/generation changes from v2.11 to v2.70**, as a porting spec — names the function and constant each change lives in, why each number is that number, and which harness verified it. Claims nothing about port status; that is `STATUS.md`'s. Note its first section: the HTML forked at v2.22 and every engine change from v2.25 on exists only on the DCC line. **The newest row is v2.68 in §8.2, and it names a defect a faithful port INHERITS**: the farmland generator has pushed `field`/`pasture` polygons into `model.details` since v0.95 and **nothing has ever drawn one** — neither map renderer reads `model.details` at all and the City Viewer's detail pass branches on nine kinds with neither among them, so 62–86 field polygons per village fell through every `else if` in silence. **Assert that every detail kind the generator can emit is reachable by the renderer, derived from a real town rather than a hand-list** (v2.65's rule, third instance). Its other finding is a build a port should NOT make: furrow hatching with a per-parcel plough direction needs no hatch pass and no stored bearing, because each grant is already a **selion** (5.8 × 91.5 m, aspect 15.8 : 1) and a furlong is a bundle of parallel ones — the texture IS the geometry. **§6s is the newest block, and BOTH its findings belong to a port whatever it decides about the feature**: `buildParcels` decided plot grain with a hardcoded `dM<160` while `assignDistricts` answered the same question 130 lines later with SEVEN wards, so a harbour, a suburb, an agrarian fringe and a riverside craft quarter all platted identically — and **the obvious feature, ward-driven DEPTH, is INERT**: 67.6% of parcels never reach `depthTarget`'s own 14 m floor (the block waist binds, not the draw) and tripling `plotDepthVariance` 0.22 → 0.60 moves median depth 11.09 → 11.07 m, which also explains why realised aspect is 1.09-1.74 against M-PAR-2's 1:3-1:10 band (a block-SIZING question). What ships is the SUBDIVISION, and it must be verified as a per-ward SIGN against that ward's own baseline, never as a ranking across wards — mean frontage per ward is not a function of pressure alone, because the blocks differ per ward. **§6s.4 is the other one**: `buildParcels`' water rejection has sampled only the four CORNERS since v0.95, and a plot spanning a narrow channel has every corner dry (the goldens caught one 41.3 m-deep, 5.8 m-wide parcel with its middle in the river) — pre-existing, unreachable until the grain varied, and proven so by measuring the prior version at `plotDepthVariance 0.60`: zero wet parcels. **§6r is the block before, and §6r.5 is the one a port must read whatever it decides about UI**: the layout engine has exported a 22-parameter generation-rules table since v0.95 and `cityGen` reads `opts.rules` on its first lines — **and the host adapter has never set it**, so every town the HTML has ever drawn came out at the defaults. Exposing those parameters reached a **NON-TERMINATING region of the engine's own documented range**: `buildParcels` re-draws a frontage grant until one fits the remaining edge, unbounded, and the escape probability collapses with `frontageWidthVariance` (the 0.22 DEFAULT already expects ~28 571 retries against a 4.6 m remainder; 0.12 — the proof of concept's own 'Planned Grid' profile — effectively never escapes). **A retry-until-it-fits loop over a heavy-tailed draw is a hang waiting for a parameter change.** The fix is a BOUND, and **the obvious claim for a bound — that it cannot change a terminating run — is FALSE here**: the goldens pass because their fixtures never reach it, while an ordinary town runs to a measured 172 644 spins, so it is a deliberate re-baseline sized by measurement at 10 of 12 towns and 46 of 8 939 parcels (0.51%). Found by BISECTION, not by reading. §6q is the block before: the status gradient made EXPLICIT as `par.status` (proximity to the market, intramural-or-not, and how far downwind on §6p's bearing) — verify its SHAPE, not its existence: 0.662 near the market against 0.218 at the edge, and outer ground 0.143 downwind against 0.403 upwind. **§6q.4 is the trap worth reading**: there are TWO district palettes, building tint and parcel fill, an unknown district is silently skipped by the second, §6p fed only the first, and §6p's own probe asserted only the palette it had remembered — strengthening that to cover every district actually OBSERVED then surfaced a pre-existing hole of the same shape. **A hand-list of things to check is the same defect as a hand-list of things to define.** §6p — the two SITE-MODEL VECTORS `docs/05` §7.1 asked for (prevailing wind from `currentWindField()`, downstream from §6n's receiver tree) shipped with their first consumer, the §4.7 industry-siting table — a field with no consumer is dead code. **Two findings a port should not rediscover**: `_civRiverFlowField` fills `km2` on EVERY cell and `fx`/`fy` only on channel cells, so a nearest-cell search keyed on catchment lands on the town's own dry ground (**1 of 14 towns got a bearing; 14 of 14 when keyed on the vector**); and **"downstream of the market" is unsatisfiable for most towns** — one town held 62 riverside parcels with all 62 upstream of its market, so §4.2 means the downstream END of the town's own frontage, an ORDER along the flow needing no origin (§6n's lesson in a second subsystem). §6o — **seven generation constants became runtime parameters** (`CIV_PARAM_DEFS`/`civParam`), their defaults ARE the constants they replaced, and `state.civParams` starts empty — so bit-identity is structural and **a port with no parameter UI can keep all seven as constants and skip the section**. Read 6o.4 before touching `foodSurplusRatio`: BOTH ag-tech branches must scale with the ceiling or one parameter means two things, and the industrial case is correctly INERT (0.750 unchanged) because the cap is not the binding constraint there. 6o.5 names three neighbours refused on a constraint — `FARMERS_PER_URBANITE` is already per-faction (v1.54), `FOOD_BASE_SURPLUS_RATIO` is the bit-identity pin, `SETTLE_COAST_SWAP_TOLERANCE` is an internal tolerance. §6n — a **per-cell** cost cannot tell walking ALONG a river from cutting ACROSS it, so the ford was charged on every river CELL (following a channel priced as fording it once per cell) while the navigable discount was multiplied into every river cell (a road crossing square-on collected 35% off the cell it was bridging): the most navigable river on the map was its **most expensive ground, 3.46x** plain. Both terms move to the EDGE — the same lesson §4 already applied to slope and did not carry ten lines down — giving ALONG **0.41x** / ACROSS **1.81x**, and `|align|` is symmetric by construction because an undirected Prim MST has no answer for an asymmetric cost (measured asymmetry **exactly 0**). **This is where §6k's recommendation gets taken, for a new consumer**: navigability is keyed on **catchment km², not Strahler order**, because one `order>=3` label spans **6 417 km² at 800 km against 1 429 009 km² at 40 000 km** while the catchment is resolution-free (483 vs 482 km² across a 4x cell-count change) — the three existing `order>=3` consumers are **untouched**, that migration is still open. The planner's current came from the route's ELEVATION PROFILE and was **backwards one step in five**; it reads the receiver tree now, asserted as a ROUND TRIP. §6m — a river was drawn as a translucent Beer-Lambert TINT at an alpha carrying the DISCHARGE MAGNITUDE while a lake is OPAQUE, so a small river painted at **38% opacity** through a flat **2.5 km** slab: a river's alpha is its COVERAGE, never its discharge, and the antialias band is bounded by the channel rather than by a pixel. In the same pass, a pooled depression was gated on LOCAL RAINFALL — right for an unfed hollow, wrong for a TERMINAL lake (Chad, Eyre, the Aral and the Dead Sea all fail that test and all are lakes), so it now also accepts `flowField >= riverFlowThresh`, which adds no constant; and v2.60's sculpt-derived digging pass is REVERTED, `field` byte-identical to v2.59, so **a port that has not implemented it should not**. §6l — a river was rasterised as a chain of ONE-CELL discs (`halfW` floors at 0.5, so `r=ceil(0.5)=1` paints only the centre cell), and a D8 chain steps diagonally 42% of the time, so **884 of 1 305 main stems broke into parts**; the carve had the right width since v2.30 and nobody carried it to the render stamp. Assert **4-connectivity** — 8-connectivity is free on a D8 chain and passes on the broken build. §6k — the source turned depression-filled routing ON by default (a deliberate re-baseline: 68.5% of land drained into an interior pit before it) and, in the same pass, measured its Strahler-order currency and ruled on it: the order ladder is resolution-STABLE here but EXTENT-dependent (`order>=3` covers 0.32% of the channel network on an 800 km map and 4.10% on a 40 000 km one), spans 123.7x in catchment inside its own top bucket, and is not monotone — so keep the ordinal tier but key every threshold on catchment AREA (§7.12), which a port writing these consumers from scratch gets for free. §6j — river SELECTION had no scale term anywhere, so a 50 km region and a 40 000 km world chose the same rivers; a traced polyline is a FRAGMENT whose length anti-correlates with importance (rho 0.207 -> 0.963 once whole main stems are assembled, **against upstream channel cells, NOT catchment area — corrected in §6j's own box**), and it moves no generated value, which is why it sits in §8.1 rather than §8.2. §6i — the coastline was the level set of a blur of a piecewise-constant plate Voronoi map, and `PLATE_BASE_BLUR_K` is the single highest-leverage constant in the height formula; §6h holds the three independent floors (storage, the relief gate, the global ramp) that each flattened a deep-zoom plain. **Read §8's heading before appending to it** — it claimed "not simulation" while holding three deliberate height-field re-baselines, and was split into §8.1/§8.2 on 2026-09-17 |
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
| `HYDROLOGY_CLASSIFICATION_RESEARCH.md` | owner-supplied, imported verbatim 2026-09-08 — lake/sea/marine-basin classification by hydrological **topology** rather than surface area. **It contradicts what the engine does today**: `build_water_bodies` calls the *largest* below-sea component the ocean and every other one a lake, which is size-primary and has no connectivity, basin-topology or map-boundary state. Changing it is a **golden re-baseline** (it is `buildWaterBodies`, reference line 5753) and needs an owner ruling |
| `docs/research/lod extra info.md` | owner-supplied 2026-09-12 — **left where the owner put it, inside the *source* project’s `docs/` tree (naming hazard 1 above) although it targets this port’s Rust renderer.** Scale-dependent terrain detail: a multiresolution pyramid, LOD chosen by physical scale rather than GUI zoom, continuous transitions, macro/meso/micro normals, snow/rock/glacier/hydrology derived from terrain, and an acceptance test of one continuous zoom with no popping. Cites Losasso & Hoppe (Geometry Clipmaps), Asirvatham & Hoppe (GPU Gems 2), Strugar (CDLOD), Li et al. 2021, Godot Terrain3D and Unreal Landscape. **Its crate names are not this workspace’s** (`cartalith-grid`, `-compute`, `-worldgen`, `-cartograph`, `-render`, `-logistics`, `-project`, `-types` do not exist). **Much of it is already built**; the gap it describes is the one `OUTSTANDING_WORK.md` measured as *"LOD tiles carry a SHADE RATIO, not colour"* — read that row first |
| `LANDMARK_GENERATION_RESEARCH.md` | owner-supplied, imported verbatim 2026-08-30; a geographic-causality framework for landmark placement (TPI, viewshed, least-cost path, Poisson-disc, spatial interaction) — see `LANDMARK_GENERATION_SCOPE.md` for what of it this port already has |
| `REFERENCE_MAP_RECONSTRUCTION_RESEARCH.md` | owner-requested 2026-09-13 — load an existing map image as an adjustable-opacity reference and sculpt a heightmap over it, with land/river edge detection and hand-drawn tectonic plates whose edges roughen like the generator’s warp. Research and a proposed design (RM-0…RM-11), **not a scope**: no reference ancestor, no design canvas yet, and eleven owner questions gate the build rows. Finds that noise β roughens height rather than plate edges, and that opened projects cannot be sculpted today |
| `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` | owner-requested 2026-09-20 — a hard, numbered acceptance bar ("always correctly detailed, no pixelated/square artifacts, no matter the zoom") for real detail on zoom, for rivers and mountains, without upsampling; raster stays the simulation substrate. EF-0…EF-9 propose a queryable multi-resolution refinement primitive, boundary-seeded local hydrology re-accumulation for real river tributaries, a generalized vector layer, and a tile structure grounded in already-real, unwired primitives. **EF-7 is Ruling N** (`LARGE_ITEM_RULINGS.md`): settlement river/coastal binding is a proxy today (byte-for-byte the same in the legacy HTML and this port) — the owner authorised siting itself to change, gated on it staying a term inside the existing suitability ranking, not a bypass. Deliberately not a vertex/TIN replacement of the simulation grid. Research and a proposed design, **not a scope**: most of six original owner questions resolved; EF-9 (generic importance-driven refinement, split out from EF-7) and doc placement remain open |
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

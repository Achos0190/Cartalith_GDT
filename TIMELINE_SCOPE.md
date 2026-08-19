# Timeline / collapse-recovery simulation: milestone plan

`FUNCTIONAL_CONTRACT.md` §4 flagged a **Timeline/collapse-recovery
simulation** layer as present in the reference and completely untracked by
this port anywhere — approved for build by the owner, 2026-08-19. This
document is that scoping pass: the function list re-verified line-by-line
against `reference/Cartalith Gen1 v2.10.html` (not trusted from
`FUNCTIONAL_CONTRACT.md`'s own high-level pass, per this repo's own working
rule — "verify a scope document's line ranges against the real reference
before slicing"), the design intent behind it, what this port already has to
build the engine work on, and a milestone breakdown in the same shape
`JOURNEY_PLANNER_SCOPE.md`/`GPU_LAYER_INTEGRATION_SCOPE.md` used for their own
subsystems.

**Correction to the task brief this doc was scoped from**: the brief
described `docs/research/collapse-timeline-dynamics.md` and
`settlement-emergence.md` as living only in a sibling `Cartalith_RC` repo not
present here. No such sibling repo exists on this machine (checked
`C:\Users\Vincent\`, `D:\Users\Vincent\`, and both drive roots) — but both
documents are already vendored **in this repo**, at `docs/research/
collapse-timeline-dynamics.md` and `docs/research/settlement-emergence.md`
(the *source project's* `docs/`, per this repo's root `CLAUDE.md` naming-
hazard #1 — not `cartalith-native/docs/`). Both were read in full for this
pass; §2 below summarizes them. `settlement-emergence.md` has one relevant
section, "## 5. Post-collapse recovery model", not two — there is no
separate "§6"; the brief's "§5-6" was imprecise, corrected here rather than
inventing content for a section that doesn't exist.

## 1. Verified reference function list

`FUNCTIONAL_CONTRACT.md`'s line range (~20597-26478) and function list
(`civAddYear`/`civGotoYear`, `_civSimulateTimeline`/`_civCollapseStep`/
`_civRecoveryGrowthStep`/`_civRunCollapseSimulation`) both check out **as a
loose envelope**, with two corrections:

1. **The range is not contiguous Timeline code.** Lines ~20597-26478 span
   ~5,880 lines, but the vast majority of that span is unrelated civ
   machinery already ported or tracked elsewhere — settlement pins/POI
   drawing, faction inspector rendering, snap-to-water/coast logic, the
   entire routing/road-network/MST/hierarchical-network pipeline, population
   and trade-balance math, province generation, village seeding, context
   menus, the label/way/journey editors, the route-planner UI. Real Timeline
   code is four disjoint clusters inside that range, totaling well under 500
   lines. Slicing the doc as "lines 20597-26478" would pull in code several
   other scope documents already claim.
2. **The list under-names two real pieces**: `civSnapshotSave`/
   `civSnapshotLoad`/`civRemoveYear` (the actual per-year snapshot
   read/write, not just the two entry points named), and the whole
   playback/UI-wiring cluster (`_civTlStartPlay`/`_civTlStopPlay`/
   `_civWireYearSlider`/`_civBuildExploreTimelineUI`). Both are load-bearing
   for "timeline playback," which the FUNCTIONAL_CONTRACT prose already
   claims as in-scope.

### Cluster A — manual timeline authoring + snapshot storage (lines 20563-20662)

| Function / symbol | Lines | What it does |
|---|---|---|
| `let civTimeline=[]`, `let civYear=0`, `let civWays=[]` | 14756-14758 | Global state. `civWays` is already live in this port (`CivData::ways`); `civTimeline`/`civYear` are net new. |
| `_civAssignTid` | 20564 | Lazily stamps a stable, monotonic `tid` onto a place/way object the first time it's touched. |
| `_civResyncNextTid` | 20565-20574 | Rescans `state.places`/`civWays`/every `civTimeline` entry for the max `tid` seen, so a freshly loaded save's next-assigned id never collides with historical ones. |
| `_civYearDiffInvalidate` / cache vars | 20575-20576 | Invalidates the memoized year-diff. |
| `_civYearDiff` | 20580-20595 | Diffs the active year's snapshot against the chronologically-previous one by `tid` set → `{present, removed, added}`. Powers the "exist only in this year" filter and the ghost/highlight overlays. Cached per `civYear`. |
| `civSnapshotSave` | 20596-20606 | Captures the *live* `getCivTerritory()` + `state.places` + `civWays` into (or over) the `civTimeline` entry for a given year, sparse-encoding territory as `[i, factionId, ...]` pairs. Assigns `tid`s to every place/way as a side effect. |
| `civSnapshotLoad` | 20607-20614 | Restores territory paint from a year's snapshot into the live `civTerritory` grid. **Never touches `state.places`/`civWays`** — those stay the single always-current, always-editable arrays every other system (pathfinding, planning) reads. |
| `civGotoYear` | 20615-20617 | `civYear = year`, load its snapshot, rebuild the timeline UI. |
| `civAddYear` | 20618-20634 | Snapshot the *current* year (so it isn't lost), then create a new entry for `year` carrying forward territory/places/ways from the nearest earlier entry (or the live state, if the timeline was empty) — then jump to it. |
| `civRemoveYear` | 20635-20641 | Deletes a `civTimeline` entry; if it was the active year, jumps to the earliest remaining one (or year 0 if none remain). |
| `_civFormatYear` | 20644 | `-1200 → "1200 BC"` / `450 → "450 AD"`. |
| `_civBuildTimelineUI` | 20645-20662 | Renders the pill list (one per recorded year) into the sidebar panel; delegates the slider/playback row to Cluster C. |

### Cluster B — mechanistic collapse/recovery simulator (v0.85, lines 24608-24950)

Everything here is deliberately **pure and deterministic** — "the model
consumes no randomness — same inputs always replay the same history" (the
reference's own comment at `_civSimulateTimeline`). This is a real gift for
golden-parity testing: no RNG-stream-alignment risk, no iteration-order
sensitivity beyond plain array order.

| Function / symbol | Lines | What it does |
|---|---|---|
| `_civUpdatePopReadout` | 24596-24606 | UI-only readout string; references `_civRecoveryPhase` (see the "adjacent, not in scope" note below). Not part of the timeline data model. |
| `_CIV_RECOVERY_FRAC` / `_CIV_RECOVERY_NAME` / `_CIV_TIER_ORDER` / `_CIV_TIER_FLOOR` / `_civTierForPopulation` | 24614-24618 | Shared tier-floor table (`hamlet`→`metropolis`) and population→tier lookup. Used by **both** the v0.82 static recovery pass and the v0.85 stepper below. |
| `_civApplyRecovery` | 24619-24640 | **v0.82, static/instant** re-weighting — not the year-stepped mechanism. See "Adjacent, not in scope" below. |
| Character weight / rate-ceiling constants (`_CIV_COLLAPSE_CHAR_WEIGHTS`, `_CIV_COLLAPSE_MAX_MORTALITY`, `_CIV_COLLAPSE_MAX_MIGRATION`, `_CIV_COLLAPSE_MIGRATION_BIAS`, `_CIV_MIGRATE_BETA`, `_CIV_ABANDON_FLOOR`, `_CIV_FORTIFIED_BONUS`) | 24653-24666 | Every tuned constant `collapse-timeline-dynamics.md` §3-5 derives or cites. |
| `_civProximityAdjacency` | 24672-24683 | World-wrap-aware symmetric k-nearest-neighbour graph among settlements, in real km — the stepper's **own** network representation, deliberately decoupled from the rendered `ways` array (whose indices go stale as settlements are removed step-to-step). |
| `_civBetweennessFromAdjacency` | 24687-24709 | Brandes (2001) betweenness centrality over a prebuilt adjacency list. A **second, standalone** betweenness implementation — not a call into `_civNetworkMetrics` (the already-referenced route-network metrics function), and neither exists in this port yet (see §3). |
| `_civSettlementStress` | 24713-24723 | Per-settlement stress in [0,1]: trade-dependency loss `L` (needs a `baselineNormB` captured at simulation start), density/connectivity exposure `D`, undefended-violence exposure `V`, blended by the active collapse **character**'s weight triple. |
| `_civMortalityMigrationRates` | 24726-24731 | stress × severity × character → this step's annual excess-mortality fraction `m` and out-migration fraction `g`. |
| `_civGravityMigrate` | 24738-24778 | Zipf/Ravenstein gravity-model redistribution of each origin's migrant pool across every other settlement, weighted by `headroom × fortifiedBonus / distance^β`, in up to 4 saturation-aware passes; returns per-destination `received` plus system-wide `unplaced` (diaspora loss). |
| `_civCollapseStep` | 24785-24848 | One `stepYears`-long collapse step: rebuilds the proximity graph + betweenness, computes stress/mortality/migration per settlement, redistributes migrants, re-derives tiers (demoting/marking `ruins` where a nucleus falls below its floor), drops anything under the abandonment floor. Returns a new places array + `{died, migrated, unplaced, failed}` stats + `normBByTid` (threaded forward as each step's stress baseline). |
| `_civRecoveryGrowthStep` | 24852-24870 | One `stepYears`-long logistic-regrowth step (Verhulst) toward each settlement's catchment ceiling; re-derives tiers upward, clearing `ruins` on promotion back into an exchange tier. |
| `_civSimulateTimeline` | 24875-24892 | Pure orchestrator: runs `steps` collapse-or-recovery steps from a starting places array, returns one `{places, stats}` snapshot per step. Never touches `civTimeline`/`state.places` itself. |
| `_civRunCollapseSimulation` | 24896-24950 | **Impure wiring**: reads the sim-panel UI fields + `state.places`, calls `_civSimulateTimeline`, and writes one `civTimeline` entry per step (anchoring a "before" frame at the simulation's start year, carrying territory/ways forward unchanged from the nearest prior entry — collapse doesn't redraw political borders). Warns before silently overwriting existing timeline years. |

### Cluster C — scrub/playback UI wiring (lines 26424-26493)

| Function / symbol | Lines | What it does |
|---|---|---|
| `_civTlStopPlay` | 26425-26428 | Clears the playback interval timer, resets the Play button label. |
| `_civTlStartPlay` | 26429-26440 | Advances `civYear` to the next recorded year every 1200ms via `setInterval`, stopping at the end. |
| `_civTlDragSrc` guard + `_civWireYearSlider` | 26451-26474 | Wires a single real-time-scale slider (`min`/`max`/`value` are actual years, not a snapshot-count index — v0.91) with a `<datalist>` for proportional tick marks; dragging snaps to the nearest recorded year. |
| `_civBuildExploreTimelineUI` | 26478-26493 | Wires the whole Explore→Timeline section: slider row visibility (gated on ≥2 recorded years), Play button, and the three filter checkboxes (`timelineExistOnly`/`timelineGhost`/`timelineHighlight`, read/written on `state.mapFilter`). |

Markup for all of this lives at lines 1888-1952 (`#explTimelineSection`) —
useful as a literal control inventory when a UI milestone is scoped, listing
every id (`civTlYear`, `civTlAddYearBtn`, `civTimelinePanel`,
`explTimelineSlider`, `explTlPlayBtn`, `explTlExistOnly`/`Ghost`/`Highlight`,
`civSimMode`/`civSimCharacter`/`civSimSeverity`/`civSimRate`/
`civSimStartYear`/`civSimDuration`/`civSimStepYears`/`civSimulateBtn`/
`civSimOut`) the wiring functions above bind to.

### Cluster D — save-format persistence (not named by `FUNCTIONAL_CONTRACT.md`, found during this pass)

`civTimeline`/`civYear` **are** part of the reference's `.zip` save format —
`_civSyncToState` (lines 26115-26139) serializes `state.civ.timeline`/
`state.civ.year`; `_civSyncFromState` (26140-26159+) restores them and calls
`_civResyncNextTid()` so freshly-created ids in a loaded project never
collide with historical ones. **`SAVEFILE_COMPAT.md` does not mention
`civTimeline`/`civYear` anywhere** — a real documentation gap this port's own
save format (`cartalith-io`) will need to address once/if Timeline
persistence is in scope (see Open Questions).

Also found: `generate()`'s wrapper (lines 26211-26224) clears
`civTerritory`/`civTimeline`/`civYear` back to empty on every fresh
procedural generation (not on a loaded save) — the Rust equivalent of
`CivData` already re-derives from scratch on `generate()`, so this is a
"make sure the new field resets too" note, not new design.

## 2. Design intent (from the vendored research docs)

### `docs/research/collapse-timeline-dynamics.md`

Written explicitly as "research + design foundation" for the v0.85 stepper
above, building on `settlement-emergence.md`'s v0.81/v0.82 work. Its
argument, compressed:

- **What v0.82 already did, and the gap this closes**: the static Recovery-
  phase pass answers "what would a Survival-era world look like," once. What
  was actually wanted is *process* — "how did it get that way, year by year,
  which settlements failed first, and where did the survivors go" — written
  into the timeline so it can be scrubbed like real history.
- **Three variables govern any real settlement-system collapse** (energy/food
  availability, transport/trade capacity, landscape carrying capacity), and
  the engine already computes analogues for all three (`carryingCapacity`/
  `_civSettlementPopulation`, and the betweenness/closeness network metrics
  that already drive exchange-tier population).
- **Which settlements fail first is not one universal ranking.** Three
  citable archetypes, each the *opposite* ranking from another: trade-
  collapse hits the biggest trade-dependent hubs hardest (Late Bronze Age
  Collapse, Cline 2014; targeted-attack network fragility, Albert/Jeong/
  Barabási 2000); disease hits dense, well-connected settlements hardest
  (Black Death, Benedictow 2004 — the *opposite* direction from trade-
  collapse); conflict kills undefended settlements regardless of size,
  fortified ones persist (post-Roman West, Wickham 2005). A **Character**
  dial (trade/disease/conflict/mixed) sets three independently-weighted
  stress components (`wL`/`wD`/`wV`) so the failure order is a real,
  historically-grounded choice per run, not a hard-coded assumption.
- **Stress → mortality/migration** via a **Severity** dial scaling two rate
  ceilings calibrated against the Black Death's own annualized mortality
  (~13.6%/yr derived from ~45% over 4 years; ceiling set at 15%/yr).
  Migration uses a **gravity model** (Zipf 1946 / Ravenstein 1885,
  distance-decay exponent β=1.5, literature-typical), with a fortified-
  settlement attractiveness bonus and unplaced migrants tracked as
  transit/diaspora loss.
- **Tiers re-derive from new population** using the same `_civTierForPopulation`
  the static v0.82 pass uses — demotion into ruins, abandonment below a
  population floor. This is explicitly framed as the mechanism that makes
  settlements *appear/disappear* on the timeline, "which the existing
  tid-diff ghost/highlight/exist-only overlay already visualises with no new
  rendering code."
- **§7, "wiring into the existing timeline — no new data model needed"**:
  the design's own stated integration plan is that the simulator's only job
  is to run the per-step math N times and push one snapshot per step in
  the *exact same shape* `civAddYear` already produces, so the existing
  slider/ghost/highlight overlay work unmodified on simulated history.
  Explicitly preserves the invariant that jumping to a recorded year
  overwrites *territory* but never `state.places`/`civWays` (the live,
  always-editable arrays).
- **§8, deferred**: new-settlement founding from displaced/transit
  populations, true travel-cost migration distance (vs. the Euclidean
  approximation used), and regrowth-phase migration (people moving back
  toward reviving hubs, not just uniform in-place logistic growth). None of
  these are part of what's being scoped here — noted so a future pass
  doesn't mistake "the reference doesn't have this either" for "this port
  missed it."

### `docs/research/settlement-emergence.md` §5, "Post-collapse recovery model"

The earlier (v0.82) design this port's v0.85 stepper builds on:

- Collapse doesn't reset the landscape — roads, ruins, fields, wells,
  bridges, mines, irrigation persist, but infrastructure decays faster than
  ecological knowledge, so a recovery population inherits *excess
  buildings, insufficient labour, damaged ecosystems*. Recovery runs
  **below** the ecological ceiling.
- Four named phases (**I Survival** <10% of former population, **II
  Subsistence** 10-30%, **III Regional** 30-70%, **IV Mature** 70%+), each
  with its own settlement logic (resource extraction/cluster-on-water-and-
  ruins → repopulate abandoned villages → crafts/markets/roads/politics
  return → prior economic geography mostly returns, permanently losing some
  sites).
- **Recovery does not delete settlements — it re-scores them**:
  `SettlementValue = Infrastructure + AgriculturalPotential + WaterAccess +
  StrategicPosition − MaintenanceCost`. A ruined city becomes a small
  fortified settlement inside its own ruins — the same "demote, don't
  delete, mark ruins+fortified" pattern `_civApplyRecovery` and
  `_civCollapseStep` both implement.

## 3. What this port already has to build on

Checked directly against `cartalith-native/crates/cartalith-civ/src/lib.rs`
(14,873 lines) and `cartalith-native/crates/cartalith-godot/src/lib.rs`.
This port has **no persistent "CivData" struct in the reference's sense
until you look at `cartalith-godot`** — `cartalith-civ` itself is entirely
stateless pure functions (matching `ARCHITECTURE.md`'s crate boundary: no
mutable world-state ownership below the Godot boundary layer). The actual
mutable civ state lives in `cartalith-godot/src/lib.rs`'s `WorldGen::civ:
Option<CivData>` (struct at lines 91-166), recomputed by
`compute_civilisation` right after `generate()`/`generate_world_structure()`
and holding: `settlements: Vec<NamedSettlement>`, `ways: Vec<Way>`,
`sea_routes: Vec<SeaRoute>`, `territory: Vec<i32>`, `provinces: Vec<i32>`,
`province_list: Vec<Province>`, `trade_balances: Vec<TradeBalance>`,
`explanations: Vec<SettlementExplanation>`, `water_bodies: Vec<u8>`. **This
struct has no `timeline`/`year` field today** — Timeline is entirely new
surface on it.

**Real, load-bearing gaps found while tracing dependencies** (not just "this
function specifically isn't ported" — these are prerequisites the stepper
literally cannot run without):

1. **No stable per-object id.** `NamedSettlement` (line 3687,
   `{placement: SettlementPlacement, name: String, pop: u32}`) and `Way`
   (line 5364) have no field playing `tid`'s role. The reference invents
   `tid` lazily via `_civAssignTid` and threads it through every snapshot so
   `_civYearDiff` can tell "this is the same settlement, just renamed/moved"
   from "this is a different settlement" across years. This port needs an
   equivalent — most naturally a `tid: u64` (or similar) field added to
   `NamedSettlement`/`Way`, assigned once at placement/road-generation time
   and carried through every snapshot, rather than reconstructing identity
   from position/name matching.
2. **`SettlementKind` has no `Metropolis` variant** (5 variants: `Capital`,
   `City`, `Town`, `Village`, `Hamlet` — confirmed at lines 2996-3002), by
   the port's own deliberate, already-documented decision (comments at
   lines 1726-1727, 3665-3666: metropolis is a separate opt-in promotion
   pass, `_civSelectMetropolises`/reference line 24961, not yet ported
   either). The reference's `_CIV_TIER_ORDER`/`_CIV_TIER_FLOOR` used by
   *this* subsystem's tier re-derivation has six tiers, metropolis highest.
   Whether the timeline stepper needs a real `Metropolis` variant, caps at
   `Capital`, or needs the promotion pass ported alongside it is a real
   design decision, not a detail — see Open Questions.
3. **`_civSettlementPopulation` and its whole dependency chain are
   unported.** The collapse stepper's migration headroom/ceiling
   (`capField`) and the recovery stepper's logistic ceiling both call
   `_civSettlementPopulation(place, K, opts)` (reference lines 23502-23511),
   which itself calls `_civCatchmentPop` (23484-23500) →
   `_civCatchmentDensityMean` (23461-23469) and reads `currentAgrarianDensity()`
   and the constant tables `_CIV_SURPLUS_FRACTION`/`_CIV_TRADE_K`. Checked:
   none of `_civSettlementPopulation`, `_civCatchmentPop`,
   `_civCatchmentDensityMean`, `currentAgrarianDensity`, `_CIV_SURPLUS_FRACTION`,
   `_CIV_TRADE_K` exist in `cartalith-civ`. What **does** already exist and
   is directly reusable: `build_carrying_capacity` (line 732, the `K` field
   itself, golden-verified per `golden_parity_carrying_capacity.rs`) and
   `civ_catchment_km2`/`civ_catchment_radius_cells` (lines 1729/1741 — the
   two small pieces `_civCatchmentRadiusRaw`/`_civCatchmentRadiusCells`
   correspond to). The gap is real but bounded: roughly 4 small functions
   plus 2 constant tables, not a rediscovery of carrying capacity itself.
4. **No betweenness-centrality implementation exists in Rust at all.**
   Neither `_civNetworkMetrics` (the already-referenced route-network
   metrics function the design doc cites as "already in the engine" — true
   of the *reference*, not of this port) nor a standalone Brandes
   implementation exists anywhere in `cartalith-civ`. This is fine for
   scoping purposes — `_civProximityAdjacency`/`_civBetweennessFromAdjacency`
   are self-contained (take a places array + `cellKm`, no dependency on
   `ways`/routing state) and can be ported as their own milestone without
   waiting on `_civNetworkMetrics` ever landing — but it means "just call
   the existing betweenness code" is not an option; a fresh Brandes port is
   real work here.
5. **`_civApplyRecovery` (v0.82, static) is unported and adjacent, not
   in scope.** It is called from `_civIterativeAutoWorld` (reference line
   25761) as part of auto-populate's "Recovery phase" dropdown (markup at
   line 1424-1425) — an instant, one-shot re-weighting of a freshly
   generated world, entirely separate from the year-stepped v0.85 timeline
   this document scopes. It shares the tier tables (`_CIV_TIER_ORDER`/
   `_civTierForPopulation`) with the v0.85 stepper, which is why it's
   commented in the same reference block, but it is a `PHASE2_SCOPE.md`-
   adjacent gap (auto-populate's own feature set), currently untracked
   there too. Milestone 1 below ports the shared tier tables in a form
   both consumers could use; whether to also port `_civApplyRecovery`
   itself is left as an explicit choice in that milestone rather than
   silently bundled in or silently dropped.

## 4. The DCC shell's own "Timeline" region — a naming collision, not a duplicate

`DCC_CONTROL_INDEX.md` §10 ("Viewport, timeline, status bar") already
specs a Timeline region — scrub track, ▶/⏸/step transport, ×1/×10/×100
speed control, and **six live simulation-layer toggles (Climate ·
Population · Economy · Politics · Infrastructure · Warfare)** — and marks
essentially all of it `engine gap`, with an explicit note: *"The engine is a
one-shot static generator by explicit, repeated owner decision... Not a gap
to close — a product decision"* (§5 of that document's own open-questions
list, unresolved as of this pass).

**That is a different, larger, still-undecided feature from the one this
document scopes.** The reference's actual `civTimeline` is a **discrete,
snapshot-based** mechanism — a handful of authored-or-simulated *years*,
each a full state snapshot, scrubbed between (never interpolated, never
continuously re-simulated) — not a live, continuously-running world
simulation with per-domain toggles. `_civRunCollapseSimulation` itself only
ever produces a bounded number of discrete steps and writes them once; there
is no "Warfare" domain in the reference's civ model at all.

**Practical consequence for milestone 6 below (UI playback controls)**: the
scrub track + ▶/⏸/step controls in `DCC_CONTROL_INDEX.md` §10 map
surprisingly well onto `civGotoYear`/`_civTlStartPlay`/`_civTlStopPlay` —
building this subsystem's UI could plausibly *close* that specific part of
the §10 gap list. But it must not be read as authorization to build the
six-toggle continuous-simulation feature or Warfare — those remain the
owner's open product decision, untouched by this scope. Flag this
distinction explicitly to the owner (or whoever picks up milestone 6) before
wiring anything into the DCC shell's Timeline region, so "I built the
Timeline" isn't read as "I built the thing §5 was still asking about."

Separately: this repo's root `CLAUDE.md` states "All UI work is on hold
(owner, 2026-08-18)" — but `DCC_SHELL_SCOPE.md`'s own top-of-file notice
records that the hold was **lifted later the same day** ("✅ THE HOLD IS
LIFTED — BUILD IT"). `CLAUDE.md` is stale on this specific point; per this
repo's own "expect these documents to age" rule, UI work (including
milestone 6) is not blocked, but should be designed against
`DCC_SHELL_SPEC.md`/`DCC_CONTROL_INDEX.md` rather than the ad hoc raw-slider
markup the reference itself used, consistent with the standing note not to
bolt controls on without a dedicated UI/UX pass.

## 5. Milestones

### Milestone 1 — shared prerequisites: population-ceiling chain + stable ids

Not glamorous, but both later milestones are blocked on it:

- Port `_civSettlementPopulation`'s dependency chain: `_civCatchmentDensityMean`,
  `_civCatchmentPop`, `currentAgrarianDensity`-equivalent (or confirm an
  existing density source already covers it), `_CIV_SURPLUS_FRACTION`/
  `_CIV_TRADE_K` constant tables, and `_civSettlementPopulation` itself.
  Golden-testable in isolation (pure, no RNG) — extract fixtures from the
  reference the same way `golden_parity_carrying_capacity.rs` did for `K`.
- Port the shared tier tables (`_CIV_RECOVERY_FRAC`/`_CIV_RECOVERY_NAME`/
  `_CIV_TIER_ORDER`/`_CIV_TIER_FLOOR`/`_civTierForPopulation`) as their own
  small module — used by both this subsystem and (optionally) `_civApplyRecovery`.
- Add a stable id field to `NamedSettlement`/`Way` (or an equivalent
  side-table keyed by index/generation), matching `_civAssignTid`'s role.
  This is a real, if small, change to two structs other subsystems already
  depend on (`journey_bridge.rs`, `civ_tools_bridge.rs`, `render.rs`) —
  verify nothing downstream assumes settlement/way vectors never carry an
  extra field, and add it additively.
- **Decide and record** (don't silently choose) whether `_civApplyRecovery`
  ports alongside the tier tables here or stays out of scope for
  `PHASE2_SCOPE.md` to pick up separately, and whether the `Metropolis`
  tier gap (§3 point 2) is resolved by adding the variant, capping at
  `Capital`, or deferring metropolis-tier settlements' demotion/growth
  behavior until `_civSelectMetropolises` itself lands. Log the decision in
  this port's `CHANGELOG.md` per the porting-discipline skill.

### Milestone 2 — proximity graph + betweenness centrality

`_civProximityAdjacency` (24672-24683) + `_civBetweennessFromAdjacency`
(24687-24709). Fully self-contained (places array + `cellKm` in, adjacency/
betweenness out), no dependency on milestone 1, genuinely new Rust (no
existing Brandes implementation to lean on per §3 point 4). Golden-testable
directly against small hand-checkable graphs plus a real settlement-array
fixture pulled from the reference.

### Milestone 3 — the collapse and recovery step functions

`_civSettlementStress`, `_civMortalityMigrationRates`, `_civGravityMigrate`,
`_civCollapseStep`, `_civRecoveryGrowthStep` — the actual mechanistic model,
depends on milestones 1 and 2. This is the core of the subsystem and the
highest-value golden-parity target: fully deterministic (no RNG), so a
single fixed places-array fixture run through both the reference (via the
project's Node harness convention, `PARITY_TESTING.md`) and the port should
match to tight tolerance every field, not just population — `died`/
`migrated`/`unplaced`/`failed` stats, tier changes, `ruins`/`fortified`
flag assignment. Per the porting-discipline skill, shape fixtures to reach
real branches: a settlement right at an abandonment-floor boundary, a
fortified vs. unfortified pair at equal stress, all four collapse
characters, both collapse and recovery modes.

### Milestone 4 — snapshot data model + orchestrator

`_civSimulateTimeline` (pure orchestrator), plus the Rust equivalent of
`civTimeline`/`civYear`/`civSnapshotSave`/`civSnapshotLoad`/`civGotoYear`/
`civAddYear`/`civRemoveYear`/`_civYearDiff`/`_civAssignTid`/
`_civResyncNextTid`. This is where `CivData` (or a new sibling struct it
owns) gains real timeline state: a `Vec` of year snapshots, each capturing
territory + settlements + ways, plus the active year cursor. Decide the
snapshot's exact shape against Rust's real types (not JS's loosely-typed
`{...p}` spread) — likely a dedicated `TimelineSnapshot { year, territory,
settlements, ways }` struct rather than reusing `CivData` wholesale (a
snapshot doesn't need `provinces`/`trade_balances`/`explanations`, which the
reference's own snapshot never captured either — it only ever stored
`territory`/`places`/`ways`, confirmed at `civSnapshotSave` line 20596-20604).
Depends on milestone 1 for the stable-id field the diff logic needs.

### Milestone 5 — the Godot boundary

`_civRunCollapseSimulation`'s impure wiring, translated to this port's own
established boundary pattern: a new `godot`-free `timeline_bridge.rs` module
(same isolation as `journey_bridge.rs`/`civ_tools_bridge.rs`/
`infra_tools_bridge.rs`/`sculpt_bridge.rs`), exposing something like
`civ_add_year`/`civ_goto_year`/`civ_remove_year`/`civ_run_collapse_simulation`/
`civ_year_diff` as plain-Rust functions over `CivData`'s new timeline state,
with `lib.rs` owning the thin `#[func]`/`Variant` conversion layer exactly as
`journey_bridge.rs`'s step 4 did for the Journey Planner. Plain-Rust tests
(no Godot runtime) for the request/response shapes, following that
milestone's own precedent rather than inventing a new boundary style.

### Milestone 6 — UI playback controls

`_civTlStartPlay`/`_civTlStopPlay`/`_civWireYearSlider`/
`_civBuildExploreTimelineUI` and the markup at lines 1888-1952, reimagined
against `DCC_SHELL_SPEC.md`/`DCC_CONTROL_INDEX.md` §10's own Timeline region
rather than ported as literal HTML/JS. Read §4 above before starting — this
closes part of §10's gap list (scrub track, play/pause/step) but explicitly
**not** the six-toggle continuous-simulation feature or Warfare, which stay
the owner's open decision. Per this project's own standing practice, this
is a dedicated UI/UX pass (ui-ux-pro-max), not raw sliders bolted onto
`right_dock.gd`.

## 6. Out of scope for all milestones above

- `_civSelectMetropolises`/the metropolis promotion pass (reference line
  24961) — a pre-existing, separately-scoped gap, referenced only where §3
  point 2/milestone 1 need a decision about its absence.
- `_civApplyRecovery`/auto-populate's static "Recovery phase" dropdown —
  adjacent, see §3 point 5. Its own scoping (if any) belongs to
  `PHASE2_SCOPE.md`, not here, unless milestone 1 explicitly chooses to
  bundle the port.
- The DCC shell's six-toggle continuous simulation-layer feature and
  Warfare (`DCC_CONTROL_INDEX.md` §10/§5) — a distinct, larger, still-open
  product question. See §4.
- `collapse-timeline-dynamics.md` §8's own deferred items (new-settlement
  founding from diaspora, true travel-cost migration distance, regrowth-
  phase migration) — the reference itself doesn't build these; this port
  matches the reference, not a superset of it.
- Save-format persistence of `civTimeline`/`civYear` into this port's own
  `.zip`/save format (`cartalith-io`, `SAVEFILE_COMPAT.md`) — real (§1
  Cluster D), but a separate decision with its own format design; noted in
  Open Questions rather than folded into a milestone above.

## 7. Success criteria

1. A fixed places-array fixture, run through the reference's
   `_civSimulateTimeline` (collapse mode, all four characters; recovery
   mode) via the project's Node harness, matches this port's output to
   golden-parity tolerance on every field the reference produces — not just
   population, but `died`/`migrated`/`unplaced`/`failed` stats and tier/
   `ruins`/`fortified` transitions.
2. `civ_add_year`/`civ_goto_year`/`civ_remove_year` reproduce the
   reference's own snapshot semantics: adding a year never loses the
   currently-active year's state, `civGotoYear` never mutates
   `state.places`/`civWays` (only territory), and removing the active year
   falls back to the earliest remaining one.
3. `civ_year_diff`'s `present`/`removed`/`added` sets match the reference's
   `tid`-based diff for a multi-year fixture, including a settlement that
   disappears in one year and a same-name settlement that's actually a
   *different* object (tid must disambiguate this, not name/position
   matching).
4. The Godot boundary (`timeline_bridge.rs`) round-trips a full simulate-
   then-scrub sequence through `#[func]` calls with no panic on malformed
   input (per `cartalith-rust-conventions`'s gdext-boundary-panic rule).
5. A playable end-to-end path exists in the actual Godot shell: add a year
   by hand, run a collapse simulation, scrub the resulting timeline with
   visible ghost/highlight/exist-only filtering — confirmed in the editor,
   with the same "cannot confirm real on-device performance from this
   session" caveat this project's other UI milestones already carry.
6. The metropolis-tier and `_civApplyRecovery` decisions from milestone 1
   are recorded in `CHANGELOG.md`, not silently made.

## 8. Open questions

- **Metropolis tier** (§3 point 2): add `SettlementKind::Metropolis`, cap
  at `Capital`, or defer? Affects milestone 1 and every tier-transition
  test after it.
- **`_civApplyRecovery` bundling** (§3 point 5): port alongside the shared
  tier tables in milestone 1, or leave for a `PHASE2_SCOPE.md` addendum?
- **Save-format persistence**: should `civTimeline`/`civYear` be added to
  this port's save format at all, now or later? The reference does persist
  them; `SAVEFILE_COMPAT.md` doesn't document that they exist in the
  reference's format, which is itself worth a correction pass to that
  document independent of this decision.
- **Snapshot memory/size**: the reference stores a full deep-copied
  `places`/`ways`/territory array per recorded year with no eviction. A
  long simulation (`civSimDuration` ÷ `civSimStepYears`, both user-set, no
  hard cap in the reference) could produce many snapshots. Worth deciding
  whether this port matches that unbounded behavior or adds a cap — a
  deliberate deviation to log if chosen, not a silent one.
- **DCC shell coordination** (§4): who confirms with the owner, before
  milestone 6 starts, that "Timeline" in this scope is understood as
  distinct from `DCC_CONTROL_INDEX.md` §5's still-open six-toggle/Warfare
  question?

## 9. Decisions (2026-08-19, made in-flight to keep the build moving)

Owner approved the subsystem and said to move forward without stopping for
per-question sign-off. Recorded here rather than made silently, per this
port's own discipline:

- **Metropolis tier**: cap the ported `_CIV_TIER_ORDER`/`_CIV_TIER_FLOOR`
  table at `Capital`. `SettlementKind::Metropolis` is not added in this
  pass — metropolis-tier demotion/promotion stays unreachable until
  `_civSelectMetropolises` itself is ported (tracked where it already was).
  This keeps milestone 1 from inventing behavior for a variant nothing else
  produces yet.
- **`_civApplyRecovery`**: out of scope here. Left for a future
  `PHASE2_SCOPE.md` addendum (auto-populate's own feature), not bundled
  into milestone 1.
- **Save-format persistence**: deferred. `civTimeline`/`civYear` are not
  added to `cartalith-io`'s save format in this pass. `SAVEFILE_COMPAT.md`
  gets a note recording the gap (the reference does persist these) so the
  omission is disclosed, not silently missing.
- **Snapshot cap**: deliberate deviation from the reference's unbounded
  storage — cap recorded timeline years at a generous ceiling (e.g. 2000)
  to bound memory, logged as a chosen deviation rather than matched
  unbounded behavior, consistent with `MEMORY_OPTIMIZATION_SCOPE.md`'s
  existing budget discipline.
- **DCC shell coordination**: handled by scoping milestone 6 explicitly to
  the discrete scrub/playback mechanism only (scrub track, play/pause/step,
  ghost/highlight/exist-only filters) — never the six-toggle continuous-
  simulation feature or Warfare, which remain untouched and still open.
  This note *is* the confirmation trail.

Milestones proceed in dependency order: 1, 2 (parallel-safe with 1 only if
isolated — dispatched sequentially in practice given both touch
`cartalith-civ/src/lib.rs`), then 3, 4, 5, 6 in sequence.

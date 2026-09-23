# TIMELINE_SCOPE.md — the timeline and collapse/recovery simulation

**What this is:** the definition of the Timeline subsystem's six milestones
(§5) and success criteria (§7), the reference function list it ports,
re-verified line by line (§1), the design intent behind it (§2), and the
decisions that shaped it (§9). **What it is not:** status. Where each milestone
stands is `cartalith-native/docs/STATUS.md`'s "Timeline" group (TL-1…TL-6); open
work is routed from `OUTSTANDING_WORK.md`.

`FUNCTIONAL_CONTRACT.md` §4 found this layer present in the reference and
untracked by the port; the owner approved it for build on 2026-08-19. The
function list below was re-verified against `reference/Cartalith Gen1 v2.10.html`
(every line number here resolves there) rather than trusted from that
contract's high-level pass. The design grounding is two research documents in
the **source project's** `docs/` (naming hazard 1 in `CLAUDE.md`):
`docs/research/collapse-timeline-dynamics.md` and
`docs/research/settlement-emergence.md`, whose one relevant section is §5
(there is no §6 on this subject).

Story planning (`STORY_PLANNING_SCOPE.md`) is built over this subsystem's year
cursor.

## 1. Verified reference function list

`FUNCTIONAL_CONTRACT.md`'s range (~20597-26478) and its function list
(`civAddYear`/`civGotoYear`, `_civSimulateTimeline`/`_civCollapseStep`/
`_civRecoveryGrowthStep`/`_civRunCollapseSimulation`) hold only **as a loose
envelope**:

1. **The range is not contiguous Timeline code.** Most of its ~5 880 lines are
   civ machinery ported or tracked elsewhere (pins, the faction inspector,
   snapping, routing, population and trade, provinces, village seeding,
   editors, the route-planner UI). Real Timeline code is four disjoint clusters
   totalling well under 500 lines. Slicing "lines 20597-26478" would pull in
   code other scope documents already claim.
2. **The list under-names two load-bearing pieces**: the snapshot read/write
   (`civSnapshotSave`/`civSnapshotLoad`/`civRemoveYear`) and the playback/UI
   wiring cluster (`_civTlStartPlay`/`_civTlStopPlay`/`_civWireYearSlider`/
   `_civBuildExploreTimelineUI`).

### Cluster A — manual timeline authoring + snapshot storage (lines 20563-20662)

| Function / symbol | Lines | What it does |
|---|---|---|
| `let civTimeline=[]`, `let civYear=0`, `let civWays=[]` | 14756-14758 | Global state. `civWays` is `CivData::ways` in this port; `civTimeline`/`civYear` are this subsystem's. |
| `_civAssignTid` | 20564 | Lazily stamps a stable, monotonic `tid` onto a place/way the first time it is touched. |
| `_civResyncNextTid` | 20565-20574 | Rescans `state.places`/`civWays`/every `civTimeline` entry for the max `tid`, so a loaded save's next id never collides with historical ones. |
| `_civYearDiffInvalidate` / cache vars | 20575-20576 | Invalidates the memoized year-diff. |
| `_civYearDiff` | 20580-20595 | Diffs the active year's snapshot against the chronologically previous one by `tid` set → `{present, removed, added}`. Powers the "exist only" filter and the ghost/highlight overlays. Cached per `civYear`. |
| `civSnapshotSave` | 20596-20606 | Captures the *live* `getCivTerritory()` + `state.places` + `civWays` into (or over) the entry for a year, sparse-encoding territory as `[i, factionId, ...]` pairs. Assigns `tid`s as a side effect. |
| `civSnapshotLoad` | 20607-20614 | Restores territory paint from a year's snapshot. **Never touches `state.places`/`civWays`** — those stay the single always-current, always-editable arrays every other system reads. |
| `civGotoYear` | 20615-20617 | `civYear = year`, load its snapshot, rebuild the timeline UI. |
| `civAddYear` | 20618-20634 | Snapshot the *current* year (so it is not lost), then create an entry for `year` carrying territory/places/ways forward from the nearest earlier entry (or the live state, if the timeline was empty), and jump to it. |
| `civRemoveYear` | 20635-20641 | Deletes an entry; if it was the active year, jumps to the earliest remaining one (or year 0). |
| `_civFormatYear` | 20644 | `-1200 → "1200 BC"` / `450 → "450 AD"`. |
| `_civBuildTimelineUI` | 20645-20662 | Renders the pill list (one per recorded year); delegates the slider/playback row to Cluster C. |

### Cluster B — mechanistic collapse/recovery simulator (v0.85, lines 24608-24950)

Everything here is deliberately **pure and deterministic** — "the model consumes
no randomness — same inputs always replay the same history" (the reference's own
comment at `_civSimulateTimeline`). For golden parity that means no
RNG-stream-alignment risk and no iteration-order sensitivity beyond plain array
order.

| Function / symbol | Lines | What it does |
|---|---|---|
| `_civUpdatePopReadout` | 24596-24606 | UI-only readout string; not part of the timeline data model. |
| `_CIV_RECOVERY_FRAC` / `_CIV_RECOVERY_NAME` / `_CIV_TIER_ORDER` / `_CIV_TIER_FLOOR` / `_civTierForPopulation` | 24614-24618 | Shared tier-floor table (`hamlet`→`metropolis`) and population→tier lookup, used by **both** the v0.82 static recovery pass and the v0.85 stepper. |
| `_civApplyRecovery` | 24619-24640 | **v0.82, static/instant** re-weighting — not the year-stepped mechanism (§3 point 5). |
| Character weights and rate ceilings (`_CIV_COLLAPSE_CHAR_WEIGHTS`, `_CIV_COLLAPSE_MAX_MORTALITY`, `_CIV_COLLAPSE_MAX_MIGRATION`, `_CIV_COLLAPSE_MIGRATION_BIAS`, `_CIV_MIGRATE_BETA`, `_CIV_ABANDON_FLOOR`, `_CIV_FORTIFIED_BONUS`) | 24653-24666 | Every tuned constant `collapse-timeline-dynamics.md` §3-5 derives or cites. |
| `_civProximityAdjacency` | 24672-24683 | World-wrap-aware symmetric k-nearest-neighbour graph among settlements, in real km — the stepper's **own** network, deliberately decoupled from the rendered `ways` (whose indices go stale as settlements are removed step to step). |
| `_civBetweennessFromAdjacency` | 24687-24709 | Brandes (2001) betweenness over a prebuilt adjacency list — a **second, standalone** implementation, not a call into `_civNetworkMetrics`. |
| `_civSettlementStress` | 24713-24723 | Per-settlement stress in [0,1]: trade-dependency loss `L` (against a `baselineNormB` captured at simulation start), density/connectivity exposure `D`, undefended-violence exposure `V`, blended by the collapse **character**'s weight triple. |
| `_civMortalityMigrationRates` | 24726-24731 | stress × severity × character → this step's annual excess mortality `m` and out-migration `g`. |
| `_civGravityMigrate` | 24738-24778 | Zipf/Ravenstein gravity redistribution of each origin's migrants, weighted by `headroom × fortifiedBonus / distance^β`, in up to 4 saturation-aware passes; returns per-destination `received` plus system-wide `unplaced` (diaspora loss). |
| `_civCollapseStep` | 24785-24848 | One `stepYears` collapse step: rebuilds the proximity graph + betweenness, computes stress/mortality/migration, redistributes migrants, re-derives tiers (demoting, marking `ruins` where a nucleus falls below its floor), drops anything under the abandonment floor. Returns the new places + `{died, migrated, unplaced, failed}` + `normBByTid` (each step's stress baseline). |
| `_civRecoveryGrowthStep` | 24852-24870 | One `stepYears` logistic (Verhulst) regrowth step toward each settlement's catchment ceiling; re-derives tiers upward, clearing `ruins` on promotion back into an exchange tier. |
| `_civSimulateTimeline` | 24875-24892 | Pure orchestrator: runs `steps` collapse-or-recovery steps from a starting places array, one `{places, stats}` per step. Never touches `civTimeline`/`state.places`. |
| `_civRunCollapseSimulation` | 24896-24950 | **Impure wiring**: reads the sim-panel fields + `state.places`, calls `_civSimulateTimeline`, writes one `civTimeline` entry per step (anchoring a "before" frame at the start year, carrying territory/ways forward unchanged — collapse does not redraw borders). Warns before overwriting existing years. |

### Cluster C — scrub/playback UI wiring (lines 26424-26493)

| Function / symbol | Lines | What it does |
|---|---|---|
| `_civTlStopPlay` | 26425-26428 | Clears the playback timer, resets the Play label. |
| `_civTlStartPlay` | 26429-26440 | Advances `civYear` to the next recorded year every 1200 ms, stopping at the end. |
| `_civTlDragSrc` guard + `_civWireYearSlider` | 26451-26474 | One real-time-scale slider (`min`/`max`/`value` are years, not a snapshot index — v0.91) with a `<datalist>` for proportional ticks; dragging snaps to the nearest recorded year. |
| `_civBuildExploreTimelineUI` | 26478-26493 | The Explore→Timeline section: slider row (shown with ≥2 recorded years), Play, and the three filters (`timelineExistOnly`/`timelineGhost`/`timelineHighlight` on `state.mapFilter`). |

Markup at lines 1888-1952 (`#explTimelineSection`) is the literal control
inventory: `civTlYear`, `civTlAddYearBtn`, `civTimelinePanel`,
`explTimelineSlider`, `explTlPlayBtn`, `explTlExistOnly`/`Ghost`/`Highlight`,
`civSimMode`/`civSimCharacter`/`civSimSeverity`/`civSimRate`/
`civSimStartYear`/`civSimDuration`/`civSimStepYears`/`civSimulateBtn`/`civSimOut`.

### Cluster D — save-format persistence (not named by `FUNCTIONAL_CONTRACT.md`)

`civTimeline`/`civYear` **are** part of the reference's save format:
`_civSyncToState` (lines 26115-26139) serializes `state.civ.timeline`/
`state.civ.year`; `_civSyncFromState` (26140-26159+) restores them and calls
`_civResyncNextTid()` so ids created after a load never collide with historical
ones. This port's own archive carries them as `history/timeline.json` plus
`history/territory/<year>.i32` (`SAVEFILE_COMPAT.md` §10.1-10.2), and its
reference-format mapping table names `state.civ.timeline`/`state.civ.year` as
their source.

`generate()`'s wrapper (lines 26211-26224) clears `civTerritory`/`civTimeline`/
`civYear` on every fresh procedural generation (not on a loaded save). The port
does the same where it rebuilds `CivData` from scratch.

## 2. Design intent (from the research documents)

### `docs/research/collapse-timeline-dynamics.md`

The research and design foundation for the v0.85 stepper, building on
`settlement-emergence.md`'s v0.81/v0.82 work. Its argument, compressed:

- **The gap it closes.** The static v0.82 Recovery-phase pass answers "what would
  a Survival-era world look like", once. What was wanted is *process* — which
  settlements failed first and where the survivors went, year by year —
  written into the timeline so it can be scrubbed like real history.
- **Three variables govern any settlement-system collapse** (energy/food,
  transport/trade capacity, landscape carrying capacity), and the engine
  already computes analogues for all three (carrying capacity /
  `_civSettlementPopulation`, and the betweenness/closeness metrics that drive
  exchange-tier population).
- **Which settlements fail first is not one universal ranking.** Three
  archetypes, each the *opposite* ranking of another: trade collapse hits the
  biggest trade-dependent hubs hardest (Late Bronze Age collapse, Cline 2014;
  targeted-attack fragility, Albert/Jeong/Barabási 2000); disease hits dense,
  well-connected settlements hardest (Black Death, Benedictow 2004); conflict
  kills undefended settlements regardless of size while fortified ones persist
  (post-Roman West, Wickham 2005). A **Character** dial (trade / disease /
  conflict / mixed) weights three stress components (`wL`/`wD`/`wV`), so the
  failure order is a historically grounded choice per run, not a hard-coded
  assumption.
- **Stress → mortality/migration** through a **Severity** dial scaling two rate
  ceilings calibrated on the Black Death's annualized mortality (~13.6 %/yr from
  ~45 % over 4 years; ceiling 15 %/yr). Migration is a **gravity model** (Zipf
  1946 / Ravenstein 1885, distance-decay β = 1.5), with a fortified-settlement
  attractiveness bonus and unplaced migrants tracked as diaspora loss.
- **Tiers re-derive from the new population** through the same
  `_civTierForPopulation` the static pass uses — demotion into ruins,
  abandonment below a floor. That is what makes settlements appear and
  disappear on the timeline, "which the existing tid-diff
  ghost/highlight/exist-only overlay already visualises with no new rendering
  code."
- **§7, "wiring into the existing timeline — no new data model needed"**: the
  simulator runs the per-step maths N times and pushes one snapshot per step in
  the *exact* shape `civAddYear` produces, so the existing slider and overlays
  work unmodified on simulated history. It preserves the invariant that jumping
  to a recorded year overwrites *territory* but never the live places/ways.
- **§8, deferred by the research itself**: founding new settlements from
  displaced populations, true travel-cost migration distance (versus the
  Euclidean approximation), and regrowth-phase migration back toward reviving
  hubs. The reference does not build these either (§6).

### `docs/research/settlement-emergence.md` §5, "Post-collapse recovery model"

The earlier (v0.82) design the v0.85 stepper builds on:

- Collapse does not reset the landscape — roads, ruins, fields, wells, bridges,
  mines, irrigation persist, but infrastructure decays faster than ecological
  knowledge, so a recovery population inherits *excess buildings, insufficient
  labour, damaged ecosystems*. Recovery runs **below** the ecological ceiling.
- Four phases (**I Survival** <10 % of former population, **II Subsistence**
  10-30 %, **III Regional** 30-70 %, **IV Mature** 70 %+), each with its own
  settlement logic (cluster on water and ruins → repopulate abandoned villages
  → crafts, markets, roads and politics return → the prior economic geography
  mostly returns, permanently losing some sites).
- **Recovery does not delete settlements — it re-scores them**:
  `SettlementValue = Infrastructure + AgriculturalPotential + WaterAccess +
  StrategicPosition − MaintenanceCost`. A ruined city becomes a small fortified
  settlement inside its own ruins — the "demote, don't delete, mark
  ruins + fortified" pattern `_civApplyRecovery` and `_civCollapseStep` both
  implement.

## 3. Where the state lives, and the five prerequisites

`cartalith-civ` is stateless pure functions (`ARCHITECTURE.md`); the mutable civ
state is `cartalith-godot`'s `WorldGen::civ: Option<CivData>`, rebuilt by
`compute_civilisation` after a generate. The timeline lives there too:
`CivData::timeline: Vec<TimelineSnapshot>`, the `year` cursor and the `next_tid`
counter. `cartalith_civ::timeline` holds everything that can be a pure function
over them.

The stepper could not run without five things the port did not have when this
was scoped. Each is a design requirement; the symbol that meets it is named.

1. **A stable per-object id.** `_civYearDiff` must tell "the same settlement,
   renamed or moved" from "a different settlement" across years. Met by a
   `tid: u64` on `NamedSettlement` and `Way`, assigned once
   (`civ_assign_tid`, `0` = unassigned, real ids from 1) and reseeded after any
   reload by `civ_resync_next_tid` — never reconstructed from position or name.
2. **A metropolis tier.** The reference's `_CIV_TIER_ORDER`/`_CIV_TIER_FLOOR`
   have six tiers, metropolis highest. `SettlementKind::Metropolis` exists and
   both tables carry the reference's six entries (§9 records how this was first
   capped, then corrected).
3. **The population-ceiling chain.** The collapse stepper's migration headroom
   and the recovery stepper's logistic ceiling both call
   `_civSettlementPopulation(place, K, opts)` (reference 23502-23511), which
   calls `_civCatchmentPop` (23484-23500) → `_civCatchmentDensityMean`
   (23461-23469) and reads `currentAgrarianDensity()` and the
   `_CIV_SURPLUS_FRACTION`/`_CIV_TRADE_K` tables. Met in `timeline.rs`
   (`civ_settlement_population`, `civ_current_agrarian_density`, …), reusing
   the already golden-verified `build_carrying_capacity` and
   `civ_catchment_km2`/`civ_catchment_radius_cells` rather than rediscovering
   carrying capacity.
4. **Betweenness centrality.** Neither `_civNetworkMetrics` nor any Brandes
   implementation existed in Rust. `_civProximityAdjacency`/
   `_civBetweennessFromAdjacency` are self-contained (a places array and
   `cellKm` in, no dependency on ways or routing), so they are their own
   milestone: `civ_proximity_adjacency`, `civ_betweenness_from_adjacency`.
5. **`_civApplyRecovery` (v0.82, static) is adjacent, not this subsystem.** It
   is called from `_civIterativeAutoWorld` (call site, reference line 25761) as
   auto-populate's "Recovery phase" dropdown (markup 1424-1425) — an instant
   re-weighting of a freshly generated world, separate from the year-stepped
   timeline. It shares the tier tables with the stepper, which is why the
   reference comments them together. It is `civ_apply_recovery` in
   `timeline.rs`, beside those tables (§9).

## 4. The shell's timeline strip: a second view of the same cursor

The discrete mechanism this document scopes — a handful of authored or simulated
*years*, each a full state snapshot, scrubbed between, never interpolated and
never continuously re-simulated — is the only timeline the reference has.
`_civRunCollapseSimulation` produces a bounded number of discrete steps and
writes them once; there is no Warfare domain in the reference's civ model at
all.

The shell draws that one cursor (`CivData::year`) in two places, and neither
keeps a year of its own (`dcc_shell.gd` §10a):

- **The CIVIL left dock's Timeline category** (`civilization_workspace.gd::
  _build_timeline`): recorded-year pills, Add year, scrub, Play/Pause/Step, the
  three filters over a live `civ_year_diff()` count, and the collapse/recovery
  form as an expander. Ruling L put the two v3 halves (Politics and Simulation)
  back into this one category.
- **The shell's timeline strip** (`timeline_bar`, shown for CIVIL): transport,
  ×1/×10/×100 speeds, a scrub track fixed at −400…1200 by the design canvas,
  six simulation-layer toggles, and SP-2's day row. Built to
  `design/dcc-environment-2026-08-31/spec/01-frame-and-tokens.md` §3.7 and
  `05-right-dock-and-bars.md` §4.2 (the phone's sim strip is `06-phone.md`
  §6.2).

**The six layer toggles (Climate · Population · Economy · Politics ·
Infrastructure · Warfare) are drawn and deliberately inert** — the owner's
design answer (`design/dcc-environment-2026-08-31/BUILD_ANSWERS.md` §3) calls
them intended and fixes the note they carry: *"they record which layer you
want; no layer renders yet"*. Nothing in this document authorises a continuous,
per-domain simulation behind them (§6).

**The standing instruction, cited by several documents:** design the panel
before guessing its region. Moving the scrubber to program scope (v3's "time is
not a domain") is a shell-frame change, `GUI_GAP_REGISTER.md` CV-24, and the
owner ruled on 2026-09-03 that it waits for a design pass
(`LARGE_ITEM_RULINGS.md`).

**The filters, as the design requires them.** *Exist only* filters the
settlements handed to the map to the active year's `civ_year_diff().present`
`tid`s. *Ghost removed* and *Highlight new* need per-pin fade/halo drawing in
`map_overlay.gd`, and *Ghost removed* additionally needs the removed
settlements' old positions and names, which `civ_year_diff()` (tid sets only)
does not return.

## 5. Milestones

Dependencies: 3 needs 1 and 2; 4 needs 1; 5 needs 4; 6 needs 5.

### Milestone 1 — shared prerequisites: population-ceiling chain + stable ids

- Port `_civSettlementPopulation`'s chain: `_civCatchmentDensityMean`,
  `_civCatchmentPop`, a `currentAgrarianDensity` equivalent, the
  `_CIV_SURPLUS_FRACTION`/`_CIV_TRADE_K` tables, and `_civSettlementPopulation`
  itself. Golden-testable in isolation (pure, no RNG), with fixtures extracted
  the way `golden_parity_carrying_capacity.rs` did for `K`.
- Port the shared tier tables (`_CIV_RECOVERY_FRAC`/`_CIV_RECOVERY_NAME`/
  `_CIV_TIER_ORDER`/`_CIV_TIER_FLOOR`/`_civTierForPopulation`) as their own
  small module, usable by both the stepper and `_civApplyRecovery`.
- Add the stable id to `NamedSettlement`/`Way` (§3 point 1). Other subsystems
  construct both structs (`journey_bridge.rs`, `civ_tools_bridge.rs`,
  `render.rs`, …), so add it additively.
- **Decide and record, don't silently choose**, the metropolis-tier and
  `_civApplyRecovery` questions (§9).

### Milestone 2 — proximity graph + betweenness centrality

`_civProximityAdjacency` (24672-24683) + `_civBetweennessFromAdjacency`
(24687-24709). Self-contained (places + `cellKm` in, adjacency/betweenness out),
independent of milestone 1, genuinely new Rust. Golden-testable against small
hand-checkable graphs plus a real settlement fixture from the reference.

### Milestone 3 — the collapse and recovery step functions

`_civSettlementStress`, `_civMortalityMigrationRates`, `_civGravityMigrate`,
`_civCollapseStep`, `_civRecoveryGrowthStep` — the mechanistic model itself.
The highest-value golden target: fully deterministic, so one fixed places
fixture run through the reference (the Node harness, `PARITY_TESTING.md`) and
the port must match every field, not just population — `died`/`migrated`/
`unplaced`/`failed`, tier changes, `ruins`/`fortified`. Shape fixtures to reach
real branches: a settlement right at the abandonment floor, a fortified and an
unfortified settlement at equal stress, all four characters, both modes.

### Milestone 4 — snapshot data model + orchestrator

`_civSimulateTimeline` plus the Rust equivalent of `civTimeline`/`civYear`/
`civSnapshotSave`/`civSnapshotLoad`/`civGotoYear`/`civAddYear`/`civRemoveYear`/
`_civYearDiff`/`_civAssignTid`/`_civResyncNextTid`. A dedicated
`TimelineSnapshot { year, territory, settlements, ways }` rather than a copy of
`CivData`: the reference's snapshot never captured provinces, trade balances or
explanations (`civSnapshotSave`, 20596-20604). Territory is stored as mutations,
not whole rasters (§9, ruling 27).

### Milestone 5 — the Godot boundary

`_civRunCollapseSimulation`'s impure wiring in the port's established boundary
pattern: a `godot`-free `timeline_bridge.rs` (the same isolation as
`journey_bridge.rs`/`civ_tools_bridge.rs`/`infra_tools_bridge.rs`/
`sculpt_bridge.rs`) owning the request parser and `run_collapse_simulation`,
with `lib.rs` owning the thin `#[func]` layer: `civ_add_year`, `civ_goto_year`,
`civ_remove_year`, `civ_year_diff`, `civ_run_collapse_simulation`. Plain-Rust
tests for the request/response shapes, no Godot runtime.

### Milestone 6 — UI playback controls

`_civTlStartPlay`/`_civTlStopPlay`/`_civWireYearSlider`/
`_civBuildExploreTimelineUI` and the markup at 1888-1952, reimagined against the
DCC shell rather than ported as literal HTML — a dedicated UI pass, not raw
sliders. Read §4 first.

Built as a `DccWidgets.category()` in the CIVIL left dock, **not** a right-dock
context: `CTX_SCULPT`/`CTX_JOURNEY` were driven by a map tool arming, and the
timeline has no map click of its own. Deliverables: years pill row + Add year, a
real-time-scale scrub, Play/Pause on a real 1200 ms timer plus a **Step** button
(a port addition; the reference has none), the three filters over a live
`civ_year_diff()` count, and the collapse/recovery form with a real confirmation
before overwriting recorded years.

## 6. Out of scope for all milestones above

- **A continuous, per-domain simulation behind the shell's six layer toggles,
  and Warfare** (`DCC_CONTROL_INDEX.md` §10 and its summary §5). See §4: the
  toggles are drawn and inert by owner design answer; the reference has no such
  simulation.
- **`collapse-timeline-dynamics.md` §8's deferred items** (founding from
  diaspora, travel-cost migration distance, regrowth-phase migration). The
  reference does not build them; this port matches the reference, not a
  superset of it.
- **`_civSelectMetropolises` and `_civApplyRecovery`** were listed here as
  separately scoped. The owner had both ported on 2026-08-20, outside these
  milestones (`PHASE2_SCOPE.md` milestone 21,
  `tests/golden_parity_metropolis_recovery.rs`). §9's metropolis decision was
  conditional on the first one's absence.

## 7. Success criteria

1. A fixed places fixture, run through the reference's `_civSimulateTimeline`
   (collapse mode with all four characters; recovery mode) via the Node
   harness, matches the port to golden-parity tolerance on every field the
   reference produces — `died`/`migrated`/`unplaced`/`failed` and the
   tier/`ruins`/`fortified` transitions, not just population.
2. `civ_add_year`/`civ_goto_year`/`civ_remove_year` reproduce the reference's
   snapshot semantics: adding a year never loses the active year's state,
   `civGotoYear` never mutates the live settlements/ways (only territory), and
   removing the active year falls back to the earliest remaining one.
3. `civ_year_diff`'s `present`/`removed`/`added` match the reference's
   `tid`-based diff over a multi-year fixture, including a settlement that
   disappears in one year and a same-name settlement that is actually a
   *different* object (the `tid` must disambiguate, not name or position).
4. The Godot boundary (`timeline_bridge.rs`) round-trips a simulate-then-scrub
   sequence through `#[func]` calls with no panic on malformed input
   (`cartalith-rust-conventions`' gdext-boundary rule).
5. A playable end-to-end path exists in the Godot shell: add a year by hand,
   run a collapse simulation, scrub the result with visible
   ghost/highlight/exist-only filtering — confirmed in the editor, with the
   same "cannot confirm on-device performance from this session" caveat other
   UI milestones carry.
6. The metropolis-tier and `_civApplyRecovery` decisions are recorded (§9), not
   silently made.

## 8. Open questions

None. Each question this document raised — the metropolis tier,
`_civApplyRecovery` bundling, save-format persistence, a snapshot cap, and
shell coordination — is answered in §9.

## 9. Decisions

The owner approved the subsystem on 2026-08-19 and said to move forward without
per-question sign-off, so the first answers below were made in flight and
recorded; later rulings are dated where they changed one.

- **Metropolis tier — the reference's six tiers.** First capped at `Capital`
  (2026-08-19), explicitly until `_civSelectMetropolises` was ported, so as not
  to invent behaviour for a variant nothing produced. That condition fired on
  2026-08-20: `SettlementKind::Metropolis` exists and both tables carry the
  reference's six entries (`metropolis` first, floor 150 000). The two tests
  that had pinned the cap were **re-extracted from the reference, not
  hand-flipped**, and the thirteen boundary samples are re-derived in
  `golden_parity_metropolis_recovery.rs`.
- **`_civApplyRecovery` — ported, on the owner's decision (2026-08-20).** First
  left out of milestone 1. It is `timeline.rs::civ_apply_recovery`, wired at the
  reference's own call site behind `set_recovery_phase`, and surfaced as the
  **Recovery phase** dropdown in `File ▸ New world ▸ Generation`.
- **Save-format persistence — the timeline is saved.** First deferred
  (2026-08-19). The project archive carries it as `history/timeline.json` +
  `history/territory/<year>.i32` (`SAVEFILE_COMPAT.md` §10). The collapse run's
  per-year `ruins`/`fortified` side table (`TimelineSnapshot::collapse_flags`)
  is in memory only; a reloaded year reads it as *not recorded*, never as
  `false`.
- **Snapshot cap and storage — a deliberate deviation from the reference's
  unbounded, whole-copy storage.** At most `TIMELINE_MAX_YEARS` = 2000 recorded
  years (`cartalith-godot/src/lib.rs`; `civ_add_year` is a no-op past it, after
  safely snapshotting the active year), consistent with
  `MEMORY_OPTIMIZATION_SCOPE.md`'s budget discipline. **Ruling 27
  (2026-09-06, `LARGE_ITEM_RULINGS.md`): the timeline stores mutations, not
  snapshots** — *"if a position doesn't change for 50 years that's 50
  datapoints we do not need."* Territory is a `TerritoryFrame` of the cells
  whose owner changed since the previous recorded year, with a keyframe every
  `TERRITORY_KEYFRAME_INTERVAL` = 64 years to bound a scrub's replay;
  reconstruction must be exact against a full snapshot, and that is what is
  tested. `TimelineSnapshot`'s doc comment records the cost this removed
  (10.32 MiB per recorded year at 2 048 × 1 311, of which the raster was
  10.24 MiB). The on-disk shape (§10.2) did not change, so no format bump was
  owed.
- **Shell coordination — the discrete mechanism only.** Milestone 6 is scoped to
  the scrub/playback mechanism and its filters; the six layer toggles and
  Warfare are §4's and §6's. The owner's later calls are there too: the toggles
  are drawn and inert (`BUILD_ANSWERS.md` §3, 2026-08-31), and the
  program-scope scrubber waits for a design (CV-24, 2026-09-03).

# Landmark generation: what exists, what is new, and in what order

`LANDMARK_GENERATION_RESEARCH.md` is the owner-supplied framework — imported
verbatim, unedited, 2026-08-30. This document is the other half: it goes and
finds out, crate by crate, which of the research's assumed inputs were real in
this engine, separates what would be genuinely new work from what is
composition of things that already exist, and lays out an order to build in.
It is an **inventory and a plan**: it defines M1-M9 and poses §4's six owner
questions, all of which have since been ruled (§4 names each ruling). Every
claim in §1 was checked against the workspace on **2026-08-30**, the same
discipline `URBAN_MORPHOLOGY_SCOPE.md` and `MARKDOWN_VAULT_SCOPE.md` were held
to before any of their milestones started; its `file:line` citations were
replaced by symbol names on 2026-09-23, with each symbol re-located then.

> **This document defines the milestones; it does not track them.** Where each
> of M1-M9 stands is recorded only in `cartalith-native/docs/STATUS.md`; the
> owner's answers to §4's questions are rulings, recorded in
> `LARGE_ITEM_RULINGS.md` and summarised in §4. Building began the day after
> this document was written, so read §1's inventory as the *starting* state it
> was, and re-check anything load-bearing against the code rather than against
> this page.

The research's own framing is worth restating because it is the test every
milestone below is held to: the goal is not "where should a landmark be
placed" but "why does this landmark exist here" (§1). A milestone that scores
candidate cells without wiring in the *reason* has not met that bar, whatever
its suitability numbers say.

## 0. The headline finding

This is **not** a green field. The research's own §6 sentence — "Cartalith
already possesses or intends to possess: flow direction, flow accumulation,
river networks, drainage basins, lakes, coastlines, terrain gradients" — is
mostly true, and the parts that are true go further than the research assumes:
this port already has a golden-verified, real-world-measured **mountain-pass
corridor detector** (`DECISIONS.md` §7i) that is very close to what the
research's §8 asks for from scratch, a **settlement-gravity / population-
weighted cost-distance influence field** (`DECISIONS.md` §7b) that is a real
precedent for the research's §13 spatial-interaction model, and a **15-mineral
resource-potential system** with real geological grounding. What was
**completely absent** on 2026-08-30, confirmed by an empty grep across all
sixteen crates, was viewshed/visibility and any general-purpose Poisson-disc sampler — exactly
the two the task brief predicted would be missing, and the two the research
itself (§9, §16) leans on hardest for "why is this landmark *significant*"
rather than merely "why is it *here*."

## 1. Inventory: what the research assumes vs. what was real, 2026-08-30

Checked against the code, not against what a scope document or the research's
own prose claims. Where a name in the research maps to a *different* name in
this codebase, that mapping is called out — per the task brief, it is the
single most useful thing this section can carry, and it is the half of this
table that does not age.

**The Exists/Absent verdicts are this pass's own reading on 2026-08-30**, and
are what the milestone ordering below was derived from. Several were changed by
the build that followed. The mappings, the naming collisions and the
"exists under a different name" findings are the durable content; treat a bare
*Absent* as a starting condition, not a live fact.

| Research input | Verdict | Evidence |
|---|---|---|
| Flow direction (D8) | **Exists** | `cartalith_hydrology::compute_flow` (`cartalith-hydrology/src/lib.rs`) — doc comment names it explicitly as `computeFlow()`, D8 steepest-descent, reference HTML 4862-4890 |
| Flow accumulation | **Exists** | Same function; seeded by cell count or by rainfall discharge. GPU-parallel redesign (pointer-doubling subtree sum) also shipped, `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 9 |
| River networks / channels | **Exists** | `build_channels` (`cartalith-hydrology/src/lib.rs`) — returns a `ChannelResult` with channel mask, per-cell receiver, per-cell slope |
| Stream order (Strahler) | **Exists** | `strahler_from_receivers` (`cartalith-hydrology/src/lib.rs`) |
| River centrelines | **Exists** | `trace_river_polylines`/`split_river_polylines` (`cartalith-hydrology/src/lib.rs`) |
| Confluences | **Exists, implicitly** | Not extracted as a labelled list — but the receiver tree (`ChannelResult::recv`) encodes every confluence as any cell with more than one upstream channel neighbour, and polyline splitting is confluence-aware: `cartalith-hydrology/tests/golden_parity_polylines.rs`, `trace_river_polylines_case_0_confluence` |
| Lakes | **Exists** | `build_water_bodies` (`cartalith-civ/src/lib.rs`; ocean/lake/land classification); `apply_force_lake`, `civ_lake_flooded` (same file) |
| Drainage basins (as a discrete, labelled entity) | **Absent** | Zero hits for `watershed`, `drainage_basin`, `BasinId` anywhere in the workspace. A basin is derivable by walking `recv[]` to its outlet, but nothing labels or aggregates one as an object today — the same shape of gap `civ_continents` closed for landmasses (`MARKDOWN_VAULT_SCOPE.md` milestone 0) |
| Slope | **Exists, more than once** | `cartalith_civ::build_slope_field` (`cartalith-civ/src/lib.rs`, `slopeAt(x,y)*GW`); also recomputed inline as `sn` in `build_landform_field` (`cartalith-terrain/src/landform.rs`) and again per-pass inside `cartalith-erosion`. Three separate local recomputations, not one canonical reusable field |
| Aspect (compass direction of steepest descent) | **Absent** | No such field anywhere. Every `aspect` hit in the workspace is the unrelated "aspect ratio of the grid" sense |
| Curvature (2D terrain surface) | **Exists, inline only** | A real discrete Laplacian — `let curv = (l + rr + u + d - 4.0*hh) * w as f64; // Laplacian, resolution-scaled` — computed inside `build_landform_field` (`cartalith-terrain/src/landform.rs`, cirque branch) and used for exactly one threshold test. Not exposed as a standalone raster. (A *different*, unrelated `curvature` exists on `BoundaryPolyline` — 1D arc curvature for coastline/boundary polylines, `cartalith-terrain/src/lib.rs` — do not confuse the two) |
| Local relief / **TPI** | **Exists under a different name and for a different purpose** | `build_ao` (`cartalith-godot/src/render.rs`) computes `blur(field, r_broad) - field` and `blur(field, r_fine) - field` at two radii (`r_broad = gw·ao_radius_frac`, `r_fine = r_broad/3`) — this is **exactly** TPI(x) = z(x) − mean(neighbourhood), sign-flipped, at two of the research's §4 "multiple spatial scales." It is RMS-normalised and blended into a single-purpose 2D ambient-occlusion darkening multiplier, `pub(crate)` and private to the renderer — not returned as data, not reusable today. This is the single most useful "exists under another name" finding in this document. Separately, `build_relief_field` (`cartalith-terrain/src/infer.rs`) is a single-scale blurred-gradient-magnitude "boundary probability" proxy, closer to a ruggedness/edge-detector than to TPI |
| Ruggedness | **Absent** | No distinct implementation; would need to be derived (e.g. slope variance) |
| Prominence | **Exists, but only in 1D** | `RIDGE_PROMINENCE_M = 100.0` and a real prominence-filtered local-maxima algorithm (`cartalith-godot/src/measure_bridge.rs`) — but it runs only along a single **user-drawn cross-section** (the Measure tool), not as a 2D field over the grid. Still a real, owner-tuned precedent for what threshold this project already considers "prominent" |
| Peaks / ridges / saddles as a 2D layer | **Absent** | Only the 1D Measure-tool version above; nothing scans the whole grid for them |
| **Mountain-pass / corridor detection** | **Exists, and unusually well-verified** | `cartalith_civ::build_route_corridors` (reference line 5903) — takes the *minimum* of two flanking maxima along four axes at `gw/64` reach ("a corridor needs a barrier on BOTH sides of the axis"), golden-verified, and independently measured on a real 512×384 world (`DECISIONS.md` §7i): 30.8% of land carries any corridor value, only 1.02% is above half-strength — i.e. it is near-zero almost everywhere and spikes only at genuine pinch points, which is exactly the shape research §8's `S_pass` wants. It was explicitly chosen over a naive one-cell saddle test (`_civEnhancedTravelCost`) after that test was measured and found to fire on 0 of 4 real long crossings. Today it is consumed only as a route-cost relief multiplier (`civ_pass_relief`), never exposed as a landmark candidate field |
| **Least-cost path / accessibility** | **Exists extensively** | `DijkstraPath`/`civ_dijkstra_path` (`cartalith-civ/src/tools.rs`) — the Route/Way tools' multi-modal (land/water/mixed) Dijkstra over a cost grid built from terrain slope, biome friction, river navigability and existing-infrastructure discount. Separately, `WayRouter` (`cartalith-civ/src/trade.rs`) is a graph-level Dijkstra over settlements/ways used for trade-flow load accumulation. Research §12's `C(A,B)` is close to already built twice over, at two different granularities |
| **Spatial interaction / gravity** | **Partially exists, as a real precedent, not the formula itself** | `civ_apply_settlement_gravity` (`cartalith-civ/src/lib.rs`) discounts path cost near settlements weighted by size; `territory_influence` (same file) returns a per-cell population-weighted cost-distance owner/rival/influence/contested field — `DECISIONS.md` §7b's cost-distance-divided-by-a-monotonic-function-of-population design, which is conceptually the same family as research §13's `I(x) = ΣP_i / d_c(x,i)^β`, just not that exact formula and not exposed for landmark scoring |
| Settlements: position/population/faction | **Exists** | `get_settlements()` (`cartalith-godot/src/lib.rs`) — `x, y, name, population, kind, faction, capital, coastal, tid` (a stable id that survives a regenerate for kept settlements) |
| Roads | **Exists** | `get_roads()` (`cartalith-godot/src/lib.rs`) — generated network plus hand-drawn ways, `way_type`, `km`, `manual` flag |
| Sea routes | **Exists** | `get_sea_routes()` (`cartalith-godot/src/lib.rs`) |
| Resources / ore | **Exists, richly — 15 types** | `build_resource_potentials` (`cartalith-civ/src/lib.rs`) — copper, tin, iron, gold, salt, timber, lead, silver, clay, buildstone, flint, obsidian, gems, sulfur, alum, each geologically grounded (e.g. copper peaks at subduction-boundary cells) and passed through a scarcity cut |
| Soils | **Exists** | `build_soil_fertility` (`cartalith-civ/src/lib.rs`) — Jenny (1941) pedological model: climate bell × moisture × lithology-weatherability × slope-shedding × age |
| Lithology | **Exists** | `build_lithology` (`cartalith-civ/src/lib.rs`) — per-cell rock-type classification (basalt/sedimentary/etc.) |
| Geological resistance | **Exists, but naming collides with a different "resistance"** | `compute_resistance` (`cartalith-terrain/src/lib.rs`) is **tectonic/erosion** resistance (crustal type × age) — a different concept from `build_lithology`'s rock classification, despite the shared English word. Research §7's "R = geological resistance or lithological contrast" almost certainly means the lithology classification, not this function; flagged so a future implementer does not reach for the wrong one |
| Ecology / biome | **Exists** | `classify_biome`/`build_biome_raster` (`cartalith-civ/src/lib.rs`) |
| Political regions / provinces / factions | **Exists** | `Province` (`cartalith-civ/src/lib.rs`: id, faction, name, capital_settlement_index), `civ_generate_provinces`; `FactionEntry`/`FactionRoster` (`cartalith-godot/src/civ_roster_bridge.rs`); `assign_territory` (`cartalith-civ/src/lib.rs`, the cost-distance weighted Voronoi of `DECISIONS.md` §7b) |
| Historical state / timeline | **Exists, for the settlement half only** | `TimelineSnapshot`/`YearDiff` (`cartalith-civ/src/timeline.rs`), `timeline_bridge.rs`'s collapse/recovery simulation (`run_collapse_simulation`) — directly usable for research §20's "Settlement → Expansion → Conflict/decline → Abandonment → Ruination" chain, **except the "Conflict" link**, which needs the conflict/battle entity `STORY_PLANNING_SCOPE.md` SP-4 defines |
| Poisson-disc sampling | **Building blocks exist; the algorithm does not** | Two related but distinct mechanisms, neither of them Bridson (2007): (1) `icon_brush_stamp` (`cartalith-assets/src/manual.rs`) is genuine dart-throwing with a blue-noise rejection radius, but scoped to one manual brush stamp — local, user-tool-driven, capped at 1 500 darts — not a global field; (2) `find_settlement_seeds` (`cartalith-civ/src/lib.rs`) is greedy non-maximum suppression with an exclusion radius over ranked local-maxima candidates, which is the same *spirit* as research §15-16's "spatial competition / exclusion radius" but is rank-then-suppress, not dart-thrown, and is specific to settlement placement. No generic, reusable multi-class Poisson-disc sampler exists. `cartalith_spatial::QuadTree<T>` existed then and was named here as the acceleration structure; it was **retired 2026-09-22** (`5c99cc9`) without ever being used for this, and M6's exclusion queries use their own bucket grid (`landmark.rs::Buckets`) |
| **Viewshed / visibility** | **Confirmed absent** | Zero hits for `viewshed`, `line_of_sight`, `los(` in any of the sixteen crates. `cartalith-godot/src/render.rs`'s own module doc then listed "SVF/cast-shadow fields" among features it **deliberately excluded**, "depend[ing] on subsystems this port hasn't built yet." The nearest architectural relative was `build_ao`'s dual-radius blur-cavity math (see the TPI row above) — a statistical local-concavity estimate, not a geometric line-of-sight test, and it cannot answer "is B visible from A" without new code. *(Both relatives have moved since: `render.rs` now builds sky-view factor and cast shadow (`build_svf`, `build_sun_shadow`), and M7's line-of-sight primitive is `cartalith_terrain::analysis::visibility`.)* |

### Two findings outside the requested checklist, worth carrying forward

- **The owner's own vault-template corpus already has a landmark vocabulary.**
  `design/vault-templates/Landmark template.md` (two copies — one at the
  vault root, one under `Region Template/Landmarks/`) predates this research
  document and asks an author for `Type` (Temple / Ruin / Natural Wonder /
  Battle Site / etc.), physical description, cultural significance (myths,
  ritual/forbidden status) and history (built/discovered, who controlled it,
  damage/restoration) — independently converging on almost the same shape as
  the research's own §22 object model (`physical_basis`,
  `cultural_associations`, `historical_state`). `cartalith_vault::EntityKind`
  (`MARKDOWN_VAULT_SCOPE.md` §1) had no `Landmark` variant then, so the
  template was unconnected to any engine entity — see §4 question 2.
- **The owner's own UI vocabulary already names the concept.** `design/
  Cartalith Menu Structure v3.dc.html` (`GUI_GAP_REGISTER.md`'s v3 menu-audit
  table) lists **"Assets & landmarks"** as a CARTO submenu category and
  **"Points of interest"** as a CIVIL category — both unbacked menu labels on
  2026-08-30, the same shape of gap `GUI_GAP_REGISTER.md` catalogues everywhere
  else. Neither implies scope by itself; both are evidence the destination
  shell already has a place for this to land.

## 2. The Category A / B / C rule (§31), carried forward as binding

The research's §31 requires that Category A (established geographic
computation), Category B (empirically-inspired modelling) and Category C
(Cartalith-specific synthesis) stay **explicit in both documentation and
source code**. This document adopts that as a hard rule for every milestone
below, matching how this repository already tags `DECISIONS.md` §7a/§7d
divergences rather than absorbing them silently:

- **Category A** — implement the real algorithm and cite it in the doc
  comment, the way `cartalith-urban`'s `js_hypot`/`js_exp` and
  `cartalith-hydrology`'s D8 both already do. TPI, curvature, flow
  accumulation, least-cost path and Poisson-disc sampling all belong here.
- **Category B** — implement with an explicit doc-comment note naming what
  literature it is grounded in and what had to be tuned for this fictional
  world (the same treatment `build_soil_fertility` already gives Jenny 1941).
  Settlement-linked suitability, resource accessibility and route importance
  belong here — and mostly **already exist** per §1 above, just not yet
  composed for landmark purposes.
- **Category C** — implement with an explicit doc-comment note saying "this
  is an engineering weighting, not an established formula" — the honesty
  `PROVENANCE.md`'s own algorithm-vs-crate framework already asks for
  elsewhere. `S_castle`, `S_sacred` and any other weighted-sum landmark score
  belong here, and every weight must be named as a **tunable constant** with
  a comment, not a bare literal, so a later calibration pass can find them.

A struct- or module-level `// Category: A|B|C` marker (or an equivalent doc
comment convention) should be picked before milestone 1 starts, so it is
consistent from the first line of code rather than retrofitted. The
convention picked was the doc-comment one: a module-doc section naming the
category (`analysis.rs`'s "§31 Category A", `landmark.rs`'s A/B/C
breakdown), and a **Category X** lead on the doc comment of each item that
needs its own (`analysis::visibility`, `VIEW_OBSERVER_CAP`).

## 3. Milestones

Dependency-ordered. The research's own Phase 1-6 (`LANDMARK_GENERATION_
RESEARCH.md`, "Implementation Priority") is the outer frame; each is split
here into a piece this repository can land and verify on its own, the same
granularity `URBAN_MORPHOLOGY_SCOPE.md` used to turn one "ports cleanly"
sentence into seventeen real milestones.

Each milestone below carries its dependencies and its **done when** — the bar
it has to clear. Neither is a status: the dependency graph does not change as
work proceeds, and the bar is what the work is measured against.
`cartalith-native/docs/STATUS.md` says which of them have cleared it.

### M1 — Extract the analytical field library (Category A)

Pull the TPI-equivalent math out of `build_ao` and the curvature math out of
`build_landform_field` into standalone, reusable, multi-scale functions —
most plausibly in `cartalith-terrain` beside `build_relief_field`, which
already lives at the right layer (`ARCHITECTURE.md`'s ladder: analytical
field, not renderer-private). Add the one real gap this milestone can close
cheaply: aspect (confirmed absent, same shape of computation as slope).
Consolidate the three separate slope recomputations found in §1 into calls
against the one canonical field where practical — flagged as a possible
follow-on cleanup, not required for this milestone's own "done."

**Not blocked on anything** — every input already exists in some form.

**Done when**: `topographic_position_index(field, radius)` and a standalone
`terrain_curvature(field)` (placeholder names — the library this milestone
defines is `cartalith_terrain::analysis`) exist as unit-tested,
resolution-independent functions returning a `Vec<f32>` the same shape as every other raster in this
project; `build_ao`'s own output is proven **bit-identical** before and after
the extraction (the same "refactor must not move a golden number"
discipline `DECISIONS.md` §7f used for the pre-carve flow skip), so the 2D
renderer's shipped look does not silently change as a side effect of this
milestone.

### M2 — Hydrological candidates: waterfall, ford, confluence (Category A + B)

Compose what already exists (`compute_flow`, `build_channels`,
`strahler_from_receivers`, `trace_river_polylines`, `build_water_bodies`) into
research §7's explicit constraint chain (`river = true AND gradient >
threshold AND vertical drop > threshold AND flow accumulation > minimum`).
The one real new piece: extract confluences as a first-class labelled list
from the receiver tree rather than leaving them implicit.

**Not blocked on anything** in §1 — the underlying fields are all real.

**Done when**: a hand-built fixture with a known steep reach on a known-order
channel and a known confluence produces exactly the expected candidate list,
deterministically, for a fixed seed — the same "shape a fixture to reach the
code" discipline `CLAUDE.md` names as hard-won.

### M3 — Mountain-pass candidates (Category A, mostly already built)

Promote `build_route_corridors` from a private route-cost relief term to an
exposed, reusable landmark-candidate field, per research §8's `S_pass`. Add a
real 2D saddle test (low point between two high regions) generalising what
`measure_bridge.rs`'s 1D `section_crossings` already does along a drawn line.
*(The saddle half was declined when M3 was built, and the reason is design,
not a gap — `saddle`'s `not_built` string: a saddle with connectivity is a
mountain pass, which is generated; a saddle without it is a shape, not a
landmark.)*

**Not blocked on anything.**

**Done when**: reading the corridor field for landmark purposes on a real
generated world reproduces the same statistics already measured in
`DECISIONS.md` §7i (≈30.8% of land carrying any value, ≈1.02% above half
strength) — proof this milestone is genuinely reusing the existing,
battle-tested field rather than quietly reimplementing a different one.

### M4 — Peak / ridge / prominence candidates, generalised to 2D (Category A)

Generalise `measure_bridge.rs`'s prominence-filtered local-maxima algorithm
(`RIDGE_PROMINENCE_M = 100.0`) from "along one drawn section" to "over the
whole grid," using M1's extracted TPI field as a cheap first-pass candidate
filter before the more expensive real-prominence walk.

**Blocked on M1** (needs the extracted TPI field to be worth doing this way
rather than reimplementing local-maxima detection from raw height a third
time).

**Done when**: the same threshold constant reproduces the same peak on a
fixture the Measure tool would also flag along a section drawn through it —
proof the two implementations agree rather than silently diverging on what
"prominent" means.

### M5 — Resource- and settlement-linked candidates (Category B)

Compose already-existing data — `build_resource_potentials`,
`build_soil_fertility`, `get_settlements`, `civ_dijkstra_path`/`WayRouter` for
accessibility — into research §14's `P(L | R, S, C, T)`. This is the
milestone where "genuinely new work" is smallest relative to "reuse of
existing subsystems," per §1's inventory.

**Not blocked on anything**, though it is naturally sequenced after M2-M4 so
a resource-linked candidate can also cite a nearby hydrological or
topographic feature in its causal chain (research §1's whole point).

**Done when**: on a constructed fixture, a resource-rich cell near a
settlement with real road access scores measurably higher than an otherwise
identical resource cell in the wilderness — and the score's provenance (which
terms contributed) is inspectable, not just a final number.

### M6 — Spatial competition / Poisson-disc filtering (Category A)

A real, reusable, multi-class exclusion-radius sampler — either a proper
Bridson (2007) grid-accelerated Poisson-disc implementation, or a generalised
version of `find_settlement_seeds`' rank-then-suppress pattern extended to
variable radii per research §16 (`r = f(class, importance, terrain,
region)`). This section originally named `cartalith_spatial::QuadTree<T>` as
the acceleration structure; that type was retired unused (§1), and the
exclusion queries run over a bucket grid, `landmark.rs::Buckets`, whose doc
comment also records why Bridson (2007) was declined.

**Not blocked on anything**, but should land after M2-M5 so there is a real
multi-class candidate cloud to filter rather than a synthetic one.

**Done when**: a synthetic two-class, two-radius candidate cloud reproduces
the documented minimum-separation property, and — per this project's own
standing rule that golden-matching is necessary and not sufficient — is
mutation-tested with at least one quantised/boundary fixture in addition to
random inputs, the same pattern `URBAN_MORPHOLOGY_SCOPE.md` milestone 3 had
to learn the hard way.

### M7 — Viewshed (Category A, the expensive one, entirely new)

The one landmark input with zero existing code and the highest cost risk.
*(As of the 2026-08-30 inventory. Since built: `landmark.rs`'s `Derived::vis`,
bounded by `VIEW_RADIUS_KM` / `VIEW_MAX_CELLS` / `VIEW_OBSERVER_CAP`. Checked
2026-09-24, it is read by `pool_military`, which serves `fort`, `watchtower`,
`fortified_pass` and `fortified_crossing`, and by `pool_volcanic` and
`pool_border_marker`. It does not follow the `needs_viewshed` flag: `peak` is
flagged and reads no `vis`, and the fortified pass and crossing read it
unflagged. Status is in `STATUS.md`.)* Scoped down from "every cell sees every cell" to research §9's own framing —
a **bounded set of observer points** (settlements, road/pass samples, pass
candidates from M3) rather than a dense viewshed field — per the cost note in
§5 below.

**Depended on an owner decision** on the accuracy/cost tradeoff (§5, and §4
question 5 — ruled: the conservative default is the budget, and it is carried
by `landmark.rs`'s `VIEW_RADIUS_KM`, `VIEW_MAX_CELLS` and `VIEW_OBSERVER_CAP`,
each with its reasoning), and on M3 for a sensible observer set.

**Done when**: wall-clock time is measured and reported honestly at three
real grid sizes (this project's own convention, not a single "it's fast
enough" claim), and if a GPU path is ever attempted for it, the CPU and GPU
results are compared under `DECISIONS.md` §7a's principled-equivalence bar
rather than assumed identical.

### M8 — Category C suitability synthesis + the Landmark object model

Combine M1-M7 into weighted scores per research §17-19 (`S_castle`,
`S_sacred`, etc., every weight a named, commented constant per §2 above), and
build the `Landmark` struct itself — research §22's object model, including
`causal_chain`.

**Blocked on** however many of M2-M7 are wanted for the landmark types in
play (a "physical landmarks only" first cut could ship after M1-M4 alone),
and on §4 questions 1 and 3 (persistence, parity-contract status — both since
ruled) before the struct's own shape is finalised.

**Done when**: a full pipeline run on a real generated world produces a
bounded, causally-labelled landmark set, and — this project's own "watch for
silently-empty golden output" rule — is exercised against at least one
edge-case world (all-ocean, single-cell landmass) without panicking across
the gdext boundary or silently returning nothing when something was expected.

### M9 — Cultural interpretation and temporal state (research §24-26)

One physical feature, several civilisations' readings of it (research §26);
state transitions (discovered → named → …→ ruined, research §25). This is
research's own Phase 6, and it is the one milestone genuinely blocked on
things outside this document's own dependency chain: the conflict/battle
entity (`STORY_PLANNING_SCOPE.md` SP-4 — needed for the "Conflict/decline"
link in a ruin's causal chain) and the Markdown Vault entity-kind decision
(§4 question 2).

**Depends on**: M8, `STORY_PLANNING_SCOPE.md` SP-4, and §4 questions 1-2
(both since ruled — a ruling authorises the work, it does not say it is
built).

**The conflict half, as designed once SP-4's conflict entity existed.**
`battlefield` reads `LandmarkInputs::battles`: every drawn
`ConflictKind::Battle`, anchor-resolved to where it is now. Two neighbouring
kinds cannot be built from that input, each for its own reason, and both
reasons are carried in `landmark.rs::kinds()`' `not_built` strings:

- `battlefield_historic` would put a second record on a cell `battlefield`
  already covers, unless a present-year read separates a past battle from one
  being fought — a landmark whose kind changes with the Timeline cursor, which
  is research §25's temporal state (§4 question 1's per-landmark *state*, not
  its save slot).
- `destroyed_fortress` has no destruction to read: `Conflict::outcome` is free
  text by `STORY_PLANNING_SCOPE.md` §5's deliberate no-resolution-model rule.

The Cultural-family kinds stay unbuildable — most on research §26's own rule
that cultural meaning must not be hardcoded into geography (it needs the
civilisation's traits as an input, which the pass does not take), `tomb` and
`monument` for want of a historical figure or event to commemorate. Each
kind's own reason is its `not_built` string.

*Checked against the code 2026-09-24.* Several `not_built` strings are now
false:
- `monument` says no battle exists as a record. Drawn battles reach this pass
  as `LandmarkInputs::battles`.
- `ruin` and `abandoned_settlement` say the collapse drops the site's
  coordinates. `TimelineSnapshot::collapse_flags` and per-step snapshots
  exist.
- `ancient_road` says way history is not retained. Each `TimelineSnapshot`
  keeps that year's `ways`.

The strings are also **not shown to the user**. `landmark_kinds` in
`cartalith-godot/src/lib.rs` emits a `not_built` key, and no `.gd` file reads
it; the shell shows one generic "not buildable" reason. `ALIGNMENT_AUDIT.md`
Part 1 item 8 has the full list. The kinds stay unbuilt, but the reasons need
rewriting.

Not specified further here — beyond the conflict half, research §24-26's
interpretation and state transitions have nothing in this repository to
compose against yet, and specifying them further would be guessing.

## 4. The owner questions — posed here, all since ruled

Posed, not answered, when this document was written — the same discipline
`STORY_PLANNING_SCOPE.md` §6 and `MARKDOWN_VAULT_SCOPE.md` §2 both used.
**All six have since been ruled.** Questions 1-5 were answered on 2026-09-06
(`LARGE_ITEM_RULINGS.md` rulings 10, 13, 12, 24 and 16) and answered again on
2026-09-23 by **Ruling AP**, which confirmed three of them, left ruling 24's
crate move superseded (question 4) and ratified what M7 had shipped for the
fifth. Question 6 was answered by ruling 14 (2026-09-06) and refined by
Ruling AL (2026-09-23).
Each question keeps its original reasoning, because that reasoning is why it
was a real question. **A ruling authorises work; it does not say the work is
built** — where each piece stands is `cartalith-native/docs/STATUS.md`'s.

1. **Does the landmark set live in the save tree, and if so, as what?**
   `DECISIONS.md` §7h fixed the save format as a tree; `SAVEFILE_COMPAT.md`
   §5 already reserves `entities/journeys.json` for a not-yet-built entity
   the way this would need an `entities/landmarks.json`. But unlike a
   journey (author-placed, cannot be recomputed), a landmark is **fully
   derived** from other already-saved rasters and entities — so a real fork
   exists: regenerate-on-load like most rasters (no save slot needed at
   all), or persist like a settlement because research §25's state
   transitions (discovered, named, monumentalized…) are exactly the kind of
   authored/accumulated state that *cannot* be recomputed from a re-run.

   **RULED — persist.** Ruling 10 (2026-09-06): persist, in
   `entities/landmarks.json`, for exactly the §25 reason above; it asked for a
   `SAVEFILE_COMPAT.md` entry and a format-version note. **Ruling AP
   (2026-09-23)** re-ruled it — *landmark state persists across save/reload* —
   and named the part still undesigned: what "state" means per landmark, at
   minimum a name the player gave it, with §25's discovered/named/
   monumentalized states as the fuller shape, **not yet scoped**. The slot
   itself is `project_bridge.rs`'s `SLOT_LANDMARKS` (`LandmarksDoc`: the
   dock's settings and the last run's placements); per-landmark authored
   state is the unscoped remainder, and it is what `battlefield_historic`
   waits on (M9).
2. **Does a landmark become a `cartalith_vault::EntityKind`?**
   `MARKDOWN_VAULT_SCOPE.md`'s own §2 table already proved that adding a
   kind is cheap (one variant, one `as_str` arm, one `parse` arm, one
   `entity_values` arm — CV-22/CV-02 both measured this in one session). The
   owner's own `design/vault-templates/Landmark template.md` already exists,
   independently converging on nearly the same field set the research's §22
   object model proposes (see §1's second "outside the checklist" finding).
   But `MARKDOWN_VAULT_SCOPE.md` §4's identity-strength table would need a
   new row, and a landmark's id is likely to be as weak as a continent's or
   province's (derived, not persistent across a regenerate) — worth deciding
   before, not after, notes start linking to it.

   **RULED — yes.** Ruling 13 (2026-09-06): finish the wiring — the
   half-wired state (a template that exists, a kind nothing resolves) was the
   worst of the three options. **Ruling AP (2026-09-23)** re-ruled it and
   settled the identity question this paragraph raised: the id is expected
   to be as weak as a settlement's or province's — derived, not stable across
   a regenerate — and that weakness is **accepted, not a blocker**. The
   variant is `cartalith_vault::EntityKind::Landmark`; its id is
   `Landmark::key()` (`kind@x,y`) hashed to a 52-bit `entity_id` by
   `cartalith_vault::links::landmark_entity_id`, and
   `MARKDOWN_VAULT_SCOPE.md` §4's identity table carries its row.
3. **Does `DECISIONS.md` §7a/§7d's parity contract apply here at all?**
   `reference/FUNCTION_INDEX.md` was grepped for "landmark" and returns
   nothing — there is no JS behaviour anywhere to be faithful to, the same
   finding `MARKDOWN_VAULT_SCOPE.md` §0 made about the vault itself. Is this
   entire subsystem tagged "principled equivalence / divergence-by-addition"
   from day one (§7a's "implement the same academic principles… judge by
   visual/qualitative outcome," the same tag `STORY_PLANNING_SCOPE.md` §5
   carries), or does the owner want a different verification standard for
   the Category A pieces specifically, since several of those (TPI,
   viewshed, Poisson-disc) do have a textbook-correct answer that JS-parity
   testing was never going to give anyway?

   **RULED — the parity contract does not apply.** Ruling 12 (2026-09-06):
   exempt, since there is no reference to diff against, to be written into
   `DECISIONS.md` as a §7-series note where a lane looks. **Ruling AP
   (2026-09-23)** confirmed it: all landmark work is divergence-by-addition,
   tested for internal correctness and determinism rather than against a
   reference that never existed for this subsystem. The Category A half of
   the question was not separately ruled — per ruling 12, the project's
   standing bar (property tests, mutation-tested constants, probes on drawn
   output) applies to it anyway.
4. **Where does this live in the crate graph?**
   §1 found the closest thing to a TPI implementation sitting inside
   `cartalith-godot`'s renderer (presentation layer), which is arguably
   already mis-homed under `ARCHITECTURE.md`'s own rules independent of
   landmarks. A new `cartalith-landmarks` crate (matching the
   one-crate-per-subsystem pattern `cartalith-urban`/`cartalith-vault` set)
   versus folding into `cartalith-civ` (which already owns settlement
   suitability and resource potentials) versus splitting analytical fields
   into `cartalith-terrain`/`cartalith-engine` and leaving only synthesis in
   a thin new crate — this is a real architectural fork, not just a filing
   question, because it decides which crate can depend on which for M1-M9.

   **RULED — landmark code stays in `cartalith-civ`; no crate split.** Ruling
   24 (2026-09-06, against the recommendation) first said to consolidate the
   "terrain-derived half" into `cartalith-civ`. Measured before anything
   moved, that half — `cartalith_terrain::analysis`, M1's library — is
   general terrain analysis that landmarks *consume* (`sample_bridge.rs` also
   reads it as user-facing fields), not landmark-only code, and moving it
   would have put a terrain primitive in the civilisation crate and inverted
   the dependency the split keeps. Put back to the owner, the existing
   arrangement was ratified on 2026-09-23 (recorded on `OUTSTANDING_WORK.md`'s
   closed ruling-24 row), and **Ruling AP** the same day: landmark code stays
   in `cartalith-civ`, no crate split. So: synthesis in `cartalith-civ`
   (`landmark.rs`), the analytical field library in `cartalith-terrain`
   (`analysis.rs`). The TPI-inside-the-renderer finding above was answered by
   M1's extraction, with `build_ao` itself left bit-identical per
   `DECISIONS.md` §7a.
5. **What is the viewshed cost budget?**
   §5 below states the complexity honestly; it does not choose a number.
   Observer count, radius cap and grid resolution are all owner-facing
   tradeoffs (accuracy vs. generation time vs. memory), not something to
   guess at from inside this document.

   **RULED — the conservative default stays.** Ruling 16 (2026-09-06, amended
   by the owner): cheap and coarse, plus a manual "recompute and refine"
   action in the shell's existing `Refine detail for the current view`
   vocabulary rather than a second one. M7 was then built on a disclosed
   conservative default — a 40 km horizon clamped to [2, 104] cells, over a
   sparse observer set of settlements plus route samples capped at 256
   (`VIEW_RADIUS_KM`, `VIEW_MAX_CELLS`, `VIEW_OBSERVER_CAP`), which makes the
   pass `O(observers · r²)` and independent of grid size, the shape §5
   predicted. **Ruling AP (2026-09-23)** ratified that default as-is and
   authorised no further work. **Open, and flagged rather than assumed:**
   Ruling AP does not say whether ruling 16's manual refine action still
   stands, and ruling 16 itself left open whether a refined result is
   view-scoped or whole-world and whether it persists (which touches
   question 1).
6. **How does a generated landmark relate to the existing manual icon tool?**
   A user can already hand-place a `family: "feature"` icon (e.g. `slot:
   "mountain"`) via `annotations/icons.json` (`SAVEFILE_COMPAT.md` §11.2).
   Does a procedurally generated landmark become one of these icons for
   rendering purposes (one representation, two origins), or a wholly
   separate data/render path? This affects the save format, the renderer,
   and whether M6's spatial-competition radius needs to consider
   hand-placed icons as pre-existing "occupied" points.

   **RULED — one representation, two origins** (ruling 14, 2026-09-06). One
   collection with an `origin` field: `ManualIcon` carries
   `IconOrigin { Manual, Generated }`, and `annotations/icons.json` gained an
   optional `origin` (absent means manual, `SAVEFILE_COMPAT.md` §11.2), so a
   pre-`origin` document loads with every icon hand-placed. The ruling's three
   consequences, as designed:
   - **The renderer draws one layer.** Where a generated landmark and a
     generated POI glyph share a cell, the ring wins. Ruling AL (2026-09-23)
     then put the landmark's own per-kind glyph inside its class ring and made
     it clickable, keeping the two things ruling 14 chose the ring for: radius
     encodes class, size within the class encodes importance.
   - **M6's spacing sees hand-placed icons**, so generation cannot place a
     landmark on top of one: `LandmarkInputs::manual_icons`, assembled in
     `landmark_run_inner` through `landmark_bridge::icon_to_mark` (manual
     origins only) and honoured by `generate` with a
     `MANUAL_ICON_EXCLUSION_KM` = 3 km exclusion. **This consequence is the
     ruling's whole justification for one layer over two, and it was first
     built inert** — the input existed while nothing assigned it, so the
     running app could still place a landmark on an icon. Check the
     assignment, not the field.
   - **A regenerate replaces only the generated ones.**

## 5. Cost and feasibility: the expensive parts, stated honestly

**Viewshed is the one that matters.** A naive all-pairs viewshed (every
candidate cell tested for visibility from every observer, each test a
line-of-sight ray march) is `O(candidates × observers × ray_length)`. At this
project's own documented ceiling — `MEMORY_OPTIMIZATION_SCOPE.md`'s **8192²
= 67 108 864 cells**, already the UI's stated maximum resolution — that is not
a viable per-generation pass at any reasonable observer count. The research's
own §9-10 already points at the affordable version: a **sparse set of
observer points** (settlements, road/pass samples — not every cell), each
with a bounded-radius viewshed rather than a whole-map one, matching the
"large numbers of observer points, not universal coverage" framing its own
reference 4 (Inglis et al. 2022) is cited for.

**What this project already knows about parallelising per-cell field work
here, and what does not transfer:**

- `CPU_MULTITHREADING_SCOPE.md` established the load-bearing distinction this
  whole area turns on: a per-cell pass with **no cross-cell dependency**
  (`output[i] = f(input, i)`) parallelises for free and bit-exactly with
  `rayon`'s `par_iter`; a pass with **real cross-cell state** (flow
  accumulation's descending-height order, `compute_stress`'s scatter writes)
  needs an actual algorithmic redesign, not just an added `par_iter`.
  Viewshed from a *fixed* observer set is the *first* shape: each candidate
  cell's visibility from each observer is independent of every other
  candidate cell, so it is `par_iter`-safe in principle, the same way
  `compute_resistance`'s own doc comment already states its independence
  explicitly (`cartalith-terrain/src/lib.rs`, "no cross-cell dependency,
  exact under parallel execution"). *(In principle only: M7's
  `analysis::visibility` stays serial on purpose, because its output is an
  `f32` sum across observers into one shared grid, and summing in observer
  order is what keeps it reproducible run to run.)*
- **What does not transfer is the cost shape.** `GPU_LAYER_INTEGRATION_
  SCOPE.md` milestone 9's flow-accumulation redesign is the closest analogue
  this project has actually built and measured for "a genuinely sequential
  hydrological algorithm made parallel" — but that milestone's whole
  technique (pointer-doubling over a receiver *forest*, `O(log n)` rounds) is
  specific to accumulation over a tree structure. Viewshed has no such
  structure to exploit; its cost is inherent to the geometry (rays, not a
  forest), so the honest expectation is "parallelises well, does not get
  cheaper by a clever reformulation the way flow accumulation did."
- Milestone 9's own measured numbers are the right calibration for what a
  from-scratch GPU per-cell kernel costs at this project's real sizes: **4.6×
  at 512², 10.4× at 1024², 15.5× at 2048²** over the equivalent CPU pass, but
  **a 128×128 GPU dispatch loses to CPU outright** (0.20×) on pure dispatch
  overhead — the same "GPU is not free at small sizes" finding
  `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 6 established generally. A
  viewshed kernel should expect the same shape: a real win at the project's
  larger sizes, a real loss at its smaller ones, and it needs to be measured
  on this codebase's actual hardware rather than assumed.

**Poisson-disc / spatial competition (M6)** is comparatively cheap: `O(n log
n)` with a spatial index (a bucket grid, `landmark.rs::Buckets` — the
`QuadTree<T>` this sentence first named was retired unused, §1), and the
existing `icon_brush_stamp` precedent already caps its own per-call work
(`ICON_BRUSH_MAX_DARTS = 1500`) for exactly this reason — the pattern to
follow, not a new problem to solve.

**Memory**, for calibration against any new per-cell field this subsystem
adds: `MEMORY_OPTIMIZATION_SCOPE.md` measures **256 MB for a single `f32`/
`u32` raster at 8192²**, and civ_continents was deliberately built *without*
a per-cell raster for exactly this reason (`MARKDOWN_VAULT_SCOPE.md` §1: "no
new per-cell memory… 268 MB at this port's 8192² ceiling for a lookup nothing
else performs"). Every new analytical field this subsystem adds (TPI at N
scales, curvature, aspect, an observer-visibility accumulator) should be
costed against that number before it is added, not after.

## 6. What this document is not

It does not implement anything. It does not pick a crate boundary, a save
format, or a verification standard — those were §4's questions 1-4, posed
for the owner rather than decided here, and §4 records how the owner ruled.
It does not promise a "done when" for M9, whose cultural half still has
nothing in this repository to compose against. Where
this document could not find something, it says so plainly rather than
guessing — the standing rule this project has been bitten by breaking before.

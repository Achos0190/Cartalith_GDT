# Phase 2 scope: milestones

`ROADMAP.md`'s one-paragraph sketch for Phase 2 is a phase-level sketch,
not a milestone boundary — the same relationship `MVP_SCOPE.md` had to
`ROADMAP.md`'s Phase 1 paragraph. Phase 1 itself shipped as many
separately-verified stages, not one pass; Phase 2 does the same. This
document is the list of milestones and what each one is, grown as each was
scoped rather than written in one shot before any of it was known.

**It defines milestones. It does not track them.** For where any milestone
stands — done, partial, blocked, declined — read
`cartalith-native/docs/STATUS.md` (rows P2-01…P2-21), which is the only place
progress is recorded.

**How to read it.** Twenty-one milestones, numbered 1–21, plus the first
milestone 9, kept as a superseded record. Each section is the milestone's
definition, followed where one was written by a **What shipped** record. That
record is history, in the past tense, kept for the reference quirks, harness
bugs and porting subtleties it carries. Where later work overtook a record, a
*Since then* line says so and names the `STATUS.md` row. Reference line
numbers resolve against the frozen `reference/Cartalith Gen1 v2.10.html`
(function-start citations re-checked 2026-09-23). A bare `CHANGELOG.md` means
the retired `cartalith-native/docs/CHANGELOG.md`, whose "Phase 2 milestone N"
entries (1–17 and 20) carry each pass's full narrative.

## Done means (per milestone, not once for the whole phase)

Each milestone: golden-verified against the real reference engine with a
real, justified tolerance, and `cargo test -p cartalith-civ` proves it with no
Godot involved. Two kinds of milestone verify differently, and each says so.
Where the reference has no algorithm to check against (territory, milestone
10, and provinces, milestone 16 — `DECISIONS.md` §7b), unit tests stand in.
For small, pure, branch-complete functions with no RNG or iteration-order risk
(17's trade-balance rule, 18, 19), each pass chose real unit tests over a
golden harness and said why. That is a precedent this port set
(`ECONOMY_SCOPE.md`, "What was actually built this pass"), not a rule stated
in `PARITY_TESTING.md`. `STATUS.md`'s row is updated to reflect
real state, verified against the code. Nothing outside a milestone's own
explicit scope gets implemented in that milestone's pass — flag and stop if
something turns out unavoidable, report it, don't silently expand.

## Milestone 1 — affordance fields foundation

Lithology classification, soil fertility, water access
(`buildLithology`/`buildSoilFertility`/`buildWaterAccess`, reference lines
5835/5852/5866). New crate `cartalith-civ`, zero `gdext` dependency, golden-
verified (lithology bit-exact, soil/water at `1e-4`). `CHANGELOG.md`'s "Phase 2
milestone 1" entry carries the full record.

**Investigated before milestone 1 was written**: traced
`currentSettlementSuitability` (the "v1.30 one function" `ROADMAP.md`
flags) and found it several milestones away, not a starting point — it
depends on resources, carrying capacity, route corridors, landmass
quality, coast SDF, water-body classification, and river network order,
none of which existed. The reference's own v0.104 history drew the exact
boundary milestone 1 adopted: "this lands lithology → soil → water access.
Resources + carrying-capacity + settlement suitability are the v0.105–
v0.106 follow-ups."

## Milestone 2 — water-body classification

**Port `buildWaterBodies`** (reference line 5753) — the ocean/lake/land
classifier every downstream affordance and civ field reads
(`currentWaterBodies()`). Two real algorithms, not one:

1. **Connected-components flood fill** over below-sea-level cells
   (4-neighbour, world-wrap-aware) — the largest component is "ocean"
   (class 1), every other below-sea component is an inland sea/lake
   (class 2).
2. **Priority-flood depression fill** (Barnes-style min-heap) for
   above-sea depressions that pool water into lakes, gated on local
   moisture so arid basins stay dry. `PROVENANCE.md` already flags this
   exact algorithm: *"Hand-port, carefully. Equal-priority pop order
   decides the fill tie-break and therefore lake shape."* The reference's
   own v1.87 comment on this function is explicit about *why* the tie-break
   matters and preserves it deliberately — read that comment (lines
   ~5776–5799) before writing the Rust port; the custom `MinHeap`'s
   sift-up/sift-down comparison operators (`<=` vs `<`) are not
   interchangeable here, they're the tie-break rule.

**In scope**: `buildWaterBodies` itself, including the lake fill-level
output (`fillOut`/`_lakeFill`, needed by later milestones' renderers, not
this one — just don't drop it from the function signature). Golden-verify
against the real reference engine, same technique/tolerance discipline as
milestone 1 (categorical `Uint8` output → expect bit-exact, same as
lithology).

**Out of scope for this milestone**: biome classification
(`classifyBiome`/`buildBiomeRaster`, reference lines 5736/6798) — it reads
`currentWaterBodies()` as an input, so it cannot land before this milestone
regardless, but is its own separate, smaller function and its own
milestone (3), not bundled in here just because it's next in line.
Resource potentials, carrying capacity, population density, settlement
suitability, factions, territory, roads — all still as far away as
milestone 1 found them.

**Where the code goes**: `cartalith-civ` (same crate milestone 1 created),
depending on `cartalith-engine`'s `WorldState` fields the same way
milestone 1 did — read-only, no modification to `cartalith-terrain`/
`cartalith-climate`/`cartalith-engine`'s own output.

**What shipped.** `build_water_bodies` ported (connected-components flood fill +
priority-flood min-heap, index-for-index port of the reference's own
heap). Golden-verified bit-exact on classification, `1e-4` on fill-level,
both fixture cases. Found and root-caused a real Node-harness bug along
the way (the reference's `state` literal seeds `tect.seed` from
`Math.random()` at load — the harness had been setting an unrelated
top-level `state.seed`, leaving generation nondeterministic until fixed to
set `state.tect.seed`), not a fixture/tolerance issue. See
`CHANGELOG.md`'s "Phase 2 milestone 2" entry for the full record.

## Milestone 3 — biome classification

`classifyBiome` (pure temp/rain → category) + `buildBiomeRaster` (applies
it per-cell, with ocean/lake cells overridden from milestone 2's water-body
classification). Ported to `cartalith-civ`, bit-exact both cases, first
attempt. `buildCartBiome` (the *different*, denser 15-category Cartalith
editor-bridge biome-paint auto-fill) confirmed out of scope — no consumer
exists anywhere in this port (no painting UI, no editor integration) —
not implemented. See `CHANGELOG.md`'s "Phase 2 milestone 3" entry.

## Milestone 4 — carrying capacity, NPP, population density

Split out from resource potentials after checking real size (2026-08-16):
`buildResourcePotentials` (reference lines 6085–6193, ~108 lines, 9
resource-type scoring rules) is substantially larger than the other three
functions here (`buildCarryingCapacity` ~15 lines at 6238,
`estimateRegionalDensityKm2` ~21 lines at 6217, `biomeDensityResidual` and
`buildNPP` both small) — bundling it in would make this milestone
disproportionate. It becomes its own milestone 5.

**In scope**:
- `buildCarryingCapacity` (line 6238) — soil × temperature-bell × water
  modifier × biome-density-residual. Needs `biomeDensityResidual` (line
  6193, one-line lookup) and `WETLAND_DENSITY_RESIDUAL` (small const) plus
  `buildWetlandMask` (reference line 6839, small; ported with this milestone
  as `build_wetland_mask`). All
  already-real inputs otherwise (soil, water access, biome, temp, field —
  all from milestones 1–3).
- `buildNPP` (line 6497) — Miami-model net primary productivity from
  temp/rain. Simple, already-real inputs.
- `estimateRegionalDensityKm2` (line 6217) — population density; reads
  carrying capacity + water access + biome + NPP, all real once the above
  two land.

**Out of scope for this milestone**: `buildResourcePotentials` (needs
`boundary_type`/`shear_field` — confirmed already computed in
`cartalith-terrain`'s tectonic substrate, but check whether retained on
`WorldState` or need the same fix `crust_field` got in milestone 1 — and
9 distinct per-resource scoring rules; genuinely milestone 5's own scope,
not this one's).

**What shipped.** `build_carrying_capacity`/`build_npp`/`estimate_regional_density_km2`
ported to `cartalith-civ`, `1e-4` tolerance, both fixture cases passed
first attempt. A real short-circuit gotcha caught: the reference's
`bK&&biome` biome-residual gate requires *both* truthy, not just
arithmetic that happens to equal the unconditional case at `bK=0` — this
port's gate matches the reference's condition exactly, not just its
output at the default. **Confirmed for milestone 5**: `WorldState`
(`cartalith-engine/src/lib.rs`) genuinely has no `boundary_type`/
`shear_field` fields — they exist only inside a local `stress` struct
computed mid-`generate_terrain` and are discarded past it, the same
situation `crust_field` was in before milestone 1's fix. Milestone 5
needed the equivalent retention fix first, and made it. See
`CHANGELOG.md`'s "Phase 2 milestone 4" entry for the full record.

## Milestone 5 — resource potentials

`buildResourcePotentials` (reference lines 6085–6172): all 15 fields
(copper/tin/iron/gold/salt/timber/lead/silver/clay/buildstone/flint/
obsidian/gems/sulfur/alum — `RESOURCE_KEYS`, not `SUIT_RESOURCE_KEYS`'s
smaller 9-ore settlement-suitability subset, and not block 2's own larger
`CIV_RESOURCE_KEYS`). Needed the predicted `WorldState` retention fix:
added `boundary_type`/`shear_field` (from `cartalith-terrain`'s
`StressResult`), matching `crust_field`'s milestone-1 precedent exactly.
`1e-4` tolerance, both cases passed first attempt. Production scarcity
default (`scarcity=true, scarcity_legacy=false` — original six unthinned,
nine v1.31 additions thinned) verified with a dedicated test, not by
inspection. See `CHANGELOG.md`'s "Phase 2 milestone 5" entry.

## Milestone 6 — settlement-suitability prerequisites: route corridors, landmass quality, coast SDF

Ported `buildRouteCorridors` (line 5903), `buildLandmassQuality` (line
5970), `buildCoastSDF` (line 7462, always via the true-Euclidean JFA
backend — the only path this port's real caller uses) to `cartalith-civ`.
All three golden-verified, `1e-4` tolerance, three cases (two established
fixtures plus a new 48×40 case added specifically to exercise
`buildRouteCorridors`'s nonzero branch — both established fixtures'
tiny grids genuinely produce zero corridors from the real reference
engine, confirmed real, not a bug, but untested by an all-zero fixture).

Root-caused a real harness bug before trusting the data: `field[0..5]`
was ~1e-5 off the trusted fixture on the first attempt — not a wrong seed,
`golden_parity_carve.rs`'s fixture uses `w_iters=12` (a speed override),
not the real default `70`. Matching it exactly reproduced the fixture
bit-for-bit. See `CHANGELOG.md`'s "Phase 2 milestone 6" entry for the full
record, including three real porting subtleties (raw vs. `*GW`-scaled
slope field, 8-neighbour vs. 4-neighbour flood fill, JFA vs. chamfer SDF
backend).

## Milestone 7 — settlement suitability / seed-finding

The "v1.30 one function" `ROADMAP.md` originally named as this phase's
landmark. `buildSettlementSuitability`/`findSettlementSeeds` (reference
lines 6319/6418) ported to `cartalith-civ`, golden-verified.

**River-network question resolved**: `build_channels` IS already a
line-for-line port of `buildRiverNetwork`'s channelization loop (confirmed
via its own doc comment). `WorldState.stream_order` still isn't the right
input for this caller, though — it's computed too early (mid-carve, before
the channel-lock stamp), while the reference's own `carveRiverValleys()`
explicitly nulls `_riverNet` at its very end so `currentSettlementSuitability()`
always rebuilds fresh on the FINAL post-carve field. Fixed with
`fresh_river_order()`, reusing `build_channels`/`strahler_from_receivers`
directly rather than porting a second implementation.

**A real gap closed**: `buildFloodField` (reference line 5634) had no
prior port anywhere in this crate — added as `build_flood_field`, since
`ctx.flood` genuinely reads it in production.

**A genuine ambiguity found and resolved**: two different real reference
call sites use different seed-finding thresholds (`SETTLE_SEED_THRESH`=
0.42 for the interactive advisory view this port doesn't have, vs. `0.65`
— the function's own bare default — for the `settlement_seeds.json`
export, the only headless production caller and this port's closest
analog). First extraction used 0.42 and found a real mismatch (6 seeds vs.
5) despite the suitability field already being bit-identical; re-extracted
at 0.65, matched exactly. See `CHANGELOG.md`'s full "Phase 2 milestone 7"
entry for the complete reasoning.

Both fixture cases passed (suitability `1e-4` tolerance, seeds checked by
exact `(x,y,score)` triples) after the threshold fix, without touching the
underlying formula.

## Milestone 8 — settlement placement + faction assignment

Investigated 2026-08-16: `_civIterativeAutoWorld` (reference line ~25336,
block 2's real "auto-populate" entry point) mixes pure algorithm with
direct DOM reads (`document.getElementById('civNCap')` etc.) and `alert()`
calls — that DOM-coupled orchestration shell is **not** what this milestone
ports; it becomes real Godot-side UI/orchestration logic later, not a JS
transliteration. But the algorithmic core it calls is pure, deterministic,
and testable exactly like every prior milestone:

- **Land-component labelling** — a plain 4-connected flood fill over
  land cells (world-wrap aware), giving each landmass an id. Same
  algorithmic shape as milestone 2's water-body fill and milestone 6's
  landmass-quality fill (a third occurrence of this pattern in this
  codebase — reusable *pattern*, still a distinct predicate each time).
- **`_civSnapLand`/`_civSnapCoast`/`_civIsCoastal`** (reference lines
  ~20747/~20841/~20917) — snap a suitability-maximum seed onto real dry
  land (a shore-hugging maximum can round into a lake at full resolution),
  then onto the shore when the optimal site is near the sea (harbour towns
  sit ON the water), plus ocean-port coastal detection (inland-sea/lake
  shores explicitly don't count).
- **`_civAssignLandmassFactions`** (reference line ~25022) — apportions
  faction "seats" across landmasses (capacity-weighted, capped by
  candidate count), assigns concrete faction ids, and for a landmass with
  multiple seats, seeds multi-capital placement by suitability+spacing
  before nearest-seed-assignment for the rest. Fully pure, no DOM/RNG-UI
  coupling — the one RNG use in the wider function (`_civRng`, for
  anything beyond this) is out of scope here.
- **Settlement tier classification** (capital/city/town/village/hamlet by
  rank — the `isCapital`/`isCity`/`isTown`/`isVillage` cascade inline in
  `_civIterativeAutoWorld`, reference lines ~25409-25421) — small, pure,
  worth porting alongside the above since it's the direct next step after
  faction assignment and has no DOM dependency either.

**Out of scope for this milestone**: the `wantCounts`/user-fixed-tier-count
branch (reads DOM inputs directly — a real Godot UI control, if ever
exposed, is separate future work, not this milestone); `_civSeedVillages`
(the v1.70 additive village layer, gated on a toggle with no UI exposed in
this port yet); territory/province generation (`_civGenerateProvinces`,
`getCivTerritory`); culture, economy, roads (`ROADMAP.md`'s Journey Planner
sub-phase). `CIV_FACTIONS` (the faction name/colour/culture roster this
milestone's output indexes into) — check whether a minimal stand-in
(count + numeric id, no names/colours/culture yet) is sufficient for this
milestone's own golden-testing, since the full roster is presentation data
with no algorithmic content, not core scope here.

**Where the code goes**: `cartalith-civ`, same crate, same conventions.

**What shipped.** Land-component labelling (fresh 4-connected fill, deliberately
not a reuse of `build_landmass_quality`'s 8-connected one — a unit test
pins the distinction), `_civSnapLand`/`_civSnapCoast`/`_civIsCoastal`
(including two real preserved reference quirks: `_civSnapLand` never
world-wraps while `_civSnapCoast` does; `_civIsCoastal` always x-wraps
unconditionally regardless of `state.world`), `_civAssignLandmassFactions`
(ported line-for-line, including its 5-attempt spacing-then-fallback
capital-seeding loop), and settlement tier classification all landed in
`cartalith-civ`. `CIV_FACTIONS` confirmed to contribute only `.length` to
the algorithm — a plain `faction_count: i32` sufficed, no roster ported.
Both fixture cases golden-verified bit-exact on the first attempt, and
both genuinely exercise the multi-capital (K>1 seats) branch (checked, not
assumed). See `CHANGELOG.md`'s "Phase 2 milestone 8" entry for the full
record, including the harness technique (a small injected function
mirroring `_civIterativeAutoWorld`'s own inline candidate-building loop,
since that loop isn't a standalone callable in the reference).

*Since then (2026-09-23):* two pieces this milestone left out were ported.
One is `_civIterativeAutoWorld`'s centrality → tier feedback loop — the one
network build this port had matched the reference's *first* pass, not its
last (`civ_iterative_network`). The other is the `wantCounts` fixed-tier-count
branch, exposed in New World ▸ Generation (`place_settlements_with_counts`).
`STATUS.md` row P2-08 has the evidence. The remaining out-of-scope items each
became their own milestone: territory 10, roads 11–14, villages 15, provinces
16, economy 17 and 20, culture 18.

## Milestone 9 (superseded) — the investigation that called territory a dead end

Kept as a record, because both its method and its correction teach something.

**What it concluded (2026-08-16).** Before assuming `_civGenerateProvinces`/
`getCivTerritory` were reachable, this investigation grepped every write site
of `civTerritory[...]` in the reference and found two: `_civPaintTerritoryAt`
(reference line 15964, an interactive brush driven by pointer events) and a
save/load deserializer (line 26145, restoring a previously painted delta).
`getCivTerritory()` (line 14933) only lazily zero-allocates the raster. It
concluded that the reference computed no territory at all — territory was a
hand-painted, editor-only feature — so `_civGenerateProvinces`, although pure
and portable (a Voronoi partition of an *already-owned* territory raster into
per-settlement provinces), had no programmatic input in this port. Porting it
then would have produced a correctly tested function with no real caller.

**That conclusion was false, and the grep method is why** (correction,
2026-08-19, cross-repo documentation audit). `_civAutoPolity` (reference line
20665, wired to the "Recalculate Territories" button at line 26662) writes the
raster through a local alias — `terr[i]=fac[i]`, line 20696 — which a grep for
`civTerritory[` cannot match. It runs `buildTravelCost` plus a multi-source
Dijkstra seeded from every settlement, diagonal-weighted, capped at
`MAX_REACH = GW*0.35`. The vendored `docs/research/political-fragmentation.md`
already described it. **A write-site grep misses every write made through an
alias; grep for the function that owns the data as well.**

**What still stands.** Milestone 10's territory design (`DECISIONS.md` §7b) was
built on this investigation's premise. On 2026-08-19 the owner ruled that it
stays the only mode — no reconciliation against `_civAutoPolity`, no second
mode (§7b's resolution). It also produces exactly the per-cell shape
`_civGenerateProvinces` needs (`Vec<i32>` faction id, `0` = unowned, matching
`civTerritory`'s own `Uint8Array` convention). That was re-checked against the
reference source, not against this note, before porting it as milestone 16.

## Milestone 9 — settlement population + naming

Investigated 2026-08-16, choosing between the milestone-8 fork's three
candidates: `_civSeedVillages` is UI-toggle-gated with no clean way to
verify it matters without the toggle (deferred again); roads/Journey
Planner is real but `ROADMAP.md` already calls it its own large sub-phase,
not investigated further here to keep momentum on a bounded win. Settlement
population/naming has a clean boundary and real programmatic inputs
(milestone 8's placed settlements + faction assignment), so it's this
milestone.

- **`_civBasePopForKind`** (reference line ~23433) — trivial: a lookup
  table (`_CIV_BASE_POP_BY_KIND`) by settlement tier, default `120`.
- **`_civSettleName`** (reference line ~20717) — RNG-driven syllable-
  combination naming, keyed by the settlement's faction culture
  (`_civCultureByKey`, reading `civFactionCulture[faction]`, falling back
  to `'common'`). Small (`CIV_CULTURES`, reference lines ~14607-14635, a
  short array of culture syllable/suffix tables — port the data verbatim,
  it's not algorithmic content to redesign). **This is RNG-driven**, so it
  must golden-verify against the *exact* RNG call sequence, not just the
  final string — same discipline `PARITY_TESTING.md` demands for
  everything downstream of `mulberry32` ("port the RNG first... a
  different PRNG makes every downstream comparison fail for reasons
  unrelated to whether the port is correct"). Find what `_civRng` actually
  is (reference: `rng=_civRng((state.seed||12345)*31337+999)`, milestone
  8's own harness likely already touched this) — almost certainly a
  `mulberry32` instance under a different name, reuse this port's already-
  verified `cartalith-rng` crate rather than re-deriving the algorithm.

**Out of scope for this milestone**: `_civSeedVillages`, territory/
provinces (milestone 9→10 renumbered by this decision — see below),
economy, roads. Culture is ported only as inert syllable/suffix *data* for
naming, not as any deeper cultural-simulation system — there isn't one at
this point in the reference to port anyway.

**Where the code goes**: `cartalith-civ`, same crate, same conventions.
Needs `cartalith-rng`'s existing `mulberry32` — a cross-crate dependency
this crate hasn't needed before, check `cartalith-civ/Cargo.toml`.

**What shipped.** `civ_settle_name`/`civ_base_pop_for_kind`/`civ_name_rng`/
`civ_default_culture`/`name_and_populate_settlements` ported to
`cartalith-civ`. Confirmed `_civRng` is `mulberry32` under a different
seed-derivation wrapper by hand-proof (XOR/OR commutativity + `ToInt32`
idempotence), not assumed — reuses `cartalith-rng` directly. Found and
documented a genuine reference quirk (`state.seed` is dead code, always
`undefined`, so the civ-naming RNG seed is a hardcoded constant
independent of the world's actual terrain seed) rather than mistaking its
symptom (identical names across different worlds for same-rank
settlements) for a bug. Two real harness bugs caught and fixed before
trusting any extracted data: a 4-script-block miscount (a comment inside
block #2 itself contains the literal text `<script>` in prose, corrupting
a naive regex-based block counter), and a `.suit`-field mixup (milestone
8's `SettlementPlacement.suit` correctly carries the pre-snap seed score
through unchanged, but this milestone's first harness attempt re-sampled
the suitability field at the post-snap position instead — caught because
names matched but population didn't, narrowing the bug precisely before
any Rust code was touched). Both fixture cases golden-verified bit-exact
(names by string equality, population as exact `u32`) on the corrected
extraction. See `CHANGELOG.md`'s "Phase 2 milestone 9" entry for the full
account.

## Milestone 10 — territory/provinces

Owner decision recorded 2026-08-16, `DECISIONS.md` §7b — read that first,
it's the authoritative design record, this is only the implementation
scope. **This port's own design, not a port.** It was scoped on the premise
that the reference computes no territory. That premise was false —
`_civAutoPolity` exists (see the superseded milestone 9) — and the owner has
since ruled that this design stays the only mode (§7b, 2026-08-19). So it is
the standard on its own terms: judged by visual plausibility and unit tests
per §7a/§7b, not golden-verified against `_civAutoPolity`.

**Algorithm**: cost-distance Voronoi from capitals, weighted by capital
population.

1. For every capital (milestone 8's `capital_of` flags + milestone 9's
   population figures), run `roadDijkstra` (milestone 11) from that
   capital's cell over `buildTravelCost`'s cost field — same real
   terrain-cost distance the road network itself uses, not a fresh metric.
2. Effective distance = raw cost-distance ÷ `w(population)`, `w` a
   monotonic weight function (e.g. `1 + ln(1 + pop/pop_ref)` — pick and
   document a real constant, don't leave it magic) so a more populous
   capital's territory reaches farther for the same terrain cost.
3. Each land cell's owner = the faction whose capital reaches it at the
   lowest effective distance. Water/unreachable cells (cost-distance
   `Infinity` from every capital) stay unowned, consistent with
   `buildTravelCost`'s water-impassable convention.
4. Multi-capital factions (milestone 8's multi-seat case): each capital
   projects its own influence independently; a cell's faction is whichever
   *capital* wins, then mapped to that capital's faction id — a faction
   with two capitals effectively gets the union of both their zones.

**In scope**: the assignment algorithm above, *visually* verified (per §7b)
on real generated worlds — not golden-verified — at more than one seed/map
shape so a single lucky-looking result isn't mistaken for "it works."

**Out of scope**: `_civGenerateProvinces` (sub-partitioning owned
territory into per-settlement provinces — a real next step once territory
itself exists, not this milestone), the interactive paint tool (this port
has no painting UI, not planned as part of this milestone), economy,
culture, roads-as-borders refinement (real roads existing, milestone 11,
could later inform border smoothing — not needed for a first working
version).

**Depends on** milestone 11 (road network algorithm) — it needs
`buildTravelCost`/`roadDijkstra` real and tested, called as a single-source
Dijkstra once per capital.

**What shipped.** `assign_territory` reuses `road_dijkstra`/`build_travel_cost`
(milestone 11) directly, with no modification. Verified by 8 unit tests
standing in for a golden test — programmatic checks only. Map-overlay
rendering was deferred to a UI/UX pass, since it needed `cartalith-godot`
binding work outside this crate. `pop_ref=15000.0` is documented as
`civ_base_pop_for_kind(Capital)`'s own value, not picked arbitrarily. See
`CHANGELOG.md`'s "Phase 2 milestone 10" entry.

*Since then:* territory is drawn — `STATUS.md` row P2-10.

## Milestone 11 — road network algorithm

Investigated 2026-08-16, choosing between the remaining candidates:
`_civSeedVillages` (reference line 25164) reads `ways` (a road network)
via `_civRoadProximityQuery(ways, cell)`, load-bearing in its village
acceptance probability — genuinely blocked on roads existing first, not a
false blocker (though on a different road network than this milestone's;
see "What shipped"). Territory stays blocked on the owner decision (milestone
10). Roads themselves turn out to be reachable now: `buildTravelCost`
(reference line 3257), `roadDijkstra` (line 3275), and `buildRoadNetwork`
(line 3316) are **block-1, pure, no DOM dependency at all** — the
reference's own comment on `buildRoadNetwork` says so outright ("MST over
the designated places using cost-distance... Pure").

- **`buildTravelCost`** — small: slope² cost field, water cells
  impassable (`Infinity`). Trivial once field/sea are real (they are).
- **`roadDijkstra`** — single-source Dijkstra over an 8-neighbour cost
  grid, own hand-rolled binary min-heap (parallel-array, same shape as
  milestone 2's `buildWaterBodies` heap — a DIFFERENT heap instance, not
  reusable code, but a precedent for how this crate already ports this
  exact structure). Read the reference's own v0.70 comment carefully: a
  real, subtle precision bug it already fixed once (`Float64` push
  priorities vs. `Float32` `dist` array causing values that compare-less
  but round-equal, an infinite-repush hazard) — the fix (a `visited` guard
  finalizing each cell on first pop) is part of the specification now, not
  optional cleanup; port it as written, don't "simplify" it back to the
  pre-fix shape.
- **`buildRoadNetwork`** — Prim's MST over the settlement set, using each
  settlement's own full-grid Dijkstra distances (one `roadDijkstra` call
  per settlement) as edge weights; reconstructs the actual cell-path for
  each MST edge via `prev` backtracking.

**Caller-agnostic on purpose**: this milestone ports the algorithm only,
not `buildRoadsOp()` (which reads `state.places` — user-clicked map
markers, a distinct manual-placement tool, not the civ auto-populate
settlements milestones 8-9 built) and not whatever step (if any)
`_civIterativeAutoWorld`'s own flow uses to connect its auto-placed
settlements — that wiring is a separate, later step, investigate it before
assuming its shape once this milestone's algorithm is real and tested.

**Out of scope for this milestone**: the Journey Planner itself
(`jpJourneyCost` and everything around it — `ROADMAP.md` already calls
this its own large sub-phase), territory/provinces (milestone 10).

**Where the code goes**: decided, not defaulted — landed in `cartalith-civ`.
`buildRoadNetwork` lives in block 1 of the reference (well before the civ
block), a real signal, but weighed against `ARCHITECTURE.md`'s own text
("later subsystems (civ, urban morphology, assets) arrive as new crates..."
naming `cartalith-civ` for this phase) and this crate's existing zero-
`gdext`/`WorldState`-read-only shape (a new crate would duplicate it for
no benefit) — `cartalith-civ` wins.

**What shipped.** All three functions ported, golden-verified (cost field `1e-4`,
edge topology bit-exact), both fixture cases, first attempt. A real
distinct-precision-regime heap needed (not reusable from milestone 2's
`MinHeap` — `roadDijkstra`'s own heap is `f64`-priority per the
reference's own v1.89 comment, a genuinely different regime, not a style
choice). Real terrain data exercised the "unreachable landmass" MST branch,
not just a synthetic unit test.

**This milestone did not unblock village seeding, though its first draft
expected it to.** Investigating for milestone 12 found that `buildRoadNetwork`
only ever serves the *manual* "Generate Roads" tool (`buildRoadsOp`, which
reads user-clicked `state.places`). The civ auto-populate flow's own road
system — `civWays`, auto-generated per the reference's own line-14758 comment
— is a separate, larger algorithm: `_civHierarchicalNetwork` (land routes,
milestone 12) plus `_civMstRoutes` (sea routes, port to port, milestone 13).
`_civSeedVillages`'s `ways` parameter is `civWays`, not `buildRoadNetwork`'s
output. (`_civPreferSeaRoutes`, first assumed part of that system, belongs only
to the manual "Auto routes" tool — milestone 12.) See `CHANGELOG.md`'s "Phase 2
milestone 11" entry for the full account.

## Milestone 12 — civ auto-populate road network: `_civHierarchicalNetwork`

Investigated further 2026-08-16: confirmed substantially larger than
milestone 11's `buildRoadNetwork`, not a same-shape sibling. Real
dependency graph (reference v2.10 line numbers, re-checked 2026-09-23):

- `_civHierarchicalNetwork` (~21526) — the entry point. Two-pass: **Pass
  1** builds a Prim MST over a no-reuse cost grid (same MST shape
  milestone 11 already has real, tested code for — check whether it's
  reusable here or needs its own copy given the surrounding differences).
  **Pass 2** re-runs Dijkstra over a *reuse-discounted* cost grid (roads
  already used are cheaper to extend) and fills every settlement up to a
  **minimum degree by tier** (`capital/metropolis:5, city:4, town:3,
  village:2, hamlet:1`) with shortcut edges the MST alone doesn't provide
  — a real, deliberate second structural pass, not a refinement of pass 1.
- `_civEnhancedTravelCost` (~20958) — a richer cost model than milestone
  11's `buildTravelCost`, taking a `usageCount` raster (nullable — null on
  pass 1, real on pass 2, which is *how* the reuse-discount works).
- `_civRoutingGrid` (~21022) — a downsampled routing grid distinct from
  full resolution (`RW`/`RH`/`sc` — check what resolution/why, likely the
  same "pathfind on a downsampled grid, road precision doesn't need full
  res" reasoning `buildRoadsOp` already used at milestone 11's own call
  site).
- `_civApplySettlementGravity` (~21119) — soft-attracts routes through
  intermediate settlements near a corridor (so A→C routes via B when B is
  close to the line), applied on *both* passes.
- `_civMstRoutes` (~21240), `_civPreferSeaRoutes` (~21389) — the sea-lane
  side (`civWays`'s own comment: "auto-generated road/**sea** network").
  `_civHierarchicalNetwork` itself is LAND-ONLY (`sea:false on every emitted
  way`, per its own v1.99 comment), so sea routing is layered on by one of
  these two with its own water-crossing cost model — which made sea lanes a
  natural sub-split. "What shipped" below records which one production
  actually calls; the answer is milestone 13.
- `opts.existingWays` (v1.64) — lets manually-drawn roads discount the
  auto-network's cost near them so it converges onto rather than
  duplicates manual work. Real but **optional/additive** (absent input ⇒
  unchanged behaviour), and the production call site
  (`_civIterativeAutoWorld`) passes empty opts, so auto-populate never
  reaches it. Not ported (`civ_hierarchical_network_topology`'s doc). The
  port has since gained a manual way tool (`tools::civ_commit_way`), which
  does not change that call site.

**What shipped.** Read `_civMstRoutes`/`_civPreferSeaRoutes` fully as instructed —
confirmed the real production call site (`_civIterativeAutoWorld`) never
calls `_civPreferSeaRoutes` at all (only the separate, manual-tool-adjacent
`_civAutoRoutes` does) and appends `_civMstRoutes(ports,true)` sea routes
directly via `ways.push(...)`. Also confirmed `_civHierarchicalNetwork`
itself has THREE passes, not two (a Floyd-Warshall shortcut-detour-relief
pass beyond MST + min-degree-fill), plus a substantial corridor-
consolidation/Catmull-Rom-smoothing/road-class-emission step. Split
accordingly: ported the raw three-pass topology
(`civ_hierarchical_network_topology`, `cartalith-civ`) — golden-verified,
both fixture cases exercising real edge conditions (an unreachable
settlement; the min-degree-fill pass hitting its natural ceiling rather
than its target). Corridor consolidation/smoothing (which needed
`_civSmoothPath`, then unported) was deferred to milestone 14 below. See
`CHANGELOG.md`'s "Phase 2 milestone 12" entry for the full record, including a real
`river_flow_thresh` parameter bug (hardcoded map width) caught before it
shipped.

## Milestone 13 — sea routes: `_civMstRoutes`

Confirmed genuinely separate from milestone 12's land network, not a
same-shape sibling, by reading the reference directly: cost grids mark
land `Infinity` (not merely expensive — the reference's own fix-history
comment explains a finite land cost let paths cut across jagged
downsampled coastline pixels, which smoothing then exaggerated into
visible loops), ports snap to the nearest navigable-ocean cell at radius
10 (wider than milestone 12/14's radius 6 on a different cost grid — a
real reference difference, not a typo "fixed" into consistency), and a
v0.73 sea-lane augmentation pass adds each port's single nearest
sea-reachable port as a direct lane (capped at 1.15× the MST's own
longest hop) beyond the bare Prim's-MST tree. Confirmed called only with
`isSea=true` at the real production call site
(`_civIterativeAutoWorld`, reference line ~25680: pushed unconditionally
onto `civWays` whenever `ports.length>=2`, NOT gated behind
`_civAutoRoutes`'s land-vs-sea cost comparison — that belongs to a
separate manual "Auto routes" tool, confirmed out of scope by reading
`_civAutoRoutes` itself). The `isSea=false` land-route branch has no
confirmed real caller and is not ported.

`_civSmoothPath` (real, ported as `civ_smooth_path` in
`cartalith-civ` — milestone 14) is reused as-is via a new `is_sea`
parameter threaded through it and three sibling helpers
(`civ_snap_finite`, `civ_is_valid_land`→`civ_is_valid_terrain`,
`civ_nearest_valid_pt`), generalizing them to both land and ocean
validity modes (`_civTerrainValidTest('land'|'ocean')`) rather than
duplicating them — a surgical parameter addition on each, all four
existing land-only call sites updated to pass their previous fixed
values explicitly.

**`_civSeaTimeEdgeCost` (current/wind-costed sea-lane pricing)
deliberately NOT ported.** Read in full: its real inputs are the
ocean-current and wind u/v vector fields, both computed internally
(`apply_ocean_currents`/`deflect_flow`, already golden-verified
elsewhere) but never retained on `WorldState` past that internal
use — only the resulting SST/rainfall corrections survive. The
reference's own code degrades gracefully when these fields are
unavailable (`if(!oceanF&&!windF) return null` → caller falls back to
`roadDijkstra`'s default uniform arithmetic-cost step), so this port
takes that same documented fallback. **Real flagged follow-up**:
wind/current-aware sea-lane costing, blocked on adding `WorldState`
retention for the ocean-current/wind fields (out of this milestone's own
scope, not silently dropped).

*Since then (2026-09-01):* that follow-up is built — `_civSeaTimeEdgeCost` is
ported as `civ_sea_time_edge_cost`, reached through the retained ocean-current
and wind fields. `STATUS.md` rows P2-13 and JP-QC3.

Shipped as `civ_sea_routes` (+ `SeaRoute` struct) in `cartalith-civ`,
golden-verified against two real cases reusing milestone 14's own
already-verified case0/case1 fixtures (genuine mixed land/ocean/lake
geography at both grids, not degenerate all-one-class) in
`tests/golden_parity_sea_routes.rs` — matched the reference's real
output exactly on the first run, including a genuine `_civSmoothPath`
rounding quirk (two of case1's four routes carry `km:0` despite real
points — `km` accumulates over rounded sample points *before* the
function's own final endpoint-precision-restore step). Full record,
including a real harness bug caught before trusting extraction
(`generate()` is `async`; a bare unawaited call silently left `field` at
its default-zero fill), in `CHANGELOG.md`'s "Phase 2 milestone 13"
entry.

**Rendering was left to a later UI/UX pass**, and this milestone recorded the
model to follow: `_civIterativeAutoWorld`'s real merge (`ways.push(...)`
alongside land ways). Sea routes are `Way`-shaped enough
(`pts`/`brks`/`km`/`name`) to reuse the rendering path milestone 14's UI/UX
catch-up built for land roads, given a `sea: true` (or equivalent) flag to
distinguish styling if desired. *Since then:* sea routes are drawn —
`STATUS.md` row P2-13.

## Milestone 14 — corridor consolidation + path smoothing

Deferred from milestone 12 (reference lines ~21670-21739): turns raw MST-
family edges into deduplicated, Catmull-Rom-smoothed, classified
(`highway`/`regional`/`road`/`track`), auto-named polylines for rendering.
Needed `_civSmoothPath` (also needed by milestone 13's sea routes — ported
once, shared, see milestone 13's note above) and `_civTerrainValidTest`
(ported narrowed to this network's one real call shape, `'land'` mode
only; milestone 13 generalized in the `'ocean'` mode). Not required for
`_civSeedVillages` to function (it needs road-proximity distance, which raw
unsmoothed edges already provide), but required for anything that actually
*draws* roads on the map.

Shipped as `civ_consolidate_and_smooth_ways` in `cartalith-civ`, golden-
verified against two real cases (reusing milestone 12's and milestone 9's
own already-verified fixtures) in
`tests/golden_parity_road_consolidation.rs`. Full record — including a
small line-range correction to a previously-documented reference-HTML
script-block convention, and a genuine short-segment Catmull-Rom
oversampling quirk traced and confirmed by hand — in `CHANGELOG.md`'s
"Phase 2 milestone 14" entry.

## Milestone 15 — village seeding: `_civSeedVillages`

Confirmed reachable now, independent of milestones 13/14 (per milestone
12's own note: `_civSeedVillages` needs road-proximity *distance*, which
raw unsmoothed MST-family edges already provide — smoothing/classification
is a rendering concern, not a functional one). Reference line 25164. Read
fresh against milestone 12's real topology, not against an earlier reading
made before that topology existed.

**Algorithm**: a Bishop-Fisher-style spatial hash grid rejects candidates
too close to any existing settlement (`spacing` from `VILLAGE_SPACING_KM`),
scans `findSettlementSeeds` at a *relaxed* threshold (`VILLAGE_SUIT_THRESH`,
lower than the main settlement threshold — dense-mode-style full-map
coverage), and for each candidate computes a soft accept probability
blending suitability with road proximity (`_civRoadProximityQuery`, line
25127) via `_civVillageAcceptProb` (line 25159). Nearest existing
settlement's faction is inherited. Named via milestone 9's
`civ_settle_name`/RNG (same shared stream discipline milestone 9
established). Capped at `_CIV_VILLAGE_CAP` = 200 (line 6445).

**In scope**: `_civSeedVillages` itself, `_civVillageAcceptProb`,
`_civRoadProximityQuery` unless milestone 12 already built an equivalent
(check first, don't duplicate), the spatial-hash rejection grid.
Golden-verify against the real reference engine — this one DOES have a JS
reference to check (unlike territory, milestone 10) since it's a real
reference function, not new design.

**Missed at scoping, and part of this feature:** `_civConnectVillageAddons`
(reference line 25248), the dirt track the reference draws from every addon
village to the network, called unconditionally after each village-seeding
pass. Nothing here named it, and it was found only from the owner's report
that villages had no roads. `STATUS.md` row P2-15 records its port, and
`MISTAKES.md`'s "Report that a feature does not exist" row records how
checking one function stood in for checking the capability.

**Out of scope**: milestones 13/14's own scope (sea routes, consolidation/
smoothing/road classification/rendering), economy, culture beyond naming
(already real, milestone 9), and the reference's `_civVillages` UI toggle —
this port had no such toggle when this was scoped. *Since then* New World ▸
Generation carries one (`new_world_dialog.gd`'s `villages_check`; `STATUS.md`
row P2-15).

**Where the code goes**: `cartalith-civ`, same crate, same conventions.

**What shipped.** `civ_seed_villages`/`civ_village_accept_prob`/`RoadProximityIndex`
(the milestone-12-topology adaptation of `_civRoadProximityQuery`) ported.
Closed a real RNG-sharing gap first: added
`name_and_populate_settlements_with_rng` (milestone 9, purely additive) so
village seeding can continue the exact stream naming left off at, matching
the reference's one-shared-`rng`-closure design. Golden-verified against
the real reference engine, bit-exact first attempt (fully synthetic but
reference-function-verified inputs, matching milestone 12's own
established standard) — see `CHANGELOG.md`'s "Phase 2 milestone 15" entry
for the full account, including a real threshold-consistency question
flagged (not fixed here) for whoever next touches `cartalith-godot`'s
orchestration, and the UI-toggle decision left to that same crate.

## Milestone 16 — provinces: `_civGenerateProvinces`

Resolves the blocker the superseded milestone-9 investigation (above)
reported. `civTerritory`, this function's real input, has one producer in this
port: milestone 10's `assign_territory`. The reference's own producer,
`_civAutoPolity`, was missed by that investigation and stays unported by
ruling (`DECISIONS.md` §7b). `assign_territory` was built for a different
reason — the port needed *a* territory system — and turned out to produce the
identical per-cell shape (`Vec<i32>` faction id, `0` = unowned). Confirmed by
reading the real reference source directly before porting, not by re-trusting
the earlier note's own summary.

`civ_generate_provinces(settlements, territory, gw, gh) -> (Vec<i32>,
Vec<Province>)` (`cartalith-civ`): a settlement-seeded Voronoi partition of
each faction's own owned cells, restricted to same-faction seeds (never
crosses a territory boundary). Seeds are every `Capital`/`City`-tier
settlement of a faction (this port's own `SettlementKind` reduces the
reference's rank>=3 filter — city=3/capital=4/metropolis=5/university=3/
industrial=3 — cleanly to a tier test, since university/industrial were
never ported into `SettlementKind` in the first place; not an
approximation, the same filter with tiers this port doesn't have removed
from the input domain. **Amended 2026-08-20**: the filter used to read
"Capital or City" because metropolis was unported too; with milestone 21's
`civ_select_metropolises` it now reads "Metropolis, Capital or City", and a
promoted imperial seat correctly seeds a province). A faction with no city-tier seed falls
back to its single highest-population settlement. A faction that owns
territory but placed zero settlements gets no province (cells stay `0`,
matching the reference's own behaviour).

Not golden-verified. The pass reasoned there was no reference run to compare
against, as for territory (§7b). That reason is weaker here than it looks:
`_civGenerateProvinces` is itself a reference function, and milestone 20's
golden harness already drives the reference with a synthetic territory raster
injected into its context, so a golden for this step is possible. Verified by
5 real unit tests instead: multi-seed Voronoi split, single-fallback-seed
case, a province never claims a cell outside its own faction's territory, a
faction with territory but no settlements stays unassigned, and every
reachable owned cell partitions into some real province (no gaps).

Wired into `cartalith-godot`'s `compute_civilisation()`/`CivData`
(`provinces: Vec<i32>`, `province_list: Vec<Province>`), with two new
`#[func]` methods: `get_provinces()` (metadata: id/faction/name/seed
settlement index) and `build_province_boundary_texture()` (a boundary-line
RGBA overlay — deliberately lines, not a per-province fill colour, since
province count isn't bounded the way `CIV_FACTION_COUNT` is and a real
per-province palette is a UI/UX design decision, not a data-porting one).
**This milestone deliberately did not wire it into `main.gd`/`map_overlay.gd`**
— no new UI toggle, no new `TextureRect` — per this port's own standing
practice of
routing new-visual-feature UI/UX through a dedicated pass rather than
improvising scene-tree changes inside a data-porting task. Both new methods
verified with real generated data via a temporary headless GDScript
(`generate()` → `get_provinces()`/`build_province_boundary_texture()`,
not committed): 7 provinces at seed 12345/512²/Classic, a real non-empty
512×512 boundary texture (2,262 boundary pixels), no crash — the same
real-invocation discipline that caught the earlier sea-routes crash,
applied here even without a permanent UI to screenshot.

**Verified**: `cargo test -p cartalith-civ` (5 new tests, 64 total, 0
failed), `cargo test --workspace` (0 regressions), `cargo clippy -p
cartalith-civ -p cartalith-godot --all-targets` clean, `godot4 --headless
--quit main.tscn` clean load, plus the real headless functional check
above.

**Out of scope for this milestone**: any actual rendering/UI wiring (the
follow-up this section itself flags), economy, culture beyond naming.

*Since then:* provinces are drawn — `viewport_host.gd`'s `province_view` —
and read by the world-data window and place search. `STATUS.md` row P2-16.

## Milestone 17 — economy investigated, first slice ported

Full investigation and reasoning now lives in `ECONOMY_SCOPE.md` (repo
root), not repeated here — this entry is the pointer. Summary: "economy" and
"Journey Planner" turned out to be two separate, both genuinely large,
subsystems (confirmed by reading the real reference, not assumed from a
scope-doc one-liner — the same correction this document's own milestone 9
note already had to make once for territory). The Journey Planner
(~70 `jp*`/`_jp*` functions) confirms `ROADMAP.md`'s own "consider it a
sub-phase" warning as accurate, not overcautious — not attempted. The
faction/settlement economy layer (`_civFactionAggregates`,
`_civPlaceTrade` and its dependency cluster, ~20 functions) is large but
bounded; `civ_resource_trade_balance` (the one fully self-contained piece,
`_civResourceTradeBalance` reference line 24175) was ported, tested and
verified in `cartalith-civ` ahead of any caller.

It surfaced one real tension, resolved the same day (2026-08-17). The full
trade layer needs all 15 `CIV_RESOURCE_KEYS` resident — both
`_civFactionAggregates` and `_civPlaceResourceContext` genuinely read every
key — but the memory-optimization pass (`MEMORY_OPTIMIZATION_SCOPE.md`) freed
6 of them after use. The fix was `_civPlaceTrade`'s own settlement-catchment
approach, which, unlike `_civFactionAggregates`' per-faction one, needs no
territory. That gave `civ_world_mean_resources`/`civ_catchment_km2`/
`civ_catchment_radius_cells`/`civ_place_resource_context` (8 new tests) and
the `get_trade_balances()` `#[func]`. The full reasoning is
`ECONOMY_SCOPE.md`'s "Memory-optimization tension: resolved" section.

## Milestone 18 — culture beyond naming

Real investigation, not another unverified "not done" mention (matching the
discipline milestone 9's territory note and milestone 17's economy
investigation both already established). Grepped the reference for every
culture-related computation beyond the syllable/suffix naming tables already
ported (milestone 9).

**Finding**: `civFactionCulture`/`civFactionGovernment`/`civFactionReligion`/
ag-technology are confirmed genuinely UI-only categorical labels with zero
derived computation — the reference's own v1.57 comment (line 26309) says so
directly ("editing a faction's Government/Culture/Religion/Ag.-technology").
**But one real thing does exist beyond naming**: `_civCultureTerrainFit`
(reference line 23748, v1.55) — a small, pure function comparing a faction's
territory terrain-mix against what its culture is thematically associated
with (highland↔hills, desert↔arid, riverlands↔river, sylvan↔forest,
maritime↔coast), producing a match/typical/mismatch verdict relative to the
world mean. `common`/`imperial` (identity-flavored, not terrain-themed)
deliberately get no verdict, matching the reference's own "never fabricate a
verdict without a real basis" discipline.

Ported as `civ_culture_terrain_fit` (`cartalith-civ`), 7 real unit tests
covering every verdict band plus both zero-world-mean edge cases. It shipped
**ahead of its caller** — its real inputs (`terrain_mix`/`world_mean_terrain`,
per-faction terrain-type fractions) are `_civFactionAggregates`'s own v1.55
"Territory Fit" output, then a 165-line territory-based aggregation that
`ECONOMY_SCOPE.md` scoped separately — the same "ship the primitive ahead of
the orchestration" precedent as `civ_resource_trade_balance`.

**Update (2026-08-18)**: the GUI parity audit (`d84dfd0`) found this made
the function *unexposable*, not merely unwired, and correctly re-classified
it. **Milestone 20 below is the answer**: it ports `_civFactionAggregates`,
which makes both maps real and golden-verified, and its own golden test calls
`civ_culture_terrain_fit` straight off them.

**Also found and correctly ruled out of Phase 2's scope**: a completely
unrelated, much larger "culture" concept exists in the reference at lines
28193+: urban-morphology "culture profiles" — Organic Growth,
Islamic/Byzantine/Chinese/Aztec/Viking/etc. city-layout patterns. (The
reference's comment there cites the source project's
`docs/07-culture-architecture.md`, which is not vendored in this repository's
`docs/`.) This belongs to `ROADMAP.md` Phase 5 (Urban morphology, block 4) — a
different system entirely, not a Phase 2 gap.

**Culture beyond naming is fully accounted for within Phase 2's scope**: this
milestone is the one real computation, and the rest is confirmed either not to
exist (Government/Religion/Ag-tech) or to belong to a different phase entirely
(urban morphology). Nothing further is scoped here. "Not to exist" is a
statement about the *reference*. This port has since given all three a
consumer, as new scope beyond it and outside Phase 2: government and
ag-technology drive `MILITARY_MANPOWER_SCOPE.md`'s model, and religion drives
`RELIGION_DIFFUSION_SCOPE.md`'s.

## Milestone 19 — Journey Planner milestone 1: physical-modeling primitives + seasonal/closure logic

Full reasoning and the remaining milestone breakdown now live in
`JOURNEY_PLANNER_SCOPE.md` (repo root, new). Summary: ported the two fully
self-contained categories of the ~70-function Journey Planner that need no
route/plan/vessel context object — `jp_fatigue`/`jp_load_penalty`/
`jp_surface_gain`/`jp_can_use_wheels` (tiny physical-modeling primitives)
and `jp_season_at`/`jp_rest_days`/`jp_seasonal_closure`/`jp_sea_closure`
(the reference's own "v1.52: four deferred items" cluster — rest-day
scheduling, season drift over long journeys, mountain-pass and sea-lane
winter closures). 22 real unit tests. It shipped ahead of its callers: the
route/plan orchestration that consumes it is real, substantial, and scoped
separately as `JOURNEY_PLANNER_SCOPE.md`'s milestones 2-6. *Since then* it has
callers — `STATUS.md` row P2-19.

## Milestone 20 — `_civFactionAggregates`

Full reasoning stays in `ECONOMY_SCOPE.md` (repo root) — this entry is the
pointer. Summary: ported `civ_faction_aggregates` (reference line 23575,
v1.16 + v1.55) with `_civFactionCapital`, the `CIV_TAX_RATE`/
`CIV_PRIMARY_SPECIALISATION` tables and `_civOceanDistField`, closing item 4
of `ECONOMY_SCOPE.md`'s "next milestones" list and with it the faction-level
half of the economy layer.

**Why now**: it is a real blocker for something already built. Milestone 18
shipped `civ_culture_terrain_fit` deliberately ahead of its caller; the GUI
parity audit (`d84dfd0`) then found it **cannot be exposed at all**, because
its `terrain_mix`/`world_mean_terrain` inputs are `_civFactionAggregates`'s
own v1.55 "Territory Fit" output and nothing computed them. That is now
computed, and the golden test calls `civ_culture_terrain_fit` straight off
the aggregate output for all seven cultures × all seven factions in both
fixtures, matching the reference's own `_civCultureTerrainFit` over the same
aggregates. **Milestone 18's one open loop is closed.**

**The tension milestone 17 recorded does not bind here.** The half of
`_civFactionAggregates` that unblocks culture-terrain-fit needs no resource
field, and `resources` is an `Option` porting the reference's own nullable
`pots`, so `compute_civilisation()`'s six-field free stays where the
memory-optimization pass put it. `ECONOMY_SCOPE.md` records the one-line move
a caller wanting the resource means would make.

Golden-verified over two fixtures shaped to reach the edges, plus 15 unit
tests for what a golden cannot reach and a 58-mutation sweep (56 killed, 2
proved equivalent, four real fixture gaps found and closed) —
`ECONOMY_SCOPE.md`'s "Verification" under the 2026-08-18 pass has each. Like
milestones 18 and 19, it shipped ahead of any `#[func]` or GDScript caller.
*Since then* it has live callers — `STATUS.md` rows P2-20 and EC-4.

**This is the last of the faction-level economy scoped here.** The remaining
settlement-level functions — `_civPlaceSmelting`, `_civSaltAccess` and the
food-surplus cluster — are `ECONOMY_SCOPE.md`'s, not this document's, and
nothing in this milestone blocks them.

## Milestone 21 — the two deferred auto-populate passes: `_civSelectMetropolises` + `_civApplyRecovery`

Owner decision, 2026-08-20. Both were long-standing registered gaps —
`GUI_GAP_REGISTER.md` CV-04 and CV-08, each deferred by `TIMELINE_SCOPE.md`
§6 with `_civApplyRecovery`'s scoping explicitly assigned *to this document*
("its own scoping (if any) belongs to `PHASE2_SCOPE.md`, not here"). This is
that scoping, and the port.

**`_civSelectMetropolises`** (reference **24961-24989** — verified, not
assumed: the function ends on 24989, and the extraction harness's own
boundary assertion caught the off-by-one before a single fixture was
generated). Lawrence et al. 2016's rule as the reference states it: a
metropolis is a *capital* that is both a dominant trade hub (normalised
betweenness ≥ 0.85) and the seat of a large polity (its faction holds ≥ 6
settlements), ranked by centrality, ≤ 1 per faction, ≤ 3 in total, with a
deterministic x-then-y tie-break. Ported to `cartalith-civ` as
`civ_select_metropolises`, returning **indices** where the reference returns
a `Set` of place objects.

That required `SettlementKind::Metropolis`, which in turn required the
reference's real rank-5 entry in **every** per-tier table this port had
capped at Capital: `_CIV_CATCHMENT_KM2` 2500, `_CIV_SURPLUS_FRACTION` 0.10,
`_CIV_TRADE_K` 2.1, `_CIV_BASE_POP_BY_KIND` 45000, `CIV_TAX_RATE` 0.10,
`CIV_SETTLEMENT_CLASSES` rank 5, `minDeg` 5, `_CIV_TIER_FLOOR` 150000 and
`_CIV_TIER_ORDER`'s first slot. Three non-exhaustive tier *predicates* also
moved and the compiler could not find them: the faction-seat filter
(`kind==='capital'||kind==='metropolis'`), the province rank≥3 seed filter,
and `civ_is_exchange_tier`. They were found by grep against the reference's
own `metropolis` occurrences instead.

**`_civApplyRecovery`** (reference 24619-24640), the v0.82 static
post-collapse re-weighting behind auto-populate's "Recovery phase" dropdown.
Ported to `cartalith-civ/src/timeline.rs` beside the tier tables
`TIMELINE_SCOPE.md` §3 point 5 had already identified as shared. Three
details are load-bearing and each has its own fixture: `was_urban` includes
**Town** (wider than `civ_is_exchange_tier`); the RNG draw happens **before**
the abandonment test, so a dropped settlement still consumes one value; and
the `max(8, pop)` floor is applied **after** the tier decision.

**Wiring.** Both run exactly where the reference runs them inside
`_civIterativeAutoWorld` — metropolis promotion at line 25711 (after the
network is scored, before naming/population), recovery at line 25761 (after
the population pass) — behind `set_metropolis_enabled` /
`set_recovery_phase`, whose defaults are the reference's own (OFF / Stable),
so an untouched engine generates exactly what it generated before. The
metropolis pass needs betweenness that this port has never had a
`_civNetworkMetrics` for; only the *ratio* `betweenness/max_btw` is read, so
the `(n-1)(n-2)` normalisation cancels and Brandes over `topology.edges`
(`timeline::civ_betweenness_from_adjacency`, already in the crate) is
bit-identical. A golden test pins that reasoning rather than leaving it as a
comment.

**Two disclosed limitations**, both consistent with boundaries this port
already records: the reference also pushes `trade_hub`/`administrative`
traits onto a promoted place and `ruins`/`fortified` onto a demoted one, and
`NamedSettlement` has no trait vector at all (`timeline_bridge.rs`'s own
top-of-file gap note). `kind` and `pop` — everything downstream reads —
survive intact; the trait writes are computed, golden-tested at the pure
function, and dropped at the Godot boundary.

Golden-verified against the real reference: 28 fixtures in
`golden_parity_metropolis_recovery.rs`, extracted by a transient Node
`vm.runInContext` harness over three verbatim slices, plus a **35-mutation
sweep with every mutant killed**. Two pre-existing golden tests changed —
both of them pinned this port's own documented *cap* rather than the
reference, and both were re-extracted rather than hand-edited. Surfaced in
`File ▸ New world ▸ Generation` as a checkbox and a five-entry dropdown, the
latter filled from the engine's own `_CIV_RECOVERY_NAME` table. This milestone
is the work `GUI_GAP_REGISTER.md` **CV-04** and **CV-08** name.

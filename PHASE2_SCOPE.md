# Phase 2 scope: milestones

`ROADMAP.md`'s one-paragraph sketch for Phase 2 is a phase-level sketch,
not a milestone boundary — the same relationship `MVP_SCOPE.md` had to
`ROADMAP.md`'s Phase 1 paragraph. Phase 1 itself shipped as many
separately-verified stages, not one pass; Phase 2 does the same. This
document is a living list of milestones, updated as each completes and the
next is scoped — mirroring `STATUS.md`'s own discipline, not a one-shot plan
written before any of it is known.

## Milestone 1 — affordance fields foundation: **done** (2026-08-16)

Lithology classification, soil fertility, water access
(`buildLithology`/`buildSoilFertility`/`buildWaterAccess`, reference lines
5835/5852/5866). New crate `cartalith-civ`, zero `gdext` dependency, golden-
verified (lithology bit-exact, soil/water at `1e-4`). See
`CHANGELOG.md`/`STATUS.md` for the full record — this document only tracks
scope, not history.

**Investigated before milestone 1 was written**: traced
`currentSettlementSuitability` (the "v1.30 one function" `ROADMAP.md`
flags) and found it several milestones away, not a starting point — it
depends on resources, carrying capacity, route corridors, landmass
quality, coast SDF, water-body classification, and river network order,
none of which existed. The reference's own v0.104 history drew the exact
boundary milestone 1 adopted: "this lands lithology → soil → water access.
Resources + carrying-capacity + settlement suitability are the v0.105–
v0.106 follow-ups."

## Milestone 2 — water-body classification: **done** (2026-08-16)

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

**Done.** `build_water_bodies` ported (connected-components flood fill +
priority-flood min-heap, index-for-index port of the reference's own
heap). Golden-verified bit-exact on classification, `1e-4` on fill-level,
both fixture cases. Found and root-caused a real Node-harness bug along
the way (the reference's `state` literal seeds `tect.seed` from
`Math.random()` at load — the harness had been setting an unrelated
top-level `state.seed`, leaving generation nondeterministic until fixed to
set `state.tect.seed`), not a fixture/tolerance issue. See
`CHANGELOG.md`'s "Phase 2 milestone 2" entry for the full record.

## Milestone 3 — biome classification: **done** (2026-08-16)

`classifyBiome` (pure temp/rain → category) + `buildBiomeRaster` (applies
it per-cell, with ocean/lake cells overridden from milestone 2's water-body
classification). Ported to `cartalith-civ`, bit-exact both cases, first
attempt. `buildCartBiome` (the *different*, denser 15-category Cartalith
editor-bridge biome-paint auto-fill) confirmed out of scope — no consumer
exists anywhere in this port (no painting UI, no editor integration) —
not implemented. See `CHANGELOG.md`'s "Phase 2 milestone 3" entry.

## Milestone 4 — carrying capacity, NPP, population density: **done** (2026-08-16)

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
  `buildWetlandMask` (reference line ~6839, small, not yet ported). All
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

**Done.** `build_carrying_capacity`/`build_npp`/`estimate_regional_density_km2`
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
needs the equivalent retention fix before it can start. See
`CHANGELOG.md`'s "Phase 2 milestone 4" entry for the full record.

## Milestone 5 — resource potentials: **done** (2026-08-16)

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

## Milestone 6 — settlement-suitability prerequisites: route corridors, landmass quality, coast SDF: **done** (2026-08-16)

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

## Milestone 7 — settlement suitability / seed-finding: **done** (2026-08-16)

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

## Milestone 8 — settlement placement + faction assignment (current)

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

## Done means (per milestone, not once for the whole phase)

Each milestone: golden-verified against the real reference engine with a
real, justified tolerance; `cargo test -p cartalith-civ` proves it with no
Godot involved; a `CHANGELOG.md` entry; `STATUS.md`'s Phase 2 row updated
to reflect real state, not "Phase 2 done" until it actually is. Nothing
outside a milestone's own explicit scope gets implemented in that
milestone's pass — flag and stop if something turns out unavoidable,
report it, don't silently expand.

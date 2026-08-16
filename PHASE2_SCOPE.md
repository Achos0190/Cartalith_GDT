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

## Milestone 4 — carrying capacity, NPP, population density (current)

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

## Milestone 5 — resource potentials (not yet started)

`buildResourcePotentials` (reference lines 6085–6193). Needs
`boundary_type`/`shear_field` retained on `WorldState` (verify/fix per
above), plus lithology (real, milestone 1), flow field (real), biome
(real, milestone 3), field/rain/age (real). `SUIT_RESOURCE_KEYS` (9 ore
types: copper/tin/iron/gold/salt/timber/lead/silver/gems — note block 2's
own `CIV_RESOURCE_KEYS` is a *different*, larger vocabulary per the
reference's own comment at line ~6293; this milestone ports only the
block-1 ore subset, not the full civ resource list). Read the full
function before scoping further — 108 lines may itself split into
sub-passes once its actual structure is clear.

## Milestone 6+ — not yet scoped

Settlement suitability/seed-finding (needs milestone 5 plus route
corridors, landmass quality, coast SDF — none built, check what those
actually require once reachable), then
factions/territory/provinces/economy, then roads (`ROADMAP.md` already
calls the Journey Planner its own sub-phase) — each scoped when reachable,
not speculatively now.

## Done means (per milestone, not once for the whole phase)

Each milestone: golden-verified against the real reference engine with a
real, justified tolerance; `cargo test -p cartalith-civ` proves it with no
Godot involved; a `CHANGELOG.md` entry; `STATUS.md`'s Phase 2 row updated
to reflect real state, not "Phase 2 done" until it actually is. Nothing
outside a milestone's own explicit scope gets implemented in that
milestone's pass — flag and stop if something turns out unavoidable,
report it, don't silently expand.

# Cartalith — the target the project is aiming at

Owner-supplied vision render (2026-08-17), shared with the words *"This is
where it needs to go."* This document records what that image actually shows,
what's genuinely new in it versus everything already scoped, and — honestly —
which parts are reachable soon versus which are a different product entirely.

It is a **direction**, not a specification. The wireframe handoff
(`design/cartalith-menu-structure.md`, `GUI_SHELL_SCOPE.md`) is the structural
spec; this is the thing that spec is trying to become.

## What the render shows

**Shell** — "CARTALITH · Advanced worldbuilding and simulation". Left rail
grouped WORLD (Terrain, Hydrology, Climate, Ecology, Resources) ·
CIVILISATION (Settlements, Population, Factions, Territories, Economy) ·
INFRASTRUCTURE (Roads, Ports, Industry, Fortifications). Right rail grouped
SCENARIO (Timeline, Journeys, Events, Conflicts, Objectives) · SIMULATION
(Climate, Hydrology, Growth, Logistics, Travel, Figures) · SCENARIO again
(Timeline, Events, Characters, Conflicts, Objectives). This maps closely onto
the wireframe's own navigator — already built, milestone 1 (`5d44c6b`).

**The layer stack, as physical sheets.** Above the map, eight translucent
plates fanned in depth — Geology · Hydrology · Climate · Ecology ·
Civilisation · Infrastructure · Logistics · Narrative/Scenario — each showing
its own rendering of the same world. Not a flat checkbox list: a *stack you
can see through and peel*. This is a real interaction metaphor, and a
genuinely different idea from the layer panel currently built.

**The map itself.** Hand-drawn atlas quality: fine hillshading, forest
stippling, river networks with real hierarchy, hand-lettered settlement
glyphs, a paper/vellum ground with a physical border. This is far beyond the
current biome/hillshade renderer — it's the target
`TERRAIN_APPEARANCE_SCOPE.md` is walking toward.

**Causal chains, drawn on the map.** Three annotated arrow-chains over the
terrain:

- `mountain range → watershed → river → fertile valley → settlements → road network → trade corridor → political importance`
- `mineral deposit → mine → industrial settlement → road → regional trade network`
- `mountain pass → restricted movement → strategic → fortification → military route`

**This is the most important thing in the image**, and the section below
explains why.

**Timeline** — bottom dock, multi-track: States · States · Events ·
Narrative, with labelled markers ("Ancient State", "Abandoned Roads", "Older
Fortifications", "Colonisation", "Events", "Conflicts"). The wireframe's
single scrub, extended into a real multi-track narrative history.

## The causal chains are not decoration — and that matters

Every other visual idea in this render is presentation. The causal chains are
different: **Cartalith already computes those exact relationships, as real
data, today.** They are not annotations someone would author by hand — they
can be *derived*.

Trace the first chain against what's already built and golden-verified:

| Chain link | Where it already exists |
|---|---|
| mountain range | `cartalith-terrain` — orogeny, `compute_height` |
| watershed | `cartalith-hydrology` — `compute_flow`, flow accumulation |
| river | `cartalith-hydrology` — river network extraction, Strahler order |
| fertile valley | `cartalith-civ` — `build_soil_fertility`, `build_npp`, `build_carrying_capacity` |
| settlements | `cartalith-civ` — `build_settlement_suitability` reads water access, soil, corridors, coast SDF |
| road network | `cartalith-civ` — `civ_hierarchical_network_topology`, real terrain-cost routing |
| trade corridor | `cartalith-civ` — `build_route_corridors`, `civ_resource_trade_balance` (just wired, `7228bb4`) |
| political importance | `cartalith-civ` — `assign_territory` (cost-distance from capitals, population-weighted) |

The second chain (`mineral deposit → mine → …`) runs through
`build_resource_potentials`' 15 real geological fields. The third
(`mountain pass → restricted movement → …`) runs through `build_travel_cost`
— the same cost surface the road Dijkstra already uses.

So the render is describing a feature this engine is *unusually* well-placed
to build, because it never faked any of that. `DECISIONS.md` §7's whole
golden-parity discipline exists precisely so these values mean something. A
"why is this settlement here?" explainer that walks real computed inputs is
a genuine, defensible product differentiator — and it is buildable now, not
after some future simulation rewrite.

It is also the honest expression of `TERRAIN_APPEARANCE_RESEARCH.md` §30's
own rule: *"the objective is to make the physical differences represented by
the world model visually legible."* Causality is the deepest form of that.

## Honest gap assessment

**Reachable now, on real data:**
- Causal-chain explanation ("why is this here?") — every input above is
  already computed and, where not yet exposed, needs only a `#[func]`.
- Layer-stack metaphor as a real UI treatment — pure Godot Control work.
- Atlas-quality rendering — `TERRAIN_APPEARANCE_SCOPE.md`'s own milestone
  2+, the largest single visual gap between today and this render.

**Real, but a substantial engine effort each:**
- Journeys / Logistics — `JOURNEY_PLANNER_SCOPE.md`, ~70 reference
  functions, 6 scoped milestones, 2 landed.
- Industry / Ports / Fortifications as first-class systems — Ports exist
  implicitly (sea routes need coastal settlements); Industry and
  Fortifications do not exist anywhere in the engine or the reference.

**A different product, not yet scoped anywhere:**
- Narrative/Scenario as a domain: Events, Characters, Conflicts,
  Objectives, and a multi-track narrative timeline. Nothing in the JS
  reference or this port has any of it. This is authored-content tooling
  layered over a generated world — closer to a scenario editor than a
  generator, and a genuine product-scope decision rather than a porting
  task.
- Year-by-year historical playback driving that timeline. The engine is a
  one-shot static generator by explicit design (`HARDWARE_ACCELERATION.md`'s
  static-generation correction; the owner's own "no need to continuously
  calculate"). The render's timeline implies real historical simulation —
  states rising and falling, roads being abandoned, colonisation waves.
  That is not a UI gap; it is a fundamentally different engine.

**The "States / Abandoned Roads / Older Fortifications / Colonisation"
markers are the clearest statement of that last point.** They describe a
world with *history*, not a world with a *state*. Reaching that means
deciding whether Cartalith stays a static generator with narrative
annotation layered on top, or becomes a temporal simulation. That is the
owner's call, and it is the single biggest open question this render raises.

## Sequencing that follows from the above

1. ~~**Causal-chain explainer**~~ — **done 2026-08-17.** Hovering a
   settlement now shows a real "WHY HERE?" chain in the Inspector,
   decomposed from `build_settlement_suitability`'s own thirteen weighted
   terms, sorted most-decisive-first, with penalties shown as penalties.
   Proved faithful rather than plausible: a test reconstructs the real
   function's output at every cell of a field from the explanation alone,
   and a headless pass over all 40 settlements of a real world confirmed
   the coastal/river terms track the actual terrain (0 violations).

   Two honest corrections to this document's own assessment came out of it:

   - The section above said the inputs "need only a `#[func]`". Not quite:
     they are locals of `compute_civilisation` and are **not retained** on
     `CivData`, so a general per-cell `explain_cell(x, y)` would have meant
     holding ~12 full-grid rasters (hundreds of MB at 2048²) — against
     `MEMORY_OPTIMIZATION_SCOPE.md`'s measured work. Shipped per-settlement
     instead, computed while the rasters are alive. That covers the question
     the render actually poses and costs ~nothing.
   - The render's *other two* chains (`mineral deposit → mine → industrial
     settlement`, `mountain pass → … → fortification`) terminate in systems
     that do not exist anywhere in this engine or the reference — Industry
     and Fortifications, as this document's own gap assessment already
     notes. Their upstream halves are real (`build_resource_potentials`,
     `build_travel_cost`, both surfaced in the explainer as `minerals` and
     `travel_cost`); their downstream halves are unbuilt, and were not
     invented to complete the picture.
2. **Atlas rendering** (`TERRAIN_APPEARANCE_SCOPE.md` m2+) — closes the
   largest purely-visual gap.
3. **Layer-stack treatment** — real polish on an already-built panel.
4. **Journey Planner** milestones 3-6 — already scoped and underway.
5. **Narrative/Scenario, and the static-vs-temporal question** — needs an
   explicit product decision before any code.

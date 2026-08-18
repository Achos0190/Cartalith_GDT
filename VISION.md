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
   largest purely-visual gap. **Milestone 2 done 2026-08-17**: the relief
   itself now reads — multidirectional hillshade (6 weighted lights, the
   primary NW sun still dominant) plus heightfield ambient occlusion, so
   drainage networks, ridges, valley floors and coastal escarpments are
   legible where the single-sun render washed them into a flat tan blur.
   Measured against §30's anti-list rather than eyeballed: the darkest
   pixel is *identical* before and after in both test worlds (no black
   valleys — AO only darkens concavities and is floored), and mean luma
   moves just 133.3→128.8, so it redistributes contrast instead of
   dimming. A 3× zoom caught one real regression mid-pass (the fine AO
   radius resolved to 1 cell and read as speckle — "random texture
   noise", also on the anti-list) which was fixed before landing.
   **Milestone 3 done 2026-08-17**: a subtle cool/dark tint near real river
   flow (`hydro_wet_strength`), reusing the same `flow` field already
   powering the settlement suitability explainer above — hydrology now
   shows up twice in this codebase, once as data (item 1) and once as
   atmosphere (this item). The first parameter guess passed every
   mechanical check but was visually undetectable in a real crop (0.4% of
   pixels, barely above the JPEG-noise floor) — caught by actually looking,
   not by trusting the diff numbers — and was strengthened until a crop
   centred on the real maximum-difference pixel showed genuine, still-
   subtle dampness along a valley floor. Honest across terrain types, same
   shape as milestone 2's own finding: real on the mountainous Classic
   world, nearly invisible on the low-relief Archipelago one, because
   there's simply less major drainage there to find.
   **Milestone 4 done 2026-08-17**: three of the four atlas elements this
   item used to list as "still ahead" — the paper/vellum ground, forest
   stippling and the physical plate border. The paper is applied to the
   whole sheet, ocean included, as a luminance-neutral parchment tint plus
   a luminance-preserving chroma muting, so the map reads as pigment on a
   sheet rather than as emitted colour, at zero cost in relief or biome
   legibility (interior mean luma moves 132.8→133.0 on Classic and
   106.3→106.2 on Archipelago, and contrast *rises* slightly in both).
   Stippling is driven by `material_weights`' own `canopy` fraction — real
   data, not decorative noise. The border is a bare-paper margin carrying a
   thick and a thin neatline.
   Two corrections came out of actually looking, again: the parchment tint
   alone was a pure hue rotation and read too weakly until the chroma wash
   was added, and the first stipple field read as a regular halftone screen
   until its sampling lattice was rotated and domain-warped (the same class
   of regression as milestone 2's AO speckle, found the same way — a 6×
   crop, not a diff statistic). And the cross-world result inverts what
   milestones 2 and 3 both found: this one is *stronger* on the low-relief
   Archipelago (−26% chroma, its bright cyan sea becoming a muted
   teal-grey) than on mountainous Classic (−13%), because the paper acts on
   the whole sheet and that world is mostly ocean.
   **Milestone 5 done 2026-08-18**: geology and separation. The world's real
   rock — `cartalith_civ::build_lithology`'s seven types, built from the
   *tectonic* substrate rather than from anything the renderer could already
   see — now reaches the image, both as the rock material's own colour and
   as bedrock showing through thin soil on steep, unvegetated ground. It
   matters more than it sounds: over Classic's land that vocabulary is 45%
   shale, 33% metamorphic, 11% basalt, and just **0.4% granite** — and
   granite is what the ported climate heuristic painted by default, so the
   map had been showing one rock for a world that has seven. Alongside it,
   local contrast (a band-passed luminance detail boost whose gain *falls to
   zero* on strong edges, so §18's "no haloing" is a property of the maths
   rather than of the tuning) raises interior contrast in all three test
   worlds — luma sd 31.9→32.9 on Classic, 27.3→28.8 on a non-square plate —
   while clipping *falls* and chroma is untouched.
   The looking-not-trusting lesson held a third time, twice over. The
   geology gate was first written in raw slope units, and raw slope is
   resolution-dependent — median land slope is 6.6× smaller at 2048² than at
   512², so the stage silently confined itself to the steepest ~5% of land
   at the resolution the app actually runs at; normalizing to `slope * gw`
   (this project's own convention) took the affected pixels from 1.2% to
   6.6%. And local contrast as a plain high-pass amplified milestone 4's own
   paper grain into a faint rectangular quilting — the same failure class as
   the AO speckle and the halftone stipple, found the same way, and fixed by
   band-passing so the sheet's texture passes through untouched.
   **Milestone 6 done 2026-08-18**: the GPU question, answered by measuring
   rather than by building. GPU compute is genuinely reachable — through the
   standalone `wgpu` instance `cartalith-gpu` already owns, not through
   Godot's own renderer — and a 2048² noise kernel runs there in 2.8 ms
   against 36.8 ms of single-thread CPU. But the renderer turned out not to
   be GPU-bound at all: after five milestones of appearance work its
   per-pixel loop had grown to ~1 s at 2048² **on one core**, the last
   O(gw·gh) serial loop left in a workspace whose engine crates have been
   Rayon-parallel for days. Parallelizing it took `cell_color` from 1040 to
   125 ms (8.3×) and the real app's whole `build_color_texture` from 955 to
   293 ms (3.3×) at the app's own 2048×1311 — bit-identical output, proven by
   re-diffing all 48 A/B dumps byte-for-byte. That also *settled* the GPU
   question for now: the appearance pass is 5% of the time to a new world,
   down from 15%, so a WGSL port of the whole material-synthesis kernel would
   buy about 5% in exchange for a second renderer permanently diverging from
   the golden-verified one. Recorded, with `apply_local_contrast` named as
   the natural beachhead if it is ever picked up.
   Alongside it, quality tiers — Performance/Balanced/Quality/Ultra, offered
   to Godot but **never applied automatically**, because what a phone should
   default to is the owner's call. Their design is the milestone's other real
   finding: a measured cost table shows that dropping five of the six light
   directions and switching off ambient occlusion — precisely what the
   research doc's own Performance recipe prescribes — saves *nothing
   measurable* in this renderer, while local contrast alone costs 30-53 ms.
   So the cheap tier keeps the relief that makes the map legible and gives up
   texture instead, and it is 2.2-3.3× cheaper for it. `Quality` is the
   milestone-5 look returned unchanged, byte for byte.
   Still ahead for the atlas look proper: hand-lettered settlement glyphs —
   which are drawn by `map_overlay.gd`, not by `render.rs`, so they are a
   GDScript overlay task rather than a renderer one. (The plate-margin
   overlay defect this item used to record was fixed in milestone 4's own
   follow-up.) Beyond that the remaining research phases are colour vibrancy,
   atmospheric distance effects, the high-precision display pipeline and the
   GPU rendering path itself — plus one pre-existing artifact milestone 6
   found by looking and deliberately did not fix: rectangular blockiness in
   the open ocean, inherited from the reference HTML's own low-frequency
   sea-colour noise lattice and *more* visible there than here.
3. **Layer-stack treatment** — real polish on an already-built panel.
4. **Journey Planner** milestones 3-6 — already scoped and underway.
5. **Narrative/Scenario, and the static-vs-temporal question** — needs an
   explicit product decision before any code.

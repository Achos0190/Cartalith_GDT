# Military manpower — what a polity can put and keep under arms

> **This document carries the specification, the derivation and the record of
> what was measured; it does not track progress.** MM-1…MM-8 and the four
> findings of §3.3 are defined here and **tracked only in
> `cartalith-native/docs/STATUS.md`**.

Written alongside the 2026-08-25 build pass, against an owner-supplied
specification. This document exists because that specification is **design
input worth preserving verbatim**: the model has no reference implementation to
fall back on, so the specification is the only ground truth there is, and a
paraphrase of it would leave nothing to check the code against.

Sections, in this order:

- **§0 · How this relates to CV-25** — what it supersedes and what it leaves alone.
- **§1 · The owner's specification**, reproduced as supplied.
- **§1a · The owner's ruling on it** — a decision made about the specification
  after the fact, kept in its own section so the verbatim text above is never
  edited: which population the era table's percentages are a share of. Later
  rulings on the model (owner ruling 11, owner ruling AI) are recorded where
  they act, in §2 and §3.3.
- **§2 · The derivation** — how each part of it maps onto quantities this port
  already computes, and every constant with its grounding.
- **§3 · Verification** — the two worked examples, the live figures, and the
  four findings the build produced, including the two where the specification
  is internally inconsistent.
- **§4 · What this deliberately does not build** — reopened by Ruling AW.
- **§5 · Campaigns over time (Ruling AW)** — the CARTO ▸ Conflict layer: what
  a siege line, a front and "territory over time" are drawn from.

---

## 0 · How this relates to CV-25

`GUI_GAP_REGISTER.md` §40 built CV-25 as a *minimal military model*: three
reference ports (`_umWallSpec`, `_umInferWalls`, `_civPlaceDefensibility`)
plus wiring `_civFactionAggregates`' `power.military` axis, which had been fed
a hardcoded `fortified: false` and was silently computing zero.

**This supersedes the manpower half of that and leaves the rest alone.**

- **Fortification stays.** Walls and defensibility are a *separate axis* —
  how hard a place is to take, not how many people a polity can raise. Both
  are ports and both are golden-verified. `cartalith_civ::military` is
  untouched by this pass.
- **`power.military` stays as it is**, and this is a deliberate decision
  rather than an omission. It is a golden-verified port of the reference's own
  `0.45·normPop + 0.35·fortifiedFraction + 0.20·capitalTierNorm`. Rewriting it
  to derive from a model the reference does not have would break that parity
  to gain nothing the headcounts do not already say better, and the two answer
  genuinely different questions: `power.military` is *this faction against the
  others on this map* (relative, 0-100), the manpower model is *how many
  people* (absolute). They are reported side by side, and the shell labels
  which is which. Recorded in `civ_military_bridge.rs`' own module doc too,
  because that file is where somebody would go to change it.
- **The reference has nothing here.** Grepping the frozen snapshot for
  `manpower`, `mobiliz`, `levy`, `conscript` and `militia` returns exactly two
  hits, both `JP_COST_TOLL_PER_BORDER`'s comment using "levy" to mean a toll.
  `reference/FUNCTION_INDEX.md` has zero. So unlike CV-25's first pass — which
  turned out to be a port nobody had recognised — this one really is new, and
  it is checked by unit tests and a live probe rather than by golden parity.

---

## 1 · The owner's specification, as supplied

> **Core correction:** agricultural technology does not directly determine
> army size. It determines *surplus, labour requirements, transport capacity,
> taxation base, and administrative capacity*, from which military manpower is
> supported.
>
> **Four separate outputs, not one "military size" statistic:**
>
> 1. **Standing Army Capacity** — people continuously maintained under arms
> 2. **Campaign / Field Army Capacity** — troops sustainably concentrated for
>    a campaign
> 3. **Emergency Mobilization** — how many can be called up temporarily
> 4. **Maximum War Duration** — how long those people can be kept away from
>    productive work
>
> These can differ radically. Imperial Rome: ~250,000 regulars under Tiberius
> rising to ~380,000–450,000 by the 2nd/early 3rd century, governing perhaps
> 45–120 million. Yet early Republican Rome temporarily mobilized 17–29% of
> its *citizen* population during the Second Punic War (Hopkins'
> reconstruction).
>
> **Five interacting variables drive it — technology era should NOT be the
> primary variable:**
>
> 1. `food_surplus_per_farmer` — agricultural productivity
> 2. `agricultural_labour_ratio` — how many must remain in agriculture
>    (medieval 70–90%; modern industrial a few percent). Extremely important.
> 3. `fiscal_extraction_efficiency` — how much surplus the state can actually
>    capture. A wealthy society with weak taxation supports a surprisingly
>    small army; a poorer centralized state can support a disproportionately
>    large one.
> 4. `professionalization` — how much is continuously maintained (standing vs.
>    levy)
> 5. `logistics_capacity` — how far the army can operate from its food base
>    (roads, rivers, ships, pack animals, wagons, carts, rail, motor
>    transport, refrigeration, preservation)
>
> **Two derivation chains:**
>
> ```
> Population → working-age → agricultural population → food surplus
>   → extractable surplus → fiscal capacity → military budget → STANDING ARMY
>
> Population → military-age → mobilization pool → available levy
>   → logistical capacity → MAXIMUM SUSTAINABLE FIELD ARMY
> ```
>
> **War duration is a real constraint, not flavour.** A state may raise 10%
> for 30 days but only 2% for a multi-year war without collapsing agricultural
> production. Feudal obligations were often ~2 months before warriors expected
> payment or went home. The agricultural calendar constrains *when* people can
> leave (pre-8th-century-BC Assyria drafted seasonally; it later developed a
> professional standing army supplemented by levies). Model: 30 days feasible
> → 90 difficult → 180 severe disruption → 365 requires a substantially
> different fiscal system.
>
> **Era table — use as modelling ranges and a sanity check, NOT as the
> driver.** "These are modelling ranges, not historical laws." Geography,
> state organization, wealth inequality, military culture and whether soldiers
> are self-supporting can move a society substantially outside them.
>
> | Era | Sustainable standing | Wartime mobilization | Main constraint |
> |---|---|---|---|
> | Hunter-gatherer | ~0–1% | ~5–15% | Food availability / seasonal movement |
> | Early horticulture | ~0–1% | ~5–15% | Very limited surplus |
> | Neolithic agriculture | ~0.1–1% | ~5–15% | Labour needed on farms |
> | Bronze Age state | ~0.5–2% | ~5–15% | Administration + food storage |
> | Iron Age agrarian state | ~1–2.5% | ~10–20% | Logistics and harvest cycle |
> | Classical agrarian state | ~1–3% | ~10–25% | Fiscal/logistical capacity |
> | Late antique / early medieval | ~0.2–1.5% | ~5–15% | Political fragmentation |
> | High medieval | ~0.5–2% | ~5–15% | Feudal obligations / campaign duration |
> | Late medieval | ~1–3% | ~10–20% | Money and logistics |
> | Early gunpowder | ~1–3% | ~10–20% | Fiscal administration |
> | Military-fiscal state | ~1–4% | ~10–25% | State finances |
> | Early industrial | ~2–5% | ~15–30% | Transport and supply |
> | Railway / industrial mass army | ~3–8% | ~20–40%+ | Industrial logistics |
> | Total industrial mobilization | ~5–10% | ~30–50%+ | Industrial capacity / demographics |
> | Modern mechanized | <1–3% active | ~5–15% usually | Technology makes manpower less valuable |
>
> **Worked example the model must reproduce** — same population, very
> different military power:
>
> - *Kingdom A*: 1,000,000 pop, 75% agricultural, high labour requirement,
>   weak taxation, poor roads → standing ~5,000; emergency levy ~40,000;
>   sustainable field army ~15,000–20,000.
> - *Kingdom B*: 1,000,000 pop, 55% agricultural, high surplus, strong
>   taxation, good roads/rivers, professional bureaucracy → standing ~20,000;
>   mobilization pool 100,000+; sustainable field army ~40,000–60,000.
>
> **Two modelling cautions:** ancient textual army numbers are massively
> exaggerated (Xerxes' invasion described in millions; modern reconstruction
> ~70,000 infantry + 9,000 cavalry) — apply a logistics plausibility check
> rather than trusting stated figures. And "warrior societies" should not
> automatically get huge standing-army modifiers: a hunter-gatherer band's
> fighters are also its hunters, herders, toolmakers, scouts and parents; the
> military is the adult population temporarily switching occupation.

---

## 1a · Annotation — the owner's ruling on the era table's denominator

> **This section is not the owner's specification.** §1 above is reproduced
> verbatim and is not edited; this is a ruling *about* it, made 2026-08-25 in
> response to finding 1 of §3.3, recorded separately so the two never blur.

**The question §3.3 put back to the owner.** The first build reported the era
bands against *total* population, and their verdicts read `below`
persistently — because the specification's era table, its worked example and
its own cited Imperial Rome figure disagree in one consistent direction. That
finding offered a reconciliation without implementing it.

**The ruling: the era table's percentages are shares of the citizen / free
population, not of the total population.**

The evidence is inside the specification itself. Its Republican Rome figure is
stated as *"17–29 % of its **citizen** population"* (Hopkins' reconstruction) —
the one place §1 names a denominator, it names that one. Under this reading:

- the live figures of **5.9–8.2 %** of total population become roughly
  **12–25 %** of a citizen body, landing inside the Iron Age (10–20 %) and
  classical (10–25 %) mobilization bands rather than under both;
- the Imperial Rome case — ~250 000 regulars governing 45–120 million — stops
  being anomalously *below* a 1 % classical floor, because the citizen body
  under a pre-Caracalla empire is a minority of the governed population.

**What the ruling does and does not change.**

| | |
|---|---|
| **Changes** | the denominator of `era_standing_verdict` and `era_mobilization_verdict`, and nothing else |
| **Does not change** | the four outputs — standing, field, emergency, war duration. They are calibrated on §1's own worked example and validated in §3.1, and this ruling did **not** recalibrate them. (Owner ruling AI (c) later re-baselined the standing army by design — §2.4, §3.1) |
| **Does not change** | the war-duration curve. Its two anchors ("10 % for 30 days, 2 % for a year") are stated as shares of a whole population and stay that way, as does the force ladder's `share` |
| **Does not change** | the era *assignment*. `era_for` reads the five variables; which row a faction lands in is untouched |

`the_citizen_ruling_moves_no_headcount` pins every Kingdom A and B headcount in
§3.1 as a literal — levy and field at the values published before the citizen
population existed, standing at the values owner ruling AI (c) re-baselined
them to — so a future edit to the denominator that leaked into an output fails
loudly rather than silently recalibrating a validated model.

---

## 2 · The derivation

Implemented in `cartalith-native/crates/cartalith-civ/src/manpower.rs`, a pure
module with no state. Read at the boundary by
`cartalith-godot/src/civ_military_bridge.rs`, nested under each faction row of
`civ_military_summary()`.

**Derived and recomputed, stored nowhere** — the same contract
`civ_faction_aggregates`, `relations`, `territory_influence`, `trade_flows` and
`wildlife_regions` all ship on. `CivData` gains no field, the save format is
untouched, and `resident_bytes` is **0**. The only allocation is per-faction
scalars plus one `O(cells)` sweep over `civ.territory` shared with the
land-capacity sum; nothing grid-sized is retained, so there is no
`transient_bytes` figure worth quoting — the working set does not move
measurably.

### 2.1 What already existed

Six of the pieces this model needs were already built and, in two cases, had
**never been read by anything**. Inventoried before writing a line, per this
repo's standing rule — the register's stated reasons had by then been wrong
six times in one session (2026-08-25):

| Need | What it maps onto | What that piece was reaching before this pass |
|---|---|---|
| Agricultural labour ratio | `roster::AG_TECH_LEVELS`' `farmers_per_urbanite` (reference line 14817, ported verbatim) | **no consumer anywhere** — its own module doc says so |
| Fiscal capacity | `roster::CIV_GOVERNMENTS` (reference 14794) | **no consumer in either codebase** — its own doc says the reference reads it nowhere either |
| Food capacity | `timeline::civ_current_agrarian_density` → `CivData::dens`, and `civ_agrarian_regional_total`'s "Land sustains ≈ N" | live, integrated over the whole map |
| Population per faction | `civ_faction_aggregates`' `pop` / `territory_km2` / `capital` | live |
| Road connectivity | `trade::RoadComponents` (union-find over `Way::a_idx`/`b_idx`, IN-13) | live |
| Water access | `trade::place_navigability` → `NavKind` (IN-13) | live |
| Way tiers | `WayType`'s four rungs and `Way::km` | live |

So this pass wrote **one new module** and read seven existing ones. Two inert
tables now have their first consumer — the same finding CV-25's first pass
made about `umWalls`/`umAge`.

### 2.2 The population identity, and why it is the reference's own

The model needs a *total* population; the aggregate supplies the **settled**
one. The bridge between them is not an assumption made here:
`AG_TECH_LEVELS.farmers_per_urbanite` is **defined** against urbanites, and
`timeline::civ_settlement_population` sizes a nucleus at a
`civ_surplus_fraction` (0.10–0.65) of what its catchment sustains. So:

```
total_population        = nucleated_pop × (1 + f)
agricultural_labour_ratio α = f / (1 + f)
farming_population      = total_population × α
non_agricultural        = total_population × (1 − α)   [ = nucleated_pop ]
```

with `f = farmers_per_urbanite`. The six ag-tech rows give α = 0.95, 0.90,
0.80, 0.50, 0.31, 0.13 — which is exactly what each row's own `hint` string
says ("~95% of the population farms", …). A unit test pins that, because a
silent drift between the table and its reading would be invisible.

### 2.3 The five variables

**1 · `food_surplus_per_farmer`** = `ecological_factor / f` — people a farmer
feeds *beyond his own household*. Technology sets the ratio; the land decides
whether it is met.

**`ecological_factor`** = `clamp(land_ratio / world_ratio, 0.25, 4.0)`, where
`land_ratio = land_capacity / total_population`, `land_capacity` is Σ
`dens[i] × cellKm²` over the faction's own territory cells — exactly the
integral `civ_agrarian_regional_total` takes over the whole map, restricted to
one owner — and `world_ratio` is Σ land / Σ population over every faction of
the world (`world_land_reference`, applied by `civ_military_manpower_world`).
So the factor is *relative*: 1.0 is the world's own land per person, and a
faction whose land per head is twice the world's reads 2.0. **This is the
geography term**, and it is why two factions on the same ag-tech row do not get
the same answer. The bounds are the named constants `ECOLOGICAL_FLOOR` and
`ECOLOGICAL_CEILING` in `cartalith-civ/src/manpower.rs`; the ceiling is
`1 / floor`, and a unit test pins that identity.

Two dated changes gave it this shape; §3.3 finding 3 carries the before- and
after-measurements for both.

- **The ceiling was `2.0` until 2026-09-06** (owner ruling 11): at `2.0` it was
  saturating, and so deciding the answer rather than guarding it. The floor
  did not move. The constants' doc carries the distribution the new ceiling was
  chosen from.
- **The world normalisation dates from 2026-09-23** (owner ruling AI, option
  (b)). The raw ratio tracked the map's area. `land_capacity` integrates
  physical km², while the population it is divided by comes from settlements
  sized off fixed-km² catchments, whose count is `clamp(gw·gh/65536·20, 8, 40)`
  — grid cells, capped at 40. Measured on the six-seed sweep in §3.3: 40
  settlements on both the 800 km and 2 000 km shapes, world reference 0.72
  against 5.43 (seed 483920). That is the 6.25× area ratio, give or take
  terrain. The single-faction `civ_military_manpower` still takes land in
  absolute terms. Only the whole-world entry point normalises.

**2 · `agricultural_labour_ratio`** — §2.2 above.

**3 · `fiscal_extraction_efficiency`** — the share of the *non-agricultural
surplus* the state captures, `0.04 … 0.16` linear in a normalised
`state_capacity`:

```
state_capacity = clamp( GOVERNMENT_EXTRACTION[gov]
                      × (0.55 + 0.45 × capital_road_reach)
                      × (0.70 + 0.60 × urbanisation), 0.03, 0.95 )
```

- `GOVERNMENT_EXTRACTION` — none 0.10, chiefdom 0.15, tribal confederacy 0.20,
  monarchy 0.45, theocracy 0.45, oligarchy 0.50, republic 0.55, city-state
  0.55, empire 0.70. Unknown keys read as `chiefdom`: a government this port
  cannot classify should not be credited with an imperial treasury.
- `capital_road_reach` — share of the faction's settlements in the same
  `RoadComponents` component as its capital. A state cannot tax what it cannot
  reach.
- `urbanisation` — `(1 − α)` normalised by the table's own maximum
  (`1/1.15`, the `industrial` row), so no hard-coded number can drift from it.

Note what this is a share *of*: with α = 0.75 the non-agricultural quarter is
what the fraction applies to, so the ceiling corresponds to a state capturing
about 7 % of everything — at the top of what pre-modern fiscal systems
managed.

**4 · `professionalization`** = `clamp(0.15 + 0.55·state_capacity +
0.30·urbanisation, 0, 1)`. Used in exactly two places, and deliberately not a
third: it splits the standing army into a `professional_core` and a seasonal
remainder, and it enters the campaign-duration capability (professionals do
not go home for the harvest). It is **not** folded into `SOLDIER_UPKEEP` —
that was tried, and it moved the two worked examples in opposite directions,
because a levy-heavy standing force is cheaper per head *and* less of it is
genuinely standing, and one constant cannot carry both effects.

**5 · `logistics_capacity`** = `clamp(0.15 + 0.45·road + 0.30·navigable +
0.10·sea, 0, 1)`:

- `road` — Σ `way.km × tier_weight` for ways with **both** endpoints in the
  faction, per 1 000 km² of its territory, against a reference of 10.
  Tier weights: highway 1.00, regional 0.80, road 0.55, track 0.30 — the
  reference's own `maxU` classification, not a second vocabulary. A way whose
  two ends sit in different factions counts for neither: it is a road
  *between* polities, and crediting both would let a shared frontier road make
  two states look better supplied than either is.
- `navigable` / `sea` — share of the faction's settlements whose
  `place_navigability` verdict is river-or-sea, and sea specifically.

### 2.4 The four outputs

**Standing army** (chain 1 — fiscal):

```
military_budget = non_agricultural × ecological_factor × fiscal_extraction_efficiency
standing_army   = military_budget / soldier_upkeep(α)
professional_core = standing_army × professionalization
```

The non-agricultural population *is* the embodied surplus: those are the
people the farmers' surplus already feeds. `soldier_upkeep` is what one
standing soldier costs the treasury, in subsistence-equivalents.

> **Until 2026-09-23 this was one constant, `SOLDIER_UPKEEP = 3.0`** (Roman
> legionary pay against a subsistence wage). Owner ruling AI, option (c),
> replaced it with one value per `era_for` α-bracket,
> `SOLDIER_UPKEEP_BY_BRACKET`, because the era table is not monotone in α — its
> Iron Age row sits above its High-medieval one — and a single divisor makes
> the standing share proportional to `(1 − α)`. **Derived, not fitted to a
> world**: for each ag-tech row (each lands in exactly one bracket), over the
> eight roster governments × capital reach 0…1 at `ecological_factor = 1` and
> logistics 0.5, the bracket's upkeep is the geometric mean of the model's
> standing citizen share at upkeep 1 over the band centre of the era that
> polity lands in. Result, α ≥ 0.93 → 0.85 → 0.70 → 0.45 → 0.25 → below:
> **1.120 · 0.798 · 1.815 · 2.221 · 1.770 · 1.071** (the `α < 0.10` bracket has
> no roster row and repeats the last). Read after the fact: a soldier costs the
> state most in the paid-army brackets and least where soldiers were largely
> self-supporting (land-grant levies) or conscripted — a value under 1 is the
> treasury paying for less than all of him, not a soldier eating less than a
> peasant. A unit test re-derives the table from `ERA_BANDS`, `era_for`,
> `GOVERNMENT_EXTRACTION` and `CITIZEN_SHARE`.

**This is where the owner's warrior-society caution is honoured
structurally.** At α = 0.95 the non-agricultural population is 5 % of the
total, so a subsistence polity's standing army collapses to almost nothing
without any special case, while its levy stays demographic and large. A unit
test pins both halves.

**Emergency mobilization** (chain 2 — demographic):

```
mobilization_pool = total_population × MILITARY_AGE_FRACTION       (= 0.25)
levy_reach        = clamp(0.04 + 0.30·state_capacity + 0.22·logistics, 0, 0.60)
emergency         = mobilization_pool × levy_reach
```

`MILITARY_AGE_FRACTION = 0.25` is the 15-50 male cohort under a
high-mortality age structure (22-26 % of the whole population). `LEVY_BASE` is
not zero because a state with no administration and no roads still raises the
men who live where the fighting is; what it cannot do is reach the rest.

**Field army** (chain 2's logistical tail):

```
field_army = emergency × (0.34 + 0.20 × logistics_capacity)
```

The base is not zero because an army marches on what it carries and forages
before any road matters; the logistics term is what lets it *stay*
concentrated once that runs out.

**War duration.** A two-parameter curve fitted through the owner's own two
anchors — *"a state may raise 10 % for 30 days but only 2 % for a multi-year
war"* — so those two points are the only thing to argue with:

```
exponent    = ln(365/30) / ln(0.10/0.02)   = 1.5525
coefficient = 365 × 0.02^exponent          = 0.8429
days(share) = clamp( coefficient / share^exponent × capability, 7, 365 )
capability  = (0.55 + 0.90·state) × (0.75 + 0.50·logistics) × (0.85 + 0.30·professionalization)
```

`capability` is exactly `1.0` at `state = logistics = professionalization =
0.5`, so the two anchors mean what they say for a median polity and are
modulated, never overridden, for anyone else. State capacity pays the army,
logistics feeds it where it stands, and professionalization is why it does not
leave at harvest.

The same curve inverted gives the **force ladder** — the largest force
sustainable at 30 / 90 / 180 / 365 days, capped at `emergency_mobilization`.
That ladder is the model's most informative output: it is what makes the other
three comparable, and it is the direct answer to "10 % for a month, 2 % for a
year". A rung marked `capped_by_pool` is limited by how many can be raised at
all rather than by how long they can be fed.

**Plausibility** is `concentration_ratio = field_army / emergency`, reported
rather than warned about. A host claimed above the field figure could not have
been fed in one place, whatever a chronicle says — the Xerxes case, as a
number instead of a caveat.

### 2.5 The era is an output

`era_for(drivers)` picks a row of the table in §1 from the **agricultural
labour ratio first** (the owner's "extremely important" variable, and the one
that actually separates the eras), split by `state_capacity` where several
rows share a ratio — which is exactly what distinguishes a Bronze Age palace
from a classical state, or a fragmented post-Roman west from a high-medieval
kingdom.

Deliberately **not** a lookup on the ag-tech key: that would make technology
the driver, which is the thing this module exists to stop doing. Two factions
on the same ag-tech row with different governments and different road networks
land in different eras, and they should.

The band is then **reported, never enforced**, and since §1a's ruling it is
reported against the **citizen population** (§2.6): `era_standing_verdict` and
`era_mobilization_verdict` read `within` / `above` / `below`, and nothing is
clamped into range. The owner's own words are the reason — *"these are
modelling ranges, not historical laws."* `Hunter-gatherer` is retained in the
table and is unreachable from this port's generated worlds, since the lowest
ag-tech row is hoe cultivation rather than foraging; kept so the table is the
owner's table, the same convention `civ_base_pop_for_kind`'s own unreachable
row already carries.

### 2.6 The citizen / free population — the band's denominator

Derived from §1a's ruling. **It is a denominator and nothing else**: no
headcount reads it, and the model's calibration is untouched.

**Grepped before inventing.** Nothing in `cartalith-civ` distinguished a
citizen, free or full-status subset of population — `citizen`, `free`, `serf`,
`slave`, `caste`, `social` and `status` return nothing but unrelated prose.
The faction profile was checked too: `FactionEntry::culture` is
`CIV_CULTURES`, which is name-syllable pools and carries no social structure,
and `religion` carries none either. So it is derived from what does exist.

```
citizen_fraction   = clamp( CITIZEN_SHARE[government]
                          + CITIZEN_MODERNISATION × urbanisation,
                          0.20, 0.98 )
citizen_population = total_population × citizen_fraction
```

`urbanisation` is the *same* normalised `(1 − α)` term `state_capacity`
already uses — no second vocabulary for the same quantity.

**Government is the driver**, which is the right one on the merits and not
merely the available one: the two cases §1 cites sit on either side of exactly
this distinction. A republic's citizen body is a much larger share of its
polity than an empire's, and that is what makes Hopkins' 17–29 % and Rome's
0.21–0.56 % consistent with one table.

| Government | Share | Grounding |
|---|---|---|
| `none`, `chiefdom` | 0.90 | kin-based polities barely distinguish status — the owner's warrior-society caution seen from the denominator's side |
| `tribal_confederacy` | 0.88 | as above, with a subject periphery |
| `monarchy`, `theocracy` | 0.55 | a servile/half-free substrate under free peasants, burghers and gentry; Domesday's ~10 % slaves and two-thirds villeins/bordars against ~1/7 free sokemen brackets it |
| `republic` | 0.50 | Rome's own case: a citizen body of order a million *with families* against an Italian population of some four million including allies and slaves |
| `city_state` | 0.45 | Athens c. 431 BC — ~150 000 citizens with families against 80–100 000 slaves and 25–50 000 metics with theirs |
| `oligarchy` | 0.40 | enfranchisement narrows by definition. **The least-grounded row**: Sparta's Spartiates over the helots is far lower, Venice's patriciate over a free populace far higher |
| `empire` | 0.30 | conquered subjects and large slave populations stand outside the citizen body; pre-Caracalla Roman citizens are usually put at a fifth to a third of the empire |

Unknown keys read as `chiefdom`, the same fallback `government_extraction`
takes and for the **opposite** reason: there the conservative direction is to
deny an unclassifiable state an imperial treasury; here a *high* citizen
fraction is the conservative one, because it makes a share of that body
*smaller* and so cannot flatter a faction into its band.

**`CITIZEN_MODERNISATION = 0.68` is derived, not chosen.** Legal servitude is
an agrarian institution — chattel slavery, serfdom and villeinage are all ways
of binding labour to land, and all disappear as the agricultural labour ratio
collapses. So the fraction is a government's *floor* plus what modernisation
adds, and the value follows from one statement: *at full industrialisation,
civic status is universal whatever the government is called.* That fixes it at
`CITIZEN_CEILING − min(CITIZEN_SHARE)` = `0.98 − 0.30`. A unit test pins the
identity, so editing the lowest row without editing this fails loudly.

It also keeps the owner's own table internally consistent at the top: the
industrial rows quote mobilization at 30–50 %, which is only reachable at all
against a denominator close to the whole population — and at those labour
ratios every government has converged on the ceiling.

The ceiling is `0.98` rather than `1.0` because children, the aged and the
infirm were never part of any "free population" a military figure was quoted
against.

**On screen.** The denominator is surfaced, not invisible: CIVIL ▸ Military
gains a *Who the bands are measured against* group with one line per faction
(citizen headcount, its share of the total, both citizen-based shares and
their verdicts, the era), whose tooltip also quotes what the same two figures
would read against total population — the previous basis, kept legible rather
than deleted. The Faction Roster's Military block names the citizen population
and the government that produced it on the line immediately above its verdict.

---

## 3 · Verification

### 3.1 The worked examples — **both reproduced**

Unit tests `worked_example_kingdom_a` / `_b`, with the owner's qualitative
inputs mapped to drivers (75 % agricultural → `f = 3`; 55 % → `f = 11/9`;
"weak taxation, poor roads" → monarchy, reach 0.20, road 0.10; "strong
taxation, good roads/rivers, professional bureaucracy" → empire, reach 0.90,
road 0.70, navigable 0.60, sea 0.50) and both at a population of exactly
1 000 000:

| | Kingdom A stated | A produced | Kingdom B stated | B produced |
|---|---|---|---|---|
| standing army | ~5 000 | **9 661** (5 846 before 2026-09-23) | ~20 000 | **25 750** (19 067 before) |
| emergency levy | ~40 000 | **41 221** | 100 000+ | **98 889** |
| field army | 15 000–20 000 | **15 870** | 40 000–60 000 | **47 368** |

**Re-baselined 2026-09-23 by owner ruling AI (c), deliberately and with the
owner's acceptance.** The standing figures no longer reproduce the stated
ones — A is **+93 %** on "~5 000", B **+29 %** on "~20 000" — because the
soldier upkeep is now derived from the era table (§2.4) rather than fitted to
this example; the specification's table and its worked example disagree
(finding 1), and the ruling chose the table. Against that table both land
inside: A 1.30 % of citizens in High medieval's 0.5–2 % (centre 1.25 %), B
3.95 % in Military-fiscal's 1–4 %. Levy and field army did not move. **One
tension this surfaced, not resolved:** B's standing army is now 6.7 % above its
own 365-day ladder rung (24 129) — the Military-fiscal band's top and the
duration curve's "2 % for a year" anchor disagree there. Nothing clamps one by
the other; that would be a ruling.

Before the re-baseline, every figure was in range, the furthest out being
Kingdom A's standing army at +17 %.

The derived eras and durations, which the specification does not state and so
cannot be fitted to:

| | Kingdom A | Kingdom B |
|---|---|---|
| era (derived) | High medieval | Military-fiscal state |
| citizen / free population (§2.6) | 745 500 (74.6 %, monarchy) | 651 900 (65.2 %, empire) |
| standing share of citizens | 1.296 % since 2026-09-23 (0.784 % before; band 0.5–2 %, **within**) | 3.950 % since 2026-09-23 (2.925 % before; band 1–4 %, **within**) |
| mobilization share of citizens | 5.53 % (band 5–15 %, **within**) | 15.17 % (band 10–25 %, **within**) |
| standing share of total, for comparison | 0.966 % (0.585 % before) | 2.575 % (1.907 % before) |
| field army sustainable | 337 days | 128 days |
| full levy sustainable | 77 days | 41 days |
| ladder 30 / 90 / 180 / 365 d | 41 221 · 37 126 · 23 756 · 15 067 | 98 889 · 59 455 · 38 045 · 24 129 |

**Re-validated 2026-08-25 after §1a's ruling**: every headcount in both tables
was unchanged to the unit, which is what
`the_citizen_ruling_moves_no_headcount` was written to assert. What moved is
the two verdict rows — Kingdom A's mobilization read `below` its 5 % floor
against total population and reads `within` against its citizen body, which
is exactly the reconciliation the ruling makes.

Two of those are worth reading twice. **Kingdom A's full levy sustains 77
days** — the feudal ~2-month obligation, out of a curve fitted on two unrelated
points. And **Kingdom B's 90-day rung is 59 455 and its 180-day rung 38 045**,
which brackets the owner's stated 40 000–60 000 field army almost exactly: a
"field army" *is* a campaign-season force, and the model says so without being
told.

### 3.2 Live figures — a real 233-settlement, six-faction world

> **Measured 2026-08-25; read §3.2 and §3.2a as the record that produced
> findings 1-3, not as today's output.** Every standing figure and verdict in
> them predates three later changes, each of which moves the standing side:
> owner ruling 11's ecological ceiling (2026-09-06), per-faction default
> governments (`roster::civ_default_government`, 2026-09-23 — the "all
> `monarchy`" roster below is no longer the default), and owner ruling AI
> (2026-09-23). The current standing-verdict counts on both of these worlds are
> the "(b) + (c)" column of the last two rows of finding 2's table.

Shell-level, seed 483920, 2400 km, 384×288, villages on, run windowed and
headless. **PASS**, every claim measured.

| faction | standing | field | levy | ladder 30 / 90 / 180 / 365 d |
|---|---|---|---|---|
| Veldmark | 1 509 | 9 305 | 20 262 | 20 262⌈pool⌉ · 13 874 · 8 878 · 5 631 |
| Korrath | 1 380 | 7 656 | 17 239 | 17 239⌈pool⌉ · 12 413 · 7 943 · 5 038 |
| Aurelia | 1 040 | 6 172 | 13 569 | 13 569⌈pool⌉ · 9 364 · 5 992 · 3 800 |
| Sythe Dominion | 965 | 5 078 | 11 624 | 11 624⌈pool⌉ · 8 594 · 5 499 · 3 488 |
| Mirelle | 877 | 4 532 | 10 449 | 10 449⌈pool⌉ · 7 872 · 5 037 · 3 195 |
| Draumr League | 87 | 3 444 | 8 009 | 8 009⌈pool⌉ · 6 041 · 3 865 · 2 452 |

Logistics capacity spreads 0.45 … 0.60 across the six, and the plausibility
line reads *"no faction here can concentrate more than 43 % of what it can
raise."*

**Band verdicts after §1a's ruling (2026-08-25), same world, same seed.** Every
headcount above is unchanged; only the denominator is. The default roster then
seeded every faction `monarchy`, so the citizen fraction is 62.8 % throughout
here — see §3.2a for the differentiated run.

| faction | citizens / total | standing % of citizens | mobilization % of citizens | era |
|---|---|---|---|---|
| Veldmark | 215 862 / 343 620 | 0.70 % **within** | 9.4 % **within** | Bronze Age state |
| Korrath | 198 078 / 315 310 | 0.70 % **within** | 8.7 % **within** | Bronze Age state |
| Aurelia | 144 737 / 230 400 | 0.72 % **within** | 9.4 % **within** | Bronze Age state |
| Sythe Dominion | 138 958 / 221 200 | 0.69 % **within** | 8.4 % **within** | Bronze Age state |
| Mirelle | 129 516 / 206 170 | 0.68 % **within** | 8.1 % **within** | Bronze Age state |
| Draumr League | 97 962 / 155 940 | 0.09 % **below** | 8.2 % **within** | Bronze Age state |

Five of six now read `within` on both bands where the previous basis read
`below` on standing for all six. **Draumr League is honestly still below**, and
for a reason the model already discloses: its `ecological_factor` is 0.428 —
its territory feeds well under half the people on it — which is finding 3, not
a denominator problem. Its standing army is 87 against Veldmark's 1 509 on
identical institutions, so no denominator was ever going to move it inside a
band.

The Faction Roster's Military block, on the same world, for Aurelia:

```
Power: 49/100 relative to the other factions  ·  2 of 27 settlements fortified (2 stone, 0 palisade, 0 ditch)
Standing army 1 040 (professional core 324)  ·  sustainable field army 6 172  ·  emergency levy 13 569
Out of a total population of 230 400 (90% in farming), of whom 57 600 are of military age.
A field army stays out 172 days; a full levy 51.
Citizen / free population 144 737 — 63% of the total, the share a monarchy confers.
This is what the era bands below are measured against, not the whole population.
Reads as a Bronze Age state (Administration + food storage). Standing 0.72% of citizens —
within that era's 0.5–2.0% band; mobilization 9.4% — within its 5–15%.
```

### 3.2a The denominator differentiated — one government per faction

The default roster was then all-`monarchy`, so a live run on it proved the
citizen population exists but not that it *discriminates*. The engine probe
therefore assigns a different government to each of the six and re-reads the
verdicts (33 settlements, 1200 km, seed 483920). **The citizen fraction spreads
0.378 … 0.978** across them, and the headcounts are pinned unchanged when the
roster is restored.

| faction | government | citizens / total | standing % of citizens | mobilization % of citizens | era |
|---|---|---|---|---|---|
| Aurelia | monarchy | 255 508 / 406 730 (62.8 %) | 0.562 % below | 13.09 % **within** | Iron Age agrarian state |
| Veldmark | empire | 98 529 / 260 520 (37.8 %) | 0.910 % below | 21.62 % **within** | Classical agrarian state |
| Korrath | republic | 113 686 / 196 620 (57.8 %) | 0.750 % below | 11.28 % **within** | Iron Age agrarian state |
| Sythe Dominion | chiefdom | 243 807 / 249 240 (97.8 %) | 0.367 % below | 5.52 % **within** | Bronze Age state |
| Mirelle | oligarchy | 76 780 / 160 560 (47.8 %) | 1.201 % **within** | 14.24 % **within** | Iron Age agrarian state |
| Draumr League | theocracy | 107 699 / 171 440 (62.8 %) | 0.185 % below | 9.35 % below | Iron Age agrarian state |

**Mobilization is fixed; standing is improved but not fixed, and that is
reported rather than tuned.** On this sparser world the same six factions
previously read `below` on *both* bands for all six; they now read `within` on
mobilization for five of six, and `within` on standing only for the oligarchy —
the narrowest citizen body of the set. The residual is finding 2's, not
finding 1's: this model's standing armies land at **Imperial Rome's own
ratio**, and the era table's standing column is the part of it that the
specification's own cited figures never agreed with. (Finding 2 later
corrects that framing: the ratio was a measured result at one ag-tech level,
not a constant.) §1a's ruling closed the
gap by roughly a factor of 1.6 rather than closing it entirely, and the honest
statement is that the standing column remains the looser fit of the two.

Engine-level (33 settlements, 1200 km) drove the assertions the shell cannot:

- **Differentiated, not all-zero and not all-equal**: standing 199 … 1 435,
  levy 10 074 … 33 444, levy share 5.88 % … 8.22 %.
- **The model's own ordering holds for every faction**: standing < field <
  levy, and no faction can mobilise more than 30 % of itself.
- **The ladder decreases everywhere**, is `capped_by_pool` at 30 days for
  every faction, and is *not* at 365 — so the demographic ceiling binds at the
  short end and the fiscal curve at the long end, which is the shape the model
  claims.
- **Ag-tech is genuinely live** (it reached nothing before this pass):
  `traditionalAgrarian → improvedAgrarian` moved faction 1's standing army
  **1 435 → 2 615**.
- **Government is genuinely live** (it reached nothing in *either* codebase):
  `chiefdom → empire` moved it **948 → 1 841**.
- **Geography is genuinely live**: with every faction forced onto identical
  institutions, standing still spreads **199 … 1 435**, logistics
  **0.415 … 0.841**, and ecological factor **0.428 … 2.000** (that upper
  figure is the *old* `2.0` ceiling truncating the spread, which is finding 3
  and was ruled on 2026-09-06; after that ruling the same probe read
  0.438 … 2.912 on the 33-settlement world, and ruling AI (b) has since
  re-normalised the factor). If this had
  collapsed to one number the model would have been a technology lookup
  wearing five variables.
- **The citizen population is a real subset, differentiated, and moves no
  headcount** (added 2026-08-25 for §1a): `citizen_population = total ×
  fraction` is asserted per faction, the fraction spreads **0.378 … 0.978**
  across six governments, total population does not move when only the
  government does, and every levy restores exactly when the roster is put
  back.

Plus `cargo test`: `cartalith-civ` **421 → 435 → 440** lib tests (14 new in
`manpower` for the model, 5 more for §1a's ruling), `cartalith-godot` 351,
`cargo check -p cartalith-godot` clean, `cargo clippy -p cartalith-civ` clean.

### 3.3 Four findings

**1 · The specification's era table and its worked example disagree with each
other, and the table disagrees with its own Imperial Rome figure.**
**→ Resolved 2026-08-25 by the owner's ruling in §1a. The finding is kept as
written, with the resolution recorded at its end.**

Kingdom A's stated ~40 000 emergency levy is **4.0 %** of its million people —
below the 5–15 % band of *every* pre-modern era in the table. Kingdom B's
stated 100 000 is **10 %**, exactly on the floor of its band. And the table's
own classical row says 1–3 % sustainable standing, while the specification's
cited Imperial Rome is ~250 000 regulars over 45–120 million, i.e.
**0.21–0.56 %** — under the floor by a factor of two to five.

The model is calibrated on the **worked example**, because it is concrete and
numeric, and reports the era band as the sanity check the specification asks
for. The consequence is that the standing verdict reads `below` more often
than it otherwise would, and that the mobilization verdict is sensitive to
which era row a faction lands in — on the 233-settlement world Aurelia reads
*Bronze Age state*, standing 0.45 % **below** the 0.5-2 % band and mobilization
5.9 % **within** the 5-15 % one, while on the sparser 33-settlement world the
same six factions land in *Iron Age agrarian state* and read `below` on both
against its tighter 1-2.5 % / 10-20 %. That is disclosed on screen rather than
hidden, and it is not tuned away.

A plausible reconciliation, offered rather than implemented: **the bands may
be shares of a citizen or free population, not of the total.** The
specification's own Republican Rome citation says so explicitly — *"17–29 % of
its **citizen** population"*. At a citizen fraction of a third to a half, the
live 5.9–8.2 % of total population becomes 12–25 % of a citizen body, which
lands inside the Iron Age and classical bands. **This is a decision for the
owner**: say which denominator the table means and the verdicts change without
any other part of the model moving.

> **Resolution (owner, 2026-08-25).** The reconciliation was accepted and is
> implemented — see §1a for the ruling, §2.6 for the derivation, and §3.2 /
> §3.2a for the re-measured verdicts. The consequence in numbers: on the
> 233-settlement world five of six factions now read `within` on both bands
> where all six read `below` on standing before, and the second paragraph of
> this finding — the one about verdicts reading `below` more often than they
> otherwise would — no longer describes the model. The residual is finding 2's
> and is stated there.

**2 · The standing-army figures land at Imperial Rome's own ratio, not at the
table's.** Aurelia reads **0.45 %** of its population under arms on the
233-settlement world and 0.35 % on the sparser 33-settlement one; forcing the
latter to `empire` moves it to 0.45 %. Rome under Tiberius is 0.21–0.56 %. So
the model agrees with the specification's *example* and disagrees with its
*table*, in the same direction and for the same reason as finding 1.

> **What §1a's ruling did not close (2026-08-25).** It moved the standing shares
> up by roughly 1.6× and that is enough on dense worlds — the 233-settlement
> six read 0.68–0.72 % against a 0.5–2 % band, all `within`. On the sparser
> 33-settlement world they read 0.19–1.20 % against a 1–2.5 % floor and only
> the narrowest citizen body clears it.
>
> **The framing was wrong, not just the numbers (2026-09-23).** Reading
> `manpower.rs` directly showed that "the model's standing armies sit at
> Imperial Rome's ratio" describes a *measured result at one ag-tech level*,
> not a constant the model is stuck at: `standing = total × (1−α) ×
> ecological_factor × fiscal_extraction_efficiency / SOLDIER_UPKEEP`, with `α`
> already driven per faction by `AG_TECH_LEVELS.farmers_per_urbanite`. The
> industrialisation dependence this finding implied was missing **already
> existed**. At equal population, land and institutions the standing army ran
> 1 090 (subsistence) → 2 219 (traditional agrarian, the default) → 24 617
> (industrial): **11.09×** industrial over traditional, 22.6× over subsistence.
> Re-measured after the 2026-09-06 ceiling raise and the 2026-09-23
> default-government wiring, the sparse world read **3 of 6 factions within
> band** (Aurelia 0.21 %, Veldmark 0.43 %, Draumr 0.34 %), not the 1 of 6
> first reported. A 108-sample sweep across seeds and world sizes then traced
> most of the remaining below-band cases to finding 3's map-scale effect, not
> to era or industrialisation. `LARGE_ITEM_RULINGS.md` Ruling AI carries the
> correction and the three options it put to the owner.
>
> **Ruled and built, 2026-09-23 — the owner chose all three options.** (a) The
> ag-tech scaling closes this finding's original ask. (b) The map-scale
> normalisation is built — see finding 3. (c) The soldier upkeep is now one
> value per `era_for` α-bracket, derived from the era table (§2.4), which
> re-baselined the worked example (§3.1) and makes the equal-population ag-tech
> sweep deliberately non-monotone: 2 919 / 8 340 / 7 598 / 17 132 / 31 426 /
> 68 929 from subsistence to industrial (traditional above advanced is the
> table's Iron-Age-above-High-medieval shape; industrial/traditional is now
> **8.27×**, from 11.09×). `standing_army_rises_with_industrialisation_at_equal_population`
> pins all six literals and the one deliberate inversion. Measured on the same
> 108 faction-samples (`_mpscale_probe.tscn`), standing verdicts below /
> within / above:
>
> | | before | (b) alone | (b) + (c) |
> |---|---|---|---|
> | 1 200 km, 33 settlements (36) | 21 / 15 / 0 | 34 / 2 / 0 | 17 / 15 / 4 |
> | 800 km, 40 settlements (36) | 33 / 3 / 0 | 34 / 2 / 0 | 20 / 11 / 5 |
> | 2 000 km, 40 settlements (36) | 11 / 25 / 0 | 35 / 1 / 0 | 17 / 17 / 2 |
> | pooled (108) | 65 / 43 / 0 | 103 / 5 / 0 | **54 / 43 / 11** |
> | §3.2a sparse world, seed 483920 (6) | 3 / 3 / 0 | 6 / 0 / 0 | 2 / 3 / 1 |
> | §3.2 dense 233-settlement world (6) | 2 / 4 / 0 | 4 / 2 / 0 | 2 / 2 / 2 |
>
> The shape column is the point of (b): the below count stops tracking map
> size (33 vs 11 → 20 vs 17). (b) alone reads worse because it removed the
> 2 000 km shape's inflated factor and nothing yet corrected the flat upkeep;
> (c) is what brings the level back. **All 54 remaining below-band samples have
> `ecological_factor < 1`** — land-poor relative to their own world, which is
> the geography term reporting, not an era or scale artefact. Mobilization
> verdicts are unchanged in every row (10 / 98 / 0 pooled), as they must be:
> neither option touches the demographic chain.

**3 · `ecological_factor` saturated on real generated worlds — the ceiling
raised by owner ruling 11 (2026-09-06), then the factor normalised to the
world by owner ruling AI (b) (2026-09-23).** As first measured: five of six
factions on the 233-settlement world hit the then-`2.0` ceiling — their territory
sustains at least twice the population the model puts on it. This is not a
bug: it is the same divergence `civ_agrarian_regional_total`'s own "Land
sustains ≈ N … x % actually live in settlements" readout has always shown, and
the clamp is what stops it becoming absurd. But it does mean geography
discriminates mainly at the *low* end — Draumr League at 0.428 is where the
term does real work, and it is why that faction's standing army is 87 against
Veldmark's 1 509 on otherwise similar institutions. Whether generated worlds
should be more densely populated relative to their carrying capacity is a
separate question, and an old one.

> **Owner ruling 11, 2026-09-06: raise the ceiling so the factor
> discriminates again.** Done, to `4.0`. What was measured, in this order,
> because the ruling asked for the distribution *before* the value:
>
> **Before.** The raw `land_capacity / total_pop` ratio, read with both clamp
> ends opened to sentinels, over **108 faction-samples** — six seeds
> (483920, 7, 101, 202501, 999331, 31337) × three world shapes, because
> `land_capacity` integrates cell area while `nucleated_pop` does not, so one
> world is one sample of the wrong thing. Pooled: **0.008 … 17.9**, p25 0.37,
> **median 1.58**, p75 3.11, p90 9.88. **45 of the 108 sat at or above 2.0.**
> Per world, out of six factions: 0–4 on the 800 km and 1 200 km shapes,
> **3–5 on the 2 000 km one** — which is where the ruling's own "five of six"
> reproduces.
>
> **The rule for the value.** The distribution picks the neighbourhood — 4.0
> lies between the measured p75 and p90 — and the *reciprocal of the
> untouched floor* picks the exact value, so the constant is not fitted to
> the worlds it was measured on: a factor of four either way about 1.0, the
> land feeding four times the people on it or a quarter of them. Both failure
> modes were live and both were checked. A lower ceiling still pins too much
> of the sample to be a guard — 37 of 108 at 2.5 and 30 of 108 at 3.0,
> against 24 at 4.0 — so the clamp would go on making the decision. A higher one (the p90, near 10) pins only a tenth, but
> `standing_army` is **linear** in this factor and at that ceiling 20 of the
> same 108 factions read *above* their own era band on standing where none
> does at 4.0 — wrong on the merits, not merely inconvenient, because
> `military_budget`'s other term is the *realised* surplus while this factor
> is only the land's potential.
>
> **After.** Pinned at the ceiling: **24 of 108 (22 %)**, from 45 of 108
> (42 %). Distinct reported values, over the same 108 samples, 47 → 68. Worst world 4 of 6, typical 0–3.
> **45 faction-samples changed value**; every one of them had a raw ratio
> above 2.0 and every other row is unchanged. Their standing armies scale by
> `new_eco / 2.0`, i.e. **1.005× … 2.000×** (median 2.000×), and *only* the
> standing side moves: `total_population`, `citizen_population`,
> `emergency_mobilization`, `field_army`, the force ladder and both durations
> are identical on all 108 rows. That list is established by control flow:
> `campaign_capability` reads only `state_capacity` / `logistics_capacity` /
> `professionalization`, `professionalization` is `0.15 + 0.55*state_capacity
> + 0.30*urban_norm`, and `levy_reach`, `emergency`, `field`, the ladder and
> both durations never touch `ecological`. **The factor has two readers, not
> one:** besides `military_budget`, `surplus_per_farmer = ecological / f` in
> `military_drivers` is published as `food_surplus_per_farmer`, and so moves on
> every changed row. (The first write-up of this ruling said *"nothing but
> `military_budget` reads this factor"*, although §2.3 states the second
> reader. The list was right; the reason given for it was not.)
> `standing < field < levy` holds on all 108. The committed
> `_manpower_probe.tscn` reported **PASS** after this ruling and printed
> `ecological 0.438 … 2.912` on the 33-settlement world, where the ceiling used
> to truncate it at 2.000 (before ruling AI (b) re-normalised the factor).
>
> **What the ruling did not fix, disclosed rather than tuned around.** The
> raw ratio's *centre* tracks world size at a fixed faction count — median
> 0.39 on a 512×384 800 km world against 5.04 on a 768×576 2 000 km one — so
> part of the upper tail is map scale, not ecology, and the 2 000 km shape
> still pins half its factions at 4.0. Normalising that meant changing how
> `land_capacity` or `nucleated_pop` are computed, which ruling 11 did not
> authorise; owner ruling AI (b), below, took it up.

> **Owner ruling AI (b), 2026-09-23: normalise it — built.** The root cause,
> read at the code rather than inferred: `land_capacity` integrates physical
> km², while `total_population` comes from settlements sized off *fixed-km²*
> catchments (`civ_catchment_km2`) whose *count* is set by grid cells and
> capped at 40 (`place_settlements_with_water_edge_snap`'s `max_places`). The
> 800 km and 2 000 km shapes both carry exactly 40 settlements, so the ratio's
> level is settlement sparsity, i.e. map area. `civ_military_manpower_world`
> divides every faction's land by the world's own `Σ land / Σ total_population`,
> which cancels the area term exactly and anchors the population-weighted
> average faction at 1.0 — the carrying-capacity point the worked examples are
> stated at. Pinned by `map_scale_does_not_move_the_ecological_factor` (every
> land ×6.25: identical outputs) and
> `the_world_anchor_keeps_relative_geography_and_averages_one`. Median factor
> per shape, before → after: 1 200 km 1.43 → 0.64, 800 km 0.32 → 0.44,
> 2 000 km 4.00 (clamped) → 0.79; pinned at the 4.0 ceiling 26 → 0.
>
> **Two consequences, disclosed.** The reference is a sum over factions, so a
> change to one faction's total population (its ag-tech row, its settlements)
> now moves every other faction's factor slightly. And the **0.25 floor now
> binds on 36 of 108** (24 before): with the scale term gone, what remains is
> real between-faction spread — on seed 483920's 2 000 km world one faction's
> land is 31 370 against another's 2 926 123 at similar populations. Whether
> that floor is still a guard or is now deciding is owner ruling 11's
> question reopened from the other end; not changed here.

**4 · The road-density reference was wrong on the first try, and measuring it
is what found that.** Anchoring on the Roman empire's ~16 km of built road per
1 000 km² suggested a reference of 40. On real worlds that made roads a dead
term: factions came out at 1.1–9.1 weighted km/1 000 km², so `road_density`
read 0.03–0.23 and contributed at most 0.10 of a logistics capacity spanning
0.37–0.53. The error is a category one — **this port's way network is
inter-settlement trunk roads only**, with no local lanes, farm tracks or
streets, so it is not comparable to a road inventory. At a reference of 10 the
same six factions spread **0.11 … 0.91** and roads carry real weight.

---

## 4 · What this deliberately does not build

> **Reopened 2026-09-24 by Ruling AW** (`LARGE_ITEM_RULINGS.md`): war campaigns
> over time are not declined. The owner scoped them as territory over time and
> siege lines in their own Conflict layer under CARTO, with derived
> per-settlement garrisons. The bullets below are the reasoning as it stood
> before that ruling. Per-settlement garrisons are now defined in **§5.6**,
> change over time in **§5.7**, and campaigns in the form the ruling names (no
> unit movement or combat resolution) in **§5**.

CV-25's own narrowing, kept, minus the manpower half this document supersedes.
These are **declined**, with the reason, rather than deferred — nobody should
re-propose them without reopening the reasoning:

- **Per-settlement garrisons.** The per-*faction* headcounts are real. Which
  settlement holds which part of a standing army is a placement rule nothing
  here implies, and inventing one would be the fabricated number CV-25's first
  pass refused.
- **Campaigns, unit movement, combat.** Each needs a clock, a map objective
  and an opposed force as simulation inputs, and a rule that resolves them.
  None exists. Story planning's conflicts (`cartalith_civ::conflict`,
  `STORY_PLANNING_SCOPE.md` §5) do not change that. They are authored
  annotations that read these headcounts per side (`side_manpower`) and decide
  nothing.
- **Change over time.** Every number here is a reading of the world as it
  stands, and stops there — the same boundary `relations` holds.

Disclosed on screen in CIVIL ▸ Military ▸ Not built, in the same words.
**Superseded on screen by Ruling AW**: CIVIL ▸ Military now points campaigns at
CARTO ▸ Conflict (§5) and lists garrisons and change over time as scheduled,
not declined.

---

## 5 · Campaigns over time (Ruling AW)

The owner, 2026-09-24 (`LARGE_ITEM_RULINGS.md`, Ruling AW): war campaigns over
time "should not be declined", built as **"Territory over time and siege lines
drawn in their own Conflict layer under Carto."** This section defines what
that layer draws and from what. It adds **no simulation**: every line on it is
either something the author drew (a conflict's kind, place, years and sides —
`cartalith_civ::conflict`, SP-4) or something the timeline recorded (territory
per year — Ruling AT's base model). No reference ancestor; divergence by
addition (`DECISIONS.md` §7d).

The code is `cartalith_civ::campaign::campaigns_at`, one pure function over
the recorded timeline, the conflict store, the map scale and one year; the
bridge is `WorldGen::conflict_campaigns(year)` (`campaign_bridge.rs`); the
layer is the `conflict` row of CARTO ▸ Layers, drawn by
`map_overlay.gd::_draw_campaign`.

### 5.0 Which conflicts, and which territory

- **Which conflicts.** Every conflict whose years contain the cursor's year —
  SP-4's own `Conflict::active_in` (inclusive both ends, an open end runs on).
  Outside every conflict's years the layer has nothing to draw, and draws
  nothing.
- **Which territory "that year" is.** The recorded snapshot **in force**: the
  latest recorded year at or before the cursor. An unrecorded year between two
  records reads the earlier — claims hold until the next record changes them,
  which is Ruling AT's model. It is deliberately **not** the live claim grid's
  "territory holds at" year (`CivData::territory_year`): that one depends on
  the path the cursor took (scrub back from a later record and the grid keeps
  the later claims), and a history layer must draw the same front for the same
  year however you arrived at it. The two agree whenever the cursor sits on a
  recorded year.
- **Before anything is recorded**, there is no territory to read: the front
  and the changes are **absent** (not an empty list, which would read as "no
  border" or "nothing changed"). A snapshot recorded on a grid of another size
  reads the same way.

### 5.1 What a siege line is drawn from

- **From an authored siege, and only from one.** A conflict of kind `siege`
  (SP-4's marker) active in the cursor's year draws one ring. The ring's
  centre is the siege's drawn point, resolved against its anchor exactly as
  SP-4 draws it — so a siege attached to a settlement moves with it.
- **The besieged settlement** is the siege's anchor, when it is anchored to a
  settlement. A free-standing siege mark names no place; the layer does **not**
  guess the nearest town, because "nearest within how far" is a number nothing
  here grounds.
- **Defender and besiegers** are read, not decided: the defender is the owner
  of the cell under the centre in the territory in force, when that owner is
  one of the conflict's sides; the besiegers are the other sides. When the
  owner is not a side, or nothing is recorded, both are absent.
- **Radius: Alesia's contravallation.** Caesar gives the circuit of the inner
  siege line as eleven Roman miles (*XI milia passuum*, *De Bello Gallico*
  VII.69). At the customary 1.48 km to the Roman mile that is 16.28 km of
  line, a ring of radius 16.28 / 2π = **2.591 km** — converted to cells by the
  map's own scale (`map_width_km / gw`). One attested work, used as the stated
  scale of "a siege line", not fitted to anything
  (`SIEGE_LINE_CIRCUIT_MILES = 11.0`, `ROMAN_MILE_KM = 1.48`).
- **Floor: one cell** (`SIEGE_RING_MIN_CELLS = 1.0`). A cell's circumradius
  is √2/2 ≈ 0.707 cells, so a tighter ring would cut through the cell it is
  meant to surround; one is the smallest whole-cell radius that clears it. It
  binds on coarse maps, above ~2.6 km per cell. A map with no usable scale has
  no radius (absent), not a default one.
- **Not scaled by the besieger's field army**, which the brief offered as a
  possibility. A faction's field army (§2.4) is its whole deployable force;
  how much of it sits at one siege is a placement rule — exactly the kind the
  derived garrisons (MM-6) will need to state — and nothing yet states it.
  Scaling the ring by the whole field army would draw Rome's entire field army
  around one town. When MM-6 lands, it can be revisited against that rule.
- **Revisited once MM-6 landed (2026-09-24): still not scaled — by the
  besieged garrison either.** §5.6 now states how many men hold the besieged
  place, so the question became whether the ring's radius should follow that
  number. It should not, on this scope's own sources: the radius is one
  attested work's stated circuit, and nothing this scope cites pairs a
  circuit with the number of people inside it. A scaling law needs at least
  that relation, and inventing one would fit the ring to nothing — the same
  objection this section raised against the field army. What the layer does
  instead is **carry the number**: each siege row gains `garrison`, the
  besieged settlement's §5.6 garrison in the cursor's year (absent when the
  siege names no place, or that place has no garrison that year), and the
  right dock's conflict view shows it as the defenders' starting force. The
  ring stays the stated scale of "a siege line".

### 5.2 What a front is drawn from

The **border between the conflict's sides** in the territory in force: every
pair of 4-neighbour cells where one side's cell meets a *different* side's
cell. Each pair is drawn as the cell edge they share, so the front is a
cell-exact line, not a smoothed guess. A border with a third faction, or with
unclaimed land, is not this war's front. The whole shared border is the front:
nothing in an authored conflict says which stretch of it is contested, and
cutting it to the drawn geometry's extent would be a rule invented here. A
conflict with one side has no front (an empty one, since the territory *was*
readable).

### 5.3 How "territory over time" is shown

The recorded snapshots, across the cursor. The layer adds one derived reading
on top of them: **the cells that changed hands between the conflict's sides
since it began** — every cell whose owner in the snapshot in force at the
conflict's *start* is one side, and whose owner in the snapshot in force at the
*cursor* is another. Both ends must be sides; a cell lost to a third faction or
to unclaimed land may be another war's or a recompute's doing, and is not
attributed to this one. Drawn as a wash in the conflict ink, so the war's
gains read against the Political territory layer rather than repainting it.
If nothing is recorded at or before the conflict's start there is no
baseline, and the changes are absent.

Nothing had to be added to the timeline for this: `TimelineSnapshot` already
records territory per year (`civ_territory_at` reconstructs any recorded
year), and the conflict already records its sides and years.

### 5.4 Visual language

No DCC canvas draws a siege line or a war front — the only siege in `design/`
is a settlement-timeline card (`settlement-editor-2026-09-21`) — so the layer
takes its vocabulary from the existing CARTO/CIVIL layers: SP-4's conflict
crimson (the hue no other civil layer uses) over the route layer's dark
two-pass underlay, told apart by shape as SP-4's four kinds are. The ring is
map-scaled (it grows with zoom, like a real work on a plate) where SP-4's
siege mark is pin-sized; its ticks face inward, the contravallation's side.
The layer sits under SP-4's own marks, so an authored annotation stays on top
of the war it annotates.

### 5.5 What stays out

- **No unit movement and no combat resolution.** The owner did not ask for
  them, and Ruling AW says to settle either with the owner before building it.
  Nothing here moves a force, decides a battle, or writes territory: the layer
  only reads what the author and the timeline already hold.
- **No new clock.** The cursor is `CivData::year` (`STORY_PLANNING_SCOPE.md`
  §5).
- **Garrisons (MM-6) and manpower across the cursor (MM-8)** are the ruling's
  other two items. They are not drawn by this layer; they are defined in §5.6
  and §5.7, and the layer reads only the besieged garrison (§5.1).

### 5.6 Per-settlement garrisons (MM-6)

Ruling AW: *"Each faction's standing army is split across its settlements by a
stated rule (walls, capital, border exposure)."* This is that rule. It splits
an existing number. It raises no soldier the model did not already count, and
it adds no simulation. The code is `cartalith_civ::garrison::civ_garrisons`.
The bridge reads it into `civ_military_summary[_at]` and
`civ_settlement_garrison`.

**What is split.** The faction's `standing_army` (§2.4), rounded to a whole
headcount. That is the figure CIVIL ▸ Military prints. It is not the field army
or the levy. Those are forces raised for a campaign, and §1 defines the
standing army as the one "continuously maintained". That is the force a
garrison is.

**The weight.** For settlement *i* of faction *f*:

```
weight_i = pop_i
         × ( 1 + (0.35 / 0.45) × walled_i
               + (0.20 / 0.45) × capital_i × tier_rank_i / 5 )
         × ( 1 + exposure_i )
```

Each term and its reason:

| Term | Value | Reason |
|---|---|---|
| `pop_i`, the base | the settlement's own population | **The model's own identity (§2.2).** The standing army is paid out of the non-agricultural population, and that population *is* the settlement sum (`nucleated_pop`). So each settlement's share of the payroll is its share of the base. With every other term at its neutral value, the split follows the money, and the rule assumes nothing further. |
| walls | `0.35 / 0.45` ≈ 0.778 more, when `um_infer_walls` says walled | **The reference's own military weights, moved from faction to settlement.** `_civFactionAggregates`' military axis is `0.45·normPop + 0.35·fortifiedFraction + 0.20·capitalTierNorm` (golden-verified, §0). Population is the base here, so walls weigh what the reference weighs them *relative to population*. `walled` is the ladder's own verdict (`cartalith_civ::military`), including the place editor's overrides. |
| capital | `0.20 / 0.45 × tier_rank / 5` more, on the faction's capital only | **The same formula's third term, on the same terms.** `capitalTierNorm` is the capital's tier rank over the table's top rank (5, `metropolis`), exactly as the aggregate computes it. The capital is the aggregate's own pick (`FactionAggregates::capital`), not a second choice. A walled capital therefore reaches `1 / 0.45` ≈ 2.22× its population weight: the reference's whole military score over its population part. |
| border exposure | `exposure_i` in 0…1, scale 1 | **Defined from territory; its scale is a modelling choice** (below). |

**Border exposure** is measured on the faction's frontier. The frontier is
every cell the faction holds that is 4-adjacent to a cell another *faction*
holds. Unclaimed land is not a frontier, because nothing in this model attacks
from it. On a wrapping map the seam counts, as in `relations`. Each frontier
cell is assigned to the faction's settlement nearest it: Euclidean distance
from the cell's centre, ties to the lower settlement index. A settlement's
`exposure` is its count divided by the largest count among the faction's
settlements. So the settlement answering for the most frontier reads 1, and a
settlement with no frontier nearest to it reads 0. This is relative within the
faction, the way `relations`' `border_fraction` is relative to the widest
border on the map. It needs no kilometre constant for "near the border".

**The scale of exposure is the rule's least-grounded number, said here as
§2.6 says of `oligarchy`.** At `1`, the faction's most exposed settlement weighs
twice what the same place would weigh in the interior. The multiplier can only
*add* weight: an interior place keeps its base, and exposure never takes men
from anywhere except through the normalisation every share goes through. No
source in this scope gives a frontier-to-interior garrison ratio. `1` is the
smallest whole-number scale that makes the term a clear multiple rather than a
rounding error. The owner may want to rule on it.

**The split: largest remainder (Hamilton), exactly.** With `S` the rounded
standing army and `W = Σ weight`:

1. Each settlement's quota is `S × weight_i / W`.
2. Each settlement takes the whole part of its quota.
3. The `S − Σ floors` men left over go one each to the largest fractional
   remainders. Ties go to the lower settlement index.

The parts sum to `S` exactly, by construction and by test, on every faction.
Other rules were weighed and rejected:

- **Rounding each quota** can miss the total by up to half the settlement count.
- **Giving the leftovers to the capital** is a second placement rule hidden in
  the arithmetic.

**Absent, never zero, where there is no reading.** These cases have no
garrison and show a dash with the reason:

- a settlement of no faction;
- a faction whose standing army is not a finite number;
- a faction with a positive standing army and zero total weight (every place
  at population 0). The rule cannot place that army, and spreading it evenly
  would be a second rule;
- a territory raster that is not the grid's size, where exposure cannot be
  read.

A real zero is a value: a village whose quota rounds down with no remainder
won is garrisoned by **0**.

**What this is not.** It is not a deployment. The same men are counted once, in
the faction's standing army, and a garrison says where they are *quartered*
when no campaign is running. Nothing moves them, and a conflict does not draw
on them. The siege row only *reports* the besieged place's garrison (§5.1).

**Tests** (`garrison.rs`) pin literal splits on four fixtures, including a
remainder tie. They also check:

- a walled capital on a border outranks an interior village of any population
  up to its own;
- a one-settlement faction puts everything there;
- each of the four terms moves the split when it is mutated.

### 5.7 Manpower across the year cursor (MM-8)

**Recomputed from the snapshot in force, not stored per snapshot**, because
all but one of the model's inputs is either recorded per year or does not vary
by year. Input by input, from `civ_military_bridge.rs`' `manpower_rows`:

| Input | Where the year's value comes from | Recorded per year? |
|---|---|---|
| settlements, their populations, factions and tiers (`nucleated_pop`, the capital, walls) | the snapshot's `settlements` | **yes** |
| territory (`territory_km2`, `land_capacity`'s cells, border exposure) | `civ_territory_at` on the snapshot | **yes** |
| roads (`capital_road_reach`, `road_density`) | the snapshot's `ways`, whose endpoint indices are into its own `settlements` | **yes** |
| land per cell (`CivData::dens`) | live. It is built from terrain (carrying capacity, water access, biome, rain), which no year changes | no, and it need not be |
| water access (`navigable_share`, `sea_share`) | live terrain, read at the snapshot's settlement positions | no, and it need not be |
| place-editor overrides (walls, age, traits, specialisation) | live, by `tid` | **no** |
| ag-tech and government (`farmers_per_urbanite`, `government`) | **the roster as it stands today** | **no** |

**The institutions are the gap.** The roster is not recorded per year, so a
year's reading uses today's ag-tech and government. Storing them would put a
new field on `TimelineSnapshot` and on the project format's `TimelineYearDto`
(`SAVEFILE_COMPAT.md` §10.1). That change was not made in this pass, because
another lane owned `project_bridge.rs` at the time. It is filed as a follow-up.
So no snapshot is missing anything this reading needs, and every older project
reads as before. **The readout says so on screen:** a recorded year's
headcounts are labelled with the year they were read from, and with the fact
that ag-tech, government and place edits are today's.

**Which year is read.** This is Ruling AT's model, and `campaign::year_in_force`
decides it, as it does for the Conflict layer:

- A recorded year reads its own snapshot.
- An unrecorded year reads the latest recorded year before it. A record holds
  until the next record changes it.
- A cursor before the first record, on a non-empty timeline, has **no
  reading**. The panel says which year is the earliest recorded.
- An empty timeline reads the live world, labelled as the world as it stands.
  Most worlds never record a year, and "nothing recorded" is not "no army".

**Live edits and the record.** A paint stroke or a new settlement after a year
was recorded is not in that year's reading until the year is recorded again
(Timeline ▸ Add year re-snapshots it). This is the price of reading the
record. The label says which year is being read.

**Wired.** CIVIL ▸ Military refills when the year in force changes. It does not
refill on every step of the cursor, because the resource-potential pass behind
the aggregate is too slow to run per frame. The garrison rows in the right dock
and the place editor read the same year.

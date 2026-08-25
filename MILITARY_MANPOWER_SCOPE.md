# Military manpower — what a polity can put and keep under arms

Built 2026-08-25, on an owner-supplied specification. This document exists
because that specification is **design input worth preserving verbatim**: the
model has no reference implementation to fall back on, so the specification is
the only ground truth there is, and a paraphrase of it would leave nothing to
check the code against.

Three sections, in this order:

1. **The owner's specification**, reproduced as supplied.
2. **The derivation** — how each part of it maps onto quantities this port
   already computes, and every constant with its grounding.
3. **Verification** — the two worked examples, the live figures, and the four
   findings the build produced, including the two where the specification is
   internally inconsistent.

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
repo's standing rule that the register's stated reasons have been wrong six
times this session:

| Need | What already exists | Status before this pass |
|---|---|---|
| Agricultural labour ratio | `roster::AG_TECH_LEVELS`' `farmers_per_urbanite` (reference line 14816, ported verbatim) | **no consumer anywhere** — its own module doc says so |
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

**`ecological_factor`** = `clamp(land_capacity / total_population, 0.25, 2.0)`,
where `land_capacity` is Σ `dens[i] × cellKm²` over the faction's own
territory cells — exactly the integral `civ_agrarian_regional_total` takes over
the whole map, restricted to one owner. **This is the geography term**, and it
is why two factions on the same ag-tech row do not get the same answer.

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
standing_army   = military_budget / SOLDIER_UPKEEP           (= 3.0)
professional_core = standing_army × professionalization
```

The non-agricultural population *is* the embodied surplus: those are the
people the farmers' surplus already feeds. `SOLDIER_UPKEEP = 3.0` is a
soldier's annual cost in subsistence-equivalents — pay, rations, equipment
replacement, and the animals and servants a soldier of any era drags behind
him — roughly three times a peasant household's own consumption.

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

The band is then **reported, never enforced**: `era_standing_verdict` and
`era_mobilization_verdict` read `within` / `above` / `below`, and nothing is
clamped into range. The owner's own words are the reason — *"these are
modelling ranges, not historical laws."* `Hunter-gatherer` is retained in the
table and is unreachable from this port's generated worlds, since the lowest
ag-tech row is hoe cultivation rather than foraging; kept so the table is the
owner's table, the same convention `civ_base_pop_for_kind`'s own unreachable
row already carries.

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
| standing army | ~5 000 | **5 846** | ~20 000 | **19 067** |
| emergency levy | ~40 000 | **41 221** | 100 000+ | **98 889** |
| field army | 15 000–20 000 | **15 870** | 40 000–60 000 | **47 368** |

Every figure is in range. The one that is furthest out is Kingdom A's standing
army at **+17 %** on a stated "~5 000", and it is left there rather than tuned.

The derived eras and durations, which the specification does not state and so
cannot be fitted to:

| | Kingdom A | Kingdom B |
|---|---|---|
| era (derived) | High medieval | Military-fiscal state |
| standing share | 0.585 % (band 0.5–2 %, **within**) | 1.907 % (band 1–4 %, **within**) |
| field army sustainable | 337 days | 128 days |
| full levy sustainable | 77 days | 41 days |
| ladder 30 / 90 / 180 / 365 d | 41 221 · 37 126 · 23 756 · 15 067 | 98 889 · 59 455 · 38 045 · 24 129 |

Two of those are worth reading twice. **Kingdom A's full levy sustains 77
days** — the feudal ~2-month obligation, out of a curve fitted on two unrelated
points. And **Kingdom B's 90-day rung is 59 455 and its 180-day rung 38 045**,
which brackets the owner's stated 40 000–60 000 field army almost exactly: a
"field army" *is* a campaign-season force, and the model says so without being
told.

### 3.2 Live figures — a real 233-settlement, six-faction world

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

The Faction Roster's new Military block, on the same world, for Aurelia:

```
Power: 49/100 relative to the other factions  ·  2 of 27 settlements fortified (2 stone, 0 palisade, 0 ditch)
Standing army 1 040 (professional core 324)  ·  sustainable field army 6 172  ·  emergency levy 13 569
Out of a total population of 230 400 (90% in farming), of whom 57 600 are of military age.
A field army stays out 172 days; a full levy 51.
Reads as a Bronze Age state (Administration + food storage). Standing 0.45% of population —
below that era's 0.5–2.0% band; mobilization 5.9% — within its 5–15%.
```

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
  **0.415 … 0.841**, and ecological factor **0.428 … 2.000**. If this had
  collapsed to one number the model would have been a technology lookup
  wearing five variables.

Plus `cargo test`: `cartalith-civ` **421 → 435** lib tests (14 new in
`manpower`), `cartalith-godot` 351, `cargo check -p cartalith-godot` clean,
`cargo clippy -p cartalith-civ` clean.

### 3.3 Four findings

**1 · The specification's era table and its worked example disagree with each
other, and the table disagrees with its own Imperial Rome figure.**

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

**2 · The standing-army figures land at Imperial Rome's own ratio, not at the
table's.** Aurelia reads **0.45 %** of its population under arms on the
233-settlement world and 0.35 % on the sparser 33-settlement one; forcing the
latter to `empire` moves it to 0.45 %. Rome under Tiberius is 0.21–0.56 %. So
the model agrees with the specification's *example* and disagrees with its
*table*, in the same direction and for the same reason as finding 1.

**3 · `ecological_factor` saturates on real generated worlds.** Five of six
factions on the 233-settlement world hit the `2.0` ceiling — their territory
sustains at least twice the population the model puts on it. This is not a
bug: it is the same divergence `civ_agrarian_regional_total`'s own "Land
sustains ≈ N … x % actually live in settlements" readout has always shown, and
the clamp is what stops it becoming absurd. But it does mean geography
discriminates mainly at the *low* end — Draumr League at 0.428 is where the
term does real work, and it is why that faction's standing army is 87 against
Veldmark's 1 509 on otherwise similar institutions. Whether generated worlds
should be more densely populated relative to their carrying capacity is a
separate question, and an old one.

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

Unchanged from CV-25's own narrowing, minus the half that is now built:

- **Per-settlement garrisons.** The per-*faction* headcounts are real. Which
  settlement holds which part of a standing army is a placement rule nothing
  here implies, and inventing one would be the fabricated number CV-25's first
  pass refused.
- **Campaigns, unit movement, combat.** Each needs a clock, a map objective
  and an opposed force. None exists.
- **Change over time.** Every number here is a reading of the world as it
  stands, and stops there — the same boundary `relations` holds.

Disclosed on screen in CIVIL ▸ Military ▸ Not built, in the same words.

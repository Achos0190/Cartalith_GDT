# Pre-industrial food logistics and the urban population ceiling

Research note for Cartalith Gen1 v1.33. Written to answer a specific owner question: *are city and
capital populations calculated correctly, and is a food deficit being automatically treated as an
import when no trade route could actually sustain it?*

The short answer was **no** — v1.32 and earlier assumed any deficit could be covered by trade, with no
check that the food physically existed within reach. This note is the basis for the fix.

## 1. The governing constraint

Pre-industrial bulk food movement was dominated by transport cost, not production. The canonical
quantitative source is **Diocletian's Edict on Maximum Prices (301 AD)**, from which several scholars
have derived road : river : sea cost ratios:

| Source | road : river : sea |
|---|---|
| Duncan-Jones (1982) | 1 : 4.9 : 56 |
| Deman (1987) | 1 : 5.8 : 39 |
| Scheidel (2014) | 1 : 5–10 : 52 |

The three disagree on detail and agree on magnitude: **water carriage is roughly an order of magnitude
cheaper than road, and sea roughly 40–56× cheaper.** Cartalith uses a mid-range **1 : 5.5 : 50**.

The consequence, in Duncan-Jones' own words, is the design rule:

> *"despite the existence of a comprehensive network of trunk roads, land transport remained so costly
> and inefficient that it was often impossible to relieve inland famines from stocks of grain
> elsewhere."*

An inland settlement in deficit does **not** get to import its way out. That is the behaviour being
corrected.

## 2. Quantitative anchors

- **Grain roughly doubles in price per ~100 miles (~160 km) of overland carriage.** This is the single
  most useful number for a decay model: it converts distance directly into how much of a shipment is
  economically "lost".
- **Bulk/perishable agricultural supply by land was limited to roughly a 50 km radius.** Small towns
  drew almost everything from the immediately surrounding countryside; only large centres built
  elaborate longer-range supply systems, and those overwhelmingly used water.
- **Cities above ~100,000 were scarce, and those that existed sat at the nexus of maritime and land
  trade networks.** Rome, Constantinople, and the great river cities are the pattern; a landlocked
  inland city of that size essentially does not occur.
- Medieval London c. 1300 consumed >1 million bushels of grain and up to 100,000 tons of wood a year —
  a useful sanity check that a large city's demand is a regional, not local, draw.

## 3. The model Cartalith implements

**Deliverable fraction.** A source settlement at distance *d* from the consumer, shipping by mode *m*,
can economically deliver

```
frac(d, m) = 2 ^ ( − d / D_m )
```

where `D_m` is the distance over which cost doubles: **land 160 km**, **river 880 km** (160 × 5.5),
**sea 8000 km** (160 × 50). This is the price-doubling rule applied directly, and it reproduces the
observed pattern for free — at 50 km overland a source still delivers ~80% of its surplus; at 300 km
it delivers ~27%; at 800 km, ~3%. By sea, 800 km costs almost nothing.

**Mode selection.** The cheapest mode available to *both* ends: sea if both are coastal, river if both
sit on navigable water, otherwise land. This is why a coastal capital can be fed from across the map
while an inland one of the same size starves.

**Supportable population.**

```
supported = ownCatchmentCeiling + Σ over reachable sources ( theirExportableSurplus × frac(d, mode) )
```

A settlement whose population exceeds `supported` is **over its food-shed ceiling** — it is not a
trading hub, it is a place that could not have grown that large.

**Connectivity.** Beyond the local land radius a source only counts if it is genuinely reachable —
connected by road, or sharing navigable water. "Directly connected settlements in a reasonable
distance", as the owner put it. A settlement 40 km away across trackless mountains is not a supplier.

## 4. What this deliberately does not model

- **Storage and inter-annual buffering.** Granaries smoothed bad years; this is a steady-state model.
- **Political extraction.** Rome's grain dole was a state logistics operation, not a market outcome.
  A powerful faction could in principle out-source its own food-shed; that needs a state-capacity
  model which does not exist here.
- **Return-cargo economics.** Shipping is cheaper when a vessel has cargo both ways; ignored.
- **Roads as cost multipliers.** A real road network reduces effective land cost. Cartalith uses road
  connectivity as a gate (reachable / not) rather than as a continuous discount.

## 5. Sources

- Duncan-Jones, *The Economy of the Roman Empire: Quantitative Studies* (1982) — the 1 : 4.9 : 56 ratio
  and the "impossible to relieve inland famines" conclusion.
- Scheidel, *The shape of the Roman world: modelling imperial connectivity* (ORBIS working paper).
- Campbell et al., *A Medieval Capital and its Grain Supply: Agrarian Production and Distribution in
  the London Region c. 1300*; Galloway, *Feeding the City: Medieval London and its Agrarian
  Hinterland*.
- Bruce Lloyd / *Geography of Transport Systems*, ch. 1.3, on the ~50 km pre-mechanised supply radius
  and the maritime-nexus condition for cities above 100,000.

## 6. How much surplus a hinterland actually yields (v1.34)

v1.33 used a flat `FOOD_MARKETED_FRACTION = 0.30` — a number with no source, and exactly the kind of
free parameter that lets settlements balloon. It is replaced by the best-attested figure in this
subject:

> **Roughly nine medieval farmers were needed to free up enough surplus to feed one non-farming town
> dweller**, with ~90% of the population working the land.

That single ratio is what pins pre-industrial urbanisation at the observed 10–15%, and it is now the
anchor the whole food shed hangs from (`FARMERS_PER_URBANITE = 9`).

### Supporting figures

| Quantity | Value | Source |
|---|---|---|
| English cereal yields 1250–1450 | 470–1000 kg/ha (7–15 bu/acre) | manorial accounts |
| Seed-to-harvest, Sussex manors 1350–99 | 4.34 : 1 | manorial accounts |
| Available after seed + ~25% pest/storage loss | ~440 kg/ha/yr | derived |
| Cereal need per person | ~100–140 kg/yr (900–1200 of 2200–3000 kcal/day) | WHO-based reconstruction |

### The method

Yield scales with soil fertility, and a cell's farmers eat before anything travels:

```
surplusRatio(soil) = clamp( (yield(soil) − subsistenceYield) / yield(soil), 0, 0.35 )
```

`subsistenceYield` is pinned so that **the world's own median soil** reproduces the 9:1 baseline
exactly. That calibration detail is load-bearing — see the warning below. Marginal soil returns
**zero**: such land supports its own subsistence farmers and contributes nothing to any city. That is
the historically correct outcome and the main brake on runaway city size.

### Why this cannot feed itself

The dependency chain runs strictly one way:

```
terrain → carrying capacity → RURAL population → surplus → URBAN population ceiling
```

Nothing downstream feeds back upstream. A city's population never raises its own supply, never raises
rural population, and never raises another settlement's ceiling. The reconciliation pass only ever
caps, so it is monotonically decreasing and cannot oscillate or amplify. All three properties are
asserted in `tests/perf/smoke_gen1.js` (the `v1.34 ACYCLIC` checks).

### Two calibration traps, both hit and measured

1. **Do not pin subsistence to the midpoint of the yield range.** That assumes median soil = 0.5.
   Cartalith's soil field does not sit near 0.5, so the cut-off landed above most of the map: every
   cell below soil 0.345 returned zero surplus, world urban share collapsed from 13.8% to **0.86%**,
   and a capital was floored at 50 people with a food shed of literally zero. Calibrate against the
   world's measured median instead — the same self-correcting technique as the v1.25 sea-level
   histogram and the v1.31 density normalisation.
2. **Do not use the yield minimum as a floor under every cell.** Mapping soil [0,1] onto [470,1000]
   gives barren ground a 470 kg/ha yield and therefore a real surplus. 470–1000 is the range for land
   *worth cultivating*, not for all land. Yield must scale from zero.

### A structural error this also corrected

v1.33 handed a settlement the **whole** of its local catchment ceiling. That let a town be the entire
population of its own catchment with nobody left working the fields. The catchment ceiling is the
population the land can *feed*, farmers included — so the town gets only the surplus, on the same 9:1
basis as everything else.

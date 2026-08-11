# Settlement Emergence — Capacity-Grounded Populations & Post-Collapse Recovery

*Design/research foundation for Cartalith's auto-populate system. Combines historical geography, agrarian
economics, settlement archaeology, transport geography, and post-collapse demographic-recovery theory.
Owner-supplied model (July 2026) + implementation research. Provenance tags: **[A]** peer-reviewed/standard
reference, **[B]** derived/secondary, **[D]** design abstraction.*

---

## 0. The core abstraction

A settlement is **not a fertile pixel** — it is an **energy-conversion system**. It exists when
`Food + Resource Extraction + Trade Value > Survival Cost`, and its population is bounded by two things:

1. **What the surrounding land can sustain** (agrarian carrying capacity of its catchment), and
2. **What the transport network lets it concentrate** (surplus circulated to exchange nodes).

The correct pipeline is therefore: **landscape → carrying capacity → population → economic nodes → transport
network → political geography → historical change** — not "place villages on fertile pixels."

This is the model Cartalith's auto-populate now follows (v0.81+): populations are **computed from carrying
capacity as a base calculation, automatically, and scale with the set map size** (persons/km² × real km²).

---

## 1. Implementation research — capacity-first (option 2) vs catchment × centrality (option 1)

Two candidate models were weighed for how a settlement's population is derived:

- **Option 1 — catchment × centrality:** population = local catchment carrying capacity, then × trade centrality.
- **Option 2 — capacity-first, derive tier:** population from carrying capacity, then classify tier by band.

**Neither pure form is realistic.** §9 of the owner model prescribes capacity-first (Step 2 carrying-capacity
map → Step 3 population → Step 4 rank into tiers), which is more grounded than the tool's old rank-then-assign
heuristic. **But §3 is equally explicit that a market town/city "is not the productive unit — it is the
exchange node."** A *pure* capacity model (option 2) therefore **misplaces cities**: they sit on trade nodes,
not the most fertile land, so their local catchment can't explain their size. Conversely a pure local-catchment
model (option 1) can't grow a hub on thin land into a real city.

**Applied result — the synthesis:** capacity-grounded populations as the base (option 2's foundation), split by
economic role:

- **Productive tiers (hamlet, village)** → **local catchment** carrying capacity × a surplus/nucleation
  fraction. They *are* productive units tied to local land.
- **Exchange tiers (town, city, capital, metropolis)** → also draw a **centrality-weighted share of a regional
  urban pool** (a fraction of the whole region's agrarian population that lives concentrated in the trade
  hierarchy). This captures §3/§4: cities import surplus from a wide hinterland via the network.

Everything is grounded in the carrying-capacity field × the **agrarian ceiling** and real km² catchments, so it
**depends on the set map size** and responds to the actual land (fertile valley vs marginal upland).

---

## 2. Agrarian carrying capacity (the productivity basis)

Pre-industrial population density is limited by solar energy capture and human labour: `Population ≈ ArableArea
× Productivity × LabourEfficiency`. Medieval simulation range **[A]/[B]** (Duby; Fossier; Campbell):

| Environment | Persons/km² |
|---|---|
| Poor / marginal land | 20–50 |
| Mixed farming | 50–150 |
| Fertile river valleys | 150–300+ |

**Implemented:** a cell's agrarian density = `carryingCapacity(K) × AGRARIAN_MAX_KM2`, with
`AGRARIAN_MAX_KM2 = 200` [B] (fertile-valley ceiling). K already encodes soil × climate × water, so prime land
(K≈0.6) ≈ 120/km² and marginal (K≈0.15) ≈ 30/km² — matching the table. *(This is distinct from the conservative
regional-average `estimateRegionalDensityKm2` "Pop density" layer, which includes all waste; the agrarian basis
is the settled-core productivity a nucleus actually draws on.)*

---

## 3. Settlement hierarchy & catchments (Christaller/Lösch, terrain-broken)

Per-tier catchment areas — midpoints of the owner model's §3.1 ranges (implemented as `_CIV_CATCHMENT_KM2`):

| Tier | Catchment (km²) | Doc population band |
|---|---|---|
| Hamlet | 6 | 30–150 |
| Village | 25 | 150–600 |
| Market town | 150 | 1,000–5,000 |
| Regional city | 800 | 5,000–50,000 |
| Capital | 1,400 | (large regional / admin) |
| Metropolis | 2,500 | imperial (Baghdad ~700k, Rome ~1M) |

**Formula (implemented, `_civSettlementPopulation`):**
```
nucleus = meanK(catchment) × AGRARIAN_MAX_KM2 × catchmentKm2 × surplusFraction × (1 + normBetweenness × tradeK)
exchange tiers additionally: max(nucleus, urbanPool × centralityShare)
```
- `surplusFraction` (nucleation rate): hamlet 0.65, village 0.55, town 0.16, city 0.12, capital 0.11, metro 0.10 [D].
- `tradeK` (centrality lift): 0.25 / 0.5 / 1.1 / 1.7 / 1.9 / 2.1 [D].
- `urbanPool = agrarianRegionalTotal × _CIV_URBAN_SHARE`, `_CIV_URBAN_SHARE = 0.09` [D] (pre-industrial
  urbanisation into the trade hierarchy was ~5–15% incl. market towns).
- `_CIV_POP_CAP` per tier keeps the hierarchy on huge maps (town ≤ 15k, city ≤ 250k, …) where the ~40-settlement
  cap under-samples the region.

**Calibration (browser, region worlds):** at 400/800/2000 km map widths the tiers land in-band and scale
strongly with map size — capital ≈ 5.5k → 9.7k → imperial cities 60–140k; villages/hamlets stay stable local
values; regional settled total scales with area. Map-size dependency confirmed.

---

## 4. Transport geography (already in the tool)

Settlement spacing is governed by travel *time*, not distance: `TravelTime = Distance ÷ TerrainAdjustedSpeed`
(paved 5–6 km/h · dirt 3–5 · forest 2 · mountain <2 · swamp <1). Hamlet→village ≈ 30–90 min; village→town ≈ 2–5 h;
town→town ≈ a day. Cartalith's economic Dijkstra routing (v0.73), transfer/handling overhead (v0.78), and the
betweenness-centrality network feed the exchange-node concentration above.

---

## 5. Post-collapse recovery model (v0.82 workstream — begun)

Collapse does not reset the landscape: roads, ruins, fields, wells, bridges, mines, irrigation persist, but
**infrastructure decays faster than ecological knowledge**, so a recovery population inherits *excess buildings,
insufficient labour, damaged ecosystems*. Recovery runs **below** the ecological ceiling and must rebuild the
institutions that convert land into civilisation.

| Phase | Population vs former | Settlement logic |
|---|---|---|
| **I Survival** | <10% | resource extraction; cluster on water, preserved food, defensible ruins, surviving infrastructure |
| **II Subsistence** | 10–30% | food security; repopulate abandoned villages; labour shortage → a former city survives only as a village |
| **III Regional** | 30–70% | crafts/markets/roads/politics return; old hierarchy re-emerges |
| **IV Mature** | 70%+ | prior economic geography largely returns, but some sites vanish permanently (resources/trade/borders shifted) |

**Recovery does not delete settlements** — it re-scores them:
`SettlementValue = Infrastructure + AgriculturalPotential + WaterAccess + StrategicPosition − MaintenanceCost`.
A ruined city (high infrastructure, low food supply) becomes a **small fortified settlement inside the ruins**.

**Planned implementation (`state.civ.recoveryPhase` I–IV):** population × `phaseFraction` (<0.1 … 0.7+), a
labour-shortage cap that demotes over-large nuclei toward what the phase's labour can sustain, and a settlement-
value bias toward ruins/water/infrastructure. Growth is surplus-gated (`FoodSurplus + Security + Trade >
PopulationPressure`). Multi-version; foundation laid in v0.81's capacity-grounded populations.

---

## References

Christaller, W. (1933) *Die zentralen Orte in Süddeutschland*. Lösch, A. (1940) *The Economics of Location*.
Duby, G. *Rural Economy and Country Life in the Medieval West*. Campbell, B. M. S. *The Medieval World*.
Fossier, R. *The Cambridge Economic History of Europe*. Bairoch, P. *Cities and Economic Development*.
Boserup, E. *The Conditions of Agricultural Growth*. Diamond, J. *Collapse*. Tainter, J. *The Collapse of
Complex Societies*. Scott, J. C. *Against the Grain*.

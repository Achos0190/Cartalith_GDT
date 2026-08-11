# Collapse Timeline Dynamics — Mortality, Migration, and Settlement Failure Over Time

*Research + design foundation for a mechanistic, year-by-year collapse/recovery simulation that writes
directly into Cartalith's existing civ timeline. Builds on `settlement-emergence.md` (capacity-grounded
populations, v0.81; static recovery-phase snapshots, v0.82). Provenance tags: **[A]** peer-reviewed/standard
reference, **[B]** derived/secondary, **[C]** tertiary, **[D]** design abstraction with no direct empirical
source.*

---

## 0. What v0.82 already does, and the gap this closes

v0.82's Recovery phase is an **instant re-weighting**: pick "Survival," auto-populate applies a phase
fraction and demotes over-large nuclei, once. It answers *"what would a Survival-era world look like"* but
not *"how did it get that way, year by year — which settlements failed first, and where did the survivors
go."* That is what was asked for here: a **process**, not a snapshot, that writes its steps into the
timeline so it can be scrubbed like history.

---

## 1. The three variables that govern any settlement collapse

Real settlement-system collapses — the Late Bronze Age collapse, the post-Roman West, the Black Death, the
post-1492 Americas — differ enormously in cause but are all constrained by the same three quantities
**[A]** (synthesis position across the collapse/demography literature; see References):

1. **Energy/food availability** — can the population still eat, locally, without trade?
2. **Transport/trade capacity** — is the network that concentrated surplus into exchange nodes still
   functioning?
3. **Carrying capacity of the (possibly degraded) landscape** — has the resource base itself changed?

Cartalith already computes (1) and (3) as `carryingCapacity(K)`/`_civSettlementPopulation`, and (2) as the
betweenness/closeness/component-size network metrics (`_civNetworkMetrics`) that already drive exchange-tier
population in v0.81. The collapse stepper below is built entirely from fields the engine already computes —
no new global state, just a new *process* over them.

---

## 2. Which settlements fail first — the network-vulnerability literature

The naive assumption — "small settlements fail first, big ones last" — is only correct for **one** collapse
archetype. Real history gives (at least) three distinct failure orders, and the mechanism for each is
citable:

### 2a. Trade-collapse archetype (Late Bronze Age Collapse style)

Cartalith's v0.81 model makes exchange-tier population (town/city/capital/metropolis) partly **trade-derived**
— drawn from a regional "urban pool" concentrated by network centrality, not purely local land. When the
trade network's *total throughput* contracts, the settlements most dependent on that pool are hit hardest,
even though they were the biggest. This is exactly the mechanism historians attribute to the Bronze Age
collapse: palace-economy administrative centers (highly trade/tribute-dependent) disappeared abruptly while
small rural/subsistence settlement continued with much less disruption **[B]/[C]** (Cline, *1177 B.C.: The
Year Civilization Collapsed*, 2014 — a synthesis of the archaeological consensus, not itself primary data).

Formally, this is the *inverse* of scale-free network robustness: real infrastructure/trade networks are
famously robust to random node loss but fragile to **targeted removal of high-centrality hubs** (Albert,
Jeong & Barabási, *Error and attack tolerance of complex networks*, **Nature** 406, 2000 — **[A]**). A
resource-starvation collapse behaves like exactly that attack, but self-inflicted: as low-order peripheral
nodes fail, the survivors' recomputed betweenness redistributes — some hubs *gain* centrality (fewer
competing paths), others lose the traffic they depended on. Re-running Brandes betweenness
(`_civNetworkMetrics`, already in the engine) after every failure captures this for free — it is the
*correct* graph-theoretic consequence of node removal, not a hand-tuned heuristic.

### 2b. Disease archetype (Black Death style)

Epidemiological mortality scales *with* density and connectivity, not against it: the best-documented
pre-industrial demographic catastrophe, the Black Death (1347–1351), hit dense, trade-connected cities
(Florence, Venice, Paris) **harder**, not less hard, than isolated rural villages — plague travelled the same
trade routes that built those cities **[A]** (Benedictow, *The Black Death 1346–1353: The Complete History*,
2004; ~30–60% mortality across Europe in roughly four years is the standard modern estimate). This is the
*opposite* ranking from 2a: here, the highest-connectivity, highest-density settlements are the *most*
exposed, not the most protected.

### 2c. Conflict archetype (raiding / migration-period style)

Undefended settlements fail first regardless of size; fortified or walled centers persist far longer because
defensibility, not economic role, is the survival variable — the standard account of the post-Roman West's
rural villa system collapsing while walled cities and hilltop *castra* persisted into the early medieval
period **[B]** (Wickham, *Framing the Early Middle Ages*, 2005).

**Design conclusion:** a single "stress" formula with a fixed settlement-size ranking cannot represent all
three histories. The model below computes stress from three *independently weighted* components, and a
**Collapse character** dial (Disease / Trade collapse / Conflict / Mixed) sets the weights — so which
settlements fall first is a genuine, historically-grounded choice, not a hard-coded assumption.

---

## 3. Per-step settlement stress score

For settlement *i* at simulation step *t*, three components, each normalised to roughly [0,1]:

- **Trade-dependency loss** `L_i = clamp(1 − normBetweenness_i(t) / normBetweenness_i(0), 0, 1)` — how much
  of this settlement's *original* network centrality it has lost as neighbours have failed. Recomputed from
  the live (currently-alive) settlement/way graph every step via the engine's existing Brandes betweenness —
  this is what encodes cascading contagion (§2a): a settlement doesn't need to be touched directly to suffer,
  only to lose the trade partners it depended on.
- **Density/connectivity exposure** `D_i = 0.5·normBetweenness_i(t) + 0.5·(pop_i(t)/maxPop(t))` — how
  exposed this settlement is to a *spreading* shock (§2b): still-well-connected, still-large settlements score
  high here (the Black-Death direction — the opposite sign from `L_i`). Population rank is used as a cheap
  proxy for density rather than a separate raster lookup, keeping the per-step cost independent of grid
  resolution.
- **Defensibility exposure** `V_i = fortified_i ? 0.3 : 1.0` (a settlement with the `fortified` trait — or a
  metropolis/capital, which are fortified by convention — is far less exposed to violence-driven collapse;
  **[D]**, direction well attested per §2c).

**Character weights** (Disease / Trade collapse / Conflict / Mixed), each a `(wL, wD, wV)` triple summing to 1
— **[D]**, calibrated to reproduce the qualitative rankings in §2a–2c, not measured from data (no numeric
per-character weighting scheme exists in the literature to cite — the *direction* of each term is cited
above, the specific blend is a design choice):

| Character | wL (trade-loss) | wD (density/connectivity) | wV (undefended) |
|---|---|---|---|
| Trade collapse | 0.70 | 0.05 | 0.25 |
| Disease | 0.05 | 0.70 | 0.25 |
| Conflict | 0.15 | 0.05 | 0.80 |
| Mixed (default) | 0.35 | 0.25 | 0.40 |

`stress_i(t) = wL·L_i + wD·D_i + wV·V_i`, clamped to [0,1].

---

## 4. Stress → mortality and migration rates

A single **severity** dial (0–1, "how bad is this collapse overall") scales two rate ceilings, both
calibrated against real historical crisis mortality **[A]**:

- **Excess mortality ceiling** `maxMortality ≈ 0.15`/step(year). Derivation: the Black Death's ~30–60%
  mortality over ~4 years implies an annualised rate `r` solving `(1−r)^4 = 1−0.45` (using the ~45% midpoint
  estimate) → `r ≈ 1−0.55^0.25 ≈ 0.136` — i.e. the single worst-recorded pre-industrial demographic
  catastrophe ran at **≈14%/year**. `maxMortality=0.15` is set just above that as the ceiling a
  maximum-severity, maximum-stress cell can approach; ordinary "bad" collapses run at a small fraction of it
  once scaled by `severity·stress`.
- **Out-migration ceiling** `maxMigration ≈ 0.25`/step (of *survivors*, applied after mortality) — the flight
  side of the same crisis. **[D]**-calibrated (no single citable rate exists across cases as varied as this),
  but the mortality:migration *ratio* is character-dependent and each end is grounded: displacement-dominated
  crises (war, ethnic conflict) push much higher migration relative to mortality than in-place crises
  (disease, famine) — a standard distinction in refugee/humanitarian demography **[B]** (general
  displacement-vs-mortality framing; also implicit in Ravenstein's original migration laws, which root
  migration in economic/security push factors — Ravenstein, *The Laws of Migration*, **J. Royal Statistical
  Society** 48, 1885, **[A]**).

Per year, per settlement (both rates are **annual** — that's what the calibration above anchors):
```
m_i = maxMortality · severity · stress_i                         (annual excess-death rate)
g_i = maxMigration · severity · stress_i · characterMigrationBias  (annual out-migration rate, of SURVIVORS)
```
A simulation **step** spans `stepYears` (UI default 10), so the step compounds the annual rates —
mirroring the recovery stepper, which compounds logistic growth the same way:
```
survivors_i = pop_i · (1 − m_i)^stepYears
migrants_i  = survivors_i · (1 − (1 − g_i)^stepYears)
stayers_i   = survivors_i − migrants_i
```
(Separable approximation: mortality is applied for the whole step before the migration split, so people
who would have left mid-step still face home-settlement mortality for the full step — a slight
overcount of deaths / undercount of migrants, acceptable at these rate magnitudes.)
`characterMigrationBias`: Conflict = 1.4× (flight-dominated), Disease = 0.6× (in-place, historically lower
flight — quarantine/immobility during plague is itself documented, though not universal), Trade
collapse/Mixed = 1.0×.

---

## 5. Migration — the gravity model

The founding empirical result in migration geography, still the standard first-order model **[A]**: migration
flow between two places is proportional to origin size and destination attractiveness, and falls off with
distance — Zipf's `P₁P₂/D` hypothesis (Zipf, *The P1 P2/D Hypothesis: On the Intercity Movement of Persons*,
**American Sociological Review** 11, 1946) generalising Ravenstein's 1885 *Laws of Migration*. The
distance-decay exponent is empirically **1–2** across replications **[B]**; this model uses **β=1.5** (a
literature-typical middle value, not independently fit).

```
Attractiveness_j = max(0, catchmentCeiling_j − pop_j) · (1 + fortifiedBonus_j)
Flow(i→j) ∝ Attractiveness_j / Distance(i,j)^1.5        for every SURVIVING j ≠ i
migrants_i redistributed across all j in proportion to Flow(i→j), capped by each Attractiveness_j
```
- `Distance(i,j)`: straight-line grid distance converted to real km via the engine's standard `cellKm =
  mapWidthKm/GW` (already used throughout the engine for exactly this purpose) — a tractable approximation of
  true travel-cost distance, standard practice in gravity-model applications where a full route-cost matrix
  isn't cheap to recompute every step **[D]**-simplification of a **[A]**-grounded model form.
- `fortifiedBonus_j`: refugees preferentially seek the security of larger/fortified centres during collapse —
  well documented for late-antique population concentrating into walled cities and hilltop *castra*
  **[B]** (Wickham 2005, as above). Modelled as +0.5 for fortified/city+ tiers.
- **Overflow**: migrants beyond the *system's total remaining headroom* (Σ Attractiveness) are not placed —
  tracked as **transit/diaspora loss**, a real and heavily documented phenomenon in famine/refugee
  demography (Irish Famine emigration mortality, forced-migration mortality generally) **[B]/[C]**, but the
  specific choice to model it as an unplaced statistic rather than seed new settlements is a **[D]**
  simplification, flagged as a deferred extension (§8).

---

## 6. Tier re-derivation and settlement failure

After mortality + migration redistribution, every settlement's tier is **re-derived from its new population**
using the existing `_civTierForPopulation` (v0.82) — a nucleus that has shrunk below its tier's floor
demotes exactly as it does in the static recovery-phase pass (former city → village, `ruins`/`fortified`
applied on a demotion out of an exchange tier). A settlement whose population falls below an **abandonment
floor** (`pop < 20`, matching v0.82's existing floor convention) is removed from the roster outright — its
last inhabitants already accounted for by the migration step. This is what makes settlements *disappear* from
the timeline, which the existing tid-diff ghost/highlight/exist-only overlay already visualises with **no new
rendering code** (§7).

---

## 7. Wiring into the existing timeline — no new data model needed

Cartalith's civ timeline (`civTimeline`, an array of `{year, territory, places, ways}` snapshots) already
supports exactly this shape — `civAddYear`/`civSnapshotSave` write entries in precisely this format today, by
hand, one year at a time. The simulator's only integration job is to **run the per-step math N times and push
one snapshot per step**, in the same format. Consequences:

- The existing timeline slider, "existence" filter, and ghost/highlight overlays (tid-diff based, §
  `_civYearDiff`) work on simulated history with **zero new rendering code** — scrubbing through a simulated
  collapse looks exactly like scrubbing through manually-authored history, because it *is* the same data
  structure.
- Per the engine's existing invariant (documented at the civTimeline definition site): jumping to a
  recorded year overwrites live *territory* but **never** touches `state.places`/`civWays`, which stay the
  single always-current, always-editable arrays used by pathfinding/planning. The simulator follows the same
  rule — it writes history, it does not silently rewrite the live, currently-being-edited world. (A "restore
  this year to live editing" action does not currently exist for *any* timeline entry, manual or simulated;
  out of scope here, noted as a natural follow-up.)

---

## 8. Deferred (documented, not built here)

- **New settlement founding from displaced/transit populations** (refugee camps growing into new hamlets) —
  the overflow/diaspora loss in §5 is currently just lost, not placed; a real and interesting extension.
- **True travel-cost migration distance** (reusing the mixed-cost Dijkstra grid instead of Euclidean) — more
  accurate, much more expensive per step; worth it only if Euclidean proves visibly wrong in practice.
- **Regrowth-phase migration** (people moving *back* toward reviving hubs during recovery, not just uniform
  logistic growth in place) — the recovery-mode stepper here is population-only; migration-driven
  resettlement patterns during recovery are a distinct, later piece.

---

## References

Albert, R., Jeong, H., & Barabási, A.-L. (2000). Error and attack tolerance of complex networks. *Nature*,
406, 378–382.
Benedictow, O. J. (2004). *The Black Death 1346–1353: The Complete History*. Woodbridge: Boydell Press.
Cline, E. H. (2014). *1177 B.C.: The Year Civilization Collapsed*. Princeton University Press.
Ravenstein, E. G. (1885). The Laws of Migration. *Journal of the Royal Statistical Society*, 48(2), 167–235.
Wickham, C. (2005). *Framing the Early Middle Ages: Europe and the Mediterranean, 400–800*. Oxford University
Press.
Zipf, G. K. (1946). The P1 P2/D Hypothesis: On the Intercity Movement of Persons. *American Sociological
Review*, 11(6), 677–686.
Chambers, J. D. (1972). *Population, Economy, and Society in Pre-Industrial England*. Oxford University
Press. *(post-plague regrowth rates, §recovery mode.)*

*Same tiering convention as `settlement-density.md`/`settlement-emergence.md`: [A]/[B] are calibration
targets, [D] are tunable design defaults, nothing invented is dressed up as sourced.*

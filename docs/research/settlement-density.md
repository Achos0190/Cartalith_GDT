# Settlement Density — Engine Formulas
### Companion to `settlement_density_reference.md` — turns the cited numbers into drop-in Cartalith code

# Settlement Density in Pre-Industrial Civilisations
### A sourced reference for Cartalith — population density, carrying capacity, settlement hierarchy, and transport cost, with biome context

*Compiled July 2026. Every quantitative claim is tagged by provenance (see tiering below). This document was written to replace invented values with citable ones, per Cartalith's "invented values are technical debt" principle.*

---

## How to read this document

Two distinct measures are constantly conflated in popular worldbuilding sources, and keeping them apart is the single most important thing in this file:

- **Intra-settlement density** — people per hectare *of the built-up settlement itself*. High (dozens to hundreds per ha). Used to convert a mapped site footprint into a population. Section 1.
- **Regional / landscape density** — people per km² *averaged over a whole territory*, including all the farmland, forest, and waste between settlements. Low (single digits to low tens per km² for pre-industrial agrarians). Used to populate a region or set carrying capacity. Section 2.

A "sustainability categories" table that lists "80,000–150,000 people per 1,000 km²" (= 80–150/km²) for a "Kingdom Core" is quietly using the *regional* frame but with numbers closer to the *intra-settlement* frame. Real prosperous medieval kingdoms sat at single digits to ~40/km² regionally. Both numbers are legitimate; they just answer different questions.

### Provenance tiers

| Tag | Meaning |
|-----|---------|
| **[A]** | Peer-reviewed primary research, or a standard scholarly reference work. Load-bearing. |
| **[B]** | Scholarly secondary synthesis, or a defensible derived estimate (e.g. a modern recalculation from primary population totals). |
| **[C]** | Tertiary or popular source. Directionally useful, not independently verified. Use with caution. |
| **[D]** | **Design abstraction** — an invented modeling convenience with no direct empirical source. Perfectly legitimate for a simulation; simply *not* an academic claim and must never be cited as one. |

---

## 0. Verification note on the source "Framework" document

The pasted framework document (settlement hierarchy, carrying-capacity formula, trade-access points, hub score, security formula) repeatedly cites Lawrence et al. 2016 (`pone.0152563`). **The full text of that paper was read; it contains none of those formulas, point-value tables, or ruleset numbers.** It is a climate-versus-settlement-size correlation study for the northern Fertile Crescent — no "Trade Access = Coast Bonus + River Bonus…", no carrying-capacity multiplier ranges, no security scoring.

What the framework document borrows correctly from the paper:
- The causal chain *Climate → Water → Food → Carrying Capacity → Population → Surplus → Trade → Urbanization* is a fair paraphrase of the paper's framing — **but the paper's actual thesis is that this chain breaks down.** After ~2000 BC, settlement and city size *decouple* from climate; growth is thereafter sustained by irrigation, transport networks, taxation, and administration rather than local rainfall. A simulation built on the chain alone reproduces the pre-2000 BC world the paper treats as the *simple* case, and misses the institutional decoupling that is the paper's whole point. **[A]**
- The era-by-era settlement size ceilings (Section 3 below) *are* genuinely from the paper and are safe to cite. **[A]**

Everything in the framework tagged to that paper but not listed above should be re-labelled **[D]** in your own notes. The ideas may be good design; the citation is decorative.

---

## 1. Intra-settlement density (people per hectare of built-up area)

Use this to turn a site's mapped area into a population estimate: `population ≈ built_area_ha × density_per_ha`. The method traces to Naroll's floor-area/population work and is the backbone of settlement archaeology. **[A]** (Naroll 1962; De Roche 1983)

| Context | Density (persons/ha) | Source | Tier |
|---|---|---|---|
| Contemporary farming villages, Fars, Iran (working conversion factor) | **160** | Sumner 1989 | [A] |
| Zagros village clustering (review of methods) | **100–150** | Kramer 1982 | [A] |
| Middle-range village systems, ethnographic + archaeological range | **30–500** (geometric mean ≈ **120**) | Duffy 2015 (via Nowak/Trypillia synthesis) | [A]/[B] |
| Bronze Age tell *on the mound* (Great Hungarian Plain) | **~220** | Duffy 2014 | [A]/[B] |
| Same system, settlement *outside* the fortified tell | **~80** | Duffy 2014 | [A]/[B] |
| Ancient Mesopotamian urban core (from building footprints) | **~150** (≈ 15,000/km²) | tertiary summary | [C] |

**Practical takeaway for Cartalith.** A single global conversion factor of **~150 persons/ha** is the mainstream default (Sumner/Kramer). If you want more fidelity, use a *higher* figure for dense fortified cores and a *lower* one for dispersed/rural footprints — the tell-vs-off-tell 220/80 split is a clean, real-world example of exactly that gradient. Every serious archaeologist warns this factor varies with culture, period, and building type (Kramer's own caveat), so treat it as a tunable, not a constant.

---

## 2. Regional / landscape density (people per km² of territory)

This is the number your "settlement distribution by region" and "sustainability categories" tables actually need. There is **no clean peer-reviewed "density by biome" table for agricultural societies** — practitioners build it from regional case studies and gridded reconstructions. The anchors below are real; the biome interpolation between them is flagged.

### 2a. Empirical anchors

| System / region | Regional density | Source | Tier |
|---|---|---|---|
| Hunter-gatherer, global mean at LGM (Binford frames) | **0.12/km²** (median 0.044) | Binford 2001; Prehistoric demography synthesis | [A]/[B] |
| Hunter-gatherer, richest biomes (temperate/subtropical forest, salmon coasts) | up to **~0.5–1/km²** | Tallavaara et al. 2018 | [A] |
| **Agricultural societies vs. most abundant HG** | **~100× denser** | Burger & Fristoe 2018 | [A] |
| Early-medieval Britain & Germany | **2–5/km²** | Chapelot & Fossier 1985 | [A] |
| Carpathian Basin, 1100 → 1440 | **3.6 → 10.3/km²** | via medieval-demography synthesis | [B] |
| France, c.1000 → 1300 (peak) | **~14 → ~26/km²** | derived from Russell 1972 | [B] |
| Italy, c.1300 | **~23–37/km²** | derived from Russell 1972 | [B] |
| Roman Italy (classical Mediterranean dry-farming), Augustan | **~15–24/km²** (rural ~15–20; Latin West ~17) | Beloch 1886 via Scheidel 2001; Frier 2000; Hin 2013 | [B] |
| England & Wales, c.1300 | **~4–12/km²** | derived from Russell 1972 | [B] |
| Scotland, c.1300 | **~1.5–3/km²** | derived from Russell 1972 | [B] |
| Scandinavia, c.1300 | **~0.4–1.5/km²** | derived from Russell 1972 | [B] |
| Low Countries (Belgium) peak, c.1500 — pre-industrial European ceiling | **~49/km²** (127/sq mi) | derived from Russell 1972 / Stone 2017 | [B] |
| Irrigated floodplain systems (pharaonic Egypt, intensive rice) — *local*, not regional average | **~100–200/km²** | tertiary; treat as illustrative upper bound | [C] |

> Unit note: `1 km² = 100 ha`; `1 person/sq mile = 0.386 persons/km²`. Several sources report per-square-mile; converted figures are marked derived.

### 2b. A usable regional-density ladder (synthesised)

This ladder is **[B]/[D]** — the endpoints are anchored to the table above; the intermediate tiers and the biome column are interpolation and design judgment, not measurements. It is offered as a *replacement* for the framework's unsourced "sustainability categories," calibrated to real numbers.

| Tier | Regional density (persons/km²) | Typical biome / subsistence context |
|---|---|---|
| Empty frontier / marginal forage | **< 0.1** | tundra, true desert, dense pathogen-rich rainforest (HG only) |
| Rich forage / incipient horticulture | **0.1 – 1** | temperate & subtropical forest, productive coast (HG at carrying capacity) |
| Shifting / early agriculture | **1 – 5** | cleared temperate woodland, savanna edge, early-medieval northern Europe |
| Established dry-farming agrarian | **5 – 15** | Mediterranean & temperate rain-fed cereal country (~the 500 mm rainfall zone) |
| Prosperous / intensive agrarian core | **15 – 40** | high-medieval France/Italy, well-watered river valleys |
| Pre-industrial ceiling | **40 – ~50** (regional) | Low Countries c.1500; irrigated floodplains reach higher *locally* |

**Do not push a regional average above ~50/km² for a rain-fed pre-industrial setting.** Densities beyond that in the real record are either (a) local floodplain/irrigated pockets, not territorial averages, or (b) already leaning on the transport-and-taxation "decoupling" that lets a core import food from a wide hinterland (Section 5, and the Lawrence thesis).

### 2c. The gold-standard gridded source

For a per-cell historical population layer (directly analogous to Cartalith's raster grids), the standard academic reconstruction is **HYDE** (History Database of the Global Environment), covering 10,000 BCE onward, hosted with processing by Our World in Data. **[A]** (Klein Goldewijk et al. 2017; HYDE 3.3, 2023). If the climate/settlement engine ever wants a real calibration target for its output density fields, HYDE is it.

---

## 3. Settlement hierarchy and size thresholds

### 3a. Era ceilings — real, from the uploaded paper **[A]**

Maximum typical settlement size in the northern Fertile Crescent, from Lawrence et al. 2016. These are excellent hard anchors because they show *ceilings breaking* as political scale grows:

| Era | Max typical settlement | Notes |
|---|---|---|
| Neolithic / Ubaid | **< 20 ha** | pre-urban |
| Late Chalcolithic (c. 4400–3400 BC) | **40–60 ha** | rare "plumes" to 130–300 ha |
| Early Bronze Age (to c. 2000 BC) | **100–130 ha** | a genuine plateau, seen region-wide and globally for large prehistoric agrarian settlements |
| Middle / Neo-Assyrian (from ~1300 BC) | **250–750 ha** | the 120–130 ha ceiling is *broken* here — Kar-Tukulti-Ninurta 250 ha, Erbil 330 ha, Nineveh 750 ha |
| Classical / Islamic | **1,000–7,000 ha** | Samarra & Baghdad reach 4,500–7,000 ha; Baghdad held **≥ 280,000 people** |

The paper's aggregate *settled-area* density (settled hectares per km² surveyed, **not** population) ranged **~0.2 to ~1.3 ha/km²** across 8,000 years — a real, citable gradient if you ever want continuous values instead of discrete tiers. **[A]**

### 3b. The Farmstead → Metropolis table

The framework's population bands (Farmstead 5–50 … Metropolis 500,000+) are a **[D]** generalization, but a *reasonable* one — the top bands land in the right ballpark for the Lawrence ceilings above. Keep it, tag it [D], and treat the Lawrence era-ceilings as the empirical check on its upper end.

### 3c. Grounded generators for *why* settlements distribute as they do

If Cartalith ever procedurally places settlements, these are the real, load-bearing models — far more defensible than a bespoke "Hub Score = Flow × Security × Surplus × Uniqueness" **[D]**:

- **Rank-size rule** — the nth-largest settlement is roughly `largest / n` in size (2nd ≈ ½, 3rd ≈ ⅓…). Origin: Auerbach 1913; formalised by Zipf 1949. A system dominated by one outsized "primate city" is the diagnostic *exception*, and that deviation is itself informative. **[A]**
- **Central place theory** (Christaller 1933; Lösch 1940) — settlement spacing follows *threshold* (minimum customers to sustain a service) and *range* (max distance a customer will travel). High-order services exist only in large, widely-spaced centres; low-order goods support small, frequent settlements. **[A]**
- Both are active tools in real Near Eastern archaeology, used to reconstruct cycles between small city-states and large territorial polities from the 4th–1st millennium BC. **[A]** (note: site-size hierarchy is a *contested* proxy for political hierarchy — Duffy 2015 shows societies with hierarchical site sizes but no central political control, so don't treat a rank-size gradient as automatic evidence of a state.)

---

## 4. Carrying capacity and biome

The framework's `Carrying Capacity = Climate × Water × Soil × Technology × Political Stability`, with factor ranges, is a **[D]** multiplicative abstraction. It's a fine *engine shape*, but the real research says the underlying relationships are non-linear and interacting, not a clean product of independent scalars. Anchors:

- **Net primary productivity (NPP) is the dominant environmental driver** of pre-agricultural human density; **subtropical and temperate forest biomes provide the highest carrying capacity** for hunter-gatherers. Biodiversity matters mainly in *low*-productivity regions; pathogen stress mainly in *high*-productivity ones. **[A]** (Tallavaara et al. 2018)
- The NPP→density relationship is **not** a clean scalar: hunter-gatherer density shows roughly **1,000-fold variation per unit NPP**, much of it explained by a **seasonal "carnivory bottleneck"** (how much of the year requires animal rather than plant calories). Annual NPP alone is therefore a weak carrying-capacity proxy. **[A]** (*Global hunter-gatherer population densities…seasonality*, Nat. Ecol. Evol. 2021)
- The agricultural transition multiplies sustainable density by **~100×** over the richest foragers, even without fossil fuels — a concrete "technology" jump, far better anchored than an arbitrary 0.5–5.0 slider. **[A]** (Burger & Fristoe 2018)
- For agrarian societies, **food availability is the empirically dominant limiting factor**, with negative density-dependence (growth slows as farmers-per-arable-land rises) — i.e. carrying capacity behaves like a real ceiling, not a soft modifier. **[A]** (Proc. R. Soc. B 2018, "Equilibrium dynamics of European pre-industrial populations")
- **Rainfall context for dry farming:** the Lawrence study region is dry-farming plains operating around a **~450–550 mm** average annual rainfall band (modern ~500 mm). This is a usable biome anchor — rain-fed cereal agrarian settlement in the Near East clusters around the 500 mm isohyet, thinning sharply as you approach the classic ~200 mm dry-farming limit. **[A]** (Lawrence et al. 2016; the 200 mm limit is standard Near Eastern archaeology — general knowledge, verify before citing a specific figure.)

**Design guidance.** Keep the multiplicative engine if it's convenient, but (a) make the biome/NPP term non-linear (diminishing returns at high NPP), (b) let a "technology" step function deliver the ~100× agrarian jump rather than a smooth slider, and (c) remember pathogen load *subtracts* capacity precisely in the wettest, most productive biomes — the reason tropical rainforest supports *low* human density despite high NPP. That interaction is real and is exactly the kind of grounded, non-obvious behavior worth modeling.

---

## 5. Transport cost ratios (high value for the logistics engine)

This is where the framework's `Sea 1 / River-down 2 / River-up 4 / Road 10 / Trail 20 / Mountain 50` meets the literature. Good news first: **your framework values sit *inside* the scholarly uncertainty band** — they are a defensible middle-ground design choice, not a fantasy. The literature does not agree on a single ratio; it spans a wide, actively-contested range, and knowing the *band* is more useful than picking one "true" number.

### 5a. The two scholarly endpoints

**Traditional / "steep" view** — sea transport vastly cheaper than land. Rooted in Diocletian's Price Edict (301 CE) and Cato, via A.H.M. Jones's much-repeated line that it was cheaper to ship grain across the Mediterranean than cart it 75 miles. **[A]** (Jones 1964; Finley 1973; Duncan-Jones 1982). Computed per-kg-of-wheat-per-km from the Edict (via the ORBIS model), the modal costs are:

| Mode | denarii / kg wheat / km | Ratio (sea = 1) |
|---|---|---|
| Sea | 0.00067 | **1** |
| River, downstream | 0.0034 | **~5** |
| River, upstream | 0.0068 | **~10** |
| Land, ox-wagon | 0.035 | **~52** |

Source: Scheidel 2014 (ORBIS), from Duncan-Jones's reading of the Edict. **[A]**

**Revisionist / "flat" view** — a 2024 archaeological reanalysis using the distribution of Late Roman pottery in Britain (not the sparse historical Edict) finds the water advantage was **much smaller**: road only ~3× river and ~4× sea. **[A]** (Wiseman, Ortman & Bulik 2024)

| Mode | Ratio (sea = 1) | 95% CI |
|---|---|---|
| Sea | **1** | — |
| River | **~1.3** | road:river best-fit 1:3 (CI 1:1–5) |
| Road | **~4** | road:sea best-fit 1:4 (CI 1:1–9) |

### 5b. Side-by-side, normalised to sea = 1

| Mode | Framework [D] | Diocletian Edict [A] | Wiseman 2024 [A] |
|---|---|---|---|
| Sea | 1 | 1 | 1 |
| River downstream | 2 | ~5 | ~1.3 |
| River upstream | 4 | ~10 | (not split) |
| Road | 10 | ~52 | ~4 |
| Trail | 20 | — | — |
| Mountain / pass | 50 | — | — |

Reading: your **road = 10** sits between the revisionist ~4 and the traditional ~52 — squarely inside the honest band. Your **river 2/4** brackets the revisionist ~1.3 and undershoots the Edict's 5/10. **Trail = 20** and **mountain = 50** have no direct scholarly anchor (the Edict doesn't price bad-terrain pack transport separately) but are reasonable extrapolations of the wagon-on-good-road → pack-animal-on-bad-ground gradient — tag them **[D]** and move on.

### 5c. The modeling insight most sources miss

Wiseman et al. stress that **transfer/handling points** ("cost points") matter as much as distance: if each transshipment adds ~5% to final cost, a dozen of them compound to **~80% overhead before any distance cost**. **[A]** For Cartalith this validates treating **crossings and mode-changes as cost multipliers in their own right** — a route with many way-transitions should cost more than its length implies, independent of terrain. That's a grounded reason for the stage engine to log crossings as real cost events, not just annotations.

---

## 6. What is invented (design abstractions — legitimate, but not academic)

For completeness, the framework constructs with **no** academic grounding. None of these are *wrong* as game design; they simply must be tagged **[D]** and never presented as sourced:

- `Carrying Capacity = Climate × Water × Soil × Technology × Stability` and all its numeric ranges.
- The entire **Trade Access** point system (Deep-Water Port +100, Mountains −50, etc.).
- **Hub Score = Flow × Security × Surplus × Uniqueness** and the hub-tier population bands.
- The **Security** formula `(State Power + Wealth) / (Population + Threat)` and its threat/bonus point tables.
- All **fantasy modifiers** (teleport circles +200, dragon ports, world-tree gateways). Obviously non-academic — but currently sitting *un-separated* from the "verified" material, which is the real problem.

The security idea does have one real hook: a 2020s strand of Roman-economy scholarship argues the sea-cost advantage shrinks once you price in **convoy protection against piracy** — i.e. security cost legitimately *eats into* transport savings. **[A]** So rather than a standalone security multiplier, security could modify the *transport* cost directly, which is both better-grounded and mechanically cleaner.

---

## 7. Cartalith quick-reference (the usable payload)

Condensed constants to plug into the planner and the forthcoming place/economic-modifier system. Every number carries its tier; treat [D] as tunable defaults and [A]/[B] as calibration targets.

**Site footprint → population**
- `pop ≈ area_ha × 150` (mainstream default) **[A]**
- Dense fortified core: ×200 · dispersed/rural: ×80 **[A]/[B]**

**Regional density caps (persons/km²)** — for populating territory **[B]/[D]**
- Frontier forage `<0.1` · rich forage `0.1–1` · shifting ag `1–5` · dry-farming `5–15` · prosperous core `15–40` · pre-industrial ceiling `~50`

**Settlement size ceilings by political scale (ha)** — real anchors **[A]**
- Pre-urban `<20` · chiefdom/early-urban `40–60` · agrarian plateau `100–130` · territorial kingdom `250–750` · empire/imperial capital `1,000–7,000`

**Transport cost multipliers (sea = 1)** — pick a stance, keep it consistent
- Design middle-ground **[D]**: sea 1 · river-down 2 · river-up 4 · road 10 · trail 20 · mountain 50
- If you want "steep/traditional" **[A]**: sea 1 · river-down 5 · river-up 10 · land 52
- If you want "flat/revisionist" **[A]**: sea 1 · river 1.3 · road 4
- **Transfer overhead:** +~5% per mode-change/transshipment, compounding **[A]**

**Carrying-capacity behavior rules (not scalars)** **[A]**
- Biome/NPP term: non-linear, diminishing returns at high NPP.
- Agrarian tech jump: ~100× over richest foragers (step, not slider).
- Wettest/most-productive biomes: pathogen load *subtracts* capacity (rainforest paradox).
- Rain-fed agrarian settlement clusters around ~500 mm annual rainfall; thins toward the ~200 mm dry-farming limit.

---

## References

Full citations grouped by topic. Peer-reviewed and standard-reference works are the load-bearing set; derived/tertiary items are flagged in-text.

**Uploaded primary source**
- Lawrence, D., Philip, G., Hunt, H., Snape-Kennedy, L., & Wilkinson, T. J. (2016). Long Term Population, City Size and Climate Trends in the Fertile Crescent: A First Approximation. *PLoS ONE*, 11(3): e0152563. doi:10.1371/journal.pone.0152563. *(Open access, CC-BY.)*

**Intra-settlement density / population from area**
- Naroll, R. (1962). Floor Area and Settlement Population. *American Antiquity*, 27(4): 587–589.
- De Roche, C. D. (1983). Population Estimates from Settlement Area and Number of Residences. *Journal of Field Archaeology*, 10(2): 187–192.
- Sumner, W. M. (1989). Population and Settlement Area: An Example from Iran. *American Anthropologist*, 91(3): 631–641.
- Kramer, C. (1982). *Village Ethnoarchaeology: Rural Iran in Archaeological Perspective*. New York: Academic Press.
- Postgate, J. N. (1994). How Many Sumerians per Hectare? Probing the Anatomy of an Early City. *Cambridge Archaeological Journal*, 4(1): 47–65.
- Duffy, P. R. (2015). Site size hierarchy in middle-range societies. *Journal of Anthropological Archaeology*, 37: 85–99. *(Also the source for the "site-size hierarchy ≠ political hierarchy" caveat.)*

**Regional / landscape density & historical demography**
- Binford, L. R. (2001). *Constructing Frames of Reference*. Berkeley: University of California Press.
- Russell, J. C. (1972). Population in Europe. In C. M. Cipolla (ed.), *The Fontana Economic History of Europe, Vol. I: The Middle Ages*, pp. 25–71. Glasgow: Collins/Fontana. *(and* Medieval Regions and their Cities*, Indiana University Press, 1972.)*
- Chapelot, J., & Fossier, R. (1985). *The Village and House in the Middle Ages*. Berkeley: University of California Press. *(source of the 2–5/km² early-medieval figure.)*
- Herlihy, D. (1989). Demography. In *Dictionary of the Middle Ages*, vol. 4.
- Klein Goldewijk, K., et al. (2017). Anthropogenic land use estimates for the Holocene — HYDE 3.2. *Earth System Science Data*, 9: 927–953. *(HYDE 3.3 update, 2023; hosted with processing by Our World in Data.)*
- Stone, L. (2017). Notes on Medieval Population Geography. *(secondary/derived recalculation from Russell-type totals; per-km² breakdowns are derived, marked [B] in-text.)*
- Beloch, K. J. (1886). *Die Bevölkerung der griechisch-römischen Welt*. Leipzig: Duncker & Humblot. *(the foundational Roman-Italy rural density estimate ~15–20/km²; discussed in Scheidel 2001 below.)*
- Frier, B. W. (2000). Demography. In *The Cambridge Ancient History*, vol. XI (2nd ed.), pp. 787–816. Cambridge University Press. *(Latin West ~17/km², Greek East ~24/km².)*
- Scheidel, W. (2001). *Debating Roman Demography*. Leiden: Brill. *(the Beloch/Frier debate; §5 Mediterranean-scrub calibration anchor.)*
- Hin, S. (2013). *The Demography of Roman Italy: Population Dynamics in an Ancient Conquest Society, 201 BCE–14 CE*. Cambridge University Press.

**Carrying capacity & biome**
- Tallavaara, M., et al. (2018). Productivity, biodiversity, and pathogens influence the global hunter-gatherer population density. *PNAS*, 115(6). doi:10.1073/pnas.1715638115.
- Burger, J. R., & Fristoe, T. S. (2018). Hunter-gatherer populations inform modern ecology. *PNAS*, 115(6): 1137–1139. doi:10.1073/pnas.1721726115.
- *Global hunter-gatherer population densities constrained by influence of seasonality on diet composition.* (2021). *Nature Ecology & Evolution*, 5. doi:10.1038/s41559-021-01548-3. *(author list via DOI.)*
- *Equilibrium dynamics of European pre-industrial populations: the evidence of carrying capacity in human agricultural societies.* (2018). *Proceedings of the Royal Society B*, 285(1871): 20172500.

**Settlement hierarchy models**
- Auerbach, F. (1913). Das Gesetz der Bevölkerungskonzentration. *Petermanns Geographische Mitteilungen*, 59: 74–76.
- Zipf, G. K. (1949). *Human Behavior and the Principle of Least Effort*. Cambridge, MA: Addison-Wesley.
- Christaller, W. (1933). *Die zentralen Orte in Süddeutschland*. (Eng.: *Central Places in Southern Germany*, Prentice-Hall, 1966.)
- Lösch, A. (1940). *Die räumliche Ordnung der Wirtschaft*. Jena: Fischer.

**Transport cost**
- Jones, A. H. M. (1964). *The Later Roman Empire, 284–602*. Oxford: Blackwell.
- Finley, M. I. (1973). *The Ancient Economy*. Berkeley: University of California Press.
- Duncan-Jones, R. (1982). *The Economy of the Roman Empire: Quantitative Studies* (2nd ed.). Cambridge: Cambridge University Press.
- Scheidel, W. (2014). The shape of the Roman world: modelling imperial connectivity. *Journal of Roman Archaeology*, 27: 7–32. *(ORBIS model; Scheidel & Meeks, orbis.stanford.edu.)*
- Wiseman, R., Ortman, S. G., & Bulik, O. (2024). The costs of transporting goods by different modes: A case study of pottery movement in late Roman Britain. *Journal of Archaeological Science*, 170: 106059. doi:10.1016/j.jas.2024.106059.

---

*Compiled for the Cartalith worldbuilding toolchain. Provenance tiers are the point: [A]/[B] are calibration targets, [D] are tunable design defaults, and nothing invented is dressed up as sourced.*

---

*Compiled July 2026. This document does not re-derive the academic claims — see the reference doc for those, with full tiering and citations. This document only asks: given those numbers, what does `buildCarryingCapacity`, `buildSettlementSuitability`, `findSettlementSeeds`, and `_civIterativeAutoWorld` actually need to change to, and where does the existing code already get it right?*

Same provenance tags as the reference doc: **[A]** primary/peer-reviewed, **[B]** derived/secondary, **[C]** tertiary, **[D]** design abstraction with no empirical source. Everything new in this file that isn't a direct citation is tagged **[D]**, same as your own document's honesty standard — a biome-density table for agricultural cells doesn't exist in the literature (your reference doc says so explicitly), so the table below is *interpolated against real anchors*, not measured.

---

## 0. Two things worth knowing before you read the rest

**Your existing tier populations are already well-calibrated.** Nobody asked me to check this, but it's the first thing worth reporting: `hamlet=120 · village=400 · town=1500 · city=6000 · capital=15000`, run through the mainstream 150 persons/ha conversion (Section 1 of the reference doc), land almost exactly on the Lawrence et al. era-ceilings from the uploaded paper. See §5.

**`buildCarryingCapacity` currently has no biome term at all.** It's `soil × tempBell × waterMod`. Biome only enters as a gate (`if(biome[i]===0) continue` — ocean only). Temperature and moisture are already *doubled up* through `soil` (which has its own temp/moisture terms) and `water`, so there's a real risk of double-counting if a biome multiplier just re-encodes "how good is this climate" a third time. §2 is designed specifically to avoid that.

---

## 1. Direct reuse: `buildNPP` → hunter-gatherer density floor

You already compute NPP via the Miami model (`buildNPP`, g dry-matter/m²/yr, capped ~3000) for the wildlife layer. Tallavaara et al. 2018 found NPP is the dominant driver of *pre-agricultural* human density; a 2021 follow-up study gives the actual fitted regression, which your reference doc cites but doesn't quote a number for:

> log₁₀(density) = 9.6×10⁻⁴ · NPP − 1.53, NPP in **gC/m²/yr** (MODIS-derived) **[A]** (*Global hunter-gatherer population densities constrained by influence of seasonality on diet composition*, Nat. Ecol. Evol. 2021, cited in your reference doc §6/References)

`buildNPP` outputs grams of **dry matter**, not carbon — the standard biomass→carbon fraction is ≈0.45, so the field needs converting before the regression applies. This one line is the whole "drop-in":

```js
/* Forager (pre-agricultural) density floor from NPP — Zhu et al. 2021, after Tallavaara et al. 2018
   [A] (settlement_density_reference.md §4, §6). Their regression is fit on MODIS NPP in gC/m²/yr;
   buildNPP() returns g DRY MATTER/m²/yr (Miami model), so ×0.45 converts to the carbon basis the
   regression assumes (standard biomass→carbon fraction). Output: persons/km², pre-agricultural floor. */
const FORAGER_NPP_SLOPE = 9.6e-4, FORAGER_NPP_INTERCEPT = -1.53, NPP_DRYMATTER_TO_CARBON = 0.45;
function foragerFloorKm2(nppDryMatter){
  const nppC = nppDryMatter * NPP_DRYMATTER_TO_CARBON;
  return Math.pow(10, FORAGER_NPP_SLOPE * nppC + FORAGER_NPP_INTERCEPT);
}
```

**Calibration check** — this is the load-bearing sanity test, run before trusting the conversion factor:

| `buildNPP` output (g DM/m²/yr) | → gC/m²/yr | `foragerFloorKm2()` | Real anchor (reference doc §2a) |
|---|---|---|---|
| 0 (ice/tundra) | 0 | **0.030/km²** | Binford median 0.044/km²; "empty frontier <0.1" tier — matches |
| 3000 (Miami-model max, richest biome) | 1350 | **0.58/km²** | "richest biomes… up to ~0.5–1/km²" (Tallavaara 2018) — matches closely |

Without the ×0.45 conversion, NPP=3000 gives 22/km² — an order of magnitude too high, and would have silently poisoned any use of this regression. Worth flagging because it's exactly the kind of unit mismatch that doesn't throw an error, just produces a plausible-looking wrong number.

One structural bonus: Tallavaara's own paper found the NPP–density relationship is *piecewise* — positive below a ~1,360 gC/m²/yr breakpoint, flattening/reversing above it as pathogen stress starts dominating (the "rainforest paradox"). Converting Cartalith's NPP ceiling (3000 g DM → 1350 gC) shows it lands essentially *at* that breakpoint — meaning the simple linear regression above is valid across the engine's *entire* NPP range without needing the piecewise correction. That's a free simplification, not a coincidence to worry about — it's a consequence of the Miami model's own dry-matter ceiling roughly matching the real-world NPP range the regression was fit on.

---

## 2. `buildCarryingCapacity` — biome as a residual term, not a restatement

### 2a. Why the multiplier has to be modest

`K = soil × tempBell(T) × waterMod(water)`, and `soil` itself is `tempBell(T) × moisture(rain) × lithWeather × slopeShed × age`. Temperature and moisture are already priced in twice. A biome multiplier that tries to re-encode "how favorable is this climate" (i.e. reproduce the whole regional-density ladder in reference doc §2b as a per-cell factor) would triple-count the same signal and distort the shape of K for no reason.

What a biome term *should* capture is the residual, well-documented effect that soil/temp/water genuinely miss:

- **Pathogen/disease burden** — real, and specifically concentrated in the wettest, most productive biomes (reference doc §4, "the reason tropical rainforest supports low human density despite high NPP"). **[A]**
- **Extreme-climate survival friction** — arid heat stress, evaporation/storage loss, extreme cold — costs that aren't nutritional/agronomic and so aren't in a soil-fertility term. **[D]**, but the direction is well attested.

That's a correction in roughly the 0.55–1.0 range, not a 0.02–1.0 range. The *big* biome effects (a desert cell being ~100× less habitable than a temperate one) are already handled correctly by `tempBell`/`moisture` doing their job — this term isn't supposed to redo that.

```js
/* Biome-residual carrying-capacity multiplier — corrects for disease burden and extreme-climate
   friction NOT already captured by soil/temp/water (see §2a). Deliberately modest (0.55–1.0):
   soil × tempBell × waterMod already do the heavy lifting on "is this climate good for farming".
   Indexed to match BIOME_KEYS order exactly (ice,tundra,boreal,conifer,tempForest,tempRain,grass,
   shrub,desert,savanna,tropDry,tropWet,lake) — access via biomeIdx-1, same idiom as koppenColor(). [D] */
const BIOME_DENSITY_RESIDUAL = [
  /* ice        */ 0.60,   // extreme-cold survival friction
  /* tundra     */ 0.65,
  /* boreal     */ 0.85,
  /* conifer    */ 0.85,
  /* tempForest */ 1.00,   // reference biome — highest forager AND agrarian density in the literature [A]
  /* tempRain   */ 0.90,   // denser vegetation / clearing effort vs. tempForest
  /* grass      */ 0.90,   // mild: rainfall-variability/drought risk not in the mean-rainfall soil term
  /* shrub      */ 0.95,   // Mediterranean dry-farming — well-adapted historically, minimal residual
  /* desert     */ 0.55,   // arid survival friction (heat, evaporation, storage) beyond soil/water
  /* savanna    */ 0.80,   // seasonal drought risk
  /* tropDry    */ 0.75,   // moderate disease/clearing burden
  /* tropWet    */ 0.55,   // the rainforest paradox — real pathogen suppression [A] (Tallavaara 2018)
  /* lake       */ 0.00    // water; excluded upstream anyway
];
function biomeDensityResidual(biomeIdx){
  return biomeIdx ? (BIOME_DENSITY_RESIDUAL[biomeIdx-1] != null ? BIOME_DENSITY_RESIDUAL[biomeIdx-1] : 0.9) : 0;
}

/* buildCarryingCapacity — biome-aware. bK gates the new term: 0 = exactly current behaviour
   (bit-identical), 1 = full residual applied. Everything else byte-identical to v0.62. */
function buildCarryingCapacity(soil, water, biome, temp, fld, W, H, sea, opts){
  const o=opts||{}, n=W*H, out=new Float32Array(n), tOpt=o.tOpt!=null?o.tOpt:18, tVar=o.tVar!=null?o.tVar:800;
  const bK=o.biomeK!=null?o.biomeK:1;                     // NEW — recommend defaulting to 1 (this is a correctness fix, not a stylistic toggle), but 0 preserves exact v0.62 output if you want to stage it separately
  for(let i=0;i<n;i++){
    if(fld[i]<sea) continue;
    if(biome&&biome[i]===0) continue;
    const T=temp[i], tF=Math.exp(-((T-tOpt)*(T-tOpt))/tVar);
    const wMod=0.25+0.75*water[i];
    const bM = biome ? (1-bK + bK*biomeDensityResidual(biome[i])) : 1;   // NEW — lerp between "off" and full residual
    out[i]=Math.max(0,Math.min(1, soil[i]*tF*wMod*bM));
  }
  return out;
}
```

### 2b. The `Wetlands / Marshes` gap

`buildCarryingCapacity`'s `biome` argument is fed `buildBiomeRaster()` — the 13-entry **climate** classification (`BIOME_KEYS`: ice…tropWet,lake). It has no "wetlands" entry. Wetlands/Marshes only exists in the *other*, 15-entry vocabulary (`CART_BIOMES`), assembled later in `buildCartBiome()` from a moisture+flatness override (`M>0.62 && r<0.18 && sn<1.0`) that runs on top of whatever climate biome is underneath. **The two pipelines don't currently talk to each other for this biome.** If you want Wetlands/Marshes to carry its own carrying-capacity story (real anchor: high *local* productivity from fish/rice/waterfowl, offset by historical malaria/flood risk — reference doc's rainforest-paradox logic applies here too, via a different disease vector), that moisture+flatness detector needs to be exposed to (or duplicated into) `currentCarryingCapacity()`'s pipeline, not just the rendering layer. Flagging as an open item rather than fixing it here, since it touches two functions you may want to consolidate differently.

---

## 3. Recovering real persons/km² — a separate function, not a change to K

`K` should stay a pure [0,1] affordance signal — `buildSettlementSuitability`'s weighted sum depends on that. Converting to actual population density is a distinct, *additive* step, calibrated against reference doc §2's real anchors:

```js
/* Real-unit regional population density from the existing affordance fields. Two additive terms:
   - a forager floor (§1) — even bad farmland supports *some* population, pre-agricultural style
   - K scaled by a ceiling that itself depends on water access — because the single best-documented
     way pre-industrial societies broke the ~45-50/km² rain-fed ceiling (reference doc §2b) was
     water-driven intensification: irrigation (Nile ~200+/km² locally [C]), wetland/raised-field
     agriculture (Classic Maya, [A] anchor below), not a generic "better soil".
   RAINFED_CEILING_KM2  = 45   — reference doc §2b, Low Countries c.1500, the real pre-industrial
                                  rain-fed regional ceiling [B]
   INTENSIVE_CEILING_KM2 = 165 — Classic Maya Lowlands, 9.5–16M people / 95,000 km² [A]
     (Estrada-Belli, Canuto, Šprajc & Fernandez-Diaz 2025, lidar-based REGIONAL average, not a local
     pocket — see new reference below). Chosen as the shared ceiling because it's the most rigorous
     large-area figure available; true floodplain-oasis extremes (Nile) may run higher still — bump
     desert's intensifyEligible weight if you want to chase that specifically, rather than raising
     the shared constant. */
const RAINFED_CEILING_KM2 = 45, INTENSIVE_CEILING_KM2 = 165;

/* How much a biome's ceiling can be pushed by water access, i.e. "if this cell had perfect water,
   how transformative would irrigation/wetland farming historically have been here?" High for the
   biomes where that's the ENTIRE reason dense civilization appeared somewhere otherwise marginal
   (desert→Nile, tropical jungle→Maya); low where good rain-fed cells were already near the ceiling
   without needing that story (temperate forest, Mediterranean scrub). [D], same array shape as §2a. */
const BIOME_INTENSIFY_ELIGIBLE = [
  /* ice        */ 0.10, /* tundra */ 0.10, /* boreal */ 0.20, /* conifer */ 0.20,
  /* tempForest */ 0.30, /* tempRain */ 0.30, /* grass  */ 0.50, /* shrub   */ 0.40,
  /* desert     */ 1.00,   // Nile / oasis civilizations
  /* savanna    */ 0.50,
  /* tropDry    */ 0.60,
  /* tropWet    */ 0.90,   // Maya raised-field / terrace agriculture
  /* lake       */ 0.00
];
function biomeIntensifyEligible(biomeIdx){
  return biomeIdx ? (BIOME_INTENSIFY_ELIGIBLE[biomeIdx-1] != null ? BIOME_INTENSIFY_ELIGIBLE[biomeIdx-1] : 0.3) : 0;
}

function estimateRegionalDensityKm2(K, water, biome, npp, fld, W, H, sea){
  const n=W*H, out=new Float32Array(n);
  for(let i=0;i<n;i++){
    if(fld[i]<sea){ out[i]=0; continue; }
    const iw=biome?biomeIntensifyEligible(biome[i]):0.3;
    const ceiling = RAINFED_CEILING_KM2 + (INTENSIVE_CEILING_KM2-RAINFED_CEILING_KM2)*iw*water[i]*water[i];
    out[i] = foragerFloorKm2(npp[i]) + K[i]*ceiling;
  }
  return out;
}
```

**Calibration check** (best-case cells, not regional averages — see the note below the table):

| Cell profile | K | biome | water | → ceiling | → density | Compare to |
|---|---|---|---|---|---|---|
| Prime temperate river valley | 1.0 | tempForest | 1.0 | 81/km² | **~81/km²** | High-medieval France/Italy prosperous core: 15–40/km² *regional*; a single best cell exceeding the regional mean is expected — see below |
| Nile-style desert floodplain | 0.4 | desert | 1.0 | 165/km² | **~66/km²** | Egypt's cultivated Nile land is regionally cited far higher than this in places [C]; treat as a floor, not a ceiling, for that specific case |
| Maya-style tropical floodplain | 0.6 | tropWet | 1.0 | 153/km² | **~92/km²** | Maya Lowlands regional average 100–168/km² [A] — same order of magnitude |

**Peak-cell vs. regional-average, stated explicitly:** a *single best cell* legitimately running above the ~45–50/km² regional ceiling from reference doc §2b is not a contradiction — that ceiling is a *territory-wide average* including all the hills, forest, and marginal land between the good cells. This is the same distinction your own reference doc draws for Egypt ("*local*, not regional average"). If you want a literal regional figure (e.g. for a faction's total population), integrate `estimateRegionalDensityKm2` over the faction's territory rather than reading any single cell.

---

## 4. New reference: the Maya finding actually revises reference doc §2b

Reference doc §2b currently reads: "**Do not push a regional average above ~50/km² for a rain-fed pre-industrial setting.**" That's correct as written — it's scoped to *rain-fed*. But the Maya lidar surveys are a large-area (95,000 km²), peer-reviewed, *regional* figure that lands at 100–168/km², well above that ceiling, precisely because Classic Maya agriculture wasn't simple rain-fed dry farming — it used raised fields, terracing, and managed wetlands. This isn't a contradiction of §2b, it's the missing worked example of exactly the caveat §2b already gestures at ("or (b) already leaning on the transport-and-taxation decoupling…" — add "or water-managed agricultural intensification" as a third case). Worth folding into the reference doc directly if you want one canonical file; noted here so nothing gets lost either way.

**New citations** (add to reference doc's References, "Regional / landscape density" or a new subsection):

- Canuto, M. A., Estrada-Belli, F., Garrison, T. G., et al. (2018). Ancient Lowland Maya Complexity as Revealed by Airborne Laser Scanning of Northern Guatemala. *Science*, 361(6409), eaau0137. doi:10.1126/science.aau0137. *(2,144 km² surveyed, Petén; ~120 persons/km² average, 7–11M people.)* **[A]**
- Estrada-Belli, F., Canuto, M. A., Šprajc, I., & Fernandez-Diaz, J. C. (2025). New regional-scale Classic Maya population density estimates and settlement distribution models through airborne lidar scanning. *(ScienceDirect, in press 2025.)* *(95,000 km², Central Maya Lowlands; revised 9.5–16M people, i.e. ~100–168 persons/km² regional average.)* **[A]**

Also worth noting: Lawrence et al. 2016 (your uploaded PDF) gives one more citable number the reference doc doesn't currently quote — Fig 5D's aggregate settled-area curve peaks at **~1.2–1.3 ha settled per km² surveyed** during the "Territorial Empires" era (Roman–Early Islamic), dropping to ~0.2–0.3 ha/km² in the earliest prehistoric phase. This is the *continuous* version of the discrete "0.2 to 1.3 ha/km²" range reference doc §3a already states — confirmed directly from the paper's own figure, not a secondary summary.

---

## 5. Settlement tiers vs. the Lawrence era-ceilings — validated, with a real gap flagged

Using reference doc §1's mainstream conversion (**150 persons/ha**) on your current hardcoded tier populations:

| Tier | `_civIterativeAutoWorld` pop | → footprint (150/ha) | Lawrence et al. era ceiling | Read |
|---|---|---|---|---|
| hamlet | 120 | 0.8 ha | — | below any named ceiling, correctly so |
| village | 400 | 2.7 ha | — | ″ |
| town | 1,500 | 10 ha | pre-urban, **<20 ha** | comfortably pre-urban |
| city | 6,000 | 40 ha | Late Chalcolithic, **40–60 ha** | lands right at the boundary |
| capital | 15,000 | 100 ha | EBA plateau, **100–130 ha** | lands right at the boundary |

This is a genuinely clean match — nobody hand-tuned these five numbers against this specific paper, and they land on three consecutive real archaeological thresholds almost exactly. The honest reading: **the current tier system caps out at Early Bronze Age urbanism scale.** If you want Cartalith to represent genuinely imperial capitals — Nineveh (750 ha), Baghdad/Samarra (4,500–7,000 ha, ≥280,000 people, reference doc §3a — a further 5–70× beyond your current `capital` ceiling — there isn't a tier for that. Current network-centrality population scaling (`normB` in `_civIterativeAutoWorld`) tops out around a 2.5× multiplier on the base 15,000, i.e. ~37,500 max — still an order of magnitude short of Baghdad. Two options, not prescribing either:

- **Extend the multiplier range** for very high betweenness centrality (a `Math.pow` or wider linear range on the existing `1+normB*1.5` term) so a single dominant hub can spike past the current ceiling; keeps five tiers.
- **Add a `metropolis`/`imperial seat` tier** above capital, populated only when network centrality *and* faction territory size both clear a high bar — closer to how the real ceiling-breaking actually happened (Lawrence et al.'s whole thesis: post-2000 BC growth is driven by administrative/taxation capacity, not local land, which is exactly what betweenness centrality + territory size already proxy for in your model).

---

## 6. Settlement spacing — the real anchor vs. what the code currently does

**Real anchor.** Site catchment analysis (Vita-Finzi & Higgs 1970) gives ~5 km (a 1-hour walk) as the standard exploitation-territory radius for agricultural settlements **[A]**. Villages historically space at roughly double that so catchments don't overlap — a **~10 km** center-to-center floor for the finest settled tier. Central-place market-town spacing (a different tier, driven by trade "range" rather than field-walking distance) runs considerably wider — commonly cited in the 20–30 km band for a practicable day-trip market catchment **[B]/[D]**, no single authoritative number.

**What Cartalith actually does today.** `_civIterativeAutoWorld`'s default suppression radius is `Math.max(6, GW/22)`; at the engine's own defaults (`GW=2048`, `mapWidthKm=800` → `cellKm≈0.39`) that's ≈93 cells ≈ **36 km**. `findSettlementSeeds`'s own internal default (`W/20`) is ≈41 km. Both are already tuned to roughly *market-town* spacing, not *village* spacing — there is currently no finer-grained layer placing anything at the true ~10 km village floor. That's not a bug; it may be exactly the right choice for a map-scale worldbuilding tool where showing every historical hamlet would be visual noise. But it's worth being a deliberate choice rather than an implicit one, since the gap between "what's coded" and "what the literature says a village grid looks like" is roughly 3–4×.

```js
/* Site-catchment-anchored settlement spacing, expressed in real km so it survives changes to
   map width / working resolution — mirrors the cellKm idiom already used throughout this file. */
const VILLAGE_SPACING_KM = 10;    // [A] Vita-Finzi & Higgs 1970 — floor for the finest settled tier
const MARKET_TOWN_SPACING_KM = 25; // [B]/[D] — central-place "range", no single authoritative figure

function suppressionRadiusCells(spacingKm, GW, mapWidthKm){
  const cellKm = mapWidthKm / GW;
  return Math.max(4, Math.round(spacingKm / cellKm));
}
```

If you do want a true village-density layer, the settlement *count* this implies is worth checking before committing — at 10 km spacing an 800 km-wide map has room for on the order of `(800/10)² × 0.6 ≈ 3,800` settlements (0.6 ≈ packing efficiency for a suppression-radius scatter, not a hex grid), which is very likely too many named, individually-editable places for the current per-settlement UI (settlement list, editors, trait pickers) to stay usable. A blended approach — keep the current ~30–40 km pass for named/major settlements, optionally add an unlabeled density-only "rural population" raster from §3 for regional population totals without individually placing every hamlet — would get the real-world density right for population/economy math without a 3,800-pin map.

---

## 7. Rank-size, briefly — for the auto-populate count fields

Not a new formula, just a usable default if you ever want the five count fields (`civNCap/civNCity/civNTown/civNVil/civNHam`) to have a sensible *relationship* to each other instead of being independently typed in. Two well-established, real models, both already namechecked in reference doc §3c:

- **Rank-size rule** (Auerbach 1913; Zipf 1949) **[A]**: the *n*th-largest settlement's population ≈ largest/*n*. Applied to your tiers, if `capital=15000` that predicts city≈7500, town≈5000, village≈3750… — this is a **population** ratio, not a **count** ratio, and it doesn't match your tier system's five discrete bands well (the real rule describes a smooth curve across many settlements of gradually shrinking size, not five fixed classes). Better used as a sanity check on the auto-assigned *populations* within one tier (are they all clustered at the tier base, or spread out realistically?) than as a count generator.
- **Central place k=3 hierarchy** (Christaller 1933) **[A]**: each order of center serves roughly 3× the market area of the next order down, which loosely translates to "roughly 3× as many settlements at each tier down." A `hamlet : village : town : city : capital` count ratio near `81 : 27 : 9 : 3 : 1` (powers of 3) is the textbook shape — offered as a *default* for the blank/auto count fields, not a hard rule; Christaller's own model assumed a featureless plain, and terrain/coastline/rivers will and should break it, which is exactly what your suitability-driven placement already does better than a fixed ratio could.

---

## 8. Summary table — everything in one place

| Constant / function | Value | Tier | Used by |
|---|---|---|---|
| `FORAGER_NPP_SLOPE`, `_INTERCEPT` | 9.6e-4, −1.53 | [A] | `foragerFloorKm2()` |
| `NPP_DRYMATTER_TO_CARBON` | 0.45 | [D]-conversion of an [A] figure | `foragerFloorKm2()` |
| `BIOME_DENSITY_RESIDUAL` | 0.55–1.00 per biome | [D], anchored to [A] pathogen finding | `buildCarryingCapacity` patch |
| `RAINFED_CEILING_KM2` | 45 | [B] | `estimateRegionalDensityKm2()` |
| `INTENSIVE_CEILING_KM2` | 165 | [A] (Maya, §4) | `estimateRegionalDensityKm2()` |
| `BIOME_INTENSIFY_ELIGIBLE` | 0.10–1.00 per biome | [D] | `estimateRegionalDensityKm2()` |
| `VILLAGE_SPACING_KM` | 10 | [A] | proposed `suppressionRadiusCells()` |
| `MARKET_TOWN_SPACING_KM` | 25 | [B]/[D] | proposed `suppressionRadiusCells()` |
| Tier populations 120/400/1500/6000/15000 | — | validated against [A] era-ceilings | `_civIterativeAutoWorld` (unchanged — already good) |

---

## 9. Open questions for you, not answered here

*Resolution status (2026-07-10): all five have now been actioned in the Gen1 tool — 1 → v0.69, 4 → v0.75,
3 → v0.76, 2 → v0.77, and 5 is resolved below with a sourced calibration. Each carrying-capacity change is
opt-in (rides the "Biome carrying-capacity" toggle) so default output stays bit-identical.*

1. **`buildCarryingCapacity`'s `bK` default** — *resolved (v0.69): defaults to 0 (bit-identical), one-click opt-in.* 0 (bit-identical, matches this codebase's pervasive convention for new terms) or 1 (this reads more like a correctness fix than a style choice, so defaulting on is defensible too). Your call given how the rest of the affordance-field code stages new terms.
2. **Wetlands/Marshes carrying-capacity** (§2b) — *resolved (v0.77): `buildWetlandMask()` exposes the exact `buildCartBiome` moisture+flatness detector to the K pipeline; wetland residual 0.70 + intensify 0.95.* needs the `buildCartBiome` moisture+flatness detector exposed to the carrying-capacity pipeline, or accept that Wetlands only affects rendering/labeling for now.
3. **Settlement spacing** (§6) — *resolved (v0.76): opt-in "Dense village grid" mode via `suppressionRadiusCells(VILLAGE_SPACING_KM)`, capped at 200 pins; plus a density-only regional-population estimate for totals without a pin per hamlet.* sparse "notable places only" (current default, probably fine for a map-scale tool) vs. genuine village density (needs a ~3–4× denser default and almost certainly a different UI for it, since 3,800 individually-editable settlements isn't a UI you'd want).
4. **Metropolis tier** (§5) — *resolved (v0.75): opt-in imperial-seat tier gated on betweenness × polity size, rare (≤1/faction).* worth it for imperial-scale worlds, or is Early-Bronze-Age-scale urbanism (the current honest ceiling) the intended register for Cartalith worlds?
5. **Mediterranean Scrub calibration** — *resolved (2026-07-10, follow-up search): the Roman-Italy anchor **confirms** `shrub`'s existing residual (0.95) and moderate intensify eligibility (0.40); no change needed.* Beloch put rural Italy at ~15–20/km²; Frier's reconstruction gives the Latin West ~17/km² (Greek East ~24/km²), with Italy/Sicily above the western average — i.e. **classical Mediterranean dry-farming country sat among the densest pre-industrial rain-fed regions, in the "prosperous agrarian core (15–40/km²)" tier of §2b.** That warrants a **high** residual (well-adapted, minimal disease/climate friction — a hair below 1.0 only for real Mediterranean summer-drought risk the mean-rainfall soil term underweights) → **0.95 stands, now sourced rather than assumed.** And because that density was rain-fed (cereals/olives/vines), *not* irrigation-driven like Egypt/Maya, its **intensify eligibility stays moderate (0.40)** — Mediterranean scrub was already near its rain-fed ceiling without a water-management story. **[B]** (Beloch 1886 via Scheidel 2001; Frier 2000; Hin 2013).

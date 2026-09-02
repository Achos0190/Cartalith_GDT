# Cartalith Landmark Generation

**Owner-supplied research, imported verbatim 2026-08-30.** Nothing below this
banner is this port's own writing — it is the owner's specification as
delivered, kept unedited the same way `TERRAIN_ARCHITECTURE_RESEARCH.md`,
`HETEROGENEOUS_COMPUTE_RESEARCH.md` and `TERRAIN_APPEARANCE_RESEARCH.md` are.
The port's own assessment of what is buildable, what already exists in the
engine, and in what order, belongs in `LANDMARK_GENERATION_SCOPE.md`, not here.

---

A Geospatial and Procedural Framework for Realistic Landmark Placement in Procedurally Generated Worlds

Technical Research Specification
Application: Cartalith Worldbuilding / Geographic Simulation System
Domain: Procedural Geography, GIS, Landscape Analysis, Hydrology, Settlement Geography, Spatial Modelling
Status: Proposed implementation framework
Date: 30 August 2026

---

## Abstract

This document proposes a mathematically grounded framework for generating and positioning landmarks within Cartalith-generated worlds.

The central premise is that landmarks should not be distributed randomly across a generated map. A convincing landmark is normally the consequence of an interaction between terrain, geology, hydrology, ecology, human movement, resources, settlement, political organization, and historical processes.

Cartalith should therefore treat landmark generation as a spatial suitability and causal inference problem rather than an object-placement problem.

The proposed system combines established geographic techniques—including digital elevation model analysis, topographic position indices, terrain curvature, hydrological flow accumulation, viewshed analysis, least-cost path analysis, spatial interaction, spatial autocorrelation, and Poisson-disc sampling—with a Cartalith-specific weighted suitability framework.

The resulting system should be capable of generating both natural landmarks and anthropogenic landmarks, while preserving causal relationships between them.

For example:

«tectonic uplift → steep gradient → waterfall → culturally significant site → shrine → pilgrimage route → settlement»

or:

«geological formation → mineral deposit → extraction → settlement → road → fortification → political importance → historical monument»

The goal is not merely to make landmarks statistically plausible. The goal is to make them geographically explainable.

---

## 1. Research Objective

The objective is to develop an algorithm capable of answering:

«Why does this landmark exist here?»

rather than simply:

«Where should a landmark be placed?»

A landmark generator should therefore use existing Cartalith world-state data as its principal input.

The system should not independently generate a generic collection of castles, ruins, waterfalls, temples and monuments.

Instead, it should derive candidate landmarks from the existing physical and socioeconomic structure of the world.

---

## 2. Foundational Principle

A procedurally generated landmark should satisfy three conditions:

### 2.1 Physical plausibility

The terrain and environmental conditions must permit the landmark.

A waterfall requires:

- flowing water;
- sufficient hydraulic gradient;
- an appropriate channel geometry;
- terrain capable of producing a vertical or near-vertical drop.

A mountain pass requires:

- surrounding high terrain;
- a relatively low saddle;
- connectivity between otherwise separated drainage or settlement regions.

A settlement requires:

- viable resources;
- access to water;
- usable terrain;
- some transportation or strategic advantage.

---

### 2.2 Spatial significance

The feature should be sufficiently unusual or consequential relative to its surroundings.

A hill is not automatically a landmark.

A hill becomes a strong landmark candidate when it has some combination of:

- high local relief;
- high visibility;
- unusual geometry;
- strategic position;
- cultural association;
- route significance.

---

### 2.3 Causal significance

Where possible, the landmark should emerge from processes already simulated by Cartalith.

For example:

```
Tectonic uplift
      ↓
High relief
      ↓
Steep river gradient
      ↓
Waterfall
      ↓
Visible natural feature
      ↓
Cultural significance
      ↓
Shrine / monument
```

This produces internally coherent worlds rather than collections of unrelated procedural objects.

---

## 3. Relevant Established Geographic Methods

Several existing GIS and geographic-analysis techniques are directly applicable.

### 3.1 Digital Elevation Model Analysis

Cartalith's heightfield can be treated as a Digital Elevation Model (DEM).

From the elevation field, the engine can derive:

- slope;
- aspect;
- curvature;
- local relief;
- topographic position;
- ruggedness;
- drainage;
- prominence;
- visibility.

These derived fields should become reusable analytical layers rather than being calculated independently by each landmark generator.

---

## 4. Topographic Position Index

One useful method is the Topographic Position Index (TPI).

A simplified formulation is:

TPI(x) = z(x) − (1/N) Σ(i=1..N) z(x_i)

where:

- z(x) is the elevation at the target cell;
- x_i are cells within a specified neighbourhood;
- N is the number of neighbouring cells.

Positive values indicate terrain above its local surroundings.

Negative values indicate terrain below its surroundings.

This allows Cartalith to distinguish between:

High elevation

and:

High elevation + locally dominant terrain

The second is substantially more useful for landmark generation.

TPI can be calculated at multiple spatial scales.

For example:

TPI_local

might use a 500 m neighbourhood while:

TPI_regional

might use a 20 km neighbourhood.

This allows the engine to distinguish:

- local hilltops;
- mountain ridges;
- regional massifs;
- isolated peaks.

---

## 5. Terrain Curvature

Terrain curvature can identify abrupt morphological features.

Relevant forms include:

- profile curvature;
- plan curvature;
- mean curvature;
- Gaussian curvature.

Curvature can contribute to candidate detection for:

- cliffs;
- ridges;
- gullies;
- escarpments;
- bowls;
- natural amphitheatres;
- unusual rock formations.

The exact numerical implementation should depend on the DEM resolution and smoothing strategy.

Cartalith should avoid interpreting individual high-frequency DEM noise as geological structure.

Therefore curvature should normally be evaluated at multiple scales.

---

## 6. Hydrological Landmark Generation

Hydrology provides one of the strongest opportunities for causally generated landmarks.

Cartalith already possesses or intends to possess:

- flow direction;
- flow accumulation;
- river networks;
- drainage basins;
- lakes;
- coastlines;
- terrain gradients.

These should directly feed landmark generation.

---

### 6.1 Flow Accumulation

The D8 algorithm is a classical raster drainage method in which each cell drains toward its steepest downslope neighbour.

The method is associated with O'Callaghan and Mark (1984).

Flow accumulation can then estimate the contributing area upstream of each cell.

A conceptual formulation is:

A(x) = 1 + Σ(i ∈ U(x)) A(i)

where U(x) represents cells contributing flow to x.

Flow accumulation is useful for determining:

- river magnitude;
- confluences;
- drainage hierarchy;
- floodplain significance;
- possible settlement locations;
- waterfall candidates.

The D8 approach is well established, although more sophisticated methods such as D∞ or multiple-flow-direction algorithms may be preferable where flow divergence matters.

---

## 7. Waterfall Detection

A waterfall candidate can be identified where several conditions coincide.

For example:

S_waterfall = w_g·G + w_q·Q + w_c·C + w_r·R

where:

- G = local elevation gradient;
- Q = flow magnitude;
- C = channel confinement;
- R = geological resistance or lithological contrast.

The equation above is not an established scientific waterfall formula. It is a proposed Cartalith suitability model using established environmental variables.

A candidate should then pass explicit constraints:

```
river = true
AND
gradient > threshold
AND
vertical drop > threshold
AND
flow accumulation > minimum
```

This is preferable to merely placing waterfalls on steep terrain.

---

## 8. Mountain Pass Detection

Mountain passes can be derived from terrain topology.

Potential candidates include low-elevation saddle points located between significant areas of elevated terrain.

A pass score could incorporate:

S_pass = w1·P + w2·C + w3·R + w4·A

where:

- P = pass/saddle morphology;
- C = connectivity between regions;
- R = route accessibility;
- A = accumulated transportation demand.

The last term is particularly important.

A physically plausible pass is not necessarily an important landmark.

A pass connecting two economically important regions is much more likely to become:

- a major route;
- a settlement;
- a fortified position;
- a cultural boundary;
- a named geographic feature.

---

## 9. Visibility and Viewshed Analysis

Viewshed analysis is an established GIS technique for calculating which portions of a terrain surface are visible from an observer position.

It operates on elevation data and line-of-sight calculations.

Viewshed analysis is widely used in GIS and landscape analysis.

For Cartalith, visibility can become a major determinant of landmark significance.

Define:

V(x) = Σ(i=1..n) w_i · visibility(i, x)

where observer points i may include:

- settlements;
- roads;
- major river crossings;
- passes;
- ports;
- political centres.

A landmark with high V(x) is visible from many important locations.

This is useful for generating:

- castles;
- watchtowers;
- monuments;
- sacred mountains;
- giant trees;
- distinctive peaks;
- signal towers.

Visibility should not be treated as merely binary.

A more sophisticated implementation can incorporate:

- distance;
- angular size;
- elevation difference;
- atmospheric visibility;
- observer importance.

Research into landscape visibility increasingly treats visibility as a richer spatial phenomenon than a simple visible/not-visible classification.

---

## 10. Route Visibility

A particularly useful extension is to calculate visibility from transport networks rather than arbitrary points.

For every important road, trail, river route or sea route:

```
sample observation points
        ↓
calculate viewshed
        ↓
accumulate visibility
        ↓
produce route-visible surface
```

This produces:

V_route(x)

A location visible from a major transportation corridor receives greater potential significance.

This provides a mathematical basis for generating features such as:

- roadside monuments;
- hilltop fortifications;
- visible temples;
- boundary markers;
- famous rock formations;
- pilgrimage landmarks.

Route-based visibility analysis is already an established GIS research area.

---

## 11. Settlement Geography

Anthropogenic landmarks should not be generated independently of settlements.

Research into archaeological settlement modelling commonly considers combinations of:

- water;
- resources;
- terrain;
- transportation;
- existing settlements;
- social and political factors.

Recent work explicitly frames settlement systems as spatial adaptive systems in which environmental and social interactions jointly influence settlement patterns.

Studies of historical landscapes similarly combine palaeogeography, rivers, transport networks, military sites and previous settlements when modelling settlement location.

Therefore Cartalith should expose settlement data to landmark generation.

---

## 12. Accessibility and Least-Cost Movement

Distance alone is insufficient.

A location 10 km away across a mountain range is not equivalent to a location 10 km away along a valley.

Cartalith should therefore use its existing route-cost system.

For two locations A and B:

C(A,B) = min over π:A→B of Σ(e ∈ π) cost(e)

where:

- π is a possible route;
- e is an edge;
- cost(e) incorporates terrain and transportation conditions.

This allows landmark significance to depend on actual accessibility.

A bridge located at a major river bottleneck can therefore score highly because it reduces travel cost between otherwise separated regions.

---

## 13. Spatial Interaction

Landmark importance should also depend on surrounding activity.

For example, define an interaction score:

I(x) = Σ_i P_i / d_c(x,i)^β

where:

- P_i = importance/population/economic weight of settlement i;
- d_c = least-cost distance;
- β = distance-decay exponent.

This is conceptually related to gravity-style spatial interaction models.

The result identifies locations that sit within the effective sphere of influence of major population centres.

A bridge, pass, fortress or market site located at such a point can therefore receive greater importance.

---

## 14. Resource-Driven Landmarks

Resources should also generate landmarks.

Examples:

```
Ore deposit
    ↓
Mine
    ↓
Mining settlement
    ↓
Road
    ↓
Fortification
```

or:

```
Salt deposit
    ↓
Extraction site
    ↓
Trade route
    ↓
Caravan station
    ↓
Market town
```

The probability of anthropogenic landmark formation should therefore depend on:

P(L | R, S, C, T)

where:

- R = resource value;
- S = settlement structure;
- C = connectivity;
- T = transportation demand.

This should remain stochastic rather than deterministic.

A valuable resource should increase the probability of exploitation, not guarantee it.

---

## 15. Spatial Autocorrelation and Landmark Clustering

Landmarks should not be spatially independent.

Geographic phenomena exhibit spatial relationships; Tobler's First Law is commonly summarized as the principle that nearby things tend to be more related than distant things. Spatial dependence and autocorrelation are therefore fundamental concepts in geographic analysis.

Cartalith should use this principle carefully.

For example:

```
Major city
   ↓
higher probability
   ├── roads
   ├── bridges
   ├── monuments
   ├── fortifications
   ├── religious structures
   └── markets
```

But this should not mean unlimited clustering.

A major landmark should also exert a competition/exclusion radius.

---

## 16. Poisson-Disc Sampling

Poisson-disc sampling provides an established method for generating spatially separated points.

Bridson's 2007 algorithm provides an efficient method for generating Poisson-disc samples and is suitable as a basis for spatial exclusion during procedural generation.

The basic constraint is:

d(p_i, p_j) > r

where r is the minimum separation distance.

Cartalith should use a variable radius:

r = f(class, importance, terrain, region)

For example:

```
Minor landmark      → small exclusion radius
Regional landmark   → medium exclusion radius
Major landmark      → large exclusion radius
World landmark      → very large exclusion radius
```

This prevents procedural landmark saturation.

---

## 17. Candidate Suitability Framework

Cartalith can combine the above variables into a normalized suitability model.

For candidate location x:

S_L(x) = Σ(k=1..n) w_k · F_k(x)

where each F_k(x) is normalized to:

0 ≤ F_k(x) ≤ 1

Possible features include:

```
Fterrain
Fslope
Ftpi
Fcurvature
Fhydrology
Fflow
Fgeology
Fresource
Fvisibility
Froute
Faccessibility
Fsettlement
Fpolitical
Fecology
Fhistorical
```

The weights are determined by landmark class.

---

## 18. Example: Castle

A castle suitability model might be:

S_castle = 0.20·F_visibility + 0.20·F_strategic + 0.15·F_route + 0.15·F_settlement + 0.10·F_slope + 0.10·F_water + 0.10·F_political

These values are initial engineering weights, not empirical scientific constants.

The system should eventually expose them to calibration and experimentation.

A strong candidate might therefore be:

```
high local relief
+
excellent viewshed
+
road bottleneck
+
river crossing
+
near major settlement
+
defensible slope
```

rather than simply:

```
random hill
```

---

## 19. Example: Sacred Mountain

A sacred mountain could instead emphasize:

S_sacred = 0.25·F_prominence + 0.20·F_visibility + 0.15·F_isolation + 0.15·F_ecological + 0.10·F_water + 0.15·F_cultural

The cultural component can depend on an existing civilization's characteristics.

The important point is that the same physical mountain can receive different cultural interpretations.

One civilization may consider it sacred.

Another may regard it as strategically important.

Another may simply avoid it.

---

## 20. Example: Ruin

A ruin should generally be generated from historical state rather than appearing as an isolated decorative object.

Potential chain:

Settlement → Expansion → Conflict/decline → Abandonment → Ruination

The resulting ruin inherits the original site's:

- roads;
- terrain;
- water access;
- economic rationale;
- defensive rationale;
- cultural identity.

This creates substantially stronger historical continuity.

---

## 21. Landmark Generation Pipeline

The proposed Cartalith pipeline is:

```
WORLD STATE
    │
    ├── Heightfield
    ├── Geology
    ├── Hydrology
    ├── Climate
    ├── Ecology
    ├── Resources
    ├── Settlements
    ├── Roads
    ├── Political regions
    └── Historical state
            │
            ▼
DERIVED ANALYTICAL FIELDS
            │
            ├── Slope
            ├── Curvature
            ├── TPI
            ├── Relief
            ├── Flow accumulation
            ├── Drainage hierarchy
            ├── Accessibility
            ├── Route centrality
            ├── Viewshed
            └── Resource influence
            │
            ▼
LANDMARK CANDIDATE GENERATION
            │
            ▼
LANDMARK-SPECIFIC CONSTRAINTS
            │
            ▼
SUITABILITY SCORING
            │
            ▼
SPATIAL COMPETITION / POISSON FILTER
            │
            ▼
CAUSAL VALIDATION
            │
            ▼
HISTORICAL / CULTURAL ASSOCIATION
            │
            ▼
LANDMARK OBJECT
```

---

## 22. Landmark Object Model

A landmark should contain more than coordinates.

Recommended structure:

```
Landmark
{
    id
    type
    subtype

    position
    elevation

    physical_basis
    geological_basis
    hydrological_basis
    ecological_basis

    visibility_score
    accessibility_score
    route_significance
    resource_significance

    settlement_associations
    political_associations
    cultural_associations

    importance
    rarity
    age
    historical_state

    parent_feature
    causal_chain

    generated_seed
}
```

The "causal_chain" is particularly important.

Example:

```
{
    cause:
        "tectonic uplift",

    consequence:
        "high relief",

    hydrological_effect:
        "high river gradient",

    landmark:
        "waterfall",

    cultural_effect:
        "sacred site"
}
```

This gives Cartalith the ability to explain generated geography.

---

## 23. Hierarchical Landmark Classes

Landmarks should be hierarchical.

**Continental**

Extremely rare:

- highest mountains;
- enormous lakes;
- exceptional deserts;
- major geological formations.

**Regional**

Moderately rare:

- major waterfalls;
- mountain passes;
- large caves;
- major forests;
- important ruins;
- large fortifications.

**Local**

Common:

- springs;
- minor waterfalls;
- isolated rocks;
- small ruins;
- unusual trees;
- shrines.

**Cultural**

Dependent on civilization:

- temples;
- tombs;
- monuments;
- battlefields;
- pilgrimage sites;
- royal roads;
- border markers.

This hierarchy should determine both generation frequency and map visibility.

---

## 24. Importance Should Be Emergent

Cartalith should avoid assigning importance solely through a random rarity variable.

Instead:

Importance = f(physical uniqueness, visibility, accessibility, population, economic value, political significance, historical age)

A feature becomes important because something makes it important.

For example:

```
ordinary hill
        ↓
road passes over it
        ↓
excellent visibility
        ↓
fort built
        ↓
regional conflict
        ↓
battle occurs
        ↓
site becomes historically important
```

The final landmark is therefore the product of world history.

---

## 25. Temporal Simulation

Landmarks should be capable of changing state.

Recommended state transitions:

```
Natural feature
      ↓
Discovered
      ↓
Named
      ↓
Utilized
      ↓
Developed
      ↓
Monumentalized
      ↓
Abandoned
      ↓
Ruined
      ↓
Rediscovered
```

Not every landmark needs to follow this entire chain.

The point is to allow Cartalith's historical simulation to modify landmark significance.

---

## 26. Cultural Interpretation

A physical landmark should be separated from its cultural interpretation.

For example:

```
PHYSICAL FEATURE
    Mountain
       │
       ├── Civilization A → Sacred Mountain
       ├── Civilization B → Border Marker
       └── Civilization C → Strategic Fortress Site
```

This avoids hardcoding cultural meaning into geography.

The physical world remains consistent while interpretation varies between cultures.

---

## 27. Generation Should Be Deterministic

Landmark generation should obey Cartalith's deterministic seed contract.

Given:

```
world_seed
+
terrain_state
+
hydrology_state
+
civilization_state
+
landmark_parameters
```

the generator should produce the same landmark distribution.

A useful conceptual seed is:

seed_L = Hash(worldSeed, featureID, landmarkClass, generationVersion)

This allows landmarks to remain reproducible while permitting versioned generator changes.

---

## 28. Multi-Scale Analysis

A major implementation requirement is scale.

A geological feature may be meaningful at 100 km while a shrine may be meaningful at 500 m.

Therefore Cartalith should not use a single analysis radius.

Instead:

```
LOCAL
10 m – 1 km

REGIONAL
1 – 50 km

MACRO
50 – 500+ km
```

The exact scales should depend on world resolution.

TPI, curvature, relief, visibility and accessibility should therefore be available at multiple scales.

---

## 29. Proposed Landmark Types

The initial implementation should prioritize features that can be derived reliably from existing world-state data.

**Physical**

```
Peak
Ridge
Saddle
Cliff
Gorge
Cave
Waterfall
Spring
Lake
Delta
River Confluence
Volcanic Feature
Rock Formation
Glacial Feature
Ancient Forest
```

**Transportation**

```
Mountain Pass
River Crossing
Ford
Bridge Site
Road Junction
Caravan Station
Portage
Harbour
```

**Economic**

```
Mine
Quarry
Salt Works
Resource Extraction Site
Market Site
Trade Depot
```

**Military**

```
Fort
Watchtower
Fortified Pass
Fortified Crossing
Battlefield
Border Marker
```

**Religious / Cultural**

```
Shrine
Temple
Sacred Grove
Sacred Mountain
Pilgrimage Site
Tomb
Monument
Ceremonial Site
```

**Historical**

```
Ruin
Abandoned Settlement
Ancient Road
Battlefield
Destroyed Fortress
Historic Crossing
```

---

## 30. Recommended Algorithm

The practical implementation should follow this sequence:

**Step 1 — Generate analytical terrain fields**

Calculate: slope, aspect, curvature, TPI, local relief, regional relief, ruggedness

**Step 2 — Generate hydrological fields**

Calculate: flow direction, flow accumulation, river hierarchy, basins, lake hierarchy, confluences, gradient

**Step 3 — Generate accessibility fields**

Calculate: least-cost distance, road centrality, river accessibility, pass connectivity, settlement accessibility

**Step 4 — Generate visibility fields**

Calculate viewsheds from: major settlements, roads, passes, political centres, important routes

**Step 5 — Generate candidate locations**

Use deterministic candidate sampling.

**Step 6 — Evaluate landmark-specific constraints**

Reject candidates that violate physical requirements.

**Step 7 — Calculate suitability**

Normalize relevant variables and calculate S_L(x) = Σ w_i·F_i(x)

**Step 8 — Apply spatial competition**

Use minimum-distance constraints / Poisson-disc sampling.

**Step 9 — Apply world-state interactions**

Modify candidate probability according to: settlements, resources, roads, politics, culture, history

**Step 10 — Assign causal history**

Generate the reason the feature became significant.

**Step 11 — Assign importance**

Calculate a final importance score.

**Step 12 — Validate**

Reject landmarks that fail causal or geographic consistency tests.

---

## 31. Important Distinction: Scientific Model vs Procedural Model

Cartalith should explicitly distinguish between three categories.

**Category A — Established geographic computation**

Examples: slope, aspect, curvature, TPI, flow accumulation, viewshed, least-cost path, spatial autocorrelation, Poisson-disc sampling.

These have established mathematical or computational foundations.

**Category B — Empirically inspired modelling**

Examples: settlement suitability, resource accessibility, route importance, cultural-site probability.

These can be grounded in archaeological and geographic literature but require calibration for the specific fictional world.

**Category C — Cartalith synthesis**

Examples: S_castle, S_sacred, Importance = f(visibility, accessibility, political influence, …)

These are engineering models created for Cartalith, not established scientific laws.

This distinction should remain explicit in both documentation and source code.

---

## 32. Research Basis

The framework draws on several established fields.

**GIS and terrain analysis** — Digital elevation models provide the foundation for terrain-derived variables and spatial analysis.

**Hydrology** — Flow-direction and contributing-area algorithms provide methods for deriving drainage structure from terrain. D8 is a classical example described in the hydrological literature.

**Landscape visibility** — Viewshed analysis provides established methods for calculating terrain visibility, with increasingly sophisticated research addressing route-based visibility and richer visual-space analysis.

**Spatial sampling** — Poisson-disc sampling provides a mathematically useful method for enforcing minimum spatial separation between generated features. Bridson's 2007 algorithm is an important reference.

**Human geography and archaeology** — Settlement-location research demonstrates the usefulness of combining terrain, hydrology, transport, resources, existing settlements and social factors rather than treating settlement as a purely physical-terrain phenomenon.

**Spatial statistics** — Spatial autocorrelation and spatial dependence provide a theoretical basis for modelling relationships between geographically proximate features.

---

## 33. Primary References

*Bibliography revised by the owner on the day of import. One entry from the
first draft — "Zhu, A. X. et al. / related spatial-analysis literature" — has
been **removed**, in the owner's own words: "That was insufficiently specified
and shouldn't have been presented as a proper bibliography entry. I would
remove it from the research document rather than pretend it is a precise
citation." The six that remain are the ones the owner names as strongest for
the actual implementation.*

### Core references

1. **O'Callaghan, J. F., & Mark, D. M. (1984).** *The Extraction of Drainage
   Networks from Digital Elevation Data.* Computer Vision, Graphics, and Image
   Processing, 28(3), 323–344. DOI:
   [10.1016/S0734-189X(84)80011-0](https://www.sciencedirect.com/science/article/pii/S0734189X84800110)
   · [DBLP record](https://dblp.org/rec/journals/cvgip/OCallaghanM84)

2. **Bridson, R. (2007).** *Fast Poisson Disk Sampling in Arbitrary
   Dimensions.* ACM SIGGRAPH 2007 Sketches. DOI:
   [10.1145/1278780.1278807](https://doi.org/10.1145/1278780.1278807) ·
   [author's PDF](https://www.cs.ubc.ca/~rbridson/docs/bridson-siggraph07-poissondisk.pdf)

3. **Miller, H. J. (2004).** *Tobler's First Law and Spatial Analysis.* Annals
   of the Association of American Geographers, 94(2), 284–289. DOI:
   [10.1111/j.1467-8306.2004.09402005.x](https://onlinelibrary.wiley.com/doi/full/10.1111/j.1467-8306.2004.09402005.x)

   Listed here and again under *Spatial geography* below because it carries two
   distinct loads: the general principle, and specifically the theoretical
   foundation for treating geographically proximate features as spatially
   related rather than independently distributed.

### Visibility / viewshed research

4. **Inglis, N. C., Vukomanovic, J., Costanza, J., & Singh, K. K. (2022).**
   *From Viewsheds to Viewscapes: Trends in Landscape Visibility and Visual
   Quality Research.* Landscape and Urban Planning, 224, 104424. DOI:
   [10.1016/j.landurbplan.2022.104424](https://www.sciencedirect.com/science/article/abs/pii/S0169204622000731)
   · [US Forest Service record/PDF](https://research.fs.usda.gov/treesearch/64151)

   Particularly relevant because the review covers visibility analysis across
   archaeology, natural-resource management and planning, **including the use of
   large numbers of observer points** — which is exactly §10's route-visibility
   construction.

### Settlement / spatial modelling

5. **Sikk, K., & Caruso, G. (2024).** *Framing Settlement Systems as Spatial
   Adaptive Systems.* Ecological Modelling, 490, 110652. DOI:
   [10.1016/j.ecolmodel.2024.110652](https://www.sciencedirect.com/science/article/pii/S0304380024000413)
   · [University of Luxembourg record](https://orbilu.uni.lu/handle/10993/60319)

   The grounding for §11's position that settlements and their associated
   landmarks should **emerge from interacting environmental, spatial and
   socioeconomic processes** rather than being independently placed.

6. **Groenhuijzen, M. R. (2019).** *Palaeogeographic-Analysis Approaches to
   Transport and Settlement in the Dutch Part of the Roman Limes.* In *Finding
   the Limits of the Limes*, pp. 251–269. Springer. DOI:
   [10.1007/978-3-030-04576-0_12](https://link.springer.com/chapter/10.1007/978-3-030-04576-0_12)

   The most directly transferable of the six: it explicitly combines
   palaeogeography, rivers, settlements, forts, transport networks, least-cost
   paths and historical landscape factors in one settlement-location analysis —
   the same combination §12 and §13 ask Cartalith for.

### Spatial geography

7. **Miller, H. J. (2004).** *Tobler's First Law and Spatial Analysis.* Annals
   of the Association of American Geographers, 94(2), 284–289. DOI:
   [10.1111/j.1467-8306.2004.09402005.x](https://onlinelibrary.wiley.com/doi/full/10.1111/j.1467-8306.2004.09402005.x)

---

## 34. Recommended Cartalith Architecture

The landmark system should not become another isolated generator.

It should operate as a consumer of Cartalith's existing world-state layers:

```
                    CARTALITH WORLD STATE
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
      TERRAIN            HYDROLOGY          GEOLOGY
          │                  │                  │
          └──────────────────┼──────────────────┘
                             │
                    ANALYSIS CACHE
                             │
        ┌────────────┬───────┼────────┬────────────┐
        │            │       │        │            │
      TPI       FLOW ACC.  ROUTES  VIEWSHED   RESOURCES
        │            │       │        │            │
        └────────────┴───────┼────────┴────────────┘
                             │
                   LANDMARK CANDIDATES
                             │
                     CLASSIFICATION
                             │
                   SUITABILITY SCORING
                             │
                   SPATIAL COMPETITION
                             │
                    CAUSAL VALIDATION
                             │
                    CULTURAL / HISTORY
                             │
                       LANDMARK DB
```

This architecture is important because the expensive analytical fields should be calculated once and reused by many systems.

For example, the same viewshed field could support:

- landmark generation;
- fortress placement;
- road planning;
- settlement planning;
- military analysis;
- exploration;
- map rendering.

---

## 35. Final Design Principle

The desired result is not:

```
RANDOM LOCATION
+
RANDOM LANDMARK
```

It is:

```
WORLD CONDITIONS
        ↓
PHYSICAL POSSIBILITY
        ↓
SPATIAL SIGNIFICANCE
        ↓
HUMAN / ECOLOGICAL INTERACTION
        ↓
HISTORICAL PROCESS
        ↓
LANDMARK
```

The landmark is therefore an emergent property of the simulated world.

That is the appropriate direction for Cartalith if landmarks are intended to support worldbuilding rather than merely decorate generated maps.

The implementation should prioritize causal consistency over visual density.

A world containing 150 geographically explainable landmarks will feel substantially more coherent than one containing 1,500 procedurally scattered points of interest.

---

## Implementation Priority

**Phase 1 — Physical landmarks**

Implement: peaks, ridges, passes, cliffs, waterfalls, springs, confluences, caves, lakes, geological formations

**Phase 2 — Transportation landmarks**

Implement: fords, bridges, road junctions, portages, harbours, caravan stations

**Phase 3 — Economic landmarks**

Implement: mines, quarries, salt works, resource sites, trade depots

**Phase 4 — Settlement-linked landmarks**

Implement: forts, watchtowers, markets, temples, shrines, monuments

**Phase 5 — Historical landmarks**

Implement: battlefields, ruins, abandoned settlements, ancient roads, destroyed fortifications, historical sites

**Phase 6 — Emergent cultural history**

Allow landmarks to acquire meaning through: civilization, religion, politics, trade, war, migration, resource exploitation, historical events

At this point, landmark generation becomes part of Cartalith's world simulation rather than a standalone map-decoration system.

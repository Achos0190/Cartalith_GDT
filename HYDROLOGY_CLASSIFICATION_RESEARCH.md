# Hydrological Classification of Lakes, Seas, and Marine Basins in Procedural World Generation

> **Owner-supplied, imported verbatim 2026-09-08.** Nothing below this line has
> been edited, summarised or annotated — the same treatment
> `LANDMARK_GENERATION_RESEARCH.md`, `TERRAIN_APPEARANCE_RESEARCH.md` and
> `HETEROGENEOUS_COMPUTE_RESEARCH.md` were given, and for the same reason: a
> direction document is evidence of what the owner asked for, and an edited copy
> is no longer that.
>
> **What this proposes is a deliberate divergence from the reference, not a
> port.** `cartalith-civ`'s `build_water_bodies` is `buildWaterBodies`
> (reference HTML line 5753) and its own doc comment states the current rule:
> *"distinguishes the open OCEAN (largest connected below-sea component) from
> inland LAKES (every other below-sea component, plus above-sea depressions a
> priority-flood fill pools past `lakeDepth`, gated on local rainfall)."* That is
> size-primary — relative size rather than an absolute km² threshold, but
> size-primary — and it is **golden-tested**. See `OUTSTANDING_WORK.md` for the
> three failure modes that follow from it and for what a change costs.

---

## Abstract

Procedural terrain generators frequently classify large inland water bodies as
seas because classification is based primarily on surface area, salinity, or
proximity to map boundaries. These criteria are insufficient for geographically
plausible world generation. In physical geography, the distinction between lakes
and seas is partly hydrological, morphological, and conventional rather than
governed by a single universal size threshold.

For Cartalith, water-body classification should therefore be based primarily on
hydrological topology and connectivity, with morphology, flow characteristics,
salinity, and scale used as secondary evidence. This approach permits the
generation of very large inland lakes without incorrectly converting them into
seas, while also preventing genuinely marine basins from being classified as
lakes when the generated map represents only a portion of a larger world.

---

## 1. Problem

A procedural world may contain a water body occupying a substantial proportion
of the generated map. If classification is based on area, a large lake can be
incorrectly classified as a sea.

For example:

```
if water_body_area > threshold:
    classify as SEA
else:
    classify as LAKE
```

is geographically unreliable.

There is no universal area at which a lake becomes a sea. The Caspian Sea
demonstrates this problem particularly well: it is an enormous saline, landlocked
basin with no natural connection to the world's oceans, yet it is conventionally
classified as a sea.

Conversely, large saline lakes can remain classified as lakes.

Therefore:

> Surface area must not be the primary determinant of lake-versus-sea
> classification.

---

## 2. Physical Classification Principles

A robust classification system should evaluate water bodies in the following
order:

1. Hydrological connectivity
2. Basin topology
3. Nature of connecting channels
4. Hydrological behaviour
5. Salinity
6. Morphological characteristics
7. Surface area
8. Geographic convention

The first four should carry substantially more weight than surface area.

---

## 3. Hydrological Connectivity

The primary question should be:

> Is the water body part of the marine/oceanic hydrological system?

A water body directly connected to the ocean through marine water should
initially be classified as marine.

A water body enclosed by land and not connected to the marine system should
initially be classified as inland water.

Conceptually:

```
                         WORLD OCEAN
                              |
                    ---------------------
                    |                   |
                 MARINE              INLAND
                    |                   |
             sea / gulf /          lake / basin
             bay / strait
```

This makes connectivity a topological property rather than a visual or
area-based property.

---

## 4. Water-Body Graph

Cartalith should represent water bodies as a graph.

Each water body is a node:

```
Ocean
Sea A
Lake A
Lake B
River A
Strait A
```

Hydrological connections become edges:

```
Ocean
  |
Strait
  |
Sea
  |
River
  |
Lake
```

The graph can then be queried to determine whether a body is connected to the
ocean and what type of connection exists.

A simple connected-component test is insufficient because a river-connected lake
should not automatically become a sea.

---

## 5. River Connection Versus Marine Connection

A lake may have an outlet that ultimately reaches the ocean:

```
Lake → River → Ocean
```

This does not make the lake a sea.

The connecting feature should therefore be classified.

### River connection

Characteristics may include:

- substantial directional discharge
- measurable channel gradient
- fluvial channel morphology
- predominantly one-way freshwater transport
- relatively narrow channel compared with the basin

### Marine connection

Characteristics may include:

- marine water exchange
- tidal influence
- bidirectional exchange
- low net flow relative to water exchange
- marine channel morphology
- connection to an established marine basin

Therefore Cartalith should distinguish:

```
LAKE
  ↓
RIVER
  ↓
OCEAN
```

from:

```
OCEAN
  ↓
STRAIT
  ↓
MARINE BASIN
```

---

## 6. Closed and Endorheic Basins

An inland water body without an outlet to the ocean should be classified as an
enclosed or endorheic basin.

For example:

```
Rivers
  ↓
LAKE
  ↓
evaporation
```

Such a body can be:

- freshwater
- saline
- hypersaline

Salinity should therefore be stored as an independent physical property rather
than being used as a synonym for "sea."

---

## 7. Scale

Surface area should be treated as a supporting characteristic.

A water body occupying one third of the generated world may still be a lake:

```
┌─────────────────────────────┐
│                             │
│      LAND                   │
│                             │
│   ┌───────────────────┐     │
│   │                   │     │
│   │       LAKE        │     │
│   │                   │     │
│   │      1/3 WORLD    │     │
│   │                   │     │
│   └───────────────────┘     │
│                             │
│                 OCEAN       │
└─────────────────────────────┘
```

If the basin is genuinely enclosed and inland, its size does not make it a sea.

Consequently, Cartalith should not use a rule such as:

```
area > X km² → sea
```

as the principal classification mechanism.

---

## 8. Map-Boundary Problem

Procedural maps introduce an additional complication.

A water body reaching the edge of the generated map cannot necessarily be
classified as enclosed.

For example:

```
┌─────────────────────────────┐
│                             │
│        WATER BODY           │
│                             │
│                             │
│~~~~~~~~~~~~~~~~~~~~~~~~~~~~~│
└─────────────────────────────┘
              ↑
         map boundary
```

The water may continue outside the generated region.

Therefore Cartalith should distinguish between:

```
ENCLOSED
```

and:

```
MAP-BOUNDED / UNRESOLVED
```

A water body touching the map boundary should not automatically be classified as
a lake.

Recommended states:

```
OCEAN
SEA
INLAND_LAKE
ENDOREIC_LAKE
MARINE_BASIN
MAP_BOUNDED_WATER
UNKNOWN
```

The final classification can be resolved when a larger world region becomes
available.

---

## 9. Recommended Cartalith Data Model

Each water body should retain physical properties independently of its final name
or classification.

```
WaterBody {

    id

    classification

    area
    perimeter

    mean_depth
    maximum_depth

    elevation

    salinity

    ocean_connected
    map_boundary_contact

    inflow_count
    outflow_count

    marine_connection
    river_connection

    tidal_influence

    basin_type

    coastline_length
}
```

This separation is important.

For example:

```
classification = INLAND_LAKE
salinity = HIGH
ocean_connected = false
basin_type = ENDOREIC
area = very_large
```

is entirely valid.

The system does not need to force physical properties to conform to the
classification name.

---

## 10. Recommended Classification Pipeline

Cartalith should classify water bodies after terrain and hydrological
generation.

```
HEIGHTMAP
    ↓
WATER DETECTION
    ↓
CONNECTED WATER COMPONENTS
    ↓
WATER-BODY IDENTIFICATION
    ↓
HYDROLOGICAL GRAPH
    ↓
CONNECTION ANALYSIS
    ↓
BASIN ANALYSIS
    ↓
MARINE / INLAND CLASSIFICATION
    ↓
RIVER / STRAIT / ESTUARY ANALYSIS
    ↓
SALINITY + MORPHOLOGY
    ↓
FINAL WATER-BODY CLASSIFICATION
    ↓
NAMING / CARTOGRAPHIC REPRESENTATION
```

This prevents rendering terminology from contaminating the underlying physical
simulation.

---

## 11. Proposed Decision Logic

A simplified decision tree is:

```
Water body
    |
    ├── Connected to marine water?
    │       |
    │       ├── YES
    │       │    |
    │       │    ├── Open global water → OCEAN
    │       │    |
    │       │    └── Partially enclosed → SEA / MARINE BASIN
    │       |
    │       └── NO
    │            |
    │            ├── Closed basin → LAKE
    │            |
    │            └── Endorheic → ENDORHEIC LAKE
    |
    └── Map boundary contacted?
            |
            └── YES → MAP-BOUNDED / UNRESOLVED
```

The final classification can then use morphology, salinity, size, and
historical/geographical naming conventions as secondary modifiers.

---

## 12. Consequences for Procedural Generation

This system produces more realistic edge cases without special-case rules.

Cartalith can generate:

- enormous freshwater lakes
- enormous saline lakes
- endorheic basins
- inland seas
- marginal seas
- narrow marine straits
- gulfs and bays
- river-connected lakes
- ocean-connected marine basins
- partially generated oceans
- lakes intersecting map boundaries

The crucial distinction is that size no longer determines identity.

A lake occupying 33% of the generated world remains a lake if its hydrological
topology identifies it as an enclosed inland basin.

Likewise, a relatively small marine basin connected to the ocean can remain
marine despite having an area smaller than some lakes.

---

## 13. Conclusion

The lake-versus-sea problem should be treated as a hydrological topology problem
rather than a surface-area problem.

For Cartalith, the preferred hierarchy is:

```
CONNECTIVITY
      ↓
BASIN TOPOLOGY
      ↓
CONNECTION TYPE
      ↓
HYDROLOGICAL BEHAVIOUR
      ↓
SALINITY
      ↓
MORPHOLOGY
      ↓
SIZE
      ↓
NAMING CONVENTION
```

The central rule is:

> A water body's classification should describe its physical relationship to the
> world's hydrological system, not merely its size.

This allows Cartalith to generate physically plausible enormous inland lakes
while avoiding the opposite error of classifying map-truncated marine bodies as
lakes.

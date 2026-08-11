# MVP scope: terrain only

The boundary for the first milestone. Anything not listed in scope is out until
`ROADMAP.md` says otherwise — including the adjacent thing that looks easy while
you are already in that part of the code.

## In scope: the full terrain pipeline

Script block 1 of `Cartalith Gen1 v*.html` (root `CLAUDE.md`, "Engine (block 1)
essentials"). Read the live file for the implementation; this is the map, not the
territory.

```
continentality field
  → buildTectonicSubstrate()
      warp → plates → stress → flexure → base blur + age → heterogeneity → resistance → orogeny
  → height formula
  → normalize
  → volcanism + craters
  → flow(area)          initial accumulation
  → climate             temperature, wind, rainfall
  → flow(discharge)     rivers accumulate runoff
  → erosion             droplet, stream-power, thermal
  → hydrology           river network extraction
```

1. **Tectonic substrate** — plate assignment (JFA Voronoi, matching
   `assignPlates()`), boundary stress, flexure, terrane age, heterogeneity,
   resistance, orogeny. Deterministic from a seed.
2. **Height formula** — reproduce it; do not improve it (`DECISIONS.md` §7).
3. **Normalization** — min-max stretch to `[0,1]`.
4. **Volcanism and craters** — the point-feature placement and carving passes.
5. **World-structure archetypes** — the continentality/fragmentation/tectonic-energy
   presets, including the sea-level histogram re-anchoring that makes them behave
   as named. Read the v1.25 CHANGELOG entry first: sea level does not follow from
   continentality without that step.
6. **Climate** — temperature (latitude, lapse rate, axial tilt, rotation), wind,
   rainfall. Ocean-current terrain coupling (`computeOceanCurrent`/`deflectFlow`,
   v1.77–v1.82 and v2.10) is a stretch goal. **If deferred, say so in the port's
   own docs** rather than dropping it quietly.
7. **Erosion** — droplet, stream-power, thermal. Choose the parallelism strategy
   deliberately and record why (`ARCHITECTURE.md`, threading).
8. **Hydrology** — flow accumulation, Strahler ordering, river polyline tracing,
   and real-km-aware channel width (v2.07): a river widens with map width as it
   does today.
9. **Sea level** — the threshold and its interaction with archetypes.
10. **Basic 2D rendering** — colour and hillshade, enough to verify what was
    generated. Not the JS engine's full renderer: no multi-octave grain, NPR
    styles, splat textures, or LOD pyramid.
11. **Minimal UI** — seed, resolution, map width, generate.
12. **Reading the HTML app's `.zip` saves** — open a real export and display its
    terrain fields. Reading only; writing is later (`SAVEFILE_COMPAT.md`). A real
    export doubles as golden data, which is why this earns MVP status.

## Out of scope

| Excluded | Where it lives today |
|---|---|
| Civilisation and politics — factions, settlements, trade, journey planner | block 2 |
| Asset Library | block 3 |
| Urban morphology | block 4 |
| Sculpt editor | block 1 |
| 3D terrain view | `DECISIONS.md` §4 |
| LOD tile pyramid, deep-zoom rendering | — |
| NPR styles, parchment/ink/watercolour, splat texturing | — |
| Multi-resolution baking, atlas export | — |

**Writing saves is out**, along with GeoJSON export, PNG tile export, and every
other persistence format. Point 12 grants reading one specific thing; it is not a
general save/load licence.

## Done means all seven

1. At a fixed seed, resolution, and map width, the height, temperature, rainfall,
   and flow fields match golden data within `PARITY_TESTING.md`'s tolerance.
2. The world renders as a recognisable 2D map — land and water distinct, biome
   colouring plausible, rivers visible.
3. It builds as a Windows `.exe` **and the owner has run it on Windows**.
4. It builds as an Android `.apk` **and the owner has installed and run it**.
5. Map width visibly scales feature size as it does today — as a consequence of
   parity, not a separate implementation.
6. A changelog entry records what was ported, how it was verified, and what was
   deferred.
7. It opens a real HTML-app `.zip` and renders that save's terrain, checked
   against what the HTML app shows for the same file — not merely "it did not
   crash."

Criteria 3 and 4 need the owner's hardware (`DECISIONS.md` §5). A cloud session
can confirm the build succeeded and packaged correctly; it cannot certify that
the thing runs.

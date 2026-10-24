# Volcano appearance and form — research

**What this file is.** The research requested by the owner's note
`VOLCANO_APPEARANCE_NOTE_2026-10-19.md` (owner, 2026-10-19: *"volcanic rock
and volcanos depending the type shape and age have different colours. Some are
green and overgrown, others are still with an active caldera. Do research into
this and what determines shape, size and visual aspects of volcanos."*).
It is **research, not a scope**: it carries no status column and claims no
progress. Progress lives in `cartalith-native/docs/STATUS.md`; the routed row
belongs in `OUTSTANDING_WORK.md`. Agent-produced, 2026-10-19, from the tree at
`b9e2db42`.

**Evidence tags.** Every claim carries one:

- **[cited]**: from a named source, listed in §9 with a link. Where a source
  was read only as an abstract or summary, the tag says so.
- **[measured-here]**: read at a named symbol in `cartalith-native/` at
  `b9e2db42`, or computed here from those constants (the script is described
  where it is used). Nothing in this file was rendered; no frame was captured.
- **[judgement]**: a labelled judgement, not sourced. Every RGB target in §6 is
  a judgement unless it says otherwise.
- **no source found**: searched for and not found. The number is left out
  rather than guessed.

---

## 1. Summary

1. **Shape follows magma viscosity and how the vent is fed; size follows
   supply and lifetime.** Low-silica basalt flows far and builds broad shields
   (H/W 0.01-0.1, mean slopes 1-15°). Andesite to dacite builds steeper
   composite cones that rarely stand more than 3 000 m above their base.
   Monogenetic scoria cones are born at H/W ≈ 0.18 with a crater about 0.4 of
   the base. Silicic systems make domes and large collapse calderas
   [cited: Grosse & Kervyn 2018; Davidson & De Silva 2000; Wood 1980; Porter
   1972]. Grosse et al. (2014) found **a continuum, not discrete classes**,
   across 759 composite volcanoes [cited, abstract].
2. **Age reads first in the profile, and the profile degrades at a rate set
   by climate.** Cinder cones lose H/W from 0.179 (<0.1 Ma) to 0.125
   (0.2-0.7 Ma) to 0.058 (0.8-3 Ma) in arid Arizona. Their slopes fall from
   30-33° to about 26°. Etna's cones lost 32→21° in 450 years. Andean
   stratovolcanoes erode at 7-22 m/Ma, and at 66-112 m/Ma while young
   [cited: Wood 1980; Karátson et al. 2012, abstract].
3. **Colour is age × climate more than rock type.** Fresh basalt glass is
   among the darkest natural surfaces: VNIR reflectance 0.02-0.04, which is
   sRGB 39-56 by the conversion in §6.1 [cited: Reeder thesis; measured-here
   conversion]. Hot oxidation turns scoria red-brown with hematite [cited:
   Ejima et al. 2017]. Weathering brightens lava and adds hematite over decades
   [cited: Rose et al. 2011, abstract]. Cover then depends on rain:
   - a 1937 cone at about 2 260 mm/yr was gullied and maturely vegetated by
     1971;
   - Jorullo (1759) carried trees within 90 years;
   - an Arizona cone about 50 000 years old at about 400 mm/yr holds little
     plant life;
   - Hawaiian dryland flows younger than 300 years are about 95 % bare.

   [cited: Wood 1980; Kinney et al. 2015]
4. **What the engine has today** [measured-here]:
   - Edifice shapes: three morphologies (`Cinder`, `Strato`, `Shield`), chosen
     by setting, with a collapsed summit, gullies and tile-time furrows.
   - No volcano type beyond those three.
   - No per-volcano activity state.
   - No age in years; per-edifice age is discarded after stamping.
   - **Every volcanic cell renders as the andesite palette**, the hotspot
     basalt shields included.
   - No weathering or vegetation rule that knows a cell is volcanic.
   - Lava glow only as user paint.
   - Measured against the cited morphometry:
     - the engine's small cones are about 10× too wide and 3-4× too flat;
     - its mid-class stratocones sit on the shield side of Grosse & Kervyn's
       0.10 H/W / 12° boundary;
     - every stratocone carries a caldera of about a quarter of its radius,
       where Pike & Clow's commonest stratocone class has a crater under
       1-1.5 km.
5. **The proposal** (§6-§8): a render-side classification (setting × kind ×
   age × activity × climate → palette and texture). It is built first because
   it moves no generated output. Morphometry corrections and new edifice types
   follow, each behind its own owner ruling. Every bar is judged at
   ×1/×4/×16/×64.

---

## 2. What determines shape and size, per class

### 2.1 The controls

- **Magma composition and viscosity.** USGS glossary silica bands [cited]:
  - basalt 45-53 % SiO₂, "dark in color (gray to black)";
  - andesite 54-62 %, "medium dark";
  - dacite 62-69 %, "light";
  - rhyolite ≥69 %, "light".

  Differentiation raises SiO₂ and viscosity. Low-viscosity basalt flows far
  from the vent and builds shields. More viscous lavas make short flows and a
  steeper cone. Volatile content plus viscosity raises explosivity and the
  share of pyroclastics [cited: Davidson & De Silva 2000, §II].
- **How the vent is fed.** Grosse & Kervyn's clustering of 158 shields
  identifies the feeding system as the control on shield morphology: central,
  along a linear structure, or spatially diffuse [cited: Grosse & Kervyn 2016
  EGU abstract, the precursor of the 2018 paper]. Tibaldi (1995) relates
  pyroclastic-cone elongation to the geometry of the feeding fracture, from
  1 315 Quaternary cones in five rifts and belts [cited, abstract only].
- **Tectonic setting** [cited: Davidson & De Silva 2000]. Convergent margins
  give H₂O-rich, differentiated basaltic andesite to andesite, and so
  composite cones. Intraplate settings give the largest composites (Ararat,
  Damavand, Kilimanjaro), "where magma production and supply rates are higher
  than those associated with arcs".
- **Lifetime and growth.**
  - Composite cones grow in spurts. Cone-building episodes are "generally
    <100 ka" to reach maturity. Systems stay active up to about 500 ka: Mount
    Adams dates to 520 ka, Baker/Rainier/Hood/Shasta to about 300 ka.
  - Eruption recurrence runs from near-daily (Sakurajima, Sangay), through
    0.1-10 years (Fuego, Klyuchevskoy, Arenal) and hundreds of years
    (Pinatubo, St Helens, Fuji), to millennia (Baker, Hood, Rainier, Adams).

  [cited: Davidson & De Silva 2000, §III]
- **Edifice collapse.** Sector collapses remove 10-15 % (Irruputuncu) to
  about 30 % (Tata Sabaya) of an edifice. "Regardless of how extensively an
  edifice is eviscerated, the conical shape is reestablished very rapidly":
  Parinacota has virtually rebuilt itself since a collapse 13 000 years ago
  [cited: Davidson & De Silva 2000].
- **A size limit.** Arc composite edifices "rarely exceed 3000 m and the vast
  majority … are between 2000 and 2500 m", with volumes up to about 200 km³.
  The explanation offered is that a growing edifice's load eventually
  overcomes the overpressure that lets magma reach the summit [cited:
  Davidson & De Silva 2000].

### 2.2 Per class

Figures are cited unless tagged otherwise. Ranges are given where the source
gives them, and "no source found" stands where it does not.

| Class | Magma / setting | Height | Basal diameter / H:W | Flank slope | Summit depression | Lifetime / frequency |
|---|---|---|---|---|---|---|
| **Scoria / cinder cone** | basalt to basaltic andesite; monogenetic; volcanic fields, rift and shield flanks | USGS: rarely more than ~1 000 ft (~300 m) above the surroundings; Parícutin built to 1 200 ft in 9 years [cited: USGS GIP] | fresh **H/W = 0.179** (Hco = 0.179 Wco, N = 9, r = 0.98, matching Porter's Mauna Kea cones); measured SFVF cones 0.5-2 km base; Sunset Crater W = 1.6 km, H = 0.3 km [cited: Wood 1980] | fresh 30-33°; Manchurian cones 28-32°; spatter cones at Réunion average 39° [cited: Wood 1980] | crater diameter ≈ **0.40 × basal diameter**, independent of degradation [cited: Wood 1980, after Porter 1972] | monogenetic; Parícutin 9 years of activity [cited: USGS GIP] |
| **Stratovolcano / composite** | andesite-dacite, also basalt and rhyolite; arcs (also intraplate) | arc composites mostly 2 000-2 500 m base to summit, rarely >3 000 m; USGS: up to 8 000 ft above base [cited: Davidson & De Silva 2000; USGS GIP] | Grosse & Kervyn place the shield/strato boundary at about **H/W 0.10**; composite statistics are in Grosse et al. 2014's supplement (no median retrieved: **no source found** for a typical value) | boundary at about 12° mean slope [cited: Grosse & Kervyn 2018, abstract]; a typical upper-flank angle: **no source found** | "crater-only" stratocones have a crater under 1.0-1.5 km; above that it is a caldera class [cited: Pike & Clow 1981, Table 1] | episodes <100 ka; systems up to ~500 ka [cited: Davidson & De Silva 2000] |
| **Shield** | low-silica basalt, effusive; hotspots, rifts, some arcs | Cascades/N. California shields 1 500-2 000 ft high, 3-4 mi across; Mauna Loa 13 677 ft above sea level, >28 000 ft above the deep ocean floor [cited: USGS GIP] | **H/W 0.01-0.1**, volumes 0.1 to >1 000 km³ [cited: Grosse & Kervyn 2016 abstract / 2018] | mean slopes mostly 1-15° [cited: Grosse & Kervyn 2018, abstract]; SE Mauna Loa about 6°, under 5° lower down [cited: Macdonald 1971, USGS GQ-957]; several large oceanic shields are "systematically much steeper" than the 4-8° usually attributed to Hawaiian shields [cited: Grosse & Kervyn 2016] | Pike & Clow's large-shield classes all carry a caldera "much broader than conduit" [cited] | many eruptions over tens to hundreds of ka [cited: Davidson & De Silva 2000] |
| **Lava dome** | dacite-rhyolite, viscous; inside craters and calderas or on flanks | Novarupta: 200 ft high [cited: USGS GIP]; Peléan spines up to about 300-350 m [cited: Mori & Oshima 2013, abstract] | Novarupta 800 ft across [cited: USGS GIP]; aspect ratio 0.125-0.35 for growing domes (9 domes) [cited: Mori & Oshima 2013]; Blake's four types: low dome (torta), Peléan, coulée, upheaved plug [cited: OSU Volcano World] | **no source found** | may carry a summit depression (Pike & Clow class D) [cited] | months to years per dome; vulnerable to collapse when overfilling a crater (Unzen, Soufrière Hills) [cited: Davidson & De Silva 2000] |
| **Caldera / ash-flow caldera ("supervolcano")** | silicic (rhyolite/dacite ignimbrite) or basaltic shield collapse | negative landform | USGS: calderas "roughly circular depressions 1 to 15 miles in diameter" [cited: USGS GIP]; Yellowstone **45 × 85 km**, from the >1 000 km³ Lava Creek Tuff about 631-640 ka [cited: USGS/NPS] | — | resurgent domes (Yellowstone: Mallard Lake and Sour Creek); post-collapse rhyolite flows and domes [cited: NPS]; caldera diameter statistics in Geyer & Martí 2008's CCDB (**no source found** for the summary numbers in this pass) | — |
| **Fissure / flood basalt** | tholeiitic basalt, LIPs | Columbia River flows 10 to >300 ft thick each; total up to 15 000 ft [cited: USGS] | Columbia River ~164 000 km², ~175 000 km³, >300 flows [cited: OSU / USGS]; Deccan ~500 000 km² today [secondary: Wikipedia] | plateau with stepped ("trap") topography; sheet lobes mode 17 m, thin lobes 8 m [cited: Frontiers 2020 Deccan study, summary] | none (fissures) | — |
| **Hotspot island chain** | basalt shields | see shield | age-progressive along plate motion: Kauai 5.6-3.8 Ma, Oahu 3.4-2.2, Molokai 1.8-1.3, Maui 1.3-0.8, Hawaiʻi <0.7 Ma [cited: USGS "Volcanoes of Hawaii"] | older islands more weathered, with permanent streams and alluvial valleys [secondary, not a geologic source] | — | — |
| **Submarine / seamount** | basalt | — | Pike & Clow class KO: small submarine shield with large summit depression [cited] | — | — | — |
| **Maar / tuff ring** | phreatomagmatic, basaltic; where magma meets groundwater, often on the lower flanks or ring plain [cited: Davidson & De Silva 2000] | negative (maar) or low (tuff ring) | maar craters several hundred metres to a few km across, tens to ~200 m deep; Lake Purrumbete 2 800 m [cited: Frontiers 2017 mini-review]; diatreme to ~2-2.5 km below [cited: Lorenz 2007, via review] | — | maar crater cuts below ground level; tuff ring rarely does [cited: Pike & Clow 1981] | monogenetic |

### 2.3 Degradation with time

- **Cinder cones** [cited: Wood 1980]:
  - **Profile.** Height, H/W and slope fall with age; crater/base does not.
    SFVF H/W is 0.179 at <0.1 Ma, 0.125 at 0.2-0.7 Ma and 0.058 at 0.8-3 Ma.
  - **Slope.** It falls about 15°/Ma for Tappan-age cones and about 8°/Ma
    averaged over 2 Ma. A Tappan cone at H/W 0.12 is predicted to stand at
    about 26°, against 30-33° fresh.
  - **Climate sets the rate.** Etna's cones fell 32→21° in 450 years.
    Massif Central cones 4 Ma old are reduced to necks.
  - **Mechanism.** Cinders weather to clay, which rainfall gullies. Fresh
    cinder is too permeable for runoff, so gullies wait for soil.
  - **Ash mantling accelerates it by orders of magnitude.** Paricutin ash on
    Cerro de Cutzato did "in a century as much degradation as would normally
    have required thousands of years".
  - **Colton's stages** (SFVF):
    - 5: steep, no gullies, scoria "little oxidized";
    - 4: clay and sparse vegetation;
    - 3: gullies;
    - 2: dikes exposed, rim subdued, soil for mature vegetation;
    - 1: plug only.
- **Composite cones** [cited: Davidson & De Silva 2000, Table III]. Five
  geomorphic stages:
  1. fresh, "often dark, pristine lava flows and summit crater visible, sharp
     profile";
  2. small gullies, degraded crater, "dark apron gone";
  3. no crater, established gullies, planèzes start;
  4. deeply incised, large planèzes, U-shaped glacial valleys where glaciated;
  5. barely recognisable, radial symmetry the only clue.

  Profiles evolve from a straight cone with a restricted base to a
  concave-up steady state with a wide talus/ring-plain apron. Debris
  avalanches extend that apron "several tens of kilometers". In arid regions
  20 Ma volcanoes stay "well preserved and easily recognizable". Mount St
  Helens deposits gained 10 m gullies within months; Pinatubo's within days;
  Katmai's (cooler, drier) are only moderately eroded after ~100 years.
  Hydrothermal alteration weakens the vent region into "an enormous bowl of
  altered volcanic material" (Mutnovsky), and does so long after activity
  ends.
- **Stratovolcano erosion rates** [cited: Karátson et al. 2012, abstract]:
  - 33 Central Andean volcanoes up to 14 Ma old;
  - <10-20 m/Ma overall;
  - 7-9 m/Ma in the hyperarid Puna, 13-22 m/Ma in more humid South Peru;
  - 66-112 m/Ma on young Late Quaternary cones, falling about tenfold after a
    few Ma.
- **Shield islands.** The age progression of §2.2 is the visible sequence:
  young, dark and smooth Hawaiʻi; deeply dissected Kauai [cited: USGS ages; the
  dissection itself only from a secondary source].
- **"Euler / Mark" shield scaling** (named in the brief): **no source found**
  under those names in this pass. Grosse & Kervyn (2018) is the shield
  morphometry used here.

---

## 3. Visual aspects

### 3.1 Rock colour by lithology, fresh

| Material | Colour | Evidence |
|---|---|---|
| Basalt | "dark in color (gray to black)" | [cited: USGS glossary] |
| Fresh glassy basalt (spatter, scoria, quenched surfaces) | VNIR reflectance **0.02-0.04**; brighter samples at flow margins show alteration and devitrification; an albedo change of about 0.03 tracks 28-81 % glass | [cited: Reeder, U. Idaho thesis, Jordan Craters, Oregon] |
| Andesite | "medium dark" | [cited: USGS glossary] |
| Dacite, rhyolite | "light in color" | [cited: USGS glossary] |
| Obsidian | "dense volcanic glass, usually rhyolite … typically black" | [cited: USGS glossary] |
| Pumice | "highly vesicular … typically silicic"; rhyolitic pumice floats | [cited: USGS glossary]; its colour: **no source found** in the glossary (light grey to white is a [judgement]) |
| Scoria | "vesicular … frothed up by escaping gases" | [cited: USGS glossary]; black when reduced, red-brown when oxidised (next row) |
| Red scoria | hot oxidation of Fe²⁺ to **hematite**; Kasayama cone interior red-brown, outer wall black; oxidation above 800 °C | [cited: Ejima et al. 2017]; fissure 8 (Kīlauea 2018) cone interior red-brown by the same mechanism [cited: Hawaii Tribune-Herald on USGS drone imagery] |
| Pahoehoe / ʻaʻā | pahoehoe smooth and ropy; ʻaʻā rubbly clinker | [cited: secondary glossaries]; the texture difference is the visible one; a colour difference: **no source found** |

### 3.2 Colour with age and climate

- **Bare lava weathers and brightens.** Pre-historic lavas with no biological
  cover show "a prominent increase in spectral reflectance in the 400–760 nm
  range" on Tenerife [cited: Portsmouth study, abstract]. On Cerro Negro, iron
  oxides weathered to hematite at the surface by 9-20 %, against 1-9 % glass
  devitrification, over 76 years [cited: Rose, Ramsey & Watson 2011, GSA
  abstract].
- **Soil.** Andosols, the volcanic-ash soils, are named from Japanese *an*,
  "black". Dark, organic-rich surface horizons are typical [cited: ISRIC
  reference soils; HU Berlin]. So a weathered volcanic slope before forest is
  dark brown-black, not the red-brown of a tropical laterite [judgement drawn
  from the cited soil colour].
- **Vegetation succession** sets most of the colour of a volcano older than
  centuries:
  - humid, about 2 260 mm/yr: Vulcan (PNG, born 1937) "deep gullies and
    mature vegetation" by Ollier & Brown 1971 [cited via Wood 1980];
  - about 1 950 mm/yr, cooler: near Parícutin the same state needs >100 years,
    and Jorullo (1759) had trees within 90 years [cited: Wood 1980];
  - wet montane Hawaii, about 4 000 mm/yr: a rain-forest chronosequence of
    eight ʻaʻā flows, 8 to ~9 000 years [cited: Kitayama, Mueller-Dombois &
    Vitousek 1995; abstract only, per-age cover not retrieved];
  - another Mauna Loa study: lichens and moss first, *Lycopodium* and
    *Dicranopteris* dominant by about 50 years, *Metrosideros* later [cited:
    search summary only, the source not identified (**unverified**)];
  - dry Hawaii, about 500 mm/yr: flows <300 years are 94-96 % bare ground,
    the vegetation entirely *Metrosideros* woodland with canopies 14-15 m; ~65 ka substrates reach 69 % lateral cover
    of 6 m shrubland. Pahoehoe vegetates faster than ʻaʻā (water pools in its
    cracks) [cited: Kinney et al. 2015];
  - arid, about 400 mm/yr: a Merriam-age SFVF cone (~50 ka) "harbors little
    plant life and no gullies" [cited: Wood 1980].
- **Arid versus humid** is therefore the dominant axis: three to four orders
  of magnitude in time-to-green across about 400 to 2 300 mm/yr of rain
  [cited data points above; the generalisation is a judgement].
- **Snow and ice.** Composite stage 2 already shows moraines; stage 4 has
  U-shaped glacial valleys where glaciated [cited: Davidson & De Silva 2000].
  Specific snow-line figures for volcanoes: **no source found** in this pass.
  The engine already derives snow from temperature (§4.4), so none is needed.

### 3.3 Hydrothermal alteration, fumaroles and sulphur

- Acid-sulfate alteration (H₂S oxidised to sulfuric acid attacking rock)
  gives kaolinite ("the white kaolinite alteration"), white silica veins,
  charcoal-grey mud coloured by pyrite and iron sulfides, red/pink mud pots,
  and "dramatic yellow, orange, and red rocks and sediments" from the
  alteration of rhyolite [cited: USGS YVO "Alterations go"].
- Altered summit rocks carry jarosite, pyrite, alunite and kaolinite at
  Rainier [cited: USGS]. Near Sherman Crater fumaroles (Mount Baker): alunite
  and silica [cited: USGS PP 1022-E]. Limonitic minerals (goethite, hematite,
  jarosite, lepidocrocite) accompany acid-sulfate alteration [cited: USGS
  Marysvale study].
  - That jarosite is yellow and hematite red is standard mineralogy, but the
    retrieved texts did not state it: **[judgement]**.
- **Native sulphur** coats vent rocks yellow; molten sulphur reads red to
  orange; burning sulphur gas gives blue flames at night (Kawah Ijen; ignition
  above about 360 °C) [cited: Smithsonian Magazine, Science Friday; popular
  sources].
- Alteration weakens the vent bowl and persists after activity ends [cited:
  Davidson & De Silva 2000]. So a pale, yellow-white, stained summit is a sign
  of a **dormant as well as an active** system.

### 3.4 Active crater glow, lava lakes and lava flows by age

- Incandescence by colour (USGS HVO: "a crude estimate of temperature")
  [cited]:
  - red ≈ 600-800 °C;
  - orange ≈ 800-1 000 °C;
  - yellow ≈ 1 000-1 200 °C;
  - Kīlauea erupts at about 1 170 °C.
- A different scheme (dark red 475 °C, orange 900 °C, white >1 150 °C) is
  attributed to Kilburn 2000 [secondary, not verified].
- Active lava from above is a dark chilled crust with incandescence in the
  cracks. This is the engine's own material model, already documented with a
  citation to Lockwood & Hazlett 2010 [measured-here:
  `cartalith_spatial::colour_paint::material` module doc].
- **Flows by age** follow §3.2:
  - glassy near-black;
  - then a brown-red hematite tint and brightening within decades;
  - then lichen and moss;
  - then shrub or forest, at a rate set by rain.

  Pahoehoe greens before ʻaʻā [cited: Kinney et al. 2015].

### 3.5 Ash fall and caldera lakes

- Ash mantles older cones. It strips vegetation, and gullying through it
  outpaces gullying of the original surface [cited: Wood 1980, on Parícutin
  ash; Segerstrom]. Visually: a fresh grey ash blanket over a green cone,
  then rapid rilling [judgement from the cited process].
- Acidic active crater lakes driven by subaqueous fumaroles are "often
  turquoise or emerald green". The blue comes from colloidal sulphur
  scattering, the green from dissolved ferrous iron. Yudamari (Aso) moved
  blue-green → green over 2000-2007 [cited: Ohsawa et al. 2010]. Ruapehu's
  Crater Lake runs blue, green and blue-grey [cited: search summary of a
  Ruapehu study, unverified].
- Pasternack & Varekamp (1997) classify volcanic lakes genetically (acid
  hyperbrine to neutral-dilute). Colour is not a class criterion [cited:
  UC Davis project page]. A neutral, dilute caldera lake reads like any deep
  lake [judgement].

---

## 4. What the engine already has (verified at the symbols, `b9e2db42`)

Every entry is [measured-here]: opened at the named symbol unless it says
otherwise.

### 4.1 The edifice model

- `cartalith_engine::VolcanismParams`:
  - `count`, `age` (one **world-wide** 0-1 slider);
  - `provinces`, `exclude_transform`;
  - `edifice_model` (on in the app via `cartalith_godot::params::defaults`,
    off in `WorldParams::defaults`, `DECISIONS.md` §7l-ii);
  - `fill_edifice_pits` (Ruling CO).
- Parity defaults: `count: 20, age: 0.40`.
- `cartalith_terrain::EdificeKind`:
  - `Cinder`, `Strato` and `Shield`, and no other kind;
  - profile `H*(1 - t^a)^b`;
  - `edifice_shape` gives:
    - `Cinder {a 1.0, b 1.15, rim 0.38, floor 0.16, depth 0.30}`;
    - `Strato {a 1.0, b 2.60, rim 0.24, floor 0.11, depth 0.35}`;
    - `Shield {a 2.0, b 1.40, rim 0.12, floor 0.05, depth 0.10}`;
  - every edifice gets a flat-floored collapse, 6-48 radial gullies
    (`EDIFICE_GULLY_SPACING_CELLS = 4`) and fbm lumps.
- `edifice_kind(setting, size_roll)`:
  - Arc gives `Strato` for the large size rolls, else `Cinder`;
  - Rift gives `Shield`, else `Cinder`;
  - Hotspot gives `Shield` always;
  - Unclassified falls back to the 70/25/5 roll.
- `place_sized_volcano` draws:
  - 70 %: H 200-1 000 m, R 2-10 km;
  - 25 %: H 1 000-3 000 m, R 10-30 km;
  - 5 %: H 3 000-7 000 m, R 30-80 km;
  - H capped at `0.95 × peak_m`;
  - R clamped by `clamp_feature_radius_cells` and floored at 2 cells in
    `stamp_one_volcano`.
- **Per-edifice age exists but is discarded.**
  - `place_province_volcanoes` draws `age = rng × volc_age`. Hotspot chains
    are age-progressive along plate motion:
    `(n/(count-1))·0.85 + rng·volc_age·0.4`.
  - That age shapes the stamp (`h × (1 - age·0.5)`, the summit exponent
    rounding via `EDIFICE_AGE_ROUNDING`) and `volcanic_field = (1-t)(1-age)`.
  - It is **not** recorded: `PlacedEdifice` holds `setting, cx, cy, r, h`
    only, and `VolcanoTrace` is "transient".
- `cartalith_terrain::edifice_pits` fills flank pits from the final field,
  keeping summits (`SUMMIT_T`) as real closed basins. `EdificeFrame::from_volcanic_field`
  recovers centre and radius from `volcanic_field`.
- `cartalith_terrain::edifice_furrows` adds tile-time radial furrows at deep
  zoom (Ruling CO item 3), read from that recovered frame.
- **No activity state anywhere.** A grep for `dormant|extinct|is_active` over
  the crates' sources finds nothing volcano-related.
- **Unit note.** `stamp_one_volcano` normalises `h = height_m / peak_m · 0.9`,
  while the engine's metres per normalised unit are `peak_m / (1 - sea)`
  (`cartalith_terrain::geology::m_to_norm`; `edifice_furrows`' doc). At the
  stamp the drawn metres are therefore scaled by `0.9 / (1 - sea)`: 1.55× at
  sea 0.42, before age damping. Whether later stages (`orogen::soft_ceiling`,
  erosion) change that end-to-end was **not measured here**. VA-0 must measure
  real edifice heights before the slope comparison below is trusted.

### 4.2 The engine's morphometry against the cited figures

**Method.** `scratchpad/slopes.py`, not committed, evaluates the `edifice_shape`
profile at the drawn H and R, at age 0, with the 1.55× factor above **not**
applied. Mean flank slope runs from the rim to the toe [measured-here].

| Draw (H, R) | Kind it builds | H/W | max / mean flank slope | Cited target |
|---|---|---|---|---|
| small mid draw (600 m, 6 km) | `Cinder` | 0.050 | 6.1° / 5.3° | H/W 0.179, slope 30-33°, base 0.5-2 km |
| small tallest (1 000 m, 2 km) | `Cinder` | 0.25 | 28.2° / 25.0° | (only this corner of the draw reaches cinder slopes) |
| mid draw (2 000 m, 20 km) | `Strato` | 0.050 | 9.5° / 3.7° | strato side of the boundary: H/W >0.10, mean >12° |
| mid tallest (3 000 m, 10 km) | `Strato` | 0.15 | 26.7° / 10.9° | — |
| large mid draw (5 000 m, 55 km) | `Shield` | 0.045 | 7.8° / 5.8° | H/W 0.01-0.1, 1-15°: **inside** |

What this shows:

- Shields are in range.
- Cinder cones are the wrong size class entirely. A drawn radius of 2-10 km
  means bases of 4-20 km, against real bases under about 2 km.
- The typical stratocone is shield-shaped by Grosse & Kervyn's boundary.
- The 1.55× factor, if it survives to the final field, raises every slope but
  does not fix the cinder footprint.
- The existing doc on `edifice_shape` already says this ("the footprints are
  the reference's and they are far too wide").
- The `Strato` collapse (rim 0.24 R, depth 0.35 H, "Crater-Lake-shaped") puts
  a caldera of about 0.5 × the base diameter on **every** stratocone. Pike &
  Clow's SC class (Mayon, Cotopaxi) has a crater under 1-1.5 km.

### 4.3 Lithology and rock colour

- **The render's lithology is the seven-class `cartalith_civ::build_lithology`.**
  Any cell with `volcanic_field > 0.35` returns code 2 ("volcanic arc /
  hotspot → andesite"), whatever the setting. A hotspot basalt shield
  therefore paints with `rock_andesite`, `(97,91,92)/(117,111,111)/(139,133,133)`,
  not `rock_basalt`, `(52,55,60)/(69,73,79)/(91,95,102)` (`render.rs`
  `TerrainAppearance` defaults; `LITHO_PALETTE_ORDER`).
- The microtexture table on the `lith` match does give basalt "broad lava-field
  patchiness" and andesite "volcanic ash speckle, darkened where cinder
  accumulates".
- **The geology-first column** (`cartalith_terrain::geology`, `GEOLOGY_FIRST_SCOPE.md`)
  is on in the app (`p.geology_model = true`):
  - it already distinguishes `Rock::PlateauBasalt` (hotspot/rift core),
    `Rock::Andesite` (arc core, `t < ARC_CORE_T`) and `Rock::Tuff` (arc flank);
  - it stores `rasters/volcanic_setting.u8` (codes 0-4) and the user's
    `rock_painted.u8` (GF-8, Ruling BJ).
  - The renderer does **not** read it. No `Rock::` or `GeologyColumn`
    reference exists in `render.rs`.
  - `geology_processes` is off in the app.
- **Satellite relief slope rock** (`b9e2db42`):
  - `TerrainAppearance::rock_slope_warm_deg = 40` on Satellite relief only;
  - `rock_knee_deg(t, veg, cold 6°, warm 40°)` and `rock_slope_mix_held`
    hold covered slopes;
  - the commit reports 24601-archipelago's volcano rock share at ×4 falling
    0.92 → 0.38 [cited: commit message, not re-measured here];
  - `rock_slope_warm_deg`'s doc records 0.38-0.57 at ×4-×64, against
    0.04-0.26 with the tint off.
  - So volcano flanks are greener but still read partly grey.
  - The rule knows slope, warmth and cover. It does not know volcanic age or
    activity, so a young bare cone in a warm climate is held green by warmth,
    which is the opposite of the real case [judgement from §3.2].

### 4.4 Snow, colour paint, landmarks

- **Snow.** `snow_seas`, `snow_perm` and `snow_glac` palettes, and
  `snow_shed_frac`, are driven by temperature and slope. A high volcano gets a
  snow cap with no special handling.
- **Colour paint lava** (`cartalith_spatial::colour_paint::material`, Rulings
  CN and CS; `COLOUR_PAINT_SCOPE.md` §18-§21):
  - an `EmissiveMaterial` (`ramp`, `heat`, `crack_density`, `glow_radius_km`,
    `heat_along`) on a user mark;
  - `GlowRamp::Fire` stops from `(55,3,0)` at 0.18 to `(255,248,200)` at 1.0,
    in blackbody order, a labelled judgement;
  - plus `Ice` and `Toxic` ramps;
  - glow is added after the light, the crust lit like ground, with cracks
    self-similar over scale;
  - **user-placed only**: nothing places it from the world model.
- **Landmarks** (`cartalith_civ::landmark`): one physical kind,
  `volcanic_feature` ("Volcanic feature", default cap 10, needs viewshed).
  - `pool_volcanic` scores local maxima of `volcanic_field` above
    `VOLCANIC_MIN_ACTIVITY = 0.35`, with "volcanic activity" weighted 0.45
    among the `VOLCANIC_TERMS`.
  - It has no sub-kind: cone, caldera, lava field, fumarole field and hot
    spring are all one kind.
  - "Activity" there is `volcanic_field`, i.e. `(1-t)(1-age)`, a proximity ×
    youth proxy, not an eruptive state.

### 4.5 Can the generator produce each type today?

| Type | Today | Evidence |
|---|---|---|
| Shield | **yes**, in range (§4.2) | `EdificeKind::Shield` |
| Stratovolcano | **yes**, too flat at typical draws, always with a caldera | `EdificeKind::Strato` |
| Scoria/cinder cone | **shape yes, size no**: 2-10 km radius | `EdificeKind::Cinder`, `place_sized_volcano` |
| Monogenetic volcanic field (clusters) | partly: the provinces cluster vents | `place_province_volcanoes` |
| Hotspot chain, age-progressive | **yes**, in placement; the age is then discarded | §4.1 |
| Large collapse caldera / ash-flow plain / resurgent dome | **no** | no kind, no ignimbrite sheet |
| Lava dome / coulée | **no** | no kind |
| Maar / tuff ring | **no** | no kind (impact craters are a separate model, `stamp_craters`) |
| Fissure / flood basalt plateau | **no** as landform; `Rock::PlateauBasalt` exists only in the geology column | `geology.rs` |
| Sector-collapse amphitheatre, debris-avalanche hummocks | **no** | — |
| Compound / nested cones | only by chance overlap (max of `volcanic_field`, sum of height) | `stamp_one_volcano` |
| Seamount | incidental: edifices can stand below sea level; no submarine class | — |
| Activity (active / dormant / extinct) | **no** | §4.1 |
| Crater/caldera lakes | **incidental**: summits stay closed basins (`edifice_pits`), so the water classifier may fill them; lake colour is generic | `edifice_pits` doc |

---

## 5. Gap list

Ranked by visible effect per unit of risk [judgement].

| # | Gap | Moves generated output? | Evidence |
|---|---|---|---|
| G1 | Volcanic cells all paint as andesite grey; hotspot and rift basalt never reads dark; the geology column's `PlateauBasalt`/`Andesite`/`Tuff` never reaches the render | no (render) | §4.3 |
| G2 | No age in years and no per-edifice age kept, so no weathering, oxidation or vegetation-succession colour; a 2 Ma cone and a fresh one colour alike given the same climate | no if the age is recorded beside the field without changing stamps | §4.1, §3.2 |
| G3 | The Satellite relief slope-rock hold uses warmth and cover, not volcanic age, so young bare cones go green in warm climates and old forested ones can still grey | no (render) | §4.3 |
| G4 | No activity state, so no procedural crater glow, fumarole staining, sulphur, alteration bleaching or steam | no (render plus a recorded attribute) | §4.1, §4.4 |
| G5 | Cinder cones 2-10 km in radius, about 10× too wide and 3-4× too flat (H/W 0.05 against 0.179) | **yes**: height field, owner ruling | §4.2 |
| G6 | Every stratocone carries a caldera of about 0.5 × base; most real ones have a crater under 1-1.5 km | **yes** | §4.2 |
| G7 | Typical stratocone H/W 0.05 / mean slope 4°, on the shield side of the 0.10 / 12° boundary | **yes** | §4.2 |
| G8 | Missing kinds: lava dome, maar/tuff ring, large caldera with lake and resurgent dome, flood-basalt plateau steps, sector-collapse amphitheatre | **yes** | §4.5 |
| G9 | Gully spacing does not depend on type or degradation stage; the Table III planèze stages are absent (existing row: "type-dependent gully spacing and a gully depth capped by the cone") | **yes** | `OUTSTANDING_WORK.md`, Ruling CO item 2 |
| G10 | Crater lake colour: no acid/turquoise variant | no (render) | §3.5 |
| G11 | One landmark kind for all volcanic features | no (landmarks) | §4.4 |
| G12 | Unverified stamp-to-final height factor (`0.9/(1-sea)`) | measure first | §4.1 |

---

## 6. A classification the engine could drive

### 6.1 Inputs, all available or cheaply recordable

- **setting**: `VolcanicSetting`, already saved as `volcanic_setting.u8`.
- **kind**: `EdificeKind` (`Cinder`/`Strato`/`Shield`), from
  `edifice_kind(setting, size_roll)`. Record it.
- **age**: per edifice, mapped from the existing 0-1 draw to years. One
  proposed map is log-scaled, `years = 10^(1 + 5.5·age)`, so that age 0 gives
  10 years and age 1 about 3 Ma [judgement]. The span matches the cited ages:
  Wood's 0.001-6 Ma, Davidson & De Silva's ≤500 ka, Hawaiian <0.7-5.6 Ma.
- **activity**: active / dormant / extinct. Proposed default [judgement]:
  - **active**: age < 0.05;
  - **dormant**: age < 0.4, matching composite systems active up to about
    500 ka [cited] on the log map;
  - **extinct**: everything else;
  - an owner override per volcano is a later GUI row.
  - **Superseded 2026-10-24 by owner Ruling CX item 2** (implementation note, not research): on the log map above, 0.4 is about 1.6 ka, not 500 ka, and every arc and rift draw (`rng × 0.40`) fell below it. The engine now maps the draw onto two log pieces with 500 ka at a draw of 0.25 and cuts activity on the years (active under 50 years, dormant under 500 ka); `cartalith_terrain::volcano_attr::respread_age_years`.
- **climate**: the cell's existing rainfall and temperature, the same inputs
  `material_weights` uses.
- **zone** within the edifice, from `t = d/R` (recovered by `EdificeFrame`):
  - crater or caldera floor `t < floor`;
  - wall `floor..rim`;
  - upper flank `rim..0.5`;
  - lower flank / apron `0.5..1`.

**The sRGB conversion used in the table.** sRGB 8-bit from linear reflectance
ρ uses `255·(1.055·ρ^(1/2.4) - 0.055)`. ρ = 0.02 / 0.03 / 0.04 gives
39 / 48 / 56 [measured-here, arithmetic]. So the cited fresh-glass
reflectance is about **sRGB 40-56 grey**. That is darker than the shipped
`rock_basalt` mid `(69,73,79)`, which is appropriate for a weathered rather
than a fresh surface.

### 6.2 The rule table

RGB values are **[judgement]** targets for the Satellite-relief look, unlit
albedo before the relief light. Each is placed against the cited descriptions
in §3 and the shipped palette so they sit in one system. The texture column
names the existing expression it would reuse.

| Kind × age band | Climate | Zone | Target albedo (sRGB) | Texture rule | Evidence for the colour |
|---|---|---|---|---|---|
| any basaltic (Shield; Rift/Hotspot Cinder), **<100 yr** | any | flank | `(40,41,45)`, glassy, slightly blue | basalt lava-field patchiness; pahoehoe smooth sheets / ʻaʻā rubble as two roughness levels | reflectance 0.02-0.04 [cited] → sRGB 39-56 [measured-here] |
| Cinder, **<100 yr** | any | upper flank, crater wall | `(46,40,40)` black scoria, with crater-interior oxidation `(118,52,38)` | ash speckle (andesite expression) at scoria grain | black outer wall, red-brown interior [cited: Ejima 2017; fissure 8] |
| basaltic, **100 yr-10 ka** | arid (<500 mm) | flank | `(62,58,56)` → `(84,70,62)`, hematite tint growing with age | patchiness + sparse dark lichen dots | brightening and hematite with age [cited: Tenerife; Rose et al.]; ~95 % bare at <300 yr dry [cited: Kinney 2015] |
| basaltic, **100 yr-10 ka** | humid (>1 500 mm) | flank | vegetation palette (`wood_trop` / `wood_temp`) at cover `c(age, rain)`; bare fraction `(55,48,42)` Andosol-dark | forest canopy (`canopy_texture`, Ruling CI) where `c > 0.5` | trees within 90 yr at 1 950 mm; mature at ~34 yr at 2 260 mm [cited: Wood 1980] |
| Strato, **<10 ka** | any | upper flank | andesite `(104,98,98)` with tephra-grey fans `(128,124,120)` | ash speckle; radial furrows (`edifice_furrows`) | "often dark, pristine lava flows", stage 1 [cited: Davidson & De Silva] |
| Strato, **10-500 ka** | humid | lower flank / apron | vegetation by climate; ring-plain apron greener than the cone | gullies → planèzes (stage 3), furrow density by stage | stages 2-3 [cited] |
| Strato, **>500 ka** | humid | all | vegetation; summit relief broken, no crater colour | deep gullies, planèzes (stage 4) | stage 4 [cited] |
| Strato / Cinder, **>500 ka** | arid | all | weathered rock `(132,104,86)` with tuff-pale gully floors `(170,160,140)` | shale/sandstone-style layering on gully walls | 20 Ma arid cones stay recognisable [cited]; the colour is a judgement |
| any, **dormant or active** | any | crater / wall | alteration bleaching `(214,208,192)` (kaolinite/alunite white) mixed with ochre `(176,138,62)` (jarosite) and red `(140,62,44)` (hematite) | patchy radial staining down-wind of vents | minerals [cited: USGS YVO, Rainier, Baker]; the colours of each mineral [judgement] |
| any, **active** | any | vent rim | native sulphur `(222,204,70)` patches | small blotches at vents | yellow sulphur coatings [cited: popular sources] |
| any, **active** (lava lake or open vent) | any | crater floor | crust `(34,33,38)` (the shipped Lava preset) + `GlowRamp::Fire` emission, heat by sub-type: lava lake 0.85 (yellow-orange, ~1 100 °C), open vent 0.6 (orange), cooling dome 0.35 (red) | the existing emissive crack network, placed procedurally instead of by paint | USGS colour bands [cited]; the heat mapping [judgement] |
| caldera lake, **active or dormant (acid)** | any | water | turquoise `(64,168,160)` → emerald `(70,140,96)` | flat; a pale sulphur slick | [cited: Ohsawa et al. 2010] |
| caldera lake, **extinct (neutral)** | any | water | the generic lake palette | — | [judgement, §3.5] |
| any, **above the snow line** | cold | any | existing snow palettes; on **active** cones, ash-dusted snow `(200,196,190)` | existing `snow_shed_frac` | ash on snow: [judgement] |
| Rhyolitic dome / pumice (future kind) | any | dome | pale grey `(176,172,166)`, obsidian flow bands `(28,28,32)` | blocky; spine highlight | dacite/rhyolite "light" and obsidian "black" [cited: USGS glossary] |

### 6.3 The cover function

The vegetation cover a flank reaches, `c(age_years, rain_mm, temp)`, is
**[judgement]** fitted by eye to the cited points:

- 50 % cover at about 50 years at ≥2 000 mm;
- about 150 years at 1 500 mm;
- >5 000 years at 500 mm (Kinney: 4-6 % cover at <300 yr; still mixed
  woodland at 3-5 ka);
- >50 000 years at ≤400 mm (Wood).

A log-time logistic, `c = smoothstep(log10(τ50(rain)) - 1, log10(τ50(rain)) + 1, log10(years))`,
with `log10 τ50` interpolated through those four points, then capped by the
climate's own biome cover. This replaces nothing: it multiplies the existing
`material_weights` cover on volcanic cells only.

### 6.4 Slope and size targets the shape side should hit

These would apply if the owner rules on G5-G7 [cited figures, §2.2-§2.3]:

| Kind | H/W fresh | Mean flank slope fresh | Crater/base | With age |
|---|---|---|---|---|
| Cinder | 0.18 (≥0.15 for <0.1 Ma) | 30-33° | 0.40 | 0.125 at 0.2-0.7 Ma, 0.058 at 0.8-3 Ma; slope −8 to −15°/Ma arid, faster humid |
| Strato | >0.10 | >12° | crater <1-1.5 km unless a caldera class is drawn | stage table; apron widens (concave-up) |
| Shield | 0.01-0.1 | 1-15° (Hawaiian 4-8°) | caldera "much broader than conduit" | dissection with island age |

---

## 7. Proposed milestones (VA-0…VA-7)

All bars are judged at **×1, ×4, ×16 and ×64** (the owner's hard requirement:
zoom gains detail at every level) on a fixed seed set. The set includes the
24601 archipelago world that `b9e2db42` measured, plus one arid and one humid
world with all three kinds. Per `MISTAKES.md`:

- pixel thresholds name and **force** their palette and look;
- every change measures a **control** frame;
- "measured" figures are re-produced from the command shown;
- render-only milestones prove that every other look is **byte-identical**.

| Milestone | What | Generated output | Bars |
|---|---|---|---|
| **VA-0 Volcano census harness** | Per world, per edifice: kind, setting, age, **final** H and W from the field (resolving G12), H/W, mean and max flank slope, crater/base. Per zoom: rock / vegetation / snow share, colour variance and distinct-colour count inside the edifice disc | none | it runs on ≥5 worlds; it prints the §4.2 table from real fields; the four zoom frames are saved as evidence |
| **VA-1 Record the edifice attributes** | Persist per-edifice kind, age (0-1 and years) and activity as a small table plus a per-cell winner raster, beside `volcanic_setting.u8`. A save without them reloads with **the keys absent**, not defaulted (never encode "no value" as a plausible value) | none: by control flow, the field hash identical | field FNV identical on all census worlds; an old save loads with the attributes absent and every consumer dashes them with a reason |
| **VA-2 Volcanic lithology reaches the render** | `build_lithology`'s code 2 splits by setting and zone: hotspot/rift → basalt; arc core → andesite, arc flank → tuff/tephra (a new eighth palette slot); or the renderer reads the GF column directly | none (render) | hotspot-shield rock pixels sit in the basalt palette's hue/value band at all four zooms; non-volcanic cells are byte-identical; every other look is byte-identical |
| **VA-3 Age × climate weathering and cover** | §6.3's `c(age, rain, temp)` on volcanic cells, plus the bare-lava oxidation ramp; the Satellite slope-rock hold reads volcanic age, so young bare stays rock and old covered stays green | none (render) | **young-basalt bare share ≥0.8** and **old-humid vegetation share ≥0.7** on the census worlds at ×4-×64 (judgement thresholds, measured against VA-0's control); the two read differently at ×1 (mean albedo ΔL* ≥15); distinct-colour count inside an edifice disc rises ×1 → ×4 → ×16 → ×64 |
| **VA-4 Activity: glow, staining, sulphur, lake colour** | Procedural placement of the existing `EmissiveMaterial` on active craters; alteration and sulphur staining by zone; acid caldera-lake colour | none (render) | an unreached volcano draws the same bytes; glow pixels appear only on active edifices; at ×64 the crack network shows ≥2 octaves more than at ×16 (the material's own `MAX_OCTAVES` law); daytime glow luminance stays below a stated cap so it does not read as paint |
| **VA-5 Morphometry corrections** (owner ruling) | Cinder footprint and H/W to the cited 0.18 / 0.40 with slope by age; Strato crater-versus-caldera split (caldera only for a drawn minority); Strato H/W above 0.10; type-dependent gully spacing and stage-dependent dissection (the existing Ruling CO item 2 row) | **yes**: app-side, `false` in `WorldParams::defaults` | VA-0 census: fresh cinder median H/W within 0.15-0.21 and crater/base within 0.35-0.45; strato median H/W >0.10; shield still 0.01-0.1; flank-pit lakes no worse than Ruling CO's 10 per world; switch-off field identical |
| **VA-6 New kinds** (owner ruling per kind) | lava dome (aspect 0.125-0.35); maar / tuff ring (crater hundreds of m to a few km, below grade); large caldera with lake and resurgent dome; flood-basalt step plateau | **yes** | each kind's census morphometry inside its §2.2 range; each reads distinctly at ×1 and gains structure through ×64 |
| **VA-7 Landmark sub-kinds** | `volcanic_feature` split or annotated: active cone, caldera, crater lake, fumarole field, lava field, using VA-1's attributes | landmarks only | every placed feature names its sub-kind and the evidence, or dashes it with a reason |

The order is VA-0 → VA-1 → VA-2 → VA-3 → VA-4, all render-side and
reversible, then VA-5 → VA-6, which move the height field and each need a
ruling, with VA-7 alongside [judgement].

---

## 8. Owner questions, with recommended defaults

1. **Is volcano colour a render-only classification first?** Default: **yes**
   (VA-0 to VA-4). It moves no generated output and needs no golden
   re-baseline.
2. **Age in years, per volcano.** Record the existing per-edifice draw and map
   it log-scale to 10 years … 3 Ma (§6.1)? Default: **yes**. The global
   `volc.age` slider keeps its meaning as the upper bound of the draw.
3. **Activity states.** Active / dormant / extinct derived from age (active
   <0.05, dormant <0.4), with a per-volcano override later? Default: **yes,
   derived**. Hotspot chains then have one active end, as Hawaiʻi does.
4. **Which looks first?** Default: **Satellite relief only**, then the
   cartographic looks as a separate pass. Every other look stays
   byte-identical until then.
5. **Glow by day?** Default: **a subtle daytime glow on active craters and
   lava lakes**, capped (VA-4's bar), with a full night glow left to a future
   lighting row.
6. **The slope-rock hold on volcanoes.** Should volcanic age override warmth
   and cover, so young bare cones stay dark rock on any slope and old
   vegetated cones hold green to the warm knee? Default: **yes** (VA-3). It
   addresses the "volcano flanks still grey" render-target row from the
   volcano side.
7. **Morphometry (G5-G7), which moves the height field.** Authorise VA-5 as an
   app-side divergence like §7l-ii? Default: **yes, after VA-0 measures the
   final heights**: cinder footprint and Strato crater first, H/W second.
8. **New kinds, in what order?** Default: **caldera + lake + resurgent dome**
   (it covers the owner's "active caldera" case), then **lava dome**, then
   **maar/tuff ring**, then **flood-basalt steps**.
9. **The eighth lithology palette.** Add a **tephra/tuff** slot (pale grey-buff)
   rather than reusing sandstone? Default: **yes**. Arc flanks then read as
   ash, not desert.

---

## 9. Sources

Read in full or in part for this file. "Abstract" means only the abstract or
a summary was read.

- USGS, *Principal Types of Volcanoes* (General Interest Publication). https://pubs.usgs.gov/gip/volc/types.html
- USGS Volcano Hazards Program glossary (basalt, andesite, dacite, rhyolite, obsidian, pumice, scoria, caldera, maar, shield, stratovolcano, fumarole). https://www.usgs.gov/volcanoes/glossary
- USGS FAQ, *How hot is a Hawaiian volcano?* (incandescence colour bands). https://www.usgs.gov/faqs/how-hot-hawaiian-volcano
- USGS YVO, *Alterations go: hydrothermal alteration at Yellowstone*. https://www.usgs.gov/news/alterations-go-hydrothermal-alteration-yellowstone
- USGS, Mount Rainier altered summit rocks, mineralogic and stable-isotope study (abstract). https://www.usgs.gov/publications/preliminary-mineralogic-and-stable-isotope-studies-altered-summit-and-flank-rocks-and
- USGS Professional Paper 1022-E, Sherman Crater, Mount Baker (summary). https://pubs.usgs.gov/pp/1022e/report.pdf
- USGS, Marysvale altered volcanic rocks, multispectral mapping (abstract). https://pubs.usgs.gov/publication/70010976
- USGS, *Volcanoes of Hawaii* island ages. https://pubs.usgs.gov/gip/hawaii/page12.html
- USGS / NPS, Yellowstone caldera (45 × 85 km, Lava Creek Tuff). https://www.nps.gov/articles/000/resurgent-calderas.htm ; https://pubs.usgs.gov/publication/70197423
- USGS Oregon Water Science Center, Columbia River Basalt Group background. https://or.water.usgs.gov/projs_dir/crbg/background.html
- Macdonald, G.A. (1971), USGS Geologic Quadrangle GQ-957 (SE Mauna Loa slope). https://pubs.usgs.gov/gq/0957/report.pdf (summary)
- Pike, R.J. & Clow, G.D. (1981), *Revised classification of terrestrial volcanoes and catalog of topographic dimensions, with new results of edifice volume*, USGS Open-File Report 81-1038. https://pubs.usgs.gov/of/1981/1038/report.pdf (Table 1 and Table 5 read; Table 2's OCR was too noisy to compute medians from)
- Wood, C.A. (1980), *Morphometric analysis of cinder cone degradation*, J. Volcanol. Geotherm. Res. 8: 137-160. Full text read. https://people.wou.edu/~taylors/g407/wood_1980.pdf
- Wood, C.A. (1980), *Morphometric evolution of cinder cones*, J. Volcanol. Geotherm. Res. 7: 387-413. Cited via the paper above, not read.
- Porter, S.C. (1972), *Distribution, morphology, and size frequency of cinder cones on Mauna Kea volcano, Hawaii*, GSA Bulletin 83(12): 3607-3612. doi:10.1130/0016-7606(1972)83[3607:DMASFO]2.0.CO;2 (via Wood 1980)
- Davidson, J. & De Silva, S. (2000), *Composite volcanoes*, in Sigurdsson (ed.), *Encyclopedia of Volcanoes*, Academic Press. Chapter text read. https://booksite.elsevier.com/9780126431407/netscape4/pdfs/CH43.pdf
- Grosse, P., Euillades, P.A., Euillades, L.D. & van Wyk de Vries, B. (2014), *A global database of composite volcano morphometry*, Bull. Volcanol. 76: 784. doi:10.1007/s00445-013-0784-4 (abstract)
- Grosse, P. & Kervyn, M. (2018), *Morphometry of terrestrial shield volcanoes*, Geomorphology 304: 1-14. doi:10.1016/j.geomorph.2017.12.017 (abstract); and the EGU2016-11888 abstract (full abstract read)
- Tibaldi, A. (1995), *Morphology of pyroclastic cones and tectonics*, J. Geophys. Res. 100. doi:10.1029/95JB02250 (abstract)
- Karátson, D., Telbisz, T. & Wörner, G. (2012), *Erosion rates and erosion patterns of Neogene to Quaternary stratovolcanoes in the Western Cordillera of the Central Andes: an SRTM DEM based analysis*, Geomorphology 139-140: 122-135. doi:10.1016/j.geomorph.2011.10.010 (abstract)
- Ejima, T. et al. (2017), *Precipitates within olivine phenocrysts in oxidized andesitic scoria from Kasayama volcano, Hagi, Japan*, J. Mineral. Petrol. Sci. 112(3): 116-126. doi:10.2465/jmps.161219 (abstract)
- Reeder, (University of Idaho MS thesis), *Crystallinity and albedo: in-situ VNIR analysis* (Jordan Craters). https://objects.lib.uidaho.edu/etd/pdf/Reeder_idaho_0089N_11933.pdf (summary)
- Rose, S., Ramsey, M.S. & Watson, I.M. (2011), *Analysis of basaltic lava flows at Cerro Negro volcano, Nicaragua using spaceborne TIR data*, GSA Abstracts with Programs 43(1). https://gsa.confex.com/gsa/2011NE/webprogram/Paper185249.html (abstract)
- Kitayama, K., Mueller-Dombois, D. & Vitousek, P.M. (1995), *Primary succession of Hawaiian montane rain forest on a chronosequence of eight lava flows*, J. Veg. Sci. 6(2): 211-222. doi:10.2307/3236216 (abstract)
- Kinney, K.M. et al. (2015), *Primary succession on a Hawaiian dryland chronosequence*, PLoS ONE 10(6): e0123995. https://pmc.ncbi.nlm.nih.gov/articles/PMC4467043/
- Ohsawa, S. et al. (2010), *Color change of lake water at the active crater lake of Aso volcano, Yudamari, Japan*, Limnology 11(3): 207-215. doi:10.1007/s10201-009-0304-6 (summary)
- Pasternack, G.B. & Varekamp, J.C. (1997), volcanic lake classification. UC Davis project page: https://pasternack.ucdavis.edu/research/projects/volcanic-lakes/vl-classification-and-modeling (journal pages not verified in this pass)
- Geyer, A. & Martí, J. (2008), *The new worldwide collapse caldera database (CCDB)*, J. Volcanol. Geotherm. Res. 175: 334-354. doi:10.1016/j.jvolgeores.2008.03.017 (existence and fields only; no summary statistics retrieved)
- Maar crater dimensions: *Frontiers in Earth Science* 2017 mini-review. https://www.frontiersin.org/articles/10.3389/feart.2017.00068/full (summary); Lorenz (2007), J. Volcanol. Geotherm. Res. 159: 285-312 (via review)
- Lava domes: Oregon State University Volcano World, *Types of lava domes* (Blake's classification). https://volcano.oregonstate.edu/types-lava-domes ; Mori & Oshima, JpGU 2013 abstract SVC48-P33. https://www2.jpgu.org/meeting/2013/session/PDF/S-VC48/SVC48-P33_E.pdf
- Andosols: ISRIC World Soil Museum reference profiles. https://museum.isric.org/monoliths/reference-soil-costa%20rica-12
- Sulphur colours and Kawah Ijen: Smithsonian Magazine. https://www.smithsonianmag.com/smart-news/indonesian-volcano-contains-electric-blue-streams-molten-sulfur-180956512/ ; Science Friday. https://www.sciencefriday.com/articles/picture-of-the-week-kawah-ijen-volcano/ (popular sources)
- Smithsonian Global Volcanism Program: the volcano-type pages returned HTTP 403 to this agent. GVP types enter here only through Grosse et al. 2014's use of them.

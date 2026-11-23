# Coastal setback: research, engine mapping and a proposed rule

**Research and design only, 2026-11-23.** This document defines; it does not
track. Status lives in `cartalith-native/docs/STATUS.md`.

**Why it exists.** The open row in `OUTSTANDING_WORK.md` read *"Ruling FK
leftovers: harbours still over 23 m from water (483920 #5 at 384, #10 at
2 048); hamlet 154:136 about 400 m inland"*. The owner answered it on
2026-11-23:

> "This isnt necesarily bad. Sometimes coastal settlements where built a bit
> off the coast for good reasons; less influence from storms, not near a
> cliff. Whilst retaining access as they arent beyond walking distance. Maybe
> Sonnet or Opus can provide a bit of research to support this and help in
> building code around it."

The owner then set the scope:

- *"But thats generally for sea's/coasts not really for rivers."*
- Revised: *"oh the floodplain for rivers is a solid point, keep it."*

So the scope is:

- **Sea and coast:** the full setback rule.
- **Rivers:** the bank rule of Rulings HG, HM and HN stays the default. A river
  town may sit back **only where the terrain shows a floodplain**, on higher
  ground, with its landing or bridge at the river. Storms, cliffs and raiders
  are coast reasons, and this document gives them no river role (section 1.8).
- **Lakes:** excluded. Section 1.8 explains why.

**Evidence tags.**

- **[source]** means a page was read, or a search returned the cited page's
  own wording. Where only a search summary was read, the tag says
  **[source, search summary]**.
- **[recalled]** means from memory, not re-checked this session.
- **[inference]** means my reasoning.
- **[code]** means read at the named symbol in this tree, at `cbc49e73` (origin/main).

---

## 0. Summary

1. **Historical coastal settlements were often set back from the shore, at
   walking distance from a landing they used.** The reasons vary: piracy and
   raiding, storms and surges, cliffs, marsh and malaria, and silting or a
   shifting shore. Sections 1.1 to 1.5 give the sources.
2. **The paired "port + mother town" is common and well attested.**
   - **Same-day walking distance (0.5–5 km):**
     - Lynton above Lynmouth: 500 ft up, joined by an 862 ft cliff railway.
     - The Cycladic choras above their ports: Schinoussa's is 1.2 km away;
       Iraklia's is 4.5 km by road.
     - Sóller and Port de Sóller, about 3 km apart.
     - Burnham Overy Town and Burnham Overy Staithe, about a mile apart.
     - Canterbury and Fordwich, about 2 miles apart.
   - **Larger outports (4–7 km):**
     - Athens and Piraeus: the Long Walls ran about 6 km.
     - Ravenna and Classe, about 4 km.
     - Bruges and Damme, about 7 km.
     - Exeter and Topsham, about 6 km.
3. **Thucydides states the piracy reason directly** (1.7): the old towns were
   built away from the sea, and later towns on the shore.
4. **Walking distance has a well-anchored scale.**
   - About half an hour each way. This is Marchetti's one-hour daily travel
     budget, about 2.5 km at 5 km/h on the flat.
   - Tobler's hiking function adjusts that speed for slope.
   - Bracton's 6⅔-mile market rule is the outer bound for a day's round trip
     with business at the far end. It is not a daily walk to the shore.
5. **Settlements sit right on the water when the water is the economy and the
   harbour shelters.** Fishing villages in sheltered clefts (Staithes) and
   trading ports are the cases. The setback is a response to a hazard. It is
   not a default.
6. **Rivers:**
   - Towns sit on the flood-free terrace with the crossing or landing at the
     river. Oxford stands on a gravel terrace above flood level, with its
     crossings at the terrace edge.
   - Where there is no terrace, they sit on raised ground inside the floodplain:
     the Nile's villages on levees and mounds.
   - Bankfull flow, which reaches the floodplain, recurs about every 1.5 years
     (Leopold, Wolman and Miller).
7. **The engine already has most of the inputs:**
   - the flood field;
   - the floodplain landform class;
   - a wetland mask;
   - cliff slopes;
   - a wind field;
   - per-town heights.
   **It also has the reference's own precedent:** `civ_snap_to_water_edge`
   refuses flood cells, *"the bank terrace, not the channel"*. **It lacks an
   exposure or fetch field on the civ side and any notion of a landing as its
   own feature.**
8. **The proposal:**
   - **No cell moves.** The cell, the badge, roads, ports and sea lanes stay as
     they are.
   - Inside the town box (1.7 x 1.25 km), the town's core (its market) stands on
     the nearest hazard-free ground, up to a cap.
   - The **quay or landing stays at the water** and is joined to the core by a
     path.
   - The marker follows the market, as Ruling HM already rules for rivers.
   - The Ruling FK bar is restated so it applies to the quay. The town body is
     then allowed back by its recorded setback.
9. **Two `DECISIONS.md` section 7p switches, both layout-only:**
   - `urban_coastal_setback`;
   - `urban_floodplain_setback`.
10. **A conflict needs a ruling.** The proposal changes Ruling FK item 1 (*"no
    harbour stands inland of its own water"*). It also changes
    `harbour_front_market`'s documented promise never to move a market farther
    from its water. That needs an owner ruling and a `DECISIONS.md` entry
    (question Q1).

---

## 1. Research

### 1.1 Raiding and piracy

- **Thucydides, *History* 1.7** [source, search summary of the Perseus/Perseids
  text, Crawley translation]:
  - The old towns, *"on account of the great prevalence of piracy, were built
    away from the sea, whether on the islands or the continent"*.
  - Later towns, founded once navigation was safer, had *"the shores becoming
    the site of walled towns, and the isthmuses being occupied for the purposes
    of commerce"*.
  - The pirates *"used to plunder one another, and indeed all coast
    populations"*.

  This is the oldest written statement of the setback-for-raiding pattern, and
  of its reversal once the sea was safe.
  <https://cts.perseids.org/read/greekLit/tlg0003/tlg001/1st1K-eng1/1.7.1>
- **The Cyclades' chora above its port** [source, search summary]:
  - **Schinoussa:** *"built on a hill, just 1.2 km from the port, Chora (or
    Panagia) cannot be seen from the sea at a glance. Its settlers chose this
    place out of fear for the pirates"*.
  - **Iraklia:** a 4.5 km road links the port with Chora.
  - **Serifos:** the Chora sits on a rock 230 m above the harbour.
  - **Kea:** the Chora is on the inland-facing slope, *"completely invisible
    from the open sea"*.

  Sources: Small Cyclades guide (Naxos municipality), Greek Reporter,
  Culture Trip.
  <https://www.naxos.gr/wp-content/uploads/2018/04/SMALL-CYCLADES-GUIDE-EN.pdf>,
  <https://greekreporter.com/2026/06/04/spectacular-greek-choras-need-see/>,
  <https://theculturetrip.com/europe/greece/articles/how-pirates-shaped-the-design-of-this-greek-island>.
  The Greek words *skala* or *gialos* for the port were **not** confirmed by
  these sources [recalled].
- **Sóller and Port de Sóller, Mallorca:** about 3 km apart (Wikipedia; other
  guides give 2 miles to 4 km). The pirate raids are documented, including the
  1561 attack. That the town was **founded** inland *because of* pirates rests
  only on travel guides [source, search summary; causation weak].
  <https://en.wikipedia.org/wiki/S%C3%B3ller>
- **Calabria's hill towns and their *marina* frazioni.** Santa Caterina dello
  Ionio's people are said to have moved uphill around 1000 AD to escape
  Saracen raids, and its seaside counterpart is Santa Caterina dello Ionio
  Marina [source, travel guide, weak].
  <https://summerinitaly.com/guide/santa-caterina-dello-ionio>.
  The wider Calabrian pattern of a hill town plus a later *Marina di X* is
  [recalled]. It was not verified here.
- **Norse.** A shared boathouse (*naust*) on the shore served farms set back
  from it. An inland farm *"often shared a naust on a neighbouring farm that
  bordered the sea"* [source, a single archaeology blog: weak].
  <https://arkeologi.blogspot.com/2005/12/boathouses-in-post-medieval-norway.html>.
  No source here gives a typical farm-to-naust distance.

### 1.2 Storms, surge and tsunami

- **Old and New Winchelsea:**
  - Storms from the 1230s, and the great storm of 1287, wrecked the old town
    on the shingle.
  - Edward I's new town was laid out on Iham hill, *"three miles from the
    sea"*. The burgesses had moved there by about 1292.
  - Sources differ on the founding date (1281 charter, 1288 hand-over).

  [source, search summary: winchelsea.com, Wikipedia, National Trust]
  <https://www.winchelsea.com/the-town-story/history/new-winchelsea>.
  The relocated town kept a landing at the foot of its hill, the Strand
  [recalled].
- **Aneyoshi tsunami stone (Iwate, Japan):**
  - The village moved uphill after the 1896 tsunami.
  - It moved back to the shore, and was destroyed again in 1933.
  - A stone then marked the line not to build below. The 2011 waves stopped
    short of it.

  [source, search summary: Smithsonian, Wikipedia]
  <https://www.smithsonianmag.com/smart-news/century-old-warnings-against-tsunamis-dot-japans-coastline-180956448/>.
  This is the cleanest modern statement of *"the core above the hazard line,
  the boats below it"*. It also shows the pull back to the water once the
  memory fades.
- **Modern coastal setback law:**
  - The Mediterranean ICZM Protocol (Barcelona Convention, 2008), Article 8-2,
    requires a **100 m** setback zone, with adaptation clauses for local
    conditions.
  - Studies argue that 100 m can be too little for dune coasts under sea-level
    rise.

  [source, search summary: PAP/RAC explanatory report]
  <https://www.marinespecies.org/r/images/2/2d/Article_8_FINAL.pdf>.
  This is used only as an order-of-magnitude floor for "back from the storm
  line". It is not a historical distance.

### 1.3 Cliffs and unstable shores

- **Lynton and Lynmouth (North Devon):**
  - Lynton stands on the cliff top. Lynmouth is the harbour village below, at
    the meeting of the East and West Lyn.
  - Since 1890 the two have been joined by a funicular. Its top station is
    500 ft above the quay, and its track is 862 ft long at 1:1.75.

  [source, search summary: Wikipedia, the railway's own site]
  <https://en.wikipedia.org/wiki/Lynton_and_Lynmouth_Cliff_Railway>.

  This pair is the best match for the owner's *"not near a cliff"*: a town on
  the cliff top, a harbour hamlet in the cleft where a river cuts the cliff,
  and a short, steep walk between them.

  The landing sits where the river breaks the cliff [inference]. Section 3.3
  uses this.
- **Dunwich (Suffolk):**
  - Storms and longshore drift moved the river's outlet about 4 km north,
    towards Walberswick, and choked the harbour.
  - Erosion then took the town, which had nearly gone by 1750.
  - Wikipedia disputes the popular "lost in one storm" story.

  [source, search summary]
  <https://en.wikipedia.org/wiki/Dunwich>.

  A town built at the edge of an eroding soft cliff loses itself. A town set
  back keeps itself and loses only its landing [inference].

### 1.4 Marsh, malaria and fresh water

- **Italy:**
  - The Maremma plain was hyperendemic for malaria until the mid-20th century.
    *Anopheles* breeds in stagnant water on coastal plains and in river basins.
  - The Pontine Marshes were drained only in the 1930s.

  [source, search summary: Italian public-health and medical-history papers]
  <https://pmc.ncbi.nlm.nih.gov/articles/PMC3395869>.
  That people withdrew from the Pontine plain to hill towns and went down by
  day rests on a 1933 newspaper [source, weak].
  <https://dynamic-rsnz.natlib.govt.nz/newspapers/RMPOST19330403.2.54>.
  W. H. S. Jones and Robert Sallares (*Malaria and Rome*, OUP 2002) is the
  scholarly lead; it was **not read here** [recalled].
- **Western Anatolia:** Brückner links *"silting up of ports, rising water
  tables ... and spreading of malaria"* to the decline of its port cities from
  the 3rd to 4th centuries AD [source, search summary of Brückner et al.].
  <https://ancientportsantiques.com/wp-content/uploads/Documents/PLACES/Turkey/Miletos&Ephesos-Brueckner2017.pdf>
- **Fresh water.** A town needs wells or a stream. On a low coast, brackish
  ground and a salt-water table make the first dry rise inland the better well
  site [inference; not sourced here].

### 1.5 Shifting shorelines and silting

- **Ephesus:**
  - The Küçük Menderes delta pushed the coast west, and the harbours were
    refounded farther west time and again.
  - The river now reaches the sea 6 km west of the city.

  [source, search summary: Stock, Brückner et al.]
  <https://www.ancientportsantiques.com/wp-content/uploads/Documents/PLACES/Turkey/Ephese-Stock2016.pdf>
- **Bruges and the Zwin:**
  - A storm opened the Zwin in 1134, a tidal channel reaching about 15 km
    inland.
  - Damme, about 7 km from Bruges, was the transshipment outport, followed by
    Sluis at the mouth.
  - The channel silted from the late 13th century and was unusable by about
    1500.

  [source, search summary: VLIZ, visit-bruges]
  <https://en.wikipedia.org/wiki/Zwin>, <https://visit-bruges.be/damme/damme>
- **Burnham Overy (Norfolk):** Burnham Overy Town, on the River Burn, and
  Burnham Overy Staithe, *"about one mile away"* on the coast. As the river
  silted by the end of the Middle Ages, *"sea-going traffic [was] forced to
  dock downstream at Overy Staithe"*, which became the port of the Burnhams
  [source, search summary: Britain Express, Bury Mercury, Wikipedia].
  <https://en.wikipedia.org/wiki/Burnham_Overy>.

  Norfolk's *staithe* means a landing place [recalled]. This is the textbook
  case of **a landing as its own small feature tied to a mother village**.

### 1.6 Paired port and inland town: distances

| Pair | Kind | Distance | Reason | Tag |
|---|---|---|---|---|
| Lynton – Lynmouth | cliff top / harbour in a river cleft | ~260 m track, 152 m (500 ft) climb | cliff | source (search summary) |
| Schinoussa Chora – port | hill chora / port | 1.2 km | piracy | source (search summary) |
| Burnham Overy Town – Staithe | village / staithe | ~1 mile (1.6 km) | silting | source (search summary) |
| Canterbury – Fordwich | city / river port on the Stour | ~2 miles (3.2 km) | navigation limit (river) | source (search summary) |
| Sóller – Port de Sóller | town / port | ~3 km | piracy (weak) | source (search summary) |
| Iraklia Chora – port | hill chora / port | 4.5 km road | piracy | source (search summary) |
| Winchelsea – the sea (1288) | relocated hill town | ~3 miles | storms | source (search summary) |
| Ravenna – Classe | city / naval and commercial port | ~4 km | lagoon, marsh | source (search summary) |
| Athens – Piraeus | city / port | ~6 km (Long Walls) | harbour quality, defence | source (search summary) |
| Exeter – Topsham | city / estuary port | ~6 km | navigation limit (river) | source (search summary) |
| Bruges – Damme | city / outport | ~7 km | silting | source (search summary) |

**Reading the table** [inference]:

- **0.2–1.6 km:** the same-community pairs, where people walked to the
  landing daily.
- **3–7 km:** a separate port town with its own population, joined to the
  city by a road, a wall or a canal.

The owner's *"not beyond walking distance"* describes the first band.

### 1.7 Walking distance: how far is "walkable"

- **Marchetti's constant** (Marchetti 1994, *Anthropological invariants in
  travel behavior*, IIASA (the PDF found is filed as RR-95-04), after Zahavi): people spend about one hour
  a day travelling. At about 5 km/h on foot that gives a **2.5 km radius**,
  and no pre-1800 city wall exceeds about 5 km across [source, search summary:
  IIASA PDF, Bloomberg CityLab, Goff].
  <https://pure.iiasa.ac.at/id/eprint/4071/1/RR-95-04.pdf>
- **Tobler's hiking function:** `v = 6 exp(-3.5 |S + 0.05|)` km/h, where `S` is
  the slope (rise over run).
  - On the flat this gives 5.04 km/h.
  - It peaks at 6 km/h on a 5 % descent.
  - It falls to about 1.8 km/h at a 30 % climb.

  Fitted by Tobler (1993) to Imhof's data [source, search summary: Wikipedia
  and an arXiv study].
  <https://en.wikipedia.org/wiki/Tobler%27s_hiking_function>

  A setback onto a hill is therefore longer in minutes than in metres:
  Lynton's climb is a short distance and a stiff walk.
- **Bracton's market distance (13th century):**
  - A reasonable day's journey is 20 miles, split into three parts: going,
    trading, returning.
  - A rival market within **6⅔ miles** was therefore a nuisance.

  [source, search summary of Bracton's text at the Ames Foundation and a
  Cambridge *Legal Studies* article]
  <https://amesfoundation.law.harvard.edu/Bracton/Unframed/English/v3/198.htm>.

  This is the outer bound for a round trip with business at the far end. It
  is not a daily walk to the shore.
- **Farm-to-field distance decay.** Chisholm (*Rural Settlement and Land Use*,
  1962) found cultivation intensity falling beyond about 1 km from the
  farmstead, with 3–4 km a practical limit [recalled; the search did not reach
  the book].

**Conclusion** [inference from the above]: a daily landing within **about
30 minutes on foot**, Tobler-weighted, is "walking distance". On the flat
that is about 2.5 km; up a steep slope it is much less. A separate port beyond
about 3 km is its own settlement.

### 1.8 When settlements sit on the water, and the scope of the rule

- **Fishing as the economy, in a sheltered place.** Staithes (North Yorkshire)
  packs its houses round a harbour *"enclosed by high cliffs"* at the mouth of
  Roxby Beck. Early in the 19th century it was the largest east-coast fishing
  port north of the Wash [source, search summary: Wikipedia, VCH Hinderwell].
  <https://en.wikipedia.org/wiki/Staithes>.

  Shelter, a landing and an economy that works the water every day keep a
  settlement on the shore despite the cliffs [inference].
- **Trade and a good harbour.** Thucydides' later Greek towns *"on the
  shores"* were walled ports. Once the sea is safe and the harbour is good,
  the town comes down to the quay [source, as 1.1]. Athens kept its city
  inland and walled a corridor to Piraeus instead [source, search summary,
  Long Walls].
- **So the rule:** the setback is a response to a **reason found in the
  terrain**. A sheltered, dry, gently sloping shore with no reason gets no
  setback. A port's quay always stays at the water [inference].
- **Rivers differ.**
  - A river is crossed, drunk from and lived along. The bank is the street.
    That is why Rulings HG, HM and HN put towns on the bank.
  - Storms, waves and cliffs act on sea coasts. The sources here give rivers
    no setback for them.
  - Raiding came up rivers too (the Viking raids are [recalled]). But no
    source here ties a river town's siting back from the bank to raiding, so
    the rule does not use it.
  - **Floodplains are the exception:**
    - **Oxford's medieval town** grew on the Summertown–Radley gravel terrace,
      *"above flood level"*. Its fords and its first bridge (1004) were at the
      terrace edge over the Cherwell [source, search summary: VCH Oxfordshire
      vol. 4, Oxford Archaeology, Oxford City Council HUCA 8].
      <https://prod.british-history.ac.uk/node/15951>.
    - **The Nile.** Its towns and villages *"were located on high ground in
      the floodplain, along the elevated river banks (levées), on rocky
      islands, or on hills of sand (geziras)"*, and some of the oldest still
      stand on mounds [source, search summary: a Cambridge chapter on Nile
      floodplain agriculture; Britannica].
      <https://www.britannica.com/place/Egypt/Rural-settlement>.
    - **Bankfull flow**, which floods the floodplain, recurs about every
      1.5 years on many rivers (Leopold, Wolman and Miller 1964; Wolman and
      Leopold, USGS PP 282-C). Gravel and sandy beds can run 2–3 years or more
      [source, search summary].
      <https://pubs.usgs.gov/pp/0282c/report.pdf>.
    - So a floodplain is flooded often. Its towns go to the terrace, or to
      raised ground within it, and keep the crossing or landing at the river.
- **Lakes: excluded.** No source found here supports a lake-shore setback.
  Small lakes have short fetch, and large lakes are a sea-like case with no
  evidence gathered. Question Q5 asks the owner.

---

## 2. The engine today, read at the symbols

### 2.1 How a coastal or river town's cell is chosen

All in `cartalith-civ/src/lib.rs` unless named [code]:

1. **`place_settlements_on_rivers`** does the following, in order:
   - `civ_snap_land`, then `civ_snap_coast`, then **`civ_snap_to_water_edge`**:
     - It nudges the site at most `SETTLE_WATER_SNAP_KM` (12 km, *"a morning's
       walk"*, reference line 20778) toward water.
     - `seaNear` widens this to 30 km.
     - It moves only if suitability stays at least the tolerance (0.80, or
       0.60 when sea-near).
     - Its `habitable` test refuses any cell whose flood exposure exceeds
       **`SETTLE_FLOOD_SAFE` = 0.55**: *"the intent is the historical
       waterfront-for-transport rule, 'the bank terrace, not the channel'"*.

     **The reference itself already encodes the floodplain point at cell
     level.**
   - Ruling EX's river constraint (`civ_settle_on_river`, behind
     `civ.river_front`).
   - **`civ_is_coastal`** gives the port badge, within
     `civ_port_radius_cells`:
     - with **`civ.port_reach`** (Ruling FK item 2) on: `PORT_REACH_KM` = 4 km;
     - otherwise: `max(6, floor(gw/60))` cells.
   - **`civ.port_shore`** (Ruling FI, `civ_settle_on_shore`): a badged town
     whose own box holds no sea (`urban_adapter::um_box_has_sea`, with sea
     above `BOX_SEA_FRAC` = 0.02 of the box's 22 m raster) moves to the
     nearest cell within `SHORE_NUDGE_KM` (4 km) whose box does. A river town
     keeps to its river where it can.
2. **Cell size** sets what the cell can express:
   - At 384 cells over 800 km a cell is 2.08 km.
   - At 2 048 cells a cell is 391 m.
   - The town box is `SITE_WM` x `SITE_HM` = **1 700 x 1 250 m**
     (`urban_adapter.rs`, `cartalith-urban/src/generate.rs`).

   **A setback of a few hundred metres is below one cell at 384 and one to two
   cells at 2 048. It belongs inside the town box, not in the cell choice**
   [inference from these figures].

### 2.2 Where the town and its marker stand inside the box

- **`urban_harbour_front`** (Ruling FK item 1; `cartalith-urban/src/generate.rs`):
  - `harbour_front_market` moves the market toward the nearest open water to
    `HARBOUR_FRONT_M` (100 m), or to `HARBOUR_FRONT_RIVER_M` (120 m) on a river
    site. These are the reference `placeAnchors`' market-to-harbour distances.
  - Its doc says it *"must never return a point farther from the water than
    `mc`"*.
  - A harbour-front layout replaces the first one only if it houses at least
    `HARBOUR_HOUSING_KEEP` (0.9) as many people.
- **The quay** (`cartalith-urban/src/water.rs`, `build_harbour`, reference
  `buildHarbour`) lays the quay, a back street, stubs, a **harbour road**,
  piers, a mole and a defence. **A road from the town to its quay already
  exists**: that is the path section 3.3 reuses.
- **Rivers, Rulings HG, HM and HN** (`cartalith-godot/src/bank_site.rs`,
  behind `civ.bank_sites`):
  - `WorldGen::town_origin` lays a river town round the drawn river's edge
    (`origin_point`). `ORIGIN_BANK_GAP_M` (30 m) is the reference's flood band
    beside a channel; `ORIGIN_MIN_CLEAR_M` is 60 m from the centreline.
  - The bank is the one the town's roads are on (`roads_bank`, a majority of
    road samples).
  - The marker follows the market (HM).
  - **Today nothing in this path asks whether the drawn edge is floodplain.**
    The cell was flood-safe in the reference's sense, but HM moves the origin
    within the cell to the drawn edge, which can be the floodplain's lowest
    ground [inference from the symbols; not measured].
- **River villages** (`cartalith-urban/src/village.rs`): `landing_market` puts a
  street village's market `LANDING_M` (32 m) from the channel, and
  `landing_lane` lays a 4 m lane down to the water. **This is a landing path
  already built, for villages.**

### 2.3 What "a harbour over 23 m from water" means

- **`cartalith_urban::generate::HARBOUR_REACH_M` = 23 m** [code]. *"One street,
  the quay, between the houses and the drawn water"*: 11 m (half a 22 m raster
  cell) plus 5 m (the quay's inset) plus 7 m (its width). It is a labelled
  judgement from the harbour's own geometry.
- **`town_reaches_water`** applies it to the town **body**: the buildings within
  `max_rf` of the market, so quay houses 650 m from the walls do not count.
- **The census** (`cartalith-godot/src/town_river_census.rs`,
  `harbour_front_audit`, `HARBOUR_STREET_GAP_M` = the same constant) counts:
  - harbours whose nearest building is over 23 m from the water;
  - harbours whose body is over 23 m from the water.
- **Its source is Ruling FK item 1** (`LARGE_ITEM_RULINGS.md`, 2026-11-16):
  *"the town layout grows toward its sea (quay, harbour district) ... so no
  harbour stands inland of its own water"*.
- **The leftovers, as `STATUS.md` records them** (not re-measured here):
  - The 2026-11-17 "Town leftovers, second pass" entry gives:
    - 483920 #5 at 384 at **83 m**. The row's earlier 191 m was stale. Its
      harbour-front try housed too few, so the reference layout was kept.
    - 483920 #10 at 2 048 at **73 m**, with *"the map's river runs between the
      city and the sea"*.
    - **154:136 "now stands at the river mouth on the shore"**.
  - So the row's "about 400 m inland" for 154:136 is stale by that entry.
  - **Neither #5 nor #10 has a terrain reason on record.** Whether a setback
    rule explains them at all is for CS-0 to measure, not for this document to
    assume.

### 2.4 Data the engine has for deciding a setback

| Reason | Field today | Where | Resolution, notes |
|---|---|---|---|
| Low, flood-prone ground (river) | `build_flood_field` (TWI + discharge + lowland, reference `buildFloodField`) | `cartalith-civ`, built in `cartalith-godot/src/lib.rs` civ pass | per cell; `SETTLE_FLOOD_SAFE` 0.55 is the reference's line |
| Floodplain | `build_landform_field` class 6: `s < 0.5`, `r_norm < 0.35`, beside a cell with flow over `flow_hi` | `cartalith-terrain/src/landform.rs` | per cell; class 1 is **cliff** (`s > 4.5`) |
| Marsh / wetland | `build_wetland_mask` (moist > 0.62, low < 0.18, slope < 1) | `cartalith-civ` | per cell, land only |
| Tidal flats, marsh accretion | `apply_tidal_sedimentation`, erosion's coastal pass `marsh_band` | `cartalith-erosion/src/passes.rs` | **alter heights only**; no persistent mask found (searched `tidal`, `marsh` in engine and erosion). The tide field exists (`cartalith_climate::tides::compute_tide_field`) but only while `tidal_flats` is on |
| Cliff at the shore | landform class 1; slope from the height field | as above | per cell |
| Exposure / fetch | `river_delta::exposure` (share of a half-disc in front of a point that is water) | **`cartalith-godot`**: `cartalith-civ` and `cartalith-urban` cannot call it (crate direction); would need moving or re-deriving | sub-cell |
| Wind | `cartalith_climate::current_wind_field`, computed in the civ pass for the planner | `cartalith-godot/src/lib.rs` | per cell (onshore component = wind · shore normal) |
| Defensibility | `civ_relative_elevation`, `terrain_ruggedness_d`, `civ_place_defensibility` | `cartalith-civ/src/military.rs` | per cell |
| In-box relief | `TerrainCtx` (heights, `cell_m`, `h_min`, `h_max`) | `cartalith-urban/src/site.rs` | interpolated from the world grid: **at 384 the box is smaller than one cell, so it may not resolve a cliff or terrace** (CS-0) |
| Raiding | none | — | no data; question Q4 |

---

## 3. Proposed rule

### 3.1 Shape

**No cell moves** [design]:

- The settlement's cell, its port badge (`um_box_has_sea`), FI's nudge, FK's
  port reach, roads, sea lanes and every civ-level pin stay as they are.
- The setback lives **inside the town layout**: where the market (the town's
  core) stands in its 1.7 x 1.25 km box, and where the quay or landing stands.
- Town layouts are generated on demand with no golden. Two `urban_*` switches
  following the `urban_harbour_front` precedent therefore move no generated
  field. The FH-R2 and G1 pins are **expected** not to move; CS-2 checks this.

A setback beyond the box (the 3–7 km outport band) needs a **paired port
hamlet** that carries the badge. That is a later milestone, CS-6, and question
Q3.

### 3.2 The coastal rule (`urban_coastal_setback`)

**Which towns.** A town whose box holds sea (`um_box_has_sea`), whether a
harbour or not. Lakes are excluded (Q5).

**Inputs.** For each candidate core point `p` on the box's 22 m raster, three
**hazard reasons**, each a yes or no:

1. **Surge or storm band:**
   - `p`'s height above sea is under `H_SURGE`, **and**
   - `p` lies within `D_SURGE` of open water that is exposed.
   - Exposure is the share of water in a half-disc of radius `R_EXP` facing
     out from the nearest shore point (`river_delta::exposure`'s definition,
     re-derived on the civ side). It must be at least `E_EXPOSED`.
2. **Cliff edge:** the shore within `D_CLIFF` of `p` is cliff (landform
   class 1, or a height rise over `H_CLIFF` within one raster cell of the
   water), and `p` is within `D_CLIFF` of the cliff top.
3. **Marsh or flood:** `p` is a wetland cell (`build_wetland_mask`), or its
   cell's flood field exceeds `SETTLE_FLOOD_SAFE`, or its height above sea is
   under the tide range where a tide field exists.

**Formula** [design]:

```
setback_core = the reference's own market point (today's harbour_front_market answer) if it has no reason;
             else the nearest raster point q to the water with no reason, such that
                  dist(q, water) - dist(market0, water) <= D_CAP_BOX
                  and walk_min(q -> landing) <= T_WALK_MAX;
             else (no such q) the reference point, reason recorded as "no safe ground in reach".
```

- `walk_min` is Tobler's function summed along the path, in minutes.
- **A port with a quay keeps its quay** (`build_harbour`) at the water whatever
  the core does. Its harbour road joins core and quay (3.3).
- **Pull to the water.** For a trading port (sea lanes call there, or
  population at least `PORT_QUARTER_POP`), the harbour quarter stays
  continuous: buildings line the harbour road from the quay to the core, with
  no field gap between them. This follows Athens' Long Walls in spirit, at
  town scale [inference].

**Constants.**

| Constant | Proposed | Source or label |
|---|---|---|
| `D_CAP_BOX` | 600 m | **judgement**: about half the box's short side (625 m), so the quay and the core both stay in the box; within the Lynton–Lynmouth and Schinoussa band (0.26–1.2 km) |
| `T_WALK_MAX` | 30 min | Marchetti's one hour a day, halved for one way [source]; binding only beyond the box (CS-6) |
| walking speed | Tobler, 5.04 km/h on the flat | [source] |
| `H_SURGE` | swept in CS-1 (start 4 m above sea on an exposed shore, 2 m sheltered) | **judgement**; to be measured against the world's real height units (`peak_m`). No historical figure is cited for it |
| `D_SURGE` | 100 m | the ICZM Article 8 setback [source], as a floor |
| `R_EXP`, `E_EXPOSED` | `river_delta`'s radius and `EXPOSURE_SHELTERED` | reuse the existing labelled judgements |
| `D_CLIFF`, `H_CLIFF` | 50 m, 15 m | **judgement**, swept in CS-1 |
| `PORT_QUARTER_POP` | `PLAZA_MARKET_POP` | **judgement**, reusing an existing threshold |

### 3.3 The landing as its own small feature

A **landing** is three things [design]:

1. **The landing point:** the shore point the core's path reaches the water at.
   Choose the point within `T_WALK_MAX` of the core that minimises
   `walk_min + λ · exposure`, preferring, in order:
   - low slope at the water;
   - a river mouth or a cleft through a cliff (Lynmouth, Staithes);
   - lower exposure (a cove).

   λ is a judgement swept in CS-1.
2. **The path** from the core to the landing point:
   - For a harbour town, the **existing harbour road** of `build_harbour`.
   - Otherwise a lane like `village::landing_lane`, generalised from villages
     to towns: dry samples, never across a river.
3. **What stands at the landing:**
   - a harbour town's **quay** (unchanged);
   - otherwise a **hard**: a strip of landing plus one to three sheds or
     boathouses (the *naust* or staithe).
   - Beyond the box, a **port hamlet** (CS-6, Q3) as a separate settlement
     tied to its mother town (`landing_of`).

The landing is drawn on the map at the zoom where the town plan is drawn, and
in the City Viewer. Inspect lists it as, for example: *"Landing: 420 m, 6 min
on foot (set back for storm exposure)"*.

### 3.4 The river floodplain rule (`urban_floodplain_setback`)

**The bank rule stays the default.** Rulings HG, HM and HN are unchanged
wherever the origin is not floodplain.

**When it applies.** Only where HM's origin point (the drawn edge plus
`ORIGIN_BANK_GAP_M`) is floodplain:

- its cell is landform class 6, or the flood field there exceeds
  `SETTLE_FLOOD_SAFE`; **and**
- the in-box heights show the origin under `H_FLOOD` above the channel.

CS-0 measures whether `TerrainCtx` resolves this at 384 and at 2 048. If it
does not at 384, the rule acts at 2 048 only and says so.

**What moves.** The market moves straight away from the river, along the
perpendicular to the nearest drawn centreline point, **on the same bank HM
chose** (`roads_bank`, the market's bank). It stops at the first raster point
that is:

- above `H_FLOOD`, and
- not wetland,

within `D_CAP_RIVER`. That point is the terrace edge.

- **No flood-free ground within the cap:** the market stays at HM's origin,
  and the reason is recorded as "no terrace in reach".
- The Nile's answer, raised ground inside the floodplain, is the same rule: a
  levee rise counts as flood-free ground wherever it is [source, 1.8].

**What stays at the river:**

- the **bridge** (`bridge_pt`, unchanged);
- a **landing lane** from the market to the drawn edge (`landing_lane`
  generalised).

The river still runs through the town box, so Ruling HM's *"keep the town's
river"* holds. `D_CAP_RIVER` must keep the drawn edge inside the box.

**The marker** follows the market (Ruling HM's rule, unchanged in kind). It
moves to the terrace on the same bank. HN's "marker across from its market"
count must not rise.

**Constants.**

| Constant | Proposed | Source or label |
|---|---|---|
| `D_CAP_RIVER` | 500 m | **judgement**: inside the box's 625 m half-height, less HM's 60 m clearance and a street |
| `H_FLOOD` | swept in CS-1 (start: the bankfull depth the town's channel width implies, at least 2 m) | **judgement**; bankfull recurs about every 1.5 years [source] |

### 3.5 How it meets the town plan, the marker and ports

| Part | Today | With the rule |
|---|---|---|
| Cell, badge, ports, sea lanes, roads | FI, FK and EX as built | unchanged (no cell moves) |
| Market (core) | reference point; FK's `harbour_front_market` pulls toward water (100 / 120 m); HM's drawn edge for rivers | on the nearest hazard-free ground, capped; no reason means today's answer exactly |
| Quay | `build_harbour` at the shore | unchanged: at the water |
| Path | harbour road (harbours), `landing_lane` (street villages) | harbour road, or a landing lane for any set-back town |
| Marker | HM: follows the market (rivers); otherwise the cell | follows the market for coast too (Q6) |
| Walls | wall hug, `snap_wall_to_shore` | the wall rings the core; the quay may stand outside the wall [inference] |
| Frozen towns | frozen | stay frozen; `URBAN_RULES_VERSION` bump and fixture re-pins (as HM) |
| Old saves | switch absent means off | offered once, as Ruling HN does (Q8) |

### 3.6 What the Ruling FK bar becomes

The bar was "every harbour's body within `HARBOUR_REACH_M` (23 m) of its
water". It becomes [design; the new wording needs Q1]:

- **B-FK1:** every harbour's **quay or landing** has a building or street
  within 23 m of open water. **0 over.**
- **B-FK2:** every harbour's **body** is within `setback + 23 m` of its water,
  where `setback` is the distance the rule recorded with a reason. **0 over.**
- **B-FK3:** towns with a body gap over 23 m and **no recorded reason**: **0**.
  This is the old bar, which still applies to every town without a reason.

### 3.7 Measurable bars

**Census** (`town_river_census.rs`):

- seeds 483920, 912345 and 246371;
- at 384 and at 2 048;
- before and after the switch;
- the medians run alone (MISTAKES: timings).

| Bar | Target |
|---|---|
| B-FK1, B-FK2, B-FK3 | 0, 0, 0 |
| Ports and sea lanes | equal before and after (no cell moves) |
| Housed p10; towns under 80 % housed | p10 not lower by more than 1 point; count not higher |
| Buildings in water; buildings over the drawn river or sea (HN) | 0; not higher |
| Core buildings in a hazard band the town was set back from | 0, except the quay row |
| Setback p90 | at most `D_CAP_BOX` (coast) and `D_CAP_RIVER` (river) |
| Walk from core to landing, maximum | at most `T_WALK_MAX` |
| Floodplain towns with a terrace in reach left on the floodplain | 0 |
| HN's markers across from their market | not above 4 |
| `fh_r2_baseline`, `g1_baseline` (11 ignored tests) | 0 pins moved |
| Share of coastal towns set back | reported per reason, with no target. A world where every town is set back, or none is, is a finding to bring to the owner |

**Frames.** Windowed shell, the app's world, the same three seeds, before and
after, opened and looked at. The zoom levels' meaning is the map's own; CS-0
confirms what is drawn at each one.

| Zoom | Bar |
|---|---|
| x1 | 0 port badges lost; no marker drawn in sea or in a drawn river; marker offset from today's within the recorded setback (sub-pixel to 1 px at 384) |
| x4 | every set-back marker on land, on the hazard-free side, at least 1 px inland of the drawn coast; none on drawn marsh |
| x16 | the path from core to landing is drawn continuously, and its water end is 0 px from the drawn water edge. If the town layer is not drawn at x16, the same bar in the City Viewer, fitted and at x0.25, as Ruling FK's frames were |

### 3.8 The section 7p switches

- **`urban_coastal_setback`** and **`urban_floodplain_setback`**:
  - **`false`** in `cartalith_engine::WorldParams::defaults()`;
  - **`true`** in `cartalith_godot::params::defaults()` once ruled;
  - **off on load** when the key is absent (`params::LOADS_WHEN_ABSENT`).
- Each needs `PARAMS` and `JS_PATHS` rows, a `GENERATION_PARAMETERS.md` row and
  a `town_freeze::switch_record` field.
- They are two switches because these are two concerns, as with FK's pair.
- Both are read fresh at layout time and never stored, like
  `urban_harbour_front`.
- No golden reads either.

---

## 4. Milestones

| Id | What | Done when |
|---|---|---|
| **CS-0** | **Measure first.** On the three seeds at 384 and 2 048, for every harbour town, every coastal town and every HM river origin, record: <br>• which reasons the shore point or origin has (surge band with exposure, cliff, marsh or flood, floodplain); <br>• whether `TerrainCtx` relief resolves them in the box at each size; <br>• the distance to the nearest reason-free point; <br>• the FK leftovers re-measured (#5, #10, 154:136), and whether any reason explains them; <br>• the baseline of every bar in 3.7. <br>Data only; no product code. | A table per size. **Go/no-go:** if under 5 % of coastal towns at 2 048 have any reason, or relief does not resolve at either size, bring it to the owner before CS-1 |
| CS-1 | The hazard test and the nearest safe point, as pure functions with unit tests. Each constant is a literal, mutation-tested. | Tests red under each mutant; the constants swept and recorded |
| CS-2 | The coastal core setback in the layout (`urban_coastal_setback`), with the quay unchanged and the harbour road as the path. | B-FK1–3, ports, housing and baseline bars at both sizes |
| CS-3 | The landing for set-back towns without a harbour: point, lane and hard; drawn on the map and in the City Viewer; shown in Inspect. | x16 and City Viewer frames; walk bar |
| CS-4 | The river floodplain setback (`urban_floodplain_setback`) on the market's bank, with the bridge and landing lane at the edge and the marker following the market. | Floodplain bar, HN bars and HM's river-kept bar |
| CS-5 | The census and audit restated (B-FK1–3, reasons per town), and the FK row closed or re-routed with numbers. | `town_river_census` prints the new bars |
| CS-6 | *(Only if Q3 says yes)* A port hamlet beyond the box, `landing_of` its mother town, carrying the badge within `T_WALK_MAX`. Moves civ output: section 7p and a baseline re-record. | Ports not reduced; baseline attribution |
| CS-7 | Old-world offer (as HN), `URBAN_RULES_VERSION` bump, fixture re-pins, frozen towns reporting the rules change. | Probe across a reopen |

Every build brief carries the `MISTAKES.md` rules:

- never encode "no value" as a plausible value: a town with no reason has
  **no** setback record, not `0 m`;
- never assert a constant against itself;
- `cargo test --workspace` and the shell parse-check before calling anything
  green;
- frames are opened and looked at.

It also carries Ruling BK's annotation rule.

---

## 5. Owner questions, with defaults

| # | Question | Default |
|---|---|---|
| Q1 | Amend Ruling FK item 1 and `harbour_front_market`'s "never farther from the water", so that a harbour's **quay** stays at the water and its **town** may stand back up to the cap where the terrain gives a reason? Needs a `DECISIONS.md` entry. | **Yes** |
| Q2 | In-box caps: 600 m on the coast, 500 m on rivers? | **Yes** (both judgements, swept in CS-1) |
| Q3 | Beyond the box (1–5 km, chora and port): build a paired port hamlet (CS-6)? | **Later**; off until CS-2 to CS-4 are judged in frames |
| Q4 | Raiding as a reason? The engine has no data for it; it would be a world setting ("dangerous seas") that widens the setback and prefers sites hidden from the sea. | **Off**, not built |
| Q5 | Lakes: include large lakes? | **Excluded** (no evidence gathered) |
| Q6 | On the coast, should the map marker follow the set-back market, as HM's does for rivers? | **Yes**, so the map and the City Viewer agree |
| Q7 | Trading ports: keep the harbour quarter continuous from the quay to the core? | **Yes**, for ports with sea lanes or at least `PLAZA_MARKET_POP` |
| Q8 | Old worlds: offer the update once, as Ruling HN does? | **Yes**; frozen towns stay frozen |
| Q9 | Walking limit 30 minutes (Tobler-weighted), used only beyond the box? | **Yes** |
| Q10 | Two switches (`urban_coastal_setback`, `urban_floodplain_setback`) or one? | **Two**, two concerns |

---

## 6. Sources

Each source is tagged in the text where it is used. Most were read as search
summaries and are tagged so. The main pages:

- Thucydides 1.7 (Crawley): <https://cts.perseids.org/read/greekLit/tlg0003/tlg001/1st1K-eng1/1.7.1>
- Small Cyclades guide: <https://www.naxos.gr/wp-content/uploads/2018/04/SMALL-CYCLADES-GUIDE-EN.pdf>
- Greek choras: <https://greekreporter.com/2026/06/04/spectacular-greek-choras-need-see/>; <https://theculturetrip.com/europe/greece/articles/how-pirates-shaped-the-design-of-this-greek-island>
- Sóller: <https://en.wikipedia.org/wiki/S%C3%B3ller>
- Santa Caterina dello Ionio: <https://summerinitaly.com/guide/santa-caterina-dello-ionio>
- Norwegian boathouses: <https://arkeologi.blogspot.com/2005/12/boathouses-in-post-medieval-norway.html>
- Winchelsea: <https://www.winchelsea.com/the-town-story/history/new-winchelsea>; <https://en.wikipedia.org/wiki/Winchelsea>
- Tsunami stones: <https://www.smithsonianmag.com/smart-news/century-old-warnings-against-tsunamis-dot-japans-coastline-180956448/>; <https://en.wikipedia.org/wiki/Tsunami_stone>
- ICZM Protocol Article 8: <https://www.marinespecies.org/r/images/2/2d/Article_8_FINAL.pdf>
- Lynton and Lynmouth: <https://en.wikipedia.org/wiki/Lynton_and_Lynmouth_Cliff_Railway>; <https://www.cliffrailwaylynton.co.uk/about/history/>
- Dunwich: <https://en.wikipedia.org/wiki/Dunwich>
- Malaria in Italy: <https://pmc.ncbi.nlm.nih.gov/articles/PMC3395869>; <https://dynamic-rsnz.natlib.govt.nz/newspapers/RMPOST19330403.2.54>; <https://academic.oup.com/book/9850>
- Ephesus and Miletus: <https://ancientportsantiques.com/wp-content/uploads/Documents/PLACES/Turkey/Miletos&Ephesos-Brueckner2017.pdf>; <https://www.ancientportsantiques.com/wp-content/uploads/Documents/PLACES/Turkey/Ephese-Stock2016.pdf>
- Zwin, Damme: <https://en.wikipedia.org/wiki/Zwin>; <https://visit-bruges.be/damme/damme>
- Burnham Overy: <https://en.wikipedia.org/wiki/Burnham_Overy>; <https://www.britainexpress.com/counties/norfolk/az/burnham-overy.htm>
- Fordwich, Topsham: <https://www.britainexpress.com/photo.htm?photo=1129>; <https://en.wikipedia.org/wiki/Topsham,_Devon>
- Long Walls: <https://www.livius.org/articles/place/athens/athens-photos/athens-long-walls>; <https://en.wikipedia.org/wiki/Long_Walls>
- Classe: <https://en.wikipedia.org/wiki/Classis_(port)>
- Marchetti: <https://pure.iiasa.ac.at/id/eprint/4071/1/RR-95-04.pdf>
- Tobler: <https://en.wikipedia.org/wiki/Tobler%27s_hiking_function>
- Bracton: <https://amesfoundation.law.harvard.edu/Bracton/Unframed/English/v3/198.htm>
- Staithes: <https://en.wikipedia.org/wiki/Staithes>; <https://www.british-history.ac.uk/vch/yorks/north/vol2/pp365-371>
- Oxford: <https://prod.british-history.ac.uk/node/15951>; <https://www.oxford.gov.uk/downloads/file/951/huca-8--thames-crossing-and-floodplain--thames-waterfront>
- Nile settlement: <https://www.britannica.com/place/Egypt/Rural-settlement>
- Wolman and Leopold, floodplains: <https://pubs.usgs.gov/pp/0282c/report.pdf>

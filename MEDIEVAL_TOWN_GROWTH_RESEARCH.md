# Medieval town growth: research for the Cartalith town generator

Researched 2026-11-15 by a research agent, for a judge deciding how the town generator should change. Not a scope and not a status record.

**Reading this report**

- **[source]** means a cited URL states it. I read most sources through search-result summaries or page extracts, so a claim marked [source] is as reliable as that extract. Where a source disagreed with another I say so.
- **[inference]** means my own reasoning or arithmetic.
- Sources are mixed. Wikipedia and tourism pages are used for dates and layout, and weaker for numbers. Every weak source is flagged.
- **No plan images.** I did not retrieve or verify any plan or map image URL, so I give none. Good candidates to look up by name are Braun and Hogenberg's *Civitates Orbis Terrarum* (c. 1572-1617) and Merian's town views for most of the European examples below.
- **Several numbers you asked for were not found.** Section 4 lists exactly which, so nothing is silently invented.

---

## 1. Urban morphology frameworks

### 1.1 Conzen's town-plan analysis

- Conzen split the townscape into three form complexes: the town plan (streets, plots, building blocks), the building fabric, and land and building use. [source: [Journal of Urban Morphology search set](https://journal.urbanform.org/index.php/jum/article/download/3896/3218/13149)]
- A **plan unit** is an area whose mix of streets, plots and building blocks differs in character from its neighbours. Conzen mapped a hierarchy of units from the town plan alone. [source: same set; see also [Conzen, "From Alnwick to Cincinnati"](https://journal.urbanform.org/index.php/jum/article/view/4353)]
- The second tier of the hierarchy separates the **core**, the **fringe belts** and the **residential accretions**. [source: same set]
- A **fringe belt** is an area where plots vary widely in shape and size. It forms at a town edge that stood still for a while, and is typically taken over by institutions, large open land and allotments. [source for plot variability: same set; the "stood still" mechanism and the institutional land use are standard Conzen doctrine that I did not re-verify in this pass, so treat them as **[inference]**]
- The **burgage cycle** describes what happens to plots after they are laid out. The backland fills with buildings until the plot is fully built, and the last phase can be "urban fallow". [source: [Alnwick-to-Newcastle study set](https://journals.socantscot.org/index.php/psas/article/download/9756/9723/9707)]
  - A Newcastle study reports East Pilgrim Street plots that have passed through the whole cycle to urban fallow.
  - A Scottish study reports that by the early 16th century Edinburgh backlands often carried three or four substantial buildings. [source: [Framework and form, early urban Scotland](https://cambridge.org/core/services/aop-cambridge-core/content/view/4FC18665945BC7A9144C4C9165838A5A/S0963926814000789a.pdf/framework_and_form_burgage_plots_street_lines_and_domestic_architecture_in_early_urban_scotland.pdf)]
- At Alnwick, most properties were still about 28 feet wide, or an obvious fraction or multiple of 28 feet, centuries after the plots were laid out. [source: [Alnwick Civic Society, Rods, poles and perches](https://alnwickcivicsociety.org.uk/2020/09/11/rods-poles-and-perches/)] So **plot module survives in the plan**. [inference]
- Critique: Haslam (2018) argued that some Conzenian plan units are too internally disparate to count as units. Defenders say Haslam misreads the method's purpose. Conzen's later Ludlow work added small "morphotopes" beneath the plan-unit frame. [source: [Journal of Urban Morphology](https://journal.urbanform.org/index.php/jum/article/download/4081/3390/11721)]

**For a generator:** the useful idea is that a town has **plan units laid down in different phases**, each with its own plot module and street grain, and each keeps its module even after the buildings are replaced. [inference]

### 1.2 Caniggia and Muratori (typo-morphology)

I did not retrieve a source for this school in this pass.

- From general knowledge: it reads a town as a sequence of building types that grow out of one another, with the street network following from the types. **[inference, unverified]**
- It is relevant to a generator in one way only: buildings should vary by phase, and lots and streets should not be generated independently of each other. This is a design inference, not a finding.

### 1.3 Hillier's space syntax applied to old towns

- Craane combined town-plan analysis with space syntax on medieval Utrecht. She found a strong correlation between medieval economic spaces and the **most integrated spaces**, similar to modern cities. She concluded that space syntax can be applied to medieval contexts. [source: [Internet Archaeology 21, Craane](https://www.intarch.ac.uk/journal/issue21/5/index.html)]
- In many English medieval towns the cathedral precincts were the most integrated spaces, locally and globally. Utrecht's cathedral area was more segregated. [source: search summary of the same Internet Archaeology paper]
- A study of small Provençal towns found that their growth involved the **rationalisation of an integration core**, with integrated streets becoming better aligned and wider over time. [source: [Journal of Space Syntax](https://journal.urbanform.org/index.php/jum/article/download/4090/3399/11730), as surfaced by search]
- Axial analysis is criticised because it is linear and does not express metric properties. [source: same search]

**For a generator:** the market and main street should sit on the **most-integrated, best-connected path**, not just at the geometric centre. This is testable by computing integration or betweenness on the generated street graph. [inference]

### 1.4 Quantitative street-network work

- Boeing (2019, "Urban Spatial Order") measured the entropy of street bearings for 100 cities, together with segment length, circuity, average node degree, and the share of four-way intersections and dead ends. [source: [Boeing](https://geoffboeing.com/publications/urban-spatial-order-entropy/)]
- High-entropy cities such as Seoul, London and São Paulo are attributed to successive developments, difficult terrain with hills and rivers, and the annexation of separate towns that had their own grids. [source: same page]
- US and Canadian cities are far more grid-like than the rest. [source: same page]
- A secondary R vignette following Boeing's method reports Rome's organic core as the highest-entropy case of its sample and Olinda's colonial pattern as irregular and high-entropy. [source: [osmnxr vignette](https://cran.nics.utk.edu/cran/web/packages/osmnxr/vignettes/street-orientation.html)]
- I did **not** find Barthelemy's *Morphogenesis of Spatial Networks* or any medieval-specific street-bearing study.

**Takeaway:** a generated town can be scored with bearing entropy, circuity and the share of 3-way against 4-way junctions. Real organic towns are **T-junction dominated with high bearing entropy**. [inference from the Boeing metrics]

---

## 2. How medieval towns grew

### 2.1 Organic versus planned

- Beresford divides medieval towns into planned and organic. More recent scholarship disputes the clean split, and points to regular burgage plots as evidence of planning even in organic towns. [source: [Herefordshire Through Time, "The origin of towns"](https://htt.herefordshire.gov.uk/herefordshires-past/the-medieval-period/towns/the-origin-of-towns)]
- **Zähringer towns.**
  - Freiburg is treated as a planned foundation with a cross of main streets and equal house plots. [source: search summary of Heidelberg Propylaeum volumes, e.g. [here](https://books.ub.uni-heidelberg.de/propylaeum/catalog/view/585/933/86938)]
  - The "Zähringer cross" as a uniform type is disputed. A later study denies it applied across all Zähringer towns. [source: same]
  - There is a whole paper titled "Zähringerstädte: ein städtebaulicher Mythos". [source: [schlossthun.ch PDF](https://schlossthun.ch/wp-content/uploads/2024/02/Zahringerstadte_Ein_stadtebaulicher_Myt.pdf), title only]
  - Freiburg's plots were laid out on a regular module, but it adapted to the winding earlier streets. [source: Untermann, via the Loebbecke summary in 4.2]
- **Bastides.** Monpazier (1284) is a rectangle of about 400 m by 220 m, **8.8 ha**, with parallel long streets crossed by four transverse streets. Houses were originally the same size and separated by narrow fire alleys ("androns"). [source: [Cornell Bastides collection](https://exhibits.library.cornell.edu/bastides-collection/feature/monpazier) and [French summary](https://www.journee-mondiale.com/cette-bastide-de-1284-conserve-son-plan-de-8-8-hectares-avec-32-maisons-9328.htm); both weak popular sources]
  - In bastides the market square often sets the module used to subdivide the town. [source: [Wikiwand "Bastide"](https://wikiwand.com/en/articles/Bastide)]
- **Gründungsstädte in the east.** Kraków was laid out on 5 June 1257 under Magdeburg law. [source: [Ancient History Sites](https://www.ancient-history-sites.com/?p=169824)]
  - The square is about 200 m by 200 m (roughly 4 ha) and occupies four blocks of the grid.
  - The 1257 chartered city covered about 0.32 km², so the square is **about 12 per cent of the founded area**. [source: [Heidelberg Ostrowski](https://digi.ub.uni-heidelberg.de/diglit/ostrowski1992/0021/text_ocr) for the 0.32 km²; the 12 per cent is my division, **[inference]**]
  - Blocks were divided into evenly sized plots. The southern part of the square follows an earlier settlement, so the grid was adapted to older traces. [source: same]

### 2.2 Market-place origins

- **Prague.** A market centre on the right bank opposite Hradčany developed into the Old Town, especially after the Judith Bridge (c. 1170). A market magistrate is first recorded in 1212, and the Old Town walls date from the 1230s. [source: [Britannica](https://www.britannica.com/place/Prague/History), [praha.eu](https://praha.eu/web/eng/prague-historical)]
- **Paris.** The Grève beach became the port and commercial centre. Louis VI bought the Champeaux land in 1137 for a grain market, which became Les Halles. [source: [Wikipedia, Medieval Paris](https://en.wikipedia.org/wiki/Medieval_Paris)]
- **Southwark.** The High Street market dates from the 13th century. [source: [Wikipedia, Southwark](https://en.wikipedia.org/wiki/Southwark)]
- Market streets often taper at their ends, so standard widths are hard to find. [source: [Victoria County History panel](https://ihr-archive.github.io/vch-explore/sites/explore/files/explore_assets/2010/03/22/panel2whole.pdf)]

**Pattern:** a market begins at a **landing, a bridgehead or a road junction**, and the planned square, where one exists, formalises that. A spindle-shaped market street is as common as a square. [inference from the sources above]

### 2.3 Growth along roads, gates and suburbs

- At Southwark, two Roman roads (Stane Street and Watling Street) met at about the line of Borough High Street, at the bridgehead. [source: [Southwark Council archaeology](https://www.southwark.gov.uk/planning-environment-and-building-control/planning/design-and-conservation/archaeology-and/roman)]
- Medieval Southwark extended south of the bridgehead along both sides of Borough High Street. [source: [Historic England entry 1422618](https://historicengland.org.uk/listing/the-list/list-entry/1422618)]
- "Borough" was glossed in one source as "suburb of a city outside the wall". [source: search summary of [Southwark: Introduction, British History Online](https://www.british-history.ac.uk/node/36126)]
- Frankfurt's Neustadt grew north of the old town and was enclosed by an additional wall from 1333. [source: [Wikipedia, Fortifications of Frankfurt](https://wikipedia.com/wiki/Fortification_of_Frankfurt)]
- Many medieval cities had to extend their walls at least once during the High Middle Ages population surge. [source: search summary, [de re militari](https://deremilitari.org/?p=1753)]

### 2.4 Burgage plots

Dimensions are collected in section 4.2. In short, plots are long and narrow with the short side on the street, and they are subdivided as population grows. Amalgamation was uncommon until the second half of the 19th century. [source: [Wikipedia, Burgage](https://en.wikipedia.org/wiki/Burgage)]

### 2.5 Successive wall rings

- **Paris** shows the clearest case of ring succession, with sources:
  - The first wall (11th century, right bank only) was about 1,700 m long, with about 30 towers.
  - By 1180 the city covered about 200 ha.
  - Philippe Auguste's wall (right bank 1190 to 1208, left bank 1209 to 1220) was 5,400 m long (2,800 m right bank, 2,600 m left), with 10 gates and 75 towers, enclosing about 273 ha.
  - Charles V's wall (1358 to 1371) was 4,900 m long, enclosed 439 ha, and was mostly on the right bank. The left-bank wall was only repaired.
  - [source: [Wikipedia, Medieval Paris](https://en.wikipedia.org/wiki/Medieval_Paris). A second extract gives the Philippe Auguste enclosure as 253 ha, and right/left dates of 1190 to 1209 and 1200 to 1215, via [City walls of Paris](https://en.wikipedia.org/wiki/City_walls_of_Paris). So the area is uncertain at 253 to 273 ha.]
  - The right bank was the priority, because it was the best landing place. The left bank got drier, simpler defences. [source: same City walls page]
- **Florence.** The Oltrarno was first walled in 1173 to 1175, mostly palisades and gates. The third circle (1284 to 1333) enclosed an area the city did not exceed until the mid-19th century. Gates came in stages (Porta Romana 1328 to 1331, Porta San Frediano 1333). [source: [Britannica](https://www.britannica.com/place/Italy/Florence-in-the-14th-century), [Studentsville](https://www.studentsville.it/oltrarno-walls/)]
- **What happened to the old ring.** I found no direct source on this in the search results.
  - In Paris the old wall stayed as a boundary that Charles V's wall outflanked. [inference from the wall dates above]
  - Conzen's fringe belts form where the old edge stalled (section 1.1).
- **Prague.** The Old Town walls date to the 1230s. Charles IV extended Malá Strana and built the Hunger Wall in the second half of the 14th century. [source: [Wikipedia, Malá Strana](https://en.wikipedia.org/wiki/Malá_Strana)]

### 2.6 Infill

Backland infill is Conzen's burgage cycle (section 1.1). In walled towns the ring also fills with infill because new walls are costly. Cesaretti et al. note that settled area differs from walled area: Bristol's walled area was 55 ha, but the settled area they used was 130 ha, counting extramural suburbs and intramural vacant space. [source: [PLOS ONE, Cesaretti et al. 2016](https://pmc.ncbi.nlm.nih.gov/articles/PMC5051806/)]

---

## 3. River towns

### 3.1 When and why towns spread to both banks

No single scholarly synthesis was retrieved. The pattern from the case sources below:

1. A town starts on one bank at a crossing, a confluence or an island. [sources in 3.3]
2. A bridge is built (Prague c. 1170, Basel c. 1225, London Bridge in the 12th century on a Roman site). [sources in 3.3]
3. A suburb or a planned town grows at the far end, often founded by a ruler or a bishop and often with a **separate market and street grid** (Malá Strana 1257, Kleinbasel).
4. A later wall ring encloses both, sometimes with the less valuable bank defended more lightly (Paris, Florence).

### 3.2 Bridges as growth drivers

- **Prague:** the Judith Bridge (c. 1170) drove the Old Town's growth. It was destroyed by flood in 1342 and replaced by the Charles Bridge from 1357. [source: [Britannica](https://www.britannica.com/place/Prague/History), [expats.cz](https://www.expats.cz/czech-news/article/on-this-day-the-medieval-judith-bridge-collapsed-in-1342)]
- **Florence:** three main roads radiate from the Ponte Vecchio bridgehead, and many poor immigrants settled there. [source: [Studentsville](https://www.studentsville.it/oltrarno-walls/), a weak source]
- **Basel:** the Rhine bridge, c. 1225, tied Grossbasel and Kleinbasel together. Walls were reinforced and extended in both parts. [source: [Wikipedia, Walls of Basel](https://www.wikipedia.com/wiki/Aeschentor)]
- **Zürich:** the town sat on both sides of the Limmat. Bridges include the Münsterbrücke and Rathausbrücke, but I found no medieval dates. [source: [Wikipedia, Altstadt (Zürich)](https://www.wikipedia.org/wiki/Selnau)]

### 3.3 Bridgehead suburbs, case by case

| Town | Facts | Source |
|---|---|---|
| **Prague** | Malá Strana (founded 1257 by Ottokar II) merged several settlements below the castle into a royal town. The original residents were expelled and mainly German craftsmen invited. Its market is Malostranské náměstí, which is split into upper and lower parts with St Nicholas in the middle. Renamed Lesser Town in 1348. | [Wikipedia](https://en.wikipedia.org/wiki/Malá_Strana) |
| **Florence** | Oltrarno left bank: palisade and gate wall 1173-75, stone circuit 1284-1333. Carraia bridge in 1128. Floods in 1333 slowed the works. | [Studentsville](https://www.studentsville.it/oltrarno-walls/), [Britannica](https://www.britannica.com/place/Italy/Florence-in-the-14th-century) |
| **Paris** | Île de la Cité is the easiest point to bridge the Seine. The Grand Pont and Petit Pont were market sites. The right bank port at the Grève, Les Halles from 1137. Left bank university quarter, with the University recognised in 1215. | [Wikipedia](https://en.wikipedia.org/wiki/Medieval_Paris) |
| **London / Southwark** | A Roman bridgehead on two gravel islands. Medieval London Bridge built in the 12th century. Market on Borough High Street from the 13th century. Southwark lay outside City control until 1550. | [Southwark Council](https://www.southwark.gov.uk/planning-environment-and-building-control/planning/design-and-conservation/archaeology-and/roman), [Historic England](https://historicengland.org.uk/listing/the-list/list-entry/1422618), [Wikipedia](https://en.wikipedia.org/wiki/Southwark) |
| **Cologne / Deutz** | Constantine's Rhine bridge, about 310 AD, guarded by Castellum Divitia (Deutz). The medieval city stayed on the left bank. Deutz joined in 1888. | [Livius](https://www.livius.org/articles/place/colonia-claudia-ara-agrippinensium-koln/deutz-divitia/), [Wikipedia, Deutz](https://en.wikipedia.org/wiki/Deutz,_Cologne) |
| **Lyon** | Medieval town concentrated on the left bank of the Saône (Saint-Jean to Vaise). Presqu'île held Saint-Nizier. Main streets linked the Saône bridge near Saint-Nizier to the Rhône bridge at La Guillotière. Presqu'île urbanised mainly in the Renaissance. I found no bridge dates. | [French heritage dossier](https://patrimoine.auvergnerhonealpes.fr/dossier/pdf/95be5836-737a-4b28-8f00-80079279a172/ville-de-lyon.pdf), [visiterlyon](https://en.visiterlyon.com/discover/not-to-be-missed-in-lyon/the-history-of-lyon) (weak) |
| **Bruges** | Charter 1128, new walls and canals. The historic core lies inside an oval canal ring (the Reie). The Zwin inlet (1134 storm) gave sea access to Damme. Silting by about 1500 to 1520. I found no source for the Markt/Burg positions or wall dates. | [kupi](https://kupi.com/en/explore/belgium/bruges/history), [Time Machine](https://timemachine.eu/ltm-projects/lost-ports-of-the-zwin) |
| **Ghent** | Confluence of Scheldt and Leie. Abbey of St Peter (later St Bavo) and a portus. Gravensteen built c. 1180. Graslei and Korenlei quays face each other on the Leie, and sources disagree on which bank is which. Population about 64,000 around 1300. | [Wikipedia, Ghent](https://en.wikipedia.org/wiki/Ghent), [Wikipedia, History of Cologne](https://en.wikipedia.org/wiki/History_of_Cologne) for the population figure |

### 3.4 Quays, waterfronts, backlots and markets

- At King's Lynn a typical waterfront property combined residence, shop, warehouse and private quay on a narrow plot. [source: search summary of [Nottingham eprint](https://eprints.nottingham.ac.uk/11087/1/293169.pdf)]
- At Gloucester a lane gave access to the quay and the cloth-working quarter. The guildhall was both administrative centre and chief market hall (recorded from 1192). [source: [British History Online, Gloucester](https://prod.british-history.ac.uk/node/35363)]
- Paris's early port moved when the Grève became too small. Philip II funded a new port, the Port de l'École. [source: [Wikipedia](https://en.wikipedia.org/wiki/Medieval_Paris)]
- Plot frontage, then, runs **from street to water**: the narrow end is on the quay and the long side is perpendicular to it. [inference from Lynn plus the general burgage form]
- I found no source that states a market-to-river distance. Observed placements: Paris Grève and Les Halles, Prague Old Town square set back from the landing, Malá Strana square set under the castle slope, Gloucester guildhall and market near the quay. [inference]

### 3.5 How walls handled rivers

- **York:**
  - The walled city is divided naturally in three by the Ouse and Foss.
  - Micklegate (west of the Ouse) was linked to the rest only by Ouse Bridge.
  - A chain across the Ouse (first mentioned 1380) ran from a tower to Davy Tower on the far bank.
  - **No wall** was built along the east marshland and the King's Fishpool near the Foss.
  - [source: [British History Online, York](https://www.british-history.ac.uk/node/105365), [Wikishire](https://wikishire.co.uk/wiki/York_City_Walls), summary]
- **Norwich:** two boom towers with a chain controlled river traffic. [source: [VisitNorwich](https://www.visitnorwich.co.uk/?p=84584)]
- **Chester:** a water tower stood in the Dee (1322 to 1325), linked to the walls by a spur wall. [source: [Wikipedia, Chester city walls](https://www.wikipedia.org/wiki/Chester_city_walls)]
- **Cologne:** the semicircular wall of about 1180 is closed by the Rhine, which forms the flat side. [source: [Britannica](https://www.britannica.com/place/Cologne-Germany)] So **the river itself is the wall on one side**. No wall on the waterfront means the quay is the defence line and the harbour gates are the openings. [inference]
- **Nuremberg:** walls across the river joined two parts of the city in the 14th century. [source: [Wikipedia, Nuremberg walls](https://www.Wikipedia.org/wiki/City_walls_of_Nuremberg)]
- **Basel:** the Rhine gate at the bridge on the western bank. Both parts were walled. [source: [Wikipedia, Walls of Basel](https://www.wikipedia.com/wiki/Aeschentor)]
- **Strasbourg:** the Ponts Couverts (1230 to 1250) are a bridge and tower complex defending the Ill. [source: [Ancient History Sites](https://www.ancient-history-sites.com/?p=206046)]

**Summary:** the observed solutions are (a) a river face with no wall and gated quays (Cologne), (b) a wall that ends at the bank with a chain or boom tower on each side (York, Norwich), (c) a wall that crosses the river with a water gate (Nuremberg), and (d) a bridge fortified at one or both ends (Basel, Strasbourg, Prague). [inference from the cited cases]

---

## 4. Typical dimensions

### 4.1 Street widths

I did **not** find a reliable table of medieval street widths. What I did find:

- Market streets taper, so standard widths are hard to establish. [source: [VCH panel](https://ihr-archive.github.io/vch-explore/sites/explore/files/explore_assets/2010/03/22/panel2whole.pdf)]
- Roman principal roads averaged about 6.5 m (22 pedes), from an average of nearly 500 roads. Not medieval, but it shows where trunk roads started. [source: search summary of [designingbuildings](https://www.designingbuildings.co.uk/wiki/The_history_of_the_dimensions_and_design_of_roads%2C_streets_and_carriageways)]
- A Florentine reconstruction gives shops 8 braccia wide and a roadway about twice that, with the 1325 statute requiring new buildings to align with the street edge. That is a Scribd paper of unclear provenance. [source: [Scribd](https://www.scribd.com/document/495706078/medieval)] At about 0.58 m per braccia this suggests about 9 m. **[inference, weak]**
- Fatimid Cairo: a cul-de-sac is at least 7 cubits wide, treated by most jurists as private to the surrounding properties. [source: [IRB net paper](https://www.irbnet.de/daten/iconda/CIB_DC28189.pdf)]. At 0.5 m per cubit that is about 3.5 m. **[inference]**
- In Fez and Cairo, access is controlled in layers (bent entrances, cul-de-sacs) and quarters form around cul-de-sacs. [source: [IEMed on Fez](https://www.iemed.org/publication/the-city-in-the-islamic-world-medieval-town-planning-the-preservation-of-architectural-heritage-in-fez/), [Chalmers thesis](https://research.chalmers.se/en/publication/1297)]

**Working defaults a generator can use, clearly labelled as judgement:** main street or market street about 8 to 12 m including its widening at the market, ordinary street about 4 to 6 m, lane or alley 2 to 3 m, fire alley (andron) narrower still. **[inference; no source found. Treat as a prior to be tuned.]**

### 4.2 Plots and blocks

- **Freiburg:** 50 by 100 Fuss, which at the 32.4 cm Freiburg foot is **16.2 by 32.4 m**. This is the first recorded mention of the Hofstatt size, and large parts of the old town were divided into equal plots of that size, varying where streets wind. [source: [Loebbecke, Parzellenstruktur Freiburg](https://regionalia.blb-karlsruhe.de/files/17820/BLB_Loebbecke_Parzellenstruktur_Freiburg.pdf), via a search summary]
- **Stollhofen** (a Zähringer-era foundation) also shows plots of about 50 by 100 Fuss. [source: [Gutmann](https://regionalia.blb-karlsruhe.de/files/18496/BLB_Gutmann_Stollhofen_Stadtplanung.pdf)]
- **Alresford** (c. 1200): plots of about 330 ft by 33 ft (about **100 m by 10 m**), which is 20 by 2 rods. [source: [alresford.org](https://alresford.org/info_pages/burghage.php), a local-history page]
- **Burford:** earliest (11th or 12th century) plots 1.5 perches wide and 22 to 28 perches deep. Later 12th century plots were 3 perches wide and 14 to 19 perches deep. [source: [VCH panel](https://ihr-archive.github.io/vch-explore/sites/explore/files/explore_assets/2010/03/22/panel2whole.pdf)] At 5.03 m per perch that is about 7.5 m by 110 to 140 m, then about 15 m by 70 to 95 m. **[inference]**
- **Frontages elsewhere:** 4.5 to 15 m on different King's Lynn streets, 9.2 m at Ludlow, an 18 m unit at Stratford-on-Avon. [source: VCH panel, as surfaced by search]
- **Alnwick:** about 28 ft (about 8.5 m), or a multiple or fraction of it. [source: [Alnwick Civic Society](https://alnwickcivicsociety.org.uk/2020/09/11/rods-poles-and-perches/)]
- **Florence:** a medieval house lot with a frontage of nine braccia (about 5.25 m). [source: [yourwaytoflorence](https://www.yourwaytoflorence.com/db/brunell/bottega.htm), a weak source]
- **Monpazier:** 400 by 220 m overall (8.8 ha). I did not find plot dimensions. [source: Cornell, above]
- **Kraków:** square about 200 by 200 m, about 4 ha, four blocks. The block dimensions are not retrieved. [source: Ancient History Sites, above]

**Ratio:** frontage-to-depth runs from about 1:2 (Freiburg) to about 1:10 (Alresford, Burford). Burgage plots are **not** all proportioned alike, so one global aspect ratio is wrong. [inference]

### 4.3 Wall circuits, areas and densities against population

**The best quantitative source.** Cesaretti, Lobo, Bettencourt, Ortman and Smith (2016) fitted settled area against population for **173 European cities c. 1300** (England 40, France and Belgium 63, northern Italy 30, Germany 40). [source: [PLOS ONE via PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC5051806/)]

- ln(area) = −2.125 + 0.714 · ln(population). That is area (ha) ≈ **0.12 · N^0.714**. The exponent's 95 per cent interval is 0.662 to 0.766, adjusted R² is 0.81, and regional exponents range from 0.71 to 0.79. The units are my reading from the paper's examples, hectares and persons. [source: same]
- The area is **settled area**, not walled area. [source: same]

My arithmetic from that fit **[inference]**:

| Population | Settled area (ha) | Density (persons/ha) | Circle-equivalent circumference (km) |
|---|---|---|---|
| 1,000 | about 17 | about 60 | about 1.5 |
| 5,000 | about 52 | about 96 | about 2.6 |
| 10,000 | about 86 | about 116 | about 3.3 |
| 20,000 | about 141 | about 142 | about 4.2 |

The circumference column is 2·√(π·A) for a circle. A real wall is longer for an irregular shape. As a check, Paris's 273 ha enclosure has a circle-equivalent of about 5.9 km against an actual 5.4 km wall (which is, however, only a half-ring plus the river). **[inference]** Nördlingen's walls are "a little under two miles" (about 3 km). [source: a forum summary, weak]

Other density figures:

- About 150 persons per ha for medieval cities, and about 75 for "garden" cities. [source: [Baghdad to London data set](https://druid.datalegend.net/dataLegend/Baghdad-to-London/assets/5e55298b461a77221fed174c)]
- Paris in 1328: at least 200,000 people, from 61,098 households at about 3.5 per household. [source: [Wikipedia](https://en.wikipedia.org/wiki/Medieval_Paris)] Against a 273 ha enclosure that is roughly **700 persons/ha** or more, so the walls cannot be read as the settled area. **[inference]** The sources disagree on both the area (253 to 273 ha) and the population (200,000 to 270,000).
- Cologne c. 1200: about 40,000 people, and "never exceeded about 45,000 until the end of the 18th century". The medieval wall is dated about 1180 and its length is given variously as 6, 8 or "over 8" km. I found no area. [source: [Wikipedia, History of Cologne](https://en.wikipedia.org/wiki/History_of_Cologne), [Cologne museum](https://www.museen.koeln/en/discovery/get-involved/accompanying-booklets/a-wall-with-a-2000-year-history) and others, in conflict]
- Philippe Auguste's earlier city, 1180: about 200 ha. [source: Wikipedia, above]

### 4.4 Market-place area against population

**I found no scholarly figure.** The only items:

- Kraków 1257: the square is about 4 ha out of a 32 ha founded town (about 12 per cent of area). [source and inference above]
- Monpazier: the town is 8.8 ha, with a central square that I could not measure.
- A forum rule gives market squares of about 50 to 100 yards on a side. [source: a forum, weak, so ignore it for tuning]

**Implication:** I cannot supply a per-1000-inhabitants market-area rule from a source. Do not hard-code one as if it were sourced. If one is wanted, use the fraction of founded area for planted towns (Kraków about 12 per cent) and a much smaller fraction for organic towns, treating both as a labelled judgement. [inference]

---

## 5. Ten worked examples

For each: phases, river relationship, market and castle or church, walls, one lesson. Where a fact was not retrieved I say "not retrieved".

### River towns on both banks

**1. Prague (Vltava).**
- Phases: Judith Bridge c. 1170. Old Town market, a market magistrate in 1212, walls in the 1230s and town status in 1230. Malá Strana founded 1257 on the left bank, renamed 1348. New Town founded by Charles IV in 1348. Hunger Wall in the second half of the 14th century. [sources in 2.2 and 3.3]
- River: a stone bridge links an old market on the right bank to a new royal town under the castle on the left bank.
- Market and castle: Old Town square near the landing, castle on the left-bank hill above Malostranské náměstí.
- Walls: separate rings for each town (Old Town in the 1230s, Malá Strana later).
- Lesson: the far-bank suburb can be a **deliberately founded second town, with its own market square, on the slope below the castle**, not an organic spill-over from the bridge.

**2. Florence (Arno).**
- Phases: Roman core on the right bank. A bridge at the Ponte Vecchio and a second at the Carraia (1128). Oltrarno palisade circuit in 1173 to 1175. Third circle 1284 to 1333. Floods in 1333. [sources in 3.3]
- River: three roads radiate from the Ponte Vecchio bridgehead on the left bank.
- Walls: the third circle enclosed an area not exceeded until the mid-19th century.
- Lesson: **later rings can be generous**, enclosing farmland that stays empty for centuries. Do not let a late wall hug the built area.

**3. Paris (Seine).**
- Phases: Île de la Cité, two bridges, a right-bank port and market, a left-bank university quarter, then three wall rings (about 1,700 m, 5,400 m, 4,900 m). [source: Wikipedia, Medieval Paris]
- River: an island core, with the port and halls on the right bank and the schools on the left.
- Market: Grève and Les Halles on the right bank, away from the island.
- Walls: right bank first (1190 to 1208), left bank later (1209 to 1220) and drier. Charles V's ring was mostly right bank and the left bank was only repaired.
- Lesson: **the two banks are not symmetric.** Give the commercial bank the bigger and earlier ring and the bigger market.

**4. London and Southwark (Thames).**
- Phases: Roman bridgehead on gravel islands, medieval London Bridge in the 12th century, a High Street market in the 13th, absorption as the Ward of Bridge Without in 1550. [sources in 3.3]
- River: Southwark is a linear suburb along the road from the bridge, with no city wall on the south bank.
- Lesson: a bridgehead suburb is a **linear strip along the approach road**, and its main street is the bridge's continuation, not a grid.

**5. Basel (Rhine).**
- Phases: first wall about 1080 (Bishop Burkhard von Fenis), Rhine bridge about 1225 joining Grossbasel and Kleinbasel, walls strengthened on both sides. [source: [Wikipedia, Walls of Basel](https://www.wikipedia.com/wiki/Aeschentor)]
- Lesson: the **bridge date precedes the second bank's ring**. The planned order is bridge, suburb, then a wall for both.

**6. Zürich (Limmat).**
- Phases: Lindenhof hill (Celtic and Roman), the Fraumünster abbey received market, toll and mint rights in 1045, a 13th century rebuild of fortifications enclosing the earlier areas. [sources: [History of Zürich](https://en.wikipedia.com/wiki/History_of_Z%C3%BCrich), [HLS](https://hls-dhs-dss.ch/articles/000171)]
- River: a short river with a town on each bank and bridges between the two minsters.
- Lesson: a **river between two churches** is a legitimate layout. The two banks can start from two ecclesiastical cores that the wall later joins.

**7. York (Ouse and Foss).**
- Three zones (Micklegate, the centre, Walmgate), joined at first only by Ouse Bridge for Micklegate. Chain across the Ouse (1380). No wall along the eastern marsh. [source: British History Online, above]
- Lesson: a bridge can be a **single choke point**, so the far-bank district has one main street from the bridge. Water and marsh can substitute for a wall.

### Towns that stayed on one bank, or mostly so

**8. Cologne (Rhine).**
- Medieval city on the left bank only. A Roman bridge and fort at Deutz (about 310 AD), no medieval wall around Deutz, annexed in 1888. About 40,000 people around 1200. The 1180 wall is a semicircle closed by the Rhine, with twelve gates. [sources in 3.3 and 3.5]
- Lesson: **the river can be the fourth side of the wall**. A large town can have no bridge at all, with the far bank a separate small place.

**9. Bruges (Reie canal ring).**
- Charter 1128. The core is in an oval canal ring, and the town grew concentrically around its first walls. Sea access via the Zwin from 1134 and silting from about 1500 to 1520. [source: [kupi](https://kupi.com/en/explore/belgium/bruges/history)]
- Lesson: for a town on a **canal or small river rather than a big one**, water is part of the street network, not a boundary.

**10. Ghent (Scheldt and Leie).**
- Two abbeys and a portus at the confluence, the Gravensteen (about 1180) and the Graslei and Korenlei quays facing each other on the Leie. [sources in 3.3]
- Lesson: a confluence supports **several quay frontages and several market places at once**.

**11. Lyon (Saône and Rhône).**
- Medieval town on the left bank of the Saône (Saint-Jean to Vaise), with the Presqu'île later and Renaissance-built. [source: French heritage dossier, above]
- Lesson: a long river can hold a **linear town on one bank for centuries** before a second bank fills.

**12. Kraków (a planted town, no river lesson).**
- Founded 1257 under Magdeburg law on a grid, square about 200 by 200 m. [source: Ancient History Sites, above]
- Lesson: if a culture preset includes "planted town", a **large central square occupying about four blocks** is the sourced pattern.

**Count check.** Both banks: Prague, Florence, Paris, London, Basel, Zürich, York (seven, which exceeds four). One bank: Cologne and Lyon. **A second one-bank example beyond Cologne is weakly supported.** Lyon is the only candidate, and its sources are weak and tourism-based. Verify Lyon, or choose another town, before relying on it.

### Beyond Europe

- **Fatimid Cairo and Fez** (a spine of market streets with branching cul-de-sacs, sections 3.4 and 4.1). [sources: IRB net, IEMed] If a "culture" is Islamic, the street hierarchy is **spine, secondary street, cul-de-sac**, with the cul-de-sac treated as private to its houses.
- I did not retrieve sources for Kyoto, Chinese or Indian walled-city grids. Treat any such preset as unsourced.

---

## 6. What generators get wrong

I found **no academic comparison** of a procedural town generator against real towns.

- **Watabou's Medieval Fantasy City Generator.**
  - The author says the generation method is "rather arbitrary" and the goal is "a nice looking map, not an accurate model of a city". [source: search summary of the [itch.io devlog](https://watabou.itch.io/medieval-fantasy-city-generator/devlog/1579/some-answers-and-comments)]
  - The same devlog says walls "make a pile of polygons look like a fantasy city map", that the alley and building algorithm is "a rather silly algorithm", and that adding rivers is "a bit tricky", while acknowledging water access matters for historical accuracy. [source: same devlog, fetched]
  - Walls and a castle are placed by default, which the author acknowledges is not always historical. [source: same]
  - Later versions added wall-less layouts, outskirts and smooth roads. [source: [devlog 0.3.0](https://watabou.itch.io/medieval-fantasy-city-generator/devlog/3091/030-wall-less-layouts-city-outskirts-smooth-roads)]
- **CityEngine and its papers.**
  - Parish and Müller (SIGGRAPH 2001) use L-systems for roads. They take land-water boundary and population-density maps as input and subdivide the land into lots. [source: [SIGGRAPH history archive](https://history.siggraph.org/?p=116923), secondary]
  - Chen et al. (2008) offer tensor-field street modelling as an alternative. [source: course slides, [phiresky](https://phiresky.github.io/procedural-cities/presentation.html)]
  - I found no medieval-accuracy evaluation of either.
- **Inferences about what such generators tend to miss.** They follow from sections 1 to 5 and are not sourced critiques **[inference]**:
  1. They draw a town as one growth event. Real towns have phases with different plot modules.
  2. They put a wall round every town, and make it hug the built area.
  3. Water is missing or decorative, though water access is why most towns exist.
  4. Both banks are treated alike, though Paris and Florence show they were not.
  5. Plots are uniform in size, though burgage widths vary by street and by phase.
  6. Blocks are all convex polygons, with no long burgage strips and backland.

---

## 7. What I could not verify

- Street widths by class, with any scholarly source.
- Market-place area per 1,000 inhabitants.
- Wall circuit length against population, directly. Only my circle-equivalent arithmetic, and Paris, Cologne and Nördlingen as spot checks.
- Caniggia and Muratori, and Barthelemy.
- Bruges's Markt and Burg positions and wall dates, Lyon's bridge dates, Zürich's bridge dates.
- Bastide plot sizes. A "8 by 24 m" figure I tried was not confirmed anywhere.
- Plan images. None retrieved.

---

## 8. Observations a generator should reproduce

Each rule is marked [source] or [inference]. Rules without a source are priors to be tuned, not facts.

1. **Settled area follows a sublinear power law in population: about 0.12 · N^0.714 hectares** (so 1,000 people need about 17 ha, 5,000 about 52 ha, 10,000 about 86 ha, 20,000 about 141 ha). Regional exponents run 0.71 to 0.79. Density therefore **rises with size**: about 60, 96, 116 and 142 persons/ha. [source: Cesaretti et al. 2016; the table values are my arithmetic]
2. **Walls do not equal settled area.** Allow intramural vacant space and extramural suburbs. Bristol: 55 ha walled, 130 ha settled. [source: same]
3. **A wall circuit for a town of N people is roughly the circle-equivalent of its settled area**: about 1.5 km at 1,000, 2.6 km at 5,000, 4.2 km at 20,000. A real wall is somewhat longer when irregular. Paris at 273 ha had a 5.4 km wall. [inference, checked against Paris and Nördlingen]
4. **Cities outgrow their walls.** Paris had 200 ha by 1180, and its wall enclosed 273 ha in 1190 to 1220 and then 439 ha by 1371. Expect **at least one ring extension** for a large town. [source: Wikipedia, Medieval Paris]
5. **Rings are not concentric offsets.** Paris's three rings covered 5.4 km and 4.9 km after a 1.7 km one, with the left bank only repaired. A later wall may be generous (Florence's third circle was not exceeded until the mid-19th century), or mostly one-sided (Paris). [source: Wikipedia, Britannica]
6. **The commercial bank is walled first and more heavily.** Paris right bank first (1190 to 1208), left later and drier. [source: City walls of Paris]
7. **The far bank can be a founded second town** with its own market square: Malá Strana (1257), with the Old Town's market and walls (1230s) already in place. [source: Wikipedia, Britannica, praha.eu]
8. **A bridge precedes the second bank's ring.** Prague bridge c. 1170 then Malá Strana 1257. Basel bridge c. 1225, then both parts walled. [source: sources above]
9. **A bridgehead suburb is a strip along the approach road**, as at Southwark's Borough High Street, with a market on that street from the 13th century. [source: Southwark Council, Wikipedia]
10. **A far-bank district may have a single link** (Micklegate, York, only Ouse Bridge) or several roads radiating from the bridge head (Oltrarno, three). [source: British History Online, Studentsville]
11. **A river can be one side of the wall** (Cologne's semicircle), **or be crossed by a chain between two towers** (York 1380, Norwich), **or by a wall with a water tower** (Chester, Nuremberg). Leave a stretch of marsh unwalled where it already protects. [source: York, Chester, Norwich, Nuremberg pages above]
12. **Markets begin at a landing, a bridgehead or a junction**, not necessarily at the geometric centre: Grève and Les Halles (1137), Prague Old Town, Malostranské náměstí under the castle slope. [source: Wikipedia, Britannica]
13. **Market streets taper**, so a spindle-shaped market is as legitimate as a rectangular square. [source: VCH panel]
14. **Planted towns have a large regular square that sets the module.** Kraków: about 200 by 200 m, four blocks, about 12 per cent of the founded 32 ha. Monpazier: a 400 by 220 m rectangle, 8.8 ha, with four cross streets and fire alleys between houses. [source: Ancient History Sites, Heidelberg, Cornell; the percentage is my division]
15. **Burgage frontages are narrow and the depth ranges widely**: Freiburg 16.2 by 32.4 m (50 by 100 Fuss), Alnwick about 8.5 m (28 ft) frontage, Ludlow 9.2 m, Alresford about 10 by 100 m, Burford 7.5 m wide and 110 to 140 m deep early, 15 m by 70 to 95 m later, King's Lynn 4.5 to 15 m, Florence about 5.25 m. [source: Loebbecke, Alnwick Civic Society, VCH panel, alresford.org, yourwaytoflorence; conversions are mine]
16. **Plot modules persist.** Alnwick's plots are still multiples or fractions of 28 ft. Use a per-phase module and **keep it when buildings change**. [source: Alnwick Civic Society]
17. **Backland fills over time**: Edinburgh backlands carried three or four substantial buildings by the early 16th century. Late plots should have more infill than early ones. [source: Cambridge, early urban Scotland]
18. **Plots subdivide and rarely amalgamate before the 19th century.** [source: Wikipedia, Burgage]
19. **Waterfront plots run from street to water**, with house, shop, warehouse and private quay on one narrow plot (King's Lynn). [source: Nottingham eprint, inferred layout]
20. **Quay frontages can face each other across a river** (Graslei and Korenlei in Ghent) **and ports migrate** when one fills (Grève to Port de l'École). [source: Wikipedia, Medieval Paris]
21. **The most-integrated streets should coincide with economic space**: markets and the main street. Check with integration or betweenness on the generated graph. [source: Craane on Utrecht]
22. **Street bearings of an organic town have high entropy and few four-way junctions.** A planted town has low entropy and many four-way junctions. Measure both with Boeing's metrics. [inference from Boeing's definitions; no medieval benchmark number was found]
23. **Institutional and open land collects at old edges** (fringe belts). When an old wall is superseded, leave its ring as a band of irregular large plots. [source: Journal of Urban Morphology for plot irregularity; mechanism is inference]
24. **Islamic-culture presets**: a spine of market streets, secondary streets branching off it, and cul-de-sacs at least 7 cubits wide treated as private to their houses. [source: IRB net paper]
25. **Mark priors as priors.** Street widths and market-area ratios have no source in this report. Tune against a stated target and record the source or the judgement. [inference]

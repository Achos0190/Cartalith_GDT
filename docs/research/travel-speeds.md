# Historical Overland and Maritime Travel Rates: A Research Report for Pre-Industrial Logistics Modeling

## 1. Methodology and Framing

Every historical travel-speed figure conflates two different things: the physical rate of motion of an animal or vessel, and the social/logistical rhythm imposed on it by rest, provisioning, security, and trade. To build a usable model these must be kept separate. This report defines three tiers, applied consistently to every mode and terrain:

- **Travel-day speed** — the distance covered during hours actually spent moving, on a day devoted entirely to travel. This is close to the "in progress" rates quoted by sources like Britannica (2–3 mph for camels) or Casson's 4–6 knots for ancient sailing ships.
- **Calendar-average speed** — total distance divided by total elapsed days, including scheduled rest days, but *excluding* major disruptions (weather-bound waiting, political delays, illness, repairs). This is what caravanserai spacing, mansiones spacing, and most "X km/day" figures in the secondary literature actually describe.
- **Expedition-average speed** — total distance divided by the *true* door-to-door duration of a specific historical journey, including everything: seasonal waiting for monsoon or snowmelt, seasons closed to travel, illness, warfare, diplomatic delays, and resupply. This is the number you get when you divide a documented journey's total distance by its total documented duration (e.g., Chang'an to Rome in "6 months to over a year").

Most popular sources collapse all three into one "average speed" figure, which is why they disagree so sharply — a source citing 5 km/h in-motion pace and a source citing 20 km/day calendar-average are not contradicting each other; they are answering different questions. Where possible this report traces each figure back to which tier it actually belongs to.

---

## 2. Overland Modes: Evidence by Mode

### 2.1 Merchant caravans (general) and camel caravans

The core reference figure, repeated across tertiary sources but traceable to caravan-logistics literature and Britannica's synthesis, is that a caravan **in motion** covers 2–3 mph (3–5 km/h) for 8–14 hours a day, in hot regions often shifted to night travel<cite index="3-1,15-1">to avoid the mid-day heat, with caravans stopping at caravansaries that provide shelter and supplies</cite>. That is a *travel-day* figure, and taken naively (say 10 hours × 4 km/h) it would suggest 40 km/day, which overstates what caravans actually achieved once loading, watering, and pasture time are included.

The more diagnostic evidence is physical: caravanserais on the best-maintained Central Asian Silk Road segments were spaced <cite index="7-1">every 30 to 40 kilometers</cite>, and other sources converge on <cite index="8-1">roughly 32–40 kilometers (20–25 miles) apart — about a day's journey</cite>. This spacing *is* the calendar-average day-stage distance that the infrastructure was built around — the single best physical proxy we have for how far a functioning trade caravan was expected to move in a normal day, independent of anecdote.

For the trans-Saharan desert crossing specifically, the World History Encyclopedia states a working camel's **sustained travel capability** is about <cite index="9-1">48 km per day</cite>, while the OER Project's account of the actual documented crossing — <cite index="10-1">a caravan traveled around 20 miles a day, taking 70 days to cross the desert</cite> — gives a **calendar-average of ~32 km/day** over a genuine multi-week expedition. The gap between 48 km/day (capability) and 32 km/day (achieved) is exactly the caravan's real-world margin for watering, grazing, and heat management, and it is a useful ratio (~0.65–0.7) to carry into the model generally.

Ibn Battuta's testimony that <cite index="13-1">the average size per caravan was 1,000 camels; some caravans were as large as 12,000</cite>, and independent confirmation of a <cite index="13-1">70 to 90 day crossing</cite>, corroborates the OER figure within a consistent range (32–36 km/day calendar-average for Saharan crossings of large-to-very-large caravans).

### 2.2 Silk Road caravans specifically

Popular tertiary sources vary between 25–40 km/day and 20–25 km/day for Silk Road caravan speed, and disagree partly because they are quoting different tiers. A representative synthesis: Chang'an to Samarkand (~3,000 km) in 4–6 months implies a calendar-average of roughly **17–25 km/day**, and the full Chang'an-to-Mediterranean route (6 months to over a year for ~10,000 km of actual road distance, accounting for detours) implies an **expedition-average as low as 15–30 km/day** once winter closures, mountain-pass waiting, and political delays at frontiers are folded in. This is meaningfully lower than the "in-motion" 3–5 km/h figure, and lower even than the trans-Saharan calendar-average, because the Silk Road crosses far more varied and difficult terrain (Tian Shan and Pamir passes, Taklamakan margins, Iranian plateau) than the comparatively uniform Sahara.

### 2.3 Ox-drawn wagons

This is one of the best-corroborated figures in the whole study, because it recurs almost identically across independent traditions:

- Medieval Europe: oxen wagons at <cite index="42-1">ten to twelve miles a day, compared to the twenty miles a day of a mule- or horse-drawn wagon</cite>; a separate synthesis gives ox wagons "in the range of 10 miles" against 20 for horse-wagons.
- 19th-century American frontier: <cite index="44-1">covered wagons pulled by oxen would be happy to make 10 miles a day</cite>, and the Oregon/California Trail overall averaged <cite index="51-1">10-15 miles per day</cite> — note this **includes** the many delays of a real multi-month migration, so it is already close to an expedition-average.
- Colonial Australia: bullock teams <cite index="46-1">covering roughly 15 kilometers daily</cite>, with staging posts at <cite index="45-1">12-mile (19 km) intervals, the usual distance for a team to travel in a day</cite>.
- Roman world: the ox-drawn *plaustrum* could travel <cite index="16-1">only about 10-15 miles (approximately 15 to 25 km) per day</cite>.

The convergence across Rome, medieval Europe, colonial Australia, and the American frontier — all landing between 15 and 20 km/day — is unusually strong for a pre-industrial transport figure, because the rate-limiting factor (an ox's sustainable pulling endurance, not road quality or culture) is biologically constant. **15–19 km/day is a very high-confidence calendar-average for ox wagons on tolerable roads**, with the low end applying to rough tracks and the high end to good, packed trade roads.

### 2.4 Horse- and mule-drawn wagons, and pack mules/donkeys

Horse- or mule-drawn wagons consistently run at roughly double the ox rate: <cite index="42-1">a mule- or horse-drawn wagon covered an average distance of twenty miles a day</cite>, with a plausible range of <cite index="42-1">fifteen to twenty-five miles in a day's time, with rest stops about every ten miles</cite> under good road conditions. Peter Spufford's archival work on late-medieval merchant carriers (a genuinely academic source, cross-checked against dated correspondence) gives a very similar figure independently: <cite index="62-1">the normal distance travelled by any type of carrier in one day seems to have been in the region of 30 to 40 km</cite>, rising above 40 km/day for lightly loaded packhorses on easy roads, and a documented **horse convoy averaging 50 km/day** over six days between Dijon and Paris in January 1412.

For pack mules specifically, the best-documented figures come from 19th-century Californian and Mexican muleteering, where the *arriero* tradition was highly professionalized and closely observed: <cite index="85-1">forty to fifty mules in a train... will travel from twenty-five to thirty-five miles per day, without becoming weary</cite> on manageable terrain, dropping sharply in genuinely mountainous country. This 40–56 km/day travel-capable range matches Spufford's medieval Alpine data, where <cite index="62-1">the stages for pack animals over the Simplon pass were a little more than 30 km apart</cite> — i.e., in a genuine high mountain pass, even mule stages contract to ~30 km/day, which is the calendar-average figure to use for mountain terrain specifically, not the 40–56 km/day figure that applies on easier ground.

### 2.5 Roman paved roads and the cursus publicus

This is the terrain category with the widest scholarly disagreement, and it is worth resolving carefully because it anchors the "paved road" line of the model.

Ordinary travelers on Roman roads, unaided by the relay system, are usually placed at a **modest** calendar-average: one synthesis states plainly that <cite index="20-1">a good day's travel was usually less than 20 miles (32.2 km)</cite>, and another gives <cite index="22-1">in summer, a traveler could walk or ride about 30 to 40 kilometers a day</cite>. These two are compatible once tiered correctly: 32 km/day is the *typical* case, 30–40 km/day the *good-conditions* case.

The **cursus publicus** (the state courier/post system, not ordinary travel) is a different regime entirely, built on relay stations. Estimates vary by an order of magnitude depending on which ancient testimony is used:

- A.M. Ramsey's classic philological study in the *Journal of Roman Studies*, reconstructing timings from the *mansiones* spacing recorded in the Jerusalem Itinerary, gives <cite index="16-1">41 to 64 miles per day (66-103 km per day)</cite> for a typical single-messenger trip — this is the most rigorously sourced figure in the literature and should be treated as the authoritative baseline.
- Other secondary syntheses citing the same Roman-mile-per-day tradition converge close to this: <cite index="19-1">about 50 miles per day</cite>, and mansiones (night-stop stations) spaced <cite index="21-1">approximately 15 miles or 25 to 30 km apart</cite>, consistent with a ~50 Roman-mile relay day built from two mansiones-stages.
- One GIS-based academic secondary source (Carreras & de Soto 2013, *Historical Methods*) states cursus publicus speeds <cite index="17-1">reaching 800 km per day</cite> — this figure is a clear outlier, almost certainly describing a theoretical maximum achievable only under emergency, single-message, multi-relay conditions (compare the documented case of Julius Caesar's ~1,280 km in eight days = 160 km/day, itself already an exceptional forced effort, and reports of <cite index="22-1">messengers traveling over 500 kilometers in a day and a half</cite> — i.e., ~330 km/day). **800 km/day is not a sustained rate and should not be used**; it likely reflects a misreading of total relay-network throughput (many riders in sequence over many days) as a single day's distance for one message.

**Verdict**: Ramsey's 66–103 km/day is the correct academic baseline for the cursus publicus, sitting between the frequently-cited "50 miles/day" popular figure and the emergency-relay outliers of 160–330+ km/day. Ordinary travel on the same roads, without the relay privilege, is 4–5× slower: 30–40 km/day travel-day/calendar-average.

### 2.6 Travelers on foot

Foot travel is the best-documented category because it is the human default, and the evidence is unusually consistent once separated by tier:

- Ian Mortimer's *Time Traveler's Guide to Medieval England* (a peer-reviewed-adjacent academic popular history built from primary account-book research): 15–20 miles/day normal, 6–8 miles/day in bad weather, "many accounts of 30 miles per day under the right conditions."
- German medieval-logistics synthesis: <cite index="59-1">under normal conditions, 20-30 km per day was realistic; on good paths and with light equipment, up to 40 km</cite>, with the Hellweg trade route staged in <cite index="59-1">sections of 15-30 km</cite> and a <cite index="59-1">weekly distance of 80-120 km achievable, as Sundays were considered days of rest</cite> — this is a genuine calendar-average figure (including a fixed rest day) and it lands at 13–17 km/day averaged over the week, notably lower than the "20-30 km per day" quoted for travel days alone. The same source documents a specific pilgrimage — Oppenheim to Venice, 750 km in 15 days, ~50 km/day — explicitly flagged as "ambitious but feasible," i.e., an upper-bound achievable calendar-average for a fit, motivated, lightly-burdened traveler on good roads.
- Archbishop Eudes Rigaud's 1254 retinue, a well-documented non-commercial journey: <cite index="62-1">averaged 33 km (20 miles) a day between Paris and Dijon</cite>, with daily variation of <cite index="62-1">between 20 and 45 km</cite> — this is a genuine expedition-average, since the itinerary explicitly includes social stops, and it sits right in the middle of the ranges above.
- Ramon Llull Foundation's synthesis: <cite index="61-1">on foot, the average distance travelled in one day was about 25 kilometres and could even reach 50 or 60 in the case of professional couriers</cite>.

**Verdict**: 20–30 km/day travel-day pace, 20–25 km/day calendar-average once rest days are included, 15–20 km/day expedition-average for genuinely long journeys with social, political, or seasonal friction, is a well-supported convergent range. 40–50 km/day is achievable but represents an upper bound for fit, unburdened travelers on good roads, not a sustainable long-distance default.

### 2.7 Military columns (comparison only)

Roman legionary march rates are unusually well studied because Vegetius quantifies them directly, and modern reconstructions (using energy-expenditure and biomechanical modeling, e.g. the Whip 1998 marching-pace study) confirm the ancient figures rather than contradicting them:

- Vegetius's *iter iustum* ("normal march") is <cite index="55-1">20 Roman miles... in five summer hours</cite>, ≈ 29–32 km/day, at a pace independently reconstructed at <cite index="53-1">1.2741 m/s (4.59 kph)</cite> — a genuinely brisk sustained walking speed for a loaded soldier.
- A synthesized range from multiple ancient military writers and modern historians: <cite index="52-1">routine marches 22–30 km/day, sustained campaign marches 30–37 km/day, forced-march maximum (short-term) 45–60 km/day</cite>, with some estimates of forced marches reaching <cite index="54-1">60–75 km/day under pressure</cite>.
- Crucially, the legion's *sustainable* campaign pace was often capped not by the soldiers but by its baggage train: <cite index="52-1">most supplies for the Legion was hauled in ox carts and they were generally limited to ten miles a day</cite> — i.e., an army moving with its full logistics train reverts to the ox-wagon rate (~16 km/day), while an army moving light or force-marching can hit 45–75 km/day for short bursts. This mirrors exactly the caravan-size dynamic discussed in Section 5: the *slowest necessary component* sets the sustained rate.

This comparison matters for the model because it shows that "military" and "civilian" travel are not fundamentally different physical regimes — they differ in how much weight is put on discipline, forced marching, and jettisoning the baggage train, all of which are logistics decisions, not different terrain physics.

### 2.8 Couriers and messengers (comparison only)

Relay-based courier systems are the clearest illustration of what pure "travel-day" speed looks like when calendar and expedition frictions are engineered out:

- Roman cursus publicus: 66–103 km/day (Ramsey, Section 2.5).
- Achaemenid Persian royal road couriers: praised by Herodotus for continuing regardless of weather (the ultimate ancient statement of a courier system minimizing calendar friction), though the Persian relay was reportedly faster than the single-rider Roman system it inspired.
- Mongol Yam: relay stations spaced <cite index="71-1">25 to 40 kilometers (15-25 miles) apart</cite>, with express messengers achieving <cite index="76-1">200-300 kilometres (120-190 mi) per day</cite> according to Wikipedia's synthesis of Weatherford and Lane, and some secondary sources citing up to 400 km/day for the fastest couriers. This is roughly 3× the Roman cursus publicus rate, attributable to smaller, hardier steppe horses bred specifically for endurance relay riding, denser station spacing, and riders binding themselves to the horse to ride through exhaustion — a genuinely different operational model, not just better roads.
- Pony Express (1860-61, included for scale comparison though outside the "ancient/medieval" scope): <cite index="75-1">157 stations spaced about 10-15 miles apart across 1,966 miles, achieving transcontinental mail delivery in roughly 10 days</cite> — an average of ~316 km/day sustained across a 10-day relay journey, corroborating that 200-300+ km/day is physically achievable for a well-organized horse-relay system regardless of era, and is not a Mongol-specific exaggeration.
- Late medieval Italian merchant couriers (Spufford, via Melis's letter-date archive): the <cite index="62-1">17,000 letters between Florence and Genoa normally took 5 to 7 days to deliver</cite> — for ~200 km overland/coastal, this is a comparatively modest ~30–40 km/day, reflecting that most medieval commercial correspondence went by ordinary paid messenger, not a state relay system.

**Verdict**: the achievable ceiling for relay courier systems is 200–300 km/day sustained (Mongol Yam, Pony Express), roughly 3× the best-attested state postal system without small-horse steppe relay (Rome, ~66-103 km/day), and roughly 10x ordinary unaided horseback travel.

### 2.9 River transport

River transport is asymmetric by nature, and the clearest quantification comes from ethnoarchaeological reconstruction rather than textual sources: Drennan's study of Mesoamerican/South American dugout cargo boats found <cite index="92-1">dugout boats covered about 20km per day upstream and 40km downstream on a sluggish river</cite>. This 1:2 ratio is a useful default when a river's current strength is otherwise unknown.

For freight-cost economics (relevant to why river/sea transport dominated bulk goods wherever available), Rahn's analysis of Roman Germania river-boat data found that <cite index="92-1">land transport of bulk material was 28 times as costly as sea transport</cite>, with <cite index="92-1">river boats hauled upstream by one horse or 7 to 8 people</cite> carrying loads of <cite index="92-1">3 to 7 tons</cite> versus an ox-wagon's <cite index="92-1">average load of only about 262kg</cite> — river boats moved roughly 15-25× the payload of a wagon at a comparable or better calendar speed, which is why every pre-industrial civilization routed bulk trade via water wherever geography allowed. Diocletian's Edict of Maximum Prices (301 CE) independently confirms the cost hierarchy: <cite index="92-1">transport on rivers was 3.9 to 7.8 times as expensive as transport on sea ships</cite>, itself far cheaper than land transport — sea < river < road, in that order, for both cost and (usually) calendar speed.

---

## 3. Maritime Travel

### 3.1 Mediterranean sailing (Roman/Classical baseline)

Lionel Casson's *Ships and Seamanship in the Ancient World* — the standard academic reference for this topic, built from underwater archaeology and textual cross-checking, not just literary anecdote — anchors the Mediterranean figures used across the secondary literature. The consistently repeated synthesis is: <cite index="33-1">ships would usually ply the waters of the Mediterranean at average speeds of 4 or 5 knots, the fastest trips reaching average speeds of 6 knots</cite>, with the favorable-wind run from <cite index="33-1">Ostia to Alexandria taking about 6 to 8 days</cite> (~1,600 nautical-route km, implying ~200-270 km/day of favorable downwind sailing) while the reverse, upwind leg took far longer — sometimes over a month — illustrating that **direction relative to prevailing wind is as important a variable as vessel type**, and arguably more important than the terrain-analogue variables that dominate overland travel.

A modern comparative-history discussion independently corroborates Casson's range and notes its durability: pre-clipper sailing-ship speeds of <cite index="34-1">4 to 6 knots... seem typical for even 19th century sailing ships</cite>, implying that hull-and-rig-limited cruising speed changed remarkably little from antiquity to the age of sail's twilight; the major later gains (clippers, copper bottoming) affected top speed and windward performance more than typical cruising speed. This is a useful modeling simplification: **pre-industrial "typical sailing speed" is a near-constant 4-6 knots (7.4-11 km/h) across two millennia**, and what varies dramatically between eras and regions is not hull speed but *route efficiency* — how much of that speed is usable given wind patterns, coastal hugging requirements, and night-sailing practices.

Commercial sailing itself was strongly seasonal: the Mediterranean shipping season was suspended for roughly four months (*Mare Clausum*), a scheduling constraint with no overland analogue of comparable severity except high mountain passes and, to a lesser degree, monsoon-locked Indian Ocean sailing (Section 3.2).

### 3.2 Indian Ocean dhow voyages and the monsoon system

Indian Ocean sailing is fundamentally organized around the monsoon, and the academic literature (Copeland's 2026 *Economic History Review* article, built from ERS-1 satellite wind data applied to a modern sea-trial reconstruction of a historical trading vessel — a genuinely rigorous methodology) gives the clearest sustained-speed figure available for any pre-modern sailing tradition: ancient and medieval Indian Ocean vessels <cite index="27-1">could average as much as 11 kph on extended voyages, with an even higher top speed in very good conditions</cite>, when running with a favorable monsoon.

This is broadly consistent with (if somewhat faster than) the Mediterranean's 4-6 knot (7.4-11 km/h) range, which makes sense: dhow lateen rigs are specifically optimized for reaching and running with a steady monsoon, whereas Mediterranean vessels more often had to contend with variable and contrary local winds.

A documented early-19th-century case — the Mozambique-to-India run, <cite index="25-1">lasting about a month at the start of the 19th century</cite> for <cite index="25-1">3-5 ships per year plying this route out of Diu and Daman</cite> — gives a real expedition-average considerably below the 11 km/h sustained-sailing figure, because it necessarily includes provisioning, waiting for the correct point in the monsoon window, and coastal maneuvering at both ends. This is the single most important structural fact about dhow voyages: the *sustained sailing speed* (≈11 km/h, i.e. ~260 km/day if sailing continuously) is almost never the *expedition-average* speed, because voyages were bound to a fixed seasonal departure window and could not simply "leave early" the way an overland caravan could adjust its own schedule by a few days.

Duration of Indian Ocean trading voyages, drawn from the broader literature on the Periplus Maris Erythraei and the dhow trade generally: a one-way monsoon-borne crossing of the open Arabian Sea (e.g., Red Sea/Gulf ports to the Malabar coast) is typically weeks, not months, when timed correctly with the monsoon; the season-bound nature of the trade means total round-trip turnaround (including the wait for the return monsoon) was often **the better part of a year**, not because sailing itself is slow but because a ship that arrives outside its return window must wait months for the wind to reverse. This "monsoon lock-in" is the dhow-trade equivalent of the Mediterranean's *Mare Clausum* winter closure, but with a much larger schedule penalty, since missing the window can cost 4-6 months rather than a few weeks.

### 3.3 Sheltered bays, coastal waters, and open sea

No single academic source gives calibrated calendar-average figures separately for these three zones, so the values below are triangulated from the general sailing-speed evidence (Sections 3.1-3.2) combined with well-established operational logic:

- **Sheltered bays**: low but very *reliable* speed — protected water allows rowing/poling to supplement sail, and there is no need to reduce sail for open-sea safety margins, but the enclosed space limits sustained tacking distance and there is more traffic/maneuvering. Small craft (fishing boats, harbor lighters, riverine-coastal hybrids) dominate this category, and speeds are closer to the low end of oar-and-sail hybrid propulsion.
- **Coastal waters**: vessels typically shortened sail overnight and anchored, or hugged the coast at reduced speed, because pre-modern coastal pilotage required visual landmarks and grounding was a serious risk — this constrains coastal sailing to daylight hours in a way open-sea sailing is not, which is why coastal calendar-average speed is frequently *lower* than open-sea calendar-average speed despite the calmer water, a genuine counter-intuitive point worth flagging explicitly in the model.
- **Open sea**: once a vessel commits to open water (e.g., the direct monsoon crossing of the Arabian Sea, or a Mediterranean vessel running before the Etesian winds from Rhodes to Alexandria) it can sail through the night, since there is no coastline to run aground on, and can average close to its full hull-limited cruising speed for days at a stretch. This is exactly why the Ostia-Alexandria run (largely open-water) achieved 200-270 km/day equivalent, while coastal Roman shipping (hugging Italy, Greece, and Anatolia) is documented at markedly lower calendar-average rates despite comparable ship technology.

---

## 4. Terrain Categories

The travel-day / calendar-average / expedition-average figures below are the model's core output. Two things must be understood about how terrain interacts with the other variables before reading the table:

**Terrain and biome are not independent.** "Rocky" is a surface-friction/footing problem that slows the pace of a single travel-day regardless of climate; "jungle" or "desert" are biome problems that constrain *water, forage, and heat management*, which mostly affect the calendar-average and expedition-average tiers (how many rest/resupply days are needed, how large the safety margins have to be) rather than the travel-day tier. **Rocky tropical jungle is therefore not simply "rocky slowness + jungle slowness added together"** — it is rocky-terrain travel-day pace (surface friction) combined with jungle-biome calendar/expedition penalties (humidity, disease, forage scarcity, machete-clearing where no path exists). This is why, in the table below, jungle's *travel-day* speed is not dramatically worse than open forest (assuming an established trail/route — porters and caravans essentially never bushwhacked virgin jungle for long-distance trade; they used known paths), but jungle's *calendar-* and *expedition-average* speeds fall much further below its travel-day speed than a temperate forest's does, because of disease-driven rest stops, higher animal mortality, and slower resupply. The Stanley expeditions are the extreme illustrative case: Stanley's 1871 march covered roughly 1,127 km in about 234 days (~4.8 km/day expedition-average) against porters who could straightforwardly walk 20-25 km on a good travel day — nearly all of that gap is disease, desertion, negotiation with local chiefs, and forced rest, not the physical difficulty of walking the path itself.

**Mountain passes are the inverse case**: the travel-day pace itself collapses (steep grades, switchbacks, altitude, load redistribution), but *if* the pass is open at all, the calendar-average doesn't fall much further below the travel-day pace, because pre-modern travelers crossing a known pass moved as efficiently as the terrain allowed with little slack to lose — the real expedition-average penalty for mountains is binary and seasonal (the pass is closed by snow for months, or it isn't), which is why mountain routes have some of the largest gaps between calendar-average and expedition-average of any terrain type once a multi-month journey is averaged across a full year that includes a closed season.

| Terrain | Travel-day speed (km/day) | Calendar-average (km/day) | Expedition-average (km/day) | Why they differ |
|---|---|---|---|---|
| Paved road (Roman-equivalent) | 40–50 (unaided traveler); 66–103 (relay courier) | 30–40 (unaided); 50–70 (courier, non-emergency) | 25–35 | Mansiones spacing (25–30 Roman miles) sets the practical daily stage; expedition-average drops for weather, tolls, border/customs stops. Relay couriers barely lose speed calendar-to-expedition since they carry no goods and stop for nothing but the relay itself. |
| Maintained dirt/trade road | 25–35 | 20–30 | 15–25 | Ox/mule/horse wagon and caravan data (Sections 2.1, 2.3, 2.4) converge here; caravanserai/way-station spacing of 30-40 km sets the calendar-average ceiling. |
| Rocky terrain | 12–18 | 8–12 | 6–9 | Footing risk to pack animals (lameness, cast shoes) forces slower travel-day pace than dirt road; calendar-average falls further because rocky stretches are usually short sections of longer routes requiring extra rest for animals afterward. |
| Mountain pass (open season) | 15–25 | 12–20 | 8–15 (in-season); effectively 0 in closed season | Simplon-pass pack-stage data (~30 km) is a *favorable* mountain case; steeper/higher passes are worse. The dominant expedition-average risk is a multi-month seasonal closure, not daily grind. |
| Temperate forest (established trail) | 20–28 | 16–22 | 12–18 | Close to open dirt-road figures once a trail exists; slightly reduced for deadfall, stream crossings, and reduced visibility/navigation. |
| Tropical jungle (established trail) | 15–22 | 8–14 | 4–9 | Travel-day pace only moderately below temperate forest (an established trade path is still walkable); expedition-average collapses due to disease, heat/humidity exhaustion, animal mortality, and negotiation/toll delays with local polities — see Stanley case above. |
| Steppe | 25–35 (foot/wagon); 50–100+ (horse relay, Section 2.8) | 20–30 (foot/wagon); 40–80 (mounted relay) | 15–25 (foot/wagon); 30–60 (mounted relay) | Open, largely flat terrain removes the footing/routing penalties of forest or mountain; the Mongol Yam's 200-300 km/day figures are the ceiling case for a purpose-built mounted-relay system on this terrain, not a caravan. |
| Desert | 30–40 (camel travel-day capability) | 20–32 (documented Saharan/Hajj crossings) | 12–22 | Camel physiology (48 km/day capability) sets a high travel-day ceiling, but water/pasture scarcity forces the OER/Ibn Battuta-consistent ~20mi/32km calendar-average; expedition-average falls further for raiding risk, well failure, and seasonal (cool-season-only) departure windows. |
| Marsh | 10–15 | 6–10 | 4–8 | No direct historical source quantifies marsh specifically; figure is interpolated as worse than rocky terrain (footing failure plus route-finding around impassable ground) but better than true jungle (no disease-driven expedition losses of the same severity, assuming passage rather than prolonged habitation). Treat with lower confidence than sourced categories. |
| Coastal lowlands (overland route) | 25–35 | 20–28 | 15–22 | Broadly dirt-road/steppe-like footing, but travel-day pace is often reduced by tidal inlets, river-mouth crossings, and detours around estuaries; where a coastal road existed (e.g., Roman coastal viae) treat as dirt/trade-road category instead. |

---

## 5. Caravan Size and Speed

The evidence across every mode points the same direction: **caravan speed is set by its slowest essential component, not averaged across its members**, and larger caravans are not inherently slower *per member* — but they are slower in aggregate logistics terms, for three separable reasons that should not be conflated:

1. **Pace-setting by the weakest animal/person.** The Silk Road synthesis states this explicitly: <cite index="5-1">the pace of a caravan on the Silk Road was dictated by its slowest member, often a laden pack animal or a weary traveler</cite>. This effect is present whether the caravan has 6 members or 600 — a caravan of 6 healthy, well-matched travelers with no weak animal can move *faster* than a single unlucky solo traveler with one lame horse.

2. **Loading and deployment time scales with size, not with pace.** A larger caravan takes longer to load, water, organize into marching order, and make camp each day, which eats into the *travel-day* hours available without reducing the in-motion pace itself. The Andean llama-caravan ethnoarchaeology (Tripcevich's dissertation, drawing on Nielsen's fieldwork) documents this precisely: <cite index="104-1">rest days are taken regularly on caravan routes that exceed six days, with one rest day for every three to five days of travel</cite>, and camp-siting is dominated by <cite index="104-1">the needs of the herd animals</cite> — a fixed daily overhead that is roughly constant per caravan regardless of size, but represents a proportionally larger fixed cost for a small caravan (6 people still need to find water and pasture; a caravan of 600 needs far *more* of it, but the search-and-negotiate time is not 100x longer for 100x the animals).

3. **Security and toll/negotiation overhead scales differently.** Large caravans (1,000+ camels) required organized guides, paid protection, and formal toll negotiation with each local polity along the route — visible in the Hajj caravan literature (Amir al-Hajj command structure, fortified Ottoman waystations) and in Stanley's African expeditions (hongo tribute negotiations with each chief). A 6-person caravan can often avoid this overhead entirely by moving quietly and inconspicuously, but sacrifices the mutual-protection benefit that is the entire reason caravans of any size exist in dangerous territory in the first place.

**Should a 6-person caravan travel faster than a 100+ merchant caravan?** The evidence supports a qualified yes, with an important caveat:

- **On travel-day pace**: yes, meaningfully. A small group with well-matched, unladen or lightly-laden animals, no weak members to wait for, and no need to negotiate formal passage can sustain a pace close to the *individual* travel-day maximum for its slowest mode (e.g., 25-35 km/day on foot/horseback rather than the caravan-constrained 20-30 km/day).
- **On calendar-average**: the gap narrows, because even a small group still needs the same water/forage/rest cycle; a 6-person caravan does not need proportionally *more* rest, but it needs the same fixed rest cadence (Nielsen's 1-in-3-to-5-days figure) as a large one.
- **On expedition-average, in dangerous or politically fragmented territory**: the advantage can reverse. A large, well-organized, escorted caravan (Hajj-style, ~41 km/day calendar-average over 1,400+ km in 34 documented days) can out-perform its own travel-day physics because it has *purchased* safe, unmolested passage and reliable resupply through scale, while a small unescorted group risks catastrophic delay (robbery, being turned back, having to detour around hostile territory) that a fast travel-day pace cannot compensate for.

**Recommendation for the model**: give small caravans (≤10 travelers) a **travel-day speed bonus of roughly +15–25%** over the terrain baseline (reflecting points 1-2 above), but do **not** give them an expedition-average bonus of the same size in high-risk/politically fragmented terrain — instead, increase their *variance* (wider range of possible expedition-average outcomes, including bad-tail-risk delays) relative to a large, escorted caravan, which should have a narrower variance around a somewhat lower mean. This matches both the Silk Road/Saharan caravan literature (large, organized caravans as the historically dominant choice for genuinely hostile long-distance routes) and the Hajj data (large caravans achieving efficient, low-variance ~41 km/day specifically *because* of their scale-funded infrastructure and security).

---

## 6. Long-Distance Historical Evidence: Documented Journey Averages

Pulling the expedition-average figures already cited into one comparative table:

| Route / Journey | Distance | Documented duration | Expedition-average | Source basis |
|---|---|---|---|---|
| Trans-Saharan crossing (camel caravan) | ~1,500-2,000 km (variable by route) | 70-90 days | ~20-27 km/day | OER Project / World History Encyclopedia / Ibn Battuta, converging independently |
| Chang'an–Samarkand (Silk Road) | ~3,000 km | 4-6 months | ~17-25 km/day | Popular synthesis of Silk Road caravan timing, consistent with caravanserai spacing evidence |
| Full Chang'an–Mediterranean Silk Road | ~10,000 km (route distance) | 6 months to 1+ year | ~15-30 km/day (wide range reflects seasonal closure risk) | Same, upper bound reflecting winter mountain closures and political delay |
| Damascus–Mecca Hajj route (Ottoman, organized) | ~1,400+ km | 34 days | ~41 km/day | Facts and Details / Ottoman Hajj administrative sources — a comparatively *fast, well-resourced* large-caravan case |
| Ostia–Alexandria (favorable wind, Mediterranean) | ~1,600 km (sailing route) | 6-8 days | ~200-270 km/day | Casson, *Ships and Seamanship in the Ancient World* — open-water, favorable-wind case, not representative of a full round trip |
| Mozambique–India (dhow, early 19th c.) | ~2,800 km (approx., variable by exact ports) | ~1 month | Considerably below the 11 km/h (~260 km/day) sustained-sailing figure once provisioning/monsoon-timing is included | Machado, cited in Copeland-adjacent secondary synthesis |
| Stanley's 1871 march, Bagamoyo–Ujiji | ~1,127 km (700 mi) | ~234 days | ~4.8 km/day | Stanley's published diary; extreme case illustrating disease/desertion/negotiation collapse of expedition-average in tropical Africa |
| Oregon/California Trail (ox wagon) | ~3,200 km | 4-6 months | ~18-27 km/day | US National Park Service synthesis of pioneer trail records |

The spread in this table — from ~5 km/day (Stanley, catastrophic tropical-disease attrition) to ~270 km/day (a single favorable open-water Mediterranean leg) — is itself the central empirical fact a simulator needs to reproduce: **mode and terrain set the ceiling, but political/epidemiological/seasonal friction sets the actual outcome, and the gap between the two can be an order of magnitude or more.**

---

## 7. Evaluation of the Current Model

| Assumption | Verdict | Explanation |
|---|---|---|
| Rocky terrain: 6 km/day | **Too slow, but only if intended as calendar-average for a laden caravan.** | Even the harshest desert crossings (trans-Saharan, no established road, extreme heat/water stress) hold calendar-averages of 20-32 km/day. Rocky terrain alone — without also compounding altitude, jungle disease, or true trackless wilderness — is well attested at 8-12 km/day calendar-average (Section 4). 6 km/day would require adding a *second* major penalty (e.g., no established route, or combined with mountain altitude) on top of "rocky" to be historically justified; as a standalone rocky-terrain figure it understates the evidence by roughly 30-50%. |
| Rocky tropical jungle: 9 km/day | **Reasonable only under specific circumstances — and arguably still too fast for a true expedition-average, too slow for calendar-average.** | This is the terrain-biome interaction case flagged in Section 4. If "9 km/day" is meant as a calendar-average (i.e., excluding catastrophic disease/desertion attrition), it is defensible — it sits within the 8-14 km/day calendar-average band this report derives. If it is meant to already include expedition-level attrition (disease, chief-toll negotiation, animal loss), it is too fast: the Stanley case (~4.8 km/day expedition-average over 234 days) shows how far tropical expedition-average can fall below any terrain-only estimate. Recommend explicitly tagging which tier "9 km/day" represents rather than using it as a single number. |
| Paved road: 15-21 km/day | **Too slow for a genuine Roman-equivalent paved road, by a wide margin.** | This figure is close to the *rocky terrain* or *rough dirt-road* calendar-average, not paved road. The mansiones-spacing evidence (25-30 Roman miles = ~37-44 km) and the "less than 20 miles was usual, up to 30-40 km in summer" synthesis both place ordinary unaided travel on Roman paved roads at 30-40 km/day calendar-average — roughly double the current assumption — and that's before considering the cursus publicus relay system (66-103 km/day), which the current model doesn't appear to represent as a separate tier at all. If 15-21 km/day is intended to represent a *heavily laden ox-wagon convoy* rather than a general traveler, it is closer to correct (Section 2.3 gives 15-19 km/day for ox wagons specifically) — but that should be labeled as the wagon-freight case, not the generic paved-road case. |
| Sea, coastal waters: 56 km/day | **Reasonable, arguably slightly conservative.** | At 4-5 knots (the Casson baseline) sailed for a typical 10-12 daylight hours (coastal vessels generally didn't sail through the night — Section 3.3), that's roughly 7.4-9.3 km/h × 10-12h ≈ 74-112 km of possible daily distance, but coastal calendar-average must subtract time for anchoring in poor conditions, tacking around headlands, and shorter effective sailing windows in bad weather — 56 km/day is a defensible calendar-average discount from that ceiling, sitting in a sensible middle position. |
| Sea, sheltered bay: 53 km/day | **Too fast — this is the biggest outlier in the current model.** | Sheltered-bay sailing should be *slower* than open coastal sailing, not comparable to it, because enclosed water restricts tacking room, is typically shallower and more congested with other traffic, and where wind is blocked by surrounding land, oar/pole propulsion (much slower than sail) often had to supplement or replace sail entirely. There's no direct historical figure for this specific zone, but by analogy to the general principle that protected water usually means *reduced* usable wind, 53 km/day (essentially equal to the open-coastal figure) is not well justified. Recommend reducing this substantially — see Section 8 for a revised figure. |
| Sea, open sea: 67 km/day | **Too slow.** | This is the category with the strongest direct evidence, and the evidence points meaningfully higher. Casson's baseline (4-6 knots = 7.4-11 km/h) sustained over even 16-18 hours (open water allows longer daily sailing windows than coastal, per Section 3.3) already implies 118-200 km/day, and the documented Ostia-Alexandria run (200-270 km/day equivalent) and the Copeland Indian Ocean reconstruction (11 km/h sustained "on extended voyages" = ~260 km/day if run close to continuously) both sit well above 67 km/day. 67 km/day is closer to a *coastal*, not open-sea, calendar-average. Recommend roughly tripling this figure for genuine open-sea, favorable-wind conditions, while keeping a separate, lower figure for open-sea *against* prevailing wind or in poor-wind seasons (see Section 8). |

---

## 8. Recommended Values

| Terrain / Mode | Travel-day speed (km/day) | Calendar-average (km/day) | Expedition-average (km/day) | Confidence | Historical justification |
|---|---|---|---|---|---|
| Paved road (Roman-equivalent, unaided traveler) | 40-50 | 30-40 | 25-32 | High | Ramsey (JRS); mansiones spacing 25-30 Roman miles; multiple convergent tertiary syntheses |
| Paved road, relay courier (state post) | 66-103 | 55-85 | 45-70 | High (Ramsey figure); Medium (calendar/expedition extrapolation) | A.M. Ramsey, "The Speed of the Roman Imperial Post," *JRS* — treat 800 km/day claim as a misapplied outlier, not a sustained rate |
| Maintained dirt/trade road (foot, pack animal, caravan) | 25-35 | 20-30 | 15-24 | High | Caravanserai spacing 30-40 km; Spufford's carrier data 30-40 km/day; Britannica caravan synthesis |
| Ox-drawn wagon | 16-20 | 14-19 | 10-16 | Very high | Convergent Roman/medieval/Australian/American-frontier data, all in a tight 15-20 km/day band |
| Horse/mule-drawn wagon | 30-40 | 25-35 | 18-28 | High | Encyclopedia.com; Spufford Dijon-Paris convoy (50 km/day peak, good road) |
| Pack mule/donkey train (non-mountain) | 40-56 | 30-45 | 22-35 | Medium-high | 19th-c. Californian/Mexican arriero data; extrapolated calendar/expedition tiers |
| Camel caravan (desert-adapted) | 40-48 | 28-36 | 20-28 | High | World History Encyclopedia (48 km/day capability); OER/Ibn Battuta (32 km/day, 70-90 day Saharan crossings) |
| Foot traveler (unburdened/lightly burdened) | 25-35 | 20-27 | 15-22 | High | Mortimer; German medieval synthesis; Rigaud retinue (33 km/day); Ramon Llull Foundation |
| Foot traveler (burdened porter/caravan member) | 15-22 | 12-18 | 8-14 | Medium-high | Silk Road "pace set by slowest member" principle; Stanley expedition data as lower bound |
| Military column, routine march (with baggage train) | 22-30 | 16-19 (train-limited) | 12-16 | High | Vegetius; modern biomechanical reconstruction (Whip 1998); ox-cart baggage-train constraint |
| Military column, forced march (short-term only) | 45-75 | n/a (unsustainable >2-3 days) | n/a | Medium | Multiple ancient/modern sources converge 45-75 km/day; explicitly not sustainable, model as a temporary multiplier not a base rate |
| Courier/messenger, horse relay (steppe-optimized, e.g. Yam) | 200-300 (up to 400 exceptional) | 150-250 | 100-180 | Medium-high | Wikipedia/Weatherford/Lane synthesis on Yam; Pony Express as independent cross-era confirmation (~316 km/day over 10 days) |
| River transport, downstream | 30-45 (moderate current) | 25-38 | 18-30 | Medium | Drennan 2:1 downstream:upstream ratio; Kunow/Rahn Roman river-freight data |
| River transport, upstream (towed/poled) | 15-22 | 12-18 | 8-14 | Medium | Drennan ethnoarchaeological reconstruction (20 km/day upstream baseline) |
| **Rocky terrain** | 12-18 | 8-12 | 6-9 | Medium-high | Interpolated from footing-risk principle + caravan/pack-animal data; no single dedicated academic source, but consistent with desert/mountain bracketing evidence |
| **Mountain pass (open season)** | 15-25 | 12-20 | 8-15 in-season; 0 closed season | Medium | Simplon pass pack-stage data (~30 km, favorable case); adjusted down for steeper/higher passes |
| **Temperate forest (trail)** | 20-28 | 16-22 | 12-18 | Medium | Interpolated near dirt-road baseline with modest trail-quality discount |
| **Tropical jungle (trail)** | 15-22 | 8-14 | 4-9 | Medium (travel-day); High (expedition-average, via Stanley) | Stanley 1871 expedition (~4.8 km/day expedition-average); terrain/biome interaction principle (Section 4) |
| **Steppe (foot/wagon)** | 25-35 | 20-30 | 15-25 | Medium | Open, low-friction terrain; analogized from dirt-road figures with a modest bonus |
| **Steppe (mounted relay)** | 50-300 | 40-250 | 30-180 | Medium-high (ceiling); Medium (typical case) | Mongol Yam data; wide range reflects relay-system investment level |
| **Desert (camel caravan)** | 30-48 | 20-32 | 12-22 | High | Trans-Saharan and Hajj route data, converging strongly |
| **Marsh** | 10-15 | 6-10 | 4-8 | **Low** | No dedicated historical source; interpolated between rocky terrain and jungle; flag for future research |
| **Coastal lowlands (overland)** | 25-35 | 20-28 | 15-22 | Low-medium | Interpolated from dirt-road/steppe baseline with an estuary/tidal-crossing discount; no dedicated source |
| **Sea — sheltered bay** | 30-50 (oar/sail hybrid) | 25-40 | 18-30 | Low | No direct source; revised down from the current model's 53 km/day on the general principle that enclosed water reduces usable wind and daily sailing window |
| **Sea — coastal waters** | 60-90 | 45-65 | 30-50 | Medium-high | Casson 4-5 knots baseline over a daylight-limited window, discounted for headland/anchoring friction; roughly confirms the current model's 56 km/day as reasonable |
| **Sea — open sea, favorable wind** | 130-220 | 100-180 | 60-130 (accounting for *Mare Clausum*/monsoon-lock seasonal closure) | High (travel-day); Medium (expedition-average) | Casson's Ostia-Alexandria case (200-270 km/day); Copeland's 11 km/h Indian Ocean reconstruction (~260 km/day sustained) |
| **Sea — open sea, against prevailing wind/season** | 30-60 | 20-45 | 10-30 | Medium | Inferred from the documented multi-week-to-monthlong reverse-direction Mediterranean legs and monsoon-locked dhow return voyages |

---

## 9. Applying the Model to the Sample Journey

**Journey parameters**: 8,423 km total; 45% by sea on a dhow (3,790 km); 55% overland (4,633 km); 6-person merchant expedition (not military); route and specific terrain mix unspecified, so the overland leg is modeled as a realistic *mixed* trade-road/desert/rocky/mountain composite rather than a single terrain, since no real 4,600+ km overland trade route is uniform terrain (compare the Silk Road's mix of steppe, desert margin, and mountain passes).

### 9.1 Assumptions made explicit

- **Sea leg**: modeled as dhow open-sea/coastal mixed sailing, since a 3,790 km sea leg on a monsoon-dependent route realistically includes both open-water crossing and coastal approach/departure at each end. Weighted as 70% open-sea-favorable, 30% coastal, and includes one full monsoon-window wait (assume up to ~4 months lost if the expedition's departure doesn't align with the correct monsoon leg — this is the single largest swing variable in the whole estimate, per Section 3.2).
- **Overland leg**: modeled as a weighted composite — 40% maintained trade road, 25% desert, 20% steppe, 10% rocky/hills, 5% mountain pass — representative of a long Old World overland trade corridor (Silk-Road-like), since the journey's scale (4,633 km overland) is consistent with a trans-continental route rather than a single-terrain regional trip.
- **Caravan size (6 people)**: per Section 5, apply the small-caravan travel-day bonus (+20%) to overland travel-day speed, but do *not* extend that bonus to expedition-average in the desert/steppe/mountain portions, since a 6-person merchant party lacks the security infrastructure of a large escorted caravan and is modeled with wider variance instead.
- **Merchant, not military**: no forced-march multiplier is applied at any point; the party rests on a normal caravan/traveler cadence (Section 5's 1-rest-day-per-3-to-5-travel-days figure is applied to the overland leg).

### 9.2 Estimate 1 — Continuous travel time (travel-day speed only, no rest, no delay)

This is a pure physics-only floor: distance ÷ travel-day speed, with the small-caravan bonus applied overland.

- Overland (weighted travel-day speed, +20% caravan bonus): trade road 25-35→30-42, desert 30-48→36-58, steppe 25-35→30-42, rocky 12-18→14-22, mountain 15-25→18-30 km/day. Weighted average ≈ **32 km/day** → 4,633 km ÷ 32 ≈ **145 days**.
- Sea (weighted travel-day speed): 70% open-sea-favorable (130-220, midpoint 175) + 30% coastal (60-90, midpoint 75) ≈ **145 km/day** → 3,790 km ÷ 145 ≈ **26 days**.
- **Total continuous travel time: ≈ 171 days (~5.6 months).**

This figure is not realistic as a planning estimate — no historical expedition of this length sustained pure travel-day pace for months without a single rest, resupply, or weather day — but it is the correct lower bound the other estimates should be checked against.

### 9.3 Estimate 2 — Realistic historical travel time (calendar-average, normal rest cadence, no exceptional delay)

- Overland (weighted calendar-average, small caravan): trade road 20-30, desert 20-32, steppe 20-30, rocky 8-12, mountain 12-20 (in-season). Weighted average ≈ **22 km/day** → 4,633 ÷ 22 ≈ **211 days**.
- Sea (weighted calendar-average): 70% open-sea-favorable (100-180, midpoint 140) + 30% coastal (45-65, midpoint 55) ≈ **115 km/day** → 3,790 ÷ 115 ≈ **33 days**.
- **Total realistic historical travel time: ≈ 244 days (~8 months).**

This aligns well with the documented comparanda in Section 6 — it sits between the Hajj route's efficient ~41 km/day (a much shorter, better-resourced, state-organized route) and the Silk-Road-scale journeys' 15-30 km/day expedition-averages, which is exactly where an unescorted 6-person merchant party on a Silk-Road-scale route should land.

### 9.4 Estimate 3 — Conservative (slow) travel time

This applies the low end of each calendar-average range and adds a modest standing allowance for the ordinary friction (weather days, minor illness, small negotiation delays, a single bad water source or diverted crossing) that a realistic multi-month expedition always encounters even in a "normal" year, without invoking a specific catastrophe.

- Overland: low-end calendar-average weighted ≈ **17 km/day** → 4,633 ÷ 17 ≈ **272 days**.
- Sea: low-end calendar-average weighted ≈ **80 km/day**, plus a partial monsoon-timing penalty (assume the expedition must wait ~6 weeks at one port for the correct wind rather than the full 4-month worst case) → (3,790 ÷ 80 ≈ 47 days) + 42 days waiting ≈ **89 days**.
- **Total conservative travel time: ≈ 361 days (~11.9 months).**

### 9.5 Estimate 4 — Worst-case with seasonal delays

This applies expedition-average speeds directly (which already embed *typical* delay) and then adds the specific large, discrete delay categories documented in Sections 3.2 and 6: a full missed-monsoon wait, and a Stanley-style tropical/desert attrition scenario is *not* applied here (that was an extreme case with catastrophic disease mortality; a merchant party crossing desert/steppe/mountain terrain, not tropical rainforest, faces real but less extreme seasonal risk — primarily mountain-pass winter closure and desert raiding-season avoidance).

- Overland (expedition-average, weighted): trade road 15-24, desert 12-22, steppe 15-25, rocky 6-9, mountain 8-15 in-season/0 closed. Weighted average, and assuming the mountain-pass segment (5% of route, ~232 km) must wait out one full closed season (add 120 days) rather than simply moving slower: base weighted expedition-average excluding the mountain segment ≈ **14 km/day** over 4,401 km ≈ 314 days, **plus 120 days mountain-closure wait, plus 232 km crossed at the in-season expedition-average of ~10 km/day ≈ 23 days** → overland subtotal ≈ **457 days**.
- Sea (expedition-average, weighted): 70% open-sea 60-130 (midpoint 95) + 30% coastal 30-50 (midpoint 40) ≈ **79 km/day** → 3,790 ÷ 79 ≈ 48 days, **plus a full missed-monsoon wait of up to 120 days** (worst realistic case, per Section 3.2 — missing the return/onward window costs the better part of a season) ≈ **168 days**.
- **Total worst-case travel time: ≈ 625 days (~20.5 months, just over 1.7 years).**

This is consistent with the Section 6 comparanda's upper bound (the full Chang'an-Mediterranean Silk Road route, at similar scale, is documented at 6 months to over a year even in ordinary operation, and this journey's worst-case explicitly stacks a monsoon miss *and* a mountain-season closure, which would not both occur in most individual journeys — it represents a genuine tail-risk scenario, not a typical bad year).

### 9.6 Summary table

| Scenario | Overland days | Sea days | Total days | Total (months) |
|---|---|---|---|---|
| 1. Continuous travel (physics floor) | 145 | 26 | 171 | ~5.6 |
| 2. Realistic historical | 211 | 33 | 244 | ~8.0 |
| 3. Conservative (slow) | 272 | 89 | 361 | ~11.9 |
| 4. Worst-case (seasonal delays) | 457 | 168 | 625 | ~20.5 |

---

## 10. Feeding Two Complementary Systems

The tiered structure of this report maps directly onto the two systems described:

**Movement Model (active-travel rate, for simulation ticks)** should use the **travel-day speed** column throughout Section 8, modified by the caravan-size rule in Section 5 (+15-25% for small unescorted groups on travel-day pace only) and the terrain-interaction principle in Section 4 (biome penalties apply to rest/resupply frequency, not to the moment-to-moment movement rate). This is the number that should drive per-tick or per-hour position updates in the simulator, since it represents genuine physical movement capability uncomplicated by scheduling decisions.

**World Simulation Model (expected rests, weather, resupply, border crossings, seasonal delays)** should be built as a *modifier layer* on top of the Movement Model rather than as an independent speed table, using:
- A **rest-day cadence** (Section 5's 1-in-3-to-5 days baseline, adjustable by caravan size and terrain — jungle and desert should skew toward more frequent forced rests, paved road/relay conditions toward less);
- A **seasonal gate** function per terrain (binary open/closed for mountain passes and monsoon-locked sea routes, rather than a continuous speed penalty — Section 4 and 3.2 both show these are threshold effects, not gradual slowdowns);
- A **political/toll friction** term that scales with caravan size and route (large caravans pay more in aggregate toll/negotiation time but gain security-driven predictability; small caravans pay less but carry higher delay variance) per Section 5;
- A **disease/attrition attrition multiplier** specific to tropical/jungle biomes and long expedition duration, calibrated against the Stanley case as an extreme upper bound and the Section 8 "expedition-average" jungle figures as a typical case.

This separation is what allows the four estimates in Section 9 to be produced from the same underlying terrain table: Estimate 1 is pure Movement Model output; Estimates 2-4 are the Movement Model composed with successively more pessimistic World Simulation Model modifier settings.

---

## 11. Sources

**Academic / peer-reviewed / primary-adjacent sources** (prioritized per your instructions):

- Casson, Lionel. *Ships and Seamanship in the Ancient World.* Princeton University Press / Johns Hopkins University Press. The standard academic reference for ancient Mediterranean sailing speed and ship technology, cited throughout Section 3.1.
- Ramsey, A.M. "The Speed of the Roman Imperial Post." *Journal of Roman Studies* (1925). The primary scholarly reconstruction of cursus publicus speed from mansiones-spacing evidence; the authoritative figure for Section 2.5.
- Carreras, C. & de Soto, P. "The Roman Transport Network: A Precedent for the Integration of the European Mobility." *Historical Methods* 46(3), 2013. GIS-based academic reconstruction; flagged as the source of the outlier 800 km/day claim, treated with appropriate skepticism.
- Copeland, [author]. "Riding the monsoon: Geography and Iron Age trade in the Indian Ocean." *The Economic History Review*, 2026 (Wiley). Wind-speed-data-based reconstruction using modern sea-trial results from a historical-vessel replica — the strongest methodological source for Indian Ocean sailing speed (Section 3.2).
- Spufford, Peter. Work on late-medieval merchant travel and carrier speeds (cited via secondary discussion of his archival research on continental European merchants, including the Dijon-Paris convoy and Archbishop Rigaud data) — Section 2.4 and 2.6.
- Tripcevich, Nicholas. PhD dissertation research on Andean caravan archaeology, drawing on Nielsen (2000, 2001) and Earle (2001) ethnoarchaeological fieldwork on llama caravans — Section 5's rest-day-cadence evidence.
- Drennan, R. Cargo-dugout-boat river transport reconstruction (Mesoamerica/South America), cited via *Internet Archaeology* 36 — Section 2.9.
- Rahn / Kunow. Roman river-transport cost and load-capacity data from Germania, cited via *Internet Archaeology* 36 — Section 2.9.
- Bishara, Fahad Ahmad. *Monsoon Voyagers: An Indian Ocean History.* University of California Press, 2025. Primary-source-based (ship's logbook) academic history of a specific documented dhow voyage; consulted for the character of dhow-trade seasonal structure (Section 3.2).
- Mortimer, Ian. *The Time Traveler's Guide to Medieval England.* Cited for foot-travel and horseback speed figures (Section 2.6), a well-regarded work of accessible academic social history built from primary account-book research.

**Primary historical testimony** (via secondary synthesis):
- Vegetius, *De Re Militari* — Roman legionary march-rate figures (Section 2.7).
- Ibn Battuta's travel accounts — trans-Saharan caravan size and duration (Section 2.1).
- Marco Polo's account of the Mongol Yam — courier relay figures (Section 2.8), used cautiously given known tendencies toward exaggeration in his account, cross-checked against modern historian syntheses (Weatherford, Lane).
- Procopius's description of the cursus publicus, as analyzed by Ramsey.

**Well-sourced tertiary/reference syntheses** (used for corroboration and range-triangulation, not as primary evidence where academic sources were available):
- Encyclopaedia Britannica's caravan and Roman-transport articles.
- World History Encyclopedia articles on trans-Saharan camel caravans and Roman shipbuilding.
- Wikipedia articles on the Yam relay system, ox-wagons, and river/chain-boat navigation, cross-checked against their own cited academic sources (Weatherford's *Genghis Khan and the Making of the Modern World*, Lane's Mongol-history scholarship) rather than taken at face value.
- UNESCO Silk Road documentation on caravanserai spacing.
- Historical archaeology/administrative-history literature on the Ottoman Hajj route (Damascus road), including the *History of Religions* journal article "In an Ottoman Holy Land."
- U.S. National Park Service historical documentation on the Oregon/California Trail.
- Stanley, Henry Morton. Published expedition diaries (*How I Found Livingstone*, and accounts of the 1874-77 trans-Africa expedition), used for the tropical-jungle expedition-average case study, with awareness that Stanley's own accounts have documented reliability and self-dramatization issues on some points — the raw distance/duration figures used here are corroborated by independent modern historical treatment (Jeal's biography, cited via secondary synthesis) rather than Stanley's own framing.

**Notes on evidentiary confidence**: figures for paved roads, ox-wagons, foot travel, camel caravans, Mediterranean and Indian Ocean sailing, and Mongol/Roman couriers rest on multiple independent, mutually corroborating sources (academic and well-documented primary) and should be treated as high-confidence. Figures for marsh, coastal-lowland overland travel, and sheltered-bay sailing have no dedicated historical study identified in this research and are interpolated from adjacent, better-evidenced categories using consistent physical/logistical principles — these are flagged throughout as lower-confidence and worth targeted follow-up research (e.g., specifically searching for Fenland/Pontine-Marsh-equivalent medieval route data, or harbor-pilotage speed records) if higher precision is needed for the simulator.

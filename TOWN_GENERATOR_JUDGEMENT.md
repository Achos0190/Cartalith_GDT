# Town generator judgement: Cartalith's towns against real medieval and early-modern towns

Written 2026-11-15 by a judging agent at the owner's request (*"Go find some middle age and old city plans and how they grew and academic information. Then let fable check and be the judge on how things should be modified."*). The rulings here are **recommendations for the owner, not owner rulings**. This is not a scope and not a status record; nothing was built and no code was changed.

**What I opened.**

- Research: `MEDIEVAL_TOWN_GROWTH_RESEARCH.md`, all of it.
- Code, at the symbols: `generate::generate_pass`, `growth::grow`, `growth::grow_far_bank`, `rules::DEFAULT_RULES`, `blocks::build_parcels`, `plaza::build_plaza` and `riverbank_square`, `districts::assign_districts`, `fortify::built_mass_hull` and `build_wall_from_mass`, `wallhug::hug_wall` and `outline`, `riverbank.rs`'s constants, and `urban_adapter.rs`'s `GenOpts` fill.
- Rulings EW item 1, EX, EZ item 1 and FB, and the four `STATUS.md` river-town entries.
- Frames: seven of the ten in `cartalith-native/docs/evidence/both-banks-fix-2026-11-15/viewer/` (483920 #6, #11, #15, #18, #70 and 2048 #0; 12345 #0 and #2).

**Fresh numbers.** I ran `town_river_census::both_banks_audit` once at `origin/main` (`25e59c03`): 384 cells, seeds 483920, 912345 and 246371, villages on, release build, with `TOWN_RIVER_PLOT` dumping every layout. A scratch script then measured the 631 dumped layouts that have at least 20 buildings. In this document, "measured" means that run.

- The script is not in the tree. It and the audit log are in `C:/cw/judge_scripts/` (`an.py`, `sum.py`, `x.py`, `both_banks_audit_384.log`). The worktree, the build and the JSON dumps were removed.
- It was one run at one grid size. Almost every layout is a river site (that is what the audit visits), so landlocked and coastal towns are not covered.
- "Settled area" is my own measure: the 20 m cells within one cell of a building's centre. It is tighter than the settled area in the research's main source, which includes gardens and vacant ground. Read the area comparisons as indicative only.

## 1. Verdict summary

1. **The street network is the generator's strongest part and should be left alone.** Measured: 62 to 70 % three-way junctions, 15 % four-way, bearing entropy 0.97 of the maximum in towns of 5 000 and up. That is the organic pattern the research describes.
2. **The river-town rework of 2026-11-15 already matches the bridge-first pattern.** The market is at the bridge foot, there is usually one crossing, and the far bank is smaller and grown after the near bank. No change is recommended to that logic.
3. **The hugged wall is the largest fault in the town frames.** It traces every bay of the built area in a scalloped line of about 390 vertices, and is 17 % longer (median) than its own convex hull. A wall was paid for by the metre, so this contradicts the reason Ruling EW item 1 gave for hugging.
4. **Villages are the largest fault by count.** They are 596 of the 631 layouts measured. A village of about 90 houses carries about 2.9 km of street (32 m per house, against 8 m in a town), and its green stands more than 150 m from its market point in 431 of 596.
5. **Plots are house-sized, not burgage plots.** Median 7.8 m frontage by 10.6 m depth, and none deeper than 60 m. The research's sourced plots are 32 to 140 m deep. This is why blocks read as small triangles packed with roofs.
6. **Towns are about twice as dense as the research's scaling law predicts.** The wall encloses almost no open ground (wall area is 1.01 times the built area inside it). Density does rise with size, as the law says it should.
7. **The market square is one size (about 0.2 ha) for every settlement**, from a hamlet of 400 to a city of 20 000. The research has no sourced ratio, so the number has to be a labelled judgement.
8. **The river inside a town is a ruler-straight band with sharp elbows.** This is the first thing the eye catches in every frame. The research does not cover it, so I record it as a look defect, not a research finding.
9. **Not ruled on, because the research is too weak:** street widths, market area per head, founded second towns, planted grid towns, and marsh left unwalled.
10. **Every recommended change is confined to the paths already switched under `DECISIONS.md` §7p** (`urban_wall_hug`, `urban_riverbank`) or needs one new switch. None needs a golden to move.

## 2. Pattern-by-pattern assessment

Research strength: **strong** = a peer-reviewed or primary source read directly; **medium** = sourced, but through summaries or encyclopaedia pages; **weak** = tourism or forum pages, or the researcher's own inference.

| # | Pattern (research rule) | Research strength | Generator today, with evidence | Verdict |
|---|---|---|---|---|
| 1 | Settled area ≈ 0.12·N^0.714 ha; density rises with size (rule 1) | Strong for the law (Cesaretti et al., 173 cities). Villages are outside its sample | Intended density is a constant 150 persons/ha (`generate_pass`: `target_len = pop × 2.1`, `max_rf` from `sqrt(pop × 21)`). Measured density: 135/ha in villages, 228/ha at 5 000 to 12 000, 235/ha above. Settled area is 0.51 to 0.58 of the law in towns of 5 000 and up, 0.35 in villages (my tighter measure) | **Partly matches.** The direction is right; towns are too compact by a factor between about 1.3 and 2 |
| 2 | Walls are not the settled area: vacant ground inside, suburbs outside (rule 2; Bristol 55 ha walled, 130 ha settled) | Strong (same paper) | Wall area ÷ built area inside it: median 1.01 (p90 1.03) in towns of 5 000 and up. 8 to 15 % of buildings stand outside the wall | **Half matches.** Suburbs exist; intramural open ground does not. This follows from Ruling EW item 1 |
| 3 | Wall length is about the circle-equivalent of the settled area (rule 3) | Weak (researcher's arithmetic, two spot checks) | Wall length ÷ circle-equivalent of its own area: median 1.35 to 1.42, p90 1.65. Wall ÷ its own convex hull: median 1.17, p90 1.30. Ring vertices: median 392 (`wallhug::outline`, traced from a 4 m grid) | **Violates.** The circuit is far more crooked than any wall the research cites, and longer than a taut one. Frames: 483920 #6, #18; 12345 #2 |
| 4 | Towns outgrow their walls; later rings are not concentric offsets (rules 4, 5) | Medium (Paris, Florence, encyclopaedia pages) | The machinery exists: `wall_generations: true` in the adapter, `SettlementRules` (up to 3 rings, 120-year gap, 80 % full and 15 % extramural before a new ring), `supersede_wall` lays the old line as a `ringroad`. I did not measure how many towns reach a second ring, and no frame I opened shows one | **Matches in code, unverified in output** |
| 5 | A river is one side of the wall, or crossed with a chain, boom towers or a water gate (rule 11) | Medium (York, Chester, Cologne, Norwich pages) | `build_wall_from_mass`: a land arc plus a bank-following water wall, a spur dipped into the water at each end, water gates at the harbour and where the river passes the wall. Audit: 11 of 32 walled river towns' walls cross the river. In frame 483920 #18 the river passes through the circuit with nothing drawn at either crossing | **Matches in the model; the crossings are not visible** |
| 6 | Marsh left unwalled (rule 11, York) | Medium, one case | No marsh input reaches the layout | **Ignored. No ruling**: one example, low visual value |
| 7 | The two banks are not symmetric; the commercial bank is first and bigger (rules 6, 8) | Medium | `riverbank::second_pass` grows the far bank only when the town houses under target, after the near bank, capped at the near bank's street length (`FAR_BANK_MAX_RATIO`). Audit: 13 of 33 towns of 1 500 and up are on both banks | **Matches** |
| 8 | A bridgehead suburb is a strip along the approach road (rule 9, Southwark) | Medium (Historic England, council archaeology page) | The reference's own rule confines far-bank growth to 190 m of the bridge at 35 % probability (`bridgehead_distance`, `bridgehead_probability`), and extramural growth must start within 90 m of a primary (`grow`). `grow_far_bank` then grows a quarter from far-bank origins. Frames 483920 #11 and #6 show a compact quarter with its own lanes, not a strip | **Partly matches.** Small far banks should read as a strip; large ones may be a quarter |
| 9 | A far-bank district has a single link, or a few roads from the bridge head (rule 10) | Medium (York) | Edges crossing the river: median 1 in towns of 12 000 and up, p90 2 (measured). `lay_town_bridge` lays one | **Matches** |
| 10 | The far bank can be a founded second town with its own market (rule 7, Prague) | Medium, one clear case | One market anchor per layout; no second foundation | **Ignored.** Optional variety, high cost; an owner question, not a ruling |
| 11 | Markets begin at a landing, bridgehead or junction (rule 12) | Medium | The real-water market is pinned at the bridge foot: 3 to 11 m from the bank (measured median). `plaza::riverbank_square` puts a chartered town's square about 49 m back from the bank beside it. Villages keep `build_plaza`'s square at the middle of the nearest primary: 284 m from the bank (median), and over 150 m from the market point in 431 of 596 | **Matches for towns. Violates for villages** |
| 12 | Market streets taper; a spindle is as common as a square (rule 13) | Medium (Victoria County History panel) | Always a rectangle of 55 to 80 m by 26 to 40 m (`build_plaza`) | **Ignored** |
| 13 | The most integrated streets carry the market (rule 21, Craane on Utrecht) | Strong for Utrecht, one town | Betweenness percentile of the best node within 40 m of the market: median 96 to 98 in towns of 5 000 and up, p10 15 to 55 (measured, sampled). `assign_districts` zones by distance from the market, not by the network, though its provenance text says "highest network access" | **Mostly matches**; a tail of towns has a poorly connected market |
| 14 | Organic towns: high bearing entropy, few four-way junctions (rule 22) | Weak (inference from Boeing; no medieval benchmark) | Entropy 0.97 of maximum; three-way 69 %, four-way 15 %, dead ends 14 to 16 % in towns of 5 000 and up | **Matches** |
| 15 | Burgage plots: narrow frontage, depth from 1:2 to 1:10 (rule 15) | Medium (Freiburg and Burford through summaries; others local history) | `build_parcels`: frontage log-normal about 11 m clamped 4.5 to 16 m; target depth 22 m (core) or 30 m. Measured on 142 000 dumped lots: frontage median 7.8 m, depth median 10.6 to 11.3 m, depth ÷ frontage median 1.4, **none over 60 m** | **Violates** |
| 16 | A plot module per growth phase, which persists (rules 16, 18) | Medium (Alnwick, one town) | One module for the whole town; only core against fringe depth differs. Subdivision by age exists (`subdivision_cap`) | **Ignored** |
| 17 | Backland fills over time (rule 17) | Medium (Edinburgh) | Rear outbuildings, wings and rear ranges exist (`districts.rs` provenance M-BLD-3, M-BLD-6), but a lot 11 m deep has no backland to fill | **Blocked by row 15** |
| 18 | Waterfront plots run street to water, with a private quay (rule 19) | Weak to medium (King's Lynn through a summary; the layout is the researcher's inference) | `riverbank::build_embankment` lays a 4 m quay wherever streets come within 46 m; `BANK_CLASS` stretches of 90 to 220 m let houses reach the water (Ruling EZ item 1). Audit: 2.4 % of houses within one street width of the bank; 99 of 617 river towns have every house over two widths away | **Partly matches.** The forms exist; the waterfront is thin |
| 19 | Fringe belts at old edges (rule 23) | Weak (mechanism is inference) | The superseded wall becomes a ring road; no belt of large irregular plots | **Ignored. No ruling** until row 4 is measured |
| 20 | Street widths by class | **None** (research §4.1 found no source) | Primary 6 to 7 m, street 3.4 to 5.4 m, lane 2.6 m, quay 4 to 7 m (measured from the dumps; set in the `add_street` calls) | **No ruling.** The values sit inside the research's own unsourced priors |
| 21 | Market area against population | **None** (research §4.4) | Constant about 0.2 ha: 7 % of a village's built area, 0.28 % of a large town's (measured) | **No sourced ruling.** A constant is still wrong at both ends; see change 6 |
| 22 | Planted towns: grid and a large square (rule 14, Kraków, Monpazier) | Weak to medium (popular sources) | Culture profiles are `medieval` (organic) and `venus` (radial) only | **Ignored.** Owner question |

## 3. Recommended changes, ranked

Ranked by visual impact against cost. "Cost" is my estimate of builder effort, not a measurement. Each change lists its §7p switch; none moves a golden, because the goldens and the FH-R2 and G1 harnesses lay towns out with these switches off.

### 1. Draw the wall taut: straight runs between towers, not a traced outline

- **What.** Keep the hug, but stop tracing the built area's outline. After `wallhug::outline` finds the mass, fit the shortest circuit that encloses it: straight curtain runs of roughly 40 to 100 m between towers. Allow a re-entrant corner only where it removes a large unbuilt patch. Apply the same to the river front.
- **Why.** Ruling EW item 1 gave the reason "walls were expensive". Expense is per metre of wall. The current circuit spends more wall to enclose less ground.
- **Evidence.** 31 walled towns of 1 500 and up:
  - wall length ÷ convex-hull length: median 1.17, p90 1.30;
  - hull area ÷ wall area: median 1.19;
  - ring vertices: median 392;
  - wall ÷ circle-equivalent of its own area: 1.35 to 1.42.
  
  The research's one sourced circuit (Paris, 5.4 km round 273 ha) is shorter than its circle-equivalent. Frames 483920 #6 and #18 and 12345 #2 show the scalloped line.
- **Expected visual effect.** A wall that reads as built: straight stretches, a tower at each angle, a compact shape. About a fifth more ground inside it, which reads as yards and gardens, not as the 50 to 100 m empty band Ruling EW removed.
- **Cost.** Small to medium. One stage in `wallhug.rs` after `outline`; gates, wall lots and the citadel already re-derive from whatever ring stays.
- **Risk.** It re-opens the "unbuilt patch inside the wall" measure the 2026-11-15 rework closed (median 6 ares, none over 0.5 ha at 384). Wall-backed lots and the faubourg move with the ring.
- **Switch.** Rides `urban_wall_hug`.
- **Acceptance bar** (`both_banks_audit`, same seeds, 384 and 2 048):
  - wall ÷ hull length: median at most 1.06, p90 at most 1.15;
  - ring vertices at most wall length ÷ 35 m;
  - no wall with an unbuilt patch over 1 ha;
  - market squares outside the wall no worse than today (4 of 33 at 384);
  - every lot still at least `MIN_MARGIN_M` inside.

### 2. Give villages a village plan: one street, tofts, the green at the landing

- **What.** Three changes for settlements under `PLAZA_MARKET_POP` on the riverbank path:
  - grow the street budget from the head count, without the 1 600 m floor;
  - grow along the through road first, so houses front one or two streets;
  - put the green at the market point (the landing or junction), not at the middle of the nearest primary.
- **Why.** Villages are almost every settlement on the map, and they are laid out as small towns with most of the houses missing.
- **Evidence.**
  - Median village: 2 907 m of street for 91 buildings, 32 m per building (towns of 5 000 and up: 8 m).
  - The green is over 150 m from the market point in 431 of 596.
  - 15 % of village lots have no building (audit).
  - Frames 483920 #15 and #70 show streets fronting nothing and empty blocks.
- **Research support is thin.** The research is about towns. The nearest sourced rules are the strip along an approach road (rule 9) and the market at a landing or junction (rule 12). I rule on this from the frames and the numbers, not from the literature.
- **Expected visual effect.** A row of houses along the road and the bank, with a green where the road meets the water, and no web of empty lanes.
- **Cost.** Medium. `riverbank::fit_village` already lays a village out again from a scaled street budget; this changes what it scales and where the green goes.
- **Risk.** A recorded attempt to move the green left 24 more villages under 80 % of target (`build_plaza`'s comment), so the green and the street budget have to be changed together. 108 villages are already under 80 %.
- **Switch.** Rides `urban_riverbank`.
- **Acceptance bar:**
  - street per building: median at most 15 m;
  - green within 60 m of the market point in at least 90 % of villages;
  - lots with no building at most 5 %;
  - villages under 80 % of target no more than today (108 at 384);
  - housed median within 95 to 115 %.

### 3. Smooth the river through the town box

- **What.** The town's river is the map's traced polyline, resampled every 8 m (`riverbank::fine_river_path`) but not smoothed. Its bends are map-cell corners, and it is drawn at a constant width. Round the corners with the same curve the map draws, so the town still sits on the map's river.
- **Why.** It is the most conspicuous artificial element in every frame: a ruler-straight band with 90° and 45° elbows (483920 #6, #18; 12345 #2).
- **Evidence.** The frames. **The research does not cover river shape**, so this is a look ruling, not a research finding.
- **Expected visual effect.** A river that bends; quays and bank houses that follow a curve.
- **Cost.** Small in the generator. The real cost is agreeing one curve with the map's river drawing (`RIVER_REALISM_SCOPE.md`).
- **Risk.** If the town smooths and the map does not, the town drifts off its river, which is the defect Ruling EX fixed. Every bank measure (setback, embankment, buildings in water) has to be re-run.
- **Switch.** Rides `urban_riverbank`.
- **Acceptance bar:**
  - no centreline turn sharper than 30° at any resampled vertex inside the box;
  - town centreline within half a channel width of the map's drawn river everywhere in the box;
  - buildings in water stay 0;
  - the bank-gap figures no worse than today.

### 4. Deep plots and backland, as a measured prototype first

- **What.** A second rules preset for the organic plan:
  - streets further apart (about two plot depths, 60 to 70 m, against today's 24 m minimum spacing and 56 m median segment);
  - narrower frontages, with the house on the street and a yard or garden behind;
  - rear buildings added with age (the burgage cycle the engine already half-models);
  - the street budget per head reduced to match.
  
  Prototype it through `Rules` (the reference already carries `MARKET_TOWN_RULES` as a preset) and measure before committing to a build.
- **Why.** This is the largest gap between the research and the generator, and it is also what makes towns too dense (row 1) and gives the wall nothing open to enclose (row 2).
- **Evidence.**
  - Measured lots: 7.8 m by 10.6 m, depth ÷ frontage 1.4, none over 60 m.
  - Sourced plots: Freiburg 16 by 32 m, Burford 7.5 by 110 to 140 m, Alresford 10 by 100 m.
  - Street density is about 400 m per hectare in large towns.
  - Buildings cover 26 % of the built area.
- **Expected visual effect.** Long blocks with a row of roofs on each street and green or yard strips behind, in place of small triangles filled edge to edge.
- **Cost.** High. It moves every housing number and touches `grow`'s street rules, `build_parcels` and `build_buildings`.
- **Risk.** Under-housing. Each lot houses one household of 5.2, so fewer, deeper lots need more ground. The layout box is 1 700 by 1 250 m (212 ha); the law gives 141 ha at 20 000, so the largest towns fit, with little room to spare beside a river.
- **Switch.** A new one (for example `urban_burgage`): on in `params::defaults`, off in `WorldParams::defaults` and for a save without the key.
- **Acceptance bar** (prototype first, then the build):
  - lot depth ÷ frontage: median at least 3, with at least 10 % of lots over 40 m deep;
  - settled area within 0.65 to 1.0 of 0.12·N^0.714 for towns of 5 000 and up, by the same 20 m-cell measure;
  - housed median within 90 to 110 % of target;
  - three-way junction share and bearing entropy not below today's.

### 5. A fuller waterfront: plots from a back street to the water

- **What.** Along the built reach of the bank, lay a street 35 to 50 m back from the water and plat narrow plots from it to the bank, with the warehouse or house at the water end. Alternate these with open quay, as Ruling EZ item 1 allows.
- **Why.** The river is why the town is there, and today the town barely touches it.
- **Evidence.** Audit: 2.4 % of houses within one street width of the bank, 6.4 % within two; 99 of 617 river towns have every house over two widths away. Research rule 19 (King's Lynn) supports the form, with the caveat that the plot layout is the researcher's inference from a summary.
- **Expected visual effect.** A continuous built river front with gaps for quays, instead of a quay street with the town standing back.
- **Cost.** Medium. Easier after change 4, which supplies deep plots.
- **Risk.** The "house over the widest street" measure already regressed in the last rework (259 to 298 of 617). Houses in water must stay 0.
- **Switch.** Rides `urban_riverbank`.
- **Acceptance bar** (towns of 1 500 and up):
  - ~~at least 15 % of houses within two street widths of the bank;~~ **about 5 %** of houses within two street widths of the bank (owner Ruling FQ, 2026-11-16, replacing 15 %, held on seeds 483920, 912345 and 246371 at 384 x 288 and 2 048 x 1 311). Measured with `urban_river_stretch` on and its worse rows fixed: 2.5 -> **4.1 %** at 384 and 2.5 -> **3.7 %** at 2 048, short of about 5 %; the prototype's 5.1 / 4.4 % came with the rows Ruling FQ had fixed, whose towns now lay out unstretched. **Owner Ruling FS (2026-11-17) accepted the measured share and the switch is on in the app: 4.1 % (384) and 3.7 % (2 048) is now this bar's floor**, re-measured the same day (byte-identical census) and not to fall under a later change (`STATUS.md`, "River towns stretched along their river, on");
  - no river town with every house over two widths away;
  - at least 70 % of the built reach's bank length fronted by a building or a quay;
  - buildings in water 0.

### 6. Scale the market square with the town, and allow a spindle

- **What.** Size the square from the population instead of the fixed 55 to 80 m by 26 to 40 m. Let some towns widen the main street into a tapering market street instead of a rectangle.
- **Why.** One size cannot be right for both a hamlet and a city.
- **Evidence.** The square is 7 % of a village's built area and 0.28 % of a large town's. Research rule 13 (market streets taper) is sourced. **The research found no sourced area ratio**, so the size has to be a labelled judgement; I suggest about 0.5 to 1 % of the built area for an organic town, and I do not claim a source for it.
- **Expected visual effect.** A market that looks like the centre of a large town; a village green in proportion to its village.
- **Cost.** Small. On the riverbank path only, because `build_plaza`'s two draws are golden-pinned elsewhere.
- **Risk.** A larger square removes lots near the centre and lowers housing a little.
- **Switch.** Rides `urban_riverbank`, or the new switch of change 4.
- **Acceptance bar:**
  - square area between 0.4 and 1.2 % of built area for towns of 5 000 and up;
  - village green at most 4 % of the village's built area;
  - square still inside the wall;
  - housed median unchanged within 3 points.

### 7. Small items, each worth doing only alongside the above

- **Show the river crossings of the wall.** A tower each side and a chain or water gate where the circuit crosses the river (11 of 32 walled river towns). Sourced (York, Norwich). Small cost. Bar: every such crossing carries a drawn water gate.
- **Small far banks as a strip.** While the far bank holds under about a third of the near bank's lots, start its streets within 90 m of the bridge approach (the rule `grow` already applies to extramural growth). Sourced (Southwark). Small cost. Bar: in such towns, at least 80 % of far-bank lots within 90 m of the approach road.
- **Add the integration check to the census.** Report the market's betweenness percentile per town, so the poorly connected tail (p10 of 15 to 55) can be listed and looked at. Measurement only.

## 4. Things to keep as they are

- **`grow`'s street rules.** T-junction snapping, the 15° branch jitter, the market gradient. They produce the organic network the research describes, and they are the reference's.
- **The far-bank logic of 2026-11-15.** The near bank first, the far bank only when short of target, never larger than the near bank, one bridge.
- **The market at the bridge foot and `riverbank_square` for chartered towns.** This is the landing and bridgehead origin the research describes.
- **Extramural growth clinging to approach roads, and the faubourg against the wall** (Ruling H). Both match the suburb pattern (research §2.3).
- **The wall-generation machinery** (`SettlementRules`, `supersede_wall`). Its gaps are drawn from Cologne, Florence and Paris, the same cases the research cites.
- **The river-front wall with spurs, and the harbour water gate.**
- **The citadel astride the curtain.**
- **Street widths.** No source supports changing them.
- **Buildings in water: 0.** Every change above must hold it.
- **Ruling EW item 1's intent.** No wide empty band inside the wall. Change 1 keeps it and removes only the scallops.

## 5. Questions only the owner can answer

1. **The wall (change 1).** A taut wall encloses about a fifth more ground than today's traced one. Is open ground of that size inside the wall acceptable, given Ruling EW item 1? My recommendation is yes: it is shorter wall, which is the ruling's own reason.
2. **Villages (change 2).** The outer blocks are re-platted as courtyard perimeter blocks (Ruling H, sited by Ruling AD), and in a village every block is an outer block. Should villages keep courtyard blocks, or be rows along a street?
3. **Plots (change 4).** Deep plots make towns larger on the map (up to about 1.5 times the footprint) and move every housing figure for new worlds. Is that wanted, and is a prototype with measurements the right first step?
4. **The river (change 3).** Should this be done here or under `RIVER_REALISM_SCOPE.md`? The town and the map must share one curve.
5. **The market size (change 6).** With no sourced figure, is "about 0.5 to 1 % of the built area" an acceptable labelled judgement?
6. **Variety not ruled on.** Do you want founded second towns on the far bank (Prague's Malá Strana), or planted grid towns (bastides, Kraków)? Both are sourced, both are large pieces of work, and neither fixes a visible fault.
7. **Old wall rings.** Should an old town visibly show its superseded ring? The machinery exists; whether it fires in the app's worlds has not been measured.

## 6. What the research did not settle

- **Street widths by class.** No source. The generator's values stand unjudged.
- **Market area against population.** No source beyond Kraków's planted square.
- **Wall length against population.** Only the researcher's circle-equivalent arithmetic, with Paris and Nördlingen as spot checks. Change 1's bar is therefore set against the wall's own hull, not against a real-world figure.
- **Villages.** The research covers towns and cities. Change 2 rests on the frames and the measurements.
- **Whether the scaling law's "settled area" is comparable with mine.** Its areas include gardens, vacant ground and suburbs; mine is a tight footprint. The "twice as dense" reading is an upper bound on the gap.
- **River shape and width in a town.** Not researched.
- **What became of superseded walls, and fringe belts.** The mechanism is the researcher's inference.
- **No plan images.** The research retrieved none, so neither it nor I compared a generated town with a real plan by eye. Before change 4 is built, someone should put two or three real plans (Braun and Hogenberg or Merian, as the research suggests) beside the frames.
- **Caniggia and Muratori, Barthelemy, and every non-European tradition** except a thin note on Cairo and Fez.
- **My own gaps.** One run, 384 cells only, three seeds, river sites only. I did not measure 2 048 cells or landlocked and coastal towns, and I did not check how often a second wall ring fires. The betweenness figures are sampled (120 sources per town).

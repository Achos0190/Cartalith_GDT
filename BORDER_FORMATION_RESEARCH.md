# Border formation research

**Written 2026-11-22 from `origin/main` at `2acb7139`. Research, not a scope; it builds nothing and carries no status column.** It answers Ruling GL item F25 (`LARGE_ITEM_RULINGS.md`; the owner finding is in `OWNER_FINDINGS_2026-11-21.md`):

> *"There is continental influence and borders drawn rather in straight lines, borders often stop at geological features research this and adapt the drawing of borders."*

The owner chose **research first**. Every statement about the code names the symbol it was read at. Every statement about the world carries a tag: **[source]** (read in a cited source), **[code]** (read in this tree), **[measured]** (produced by a command listed in §1.6 on this date), **[inference]** (this document's reasoning, not verified). Web sources are search results summarised, not fetched in full; where a source is a reference mirror or a secondary site it is marked, and the claim is held at the weaker tag.

---

## 0. The answer in six lines

1. **Territory is a multiplicatively-weighted Voronoi diagram in disguise.** `assign_territory` runs a Dijkstra from each capital over `build_travel_cost`'s field, but on land that field is `1 + 50·slope²` with a slope that is almost nil: on all three seeds the **median land cell costs 1.000 and 98 to 99 % of land costs under 1.1** **[measured]**. Cost-distance therefore collapses to straight-line distance divided by a capital weight, and the border between two capitals is a straight bisector (equal weights) or an Apollonius circle (unequal), which is what the owner is seeing.
2. **Nothing the cost field ignores can bend a border:** rivers, ridges and passes as such, biome (forest, marsh, desert), the coast other than as a hard stop. The three measured worlds show **no border preference for ridges** (border share on a local high 0.63 / 0.53 / 0.54 against a land baseline 0.61 / 0.53 / 0.57), and borders sit near rivers *less* than land does (9 to 11 % against 13 %) **[measured]**.
3. **Borders are drawn as a raster wash, not a line.** The faction border exists only as the edge of a colour wash in a nearest-neighbour `TextureRect`; only provinces get a drawn line (a 3 px raster stroke). There is no vector border to smooth, snap or style **[code]**.
4. **Real borders are not natural lines either.** The scholarship calls "natural borders" a persistent myth, crests rarely coincide with watersheds, rivers alternate between frontier and spine, and states leave margins unclaimed where friction is high **[source]**. So "follow geological features" should mean *terrain shapes the cost of reaching a cell*, and borders emerge there, not that a border is welded to the nearest ridge.
5. **Recommendation:** a **separate territory friction field** (ridge/relief, river crossing and river corridor, biome, pass corridors), a feature-aware vector border line for drawing, and an optional unclaimed-margin rule; stage render-only work first, generation change second behind a §7p switch for new worlds only (§4).
6. **"Continental influence" is not a name anything in the tree carries**; §1.5 lists what it may mean and asks.

---

## 1. How borders are computed and drawn today

### 1.1 The chain, at its symbols

| Stage | Where | What it does |
|---|---|---|
| Cost field | `cartalith_civ::build_travel_cost(field, gw, gh, sea)` | `1 + SLOPE_K · slope²` with `SLOPE_K = 50` (reference `buildTravelCost`, line 3257), slope being the central-difference gradient in height units per cell; cells below sea are `INFINITY` (water is impassable). Nothing else enters: no river, biome, geology, coast distance or pass term. |
| Capitals | `compute_civilisation` in `cartalith-godot/src/lib.rs`: `let cost = cartalith_civ::build_travel_cost(&ws.field, gw, gh, sea_level)` then `assign_territory(&settlements, &cost, gw, gh, world)` | The **same** field also feeds the road network (`civ_hierarchical_network_topology_with`, `civ_merge_network_paths`) and the settlement explanation's `travel_cost`. |
| Territory | `assign_territory` → `territory_sweep` / `territory_sweep_batched` | One `road_dijkstra` per **capital** (`placement.capital`; other settlements project nothing), 8-neighbour, diagonals × √2, optional x-wrap (`world`). A cell's effective distance is `dist / territory_weight(pop)`; the lowest wins; the winner's faction id is the owner. Unreached cells (water, or land no capital reaches) stay `0`. |
| Weight | `territory_weight(pop) = 1 + ln(1 + pop / 15000)` with `TERRITORY_POP_REF = 15000` | Documented in the code as the owner-decided design of `DECISIONS.md` §7b: cost-distance Voronoi, *"not straight-line Voronoi (ignores terrain, reads as artificial)"*. A capital's real population puts `w` roughly between 1.41 and 2.10. |
| Claims and influence | `territory_influence` (same sweep with `want_rival`) → `TerritoryInfluence { owner, rival, influence, contested }` | Built on demand for the Layers popover's *Political control* and *Contested borders* views and CIVIL ▸ Territories ▸ Borders & influence (`civ_territory_influence`); nothing retains it. `contested` is the ratio of the winner's to the runner-up's effective distance. |
| Provinces | `civ_generate_provinces(settlements, territory, gw, gh)` | A **plain unweighted** Voronoi of same-faction city-tier seeds, restricted to cells the faction owns. |
| Paint | `civ_rebase_territory_paint`, `civ_commit_territory_paint`, `civ_territory_paint_at/_polygon` | Hand paint overrides the computed grid; provinces are rebuilt over the merged grid. |
| Smoothing | **none** | No pass touches `territory` between the sweep and the texture. Smoothing exists elsewhere in civ (way geometry) and does not touch territory. |
| Timeline | `timeline::civ_territory_at`, `TerritoryFrame` keyframes and deltas | Each recorded year stores the raster; the year cursor replays it. A territory change is a change to every stored frame only for newly generated worlds. |

### 1.2 What reads the territory raster

Changing territory moves more than a picture. Readers found by `grep territory` over the crates **[code]**: `manpower.rs` (`land_capacity` is a sum over the faction's own cells: the military manpower model and Ruling DE's ecological term), `trade.rs`, `belief.rs` (custom religions apply only in their faction's territory), `relations.rs` (`border_cells`, `border_fraction`, the friction term), `garrison.rs`, `landmark.rs` and `landmark/cultural_kinds.rs` (cultural landmarks by faction), `labels.rs`, `civ_continents_with_styles` (continent naming culture by plurality owner), `roster.rs` (faction removal reverts cells), `timeline.rs`, `campaign.rs`/`conflict_bridge.rs`, `export_raster.rs` and `cartalith-engine::geojson` (`_geoTerritoryFeature`'s polygons), and the saved claim grid (`SAVEFILE_COMPAT.md`).

### 1.3 How it is drawn

- `CivData`'s grid becomes `build_territory_texture()` in `cartalith-godot/src/lib.rs`: each owned cell is its faction swatch at alpha `territory_opacity` (default 82/255), faded under the plate frame by `render::border_cover`. Unowned is transparent.
- `viewport_host.gd` puts it in `territory_view = _raster()`, and `_raster()` sets `TEXTURE_FILTER_NEAREST`. **A faction border is therefore the staircase edge of a colour block at grid resolution.** There is no faction border ink, and none of the ink settings reach a faction border (`border_ink`/`border_alpha` are the plate frame and the province line).
- Provinces: `build_province_boundary_texture()` flags a cell whose four neighbours differ, dilates it to 3 px, and draws it in a fixed ink `[35, 24, 9, 235]`. Also a raster, also nearest-neighbour.
- Export: `geojson` traces territory polygons from the raster, so any stair-step or straightness travels into the export too.

### 1.4 What the three seeds show

World parameters of every number here: `generate_sized(seed, 4000 km, 1024, 672)` then `civ_populate()` (default archetype, default factions: six, with villages off), release build of this tree, on 2026-11-22. Seeds 483920, 912345, 246371.

**Frames (composite of `build_color_texture()` and the territory wash).** Read by eye on two of them; held in a scratch directory and not committed under this task's brief:

- **483920.** The west side is split by a **dead-straight near-vertical border** (about x = 115 of 1024) running the full height of a flat plain, and by two further straight diagonals; the faction in the south-west plain is cut from its neighbour by a straight line that ignores a river valley running diagonally through it. A coastal ridge that is drawn on the coast line is crossed by borders, not used by them.
- **246371.** A **horizontal border (about y = 157, x = 310 to 560) ruled across open ground**; a prominent snow-capped range (x about 540 to 680) lies **inside** one territory rather than on a border; two borders meet the coast at angles that follow neither shore nor relief.
- **Both.** At the outer rim the wash simply stops at the shoreline: unowned land is 0.0 to 0.2 % of sampled land, so nothing is left unclaimed anywhere a capital can reach.

**Measured straightness** (territory raster at 1024 × 672, border pixels = 4-adjacent owned cells of different faction, "linear" = the border pixel's radius-12 px neighbourhood fits a line with RMS residual < 0.8 px; control = a pure weighted Euclidean Voronoi built in the script from the same capitals and populations, masked to the same owned land):

| Seed | Border px | Linear share, ours | Linear share, Euclidean control | Axis-aligned runs ≥ 12 px |
|---|---|---|---|---|
| 483920 | 2327 | 0.902 | 0.963 | 0.396 |
| 912345 | 2141 | 0.930 | 0.962 | 0.378 |
| 246371 | 2064 | 0.938 | 0.951 | 0.288 |

The shipped borders are **only 1.3 to 6 points less straight than a terrain-free diagram** **[measured]**. Terrain bends them hardly at all, which is the headline of this section. The axis-aligned share is the nearest-neighbour stair of a few shallow-angle edges and is mostly a drawing artefact (§1.3).

**Measured cost field** (stride-4 samples of land, `cost = 1 + 50·(slope_n / gw)²`, with `slope_n` from `sample_cell`, `gw` = 1024):

| Seed | Median | 90th pct | 99th pct | Max | Land > 1.1 | Land > 1.5 |
|---|---|---|---|---|---|---|
| 483920 | 1.000 | 1.004 | 1.168 | 5.9 | 1.9 % | 0.04 % |
| 912345 | 1.000 | 1.032 | 1.183 | 5.4 | 3.3 % | 0.07 % |
| 246371 | 1.000 | 1.009 | 1.116 | 3.5 | 1.5 % | 0.01 % |

So the only cells that bend a path are a **sub-1 % sliver of steep land**, and the weighted Voronoi has the shape the weights give it. (The slope term is a reference constant tuned to make *roads* avoid cliffs; as a territory barrier it is two orders of magnitude too weak. **[inference]** that it was never meant to model a barrier, since §7b chose it only because it is "one real terrain-cost metric, not a second one invented for this".)

**Measured terrain relation at border cells** (stride-4 grid of 256 × 168 samples, 525 to 604 land-land border samples per seed; "ridge" = the sample is above the mean of its 7 × 7 sample window; "river" = a river of order ≥ 3 within one sample; "steep" = slope ≥ 8°):

| Seed | Ridge: border / land | River within one sample: border / land | Steep: border / land | Mean slope (deg): border / land |
|---|---|---|---|---|
| 483920 | 0.63 / 0.61 | 9.8 % / 12.7 % | 0 % / 0.20 % | 0.36 / 0.45 |
| 912345 | 0.53 / 0.53 | 11.0 % / 12.8 % | 0.9 % / 0.27 % | 0.67 / 0.71 |
| 246371 | 0.54 / 0.57 | 9.1 % / 13.0 % | 0 % / 0.09 % | 0.67 / 0.49 |

A border that followed relief would show a ridge share well above the land baseline; none does, and borders touch rivers *less* than random land does. (The river measure undercounts: a one-cell river is hit by about one sample in four, so the figure is for comparison with the same measure on land, not an absolute.)

**Frontier width** (`civ_territory_influence().borders`): six or seven neighbouring pairs per world, 1 482 to 14 626 contested cells per pair, mean contested ratio 0.93 to 0.94. A frontier is a wide band where winner and rival are nearly equal in cost, because on flat ground the cost surface has no gradient to localise the line **[inference]**.

### 1.5 "Continental influence"

No symbol, setting or document in the tree carries this phrase (`grep -i "continental.influence"` finds only the owner's finding). Candidates, which the owner can settle in one line (Q1):

1. The **weighted-Voronoi reach** itself: a big capital's weight pushes its border far across a continent as a gentle circular arc, so a single continent reads as sliced by a few giant arcs. This is what §1.4 measures.
2. The **Political control / Contested borders** debug views (`sample_bridge.rs`, `control` and `influence`), where the influence ramp is the same sweep.
3. **Continent naming** (`civ_continents_with_styles`), which follows the plurality owner, so a continent can wear a faction's name.
4. The **continental plate** (`plate_type` "continental" in `sample_cell`): borders sitting on the tectonic boundary would be "geological features", the other half of the finding.

This document assumes (1) and notes where (4) matters.

### 1.6 Commands that produced the numbers

A throwaway GDScript run under Godot 4.7.2 (`--headless --script`) against a release `cartalith_godot.dll` built from `origin/main` at `2acb7139`, writing, per seed, the territory and colour textures, `get_settlements()` and a stride-4 table of `sample_cell` (`elevation_m`, `slope_deg`, `slope_n`, `controlling_faction`, `river_order`, `water`), then a Python/NumPy script for the straightness, control and terrain tables. Nothing was committed; the scripts (`_border_dump.gd`, `analyze.py`) are scratch. Straightness uses one radius and tolerance, chosen once, not tuned; single run per seed, deterministic, so no spread. **Not measured:** anything on a different map size, archetype or faction count; the 16 k plan; how the numbers change under the proposed options (that is §4's first milestone).

---

## 2. How real borders form

Tags: **[source]** read in the cited result; **[inference]** this document's reasoning. Search results were summaries, and several came through mirror or popular sites; those are marked **(weak)** and are not load-bearing.

### 2.1 The "natural border" is a contested idea

- Natural borders are defined as borders concomitant with natural formations such as rivers or mountain ranges **[source: Wikipedia, "Natural border"]**.
- Juliet Fall, *"Artificial States? On the enduring geographical myth of natural borders"*, *Political Geography* (2010), treats the idea as a long-lived myth that disappears and reappears in new guises **[source]**. A 2025 paper by Proto and Buscemi revisits it and notes critical border studies have deeply questioned the "naturality" of borders **[source]**.
- Practical problem: a mountain boundary drawn on the crest rarely coincides with the watershed; the Pyrenees between France and Spain are the cited example, with water disputes at the mismatch **[source: Biger, *Historical geography and international boundaries*, via Cambridge Core]**.
- Technology reduced the cost of crossing natural barriers and so their importance; borders drawn more recently (much of Africa and the Americas) fit natural features less **[source, secondary]**.
- **Consequence for Cartalith [inference]:** the target is *plausibility under friction*, not a rule that borders lie on ridges. A world with a medieval-to-modern timeline wants barriers that matter more in the early years, which the year cursor already exists to express.

### 2.2 Mountains, ridges and watersheds

- Orographic boundaries follow ridgelines or watersheds, with the Pyrenees and the Andes (Chile/Argentina) as standard examples **[source: reference summary]**.
- Scott's *friction of terrain*: rugged ground slows movement and limits how far a state projects power uphill; state power does not spread evenly but goes around mountains and into valleys, so a hilly area a few kilometres from a centre can be more autonomous than its distance suggests. It limits **effective control**, not **claimed sovereignty** **[source: James C. Scott, *The Art of Not Being Governed*, via reviews; Scott lists railways, roads, telegraph and aircraft as what weakened it]**.
- **[inference]** This is the single most useful idea for the engine: a cost field with real friction in rough terrain, and reach limited by it, yields highland autonomy *and* valley-shaped territories with no snapping rule at all.

### 2.3 Rivers: borders and connectors

- Rivers are both. The Rhine was a Roman frontier and later the central axis of the Frankish and Carolingian realms; south of Bonn the Roman limes ran well east of the river **[source: Britannica, History of the Rhine]**. The Rhine and Danube formed the backbone of the Roman frontiers of mainland Europe *by default*, not as policy **[source: Taylor & Francis chapter, as summarised]**.
- The 1870 essay by Heilprin argues watersheds, not rivers, are the natural boundaries and that river frontiers are held by force **(weak: polemic, not settled scholarship)**.
- Legal practice: median line for non-navigable rivers and the principal channel (thalweg) for navigable ones, broadly from Article 30 of the 1919 Treaty of Versailles **[source, as summarised]**.
- A medieval internal border: the Catalonia–Aragón boundary, stable since about 1305, follows the Noguera Ribagorzana for about 140 km of its northern half **[source: academic abstract]**.
- **[inference]** Both roles come from one mechanism: a river is cheap to travel *along* (a corridor that unites a basin) and expensive to cross (a barrier between banks), and the later settled border is wherever contested power balances. The existing engine already treats rivers as settlement magnets (Ruling N) but not as a cost either way.

### 2.4 Passes, and who controls them

- The Gotthard Pass lies on the main watershed between Valais, Ticino, Graubünden and Uri; it became usable around 1200 (bridges), Uri, Schwyz and Unterwalden allied against Habsburg control of the northern approach, and the Confederates took the Leventina from Milan at Giornico in 1478; later efforts to secure the route are cited for why Ticino is Swiss **[source: Britannica, swissinfo, Wikipedia]**.
- **[inference]** A pass is the opposite of a barrier on the line of a ridge: it pulls the border *over* the ridge to whoever holds the approach, producing the "valley bound to a distant centre" pattern. The tree already has a golden-verified mountain-pass corridor detector (`LANDMARK_GENERATION_SCOPE.md`), which is a ready input.

### 2.5 Coasts, forests, marsh and desert

- The Silva Carbonaria acted as a natural boundary from the Late Iron Age into the early Middle Ages: it divided Clovis's kingdom from Sigebert's, later Austrasia from Neustria, and continued as the boundary between the bishoprics of Liège and Cambrai; it was also managed and partly cleared, so it was a frontier, not a void **[source: Wikipedia mirror of a Cambridge Ancient History passage, (weak)]**. The Black Forest is tied to *marka*, "border", by name **(weak)**.
- **[inference]** Dense forest, marsh and desert add friction and thin settlement rather than forming walls; borderlands of low population are a common result. The engine's biome raster and the existing marsh and forest fields can express this as multipliers.
- Coasts: the sea is already a hard stop, which is realistic for a land sweep; sea reach (ports, `civ_sea_routes`) is a separate question that this document leaves out.

### 2.5a Distance, power and size

- Boulding's loss-of-strength gradient (*Conflict and Defense*, 1962): military power usable at a place falls with distance from its base; two powers of unequal total strength have equal *practicable* strength at some line, and forward bases move the line **[source]**.
- Renfrew and Level's **XTENT** model computes hypothetical territories from centre size and inter-centre distance, so larger centres reach farther; Ducke and Kroefges present a GIS version that respects terrain properties, and note the plain Voronoi baseline is very sensitive to the spatial configuration of sites **[source: Ducke and Kroefges, XTENT, via the Heidelberg archive]**. I did not confirm how their terrain handling is parameterised **[unverified]**.
- **[inference]** Cartalith's design (§7b) is already a cost-distance XTENT. The finding is that the *cost* half is empty, not that the idea is wrong.

### 2.6 Takeaways used below

1. Barriers act by **friction and thin control**, not by a rule that a line equals a feature **[source, 2.1–2.2]**.
2. Rivers need **two signs**: cheap along, dear across **[inference from 2.3]**.
3. Passes **re-route** reach across ridges **[2.4]**.
4. Margins can stay **unclaimed or thin** where friction is high **[2.2, 2.5]**.
5. Border *lines* in history (treaty or thalweg lines) are drawn **after** control, in finished form; a drawn border may sensibly snap to a feature even where the underlying reach is soft **[inference]**.

---

## 3. Options for Cartalith

Every option states what it moves. "§7p" means a generated-output change needing an owner ruling, a `cartalith_engine::WorldParams::defaults()` / `cartalith_godot::params::defaults()` split and `PARAMS` plus `JS_PATHS` rows (`MISTAKES.md` preflight "Change generated output"), new worlds only. "Baselines" means `fh_r2_baseline.rs`'s harness worlds, Ruling DE's ecological-term bars, and any golden that reads a territory raster (the territory has no JS golden: `assign_territory`'s doc says so).

### O1. Territory friction field (cost-distance over real friction)

Replace the territory sweep's cost with a dedicated `territory_friction(...)` field: the slope term rescaled for relief, plus a **ridge/relief** term (local elevation gain or TPI), a **river** term (cheap along an order-weighted corridor, dear across, so crossings keep a bridge/ford discount where the route builder already charges them (`map_crossings`, `way_audit::river_crossings`)), a **biome** term (forest, marsh, desert, ice as multipliers), and a **pass** discount from the existing corridor detector. The sweep (`territory_sweep`) is unchanged; only its input is.

- **Must not** touch `build_travel_cost` itself: the road network, `jp*` journey planner and settlement explanations read it and are golden-pinned. A second field keeps those bit-identical.
- **Moves:** generated territory and provinces; every consumer in §1.2 (manpower land capacity, trade catchments, relations' border cells, garrisons, cultural landmarks, continent naming); saves only for newly generated worlds (the claim grid is stored, so existing saves are untouched); baselines (yes: Ruling DE's bars, the FH-R2 harness worlds). §7p, new worlds only. Labels move with the cells, not by themselves.
- **Cost:** a new field builder (`O(cells)`, cheaper than one Dijkstra), a few parameters, tests, the re-baseline. The sweep's cost is unchanged apart from more varied edge weights, which can slow Dijkstra's frontier modestly **[inference]**; measure at 2048² and 8192² before adopting, since `PERFORMANCE_BENCHMARKS.md` §12.6 tuned the batch to six default grids.
- **Strength:** the only option that produces highland autonomy, valley-shaped states and pass control from one mechanism (2.2–2.4). Risk: unbounded friction can orphan land (reach limit) or give a capital a bizarre claim; needs the sweep's invariant tests (capital cell self-owned, unreachable cells unowned) and bars.

### O2. Snap or relax border seams to nearby features (post-process)

After the sweep, take each border chain between two factions (between junctions) and re-route it as a **minimum-cost seam** over a band of N cells either side of the current line, with a cost favouring ridge crests, rivers' centre lines (thalweg) and shore, and penalising length change. Capital cells and their immediate surround are pinned; a seam may not cut a settlement off from its owner.

- **Moves:** territory cells within the band (generated), so the same consumers as O1 but smaller deltas; labels where a settlement flips owner; saves and baselines as O1 (§7p, new worlds only).
- **Cost:** medium: seam extraction, junction handling (three-way meets), a pin rule, tests. Self-consistent with timeline frames only if run in generation and on each replayed year (it would otherwise change the frame sequence).
- **Strength and weakness:** it lets a border land on a feature even on flat reach, which is the "treaty line after the fact" case (2.6.5). But it can look arbitrary where nothing physical explains the move, and it cannot create pass control or highland autonomy. Best as a finisher after O1, not instead.

### O3. Watershed-based partition

Partition land into drainage basins from the flow field (`ws.flow_discharge`, `river_order`), merge small basins into their receivers, give each basin to the capital that sits in it, or nearest by basin-graph distance; borders are then drainage divides and rivers are spines.

- **Moves:** generated territory wholesale (a different algorithm, not a tweak): everything in O1's list, with larger deltas; saves, baselines §7p, new worlds only.
- **Cost:** high: basin merge rules (a continent-scale river's basin dwarfs the rest), basins with no capital, and basins that cross coasts or lakes; a new owner decision for the capital-less ones. Likely to need O1 anyway for the flat interior.
- **Strength and weakness:** gives crisp river-basin realms (the "connector" role) and ridge borders (the "barrier" role), consistent with the watershed view in 2.1 and 2.2. But it enshrines the 1870 "watersheds are natural" claim that the literature disputes (2.1), and ignores that a pass crosses the divide (2.4). Weaker than O1 for the flat plains where the owner's straight lines actually are, because flat land has ill-defined basins.

### O4. Feature-preserving smoothing and a vector border line (render only)

Extract the border from the raster as polylines (marching squares or edge-following, with consistent shared edges), simplify then smooth, and draw an ink line (with the wash kept), with a constraint that smoothing cannot move a vertex more than N px if that vertex lies within M px of a mask built from coast, river centre line and ridge. Same polylines feed `geojson` so the export stops being a staircase.

- **Moves:** shell drawing (`viewport_host.gd` gains a border layer; `territory_view` stays), possibly `export_raster.rs` and `geojson`; **does not** move generated territory, saves, labels, baselines or §7p. Because it only redraws the same cells, `civ_territory_influence` and every reader of the raster are untouched.
- **Cost:** low to medium: a new Rust function returning polylines through the bridge, a GDScript draw layer with a zoom-scaled stroke floor (`map_overlay.gd`'s `1 / _camera_zoom` rule in `MISTAKES.md`), phone behaviour, tests of closed rings and shared edges.
- **Limit:** it improves how borders *look* (stairs, jaggedness) and lets a line sit tidily on a coast or river, but it **cannot make a straight bisector follow a ridge**, because the territory under it is still a Voronoi. Should not be sold as the fix for the finding.

### O5. Unclaimed and thin-control margins

Let a cell stay unowned when its friction-distance from every capital exceeds a reach limit (scaled by capital weight), restricted by default to desert, ice, high relief and marsh, never plain forest. Contested mid-bands (`contested` close to 1) may be drawn as marches.

- **Moves:** generated territory (more `0` cells), labels (unowned land now exists; `civ_continents_with_styles` already tolerates it), manpower land capacity (smaller), relations' `border_cells` (a frontier may have a gap, so `border_fraction` changes), faction stats and roster counts; saves unchanged for old worlds; baselines yes. §7p, new worlds only.
- **Cost:** low once O1's friction exists (a threshold), medium for the UI reading ("unclaimed" vs "wilderness") and for the timeline (a faction growing into margins over years is a natural future use).
- **Source link:** 2.2 and 2.5.

### O6. Procedural border irregularity (rejected)

Displacing borders with noise to "look natural" is cheap and tempting; it would add wobble unrelated to any feature, and measures as less straight without being more plausible. Listed to record that it is not recommended **[inference]**, and because it is what many map apps do.

### 3.1 Comparison

| Option | Fixes straightness | Follows relief/rivers | Pass control | Generated output | Saves | Baselines | Cost |
|---|---|---|---|---|---|---|---|
| O1 friction field | yes (where friction varies) | yes, as reach | yes | §7p, new worlds | new only | yes | medium |
| O2 seam snap | yes | on the line only | no | §7p, new worlds | new only | yes (smaller) | medium |
| O3 watersheds | yes (hilly), weak on plains | basins and divides | no | §7p, new worlds | new only | yes (large) | high |
| O4 vector line | look only | line sits on features | no | none | none | none | low-medium |
| O5 margins | no | thin control | partial | §7p, new worlds | new only | yes | low after O1 |
| O6 noise | cosmetic | no | no | none or §7p | none | none | low (not recommended) |

---

## 4. Recommendation

**Stage it so each step is measured and the generation change is a switch.**

1. **R0, measure first (no product code, a harness).** Turn §1.4's scratch scripts into a committed measurement (`#[ignore]`d test, in `cartalith-godot/tests/`) over the 20 FH-R2 harness worlds plus these three seeds: border linear share, ridge/river/steep shares against the land baseline, owned-land fraction, per-faction area change under any candidate. This fixes the bars before anything is built (`MISTAKES.md`: never pin a constant against itself).
2. **R1, O4 (render only, no §7p).** Vector border line with feature-preserving smoothing, shared with the GeoJSON export. It changes nothing generated, so it can ship before any ruling and answers the "drawn" half of the finding. State plainly in the UI copy and the doc that it does not change territory.
3. **R2, O1 (the cause), behind a §7p switch, new worlds only.** A separate `territory_friction` with: relief rescaled so a steep ridge costs as much as a long detour (starting values proposed, to be swept like `PS-2`'s heading gain), river term cheap-along / dear-across with bridges as a discount, biome multipliers, pass discount from the corridor detector. Keep `build_travel_cost` and the road network bit-identical (a test must prove roads did not move). Re-baseline Ruling DE's bars and the harness worlds with an owner ruling.
4. **R3, O5 (margins) as a second switch,** after R2's reach distribution is known; default off until the owner sees frames.
5. **R4, O2 only if** R2's frames still show a border that ought to sit on a coast or river and does not. Pin capitals, measure again.
6. Not recommended: O3 as the main algorithm (§3), O6.

**Why O1 and O4 first.** O1 is the one option that follows from the measured cause (§1.4: the cost surface is flat) and the sources (§2.2: friction, not lines), and it keeps the §7b design the owner already approved. O4 is the only part that can ship with no ruling and no baseline movement, and the visual half of the finding is partly a drawing problem (stairs, no line).

**Bars proposed for R0 to confirm (defaults, to be tuned by measurement):** border cells on a ridge or within 2 cells of a river or the coast at least 1.5 times the land baseline over the harness worlds; border linear share at most 0.80 (today 0.90 to 0.94); every capital's own cell and its 3-cell surround owned by it; owned land at least 90 % of reachable land unless O5 is on; no faction's area changing by more than a stated factor without a recorded cause; road geometry bit-identical (switch on and off); sweep time within 1.3 times today's at 2048² (median with min..max, harness alone).

---

## 5. Owner questions, with defaults

| # | Question | Default if unanswered |
|---|---|---|
| Q1 | What does "continental influence" mean in your finding (§1.5): big capitals' arcs, the Contested/Political-control views, continent naming, or the continental plate? | The first: big capitals' weighted reach cutting continents with long arcs |
| Q2 | Is a render-only first step (R1, a smoothed vector border line, no change to who owns what) welcome before any generation change? | Yes |
| Q3 | Is the territory change a §7p switch for **new worlds only**, existing saves and painted territory untouched (Rulings GK, GL pattern)? | Yes |
| Q4 | Are rivers mostly **connectors** (cheap along, a moderate crossing cost, borders usually on ridges) or **borders**? | Connectors, with a crossing cost that bridges halve; a river becomes a border mainly where basins are contested in the lower reach |
| Q5 | May land be left **unclaimed** (desert, ice, high relief, marsh), and should forest count? | Yes for desert, ice, high relief, marsh; no for forest |
| Q6 | Should passes **pull** a border across a ridge to the nearer capital (and later invite a garrison or a fort landmark)? | Yes, pass cells keep low friction; no garrison change now |
| Q7 | One "border terrain-following" slider (0 = today's flat Voronoi, 1 = full friction) or a fixed behaviour? | One slider in New World advanced, default 0.6; 0 reproduces today's territory exactly |
| Q8 | How should a faction border be drawn: wash only (today), a thin ink line over the wash, or a line with a march (hatched) band where contested? | A thin smoothed ink line over the wash; hatching stays in the Contested view |
| Q9 | Does the timeline's early era get stronger friction than its late era (the cost of crossing barriers fell with technology, 2.1)? | No for now; one friction, revisit after R2 |
| Q10 | Is re-baselining Ruling DE's ecological bars and the FH-R2 harness worlds on R2 acceptable (the territory feeds land capacity and so armies)? | Yes, one ruling at R2 |

Every row's default is the owner's to overrule; none is assumed approved. This document changes no behaviour, so none of its rows needs a ruling to *read*.

---

## 6. Sources

Search-result summaries, not full-text reads; confirm before quoting.

- Fall, J. J. (2010), "Artificial States? On the enduring geographical myth of natural borders", *Political Geography*; Proto, M. and Buscemi, F. (2025) on the critical turn (cris.unibo.it). Wikipedia, "Natural border".
- Biger, G., "Historical geography and international boundaries" (Cambridge Core), for the crest-versus-watershed mismatch and the Pyrenees.
- Scott, J. C. (2009), *The Art of Not Being Governed* (Yale), via the *La Vie des idées* review and others, for the friction of terrain.
- Britannica, "History of the Rhine River"; NASA Earth Observatory, "When rivers are borders" (Smith and Popelka's river-border database); a Taylor & Francis chapter on the Roman Rhine and Danube frontier; an academic abstract on the Catalonia–Aragón boundary (about 1305, Noguera Ribagorzana).
- Britannica, "Switzerland: the Swiss Confederation during the Late Middle Ages"; swissinfo; Wikipedia, "Gotthard Pass" (secondary).
- Boulding, K. E. (1962), *Conflict and Defense*, via the Wikipedia article on the loss-of-strength gradient and the Mercatus working paper (secondary).
- Renfrew, C. and Level, E. (1979), XTENT; Ducke, B. and Kroefges, P., "From points to areas: constructing territories from archaeological site patterns using an enhanced Xtent model", Heidelberg archive (propylaeumdok/550).
- Wikipedia mirrors on the *Silva Carbonaria* (weak). Not used as evidence: a forum thread and a hobby blog that appeared in the results.

Related in this repository: `DECISIONS.md` §7b (the territory design), `PHASE2_SCOPE.md` milestone 10, `OWNER_FINDINGS_2026-11-21.md`, `LARGE_ITEM_RULINGS.md` Ruling GL, `LANDMARK_GENERATION_SCOPE.md` (the pass-corridor detector), `RIVER_REALISM_SCOPE.md` (river geometry), `PARALLEL_STREETS_SCOPE.md` (the measure-first pattern this document follows).

# Parallel streets: long blocks where a town's plan calls for them

**A scope of 2026-11-17, from owner Ruling FY** (`LARGE_ITEM_RULINGS.md`): *"Plan parallel streets."* After the
second deep-plots prototype (`e53a480a`, behind `urban_burgage`, off) the owner asked for a `grow` variant that lays
long, near-parallel streets with rectangular blocks where a town's plan calls for it (planted towns, market-street
towns, river-front towns), so that deep burgage plots become possible. **Owner note on the frames, the same day**
(off on the left, deep plots on the right): *"Because in the base I like the overall impression better, under
scrutiny it doesn't hold up."* The base layout reads better at a glance; its streets and plots do not hold up close.
So this design must **keep the base's first-sight impression** (its silhouette, its varied street rhythm, how it
reads at the town layer's zoom) and **fix what fails close up** (block shapes, plot form, street logic). Every option
below is rated on both, and section 5 has a bar that fails a fix that looks good up close but loses the base's
character.

This document **defines** milestones and holds the design reasoning. It carries no status column: what is built is
`cartalith-native/docs/STATUS.md`'s answer (CLAUDE.md, "One place holds state"). **Nothing was built.** The only code
written is a scratch `#[ignore]` measurement (`block_shape_audit`), appended to `town_river_census.rs` in a worktree
that was then removed; it is kept as `cartalith-native/docs/evidence/parallel-streets-2026-11-17/block_shape_audit.rs.txt`
and pins nothing. `urban_burgage` stays in and off (Ruling FY; Q8's 56 m census reach kept).

**What I opened.**

- Code, at the symbols, on `origin/main` at `e01972ad`: `cartalith-urban` `growth::grow`, `GrowOpts`,
  `legalize_street`, `grow_far_bank`; `graph::Graph::add_street`, `attach_point`, `extract_faces`;
  `rules::DEFAULT_RULES`, `MARKET_TOWN_RULES`; `generate::generate_pass` (the organic and radial branches, the order
  of passes); `cleanup::lane_pass`; `blocks::build_blocks` and `build_parcels_with`; `burgage.rs` (all of it);
  `village.rs` (`plat_tofts`, `ROAD_REACH_M`, `road_bank_market`); `stretch.rs` (the module doc, `StretchFrame::dist`,
  `lay_river_spine`, `stretch_holds`); `wallhug.rs` (the module doc); `radial.rs` (the module doc);
  `routes::build_primaries_from_paths`; `cartalith-civ` `urban_adapter::UrbanLayout`, `PlaceOverrides`, `run_layout`.
- Documents: `TOWN_GENERATOR_JUDGEMENT.md` (sections 1, 2, change 4, 4, 5, 6), `MEDIEVAL_TOWN_GROWTH_RESEARCH.md`
  (sections 2.1-2.4, 4.1-4.2), `DECISIONS.md` section 7p and its two `urban_burgage` entries, `STATUS.md`'s two
  deep-plot entries, `OUTSTANDING_WORK.md`'s judgement row (Q5-Q8), the commits `cd4448c7` and `e53a480a`.
- Evidence: `cartalith-native/docs/evidence/burgage-backlanes-2026-11-17/` (summary, `cmp.txt`, the three plans
  side by side, the frames `pair_483920_0` and the three real plans at full resolution).

**Fresh numbers.** "Measured" in this document means the runs in
`cartalith-native/docs/evidence/parallel-streets-2026-11-17/census/` (script `run.sh.txt`): the census world of
`burgage-backlanes-2026-11-17/audit.sh.txt` (seeds 483920, 912345, 246371; 800 km; villages on; the app's
`params::defaults`; 384 x 288 and 2048 x 1311), release test binary built from `e01972ad` plus the scratch test.
Every other figure is cited with its source.

---

## 1. Why `grow`'s blocks fail close up

### 1.1 The growth rule, at the code

`growth::grow` runs `epochs` passes. Each pass draws up to 2 600 candidate streets until it has laid its share of
`GrowOpts::target_len`. A candidate is one of two kinds:

- **an exploration continuation** (`explore && r.chance(0.55)`): a dead-end node, continued outward along its own
  edge's heading plus `r.norm() * continuation_jitter` (0.22 rad, about 12.6 degrees, `DEFAULT_RULES`). Each step is
  a random walk on the heading, so a street that explores for several segments curls.
- **a densification branch** (every other try): a random live edge of 38 m or more, a point at 35-65 % along it, and a
  heading of the edge's bearing **plus or minus 90 degrees** plus `r.norm() * branch_angle_jitter` (0.26 rad, about 15
  degrees).

`legalize_street` then gives the candidate a length (log-normal, median 56 m, clamped 30-125 m) and runs the local
constraints, in this order:

1. **First-hit snap.** The candidate stops at the **first** street it crosses (`seg_int` over `edges_near`) and
   attaches there as a T-junction, unless `pierce_chance` (0.10) lets it cross. A stub under 18 m is refused.
2. **Junction angle.** Refused only if it meets the street it hit at under `junction_angle_limit` (0.44 rad, 25
   degrees). **Any angle from 25 to 90 degrees is accepted**, whatever the street it hit was branched from.
3. Box margin, slope (0.34), water (six samples), wall gates.
4. **Parallel spacing** (M-BLK-4): refused only if a street within `parallel_street_spacing` (24 m) of the
   candidate's midpoint runs **within 0.5 rad (28.6 degrees) of its heading**. The reference's query reads only the
   index cells next to the midpoint (`edges_near`, about one 26 m cell); `GrowOpts::wide_spacing` reads every street
   within the spacing. **A street at an oblique angle is never refused by spacing.**
5. `Graph::add_street` (`attach_point`): each free end snaps to a node within 11 m, else to an edge within 9 m.

Then, after `grow`: `cleanup::lane_pass` (up to three passes) splits each interior face of 12 000 m2 or more within
520 m of the market by a lane **joining the midpoints of its two longest edges**, which cuts a corner off a face when
those two edges are adjacent; `build_blocks` insets each planar face (`Graph::extract_faces`, dead-end spurs
collapsed) by half its street widths.

**So the angles of a block's corners are set by unrelated parents.** A branch is square to its own parent, but its far
end lands on whatever street is first in its way, at any angle over 25 degrees. A four-sided block with two parallel
sides needs two siblings from the same or a parallel parent, both reaching the same far street; with a 15-degree
sigma on each and a 35-65 % random origin, that is rare. Blocks are about one segment across (56 m median, 24 m
spacing), so they are small.

### 1.2 Measured: block shapes on the census world

`block_shape_audit`, towns of 1 500 and up (`PLAZA_MARKET_POP`) with buildings, the market square's block left out,
blocks under 200 m2 left out (slivers smaller than one lot). Corners: an outline's points closer than 6 m merged and
bends under 20 degrees dropped (judgement; the triangle share is also printed at 10 and 30 degrees). Depth across is
the short side of the block's minimum-area rectangle; **83 m** is the block a 40 m lot needs from both long sides
(`build_parcels_with` gives a lot at most 0.48 of the distance across, `burgage::WAIST`).

| | app 384 | app 2 048 | no jitter | no jitter, no pierce | `MARKET_TOWN_RULES` | `urban_burgage` on |
|---|---|---|---|---|---|---|
| towns / blocks | 35 / 5 648 | 36 / 5 282 | 35 / 5 317 | 35 / 5 290 | 35 / 3 414 | 35 / 3 509 |
| triangles, by count (10 / 20 / 30 deg) | 7.2 / **10.7** / 13.6 % | 7.6 / **10.8** / 14.0 % | 3.5 / 4.6 / 5.5 % | 2.9 / 4.1 / 5.1 % | 16.6 / 21.1 / 24.0 % | 18.7 / **24.1** / 27.8 % |
| compact triangles (aspect under 2) | 7.0 % | 7.0 % | 2.5 % | 2.0 % | 12.8 % | 15.7 % |
| quads / pentagon+ by count | **50.3 / 39.1 %** | 50.9 / 38.3 % | 71.2 / 24.2 % | 72.2 / 23.7 % | 41.7 / 37.2 % | 34.6 / 41.3 % |
| triangles / quads / pentagon+ by area | 3.9 / 31.9 / **64.2 %** | 3.8 / 31.1 / 65.0 % | 1.7 / 50.5 / 47.7 % | 1.3 / 52.9 / 45.8 % | 5.2 / 37.7 / 57.1 % | 6.1 / 26.2 / 67.7 % |
| aspect, median (p90) | **1.57** (2.74) | 1.57 (2.75) | 1.60 (3.41) | 1.61 (3.50) | 1.53 (2.62) | 1.59 (2.99) |
| rectangularity, median | **0.70** | 0.69 | 0.89 | 0.90 | 0.67 | 0.63 |
| depth across, median (p90), m | **28.6** (61.2) | 29.0 (60.3) | 26.7 (58.2) | 27.2 (58.6) | 51.4 (87.2) | 51.8 (90.4) |
| depth across 83 m or more | **3.6 %** | 3.7 % | 3.0 % | 3.1 % | 12.6 % | 14.6 % |
| block area, median m2 | 1 005 | 992 | 1 140 | 1 211 | 2 878 | 2 716 |
| interior angles 75-105 deg | 46.2 % | 46.2 % | 79.6 % | 80.9 % | 48.8 % | 39.0 % |
| junction gaps 75-90 deg (folded) | 33.4 % | 33.6 % | 56.4 % | 56.5 % | 35.3 % | 28.5 % |
| per town: bearing entropy / max, median (p10) | **0.98** (0.92) | 0.98 (0.91) | 0.78 (0.64) | 0.77 (0.65) | 0.97 (0.88) | 0.96 (0.90) |
| per town: orientation order phi, median (p90) | **0.08** (0.25) | 0.07 (0.39) | **0.60** (0.83) | 0.62 (0.82) | 0.11 (0.36) | 0.13 (0.29) |
| per town: street with a parallel neighbour 25-120 m away | 24.9 % | 24.5 % | 39.2 % | 40.0 % | 35.3 % | 29.3 % |
| node degree 1 / 2 / 3 / 4 / 5+ | 11 / 33 / 45 / 10 / 1 % | 11 / 32 / 45 / 10 / 1 % | 13 / 34 / 46 / 7 / 1 % | 12 / 34 / 48 / 5 / 1 % | 14 / 28 / 51 / 7 / 0 % | 12 / 41 / 39 / 8 / 1 % |

**Isolating spacing from segment length** (a second pass the same day; app rules with one field changed, 384;
`census/blocks_384_spacing60.txt`, `_seg80`, `_spacing60_seg80`):

| | app | spacing 60 m only | segment median 80 m only | both |
|---|---|---|---|---|
| triangles (compact) | 10.7 % (7.0) | **23.2 %** (12.6) | 11.8 % (7.6) | 24.3 % (14.3) |
| quads / pentagon+ | 50.3 / 39.1 % | 33.3 / 43.5 % | 55.5 / 32.7 % | 34.0 / 41.6 % |
| rectangularity, median | 0.70 | 0.63 | 0.71 | 0.63 |
| depth across, median; 83 m or more | 28.6 m; 3.6 % | 47.7 m; **17.2 %** | 28.0 m; 2.1 % | 46.1 m; 15.2 % |
| interior angles 75-105 deg | 46.2 % | 35.3 % | 48.0 % | 37.0 % |
| per town phi, median (p90) | 0.08 (0.25) | 0.10 (0.24) | 0.09 (0.25) | 0.10 (0.26) |

The spacing alone (through the reference's narrow `edges_near` query, without `wide_spacing`) moves both triangles
and depth; the segment length alone moves neither.

Interior-angle histogram, 15-degree bins from 0 to 180 degrees (app, 384): 0.3 0.8 3.4 5.8 11.9 **22.0 24.2** 14.1
7.2 6.0 4.5 0.0 %. No jitter: 0.3 0.5 1.4 2.3 3.9 **35.8 43.8** 4.1 3.0 3.0 1.8 0.0 %. Junction gaps folded to 0-90
degrees, 15-degree bins (app, 384): 25.6 2.3 7.6 11.1 20.0 33.4 % (the 0-15 bin is mostly the two halves of a street
running straight through a T). Per-town lists and the splits by "touches a primary" and "within 300 m of the market"
are in `census/blocks_*.txt`; the shares barely move across those splits (app 384: triangles 7.8-12.4 %, quads
48.2-53.0 %, depth over 83 m 2.5-4.6 %), so neither the radial primaries nor the market's lanes are the cause.

**The baseline is reproduced.** `both_banks_audit` and `harbour_front_audit`, at 384 and at 2 048, from the same
binary are byte-identical to `burgage-backlanes-2026-11-17/census_off/` except for their timing lines (`diff`, empty),
so the must-hold values of section 5.1 that those two audits print are produced, not cited.

### 1.3 What the numbers say

1. **The premise is half right.** In the shipped town (app, both sizes) triangles are **about 11 % of blocks and 4 % of
   the area**, inside the 8-20 % the real plans show (section 2). What fails close up is not triangles: blocks are
   **small** (median 1 000 m2, 29 m across), **square-ish** (aspect 1.57) and **irregular** (rectangularity 0.70,
   interior angles spread from 45 to 165 degrees), and the large blocks are **five-or-more-sided** (64 % of the block
   area). Only 3.6 % of blocks can hold a 40 m lot.
2. **Triangles are what the deep-plot prototypes made, and the spacing is what makes them.** With `urban_burgage` on,
   triangles rise to **24 %** (compact ones to 16 %), quads fall to 35 %; `MARKET_TOWN_RULES` (spacing 50 m) gives
   21 %. The isolating pass pins it on `parallel_street_spacing`: 60 m alone gives 23.2 % triangles, an 80 m segment
   median alone 11.8 %. **The mechanism is a hypothesis, consistent with the code and the angles but not isolated:**
   rule 4 refuses only near-parallel candidates (within 28.6 degrees) inside the spacing, while oblique ones always
   pass, so a wider spacing selects for oblique streets (the share of corners at 75-105 degrees falls from 46 to 35 %).
   Larger faces also give `lane_pass` more to cut, which this pass cannot separate. PS-0 item 3 tests it directly
   (refuse obliques too, and count).
3. **The angle is the jitter; the depth is the spacing.** Removing both jitters gives 71 % quads, rectangularity
   0.89 and 80 % of corners at 75-105 degrees, but leaves depth unchanged (27 m, 3.0 % over 83 m). Spacing and segment
   length (market town, burgage) give depth (51 m, 13-15 % over 83 m) but more triangles. No rules preset does both.
4. **Removing the jitter destroys the first-sight impression.** Per town, orientation order rises from 0.08 to
   **0.60** (Boeing's phi: 0 is no order, 1 a perfect grid [source: Boeing, "Urban spatial order", Applied Network
   Science 4, 2019, which uses 36 bins, H_grid = ln 4 and H_max = ln 36; I bin plainly from 0 degrees rather than with the paper's half-bin offset]) and bearing entropy falls from 0.98
   to 0.78 of its maximum: every town becomes one grid. This is the measured form of the owner's note: a fix that
   orders the whole town loses the base's character.
5. **So the design must give local order with global variety:** streets square to a direction that is **local**
   (a quarter's own grain, from its road, its river or its market street), not to their random parent, at a spacing of
   two plot depths, while the town's outline and its quarters' different grains keep the base's variety. The real
   plans read the same way (section 2.5, item 3).
6. **The must-hold failures of both prototypes were not caused by block shape** (`STATUS.md`, "Deep plots, second
   prototype": wall / hull and riverside fell the same with and without back lanes). Rectangular blocks will not fix
   them by themselves; they come from a larger footprint (section 3.6).

What this measurement cannot attribute: it reads the finished layout, after the primaries, the plaza's streets,
`lane_pass`, the waterfront back street and the embankment; the rules variants change `grow` only, so their deltas are
`grow`'s, but the base's shares mix every pass. The corner thresholds are judgements (the 10 / 20 / 30 degree
triangle shares bound their effect: 7.2-13.6 %).

---

## 2. What real plans show, by town type

Six plans, all with a clear licence, are in the evidence folders:

| Plan | Type | Folder | Licence |
|---|---|---|---|
| Dinkelsbuehl 1825, Bavarian cadastral town sheet | walled market town, part planned | `burgage-backlanes-2026-11-17/real_plans/` | public domain |
| Wasserburg am Inn 1813, the same survey | river-front town on a loop of the Inn | the same | public domain |
| Noerdlingen 1823, the same survey | organic core, round wall, radial streets | the same | public domain |
| Straubing 1827, the same survey | planted street market (Neustadt, 1218) along the Danube | `parallel-streets-2026-11-17/real_plans/` | public domain |
| Neumarkt in der Oberpfalz 1830, the same survey | planted market-street town, oval wall | the same | public domain |
| Monpazier (bastide, 1284), a modern drawing by Klaus Schaefer | planted grid | the same | CC BY-SA 2.0 de |

The licences were read from each file's Wikimedia Commons page through the Commons API (`SOURCES.txt` in each
folder). The block counts below are **by eye**, one reader, +/- 2 blocks a class
(`parallel-streets-2026-11-17/real_plans/block_counts_by_eye.txt` has the method and the crops); they are
provisional targets, and PS-0 replaces them with traced ones. Metre figures from the scans assume about 0.47 m a
pixel (town sheets at 1:2 500, field sheets of 800 Ruten = 2 334.87 m at 1:5 000 [source: de.wikipedia
"Bayerische Uraufnahme"]; the scan's margins make every metre figure **[inference, +/- 15 %]**).

### 2.1 Planted grid (bastide, Gruendungsstadt)

- Monpazier: a rectangle of about 400 by 220 m, parallel long streets crossed by four transverse streets, houses
  originally of one size [source: Cornell Bastides collection and a French summary, both weak, via
  `MEDIEVAL_TOWN_GROWTH_RESEARCH.md` 2.1]. The drawing shows about 20-25 blocks, all four-cornered except where the
  circuit cuts a corner, long rectangles of aspect about 2-4, plots in strips across each block, back to back
  [source: the drawing; the count is mine].
- Krakow (1257): a grid of evenly sized plots, the square four blocks of it, adapted to older traces in the south
  [source: research 2.1].
- In bastides the square often sets the module the town is divided by [source: research 2.1, Wikiwand, weak].
- **For a generator** [inference]: the most regular type; one module, one orientation; a square of one or more
  blocks. Rare in this world's settlements unless the world has founders (section 7, Q2).

### 2.2 Market-street town (Strassenmarkt)

- Neumarkt 1830: one straight market street (Unterer und Oberer Markt) runs gate to gate, about 35-40 m wide
  [inference]; a street parallel to it on each side about 110-120 m from its centre [inference], so the blocks on
  the market are about 70-85 m deep [inference] with plots running back from the market and from the parallel street,
  back to back; cross lanes at roughly right angles to the market. The **wall is an oval round a straight spine**: the
  outer tier of blocks is cut by the curving wall street into five- and three-cornered pieces. About 28 blocks:
  triangles about 14 %, quads about 60 %, pentagon+ about 25 %; the blocks on the spine have aspect 3-6 [source: the
  plan; counts by eye].
- Straubing 1827: the market place (Theresienplatz / Ludwigsplatz) is a long straight spine parallel to the Danube.
  North of it about eight lanes run at right angles toward the river, about 30-45 m apart [inference], so those blocks
  are long narrow strips (aspect 4-6) with plots fronting the lanes; south of it a back street parallel to the spine
  about 65-70 m away [inference]. About 26 blocks: triangles about 8 %, quads about 73 %, pentagon+ about 19 %
  [source: the plan; counts by eye].
- Dinkelsbuehl 1825: the market (Bretter-, Brot-, Hafenmarkt, then the Weinmarkt) is a long street-market; north of
  it four streets run parallel about 33-38 m apart [inference], blocks two house plots deep; south of it streets
  fan from the market to two south gates, nearly parallel, blocks long and tapering, some to a point near a gate.
  About 30 blocks: triangles about 13 % (all long wedges), quads about 60 %, pentagon+ about 27 % [source: the plan;
  counts by eye].
- Market streets taper at their ends and a spindle-shaped market street is as common as a square [source: research
  2.2, the Victoria County History panel; inference from it].
- **For a generator** [inference]: a spine (the market street, widened, sometimes a spindle) on the main road
  through the market; one or two parallel streets each side at about two plot depths; cross lanes (ribs) at right
  angles; an organic or oval wall round it. Block depth across the spine tier is two plot depths plus a street.
  **Plots are not all deep**: Dinkelsbuehl's north tier is about 15-18 m a plot; Neumarkt's spine tier about 35-40 m.

### 2.3 River-front linear town

- Wasserburg 1813: three or four main streets run along the peninsula roughly parallel to each other and to the
  river's axis; on the outer side of the loop, plots run from a curving street straight down to the bank [source:
  the plan]. Straubing's spine also runs parallel to the river.
- Waterfront plots run street to water with a private quay at King's Lynn [source: research 3.4, weak to medium].
- **For a generator** [inference]: streets parallel to the bank (the stretch's `lay_river_spine` already lays one),
  plots and lanes at right angles to the bank, so the river front fills. Ruling FN's stretch frame already reads the
  town in along / across coordinates; the field this scope needs is that frame's along direction.

### 2.4 Organic core

- Noerdlingen 1823: radial streets to the gates, an inner ring street on the old wall line, irregular blocks of four,
  five and more corners and some three-cornered, plots deep between the radials. Central half by eye: triangles
  about 15-20 %, quads about 45 %, pentagon+ about 35-40 % [source: the plan; counts by eye].
- Freiburg's regular plot module adapted to the winding earlier streets; Krakow's grid adapted to older traces
  [source: research 2.1, 4.2]: a planned extension next to an organic core is ordinary.
- Beresford's planned / organic split is disputed; regular plots appear in organic towns too [source: research 2.1].
- **For a generator** [inference]: the base generator is closest to this type at town scale (radial primaries to
  the gates, a tangle round the market). What it lacks even here is the long, deep block between two radials.

### 2.5 Read across the plans

1. **No plan shows compact triangles as a large share of its blocks.** Three-cornered blocks exist (about 8-20 %),
   but they are long wedges where two streets converge on a gate, or pieces cut off by an oval wall [source: the
   plans; inference].
2. **Planted and market-street towns run about 60-75 % four-cornered blocks** with aspect around 3 or more; Monpazier
   about 90 % [source: the plans, by eye].
3. **Parallelism is local, not town-wide.** Dinkelsbuehl has a north grain (east-west) and a south grain (fanning
   south-east); Neumarkt's grid sits inside an oval wall; Noerdlingen's radials all differ. Each town has one or two
   quarters with a strong grain and a varied outline [source: the plans; inference]. This is what lets parallel
   streets keep the base's first-sight impression (section 3).
4. **Block depth is two plot depths plus a street**, and plot depth varies by quarter (about 15 to 40 m on these
   sheets) [inference].

---

## 3. Design options

Each option is rated on the owner's two tests: **first sight** (the City Viewer's opening frame and the map's town
layer, where the base reads better) and **close scrutiny** (deep zoom, where the base's triangles and fans fail).

### 3.1 Option A: rules only (a preset or retuned `DEFAULT_RULES`)

- **What.** Tune `StreetRules` (jitter, spacing, segment length, pierce) as `MARKET_TOWN_RULES` did; no new code.
- **Measured (section 1.2).** No setting does both jobs: low jitter gives quads but every town turns into one grid
  (phi 0.60); wide spacing gives depth but triangles (21-24 %).
- **Fits the pipeline** trivially (a `Rules` preset, the Generation rules window). **When:** never as the answer;
  kept as the measured baseline. **Cost:** none. **Moves:** every organic town it is applied to.
- **Risks:** both failure modes above are measured, not guessed.
- **First sight:** low jitter fails (one grid per town); wide spacing passes at a glance. **Close scrutiny:** fails
  either way.

### 3.2 Option B: spine and ribs, a third planning branch

- **What.** A new `profile.planning` value beside `organic` and `radial` (`generate_pass` already forks on it, and
  `radial.rs` is the precedent: one pass that lays the network, then the same face detector). The spine is the
  primary through the market, straightened and widened as the market street (a spindle where Q9 says so); one or two
  parallel back streets each side at a module of two plot depths plus a street; cross lanes (ribs) at right angles every
  60-120 m; all clipped to the box, water, slope and `max_rf`, the way `build_radial_streets` clips its spokes. `grow`
  then runs only for the extramural fringe and the far bank.
- **Fits.** Market: the spine **is** the market street (`plaza::build_plaza_with` gives way to a spindle on the
  spine). Walls: unchanged, `wallhug` refits to the lots, so an oval wall round a straight spine (Neumarkt's form)
  comes for free. Bridges: the spine ends at the bridge foot (`site.bridge_pt`), `lay_town_bridge` unchanged. Both
  banks: `grow_far_bank` unchanged (a strip by the bridge). Stretch: for a river town the spine is
  `stretch::lay_river_spine`'s road, already laid along the bank. Villages: untouched (street villages are already a
  one-street plan). Radial: untouched.
- **When.** Market-street towns: a chartered town (1 500 and up) whose market stands on a through primary.
- **Cost.** Medium: one new module (about the size of `radial.rs`), a type rule, platting reuse.
- **Moves.** Every town of that type: streets, blocks, lots, buildings, wall; new worlds only (a save without the
  key stays off).
- **Risks.** **The wall is fixed early:** `grow` raises the circuit at `max(3, floor(0.6 * epochs))` from
  `fortify::built_mass_hull`, and after that a street crosses the ring only within 20 m of a gate
  (`legalize_street`'s wall rule), so a plan laid in one pass must leave its extension to gates; the wall hug then
  refits to the lots (`wallhug`), which is where both deep-plot prototypes' wall / hull rose. The whole intramural town
  becomes one plan: **too regular** for most towns, and the base's varied
  rhythm is lost where the spine dominates. Terrain and river bends break a straight spine into stubs. Housing: the
  plan fixes the street length, so a town that needs more ground cannot grow into it except outside the wall.
- **First sight:** weak for most towns (a regular comb where the base had a tangle; the silhouette survives).
  **Close scrutiny:** strong.

### 3.3 Option C: a grid seeded on the market axis, fitted to terrain and river (bastide)

- **What.** An orthogonal grid of one module (Monpazier's long blocks), oriented on the market square's long axis
  (or the river), warped gently by slope, clipped like B; the square is one module.
- **Fits.** Like B, as a planning branch; the square sets the module (research 2.1, weak).
- **When.** Planted towns only. The world has no founding date or founder to tell a planted town from an organic
  one (`UrbanContext` carries a culture, an age and a population, not a founding), so a rule would be a labelled judgement (Q2).
- **Cost.** Medium. **Moves:** the towns it is chosen for.
- **Risks.** The most "silly": a perfect grid in a world of organic towns reads as a modern plan unless rare and
  justified. First-sight change is the largest of all options.
- **First sight:** fails for anything but the rare planted town. **Close scrutiny:** strongest.

### 3.4 Option D: parallel growth inside `grow` from a local direction field (the hybrid)

- **What.** Keep `grow`'s whole loop (the budget, the growth front, the Clark gradient, exploration, the bridgehead
  rule, the wall's gates, the first-hit T-snap) and change only what section 1 found:
  1. **A direction field.** Each point of the town gets a grain: a direction and a strength, blended from a few
     sources by distance (a tensor field in the sense of Chen et al., "Interactive procedural street modeling", ACM
     TOG 27(3), 2008 [source; I have not re-read it for this scope, the idea is the cited one]): the market street's
     axis (the spine), each approach road (its own bearing near it), the river's along-direction near the bank (the
     stretch frame already computes it, `StretchFrame`), and the wall line late in the run. Two quarters on two roads
     get two grains, as Dinkelsbuehl's north and south do.
  2. **Headings from the field, not the parent.** A densification branch takes the field's across-direction or
     along-direction (whichever is nearer its parent's perpendicular), with a small jitter that falls with the
     field's strength; an exploration continuation bends toward the along-direction.
  3. **A module spacing.** Near-parallel streets at least one block module apart (two plot depths plus a street:
     about 60-90 m by quarter, section 4), and oblique candidates refused where the field is strong, which should
     close what the isolating pass found: spacing alone made the prototypes' triangles (section 1.3 item 2; the
     mechanism is PS-0's to confirm).
  4. **A back-street candidate.** A share of tries offsets an existing street by one module along the across-direction
     (a parallel copy, clipped by the first-hit rule), so blocks close as long quads; ribs come from the branch rule.
  5. **An organic core** (Q3): inside the first growth front (`0.38 * max_rf`, `grow`'s own epoch-1 radius) the field
     is weak, so the core keeps today's tangle and the regular quarters read as later, planned growth.
- **Fits.** Market: the field's spine source is the market street (and B's spindle can be added as D's market-street
  type). Walls, bridges, both banks, harbour: unchanged code paths; the wall hug refits to the lots. Stretch: the
  field reads the stretch frame, so a river-front town's streets run along its river and its lanes to the water.
  Villages: untouched (`road_reach` path). Radial: untouched. Burgage: D replaces `back_lane_pass` and `wide_spacing`
  (Q7).
- **When.** Every organic town of 1 500 and up, with the type rule (section 4) setting the field's sources and
  strength: river-front, market-street, organic core; planted only if Q2 says so.
- **Cost.** Medium to high: a field module, three changes in `grow`/`legalize_street`, the type rule, platting depth
  by quarter, and the measurement passes. No new face or block code.
- **Moves.** Every organic town of 1 500 and up with the switch on: streets, blocks, lots, buildings, wall,
  districts, and so the census figures. Villages and the radial plan are untouched by control flow.
- **Risks.** (0) **The wall:** the circuit rises at 0.6 of the epochs from `fortify::built_mass_hull` and later
  streets cross it only at gates, so a quarter's grain that reaches the wall after it rises is cut there; the wall hug
  refits to the lots and its wall / hull is a must-hold that both prototypes broke. (a) Field strength too high
  reproduces the no-jitter failure (phi 0.60): the first-sight bar FS-2
  catches it. (b) Housing (section 3.6). (c) The first-hit snap still lands a street on an oblique neighbour where two
  grains meet; that seam is where real towns have their wedges (section 2.5 item 1), and the bar allows about 10 %.
  (d) RNG: a new draw sequence for the switched path; off stays the reference's by control flow.
- **First sight:** strongest of the options that fix anything: the outline, the growth front, the market gradient
  and the quarter-to-quarter variety are the base's. **Close scrutiny:** strong (long quads in each quarter, deep
  blocks), with seams.

### 3.5 Option E: a post-pass that straightens the existing network

- **What.** After `grow`, snap near-perpendicular junctions to 90 degrees and straighten near-collinear chains.
- **Fits** as a pass before `lane_pass`. **Cost:** low. **Moves:** geometry only, not topology.
- **Risks / rating.** It cannot make a block deeper or turn a pentagon into a long quad; it makes the base look
  drafted. **First sight:** passes. **Close scrutiny:** fails (depth 29 m stays). Dropped.

### 3.6 Common to every option: the ground deep plots need

A feasibility sum, every term cited or labelled:

- A lot houses one household of 5.2 (`build_buildings`; judgement change 4), so a town of 20 000 needs about 3 850
  lots.
- Today's lots are 7.8 m by 10.6 m median (judgement section 1 item 5). At the plot bars (depth / frontage 3 or more,
  10 % over 40 m) a median lot is about 8 m by 24 m or more, about 2.3 times today's area [inference].
- Settled area today for a town of 20 000: 79-106 ha in the reproduced census (`both_banks_audit_384.txt`, "plots #"
  rows); the research's law gives 0.12 N^0.714 = 141 ha [source: research rule 1, Cesaretti et al.]; settled / law
  median 0.64.
- The growth front is a disc of `max_rf = sqrt(pop * 21)` = 648 m for 20 000 [source: `generate_pass`, via the
  judgement], 132 ha; a one-bank river town has about half of it unless stretched (Ruling FN).
- Lots alone at 2.3 times today's area need about 85 ha for 20 000 [inference]; with streets and yards the town needs
  roughly 1.2-1.5 times today's footprint, which matches what both prototypes measured (x1.20 median, p90 x1.68).

So **deep plots fit the law's band (0.65-1.0) but not every site**: a one-bank river town of 20 000 that is not
stretched has about 66 ha of front for about 100 ha of town. That is a likely reason housed p10 fell in both prototypes [inference: the prototypes' per-town housing was not split by bank or front]. It is
the same for every option; D is the only one that lets a short town keep growing outward by the base's own rule.
PS-0 makes it a go / no-go per town.

### 3.7 The options side by side

| | A rules | B spine and ribs | C bastide grid | **D field-guided growth** | E straighten |
|---|---|---|---|---|---|
| first sight (opening frame, town layer) | fails (low jitter) / passes (wide) | weak | fails except rare towns | **keeps the base** | passes |
| close scrutiny (deep zoom) | fails | strong | strongest | **strong, with seams** | fails |
| deep blocks (83 m+) | 13-15 % with triangles | yes | yes | yes, by module | no |
| fork point | `Rules` | new `planning` branch | new `planning` branch | `GrowOpts` flag + field | a pass after `grow` |
| cost | none | medium | medium | medium-high | low |
| risk of "silly" regularity | measured (phi 0.60) | high | highest | bounded by FS-2 | low |

---

## 4. Recommendation

**Option D, behind one new switch, `urban_parallel_streets`, with B's market spine as one of D's field sources and
C left out of the first build.**

- **Switch semantics (`DECISIONS.md` section 7p).** `false` in `cartalith_engine::WorldParams::defaults` (the reference
  boundary: every JS-captured golden unmoved by control flow) and `false` for a save without the key. In
  `cartalith_godot::params::defaults` it **ships `false` until the owner rules at PS-7**, the same exception
  `urban_burgage` and the river stretch's first prototype took (`496c7a2d`): the owner asked to see it measured and
  shown first. If the owner approves, `params::defaults` turns it `true`, and the superseded pieces of `urban_burgage`
  are scrubbed, not kept as a second path (section 7p item 2; Q7). New params need their `PARAMS` and `JS_PATHS` rows
  and the assistant registry row, as `urban_burgage` had.
- **The town-type rule** (labelled judgement; section 2's types; Q1). For an organic-plan town of 1 500 and up, not a
  street village:
  - **river-front** where the site is a real-water river site and the town is stretched (Ruling FN) or its market is
    within two street widths of the bank: the field's along-direction is the river's;
  - **market-street** where a primary runs through the market square: the spine is that primary;
  - **organic core** in every type, inside `0.38 * max_rf` (Q3);
  - otherwise the approach roads alone set the grains (a town of several quarters).
  - **Era and size:** a town under 5 000 gets one grain (one street market or one riverside), a larger one up to its
    number of approach roads [judgement]; the settlement's age (`settlement_age`) sets the core's share: the older the
    town, the larger its organic core [judgement, after research 2.1's planned-extension cases].
- **Plot depth by quarter** (platting, PS-5): the spine and river-front tier at the deep target (`burgage::DEPTH_M`,
  42 m; core 30 m), other quarters at today's targets; module = two plot depths plus a street.
- **Why not B alone:** B fixes close scrutiny but loses the first sight for most towns; as a field source inside D it
  gives the spine without making the whole town a comb. **Why not C now:** no world fact says which towns were planted.

---

## 5. Bars

Every figure is measured by the census world (section 1's command), at **384 x 288 and 2 048 x 1 311**, switch off
against on, three seeds, villages on. "Today" is `origin/main` at `e01972ad`. The rows printed by `both_banks_audit`
and `harbour_front_audit` were reproduced at both sizes (section 1.2); none of the table's rows comes from another
audit, but the "villages and radial" row is a design requirement, not a figure.

### 5.1 Must-hold (no worse than today)

| bar | 384 today | 2 048 today |
|---|---|---|
| towns (1 500+) housed p10 | 94 % | 85 % |
| towns under 80 % housed | 1 | 3 |
| villages under 80 % housed | 49 | 21 |
| wall / hull length p90 (walled towns 1 500+) | 1.11 (max 1.14) | 1.09 (max 1.21) |
| market squares outside the wall | 5 | 1 |
| unbuilt patches inside the wall over 1 ha (56 m reach for deep plots, also printed at 40 m) | 1 | 0 |
| riverside: houses within 2 street widths of the bank, river towns 1 500+ | 4.3 % | 3.7 % |
| buildings in water (channel, sea or lake, water_poly) | 0 | 0 |
| wall rings in open water | 0 | 0 |
| village greens within 60 m of the market point | 629 of 662 | 676 of 684 |
| stretched towns refused by Ruling FQ | 5 | 6 |
| villages and radial towns | byte-identical layouts with the switch on (control flow) | same |

### 5.2 Plots (Ruling FY; judgement change 4)

- lot depth / frontage, median, four-cornered lots of towns 1 500+: **3 or more** (today 1.42 / 1.43);
- lots over 40 m deep: **10 % or more** (today 0.4 / 0.4 %), with the census's unbuilt and streetless reaches at
  **56 m** (`burgage::UNBUILT_REACH_M`, Q8 of the judgement row, kept);
- settled area / 0.12 N^0.714, towns 5 000+, median: 0.65-1.0 (today 0.64 / 0.65).

### 5.3 Close scrutiny: block shape (targets from the real plans, provisional until PS-0 traces them)

**Measured on which blocks:** the blocks outside the organic core (more than `0.38 * max_rf` from the market) of towns
of 1 500 and up whose type gives them a grain (river-front or market-street, section 4). The organic core is held to
"no worse than today" on each row instead, since Noerdlingen, the organic case, has only about 45 % quads.

| bar | today (384) | target | source of the target |
|---|---|---|---|
| quads, by count | 50.3 % | **65 % or more** | planted and market-street plans 60-75 %, Monpazier about 90 % (section 2.5) |
| compact triangles (3 corners, aspect under 2) | 7.0 % | **5 % or less** | no plan shows them as a share; real three-cornered blocks are long wedges |
| all triangles | 10.7 % | **15 % or less** | the planned and market-street plans' 8-15 % (wedges at gates and walls; Noerdlingen about 15-20 %) |
| block aspect, median | 1.57 | **2.5 or more** | spine and rib blocks 3-6, Monpazier 2-4 |
| rectangularity, median | 0.70 | **0.85 or more** | no jitter reached 0.89; planned blocks are near rectangles [inference] |
| depth across 83 m or more | 3.6 % | **15 % or more** | needed for 10 % of lots over 40 m from both long sides [arithmetic: 0.48 waist] |

### 5.4 First sight: the base's character (the owner's note, 2026-11-17)

A fix that passes 5.3 but fails any of these fails the milestone:

- **FS-1 silhouette.** Per town, the settled footprint (20 m cells within one cell of a building, the census's own
  measure) on against off: area ratio **0.9-1.4** (deep plots need more ground, section 3.6), and **85 % or more** of
  the off footprint's cells still settled; wall / hull p90 as 5.1.
- **FS-2 town-scale variety.** Per town, orientation order phi: **median 0.25 or less and p90 0.45 or less** (today
  0.08 / 0.25 at 384; the no-jitter variant's 0.60 / 0.83 is the failure this bar exists for); bearing entropy / max
  median **0.90 or more** (today 0.98, no jitter 0.78), and **per town, phi on minus phi off at most 0.20** (the bar
  is anchored to the base, because the owner's note is about the base's impression). This replaces the judgement's
  "bearing entropy not below today's" (Q4), which parallel streets cannot meet by construction. PS-0 may tighten the
  caps from the traced plans but may **not loosen them**: a traced plan's phi above these caps is a reason to ask the
  owner, not to raise the bar toward the no-jitter failure.
- **FS-3 street rhythm.** Node degree 4 at **15 % or less** of nodes (today 10 %; a grid raises it), dead ends 8-15 %
  (today 11 %), segment length median within 0.8-1.5 times today's 24 m.
- **FS-4 visual gate, first sight.** The City Viewer's opening frame and the map's town layer (fit zoom) of the same
  **14 towns** as the deep-plot frames (`burgage-backlanes-2026-11-17/frames/`), off and on side by side, plus the same
  towns shown one at a time in random order: **12 of 14** judged "the same town, same character" at first sight
  (owner, after a verifier's pre-screen; Q8).
- **FS-5 visual gate, close scrutiny.** The same towns at **x4 and x16** of the opening frame's zoom and at the
  deep-zoom frame the owner judged: **12 of 14** judged better "under scrutiny" (rows of plots on long blocks, back
  lanes or yards behind), with any town that reads as a modern grid counted as a failure of FS-4 too.

---

## 6. Milestones

Each milestone states its bars; every one runs the census at 384 and 2 048 and checks off is byte-identical
(`both_banks_audit` and the five audits, `diff` empty but for timing).

- **PS-0, measure-first spike** (no product code; scratch worktree; findings into this scope).
  1. Trace the street centrelines of five plans (Dinkelsbuehl, Neumarkt, Straubing, Noerdlingen, Monpazier) into
     polylines, **20 blocks or more each**, and run `block_shape_audit`'s measures on them: quads, triangles, aspect,
     rectangularity, phi, parallel share. Replace 5.3's and FS-2's provisional targets. Calibrate one sheet's scale on
     a sourced length before quoting metres.
  2. Per census town, the dry buildable ground inside the growth front (box, water, slope, `max_rf` or the stretch
     frame) against the ground the plot bars need (section 3.6). **Go** if 90 % or more of towns 5 000+ fit at both
     sizes; otherwise stop and put Q5 / Q6 to the owner.
  3. A throwaway prototype of D's heading rule and module spacing (items 2 and 3 only, one grain per town from the
     market's primary) on eight towns a seed at 384. **Go** if quads 65 % or more, phi per town median 0.25 or less and
     housed p10 90 or more on those towns.
  4. Re-run the baseline from the tree of the day (all five audits at both sizes) before any comparison.
- **PS-1, the field and the type rule** (behind the switch; no growth change yet). `cartalith_urban::field` (new):
  sources, blend, strength; the type rule. Bars: every town 1 500+ gets a type and a field; a probe draws the field
  over the City Viewer for 14 towns; off byte-identical; unit tests pin the blend on hand-built fixtures (literal
  directions, mutation-tested).
- **PS-2, field-guided branching** (`grow`, `legalize_street`): headings, module spacing, the oblique refusal.
  Bars: 5.3 quads 60 % or more and rectangularity 0.80 or more; FS-2 and FS-3; 5.1 housed p10 within 3 points of
  today.
- **PS-3, the back-street candidate and the market spine** (D item 4; B's spine as a field source; a spindle market on
  the spine with `urban_market_scale`). Bars: 5.3 in full; depth across 83 m or more 15 %; FS-1.
- **PS-4, river-front towns**: the stretch frame as a field source, lanes to the water. Bars: riverside 4.3 / 3.7 %
  or better; stretched towns refused no more than today; buildings in water 0.
- **PS-5, platting by quarter**: depth targets by tier (42 m / 30 m core), lots from the long sides; the census's 56 m
  reach. Bars: 5.2 in full.
- **PS-6, integration**: walls (wall / hull), both banks, harbour, wall generations, villages and radial
  byte-identical, `cargo test --workspace` and the shell parse checks. Bars: all of 5.1 at both sizes.
- **PS-7, the visual gates and the owner's decision**: FS-4 and FS-5 on 14 towns, frames at several zooms in
  `cartalith-native/docs/evidence/`; the owner decides on, adjust or drop; on, `params::defaults` turns true and the
  `urban_burgage` remnants are scrubbed (Q7).

---

## 7. Owner questions (defaults in brackets)

1. **Which towns get parallel streets?** The type rule of section 4: river-front, market-street, the rest by their
   roads' grains, an organic core in all. [as section 4]
2. **Planted grids (bastides, Gruendungsstaedte)?** The world has no founding fact to choose them by. [not in this
   build; a later row behind a culture or era choice]
3. **Keep an organic core** inside the first growth front, so the regular quarters read as later growth? [yes, with
   the core's share growing with the town's age]
4. **Replace the judgement's "bearing entropy not below today's"** with FS-2 (per-town phi cap, per-quarter order)?
   It cannot hold for any parallel-street design. [yes]
5. **Backland households.** Every lot houses one household of 5.2, so deep lots need more ground (section 3.6).
   Count a rear dwelling on lots over 30 m deep as a second household (research 2.6: backland fills over time)? [no;
   decide after PS-0 item 2]
6. **A larger footprint**, up to 1.4 times today's per town (inside the law's 0.65-1.0)? [yes]
7. **`urban_burgage`.** Fold its depth targets into `urban_parallel_streets` and scrub `back_lane_pass`,
   `GrowOpts::wide_spacing` and `GrowOpts::whole_budget` at PS-7 if unused; until then it stays in and off. [yes]
8. **Who judges FS-4 and FS-5?** [the owner, on 14 towns, after a verifier's pre-screen]
9. **A spindle market street** on a market-street town's spine (judgement change 6 allowed one)? [yes, sized by
   `urban_market_scale`'s rule]
10. **Must-hold bars strict** (no rise in wall / hull, no fall in riverside or housed p10)? [yes, strict]

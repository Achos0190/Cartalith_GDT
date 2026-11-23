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
`cartalith-native/docs/STATUS.md`'s answer (CLAUDE.md, "One place holds state"). **Nothing was built when this scope was written** (PS-1 and PS-2 have been built since: sections 10 and 14; PS-3 was measured and not built: section 15; PS-4 likewise: section 16). The only code
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
(`parallel-streets-2026-11-17/real_plans/block_counts_by_eye.txt` has the method and the crops). **PS-0 traced four
of these plans and measured them with the census's own measures; the traced figures are section 8.1 and replace the
by-eye shares as targets.** Metre figures here assumed about 0.47 m a pixel; **PS-0 calibrated the Noerdlingen sheet at
0.573 m a pixel** on its wall walk (2.6 km [source: de.wikipedia "Stadtmauer Noerdlingen", read 2026-11-17]) and
applies it to the other Bavarian sheets as an **[inference, +/- 15 %]**, so the metre figures in this section read
about 20 % low.

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
PS-0 makes it a go / no-go per town. **Superseded in part by section 9.2-9.3 (Ruling GC): rear households and a footprint without a fixed 1.4x ceiling replace the single 1.2-1.5 figure.**

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
- settled area / 0.12 N^0.714, towns 5 000+, median: 0.65-1.0 (today 0.64 / 0.65);
- **needle lots (section 8.4's definition; Ruling GA)**: **5 % or less** of the lots of towns 1 500+ at PS-5 and
  **never above today's 11 %** (11.1 / 11.4 % at 384 / 2 048) at any milestone; the scale-free taper leg **no worse
  than today's 5.6 %**; needle buildings **0.6 % or less** (today 0.56 %). Also reported at the 3.3 m and 5.0 m width
  floors, because the 4.15 m floor rests on a weak scale. Platting owns it (PS-5): deep lots are cut only where the
  block is near-rectangular. A rear household's building (section 9.2) is held to the same definition;
- **rear households (Ruling GC, section 9.2)**: rear dwellings only on lots **over 30 m deep**, at most one per lot,
  and no more than **40 % of the lots** of any town carry one [judgement, so deep lots stay the minority the traces
  show: 10 % over 40 m].

### 5.3 Close scrutiny: block shape (targets replaced 2026-11-17 by PS-0's traced ones, section 8.1; Ruling GC)

**Measured on which blocks:** the blocks outside the organic core (more than `0.38 * max_rf` from the market) of towns
of 1 500 and up whose type gives them a grain (river-front or market-street, section 4). The organic core is held to
"no worse than today" on each row instead, since Noerdlingen, the organic case, has only about 45 % quads.

| bar | today (384) | target | source of the target |
|---|---|---|---|
| quads, by count | 50.3 % | **60 % or more** (PS-2's go bar stays 65 %) | traced market-street towns 55-59 %, Straubing's Neustadt 95 % (section 8.1) |
| compact triangles (3 corners, aspect under 2) | 7.0 % | **5 % or less** | traced compact triangles 0 %; real three-cornered blocks are long wedges |
| all triangles | 10.7 % | **15 % or less** | traced 0-14 % (wedges at gates and walls) |
| block aspect, median | 1.57 | **2.0 or more** | traced 1.9-3.0; the first 2.5 was above two of three traces |
| rectangularity, median | 0.70 | **0.75 or more** | traced 0.63-0.88; the first 0.85 was above every traced market-street town |
| depth across 83 m or more | 3.6 % | **15 % or more** (the 83 m target, unchanged; the traces' 32-40 % are upper bounds) | needed for 10 % of lots over 40 m from both long sides [arithmetic: 0.48 waist] |

### 5.4 First sight: the base's character (the owner's note, 2026-11-17)

A fix that passes 5.3 but fails any of these fails the milestone:

- **FS-1 silhouette.** Per town, the settled footprint (20 m cells within one cell of a building, the census's own
  measure) on against off: area ratio **0.9 or more, with no fixed upper cap** (Ruling GC replaced the 1.4 ceiling:
  growth beyond 1.4x, far-bank growth included, is allowed wherever the ground gate of section 9.3 passes, and is
  bounded by the law's 1.0 ceiling on settled area / 0.12 N^0.714 and by wall / hull below), and **85 % or more** of
  the off footprint's cells still settled; wall / hull p90 as 5.1 (strict; footprint growth stops before it rises).
- **FS-2 town-scale variety, by town type (Ruling GC).** Per town, orientation order phi. **Market-street towns
  (section 4's type): phi per town at most 0.35** (owner; the traced Neumarkt and Dinkelsbuehl sit at 0.29-0.32), and
  phi on minus phi off at most **0.30** [judgement: today's 0.08 plus the cap, so the delta cannot undercut the cap;
  Q12]. **Every other type, river-front and organic-core towns included: median 0.25 or less and p90 0.45 or less,
  and phi on minus off at most 0.20, as before.** **Planted grids stay out** (Q2): no type, no field, no cap. The
  no-jitter variant's 0.60 / 0.83 is the failure this bar exists for (today 0.08 / 0.25 at 384); bearing entropy /
  max median **0.90 or more** for every type (today 0.98, no jitter 0.78; Q12 asks whether market-street towns may
  fall to 0.85 if 0.35 cannot meet 0.90). The delta bars are anchored to the base, because the owner's note is about
  the base's impression. This replaces the judgement's
  "bearing entropy not below today's" (Q4), which parallel streets cannot meet by construction. PS-0 may tighten the
  caps from the traced plans but may **not loosen them**: a traced plan's phi above these caps is a reason to ask the
  owner, not to raise the bar toward the no-jitter failure. PS-0 asked (Q11) and the owner loosened exactly one cap,
  for exactly one type (Ruling GC); no other cap moves without a further ruling.
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
- **PS-1, the field and the type rule** (behind the switch; no growth change yet; Ruling GC: first after this
  re-scope). `cartalith_urban::field` (new): sources, blend, strength, and the **type rule with the Ruling GC split**:
  each town 1 500+ gets one of market-street, river-front, organic-core or **no grain** (a planted grid is never
  assigned; no founding fact exists, Q2), and the type picks FS-2's cap (0.35 market-street, 0.25 / 0.45 / delta 0.20
  otherwise). Also the ground gate's measurement (section 9.3) as a pure function over `Site`, so PS-2 and PS-5 read
  one answer. Bars: every town 1 500+ gets a type, a field and a ground figure; a probe draws the field over the City
  Viewer for 14 towns; the ground recount of section 9.3 on all 28 / 22 towns of 5 000+ at both sizes (**go: 90 % or
  more fit the section 9.3 rule on the market's bank plus the bridge-strip far bank**, else Q13); off byte-identical
  (diff empty but for timing); unit tests pin the blend and the type rule on hand-built fixtures (literal
  directions, mutation-tested). **Before / after frames:** the field drawn over the same 14 towns, off against on
  (the layout is unchanged, so the frames show the field only).
- **PS-2, back streets and cross lanes** (re-scoped 2026-11-17, section 9.1; `grow`, `legalize_street`,
  `Graph::add_street`, `lane_pass`). **Step 0, a throwaway measurement on the same 24 towns of PS-0 before any product
  code (a scratch worktree, `.txt` only):** back-street candidate plus closing cross lanes at gain 0.6. **Go:** quads
  outside the core **60 % or more** (the 65 % PS-0 go bar was missed at 44-54 %; the traced 55-59 % set 60 %), compact
  triangles 5 % or less, all triangles 15 % or less, housed p10 **90 or more**, FS-1's 85 % kept share in **28 of 35
  towns or more** (PS-0: 17 of 35), phi per town within Ruling GC's caps by type. **No-go:** stop and report; do not
  build a third heading variant. Then the product build behind the switch. Bars: 5.3 quads 60 %, aspect 2.0,
  rectangularity 0.75, triangles as written; FS-2 by type and FS-3; 5.1 housed p10 within 3 points of today, wall /
  hull p90 no higher than 5.1's row (the framed setting broke it at 1.12 / 1.13). **Before / after frames** (Ruling
  GA): the 14 towns, opening frame, x4 and x16, off and on, one town a seed in full size.
- **PS-3, the market spine and the back-street tier** (B's spine as a field source; a spindle market on the spine
  with `urban_market_scale`; the back street's offset and the module depth tuned). Bars: 5.3 in full, **depth across
  83 m or more at 15 % or more**; FS-1 (area ratio 0.9 or more, 85 % kept); wall / hull as 5.1. **Frames** as PS-2.
- **PS-4, river-front towns**: the stretch frame as a field source, lanes to the water, **far-bank growth past 1.4x
  where section 9.3's gate passes** (`grow_far_bank`'s `FAR_BANK_MAX_RATIO` read, not raised without a ruling). Bars:
  riverside 4.3 / 3.7 % or better; stretched towns refused no more than today (5 / 6); buildings in water 0;
  stretch-water audit flags 0. **Frames** as PS-2, river towns first.
- **PS-5, platting by quarter and rear households**: depth targets by tier (42 m spine and river-front / 30 m core);
  lots from the long sides; **a rear household on every lot over 30 m deep under section 9.2's order of filling**;
  the census's 56 m unbuilt reach. Bars: 5.2 in full, **needle lots 5 % or less (never above 11 % at any
  milestone), taper leg 5.6 % or less, needle buildings 0.6 % or less**; towns housed by the 5.1 rows; rear
  dwellings only on lots over 30 m deep and on at most 40 % of the lots. **Frames** as PS-2, plus needle crops of the
  same 260 m window as PS-0's.
- **PS-6, integration**: walls (wall / hull), both banks, harbour, wall generations, villages and radial towns
  byte-identical, planted-grid-free (no town gets a grid type), `cargo test --workspace` and the shell parse checks.
  Bars: all of 5.1 at both sizes; off byte-identical; the five audits' summaries diffed against `census/base/`.
  **Frames:** a final contact sheet per seed, off and on.
- **PS-7, the visual gates and the owner's decision**: FS-4 and FS-5 on 14 towns, frames at several zooms in
  `cartalith-native/docs/evidence/`; the owner decides on, adjust or drop; on, `params::defaults` turns true and the
  `urban_burgage` remnants are scrubbed (Q7).

**Every milestone reports with before / after frames** (owner, Ruling GA), the same 14 towns each time, off beside on,
and **the milestone is not reported done without them**; a milestone whose change is invisible (PS-1) says so and
shows the field instead.

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
5. **Backland households.** **Answered, Ruling GC: yes**, a lot over about 30 m deep holds a rear household
   (section 9.2).
6. **A larger footprint.** **Answered, Ruling GC: yes, and beyond 1.4x where the ground allows, far-bank growth
   included** (section 9.3).
7. **`urban_burgage`.** Fold its depth targets into `urban_parallel_streets` and scrub `back_lane_pass`,
   `GrowOpts::wide_spacing` and `GrowOpts::whole_budget` at PS-7 if unused; until then it stays in and off. [yes]
8. **Who judges FS-4 and FS-5?** [the owner, on 14 towns, after a verifier's pre-screen]
9. **A spindle market street** on a market-street town's spine (judgement change 6 allowed one)? [yes, sized by
   `urban_market_scale`'s rule]
10. **Must-hold bars strict** (no rise in wall / hull, no fall in riverside or housed p10)? [yes, strict]

11. **Market-street towns' order cap.** **Answered, Ruling GC: about 0.35 per town**; other types keep 0.25 / 0.45 /
    0.20; planted grids stay out (section 5.4).
12. **New, from folding Ruling GC in.** (a) A market-street town's phi **delta** (on minus off): 0.30 so it cannot
    undercut the 0.35 cap? [yes] (b) If 0.35 cannot meet the bearing-entropy floor of 0.90, may market-street towns
    use 0.85? [no, unless PS-2's step 0 shows it is the binding bar; then ask]
13. **New.** Rear households: the **40 % ceiling** on lots carrying one, and filling **front lots first** (section
    9.2)? [yes to both]. If PS-1's ground recount finds under 90 % of towns fitting even with rear households and
    far-bank growth: **let those towns keep today's layout** (no parallel grain), or raise the far-bank cap? [keep
    today's layout]

---

## 8. PS-0 result (2026-11-17, Ruling GA: every default)

**Verdict for PS-1: adjust.** The heading rule turns the streets (aligned street length outside the core 40 % to
67 %) but not the blocks: quads stay at 44-54 % against the 65 % go bar at every one of six settings. On the
market's bank the ground for a 1.4x deep-plot footprint is there for only a quarter of the towns of 5 000+. And the
deep-plot prototype's "needles" are measured: 2.5 times the shipped town's share, 8 times the real plan's. PS-1
(the field and the type rule, no growth change) is still needed by any variant. It should start only after the owner
answers Q5 and Q6 (the ground no-go) and Q11 (below). PS-2's design must change as section 8.6 says. Nothing ships:
no product behaviour changed, and the scratch code is kept only as `.txt`.

Evidence: `cartalith-native/docs/evidence/parallel-streets-ps0-2026-11-17/` (its `summary.txt` lists every file).
"Measured" below means those runs. Census world: seeds 483920, 912345 and 246371, 800 km, villages on, the app's
`params::defaults`, 384 x 288 and 2 048 x 1 311. Release test binary from `origin/main` `cd23c83a` plus the scratch
code (`scratch_code/prototype_and_audit.diff.txt`).

### 8.0 The baseline, re-run (item 5)

The five audits (`both_banks`, `town_housing`, `riverbank_gap`, `harbour_front`, `stretch_water`) at both sizes, from
`cd23c83a`, are byte-identical to `burgage-backlanes-2026-11-17/census_off/` apart from their timing lines (`diff`
empty, ten files; `census/base/`). The off path of the scratch build is therefore the shipped path. Section 5.1's
"today" column stands as written. `ps0_audit`'s off columns reproduce section 1.2's block figures exactly: quads
50.3 %, triangles 10.7 %, aspect 1.57, rectangularity 0.70, depth 28.6 m, 3.6 % of blocks 83 m or more across.

### 8.1 Traced real plans (item 1)

**Method.** Each street centreline was read off the scan at 1-2x, using a 50 px grid, as a polyline with a width. The
traces are kept as `traced/trace_*.txt`. A Python port of the census's measures then planarises the centrelines,
extracts the faces and insets each face by half its streets' widths, which gives the block as built, like
`build_blocks`. It keeps blocks of 200 m2 and up, and measures them with `ps_corners` (6 m merge, 20-degree turn),
`ps_mabr` and the 36-bin bearing phi. Monpazier is extracted automatically from Schaefer's figure-ground instead: a
1 px closing removes the plot lines, then the black components are traced and Douglas-Peucker simplified at 1.5 m.
The port agrees with the Rust audit on one dumped town (483920 #0): the corner-count histogram matches block for
block except one 5-cornered block, and the aspect and rectangularity medians agree within 0.5 %. Every trace is
drawn over its scan in `traced/*_trace.jpg`; I looked at each overlay and corrected Dinkelsbuehl's south-west
boundary, which first ran through built ground.

| plan | blocks | quads | triangles (compact) | pentagon+ | aspect median | rectangularity median | depth across median | 83 m+ | phi (whole trace) |
|---|---|---|---|---|---|---|---|---|---|
| Neumarkt 1830, market street | 20 | 55 % | 0 (0) % | 45 % | 1.92 | 0.74 | 79 m * | 35 % * | 0.32 |
| Straubing 1827, Neustadt | 20 | 95 % | 0 (0) % | 5 % | 1.97 | 0.88 | 64 m * | 40 % * | 0.80 |
| Dinkelsbuehl 1825 | 22 | 59 % | 14 (0) % | 27 % | 3.03 | 0.63 | 57 m * | 32 % * | 0.29 |
| Monpazier, planted grid | 32 | 69 % ** | 0 (0) % | 31 % ** | 2.70 | 0.96 | 17 m *** | - | 0.91 |
| shipped town, today (384, towns 1 500+) | 5 648 | 50 % | 11 (7) % | 39 % | 1.57 | 0.70 | 29 m | 3.6 % | 0.08 per town |

`*` These are upper bounds. The traces keep streets and named lanes and omit private passages, so the true blocks
are smaller. Section 5.3's depth targets are not reset from them. Metres are at 0.573 m a pixel, calibrated on the
Noerdlingen sheet; applying that figure to the other three sheets is an inference, +/- 15 %. `**` Monpazier is a grid
and its "pentagons" are pixel noise on the outline after closing, so its real quad share is higher. `***` These are
half-blocks: the drawing's eaves gaps (andrones) split each block lengthwise. Monpazier's metres use the weakly
sourced 400 m long side (about 0.83 m a pixel).

**Noerdlingen is excluded.** Its trace is schematic: radial and sector streets were placed from the overview, and at
the centre-crop zoom several of them do not lie on a real street. Its figures (21 blocks, quads 57 %, phi 0.11) are
kept in `traced/` for the record, but they set no target. The organic core keeps 5.3's "no worse than today" rule.
**Lots were not traced on the Bavarian sheets**: the plot lines could not be read reliably at the scans' resolution.
The lot thresholds come from Monpazier's 301 plots (automatic, 30 m2 and up): depth / width median 1.70, p95 3.5,
p99 6.0; width p5 4.15 m (5 px at a weak scale). By eye, Dinkelsbuehl's Schmiedgasse strips run about 5.7 m by 24 m
(ratio about 4.2).

**Replacement targets for 5.3** (for the blocks outside the core of grain-type towns):

- quads 60 % or more (traced market-street towns 55-59 %, the planted Neustadt 95 %; the 65 % PS-0 bar stays as
  PS-0's);
- triangles 15 % or less, compact triangles 5 % or less (traced 0-14 % and 0 %);
- aspect median 2.0 or more (traced 1.9-3.0; the scope's 2.5 was above two of three traces);
- rectangularity median 0.75 or more (traced 0.63-0.88; the scope's 0.85 was above every traced market-street town);
- depth across 83 m or more: 15 % or more, unchanged (the traces' 32-40 % are upper bounds).

**FS-2, and new owner question Q11.** The traced plans' phi is above FS-2's caps: 0.29-0.32 for the market-street
towns, 0.80 for Straubing's Neustadt, 0.91 for Monpazier. Each trace is a partial town with lanes omitted, so these
overstate a whole town's order. As section 5.4 requires, the caps are **not** loosened. **Q11: is a per-town phi
median near 0.3, which is Neumarkt's and Dinkelsbuehl's, acceptable for a market-street town?** [default: no; keep
0.25 / 0.45 and the 0.20 per-town delta]

### 8.2 Ground feasibility (item 2): no-go on the market's bank

**Method.** Per town of 5 000+, I counted the dry buildable ground inside the growth front on 10 m cells, at the
layout's own site (scratch code in `generate_pass`, before `grow`). A cell counts if it lies inside the 40 m box
margin, `!site.is_water`, `site.slope <= 0.34`, and `site.market_dist <= max_rf`, which uses the stretch frame where
the town is stretched. I counted it twice: on the market's bank only, and on all banks. Against it, today's settled
footprint (the census's 20 m cells within one cell of a building) is counted **inside the same region**, from the
layout dumps. A town fits if the buildable ground is 1.4x (and 1.2x) that settled area or more.

| 384, towns 5 000+ (28) | fits at x1.2 | fits at x1.4 | buildable / settled in region: min, p10, median |
|---|---|---|---|
| market's bank | 23 (82 %) | **7 (25 %)** | 0.97, 1.16, 1.36 |
| all banks | 28 (100 %) | 27 (96 %) | 1.37, 1.67, 2.11 |

The market's bank decides. "All banks" is an upper bound, because a town grows on its far bank only in the bridge
strip (`grow_far_bank`). Of the 8 unstretched towns, 3 fit at x1.4 on the market's bank. **No-go by the scope's own
rule** (90 %), so Q5 (backland households) and Q6 (a larger footprint) go to the owner before PS-2. The like-with-like
count was made at 384 only. At 2 048 only the first, unequal comparison was run (settled over the whole town), and it
agrees in direction: market's bank 1 of 22 at x1.4, all banks 21 of 22.

### 8.3 Heading prototype (item 3)

**What.** This is scratch code in `growth::grow` and `legalize_street`, read from environment variables at call
time, with off the shipped path by control flow (8.0). A field gives each point an axis and a strength. The axis is a
doubled-angle blend of the approach roads (`1 / (1 + (d / 80 m)^2)`), the river (x1.5) and the primary through the
market (x0.7, out to about 250 m). The strength is the blend's coherence times its total weight, times a ramp from 0
at `0.38 max_rf` (the organic core is kept) to 1 at `0.50 max_rf`, times `PS0_GAIN`. Branch headings bend to the
nearest of the four axes by that strength, and exploration bends by 0.7 times it. Where the strength is over 0.5,
oblique candidates (more than 0.35 rad off an axis) are refused, and near-parallel streets keep `PS0_MODULE` apart.
Villages and street villages are untouched: the rows under 1 500 in `both_banks_audit` are identical at both sizes.

**The 8 towns a seed** are the 8 most populous of 1 500 and up in the census world. Census and shell worlds differ, so
the frames (8.5) show the shell's own 8 most populous towns. Blocks are counted outside the core, as 5.3 defines.

| setting (8 largest towns a seed) | housed p10 | quads | triangles | phi median / p90 |
|---|---|---|---|---|
| off, 384 | 95 | 49.6 % | 11.1 % | 0.07 / 0.10 |
| gain 1, module 60 m | 86 | 51.4 % | 12.6 % | 0.10 / 0.19 |
| gain 1, module 90 m | 78 | 51.0 % | 11.6 % | 0.07 / 0.15 |
| gain 0.6, module 60 m | 91 | 54.1 % | 10.8 % | 0.11 / 0.20 |
| gain 3, module 60 m | 72 | 44.6 % | 19.0 % | 0.13 / 0.22 |
| **gain 3, module 40 m (the framed setting)** | 91 | 52.9 % | 10.2 % | 0.17 / 0.36 |
| gain 3, module 40 m, 2 048 (off: 62, 50.7 %) | 87 | 48.8 % | 10.8 % | 0.15 / 0.29 |
| **go bars** | **90+** | **65 %+** | | **0.25 or less (median)** |

**Positive control.** On the framed setting's 24 towns, the share of street length outside the core lying within 15
degrees of the nearest primary's bearing or its perpendicular rises from **39.8 % to 67.2 %** (median; p10 31 to 47,
p90 46 to 84). The rule turns the streets; the blocks still do not close as quads. I have not isolated why. Possible
causes are the first-hit T-snap, which lands a street on whatever lies across its path, `lane_pass` cuts, and seams
between two grains.

**Scope 1.3 item 2's test** (no field; any street alongside within 60 m refused, whatever its angle): triangles
20.7 %, against 23.2 % for a 60 m spacing alone. Housing collapses (p10 39). The test is not isolated, because it
also widens the spacing query (`edges_within`). It supports the oblique-selection hypothesis only weakly.

**The other bars, framed setting, all towns 1 500+, 384 / 2 048 (today in brackets):**

- **FS-1 fails.** Area ratio 0.9-1.4 holds in 28 of 35 / 30 of 36 towns. 85 % or more of the off footprint is kept
  in only 17 of 35 / 20 of 36 towns (median 0.85). The bar has no measured null. `urban_burgage`'s second prototype
  keeps 85 % in 25 of 35.
- **FS-2 passes at 384 and fails at 2 048.** Phi median 0.18 / 0.18 (cap 0.25). Phi p90 0.40 / **0.57** (cap 0.45).
  Per-town phi on minus off reaches **0.47 / 0.45** (cap 0.20). Bearing entropy median 0.94 / 0.95 (floor 0.90).
- **FS-3 passes.** Degree-4 nodes 8.5 / 8.4 %; dead ends 11.8 / 12.7 %; segment median 24 m.
- **Must-hold (5.1).** Housed p10 93 / 90 (94 / 85); towns under 80 % 2 / 3 (1 / 3); villages under 80 % 49 / 21
  (49 / 21); market squares outside the wall 5 / 0 (5 / 1); unbuilt patches over 1 ha 1 / 0 (1 / 0); riverside 4.3 /
  3.8 % (4.3 / 3.7); buildings in water 0 / 0; village greens 629 of 662 / 676 of 684 (the same); stretched towns
  refused 5 / 4 (5 / 6). **Wall / hull p90 1.12 / 1.13, max 1.26 / 1.19 (1.11 / 1.09, max 1.14 / 1.21) breaks the
  strict bar**, and `stretch_water_audit` flags 1 town at 384 (0). The five audits' summaries have no row for wall
  rings in open water, so that bar is not reported. Files: `census/proto_g3m40/`.

### 8.4 Needle lots (item 4)

**Definition.** A four-cornered lot `[p0, p1, q1, q0]` is a needle if any of these holds: its narrow end is under
4.15 m **and** its depth over mean width is over 3.5 (thin and long; the real p5 width and p95 ratio); its depth over
mean width is over 6.0 (the real p99); or either end is under 1 m (a sliver). A lot that is not four-cornered counts
too; none occurs. Building needles: short side under 3 m and long over short over 6.0. The definition flags what the
owner saw. In the close-up crops (`needles/`), the red lots are the fans that radiate from an irregular block's
interior to a point, and the slivers at block corners.

| lots of towns 1 500+, 384 / 2 048 | needle lots | at a 3.3 / 5.0 m floor (384) | taper leg only (narrow end under half the wide one, ratio over 3.5) | needle buildings |
|---|---|---|---|---|
| real (Monpazier, 301 plots) | 3.3 % | | | |
| shipped | 11.1 / 11.4 % | 9.3 / 12.2 % | 5.6 % | 0.56 % |
| `urban_burgage` on | **27.5 / 27.9 %** | 23.2 / 31.3 % | 14.9 % | **3.39 %** |
| heading prototype (framed setting) | 16.5 / 16.6 % | 13.5 / 18.6 % | 8.4 % | 1.05 % |

**Proposed PS bar:**

- needle lots **5 % or less** of the lots of towns 1 500+ at PS-5, and **never above today's 11 %** at any milestone;
- the scale-free taper leg no worse than today's 5.6 %;
- needle buildings no worse than today's 0.6 %.

The width floor depends on a weak scale, so the bar is also reported at 3.3 m and 5.0 m. The bar belongs to platting
(PS-5): deep lots cut toward the centre of an irregular block fan by construction.

### 8.5 Frames (Ruling GA)

City Viewer frames, at 384 x 288 and windowed, for the shell's 8 most populous towns a seed. Views: the opening frame,
x4 and x16 centred on the layout's market (`_zoom` set directly, since the viewer's own cap is 12), and the map's town
layer two zoom steps past the zoom where the layer first draws the town. Each view is shown off and on.

- `contact_<seed>.jpg`: one grid per seed.
- `frames/`: full-size pairs for one town a seed.
- `real_vs_generated/`: each plan beside a comparable generated town, off and on.
- `needles/`: crops of off, burgage and prototype in the same 260 m window.

The building counts printed per layout differ between off and on for every town, so the prototype reached the shell.
The map frames of towns inside the plate's frame margin show no town layer: the town layer never draws there, which is
the frame clip, not the prototype. "Invalid polygon data, triangulation failed" errors appear in equal numbers in the
off and on runs.

**What the frames show.** At first sight the silhouettes hold, but several river towns lose part of a bank, which is
FS-1's failure. At x4, long streets run square to the approach roads in places, with blocks still irregular between
them. At x16 the prototype's lots are not visibly better. In the needle crops, the burgage fans are the dominant
defect.

### 8.6 What PS-1 and PS-2 should change (the "adjust")

1. **Keep PS-1** (the field and the type rule; no growth change). The field turned the streets as designed (8.3).
2. **Re-scope PS-2 before building it.** Headings alone do not make quads. The next spike should add D item 4 (the
   back-street candidate) and a rule that closes a block at right angles at **both** ends: a cross lane between two
   parallels, not a first-hit stub. It should measure the quads and the FS-1 kept share against the same 24 towns.
3. **Hold FS-1 and FS-2's delta in the next spike**, not only at PS-7. The framed setting broke both. A weaker gain
   (0.6) keeps housing at 91 and phi near today's, with quads at 54 %.
4. **Platting owns the needle bar (8.4).** It must cut deep lots only where the block is near-rectangular (PS-5).
5. **Owner questions before PS-2:** Q5 and Q6 (the ground no-go, 8.2) and Q11 (8.1).
   Answered by Ruling GC; folded in at section 9, with new questions Q12 and Q13 at section 7.

---

## 9. After PS-0: PS-2 re-scoped, rear households and footprint (2026-11-17, Ruling GC)

Ruling GC (`LARGE_ITEM_RULINGS.md`): **both** Q5 and Q6; market-street towns may reach per-town phi of about 0.35,
other types keep 0.25, planted grids stay out. Section 5's bars carry the numbers (5.2 needles and rear households,
5.3 the traced targets, 5.4 FS-1 and FS-2 by type); section 6's milestones carry the order. This section holds the
mechanism and the two rules. Everything here is a design read at the symbols plus PS-0's measurements; nothing was
re-measured, so nothing below is a "measured" figure beyond section 8's.

### 9.1 PS-2: why headings did not close blocks, and what changes

**The mechanism at the symbols** (section 1.1's code, with PS-0's prototype added; the causes below are hypotheses
PS-0 named and did not isolate, section 8.3):

1. **`grow`** bends the *origin's* heading toward the field's axis (gain times strength) and so turns the street
   (aligned street length outside the core 40 % to 67 %). But it chooses *where* a branch starts (a random live edge
   of 38 m or more, 35-65 % along it) and *how far* it goes (`legalize_street`'s log-normal 30-125 m, median 56 m)
   independently of any neighbouring parallel. Two parallels get a connecting street only if a random branch from
   one happens to reach the other.
2. **`legalize_street`** stops a candidate at the **first** street it crosses and accepts any junction angle from 25
   degrees. A lane that ends on a street that is not on an axis (an old organic street, a wedge from a gate)
   closes its block at an oblique angle, whatever heading it left with. The prototype's oblique refusal acts only
   at the origin end.
3. **`Graph::add_street`** then snaps each free end to a node within 11 m or an edge within 9 m
   (`attach_point`) and splits every edge it crosses, which also **moves a T-junction by up to 11 m** along the
   hit street and creates short stub edges; the junction lands where the geometry happens to be, not at a module
   station.
4. **`lane_pass`** (up to three passes) splits interior faces of 12 000 m2 or more by a lane joining the midpoints of
   the two longest edges; on an almost-rectangular face this can cut a corner and make a triangle.
5. So a block needs **both** ends square to two parallels, and each end is a separate random event. With each end
   closing square perhaps one time in three, a block closing both is about one in ten [inference; the measured quad
   share moved only from 50 % to 51-54 %].

**What PS-2 changes** (behind `urban_parallel_streets`; off paths untouched by control flow):

- **A back-street candidate** (Option D's item 4, section 3.4), a third candidate kind in `grow`: for a live edge of
  a spine (market primary) or a river-front street, an origin on the perpendicular at the **block depth**, which is
  the 83 m depth-across target plus one street width, about 90 m centreline to centreline [from 5.3's 83 m;
  labelled], then walked outward **along the field axis** by the exploration step, to the extent the dry ground
  and the growth front allow (section 9.3). The branch heading stays 90 degrees to the edge, as today.
- **A cross lane** (new, after the back street exists; a `closing` mode of `legalize_street`): at stations spaced
  by the block length target along the pair (length about twice the depth, so aspect 2.0 or more, jittered +/- 20 %,
  never within 30 m of a node), a segment placed **with both ends exactly on the two parallels** and perpendicular
  to them. It is **not** a first-hit stub: it is refused whole unless it reaches the far parallel within 5 degrees
  of square at **both** ends, crosses nothing in between and passes the water, slope and gate rules as today. Both
  ends are then given to `add_street` as exact points on edges, with an edge-snap, so the T-junction sits at the
  station, not up to 11 m from it.
- **The oblique refusal** at the *far* end: a candidate in a grain whose first hit is a street off the axes by more
  than 0.35 rad is not accepted as a T there; it may continue to the next street or be dropped, so a wedge is a
  seam, not a default.
- **`lane_pass`** skips any face that is already four-cornered with rectangularity 0.75 or more, so a closed block
  is not cut into a wedge afterwards.
- **Kept as today:** the organic core inside `0.38 * max_rf`, the first-hit snap and the 25-degree rule for every
  non-grain town, villages and radial towns byte-identical, the 0.6 gain (keeps housing at 91 and phi near today's,
  section 8.3) rather than PS-0's framed 3.0.
- **Seams** between two grains or a grain and the core may keep triangles; 5.3's 15 % ceiling is their allowance.

The step-0 measurement (section 6) checks that the mechanism in 1-5 is the real one: if cross lanes with both ends
exact do not move quads from 54 % past 60 %, the cause is elsewhere (`lane_pass`, seams) and the milestone stops
at its no-go.

### 9.2 Housing with rear households (Q5; Ruling GC)

Today every four-cornered lot houses one household of 5.2 (`build_buildings`; section 3.6). The rule becomes:

- **A rear household exists only on a lot more than 30 m deep** ("about 30 m", Ruling GC: the threshold is 30 m,
  measured as the lot's depth along its front-to-back axis). Research 2.6: backland fills over time.
- **At most one rear household per lot**, so a lot houses at most two. The rear dwelling is a building on the lot's
  rear part, set back from the front building by at least 4 m, and it takes the lot's frontage as its width, so it
  cannot be a needle (section 5.2): its short side is the lot width, never under 3 m.
- **Order of filling, the deterministic rule** (so the count is reproducible, section 7p): front households first,
  in the engine's existing lot order; a rear household is created **only for the demand the front households did
  not house**, and in the order of the lots' distance from the market (nearer first: backland fills where land is
  dear). A lot's rear household never exists unless the town needs it.
- **Counting:** `housed` counts every household in either position; a rear household counts as one household of 5.2
  and nothing else changes. The **40 % ceiling** (5.2; Q13) caps lots carrying one, so a town cannot be housed by
  doubling every deep lot.
- **Access:** a rear dwelling needs a way in: a back lane (the back-street tier, PS-3) within 30 m, or a side passage
  of 2.5 m reserved in the lot [judgement; Q13 may reject]. A lot with neither gets no rear household.
- A **building's** needle test and the 56 m unbuilt reach apply to the rear dwelling as to any other.

### 9.3 The footprint beyond 1.4x, gated by ground (Q6; Ruling GC)

PS-0 measured (8.2): on the market's bank, buildable ground over settled area has min 0.97, p10 1.16, median 1.36;
all banks p10 1.67. The deep-plot footprint measured x1.20 median, p90 x1.68 (3.6). Ruling GC lifts the 1.4x ceiling
**wherever the ground allows**. The gate, per town:

- **Growable cell**: dry (`!site.is_water`), slope at most 0.34, inside the box's 40 m margin, within `max_rf` of the
  market (the stretch frame where the town is stretched), **and** either on the market's bank or **on the far bank
  inside the bridge strip** that `grow_far_bank` already serves (`FAR_BANK_MAX_RATIO` unchanged; raising it needs a
  ruling). Far-bank growth is therefore **allowed, not unbounded**: it is `grow_far_bank`'s own rule, run in the
  grain's field.
- **Ground ratio** `g = growable area / today's settled area` in the same region (PS-0's measure).
- **Never past the law or the wall**: the footprint stops when settled area / 0.12 N^0.714 would pass **1.0**, or
  when wall / hull p90 would rise above 5.1's row; whichever binds first. A town that would pass either keeps the
  footprint it has and shortens its plots instead (PS-5).
- **No fixed multiple.** The footprint factor for a town is `min(g, law ceiling, wall ceiling)`; a town with `g`
  under what its plots need gets **fewer deep plots**, never a worse housed share (5.1).

**The feasibility rule, redone with both** (replaces 3.6's single "1.2-1.5 times" figure and PS-0 item 2's 1.4x
test):

1. Rear households carry a share `rho` of the households, at most 0.40 of the lots (9.2). The front lots a town
   needs fall from H to `H / (1 + rho)`.
2. The footprint the deep plots need is `F_req = F_deep / (1 + rho)`, with `F_deep` = PS-0's deep-plot figure (1.4 for
   the median, the p90 figure 1.68 as the sensitivity) and `rho` taken at its **ceiling 0.40 only for the p90 case,
   0.25 for the median** [judgement; both labelled]. That is **F_req about 1.1 (median) and 1.2 (p90 case)**.
3. A town **fits** when `g >= F_req`, counting the market's bank plus the bridge-strip far bank.
4. **Go for PS-1** when 90 % or more of towns 5 000+ fit at both sizes. From PS-0's distribution (min 0.97, p10 1.16
   on the market's bank alone; 23 of 28 fit at x1.2) this is likely at about 1.1 [inference, **not measured**: the
   recount at F_req on the far bank's strip is PS-1's bar, section 6]. A town that does not fit keeps today's
   layout (Q13).

### 9.4 What did not change

The Option D recommendation, the switch's semantics and boundaries (section 4), the must-hold bars (5.1), FS-3,
FS-4 and FS-5, and the rule that no bar is loosened without a ruling. The "planted grids stay out" decision is Q2's
default, now restated by Ruling GC.

---

## 10. PS-1 result (2026-11-18, Ruling GF): built behind `urban_parallel_streets`, OFF at both boundaries; ground recount GO

**What was built.** `cartalith_urban::field` (new), wired through `GenOpts::parallel_streets`, `PlaceOverrides` /
`UrbanContext::parallel_streets`, `WorldParams::urban_parallel_streets` (registry row, `JS_PATHS`, `LOADS_WHEN_ABSENT`,
assistant catalogue row, `Regen::Live`) and `urban_bridge`'s layout dictionary (`parallel_type`, `parallel_approaches`,
`parallel_ground_*`, `parallel_core_m`, `parallel_field` / `parallel_grain`, absent with the switch off). With the switch
on, `generate_pass` computes `field::plan` for every town of 1 500 and up that is not a street village, before `grow`,
and fills its settled area from the finished layout; it reports it on `Town::parallel` and touches nothing else.

- **Type rule** (`town_type`, section 4's order): radial plan, no grain; then **river-front** (a real river, `river` or
  `riverthrough` site, and the town stretched or its market square's outline within 10.8 m, two street widths, of the
  bank); then **market-street** (two approach roads, read where the primaries cross a 100 m circle round the market,
  at least 150 degrees apart: a road through the market); then **organic-core** (any approach road); else no grain.
  River-front is tested first, so a town that is both takes the stricter FS-2 cap, never the looser one. No planted
  grid type exists (Q2). `TownType::fs2_caps` carries Ruling GC's split: market-street 0.35 per town and a delta of
  0.30 (Q12 (a)); river-front and organic-core 0.25 / 0.45 / 0.20.
- **Field** (`DirectionField::at`): PS-0's doubled-angle blend (roads 1 within about 80 m, river 1.5 within about 80 m,
  spine 0.7 out to about 250 m), strength = coherence x total weight capped at 1 x a ramp from the core's edge over
  0.12 `max_rf`. The core is 0.38 `max_rf` at 300 years, growing 0.12 per 700 years (0.50 at 1 000, floor 0.33): Q3's
  "older town, larger core", a labelled judgement. Under 5 000 one grain (the river, the spine, or one road along the
  town's mean road orientation); from 5 000 every approach-road edge besides.
- **Ground gate** (`ground_gate`, section 9.3, a pure function over `Site`): PS-0 item 2's 10 m lattice (dry, slope
  0.34 or less, inside the 40 m margin, within `max_rf` by `Site::market_dist`) on the market's bank, plus the far bank
  only where `grow_far_bank` would act (a both-banks town, `far_bridge_end` buildable, each cell `far_bank_buildable`
  and within `max_rf` of the bridge end or the stretch frame), the far area capped at `FAR_BANK_MAX_RATIO` (1.0) x the
  market bank's (a labelled reading of the street-length cap as an area cap). The denominator is today's settled area
  (the census's FS-1 cells) inside the same region. `F_REQ_MEDIAN` = 1.4 / 1.25 = 1.12 and `F_REQ_P90` = 1.68 / 1.40 =
  1.2, from section 9.3's formula; the scope's "about 1.1" is the 1.12.

**Measured** (evidence `cartalith-native/docs/evidence/parallel-streets-ps1-2026-11-18/`, `summary.txt` lists every
file): census world, seeds 483920 912345 246371, 800 km, villages on, the app's `params::defaults`, 384 x 288 and
2 048 x 1 311, release test binary of this tree.

| ground recount, towns 5 000+ | 384 (28 towns) | 2 048 (26 towns) |
|---|---|---|
| fit at F_req 1.10 | 28 (100 %) | 26 (100 %) |
| **fit at F_req 1.12 (section 9.3's median case)** | **28 (100 %)** | **25 (96 %)** |
| fit at F_req 1.20 (p90 case) | 28 (100 %) | 24 (92 %) |
| g min / p10 / median | 1.21 / 1.55 / 1.88 | 1.11 / 1.45 / 1.96 |
| control: market's bank only, x1.2 / x1.4 (PS-0 item 2's test) | 16 / 5 of 28 | 12 / 5 of 26 |
| control: market's bank only at 1.12 | 21 of 28 | 16 of 26 |

**Go** (90 % or more at both sizes) at every one of the three requirements; the verdict does not flip between 1.10
and 1.20. Under 1.20 at 2 048: 246371 #0 (g 1.11) and #2 (1.16), both 20 000. The go rests on the far bank's strip:
on the market's bank alone it would be a no-go (16 / 12 at x1.2; 21 / 16 at 1.12), so two more misses at 2 048 would
flip it if the far strip were lost. The numerator reproduces PS-0's buildable figure (`towns_g3m40d_384.csv`) to the
tenth of a hectare for 23 of 28 towns at 384; the five that differ (483920 #12 and #13, 246371 #2, #7 and #11, one of
them lower) are not isolated. The market's-bank control (16 / 5 of 28 at x1.2 / x1.4) sits between PS-0's two kept
figures: `ps0_agg.py` over the whole town's settled area gives 6 / 2 of 28, and section 8.2's like-with-like 23 / 7 came
from a region count that is not in PS-0's kept scripts; the difference is unexplained. Towns of 5 000+ at 2 048 are
26, as in PS-0's own `towns_g3m40_2048.csv`; section 6's "22" is the count among the 8 largest towns a seed. The far
bank counts ground `grow_far_bank` is allowed to grow, not ground today's towns use, and whether a bridge can be laid
there was not cross-checked (the layout does not carry `Town::town_bridge`): several far-bank-dependent towns have no
far-bank settlement today (912345 #6 at 384: settled 83.8 ha in the region, all of it on the market's bank).

**Types** (towns 1 500+): 384: river-front 21, organic-core 10, market-street 4 (of 35); 2 048: river-front 20,
organic-core 9, market-street 7 (of 36). Market-street does not dominate, so the through-road test is not too loose.

**Every bar** (the five audits' text with the switch on is byte-identical to off, and off to Ruling GB's
`underbuilt-fix-2026-11-17/census_on`, at both sizes; all 35 / 36 towns' full geometry hashes match off and on):

- 5.1 at today's values, unchanged: housed p10 95 / 91; towns under 80 % 1 / 3; villages under 80 % 49 / 21; wall /
  hull p90 1.11 / 1.09 (max 1.14 / 1.25); squares outside the wall 5 / 1; unbuilt over 1 ha 1 / 0; riverside 4.6 /
  3.7 %; buildings in water 0; wall rings in open water 0; greens 629 of 662 / 676 of 684; stretched refused 4 / 4;
  villages and radial towns: no plan (villages) or no grain (radial), layouts identical. **"Today" has moved since
  section 5.1 was written** (Ruling GB: housed p10 94 to 95 and 85 to 91, wall / hull max at 2 048 1.21 to 1.25,
  riverside 4.3 to 4.6 %, refused 5 / 6 to 4 / 4); PS-2 measures against these.
- Block shape and needles (PS-0's instrument, off = on): quads 50.3 / 50.5 %, triangles 10.6 / 11.0 % (compact 6.8 /
  7.1), aspect 1.57 / 1.57, rectangularity 0.70 / 0.69, 83 m+ across 3.7 / 3.7 %; needle lots 11.3 / 11.3 %, needle
  buildings 0.57 / 0.55 %.
- FS-1: area ratio 1.00 and 100 % kept in every town. FS-2: phi on minus off 0.000 in every town; by type (off = on),
  384: market-street median 0.25, max 0.29; organic-core median 0.07, p90 0.34; river-front median 0.08, p90 0.12;
  2 048: market-street median 0.21, max **0.42**; organic-core median 0.23, p90 0.39; river-front median 0.06, p90
  0.10. **Two small market-street towns at 2 048 are already over 0.35 today** (483920 #38, 1 584, phi 0.42; 912345
  #9, 2 003, 0.39): few streets, high order. PS-2 should judge them on the delta, and the owner may want to know. FS-3:
  degree 4 10.1 / 10.3 %, dead ends 11.5 / 11.4 %, segment median 24.3 / 23.9 m.

**Frames** (`contact_<seed>.png`, `frames/pair_<seed>_<id>.png`): the scope's 14 towns, found by map cell, City Viewer
opening frame off, then on in the same session with the field drawn by `_viewerframe_probe.gd --parallel 2` (magenta
ticks along the grain, the core's circle, the type and ratio). The probe checks the data, not only the pixels:
building and edge counts identical off and on, no `parallel_type` off, a type on (`RESULT: PASS`, all three seeds).
River-front grains follow their rivers (912345 #8, #15; 246371 #4, #13); market-street spines run along the through
road. 13 of 14 "on" frames show the field. 246371 #27 (1 790) shows no ticks: measured on its dumped layout
(`frames/246371_27_buildings.txt`), its buildings' centroids are a median 603 m from its market point and only 67 of
336 lie within 350 m, while its field samples reach about 340 m (`frames/246371_27_field_extent.log`), and the opening
frame shows the far part. Not investigated; it is under 5 000, outside the go bar.

**Mutants:** 34 on `field.rs`, all killed (two survived the first pass, the bearing-wrap merge and the stretched
river-front branch; a test was added for each and both were then killed). Tests: 18 in `field/tests.rs`.

**River-front equals the stretched count here** (21 / 20 towns, and every stretched town is river-front by the
rule), so with river-front tested first Ruling GC's 0.35 cap applies to only 4 / 7 towns. How many river-front towns
also have a road through the market was not measured; that is the owner's to weigh (Q14 below).

**PS-2 can start** (its step 0 first, section 6). The ground no-go of section 8.2 is answered by rear households and
the far-bank strip. Towns short of ground keep today's layout (Q13 (b)): none at 384; at 2 048, 246371 #0 at F_req
1.12, and #0 and #2 at 1.20.

**New owner questions from PS-1.** **Q14.** River-front is tested before market-street, so a river town with a road
through its market takes the 0.25 cap, not 0.35. Keep that order? [yes: the stricter cap; measure the overlap in PS-2
step 0] **Q15.** Which requirement decides Q13 (b) for a town: section 9.3's median case 1.12 or the p90 case 1.20?
[1.12, the median case; it keeps today's layout for 246371 #0 at 2 048 only]

---

## 11. PS-2 step 0 result (2026-11-21, Ruling GG): NO-GO; nothing built

**Verdict: no-go.** The scratch prototype of section 9.1 (back street, cross lanes in a closing mode, the far-end
oblique refusal, `lane_pass` keeping closed blocks, gain 0.6) closes blocks as designed: quads outside the core rise
from 49.9 % to **65 %** on the 24 towns, past the 60 % bar, and housing holds (p10 96-98). It fails **FS-1's kept
share: 17-18 of 35 towns keep 85 % of their off footprint, against the bar's 28**, and FS-2's per-town delta (up to
0.37 / 0.47 against 0.20 / 0.30). By section 6's rule PS-2 stops here: no product code, no third heading variant.
The prototype is kept only as `scratch_code/step0_prototype.diff.txt`.

**The measured null (new).** FS-1's 85 % had no null (section 8.3). Step 0 measured one: the same town, same rules,
with only `grow`'s random stream re-seeded (no field, no PS-2 rule) keeps 85 % of its footprint in **10 of 35 towns**
(median 0.81). The kept-share bar is therefore below the generator's own seed noise: no change to growth that draws
a different sequence can pass it, and the prototype (17-22 of 35) already keeps more than the null. **This is for the
owner (Q16 below), not a reason to loosen the bar here.**

Evidence: `cartalith-native/docs/evidence/parallel-streets-ps2-2026-11-18/` (`summary.txt` lists every file). Census
world: seeds 483920 912345 246371, 800 km, villages on, the app's `params::defaults`, 384 x 288; release test binary
of `origin/main` `248e2e2f` plus the prototype; `parallel_streets_audit` (PS-1's, with `PS1_CSV`) and PS-1's
`ps1_agg.py` (PS-0's measures). "Off" is today's layout in the same process; "on" the switch with the prototype.

| step 0, 384 | quads outside core (24 towns) | compact / all triangles | housed p10 (35 / 24) | FS-1 kept >= 85 % (35 towns) | phi on-off max (by type) | needle lots, 35 towns |
|---|---|---|---|---|---|---|
| off (today) | 49.9 % | 6.7 / 10.8 % | 95 / 95 | 35 | 0 | 11.3 % |
| prototype, first run (`s0a`) | **64.4 %** | 3.9 / 6.5 % | 99 / 98 | **17** | 0.47 market-street, 0.34 river-front | 12.6 % |
| prototype, back streets on their own substream (`s0b`) | **65.3 %** | 3.9 / 6.5 % | 99 / 96 | **18** | 0.38 market-street, 0.32 river-front | 12.4 % |
| diagnostic: back streets and cross lanes only, gain 0 (`s0g0`) | 57.0 % | 5.5 / 8.7 % | 96 / 95 | 22 | 0.23, 0.18 | 11.9 % |
| null: `grow` re-seeded, no PS-2 rule (`s0null`) | 49.9 % | 7.5 / 11.0 % | 94 / 94 | **10** | 0.20 organic-core | 11.6 % |
| **go bar** | **60 %+** | 5 / 15 % or less | **90+** | **28+** | 0.30 / 0.20 | never above 11 % |

- Passes: quads, both triangle bars, housed p10, FS-3 (degree 4 9.1-9.5 %, dead ends 10.4-10.8 %, segments 24.6 m).
- Fails: FS-1 kept share (both runs); FS-2's delta for market-street and river-front towns; needle lots 12.4-12.6 %
  against the "never above today's 11 %" (5.2; today 11.3 % on this tree); FS-1's area ratio 0.9-1.4 in 28-32 of 35.
- Not moved: block depth across (median 29 m, 83 m+ 3.7 %) and aspect (1.57): the back streets' 90 m blocks are
  filled by ordinary branches afterwards, so the deep blocks section 5.3 wants are not there. That is PS-3's lever
  (module depth), and it was not tried.
- The gain-0 diagnostic says the heading bend, not the back street, carries most of the FS-2 delta and part of the
  FS-1 loss; the back streets and cross lanes alone still reach 57 % quads.
- Market-street towns rose to phi 0.63-0.72 in the first two runs (cap 0.35); one is 483920 #38, already 0.42 off.

**Frames** (owner condition, Ruling GA): City Viewer frames of the scope's 14 towns (PS-1's cells), windowed, scratch APPDATA, under the
window lock, from the release-built prototype (the `s0b` code) with the switch toggled live in one session
(`scratch_code/_ps2s0_probe.gd.txt`, PS-1's probe with an x4 view, a layout dump per phase, and its PS-1 "unchanged"
check turned into "moved": `RESULT: PASS` on all three seeds, every one of the 14 layouts moved). `contact_<seed>.png`:
opening frame off, on, then x4 (market-centred) off, on; `frames/pair_<seed>_<id>_z1|z4.png` full size;
`needles/needles_<seed>_<id>.png`: the same 260 m window off and on from the dumps, needle lots (section 8.4) red.
What they show, all 14 looked at: **at first sight several river towns lose a bank** (483920 #0 and #11, 912345 #2,
246371 #4: the off town spans the river, the on town sits on one bank), which is FS-1's failure in the pixels; one
cause consistent with the data is that the on layout houses more on the market's bank (housed median 115 against
107), so the far-bank pass is not reached, but it was not isolated. At x4 the on blocks are visibly more rectangular,
with long parallel streets (483920 #4 and #23, 246371 #13 and #14), and 246371 #14 (market-street) begins to read as a
grid along its spine. For the small towns (483920 #10, 912345 #30, 246371 #27) the x4 view lands on the market and
river and shows little. The needle crops show the same defect off and on: fans of thin lots into an irregular
block's interior corners (483920 #4 16.1 % off, 16.2 % on in its window; 246371 #14 16.8 / 14.7 %); the prototype
neither removes nor adds them visibly, in line with the census's 11.3 to 12.4 %.

**What this leaves for the owner.**

- **Q16. FS-1's kept share.** The null keeps 85 % in 10 of 35 towns, so the bar (28 of 35) cannot be met by any
  growth change. Options: (a) judge FS-1 by the area ratio (0.9 or more, as written) and the visual gates FS-4/FS-5
  only, dropping the kept share; (b) restate it against the null (no worse than the re-seeded layout, 10 of 35, or
  its median 0.81); (c) keep it, and parallel streets stop at PS-0/PS-1. [(b): the bar measures "the same town",
  and the null says what the same town looks like grown twice]
- **Q17. After Q16.** Under (a) or (b) the prototype passes FS-1 but still fails FS-2's delta (0.38 / 0.32 against
  0.30 / 0.20) and the needle bar (12.4 % against 11 %); the gain-0 variant meets FS-2 but not quads (57 %) or needles
  (11.9 %). [re-run step 0 under the restated FS-1, with the far-bank loss isolated first, since it is the frames'
  most visible change; no third heading variant without a ruling]

## 12. PS-2 step 0b (2026-11-21/22, Ruling GH and the owner's wall question): NO-GO again; nothing built

**Verdict: no-go.** The lost bank was found and mostly fixed, and the wall spikes the owner saw are explained and fixed
within the prototype; the restated FS-1 now passes. The prototype still fails **FS-2's delta** and the **needle bar**,
both unchanged by these fixes, so by section 6's rule PS-2 stops. No third heading variant was tried (Ruling GH).
Evidence: `cartalith-native/docs/evidence/parallel-streets-ps2b-2026-11-21/` (`summary.txt` lists every file); census
world, tools and the null are step 0's (section 11); the prototype is `scratch_code/step0b_prototype.diff.txt`.

**1. The lost bank (Ruling GH Q17).** Hypothesis: the on layout houses more on the market's bank, so the far-bank pass
(`riverbank::second_pass`, which runs only while the first layout houses under `pop_target`) never runs.
**Proved for part, refuted as the whole.** Temporary diagnostics (first-layout housed, target, pass, buildings per
bank) on the 35 census towns: the on layout's first pass houses 5 to 50 % more than today's (median 1.06, p75 1.22,
`towns_s0b_384.csv`), and of 32 both-banks towns two lose the far bank exactly this way (912345 #2: off 2 111 against
a target of 2 458, far-bank pass, 394 far-bank buildings; on 2 688, no pass, 0; 246371 #15 likewise). The rest keep it
(13 hold it both ways), and in 483920 #0 today's layout has none either (the pass does not run for a town over its
target). **Fix (prototype, `riverbank::PARALLEL_FAR_BANK_SLACK` 1.25):** a town with a grain is judged against 1.25
times its target (the p75 of the housing ratio), and the share is taken against that. Re-run: 912345 #2 now keeps its
far bank (3 099 / 675 buildings against off 3 121 / 684). **Still seen in the frames:** 483920 #11's wall no longer
spans the river (the far bank holds 687 buildings against 870, under `FAR_BANK_LOT_SHARE`'s 0.32 of the market
bank's lots, so the wall rings one bank), a wall effect more than a bank effect. The audit's town indices are not the
probe's (the audit's 483920 #0 is not the probe's #0), so the four named towns were checked in the probe's dumps.

**2. The wall spikes (coordinator, on 246371 #14).** Cause: `wallhug` fits the wall to the lots inside the grown
circuit. The back streets' 90 m blocks leave the lot mass in clumps with open ground between them, the taut pass fills
a notch only under 8 000 m2 (`TAUT_POCKET_MAX_M2`) and never a spike, so the ring follows the clumps (912345 #32 and
246371 #9 go from a hull-like 1.00 to a V-notched 1.21 to 1.25 of their hull). Today's walls are not clean either: on
the census a wall has a median 4 and a p90 6 vertices turning over 60 degrees (max 8), and today's 246371 #14 has 14
(`wall_stats.txt`). **Bar added (step 0):** over the walled towns, the median and total of vertices turning over
60 degrees and the wall-to-hull length (median, p90, max) must be no worse than today's; per town a refit ring over 6
such vertices (today's p90) or over 1.15 of its hull (the judgement's p90 bar, today's max 1.144) is jagged.
**Refuted first:** a x3 pocket cap raised the sharpest ring to 27 sharp vertices and the hull ratio to 1.38 (the
open-ground guard falls back to the raw outline). **Fix kept (`wallhug::ring_is_jagged`, `KeptReason::Jagged`):** a
refit ring of a town with a grain that is jagged is refused and the grown circuit kept, as for any refused ring.

**Numbers, the final prototype (`c5`) against the bars.**

| step 0b, 384 | quads outside core (24) | housed p10 (35 / 24) | FS-1 kept >= 85 % (35) | FS-2 delta (market-street / river-front) | needle lots (35) | wall sharp med / p90 / total | wall / hull med / p90 / max |
|---|---|---|---|---|---|---|---|
| off (today) | 49.9 % | 95 / 95 | 35 | 0 | 11.3 % | 4 / 6 / 133 | 1.045 / 1.113 / 1.144 |
| null (re-seed, step 0) | 49.9 % | 94 / 94 | **10** (median 0.81) | 0.20 (organic) | 11.6 % | | |
| step 0 (`s0b`) | 65.3 % | 99 / 96 | 18 (0.87) | 0.38 / 0.32 | 12.4 % | 4 / 7 / 147 | 1.063 / 1.114 / 1.248 |
| far-bank slack only (`c1`) | 64.4 % | 115 / 108 | **23** (0.885) | 0.378 / 0.274 | 12.5 % | | |
| **final (`c5`, slack + jag guard)** | **64.4 %** | **101 / 99** | **23** (0.875) | **0.398 / 0.274** | **12.6 %** | **3 / 7 / 127** | **1.034 / 1.086 / 1.114** |
| **bar** | 60 %+ | 90+ | **the null's 10, median 0.81** | 0.30 / 0.20 | never above 11.3 % | median, total <= today's | <= today's |

Passes: quads; housed p10; **the restated FS-1 (23 of 35, median 0.875, against the null's 10 and 0.81)**; the wall bar
(median 3 against 4, total 127 against 133, hull ratio max 1.114 against 1.144; the p90 7 and max 9 sit one above
today's 6 and 8, from towns whose kept circuit is itself sharp). Fails: **FS-2's delta (market-street 0.398 against
0.30, phi to 0.62 against the cap 0.35; river-front 0.274 against 0.20)**; **needle lots 12.6 % against 11.3 %**; and
the area ratio's 0.9 to 1.4 holds in 32 of 35 (min 0.80). The heading bend carries FS-2 (step 0's gain-0 diagnostic) and
the back streets the needles; neither is touched by the far-bank slack or the wall guard, and a lower gain is the third
variant Ruling GH forbids without a ruling.

**Frames (owner condition).** City Viewer, the 14 towns, opening frame and x4, off and on, windowed under the lock
(`contact_<seed>.png`, `frames/`, probe `RESULT: PASS` x3); all 14 looked at, and the data checked beside them
(`dumps_bank_wall_14towns.txt`). 246371 #14's wall is now a smooth taut curtain (`wall_closeups/`, with 246371 #6, #13
and 483920 #23; off | step 0 | step 0b); the guard keeps the grown circuit, whose rounded ring carries many towers
(912345 #2 shows a ball chain at x4, as any kept circuit does today). 912345 #2 keeps its far bank; 483920 #11 shows one
bank inside the wall. x4 still shows long parallel streets and squarer blocks (483920 #4, 246371 #13 and #14).

**What this leaves for the owner.** Q18: FS-2's delta and the needle bar are what stands between this prototype and a
go, and they are the heading gain and the back streets' blocks; options: (a) a lower gain (the third variant),
(b) restate FS-2's delta against a re-seed as Q16 did for FS-1 (the null's own organic delta is 0.20), (c) stop at PS-1.

## 13. PS-2 step 0c (2026-11-22, Ruling GJ: a lower heading gain): NO-GO again; nothing built

**Verdict: no-go.** Step 0b's two fixes kept (`PARALLEL_FAR_BANK_SLACK`, `wallhug::ring_is_jagged`), the heading gain
swept at 0.4, 0.3 and 0.2 (step 0b's 0.6, step 0's gain-0 diagnostic: 0.0), every step-0 bar read per gain. **No gain
passes them all.** Evidence: `cartalith-native/docs/evidence/parallel-streets-ps2c-2026-11-22/` (`summary.txt`); census
world, tools and the null as before; the prototype is `scratch_code/step0c_prototype.diff.txt`.

| step 0c, 384 | quads outside core (24) | housed p10 | FS-1 kept >= 85 % (null 10, median 0.81) | FS-2 delta, market-street / river-front (caps 0.30 / 0.20) | market-street phi max (cap 0.35) | needle lots (<= 11.3 %) | wall: sharp median / p90 / total, hull ratio max (today 4 / 6 / 133, 1.144) |
|---|---|---|---|---|---|---|---|
| gain 0.6 (step 0b) | 64.4 % | 101 | 23 (0.875) | 0.398 / 0.274 | 0.62 | 12.6 % | 3 / 7 / 127, 1.114 |
| **0.4** | **61.5 %** | 100 | 22 (0.88) | **0.297** / 0.231 | 0.548 | **12.1 %** | 3 / 6 / 125, 1.126 |
| 0.3 | 60.0 % | 102 | 22 (0.87) | 0.317 / 0.266 | 0.569 | 12.2 % | 4 / 6 / 127, 1.115 |
| 0.2 | 60.0 % | 106 | 21 (0.88) | 0.330 / **0.161** | 0.581 | 12.3 % | 4 / 6 / 130, 1.132 |
| 0.0 (step 0, before the 0b fixes) | 57.0 % | 96 | 22 | 0.23 / 0.18 | | 11.9 % | |

Every gain passes quads, housed p10, the restated FS-1, the wall bar and the river-front and organic medians and p90.
Every gain fails two bars the gain does not move:
- **Needle lots 12.1 to 12.3 % against 11.3 %** (today's 11.3 %): the back streets' blocks, not the heading bend (gain 0
  in step 0 already gave 11.9 %).
- **One market-street town's phi, 0.55 to 0.58 against the cap 0.35** (246371 #9, a town of 2 006: off 0.25, on 0.55 at
  every gain): the order comes from its back streets and cross lanes in a town with few edges, not from the bend.
The FS-2 delta is met by the market-street towns only at 0.4 (0.297) and by the river-front towns only at 0.2
(0.161), so no single gain meets both, and the delta moves between gains by noise as much as by the gain (0.317 at
0.3 against 0.297 at 0.4); market-street entropy is 0.88 at 0.4 (bar 0.90), 0.92 at 0.3, 0.90 at 0.2.

**Frames** (owner condition): City Viewer, the 14 towns, opening frame and x4, off and on at gain 0.4, windowed under the
lock, probe `RESULT: PASS` x3; all 14 looked at. Wall close-ups (`wall_closeups/`, off | gain 0.6 | gain 0.4) for 246371
#14, #6, #13 and 483920 #23: at 0.4 the walls are taut curtains as at 0.6 (the jag guard keeps the grown circuit), and
the streets are visibly less ruled than at 0.6 (long parallels remain along the market spine of 246371 #14 and in the
blocks beside it at x4). Needle crops in `needles/`.

**What this leaves for the owner.** The two failures are the back streets' blocks (needles; the small market town's
phi), so a further gain change is not the lever. Q19: (a) judge the needle bar and FS-2's cap against a re-roll as Q16
did for FS-1 (needles: the re-seed gives 11.6 %, today 11.3 %; the small town's phi against its own re-roll);
(b) change the back-street rule (fewer or shorter back streets, or none in a town under about 3 000, the lever for the
needles and the small town's phi; a fourth variant); (c) stop at PS-1.


## 14. PS-2 built (2026-11-22, owner Ruling GM): behind `urban_parallel_streets`, OFF at both boundaries

**Built.** Step 0c's prototype as real code, at heading gain 0.4, with step 0b's two fixes. Owner (Ruling GM): *"I like ps2"*;
the FS-2 delta, the market-street phi cap and the needle bar are judged **against a re-roll of today's `grow`** (as Ruling GH
did for FS-1), not as absolute caps. Evidence: `cartalith-native/docs/evidence/parallel-streets-ps2d-2026-11-22/`.

| Piece | Where |
|---|---|
| the grain reaches `grow` (`GrowOpts::field`, `None` = the reference, every golden) | `cartalith_urban::growth` |
| heading bent toward the grain, gain `PARALLEL_GAIN` 0.4 (explore 0.7 of it) | `grow` |
| back street (`lay_back_street`): a rib of exactly `BACK_DEPTH_M` 90 m square to the frontage, then a street walked both ways (`BACK_WALK_STEPS` 3), on 50 % of qualifying taps (`BACK_SHARE`, its own substream so the epoch's draws stay as near today's as the geometry allows) | `growth.rs` |
| cross lane (`cross_lane`): square within 5 degrees, both ends 30 m clear of any node, one block depth | `growth.rs` |
| far-end oblique refusal (a street does not end as a T on a street over 0.35 rad off every axis of a strong grain) | `legalize_street` |
| the site rules shared by streets and lanes (`refused_by_site`, moved unchanged) | `growth.rs` |
| closed four-cornered blocks of rectangularity 0.75 are not cut (`lane_pass_with`, `KEEP_RECT`) | `cleanup.rs`, `field.rs` (`block_corners`, `rectangularity`, `axis_off`, moved from the census) |
| a town with a grain is judged against 1.25 times its target for the far bank (`PARALLEL_FAR_BANK_SLACK`) | `riverbank::second_pass` |
| a refit wall of such a town that is jagged is refused, the grown circuit kept (`ring_is_jagged`, `KeptReason::Jagged`; at most 6 vertices over 60 degrees, at most 1.15 of the hull) | `wallhug.rs` |

Only a town of 1 500 and up with a grain changes (`Town::parallel`'s `field` is `Some`): villages (street villages),
radial towns and no-grain towns are laid out exactly as with the switch off (`field::tests::only_a_town_with_a_grain_changes_layout`,
hash equality at 800, 2 500 and the radial culture; the 48 growth goldens and every urban golden run with `field: None`).

**The census confirms step 0c exactly.** The same 35 towns at 384, the new code from `origin/main`: every row of
`towns_384.csv` is identical to step 0c's gain-0.4 rows (0 of 71 lines differ), so every bar below is step 0c's. **These are
PS-2's baseline, which later milestones must not worsen** (owner Ruling GM):

| baseline, 384 | off (today) | PS-2 |
|---|---|---|
| quads outside core (24 towns) | 49.9 % | **61.5 %** |
| housed p10 | 95 | **100** |
| FS-1 kept >= 85 % (re-seed null: 10 of 35, median 0.81) | 35 | **22 (median 0.88)** |
| needle lots | 11.3 % | **12.1 %** |
| FS-2 delta, market-street / river-front / organic | 0 | **0.297 / 0.231 / 0.168** |
| market-street phi, per-town max | | 0.548 (one small town, 246371 #9) |
| wall: sharp vertices median / p90 / total; wall over hull max | 4 / 6 / 133; 1.144 | 3 / 6 / 125; 1.126 |

**Cross lanes are rare.** A station 144 to 216 m along a back street needs the walk to have gone that far, so 11 cross lanes were laid
over the whole census (against hundreds of back-street edges) and none shows in the 6 000-person fixture; the four-cornered blocks
come from the back streets and the kept-closed rule. They are kept because step 0c measured with them (removing them would move
the baseline); a candidate for PS-3's review.

**Tests (literals).** `growth/ps2.rs`: the cross lane's acceptance and its five refusals (oblique, node near either end, too far,
too near), the oblique T refusal against a 45-degree and an on-axis grain, the back street's geometry (rib one depth long, one
cross lane at stream seed 0, the blocked rib), the 6 000 town (35 back-street and 0 cross-lane edges, 410 edges against today's 396),
seed 13 at 12 000 (534 edges, the closed-block rule through `generate`), the constants. `cleanup`: `keep_rect` leaves a rectangular
grid whole and cuts a sheared one. `field`: `axis_off`, `block_corners`, `rectangularity`. `wallhug`: `ring_is_jagged` on six rings
(square, L at the cap of 6, a 7-sharp tooth, a star, a crescent, a triangle) and the guard on seeds 17 and 56 (`Kept(Jagged)`) with
seed 9 at 3 000 off untouched. `riverbank`: the slack (7 000 of 6 000 gets the far bank, 7 600 and a no-grain town do not).
**Mutants** (Python, exact literal replace, occurs-once check, restored in a `finally`): 37 on the new branches (the heading sign, the explore factor,
every threshold and constant, each refusal in `cross_lane` and `legalize_street`, the back street's depth and walk, `keep_rect`, the jag
guard's cap, turn and hull ratio, the slack, the three hand-overs in `generate`). Five survived the first tests (the 30 m station test, the end-node
clearance, the 0.03 reach floor, the guard's `parallel &&`, `generate`'s `keep_rect` flag); a test was added for each and all five were then killed, except the
station test, which `cross_lane`'s own node clearance makes redundant and which was deleted. Two mutants did not match the CRLF text and were re-run on a
single line (one killed; `ring_is_jagged`'s `n < 4` against `n < 3` is equivalent, a triangle being no jagged ring either way). Census, frames, mutant and test logs are in the evidence directory.

**Frames** (owner condition): City Viewer, the 14 towns, opening frame and x4, off and on, windowed under the lock, probe
`RESULT: PASS` x3 (`contact_<seed>.png`, `frames/`; needle crops in `needles/`); the same view as step 0c at gain 0.4.

**Not done, by design.** The switch is OFF at both boundaries until PS-7 (owner). PS-3 onward (block depth, the type rules' per-quarter
variety, the zoom detail) are unbuilt; the baseline above is what they must not worsen.

## 15. PS-3 measured (2026-11-22): NO-GO on its own bars; nothing built but the 30 m access measure

**Verdict: no-go.** A prototype of PS-3 (the back-street tier kept whole, a lane closing each tier, a spindle market
on a market-street town's spine) was built behind `urban_parallel_streets`, measured on the census world against
every PS-3 bar and PS-2's baseline (Ruling GM), swept over its four judgement settings, framed, and reverted. **No
setting meets section 5.3's "depth across 83 m or more at 15 %"** (best 10.2 %), and **every setting that moves that
bar at all worsens PS-2's baseline** (quads outside the core, needle lots), which Ruling GM forbids. By the rule the
PS-2 steps followed, the product code is not kept; it is `scratch_code/ps3_prototype.diff.txt`. **What is kept** is
the measure the brief asked for: section 9.2's "a back lane within 30 m" as `field::rear_served` /
`field::rear_access_m` (`REAR_ACCESS_M`, `REAR_LOT_DEPTH_M`, 30 m each) and a `rear_served` column in
`parallel_streets_audit`'s lot CSV. It reads a finished layout and changes none; PS-5's rear households will read it.
Evidence: `cartalith-native/docs/evidence/parallel-streets-ps3-2026-11-22/` (`summary.txt` lists every file).

**The world moved under PS-2's baseline.** Section 14's figures were measured on the census world of `a15b4ec6`;
since then the world itself changed (among others Ruling GQ's peak law, F27's junction settlements, Ruling GO's
borders), so the census now has 38 towns of 1 500+ at 384 (35 before) and 39 at 2 048, at other places. Every
"before" and "mid" below is **re-measured on today's tree** (`origin/main` `2f539ef8`): before = switch off, mid =
PS-2 as shipped (the prototype with its PS-3 pieces disabled, whose 38 towns' CSV rows are byte-identical to a build
of `origin/main`), after = the prototype at its default settings. **PS-2 on today's world misses its own quads bar**
(58.5 % against 60 %); section 14's 61.5 % was the old world's.

**What the prototype did** (all of it on a town with a grain, `field: Some`; villages, radial and no-grain towns
untouched by control flow):
- **The tier** (`growth::Tier`): when `lay_back_street` laid a rib and walked its back street, the band from the
  frontage to the back street over the walked length was recorded; `legalize_street` refused any later street
  whose midpoint lay inside it by more than `TIER_EDGE_M` (12 m, labelled), so the block behind the frontage stayed
  one block depth (90 m) deep.
- **The closing lane**: a walk that ended free was closed by a lane from its end square back to the frontage
  (`cross_lane` with the start's node clearance waived and 12 m at the frontage), so the tier could close as a
  four-cornered block that `lane_pass_with`'s `keep_rect` keeps.
- **The spindle market** (`plaza::build_plaza_shaped`, Q9): for a town that is market-street by the market point,
  with `urban_market_scale`, the square's far side pulled in by a quarter of its length at each end, so the market
  street widens and tapers inside the square's own rectangle.

**Numbers** (census world, seeds 483920 912345 246371, 800 km, villages on, the app's `params::defaults`, release test
binary; 5.3's rows over the blocks outside the core of the 8 largest towns a seed, 24 towns, as PS-2 reported them;
every other row over the 38 towns of 1 500+):

| 384 | bar | before (off) | mid (PS-2) | after (PS-3 prototype) |
|---|---|---|---|---|
| quads outside the core | 60 %+; PS-2's 58.5 not worse | 51.8 % | 58.5 % | **53.4 %** |
| compact / all triangles | 5 / 15 % or less | 7.3 / 10.8 % | 5.0 / 8.1 % | 5.6 / 9.7 % |
| block aspect, median | 2.0 or more | 1.57 | 1.57 | **1.59** |
| rectangularity, median | 0.75 or more | 0.69 | 0.74 | 0.72 |
| **depth across 83 m+, by count** | **15 % or more** | 4.0 % | 4.1 % | **7.3 %** |
| the same by area (all 38 towns' outside-core blocks; not a bar) | | 18.7 % | 19.3 % | 27.6 % |
| housed p10 (38 towns) | 5.1 | 91 | 97 | 95 |
| FS-1: area ratio in 0.9-1.4 / min | 0.9 or more | | 30 of 38 / 0.80 | 31 of 38 / 0.83 |
| FS-1: 85 % of off kept (re-roll null not re-measured on this world; 10 of 35 on the old one) | the null | | 16 of 38 (0.82) | 18 of 38 (0.85) |
| needle lots / needle buildings | not above PS-2's | 11.5 % / 0.50 % | 12.0 % / 0.62 % | **14.0 % / 0.91 %** |
| FS-2 delta max, market-street / organic / river-front | PS-2's | 0 | 0.164 / 0.433 / 0.207 | **0.311** / 0.362 / **0.277** |
| wall over hull p90 / max; sharp vertices total | 5.1, no spikes | 1.099 / 1.167; 162 | 1.088 / 1.130; 134 | 1.088 / 1.129; 123 |
| lots over 30 m deep / of them a street within 30 m of the rear (`rear_served`) | measurable | 2.4 % / 58.1 % | 3.0 % / 56.9 % | 5.1 % / **40.8 %** (the prototype's run wrote the distance; the same rule applied in `ps1_agg.py`) |

| 2 048 (39 towns; 5.3's rows over all towns' outside-core blocks) | before | mid (PS-2) | after |
|---|---|---|---|
| quads outside the core | 49.6 % | 58.0 % | 52.5 % |
| depth across 83 m+ | 4.1 % | 4.4 % | 7.3 % |
| housed p10 | 91 | 101 | 93 |
| needle lots | 11.4 % | 12.1 % | 14.3 % |
| wall over hull p90 / max | 1.099 / 1.125 | 1.073 / 1.117 | 1.057 / 1.115 |

**The sweep** (384; each a labelled judgement of the prototype, set by an environment knob in a scratch build):

| setting | quads (24) | 83 m+ (24) | needles | housed p10 | FS-2 delta max (market / organic / river) |
|---|---|---|---|---|---|
| default (share 0.5, walk 3, edge 12 m, close, spindle) | 53.4 | 7.3 | 14.0 | 95 | 0.311 / 0.362 / 0.277 |
| every tap a back street (`BACK_SHARE` 1.0) | 54.3 | 9.1 | 16.0 | 98 | 0.292 / 0.268 / 0.279 |
| walk 5 steps | 53.1 | 6.7 | 14.2 | 97 | 0.195 / 0.259 / 0.330 |
| share 1.0 and walk 5 | 52.5 | **10.2** | 15.4 | 100 | 0.258 / 0.266 / 0.297 |
| tier edge 20 m | 54.9 | 7.1 | 14.4 | 95 | 0.172 / 0.281 / 0.211 |
| tiers without the closing lane or the spindle | 54.8 | 7.0 | 13.3 | 92 | 0.148 / 0.249 / 0.144 |
| no tiers (closing lane and spindle only) | 59.5 | 4.3 | 12.8 | 103 | 0.364 / 0.235 / 0.205 |
| the spindle only | 58.6 | 4.0 | 12.1 | 97 | **0.390** / 0.433 / 0.207 |
| the closing lane only | 59.5 | 4.3 | 12.8 | 103 | 0.274 / 0.235 / 0.205 |

**Why it misses.** The tier does what it was built for: by area the deep share rises from 19 % to 28-37 %, about
the traced plans' 32-40 % (section 8.1), and in the block plots (`plots/`) the tiers read as rows of long
rectangular blocks. But 5.3 counts blocks, and a tier is two large blocks among the dozens of small ones grown round
it: at most one back street per tapped frontage, so even with every tap a back street the count reaches 10 %. And
the tiers that do not close square (the frontage bends, or the walk overshoots it) are deep irregular blocks, whose
platting fans into needles (12.0 to 13.3-16.0 %) and whose interior has no lane within 30 m of a rear (the served
share falls from 57 to 41 %): that is PS-5's platting, but PS-3 may not raise the needle bar before it. The
closing lane alone adds needles (12.8 %) at no gain in depth, and the spindle alone moves a market-street town's
order (FS-2 delta 0.164 to 0.390) without a measurable gain in any 5.3 row.

**Frames** (owner condition, Ruling GA): City Viewer, windowed under the lock, opening frame and x4 market-centred,
before / mid / after for 23 settlements of the shell's world at 384 x 288 (seed 483920: 14 towns and a village; seed 912345: 8 towns) (`contact_<seed>.png`, `frames/triple_<seed>_<id>_z1|z4.png`; the combined
sheet `ps3_combined.png`). All looked at. The probe passed in all four runs; the before frames of the two sessions are pixel-identical (46 of 46); the village (483920 #2) is the same in all three, as it must be. **At first sight the prototype is a different town in several places**, which FS-4 would fail: 483920 #8 (walled, market-street) is re-shaped, its wall now round a long deep block and its growth spilling outside the circuit; 483920 #4 and 912345 #0 (river-front) move mass from one bank to the other (483920 #4's far bank fills). 483920 #21 changes type (market-street to river-front: the spindle's outline is nearer the bank). At x4 the tiers read as longer blocks with rows of houses along them (912345 #4, 483920 #4's far bank), but not as a clear gain over PS-2 under scrutiny; the spindle is visible on market-street towns (912345 #8, 483920 #8). The census block plots (`plots/`) show the tiers best: rows of long rectangular blocks in 483920 #6, irregular deep blocks elsewhere.

**Mutants** on the kept measure (`mutants.txt`): 9 literal mutants (Python, exact replace, occurs-once check, restored in a `finally`, hash checked), all killed by `field::tests::a_deep_lot_is_served_from_behind_only_by_a_street_within_30_m_of_its_rear_edge`: `REAR_ACCESS_M` 30 to 29 and to 31; `REAR_LOT_DEPTH_M` 30 to 29 and to 31; `<= REAR_LOT_DEPTH_M` to `<`; `d <= REAR_ACCESS_M` to `<`; the rear edge's middle taken from the side (`lot[1]`-`lot[2]`); the depth taken across the front; the four-corner test at 3. The prototype's constants were not mutated, since none ships.

**What this leaves for the owner.** **Q20.** PS-3's depth bar (83 m+ across at 15 % of blocks by count) is out of
reach of a tier inside `grow` (best 10.2 %), while the same blocks are 28-37 % of the area. Options: (a) restate the
bar by area (the prototype then passes at 27.6-37 %), keep the tier, and let PS-5's platting take the needles and the
rear lanes the deep blocks need; the tier would then be built with PS-5, since alone it worsens PS-2's needle and
quads baseline; (b) lay the spine tier by Option B's deterministic spine and ribs for market-street towns only (the
"too regular" risk, section 3.2) and measure it; (c) stop parallel streets at PS-2 and go to PS-4 (river-front towns)
and PS-5 with the tier left out. [(a): the bar was set against the traced plans' 32-40 %, which the area figure
matches; a count bar cannot tell two deep blocks from twenty shallow ones] **Q21.** The spindle market moves
market-street towns' order and adds nothing to 5.3. Keep it for its look (frames), or drop it? [drop it from PS-3;
put it to the owner on the frames] **Q22.** PS-2's quads on today's world are 58.5 %, under the 60 % bar it passed on
the old world. Accept PS-2 as built (Ruling GM accepted it on look) or re-tune? [accept; record 58.5 % as the
baseline]

## 16. PS-4 measured (2026-11-22): NO-GO on its own riverside bar; nothing built

**Verdict: no-go.** PS-4's three pieces (section 6: the stretch frame as a field source, lanes to the water, far-bank
growth past 1.4x where section 9.3's gate passes) and a fourth found on the way (a stretched town takes the
switch-off town's stretch choice) were prototyped behind `urban_parallel_streets`, measured on the census world at 384
and 2 048 against PS-4's bars and PS-2's baseline on today's world (Ruling GY), swept, framed and reverted. **No
setting meets PS-4's riverside bar at both sizes** (4.3 / 3.7 % or better; today, switch off, 4.1 / 3.9 %; PS-2 3.8 /
3.5 %): the best at 2 048 is 3.5 %. The product code is `scratch_code/ps4_prototype.diff.txt` and
`ps4_stretch_choice.diff.txt`. Evidence: `cartalith-native/docs/evidence/parallel-streets-ps4-2026-11-22/`
(`summary.txt` lists every file). Census world as section 15, today's tree `origin/main` `d8a30493`; before = switch
off, mid = PS-2 as shipped, after = the best candidate, E (below).

**The pieces, each measured on and off (384):**
- **A, the stretch frame as a field source** (`field::sources_for`): a stretched river-front town's river grain reaches
  the frame's depth across (`StretchPlan::across_m`, about 450 m) instead of 80 m, so the whole stretched town takes the
  river's grain. It changes which stretched layouts pass Ruling FQ's `stretch_holds` (refused 4 to 0-2), and it
  strips 483920 #2's quay (180 m to 0) whenever that town is laid stretched with it: a stretch-water flag, which the
  bar forbids. At reach 0.25-0.75 of the depth: riverside 3.7-4.2 %, the river-front FS-2 delta 0.244-0.281 against
  PS-2's 0.207. Dropped.
- **B, lanes to the water** (`riverbank::lay_back_street`'s water lanes every 30-60 m for a river-front town, half
  the base spacing): **inert.** Every census run with it is identical to the same run without it to the last digit.
  The cause, read in the fixture (seeds 4240-4251, a town of 6 000): every water lane is refused by
  `crosses_wall`, because the wall of a one-bank river town follows its bank (`fortify`, "let the wall follow the bank
  on the water") between the back line and the embankment, and all 38 / 39 census towns of 1 500+ are walled. So
  change 5's water lanes (2026-11-15) have never been laid in a walled town, and PS-4's "lanes to the water" needs a
  rule for passing that wall (a water gate or postern per lane) before it can move anything (Q23). Dropped.
- **C, the far bank in the grain** (`growth::grow_far_bank` bending its headings and keeping PS-2's oblique-T refusal,
  section 9.3's "run in the grain's field"): on its unit fixture the far bank's streets are on an axis for 100 % of
  their length against 76 %; on the census, with E, riverside 3.8-4.1 % (E alone 4.1 %) and no other gain.
  Dropped.
- **D, far-bank growth past the shortfall** (`riverbank::second_pass`: a river-front town that fits section 9.3's gate
  at `F_REQ_MEDIAN` is given the far bank with a share of at least 0.25 or 0.5 even when housed; `FAR_BANK_MAX_RATIO`
  read, not raised): riverside 4.1-4.3 % at 384, 3.5 % at 2 048; FS-1's area ratio in 0.9-1.4 falls from 30 to 23-27
  of 38 towns, housed median 133 %. It grows exactly the cross-bank mass section 15 watched for. Dropped.
- **E, the switch-off town's stretch choice** (`urban_adapter::run_layout_choosing`: with the switch on, Ruling FQ's
  choice (full, milder, plain) is made from the switch-off layouts and replayed with the switch on): found while
  reading A. PS-2 makes another choice than the switch-off town for 8 of the 38 towns at 384, and 4 of them keep
  under 70 % of the switch-off footprint (246371 #0 0.21, 483920 #6 0.68, #11 0.20, #20 0.14); with E they keep 0.86-0.91.
  It is the best candidate, and it fails at 2 048 (below).

**Bars, E against before and mid** (24 towns' outside-core blocks for quads, as PS-2 and PS-3 reported them; every
other row over the 38 / 39 towns of 1 500+; housed from `both_banks_audit`):

| bar | 384 before / mid / **after** | 2 048 before / mid / **after** |
|---|---|---|
| riverside, river towns 1 500+ (4.3 / 3.7 % or better) | 4.1 / 3.8 / **4.1 %** | 3.9 / 3.5 / **3.4 %** |
| stretched towns refused (no more than today) | 2 / 4 / **2** | 0 / 1 / **0** |
| buildings in water; stretch-water flags | 0; 0 / 0; 0 / **0; 0** | 0; 0 / 0; 0 / **0; 0** |
| quads outside the core (PS-2's 58.5 % not worse) | 51.8 / 58.5 / **58.8 %** | 48.4 / 57.7 / **57.9 %** |
| needle lots (PS-2's 12.0 % not worse); needle buildings | 11.5 / 12.0 / **11.6 %**; 0.50 / 0.62 / 0.60 % | 11.4 / 12.1 / **12.2 %**; 0.60 / 0.64 / 0.64 % |
| FS-1: towns keeping 85 % of the off footprint | 38 / 16 / **22** of 38 | 39 / 21 / **20** of 39 |
| FS-2 delta max, market-street / organic / river-front | 0 / 0.164, 0.433, 0.207 / **0.164, 0.433, 0.163** | 0 / 0.202, 0.392, 0.228 / **0.202, 0.392, 0.228** |
| wall over hull p90 / max; sharp vertices total | 1.099 / 1.167; 162 -- 1.088 / 1.130; 134 -- **1.106 / 1.131; 140** | 1.099 / 1.125; 170 -- 1.073 / 1.117; 120 -- **1.092 / 1.117; 118** |
| housed p10 | 91 / 97 / **96** | 91 / 101 / **100** |
| unbuilt patches inside the wall over 1 ha; squares outside the wall | 0; 1 / 5; 0 / **4; 1** | 0; 1 / 2; 0 / **3; 0** |

At 384 E meets every bar but the scope's 4.3 % (it holds Ruling FS's floor, 4.1 %); at 2 048 it misses the riverside
bar (3.4 against 3.7 %, and under PS-2's 3.5 %), adds a needle-lot tenth over PS-2's baseline, keeps one town fewer,
and adds an unbuilt walled patch over 1 ha. It changes one town at 2 048, so its gain there is nil. **Why riverside
misses:** PS-2 houses more than today (median 126-127 % of target against 106-107 %), and the extra houses stand
inland, so the riverside share falls whatever the stretch does; the lever that would add houses at the water, B's
lanes, is held behind the riverside wall.

**Also measured: PS-2 itself leaves unbuilt walled patches over 1 ha** (5 at 384, 2 at 2 048; switch off 0 and 0) and
fails PS-4's riverside bar; neither was in section 14's table.

**Frames** (owner condition): City Viewer, the shell's world at 384 x 288, windowed under the lock, opening frame and
x4 market-centred, before / mid / after (E) for 16 settlements (`contact_<seed>.png`, `frames/`, the combined sheet
`ps4_combined.png`); all looked at; probe PASS in all four runs; the before frames of the two sessions are
pixel-identical (32 of 32). E changes two of them: 483920 #3 and 912345 #20, each from PS-2's square layout back to
the switch-off town's stretched one (912345 #20's x4 frame, empty in PS-2's layout, is the town again); 483920 #4 and
912345 #0, the towns section 15 watched, are PS-2's in both. The controls (483920 #5, #13, the village #2; 912345 #8,
#32) are unchanged. 912345 #4's opening frame differs between before and mid / after (PS-2's, not PS-4's).

**Mutants** on E (`mutants.txt`): 7 literal mutants, 6 killed by its test; the seventh (dropping the switch test) is
equivalent: an off context takes its own choice and replays it to the same layout. Reverted with E.

**What this leaves for the owner.** **Q23.** Change 5's water lanes are refused by the riverside wall in every walled
town, so "lanes to the water" cannot act as written. Options: (a) a water gate (postern) where a lane meets the
riverside curtain, so the lanes are laid and the wall drawn with a gate there; (b) leave the waterfront to the back
street and drop the lanes from PS-4; (c) let the riverside curtain stand at the water's edge, outside the lanes. [(a),
measured first: it is the historical form (water gates on river walls) and the only one that adds houses at the water]
**Q24.** E (the switch-off town's stretch choice) repairs PS-2's first-sight changes at 384 (4 towns) but gains nothing
at 2 048 and misses the riverside bar there. Build it on its own as a PS-2 fix, judged on the frames, or fold it into
PS-6? [build it on its own: the frames of 483920 #3 and 912345 #20 are the base's towns again] **Q25.** PS-4's riverside
bar is out of reach while the grain over-houses (median 126 % of target): restate it against PS-2's 3.8 / 3.5 %, or
make PS-5's rear households and platting carry it (fewer, deeper lots at the same housing)? [carry it with PS-5;
restate against PS-2 meanwhile]

## 17. Ruling HB built and measured (2026-11-22): fix E BUILT, OFF at both boundaries; the water gate NO-GO, reverted

**Fix E is built** (Q24): with `urban_parallel_streets` on, a stretched town takes the switch-off town's stretch choice
(Ruling FQ's full, milder or plain), which is then replayed with the switch on (`urban_adapter::run_layout_choosing`,
`choose_stretch`). It is section 16's prototype, re-applied unchanged but for its prose. Nothing changes with the switch
off; an unstretched town is laid out once, as before. Hosts cache the verdict by `stretch_choice_key`, whose hash
carries `parallel_streets` (read at the symbol). The switch stays OFF at both boundaries. **The water gate (Q23 (a)) was
built, measured, framed and reverted**: it meets PS-4's riverside bar at both sizes for the first time, but it raises
PS-2's needle-lot baseline at both sizes, which Ruling GM forbids (section 15 declared PS-3 a no-go on the same row).
Evidence: `cartalith-native/docs/evidence/parallel-streets-E-gate-2026-11-22/` (`summary.txt` lists every file).

**Numbers** (census world as section 15, today's tree `origin/main` `7cd2de2b`; before = switch off, mid = PS-2 as
shipped, E = what ships, gate = E plus the water gate; quads over the 24 towns' outside-core blocks, every other row over
the 38 / 39 towns of 1 500+; housed from `both_banks_audit`). Before and mid reproduce section 16's figures exactly, and
E's rows reproduce its E rows:

| bar | 384 before / mid / **E** / gate | 2 048 before / mid / **E** / gate |
|---|---|---|
| riverside, river towns 1 500+ (PS-4's 4.3 / 3.7 %; Q25: judged against PS-2's 3.8 / 3.5 %) | 4.1 / 3.8 / **4.1** / 4.5 % | 3.9 / 3.5 / **3.4** / 4.0 % |
| stretched towns refused (no more than off) | 2 / 4 / **2** / 2 | 0 / 1 / **0** / 0 |
| buildings in water; stretch-water flags | 0; 0 in every column | 0; 0 in every column |
| quads outside the core (PS-2's 58.5 % not worse) | 51.8 / 58.5 / **58.8** / 59.5 % | 48.4 / 57.7 / **57.9** / 58.2 % |
| needle lots (PS-2's 12.0 % not worse); needle buildings | 11.5 / 12.0 / **11.6** / 12.1 %; 0.50 / 0.62 / **0.60** / 0.56 % | 11.4 / 12.1 / **12.2** / 12.6 %; 0.60 / 0.64 / **0.64** / 0.67 % |
| FS-1: towns keeping 85 % of the off footprint (PS-4's prototype 22 / 20) | 38 / 16 / **22** / 21 of 38 | 39 / 21 / **20** / 21 of 39 |
| FS-2 delta max, market-street / organic / river-front | 0 / 0.164, 0.433, 0.207 / **0.164, 0.433, 0.163** / 0.158, 0.428, 0.163 | 0 / 0.202, 0.392, 0.228 / **0.202, 0.392, 0.228** / 0.230, 0.391, 0.218 |
| wall over hull p90 / max; sharp vertices total | 1.099 / 1.167; 162 -- 1.088 / 1.130; 134 -- **1.106 / 1.131; 140** -- 1.088 / 1.130; 144 | 1.099 / 1.125; 170 -- 1.073 / 1.117; 120 -- **1.092 / 1.117; 118** -- 1.097 / 1.113; 124 |
| housed p10 | 91 / 97 / **96** / 96 | 91 / 101 / **100** / 104 |
| unbuilt patches inside the wall over 1 ha; squares outside the wall | 0; 1 / 5; 0 / **4; 1** / 4; 0 | 0; 1 / 2; 0 / **3; 0** / 3; 0 |
| water lanes laid in walled towns with a grain, summed over `generate()` calls (not towns) | | 85 / 599 (E / gate) | 41 / 407 |

**E against its bars.** At 384 E meets every bar it was given: FS-1's kept share 22 (the prototype's 22), refused 2 (off
2), needles 11.6 % and quads 58.8 % (PS-2 12.0 / 58.5), riverside 4.1 % against PS-2's 3.8 %. **At 2 048 it is not green
on three rows**, as section 16 already measured and the owner ruled Q24 with those figures in hand: riverside 3.4 %
against PS-2's 3.5 %, needles 12.2 % against 12.1 %, FS-1 20 against 21 towns. E changes one town at 2 048, so these are
that town's moves. The switch-off layouts are untouched: the towns CSV's off rows are identical to `origin/main`'s but
for the `ptype` label of 4 towns, which the audit takes from the switch-on plan; `both_banks_audit` off is identical but
for timing.

**The water gate, as measured** (`scratch_code/water_gate.diff.txt`): `riverbank::lay_back_street_with(.., water_gate)`
laid a water lane through the town's wall instead of refusing it (`crosses_wall`), and `generate`, after the wall hug and
the shore snap, opened a `Gate { water: true }` wherever a live water lane crossed the finished circuit
(`riverbank::water_gates`), so `cleanup::clear_fort_zone` kept the lane. It applied to a town with a grain only, the
predicate PS-2 uses for "only a town with a grain changes". **Which switch:** `urban_parallel_streets`, not
`urban_riverbank`, which owns change 5. It is a PS-4 piece, and putting it behind `urban_riverbank` would change shipped
towns with no ruling. Its two tests passed (8 lanes and 8 gates on the 20 m wall fixture; 89 lanes kept through the wall
at gates over seeds 4240-4251, none with the switch off). Laid lanes rose about sevenfold (table). The riverside share
rose to 4.5 / 4.0 %, which meets PS-4's own bar at both sizes, and 384's housed p10 held. But the lots it adds are mostly
needles: at 384, 1 110 more lots and 638 more needle lots. **Needle lots rose to 12.1 / 12.6 % against PS-2's 12.0 /
12.1 %**, and at 2 048 needle buildings rose too (0.67 against 0.64 %). On the frames it changes 21 of the 27 settlements shot. Some
look more like the switch-off town (912345 #4 gets its far bank back; 483920 #6 loses PS-2's small walled annex on the
far bank). One goes the other way at first sight: 483920 #7 (2 056 people) turns from PS-2's compact walled town into a
long wall round a mostly empty field, as the switch-off town has it. The frame shows this; its cause was not traced.
Neither the shell nor the reference draws water gates (`wall_water_gates` is skipped), so a gated lane would cross the
drawn curtain with no opening drawn. **By Ruling GM the gate is a no-go; it is reverted and E ships alone.**

**A correction to section 16.** Its B piece found "change 5's water lanes have never been laid in a walled town". That is
too strong. Without the gate, walled towns with a grain lay 85 lanes at 384 and 41 at 2 048 (per `generate()` call), where
a back line inside the wall stands clear of the riverside curtain. The finding holds for most lanes, not all.

**Frames** (owner condition): City Viewer, the shell's world at 384 x 288, windowed under the lock, opening frame and x4
market-centred, before / mid / E / gate for every town of 1 500+ (26) and the village 483920 #2. They are in
`contact_<seed>.png` and `frames/quad_*.png` (kept for the changed towns and the controls), with the combined sheet in
`combined.png`. All were looked at. The probe passed in all six runs, and the before frames of the three sessions are
pixel-identical (108 of 108). E changes three settlements, each back to the switch-off town's stretched shape:
483920 #3 (5 097 to 4 773 buildings), 483920 #21 (2 870 to 2 956; PS-2's far-bank sprawl gone, the V-shaped two-bank
town again) and 912345 #20 (3 614 to 3 239; its x4 frame, empty in PS-2's layout, is the town again). The controls are
unchanged under E: 483920 #5 and #13, the village #2, and 912345 #8 and #32. The frames came from a debug DLL built from
the tree with the scratch knobs in.

**Mutants** on E (`mutants.txt`): 7 literal mutants, 6 killed by
`urban_adapter::tests::a_town_with_a_grain_is_stretched_where_the_switch_off_town_is`. The seventh (the switch test
dropped) is equivalent, as in section 16. E ships no numeric constant.

**What this leaves for the owner.** **Q26.** The water gate meets PS-4's riverside bar at both sizes (4.5 / 4.0 %), but
it adds needle lots over PS-2's baseline (12.1 / 12.6 %). Build it with PS-5, whose platting is to carry the needles and
the riverside bar (Q25), or drop it? If it comes back, the shell must draw a postern where a gated lane crosses the
curtain. [build it with PS-5, measured with PS-5's platting; draw the postern with it] **Q27.** E misses PS-2 at 2 048 on
riverside (3.4 / 3.5 %), needles (12.2 / 12.1 %) and FS-1 (20 / 21) through the one town it changes there. Accept it as
Q24 did, or hold it? [accept: those are section 16's figures, which Q24 ruled on]

## 18. PS-5 measured (2026-11-23, owner Rulings GY, HB, HD, HT): NO-GO on its bars; nothing built

**Verdict: no-go.** PS-5's four pieces (platting by quarter on `burgage`'s depths, rear households served from behind,
PS-3's back-street tier, the water gate with a postern drawn in the wall) were prototyped behind
`urban_parallel_streets`, measured on the census world at 384 and 2 048 against PS-5's bars and PS-2's baseline
(Ruling GM), swept, framed and reverted. **No variant holds every bar.** The variants that leave the block shapes alone
(platting, rear households, the gate) hold PS-2's baseline but do not move depth by area at all (18.6-18.9 % against
19.2 %), and so do none of PS-5's purpose; every variant with the tier moves depth by area toward the traced plans'
32-40 % (26.9 % at 384, 30.8 % at 2 048) but drops quads by count under 58.5 % (54.4 / 52.9 %) and housed p10 under
PS-2's (91 / 87 against 96 / 100; at 2 048 under the switch-off town's 91). The product code is
`scratch_code/ps5_prototype.diff.txt` (all four pieces, the tier variant `PS5_DEEPALL`, the scratch knobs marked
`SCRATCH-PS5`, its unit tests). Evidence: `cartalith-native/docs/evidence/parallel-streets-ps5-2026-11-23/`
(`summary.txt` lists every file). Census world as section 15, today's tree `origin/main` `69a88c48`; before = switch
off, mid = PS-2 and fix E as shipped (both reproduce section 17's before and E columns exactly).

**The pieces, as prototyped:**
- **Platting by quarter** (Ruling HT: no second platting code). `blocks::build_parcels_with` took a per-block `tier`
  test; a block it accepted was platted with `burgage::DEPTH_CORE_M` / `DEPTH_M` / `DEPTH_MAX_M` / `WAIST`, and each
  edge's frontage grant was floored at the depth its lots reach over 5 (`TIER_MAX_RATIO`, labelled; section 8.4's
  needle ratios), the burgage cycle splitting a frontage only where both halves keep the floor. The test
  (`field::in_tier`): a four-cornered block of rectangularity 0.75+ (`KEEP_RECT`, scope 8.6 item 4) whose centroid lies
  in a grain of strength 0.5+ (`PARALLEL_STRONG`); variant E (`PS5_DEEPALL`) also every grain block 60 m or more deep
  whatever its shape. A block it refused was the reference's draw for draw (unit test).
- **Rear households** (section 9.2, Rulings GC and GF). `districts::build_rear_households`: for the demand the front
  lots left, a cottage across the back 8 m of a lot over 30 m deep with a street within 30 m of its rear edge
  (`field::rear_served`, PS-3's measure; the brief's "back lane within 30 m", so the 2.5 m side passage Ruling GF also
  allows was not built), nearest the market first, at most one a lot and on at most 40 % of the lots, 4 m clear of
  every other building on its lot, each one household of 5.2 in `Town::pop`.
- **The back-street tier** (PS-3's prototype, section 15, without the spindle market, Ruling GY Q21): `growth::Tier`
  refuses a street inside the band behind a frontage; a free end is closed by a lane. Variant D kept only tiers that
  closed at both ends.
- **The water gate** (section 17's prototype, Ruling HD Q26): water lanes laid through the riverside wall and a
  `Gate { water: true }` opened on the finished ring **after** `wallhug::wall_off_water` (which reads water gates as
  leave for a wet run to stay; section 17's gate predates it), the posterns bridged as `wall_posterns` and drawn in
  `urban_layout_draw.gd` (a small gate) and `urban_layout_look.gd` (a short paved opening); a bastioned trace draws no
  gate, as before. `urban_adapter::layout_from_town`'s ring-off-the-drawn-water and the town census's gated-wall test
  both read every water gate, so a postern would have let a wet wall run stand and pass the census vacuously; both
  were made to skip posterns.

**Numbers** (24 towns' outside-core blocks for quads and depth, as before; every other row over the 38 / 39 towns of 1 500+;
riverside and squares from `both_banks_audit`; quads and depth by area from `area.txt`). Variants: A platting + rear;
B A + gate; C B + tier; D C with only closed tiers; E C with the deep platting on every deep grain block. A, B, C and D
took the floor from the block's target depth, A3 and B3 and E from the depth the lots reach (below):

| 384 | quads count / area | 83 m+ by area | needles / bldg | housed p10 | FS-1 kept | riverside | squares out; unbuilt > 1 ha | wall / hull p90, max; sharp | rear |
|---|---|---|---|---|---|---|---|---|---|
| before (off) | 51.8 / 34.4 | 19.3 | 11.5 / 0.50 | 91 | 38 | 4.1 | 1; 0 | 1.099, 1.167; 162 | |
| mid (PS-2 + E) | **58.8 / 39.6** | 19.2 | **11.6** / 0.60 | 96 | 22 | 4.1 | 1; 4 | 1.106, 1.131; 140 | |
| A3 | 58.6 / 39.2 | 19.1 | 11.4 / 0.56 | 96 | 22 | 4.0 | 0; 5 | 1.087, 1.130; 148 | 99 |
| B3 | 59.4 / 40.3 | 18.6 | 12.0 / 0.57 | 96 | 22 | 4.5 | 0; 4 | 1.086, 1.130; 147 | 92 |
| B (target floor) | 59.2 / 40.0 | 18.9 | 11.5 / 0.55 | 95 | 21 | 4.4 | 0; 4 | 1.090, 1.135; 131 | 102 |
| C | 54.5 / 40.3 | 26.4 | 12.9 / 0.69 | 93 | 24 | 4.3 | 4; 3 | 1.095, 1.129; 145 | 144 |
| D | 58.0 / 42.6 | 22.5 | 12.2 / 0.62 | 94 | 23 | | | 1.086, 1.136; 129 | 96 |
| **E** | **54.4 / 40.0** | **26.9** | **11.6 / 0.58** | **91** | 24 | **4.6** | **3**; 4 | 1.096, 1.129; 137 | 173 |

| 2 048 | quads count / area | 83 m+ by area | needles / bldg | housed p10 | FS-1 kept | riverside | squares out; unbuilt > 1 ha | wall / hull p90, max; sharp | rear |
|---|---|---|---|---|---|---|---|---|---|
| before (off) | 48.4 / 29.0 | 25.8 | 11.4 / 0.60 | 91 | 39 | 3.9 | 1; 0 | 1.099, 1.125; 170 | |
| mid (PS-2 + E) | **57.9 / 39.1** | 23.2 | **12.2** / 0.64 | 100 | 20 | 3.4 | 0; 3 | 1.092, 1.117; 118 | |
| B3 | 58.0 / 39.5 | 22.9 | 12.2 / 0.62 | 102 | 20 | 4.1 | 0; 2 | 1.098, 1.116; 134 | 9 |
| B (target floor) | 57.7 / 39.0 | 23.5 | 11.6 / 0.65 | 98 | 20 | 4.0 | 0; 2 | 1.101, 1.121; 118 | 85 |
| **E** | **52.9 / 39.1** | **30.8** | **11.8 / 0.63** | **87** | 22 | **4.3** | 1; 3 | 1.104, 1.134; 152 | 196 |

Every run: buildings in water 0, stretch-water flags 0, stretched towns refused as off (2 / 0). FS-2 (market-street
phi on minus off, max): mid 0.164 / 0.202, E 0.167 / **0.319** (over 0.30 at 2 048; Ruling GM judges it against a
re-roll). Section 5.2's lot bars are out of reach in every variant: lots over 40 m deep at most 1.0 % (bar 10 %),
depth over frontage median at most 1.49 (bar 3).

**Why it misses.**
- **Without the tier there is nothing deep to plat.** The grain's near-rectangular blocks are about 30 m across, so a
  lot reaches the 0.48 waist, about 14 m, whatever its target. The first build floored the frontage at the *target*
  depth (42 m over 5, 8.4 m) and so platted wide shallow lots: its needle gain (B, 11.5 %) was a frontage floor, not
  deep plots. Taken from the depth the lots reach (B3), the gain is gone (12.0 / 12.2 %, PS-2's figures), and the gate
  alone is section 17's result again.
- **The tier makes the deep blocks and costs the count.** It joins many small quads into fewer long blocks: by count
  quads fall to 54.4 / 52.9 %, by area they hold (40.0 / 39.1 % against 39.6 / 39.1 %). Platted with the deep depths
  and the floor (E), the tier's deep irregular blocks no longer fan into needles (C 12.9 %, E 11.6 / 11.8 %).
- **Housing falls with the tier.** Deep lots are fewer lots, and rear households served by a street within 30 m of
  the back are few (173 / 196 households in 7 / 6 towns; 41 / 37 % of the lots over 30 m deep are served), so housed p10 falls
  to 91 / 87. The 2.5 m side passage Ruling GF allows would serve the rest.
- **The tier also puts squares outside the wall** (3 at 384 against 1; C 4); not traced.

**The town-on-bank census with the switch on** (`town_census/`, `_towncensus_probe.gd --rows` on four seeds at 384,
`civ.bank_sites` off and on; before = the switch off, mid = PS-2 by `--params urban_parallel_streets: true`, after =
E): no building or wall run over the drawn river, lake or sea in any build; one street edge over the drawn river in
seed 483920 in **mid and after alike** (PS-2's, not PS-5's: the probe exits 1 on it in both); markers across from
their market 0 at x1-x64 in all three; bank-sites-on housed p10 held (after 92.3 / 85.8 / 91.0 / 91.0 against mid
92.3 / 85.8 / 89.8 / 91.0); no town loses its crossing against before (`fate.txt`: 2 bridges, 37 ferries; mid 38). Two
towns' ferries differ from mid: 483920 #11 (a ferry PS-2 added; E has none, as off) and 912345 #7 (lost in mid and
after alike).

**Frames:** City Viewer, the shell's world at 384 x 288, windowed under `C:/cw/.win_lock` (scratch APPDATA), opening frame and x4 market-centred, every town of 1 500+ the probe lists and a village a seed (483920: 17 towns and #9; 912345: 8 towns and #1); before and mid from the base build's dll (off, then on live), after from the prototype's dll (E). Probe `RESULT: PASS` in all four sessions; the two sessions' before frames are pixel-identical (54 of 54); the villages are the same in all three. All looked at (`contact_<seed>.png`, `combined.png`, `frames/triple_*`). At first sight E keeps PS-2's towns: the silhouettes that move are PS-2's own (483920 #8 a long wall round a thin town, 912345 #4 laid round another centre, as in mid). E adds long rectangular blocks along the spines (912345 #13's south half, 483920 #11 and #20 at x4), and one first-sight change of its own: 483920 #15's wall is routed along the shore with dense towers at x4 (mid: on land, few towers). Posterns were not looked for pixel by pixel.

**Mutants and tests.** Nothing ships, so no constant is mutated. The prototype carried unit tests (in the diff): the
tier test on literal blocks; the platting (a refused block plats as the reference; on 90 m blocks 456 lots with 217
needles become 252 with 41; on 30 m and 63 m blocks the reference's 4.5 m and the split's 0.4 bind); rear households
(the 40 % cap at 10 and 12 lots, the 4 m setback both ways, the 3 m back, the 30 m access, nearest first, ids);
section 17's two water-gate tests, passing unchanged (8 lanes and gates; 89 over seeds 4240-4251).

**What this leaves for the owner.** **Q28.** Restate PS-2's quads baseline by area, as Ruling GY restated depth? By
area the full prototype holds it (40.0 / 39.1 % against 39.6 / 39.1 %), and a count bar cannot tell a few long blocks
from many short ones. [yes, by area] **Q29.** Housing falls with the tier (housed p10 91 / 87 against 96 / 100). Let a
rear household be reached by the 2.5 m side passage Ruling GF already allows, as well as by a street within 30 m?
[yes: build the passage, measured] **Q30.** With Q28 and Q29, one more measured attempt at E (squares outside the wall
traced first), or stop parallel streets at PS-2 and fix E and go to PS-6 / PS-7? [one more attempt] **Ruling HT's
switch question:** `urban_burgage` keeps its own switch (off at both boundaries): the prototype read its depths
through `build_parcels_with` and wrote no second platting; it is scrubbed at PS-7 with the rest (Q7) if the owner
turns parallel streets on.

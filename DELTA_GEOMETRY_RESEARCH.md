# Delta geometry: research, diagnosis and redesign

**Agent-produced 2026-11-11**, from the owner's remark on the Ruling EN before/after frames: *"deltas don't look
natural"* (`OUTSTANDING_WORK.md` §2.5, "Natural-looking river deltas"). Research, a reading of the code at its
symbols, a redesign, and its build notes. It defines and measures; it does not track status (`STATUS.md` does).

**Evidence tags.** CONFIRMED: read on a page or in a paper's own text. SECONDARY: read only in an abstract, a
citing paper or a summary. UNCONFIRMED: from memory, or not found. Most publisher PDFs could not be fetched from
this environment, so most citations below are SECONDARY, and that is said each time.

---

## 1. How real deltas branch

1.1 **Mouth-bar bifurcation.** A river leaving its mouth spreads as a turbulent jet, slows, and drops a bar in
front of itself; the flow parts round the bar, and each half repeats the process. Edmonds & Slingerland 2007,
"Mechanics of river mouth bar formation: implications for the morphodynamics of delta distributary networks",
*JGR* 112, F02034, doi:10.1029/2006JF000574 — the distance to the bar scales with the jet's momentum flux and
inversely with grain size. SECONDARY.

1.2 **The distance to the next split is about ten channel widths.** Jerolmack & Swenson 2007, "Scaling
relationships and evolution of distributary networks on wave-influenced deltas", *GRL* 34, L23402,
doi:10.1029/2007GL031823, restate Edmonds' result as `L = a B`, with `B` the channel width and `a` "of order
ten". SECONDARY: the citing paper's summary; Edmonds' coefficient was not read first-hand. Edmonds & Slingerland
2007 also report that distributary widths, depths and lengths **decrease with successive bifurcations**, across
11 deltas including Wax Lake. SECONDARY.

1.3 **The bifurcation angle is about 70 degrees.** Coffey & Shaw 2017, "Congruent bifurcation angles in river
delta and tributary channel networks", *GRL* 44, 11 427-11 436: theory predicts about 72 degrees, field deltas
average 70.4 (n = 222 bifurcations in 10 deltas), experiments 68.3 ± 8.7 (n = 21). The angle holds at small scale
relative to channel width and is smaller at braid-bar and avulsion splits. SECONDARY: the figures come from an
abstract and a citing paper.

1.4 **Splits are unequal.** Edmonds & Slingerland 2008, "Stability of delta distributary networks and their
bifurcations", *WRR* 44, W09426, doi:10.1029/2008WR006992 (Delft3D, compared with the Mossy delta): three
equilibria, one symmetric and two asymmetric, and **only the asymmetric ones are stable**, which may explain why
asymmetric bifurcations are common. SECONDARY; no numeric daughter ratio was found. Kleinhans et al. 2013,
"Splitting rivers at their seams", *ESPL* 38, 47-61, doi:10.1002/esp.3268, reviews bifurcation stability and
avulsion. SECONDARY. Dong et al. 2020 (*WRR*, doi:10.1029/2020WR027199) predict the discharge split from the
downstream width ratio, tested on Wax Lake. SECONDARY.

1.5 **Avulsion.** Jerolmack & Mohrig 2007, "Conditions for branching in depositional rivers", *Geology* 35: a
mobility number (bank erosion against channel aggradation) predicts when a river branches; branching happens when
avulsion, not bank migration, dominates. SECONDARY (from the conference abstract). Slingerland & Smith 2004,
"River avulsion and its deposits", *Annu. Rev. Earth Planet. Sci.* 32: UNCONFIRMED, not opened. Jerolmack &
Swenson 2007 distinguish two kinds of distributary: mouth-bar ones, whose length scales with the parent's width,
and avulsive ones, whose length scales with the backwater length. SECONDARY.

1.6 **Topology.** Tejedor et al. 2015, "Delta channel networks: 1. A graph-theoretic approach", *WRR* 51,
3998-4018, doi:10.1002/2014WR016577: a delta is a rooted directed graph, apex the root, shoreline outlets the
leaves (Wax Lake, Niger). SECONDARY; no channel or loop counts recovered. A Wax Lake "recombination factor" of
0.32 (more splitting than rejoining) is reported by a project page. SECONDARY.

## 2. Distributary count against discharge, waves and sediment

2.1 **Waves suppress branching.** Jerolmack & Swenson 2007's abstract: wave energy controls network topology by
suppressing mouth-bar growth, preferentially removing the smaller distributaries. SECONDARY. Nienhuis et al.
2020, "Global-scale human impact on delta morphology has led to net land area gain", *Nature* 577, 514-518,
doi:10.1038/s41586-019-1905-9: `Q_river` is split across the distributaries while `Q_wave` and `Q_tide` act on
each mouth, and because `Q_wave` suppresses bifurcation the number of distributaries "could potentially be
predicted"; the paper also notes that no global distributary-network data exist. CONFIRMED (paper text read).

2.2 **Widths.** Syvitski & Saito 2007, *Global Planet. Change* 57, 261-282, doi:10.1016/j.gloplacha.2006.12.001
(51 deltas): distributary widths are lognormally distributed; cumulative mouth width tracks maximum discharge and
wave and tidal energy. SECONDARY. Olariu & Bhattacharya 2006, *J. Sed. Res.* 76, 212-233,
doi:10.2110/jsr.2006.026: terminal distributary channels are much smaller than the trunk (commonly 100-400 m
wide, 1-3 m deep); their number is set by basin depth relative to channel depth and by discharge; many coeval
terminal channels make a **lobate** delta, few long ones the **digitate** Mississippi. SECONDARY.

2.3 **Per-type counts.** No source found giving a count of active distributaries per delta type. A wave-dominated
delta has "about one" channel (a coastal-dynamics textbook summary). SECONDARY.

## 3. Plan shapes (Galloway 1975; Wright & Coleman 1973; Nienhuis 2020)

3.1 Galloway 1975 (in Broussard, ed., *Deltas: Models for Exploration*) places deltas on a triangle of river,
wave and tide dominance. SECONDARY.

| Type | Plan shape | Distributaries | Examples |
|---|---|---|---|
| River-dominated | elongate (bird's foot) or lobate; indented coast with bays and marsh between channels | many; long, levee-bound, digitate | Mississippi, Po, Danube |
| Wave-dominated | smooth, arcuate to cuspate coast; beach ridges | few, about one | Rhone, São Francisco, Ebro, Tiber |
| Tide-dominated | funnel-shaped, embayed; channels flare seaward; tidal flats | many, wide, linked | Ganges-Brahmaputra, Fly, Colorado |

All rows SECONDARY (Galloway's abstract and textbook summaries); the Nile is classed arcuate (wave-influenced) by
some sources and river-dominated by others. Wright & Coleman 1973 compared 16 deltas, the Mississippi the most
river-dominated and the São Francisco the most wave-dominated. SECONDARY. "Funnel-shaped, flared mouths" for the
tidal class: UNCONFIRMED (no source read).

3.2 **Nienhuis 2020's classifier** (CONFIRMED, paper text): `R = Q_river / Q_wave`, the fluvial dominance ratio;
without tides a delta is wave-dominated (triangular, cuspate) when `Q_wave` exceeds `Q_river`, and `R` near 0
means no coastline deflection. `T = Q_tide / Q_river`, with `Q_tide` from the tidal prism. The model outputs the
delta's protrusion angle and the channels' width divergence. Over 10 848 deltas, about 79 ± 9 % of present-day
morphology is wave-dominated. Numeric `R`/`T` class thresholds were not recovered. The source project's own
v2.41 (`RC_ENGINE_CHANGES.md` "Deltas, which this unblocks") calibrated the same `R` to the world's own
distribution so that about 10 % of mouths are river-dominated: *"Most mouths must NOT become bird's-feet."*

## 4. What makes channels look natural on a map

4.1 **Low sinuosity.** Delta distributaries are low-sinuosity channels: one digitate-delta study calls a
sinuosity of 1.2 or more high (SECONDARY), and single-thread delta-plain channels are typically low-sinuosity with
long half-wavelengths of 2-10 km (SECONDARY). That distributaries are straighter than the meanders upstream of
them: UNCONFIRMED (no side-by-side source found).

4.2 **Meanders as a sine-generated curve.** Langbein & Leopold 1966 (*USGS Prof. Paper* 422-H) model a meander
as a curve whose heading varies sinusoidally with distance; its sinuosity is `1 / J0(ω)` for heading amplitude
`ω`. Meander wavelength is about 7-11 channel widths (Leopold & Wolman 1960, *GSA Bulletin* 71). Both from
memory: UNCONFIRMED here.

4.3 **What the eye reads.** From the above, a natural delta on a map has: a hierarchy (a trunk, smaller
distributaries, smaller still), unequal widths at every split, splits at irregular distances that shrink
downstream, a spread of angles around 70 degrees rather than one repeated angle, gently curving channels that
bend seaward, and **no two channels running side by side** — the comb the owner saw.

## 5. How map tools draw deltas

5.1 **OpenStreetMap.** `waterway=river` goes on ways (lines), drawn downstream; the water surface is an area
(`natural=water` + `water=river`); `width=*` is the average width. The wiki page names no delta or distributary
tag, and none was found. CONFIRMED (wiki page read). A delta in OSM is therefore just more river lines
(each distributary a `waterway=river` or `stream`) plus their water areas.

5.2 **Natural Earth.** `rivers_lake_centerlines` are single lines with `scalerank`, name and stroke-weight
attributes (derived mainly from World Data Bank 2), with a separate tapering set. SECONDARY. Whether
distributaries are included: UNCONFIRMED.

5.3 **Azgaar's Fantasy Map Generator.** Rivers carry a mouth cell and a mouth width, parent and basin ids;
tributaries merge by flux dominance; no delta or distributary generation was found (`src/modules/river-generator`,
read through summaries). SECONDARY; the absence is UNCONFIRMED.

5.4 **Procedural literature.** Génevaux et al. 2013, "Terrain generation using procedural models based on
hydrology", *ACM TOG* 32(4), 143: a drainage network grown from coastal outlets, terrain built round it; no
delta. SECONDARY/UNCONFIRMED. No delta-specific procedural-generation paper was found.

**So the map-tool convention is lines, not polygons**: a delta is drawn as a set of distributary lines over a
water/land boundary. That is what this engine's painted river already is (a distance field built from line
segments), so no new primitive is needed.

---

## 6. Cartalith's deltas, read at the symbols (origin/main `54e3aace`)

6.1 **Where they are made.** `cartalith-godot/src/river_delta.rs::delta_fans` (RIM-4, Ruling BU). Render-only:
a pure function of the drawn network (`river_stroke::RiverGeometry`, from `WorldGen::river_geometry_any`) and the
drawn-water classification (`WorldGen::drawn_water_classification`), cached in `WorldGen::river_delta_fans` on the
network's key. Its output is added to the river distance field (`river_field::build_with`), the colour field
(`river_stroke::rasterize_colour_field_with`), the deep-zoom tiles (`lod_worker::SnapshotInputs::river_fans`) and
the export (`export_raster`), all behind one gate, `river_delta::fans_drawn`. Nothing a simulation reads comes from
it; no world field, save key or golden involves it. There is **no delta logic in generation**: the source
project's v2.41 sediment-built deltas (`RC_ENGINE_CHANGES.md` §6g, default off there) are not ported.

6.2 **The old shape** (`delta_fans` and `march` before this change): per eligible mouth (own order ≥ 3,
discharge ≥ 2e-4 of the grid), an apex `(order − 1) × 8` cells upstream of the shore crossing, and from that ONE
point 2 (order 3) or 4 (order ≥ 4) branches at the fixed angles ±22 and ±45 degrees about the chord, each turning
from half its angle to its full angle over one setback and then marching **straight** until it met water; all of
equal width `w / sqrt(channels)` tapering by 0.4.

6.3 **Why it read as a comb.** Five causes, all in that function:
1. **One apex, fixed angles**: every fan was the same symmetric rake, ±22/±45, on every world; nothing varied.
2. **Straight branches**: after the bend the heading was constant, so each branch was a ruled line (sinuosity
   1.000), beside a network whose rivers wander.
3. **No hierarchy**: no branch ever split; four equal lines from one point.
4. **Equal widths**: every branch the same `1/sqrt(5)` of the river.
5. **Blind to the coast**: the angles were about the chord, not toward the water, so on a coast met obliquely the
   branches ran nearly along it.

The before frames (`cartalith-native/docs/evidence/delta-2026-11-11/before_*`, §8) show it: at x16 on the owner
world each delta is a rake of straight lines from one point.

6.4 **The comb that is NOT the fans.** At x1-x4 on the owner world much of what reads as a comb is the
**network's own** parallel coastal streams: order-1 and 2 runs crossing a smooth coastal slope side by side
(the comb Ruling BW's thinning, `cartalith_hydrology::river_draw_plan`, addresses). The `_nodelta` frames show
which is which. This redesign changes the fans only; the network's parallel streams are generated river geometry
(Ruling EN item 5: the network never moves here) and are left for their own row.

6.5 **Ruling EN and the fans.** RZ-2 gave each fan its apex river's rank (`DrawnRun::ranks`), so a fan is never
drawn without its river; RZ-5 (fans fading in by on-screen reach) was not built.

## 7. The redesign

7.1 **Principle**: keep the gate (which mouths), the setback, the reach limit, the off switch and every path's
plumbing; replace the shape. Every number is cited or a labelled judgement in `river_delta.rs`.

7.2 **The tree.**
- The river's own drawn course from the apex to the sea is the delta's **main channel** and is never redrawn
  (the network is not touched anywhere).
- **Offtakes**: `DeltaKind::offtakes` distributaries leave the main channel at points spread over 0.15-0.75 of its
  length from the apex, jittered, on alternating sides from a seeded first side. Each takes a share drawn in
  `MINOR_SHARE` (0.2-0.45, §1.4) of the remaining discharge and turns from the course by
  `BIFURCATION_ANGLE_DEG` (70, §1.3) ± `ANGLE_JITTER_DEG` (12) times the river's remaining share.
- **Mouth-bar splits**: after `BIFURCATION_LENGTH_WIDTHS` (10, §1.2) of its own widths (floor 3 cells, ±20 %),
  still on land and within its type's depth, a channel splits into two daughters sharing its discharge unequally
  (`MINOR_SHARE`), parting at 70 ± 12 degrees, each turning by the angle times the OTHER's share — the larger
  daughter keeps closer to the parent's heading. Because width goes as `sqrt(Q)`, segments shorten downstream.
- **Seaward steering**: each channel's mean heading turns toward the shortest way to the water (a chamfer distance
  transform round the mouth, `SeaField`) at most `SEAWARD_TURN_RAD_PER_CELL` (0.06 rad a cell). Water on a delta
  plain runs seaward; without it the daughters' turns add up until a channel heads inland.
- **Meanders**: each channel is a sine-generated curve (§4.2) of amplitude `DeltaKind::meander_rad` (0.45 or 0.35
  rad: sinuosity 1.03-1.06 by `1/J0`) and wavelength 11 widths (floor 4 cells), faded in over the first half-wave
  so it leaves its parent at exactly its heading.
- **No comb**: a channel that comes within one cell (3 × 3 neighbourhood) of any other channel, the main channel
  included, once it is 3 cells from its own start, is dropped (`CLEAR_CELLS`, `EXEMPT_CELLS`); so is one that has
  not met water within 1.6 setbacks of path. A dropped daughter's sibling takes the whole share.
- **Widths**: `w_ref × sqrt(q)`, `w_ref` the river's drawn width at the offtake and `q` the channel's share of
  the apex discharge: constant on a channel that splits (so `w_a² + w_b² = w_p²` exactly), tapering by
  `TERMINAL_TAPER` (0.25) on a terminal one. The main channel keeps its network width (it is the network's run;
  narrowing it would mean editing the network, which this change must not).
- **Determinism**: SplitMix64 seeded by the apex position's bits and the river's order (`Rng::for_mouth`): same
  world, same fan; a different mouth, a different tree; independent of network order.

7.3 **The plan type by exposure.** The engine has no wave or tide field. The fetch in front of a mouth is what
sets how much wave energy can reach it, so `exposure` takes the water share of the half-disc of radius one
setback in front of the shore crossing, and `DeltaKind::of_exposure` maps it: below 0.55 **river-dominated**
(2 offtakes, 3 levels deep: up to 8 terminal channels each), 0.55-0.8 **lobate** (2 offtakes, 2 levels), above
0.8 **wave-dominated** (1 offtake, unbranched: the cuspate one-channel delta of §2.3); order ≥ 5 adds an offtake.
A labelled proxy for Nienhuis' `R` (§3.2), with discharge entering only through the order. The tidal class is
not drawn.

7.4 **Zoom (Ruling EN, RZ-5).** Each channel takes the rank a network river of its own discharge has
(`RankOfDischarge`, read from the network's own (discharge, rank) pairs), never ahead of its apex river. So the
delta's big channels appear with its river and the small ones as the zoom grows, by the one visibility law the
field's B channel already carries. **No second texture**: RZ-5's "fade by on-screen reach" is done through the
existing rank channel, because a channel's reach on screen scales with its discharge.

7.5 **§7p and the defaults rule.** `DECISIONS.md` §7p: a deliberate graphics improvement replaces the old
approach, which is scrubbed, not kept beside it. The "off in `WorldParams::defaults`, on in `params::defaults`"
rule (`MISTAKES.md`, "Change generated output") applies to values the generator computes and a world hash covers;
the fans are neither (6.1). The code's precedent for a render-only look change is the lava-flow and acid-crater
entries under §7p: the new drawing replaces the old by control flow, and the reference look (`js_reference`)
stays byte-identical. That is what is done here: the old rake is removed, not kept as an option; `fans_drawn`
is unchanged, so `js_reference` and every look that keeps the stroke draw no fan, and `set_river_deltas(false)`
is the same byte-identical no-fan render it always was. No world field, save key, golden or generated value
moves.

## 8. Bars

Measured on the probe (`river_delta_stats`, which now also returns `river_delta::shape_of`) and pinned in
`river_delta.rs`'s tests by literals.

| Bar | Target | How measured |
|---|---|---|
| B1 no comb | `parallel_frac` (fan land length within 2 cells of another fan channel and within 15° of its heading) < 0.05 | `shape_of`, probe and `channels_meander_and_never_run_side_by_side` |
| B2 no contact | no two channels share a 3 × 3 neighbourhood away from their starts; none crosses the main channel | tests |
| B3 sinuosity | median sinuosity of channels ≥ 3 cells in 1.01-1.2 (old: 1.000) | `shape_of` |
| B4 angle | mean full split angle 60-80° (cited 70), with spread | `shape_of`, test |
| B5 unequal, conserved | at every split `w_a² + w_b² = w_p²` (±0.1 %), width sum 1.0-1.42 × parent, minor share 0.2-0.45 | test; `minor_share_mean` on the probe |
| B6 type follows the coast | sheltered → more channels and splits than open | test; `kind_*` counts on the probe |
| B7 in bounds | every terminal channel ends on water; no land point beyond 1.6 setbacks of the apex | tests |
| B8 zoom | a channel never ranks ahead of its river; smaller channels rank later | test; frames at x1/x4/x16/x64 |
| B9 network untouched | fans-off frames byte-identical to origin/main's | probe frames, both builds |
| B10 cost | derivation `ms` stays a few milliseconds on the owner world | `river_delta_stats.ms` |

## 9. Owner questions (defaults taken; the build proceeds on them)

1. **Exposure thresholds** (0.55 / 0.8): how many mouths should be river-dominated? Default: as measured (§10);
   the source project's v2.41 aimed at about 10 %.
2. **The network's own parallel coastal streams** (6.4) — a separate row? Default: yes, not touched here.
3. **Land fingers**: a real bird's-foot delta builds land out into the sea along its distributaries. Drawing that
   needs the land/water raster to change (generation, or the v2.41 deposition port). Default: not done.
4. **Tide-dominated deltas** need a tide field. Default: not drawn.

## 10. Build notes (2026-11-11)

**Built** in `cartalith-godot/src/river_delta.rs` as section 7 describes (`delta_fans`, `Grow`, `SeaField`,
`DeltaKind`, `RankOfDischarge`, `shape_of`); `WorldGen::river_delta_stats` now also reports `collided`, `splits`, the
three kind counts and the measured shape. One change after the first windowed run: seaward steering starts only
`EXEMPT_CELLS` past a channel's start, because steering from the first step read the owner world's split angle as
57.5 degrees.

**Verified.**
- `cargo test --workspace` (separate target dir): 267 result lines, 7 827 passed, 0 failed. Four other tests whose
  premise was the old four-branch rake were restated, not weakened in what they protect: `lod_bridge`
  `a_tile_strokes_the_delta_fans_of_a_river_mouth` and `lod_worker`'s `delta_inputs` premise (an open coast now
  grows one wave-dominated channel, so `>= 1` branch), `lod_worker` `a_snapshot_with_delta_fans_draws_them_and_keys_them`
  (positive control `w / 4` changed pixels, measured 118), `export_raster`
  `the_export_paints_the_delta_fans_on_land_only` (a fan pixel must lie on land or touch a land cell in its bilinear
  footprint: the new tip ends at a shore corner), and the `lod_bridge` opaque-pixel rule (an opaque pixel beside a
  fan may move by at most 2 levels: measured 1, 0.7 cells off the line).
- Literal mutants (Python, exact replace, restored in `finally`, hash checked): 21 of 21 killed after adding
  `the_shape_measure_sees_a_comb` and the shape constants to the literal pin (`PARALLEL_DEG` 0 survived the first
  sweep).
- **The network did not move**: every fans-off frame (`_riverzoom_probe.gd --rz-frames --rz-delta-pair`, new) at x1,
  x4, x16 and x64 on the owner world, 483920 (1024x656) and 912345 (1536x983) is pixel-identical between origin/main
  and this build (12 of 12). No world field, save key or golden is involved (section 6.1).
- **Frames**: `cartalith-native/docs/evidence/delta-2026-11-11/` -- `before_owner_zNN` / `after_owner_zNN` (the
  whole view, x1-x64, final build), and per mouth `*_mouthNNNN_*` (owner 2584 lake and 3246 ocean, 483920's 331 lake,
  912345's 291 ocean). The per-mouth frames and the two smaller seeds' numbers were taken on the build before the
  steering change above (a one-condition change; the owner's whole-view frames and the owner numbers below are the
  final build's). Looked at: the before frames show each delta as a rake of straight lines from one point; the after
  frames show branching, gently meandering distributaries with unequal widths.

**Measured** (`river_delta_stats`; before = origin/main):

| World | Mouths eligible | Fans before -> after | Channels | Kinds river/lobate/wave | Splits | Sinuosity p10/p50/p90 | `parallel_frac` | Split angle mean ± sd | Minor share | ms before -> after |
|---|---|---|---|---|---|---|---|---|---|---|
| owner 2048x1311 (246371), pre-steering-change build | 96 | 94 -> 71 | 223 -> 152 | 5 / 26 / 40 | 18 | 1.021 / 1.042 / 1.081 | 0.028 | 57.5 ± 9.8 | 0.33 | 0.7 -> 48.9 |
| 483920 1024x656 | 15 | 15 -> 13 | 23 -> 38 | 4 / 4 / 5 | 5 | 1.012 / 1.040 / 1.065 | 0.000 | 61.1 ± 7.5 | 0.29 | 0.14 -> 3.8 |
| 912345 1536x983 | 10 | 9 -> 5 | 18 -> 14 | 1 / 3 / 1 | 2 | 1.016 / 1.040 / 1.062 | 0.000 | 50.0 ± 2.5 | 0.38 | 0.13 -> 4.4 |

Final build, owner world (`after2_owner.log`): 96 eligible, **77 fans**, 172 channels, kinds 6 / 26 / 45, 17 splits,
sinuosity 1.020 / 1.042 / 1.090, `parallel_frac` 0.024, **split angle 70.0 ± 10.0 degrees** (cited 70.4; experiments
68.3 ± 8.7), minor share 0.31, 33.5 ms.

**Against the bars (section 8).** B1 no comb PASS (0.000-0.028 < 0.05). B2, B5, B6, B7, B8 PASS in the tests. B3
sinuosity PASS (medians 1.04). B9 PASS (12 of 12 identical). B4 split angle: PASS on the tests' fixtures (60-80), and
70.0 on the owner world with the final build (57.5 before the steering change; 61.1 and 50.0 on the two smaller seeds, pre-change build). B10 cost: **not met as written** -- the
derivation is 4 to 34 ms (owner world 33.5 ms, final build) where it was under 1 ms (the per-mouth distance transform and the occupancy map); it runs
once per network and is cached (`WorldGen::river_delta_fans`), so it is a one-off cost on a repaint, not per frame.

**Open, measured.** (1) **Fewer mouths get a fan**: 77 of 96 on the owner world (final build; 71 before the steering change), 5 of 10 on 912345, because a
channel that would touch another (most often the main channel itself, which meanders) is dropped
(`collided` 57 on the owner world, 14 and 11 on the smaller seeds). The rake drew at every eligible mouth because its straight branches never came back. A
next step that keeps the no-comb rule: let a channel that meets the main channel END there (an anabranch loop:
Tejedor et al. 2015 report recombination on real deltas), instead of dropping it. (Built in section 12.) (2) The kinds split 7 % / 37 % /
56 % river / lobate / wave on the owner world, close to the source project's v2.41 target of about 10 % river-
dominated (section 3.2). (3) The whole-view frames at x1 and x4 still show straight parallel strokes at the coast:
those are the network's own streams (6.4), identical with the fans off, and are the subject of the separate
straight-river investigation (section 11).

**Not changed, by design**: the gate (which mouths), the setback, the reach limit, the off switch, `fans_drawn`, every
consumer path; the reference look draws no fan.

## 11. Straight river lines beyond the deltas (owner, 2026-11-11): measured by cause

The owner, relayed: *"Investigate why sometimes we still get straight lines for rivers and correct it so we don't get
these straight lines."* This section is the measurement. The fix needed an owner ruling (11.4); Ruling ER answered it, and section 13 has
what was built.

11.1 **The metric** (`cartalith-godot/src/river_straight.rs::straight_stats`, tested): over each polyline, the
maximal stretches whose every point lies within `tol` cells of the stretch's own chord and that are at least
`min_len` cells long; their share of the length, and the part whose chord lies within 3 degrees of a multiple of 45
(a grid axis or diagonal: the D8 signature). `WorldGen::river_straightness` applies it to each stage of the drawn
river, an ablation by stage: `traced` (the D8 cell path), `traced_const` (runs of ONE repeated D8 step), `spline`
(Catmull-Rom through every traced cell, no simplification), `rdp` (the shipped curve, simplification then spline),
`drawn` (the drawn pieces) and `drawn_open` (only what Ruling EN shows at the opening view). Probe:
`_riverzoom_probe.gd --stats-only --straight` (windowed), `min_len` 8 cells, `tol` 0.25 cells, on origin/main before
the delta redesign.

11.2 **Measured** (share of length in straight stretches; axis-aligned share of that in brackets; longest stretch):

| World | traced (D8) | traced, one repeated step | spline, no RDP | shipped curve | drawn | drawn at opening view |
|---|---|---|---|---|---|---|
| owner 2048x1311 (246371) | 58.1 % (100 %), 162 cells | 58.1 % | 58.5 % | 54.2 % | 44.5 % (82 %), 114 cells | 50.1 % (85 %) |
| 483920, 1024x656 | 32.1 % (100 %), 61 | 32.1 % | 32.6 % | 30.4 % | 30.3 % (77 %), 50 | 31.3 % (77 %) |
| 912345, 1024x656 | 40.9 % (100 %), 219 | 40.9 % | 41.2 % | 38.8 % | 35.3 % (75 %), 100 | 37.7 % (74 %) |
| 24601, 1024x656 | 36.5 % (100 %), 93 | 36.5 % | 37.0 % | 33.8 % | 33.6 % (70 %), 64 | 33.3 % (70 %) |

(The old fans were 92 % straight on the owner world, which the redesign in section 7 removes.)

11.3 **Cause.** The straightness is in the **generated network**: a third to three fifths of every network's length
is runs of one repeated D8 step, every one of them on a grid axis or diagonal, up to 162 cells (3 200 km on the owner
world) in a line. D8 routing sends a whole smooth slope's water in one of eight directions, so a river crossing a
planar slope is a ruler line. The drawing neither makes nor removes it: a spline through collinear cells is the same
line (`spline` equals `traced`), and the simplification (`rdp`) changes the share by 1 to 4 points. Of the
candidates the coordinator named: D8 straight runs are the cause; lakes are already cut out of the drawn pieces;
the stroke applies its smoothing everywhere (it has nothing to smooth on a straight run); the coastal parallel runs
are the same D8 runs on the coastal slope; the vector fallback strokes the same points.

11.4 **What a fix needs, and why none is built here.**
- **At the root (generation)**: routing that does not run a planar slope in one of eight directions -- for example a
  seeded tie-break or a small deterministic noise term on the routing surface, or a multiple-direction method for the
  trace. That moves the generated river network, which Ruling EN item 5 says never moves, and the goldens of the
  reference path (`DECISIONS.md` §7p: a change to what the generator computes needs its own ruling and a disclosed
  re-baseline). **An owner ruling is needed first.**
- **Drawing only**: a deterministic, bounded display meander on the drawn curve, its amplitude limited by the cross
  slope (so it never climbs out of a valley or over a ridge) and faded to zero at junctions and shore ends. Bounded to
  stay inside the cells the river was traced through, it is under 0.5 cells: invisible at x1 and x4 (0.46 and 1.8
  pixels a cell on the owner world), visible from about x16. Wide enough to break a 100-cell ruler line at x1 it would
  leave the traced cells by several cells. So drawing alone fixes x16 and x64 only.
- **Recommendation for the owner**: rule on the generation fix (it is the only one that changes x1 and x4, where the
  owner saw the lines), and take the bounded display meander for the deep zooms either way.


## 12. "The deltas seem to have gone" (owner, 2026-11-11): diagnosed and fixed

The owner, on the x16 frames of section 10 (relayed): *"the deltas seem to have gone"*. In
`evidence/delta-2026-11-11/after_owner_mouth3246_ocean_z16.png` no distributary is drawn; the other mouths show faint
forks. Three causes were candidates: the no-contact rule dropping channels that touch the meandering main channel;
channels getting late visibility ranks, so the zoom law hides them at x16; widths too thin to read.

12.1 **Measured** (`river_delta::fan_seen`, new: per zoom over the opening fit view, the fans whose river is drawn
and the channels drawn, i.e. with some point at or above the picking line `PICK_MIN_VISIBILITY` 0.5, by the one
visibility law; keys `seen_zNN_*` of `WorldGen::river_delta_stats`; `_riverzoom_probe.gd --stats-only --rim4`,
windowed). Owner world (246371, 2048 x 1311, density 1.55), origin/main `0f722870` plus the measurement:

| Zoom | Fans whose river is drawn | Channels drawn / built | Median channel rank | Median apex rank |
|---|---|---|---|---|
| x1 | 11 of 77 | 0 / 172 | 1.0 | 0.21 |
| x4 | 24 | 0 / 172 | 1.0 | 0.21 |
| x16 | 58 | **0 / 172** | 1.0 | 0.21 |
| x64 | 77 | 172 / 172 (median 13.4 px wide) | 1.0 | 0.21 |

**Cause: the ranks.** Every channel had been built (172) and every one was hidden until x64, where the law draws the
whole network. The median channel ranked 1.0, the very last of the network, against 0.21 for its river.
`RankOfDischarge` read the network's own `(discharge, rank)` pairs and took a running maximum of the rank down the
discharges, so any run with a late rank and a large discharge pushed every smaller discharge to that rank. Ruling EQ
item 3 (`hide_short_stubs`) raises a short run's ranks toward 1 whatever its discharge, which supplies exactly such
runs. The no-contact rule did drop channels (57), but it is not why the built ones vanished. Width is not the cause
either: nothing was drawn to be thin.

12.2 **Fixed** (`river_delta.rs`):
- **The rank table reads discharge only.** `RankOfDischarge` is now the share of the drawn network's length whose
  discharge is at least `q`, which is what `visibility_ranks` computes. A rank raised for another reason cannot
  leak in. Tested at literals: 0, 0.4643, 0.7321, 1 on a three-river fixture, unchanged when one river's rank is
  raised to 1.
- **Whole by x16** (`DELTA_FULL_ZOOM` 16, `delta_rank`): a channel's rank is its own discharge's, never ahead of its
  river's, and never later than the share the law reaches at x16 (`start^(1/3)`, 0.353 for a whole world), unless
  the river itself is later. Between the opening view and x16 the smaller channels still fade in by discharge
  (Ruling EN's RZ-5). From x16 a fan is whole wherever its river is drawn. This is the coordinator's "floor a delta
  channel's visibility to its parent river's", applied from x16 so the opening view keeps showing only big rivers
  (the second addendum to Ruling EN). It does not depend on the window: the share at a zoom over the fit depends
  on the start share only. `delta_fans` takes the start share, and the fan cache key carries it.
- **Loops** (`LOOP_MIN_CELLS` 6): a distributary that meets the river's own course, after running at least six
  cells and at least six cells from where it left, ends on the course. It forms an anabranch round an island
  (Tejedor et al. 2015) instead of being dropped. A channel that meets another distributary is still dropped, so the
  no-comb rule stands. A loop keeps its width to the river and does not reduce the share of the later offtakes.
  `DeltaStats::rejoined` counts them.
- **No width floor was added.** The measured widths at x16 are 2.2 px (10th percentile) and 3.3 px (median) on the
  owner world, and 5.0 and 7.2 px on 483920, above the existing 1 px stroke floor. A wider floor would break
  `w_a^2 + w_b^2 = w_p^2` (bar B5) for no visible gain.

12.3 **After** (same probe, same worlds, D8 routing as before):

| World | Zoom | Fans whose river is drawn | Channels drawn / built | Width p10 / p50, px |
|---|---|---|---|---|
| owner | x1 | 11 of 78 | 25 / 180 | 1.0 / 1.0 |
| owner | x4 | 24 | 51 / 180 | 1.0 / 1.0 |
| owner | x16 | 59 | **135 / 180**, all 59 drawn rivers with channels | 2.2 / 3.3 |
| owner | x64 | 78 | 180 / 180 | 8.5 / 13.3 |
| 483920 (1024 x 656) | x1 | 10 of 13 | 17 / 32 | 1.0 / 1.0 |
| 483920 | x4 | 12 | 28 / 32 | 1.3 / 1.9 |
| 483920 | x16 | 13 | **32 / 32** | 5.0 / 7.2 |
| 483920 | x64 | 13 | 32 / 32 | 20.0 / 28.8 |

At x16 the 45 channels not drawn on the owner world belong to the 19 fans whose river is itself not drawn yet.
Loops: 6 on the owner world (collisions 57 -> 50), 0 on 483920. Fans 77 -> 78, channels 172 -> 180. The shape bars
still pass on the owner world: sinuosity p50 1.042, `parallel_frac` 0.026, split angle 69.7 degrees. Derivation:
38 ms (33.5 before).
Frames: `cartalith-native/docs/evidence/rivers-straight-2026-11-11/delta_*`, and the composite there. The owner's
"gone" mouth 3246 at zoom 1600 % now draws its forks, and lake mouth 2584 its branching fan.

## 13. Ruling ER: the routing changed for new worlds, and a display meander (2026-11-11)

The owner, on the options in 11.4: *"3, and would it change if we had 32 instead of 8 directions?"* (Ruling ER).
So both are built, and the question is answered by measurement.

13.1 **The candidates, on the same ground** (`cartalith-godot/tests/river_routing_measurement.rs`, `#[ignore]`d, run
in release). Each world is generated with the app's parameters and `carve_rivers` off, so the field is the eroded
terrain before any river is cut into it. A carved field would hold the shipped D8 channels as trenches and pull every
candidate back onto them. Each candidate routes over that field's depression-filled surface, accumulates the world's
rain, channelises at the app's threshold, and is traced. "Traced" is the traced path densified to one point a cell;
"drawn curve" is the app's `river_render_polyline` steps (RDP at 0.5 cells, then Catmull-Rom at 0.25) without the
pins. Straight means stretches of at least 8 cells within 0.25 cells of their chord, as in section 11. Plausibility:
uphill steps between cells the fill did not raise, steps not strictly down the routing surface, and "dry ends" (a
traced river ending on land, away from water, the map edge or another river). All three were 0 for every candidate
except where noted.

| Candidate | owner: traced / drawn curve | 483920 | 912345 | Length vs D8 (owner / 483920 / 912345) | Dry ends | Route ms, owner |
|---|---|---|---|---|---|---|
| **D8** (shipped until now) | 67.8 % / 63.1 % | 43.7 / 40.7 | 48.6 / 45.9 | 1 | 0 | 67 |
| D8 + near-tie break (5 %) | 64.4 / 60.7 | 40.5 / 38.2 | 46.4 / 44.0 | 0.99 / 0.99 / 0.99 | 0 | 289 |
| D16 (knight moves) | 55.1 / 59.8 | 25.1 / 36.3 | 36.2 / 44.8 | 1.05 / 1.04 / 1.05 | **557 / 388 / 305** | 204 |
| **D32** | 55.1 / **62.0** | 30.4 / **41.7** | 37.8 / **47.5** | 1.10 / 1.08 / 1.08 | **1039 / 640 / 587** | 304 |
| D-infinity, least transverse deviation | 38.9 / 45.2 | 14.2 / 21.2 | 29.9 / 33.1 | 1.01 / 0.99 / 0.99 | 0 | 328 |
| rough k 1, L 8 | 66.4 / 61.7 | 42.3 / 39.3 | 47.9 / 45.0 | 0.97 / 0.99 / 0.99 | 0 | 192 |
| rough k 4, L 8 | 45.6 / 37.3 | 29.6 / 25.8 | 31.5 / 27.2 | 0.69 / 0.89 / 0.83 | 0 | 197 |
| rough k 8, L 8 | 35.1 / 28.2 | 20.7 / 17.5 | 21.9 / 18.6 | 0.60 / 0.83 / 0.76 | 0 | 183 |
| rough k 2, L 4 | 32.7 / 26.9 | 18.7 / 16.8 | 20.8 / 18.1 | 0.70 / 0.82 / 0.77 | 0 | 184 |
| rough k 3, L 4 | 23.9 / 19.8 | 13.0 / 12.1 | 15.5 / 13.7 | 0.65 / 0.76 / 0.71 | 0 | 183 |
| **rough k 4, L 4 (built)** | **20.0 / 16.8** | **10.5 / 10.0** | **13.2 / 11.7** | 0.64 / 0.73 / 0.69 | 0 | 185 |
| rough k 4, L 6 | 33.4 / 26.3 | 20.5 / 17.9 | 21.3 / 17.9 | 0.63 / 0.82 / 0.75 | 0 | 184 |
| rough k 4, L 16 | 63.9 / 58.2 | 41.7 / 38.5 | 46.2 / 42.7 | 0.93 / 0.99 / 0.97 | 0 | 184 |

The ms is one routing and accumulation, including building the roughness noise. Each was run once, so read it as an
order of magnitude, not a benchmark.

13.2 **The owner's question: would 32 directions change it?** On the drawn curve, no. D32 leaves the drawn rivers as
straight as D8: 62.0 / 41.7 / 47.5 % against 63.1 / 40.7 / 45.9 %. It shrinks the angle steps from 45 degrees to about
11, so the straight share on the traced path falls (55 / 30 / 38 %) and the runs are on a grid axis less often (72 /
33 / 77 % against 100 %). But the runs are still straight, just at finer angles, and the spline draws them as the same
ruler lines. Steepest descent over a planar slope picks one direction whatever the number of directions offered, so
the expectation in the brief holds. D16 and D32 also break the drainage the way they were built here: neighbours
two or three cells away let a channel step over a cell, 300 to 1 000 traced rivers end on dry land, and every consumer
that walks the river cell by cell (carving, settlements, the planner, lakes) would need rework. A D-infinity direction
(Tarboton 1997's aspect, made single-receiver by Orlandini et al. 2003's least-transverse-deviation rule) follows the
true aspect rather than the grid. It helps (45 / 21 / 33 %), but it still follows a planar slope straight. **Only a
perturbation of the choice makes a river wander.** It stands for the sub-grid relief the height field does not
resolve.

13.3 **Built: rough routing** (`cartalith_hydrology::rough`, `WorldParams::rough_routing`).
- **The rule.** Among the strictly lower neighbours on the routing surface, the cell picks the largest
  `(h_i - h_j + k * s_i * (n_i - n_j)) / d`. Here `n` is seeded value noise (amplitude 1, lattice
  `ROUGH_WAVELENGTH_CELLS` = 4), `s_i` is the cell's own steepest drop per cell, and `k` is `ROUGH_STRENGTH` = 4. The
  constants were chosen from the table.
- **What it guarantees.** Only strictly lower cells are candidates, so no step goes up the routing surface. Every cell
  that had a receiver still has one, and the descending-height accumulation order stays valid. (This said, until
  2026-11-13, that on the fill's flats "the slope is a few ULPs, so the perturbation vanishes there and water leaves a
  lake by the plain D8 path". **That was wrong**: `s_i` is a few ULPs there, but so is every drop it is compared with,
  so the noise decides the pick on a flat as on a slope. Measured on the owner's combing world, it is what breaks the
  comb there. What it cannot do is choose among one candidate; section 14 has the measurement and the fix.)
- **Where it applies.** Every flow routing of the generation uses it (the pre-carve, coastal, network and final
  passes), and so does `refresh_climate`. The GPU flow kernel, which is plain D8, is skipped when it is on. The
  channel network follows the rough accumulation tree (`build_channels_routed_with`) instead of real-aspect steering:
  steering toward a planar slope's aspect is what draws the ruler line back.
- **§7p.** It is off in `WorldParams::defaults`, so every golden and the reference path are unchanged by control flow.
  It is on in `params::defaults` and acts only with `integrate_drainage`.
- **Saves.** A save without the key loads it off. A world records its roughness seed (`WorldState::routing_roughness`;
  the substrate manifest's optional `routing_roughness`, omitted for D8, so old archives and D8 manifests are
  byte-identical). Every consumer that rebuilds the receiver tree uses it: `fresh_river_network_with` and
  `fresh_river_channels_with` (settlements, Ruling N's river term, the planner's river transport, the route tool,
  landmarks, the export), the pick bridges, and the draw plan's `down` tree.
- **Saved worlds keep their rivers.** A save stores the channel receivers, the channel mask, the order and the
  discharge as rasters (`substrate.rs`, `SAVEFILE_COMPAT.md` §8.3), so nothing is re-routed on load.
- **Not changed.** A heightmap import routes plain D8 (its relief is data). The rock-paint pass's local coastal
  discharge is plain D8 too: it is a sub-window estuary cut, and the noise is defined on the world's grid.

13.4 **In the app, full pipeline** (`_riverzoom_probe.gd --stats-only --straight`, windowed). Before is the same
build with `--rough-off`. Neither has the display meander of 13.5.

| World | traced D8 run share | shipped curve | **drawn** | drawn at opening view | longest drawn stretch | traced length | drawn length | lakes |
|---|---|---|---|---|---|---|---|---|
| owner, before | 58.1 % | 54.2 % | **44.5 %** | 54.3 % | 114 cells | 302 783 | 161 281 | 79 |
| owner, after | 17.9 % | 15.7 % | **16.8 %** | 25.9 % | 92 | 220 536 | 183 816 | 74 |
| 483920, before | 32.1 % | 30.4 % | **30.3 %** | 32.7 % | 50 | 24 737 | 21 360 | 25 |
| 483920, after | 17.5 % | 17.1 % | **17.3 %** | 18.8 % | 28 | 27 056 | 24 183 | 28 |
| 912345, before | 40.9 % | 38.8 % | **35.3 %** | 37.4 % | 100 | 20 835 | 17 428 | 31 |
| 912345, after | 17.9 % | 15.6 % | **15.0 %** | 17.4 % | 38 | 23 036 | 20 539 | 31 |

The owner world's "before" reproduces section 11 exactly (58.1 % and 44.5 %).
- **Network length.** The traced network on the owner world is 27 % shorter, because parallel one-cell strips on
  smooth slopes now merge into fewer, larger channels. The drawn network is 14 % longer, because fewer of its rivers
  are thinned as parallel to a heavier one. On the smaller worlds both grow 9-13 %.
- **Lakes.** Lakes are classified from the field, and the field moves only through the carve: 79 -> 74, 25 -> 28 and
  31 -> 31. The largest basins are the same. Channel-shaped lakes on the owner world fall from 11 to 6.
- **Downstream consumers.** Settlements, the planner, ways, landmarks and deltas read the rebuilt network through the
  `_with` functions above, and `cargo test --workspace` passes (13.6). The knowledge fixtures were regenerated,
  because their worlds' rivers moved. One knowledge check asked for at least one settlement on a river per seed;
  rough routing moved seed 24601 from 1 of 40 to 0, so it now asks across the three seeds. Siting is unchanged in
  kind: the river term is still one term in the suitability ranking (Ruling N).

13.5 **Built: the display meander** (`river_meander::display_meander`).
- **What it does.** The drawn curve of every run is displaced sideways by a sine of `MEANDER_AMP_CELLS` (0.3 cells)
  and `MEANDER_WAVELENGTH_CELLS` (4 cells) along its arc length. The phase comes from the run's first cell. The
  displacement fades to zero over half a wavelength round both ends and every pinned junction cell, and is clamped
  point by point so no point leaves the run's traced cells.
- **Why it only shows from x16.** The amplitude is fixed on the ground, so on screen it grows with the zoom: 0.14 px
  at the owner world's opening view, 0.55 at x4, 2.2 at x16 and 8.8 at x64.
- **One law.** The meander is part of the one drawn geometry (`RiverGeometry`) that the screen's field, the deep-zoom
  tiles, the export and picking all read, so they agree, and the visibility law is untouched. No shader changed.
- **Render only.** No world field, save key or golden. A probe switch (`WorldGen::set_river_meander`, in the
  network's cache key; `_riverzoom_probe.gd --meander-off`) draws without it.

13.6 **Verified.**
- `cargo test --workspace` (separate target dir): 268 result lines, 7 850 passed, 0 failed, 147 ignored.
- Literal mutants (Python, exact replace, restored in `finally`, hash checked): 8 of 8 killed. They covered
  `ROUGH_STRENGTH`, `ROUGH_WAVELENGTH_CELLS`, the strictly-lower test `hj < h`, `MEANDER_AMP_CELLS`,
  `MEANDER_WAVELENGTH_CELLS`, `SHRINK_STEPS`, `LOOP_MIN_CELLS` and `DELTA_FULL_ZOOM`.
- Tests updated, not weakened:
  - `params_mapping` lists the twenty-first divergence, and an old save loads `rough_routing` off.
  - `param_catalogue` counts 121 rows; `rough_routing` is excluded from the model's catalogue, like the geology
    switches with no GUI control.
  - `gf0_geology_harness`'s replica routes with the same roughness as `generate_terrain`.
  - The knowledge fixtures were regenerated, and the on-river guard now applies across the three seeds.
- `_riverzoom_probe.gd` and `shell/app.gd` pass `--check-only`. Every probe above ran windowed.

13.7 **The metric and the meander.** With the meander on, the probe's `drawn` stage reads 0.7 % straight on the owner
world and 0.5 % on 483920. That is an artefact of the metric's 0.25-cell tolerance against a 0.3-cell wander, which
is invisible below x16. The routing's own effect is the 13.4 table, measured without the meander. The meander's
effect is the x16 and x64 frames.

13.8 **Frames** (`cartalith-native/docs/evidence/rivers-straight-2026-11-11/`). `before_<world>_zNN` is rough routing
and meander off; `after_<world>_zNN` is both on (with section 12's delta fix in both). They are taken at x1, x4, x16
and x64 on the owner world and 483920, centred on the same cell (`targets.json`: owner lake mouth 2584 and a 483920
lake mouth). `delta_after_*` are section 12's per-mouth frames; their "before" is the owner's frames in
`evidence/delta-2026-11-11/`. **`composite_rivers_and_deltas.png`** puts them side by side for the owner. Logs:
`d1_owner.log` (the delta diagnosis), `d2_*.log` (after), `r_*_off/on.log` (13.4) and `routing_candidates.txt`
(13.1). Looked at: at x16 and x64 the after frames show winding channels where the before frames show parallel ruler
lines down the lake shore, and the 2584 delta draws its branching fan.


## 14. "Still see some heavy combing" (owner, 2026-11-13): two causes, both fixed (Ruling EV)

The owner, with two phone screenshots of seed 51203 (the river flow view and the relief view at zoom 788 %; APK from
`da0dbbbf`): *"Still see some heavy combing:"*, then *"resolve it"* (Ruling EV). The screenshots show dozens of
dead-straight north-south channels hanging from a 45-degree line across a broad basin, and in the relief view the
same comb carved into the ground.

14.1 **Cause one: the world was made by the old rules.** The phone's World info printed `rough_routing`,
`lake_balance` and `civ.merge_ways` as `false`, in a build whose `params::defaults()` has all three `true`. Opening a
save writes its generation rules into the engine's live table (`params::apply_saved_state`), and a save written
before a rule existed loads that rule off, correctly, so the opened world regenerates as it was saved. But New
World's Create generated from that same live table and never went back to the app's rules, so every world made
after opening an older save in the session was made by the old rules (reproduced windowed on `d3942d42`: open a
save without the three keys, then New World, Earth, Create: all three off). The New World screen's Lake water
balance box read ON meanwhile: it was set once when the dialog was built.
- **Fixed** (`params::LOADS_WHEN_ABSENT`, `params::restore_new_world_rules`, `WorldGen::restore_new_world_rules`,
  `EngineBridge._restore_new_world_rules`). One list holds what a save without each key loads; `apply_saved_state`
  writes it, and before New World's Create and the heightmap import the bridge puts every rule in it (its boolean
  rows) back to `params::defaults()`, except a rule the user set this session (`WorldGen::rules_chosen`, recorded by
  `set_params`, cleared by Edit > Reset parameters). Settings (the numeric rows and every other parameter) are not
  touched: a New World still carries the world on screen's settings. A saved world still loads and regenerates with
  its own rules. The dialog's Lake water balance box now shows the value Create will use
  (`WorldGen.get_new_world_params`), re-read on every open.
- **Verifier follow-ups (2026-11-13), each confirmed at its symbol first.**
  - *Another route made the same failure.* WORLD ▸ Generate world after New seed, the tool-options Generate and the
    assistant's regenerate all reach `EngineBridge.generate` with `new_world` false, so after opening an older save they
    built the NEW seed's world by the save's rules. The rules now follow the seed, not the route: any generate at a
    seed other than the open world's (`WorldGen.get_seed`) runs `_restore_new_world_rules` too
    (`EngineBridge._makes_another_world`); a generate at the open world's own seed still regenerates that world with
    its rules. (The New World dialog's seed is not synced to an opened world's, so after an open most regenerates are
    at another seed, and are now new worlds by this build's rules.)
  - *The preview drew another world.* It built its request from the live table while Create restored the rules, and
    the preview's substrate reads `tect.narrow_plate_base_blur` (`plate_base_blur_r`), so after opening a save without
    that key it drew the pre-v2.57 coastline. It now reads `EngineBridge.new_world_params_snapshot`
    (`get_new_world_params`).
  - *The override was silent.* A rule a save carried off, a choice made in an earlier session included, came back on
    with only a console line. The New World screen now names every rule Create will change
    (`_refresh_rules_notice`, on every form, hidden when there is none), and the status hint after any run that put
    rules back names them (`app.gd`'s finish handler). The restore itself is unchanged: a save's explicit `false`
    cannot be told from a rule that was off by default when the save was written (`restore_new_world_rules`' doc), and
    the owner's own world was saved with the three rules off explicitly, so restoring only absent keys would not have
    fixed it.
  - *A failed run kept the new rules.* The restore ran before a run that can fail, and a heightmap the engine cannot
    read leaves the open world on screen (`WorldGen::import_heightmap`), whose lakes are classified from the live
    `lake_balance`. A failed run now puts the recorded values back (`EngineBridge._undo_new_world_restore`,
    `WorldGen::revert_new_world_rules`, which never counts them as the user's choice).
  - Probed windowed (`_newworld_rules_probe.tscn`, legs L2, L2b, L2c, L3's notice, L4's hint and L11, on the desktop
    shell, the phone form and the tablet form, 180 checks each, 0 failed); the same probe fails 42 of its checks on
    `fcda4d3d`'s shell, so its new legs pin the fixes (`STATUS.md`).

14.2 **Cause two: the fill tilts a basin as a chessboard.** Even with rough routing on, a long 45-degree ruler line
and straight north-south runs remained in the owner's window (the "before" column of the frames). The window holds
the map's largest filled depression (lane B: 20 114 cells, about 118 m deep at the 4 000 m peak), mostly drawn as
land. Every routing surface is built by `cartalith_hydrology::build_routing_surface`, Barnes, Lehman & Mulla (2014)'s
Priority-Flood+epsilon with a first-in-first-out pit queue: a filled cell sits one `f32` ULP above the cell that
reached it, so its height is the spill plus its chessboard distance from the spill. Water then crosses the basin on
shortest grid paths, parallel runs on an axis that join 45-degree diagonals, and a cell ON such a diagonal has one
strictly lower neighbour, so no receiver rule that only steps down (D8 or rough) can bend it. Measured with rough
routing on (14.5, `before`): the straight share on filled ground is 26 to 34 % on the four worlds against 8 to 18 %
elsewhere, with stretches up to 88 cells on filled ground. (Section 13.3's claim that rough routing's perturbation
"vanishes" on the flats was wrong and is corrected there: it is the single candidate, not the noise, that holds the
line.)

14.3 **Built: basin routing** (`cartalith_hydrology::FillTilt::Floor`, `WorldParams::basin_routing`).
- **The rule.** The pit queue is a priority queue on the cell's own terrain height (then its raised height, then its
  index), so from the spill the fill runs down the basin's lowest ground to its floor and spreads up from there, the
  way a basin fills; and a cell is raised `FLOOR_TILT_ORTH_ULPS` (5) ULPs above the cell that reached it
  orthogonally and `FLOOR_TILT_DIAG_ULPS` (7) diagonally, Borgefors (1986)'s 5-7 chamfer, so the tilt grows like a
  Euclidean distance (within about 2 %) rather than a chessboard one. Water inside a basin runs down its real slopes
  to its floor and leaves by one path: least-cost routing through the depression (Metz, Mitasova & Harmon 2011),
  carried as an epsilon tilt that every steepest-descent or rough receiver follows. A filled cell has several
  strictly lower neighbours almost everywhere, so rough routing can turn the flow there too.
- **Why both halves.** Screened on the owner's window with rough routing on (frames looked at, from a scratch
  harness that was not kept): the chamfer steps alone turn the 45-degree lines into parallel diagonal runs; ordering
  by terrain alone gives a converging network across sloping basin floors, but on an exactly flat floor it falls back
  to the raised height, a chessboard again; together they give a converging network with no forced line. The hydrology tests pin each half
  (`the_floor_tilt_sends_a_basins_water_to_its_floor_before_the_spill`,
  `the_floor_tilt_leaves_no_filled_cell_a_single_way_down_on_a_flat`).
- **What it guarantees.** The seeds, the open queue, RichDEM's equal-height tie rule and the raise test are the
  breadth-first fill's; a field with nothing enclosed comes back bit-identical; every filled cell has a strictly lower
  neighbour and drains to a seed (`the_floor_tilt_drains_every_cell_to_a_seed_and_never_lowers`). The tilt rises at
  most 0.94 m above the spill at a 4 000 m peak on the four worlds (14.5).
- **What it does not guarantee (corrected 2026-11-13).** This said the two tilts "fill the same depressions to the
  same spill". A verifier disproved it, and it is false: the raise test compares a neighbour's terrain with the RAISED
  height of the cell that reached it, so a cell standing less than the tilt's rise above a spill is filled with the
  depression -- the creep every Priority-Flood+ε has -- and the floor tilt's rise is larger (5 to 7 ULPs a step along
  longer, floor-first paths, against one ULP a ring). So it fills a few more rim cells, and where a saddle stands
  within its rise it tops the saddle and the basin behind it drains through this basin's spill
  (`the_floor_tilt_can_capture_a_basin_behind_a_saddle_within_its_rise`, the verifier's 123 x 5 corridor: Breadth
  rises 3.5e-6 and leaves a saddle 1e-5 above the spill alone; Floor rises 1.76e-5 and captures the basin behind it).
  Measured on the final fields (`floor_tilt_against_breadth_on_the_final_field`): Floor raises 1 946 / 1 991 / 577 /
  642 cells that Breadth leaves at their terrain on 51203 / 246371 / 483920 / 912345 (GPU off), and Breadth none that
  Floor leaves; 0.081 / 0.089 / 0.056 / 0.090 % of land cells drain to another seed, almost all of them outside every
  Breadth-raised cell (rim cells, not captured basins: 2 / 0 / 0 / 5 inside). On the owner's world as the app makes it
  (GPU on): 1 330 cells and 0.047 %. A fix would have to keep the tilt below the terrain around a basin, which an
  `f32` routing surface cannot do at 5 ULPs a step; it is routed, not built (`OUTSTANDING_WORK.md` §2.13).
- **Where it applies.** Every routing surface of generation (the pre-carve, coastal, network and final passes, the
  sediment routing) and `refresh_climate`. The world records it (`WorldState::basin_routing`; the substrate
  manifest's optional `basin_routing`, omitted when off, so earlier archives and breadth-first worlds are
  byte-identical), and every consumer that rebuilds its receiver tree uses the world's own record
  (`cartalith_engine::fill_tilt_of`): `fresh_river_network_with` and `fresh_river_channels_with` (settlements and
  Ruling N's river term, the planner's river transport, the route tool, landmarks, the export, the way census), the
  pick bridges and the draw plan's downhill tree. The GPU flow kernel is skipped when it is on, as with rough
  routing: its D8 pick over the 5- and 7-ULP steps has not been measured against the CPU's.
- **§7p.** OFF in `WorldParams::defaults` (every golden and the reference path unchanged by control flow), ON in
  `params::defaults`, OFF for a save without the key; acts only with `integrate_drainage`. A heightmap import keeps
  the breadth-first fill (its relief is data), and so does the rock-paint pass's local coastal cut.

14.4 **The comb measure** (`river_straight::comb_stats`, tested against known combs and known non-combs): the length
of a network that runs 2 to 5 cells beside ANOTHER river, within 20 degrees of its direction, for 12 cells or more,
every line resampled at one cell. `straight_stats` (section 11) cannot see a comb of wavy teeth, and one straight
trunk is no comb. `WorldGen.river_comb` applies both to the traced network and to what Ruling EN's law draws at a
given zoom (`_rivercomb_probe.tscn`).

14.5 **Measured, full pipeline** (`tests/river_basin_routing_measurement.rs`, release, alone, `use_gpu` off; the
world's own drawn network, traced). Columns: the comb share; the straight share at 0.25 cells with the longest
stretch; the same on filled ground and elsewhere; lake cells by the rain test and under the lake water balance
(without Ruling CC's area floor); river mouths; the tilt's largest rise above the spill. `owner` is seed 51203 as the
phone made it; `before` is rough routing on and basin routing off; `after` is Ruling EV.

| World | Arm | Comb | Straight (longest) | Filled ground | Elsewhere | Lake cells, rain test / balance | Mouths | Max rise |
|---|---|---|---|---|---|---|---|---|
| 51203 (owner's, 2048x1024, density 2.5) | owner | 20.9 % | 30.9 % (174) | 81.9 % (127) | 24.3 % (49) | 6 374 / 9 654 | 6 533 | 0.05 m |
| | before | 0.8 % | 9.9 % (59) | 30.9 % (59) | 7.7 % (35) | 6 483 / 10 693 | 4 749 | 0.05 m |
| | **after** | **0.7 %** | **6.3 % (21)** | **7.2 % (17)** | 6.0 % (21) | 7 261 / 12 560 | 4 821 | 0.44 m |
| 246371 (2048x1311, 1.55) | before | 0.8 % | 19.5 % (117) | 34.1 % (88) | 17.6 % (106) | 166 103 / 150 058 | 1 394 | 0.06 m |
| | **after** | **0.7 %** | **13.4 % (42)** | **13.9 % (21)** | 13.2 % (42) | 188 721 / 184 513 | 1 618 | 0.94 m |
| 483920 (1024x656) | before | 1.6 % | 10.2 % (54) | 26.2 % (25) | 9.6 % (28) | 91 589 / 92 366 | 1 635 | 0.02 m |
| | **after** | **1.5 %** | **8.5 % (21)** | **6.6 % (16)** | 8.4 % (21) | 91 704 / 93 106 | 1 675 | 0.12 m |
| 912345 (1024x656) | before | 1.7 % | 12.7 % (58) | 26.5 % (43) | 10.7 % (34) | 11 277 / 17 066 | 1 130 | 0.05 m |
| | **after** | **1.3 %** | **8.4 % (23)** | **7.3 % (16)** | 8.3 % (23) | 12 879 / 18 093 | 1 291 | 0.38 m |

Inside the owner's window (cells 35..295 x 50..463) the traced comb share is 44.3 % as made, 0.6 % before and 0.4 %
after, and the straight share 57.7 % (longest 174 cells), 17.8 % (46) and 7.2 % (17). Plausibility on every arm: no
step of the consumers' rebuilt tree fails to descend the routing surface; dry ends are 8 / 2 / 2 on 51203, 2 / 2 on
246371 and 0 on the two small worlds, so the switch adds none. Building the routing surface costs 73 ms against 66
on 51203, 158 against 142 on 246371 and 33 against 30 on the small worlds (median of five, each spread under 7 ms).

**In the app** (`_rivercomb_probe.tscn`, windowed, GPU on as in the owner's world; seed 51203, the owner's window,
what Ruling EN's law draws at half strength or more): the comb share at x1 / x4 / x7.88 / x16 / x64 is 77 / 66 / 63 /
54 / 46 % as made, 0.8 / 0 / 0 / 0 / 0.3 % before and 0 at every zoom after; the longest drawn straight stretch is
136 to 212 cells as made, 67.9 cells before (the 45-degree line) and 12.4 to 17.7 cells after.

14.6 **What else moves.**
- **Lakes grow where a basin's water now reaches its floor.** The lake water balance (Ruling ET) takes a pool's
  inflow as the highest routed discharge among its cells; with the breadth-first tilt the water crossed a basin
  beside its deepest part, so the pool saw only part of it. Now the whole inflow reaches the floor. The carve also
  follows the converged trunk into the floor, so depressions deepen: filled ground grows 10 to 27 % on the four
  worlds. Lake cells under the balance (no area floor) grow 17 % on 51203, 23 % on 246371, and 1 % and 6 % on the two
  small worlds. In the owner's window the basin that was a part-playa flat crossed by rivers is now a lake (the
  relief frames).
- **On the owner's world as the app makes it the growth is three times larger** (corrected 2026-11-13; the figures
  above are `use_gpu` off, and the GPU stages make a slightly different world). With `BR_GPU=1`, under the balance and
  Ruling CC's area floor as the map draws them: 18 130 lake cells before and 27 640 after on the whole map (+52 %),
  and **8 622 before and 17 782 after inside the owner's window (2.06x)**, which is 64 % of the map's balanced lake
  cells; by the rain test alone 11 284 and 11 791 (+4.5 %; 2 583 and 2 923 in the window), so most of it is the
  balance's inflow, not the carve. As the owner's phone made it (balance off) the window held 1 013 rain-test lake
  cells (before the area floor); the balance alone, on that same world, holds 8 205 there. So Ruling ET's own change
  (approved on frames routed breadth-first) is the first step, about 8x, and basin routing the second, 2.06x. The
  balance counts every cell's rain as inflow (a mean of 1 a cell, against
  2.0 a cell of evaporation at 25 C), which the breadth-first tilt hid in large flat basins by under-counting what
  reached the floor. An owner question (`OUTSTANDING_WORK.md` §2.13).
- **Rivers.** The traced network is 1 to 5 % shorter (parallel channels merge) and more rivers end in a lake (mouths
  up 2 to 16 %). On the owner's world the window draws less river at x1, because the lake covers it.
- **Consumers.** In the app, 51203 has 40 settlements on all three arms and 48 / 42 / 40 roads; the knowledge
  fixtures were regenerated (their worlds' rivers moved).

14.7 **Verified.** `cargo test --workspace` and the literal mutants are recorded in `STATUS.md`'s entry for this
work. Frames: `cartalith-native/docs/evidence/river-comb-2026-11-13/` (the river flow and relief views at x1, x4,
x7.88, x16 and x64, owner / before / after, and the owner's composite), looked at at every zoom: the after frames
show a converging, branching network across the basin, with no parallel teeth and no 45-degree line. One thing the
frames show that is not a river: at x64 the new lake's shore is drawn cell-stepped, where earlier x64 lake frames are
smooth; it is routed as a symptom to measure (`OUTSTANDING_WORK.md` §2.13). *(Superseded the same day, 14.8: the
stepping showed from x7.88, not only at x64, and it is diagnosed and fixed.)*

14.8 **Verifier findings on this work (2026-11-13), each confirmed at its symbol first.**
- **The stepped lake shore: diagnosed and fixed, render only.** It shows at x16 and in the owner's own 788 % view too,
  not only at x64 as 14.7 said, and on balanced lakes elsewhere before this ruling (seed 483920). Cause: the shore
  field (`render::shore_margins`) is the smaller of a depth margin and a RAIN margin, because the reference's rule
  makes a pooled cell lake only where its rain reaches `LAKE_RAIN`; the water balance replaces that test for every
  pool, so a balanced lake in dry country (rain under 0.22) had a negative rain margin on every cell, the class clamp
  lifted each to `SHORE_EPS`, and the contour hugged the water cell centres: straight grid-aligned runs, rectangular
  inlets, square islands. A world the balance classified now contours its lakes by depth alone
  (`shore_margins_with`, `lakes_rain_gated`, the screen, the deep-zoom tiles and both exports). Tested at literals on
  a dry bowl (`tests/balanced_lake_shore.rs`: the shore crosses 0.6 of a cell where the ground meets the water, where
  it crossed 0.0001 before) and on the tiles (`a_balanced_dry_lake_has_the_same_smooth_shore_in_the_tiles`); a
  rain-test world's field is bit-identical. Looked at in the frames at x7.88, x16 and x64.
- **The lake growth in the owner's view was understated** (14.6, last paragraph): 2.06x in the window and +52 % on the
  map with the GPU on, where 14.5 quoted +17 % with it off.
- **The comb measure in the window, and what Fix 2 changed there.** With rough routing alone (`before`) the drawn comb
  share in the owner's window is already 0 to 0.8 %: Fix 1 removes the comb. What remained is the 45-degree ruler line
  and straight runs: the traced network in the window was 22.8 % straight with a 110-cell stretch, and basin routing
  takes it to 7.6 % and 17 cells. The drawn total at x7.88 falls 3 616 -> 1 487 cells; the traced network in the window
  only 14 486 -> 11 853 (14.9 has how much of each lies under a lake).
- **Floor's creep** (14.3, "What it does not guarantee"): confirmed, measured, the claim corrected; not fixed.
- **Heightmap imports never get basin routing**: confirmed, and by design. `infer_tectonics` routes over the
  breadth-first fill and records `basin_routing: false` (`import.rs`), so the import restore (14.1) gives an imported
  world the current rules table but not basin routing; an imported DEM's filled pits are data, as its relief is. The
  claim "Fix 1 covers the import" stands for the table; the routing of an import is unchanged and not measured.
- **Small knots at confluences** (a two-cell blob where two trunks join with a one-cell wobble, a four-way meeting):
  seen in the River flow view, the per-cell flow raster, not combing; routed (`OUTSTANDING_WORK.md` §2.13).

14.9 **The owner's window, measured in the app** (`_rivercomb_probe.tscn`, windowed, GPU on, seed 51203, cells
35..295 x 50..463; `traced_lake` is the traced network where it lies on a cell the map draws as lake, a stage added
for this). Owner / before / after:

| Measure in the window | as made | before | after |
|---|---|---|---|
| Traced network, cells | 30 443 | 14 486 | 11 853 |
| of it under a drawn lake | 630 (2.1 %) | 1 543 (10.7 %) | 3 456 (29.2 %) |
| Traced comb / straight share, longest straight | 75.1 % / 81.9 %, 341 | 0.6 % / 22.8 %, 110 | 0.2 % / 7.6 %, 17 |
| Drawn at x7.88 (Ruling EN's law, half strength), cells | 5 265 | 3 616 | 1 487 |
| Drawn comb / straight share at x7.88, longest straight | 62.8 % / 65.1 %, 211 | 0 % / 29.6 %, 68 | 0 % / 19.9 %, 12 |

So in the window rough routing removes the comb, and basin routing removes the 45-degree ruler line and the straight
runs; the drawn total falls 59 % at x7.88, where the traced network falls 18 % and 29 % of what is left lies under the
new lake: the rest of the drop is the network converging into fewer, larger rivers, which the law draws by discharge.

14.10 **The lake balance counts a runoff share of rain (owner choice 2026-11-13, "Runoff share of rain").** The question
14.6 left open. Built as `cartalith_civ::lake_balance::RUNOFF_SHARE` (0.58), a factor on every pool's inflow; basin
routing stays on. Measured with `tests/lake_runoff_measurement.rs` (`BR_GPU=1`, whole map, Ruling CC's area floor
applied, lake cells / lakes):

| World | before basin routing, whole discharge | basin routing, whole discharge | basin routing, share 0.58 |
|---|---|---|---|
| 246371 (owner) | 160 817 / 94 | 171 809 / 111 (+6.8 %) | **171 399 / 95 (+6.6 %)** |
| 483920 | 58 516 / 40 | 58 976 / 41 (+0.8 %) | **47 189 / 41 (-19.4 %)** |
| 912345 | 19 984 / 33 | 26 261 / 31 (+31.4 %) | **22 971 / 30 (+14.9 %)** |

A flat share sweep on the basin-routed fields (cells, change against the first column): 246371 is almost insensitive (0.2: +1 %; 0.5 to 0.8: +6 to +7 %, a
single large basin set that always spills); 483920 falls steadily as the share does (0.2: -39 %, 0.4: -29 %, 0.5: -23 %,
0.55: -21 %, 0.58: -19 %, 0.8: -10 %); 912345 is a cliff (0.4: -27 %, 0.5: -3 %, 0.55: +9 %, 0.58: +15 %, 0.6: +31 %, as one large
basin begins to spill). Hence a narrow window, 0.56 to 0.59, in which all three are within 20 % of their pre-Ruling-EV
size. **A climate-dependent share was tried first and dropped:** the Budyko/Fu share `1 - E/P` (`w` 2.6) read at each
pool's own mean rain and temperature, clamped to 0.1..0.6, gave 0 %, -33 % and -60 % on the three worlds; clamped to
0.3..0.8, -31 % and -50 % on the last two; clamped to 0.5..0.8 it was identical to the flat 0.5 (every pool at the
floor). The pool's local climate is the wrong read for an inflow that is the whole upstream catchment's discharge.

The owner world (246371) is dominated by one inland sea of about 100 000 cells that spills at any share, so its total
hardly moves (410 cells dry at the rim); the share's effect is in the smaller basins (483920 loses 11 787 cells). Frames
(a classification drawing, not the app's renderer): `cartalith-native/docs/evidence/lake-runoff-2026-11-13/` (`<seed>_before_x1`,
`_after_x1`, `_before_x4`, `_after_x4`; before = whole discharge, after = the share; a tan cell is a lake the other arm held).
The 14.6 owner-window figure (2.06x on seed 51203) was not re-measured: the owner's 51203 world is not one of the three
measured here.

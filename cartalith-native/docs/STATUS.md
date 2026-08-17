# Status quicklist

A living checklist, not a narrative — read this first, before `CHANGELOG.md`,
to know what's done vs. open without re-reading the whole history each
session. Update it in the same commit as whatever changes its answer.
`CHANGELOG.md` stays the detailed record of *how*; this is only *what/done?*.

Last updated: 2026-08-17 (post real Android device pass — MVP criterion 4 fully closed —, sea routes (Phase 2 milestone 13) wired into `cartalith-godot`'s rendering with a real render-loop crash found and fixed along the way, CPU-multithreading milestones 2-3 — `cartalith-civ` then `cartalith-climate`/`cartalith-erosion`/`cartalith-hydrology` Rayon-parallelized — Phase 1's two closeout items (credits screen, crate license audit) both done, GPU layer integration milestones 7-8 — GPU-backed weather simulation, shared GpuContext across `generate_terrain`'s stages — a new standalone `cartalith-spatial` crate (tiling/quadtree/dirty-tracking base for a future LOD integration — real, tested, referenced by nothing yet), Phase 2 milestone 16 (`_civGenerateProvinces` — resolved the milestone-9 territory-input blocker via milestone 10's own `assign_territory`, data/backend done and verified, rendering wired as a boundary-line overlay in a same-day follow-up), and Phase 2 milestone 17 (economy/Journey Planner investigated for real — two separate large subsystems found, not one; the ~70-function Journey Planner confirmed to genuinely need its own sub-phase per `ROADMAP.md`, not attempted; `civ_resource_trade_balance` ported/tested from the smaller economy layer — **now genuinely wired**, same day: `civ_world_mean_resources`/`civ_place_resource_context` give it real per-settlement inputs, `get_trade_balances()` exposes the result to Godot, and the memory-optimization tension (needs all 15 resource keys, six were being freed early) resolved by moving that free to after settlements are placed — full reasoning in `ECONOMY_SCOPE.md`), Phase 2 milestone 18 (culture beyond naming, investigated — confirmed one real computation exists beyond the already-ported syllable tables, `_civCultureTerrainFit`/`civ_culture_terrain_fit`, ported and tested but not yet wired since its real inputs depend on the still-unstarted `_civFactionAggregates` territory aggregation; Government/Religion/Ag-tech confirmed genuinely UI-only with zero derived computation; a completely unrelated "culture profiles" system found at reference lines 28193+ correctly identified as Phase 5 Urban Morphology scope, not Phase 2), Phase 2 milestone 19 (Journey Planner milestone 1 — the two fully self-contained categories of its ~70 functions ported: physical-modeling primitives and the reference's own "four deferred items" seasonal/closure cluster, 22 tests, full remaining milestone breakdown in new `JOURNEY_PLANNER_SCOPE.md`), Phase 3 milestone 1 (`TerrainAppearance` abstraction in `render.rs` — colour data now owned/structured, pixel-identical output verified, real audit finding that no elevation-breakpoint ramp exists in this renderer), and the GUI shell redesign milestone 1 (`GUI_SHELL_SCOPE.md` — full 6-region professional-editor shell rebuilt in `main.tscn`/`main.gd` from an owner-supplied design import, zero Rust changes, every real control re-parented and screenshot-verified working end-to-end, every not-yet-real feature visibly present but honestly disabled), and the causal-chain explainer (`VISION.md` sequencing item 1 — hovering a settlement shows a real "WHY HERE?" decomposition of `build_settlement_suitability`'s own thirteen weighted terms; proved faithful by a test that reconstructs the real function's output at every cell from the explanation alone, and cross-checked against real terrain across all 40 settlements of a generated world with 0 violations; deliberately per-settlement rather than a general `explain_cell(x,y)`, since the source rasters aren't retained on `CivData` and holding them would undo the memory work) — see `CHANGELOG.md`).

## GUI shell (`GUI_SHELL_SCOPE.md`, milestone 1 done 2026-08-17)

Owner-supplied design import (`claude_design` MCP) redesigning the whole
Godot UI as a professional-editor shell — top bar (7 domain menus),
workspace navigator (4 subject groups), a second panel that swaps with
navigator selection, mode bar + viewport, right context inspector, bottom
timeline bar. Owner decided: target this port not the JS reference app (the
mockup's own `#id`-re-parent notes describe `Cartalith Gen1 v2.10.html`'s
DOM, a different frozen file in a different repo); build the full shell
structure now, wire only what has real engine backing, leave the rest
visibly present but honestly `disabled`.

**Milestone 1 done**: the shell exists, every real control (seed/
resolution/width/sea level/world shape/experimental flags/villages/the
three map-overlay toggles/load-save/credits) re-parented with zero
`main.gd` reference changes (Godot's `%UniqueName` lookup is
position-independent) and zero Rust changes. New: a settlement-hover
signal (`map_overlay.gd`) feeding the new Inspector panel with real data.
Screenshot-verified end-to-end: generation, all overlay toggles, navigator
swapping, settlement-hover inspector, and the credits dialog all confirmed
working through the new shell on a real Windows run. Deferred, as scoped:
light theme, panel collapse/rails, all three responsive breakpoints,
terrain appearance's actual editing GUI. Full record, including every real
feature-inventory correction found along the way: `CHANGELOG.md`'s "GUI
shell redesign milestone 1" entry, `GUI_SHELL_SCOPE.md`'s own "done" note.

## MVP_SCOPE.md — "done means all seven"

| # | Criterion | Status |
|---|---|---|
| 1 | Height/temp/rain/flow match golden data | **Done.** Every pipeline stage golden-verified bit-exact/tight-tolerance against the real JS engine: tectonics/orogeny (graph-driven T1-T5), volcanism+provinces, climate (temp/wind/rain), ocean currents, terrain wind deflection, erosion, hydrology, world-structure archetypes, full carve pipeline. Nothing left pinned to a stale default. The Rust side was always correct; a separate UI-only bug (fixed 2026-08-17, see `CHANGELOG.md`'s "Fix: World Shape archetype selection had no effect on generation") meant the Godot UI's World Shape dropdown never actually reached `generate_world_structure()` — that gap is now closed, real screenshot-verified. |
| — | UI/UX (not one of the seven, but part of the `/goal` "feature and graphic parity" directive) | **Reskinned 2026-08-16, then re-themed same day per explicit owner feedback.** First pass: `ui-ux-pro-max` dark-dashboard design system, grouped World Parameters/World Structure/Advanced cards, visible keyboard-focus states. Owner preferred the reference HTML's own look, so the palette was swapped to a literal port of the reference's real `:root[data-theme="light"]` parchment theme (`#efe7d6`/`#fbf5e9`/`#b07f3f` accent) — not a fresh design-system search, the actual CSS values from `Cartalith Gen1 v2.10.html` line 271. Confirmed by real-window screenshot that the map's own pixels are untouched by the theme swap (JS/Rust colour ramps, not CSS/Theme — same guarantee the reference's own code comment makes). Deferred: real Fira font files (license-unverified, kept Godot's default font). **`MVP_SCOPE.md` point 9 (sea level) done 2026-08-17**: a new `Sea level` `SpinBox` (0-100%, matching the reference's own `#seaV` slider convention) in `WORLD PARAMETERS`, wired via a new `WorldGen.set_sea_level` `#[func]`. Real screenshot-verified: seed 12345/512²/Classic at 42% vs. 15% produced dramatically different coastlines (most of the ocean became land at 15%), confirming the control has a real effect, not just a cosmetic one. Only takes effect under the Classic world shape — named archetypes re-anchor sea level from their own land-fraction target (`apply_world_structure_sea_level`), a real, documented, pre-existing interaction, not a new limitation. See `CHANGELOG.md`'s UI reskin and "real Windows hands-on verification" entries. |
| 2 | Recognisable 2D map render | **Done (2026-08-16).** Replaced the placeholder elevation-only tint with the reference's real default-settings biome/hillshade renderer (`crates/cartalith-godot/src/render.rs`, new): `materialWeights` (snow/rock/sand/wetland/canopy/grass), the six climate-selected colour ramps, multi-scale hillshade, `bioBlend` desaturation, edge haze, and `seaColorCore` (smoothed-bathymetry depth/temperature banding — confirmed this is JS's real default, not a stretch feature). Two real bugs caught by golden verification, not by read-through: a missing final `ao*vignette` multiply (~40% too bright at corners) and sea colour needing the smoothed, not raw, depth field. Golden-verified against two real `generate()` runs at `1e-4` tolerance (`golden_parity_render.rs`). Deliberately excludes every `state.viz.*`-gated stretch feature (splat texturing, geology, NPR "Painter" styles, AO/SVF/shadow, SDF tinting) — all off at JS's own defaults; that's genuine Phase 3 scope, see below. |
| 3 | Windows `.exe` builds + owner has run it | **Done (2026-08-16).** Ran the actual windowed MVP UI (not `--headless`) on this session's real Windows desktop: launched, screenshotted via `PrintWindow`, drove real synthetic mouse clicks at real screen coordinates. Confirmed generation end-to-end (real biome-coloured map, correct status label) under the new light theme. Caught two real bugs this way that no amount of code review had surfaced: the World-Structure dropdown rendered blank (malformed hand-authored `.tscn` item properties; GDScript's negative-index fallback meant it may have silently been generating with the `Rift` archetype instead of `Classic` this whole time), and the window title was still "walking skeleton". Both fixed and re-verified by the same screenshot method. See `CHANGELOG.md`'s "real Windows hands-on verification" entry. |
| 4 | Android `.apk` builds + owner has installed/run | **Fully done, 2026-08-17** — a genuine first for this project (real OnePlus 6T, Android 14). Build → install → launch confirmed via logcat (GDExtension loaded, real OpenGL ES 3.2 context on the device's Adreno 630 GPU). Once the owner unlocked the device mid-session, the **golden path was driven for real**: tapped Generate, a fresh 512×512 generation completed in ~7-9s wall-clock with **peak memory ~283,326 KB PSS (~277 MB)**, settling to ~271,290 KB steady-state, no leak; a same-seed regeneration reproduced the identical rendered world (terrain/settlements/roads), confirming the full pipeline ran correctly on-device; no ANR/crash/hang anywhere in logcat. `TOOLCHAIN.md`'s own "highest-risk item" framing turned out to already be a non-issue — every piece of the Android toolchain was already correctly installed and wired from earlier work. Full record in `ANDROID_BUILD_SCOPE.md`. |
| 5 | Map width scales feature size | **Done** — a consequence of criterion 1's parity, verified via the world-structure archetype port. |
| 6 | Changelog entry per milestone | **Ongoing** — `CHANGELOG.md` has an entry for every milestone so far; keep this up. |
| 7 | Opens a real HTML-app `.zip`, renders it, checked against the HTML app's own output | **Done (2026-08-16).** `cartalith-io::load_save` verified bit-exact against a real export produced by running the actual, unmodified reference engine (not just its own synthetic round-trip tests): `crates/cartalith-io/tests/golden_parity_real_export.rs` against `crates/cartalith-io/tests/fixtures/real_export_seed24601.zip`. See `CHANGELOG.md`'s "cartalith-io verified against a real HTML-app export" entry for the harness technique (including a genuine `generate()`-name-collision gotcha found along the way). |

## ROADMAP.md phases

| Phase | Status |
|---|---|
| 0 — Walking skeleton | **Done.** Triangle/button/`ping()` confirmed on Windows and Android (build+package; Android run-on-device is the one open half, see criterion 4 above). |
| 1 — Terrain MVP | **7/7, all done, plus both closeout items, 2026-08-17.** Criteria 1/2/3/5/6/7 done; criterion 4 (see its own row above) fully closed 2026-08-17 — real device build/install/launch plus a real driven golden-path generation, both confirmed. The two "easy to forget" Phase-1 closeout items `ROADMAP.md` names are now also done: a real crate license audit (`cargo license --all-features`, ~190 of ~200 workspace dependencies permissive MIT/Apache-2.0/BSD/Zlib/ISC-family; the one weak-copyleft dependency is `gdext` itself under MPL-2.0, used unmodified as this port's own Rust-Godot binding; no GPL/LGPL/AGPL anywhere) and a real, reachable credits screen (header "ⓘ" button → `CreditsDialog`, `godot-project/credits.gd`) carrying forward the reference HTML's own `#creditsModal` attribution plus this port's own license-audit findings. Screenshot-verified reachable and scrollable through both sections. See `CHANGELOG.md`'s "Phase 1 closeout" entry. |
| 2 — Civilisation layer | **Started 2026-08-16, milestones 1–15 of an unknown-but-large number done** (milestone 10, territory/border generation, has an owner decision recorded — `DECISIONS.md` §7b, cost-distance Voronoi from capitals, strength-weighted — implementation status tracked separately, not this row's concern to restate). `cartalith-civ` crate (zero `gdext` dependency), every field golden-verified against the real reference engine. **1** lithology/soil fertility/water access. **2** water-body classification (ocean/lake, priority-flood depression fill). **3** biome classification (12 climate categories). **4** carrying capacity/NPP/population density. **5** resource potentials (15 geological fields). **6** route corridors/landmass quality/coast SDF. **7** `buildSettlementSuitability`/`findSettlementSeeds` — the "v1.30 one function" `ROADMAP.md` originally named as this phase's landmark, reached and golden-verified. **8** settlement placement + faction assignment — the pure core of `_civIterativeAutoWorld` (land-component labelling, snap-to-land/coast, `_civAssignLandmassFactions`'s capacity-weighted seat apportionment + multi-capital spacing, settlement tier classification), stopping deliberately before the DOM-coupled orchestration shell. **9** settlement population + naming — `_civBasePopForKind`/`_civSettleName` (RNG-driven, reuses `cartalith-rng`'s already-verified `mulberry32` — `_civRng` is the same algorithm under a different seed wrapper, proved by hand not assumed). A genuine, verified reference quirk found here: `state.seed` (distinct from the real per-world `state.tect.seed`) is never assigned anywhere in the reference, so the civ-naming RNG stream is seeded identically for every world regardless of its actual seed — same-rank, same-faction settlements across *different* worlds get identical generated names, a real mechanical consequence, not a bug. Full history, every real bug/gap found (a Node-harness seeding bug, a stale-vs-fresh river-network mismatch, a threshold ambiguity between two real reference call sites, several `WorldState`-retention fixes, a 4-vs-8-connectivity flood-fill distinction, a 4-script-block harness miscount, a snapped-position-vs-original-seed-score `.suit` mixup), and reasoning is in `CHANGELOG.md`'s "Phase 2 milestone 1–9" entries — this row stays a summary, not a repeat of it. **11** road network algorithm — `buildTravelCost`/`roadDijkstra`/`buildRoadNetwork` (a distinct `f64`-priority heap from milestone 2's, per the reference's own v1.89 perf comment; real terrain data exercised the "unreachable landmass" branch, not just a synthetic test). Landed in `cartalith-civ` (a deliberate placement decision, not a default — the functions live in the reference's block 1, weighed against `ARCHITECTURE.md`'s "civ" framing and decided the latter wins). **Investigated for milestone 12, found a real correction to the earlier assumption**: this port's own `_civSeedVillages` dependency reasoning (milestones 8/9's "villages need roads" note) pointed at the wrong system — `buildRoadNetwork` only ever serves the *manual* "Generate Roads" tool (`buildRoadsOp`, reads user-clicked `state.places`); the civ auto-populate flow's own road network (`civWays`) is built by a separate, larger algorithm (`_civHierarchicalNetwork` + `_civMstRoutes` + `_civPreferSeaRoutes`) not yet read or ported. **Milestone 11 does not unblock village seeding** — it's real, useful, tested code for a different (manual-tool) purpose. **12** the real civ-auto-populate road network — `civ_hierarchical_network_topology` (`_civHierarchicalNetwork`'s three real passes: Prim MST, min-degree-fill by settlement tier, Floyd-Warshall shortcut-detour-relief — confirmed a third pass beyond what milestone 11's own scoping estimated). This is the real `_civSeedVillages` dependency milestones 8/9/11 all pointed at without reaching. Split deliberately: ships the raw topology (what road-proximity queries actually need), defers corridor-consolidation/Catmull-Rom-smoothing/road-classification (needs `_civSmoothPath`, not yet ported — milestone 14) since that's presentation polish, not the graph structure itself. Both golden fixtures are real edge cases, not synthetic: one settlement genuinely unreachable in case0; case1's min-degree-fill hitting its natural ceiling (a complete K5 graph) rather than its per-tier target. A real `river_flow_thresh` parameter bug (hardcoded map width instead of the real per-world value) caught and fixed before it shipped. **10** territory assignment — cost-distance Voronoi from capitals, weighted by capital population (`DECISIONS.md` §7b's own design, implemented as designed). The first Phase 2 milestone with no JS reference to port at all — the reference has zero algorithmic territory generation (paint tool + save/load only) — so verification is 8 unit tests standing in for a golden test (equal-population capitals split at the geometric midpoint; a 100k-vs-5k-population pair moves that midpoint to the larger capital, the actual weighting behaviour measured, not just present; unreachable cells stay unowned; a two-capital faction's territory unions both zones). `pop_ref=15000.0` (== a capital's base population before variance) is a documented, non-arbitrary constant. **Rendered 2026-08-16**: `cartalith-godot`'s `build_territory_texture()` turns the per-cell `Vec<i32>` into a low-alpha (`~0.32`) RGBA8 overlay texture, Okabe-Ito-coloured by faction, toggleable via a new default-OFF "Show territory" checkbox — see this row's own catch-up note below and `CHANGELOG.md`'s "UI/UX catch-up: territory + villages" entry. **15** village seeding — `_civSeedVillages`/`_civVillageAcceptProb`/a milestone-12-topology-adapted `_civRoadProximityQuery`, the feature milestones 8/9/11/12 were all working toward unblocking. Closed a real RNG-sharing gap first: `name_and_populate_settlements_with_rng` now threads an external `Mulberry32` (purely additive alongside the existing zero-arg function) so village seeding continues the exact same stream naming left off at, matching the reference's own one-shared-`rng`-closure design. Golden-verified against the real reference (fully synthetic but reference-function-verified inputs, same standard milestone 12 already set) — bit-exact first attempt, including RNG-derived village names and nearest-capital faction inheritance; a second targeted extraction independently confirmed the downsampled-routing-grid-to-full-grid coordinate conversion by matching a hand-calculated `exp()` distance formula to 15 significant figures. **Flagged, not fixed here**: milestones 7-9's existing golden tests seed their candidate lists at a threshold (`0.65`) that traces to a *different* real call site (the standalone JSON-export default) than `_civIterativeAutoWorld`'s own real default (`SETTLE_SEED_THRESH=0.42`, confirmed by tracing why a headless harness's `wantCounts` is always falsy) — not a bug in those milestones' own pure-function correctness, but a pipeline-orchestration question for whatever in `cartalith-godot` builds the real base-settlement candidate list. **Wired 2026-08-16**: `cartalith-godot`'s `compute_civilisation()` now calls `civ_seed_villages` after base settlement naming, sharing the one `Mulberry32` stream this milestone's own doc comment requires (no second, desynced RNG instance), and merges the output `Hamlet`-tier settlements into the same list the UI already draws. The gating question is resolved the way flagged here: a new default-OFF `VillagesCheck` toggle in `cartalith-godot`, matching the reference's own real `_civVillages` default. **14** corridor consolidation + path smoothing — `civ_consolidate_and_smooth_ways`, milestone 12's own deferred tail (reference `_civHierarchicalNetwork` lines ~21670-21739): claims corridor cells busiest-edge-first so shared trunk segments render once, classifies each way by peak usage (`highway`/`regional`/`road`/`track`), auto-names it from its endpoint settlements, and Catmull-Rom-smooths the result (RDP-simplify then chord-length-parameterized spline sampling, both ported fresh from the reference's own `rdpSimplify`/`catmullRomSample`, reference lines 8701/8790) with a terrain-validity repair pass and an endpoint-snap pass so strokes land on their settlement pins. Also ports `_civSmoothPath`/`_civTerrainValidTest`/`_civNearestValidPt` (reference lines 21892/21843/21872), narrowed here to the one call shape this network uses (land-only validity) — the general terrain-validity test also has an ocean-only mode, generalized in by milestone 13's sea routes (this row's own **13** entry above). Golden-verified against two real cases reusing milestone 12's and milestone 9's own already-verified fixtures (no new settlement/topology data invented): a genuine short-segment Catmull-Rom oversampling quirk (a 2-cell path produces a 3-point output whose rounded midpoint coincides with its own start point) and a real K5 corridor-sharing case (10 edges, a mix of visible and fully-consolidated hidden ways). **13** sea routes — `civ_sea_routes` (`_civMstRoutes(ports,true)`, reference line 21240, `isSea` branch only — the `isSea=false` land branch has no confirmed real caller, `_civHierarchicalNetwork`/milestone 12 is what the real land network uses). Shares `_civSmoothPath`/Dijkstra/Prim's-MST shape with milestone 12 but is a genuinely separately-scoped algorithm: the cost grid marks land `Infinity` (impassable, not merely expensive — the reference's own fix-history comment explains a finite land cost let Dijkstra cut across jagged coastline pixels, and smoothing then exaggerated those cuts into visible loops), ports snap to the nearest navigable-ocean cell at radius 10 (deliberately wider than milestone 12/14's radius 6 on a different cost grid), and a v0.73 sea-lane augmentation pass adds each port's nearest reachable port as a direct lane (capped at 1.15x the MST's own longest hop) beyond the bare tree. `_civSeaTimeEdgeCost` (v1.98 current/wind-costed sea-lane pricing) deliberately not ported — its real inputs (ocean-current/wind u/v fields) aren't retained on `WorldState` past their internal use in `apply_ocean_currents`/`deflect_flow`, so this port takes the reference's own documented graceful-degradation fallback (uniform arithmetic cost) rather than adding new plumbing outside this milestone's scope — a real, flagged follow-up. Four existing helpers generalized (not duplicated) to support both land and ocean validity modes: `civ_snap_finite` (added a `max_r` parameter), `civ_is_valid_land`→`civ_is_valid_terrain` (added the `_civTerrainValidTest('ocean')` branch this row's milestone-14 note flagged as unported), `civ_nearest_valid_pt`/`civ_smooth_path` (both threaded the same `is_sea` flag through). Golden-verified against the real reference: a fresh Node harness caught and fixed a real bug in itself before trusting extraction (`generate()` is `async`, and a bare unawaited call left `field` at its default-zero fill, `currentWaterBodies()` reporting 100% ocean — fixed by awaiting properly, then cross-checked `field[0]` plus land/ocean/lake cell counts against already-trusted fixtures). Reused milestone 14's own case0/case1 fixtures (already-verified coastal settlements over genuine mixed land/ocean/lake geography at both grids) — both cases matched the Rust port's output exactly on the first run, including a real reference quirk where two of case1's four routes carry `km:0` despite having real points (`_civSmoothPath` accumulates `km` over rounded sample points before its own final step restores full-precision endpoints, so a short diagonal hop's only interior sample can round to coincide with the pre-restore rounded start point). **17** economy/Journey Planner investigated for real (2026-08-17), full reasoning in `ECONOMY_SCOPE.md` — two separate, both genuinely large subsystems turned out to exist under "economy": the Journey Planner (`jp*`/`_jp*`, reference lines ~17300-20400, ~70 functions covering transport-mode selection, physical travel cost, consumption/resupply, seasonal closures, multi-stage route derivation) confirms `ROADMAP.md`'s own "consider it a sub-phase" warning as accurate, comparable in size to this port's entire civ-layer effort to date — not attempted. The faction/settlement economy layer (`_civFactionAggregates`, ~165 lines; `_civPlaceTrade` and its dependency cluster) is smaller but still real, explicitly "NOT new simulation" per the reference's own header comment (a display/aggregation layer over already-computed state). `civ_resource_trade_balance` (`_civResourceTradeBalance`, reference line 24175, v1.33's unification of two drifted copies) ported and tested — the one fully self-contained piece, operating on caller-supplied catchment/world resource means with no new upstream dependency. Seven real unit tests (no golden harness needed — small, pure, branch-complete, no RNG/iteration-order risk, same precedent as territory/provinces). A real, disclosed tension found and left unresolved: the full trade layer needs all 15 `CIV_RESOURCE_KEYS` resident, but the memory-optimization pass (commit `62b9b51`) frees 6 of them after use — flagged for whoever ports the next slice. Not wired anywhere yet — no real caller exists until the broader orchestration is built, the same "don't wire in what nothing calls" discipline milestone 9's own territory note established.

**Reached**: settlements with real names/populations/faction ownership, faction territory ownership per cell (wired and rendered), the real auto-populate road topology (12) consolidated, classified, and Catmull-Rom-smoothed (14) — **now wired into `cartalith-godot`'s `compute_civilisation()` and rendered as the map's actual road layer**, replacing both milestone 11's manual-tool stand-in and milestone 12's raw unsmoothed topology (fixed same-day as a third UI/UX catch-up pass — see below), and village seeding (15, wired and rendered, and now reading the real milestone-12 network for its own road-proximity check too, not the old stand-in), plus sea routes (13, `civ_sea_routes`, golden-verified, and now wired into `cartalith-godot`'s rendering too — dashed-style, distinct from land roads, see this row's own **13** entry above and `CHANGELOG.md`'s "Wire sea routes" entry). plus provinces (16, `civ_generate_provinces`, resolved a blocker recorded since milestone 9 once milestone 10's own `assign_territory` turned out to produce `civTerritory`'s exact needed shape — data wired into `cartalith-godot`, `get_provinces()`/`build_province_boundary_texture()` real and verified against live generated data, **and now rendered too** — a `ProvinceBoundaryView` overlay + `ProvinceLayerCheck` toggle, thin boundary lines layered on top of territory's own fill, sidestepping the unbounded-province-count palette problem entirely since a boundary line needs no palette; see `CHANGELOG.md`'s "UI/UX catch-up: render province boundaries" entry, including the direct headless pixel-count verification used after a static screenshot proved inconclusive for a 1px line). **Not reached**: culture (beyond naming flavour), economy, the Journey Planner (block 2 proper, otherwise untouched). See `PHASE2_SCOPE.md` for the living milestone list. **UI/UX caught up 2026-08-16** (owner request: "with every milestone and phase the GUI and UX should be updated as well... use a separate agent", a continuous per-milestone practice) — **first pass**: settlements + the milestone-11 road network render on the map. **Second pass**: territory (10) and villages (15) wired and rendered (low-alpha faction-colour territory overlay, default OFF; villages merged into the settlement marker list, default OFF). **Third pass, same day**: found `compute_civilisation()` was still building its road data from milestone 11's manual-tool stand-in — not even milestone 12's own raw topology, a deeper gap than "just wire in milestone 14's smoothing." Fixed the real chain (`civ_hierarchical_network_topology` → `civ_consolidate_and_smooth_ways`, reordered so the smoothing/naming step runs *after* settlement naming, since it needs named endpoints); `get_roads()` now returns classified `Way` data (`points`/`brks`/`way_type`/`name`) instead of raw cell-index paths, `map_overlay.gd` gained a distinct continuous-coordinate `_point_to_screen` (settlement markers still use the cell-centering `_cell_to_screen` — using the wrong one for roads would have shifted every line half a cell) and break-aware polyline drawing so real internal gaps in a consolidated way don't render as a phantom straight line across them. Road width now varies by classification. Screenshot-verified: roads changed from straight/jagged MST approximations to visibly smooth curves following terrain. See `CHANGELOG.md`'s "UI/UX catch-up: wire milestone 14's smoothed roads into the map" entry. **Fourth pass, 2026-08-17**: sea routes (13) wired end-to-end — `CivData.sea_routes`, `get_sea_routes()`, `map_overlay.gd`'s dashed navy-underlay/light-dash rendering (reference's own line-~15511 convention). Real screenshot verification caught a genuine crash (an infinite-loop/buffer-overflow bug in the dashed-line draw routine, triggered by float drift over a long route) before it could ship — fixed and re-verified against the exact config that crashed. See `CHANGELOG.md`'s "Wire sea routes" entry. |
| 3 — Rendering and 3D | **Started 2026-08-17** (`TERRAIN_APPEARANCE_SCOPE.md` milestone 1, owner-supplied `TERRAIN_APPEARANCE_RESEARCH.md`) — `render.rs`'s colour logic (25 material/water palettes + shading constants, previously bare module consts) now lives behind a real, owned `TerrainAppearance` struct, pixel-identical output verified via `golden_parity_render.rs` unmodified. Real audit correction: there's no elevation-keyed colour *breakpoint ramp* in this renderer at all — colour comes from a continuous material-weight blend (temperature/moisture/slope/relative-elevation/aspect/curvature), not a MapTiler-style elevation lookup, so the research doc's own mental model doesn't map onto how this renderer actually works; a literal elevation ramp would be new visual-layer design work for a future milestone, not a re-encoding. Not yet wired to any UI — standalone-but-real, matching `cartalith-spatial`'s precedent. Three things to remember for what comes next: **(a)** criterion 2's renderer (above) ports the reference's *default-settings* material model only — real biome colours, real hillshade — explicitly excluding every `state.viz.*`-gated stretch feature (splat texturing, geology microtexture, NPR "Painter" styles, AO/SVF/shadows, multi-sun, SDF coast/river/biome tinting). Wiring any of those in is genuine Phase 3 work. **(b)** When that work lands, re-invoke `ui-ux-pro-max` for the UI side rather than bolting raw sliders onto the newly-exposed params — keep it consistent with the 2026-08-16 light parchment theme (ported from the reference's own `:root[data-theme="light"]`), not the earlier dark-dashboard match that theme replaced. **(c)** GPU compute *via Godot's own renderer* was researched 2026-08-16 (prompted by `godot-demo-projects/compute/heightmap`) and found not applicable *through that path*: `project.godot` uses the `gl_compatibility` renderer, which doesn't support `RenderingDevice` compute dispatch at all (engine-level constraint, already documented in `.claude/skills/godot-shell/SKILL.md`). That finding does **not** apply to a *standalone* `wgpu` instance created directly by Rust code — see the GPU-compute pilot section below, which tested exactly that and found the hardware path itself viable (the renderer choice is irrelevant to a `wgpu` instance that never touches Godot's own rendering pipeline). If Phase 3 revisits Godot's own renderer for other reasons (3D terrain drape, particles), GPU-accelerated presentation-layer work *through Godot* becomes reachable as a further, separate option — not before, and not for core generation (which must stay CPU-Rust for golden-parity reproducibility regardless of renderer). |
| 4 — Asset Library | Not started. |
| 5 — Urban morphology | Not started. |

## GPU-compute pilot (`GPU_COMPUTE_PILOT_SCOPE.md`, `HARDWARE_ACCELERATION.md`)

**Done, 2026-08-16.** Piloted a standalone `wgpu` compute path (new crate
`cartalith-gpu`, no `gdext` dependency) on one kernel: `cartalith_noise::vnoise`.
Findings:

- **The `wgpu` hardware path itself works cleanly** on this session's real
  hardware (AMD Radeon RX 7800 XT, Vulkan backend, discrete GPU) —
  instance/adapter/device creation, conservative limits, shader compile,
  dispatch, readback all function correctly.
- **This specific formula is not GPU-viable in `f32`** — `hash`'s
  f64-magnitude-dependent rounding (its own doc comment already flagged
  ~2^61 intermediate products, past `f64`'s own exact range) does not
  survive a portable `f32` WGSL port: 100% of cells diverge at 128×128,
  max abs diff `0.93` on a `[0,1]` output. Measured, not assumed.
  `self_test` (the real correctness gate) correctly reports FAIL and the
  CPU fallback is correctly used instead.
- **`f64` in WGSL is a dead end on this toolchain regardless of hardware
  support** — `wgpu::Features::SHADER_F64` is present on this adapter, but
  naga (wgpu 30's WGSL compiler) has no `enable f64;` implementation at
  all. A real, precise finding, not a shrug.
- **Real GPU-vs-CPU timing measured**: GPU loses at 128×128 (dispatch
  overhead dominates, 0.20×) but wins increasingly at scale — 4.46× at
  512×512, 15.65× at 1024×1024, 19.55× at 2048×2048.
- **Verdict**: the `wgpu` path is a real, viable option for *future*
  candidate kernels that don't share `hash`'s f64-precision dependency
  (e.g. presentation-layer work — hillshade/AO synthesis, biome
  classification — pure functions of already-computed fields). Not this
  kernel, not right now, and no wider `HARDWARE_ACCELERATION.md` adoption
  decision has been made — this pilot answers one narrow question, per its
  own scope doc's explicit boundary.

See `CHANGELOG.md`'s "GPU-compute pilot" entry for the full numbers and
reasoning. Nothing outside `GPU_COMPUTE_PILOT_SCOPE.md`'s "In scope" list
was implemented (no capability-tier classifier, no diagnostics panel, no
telemetry system, no tiled compute) — all still deferred exactly as that
document scoped them.

## GPU layer integration (`GPU_LAYER_INTEGRATION_SCOPE.md`)

Follow-up to the pilot above, prompted by the owner's explicit "connect
GPU for each layer" directive (2026-08-16) plus a real architectural
correction: Cartalith generates a **static map from a one-shot batch
simulation**, not a continuously recomputing app — significantly narrows
`HARDWARE_ACCELERATION.md`'s scheduling/priority/thermal sections (see
`GPU_LAYER_INTEGRATION_SCOPE.md`'s own annotation).

**Milestone 1 — GPU-safe noise redesign: done (2026-08-16).** The pilot's
"not viable" verdict on `hash` was specifically about reproducing JS's
exact double-precision rounding, not about GPU noise being impossible.
`cartalith_noise::gpu_hash`/`gpu_vnoise` (PCG3D-based, pure `u32`
wrapping arithmetic, cited: Jarzynski & Olano, JCGT 2020) verified against
their own GPU counterpart (not JS — `DECISIONS.md` §7a) at 512×512: 0
mismatches at `1e-5` tolerance, max diff 1.28e-6. Real timing: 2.85× at
512², 10.39× at 1024², 11.94× at 2048² (the port's real default
resolution). `hash`/`vnoise` themselves untouched — every existing
JS-matching golden test still passes unmodified. See `CHANGELOG.md`'s
"GPU-safe noise redesign" entry for the full record.

**Milestone 2 — domain warp + crustal heterogeneity on GPU: done
(2026-08-16).** `cartalith_noise::gpu_fbm` (6-octave combinator over
`gpu_vnoise`) plus `cartalith-gpu`'s `gpu_warp.wgsl`/
`gpu_heterogeneity.wgsl`. Non-`world` branch only (periodic/`pfbm`
GPU equivalent deferred). `gpu_heterogeneity` (one `gpu_fbm` call/cell)
matches its CPU twin at `1e-5`, 0 mismatches at 512×512 — confirms
`gpu_fbm` itself is clean. `gpu_warp` (chains two nested `gpu_fbm`
evaluations) needed its own, separately-justified `WARP_TOLERANCE=2e-4`
— a real, measured, structural effect (float-scheduling residue from the
first evaluation amplified through the second), not a loosened test.
Real timing: `gpu_warp` up to 80× at 2048² (24 octave-calls/cell — even
better than milestone 1's bare noise, since GPU's fixed dispatch
overhead amortizes further against costlier per-cell work);
`gpu_heterogeneity` up to 16.7×. `compute_warp`/`compute_heterogeneity`
(CPU, JS-matching) untouched, their own golden-parity tests unaffected.
Found (not introduced): `cargo test -p cartalith-gpu` alone can hit a
flaky driver-level crash under parallel GPU-context churn — reliable
with `--test-threads=1` or as part of a full workspace run. See
`CHANGELOG.md`'s "GPU layer integration milestone 2" entry.

**Milestone 3 — the height formula (`compute_height`) on GPU: done
(2026-08-16).** Treats upstream fields (base/stress/flex/hetero/age/
warp/oro) as opaque GPU buffers — plate assignment/stress/flexure/
orogeny's own GPU portability is deliberately NOT this milestone's scope.
Added `cartalith_noise::gpu_ridged` (the noise-combinator gap milestone 2
anticipated) plus `cartalith-gpu`'s `gpu_height.wgsl`/`dispatch_gpu_height`.
Both `ridged=false`/`true` verified against a CPU twin at 512×512: 0
mismatches, max diff `1.19e-7` — essentially `f32` machine epsilon, given
its own tight `HEIGHT_TOLERANCE` (this kernel has one noise call/cell,
`gpu_heterogeneity`'s clean shape, not `gpu_warp`'s compounding one).
`oro`'s absence changes the formula (not an additive no-op like
warp_x/warp_y) — a dedicated regression test proves the branch is
genuinely wired. `init_gpu_with` gained an automatic storage-buffer-limit
derivation from each kernel's own layout (this kernel needs 9, past
`downlevel_defaults()`'s baseline) — self-contained, existing call sites
unaffected. Real timing: 512²/1024²/2048² at 5.17×/8.13×/4.84× (the
1024²→2048² drop reported honestly, not investigated — possibly memory-
bandwidth-bound at 8 input buffers). `compute_height` (CPU) untouched,
its golden-parity tests unaffected. Also fixed a doc-merge artifact in
`GPU_LAYER_INTEGRATION_SCOPE.md` (milestone 2's own completion note had
been misplaced under milestone 3's heading). See `CHANGELOG.md`'s "GPU
layer integration milestone 3" entry for the full record.

**Milestone 4 — `gauss_blur` + `compute_resistance` on GPU: done
(2026-08-16), genuine three-way JS/CPU/GPU parity.** Unlike milestones
1-3 (all noise-driven, all only GPU-vs-CPU-twin verifiable per
`DECISIONS.md` §7c), neither of these touches noise — verified directly
against the real, untouched `cartalith_terrain::gauss_blur`/
`compute_resistance` (`cartalith-terrain` added as a `cartalith-gpu`
dev-dependency, test-only). `gauss_blur`: max observed divergence
`7.15e-7` at 512×512 across three radius/wrap configs (a direct-sum-in-f32
GPU kernel vs. the CPU's running-sum-in-f64 — the real precision-regime
gap turned out negligible for a bounded linear sum, unlike noise's
chaotic compounding). `compute_resistance`: max divergence `5.96e-8`,
essentially `f32` epsilon. New `GpuBlurContext` (two pipelines — `box_h`/
`box_v` — sharing one device, since `gauss_blur`'s 3-pass structure needs
both kernels reading what the other just wrote). `compute_flexure`
(a thin `gauss_blur`-plus-mask-plus-normalize wrapper) checked, not
ported this pass — noted for whoever wires `gauss_blur` into it.

**Real, honestly-reported timing** — not every kernel wins: `gauss_blur`
20.49× at 2048² (a real win), but `compute_resistance` **loses to CPU at
every size tested, including 2048² (0.38×)** — its formula is too trivial
for GPU dispatch overhead to ever amortize, exactly the case
`HARDWARE_ACCELERATION.md` §6 already warns about. Recorded plainly, not
hidden — not every candidate should actually move to GPU even once it's
technically been verified there.

**Milestone 5 — plate assignment (JFA) on GPU: done (2026-08-16), GPU
beats brute-force exactly.** Confirmed the JFA hypothesis: `assign_plates`
is a textbook Jump Flooding Algorithm, but a specific **in-place-mutation**
variant (a cell can see another cell's update from earlier in the *same*
pass, not just the previous pass's frozen state) — a real algorithm
variant, not an implementation detail. `gpu_jfa_plates.wgsl` implements
the standard **double-buffered** JFA instead (the textbook, race-free GPU
formulation) and doesn't attempt to match the CPU's in-place answer
cell-for-cell — verified against **brute-force exact-nearest-plate ground
truth** instead, per the scope doc's own instruction to investigate which
framing fits rather than assume. Result across three configurations
(512×512 at 14/40 plates, 1024×768 at 22 plates): **GPU JFA matched
ground truth exactly, 0 mismatches, every time.** CPU's in-place JFA had a
tiny, consistent, expected approximation error (1-2 cells out of
262k-786k) against the same truth — a known JFA property, not a bug in
either variant. Also investigated `compute_stress`: confirmed genuinely
harder, not a same-shape sibling — its main loop is a *scatter* (writes to
both a cell and its neighbour in one pass), a real cross-thread write
hazard WGSL's core atomics don't cover, needing a gather reformulation
and its own re-verification. Deferred to its own future milestone, not
bundled in.

**Real timing** (128/512/1024/2048, 24 plates): GPU wins even at 128×128
(1.63×) — the first GPU milestone to win at that size, since JFA's
`log2(size)`-pass structure means real compute work happens even on a
small grid. Scaling to 11.50×/18.22×/15.65× at 512²/1024²/2048² (the last
a real, honestly-reported dip, not investigated). See `CHANGELOG.md`'s
"GPU layer integration milestone 5" entry for the full record.

**Milestone 6 (orogeny sub-investigation) — confirmed poor GPU fit
(2026-08-16).** Orogeny's graph-tracing (`trace_boundaries`/
`tag_boundary_types`/`build_orogeny_field`) is sequential graph
traversal, the same poor-fit category as `compute_stress`'s scatter
hazard and Phase 2's Dijkstra/MST road networks — informational finding,
no kernel built.

**Milestone 6 — first real partial-GPU pipeline integration: done
(2026-08-16), the architecturally significant one.** Every prior
milestone (1-5) built a standalone, never-called kernel — generating a
map has been CPU-only this whole time not because GPU didn't work, but
because nothing wired it into `generate_terrain` itself. This milestone
is that wiring: a new opt-in `WorldParams.use_gpu` flag (default
`false`) runs domain warp, crustal heterogeneity, plate assignment, and
the flexure/base-field blur on GPU inside the real pipeline, with
per-stage CPU fallback on any GPU failure (never a panic) and a new
`WorldState.gpu_stages_used` field so callers can tell which path
actually ran. **Headline result: with the flag at its default `false`,
`generate_terrain`'s output is unchanged** — `cargo test --workspace`
100% green, every existing golden-parity test (this pilot's whole
foundation) unmodified. Closed a real gap along the way: milestones
2/4/5's own dispatch functions were private, unreachable outside
`cartalith-gpu` — four new public wrappers fixed that. **Real end-to-end
timing is the honest, sobering number this milestone adds**: each GPU
wrapper creates its own fresh `GpuContext` per call, so at every size
this pilot ships at by default (128×128 through 1024×1024), the
`use_gpu=true` path is *slower* than CPU (up to ~16× at 128×128),
dominated by ~1.3-1.4s of fixed context-creation overhead that only the
largest tested size (2048×2048) outruns, and only by 19%. Context
reuse/caching across the four stages is flagged as the clear next
optimization, not attempted this pass. See `GPU_LAYER_INTEGRATION_
SCOPE.md`'s milestone 6 "Done." section and `CHANGELOG.md`'s "GPU layer
integration milestone 6" entry for the full numbers.

**Milestone 7 — climate's wind/rain loop on GPU: done (2026-08-17), a
real loss even with milestone 8's own fix applied from the start.**
Built `gpu_weather.wgsl` (`evap_main`/`advect_main`/`deposit_main`) using
the shared-`GpuDevice` pattern from day one (milestone 7 landed after 8,
no reason to repeat 6's original per-call-context mistake). Required a
real refactor first: `simulate_weather`'s previously-inline setup/
teardown extracted into new `pub fn build_weather_grid`/`finish_weather_
grid` (`cartalith-climate`) — pure extraction, `golden_parity_weather.rs`
unchanged. **Correctness**: no noise dependency, verified directly
against the real CPU `simulate_weather` at production `iters=70`: max
abs diff `1.79e-7`, essentially f32 epsilon — 70 iterations of gather/
advect/deposit didn't compound meaningfully (bounded arithmetic, unlike
nested noise). **Real timing, the honest finding**: this kernel's
working set is capped at `min(gw,240)` and stops growing with map
resolution past that — unlike every other GPU-wired stage. Measured at
its real production size (240×240, 70 iters, from a real 2048² map):
**GPU 23.8ms vs CPU 22.2ms, 0.93× — GPU loses**, even with milestone 8's
fix. 210 dispatches (70×3) against a 57,600-cell working set is too
little work to amortize even the remaining per-dispatch overhead once
context-creation stops dominating. Joins `compute_resistance` (milestone
4, 0.38×) as a second confirmed "verified on GPU, shouldn't run there"
case — a different structural reason (dispatch-count-dominated, not
formula-triviality-dominated). **Wired anyway** behind `p.use_gpu` for
architectural consistency (`"weather"` joins `gpu_stages_used`), expected
to keep losing regardless of map size. Found and fixed a real pre-
existing bug along the way: `cartalith-civ`/`cartalith-engine`'s two
`examples/timing_bench.rs` (from the CPU-multithreading milestones)
collided at the same output path, breaking `cargo test --workspace` —
renamed the civ one to `civ_timing_bench.rs`. See `GPU_LAYER_INTEGRATION_
SCOPE.md`'s milestone 7 section and `CHANGELOG.md` for the full record.

Per the scope doc's own feasibility table: graph/sequential algorithms
(flow accumulation, water-body priority-flood, Dijkstra/MST road
networks, orogeny) remain a poor GPU fit without real algorithmic
redesign — not in scope for near-term milestones, which continue to
follow the per-cell-math layers (terrain → climate → erosion's per-cell
parts → Phase 2's per-cell affordance fields → rendering).

**Milestone 8 — GPU context reuse across `generate_terrain`'s stages:
done (2026-08-17).** Picked up milestone 6's own flagged next
optimization directly. New `cartalith-gpu::GpuDevice` (adapter+device+
queue, no pipeline) + `init_gpu_shared_device()`, built once per
`generate_terrain(use_gpu=true)` call and threaded through all five GPU
call sites (warp, heterogeneity, plate assignment, two `gauss_blur`
calls) via new `_with(gpu: &GpuDevice)` pipeline builders and wrapper
functions, instead of each stage independently paying its own ~1.3-1.4s
adapter/device handshake. Confirmed (not assumed) `wgpu::Device`/`Queue`
are cheap `Clone` handles by reading `wgpu` 30.0.0's own source before
relying on it. Original standalone functions byte-untouched — every
milestone 1-6 test still exercises the identical code path. **CPU path
confirmed unchanged**: `cargo test --workspace` 0 failures, every
golden-parity test unmodified. **Real result: GPU now beats CPU
starting at 1024×1024** (128²: 1.44s→813ms, 512²: 1.46s→689ms, 1024²:
2.32s→1.39s and crosses from a 0.78× loss to a **1.14× win**, 2048²:
6.03s→5.92s at ~0.98× — reported honestly as likely single-run
variance rather than a regression, per the benchmark's own "not
averaged" caveat, not re-run to chase a better number). See
`GPU_LAYER_INTEGRATION_SCOPE.md`'s milestone 8 section and
`CHANGELOG.md`'s own entry for the full record.

## Memory optimization (`MEMORY_OPTIMIZATION_SCOPE.md`, done 2026-08-16)

Owner-reported "consumes a ton of memory" on generation, investigated
with real measurement, not assumption. Confirmed dominant contributor:
`ResourcePotentials` (`cartalith-civ`) held six resource fields
(clay/buildstone/flint/obsidian/sulfur/alum, ~96 MB at 2048²) that
nothing in the pipeline reads. Fixed by freeing them immediately after
computation in `compute_civilisation()`. Real before/after at 2048²:
peak 1,445-1,653 MB → 1,434.5-1,501.8 MB, steady-state 689-691 MB →
678.0-679.9 MB, no persistent leak (re-confirmed). A real but modest
win — the bulk of the remaining ~1.1-1.3 GB transient peak above
baseline is `cartalith-terrain`/`-climate`/`-erosion`/`-hydrology`'s own
~96 full-grid allocations, not instrumented stage-by-stage in this
pass; a real candidate for a follow-up if the owner wants the peak
pushed further. Full numbers in `cartalith-native/docs/CHANGELOG.md`.

## CPU multithreading (`CPU_MULTITHREADING_SCOPE.md`, milestone 1 done 2026-08-16)

Owner-reported "doesn't seem to fully use the cpu" (16 logical cores,
generation used effectively one -- confirmed, `rayon` was not a
dependency anywhere in the workspace before this). Unlike GPU work,
needs no `DECISIONS.md` §7a carve-out: parallelizing an existing
per-cell loop preserves golden-parity output exactly, bit-for-bit, not
within a tolerance -- confirmed by every existing test for the touched
functions passing completely unmodified, plus a full `cargo test
--workspace` (0 failures, 0 modified tests).

**Milestone 1 — `cartalith-terrain` (done 2026-08-16).** Added
`rayon = "1"`; parallelized `compute_warp`, `compute_heterogeneity`
(the fbm loop; the trailing reduction stayed sequential, not the
bottleneck), `compute_height`, `compute_resistance`, and `gauss_blur`'s
`box_h`/`box_v`. Real timing (16-core machine, best of 3, seed 12345):
128² 0.0973s→0.0936s (~1.04x), 512² 0.6019s→0.4859s (~1.24x), 1024²
1.8328s→1.3143s (~1.39x), 2048² 7.0670s→5.1071s (~1.38x). Honest,
modest, not near 16x -- Amdahl's law: plate seeding/Lloyd relaxation,
JFA plate assignment, `compute_stress`, `build_age_field`, and all of
climate/erosion/hydrology stay fully sequential this pass and set the
real ceiling measured. Full record and per-function reasoning:
`cartalith-native/docs/CHANGELOG.md`'s "CPU multithreading milestone 1"
entry.

**Milestone 2 — `cartalith-civ` (done 2026-08-17).** Added `rayon` to
`cartalith-civ`; parallelized 16 functions (`build_lithology`,
`build_slope_field`, `build_soil_fertility`, `build_water_access`,
`build_biome_raster`, `build_wetland_mask`, `build_carrying_capacity`,
`build_npp`, `estimate_regional_density_km2`, `build_resource_
potentials`'s 15-field main loop, `apply_resource_scarcity`, `build_
raw_slope_field`, `build_route_corridors`, `build_landmass_quality`'s
final fold, `build_flood_field`, `build_settlement_suitability`,
`build_travel_cost`, `assign_territory`'s inner cell loop). Left
sequential and why: `chamfer_dist`/`jfa_dist` (wavefront/iterative,
not independent), `build_water_bodies`/`label_land_components`/
`build_landmass_quality`'s flood-fill (connected components),
`road_dijkstra`/`build_road_network`/`civ_hierarchical_network_
topology`/`civ_sea_routes`/`civ_consolidate_and_smooth_ways`
(graph/Dijkstra/MST), settlement placement/naming/villages (RNG-order,
not grid-shaped), `fresh_river_order` (delegates to
`cartalith-hydrology`). Golden-parity exact-unchanged: every existing
`cartalith-civ` test passes unmodified, full `cargo test --workspace`
68 suites 0 failures. Real timing (new `cartalith-civ/examples/
civ_timing_bench.rs` -- renamed 2026-08-17 from `timing_bench.rs`,
which collided with `cartalith-engine`'s own example of the same name,
see `CPU_MULTITHREADING_SCOPE.md` -- chaining this crate's own real per-cell pipeline
since `compute_civilisation()` itself is a private `fn` in the
`cdylib`-only `cartalith-godot`, unreachable for direct benchmarking):
128² ~0.99x, 512² ~1.34x, 1024² ~1.52x, 2048² ~1.81x -- better-scaling
than milestone 1's terrain result, since this crate has larger
independent per-cell functions. Combined with milestone 1: a full
`generate_terrain` + civ-layer pass at 2048² goes from ~10.62s
sequential to ~7.07s parallelized. Full record:
`cartalith-native/docs/CHANGELOG.md`'s "CPU multithreading milestone 2"
entry.

**Milestone 3 — `cartalith-climate`/`cartalith-erosion`/`cartalith-
hydrology` (done 2026-08-17).** Read every candidate function fully
before touching it (same discipline as milestones 1-2). Climate: the
deepest pass, most of the crate genuinely parallelizes (`compute_
temperature`, `apply_cryosphere_albedo`, `blur_coarse`, `deflect_flow`,
`build_wind`, `compute_ocean_current`, `ocean_sst_anomaly`, `apply_
ocean_currents`, `apply_climate_moisture_correctors`, `simulate_
weather`'s `iters` loop — parallel within each iteration, sequential
across, confirming `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 7's own
"gather-shaped" finding applies to the CPU path too). Erosion: mixed,
confirmed real hazards — `droplet_kernel` (genuine per-droplet
sequential state) and `stream_power_kernel`'s donor-receiver `iters`
loop (wavefront *within* one iteration) stay fully sequential; `erode_
thermal`/`stream_power_kernel`'s safe pieces (final clamps, receiver
computation, `u_max`/`cc`) parallelized. Hydrology: confirmed mostly
sequential, matching this scope doc's own leading hypothesis — `compute_
flow` (flow accumulation) stays fully sequential exactly as its own
pre-existing doc comment already flagged; the one real win is `build_
channels`'s main per-cell channelization loop. Golden-parity exact-
unchanged across all three crates; full `cargo test --workspace` 0
failures, 0 modified tests. Real timing (`timing_bench`, measured via a
temporary `git worktree` since a concurrent fork's own uncommitted GPU-
weather work lived in the same `cartalith-climate/src/lib.rs` file):
128² ~1.32x, 512² ~1.55x, 1024² ~1.26x, 2048² ~1.09x — unusually
better-scaling at smaller sizes than larger ones for this session's own
results, plausibly climate's coarse weather grid capping the `iters`
loop's own growth while erosion/hydrology's full-resolution passes keep
growing; not chased further. Full record: `CPU_MULTITHREADING_SCOPE.md`'s
own third-pass section and `cartalith-native/docs/CHANGELOG.md`'s "CPU
multithreading milestone 3" entry.

**Remaining, not yet scoped**: the remaining sequential `cartalith-civ`
stages (settlement placement, naming, roads, territory's outer capital
loop, villages) — confirmed genuinely hard (RNG-order/graph-shaped), not
just unattempted. Every crate's own hard-hazard functions (flow
accumulation, priority-flood, scatter-writes, per-particle/per-iteration
wavefronts) are the real remaining ceiling, per this scope doc's own
"Out of scope" section from the first pass.

## LOD/tiling base (`LOD_TILING_BASE_SCOPE.md`, done 2026-08-17)

Owner directive, directly after `TERRAIN_ARCHITECTURE_RESEARCH.md` was
filed as forward-looking research (not current scope -- most of it
assumes a real-time camera/LOD/streaming/painting engine Cartalith
isn't): "LOD and zoom etc might be out of scope for the base, but
they're still goals in this project. The base should be present before
integration." Given three concrete scope options, the owner chose the
middle one -- foundational data structures now, real and unit-tested,
zero integration into the live pipeline.

New crate `cartalith-spatial` (no `gdext` dependency): `TiledField<T>`
(zero-copy tile/region/row/column views over a flat `Vec<T>`, the same
SoA layout `WorldState`/`CivData` already use; `tile_size` is a
constructor parameter, not hardcoded, since no real workload exists yet
to benchmark against), a packed `QuadTree<T>` (`Vec<Node>`, integer
child indices, generic caller-defined aggregate flags, real
bounds-rejection proven by a visited-node counter -- a 64x64/leaf_max-4
tree queried with a 1x1 region visits `< len()/4` nodes, not a brute-force
full traversal), and a generic `DirtyTracker` (per-tile dirty flag +
monotonic version counter, no Cartalith-specific field-dependency
semantics baked in). `serde` round-trip tested on all three. 24 real
unit tests (not compile-only), `cargo build/test/clippy -p
cartalith-spatial` clean, full workspace `cargo test` clean (one
`cartalith-engine` GPU-determinism test flake reproduced the
already-documented pre-existing GPU-driver flakiness under parallel
scheduling, unrelated -- passed on isolation and on a clean re-run).

**Confirmed nothing else in the workspace references this crate** --
`cartalith-engine`/`-terrain`/`-climate`/`-erosion`/`-hydrology`/`-civ`/
`-godot` and every `.gd`/`.tscn` file are untouched. Exists purely as a
tested foundation for whenever Phase 3 (3D) or a real large-world need
actually triggers LOD/tiling integration -- not wired to anything today.
Full record: `cartalith-native/docs/CHANGELOG.md`'s "New crate
cartalith-spatial" entry.

## Province boundary legibility (fixed 2026-08-17)

The province-boundary overlay (`build_province_boundary_texture`, wired
same-day as milestone 16's own follow-up) was flagged as a known
legibility issue: functionally correct data, but a literal 1px-wide line
at full grid resolution became sub-pixel and near-invisible once
downscaled to the viewport. Fixed with symmetric boundary detection plus
a one-cell dilation for a real ~3px stroke and a modest alpha bump
(not to fully opaque). Real screenshot-verified (seed 12345, Classic,
512×512, both territory and province layers on): boundaries now read as
clean, bold lines at normal view, clearly distinct from roads. See
`CHANGELOG.md`'s "Fix: province boundary lines were illegible at normal
zoom" entry.

## Known-open items (not owner-blocked, just not done yet)

- Real Fira Sans/Fira Code font files for the UI theme (design-system match found the pairing; sourcing + OFL-license verification deferred).

## Owner-only items

- None currently open. Criterion 4 (real Android device build/install/launch/golden-path) fully closed 2026-08-17 once the owner unlocked the connected phone mid-session — see `ANDROID_BUILD_SCOPE.md`.
- This session has real Windows desktop + `godot4` CLI access + real Android device access, which closes most of what earlier sessions couldn't do themselves.

# Status quicklist

A living checklist, not a narrative — read this first, before `CHANGELOG.md`,
to know what's done vs. open without re-reading the whole history each
session. Update it in the same commit as whatever changes its answer.
`CHANGELOG.md` stays the detailed record of *how*; this is only *what/done?*.

Last updated: 2026-08-17 (post real Android device pass — MVP criterion 4 fully closed —, sea routes (Phase 2 milestone 13) wired into `cartalith-godot`'s rendering with a real render-loop crash found and fixed along the way, CPU-multithreading milestones 2-3 — `cartalith-civ` then `cartalith-climate`/`cartalith-erosion`/`cartalith-hydrology` Rayon-parallelized — Phase 1's two closeout items (credits screen, crate license audit) both done, GPU layer integration milestones 7-8 — GPU-backed weather simulation, shared GpuContext across `generate_terrain`'s stages — a new standalone `cartalith-spatial` crate (tiling/quadtree/dirty-tracking base for a future LOD integration — real, tested, referenced by nothing yet), Phase 2 milestone 16 (`_civGenerateProvinces` — resolved the milestone-9 territory-input blocker via milestone 10's own `assign_territory`, data/backend done and verified, rendering wired as a boundary-line overlay in a same-day follow-up), and Phase 2 milestone 17 (economy/Journey Planner investigated for real — two separate large subsystems found, not one; the ~70-function Journey Planner confirmed to genuinely need its own sub-phase per `ROADMAP.md`, not attempted; `civ_resource_trade_balance` ported/tested from the smaller economy layer — **now genuinely wired**, same day: `civ_world_mean_resources`/`civ_place_resource_context` give it real per-settlement inputs, `get_trade_balances()` exposes the result to Godot, and the memory-optimization tension (needs all 15 resource keys, six were being freed early) resolved by moving that free to after settlements are placed — full reasoning in `ECONOMY_SCOPE.md`), Phase 2 milestone 18 (culture beyond naming, investigated — confirmed one real computation exists beyond the already-ported syllable tables, `_civCultureTerrainFit`/`civ_culture_terrain_fit`, ported and tested but not yet wired since its real inputs depend on the still-unstarted `_civFactionAggregates` territory aggregation; Government/Religion/Ag-tech confirmed genuinely UI-only with zero derived computation; a completely unrelated "culture profiles" system found at reference lines 28193+ correctly identified as Phase 5 Urban Morphology scope, not Phase 2), Phase 2 milestone 19 (Journey Planner milestone 1 — the two fully self-contained categories of its ~70 functions ported: physical-modeling primitives and the reference's own "four deferred items" seasonal/closure cluster, 22 tests, full remaining milestone breakdown in new `JOURNEY_PLANNER_SCOPE.md`), Phase 3 milestone 1 (`TerrainAppearance` abstraction in `render.rs` — colour data now owned/structured, pixel-identical output verified, real audit finding that no elevation-breakpoint ramp exists in this renderer), Phase 3 milestone 2 (multidirectional hillshade + ambient occlusion — the first pass where the default render visibly improves; JS golden parity kept exact via a new `js_reference()` appearance rather than re-baselining, min-luma identical before/after so no black valleys, ~free at 45 ms/512²), Phase 3 milestones 3-4 (hydrology tint; then the atlas look — paper/vellum ground, forest stippling, physical plate border — closing three of `VISION.md`'s four remaining atlas elements, with the `js_reference()` gating extended by three more early-returning zeros so `golden_parity_render.rs` stays completely unmodified, and with the cross-world result *inverting* milestones 2-3: stronger on low-relief Archipelago than on mountainous Classic, because the paper acts on the whole sheet), and the GUI shell redesign milestone 1 (`GUI_SHELL_SCOPE.md` — full 6-region professional-editor shell rebuilt in `main.tscn`/`main.gd` from an owner-supplied design import, zero Rust changes, every real control re-parented and screenshot-verified working end-to-end, every not-yet-real feature visibly present but honestly disabled), and the causal-chain explainer (`VISION.md` sequencing item 1 — hovering a settlement shows a real "WHY HERE?" decomposition of `build_settlement_suitability`'s own thirteen weighted terms; proved faithful by a test that reconstructs the real function's output at every cell from the explanation alone, and cross-checked against real terrain across all 40 settlements of a generated world with 0 violations; deliberately per-settlement rather than a general `explain_cell(x,y)`, since the source rasters aren't retained on `CivData` and holding them would undo the memory work), and Journey Planner milestone 2 (transport mode selection — 6 of 10 originally-listed functions shipped, given caller-supplied stage lists; the other 4 confirmed by reading the real reference code to depend on milestone 5's unbuilt route derivation or milestone 3's unbuilt `jpCalcLand`, re-flagged rather than forced; the biome-mapping question this doc worried about turned out to already be answered by the reference's own `jpLegacyBiomeOf`, ported as `jp_biome_key` rather than invented; 15 new tests, `JOURNEY_PLANNER_SCOPE.md` updated), and the GUI decluttering pass (`GUI_SHELL_SCOPE.md` — a design-lead-researched target IA implemented for real: `INFRASTRUCTURE`→`EXPLORE`, `CARTOGRAPHY:Layers` consolidated into the one real `LayersPanel` surface freeing a slot for `Paint`, `WORLD:Resources`→`Sculpt`, CIVILIZATION/CARTOGRAPHY subjects renamed to the reference's real buckets, the invented `GenerateMenu` 11-stage pipeline replaced with the reference's real Step 1→2→3 sequence, a real dark `Theme` resource replacing the light-parchment `SettingsCard` panels that had been sitting on the dark shell, a real `FooterVBox`-visibility bug fixed, before/after windowed screenshots confirming the full golden path unbroken) — see `CHANGELOG.md`), and Journey Planner milestone 3 (physical travel cost — 7 functions shipped including the v1.97 sail polar, the season×biome weather blend and the whole day-wage cost model; 2 of the 11 listed had already shipped with JP milestone 2; the remaining 2 (`jp_calc_land`/`jp_calc_water`) exposed a real dependency-ordering error in `JOURNEY_PLANNER_SCOPE.md` — they need milestone 4's consumption/resupply cluster, which that doc orders *after* them — so they are deferred and the doc is corrected rather than the dependency stubbed; the flagged `JP_BIOMES[...].weather` table confirmed unported and ported here; `jp_journey_cost` confirmed to need no milestone-5 plan object; milestone 2's four deferrals re-read and none resolved; golden-verified via a bare-`vm` Node run of the reference's own source lines, 12 new tests). **Phase 4 started** (`ASSET_LIBRARY_SCOPE.md`, new): the Asset Library investigated for real against the reference rather than its pre-implementation design docs — an asset is one PNG bound to one slot in a frozen ordered vocabulary (8 families), an asset pack is a real PKZIP+`pack.json`/`pack.csv` serialization format, a second `assetlib/library.json` project-embedded format also exists, and the renderer genuinely draws pack sprites with the vector glyphs as fallback; ~2,250+ lines total but only ~600-800 of them portable, so a real sub-phase of seven milestones. Milestone 1 done: new standalone `cartalith-assets` crate (pack manifest model/parse/validate/serialize, 28 tests, golden-verified against the real `parsePackCsv`/`parsePackManifest`/`packSummary`), wired to nothing. **Milestone 2 done**: pack `.zip` read/write, placed in `cartalith-assets::archive` behind an on-by-default `zip` feature after reading `cartalith-io` and finding nothing to share (its whole zip surface is three `zip`-crate calls) plus two reasons not to put it there (reading-only by explicit scope; the dependency would point the wrong way); what is actually ported is the reference's export *policy* — `.png` STORED, timestamps frozen at 1980-01-01 so exports are byte-reproducible, `pack.json` last, names verbatim — and it is verified **in both directions** against a pack the reference's own `PackManifestBuilder.build()` + `zipStore()` produced headlessly, including feeding this port's own output back through the reference's `unzipAny`/`parsePackManifest` (identical payloads, `pack.json`, summary and warnings; the two archives differ by 2 bytes total), 14 new tests, still wired to nothing. Milestone 3 done: scatter rules (`cartalith-assets::scatter` — the `ScatterRule` model, ten slot presets, keyed rule table, weighted variant selection, hardened normalizer), with the three v1.27 hardening fixes **re-derived for Rust rather than transcribed**: the `NaN`-density carpet is still reachable here but by the *opposite* IEEE rule (`f64::min` absorbs NaN where `Math.min` propagates it), the `NaN`-spacing bucket-grid collapse is real and `f64::max` would have masked it, and the `Object.assign` aliasing bug is structurally unreachable — not from ownership but because defaults and untrusted input are different *types* here, so no defensive code was written for it; plus a guarantee the reference cannot have (`Serialize` but deliberately no `Deserialize`, so the hardening cannot be bypassed). Golden-verified: `pick_weighted_variant` diffed exactly over 11 cases × 36 positions, and 37 normalizer fixtures caught a real first-run bug — `density`'s fallback is not symmetric with the other numeric fields (absent keeps the preset, *rejected* lands on a literal 1). 24 new tests; three corrections to milestone 4 recorded (it is not the first cross-crate dependency — this is; `pickIconVariant`/`spaceOf` shipped here; `biomes` is `Vec<f64>` because `Number.isFinite` does not coerce). **Milestone 4 done**: rule-driven icon placement (`cartalith-assets::placement` — `place_map_icons_ruled`/`icon_slot_for_item`/`sprite_draw_rect`), the first real placement golden-parity surface (positional and seeded, diffs exactly); both of milestone 4's own v1.27 fixes (most-specific-first priority sort, `requireWetland` ANDed with the biome test) confirmed **structurally necessary in Rust**, unlike one of milestone 3's three, and proven with a hand-traceable `tGap=1` fixture where the winner is shown independent of rule-insertion order; 23 new tests (12 unit + 11 golden), still wired to nothing. **GPU layer integration milestone 9** (flow accumulation — the first genuinely sequential algorithm in this pipeline redesigned for GPU rather than ported: per-cell D8 flow direction plus pointer-doubling subtree sums in `ceil(log2(n))` rounds, `atomic<u32>` fixed point for order-independent bit-reproducible accumulation; bit-exact against the real `compute_flow` for area seeding and 1.3e-4/3.3e-4 relative at and above the channel threshold for discharge seeding; **measured through to the civilisation layer — river network and settlement positions both come out identical, 104/104 and 125/125 seeds, zero moved**; 15.5× on the kernel at 2048² and the end-to-end `generate_terrain` ratio moving 0.98×→1.74× there; plus two honest "shouldn't run on GPU" findings for the water-body depression fill and `road_dijkstra`).

## GUI shell (`GUI_SHELL_SCOPE.md`, milestone 1 done 2026-08-17; decluttering pass done 2026-08-17)

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
terrain appearance's actual editing GUI.

**Decluttering pass done** (design-lead-researched target IA, implemented
in full): `INFRASTRUCTURE` (zero reference grounding) → `EXPLORE` (the
reference's real second mode); `CARTOGRAPHY:Layers` nav subject removed
(consolidated into the always-visible `LayersPanel`, the one real layer
surface), freeing a slot for `Paint`; `WORLD:Resources` → `WORLD:Sculpt`;
CIVILIZATION/CARTOGRAPHY subjects renamed to the reference's real buckets;
18-of-20 placeholder subjects now carry specific, reference-grounded honest
text instead of one generic string. Top bar: invented `New world.../Save
project` deleted, `GenerateMenu`'s fabricated 11-stage pipeline replaced
with the reference's real Step 1→2→3 sequence, `SimulateMenu`/`MapMenu`/
`ViewMenu` renamed, `AssetsMenu` converted `MenuButton`→`Button`, a
`ThemeToggleButton` added (disabled — light theme itself still deferred).
Real bug fixed: `FooterVBox` was visible on all 20 nav subjects instead of
`WORLD:Overview` alone. A real dark `Theme` resource
(`theme/dark_theme.tres`) now covers every control including SpinBox/
OptionButton/CheckBox, retiring `app_theme.tres` (the MVP's light-parchment
theme) from the live path; the three light-parchment `SettingsCard` panels
sitting on the dark shell — the single most visible inconsistency in the
prior shell — are gone, flattened into plain sections with one
`FoldableContainer` for Advanced Features. `CreditsDialog` explicitly
themed (Window nodes don't inherit Control-tree themes); map-overlay hover
card recolored dark. Real before/after windowed screenshots (the *before*
shot from genuinely running the old shell via `git stash`, not memory);
full golden path — generate/overlay toggles/causal-chain hover inspector/
load-save/credits — reconfirmed working through the restructured shell.
Full record: `CHANGELOG.md`'s "GUI decluttering pass" entry,
`GUI_SHELL_SCOPE.md`'s own dedicated section.

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
| 2 — Civilisation layer | **Started 2026-08-16, milestones 1–15 of an unknown-but-large number done** (milestone 10, territory/border generation, has an owner decision recorded — `DECISIONS.md` §7b, cost-distance Voronoi from capitals, strength-weighted — implementation status tracked separately, not this row's concern to restate). `cartalith-civ` crate (zero `gdext` dependency), every field golden-verified against the real reference engine. **1** lithology/soil fertility/water access. **2** water-body classification (ocean/lake, priority-flood depression fill). **3** biome classification (12 climate categories). **4** carrying capacity/NPP/population density. **5** resource potentials (15 geological fields). **6** route corridors/landmass quality/coast SDF. **7** `buildSettlementSuitability`/`findSettlementSeeds` — the "v1.30 one function" `ROADMAP.md` originally named as this phase's landmark, reached and golden-verified. **8** settlement placement + faction assignment — the pure core of `_civIterativeAutoWorld` (land-component labelling, snap-to-land/coast, `_civAssignLandmassFactions`'s capacity-weighted seat apportionment + multi-capital spacing, settlement tier classification), stopping deliberately before the DOM-coupled orchestration shell. **9** settlement population + naming — `_civBasePopForKind`/`_civSettleName` (RNG-driven, reuses `cartalith-rng`'s already-verified `mulberry32` — `_civRng` is the same algorithm under a different seed wrapper, proved by hand not assumed). A genuine, verified reference quirk found here: `state.seed` (distinct from the real per-world `state.tect.seed`) is never assigned anywhere in the reference, so the civ-naming RNG stream is seeded identically for every world regardless of its actual seed — same-rank, same-faction settlements across *different* worlds get identical generated names, a real mechanical consequence, not a bug. Full history, every real bug/gap found (a Node-harness seeding bug, a stale-vs-fresh river-network mismatch, a threshold ambiguity between two real reference call sites, several `WorldState`-retention fixes, a 4-vs-8-connectivity flood-fill distinction, a 4-script-block harness miscount, a snapped-position-vs-original-seed-score `.suit` mixup), and reasoning is in `CHANGELOG.md`'s "Phase 2 milestone 1–9" entries — this row stays a summary, not a repeat of it. **11** road network algorithm — `buildTravelCost`/`roadDijkstra`/`buildRoadNetwork` (a distinct `f64`-priority heap from milestone 2's, per the reference's own v1.89 perf comment; real terrain data exercised the "unreachable landmass" branch, not just a synthetic test). Landed in `cartalith-civ` (a deliberate placement decision, not a default — the functions live in the reference's block 1, weighed against `ARCHITECTURE.md`'s "civ" framing and decided the latter wins). **Investigated for milestone 12, found a real correction to the earlier assumption**: this port's own `_civSeedVillages` dependency reasoning (milestones 8/9's "villages need roads" note) pointed at the wrong system — `buildRoadNetwork` only ever serves the *manual* "Generate Roads" tool (`buildRoadsOp`, reads user-clicked `state.places`); the civ auto-populate flow's own road network (`civWays`) is built by a separate, larger algorithm (`_civHierarchicalNetwork` + `_civMstRoutes` + `_civPreferSeaRoutes`) not yet read or ported. **Milestone 11 does not unblock village seeding** — it's real, useful, tested code for a different (manual-tool) purpose. **12** the real civ-auto-populate road network — `civ_hierarchical_network_topology` (`_civHierarchicalNetwork`'s three real passes: Prim MST, min-degree-fill by settlement tier, Floyd-Warshall shortcut-detour-relief — confirmed a third pass beyond what milestone 11's own scoping estimated). This is the real `_civSeedVillages` dependency milestones 8/9/11 all pointed at without reaching. Split deliberately: ships the raw topology (what road-proximity queries actually need), defers corridor-consolidation/Catmull-Rom-smoothing/road-classification (needs `_civSmoothPath`, not yet ported — milestone 14) since that's presentation polish, not the graph structure itself. Both golden fixtures are real edge cases, not synthetic: one settlement genuinely unreachable in case0; case1's min-degree-fill hitting its natural ceiling (a complete K5 graph) rather than its per-tier target. A real `river_flow_thresh` parameter bug (hardcoded map width instead of the real per-world value) caught and fixed before it shipped. **10** territory assignment — cost-distance Voronoi from capitals, weighted by capital population (`DECISIONS.md` §7b's own design, implemented as designed). The first Phase 2 milestone with no JS reference to port at all — the reference has zero algorithmic territory generation (paint tool + save/load only) — so verification is 8 unit tests standing in for a golden test (equal-population capitals split at the geometric midpoint; a 100k-vs-5k-population pair moves that midpoint to the larger capital, the actual weighting behaviour measured, not just present; unreachable cells stay unowned; a two-capital faction's territory unions both zones). `pop_ref=15000.0` (== a capital's base population before variance) is a documented, non-arbitrary constant. **Rendered 2026-08-16**: `cartalith-godot`'s `build_territory_texture()` turns the per-cell `Vec<i32>` into a low-alpha (`~0.32`) RGBA8 overlay texture, Okabe-Ito-coloured by faction, toggleable via a new default-OFF "Show territory" checkbox — see this row's own catch-up note below and `CHANGELOG.md`'s "UI/UX catch-up: territory + villages" entry. **15** village seeding — `_civSeedVillages`/`_civVillageAcceptProb`/a milestone-12-topology-adapted `_civRoadProximityQuery`, the feature milestones 8/9/11/12 were all working toward unblocking. Closed a real RNG-sharing gap first: `name_and_populate_settlements_with_rng` now threads an external `Mulberry32` (purely additive alongside the existing zero-arg function) so village seeding continues the exact same stream naming left off at, matching the reference's own one-shared-`rng`-closure design. Golden-verified against the real reference (fully synthetic but reference-function-verified inputs, same standard milestone 12 already set) — bit-exact first attempt, including RNG-derived village names and nearest-capital faction inheritance; a second targeted extraction independently confirmed the downsampled-routing-grid-to-full-grid coordinate conversion by matching a hand-calculated `exp()` distance formula to 15 significant figures. **Flagged, not fixed here**: milestones 7-9's existing golden tests seed their candidate lists at a threshold (`0.65`) that traces to a *different* real call site (the standalone JSON-export default) than `_civIterativeAutoWorld`'s own real default (`SETTLE_SEED_THRESH=0.42`, confirmed by tracing why a headless harness's `wantCounts` is always falsy) — not a bug in those milestones' own pure-function correctness, but a pipeline-orchestration question for whatever in `cartalith-godot` builds the real base-settlement candidate list. **Wired 2026-08-16**: `cartalith-godot`'s `compute_civilisation()` now calls `civ_seed_villages` after base settlement naming, sharing the one `Mulberry32` stream this milestone's own doc comment requires (no second, desynced RNG instance), and merges the output `Hamlet`-tier settlements into the same list the UI already draws. The gating question is resolved the way flagged here: a new default-OFF `VillagesCheck` toggle in `cartalith-godot`, matching the reference's own real `_civVillages` default. **14** corridor consolidation + path smoothing — `civ_consolidate_and_smooth_ways`, milestone 12's own deferred tail (reference `_civHierarchicalNetwork` lines ~21670-21739): claims corridor cells busiest-edge-first so shared trunk segments render once, classifies each way by peak usage (`highway`/`regional`/`road`/`track`), auto-names it from its endpoint settlements, and Catmull-Rom-smooths the result (RDP-simplify then chord-length-parameterized spline sampling, both ported fresh from the reference's own `rdpSimplify`/`catmullRomSample`, reference lines 8701/8790) with a terrain-validity repair pass and an endpoint-snap pass so strokes land on their settlement pins. Also ports `_civSmoothPath`/`_civTerrainValidTest`/`_civNearestValidPt` (reference lines 21892/21843/21872), narrowed here to the one call shape this network uses (land-only validity) — the general terrain-validity test also has an ocean-only mode, generalized in by milestone 13's sea routes (this row's own **13** entry above). Golden-verified against two real cases reusing milestone 12's and milestone 9's own already-verified fixtures (no new settlement/topology data invented): a genuine short-segment Catmull-Rom oversampling quirk (a 2-cell path produces a 3-point output whose rounded midpoint coincides with its own start point) and a real K5 corridor-sharing case (10 edges, a mix of visible and fully-consolidated hidden ways). **13** sea routes — `civ_sea_routes` (`_civMstRoutes(ports,true)`, reference line 21240, `isSea` branch only — the `isSea=false` land branch has no confirmed real caller, `_civHierarchicalNetwork`/milestone 12 is what the real land network uses). Shares `_civSmoothPath`/Dijkstra/Prim's-MST shape with milestone 12 but is a genuinely separately-scoped algorithm: the cost grid marks land `Infinity` (impassable, not merely expensive — the reference's own fix-history comment explains a finite land cost let Dijkstra cut across jagged coastline pixels, and smoothing then exaggerated those cuts into visible loops), ports snap to the nearest navigable-ocean cell at radius 10 (deliberately wider than milestone 12/14's radius 6 on a different cost grid), and a v0.73 sea-lane augmentation pass adds each port's nearest reachable port as a direct lane (capped at 1.15x the MST's own longest hop) beyond the bare tree. `_civSeaTimeEdgeCost` (v1.98 current/wind-costed sea-lane pricing) deliberately not ported — its real inputs (ocean-current/wind u/v fields) aren't retained on `WorldState` past their internal use in `apply_ocean_currents`/`deflect_flow`, so this port takes the reference's own documented graceful-degradation fallback (uniform arithmetic cost) rather than adding new plumbing outside this milestone's scope — a real, flagged follow-up. Four existing helpers generalized (not duplicated) to support both land and ocean validity modes: `civ_snap_finite` (added a `max_r` parameter), `civ_is_valid_land`→`civ_is_valid_terrain` (added the `_civTerrainValidTest('ocean')` branch this row's milestone-14 note flagged as unported), `civ_nearest_valid_pt`/`civ_smooth_path` (both threaded the same `is_sea` flag through). Golden-verified against the real reference: a fresh Node harness caught and fixed a real bug in itself before trusting extraction (`generate()` is `async`, and a bare unawaited call left `field` at its default-zero fill, `currentWaterBodies()` reporting 100% ocean — fixed by awaiting properly, then cross-checked `field[0]` plus land/ocean/lake cell counts against already-trusted fixtures). Reused milestone 14's own case0/case1 fixtures (already-verified coastal settlements over genuine mixed land/ocean/lake geography at both grids) — both cases matched the Rust port's output exactly on the first run, including a real reference quirk where two of case1's four routes carry `km:0` despite having real points (`_civSmoothPath` accumulates `km` over rounded sample points before its own final step restores full-precision endpoints, so a short diagonal hop's only interior sample can round to coincide with the pre-restore rounded start point). **17** economy/Journey Planner investigated for real (2026-08-17), full reasoning in `ECONOMY_SCOPE.md` — two separate, both genuinely large subsystems turned out to exist under "economy": the Journey Planner (`jp*`/`_jp*`, reference lines ~17300-20400, ~70 functions covering transport-mode selection, physical travel cost, consumption/resupply, seasonal closures, multi-stage route derivation) confirms `ROADMAP.md`'s own "consider it a sub-phase" warning as accurate, comparable in size to this port's entire civ-layer effort to date — not attempted. The faction/settlement economy layer (`_civFactionAggregates`, ~165 lines; `_civPlaceTrade` and its dependency cluster) is smaller but still real, explicitly "NOT new simulation" per the reference's own header comment (a display/aggregation layer over already-computed state). `civ_resource_trade_balance` (`_civResourceTradeBalance`, reference line 24175, v1.33's unification of two drifted copies) ported and tested — the one fully self-contained piece, operating on caller-supplied catchment/world resource means with no new upstream dependency. Seven real unit tests (no golden harness needed — small, pure, branch-complete, no RNG/iteration-order risk, same precedent as territory/provinces). A real, disclosed tension found and left unresolved: the full trade layer needs all 15 `CIV_RESOURCE_KEYS` resident, but the memory-optimization pass (commit `62b9b51`) frees 6 of them after use — flagged for whoever ports the next slice. Not wired anywhere yet — no real caller exists until the broader orchestration is built, the same "don't wire in what nothing calls" discipline milestone 9's own territory note established. **Journey Planner sub-phase** (`JOURNEY_PLANNER_SCOPE.md`, the ~70-function subsystem milestone 17 confirmed genuinely needs its own sub-phase): milestones 1-3 of 6 done, nothing wired to any caller by design — it is real interactive per-journey tooling, a future GUI feature, not something auto-computed for every settlement pair. **JP-1** physical-modeling primitives plus the reference's own "four deferred items" seasonal/closure cluster (22 tests). **JP-2** transport mode selection — 6 of 10 listed functions shipped given caller-supplied stage lists, the other 4 confirmed by reading the real code to depend on unbuilt milestones; the biome-mapping question that doc worried about turned out already answered by the reference's own `jpLegacyBiomeOf`, ported as `jp_biome_key` rather than invented (15 tests). **JP-3** physical travel cost — 7 shipped (`jp_train_pace`, `jp_sail_factor` (v1.97's rig-class sail polar), `jp_wx_weighted`/`jp_weather_factor` (season×biome weather blend), `jp_column_length_km`/`jp_column_factor` (v1.51's road-capacity damping), `jp_journey_cost` (the whole day-wage cost model)), 2 of the 11 listed had already shipped with JP-2, and **the last 2 exposed a real ordering error in the scope doc itself**: `jp_calc_land`/`jp_calc_water` depend on milestone *4*'s consumption/resupply cluster (`jpCapacity`/`jpForaging`/`jpAssessResupply`/`_jpDesertTierForGap`), which that doc orders *after* them — so JP-4 must land first, and the doc is corrected rather than the dependency stubbed. Three flagged questions all answered by checking rather than assuming: `JP_BIOMES[...].weather` was indeed unported (JP-2 had deliberately narrowed its `JP_BIOMES` port) and is ported here; `jp_journey_cost` needs no milestone-5 plan object and is ported; JP-2's four deferrals were re-read and **none** resolved. First JP milestone to use a real golden harness rather than pure unit tests — the weather blend is a 48-cell five-term float sum where hand arithmetic would be the weak link, so the reference's own source lines were sliced out and run in a bare Node `vm.runInContext` with no DOM, and all 48 `jpWxWeighted` biome×season cells are verified as a block (12 tests).

**Reached**: settlements with real names/populations/faction ownership, faction territory ownership per cell (wired and rendered), the real auto-populate road topology (12) consolidated, classified, and Catmull-Rom-smoothed (14) — **now wired into `cartalith-godot`'s `compute_civilisation()` and rendered as the map's actual road layer**, replacing both milestone 11's manual-tool stand-in and milestone 12's raw unsmoothed topology (fixed same-day as a third UI/UX catch-up pass — see below), and village seeding (15, wired and rendered, and now reading the real milestone-12 network for its own road-proximity check too, not the old stand-in), plus sea routes (13, `civ_sea_routes`, golden-verified, and now wired into `cartalith-godot`'s rendering too — dashed-style, distinct from land roads, see this row's own **13** entry above and `CHANGELOG.md`'s "Wire sea routes" entry). plus provinces (16, `civ_generate_provinces`, resolved a blocker recorded since milestone 9 once milestone 10's own `assign_territory` turned out to produce `civTerritory`'s exact needed shape — data wired into `cartalith-godot`, `get_provinces()`/`build_province_boundary_texture()` real and verified against live generated data, **and now rendered too** — a `ProvinceBoundaryView` overlay + `ProvinceLayerCheck` toggle, thin boundary lines layered on top of territory's own fill, sidestepping the unbounded-province-count palette problem entirely since a boundary line needs no palette; see `CHANGELOG.md`'s "UI/UX catch-up: render province boundaries" entry, including the direct headless pixel-count verification used after a static screenshot proved inconclusive for a 1px line). **Not reached**: culture (beyond naming flavour), economy, and the Journey Planner as a usable whole — its milestones 1-3 of 6 are ported and tested but deliberately unwired, and milestones 4-6 (consumption/resupply, route/stage derivation, verdict/reporting) are untouched. See `PHASE2_SCOPE.md` for the living milestone list. **UI/UX caught up 2026-08-16** (owner request: "with every milestone and phase the GUI and UX should be updated as well... use a separate agent", a continuous per-milestone practice) — **first pass**: settlements + the milestone-11 road network render on the map. **Second pass**: territory (10) and villages (15) wired and rendered (low-alpha faction-colour territory overlay, default OFF; villages merged into the settlement marker list, default OFF). **Third pass, same day**: found `compute_civilisation()` was still building its road data from milestone 11's manual-tool stand-in — not even milestone 12's own raw topology, a deeper gap than "just wire in milestone 14's smoothing." Fixed the real chain (`civ_hierarchical_network_topology` → `civ_consolidate_and_smooth_ways`, reordered so the smoothing/naming step runs *after* settlement naming, since it needs named endpoints); `get_roads()` now returns classified `Way` data (`points`/`brks`/`way_type`/`name`) instead of raw cell-index paths, `map_overlay.gd` gained a distinct continuous-coordinate `_point_to_screen` (settlement markers still use the cell-centering `_cell_to_screen` — using the wrong one for roads would have shifted every line half a cell) and break-aware polyline drawing so real internal gaps in a consolidated way don't render as a phantom straight line across them. Road width now varies by classification. Screenshot-verified: roads changed from straight/jagged MST approximations to visibly smooth curves following terrain. See `CHANGELOG.md`'s "UI/UX catch-up: wire milestone 14's smoothed roads into the map" entry. **Fourth pass, 2026-08-17**: sea routes (13) wired end-to-end — `CivData.sea_routes`, `get_sea_routes()`, `map_overlay.gd`'s dashed navy-underlay/light-dash rendering (reference's own line-~15511 convention). Real screenshot verification caught a genuine crash (an infinite-loop/buffer-overflow bug in the dashed-line draw routine, triggered by float drift over a long route) before it could ship — fixed and re-verified against the exact config that crashed. See `CHANGELOG.md`'s "Wire sea routes" entry. |
| 3 — Rendering and 3D | **Started 2026-08-17, milestones 1-4 done** (`TERRAIN_APPEARANCE_SCOPE.md`, owner-supplied `TERRAIN_APPEARANCE_RESEARCH.md`). **Milestone 4 (the atlas look)**: three of the four elements `VISION.md`'s sequencing item 2 still listed as ahead — a **paper/vellum ground** applied in `cell_color` after *both* the land and sea branches (an ocean not on the same sheet as the land makes the map read as terrain art pasted onto parchment), composed of a parchment tint divided by its own Rec.709 luma plus `paper_wash`, a pull toward a paper-coloured grey *of the same luminance*, so both parts are luminance-preserving and only chroma moves; **forest stippling** weighted by `material_weights`' own `canopy` fraction (real data, not decorative noise), `smoothstep`-gated and zero-mean so canopy gains texture without net darkening; and a **physical plate border** (paper margin, thick + thin neatline, ink density varied along the rule). None touches `material_weights` or the palettes. **Golden parity: same mechanism extended, not replaced** — `js_reference()` gains three more `0.0`s and each stage early-returns on its own zero (a dedicated branch, exactly as `relief_lights <= 1` established), so `golden_parity_render.rs` stays **completely unmodified**, both tests at their original `1e-4`. **Two corrections caught by looking, not by diff statistics** (milestone 3's lesson holding a second time): the parchment tint alone was only a hue rotation and read far too weakly until the chroma wash was added, and the first stipple field read as a regular diagonal halftone screen — §30's "random texture noise", the same class of regression as milestone 2's AO speckle — fixed by rotating the sampling lattice ~34°, domain-warping it, and flooring mark size at 4 cells. Anti-list numbers, terrain only (2048², frame band excluded): interior mean luma 132.8→**133.0** (Classic) and 106.3→**106.2** (Archipelago), contrast *rises* (sd 31.32→31.89, 27.66→28.30) so nothing is washed out, luma min drops just 1.4/0.8 levels from grain (no black valleys), terrain clipping unchanged. **Cross-world result inverts milestones 2 and 3**: those were strong on mountainous Classic and near-invisible on Archipelago; this one is stronger on Archipelago (−26% chroma vs Classic's −13%, its bright cyan sea becoming a muted teal-grey) because the paper acts on the whole sheet and that world is mostly ocean — and the two worlds converge from 18% apart in chroma to within 0.01 (51.960 vs 51.963), not by clamping but because a shared printing medium is what converges differently coloured subjects. **Not free**, unlike milestone 2: 2048² render 598→915 ms (Classic), 295→597 ms (Archipelago), four extra `vnoise` calls per pixel including ocean — accepted as a one-shot generate-time cost, and recorded as the first thing to optimize if the render ever needs to be fast. **Known limitation flagged, then fixed in a same-day follow-up**: `lib.rs`'s river channel tint and `map_overlay.gd`'s settlement markers both drew over the finished raster and knew nothing about the frame, so an edge settlement's marker landed partly on the plate margin. Resolved (see the milestone-4 follow-up entry in `CHANGELOG.md`) — and it was **four** systems, not two, the territory wash and province boundary lines having the same bug. `render.rs` now exports the frame geometry (`border_width_cells`/`border_cover`, plus `WorldGen::get_border_inset_frac()` as a fraction of texture width); the three Rust rasters fade by `1 - border_cover` and `map_overlay.gd` scissors to the plate interior. Insetting the overlay coordinate space was considered and rejected as the wrong shape for this frame: `apply_border` composites *over* the outermost cells rather than shrinking the map into a margin, so the terrain under the margin is covered, not moved, and inset markers would be displaced from the coastline they sit on. Instead linear features are clipped at the neatline (a road genuinely continues off the sheet) while point symbols are placed or omitted, never sliced. Margin overlay ink at 2048²/seed 12345/Classic: 268 px marker orange and 67 px river cyan before, 0 and 0 after, with all before/after difference confined to the frame band. Verified: `cargo test --workspace` 383/0, clippy clean for this milestone's files, headless load clean, real windowed app screenshotted at 2048² for **both** worlds, with the controlled before/after coming from `appearance_ab_dump.rs` extended with `noatlas`/`withatlas`/`paperonly`/`stippleonly` dumps at that same resolution. Hand-lettered glyphs, the fourth atlas element, are `map_overlay.gd`'s (GDScript overlay work, not renderer work). **Milestone 3 (hydrology tint)**: `land_color` gains a subtle cool/dark pull near high flow accumulation (`hydro_wet_strength`/`hydro_wet_radius_frac`, applied at the same final tonal stage as AO/vignette, never touching `material_weights`) — reuses the existing `flow` field already threaded through `RenderCtx` (zero `lib.rs` changes), log-compressed/min-max-normalized the same way `build_ao` already is, kept only above a `smoothstep` threshold, blurred into a soft halo. `js_reference()` sets it to `0.0` (a true no-op), both golden-parity render tests unchanged at `1e-4`. **Real tuning pass, disclosed**: the first parameter guess passed every mechanical check but a real crop at actual strength showed nothing perceptible (0.4% of pixels, mean diff 2.5/765) — caught by looking, not by the diff stats; retuned (0.20→0.38 strength, 0.004→0.006 radius, widened activation threshold) until a crop centred on the programmatically-found max-diff pixel showed a real, deliberately subtle valley-floor cooling. Cross-world honesty matching milestone 2's own AO finding: visible on Classic (2.19% of pixels), essentially imperceptible on low-relief Archipelago (0.75%) since there's simply less major drainage there — not a bug. Anti-list held: identical luma minimum before/after in both worlds (no new black valleys), no banding/haloing. Verified via the extended `appearance_ab_dump.rs` harness (an isolation pair holding milestone 2's own relief/AO fixed) rather than repeated windowed screenshots, following milestone 2's own finding that UI automation was unreliable this session — one real end-to-end windowed run confirmed correct generation/rendering, not a multi-shot comparison. **Milestone 2 (relief lighting)**: multidirectional hillshade (6 weighted lights, primary NW sun still dominant at 43%; the normal is computed once and dotted against a precomputed light table) plus heightfield ambient occlusion (`build_ao`, a two-scale cavity map over the existing box blur, replacing a `1.0` hardcoded in `land_color` since the renderer landed). Chosen because both act on the *lighting* term only, never on `material_weights`/the palettes — the golden-verified part, and the part §32 warns is easiest to improve for one terrain type while wrecking another. They're complementary: multi-light reveals ridgelines parallel to the single sun, but flattens depth; AO restores it from terrain concavity. AO normalizes each scale by its own RMS **over land cells only**, so occlusion is measured against each world's own relief statistics — a fixed threshold would give a flat world no AO and crush an alpine one. **Golden parity kept exact, not re-baselined and not loosened**: new `TerrainAppearance::js_reference()` reproduces the pre-milestone renderer bit-for-bit (`relief_lights: 1` takes a dedicated early-return branch; `ao_strength: 0.0` skips the precompute), and `golden_parity_render.rs` both tests still pass at their original `1e-4` tolerance with every expected value unchanged — the only edit is which appearance the context is built with. That follows `DECISIONS.md` §7a read strictly: its carve-out is for paths where JS parity is *impractical*, and it explicitly says the CPU rendering port stays golden-verified. Real before/after (deterministic dump + real windowed app, 2048², seed 12345): drainage networks, ridge/valley structure and coastal escarpments become legible where the single-sun render was a flat tan wash; measured against §30's anti-list, min luma is **identical** before/after in both test worlds (no black valleys) and mean luma moves only 133.3→128.8. A 3× zoom caught one real regression mid-pass (fine AO radius resolving to 1 cell read as speckle — "random texture noise") which was fixed before landing. Cost essentially nil: 512² render 45→45 ms. New `tests/appearance_ab_dump.rs` (`#[ignore]`d) is research doc §1.6's deterministic A/B comparison harness. **Milestone 1 (`TerrainAppearance` abstraction)** — `render.rs`'s colour logic (25 material/water palettes + shading constants, previously bare module consts) now lives behind a real, owned `TerrainAppearance` struct, pixel-identical output verified via `golden_parity_render.rs` unmodified. Real audit correction: there's no elevation-keyed colour *breakpoint ramp* in this renderer at all — colour comes from a continuous material-weight blend (temperature/moisture/slope/relative-elevation/aspect/curvature), not a MapTiler-style elevation lookup, so the research doc's own mental model doesn't map onto how this renderer actually works; a literal elevation ramp would be new visual-layer design work for a future milestone, not a re-encoding. Not yet wired to any UI — standalone-but-real, matching `cartalith-spatial`'s precedent. Three things to remember for what comes next: **(a)** criterion 2's renderer (above) ports the reference's *default-settings* material model only — real biome colours, real hillshade — explicitly excluding every `state.viz.*`-gated stretch feature (splat texturing, geology microtexture, NPR "Painter" styles, AO/SVF/shadows, multi-sun, SDF coast/river/biome tinting). Wiring any of those in is genuine Phase 3 work. **(b)** When that work lands, re-invoke `ui-ux-pro-max` for the UI side rather than bolting raw sliders onto the newly-exposed params — keep it consistent with the 2026-08-16 light parchment theme (ported from the reference's own `:root[data-theme="light"]`), not the earlier dark-dashboard match that theme replaced. **(c)** GPU compute *via Godot's own renderer* was researched 2026-08-16 (prompted by `godot-demo-projects/compute/heightmap`) and found not applicable *through that path*: `project.godot` uses the `gl_compatibility` renderer, which doesn't support `RenderingDevice` compute dispatch at all (engine-level constraint, already documented in `.claude/skills/godot-shell/SKILL.md`). That finding does **not** apply to a *standalone* `wgpu` instance created directly by Rust code — see the GPU-compute pilot section below, which tested exactly that and found the hardware path itself viable (the renderer choice is irrelevant to a `wgpu` instance that never touches Godot's own rendering pipeline). If Phase 3 revisits Godot's own renderer for other reasons (3D terrain drape, particles), GPU-accelerated presentation-layer work *through Godot* becomes reachable as a further, separate option — not before, and not for core generation (which must stay CPU-Rust for golden-parity reproducibility regardless of renderer). |
| 4 — Asset Library | **Started 2026-08-17, investigated for real and milestones 1-4 of 7 done** (`ASSET_LIBRARY_SCOPE.md`, new). `ROADMAP.md`'s own "Confirm before starting" note satisfied by the owner's direction to continue "until you've finished phase 4". **What it really is**, read out of the reference rather than out of the two pre-implementation design docs in `docs/`: an "asset" is not an arbitrary named image but **one PNG bound to one slot in a frozen, ordered vocabulary** — 8 families, 7 closed (7 splat channels / 15 biome grounds / 13 terrain grounds / 10 feature icons / 9 settlement pins / 7 trait overlays / 8 POI markers) plus one open-vocabulary `custom` family; slots hold 1..N variants picked by deterministic position hash. Order is load-bearing twice over (biome/terrain lists index-align 1:1 with the frozen `CART_BIOMES`/`CART_TERRAINS` paint vocabularies; structure lists mirror `CIV_SETTLEMENT_CLASSES`/`CIV_POI_TYPES`/`CIV_TRAITS`). An **asset pack is a real serialization format**, not a proposal — plain PKZIP via the same `zipStore()` the world save uses, `pack.json` (schema 1 or the schema-2 superset) or a real `pack.csv` alternative, manifest-is-source-of-truth, unknown keys warned rather than rejected. A **second, different** format also exists: `assetlib/library.json` + `assetlib/img/N.png` embedded in a project `.zip` (`_alExportEntries`/`_alImportProject`) — that is the "Asset Library payload" `SAVEFILE_COMPAT.md` already lists among ignored entries. The renderer genuinely draws pack sprites (`placeMapIcons`→`iconSlotForItem`→`pickWeightedVariant`→`drawMapIcons`, bottom-anchored); the vector glyphs are the fallback, not the reverse. Phase 5's urban morphology does **not** consume packs (checked). **Size, stated plainly**: ~2,250+ lines against the Journey Planner's ~3,100 — but only ~600-800 lines of that are portable logic, wrapped in 1,000+ lines of editor UI (the sprite-sheet slicer modal alone is ~408 lines of canvas/pointer interaction) plus an image/ZIP platform layer that is crate work, not porting. A real sub-phase, seven milestones. **Milestone 1 done**: new standalone crate `cartalith-assets` (no `gdext`, no dependency on any other Cartalith crate — `cartalith-spatial`'s precedent) carrying the pack manifest layer: the seven frozen vocabularies + a `Family` metadata enum, `RawManifest`/`PackManifest`, `parse_pack_csv`/`parse_pack_manifest`/`parse_pack_entries`, `pack_summary`, schema-2 `to_raw`/`to_pack_json`, and a ~40-line insertion-ordered map (needed because warning order follows the *author's* key order, `BTreeMap` would sort it away, and serde_json's `preserve_order` would have leaked into `cartalith-io` via workspace feature unification). **Golden-verified against the real reference** via a transient Node `vm` harness over `parsePackCsv`/`parsePackManifest`/`packSummary`; all five fixtures matched first run, targeting the plausibly-wrong cases (missing file vs. unknown slot, one variant missing vs. all missing, bare string as one-element list, stable CSV variant ordering, JSON-wins-over-CSV, empty path as missing file, exact wording *and order* of nine warnings). 28 tests. **Not wired to anything**, per the standing "don't wire in what nothing calls" discipline. **Milestone 2 done**: pack `.zip` read/write, placed in `cartalith-assets::archive` behind an on-by-default `zip` feature (the scope doc had left `cartalith-assets`-vs-`cartalith-io` open; reading `cartalith-io` first settled it — its whole zip surface is three `zip`-crate calls, so there is no helper to extract, it is reading-only by explicit scope so a pack *writer* would break that boundary, and the dependency would point the wrong way). What is actually ported is the reference's export *policy*, not the container: `.png` STORED and everything else DEFLATED, timestamps frozen at 1980-01-01 so exports are byte-reproducible (the `zip` crate's own default is the wall clock), `pack.json` written last, names read verbatim so a wrapping folder still fails the way the reference fails, directory entries kept, and an unreadable method erroring in the reference's own words. **Verified in both directions against a pack the reference itself exported** — the harness ran the reference's own `PackManifestBuilder.build()` + `zipStore()` headlessly (only the canvas rasteriser and three DOM inputs stubbed, stated in the test file); this port's read matches every name and CRC-32 and reproduces `pack.json` byte for byte, and its write reproduces order/method/CRC/size/timestamps *and* was fed back through the reference's own `unzipAny`+`parsePackManifest`, which read it with identical payloads, summary and warnings (the two archives differ by 2 bytes total, first divergence at the version-needed field). 14 new tests. **Milestone 3 done**: scatter rules (`cartalith-assets::scatter`) — `ScatterRule` + `ScatterMode`, the ten `SCATTER_RULE_PRESETS` that reproduce v1.25's hard-coded biome→asset switch, `scatter_rule_key`, `normalize_scatter_rule`, `current_scatter_rules`, `autopopulate_scatter_rules`, `pick_weighted_variant`/`pick_icon_variant` and `ScatterRule::spacing_cells`. The v1.27 hardening was **ported as fixes and re-derived for Rust**, one test naming each: the `NaN`-density carpet survives translation *by the opposite IEEE rule* (`f64::min` absorbs NaN where `Math.min` propagates it, and `keep >= 1.0` is false anyway); the `NaN`-spacing collapse of the relief bucket grid to 1×1 is real and Rust's `f64::max` would have masked it, so the `is_finite` guard stays explicit; and the `Object.assign` aliasing bug is **structurally unreachable — not because of ownership** but because defaults and untrusted input are different *types* (`ScatterRule` with `f64` fields vs. `serde_json::Value`), so no defensive code was written for it and the test asserts the observable outcome instead. Plus one guarantee the reference cannot have: `Serialize` but **deliberately no `Deserialize`**, making `normalize_scatter_rule` the only door in. **Golden-verified** by the same Node `vm` technique — `pick_weighted_variant` is deterministic-hash-driven and diffed exactly (11 cases × 36 positions, including the three degenerate weightings that must fall through to `pickIconVariant`'s untouched v1.25 hash), and 37 normalizer fixtures caught a real bug on the first run: `density`'s fallback is **not** symmetric with the other numeric fields — absent keeps the slot preset's own value (`cactus` stays 0.35) while a *rejected* one lands on a literal 1. 24 new tests, still wired to nothing. Three corrections to milestone 4 recorded: it is not the first milestone with a cross-crate dependency (milestone 3 is — `cartalith-noise`, for the variant hash); `pickIconVariant` and `spaceOf` shipped here rather than there; and `biomes` is `Vec<f64>` because `Number.isFinite` does not coerce, so a hand-edited `5.5` is kept and simply never matches. **Milestone 4 done**: rule-driven icon placement, `cartalith-assets::placement` — `place_map_icons_ruled` (the reference's `placeMapIconsRuled`), `icon_slot_for_item` with the `TREE_SLOT`/`SCATTER_SLOT` legacy fallback maps, and `sprite_draw_rect`; the reference's own legacy (non-ruled) `placeMapIcons` body is out of scope (nothing calls it, and `iconSlotForItem`'s legacy branches are ported for completeness without it). The first real placement golden-parity surface in this crate: positional and seeded, so it diffs **exactly**, not within a tolerance. **Both v1.27 fixes confirmed structurally necessary in Rust** (unlike one of milestone 3's three) — the most-specific-wins priority sort, because insertion-order dependence is a `Vec`/array property in any language; `requireWetland` ANDed with the biome test, because the old "replace" predicate is an algorithm defect a straight transcription would reproduce regardless of language. Proven with a hand-traceable 3-cell, `tGap=1` fixture (the scatter grid's jitter degenerates to zero at `tGap=1`, so sampling is exact per cell): a wetland+matching-biome cell, a dry+matching-biome cell, and a wetland+wrong-biome cell resolve to `wetland_grass`/`narrow_biome`/`generic_land` respectively, unchanged whether the rule array is inserted least-specific-first or reversed. **Golden-verified** against the real reference via the same Node `vm` technique: broad sweeps over a synthetic 10×8 grid across six seed/sea/density configurations match cell-for-cell and size-for-size (1e-9), including a dense case that exercises both relief bands, three different scatter specificities, and the `ghost_biome` non-integer-biome probe (`biomes:[5.5]`) placing nothing anywhere, confirming `biomeOk`'s `biome[i] as f64` cast. 23 new tests (12 unit + 11 golden), still wired to nothing. Milestones 5-7 (the Library model, image handling, renderer+Godot integration) scoped, no corrections found to them on this read. |
| 5 — Urban morphology | Not started. |

## Phase 4 — Asset Library (`ASSET_LIBRARY_SCOPE.md`, started 2026-08-17)

Seven milestones, three done. The scope doc carries the full investigation —
what an asset and an asset pack really are in the reference, the eight
families and their frozen slot vocabularies, how sprites actually reach the
map, the portable-vs-UI split with measured line counts, and what is
explicitly out of scope (the Library page UI, the sprite-sheet slicer modal,
the standalone pack compiler, and any wiring before milestone 7).

**Milestone 1 done** — `cartalith-assets`, the pack **manifest** layer:
data model, parser, validation warnings, schema-2 serialization. No images,
no archive, no renderer, no UI, and nothing in the workspace depends on it
yet — deliberately the piece every later milestone is defined against.
Golden-verified against the real reference implementation rather than
unit-tested by inspection, because a real headless execution path exists for
`parsePackCsv`/`parsePackManifest`/`packSummary`.

**Milestone 2 done** — pack `.zip` read/write, as `cartalith-assets::archive`
behind an on-by-default `zip` feature. The scope doc had deliberately left the
`cartalith-assets`-vs-`cartalith-io` placement open "until it starts"; reading
`cartalith-io` first is what decided it. Its whole zip surface is three
`zip`-crate calls, so milestone 1's "packs use the same `zipStore()` the world
save uses" implies a shared *crate*, not shared code; it is reading-only by
explicit scope, so a pack writer there would break that boundary; and the
dependency would point the wrong way, making the world-save loader drag in the
asset vocabulary. `default-features = false` still gives back exactly the
archive-free manifest model, and is tested that way.

The container is the crate's job; what is ported is the reference's own export
policy, which a plain `zip` call gets wrong by default — `.png` STORED and
everything else DEFLATED, timestamps frozen at 1980-01-01 so exports are
byte-reproducible, `pack.json` written last, names read verbatim (so zipping
the folder rather than its contents still fails exactly as the reference
fails), directory entries kept, and an unreadable compression method erroring
in the reference's own words. Two non-ports are stated rather than smuggled:
`zipStore`'s "only if it actually got smaller" fallback and `unzipStore`, both
browser-side concerns no reader can observe.

**Verified in both directions against a pack the reference itself exported.**
The harness ran the reference's own `PackManifestBuilder.build()` over its own
`FAMILIES`/`AssetDB` and its own `zipStore()` headlessly under Node's `vm`,
with only the canvas rasteriser and three DOM inputs stubbed — stated up front
in the test file rather than glossed. This port's read matches the reference's
`unzipAny` name for name and CRC-32 for CRC-32 and reproduces the exporter's
`pack.json` text byte for byte; its write reproduces entry order, method,
CRC-32, size and timestamps, and the bytes were fed back through the
reference's own `unzipAny` + `parsePackManifest`, which read all 18 entries
with identical payloads, summary and warnings. The two archives differ by 2
bytes in total. 14 new tests.

**Milestone 3 done** — scatter rules, as `cartalith-assets::scatter`: the
`ScatterRule` model that decides *where* an asset gets scattered, its ten slot
presets, the keyed rule table, weighted variant selection, and the hardened
normalizer. Still wired to nothing; the placement engine that consumes rules is
milestone 4.

**The three v1.27 hardening fixes were re-derived for Rust, not transcribed**,
with a test naming each. (1) A `NaN` `density` scattering on *every* cell is
**still a real hazard here, by the opposite IEEE rule** — JS reaches it through
`Math.min(1,NaN) === NaN`, Rust through `f64::min`'s NaN *absorption* giving
`1.0`, and `keep >= 1.0` is false anyway. (2) A `NaN` `spacing` collapsing the
relief bucket grid to 1×1 (an O(1) neighbour test becoming O(n²)) is real, and
Rust's `f64::max` would have masked it — so the `is_finite` check is kept
explicit rather than left to an IEEE corner, which fix 1 shows cannot be
trusted. (3) The `Object.assign` aliasing bug is **structurally unreachable**,
and *not* because of ownership: the bug needs defaults and untrusted input in
one mutable object, and here they are different *types* (`ScatterRule` with
`f64` fields vs. `serde_json::Value`), so a `"x"` can never be stored in the
field it would corrupt. No defensive code was written for it — the test pins
the reference's own probe case so a refactor toward a "merge" helper fails
loudly. A fourth guarantee the reference cannot have: `ScatterRule` implements
`Serialize` but **deliberately not `Deserialize`**, so the hardening is not
bypassable via `serde_json::from_str`.

**Golden-verified against the real reference**, same transient Node `vm`
technique. `pick_weighted_variant` is deterministic-hash-driven and diffed
exactly — 11 cases × 36 positions, index for index, including the three
degenerate weightings that must fall through to `pickIconVariant`'s untouched
v1.25 hash. 37 normalizer fixtures caught one real bug on the first run:
**`density`'s fallback is not symmetric with the other numeric fields** — an
absent `density` keeps the slot preset's own value (`cactus` stays 0.35) while
a *rejected* one lands on a literal `1`. 24 new tests. Three corrections to
milestone 4 recorded: it is not the first cross-crate dependency (this is —
`cartalith-noise`, for the variant hash); `pickIconVariant` and `spaceOf`
shipped here rather than there; and `biomes` is `Vec<f64>` because
`Number.isFinite` does not coerce.

**Milestone 4 done** — rule-driven icon placement, as
`cartalith-assets::placement`: `place_map_icons_ruled` (the reference's
`placeMapIconsRuled`), `icon_slot_for_item` with the `TREE_SLOT`/
`SCATTER_SLOT` legacy fallback maps, and `sprite_draw_rect`. The first real
placement golden-parity surface in this crate — positional and seeded, so it
diffs **exactly** rather than within a tolerance. Still wired to nothing.

**Both of milestone 4's own v1.27 fixes are structurally necessary in Rust,
not JS-only artifacts** — a real difference from milestone 3, where one of
three ported fixes turned out to be structurally unreachable here. (1) The
most-specific-wins priority sort: nothing about ownership or types makes
insertion-order dependence go away, a `Vec` iterates in build order exactly
like a JS array, so the sort is real ported logic. (2) `requireWetland` ANDed
with the biome test rather than replacing it: a straight transcription of the
old "replace" predicate would reproduce the bug faithfully in any language,
since it's an algorithm defect, not a consequence of JS coercion. Proven with
a hand-traceable fixture (`tGap=1` makes the scatter grid's own jitter
degenerate to zero, so `jx=gx,jy=gy` exactly): three cells, wetland+matching
biome / dry+matching biome / wetland+wrong biome, with the least-specific rule
inserted first — the winner comes out `wetland_grass` / `narrow_biome` /
`generic_land` regardless of insertion order, and reversing the whole rule
array doesn't change it.

**Golden-verified against the real reference**, same transient Node `vm`
technique. Broad sweeps over a synthetic 10×8 grid (a circular elevation peak,
a cycling biome pattern, a periodic wetland mask) across six seed/sea/density
configurations match cell-for-cell, key-for-key, and size-for-size to 1e-9 —
including one case exercising every rule family at once (both relief bands,
three different scatter specificities, and the always-empty `ghost_biome`
non-integer-biome probe placing nothing, confirming the `biome[i] as f64`
comparison). 23 new tests (12 unit + 11 golden).

**Not started**: 5 the Library model (`AssetDB`/collections/metadata/
validator/`assetlib/library.json`) · 6 image handling · 7 renderer + Godot
integration.

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

Per the scope doc's own feasibility table: the remaining graph/sequential
algorithms (water-body priority-flood's depression-fill half, Dijkstra/MST
road networks, orogeny, `compute_stress`'s scatter) remain a poor GPU fit
without real algorithmic redesign. Flow accumulation, the flagship entry on
that list, is no longer among them — see milestone 9 below.

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

**Milestone 9 — flow accumulation on GPU: done (2026-08-17), the first
genuinely sequential algorithm redesigned rather than ported.** The
owner's "do the algorithms for the GPU" directive, aimed at the one row
this document's own feasibility table had deferred longest.
`compute_flow` sorts every cell by descending height then walks that
order — but those are separable: **flow direction** is a pure function of
the height field (never reads `acc`, so the ordering is irrelevant —
embarrassingly parallel), and **accumulation** over the resulting
receiver forest is a subtree sum, which parallelizes by **pointer
doubling** in `ceil(log2(n))` rounds (22 at 2048²) rather than the
thousands a naive fixpoint iteration would need or the global sort the
CPU pays. Qin & Zhan 2012 / the 2016 RUSLE paper /
`HETEROGENEOUS_COMPUTE_RESEARCH.md` §48-49's own decomposition, applied
for real. Accumulation is `atomic<u32>` **fixed point**, not floats:
WGSL has no atomic float add, and a compare-exchange emulation would be
non-deterministic run to run, whereas integer addition is exactly
order-independent *and* bit-reproducible.

**Correctness**: flow directions **0 mismatches out of 262,144** (both
world-wrap modes, two roughness regimes). Accumulation against the real,
untouched `cartalith_hydrology::compute_flow` is **bit-exact for
`use_rain=false`** (the pipeline's first call), and for discharge seeding
diverges only by seed quantization — with the *opposite* shape to the
CPU's error (worst at tiny accumulations, shrinking as accumulation
grows, because the GPU rounds each seed once and is exact thereafter
while the CPU rounds to `f32` on every one of thousands of writes). At
and above `river_flow_thresh`, the only regime anything downstream
distinguishes: **1.3e-4 relative at 512², 3.3e-4 at 1024²**.

**The measured downstream effect is the real headline** — this is the
first GPU kernel here that is not a leaf computation, so the divergence
was traced through to the civilisation layer, holding terrain fixed:
**river network zero difference** (identical river-cell counts, 0
channel-mask cells, 0 channel receivers, 0 Strahler-order cells
differing) and **settlements zero difference** (`find_settlement_seeds`
returns the same count *and the same positions* — 104/104 at 512²,
125/125 at 1024², zero seeds moved; the suitability raster differs only
in its last `f32` digits, max 1.3e-5).

**Real timing**: isolated kernel 0.20× at 128² (GPU loses — the round
count barely falls with grid size, so a small grid pays nearly the same
dispatch count over far less work), 4.6× at 512², 10.4× at 1024², **15.5×
at 2048²** (31.5ms vs 488.9ms). End-to-end `generate_terrain` ratio moves
0.11×→0.16× / 0.76×→0.83× / 1.14×→**1.36×** / 0.98×→**1.74×** across
128²/512²/1024²/2048² — the largest single-milestone shift this effort
has produced, since `compute_flow` is called up to four times per
generation. Wired behind `p.use_gpu` with per-stage CPU fallback,
`"flow"` in `gpu_stages_used`, `compute_flow` itself byte-untouched,
`cargo test --workspace` 0 failures and 0 modified tests.

**Two honest "shouldn't run on GPU" findings** from reading the real
code: `build_water_bodies`' depression-fill half is a global priority
queue whose parallel formulations trade O(longest ascending path)
iterations for parallelism, with no pointer structure to double (its
connected-components half *is* tractable, and its exact CPU answer even
reproducible) — and it costs only ~92ms at 1024², an order of magnitude
below what flow accumulation was costing. `road_dijkstra` should stay on
CPU: its `prev` array literally *is* the road geometry and is
settle-order-dependent on ties (every GPU alternative would move roads),
and it is already called many independent times over a small downsampled
grid at four still-sequential `.iter().map()` call sites — the available
parallelism is across sources on CPU, not within one traversal on GPU.
See `GPU_LAYER_INTEGRATION_SCOPE.md`'s milestone 9 section and
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

## App icon (done 2026-08-17)

Owner-supplied icon (`design/app-icon.png`) wired into both platform build
targets: `project.godot`'s `config/icon` (editor/debug-run window icon —
screenshot-confirmed real, not assumed from config alone), Windows export's
`application/icon`/`console_wrapper_icon` (a real multi-resolution `.ico`
generated from the source), and Android's four `launcher_icons/*` fields
(legacy + adaptive foreground/background/monochrome, generated with real
safe-zone margins so launcher masks don't clip the content). Full record in
`CHANGELOG.md`'s "App icon wired for Windows and Android" entry.

## GUI shell + terrain appearance, second pass (done 2026-08-17)

Second workflow re-audit found and fixed a real structural gap: the Layers
panel is now a permanent fifth region (nav / params-or-placeholder / layers
/ viewport / inspector) rather than something the navigator swapped to —
matching the mockup's own always-visible region count. `GUI_SHELL_SCOPE.md`'s
own "second workflow re-audit" section has the full reasoning.

`TERRAIN_APPEARANCE_SCOPE.md` milestone 3 (hydrology-based colour
modulation, research doc §13) also landed: a subtle, flow-accumulation-
driven wetness tint on land colour near rivers/high flow, gated the same
way milestone 2's hillshade/AO were — `js_reference()` stays a true no-op,
`golden_parity_render.rs` unmodified at its original tolerance.

Both verified together: real end-to-end generation (seed 12345, Classic,
2048×2048, 40 settlements) through the restructured shell, full workspace
test suite green, headless load clean.

## Known-open items (not owner-blocked, just not done yet)

- Real Fira Sans/Fira Code font files for the UI theme (design-system match found the pairing; sourcing + OFL-license verification deferred).

## Owner-only items

- None currently open. Criterion 4 (real Android device build/install/launch/golden-path) fully closed 2026-08-17 once the owner unlocked the connected phone mid-session — see `ANDROID_BUILD_SCOPE.md`.
- This session has real Windows desktop + `godot4` CLI access + real Android device access, which closes most of what earlier sessions couldn't do themselves.

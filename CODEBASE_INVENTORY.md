# Codebase inventory

**R0 of `CODEBASE_REFACTOR_PROPOSAL.md`, written 2026-11-16 by the docs audit (`DOCS_AUDIT_2026-11-16.md`) at `c28e3aa3`.** An index, not a status record: what is built and how far is `cartalith-native/docs/STATUS.md`'s answer. Every figure below was produced by a script over the tree at that commit (the method is section 9); re-run it rather than trust a count here once the tree moves.

## 1. Dependency graph

Normal `[dependencies]` between workspace crates (arrows point at the dependency). Dev-dependencies are listed after it.

```
jsmath           -> (none)
noise            -> (none)
rng              -> (none)
vault            -> (none)
gpu              -> noise
io               -> spatial
knowledge        -> vault
llm              -> vault
llm-loopback     -> llm
spatial          -> jsmath
assets           -> jsmath, noise
hydrology        -> jsmath, terrain
urban            -> jsmath, rng
climate          -> jsmath, noise, terrain
erosion          -> jsmath, noise, rng, terrain
terrain          -> jsmath, noise, rng, spatial
civ              -> engine, erosion, hydrology, jsmath, noise, rng, terrain, urban
engine           -> assets, climate, erosion, gpu, hydrology, io, jsmath, spatial, terrain
godot            -> assets, civ, climate, engine, gpu, hydrology, io, jsmath, knowledge, llm, noise, rng, spatial, terrain, vault
```

Dev-dependencies: `gpu` -> civ, climate, engine, hydrology, terrain; `civ` -> spatial; `godot` -> erosion. `gpu`'s dev-dependencies on `engine` and `civ` close a test-only cycle (`ARCHITECTURE.md`).

## 2. Crates at a glance

| Crate | src files | src lines | test files | test lines | `#[test]` | `#[ignore]` | pub fn | pub types | used by (normal deps) |
|---|---|---|---|---|---|---|---|---|---|
| `cartalith-assets` | 20 | 14966 | 12 | 4712 | 344 | 0 | 203 | 93 | engine, godot |
| `cartalith-civ` | 42 | 85668 | 51 | 22110 | 1402 | 30 | 639 | 329 | godot |
| `cartalith-climate` | 5 | 3145 | 11 | 1439 | 68 | 0 | 31 | 13 | engine, godot |
| `cartalith-engine` | 20 | 19124 | 23 | 5236 | 342 | 20 | 154 | 73 | civ, godot |
| `cartalith-erosion` | 6 | 6938 | 8 | 1579 | 115 | 0 | 53 | 12 | civ, engine |
| `cartalith-godot` | 94 | 153753 | 85 | 48864 | 2087 | 215 | 983 | 230 | — |
| `cartalith-gpu` | 6 | 10337 | 2 | 815 | 118 | 2 | 76 | 25 | engine, godot |
| `cartalith-hydrology` | 3 | 6805 | 4 | 1035 | 84 | 0 | 42 | 8 | civ, engine, godot |
| `cartalith-io` | 9 | 9463 | 5 | 862 | 184 | 0 | 67 | 38 | engine, godot |
| `cartalith-jsmath` | 2 | 2260 | 0 | 0 | 17 | 0 | 20 | 0 | assets, civ, climate, engine, erosion, godot, hydrology, spatial, terrain, urban |
| `cartalith-knowledge` | 12 | 7104 | 5 | 793 | 101 | 2 | 129 | 87 | godot |
| `cartalith-llm` | 36 | 20668 | 6 | 1400 | 330 | 3 | 293 | 155 | godot, llm-loopback |
| `cartalith-llm-loopback` | 1 | 11 | 1 | 481 | 8 | 0 | 0 | 0 | — |
| `cartalith-noise` | 1 | 609 | 2 | 724 | 16 | 0 | 14 | 0 | assets, civ, climate, erosion, godot, gpu, terrain |
| `cartalith-rng` | 1 | 108 | 1 | 159 | 4 | 0 | 3 | 1 | civ, erosion, godot, terrain, urban |
| `cartalith-spatial` | 13 | 10065 | 5 | 1092 | 225 | 0 | 152 | 44 | engine, godot, io, terrain |
| `cartalith-terrain` | 19 | 22122 | 23 | 4161 | 412 | 0 | 195 | 74 | civ, climate, engine, erosion, godot, hydrology |
| `cartalith-urban` | 65 | 63535 | 16 | 25300 | 450 | 3 | 181 | 144 | civ |
| `cartalith-vault` | 18 | 15078 | 0 | 0 | 231 | 0 | 217 | 72 | godot, knowledge, llm |
| **total** | 373 | 451759 | 260 | 120762 | 6538 | 275 | 3452 | 1398 | |

`#[test]` counts attributes in every `.rs` file of the crate (unit tests in `src/` and integration tests in `tests/`), so it is not `cargo test`'s pass count, which also counts doc tests and excludes `cfg`-gated ones.

## 3. Section 7p switches: which crates read each

A crate "reads" a switch when its source names the switch's field (`.field`) outside a string; a lexical census, so a field name shared with another struct can over-count, and a switch handed on as a function argument (the `urban_*` switches reach `cartalith-civ`'s `urban_adapter` and `cartalith-urban` that way) under-counts. The switch list is every boolean `PARAMS` row (`cartalith_godot::params`).

| Switch | Crates whose source names it |
|---|---|
| `basin_routing` | civ, engine, godot |
| `carve_rivers` | engine, erosion, godot |
| `civ.biome_k` | godot |
| `civ.fixed_counts` | engine, godot |
| `civ.harbour_box` | godot |
| `civ.map_crossings` | civ, godot |
| `civ.map_rivers` | civ, godot |
| `civ.merge_ways` | civ, godot |
| `civ.metropolis` | godot |
| `civ.port_reach` | civ, godot |
| `civ.port_shore` | civ, godot |
| `civ.river_front` | civ, godot, gpu |
| `civ.villages` | godot |
| `climate.bulk_evap` | climate, engine, godot, gpu |
| `climate.currents` | climate, engine, godot |
| `climate.rain_bspline` | engine, godot |
| `climate.terrain_wind_deflection` | climate, engine, godot |
| `climate.wind_manual` | climate, engine, godot |
| `crater.physical_model` | engine, godot, terrain |
| `geology_carve` | engine, godot |
| `geology_karst` | civ, engine, godot |
| `geology_model` | engine, godot |
| `geology_processes` | civ, engine, godot |
| `integrate_drainage` | engine, godot |
| `lake_balance` | godot |
| `passes.coastal` | civ, engine, erosion, godot, io, knowledge |
| `passes.glacial` | civ, engine, erosion, godot |
| `passes.hillslope` | engine, godot |
| `passes.sediment_fill` | engine, godot |
| `passes.tidal_flats` | engine, godot |
| `passes.velocity` | engine, godot |
| `relief_model` | engine, godot |
| `rough_routing` | engine, godot |
| `tect.dynamic_lithology` | engine, erosion, godot |
| `tect.high_ceiling` | engine, godot |
| `tect.narrow_plate_base_blur` | engine, godot |
| `tect.orogen_relief` | engine, godot, llm |
| `tect.ridged` | engine, godot, llm, terrain |
| `urban_harbour_front` | godot |
| `urban_market_scale` | godot |
| `urban_river_kind_map` | godot |
| `urban_riverbank` | godot |
| `urban_street_village` | godot |
| `urban_wall_hug` | godot |
| `use_gpu` | engine, godot, gpu, terrain |
| `volc.cinder_grid_law` | engine, godot |
| `volc.dry_active_craters` | engine, godot |
| `volc.edifice_model` | engine, godot |
| `volc.exclude_transform` | engine, godot |
| `volc.fill_edifice_pits` | engine, godot |
| `volc.kind_caldera` | engine, godot |
| `volc.kind_dome` | engine, godot |
| `volc.kind_flood_basalt` | engine, godot |
| `volc.kind_maar` | engine, godot |
| `volc.kind_supervolcano` | engine, godot |
| `volc.lava_bench` | engine, godot |
| `volc.morphometry` | engine, godot |
| `volc.provinces` | civ, engine, godot, knowledge, terrain |
| `world` | not measurable lexically (`.world` is a common field name) |
| `world_structure.enabled` | not measurable lexically (`.enabled` is a common field name) |

## 4. Modules, per crate

Each module's first `//!` line (every `src/` file has one, checked), its line count, `pub fn` count and test attributes. Tests directories follow.

### `cartalith-assets`

Depends on: `cartalith-jsmath`, `cartalith-noise`. Used by: `cartalith-engine`, `cartalith-godot`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `examples/lineart_frames.rs` | 95 | 0 | 0 / 0 | AP-10's frame writer: runs `cartalith_assets::lineart` on every |
| `src/archive.rs` | 581 | 6 | 10 / 0 | Pack `.zip` read/write — the reference's `unzipAny` (line 12210) and |
| `src/coast.rs` | 278 | 3 | 8 / 0 | The coastline, as the *"snap sea marks to coast"* rule needs it. |
| `src/folder_import.rs` | 852 | 7 | 8 / 0 | Importing a **folder of art** into the Asset Library |
| `src/lib.rs` | 217 | 0 | 0 / 0 | Cartalith asset packs — the manifest layer. |
| `src/library.rs` | 1782 | 39 | 19 / 0 | The Asset Library model: `AssetDB`'s slot registry, `AssetCollections`, |
| `src/lineart.rs` | 1470 | 13 | 17 / 0 | AP-10 (Ruling CR, 2026-10-18): line art to a clean, solid sprite. |
| `src/manifest.rs` | 1020 | 15 | 11 / 0 | Asset-pack manifest: the `pack.json` / `pack.csv` data model, its parser, |
| `src/manual.rs` | 1126 | 14 | 31 / 0 | Hand-placed map icons — `UNIFIED_TOOL_PLAN.md` milestone E, the Icon stamp |
| `src/near_dupes.rs` | 617 | 5 | 8 / 0 | AP-6 (`ASSET_PACK_TOOLS_PROPOSAL.md` §6, Ruling CQ): find the *near* |
| `src/ordered_map.rs` | 234 | 12 | 4 / 0 | A string-keyed map that preserves JSON document order. |
| `src/pack_diff.rs` | 385 | 5 | 4 / 0 | Pack diff and merge (`ASSET_PACK_TOOLS_PROPOSAL.md` AP-9, Ruling CQ): |
| `src/pack_import.rs` | 646 | 4 | 9 / 0 | Opening a pack `.zip` **into the Asset Library** (`ASSET_PACK_TOOLS_PROPOSAL.md` |
| `src/pack_report.rs` | 565 | 5 | 6 / 0 | The pack report (`ASSET_PACK_TOOLS_PROPOSAL.md` AP-2, Ruling CQ): what is |
| `src/placement.rs` | 1009 | 8 | 20 / 0 | Rule-driven icon placement — *where* a scattered asset lands on the map. |
| `src/raster.rs` | 993 | 14 | 23 / 0 | Milestone 6 (`ASSET_LIBRARY_SCOPE.md`): the first place this crate touches |
| `src/refine.rs` | 578 | 11 | 8 / 0 | AP-7 (`ASSET_PACK_TOOLS_PROPOSAL.md` §6, Ruling CQ): the pixel half of |
| `src/scatter.rs` | 845 | 10 | 17 / 0 | Scatter rules — the data that decides *where* an asset gets scattered. |
| `src/seamless.rs` | 298 | 2 | 5 / 0 | AP-4 (`ASSET_PACK_TOOLS_PROPOSAL.md` §6, Ruling CQ): make a ground tile |
| `src/slicer.rs` | 900 | 21 | 22 / 0 | The sprite-sheet slicer's arithmetic and pixel work — the reference's |
| `src/slots.rs` | 570 | 12 | 9 / 0 | The frozen slot vocabularies an asset pack is written against. |
| `tests/golden_parity_library.rs` | 563 | 0 | 32 / 0 | Golden-parity tests for `cartalith-assets`' Library model against the |
| `tests/golden_parity_manual_icons.rs` | 518 | 0 | 7 / 0 | Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E's Icon stamp |
| `tests/golden_parity_pack_manifest.rs` | 575 | 0 | 9 / 0 | Golden-parity tests for `cartalith-assets`' manifest parser against the |
| `tests/golden_parity_pack_zip.rs` | 283 | 0 | 4 / 0 | Golden-parity test for the pack **archive** layer against a pack the |
| `tests/golden_parity_placement.rs` | 446 | 0 | 11 / 0 | Golden-parity tests for `cartalith-assets`' rule-driven icon placement |
| `tests/golden_parity_raster.rs` | 144 | 0 | 2 / 0 | Golden-parity tests for the two milestone-6 (`ASSET_LIBRARY_SCOPE.md`) |
| `tests/golden_parity_scatter_rules.rs` | 655 | 0 | 11 / 0 | Golden-parity tests for `cartalith-assets`' scatter rules against the |
| `tests/golden_parity_slicer.rs` | 540 | 0 | 5 / 0 | Golden-parity tests for the sprite-sheet slicer (`ASSET_LIBRARY_SCOPE.md` |
| `tests/golden_parity_zip_store.rs` | 173 | 0 | 5 / 0 | Golden-parity tests for `zipStore` (reference `Cartalith Gen1 v2.10.html` |
| `tests/hardening_asset_db.rs` | 252 | 0 | 7 / 0 | `AssetDB`'s own id-slugging and uid-collision hardening, named and pinned |
| `tests/hardening_v1_27.rs` | 282 | 0 | 4 / 0 | The three v1.27 hardening fixes, each named and each pinned by a test. |
| `tests/lineart_ap10.rs` | 281 | 0 | 8 / 0 | AP-10's acceptance bars (`ASSET_PACK_TOOLS_PROPOSAL.md` §9.3), checked on |

### `cartalith-civ`

Depends on: `cartalith-engine`, `cartalith-erosion`, `cartalith-hydrology`, `cartalith-jsmath`, `cartalith-noise`, `cartalith-rng`, `cartalith-terrain`, `cartalith-urban`. Used by: `cartalith-godot`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `examples/_memlane_peak.rs` | 432 | 0 | 0 / 0 | `_memlane_peak` -- generation-peak allocation audit, 2026-09-27 re-run of |
| `examples/civ_timing_bench.rs` | 120 | 0 | 0 / 0 |  |
| `src/belief.rs` | 3445 | 33 | 47 / 0 | Culture and religion as quantitative traits, the compatibility relation |
| `src/campaign.rs` | 463 | 4 | 8 / 0 | Ruling AW (`LARGE_ITEM_RULINGS.md`, 2026-09-24) -- war campaigns over time, |
| `src/conflict.rs` | 493 | 15 | 8 / 0 | `STORY_PLANNING_SCOPE.md` SP-4 -- the conflict overlay's entity. |
| `src/currency.rs` | 127 | 4 | 4 / 0 | Per-faction currencies — Ruling R (2026-09-21), kept by Ruling AR and |
| `src/garrison.rs` | 555 | 4 | 8 / 0 | Ruling AW (`LARGE_ITEM_RULINGS.md`, 2026-09-24) -- per-settlement |
| `src/journey_progress.rs` | 581 | 12 | 8 / 0 | `STORY_PLANNING_SCOPE.md` SP-2 -- journey progression over the cursor. |
| `src/labels.rs` | 2957 | 33 | 80 / 3 | Region-name labels — `UNIFIED_TOOL_PLAN.md` milestone E, the Label (`T`) |
| `src/lake_balance.rs` | 369 | 3 | 10 / 0 | **RR-6, the per-basin lake water balance** (`RIVER_REALISM_SCOPE.md` RR-6; |
| `src/landmark.rs` | 10784 | 37 | 94 / 0 | **Landmark generation** — the record, the cap table and one pass. |
| `src/landmark/cultural_kinds.rs` | 131 | 2 | 0 / 0 | **M9's cultural half** — Sacred mountain, Sacred grove, Shrine and Ceremonial site |
| `src/landmark/karst_view.rs` | 161 | 5 | 0 / 0 | **GF-6's landmark reading of GF-5** (`GEOLOGY_FIRST_SCOPE.md` §4.11, §7 |
| `src/landmark/m9_tests.rs` | 628 | 0 | 15 / 3 |  |
| `src/landmark/natural_kinds.rs` | 588 | 7 | 0 / 0 | **M8 residual, physical and economic kinds** — Delta, Ancient forest, Salt works and |
| `src/landmark/natural_kinds_tests.rs` | 381 | 0 | 14 / 0 | Tests for `natural_kinds.rs`: fixtures shaped to reach each detector (`CLAUDE.md`: "shape |
| `src/landmark/place_name.rs` | 356 | 1 | 8 / 0 | Ruling EI (owner, 2026-11-08): the name a placed landmark is linked by. |
| `src/landmark/refined_view.rs` | 1062 | 15 | 9 / 0 | **Ruling AT (owner, 2026-09-24): the manual "refine viewshed for the |
| `src/landmark/rock_view.rs` | 151 | 5 | 0 / 0 | **GF-6's landmark reading of the rock column** (`GEOLOGY_FIRST_SCOPE.md` |
| `src/landmark/temporal_kinds.rs` | 446 | 7 | 0 / 0 | **M9's temporal half** (`LANDMARK_GENERATION_SCOPE.md` M9; research §20 and §25): the four |
| `src/lib.rs` | 30180 | 224 | 317 / 0 | Civilisation-layer affordance fields (`ROADMAP.md` Phase 2, milestone 1 |
| `src/manpower.rs` | 2991 | 16 | 38 / 0 | Military manpower — what a polity can actually put and keep under arms |
| `src/military.rs` | 354 | 4 | 9 / 0 | Fortification and defensive strength — `GUI_GAP_REGISTER.md` **CV-25**. |
| `src/naming.rs` | 475 | 6 | 11 / 0 | Name shaping for things the reference does not name. |
| `src/poi.rs` | 320 | 8 | 5 / 0 | User-placed points of interest -- `CITY_MAP_LOOK_SCOPE.md` CL-11, Ruling EW item 4 |
| `src/relations.rs` | 605 | 2 | 11 / 0 | Pairwise faction relations — `GUI_GAP_REGISTER.md` **CV-26**. |
| `src/religion_library.rs` | 774 | 17 | 12 / 0 | Custom religions as **named variants** of the built-ins -- |
| `src/religion_profile.rs` | 723 | 8 | 8 / 0 | FH-R2 milestone R2-1's **`ReligionProfile` table**: each built-in religion's |
| `src/religious_policy.rs` | 171 | 6 | 2 / 0 | FH-R2 milestone R2-6, the research document's **S6: a per-faction |
| `src/river_class.rs` | 212 | 4 | 4 / 0 | River-channel classes, read by **both** the Route tool's mixed-domain router |
| `src/river_stretch.rs` | 702 | 10 | 0 / 0 | Ruling EU item 3 (owner, 2026-11-12): **the planner offers river stretches, one |
| `src/roster.rs` | 943 | 5 | 16 / 0 | The faction roster's and place editor's **vocabulary tables**, plus the |
| `src/timeline.rs` | 4885 | 47 | 66 / 0 | Milestone 1 of `TIMELINE_SCOPE.md` -- the shared prerequisites the |
| `src/tools.rs` | 2233 | 26 | 32 / 0 | `UNIFIED_TOOL_PLAN.md` milestone D -- the Civilization tool group's |
| `src/trade.rs` | 2168 | 21 | 0 / 0 | Trade **flows** — who trades with whom, over what water, along which way |
| `src/trade/tests.rs` | 1930 | 0 | 68 / 0 | `trade.rs`'s own tests. |
| `src/travel_library.rs` | 2048 | 22 | 21 / 0 | The Travel Library data model: `TRAVEL_LIBRARY_SPEC.md`'s four |
| `src/urban_adapter.rs` | 2910 | 25 | 0 / 0 | The civ-side adapter between this port's world state and |
| `src/urban_adapter/tests.rs` | 2583 | 0 | 81 / 0 | Ordinary unit tests over synthetic fields. |
| `src/way_audit.rs` | 636 | 9 | 7 / 0 | Measures of the drawn road network (owner request 2026-11-12, Ruling EU |
| `src/way_class.rs` | 283 | 3 | 5 / 0 | One table of way classes, read by **both** the Route/Way tools' least-cost |
| `src/way_join.rs` | 1394 | 5 | 22 / 0 | Ruling EU item 2 (owner, 2026-11-12): **a planned route joins a way heading |
| `src/way_merge.rs` | 1506 | 6 | 21 / 0 | Shared trunks for the generated road network (owner request 2026-11-12, |
| `src/wildlife.rs` | 964 | 10 | 9 / 0 | Wildlife layer — `buildTRI`/`guildTrophic`/`buildEcoregions`/ |
| `tests/bugsweep_road_edges.rs` | 191 | 0 | 7 / 0 | Found-bug sweep 2026-10-11, item R2: `RoadEdge::path` holds **routing-grid** |
| `tests/carve_leaves_no_pits.rs` | 207 | 0 | 4 / 0 | RV-1 (`LARGE_ITEM_RULINGS.md` Ruling BD): the river carve must not leave |
| `tests/dry_active_craters.rs` | 142 | 0 | 4 / 0 | Owner Ruling CX item 6 (2026-10-24): an active volcano keeps a dry crater; |
| `tests/ef9_mixed_edges.rs` | 613 | 0 | 1 / 1 | **EF-9.3 -- mixed-level shared edges. Measurement first, no output change.** |
| `tests/ef9_refinement_gain.rs` | 2391 | 0 | 30 / 2 | **EF-9.0 -- what "refines sooner" buys. Measurement only; no output change.** |
| `tests/gf6_carve.rs` | 248 | 0 | 2 / 2 | GF-6's carve measurement (`GEOLOGY_FIRST_SCOPE.md` §4.7, §7 GF-6): the |
| `tests/gf6_identity_hashes.rs` | 73 | 0 | 1 / 1 | GF-6's switch-off identity check for the landmark pass: app worlds |
| `tests/gf6_karst_landmarks.rs` | 257 | 0 | 4 / 1 | GF-6's landmark half (`GEOLOGY_FIRST_SCOPE.md` §4.11, §7 GF-6): *cave* |
| `tests/gf6_rock_landmarks.rs` | 170 | 0 | 3 / 1 | GF-6's rock half of the landmarks (`GEOLOGY_FIRST_SCOPE.md` §4.7, §4.11, |
| `tests/golden_parity_affordance.rs` | 102 | 0 | 2 / 0 | Golden-parity tests for cartalith-civ's affordance fields (lithology, |
| `tests/golden_parity_biome.rs` | 106 | 0 | 2 / 0 | Golden-parity tests for cartalith-civ's biome classification |
| `tests/golden_parity_carrying_capacity.rs` | 302 | 0 | 2 / 0 | Golden-parity tests for cartalith-civ's carrying capacity, NPP, and |
| `tests/golden_parity_centrality_feedback.rs` | 318 | 0 | 20 / 0 | GENERATED by `tools/civ_tier_feedback_capture.js` -- do not hand-edit. |
| `tests/golden_parity_civ_tools.rs` | 769 | 0 | 16 / 0 | Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone D -- the |
| `tests/golden_parity_faction_aggregates.rs` | 1345 | 0 | 2 / 0 | Golden-parity tests for `ECONOMY_SCOPE.md`'s last unstarted piece: |
| `tests/golden_parity_hierarchical_network.rs` | 222 | 0 | 2 / 0 | Golden-parity tests for the civ auto-populate road network's raw |
| `tests/golden_parity_labels.rs` | 691 | 0 | 21 / 0 | Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E's Label tool: |
| `tests/golden_parity_metropolis_recovery.rs` | 933 | 0 | 27 / 0 | Golden-parity tests for the two v0.75/v0.82 passes ported 2026-08-20 on |
| `tests/golden_parity_military.rs` | 308 | 0 | 3 / 0 | Golden-parity tests for `GUI_GAP_REGISTER.md` **CV-25**'s three ported |
| `tests/golden_parity_resource_potentials.rs` | 204 | 0 | 2 / 0 | Golden-parity tests for cartalith-civ's resource potentials |
| `tests/golden_parity_road_consolidation.rs` | 396 | 0 | 4 / 0 | Golden-parity tests for road-corridor consolidation, classification, |
| `tests/golden_parity_road_network.rs` | 201 | 0 | 2 / 0 | Golden-parity tests for the road network algorithm -- |
| `tests/golden_parity_roster.rs` | 161 | 0 | 3 / 0 | Golden-parity tests for the two genuinely-ported functions behind |
| `tests/golden_parity_sea_routes.rs` | 229 | 0 | 2 / 0 | Golden-parity tests for sea-lane routing -- `PHASE2_SCOPE.md` milestone |
| `tests/golden_parity_settlement_naming.rs` | 450 | 0 | 3 / 0 | Golden-parity tests for settlement population + naming -- |
| `tests/golden_parity_settlement_placement.rs` | 361 | 0 | 2 / 0 | Golden-parity tests for the pure, non-DOM-coupled core of |
| `tests/golden_parity_settlement_population.rs` | 313 | 0 | 9 / 0 | Golden-parity tests for `TIMELINE_SCOPE.md` milestone 1's population- |
| `tests/golden_parity_settlement_prereqs.rs` | 224 | 0 | 3 / 0 | Golden-parity tests for cartalith-civ's settlement-suitability |
| `tests/golden_parity_settlement_suitability.rs` | 808 | 0 | 2 / 0 | Golden-parity tests for `buildSettlementSuitability`/`findSettlementSeeds` |
| `tests/golden_parity_smelting_salt.rs` | 602 | 0 | 4 / 0 | Golden-parity tests for `ECONOMY_SCOPE.md`'s two remaining unblocked |
| `tests/golden_parity_timeline_collapse.rs` | 619 | 0 | 9 / 0 | Golden-parity tests for `TIMELINE_SCOPE.md` milestone 3 |
| `tests/golden_parity_timeline_graph.rs` | 324 | 0 | 6 / 0 | Golden-parity tests for `TIMELINE_SCOPE.md` milestone 2 |
| `tests/golden_parity_timeline_orchestrator.rs` | 416 | 0 | 4 / 0 | Golden-parity tests for `TIMELINE_SCOPE.md` milestone 4's orchestrator: |
| `tests/golden_parity_urban_adapter.rs` | 1546 | 0 | 9 / 0 | GENERATED by `cartalith-native/tools/um_block2_capture.js` -- do not hand-edit. |
| `tests/golden_parity_village_connect.rs` | 293 | 0 | 7 / 0 | Golden parity for `_civConnectVillageAddons` (reference v2.11 line 25766) |
| `tests/golden_parity_village_seeding.rs` | 109 | 0 | 1 / 0 | Golden-parity test for village seeding -- `PHASE2_SCOPE.md` milestone |
| `tests/golden_parity_want_counts.rs` | 51 | 0 | 1 / 0 | GENERATED by `tools/civ_want_counts_capture.js` -- do not edit by hand. |
| `tests/golden_parity_waterbodies.rs` | 209 | 0 | 2 / 0 | Golden-parity tests for cartalith-civ's water-body classification |
| `tests/golden_parity_wildlife.rs` | 462 | 0 | 8 / 0 | Golden-parity test for the wildlife / ecoregion layer |
| `tests/integrated_drainage.rs` | 394 | 0 | 3 / 2 | Integrated drainage (`RC_ENGINE_CHANGES.md` §6g/§6k), measured the way §6g |
| `tests/iterative_network.rs` | 435 | 0 | 9 / 2 | `civ_iterative_network` -- `_civIterativeAutoWorld`'s centrality -> tier |
| `tests/landmark_timing.rs` | 433 | 0 | 6 / 6 | Ad-hoc timing harness for `landmark::generate` — diagnostic only. |
| `tests/mountain_relief_civ.rs` | 265 | 0 | 1 / 2 | **Ruling CU item 1** -- the regression check the owner asked for on the |
| `tests/route_km_on_map.rs` | 323 | 0 | 5 / 0 | Route km measures the on-map line the router returns (Ruling CF item 4, |
| `tests/route_points_in_bounds.rs` | 170 | 0 | 4 / 0 | The Route/Way router's returned points stay on the map (found-bug row of |
| `tests/ruling_cd_batch1.rs` | 307 | 0 | 7 / 0 | Ruling CD batch 1 (owner, 2026-10-11): unit tests on hand-built fixtures for |
| `tests/ruling_cd_rivers.rs` | 388 | 0 | 10 / 0 | Ruling CD batch 2 (owner, 2026-10-11, item 1): real river travel. Unit tests |
| `tests/ruling_cd_slope.rs` | 503 | 0 | 15 / 0 | Ruling CD batch 3 (owner, 2026-10-11, item 2 and addendum item 5): slope in |
| `tests/ruling_eu_river_legs.rs` | 660 | 0 | 22 / 0 | Ruling EU item 3 (owner, 2026-11-12): river stretches offered per stretch, |
| `tests/volcano_cx_bars.rs` | 723 | 0 | 3 / 4 | **Ruling CX's regression bars** (owner, 2026-10-24): every Ruling CX item |
| `tests/zero_length_route.rs` | 143 | 0 | 3 / 0 | A zero-length route is recognised, not planned as a 0-day journey |

Ignored harnesses (26): `src/labels.rs::cull_timing_probe`, `src/landmark/m9_tests.rs::delta_plain_share_distribution`, `src/landmark/m9_tests.rs::displacement_of_old_kinds_by_the_new_ones`, `src/landmark/m9_tests.rs::salt_constraint_breakdown`, `tests/ef9_mixed_edges.rs::ef9_3_mixed_edge_measurement`, `tests/ef9_refinement_gain.rs::ef9_0_refinement_gain_measurement`, `tests/ef9_refinement_gain.rs::ef9_1_importance_fit`, `tests/gf6_carve.rs::gf6_carve_bars`, `tests/gf6_carve.rs::gf6_carve_identity_hashes`, `tests/gf6_identity_hashes.rs::gf6_identity_hashes`, `tests/gf6_karst_landmarks.rs::gf6_measure`, `tests/gf6_rock_landmarks.rs::gf6_rock_measure`, `tests/integrated_drainage.rs::measure_integrated_drainage`, `tests/integrated_drainage.rs::time_routing_surface_at_shell_size`, `tests/iterative_network.rs::measure_duplicate_ways`, `tests/iterative_network.rs::measure_tier_shift`, `tests/landmark_timing.rs::derived_build_component_cost`, `tests/landmark_timing.rs::geology_inputs_cost`, `tests/landmark_timing.rs::landmark_generate_at_the_shipping_default`, `tests/landmark_timing.rs::landmark_generate_on_a_rough_field`, `tests/landmark_timing.rs::landmark_generate_scaling`, `tests/landmark_timing.rs::landmark_generate_settlement_scaling`, `tests/mountain_relief_civ.rs::mountain_relief_civ_regression`, `tests/volcano_cx_bars.rs::va6_late_kind_refusals`, `tests/volcano_cx_bars.rs::va6_rare_kind_census`, `tests/volcano_cx_bars.rs::volcano_cx_bars`.

### `cartalith-climate`

Depends on: `cartalith-jsmath`, `cartalith-noise`, `cartalith-terrain`. Used by: `cartalith-engine`, `cartalith-godot`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `src/geoid.rs` | 291 | 4 | 5 / 0 | G2 geoid — `buildGeoid`/`refreshGeoid`/`geoAt`/`currentGeoidPreview` |
| `src/koppen.rs` | 488 | 7 | 9 / 0 | Seasons and Köppen–Geiger classification — `computeTempInto`/ |
| `src/lib.rs` | 1908 | 17 | 5 / 0 | temperature, wind, rainfall |
| `src/tides.rs` | 269 | 4 | 4 / 0 | G3 moons & tides — `tidalForcing`/`computeTideField`/`buildTideField`/ |
| `src/windthrow.rs` | 189 | 1 | 4 / 0 | Wind-throw hazard (reference HTML lines 5599-5636). |
| `tests/deflect_flow_regression.rs` | 93 | 0 | 2 / 0 | NOT a golden-parity test, and no longer the only coverage `deflect_flow` |
| `tests/golden_parity_deflect_flow.rs` | 60 | 0 | 3 / 0 | Golden-parity tests for `deflect_flow` (PARITY_TESTING.md). Reference |
| `tests/golden_parity_geoid.rs` | 235 | 0 | 7 / 0 | Golden-parity test for the G2 geoid (`PARITY_TESTING.md`) — reference |
| `tests/golden_parity_koppen.rs` | 312 | 0 | 6 / 0 | Golden-parity test for seasons + Köppen–Geiger classification |
| `tests/golden_parity_moisture_correctors.rs` | 42 | 0 | 2 / 0 | Golden-parity test for apply_climate_moisture_correctors (PARITY_TESTING.md). |
| `tests/golden_parity_ocean_current.rs` | 54 | 0 | 2 / 0 | Golden-parity tests for `compute_ocean_current` (PARITY_TESTING.md). |
| `tests/golden_parity_temperature.rs` | 63 | 0 | 3 / 0 | Golden-parity test for compute_temperature (PARITY_TESTING.md). |
| `tests/golden_parity_tides.rs` | 205 | 0 | 6 / 0 | Golden-parity test for the G3 tidal-range field (`PARITY_TESTING.md`) — |
| `tests/golden_parity_weather.rs` | 134 | 0 | 4 / 0 | Golden-parity test for simulate_weather (PARITY_TESTING.md). |
| `tests/golden_parity_windthrow.rs` | 135 | 0 | 4 / 0 | Golden-parity test for the wind-throw hazard field |
| `tests/ocean_current_regression.rs` | 106 | 0 | 2 / 0 | NOT a golden-parity test, and no longer the only coverage these two |

### `cartalith-engine`

Depends on: `cartalith-assets`, `cartalith-climate`, `cartalith-erosion`, `cartalith-gpu`, `cartalith-hydrology`, `cartalith-io`, `cartalith-jsmath`, `cartalith-spatial`, `cartalith-terrain`. Used by: `cartalith-civ`, `cartalith-godot`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `examples/compute_config_bench.rs` | 452 | 0 | 0 / 0 | Compute-configuration benchmark: CPU/Rayon vs. each real GPU vs. the |
| `examples/gpu_pool_bench.rs` | 91 | 0 | 0 / 0 | `use_gpu` generation time with and without `cartalith-gpu`'s buffer pool |
| `examples/timing_bench.rs` | 31 | 0 | 0 / 0 |  |
| `src/bake.rs` | 824 | 10 | 10 / 0 | The bake — reference lines 10575 (`pyramidTile`), 10765 |
| `src/center.rs` | 432 | 1 | 5 / 0 | `centerLandmasses()` (reference HTML lines 3179-3199) — the WORLD |
| `src/channel_atlas.rs` | 479 | 5 | 7 / 0 | The channel atlas — `packRGB8`/`unpackRGB8`/`channelAtlasGroups`/ |
| `src/continuous_lod.rs` | 648 | 16 | 12 / 0 | **EF-9.4's continuous effective level** -- importance-driven refinement as a |
| `src/elevation.rs` | 1712 | 8 | 23 / 0 | **EF-0** — the elevation field as a queryable, deterministic, |
| `src/erode_op.rs` | 742 | 2 | 11 / 2 | `erode()` — the reference's droplet-erosion button, assembled. |
| `src/geo_clock.rs` | 394 | 13 | 6 / 0 | GF-7: the geological clock (`GEOLOGY_FIRST_SCOPE.md` §4.12; owner Ruling BJ, |
| `src/geojson.rs` | 623 | 5 | 10 / 0 | `exportGeoJSON` — `UNIFIED_TOOL_PLAN.md` milestone E2, the vector half of |
| `src/gpu_divergence_bench.rs` | 244 | 0 | 1 / 2 | Which GPU kernel makes the GPU path's world differ from one device to |
| `src/import.rs` | 533 | 3 | 6 / 0 | Heightmap import — `loadImage()` + `inferTectonics()` (reference HTML |
| `src/importance.rs` | 1051 | 19 | 14 / 0 | **EF-9.1** -- per-tile *importance*: how much a pyramid tile would gain |
| `src/lib.rs` | 6203 | 30 | 41 / 7 | orchestrator: owns WorldState, runs the pipeline stages in order |
| `src/preview.rs` | 433 | 7 | 9 / 0 | **The New World seed preview** (`NEW_WORLD_PREVIEW_PROPOSAL.md` NWP-1, |
| `src/progress.rs` | 298 | 4 | 4 / 0 | Process-global generation-stage progress counter (the owner's locked |
| `src/region_export.rs` | 992 | 5 | 27 / 0 | `exportRegionTiles` — `UNIFIED_TOOL_PLAN.md` milestones E and E2, the |
| `src/rock_paint.rs` | 722 | 6 | 8 / 0 | GF-8: lithology painting's engine half (`GEOLOGY_FIRST_SCOPE.md` §4.13, |
| `src/sculpt_commit.rs` | 516 | 3 | 10 / 0 | `sculptCommit`'s water hooks — `UNIFIED_TOOL_PLAN.md` milestone C's |
| `src/slippy_export.rs` | 241 | 2 | 6 / 0 | A slippy-map tile pyramid export — the LOD pyramid written out under |
| `src/staleness.rs` | 709 | 4 | 16 / 2 | Cartalith's own generation-stage dependency chain, as a |
| `src/subdivision.rs` | 1328 | 16 | 27 / 0 | **EF-9.2** -- the subdivision rule: which pyramid tiles of a view to draw, |
| `tests/bake_real_world.rs` | 168 | 0 | 1 / 2 | The bake, end to end, over a **real generated world** rather than a |
| `tests/ef3_tile_erosion.rs` | 986 | 0 | 6 / 3 | **EF-3** — tile-bounded erosion (`cartalith_erosion::tile`) against a **real |
| `tests/fixtures/pre_rv1_world.rs` | 137 | 0 | 0 / 0 | The generated worlds the JS-parity golden suites were captured against, |
| `tests/geology_gf1.rs` | 257 | 0 | 5 / 0 | GF-1 of `GEOLOGY_FIRST_SCOPE.md`: the lithology model and column, read by |
| `tests/geology_gf11.rs` | 161 | 0 | 4 / 0 | GF-11 of `GEOLOGY_FIRST_SCOPE.md` (owner Rulings BM and BY): rock-aware |
| `tests/geology_gf2.rs` | 344 | 0 | 6 / 0 | GF-2 of `GEOLOGY_FIRST_SCOPE.md` (owner Rulings BH and BJ), on whole |
| `tests/geology_gf4.rs` | 48 | 0 | 2 / 0 | GF-4 (`GEOLOGY_FIRST_SCOPE.md` §3.2 item 2, §4.5; owner Rulings BH and |
| `tests/geology_gf5.rs` | 172 | 0 | 4 / 0 | GF-5 of `GEOLOGY_FIRST_SCOPE.md` (owner Rulings BH and BJ): karst |
| `tests/geology_gf6.rs` | 121 | 0 | 2 / 0 | GF-6 of `GEOLOGY_FIRST_SCOPE.md` (owner Ruling BH): the river carve's |
| `tests/geology_gf7.rs` | 157 | 0 | 4 / 0 | GF-7 of `GEOLOGY_FIRST_SCOPE.md` (owner Ruling BJ): the geological clock, |
| `tests/gf8_rock_paint.rs` | 249 | 0 | 5 / 0 | GF-8 of `GEOLOGY_FIRST_SCOPE.md` (owner Ruling BJ): bar B11 (§5) on |
| `tests/golden_parity_bake.rs` | 278 | 0 | 6 / 0 | Golden-parity tests for the bake's composed half — `pyramidTile` |
| `tests/golden_parity_carve.rs` | 215 | 0 | 3 / 0 | Golden-parity test for cartalith-engine::generate_terrain's full |
| `tests/golden_parity_geojson.rs` | 226 | 0 | 2 / 0 | Golden-parity test for `exportGeoJSON` (reference `Cartalith Gen1 v2.10.html` |
| `tests/golden_parity_pipeline.rs` | 119 | 0 | 2 / 0 | Golden-parity test for cartalith-engine::generate_terrain -- the tectonic |
| `tests/golden_parity_region_export.rs` | 259 | 0 | 3 / 0 | Golden-parity test for `UNIFIED_TOOL_PLAN.md` milestones E and E2's |
| `tests/golden_parity_sculpt_water.rs` | 566 | 0 | 11 / 0 | Golden-parity tests for `sculptCommit`'s water hooks — |
| `tests/non_square_pipeline.rs` | 129 | 0 | 7 / 0 | The full pipeline at non-square grid dimensions. |
| `tests/thread_pool_setter_applies_first.rs` | 49 | 0 | 1 / 0 | The other half of `thread_pool_setter_honesty.rs`: `set_configured_thread_ |
| `tests/thread_pool_setter_honesty.rs` | 67 | 0 | 2 / 0 | `set_configured_thread_count` must not claim a pool it did not build. |
| `tests/vector_features_real_world.rs` | 298 | 0 | 6 / 2 | EF-6's three tracers against a **real generated world** rather than a |
| `tests/volcano_record.rs` | 111 | 0 | 3 / 0 | VA-1 of `VOLCANO_APPEARANCE_RESEARCH.md` (owner Ruling CW, 2026-10-24): |
| `tests/world_structure_orogeny.rs` | 119 | 0 | 4 / 0 | Ruling AS (2026-09-24): with World Structure on, the orogeny knobs are the |

Ignored harnesses (8): `src/erode_op.rs::measured_thermal_gpu_vs_cpu`, `src/gpu_divergence_bench.rs::gpu_divergence_bench`, `src/lib.rs::measured_generate_terrain_gpu_vs_cpu_timing`, `src/lib.rs::measured_generate_terrain_reuses_the_gpu_device_across_calls`, `src/lib.rs::measured_stress_gpu_vs_cpu`, `src/staleness.rs::measured_recompute_stale_vs_a_full_generate`, `tests/bake_real_world.rs::a_real_world_bakes_persists_and_round_trips`, `tests/ef3_tile_erosion.rs::measure_tile_erosion_cost`.

### `cartalith-erosion`

Depends on: `cartalith-jsmath`, `cartalith-noise`, `cartalith-rng`, `cartalith-terrain`. Used by: `cartalith-civ`, `cartalith-engine`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `src/carve_width.rs` | 287 | 4 | 4 / 0 | **GF-6's rock-dependent river carve** (`GEOLOGY_FIRST_SCOPE.md` §4.7, §7 |
| `src/construction.rs` | 1233 | 9 | 10 / 0 | GF-11: rock-aware height construction (`GEOLOGY_FIRST_SCOPE.md` §4.14, |
| `src/karst.rs` | 362 | 5 | 6 / 0 | GF-5: karst dissolution, permeability-reduced runoff and the spring line |
| `src/lib.rs` | 2197 | 23 | 8 / 0 | droplet, stream-power, thermal erosion |
| `src/passes.rs` | 1861 | 15 | 22 / 0 | The reference's **manual** erosion passes — the ones `generate()` never |
| `src/tile.rs` | 998 | 4 | 8 / 0 | Tile-bounded erosion — EF-3 of `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`. |
| `tests/gf2_rock_stream_power.rs` | 243 | 0 | 7 / 0 | GF-2 of `GEOLOGY_FIRST_SCOPE.md` (owner Rulings BH and BJ): stream power |
| `tests/gf3_threshold_hillslope.rs` | 281 | 0 | 11 / 0 | GF-3 of `GEOLOGY_FIRST_SCOPE.md` (owner Rulings BH and BJ): the |
| `tests/golden_parity_droplet.rs` | 51 | 0 | 2 / 0 | Golden-parity test for droplet_kernel (PARITY_TESTING.md). |
| `tests/golden_parity_passes.rs` | 611 | 0 | 26 / 0 | Golden-parity tests for the reference's manual erosion passes |
| `tests/golden_parity_rebound.rs` | 37 | 0 | 2 / 0 | Golden-parity test for isostatic_rebound (PARITY_TESTING.md). |
| `tests/golden_parity_streampower.rs` | 60 | 0 | 2 / 0 | Golden-parity test for stream_power_kernel (PARITY_TESTING.md). |
| `tests/golden_parity_thermal.rs` | 36 | 0 | 2 / 0 | Golden-parity test for erode_thermal (PARITY_TESTING.md). |
| `tests/stream_power_refresh.rs` | 260 | 0 | 5 / 0 | The stream-power routing refresh (`GEOLOGY_FIRST_SCOPE.md` §5.11): the |

### `cartalith-godot`

Depends on: `cartalith-assets`, `cartalith-civ`, `cartalith-climate`, `cartalith-engine`, `cartalith-gpu`, `cartalith-hydrology`, `cartalith-io`, `cartalith-jsmath`, `cartalith-knowledge`, `cartalith-llm`, `cartalith-noise`, `cartalith-rng`, `cartalith-spatial`, `cartalith-terrain`, `cartalith-vault`. Used by: nothing (a leaf consumer).

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `src/adjust.rs` | 1121 | 13 | 19 / 0 | Per-layer, non-destructive **adjustment stacks** (`EDITING_TOOLS_SCOPE.md` |
| `src/annotation_bridge.rs` | 333 | 13 | 1 / 0 | `WorldGen`'s half of the annotation undo (`annotation_undo.rs` holds the |
| `src/annotation_undo.rs` | 578 | 28 | 8 / 0 | Undo and redo for the hand-edited annotation stores: map labels, placed |
| `src/asset_bridge.rs` | 2980 | 38 | 51 / 0 | The Asset Library's mutable, in-memory session: `GUI_GAP_REGISTER.md` |
| `src/asset_bridge_ap89_tests.rs` | 376 | 0 | 10 / 0 | AP-8 (folder import) and AP-9 (pack diff and merge) at the session level: |
| `src/bake_bridge.rs` | 664 | 9 | 14 / 0 | The bake/atlas/finalize system's shell-side state — `GUI_GAP_REGISTER.md` |
| `src/bugsweep_seed707.rs` | 207 | 0 | 1 / 1 | Found-bug sweep 2026-10-11, item R8: an **ignored measurement**, not a |
| `src/campaign_bridge.rs` | 126 | 0 | 0 / 0 | Ruling AW -- the CARTO ▸ Conflict layer's data, over |
| `src/civ_military_bridge.rs` | 1486 | 8 | 4 / 0 | CIVIL ▸ Military and CIVIL ▸ Relationships — `GUI_GAP_REGISTER.md` |
| `src/civ_religion_bridge.rs` | 555 | 8 | 4 / 0 | The custom-religion library's `#[func]` surface -- `FACTION_HUB_DESIGN.md` |
| `src/civ_roster_bridge.rs` | 1352 | 34 | 24 / 0 | The mutable civ **roster** state — `PARITY_AUDIT.md` §5 items 2, 3, 7, 9 |
| `src/civ_tools_bridge.rs` | 965 | 14 | 31 / 0 | The CIVIL tool group's Godot-facing bridge state — `UNIFIED_TOOL_PLAN.md` |
| `src/civ_trade_bridge.rs` | 1005 | 2 | 0 / 0 | CIVIL ▸ Trade — `GUI_GAP_REGISTER.md` **IN-13**, plus `ECONOMY_SCOPE.md` |
| `src/colour_import.rs` | 645 | 10 | 0 / 0 | Colour-managed PNG import for **asset images** -- `EDITING_TOOLS_SCOPE.md` |
| `src/colour_import_tests.rs` | 704 | 0 | 23 / 0 | Tests for `pack::colour_import` (ET-E2), compiled by `lib.rs` only: they |
| `src/colour_paint_bridge.rs` | 423 | 7 | 0 / 0 | Ruling CN's colour-paint layer on `WorldGen` (`COLOUR_PAINT_SCOPE.md` |
| `src/conflict_bridge.rs` | 564 | 4 | 3 / 0 | `STORY_PLANNING_SCOPE.md` SP-4 -- the conflict overlay's Godot surface |
| `src/context_pick_bridge.rs` | 2125 | 47 | 32 / 0 | CM-7's two engine rows (`MAP_CONTEXT_SCOPE.md` §9.2; Ruling BA, |
| `src/device_stage_bench.rs` | 320 | 0 | 1 / 2 | Per-stage New World generation timing, run on the PC and on Android |
| `src/ef9_bridge.rs` | 623 | 0 | 3 / 0 | **EF-9.4** -- importance-driven refinement, wired to the viewport |
| `src/erode_bridge.rs` | 695 | 1 | 8 / 0 | The reference's `erode()` op — `PARITY_AUDIT.md` §23 F11 — and the |
| `src/export_options.rs` | 454 | 22 | 0 / 0 | The export options (`EXPORT_SCOPE.md` §7, milestone E3): one options value |
| `src/export_raster.rs` | 4376 | 13 | 38 / 1 | The export raster and the channel atlas — `PARITY_AUDIT.md` §5 item 14's |
| `src/export_session.rs` | 2050 | 16 | 33 / 0 | The export overlay session's Godot-free core (`EXPORT_SCOPE.md` §7, |
| `src/export_stream.rs` | 894 | 15 | 0 / 0 | The streaming export writer (`EXPORT_SCOPE.md` §7, milestone E2): band in, |
| `src/fh_r2_baseline.rs` | 1215 | 4 | 7 / 7 | FH-R2 milestone R2-0's **M0 baseline** and R2-1's **M1 "zero bytes moved"** |
| `src/g1_baseline.rs` | 1010 | 0 | 4 / 4 | G1 governments: milestone **G1-0's measurement columns** and the bars of |
| `src/geojson_apply.rs` | 893 | 7 | 14 / 0 | Applying an imported GeoJSON document to a live world — the other half of |
| `src/geojson_bridge.rs` | 599 | 0 | 1 / 0 | `exportGeoJSON`'s Godot boundary, and the import direction alongside it |
| `src/geology_save.rs` | 456 | 5 | 0 / 0 | The geology column in a project archive (`SAVEFILE_COMPAT.md` §8.4, |
| `src/icon_bridge.rs` | 1938 | 25 | 66 / 0 | The Icon tool's Godot-facing bridge state — `UNIFIED_TOOL_PLAN.md` |
| `src/icon_bridge/brush.rs` | 144 | 0 | 0 / 0 | CARTO ▸ Icon — the **density brush**'s Godot surface. |
| `src/icon_bridge/generate.rs` | 522 | 1 | 4 / 0 | CARTO ▸ Icons — the generated placement pass's Godot surface. |
| `src/icon_bridge/pick.rs` | 27 | 0 | 0 / 0 | The map context request's read-only icon pick (`MAP_CONTEXT_SCOPE.md` |
| `src/info_tile.rs` | 536 | 6 | 8 / 0 | Deep-zoom tiles for the **info views** (LOD-D7, Ruling AV, 2026-09-24). |
| `src/infra_tools_bridge.rs` | 2371 | 50 | 51 / 0 | The INFRA domain's Way/Route tools, plus the two global tools that ride |
| `src/journey_bridge.rs` | 1253 | 16 | 29 / 0 | The Journey Planner's Godot-facing bridge — `JOURNEY_PLANNER_SCOPE.md`'s |
| `src/knowledge_bridge.rs` | 1164 | 1 | 12 / 4 | The world as retrievable documents: this crate's half of `RAG_KNOWLEDGE_BASE_RESEARCH.md` |
| `src/knowledge_service.rs` | 391 | 7 | 5 / 0 | The knowledge base's runtime: holds the index the assistant's knowledge tools and the |
| `src/label_bridge.rs` | 1735 | 29 | 54 / 1 | The Label tool's Godot-facing bridge state — `UNIFIED_TOOL_PLAN.md` |
| `src/label_bridge/generate.rs` | 533 | 0 | 5 / 0 | CARTO ▸ Labels — the generated labelling pass's Godot surface. |
| `src/label_bridge/pick.rs` | 29 | 0 | 0 / 0 | The map context request's read-only label pick (`MAP_CONTEXT_SCOPE.md` |
| `src/landmark_bridge.rs` | 318 | 7 | 9 / 0 | The Landmark Generation dock's thin `Variant`-adjacent helpers over |
| `src/landmark_bridge/rejects.rs` | 233 | 0 | 3 / 0 | CIVIL ▸ Landmarks — the rejected-candidate list's Godot surface. |
| `src/legacy_import.rs` | 305 | 1 | 4 / 0 | Owner **Ruling AU** (`LARGE_ITEM_RULINGS.md`, 2026-09-24): a legacy flat |
| `src/lib.rs` | 29664 | 22 | 109 / 9 | Boundary layer between Godot and the engine crates (ARCHITECTURE.md). |
| `src/llm_bridge.rs` | 534 | 5 | 0 / 0 | The LLM assistant's settings surface for the Godot shell |
| `src/llm_keystore.rs` | 226 | 3 | 2 / 0 | The OS credential store behind the LLM assistant's API key |
| `src/llm_keystore_android.rs` | 592 | 8 | 3 / 0 | The Android half of the LLM assistant's key store (`LLM_ASSISTANT_SCOPE.md` |
| `src/llm_models.rs` | 220 | 0 | 0 / 0 | The model list for the settings dropdown, for the Godot shell (the owner's 2026-10-26 |
| `src/llm_stream.rs` | 1975 | 0 | 0 / 0 | One streamed assistant reply, for the Godot shell (`LLM_ASSISTANT_SCOPE.md` |
| `src/lod_bridge.rs` | 3369 | 23 | 47 / 7 | Interactive per-tile deep-zoom synthesis — `LOD_TILING_INTEGRATION_SCOPE.md` |
| `src/lod_sweep.rs` | 1140 | 9 | 21 / 0 | LOD-D0's zoom-sweep metrics — `LOD_DETAIL_SCOPE.md`, "LOD-D0 · Zoom-sweep |
| `src/lod_worker.rs` | 3575 | 38 | 33 / 0 | **`LOD_DETAIL_SCOPE.md` LOD-D6 — tile synthesis off the main thread, and |
| `src/measure_bridge.rs` | 1185 | 6 | 29 / 0 | The measurement toolbar's world-reading half — |
| `src/ops_bridge.rs` | 439 | 0 | 0 / 0 | Ported-and-unexposed engine capability — `PARITY_AUDIT.md` §23 F13. |
| `src/pack.rs` | 1539 | 18 | 0 / 0 | Real asset-pack loading and sprite compositing — `ASSET_LIBRARY_SCOPE.md` |
| `src/paint_bridge.rs` | 2360 | 35 | 45 / 0 | The Paint editor's Godot-facing bridge state — `UNIFIED_TOOL_PLAN.md` |
| `src/param_catalogue.rs` | 357 | 5 | 0 / 0 | The generation-parameter catalogue for the language-model assistant |
| `src/params.rs` | 1884 | 14 | 0 / 0 | The generation-parameter table: one flat, dotted-key namespace over |
| `src/poi_bridge.rs` | 530 | 4 | 6 / 0 | `WorldGen`'s user-placed points of interest (`CITY_MAP_LOOK_SCOPE.md` CL-11, Ruling EW item 4: *"Yes, user-placed |
| `src/print_gamut.rs` | 660 | 6 | 1 / 0 | The print (CMYK) gamut check behind the colour picker's warning -- |
| `src/print_gamut_tests.rs` | 221 | 0 | 10 / 0 | Tests for `print_gamut.rs` (ET-E3). Kept in their own file so the module |
| `src/progress_bridge.rs` | 73 | 0 | 0 / 0 | A stateless, separately-instantiated read-only window onto |
| `src/project_bridge.rs` | 11323 | 3 | 126 / 0 | The project archive's Godot surface — `SAVEFILE_COMPAT.md`, owner |
| `src/region_edit_bridge.rs` | 327 | 0 | 0 / 0 | The engine half of the assistant's edit actions (`LLM_ASSISTANT_SCOPE.md` s.5c, milestone |
| `src/render.rs` | 15912 | 248 | 18 / 0 | Real default-settings 2D map rendering, ported from the reference HTML's |
| `src/river_delta.rs` | 2113 | 12 | 23 / 0 | **River deltas on the painted path** (`RIVERS_IN_MAP_SCOPE.md` RIM-4 and |
| `src/river_field.rs` | 2003 | 23 | 32 / 0 | **RIM-1: the river as water in the map's own per-pixel field** |
| `src/river_meander.rs` | 269 | 2 | 6 / 0 | **The display meander: a bounded wander on the drawn river for close zoom** |
| `src/river_realism.rs` | 838 | 20 | 12 / 0 | **River realism (RR, `RIVER_REALISM_SCOPE.md`; owner Ruling ES, 2026-11-12).** |
| `src/river_straight.rs` | 303 | 2 | 3 / 0 | **Straight river runs, measured** (owner, 2026-11-11). A diagnostic for the probe (`WorldGen::river_straightness`); |
| `src/river_stroke.rs` | 4215 | 51 | 44 / 0 | RV-2 -- the drawn river stroke's geometry (`OUTSTANDING_WORK.md`, "RV-2, |
| `src/rock_detail_bridge.rs` | 183 | 2 | 0 / 0 | **GF-6 §4.10** -- rock-styled deep-zoom detail, wired to the world |
| `src/sample_bridge.rs` | 4365 | 37 | 38 / 6 | Per-cell field sampling and the debug-view raster palette — |
| `src/sculpt_bridge.rs` | 1526 | 19 | 33 / 0 | The Sculpt editor's Godot-facing bridge state — `UNIFIED_TOOL_PLAN.md` |
| `src/sea_live_bridge.rs` | 438 | 5 | 5 / 0 | **Live sea level** — Ruling AQ (`LARGE_ITEM_RULINGS.md`, 2026-09-24), with |
| `src/selection.rs` | 390 | 13 | 15 / 0 | One selection **set** per entity kind — step one of the owner's ruling in |
| `src/story_bridge.rs` | 1159 | 7 | 4 / 0 | `STORY_PLANNING_SCOPE.md` SP-2 -- journey progression over the year |
| `src/substrate.rs` | 416 | 4 | 1 / 0 | The world substrate in a project archive (`SAVEFILE_COMPAT.md` §8.3, owner |
| `src/timeline_bridge.rs` | 1059 | 4 | 18 / 0 | `TIMELINE_SCOPE.md` §5 milestone 5 -- the Godot boundary for the mechanistic |
| `src/town_river_census.rs` | 2486 | 0 | 6 / 7 | The town-on-its-river census for the owner's report of 2026-11-14 |
| `src/travel_bridge.rs` | 1802 | 33 | 28 / 0 | The Travel Library's mutable store: `TRAVEL_LIBRARY_SPEC.md`'s |
| `src/undo.rs` | 1594 | 38 | 42 / 0 | Global heightmap undo — the reference's `pushUndo`/`undoLast`/ |
| `src/undo_group_bridge.rs` | 298 | 3 | 6 / 0 | LLM-5 (`LLM_ASSISTANT_SCOPE.md` s.3, s.5b.4): the two read-only engine |
| `src/urban_bridge.rs` | 1241 | 1 | 0 / 0 | Urban morphology's Godot-facing bridge — the first consumer |
| `src/valley_shade.rs` | 904 | 2 | 19 / 0 | **RV-3: the valley the map shades is cut along the drawn river line** |
| `src/vault_bridge.rs` | 3193 | 9 | 10 / 0 | The Markdown Vault's Godot surface (`MARKDOWN_VAULT_SCOPE.md` milestone 1). |
| `src/vault_saf.rs` | 313 | 1 | 1 / 0 | The Android half of `MARKDOWN_VAULT_SCOPE.md` milestone 4 — the Storage |
| `src/volcano_look.rs` | 1248 | 16 | 6 / 0 | Volcano colours for the Satellite relief look (VA-2, VA-3, VA-4 of |
| `src/wall_gap_census.rs` | 155 | 0 | 1 / 2 | The wall-gap census for owner Ruling EW item 1, last point (2026-11-13, |
| `src/way_census.rs` | 1188 | 14 | 14 / 11 | The road-network census for owner Ruling EU item 1 (2026-11-12): how much |
| `src/way_join_census.rs` | 527 | 0 | 2 / 3 | The planner census for owner Ruling EU items 2 and 3 (2026-11-13): how a |
| `src/world_summary.rs` | 129 | 1 | 0 / 0 | Whole-world summary statistics for the assistant's `get_world_summary` tool |
| `tests/ac2_actions.rs` | 1229 | 0 | 26 / 0 | AC-1 and AC-2 over the REAL rows (`ASSISTANT_ACTIONS_PROPOSAL.md` s.4; Ruling DW): the action |
| `tests/ac2_live.rs` | 243 | 0 | 1 / 2 | AC-2's model-in-the-loop CHECK with the REAL local model (owner rule: no mock models for new checks): |
| `tests/ac4_live.rs` | 299 | 0 | 1 / 2 | AC-4's model-in-the-loop SCORE (`ASSISTANT_ACTIONS_PROPOSAL.md` AC-4: "pre-retrieval of actions raises |
| `tests/appearance_ab_dump.rs` | 375 | 0 | 3 / 5 | Deterministic A/B appearance dump — `TERRAIN_APPEARANCE_RESEARCH.md` |
| `tests/appearance_tiers.rs` | 1536 | 0 | 40 / 1 | `TERRAIN_APPEARANCE_SCOPE.md` milestone 6 — the §29 quality-tier ladder and |
| `tests/bake_raster.rs` | 625 | 0 | 13 / 0 | The export raster (`bakeDims`/`bakePixel`/`bakeSingle`/`bakeTiled`, |
| `tests/balanced_lake_shore.rs` | 135 | 0 | 4 / 0 | **A lake the water balance made has a smooth shore** (Ruling ET's balance, RV-4's shoreline; verifier finding, |
| `tests/base_lake_stamp.rs` | 70 | 0 | 2 / 0 | `RenderCtx::with_lakes` -- the reference's v0.103 base-map lake stamp |
| `tests/canopy_and_mottle.rs` | 1072 | 0 | 29 / 0 | **Ruling CI, the owner's order after mountain relief** (2026-10-18): the |
| `tests/cel_toon.rs` | 680 | 0 | 18 / 0 | The **Cel / Toon** style (owner, 2026-09-27: *"cel shading for a bit of a |
| `tests/color_space.rs` | 486 | 0 | 12 / 0 | The output colour space on the render target (`LARGE_ITEM_RULINGS.md`, |
| `tests/colour_paint.rs` | 740 | 0 | 17 / 0 | Ruling CN's colour-paint layer in the renderer (`COLOUR_PAINT_SCOPE.md` |
| `tests/colour_paint_frames.rs` | 1165 | 0 | 8 / 9 | Ruling CN's measurement harnesses (`COLOUR_PAINT_SCOPE.md` CP-1): the |
| `tests/edifice_furrow_tiles.rs` | 147 | 0 | 3 / 0 | Ruling CO item 3 (owner, 2026-10-18): the deep-zoom tiles' volcano flank |
| `tests/export_bands.rs` | 299 | 0 | 6 / 0 | The banded export renderer (`EXPORT_SCOPE.md` §4, milestone E1) — |
| `tests/export_options.rs` | 335 | 0 | 14 / 0 | The export options (`EXPORT_SCOPE.md` §7, milestone E3) — the Godot-free |
| `tests/export_stream.rs` | 1003 | 0 | 17 / 2 | The streaming export writer (`EXPORT_SCOPE.md` §7, milestone E2) — |
| `tests/feature_icon_overlay.rs` | 316 | 0 | 9 / 0 | AP-5 (`ASSET_PACK_TOOLS_PROPOSAL.md`, Ruling CQ Q3): the scattered feature |
| `tests/fixtures/tile_biome_fixture.rs` | 55 | 0 | 0 / 0 | GENERATED by `cartalith-native/tools/tile_biome_capture.js` -- do not hand-edit. |
| `tests/geology_micro_and_sky_fields.rs` | 309 | 0 | 8 / 0 | **The two `OUTSTANDING_WORK.md` §2.5 rows ported 2026-09-03, second pass**: |
| `tests/gf0_geology_harness.rs` | 5323 | 0 | 29 / 20 | **GF-0: the geology-first measurement harness** (`GEOLOGY_FIRST_SCOPE.md` |
| `tests/gf4_glacial_coastal.rs` | 485 | 0 | 2 / 2 | GF-4's measurement (`GEOLOGY_FIRST_SCOPE.md` §7 GF-4, bars B6, B7, B8 and |
| `tests/gf5_karst.rs` | 302 | 0 | 2 / 2 | GF-5's measurement (`GEOLOGY_FIRST_SCOPE.md` §7 GF-5, bars B5 and B8 of |
| `tests/gf6_rock_detail.rs` | 372 | 0 | 10 / 0 | **GF-6 §4.10: rock-styled deep-zoom detail** (`GEOLOGY_FIRST_SCOPE.md` |
| `tests/gf6_rock_ladder.rs` | 244 | 0 | 1 / 2 | **GF-6 §4.10's measurement harness** (`GEOLOGY_FIRST_SCOPE.md` §4.10, §7 |
| `tests/golden_parity_npr.rs` | 225 | 0 | 5 / 0 | Golden-parity test for the reference's **NPR block** (`GUI_GAP_REGISTER.md` |
| `tests/golden_parity_render.rs` | 100 | 0 | 2 / 0 | Golden-parity test for `render::cell_color` (`PARITY_TESTING.md`). |
| `tests/golden_parity_tile_biome.rs` | 969 | 0 | 13 / 2 | `render::render_biome_tile_rgba` — the LOD/atlas biome tile |
| `tests/hatch_shade_diag.rs` | 222 | 0 | 3 / 2 | Measures: the dark diagonal hatching in the base texture (`OUTSTANDING_WORK.md`, |
| `tests/lake_balance_generation.rs` | 147 | 0 | 2 / 1 | **Ruling ET: the lake water balance in the generation path** |
| `tests/lake_balance_harness.rs` | 209 | 0 | 1 / 1 | **RR-6: which lakes a water balance would make appear or vanish -- measured, |
| `tests/lake_candidates_harness.rs` | 725 | 0 | 1 / 1 | **Lake-fill candidates 1 and 2, measured and drawn side by side** (Ruling CA |
| `tests/lake_floor_measurement.rs` | 222 | 0 | 1 / 1 | **Ruling CC's lake basin-area floor, measured on the lake-candidates |
| `tests/lake_runoff_measurement.rs` | 158 | 0 | 1 / 1 | **Lake area before and after the runoff share** (owner choice 2026-11-13, *"Runoff share of rain"*; |
| `tests/lattice_diag.rs` | 219 | 0 | 2 / 4 | SCRATCH diagnosis harness for the "diamond lattice" row. Not a regression test. |
| `tests/lava_flows.rs` | 1229 | 0 | 16 / 10 | **Terrain-following lava flows** (owner Ruling DA step 1, 2026-10-25; |
| `tests/layer_stack.rs` | 683 | 0 | 14 / 5 | `GUI_GAP_REGISTER.md` CA-03 / CA-04 and RD-10's precondition — the raster's |
| `tests/litho_blend.rs` | 169 | 0 | 5 / 0 | The lithology corner blend (Ruling CG addendum 2, owner, 2026-10-12; |
| `tests/llm7_corpus.rs` | 388 | 0 | 4 / 0 | LLM-7: the hostile-string corpus for EDIT requests, run as a regression gate |
| `tests/llm9_corpus.rs` | 939 | 0 | 8 / 0 | LLM-9: the hostile-string corpus, run as a regression gate over the REAL registry rows |
| `tests/lod_d4_ice_and_snow.rs` | 1273 | 0 | 19 / 6 | `LOD_DETAIL_SCOPE.md` **LOD-D4** — ice and snow from fields that already |
| `tests/lod_d5_scale_aware.rs` | 855 | 0 | 14 / 0 | **LOD-D5** (`LOD_DETAIL_SCOPE.md`) — *"scale-aware shading weights, and |
| `tests/mountain_relief.rs` | 498 | 0 | 2 / 5 | **Ruling CU item 1** (`LARGE_ITEM_RULINGS.md`, owner, 2026-10-19) -- the |
| `tests/nonsquare.rs` | 264 | 0 | 7 / 3 | Non-square grids, end to end: generation, rendering, and sprite |
| `tests/nwprev_cost.rs` | 466 | 0 | 3 / 4 | **New World seed preview: acceptance harness** (`NEW_WORLD_PREVIEW_PROPOSAL.md` |
| `tests/pack_compositing.rs` | 174 | 0 | 3 / 0 | Real integration test for `ASSET_LIBRARY_SCOPE.md` milestone 7's sprite |
| `tests/pack_marker_art.rs` | 197 | 0 | 7 / 0 | AP-3 (`ASSET_PACK_TOOLS_PROPOSAL.md` §6, Ruling CQ Q2): the engine half of |
| `tests/pack_trait_badges.rs` | 370 | 0 | 13 / 0 | `OUTSTANDING_WORK.md` §2.5, sprite half: compositing an imported pack's own |
| `tests/paint_blend.rs` | 602 | 0 | 19 / 0 | `landColorCore`'s paint-brush Biome/Terrain tint (reference HTML |
| `tests/paint_preview_cost.rs` | 667 | 0 | 9 / 5 | `OUTSTANDING_WORK.md` §2.6 — *"Previews re-upload the whole texture, |
| `tests/param_catalogue.rs` | 535 | 0 | 15 / 0 | LLM-6a and LLM-6b over the REAL registry rows (`LLM_ASSISTANT_SCOPE.md` |
| `tests/params_mapping.rs` | 1819 | 0 | 55 / 0 | Unit tests for the generation-parameter table (`src/params.rs`) — the |
| `tests/pass_relief_measure.rs` | 130 | 0 | 1 / 0 | `DECISIONS.md` §7i, measured on **real generated terrain** rather than a |
| `tests/planner_audit_journey.rs` | 676 | 0 | 6 / 6 | Journey-planner route audit (2026-10-11): measurement harness, **ignored**. |
| `tests/planner_audit_rivers.rs` | 153 | 0 | 1 / 1 | Ruling CD batch 2 (2026-10-11): River Transport on a real traced river, |
| `tests/planner_audit_slope.rs` | 269 | 0 | 2 / 2 | Ruling CD batch 3 (2026-10-11): slope in planned land time, and Track's own |
| `tests/playa_and_lake_depth.rs` | 339 | 0 | 12 / 0 | Ruling ET's playa follow-up and RR-5 (Ruling ES): the salt-flat tint of a |
| `tests/project_document_channel.rs` | 425 | 0 | 6 / 0 | The caller-owned document channel, end to end |
| `tests/project_round_trip.rs` | 367 | 0 | 5 / 2 | The project archive, end to end over a **real** generated world |
| `tests/region_edit_limits.rs` | 149 | 0 | 2 / 0 | LLM-7's engine bars that need no Godot (`LLM_ASSISTANT_SCOPE.md` s.5c): the model-facing limits |
| `tests/river_basin_routing_measurement.rs` | 473 | 0 | 2 / 2 | **Ruling EV: river combs before and after basin routing, measured on the full pipeline** |
| `tests/river_comb_harness.rs` | 860 | 0 | 7 / 7 | **River comb thinning, measured** (Ruling BW item 3, 2026-10-10: draw-side |
| `tests/river_entities.rs` | 335 | 0 | 6 / 0 | The river entity on a **real generated world** — `OUTSTANDING_WORK.md` §2.2. |
| `tests/river_gap_harness.rs` | 1188 | 0 | 4 / 3 | **River-gap measurement** -- `OUTSTANDING_WORK.md` §2.3, "Rivers render as |
| `tests/river_gate_measurement.rs` | 1224 | 0 | 2 / 2 | **The two river gates, measured before anything changes** (owner, Ruling |
| `tests/river_lake_followup_measurement.rs` | 181 | 0 | 1 / 1 | **Ruling ES / ET follow-ups, measured: the playa tint, RR-5's lake tone by |
| `tests/river_realism_measurement.rs` | 272 | 0 | 1 / 1 | **RR-0: river realism measured before anything is pinned** |
| `tests/river_routing_measurement.rs` | 397 | 0 | 1 / 1 | **Ruling ER: which flow routing breaks the straight river runs?** |
| `tests/river_zoom_measurement.rs` | 334 | 0 | 1 / 1 | **RZ-0: the three visibility keys, measured before anything is pinned** |
| `tests/rk7_knowledge_hardening.rs` | 462 | 0 | 6 / 0 | RK-7: injection and privacy hardening of the knowledge tools and the pre-retrieval pack |
| `tests/ruling_bi_style_presets.rs` | 195 | 0 | 6 / 0 | **Ruling BI** (`LARGE_ITEM_RULINGS.md`, 2026-09-27) — the eight researched |
| `tests/satellite_relief.rs` | 656 | 0 | 21 / 0 | **The "Satellite relief" look** (Ruling CI item 2, owner 2026-10-17, and its |
| `tests/save_round_trip.rs` | 185 | 0 | 2 / 0 | The end-to-end save round trip (`SAVEFILE_COMPAT.md`, `GUI_GAP_REGISTER.md` |
| `tests/sculpt_live_l0_bench.rs` | 503 | 0 | 1 / 2 | `SCULPT_LIVE_SCOPE.md` milestone **L0** — measure, don't optimise. |
| `tests/sdf_river_and_biome.rs` | 606 | 0 | 10 / 0 | **The B3/B4 legs of the reference's SDF trio** (`OUTSTANDING_WORK.md` §2.5, |
| `tests/slope_rock_and_wetness.rs` | 113 | 0 | 2 / 0 | **The constants of the two reference viz stages ported 2026-09-03** |
| `tests/sprite_filter.rs` | 358 | 0 | 3 / 0 | Ruling CR's switched sprite-chain fix (2026-10-18, last paragraph: |
| `tests/tanaka_contours.rs` | 448 | 0 | 16 / 0 | Tanaka illuminated contours (Ruling BI, `MAP_STYLE_RESEARCH.md` section 2.2). |
| `tests/terrain_streak_spike.rs` | 1119 | 0 | 5 / 3 | The terrain-streak measurement spike (`TERRAIN_STREAK_SPIKE_2026-10-18.md` |
| `tests/tile_shade_exag.rs` | 256 | 0 | 4 / 0 | v2.25's `tileShadeExag(bounds, W)` (`RC_ENGINE_CHANGES.md` §4, followed |
| `tests/village_bands.rs` | 125 | 0 | 2 / 0 | `OUTSTANDING_WORK.md` §2.11, "The Village carto preset still looks harsh |
| `tests/volcano_appearance.rs` | 1189 | 0 | 15 / 2 | **Volcano appearance** (`VOLCANO_APPEARANCE_RESEARCH.md` §7, VA-0 to VA-4; |
| `tests/world_summary.rs` | 144 | 0 | 5 / 0 | LLM-8's `get_world_summary` figures against an independent recount |
| `tests/zoom_detail_ladder.rs` | 850 | 0 | 4 / 9 | **Ruling CI item 1** (`LARGE_ITEM_RULINGS.md`, owner, 2026-10-17) -- the |
| `tests/zr0_refine.rs` | 1714 | 0 | 3 / 7 | **ZR-0 of `ZOOM_REFINEMENT_SCOPE.md`** (owner Ruling FH, 2026-11-16: "start |

Ignored harnesses (147): `src/bugsweep_seed707.rs::bugsweep_seed707_sea_route`, `src/device_stage_bench.rs::device_stage_bench`, `src/export_raster.rs::bench_finish_rgb_stack_cost`, `src/fh_r2_baseline.rs::fh_r2_m1_moves_zero_bytes_against_m0`, `src/fh_r2_baseline.rs::fh_r2_m2_belief_cost`, `src/fh_r2_baseline.rs::fh_r2_m2_coexistence_bars`, `src/fh_r2_baseline.rs::fh_r2_m3_rite_only`, `src/fh_r2_baseline.rs::fh_r2_m5_custom_axes`, `src/fh_r2_baseline.rs::fh_r2_m6_policy_bars`, `src/fh_r2_baseline.rs::fh_r2_urban_breakdown`, `src/g1_baseline.rs::ea0_ecological_ordering_diagnostic`, `src/g1_baseline.rs::g1_0_baseline_columns`, `src/g1_baseline.rs::g1_comparison_report`, `src/g1_baseline.rs::g1_forced_forms_are_ordered_and_near_their_anchor`, `src/knowledge_bridge.rs::measure_knowledge_render_and_index_at_the_default_grid`, `src/knowledge_bridge.rs::regenerate_knowledge_fixtures`, `src/label_bridge.rs::measure_generated_overlap_before_and_after`, `src/lib.rs::landmark_dict_carries_every_contract_key_and_omits_seed`, `src/lib.rs::landmark_funnel_dict_carries_every_contract_key`, `src/lib.rs::landmark_settings_dict_carries_every_contract_key`, `src/lod_bridge.rs::deep_tile_cost_by_level`, `src/lod_bridge.rs::measure_a_stored_pyramid`, `src/lod_bridge.rs::the_tile_context_costs_an_order_of_magnitude_more_than_the_tile_it_serves`, `src/sample_bridge.rs::timing_probe_fingerprint`, `src/sample_bridge.rs::timing_probe_jp_compute_core`, `src/sample_bridge.rs::timing_probe_wildlife_regions`, `src/town_river_census.rs::both_banks_audit`, `src/town_river_census.rs::harbour_front_audit`, `src/town_river_census.rs::riverbank_gap_audit`, `src/town_river_census.rs::town_housing_audit`, `src/town_river_census.rs::town_placement_audit`, `src/town_river_census.rs::town_river_census`, `src/wall_gap_census.rs::wall_gap_before_and_after`, `src/way_census.rs::diagnose_remaining_overlaps`, `src/way_census.rs::diagnose_residual_bundle_pairs`, `src/way_census.rs::diagnose_track_detours`, `src/way_census.rs::measure_landmarks_on_merged_ways`, `src/way_census.rs::measure_track_detours`, `src/way_census.rs::measure_way_bundles`, `src/way_census.rs::measure_way_bundles_small_worlds`, `src/way_census.rs::measure_way_classes_and_sea_lanes`, `src/way_census.rs::measure_way_merge_cost`, `src/way_census.rs::measure_way_merge_sweep`, `src/way_join_census.rs::find_v_detours`, `src/way_join_census.rs::measure_planner_way_join_and_river_offers`, `tests/ac2_live.rs::live_runs_with_the_real_local_model`, `tests/ac4_live.rs::live_score_on_ac0s_thirty_tasks`, `tests/appearance_ab_dump.rs::dump_ab_classic_and_archipelago`, `tests/appearance_ab_dump.rs::hydro_wetness_visibility_by_resolution`, `tests/colour_paint_frames.rs::colour_paint_fill_frames`, `tests/colour_paint_frames.rs::colour_paint_flatten_cost`, `tests/colour_paint_frames.rs::colour_paint_frames`, `tests/colour_paint_frames.rs::colour_paint_lava_frames`, `tests/colour_paint_frames.rs::colour_paint_layer_frames`, `tests/colour_paint_frames.rs::colour_paint_profile`, `tests/colour_paint_frames.rs::colour_paint_save_size`, `tests/colour_paint_frames.rs::colour_paint_tile_cost`, `tests/export_stream.rs::streamed_export_peak_memory_harness`, `tests/gf0_geology_harness.rs::gf0_b9_cost`, `tests/gf0_geology_harness.rs::gf0_bars`, `tests/gf0_geology_harness.rs::gf10_b9_cost`, `tests/gf0_geology_harness.rs::gf10_bm_prototype`, `tests/gf0_geology_harness.rs::gf10_small_lake_diag`, `tests/gf0_geology_harness.rs::gf11_b9_cost`, `tests/gf0_geology_harness.rs::gf11_built_stage_full_size`, `tests/gf0_geology_harness.rs::gf11_construction_alone`, `tests/gf0_geology_harness.rs::gf11_g3_lake_diag`, `tests/gf0_geology_harness.rs::gf11_staircase_diag`, `tests/gf0_geology_harness.rs::gf11_stranded_diag`, `tests/gf0_geology_harness.rs::gf2_arms`, `tests/gf0_geology_harness.rs::gf2_b9_cost`, `tests/gf0_geology_harness.rs::gf2_small_lake_diag`, `tests/gf0_geology_harness.rs::gf4_b6_moved`, `tests/gf0_geology_harness.rs::gf7_b9_cost`, `tests/gf0_geology_harness.rs::gf7_clock_sweep`, `tests/gf0_geology_harness.rs::rr_routing_refresh`, `tests/gf0_geology_harness.rs::rr_routing_refresh_cost`, `tests/gf4_glacial_coastal.rs::gf4_bars`, `tests/gf4_glacial_coastal.rs::gf4_coastal_cost`, `tests/gf5_karst.rs::gf5_bars`, `tests/gf5_karst.rs::gf5_identity_hashes`, `tests/gf6_rock_ladder.rs::gf6_rock_ladder`, `tests/golden_parity_tile_biome.rs::tile_synthesis_cost_at_256_squared_is_over_the_scope_budget`, `tests/hatch_shade_diag.rs::hatch_shade_measure`, `tests/lake_balance_generation.rs::lake_balance_remeasure`, `tests/lake_balance_harness.rs::lake_balance_measurement`, `tests/lake_candidates_harness.rs::lake_candidates_measurement`, `tests/lake_floor_measurement.rs::lake_floor_measurement`, `tests/lake_runoff_measurement.rs::lake_area_before_and_after_the_runoff_share`, `tests/lattice_diag.rs::diag_lattice`, `tests/lattice_diag.rs::diag_variants`, `tests/lava_flows.rs::bench_census`, `tests/lava_flows.rs::bench_frames`, `tests/lava_flows.rs::find_young_shield_worlds`, `tests/lava_flows.rs::lava_census`, `tests/lava_flows.rs::summit_ceiling_census`, `tests/lava_flows.rs::summit_ceiling_frames`, `tests/layer_stack.rs::cost_of_the_per_pixel_stack_test`, `tests/layer_stack.rs::print_digests`, `tests/lod_d4_ice_and_snow.rs::measure_the_aletsch_sheet`, `tests/lod_d4_ice_and_snow.rs::measure_the_cost`, `tests/lod_d4_ice_and_snow.rs::measure_the_glacial_default`, `tests/mountain_relief.rs::mountain_relief_cost`, `tests/mountain_relief.rs::mountain_relief_hstats`, `tests/nonsquare.rs::dump_non_square_pngs`, `tests/nwprev_cost.rs::nwprev_cost`, `tests/nwprev_cost.rs::nwprev_generate_identity`, `tests/nwprev_cost.rs::nwprev_overlap`, `tests/paint_preview_cost.rs::paint_preview_cost`, `tests/paint_preview_cost.rs::the_measured_win`, `tests/planner_audit_journey.rs::hidden_generated_way_is_still_credited_through_its_edge`, `tests/planner_audit_journey.rs::q1_land_route_follows_a_hand_drawn_way`, `tests/planner_audit_journey.rs::q1_sea_route_follows_a_sea_lane`, `tests/planner_audit_journey.rs::q3_real_river_channel_is_planned_as_land`, `tests/planner_audit_journey.rs::q3_river_condition_reads_elevation_not_flow`, `tests/planner_audit_journey.rs::q4_sea_leg_wind_and_current_both_directions`, `tests/planner_audit_rivers.rs::q3_real_river_with_the_river_network`, `tests/planner_audit_slope.rs::q1_routes_before_and_after_slope`, `tests/planner_audit_slope.rs::slope_distribution_over_land`, `tests/project_round_trip.rs::measure_a_real_save`, `tests/river_basin_routing_measurement.rs::basin_routing_before_and_after`, `tests/river_basin_routing_measurement.rs::floor_tilt_against_breadth_on_the_final_field`, `tests/river_comb_harness.rs::river_comb_before_after`, `tests/river_comb_harness.rs::river_comb_grid`, `tests/river_comb_harness.rs::river_comb_measurement`, `tests/river_comb_harness.rs::river_comb_pair_pictures`, `tests/river_comb_harness.rs::river_comb_why_not_thinned`, `tests/river_gap_harness.rs::river_gap_bars`, `tests/river_gap_harness.rs::river_gap_measurement`, `tests/river_gate_measurement.rs::river_chain_breaks`, `tests/river_gate_measurement.rs::river_gate_measurement`, `tests/river_lake_followup_measurement.rs::river_lake_followup_measurement`, `tests/river_realism_measurement.rs::river_realism_measurement`, `tests/river_routing_measurement.rs::river_routing_measurement`, `tests/river_zoom_measurement.rs::river_zoom_measurement`, `tests/sculpt_live_l0_bench.rs::l0_measure`, `tests/terrain_streak_spike.rs::streak_spike_cost`, `tests/terrain_streak_spike.rs::streak_spike_measure`, `tests/terrain_streak_spike.rs::streak_spike_render_ablation`, `tests/volcano_appearance.rs::volcano_census`, `tests/zoom_detail_ladder.rs::fg_deep_tile_cost`, `tests/zoom_detail_ladder.rs::furrow_tile_cost`, `tests/zoom_detail_ladder.rs::zoom_detail_ladder`, `tests/zoom_detail_ladder.rs::zoom_detail_tile_cost`, `tests/zr0_refine.rs::zr0_bars`, `tests/zr0_refine.rs::zr0_determinism`, `tests/zr0_refine.rs::zr0_timing`.

### `cartalith-gpu`

Depends on: `cartalith-noise`. Used by: `cartalith-engine`, `cartalith-godot`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `examples/affordance_gpu_compare.rs` | 315 | 0 | 0 / 0 | CPU vs GPU for the four Phase 2 affordance fields (`OUTSTANDING_WORK.md` |
| `examples/flow_downstream_settlements.rs` | 228 | 0 | 0 / 0 | `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 9's real headline measurement. |
| `src/affordance.rs` | 728 | 4 | 3 / 0 | Phase 2 per-cell affordance fields on GPU (`OUTSTANDING_WORK.md` §2.6, |
| `src/lib.rs` | 6506 | 36 | 61 / 2 | GPU compute, grown from the original pilot (`GPU_COMPUTE_PILOT_SCOPE.md`, |
| `src/multi.rs` | 1737 | 46 | 20 / 0 | Multi-GPU: adapter enumeration, device selection, VRAM budgeting, and |
| `src/pool.rs` | 913 | 10 | 14 / 0 | The VRAM buffer pool (`HARDWARE_ACCELERATION.md` §14; `OUTSTANDING_WORK.md` |
| `src/tier.rs` | 277 | 2 | 6 / 0 | `HARDWARE_ACCELERATION.md` §4: the hardware capability tier. |
| `src/timing_harness.rs` | 176 | 11 | 0 / 0 | The timing harness every `measured_*` / `*_measured` test in this crate uses. |
| `tests/affordance.rs` | 206 | 0 | 1 / 0 | The four Phase 2 affordance kernels (`src/affordance.rs`) against the real |
| `tests/multi_gpu.rs` | 609 | 0 | 13 / 0 | Multi-GPU against real hardware (`GUI_GAP_REGISTER.md` PR-01/PR-02/PR-04). |

Ignored harnesses (1): `src/lib.rs::measured_device_handshake_and_per_stage_pipeline_build`.

### `cartalith-hydrology`

Depends on: `cartalith-jsmath`, `cartalith-terrain`. Used by: `cartalith-civ`, `cartalith-engine`, `cartalith-godot`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `src/lib.rs` | 5961 | 35 | 62 / 0 | flow accumulation, river network, channel width |
| `src/rough.rs` | 276 | 4 | 4 / 0 | **Rough routing: sub-grid roughness in the choice of where water goes** |
| `src/tile.rs` | 568 | 6 | 0 / 0 | Tile-bounded hydrology refinement — EF-1 of |
| `tests/golden_parity_flow.rs` | 58 | 0 | 3 / 0 | Golden-parity test for compute_flow (PARITY_TESTING.md, D8 flow accumulation). |
| `tests/golden_parity_polylines.rs` | 127 | 0 | 5 / 0 | Golden-parity tests for trace_river_polylines and enforce_channel_descent |
| `tests/golden_parity_river.rs` | 90 | 0 | 3 / 0 | Golden-parity test for build_channels + strahler_from_receivers (PARITY_TESTING.md). |
| `tests/tile_hydrology.rs` | 760 | 0 | 7 / 0 | EF-1 — tile-bounded hydrology refinement (`cartalith_hydrology::tile`). |

### `cartalith-io`

Depends on: `cartalith-spatial`. Used by: `cartalith-engine`, `cartalith-godot`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `src/atlas.rs` | 1055 | 22 | 18 / 0 | The persistent tile atlas — reference lines 10688-10930, the store the |
| `src/geojson_import.rs` | 1046 | 7 | 30 / 0 | GeoJSON **import** — `FUNCTIONAL_CONTRACT.md` DM-03's other half. |
| `src/gzip.rs` | 135 | 2 | 6 / 0 | gzip for the region export's `.bin` tiles — `UNIFIED_TOOL_PLAN.md` |
| `src/legacy.rs` | 749 | 2 | 5 / 0 | The flat layout's project records, normalised — owner **Ruling AU** |
| `src/lib.rs` | 499 | 2 | 4 / 0 | save/load — the HTML app's .zip save format (SAVEFILE_COMPAT.md) |
| `src/project.rs` | 4366 | 18 | 52 / 0 | The Cartalith **project archive** — reader and writer for the tree |
| `src/save.rs` | 545 | 2 | 11 / 0 | The **interoperability export** — the flat legacy layout |
| `src/slippy.rs` | 441 | 7 | 12 / 0 | Slippy-map addressing over the LOD pyramid — XYZ, TMS and WMTS paths, the |
| `src/tiles.rs` | 627 | 6 | 20 / 0 | Region-export encodings — `UNIFIED_TOOL_PLAN.md` milestone E, the |
| `tests/golden_parity_real_export.rs` | 88 | 0 | 1 / 0 | Golden-parity test against a **real** HTML-app export, not a synthetic |
| `tests/golden_parity_save_writer.rs` | 126 | 0 | 2 / 0 | The writer, checked against the same **real** HTML-app export |
| `tests/golden_parity_tiles.rs` | 245 | 0 | 8 / 0 | Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E's region-export |
| `tests/legacy_records_import.rs` | 167 | 0 | 3 / 0 | Owner Ruling AU: a legacy flat `.zip` imports its settlements, labels and |
| `tests/reference_geojson_round_trip.rs` | 236 | 0 | 12 / 0 | Importing the documents the *reference implementation itself* wrote — |

### `cartalith-jsmath`

Depends on: nothing in the workspace. Used by: `cartalith-assets`, `cartalith-civ`, `cartalith-climate`, `cartalith-engine`, `cartalith-erosion`, `cartalith-godot`, `cartalith-hydrology`, `cartalith-spatial`, `cartalith-terrain`, `cartalith-urban`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `src/lib.rs` | 1405 | 12 | 17 / 0 | JS-semantics math: the operations where Rust's standard library and V8 |
| `src/libm.rs` | 855 | 8 | 0 / 0 | The FDLIBM family: the transcendentals V8 does **not** take from the |

### `cartalith-knowledge`

Depends on: `cartalith-vault`. Used by: `cartalith-godot`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `examples/misses.rs` | 25 | 0 | 0 / 0 | Developer aid: prints, per question, the gold ids, whether each expected literal reached the top-5 |
| `examples/rk11_trials.rs` | 40 | 0 | 0 / 0 | RK-11's control trials on the three committed worlds: the reranker and the chunk-context rules run |
| `examples/rk8_report.rs` | 46 | 0 | 0 / 0 | RK-8's report generator: reads the `*.jsonl` run files the live driver wrote |
| `examples/rk9_gate.rs` | 78 | 0 | 0 / 0 | RK-9's decision-gate report on the three committed evaluation worlds. |
| `src/bm25.rs` | 354 | 12 | 7 / 0 | An in-memory, fielded BM25 (BM25F) inverted index (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.7.1: "own |
| `src/embed.rs` | 642 | 28 | 9 / 0 | The embeddings core for RK-9's decision gate and RK-10's hybrid retrieval |
| `src/eval.rs` | 1026 | 20 | 2 / 0 | The evaluation set, scorer and baselines (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8.1, milestone |
| `src/gate.rs` | 521 | 5 | 6 / 0 | RK-9: the step-2 decision gate (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8 RK-9, owner question K10). |
| `src/lib.rs` | 33 | 0 | 0 / 0 | Retrieval knowledge base over the Markdown Vault and the world's own data |
| `src/live_eval.rs` | 742 | 10 | 13 / 0 | The live-evaluation scorer and report (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8, milestone RK-8). |
| `src/model.rs` | 139 | 6 | 3 / 0 | The shared document model: what a retrievable unit is, and the id scheme that names it |
| `src/rerank.rs` | 511 | 4 | 7 / 0 | RK-11: a reranker and LLM-written chunk context, each adopted only against a measured lift |
| `src/search.rs` | 936 | 19 | 11 / 0 | The retrieval entry point: BM25 over BOTH source kinds, a gazetteer boost, bounded link |
| `src/tokenize.rs` | 160 | 3 | 4 / 0 | The tokenizer every index and every query goes through (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.6.4). |
| `src/vault.rs` | 841 | 19 | 8 / 0 | The Vault side of the knowledge base: heading-aware chunks with free contextual headers, |
| `src/world.rs` | 1199 | 3 | 7 / 0 | The world as documents (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.4.3, milestone RK-1; Ruling DM item |
| `tests/knowledge_embed.rs` | 139 | 0 | 4 / 0 | RK-10's bars on three real worlds (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8 RK-10), at the library level: |
| `tests/knowledge_eval.rs` | 343 | 0 | 13 / 0 | RK-0 and RK-3 on three real generated worlds (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8, s.8.1). |
| `tests/knowledge_graph.rs` | 129 | 0 | 3 / 0 | RK-4's graph bars on a hand-built vault and world (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8 RK-4): |
| `tests/knowledge_scale.rs` | 132 | 0 | 1 / 2 | RK-3's cold-build and query-latency bars, and owner answer K9's vault-size measurement |
| `tests/no_network.rs` | 50 | 0 | 3 / 0 | The knowledge crate has no socket (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.1 hard constraints: "zero |

Ignored harnesses (1): `tests/knowledge_scale.rs::cold_build_and_query_latency_at_2k_5k_10k_notes`.

### `cartalith-llm`

Depends on: `cartalith-vault`. Used by: `cartalith-godot`, `cartalith-llm-loopback`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `src/action_run.rs` | 259 | 3 | 7 / 0 | Running a `propose_actions` card: the rules that decide whether Apply may run at all |
| `src/action_tools.rs` | 861 | 20 | 0 / 0 | The four catalogue tools and the request-size guard (`ASSISTANT_ACTIONS_PROPOSAL.md` s.3.2 and |
| `src/action_tools_tests.rs` | 301 | 0 | 9 / 0 | AC-1 / AC-2 unit tests over a small SYNTHETIC catalogue: the rules, the caps and the guard. |
| `src/actions.rs` | 1305 | 21 | 3 / 0 | The action catalogue (`ASSISTANT_ACTIONS_PROPOSAL.md` s.3.1, milestone AC-1; Ruling DW): ONE |
| `src/anthropic.rs` | 664 | 6 | 8 / 0 | The Anthropic Messages adapter: request builder and typed-event stream decoder |
| `src/catalogue.rs` | 1212 | 26 | 11 / 0 | The generation-parameter catalogue, its tool schema, and the proposal |
| `src/counter.rs` | 166 | 7 | 3 / 0 | The running token counter, and the optional spend cap (LLM-4; Ruling DD Q8). |
| `src/edit_actions.rs` | 939 | 13 | 11 / 0 | Edit actions the model may PROPOSE (`LLM_ASSISTANT_SCOPE.md` s.5c and the LLM-7 row; owner |
| `src/error.rs` | 120 | 2 | 0 / 0 | Errors for the protocol crate, and the one type that may carry free text. |
| `src/event.rs` | 46 | 0 | 0 / 0 | The one event type both adapters produce (LLM_ASSISTANT_SCOPE.md s.2: |
| `src/gpu.rs` | 338 | 6 | 7 / 0 | The local-provider GPU rule (Ruling DF, `LLM_ASSISTANT_SCOPE.md` s.5a), as |
| `src/grounding.rs` | 497 | 2 | 9 / 0 | The grounding check: what in a reply is backed by the facts, and what is not (LLM-11). |
| `src/host.rs` | 444 | 4 | 10 / 0 | Address classification: `classify_host`, a pure function of a host string. |
| `src/keystore.rs` | 332 | 7 | 4 / 0 | Where the API key comes from, and the trait an OS credential store fills. |
| `src/knowledge_tools.rs` | 1180 | 14 | 12 / 0 | The three knowledge tools, the pre-retrieval pack and the citation check |
| `src/lib.rs` | 103 | 0 | 0 / 0 | `cartalith-llm`: the protocol half of the optional LLM assistant. |
| `src/models.rs` | 660 | 15 | 0 / 0 | Listing the models a server offers: pure request building and response parsing |
| `src/note.rs` | 201 | 5 | 7 / 0 | The model-generated marker, and the text a Vault note receives (LLM-3). |
| `src/openai.rs` | 507 | 9 | 0 / 0 | The OpenAI-compatible adapter: request builder and streaming decoder. |
| `src/params_flow.rs` | 1730 | 35 | 29 / 0 | The two-step parameter conversation (`LLM_ASSISTANT_SCOPE.md` s.5b.1-s.5b.2, |
| `src/place_facts.rs` | 1450 | 9 | 13 / 0 | Grounded place facts (LLM-11, `LLM_ASSISTANT_SCOPE.md` "Grounded place prose"). |
| `src/place_vault.rs` | 533 | 9 | 10 / 0 | Which Vault notes are relevant to a place, and the excerpts that are sent (LLM-11). |
| `src/prepare.rs` | 287 | 3 | 8 / 0 | The one door a request leaves through: gate, then build (LLM-3, widened by LLM-4). |
| `src/presets.rs` | 373 | 2 | 7 / 0 | Provider presets: standard endpoints as data (owner addition to LLM-1). |
| `src/profiles.rs` | 1107 | 27 | 26 / 0 | Assistant profiles: named bundles of the settings that differ between "a model on my |
| `src/prompt.rs` | 907 | 7 | 16 / 0 | What the assistant is asked, and exactly what is sent with it (LLM-3). |
| `src/query.rs` | 1062 | 5 | 16 / 0 | The read-only query tools for questions (`LLM_ASSISTANT_SCOPE.md` s.5b.5, milestone LLM-8; owner |
| `src/request.rs` | 59 | 1 | 1 / 0 | One entry point that builds a request for either wire (LLM-4). |
| `src/retry.rs` | 114 | 4 | 3 / 0 | The one automatic retry (LLM-4; scope s.4 rule 7: "no automatic retry beyond one"). |
| `src/secret.rs` | 114 | 5 | 0 / 0 | The API key type and the redactor that keeps it out of error text. |
| `src/session.rs` | 109 | 2 | 0 / 0 | Drive one request through a [`Transport`] and collect its events. |
| `src/settings.rs` | 856 | 10 | 14 / 0 | The settings gate: may a request leave the machine, and if not, why not? |
| `src/sse.rs` | 212 | 3 | 6 / 0 | Server-Sent Events framing, incremental, over raw byte chunks. |
| `src/stream.rs` | 398 | 12 | 9 / 0 | One streaming reply, fed by whatever moves the bytes (LLM-3). |
| `src/transport.rs` | 274 | 8 | 3 / 0 | The transport seam, and the mock that stands in for it in every test. |
| `src/view_actions.rs` | 948 | 12 | 13 / 0 | View and layer actions the model may PROPOSE (`LLM_ASSISTANT_SCOPE.md` s.5b.8 item 12 and |
| `tests/ac1_static_sources.rs` | 180 | 0 | 3 / 3 | AC-1 (`ASSISTANT_ACTIONS_PROPOSAL.md` s.4): the static half of the action catalogue generator. |
| `tests/llm1_bars.rs` | 455 | 0 | 20 / 0 | LLM-1 acceptance bars (LLM_ASSISTANT_SCOPE.md s.3), one test per bar. |
| `tests/llm3_live_fixtures.rs` | 120 | 0 | 3 / 0 | The decoder against streams RECORDED from a real local server (LLM-3). |
| `tests/llm4_bars.rs` | 178 | 0 | 8 / 0 | LLM-4 acceptance bars (LLM_ASSISTANT_SCOPE.md s.3), one test per bar, over the |
| `tests/models_wires.rs` | 381 | 0 | 19 / 0 | The model-list layer (`src/models.rs`), over real recorded replies and hostile ones. |
| `tests/no_network.rs` | 86 | 0 | 2 / 0 | "Zero network use in tests" (LLM-1 bar) and the stronger claim behind it: |

Ignored harnesses (1): `tests/ac1_static_sources.rs::regenerate_static_sources`.

### `cartalith-llm-loopback`

Depends on: `cartalith-llm`. Used by: nothing (a leaf consumer).

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `src/lib.rs` | 11 | 0 | 0 / 0 | Intentionally empty: this crate exists for `tests/loopback.rs`. |
| `tests/loopback.rs` | 481 | 0 | 8 / 0 | LLM-4 mock-SERVER tests: the real gate, request builders and decoders of |

### `cartalith-noise`

Depends on: nothing in the workspace. Used by: `cartalith-assets`, `cartalith-civ`, `cartalith-climate`, `cartalith-erosion`, `cartalith-godot`, `cartalith-gpu`, `cartalith-terrain`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `src/lib.rs` | 609 | 14 | 13 / 0 | hash/vnoise/fbm/ridged — hand-ported for parity, not `noise-rs` |
| `tests/golden_parity.rs` | 393 | 0 | 2 / 0 | Golden-parity test for `hash`, `vnoise`, `fbm`, `ridged` (PARITY_TESTING.md). |
| `tests/golden_parity_periodic.rs` | 331 | 0 | 1 / 0 | Golden-parity test for `pvnoise`, `pfbm`, `pridged` (world-wrap noise variants, PARITY_TESTING.md). |

### `cartalith-rng`

Depends on: nothing in the workspace. Used by: `cartalith-civ`, `cartalith-erosion`, `cartalith-godot`, `cartalith-terrain`, `cartalith-urban`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `src/lib.rs` | 108 | 3 | 3 / 0 | mulberry32, ported exactly (PARITY_TESTING.md) |
| `tests/golden_parity.rs` | 159 | 0 | 1 / 0 | Golden-parity test for `mulberry32` (PARITY_TESTING.md). |

### `cartalith-spatial`

Depends on: `cartalith-jsmath`. Used by: `cartalith-engine`, `cartalith-godot`, `cartalith-io`, `cartalith-terrain`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `src/colour_paint.rs` | 2234 | 32 | 26 / 0 | The non-destructive colour-paint layer (`COLOUR_PAINT_SCOPE.md`, milestone |
| `src/colour_paint/fill.rs` | 592 | 3 | 5 / 0 | The colour-paint **fill helper** (`COLOUR_PAINT_SCOPE.md` §13, milestone |
| `src/colour_paint/material.rs` | 626 | 13 | 5 / 0 | Emissive paint **materials** for the colour-paint layer: lava and its |
| `src/contour.rs` | 451 | 1 | 7 / 0 | Level-set contour tracing over a scalar grid — the shared |
| `src/geo.rs` | 461 | 6 | 12 / 0 | Raster→vector boundary tracing and the local-planar coordinate transform |
| `src/lib.rs` | 294 | 10 | 8 / 0 | Generic tiling, dirty-tracking and geometry pieces with no opinion on what |
| `src/measure.rs` | 475 | 7 | 23 / 0 | The Measure tool — `UNIFIED_TOOL_PLAN.md` milestone E. |
| `src/paint.rs` | 1161 | 17 | 30 / 0 | Categorical override painting — `UNIFIED_TOOL_PLAN.md` milestone C's |
| `src/pass.rs` | 1345 | 29 | 31 / 0 | `PassBuffer<S>` — the non-destructive draft/commit/discard layer |
| `src/pyramid.rs` | 312 | 9 | 7 / 0 | The LOD tile pyramid's addressing — the index arithmetic the reference's |
| `src/region.rs` | 284 | 3 | 12 / 0 | Region selection geometry — `UNIFIED_TOOL_PLAN.md` milestone E, |
| `src/staleness.rs` | 592 | 18 | 17 / 0 | `StageGraph` — deferred, lazily-evaluated staleness across a pipeline of |
| `src/stroke.rs` | 1238 | 7 | 16 / 0 | A deterministic, streamable brush-stroke engine: pointer samples in, dabs |
| `tests/golden_parity_geo.rs` | 320 | 0 | 8 / 0 | Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E2's raster→vector |
| `tests/golden_parity_measure_poly.rs` | 278 | 0 | 5 / 0 | Golden-parity tests for the Area tool's three polygon primitives: |
| `tests/golden_parity_paint.rs` | 279 | 0 | 7 / 0 | Golden-parity tests for the Cartography paint brush — |
| `tests/golden_parity_pyramid.rs` | 126 | 0 | 4 / 0 | Golden-parity tests for the LOD pyramid's addressing — `pyramidDims` |
| `tests/golden_parity_region.rs` | 89 | 0 | 2 / 0 | Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E's region |

### `cartalith-terrain`

Depends on: `cartalith-jsmath`, `cartalith-noise`, `cartalith-rng`, `cartalith-spatial`. Used by: `cartalith-civ`, `cartalith-climate`, `cartalith-engine`, `cartalith-erosion`, `cartalith-godot`, `cartalith-hydrology`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `src/amplify.rs` | 2179 | 16 | 37 / 0 | Regional amplification — `UNIFIED_TOOL_PLAN.md` milestone E, the compute |
| `src/analysis.rs` | 1112 | 11 | 21 / 0 | Reusable terrain-analysis fields — slope, aspect, curvature, TPI, local |
| `src/center.rs` | 186 | 4 | 4 / 0 | Landmass centering — the reference's `#centerBtn` (HTML lines 3150-3199). |
| `src/edifice_furrows.rs` | 462 | 3 | 7 / 0 | **Tile-time radial furrows on volcano flanks** (owner Ruling CO item 3, |
| `src/edifice_pits.rs` | 572 | 6 | 5 / 0 | **Volcano flank-pit fill** (owner Ruling CO, 2026-10-18; measured as |
| `src/fjord.rs` | 280 | 3 | 4 / 0 | Fjords — constrained glacial-coastal incision (reference HTML lines |
| `src/geology.rs` | 1310 | 15 | 13 / 0 | **The lithology model and the rock column** -- GF-1 of |
| `src/infer.rs` | 750 | 9 | 9 / 0 | Tectonic inversion for imported heightmaps — reference HTML lines |
| `src/landform.rs` | 216 | 1 | 4 / 0 | R5 landform classification (reference HTML lines 8075-8107). |
| `src/lava_flows.rs` | 1200 | 6 | 7 / 0 | Terrain-following lava flows, traced down the real height field from each |
| `src/lib.rs` | 5052 | 38 | 47 / 0 | tectonics, height formula, normalize, volcanism, world-structure archetypes |
| `src/orogen.rs` | 995 | 7 | 9 / 0 | **Orogen relief** -- a dissected mountain-range height term localised to |
| `src/region_edit.rs` | 1041 | 10 | 19 / 0 | Region edits for the assistant's edit actions (`LLM_ASSISTANT_SCOPE.md` s.5c, milestone |
| `src/rift_trend.rs` | 206 | 1 | 3 / 0 | The trend of a rift at a vent, read off the plate boundary at generation |
| `src/sculpt.rs` | 3393 | 28 | 51 / 0 | The Sculpt editor's landform-stamp registry — `UNIFIED_TOOL_PLAN.md` |
| `src/tile_render.rs` | 495 | 7 | 12 / 0 | The refined-tile visual: hypsometric tint × hillshade — `UNIFIED_TOOL_PLAN.md` |
| `src/vector.rs` | 663 | 3 | 8 / 0 | **EF-6** — the world's linear features as vectors, beyond the rivers |
| `src/volcano_attr.rs` | 720 | 24 | 10 / 0 | Per-volcano attributes derived from the edifice record (VA-1, |
| `src/volcano_kinds.rs` | 1290 | 8 | 11 / 0 | VA-6 of `VOLCANO_APPEARANCE_RESEARCH.md`: new volcano kinds, placed only |
| `tests/golden_parity.rs` | 55 | 0 | 4 / 0 | Golden-parity test for `compute_warp` (fillWarpRows + computeWarpPrep + warpParams, |
| `tests/golden_parity_age.rs` | 33 | 0 | 3 / 0 | Golden-parity test for build_age_field (PARITY_TESTING.md). |
| `tests/golden_parity_amplify.rs` | 290 | 0 | 11 / 0 | Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E's region-export |
| `tests/golden_parity_assign.rs` | 51 | 0 | 4 / 0 | Golden-parity test for `assign_plates` (JFA Voronoi, PARITY_TESTING.md). |
| `tests/golden_parity_blur.rs` | 48 | 0 | 4 / 0 | Golden-parity test for `gauss_blur` (three-pass box blur, PARITY_TESTING.md). |
| `tests/golden_parity_center.rs` | 170 | 0 | 6 / 0 | Golden-parity test for landmass centering (`PARITY_TESTING.md`) — |
| `tests/golden_parity_fjord.rs` | 172 | 0 | 6 / 0 | Golden-parity test for fjord masking and carving (`PARITY_TESTING.md`) |
| `tests/golden_parity_flex_hetero_resist.rs` | 59 | 0 | 5 / 0 | Golden-parity test for compute_flexure, compute_heterogeneity, compute_resistance (PARITY_TESTING.md). |
| `tests/golden_parity_height.rs` | 68 | 0 | 3 / 0 | Golden-parity test for compute_height + normalize_field (PARITY_TESTING.md). |
| `tests/golden_parity_infer.rs` | 458 | 0 | 8 / 0 | Golden-parity test for the tectonic-inversion pass (`PARITY_TESTING.md`) |
| `tests/golden_parity_landform.rs` | 208 | 0 | 6 / 0 | Golden-parity test for R5 landform classification |
| `tests/golden_parity_orogeny.rs` | 149 | 0 | 5 / 0 | Golden-parity tests for `build_orogeny_field`/`smooth_orogeny` (T2+T3, |
| `tests/golden_parity_plate_circular_mean.rs` | 166 | 0 | 2 / 0 | `JS_SEMANTICS_AUDIT.md` §4.4's `-terrain:372`, closed. |
| `tests/golden_parity_plates.rs` | 81 | 0 | 5 / 0 | Golden-parity test for `build_plates` (PARITY_TESTING.md). Reference HTML |
| `tests/golden_parity_sculpt.rs` | 669 | 0 | 23 / 0 | Golden-parity tests for the Sculpt editor's landform stamps — |
| `tests/golden_parity_stress.rs` | 56 | 0 | 3 / 0 | Golden-parity test for `compute_stress` (PARITY_TESTING.md). Reference |
| `tests/golden_parity_tile_render.rs` | 260 | 0 | 3 / 0 | Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E2's pixel half: |
| `tests/golden_parity_volc_craters.rs` | 140 | 0 | 5 / 0 | Golden-parity test for stamp_volcanoes_simple and stamp_craters (PARITY_TESTING.md). |
| `tests/golden_parity_volc_provinces.rs` | 81 | 0 | 1 / 0 | Golden-parity test for `stamp_volcanoes_provinces` (PARITY_TESTING.md). |
| `tests/golden_parity_world_structure.rs` | 50 | 0 | 5 / 0 | Golden-parity tests for generate_continentality_field and |
| `tests/golden_parity_zoom_detail.rs` | 271 | 0 | 7 / 0 | Golden-parity tests for `addZoomDetail` (reference line 10467) — the |
| `tests/volcano_edifice.rs` | 312 | 0 | 7 / 0 | End-to-end behaviour of `EdificeModel::Morphological` (Lane 3: the volcanic |
| `tests/volcano_transform_boundaries.rs` | 314 | 0 | 5 / 0 | Volcanoes must not be placed on transform plate margins. |

### `cartalith-urban`

Depends on: `cartalith-jsmath`, `cartalith-rng`. Used by: `cartalith-civ`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `examples/town_json.rs` | 109 | 0 | 0 / 0 | Write one `generate()` town as JSON, in the key shape |
| `src/amenities.rs` | 764 | 7 | 0 / 0 | Milestone 14 — amenities (`buildMarkets`, `buildCivic`, `orientedRect`, |
| `src/amenities/tests.rs` | 1058 | 0 | 22 / 0 | Milestone 14's tests. |
| `src/amenities/tests/golden.rs` | 1618 | 0 | 0 / 0 | GENERATED by the milestone-14 `amenities` golden capture -- do not hand-edit. |
| `src/astar.rs` | 207 | 1 | 0 / 0 | A\* over the site cost raster — reference lines 28514-28547, one function. |
| `src/astar/tests.rs` | 416 | 0 | 7 / 0 | Milestone 3's tests. All but two are **golden**: `astar` is one of the |
| `src/astar/tests/golden.rs` | 78 | 0 | 0 / 0 | GENERATED by the milestone-3 golden capture -- do not hand-edit. |
| `src/blocks.rs` | 558 | 4 | 0 / 1 | Milestone 12 — blocks and parcels (`buildBlocks`, `buildParcels`, reference |
| `src/blocks/tests.rs` | 781 | 0 | 12 / 2 | Milestone 12's tests. |
| `src/blocks/tests/golden.rs` | 679 | 0 | 0 / 0 | GENERATED by the milestone-12 golden capture -- do not hand-edit. |
| `src/citadel.rs` | 357 | 2 | 0 / 0 | A citadel astride the town wall — **not a reference port.** New engine work |
| `src/citadel/tests.rs` | 171 | 0 | 3 / 0 | The citadel's own acceptance bar. There is no reference to diff against |
| `src/cleanup.rs` | 662 | 8 | 0 / 0 | Milestone 11 — the graph cleanup passes (reference lines 30038-30190). |
| `src/cleanup/tests.rs` | 1472 | 0 | 43 / 0 | Milestone 11's tests. |
| `src/cleanup/tests/golden.rs` | 169 | 0 | 0 / 0 | GENERATED by the milestone-11 golden capture -- do not hand-edit. |
| `src/courtyard.rs` | 244 | 4 | 0 / 0 | Perimeter blocks around an open courtyard, on the town's outermost ring — |
| `src/courtyard/tests.rs` | 464 | 0 | 13 / 0 | The perimeter block's own acceptance bar. There is no reference to diff |
| `src/districts.rs` | 1515 | 9 | 0 / 0 | Milestone 13 — districts and buildings (reference lines **30345-30682**, |
| `src/districts/tests.rs` | 1049 | 0 | 13 / 0 | Milestone 13's tests — districts, buildings and places of worship. |
| `src/districts/tests/golden.rs` | 2905 | 0 | 0 / 0 | GENERATED by the milestone-13 golden capture -- do not hand-edit. |
| `src/fortify.rs` | 1249 | 8 | 0 / 0 | Fortification — reference lines **29631-30032**, nine functions. |
| `src/fortify/tests.rs` | 2224 | 0 | 49 / 0 | Milestone 10's tests. |
| `src/fortify/tests/golden.rs` | 2766 | 0 | 0 / 0 | GENERATED by the milestone-10 golden capture -- do not hand-edit. |
| `src/generate.rs` | 1545 | 7 | 0 / 0 | Milestone 16 — `generate()` and `hashModel` (reference lines 30931-31094). |
| `src/generate/harbour_front_tests.rs` | 322 | 0 | 9 / 0 | Ruling FK item 1's gate (owner, 2026-11-16: *"harbour towns lay out to |
| `src/generate/tests.rs` | 986 | 0 | 13 / 0 | Milestone 16's tests. |
| `src/generate/tests/golden.rs` | 3636 | 0 | 0 / 0 | GENERATED by `cartalith-native/tools/um_capture.js` -- do not hand-edit. |
| `src/geom.rs` | 755 | 22 | 13 / 0 | The vector / polygon geometry kernel — reference lines 28286-28362 |
| `src/graph.rs` | 741 | 10 | 0 / 0 | The planar street graph — reference lines 28363-28512 (`makeGraph` … |
| `src/graph/tests.rs` | 545 | 0 | 8 / 0 | Milestone 2's tests. Almost all of them are **golden**: the reference's own |
| `src/graph/tests/golden.rs` | 189 | 0 | 0 / 0 | GENERATED by the milestone-2 golden capture -- do not hand-edit. |
| `src/growth.rs` | 1128 | 8 | 0 / 0 | Organic growth — reference lines **29384-29630**, five functions. |
| `src/growth/tests.rs` | 1021 | 0 | 20 / 0 | Milestone 7's tests. **Every one of the 48 scenarios is golden** — the total |
| `src/growth/tests/golden.rs` | 2162 | 0 | 0 / 0 | GENERATED by the milestone-7 golden capture -- do not hand-edit. |
| `src/hinterland.rs` | 1058 | 9 | 0 / 0 | Milestone 15 — hinterland, decay, details and metrics (reference lines |
| `src/hinterland/tests.rs` | 1675 | 0 | 35 / 0 | Milestone 15's tests. |
| `src/hinterland/tests/golden.rs` | 1661 | 0 | 0 / 0 | GENERATED by the milestone-15 golden capture -- do not hand-edit. |
| `src/lib.rs` | 159 | 0 | 0 / 0 | Urban morphology — the reference's script block 4 ("UME", lines 28166-31104 |
| `src/plaza.rs` | 497 | 6 | 0 / 0 | Milestone 8, part one — `buildPlaza` (reference lines 28941-28963). |
| `src/plaza/tests.rs` | 737 | 0 | 17 / 0 | Milestone 8's `buildPlaza` tests. |
| `src/plaza/tests/golden.rs` | 417 | 0 | 0 / 0 | GENERATED by the milestone-8 `buildPlaza` golden capture -- do not hand-edit. |
| `src/radial.rs` | 319 | 2 | 0 / 0 | Milestone 8 — the radial (Venus) planning mode: `buildRadialStreets` and |
| `src/radial/tests.rs` | 640 | 0 | 13 / 0 | Milestone 8's `buildRadialStreets` / `buildWaterway` tests. |
| `src/radial/tests/golden.rs` | 1018 | 0 | 0 / 0 | GENERATED by the milestone-8 `buildRadialStreets`/`buildWaterway` golden |
| `src/riverbank.rs` | 1252 | 12 | 0 / 0 | **Not the reference** -- a river town builds to its riverbank (2026-11-15, |
| `src/riverbank/tests.rs` | 1405 | 0 | 48 / 0 | The riverbank geometry's gate ([`crate::riverbank`], 2026-11-15). Each |
| `src/rng.rs` | 295 | 10 | 5 / 0 | Labelled deterministic RNG substreams — reference lines 28178-28190. |
| `src/routes.rs` | 446 | 5 | 0 / 0 | Anchors and primary routes — reference lines **28743-28833**, three |
| `src/routes/tests.rs` | 575 | 0 | 6 / 0 | Milestone 6's tests. **Every one of the 29 scenarios is golden** — market, |
| `src/routes/tests/golden.rs` | 676 | 0 | 0 / 0 | GENERATED by the milestone-6 golden capture -- do not hand-edit. |
| `src/rules.rs` | 706 | 8 | 0 / 0 | Generation rules and culture profiles — reference lines **28193-28280**. |
| `src/rules/tests.rs` | 649 | 0 | 13 / 0 | Milestone 4's tests. **All fifty rule cases, both profiles and all fifteen |
| `src/rules/tests/golden.rs` | 200 | 0 | 0 / 0 | GENERATED by the milestone-4 golden capture -- do not hand-edit. |
| `src/site.rs` | 1063 | 20 | 0 / 0 | The site model — reference lines **28549-28741**, three functions. |
| `src/site/tests.rs` | 746 | 0 | 16 / 0 | Milestone 5's tests. **All nineteen `shoreFromMask` scenarios and all |
| `src/site/tests/golden.rs` | 4502 | 0 | 0 / 0 | GENERATED by the milestone-5 golden capture -- do not hand-edit. |
| `src/village.rs` | 328 | 1 | 0 / 0 | Street villages -- **not a reference port.** Owner Ruling FE item 2 |
| `src/village/tests.rs` | 215 | 0 | 8 / 0 | Ruling FE item 2's street village ([`super::plat_tofts`] and the |
| `src/wallgap.rs` | 197 | 1 | 0 / 0 | How much empty ground a town wall encloses -- the measurement behind Ruling |
| `src/wallhug.rs` | 1381 | 12 | 0 / 0 | A town wall that hugs the built-up area -- **not a reference port.** Owner |
| `src/wallhug/tests.rs` | 705 | 0 | 27 / 0 | Ruling EW item 1's acceptance bars for [`super::hug_wall`]: properties of |
| `src/wallside.rs` | 729 | 5 | 0 / 0 | Lots built against the town wall — **not a reference port.** A deliberate |
| `src/wallside/tests.rs` | 609 | 0 | 25 / 0 | `wallside`'s own checks. The before/after measurement across real towns is |
| `src/water.rs` | 718 | 6 | 0 / 0 | Milestone 9 — water infrastructure (reference lines 28967-29154). |
| `src/water/tests.rs` | 897 | 0 | 12 / 0 | Milestone 9's water-infrastructure tests. |
| `src/water/tests/golden.rs` | 2624 | 0 | 0 / 0 | GENERATED by the milestone-9 water-infrastructure golden capture -- do not hand-edit. |

Ignored harnesses (1): `src/blocks/tests.rs::measure_parcel_water_footprint_tests`.

### `cartalith-vault`

Depends on: nothing in the workspace. Used by: `cartalith-godot`, `cartalith-knowledge`, `cartalith-llm`.

| File | Lines | pub fn | tests / ignored | What it is (first `//!` line) |
|---|---|---|---|---|
| `src/backlinks.rs` | 556 | 24 | 0 / 0 | Backlinks and unlinked mentions — `GUI_GAP_REGISTER.md` **VA-01**. |
| `src/backlinks/tests.rs` | 441 | 0 | 18 / 0 | `backlinks.rs`'s own tests. |
| `src/block.rs` | 371 | 5 | 7 / 0 | The machine-owned Cartalith block (`MARKDOWN_VAULT_INTEGRATION.md` §23, |
| `src/chronos.rs` | 840 | 11 | 19 / 0 | Authored, dated events read out of a note's ` ```chronos ` code blocks |
| `src/embed.rs` | 701 | 8 | 15 / 0 | Embeds (`![[note]]`, `![[note#Heading]]`, `![[pic.png/300]]`, `![alt](pic.png)`): what the |
| `src/export.rs` | 526 | 5 | 7 / 0 | The exportable-field registry and the block renderer |
| `src/frontmatter.rs` | 677 | 5 | 8 / 0 | Reading and editing one property of a note's YAML frontmatter, as lines. |
| `src/lib.rs` | 3094 | 52 | 37 / 0 | Markdown Vault integration — the engine half. |
| `src/links.rs` | 1798 | 36 | 17 / 0 | Knowledge links (`MARKDOWN_VAULT_INTEGRATION.md` §11, §26, §27). |
| `src/markdown.rs` | 768 | 13 | 17 / 0 | Section-aware Markdown reading and writing. |
| `src/noteedit.rs` | 543 | 6 | 9 / 0 | Pure helpers the note editor calls on the text it is holding: the |
| `src/notelinks.rs` | 1274 | 18 | 0 / 0 | Note-to-note links as an author writes them, in Obsidian's own syntax |
| `src/notelinks/tests.rs` | 610 | 0 | 32 / 0 | `notelinks.rs`'s own tests: link spans, the resolver, the shortest-link |
| `src/private.rs` | 430 | 5 | 7 / 0 | The ONE rule for "is this Vault note private?" (Ruling DP; `MARKDOWN_VAULT_SCOPE.md` s.Private notes). |
| `src/provider.rs` | 723 | 15 | 8 / 0 | The vault provider trait, and the filesystem implementation of it |
| `src/suggest.rs` | 231 | 2 | 4 / 0 | Ranking link targets for the editor's `[[` autocomplete. |
| `src/template.rs` | 1188 | 18 | 20 / 0 | Creating a note from one of the author's own templates, and inserting one |
| `src/textscan.rs` | 307 | 6 | 6 / 0 | Shared scanning primitives for the note editor's pure helpers |

## 5. Similarity table (families for R1)

Function definitions in `src/` (test modules excluded) whose names match each family's pattern. **A shared name is a candidate, not a finding:** whether two entries are the same system, and which differences are real (precision, tie-break, NaN, iteration order, world wrap), is R1's judgement (`CODEBASE_REFACTOR_PROPOSAL.md` section 4). Priority-queue users were found separately by `BinaryHeap`.

| Family | Implementations (crate: file::fn) |
|---|---|
| distance transform / jump flood / SDF | **assets**: `lineart.rs::edt`<br>**civ**: `lib.rs::jfa_dist`; `lib.rs::build_coast_sdf`; `lib.rs::civ_ocean_dist_field`; `urban_adapter.rs::civ_coast_dist_field`; `landmark/natural_kinds.rs::chamfer_km`<br>**godot**: `pack.rs::feature_sdf_texture`; `render.rs::apply_coast_sdf`; `render.rs::apply_river_sdf`; `render.rs::sdf_eco_k`; `render.rs::build_river_sdf`; `render.rs::coast_distance`<br>**gpu**: `lib.rs::init_gpu_jfa_plates`; `lib.rs::init_gpu_jfa_plates_with`; `lib.rs::try_gpu_jfa_plates`<br>**terrain**: `infer.rs::chamfer_dist`<br>**urban**: `site.rs::chamfer`; `wallhug.rs::chamfer_grid` |
| bilinear / bicubic sampling, upsample | **assets**: `lineart.rs::bilinear`<br>**civ**: `lib.rs::civ_catmull_rom_sample`<br>**climate**: `lib.rs::bil_c`<br>**engine**: `lib.rs::rain_upsample`<br>**godot**: `lib.rs::resamples_a_way_to_render_density`; `lib.rs::remaps_breaks_onto_the_resampled_list`; `pack.rs::sample_bilinear`; `pack.rs::area_resample_premultiplied`; `render.rs::bilinear`; `river_straight.rs::resample`; `sample_bridge.rs::bil_c`<br>**terrain**: `lib.rs::bil_c` |
| blur / box filter | **assets**: `lineart.rs::gaussian_blur`; `refine.rs::box_blur`<br>**climate**: `lib.rs::blur_coarse`<br>**engine**: `lib.rs::plate_base_blur_r`<br>**godot**: `param_catalogue.rs::blurb`; `render.rs::blur_once`<br>**gpu**: `lib.rs::blur_norm`; `lib.rs::init_gpu_gauss_blur`; `lib.rs::init_gpu_gauss_blur_with`; `lib.rs::dispatch_gpu_gauss_blur`; `lib.rs::gauss_blur_grid_gpu_with`; `lib.rs::blur_output_plausible`; `lib.rs::try_gpu_blur`; `lib.rs::blur_output_plausible_accepts_a_blur_and_rejects_corruption`<br>**terrain**: `analysis.rs::blur`; `lib.rs::gauss_blur` |
| connected components / union-find | **assets**: `lineart.rs::label_components` |
| PNG encode | **assets**: `raster.rs::encode_png`; `raster.rs::encode_png_rgb8`; `raster.rs::encode_png_luma16`<br>**engine**: `region_export.rs::tile_png_bytes`<br>**godot**: `export_stream.rs::write_png` |
| slope / gradient | **civ**: `landmark.rs::gradient`; `landmark.rs::landmark_gradient`; `landmark.rs::gradient`; `lib.rs::slope_at`<br>**climate**: `windthrow.rs::slope_at`<br>**erosion**: `passes.rs::slope_at`<br>**godot**: `pack.rs::slope_at`; `render.rs::slope_at`; `sample_bridge.rs::slope_gradient`; `sample_bridge.rs::slope_at`; `sample_bridge.rs::aspect_of_gradient`<br>**terrain**: `lava_flows.rs::gradient` |
| smoothstep / clamp01 / lerp helpers | **civ**: `lib.rs::clamp01`<br>**godot**: `render.rs::clamp01`; `render.rs::lerp`; `render.rs::mix`; `render.rs::saturate`; `river_field.rs::smoothstep`; `river_meander.rs::mix`; `sample_bridge.rs::lerp`; `sample_bridge.rs::mix`<br>**hydrology**: `rough.rs::mix`<br>**jsmath**: `lib.rs::smoothstep`<br>**spatial**: `colour_paint/material.rs::smoothstep`<br>**terrain**: `sculpt.rs::clamp01`; `sculpt.rs::lerp`; `tile_render.rs::lerp`; `tile_render.rs::mix`; `volcano_kinds.rs::lerp`<br>**urban**: `geom.rs::lerp` |
| shortest path (Dijkstra / A*) | **civ**: `lib.rs::road_dijkstra`; `lib.rs::road_dijkstra_multi`; `lib.rs::territory_dijkstra_batch`; `tools.rs::civ_dijkstra_path`; `tools.rs::civ_dijkstra_path_guided`; `tools.rs::civ_join_dijkstra_segs`; `tools.rs::civ_join_dijkstra_segs_guided`; `trade.rs::dijkstra`; `way_join.rs::civ_dijkstra_path_joined`; `way_join.rs::civ_join_dijkstra_segs_joined`<br>**urban**: `astar.rs::astar` |
| polyline / polygon simplify, smooth | **civ**: `lib.rs::civ_rdp_simplify`<br>**urban**: `geom.rs::chaikin`; `geom.rs::simplify`; `wallhug.rs::simplify_out` |
| D8 flow direction / accumulation | **civ**: `trade.rs::accumulate`<br>**erosion**: `lib.rs::d8_table`; `lib.rs::accumulate_by_basin`<br>**godot**: `context_pick_bridge.rs::combined_receivers`<br>**gpu**: `lib.rs::cpu_receivers`<br>**hydrology**: `lib.rs::flow_receivers`; `lib.rs::flow_receivers_with`; `lib.rs::d8_table`; `lib.rs::d8_receiver`; `lib.rs::strahler_from_receivers`; `rough.rs::rough_receiver` |
| FNV / hash helpers | **godot**: `asset_bridge.rs::fnv1a64`; `device_stage_bench.rs::fnv`; `device_stage_bench.rs::fnv_f32`; `story_bridge.rs::hash_usizes`; `undo_group_bridge.rs::fnv1a_from`; `undo_group_bridge.rs::fnv1a`<br>**io**: `atlas.rs::fnv1a32_hex`<br>**knowledge**: `gate.rs::fnv`<br>**urban**: `rng.rs::fnv1a` |
| marching squares / contour | **godot**: `render.rs::tanaka_contour`<br>**spatial**: `contour.rs::contour_polylines` |
| point in polygon / polygon area | **godot**: `town_river_census.rs::poly_area_xy`<br>**spatial**: `measure.rs::polygon_area`; `measure.rs::point_in_polygon`<br>**urban**: `geom.rs::poly_area`; `geom.rs::point_in_poly` |
| value noise / fbm / ridged | **godot**: `volcano_look.rs::zoom_fbm`<br>**gpu**: `lib.rs::vnoise_grid_cpu`; `lib.rs::vnoise_grid`<br>**noise**: `lib.rs::vnoise`; `lib.rs::fbm`; `lib.rs::ridged`; `lib.rs::ridged_oct`; `lib.rs::pvnoise`; `lib.rs::pfbm`; `lib.rs::pridged`<br>**spatial**: `colour_paint/material.rs::value_noise`; `colour_paint/material.rs::fbm`<br>**terrain**: `sculpt.rs::octave_seed`; `sculpt.rs::sculpt_fbm`; `sculpt.rs::sculpt_ridged`; `sculpt.rs::fbm`; `sculpt.rs::ridged` |
| depression fill / priority flood | **hydrology**: `lib.rs::build_routing_surface`; `lib.rs::build_routing_surface_with`<br>**terrain**: `orogen.rs::breach_belt_pits` |
| Poisson disc / blue noise | **terrain**: `lib.rs::poisson_sample` |
| priority flood (by `BinaryHeap`) | **hydrology**: `lib.rs::build_routing_surface_with`, `lib.rs::rv1_pits`; **erosion**: `construction.rs::flood4`; **terrain**: `edifice_pits.rs::dry_active_craters`, `edifice_pits.rs::fill_edifice_flank_pits`, `orogen.rs::breach_belt_pits`, `volcano_kinds.rs::fill_new_pockets`; **spatial**: `colour_paint/fill.rs::pour_point`; **civ**: `way_merge.rs::search` (a path search); **urban**: `astar.rs::astar`; **godot**: `river_stroke.rs::draw_ranks`, `way_census.rs` (census only) |

Also visible from the census, for R1: per-crate copies of small numeric helpers (`bil_c` in climate, terrain and godot; `lerp`/`mix`/`clamp01` in godot, terrain, urban and civ; `slope_at` in civ, climate, erosion and godot), three FNV variants (godot, io, knowledge, urban), and two polygon toolkits (`spatial::measure`, `urban::geom`). `cartalith-jsmath` already unified the V8-semantics helpers (`ARCHITECTURE.md`); these are the next layer down.

## 6. Dead and near-dead code

`pub fn`s defined in `src/` whose name appears in no other file and only once in their own (the definition): **4**, each opened by name on 2026-11-16:

- `cartalith-civ` `src/way_join.rs::spliced_share`
- `cartalith-hydrology` `src/rough.rs::noise_at`
- `cartalith-llm` `src/place_vault.rs::plan_json`
- `cartalith-spatial` `src/colour_paint.rs::has_emissive`

A further 336 `pub fn`s are used only inside their own file (public without a reason, or reached through a `#[func]` export, macro or trait). A list is in the audit's scratch output; R1 should decide visibility, not deletion. A lexical census cannot see trait dispatch or `#[func]` names called from GDScript by string, so every entry needs a look before removal.

## 7. The Godot shell

144 scripts (`shell/*.gd`, `shell/workspaces/*.gd`, and the root `map_overlay.gd`, `credits.gd`, `smoke_test.gd`), 194339 lines, 6959 `func`s. Probes (`_*.gd` at the project root, 755 scripts of which 725 are `_*_probe.gd`) and the untracked editor add-ons are not indexed here. `menus.gd` defines 177 `ID_*` command ids; `DccSettings` (`dcc_settings.gd`) stores 44 keys in 29 sections of `user://cartalith_settings.cfg`.

| Script | Lines | `func`s | class_name | extends | What it is (first `##` line) |
|---|---|---|---|---|---|
| `credits.gd` | 95 | 4 | — | AcceptDialog | Credits & academic principles (Phase 1 closeout item, `ROADMAP.md`/ |
| `map_overlay.gd` | 10000 | 339 | — | Control | Phase 2 civilisation-layer overlay (cartalith-civ): settlements + roads |
| `shell/adjust_panel.gd` | 991 | 56 | — | VBoxContainer | ET-C4 (`EDITING_TOOLS_SCOPE.md` section 5; Ruling BX item 4): the per-layer adjustment |
| `shell/app.gd` | 7698 | 264 | DccApp | DccShell | The application root: the frame plus everything hung off it. |
| `shell/asset_library_window.gd` | 5542 | 144 | AssetLibraryWindow | AcceptDialog | §8's Asset library window (`DCC_SHELL_SPEC.md` §2.3/§8) -- Assets ▸ |
| `shell/assistant_chip.gd` | 109 | 5 | AssistantChip | RefCounted | The conditional Assistant chip's STATE -- which of "hidden", "disabled" and "live" it is, and why |
| `shell/auto_populate.gd` | 135 | 3 | — | RefCounted | MS-8 (`MOBILE_SHELL_PROPOSAL.md` section 2.3, Ruling DT): New World's AUTO-POPULATE. |
| `shell/browse_dialog.gd` | 1290 | 38 | DccBrowseDialog | AcceptDialog | The in-shell replacement for Godot's stock `FileDialog`, built from the |
| `shell/busy_overlay.gd` | 849 | 42 | — | Control | The busy overlay: a modal, input-swallowing layer shown while the engine is busy, with one line |
| `shell/city_viewer_window.gd` | 1406 | 48 | CityViewerWindow | AcceptDialog | The City Viewer (`GUI_GAP_REGISTER.md` UM-02) — the reference's |
| `shell/colour_fill.gd` | 270 | 17 | — | RefCounted | CARTO ▸ Colours ▸ **Fill an area** -- the colour-paint fill helper (`COLOUR_PAINT_SCOPE.md` §13, milestone CP-5). |
| `shell/colour_paint_notice.gd` | 249 | 16 | — | RefCounted | The colour-paint **map-width notice** with **Scale with the map** (`COLOUR_PAINT_SCOPE.md` §5 and §17 Q5; owner |
| `shell/colour_paint_tool.gd` | 1021 | 58 | — | RefCounted | CARTO ▸ Colours ▸ **Paint colour** -- Ruling CN's non-destructive colour |
| `shell/command_index.gd` | 782 | 29 | CommandIndex | RefCounted | Every action, parameter and export the app has, as one searchable list. |
| `shell/conflicts_window.gd` | 362 | 15 | — | AcceptDialog | Story ▸ CONFLICTS & CAMPAIGNS ▸ Conflicts… -- milestone SM-6 of |
| `shell/context_broker.gd` | 812 | 34 | — | RefCounted | `MAP_CONTEXT_SCOPE.md` §3 and §9.1: **one request, many providers, one |
| `shell/context_card.gd` | 1180 | 48 | — | PopupPanel | `MAP_CONTEXT_SCOPE.md` §4 and §9.1 -- **the context card**, CM-2. The |
| `shell/context_key_relay.gd` | 87 | 3 | — | Node | PC-7. Carries the menu bar's chords out of a non-blocking context window (`context_window.gd`). |
| `shell/context_module_window.gd` | 130 | 7 | — | AcceptDialog | PC-10 (`PC_SHELL_PROPOSAL.md` §3.3, "A window": a tab's menu ▸ Open in a window "shows that module in |
| `shell/context_modules.gd` | 476 | 23 | — | RefCounted | PC-10 (`PC_SHELL_PROPOSAL.md` §3.3 "Modules: tabs you arrange", milestone PC-10; Ruling DY, PQ1): |
| `shell/context_nav.gd` | 313 | 23 | — | RefCounted | PC-7's leftover (`PC_SHELL_PROPOSAL.md` §3.6 "One host with a back stack", finding W2): the one place that |
| `shell/context_note.gd` | 503 | 18 | — | RefCounted | PC-5 (`PC_SHELL_PROPOSAL.md` §3.4 "The Note tab", milestone PC-5; Ruling DY, VQ1, VQ4, VQ6): the Context |
| `shell/context_window.gd` | 391 | 18 | — | RefCounted | PC-7 (`PC_SHELL_PROPOSAL.md` §3.6, milestone PC-7; Ruling DY PQ3): the context windows that do |
| `shell/data_manager_window.gd` | 4251 | 121 | DataManagerWindow | AcceptDialog | MS-1 S4.5 (`window_represent.gd`, `S4_INVENTORY.md` section 18.7): the contract's shared helpers. |
| `shell/dcc_icons.gd` | 408 | 4 | DccIcons | RefCounted | The drawn glyph set (`DCC_SHELL_SPEC.md` §12). |
| `shell/dcc_settings.gd` | 2470 | 173 | DccSettings | RefCounted | Persisted, user-configurable shell state: the four storage roots |
| `shell/dcc_shell.gd` | 12801 | 348 | DccShell | Control | The DCC editor frame (`DCC_SHELL_SPEC.md` §1-§3, §6, §7, §9, §11). |
| `shell/dcc_theme.gd` | 2217 | 47 | DccTheme | RefCounted | Colour tokens and StyleBox factories for the DCC shell. |
| `shell/dcc_units.gd` | 139 | 8 | DccUnits | RefCounted | Distance-unit conversion and display formatting -- `DCC_SHELL_SPEC.md` §2.5's |
| `shell/dcc_widgets.gd` | 5312 | 110 | DccWidgets | RefCounted | The disclosure grammar (`UI_SHELL_DESIGN.md`) and the row vocabulary every |
| `shell/diagnostic_report.gd` | 441 | 13 | DiagnosticReport | RefCounted | `Help ▸ Save diagnostic report` -- `LARGE_ITEM_RULINGS.md`'s "Build" ruling |
| `shell/diagnostic_review_dialog.gd` | 155 | 4 | DiagnosticReviewDialog | RefCounted | The review panel in front of `Help ▸ Save diagnostic report`. |
| `shell/element_chips.gd` | 121 | 5 | — | PanelContainer | Ruling EO (2026-11-11): **on touch, a tap on a map element selects it and shows its action chips** -- Move, Rename, |
| `shell/engine_bridge.gd` | 10172 | 737 | EngineBridge | Node | The one place the shell touches `WorldGen`. |
| `shell/export_dialog.gd` | 725 | 31 | — | AcceptDialog | **The single-image export dialog** (`EXPORT_SCOPE.md` §7, milestone E5): |
| `shell/export_overlay.gd` | 472 | 16 | ExportOverlay | Node | The map overlay drawn for a 16K/32K image export (`EXPORT_SCOPE.md` §7, |
| `shell/faction_banner.gd` | 130 | 4 | FactionBanner | Control | `_civFactionBannerCanvas` (reference line 14849) -- `PARITY_AUDIT.md` §5 |
| `shell/faction_roster_window.gd` | 4232 | 137 | FactionRosterWindow | AcceptDialog | `civOpenFactionsBtn` → `civFactionsModal` (`_civOpenFactionsModal`, |
| `shell/gen_info_dialog.gd` | 276 | 12 | GenInfoDialog | AcceptDialog | The reference's ℹ️ `#genInfoBtn` / `#genInfoPanel` / `generationInfoText()` |
| `shell/generation_rules_window.gd` | 550 | 18 | GenerationRulesWindow | AcceptDialog | Design canvas artboard `1h` ("Generation rules"), `lazy-riding-piglet.md` |
| `shell/global_tools.gd` | 841 | 42 | GlobalTools | RefCounted | §4.5.1's Measure and Region select -- present in every domain, so unlike |
| `shell/info_tiles.gd` | 392 | 24 | — | RefCounted | LOD-D7 (Ruling AV) -- deep-zoom tiles for the Layers popover's info views. |
| `shell/journey_planner_view.gd` | 6572 | 164 | JourneyPlannerView | Control | The Journey Planner as an in-shell tool takeover -- `JOURNEY_PLANNER_SPEC.md`'s |
| `shell/landmark_filter_panel.gd` | 168 | 5 | — | VBoxContainer | The landmark filter's controls (owner, 2026-10-19: *"In the layers I'd also |
| `shell/landmark_types_window.gd` | 315 | 12 | — | AcceptDialog | `Assets ▸ Landmark types…` -- Ruling EH item 2 (`LARGE_ITEM_RULINGS.md`, owner 2026-11-08). |
| `shell/layers_popover.gd` | 966 | 17 | LayersPopover | PopupPanel | The map canvas's Layers popover (`DCC_SHELL_SPEC.md` §9's layers button), |
| `shell/lineart_refine.gd` | 310 | 12 | — | RefCounted | AP-10 (Ruling CR, 2026-10-18): the Asset library inspector's line-art |
| `shell/llm_action_cards.gd` | 154 | 7 | — | RefCounted | AC-7 (`ASSISTANT_ACTIONS_PROPOSAL.md` milestone AC-7; `PC_SHELL_PROPOSAL.md` s.3.5 "Cards"): the WORDS of an |
| `shell/llm_actions.gd` | 416 | 11 | — | RefCounted | AC-1 (`ASSISTANT_ACTIONS_PROPOSAL.md` s.3.1; Ruling DW): the shell half of the ACTION CATALOGUE. |
| `shell/llm_assistant_window.gd` | 2943 | 132 | LlmAssistantWindow | AcceptDialog | **The Assistant panel** (`LLM_ASSISTANT_SCOPE.md` milestone LLM-3: prose and Vault assist, |
| `shell/llm_consent.gd` | 212 | 6 | LlmConsent | RefCounted | **The consent screen** (`LLM_ASSISTANT_SCOPE.md` s.1 rule 6, s.4 rule 6; Ruling DD |
| `shell/llm_edit.gd` | 219 | 8 | — | RefCounted | LLM-7 (`LLM_ASSISTANT_SCOPE.md` s.5c; owner Rulings DD Q10, DI): the shell half of the Assistant's |
| `shell/llm_facts.gd` | 67 | 1 | LlmFacts | RefCounted | Reads the world for the assistant's read-only lookups (`LLM_ASSISTANT_SCOPE.md` s.5b.5, |
| `shell/llm_gpu_rule.gd` | 117 | 7 | LlmGpuRule | RefCounted | **The local-provider GPU rule, applied** (Ruling DF; `LLM_ASSISTANT_SCOPE.md` s.5a). |
| `shell/llm_knowledge.gd` | 91 | 6 | LlmKnowledge | RefCounted | The shell half of the assistant's knowledge base (`RAG_KNOWLEDGE_BASE_RESEARCH.md` milestones RK-5 |
| `shell/llm_models.gd` | 224 | 15 | LlmModelList | Node | One model listing for the Assistant settings dropdown, and the in-memory cache of the last |
| `shell/llm_net_hint.gd` | 97 | 5 | LlmNetHint | RefCounted | The Android "network access may be off" hint and the button that opens the app's system page. |
| `shell/llm_place_facts.gd` | 425 | 10 | LlmPlaceFacts | RefCounted | Gathers the world's own data for ONE place, and the Vault notes that bear on it, for the |
| `shell/llm_profile_switch.gd` | 121 | 5 | LlmProfileSwitch | VBoxContainer | **The profile switch** (Ruling DQ, `LLM_ASSISTANT_SCOPE.md` s.5g): a "Profile" dropdown listing the |
| `shell/llm_profiles.gd` | 338 | 21 | LlmProfiles | RefCounted | **Assistant profiles** -- the shell side (`LLM_ASSISTANT_SCOPE.md` s.5g, Ruling DQ, owner request |
| `shell/llm_region.gd` | 333 | 20 | — | RefCounted | LLM-7 (`LLM_ASSISTANT_SCOPE.md` s.5c; owner Ruling DD Q10): the REGION INPUT of the Assistant's |
| `shell/llm_run.gd` | 412 | 14 | LlmRun | Node | One assistant reply from request to last token, on the MAIN thread |
| `shell/llm_settings_window.gd` | 1374 | 66 | LlmSettingsWindow | AcceptDialog | **`Preferences ▸ Assistant…`** -- the settings section for the optional LLM assistant |
| `shell/llm_view.gd` | 312 | 10 | — | RefCounted | LLM-6d (`LLM_ASSISTANT_SCOPE.md` s.5b.8 item 12, s.5b.10; Rulings DD and DI): the shell half of |
| `shell/llm_wire.gd` | 270 | 12 | LlmWire | RefCounted | The Godot half of the assistant's hybrid transport (`LLM_ASSISTANT_SCOPE.md` LLM-3; |
| `shell/loading_messages.gd` | 720 | 7 | — | RefCounted | The busy overlay's message bank: one line of wit per step, the HTML's voice ported whole and extended. |
| `shell/look_history.gd` | 299 | 15 | — | RefCounted | One shared undo/redo history for LOOK edits (owner Ruling BY item 2, |
| `shell/markdown_editor.gd` | 3652 | 161 | — | VBoxContainer | The vault's Markdown editor -- owner Ruling BE (`LARGE_ITEM_RULINGS.md`, |
| `shell/markdown_highlighter.gd` | 365 | 17 | — | SyntaxHighlighter | Write-mode colouring for the vault's Markdown editor (VE-3, `VAULT_EDITOR_PROPOSAL.md` s.3.1, s.5). |
| `shell/menus.gd` | 8670 | 235 | DccMenus | RefCounted | The seven program menus (`DCC_SHELL_SPEC.md` §2). |
| `shell/new_world_dialog.gd` | 3136 | 86 | NewWorldDialog | AcceptDialog | MS-1 S4.5 (`window_represent.gd`, `S4_INVENTORY.md` section 18.8): the contract's shared helpers. |
| `shell/notes_find.gd` | 70 | 1 | — | RefCounted | The map's Find `n ` scope: the vault's notes (VE-9, `VAULT_EDITOR_PROPOSAL.md` s.7). |
| `shell/open_project_dialog.gd` | 1850 | 46 | OpenProjectDialog | AcceptDialog | File ▸ Open project…, drawn from the "Open project dialog 1920" screen in |
| `shell/overlay_adjust.gd` | 708 | 34 | — | RefCounted | ET-C3 (`EDITING_TOOLS_SCOPE.md` section 5; Ruling BX item 4): the third per-layer |
| `shell/paint_original.gd` | 246 | 10 | — | RefCounted | Ruling BR (`LARGE_ITEM_RULINGS.md`, 2026-09-28): while a paint tool is armed |
| `shell/pen_link.gd` | 344 | 27 | PenLink | Node | The GDScript facade of the OnePlus Stylo 2 Pen link (Ruling EK; STYLO2_PEN_PROPOSAL.md sections 4, 5 and 6.3). |
| `shell/phone_domain_fan.gd` | 178 | 9 | — | Control | MS-2: the domain fan. Holding the phone's domain tab opens three targets, WORLD · GENERATE, CIVIL and CARTO, |
| `shell/phone_menu.gd` | 2472 | 92 | PhoneMenu | Control | The phone menu, built to `design/Cartalith Android Phone.dc.html` and, since MS-4, to the mobile shell mockup |
| `shell/phone_project_picker.gd` | 533 | 14 | PhoneProjectPicker | AcceptDialog | The Android phone's entry screen. The owner's locked `docs/ANDROID_UI_SPEC |
| `shell/pin_marker.gd` | 1056 | 38 | — | RefCounted | CL-1 / CL-2 (`CITY_MAP_LOOK_SCOPE.md` section 4.2, milestones CL-1 and CL-2; Ruling EW item 3): the one teardrop |
| `shell/pin_palette.gd` | 519 | 21 | — | VBoxContainer | The pin palette (Ruling FD, `PIN_PALETTE_SCOPE.md` PP-3/PP-4/PP-5): the right dock's body while CARTO's Pins tool (tool id |
| `shell/place_editor_window.gd` | 2972 | 62 | PlaceEditorWindow | AcceptDialog | `placeEditPopup` / `_civPopulatePlaceEditor` (reference 16694) -- |
| `shell/place_search.gd` | 412 | 14 | PlaceSearch | RefCounted | The world-entity search index `Edit ▸ Find on map…` (menus.gd ~532) was |
| `shell/planner_window.gd` | 235 | 6 | — | AcceptDialog | PC-8 (`PC_SHELL_PROPOSAL.md` §3.7, milestone PC-8; Ruling DY PQ4) and PW-4 (`POPOUT_WINDOWS_SCOPE.md`, |
| `shell/popout_geometry.gd` | 151 | 4 | PopoutGeometry | RefCounted | Where a popped-out window should be put back (`POPOUT_WINDOWS_SCOPE.md` s.4 and PW-1, |
| `shell/popout_host.gd` | 597 | 30 | PopoutHost | RefCounted | The one place that decides whether a tool window is a native OS window |
| `shell/popout_key_forwarder.gd` | 49 | 1 | PopoutKeyForwarder | Node | Carries the three global keys out of a native pop-out (`POPOUT_WINDOWS_SCOPE.md` s.2.5 and PW-2 |
| `shell/quick_bar.gd` | 762 | 48 | DccQuickBar | Container | The quick cluster: the user's pinned commands, drawn at the right end of the tool |
| `shell/quick_bar_menu_pin.gd` | 126 | 7 | QuickBarMenuPin | Node | **Right-click a menu row to pin or unpin it** -- Blender's "Add to Quick Favorites" gesture |
| `shell/quick_bar_phone.gd` | 216 | 12 | QuickBarPhone | RefCounted | The phone's quick bar: where a pinned command lives on a handset |
| `shell/quick_bar_ring.gd` | 190 | 9 | QuickBarRing | RefCounted | What the user's pins become ON THE CARTWHEEL (`CUSTOM_QUICK_BAR_SCOPE.md` s.4, milestone CQ-4, |
| `shell/quick_bar_window.gd` | 1099 | 51 | QuickBarWindow | AcceptDialog | **`Preferences ▸ Quick bar…`** -- the Customise window for the user's pinned commands |
| `shell/quick_drawer.gd` | 538 | 33 | — | Control | MS-11 (`MOBILE_SHELL_PROPOSAL.md` section 2.4, Ruling DY Q10 to Q12): the phone's quick drawer. The quick bar's |
| `shell/quick_pin_reorder.gd` | 158 | 13 | — | RefCounted | CQ-6's touch reorder (`CUSTOM_QUICK_BAR_SCOPE.md` CQ-6; the section 12 table's "long-press then drag"): a pin held under a |
| `shell/radial_ring.gd` | 1208 | 48 | — | Control | `MAP_CONTEXT_SCOPE.md` §5 and §9.1 -- CM-3, the desktop ring. Draws the |
| `shell/ramp_bar.gd` | 394 | 17 | — | Control | The elevation colour ramp as an interactive gradient bar (editing-tools plan, |
| `shell/resource_overlay.gd` | 143 | 4 | ResourceOverlay | PanelContainer | The reference's `#resOverlay` (`updateResOverlay`/`toggleResOverlay`, |
| `shell/right_dock.gd` | 9192 | 292 | RightDock | Node | Controller for the right dock's content (`DCC_SHELL_SPEC.md` §6): |
| `shell/rock_paint_ui.gd` | 121 | 10 | — | RefCounted | GF-8 (`GEOLOGY_FIRST_SCOPE.md` §4.13, owner Ruling BJ): the shell's shared |
| `shell/saved_journeys_window.gd` | 278 | 11 | — | AcceptDialog | Story ▸ JOURNEYS ▸ Saved journeys… -- milestone SM-6 of |
| `shell/section_strip.gd` | 390 | 13 | SectionStrip | PanelContainer | The cross-section's own strip — |
| `shell/settlement_type_store.gd` | 308 | 14 | SettlementTypeStore | RefCounted | `lazy-riding-piglet.md` Batch D -- the Settlement Editor's artboard 1f, |
| `shell/settlement_types_window.gd` | 414 | 11 | SettlementTypesWindow | AcceptDialog | `lazy-riding-piglet.md` Batch D -- artboard `1f` |
| `shell/shortcuts_dialog.gd` | 604 | 27 | ShortcutsDialog | AcceptDialog | `Help ▸ Keyboard reference…` (titled Keyboard shortcuts… until PC-9), which was a `_todo` reading *"No shortcut table |
| `shell/stylus_input.gd` | 136 | 12 | StylusInput | RefCounted | Standard stylus input helpers (Ruling EE: pressure, tilt, hover and touch as Godot 4.7.2 delivers them; no |
| `shell/stylus_tester.gd` | 280 | 11 | StylusTester | Control | SH-9 (`STYLUS_HAPTICS_RESEARCH.md` section 8.1), in the spirit of Krita's tablet tester: a diagnostic that shows what |
| `shell/tool_bar.gd` | 1547 | 76 | DccToolBar | RefCounted | The unified Sculpt / Paint / Measure tool bar — |
| `shell/tool_fan.gd` | 919 | 31 | — | Node | CM-6 (`MAP_CONTEXT_SCOPE.md` §8.2, §11; Ruling AX F2): the phone's thumb |
| `shell/tool_overlay.gd` | 548 | 22 | ToolOverlay | Control | §4.5.1's two global-tool overlays: the Region select marquee and the |
| `shell/tool_strip.gd` | 450 | 16 | — | PanelContainer | MS-5, second step (`MOBILE_SHELL_PROPOSAL.md` section 2.1 "The armed tool", section 5's "`tool_fan.gd` pill plus |
| `shell/trade_store.gd` | 255 | 14 | TradeStore | RefCounted | The last trade-flow match AND the last food-shed pass, held on the shell |
| `shell/travel_library_window.gd` | 1236 | 33 | TravelLibraryWindow | AcceptDialog | MS-1 S4.5 (`window_represent.gd`, `S4_INVENTORY.md` section 18.7): the contract's shared helpers. |
| `shell/undo_group.gd` | 880 | 46 | — | RefCounted | LLM-5 (`LLM_ASSISTANT_SCOPE.md` s.2, s.3, s.5b.4; Ruling DD Q10): the |
| `shell/urban_layout_draw.gd` | 1423 | 17 | UrbanLayoutDraw | RefCounted | The one place a `urban_layouts()` dictionary is turned into strokes. |
| `shell/urban_layout_look.gd` | 872 | 27 | — | RefCounted | The map look for a town plan (CITY_MAP_LOOK_SCOPE.md milestone CL-1, Ruling EW). |
| `shell/urban_shore.gd` | 394 | 15 | — | RefCounted | Smooth sea and lake shorelines for the town layouts (owner, 2026-11-15: "Sea and lake should not draw as |
| `shell/vault_compare.gd` | 67 | 1 | VaultCompare | RefCounted | VE-4's Compare (`VAULT_EDITOR_PROPOSAL.md` s.7a): the pure half. A line diff of the editor's text |
| `shell/vault_folder_pick.gd` | 332 | 11 | — | RefCounted | The folder chooser for a new vault note (owner, 2026-11-13: "make sure that from the note ... we also get the |
| `shell/vault_privacy.gd` | 108 | 10 | — | RefCounted | The private badge's cache (Ruling DP; VE-9): which vault notes are private, so every list and link that |
| `shell/vault_saf_android.gd` | 226 | 18 | — | RefCounted | The real driver `vault_saf_handler.gd` talks through on Android: `DocumentsContract` and |
| `shell/vault_saf_connect.gd` | 73 | 5 | — | RefCounted | Connecting an Android folder as the vault (Ruling AV; VE-6's Android half): the picker, the persisted |
| `shell/vault_saf_handler.gd` | 365 | 15 | — | RefCounted | The Android Storage Access Framework half of the vault, as the `_saf_dispatch` handler |
| `shell/vault_side_panel.gd` | 781 | 42 | — | VBoxContainer | The vault editor's side panel (VE-5 of `VAULT_EDITOR_PROPOSAL.md` s.3.2, s.3.5): **Links** |
| `shell/vault_store.gd` | 361 | 7 | VaultStore | RefCounted | Persistence for the Markdown Vault's knowledge links |
| `shell/vault_template_chooser.gd` | 86 | 3 | — | RefCounted | VE-6's template chooser (`VAULT_EDITOR_PROPOSAL.md` VE-6, audit `GUI_PLANS_AUDIT_2026-11-16.md`): the one control a new |
| `shell/vault_window.gd` | 5938 | 184 | VaultWindow | AcceptDialog | MS-1 S4.5 (`window_represent.gd`, `S4_INVENTORY.md` section 18.8): the contract's shared helpers. |
| `shell/view3d.gd` | 862 | 48 | — | Control | No `class_name`, deliberately: `viewport_host.gd` preloads this file as |
| `shell/view3d_camera.gd` | 294 | 23 | — | RefCounted | No `class_name`, deliberately: `view3d.gd` preloads this file, so it works in |
| `shell/view3d_gate.gd` | 675 | 32 | — | Node | No `class_name`, deliberately: every user preloads this file |
| `shell/view3d_quadtree.gd` | 749 | 38 | — | Node3D | No `class_name`, deliberately: `view3d.gd` preloads this file (3D-1's reason). |
| `shell/view3d_terrain.gd` | 507 | 26 | — | Node3D | No `class_name`, deliberately: `view3d.gd` preloads this file (3D-1's reason). |
| `shell/viewport_host.gd` | 5835 | 162 | ViewportHost | Control | The map surface (`DCC_SHELL_SPEC.md` §9). |
| `shell/water_anim_layer.gd` | 130 | 6 | WaterAnimLayer | ColorRect | Animated water (`GUI_GAP_REGISTER.md` RN-01) -- the reference's own |
| `shell/wind_fx_layer.gd` | 512 | 12 | WindFxLayer | Control | Animated particle streaks over the Wind and Ocean-currents field views -- |
| `shell/window_represent.gd` | 61 | 3 | — | RefCounted | MS-1 S4.5 (`MOBILE_SHELL_MS1_SPEC.md` section 3.5, Ruling DY MQ-5; `S4_INVENTORY.md` sections 5 and 18): the helpers |
| `shell/workspaces/cartography_workspace.gd` | 4152 | 107 | CartographyWorkspace | Workspace | CARTO domain (§7 + §4.5.5): layer list, layer properties, colour ramp, stop |
| `shell/workspaces/civilization_workspace.gd` | 8225 | 264 | CivilizationWorkspace | Workspace | CIVIL domain (§3): settlements, population, economy, politics, culture, |
| `shell/workspaces/infrastructure_workspace.gd` | 2003 | 77 | InfrastructureWorkspace | Workspace | Roads, rivers, ports, trade, logistics, and the Way/Route drawing tools |
| `shell/workspaces/render_workspace.gd` | 4079 | 107 | RenderWorkspace | Workspace | Terrain appearance groups; right dock preview & quality -- formerly the |
| `shell/workspaces/workspace.gd` | 357 | 16 | Workspace | VBoxContainer | Base for the five domain workspaces (`DCC_SHELL_SPEC.md` §3). |
| `shell/workspaces/world_workspace.gd` | 5244 | 175 | WorldWorkspace | Workspace | WORLD domain (`DCC_SHELL_SPEC.md` §5): a two-button switch between the |
| `shell/world_data_window.gd` | 646 | 20 | WorldDataWindow | AcceptDialog | Data ▸ World data tables. **No design draws this window.** |
| `shell/world_delete.gd` | 82 | 4 | — | RefCounted | The red trash can on every world card, and the confirmation that stands between a tap on it and |
| `smoke_test.gd` | 22 | 1 | — | SceneTree |  |

### Disabled menu rows (`menus.gd` `_todo`)

Thirty-seven `_todo(` calls. **Four are unconditional** (always disabled): *Export ▸ Print sheet (PDF)…* and *VTT battle map (grid + walls)…* (routed by `STORY_MENUS_PROPOSAL.md` section 8 and `OUTSTANDING_WORK.md`'s "Story menus: the proposal's §8 later rows", unscheduled by Ruling EA item 7), *Preferences ▸ Anti-aliasing · anisotropy* and *3D viewport defaults* (both in `UNWIRED_FUNCTIONS.md` and `GUI_GAP_REGISTER.md`). The rest are fallbacks that show only when a binding is missing from an older native library or a state is empty (Redo, selection history, GPU devices, viewshed refine, atlas export/import/cap, Forget layout, the Context-panel tab rows, Documentation outside a checkout, Open panels in their own window where sub-windows are unsupported).

## 8. Settings keys (`DccSettings`)

| Section constant | Keys read or written by name |
|---|---|
| `_SEC_ASSET_LIBRARY` | `favourites`, `recent` |
| `_SEC_ATLAS` | `cap_gb` |
| `_SEC_AUTOSAVE` | `enabled`, `minutes` |
| `_SEC_CITY_LOOK` | `style` |
| `_SEC_CONTEXT_PANEL` | `tabs`, `version` |
| `_SEC_CPU` | `threads` |
| `_SEC_GPU` | `devices`, `enabled`, `fallback`, `mode`, `vram_budget_gb` |
| `_SEC_GRAPHICS` | `exag` |
| `_SEC_HANDEDNESS` | `hand` |
| `_SEC_LANDMARKS` | `hidden_kinds`, `visible`, `zoom_law` |
| `_SEC_LAYOUT` | `named`, `seeded` |
| `_SEC_LLM` | `consented_origins`, `rag_hosted_origins` |
| `_SEC_LLM_PROFILES` | `data` |
| `_SEC_LOD` | `auto_on_zoom`, `bake_depth` |
| `_SEC_MARKER_STYLE` | `style` |
| `_SEC_PIN_PALETTE` | `recents` |
| `_SEC_POPOUT` | `version` |
| `_SEC_QUICK_BAR` | `version` |
| `_SEC_RECENT` | `paths` |
| `_SEC_STYLUS` | `double_tap_`, `low_battery_chip`, `pen_draws_finger_pans`, `pen_link_on`, `pressure_`, `pressure_curve`, `pressure_max`, `slide_`, `slide_invert`, `tilt_to_size` |
| `_SEC_THEME` | `mode` |
| `_SEC_UI_SCALE` | `scale` |
| `_SEC_UNITS` | `mode` |

Keys written through a variable (per-window sizes, quick-bar pins, layouts) are not in this table.

## 9. Method

Python over the working tree at the commit above, no build: `Cargo.toml` `[dependencies]` and `[dev-dependencies]` per crate; regex counts of `pub fn`, `pub struct|enum|trait|type`, `#[test]` and `#[ignore` per file; the first `//!` (Rust) or `##` (GDScript) line per file; a token index of every `.rs` and `.gd` file for the dead-code and documentation checks; `#[func]` names collected so exported bindings are not called dead. Searches stayed inside the repository.

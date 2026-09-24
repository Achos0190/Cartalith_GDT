# ALIGNMENT_AUDIT.md — code against documentation, 2026-09-24

**What this is:** the findings of the code-versus-documentation alignment audit
the owner asked for on 2026-09-23 ("a proper audit that every document still
aligns and what needs fixing before we get any more regressions"). Two
read-only auditors ran on 2026-09-24 at `e4bc522`–`f163c52`: one over the
civilisation half, one over engine and rendering. Every headline finding was
re-opened at its symbol; confidence is stated where it was not. **What it is
not:** status or a to-do list. It is a dated trail, like `PARITY_AUDIT.md`.
The work it found is routed from `OUTSTANDING_WORK.md` §2.11; status stays in
`cartalith-native/docs/STATUS.md`. Line numbers here are conveniences and will
drift; the symbols are what to search for.

Owner decisions taken on the audit's questions the same day: Rulings AQ and AR
(`LARGE_ITEM_RULINGS.md`).

---

## Part 1 — civilisation

### A. Engine defects (no backlog row)
1. Reopened projects: project_open -> load_save sets WorldSource::Loaded, then restores self.civ. jp_compute, jp_plan_for_route, story_bridge plan_saved_journeys/resnap_carried_journeys, urban_bridge urban_layouts/settlement_diagnostics, civ_military_bridge (3), civ_trade_bridge (4 incl trade flows, food shed), civ_faction_economy, civ_faction_terrain_fits all match WorldSource::Generated and return nothing. User told "a loaded save carries no civilisation layer" (false for .ctl projects): lib.rs jp_compute/jp_plan_for_route errors; data_manager_window.gd GEOJSON_CIV_NOTE, _gis_count civ_absent, atlas note; right_dock.gd:308; civilization_workspace.gd:712,:1604; infrastructure_workspace.gd:867; geojson_bridge.rs/civ_trade_bridge.rs docs. Other wrong empty reasons: City Viewer "open water", Diagnostics "native library is older", Roster "Reopen the roster", Military "generate a world first". _jprestore_probe never computes a plan on the reopened world. OWNER Q4.
2. Settlement naming ignores edited faction culture: civ_settle_name uses civ_default_culture(faction)=CIV_CULTURES[fid%7]; callers lib.rs:5842,:7875, naming.rs:111. Reference reads civFactionCulture[faction] (v2.10:20718). civ_default_culture doc falsely says no UI can change it (FactionEntry.culture edited by roster & culture windows). Fix: thread roster culture into naming.
3. Recovery phase leaves way endpoint indices stale: compute_civilisation builds ways + village connectors before civ_apply_recovery which drops/re-indexes settlements (~2631-2642); ways never remapped; road_edges un-remapped. Readers: trade RoadComponents/WayRouter, belief_links_from_ways, manpower navigability, project_bridge saves from: w.a_idx. Reachable: New World > Recovery phase != Stable. trade.rs module doc "cannot change a number" false under recovery.
4. Provinces built from unpainted territory: civ_generate_provinces at lib.rs:2737 on raw assign_territory; CivTools::rebase merges territory_paint later (6123, 6228). Reference _civGenerateProvinces reads painted civTerritory. geojson.rs::province_feature "No clipping needed" assumes opposite.
5. Manpower plausibility note (civilization_workspace.gd _fill_manpower ~L6252) takes MIN concentration_ratio but claims "no faction can concentrate more than"; worst:=1.0 and get(...,1.0) print "100%" for no value.
6. generate.rs building_ruined.resize comment: build_faith_sites only removes (retain) -> flags shift; nothing reads Town::building_ruined today.

### B. False user-visible strings
7. Landmarks "[no viewshed]" tag/text false (civilization_workspace.gd _lm_types note ~4742, _lm_type_row tooltip ~4872, tag ~4948; phone_menu.gd:2233, 2271-2272). Derived::vis read by fort, watchtower, volcanic, border-marker pools (landmark.rs ~4069/4223/4308). Peak flagged needs_viewshed but reads no vis. _landmark_probe asserts the false tag.
8. 22 not_built reasons never shown (no .gd reads the key; shows LM_LIMIT_WHY["not_buildable"] "no placement rule ... yet"). Comments claiming shown: phone_menu.gd, LandmarkKindSpec::buildable doc, landmark_kinds doc. Several reasons false: monument (battles are Conflict records reaching LandmarkInputs::battles), ruin/abandoned_settlement (collapse_flags + per-step snapshots exist; "(SP-4)" stale), ancient_road/historic_crossing ("not retained" — TimelineSnapshot.ways persisted), delta/glacial_feature (route_sediment, glacial_kernel, landform cirque exist), river_crossing overstated.
9. Landmark causal text "N settlements within reach" = total count c.inp.settlements.len() (landmark.rs ~3626, 3701, 3874); shown in right dock CTX_LANDMARK.
10. right_dock.gd ~2971 "Analyse catchment" tooltip names Portage's reason for river_confluence.
11. infrastructure_workspace.gd::build_trade_gaps_into (~647) "Prices · tariffs · caravans ... None of the four is derivable" — prices (AB), tariffs (AE), caravans built; Ruling R decided currency.
12. Military "Not built" (~6047) "campaign needs a clock ... none exists ... needs a decision" stale (SP-4 Conflict); scope + STATUS MM-6/7/8 say declined. OWNER Q8.
13. civilization_workspace.gd: :1487 "three vocabulary fields drive nothing" (false); :2481 "nothing ties a trade relationship to the road or sea lane" (false); :1558-1561 "not a per-year ownership grid" (TerritoryFrame exists); Economy by faction "Tax income and five-axis power ... not drawn" (power.military drawn); Population "Province-level has no binding"; Populate "seven civilisation parameters" (13). world_data_window.gd:493 "faction-level aggregation remains unstarted" (civ_faction_economy exists).
14. Urban strings: a) civilization_workspace.gd:2251 "bridge/ford not surfaced by any binding" (layout_dict emits bridges/ford); b) right_dock.gd:2425 "adapter does not carry that field"; c) city_viewer_window.gd:510 "Ten lines above" (12 stages), "all 29 reference stages" hides 3 port-only stages, not-drawn list names civic/churches wrong & omits harbour works; urban_adapter.rs 3 places; _entwin_probe CV6 asserts 10; d) city_viewer_window.gd:593 Culture tooltip "no faction-culture table" (+ urban_adapter.rs header, run_layout comment, URBAN scope m17); e) no-wall note gate on wall_spec=="none".
15. journey_planner_view.gd:3709 "jp_plan does not return one" (load term exists; GUI_GAP JP-11 stale); app.gd:2171 & dcc_shell.gd:8874 (+comment 4926) "CIVIL > Politics" -> category is "Timeline" (STATUS TL-6 too); travel_library_window _build_validation_banners "re-plans N saved journeys" for vehicles/slotless animals; journey_resnap_report no shell caller (Ruling AO "reported by name"); restored-journey "Route #-1" nit.
16. Saved journeys persist only 20-key PartyPreset: lost desert_water, weather_override, seasonal_closures, route_cond, infra, season_drift, rest_cadence, auto_promote, stage overrides, layovers, animals, trim, auto_carriage, auto_stage -> SP-2 markers/SP-3 dates planned from defaults. OWNER Q3.
17. Vault: "Unlink" calls vault_disconnect directly; VaultStore.save_from only on store_changed -> rebinds next launch; "Re-scan" never saves index; VAULT_NOTE "three entity kinds" (six).
18. place_editor Polity tooltip self-contradicts; Class tooltip "Settlement tool refuses Metropolis" (not since 08-20); dead nav "Civilization > Trade > Match trade flows", "Cartography > Roads & routes"; generation_rules_window "1.00 reproduces DEFAULT_RULES exactly" false after drag; faction_roster _build_gaps lists food/exports/resources unbuilt; landmark limit token `unrecorded` has no LM_LIMIT_WORD/WHY.

### C. STATUS rows contradicted
19. EC-8 "blocked" -> partial (civ_faction_economy lib.rs:17016 45b368d + _fill_faction_economy); FR-03 + roster header stale.
20. MV-4 "not started" -> partial (vault_saf.rs SafVaultProvider, vault_connect_saf cff1edc; no _saf_dispatch, no shell caller; no .md mentions vault_saf).
21. SP-2 "no timeline cache ... 48-54 ms" false (JourneyPlanCache b5aff17).
22. SF-3: 18 SLOT_*, 20 DOCUMENT_SLOTS, 14 ENGINE_OWNED; journeys engine-owned; measurements exists; real written-never-read = cartography/tiles/**; OUTSTANDING §6 "fifth save slot unbuilt" stale.
23. Phase 5: Orientation "block-ground fill still fails triangulation" closed 09-13; 20 UM rows not 19; UM-5/6/8A/12 callers in generate.rs; stale line counts; UM-17A settlement_layout_with; no STATUS rows for Ruling H/I/J/AA/AC/AD work.
24. Landmark group: quotes removed scope phrases; "6 needs_viewshed" wrong (4 read vis; peak doesn't; fortified pass & crossing read it unflagged); LM-1 analysis.rs 1056 lines; ruling 12 DECISIONS note missing; ruling 16 refine action never built.
25. TL-6/SP-3 should be partial; SP-3 "(read-only)" but add-event form exists; collapse_flags not persisted.
26. Stale known-defects notes (PHASE2 m17, ECONOMY "not started", MV-3 paragraph; cartalith-vault links.rs identity table "civ is not saved" false); P2-16 place_search declines provinces; P2-13 drawn in map_overlay; GGR-49 CV-12 now working.
27. Counts: JP-1 92 jp_*; MV-1 53 #[func]; JP-QC2/3/4 citations drifted; EC-3 citations; EC-1 "three workspaces".
28. "The last seven days" ends 09-20; empty heading.

### D. OUTSTANDING
29. Untracked gaps: A1-A6; Ruling R; prices/tariffs have no GUI (no engine_bridge wrapper); 5 economy-district fills missing (DISTRICT_FILL: oreyard, fishery, sawyard, granary, warehouse; v2.10:22991); undrawn urban layers (bridge decks, ford, churches, civic/games buildings, harbour piers/mole/defence); legacy flat .zip settlements/labels/icons dropped (read_flat vs SAVEFILE_COMPAT §15); vault restore failures/unresolvable_kind_report never surfaced (§13.3.5 MUST); landmark<->note entry point (ruling 13); settlement_types.json no format spec; re-snap invisible; planner list not rebuilt after regenerate; CV-24 narrative only; JP-10 foraging offset; mutation coverage courtyard.rs, wallside.rs.
30. Stale rows: SP-2 in §3.2; Landmark M8 residual ending; §3.3 vault M4 omits dispatcher; closed urban clutter/Ruling H "culture: None" stale (DETAIL_KINDS_DRAWN doc).

### E. Rulings/scope
31. Ruling R per-faction currencies unbuilt/untracked/not superseded; AB single index built. OWNER Q1.
32. Ruling N: text says civ_is_coastal re-baselines, code left it; x-wrap on non-wrapping maps latent defect, no row. OWNER Q2. Closing "not scheduled" stale (649897f, 9ad4399).
33. AO/AC/AD/AF "Not yet built" stale; AF superseded by AP unmarked; ruling 10 SAVEFILE_COMPAT entry for landmarks.json missing.
34. FUNCTIONAL_CONTRACT: §5 journeys "nothing persists"; §4 19->21 milestones, timeline stale, provinces not golden; §10 File>Save wrong; §13 refinement list closed, 842->1158 lines, "port as-is" vs H/I/J; "GeoJSON import: absent".
35. SAVEFILE_COMPAT: §9.6 journeys engine-owned; §17 counts/example/landmarks draw twice; settlement_types.json absent; §13.3.1 omits snapshots; §5 cartography/ reached, 3 channels.
36. TRAVEL_LIBRARY_SPEC §6.1 csv (now built); STATUS no Travel Library group; ECONOMY_SCOPE EC-1..EC-9 (10); LANDMARK scope M7/M9/§6 stale; MARKDOWN_VAULT §4 survives save; URBAN m17 culture reason; GUI_GAP FR-03, JP-11, ED-03b, UM-01/02/03, CV-12 §18.3/§49, DM-14, DM-15.

### F. False code comments (grouped)
- civ_generate_provinces/Province: 5 false parts (reference _civAutoPolity v2.10:20665 writes civTerritory; golden possible; max_by_key last-tie vs reduce first; numbering not cosmetic (vault keys Province::id); capital_settlement_index "no persistent identity" (tid)); CivData::provinces "no Godot rendering".
- Urban: urban_layout_draw.gd DETAIL_KINDS_DRAWN block (bastioned built c6dc8bf; crossings absent); map_overlay/viewport_host reveal-gate comments; viewport_host "set_civ_data the only invalidation"; wallside.rs "no wealth field" (Parcel::gate_quality); generate.rs "one stage not reference's" (3); cartalith-urban lib.rs module map omits generate/citadel/courtyard/wallside; urban_bridge.rs "whatever buildFaithSites inserted"; doc above urban_town_plan_options belongs to apply_urban_rules_preset.
- Landmarks: landmark_last_run invalidate; landmark_run_inner "no engine signal" (icon_placed_since_run); engine_bridge UNREACHED "no per-landmark panel" (CTX_LANDMARK exists); LandmarkInputs::volcanism; needs_viewshed "does not exist anywhere"; LM_LIMIT_WORD "six variants" (7).
- Vault/save: app.gd journeys "the SHELL's"; project_open "Eleven" (13); cartalith-io DOCUMENT_SLOTS measurements doc; vault_store.gd "eleven siblings" (19); cartalith-vault lib.rs "four modules"/"five status states" (six); vault_window.gd header "three entity kinds" (six), "four write buttons" (seven).
- Journeys: travel_library_window _usage_journeys; journey_planner_view setup/journey_usage/clear_journeys/module doc; jp_plan_ex vs jp_plan_full (travel_bridge.rs, journey_bridge.rs, user-visible trace note).
- Civ: civ_tools_bridge "nothing computes contested" (territory_influence); civ_faction_economy "frees nine of fifteen" (six); civilization_workspace "no get_cultures wrapper" (engine_bridge:3193 has one).

### Owner questions (unruled at audit time)
*Answered the same day by Ruling AR: 1 (R stands), 2 (fix the wrap), 3 (full plan), 4 (save the rasters). The rest remain open.*
1 Ruling R vs AB. 2 civ_is_coastal x-wrap guard. 3 journeys full plan persist? 4 what should a reopened project support. 5 MV-4 folder picking (.aar?). 6 legacy flat .zip import. 7 viewshed refine action & formula; Peak term. 8 Military "Not built" wording. 9 citadel area in growth; citadel for bastioned; courtyard for radial. 10 authored battles spacing.

### Probes pinning stale state
_entwin_probe CV6 stage_lines==10; _landmark_probe "[no viewshed]"; _jprestore_probe no plan on reopened world.

---

## Part 2 — engine and rendering

### A. Behaviour defects (no row)
A1. GPU plate assignment ignores world-wrap and runs on wrapping worlds: cartalith-engine lib.rs generate_terrain_inner `if p.use_gpu { assign_plates_grid_gpu_with(...) }` no world arg/gate; CPU assign_plates wraps; cartalith-gpu dispatch_gpu_assign_plates doc says callers must pass world=false. Live (use_gpu on at boot). STATUS GLI-D2 overclaims (only warp + heterogeneity wrap). Fix: gate p.use_gpu && !world or implement wrap in gpu_jfa_plates.wgsl; retitle GLI-D2.
A2. World Structure on: OrogenyParams{fold_k:0.16, trench_k:1.0, fault_block_k:0.0} hardcoded; crate doc ~30 + inline comment + GENERATION_PARAMETERS "Structured-orogeny tuning" claim these are JS null-coalescing defaults. Reference v2.10: state faultBlock:0.6 (2265), deriveFromWorldStructure sets foldIntensity=0.6+tectonicEnergy, trenchDepth=0.7+0.8*oceanDepth (2536-2538), call 3440-3442 reads them -> fallback never reached. faultBlockK>0 horst-graben never runs. No golden catches. Fix texts now; derivation = parity change needing owner call. OWNER Q.
A3. v2.57 PLATE_BASE_BLUR_K 0.35->0.18 unported: lib.rs `(p.tect.blur_r * 0.35).max(2.0)` at GPU call + both CPU fallbacks; import.rs::infer_tectonics dead `let _base_field = gauss_blur(...)` never read. OUTSTANDING §2.9 survey "NOT confirmed" (grepped wrong crate). No build row. §7n + Ruling AP authorise. Fix: mark confirmed, file v2.57 row, delete dead blur.

### B. False user-visible strings
B1. render_workspace.gd::_build_biome_colours: "A writable biome table · costs a re-baseline ... not built" — Ruling P landed cc0f561 (TerrainAppearance::biome_cols, set_biome_color, reset_biome_color(s), get_biome_color). Missing: no .gd calls set_biome_color (no picker); legend (sample_bridge "bclass" => CART_BIOME_COLS), bclass layer, paint preview (paint_bridge pack_window -> swatch_color) read frozen CART_BIOME_COLS; swatch_color_with has no override caller. No open row.
B2. layers_popover.gd GAP_LAYERS["velo"] "No hydraulic velocity-erosion pass exists" + sample_bridge LAYER_GROUPS velo hint: velocity_erode_kernel exists (cartalith-erosion passes.rs) and runs under passes.velocity; field discarded (`let _ = velocity_erode_kernel(...)`). "oro" labelled missing computation though computed-not-kept. Fix: "runs, but its field is discarded".
B3. menus.gd GPU_TOGGLE_TIP lists 4 GPU stages "Takes effect on next generate"; "the four GPU-eligible substrate stages" in menus.gd + diagnostic_report.gd x2. Real: warp/warp_split, plate_assignment, stress, base_field_blur, heterogeneity, flow, weather; compute_civilisation resource_potentials, settlement_suitability; erode_op thermal (affects next Erode too).
B4. render_workspace.gd APPEARANCE_HELP["ice_strength"] "Inert ... world whose glacial erosion pass never ran, every world at shipped default": build_glacier_potential gates on height/temp/snowline/flow, not passes.glacial; grid path passes 0.0 -> tiles only.
B5. world_workspace.gd _build_param_row "Not exposed by the reference app" for passes.velocity/.glacial/.coastal/.hillslope/.sediment_fill/.tidal_flats (reference_control ""); v2.11 has #veloBtn #glacBtn #coastBtn #diffuseBtn #sedimentBtn #tidalFlatsBtn.
B6. world_workspace.gd: STAGES[5]["gap"] lists "Glacial group's fjord carve" among passes.* (carve_fjords is on-demand #[func]); cites _build_erosion_passes (is _build_erosion_water_ice). "There is no partial recompute in this engine" CATEGORIES["Generate"]["lead"] + notes ~846/849/3821 — recompute_stale_stages + recompute_civilisation exist; true: Generate runs all ten. STAGES[0]["gap"] "Geoid ... no cartalith-engine equivalent" — cartalith-climate geoid.rs ported + golden; only generation-time enable missing; compute_temperature doc repeats.
B7. sea_grain_warp: render_workspace help "0 is reference exact lattice, artifact included... deliberate divergence"; render.rs field doc "divergence, flagged rather than fixed... golden-verified path" (false: golden uses js_reference). js_reference inherits via ..default() -> must pin sea_grain_warp: 0.0 before flipping default. render.rs::sea_grain "why a fix is a divergence", "Above zero ... artifact, flagged" backwards; cites nonexistent surfaceColorSampled (11939); render.rs ~1635/~1967 cite nonexistent render_default_pin.rs (real: tests/color_space.rs FINISHED_RENDER_FNV1A). OWNER Q: flip default?
B8. render_workspace.gd::_build_map_style Antique note "stylized glyph layer not built" — pack.rs composite_map_icons + draw_icon_glyph is it; missing: toggle (#iconsChk) + drawing without pack.
B9. data_manager_window.gd GEOJSON_CIV_NOTE "no point-of-interest kind", import note "no POI concept" — landmarks + POI icon family exist; geojson_bridge hardcodes is_poi:false.
B10. data_manager_window.gd WD_RASTER_NOTE "a dozen bytes of 8,060,928 differ" vs viewport — since d657091 screen bakes no rivers but exports stamp them.
B11. menus.gd Landmark icons tooltip "ten slots" (PACK_POI_SLOTS 8), "mapping the port still owes" (poi_slot_for_landmark exists); Assets FILLED "all eight families" (9); new_world_dialog _update_derived_readout "CPU-only pipeline"; VRAM fallback "CPU tile pass" "world stays correct only slower" vs GPU_TOGGLE_TIP + §7p; Working set "This process's own allocations" = OS.get_static_memory_usage (excludes Rust heap); "Fail with error": engine_bridge.generate refuses on gpu_vram_estimate without checking use_gpu; Split tiles "the only GPU stage ... reads nothing outside its own cell" (heterogeneity per-cell too).
B12. Lower confidence: Render quality "Sample counts" (for_tier switches stages off); Auto on zoom omits LOD_AUTO_ZOOM 2.2; atlas import implies stops synthesis; population density CARTO vs popover; val_repair "No validation pass" vs val_check; sharper-ecotones reason vs bio_jitter hardcoded; _pg_warn -> DccTheme.c("warn").

### C. STATUS contradicted
C1. LOD-D4 "aspect term not built" (19c38d9 built it) — LOD_DETAIL_SCOPE §LOD-D4 present-tense, TERRAIN_APPEARANCE_SCOPE "scheduled", Ruling AP "Scheduled, not yet built", OUTSTANDING snow row "re-baseline golden_parity_render.rs" wrong (parity keeps 0). snow_aspect_c no TUNABLE/GUI; snow_aspect_shift doc sits on tile_snow_facing; TA-20s "FINISHED_RENDER_FNV1A unchanged" + color_space.rs module doc "pinned unchanged afterwards".
C2. "The last seven days" stops 09-20; empty heading.
C3. SL-0 "not started — no instrumented harness": tests/sculpt_live_l0_bench.rs::l0_measure (611c5fa).
C4. Android group "Twelve rows" (16); total "16 — 10 done..." real 12 done (2 done*), 2 unverified, 2 declined.
C5. LODI-M3 atlas "done" but nothing reads atlas back (menus.gd "still write-only"); Orientation "persistent chunk atlas is on screen" overstates; no reader row.
C6. "open owner decisions" list incomplete: LOD-D1 two decisions, LOD-D5 octave decay, LOD-D7 q6; LOD_DETAIL_SCOPE q1-6 unruled; §3.1 no LOD rows.
C7. GPU group header/GLI-M/GLI-E say scope has no milestone M/E (since 0d8a547 it does); scope table "E | (no row)" stale; CPU-6 quotes removed sentences; MEM-8 "never edited"; MEM-1 release location moved (7228bb4); CPU-5 build_water_bodies now 4 call sites (compute_civilisation, build_color_texture, get_rivers, build_sculpt_preview_texture) — cost unmeasured; GLI-8 "one device per call" cached (Ruling Y); no row for CPU worker pool (ensure_thread_pool, set_configured_thread_count).
C8. counts: render_workspace.gd 2123 lines not 1055; AL-9 189132 bytes, nine families; MVP-1 94 golden files; GLI-M multi.rs 1653; LODI-M1 lod_bridge.rs 1846, shade_tile off deep-zoom path; TA-4F get_border_inset_frac; EXP-E4 five #[func]s; LOD group ROADMAP "originally ended"; STATUS 1668 reference/ only v2.10.

### D. OUTSTANDING
D1. §2.9 survey "confirmed digging pass" false: enforce_channel_descent call is v2.11's own carveRiverValleys step 2 (8800-8822); reverted pass is v2.60 §6l step 2c (CHANNEL_DESCENT_CENTRE_HALFW) — zero hits; port complies with §6m.4. §6r.5 wording fixed 86ccf73 (PARCEL_GRANT_MAX_SPIN). Multi-ridge row hint cartalith-spatial::contour wrong; junction defect in trace_boundaries.
D2. Caveat 2 "how many ported not established"; no rows for v2.47-v2.53, v2.55 A/B, v2.57, §2.1/§2.2, §4 v2.25 tileShadeExag, §6.1/§6.2 Tobler, §6m.3 lake flow gate, §6n, §6s. Lane has a section->symbol table (can resend).
D3. §5 "bounded thread pool — declined" but built (ensure_thread_pool, set_configured_thread_count, menus _build_cpu_threads_menu); §5 hard-hazard bullet GPU ceiling stale; §3.3 "no Android GPU compute path" vs multi.rs LAST_BACKEND doc; "Nine kernels" (15).
D4. Seven docs say reference/ holds only v2.10 (v2.11 + FUNCTION_INDEX_v2.11 since 45b368d): OUTSTANDING §2.8 re-freeze row + line ~3168; STATUS 1668; FUNCTIONAL_CONTRACT 28, 41 (v2.73); DECISIONS §7o 1273; RC_ENGINE_CHANGES span table; README reference/ row. DCC counts disagree (§2.8 44 files v2.66; STATUS/FUNCTIONAL_CONTRACT 49 v2.71).

### E. Scope/contract/rulings
E1. FUNCTIONAL_CONTRACT absent-list "vector river overlay absent" (map_overlay._draw_rivers exists); §11 atlas contradicts §6; "All ten styles" (11, Npr::village); §1 golden-verified without depression-filled routing (§7o).
E2. PORT_ONLY_FEATURES: physical crater "off by default" (params::defaults true); colour grade "no bridge" (render_workspace rows); deep-zoom "renderBiomeTileRGBA unported" (exists); missing depression-filled routing entry; render.rs::build_ao "reference has no AO" false (v2.10 buildAOField 7994).
E3. RC_ENGINE_CHANGES §4 "port found same gap at lod_bridge.rs:420" (LOD-D2 removed; render_biome_tile_rgba bare a.exag -> v2.25 tileShadeExag unported untracked); §6m intro/.3/.4 halves belong in §8.1; §8.1 "Nine of eighteen" lists ten; STATUS Orientation "Seven of the interval's changes" lists eight.
E4. GENERATION_PARAMETERS: tect.resist filed under Tectonics (read by stream power only); passes.diffuse_d "unconditionally" (only physical); use_gpu names Volcanism; civ.seed_thresh cite 14568 wrong; "85 parameters" (OUTSTANDING §2.5 repeats); "main.gd drives" (engine_bridge.gd); "Every manual-pass knob has a reference slider" false (glacial_mg, sediment_capacity, tidal_k); "— means default reproduces reference" false (4 ruled flags, crater.surface_age_myr, passes.evolve_cycles 0 vs 5).
E5. EXPORT_SCOPE §5 "owned parts reached but not built" (ExportSnapshot 34db87f); LOD_DETAIL_SCOPE q5 ruling not recorded in LOD_TILING_INTEGRATION clipmaps bullet; GPU_LAYER_INTEGRATION m3 "2.13x ... 1.15x" withdrawn 09-05, names nonexistent gpu_compute_height; ASSET_LIBRARY_SCOPE §3 composite_trait_badges live path (pack.rs: no caller).
E6. Stored LOD pyramids untagged colour space: lod_bridge tile_producer_id doc "storage unwired"; now wired (include_lod_tiles); LodSnapshot::render_tile applies session color_space -> P3 tiles stored untagged. No row.

### F. Architecture/README
F1. ARCHITECTURE crate tree lists 10 of 16 (missing spatial, gpu, civ, urban, assets, vault); docs/ line "CHANGELOG.md, HANDOFF.md" (no HANDOFF); assets deps claim io/noise (is jsmath+noise); "later subsystems depend on cartalith-engine" only civ; undocumented dev-dep cycle (gpu dev-deps engine/civ), io depends on spatial; "Threading"/"Not decided yet" stale; MVP_SCOPE §7 points there.
F2. cartalith-jsmath Cargo.toml comment + crate doc repeat assets->io.
F3. README reference/ row "both v2.10".
F4. JS_SEMANTICS_AUDIT cites -urban::site::js_or (geom.rs).

### G. False code comments (grouped)
G1 unwired-but-live: engine lib.rs pub mod docs staleness/sculpt_commit/region_export/geojson "Unwired"; render.rs module doc 127, 866, 3495, 1957; lib.rs atlas_export_zip/import_zip "No menu row", atlas_is_covered "No shell caller"; climate deflect_flow/apply_climate_moisture_correctors/build_wind; noise gpu_pfbm; placement.rs trait_badge_layout; project.rs write_project; region_export UI hold; label_bridge UI hold; lod_bridge module doc.
G2 wrong defaults: CraterParams::physical_model doc; terrain_wind_deflection/currents "false" (crate doc 46/55, ClimateInputParams 311/316, inline 699; climate WeatherParams, simulate_weather, golden_parity_weather header); params.rs defaults() orphan paragraph; get_param_defaults/reset_params say WorldParams::defaults; lib.rs cites main.gd x5 (4679, 5372, 5396, 6343, 6383).
G3 misplaced docs: refresh_climate doc above climate_params_for; with_lithology doc on with_lakes; snow_aspect_shift doc on tile_snow_facing.
G4 citations: 9 Rust comments cite OUTSTANDING §2.9 for world-wrap row (is §2.6); ref lines: classify_boundary 2818->2825, ClimateInputParams 2280->2287, import computeFlow 6797->6790, pyramid chunkParent/Children 10933/10934->10936/10937; GpuDevice doc cites nonexistent measured_pipeline_build_cost.
G5 GPU prose: cartalith-gpu crate header + Cargo description "one kernel vnoise", "never authoritative"; gpu_stages_used docs list 4-5 of 8; gauss_blur/erode_thermal "CPU only"; REUSED_STAGE_MAX_STORAGE_BUFFERS counts; "Performance window" (deleted ruling 19).
G6 dead: import.rs _base_field blur; AtlasStore::chunk_len; cartalith-gpu test-only pub fns (init_gpu_shared_device, init_gpu, per-stage init_gpu_*, self_test, vnoise_grid, gpu_*_grid_cpu twins, brute_force_nearest_plate, warp_band_gpu_with) — Ruling 22 rule.

### H. Untracked gaps
1 GPU plates wrap; 2 orogeny knobs; 3 v2.57 + unported interval rows incl tileShadeExag; 4 biome colour GUI + legend/bclass/paint preview ignore override; 5 atlas no reader; 6 snow_aspect_c tunable/GUI; 7 LOD pyramid colour space; 8 Fail-with-error refuses with GPU off; 9 build_water_bodies 4 call sites cost; 10 corner-only parcel water test §6s.4 in blocks.rs::build_parcels ("The full footprint" comment); 11 two-depression-model + lake flow gate with integrate_drainage (§6m.3/.7); 12 region export omits params.json (caller passes None).

### I. Owner questions (unruled)
orogeny knobs derivation; sea_grain_warp default flip; LOD questions (LOD_DETAIL q1,4,5,6; D5 octave decay; D1 40ms budget; sawtooth tolerance); v2.25 tileShadeExag port/decline; §7p vs GPU default/CPU toggle interpretation; Ruling AH phone New World offers 4K/8K.

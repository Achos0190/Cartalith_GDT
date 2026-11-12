# OUTSTANDING_WORK.md audit, first half (top through the end of section 2.6), 2026-11-12

A dated cut, findings only. Verdicts were produced by reading and grepping the code at `d3873d70` (origin/main when the audit began) and `git log -S`; nothing was built or run, and no code was changed. Row identifiers are line numbers in `OUTSTANDING_WORK.md` at that commit (they drift; use the title). UNVERIFIED means the open half could not be settled from code. The second half (2.7 to the end) is `OUTSTANDING_AUDIT_2026-11-12_B.md`.

Verdict counts by the VERDICT lines below (59 judged blocks plus the parked HTML group): CLOSED-ALREADY-OK 17, DONE 2 (IN-13 caravans; the Close button floor), PARTLY 29, OPEN 25, OPEN-owner/device 5, OPEN-parked 2, STALE 5 (their text corrected). Several DONE-like findings sit inside PARTLY rows, whose stale clauses were fixed.


Edits applied: section 1 UNWIRED row (3D viewport no longer parked); 2.3 export head, IN-13 caravans, Landmark M9, router and zero-length rows struck, seed 707; 2.4 phone toolbar; 2.5 style-preset next-step, diamond-lattice, overlay-export row rewritten, EF-9 caveat, asset pack "Nothing is built"; top block consent, probe, MORE, Close button, welcome patch, pen prose, gate, hygiene, manual items.

---- block A ----

Worktree HEAD d3873d70. Read-only. Line numbers are those of OUTSTANDING_WORK.md in cg_wt_A.

### 3018 The `UNWIRED_FUNCTIONS.md` backlog - 2 open rows (0 small, 1 medium...
VERDICT: STALE (partly; the counts are right, the 3D-viewport clause is not)
EVIDENCE: UNWIRED_FUNCTIONS.md section 2 still lists exactly 2 open rows (3D viewport, bounded sculpt preview) - matches. label_glyph_layout closure: label_bridge.rs::shell_label_box exists (+ test shell_label_box_centre_has_no_added_half_cell); _labelboxmodel_probe.gd/.tscn, _paintpatch_probe.gd, _clipboard_probe.gd all exist. statusMid: app.gd::_refresh_status_mid, _repaint_open exist. SL-1 (bounded sculpt preview) still "not started" in STATUS.md and build_ao still has no window parameter - correct. BUT: "the 3D viewport stays open by ruling / parked / only the deferred-by-ruling row remains" is superseded: Ruling CS approved it (DECISIONS 4a), 3D-0 measured GO and 3D-1 built (shell/view3d.gd holds a SubViewport with Camera3D; shell/view3d_gate.gd; commit c0ca18a9); 3D-2..3D-8 not built (STATUS.md line ~1782). The `_todo` "3D viewport defaults" in menus.gd is still a _todo.
LEFT: nothing new for this register; the 3D track is now owned by 3D_VIEWPORT_SCOPE.md / STATUS.md, not "parked".
FIX: replace "The 3D viewport stays open by ruling (deferred, research first) - correctly parked" with "The 3D viewport is no longer parked: Ruling CS approved it, 3D-0/3D-1 are built (c0ca18a9), 3D-2..3D-8 are `3D_VIEWPORT_SCOPE.md`'s (STATUS.md); only the `_todo` 3D-defaults menu rows remain from this register." Also UNWIRED_FUNCTIONS.md section 2 row 1 ("no Camera3D or MeshInstance3D in shell/") is false now (view3d.gd has a Camera3D) - fix the same way.

### 3019 Landmark M8 residual - 18 of 49 declared kinds still ship buildable:false
VERDICT: OPEN (count verified current)
EVIDENCE: landmark.rs::kinds(): counted 49 `LandmarkKindSpec { key:` entries, 31 `buildable: true`, 18 `buildable: false` - matches the row exactly. Test "the thirty-one kinds this engine actually generates" present near line 6982 and funnel_arithmetic_closes_on_every_kind.
LEFT: the 18 refused kinds (glacial_feature, ruin, historic_crossing, shrine, battlefield_historic, destroyed_fortress and the others needing deposition/forest age/coastal salt/civ traits/events). Nothing built beyond what the row says.
FIX: none needed. Optionally trim the long "Earlier text follows" tail (historic 29/25/24/23 counts) since the lead already says 18.

### 3208 Territory lasso - assign an area to a faction by drawing a polygon (struck, CLOSED)
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: civ_tools_bridge.rs::polygon_cell_mask, CivTools::paint_polygon, 9 `fn lasso_*` tests; lib.rs::civ_territory_paint_polygon; civilization_workspace.gd registers "territory_lasso" tool + handlers; _terrlasso_probe.gd/.tscn exist. Introduced 70437e91. The "province lasso" follow-up it mentions was not re-checked (outside this row).
LEFT: -
FIX: none.

### 3209 Un-shelve the 16K/32K export (ruling 15) - THE LADDER SHIPS; E1, E2, E3...
VERDICT: PARTLY
EVIDENCE: export_stream.rs::write_jpeg (JPEG built), export_raster.rs::export_session_begin, shell/export_overlay.gd::run, shell/export_dialog.gd, _exportoverlay_probe.gd all exist. STATUS.md: EXP-E3 done, EXP-E4 "built, batches A-D", EXP-E5 "partial - first slice 2026-10-13; JPEG built 2026-10-18"; row 1789 lists the dialog opening from File > Export. The row's own tail already agrees. Stale head of row: "Next step (2026-10-10): build E4 batches B, C, D ... then E5 dialog" and "STATUS.md has EXP-E4 partial and EXP-E5 not started" - both superseded by its own later text.
LEFT (from the row's own tail, UNVERIFIED in code beyond presence of the symbols): the JPEG/PNG size model under-estimates small grids ~2.5x; the landmark glyph raster cap; Android tile/frame measurement (device); an owner look at EXPORT_URBAN_MIN_BOX_PX (75 px; constant lives in map_overlay.gd). The "style/look control" and "file-exists confirmation" are struck as built.
FIX: replace the "**Next step (2026-10-10):** build E4 batches B..." sentence with "**Next step:** the E5 residuals listed in the last paragraph (size-model under-estimate on small grids, glyph raster cap, Android measurement - owner/device - and the EXPORT_URBAN_MIN_BOX_PX look)." and drop "STATUS.md has EXP-E4 partial and EXP-E5 not started".

### 3210 CV-23 historical territorial occupation over time (struck, CLOSED)
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: project_bridge.rs::a_recorded_year_comes_back_holding_what_was_true_then_not_what_is_live_now exists; history/territory/<year>.i32 slot referenced in project_bridge.rs; the row's one open note (archive still writes one whole raster per year) is still true (comment near "The archive shape did not move"). 
LEFT: -
FIX: none.

### 3211 The wantCounts / user-fixed-tier-count branch of _civIterativeAutoWorld (struck, CLOSED)
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: cartalith-civ lib.rs::place_settlements_with_counts, civ_quota_tier; engine lib.rs CivParams::want_counts; tests/golden_parity_want_counts.rs, tests/iterative_network.rs::fixed_counts_hold_on_real_worlds and want_counts_seed_params_are_the_references; new_world_dialog.gd::_build_fixed_counts; _wantcounts_probe.gd/.tscn.
LEFT: -
FIX: none.

### 3212 IN-13 - trade flows (struck, CLOSED-MOSTLY, caravans deferred)
VERDICT: DONE (the "caravans deferred / small remaining / own open row directly below" part is STALE)
EVIDENCE: trade.rs::trade_flows, Tariff, sea_caravans (+ tests sea_caravans_are_one_per_port_pair..., sea_caravans_carry_exactly_the_matched_sea_volume per STATUS); civ_trade_bridge.rs::civ_set_trade_tariff; faction_roster_window.gd calls bridge.civ_set_trade_tariff (tariff control, _build_tariffs). STATUS.md EC-10: "done - all 4 pieces built": land/river caravans da51a57 (2026-09-24), sea caravans 2026-10-13 (Ruling AQ), tariff control 2026-09-27. No caravans row exists in OUTSTANDING_WORK.md any more (the "directly below" row is the Landmark M9 row).
LEFT: -
FIX: change the status head to "**CLOSED 2026-09-23 (verified); caravans built `da51a57` 2026-09-24 and sea caravans 2026-10-13 (STATUS.md EC-10); tariff control built 2026-09-27**", remove "small remaining" in the size column ("-"), and delete the "Caravans are now their own open row, directly below" italic and the "Tariffs have no GUI yet" / "Caravans deferred" sentences.

### 3213 Landmark M9 - cultural interpretation and temporal state - partial
VERDICT: PARTLY
EVIDENCE: Battlefield wiring built (775df17, battlefield buildable). Row says "none of (a)-(d) built": now (a)-part and (d) ARE built: user-given landmark names - place_name.rs::landmark_names takes `user_names`, WorldGen::landmark_user_names persisted via project_bridge.rs LandmarksDoc `user_names` (df36f8d1 "VE-9 finished: landmark names and links (Ruling EI)", 54e3aace); shell entry point for notes: shell/context_note.gd calls bridge.vault_landmark_entity_id (the row says it has no shell caller - stale). Still unbuilt: kinds() shows shrine, battlefield_historic, destroyed_fortress `buildable: false`; no discovered/named/monumentalized state fields.
LEFT: discovered/monumentalized states (research s25), the present-year read for battlefield_historic, the civilisation-traits input (s26) for shrine and the other Cultural kinds, destroyed_fortress (needs a destruction to read - design question). Also STATUS.md LM-9 still says user-name and shell link are unbuilt: stale too.
FIX: in this row replace "What is buildable now, none of it built" with "(a) user-given names and (d) the vault link from the CTX_LANDMARK context are BUILT (df36f8d1, 54e3aace; Ruling EI); still open: discovered/monumentalized state, battlefield_historic's present-year read, the s26 civilisation-traits input, destroyed_fortress". Update STATUS.md LM-9 the same way.

### 3214 Story planning SP-2 residual: the journey's departure date
VERDICT: OPEN (small)
EVIDENCE: project_bridge.rs JourneyDto has only `start_year` (no day field); story_bridge.rs module doc still says "A departure is 1 January of Journey::start_year ... widening entities/journeys.json to a full departure date is a [format change]". Everything else in the row (JourneyTimeline, chronos calendar, JourneyPlanCache) is built and not re-opened here.
LEFT: a departure month/day on the persisted journey (optional additive field in entities/journeys.json, SAVEFILE_COMPAT 9.6) and its editor.
FIX: none needed.

### 3215 The router's smoothed points can leave the map: civ_dijkstra_path (FIXED)
VERDICT: CLOSED-ALREADY-OK (row already says FIXED; the tail "Needs its own test and fix" is the original find text)
EVIDENCE: tests/route_km_on_map.rs exists in cartalith-civ.
LEFT: -
FIX: strike the row (it is fixed, per its own head), like its neighbours.

### 3216 A zero-length route plans as 0 days with a 0 km stage and no flag (FIXED)
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: cartalith-civ lib.rs::jp_route_is_zero_length exists (line ~16516); journey_planner_view.gd present.
LEFT: -
FIX: strike the row.

### 3217 Seed 707: a committed sea route came out about 0.1 day long (R8)
VERDICT: OPEN (not reproduced; UNVERIFIED in-shell)
EVIDENCE: STATUS.md 2450: "R8 not reproduced"; journey_planner_view.gd reads route_get but no print of `route_get(i).points` was added (grep shows only comments/guards). Probably a symptom of the zero-length case, now guarded by jp_route_is_zero_length (876ae9c7), so a repeat would now show the refusal/dashed card.
LEFT: run the seed-707 planner probe in the shell and look at the refusal; close if the card shows or it does not repeat.
FIX: append "After jp_route_is_zero_length (the row above) a recurrence would show the dashed refusal card; re-run seed 707 in the shell once and close if clean."

### 3218 Military manpower: industrial rows, high-state-capacity standing army above its band (Ruling DL Q2)
VERDICT: OPEN (measure-first only; owner-gated build)
EVIDENCE: g1_baseline.rs::ea0_ecological_ordering_diagnostic with "EA industrial unordered" output; manpower.rs::the_ordering_holds_across_the_driver_space_but_one_class; ECOLOGICAL_ARMY_TERM_SCOPE.md 10.7 names the 3 worlds (12345/40075/24 f7, 314159/800/24 f15, 707/800/24 f23). git log of manpower.rs/g1_baseline.rs ends at 9a83061c (Ruling DL); nothing since.
LEFT: the measure-first pass distinguishing bracket-5 upkeep vs extraction ceiling, and a ruling before any build.
FIX: none.

### 3219 Military manpower: Ruling DL's open questions (Q7, Q8)
VERDICT: OPEN (Q7 = nothing to do by ruling; Q8 = measure-first)
EVIDENCE: manpower.rs ERA_BANDS has `Early industrial` (comment ~line 785 notes it was not restated); no bracket-3 levy term in git log since 9a83061c.
LEFT: Q8 measurement of a bracket-3 levy term (incl. Kingdom B); Q7 only if the owner reopens it.
FIX: none.

### 3220 A saved journey's per-leg overrides can land on a different leg: save a fingerprint
VERDICT: OPEN
EVIDENCE: project_bridge.rs JourneyPlan DTO has `stage_overrides: BTreeMap<String, BTreeMap<String, Value>>` keyed by stage index, no fingerprint field; grep for leg fingerprint/override fingerprint finds nothing in project_bridge.rs, story_bridge.rs, journey_planner_view.gd (only unrelated wildlife fingerprint). SAVEFILE_COMPAT 9.6 not re-opened for the field.
LEFT: the optional additive per-override leg fingerprint, drop-with-notice on load, SAVEFILE_COMPAT 9.6 note, tests.
FIX: none.

### 3226 Project archive remainder - the library/ art binary payload (BUILDABLE since Ruling AP)
VERDICT: OPEN
EVIDENCE: SAVEFILE_COMPAT.md 16.5 still titled "the binary half, still not stored" and 12's table says the item images are not in assets.json; cartalith-io/src/project.rs and project_bridge.rs register only library/assets.json, travel.json, settlement_types.json (grep for any per-image library entry finds none); STATUS.md has no embed milestone.
LEFT: specify entry layout under library/ in 16.5, version/fail-loud rule, write in cartalith-io, restore in project_bridge.rs, round-trip test plus old-archive-loads-unchanged test.
FIX: none.

### 3227 Record the owner-approved Vault Browser mockup (struck, CLOSED 2026-09-27)
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: shell/vault_window.gd and _vaultlayout_probe.gd/.tscn exist. (design/vault-browser-2026-09-21/ not listed here; not checked.)
LEFT: -
FIX: none.

### 3228 Vault editor, on device: check the phone toolbar against the on-screen keyboard
VERDICT: PARTLY (build done; device check OPEN-owner/device)
EVIDENCE: markdown_editor.gd now docks the toolbar as a keyboard accessory (`EditorKeyboardDock`, `keyboard_height` hook, "Done"), built in VE-8 (d91f9a5e, STATUS.md "VE-8 built"). STATUS.md VE-8 says "The editor and its row were not seen on glass" on the 6T; bar "row stays above keyboard on the 6T at both orientations" is simulated only. The row's "toolbar sits above the text area, not docked" text is stale.
LEFT: only the on-handset confirmation (owner/device).
FIX: replace the row body with "Docking built in VE-8 (`EditorKeyboardDock`, d91f9a5e); left: confirm on the 6T at both orientations that the row rides above the real keyboard (STATUS.md VE-8 says it was not seen on glass)."

---- block B ----

### 3289 Vertex-mesh 3D viewport (Ruling CS) ...3D-2 next
VERDICT: PARTLY
EVIDENCE: shell/view3d.gd (VERTICAL_EXAGGERATION_DEFAULT=1.0; _info text "Terrain is not drawn in 3D yet"), shell/view3d_gate.gd (make_patch/ArrayMesh only inside the gate probe). Commits c0ca18a9 (3D-1), 791ad1d1 (3D-0). STATUS.md "3D viewport" row: 3D-2..3D-8 not built. No displaced terrain mesh, drape or orbit in view3d.gd (grep ArrayMesh/MeshInstance3D/orbit: none outside gate).
LEFT: 3D-2..3D-8; 3D-0(c) 6T run (owner device); gate thresholds uncalibrated (3D-7).
FIX: none needed; row is accurate. Optional: strike the "Earlier text of this row" tail.

### 3290 3D view: a live height-exaggeration slider
VERDICT: OPEN
EVIDENCE: no slider in view3d.gd; menus.gd ~4597 text still says the 3D view "draws no terrain yet" and the 3D defaults rows are _todo; dcc_settings.gd has relief_exaggeration_default() (2D's). Blocked on 3D-2 as the row says.
LEFT: whole slider; depends on 3D-2 terrain.
FIX: none.

### 3291 Colour paint tool (Ruling CN) ... CP-4..CP-7
VERDICT: PARTLY
EVIDENCE: cartalith_spatial::colour_paint PaintLayers, MAX_LAYERS=32; shell/colour_paint_tool.gd (heat_along default 0.0, "Cool along stroke" slider). STATUS.md line "Colour paint": CP-4..CP-7 not built. grep for Scale-with-the-map / rescale in colour_paint_tool.gd and colour_paint.rs: none (Q5 notice not built). No fill-helper code found in colour_paint_tool.gd (grep "fill" only hits an unrelated array fill).
LEFT: map-width Scale-with-the-map notice, CP-4 phone reachability (device), CP-5 fill helper, CP-6 export-without-paint, local-contrast halo measurement.
FIX: none; accurate.

### 3293 Roads, sea lanes, borders and labels now follow the style preset ...
VERDICT: PARTLY
EVIDENCE: map_overlay.gd set_style_treatment(), called from shell/workspaces/render_workspace.gd and viewport_host.gd; STYLE_PRESETS in render_workspace.gd. Header states everything built except thin-glyph label weight (owner call) and territory wash (baked in Rust, deliberate). Body "Next step" text is original history. UNVERIFIED: per-preset colours of icons/pins/journey labels were not each opened.
LEFT: owner call on thin-glyph label weight (Woodcut/Ink/Vintage); territory wash intentionally unstyled.
FIX: replace the stale "**Next step:** each style preset carries a river treatment..." sentence with "(built; see header)" or strike it, so the row's open part is only the label-weight owner call.

### 3294 ~~Zoom-sensitive rivers~~ superseded by RIM-5
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: struck, re-homed to RIVERS_IN_MAP_SCOPE.md RIM-5; RIVER_ZOOM_RESEARCH.md was never needed. Not deeply checked.
LEFT: cartographic-generalisation research owed under RIM-5 (belongs to that row).
FIX: none.

### 3295 ~~Tanaka illuminated contours~~
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: Npr.tanaka in cartalith-godot/src/render.rs; tests/tanaka_contours.rs (tanaka_without_contour_veins_draws_nothing, strength_blends_between_plain_and_tanaka); commit 03e94824.
LEFT: deferred width-by-facing variant only (disclosed).
FIX: none.

### 3296 Raisz physiographic pictograms
VERDICT: OPEN
EVIDENCE: grep "raisz|physiograph" across crates and godot-project .rs/.gd: no implementation (only unrelated word hits in shortcut/llm files). Mentioned only in docs (STATUS.md ~5925 "scheduled, not built").
LEFT: scope, landform classifier, symbol stamper.
FIX: none.

### 3297 Mappa-mundi figurative icons
VERDICT: OPEN
EVIDENCE: no mappa-mundi code found by grep in crates/godot-project.
LEFT: scope + build.
FIX: none.

### 3298 Ordnance Survey-style symbology
VERDICT: OPEN
EVIDENCE: no OS-symbology code found by grep.
LEFT: scope + build.
FIX: none.

### 3299 ~~LOD-tile zoom sawtooth~~ CLOSED 2026-09-27
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: lod_sweep.rs::seam_ratio exists. Note: the body cites export_raster.rs::screen_river_ink, which now appears only as a doc-comment mention (no fn found), consistent with its being historical. Body text is the superseded investigation.
LEFT: none.
FIX: none.

### 3300 ~~LOD-D3 continuous transitions~~
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: lod_bridge.rs::morph_for_zoom exists (commit b6cc014 cited, not re-checked).
LEFT: disclosed unmet bars live elsewhere.
FIX: none.

### 3301 ~~LOD-D6 tile synthesis off the main thread~~
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: lod_worker.rs::LodSnapshot, budget_for_tier exist (commit c74a150 cited).
LEFT: phone bars device-only (stated in row).
FIX: none.

### 3302 EF-9 Importance-driven refinement
VERDICT: PARTLY
EVIDENCE: cartalith-godot/src/ef9_bridge.rs (uses subdivision::select_tiles, conform_plan), cartalith-engine/src/importance.rs, subdivision.rs, elevation.rs::world_elevation_tile_conformed; lod_worker.rs::budget_for_tier: Performance ef9_refine false, Balanced/Quality true (Ruling CL / EA item 16 as stated). Commits 0c768d17, 273bc220, 465841aa. Open items checked: item (b3) still open because select_tiles and world_elevation_tile_conformed are still used by ef9_bridge.rs (not removed); no incremental importance refresh function found (EF-9.5 open).
LEFT: (b2) z+1 layer vs parked-tile cache decision; (b3) remove unused discrete-model code; (c) EF-9.5 incremental importance refresh; carve_rivers=false river-gain measurement. Note (b3) wording says "unused" but ef9_bridge.rs still calls select_tiles: re-confirm it is truly unused before deleting.
FIX: append "(b3: as of this audit ef9_bridge.rs still imports select_tiles/conform_plan; confirm which are dead before removal)".

### 3303 Snow gets an aspect term on the MAIN map
VERDICT: PARTLY
EVIDENCE: render.rs TerrainAppearance::snow_aspect_c (2.0 shipped, 0.0 in js_reference), snow_aspect_shift (knee still hard-coded slope/0.04), tile_snow_facing; commit 19c38d96; tests/lod_d4_ice_and_snow.rs measure_the_aletsch_sheet (#[ignore]). No later commit changed the knee (git log -S'fn snow_aspect_shift' = 19c38d96 only). STATUS.md LOD-D4 row agrees: bar 1b not met, 1 of 4.
LEFT: scale-aware slope knee, same-pixels on/off effect measure, GUI control for snow_aspect_c.
FIX: none.

---- block C ----

Worktree HEAD d3873d70. Read-only; no builds run.

### 3304 diamond-lattice pattern in deep-zoom tiles / dark diagonal hatching
VERDICT: PARTLY
EVIDENCE: render.rs TerrainAppearance::relief_lowpass exists (used by tests cel_toon.rs, color_space.rs, hatch_shade_diag.rs, layer_stack.rs); commit 8c0eb5b2 (Ruling BZ item 2); STATUS.md "Base-texture hatching: candidate (b) ... built, verified; the hatch is visibly gone". sea_grain_warp: 1.0 since 51df4d21 (git log -S).
LEFT: only the owner's re-check of the deep-zoom lattice on a real world, and candidate (c) "only if residue remains" (OPEN-owner). No code is open.
FIX: Retitle from "partly diagnosed ... cause not established" to "candidate (b) relief_lowpass BUILT 8c0eb5b2 (hatch gone); open: owner re-check of the deep-zoom lattice; (c) only if residue". Strike or compress the "Next step" and "Superseded next step" paragraphs.

### 3305 Ruling 17 unbuilt / sea stamps re-read sea level live
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: sea_live_bridge.rs::set_sea_level_live; cartalith-spatial/src/pass.rs::rewrite_stamps (test rewrite_stamps_reaches_live_history_and_redo_without_a_history_step); commit 49cc10b0.
LEFT: none. FIX: none.

### 3306 Sea level is not a live control
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: same as 3305 (set_sea_level_live, 49cc10b0).
LEFT: none. FIX: none.

### 3307 PhotoCraft-inspired editing tools (A-F)
VERDICT: PARTLY
EVIDENCE: adjust.rs::hsl_oklch exists (ET-F1 chroma fix); export_raster.rs test screen_equals_export_at_grid_resolution_under_every_stack exists; print_gamut.rs (ET-E3) has from_icc and a header note "Not built here: loading the owner's own press .icc"; STATUS.md "Four shell backlog rows closed (2026-10-12): a loaded look restores its map" (load_appearance_preset now adopts the file's npr); STATUS 3D-0 note (a): the 2026-09-01 device loss does not reproduce on Godot 4.7.2.
LEFT: (1) loader for a user-supplied press .icc (owner question); (2) HDR display/export (deferred by ruling); (3) real pen/touch hardware checks for D2-D5 and C4 on glass (device/owner); sub-rows 3310-3314 carry the rest.
FIX: Replace "Newly found and not fixed: loading a saved look never restores the map texture it was saved from; the gallery tile and inks keep the session's state when a project opens" with "CLOSED 2026-10-12 (STATUS.md 'Four shell backlog rows closed')". Replace "the Forward+ device-loss investigation NOT done" with "isolation control run 2026-10-19 on Godot 4.7.2: device loss does not reproduce (3 of 3 clean with use_gpu on); the 6T was not run".

### 3308 Lake filling
VERDICT: PARTLY
EVIDENCE: cartalith-civ/src/lib.rs::apply_lake_area_floor and lake_area_floor_cells (test lake_area_floor_cells_scales_with_the_grid_at_literal_values), commit 6f36b146; WaterBodies::basin / basin_outlet data only.
LEFT: item (2) per-basin water balance PARKED by Ruling CC; items (4) 4-vs-8 connectivity measurement and (5) selective breach unscheduled; item (3)'s data has no consumer; on-screen result of the floor not looked at (UNVERIFIED).
FIX: none required; the row is accurate (the "Owner call needed" lead is already marked answered).

### 3309 Export of overlays: no shipped export path draws MapOverlay
VERDICT: STALE
EVIDENCE: shell/engine_bridge.gd::export_session_begin and other export_session_* forwards exist ("Thin, gated forwards ... for ExportOverlay.run"); shell/export_overlay.gd header: "a copy of the live map_overlay.gd laid out at the export's size" (E4), incl. borders (Ruling AZ); map_overlay.gd has _sync_overlay_adjust / set_overlay_adjust (shared static OverlayAdjust stack, so a copy resolves the same ink). STATUS.md row "Export: E5 dialog, JPEG, banded fix".
LEFT: UNVERIFIED whether the export copy applies the labels/overlays adjustment stack in output pixels (needs a windowed export probe; no test asserting it found). The row's claims "no GDScript caller" and "no overlay draw path" are false.
FIX: Replace the row text with: "Overlay drawing for exports is BUILT (ExportOverlay / export_overlay.gd, EXPORT_SCOPE E4). Only open: confirm by a windowed export probe that ET-C3's overlay adjustment stack reaches the exported overlay pixels (the copy shares OverlayAdjust state)."

### 3310 ET-C2 stack edit rebuilds river colour field / TileFields
VERDICT: PARTLY
EVIDENCE: screen-equals-export half already struck as done: _exportraster_probe.gd section "13b" exists; Rust test screen_equals_export_at_grid_resolution_under_every_stack exists; STATUS 2026-10-13 entry.
LEFT: the stack-edit rebuild timing (min..max, alone) and the river-bank seam within 1 LSB windowed probe: no evidence either was done (UNVERIFIED).
FIX: none.

### 3311 Curves look-up table for the adjustment stack
VERDICT: PARTLY
EVIDENCE: adjust.rs Curves (Fritsch-Carlson); measured 2026-10-12, no LUT added (STATUS).
LEFT: only the end-to-end live-drag timing in Godot (device/windowed, OPEN-owner/device); no LUT unless that says so.
FIX: none.

### 3312 ET-D3 / ET-D2 pen gaps
VERDICT: OPEN
EVIDENCE: world_workspace.gd::_on_map_stroke_sample takes pressure / is_pen; paint_bridge.rs has undo-group logic in Rust (stroke = one undo step); no shell-facing undo #[func] seen (grep "undo" in paint_bridge.rs shows doc comments only). Needs pen hardware.
LEFT: OPEN-owner/device (real pen). FIX: none.

### 3313 ET-D4 200-waypoint commit time never measured
VERDICT: OPEN
EVIDENCE: cartalith-civ/src/tools.rs::civ_dijkstra_path_guided exists; no measurement found in STATUS (grep "200.waypoint" / "drag commit" gave nothing).
LEFT: measure alone, median with min..max, on a real 2048 world; redo_labels undercount unchanged. FIX: none.

### 3314 Way tool 'Advanced': Waypoints slider added unasked
VERDICT: OPEN
EVIDENCE: tool_bar.gd waypoint constants and infrastructure_workspace.gd::_way_drag_options exist.
LEFT: owner decision (keep slider or fix spacing). OPEN-owner. FIX: none.

### 3315 Render target gap list
VERDICT: PARTLY
EVIDENCE: render.rs canopy_coarse_km, land_grain_warp, KmZoomDetail, satellite_relief; STATUS rows "Forest canopy in Natural Vibrant and Antique Parchment", "Satellite relief look", Ruling CI/CU/CW entries.
LEFT: the row's own remaining items: fit-scale relief / mountain relief (now section 2.13), forest share (world climate), Satellite relief edge energy, thin snow sliver and terrace banding at x4-x16, region names not hideable by a preset, x64 sub-cell share and sea under zoom. Not re-measured (UNVERIFIED); several are look judgements.
FIX: delete the "Blocker" sentence (already marked stale by the row itself). Otherwise accurate.

### 3316 Asset pack refinement tools
VERDICT: PARTLY
EVIDENCE: STATUS.md AP-5 (pack::place_feature_icons, composite_placed_features), AP-8/9 (cartalith-assets::folder_import), AP-10 (cartalith-assets::lineart), all marked built; shell/feature_icon.gdshader exists.
LEFT: ground candidates matching no slot have no move-to-slot verb; .wonderdraft_symbols unparsed; Android folder import untested on glass (device); painted sprites miss AP-5(b) 2 px (faithful per Ruling CS); AP-5(c) 1% not met; pack_report / near_dupes / lineart::working_image bake straight; line-art variants carry no flag; pencil scan misses IoU by 0.001; owner scans untested; pack_compositing.rs "forces shrub" glyph claim found false, not fixed (not re-checked).
FIX: delete the closing sentence "Nothing is built." (contradicts the row's own AP-1...AP-10 built text and STATUS); the "next AP-5 (Q3, measure first), AP-8, AP-9" sentence is historical and stale.

### 3332 Erosion per-cell parts on GPU (thermal landed; stream-power open)
VERDICT: PARTLY
EVIDENCE: crates/cartalith-gpu/shaders/gpu_thermal.wgsl exists; no stream-power shader in shaders/; cartalith-erosion::stream_power_kernel_bounded is not factored by SP-G0 (stream_power_routing is GF-10's, commit 692fc8d5); STATUS GLI-E row: "SP-G0...SP-G6 none started". Thermal commit 08020ee.
LEFT: SP-G0 (phase factoring, per-phase measurement, go/no-go) and everything after.
FIX: none.

### 3333 Water-body priority-flood on GPU
VERDICT: OPEN
EVIDENCE: no water-body / flood shader in cartalith-gpu/shaders (list read).
LEFT: all of it. FIX: none.

### 3334 Performance telemetry system (parked, Ruling AZ)
VERDICT: OPEN
EVIDENCE: no telemetry code found (STATUS grep hit is unrelated).
LEFT: parked by ruling. FIX: none.

### 3335 Tiled / chunked GPU compute (section 18)
VERDICT: OPEN
EVIDENCE: cartalith-gpu/src/multi.rs exists; band-split-covers-one-kernel claim not re-measured (UNVERIFIED).
LEFT: large, parked. FIX: none.

### 3336 cartalith-gpu real-device throughput tests deferred (page file)
VERDICT: OPEN
EVIDENCE: tests exist: cartalith-gpu/tests/multi_gpu.rs::per_device_warp_throughput_measured and ::split_tiles_across_two_real_devices_measured; STATUS "Live-tree test run, crate by crate" (2026-10-11) records the guard kills.
LEFT: OPEN-owner (enlarge page file); cannot be settled from code. FIX: none.

---- block D ----

## Verifier D: OUTSTANDING_WORK.md "Open after 2026-10-26" (cg_wt_A lines 48-283)
Line numbers are cg_wt_A's file (it holds more bullets than the main checkout's copy). Read-only; no builds or probes run. UNVERIFIED = could not settle from code.

## Assistant and consent

### L55 A Cartalith-specific small LLM, trained on the owner's own hardware (2026-11-11)
VERDICT: OPEN
EVIDENCE: LARGE_ITEM_RULINGS.md "## Ruling EP" (deferred, Qwen3.5-4B); SMALL_LLM_ADDON_RESEARCH.md "# Part B" (~L160) holds the first deliverable. No SL code exists.
LEFT: Deferred by Ruling EP until the rest of Cartalith is done; owner answers Q-SL-1..10 decide scheduling.
FIX: none.

### L71 Consent UX (TOP PRIORITY)
VERDICT: PARTLY
EVIDENCE: llm_assistant_window.gd (`LlmConsent.present(PopoutHost.dialog_host(app, self), ...)` ~L1630; "Turn on Assistant enabled" button ~L1203), llm_settings_window.gd (`LlmConsent.present` ~L1041); STATUS.md ~L1716 (probes `_llmmodels_probe` leg 1b, `_llmpanel_probe` leg 5b); commit beee4808.
LEFT: Real LM Studio, a real remote host and a device run only. The code is built.
FIX: Retitle "Consent UX (built; real-server and device check left)", drop "(TOP PRIORITY)". STATUS.md L36-37 still says the consent fix "is still not" built and contradicts its own L1716 entry: fix there too.

### L74 Undo-and-regenerate offer after an Apply and regenerate (Ruling DS follow-up)
VERDICT: OPEN
EVIDENCE: llm_assistant_window.gd `_regenerate_after_apply` / `_on_regen_finished` only regenerate and, on failure, mention Edit > Undo in a status hint; no undo-and-regenerate offer/button anywhere in shell/. LLM_ASSISTANT_SCOPE.md s.5b.4 item 3 describes only the "undo restores settings, not a world" note.
LEFT: A one-click "Undo and regenerate" offer on the applied card.
FIX: none.

### L75 LLM-7 remainder
VERDICT: PARTLY
EVIDENCE: cartalith-llm/src/edit_actions.rs `OPERATIONS = ["raise","lower","smooth"]` (no more ops); llm_edit.gd, llm_region.gd (paint brush, "Use the box"); no profile-as-region-source symbol found in llm_region.gd or llm_settings_window.gd; touch painting of the region not found (UNVERIFIED); _llmedit_probe.gd exists, commit 0550e1b1 "tests for four mutation gaps"; STATUS.md L33 still says apply-path mutation tests not run.
LEFT: more edit ops, touch region painting, profile as region source, the apply-path mutation run.
FIX: none (row accurate).

### L77 Probe failures not yet attributed
VERDICT: PARTLY
EVIDENCE: STATUS.md ~L964 "`_llmpanel` twice green" and ~L1245-1247 re-runs `_llmpanel` green (PC-6 era), after the L29 5-fail note: so `_llmpanel_probe` is no longer failing. `_nethint_probe` phone leg: no later green note found (UNVERIFIED). JNI "Open app settings": never run on a device (STATUS L30).
LEFT: `_nethint_probe` phone leg; JNI call and LineageOS "Allow network access" on a device (OPEN-owner/device).
FIX: replace "`_llmpanel_probe` (5 checks)," with "`_llmpanel_probe` is green again (STATUS.md, PC-6 entries);".

### L79 UI copy pass, remaining surfaces
VERDICT: OPEN
EVIDENCE: evidence/ui-copy-2026-10-26/COPY_CHANGES.md covers only Assistant settings, the panel, quick-bar chip and LlmError Display; STATUS.md has only the "first cut" entry (~L2005).
LEFT: the rest of the shell's strings.
FIX: none.

### L81 Retrieval: RK-8 to RK-11
VERDICT: OPEN
EVIDENCE: STATUS.md table ~L5654-5657: RK-8 "not started" (single-run observations only), RK-9/10/11 "not started", conditional.
FIX: none.

## Phone and touch

### L84 Owner questions of 2026-11-09: answered (Ruling EM) (sub-bullets L85-L87)
VERDICT: PARTLY
EVIDENCE: STATUS.md has "Ruling EM", "Pressure controls", "Colour paint per-point pen strength", and the entry "Edit any map element from its context: step 3 ... the Note tab for labels and icons (2026-11-12)" (~L194-198) which says that Note tab is built. app.gd `_open_welcome_when_drawn` sets `OS.low_processor_usage_mode = false` across the waits and restores it (commit 50769614).
LEFT: the welcome idle-mode handset check (owner's).
FIX: in L87 delete "except the Context panel's Note tab for them" (built 2026-11-12).

### L91 Performance pass across devices (sub-bullets L94, L95, L102, L113)
VERDICT: PARTLY
EVIDENCE: L94 menus slow: no measurement found (OPEN-owner/device). L95 and L113: the candidate patch IS applied (app.gd `_open_welcome_when_drawn`, commit 50769614), so "unverified on a device and not committed" and "The candidate patch was not applied" are STALE. L102 (struck, closed): menus.gd `_build_ui_scale_submenu` exists. The whole pass is owner-scheduled after the rest of the list.
LEFT: the pass itself (later by owner order); handset check of the welcome fix; tablet menu timing.
FIX: in L95/L113 replace the "candidate ... not committed/not applied" sentences with "The fix (hold idle mode off across the first-frame waits) is applied, 50769614; the handset check is the owner's."

### L117 Menu drag panned the map behind it: fixed 2026-11-09; handset check open
VERDICT: OPEN-owner/device
EVIDENCE: STATUS.md has a "Scroll-fix" entry; only the owner's finger on the Pad 3 settles it.
FIX: none.

### L118 Central touch-scroll fix: built; handset check open
VERDICT: PARTLY (built; handset check is OPEN-owner/device)
EVIDENCE: STATUS.md "### Central touch-scroll fix built (2026-11-08)" (~L677). Code not re-opened beyond STATUS (UNVERIFIED in code).
LEFT: owner confirmation; Asset library survey rows, editable TextEdit, inertia on proxy-scrolled lists.
FIX: none.

### L123 After a MORE window closes the tab stays lit
VERDICT: STALE
EVIDENCE: commit d0045b68 (MS-2) made the bar "MAP . domain . STORY . MENU": there is no MORE tab now. dcc_shell.gd `_refresh_phone_tabs()` lights cells from `_phone_tab`. Whether MENU stays lit after a window closes: UNVERIFIED.
LEFT: re-check on MENU (needs a probe or device).
FIX: change "MORE" to "MENU (renamed by MS-2); unverified since".

### L124 Phone Close button is 29 px, under the 44 px floor
VERDICT: DONE (code); handset check UNVERIFIED
EVIDENCE: dcc_widgets.gd phone button-bar pass (~L3437-3470): every OK/Cancel button gets `custom_minimum_size = Vector2(0, DccTheme.PHONE_TAP_MIN)` (44) plus a 12 dp foot; `git log -S'Plus a foot, because the bar had none'` = 0ed18540. The 29 px figures in GUI_GAP_REGISTER ~L6322 are the pre-fix table.
FIX: strike the row, "fixed in 0ed18540 (`DccWidgets` phone button-bar floor)".

### L125 The phone's gesture list overclaims one-finger panning (struck)
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: viewport_host.gd `_touch_motion`, `_TP_CANDIDATE`, `_finger_navigates()` exist.
FIX: none.

### L128 Search field lag of 10 to 40 s on the 24601 world over adb
VERDICT: OPEN-owner/device (unexplained; no cause or fix found in STATUS/GUI_GAP).
FIX: none.

### L129 DS-13 the phone viewport control column redesign
VERDICT: PARTLY
EVIDENCE: STATUS.md MS-5 steps 1-4 (~L421-L451); viewport_host.gd `_pen_seen`, `DccSettings.pen_draws_finger_pans()` (~L1512-1517), `_finger_navigates()` (~L1637); STATUS ~L288 "Standard stylus input built" lists SH-3 in the shell; STATUS ~L435: item 7 (3D camera) is 3D-2's, no 3D-2 build entry found.
LEFT: the 3D camera (3D-2); glass figures (300 ms, 34 px, 0.15/doubling: MS-10); mockup follow-up (design/mobile-shell-2026-10-31/ ds13.js still "pending DS13-Q1"; not re-checked).
FIX: remove "*Pen draws, finger pans* (SH-3: ... the finger-versus-pen split is not)" from Left: SH-3 is in the shell; only its glass pass remains.

### L143 Mobile shell refactor (MS-1 .. MS-11)
VERDICT: PARTLY (the row's "What is left" is badly stale)
EVIDENCE: STATUS.md headings + git log: MS-1 S4.2 (1e996036 line), S4.3 (2347ff1d, 37c04bce), S4.4 (1e996036, 415f0196), S4.5 batches 1-5 (0d09445f, bca96ac7, ccaf6b61, 747a4213, 021821c1), S4.6 parts 1-2 (faac91ee, 73605b4c) all built; MS-2 d0045b68; MS-3 2d029770 + 82234124; MS-4 9bae6952; MS-5 steps 1-4 (c302a4ed, 8fcab5fc, e6f3cfbd); MS-6 3b04e444; MS-7 docking rule 263a7b26; MS-8 99566ee5. No commit/heading for MS-9 (search runs every command) or MS-11 (quick drawer: only a comment at dcc_shell.gd ~L8302); MS-10 is the device pass.
LEFT: MS-1 S0 device rows and B7 (6T); 11-inch tablet at true dp / 150 % UI scale; B9 export half and B10's other probes not re-run; MS-5: card `param` rows on phone, 3D camera; MS-7 menu bar beyond 263a7b26 (UNVERIFIED); MS-9, MS-10, MS-11.
FIX: replace "What is left of MS-1 (S4.1 ... is built): S4.2...S4.6 ..." with "S4.1-S4.6 are built (STATUS.md); what is left of MS-1 is the device rows (S0, B7)"; replace "Left of MS-5: the tool strip, the tool fan ... context card's per-domain verbs" (built in steps 2-3) and "MS-6, MS-7's menu bar and MS-11 are next" with "MS-6 built; MS-9, MS-10, MS-11 remain".

### L174 EF-9 timing on the 6T
VERDICT: OPEN-owner/device
EVIDENCE: STATUS.md ~L2308 (Ruling CL; EA item 16 correction): "no phone was measured".
FIX: none.

## Windows and shell

### L178 PW-4 (planner pop-out) (struck)
VERDICT: CLOSED-ALREADY-OK (built with PC-8; leftover owner-run: monitor 2, OS Ctrl+Z/Y/S).

### L179 Dialogs a pop-out spawns open on the asking window's monitor (struck, built)
VERDICT: CLOSED-ALREADY-OK
EVIDENCE: popout_host.gd `dialog_host` (~L508) and `present` (~L524); journey planner "Save journey" (~L3867) and "Capture party" (~L4071) prompts go through `PopoutHost.dialog_host`; STATUS ~L375-377 `_popdlg_probe` 97 checks. Leftover: dialog from a pop-out on monitor 2 (owner-run).

### L188 CQ-5
VERDICT: OPEN
EVIDENCE: CUSTOM_QUICK_BAR_SCOPE.md ~L596 "CQ-5 and CQ-6 left as next ... need a touch device"; CQ-6 later built (STATUS ~L2039); no CQ-5 heading in STATUS, no hold-slide-lift dock code in quick_bar*.gd. CQ-8 built (STATUS ~L403, `_cq8_probe.gd` exists).
LEFT: CQ-5 (tablet touch scale/gesture); its bars need glass.
FIX: none.

## Religion (FH-R2)

### L191 S7, world-instanced religions and schisms
VERDICT: OPEN
EVIDENCE: no per-world religion instance type in cartalith-civ; belief.rs ~L841 only names it as outside FH-R2; `roster::CIV_RELIGIONS` still the fixed list.
FIX: none.

### L198 Re-measure FH-R2's M2 cost bar on today's tree
VERDICT: OPEN
EVIDENCE: FH_R2_PLAN.md ~L314-315 still carries only the 2.4x / 0.75 ms (0.72..0.79) vs 0.30 ms figures; no later re-measurement in STATUS.md.
FIX: none.

## HTML working copy (owner: no HTML work)
### L207 Stage 2; L212 Stage 3; L215 Moving each version into Cartalith_RC; L220 The next versions (Ruling EC/EF) and v2.72 audits; L233 Found in the HTML by stage 1
VERDICT: OPEN-parked (per owner, not checked)

## Stylus and haptics

### L239 The VIBRATE permission (SH-1's preset half) is set
VERDICT: PARTLY (preset half DONE, device half OPEN-owner/device)
EVIDENCE: godot-project/export_presets.cfg L98 `permissions/vibrate=true`; STATUS ~L273/~L577 aapt output lists VIBRATE.
LEFT: the on-device `adb logcat | grep VIBRATE` check.
FIX: none.

### L242 Superseded by Ruling EE: standard input only
VERDICT: CLOSED-ALREADY-OK (a ruling, not work)
EVIDENCE: but Ruling EK later allowed a read-only plugin (cartalith-native/android/pen-plugin exists), so "SH-10 off" is only true of the input plugin.
FIX: append "(Ruling EK later admitted one read-only Stylo 2 plugin; see below)".

### L245 The Pad 3 has no haptics (owner, 2026-11-02)
VERDICT: OPEN-owner/device (informational fact; routes research Q10/Q11).

### L249 The rest of SH-0 still wants the device (wm density, getevent -lt, dumpsys input)
VERDICT: OPEN-owner/device

### L251 Parked by the owner: haptic feedback (SH-7, SH-8, SH-12, Q13)
VERDICT: OPEN-parked (owner decision).

### L256 The Stylo 2's double tap, side slide and battery (Ruling EK)
VERDICT: PARTLY
EVIDENCE: cartalith-native/android/pen-plugin/{build.gradle.kts, src/main, src/test}; shell/pen_link.gd; menus.gd `ID_PREF_TESTER := 951` "Stylus tester..."; stylus_tester.gd.
LEFT: Tool fan target, SP-0 (needs a person holding the pen), on-device bars SP-2..SP-6, SP-7, the experimental label.
FIX: none.

### L264 Stylus input, standard pointer input only: SH-2..SH-6, SH-9 in the shell
VERDICT: PARTLY
EVIDENCE: shell/stylus_input.gd, stylus_tester.gd, dcc_settings.gd `pen_draws_finger_pans` (~L633) and the tilt toggle (~L753); STATUS ~L288. Left items confirmed: no `tilt_size`/`tilt_min` in any crate (`paint_stroke_options`, cartalith-godot lib.rs ~L16551, has no tilt field); project.godot has no `disable_scroll_deadzone`.
LEFT: Rust half of SH-5; sculpt's inner hover ring (not re-opened, UNVERIFIED); the project.godot deadzone setting (owner); the glass pass (owner).
FIX: none.

### L271 Stale pen prose (Ruling BK)
VERDICT: PARTLY
EVIDENCE: fixed: dcc_shell.gd `_HAPTIC_MS` comment now says "only four have call sites ... (2026-11-02)". Still stale: TABLET_UI_SPEC.md ~L649 ("nothing in the shell reads pen pressure"); EDITING_TOOLS_SCOPE.md ~L204 ("are never read"); analysis.rs `ruggedness` doc ~L395 (*small* ruggedness claim); cartalith-godot/src/vault_saf.rs L9 ("no JNI binding anywhere in this workspace"); llm_net_hint.gd ~L67 ("no in-repo JNI precedent") while llm_keystore.rs / lib.rs in cartalith-godot use JNI.
LEFT: those five prose/comment fixes.
FIX: drop "`dcc_shell.gd` `_HAPTIC_MS`" from the list (fixed).

## Housekeeping and gates

### L278 Gate: the Rust workspace test is recorded in STATUS.md (2026-10-26)
VERDICT: STALE (date only)
EVIDENCE: STATUS.md has later gate lines (~L122 "270 suites, 7 873 passed, 0 failed, 149 ignored"; ~L150, ~L178). Standing instruction fine. Not re-run (no builds).
FIX: replace "(2026-10-26)" with "(newest entry: grep STATUS.md for 'cargo test --workspace')".

### L279 Test hygiene list (MISTAKES_CANDIDATES_2026-10-26.md)
VERDICT: STALE
EVIDENCE: commit 82c2b8fd "MISTAKES.md: the owner's edits plus the staged candidates, merged at the owner's request"; the staging file still exists.
FIX: point the row at MISTAKES.md (merged) rather than the staging file.

### L280 Manual items for the owner
VERDICT: PARTLY
EVIDENCE: "merge the candidates into MISTAKES.md": DONE (82c2b8fd). Rest (uncommitted project.godot change, Credential Manager entry, Cartalith CPL Scratch folder, C:\cw leftovers) is owner-machine state: UNVERIFIABLE-HERE.
FIX: delete "merge the candidates into `MISTAKES.md`,".

## Stale statements in other documents (not edited in this pass)

- `UNWIRED_FUNCTIONS.md` section 2 row 1 ("no Camera3D or MeshInstance3D in shell/") is false since `shell/view3d.gd` (3D-1, `c0ca18a9`).
- `STATUS.md` LM-9 still says user-given landmark names and the shell link are unbuilt; `df36f8d1`, `54e3aace` and `context_note.gd` (`vault_landmark_entity_id`) built them.
- `STATUS.md` lines near the top still say the consent fix is not built, contradicting its own LLM consent entry.
- `TABLET_UI_SPEC.md` 4.2 and `EDITING_TOOLS_SCOPE.md` 3.2 (pen pressure "never read"), `analysis.rs::ruggedness` doc, `vault_saf.rs` and `llm_net_hint.gd` (no JNI) remain stale prose, as the stylus row says.

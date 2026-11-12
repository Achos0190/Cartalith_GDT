# OUTSTANDING_WORK.md audit C: the independent check of audits A and B, 2026-11-12

A dated cut, findings only. `cartalith-native/docs/STATUS.md` is the state. This file records an independent re-check of
`OUTSTANDING_AUDIT_2026-11-12_A.md` (top through 2.6) and `OUTSTANDING_AUDIT_2026-11-12_B.md` (2.7 to the end), done on
origin/main `19cc19b9` (the four pushed branches were at the same commit). No sub-agents were used. Nothing was built or
run, and no code was changed. "Opened" means the symbol, test or commit was grepped or read in this pass; a row's own
text or a STATUS line was never taken as the only evidence for a DONE.

Row titles are the key; line numbers drift. "A" and "B" below mean the two audits.

## What A and B got wrong

| | A (top to 2.6) | B (2.7 to the end) |
|---|---|---|
| Rows re-checked | 85: every row A listed, DONE and CLOSED rows included | the PARTLY and OPEN rows (72) against the day's landings; 10 of its closed rows spot-checked at their symbols |
| Verdicts confirmed | 83 (two of them, UI copy pass and RK-8..RK-11, were right when made and were superseded by later commits the same day) | 70 of the 72 re-checked; the 10 spot-checked closures all held at their symbols |
| Verdicts corrected | **2**: Lake filling (item 2 was built in the tree A audited); the mobile shell row's "what is left of MS-1 is the device rows" (S4.5's assistant dialog is not built) | **4**: two of the 72 (the Vault editor row, stale clauses missed; the PC shell row, PC-5's landmark binding built and still listed) and two of its 26 closed rows (LOD-D7 and Volcano summits, counted as plain closed though their struck text routes open halves) |
| Damage done by the audit's own edit | **5 rows** blanked by a literal `\1` (a regex back-reference written into the file by commit `66baf61c`): the router row's and zero-length row's titles, the whole Seed 707 row (with the next row joined onto its line by a literal `\n`), the whole EF-9 row (same), the overlay-export row's title. All five restored here from `66baf61c^` | none found (its diff is 22 line-for-line replacements) |
| Fixes A wrote down and did not apply | the owner-questions Note tab clause, the welcome-fix "not committed" clause, DS-13's SH-3 clause, the PhotoCraft row's two stale clauses, and the four other-document fixes it listed (`UNWIRED_FUNCTIONS.md`, `STATUS.md` LM-9, consent, and B's RP cells). All applied here after re-verifying each | — |
| Evidence quality | blocks A-D each name symbols; the top paragraph's "Edits applied" overstates what landed | DONE/STALE rows name symbols; its 37 OPEN rows say only "re-opened; no code found that closes it". A sample re-checked (multi-ridge, gully spacing `EDIFICE_GULLY_SPACING_CELLS` 4.0, edifice floor `rad_cells.max(2.0)`, GF-9 commits, settlement Move/way Rename absent) held |

## Rows closed or corrected in `OUTSTANDING_WORK.md`

| Row | Verdict | Evidence opened |
|---|---|---|
| The router's smoothed points can leave the map | restored (struck, FIXED) | text from `66baf61c^`; `cartalith-civ/tests/route_km_on_map.rs` exists |
| A zero-length route plans as 0 days | restored (struck, FIXED) | `cartalith_civ::jp_route_is_zero_length` (`lib.rs`) |
| Seed 707 sea route of 0.1 day (R8) | restored, OPEN | text from `66baf61c^`; no `route_get(i).points` print in `journey_planner_view.gd`; the Military manpower row that had been joined onto its line is kept as `b00a0686` left it |
| EF-9 importance-driven refinement | restored, PARTLY | text from `66baf61c^`; `ef9_bridge.rs` imports `conform_plan, select_tiles` and calls `select_tiles`; no incremental importance refresh in `importance.rs` |
| Export of overlays | title restored, open half kept | `export_overlay.gd`, `engine_bridge.gd::export_session_begin`; no test asserts the overlay adjustment stack in export pixels |
| Lake filling | **corrected**: item (2) BUILT | `cartalith_civ::lake_balance::apply_lake_balance`, `lake_pools`; `WorldParams::lake_balance` true in `cartalith-godot/src/params.rs` defaults; `render::water_bodies_floored_balanced`; `a193b421` (Ruling ET), an ancestor of A's `d3873d70`. Items (4), (5) open |
| PhotoCraft editing tools | two stale clauses fixed | `STATUS.md` "Four shell backlog rows closed (2026-10-12): a loaded look restores its map"; `STATUS.md` 3D-0 note (a): device loss does not reproduce on 4.7.2 |
| Rivers painted into the map | two residuals marked superseded | `river_delta.rs` (the 2026-11-11 delta redesign, `d2aa4f3c`); `river_field.rs` `P_FULL` (the RZ law, `2fb48fdd`, `871052f8`) |
| `UNWIRED_FUNCTIONS.md` backlog (section 1) | title trimmed | see the `UNWIRED_FUNCTIONS.md` fix below |
| Owner questions of 2026-11-09 (Ruling EQ item 2) | "except the Note tab" removed | `context_note.gd` binds `label` and `icon` (`vault_label_entity_id`, `vault_icon_entity_id`); `8206f6cb` |
| Performance pass: welcome screen | "not committed" removed | `app.gd::_open_welcome_when_drawn` holds `OS.low_processor_usage_mode` false and restores it; `50769614` |
| DS-13 (top block) | SH-3 is built | `viewport_host.gd` returns `_pen_seen and DccSettings.pen_draws_finger_pans()`; `STATUS.md` "SH-3 built" |
| Mobile shell refactor | "MS-7's menu bar and MS-11 are next" corrected | `263a7b26` (MS-7); no drawer symbol (only a comment at `dcc_shell.gd` naming it); MS-9 not recorded as met; S4.5's assistant dialog: `b18169c4` "contract chosen; not built" |
| Vault editor and linking (3.3) | **corrected** (B missed it) | `vault_saf.rs` `move_file`, `can_move` (`75e8947e`); `vault_window.gd::request_rename_of`, `NoteRename` (`b5cd8c2a`); `LandmarksDoc.user_names` and `global_tools.gd` calling `landmark_set_user_name`; `cartalith_vault::LIST_CAP` 12 000 (`923cb657`). Still open: landmarks in Find (no landmark rows in `place_search.gd`), the device passes |
| PC shell (3.3) | **corrected** (B missed it) | PC-5's "landmark binding" built: `context_note.gd` calls `vault_landmark_entity_id` (`df36f8d1`). Private notes in the Note tab: no `private` in `context_note.gd`, still open |
| River combs (2.13) | OPEN, annotated | Ruling ER's rough routing (`77270a55`) changed the network the combs come from; re-measure first |
| Optional LLM assistant (3.1) | PARTLY, annotated | MORE is MENU since `d0045b68`; the touch-scroll fix covers lists (`STATUS.md` 2026-11-08); glass re-check owed |
| GUI replacement stages 3, 5, 6, 7 (2.10) | PARTLY, pointer updated | the `STATUS.md` RP cells corrected below |

## Audit A, row by row

C = confirmed, X = corrected. Evidence is what this pass opened.

| Row | A said | C | Evidence |
|---|---|---|---|
| `UNWIRED_FUNCTIONS.md` backlog | STALE | C | `view3d.gd` `_cam: Camera3D`; the doc fix A listed had not been made (made here) |
| Landmark M8 | OPEN | C | `landmark.rs::kinds()`: 49 entries, 31 `buildable: true`, 18 `false` |
| Territory lasso | closed | C | `civ_tools_bridge.rs::polygon_cell_mask` |
| 16K/32K export | PARTLY | C | `export_overlay.gd`, `export_dialog.gd`; stale head annotated in the row |
| CV-23 | closed | C | `project_bridge.rs` test `a_recorded_year_comes_back_holding_what_was_true_then_not_what_is_live_now` |
| wantCounts | closed | C | `cartalith_civ::place_settlements_with_counts` |
| IN-13 trade flows | DONE | C | `trade/tests.rs::sea_caravans_are_one_per_port_pair_combining_both_directions`; `civ_trade_bridge.rs::civ_set_trade_tariff` |
| Landmark M9 | PARTLY | C | `place_name.rs::landmark_names` with `user_names`; `context_note.gd` calls `vault_landmark_entity_id`; still `buildable: false`: `shrine`, `battlefield_historic`, `destroyed_fortress` |
| SP-2 departure date | OPEN | C | `project_bridge.rs` journey DTOs carry `start_year` only |
| Router off-map | closed | C, row restored | see above |
| Zero-length route | closed | C, row restored | see above |
| Seed 707 | OPEN | C, row restored | see above |
| Military DL Q2 | OPEN | C | `b00a0686` re-recorded harness pins only; no bracket-5 build |
| DL Q7, Q8 | OPEN | C | no bracket-3 levy commit |
| Journey override fingerprint | OPEN | C | no leg or override fingerprint in `project_bridge.rs`, `story_bridge.rs`, `journey_planner_view.gd` |
| `library/` binary payload | OPEN | C | `cartalith-io/src/project.rs` registers only `library/assets.json`, `travel.json`, `settlement_types.json` |
| Vault Browser mockup | closed | C | `_vaultlayout_probe.gd` exists |
| Vault phone toolbar on device | PARTLY | C | `markdown_editor.gd` `EditorKeyboardDock`; glass pass owed |
| 3D viewport | PARTLY | C | `view3d.gd` still sets "Terrain is not drawn in 3D yet"; no `MeshInstance3D` outside `view3d_gate.gd` |
| 3D height slider | OPEN | C | no slider in `view3d.gd` |
| Colour paint | PARTLY | C | no CP-4..CP-7 commit; the Scale-with-the-map notice is documents only (`STATUS.md` "Ruling EB ... nothing built in the shell") |
| Overlays follow the style preset | PARTLY | C | `map_overlay.gd::set_style_treatment` |
| Zoom-sensitive rivers, Tanaka | closed | C | `tests/tanaka_contours.rs` |
| Raisz, mappa mundi, OS symbology | OPEN | C | no implementation in `crates/` or `shell/` (the one grep hit is "unmappable") |
| Sawtooth, LOD-D3, LOD-D6 | closed | C | `lod_sweep.rs::seam_ratio`, `lod_bridge.rs::morph_for_zoom`, `lod_worker.rs::LodSnapshot` |
| EF-9 | PARTLY | C, row restored | see above |
| Snow aspect term | PARTLY | C | `render.rs::snow_aspect_shift` knee still `slope / 0.04`; `git log -S` on `crates/`: `19c38d96` only |
| Diamond lattice | PARTLY | C | `render.rs` `relief_lowpass` |
| Ruling 17, live sea level | closed | C | `sea_live_bridge.rs::set_sea_level_live`, `pass.rs::rewrite_stamps` |
| PhotoCraft tools | PARTLY | C, fixes applied | see above |
| Lake filling | PARTLY ("item 2 PARKED") | **X** | see above |
| Overlay export | STALE | C, title restored | see above |
| ET-C2 | PARTLY | C | `export_raster.rs` test `screen_equals_export_at_grid_resolution_under_every_stack` |
| Curves LUT | PARTLY | C | `adjust.rs` Curves |
| ET-D3/D2 pen gaps | OPEN | C | `PaintEditor` (`paint_bridge.rs`) has no undo `#[func]`; the ledger's `EntryKind` (`undo.rs`) has no paint-specific kind |
| ET-D4 timing | OPEN | C | `tools.rs::civ_dijkstra_path_guided`; `WAY_DRAG_SPACING_CELLS` doc says the measurement "has NOT been made" |
| Waypoints slider | OPEN (owner) | C | `infrastructure_workspace.gd` `var _way_drag_options`, `tool_bar.gd::_controls_way` |
| Render target gap list | PARTLY | C | row text; look items not re-measured |
| Asset pack tools | PARTLY | C | row's "Nothing is built" already marked history |
| GPU stream power | PARTLY | C | `cartalith-gpu/shaders/` has `gpu_thermal.wgsl`, no stream-power shader; `STATUS.md` GLI-E "none started" |
| GPU flood, telemetry, tiled, real-device tests | OPEN | C | no flood shader; no telemetry code; `multi.rs` exists |
| Small Cartalith LLM | OPEN (deferred) | C | Ruling EP |
| Consent UX | PARTLY | C | `llm_assistant_window.gd` `LlmConsent.present(PopoutHost.dialog_host(...))`, "Turn on Assistant enabled"; `beee4808` |
| Undo-and-regenerate offer | OPEN | C | no such offer in `shell/` |
| LLM-7 remainder | PARTLY | C | `edit_actions.rs` `OPERATIONS` = raise, lower, smooth; `0550e1b1` body: "Not done: apply-path mutation tests" |
| Probe failures | PARTLY | C | `STATUS.md` "`_llmpanel` twice green" |
| UI copy pass | OPEN | C, superseded | `f5e61001`, `19cc19b9` landed after A; the remainder is the held files |
| RK-8..RK-11 | OPEN | C, superseded | `3d29e575` built the machinery after A; `STATUS.md` RK-8..RK-11 rows say the live runs are not done |
| Owner questions 2026-11-09 | PARTLY | C, fix applied | see above |
| Performance pass | PARTLY | C, fix applied | see above |
| Menu drag, touch-scroll | device / PARTLY | C | `STATUS.md` "Scroll-fix", "Central touch-scroll fix built" |
| MORE tab stays lit | STALE | C | `PHONE_TABS` is MAP, domain, STORY, MENU |
| Phone Close button | DONE | C | `dcc_widgets.gd` uses `DccTheme.PHONE_TAP_MIN`; `0ed18540` |
| Gesture list, search lag | closed / device | C | `viewport_host.gd::_touch_motion` |
| DS-13 | PARTLY | C, fix applied | see above |
| Mobile shell refactor | PARTLY, "what is left of MS-1 is the device rows" | **X** | `b18169c4`: the assistant dialog's swap is chosen and not built; MS-9 and MS-11 have no build commit |
| EF-9 on the 6T | device | C | — |
| PW-4, pop-out dialogs | closed | C | `popout_host.gd` `dialog_host`, `present` |
| CQ-5 | OPEN | C | no CQ-5 entry in `STATUS.md` |
| S7, M2 cost re-measure | OPEN | C | `belief.rs` names S7 as outside FH-R2; no re-measure commit |
| HTML working copy | parked | C | owner: no HTML work |
| VIBRATE | PARTLY | C | `export_presets.cfg` `permissions/vibrate=true` |
| EE, Pad haptics, SH-0, parked haptics | ruling / device / parked | C | — |
| Stylo 2 | PARTLY | C | `android/pen-plugin/`, `shell/pen_link.gd`, `stylus_tester.gd` |
| Stylus input | PARTLY | C | `dcc_settings.gd::pen_draws_finger_pans` |
| Stale pen prose | PARTLY | C | all five still stale: `TABLET_UI_SPEC.md` "nothing in the shell reads pen pressure", `EDITING_TOOLS_SCOPE.md` "are never read", `analysis.rs` "*small* ruggedness", `vault_saf.rs` "There is no JNI", `llm_net_hint.gd` "no in-repo JNI precedent" |
| Gate, test hygiene, manual items | STALE / STALE / PARTLY | C | `82c2b8fd` merged the candidates |

## Audit B

* **Spot-check of 10 closed rows, all held at their symbols:** G1 (`roster.rs` `CIV_GOVERNMENTS` [17], `CIV_GOVERNMENTS_REFERENCE_LEN` 9; `manpower.rs` `GOVERNMENT_EXTRACTION`, `CITIZEN_MODERNISATION` 0.68, `SOLDIER_UPKEEP_BY_BRACKET`, `GOVERNMENT_LEVY_BONUS`; test `the_khanate_levy_bonus_is_derived_from_its_era_band`; `g1_baseline.rs::g1_0_baseline_columns`; `_gov_probe.gd`; `faction_roster_window.gd::_government_choice`), the vault `private` badge (`vault_privacy.gd`, `vault_window.gd` badge block), the phone generation menu (`PHONE_TABS`), Recent worlds names (`naming.rs::world_name`), the status-bar width (`dcc_shell.gd` `_status_tail`, `_fit_status_tail`), lava flows (`cartalith_terrain::lava_flows`, `volcano_look.rs::flow_ground`), volcano summits (`orogen.rs::high_ceiling_top`, `tests/lava_flows.rs::summit_ceiling_census`), the Village preset (`quantize_flat_palette` `BANDS` 5.0), LOD-D7 (`info_tile.rs`, `shell/info_tiles.gd`), the Faction hub (`cartalith-civ/src/religion_library.rs`).
* **Corrected:** LOD-D7 and Volcano summits are struck rows whose text still routes open work (categorical and civ-dependent views; heightmap exports clip summits above 1), so they are PARTLY, not plain closed. The Vault editor and PC shell rows (above).
* **Re-checked against the day's landings, no change needed:** Retrieval (row already updated by `3d29e575`), Vault milestone 4 (updated by `75e8947e`; still a device pass), the map-context row in 2.12 (updated by `8206f6cb`, `bf9622b6`; settlement Move and way Rename still have no engine function), the mobile menus row (MS-11 = drafts 06 and 07, unbuilt), geology (`construct_relief` exists; GF-9 has no commit since `21dfc52e`), the volcano edifice rows.

## The "built but listed" search

Method: a script took every milestone identifier on a `STATUS.md` line that says built, finished, done or closed, and
listed every open-region line of `OUTSTANDING_WORK.md` (the top block and sections 1 to 3.3) naming that identifier
within 120 characters of "not built", "not started", "next", "still open", "left:" or "nothing is built". Each hit was
read. Most were history inside rows that already say so (GF-11, EXP-E4/E5, SP-G0, RR and RZ rows). The real ones are the
corrections above: Lake filling (RR-6), the Vault editor's VE-5, VE-6 and VE-9 edges, PC-5's landmark binding, SH-3 in
DS-13, MS-7 "next", the PhotoCraft saved-look defect, the welcome fix, the Note tab, the rivers-in-map residuals, and in
`STATUS.md` LM-9 and the RP stages. The EA row itself was closed in the archive on 2026-10-25; G1's row stayed open in
§3.1 while `STATUS.md`'s Governments ledger recorded G1-0 to G1-5 done.

## Stale claims fixed in other documents

* `UNWIRED_FUNCTIONS.md` §2, the 3D viewport row: "no `Camera3D`" replaced; `view3d.gd` has one, `view3d_gate.gd` holds the only `MeshInstance3D`; Ruling CS, 3D-0 and 3D-1 named.
* `STATUS.md`, top block: `_llmpanel_probe` marked green since; the consent fix marked built (`beee4808`); the candidates merge marked done (`82c2b8fd`).
* `STATUS.md` LM-9 and the landmark open-decision row 4: the player-given name and the shell link are built (`df36f8d1`, `54e3aace`, `context_note.gd`).
* `STATUS.md` RP table: RP-S3 and RP-S5 superseded in part (PC-9/PC-11 menus; the Context panel, `H_TOOL_OPTIONS` 40), RP-S6 superseded by the mobile shell, RP-S7 not run as a pass though every window reads `DccTheme`. None is measured against its `spec/` file. The group header and total corrected.

Not fixed here (a backlog row owns them): the five stale pen and JNI comments listed under "Stale pen prose".

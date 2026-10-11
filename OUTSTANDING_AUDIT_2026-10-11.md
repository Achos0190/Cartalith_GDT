# OUTSTANDING_WORK.md audit, 2026-10-11 (read-only; no repository file edited)

Method: read MISTAKES.md preflight (all rows), then every non-struck row/list item in OUTSTANDING_WORK.md sections 1, 2.1-2.13, 3, 4, 5; opened STATUS.md entries and the code symbol for each item where noted. "Evidence" names what was opened. Status vocabulary as briefed. Table-row IDs are `section.n` (my numbering, row line numbers in OUTSTANDING_WORK.md given as L####).
Verification depth key: **[S]** = opened at the code symbol/file; **[ST]** = STATUS.md entry only; **[R]** = row text only (could not determine more; stated).

## Section 1. In flight

| ID | Item (row) | Status | Size | Evidence opened | Next step |
|---|---|---|---|---|---|
| 1.1 | UNWIRED backlog: 3D viewport (L2740) | OPEN-READY (research only) | L | [S] UNWIRED_FUNCTIONS.md section 2 (2 rows left); no Camera3D/MeshInstance3D in shell; LARGE_ITEM_RULINGS Ruling AZ ("Research tracks to start: the 3D viewport") | Answer the 3 commissioned research questions (3D_TERRAIN_RENDER_RESEARCH.md); no build scheduled |
| 1.2 | UNWIRED backlog: bounded sculpt preview (L2740) | BLOCKED-OWNER (declined at its own doc) | M | [S] UNWIRED_FUNCTIONS.md section 2 "Declined at its own doc" (golden_parity_render risk) | Only an owner word reopens it |
| 1.3 | Landmark M8 residual: 22 of 49 kinds `buildable:false` (L2743) | PARTIAL | L | [S] `landmark.rs` kinds(): `grep -c 'buildable: false'` = 24 incl. 2 doc lines = 22; `buildable: true` = 27; test string "twenty-seven kinds" at ~L5980 | Per-kind: most wait on M9 inputs (3.4-3.7) or civ traits; take kinds one at a time |
| 1.4 | `LandmarkKindSpec::not_built` doc comment stale (named in L2743) | OPEN-READY | S | [S] `landmark.rs` doc on `not_built` ends at "23 once border marker ... 2026-09-21": misses Battlefield (22) | Add the 22 link; comment-only |

## Section 2.1 Urban morphology
Every row struck/closed; 0 items.

## Section 2.2 The 2026-08-31 Large rulings (closed-in-text, not struck)
The section's table is empty; the 17 bullets are a closed-verification log whose titles are not struck. Each opened at its named symbol today, all present: `LabelClass`+`_draw_labels`; label cull `total_cmp`; `Family::SeaMark`; `get_rivers`; `civ_rebuild`; `crowding_in_force`; `_cut_selection` in `menus.gd`; `UNIT_MODES`; `_conflicts_within_menu`; `measurements.json` slot in `project.rs`; `_sync_color_space`; `PaintStamp::with_falloff`; `set_configured_thread_count`; `diagnostic_report.gd`/`diagnostic_review_dialog.gd`; `icon_arm`; `region_as_new_world`; settlement diagnostics ("dashed rather than printed as 0") **[R for the last, S for the other 16]**.

| ID | Item | Status | Size |
|---|---|---|---|
| 2.2.1-2.2.17 | the 17 bullets above (Labels panel, collision culling, Icons placement, river entity, civ authoring ops, settlement diagnostics, landmark funnel, Cut/Copy/Paste, Units, shortcuts, saved measurements, colour management, paint falloff, CPU threads, Report an issue, manual-icon tool, Region > New world) | DONE-BUT-LISTED (log, no work) | n/a |

## Section 2.3 Civilisation, economy, journeys

| ID | Item (row) | Status | Size | Evidence | Next step |
|---|---|---|---|---|---|
| 3.1 | Export E4 batches B-D, off-thread with progress (L2946) | OPEN-READY | M | [S] shell grep: `export_session_*`/`export_image` called only from `_exportsession_probe.gd`, `_exportoptions_probe.gd`, `_riverzoom_probe.gd`, `_rv5export_probe.gd`; `export_session.rs` exists; STATUS EXP-E4 partial | Build batches B-D (one-tile registration etc.) |
| 3.2 | Export E5 dialog + JPEG alongside PNG (L2946) | BLOCKED-WORK (on 3.1) | M | [S] no JPEG crate in Cargo.lock; no shell caller; STATUS EXP-E5 not started | After 3.1; add a JPEG encoder dependency |
| 3.3 | IN-13 caravans, sea-lane half (Ruling AQ) (L2956) | OPEN-READY | S | [S] `civ_trade_bridge.rs` ~L613-641 builds `caravans` from `net.way_load` only; its comment says a sea route "is not a way", stale since Ruling AQ | One caravan per sea lane; fix the stale comment |
| 3.4 | Landmark M9(a): per-landmark authored name/state (L2957) | OPEN-READY | M | [S] `struct Landmark` fields: id, kind, class, x, y, elevation, score, importance, causal, seed: no name/state | Add field + `LandmarksDoc` round trip |
| 3.5 | M9(b): `battlefield_historic` past vs current battles (L2957) | BLOCKED-WORK (on 3.4) | S | [R]+[S] no state field to separate years | After 3.4 |
| 3.6 | M9(c): civilisation-traits input for Cultural kinds (`shrine`) (L2957) | OPEN-READY (needs a design pass; research section 26 forbids hardcoding) | M | [R] | Design the input, then build |
| 3.7 | M9(d): shell entry linking a landmark to a vault note (L2957) | OPEN-READY | S | [S] `vault_landmark_entity_id` has no shell caller: only a comment in `engine_bridge.gd` L476 | Hang a "notes" row on `CTX_LANDMARK` |
| 3.8 | M9: should an authored battle be exempt from landmark spacing (L2957) | BLOCKED-OWNER | S | [R] no ruling found in LARGE_ITEM_RULINGS | Owner call |
| 3.9 | SP-2 residual: journey departure date (L2960) | OPEN-READY | S | [S] `struct Journey` (`travel_library.rs`) has id,name,party_preset,route,start_year,...,plan; no day field; row says widening `entities/journeys.json` is a format change | Add optional start day + format note |

## Section 2.4 Vault, archive

| ID | Item | Status | Size | Evidence | Next step |
|---|---|---|---|---|---|
| 4.1 | Project archive: `library/` art binary payload (L2969) | OPEN-READY (Ruling AP settled storage: embed in the zip) | M | [S] `SAVEFILE_COMPAT.md` still has heading "16.5 library/ the binary half, still not stored"; `project.rs` stores only `library/assets.json`, `travel.json`, `settlement_types.json` | Spec the entry layout in section 16.5, then build + round-trip test |
| 4.2 | Vault editor: phone toolbar vs on-screen keyboard (L2975) | BLOCKED-HARDWARE | S | [R] | Check on handset |

## Section 2.5 Rendering, appearance, export-adjacent

| ID | Item | Status | Size | Evidence | Next step |
|---|---|---|---|---|---|
| 5.1 | Rivers in map (L2985): delta look (thin hairlines) | BLOCKED-OWNER (a look decision) | S | [ST] STATUS RIM-4 entries | Owner to rule on delta look |
| 5.2 | Rivers in map: owner gate call, order vs discharge | BLOCKED-OWNER | S | [R] | Owner call |
| 5.3 | Rivers in map: owner nod that exports now carry floodplain tint and bank line | BLOCKED-OWNER | S | [R] | Owner nod |
| 5.4 | Rivers in map: export-test gap (two `WorldGen` call sites), 8192 tile/export check, z2.5-4 tile step, coverage bar | OPEN-READY (8192 windowed probe wrote no frame: may need hardware) | S-M | [ST] STATUS L379-384 RIM-2/4/5/7 built; `river_field.rs` exists | Windowed 8192 run; tighter coverage metric |
| 5.5 | Rivers in map: flat stub at tributary end in z16 tile | OPEN-READY | S | [R] row says pre-existing | Fix stroke rasterizer butt end |
| 5.6 | Style-preset overlays: thin-glyph label weight on Woodcut/Ink/Vintage (L2986) | BLOCKED-OWNER | S | [S] grep of LARGE_ITEM_RULINGS and STATUS for "thin-glyph/label weight": no ruling | Owner call (rest of the row is built) |
| 5.7 | Raisz physiographic pictograms (Ruling BI) (L2993) | OPEN-READY (scope it first) | L | [R] no scope file exists (`ls *SCOPE.md`) | Write the scope |
| 5.8 | Mappa-mundi figurative icons (L2994) | OPEN-READY (scope first) | L | same | Scope |
| 5.9 | Ordnance Survey symbology (L2995) | OPEN-READY (scope first) | L | same | Scope |
| 5.10 | EF-9.4: wire `select_tiles` + conformed tile into the viewport behind a setting (L3022) | OPEN-READY | M | [S] `select_tiles` (subdivision.rs) and `world_elevation_tile_conformed` (elevation.rs) referenced only in cartalith-engine and `cartalith-civ/tests/ef9_mixed_edges.rs`; no shell/godot use | Build EF-9.4 |
| 5.11 | EF-9.5: keep importance current after an edit | BLOCKED-WORK (on 5.10) | M | [S] heading exists in ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md L891 | After 5.10 |
| 5.12 | EF-9: `carve_rivers=false` river-gain measurement | OPEN-READY | S | [R] | One harness run |
| 5.13 | Snow aspect term on main map (L3026) | PARTIAL (built; acceptance bar not met, found biased) | M | [S] `snow_aspect_c` in `render.rs`, tests `color_space.rs`, `lod_d4_ice_and_snow.rs`; no shell reference | Scale-aware slope knee; on/off same-pixels measure |
| 5.14 | Diamond lattice / diagonal hatching (L3029) | PARTIAL (hatching fixed; deep-zoom lattice unconfirmed) | S-M | [S] `relief_lowpass` in `render.rs` (true in shipped appearance, false in `js_reference`); STATUS L271 | Owner re-check of deep-zoom lattice on a real world; candidate (c) only if residue |
| 5.15 | Editing tools: ET-D2 real pen/touch/hardware checks (L3032) | BLOCKED-HARDWARE | S | [ST] STATUS L275 ("done except real-hardware checks"; pen/touch rows synthetic) | Real pen/digitizer |
| 5.16 | ET-D3 sculpt on the stroke engine | OPEN-READY | M | [S] `cartalith-spatial/src/stroke.rs` exists; EDITING_TOOLS_SCOPE.md section 4 | Build ET-D3 |
| 5.17 | ET-D4 road drawing + ET-D5 brush controls | BLOCKED-WORK (sequential after D3 by scope order) | M | [S] scope section 8 order | After 5.16 |
| 5.18 | ET-C1..C4 per-layer adjustment stacks | OPEN-READY (queued by Ruling BZ order D,C,E,F) | L | [S] scope section 5 | After D |
| 5.19 | ET-E1..E3 colour management (tagged PNG, profile import, CMYK warning) | OPEN-READY (queued) | L | [S] scope section 6 | After C |
| 5.20 | ET-E4 wide-gamut/HDR display | BLOCKED-WORK (needs ET-C3) | M | [S] scope section 8 | After C3 |
| 5.21 | ET-F1..F3 Look control + variants | BLOCKED-WORK (on C) | L | [S] scope section 7 | After C |
| 5.22 | Lake filling: which of the ranked fixes to build (L3033) | BLOCKED-OWNER ("Owner call needed on which to build"; no ruling in LARGE_ITEM_RULINGS) | M | [S] `WaterBodies::basin`/`basin_outlet` exist in `cartalith-civ/src/lib.rs` L412/L419 (item 3 done, data only; `lib.rs` godot L4807 clears them); LAKE_FILL_RESEARCH.md exists | Owner picks among floor / water balance / connectivity / breach |
| 5.23 | Generated labels: regenerate on a fit change (L3034) | OPEN-READY | S | [S] `cartography_workspace.gd::_regenerate_labels` sends `px_per_cell` once; no resize/fit hook found | Hook fit change to regenerate |
| 5.24 | Generated labels: exact culling keeps 49/141 settlement names; milder box? | BLOCKED-OWNER | S | [R] | Owner call |

## Section 2.6 GPU, threading, memory

| ID | Item | Status | Size | Evidence | Next step |
|---|---|---|---|---|---|
| 6.1 | Stream-power erosion on GPU (L3051) | OPEN-READY | L | [S] `GPU_STREAM_POWER_SCOPE.md` exists; no `gpu_stream*.wgsl` (17 shaders listed; gpu_thermal.wgsl present = thermal done); `stream_power_kernel_bounded` not yet phase-split as far as STATUS shows | SP-G0: split phases, measure, go/no-go |
| 6.2 | `build_water_bodies` priority flood on GPU (L3055) | OPEN-READY (research-grade) | L | [R] | Research parallel Planchon-Darboux |
| 6.3 | Performance telemetry (L3060) | BLOCKED-OWNER (parked, Ruling AZ) | M | [R] | Reopen only on an owner word |
| 6.4 | Tiled/chunked GPU compute (L3062) | OPEN-READY | L | [R] `multi.rs` band split covers one kernel | Extend beyond gpu_warp |

## Section 2.7 Android and device

| ID | Item (row) | Status | Size | Evidence | Next step |
|---|---|---|---|---|---|
| 7.1 | Three-platform design conformance (L3127) | BLOCKED-WORK ("when ALL GUI work is done") | L | [R] | After GUI work closes |
| 7.2 | Phone shell must match Android canvas 100% (L3131) | PARTIAL | L | [ST] DCC-P412 done (phone 412dp migration), CM-5/6, phone menu; no conformance ledger | Per-screen inventory conformance pass |
| 7.3 | `MOUSE_FILTER_STOP` controls eat drags (L3132) | OPEN-READY | M | [S] `grep ColorPickerButton`: 8 `.new()` sites remain (asset_library, faction_roster x2, ramp_bar, right_dock, cartography, render x2); population figure 123-155 is stale | Re-measure, then triage |
| 7.4 | Godot remote debugger against the APK (L3133) | OPEN-READY (may need an export_presets owner call) | M | [R] | Try attach |
| 7.5 | Tablet canvas is a different shell (L3134) | PARTIAL | L | [ST]+[R] tablet rail/menu committed wf49 per L3150; `.claude/resume-2026-09-12/` holds reverted patches, untracked | Finish composition per TABLET_UI_SPEC |
| 7.6 | Tablet palette structural change (L3135) | UNVERIFIED (no STATUS record of adoption; needs windowed capture) | M | [S] `DccTheme.TABLET`/`TABLET_PORTRAIT` exist, hairline model unverified | Windowed contrast recompute |
| 7.7 | DCC_SHELL_SPEC section 11: stale citations (L3136) | OPEN-READY | S | [S] counted in shell: `DCC Shell.dc` 22, `Android Phone.dc` 17, `Menu Structure v2/v3` 9 (= 48; row says 49-51) | Judge each citation; not a blind rename |
| 7.8 | App never idles (L3137) | BLOCKED-HARDWARE (fix built, device re-measure owed) | S | [S] `app.gd` L625 `OS.low_processor_usage_mode = true` gated on owning the window | SurfaceFlinger run on the OnePlus 12 |
| 7.9 | Tablet portrait: 5 Planet sliders 290 px in 232 dock (L3141) | OPEN-READY (patches kept) | M | [S] `ROW_LABEL_W := 132` still a plain const in `dcc_widgets.gd`; `.claude/resume-2026-09-12/grid_reverted_2026-09-13.patch`, `grid_v2_reverted_...patch` present | Third attempt from the kept patches |
| 7.10 | Tablet shows the PC layout (L3150) | PARTIAL | L | [ST] STATUS "Tablet targets 260 -> 1", tablet fits frame; rail work partly reverted | Remaining composition per TABLET_UI_SPEC |
| 7.11 | Invisible OFF switch track: fix the canvas, not the shell (L3151) | DONE-BUT-LISTED (Ruling C: no code change by design); canvas fix is the owner's design project | S | [R] | Strike; note the designer action |
| 7.12 | `--generate` capture leg crashes the GPU (L3152) | UNVERIFIED (3 tablet figures; Ruling Y device-loss handling since built) | M | [R] | Re-run capture |
| 7.13 | Generate blocks the main thread (L3155) | PARTIAL | M | [S] `warm_render_caches`, `drawn_water_digest` in godot `lib.rs`; `engine_bridge.gd` L758-766; [ST] STATUS L283: stall halved at 2048, ~40% at 4096; remaining stall ~1.0 s / ~3.0 s; 8192 not measured | `cell_color`/texture pass off-thread; 8192 re-measure |
| 7.14 | Paint layers: original raster on opacity slider (Ruling BR) (L3156) | PARTIAL | M | [ST] STATUS L310 (desktop Biome+Terrain, GDScript only), L378 (phone slider built) | True snapshot in Rust (Sculpt moves the "original"); Splat has no raster |
| 7.15 | Zoom residual: ~8 frames over 16.7 ms; one ~27 ms first notch (L3158) | OPEN-READY | S | [R] | Identify/pre-warm first-notch frame |
| 7.16 | Boot ~9.4 s main-thread stall (L3163) | OPEN-READY (needs profileable build or bisection) | M | [S] `project.godot` has no `boot_splash`; no STATUS entry | Debuggable build or bisect |
| 7.17 | Generation menu not found on phone (L3164) | BLOCKED-HARDWARE (reachability on glass) | M | [R]; GENERATE tab exists in code per row | Tap-through on handset |
| 7.18 | On-glass method run/calibrated (L3165) | DONE-BUT-LISTED (method recorded; the open six features are 7.19) | M | [R] | Strike |
| 7.19 | Six features never driven on device (L3166) | BLOCKED-HARDWARE | M | [R] | Device pass |
| 7.20 | Measure reduced generation peak on handset (L3169) | BLOCKED-HARDWARE | S | [R] | Record app peak |
| 7.21 | Lower suitability plateau further (L3170) | BLOCKED-WORK (only if 7.20 shows the phone too tight) | M | [R] | Conditional |
| 7.22 | Exercise R1's Godot-side hunk on the handset (L3172) | BLOCKED-HARDWARE | S | [R] | Device run |

## Section 2.8 Discipline debts

| ID | Item | Status | Size | Evidence | Next step |
|---|---|---|---|---|---|
| 8.1 | Annotate: GDScript `static func`/`var` pass (L3182) | OPEN-READY | M | [ST] STATUS L279-280; row says census_v2 does not count `static func` (dcc_widgets ~110 vs 17 counted) | Extend census, annotate |
| 8.2 | Annotate: generated test files, annotate in their generators (3 golden_parity_* + tile_biome_fixture.rs + untracked scratch `lattice_diag.rs`) | OPEN-READY | S | [R] | Edit generators |
| 8.3 | Annotate: independent verifier for batches 6-19 + the ~40 shell files (none parse-checked in Godot) + rest of gpu crate | OPEN-READY | M | [ST] STATUS L270, L279-280 | Verifier pass; `--check-only` each file |
| 8.4 | Annotate `shell/tool_bar.gd` (owner's uncommitted edit) | BLOCKED-OWNER | S | [S] `git status` shows ` M tool_bar.gd` | After the owner commits it |
| 8.5 | Re-freeze the reference to the DCC line (L3183) | BLOCKED-OWNER (v2.71 file unavailable, Ruling BZ item 4) | L | [S] `reference/` holds only v2.10, v2.11 | Owner supplies file |

## Section 2.9 Source-engine changes

| ID | Item | Status | Size | Evidence | Next step |
|---|---|---|---|---|---|
| 9.1 | Multi-ridge orogenic belts (v2.36/v2.35) (L3212) | OPEN-READY (Ruling AP scheduled) | L | [S] `cartalith-terrain` has `OrogenyParams`/`build_orogeny_field` with no sheet fields (`grep n_sheet/sheets` empty) | Multi-batch build per RC_ENGINE_CHANGES 6b |

## Section 2.10 Desktop shell

| ID | Item | Status | Size | Evidence | Next step |
|---|---|---|---|---|---|
| 10.1 | Look-history gaps: Painter/NPR/Rendering-advanced sliders, layer stack, colour space, saved-look load, quality tier (L3243) | OPEN-READY (later ET-C4/F3 overlap) | M | [S] `look_history.gd` header: "What it does NOT cover" lists exactly these | Extend `snapshot()` |
| 10.2 | Picker follow-ups: phone/touch scaling untested, popup 298x624 hits bottom edge on a 1000 px window | OPEN-READY (part device-bound) | S | [ST] STATUS dbd09e6f entry | Windowed probe at phone/short window |
| 10.3 | Toolbar undo/redo squares, Ctrl+Y, ramp undo (the row's other clauses) | DONE-BUT-LISTED (clauses stale) | S | [S] `dcc_shell.gd` L1511-1528 `can_undo_any`/`can_redo_any`; `app.gd` L407 `KEY_Y`; `look_history.gd` covers ramp | Strike those clauses |

## Section 2.11 Alignment audit residue (L3324, 12 open sub-items)

| ID | Sub-item | Status | Size | Evidence |
|---|---|---|---|---|
| 11.1 | Economy district fills (oreyard, fishery, sawyard, granary, warehouse) | OPEN-READY | M | [S] `grep -c` of those names in `urban_layout_draw.gd` = 0 |
| 11.2 | Generated-but-undrawn urban layers | UNVERIFIED (row names none) | M | [R] |
| 11.3 | Vault restore failures never surfaced | UNVERIFIED | S | [S] grep of shell for restore-failure surfacing empty, not an absence proof |
| 11.4 | Journey re-snap outcomes invisible | OPEN-READY | S | [S] `journey_resnap_report` has no caller outside `engine_bridge.gd` |
| 11.5 | Planner list not rebuilt after regenerate | UNVERIFIED | S | [R]; related closed row restored list on reopen only |
| 11.6 | Atlas has no reader | OPEN-READY | M | [ST] STATUS LODI-M3 "done, no consumer" |
| 11.7 | Stored LOD pyramids carry no colour space | UNVERIFIED | S | [R] |
| 11.8 | "Fail with error" refuses generation with GPU off | UNVERIFIED | S | [S] `engine_bridge.gd` ~L707-723 VRAM-budget refusal exists |
| 11.9 | `build_water_bodies` four call sites, cost unmeasured | OPEN-READY (a measurement) | S | [R] |
| 11.10 | Section 6s.4 corner-only parcel water test (inherited defect) | BLOCKED-OWNER (fix needs a golden re-baseline ruling) | S-M | [S] `blocks.rs` module doc L43: "rejects a lot with any corner in the water" |
| 11.11 | Region export omits `params.json` | OPEN-READY | S | [S] `zip_region_export(e, params_json: Option<..>)`; only tests call it, and with None/test bytes |
| 11.12 | `snow_aspect_c` has no control | OPEN-READY | S | [S] grep of shell `.gd` for `snow_aspect` empty |

## Section 2.12 Map context / faction hub

| ID | Item | Status | Size | Evidence | Next step |
|---|---|---|---|---|---|
| 12.1 | Faction hub FH-0..FH-9, FH-R1 (L3339) | DONE-BUT-LISTED | L | [S] STATUS L352-361 FH-0..FH-9; `religion_library.rs`; `faction_roster_window.gd` 46 FH- mentions | Strike the title; keep 12.2 |
| 12.2 | FH-R2 custom religion with real attributes | BLOCKED-WORK (the researched expansion S1-S7) + owner S3 ruling | M | [S] FACTION_HUB_DESIGN.md L421 "not scheduled" | Needs the religion-model expansion first |
| 12.3 | CM-4 device check: pen barrel button, haptics (L3348) | BLOCKED-HARDWARE | S | [ST] STATUS CM-4 row (title says "pen barrel button verified" but body says haptics not observed on desktop; not device-verified) | Owner's tablet |

## Section 2.13 Terrain generation

| ID | Item | Status | Size | Evidence | Next step |
|---|---|---|---|---|---|
| 13.1 | Geology: GF-11 construction-alone, G5 cost 1.211/1.187x vs 1.20 bar (L3360) | BLOCKED-OWNER (relax bar or optimise) | M | [ST] STATUS GF-11 entries; Ruling BY items 1,3,4 | Owner call |
| 13.2 | Geology: GF-4 glacial/coastal, GF-5 karst, GF-6 carve/LOD/landmarks, GF-8 lithology painting not started | UNVERIFIED (dependencies on GF-2/3 gating not established) | L | [S] STATUS: GF-1,2,3,7,10,11 entries only; `params.rs` `geology_processes = false` | Decide after 13.1 |
| 13.3 | Geology: GF-2/GF-3 gated-off mechanisms, GF-9 re-baseline, GF-13 | BLOCKED-WORK (on 13.1) | M | [S] `params.rs` L188; GF-12 not built by Ruling BY | After 13.1 |
| 13.4 | Landslip and pinnacle landforms (Ruling BG) (L3363) | BLOCKED-WORK (on geology + no scope doc exists) | L | [S] `ls *SCOPE.md`: none for landslip | Scope doc after 13.1 |
| 13.5 | River combs after Ruling BV: stronger factor (L3364) | BLOCKED-OWNER | S | [S] thinning built: `river_draw_plan`, `PARALLEL_GAP_CELLS` in `cartalith-hydrology/src/lib.rs` (row cites the wrong crate) | Owner call |
| 13.6 | Fixed-size labels: per-label fixed/scaling choice (Ruling BW item 1) (L3365) | OPEN-READY | S | [S] `cartography_workspace.gd` size_mode keyed per label class (`_label_role_size_mode`); no per-label control found in `right_dock.gd` | Add per-label control |
| 13.7 | Labels: hit-box staleness after a zoom notch | UNVERIFIED | S | [R] |
| 13.8 | Labels: 4 px handle floor | DONE-BUT-LISTED | S | [S] Ruling BZ item 5c: floor stays |
| 13.9 | Receiver 2-cycle at river mouth, seed 71077345 (L3366) | OPEN-READY | S | [S] `river_gap_harness.rs` present; not re-run (no cargo) |

## Section 3.1 / 3.2
All rows struck; 0 items.

## Section 3.3 Blocked on hardware or design

| ID | Item | Status | Size | Evidence | Next step |
|---|---|---|---|---|---|
| 33.1 | BUILD_ANSWERS section 3: Data manager in Environment prototype; 13 of 24 families (L3420) | PARTIAL / stale | M | [S] `data_manager_window.gd` exists (rebuilt 2026-08-20 against canvas); Asset library has 9 families and Section 5 AS-16 declines the 24-family rail | Show screenshots to owner for approval; drop the 24-family half |
| 33.2 | BUILD_ANSWERS section 4: phone failure / storage-full states (L3421) | PARTIAL | S | [S] `app.gd` "storage-full guard" `_save_blocked_by_space`, "Generation failed" via `_report_failure` (commit fd9de7cc); `content_description` ~19 sites | Content descriptions, dynamic type, 48dp sweep; screenshots |
| 33.3 | DS-13 phone viewport control column (L3422) | OPEN-READY (derive from DCC vocabulary by standing rule) | M | [ST] STATUS GGR-DS13 "not started" | New derivation |
| 33.4 | Observe phone landscape composition (L3423) | OPEN-READY (blocker claim is false) | S | [ST] STATUS AND-10: `adb shell wm user-rotation lock 1` rotates it; landscape captured 2026-08-20 | Re-capture current build |
| 33.5 | Section 47 hi-DPI blur at scale 3.664 (L3424) | BLOCKED-HARDWARE | S | [R] |
| 33.6 | GPU section 21 thermal/mobile GPU scheduling (L3425) | BLOCKED-HARDWARE (needs a LAST_BACKEND reading on a handset) | M | [R] |
| 33.7 | 3D research's three commissioned questions (L3426) | OPEN-READY (Ruling AZ started the track) | M | [S] Ruling AZ text | Answer Q2 first |
| 33.8 | Vault M4: Android SAF picker plugin + `_saf_dispatch` handler + shell surface (L3427) | OPEN-READY | L | [S] `grep _saf_dispatch`: only `_vaultsaf_probe.gd` defines it; no `.java/.kt` `OPEN_DOCUMENT_TREE`; `android/build` only | Build plugin, handler, surface |
| 33.9 | Vault M4: device pass | BLOCKED-HARDWARE (and on 33.8) | M | same |

## Section 4 Open decisions
Zero rows by design (text). 0 items.

## Section 5 Declined/shelved (listed, not audited as work): 28 live entries (+1 struck)
Owner-parked: 3D viewport; v2.71 half 2 woodland; province lasso; hillshade on thumbnails. Declined on measurement/architecture: section 21 GPU render path; tile-scoped recompute; per-stage re-execution; orogeny graph-tracing and Dijkstra GPU; the hard-hazard sequential functions; Landmark M1 consolidation; R6; crater constants (Ruling AP); bounded thread pool (text already says built); ComputeBackend trait; overlay lever 1; hi-DPI mitigations; Android .so size. No engine counterpart: AS-14/15/16; Vault section 35 criteria 6-7; Vault section 11 etc.; Journey Planner DOM/widening; Military; Urban; asset-library conveniences; `state.erosion`; Warfare/Narrative/playback/projection; DECISIONS 7b territorial expansion; Data manager five nav rows. Struck: 16K/32K export.

## Totals

**Distinct work items audited: 124** (107 individually numbered open/sub-items in sections 1-3.3, plus the 17 closed-in-text bullets of 2.2). Sections 2.1, 3.1, 3.2 and 4 hold no live item; section 5 lists 28 live declined entries (not audited as work). Several rows were split into independent sub-items (Export 2, Landmark M9 5, Rivers-in-map 5, Editing tools 7 groups covering 14 ET milestones, Smaller gaps 12, Annotation 4, Geology 3, Vault M4 2); counting each source row once gives about 65 rows.

| Status | Count (of 124) |
|---|---|
| OPEN-READY | 47 |
| BLOCKED-OWNER | 14 |
| BLOCKED-WORK | 11 |
| BLOCKED-HARDWARE | 11 |
| PARTIAL | 10 |
| UNVERIFIED | 9 |
| DONE-BUT-LISTED | 22 (5 numbered + the 17 in 2.2) |

**DONE-BUT-LISTED, by name (coordinator may strike):**
- 2.2.1-2.2.17: the 17 bullets (CARTO Labels panel; label collision culling; CARTO Icons generated placement; the river entity; civilisation authoring operations; settlement diagnostics overlay; landmark funnel; Cut/Copy/Paste/Select all; Units km/mi/nmi; rebindable shortcuts; saved measurements + CSV; colour management; paint brush falloff; CPU worker threads; Report an issue; the manual-icon tool; Region > New world from selection). Sixteen opened at their symbol today, one (settlement diagnostics "dashed not 0") by text only.
- 7.11 invisible OFF switch track (Ruling C: no code change; the canvas fix is the designer's).
- 7.18 "METHOD: on-glass method has now been run" (a method record; its open part is 7.19, the six unverified features).
- 10.3 the toolbar undo/redo squares / Ctrl+Y / ramp-undo clauses of the colour-editing follow-up row (the row itself stays open for 10.1, 10.2).
- 12.1 Consolidated Faction hub FH-0..FH-9 and FH-R1 (only FH-R2 remains, 12.2).
- 13.8 the 4 px label-handle floor (Ruling BZ item 5c: stays).

Near-done rows worth striking once one small thing lands: 7.13 (warm built; only the remaining stall), 5.14 (hatching fixed; owner re-check of lattice), 33.2 (storage-full and failure states exist).

**Verification depth.** About 62 of the 124 items were opened at a code symbol or repo file (shell grep, Rust struct/fn, SAVEFILE_COMPAT heading, git status/ls); about 20 were checked against a STATUS.md entry; the rest (about 42, mostly device-bound, owner-call and the long Android/tablet rows) rest on the row's own text and are tagged [R]. The 17 items in 2.2 count as symbol-verified. I did not run cargo, Godot or any build, so no test result is asserted.

## Rows whose text contradicts STATUS.md or the code

1. **Section 1 UNWIRED row says "Ruling AZ: the 3D viewport track is greenlit"; section 5, STATUS.md Orientation and UNWIRED_FUNCTIONS.md say 3D is parked and no 3D work is scheduled.** Ruling AZ says only "Research tracks to start: the 3D viewport (its three commissioned questions)". Section 5's "parked" entry and section 3.3's "Ruling AZ: start the 3D viewport track" cannot both stand as written; accurate reading: research started, viewport build not scheduled.
2. **L3155 (generate freeze): "Authorised and in progress; no landing commit found yet (2026-10-10)".** STATUS.md L283 (same date) records it built and verified, and the symbols exist: `WorldGen::warm_render_caches`, `drawn_water_digest` (godot `lib.rs`), `engine_bridge.gd` L758-766. Row should be PARTIAL (remaining stall about 1.0 s at 2048, 3.0 s at 4096).
3. **L3032 (editing tools): "Next step: write the scope document" contradicts its own header and the repo**: `EDITING_TOOLS_SCOPE.md` exists (ET-D1..ET-F3), `cartalith-spatial/src/stroke.rs` exists, ET-D1/ET-D2 built (STATUS L275-277; commits 5a8731fb, e5893dd1, b649ca3d).
4. **L3243 (colour editing follow-ups): "toolbar undo/redo squares read only can_undo(); Ctrl+Y unbound; no ramp undo".** `dcc_shell.gd` L1511-1528 uses `can_undo_any()/can_redo_any()`, `app.gd` L407 binds `KEY_Y`, `look_history.gd` covers ramp stops.
5. **L3423 (phone landscape): "adb cannot force it".** STATUS AND-10 (done) records `adb shell wm user-rotation lock 1` rotating it, and landscape captured 2026-08-20.
6. **L3421 (BUILD_ANSWERS section 4): phone failure/storage-full states "undesigned... build them".** `app.gd` carries a documented "storage-full guard" (`_save_blocked_by_space`) and "Generation failed" via `_report_failure` (commit fd9de7cc); only content descriptions/dynamic type/48dp sweep remain. **L3420 (section 3)**: `data_manager_window.gd` exists, and its "13 of 24 families" half collides with section 5's AS-16 (24-family rail declined; library has 9 families).
7. **Section 5 "Warfare ... declined" and "Military: campaigns / change over time / garrisons declined"** are superseded by Ruling AW (2026-09-24); `cartalith-civ/src/campaign.rs` exists.
8. **Section 5 "The Android .so at 156-171 MB with line-tables-only"** vs STATUS AND-13 (done 2026-09-28: `strip = "debuginfo"`, .so 35.5 MB) and the struck 2.7 row that says so.
9. **L3364 (river combs) cites `cartalith-godot/src/river_stroke.rs (river_draw_plan, PARALLEL_GAP_CELLS)`**; both live in `cartalith-hydrology/src/lib.rs` (tests in `cartalith-godot/tests/river_gap_harness.rs`).
10. **L3134 (tablet canvas) says adopting it "needs an owner decision before it is built"; L3150 and TABLET_UI_SPEC.md say adoption was decided by the owner on 2026-09-07.**
11. **L3156 (paint original raster): "the phone cannot reach the Paint bar".** STATUS L378: phone Paint is reachable and carries the Ruling BR Original slider.
12. **STATUS.md disagrees internally on RIM-7**: L309 "RIM-7 not built" vs L379-380 built; the OUTSTANDING row (L2985) says built.
13. **Stale comments the rows themselves name, confirmed**: `landmark.rs` `not_built` doc stops at 23 (item 1.4); `civ_trade_bridge.rs` ~L616 sea-route comment (item 3.3).
14. **L3132**: its "ColorPickerButton x8 / 123-155 STOP controls" predates the confirm-dialog migration; 8 `ColorPickerButton.new()` sites still exist, the total was not re-measured.
15. **L3182 (annotation)**: the 0/0/0 census excludes GDScript `static func`/`var` (row discloses it); `tool_bar.gd` is still modified in the working tree.
16. **STATUS.md "What is left" counts (155/69/122) and "Android build: Sixteen rows" (table lists 17)** are stale; STATUS marks the first as stale itself.

## Owner decisions that currently block the most work

1. **GF-11 (rock-aware construction): accept about 1.2x cost (relax the G5 bar) or ask for optimisation.** Blocks the rest of the geology milestones (13.2, 13.3) and the landslip/pinnacle landforms (13.4): the longest chain.
2. **The v2.71 DCC-line reference file (Ruling BZ item 4)** blocks the re-freeze (8.5), the regenerated FUNCTION_INDEX and re-tagging FUNCTIONAL_CONTRACT.md.
3. **Lake fill: which ranked fix to build (floor / water balance / connectivity / breach)** blocks 5.22 and any consumer of the new `basin` data.
4. **Rivers-in-map look calls: delta look, order-vs-discharge gate, and the nod that exports now carry floodplain tint and bank line** (5.1-5.3) block closing the RIM scope.
5. **FH-R2 (custom religion attributes) needs the researched religion expansion S1-S7 plus an S3 ruling** (12.2): an unscheduled chain.
6. **Single small calls**: thin-glyph label weight on Woodcut/Ink/Vintage (5.6), generated-label survival tradeoff (5.24), stronger river-comb factor (13.5), authored-battle spacing (3.8), the section 6s.4 corner-only parcel water test re-baseline (11.10), reopening parked telemetry (6.3).
7. **Not an owner decision but the largest single blocker: 11 items need a handset or tablet** (7.8, 7.17, 7.19, 7.20, 7.22, 4.2, 5.15, 12.3, 33.5, 33.6, 33.9).


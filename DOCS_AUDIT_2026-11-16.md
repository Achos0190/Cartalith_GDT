# Docs audit, 2026-11-16

A documentation-only audit, requested by the owner: *"go over the work that's done already, the documentation and the
outstanding work list with an opus agent to make sure everything is up to date and correct."* The scope was then widened
by the owner: *"I want every crate, gui, piece of code and functionality indexed and checked against the documentation."*
So it runs in phases, each committed on its own:

- **Phase 1** (this section): the work landed from 2026-11-12 to 2026-11-16 (`git log --since=2026-11-12 origin/main`,
  197 commits at `1952a77e`) and the documents that track it.
- **Phases 2-4** (sections below, added as they finish): the crate index, the Godot shell index, and the full cross-check.
  Their index is `CODEBASE_INVENTORY.md`, which is also R0 of `CODEBASE_REFACTOR_PROPOSAL.md`.

No product code was changed. This file is a dated cut of findings; status is `cartalith-native/docs/STATUS.md`'s.

## Phase 1: the 2026-11-12..16 work

### Method

- Read `MISTAKES.md`'s preflight first; applied its rows on backlog rows (re-open at the symbol), a document's claim not
  being evidence, constants not asserted against themselves, and absence claims (grep the concept).
- For every row struck or marked built since 2026-11-12 that names a symbol, grepped the symbol in
  `cartalith-native/crates` (`*.rs`) and `cartalith-native/godot-project` (`*.gd`), and read it where the claim was more
  than existence.
- For Rulings EW-FL (`LARGE_ITEM_RULINGS.md`), checked each item against a symbol, a `STATUS.md` entry and an
  `OUTSTANDING_WORK.md` row.
- For the section 7p switches, compared `DECISIONS.md` section 7p, `GENERATION_PARAMETERS.md` (both tables and the
  count line), `cartalith_engine::WorldParams::defaults`, `cartalith_godot::params::defaults` and
  `params::LOADS_WHEN_ABSENT`, key by key.

### Verified correct (evidence named)

| # | Claim | Evidence |
|---|---|---|
| V1 | 13 switches added since 2026-11-11 (`rough_routing`, `basin_routing`, `lake_balance`, `urban_wall_hug`, `urban_river_kind_map`, `urban_riverbank`, `urban_street_village`, `urban_market_scale`, `urban_harbour_front`, `civ.merge_ways`, `civ.river_front`, `civ.map_rivers`, `civ.harbour_box`, `civ.port_shore`, `civ.map_crossings`, `civ.port_reach`; 16 keys) are OFF in `WorldParams::defaults`, ON in `params::defaults`, OFF in `LOADS_WHEN_ABSENT`, and each has a `PARAMS` row and a `JS_PATHS` row | `cartalith-engine/src/lib.rs` defaults block; `params.rs` `defaults()`, `PARAMS`, `LOADS_WHEN_ABSENT` |
| V2 | `GENERATION_PARAMETERS.md` count 137 | `tests/param_catalogue.rs` asserts `PARAMS.len() == 137` (a literal); 137 `ParamSpec {` rows plus the struct definition |
| V3 | Every switch of V1 has a `GENERATION_PARAMETERS.md` provenance row and a table row | rows `rough_routing` .. `civ.map_crossings` |
| V4 | Every switch of V1 except `rough_routing` has a dated `DECISIONS.md` section 7p entry | entries dated 2026-11-12 .. 2026-11-16 (see C23 for the exception) |
| V5 | Symbols named by built rows exist: `DccTheme.key_hint`, `DccQuickBar.accelerator_text`, `GlobalTools.ring_cardinals`, `plate_ids_plausible`, `blur_output_plausible`, `VaultWindow._build_place_notes`, `note.open`, `PinMarker.tilt_angles`, `_lm_hidden`, `_coach_blocked`, `_dismiss_coach_tips`, `PhoneMenu._peek_chip_row`, `process_resident_bytes`, `_field_label_fit`, `GPU_PATH_KERNELS`, `default_gpu_on`, `gpu_enabled`, `GrainScale::relief_fade`, `build_river_front`, `traced_to_cell`, `dry_box_kind`, `wallhug::CLEARANCE_M`, `PORT_REACH_KM`, `um_box_has_sea`, `HARBOUR_HOUSING_KEEP`, `auto_thread_count`, `town_placement_audit`, `PLAZA_MARKET_POP`, `_town_layer_zoom_reached` (Ruling FC), `cartalith_hydrology::rough` | grep in `crates` and `godot-project`, 2026-11-16 |
| V6 | `QuickBarPhoneList` deleted (`3f7ea940`) | only comments and `_qbplist_probe.gd`'s absence checks name it; `quick_bar_phone_list.gd` is gone (an untracked `.uid` remains) |
| V7 | `PoiKind::Inn` exists and keeps the wire form (`kind: "other"` + `kind_name: "inn"`) | `cartalith-civ/src/poi.rs`; `SAVEFILE_COMPAT.md` section 11.2a agrees |
| V8 | The `cartalith-llm` `ac1_static_sources` test is still red (open row is right) | `crates/cartalith-llm/data/action_static_sources.json` has no `poi` entry; last regenerated `80005fec` |
| V9 | Ruling FL items 1 and 2 are not built (open row is right) | no domain-sheet tool palette or quick-bar layout export/import in `shell/` (the `_tool_palette_row` in `app.gd` is the desktop bar) |
| V10 | Rulings EW, EX, EY, EZ (1-4), FA, FB, FC, FD (1-4, PP-1..PP-7), FE (1, 2), FF (1, 2, 5, 6; 3 and 4 need no build), FG, FI, FJ, FK are built as ruled, each with a `STATUS.md` entry; FE item 3 (deep plots) and FL are routed open | `LARGE_ITEM_RULINGS.md` lines for each, `STATUS.md` entries "Harbour towns built to their water" .. "Town walls hug the built-up area" |
| V11 | Ruling FH: ZR-0 run, no-go | `STATUS.md` "ZR-0"; `ZOOM_REFINEMENT_SCOPE.md` section 8 |

### Corrected (document, claim, truth, action)

| # | Document | Claim | Truth and evidence | Action |
|---|---|---|---|---|
| C1 | `OUTSTANDING_WORK.md`, "Where the current list stands" | Running: Ruling FK and the small GUI batch | both landed: `73e0df21`, `662d488c`; `3f7ea940`, `8dd94baf` | struck, dated note |
| C2 | same, Engine open | the serial generation floor needs a section 7p ruling | Ruling FL item 3 moves it into the refactor | struck, noted; FK and FI leftovers and FE item 4 added to the line |
| C3 | same, GUI open | MS-7 small items, VE-5/VE-6, CQ-8 deferrals "owner answers" | MS-7 pin and readouts and VE-5 tablet form built `8dd94baf`; VE-6 chooser and CQ-6 reorder built `3f7ea940`; MS-3 and CQ-8 answered by Ruling FL | line rewritten, dated note |
| C4 | same, "16k/32k PC runs, leftovers" | open: why the iGPU's 16k world differs | answered and fixed, `1a6800d2` (`plate_ids_plausible`) | struck, dated note |
| C5 | same, Ruling FH row | "Not built" | ZR-0 was run the same day (no-go); save prompt and disk cap wait on the owner's ZR decision | rewritten, dated note |
| C6 | same, City Viewer square-blocks row, item (5) | sea and lake mask runs still draw as squares | built, `5d3ea071` (`urban_shore.gd`) | struck |
| C7 | same, Ruling EZ item 1 sub-row | written as a build instruction, no status | built with Ruling FB, `88fb0ef3` (`riverbank::BANK_CLASS`) | status added |
| C8 | same, "The serial floor left in generation" | "Not scheduled" | Ruling FL item 3 schedules it inside the refactor | note added |
| C9 | same, Pad re-measure row | title: timed, digest identical; body: "Still open: the timing matrix, the Pad's digest and the throttling data" | the reduced matrix and digest are in `PERFORMANCE_BENCHMARKS.md` 12.7's table; throttling dropped by the owner (`c066f822`); only `RAYON_NUM_THREADS=1` is left | body sentence struck, dated note |
| C10 | same, town judgement row | river smoothing "moves to `RIVER_REALISM_SCOPE.md`" | that scope had no row or note for it (a ruling with no home) | note added there (C22) and pointed to here |
| C11 | same, Mobile shell row | MS-7 left: the pin on a floating dock and the 1 920 dp readouts | built, `8dd94baf` (`_ms7_probe.gd`) | struck, dated note |
| C12 | same, Mobile shell row and GUI plans audit row | MS-3 "node chips and tool palette", "needs an owner design call" | Ruling FL item 1: a tool palette only, node chips dropped | both rewritten |
| C13 | same, GUI plans audit row (Quick bar) and the CQ-5 row | CQ-8 deferrals each need an owner answer; "CQ-8's three extensions were built" read as the same three | Ruling FL item 2 answered all three (wanted, not built); the built three (Ruling EH) are different features | both clarified |
| C14 | `cartalith-native/docs/STATUS.md`, "Pad 3 re-measure of generation" | heading "timing matrix blocked" and "Not done: the matrix, the digest" under a first sentence that reports both | as C9 | heading and sentence given a dated correction |
| C15 | `PERFORMANCE_BENCHMARKS.md` 12.7 | "There are no Pad timings, no Pad digest ... in this section" | the section's own later table has both | dated correction before the paragraph |
| C16 | `PERFORMANCE_BENCHMARKS.md` 13.6 | the iGPU wrong-world hypothesis; "a follow-up should compare its `plate_id`" | done: every cell plate 0, fixed `1a6800d2` | dated note |
| C17 | `WORLD_SCALE_TERRAIN_PLAN.md` | the iGPU result "unexplained (hypothesis ... unverified)" | as C16 | dated note |
| C18 | `GUI_PLANS_AUDIT_2026-11-16.md` | CQ-6 reorder "never built"; VE-6 chooser open; MS-3 needs a design call; CQ-8 deferrals need answers | as C3, C12, C13 | "Post-audit changes the same day" section appended; rows left as the dated cut |
| C19 | `CUSTOM_QUICK_BAR_SCOPE.md`, CQ-8 row | the three deferrals each need an owner answer | answered, Ruling FL item 2 | dated note in the row |
| C20 | `CITY_MAP_LOOK_SCOPE.md`, opening | "The left-click behaviour was not built"; "the engine has no tavern" | CL-3 built `d2a8b6a3`; CL-11 POIs `449bb22a` (`cartalith_civ::poi::PoiKind`) | dated note under the paragraph |
| C21 | `CODEBASE_REFACTOR_PROPOSAL.md` | no mention of the serial floor | Ruling FL item 3 routes it here | routed note in section 8 |
| C22 | `RIVER_REALISM_SCOPE.md` | nothing on the river through a town layout | Ruling FE item 4 routes it here | "Routed by Ruling FE item 4" note (candidate, unbuilt) |
| C23 | `DECISIONS.md` section 7p | (absence) `rough_routing`, Ruling ER (2026-11-11), the only switch of V1 with no dated entry | the switch exists, ON in the app, OFF in the engine and for an old save | dated entry added before 2026-11-12's |

### Found, not changed

| # | Document | Finding | Why left |
|---|---|---|---|
| N1 | `STATUS.md` | two 2026-11-16 entries ("Pin palette PP-6, landscape phone", "Pan-drag hint probe") sit at the very end of the file, not with the dated entries at the top; the "GUI leftovers built (2026-11-16)" paragraph sits above the "Where we left off" header | placement only, both correct in content; moving them while two lanes write `STATUS.md` invites conflicts |
| N2 | `OUTSTANDING_WORK.md` | the City Viewer square-blocks row and several sub-rows are not struck in their titles although every half is now built or routed elsewhere | each carries its own dated status; striking needs the owner's call on leftovers (1), (4), (6) |

### Stale `CLAUDE.md` rows (not edited, per the brief)

| # | Row | What is stale |
|---|---|---|
| L1 | `CITY_MAP_LOOK_SCOPE.md` | "the finding that the engine has no tavern entity: POIs are not ported" (CL-11 ported them, `PoiKind`); "The left-click change was **not** built; it is CL-3" (built `d2a8b6a3`) |
| L2-L14 | (absent rows) | 13 root documents have no routing row: `CODEBASE_REFACTOR_PROPOSAL.md`, `DELTA_GEOMETRY_RESEARCH.md`, `EDIT_FROM_CONTEXT_AUDIT.md`, `GUI_PLANS_AUDIT_2026-11-16.md`, `MEDIEVAL_TOWN_GROWTH_RESEARCH.md`, `PIN_PALETTE_SCOPE.md`, `RIVER_ZOOM_VISIBILITY_RESEARCH.md`, `STYLO2_PEN_PROPOSAL.md`, `TERRAIN_COMPUTE_METHODS_RESEARCH.md`, `TOWN_GENERATOR_JUDGEMENT.md`, `WORLD_EMBOSS_FINDINGS_2026-11-15.md`, `WORLD_SCALE_TERRAIN_PLAN.md`, `ZOOM_REFINEMENT_SCOPE.md` (method: every `*.md` at the root not named in backticks in `CLAUDE.md`); this file and `CODEBASE_INVENTORY.md` will join them |

### Phase 1 counts

- Verified correct: **11** claim groups (V1-V11), covering 16 switch keys in 5 places, 1 count, 30 symbols and 16 rulings.
- Corrected: **23** (C1-C23) in 10 documents.
- Found, not changed: **2** (N1, N2).
- Still open and correctly listed: Ruling FL items 1-2, FK leftovers (i)-(v), FI (iii) and (v), FF (iii)-(v), FE item 3,
  FE item 4 (now routed), FH's save and cap (behind the ZR decision), the `ac1_static_sources` test, the Pad's
  `RAYON_NUM_THREADS=1` column.
- Stale `CLAUDE.md` rows: **1** wrong row plus **13** missing rows.

## Phase 2: the crates, indexed and checked against their documents

The index is `CODEBASE_INVENTORY.md` (sections 1-6 and 9): 19 crates, 373 source files, about 452 000 source lines and
121 000 test lines, 6 538 `#[test]` and 275 `#[ignore]` attributes, the dependency graph, every module's first `//!`
line, the section 7p switches each crate names, a similarity table of 17 algorithm families, and the dead code.

### Method

A script over the tree (no build): `Cargo.toml` dependency sections; per-file regex counts; the first `//!` line per
`src/` file; a token index of all `.rs` and `.gd` files. Against the documents: `ARCHITECTURE.md`'s crate tree and
`README.md`'s crate list (crate by crate, dependency by dependency); `GENERATION_PARAMETERS.md` against every `PARAMS`
key (both directions); `SAVEFILE_COMPAT.md`'s archive layout (48 entries) against the code's strings; every
`js_*` helper `JS_SEMANTICS_AUDIT.md` names against `cartalith-jsmath` (including `libm.rs`'s re-exports); and every
`src/` module's file stem against the root documents plus `STATUS.md`.

### Verified correct

| # | Claim | Evidence |
|---|---|---|
| V12 | Every one of the 137 `PARAMS` keys is documented in `GENERATION_PARAMETERS.md`, and its 125 table rows name only live keys | key-by-key comparison |
| V13 | All 48 archive entries of `SAVEFILE_COMPAT.md`'s layout are named in the code | string search over `.rs` and `.gd` |
| V14 | Every `js_*` helper `JS_SEMANTICS_AUDIT.md` names exists in `cartalith-jsmath` | `lib.rs` and `libm.rs` (`pub use libm::{js_acos, js_atan, js_atan2, js_cos, js_exp, js_log, js_log10, js_sin}`) |
| V15 | Every `src/` file carries a `//!` module doc (Ruling BK) | 0 of 373 without one |
| V16 | Every `src/` module but one is named in a root document or `STATUS.md` | the exception is N4 |
| V17 | `ARCHITECTURE.md`'s dev-dependency cycle (civ -> engine -> gpu -> civ, test-only) and `cartalith-jsmath`'s zero dependencies | `Cargo.toml`s |

### Corrected

| # | Document | Claim | Truth and evidence | Action |
|---|---|---|---|---|
| C24 | `ARCHITECTURE.md` crate tree | "16 crates"; no `cartalith-llm` or `cartalith-llm-loopback`; `cartalith-civ` without `erosion`; `cartalith-godot` "every crate above except urban and erosion" | 19 crates; `cartalith-llm` [vault] and the test-only `cartalith-llm-loopback` [llm] exist; `cartalith-civ/Cargo.toml` depends on `cartalith-erosion` | tree corrected |
| C25 | `README.md` crate list | 17 crates; "`cartalith-vault` is the newest" | the two LLM crates are missing and are newer | list extended, dated note |

### Found, not changed

| # | Where | Finding | Why left |
|---|---|---|---|
| N3 | code: 4 `pub fn`s referenced nowhere, not even in their own file | `cartalith-civ` `way_join.rs::spliced_share`, `cartalith-hydrology` `rough.rs::noise_at`, `cartalith-llm` `place_vault.rs::plan_json`, `cartalith-spatial` `colour_paint.rs::has_emissive` | code; dead-code removal is R2's (`CODEBASE_REFACTOR_PROPOSAL.md`) |
| N4 | code, `cartalith-godot/src/progress_bridge.rs` | the only module no document names (`GenerationProgress`, the per-stage readout); it is annotated in place | low value; R0 records it |
| N5 | code comment, `cartalith-llm/Cargo.toml` | "Deliberately one dependency. A test (`no_network_dependency`) fails if another appears": there are two (`serde_json`, `cartalith-vault`) and the test (`tests/no_network.rs`) pins both | a comment in code; report only (Ruling BK backlog) |
| N6 | code | per-crate copies of small numeric helpers (`bil_c` x3, `lerp`/`mix`/`clamp01` x4 crates, `slope_at` x4) and three FNV variants | R1's similarity question, `CODEBASE_INVENTORY.md` section 5 |

## Phase 3: the Godot shell, indexed and checked

The index is `CODEBASE_INVENTORY.md` sections 7 and 8: 144 scripts (`shell/`, `shell/workspaces/`, and the root
`map_overlay.gd`, `credits.gd`, `smoke_test.gd`), about 194 000 lines and 6 959 `func`s; 177 `menus.gd` command ids;
44 named `DccSettings` keys in 29 sections; and the disabled menu rows. Probes (755 root `_*.gd`, 725 of them
`_*_probe.gd`) and the untracked editor add-ons (`addons/godot_ai`, `addons/godotsteam`, ignored by `.gitignore`;
`addons/fennara` is tracked and documented in `project.godot`) are not indexed.

### Method

The 37 `_todo(` calls in `menus.gd` were each read in context to separate the always-disabled rows from build- or
state-gated fallbacks, then searched in `UNWIRED_FUNCTIONS.md`, `GUI_GAP_REGISTER.md`, `DCC_CONTROL_INDEX.md`,
`STORY_MENUS_PROPOSAL.md` and `OUTSTANDING_WORK.md`. The GUI plans' milestones were checked by Phase 1 against
`GUI_PLANS_AUDIT_2026-11-16.md` (its 56 DONE verdicts were not re-opened one by one here; the four rows it got wrong
after landing are C18).

### Findings

| # | Where | Finding | Action |
|---|---|---|---|
| V18 | `menus.gd` | 4 rows are always disabled (*Print sheet (PDF)…*, *VTT battle map…*, *Anti-aliasing · anisotropy*, *3D viewport defaults*); the other 33 `_todo` calls are fallbacks for an older native library or an empty state | verified; listed in `CODEBASE_INVENTORY.md` section 7 |
| N7 | `UNWIRED_FUNCTIONS.md`, `GUI_GAP_REGISTER.md`, `DCC_CONTROL_INDEX.md` | none lists *Print sheet (PDF)…* or *VTT battle map…*; both are routed only through `STORY_MENUS_PROPOSAL.md` section 8 and `OUTSTANDING_WORK.md` ("Story menus: the proposal's §8 later rows", unscheduled by Ruling EA item 7) | left: routed and owned; the registers are dated cuts |
| C26 | `TABLET_UI_SPEC.md` | "**`Run stage NN` — Ruling D.** ... The button is drawn, runs the whole pipeline, and says so in its tooltip." | no such control exists (`git log -S 'Run stage'` over `shell/` finds only comments; `world_workspace.gd` explains each stage row regenerates on release) | dated correction; Ruling D routed in `OUTSTANDING_WORK.md` section 3.1 as an owner call (build or record as superseded) |

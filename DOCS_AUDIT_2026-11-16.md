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

## Phase 4: the whole record, since the start of the project

The owner widened the scope twice (*"I want every crate, gui, piece of code and functionality indexed and checked
against the documentation"*, then *"Not just since 12-11. Since the start of the project."*). This phase checks every
ruling, every `OUTSTANDING_WORK.md` row (open or struck), every `STATUS.md` entry and the reference documents against
the code as it stands today, so that a claim made true once and later broken or removed is caught.

### Method, and what it can and cannot see

1. **Symbol regression sweep.** Every backticked code-like token (a file name, or an identifier with an underscore of 6+
   characters or a CamelCase name) in `STATUS.md`, `OUTSTANDING_WORK.md`, `LARGE_ITEM_RULINGS.md`, `DECISIONS.md`,
   `GENERATION_PARAMETERS.md`, `SAVEFILE_COMPAT.md`, `ARCHITECTURE.md`, `README.md` and `JS_SEMANTICS_AUDIT.md`:
   **11 476** distinct tokens, each looked up in a token index of every `.rs`, `.gd`, `.wgsl`, `.gdshader`, `.tscn`,
   `.cfg`, `.toml`, `.kt`, `.json` and script file under `cartalith-native` (evidence folders excluded) and in the file
   names there. **10 716 resolve today.** The 760 that do not were triaged by script (identifiers in the reference
   HTML; probe short names whose `_<name>_probe.gd` exists; lines that themselves say *removed*, *renamed*,
   *replaced*, *superseded* and the like; struck rows), leaving **240** to read by hand, all of which were read. For each
   the last commit that touched the name in code was found with `git log -S` (appendix A). This sweep sees a named
   symbol vanish; it cannot see a symbol that still exists but no longer does what the prose says. That needs the
   reading of method 3.
2. **Ruling coverage.** All **168** lettered rulings (A to FL; 73 dated September, 54 October, 41 November) were
   counted against `STATUS.md`, `OUTSTANDING_WORK.md`, `DECISIONS.md`, the other root documents and the code. 158 are
   cited by `STATUS.md` or by a code comment. The other **10** were read one by one (below). The unlettered owner
   rulings of 2026-08-31 to 2026-09-06 (the eighteen Large rows, the numbered rulings 1-26) are cited by number in
   their own rows and were checked only where those rows came up in method 3.
3. **Reading.** The open rows of `OUTSTANDING_WORK.md` sections 1 to 4 were listed and every one whose text claims
   *not built*, *not started*, *nothing is built* or *never built* was checked at its symbol; rows whose title says
   CLOSED but which sit in a numbered section were found by the same listing.

### Rulings with no `STATUS.md` or code citation (all ten read)

| Ruling | Subject | Verdict at the code |
|---|---|---|
| B (2026-09-07) | the selected Preferences chip keeps Medium and gains the accent | **built**: `DccTheme.FONT_MONO_MED` / `mono(..., medium = true)`, the lit segment's accent wash in `dcc_widgets.gd` |
| C (2026-09-07) | the invisible OFF track is a canvas defect | needs no build (a canvas ruling); `OUTSTANDING_WORK.md` closed its row 2026-10-11 |
| **D (2026-09-07)** | `Run stage NN` drawn on the tablet, wired to a full run | **neither built nor routed** (C26); now an owner call in section 3.1 |
| S (2026-09-21) | woodland as a spatial area | declined by the ruling itself |
| X (2026-09-21) | tiles stay in the export menu | **built**: `menus.gd`'s EXPORT `Maps (image · tiles)` |
| AK (2026-09-23) | the general tool row moves to an always-visible top bar | **built**: `app.gd` `tool_palette_bar`, `_install_tool_palette_bar()` |
| BG (2026-09-27) | landslip and pinnacle landforms | **routed, not built**: `OUTSTANDING_WORK.md` section 2.13 ("Landslip and pinnacle landforms"); no `landslip` symbol in the crates |
| BQ (2026-09-28) | v2.72 and v2.73 do not exist | a record, no build |
| EP (2026-11-11) | the custom LLM is Qwen3.5-4B, after the rest of Cartalith | routed and deferred ("A Cartalith-specific small LLM") |
| FL (2026-11-16) | MS-3 tool palette, CQ-8's extras, the serial floor | routed (Phase 1, C2, C12, C13, C21) |

### Era by era

**August (the port's first weeks, to 2026-08-31).** Symbol sweep: 6 unresolved tokens in `STATUS.md`, 1 in
`OUTSTANDING_WORK.md`, 2 in `JS_SEMANTICS_AUDIT.md`; all are probes or examples deleted on purpose (`_peakaudit_*.rs`,
`_phonepaint_probe.tscn`, both described as deleted where they are cited) except one: `JS_SEMANTICS_AUDIT.md` names a
test, `js_to_fixed_matches_v8_on_every_rounding_branch`, that went when the helpers were folded into `cartalith-jsmath`
(`c3faa6b8`). **C27**: dated note naming the tests that cover `toFixed` today. No August ruling is lettered.

**September.** Rulings A to BU (73). Symbol sweep: 39 unresolved in `STATUS.md`, 16 in `OUTSTANDING_WORK.md`, 9 in
the rulings, 3 in `DECISIONS.md`, 2 in `GENERATION_PARAMETERS.md`; read one by one, nearly all are probes deleted with
their surface (MS-4, `9bae6952` 2026-11-08), mutant labels, census-script internals or external names
(`VK_ERROR_DEVICE_LOST`, `OnGodotMainLoopStarted`). Real findings:
- **C26** Ruling D (above).
- **C28** `GENERATION_PARAMETERS.md`'s `use_gpu` row still described the September GPU path (the shell turns it on at
  boot; seven GPU stages). Since Ruling EY the touch forms boot with it off, and since Ruling EZ item 3 only
  `GPU_PATH_KERNELS` (`plate_assignment`, `base_field_blur`) run on the GPU. Dated correction in the row.
- **C29** `STATUS.md`'s GPU layer integration table (GLI-1 to GLI-POOL, September) reads as if every kernel were
  still wired; GLI-P2 says the civ affordance kernels are "wired in `compute_civilisation` under `use_gpu`", and no
  `cartalith-godot` source calls them since `e4183543`. Dated correction above the table's total.
- **C30** `OUTSTANDING_WORK.md`'s DS-13 row cites `ToolFan.pill_reserve()`, gone with the pill (`c302a4ed`). Note added.
- **N8** `LARGE_ITEM_RULINGS.md` cites two test names that no longer exist
  (`build_water_bodies_largest_below_sea_component_is_ocean`, Ruling Q; `settlement_and_poi_are_not_named_by_the_unused_warning`,
  Ruling W). Rulings are a dated record and were not edited; the tests were renamed or folded later (appendix A).

**October.** Rulings BV to DW (54). Symbol sweep: 42 unresolved in `STATUS.md`, 9 in `OUTSTANDING_WORK.md`, 1 in
the rulings, 1 in `DECISIONS.md`, 1 in `SAVEFILE_COMPAT.md`; read one by one, they are mutant and probe-leg labels,
temporary accessors the entries themselves call scratch, and private helpers renamed since. Real findings:
- **C31** `OUTSTANDING_WORK.md`'s mountain relief row: `km_relief_taper` (cause 2, built 2026-10-17) is gone,
  replaced by Ruling FG's local-relief amplitude (`a673a1df`); and "Ridged orogen detail is still not built" was
  written before H1/H2 landed (`782ec8c7`, 2026-10-19). Dated notes on both.

**November.** Rulings DX to FL (41). Symbol sweep: 105 unresolved in `STATUS.md`, almost all mutant labels and probe
leg names in the 11-14..16 entries; 2 in `OUTSTANDING_WORK.md`. Real findings are Phase 1's C1-C23, plus:
- **C32** two rows whose titles say **CLOSED 2026-11-12** (the journey departure date, SP-2's residual; the saved
  journey's per-leg override fingerprints) still sat in section 2.3; moved verbatim to a new "Archive sweep 32".
- **C33** `OUTSTANDING_WORK.md`'s "the tablet still shows the PC layout" row listed as open the 320 landscape dock,
  the palette tokens and `scrPlanner`, all built or closed 2026-11-15 (`6cf7a27e`, `cc0e83b2`); dated note.

### `CLAUDE.md` routing table, checked whole (not edited)

Every document the table names exists (the three that are not at the root are in `reference/`, `cartalith-native/docs/`
or `design/`). Stale rows, adding to Phase 1's L1-L14:

| # | Row | What is stale |
|---|---|---|
| L15 | `cartalith-native/drafts/` | names `mobile-shell/` and `pc-shell/`; a third set, `stylus-input/` (four drafts, a README and `check_symbols.sh`), is not mentioned |
| L16 | `MOBILE_SHELL_MS1_SPEC.md` | "is applied by the code only below aspect 0.6": a status claim, false since Ruling EG's 900 dp line was built (`STATUS.md`, "Ruling EM" and the MS-1 entries) |
| L17 | (absent rows) | `CODEBASE_INVENTORY.md` and `DOCS_AUDIT_2026-11-16.md`, both new today |

## Counts (all phases)

- **Verified correct:** 18 claim groups (V1-V18); 10 716 of 11 476 code tokens the nine tracked documents name resolve
  in today's tree, and of the 760 that do not, 520 were explained by script and the other 240 read by hand (appendix A);
  165 of 168 lettered rulings built, routed, declined or record-only (D is the exception, now routed; FL and EP are
  routed and unbuilt); 137 of 137 parameters documented; 48 of 48 archive entries named in code.
- **Corrected:** **33** (C1-C33) in 17 documents: `OUTSTANDING_WORK.md`, `cartalith-native/docs/STATUS.md`,
  `PERFORMANCE_BENCHMARKS.md`, `WORLD_SCALE_TERRAIN_PLAN.md`, `GUI_PLANS_AUDIT_2026-11-16.md`,
  `CUSTOM_QUICK_BAR_SCOPE.md`, `CITY_MAP_LOOK_SCOPE.md`, `CODEBASE_REFACTOR_PROPOSAL.md`, `RIVER_REALISM_SCOPE.md`,
  `DECISIONS.md`, `ARCHITECTURE.md`, `README.md`, `TABLET_UI_SPEC.md`, `GENERATION_PARAMETERS.md`,
  `JS_SEMANTICS_AUDIT.md`, plus the two new files.
- **Found, not changed:** **8** (N1-N8): document placement, code comments and dead code (reported, not code-edited),
  and ruling-record test names.
- **Still open, correctly routed:** Ruling FL items 1-2; Ruling D (new owner call); FK leftovers (i)-(v); FI (iii),
  (v); FF (iii)-(v); FE items 3 and 4; FH's save prompt and disk cap behind the ZR decision; BG landslips; EP; the
  `ac1_static_sources` test; the Pad's `RAYON_NUM_THREADS=1` column; everything else in `OUTSTANDING_WORK.md` sections
  1-4 that this audit did not contradict.
- **Stale `CLAUDE.md` rows:** **3 wrong** (L1 `CITY_MAP_LOOK_SCOPE.md`, L15 `cartalith-native/drafts/`, L16
  `MOBILE_SHELL_MS1_SPEC.md`) and **15 missing** (L2-L14, L17).

### Limits

The symbol sweep proves a name exists, not that it still does what the prose says; that was checked only by reading
(method 3, every phase-1 row, and the rows the sweep flagged). `STATUS.md`'s 9 900 lines were swept for names and
read where flagged, not entry by entry. The GUI plans' 56 DONE verdicts and the unlettered August/early-September
rulings were taken from their own audits and rows rather than re-opened one by one.


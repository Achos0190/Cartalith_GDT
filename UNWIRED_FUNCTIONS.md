# UNWIRED_FUNCTIONS.md — presented but not wired

**What this is:** a per-control register answering one question `STATUS.md`
does not ask — *the product draws this control; is anything behind it?* Each
row names the symbol that was opened, the reason a user is shown, and how the
row closed or why it stays open. **What it is not:** a live count, a schedule
or a status ledger. It is a **dated cut**: its counts hold for the date beside
them. `cartalith-native/docs/STATUS.md` is authoritative for status; what is
*ruled* for these rows is in `LARGE_ITEM_RULINGS.md`; what is *queued* is in
`OUTSTANDING_WORK.md`, which counts this whole register as one row.

Its neighbours: `GUI_GAP_REGISTER.md` is its ancestor (every disconnected
control, classified by whether a design exists, with comparable-application
research), and `DCC_CONTROL_INDEX.md` indexes the *design's* controls against
the engine. This register is the successor of the first one's open half.

Owner, 2026-08-30: *"Make sure all menu's are created and all presented
functions are wired or the ones that have no code behind them get listed in a
table with a proper proposal (inferred from the menu name and highest probable
explanation/design spec of the named function.)"*

**The cuts.** Last full cut: **2026-09-03** (second pass), against `HEAD`
`0bba2f9` plus the working tree. The open rows, the owner questions and the
dangerous class were re-opened at their symbols on **2026-09-21** and again on
**2026-09-23**. Rows that closed at earlier cuts are not reproduced here; they
are in history — `ac483ad` (2026-08-30, the first table), `5543ef3` (2026-08-31,
the full per-row table rewritten against the new shell), `0bba2f9` (2026-09-03,
first pass). A code comment that cites this file's row by name — *"Atlas cache
`Size cap · GB`"*, *"No storage-full handling"*, *"No content descriptions, no
dynamic type"*, *"the tablet interior walk"* — resolves in one of those.

---

## 1. Method — what this register learned about itself

**The code wins; a row that disagrees with it is a defect.** That is this
document's own primary failure mode, and it has been caught repeatedly:
12 of 44 rows wrong on the first re-cut (2026-08-30); **14 of 17** open Large
rows already built at the first 2026-09-03 pass; a "correction" that reversed
an accurate note and a provenance claim that made a defect look older and less
culpable, both at the second 2026-09-03 pass; two rows found already built on
2026-09-21 (the paint preview patch, the clipboard) only when agents were sent
to build them; and, found on 2026-09-23, a Medium row re-asserted on 2026-09-21
as "blocked on owner question 2" fifteen days after ruling 21 answered it and
the field shipped, plus six owner questions (2, 3, 6, 8, 9, 10) listed as open
that had all been ruled on 2026-09-05 or 2026-09-06.
**A wrong correction and a false exoneration are both worse than the original
defect, because each looks freshly checked.**

- **Re-open every row at its symbol, never at its line.** Measured on
  2026-09-03 while two other lanes edited the same files: citations moved
  **148** lines (`civilization_workspace.gd`) and **241** lines (`render.rs`)
  inside one session, while untouched files held exactly. Grep the quoted
  string or the symbol; do not jump to a number.
- **A capability landing turns the sentence explaining its absence into a lie,
  and nothing in the build catches it.** `tools/audit_wiring.py` cannot — the
  binding *is* called; only the prose is wrong. `cargo test --workspace` cannot
  — the strings are GDScript literals and Rust hint text that no test asserts
  on. All nine §4 entries stood through a green workspace suite.
- **Sweep the old files, not just each wave's diff.** Two of the nine had been
  false for a fortnight: b-7 was written `7f5e54c` (2026-08-18) and falsified
  by `b7a46a7` (2026-08-23) — false for 16 days; b-8 was written `595582d`
  (2026-08-19) and falsified the **next day** by `0de790a` — false for 14.
  Watching closures catches young defects and neither of those.
- **The productive query** names a crate or a Rust path and asserts an
  absence, run from `cartalith-native/godot-project/`:

  ```
  grep -rn "cartalith[-_]" --include=*.gd shell/ | grep -iE "no |not |never |missing|absent"
  ```

  40 hits at 2026-09-03, three of them false. It is a **floor, not a census**:
  a sentence saying *"there is no way to …"* without naming a symbol is
  invisible to it.

## 2. Rows open at the last re-check (2026-09-23)

Two, and neither is actionable.

| Row | Tier | Checked at | Why it stays open |
|---|---|---|---|
| **The 3D viewport** — `3D viewport defaults` and `Anti-aliasing · anisotropy` are `_todo` rows in `menus.gd` | Large | no `Camera3D` or `MeshInstance3D` in `shell/` or any `.tscn`; the only hits are the vendored `addons/godot_ai/` plugin, which is not this shell | **Deferred by ruling, research first** (`LARGE_ITEM_RULINGS.md`, "Deferred, with research first"), then parked; the research is `cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md`. Both `_todo` reasons ("there is no 3D viewport") are true |
| **Bounded sculpt preview** — the sculpt half of "Previews re-upload the whole texture" | Medium | `build_sculpt_preview_texture`'s doc | **Declined at its own doc:** a bounded preview needs `render.rs`'s AO, wetness and sea passes reworked over a caller-supplied window, and `golden_parity_render.rs` pins that file bit for bit. Routed as `STATUS.md` SL-1 (`SCULPT_LIVE_SCOPE.md` L1) |

## 3. Closed rows

Tiers at the last cut: **Trivial** 17 of 17 closed; **Small** 25 of 25;
**Medium** 16 of 17; **Large** 17 of 18 (all eighteen ruled 2026-08-31). Rows
closed from 2026-09-01 on are listed; earlier closures are in the history
named above. Each was re-opened at the symbols given.

### Small and Medium

| Row | Closed by | Symbols, and what to know |
|---|---|---|
| `label_glyph_layout` re-implemented in GDScript (Small) | Ruling AG, `a52ddcf`, 2026-09-23 | The label hit box and handles now size off `map_overlay.gd`'s own font model — `label_bridge::shell_label_box`, fed `ViewportHost.label_px_per_cell()` — instead of `labels::label_font_size`, which is untouched and still golden-pinned. The ruling chose the **newer** model (`fd9de7c`, 2026-09-01) over the engine's (2026-08-18), so the original proposal — route drawing through `label_glyph_layout` — would have reverted to the older one and is withdrawn. Drawing stays in GDScript; the `engine_bridge.gd` forwarder for `label_glyph_layout` has no caller, and whether the engine's `arc_label_layout` follows the newer model is left to its builder by the ruling |
| `statusMid` composite (Medium) | ruling 21, 2026-09-06; built `d647dd3` the same day | `repaint NN ms` is `_refresh_map()`'s **wall time** — the number a user can act on and the one measurable without a rendering-server hook. `app.gd::_refresh_status_mid()`; the bracket is `_repaint_open`/`_repaint_close`, and `_repaintbracket_probe.gd` drives both branches (a sentinel omits the field; a real `0.0` still draws `repaint 0 ms`) |
| The right dock does not follow the armed tool (`rdExtraMode()`) (Medium) | owner ruling 2026-09-03: *"Selection wins; the tool appends a section."* | `right_dock.gd::_append_tool()` / `_tool_section()`, over `TOOL_*` **section ids** derived from `app.armed_tool` and the domain on every rebuild — never stored, and never `CTX_*` constants, which made arming a tool *replace* the selection (measured: arming Territory over a selected settlement lost the settlement). The file carries a warning against renaming them. Extended to the Journey planner 2026-09-04 |
| Previews re-upload the whole texture — paint half (Medium) | `45df3019`, 2026-09-04; found already built 2026-09-21 | `EngineBridge.build_paint_preview_patch()` → `ViewportHost.set_preview_patch()` (`Image.blit_rect` + `ImageTexture.update()`, so erase replaces pixels) from `world_workspace.gd::_paint_show_preview()`, falling back to a full re-upload only on an explicit `false` (no base, format mismatch, resize, out-of-bounds window). Measured 2026-09-21 at 2048²: **14.889 ms → 1.907 ms** median per dab over 20 dabs. The sculpt half is §2 |

### Large

| Row | Closed | Symbols, and what to know |
|---|---|---|
| CARTO ▸ Labels: the whole panel | `45b368d`/`0bba2f9` | `labels.rs` `LabelClass`, `LABEL_CLASSES`, `LabelTypography`; `generate_labels` emits per-class placements in drawing order; `label_class_table()` and `labels_generated_counts()` feed `cartography_workspace.gd::_build_label_classes`. The GDScript class array is the fallback for an older cdylib, not the authority, and a Rust test pins the two identical |
| Label collision culling | same | `label_cull_rect`, `LabelCullMetrics`; suppression by rank, hand-placed labels never suppressed. The one number the engine cannot know — mean glyph advance as a fraction of font size — is measured off the shell's font by `_label_advance_ratio()` and sent with every run, and the panel says the boxes are *estimated* |
| CARTO ▸ Icons: generated placement | same | `icon_bridge/generate.rs` `IconEditor::generate`, `PlacementFamily` (`Places`/`Poi`/`Trees`/**`SeaMarks`** — owner question 4's answer), `CoastSnap`, `sea_mark_gap`; four **disjoint** rejection counters, so a run that placed nothing says which wall it hit. `ICON_FAMILIES` in `cartography_workspace.gd` is the *manual* icon vocabulary (three entries), a different list from `PlacementFamily` |
| The manual-icon tool | same (`UNIFIED_TOOL_PLAN.md` milestone E) | `icon_bridge/brush.rs`; `icon_brush_set`/`icon_brush`/`icon_brush_stamp`. The brush settings are re-sent on every re-arm because `absorb()` rebuilds the engine's editor on each generate — without that, a brush set before a regenerate silently reverts while the UI still shows the user's numbers |
| The river entity | same | `get_rivers(min_order)`, `river_at(gx, gy, radius_cells, min_order)` (separate, so a click does not trace the network twice), `river_dict`; `right_dock.gd::_on_map_clicked_river`, `show_river`. The remaining disabled Actions carry narrower, re-derived reasons, and the third was **removed rather than re-labelled with a new pretext** |
| Civilisation authoring operations | same | `civ_populate`, `civ_clear_places`, `civ_clear_territory`, `civ_clear_ways`, plus the civ `PARAMS` group, derived from `group == "civ"` in `params.rs` rather than a second key list. Each destructive operation confirms and says what it destroys |
| Settlement diagnostics overlay | same | `urban_bridge.rs::settlement_diagnostics`; `civilization_workspace.gd::_build_settlement_diagnostics`. Its two absent-value cases are dashed with their reasons rather than defaulted |
| Colour management | same | `render.rs::apply_color_space`, the last stage of `build_color_texture`; `list_color_spaces`/`get_color_space`/`set_color_space`; CARTO ▸ COLOURS (`render_workspace.gd::_build_color_management`). **Two devices, not the spec's three:** sRGB and Display P3 are display devices, and linear is a *working* space, unshippable at 8 bits. The menu row is a `_signpost`, not a `_todo`, so `command_index.gd` does not count a shipped feature as missing |
| Rebindable keyboard shortcuts | same | `DccSettings.shortcut_binding`/`set_shortcut_binding`/`clear_shortcut_binding` (per context, `SHORTCUT_CONTEXT_MENU`), applied over the menu accelerators at build; `shortcuts_dialog.gd` — one class opened two ways (`open_editable()` for Preferences), because `GUI_GAP_REGISTER.md` §7.9 calls two dialogs a bug. The table is walked off the live `MenuBar`, so it cannot disagree with the app; accelerators with no menu row are declared in `UNLISTED` |
| `Units` (km / mi / nmi) | same | `DccUnits` (`to_unit`, `suffix`, `format*`); storage stays km and only what a user reads is converted. Nautical miles are the third unit the ruling added |
| CPU worker threads | same | `cartalith-engine`'s `rayon::ThreadPoolBuilder`; `cpu_logical_core_count`, `cpu_thread_count_active`, `set_cpu_thread_count`. Rayon's global pool builds once per process, before a menu can open, so the readout prints the **measured running count beside the stored preference** ("12 chosen, 16 running, next start") rather than implying a change took effect |
| `Report an issue` | same | `diagnostic_report.gd` `DiagnosticReport.write()` — a local diagnostic dump, no endpoint; it reuses `GenInfoDialog`'s generation-info statics, `EngineBridge.missing_bindings()` and `project_format_version()` rather than re-implementing them, and adds GPU state and a last-error retention (`note_error()`/`last_error()`) |
| Landmark funnel: crowding and rejected candidates | same | `LandmarkRun::rejects`, `LandmarkReject` (score and position); `landmark_rejects()` drives two chips and a diagnostic map layer off one pull |
| Saved measurements + CSV | 2026-09-03 | `right_dock.gd` `measurements_document`/`restore_measurements_document`; the caller-owned slot `annotations/measurements.json` (`cartalith-io` `project.rs`) — riding the same document dictionary as the engine's slots, not a second mechanism |
| Paint brush falloff | 2026-09-01 | `DECISIONS.md` §7k; `paint_set_brush`'s doc now reads "consumed since §7k" |
| `Cut` · `Copy` · `Paste` · `Select all` | `686cd2a`, 2026-09-03; found already built 2026-09-21 | `menus.gd` `_cut_selection`/`_copy_selection`/`_paste_clipboard`/`_select_all`, scoped to the active domain. A session-lived clipboard holding the engine's own `icon_get()`/`label_get()` records, so a paste after a world change re-creates entities rather than referencing stale indices; `PASTE_OFFSET_CELLS := 4.0`, clamped into the grid; Cut deletes highest index first. **Icons and labels only, by design**: a sculpt stamp cannot be read back and settlements have no selection set. `_clipboard_probe.gd`'s icon leg now runs against `cartalith-assets/tests/fixtures/reference_pack.zip` |
| `Region ▸ New world from selection` | `76cbf8b`, 2026-09-20 | `ops_bridge.rs::region_new_world` from the File menu, behind a mandatory confirm naming everything a resample destroys; gated on the binding, a world and a real marquee |

**Two Trivial closure notes from the 2026-09-01 cut, re-examined.** The State
religion field (`right_dock.gd::_build_faction`) prints
`rel.capitalize()` and dashes only a *missing* key — `"none"` is a real answer
from `cartalith-civ`'s vocabulary, as the comment beside it says. The first
2026-09-03 pass wrongly "corrected" that note to the opposite and was retracted
by the second. And `pack.rs`'s painted-layer decline closed when
`LoadedPack::biomes`/`::terrains` began decoding to `render::GroundTile`
(`ASSET_LIBRARY_SCOPE.md` milestone 7).

## 4. The dangerous class — a disabled row whose stated reason is false

A disabled row with an honest tooltip costs a user nothing; these cost trust.
**All nine are fixed** (re-verified at the corrected strings 2026-09-21; b-8 by
its absence, since `performance_window.gd` was folded away under ruling 19).
Kept as history so the class and its method are not rediscovered. The model to
fix against is `render_workspace.gd`'s own note on the render stages it lacks,
which re-derived what was left rather than trimming the old sentence.

| # | Where (symbol) | The false claim | The truth |
|---|---|---|---|
| b-1 | `infrastructure_workspace.gd::rivers_note()`, shown in WORLD ▸ Hydrology — the file declares it the single owner of the disclosure | *"there is no get_rivers() and no way to select one"* | `get_rivers`/`river_at` exist and selection is wired; the right dock's own source had already said so |
| b-2 | `cartography_workspace.gd`, the river-filter note | the river network *"never crosses the GDExtension boundary"* | it crosses as entities; the true gap is per-filter drawing |
| b-3 | `layers_popover.gd` `"popdensity"` **and** `sample_bridge.rs`'s `LAYER_GROUPS` hint it transcribes | *"No regional population-density estimator exists"* | `cartalith_civ::estimate_regional_density_km2` exists, golden-tested, run by `civ_regional_population()` — which integrates the field away to a total. Both files had to change together |
| b-4 | `place_editor_window.gd`, the overrides note | the overrides are unread because urban milestones 8-17 are unported | both readers are ported (`um_infer_age`, `military::um_wall_spec`); the conclusion was true for a different reason, and has since stopped being true — both overrides now reach the drawn town |
| b-5 | `tool_bar.gd`, the measure bar's hint **and its count label** | *"no such preference exists in this shell yet, so every reading is km"*; "4 canvas options unbuilt" | `Preferences ▸ Units` is live (km/mi/nmi) and the readouts route through `DccUnits`; three options, and the count lives in the label, so both had to move |
| b-6 | `cartography_workspace.gd`, the declutter-budget note | *"Label and icon collision is not resolved anywhere"* | resolved by `label_cull_rect` and the placement pass's *avoid label boxes* rule. `git log -S` shows the note was **edited on the day the culler landed** with the false sentence left standing above the new true one — read past, not overlooked |
| b-7 | `world_workspace.gd`, the `STAGES` row for **08 Climate** | *"Seasons and Köppen-Geiger classification are not ported"* | `cartalith-climate/src/koppen.rs`, golden-tested, and drawable from Layers (`sample_bridge.rs` registers `"koppen"`) — while the Erosion row directly above had just been audited |
| b-8 | `performance_window.gd` (since removed) | *"no per-device enumeration exists in cartalith-gpu"* | `cartalith_gpu::enumerate_devices`, bound as `WorldGen::gpu_enumerate_devices`; `menus.gd`'s build-conditional wording was the true one |
| b-9 | `civilization_workspace.gd::_build_politics_gaps()` | no faction relation exists *"because cartalith-civ has no such relation to record at any year"* | `relations.rs` (CV-26) builds that edge (`civ_faction_relations`), drawn by three surfaces, one of them in the same file (`_build_relationships()`); the true gap is that nothing snapshots a relation **at a year** |

**Stale source comments** — not user-visible, same class, all six fixed by
2026-09-21 (`e462ed9` and an earlier 2026-09-20 pass): two in
`cartography_workspace.gd` that still called the culler future work, three in
`map_overlay.gd`'s label constants block pointing `grep` at the wrong
`labels.rs` and `engine_bridge.gd` lines — **a comment written to stop a
citation drifting, drifted in every citation it carried** — and
`paint_set_brush`'s "never consumed". The same block's citations have drifted
again since; its own note says the number moves with every edit above it.

Drawn-enabled-but-inert rows (the 2026-09-01 cut's class (a)) and
true-reason-misleading-presentation rows (class (c)) stayed at zero.

## 5. Not gaps — recorded so they are not re-listed

**Build-conditional `_todo`s.** Rows that are disabled only against a cdylib
predating their binding, or until a condition changes: the multi-GPU device and
mode rows, the VRAM budget and fallback, the landmark vocabulary, *Try the GPU
again*, atlas export/import/cap, *Forget layout* with none saved,
*Documentation* with no `res://` beside the build, Redo on an older cdylib, and
three in `journey_planner_view.gd`. **Any table that lists these flat is
wrong** — each enables itself the moment the library is rebuilt or the
condition changes. (The CPU-threads `_todo` is the fallback for an older
cdylib; the row itself is built, §3.)

**Presented, permanently inert, and correct.** `Follow system`;
`Alternate frames` / `Reduce working res`, enforced engine-side; the Working
set and VRAM estimate readouts, indexed as readouts.

**Deliberate omissions, argued in code.** CIVIL POI (no `_civDropPOI` port —
`STRANDED_TOOLS.md` §3), WORLD palette Sculpt/Freehand, `Data ▸ Conversion`
(removed by the owner 2026-08-20).

**Superseded twins — the sibling is the production path, and nothing should
call these.** Cited by name from their own doc comments; each checked
2026-09-23 to have no caller outside `#[cfg(test)]` code:
`cartalith_civ::place_settlements` (the pre-snap reference kept beside the
water-edge-snap fix, so the snap's effect is
a diff between two live functions), `timeline::civ_resync_next_tid` (the
timeline-aware sibling is production), `AtlasStore::chunk_len` (sizing goes
through `key_bytes`, which stats both files of the pair), and
`LandmarkClass::badge()` (the shell derives the same strings, deliberately, so
a new class gets a badge rather than a blank). The fuller 2026-08-31 list is in
`5543ef3`; it was not re-walked, and several of its GPU wrappers and
`init_gpu_f64` have since been deleted.

**Declines verified true** — found while sweeping for false ones, and recorded
because a sweep that reports only hits reads as though everything it touched was
wrong. Checked at the 2026-09-03 passes unless marked:

- Province-level assignment "has no binding" (`civilization_workspace.gd`):
  `get_provinces()` carries name, faction and `capital_settlement_index`, and
  no settlement's province crosses the boundary.
- A year diff's old settlement data "no `#[func]` exposes yet":
  `civ_year_diff()` returns three `PackedInt64Array`s of tids and nothing else.
- The right dock's "no `#[func]` evaluates the cost surface pointwise" and "no
  row-slice `#[func]`".
- `place_editor_window.gd`: `assign_territory` runs inside `generate()` and no
  `#[func]` re-runs it.
- The asset library's tag chip: "removing one has no binding yet" — still no
  tag-removal function (re-checked 2026-09-23).
- *Refine detail*'s "nothing reads the cache at draw time yet" — no atlas
  lookup on the draw path (`menus.gd` `REFINE_TOOLTIP`, still so worded
  2026-09-23).
- `render_workspace.gd`: "Godot's compatibility renderer does no colour
  management" — true and correctly scoped to the overlay `Control`s and chrome,
  not the engine raster that `apply_color_space` re-encodes.
- `layers_popover.gd`'s `oro`, `velo` and `siteprofile` sentences (the
  `popdensity` sentence between them was b-3 — four sentences from one
  constant, three right and one wrong, which is why this sweep opens every
  symbol).
- `render_workspace.gd`'s *"the engine has no such stage"* note for minor
  channels, season blend and two SDF legs — re-checked, not inherited.
- `place_editor_window.gd`: the reference's `_civApplyFoodShedCeilings` is not
  ported; `civ_food_shed` computes the ceiling, nothing applies it.
- The two `needs a decision` notes (CIVIL politics, INFRA), each saying what is
  live in the next sentence; `relations.rs` agrees in its own words.
- `world_workspace.gd`'s Geoid/tides `STAGES` row, which separates a missing
  sub-system from a present one enabled by another toggle.
- `world_workspace.gd`'s Orogeny `STAGES` row — fold intensity, trench depth
  and fault blocks have no dials. **Corrected 2026-09-24:** the 2026-09-23 pass
  recorded the hardcoded `OrogenyParams` literal (`fold_k: 0.16`,
  `trench_k: 1.0`, `fault_block_k: 0.0`) as "the reference's own defaults"; it
  was not (the reference derives them from World Structure). Since Ruling AS
  `generate_terrain_inner` uses that derivation (`world_structure_orogeny_ks`).

**Stopped being true: GeoJSON import.** `data_manager_window.gd`'s *"No
GeoJSON import path exists; `cartalith-engine::geojson` is write-only"* was
true at the 2026-09-03 pass. Since `d79d776` (2026-09-21, Ruling V) the Data
manager imports a GeoJSON document through `apply_geojson_document`
(`geojson_bridge.rs` → `geojson_apply.rs`), placing settlement points and
rasterising territory polygons; the parser is `cartalith-io`'s, and
`cartalith-engine`'s own module is still write-only.

## 6. Owner questions — all ten answered

| # | Question | Answer |
|---|---|---|
| 1 | Does `rdExtraMode()` replace the right dock's selection contexts, or sit beside them? | **Beside** — "Selection wins; the tool appends a section" (2026-09-03); built (§3) |
| 2 | What should `statusMid`'s `repaint NN ms` measure? | **`_refresh_map()` wall time** — ruling 21, 2026-09-06; built (§3) |
| 3 | Should the WORLD left-dock A/B switch come back? | **Yes, Option B** — ruling 6 of 2026-09-05 (evening); built as `dcc_shell.gd`'s §2.3 `ldSwitch` pill (`PIPELINE` \| `SCULPT`, the words of the approved round-2 canvas) |
| 4 | How do the design's four icon placement families map onto three? | **They don't — a fourth family**, sea marks (2026-08-31); built (§3) |
| 5 | Paint falloff: bind it, or delete the sliders? | **Bind** (2026-08-31); built 2026-09-01 |
| 6 | Should a committed sculpt stamp re-evaluate when sea level moves? | **Re-read live, matching the reference** — ruling 17, 2026-09-06, against the recommendation. `SculptStamp::with_sea_level` is the explicit stand-in; for whether the build is done, see `STATUS.md` / `OUTSTANDING_WORK.md` |
| 7 | Are the four unwritten save slots deliberate? | **Answered and executed**, plus a fifth slot (`annotations/measurements.json`) |
| 8 | Is `init_gpu_f64` kept or deleted? | **Deleted** — ruling 22, 2026-09-06 |
| 9 | Is the phone app bar's `☰`/`▤` pair stale? | **Stale** — ruling 20: `[world pill] · ⌕ · ⋮`; built as `dcc_shell.gd::_build_phone_app_bar()` |
| 10 | `--good` and `--accH`? | **Kept** — ruling 22: declared-and-unused in the prototype too, so declaring them is fidelity. `dcc_theme.gd` records the reason beside each; `good` also has a consumer here (`faction_roster_window.gd::_build_terrain_fit()`) |

**The 2026-09-03 cut's "left undetermined" list** (light-theme reading of the
CARTO panels, the phone measure strip / label bar / way card, the 44 vs 48 dp
sweep, whether `sculpt_stroke_point` can reject an appended point, landscape
composition, stale `PARITY_AUDIT.md` section numbers in `_todo` reasons) was
answered on 2026-09-13; the answers are in `OUTSTANDING_WORK.md`'s archived
row for it.

## 7. The reverse census: `#[func]`s no `.gd` file calls (2026-09-08)

Every section above asks *"the product draws this control — is anything behind
it?"* This asks the reverse: *"the engine exposes this binding — does anything
in `godot-project/` call it by name?"* The row had shipped a wrong count twice
("27 without a forwarder, 12 unreachable", then "six"), so the count was
re-derived from scratch and only then checked against `engine_bridge.gd`'s own
register, "Bindings with no forwarder, and why" (dated 2026-09-06), which
matched.

**Method.** 468 `#[func]` names under `cartalith-godot/src/` at that date,
matched by word boundary against every project `.gd` file, excluding the
vendored `addons/godot_ai/` plugin (untracked, and not this shell). Every name
appears somewhere in raw `.gd` text; the question is whether as a call. The
first attempt returned 11 zero-hit names, three wrong, because comments were
stripped **before** strings and a `#` inside a triple-quoted fixture string
orphaned its closing quote — one file's stripped text fell from 299 lines to
62. Stripping triple-quoted strings, then single-line strings, then comments
gives **eight** (`MISTAKES.md` carries the rule).

| Function | Verdict |
|---|---|
| `ping` (`WalkingSkeleton`) | **A diagnostic seam**, by its own doc: a `godot --headless -s` smoke check that the library loads at all, before a `WorldGen` exists |
| `asset_library_document_json` | **Reached from Rust**: `project_engine_built_documents()` calls it, and every project save goes through that |
| `export_snapshot_png` | **Reached from Rust**: `vault_snapshot()` calls it for every map snapshot |
| `arc_label_line_width` | **Dead as a binding, deliberately; the formula is not.** `map_overlay.gd` hand-duplicates `max(1, size_px × 0.16)` to avoid an FFI call per label per frame, and has since gained a zero-halo branch the engine function lacks, so routing through the binding would change what draws. Named in three files' doc comments |
| `project_read_document` | **Dead by ruling** (2026-08-26, `PARITY_AUDIT.md` §23): the one capability `project_open` cannot offer — read one document and keep the world — kept for a command that does not exist |
| `geojson_inspect` | **Uncalled.** Staged as the surface a GeoJSON import would call; the import that landed (§5) is built on `apply_geojson_document` instead, so this summariser still has no caller |
| `vault_landmark_entity_id` | **Staged ahead of its UI** (ruling 13, 2026-09-06): no shell path opens a landmark as a vault entity; every `open_vault()` call passes settlement, faction, province or culture |
| `labels_clear_generated` | **Deliberately unwired**: the generated pass only re-runs from `generation_finished`, `world_loaded` or a class-dial release, so a "clear" that dropped it would strand the user — see `label_clear_all`'s doc |

No forwarder was added and nothing was deleted: two have a live caller one call
frame up, and the other six are each named by symbol in another file, so
removing any would edit that file too.

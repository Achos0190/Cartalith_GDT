# S4_INVENTORY.md — MS-1 stage S4.0: what a class swap touches

**What this is.** The inventory `MOBILE_SHELL_MS1_SPEC.md` §3.5 requires before any S4 code, under `MISTAKES.md`'s
"Delete a surface" row: a swap between the phone and the desktop/tablet composition deletes one composition and
builds another, so every field, connection, registry, window and piece of "the class never changes" code gets a
stated action and a reason. Written 2026-11-07 against `bd784584`, read at the symbols named (never line numbers).
Documentation only: no `.gd` file was changed. How far S4 has got is `cartalith-native/docs/STATUS.md`'s answer.

**Actions used.** *survives*: the object lives through the swap unchanged. *rebuilt*: a new one is made by the mount
path. *reset*: the object stays but its value or its contents are cleared and re-derived. *freed*: the object goes
with the old composition and nothing replaces it in the new class. *re-presented*: an open window stays open and is
laid out again for the new class (Ruling DY, MQ-5).

**Method and its limit.** Fields are the `var` declarations of `shell/dcc_shell.gd` and `shell/app.gd` (134 and
100). The setter of each was found by a call-graph script over the function bodies and then **checked by hand at the
builder** (the script over-reaches: it counts a runtime handler as reachable from a builder, so its builder column
was discarded and rewritten from the builders' own code). Window rows (§5) come from each window's `_phone` /
`is_phone()` / `is_tablet()` branches and its member variables; a window's *state* column is read from its fields
and is the list its own S4 batch must confirm, not a finished spec.

## 0. Ruling EG changes what S4 is for

Ruling EG (owner, 2026-11-07, `LARGE_ITEM_RULINGS.md`): *"On 11" tablet we might just use the phone gui"*, sized
*"Agreed on the density to prevent the blow up"*. Below 900 dp of shortest width, **at every aspect ratio**, a touch
window gets the phone shell, sized by `_phone_scale = min(density x ui_scale, short_px / 412)`, never short / 412
alone. Consequences for S4 and MS-7, read against the code:

1. **The gate changes, and S4 inherits the swap's frequency.** Today `DccTheme.classify()` applies the 900 dp line
   only below aspect 0.6 (`_PHONE_ASPECT_MAX` gates it; the spec §2.2). Ruling EG removes the aspect gate for the
   900 dp half. Against §2.2's fourteen windows that moves the foldable (673 x 841), `TABLET 800`, 889 x 1422 and the
   11-inch class (1194 x 834) from tablet to phone. B1's literal table, and B2/B10's baselines for those sizes,
   change with it. That is S1's table row, not S4, but S4's probe sizes (B5) must be chosen against the new line.
2. **Rotation alone does not cross a shortest-width line.** `min(w, h)` is the same in both orientations, so an
   11-inch tablet (834 dp short side) rotating stays a phone, and a 1 366 x 1 024 one (1 024 dp) stays a tablet. What
   crosses the 900 dp line on a tablet is a change of the *window*: split screen, a freeform or desktop-mode window,
   a foldable's fold, Android's Display size or font setting (which moves the density), and the UI scale (S5; a 125 %
   scale takes a 1 080 dp-short tablet to 864 dp). Ruling EG's own sentence names rotation; that holds only for a rule
   written on *width* (Ruling DY MQ-1's "under 600 dp wide", which MS-7 lands). **The two rulings are not
   obviously one rule, and the build should not pick silently** (see §10).
3. **Class swaps become routine on one device**, so the swap is a hot path, not an edge case: B5's ten round trips
   are the least that should hold, and `S4`'s cost versus a restart (B5's timing) matters more than the spec assumed.
   Per-class state that is *kept* (WI-04 dock widths, the sheet detent) matters more too: a user who splits the screen
   back and forth should find the same layout each time.
4. **The phone shell on a 900 dp-wide landscape window.** The phone composition's landscape form (a left rail, the tool
   sheet docked right: `_apply_phone_orientation()`) is then the layout at 834 to 899 dp of short side and up to about
   1 200 dp of long side. Nothing in S4 builds that, but the swap must not assume a phone is narrow: `phone_present()`
   sizes windows to `host.get_viewport_rect().size`, which is right at any width, whereas the `_phone_land_sheet_width()`
   and `_phone_nav_reserve()` constants were drawn for 360 to 480 dp and are an MS-2 to MS-6 question.
5. **MS-7 (the docking rule) is measured above 900 dp only** (Ruling EG item 3), so the 346 dp-map finding of B3 leaves
   MS-7's scope as written: a tablet that wide has no map problem to solve, and the phone shell covers the narrow one.
   MS-7 is still needed for 900 to about 1 200 dp (two 400 dp docks leave 100 to 400 dp of map at density 1), which is
   where the **tablet to tablet** width classes begin.
6. **The 900 dp line becomes a hysteresis site that real users will oscillate across** (a split-screen divider dragged
   by hand). §3.1's 16 dp band and B5's "one second of oscillation between 896 and 904 dp causes at most one swap" are
   then load-bearing, not hygiene; S4 should be built on the debounce (150 ms) and the band as written, and the
   quiescence wait (§8) must not turn a drag into a stuck "waiting" status.
7. **Density is now the phone's only unit on a big screen**: the phone window multiplier `phone_scale()` feeds
   `phone_present()` as the window's `content_scale_factor` (`DccWidgets.phone_present`). A tablet-sized phone window
   at density 2 is `phone_scale` 2.0, so every window is content-scaled at 2.0 inside a 1 194 dp screen. That is correct
   under Ruling EG item 2 and is what the re-present path (§5) reads: it must take `phone_scale()` at present time, not
   cache it at `setup()`.

## 1. Summary

| Inventory | Entries | survives | rebuilt | reset | freed | re-presented |
|---|---|---|---|---|---|---|
| `dcc_shell.gd` fields (§2.1) | 134 (80 Node-typed, 17 `Dictionary`, 1 `Array[Control]`, 1 `Tween`, 1 untyped, 34 inferred scalars) | 24 | 28 | 19 | 58 | 5 |
| `app.gd` fields (§2.2) | 100 | 36 | 9 | 32 | 0 | 23 |
| Builder callees outside the two files (§2.3) | 11 objects | 1 | 7 | 1 | 2 | 0 |
| Connections made in the build (§3) | 15 spec-counted, in 10 files, plus 9 more listed | see §3 | | | | |
| Static registries and class statics (§4) | 9 entries (2 registries in `DccTheme`) | 5 | 0 | 4 | 0 | 0 |
| Windows that call `phone_window()` / `phone_present()` (§5) | 28 files: 24 windows in the table (22 long-lived or lazy, 2 phone-only), plus the slicer sub-window and the desktop search dialog, and 14 transient dialog sites | | | | | 26 re-presented; 14 transient sites wait (§5.3) |
| Class-never-changes code (§6) | 21 sites | | | | | |
| Survivors (§7) | 11 | 11 | | | | |
| Quiescence sources (§8) | 14 | | | | | |

The **per-file** distribution of what must change is in §9; the biggest single risk is §2.3 (the three workspace panels
and `right_dock_ctrl` bake the class into their rows and into `phone_fit()`'s irreversible edits), then §5.2 (the
phone helpers cannot be undone), then the lambda connections in §3.

Re-measured counts (2026-11-07, `bd784584`, the spec's `cnt` commands): `phone_present(` 41 call lines in 28 files;
`phone_fit(` 58 in 28; `tablet_fit(` 2 in 1; `phone_window(` in 31 files; `is_phone()` 71 lines in 28 files, `is_tablet()`
119 in 31, `is_touch()` 32 in 14; `role_px(` 206 lines; `phone_scale()` 25 in 8 files; `_pscale(` 142, `_ptap(` 26, `_pfont(` 28.
The spec's "at least 78 Node-typed fields" holds: 80 by this file's declared types, and its 16 `Dictionary` fields is 17.

## 2. Fields and objects the builders assign

### 2.1 `shell/dcc_shell.gd` (134 fields)

Builders. `_ready()` calls `_compute_layout_mode()`, then **`_build_phone_shell()` (P)** or **`_build_desktop_shell()`
(D)**. Both call the shared builders `_build_menu_bar`, `_build_status_bar`, `_build_viewport`, `_build_timeline`,
`_build_left_dock`, `_build_right_dock` (the phone with `as_sheet = true`). D alone calls `_build_tool_options_bar`,
`_build_rail` (-> `_build_rail_expansion`). P alone calls `_build_phone_top_safe`, `_build_phone_side_safe`,
`_build_phone_app_bar`, `_build_phone_tool_sheet`, `_build_phone_menu_bar`, `_build_phone_gesture_inset`,
`_build_phone_search_overlay`, `_build_phone_undo_chip`, `_build_phone_undo_popover`, `_build_phone_sim_strip`,
`_build_phone_overflow`, `PhoneMenu.new()` and `ToolFan.new()`. The "Set by" column is read from those functions.

| Field | Class | Set by | Action | Why |
|---|---|---|---|---|
| `menu_bar_row` | HBoxContainer | both: `_build_menu_bar` (the phone parks it in the hidden `PhoneMenuModel`) | **rebuilt** | Part of the bar `_build_menu_bar()` makes per composition. `menus.build()` re-adds its `MenuButton`s to the new `menu_bar_row`. |
| `tool_options_row` | HBoxContainer | D: `_build_tool_options_bar`; P: `_build_phone_tool_sheet` | **rebuilt** | Different parent in each class; `set_tool_options()` refills it from `app`'s tool state. |
| `tool_options_bar` | Control | D: `_build_tool_options_bar` | **freed** | Null on the phone. On a phone to tablet swap it is built, and `app._install_tool_palette_bar()` inserts the palette bar above it. |
| `rail_column` | VBoxContainer | D: `_build_rail`; P: `_build_phone_menu_bar` | **rebuilt** | The rail and the phone bottom bar are the two forms of the domain switch. `rail_region()` is read by `app._region_nodes`. |
| `left_dock` | PanelContainer | both: `_build_left_dock` and `_build_right_dock` (`as_sheet` on the phone) | **rebuilt** | A dock in one class, a full-height sheet in the other. **Their contents (the workspace panels, `right_dock_ctrl`) are not these nodes: see section 2.3.** The `left_dock != null and right_dock != null` test in `_ready()` becomes the mount path's gate for `dock_fit` and its `node_added` hook. |
| `left_dock_title` | Label | both: `_build_left_dock` and `_build_right_dock` (`as_sheet` on the phone) | **rebuilt** | A dock in one class, a full-height sheet in the other. **Their contents (the workspace panels, `right_dock_ctrl`) are not these nodes: see section 2.3.** The `left_dock != null and right_dock != null` test in `_ready()` becomes the mount path's gate for `dock_fit` and its `node_added` hook. |
| `left_dock_body` | VBoxContainer | both: `_build_left_dock` and `_build_right_dock` (`as_sheet` on the phone) | **rebuilt** | A dock in one class, a full-height sheet in the other. **Their contents (the workspace panels, `right_dock_ctrl`) are not these nodes: see section 2.3.** The `left_dock != null and right_dock != null` test in `_ready()` becomes the mount path's gate for `dock_fit` and its `node_added` hook. |
| `viewport_area` | Control | both: `_build_viewport` | **rebuilt** | A fresh container per composition. Its three children (`ViewportHost`, `ResourceOverlay`, `SectionStrip`) are survivors and are detached first (section 7). |
| `viewport_content` | Control | both: `_build_viewport` | **rebuilt** | A fresh container per composition. Its three children (`ViewportHost`, `ResourceOverlay`, `SectionStrip`) are survivors and are detached first (section 7). |
| `right_dock` | PanelContainer | both: `_build_left_dock` and `_build_right_dock` (`as_sheet` on the phone) | **rebuilt** | A dock in one class, a full-height sheet in the other. **Their contents (the workspace panels, `right_dock_ctrl`) are not these nodes: see section 2.3.** The `left_dock != null and right_dock != null` test in `_ready()` becomes the mount path's gate for `dock_fit` and its `node_added` hook. |
| `right_dock_title` | Label | both: `_build_left_dock` and `_build_right_dock` (`as_sheet` on the phone) | **rebuilt** | A dock in one class, a full-height sheet in the other. **Their contents (the workspace panels, `right_dock_ctrl`) are not these nodes: see section 2.3.** The `left_dock != null and right_dock != null` test in `_ready()` becomes the mount path's gate for `dock_fit` and its `node_added` hook. |
| `right_dock_body` | VBoxContainer | both: `_build_left_dock` and `_build_right_dock` (`as_sheet` on the phone) | **rebuilt** | A dock in one class, a full-height sheet in the other. **Their contents (the workspace panels, `right_dock_ctrl`) are not these nodes: see section 2.3.** The `left_dock != null and right_dock != null` test in `_ready()` becomes the mount path's gate for `dock_fit` and its `node_added` hook. |
| `timeline_bar` | Control | both: `_build_timeline` | **rebuilt** | A fresh panel per composition. `app.gd`'s `_tl_*` handles point into the row and are reset with it (section 2.2). |
| `timeline_row` | HBoxContainer | both: `_build_timeline` | **rebuilt** | A fresh panel per composition. `app.gd`'s `_tl_*` handles point into the row and are reset with it (section 2.2). |
| `status_row` | HBoxContainer | both: `_build_status_bar` (the phone parks it in `PhoneMenuModel`) | **rebuilt** | `_status_cells` and `_status_rules` are dictionaries of nodes, cleared first. |
| `rail_foot` | Label | D: `_build_rail`, `_build_rail_expansion` | **freed** | Desktop and tablet rail only. `_rail_seps`, `_rail_exp_heads` and `_rail_node_rows` are dictionaries of nodes and are cleared; nothing reads them on the phone. |
| `_rail_foot_stack` | MarginContainer | D: `_build_rail`, `_build_rail_expansion` | **freed** | Desktop and tablet rail only. `_rail_seps`, `_rail_exp_heads` and `_rail_node_rows` are dictionaries of nodes and are cleared; nothing reads them on the phone. |
| `_rail_foot_run` | Label | D: `_build_rail`, `_build_rail_expansion` | **freed** | Desktop and tablet rail only. `_rail_seps`, `_rail_exp_heads` and `_rail_node_rows` are dictionaries of nodes and are cleared; nothing reads them on the phone. |
| `_rail_foot_total` | Label | D: `_build_rail`, `_build_rail_expansion` | **freed** | Desktop and tablet rail only. `_rail_seps`, `_rail_exp_heads` and `_rail_node_rows` are dictionaries of nodes and are cleared; nothing reads them on the phone. |
| `_domain_buttons` | Dictionary | D: `_build_rail`; P: `_build_phone_menu_bar` | **reset** | Dictionaries of nodes keyed by domain id. Cleared before the mount, or a lookup returns a freed button. |
| `_domain_marks` | Dictionary | D: `_build_rail`; P: `_build_phone_menu_bar` | **reset** | Dictionaries of nodes keyed by domain id. Cleared before the mount, or a lookup returns a freed button. |
| `_rail_seps` | Dictionary | D: `_build_rail`, `_build_rail_expansion` | **freed** | Desktop and tablet rail only. `_rail_seps`, `_rail_exp_heads` and `_rail_node_rows` are dictionaries of nodes and are cleared; nothing reads them on the phone. |
| `_rail_exp_heads` | Dictionary | D: `_build_rail`, `_build_rail_expansion` | **freed** | Desktop and tablet rail only. `_rail_seps`, `_rail_exp_heads` and `_rail_node_rows` are dictionaries of nodes and are cleared; nothing reads them on the phone. |
| `_session_mode` | inferred | `set_session_gate` | **survives** | Session gate state; not composition. |
| `_active_domain` | inferred | `_select_domain`, `select_domain_mode` | **survives** | The restored state of spec 3.5 (domain and node). `_ready()`'s `_select_domain(_active_domain)` is re-run by the mount. |
| `_domain_mode` | Dictionary | `_select_domain`, `select_domain_mode` | **survives** | The restored state of spec 3.5 (domain and node). `_ready()`'s `_select_domain(_active_domain)` is re-run by the mount. |
| `_left_collapsed` | inferred | D: `_toggle_dock`, `_on_dock_drag_input`, `_compute_layout_mode` | **survives** | WI-04's dragged widths and collapse flags are kept per class (spec 3.5): a per-class pair must be stored, because `_compute_layout_mode()` overwrites both widths from `role_px()` on a tablet. `_dragging_dock` is transient: quiescence (section 8). |
| `_right_collapsed` | inferred | D: `_toggle_dock`, `_on_dock_drag_input`, `_compute_layout_mode` | **survives** | WI-04's dragged widths and collapse flags are kept per class (spec 3.5): a per-class pair must be stored, because `_compute_layout_mode()` overwrites both widths from `role_px()` on a tablet. `_dragging_dock` is transient: quiescence (section 8). |
| `_left_width` | inferred | D: `_toggle_dock`, `_on_dock_drag_input`, `_compute_layout_mode` | **survives** | WI-04's dragged widths and collapse flags are kept per class (spec 3.5): a per-class pair must be stored, because `_compute_layout_mode()` overwrites both widths from `role_px()` on a tablet. `_dragging_dock` is transient: quiescence (section 8). |
| `_right_width` | inferred | D: `_toggle_dock`, `_on_dock_drag_input`, `_compute_layout_mode` | **survives** | WI-04's dragged widths and collapse flags are kept per class (spec 3.5): a per-class pair must be stored, because `_compute_layout_mode()` overwrites both widths from `role_px()` on a tablet. `_dragging_dock` is transient: quiescence (section 8). |
| `_status_labels` | Dictionary | both: `_build_menu_bar`, `_build_status_bar`, `_build_phone_app_bar` | **reset** | **One dictionary, last registered wins** (the comment in `_build_phone_shell`): the builders write the same `top_world` slot, and the phone's app bar must be built after the menu bar. A mount must clear it and keep that order. |
| `_menu_undo_btn` | Button | both: `_build_menu_bar` (the phone parks it in the hidden `PhoneMenuModel`) | **rebuilt** | Part of the bar `_build_menu_bar()` makes per composition. `menus.build()` re-adds its `MenuButton`s to the new `menu_bar_row`. |
| `_menu_redo_btn` | Button | both: `_build_menu_bar` (the phone parks it in the hidden `PhoneMenuModel`) | **rebuilt** | Part of the bar `_build_menu_bar()` makes per composition. `menus.build()` re-adds its `MenuButton`s to the new `menu_bar_row`. |
| `_collapse_buttons` | Dictionary | D: `_collapse_button` (not built for a sheet) | **reset** | Dictionary of buttons, cleared. Phone sheets have a close button instead. |
| `_dock_readouts` | Dictionary | both: `_dock_readout` | **reset** | Dictionary of labels, cleared. The last text is not stored, so the mount re-pushes it (`right_dock._push_dock_readout()`). |
| `_workspace_panels` | Dictionary | `register_workspace` | **reset** | Domain id to panel node. The three workspace panels are rebuilt (section 2.3), so the dictionary is rebuilt with them. |
| `_touch` | inferred | `_ready` | **survives** | A device property, constant for the process. |
| `_rail_region` | Control | D: `_build_rail`; P: `_build_phone_menu_bar` | **rebuilt** | The rail and the phone bottom bar are the two forms of the domain switch. `rail_region()` is read by `app._region_nodes`. |
| `_rail_expanded` | inferred | D: `set_rail_expanded` | **survives** | A user choice (the 200/264 px rail). Re-applied by `set_rail_expanded()` after a tablet mount; ignored on the phone. |
| `_rail_exp_column` | Control | D: `_build_rail`, `_build_rail_expansion` | **freed** | Desktop and tablet rail only. `_rail_seps`, `_rail_exp_heads` and `_rail_node_rows` are dictionaries of nodes and are cleared; nothing reads them on the phone. |
| `_rail_chevron` | Control | D: `_build_rail`, `_build_rail_expansion` | **freed** | Desktop and tablet rail only. `_rail_seps`, `_rail_exp_heads` and `_rail_node_rows` are dictionaries of nodes and are cleared; nothing reads them on the phone. |
| `_rail_node_rows` | Dictionary | D: `_build_rail`, `_build_rail_expansion` | **freed** | Desktop and tablet rail only. `_rail_seps`, `_rail_exp_heads` and `_rail_node_rows` are dictionaries of nodes and are cleared; nothing reads them on the phone. |
| `_dragging_dock` | inferred | D: `_toggle_dock`, `_on_dock_drag_input`, `_compute_layout_mode` | **survives** | WI-04's dragged widths and collapse flags are kept per class (spec 3.5): a per-class pair must be stored, because `_compute_layout_mode()` overwrites both widths from `role_px()` on a tablet. `_dragging_dock` is transient: quiescence (section 8). |
| `_phone` | inferred | `_compute_layout_mode` | **reset** | `_phone` is written only by the boot call (`_booted` guard); after S4 only by the swap. Published again to `DccTheme` in the same step (section 4). |
| `_landscape` | inferred | `_compute_layout_mode` | **reset** | `_phone` is written only by the boot call (`_booted` guard); after S4 only by the swap. Published again to `DccTheme` in the same step (section 4). |
| `_phone_kb_height` | inferred | `_process` | **reset** | Polled keyboard height; zero is right after a swap. `_process()` turns itself off on a non-phone boot, so a tablet to phone swap must `set_process(true)` (its own comment says so). |
| `_phone_scale` | inferred | `_compute_layout_mode` | **reset** | `_phone` is written only by the boot call (`_booted` guard); after S4 only by the swap. Published again to `DccTheme` in the same step (section 4). |
| `_phone_root` | Control | P: `_build_phone_shell` | **freed** | The whole phone composition hangs from `_phone_root`: freeing it frees these. The fields are nulled, because `_phone_root` validity is how B5 reads the class. |
| `_phone_top_safe` | Control | P: `_build_phone_shell` | **freed** | The whole phone composition hangs from `_phone_root`: freeing it frees these. The fields are nulled, because `_phone_root` validity is how B5 reads the class. |
| `_phone_side_safe` | Control | P: `_build_phone_shell` | **freed** | The whole phone composition hangs from `_phone_root`: freeing it frees these. The fields are nulled, because `_phone_root` validity is how B5 reads the class. |
| `_phone_chrome_margin` | MarginContainer | P: `_build_phone_shell` | **freed** | The whole phone composition hangs from `_phone_root`: freeing it frees these. The fields are nulled, because `_phone_root` validity is how B5 reads the class. |
| `_phone_content_gap` | Control | P: `_build_phone_shell` | **freed** | The whole phone composition hangs from `_phone_root`: freeing it frees these. The fields are nulled, because `_phone_root` validity is how B5 reads the class. |
| `_phone_tool_sheet` | PanelContainer | P: `_build_phone_shell` | **freed** | The whole phone composition hangs from `_phone_root`: freeing it frees these. The fields are nulled, because `_phone_root` validity is how B5 reads the class. |
| `_phone_tool_scroll` | ScrollContainer | P: `_build_phone_tool_sheet`, `set_phone_quick_row` | **freed** | `_phone_quick_row` is handed in by `QuickBarPhone`, a `RefCounted` that survives: it re-offers a row on the next `refresh()`. The row's parent is freed with the sheet. |
| `_phone_sheet_col` | VBoxContainer | P: `_build_phone_tool_sheet`, `set_phone_quick_row` | **freed** | `_phone_quick_row` is handed in by `QuickBarPhone`, a `RefCounted` that survives: it re-offers a row on the next `refresh()`. The row's parent is freed with the sheet. |
| `_phone_quick_row` | Control | P: `_build_phone_tool_sheet`, `set_phone_quick_row` | **freed** | `_phone_quick_row` is handed in by `QuickBarPhone`, a `RefCounted` that survives: it re-offers a row on the next `refresh()`. The row's parent is freed with the sheet. |
| `_phone_gen_scroll` | ScrollContainer | P: `_build_phone_tool_sheet`, `set_phone_quick_row` | **freed** | `_phone_quick_row` is handed in by `QuickBarPhone`, a `RefCounted` that survives: it re-offers a row on the next `refresh()`. The row's parent is freed with the sheet. |
| `_phone_gen_col` | VBoxContainer | P: `_build_phone_tool_sheet`, `set_phone_quick_row` | **freed** | `_phone_quick_row` is handed in by `QuickBarPhone`, a `RefCounted` that survives: it re-offers a row on the next `refresh()`. The row's parent is freed with the sheet. |
| `_phone_app_bar` | PanelContainer | P: `_build_phone_shell` | **freed** | The whole phone composition hangs from `_phone_root`: freeing it frees these. The fields are nulled, because `_phone_root` validity is how B5 reads the class. |
| `_phone_gesture_inset` | Control | P: `_build_phone_shell` | **freed** | The whole phone composition hangs from `_phone_root`: freeing it frees these. The fields are nulled, because `_phone_root` validity is how B5 reads the class. |
| `_phone_detent` | inferred | P: `_set_phone_detent`, `_set_sheet_open` | **reset** | Sheet detent and which dock sheet is open. A phone to tablet swap closes both (the dock is then a column); a tablet to phone swap starts at `peek` (spec 3.5: "within the new class's rules"). |
| `_phone_sheet_grab` | Control | P: `_build_phone_tool_sheet`, `set_phone_quick_row` | **freed** | `_phone_quick_row` is handed in by `QuickBarPhone`, a `RefCounted` that survives: it re-offers a row on the next `refresh()`. The row's parent is freed with the sheet. |
| `_phone_sheet_drag` | inferred | P: `_on_phone_sheet_grab_input`, `_snap_phone_sheet` | **reset** | A live drag and the snap tween. The drag is a quiescence source; the `Tween` is killed before the sheet is freed (a `Tween` is not a Node, so freeing the sheet would leave it running). |
| `_phone_sheet_tween` | Tween | P: `_on_phone_sheet_grab_input`, `_snap_phone_sheet` | **reset** | A live drag and the snap tween. The drag is a quiescence source; the `Tween` is killed before the sheet is freed (a `Tween` is not a Node, so freeing the sheet would leave it running). |
| `_phone_sheet_title` | Label | P: `_build_phone_tool_sheet`, `set_phone_quick_row` | **freed** | `_phone_quick_row` is handed in by `QuickBarPhone`, a `RefCounted` that survives: it re-offers a row on the next `refresh()`. The row's parent is freed with the sheet. |
| `_phone_sheet_subtitle` | Label | P: `_build_phone_tool_sheet`, `set_phone_quick_row` | **freed** | `_phone_quick_row` is handed in by `QuickBarPhone`, a `RefCounted` that survives: it re-offers a row on the next `refresh()`. The row's parent is freed with the sheet. |
| `_phone_chrome_col` | VBoxContainer | P: `_build_phone_shell` | **freed** | The whole phone composition hangs from `_phone_root`: freeing it frees these. The fields are nulled, because `_phone_root` validity is how B5 reads the class. |
| `_phone_bar_row` | BoxContainer | P: `_build_phone_menu_bar` | **freed** | The bottom bar's boxes. `_phone_tab_cells` is a dictionary of buttons, cleared. |
| `_phone_bar_domains` | BoxContainer | P: `_build_phone_menu_bar` | **freed** | The bottom bar's boxes. `_phone_tab_cells` is a dictionary of buttons, cleared. |
| `_phone_bar_cells` | BoxContainer | P: `_build_phone_menu_bar` | **freed** | The bottom bar's boxes. `_phone_tab_cells` is a dictionary of buttons, cleared. |
| `_phone_bar_dests` | BoxContainer | P: `_build_phone_menu_bar` | **freed** | The bottom bar's boxes. `_phone_tab_cells` is a dictionary of buttons, cleared. |
| `_phone_clock_label` | Label | P: `_build_phone_top_safe`, `_build_phone_side_safe` | **freed** | Status glyphs. `_refresh_phone_status_glyphs()` re-fills them. |
| `_phone_battery_label` | Label | P: `_build_phone_top_safe`, `_build_phone_side_safe` | **freed** | Status glyphs. `_refresh_phone_status_glyphs()` re-fills them. |
| `_phone_side_clock_label` | Label | P: `_build_phone_top_safe`, `_build_phone_side_safe` | **freed** | Status glyphs. `_refresh_phone_status_glyphs()` re-fills them. |
| `_phone_side_battery_label` | Label | P: `_build_phone_top_safe`, `_build_phone_side_safe` | **freed** | Status glyphs. `_refresh_phone_status_glyphs()` re-fills them. |
| `_phone_menu_bar` | Control | P: `_build_phone_shell` | **freed** | The whole phone composition hangs from `_phone_root`: freeing it frees these. The fields are nulled, because `_phone_root` validity is how B5 reads the class. |
| `_phone_menu` | PhoneMenu | P: `_build_phone_shell` | **freed** | **Not wholly under `_phone_root`**: `ToolFan.setup()` parents an overlay onto `self` (the app) and `mount()`s a pill into `_phone_content_gap`. `PhoneMenu` is under `_phone_root`. `ToolFan` is a plain `Node`, so it must be freed explicitly. |
| `_tool_fan` | ToolFan | P: `_build_phone_shell` | **freed** | **Not wholly under `_phone_root`**: `ToolFan.setup()` parents an overlay onto `self` (the app) and `mount()`s a pill into `_phone_content_gap`. `PhoneMenu` is under `_phone_root`. `ToolFan` is a plain `Node`, so it must be freed explicitly. |
| `_left_sheet_open` | inferred | P: `_set_phone_detent`, `_set_sheet_open` | **reset** | Sheet detent and which dock sheet is open. A phone to tablet swap closes both (the dock is then a column); a tablet to phone swap starts at `peek` (spec 3.5: "within the new class's rules"). |
| `_right_sheet_open` | inferred | P: `_set_phone_detent`, `_set_sheet_open` | **reset** | Sheet detent and which dock sheet is open. A phone to tablet swap closes both (the dock is then a column); a tablet to phone swap starts at `peek` (spec 3.5: "within the new class's rules"). |
| `_left_dock_scroll` | ScrollContainer | both: `_build_left_dock` and `_build_right_dock` (`as_sheet` on the phone) | **rebuilt** | A dock in one class, a full-height sheet in the other. **Their contents (the workspace panels, `right_dock_ctrl`) are not these nodes: see section 2.3.** The `left_dock != null and right_dock != null` test in `_ready()` becomes the mount path's gate for `dock_fit` and its `node_added` hook. |
| `_right_dock_scroll` | ScrollContainer | both: `_build_left_dock` and `_build_right_dock` (`as_sheet` on the phone) | **rebuilt** | A dock in one class, a full-height sheet in the other. **Their contents (the workspace panels, `right_dock_ctrl`) are not these nodes: see section 2.3.** The `left_dock != null and right_dock != null` test in `_ready()` becomes the mount path's gate for `dock_fit` and its `node_added` hook. |
| `_phone_search_overlay` | Control | P: `_build_phone_search_overlay` | **freed** | The ⌕ overlay. Whether it was open and its query text are carried (`_set_search_query()`) into the desktop dialog. |
| `_phone_search_field` | LineEdit | P: `_build_phone_search_overlay` | **freed** | The ⌕ overlay. Whether it was open and its query text are carried (`_set_search_query()`) into the desktop dialog. |
| `_phone_search_results` | VBoxContainer | P: `_build_phone_search_overlay` | **freed** | The ⌕ overlay. Whether it was open and its query text are carried (`_set_search_query()`) into the desktop dialog. |
| `_phone_search_chips` | HFlowContainer | P: `_build_phone_search_overlay` | **freed** | The ⌕ overlay. Whether it was open and its query text are carried (`_set_search_query()`) into the desktop dialog. |
| `_phone_search_count` | Label | P: `_build_phone_search_overlay` | **freed** | The ⌕ overlay. Whether it was open and its query text are carried (`_set_search_query()`) into the desktop dialog. |
| `_place_search_index` | untyped | `_run_place_search`, `_reset_place_search_cache` | **survives** | Indexes world places, not UI (cleared on a world change). |
| `_command_index` | CommandIndex | `_ensure_command_index` | **rebuilt** | Walks the live `MenuBar` (`MISTAKES.md`, "Move a command off the menu bar"), which no longer exists after a swap. Nulled, rebuilt on first use. |
| `_phone_undo_chip` | Button | P: `_build_phone_undo_chip`, `_build_phone_undo_popover` | **freed** | Chip and popover. The two flags are transient: quiescence while the chip is held (section 8). |
| `_phone_undo_pop` | Control | P: `_build_phone_undo_chip`, `_build_phone_undo_popover` | **freed** | Chip and popover. The two flags are transient: quiescence while the chip is held (section 8). |
| `_phone_sim_strip` | Control | P: `_build_phone_sim_strip` | **freed** | The sim strip (open state, `is_phone_sim_strip_open()`, is not carried to a tablet, which has the timeline bar). `_phone_sim_speeds` and `_phone_sim_transport` hold nodes and are cleared. |
| `_phone_sim_year` | Label | P: `_build_phone_sim_strip` | **freed** | The sim strip (open state, `is_phone_sim_strip_open()`, is not carried to a tablet, which has the timeline bar). `_phone_sim_speeds` and `_phone_sim_transport` hold nodes and are cleared. |
| `_phone_sim_slider` | HSlider | P: `_build_phone_sim_strip` | **freed** | The sim strip (open state, `is_phone_sim_strip_open()`, is not carried to a tablet, which has the timeline bar). `_phone_sim_speeds` and `_phone_sim_transport` hold nodes and are cleared. |
| `_phone_sim_play` | Button | P: `_build_phone_sim_strip` | **freed** | The sim strip (open state, `is_phone_sim_strip_open()`, is not carried to a tablet, which has the timeline bar). `_phone_sim_speeds` and `_phone_sim_transport` hold nodes and are cleared. |
| `_phone_sim_speeds` | Dictionary | P: `_build_phone_sim_strip` | **freed** | The sim strip (open state, `is_phone_sim_strip_open()`, is not carried to a tablet, which has the timeline bar). `_phone_sim_speeds` and `_phone_sim_transport` hold nodes and are cleared. |
| `_phone_sim_transport` | Array[Control] | P: `_build_phone_sim_strip` | **freed** | The sim strip (open state, `is_phone_sim_strip_open()`, is not carried to a tablet, which has the timeline bar). `_phone_sim_speeds` and `_phone_sim_transport` hold nodes and are cleared. |
| `_phone_overflow_pop` | Control | P: `_build_phone_overflow`, `_wire_phone_overflow` | **freed** | The ⋮ popover. `_phone_saved_at` is a string, recomputed from the autosave state. |
| `_phone_overflow_saved` | Label | P: `_build_phone_overflow`, `_wire_phone_overflow` | **freed** | The ⋮ popover. `_phone_saved_at` is a string, recomputed from the autosave state. |
| `_phone_overflow_theme` | Label | P: `_build_phone_overflow`, `_wire_phone_overflow` | **freed** | The ⋮ popover. `_phone_saved_at` is a string, recomputed from the autosave state. |
| `_phone_saved_at` | inferred | P: `_build_phone_overflow`, `_wire_phone_overflow` | **freed** | The ⋮ popover. `_phone_saved_at` is a string, recomputed from the autosave state. |
| `_undo_chip_down` | inferred | P: `_build_phone_undo_chip`, `_build_phone_undo_popover` | **freed** | Chip and popover. The two flags are transient: quiescence while the chip is held (section 8). |
| `_undo_chip_hold_fired` | inferred | P: `_build_phone_undo_chip`, `_build_phone_undo_popover` | **freed** | Chip and popover. The two flags are transient: quiescence while the chip is held (section 8). |
| `_desktop_search_dialog` | AcceptDialog | `_ensure_desktop_search_dialog` (lazy; desktop and tablet) | **re-presented** | A window child of the shell, outside both compositions. On a tablet to phone swap it is freed and the phone's ⌕ overlay takes over with the query text (`_set_search_query()`). |
| `_desktop_search_field` | LineEdit | `_ensure_desktop_search_dialog` (lazy; desktop and tablet) | **re-presented** | A window child of the shell, outside both compositions. On a tablet to phone swap it is freed and the phone's ⌕ overlay takes over with the query text (`_set_search_query()`). |
| `_desktop_search_results` | VBoxContainer | `_ensure_desktop_search_dialog` (lazy; desktop and tablet) | **re-presented** | A window child of the shell, outside both compositions. On a tablet to phone swap it is freed and the phone's ⌕ overlay takes over with the query text (`_set_search_query()`). |
| `_desktop_search_chips` | HFlowContainer | `_ensure_desktop_search_dialog` (lazy; desktop and tablet) | **re-presented** | A window child of the shell, outside both compositions. On a tablet to phone swap it is freed and the phone's ⌕ overlay takes over with the query text (`_set_search_query()`). |
| `_desktop_search_count` | Label | `_ensure_desktop_search_dialog` (lazy; desktop and tablet) | **re-presented** | A window child of the shell, outside both compositions. On a tablet to phone swap it is freed and the phone's ⌕ overlay takes over with the query text (`_set_search_query()`). |
| `_layout` | Dictionary | `_compute_layout_mode`, `_ready`, `_on_window_resized`, `_os_text_scale` | **survives** | The classifier's last result, the boot guard, the debounce timer (a child of the shell) and a font-scale cache. |
| `_booted` | inferred | `_compute_layout_mode`, `_ready`, `_on_window_resized`, `_os_text_scale` | **survives** | The classifier's last result, the boot guard, the debounce timer (a child of the shell) and a font-scale cache. |
| `_resize_timer` | Timer | `_compute_layout_mode`, `_ready`, `_on_window_resized`, `_os_text_scale` | **survives** | The classifier's last result, the boot guard, the debounce timer (a child of the shell) and a font-scale cache. |
| `_os_text_scale_cache` | inferred | `_compute_layout_mode`, `_ready`, `_on_window_resized`, `_os_text_scale` | **survives** | The classifier's last result, the boot guard, the debounce timer (a child of the shell) and a font-scale cache. |
| `_tablet_arb_pending` | inferred | hooks `_on_phone_node_added`, `_on_tablet_node_added`, `_on_dock_node_added` | **reset** | Debounce flags of the `node_added` hooks. A flag left true when its hook is disconnected would block the next class's pass forever. |
| `_phone_fit_pending` | inferred | hooks `_on_phone_node_added`, `_on_tablet_node_added`, `_on_dock_node_added` | **reset** | Debounce flags of the `node_added` hooks. A flag left true when its hook is disconnected would block the next class's pass forever. |
| `_dock_fit_pending` | inferred | hooks `_on_phone_node_added`, `_on_tablet_node_added`, `_on_dock_node_added` | **reset** | Debounce flags of the `node_added` hooks. A flag left true when its hook is disconnected would block the next class's pass forever. |
| `_phone_status_shown` | inferred | `set_status_region_shown` | **survives** | A user toggle of the status region. |
| `_mode_switch_row` | Control | both: `_build_mode_switch`, `_rebuild_mode_switch` | **rebuilt** | One per dock. `_mode_switch_domain` is a guard ("which domain the buttons were built for"): reset to "" so `_rebuild_mode_switch()` runs. |
| `_mode_switch_pill` | HBoxContainer | both: `_build_mode_switch`, `_rebuild_mode_switch` | **rebuilt** | One per dock. `_mode_switch_domain` is a guard ("which domain the buttons were built for"): reset to "" so `_rebuild_mode_switch()` runs. |
| `_mode_switch_buttons` | Dictionary | both: `_build_mode_switch`, `_rebuild_mode_switch` | **rebuilt** | One per dock. `_mode_switch_domain` is a guard ("which domain the buttons were built for"): reset to "" so `_rebuild_mode_switch()` runs. |
| `_mode_switch_domain` | inferred | both: `_build_mode_switch`, `_rebuild_mode_switch` | **rebuilt** | One per dock. `_mode_switch_domain` is a guard ("which domain the buttons were built for"): reset to "" so `_rebuild_mode_switch()` runs. |
| `_vp_stage` | inferred | `_refresh_viewport_context` | **survives** | Guards on wiring `ViewportHost`, which survives. `_refresh_viewport_context()` is called again after the mount. |
| `_vp_wired` | inferred | `_refresh_viewport_context` | **survives** | Guards on wiring `ViewportHost`, which survives. `_refresh_viewport_context()` is called again after the mount. |
| `tl_speed` | inferred | `tl_set_speed`, `tl_toggle_play`, `_tl_load_layers` | **survives** | Timeline playback state; the timer is a child of the shell. Playback continues across a swap. |
| `tl_playing` | inferred | `tl_set_speed`, `tl_toggle_play`, `_tl_load_layers` | **survives** | Timeline playback state; the timer is a child of the shell. Playback continues across a swap. |
| `tl_layers` | Dictionary | `tl_set_speed`, `tl_toggle_play`, `_tl_load_layers` | **survives** | Timeline playback state; the timer is a child of the shell. Playback continues across a swap. |
| `_tl_timer` | Timer | `tl_set_speed`, `tl_toggle_play`, `_tl_load_layers` | **survives** | Timeline playback state; the timer is a child of the shell. Playback continues across a swap. |
| `_status_cells` | Dictionary | both: `_build_status_bar` (the phone parks it in `PhoneMenuModel`) | **rebuilt** | `_status_cells` and `_status_rules` are dictionaries of nodes, cleared first. |
| `_status_rules` | Dictionary | both: `_build_status_bar` (the phone parks it in `PhoneMenuModel`) | **rebuilt** | `_status_cells` and `_status_rules` are dictionaries of nodes, cleared first. |
| `_status_dot` | Label | both: `_build_status_bar` (the phone parks it in `PhoneMenuModel`) | **rebuilt** | `_status_cells` and `_status_rules` are dictionaries of nodes, cleared first. |
| `_status_wired` | inferred | `_wire_status_progress` | **survives** | Guard flag that connects `bridge` signals once. The lambdas read `_status_labels` at call time, so they follow the new nodes. Must not be reset, or each mount doubles the connections. |
| `_status_tail` | Control | both: `_build_status_bar` (the phone parks it in `PhoneMenuModel`) | **rebuilt** | `_status_cells` and `_status_rules` are dictionaries of nodes, cleared first. |
| `_phone_tab_cells` | Dictionary | P: `_build_phone_menu_bar` | **freed** | The bottom bar's boxes. `_phone_tab_cells` is a dictionary of buttons, cleared. |
| `_phone_tab` | inferred | P: `_pick_phone_tab`, `_select_domain` | **survives** | Derived from the domain by `_phone_tab_for_domain()`; kept so the GENERATE-tab boot case in `register_workspace()` still holds. |
| `_coach_marks_wait_for_picker` | inferred | `_start_or_defer_coach_marks` | **reset** | Phone coach marks; a swap into the phone class starts from the stored "seen" flags (`_coach_mark_seen()`). |

### 2.2 `shell/app.gd` (100 fields; `_ready()` and what it calls)

`DccApp._ready()` runs `super._ready()` (the composition above) and then everything below. The
**composition-dependent** part, which the mount path must own, is: `_install_tool_palette_bar()`; the `bridge`-signal
lambdas that capture `_quick_bar`; `viewport.set_safe_insets` and its `phone_insets_changed` lambda (`if _phone`);
`menus.build()`; `phone_project_picker` (`if is_phone()`); `_register_workspaces()` and `right_dock_ctrl`; `section_strip`;
`_region_nodes` (`if not is_phone()`); `journey_planner_view`; `_open_welcome_when_drawn()` (picker versus dialog). The rest
(the windows, `bridge`, autosave, `GlobalTools.install`, `DccToolBar.install`, `paint_original`, the busy overlay) is
class-independent at build and stays in `_ready()`.

| Field | Class | Set by | Action | Why |
|---|---|---|---|---|
| `bridge` | EngineBridge | `_ready`: `EngineBridge.new()` child of the app | **survives** | Owns the world and the engine. A child of the app, outside both compositions. Never detached (section 7). |
| `viewport` | ViewportHost | `_ready`: `ViewportHost.new()` into `viewport_content` | **survives** | Detached from the old `viewport_content` and re-added to the new one; the camera, zoom and the overlay live inside it (section 7). |
| `menus` | inferred | `_ready`: `menus.build(self, bridge, self)` | **rebuilt** | **`DccMenus` builds a different bar per class** (a tablet gets `☰ · File · World · Story`, a phone and the PC seven menus; 12 `is_phone()` reads and about 38 `PopupMenu` fields, all node-holding). A fresh `DccMenus.new()` is built into the new `menu_bar_row`, and its state is re-derived from `DccSettings`, the bridge and `armed_tool`. |
| `new_world_dialog` | NewWorldDialog | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `world_data_window` | WorldDataWindow | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `resource_overlay` | ResourceOverlay | `_ready`: children of `viewport_content` | **survives** | The other two overlay children of `viewport_content`. Detached and re-attached with `viewport`, in the same order (`section_strip` after `resource_overlay` after `viewport`, for draw order). |
| `gen_info_dialog` | GenInfoDialog | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `shortcuts_dialog` | ShortcutsDialog | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `assistant_settings_window` | LlmSettingsWindow | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `quick_bar_window` | QuickBarWindow | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `quick_bar_menu_pin` | QuickBarMenuPin | `_ready` | **survives** | Not a window of its own: it hooks every `PopupMenu` through `node_added`. The `DccMenus` rebuild creates new popups, which its `node_added` hook picks up; its own `_menu` is a child of itself. |
| `phone_quick` | QuickBarPhone | `_ready`: `QuickBarPhone.new()` (`RefCounted`) | **survives** | No node of its own; its row is parented by `_phone_tool_sheet`. `refresh()` re-offers the row after a mount. Its `world_loaded` and `generation_finished` lambdas read `phone_quick` at call time. |
| `assistant_window` | LlmAssistantWindow | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `journey_planner_view` | JourneyPlannerView | `_ready`: `JourneyPlannerView.new()` after `workspace_changed` (order matters) | **rebuilt** | Reads `_phone` at `setup()` (its own `_phone`, line `_phone = app != null and app.is_phone()`), caches `app.left_dock` and `tool_options_bar`, and swaps regions of the composition. Rebuilt after the mount and re-connected after `workspace_changed`, for the order its class comment states. |
| `layers_popover` | LayersPopover | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `right_dock_ctrl` | RightDock | `_ready`: `RightDock.new()`, `setup()` | **rebuilt** | Builds into `right_dock_body` (new each mount). 83 class reads (`is_tablet()` 32, `role_px()` 62 in the file); the context chain is built per class. Appended to `_workspaces`, so rebuilt with them. It carries the current selection (`_wire_selection()` forwards it again). |
| `look_history` | untyped | field initialisers | **survives** | Engine-adjacent state objects (undo history), not UI. |
| `undo_groups` | untyped | field initialisers | **survives** | Engine-adjacent state objects (undo history), not UI. |
| `section_strip` | SectionStrip | `_ready`: children of `viewport_content` | **survives** | The other two overlay children of `viewport_content`. Detached and re-attached with `viewport`, in the same order (`section_strip` after `resource_overlay` after `viewport`, for draw order). |
| `tool_bar` | DccToolBar | `_ready`: `DccToolBar.install()`, `RefCounted` | **survives** | Plain object with signal connections to `tool_armed` and the bridge, and a static `_instance`. It draws into `tool_options_row`, which `set_tool_options()` refills; refresh after the mount. |
| `data_manager_window` | DataManagerWindow | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `asset_library_window` | AssetLibraryWindow | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `travel_library_window` | TravelLibraryWindow | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `city_viewer_window` | CityViewerWindow | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `place_editor_window` | PlaceEditorWindow | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `faction_roster_window` | FactionRosterWindow | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `settlement_types_window` | SettlementTypesWindow | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `generation_rules_window` | GenerationRulesWindow | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `vault_window` | VaultWindow | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `open_project_dialog` | OpenProjectDialog | `_ready`: `X.new()`, `add_child(X)`, `X.setup(...)` | **re-presented** | Long-lived windows, children of the app and so outside both compositions. Each decides its layout in `setup()` through `phone_window()` (section 5), so each needs a re-present path. |
| `phone_project_picker` | PhoneProjectPicker | `_ready`: only `if is_phone()` | **rebuilt** | **Built only on the phone, and the node not existing at all is load-bearing** (its header). A phone to tablet swap frees it and nulls the field; a tablet to phone swap builds it. `open_welcome()` and the `is_phone()` branches at its two call sites read the field. |
| `current_project_path` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_include_lod_tiles_pref` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `last_open_refusal` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_workspaces` | Array | `_register_workspaces`, `_ready` | **reset** | Array of the three workspaces and `right_dock_ctrl`; rebuilt (section 2.3). `_wire_selection()` forwards to this array, so it is re-run after the rebuild. |
| `_region_nodes` | Dictionary | `_ready`: dictionary literal | **reset** | Maps `ID_WIN_*` menu ids to composition nodes (`timeline_row`'s grandparent, `rail_region()`, and on desktop the docks and `status_row`). Rebuilt in the mount path (`toggle_region()` reads it); the phone leaves out `ID_WIN_LEFT/RIGHT/STATUS`. |
| `_tool_options_stale` | Label | `set_tool_options` (line 3208, 3226) and `_wire_status` (1731) | **reset** | Nodes in the tool-options row and the status row, both rebuilt. Nulled; `set_tool_options()` and `_wire_status()`'s own paint rebuild them. |
| `_tool_options_bake` | Button | `set_tool_options` (line 3208, 3226) and `_wire_status` (1731) | **reset** | Nodes in the tool-options row and the status row, both rebuilt. Nulled; `set_tool_options()` and `_wire_status()`'s own paint rebuild them. |
| `tool_group` | inferred | field initialiser | **survives** | A `ButtonGroup` (not a node). Palette buttons join it as they are built, so the new palette's buttons rejoin it. |
| `armed_tool` | inferred | `arm_tool` | **survives** | The armed tool is **user state and a quiescence source** (section 8): a swap waits while a draft is armed and live. |
| `_arm_pending` | inferred | `arm_tool` | **survives** | The armed tool is **user state and a quiescence source** (section 8): a swap waits while a draft is armed and live. |
| `_emitted_tool` | inferred | `arm_tool` | **survives** | The armed tool is **user state and a quiescence source** (section 8): a swap waits while a draft is armed and live. |
| `_click_handlers` | Dictionary | workspaces register into them | **reset** | Callables keyed by tool id or domain, registered by workspace instances. A rebuilt workspace replaces its own ids, but **an id that a rebuilt workspace does not re-register keeps a Callable to a freed object**. The mount clears all six before `_register_workspaces()`. |
| `_drag_handlers` | Dictionary | workspaces register into them | **reset** | Callables keyed by tool id or domain, registered by workspace instances. A rebuilt workspace replaces its own ids, but **an id that a rebuilt workspace does not re-register keeps a Callable to a freed object**. The mount clears all six before `_register_workspaces()`. |
| `_release_handlers` | Dictionary | workspaces register into them | **reset** | Callables keyed by tool id or domain, registered by workspace instances. A rebuilt workspace replaces its own ids, but **an id that a rebuilt workspace does not re-register keeps a Callable to a freed object**. The mount clears all six before `_register_workspaces()`. |
| `_escape_handlers` | Dictionary | workspaces register into them | **reset** | Callables keyed by tool id or domain, registered by workspace instances. A rebuilt workspace replaces its own ids, but **an id that a rebuilt workspace does not re-register keeps a Callable to a freed object**. The mount clears all six before `_register_workspaces()`. |
| `_backspace_handlers` | Dictionary | workspaces register into them | **reset** | Callables keyed by tool id or domain, registered by workspace instances. A rebuilt workspace replaces its own ids, but **an id that a rebuilt workspace does not re-register keeps a Callable to a freed object**. The mount clears all six before `_register_workspaces()`. |
| `_domain_tools` | Dictionary | workspaces register into them | **reset** | Callables keyed by tool id or domain, registered by workspace instances. A rebuilt workspace replaces its own ids, but **an id that a rebuilt workspace does not re-register keeps a Callable to a freed object**. The mount clears all six before `_register_workspaces()`. |
| `tool_palette_bar` | PanelContainer | `_install_tool_palette_bar` (desktop and tablet only; `if is_phone(): return`) | **rebuilt** | **A phone to tablet swap must call `_install_tool_palette_bar()`, and a tablet to phone swap must free these five and null them.** `_quick_bar` is read by lambdas connected in `_ready()` (`bridge.world_loaded`, `generation_finished`) and by `quick_bar_window.pins_changed`: only the last null-checks, so the first two must be re-pointed or they error on a null `_quick_bar`. |
| `_tool_palette_row` | HBoxContainer | `_install_tool_palette_bar` (desktop and tablet only; `if is_phone(): return`) | **rebuilt** | **A phone to tablet swap must call `_install_tool_palette_bar()`, and a tablet to phone swap must free these five and null them.** `_quick_bar` is read by lambdas connected in `_ready()` (`bridge.world_loaded`, `generation_finished`) and by `quick_bar_window.pins_changed`: only the last null-checks, so the first two must be re-pointed or they error on a null `_quick_bar`. |
| `_tool_strip_row` | HBoxContainer | `_install_tool_palette_bar` (desktop and tablet only; `if is_phone(): return`) | **rebuilt** | **A phone to tablet swap must call `_install_tool_palette_bar()`, and a tablet to phone swap must free these five and null them.** `_quick_bar` is read by lambdas connected in `_ready()` (`bridge.world_loaded`, `generation_finished`) and by `quick_bar_window.pins_changed`: only the last null-checks, so the first two must be re-pointed or they error on a null `_quick_bar`. |
| `_quick_bar` | DccQuickBar | `_install_tool_palette_bar` (desktop and tablet only; `if is_phone(): return`) | **rebuilt** | **A phone to tablet swap must call `_install_tool_palette_bar()`, and a tablet to phone swap must free these five and null them.** `_quick_bar` is read by lambdas connected in `_ready()` (`bridge.world_loaded`, `generation_finished`) and by `quick_bar_window.pins_changed`: only the last null-checks, so the first two must be re-pointed or they error on a null `_quick_bar`. |
| `_quick_rule` | Control | `_install_tool_palette_bar` (desktop and tablet only; `if is_phone(): return`) | **rebuilt** | **A phone to tablet swap must call `_install_tool_palette_bar()`, and a tablet to phone swap must free these five and null them.** `_quick_bar` is read by lambdas connected in `_ready()` (`bridge.world_loaded`, `generation_finished`) and by `quick_bar_window.pins_changed`: only the last null-checks, so the first two must be re-pointed or they error on a null `_quick_bar`. |
| `context_broker` | ContextBroker | `_wire_selection` (`ContextBroker.new(self)`) | **survives** | `RefCounted` that lazily builds `RadialRing` (a `Control` parented to the app) and `ContextCard`. The ring's `_apply_touch_scale()` must be re-run, or the ring freed and lazily rebuilt (section 6). |
| `paint_original` | PaintOriginal | `_ready`: `PaintOriginal.new()` | **survives** | A viewing aid with a `setup(app, bridge)`; reads the tool-options row when asked. |
| `busy_overlay` | Control | `_ready`: `_mount_busy_overlay()` (a `CanvasLayer` child of the app) | **survives** | Its own layer, outside both compositions. It re-asserts last-child on every show. Its `_phone_scale()` helper reads `DccTheme.is_phone()` at call time, so it follows the swap. |
| `_mid_stage` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_mid_running_stage` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_repaint_ms` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_repaint_t0` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_stale_timer` | Timer | `_setup_autosave` and the stale pass | **survives** | Timers that are children of the app. |
| `_stale_recompute` | Button | `set_tool_options` (line 3208, 3226) and `_wire_status` (1731) | **reset** | Nodes in the tool-options row and the status row, both rebuilt. Nulled; `set_tool_options()` and `_wire_status()`'s own paint rebuild them. |
| `_context_surface_depth` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_last_workspace_domain` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_last_workspace_mode` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_tl_expanded` | inferred | timeline fill | **survives** | UI state of the timeline strip (expanded or not; a scrub in progress, which is a drag and so quiescence). |
| `_tl_year_labels` | Array[Label] | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_state_label` | Label | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_play_button` | Button | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_fwd_button` | Button | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_speed_segments` | Dictionary | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_track` | Control | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_head` | ColorRect | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_phone_button` | Button | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_status_labels` | Array[Label] | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_detail_label` | Label | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_detail_dot` | Label | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_count_row` | HBoxContainer | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_count_parts` | Dictionary | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_bubble` | Label | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_min_label` | Label | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_max_label` | Label | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_end_hint` | Label | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_collapsed_dot` | Label | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_scrubbing` | inferred | timeline fill | **survives** | UI state of the timeline strip (expanded or not; a scrub in progress, which is a drag and so quiescence). |
| `_tl_transport` | Array[Control] | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_day_slider` | HSlider | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_tl_date_label` | Label | `_build_timeline_collapsed`, `_build_timeline_expanded`, `_build_timeline_scrub`, `_build_timeline_readout`, `_build_timeline_day_row`, `_build_timeline_layers` | **reset** | Handles into `timeline_row`, which is rebuilt. `_repaint_timeline()` rebuilds when they are gone (its own comment), so they are nulled or emptied and the repaint is called. The three dictionaries and arrays hold nodes and are cleared. |
| `_autosave_timer` | Timer | `_setup_autosave` and the stale pass | **survives** | Timers that are children of the app. |
| `_last_autosave_at` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_overlay_adjust_unsaved_told` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_overlay_slot_state` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_style_slot_state` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_style_unsaved_told` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `_quit_prompt` | AcceptDialog | lazy (first open) | **re-presented** | Created on first use, so a swap only has work for one that exists. `_quit_prompt` is modal and counts as held input (section 8). |
| `_quit_asked` | inferred | scalar state | **survives** | No node, no class. `_context_surface_depth` is a quiescence source while a context surface is open (section 8). |
| `saved_journeys_window` | AcceptDialog | lazy (first open) | **re-presented** | Created on first use, so a swap only has work for one that exists. `_quit_prompt` is modal and counts as held input (section 8). |
| `conflicts_window` | AcceptDialog | lazy (first open) | **re-presented** | Created on first use, so a swap only has work for one that exists. `_quit_prompt` is modal and counts as held input (section 8). |
| `export_dialog` | AcceptDialog | lazy (first open) | **re-presented** | Created on first use, so a swap only has work for one that exists. `_quit_prompt` is modal and counts as held input (section 8). |
| `_credits_dialog` | AcceptDialog | lazy (first open) | **re-presented** | Created on first use, so a swap only has work for one that exists. `_quit_prompt` is modal and counts as held input (section 8). |

### 2.3 Other builder callees that hold the class

These are not fields of the two files; they are what `_register_workspaces()` and the shared builders build, and they are
the largest single cost of S4.

| Object | Where | Class dependence read at its symbols | Action | Why |
|---|---|---|---|---|
| `WorldWorkspace` | `workspaces/world_workspace.gd`, built by `_register_workspaces()`, `register_workspace()` parents it to `left_dock_body` | 13 class reads (`is_tablet_portrait()` 3, `is_tablet()` 3, `phone_scale()` 3, `role_px()`); `DccTheme.watch_portrait(_on_portrait_changed)` registered once in `_build()`; `phone_fit` 2; **a one-time build of every category and row** | rebuilt | Rows bake `role_px(...) if is_tablet() else 24`-style sizes at build, and `phone_fit()` rewrites font sizes and minimum sizes with no undo (§5.2). Rebuilding from the engine's parameters is the only way to get a byte match with a cold boot (B5). Its UI state (open L2 category, the brush dictionary the tool bar reads, scroll) is carried by a state bag; the parameter values are in the engine. |
| `CivilizationWorkspace` (with its nested `InfrastructureWorkspace`) | `workspaces/civilization_workspace.gd` | 7 class reads, `phone_fit` 1; "builds the CIVIL dock once at startup" | rebuilt | Same reason. Holds a match, trade and roster refills driven by `TradeStore` statics (survive) and the bridge. |
| `CartographyWorkspace` | `workspaces/cartography_workspace.gd` | 6 `is_tablet()`, `phone_fit` 4; builds the L2 categories "once, at launch" (RF-05) | rebuilt | Same. |
| `RenderWorkspace` (inside CARTO) | `workspaces/render_workspace.gd` | 20 class reads | rebuilt | Same. Built by `CartographyWorkspace`. |
| `right_dock_ctrl` (`RightDock`) | `right_dock.gd` | **83 class reads** (`is_tablet()` 32, `role_px()` 62); rebuilds its context per selection | rebuilt | Rebuilds anyway on every selection; the mount makes it again under the new `right_dock_body` and replays the selection. |
| `DccMenus` | `menus.gd`, `menus.build()` | **tablet builds `☰ · File · World · Story`, phone and PC the seven menus**; 12 `is_phone()` reads; ~38 `PopupMenu` fields | rebuilt | See §2.2, `menus`. Quick-bar pins, accelerators and the Window menu's checked states come back from `DccSettings`, the bridge and `armed_tool`. |
| `PhoneMenu` | `phone_menu.gd` (`Control`, 3 800+ lines) | reads `menu_bar_row` and `_status_labels` as its model | freed | Phone only. |
| `ToolFan` | `tool_fan.gd` (`Node`) | 8 `phone_scale()` reads; parents an overlay onto the app | freed | Phone only; free explicitly (§2.1). |
| `RadialRing` and `ContextCard` | `radial_ring.gd`, `context_card.gd`, lazily built by `ContextBroker._ensure_ring()` / `card` | `_apply_touch_scale()` scales **only `if is_tablet()`**, and never resets | reset | Free both and null the broker's `ring` and `card`: they are rebuilt lazily and the tablet's scale then applies on first use. Re-running `_apply_touch_scale()` on a tablet to phone swap would leave the scaled values in place (the early return), which is the trap of §6. |
| `DccQuickBar` and its rule | `quick_bar.gd` (2 `is_tablet()`) | built into the tablet palette row | rebuilt | Part of `_install_tool_palette_bar()`; the phone has `phone_quick` instead (survives). |
| `ViewportHost` class reads | `viewport_host.gd` | `_scale_rule.visible = not DccTheme.is_phone()` at `_ready` (and `is_phone()` at 3 sites); `_apply_touch_scale(scale)` only from `set_safe_insets()` | survives | Re-evaluate those three reads after a swap, and call `set_safe_insets({})` equivalent on phone to tablet so the navpad insets clear (§7). |

## 3. Connections made during the build

### 3.1 The fifteen the spec counts (`node_added`, `size_changed`, `phone_insets_changed`)

Re-run with the spec's command at `bd784584`: 15 lines in 10 files (the spec said 14 in 9; `view3d.gd` arrived with S2).

| # | File, symbol | Emitter -> receiver | Class | Disconnect on unmount? |
|---|---|---|---|---|
| 1 | `dcc_shell.gd` `_ready()` | `get_tree().node_added` -> `_on_dock_node_added` | both (gated on both docks existing) | No, class-agnostic (`dock_fit()` is density-independent): connect once, and make the handler return while `left_dock`/`right_dock` are null. Re-run `_run_dock_fit()` after the mount. |
| 2 | `dcc_shell.gd` `_ready()` | `get_tree().root.size_changed` -> `_on_window_resized` | all touch | No: survives (the S3 handler is the swap's trigger). |
| 3 | `dcc_shell.gd` `_build_desktop_shell()` | `node_added` -> `_on_tablet_node_added` (`if DccTheme.is_tablet()`) | tablet | **Yes**, on tablet to phone; connect on phone to tablet. Also reset `_tablet_arb_pending`. |
| 4 | `dcc_shell.gd` `_build_phone_shell()` | `node_added` -> `_on_phone_node_added` | phone | **Yes**, on phone to tablet; connect on the other way. Also reset `_phone_fit_pending`. |
| 5 | `app.gd` `_ready()` | `node_added` -> `_watch_dialog` | all | No: hooks every `AcceptDialog`; its `_fit_dialog` returns early on the phone (`is_phone()` at call time). |
| 6 | `app.gd` `_ready()` (`if _phone`) | `phone_insets_changed` -> lambda (`viewport.set_safe_insets(phone_content_insets())`) | phone | **Yes.** An anonymous lambda: store the `Callable` in a field so it can be disconnected, or it fires on a freed composition. Phone to tablet must also clear the insets (§7). |
| 7 | `dcc_widgets.gd` `phone_window()` | `host.phone_insets_changed` -> per-window `relay` (re-runs `phone_present(dlg, host)` while visible) | phone, per window | It self-disconnects on `tree_exiting`. For a *surviving* window it must stay, and become a general "layout moved" relay that calls the window's re-present, not `phone_present` alone (§5.1). On a tablet the relay is never made (`phone_window()` returns before it), so **a window first opened after a phone to tablet swap has none, and one opened on the phone keeps a relay that is wrong on a tablet**. |
| 8 | `dcc_widgets.gd` `oversample()` | `Window.size_changed` -> `_reoversample.bind(w)` | phone windows (content scale > 1) | No: harmless on a tablet (a window keeps its own factor); `oversample()` clears itself at scale 1 by returning early, but the connection and the `oversampling_override` meta remain. The re-present path resets both when it takes a window off the phone path. |
| 9 | `new_world_dialog.gd` (phone card) | `get_parent().phone_insets_changed` -> `_fit_phone_card` | phone | Per instance, long-lived: stays connected, and `_fit_phone_card` must return when the class is not phone (confirm it does; the connection itself is guarded only by `has_signal`). |
| 10 | `open_project_dialog.gd` `setup()` (`if _phone`) | `_host.phone_insets_changed` -> lambda (`_fit_phone_content`) | phone | The connection is made only `if _phone` at `setup()`, so a window set up on a tablet never gets it. Connect always, and gate in the handler. |
| 11 | `quick_bar_menu_pin.gd` `setup()` | `node_added` -> `_on_node_added` | all | No. It will see the new `DccMenus`' popups. |
| 12 | `quick_bar_window.gd` `open()` | own `size_changed` -> `_fit_content` | tablet and PC (the phone presents it) | No; guarded by `is_connected`. |
| 13 | `vault_window.gd` | own `size_changed` -> `_on_size_changed` | all | No; `_unified()` reads `_phone or size.x < TWO_PANE_MIN_W` at call time (§5). |
| 14 | `asset_library_window.gd` | own `size_changed` -> `_fit_to_width` | all | No. |
| 15 | `view3d.gd` | `get_viewport().size_changed` -> `_fit_physical`, plus `resized` | all | No; `view3d` is a child of `ViewportHost` and survives (S2). |

### 3.2 Other connections made while building, which a swap leaves dangling

The spec's three signal names do not cover every connection a mount has to undo or re-point.

| Where | Emitter -> receiver | Class | Action |
|---|---|---|---|
| `dcc_shell.gd` `_build_phone_shell()` | `timeline_changed` -> `_refresh_phone_sim_strip` | phone | Disconnect on unmount (the receiver reads `_phone_sim_*` nodes that are then freed). |
| `app.gd` `_install_tool_palette_bar()` | `workspace_changed` -> lambda (calls `_rebuild_tool_strip()` and `_quick_bar.refresh()`) | tablet and PC | **Anonymous, and it captures `_quick_bar`.** After the tablet-to-phone free it would call `refresh()` on null. Store the Callable and disconnect. |
| `app.gd` `_ready()` | `bridge.world_loaded` and `generation_finished` -> lambdas (`_quick_bar.refresh.call_deferred()`), connected **only `if _quick_bar != null`** | tablet and PC | Same trap, and a second one: a phone boot never makes them, so **a phone to tablet swap leaves the new `_quick_bar` unrefreshed after a load**. Move them into a method connected once, null-checked. |
| `app.gd` `_ready()` | `bridge` -> `phone_quick.refresh` lambdas | all | Survive (`phone_quick` is a `RefCounted`; it no-ops off the phone). |
| `app.gd` `_ready()` | `quick_bar_window.pins_changed` -> lambda | all | Survives; it null-checks both `_quick_bar` and `phone_quick`. |
| `app.gd` `_ready()` | `tool_armed` -> `_sync_tool_strip` | all | Survives; `is_instance_valid(_tool_strip_row)` first. |
| `app.gd` `_ready()` | `workspace_changed` -> `_on_workspace_changed`, then `JourneyPlannerView` | all | The planner is rebuilt and must connect **after** `_on_workspace_changed` (its class comment). Disconnect the old planner's connection first. |
| `dcc_widgets.gd` `_desktop_window_size()` | window `visibility_changed` -> lambda (stores size), a resize grip child, meta `_dcc_sized` | desktop and tablet | Once per window, guarded by the meta. A window that goes tablet to phone keeps the grip child and the connection (§5.2). |
| every workspace's `_build()` | `DccTheme.watch_portrait` | see §4 | |

The **`viewport.setup(bridge)`**, `section_strip.setup(self)` and `resource_overlay.setup(bridge)` connections are
between survivors and stay.

## 4. Static registries and statics that hold nodes or class state

| Registry | Holds | Registered by | Action | Why |
|---|---|---|---|---|
| `DccTheme._portrait_watchers` | `Array[Callable]` | `watch_portrait()`; `WorldWorkspace._build()` once | reset | A rebuilt workspace registers again, so without a clear the array holds the dead callable and the new one. It is pruned lazily **only when the portrait axis flips** (`set_portrait`), so a swap that does not flip it never prunes. Clear in `_unmount_composition()`. |
| `DccTheme._elide_labels` | `Array[Label]` | `header(..., elide = true)` **only `if is_tablet()`** | reset | The comment (rewritten in S5) says the swap must rebuild it. Clear it on unmount; the rebuilt workspaces re-register on a tablet. |
| `DccTheme._layout`, `_touch`, `_phone_mode`, `_phone_scale`, `_narrow`, `_portrait`, `_dark` | class flags | `set_layout`, `set_touch`, `set_phone`, `set_phone_scale`, `set_narrow`, `set_portrait` | reset | Republished by the swap **before** the mount: a builder reads `is_tablet()` (`touch and not phone`) at build time. `set_portrait()` also runs the watchers and the label walk, so it must run after the clear above. |
| `DccTheme._tracked` | `FontVariation` cache | `tracked()` | survives | Keyed by spacing px; no class. |
| `DccToolBar._instance` | `DccToolBar` (`RefCounted`) | `install()` | survives | The tool bar is not a node; its output goes to `tool_options_row` and refreshes from `set_tool_options()`. |
| `LlmRegion._bar`, `_status`, `_done` | `Control`, `Label`, `Callable` | `LlmRegion.install()` while an assistant region is being painted | reset | A transient bar the assistant adds to the viewport. If one is open the swap waits (§8, `LlmRegion.painting`); otherwise both are null. |
| `LlmProfiles._app`, `_bridge` | the app and bridge | `LlmProfiles.install()` | survives | Not composition. |
| `DccSettings.profile_hook` | `Callable` | `LlmProfiles` | survives | Bound to survivors. |
| `OpenProjectDialog._meta_cache`, `_thumb_cache` | dictionaries of values | the window | survives | Image and metadata caches, no nodes. |

No other `static var` in `shell/` holds a `Node` or a `Control` (checked with `grep -rn "^static var" shell/`; `GlobalTools`'
statics hold values only).

## 5. Windows

### 5.1 The mechanism, and what a swap changes

`DccWidgets.phone_window(dlg, host)` runs once in a window's `setup()`: it sets `wrap_controls = false` and
`ok_button_text = "Close"`; on a **non-phone** host it calls `_desktop_window_size(dlg)` (saved-size restore, a resize
grip, a `visibility_changed` lambda) and `PopoutHost.prepare(dlg)` and returns false; on the phone it sets
`dlg.borderless = true`, connects the `relay` of §3.1 row 7 and returns true. `phone_present(dlg, host)` runs on every
open: it sets `content_scale_mode = CANVAS_ITEMS`, `content_scale_aspect = IGNORE`, `content_scale_factor =
host.phone_scale()`, clears `min_size`/`max_size`, `popup(Rect2i(0, 0, screen.x, screen.y - gesture - kb))`, floors the
button bar (`_floor_dialog_bar()`) and calls `oversample()`; it returns false at once on a non-phone host. The phone's own
`phone_fit()` (58 lines) then walks the content.

So a window's layout is decided in **three places**: `setup()`'s branches on `_phone` (different content), `phone_present()`
(presentation) and `phone_fit()` (edits to the content). Re-presenting needs all three undone and redone.

**The re-present contract (proposed; S4.2 builds it).** Each window grows `_represent(was_phone: bool)`. The swap calls
it after the mount for every window that is open. It must, in order: (1) capture a **state bag** (the column "State" below);
(2) take the window off the old class's presentation (`content_scale_mode/aspect/factor` back to the window's defaults,
`borderless` and `min_size`/`max_size` back to what `setup()` set on the other branch, the resize grip added or removed,
`_PHONE_FIT_META`-marked controls' fonts and minimum sizes reset by **rebuilding the content**, since `phone_fit()` has no
inverse); (3) rebuild the content for `DccTheme.is_phone()` where `setup()` branched; (4) present for the new class
(`phone_present()` or `popup_centered()`); (5) restore the state bag; (6) re-fit. Windows whose body is already **rebuilt
on every open** (`PlaceEditorWindow.open_for()` calls `_clear()` and rebuilds; `ConflictsWindow`, `SavedJourneysWindow`
and `LayersPopover` refill rows per open) need only steps (1), (4) and (5): re-run their own `open`. A window's
`_phone` field is read as a boolean at `setup()` and at every branch, so (3) also re-reads it (`_phone = DccTheme.is_phone()`).

### 5.2 Why `phone_fit()` and the phone presentation cannot be undone in place

- `phone_fit()` is **idempotent by meta flag** (`_PHONE_FIT_META = "_phone_fitted"`): it writes `add_theme_font_size_override`,
  `custom_minimum_size`, `add_theme_constant_override("v_separation", ...)`, and attaches `touch_slider`/`touch_release_button`
  wrappers, and a second pass skips every flagged control. There is no un-fit. A fitted subtree on a tablet keeps phone-sized
  fonts and minimum sizes. **Re-present on phone to tablet therefore rebuilds the content, not only the window.**
- `phone_present()` sets a window's own `content_scale_factor`; `DccWidgets.oversample()` keeps a connection and a meta on it.
- `_desktop_window_size()` is meta-flagged (`_dcc_sized`): done once, so a window that was set up on the phone and moves to a tablet
  **has never had its grip or saved size**, and one that goes the other way **keeps both**.
- A window whose `setup()` assigns `_phone` and builds once (most of them, §5.3's table) has no other rebuild path.

### 5.3 Per window

Columns: *Branches* = what `setup()` (or the build it calls) branches on; *Differs* = the content that is not the same in
the two classes; *State* = what the re-present path must carry. "P" is a long-lived window; "T" is a transient dialog
(created per call, freed on close); "L" is lazy. File paths are under `shell/`.

| Window (file, class) | Kind | Branches (`phone_window()`, `phone_present()`, `phone_fit()` lines) | Differs | State to carry |
|---|---|---|---|---|
| `faction_roster_window.gd` `FactionRosterWindow` | P | `phone_window` in `setup()`; `phone_present` in `open()`; `phone_fit` x7 (one per refill); ~20 `_phone` branches | VBox versus HBox split; the list pane is only built `if not _phone`; tap-height cells (`PHONE_TAP_MIN`); the pop-out toggle is desktop-only | `_selected` faction, `_tab` (and which of `_tab_panes` is shown), `_religion_edit_id` (an edit in progress), per-tab scroll, typed text in any field. Edits apply live: no held changes. |
| `place_editor_window.gd` `PlaceEditorWindow` | P | `phone_window` `setup()`; `phone_present` `open_for()`; `phone_fit` x1; `_phone` at 3 build sites plus the metrics helpers (row 36 or 24, tap 44 or 16, 8.4 or 4.8) | row and tap metrics; the body is **rebuilt from `_index` on every `open_for()`** (`_clear()`), so a re-present is `open_for_tab(_index, _active_tab)` | `_index`, `_active_tab`, `_sim_mode`, the name field's text, scroll. Applies live. |
| `vault_window.gd` `VaultWindow` | P | `phone_window` `setup()`; `phone_present` x5 (`open`, `open_browse`, `_warn_unsaved`, two more); `phone_fit` x6; 19 `_phone` branches | `_unified()` is `_phone or size.x < TWO_PANE_MIN_W` (one pane versus two); `ed.setup(..., _phone, ...)` puts the editor in a phone mode; row layouts | `_browse_path`, `_browse_text` against `_browse_saved` (**held: unsaved text**, `_is_dirty()`), `_editor_mode`, the search field text, the scope and entity checkbox set, scroll, the open reader. |
| `data_manager_window.gd` `DataManagerWindow` | P | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x2; 38 `_phone` branches and 9 `is_tablet()`/`is_touch()` | its own rail (`_phone_rail = _build_rail()`) against the desktop's; touch sizes | `_selected_id`, section scroll, the export form's field values (held until the action runs). |
| `asset_library_window.gd` `AssetLibraryWindow` (and its slicer sub-window `_slicer`) | P | `phone_window` `setup()` and for `_slicer` (`_slicer_phone`); `phone_present` x2; `phone_fit` x4; 48 `_phone` branches | `_cell_px = 120` on the phone, `_apply_phone_toolbar()`, the inspector as a sheet, the slicer's layout | the selected pack and asset, the collections rail selection, search text, the cell-size slider, scroll, the inspector (built once, never rebuilt under the caret), and the **slicer session (held changes)**. |
| `new_world_dialog.gd` `NewWorldDialog` | P | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x1; 33 `_phone` branches and 10 `is_touch()`/`is_tablet()` | the phone's card form (`_card_scroll`, `_card_form`, `_fit_phone_card`) against the desktop form | every input (`seed_input`, `extent_input`, `size_preset_input`, `width_input`, `resolution_input`, `grid_w_input`, `aspect_input`, `grid_h_input`, `archetype_input`, `recovery_input`), `_seed_dirty`, scroll. **Held until Create or Cancel.** |
| `open_project_dialog.gd` `OpenProjectDialog` | P | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x1; 8 `_phone` branches | gallery tiles per row, `_toolbar_row` orientation, borderless | `_search` text, `_selected`, gallery scroll. The static `_meta_cache` and `_thumb_cache` survive. |
| `phone_project_picker.gd` `PhoneProjectPicker` | phone only | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x2; 13 `_phone` tokens | exists only on the phone (the node's absence on a tablet is load-bearing, its header) | none worth carrying; on a tablet it is freed and `open_welcome()` opens `OpenProjectDialog` instead. |
| `travel_library_window.gd` `TravelLibraryWindow` | P | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x2; 14 `_phone` branches | `_rail_wrap` rail against tab buttons (`_tab_buttons`); main VBox versus HBox | `_current_kind`, `_current_id`, `_filter_text`, `_draft` and `_entry` (**held: an entry being edited**), scroll. |
| `llm_assistant_window.gd` `LlmAssistantWindow` | P | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x1; 16 `_phone` branches and 2 `is_tablet()` | padding, scroll minimums, the `edit_note` visibility, **Paint disabled on the phone** (`paint.disabled = not can or _phone`) | `_input` text (and `_pending_input`), `_task`, `_edit_facts`, `_edit_key`, `_scroll`, the transcript. A run in flight (`LlmRun.busy()`) blocks the swap. |
| `llm_settings_window.gd` `LlmSettingsWindow` | P | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x2; 15 `_phone` branches | VBox versus HBox rows, `LlmNetHint` tap minimum | `_scroll`, `_consent_open`, endpoint and key field text (**the key field is a secret: carry it in memory only and never log it**), `_ml` selection. |
| `layers_popover.gd` `LayersPopover` | P | `_phone = shell.is_phone()` at `setup()` (no `phone_window`); `phone_present` x1; `phone_fit` x4 via `_phone_fit()`; 10 `_phone` branches | a full-screen window on the phone, a popover on a tablet and the PC; rows rebuilt on every open | `_query`, scroll. Rows are rebuilt on open, so re-run `open()`. |
| `quick_bar_window.gd` `QuickBarWindow` | P | `phone_window` `setup()`; `phone_present` `open()`; no `phone_fit`; 3 branches | tablet spacing; columns re-parented into a VBox on the phone (a `HBoxContainer` cannot change orientation); controls created at `open()` only (X6) | `_search` text, the `_tabs` and `_pages` selection, `_catalogue` and `_pins_list` selection, `_show_unpinnable`. Pins apply live. |
| `quick_bar_phone_list.gd` `QuickBarPhoneList` | phone only | `phone_window` `setup()`; `phone_present` `open(es)` | the phone's pin list; the tablet has the bar itself | `_scroll`. Freed on a tablet. |
| `world_data_window.gd` `WorldDataWindow` | P | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x2; 10 `_phone` branches | `_row_cap` pagination on the phone (`shown > _row_cap`), tab layout | `_tabs` current tab, `_filter`, `_sort_by_pop`, rows shown, scroll. |
| `city_viewer_window.gd` `CityViewerWindow` | P | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x1; 11 `_phone` branches | VBox versus HBox outer, `_title_label` hidden on the phone, town-plan metrics | `_index`, plan scroll. |
| `conflicts_window.gd` (no `class_name`) | L | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x2; 10 branches | row VBox versus HBox, mark plus name | `_filter`, scroll. Refilled per open. |
| `saved_journeys_window.gd` (no `class_name`) | L | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x2; 9 branches | row layout | scroll, selection. Refilled per open. |
| `settlement_types_window.gd` `SettlementTypesWindow` | P | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x1; 6 branches | list and detail split | `_selected_id`, scroll. Applies live. |
| `generation_rules_window.gd` `GenerationRulesWindow` | P | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x1; 7 branches (`_build()` has `if not _phone` extras) | main VBox versus HBox; the sliders pane and the tables pane | `_active_preset`, pane scroll. Whether a preset holds un-applied values is not read here; confirm in its S4.5 batch. |
| `gen_info_dialog.gd` `GenInfoDialog` | P | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x2; 6 branches | the copy button's expand flag | the `_text` `TextEdit`'s scroll and selection. |
| `shortcuts_dialog.gd` `ShortcutsDialog` | P | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x2; 7 branches | row layout | `_editable`, a rebind capture in progress (**held**), scroll. |
| `export_dialog.gd` (no `class_name`) | L | `phone_window` `setup()`; `phone_present` x1; `phone_fit` x1 (`_presented_once`); 8 branches | grid columns 1 versus 2; the fit-once flag `_presented_once` | the chosen format, size and fields (held until Export); `_presented_once` must be reset. An export in flight blocks the swap. |
| `app.gd` `_credits_dialog` (`open_credits`, `_present_credits`) | L | `phone_window` and `phone_present` in the same function; `phone_fit` once | a sheet on the phone, a dialog elsewhere | scroll. |
| `dcc_shell.gd` `_desktop_search_dialog` (see §2.1) | L | none (not a `phone_present` window; the phone has the ⌕ overlay) | the dialog on a tablet and the PC, the overlay on the phone | the query text and the chosen scope chips (`_set_search_query()`). |

**Transient dialogs.** Fourteen call sites build a `ConfirmationDialog`/`AcceptDialog`, show it, and free it on close:
`DccWidgets.confirm`, `prompt` and `modal_present` (the helpers every confirmation uses), `DccBrowseDialog`,
`menus.gd`'s `_open_pack_metadata`, `_prompt_save_layout` and `_prompt_forget_layout`, `cartography_workspace.gd`'s
`_prompt_label_name`, `app.gd`'s `open_storage_locations` and `open_about`, and
`vault_window.gd`'s `open_browse`, `_warn_unsaved`, `_compare_dialog` and `_preview_dialog`. Each is a **pending decision**:
the user is mid-answer. By the spec's quiescence rule ("a window with held changes") a swap waits for them, so they need no
re-present path, only the guarantee that the wait ends when they close. **This reads the spec's two statements together
(MQ-5 re-presents open windows; quiescence waits on held input) and is an interpretation, not a quote**; if the owner means
that a confirm card must also move, each of the fourteen needs the contract of §5.1.

## 6. Code that relies on the class never changing (found by its prose)

Search: `grep -rn -i --include='*.gd'` over `shell/` for `decided once`, `once at boot`, `at boot`, `never revisited`, `never changes`,
`for the life of`, `built once`, `resize hook`, `fixed after boot`, plus every `is_phone()`, `is_tablet()`, `is_touch()`, `phone_scale()` and `role_px()`
read at build time. The first four hits of each kind were read at the symbol; many "built once" hits are unrelated (parameter tables,
asset inspectors) and are not listed.

| # | Symbol | What it assumes | Action in S4 |
|---|---|---|---|
| 1 | `DccTheme._elide_labels` / `header()` | registered "only `if is_tablet()`", because `is_tablet()` "is fixed after boot" | Clear on unmount (§4). The comment was rewritten in S5. |
| 2 | `radial_ring.gd` `_apply_touch_scale()` | computed once; scales only `if is_tablet()`; never resets | Free the ring and card on a swap (§2.3). The comment was rewritten in S5. |
| 3 | `dcc_shell.gd` `_process()` | self-disables on a non-phone boot ("a one-time self-disable rather than a per-frame branch") | `set_process(true)` on tablet to phone. Rewritten in S3. |
| 4 | `dcc_shell.gd` `_ready()` `_tl_load_layers()`, `DccTheme.set_phone/set_phone_scale/set_touch` | published once, before either build | The swap republishes `set_layout`, `set_phone`, `set_phone_scale` before the mount. |
| 5 | `dcc_shell.gd` `_compute_layout_mode()` | `_phone` written only `if not _booted` | The swap writes it; the guard stays for the in-class resize. |
| 6 | `dcc_shell.gd` `_build_desktop_shell()` | `tablet_arbitrate(self)` and the `node_added` hook only at the end of the desktop build; `tablet_fit()` "floors height only and runs once, from a deferred pass" | Both inside `_mount_composition()` (R8). |
| 7 | `dcc_shell.gd` `_ready()` | `_run_dock_fit()` after the build, `node_added.connect(_on_dock_node_added)` once | In the mount; connect once (§3.1 row 1). |
| 8 | `dcc_shell.gd` `_ready()` | `_select_domain(_active_domain)` and `_refresh_viewport_context.call_deferred()` after the build | In the mount; the deferred one needs `viewport` attached first. |
| 9 | `dcc_shell.gd` `register_workspace()` | `_refresh_phone_gen_panel.call_deferred()`, "`_phone_tab` starts at `gen`" | Re-run by the workspace rebuild; `_phone_tab` must be valid before it. |
| 10 | `dcc_shell.gd` `_compute_layout_mode()` | `LAPTOP` set via `set_narrow(size.x < W_LAPTOP_MAX)`, "decided at boot" for the PC; `_left_width`/`_right_width` overwritten on a tablet | Not S4 for the PC (spec §9). On a tablet mount the per-class widths are restored after `_compute_layout_mode()`. |
| 11 | `app.gd` `_install_tool_palette_bar()` | `if is_phone() or tool_options_bar == null: return`, once | In the mount (§2.2). |
| 12 | `app.gd` `_ready()` | `if _phone:` insets lambda; `if is_phone():` picker; `if not is_phone():` `_region_nodes` | In the mount. |
| 13 | `app.gd` `_fit_dialog()` | returns on `is_phone()` at call time | Fine: reads live. |
| 14 | `menus.gd` `build()` | **"`is_tablet()` reads `DccShell._touch`/`_phone_mode`, both written by `_ready()` before `menus.build()`"**; the menu set differs by class | Rebuilt (§2.3). |
| 15 | `dcc_widgets.gd` `phone_window()`/`_desktop_window_size()` | decided once per window in `setup()`, meta `_dcc_sized` | The re-present contract (§5.1). |
| 16 | `open_project_dialog.gd` `setup()` | `if _phone:` makes the `phone_insets_changed` connection and `phone_fit` ("**The composition is built once, so one pass does**") | Rebuilt by re-present (§5.3, §3.1 row 10). |
| 17 | `new_world_dialog.gd` | "the form is built once, so one pass is enough"; `_fit_phone_card` connection per instance | Same. |
| 18 | `quick_bar_window.gd` / `quick_bar_phone.gd` | controls and connections created at `open()` and not at `setup()` "creating controls in a never-shown window at boot moved 26 014 pixels of the phone's Cartography sheet" | Keep the lazy rule: a re-present must **not** create controls in a window that is not open. This is a pixel-identity constraint on B5's cold-boot match. |
| 19 | `journey_planner_view.gd` `setup()` | `_phone = app.is_phone()` once; caches `app.left_dock` | Rebuilt (§2.2). |
| 20 | `busy_overlay.gd` `_phone_scale()` | `phone_scale() if is_phone() else 1.0`, read live | Fine. |
| 21 | `world_workspace.gd` `_build()` | `watch_portrait()` once; `is_tablet_portrait()` baked into rows ("a section built once, at construction time, while `is_tablet_portrait()` still read false keeps `clip_text = false`") | Rebuilt (§2.3). |

A further class of readers is **safe without work because it reads at call time**: `DccTheme.role_px()` (206 call lines) reads the flags
on each call, `_pscale()/_ptap()/_pfont()` read `_phone_scale`, and `popout_host.gd` read live. Anything
that *stores* the result (a `custom_minimum_size`, a font size override, a constant) is the problem, and it is every row of §2.3.

## 7. Survivors, and how each is detached and re-attached

| Survivor | Today's parent | Detach / re-attach | Check |
|---|---|---|---|
| `ViewportHost` (`viewport`) with its camera, zoom, `_view3d`, overlay | `viewport_content` | `viewport.get_parent().remove_child(viewport)` then `new_content.add_child(viewport)` **first of the three**, preserving draw order. It is not freed, so `_ready()` is not re-run; its `resized` connections to `_update_lod` and `_apply_safe_insets` follow the node. A `Control` removed from the tree loses nothing but its layout; mouse capture (`_panning`) must be quiet (§8). | Camera zoom and centre equal to 1e-6; `_lod_active` unchanged; B5's height hash. |
| `ResourceOverlay`, `SectionStrip` | `viewport_content` | Same, after `viewport`, in the original order (HUD, then the strip). | Draw order; both keep their `setup()` connections. |
| `EngineBridge` and its world | the app | Never touched. `bridge.generating`, `busy_kind` and the world hash are the oracle. | B5's hash. |
| the debounce `Timer` (`_resize_timer`) | the shell | Never detached: it is not under a composition. | |
| `BusyOverlay` (`CanvasLayer`) | the app | Never touched; it re-asserts last-child on each show. | |
| autosave, stale and timeline `Timer`s | the app / shell | Never touched. | |
| the long-lived windows | the app | Not detached; re-presented in place (§5). | |
| `tool_bar`, `context_broker`, `paint_original`, `phone_quick`, `undo_groups`, `look_history` | objects, not nodes | Untouched, then refreshed (§2.2). | |
| `QuickBarMenuPin` | the app | Untouched; picks up the new popups. | |
| `viewport_host`'s view3d `SubViewportContainer` | `ViewportHost` | Moves with it. `view3d.gd` is sized in physical px (S2) and re-fits on `resized`. | |
| `tool_group`, `DccSettings` caches, `TradeStore`, `VaultStore` | statics | Untouched. | |

**Not survivors, although they look like it:** the three workspace panels (§2.3) are children of `left_dock_body`, so they leave with
the dock; the spec §3.5 does not list them as survivors and this inventory agrees, because they carry the class. `right_dock_ctrl` likewise.

## 8. Quiescence: what holds unconfirmed input, and the symbol that reports it

A swap waits while any of these is true, with the status line saying why (spec §3.5). Each symbol was opened.

| Source | Reported by |
|---|---|
| a generation, import or landmark pass | `bridge.generating`; `bridge.busy_kind`; the busy overlay is visible while it holds |
| a synchronous busy action | `BusyOverlay.busy_run` / `DccApp.busy_run` (`busy_overlay` visible) |
| an armed tool with a live draft | `armed_tool != "inspect"`; `bridge.paint_draft_count() > 0`; `bridge.sculpt_stamp_count() > 0` (`RightDock._draft_stack_live()`); the way draft of `infrastructure_workspace._push_way_draft` |
| an open undo group | `undo_groups.is_open()` (`undo_group.gd`) |
| a map drag or pan | `ViewportHost._panning`; `map_overlay.gd` `_pin_drag_active` |
| a dock drag | `DccShell._dragging_dock != ""` |
| a phone sheet drag, or a held undo chip | `_phone_sheet_drag` (non-empty), `_undo_chip_down` |
| a timeline scrub | `app._tl_scrubbing` |
| an assistant region being painted, or a run in flight | `LlmRegion.painting`; `LlmRun.busy()` |
| a vault edit not yet written | `VaultWindow._is_dirty()`; `markdown_editor.is_dirty()` |
| a form open for confirmation | a visible `NewWorldDialog`, `ExportDialog`, the shortcuts rebind capture, any of the §5.3 transient dialogs (`Window.visible` of a `ConfirmationDialog`) |
| the quit prompt | `app._quit_prompt` visible / `_quit_asked` |
| the slicer session | `AssetLibraryWindow._slicer` visible |
| a context surface | `app._context_surface_depth > 0` |

Two issues. **(a) There is no single `is_quiescent()` today**: the symbols are scattered and the dirty/held ones live in windows,
so S4.1 needs one function on the shell that asks a `quiescent_blockers() -> Array[String]` of each (a `has_method` probe on
the windows, the way `phone_present` already probes `host`), so the status line can name the blocker. **(b) A window that holds
input makes the swap wait, but under Ruling EG the swap happens during a user's own split-screen drag**, so a form left open in a
window must not freeze the layout indefinitely: the swap should wait *with a visible reason* and run when the form closes, and
never discard (spec). Whether the wait should instead degrade to "re-present with the state bag" for held-input windows is an owner
question (§10).

## 9. Counts, per action and per file

| File | Fields | survives | rebuilt | reset | freed | re-presented | Other |
|---|---|---|---|---|---|---|---|
| `shell/dcc_shell.gd` | 134 | 24 | 28 | 19 | 58 | 5 | 2 connections to undo (§3.1 rows 3, 4) |
| `shell/app.gd` | 100 | 36 | 9 | 32 | 0 | 23 | 1 lambda to store (§3.1 row 6) and 3 to re-point (§3.2) |
| `shell/dcc_theme.gd` | 2 registries + the flag set (§4) | 1 (`_tracked`) | 0 | 3 | 0 | 0 | |
| `shell/menus.gd` | 72 (about 38 `PopupMenu`) | 0 | 72 | 0 | 0 | 0 | `DccMenus` replaced as a whole |
| `shell/right_dock.gd`, `workspaces/*.gd` | 4 workspaces + right dock | 0 | 5 | 0 | 0 | 0 | the largest cost (§2.3) |
| `shell/dcc_widgets.gd` | helpers, no fields | | | | | 3 helper sites (`confirm`, `prompt`, `modal_present`) | `phone_window`, `phone_present`, `oversample`, `_desktop_window_size` get an inverse |
| the 26 re-presented window files (§5.3) | | | | | | 26 | one `_represent()` each |

By the spec's own counts the 28 files with `phone_present()` are the 24 files of §5.3's rows (22 windows and the two phone-only ones; `app.gd` is there for its credits sheet), plus `menus.gd`,
`dcc_widgets.gd`, `workspaces/cartography_workspace.gd` and `browse_dialog.gd` (transient sites). `phone_present(` per file: `vault_window` 5, `app` 4, `dcc_widgets` 4, `menus` 3,
`asset_library_window` 2, and one in each of the other 23 (41 total).

## 10. Risks, and questions for the owner

| # | Risk | Where | Containment |
|---|---|---|---|
| X1 | **The workspaces bake the class and `phone_fit()` cannot be undone**, so a swap needs a workspace rebuild with a state bag. That is a bigger change than the spec's "survivors" list implies (it lists the camera, the bridge and the overlays; not the 17 000 lines of workspace UI) | §2.3 | S4.2 builds the rebuild first; B5's cold-boot match proves it. If the rebuild costs more than a restart, B5's timing says so. |
| X2 | Anonymous lambdas connected to survivors' signals that capture a field a swap nulls (`_quick_bar`, `phone_insets_changed`, the `workspace_changed` palette lambda) | §3.2 | Replace with stored `Callable`s; B5 counts connections and `SCRIPT ERROR`. |
| X3 | `_status_labels` is a "last registered wins" dictionary, and the phone's order is load-bearing | §2.1 | The mount rebuilds in the original order; a probe compares `top_world`'s owner after a swap. |
| X4 | Held-input windows can make the swap wait under a split-screen drag | §8 | Owner question Q-S4-2. |
| X5 | `phone_present()` measures against `host.get_viewport_rect()` and `phone_scale()`; both change at the swap and the order matters (`_apply_unit()` before the re-present) | §5 | The swap order of §11, step 6. |
| X6 | Pixel identity on the phone: the lazy-creation rule of `quick_bar_window` and `quick_bar_phone` (26 014 px) | §6 row 18 | The re-present path must create nothing in a window that is not open. |
| X7 | The ring and the card keep tablet scale after a tablet to phone swap | §2.3 | Free, do not re-run. |
| X8 | Two rulings on the line: EG is **shortest width 900 dp**, DY MQ-1 (c) is **width 600 dp, landing with MS-7**. Rotation swaps on one and not on the other | §0 | Q-S4-1, answered: EG governs (Ruling EG addendum, 2026-11-08). |
| X9 | `_tl_*` handles and `timeline_row`: the journey planner takes `timeline_row` over, so the "rebuild when the handles are gone" guard must be exercised with the planner open | §2.2 | B5 runs once with the planner open. |
| X10 | A phone-class window keeps a `relay` on `phone_insets_changed` that calls `phone_present`, which returns false off the phone, so a relay left on a window that moved to a tablet is harmless but a window first opened on a tablet then moved to the phone has none | §3.1 row 7 | Make the relay general (§5.1). |

**Owner questions (defaults).** Q-S4-1: with Ruling EG (900 dp *shortest* width) and Ruling DY MQ-1 (c) (600 dp *width*, landing with MS-7)
both on record, which governs the touch line, and does a rotation swap? Default: EG governs the phone/tablet line (it is the later ruling and is
orientation-independent); DY's width classes (`compact`, `medium`, `expanded`...) stay the *within-tablet* classes that MS-7 reads. **Answered 2026-11-08 (Ruling EG addendum): the default stands; EG governs.** Q-S4-2: a held-input
window (New world with typed values, the vault with unsaved text) during a class change: wait with a visible reason (the spec), or re-present with the state bag
as for any other window? Default: wait, because the spec says nothing is discarded, and add the bag path only if users hit the wait. Q-S4-3: transient confirm
dialogs: wait (this file's reading) or re-present? Default: wait.

## 11. The S4 implementation plan, in order

1. **S4.1 quiescence and the dp-free scaffolding.** `DccShell.quiescent_blockers()` (§8), `_defer_swap_until_quiet()`, and the status line. No swap yet. Probe: arm a tool with a draft; `_request_class_swap` returns the blocker and does not swap.
2. **S4.2 the mount path.** Split `DccShell._ready()` into: boot classification, `_mount_composition()` (build the composition, attach the survivors, the hooks of §3, `_run_dock_fit()`, `tablet_arbitrate`, `_select_domain`, deferred viewport context) and `_unmount_composition()` (§2.1 *freed*, the §3 disconnects, §4 clears, `Tween` kill, field nulling, detach of the survivors **first**). Split `DccApp._ready()` into class-independent (stays) and `_mount_app_composition()` (§2.2's list). Boot calls the same pair: **a cold boot goes through the mount path**, so a swap cannot diverge from it. Probe: boot twice and diff the composition dump; no behaviour change at this step (B2 captures byte-identical).
3. **S4.3 the workspace rebuild with a state bag** (§2.3): `WorldWorkspace`, `CivilizationWorkspace`, `CartographyWorkspace`, `right_dock_ctrl`, `DccMenus`. Order matters: clear §2.2's handler dictionaries, rebuild, re-register, `_wire_selection()`, re-select. Probe: rebuild twice on the same class and check no orphan growth and an identical dump.
4. **S4.4 `_request_class_swap()` becomes the swap**: quiescence gate, `_unmount_composition()`, republish `DccTheme` flags and `_apply_unit()`, `_mount_composition()`, restore. Probe: B5's class sequence without windows.
5. **S4.5 the window re-present contract (§5.1) and the helpers' inverse**: `DccWidgets.phone_window`/`phone_present` gain `unpresent()`; the shared relay; then per-window batches, in this order (shared state first, riskiest last): (a) the *refill-per-open* windows `PlaceEditorWindow`, `ConflictsWindow`, `SavedJourneysWindow`, `LayersPopover`, `GenInfoDialog`, `WorldDataWindow`, `SettlementTypesWindow`, `QuickBarPhoneList`/`QuickBarWindow` (re-run `open`); (b) windows with one build and a `_phone` branch: `CityViewerWindow`, `GenerationRulesWindow`, `ShortcutsDialog`, `ExportDialog`, `TravelLibraryWindow`, `OpenProjectDialog`, `PhoneProjectPicker`; (c) the large ones: `FactionRosterWindow`, `VaultWindow`, `DataManagerWindow`, `LlmSettingsWindow`, `LlmAssistantWindow`, `AssetLibraryWindow` (+ slicer), `NewWorldDialog`.
6. **S4.6 the ring, the context card and the remaining §6 rows**, then B5's full probe and the timing.

The swap's order of operations inside S4.4: quiescence, detach survivors, unmount (disconnect, clear, free), `DccTheme.set_layout/set_phone/set_phone_scale/set_portrait`, `_apply_unit()`, `_compute_layout_mode()` (widths), mount (build, attach survivors, connect, fit), `app` mount, re-select the domain and node, **re-present the windows** (they read the new `phone_scale()`), restore per-class state.

## 12. The B5 probe outline

A new `_ms1class_probe.gd --swap` (extends the file the spec names), windowed, `--force-touch --density 2` (a Ruling EG-valid pair: phone at 360 x 780 dp, tablet at 900 x 1 200 dp). Steps per the spec's §1 sequence, plus:

1. Open **three windows** before the first swap: `FactionRosterWindow` on a chosen tab (type text in a field), `PlaceEditorWindow` on a chosen tab, and `VaultWindow` with unsaved text (that one holds input, so with Q-S4-2's default the swap must *wait*, and the probe asserts the wait and the reason, then saves and asserts the swap runs). Plus one without held input for the "re-presented" leg (the roster).
2. After each step: class from `DccTheme.classify()`; `_phone_root` valid iff phone; both docks valid iff tablet; the structural dump (path, class, visibility, rect to 1 dp) equal to a **cold boot** at the same size and density with an empty allow-list; the height-field hash through the bridge unchanged; camera zoom and centre equal to 1e-6; domain and node unchanged.
3. For each of the three windows after each step: still visible; its layout dump equals a fresh open of the same window in the new class; its state kept (the tab, the typed text, the scroll position, within a pixel).
4. **Orphan and connection counts over ten round trips**: `Performance.OBJECT_ORPHAN_NODE_COUNT` before the first swap and after the tenth equal; the connection counts on `root.size_changed`, `tree.node_added`, `phone_insets_changed`, `timeline_changed`, `workspace_changed`, `bridge.world_loaded` and `bridge.generation_finished` equal (listing each, since §3.2's lambdas are on the last four); `DccTheme._portrait_watchers.size()` and `_elide_labels.size()` equal; `SCRIPT ERROR` absent from captured stderr.
5. The hysteresis legs of B5, adjusted to Ruling EG's line: 900 x 1 600 dp stays a tablet at 890 x 1 582, a phone at 880 x 1 564, still a phone at 900 x 1 600, a tablet at 920 x 1 636; and a **16:10** window (`1 280 x 800` dp) swaps at 880 x 550 to a phone, the case the aspect gate used to hide.
6. Timing: swap versus cold boot, median (min..max) of five serial runs, alone.
7. Mutation: skip each of §3's disconnects, one at a time, and the orphan/connection leg must go red; skip each §4 clear likewise.

The probe **must fail on the code before S4.2**: with the swap request still logging only, B5's class check is red.

## 13. What would make this inventory wrong

It is read from the code at `bd784584` on 2026-11-07. A field added, a window opened from a new place, or a `phone_present()` call in a file not
listed changes the counts; re-run the commands of the spec's §6. The window *state* column is the weakest part: it was read from member names, not
from each window's save/restore code, because no re-present path exists to read. Each S4.5 batch re-reads its window at the symbol before writing its `_represent()`.

## 14. Addendum, 2026-11-08: what landed after this inventory (PC-1 to PC-8, VE-0 to VE-5)

Written for stage S4.1 against `996cb307`, read at the symbols named. This pass re-inventories what **S4.1
(quiescence)** needs in full; the rows S4.3 and S4.5 need are listed with what each must re-read at its symbol
before coding, not yet given actions.

### 14.1 New sources of held input (S4.1, built)

| Source | Reported by | Where S4.1 reads it |
|---|---|---|
| PC-7's held changes in a context window: the Place editor's unsubmitted event (`place_editor_window.gd::held_changes()`, any of `_ae_end`, `_ae_name`, `_ae_desc`, `_ae_color`, `_ae_group` non-blank) and the faction roster's open religion editor (`faction_roster_window.gd::held_changes()`) | the window's own `held_changes()` (`context_window.gd`'s contract: a reason, or `""`) | `DccShell._collect_window_blockers()`, for a visible window with no `quiescent_blockers()` |
| PC-6's docked assistant: on the desktop and the tablet the conversation lives in the Context panel's Ask tab while `LlmAssistantWindow` stays hidden (`llm_assistant_window.gd`, "Where it lives"), and a reply goes on streaming there | `LlmAssistantWindow.quiescent_blockers()` (`run.busy()`) | `DccApp._app_quiescent_blockers()`, asked directly when the dialog is hidden (the walk asks only visible windows) |
| VE-4's autosave: a note now writes itself on a timer, so `VaultWindow._is_dirty()` is true only between an edit and the next write, or after a write that could not land (a conflict, a deleted or read-only file) | `VaultWindow.quiescent_blockers()` (`_is_dirty()`) | the walk |

Not held input, so not a blocker (their state is S4.5's to carry): the PC-7 context windows' FOLLOW/PINNED
state; the PC-8 planner window (a frame whose slots `JourneyPlannerView` fills; nothing in it waits for a
confirm); the Ask tab's typed draft (the content is reparented, never freed, so a swap cannot lose it); VE-5's
side panel (`vault_side_panel.gd`, its selected `tab` and `_tag_open`).

### 14.2 For S4.3 and S4.5 to re-open at their symbols

- `right_dock.gd`'s Context panel (PC-4/PC-6): `_tab`, `_ask_frame`, `_ask_body`, and the Ask frame "added once
  to the dock column right after the scroll". A rebuilt `right_dock_body` drops that frame; the assistant's
  content (`present_in()`) must be moved out before the old dock is freed, or it is freed with it. PC-10 (running
  in parallel, 2026-11-08) adds Context panel modules and a per-user store to the same file.
- `context_window.gd` (PC-7, a `RefCounted`): `make_context()` adds a FOLLOW/PINNED chip and a
  `selection_changed` connection per window, on the desktop form only (`applies()` false on phone and tablet).
  A swap between the tablet and the phone never changes `applies()`, but a re-present must not add a second chip.
- `context_key_relay.gd` (PC-7, a `Node` child of each non-blocking window): survives with its window.
- `planner_window.gd` (PC-8): desktop only; `JourneyPlannerView.setup()` builds it once and parents its panels
  into five slots. The inventory's §2.2 row for `journey_planner_view` (rebuilt) now has a window to rebuild into.
- `vault_side_panel.gd` (VE-5): `phone` is set at `setup()` and read by the panel's build, so it is one more
  piece of the vault window's re-present path (§5.3's `VaultWindow` row).

## 15. Addendum, 2026-11-08: S4.2, the mount path (what `_ready()` builds now, and the ordering it kept)

Written for stage S4.2 against `5c1d178f` (PC-1 to PC-10, VE-0 to VE-7, MS-2 to MS-4 and MS-3's remainder landed
since §2), read at the symbols named. Verified against a structure and connection dump of the real shell
(`_ms1swap_probe.gd --leg dump`, below), not against this prose.

### 15.1 What `_ready()` builds now that §2 does not list

- `DccShell._ready()` also connects `_on_window_node_added` (the touch-scroll pass over every window), and only
  `if _phone or DccTheme.is_tablet()`. It is not undone by `_unmount_composition()` (it is class-agnostic once
  connected), so S4.4 owns the disconnect when a swap lands on the desktop and the connect when it leaves it. S4.2 guards
  the connect with `is_connected()` so a second mount adds nothing.
- New composition fields (MS-3, MS-2): `_phone_dom_host`, `_left_dock_mode_switch`, `_left_dock_col`, `_dock_in_sheet`,
  `_phone_dom_back_row`, `_phone_dom_journeys_row`, `_phone_map_row`, `_domain_fan` (built lazily under `_phone_root`),
  `_dom_gesture`. Nulled or reset by `_unmount_composition()`. `_dock_in_sheet` true at unmount time is harmless: the dock
  pieces that were reparented into the sheet are freed with the sheet, and the left dock itself with the root.
- Fields §2.2 lists that no longer exist: `_tool_options_stale` and `_tool_options_bake` (PC-11 removed the tool-options
  row's buttons). `_stale_recompute` still exists and is nulled.
- Nodes that are children of the app and not of the composition, built by `DccApp._ready()` and freed explicitly by
  `_unmount_app_composition()`: `right_dock_ctrl` (a `Node`; PC-4's Context panel, PC-5's Note tab, PC-6's Ask frame and
  PC-10's modules live under `right_dock_body` and go with the dock), `journey_planner_view` (PC-8's planner window is
  parented by it), `phone_project_picker`, and the phone's `ToolFan` and its overlay (`ToolFan.teardown()`, new).
- **Connections a build makes to a survivor with a lambda the shell owns**, which §3.2 did not list and which a second
  mount would double (found by the round trip's connection counts, see §15.4): `_wire_menu_squares()` (four bridge signals and
  `look_history.changed`, from `_build_menu_bar()`), `_wire_phone_undo_chip()` (the same five, phone), `_wire_phone_overflow()`
  (`bridge.project_saved`, phone). `_wire_status_progress()` already carries a once-guard (`_status_wired`).

### 15.2 The ordering dependencies, and what S4.2 did with each

`DccApp._ready()` interleaves composition-dependent steps (C) with class-independent ones (I). The mount call replaces C
steps scattered between I steps; these are the dependencies that decided what could move.

| # | Dependency | Verdict |
|---|---|---|
| O1 | **The workspaces connect to the Place editor and the Factions window while they build** (`civilization_workspace.gd`: `place_editor_window.place_changed`, `faction_roster_window.roster_changed`, ...), and read `new_world_dialog` and `layers_popover` in handlers. | The windows must be set up before `_register_workspaces()`. So the C block cannot move ahead of the windows; the windows' `setup()` calls stay where they were and the **mount call sits after `layers_popover.setup()`**. |
| O2 | No window's `setup()` reads `menus`, `_quick_bar`, `left_dock`, `tool_options_bar` or `timeline_row` (grepped over all twenty; the one reader, `ShortcutsDialog`, reads `_app.menus` when it opens). `QuickBarMenuPin.setup()` hooks every popup already in the tree and then every later `node_added` (`hook()` is idempotent). | **`menus.build()` may move after the windows.** Its popups are then hooked by the `node_added` branch instead of the walk. |
| O3 | `_install_tool_palette_bar()` connects the `workspace_changed` lambda that rebuilds the tool strip. Order of `workspace_changed` receivers is `palette lambda`, `RightDock`'s, `_on_workspace_changed`, the planner's. | Kept: the palette still precedes all three (it moved from before the bridge to after the windows; nothing connects to `workspace_changed` in between: grepped). `_quick_bar.setup.call_deferred()` still lands after `_ready()` returns, by which time the menus exist. |
| O4 | The bridge lambdas that refresh the quick bar were connected only `if _quick_bar != null`, which was true only because the palette was built before the bridge. | The palette is now built after them, so the guard would have skipped them on every class. They are connected unconditionally and call `_refresh_quick_bar_deferred()`, which reads `_quick_bar` at call time and is null-safe. Phone gains two no-op connections (`world_loaded`, `generation_finished`); desktop and tablet keep their count. |
| O5 | `viewport` and `resource_overlay` are made and added to `viewport_content` before `viewport.setup(bridge)` between the two repaint-bracket probes (`_repaintbracket_probe.gd` walks `generation_finished`'s connections in order). | They stay where they are: survivors are created once, at boot, in `_ready()`. `_attach_survivors()` re-parents them on a remount and does nothing at boot. `section_strip` likewise (after the mount, before `_wire_status()`). |
| O6 | `_wire_status()`, `_wire_selection()`, `GlobalTools.install()`, `PaintOriginal` and `DccToolBar.install()` come after the workspaces and before `workspace_changed` is connected; the Paint row reads `WorldWorkspace`'s brush dictionary. | Not in the mount: they connect once to survivors (`_wire_status()` has no guard) or build survivor objects. A remount refreshes what they painted (§15.5), it does not call them again. |
| O7 | `workspace_changed.connect(_on_workspace_changed)` precedes the planner (its class comment). | Kept. `_mount_journey_planner()` is the planner's builder, called from `_ready()` in the same place and from the round trip. |
| O8 | `DccShell._ready()`'s root `size_changed` connection used to sit between the dock hooks and `_select_domain()`. | It now follows `_mount_composition()`. Nothing the build or `_select_domain()` does connects to or emits that signal, and the dump shows the root's `size_changed` list unchanged. |

**What moved, measured:** with the pre-split tree (`5c1d178f`) and this tree each booted four times (desktop 1600 x 1000,
touch tablet 1800 x 3200 at density 1, phone 1080 x 2340 at 2.8125, tablet 2388 x 1668 at 2) and dumped (class, script,
visibility and rect of every node under the app; every signal's receivers in connection order), the two runs of one tree
are byte-equal and the trees differ in exactly three ways: (1) `menus.gd`'s lambdas on `bridge.generation_finished` and
`world_loaded` sit after the windows' instead of before them (O2); (2) on the phone, the picker window is the app's child
after `layers_popover` instead of before `vault_window`, and the insets relay is last on `phone_insets_changed` (the
mount); (3) the phone's two no-op quick-bar connections (O4). The structure is otherwise identical line for line.

### 15.3 What S4.2 built

- `DccShell._mount_composition()` and `_unmount_composition()` (`dcc_shell.gd`). `_ready()` keeps the classification, the
  resize connection and `_booted`, and calls the mount, so a cold boot takes the path a swap will. The unmount frees
  `_phone_root` or `_desktop_root` (new field), the `ToolFan` and its overlay (`ToolFan.teardown()`), disconnects
  `_on_tablet_node_added`, `_on_phone_node_added` and `_refresh_phone_sim_strip`, kills the sheet's `Tween`, nulls every
  field into the composition and clears every dictionary and array of its nodes (§2.1's *freed* and *reset* rows, plus
  the MS-2 and MS-3 fields of §15.1), and calls `DccTheme.clear_composition_registries()` (new: empties
  `_portrait_watchers`, prunes freed labels from `_elide_labels`).
- `DccApp._mount_app_composition()`, `_unmount_app_composition()`, `_attach_survivors()` and
  `_mount_journey_planner()` (`app.gd`); the order is §15.2's. The unmount detaches the survivors first, drops the
  connections the old `DccMenus` and the Cartography workspace's `ColourPaintTool` made to survivors
  (`_drop_connections_to()`, a walk over every signal of each emitter, so no list of what they connected is kept;
  `DccMenus.release_survivor_connections()` for the one lambda whose receiver reads as a script), frees the planner,
  `RightDock` and the picker, and clears the six handler dictionaries, `_domain_tools` and the timeline handles.
- `_remount_composition_for_test()`: unmount, mount, and the boot steps a fresh composition needs. **Test-only: nothing
  calls it but `_ms1swap_probe.gd --leg roundtrip`.** It republishes no class flag, restores no per-class state and
  re-presents no window; that is S4.3 to S4.5.
- Small changes the round trip forced, each a bug a second mount would have shipped: `_select_domain()` does not emit
  `workspace_changed` while a mount is running (`_mounting_composition`; the app's handler read a `right_dock_ctrl`
  that did not exist yet); the palette bar's `workspace_changed` relay is a field that is disconnected on unmount;
  `_wire_menu_squares()`, `_wire_phone_undo_chip()` and `_wire_phone_overflow()` connect once (§15.1);
  `JourneyPlannerView._exit_tree()` frees the window and the layer popup it parented to the app;
  `WorldWorkspace` frees the tablet slider rows' unmounted shapes when it is freed (568 orphan nodes after three round
  trips at 1 800 x 3 200 dp against 142 at boot); `_build_stale_recompute()` is split from `_setup_staleness()`;
  `status_snapshot()` and `restore_status()` carry the status text and ink across (`set_status()` now remembers the
  token it was last given).

### 15.4 What the round trip measured

`_ms1swap_probe.gd --leg roundtrip` (windowed, wiped user dir): three unmount and mount rounds at the same class, then
11 checks. Expected values are literals or the boot's own figures; positive controls assert that the old docks,
`RightDock` and root were freed and that the survivors are the same instances. Desktop 1 600 x 1 000, the tablet
(`--force-touch --vp 1800x3200 --density 1`) and the phone (`--vp 1080x2340 --density 2.8125`) all pass:

| Check | Bar |
|---|---|
| orphan nodes after three round trips | equal to boot's (0, 0, 1) |
| node count (the phone picker's lazily built content left out) | equal to boot's |
| receivers of every signal of the tree, the root, the app, the bridge and the viewport | the same multiset; order discarded, a rebuilt receiver comes last |
| `DccTheme._portrait_watchers` and `_elide_labels` | the same size (1 and 98 on the tablet) |
| the composition's structure: class, script, visibility and rect of each visible control, as a sorted multiset | equal to boot's |

What it found, in the order it found it (every one fixed above): `workspace_changed` emitted into a half-built shell, an
app-owned lambda that survived the bar it captured, five lambdas the shell connects to the bridge on every build, one
on `viewport.view_mode_changed` and nine on bridge signals from `DccMenus`, three planner popups and windows parented to
the app, 426 orphan slider rows on the tablet, and a status bar and stale button that were not refilled.

### 15.5 What S4.4 must still do (not built; the round trip does not hide it)

- **Re-derive survivor-driven state after the mount**: only the status text is carried (`status_snapshot()`). The tool-options
  row (`set_tool_options()` from `armed_tool`), the Edit menu's undo rows, the Window menu's checked states, the
  domain rail's selection, the phone's sheet detent and tab, the timeline's expanded state and the right dock's
  selection replay all read survivors and are rebuilt to their boot defaults today.
- **Per-class widths and collapse flags** (§2.1), the swap's `DccTheme` republication before the mount, `_apply_unit()`,
  `set_process(true)` on a tablet to phone swap, and the `_on_window_node_added` disconnect or connect.
- **The probe cannot see a first-run surface** (the phone's coach toasts, the picker's opened content): the leg waits the
  toasts out and leaves the picker's subtree out of the comparison. A real swap must decide whether a toast in flight
  is quiescence (§8) or is dropped.
- **Rows of §2.2 not exercised by a same-class round trip**: the phone to tablet half of the palette bar, the picker
  and the insets relay (each is built or freed only when the class differs); the mutation harness proves the
  disconnects on the class that has them.

## 16. Addendum, 2026-11-09: S4.3, the cross-class remount and the state it carries

Written for stage S4.3 against `950946f3`, verified by `_ms1swap_probe.gd --leg carry|crossclass|norm`, not by this prose.

### 16.1 Who holds each carried item, and the action

| Item | Held by | Action across a swap |
|---|---|---|
| Open accordion category per workspace panel | the panels (rebuilt) | **memory**: `DccApp._app_memory["open"]`, keyed by the panel's path under `left_dock_body`; restored by `open_category()` after the mount |
| Scroll of the left and right docks; of the phone sheet and its generate column | `ScrollContainer`s (rebuilt) | **memory**, per class (`DccShell._class_memory`); written two laid-out frames after the mount |
| Tool-options row | rebuilt by the `tool_armed` listeners | **re-derived**: `armed_tool` survives; `_complete_arm()` tells the new listeners (not `arm_tool()`, which no-ops on the armed id and buzzes) |
| Domain rail, current domain, each domain's mode | `_active_domain`, `_domain_mode` (survive) | re-derived by the mount's `_select_domain()` |
| Selection and its history | `DccApp.selection`, `_selection_back`, `_selection_forward` (survive) | replayed to every surface by `_apply_selection()` after the pin is restored |
| Timeline year | the engine's cursor (survives) | none: the strip is filled from it. A `timeline_changed` emit was tried and removed: it ran the strip's no-world gate a second time |
| Context panel pin, subject, tab | `RightDock` (rebuilt) | **memory**: `RightDock.export_state()` and `import_state()`, only from and to a class with the panel's chrome |
| Context panel arrangement (which tabs) | `DccSettings` (on disk) | none |
| Phone sheet detent and tab | `DccShell._phone_detent`, `_phone_tab` | **memory**, per class; the tab is written after the mount's `_select_domain()`, which derives one from the domain |
| Dock widths and collapse flags (WI-04) | `_left_width`, `_right_width`, the flags | **memory**, per class; the widths before the mount (the build reads them), the collapse through `_toggle_dock()` after it |

### 16.2 The order, and why

Capture, unmount, republish the class (clear `DccTheme`'s phone flag, phone scale and layout first: `role_px()` reads them
inside the classification), restore the dock widths, mount, `_on_workspace_changed()`, planner, stale button, then the
state: accordions, **the tool re-armed before the panel's state** (arming a measuring tool makes the panel show its reading,
which releases a pin and returns to Inspect, so a pin restored first would be undone), the panel's pin and tab, the
selection replay, the status text, and last the collapse, tab, detent and scrolls.

### 16.3 What the comparison with a cold boot excludes

Long-lived windows and their static-lambda relays (S4.5); the `ViewportHost` survivor's chrome (`_layers_btn`'s size, the scale
rule, the map control row are built for the boot class: S4.4, in a file MS-5 is changing). Touch is hardware: the viewport
reads it once, so the swap chains keep it. Workspace tool settings held in fields (brush, erode, bake) are not carried.
*S4.4 (section 17) refreshes the viewport's chrome and carries the tool settings and the Ask conversation; this section is as written on 2026-11-09.*


## 17. Addendum, 2026-11-09: S4.4, the swap itself (what changed, what it carries, what it waits for)

Written for stage S4.4 against `a61eb9e6`, verified by `_ms1swap_probe.gd --leg live`, not by this prose.

### 17.1 What S4.4 built

- **`_run_class_swap()` performs the swap.** `DccShell._request_class_swap()` (S4.1: it waits for `quiescent_blockers()` and is
  cancelled if the size comes back) now calls a coroutine, `_run_class_swap()`, which calls `_perform_class_swap()`; `DccApp`
  overrides that with `_remount_composition()`, the S4.3 test path promoted to the real one (the `_for_test` suffix is gone).
  A bare `DccShell` (the phone-chrome probes) keeps its boot class. `_swap_running` spans the swap and is itself a blocker
  ("a layout change is running"), so a resize that arrives mid-swap is classified after it lands. `class_swap_runs`,
  `last_swap_ms` and `last_swap_sync_ms` are the probe's counters.
- **The viewport survivor's chrome is rebuilt.** `ViewportHost._ready()`'s chrome block is now `_build_chrome()` (moved unchanged:
  the scale bar and rule, the readout and coordinate labels, the Layers button, the context chip, the navpad), and
  `rebuild_chrome()` frees those nodes and calls it again, resets `_safe_insets` and `_touch_scale`, and keeps the pan latch
  (without the toast an arming would show). A cold boot and a swap therefore take one path. The camera, the tiles and the
  overlays are not touched. `DccApp._remount_composition()` calls it after the app's mount; a phone's `set_safe_insets()` then
  arrives through the existing deferred relay.
- **Workspace tool settings are carried.** `Workspace.tool_state_keys()`, `export_tool_state()` and `import_tool_state()`;
  `DccApp._app_memory["tools"]`, written at capture and handed to the new workspace BEFORE its `setup()`, so the build draws
  the carried values. Carried: `WorldWorkspace` (`_paint_layer`, `_paint_brush`, `_rock_layer_last`, `_erode_op` with
  `_erode_op_seeded`, `_sculpt_symmetry`, `_sculpt_pressure_on`, `_pg_mode`, `_pg_open`), `CivilizationWorkspace`
  (`_territory_radius`), `CartographyWorkspace` (`_icon_brush_r`, `_icon_brush_density`, and `ColourPaintTool`'s material,
  colour, blend, radius, hardness, opacity, land-only and heat settings). **Not carried, because a survivor holds the truth:**
  the Finalize bake depth (`DccSettings.bake_depth()`; `_bake_depth` is its mirror, re-read at build) and the engine's copy of
  the paint brush. The first draft of the probe wrote the mirror field and failed: the setting is the source.
- **The Ask conversation survives.** `LlmAssistantWindow.release_dock()` gives the content (`_host`, the session's transcript,
  cards and draft) back to its window before the Ask tab's body is freed, and the remounted tab's `present_in()` takes it
  again. `DccApp._unmount_app_composition()` also releases a docked vault note (`VaultWindow.release_dock()`; the vault's own
  `quiescent_blockers()` already makes a swap wait while text is unsaved).
- **Dock widths are remembered per orientation.** `DccShell._dock_widths` is keyed `"<class>:<land|port>"`.
  `_on_tablet_resized()` puts the widths away under the orientation it is leaving before it re-lays out, and the capture and
  restore use the same key. Found by the live leg: a resize that crosses the line flips a tablet's orientation first
  (WI-04 resets the widths there) and swaps after the debounce, so the S4.3 capture read the portrait default (232) and the
  next landscape tablet drew a 232 px dock.
- **Open windows hold the swap, with their name.** `_collect_window_blockers()` reports any visible window that reported nothing
  of its own as "the X window is open (close it to change the layout)", and an open menu as "a menu is open". That is S4.5's
  stopgap: nothing is re-presented yet, so a swap would leave the window in the wrong layout, or lose its text. The status
  hint reads "Layout change waiting: ..." and the swap runs within `SWAP_QUIET_POLL_S` of the window closing.

### 17.2 Defects the live path exposed (none is a swap bug in itself)

- **A toast tween owned by the shell.** `_show_phone_toast()` made its fade with `create_tween()` on the shell, so a toast still
  fading when a swap freed it finished a tween whose `finished` lambda had captured the freed toast, seconds later. It is
  `wrap.create_tween()` now, which stops with the node (the "Lambda capture at index 0 was freed" errors, five per run, went with it;
  its mutant is killed by the stderr scan). This is what S4.2's note about "a toast in flight" meant: the toast is dropped with the
  phone composition, and nothing is left behind. A first guess for the same error, `LandmarkFilterPanel._queue_refresh()`'s deferred
  lambda, was tried as a method, found to change nothing (its mutant survived) and reverted.
- `civilization_workspace.gd`: `"... up to 90% of the levy ..."` in a `%` format string raised "unsupported format character"
  and left the manpower tooltip empty whenever a world stood while the workspace was built; it is `90%%`. Found because a remount
  builds the workspace with a world standing, which a cold boot never does.

### 17.3 What is still not done

- **Open windows are not re-presented (S4.5)**; until then they hold the swap (above), and `LlmAssistantWindow._phone` and the
  other windows' baked class flags are what S4.5 owns. The Ask content built on one class keeps that class's layout until it
  is rebuilt (its conversation is kept).
- The Cartography workspace's Render parts (`RenderWorkspace`'s ramp mode and similar fields) and the colour-paint tool's active
  layer and undo stacks are not in the bag.
- B5's full bar (S4.6): the three-window probe of section 12 needs S4.5.

## 18. Addendum, 2026-11-09: S4.5, the re-present path (batch 1: the shared helper, the faction roster, the Place editor)

Written against `415f0196`, read at the symbols named, verified by `_ms1swap_probe.gd --leg represent`, not by this prose.

### 18.1 The decision the contract of section 5.1 became

Section 5.1 proposed an in-place `_represent(was_phone)` that undoes the presentation and rebuilds the content. It was not
built that way, because section 5.2's own finding makes it unsafe: `phone_fit()` has no inverse, `phone_present()` and
`_desktop_window_size()` are meta-flagged one-way, and a window's `_phone` is read at its build sites. **A window is built
again instead**, through the same constructor `_ready()` uses, which is the one path that cannot differ from a cold boot.
`window_represent.gd` (a `RefCounted`, no `class_name`) states the contract; `DccApp` runs it:

- `REPRESENTED_WINDOWS` names the windows that take part; `_make_window(field)` builds one (the lines `_ready()` held, moved);
- `_capture_windows()` runs first in `_remount_composition()`, before anything is unmounted: each window answers
  `represent_capture() -> Dictionary` (it commits a half-typed field as its own re-open does, and holds no node);
- `_represent_windows()` runs after `_republish_class()` and before the mount: the old instance is freed and a new one built for
  the new class, **open or hidden** (a hidden window's `_phone` and presentation were baked for the old class too, which was
  already wrong before S4.5 for every window a swap passed while closed), so the workspaces the mount builds connect to the new
  instances;
- `_restore_windows()` runs after the mount, one frame later (a window opened before the phone's insets and the dialogs' button
  bars settle measured 27 px taller than the same window opened after): an open window calls its own `open*` for the carried
  subject, which presents for the new class, and `WindowRepresent.restore_scroll()` puts the scroll back.
- `DccShell._collect_window_blockers()` no longer reports "the X window is open" for a window with `represent_capture()`. What
  such a window holds unconfirmed it still reports (`held_changes()`, `quiescent_blockers()`), so it still makes a swap wait
  (Q-S4-2): the Place editor's unsubmitted event and the roster's open religion editor.

### 18.2 Batch 1: what is converted, what each carries

| Window | Carried | Not carried, and why |
|---|---|---|
| `faction_roster_window.gd` | `_selected`, `_tab`, `_pair`, the inspector's scroll (a half-typed field is committed first, FR-02) | `_religion_edit_id` (held: it makes the swap wait); the list pane's scroll (the phone has no list pane while an inspector shows) |
| `place_editor_window.gd` | `_index`, `_active_tab`, `_sim_mode`, the scroll (committed first, PE-01) | the `_ae_*` event draft (held, waits) |

### 18.3 Findings

- **B5's "three windows open at once" is a desktop state only.** On the phone and the tablet an `AcceptDialog` is exclusive (PC-7's
  `make_context()` clears the flag on the desktop only), so the engine refuses a second one ("Attempting to make child window
  exclusive"), and a desktop never swaps (touch is hardware). The probe therefore opens one converted window at a time and checks
  the vault's wait in its own step; the spec's B5 sentence stands as a desktop-shaped wording and S4.6 should read it so.
- **A window's scroll is clamped by the engine's own resize before the swap reads it.** When the viewport grows, the old phone
  window is stretched to it and the roster's scroll clamps (35 here), so the bar uses 20: a larger value would be lost to the
  resize, not to the swap.
- **A phone window as tall as the screen has nothing to scroll**, so the probe's phone is 880 x 800 dp, not the live leg's 880 x 1564
  (the class depends on the width only, Ruling EG).
- **The live leg's "a window is open" step used the faction roster**; it now uses the travel library, a window with no
  re-present path yet, and asserts that it has none (so the step moves to the next unconverted window when that one is converted).

### 18.5 Batch 2: six windows (World data, City viewer, Settlement types, Generation info, Conflicts, Saved journeys)

Built on the contract of 18.1; verified by `_ms1swap_probe.gd --leg represent` (now table-driven: one `match` arm per window in
`_rep_open`, `_rep_state`, `_rep_expect`, `_rep_scroll`, and a bag in `_rep_init`).

| Window | Carried | Not carried, and why |
|---|---|---|
| `world_data_window.gd` | tab (by title), the filter box's text, the shown tab's scroll | `_row_cap` (per-visit by PH-12: `open()` resets it) |
| `city_viewer_window.gd` | town (`_index`), `_zoom`, `_pan` as a fraction of `_view_canvas` | a drag in progress |
| `settlement_types_window.gd` | selected entry id, scroll (committed first: a half-typed name lands) | scroll on the way back from the phone (see below) |
| `gen_info_dialog.gd` | open, the text's scroll | the text selection (the text is regenerated from the world on every open) |
| `conflicts_window.gd` (lazy) | filter chip, scroll | the marked conflict (re-derived from the selection, PC-7) |
| `saved_journeys_window.gd` (lazy) | open, scroll | nothing else is state |

**Findings.** (1) `_pan` is in canvas pixels, and by the time the swap captures a window the engine has already resized it to the
new viewport, so `_canvas.size` is no longer what the user was looking at: the City viewer records the size each view change was made
in (`_view_canvas`) and carries the pan against it. (2) Two windows are built on first open (`open_conflicts_list()`,
`open_saved_journeys()`), so `DccApp.LAZY_WINDOWS` keeps a never-built one null and rebuilds one that exists. (3) The freed windows'
headers stayed in `DccTheme._elide_labels`; `_represent_windows()` prunes the registry after freeing (the roundtrip leg caught it).
(4) Settlement types' scroll cannot survive phone to tablet: the stretched phone window needs no scroll, so the engine clamps it to 0
before the capture (the 18.3 clamp, here with no room at all); the probe checks it on the way to the phone and expects 0 on the way
back.

### 18.4 Which windows are not converted (they still hold a swap while open, and are still wrong if opened after a swap)

From section 5.3's table, all but the twenty-four converted in batches 1 to 5 (18.2, 18.5, 18.6, 18.7, 18.8). **1 remains:** `llm_assistant_window` (section 18.8 says why it needs a different contract). (Before batch 5 the four were `vault_window`, `asset_library_window` (and its slicer), `new_world_dialog` and `llm_assistant_window`.) (Before batch 4 the ten were: **10 remain:** `vault_window`, `data_manager_window`, `asset_library_window` (and its slicer),
`new_world_dialog`, `open_project_dialog`, `phone_project_picker`, `travel_library_window`, `llm_assistant_window`,
`llm_settings_window` (`assistant_settings_window`) and `quick_bar_window`; batch 4 converted six of them.) Each needs its `represent_capture()` /
`represent_restore()` and a `_make_window()` arm; the ones `_ready()` builds with extra wiring (`quick_bar_window.pins_changed`,
`vault_window.store_changed`, `assistant_settings_window.settings_changed`) need that wiring in the arm. The windows with a
held-input state (the vault, New world, the slicer, Export, the Travel library draft) keep waiting by Q-S4-2 and need the
re-present only for the moments they hold nothing. A pop-out (`PopoutHost`) window is a native window on the desktop only and a
desktop never swaps, so a converted window's native state is not carried (PW owns that file).

### 18.6 Batch 3: seven windows (Shortcuts, Generation rules, credits, Export, Layers popover, the quick bar's phone list, the desktop Find dialog)

Built on the contract of 18.1; verified by `_ms1swap_probe.gd --leg represent` (six more table rows, plus two blocks of their own).

| Window | Carried | Not carried, and why |
|---|---|---|
| `shortcuts_dialog.gd` | open, mode (reference or rebindable), scroll | a rebind capture in progress (held input: still makes the swap wait) |
| `generation_rules_window.gd` | open, the lit preset chip (`_active_preset`), the tables' scroll | the rules and sliders (the engine's, re-read by `open()`) |
| credits (`credits.gd`, `DccApp._credits_dialog`, lazy) | open, scroll | nothing else is state |
| `export_dialog.gd` (lazy) | open, size by width, format, JPEG quality, settlements switch and tier, every layer switch, colour space, look, destination, preset title and note, scroll | a running export (see below), the measured-run line (re-derived), the overwrite sub-dialog |
| `layers_popover.gd` | open, the filter text, scroll | which layer is on (the engine's) |
| `quick_bar_phone_list.gd` (lazy) | open, scroll | the rows (regenerated from the store) |
| desktop Find dialog (`DccShell.search_represent_*`) | open, the query (it holds the scope prefix), scroll | on the phone it is rebuilt closed: the phone searches in its own overlay |

**Findings.** (1) **Export no longer holds a swap while merely open.** `quiescent_blockers()` was `["the export form is open"]` (Q-S4-2's
default) only because a rebuild lost the choices; they are carried now, so only a running export holds it (the runner is a child of the
dialog, and freeing the dialog would cancel it and delete the partial file). (2) The layers popover attached its wind/current layer
to the **surviving** viewport overlay, so a rebuilt popover would stack a second one per swap; `_attach_flow_fx()` now reuses the one
named `WindFxLayer` (a mutant that stacks it is killed by a count of exactly one). (3) The quick bar's phone list belongs to
`QuickBarPhone` (a `RefCounted`), so `DccApp.quick_bar_phone_list` is a forwarding property and the one `REPRESENTED_WINDOWS` mechanism
serves it; the credits dialog and the Export dialog are named by their existing fields (`_credits_dialog`, `export_dialog`), their
builders moved into `_make_window()` (`_build_credits_dialog()`). (4) The credits text's scroll is one-way (phone to tablet is
clamped to 0 by the stretched phone window, as for Settlement types) and Generation rules' tables have no room to scroll on the
tablet, so its scroll is checked on the phone across three same-class `_remount_composition()` calls (`--only gr_scroll`). (5) A
`PopupPanel` opened in the frame a dialog was hidden is closed by the engine's focus hand-back (seen in the full leg only), so the
probe lets the focus settle first. (6) `represent_restore()` of the credits dialog presents ITSELF (`_present_credits(dlg)`), so the
probe's fresh oracle cannot be silently satisfied by the field's instance.

### 18.7 Batch 4: six windows (Customise quick bar, assistant settings, Travel library, Data manager, Open project, the phone project picker)

Built on the contract of 18.1; verified by `_ms1swap_probe.gd --leg represent` (four more table rows, plus four blocks of their own).

| Window | Carried | Not carried, and why |
|---|---|---|
| `quick_bar_window.gd` | open, the search text, the "cannot be pinned" toggle, the selected catalogue row and pin (by `pin_key`), the ring tab and diagonal (desktop and tablet), the phone's Commands/Pinned page, scroll | the ring tab on a round trip through the phone (the phone has no ring tabs: dropped by design) |
| `llm_settings_window.gd` | open, scroll; the address, model and variable-name fields are committed at capture exactly as a close commits them | a typed API key: it is cleared by design on every rebuild, so it is **held input** (`quiescent_blockers()`: "an API key is typed but not saved") |
| `travel_library_window.gd` | open, tab, selected entry, the rail filter's text, the staged `_draft`, the phone's pane, scroll; a focused, unstaged field is staged by the capture | nothing; a staged draft still makes a real swap wait (Q-S4-2) |
| `data_manager_window.gd` | open, route, every session option of every route (`REPRESENT_FIELDS`: `_tx_*`, `_wd_*`, `_gis_*`, the run log, the Validation result), the phone pane, scroll | `_wd_widths` (the binding's answer, re-asked) |
| `open_project_dialog.gd` | open, welcome or gallery mode, scope chip, search text, selected tile | on the phone it opens `PhoneProjectPicker` instead |
| `phone_project_picker.gd` | open (it is not in `REPRESENTED_WINDOWS`: the composition already frees and builds it; `DccApp._picker_bag` carries the flag, and a tablet gets its dialog opened) | the error line (`open()` clears it) |

**Findings.** (1) The Travel library stages a field only on Enter or focus loss, so unstaged typed text would have been lost; `represent_capture()` releases the focused inspector field first. (2) The rail filter's text was a local `LineEdit`; it is a field now (`_filter_edit`) and the capture reads its text, not `_filter_text` (a probe or paste can change the box without the signal). (3) Every Data manager route keeps its options only in session fields, so a rebuild would have silently reset an export the user had configured. (4) `DccApp._make_window()` carries the two `_ready()` wirings these windows had (`pins_changed`, `settings_changed`), reading the other windows when the signal fires. (5) The live leg's wait example was the Travel library merely being open; it is now the Travel library with a staged definition edit, the only kind of wait left for these windows. (6) Still not converted, and still holding a swap while open: the vault, the asset library and slicer, New world and the assistant dialog; New world's form state and the assistant's conversation need their own contracts (New world re-reads flags and extent from the engine on every open, so only seed, size, grid, archetype, plates and history are its own).

### 18.8 Batch 5: three windows (New world, the asset library with its slicer, the vault) and why the assistant dialog is not one of them

Built on the contract of 18.1; verified by `_ms1swap_probe.gd --leg represent` (three more table rows, plus the `--only nw_extra` block).

| Window | Carried | Not carried, and why |
|---|---|---|
| `new_world_dialog.gd` | seed (a typed, uncommitted one is committed by the capture), its lock, width, grid columns and rows (typed text committed), aspect, archetype (selected again WITHOUT applying it again), the plates dial and its manual flag, the history strip (its entries hold `ImageTexture` resources, not nodes), "more options", scroll | the four experimental flags and the extent (the engine's: `_sync_from_engine()` re-reads them on every open, which is the point); the preview (recomputed) |
| `asset_library_window.gd` and its slicer | the view (family, collection, unassigned or pinned list), search, sort, grid zoom, batch selection, the focused slot and variant, the preview backdrop, the phone's pane, scroll, the pack fields (committed first) and the unsaved flag; slicer: open, the decoded sheet `Image`, its readout, dragged lines, columns, rows, margin, spacing, trim, skip-empty, chroma key, key colour and tolerance, target, family, fill mode, new name, set, picked cell | the sheet preview's own pan and zoom (it reopens fit-to-view) |
| `vault_window.gd` | scope (entity, browser or overview), the note and the reader's pick, the mode (Write, Split, Preview), the phone's pane, scroll, the session undo `_hist` (memory only), the link trail; the text is flushed through `_write_now()` first (the one writer) | the privacy unlock state (every open clears it already) |

**Findings.** (1) **A `SpinBox` whose value was written with `set_value_no_signal()` shows stale text**, and `apply()` would parse that text back over the value (the hazard `request()`'s own comment records). The New world capture therefore applies only the boxes whose line edit emitted `text_changed` (a typing signal, never raised by code), and the restore writes value and text together (`_set_spin()`). (2) **A typed seed's commit pushes a history entry** (`_on_seed_changed`), so it has its own block (`--only nw_extra`), not the main fixture's oracle. (3) **The slicer is parented to the shell, not to the library**, so freeing the library strands it and a second one stacks per swap: `DccApp._represent_windows()` calls a window's optional `represent_release()` first, and the slicer carries the new meta `DccShell.REPRESENTED_BY_OWNER_META` so the shell's walk stops reporting it as "an open window". It is freed WITHOUT `_close_slicer()`, which would drop the engine's decoded sheet; the sheet's `Image` rides in the bag and `_sheet_loaded` stays true. The probe's fresh-open oracle calls `represent_release()` too, or each oracle build leaked a slicer. (4) **A phone build of the library leaked one unparented `Label` per build** (the grid foot's drop hint was built for every class and parented only on the desktop; found by diffing `Node.get_orphan_node_ids()` around a build), which a swap that rebuilds the window each class change turned into one per round trip. It is freed on the phone now (a mutant that restores it is killed by the orphan-count check). (4b) **`_refresh_slicer_summary()` clears the picked sheet cell** (a grid edit makes the index stale), so the restore puts it back after the summary; a probe run with that block removed reads -1. (5) New world's scroll on the way back from the phone is 12, not 0 or 20: the phone card is stretched to the tablet viewport before the swap captures it (the 18.3 clamp with some room), so the probe expects the measured 12 (`REP_SCROLL_BACK`). (6) The dialog's layout settles over about thirty frames after it opens; the probe waits for it (`REP_SETTLE`) before comparing with a fresh open. (7) The vault's session undo is memory only, so it rides in the bag, closed or open: the Note tab that docks a note after the swap reads it too.

**Why `llm_assistant_window` is not converted (a design finding, not a postponement).** Its content (`_host`: the transcript, the cards, the draft, the chips) is built once and every control in it is connected to a method or a lambda **of the window instance**, and so are `run` (the `LlmRun` child, with a worker thread that may be streaming) and the nine `run.*` signals. A rebuilt window cannot adopt that content the way PC-6's `release_dock()` hands it back to the SAME window: the connections of the freed instance die with it, and re-wiring every card (proposal, view, edit, action and source cards, each with its own Apply state and callables) is a second implementation of the panel. The two contracts that would work are: (a) **keep the instance and re-lay out in place**: `_phone` is read in about twelve build conditionals plus `phone_window()` and `phone_fit()` at `setup()`, none with an inverse, so the content would be rebuilt around the preserved `_msgs` children (reparented, not recreated) while the dialog layer (borderless, scale, relay) is presented once more, which needs `phone_window()` to become idempotent first; or (b) **a transcript model**: the messages and cards as data, drawn by one function, so a new instance can draw the same conversation. Either is a milestone of its own; neither is a batch. Today the assistant's dialog form is correct only for the class it was built in, and the Ask tab's content is carried through a swap (S4.4) with its stale `_phone`.



### 18.9 S4.6 part 1: the radial ring and the context card (2026-11-11)

Built at the symbols `ContextBroker.represent_capture()`, `represent_free()` and `represent_restore()`, `RadialRing.represent_capture()` and
`represent_restore()`, and the three calls in `DccApp._remount_composition()` (capture before the unmount, free before the new class is
published, restore in `_restore_windows()` once the new workspaces exist); verified by `_ms1swap_probe.gd --leg ringcard`, not by this prose.

- **The decision, and the finding that shaped it.** Section 2.3 said "free both and let the broker rebuild them lazily", and that is what is
  built, for a reason the section did not have: a real swap never meets an OPEN ring or card, because `DccApp._app_quiescent_blockers()`
  already reports "a context card is open" while `_context_surface_depth` is above 0 (the ring and the card each add one). What every real
  swap meets is a **closed ring or card built for the old class**: `RadialRing._apply_touch_scale()` only scales up (`if not is_tablet():
  return`, never a reset) and the card's `_m()` figures and chrome were drawn for its last `open()`, so a kept ring would keep the
  tablet's 140.76 px radius on a desktop-size window. So the swap frees the ring, the card and the broker's fallback `PopupMenu` and nulls
  the three fields; `_ensure_ring()` and `present()` build them for the class then current. The `_apply_touch_scale()` comment, which said
  S4 would have to re-run it, now says the rule is the other one.
- **The carry covers the paths that bypass the gate** (a direct `_remount_composition()`, which the probes and any caller that has already
  decided take, and a surface opened between the gate's last look and the capture). Carried: the **card's subject** (`last_request`: the
  point, the hits under it, the picked hit if Select narrowed it, the source; the rows are collected again from the new workspaces, so no
  callable bound to a freed node is carried) and the **ring** (its centre, its sticky flag and its Q-form flag, re-armed with the new
  class's slots and geometry, off the phone only). A phone's peek sheet is the card's phone form: its subject becomes the tablet's card
  and the reverse. **Not carried:** the card's filter text, expanded lists and a stepped parameter (a card is one decision's worth of
  surface), a sub-ring (re-opened with one flick), the ring's hover, and the phone's sample pin on a phone to tablet swap (the sheet is
  closed, not dismissed).
- **Found by the probe, fixed:** the phone's peek sheet holds one share of the same world pause (`phone_menu.gd`: `peek_card()` pauses,
  `close()` resumes) and is freed with the phone composition. A phone to tablet swap with the sheet open left `_context_surface_depth` one
  too high for good (the probe read 2 with only a card open); `represent_free()` now closes the sheet first. A ring or card freed while open
  would have done the same, which is why each is closed before it is freed.
- **Bar (`--leg ringcard`, tablet 1800 x 1000 and phone 880 x 800 at density 1, 44 checks):** the tablet ring's radius is the literal 140.76
  (92 x 1.53) and a ring armed on the phone after a swap is the literal 92, so the two instances are told apart; a real swap WAITS while
  the ring and card are open (named "a context card is open") and runs once they close; the old, closed instances are invalid afterwards and
  the broker's fields are null; a direct tablet to phone remount with both open leaves depth 1 (the peek sheet) and the same point (gx, gy to
  1e-6) in the phone form; phone to tablet gives the card in the tablet form with no ring and depth 1; a same-class remount with both
  open carries the radius, the centre (within 1 px), the sticky and Q flags and the subject, and depth returns to 2, then 0 when closed; a
  remount with neither open builds neither; no connection keeps a freed receiver.
- **Mutants (literal, one at a time, restored by the script, `C:/cw/s46_mut.py`):** 11 of 11 killed (the ring's free, the card's hide before
  its free, the peek sheet's close, the ring restore, the subject restore, the sticky flag, the Q flag, the centre, and the capture, free
  and restore calls in `app.gd`).

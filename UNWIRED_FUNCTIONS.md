# Presented but not wired — the standing table

Owner, 2026-08-30: *"Make sure all menu's are created and all presented
functions are wired or the ones that have no code behind them get listed in a
table with a proper proposal (inferred from the menu name and highest probable
explanation/design spec of the named function.)"*

This is that table, re-cut on **2026-08-31** against the two new prototypes in
`design/dcc-environment-2026-08-31/` after stage 1 (tokens) and stage 2 (the
rail fold to WORLD / CIVIL / CARTO) landed. It carries **77 open rows** —
17 trivial, 25 small, 17 medium and 18 large (two of the large ones recorded
without a proposal, because they need a decision first) — plus a section for the
**25 dangerous-class items**: controls drawn *enabled* that do nothing, and rows
whose stated reason is false. Every proposal is drawn from
`docs/DCC_SHELL_SPEC.md`'s behaviour column, from the 2026-08-31 spec set
(`design/dcc-environment-2026-08-31/spec/01`–`06` and `BUILD_ANSWERS.md`), or
from an engine binding that already exists; the source is named in each row.
None is invented.

**Most rows here are honest in the product.** `menus.gd::_todo(p, text, why)`
(`menus.gd:253-257`) always sets a tooltip, so a `_todo` row appears disabled
*with its reason* where a user meets it, and `phone_menu.gd:798-799` draws that
reason as the row's second line on a handset — which means a false reason is a
full-width sentence on a phone. The workspace panels disclose the same way,
through `_dead_slider` (`cartography_workspace.gd:1333-1346`) and `_mark_inert`
(`:1321`). **The exceptions are the whole point of the second section**, and
this cut found more of them than the last one did.

---

## What was re-verified, and what was stale

The previous generation of this file had **44 rows and 12 of them wrong**. That
was recorded honestly at the time, and this cut treats the same defect class as
the primary target rather than a footnote. Five auditors read the code on
2026-08-31, and every row below carries a `file:line` that was opened, not
carried forward.

| Claim in the previous cut | Status on 2026-08-31 |
|---|---|
| `Slot transform` reason false | **Confirmed fixed** — `as_set_item_transform` at `lib.rs:10811`, called on every slider tick from `asset_library_window.gd:1703`, Fit/Reset at `:2485`/`:2488`. Struck. |
| `_refresh_family_counts()` computes filled/capacity live | **Substance holds, name wrong** — the real function is `asset_library_window.gd:1315` `_refresh_rail_counts`. Corrected here. |
| `Keyboard shortcuts…` reason *"No shortcut table yet"* | **Confirmed fixed** — rewritten to name *rebinding* as the missing half, which is true: `shortcuts_dialog.gd` exists and walks the menus; no per-context store exists in `dcc_settings.gd`. |
| Five GPU / Documentation `_todo` rows listed flat as gaps | **Wrong shape.** `menus.gd:1419`, `:1420`, `:1427`, `:1428` are the `else` of `if _bridge.gpu_api` (`:1415`, `:1425`), and `gpu_api` is true on any build carrying `lib.rs:2665` + `:2721` — every current one. `menus.gd:2565` is the `else` of `_docs_dir() == ""`. On a current build run from the repo, **none of the five is drawn.** Moved to *Not gaps*. |
| Timeline strip *"needs the owner's word first"* | **The word arrived.** `01-frame-and-tokens.md` §3.7 and `05-right-dock-and-bars.md` §4.2 author the strip in full. Promoted from blocked to Medium. |
| — | **New:** `right_dock.gd:1653-1656` says the hero readout is *deliberately* unscaled. `BUILD_ANSWERS.md` §2.4 reverses that on 2026-08-31 and calls it an oversight. Stage 1 landed the token; nothing consumes it. |
| — | **New:** `pack.rs:24-45` declines to decode painted-layer pack art because *"there is no producer of a painted-cell array anywhere in this workspace"*. There has been one since 2026-08-24. |
| — | **New:** `dcc_shell.gd:3302-3308`'s `_refresh_phone_bar_lit()` documents behaviour it has never had — its two backing dictionaries are declared and assigned nowhere in the repo. |
| — | **New:** `right_dock.gd:844-847` dashes `State religion` citing a missing binding, while the value sits in a dictionary the same function fetched thirty lines earlier. |

Tooling limit worth restating: `audit_wiring.py` (run 2026-08-31 — A: 58/1217 ·
B: 2/384 · C: 0 · D: 14) finds *unwired bindings*. It structurally cannot see a
**stale reason**, because every function involved is called and it is the prose
that lies. Nine of this cut's flags are of that kind.

---

## Fixed during the audit itself (2)

Two findings were cheap enough that leaving them in a table would have cost more
than fixing them, so they are closed rather than listed:

- **`State religion`** (`right_dock.gd:844`) — dashed with *"get_provinces()
  doesn't carry it and there is no get_faction_aggregates() binding"*. Both
  clauses true, neither relevant: the row is built from `roster`, which comes
  from `get_factions()`, which has carried `"religion"` since the roster window
  shipped (`lib.rs:6243`). `Culture` two rows above already reads out of the same
  dict. **Now live.**
- **`_refresh_phone_bar_lit()`** (`dcc_shell.gd`) — guarded on
  `_phone_bar_more.is_empty()`, and those two fields were declared and read but
  **assigned nowhere in the repo**, so the function returned at its first line
  from six call sites and had never run. Introduced when the five-cell bottom bar
  became four `PHONE_TABS` cells; `_refresh_phone_tabs()` is the live
  replacement. **Deleted** — the function, its only helper `_light_bar_cell`,
  the two dead fields and all six call sites, 29 lines.

## Trivial — a string, a line, or a value already in scope (17)

| Item | Where (`file:line`) | Proposal | Size |
|---|---|---|---|
| Faction ▸ **State religion** reads `—` | `right_dock.gd:844-847` | Delete the dead `_field` and print `String(roster.get("religion", "none")).capitalize()`. `roster` is fetched at `:813-818` and already read for `culture` at `:826`; `get_factions()` carries `"religion"` at `lib.rs:6243`. Same pass: add `government` and `ag_tech`, both carried by the same dict and shown nowhere in this dock. **Source: the existing binding.** | trivial |
| `Auto-populate world` tooltip says *"params.rs has 58 entries"* | `civilization_workspace.gd:1305-1307` | `PARAMS` now holds 81. Update or drop the number — a countable claim in a user-visible tooltip is a standing staleness liability. | trivial |
| Faction comment cites `get_factions()` at `lib.rs:3442` | `right_dock.gd:810` | It is at `lib.rs:6225`. Fix the citation. | trivial |
| `No landmark types` — unreachable dead branch with a false reason | `menus.gd:826-830` | `landmark_kinds()` exists (`lib.rs:12453`) and is cached at init (`engine_bridge.gd:3097`, read at `:3104`) before `menus.build()` runs (`app.gd:289` → `:365`), so `kinds.is_empty()` never fires. Delete the branch, or re-word it as an explicit stale-`.so` guard (*"this build's GDExtension predates `landmark_kinds()`"*). **Source: the binding.** | trivial |
| *"Not available for this world."* appended to permanently-unavailable layers | `layers_popover.gd:312-314` | `layer_available()` (`sample_bridge.rs:726-745`) refuses on two different grounds. Branch the suffix: `GAP_LAYERS` (`sample_bridge.rs:716` — `oro`, `velo`, `popdensity`, `siteprofile`) get *"No estimator for this in the engine."*; the seven conditional ones (`strahler`, `bclass`, `cterrain`, `windthrow`, `wildlife`, `control`, `contested`) keep the current sentence. | trivial |
| `layers_popover.gd:63` asserts *"the eleven permanent engine gaps (`GAP_LAYERS`)"* | `layers_popover.gd:63-65` | `GAP_LAYERS` holds four. Eleven is 4 + the 7 conditional refusals, which the same sentence calls *"disabled on every world that will ever exist"* — false of the seven. Split the count; this is the one file that reasons about hotkey allocation from it. | trivial |
| WORLD gap note claims tides have *"no cartalith-engine equivalent yet"* | `world_workspace.gd:68` | True for geoid (`params.rs` has no geoid entry; `refresh_geoid` at `geoid.rs:135` has no caller). **False for tides** — `passes.tidal_flats` (`params.rs:554`) is live and its engine doc says *"This port has no separate enable: this toggle is it"* (`cartalith-engine/src/lib.rs:354-364`). Split the sentence in two. | trivial |
| First-run coach mark names a bar that no longer exists | `dcc_shell.gd:4337-4339` | Text reads *"WORLD · CIVIL · CARTO switch domains here — PANELS and MORE reach everything else."* The bar reads MAP · GENERATE · PLAN · MORE (`PHONE_TABS`, `:3210-3219`) and PANELS is not a tab at all. Rewrite against `PHONE_TABS`. | trivial |
| `--hero` / `--hero2` density tokens consumed by nothing | `dcc_theme.gd:778-779`; call sites `right_dock.gd:1659`, `DccTheme.hero()` `:1151` | `"fs_hero": [26, 30]` and `"fs_hero_2": [22, 26]` are defined and read by no call site (`grep -rn '"fs_hero"' shell/` → the definition only); both call sites hard-code `26`. Route both through `ROLE`. **Source: `BUILD_ANSWERS.md` §2.4** — *"The three unscaled values — all three now scale. They were oversights."* | trivial |
| `--popW` density token consumed by nothing | `dcc_theme.gd:766`; `layers_popover.gd` | `"w_popover": [238, 300]` is defined; the popover sets no `custom_minimum_size.x` at any density and sizes to content. Set it from the token. **Source: `BUILD_ANSWERS.md` §2.4.** | trivial |
| `pack.rs` declines painted layers with a producer that now exists | `crates/cartalith-godot/src/pack.rs:24-45` | Rewrite the module doc: `PaintEditor`/`PaintLayer`/`commit_all` ship at `paint_bridge.rs:494-505`, `get_paint_layers` at `lib.rs:6307`, `with_paint` at `lib.rs:4502`, and the tool is drawn at `world_workspace.gd:1683` and `tool_bar.gd:433`. `render.rs:32-34` already records the correction; `pack.rs` never got it. **Decoding pack `biomes`/`terrains` is a separate Medium-sized job — the doc fix is trivial and must not wait on it.** | trivial |
| `Saved` look picker's `choice()` is a literal no-op | `render_workspace.gd:852-853` | Pick-then-**Load look** (`:854`) is a deliberate two-step, but a user who picks and walks away sees nothing happen. Either name the Load button in the picker's tooltip, or make selection load directly. Lowest-severity dead handler in the shell. | trivial |
| Icon-placement fill line reads as live data | `cartography_workspace.gd:1296-1299` | *"10 of 12 slots filled · unfilled slots fall back to the family default glyph"* comes from `ICON_PLACEMENT_FAMILIES`' hard-coded design figures (documented at `:1259-1264`), not the user's world. Give it the `--` treatment the drawn-count column already uses, or tag it `(design figures)`. | trivial |
| No project format version anywhere in the UI | `engine_bridge.gd:2993` `project_format_version()`; binding `project_bridge.rs:1685` | Print it in `Help ▸ Generation info…` (`gen_info_dialog.gd`, already a plain-text dump) and in the Open dialog's tile caption. A format number is the first thing asked when an old save misbehaves. | trivial |
| Missing-binding fingerprint has no readout | `engine_bridge.gd:249` `missing_bindings()` | `_has()` (`:235-244`) already `push_warning`s and accumulates the list; nothing reads the accessor. One line in `Generation info…`: `bindings missing: none` or the list, so a user reporting *"feature X is greyed out"* ships the answer with the report. | trivial |
| Disabled radio rows lose their radio mark on phone | `phone_menu.gd:798-801` | The `if disabled` branch returns before the `is_item_radio_checkable` branch, so `Alternate frames` and `Reduce working res` read as plain dim rows rather than unselected choices in a group — a presentation the desktop does not have. Draw the mark, then dim. | trivial |
| Landscape sheet handle is drawn and inert | `dcc_shell.gd:3428-3440` (drawn), `:3602` and `:3552-3554` (no-op) | `BUILD_ANSWERS.md:108-109` rules the landscape drawer has no detents to re-snap to — correct, and implemented. But nothing hides or dims the 40×4 handle, and coach mark #2 tells the user to drag it. Hide the handle when `_landscape`. | trivial |

---

## Small — one function, against a binding or a spec line that already exists (25)

| Item | Where (`file:line`) | Proposal | Size |
|---|---|---|---|
| Biome-K toggle is never re-read from the engine | `new_world_dialog.gd:253` (CheckBox discarded), `:460` `_sync_from_engine()`; `engine_bridge.gd:1875` `get_biome_k_enabled()` | Retain the CheckBox in a var the way `villages_check`/`metropolis_check` are, and add the fourth clause to `_sync_from_engine()`. The F14 block (`engine_bridge.gd:2998-3002`) fixed exactly this for three of four toggles and missed the fourth; today the box reads unchecked while the engine has it on, and the next Create silently turns it off. **Source: the F14 block's own premise.** | small |
| Theme choice does not survive a relaunch | `menus.gd:210`, `:242`, `:1557-1568`; `dcc_settings.gd:21-33` | `_theme_mode` is a plain `var`; `DccSettings` has roots / recent / gpu / autosave / tiles_lod / lighting and no theme section, and `DccTheme.apply_theme()` has one caller. Add the section and restore at boot. **Source: `BUILD_ANSWERS.md:98-99`** — *"Device, theme and units persist … and restore on load."* | small |
| `Window ▸ Status bar` moves a check mark and nothing else on phone | `app.gd:481`; `dcc_shell.gd:2692-2697` | On phone the status bar is built into `model_host`, which is permanently `visible = false`; the toggle flips a node inside a hidden host. Give `_region_nodes` a phone-aware entry the way `ID_WIN_RAIL` already has (`app.gd:482-484`), or disable the row with that reason on phone. | small |
| `Window ▸ Left dock` / `Right dock` / `Reset layout` desync the phone sheets | `app.gd:484`, `:1888-1892`; back chain `dcc_shell.gd:4478-4482`, `:4530-4545` | `toggle_region()` writes `node.visible`; the phone sheets' truth is `_left_sheet_open`/`_right_sheet_open`, which the system-back chain tests. `Reset layout` currently shows both docks with both flags still `false`, so back will not close them. Route all three through the sheet openers on phone. | small |
| `Window ▸ Domain rail` is a one-way door on phone | `dcc_shell.gd:3157` (`_rail_region = domains`), `:3188-3196` | The loop adds **every** `PHONE_TABS` cell — MORE included — under `domains`, so hiding the rail hides the only route back to the menu row that un-hides it. `_phone_menu_bar.visible` also stays true, so `_phone_nav_reserve()` (`:3509`) reserves 64 dp of empty strip. Point `_rail_region` at a container holding only the three domain cells — exactly what the comment at `:3131-3134` already specifies. **Source: that comment.** | small |
| `_refresh_phone_bar_lit()` has never run | `dcc_shell.gd:390-391`, `:3302-3321` | `_phone_bar_more`/`_phone_bar_panels` are declared and assigned nowhere in the repo, so the `is_empty()` guard at `:3310` fires on all six call sites (`:3857`, `:3862`, `:4031`, `:4414`, `:4425`, `:4543`) and `_light_bar_cell()` is unreachable. Either assign them in `_build_phone_menu_bar()`, or delete the function and let `_refresh_phone_tabs()` (`:3280`, which works) own lit state. Visible symptom: closing MORE leaves the MORE pill lit. | small |
| Selecting CIVIL leaves MORE lit | `dcc_shell.gd:3193-3196`; `phone_menu.gd:684-687` | `PHONE_TABS` has no `civilization` entry (defensible, and argued at `:3204-3208`), so `_domain_buttons` has nothing to light — but no public `select_domain*` path calls `_refresh_phone_tabs()`, so the bar names a destination the user is not at. Call it from `_select_domain()`; when the active domain has no tab, light MORE. Applies equally to `Window ▸ Workspace` and every in-shell *"→ Cartography ▸ …"* jump. | small |
| `CommandIndex` indexes read-only readouts as unavailable *commands* | `command_index.gd:122-142` | The chrome test is *"disabled and no tooltip"*. Five permanent readouts are disabled **with** a tooltip and get indexed with `available: false`: `menus.gd:1536` Working set, `:1067` schema, `:1806` VRAM estimate, `:1688`/`:1695` GPU memory, `:461` No recent projects (the recursion at `:122-126` reaches the GPU ones). Add a third state (readout) — only `_todo()` should mint an unavailable command. **Source: `command_index.gd`'s own header.** | small |
| No per-slider "reset to default" | `engine_bridge.gd:277` `param_default(key)`; the one Reset button `world_workspace.gd:1015` | The accessor is filled at `:258` and read by nobody; the Reset button uses a GDScript-local `ERODE_DEFAULTS`, a second transcription of numbers the engine already owns. Add a per-slider reset from `param_default(key)` and rebuild `ERODE_DEFAULTS`' consumer off it. | small |
| Stage group names are hardcoded with no check | `engine_bridge.gd:272` `param_groups()`; `world_workspace.gd:879-881`, `:908-911` | The shell filters `param_keys()` on `info["group"]` against names hardcoded in `STAGES`. Rename a group in `params.rs` and that stage section renders **empty**, silently — the exact silent-degradation shape `audit_wiring.py` question C exists to catch. Add one boot-time assertion that every `STAGES` group is in `param_groups()`. | small |
| No `Reset generation parameters` command | `engine_bridge.gd:325` `reset_params(keys)`; binding `lib.rs:2612` | Add `Edit ▸ Reset generation parameters` (all), plus a per-stage *reset this stage* using the `keys` overload with that stage's `STAGES` group keys. **Source: the binding's own two forms.** | small |
| Heightmap import commits without showing the working grid | `engine_bridge.gd:547` `heightmap_grid_size()`; commit site `app.gd:1150` | Its own doc names the consumer: *"for a dialog that wants to show the working grid before committing."* On file-pick, print `2048 × 1311 → working grid 1024 × 656` in the New World dialog before Import. | small |
| Menu bar is missing `↶ ↷ ◐` | `dcc_shell.gd:747-788` `_build_menu_bar` | Design children 4, 5 and 8 (`03-menu-bar.md` §2; surviving markup at `ENV:101-105`): undo, redo and theme squares, `var(--ctl)`, ground `var(--ins)`. Today undo is reachable only via `Edit ▸ Undo` (`menus.gd:513`) and ⌘Z, theme only via Preferences. Build undo and theme now; redo is blocked on the Medium row below, so either omit it or draw it disabled with that reason. The `undoCol`/`redoCol` values were in the truncated tail — use `ROLE` equivalents. | small |
| Fourth global tool `pan` is not drawn | `dcc_widgets.gd:869-873` `GLOBAL_TOOL_ENTRIES` | Add a fourth, permanently-inert legend cell: label `Pan`, tooltip verbatim *"Pan / zoom — always available"*, painted `ins` ground / `dis` ink, no arm callback. **Source: `02-rail-and-domains.md` §4d and `01-frame-and-tokens.md` §3.6c** — it exists to teach that pan needs no arming, which the desktop teaches nowhere (there is a pan *mode* on the touch navpad, `viewport_host.gd:1048`, and no pan cell in any TOOLS block). | small |
| A GPU readback failure bans the GPU for the whole session with no way back | `crates/cartalith-gpu/src/multi.rs:641` `clear_readback_failures`; ban enforced at `lib.rs:1217`, `:1240` | Add `Preferences ▸ GPU ▸ Try the GPU again`, enabled only when a failure is recorded. **Source: the function's own doc** — *"for a 'try the GPU again' affordance after the user changes something (a driver update, a smaller world) that might make it work."* The Preferences ▸ GPU block already carries device, multi-GPU mode, VRAM budget and fallback rows (`menus.gd:1425`, `:1820`); this belongs beside them. | small |
| The place tool does not hit-test through the engine | `engine_bridge.gd:1658` `civ_pick_place_at`; binding `lib.rs:6036` | Route the CIVIL place-tool click through it rather than any GDScript nearest-search. `lib.rs:6027-6034` explains the missing zoom parameter (the `civ_zoom_pick_r` decline) but nothing explains why the shell never hit-tests at all. | small |
| Safe-area insets are mock constants | `dcc_theme.gd:828` `H_PHONE_TOP_SAFE := 28`, `:843` `H_PHONE_GESTURE := 20`; consumers `dcc_shell.gd:2856`, `:3679`, `:4798-4802` | `DisplayServer.get_display_safe_area()` is never called anywhere in `shell/`; on a device whose real inset exceeds 28 dp, chrome sits under the cutout. Take `max(real_inset, mock)`. **Source: `BUILD_ANSWERS.md:87-89`** — *"the mock value is the floor, the real inset wins when it is larger"*, matching the prototype's `max(env(safe-area-inset-top),30px)` at `Cartalith Android.dc.html:36`. | small |
| No haptics anywhere | `shell/*.gd` — zero `Input.vibrate_handheld` | Add a `_haptic(kind)` helper on the phone paths and call it from `_set_phone_detent`, tool arm and the back chain. **Source: `BUILD_ANSWERS.md:100-101`**, which specifies the whole table: sample 12 ms · detent 8 ms · tool arm 10 ms · verdict `[14,40,14]` · back 6 ms · blocked `[20,60,20]`. | small |
| No relief-exaggeration default in Preferences | `menus.gd:1449`; the live per-world slider is `render_workspace.gd:180` | Add `Preferences ▸ Graphics ▸ relief exaggeration 1× / 2× / 4×` as a project-level default for a value the shell already renders live. **Source: `BUILD_ANSWERS.md` §3** — *"Wired the missing half … so the toast points somewhere real."* No 3D viewport is required; this is the same shape as `Lighting rig defaults`, which closed 2026-08-30 as *"a settings key, not new rendering"*. It also gives the phone's 2D/3D FAB toast a real destination. | small |
| Landmark viewshed note states the superseded weighting | `civilization_workspace.gd:1984-1990` (panel), `:2096-2098` (per-row `[no viewshed]`) | The panel presents the older research figure *"weights it at 0.20 — its joint-largest"*. **Source: `BUILD_ANSWERS.md` §3** gives the owner's 2026-08-31 formula: *"once visibility analysis lands, `score = 0.6 × prominence + 0.4 × visible land area inside 30 km`, caps unchanged."* Replace the text only — the placement (panel *and* every affected row) is already better than the design asks for. | small |
| Two computed analysis fields have no debug view | `crates/cartalith-terrain/src/analysis.rs:266` `tpi_multiscale`, `:319` `local_relief` | Add both to `sample_bridge.rs`'s view registry. Both are already computed and golden-tested and are called only from tests; `local_relief` is *"§2.2's first named ingredient for 'is this a landmark'"* per its own doc — exactly the field a user needs to understand why the landmark pass chose what it chose. `tpi_multiscale`'s doc says it is *"exposed here as an analysis field rather than a shading term"*, so the slot is the one it was built for. | small |
| Timeline-aware tid reseed is never called | `crates/cartalith-civ/src/timeline.rs:1634` `civ_resync_next_tid_with_timeline` | `grep resync_next_tid crates/cartalith-godot/src/` returns nothing — neither resync function is bound at all. Call the milestone-4 one wherever `CivData::next_tid` is reset, or a reseed after timeline history exists can collide with historical ids. | small |
| `label_glyph_layout` is re-implemented in GDScript | `engine_bridge.gd:2153`; `map_overlay.gd:1482-1523` vs. `crates/cartalith-civ/src/labels.rs:168-191` | The GDScript copy reproduces the radius (`maxf(font_px * 1.2, total_w / (2.2 * absf(a)))`, `:1509`), the per-glyph theta/dx/dy/rot (`:1511-1519`) and the `0.01` straightness threshold that is the named `ARC_STRAIGHT_THRESHOLD` (`labels.rs:150`) on the Rust side. They agree today; the Rust doc (`lib.rs:8086-8091`) warns that summing per-char widths instead of measuring the whole string drifts on kerned fonts. Labels are tens per frame, not thousands — call the binding once per label. Minimum: pin `0.01`/`1.2`/`2.2` with a comment naming `labels.rs`. The neighbouring decline (`ops_bridge.rs:139-156`) argues the FFI cost of saving *one multiplication*; that does not transfer to a whole layout loop, and no document makes the argument for this one. | small |
| Generation and save failures never surface on phone | `app.gd:607-612`, `:1264`, `:1330`; toast helper `dcc_shell.gd:4247` | All three write status slots inside the hidden `model_host`, so the only surface is MORE ▸ Generator — and *"save failed — see console"* is unactionable on Android, where no console is reachable. Route failures through `_show_phone_toast()` (three callers today, all coach marks / undo chip). **Source: `BUILD_ANSWERS.md:110` §4** — *"Generation-failure and storage-full states. Not designed yet — ask and I will add both."* Build the toast; the storage-full detection itself is Medium below. | small |
| Bottom-docked controls do not ride above the IME | `dcc_shell.gd:3971-4018`, search field `:3995` | Zero `get_virtual_keyboard_height` / `virtual_keyboard` calls in `shell/`. **Source: `BUILD_ANSWERS.md:93-94`** — `visualViewport` resize → `state.kb` → `dockBottom` adds it. Read the keyboard height and add it to the bottom dock inset. The search field itself survives (anchored under the app bar); everything bottom-docked does not. | small |

---

## Medium — a real feature, but every ingredient exists (17)

| Item | Where (`file:line`) | Proposal | Size |
|---|---|---|---|
| The expanded timeline strip | `dcc_shell.gd:2546` `_build_timeline`; `app.gd:906-949` `_fill_timeline_strip` | Today the region exists and `Window ▸ Timeline` toggles it, but it holds one label `TIMELINE`, a clipped hint, and an `Open Timeline` button that presses the CIVIL Politics accordion (`app.gd:929-949`). **Source: `01-frame-and-tokens.md` §3.7 + `05-right-dock-and-bars.md` §4.2**, which author it in full: transport `▶ ◀ ▶`, speed pill group `×1 ⁄ ×10 ⁄ ×100`, `{{ tlState }}`, six layer toggles, a 3 px scrub track with a 2 px accent playhead, footer `YEAR −400 · {year} · YEAR 1200`, and the collapsed form `TIMELINE · {tlYearLabel} · ▴ expand`. The year cursor already lives in the CIVIL dock, so the strip is a second *view* of one model, not a second model. `app.gd:906-920`'s `TIMELINE_SCOPE.md` §4 reason is true and predates this spec. | medium |
| Six simulation layer toggles | nowhere — `grep -n "Warfare" shell/` finds nothing; nearest categories `civilization_workspace.gd:1634`, `:1648` | `Climate · Population · Economy · Politics · Infrastructure · Warfare`, on the timeline strip above. **Source: `01-frame-and-tokens.md` §3.7 state defaults (line 1204).** `BUILD_ANSWERS.md` §3 classes them *intended* and fixes the required note verbatim — *"they record which layer you want; no layer renders yet"* — and rules it must sit **on the timeline**, so the note has nowhere to go until the strip exists. Build both in one pass. | medium |
| `statusMid` composite | `dcc_shell.gd:2563-2587` `_build_status_bar`; writers at `app.gd:503`, `:608`, `:711`, `:814`, `:1262-1274` | **Source: `BUILD_ANSWERS.md` §2.2**, fixed verbatim: `last pass 09 Ecology · 101 ms · repaint 84 ms · autosave 14:02`, with the stage name being the last stage whose `stageState()` is `resolved` and `autosave` reading `off` when autosave is off. Today: four independent slots (`pass`, `stale`, `autosave`, `atlas`) plus `hint`, written from ten-odd sites, with `pass` reading `generated · 1.4s` / `loaded` / `no world`. Everything but `repaint NN ms` exists (`grep -rn repaint shell/*.gd` finds only prose). See owner question 2 for what `repaint` should measure here; the rest can ship without it. | medium |
| Sample fields do not read `—` when their stage is stale | `right_dock.gd:466` `_build_sample` | **Source: `05-right-dock-and-bars.md` §1.4**, the verbatim footnote `fields owned by stale stages read —`, which `BUILD_ANSWERS.md` §3 moved from *declared but inert* to **wired** — i.e. settled design, not an open question. The shell has the staleness data (`app.gd:698-711`) and surfaces it only as a whole-bar message; `grep -n stale right_dock.gd` finds five hits, none a per-field gate. Build the field→stage ownership map and gate each `_field` on it. | medium |
| The right dock does not follow the armed tool | `right_dock.gd:439-465` `_dispatch`, titles `:350` | **Source: `05-right-dock-and-bars.md` §1.2/§1.3** — an `rdExtraMode()` first-match-wins ladder over the armed tool, adding nine contexts (`STAMP STACK · PAINT · BIOME · ANNOTATION · SETTLEMENT · TERRITORY · RAMP · STOPS · WAY · JOURNEY — RESULTS`). Each context's *content* already exists, relocated: paint legend `world_workspace.gd:1692`, ramp stops `render_workspace.gd:575-620`, annotation `cartography_workspace.gd:735`/`:1365`/`:1472`, territory cells+area `civilization_workspace.gd:766`, way draft count `infrastructure_workspace.gd:440`. What is genuinely absent is the *rule* that arming re-points the dock. **See owner question 1 before building.** | medium |
| Phone undo-history popover | `dcc_shell.gd:4224-4248` `_check_phone_undo_chip_hold` | **Source: `06-phone.md` §6.2** — a 520 ms hold opens a popover: header `EDIT HISTORY · TAP TO ROLL BACK`, rows `{i+1} · {action}` newest-first capped at 6, tapping row *i* reverts everything above it. Today it shows a toast (`Next undo: <label> (N steps saved)`). The stated reason at `:4224-4231` (`EngineBridge` exposes one label and counts, never an array) is **true but narrower than it reads**: the desktop already has a multi-step history view at `right_dock.gd:1230` `_build_history` with `_revert_history`, off `undo_ledger`/`undo_revert_to` (`lib.rs:12005-12155`). Reuse that path — no new binding is needed. | medium |
| Phone sim strip | absent — `grep -rn "sim_strip" shell/` finds nothing; `phone_menu.gd` routes MORE ▸ Simulation to the CIVIL Simulation category | **Source: `06-phone.md` §6.2**: `▶/⏸` in a 38 px accent circle · `YEAR {n}` · slider `min=-400 max=1200` · `×1 ×10 ×100` · `✕`; playing advances the year every 600 ms. Same year cursor as the desktop timeline row — build them against one model or they will diverge. | medium |
| App-bar `⋮` overflow | `dcc_shell.gd:2954-2961`, bar built `:2979-3042`; `phone_menu.gd:258-262`, `:284` | Both comments say `⋮` is a *contextual* overflow with nothing to put behind it. **The 2026-08-31 canvas defines it exactly**: `Cartalith Android.dc.html:89-95` (`hMenu`) and `:897` — a 230 px popover with `Save project` (+ `savedAt`), `Theme` (+ `themeLabel`) and `Close world`. All three destinations exist in this shell already. Under `CLAUDE.md`'s *"newer canvas wins"*, this became buildable on 2026-08-31. | medium |
| `drafts/paint.json` and `drafts/sculpt.json` are declared slots nothing writes | write site `app.gd:1322-1328`; slots `cartalith-io/src/project.rs:289`, asserted `project_bridge.rs:2054-2068`; codec `crates/cartalith-spatial/src/paint.rs:319` `encode_sparse`, `:335` `decode_sparse` | **Painted biome/terrain/splat layers and the sculpt draft stack are lost on save today.** The shell puts only `entities/journeys.json` into `documents`. Fill the two slots on save and read them on open; `encode_sparse`/`decode_sparse` are the `state.cartoPaint` persistence pair and exist unwired for exactly this. | medium |
| `library/assets.json` and `library/travel.json` are declared slots nothing writes | same write site; builders `crates/cartalith-assets/src/library.rs:725` `to_library_json`, `:851` `apply_library_file_with_items` | The asset library has no Export/Import pack round trip. `ops_bridge.rs:33-35` correctly records the blocker — *"they need `project_bridge.rs` to read and write the section"* — which is the same root cause as the row above, so ship them together. Add `project_document_slots()` (`engine_bridge.gd:2982`) validation at the call site while you are there: its own doc exists so a caller can catch a typo *"rather than discovering the typo as a failed save."* | medium |
| Global `Redo` | `menus.gd:504` | Reason is **true and verified**: `lib.rs:12005-12155` exposes `can_undo`/`undo_label`/`undo_last`/`undo_ledger`/`undo_revert_to`/`undo_stats` and **no `redo`**; the global stack pops rather than cursoring. The Sculpt draft does have one (`sculpt_redo`, `engine_bridge.gd:1359`, called at `right_dock.gd:1537`). Proposal: convert the global ledger to a cursor and add a `redo` `#[func]` — the ledger already records enough to revert *to* a point (`undo_revert_to`), which is most of the work. | medium |
| No content descriptions, no dynamic type | zero `accessibility_name`/`accessibility_description` in `shell/*.gd`; glyph buttons `dcc_shell.gd:2998` `☰`, `:3035` `⌕`, `:3038` `▤`, `phone_menu.gd:379` `←`/`✕` | Godot has no touch tooltip, so a `tooltip_text` on a phone reaches nobody and those five controls are unnamed. This is a **Godot 4.7** project (`project.godot:15`) whose AccessKit bindings exist. Set `accessibility_name` on every glyph-only control, and scale type off the OS font scale rather than `_phone_scale` (short side / 412) alone. **Source: `BUILD_ANSWERS.md:111-112` §4**, which rules *disclose rather than build* — the shell has done neither half. | medium |
| Previews re-upload the whole texture | `crates/cartalith-spatial/src/pass.rs:193` `touched_tiles`, `:199` `touched_bounds` | `touched_tiles`' own doc names the consumer: *"the exact set a renderer needs to re-upload for a preview, instead of the whole map."* Sculpt and Paint previews repaint everything today. `LOD_TILING_BASE_SCOPE.md`'s *"built ahead of its caller"* reason covers the rest of that crate's surface; this is the one piece with a live caller waiting. Worth costing before committing. | medium |
| No storage-full handling | nowhere — `grep` for disk / space / ENOSPC / free-bytes finds nothing | Check free bytes before a save or export and refuse with a real message rather than *"save failed — see console"*. **Source: `BUILD_ANSWERS.md:110` §4**, second half. | medium |
| `Save layout as…` | `menus.gd:2489` (`Reset layout` above it, `:2488`, is live) | Reason is **true** — every ingredient is live on `DccShell` and none is collected; `dcc_settings.gd` contains no `layout` key whatsoever. Proposal: collect dock visibility, sheet detents, active domain and rail state into one dictionary, persist a named list in `DccSettings` beside the existing machine state, and make `Reset layout` the built-in entry of that list. | medium |
| Atlas cache `Size cap · GB` | `menus.gd:2222` | Reason is **true** — `lib.rs` has `atlas_status` (`:7338`) and `atlas_clear` (`:7461`) and no per-chunk eviction, no access order, no level priority. Proposal: record last-access and level per chunk in `cartalith-io/src/atlas.rs`, add an `atlas_evict_to(bytes)` `#[func]`, then the cap is a settings key over it. `chunk_len` (`atlas.rs:263`) already exists to size the accounting cheaply. | medium |
| Manual road tool / `road_edges` never retained | `crates/cartalith-civ/src/lib.rs:5396` `build_road_network`; downstream cost recorded at `journey_bridge.rs:58-61` | Auto-populate builds roads by a different algorithm (`crates/cartalith-godot/src/lib.rs:897`) and `build_road_network` is *"the manual-tool algorithm this pipeline"* does not use (`lib.rs:149`) — a true reason for a tool that does not exist and that no scope document schedules. `journey_bridge.rs:58-61` records the consequence: *"`road_edges` is empty… `build_road_network`'s `RoadEdge` list is not retained"*, so the Journey Planner's second road source is permanently absent. Proposal: retain `RoadEdge` from the auto-populate pass first — that alone closes the planner gap — then schedule the manual tool separately. | medium |

---

## Large — a subsystem, a model, or a viewport that does not exist (16)

| Item | Where (`file:line`) | Proposal | Size |
|---|---|---|---|
| `Cut` · `Copy` · `Paste` · `Select all` | `menus.gd:547`, `:551`, `:552`, `:568` | Reasons are **true and verified**: no clipboard model exists, and `icon_get_selected` (`lib.rs:5856`) and `label_get_selected` (`lib.rs:7871`) each return a single `i64`, so there are three unrelated single-item selections and nothing to select *into*. `DisplayServer.clipboard_set` appears only in two text dumps (`gen_info_dialog.gd:81`, `journey_planner_view.gd:2328`). Proposal, in order: a selection *set* per entity kind engine-side; then a typed clipboard buffer; then the four commands. **This is a subsystem, not four menu rows.** The deliberate asymmetry with `Deselect` — which shipped (`menus.gd:583` → `app.gd:648`) — is explained correctly at `:571-582`. | large |
| CARTO ▸ Labels: the whole panel | `cartography_workspace.gd:1126-1131` (drawn counts `--`), `:1143-1148` (Class picker), `:1152-1154` (size / halo / tracking, `_dead_slider`) | Nothing generates labels automatically, so there is nothing to count, classify or style. `MapLabel` → `label_dict` (`lib.rs:7782-7798`) emits `x,y,text,angle,arc,size,size_mode,font,color` — **no class**, and no `halo` or `tracking` field at all (`grep halo` across the crates finds only `labels.rs:193`, the arc-label stroke width `max(1, size*0.16)`, a draw-time constant). `size` exists (8–48, `labels.rs:70-71`) but only per hand-placed label, which the live Region-labels section already edits. Proposal, in order: (1) a `label_class` field on `MapLabel`, which is what the live Class picker would then select against; (2) a generated labelling pass emitting per-class placements — that alone makes the drawn-count column real; (3) a per-class typography record carrying size/halo/tracking. Not transcribing the design's `4 · 11 · 48 · 22 · 37` (`:1051-1057`) was the right call and should stay. | large |
| Label collision culling | `cartography_workspace.gd:1160-1175` | Toggle drawn checked and disabled; the reason is **true** — label boxes are never measured against each other anywhere in the engine. Proposal: a label-box measurement and suppression pass over placed labels; the suppressed count in the note (`:1171-1175`) has nowhere to come from until it exists. Prerequisite for the icons *avoid label boxes* rule below. | large |
| CARTO ▸ Icons: generated placement | `cartography_workspace.gd:1250-1257` (four family chips), `:1268-1269` (scale / min spacing, dead), `:1276-1289` (three placement rules) | **True** — `icon_bridge.rs` only *arms* a family/slot for the Icon tool to stamp by click; no generated placement pass exists, so there is no set for a spacing to thin. Note `_icon_placement_family` is never written (read only, `:1294`/`:1298`), so PLACES is permanently the lit chip. Proposal: a generated placement pass, after which the two sliders and rules 1–2 bind to it (rule 1 also needs the label collision test above). Rule 3 (*snap sea marks to coast*) and the `SEA MARKS` chip need a decision first — **owner question 4**. Keep the separation the file already documents at `:1237-1247`: the *live* per-stamp scale/rotation/jitter belong in the tool-options bar (`:628-677`), not here. | large |
| The river entity does not cross the boundary | `right_dock.gd:790-791` (seven dashed fields), `:793-796` (three disabled Actions); CARTO prose `cartography_workspace.gd:389-409` | **True** — zero `get_rivers` / `river_*` `#[func]` in `cartalith-godot`. The context is also **unreachable**: nothing in the viewport can select a river, so `_dispatch` never routes there; it was built for completeness and says so at `:777-780`. Proposal: one `get_rivers()` binding returning polylines with `id, name, length_km, source_elev, discharge, catchment_km2, tributaries, navigable`, plus river hit-testing in the viewport. That single binding closes the dock context, the three Actions and CARTO's *rivers-as-ways* prose section together. Split the three Actions' shared seven-word tooltip when they land — the dashed fields above them carry the real explanation. | large |
| Civilisation authoring operations | `civilization_workspace.gd:971-973` Clear territory, `:1305-1307` Auto-populate world, `:1308-1310` Clear places & routes; `infrastructure_workspace.gd:964-966` Generate roads, `:967-969` Clear ways & journeys | All five are `func(): pass` + `disabled = true`, and all five reasons are **true**: `grep` finds no `civ_clear_territory`, `civ_populate`, `civ_clear_places` or `civ_auto_routes` in `crates/cartalith-godot/src/`; route generation runs inside `compute_civilisation` within `generate()`; neither `CivData::ways/sea_routes` nor `InfraTools::ways` has a clear binding (the tooltip correctly narrows what *can* be cleared — per-journey `route_delete`, IN-09). Proposal: expose the civ pipeline's own stages as five re-entrant `#[func]`s over an existing world, plus a civ parameter group — `PARAMS` groups today are `world · world_structure · planet · tectonics · volcanism · climate · weather · erosion` only. **The single largest CIVIL gap.** | large |
| Settlement diagnostics overlay | `civilization_workspace.gd:1311-1313` | **True** — every field the reference's card draws is urban-morphology data, and `cartalith-urban` (milestones 8–17) has no consumer; the crate is not even a dependency of `cartalith-godot`. Blocked on `URBAN_MORPHOLOGY_SCOPE.md`, the largest unported subsystem. | large |
| The 3D viewport | `menus.gd:1444` (AA · anisotropy), `:1448` (3D viewport defaults); phone FAB `viewport_host.gd:1033-1056` | **True and verified**: `grep -rl "Camera3D\|MeshInstance3D"` over `godot-project/shell/` and the `.tscn` files returns nothing. Deferred by `DECISIONS.md` §4 to Phase 3. The design's 2D/3D FAB toast points at `Preferences ▸ Graphics ▸ relief exaggeration` — build that Small row and the FAB becomes an honest mock instead of a toast with nothing behind it. | large |
| Colour management | `menus.gd:1446` | **True** — no colour-space binding exists anywhere in `lib.rs` (no `color_space` / `colorspace` / `Display P3` symbol); the renderer is sRGB end to end. Proposal: a colour space on the render target threaded through to the texture. A menu row ahead of that is a promise the pipeline cannot keep. | large |
| `Region ▸ New world from selection` | `crates/cartalith-engine/src/region_export.rs:277` `extract_region_as_world` | The resample is built and tested; the orchestration around it — `allocate`, clearing warp fields, cache invalidation, climate refresh, emptying the civ layer — is unported by design and doubly recorded (its own doc, and `ops_bridge.rs:25-28`: *"that is new `WorldGen` state in `lib.rs`"*). The reference **has** this button, so it is a real parity gap for a later pass, correctly declined for now. | large |
| Paint brush falloff | `world_workspace.gd:1683-1686`; `tool_bar.gd:433-436`; engine `crates/cartalith-godot/src/paint_bridge.rs:34-42`, `:190`, `:390-402` | Two live-looking Hardness sliders and one Softness slider write `Brush.hardness`/`softness`, echo the value back, and are **never consumed** — a dab is a hard disc. The engine's module doc is explicit: *"`hardness`/`softness` are accepted, not consumed"*, quoting `cartalith-spatial/src/paint.rs`, which quotes the reference verbatim. Proposal: add a falloff term to `PaintStamp`. **The alternative — deleting all three sliders — is a design decision with no ruling: owner question 5.** | large |
| Saved measurements + CSV | `right_dock.gd:1030-1043` | The canvas's *Saved measurements* list, *Save* and *CSV* are **not drawn at all**; only `Copy reading` and `Plan a journey` are. The reasoning at `:1030-1036` is argued rather than dodged — no measurement store exists, and inventing one is a persistence feature. Correctly not drawn as two disabled buttons. Proposal: a measurement store as a caller-owned save slot, which folds into the save-slot work above. | large |
| Landmark funnel: crowding and rejected candidates | `civilization_workspace.gd:2419-2423` (Lower crowding to fit), `:2425-2428` (Show rejected) | **True** — `landmark_funnels()` (`lib.rs:12630`) returns `landmark_funnel_dict` (`:12230-12246`), eight scalars only (`kind, candidates, rejected_{constraint,score,spacing,cap}, cap, placed, limit`). No crowding multiplier exists and none is derivable from counts; the dict carries **no coordinates** for rejected candidates and there is no map layer for them. Proposal: a crowding parameter on the placement pass, plus a rejected-candidate coordinate list and an overlay layer to draw it. | large |
| The manual-icon tool | `crates/cartalith-assets/src/manual.rs:189` `icon_brush_rule`, `:211` `icon_brush_stamp` | **True and scoped**: `ops_bridge.rs:29-32` — *"there is no manual-icon tool in the shell at all: nothing arms it, nothing renders `state.mapIcons`, and nothing stores them"*, `UNIFIED_TOOL_PLAN.md` milestone E. Verified: no icon-brush arming anywhere in `godot-project/shell/`. | large |
| Rebindable keyboard shortcuts | `menus.gd:1578` (Preferences, disabled); the live Help row is `:2578` → `app.gd:2756` | The reason was already corrected once and is now **true**: what is missing is *rebinding*, not the list. `shortcuts_dialog.gd` exists and walks the menus; `dcc_settings.gd` has no per-context shortcut store. Proposal: a per-context binding table in `DccSettings`, applied over the menu accelerators at build time, with conflict detection. | large |
| `Units` (km / mi) | `menus.gd:1571` | **True** — `dcc_settings.gd` has no unit key at all, and km is hard-coded at five call sites (status cursor coords, scale bar, Sculpt brush km, Measure totals, Region-select km column). Proposal: one formatter ahead of all five, plus the settings key. Large for reach, not difficulty — and flagged because `phone_menu.gd:84-85` quotes the locked phone spec verbatim as *"Preferences (theme dark/light + **units km/mi wired**)"*, so the phone spec promises what the phone shows disabled. | large |

*Two further Large rows are recorded without proposals because they need a
decision, not a design.* `CPU worker threads` (`menus.gd:1421` — reason
**true**: this port never calls `ThreadPoolBuilder`, so there is no `#[func]` to
set; the only hit in the whole workspace is a printout at
`cartalith-engine/examples/compute_config_bench.rs:359`) and `Report an issue`
(`menus.gd:2585` — reason **true**: no issue tracker, support address or crash
endpoint exists in this port to send to).

---

## The dangerous class — enabled and inert, or a reason that is false

A disabled row with an honest tooltip costs a user nothing. These cost them
trust. **25 items**, in three kinds.

### (a) Drawn ENABLED, does nothing meaningful (7)

| Item | Where | What actually happens | Severity |
|---|---|---|---|
| **Paint `Hardness`** — two copies on screen at once | `world_workspace.gd:1683`; `tool_bar.gd:433` | Fully draggable, writes through `paint_set_brush` → `Brush.hardness`, echoes back, **never consumed** (`paint_bridge.rs:34-42`, `:190`, `:390-402`). The control moves, the readout changes, the map does not. Disclosed in a tooltip only — and the WW-13 pairing puts both copies on screen simultaneously. | **highest** |
| **Paint `Softness`** | `world_workspace.gd:1685` | Same value, same path, same non-consumption. | high |
| **Biome-K checkbox reads the wrong state** | `new_world_dialog.gd:253`, `:460` | Not re-read from the engine on `about_to_popup` — the CheckBox return is discarded at `:253`, so there is nothing to `set_pressed_no_signal` on. After a load, the box reads unchecked while the engine has it on, and the next Create **silently turns it off**. | high — it changes a generation input |
| **`Preferences ▸ Theme` does not persist** | `menus.gd:210`, `:242`, `:1557-1568` | `_theme_mode` is a plain `var`; `DccSettings` has no theme section, and `:242` re-derives the mode from whatever `DccTheme` booted with — always dark. The radio looks like a setting and is a session toggle. | high |
| **`Window ▸ Status bar` on phone** | `app.gd:481`; `dcc_shell.gd:2692-2697` | The check mark moves; the node it flips lives inside `model_host`, which is `visible = false` permanently. | medium |
| **`Window ▸ Left/Right dock` and `Reset layout` on phone** | `app.gd:484`, `:1888-1892` | Write `node.visible`; the phone sheets' truth is `_left_sheet_open`/`_right_sheet_open`, which the system-back chain tests (`dcc_shell.gd:4478-4482`). `Reset layout` leaves both docks shown with both flags `false` — back will not close them. | medium |
| **CARTO ▸ Labels `Class` dropdown** | `cartography_workspace.gd:1143-1148`, sync `:1192-1216` | Live, but panel-local: repaints the class list ink, the `X · TYPE` title, and re-seats the three inert sliders on that class's design defaults. Writes nothing any renderer reads. **This is the deliberate version** — the one control that costs the engine nothing is left live so the frozen half reads as intentional, and the picker's own tooltip discloses it. Listed for completeness, not as a defect. | low, disclosed |

### (b) The stated reason is FALSE (9)

| Item | Where | The claim | The truth |
|---|---|---|---|
| **`State religion`** | `right_dock.gd:844-847` | *"cartalith-civ computes a has_religion flag internally … but `get_provinces()` doesn't carry it and there is no `get_faction_aggregates()` binding."* | Both halves are irrelevant. `get_factions()` carries `"religion"` (`lib.rs:6243`), and `_build_faction` **already fetched that dict** into `roster` (`:813-818`) and reads `culture` out of it at `:826`. `faction_roster_window.gd:436` reads *and edits* the same key. The value is one `.get()` away and is dashed with a reason about a different binding entirely. |
| **`pack.rs` painted layers** | `crates/cartalith-godot/src/pack.rs:24-45` | *"This port has never ported that tool — there is no producer of a painted-cell array anywhere in this workspace."* | False since 2026-08-24: `paint_bridge.rs:494-505`, `lib.rs:6307`, `lib.rs:4502`, and the drawn tool at `world_workspace.gd:1683` / `tool_bar.gd:433`. `render.rs:32-34` records the correction; `pack.rs` never got it. The consequence is invisible — a pack's painted-layer art is parsed for the warning count and silently never rendered, justified by a producer that now exists. |
| **`_refresh_phone_bar_lit()`** | `dcc_shell.gd:390-391`, `:3302-3321` | *"without this the bar said WORLD while the MORE screen was on top of it — a tab bar naming a destination the user is not at"*, and *"Read off live state rather than tracked, so every opener … need only call it"* | It has never run. `_phone_bar_more` and `_phone_bar_panels` are declared at `:390-391` and assigned nowhere in the repo, so `:3310` returns on all six call sites and `_light_bar_cell()` is unreachable. The doc also names "PANELS", which stopped being a tab when `PHONE_TABS` (`:3210`) became MAP/GENERATE/PLAN/MORE. The symptom it claims to prevent is live twice over: closing MORE leaves MORE lit, and selecting CIVIL leaves MORE lit. |
| **`Window ▸ Domain rail` on phone** | `dcc_shell.gd:3131-3134`, `:3157`, `:3188-3196` | *"What `Window ▸ Domain rail` hides is `_rail_region`, which here is only the three domain cells — PANELS and MENU stay. Hiding the whole bar would hide the MENU cell, and the menu is the only place the row that un-hides it lives: a one-way door."* | The comment describes a five-cell bar that no longer exists. The loop at `:3188` adds **every** `PHONE_TABS` cell — MORE included — under `_rail_region`. Unchecking the row builds exactly the one-way door the comment forbids, and leaves 64 dp of empty reserved strip behind it. |
| **First-run coach mark** | `dcc_shell.gd:4337-4339` | *"WORLD · CIVIL · CARTO switch domains here — PANELS and MORE reach everything else."* | Three of the four names are not captions in the bar, and PANELS is not a tab at all. Written against the pre-`PHONE_TABS` five-cell bar and not updated by the tab migration. Coach mark #2 (`:4341`, *"Drag this handle to expand tool options."*) is also wrong on a device that boots landscape, where the handle is inert by ruling. |
| **`No landmark types`** | `menus.gd:826-830` | *"EngineBridge carries no `landmark_kinds()`, so there is no vocabulary to list."* | `landmark_kinds()` is at `lib.rs:12453` and its own doc says it *"needs no generated world at all"*; the cache is filled at init (`engine_bridge.gd:3097`, read `:3104`) before `menus.build()` runs (`app.gd:289` → `:365`), so the `is_empty()` guard never fires. The branch is unreachable — worse in one specific way: it will never be seen, never be corrected, and reads to the next auditor as a live gap. |
| **Hero readout "deliberately unscaled"** | `right_dock.gd:1653-1656`; token `dcc_theme.gd:778-779` | *"`26` (`FS_HERO`) is left unscaled — §6's 'one big accent readout per context' is pinned"* | `BUILD_ANSWERS.md` §2.4 reverses this on 2026-08-31: *"The three unscaled values — all three now scale. They were oversights."* Stage 1 landed `fs_hero`/`fs_hero_2`; nothing consumes them, so the readout still does not grow on tablet — **and it reads as done because the constant exists.** `w_popover` (`dcc_theme.gd:766`) is the same shape. |
| **Tides have "no cartalith-engine equivalent yet"** | `world_workspace.gd:68` | Applied to geoid *and* tides together | True for geoid; **false for tides**. `passes.tidal_flats` (`params.rs:554`) is live, and its own engine doc says *"This port has no separate enable: this toggle is it, and turning it on computes the tide field"* (`cartalith-engine/src/lib.rs:354-364`). |
| **App-bar `⋮` has "no per-screen overflow to put behind it"** | `dcc_shell.gd:2954-2961`; `phone_menu.gd:258-262` | Was true against the 2026-08-30 canvas | The 2026-08-31 canvas defines it: `Cartalith Android.dc.html:89-95` + `:897` — Save project / Theme / Close world, all three of which exist in this shell. Under *"the newer canvas wins"*, the reason expired on 2026-08-31. |

### (c) Reason true, but the presentation misleads (9)

| Item | Where | Why it misleads |
|---|---|---|
| `CommandIndex` mints readouts as unavailable commands | `command_index.gd:129-142` | Its own header says the defect it fixed was *"entries in the list that a user could search for, tap, and get nothing from."* Searching "memory" or "working set" surfaces exactly that. |
| *"Not available for this world."* on permanent gaps | `layers_popover.gd:312-314` | Reads as a self-contradiction beside those rows' own *"Not available: no such estimator exists in this engine"*. |
| `layers_popover.gd:63` says eleven `GAP_LAYERS` | `layers_popover.gd:63-65` | Four. This is the one file that reasons about hotkey allocation from that number. |
| *"params.rs has 58 entries"* | `civilization_workspace.gd:1305-1307` | 81. A countable claim in a user-visible tooltip. |
| Icon fill line reads as measured data | `cartography_workspace.gd:1296-1299` | *"10 of 12 slots filled"* is a design figure, frozen on PLACES forever because `_icon_placement_family` is never written. The one line in the two CARTO panels that looks live and is not. |
| Viewshed note states the superseded weighting | `civilization_workspace.gd:1984-1990` | The panel presents *"weights it at 0.20"*; the owner's 2026-08-31 formula is `0.6 × prominence + 0.4 × visible land area inside 30 km`. Two different claims, and the panel shows the dead one. |
| Landscape drag handle drawn and inert | `dcc_shell.gd:3428-3440` vs. `:3602`, `:3552-3554` | The *behaviour* matches `BUILD_ANSWERS.md:108-109` exactly; the *affordance* is presented with no disclosure, and a coach mark points at it. |
| Disabled radios lose their radio mark on phone | `phone_menu.gd:798-801` | `Alternate frames` and `Reduce working res` read as rows rather than unselected choices in a group. |
| Three river Actions share one seven-word tooltip | `right_dock.gd:793-796` | *"No river binding to act on."* covers Hydrology, Edit geometry and Analyse catchment; the dashed fields directly above carry the real explanation. |

### The §4 disclosure scorecard

`BUILD_ANSWERS.md` §4 rules four items *"deliberate — disclose rather than
build."* **None of the four is disclosed anywhere a user can see.**

| §4 ruling | Built as ruled? | Disclosed? |
|---|---|---|
| Map canvas stays dark in light theme (`:106-107`) — *"a light map is a style preset, not a theme consequence"* | Yes, by omission — `viewport_host.gd` contains no `is_dark` / `rebuild_theme` / palette reference at all | **No.** Nothing on `Preferences ▸ Theme ▸ Light` says the map will not follow. |
| Rotation with a sheet open in landscape (`:108-109`) | **Yes** — `dcc_shell.gd:4761` re-snaps portrait, `:3602` / `:3553` no-op landscape. The one §4 item implemented as decided. | **No.** Handle still drawn and inert; coach mark still says to drag it. |
| Generation-failure and storage-full states (`:110`) | Failure states exist only in hidden status slots; **storage-full does not exist at all** | **No.** Both reach the phone only inside MORE, and one says *"see console"* on a device with no console. |
| Content descriptions and dynamic type (`:111-112`) | **No** — zero `accessibility_*`, zero OS font-scale reads | **No.** |

The fix for all four is small: one note on `Theme ▸ Light`, one visible failure
surface (`_show_phone_toast`), one hidden handle in landscape, and the
accessibility pass. Three are Small rows above; only the last is Medium.

---

## Not gaps — recorded so they are not re-listed next time

**Build-conditional `_todo`s, not permanent rows.** On a current build run from
the repository, **none of these five is drawn**: `menus.gd:1419` Devices,
`:1420` Multi-GPU mode, `:1427` VRAM budget, `:1428` Fallback when VRAM full —
all the `else` of `if _bridge.gpu_api` (`:1415`, `:1425`), which is true
whenever `lib.rs:2665` and `:2721` exist; and `:2565` Documentation, the `else`
of `_docs_dir() == ""`, live at `:2570` from a checkout. Also build-guarded and
correct: `world_workspace.gd:1005-1009` Erode (droplet), `:1499-1502` Count
painted lakes as water. **Any table that lists these flat is wrong.**

**Presented, permanently inert, and correct.** `menus.gd:1534-1537` Working set
readout (§2.5's *"read-only"*); `:1806-1810` VRAM estimate readout;
`:1673-1676` software-rasterizer device rows (*"Listed rather than hidden
because it is genuinely what the system enumerates"*); `:1563-1566` `Follow
system` (disabled only where `DisplayServer.is_dark_mode_supported()` is false);
`:1741-1746` `Alternate frames` and `:1827-1831` `Reduce working res` — both
**enforced in the engine**, `gpu_set_multi_mode` (`lib.rs:2721-2731`) and
`gpu_set_vram_fallback` (`lib.rs:2768-2777`) each returning `false` for them, so
row, tooltip and binding agree; `dcc_shell.gd:2884-2895` the phone's `▲ ▮▮ --`
band (Godot 4 has no battery API — checked against `ClassDB.class_get_method_list`
on `OS` — and the prototype's own band is decorative too).

**State-driven disables that re-enable on their own.** `menus.gd:378-404`,
`:461`, `:605`, `:1401-1406`, `:1661`, `:1688`, `:1695`, `:2058-2063`, `:2191`,
`:2268`, `:2379`, `:2543`; `cartography_workspace.gd:576-580` (Thicken ways by
carried volume — exemplary, state-driven in both directions and funnelled
through one setter) and `:745-764` (Clear all labels/icons at count 0, CA-20);
`civilization_workspace.gd:2226` (cap slider on a `buildable == false` type,
sourced from the engine's own flag so it cannot drift); `world_workspace.gd:721`,
`:779`; `right_dock.gd:1443-1459`, `:1493`; `tool_bar.gd:166-168`, `:508-511`,
`:531-545`. Deliberately tooltip-less chrome, correctly excluded from the
command index: `menus.gd:342`, `:368`, `:900`, `:1060`, `:2221`.

**Deliberate omissions, argued in code.** CIVIL `POI`
(`civilization_workspace.gd:498-504` — `civ_tools_bridge.rs`'s module doc says
POI *"is not a ported concept"*; omission over a fake control is the house
rule); WORLD palette Sculpt/Freehand (`world_workspace.gd:236-242` — Freehand
is a `FeatureParams` variant, not a separate tool); the tool-bar trailing notes
for Flatten/Noise/Mask (`tool_bar.gd:273-274`), Water/Lithology/Mask (`:361-362`)
and `Custom ▾`/`3D distance` (`:550-552`), each read off the engine registry
rather than the canvas; `phone_project_picker.gd:52-58` (no per-world finalized
flag, resolution or on-disk size is tracked for a project that is not open);
`dcc_shell.gd:3592-3599` (the tool sheet's `peek` floor — a stated departure:
the sheet here is the desktop tool-options bar, which has no "gone" state on the
other two form factors); `phone_menu.gd:677` (*"No destination is wired to this
row."* — an unreachable guard, not a live gap: `GROUPS` at `:114-120` and the
match at `:660-672` cover all three cases).

**`Data ▸ Conversion` must not be re-added.** `03-menu-bar.md` §6.4 still draws
`CONVERSION` with `Coordinate systems (EPSG)` and `Format conversion`. Owner
decision 2026-08-20 removed it outright (`menus.gd:1235-1243`,
`GUI_GAP_REGISTER.md` §7.4), and `CLAUDE.md`'s own working rules name this exact
row as the case where **the canvas is the stale party, not the shell**.

**The 9-colour-ramps note must not be copied across.** `BUILD_ANSWERS.md` §3
marks ramps *intended · inert* with the note *"a ramp sets the legend and export
LUT; the viewport keeps its built-in relief ramp in this build"*. **In this shell
that note would be false**: ramps are live end to end,
`render_workspace.gd:818-822` → `bridge.load_ramp_preset` → `_refresh_map()`,
gated on `bridge.ramp_api` (`engine_bridge.gd:128`, `:768`). The design
under-claims here; do not add the disclosure.

**The four items `BUILD_ANSWERS.md` §3 assigns to the Shell file are already
built here.** *Data-manager routes*: 14 over four groups
(`data_manager_window.gd:121-153`, `:156` `GROUP_ORDER`), with menu rows
generated from that table (`menus.gd:1263-1288`), five live and nine drawn with
a per-route `reason` — the design's "8 routes" is an undercount. *Asset
families*: eight, being what `cartalith-assets::slots`/`library` actually defines
(`asset_library_window.gd:129`), with the disclosure at `:218` shown on the
FAMILIES band and its count (`:1188`, `:1193`); the design's 24 is a finer
subdivision (`Feature icons` → `Trees & cover` / `Rock & scree`) that no Rust
type draws. *Slot grids*: `:1733` `_build_slot_grid`, cells `:2068`, two-column
phone variant `:199`/`:987`. *Per-slot data*: `:2377` `_build_inspector`, live
transform writing `as_set_item_transform` (`:1703`), Fit/Reset `:2485`/`:2488`.

**Superseded twins — the sibling is the production path, and nothing should call
these.** `jp_plan` (`cartalith-civ/src/lib.rs:12926`), `jp_capacity` (`:9992`),
`jp_calc_water` (`:10816`), `place_settlements` (`:4613`),
`name_and_populate_settlements` (`:5006`), `civ_resync_next_tid`
(`timeline.rs:499`); the four one-shot GPU wrappers `warp_grid_gpu` /
`heterogeneity_grid_gpu` / `gauss_blur_grid_gpu` / `assign_plates_grid_gpu`
(`cartalith-gpu/src/lib.rs:2837-2877`), `flow_accumulation_gpu_with` (`:2536`),
`warp_band_gpu_with` (`:2922`), `gpu_resistance_grid_cpu` (`:2768`),
`init_gpu_f64` (`:398`, pilot residue); `build_tide_field` (`tides.rs:121`),
`compute_affordance_fields` (`cartalith-civ/src/lib.rs:294`), `unpack_rgb8`
(`channel_atlas.rs:164`), `badge()` (`landmark.rs:101` — the shell derives the
same strings at `civilization_workspace.gd:2790` and defends the derivation),
`closes()` (`landmark.rs:509`), `family_summary` (`:671`), `compat_value`
(`belief.rs:420`), `jp_fmt_days` (`cartalith-civ/src/lib.rs:13757`), `chunk_len`
and `get_meta` (`cartalith-io/src/atlas.rs:263`, `:332`), `filled_count`
(`cartalith-assets/src/library.rs:511` — the shell computes it correctly from
live engine data at `asset_library_window.gd:1315-1343`), `civ_has_faction_colors`
(`engine_bridge.gd:1799`), `set_biome_k_enabled` (`:1870` — the *capability* is
wired through `generate()` at `:432-433` ← `new_world_dialog.gd:254`/`:482`; only
the wrapper is redundant), `sculpt_stroke_point_count` (`:1289`),
`project_engine_owned_slots` (`:2988`).

**`audit_wiring.py` false positive.** `render.rs:1330` `js_reference` carries
`#[allow(dead_code)]` and is called from `golden_parity_render.rs`; the harness
excludes `tests/`, so a tests-only caller reads as none.

**Declines verified true.** `cartalith-spatial`'s accessor surface
(`lib.rs:176`, `:195`, `:240`, `:326`, `:486`, `:581`, `:593`;
`staleness.rs:165`, `:230`; `paint.rs:123`, `:219`, `:265`, `:307`) —
`LOD_TILING_BASE_SCOPE.md`'s *"why standalone, not wired in"*; `merge_over`
(`paint.rs:307`) belongs to a Cartalith-editor export that does not exist, and
the renderer's 0.60 alpha tint **is** wired (`render.rs::land_color`'s `paint`
parameter via `lib.rs:4502`). `arc_label_line_width` (`ops_bridge.rs:157`) — all
three claims in its doc check out (`map_overlay.gd:1482`, `:1495`; the file takes
`_labels` as data and holds no bridge reference). `project_read_document`
(`project_bridge.rs:1624`) — both claims confirmed at `open_project_dialog.gd:42`,
`:49`, `:106`, `:658`, and journeys ride `project_open`'s `documents`.
`civ_zoom_pick_r` (`cartalith-civ/src/tools.rs:110`) — disclosed at the call site
(`lib.rs:6027-6034`). `clip_convex` (`cartalith-urban/src/geom.rs:352`) — the
whole crate is pre-integration. `slot_paths` / `referenced_files`
(`manifest.rs:261`, `:281`) — internal cleanup, not a UI gap (`ops_bridge.rs:36-38`).
`Undo depth 1–50` — deliberately substituted by a memory-budget ladder
(`menus.gd:54`, reason at `:51-53`, *"Budgets, not step counts"*), with depth
reported read-only at `:627`; defensible, though the two do not present the same
promise and nothing tells the user of the substitution.

**Prose "Not built" sections — class 4, declined with the reason in place, no
controls drawn.** `cartography_workspace.gd:376-387`, `:389-409`, `:458-467`,
`:497-518`; `civilization_workspace.gd:927`, `:2923`, `:3221`, `:3503` (each
tagged *"needs a decision"* rather than *"needs wiring"* — correct: they name a
missing model, not a missing binding); `infrastructure_workspace.gd:546-553`;
`render_workspace.gd:464-470` with the reasoning stated at `:1015-1019`
(*"disabled sliders would imply that many separate gaps"*), and `:1020-1047`;
`world_workspace.gd:1447-1454`, correctly identified as new unscoped design work
rather than a port gap — none of it is in the reference either.

---

## Owner questions

1. **Does `rdExtraMode()` replace the right dock's ten selection contexts, or sit
   beside them?** `05-right-dock-and-bars.md` §1.2/§1.3 gives a first-match-wins
   ladder over the *armed tool*; `right_dock.gd:439-465` drives contexts from
   *selection events*. Both are coherent, and merging them naively makes the dock
   flip away from a selected settlement the moment a tool arms. **Blocks the
   Medium row.**
2. **What should `statusMid`'s `repaint NN ms` measure in a Godot build?** The
   prototype times a canvas repaint. This shell composites through
   `ViewportHost`/`map_overlay.gd` with no equivalent single-pass timer — frame
   time, texture-upload time, or `_refresh_map()` wall time? **Blocks one field;
   the rest of the composite can ship without it.**
3. **Should the WORLD left-dock A/B switch come back?** `world_workspace.gd:248-256`
   records its removal on 2026-08-24 against the previous canvas; the 2026-08-31
   canvas draws it at `ENV:309-314` and the rail's `Generation pipeline` / `Sculpt`
   nodes reinstate the same two modes — but its captions and its gate
   (`ldSwitch` / `ldSwA` / `ldSwB`) are all in the truncated tail
   (`02-rail-and-domains.md` §8 item 6), so there is no label to build it with.
4. **How do the design's four icon *placement* families map onto the engine's
   three *asset* families?** `SEA MARKS` has no counterpart in `ICON_FAMILIES` at
   all (`cartography_workspace.gd:1219-1226` says so), and the *snap sea marks to
   coast* rule (`:1287-1289`) names it. **Blocks part of the CARTO Icons row.**
5. **Paint falloff: bind it, or delete all three sliders?**
   `world_workspace.gd:1683-1686` and `tool_bar.gd:433-436` put two live-looking
   copies of the same dead value on screen at once. `BUILD_ANSWERS.md` does not
   mention paint falloff, and the reference itself has no soft falloff for
   painting — `cartalith-spatial/src/paint.rs`'s module doc quotes it verbatim:
   *"a hard disc… unlike `sculpt()`/`brushHeight` there's no soft falloff here"* —
   so **binding it would be a deliberate divergence from the reference, not a
   parity fix.** This is the highest-severity row in the document and it has no
   ruling.
6. **Should a committed sculpt stamp re-evaluate when sea level moves?**
   `crates/cartalith-terrain/src/sculpt.rs:1076` `with_sea_level` exists and
   nothing calls it. `SculptStamp::sea_level`'s doc calls itself *"the explicit
   stand-in for the reference's live `state.seaLevel` read"*, which implies the
   reference re-reads live and this port snapshots. Today, changing sea level
   after sculpting leaves committed stamps unre-evaluated.
7. **Are the four unwritten caller-owned save slots deliberate or an oversight?**
   `project_bridge.rs:2054-2068` asserts the shell may write five;
   `app.gd:1322-1328` writes one. No document states whether `drafts/paint.json`,
   `drafts/sculpt.json`, `library/assets.json` and `library/travel.json` are
   scheduled or declined — and the four unwired functions that would serve them
   each cite a *different* blocker, so the slot list may predate them. **Two
   Medium rows depend on the answer, and today painted layers and the sculpt
   draft stack are lost on save.**
8. **Is `init_gpu_f64` (`cartalith-gpu/src/lib.rs:398`) kept or deleted?**
   `GPU_COMPUTE_PILOT_SCOPE.md` has no `f64` / `SHADER_F64` mention at all, so the
   pilot recorded no disposition for its own residue.
9. **Is the phone app bar's `☰` / `▤` pair now stale?** The 2026-08-31 Android
   canvas's app bar is `[world pill] · ⌕ · ⋮` (`Cartalith Android.dc.html:80-87`) —
   no `☰`, no `▤`. The shell was built to the superseded
   `design/android-2026-08-30/` canvas, and stages 1–2 covered tokens and the rail
   fold, not the phone app bar. Whether replacing them is in stage 3's scope is an
   owner call, not something the code answers.
10. **`--good` and `--accH`.** `01-frame-and-tokens.md` §6 items 12–13 record both
    as declared-and-never-used **in the prototype itself**, so a shell with no
    consumer may be fidelity rather than a gap.

### Left undetermined by this pass — work someone must do, not decisions to take

- Whether the disabled CARTO Labels/Icons panels still read as *inert* against the
  light theme's `#f4f2ee` ground. `_mark_inert()` (`cartography_workspace.gd:1321`)
  applies a flat `modulate` α 0.55 chosen against the dark palette's
  `text_ghost`/`text` ratio. Needs a light-theme capture, not a read.
- Whether the phone's `measure strip` / `label bar` / `way card` count as missing.
  Their content is reachable through the sheet and the tool-options bar
  (`tool_bar.gd`, `infrastructure_workspace.gd:440`), but the design specifies
  them as floating map-level overlays with their own `CLEAR`/`DONE`, `✕`/`ADD`,
  `CANCEL`/`COMMIT` affordances. Needs a handset run.
- The 44 dp vs 48 dp target sweep (`BUILD_ANSWERS.md:111-112`).
  `PHONE_TAP_MIN := 44` (`dcc_theme.gd:846`) and `H_PHONE_PILL := 48` (`:849`);
  which *drawn* targets land below 48 needs a measured run on a device.
- Whether `sculpt_stroke_point` can reject a point the shell has already appended
  to its own `_sculpt_stroke_points` (`world_workspace.gd:200`, `:1556`) — the only
  value `sculpt_stroke_point_count` would have is as that divergence check.
- Whether the `right_dock.gd` River context is reachable at all. `_build_river` is
  complete and `_dispatch()` has a branch for it, but no caller was found that sets
  the context to `"river"`; if nothing can select a river, those rows are
  unreachable rather than merely inert — a different and cheaper category.
- Landscape composition beyond the sheet handle: the 2026-08-31 Android canvas's
  `land` branch (`:1461`) was not compared line by line.
- Whether any `_todo` reason cites a `PARITY_AUDIT.md` section number that has
  since moved. Three reasons cite `§20`, `§23` and `§2.5`/`§2.7` by number
  (`menus.gd:548`, `:1421`, `:1444`, `:2585`); the *technical* claims were verified,
  the section numbers were not.

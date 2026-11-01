## DRAFT, not loaded by Godot. MS-7, the menu-bar half (Ruling DU Q7). From 1 180 dp of width the tablet shows
## the eight desktop menus whole. Below that it keeps a short bar, which Ruling DY (owner, 2026-11-01) makes
## `☰ · File · World · Records · Story`. The PC keeps its eight menus and their order (owner, 2026-10-31: *"Leave
## the pc menu as is"*); Ruling DY renames one of them, World, to Records, on every bar that has it.
## Target: `shell/menus.gd::build()`. Prerequisites: MS-1's classifier (draft 01), which publishes the layout's dp
## size through `DccTheme.layout()`; and `drafts/pc-shell/02_menus_world.gd` (with its `01`), which adds
## `_generate_in_world` and File's `_build_generate_world_row()`, read while `_file()` builds. If the PC drafts are
## applied after this one, their EDIT W1 (the shipped `build()`'s three bar lines) is already done here.
##
## **Ruling DY (2026-11-01).** The owner renamed the World menu Records (PQ6), moved Generate world to "File, after
## New world…", and chose `File · World · Records · Story` for the tablet's short bar over keeping three menus.
## So this `build()` no longer uses `_pc_world()` (the drafts' first PC World menu, Generate world then the records
## rows; never pasted, now gone) or `_tablet_world()` (the shipped merge of the two, Ruling CK Q2; retired by
## `drafts/pc-shell/02`):
##   - the whole bar (PC, phone, tablet from 1 180 dp): File (with Generate world after New world…) · Edit ·
##     Assets · Records (`_data()`) · Story · Preferences · Window · Help;
##   - the short bar: ☰ · File (without Generate world) · World (`_world()` alone: Generate world, Reset
##     generation parameters, Reset one stage, Generation pipeline / Sculpt, Seed & size…) · Records (the same
##     `_data()`) · Story.
## **The short bar's width is unmeasured.** Records adds one title of seven letters to a bar that starts at 600 dp
## (about 80 dp at the tablet's menu padding, read off the mockup's own sizing rule, not measured in Godot). The
## implementing session measures it at 600, 768, 1 024 and 1 179 dp, windowed, before calling this done: every
## title drawn inside the bar, none clipped and none overlapping the quick cluster (the README's draft 8 bar).
##
## **Corrected 2026-11-01 (menu parity check, `cartalith-native/docs/evidence/menu-parity-2026-11-01/`).** The
## whole bar's World was `_tablet_world if DccTheme.is_tablet() else _data`. Two faults, both measured by that
## check: replacing `build()` dropped the PC drafts' Generate world from the desktop, and on a wide tablet
## `_tablet_world` repeated Edit's Reset generation parameters and Reset one stage (`_world()` builds them, and the
## whole bar's Edit does too). Inventory of what the wide tablet no longer carries in a World menu (`MISTAKES.md`,
## "Delete a surface"): Generate world is File's second row (id 710, Ctrl+R; it read "Run pipeline" until Ruling
## DX PQ11 gave the command one label, `drafts/pc-shell/02` EDIT W5); Reset generation parameters and Reset one
## stage are in Edit; Generation pipeline and Sculpt are the WORLD rail's two modes; Seed & size… opens the same
## dialog as File ▸ New world…; every records row is in Records.
##
## A rotation or resize that crosses 1 180 dp changes which menus exist, so it goes through MS-1's mount path
## (spec §3.5), which rebuilds the bar and then the `CommandIndex` that walks it. Treat "whole bar or not" as
## part of the class the swap compares, so that crossing it triggers the rebuild. That is also why `build()` sets
## `_generate_in_world` in both branches: a rebuild on the same instance must not inherit the last bar's answer.
##
## Nothing is lost between the two forms: every row of `_overflow` (☰) lives in one of the eight menus
## (Undo, Redo and history in Edit; the asset library and Landmark types in Assets; Theme and Units in
## Preferences; Keyboard shortcuts and About in Help; Edit, Assets, Preferences, Window and Help themselves),
## and every row of the short bar's World has the home listed above. `MENU_INVENTORY.md` §2.1 lists ☰'s rows.
##
## NOT parse-checked (no Godot binary in the session that wrote it).

## The width, in dp, from which the tablet shows the eight menus whole (Ruling DU Q7).
const TABLET_WHOLE_BAR_DP := 1180.0

## Replaces `build()`. The tablet's short bar is ☰ · File · World · Records · Story (Ruling DY); the whole bar is
## the PC's eight menus, Records among them. The phone also takes the `else` branch, so MENU ▸ Records re-presents
## the same popup and MENU ▸ File carries Generate world. `_generate_in_world` is written before File is added,
## in both branches, because `_file()` reads it while it builds (`DccShell.add_menu()` calls the builder at once).
## `_preferences(p)` in full is safe on a tablet: its pop-out row appears only when `PopoutHost.supported()`.
## Must not add a World menu to the whole bar: there, "World" is the WORLD domain (Window ▸ Workspace ▸ World),
## and the records menu is Records.
func build(shell: DccShell, bridge: EngineBridge, host: Node) -> void:
	_shell = shell
	_bridge = bridge
	_host = host
	_theme_mode = DccSettings.theme_mode()
	var short_bar := DccTheme.is_tablet() and not _tablet_bar_whole()
	_generate_in_world = short_bar
	if short_bar:
		shell.add_menu("☰", _overflow)
		shell.add_menu("File", _file)
		shell.add_menu("World", _world)
		shell.add_menu("Records", _data)
		shell.add_menu("Story", _story)
	else:
		shell.add_menu("File", _file)
		shell.add_menu("Edit", _edit)
		shell.add_menu("Assets", _assets)
		shell.add_menu("Records", _data)
		shell.add_menu("Story", _story)
		shell.add_menu("Preferences", _preferences)
		shell.add_menu("Window", _window)
		shell.add_menu("Help", _help)
	_apply_theme_mode(_theme_mode)
	_connect_session_gate()

## True when a tablet is wide enough for the whole bar. Before the first classification (`layout()` is `{}`)
## it answers false, which is today's tablet bar, never a guess at a width.
static func _tablet_bar_whole() -> bool:
	var dp: Vector2 = DccTheme.layout().get("dp", Vector2.ZERO)
	return dp.x >= TABLET_WHOLE_BAR_DP

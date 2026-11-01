## DRAFT, not loaded by Godot. MS-7, the menu-bar half (Ruling DU Q7). From 1 180 dp of width the tablet shows
## the eight desktop menus whole. Below that it keeps the shipped `☰ · File · World · Story`. The PC bar is
## unchanged (owner, 2026-10-31: *"Leave the pc menu as is"*).
## Target: `shell/menus.gd::build()`. Prerequisites: MS-1's classifier (draft 01), which publishes the layout's dp
## size through `DccTheme.layout()`; and `drafts/pc-shell/02_menus_world.gd` (with its `01`), which adds
## `_pc_world()`, the World menu this `build()` uses on the PC and on the wide tablet. If the PC drafts are applied
## after this one, their EDIT W1 (the shipped `build()`'s World line) is already done here.
##
## **Corrected 2026-11-01 (menu parity check, `cartalith-native/docs/evidence/menu-parity-2026-11-01/`).** The
## whole bar's World was `_tablet_world if DccTheme.is_tablet() else _data`. Two faults, both measured by that
## check: replacing `build()` dropped the PC drafts' Generate world (`_pc_world`) from the desktop, and on a wide
## tablet `_tablet_world` repeated Edit's Reset generation parameters and Reset one stage in World (`_world()`
## builds them, and the whole bar's Edit does too). The mockup's wide tablet and Ruling DU Q7 ("the eight menus
## whole") both draw the desktop's World. Inventory of what the wide tablet's World no longer carries
## (`MISTAKES.md`, "Delete a surface"): its first row is the same command as Generate world (id 710, Ctrl+R; it
## read "Run pipeline" until Ruling DX PQ11 gave the command one label, `drafts/pc-shell/02` EDIT W5);
## Reset generation parameters and Reset one stage are in Edit; Generation pipeline and Sculpt are the WORLD
## rail's two modes; Seed & size… opens the same dialog as File ▸ New world…. Below 1 180 dp the tablet keeps
## `_tablet_world`, as shipped.
##
## A rotation or resize that crosses 1 180 dp changes which menus exist, so it goes through MS-1's mount path
## (spec §3.5), which rebuilds the bar and then the `CommandIndex` that walks it. Treat "whole bar or not" as
## part of the class the swap compares, so that crossing it triggers the rebuild.
##
## Nothing is lost between the two forms: every row of `_overflow` (☰) lives in one of the eight menus
## (Undo, Redo and history in Edit; the asset library and Landmark types in Assets; Theme and Units in
## Preferences; Keyboard shortcuts and About in Help; Edit, Assets, Preferences, Window and Help themselves),
## and every row of the narrow World (`_tablet_world`) has the home listed above.
## `MENU_INVENTORY.md` §2.1 lists ☰'s rows.
##
## NOT parse-checked (no Godot binary in the session that wrote it).

## The width, in dp, from which the tablet shows the eight menus whole (Ruling DU Q7).
const TABLET_WHOLE_BAR_DP := 1180.0

## Replaces `build()`. The tablet's narrow form is today's; the whole form is the PC's eight menus, World
## included (`_pc_world()`: Generate world · Ctrl+R first, then the World rows). The phone also takes the
## `else` branch, so MENU ▸ World re-presents the same popup.
## `_preferences(p)` in full is safe on a tablet: its pop-out row appears only when `PopoutHost.supported()`.
func build(shell: DccShell, bridge: EngineBridge, host: Node) -> void:
	_shell = shell
	_bridge = bridge
	_host = host
	_theme_mode = DccSettings.theme_mode()
	if DccTheme.is_tablet() and not _tablet_bar_whole():
		shell.add_menu("☰", _overflow)
		shell.add_menu("File", _file)
		shell.add_menu("World", _tablet_world)
		shell.add_menu("Story", _story)
	else:
		shell.add_menu("File", _file)
		shell.add_menu("Edit", _edit)
		shell.add_menu("Assets", _assets)
		shell.add_menu("World", _pc_world)
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

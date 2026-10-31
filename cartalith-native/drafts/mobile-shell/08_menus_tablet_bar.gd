## DRAFT, not loaded by Godot. MS-7, the menu-bar half (Ruling DU Q7). From 1 180 dp of width the tablet shows
## the eight desktop menus whole. Below that it keeps the shipped `☰ · File · World · Story`. The PC bar is
## unchanged (owner, 2026-10-31: *"Leave the pc menu as is"*).
## Target: `shell/menus.gd::build()`. Prerequisite: MS-1's classifier (draft 01), which publishes the layout's dp
## size through `DccTheme.layout()`.
##
## A rotation or resize that crosses 1 180 dp changes which menus exist, so it goes through MS-1's mount path
## (spec §3.5), which rebuilds the bar and then the `CommandIndex` that walks it. Treat "whole bar or not" as
## part of the class the swap compares, so that crossing it triggers the rebuild.
##
## Nothing is lost between the two forms: every row of `_overflow` (☰) lives in one of the eight menus
## (Undo, Redo and history in Edit; the asset library and Landmark types in Assets; Theme and Units in
## Preferences; Keyboard shortcuts and About in Help; Edit, Assets, Preferences, Window and Help themselves).
## `MENU_INVENTORY.md` §2.1 lists ☰'s rows.
##
## NOT parse-checked (no Godot binary in the session that wrote it).

## The width, in dp, from which the tablet shows the eight menus whole (Ruling DU Q7).
const TABLET_WHOLE_BAR_DP := 1180.0

## Replaces `build()`. The tablet's narrow form is today's; the whole form is the PC's eight menus, except that
## World stays `_tablet_world`, so the tablet keeps Run pipeline · Ctrl+R and its generation rows.
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
		shell.add_menu("World", _tablet_world if DccTheme.is_tablet() else _data)
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

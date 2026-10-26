extends Node
class_name PopoutKeyForwarder

## Carries the three global keys out of a native pop-out (`POPOUT_WINDOWS_SCOPE.md` s.2.5 and PW-2
## bar b). A key typed in a native window goes to ITS viewport, so `DccApp._unhandled_key_input` and
## the main window's menu accelerators never see it (PW-0, by event delivery): Ctrl+Z, Ctrl+Y and
## Ctrl+S would go dead the moment a panel left the main window.
##
## Added as a child of a native window by `PopoutHost.prepare()` (never to an embedded one, and
## never when the setting is off, so it costs nothing there). It listens in `_unhandled_key_input`,
## i.e. only for a key the window's own controls did not take: a focused `LineEdit` keeps its own
## Ctrl+Z. `PopoutHost.forward_action()` decides which command a key means; this node runs it on
## the app.
##
## What it must never do: act on a key while the window is embedded (the main window already
## handles it there), act on an echo or a key-up, or call the engine itself -- it calls the same
## `DccApp.undo_last()`, `redo_last()` and `save_project()` the menu rows call, which carry their
## own generating guard.

func _unhandled_key_input(event: InputEvent) -> void:
	var host := get_parent() as Window
	if host == null or host.is_embedded() or not (event is InputEventKey):
		return
	var k := event as InputEventKey
	if not k.pressed or k.echo:
		return
	var focus := host.gui_get_focus_owner()
	var typing := focus is LineEdit or focus is TextEdit or focus is SpinBox
	var act := PopoutHost.forward_action(k.keycode, k.is_command_or_control_pressed(), k.shift_pressed, k.alt_pressed, typing)
	if act == "":
		return
	## The app is the window's `app` property when it has one; `CityViewerWindow` keeps only the bridge
	## and was handed its host as a `setup()` argument, so the window's parent -- the `DccApp` every tool
	## window is added to -- is the fallback (PW-3: without it Ctrl+Z, Ctrl+Y and Ctrl+S would be dead in
	## the City viewer pop-out; `_popout_windows_probe.gd` sends them to every window). A window with
	## neither does nothing, as before.
	var app = host.get("app")
	if app == null:
		app = host.get_parent()
	if app == null or not app.has_method("undo_last"):
		return
	match act:
		"undo":
			app.undo_last()
		"redo":
			app.redo_last()
		"save":
			app.save_project()
	host.set_input_as_handled()

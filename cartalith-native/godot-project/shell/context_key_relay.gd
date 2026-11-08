extends Node

## PC-7. Carries the menu bar's chords out of a non-blocking context window (`context_window.gd`).
##
## **Measured on Godot 4.7.2** (`_pc7windows_probe.gd`, leg K): while an embedded window has the
## focus, the main viewport hands every key to it, and a key the window does not use is dropped
## there; it never reaches the menu bar. So with the roster focused, Ctrl+F opened no Find and Q (Edit
## ▸ Cartwheel at the pointer) no cartwheel until the user clicked the map; every other accelerator
## goes the same way, since they are all the menu bar's. A modal window had the same hole, but nobody
## could reach the map then; a window that sits beside the map needs the menu keys.
##
## Added as a child of each window `ContextWindow.make_context()` makes non-blocking (the desktop
## only). It listens in `_unhandled_key_input`, so a key one of the window's own controls took (a
## field's Ctrl+Z, Ctrl+A, an arrow) never reaches it. For a key `relays()` accepts it does what
## `MenuButton::shortcut_input` does for the main window: asks each menu's popup to run the item whose
## accelerator matches (`PopupMenu.activate_item_by_event`), so the row runs exactly as the accelerator
## runs it, with its own guards, and the window keeps the focus.
##
## What it must never do: pass a key with no Ctrl/Cmd or Alt (Q, Shift+A, Delete, Escape and every
## letter are the map's or the field's; the brief's hazard "typing in a field must not trigger map
## accelerators"); pass an Alt chord or a text-editing chord while a field has the focus (Ruling DX
## PQ12: Alt+arrows do nothing in a text field); act on an echo, a key-up, a window the busy overlay
## muted, or a native pop-out (`PopoutKeyForwarder` carries its keys).

## Ctrl/Cmd chords a focused text field owns even when it does not consume them in this build, so they
## are never passed on while one has the focus: select all, copy, cut, paste, undo, redo, and the
## word moves and deletes.
const FIELD_CHORDS: Array[int] = [KEY_A, KEY_C, KEY_X, KEY_V, KEY_Z, KEY_Y, KEY_LEFT, KEY_RIGHT,
	KEY_UP, KEY_DOWN, KEY_HOME, KEY_END, KEY_BACKSPACE, KEY_DELETE]

## The rule as a pure function, so a table of literals pins each term. A key is passed to the menu
## bar only with Ctrl/Cmd or Alt held; while a field types (`typing`), only a Ctrl/Cmd chord that is
## not one of `FIELD_CHORDS`, and never an Alt chord.
static func relays(keycode: int, cmd_or_ctrl: bool, alt: bool, typing: bool) -> bool:
	if not cmd_or_ctrl and not alt:
		return false
	if typing:
		return cmd_or_ctrl and not alt and not FIELD_CHORDS.has(keycode)
	return true

## Takes the keyboard focus away from embedded window `dlg`, as a left press on the map does. The
## engine has no call for "no embedded window has the focus"; the one public path to it is the one a
## closing window takes (`Viewport::_sub_window_remove` hands the focus to the closing window's parent,
## which for a child of the shell is the main viewport itself). So a 1 x 1 transparent helper window is
## shown, given the focus and hidden again in the same call, and freed: nothing is drawn (it never
## lives to a frame), and the window's own field keeps its caret position for the user's return.
static func drop_focus(dlg: Window) -> void:
	var parent := dlg.get_parent()
	if parent == null:
		return
	var h := Window.new()
	h.name = "Pc7FocusDrop"
	h.borderless = true
	h.transparent = true
	h.size = Vector2i(1, 1)
	h.position = Vector2i.ZERO
	h.visible = false
	parent.add_child(h)
	h.show()
	h.grab_focus()
	h.hide()
	h.queue_free()

func _unhandled_key_input(event: InputEvent) -> void:
	var dlg := get_parent() as Window
	if dlg == null or not dlg.is_embedded() or dlg.gui_disable_input or not (event is InputEventKey):
		return
	var k := event as InputEventKey
	if not k.pressed or k.echo:
		return
	var focus := dlg.gui_get_focus_owner()
	var typing := focus is LineEdit or focus is TextEdit or focus is SpinBox
	if not relays(k.keycode, k.is_command_or_control_pressed(), k.alt_pressed, typing):
		return
	var host = dlg.get_parent()
	var row = host.get("menu_bar_row") if host != null else null
	if not (row is Node):
		return
	## A command that puts the keyboard focus into the main window needs nothing more here: Find's
	## `grab_focus()` on its field takes the focus from this window by itself, so the next letter
	## lands in Find (measured: `_pc7windows_probe.gd` leg K passes with no extra call: a mutant run
	## that disabled an earlier one changed nothing, so it was removed).
	for mb in (row as Node).get_children():
		if mb is MenuButton and (mb as MenuButton).is_visible_in_tree() and not (mb as MenuButton).disabled:
			if (mb as MenuButton).get_popup().activate_item_by_event(k, false):
				dlg.set_input_as_handled()
				return

extends Node
## `Help ▸ Keyboard shortcuts` probe.
##
##   godot --headless --path . --resolution 1600x900 _shortcuts_probe.tscn
##
## The dialog's whole design claim is that it reads accelerators back off the
## LIVE menus rather than carrying a written table, so the only test that means
## anything is one that boots the real shell and checks the rows it produces
## against the accelerators `menus.gd` actually registered.
##
## A dialog that lists nothing would satisfy every structural check, which is
## the silently-empty-output trap this repository has been bitten by four
## times -- so the row count is asserted non-zero and specific shortcuts are
## named.

var _vp: SubViewport
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _texts(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	for c in n.get_children(true):
		_texts(c, out)

## `id -> {"popup": PopupMenu, "index": int}`, read straight out of the live
## dialog's own bookkeeping (`ShortcutsDialog._capture_popups`) rather than
## re-walked here -- a second walk would be a second place for this probe to
## disagree with what the dialog itself thinks a row is.
func _live_accel(dlg, id: int) -> int:
	var pi: Dictionary = dlg._capture_popups.get(id, {})
	if pi.is_empty():
		return -1
	return (pi["popup"] as PopupMenu).get_item_accelerator(int(pi["index"]))

## Simulates the real two-step UI gesture -- click the chip (a real `pressed`
## signal emission, so it runs the SAME connected lambda a mouse click would),
## then a real keypress dispatched through `_unhandled_key_input` itself
## rather than a re-implementation of what it should do. Building and pushing
## a synthetic OS-level event through `dlg.push_input()` was the more
## end-to-end option; called directly instead because a headless run with no
## window focus makes that routing unverified, and this still exercises the
## production method exactly as `_begin_capture`'s own connection reaches it.
func _rebind_via_ui(dlg, id: int, keycode: int, ctrl: bool, shift: bool, alt: bool) -> void:
	var chip: Button = dlg._capture_chips.get(id)
	if chip == null:
		print("  [FATAL] no chip for id ", id)
		return
	chip.pressed.emit()
	var ev := InputEventKey.new()
	ev.pressed = true
	ev.keycode = keycode
	ev.ctrl_pressed = ctrl
	ev.shift_pressed = shift
	ev.alt_pressed = alt
	dlg._unhandled_key_input(ev)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return

	## === 0: a rebind from an EARLIER session restores on boot ===
	## Seeded before `app.tscn` even loads, so `menus.gd`'s own `build()` --
	## which runs exactly once, during this instantiate -- is the thing under
	## test: "applied over the menu accelerators at build time" is a claim
	## about THIS moment, not about the editor dialog. The static cache is
	## force-cleared after seeding so the boot below reads it back off disk
	## rather than off the ConfigFile object still sitting in memory --
	## otherwise this would only prove the setter and getter agree with each
	## other, which `cpu_thread_count`'s own three-day bug shows is not the
	## same thing as restoring.
	const SEEDED := KEY_MASK_CTRL | KEY_MASK_ALT | KEY_N
	DccSettings.set_shortcut_binding(DccSettings.SHORTCUT_CONTEXT_MENU, 10, SEEDED)   # ID_NEW_WORLD
	DccSettings._loaded = false
	DccSettings._cfg = null

	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(50)
	print("[BOOT] shell up")

	var dlg = app.get("shortcuts_dialog")
	print("\n=== 1: the dialog exists and is wired ===")
	_ok("shortcuts_dialog constructed", dlg != null, true)
	if dlg == null:
		get_tree().quit(1); return
	_ok("app exposes open_shortcuts()", app.has_method("open_shortcuts"), true)

	print("\n=== 0: a rebind saved before this boot is live the instant menus.gd builds ===")
	dlg.open_editable()
	await _frames(4)
	_ok("New world's accelerator is the seeded override, not the shipped Ctrl+N",
		_live_accel(dlg, 10), SEEDED)
	_ok("... and DccSettings agrees (read back off the ConfigFile this boot loaded)",
		DccSettings.shortcut_binding(DccSettings.SHORTCUT_CONTEXT_MENU, 10, -999), SEEDED)
	## Put it back now, not only at the very end -- section 3 below still
	## checks for the SHIPPED "Ctrl+N", and this probe's own sections should
	## not have to know about each other's seeds to stay green.
	dlg._reset_one(10)
	await _frames(2)

	# Build its contents the way the menu row does.
	dlg.open()
	await _frames(4)

	var rows: Array = []
	_texts(dlg, rows)
	print("\n=== 2: it produced real rows, not an empty list ===")
	print("  info label count in dialog: ", rows.size())
	_ok("more than the header labels", rows.size() > 8, true)

	var joined := "\n".join(rows)
	_ok("no 'no menu accelerators found' fallback fired",
		joined.findn("No menu accelerators found") < 0, true)

	# Accelerators menus.gd really registers -- if the walk works, these appear.
	print("\n=== 3: the accelerators menus.gd registers are present ===")
	for want in ["Ctrl+N", "Ctrl+O", "Ctrl+S", "Ctrl+W", "Ctrl+Z"]:
		_ok("lists %s" % want, joined.find(want) >= 0, true)

	print("\n=== 4: the non-menu shortcuts are listed and marked as such ===")
	_ok("lists the layer digits", joined.find("1 – 8") >= 0, true)
	_ok("lists space-to-pan", joined.find("Space") >= 0, true)
	_ok("has the 'Not on a menu' group", joined.findn("NOT ON A MENU") >= 0, true)
	_ok("names the file that owns a non-menu shortcut",
		joined.find("layers_popover.gd") >= 0, true)

	print("\n=== 5: reopening re-reads rather than accumulating ===")
	var first := rows.size()
	dlg.open()
	await _frames(4)
	var again: Array = []
	_texts(dlg, again)
	_ok("row count stable across reopen", again.size(), first)

	# The rank-1 finding from the Nortantis comparison: six Edit rows carried
	# "Nothing is selectable for editing yet beyond settlements, which are
	# read-only", which the build falsifies -- Delete has always worked from
	# the keyboard. Asserted here so that sentence cannot come back.
	print("")
	print("=== 6: the Edit menu no longer states a falsehood ===")
	var pops: Array = []
	_find_popups(app, pops)
	var edit: PopupMenu = null
	for pm in pops:
		for k in (pm as PopupMenu).item_count:
			if (pm as PopupMenu).get_item_text(k).begins_with("Undo history"):
				edit = pm
				break
		if edit != null:
			break
	_ok("found the Edit popup", edit != null, true)
	if edit != null:
		var del_i := -1
		var stale := 0
		for k in edit.item_count:
			if edit.get_item_text(k) == "Delete":
				del_i = k
			var tip := edit.get_item_tooltip(k)
			if tip.findn("which are read-only") >= 0 or tip == "Same.":
				stale += 1
		_ok("a Delete row exists", del_i >= 0, true)
		if del_i >= 0:
			_ok("the Delete row is live, not a _todo", edit.is_item_disabled(del_i), false)
		_ok("no row still claims settlements are read-only", stale, 0)
	_ok("app exposes delete_selection()", app.has_method("delete_selection"), true)
	_ok("delete_selection() is safe with nothing selected", app.delete_selection(), false)

	## PR-16/HE-02: the editor half. `dlg.open()` at section 5 above left the
	## dialog in read-only mode, so every section below re-enters editable
	## mode itself rather than assume the mode a previous section left it in.
	const ID_OPEN_PROJECT := 11
	const ID_SAVE_AS := 13
	const ID_CLOSE := 16
	const ID_UNDO := 20

	print("\n=== 7: a rebind takes effect live -- no restart, no menu rebuild ===")
	dlg.open_editable()
	await _frames(2)
	var open_default: int = app.menus.shortcut_default(ID_OPEN_PROJECT)
	_ok("Open project starts at its shipped default",
		_live_accel(dlg, ID_OPEN_PROJECT), open_default)
	const REBOUND := KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_G
	_rebind_via_ui(dlg, ID_OPEN_PROJECT, KEY_G, true, true, false)
	await _frames(2)
	_ok("DccSettings stored the new binding",
		DccSettings.shortcut_binding(DccSettings.SHORTCUT_CONTEXT_MENU, ID_OPEN_PROJECT, -999), REBOUND)
	_ok("the LIVE popup item changed -- no rebuild needed to see it",
		_live_accel(dlg, ID_OPEN_PROJECT), REBOUND)
	_ok("has_shortcut_override is now true", DccSettings.has_shortcut_override(
		DccSettings.SHORTCUT_CONTEXT_MENU, ID_OPEN_PROJECT), true)
	_ok("capture ended (not stuck reading 'Press a key...')", dlg._capturing_id, -1)
	var chip1: Button = dlg._capture_chips.get(ID_OPEN_PROJECT)
	_ok("the chip's own label updated", chip1 != null and chip1.text == OS.get_keycode_string(REBOUND), true)

	print("\n=== 8: reset -- the way back from a rebind gone wrong ===")
	dlg._reset_one(ID_OPEN_PROJECT)
	await _frames(2)
	_ok("back to the shipped key", _live_accel(dlg, ID_OPEN_PROJECT), open_default)
	_ok("has_shortcut_override is false again", DccSettings.has_shortcut_override(
		DccSettings.SHORTCUT_CONTEXT_MENU, ID_OPEN_PROJECT), false)

	print("\n=== 9: a same-context collision is SHOWN, never BLOCKED (Blender's rule) ===")
	var save_as_default: int = app.menus.shortcut_default(ID_SAVE_AS)
	const SHARED := KEY_MASK_CTRL | KEY_MASK_ALT | KEY_G
	_rebind_via_ui(dlg, ID_SAVE_AS, KEY_G, true, false, true)
	await _frames(2)
	_ok("Save as... took the new key", _live_accel(dlg, ID_SAVE_AS), SHARED)
	_ok("no collision yet -- nothing else uses it", dlg._status.text.length(), 0)
	_rebind_via_ui(dlg, ID_CLOSE, KEY_G, true, false, true)
	await _frames(2)
	_ok("Close ALSO took the same key -- not refused", _live_accel(dlg, ID_CLOSE), SHARED)
	_ok("Save as... still has it too -- both fire, neither was silently dropped",
		_live_accel(dlg, ID_SAVE_AS), SHARED)
	_ok("the status line names the OTHER row sharing it",
		dlg._status.text.findn("Save as") >= 0, true)

	print("\n=== 10: a DIFFERENT context's key is never flagged -- the ruling's whole point ===")
	## Bare F is Freehand's real, shipped shortcut (`world_workspace.gd`, a
	## file this table cannot see and does not try to) -- picked precisely
	## because it is a real collision in the application, in a context this
	## dialog has no visibility into and must not pretend to police.
	dlg._status.text = "unset"
	_rebind_via_ui(dlg, ID_UNDO, KEY_F, false, false, false)
	await _frames(2)
	_ok("Undo took bare F", _live_accel(dlg, ID_UNDO), KEY_F)
	_ok("status was cleared, not left naming a false conflict",
		dlg._status.text, "")

	print("\n=== 11: Restore all to defaults -- the way back from the whole table ===")
	var undo_default: int = app.menus.shortcut_default(ID_UNDO)
	var close_default: int = app.menus.shortcut_default(ID_CLOSE)
	dlg._reset_all()
	await _frames(2)
	_ok("Save as... back to its shipped key", _live_accel(dlg, ID_SAVE_AS), save_as_default)
	_ok("Close back to its shipped key", _live_accel(dlg, ID_CLOSE), close_default)
	_ok("Undo back to its shipped key", _live_accel(dlg, ID_UNDO), undo_default)
	_ok("no override remains for any of the three",
		DccSettings.has_shortcut_override(DccSettings.SHORTCUT_CONTEXT_MENU, ID_SAVE_AS)
			or DccSettings.has_shortcut_override(DccSettings.SHORTCUT_CONTEXT_MENU, ID_CLOSE)
			or DccSettings.has_shortcut_override(DccSettings.SHORTCUT_CONTEXT_MENU, ID_UNDO), false)

	## Belt-and-suspenders past section 11's own reset: this probe seeded and
	## wrote to the SAME `user://cartalith_settings.cfg` a real session on
	## this machine uses, so the shared context this table lives in is left
	## clean no matter which assertion above failed first.
	DccSettings.reset_shortcuts(DccSettings.SHORTCUT_CONTEXT_MENU)

	print("\n_shortcuts_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)


func _find_popups(n: Node, out: Array) -> void:
	if n is PopupMenu:
		out.append(n)
	for c in n.get_children(true):
		_find_popups(c, out)

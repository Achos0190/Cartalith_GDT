extends Node
## Chord capture and the override store -- the two PR-16/HE-02 claims
## `_shortcuts_probe.gd` cannot make on its own.
##
##   godot --headless --path . _chordcap_probe.tscn -- --mode full
##   godot --headless --path . _chordcap_probe.tscn -- --mode seed
##   godot --headless --path . _chordcap_probe.tscn -- --mode verify
##   godot --headless --path . _chordcap_probe.tscn -- --mode clean
##
## `--mode` is the ONLY flag this probe reads, and an unknown argument is a
## FATAL rather than a silent default -- a probe whose header documents a flag
## it never reads measures the same thing three times and calls it three
## cases (`MISTAKES.md`, "write a probe's usage header").
##
## **`seed` -> `verify` -> `clean` are three SEPARATE godot processes**, which
## is the whole point of them. `_shortcuts_probe.gd` §0 proves the same
## restore by force-clearing `DccSettings`' static cache
## (`DccSettings._loaded = false`) inside one process; that is the strongest
## in-process form of the check and it still cannot see a `ConfigFile` that
## was never flushed to disk, a `user://` path that differs per launch, or a
## value that survives only because the object was still alive. Two processes
## can.
##
## `full` is the single-process half: what the capture control does with keys,
## and what the conflict line says when more than one other row already
## answers to the chord.

const MENU := "menu"                     ## DccSettings.SHORTCUT_CONTEXT_MENU
const ID_NEW_WORLD := 10
const ID_OPEN_PROJECT := 11
const ID_SAVE_AS := 13
const ID_CLOSE := 16
const ID_UNDO := 20

## Deliberately a chord nothing in `menus.gd` ships: Ctrl+Alt+N. If the
## verify pass read the SHIPPED Ctrl+N back it would pass on a store that
## never persisted anything at all.
const SEEDED := KEY_MASK_CTRL | KEY_MASK_ALT | KEY_N

var _vp: SubViewport
var _fail := 0
var _mode := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _done() -> void:
	print("\n_chordcap_probe[", _mode, "]: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

func _parse_args() -> bool:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--mode" and i + 1 < args.size():
			_mode = args[i + 1]
			i += 2
			continue
		print("[FATAL] unknown argument: ", args[i], "  (this probe reads --mode only)")
		return false
	if not _mode in ["full", "seed", "verify", "clean"]:
		print("[FATAL] --mode must be one of full|seed|verify|clean, got: '", _mode, "'")
		return false
	return true

## `id -> {"popup", "index"}` off the live dialog's own bookkeeping, the same
## way `_shortcuts_probe.gd` reads it -- a second independent walk would be a
## second place to disagree with what the dialog thinks a row is.
func _live_accel(dlg, id: int) -> int:
	var pi: Dictionary = dlg._capture_popups.get(id, {})
	if pi.is_empty():
		return -1
	return (pi["popup"] as PopupMenu).get_item_accelerator(int(pi["index"]))

func _rebind_via_ui(dlg, id: int, keycode: int, ctrl: bool, shift: bool, alt: bool) -> void:
	var chip: Button = dlg._capture_chips.get(id)
	if chip == null:
		print("  [FATAL] no chip for id ", id)
		_fail += 1
		return
	chip.pressed.emit()
	var ev := InputEventKey.new()
	ev.pressed = true
	ev.keycode = keycode
	ev.ctrl_pressed = ctrl
	ev.shift_pressed = shift
	ev.alt_pressed = alt
	dlg._unhandled_key_input(ev)

func _boot() -> Node:
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(50)
	print("[BOOT] shell up")
	return app

func _ready() -> void:
	await _frames(2)
	if not _parse_args():
		get_tree().quit(2)
		return
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return

	print("[cfg] ", ProjectSettings.globalize_path("user://cartalith_settings.cfg"))
	match _mode:
		"seed": await _seed()
		"verify": await _verify()
		"clean": await _clean()
		"full": await _full()

# -- process 1 of 3 ------------------------------------------------------------

## No shell boot at all: this process only writes. If the store needs a live
## menu bar to persist, `verify` in the next process will say so.
func _seed() -> void:
	print("\n=== SEED: write one override and exit ===")
	DccSettings.set_shortcut_binding(MENU, ID_NEW_WORLD, SEEDED)
	_ok("the setter reports it stored", DccSettings.has_shortcut_override(MENU, ID_NEW_WORLD), true)
	var p := ProjectSettings.globalize_path("user://cartalith_settings.cfg")
	_ok("the config file exists on disk after the write", FileAccess.file_exists(p), true)
	## Read the bytes back through a SECOND ConfigFile object, not the static
	## cache the setter just wrote through -- proving the setter and getter
	## agree with each other is not proving anything reached the disk.
	var fresh := ConfigFile.new()
	_ok("a fresh ConfigFile parses the file", fresh.load(p), OK)
	_ok("... and finds the seeded chord in the menu context",
		int(fresh.get_value("shortcuts_" + MENU, str(ID_NEW_WORLD), -999)), SEEDED)
	print("  [seed] wrote ", OS.get_keycode_string(SEEDED), " (", SEEDED, ") for ID_NEW_WORLD")
	_done()

# -- process 2 of 3 ------------------------------------------------------------

## A different process. Nothing is written here and no cache is cleared: the
## only reason `menus.gd` could hand out the seeded chord is that
## `_bind_accelerator()` read it back off disk while building the menu bar.
func _verify() -> void:
	print("\n=== VERIFY: a chord written by an EARLIER PROCESS is live at menu build ===")
	_ok("this process starts having loaded nothing yet", DccSettings._loaded, false)
	var app := await _boot()
	var dlg = app.get("shortcuts_dialog")
	if dlg == null:
		print("[FATAL] no shortcuts_dialog"); _fail += 1; _done(); return
	dlg.open_editable()
	await _frames(4)
	_ok("New world's LIVE menu accelerator is the chord process 1 wrote",
		_live_accel(dlg, ID_NEW_WORLD), SEEDED)
	_ok("the store agrees, read off this process's own file load",
		DccSettings.shortcut_binding(MENU, ID_NEW_WORLD, -999), SEEDED)
	_ok("has_shortcut_override survived the process boundary",
		DccSettings.has_shortcut_override(MENU, ID_NEW_WORLD), true)
	## The override is a LAYER, not a moved default: `menus.gd` must still
	## know the shipped key, or "reset" in the next process has nothing to
	## go back to.
	_ok("the SHIPPED default is still Ctrl+N underneath it",
		int(app.menus.shortcut_default(ID_NEW_WORLD)), KEY_MASK_CTRL | KEY_N)
	_ok("and the two are actually different, so this test can fail",
		int(app.menus.shortcut_default(ID_NEW_WORLD)) != SEEDED, true)
	_done()

# -- process 3 of 3 ------------------------------------------------------------

func _clean() -> void:
	print("\n=== CLEAN: leave the shared settings file as it was found ===")
	DccSettings.reset_shortcuts(MENU)
	_ok("no override remains for New world",
		DccSettings.has_shortcut_override(MENU, ID_NEW_WORLD), false)
	var fresh := ConfigFile.new()
	fresh.load(ProjectSettings.globalize_path("user://cartalith_settings.cfg"))
	_ok("... and the section is gone from the file on disk",
		fresh.has_section("shortcuts_" + MENU), false)
	_done()

# -- single process: what the capture control does ------------------------------

func _full() -> void:
	var app := await _boot()
	var dlg = app.get("shortcuts_dialog")
	if dlg == null:
		print("[FATAL] no shortcuts_dialog"); _fail += 1; _done(); return
	dlg.open_editable()
	await _frames(4)

	print("\n=== A: no generated row can be spelled with 'no chord' ===")
	## `_accel_text(0)` renders an em dash, and the walk that builds the table
	## only collects rows whose accelerator is non-zero -- so the dash is what
	## an ABSENT chord would look like and no row in the table is allowed to
	## be wearing it. Both halves asserted: the spelling, and that nothing
	## uses it.
	_ok("an absent chord spells as a dash, not as a plausible key",
		dlg._accel_text(0), "—")
	var zero_rows := 0
	var dash_rows := 0
	for entry in dlg._collect_from_menus():
		if int(entry[1] == "—"):
			dash_rows += 1
		var pi = entry[4] as PopupMenu
		if pi.get_item_accelerator(int(entry[5])) == 0:
			zero_rows += 1
	_ok("no generated row carries a zero accelerator", zero_rows, 0)
	_ok("no generated row renders as the dash", dash_rows, 0)
	_ok("the walk found real rows to say that about",
		dlg._collect_from_menus().size() >= 16, true)

	## **The three assertions above pass on a fresh boot whether or not the
	## dialog can be made to destroy a row, because no override exists yet.**
	## This drives the destruction instead. A verifier found it 2026-09-06:
	## `get_keycode_with_modifiers()` is 0 for an unmappable key, and
	## committing 0 persisted it as an override, after which `_walk_popup`
	## skipped the row -- the entry and its own rebind chip left the table, and
	## only Restore all brought them back.
	var rows_before: int = dlg._collect_from_menus().size()
	## Aim at a row the dialog itself says is rebindable: `_capture_chips` is
	## keyed by exactly those ids, and rows whose key is owned by `app.gd`
	## (Escape, Delete) deliberately carry no chip. Reading the dict beats
	## re-deriving the rule and disagreeing with it.
	var chip_ids: Array = dlg._capture_chips.keys()
	_ok("the dialog offers rebindable rows to aim at", chip_ids.size() > 0, true)
	var victim: int = int(chip_ids[0]) if chip_ids.size() > 0 else -1
	var was: int = DccSettings.shortcut_binding(
		DccSettings.SHORTCUT_CONTEXT_MENU, victim, -1)
	_rebind_via_ui(dlg, victim, 0, false, false, false)
	_ok("an unmappable key is refused, not persisted as 'no shortcut'",
		DccSettings.shortcut_binding(DccSettings.SHORTCUT_CONTEXT_MENU, victim, -1), was)
	_ok("and the row it was aimed at is still in the table",
		dlg._collect_from_menus().size(), rows_before)
	## **Disarm before leaving.** Refusing an unmappable key deliberately KEEPS
	## WAITING rather than aborting -- the user pressed something, and closing
	## silently would read as "it took it" -- so the capture is still armed here.
	## Section B tests Escape "while idle" and would be testing a live capture
	## instead, which is how this assertion first went red.
	var esc_out := InputEventKey.new()
	esc_out.pressed = true
	esc_out.keycode = KEY_ESCAPE
	dlg._unhandled_key_input(esc_out)
	_ok("and the capture it left armed is disarmed again", dlg._capturing_id, -1)

	print("\n=== B: Escape gets out of a capture WITHOUT assigning ===")
	## The chip's own tooltip promises "Esc cancels". `AcceptDialog` closes
	## itself on `ui_cancel` before `_unhandled_key_input` ever runs, so that
	## promise is only true if the dialog suspends that while a capture is
	## armed. Asserted as a state machine here, then as real routed input
	## below.
	var before := _live_accel(dlg, ID_UNDO)
	_ok("dialog closes on Escape while idle (the normal dialog behaviour)",
		dlg.dialog_close_on_escape, true)
	var chip: Button = dlg._capture_chips.get(ID_UNDO)
	_ok("Undo has a rebind chip", chip != null, true)
	chip.pressed.emit()
	await _frames(1)
	_ok("armed", dlg._capturing_id, ID_UNDO)
	_ok("the chip says so rather than showing a stale key",
		chip.text.findn("press a key") >= 0, true)
	_ok("Escape no longer closes the dialog while a chord is being captured",
		dlg.dialog_close_on_escape, false)
	## The control for the assertion three lines below. "The dialog is still
	## open after Escape" says nothing unless it was open before it -- and on
	## the version before `_set_capturing()` existed this pair reads
	## true / false, which is what makes the check discriminating.
	_ok("[control] the dialog is open BEFORE the Escape is pushed", dlg.visible, true)

	## Real routed input, not a direct `_unhandled_key_input()` call: pushed
	## into the SubViewport that owns the embedded dialog window, so the
	## event travels the same path a keyboard produces.
	var esc := InputEventKey.new()
	esc.pressed = true
	esc.keycode = KEY_ESCAPE
	_vp.push_input(esc)
	await _frames(2)
	_ok("the capture is cancelled", dlg._capturing_id, -1)
	_ok("the dialog is still open -- Escape cancelled the capture, not the dialog",
		dlg.visible, true)
	_ok("nothing was assigned", _live_accel(dlg, ID_UNDO), before)
	_ok("no override was written", DccSettings.has_shortcut_override(MENU, ID_UNDO), false)
	_ok("the chip is back to showing the live key",
		chip.text, OS.get_keycode_string(before))
	_ok("Escape closes the dialog again now the capture is over",
		dlg.dialog_close_on_escape, true)

	## The one exit `_set_capturing()` is NOT on: closing the dialog with its
	## own Close button while a chord is still being waited for. The flag is
	## left suspended by that path, so the next open has to be what clears it
	## -- or the reader gets a reference sheet Escape silently refuses to
	## close, with nothing on screen explaining why.
	chip.pressed.emit()
	await _frames(1)
	_ok("[control] armed again, so the flag really is suspended", dlg.dialog_close_on_escape, false)
	dlg.hide()
	await _frames(1)
	dlg.open_editable()
	await _frames(4)
	_ok("closing the dialog mid-capture cannot leave Escape disarmed",
		dlg.dialog_close_on_escape, true)
	_ok("... and no capture survived the close either", dlg._capturing_id, -1)

	print("\n=== C: a third row on one chord names BOTH of the others ===")
	## Blender's rule is "show the conflict", and with 18 rebindable actions a
	## chord can genuinely be shared by more than two. A message that names
	## only the first other row is a list silently truncated to one.
	## Reopened, not assumed: §B's Escape is allowed to have closed the dialog
	## on a build without `_set_capturing()`, and `_unhandled_key_input`
	## refuses to capture while hidden -- so without this every rebind below
	## would be swallowed and the assertions would report the SHIPPED keys as
	## though nothing had been asked of them. That is exactly how the first
	## run of this probe read.
	dlg.open_editable()
	await _frames(4)
	_ok("[control] reopened and editable again", dlg.visible and dlg._editable, true)

	const SHARED := KEY_MASK_CTRL | KEY_MASK_ALT | KEY_G
	var save_as_default: int = int(app.menus.shortcut_default(ID_SAVE_AS))
	_rebind_via_ui(dlg, ID_SAVE_AS, KEY_G, true, false, true)
	await _frames(2)
	_ok("first row actually took the chord", _live_accel(dlg, ID_SAVE_AS), SHARED)
	_ok("... and it is not what it shipped with, so this can fail",
		SHARED != save_as_default, true)
	_ok("no conflict to report yet", dlg._status.text.length(), 0)
	_rebind_via_ui(dlg, ID_CLOSE, KEY_G, true, false, true)
	await _frames(2)
	_ok("second row took it too", _live_accel(dlg, ID_CLOSE), SHARED)
	_ok("and the line names the one other row", dlg._status.text.findn("Save as") >= 0, true)
	_rebind_via_ui(dlg, ID_OPEN_PROJECT, KEY_G, true, false, true)
	await _frames(2)
	_ok("third row took it as well -- never blocked", _live_accel(dlg, ID_OPEN_PROJECT), SHARED)
	print("  [status] ", dlg._status.text)
	## The whole sentence as a literal, not two `findn()`s over it: the
	## counting word is part of the claim, and "both will fire" in front of a
	## list of two other rows is a message that contradicts its own list.
	## Asserting the substrings alone leaves that free to be wrong.
	_ok("the whole line, counting word included", dlg._status.text,
		"Also bound to \"Save as…\" and \"Close project\" -- all three will fire until one of them changes.")
	_ok("all three rows still answer to the chord -- none silently dropped",
		[_live_accel(dlg, ID_SAVE_AS), _live_accel(dlg, ID_CLOSE), _live_accel(dlg, ID_OPEN_PROJECT)],
		[SHARED, SHARED, SHARED])

	print("\n=== D: restore all, and leave the shared settings file clean ===")
	dlg._reset_all()
	await _frames(2)
	_ok("Save as... back to its shipped key",
		_live_accel(dlg, ID_SAVE_AS), int(app.menus.shortcut_default(ID_SAVE_AS)))
	_ok("Close back to its shipped key",
		_live_accel(dlg, ID_CLOSE), int(app.menus.shortcut_default(ID_CLOSE)))
	_ok("Open project back to its shipped key",
		_live_accel(dlg, ID_OPEN_PROJECT), int(app.menus.shortcut_default(ID_OPEN_PROJECT)))
	DccSettings.reset_shortcuts(MENU)
	_ok("the menu context holds no override at all",
		DccSettings.has_shortcut_override(MENU, ID_SAVE_AS)
			or DccSettings.has_shortcut_override(MENU, ID_CLOSE)
			or DccSettings.has_shortcut_override(MENU, ID_OPEN_PROJECT), false)
	_done()

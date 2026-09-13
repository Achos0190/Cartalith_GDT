extends Node
## Lane SEED — "a seed typed into New World is sometimes not the seed you
## get" (found on glass 2026-09-07, intermittent).
##
## MECHANISM, measured below rather than assumed: a `SpinBox` does not copy
## typed `LineEdit` text into `.value` as you type, and -- measured here --
## does NOT do so on a bare `text_submitted` signal emission either (a
## natural first guess, and wrong: emitting it, with matching text, on a
## freshly-built SpinBox leaves `.value` untouched). The one thing that DOES
## commit it, measured with the dialog actually popped and the field actually
## focused, is the `LineEdit` losing Godot's own keyboard focus -- and neither
## `hide()`ing the dialog nor a bare signal emission is that. Android's BACK
## dismissing the on-screen keyboard is consumed by the OS/IME layer and never
## reaches Godot at all, so it is neither: the field goes on SHOWING what was
## typed while `.value` -- and therefore `request()["seed"]`, the one thing
## `EngineBridge.generate()` ever reads -- stays whatever it was before. The
## pending edit then sits latent until focus happens to move for an unrelated
## reason, at which point it commits silently and surfaces on whichever
## `request()` call runs next -- the row's own "the typed value was then used
## by the next Generate World", and also `app.gd::_run_pipeline()` /
## `open_heightmap_import()`, both of which call `new_world_dialog.request()`
## with the dialog not even open.
##
## **2026-09-13 (this batch): the fix, and why real keystrokes replace a
## `.text` poke below.** The refuted 2026-09-12 attempt called
## `seed_input.apply()` unconditionally at the top of `request()`. It broke a
## ROLLED seed read while the dialog is hidden: `_vfy_seedroll_probe.gd`'s S0
## measured that while hidden, `LineEdit.text` does NOT resync to a `.value`
## written by code, so `apply()` re-parsed the stale display text straight
## over a freshly rolled `.value`. The shipped fix instead tracks a
## `_seed_dirty` flag on `NewWorldDialog`, raised by `LineEdit.text_changed`
## and cleared by every programmatic writer of `.value`
## (`randomise_seed()`) and by `request()` itself after committing --
## `request()` calls `apply()` only when the flag is set. **That flag's
## entire correctness rests on what actually fires `text_changed` in this
## Godot build**, so the SIGNAL DIAGNOSTIC section below pokes the
## PRODUCTION field (`dlg._seed_dirty`) directly after each kind of write,
## rather than trusting the class reference or a second, parallel listener.
## Because of that dependency, the PRIMARY case below now drives a real typed
## digit through `Input.parse_input_event()` (Godot's `Key` enum reuses ASCII
## for the printable range, so a digit's own unicode codepoint is its
## keycode) instead of the plain `le.text = "777777"` poke this probe used
## before the fix existed -- a poke reproduces the STATE Android BACK leaves
## behind, but not the SIGNAL the fix now depends on to have fired while it
## happened.
##
## Reproduced here on the desktop composition, no device needed. Windowed,
## not `--headless`: the focus-commit mechanism this probe measures did not
## reproduce under `--headless` with the dialog never popped (`grab_focus()`
## on a control that was never actually focused has nothing to steal focus
## FROM) -- popping the dialog needs a real window, and a real `InputEventKey`
## needs a real viewport to route it to whatever holds focus.
##
## Cell 4 (the phone roll path) needs `--force-touch` on the command line to
## count as measured for that cell -- see the gate at the bottom of `_ready()`.
## Every other cell here is `pressed.emit()`/a scripted key event, not a
## finger, and is reported as such.
##
## Run: godot4 --path . _seedcommit_probe.tscn
## Run (cell 4): godot4 --path . --force-touch _seedcommit_probe.tscn

var _app: Node
var _fail := 0

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("SEED %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

## Runs the real Create path and returns the seed the world was actually
## built with -- `bridge.world_gen.get_seed()`, the same getter `app.gd`'s own
## status line reads for "ELDRA · <seed>". Forces a 4x4 grid first so this is
## fast regardless of what the dialog's width/resolution controls hold.
func _create_and_get_seed(dlg: NewWorldDialog) -> int:
	dlg.grid_w_input.set_value_no_signal(4)
	dlg.grid_h_input.set_value_no_signal(4)
	dlg.width_input.set_value_no_signal(40.0)
	dlg._on_create()
	var waited := 0.0
	while _app.bridge.generating and waited < 60.0:
		await get_tree().create_timer(0.1).timeout
		waited += 0.1
	return _app.bridge.world_gen.get_seed()

## A real `InputEventKey` pair through `Input.parse_input_event()`, not a
## `.text =` poke -- so whatever signal a genuine keystroke fires (measured
## in SIGNAL DIAGNOSTIC below) fires here too. `flush_buffered_events()`
## matches `_vfy_seedroll_probe.gd`'s own proven Enter-key pattern.
func _send_key(code: int, unicode_val: int) -> void:
	var down := InputEventKey.new()
	down.keycode = code
	down.physical_keycode = code
	down.unicode = unicode_val
	down.pressed = true
	Input.parse_input_event(down)
	var up := down.duplicate() as InputEventKey
	up.pressed = false
	Input.parse_input_event(up)
	Input.flush_buffered_events()

func _send_enter() -> void:
	var down := InputEventKey.new()
	down.keycode = KEY_ENTER
	down.physical_keycode = KEY_ENTER
	down.pressed = true
	Input.parse_input_event(down)
	var up := down.duplicate() as InputEventKey
	up.pressed = false
	Input.parse_input_event(up)
	Input.flush_buffered_events()

## Leaves `le` holding exactly `digits`, using ONLY real input events end to
## end (caret placement via the property, which measurement showed holds;
## deletion and insertion via real key events, which measurement showed
## fires the fix's own signal) -- no `.text =` write anywhere in this
## function. `le` must already have focus. Three things measured directly
## here (this file's own first three runs, 2026-09-13) is why: a single
## `select_all()` up front did not replace the selection on the first
## keystroke, splicing typed digits into the OLD value's middle instead
## ("777777" over "66575" -> "5777773729"); `le.clear()` did not hold past
## the next awaited frame, which resynced the field back to the pre-clear
## baseline before the first keystroke landed ("38561" baseline + typed
## "777777" -> "73856177777"); and even a direct `.text = ` write of the
## target's first five characters, immediately followed by a caret
## assignment, was silently discarded the same way, only the caret landing
## where asked -- the trailing keystroke appended onto the UNCHANGED
## baseline instead of the intended prefix ("76668" baseline + typed
## "777777" -> "766687"). Real backspaces are not subject to whatever
## resync a plain `.text =` write is: this control's own displayed value
## only ever changes here through the same input path a real keystroke
## takes.
func _type_digits(le: LineEdit, digits: String) -> void:
	le.caret_column = le.text.length()
	await get_tree().process_frame
	for i in 12:  # generous: every baseline this probe writes is <= 6 digits.
		_send_key(KEY_BACKSPACE, 0)
		await get_tree().process_frame
	for i in digits.length():
		var code := digits.unicode_at(i)
		_send_key(code, code)
		await get_tree().process_frame

## Resets to a known, non-dirty baseline through the real production writer
## (`randomise_seed()`) rather than a probe-side poke, so every case starts
## from a state this dialog's own code actually produces -- and re-proves the
## clear on every use rather than asserting it once.
func _settle(dlg: NewWorldDialog) -> int:
	dlg.randomise_seed()
	await get_tree().process_frame
	return int(dlg.seed_input.value)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var force_touch := "--force-touch" in OS.get_cmdline_args()
	print("SEED run: force_touch=%s" % str(force_touch))

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	## Cosmetic only (confirmed: identical in a run that still passed 0/20
	## checks) -- `open_project_dialog` auto-shows at startup as an exclusive
	## child too, and popping this dialog on top of it without dismissing it
	## first logs "Attempting to make child window exclusive...". Matches
	## `_vfy_seedroll_probe.gd`'s own precedent for the same startup state.
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await get_tree().process_frame

	var dlg: NewWorldDialog = _app.new_world_dialog
	var le: LineEdit = dlg.seed_input.get_line_edit()

	# A real window is what the focus-commit mechanism needs; see header.
	dlg.popup_centered()
	await get_tree().process_frame
	await get_tree().process_frame

	# =====================================================================
	# SIGNAL DIAGNOSTIC -- Gate G1. Pokes the PRODUCTION `_seed_dirty` field
	# directly after each kind of write, rather than a second listener
	# guessing at the same signal. The whole fix rests on this.
	# =====================================================================
	await _settle(dlg)
	print("DIAG signal: after randomise_seed() [production writer]: _seed_dirty=%s (want false)" % dlg._seed_dirty)
	_check("SIGNAL: a programmatic .value write (randomise_seed) leaves _seed_dirty false",
		dlg._seed_dirty == false, "_seed_dirty=%s" % dlg._seed_dirty)

	dlg.seed_input.set_value_no_signal(int(dlg.seed_input.value) + 1)
	await get_tree().process_frame
	print("DIAG signal: after set_value_no_signal() [not a real seed writer, checked anyway]: _seed_dirty=%s" % dlg._seed_dirty)
	dlg._seed_dirty = false  # not a production writer -- don't let its answer leak into the next case.

	le.grab_focus()
	await get_tree().process_frame
	_send_key(53, 53)  # '5'
	await get_tree().process_frame
	print("DIAG signal: after ONE real keystroke while focused: _seed_dirty=%s le.text='%s' (want true -- the fix's load-bearing signal)" % [dlg._seed_dirty, le.text])
	_check("SIGNAL: a real keystroke while focused sets _seed_dirty true",
		dlg._seed_dirty == true, "_seed_dirty=%s le.text='%s'" % [dlg._seed_dirty, le.text])
	le.release_focus()

	# -- Diagnostics: name the commit trigger precisely (kept from the prior
	# batch -- still true, and unaffected by the fix: none of this calls
	# request() or checks _seed_dirty). ---------------------------------------
	le.grab_focus()
	dlg.seed_input.value = 12345
	le.text = "222222"
	print("DIAG pending edit while focused: value=%s text=%s" % [dlg.seed_input.value, le.text])
	le.text_submitted.emit("222222")
	print("DIAG after a BARE text_submitted emit (matching text): value=%s  -- signal alone does not commit" % dlg.seed_input.value)
	dlg.hide()
	print("DIAG after hide(): value=%s  -- hide() does not commit" % dlg.seed_input.value)
	dlg.popup_centered()
	await get_tree().process_frame
	dlg.width_input.get_line_edit().grab_focus()  # steals Godot's own key focus from `le`
	await get_tree().process_frame
	print("DIAG after focus moved to a DIFFERENT field: value=%s  -- THIS is what commits it" % dlg.seed_input.value)
	dlg._seed_dirty = false  # scripted pokes above may have set it; the matrix below starts clean.

	# -- Control 1: a rolled seed (randomise_seed(), a plain property write),
	# no LineEdit text ever pending, still reaches the built world. -----------
	dlg.randomise_seed()
	await get_tree().process_frame
	var rolled := int(dlg.seed_input.value)
	var built_rolled: int = await _create_and_get_seed(dlg)
	_check("cell 3a: a rolled seed (dialog open) still reaches the built world",
		built_rolled == rolled, "rolled=%d built=%d" % [rolled, built_rolled])

	# -- Control 2: apply() is a no-op on an already-settled value (kept). ----
	dlg.seed_input.value = 33333
	await get_tree().process_frame
	dlg.seed_input.apply()
	_check("control: apply() is a no-op on an already-settled value",
		dlg.seed_input.value == 33333.0,
		"value=%s after apply() with nothing pending" % dlg.seed_input.value)
	dlg._seed_dirty = false
	var built_settled: int = await _create_and_get_seed(dlg)
	_check("control: a normally-committed value still reaches the built world",
		built_settled == 33333, "built=%d" % built_settled)

	# =====================================================================
	# PROOF MATRIX
	# =====================================================================

	# -- Cell 1 (PRIMARY): typed, not committed (real keystrokes, no Enter, no
	# focus change) -> Create -> built == typed. This is what an Android
	# BACK-then-tap-Create leaves behind -- see header. Mutate away the fix
	# and this FAILS: typed 777777, built stays whatever `.value` was before.
	await _settle(dlg)
	le.grab_focus()
	await _type_digits(le, "777777")
	print("DIAG cell1 before Create: value=%s text='%s' _seed_dirty=%s" % [dlg.seed_input.value, le.text, dlg._seed_dirty])
	_check("cell 1 pre-check: real typing set _seed_dirty true (no Enter, no focus change yet)",
		dlg._seed_dirty == true, "_seed_dirty=%s" % dlg._seed_dirty)
	var built_pending: int = await _create_and_get_seed(dlg)
	_check("cell 1 PRIMARY: an uncommitted typed seed (real keystrokes) is what the world is built with",
		built_pending == 777777,
		"typed 777777 (uncommitted) -> built %d, _seed_dirty_after=%s" % [built_pending, dlg._seed_dirty])
	_check("cell 1 post-check: request() cleared _seed_dirty after committing",
		dlg._seed_dirty == false, "_seed_dirty=%s" % dlg._seed_dirty)

	# -- Cell 2: typed + a REAL Enter key event -> Create -> built == typed.
	await _settle(dlg)
	le.grab_focus()
	await _type_digits(le, "434343")
	_send_enter()
	await get_tree().process_frame
	print("DIAG cell2 after real Enter: value=%s _seed_dirty=%s" % [dlg.seed_input.value, dlg._seed_dirty])
	var built_enter: int = await _create_and_get_seed(dlg)
	_check("cell 2: typed + real Enter (Godot's own commit) builds the typed seed",
		built_enter == 434343, "built=%d" % built_enter)

	# -- Cell 3b (the refuted case): rolled while the dialog is HIDDEN ->
	# request() via app._run_pipeline() -> built == rolled. `_run_pipeline()`
	# is `app.gd`'s own second caller of request() (`_on_create()` is the
	# first); this is the exact path the 2026-09-12 fix broke.
	dlg.hide()
	await get_tree().process_frame
	var rolled3b := await _settle(dlg)
	await get_tree().create_timer(0.3).timeout
	dlg.grid_w_input.set_value_no_signal(4)
	dlg.grid_h_input.set_value_no_signal(4)
	dlg.width_input.set_value_no_signal(40.0)
	_app._run_pipeline()
	var waited3b := 0.0
	while _app.bridge.generating and waited3b < 60.0:
		await get_tree().create_timer(0.1).timeout
		waited3b += 0.1
	var built3b: int = _app.bridge.world_gen.get_seed()
	_check("cell 3b REFUTED-CASE: rolled while HIDDEN, request() via app._run_pipeline() builds the roll",
		built3b == rolled3b, "rolled=%d built=%d dlg.visible=%s" % [rolled3b, built3b, str(dlg.visible)])

	# -- Cell 5a: typed-uncommitted, THEN rolled -> built == rolled (the roll,
	# being the LATER write, is what request() should see as current).
	dlg.popup_centered()
	await get_tree().process_frame
	await get_tree().process_frame
	await _settle(dlg)
	le.grab_focus()
	await _type_digits(le, "515151")
	dlg.randomise_seed()
	await get_tree().process_frame
	var rolled5a := int(dlg.seed_input.value)
	var built5a: int = await _create_and_get_seed(dlg)
	_check("cell 5a: typed-uncommitted THEN rolled -> the roll wins",
		built5a == rolled5a, "rolled=%d built=%d" % [rolled5a, built5a])

	# -- Cell 5b: rolled, THEN typed-uncommitted -> built == typed (the typed
	# edit, being the LATER write, is what request() should see as current).
	await _settle(dlg)
	dlg.randomise_seed()
	await get_tree().process_frame
	le.grab_focus()
	await _type_digits(le, "626262")
	print("DIAG cell5b before Create: _seed_dirty=%s" % dlg._seed_dirty)
	var built5b: int = await _create_and_get_seed(dlg)
	_check("cell 5b: rolled THEN typed-uncommitted -> the typed value wins",
		built5b == 626262, "built=%d" % built5b)

	# -- Cell 4: the phone roll path, gated on --force-touch. Reported as
	# skipped (not a pass) when the flag is absent -- `pressed.emit()` and a
	# scripted `--force-touch` run are both still not a finger; that is
	# disclosed in the check name and in the run banner above, not implied.
	if force_touch:
		var ws4 = _app._world_workspace()
		var host4 := VBoxContainer.new()
		add_child(host4)
		ws4.build_phone_generate(host4)
		await get_tree().process_frame
		dlg.hide()
		await get_tree().process_frame
		var base4 := await _settle(dlg)
		ws4._pg_rebuild()
		await get_tree().process_frame
		ws4._pg_roll_seed()
		var rolled4 := int(dlg.seed_input.value)
		await get_tree().process_frame  # "survives the frame"
		var survived4 := int(dlg.seed_input.value) == rolled4 and rolled4 != base4
		var built4: int = await _create_and_get_seed(dlg)
		_check("cell 4 [--force-touch]: phone roll path survives the frame and builds",
			survived4 and built4 == rolled4,
			"force_touch=true base=%d rolled=%d survived=%s built=%d" % [base4, rolled4, str(survived4), built4])
	else:
		print("SEED cell 4 SKIPPED this run -- no --force-touch (re-run with the flag)")

	print("SEED RESULT: %d failed  (force_touch=%s)" % [_fail, str(force_touch)])
	get_tree().quit(1 if _fail > 0 else 0)

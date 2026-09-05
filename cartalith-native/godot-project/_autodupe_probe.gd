extends Node
## The autosave state is drawn ONCE, and the interval the bar prints is the one
## the clock is actually armed to.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _autodupe_probe.tscn
##
## Two things, both read back off the drawn nodes of a booted shell:
##
##   A. `design/proposed-2026-09-05/StatusBar.dc.html` state 3 draws the autosave
##      state in ONE cell. Until this change `app.gd` also appended it to the
##      `mid` composite, so the moment `DccShell._build_status_bar()` started
##      drawing the cell the same sentence stood twice in one bar. Check 3 walks
##      every Label in `status_row` and counts the ones that mention it -- and
##      check 4 stages a real duplicate first, so the counter is known to be able
##      to report 2 rather than trivially reporting 1.
##   B. The interval. `dcc_shell.gd` says the artboard's `every 4 min` is "real
##      and not 4". It is also not any ONE number: it is settable off
##      `File ▸ Autosave interval`. So the probe drives every rung of that ladder
##      through the shell's own `apply_autosave_setting()` and asserts the armed
##      `Timer.wait_time` AND the drawn text follow it each time.
##
## **No check here is palette-bound** -- every assertion is a string, a node
## index or a float off a `Timer`, so nothing changes value between light and
## dark. The palette is printed rather than forced, because forcing one would be
## the only palette-dependent act in the file.
##
## **Ladder values are the artboard's / the menu's own literals**, carried here
## rather than read out of `DccMenus.AUTOSAVE_MINUTES` and compared to itself.
##
## **The settings file is restored byte-for-byte in `_restore_cfg()`**, which
## runs on every exit path: this probe writes `user://cartalith_settings.cfg`
## through the real setter, and that file is the user's own state.

const LADDER := [1, 5, 15]      ## `DccMenus.AUTOSAVE_MINUTES`, off §2.1
const ARTBOARD_MINUTES := 4     ## `autosave every 4 min`, the artboard's figure
const DESIGN_DEFAULT := 5       ## `03-menu-bar.md` §6.1 row 8, "default 5 min"
const SEC := 60.0               ## minutes -> `Timer.wait_time`

var app: Node
var _fail := 0
var _cfg_before := ""
var _cfg_existed := false

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("AD %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _save_cfg() -> void:
	var f := FileAccess.open(DccSettings.CONFIG_PATH, FileAccess.READ)
	if f != null:
		_cfg_existed = true
		_cfg_before = f.get_as_text()
		f.close()

func _restore_cfg() -> void:
	if _cfg_existed:
		var f := FileAccess.open(DccSettings.CONFIG_PATH, FileAccess.WRITE)
		if f != null:
			f.store_string(_cfg_before)
			f.close()
	else:
		DirAccess.remove_absolute(ProjectSettings.globalize_path(DccSettings.CONFIG_PATH))

func _done(code: int) -> void:
	_restore_cfg()
	get_tree().quit(code)

# -- Reading the bar back off the tree ----------------------------------------

func _cell(slot: String) -> Control:
	for c in app.status_row.get_children():
		if c.name == "Slot_" + slot:
			return c
	return null

func _labels_under(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append(c)
		_labels_under(c, out)

## Every Label anywhere in the status row, drawn or not, with its text.
func _row_texts() -> Array:
	var found: Array = []
	_labels_under(app.status_row, found)
	var out: Array = []
	for l in found:
		out.append((l as Label).text)
	return out

## How many of the row's labels carry the autosave STATE. Matched on the word
## `autosave`, which no other slot uses -- deliberately not on `saved`, because
## `unsaved changes` is this same cell's off-and-dirty text and `saved world.zip`
## is the `hint` slot after File ▸ Save.
func _autosave_mentions() -> int:
	var n := 0
	for t in _row_texts():
		if String(t).contains("autosave"):
			n += 1
	return n

func _slot_order() -> Array:
	var out: Array = []
	for c in app.status_row.get_children():
		if String(c.name).begins_with("Slot_"):
			out.append(String(c.name).substr(5))
	return out

func _ready() -> void:
	_save_cfg()
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	## The shell has to have actually booted -- `app.tscn` still instantiates
	## when a script in the chain will not compile, and every check below would
	## then be measuring a bare Node while reading green.
	if app.get_script() == null or not app.has_method("_refresh_save_status"):
		print("AD  !! app.gd did not compile -- nothing below would be measuring the app")
		_done(1)
		return
	print("AD NOTE palette dark=%s (no check below is palette-bound)" % DccTheme.is_dark())

	# -- 1. The row's slot order, and the move_child(_stale_recompute, 2) contract
	_check("slot order is the artboard's, read off the node names",
		_slot_order() == ["pass", "stale", "atlas", "progress", "autosave"],
		str(_slot_order()))
	_check("`Recompute` still sits at index 2, immediately after `Slot_stale`",
		app.status_row.get_child(0).name == "Slot_pass"
			and app.status_row.get_child(1).name == "Slot_stale"
			and app._stale_recompute.get_index() == 2,
		"children 0..3 = %s, %s, %s, %s" % [
			app.status_row.get_child(0).name, app.status_row.get_child(1).name,
			app.status_row.get_child(2).name, app.status_row.get_child(3).name])

	# -- 2. A world, so the composite has a stage to name --------------------
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(6)

	## The state the duplicate showed in: autosave on, a project path, a world.
	DccSettings.set_autosave_enabled(true)
	DccSettings.set_autosave_minutes(DESIGN_DEFAULT)
	## An absolute OS path, not `user://`: the writer is the Rust side
	## (`project_save_with_documents`), which takes a filesystem path and cannot
	## resolve a Godot scheme -- measured, the first run of this probe wrote
	## `autosave failed` against a `user://` target.
	app.current_project_path = ProjectSettings.globalize_path("user://_autodupe_target.zip")
	app.apply_autosave_setting()
	await _frames(2)
	print("AD NOTE autosave slot = [%s]" % app.status_slot_text("autosave"))
	print("AD NOTE mid slot      = [%s]" % app.status_slot_text("mid"))

	# -- 3. Drawn exactly once ------------------------------------------------
	_check("the autosave state appears in exactly one label of the whole row",
		_autosave_mentions() == 1,
		"%d mention(s) in %s" % [_autosave_mentions(), str(_row_texts())])
	_check("the one that has it is the `autosave` cell, and it is drawn",
		app.status_slot_text("autosave").contains("autosave")
			and _cell("autosave") != null and _cell("autosave").is_visible_in_tree(),
		"[%s]" % app.status_slot_text("autosave"))
	_check("the `mid` composite carries no autosave field of its own",
		not app.status_slot_text("mid").contains("autosave"),
		"[%s]" % app.status_slot_text("mid"))

	# -- 4. Positive control: the counter can report a duplicate --------------
	#
	# Without this the check above passes for a probe that simply cannot count
	# to 2. `mid` is written directly with the exact text the removed line used
	# to append, then put back.
	var mid_real: String = app.status_slot_text("mid")
	app.set_status("mid", "%s · autosave every %d min" % [mid_real, DESIGN_DEFAULT],
		"text_ghost")
	await _frames(2)
	_check("POSITIVE CONTROL: staging the removed line back makes the count 2",
		_autosave_mentions() == 2, "%d mention(s)" % _autosave_mentions())
	app.set_status("mid", mid_real, "text_ghost")
	await _frames(2)
	_check("...and restoring `mid` brings it back to 1",
		_autosave_mentions() == 1, "%d mention(s)" % _autosave_mentions())

	# -- 5. The interval: measured off the armed Timer, at every rung ---------
	#
	# `4` is asserted absent rather than assumed absent: it is the artboard's
	# own figure and the whole reason `dcc_shell.gd` says "real, and not 4".
	## The carried literal pinned to the shipping table, in that direction: a
	## ladder check that read `DccMenus.AUTOSAVE_MINUTES` on both sides would hold
	## for every value of it.
	_check("the carried ladder is the menu's own",
		LADDER == Array(DccMenus.AUTOSAVE_MINUTES),
		"probe=%s  DccMenus=%s" % [str(LADDER), str(DccMenus.AUTOSAVE_MINUTES)])
	_check("the artboard's `4` is not a settable interval",
		not LADDER.has(ARTBOARD_MINUTES), str(LADDER))
	for m in LADDER:
		DccSettings.set_autosave_minutes(m)
		app.apply_autosave_setting()
		await _frames(2)
		var armed: float = app._autosave_timer.wait_time
		var drawn: String = app.status_slot_text("autosave")
		_check("interval %d min: the clock is armed to %d s and the bar says so"
				% [m, m * int(SEC)],
			is_equal_approx(armed, float(m) * SEC)
				and drawn == "autosave every %d min" % m,
			"wait_time=%.1f s  slot=[%s]" % [armed, drawn])

	# -- 6. A real autosave write, and what the slot says afterwards ----------
	DccSettings.set_autosave_minutes(DESIGN_DEFAULT)
	app.apply_autosave_setting()
	app.bridge.mark_world_dirty()
	await _frames(2)
	var before_text: String = app.status_slot_text("autosave")
	app._autosave_tick()
	await _frames(2)
	var target := ProjectSettings.globalize_path("user://_autodupe_target.autosave.zip")
	var wrote := FileAccess.file_exists(target)
	var after_text: String = app.status_slot_text("autosave")
	_check("a real `_autosave_tick()` wrote the archive beside the project",
		wrote, "%s exists=%s" % [target, wrote])
	_check("after the write the slot carries BOTH halves the artboard draws",
		after_text.begins_with("autosave every %d min · saved " % DESIGN_DEFAULT)
			and after_text.length() == ("autosave every %d min · saved " % DESIGN_DEFAULT).length() + 5,
		"before=[%s] after=[%s]" % [before_text, after_text])
	## The regression the composer closes: `dirty_changed` runs
	## `_refresh_save_status()`, which used to overwrite the clock with the bare
	## interval, so the two halves could never be on screen together.
	app._refresh_save_status()
	await _frames(2)
	_check("the clock survives the next `dirty_changed` refresh",
		app.status_slot_text("autosave") == after_text,
		"[%s]" % app.status_slot_text("autosave"))
	_check("still exactly one autosave label after the write",
		_autosave_mentions() == 1,
		"%d mention(s) in %s" % [_autosave_mentions(), str(_row_texts())])
	if wrote:
		DirAccess.remove_absolute(target)

	print("AD %d failed" % _fail)
	_done(1 if _fail > 0 else 0)

extends Node
## Protects: GF-8 slice 2, the shell's **Rock** paint target (`GEOLOGY_FIRST_SCOPE.md`
## §4.13, owner Ruling BJ), end to end through the real shell and the real engine:
##
##   Leg A -- an **app world** (the app's defaults: geology processes off). The Rock
##   target is drawn and DISABLED with the engine's reason and the save caveat
##   ("not saved with the project yet"); `paint_set_layer("rock_top")` and the
##   shell's own `select_paint_layer` refuse it; the field and the column do not
##   move (digest before == after).
##
##   Leg B -- the same seed regenerated with `geology_processes` on (`set_params`).
##   The Rock target is enabled; a radius-6 disc of a different rock (shale, or
##   granite where the centre is shale) is painted through the workspace's own
##   press/release handlers and committed through its Commit handler. Checked:
##   the engine's re-erosion window `W` is the predicted disc box + 16 cells; the
##   field AND the column change inside `W` and NOT outside it (FNV digests from
##   `WorldGen.geology_digest`); Undo (`app.undo_last`, the Ctrl+Z path) restores
##   both arrays byte for byte; Redo (`app.redo_last`) reapplies both byte for
##   byte. Positive control: the inside digests MUST move on commit (a digest that
##   cannot see the change would pass the outside checks vacuously).
##
## Navigation is by call where the probe needs a surface (`app.arm_tool`,
## `_pick_phone_tab`, `_pg_set_mode`): this probe proves the Rock target renders,
## is disabled or enabled with the right reason, and that its commit / undo / redo
## are exact. It is NOT a reachability proof of the phone path (`MISTAKES.md`).
##
## MUST run WINDOWED from a scratch copy whose `target/debug/cartalith_godot.dll`
## was built from this change (`--headless` never fires `frame_post_draw`):
##   desktop: Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x900 _gf8rock_probe.tscn -- --out DIR
##   phone:   ... --resolution 393x852  _gf8rock_probe.tscn -- --force-touch --out DIR
##   tablet:  ... --resolution 1600x2560 _gf8rock_probe.tscn -- --force-touch --out DIR
## `--force-touch` is `dcc_shell.gd`'s own override and is accepted here unread;
## `--out DIR` (default `user://gf8rock`) names where the frames go. Any other
## argument aborts (exit 2).
##
## Result: an assertion. Prints `[FORM] <form>`, `[PASS|FAIL] <check>` rows,
## `[INFO] <text>`, `[OUT] <path>` per frame and `[DONE] <n> checks, <m> failed`.
## Exit 0 every check passed; 1 a check failed; 2 could not run (unknown argument,
## no extension, no world); 3 watchdog.

var _fail := 0
var _checks := 0
var _out := "user://gf8rock"
var _form := "desktop"

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(ok: bool, what: String) -> void:
	_checks += 1
	if not ok:
		_fail += 1
	print("[%s] %s" % ["PASS" if ok else "FAIL", what])

## The whole window, saved as `<form>_<name>.png`.
func _shot(name: String) -> void:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	var path := "%s/%s_%s.png" % [_out, _form, name]
	img.save_png(path)
	print("[OUT] %s" % ProjectSettings.globalize_path(path))

## Every visible-in-tree `Button` under `n` whose text is `text` (internal children included).
func _buttons(n: Node, text: String, out: Array) -> void:
	if n is Button and (n as Button).text == text and (n as Button).is_visible_in_tree():
		out.append(n)
	for c in n.get_children(true):
		_buttons(c, text, out)

## Every visible `Label` under `n` whose text contains `needle`.
func _labels_with(n: Node, needle: String, out: Array) -> void:
	if n is Label and needle in (n as Label).text and (n as Label).is_visible_in_tree():
		out.append(n)
	for c in n.get_children(true):
		_labels_with(c, needle, out)

func _generate(bridge, seed: int) -> bool:
	bridge.generate({"seed": seed, "width_km": 800.0, "grid_w": 512, "grid_h": 328})
	var waited := 0
	while bridge.generating and waited < 3600:
		await _frames(1)
		waited += 1
	await _frames(12)
	return bool(bridge.has_world)

## Puts the Paint surfaces on screen for this form: arms Paint (desktop/tablet tool
## bar), and on a phone opens the GENERATE sheet's PAINT column.
func _show_paint(app, ws) -> void:
	app.arm_tool("paint")
	await _frames(6)
	if _form == "phone":
		if app.has_method("_pick_phone_tab"):
			app._pick_phone_tab("gen")
		await _frames(6)
		ws._pg_set_mode("paint")
		await _frames(8)

## Phone only: opens the sheet to its full detent and scrolls the first visible
## label containing `needle` into view, so a frame shows the Rock rows. A no-op on
## the other forms (their dock and tool bar are always on screen).
func _phone_reveal(app, needle: String) -> void:
	if _form != "phone":
		return
	app.set_phone_detent("full")
	await _frames(40)   # the detent tween (0.28 s, MISTAKES "await the tween")
	var hits: Array = []
	_labels_with(app, needle, hits)
	if hits.is_empty():
		print("[INFO] phone: no label containing '%s' to scroll to" % needle)
		return
	var sc: Node = (hits[0] as Node).get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(hits[0])
	await _frames(6)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("[WATCHDOG] timed out"); get_tree().quit(3))
	wd.start()

	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--out" and i + 1 < args.size():
			_out = args[i + 1]
			i += 2
		elif args[i] == "--force-touch":
			i += 1
		else:
			print("[FATAL] unknown argument %s (read: --out DIR, --force-touch)" % args[i]); get_tree().quit(2); return
	DirAccess.make_dir_recursive_absolute(_out)

	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(2); return
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(40)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(10)
	_form = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("[FORM] %s (window %s)" % [_form, get_viewport().get_visible_rect().size])
	app.call("_on_rail_node_pressed", "world", "a")
	await _frames(10)
	var bridge = app.get("bridge")
	var wg = bridge.world_gen
	if not wg.has_method("rock_paint_status") or not wg.has_method("geology_digest"):
		print("[FATAL] the loaded extension predates GF-8 slice 2"); get_tree().quit(2); return
	var ws = app._workspace_panels.get("world")

	# ---------------- Leg A: an app world, processes off ----------------
	if not await _generate(bridge, 483920):
		print("[FATAL] no world after generate"); get_tree().quit(2); return
	var p: Dictionary = wg.get_params()
	_check(p.get("geology_processes") == false, "leg A runs the app default: geology_processes off (read %s)" % p.get("geology_processes"))
	var st: Dictionary = bridge.rock_paint_status()
	_check(not bool(st.get("available", true)), "leg A: Rock is not available")
	_check("geology processes are off" in String(st.get("reason", "")), "leg A: the reason is the engine's processes-off sentence: %s" % st.get("reason", "<none>"))
	_check(not st.has("painted_cells"), "leg A: no painted mask (key absent, not 0)")
	print("[INFO] geology_available() = %s (the app keeps the inert GF-1 column)" % bridge.geology_available())
	var whole: Dictionary = wg.geology_digest(0, 0, 0, 0)
	_check(not whole.is_empty(), "leg A: the digest answers (the app world carries a column)")
	_check(not bool(wg.paint_set_layer("rock_top")), "leg A: the engine refuses a Rock layer")
	await _show_paint(app, ws)
	_check(not bool(ws.select_paint_layer("rock_top")), "leg A: the shell's select_paint_layer refuses Rock")
	_check(not String(ws._paint_layer).begins_with("rock_"), "leg A: the shell stays off Rock (layer %s)" % ws._paint_layer)
	await _frames(6)
	var rock_buttons: Array = []
	_buttons(app, "Rock", rock_buttons)
	_check(rock_buttons.size() > 0, "leg A: a Rock target is drawn (%d found)" % rock_buttons.size())
	var all_disabled := rock_buttons.size() > 0
	var reason_shown := false
	for b in rock_buttons:
		all_disabled = all_disabled and (b as Button).disabled
		if "not saved with the project yet" in (b as Button).tooltip_text and "geology processes are off" in (b as Button).tooltip_text:
			reason_shown = true
	var why_labels: Array = []
	_labels_with(app, "Rock unavailable", why_labels)
	_check(all_disabled, "leg A: every drawn Rock target is disabled")
	_check(reason_shown or why_labels.size() > 0, "leg A: the reason is on the control (tooltip with the save caveat) or written beside it (%d labels)" % why_labels.size())
	var whole2: Dictionary = wg.geology_digest(0, 0, 0, 0)
	_check(whole2 == whole, "leg A: nothing changed (field and column digests identical)")
	await _shot("A_app_rock_disabled")

	# ---------------- Leg B: processes on ----------------
	var rep: Dictionary = wg.set_params({"geology_processes": true})
	_check((rep.get("rejected", PackedStringArray()) as PackedStringArray).is_empty(), "set_params took geology_processes: %s" % [rep])
	if not await _generate(bridge, 483920):
		print("[FATAL] no world after the processes-on generate"); get_tree().quit(2); return
	st = bridge.rock_paint_status()
	_check(bool(st.get("available", false)), "leg B: Rock is available (reason %s)" % st.get("reason", "<none>"))
	_check(bridge.geology_available(), "leg B: geology_available() is true")
	# The land cell nearest the centre on an 8-cell lattice, 40 cells from every edge.
	var best := Vector2i(-1, -1)
	var best_d := 1 << 30
	for y in range(40, 328 - 40, 8):
		for x in range(40, 512 - 40, 8):
			var s: Dictionary = wg.sample_cell(x, y)
			if String(s.get("water", "")) == "land" and float(s.get("elevation_m", 0.0)) > 150.0:
				var d := (x - 256) * (x - 256) + (y - 164) * (y - 164)
				if d < best_d:
					best_d = d
					best = Vector2i(x, y)
	if best.x < 0:
		print("[FATAL] no interior land on this world"); get_tree().quit(2); return
	var r := 6
	var win := Rect2i(best.x - r - 16, best.y - r - 16, 2 * r + 1 + 32, 2 * r + 1 + 32)
	var centre_rock := int(wg.geology_digest(best.x, best.y, 1, 1).get("rock_top_centre", -1))
	var value := 1 if centre_rock == 9 else 10   # granite onto shale, else shale
	print("[INFO] painting value %d at %s (centre rock index %d), predicted W %s" % [value, best, centre_rock, win])

	await _show_paint(app, ws)
	# Select Rock through the drawn control's own handler where one is drawn and enabled.
	rock_buttons.clear()
	_buttons(app, "Rock", rock_buttons)
	var enabled_rock: Button = null
	for b in rock_buttons:
		if not (b as Button).disabled:
			enabled_rock = b
	_check(enabled_rock != null, "leg B: an enabled Rock target is drawn")
	if enabled_rock != null:
		enabled_rock.pressed.emit()
	else:
		ws.select_paint_layer("rock_top")
	await _frames(6)
	_check(String(ws._paint_layer) == "rock_top", "leg B: Rock opens on Surface (layer %s)" % ws._paint_layer)
	ws._paint_brush["value"] = value
	ws._paint_brush["radius"] = float(r)
	ws._paint_brush["land_only"] = false
	ws._paint_brush["erase"] = true   # Erase latched: Rock must still paint rock (§4.13)
	ws._sync_paint_brush()
	await _frames(4)
	if _form == "phone":
		ws._pg_rebuild()
		await _frames(6)
	var erase_buttons: Array = []
	_buttons(app, "Erase", erase_buttons)
	_buttons(app, "ERASE", erase_buttons)
	var erase_off := erase_buttons.size() > 0
	for b in erase_buttons:
		erase_off = erase_off and (b as Button).disabled
	_check(erase_off, "leg B: every drawn Erase control is disabled on Rock (%d found)" % erase_buttons.size())
	await _phone_reveal(app, "ROCK LAYER")
	await _shot("B_rock_options")

	var before: Dictionary = wg.geology_digest(win.position.x, win.position.y, win.size.x, win.size.y)
	ws._paint_click(float(best.x), float(best.y))
	ws._paint_release(float(best.x), float(best.y), true)
	await _frames(4)
	_check(bridge.paint_draft_count() > 0, "leg B: the Rock dab is pending (%d)" % bridge.paint_draft_count())
	_check(wg.geology_digest(win.position.x, win.position.y, win.size.x, win.size.y) == before, "leg B: a draft changes neither field nor column")
	await _shot("B_rock_draft")

	# Commit: on a phone through the Paint column's own COMMIT chip (its handler
	# rebuilds the column, which is what draws the LAST COMMIT note); elsewhere
	# through the dock panel's Commit handler, which every desktop Commit shares.
	var phone_commit: Button = null
	if _form == "phone":
		var cs: Array = []
		_buttons(app, "%s COMMIT" % DccIcons.SYMBOLS["tick"], cs)
		if cs.size() > 0:
			phone_commit = cs[0]
	if phone_commit != null:
		phone_commit.pressed.emit()
	else:
		ws._on_paint_commit()
	await _frames(8)
	var rc: Dictionary = ws.rock_last_commit
	_check(bool(rc.get("ok", false)), "leg B: the Rock commit succeeded: %s" % rc.get("reason", ""))
	var e: Dictionary = rc.get("reerode", {})
	_check(bool(e.get("ran", false)), "leg B: the window re-eroded (%s)" % e.get("reason", "ran"))
	var w: Dictionary = e.get("window", {})
	_check(Rect2i(int(w.get("x", -1)), int(w.get("y", -1)), int(w.get("w", -1)), int(w.get("h", -1))) == win,
		"leg B: W is the disc box + 16 cells: engine %s, predicted %s" % [w, win])
	_check(int(rc.get("cells_baked", 0)) > 0 and int(rc.get("cells_changed", 0)) > 0, "leg B: rock baked (%d) and changed (%d)" % [int(rc.get("cells_baked", 0)), int(rc.get("cells_changed", 0))])
	print("[INFO] commit: cells_moved %s, ring_step_pct %s, ms %.1f, not_rerun %s" % [e.get("cells_moved"), e.get("ring_step_pct", "-"), float(rc.get("ms", -1.0)), e.get("not_rerun")])
	var after: Dictionary = wg.geology_digest(win.position.x, win.position.y, win.size.x, win.size.y)
	_check(after["field_in"] != before["field_in"], "positive control: the field inside W moved")
	_check(after["column_in"] != before["column_in"], "positive control: the column inside W moved")
	_check(after["field_out"] == before["field_out"], "locality: the field outside W did not move")
	_check(after["column_out"] == before["column_out"], "locality: the column outside W did not move")
	_check(before["painted_mask"] == "absent" and after["painted_mask"] == "present",
		"the painted mask goes from absent to present (%s -> %s)" % [before["painted_mask"], after["painted_mask"]])
	_check(bridge.paint_draft_count() == 0, "leg B: the commit emptied the draft")
	# The report is on the status line (desktop, tablet) or, on a phone, which has no
	# status bar on screen, in the Paint column's LAST COMMIT note.
	var status_labels: Array = []
	_labels_with(app, "LAST COMMIT · rock painted:" if _form == "phone" else "rock painted:", status_labels)
	_check(status_labels.size() > 0, "leg B: the commit report is on screen (%d labels)" % status_labels.size())
	await _phone_reveal(app, "LAST COMMIT")
	await _shot("B_rock_committed")

	app.undo_last()
	await _frames(8)
	var undone: Dictionary = wg.geology_digest(win.position.x, win.position.y, win.size.x, win.size.y)
	_check(undone == before, "undo restores the field AND the column byte for byte (in and out of W)")
	app.redo_last()
	await _frames(8)
	var redone: Dictionary = wg.geology_digest(win.position.x, win.position.y, win.size.x, win.size.y)
	_check(redone == after, "redo reapplies the field AND the column byte for byte")
	st = bridge.rock_paint_status()
	_check(int(st.get("painted_cells", 0)) > 0, "after redo the column marks %d cells painted" % int(st.get("painted_cells", 0)))
	var inv: PackedStringArray = ws._authored_inventory()
	var inv_rock := false
	for s2 in inv:
		if "painted rock" in String(s2):
			inv_rock = true
	_check(inv_rock, "the regenerate prompt's inventory lists the painted rock: %s" % [inv])
	await _shot("B_after_redo")

	print("[DONE] %d checks, %d failed" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

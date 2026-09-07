extends Node
## Lane DRAWER+PREFS, 2026-09-07. Owner: "in the preferences menu we should show
## what options have been selected." This drives the real Preferences popup's
## own `about_to_popup` and reports, per value group, whether the row now
## carries its selection -- and, for the groups that do not, why.
##
##   godot --headless --path . _prefvalue_probe.tscn
##
## Reads no flags; anything after `--` aborts.
##
## Headless is sound: nothing here reads a pixel. It reads `PopupMenu` item text
## and check state, which exist without a framebuffer.

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[prefval] %s" % s)

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		if String(a).begins_with("--"):
			_log("ABORT unknown flag %s -- this probe reads none" % a)
			get_tree().quit(2)
			return
	var vp := SubViewport.new()
	vp.size = Vector2i(1920, 1080)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	app = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	var prefs: PopupMenu = null
	for mb in app.find_children("", "MenuButton", true, false):
		if String((mb as MenuButton).text) == "Preferences":
			prefs = (mb as MenuButton).get_popup()
	if prefs == null:
		_log("ABORT no Preferences MenuButton found")
		get_tree().quit(2)
		return

	## The rows BEFORE the handler runs, so the stamp is a measured delta and
	## not a claim about a string that might always have said that.
	var before := {}
	for i in prefs.item_count:
		var s := prefs.get_item_submenu(i)
		if s != "":
			before[s] = prefs.get_item_text(i)

	prefs.about_to_popup.emit()
	await _frames(2)

	_log("=== Preferences value groups after about_to_popup ===")
	var stamped := 0
	var blank: Array[String] = []
	for i in prefs.item_count:
		var s := prefs.get_item_submenu(i)
		if s == "":
			continue
		var sub := prefs.get_node_or_null(NodePath(s))
		var checks := 0
		var n_items := 0
		if sub is PopupMenu:
			var pm := sub as PopupMenu
			n_items = pm.item_count
			for j in pm.item_count:
				if not pm.is_item_separator(j) and pm.is_item_checked(j):
					checks += 1
		var now := prefs.get_item_text(i)
		var moved: bool = now != String(before[s])
		if moved:
			stamped += 1
		else:
			blank.append(s)
		_log("  %-24s items=%-3d checked=%-2d %s  %s" % [
			s, n_items, checks, "STAMPED" if moved else "blank  ", now])
	_log("  %d of %d value groups carry their selection" % [
		stamped, before.size()])

	## The two halves of the contract, and the second is the one that matters:
	## a group with no check must show NOTHING, never a plausible-looking value.
	_check(stamped > 0, "positive control: at least one group stamped")
	var lied := 0
	for i in prefs.item_count:
		var s := prefs.get_item_submenu(i)
		if s == "":
			continue
		var sub := prefs.get_node_or_null(NodePath(s))
		var checks := 0
		if sub is PopupMenu:
			for j in (sub as PopupMenu).item_count:
				if not (sub as PopupMenu).is_item_separator(j) \
						and (sub as PopupMenu).is_item_checked(j):
					checks += 1
		var moved: bool = prefs.get_item_text(i) != String(before[s])
		if checks != 1 and moved:
			lied += 1
			_log("  LIED %s: %d checked but the row was stamped anyway" % [s, checks])
	_check(lied == 0,
		"a group without exactly one checked item shows no value at all")

	## Idempotent: firing the handler again must not append a second readout.
	var snap := {}
	for i in prefs.item_count:
		var s := prefs.get_item_submenu(i)
		if s != "":
			snap[s] = prefs.get_item_text(i)
	prefs.about_to_popup.emit()
	await _frames(2)
	var drifted := 0
	for i in prefs.item_count:
		var s := prefs.get_item_submenu(i)
		if s != "" and prefs.get_item_text(i) != String(snap[s]):
			drifted += 1
			_log("  DRIFT %s: %s -> %s" % [s, snap[s], prefs.get_item_text(i)])
	_check(drifted == 0, "re-opening the menu does not append a second readout")

	## The separator is pinned against a literal, not against its own constant.
	for i in prefs.item_count:
		var s := prefs.get_item_submenu(i)
		if s == "" or prefs.get_item_text(i) == String(before[s]):
			continue
		_check(prefs.get_item_text(i).begins_with(String(before[s]) + "   "),
			"%s keeps its authored label and a three-space separator" % s)
		break

	## **A fixture built to reach the `n == 1` guard.** Without it that guard
	## survives mutation to `n >= 1`: no group in this build ever has two items
	## checked, so nothing distinguishes "exactly one" from "at least one". This
	## checks a SECOND item in a real radio group and requires the row to go
	## blank -- a group that cannot say which option is selected must not name
	## one. Restored afterwards so the assertions above stay reproducible.
	var victim: PopupMenu = null
	var victim_row := -1
	for i in prefs.item_count:
		if prefs.get_item_submenu(i) == "GpuMultiMode":
			victim = prefs.get_node_or_null(NodePath("GpuMultiMode")) as PopupMenu
			victim_row = i
	if victim == null:
		_check(false, "fixture: GpuMultiMode submenu not found")
	else:
		var was := []
		for j in victim.item_count:
			was.append(victim.is_item_checked(j))
		var second := -1
		for j in victim.item_count:
			if not victim.is_item_checked(j) and not victim.is_item_separator(j):
				second = j
				break
		victim.set_item_checked(second, true)
		prefs.about_to_popup.emit()
		await _frames(2)
		var txt := prefs.get_item_text(victim_row)
		_log("  fixture: two items checked in GpuMultiMode -> row reads %s" % txt)
		_check(txt == String(before["GpuMultiMode"]),
			"two checked items leaves the row at its authored label")
		for j in victim.item_count:
			victim.set_item_checked(j, bool(was[j]))
		prefs.about_to_popup.emit()
		await _frames(2)
		_check(prefs.get_item_text(victim_row) != String(before["GpuMultiMode"]),
			"fixture restored: the row names its value again")

	if not blank.is_empty():
		_log("  groups with no value yet (their own about_to_popup has not "
			+ "run, so nothing is checked): %s" % str(blank))
	_log("RESULT %s  failures=%d" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(1 if _fail > 0 else 0)

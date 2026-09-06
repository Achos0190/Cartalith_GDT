extends Node
## `Help ▸ Save diagnostic report…` -- does the review panel list the real
## values, and does the file it writes match the panel row for row?
##
##   Godot_v4.7.1-stable_win64.exe --path . _diagreview_probe.tscn
##
## **WINDOWED, at 1600x1000 -- pointer density.** Not a style preference:
## `DccWidgets.modal_card()` is drawn, `modal_stat()` rows are laid out by a
## real `VBoxContainer`, and the screenshot at the end is a framebuffer read.
## `RenderingServer.frame_post_draw` never fires under the dummy display
## driver, so a `--headless` run would hang to the watchdog having proven
## nothing (`_cull_probe.gd` measured this 2026-09-01). The density is named
## beside every measurement because `DccTheme` resolves different figures for
## phone and tablet -- 1600x1000 is neither.
##
## The claim this exists to test is not "a dialog opened". It is:
##
##   1. the panel's rows come from `DiagnosticReport.manifest()` and are drawn
##      with the *real* values, not placeholders;
##   2. every row names where its value came from;
##   3. the WRITTEN FILE lists exactly those rows, in that order, and carries a
##      section for each included one -- so the user saw what was written;
##   4. the two toggles actually change the file, in both directions, and an
##      excluded row is DASHED WITH A REASON rather than blanked;
##   5. the log-tail row is not a fiction in either direction -- it is included
##      with real bytes where the log exists, dashed where it does not.
##
## `_diagreport_probe.gd` is the sibling that exercises `DiagnosticReport.write()`
## end to end from a real engine failure; this one is about the panel and the
## correspondence, and deliberately does not re-test what that one covers.

const W := 1600
const H := 1000
const DENSITY := "1600x1000 pointer"

var _app: Node
var _bridge
var _fails := 0

func _fail(msg: String) -> void:
	print("PROBE FAIL: ", msg)
	_fails += 1

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 180.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	if DisplayServer.get_name() == "headless":
		print("PROBE ABORT: run windowed -- frame_post_draw never fires under the dummy driver")
		get_tree().quit(2)
		return
	DisplayServer.window_set_size(Vector2i(W, H))
	get_window().size = Vector2i(W, H)

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	print("PROBE density=", DENSITY, " phone=", DccTheme.is_phone(), " tablet=", DccTheme.is_tablet())

	## A REAL engine failure, so the last-error row has something true in it.
	var missing_path := ProjectSettings.globalize_path("user://__diagreview_missing__.zip")
	_bridge.load_save(missing_path)

	_bridge.generate({
		"seed": 55019, "width_km": 900.0, "grid_w": 192, "grid_h": 144,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.5).timeout

	_check_still_findable()

	await _case("defaults", {}, missing_path)
	await _case("seed_off", {DiagnosticReport.OPT_SEED_PARAMS: false}, missing_path)
	## The path row with NO project open -- the dashed branch, which is the one
	## a fresh session actually hits.
	await _case("path_on_no_project", {DiagnosticReport.OPT_PROJECT_PATH: true}, missing_path)

	## …and the same toggle with a project open, which is the other branch.
	## `MISTAKES.md`: "for a two-branch dash, exercise BOTH branches -- a probe
	## that only walks the live path cannot see an inversion." Written through
	## `app.gd`'s own `_write_project()` rather than by assigning
	## `current_project_path`, so what the row reads is what a real save set.
	var proj := ProjectSettings.globalize_path("user://__diagreview_project__.zip")
	_app._write_project(proj)
	await get_tree().create_timer(1.5).timeout
	print("PROBE current_project_path after save = ", _app.current_project_path)
	if String(_app.current_project_path) == "":
		_fail("setup: _write_project did not set current_project_path -- the included branch cannot be tested")
	await _case("path_on_with_project", {DiagnosticReport.OPT_PROJECT_PATH: true}, missing_path)

	await _shot()

	print("PROBE fails=", _fails)
	get_tree().quit(0 if _fails == 0 else 1)

## Opens the panel, flips the toggles the case names, reads the rows OFF THE
## DRAWN TREE, writes the file, and compares. Reading the labels back out of
## the `Control`s rather than out of `manifest()` twice is the point: a panel
## that renders nothing would otherwise pass every check.
func _case(name: String, opts_overrides: Dictionary, missing_path: String) -> void:
	print("PROBE --- case ", name, " ---")
	DiagnosticReviewDialog.open(_app, _bridge)
	await get_tree().process_frame
	await get_tree().process_frame
	var dlg := _find_dialog()
	if dlg == null:
		_fail("case %s: no review dialog was added to the app" % name)
		return

	## Flip through the real CheckBoxes, which is what a user does -- not by
	## reaching into the options dictionary the dialog closed over.
	for key in opts_overrides:
		var want: bool = bool(opts_overrides[key])
		var cb := _checkbox_for(dlg, key)
		if cb == null:
			_fail("case %s: no toggle found for %s" % [name, key])
			continue
		if cb.button_pressed != want:
			cb.button_pressed = want
			cb.toggled.emit(want)
			await get_tree().process_frame

	var panel := _panel_rows(dlg)
	print("PROBE case ", name, " panel rows=", panel.size())
	if panel.is_empty():
		_fail("case %s: the panel drew no rows" % name)
		_dismiss(dlg)
		return
	for r in panel:
		print("PROBE   row  ", r["label"], "  |  ", r["value"], "  |  ", r["source"])
		if String(r["source"]).strip_edges() == "":
			_fail("case %s: row %s names no source" % [name, r["label"]])
		if String(r["value"]).strip_edges() == "":
			_fail("case %s: row %s drew an EMPTY value -- a dash with a reason or a value, never blank"
				% [name, r["label"]])

	## Requirement 4: an excluded row is dashed, not blank, and its reason is on
	## the row. `modal_stat_absent()` writes the em dash and the tooltip.
	for r in panel:
		if String(r["value"]) == "—" and String(r["tip"]).strip_edges() == "":
			_fail("case %s: row %s is dashed with NO reason" % [name, r["label"]])

	## Now the file, through the dialog's own Save button.
	var before := _newest_report()
	var save := _button_named(dlg, "Save report")
	if save == null:
		_fail("case %s: no Save report button" % name)
		_dismiss(dlg)
		return
	save.pressed.emit()
	await get_tree().process_frame
	var path := _newest_report()
	if path == "" or path == before:
		_fail("case %s: Save report wrote no new file" % name)
		return
	var rf := FileAccess.open(path, FileAccess.READ)
	var text := rf.get_as_text()
	rf.close()
	print("PROBE case ", name, " file=", path.get_file(), " bytes=", text.length())

	_check_manifest_matches(name, panel, text)
	_check_toggles(name, opts_overrides, text, missing_path)

## Requirement 3. The file's `== WHAT THIS FILE CONTAINS ==` block, parsed back
## out, must be the panel's labels in the panel's order -- and each row the
## block marks `[x]` must have its own `== label ==` section further down.
func _check_manifest_matches(name: String, panel: Array, text: String) -> void:
	var listed: Array[String] = []
	var included: Array[String] = []
	var inside := false
	for raw in text.split("\n"):
		var line := String(raw)
		if line.begins_with("== WHAT THIS FILE CONTAINS =="):
			inside = true
			continue
		if inside:
			if line.strip_edges() == "":
				break
			if not (line.begins_with("[x] ") or line.begins_with("[ ] ")):
				continue
			var rest := line.substr(4)
			var cut := rest.rfind("  [")
			var label := rest.substr(0, cut) if cut > 0 else rest
			listed.append(label)
			if line.begins_with("[x] "):
				included.append(label)
	if listed.size() != panel.size():
		_fail("case %s: file lists %d rows, panel drew %d" % [name, listed.size(), panel.size()])
	var n: int = min(listed.size(), panel.size())
	for i in n:
		if listed[i].to_upper() != String(panel[i]["label"]).to_upper():
			_fail("case %s: row %d -- panel says \"%s\", file says \"%s\""
				% [name, i, panel[i]["label"], listed[i]])
	for label in included:
		if text.find("== %s ==" % label) < 0:
			_fail("case %s: \"%s\" is listed as included and has no section" % [name, label])
	print("PROBE case ", name, " manifest rows matched=", n, " included=", included.size())

## Requirement 4, the half a row-count cannot see: the toggles must actually
## change the BYTES, in the direction they claim.
func _check_toggles(name: String, opts_overrides: Dictionary, text: String,
		missing_path: String) -> void:
	var seed_line := GenInfoDialog.seed_text(_bridge)
	var seed_off: bool = opts_overrides.has(DiagnosticReport.OPT_SEED_PARAMS) \
		and not bool(opts_overrides[DiagnosticReport.OPT_SEED_PARAMS])
	## **Scoped to before the log tail, and the scoping is the finding.** The
	## first version of this check searched the WHOLE file for the seed string
	## and failed with the toggle off -- correctly. The seed was in the log
	## tail, put there by this probe's own `print` of the panel rows. A toggle
	## drops a SECTION; it is not a redaction over verbatim app output, and
	## both surfaces now say so. Asserting otherwise would have demanded a
	## filter nobody can write honestly.
	var upto: int = text.find("== Log tail")
	var scoped: String = text if upto < 0 else text.substr(0, upto)
	var has_seed: bool = seed_line != "" and scoped.find(seed_line) >= 0
	var has_params: bool = scoped.find(GenInfoDialog.PARAMS_HEADING) >= 0
	if seed_off:
		if has_seed or has_params:
			_fail("case %s: the seed toggle is OFF and the file still carries seed=%s params=%s"
				% [name, has_seed, has_params])
		if text.find("== Seed & generation parameters ==") >= 0:
			_fail("case %s: the seed toggle is OFF and the section is still written" % name)
		## And the disclosure that makes the scoping above honest must be in
		## the file, or the toggle is overselling itself to the user.
		if text.find("not filtered") < 0:
			_fail("case %s: the file does not disclose that the log tail is unfiltered" % name)
	else:
		if seed_line != "" and not has_seed:
			_fail("case %s: the seed toggle is ON and the file has no seed line" % name)
		if not has_params:
			_fail("case %s: the seed toggle is ON and the file has no parameter dump" % name)

	## The project path is only ASSERTABLE when a project is open; with none
	## open the row must dash with that as its reason, which is the honest
	## outcome and is checked rather than skipped.
	var path_on: bool = bool(opts_overrides.get(DiagnosticReport.OPT_PROJECT_PATH, false))
	var project: String = String(_app.current_project_path)
	if path_on and project != "":
		if text.find(project) < 0:
			_fail("case %s: the path toggle is ON, a project is open, and the file omits it" % name)
	elif path_on:
		if text.find("no project is open") < 0:
			_fail("case %s: no project is open and the file does not say so" % name)
	else:
		if text.find("[x] Project file path") >= 0:
			_fail("case %s: the path toggle is OFF and the path was written anyway" % name)

	## Requirement 5. The log row, in whichever direction is true here.
	var log_path := String(ProjectSettings.get_setting("debug/file_logging/log_path",
		"user://logs/godot.log"))
	var log_exists: bool = FileAccess.file_exists(log_path)
	var log_included: bool = text.find("[x] Log tail") >= 0
	print("PROBE case ", name, " log file exists=", log_exists, " row included=", log_included)
	if log_exists != log_included:
		_fail("case %s: log file exists=%s but the row says included=%s"
			% [name, log_exists, log_included])

	## The last error must have survived from a REAL failure, unconditionally.
	if text.find(missing_path) < 0:
		_fail("case %s: the file does not carry the real load failure" % name)

## **The row was RENAMED (`…` added), so it has to still be findable.**
## `MISTAKES.md`: the searchable index is built by walking the live `MenuBar`,
## and it asserts by TITLE rather than through `search()`, which matches blurbs
## and would pass on a neighbour's tooltip. This row did not leave the bar --
## it only grew an ellipsis, which is the shell's own mark for "asks before it
## acts" -- so no `EXTRAS`/`UNLISTED` row is owed; this checks that.
## The bar's text and the index's title are DIFFERENT STRINGS, and the first
## version of this check asserted the wrong one and failed -- correctly.
## `command_index.gd::_walk_popup()` indexes
## `text.replace("…", "").strip_edges()`, so an ellipsis is invisible to the
## searcher by design. Both are asserted rather than one: the ellipsis has to
## be ON the row (it is the shell's mark for "asks before it acts") and OFF the
## index entry (or a search for the plain words would miss it).
const ROW_TITLE := "Save diagnostic report…"
const INDEX_TITLE := "Save diagnostic report"

func _check_still_findable() -> void:
	var bar_hits := 0
	var buttons: Array = []
	_menu_buttons(_app, buttons)
	for mb in buttons:
		var popup: PopupMenu = (mb as MenuButton).get_popup()
		if popup == null:
			continue
		for i in popup.item_count:
			if popup.get_item_text(i) == ROW_TITLE:
				bar_hits += 1
	print("PROBE menu rows titled \"", ROW_TITLE, "\" = ", bar_hits)
	if bar_hits != 1:
		_fail("the Help row carries its ellipsis %d times, want exactly 1" % bar_hits)
	_check_indexed()

## `get_children(true)` -- a MenuButton keeps its popup as an INTERNAL child.
func _menu_buttons(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_menu_buttons(c, out)

func _check_indexed() -> void:
	## Built here rather than reached for on the app: `dcc_shell.gd` keeps
	## `_command_index` private and rebuilds it lazily, and `_cmdindex_probe.gd`
	## takes the same `CommandIndex.new()` / `build(app, bridge)` route.
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	var by_title := 0
	var available := false
	for e in idx.all():
		var row := e as Dictionary
		if String(row.get("title", "")) == INDEX_TITLE:
			by_title += 1
			available = bool(row.get("available", false))
	print("PROBE index entries titled \"", INDEX_TITLE, "\" = ", by_title,
		" of ", idx.size(), " available=", available)
	if by_title != 1:
		_fail("the renamed Help row is indexed %d times, want exactly 1" % by_title)
	if not available:
		_fail("the Help row is indexed but marked unavailable")

func _find_dialog() -> AcceptDialog:
	var found: AcceptDialog = null
	for c in _app.get_children():
		if c is AcceptDialog and (c as AcceptDialog).title == "Save diagnostic report":
			found = c
	return found

func _dismiss(dlg: AcceptDialog) -> void:
	if is_instance_valid(dlg):
		dlg.hide()
		dlg.queue_free()

## The drawn rows: every `HBoxContainer` whose first child is a mono label and
## whose second is the value, each followed by the `from ...` source line the
## panel puts under it.
func _panel_rows(dlg: AcceptDialog) -> Array:
	var out: Array = []
	var labels: Array[Control] = []
	_collect(dlg, labels)
	var i := 0
	while i < labels.size():
		var node := labels[i]
		if node is HBoxContainer:
			var kids := (node as HBoxContainer).get_children()
			if kids.size() == 2 and kids[0] is Label and kids[1] is Label:
				## The source line is the next `from …` Label after the row,
				## not the next Control: `_collect` is depth-first, so the
				## row's own two Labels come between them.
				var src := ""
				for j in range(i + 1, labels.size()):
					if labels[j] is HBoxContainer:
						break
					if labels[j] is Label:
						var t := String((labels[j] as Label).text)
						if t.begins_with("from "):
							src = t.substr(5)
							break
				out.append({
					"label": String((kids[0] as Label).text),
					"value": String((kids[1] as Label).text),
					"source": src,
					"tip": String((node as Control).tooltip_text),
				})
		i += 1
	return out

## Depth-first, in draw order -- the sequence matters, because the source line
## is identified by being the node right after its row.
func _collect(n: Node, out: Array[Control]) -> void:
	for c in n.get_children():
		if c is Control:
			out.append(c as Control)
		_collect(c, out)

func _checkbox_for(dlg: AcceptDialog, key: String) -> CheckBox:
	var want := "seed" if key == DiagnosticReport.OPT_SEED_PARAMS else "project file path"
	var all: Array[Control] = []
	_collect(dlg, all)
	var last_label := ""
	for c in all:
		if c is Label:
			last_label = String((c as Label).text).to_lower()
		elif c is CheckBox and last_label.find(want) >= 0:
			return c as CheckBox
	return null

func _button_named(dlg: AcceptDialog, text: String) -> Button:
	var all: Array[Control] = []
	_collect(dlg, all)
	for c in all:
		if c is Button and String((c as Button).text) == text:
			return c as Button
	return null

func _newest_report() -> String:
	var dir := DccSettings.storage_root("exports")
	var best := ""
	var best_t := -1
	var da := DirAccess.open(dir)
	if da == null:
		return ""
	da.list_dir_begin()
	var f := da.get_next()
	while f != "":
		if f.begins_with("cartalith_diagnostic_report_") and f.ends_with(".txt"):
			var full := dir.path_join(f)
			var t := FileAccess.get_modified_time(full)
			if t > best_t or (t == best_t and full > best):
				best_t = t
				best = full
		f = da.get_next()
	da.list_dir_end()
	return best

## The panel on screen, at the named density, with the real values in it --
## plus the one layout number `diagnostic_review_dialog.gd` quotes, measured
## here rather than asserted there. The source strings are constants, not
## world-dependent content, so one sample is the whole population for them.
func _shot() -> void:
	DiagnosticReviewDialog.open(_app, _bridge)
	await get_tree().process_frame
	await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var dlg := _find_dialog()
	if dlg != null:
		var all: Array[Control] = []
		_collect(dlg, all)
		var worst := 0
		var worst_text := ""
		for c in all:
			if c is Label and String((c as Label).text).begins_with("from "):
				var n := (c as Label).get_line_count()
				if n > worst:
					worst = n
					worst_text = String((c as Label).text)
		print("PROBE source lines at CARD_W=", DiagnosticReviewDialog.CARD_W,
			" worst=", worst, " (", worst_text, ")")
		if worst > 2:
			_fail("a source line wraps to %d lines at CARD_W=%d -- the table stops being scannable"
				% [worst, DiagnosticReviewDialog.CARD_W])
	var img := get_viewport().get_texture().get_image()
	var out := DccSettings.storage_root("exports").path_join("diagreview_panel.png")
	img.save_png(out)
	print("PROBE shot -> ", out, " (", DENSITY, ")")

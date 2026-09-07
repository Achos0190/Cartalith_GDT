extends Node
## TABLET-FIT lane, 2026-09-07. Guards the portrait dock split and measures what
## is left of the frame overflow after it.
##
##   godot --headless --path . _tabfit_probe.tscn -- --force-touch --vp 800x1280
##   godot --headless --path . _tabfit_probe.tscn -- --force-touch --vp 1280x800
##   godot --headless --path . _tabfit_probe.tscn --              --vp 1920x1080
##
## **Every expected figure below is a literal read off the canvas or off the
## desktop token table, never off the constant under test.** `TABLET_PORTRAIT`'s
## 232 is `Cartalith Tablet.dc.html`'s `--ldW:232px` in `valsShell()`'s portrait
## branch; 400 is `W_DOCK_TABLET`'s documented landscape figure written out; and
## 372/304 are `--ldW`/`--rdW` at `ENV:25`. Asserting any of them against
## `DccTheme.TABLET_PORTRAIT["w_left_dock"]` would assert a relationship and
## pass under a mutant, which is the whole point of writing them out.
##
## ── The residual, and why it is measured rather than asserted ────────────────
##
## **The docks were not the cause of the frame overflow, only a passenger, and
## this probe is where that was found.** The brief's arithmetic -- rail 47 +
## left 400 + viewport 237 + right 400 = 1084 against a frame of 800 -- reads
## the *result* of the layout: the viewport had no minimum of its own and simply
## expanded into whatever the row was given. Walking every node's combined
## minimum at 800 x 1280 puts the row's real driver somewhere else entirely:
##
##   dock row HBox (rail + both docks)     847 before this batch, 611 after
##   menu_bar_row                          486  (850 with its panel's readouts)
##   tool_options_row                    1 057  <- six touch-sized text buttons
##   shell VBox                          1 085  = tool_options_row + margins
##
## So `App` is 800 wide, the shell VBox under it cannot go below its own 1085 px
## combined minimum, and the overflow is 285 px whatever the docks do. Cutting
## them from 800 px of that budget to 563 removes their share of it and is a
## prerequisite for any fit at all, but it does not clear the frame on its own.
##
## **The rest is not attempted here, deliberately.** Clearing 1085 -> 850 needs
## the tool-options bar to stop propagating a minimum (a scroll or an
## icon-collapse -- a design question, and the canvas answers it with a
## different bar composition), and clearing 850 -> 800 then needs the 7-to-3
## menu collapse, which `TABLET_UI_SPEC.md` holds behind an owner decision that
## has not been made and which this lane was explicitly scoped out of. Guessing
## either would be canvas adoption under another name. The figures above are
## printed on every run so the next lane starts from a measurement.

var app: Node
var _vp: SubViewport
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _eq(label: String, got: float, want: float) -> void:
	var ok := absf(got - want) < 0.5
	if not ok:
		_fail += 1
	print("  [%s] %-42s got=%8.1f want=%8.1f" % ["ok" if ok else "FAIL", label, got, want])

func _le(label: String, got: float, cap: float) -> void:
	var ok := got <= cap + 0.5
	if not ok:
		_fail += 1
	print("  [%s] %-42s got=%8.1f  cap=%8.1f" % ["ok" if ok else "FAIL", label, got, cap])

func _ready() -> void:
	var parts: PackedStringArray = _arg("--vp", "800x1280").split("x")
	var vw := int(parts[0])
	var vh := int(parts[1])
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(6)

	var portrait := vh > vw
	var tablet: bool = DccTheme.is_tablet()
	print("=== tabfit vp=%dx%d portrait=%s tablet=%s tablet_portrait=%s ===" % [
		vw, vh, str(portrait), str(tablet), str(DccTheme.is_tablet_portrait())])

	## The width the shell ASKED for. `size.x` is content-driven and can exceed
	## it (the left dock's Generate panel does, at 331); `custom_minimum_size.x`
	## is the reservation `_compute_layout_mode()` made and is the thing this
	## batch changed.
	var ld := app.left_dock as Control
	var rd := app.right_dock as Control
	var rail := app.rail_column as Control

	## Canvas literals, written out. Portrait tablet 232 (`--ldW:232px`);
	## landscape tablet 400 (unchanged this batch, and deliberately NOT the
	## canvas's 320 -- see `DccTheme.TABLET_PORTRAIT`); pointer 372 / 304.
	var want_l := 372.0
	var want_r := 304.0
	if tablet:
		want_l = 232.0 if portrait else 400.0
		want_r = 232.0 if portrait else 400.0
	elif DccTheme.is_laptop():
		## `LAPTOP`'s own pair, written out: `--ldW:330px;--rdW:280px` at
		## `ENV:1819`'s `w1366` override. Present so a pointer run at 1366 is a
		## real PC-unchanged measurement and not a skipped one.
		want_l = 330.0
		want_r = 280.0
	print("[dock reservation]")
	_eq("left_dock custom_minimum_size.x", ld.custom_minimum_size.x, want_l)
	_eq("right_dock custom_minimum_size.x", rd.custom_minimum_size.x, want_r)
	_eq("role_px(w_left_dock)", float(DccTheme.role_px("w_left_dock")), want_l)
	_eq("role_px(w_right_dock)", float(DccTheme.role_px("w_right_dock")), want_r)

	## The dock row's own budget: rail + both docks must leave the frame room
	## for a map. This is the assertion the batch exists for.
	print("[dock row fits the frame]")
	var row_min: float = rail.get_combined_minimum_size().x \
		+ ld.get_combined_minimum_size().x + rd.get_combined_minimum_size().x
	_le("rail+left+right combined minimum", row_min, float(vw))
	var dock_row := ld.get_parent() as Control
	_le("dock row HBox combined minimum", dock_row.get_combined_minimum_size().x, float(vw))

	## What is left. The shell's own frame fit is NOT the dock row's: the
	## tool-options bar carries a combined minimum of its own and is measured
	## rather than asserted, because its remedy is a design question this batch
	## was scoped out of.
	print("[residual: whole-shell frame fit]")
	var shell_vb := dock_row.get_parent() as Control
	print("  shell VBox   min.x=%7.1f  size.x=%7.1f  frame=%d  overflow=%.1f" % [
		shell_vb.get_combined_minimum_size().x, shell_vb.size.x, vw,
		maxf(0.0, shell_vb.size.x - float(vw))])
	for pair in [["menu_bar_row", app.menu_bar_row], ["tool_options_row", app.tool_options_row],
			["status_row", app.status_row]]:
		var row := pair[1] as Control
		if row != null:
			print("  %-18s min.x=%7.1f" % [pair[0], row.get_combined_minimum_size().x])
	print("  right_dock right edge x=%.1f" % [rd.global_position.x + rd.size.x])
	print("=== end tabfit fails=%d ===" % _fail)
	get_tree().quit()

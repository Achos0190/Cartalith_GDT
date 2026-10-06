extends Node
## The status bar's `hint` slot must never widen the shell
## (`OUTSTANDING_WORK.md`: "The status bar lets a long hint sentence widen the
## whole shell past a 1600 px window"; fix: `dcc_shell.gd`'s `_status_tail` and
## `_fit_status_tail()`).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _statushint_probe.tscn \
##       -- [--vp WxH] [--shot PATH]
##
## Windowed: the screenshot leg reads the framebuffer, and a headless run aborts.
## `--vp` sets the SubViewport size (default 1600x1000); `--shot` saves a PNG of
## it with the long catchment sentence showing. Any other argument is refused.
##
## Every check starts with "Protects:" and says what regression it catches.
## Prints one `### STATUSHINT GREEN|RED n/m ###` line; the exit code is 0 either
## way (the harness greps), as for every probe here.

const LONG_HINT := "Catchment: 18 342 cells upstream of the clicked cell, draining 41 % of the continent through 7 named rivers to a single mouth on the western sea; Clear river highlight (right-click) removes the wash"
const SHORT_HINT := "V M R · B F · ⌘Z · Esc"
const MID := "last pass 10 Resources · 9507 ms · repaint 1569 ms"

var app: Node
var _vp: SubViewport
var _checks := 0
var _fails := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## One counted check; prints `PROBE-FAIL` on a miss so the harness can grep it.
func _ok_true(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("SH %s  %s%s" % ["ok  " if cond else "PROBE-FAIL", name,
		("  -- " + detail) if detail != "" else ""])

## The hint label's drawn text width (what `Label` would need unclipped).
func _natural(l: Label) -> float:
	var font: Font = l.get_theme_font("font")
	return font.get_string_size(l.text, HORIZONTAL_ALIGNMENT_LEFT, -1,
		l.get_theme_font_size("font_size")).x

func _ready() -> void:
	var vp_size := Vector2i(1600, 1000)
	var shot := ""
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--vp" and i + 1 < args.size():
			var p := String(args[i + 1]).split("x")
			vp_size = Vector2i(int(p[0]), int(p[1]))
			i += 2
		elif args[i] == "--shot" and i + 1 < args.size():
			shot = args[i + 1]
			i += 2
		else:
			print("### STATUSHINT ABORT: unknown argument '%s' ###" % args[i])
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("### STATUSHINT ABORT: headless -- the screenshot leg reads the framebuffer ###")
		get_tree().quit(2)
		return
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)
	if app.get_script() == null or not app.has_method("_refresh_rail_foot"):
		print("### STATUSHINT ABORT: app.gd did not compile ###")
		get_tree().quit(1)
		return
	print("SH viewport %s" % [vp_size])

	var hint: Label = app._status_labels["hint"]
	var mid: Label = app._status_labels["mid"]
	var tail: Control = app._status_tail
	var row: Control = app.status_row

	## 1. A short hint with room: drawn in full, END-aligned, exactly as the old
	## spacer layout drew it.
	app.set_status("mid", MID, "text_ghost")
	app.set_status("hint", SHORT_HINT, "text_ghost")
	await _frames(3)
	var base_min: float = row.get_combined_minimum_size().x
	_ok_true("Protects: a short hint with room draws in full (label %.0f px, text %.0f px)" % [hint.size.x, _natural(hint)],
		hint.size.x >= _natural(hint) and hint.visible)
	_ok_true("Protects: the hint's right edge is the status bar's right edge, as before the fix (%.1f vs %.1f)" % [
			hint.get_global_rect().end.x, tail.get_global_rect().end.x],
		absf(hint.get_global_rect().end.x - tail.get_global_rect().end.x) <= 0.5)
	_ok_true("Protects: mid sits one slot gap left of the hint, as before the fix",
		absf(hint.get_global_rect().position.x - mid.get_global_rect().end.x - float(DccShell.STATUS_SLOT_GAP)) <= 1.0)

	## 2. The long catchment sentence must not move the minimum, and must still
	## be shown (ellipsised) with the whole sentence as the tooltip.
	app.set_status("hint", LONG_HINT, "text_ghost")
	await _frames(3)
	var long_min: float = row.get_combined_minimum_size().x
	_ok_true("Protects: a long hint does not raise the status bar's minimum width (%.0f px with it, %.0f px with the short one)" % [long_min, base_min],
		long_min == base_min and long_min <= float(vp_size.x))
	_ok_true("Protects: the long hint is still drawn, not collapsed by clip_text (%.0f px wide)" % hint.size.x,
		hint.visible and hint.size.x > 100.0)
	_ok_true("Protects: the long hint's label is no wider than the tail's room (never past the window)",
		hint.get_global_rect().end.x <= tail.get_global_rect().end.x + 0.5
			and hint.get_global_rect().position.x >= mid.get_global_rect().end.x)
	_ok_true("Protects: the full sentence is the tooltip", tail.tooltip_text == LONG_HINT)
	var bar: Control = row.get_parent().get_parent()
	_ok_true("Protects: the whole status bar (padding included) is no wider than the window with the long hint (min %.0f of %d)" % [bar.get_combined_minimum_size().x, vp_size.x],
		bar.get_combined_minimum_size().x <= float(vp_size.x) and bar.size.x <= float(vp_size.x))
	if shot != "":
		await RenderingServer.frame_post_draw
		var img: Image = _vp.get_texture().get_image()
		img.save_png(shot)
		print("SH screenshot %s (%dx%d)" % [shot, img.get_width(), img.get_height()])

	## 3. Clearing the hint restores the old minimum and hides the label.
	app.set_status("hint", "", "text_ghost")
	await _frames(3)
	_ok_true("Protects: an empty hint is omitted and the minimum width returns", not hint.visible
		and row.get_combined_minimum_size().x <= base_min)

	print("### STATUSHINT %s  %d/%d checks passed ###" % [
		"GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0)

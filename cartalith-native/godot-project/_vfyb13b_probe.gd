extends Node
## VERIFIER probe, 2026-09-05, Lane B. Independent re-derivation of the browse
## dialog overflow, BEFORE and AFTER, shallow and deep.
##
## Run WINDOWED. `--headless` gives a 64x64 dummy viewport, which resolves the
## shell to LAPTOP and clamps the dialog to `min_size`, so every figure below
## would be a measurement of the dummy driver. Size is set with
## `DisplayServer.window_set_size` and the shell's OWN `_compute_layout_mode()`
## is re-run, because this machine's screen is 1680x1050 and DESKTOP density is
## otherwise unreachable here.
##
##   Godot..._console.exe --path . _vfyb13b_probe.tscn -- --vp 1920x1080
##
## The three things this asks that a contents-fit check alone does not:
##   * the LAST crumb -- the current folder -- must be inside the scroll
##     viewport. Eliding the tail is useless; a fix that scrolls to 0 and leaves
##     the user's own folder off-screen passes a width check and fails the user.
##   * no `clip_text` anywhere on the crumbs. It collapses a Label minimum to 1
##     and would make the width check pass vacuously.
##   * a path that FITS must not be scrolled (the positive control), so a scroll
##     stuck at 0 cannot be mistaken for a working pin.

var app: Node
var _fail := 0
var _root := ""

const NAMES := ["Cartalith Projects", "Northern Reach", "exports", "heightmaps",
	"2026-09-05", "revision-04", "tiles", "z12", "final", "scratch"]

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(label: String, cond: bool, detail: String = "") -> void:
	print("VB %s  %s%s" % ["ok  " if cond else "FAIL", label,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _deep(segments: int, realistic: bool) -> String:
	var p := _root.path_join("real" if realistic else "short")
	var guard := 0
	while p.split("/", false).size() < segments and guard < 40:
		p = p.path_join(String(NAMES[guard % NAMES.size()]) if realistic else "d%d" % guard)
		guard += 1
	DirAccess.make_dir_recursive_absolute(p)
	return p

## True when this build wraps the crumbs in a scroll at all (the AFTER shape).
func _has_scroll(d) -> bool:
	return d.get("_crumb_scroll") != null

func _clip_anywhere(n: Node) -> bool:
	for c in n.get_children():
		if c is Button and (c as Button).clip_text:
			return true
		if c is Label and (c as Label).clip_text:
			return true
		if _clip_anywhere(c):
			return true
	return false

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vb13"
	DirAccess.make_dir_recursive_absolute(_root)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	var ua := OS.get_cmdline_user_args()
	for i in ua.size():
		if String(ua[i]) == "--vp" and i + 1 < ua.size():
			var wh := String(ua[i + 1]).split("x")
			DisplayServer.window_set_size(Vector2i(int(wh[0]), int(wh[1])))
			await _frames(6)
			app._compute_layout_mode()
			DccTheme.set_phone(app._phone)
			await _frames(2)
	var vp: Vector2 = app.get_viewport_rect().size
	var density := "PHONE" if DccTheme.is_phone() else (
		"TABLET" if DccTheme.is_tablet() else (
			"LAPTOP" if DccTheme.is_laptop() else "DESKTOP"))
	print("VB banner: display=%s density=%s viewport=%dx%d" % [
		DisplayServer.get_name(), density, vp.x, vp.y])

	var shallow := DccBrowseDialog.home_dir()
	var r9 := _deep(9, true)
	var r13 := _deep(13, true)

	for spec in [["FOLDERS", shallow, "shallow3"], ["FOLDERS", r9, "real9"],
			["FOLDERS", r13, "real13"], ["FILES", r13, "real13"],
			["SAVE", r13, "real13"]]:
		var mode: String = spec[0]
		var start: String = spec[1]
		var pname: String = spec[2]
		var d
		if mode == "FOLDERS":
			d = DccBrowseDialog.choose_folder(app, "vb", start, "a hint", Callable())
		elif mode == "FILES":
			d = DccBrowseDialog.choose_file(app, "vb", PackedStringArray(["zip"]), start,
				"a hint", Callable())
		else:
			d = DccBrowseDialog.choose_save_path(app, "vb", "zip", start, "a hint",
				"x.zip", Callable())
		await _frames(5)
		var cm: Vector2 = d.get_contents_minimum_size()
		var depth: int = d._cwd.split("/", false).size()
		var crumb_min: float = d._crumb_row.get_combined_minimum_size().x
		print("VB   %-8s %-9s depth=%2d win=%dx%d contents_min.x=%7.1f crumb_row_min=%7.1f scrolled=%s"
			% [mode, pname, depth, d.size.x, d.size.y, cm.x, crumb_min, _has_scroll(d)])
		_ok("%s %s: contents fit the window" % [mode, pname],
			cm.x <= float(d.size.x), "contents_min.x=%.0f window=%d" % [cm.x, d.size.x])
		_ok("%s %s: no clip_text on the crumbs" % [mode, pname],
			not _clip_anywhere(d._crumb_row))
		if _has_scroll(d):
			var sc: ScrollContainer = d._crumb_scroll
			var overflows: bool = sc.get_h_scroll_bar().max_value > sc.size.x
			var last: Control = d._crumb_row.get_child(d._crumb_row.get_child_count() - 1)
			var lo: float = last.position.x
			var hi: float = lo + last.size.x
			var view_lo: float = sc.scroll_horizontal
			var view_hi: float = view_lo + sc.size.x
			print("VB     scroll_h=%.0f max=%.0f viewport=%.0f  last crumb '%s' [%.0f..%.0f]"
				% [sc.scroll_horizontal, sc.get_h_scroll_bar().max_value, sc.size.x,
					(last as Button).text if last is Button else "?", lo, hi])
			if overflows:
				## The TAIL is what must survive. Eliding the end is useless.
				_ok("%s %s: LAST crumb (current folder) is inside the viewport"
						% [mode, pname], lo >= view_lo - 1.0 and hi <= view_hi + 1.0)
			else:
				## Positive control: a path that fits must sit at 0, so a scroll
				## stuck at 0 cannot be read as a working pin.
				_ok("%s %s: a fitting path is NOT scrolled (positive control)"
						% [mode, pname], is_equal_approx(sc.scroll_horizontal, 0.0))
		d.queue_free()
		await _frames(2)

	print("VB DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

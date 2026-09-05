extends Node
## Lane B, 2026-09-05 -- re-derivation of "the browse dialog is narrower than
## its own contents", and the before/after harness for the fix.
##
## **Corrected 2026-09-05 by a verifier: this header used to document
## `--resolution`, which the probe does not read.** It reads its own
## `-- --vp WxH` user arg (`OS.get_cmdline_user_args()`, below). `--resolution`
## would have been silently ignored and all three "named densities" would have
## measured the same box — three names for one measurement, which is exactly
## the shape of single-sample result this project keeps having to withdraw.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _lb2browse_probe.tscn -- --vp 1920x1080
##   ... -- --vp 1366x768                        (LAPTOP)
##   ... -- --vp 2560x1600 --force-touch         (TABLET)

var app: Node
var _fail := 0
var _root := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(label: String, cond: bool, detail: String = "") -> void:
	print("LB2 %s  %s%s" % ["ok  " if cond else "FAIL", label,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

## A real directory whose absolute path splits into exactly `segments` parts,
## which is what `_refresh_crumbs()` counts. `realistic` picks the folder names
## a map project actually has rather than `d7/d8/d9`: a breadcrumb `Button`
## reports its own text as its minimum width, so segment COUNT is only half the
## variable and a probe built from one-letter names measures a floor.
const NAMES := ["Cartalith Projects", "Northern Reach", "exports", "heightmaps",
	"2026-09-05", "revision-04", "tiles", "z12", "final", "scratch"]

func _deep(segments: int, realistic: bool) -> String:
	var p := _root.path_join("short" if not realistic else "real")
	var guard := 0
	while p.split("/", false).size() < segments and guard < 40:
		p = p.path_join(String(NAMES[guard % NAMES.size()]) if realistic else "d%d" % guard)
		guard += 1
	DirAccess.make_dir_recursive_absolute(p)
	return p

func _dump(d, tag: String) -> void:
	var outer := d.get_child(0) as Control
	## `_build()`'s own order: head, rule, crumbs, well, list, rule, foot.
	var names := ["head", "rule", "crumbs", "well", "list", "rule", "foot"]
	var parts: Array = []
	for i in outer.get_child_count():
		var ctl := outer.get_child(i) as Control
		parts.append("%s=%.0f" % [names[i] if i < names.size() else str(i),
			ctl.get_combined_minimum_size().x])
	print("LB2 %-26s depth=%2d win=%dx%d contents_min=%s crumbs_min=%.0f  [%s]" % [
		tag, d._cwd.split("/", false).size(), d.size.x, d.size.y,
		d.get_contents_minimum_size(), d._crumb_row.get_combined_minimum_size().x,
		", ".join(parts)])

## The accent segment -- the current folder, last child of `_crumb_row`.
func _last_crumb(d) -> Control:
	return d._crumb_row.get_child(d._crumb_row.get_child_count() - 1) as Control

## `⌂ Home` must be a sibling of the scroll, not a passenger inside it.
func _home_outside(d) -> bool:
	var sc: ScrollContainer = d._crumb_scroll
	for c in (sc.get_parent() as Control).get_children():
		if c is Button and String((c as Button).text).contains("Home"):
			return true
	return false

func _ready() -> void:
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-lb2"
	DirAccess.make_dir_recursive_absolute(_root)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	## `--vp WxH`: this machine's only screen is 1680 x 1050, so a windowed run
	## can never make `get_viewport_rect().size.x` reach `W_LAPTOP_MAX` (1920)
	## and the DESKTOP set is otherwise unreachable here. Resizing the OS window
	## past the screen and re-running the shell's OWN `_compute_layout_mode()`
	## -- the single writer of `DccTheme.set_narrow()` -- resolves the density
	## the same way a real 1920 monitor would, rather than asserting it.
	for i in OS.get_cmdline_user_args().size():
		if String(OS.get_cmdline_user_args()[i]) == "--vp" and i + 1 < OS.get_cmdline_user_args().size():
			var wh := String(OS.get_cmdline_user_args()[i + 1]).split("x")
			DisplayServer.window_set_size(Vector2i(int(wh[0]), int(wh[1])))
			await _frames(6)
			app._compute_layout_mode()
			DccTheme.set_phone(app._phone)
			await _frames(2)
	var vp: Vector2 = app.get_viewport_rect().size
	var density := "PHONE" if DccTheme.is_phone() else (
		"TABLET" if DccTheme.is_tablet() else (
			"LAPTOP" if DccTheme.is_laptop() else "DESKTOP"))
	print("LB2 density=%s viewport=%dx%d touch=%s narrow=%s" % [
		density, vp.x, vp.y, DccTheme.is_touch(), DccTheme.is_laptop()])

	var shallow := DccBrowseDialog.home_dir()
	var d9 := _deep(9, false)
	var d13 := _deep(13, false)
	var r9 := _deep(9, true)
	var r13 := _deep(13, true)
	print("LB2 paths shallow(%d)=%s" % [shallow.split("/", false).size(), shallow])
	print("LB2 paths d9=%s" % d9)
	print("LB2 paths d13=%s" % d13)
	print("LB2 paths r9=%s" % r9)
	print("LB2 paths r13=%s" % r13)

	for spec in [["FOLDERS", shallow, "shallow"], ["FOLDERS", d9, "d9"], ["FOLDERS", d13, "d13"],
			["FOLDERS", r9, "r9"], ["FOLDERS", r13, "r13"],
			["FILES", shallow, "shallow"], ["FILES", r13, "r13"],
			["SAVE", r13, "r13"]]:
		var mode: String = spec[0]
		var start: String = spec[1]
		var pname: String = spec[2]
		var d
		if mode == "FOLDERS":
			d = DccBrowseDialog.choose_folder(app, "lb2", start, "a hint worth measuring", Callable())
		elif mode == "FILES":
			d = DccBrowseDialog.choose_file(app, "lb2", PackedStringArray(["zip"]), start,
				"a hint worth measuring", Callable())
		else:
			d = DccBrowseDialog.choose_save_path(app, "lb2", "zip", start,
				"a hint worth measuring", "x.zip", Callable())
		await _frames(4)
		_dump(d, "%s %s %s" % [density, mode, pname])
		var cm: Vector2 = d.get_contents_minimum_size()
		_ok("%s %s %s depth=%d: contents fit the window" % [
				density, mode, pname, d._cwd.split("/", false).size()],
			cm.x <= float(d.size.x),
			"contents_min.x=%.0f window=%d" % [cm.x, d.size.x])
		## The tail pin, and the positive control that must NOT move it: a
		## path that fits has `max_value == 0` and stays at 0, so a probe that
		## only checked the deep case would pass on a scroll stuck at 0.
		var sc: ScrollContainer = d._crumb_scroll
		## Overflow is a MEASURED property, not a property of the path name:
		## 13 one-letter segments are 618 px inside a 655 px viewport and do not
		## overflow at all. Classifying by depth made this probe demand a scroll
		## from a row that had nothing to scroll -- a check that could only fail.
		var deep: bool = sc.get_h_scroll_bar().max_value > sc.size.x
		var last := _last_crumb(d)
		print("LB2      scroll_h=%d max=%.0f visible_w=%.0f last_crumb=[%.0f..%.0f] home_out=%s" % [
			sc.scroll_horizontal, sc.get_h_scroll_bar().max_value, sc.size.x,
			last.position.x, last.position.x + last.size.x, _home_outside(d)])
		if deep:
			_ok("%s %s %s: the crumb view is pinned to its tail" % [density, mode, pname],
				sc.scroll_horizontal > 0, "scroll_h=%d" % sc.scroll_horizontal)
			_ok("%s %s %s: the current folder is on screen" % [density, mode, pname],
				last.position.x - float(sc.scroll_horizontal) >= -0.5
					and last.position.x + last.size.x - float(sc.scroll_horizontal)
						<= sc.size.x + 0.5,
				"[%.0f..%.0f] scroll=%d viewport=%.0f" % [last.position.x,
					last.position.x + last.size.x, sc.scroll_horizontal, sc.size.x])
		else:
			_ok("%s %s %s: a path that fits is not scrolled" % [density, mode, pname],
				sc.scroll_horizontal == 0,
				"scroll_h=%d max=%.0f" % [sc.scroll_horizontal, sc.get_h_scroll_bar().max_value])
			_ok("%s %s %s: the current folder is on screen" % [density, mode, pname],
				last.position.x + last.size.x <= sc.size.x + 0.5,
				"[%.0f..%.0f] viewport=%.0f" % [last.position.x,
					last.position.x + last.size.x, sc.size.x])
		_ok("%s %s %s: Home is outside the scroll" % [density, mode, pname], _home_outside(d))
		d.hide()
		await _frames(2)

	## The foot hint's own width, which `_build_foot()` records three measured
	## figures for. It is `SIZE_EXPAND_FILL`, so its width is the dialog's laid
	## out width minus the buttons -- and the dialog's laid out width used to be
	## whatever the crumb row forced. Re-measured here at the same three note
	## lengths that comment names, shallow and deep, because a fix that stops
	## the crumbs widening the layout moves this number too.
	for note_len in [11, 33, 76]:
		var note := "x".repeat(note_len)
		for spec2 in [["FOLDERS", shallow, "shallow"], ["FOLDERS", r13, "r13"],
				["FILES", shallow, "shallow"], ["FILES", r13, "r13"],
				["SAVE", shallow, "shallow"], ["SAVE", r13, "r13"]]:
			var m: String = spec2[0]
			var st: String = spec2[1]
			var d2
			if m == "FOLDERS":
				d2 = DccBrowseDialog.choose_folder(app, "lb2", st, note, Callable())
			elif m == "FILES":
				d2 = DccBrowseDialog.choose_file(app, "lb2", PackedStringArray(["zip"]),
					st, note, Callable())
			else:
				d2 = DccBrowseDialog.choose_save_path(app, "lb2", "zip", st, note,
					"x.zip", Callable())
			await _frames(4)
			print("LB2 note len=%2d %-8s %-8s note_w=%.1f  win=%d" % [
				note_len, m, spec2[2], d2._foot_note.size.x, d2.size.x])
			_ok("%s %s %s note(%d) is wider than 1 px" % [density, m, spec2[2], note_len],
				d2._foot_note.size.x > 100.0, "%.1f" % d2._foot_note.size.x)
			d2.hide()
			await _frames(2)

	print("LB2 ---- %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

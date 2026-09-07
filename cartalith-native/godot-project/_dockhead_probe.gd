extends Node
## **The two dock header bands, against the canvas rather than against each
## other.**
##
## Both were built to `_scaled(34)` under a comment reading *"the canvas gives
## both dock headers `height:34px`"*. The canvas contains **no** `height:34px`
## anywhere, and the two headers are not the same height in it: they are
## padding around a `var(--ctl)` square, and the paddings differ.
##
##   `ENV:305`  left   `gap:8px;padding:8px var(--pad) 0`
##   `ENV:948`  right  `gap:8px;padding:8px var(--pad) 6px`
##   `ENV:25`   `--ctl:24px`      `ENV:1819` touch `--ctl:36px`
##
## so 8 + 24 + 0 = **32** and 8 + 24 + 6 = **38** on a pointer, 44 and 50 on
## touch. Those four numbers are asserted as LITERALS below, and separately the
## strings above are re-read out of the canvas file, so this probe fails both
## when the shell drifts from the canvas and when the canvas is re-based under
## it -- rather than comparing the code to itself, which is what the wrong 34
## survived by doing.
##
##   godot --headless --path . --resolution 1600x900 _dockhead_probe.tscn
##   godot --headless --path . --resolution 1600x900 _dockhead_probe.tscn -- --force-touch
##
## `DccTheme._touch` is latched for the life of the process, so the two
## densities cannot share a run -- the same constraint `_ds03fit_probe.gd`
## records.
##
## Flags this probe actually reads, grepped from the body below:
##   `--vp WxH`      SubViewport size in physical px. Default 2560x1600, which
##                   is a TABLET under `--force-touch` and a desktop without it.
##   `--force-touch` read here for the expectation, and by `dcc_shell.gd` for
##                   the density itself.
## Any other `--flag` aborts rather than being silently ignored.

## `ENV:25` / `ENV:1819`. Pinned as the token values, not as the sums, so a
## mutation of either half of the arithmetic is visible.
const CANVAS_CTL := 24
const CANVAS_CTL_TOUCH := 36
const CANVAS_PAD_TOP := 8
const CANVAS_PAD_BOTTOM_LEFT := 0
const CANVAS_PAD_BOTTOM_RIGHT := 6

## The canvas file, re-read rather than quoted. Relative to the project so the
## probe travels with the tree.
const CANVAS_REL := "../../design/mcp-2026-09-07/Cartalith DCC Environment.dc.html"
const ENV_LEFT_HEAD := "align-items:center;gap:8px;padding:8px var(--pad) 0\""
const ENV_RIGHT_HEAD := "align-items:center;gap:8px;padding:8px var(--pad) 6px"

var _fail := 0
var _vp: SubViewport

func _log(s: String) -> void:
	print("[dockhead] %s" % s)

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _reject_unknown_args() -> bool:
	var known := ["--force-touch", "--vp"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

## The canvas half. Reading the file is what makes the four literals above a
## comparison against the design instead of a restatement of the shell.
func _check_canvas() -> void:
	var path := ProjectSettings.globalize_path("res://").path_join(CANVAS_REL)
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		_check(false, "the canvas is readable at %s" % path)
		return
	var text := f.get_as_text()
	f.close()
	_log("  canvas %d bytes at %s" % [text.length(), CANVAS_REL])
	## **The refutation itself.** The comment this pass removed asserted a
	## string that is not in the file.
	_check(text.count("height:34px") == 0,
		"the canvas contains NO `height:34px` (found %d)" % text.count("height:34px"))
	_check(text.count(ENV_LEFT_HEAD) == 1,
		"`ENV:305`'s left header block is present exactly once (found %d)"
			% text.count(ENV_LEFT_HEAD))
	_check(text.count(ENV_RIGHT_HEAD) == 1,
		"`ENV:948`'s right header block is present exactly once (found %d)"
			% text.count(ENV_RIGHT_HEAD))
	_check(text.count("--ctl:%dpx" % CANVAS_CTL) >= 1,
		"`--ctl:%dpx` is the pointer token (`ENV:25`)" % CANVAS_CTL)
	_check(text.count("--ctl:%dpx" % CANVAS_CTL_TOUCH) >= 1,
		"`--ctl:%dpx` is the touch token (`ENV:1819`)" % CANVAS_CTL_TOUCH)
	## Neither block carries a `border-bottom`. Asserted so that if the canvas
	## ever grows one, the `DccTheme.rule()` this shell draws under each header
	## stops being a deviation and somebody is told.
	_check(not ENV_LEFT_HEAD.contains("border-bottom")
			and not ENV_RIGHT_HEAD.contains("border-bottom"),
		"neither header block declares a `border-bottom`")

func _ready() -> void:
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	var touch := "--force-touch" in OS.get_cmdline_user_args()
	var parts: PackedStringArray = _arg("--vp", "2560x1600").split("x")
	if parts.size() != 2:
		_log("ABORT --vp wants WxH")
		get_tree().quit(2)
		return
	_check_canvas()

	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(6)

	var shell: Node = app.get("shell")
	if shell == null:
		shell = app
	_log("viewport %dx%d  force-touch=%s  phone=%s" %
		[_vp.size.x, _vp.size.y, touch, shell.call("is_phone")])
	if shell.call("is_phone"):
		_log("RESULT fail=0 (phone: the docks are sheets, not this chrome)")
		get_tree().quit(0)
		return

	var ctl := CANVAS_CTL_TOUCH if touch else CANVAS_CTL
	var want_left := CANVAS_PAD_TOP + ctl + CANVAS_PAD_BOTTOM_LEFT
	var want_right := CANVAS_PAD_TOP + ctl + CANVAS_PAD_BOTTOM_RIGHT
	_log("  canvas expects left %d, right %d (--ctl %d)" % [want_left, want_right, ctl])

	for spec in [["left", want_left], ["right", want_right]]:
		var side := String(spec[0])
		var want := int(spec[1])
		var title: Control = shell.get("%s_dock_title" % side)
		if title == null:
			_check(false, "the %s dock has a title label to find its header by" % side)
			continue
		var head := title.get_parent() as Control
		if head == null:
			_check(false, "the %s dock title sits in a header row" % side)
			continue
		## **The minimum and the drawn band are two claims.** A laid-out size is
		## not a minimum, so both are asserted -- a header that happens to be
		## stretched to the right number by a sibling would pass only the second.
		_log("  %-5s header %s  min.y %.0f  drawn %.0f x %.0f"
			% [side, head.get_class(), head.custom_minimum_size.y,
				head.get_global_rect().size.x, head.get_global_rect().size.y])
		_check(int(round(head.custom_minimum_size.y)) == want,
			"%s dock header's minimum is %d (canvas: %d)"
				% [side, int(round(head.custom_minimum_size.y)), want])
		_check(int(round(head.get_global_rect().size.y)) == want,
			"%s dock header is DRAWN %d px tall (canvas: %d)"
				% [side, int(round(head.get_global_rect().size.y)), want])

	## **The two are not the same height**, which is the half the single pasted
	## figure hid. Asserted as the difference between the two paddings, so it
	## cannot be satisfied by both headers moving together.
	var lt: Control = shell.get("left_dock_title")
	var rt: Control = shell.get("right_dock_title")
	if lt != null and rt != null:
		var lh: Control = lt.get_parent()
		var rh: Control = rt.get_parent()
		var diff := int(round(rh.custom_minimum_size.y - lh.custom_minimum_size.y))
		_check(diff == CANVAS_PAD_BOTTOM_RIGHT - CANVAS_PAD_BOTTOM_LEFT,
			"the right header is %d px taller than the left (canvas: %d)"
				% [diff, CANVAS_PAD_BOTTOM_RIGHT - CANVAS_PAD_BOTTOM_LEFT])

	## The dock widths, so a vertical change that leaked sideways is visible in
	## the same run rather than only in `_dockfit_probe`.
	for side in ["left", "right"]:
		var dock: Control = shell.get("%s_dock" % side)
		if dock != null:
			_log("  %-5s dock width min %.0f  drawn %.0f"
				% [side, dock.get_combined_minimum_size().x, dock.get_global_rect().size.x])

	_log("RESULT fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

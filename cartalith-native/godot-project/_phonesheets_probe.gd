extends Node
## **The phone left/right dock sheet, scrolled by a real touch drag rather
## than a programmatic write, across close/reopen.**
##
## `OUTSTANDING_WORK.md`'s row ("retains its scroll offset ... will not scroll
## back up") was last checked with `_sheetscroll_probe.gd`, which (a) drives
## `--resolution` instead of the `--vp` SubViewport convention every other
## phone probe uses -- so it never actually reaches phone mode (`is_phone()`
## reads false in its own output) -- and (b) sets `scroll_vertical` directly
## rather than dragging, so it never exercises the touch path the "will not
## scroll back up" half of the row is about. This probe fixes both: it boots
## a real phone shell via `--vp`/`--force-touch`, and every scroll assertion
## below is read live off the `ScrollContainer` after a pushed touch drag, not
## after an assignment.
##
##   godot4 --path . _phonesheets_probe.tscn -- --vp 1080x2340 --force-touch
##
## Flags this probe actually reads:
##   `--vp WxH`      SubViewport size in physical px. Default 1080x2340.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it out of
##                   `OS.get_cmdline_user_args()`; without it the shell boots
##                   desktop and every check below is void.
## Any other `--flag` aborts rather than being silently ignored.

var app: Node
var _vp: SubViewport
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[phonesheets] %s" % s)

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _shot(name: String) -> void:
	var dir := "C:/Users/Vincent/AppData/Local/Temp/claude/C--Users-Vincent-Cartalith-GDT/00ffe296-8aa1-4e76-b7ec-e81351b1a7b7/scratchpad/phonefix"
	DirAccess.make_dir_recursive_absolute(dir)
	var img := _vp.get_texture().get_image()
	img.save_png("%s/%s.png" % [dir, name])

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

# -- Touch drag, pushed through the SubViewport's own hit-test ----------------

## A press, `steps` motion samples toward `to`, then a release -- a real
## finger drag pushed as mouse-emulated touch, the same pipeline
## `_rangeswipe_probe.gd`'s `_push_path()` drives every `ScrollContainer`
## gesture through elsewhere in this project (`Input.set_emulate_touch_from_mouse`
## below turns the mouse events it emits into the touch events the widgets
## actually listen for).
func _drag(from: Vector2, to: Vector2, steps: int = 12) -> void:
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.pressed = true
	down.position = from
	down.global_position = from
	_vp.push_input(down, true)
	await _frames(1)
	var prev := from
	for i in range(1, steps + 1):
		var t: float = float(i) / float(steps)
		var at: Vector2 = from.lerp(to, t)
		var mm := InputEventMouseMotion.new()
		mm.position = at
		mm.global_position = at
		mm.relative = at - prev
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(mm, true)
		prev = at
		await _frames(1)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = prev
	up.global_position = prev
	_vp.push_input(up, true)
	await _frames(4)

func _find_scroll(n: Node) -> ScrollContainer:
	if n is ScrollContainer:
		return n
	for c in n.get_children():
		var s := _find_scroll(c)
		if s != null:
			return s
	return null

## Waits for `max_value` to hold steady across five consecutive frames before
## trusting it, rather than fighting fonts/deferred rebuilds still settling
## right after the sheet opens.
func _settled_max(scroll: ScrollContainer) -> float:
	var stable := 0
	var last := -1.0
	var settle_frames := 0
	while stable < 5 and settle_frames < 60:
		await get_tree().process_frame
		var mv: float = scroll.get_v_scroll_bar().max_value
		stable = (stable + 1) if mv == last else 0
		last = mv
		settle_frames += 1
	return scroll.get_v_scroll_bar().max_value

## Full check for one dock sheet: open, drag-scroll down (live), close,
## reopen, assert the offset reads 0 live, then drag-scroll down and back up
## again inside the now-open sheet and assert it reaches 0 by touch alone.
func _check_side(side: String, dock: Control, body: Control, drag_x: float) -> void:
	app._set_sheet_open(side, true)
	await _frames(10)
	var scroll := _find_scroll(dock)
	if scroll == null:
		_check(false, "%s: no ScrollContainer found under dock" % side)
		return

	## Guaranteed overflow regardless of whatever real content this dock holds
	## right now (no world generated => a short right dock) -- a filler
	## control decouples this probe from panel content.
	var filler := Control.new()
	filler.custom_minimum_size = Vector2(0, 4000)
	body.add_child(filler)
	await _frames(3)
	var max_v: float = await _settled_max(scroll)
	_check(max_v > 0.0, "%s: content overflows (max_value=%d)" % [side, int(max_v)])

	var r: Rect2 = scroll.get_global_rect()
	var mid_y: float = r.position.y + r.size.y * 0.5

	## Drag UP the screen (content moves up, scroll position increases).
	await _drag(Vector2(drag_x, r.position.y + r.size.y - 20.0), Vector2(drag_x, r.position.y + 10.0))
	await _drag(Vector2(drag_x, r.position.y + r.size.y - 20.0), Vector2(drag_x, r.position.y + 10.0))
	await _drag(Vector2(drag_x, r.position.y + r.size.y - 20.0), Vector2(drag_x, r.position.y + 10.0))
	var after_drag: int = scroll.scroll_vertical
	_check(after_drag > 0, "%s: touch drag scrolled it down (scroll_vertical=%d)" % [side, after_drag])
	await _shot("%s_scrolled_before_close" % side)

	app._set_sheet_open(side, false)
	await _frames(3)
	app._set_sheet_open(side, true)
	await _frames(6)
	var reopened: int = scroll.scroll_vertical
	_check(reopened == 0, "%s: reopens at top (scroll_vertical=%d, expect 0)" % [side, reopened])
	await _shot("%s_reopened" % side)

	## Now, still open: drag it back down, then drag it back UP by touch and
	## confirm it actually reaches 0 -- the "will not scroll back up" half of
	## the row, tested as a gesture rather than an assignment.
	await _drag(Vector2(drag_x, r.position.y + r.size.y - 20.0), Vector2(drag_x, r.position.y + 10.0))
	await _drag(Vector2(drag_x, r.position.y + r.size.y - 20.0), Vector2(drag_x, r.position.y + 10.0))
	var mid: int = scroll.scroll_vertical
	_check(mid > 0, "%s: second drag-down moved it off the top (scroll_vertical=%d)" % [side, mid])

	for i in 8:
		await _drag(Vector2(drag_x, r.position.y + 10.0), Vector2(drag_x, r.position.y + r.size.y - 20.0))
	var back: int = scroll.scroll_vertical
	_check(back == 0, "%s: touch drag can scroll it back to 0 (scroll_vertical=%d)" % [side, back])
	await _shot("%s_scrolled_back_to_top" % side)

	app._set_sheet_open(side, false)
	await _frames(2)

func _ready() -> void:
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1080x2340").split("x")
	if parts.size() != 2:
		_log("ABORT --vp wants WxH")
		get_tree().quit(2)
		return
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.2).timeout
	if "open_project_dialog" in app and app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(2)

	_check(app.is_phone(), "shell booted in phone mode")

	await _check_side("left", app.left_dock, app.left_dock_body, float(int(parts[0])) * 0.5)
	await _check_side("right", app.right_dock, app.right_dock_body, float(int(parts[0])) * 0.5)

	_log("=== %d failure(s) ===" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

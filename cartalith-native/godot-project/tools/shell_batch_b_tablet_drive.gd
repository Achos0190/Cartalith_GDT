extends RefCounted
## Shell batch B (2026-11-23), row "Tablet canvas: the one remaining difference after the 2026-11-15 re-read; the radii and `scrPicker` not
## re-checked" (`TABLET_UI_SPEC.md`). Tablet only. Prints the corner radii the shell really draws (the number the canvas's `--rCtl`
## 12 and `--rPan` 16 are compared with) and captures the tablet's first screen (`scrPicker`'s counterpart: the welcome card),
## a menu popup and the New World dialog, for the owner and for the comparison in `TABLET_UI_SPEC.md`. Measurements, not assertions,
## except the one literal that was already pinned: `ROLE["btn_radius"]` is 12 on a tablet (the canvas's `--rCtl`).
## Run: `python C:/cw/shellB_run.py tablet res://tools/shell_batch_b_tablet_drive.gd <out> SHELLB_SHOTS=<dir>`.

var _fails := 0
var _app: Node


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _shot(ctx, label: String) -> void:
	var dir := OS.get_environment("SHELLB_SHOTS")
	var img: Image = await ctx.frame(4096)
	if img != null and dir != "":
		DirAccess.make_dir_recursive_absolute(dir)
		img.save_png("%s/%s.png" % [dir, label])


func _radius(sb: StyleBox) -> String:
	var f := sb as StyleBoxFlat
	return "%d/%d/%d/%d" % [f.corner_radius_top_left, f.corner_radius_top_right, f.corner_radius_bottom_right, f.corner_radius_bottom_left] if f != null else "not flat (%s)" % str(sb)


func run(ctx) -> void:
	_app = ctx.get_scene_root()
	var size: Vector2 = _app.get_viewport().get_visible_rect().size
	ctx.log("form: canvas %s, tablet=%s" % [str(size), str(DccTheme.is_tablet())])
	_check(ctx, "the tablet's button radius is the canvas's --rCtl (12)", DccTheme.role_px("btn_radius") == 12, str(DccTheme.role_px("btn_radius")))
	await ctx.wait(1.5)
	await _shot(ctx, "tablet_welcome")
	for e in DriveCtx.dump_ui(ctx, "Continue without a world"):
		var r: Array = e["rect"]
		await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))
		break
	await ctx.wait(1.0)
	var popup_box: StyleBox = null
	await DriveCtx.menu_open(ctx, "Edit")
	for n in _app.find_children("*", "PopupMenu", true, false):
		if (n as PopupMenu).visible:
			popup_box = (n as PopupMenu).get_theme_stylebox("panel")
	ctx.log("   radii: menu popup panel %s; dock/panel via DccTheme.panel('panel') %s; button via DccTheme.pill %s" % [
		_radius(popup_box) if popup_box != null else "no popup", _radius(DccTheme.panel("panel")), _radius(DccTheme.pill(false, DccTheme.role_px("btn_radius"), 4, 4))])
	await _shot(ctx, "tablet_edit_menu")
	Input.parse_input_event(_esc())
	await ctx.wait(0.5)
	await DriveCtx.menu_open(ctx, "File", "New world")
	await ctx.wait(1.2)
	await _shot(ctx, "tablet_new_world")
	ctx.log("shell batch B tablet drive: %d failure(s)" % _fails)


func _esc() -> InputEventKey:
	var k := InputEventKey.new()
	k.keycode = KEY_ESCAPE
	k.pressed = true
	return k

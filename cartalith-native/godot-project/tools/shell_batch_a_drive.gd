extends RefCounted
## Shell batch A (2026-11-23): the scripted session that checks the `OUTSTANDING_WORK.md` "Desktop, tablet and phone shell" rows
## fixed in that batch, against the REAL shell on each form, with real input events.
## Run: `python C:/cw/shellA_drive.py desktop|tabL|tabP|phone res://tools/shell_batch_a_drive.gd <out>` (a thin wrapper over
## `tools/fennara_drive.py::drive` that points `CARTALITH_CONFIG_PATH`, APPDATA and `SHELLA_SHOTS` at scratch).
## Each section prints `ok` or `FAIL`; a `FAIL` fails the run. Literal expectations only. Every check that proves a fix has a
## named literal mutant in `tools/shell_batch_a_mutants.py`, which must turn exactly its own check red.
##
## What it checks, by row:
##   S  settings seam: `DccSettings.CONFIG_PATH` follows `CARTALITH_CONFIG_PATH`, and a write lands there, not in `user://`.
##   W  welcome (desktop/tablet): "Continue without a world" fills its column, not just its text.
##   N  New World: Imperial-seat tier / Biome K / Lake water balance are reachable (phone: on the form; desktop/tablet: under
##      More options, which is hidden until opened -- the positive control that the phone check means something).
##   C  Create over unsaved edits asks first (Cancel generates nothing).
##   P  phone: a new world clears the overflow's Save time (`_phone_saved_at`).
##   L  CIVIL > Landmarks: each type row's show box is the same switch as the map's per-kind visibility.
##   M  memory: the Working set menu row and the resource overlay read the whole process, not Godot's allocator.

var _fails := 0
var _shots := ""
var _app: Node


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _find(ctx, t: String, cls: String = "") -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
		if cls == "" or e["class"] == cls:
			return e
	return {}


func _wait_for(ctx, t: String, secs: float = 20.0, cls: String = "") -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		var e := _find(ctx, t, cls)
		if not e.is_empty():
			return e
		await ctx.wait(0.25)
	return {}


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


func _shot(ctx, label: String) -> void:
	var img: Image = await ctx.frame(4096)
	if img != null and _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
		img.save_png("%s/%s.png" % [_shots, label])


## Scroll the nearest ScrollContainer ancestor so `c` is on screen (layout only; the click that follows is a real one).
func _reveal(ctx, c: Control) -> void:
	var sc := c.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(c)
	await ctx.wait(0.3)


## The CheckBox of the `DccWidgets.toggle` row whose label text starts with `t` (the label is a sibling Label, not the box's
## own text), searched inside the New World dialog only, or null.
func _check_box(t: String) -> Control:
	for n in _app.new_world_dialog.find_children("*", "Label", true, false):
		if (n as Label).text.begins_with(t) and n.get_parent() != null:
			for sib in n.get_parent().get_children():
				if sib is CheckBox:
					return sib as Control
	return null


func _small_world(new_world: bool) -> void:
	_app.bridge.generate({"seed": 4242, "width_km": 400.0, "grid_w": 256, "grid_h": 160,
		"archetype": "", "villages": true, "sea_level": 0.45}, new_world)
	while _app.bridge.generating:
		await _app.get_tree().create_timer(0.25).timeout
	await _app.get_tree().create_timer(0.5).timeout


func run(ctx) -> void:
	_app = ctx.get_scene_root()
	_shots = OS.get_environment("SHELLA_SHOTS")
	var size: Vector2 = _app.get_viewport().get_visible_rect().size
	var phone := size.x < 700.0
	var tabp := DccTheme.is_tablet() and size.y > size.x
	var form := "phone" if phone else ("tabP" if tabp else ("tabL" if DccTheme.is_tablet() else "desktop"))
	ctx.log("form: %s canvas %s dark=%s" % [form, str(size), str(DccTheme.is_dark())])

	# S. the settings seam
	var want := OS.get_environment("CARTALITH_CONFIG_PATH")
	_check(ctx, "S the env var reached the process", want != "" and want.ends_with("cfg.cfg"), want)
	_check(ctx, "S CONFIG_PATH is the env path, not user://", DccSettings.CONFIG_PATH == want, DccSettings.CONFIG_PATH)
	DccSettings.remember_project(want.get_base_dir() + "/seam_probe_world.ctl")
	_check(ctx, "S a settings write lands at the seam path", FileAccess.file_exists(want))
	_check(ctx, "S and never in the shared user:// file", not FileAccess.file_exists("user://cartalith_settings.cfg"))

	# W. the welcome card's opt-out fills its column (desktop and tablet; the phone shows its worlds list instead)
	if not phone:
		var cont := await _wait_for(ctx, "Continue without a world")
		_check(ctx, "W welcome card shows Continue", not cont.is_empty())
		if not cont.is_empty():
			var btn := ctx.node(cont["path"]) as Control
			var col := btn.get_parent() as Control
			_check(ctx, "W Continue fills its column (>= 90 % of it)", btn.size.x >= 0.9 * col.size.x,
				"button %.0f of column %.0f" % [btn.size.x, col.size.x])
			await _shot(ctx, "welcome_%s" % form)
			await _click_entry(ctx, cont)
			await ctx.wait(0.5)

	# N. New World: the Generation toggles
	if phone:
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		await _click_entry(ctx, nw)
		await ctx.wait(1.0)
	else:
		await DriveCtx.menu_open(ctx, "File", "New world")
		await ctx.wait(1.0)
	var imperial_box := _check_box("Imperial-seat tier")
	if phone:
		_check(ctx, "N phone: Imperial-seat tier is on the New World form", imperial_box != null and imperial_box.is_visible_in_tree())
		var bk := _check_box("Biome carrying-capacity")
		var lk := _check_box("Lake water balance")
		_check(ctx, "N phone: Biome carrying-capacity residual is on the form", bk != null and bk.is_visible_in_tree())
		_check(ctx, "N phone: Lake water balance is on the form", lk != null and lk.is_visible_in_tree())
		if imperial_box != null:
			await _reveal(ctx, imperial_box)
	else:
		_check(ctx, "N control: Imperial-seat tier is folded away until More options opens", imperial_box == null or not imperial_box.is_visible_in_tree())
		# the form scrolls: bring the toggle on screen (layout, not a handler), then click it for real
		var more_btn := _app.new_world_dialog.find_child("nw_more", true, false) as Control
		await _reveal(ctx, more_btn)
		await DriveCtx.click(ctx, DriveCtx.rect_of(more_btn).get_center())
		await ctx.wait(0.5)
		imperial_box = _check_box("Imperial-seat tier")
		_check(ctx, "N desktop/tablet: Imperial-seat tier shows under More options", imperial_box != null and imperial_box.is_visible_in_tree())
		if imperial_box != null:
			await _reveal(ctx, imperial_box)
	await _shot(ctx, "newworld_%s" % form)
	if not DccTheme.is_dark():
		# light palette (the run is started with a stored light theme): every number box must be a light field with dark ink
		var n_spin := 0
		var bad := ""
		for sbx in _app.new_world_dialog.find_children("*", "SpinBox", true, false):
			var le := (sbx as SpinBox).get_line_edit()
			var sbox := le.get_theme_stylebox("normal") as StyleBoxFlat
			var fill: Color = sbox.bg_color if sbox != null else Color(0, 0, 0, 1)
			var ink: Color = le.get_theme_color("font_color")
			n_spin += 1
			if fill.get_luminance() < 0.6 or ink.get_luminance() > 0.3:
				bad += " [%s fill %.2f ink %.2f]" % [le.text, fill.get_luminance(), ink.get_luminance()]
		_check(ctx, "T light palette: every New World number box is a light field with dark ink (%d boxes)" % n_spin, n_spin >= 5 and bad == "", bad)
	# leave the dialog: Cancel
	var cancel := _find(ctx, "Cancel", "Button")
	if cancel.is_empty():
		cancel = _find(ctx, "CANCEL", "Button")
	if not cancel.is_empty():
		await _click_entry(ctx, cancel)
		await ctx.wait(0.5)

	# C / P. a world with unsaved work, then Create over it
	await _small_world(false)
	_check(ctx, "setup: a small world exists", _app.bridge.has_world)
	_app.bridge.mark_world_dirty()
	if phone:
		_app._phone_saved_at = "12:34"
	var seed_before: int = int(_app.bridge.world_gen.get_seed())
	await DriveCtx.menu_open(ctx, "File", "New world")
	await ctx.wait(1.0)
	var create_btn: Control = _app.new_world_dialog._phone_create if phone else _app.new_world_dialog.get_ok_button()
	_check(ctx, "C the New World form offers Create", create_btn != null and create_btn.is_visible_in_tree())
	await DriveCtx.click(ctx, DriveCtx.rect_of(create_btn).get_center())
	await ctx.wait(1.0)
	var prompt := _find(ctx, "This world has unsaved changes")
	_check(ctx, "C Create over unsaved edits asks first", not prompt.is_empty())
	_check(ctx, "C nothing generated while it asks", not _app.bridge.generating)
	await _shot(ctx, "create_unsaved_%s" % form)
	var cn := _find(ctx, "Cancel", "Button")
	if not cn.is_empty():
		await _click_entry(ctx, cn)
		await ctx.wait(0.7)
	_check(ctx, "C Cancel generated nothing and kept the world", not _app.bridge.generating and int(_app.bridge.world_gen.get_seed()) == seed_before)

	# P. the phone's Save time belongs to the world it was saved from
	if phone:
		await _small_world(true)
		_check(ctx, "P a new world clears the overflow's Save time", _app._phone_saved_at == "", _app._phone_saved_at)

	# M. memory readouts read the whole process
	var rss_mb: float = _app.bridge.memory_bytes() / 1048576.0
	var static_mb: float = OS.get_static_memory_usage() / 1048576.0
	ctx.log("   memory: process %.0f MB, godot allocator %.0f MB" % [rss_mb, static_mb])
	_check(ctx, "M control: the two sources differ (process > 1.5 x allocator)", rss_mb > 1.5 * static_mb)
	_app.resource_overlay.visible = true
	await ctx.wait(1.0)
	var ov: String = _app.resource_overlay._label.text
	var got := -1.0
	for ln in ov.split("\n"):
		if ln.begins_with("Working set: "):
			got = float(ln.trim_prefix("Working set: ").trim_suffix(" MB"))
	_check(ctx, "M the resource overlay's Working set is the process figure", got > 0.0 and absf(got - rss_mb) < 0.15 * rss_mb,
		"overlay %.1f MB vs process %.1f MB" % [got, rss_mb])
	_app.resource_overlay.visible = false
	var row_text := ""
	for n in _app.get_tree().root.find_children("*", "PopupMenu", true, false):
		var p := n as PopupMenu
		for i in p.item_count:
			if p.get_item_text(i).begins_with("Working set"):
				p.about_to_popup.emit()
				row_text = p.get_item_text(i)
	_check(ctx, "M the Working set menu row is the process figure", _row_mb(row_text) > 0.0 and absf(_row_mb(row_text) - rss_mb) < 0.15 * rss_mb + 1.0,
		"'%s' vs process %.0f MB" % [row_text, rss_mb])

	# D. a dock layer row's name ellipsises (full text as tooltip) instead of being cut hard. Built through the dock's own row
	# builder into a 160 px column so the name cannot fit.
	var col := VBoxContainer.new()
	col.custom_minimum_size.x = 160
	col.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	_app.add_child(col)
	var long_name := "Terrain relief shading with a very long layer name"
	_app.right_dock_ctrl._layer_row(col, {"id": "layer_x", "label": long_name, "visible": true, "opacity": 1.0, "blend": "normal"}, 0, 1)
	await ctx.wait(0.3)
	var nl: Label = null
	for n in col.find_children("*", "Label", true, false):
		if (n as Label).text == long_name:
			nl = n as Label
	_check(ctx, "D the layer row's name is built", nl != null)
	if nl != null:
		_check(ctx, "D it ellipsises", nl.text_overrun_behavior == TextServer.OVERRUN_TRIM_ELLIPSIS)
		_check(ctx, "D and carries its full text as the tooltip", nl.tooltip_text == long_name)
	col.queue_free()

	# L. CIVIL > Landmarks rows
	if not phone:
		_app.select_domain_category("civilization", "Landmarks")
		await ctx.wait(1.0)
		var box: CheckBox = null
		for n in _app.get_tree().root.find_children("LmShow_*", "CheckBox", true, false):
			if (n as Control).is_visible_in_tree():
				box = n as CheckBox
				break
		_check(ctx, "L the Landmarks rows carry a show box", box != null)
		if box != null:
			await _reveal(ctx, box)
			var key := String(box.name).trim_prefix("LmShow_")
			var vp = _app.viewport
			var was: bool = vp.landmark_kind_visible(key)
			_check(ctx, "L the box starts equal to the map's per-kind switch", box.button_pressed == was)
			await DriveCtx.click(ctx, DriveCtx.rect_of(box).get_center())
			await ctx.wait(0.3)
			_check(ctx, "L clicking the box flips the map's per-kind visibility", vp.landmark_kind_visible(key) == (not was))
			vp.set_landmark_kind_visible(key, was)
			await ctx.wait(0.3)
			_check(ctx, "L and the map's switch moves the box back", box.button_pressed == was)
			await _shot(ctx, "landmarks_%s" % form)
	ctx.log("RESULT %s: %d failed" % [form, _fails])
	if _fails > 0:
		ctx.error("FAIL %d checks" % _fails)


## `Working set   1.6 GB of 12 GB` or `Working set   412 MB` -> megabytes of the first figure, or -1.
func _row_mb(t: String) -> float:
	var parts := t.trim_prefix("Working set").strip_edges().split(" ")
	if parts.size() < 2:
		return -1.0
	var v := float(parts[0])
	return v * 1024.0 if String(parts[1]) == "GB" else v

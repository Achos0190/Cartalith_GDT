extends RefCounted
## Shell batch A (2026-11-23), tablet portrait rows. Run on `tabP` (a boot in portrait) and on `tabL` (a boot in landscape that is then
## rotated by resizing the window, which `MS-1` classifies live). Python wrapper: `C:/cw/shellA_drive.py` (see
## `shell_batch_a_drive.gd`). Literal expectations; mutants in `tools/shell_batch_a_mutants.py`.
##
##   (The Climate lapse-rate readout was measured inside the dock on 2026-11-23, row not reproduced; its guard survived both
##   mutants tried, so it is not kept as a check.)
##   T  every `DccWidgets.toggle` row's box ends at the row's right edge (the label carries the slack), whichever way the row was
##      built and whatever the window has been rotated to since (the spacer that was built once and never re-run is gone).

var _fails := 0
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


func _shot(ctx, label: String) -> void:
	var img: Image = await ctx.frame(4096)
	var d := OS.get_environment("SHELLA_SHOTS")
	if img != null and d != "":
		DirAccess.make_dir_recursive_absolute(d)
		img.save_png("%s/%s.png" % [d, label])


## The lapse-rate readout Label (text ends "°C/km"), or null.
func _lapse() -> Label:
	for n in _app.left_dock.find_children("*", "Label", true, false):
		if (n as Label).text.ends_with("°C/km") and (n as Label).is_visible_in_tree():
			return n as Label
	return null


## How many toggle boxes in the left dock sit more than 3 px short of their row's right edge, and how many were measured.
func _toggle_edges() -> Array:
	var short := 0
	var seen := 0
	for n in _app.left_dock.find_children("*", "CheckBox", true, false):
		var cb := n as CheckBox
		if not cb.is_visible_in_tree() or cb.text != "":
			continue
		var row := cb.get_parent() as Control
		if not (row is HBoxContainer) or row.size.x < 120.0:
			continue
		seen += 1
		var gap: float = (row.get_global_rect().end.x - cb.get_global_rect().end.x)
		# a toggle row is label + box and nothing else: a spacer left from a build in the other orientation is a third child
		if gap > 3.0 or row.get_child_count() != 2:
			short += 1
	return [short, seen]


func _climate(ctx) -> void:
	_app.select_domain_category("world", "Climate")
	await ctx.wait(1.0)


func run(ctx) -> void:
	_app = ctx.get_scene_root()
	var size: Vector2 = _app.get_viewport().get_visible_rect().size
	ctx.log("boot canvas %s dark=%s portrait=%s" % [str(size), str(DccTheme.is_dark()), str(DccTheme.is_tablet_portrait())])
	var cont := _find(ctx, "Continue without a world")
	if not cont.is_empty():
		var r: Array = cont["rect"]
		await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))
		await ctx.wait(0.5)
	_app.bridge.generate({"seed": 4242, "width_km": 400.0, "grid_w": 256, "grid_h": 160,
		"archetype": "", "villages": false, "sea_level": 0.45}, false)
	while _app.bridge.generating:
		await _app.get_tree().create_timer(0.25).timeout
	await _climate(ctx)
	var booted_portrait := DccTheme.is_tablet_portrait()
	if not booted_portrait:
		# the rotation leg: the rows exist, built in landscape; now turn the tablet
		var before := _toggle_edges()
		_check(ctx, "T landscape: toggle boxes end at the row edge (measured %d)" % before[1], before[1] > 0 and before[0] == 0)
		DisplayServer.window_set_size(Vector2i(1000, 1031))
		await ctx.wait(2.0)
		_check(ctx, "rotated: the shell now classifies as tablet portrait", DccTheme.is_tablet_portrait())
		await _climate(ctx)
	var after := _toggle_edges()
	_check(ctx, "T portrait: toggle boxes end at the row edge (measured %d)" % after[1], after[1] > 0 and after[0] == 0,
		"%d short of %d" % [after[0], after[1]])
	await _shot(ctx, "climate_%s" % ("tabP" if booted_portrait else "tabL_rotated"))
	ctx.log("RESULT: %d failed" % _fails)
	if _fails > 0:
		ctx.error("FAIL %d checks" % _fails)

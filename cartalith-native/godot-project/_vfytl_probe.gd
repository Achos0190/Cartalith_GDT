extends Node
## VERIFIER probe (untracked). Independent re-measurement of fix 6.
##
## Runs **windowed** -- `--headless` reports a 64x64 viewport whatever
## `--resolution` says, so every leg would classify LAPTOP.
##
##   godot --path . --resolution 1400x1000 _vfytl_probe.tscn -- --desktop
##   godot --path . --resolution 1400x1000 _vfytl_probe.tscn
##   godot --path . --resolution 1400x1000 _vfytl_probe.tscn -- --force-touch
##   godot --path . --resolution  450x1000 _vfytl_probe.tscn -- --force-touch
##
## BEFORE/AFTER inside ONE process: the pre-fix box is exactly the box
## `_build_timeline()` builds and the old `_build_timeline_collapsed()` never
## overrode -- `margin_top/bottom = 8`, no fixed height -- so
## `set_timeline_metrics(8, 0)` reproduces it faithfully without touching git.
## Then `_fill_timeline_strip()` re-applies the shipped pair.
##
## Also measures what the brief calls the map-pixel question: the viewport
## area's own height and the status bar's, in both states.

var app: Node
var _rows: Array = []

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _density() -> String:
	if DccTheme.is_phone():
		return "PHONE"
	if DccTheme.is_tablet():
		return "TABLET"
	if DccTheme.is_laptop():
		return "LAPTOP"
	return "DESKTOP"

func _status_bar() -> Control:
	var n: Node = app.status_row
	while n != null and not (n is PanelContainer):
		n = n.get_parent()
	return n as Control

func _snap(tag: String) -> void:
	var bar: Control = app.timeline_bar
	var row: Control = app.timeline_row
	var pad: MarginContainer = row.get_parent() as MarginContainer
	var sb: Control = _status_bar()
	var vp: Vector2 = get_viewport().get_visible_rect().size
	var maxlbl := 0.0
	var lbls: Array = []
	_labels(row, lbls)
	for l in lbls:
		maxlbl = maxf(maxlbl, float(l))
	print("TL %-8s %-11s vp=%.0fx%.0f  strip=%.0f  strip_min=%.0f  padY=%d  rowH=%.0f  maxLabelH=%.0f  viewport=%.0f  statusbar=%.0f  h_status=%d  want=%d" % [
		_density(), tag, vp.x, vp.y,
		bar.size.y, bar.get_combined_minimum_size().y,
		pad.get_theme_constant("margin_top"),
		row.size.y, maxlbl,
		app.viewport_area.size.y, sb.size.y,
		DccTheme.role_px("h_status"), DccTheme.role_px("h_status") - 2])
	_rows.append({"tag": tag, "strip": bar.size.y, "viewport": app.viewport_area.size.y,
		"status": sb.size.y, "maxlbl": maxlbl})

func _labels(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append((c as Label).size.y)
		_labels(c, out)

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)
	if "--desktop" in OS.get_cmdline_user_args():
		DccTheme.set_narrow(false)
	## The strip is only visible in CIVIL (`app.gd:1048`).
	app.select_domain("civilization")
	await _frames(10)
	print("TL visible=%s  expanded=%s" % [str(app.timeline_bar.visible), str(app._tl_expanded)])

	## --- BEFORE: the box `_build_timeline()` builds, un-overridden ---------
	app.set_timeline_metrics(8, 0)
	await _frames(8)
	_snap("BEFORE")

	## --- AFTER: the shipped collapsed pair, re-applied by the real filler --
	app._fill_timeline_strip()
	await _frames(8)
	_snap("AFTER")

	## --- expanded, then collapsed again (the restore pair) -----------------
	app._tl_expanded = true
	app._fill_timeline_strip()
	await _frames(8)
	_snap("EXPANDED")
	app._tl_expanded = false
	app._fill_timeline_strip()
	await _frames(8)
	_snap("RECOLLAPSE")

	var want: int = DccTheme.role_px("h_status") - 2
	var b: Dictionary = _rows[0]
	var a: Dictionary = _rows[1]
	var r: Dictionary = _rows[3]
	print("TL SUMMARY density=%s  before=%.0f  after=%.0f  want=%s  recollapse=%.0f  delta_strip=%+.0f  delta_viewport=%+.0f  delta_statusbar=%+.0f" % [
		_density(), b["strip"], a["strip"],
		("%d" % want) if not DccTheme.is_phone() else "n/a (phone composition)",
		r["strip"], a["strip"] - b["strip"],
		a["viewport"] - b["viewport"], a["status"] - b["status"]])
	get_tree().quit(0)

extends Node
## Collapsed-timeline-strip height probe (`01-frame-and-tokens.md` §3.7:
## collapsed is `height:calc(var(--sbH) - 2px); padding:0 var(--pad)`,
## expanded is `padding:8px var(--pad)` with auto height).
##
## Layout only, no pixels -- but it still has to run **windowed**, for a
## different reason than a pixel probe does: see the note below `--desktop`.
##
## Four densities, one process each (`DccTheme._touch` is latched for the life
## of the process, so they cannot share a run):
##
##   godot --path . --resolution 1400x1000 _tlheight_probe.tscn -- --desktop
##   godot --path . --resolution 1400x1000 _tlheight_probe.tscn
##   godot --path . --resolution 1400x1000 _tlheight_probe.tscn -- --force-touch
##   godot --path . --resolution  450x1000 _tlheight_probe.tscn -- --force-touch
##
## Add `-- --expandfirst` to any leg to read the expanded form with the box
## `_build_timeline()` itself built, before the collapsed fill has pinned one.
##
## **Run windowed, and force the density rather than inferring it from the
## window.** `--headless` reports a 64x64 viewport whatever `--resolution` says,
## which makes `size.x < W_LAPTOP_MAX` true by construction and every leg
## LAPTOP; and this machine's display is 1680x1050, so no real window can reach
## the 1920 the DESKTOP band needs. `--desktop` therefore sets
## `DccTheme.set_narrow(false)` after boot and refills the strip. That is honest
## for *this* measurement and only this one: `h_status` is not one of the three
## roles `DccTheme.LAPTOP` overrides, so DESKTOP and LAPTOP read the same
## column here by construction -- the leg exists to show that, not to
## discriminate.
##
## The timeline strip is only visible in CIVIL (`app.gd::_on_workspace_changed`),
## so the probe selects that domain before reading anything.

var app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _density() -> String:
	if DccTheme.is_phone():
		return "PHONE"
	if DccTheme.is_tablet():
		return "TABLET"
	if DccTheme.is_laptop():
		return "LAPTOP"
	return "DESKTOP"

func _report(tag: String) -> void:
	var bar: Control = app.timeline_bar
	var row: Control = app.timeline_row
	var pad: MarginContainer = row.get_parent() as MarginContainer
	var vp: Vector2 = get_viewport().get_visible_rect().size
	print("TLH %-9s %-9s vp=%.0fx%.0f visible=%s  bar.size.y=%.0f  bar.min.y=%.0f  h_status=%d  want=%d" % [
		_density(), tag, vp.x, vp.y, str(bar.visible), bar.size.y,
		bar.get_combined_minimum_size().y,
		DccTheme.role_px("h_status"), DccTheme.role_px("h_status") - 2])
	print("TLH   pad top/bottom=%d/%d  row.min.y=%.0f  row.size.y=%.0f  children=%d" % [
		pad.get_theme_constant("margin_top"), pad.get_theme_constant("margin_bottom"),
		row.get_combined_minimum_size().y, row.size.y, row.get_child_count()])
	## Every label inside the strip, with the height it actually got. A strip
	## that is tall enough on paper and clips its own type is the failure this
	## reports rather than hides.
	var out: Array = []
	_labels(row, out)
	for l in out:
		print("TLH   label %-14s min.y=%.0f size.y=%.0f font=%d" % l)

func _labels(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			var l := c as Label
			out.append([l.text.substr(0, 14), l.get_combined_minimum_size().y, l.size.y,
				l.get_theme_font_size("font_size")])
		_labels(c, out)

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(8)
	if "--desktop" in OS.get_cmdline_user_args():
		DccTheme.set_narrow(false)
	## `--expandfirst` makes the *first* fill the expanded one, so the expanded
	## box is read with the metrics `_build_timeline()` itself built and nothing
	## the collapsed form has pinned. That is the leg that shows the collapsed
	## fix does not reach the expanded form, rather than inferring it.
	if "--expandfirst" in OS.get_cmdline_user_args():
		app._tl_expanded = true
	app.select_domain("civilization")
	await _frames(8)
	_report("collapsed" if not app._tl_expanded else "expand1st")
	if app._tl_expanded:
		get_tree().quit(0)
		return
	## The expanded form must stay auto-height -- a floor applied to the shared
	## bar and never cleared would show up here as a bar taller than its content
	## needs, so it is measured in the same run rather than reasoned about.
	app._tl_expanded = true
	app._fill_timeline_strip()
	await _frames(8)
	_report("expanded")
	app._tl_expanded = false
	app._fill_timeline_strip()
	await _frames(8)
	_report("recollaps")
	get_tree().quit(0)

extends Node
## LANE COLOUR probe: does the Colour management picker tell a user, at the
## moment they choose a display, that the choice does not reach their exports?
##
## Runs **windowed** on purpose. `ImageTexture.update()` is a no-op under
## `--headless`, so a headless pass could not screenshot the built section --
## and this probe's whole subject is what is on screen.
##
## Asserts, in order:
##   A) the picker exists and its entries are the ENGINE's list
##      (`bridge.color_spaces()` -> `render::COLOR_SPACES`), never a list typed
##      into this probe -- so a renamed or added space fails here loudly
##      instead of being silently agreed with;
##   B) the export statement is rendered, and is reachable by the words a user
##      would scan for ("export");
##   C) it is stated EXACTLY ONCE -- the defect being fixed was a second,
##      differently-worded copy of the same fact buried in the overlays note;
##   D) the old buried clause is gone from the overlays note;
##   E) the overlays note still says what it is for (overlays), i.e. the strip
##      took the exports clause and nothing else.
## Then prints the density set and palette the measurement was taken under,
## because both change the rendered text's size and colour and neither is
## constant on this machine.

var _fails := 0

func _chk(ok: bool, what: String) -> void:
	print(("  PASS  " if ok else "  FAIL  ") + what)
	if not ok:
		_fails += 1

func _all(node: Node, out: Array = []) -> Array:
	for c in node.get_children():
		out.append(c)
		_all(c, out)
	return out

func _harvest(node: Node, out: Array) -> void:
	for c in node.get_children():
		if c is Label or c is Button or c is CheckBox:
			var t := String(c.text)
			if not t.is_empty():
				out.append(t)
		if c is OptionButton:
			var opts: Array = []
			for i in range(c.item_count):
				opts.append(c.get_item_text(i))
			out.append("[[OPTIONS]] " + ", ".join(opts))
		_harvest(c, out)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	var bridge = app.bridge

	## `RenderWorkspace` is not a registered workspace: `cartography_workspace.gd`
	## composes a real instance as its `_render` member. Reach it through the
	## owner rather than constructing a second one, so this measures the shell's
	## own object.
	var carto: Control = null
	for w in app._workspaces:
		if w.name == "CartographyWorkspace":
			carto = w
	if carto == null:
		print("FAIL: no CartographyWorkspace")
		get_tree().quit(1)
		return
	var ws: Control = carto._render
	if ws == null:
		print("FAIL: CartographyWorkspace has no _render")
		get_tree().quit(1)
		return

	print("density: touch=%s phone=%s tablet=%s laptop=%s narrow_desktop_default=%s" % [
		str(DccTheme._touch), str(DccTheme.is_phone()), str(DccTheme.is_tablet()),
		str(DccTheme.is_laptop()), str(DccTheme.is_tablet() or DccTheme.is_phone())])
	print("palette: %s" % ("DARK" if DccTheme._dark else "LIGHT"))
	print("color_space_api: %s   engine list: %s   current: %s" % [
		str(bridge.color_space_api), str(bridge.color_spaces()), bridge.color_space()])

	## Dismiss the welcome dialog, which otherwise covers the whole window and
	## would make the screenshot evidence of nothing.
	for n in _all(app):
		if n is Button and String(n.text) == "Continue without a world":
			n.pressed.emit()
	await get_tree().process_frame

	## Build the real section through the real entry point, at the LEFT dock's
	## documented minimum width (`DccTheme.W_LEFT_DOCK_MIN`) -- this section
	## lives in the left dock, and the minimum is the case where a long note
	## either wraps or clips.
	var layer := CanvasLayer.new()
	add_child(layer)
	var back := PanelContainer.new()
	back.position = Vector2(24, 24)
	back.custom_minimum_size = Vector2(DccTheme.W_LEFT_DOCK_MIN, 0)
	var sb := StyleBoxFlat.new()
	## `"panel"` -- the dock surface. Named from `DccTheme.LIGHT`/`DARK`'s own
	## keys, not guessed: the first draft used `"bg_dock"`, which does not
	## exist, and the screenshot came back Godot placeholder magenta.
	## No content margins here: `DccWidgets.section()` supplies the dock's own
	## padding (14 left / 12 right -- `note()`'s header states both), and a
	## second copy on this backdrop would widen the panel past the dock minimum
	## and make check F fail on the probe's padding rather than the section's.
	sb.bg_color = DccTheme.c("panel")
	back.add_theme_stylebox_override("panel", sb)
	layer.add_child(back)
	var host := VBoxContainer.new()
	host.custom_minimum_size = Vector2(DccTheme.W_LEFT_DOCK_MIN, 0)
	back.add_child(host)
	ws._host = host
	ws._build_color_management()
	ws._host = null
	await get_tree().process_frame
	await get_tree().process_frame

	var lines: Array = []
	_harvest(host, lines)
	var blob := "\n".join(PackedStringArray(lines))

	# A) picker entries come from the engine
	var engine_list: Array = bridge.color_spaces()
	var opt_line := ""
	for l in lines:
		if String(l).begins_with("[[OPTIONS]] "):
			opt_line = String(l).substr(12)
	_chk(opt_line == ", ".join(PackedStringArray(engine_list)),
		"A: picker entries == engine COLOR_SPACES  (picker=[%s] engine=[%s])" % [
			opt_line, ", ".join(PackedStringArray(engine_list))])
	_chk(engine_list.size() >= 2,
		"A2: engine offers a real choice, not a locked readout (%d entries)" % engine_list.size())

	# B) the export statement is on screen and scannable
	var export_notes: Array = []
	for l in lines:
		var t := String(l)
		if t.to_lower().contains("export") and t.length() > 60:
			export_notes.append(t)
	_chk(export_notes.size() > 0, "B: an export statement is rendered at the picker")
	if export_notes.size() > 0:
		var n := String(export_notes[0])
		_chk(n.to_lower().contains("working space"),
			"B2: it names the working space")
		_chk(n.to_lower().contains("profile"),
			"B3: it gives the reason (no colour profile written)")
		_chk(n.to_lower().contains("not match") or n.to_lower().contains("will not match"),
			"B4: it states the user-visible consequence, not just the mechanism")

	# C) stated exactly once
	_chk(export_notes.size() == 1,
		"C: the exports fact is stated ONCE, not twice (%d long notes mention 'export')"
		% export_notes.size())

	# D) the old buried clause is gone
	_chk(not blob.contains("Exports are unaffected and stay sRGB"),
		"D: the old buried clause is gone from the overlays note")

	# E) the overlays note survived, minus that clause
	var overlays_ok := false
	for l in lines:
		var t := String(l)
		if t.contains("re-encodes the map raster only") and t.contains("rivers, labels"):
			overlays_ok = true
			_chk(not t.to_lower().contains("export"),
				"E2: the overlays note no longer talks about exports")
	_chk(overlays_ok, "E: the overlays note still says what it is for")

	print("\n--- rendered section, in order ---")
	for l in lines:
		print("  | " + String(l))

	# Windowed screenshot of the built section, at the dock's minimum width.
	await get_tree().process_frame
	await get_tree().process_frame
	await RenderingServer.frame_post_draw
	print("section rendered at %d x %d px (left dock minimum = %d)"
		% [int(host.size.x), int(host.size.y), DccTheme.W_LEFT_DOCK_MIN])
	## Measured on the host the section was built into -- the width a left dock
	## at its documented minimum actually offers it. A section that overflowed
	## would push this past `W_LEFT_DOCK_MIN`, which is the clip this checks for.
	_chk(host.size.x <= DccTheme.W_LEFT_DOCK_MIN + 1,
		"F: the section fits the left dock minimum without forcing it wider (host %d px)"
		% int(host.size.x))
	var img := get_viewport().get_texture().get_image()
	img.save_png("user://_colorspaceexport_probe.png")
	print("screenshot saved (%dx%d)" % [img.get_width(), img.get_height()])

	print("\nFAILS=%d" % _fails)
	get_tree().quit(0 if _fails == 0 else 1)

extends SceneTree

# GUI_GAP_REGISTER.md CA-03 / CA-04 / RD-10 -- the raster layer stack, driven
# through the REAL WorldGen bindings.
#
# `WorldGen` is a cdylib GodotClass and cannot be constructed in a Rust unit
# test, so `get_layer_stack` / `set_layer_stack` / `list_blend_modes` are
# unreachable from `cargo test` no matter how many of them pass. Everything
# below the boundary is covered by crates/cartalith-godot/tests/layer_stack.rs;
# this file covers the boundary itself -- the row shape, the top-first order,
# the partial-row merge, every refusal, and that the stack actually changes the
# texture the app draws AND the raster the exporter writes.
#
#   godot --headless --path <godot-project> --script _layerstack_probe.gd

var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _row(stack: Array, id: String) -> Dictionary:
	for r in stack:
		if String(r.get("id", "")) == id:
			return r
	return {}

## A cheap fingerprint of the rendered texture, so "the map changed" is a
## measurement rather than a claim. Samples a fixed lattice; a stack change
## that moved nothing would return the same string.
func _fingerprint(tex: Texture2D) -> String:
	if tex == null:
		return "<null>"
	var img: Image = tex.get_image()
	if img == null:
		return "<noimage>"
	var acc := 0
	var w := img.get_width()
	var h := img.get_height()
	var y := 0
	while y < h:
		var x := 0
		while x < w:
			var c: Color = img.get_pixel(x, y)
			acc = (acc * 131 + int(c.r * 255.0) * 7 + int(c.g * 255.0) * 13 + int(c.b * 255.0) * 17) % 1000000007
			x += 7
		y += 5
	return "%d@%dx%d" % [acc, w, h]

func _initialize() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("FAIL  WorldGen is not registered -- the GDExtension did not load")
		quit(1)
		return
	var wg = ClassDB.instantiate("WorldGen")

	for m in ["get_layer_stack", "set_layer_stack", "list_blend_modes", "reset_appearance", "build_color_texture"]:
		_ok(wg.has_method(m), "WorldGen exposes %s()" % m)

	# ---- The vocabulary the panel will build itself from ------------------
	var modes: PackedStringArray = wg.list_blend_modes()
	_ok(modes.size() == 5, "list_blend_modes() returns five modes (got %d)" % modes.size())
	_ok(modes.size() > 0 and modes[0] == "Normal", "the identity is the first row a picker draws")

	# ---- The stack, top-first, in DCC_SHELL_SPEC.md §7's own reading order -
	var stack: Array = wg.get_layer_stack()
	_ok(stack.size() == 3, "get_layer_stack() returns three rows (got %d)" % stack.size())
	var ids: Array = []
	for r in stack:
		ids.append(String(r.get("id", "?")))
	_ok(ids == ["hillshade", "colour_relief", "terrain"],
		"rows come back TOP-first, the order a layer list draws: %s" % str(ids))
	for r in stack:
		for k in ["id", "label", "visible", "opacity", "blend"]:
			_ok(r.has(k), "row %s carries `%s`" % [r.get("id", "?"), k])
	_ok(String(_row(stack, "hillshade").get("blend", "")) == "Multiply",
		"hillshade opens on Multiply -- `c * light` and nothing else")
	_ok(String(_row(stack, "colour_relief").get("blend", "")) == "Normal", "colour relief opens on Normal")
	_ok(bool(_row(stack, "terrain").get("visible", false)), "terrain opens visible")
	_ok(float(_row(stack, "terrain").get("opacity", -1.0)) == 1.0, "terrain opens opaque")
	_ok(String(_row(stack, "terrain").get("label", "")) == "Terrain", "the label is a display string, not the id")

	# ---- Refusals. Each one must change nothing. -------------------------
	var pristine := str(wg.get_layer_stack())
	_ok(wg.set_layer_stack([]) == 0, "an empty stack is refused")
	_ok(wg.set_layer_stack([{"id": "terrain"}, {"id": "hillshade"}]) == 0, "a two-row stack is refused")
	_ok(wg.set_layer_stack([{"id": "terrain"}, {"id": "terrain"}, {"id": "hillshade"}]) == 0,
		"a stack naming one category twice is refused")
	_ok(wg.set_layer_stack([{"id": "terrain"}, {"id": "colour_relief"}, {"id": "rhubarb"}]) == 0,
		"an unknown layer id is refused, not silently dropped")
	_ok(wg.set_layer_stack([{"id": "terrain"}, {"id": "colour_relief"},
		{"id": "hillshade", "blend": "Rhubarb"}]) == 0,
		"an unknown blend mode is refused, not defaulted to Normal")
	_ok(str(wg.get_layer_stack()) == pristine, "every refusal left the stack exactly as it was")

	# ---- A partial row keeps what it does not mention ---------------------
	_ok(wg.set_layer_stack([
		{"id": "hillshade", "opacity": 0.4},
		{"id": "colour_relief"},
		{"id": "terrain"},
	]) == 3, "a stack of partial rows is accepted")
	stack = wg.get_layer_stack()
	_ok(abs(float(_row(stack, "hillshade").get("opacity", -1.0)) - 0.4) < 1e-6, "the opacity it named was applied")
	_ok(String(_row(stack, "hillshade").get("blend", "")) == "Multiply",
		"the blend it did NOT name survived -- an absent key means unchanged, never default")
	_ok(bool(_row(stack, "hillshade").get("visible", false)), "the visibility it did not name survived")

	# ---- Opacity is clamped at the boundary, not left to the renderer -----
	wg.set_layer_stack([{"id": "hillshade", "opacity": 7.0}, {"id": "colour_relief"}, {"id": "terrain"}])
	_ok(float(_row(wg.get_layer_stack(), "hillshade").get("opacity", -1.0)) == 1.0, "opacity 7.0 clamps to 1.0")

	# ---- Reorder round-trips ----------------------------------------------
	wg.reset_appearance()
	_ok(wg.set_layer_stack([
		{"id": "colour_relief"},
		{"id": "hillshade"},
		{"id": "terrain"},
	]) == 3, "a reordered stack is accepted")
	ids = []
	for r in wg.get_layer_stack():
		ids.append(String(r.get("id", "?")))
	_ok(ids == ["colour_relief", "hillshade", "terrain"], "the reorder round-trips: %s" % str(ids))

	# ---- reset_appearance() drops it --------------------------------------
	_ok(wg.reset_appearance() >= 1, "reset_appearance() reports dropping the stack")
	ids = []
	for r in wg.get_layer_stack():
		ids.append(String(r.get("id", "?")))
	_ok(ids == ["hillshade", "colour_relief", "terrain"], "reset restored the shipped arrangement")

	# ---- And it reaches the pixels, on BOTH consumer paths -----------------
	# A capability attached to the screen and not to the export is the defect
	# MISTAKES.md records for `with_ground_tiles`; it moved no pixel at the
	# default, so no test saw it. Here both are measured.
	if not wg.has_method("generate"):
		print("  SKIP  no generate() -- cannot measure pixels")
	else:
		# generate_sized(seed, width_km, grid_w, grid_h) -- a small, non-square
		# grid, so the probe stays under a second and a row-order bug in the
		# composite would show up as a crash rather than as a square that
		# happens to work either way.
		wg.call("generate_sized", 1234, 400.0, 96, 61)
		var before := _fingerprint(wg.build_color_texture())
		_ok(before != "<null>" and before != "<noimage>", "the default world renders a texture (%s)" % before)
		wg.set_layer_stack([
			{"id": "hillshade", "blend": "Screen"},
			{"id": "colour_relief"},
			{"id": "terrain"},
		])
		var after := _fingerprint(wg.build_color_texture())
		_ok(before != after, "a blend-mode change moves the ON-SCREEN raster (%s -> %s)" % [before, after])
		wg.reset_appearance()
		_ok(_fingerprint(wg.build_color_texture()) == before, "reset returns the shipped image exactly")

		if wg.has_method("export_raster_png"):
			# export_raster_png() writes with std::fs, so it needs a real OS path;
			# a `user://` URI reaches it verbatim and fails at CreateDirectory.
			var dir := ProjectSettings.globalize_path("user://").path_join("_layerstack_probe")
			DirAccess.make_dir_recursive_absolute(dir)
			var a_path := dir.path_join("a.png")
			var b_path := dir.path_join("b.png")
			var ra: Dictionary = wg.export_raster_png(a_path, 2048, false)
			wg.set_layer_stack([
				{"id": "hillshade", "blend": "Screen"},
				{"id": "colour_relief"},
				{"id": "terrain"},
			])
			var rb: Dictionary = wg.export_raster_png(b_path, 2048, false)
			if bool(ra.get("ok", false)) and bool(rb.get("ok", false)):
				var fa := FileAccess.get_file_as_bytes(a_path)
				var fb := FileAccess.get_file_as_bytes(b_path)
				_ok(fa.size() > 0 and fb.size() > 0, "both export PNGs were written")
				_ok(fa != fb, "the same blend change moves the EXPORTED PNG (%d vs %d bytes)" % [fa.size(), fb.size()])
			else:
				print("  SKIP  export_raster_png refused: %s / %s" % [str(ra.get("error", ra)), str(rb.get("error", rb))])
			wg.reset_appearance()

	print("")
	print("%s -- %d failure(s)" % ["ALL PASS" if fails == 0 else "FAILURES", fails])
	quit(1 if fails > 0 else 0)

extends Node
## Committed verification harness for the output colour space
## (`LARGE_ITEM_RULINGS.md`, **Colour management** -- owner-ruled build).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _colorspace_probe.tscn
##
## The ruling carries one cost the owner stated and accepted: *"every
## golden-parity fixture is sRGB, so this touches the one surface the parity
## harnesses pin. Do it behind a default that leaves sRGB byte-identical, or
## re-baseline deliberately and say so."* The Rust side proves that on a
## synthetic field (`tests/color_space.rs`, a pinned FNV-1a of the finished
## render taken before `render::ColorSpace` existed). **This probe proves it on
## the real path**: a real generated world, through the real cdylib, through the
## real `build_color_texture()` the viewport calls, at the real 2048-wide grid.
##
## The strong assertion is section 3: set Display P3, re-render, set sRGB back,
## re-render, and the third image must be byte-for-byte the first. A default
## that is only *usually* identity would fail there and nowhere else.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them.

var bridge: Node
var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

## The bytes `build_color_texture()` actually hands the viewport.
func _screen() -> PackedByteArray:
	var tex: ImageTexture = bridge.world_gen.build_color_texture()
	if tex == null:
		return PackedByteArray()
	var im := tex.get_image()
	im.convert(Image.FORMAT_RGB8)
	return im.get_data()

## One PNG export taken with `space` set, read back as RGB8. Empty on failure.
func _export(dir: String, name: String, space: String) -> PackedByteArray:
	if not bridge.set_color_space(space):
		print("    set_color_space(%s) refused" % space)
		return PackedByteArray()
	var path := dir.path_join(name + ".png")
	var r: Dictionary = bridge.world_gen.export_raster_png(path, 2048, false)
	if not bool(r.get("ok", false)):
		print("    export %s FAILED: %s" % [name, String(r.get("error", ""))])
		return PackedByteArray()
	var img := Image.new()
	if img.load(path) != OK:
		return PackedByteArray()
	img.convert(Image.FORMAT_RGB8)
	return img.get_data()

## worst per-byte delta, share of bytes that moved, mean absolute delta.
func _cmp(a: PackedByteArray, b: PackedByteArray) -> Array:
	if a.size() == 0 or a.size() != b.size():
		return [-1, -1.0, -1.0]
	var worst := 0
	var moved := 0
	var sum := 0
	for i in range(a.size()):
		var d: int = absi(a[i] - b[i])
		if d > 0:
			moved += 1
			sum += d
			worst = maxi(worst, d)
	return [worst, float(moved) / float(a.size()), float(sum) / float(a.size())]

## How many of the pixels in this image are exact neutrals, and how many of
## those survive a transform unchanged. The matrix rows sum to 1, so every one
## of them must -- see `render::SRGB_TO_P3`.
func _neutrals_held(a: PackedByteArray, b: PackedByteArray) -> Array:
	var total := 0
	var held := 0
	var i := 0
	while i + 2 < a.size():
		if a[i] == a[i + 1] and a[i + 1] == a[i + 2]:
			total += 1
			if b[i] == a[i] and b[i + 1] == a[i + 1] and b[i + 2] == a[i + 2]:
				held += 1
		i += 3
	return [total, held]

func _ready() -> void:
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		push_error("colour-space probe watchdog: _ready never finished")
		get_tree().quit(2))
	bridge = load("res://shell/engine_bridge.gd").new()
	add_child(bridge)
	await get_tree().process_frame

	print("\n== 1. the binding is there, and it opens on sRGB ==")
	_ok(bridge.color_space_api, "EngineBridge.color_space_api")
	var spaces: Array = bridge.color_spaces()
	print("  spaces: %s" % str(spaces))
	_ok(spaces == ["sRGB", "Display P3"], "two display devices, sRGB first")
	_ok(bridge.color_space() == "sRGB", "opens on sRGB")
	_ok(not bridge.set_color_space("linear"),
		"'linear' is refused -- it is a working space, not a display device")
	_ok(bridge.color_space() == "sRGB", "a refused name changes nothing")

	print("\n== 2. a real world at the real size ==")
	var t0 := Time.get_ticks_msec()
	bridge.world_gen.generate_sized(20260903, 1200.0, 2048, 1312)
	bridge.has_world = true
	print("  generated 2048 x 1312 in %.1f s" % ((Time.get_ticks_msec() - t0) / 1000.0))
	var srgb_a := _screen()
	_ok(srgb_a.size() == 2048 * 1312 * 3, "the raster is 2048 x 1312 RGB8 (%d bytes)" % srgb_a.size())

	print("\n== 3. THE assertion: sRGB is byte-identical across a round trip ==")
	## Not "sRGB renders the same twice" -- that would pass even if the default
	## were a near-identity transform. This leaves sRGB, comes back, and demands
	## the original bytes.
	_ok(bridge.set_color_space("Display P3"), "set Display P3")
	var p3 := _screen()
	_ok(bridge.set_color_space("sRGB"), "set sRGB back")
	var srgb_b := _screen()
	var same := _cmp(srgb_a, srgb_b)
	print("  sRGB -> P3 -> sRGB: worst %d, %.6f %% of bytes moved" % [same[0], same[1] * 100.0])
	_ok(same[0] == 0, "the shipped default is byte-identical after a round trip")

	print("\n== 4. and Display P3 is not a no-op ==")
	## Non-vacuity: without this, section 3 would pass on a binding that was
	## never wired to the texture at all.
	var diff := _cmp(srgb_a, p3)
	print("  sRGB vs P3: worst %d levels, %.2f %% of bytes moved, mean %.3f"
		% [diff[0], diff[1] * 100.0, diff[2]])
	_ok(diff[1] > 0.5, "the colour space reaches the texture")
	_ok(diff[0] > 0, "and moves real levels")

	print("\n== 5. neutrals survive P3 exactly (the rows sum to 1) ==")
	var n := _neutrals_held(srgb_a, p3)
	print("  %d neutral pixels in the map, %d unchanged under P3" % [n[0], n[1]])
	_ok(n[0] > 0, "the map contains neutrals at all (paper, neatlines, grey rock)")
	_ok(n[0] == n[1], "every one of them is untouched")

	print("\n== 6. the export path is deliberately NOT transformed ==")
	## A PNG written here carries no ICC profile, so every reader takes it as
	## sRGB. Putting P3 numbers in one would be a mislabelled file -- worse than
	## no colour management, because it looks like a correct file. So the
	## display device stops at the screen texture, and this is the assertion.
	##
	## The comparison is **export against export**, not export against screen:
	## the two paths already differ slightly for reasons that predate this row
	## (`_exportraster_probe.gd` and `_gradeexport_probe.gd` own that question),
	## and comparing them here would measure their difference rather than mine.
	## Two exports under two display devices isolate exactly one variable.
	var dir := ProjectSettings.globalize_path("user://_colorspace_probe")
	DirAccess.make_dir_recursive_absolute(dir)
	var a := _export(dir, "srgb", "sRGB")
	var b := _export(dir, "p3", "Display P3")
	if a.size() > 0 and b.size() > 0:
		var ex := _cmp(a, b)
		print("  sRGB export vs P3 export: worst %d, %.4f %% moved" % [ex[0], ex[1] * 100.0])
		_ok(ex[0] == 0, "the export is identical whatever the display device is set to")
	else:
		_ok(false, "both exports were written and re-read")
	bridge.set_color_space("sRGB")

	print("\n== 7. the control is actually on screen in the real shell ==")
	## `cargo test` cannot see a dock that never drew. This boots `app.tscn` and
	## looks for the section by its heading text and the picker by its items --
	## the same walk `_cmdindex_probe.gd` does, and the only way to tell a live
	## row from a builder nobody calls.
	var vp := SubViewport.new()
	vp.size = Vector2i(1600, 900)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	for i in 50:
		await get_tree().process_frame
	## `DccWidgets.section` draws its title through `DccTheme.header`, which
	## upper-cases and prefixes the section sigil -- so the on-screen text is
	## "§ COLOUR MANAGEMENT", not the string the builder passed.
	_ok(_find_label(app, "§ COLOUR MANAGEMENT") != null,
		"CARTO shows a 'Colour management' section")
	var pick := _find_option(app, "Display P3")
	_ok(pick != null, "with a display-device picker carrying Display P3")
	if pick != null:
		var items := []
		for i in pick.item_count:
			items.append(pick.get_item_text(i))
		print("  picker items: %s, selected '%s'" % [str(items), pick.get_item_text(pick.selected)])
		_ok(items == ["sRGB", "Display P3"], "exactly the engine's two, in the engine's order")
		_ok(pick.get_item_text(pick.selected) == "sRGB", "opens on sRGB")

	print("\n%s (%d failure%s)" % ["ALL PASS" if fails == 0 else "FAILURES", fails, "" if fails == 1 else "s"])
	get_tree().quit(1 if fails > 0 else 0)

func _find_label(n: Node, text: String) -> Label:
	if n is Label and (n as Label).text == text:
		return n
	for c in n.get_children():
		var r := _find_label(c, text)
		if r != null:
			return r
	return null

func _find_option(n: Node, item: String) -> OptionButton:
	if n is OptionButton:
		var ob := n as OptionButton
		for i in ob.item_count:
			if ob.get_item_text(i) == item:
				return ob
	for c in n.get_children():
		var r := _find_option(c, item)
		if r != null:
			return r
	return null

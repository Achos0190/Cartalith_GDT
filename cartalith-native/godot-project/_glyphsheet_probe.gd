extends Node
## Renders all 49 landmark glyphs through **Godot's own SVG rasteriser** — the
## one that will actually ship them — and saves a specimen sheet.
##
##   Godot_v4.7.1 --path . --resolution 1500x1000 --rendering-driver opengl3 _glyphsheet_probe.tscn
##
## A design canvas shows the markup rendered by a browser. That is not the
## renderer these go through, and the thing §12 cares about — whether a 1.2
## hairline survives at 12 px — is a property of the rasteriser, not of the
## path data. So this draws them the real way and writes a PNG to look at.
##
## It also asserts the boring things that would otherwise be found by eye much
## later: every engine type resolves to a glyph, no glyph is blank, and the
## three reuses resolve to shipped names rather than to `lm_*`.

const COLS := 7
const CELL := Vector2(196, 108)

var _fail := 0

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var bridge = EngineBridge.new()
	add_child(bridge)
	await _frames(4)

	var kinds: Array = bridge.landmark_kinds() if bridge.has_method("landmark_kinds") else []
	print("[KINDS] ", kinds.size())
	_ok("the engine reports its 49 types", kinds.size(), 49)

	print("")
	print("=== 1: every engine type resolves to a glyph, and none is blank ===")
	var unresolved: Array = []
	var blank: Array = []
	for k in kinds:
		var key := String((k as Dictionary).get("key", ""))
		var g := DccIcons.landmark_glyph(key)
		if g == "":
			unresolved.append(key)
			continue
		var tex := DccIcons.get_icon(g, 24)
		if tex == null or tex.get_width() <= 0:
			blank.append(key)
	for u in unresolved:
		print("  UNRESOLVED  ", u)
	for b in blank:
		print("  BLANK       ", b)
	_ok("every type resolves", unresolved.size(), 0)
	_ok("every glyph rasterises", blank.size(), 0)

	print("")
	print("=== 2: the three reuses point at shipped glyphs, not lm_* ===")
	for pair in [["cliff", "cliff"], ["lake", "lake"], ["volcanic_feature", "volcano"]]:
		_ok("%s -> %s" % [pair[0], pair[1]], DccIcons.landmark_glyph(String(pair[0])), String(pair[1]))
	_ok("an unknown key returns empty, not a wrong glyph",
		DccIcons.landmark_glyph("no_such_landmark"), "")

	print("")
	print("=== 3: the specimen sheet ===")
	var rows: int = int(ceil(float(kinds.size()) / float(COLS)))
	var sheet := Control.new()
	sheet.custom_minimum_size = Vector2(COLS * CELL.x, rows * CELL.y + 46)
	var bg := ColorRect.new()
	bg.color = Color(0.051, 0.055, 0.059)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	sheet.add_child(bg)
	add_child(sheet)

	var title := DccTheme.mono_label(
		"LANDMARK GLYPHS · 49 TYPES · 12 / 16 / 24 px · DCC_SHELL_SPEC §12",
		"text_bright", 12, 2, true)
	title.position = Vector2(16, 14)
	sheet.add_child(title)

	for i in kinds.size():
		var kd: Dictionary = kinds[i]
		var key := String(kd.get("key", ""))
		var g := DccIcons.landmark_glyph(key)
		var cx := float(i % COLS) * CELL.x + 16.0
		var cy := float(i / COLS) * CELL.y + 46.0
		## The three sizes side by side, baseline-aligned, because the 12 px one
		## is the only one that decides anything and it needs its neighbours to
		## be judged against.
		var x := cx
		for px in [12, 16, 24]:
			var t := TextureRect.new()
			t.texture = DccIcons.get_icon(g, int(px))
			t.position = Vector2(x, cy + (24 - px))
			t.custom_minimum_size = Vector2(px, px)
			t.size = Vector2(px, px)
			sheet.add_child(t)
			x += float(px) + 10.0
		var nm := DccTheme.mono_label(String(kd.get("label", key)), "text_dim", 9, 0)
		nm.position = Vector2(cx, cy + 34)
		sheet.add_child(nm)
		var meta := DccTheme.mono_label("%s · %s%s" % [
			String(kd.get("class", "")).substr(0, 3).to_upper(),
			String(kd.get("family", "")),
			"" if bool(kd.get("buildable", false)) else " · not generated"],
			"text_ghost", 8, 0)
		meta.position = Vector2(cx, cy + 48)
		sheet.add_child(meta)

	await _frames(12)
	var img := get_viewport().get_texture().get_image()
	var out := "user://landmark_glyphs.png"
	img.save_png(out)
	print("  wrote ", ProjectSettings.globalize_path(out))

	print("")
	print("_glyphsheet_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

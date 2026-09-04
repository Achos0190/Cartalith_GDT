extends Node
## VERIFIER, WINDOWED. Independent re-derivation of Lane A's two central
## claims, built without reusing Lane A's saved PNG baselines.
##
##   Godot_v4.7.1-stable_win64.exe --path . _vfy_traitart_probe.tscn
##
## Claim 1 -- the no-art path is byte-identical to the COMMITTED file. Proved
## by rendering the committed `map_overlay.gd` (extracted to
## `_vfy_head_overlay.gd` by `git show HEAD:`) and the working-tree one side
## by side in the same frame. Regenerate the baseline copy with:
##   git show HEAD:cartalith-native/godot-project/map_overlay.gd > _vfy_head_overlay.gd
## and diff the WHOLE 2400x1200 frame, not a
## scan window.
##
## Claim 2 -- `civ_trait_badge_row` is reachable from GDScript with Godot
## argument types. Proved from `ClassDB.class_get_method_list`, then called.

const W := 2400
const H := 1200
const GW := 96
const GH := 48
const CELL := Vector2(48, 24)

const TRAITS := ["port", "mining", "military"]
const GLYPHS := {"port": "⚓", "mining": "⚒", "military": "⚔"}
const FIXTURES := [["Ka", "city"], ["Vandermeer", "metropolis"], ["Ost", "town"]]
const PACK_ZIP := "../crates/cartalith-assets/tests/fixtures/reference_pack.zip"

var _fails := 0
var _wg: Object = null
var _wb: Object = null


func _chk(ok: bool, what: String) -> void:
	print(("  PASS  " if ok else "  FAIL  ") + what)
	if not ok:
		_fails += 1


static func _places(nm: String, kind: String) -> Array:
	return [{
		"x": CELL.x, "y": CELL.y, "name": nm, "population": 41000,
		"kind": kind, "faction": 1, "capital": false, "coastal": false,
		"tid": 7,
	}]


func _arm(script_path: String) -> Array:
	var vp := SubViewport.new()
	vp.size = Vector2i(W, H)
	vp.transparent_bg = false
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	var bg := ColorRect.new()
	bg.color = Color(0.09, 0.10, 0.11)
	bg.size = Vector2(W, H)
	vp.add_child(bg)
	add_child(vp)
	var ov: Control = Control.new()
	ov.set_script(load(script_path))
	ov.size = Vector2(W, H)
	vp.add_child(ov)
	return [vp, ov]


func _push(arm: Array, nm: String, kind: String) -> void:
	var ov: Control = arm[1]
	ov.set_civ_data(_places(nm, kind), [], [], GW, GH, 0.0)
	ov.set_settlement_traits({7: TRAITS}, GLYPHS)


func _shot(arm: Array) -> Image:
	return (arm[0] as SubViewport).get_texture().get_image()


func _settle() -> void:
	for f in 6:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw


## FULL-frame byte identity, compared on the C++ side (a GDScript per-pixel
## loop over 2.88 M pixels is minutes, this is instant and STRICTER: it
## compares every channel byte of every pixel, not a Color equality).
static func _same_all(a: Image, b: Image) -> bool:
	return a.get_data() == b.get_data()


## Differing pixels inside the badge window, for magnitude only.
const SX := 1130
const SY := 598
const SW := 180
const SH := 110

static func _diff_win(a: Image, b: Image) -> int:
	var n := 0
	for j in SH:
		for i in SW:
			if a.get_pixel(SX + i, SY + j) != b.get_pixel(SX + i, SY + j):
				n += 1
	return n


static func _flat_tex(c: Color) -> ImageTexture:
	var img := Image.create_empty(16, 16, false, Image.FORMAT_RGBA8)
	img.fill(c)
	return ImageTexture.create_from_image(img)


func _stub_row(px: float, py: float, keys: PackedStringArray, sz: float, sc: float) -> Array:
	var shown: int = mini(keys.size(), 4)
	if shown <= 0:
		return []
	var r: float = maxf(2.2, sz * 0.42)
	var gap := r * 2.35
	var bx := px - float(shown - 1) * gap / 2.0
	var cy := py + sz + r + 1.2 * sc
	var out: Array = []
	for i in shown:
		var cx := bx + float(i) * gap
		out.append({
			"key": keys[i], "cx": cx, "cy": cy, "r": r,
			"texture": _flat_tex(Color(1, 0, 1, 1)),
			"dx": cx - r, "dy": cy - r, "dw": r * 2.0, "dh": r * 2.0,
		})
	return out


static func _broken_pack() -> String:
	var path := "user://vfy_traitart_broken.zip"
	var z := ZIPPacker.new()
	if z.open(path, ZIPPacker.APPEND_CREATE) != OK:
		return ""
	z.start_file("pack.json")
	z.write_file(('{"schema":2,"name":"broken","structures":'
		+ '{"trait":{"port":["structures/trait/port_01.png"]}}}').to_utf8_buffer())
	z.close_file()
	z.start_file("structures/trait/port_01.png")
	z.write_file("not a png at all".to_utf8_buffer())
	z.close_file()
	z.close()
	return ProjectSettings.globalize_path(path)


## The signature check the brief demands: not "the doc says GDScript calls
## it", but what ClassDB actually exposes and what argument types it takes.
func _check_signature() -> void:
	print("")
	print("SIGNATURE (ClassDB, not prose)")
	var have_row := ClassDB.class_has_method("WorldGen", "civ_trait_badge_row")
	var have_status := ClassDB.class_has_method("WorldGen", "pack_trait_art_status")
	_chk(have_row, "ClassDB exposes WorldGen.civ_trait_badge_row")
	_chk(have_status, "ClassDB exposes WorldGen.pack_trait_art_status")
	if not have_row:
		return
	for m in ClassDB.class_get_method_list("WorldGen", true):
		if m["name"] == "civ_trait_badge_row":
			var names: Array = []
			var types: Array = []
			for a in m["args"]:
				names.append(a["name"])
				types.append(type_string(a["type"]))
			print("    args   ", names)
			print("    types  ", types)
			print("    return ", type_string(m["return"]["type"]))
			_chk(types == ["float", "float", "PackedStringArray", "float", "float"],
				"all five arguments are Godot types (float x4 + PackedStringArray)")
			_chk(m["return"]["type"] == TYPE_ARRAY, "returns a Godot Array")


func _check_funcs() -> void:
	print("")
	print("RETURN SHAPE (called from GDScript over the real fixture pack)")
	var wg: Object = ClassDB.instantiate("WorldGen")
	# No pack loaded: both funcs must be empty, not rows of `no_art_in_pack`.
	_chk(wg.civ_trait_badge_row(100.0, 100.0, PackedStringArray(TRAITS), 10.0, 1.0).is_empty(),
		"no pack -> civ_trait_badge_row is empty")
	_chk(wg.pack_trait_art_status().is_empty(), "no pack -> pack_trait_art_status is empty")

	_wg = ClassDB.instantiate("WorldGen")
	var ok: bool = _wg.load_asset_pack(ProjectSettings.globalize_path("res://" + PACK_ZIP))
	_chk(ok, "reference_pack.zip loads")
	if not ok:
		return
	var row: Array = _wg.civ_trait_badge_row(1200.0, 600.0, PackedStringArray(TRAITS), 10.0, 1.0)
	print("    row = ", row)
	_chk(row.size() == 3, "three traits -> three badges")
	_chk(row[0].has("texture") and not row[0].has("miss"),
		"port carries a texture and no miss")
	_chk(row[0]["texture"] is Texture2D, "the texture is a Godot Texture2D")
	_chk(not row[1].has("texture") and row[1].get("miss", "") == "no_art_in_pack",
		"mining carries miss=no_art_in_pack and no texture")
	# Geometry must equal the reference's own arithmetic, computed here.
	var r: float = maxf(2.2, 10.0 * 0.42)
	var gap := r * 2.35
	_chk(is_equal_approx(row[0]["cx"], 1200.0 - gap) and is_equal_approx(row[0]["cy"], 600.0 + 10.0 + r + 1.2),
		"badge 0 sits at the reference's own cx/cy")
	_chk(is_equal_approx(row[0]["dw"], r * 2.0) and is_equal_approx(row[0]["dh"], r * 2.0),
		"the destination box is the centre-anchored r*2 square")
	var st: Dictionary = _wg.pack_trait_art_status()
	print("    status = ", st)
	_chk(st.size() == 7, "seven PACK_TRAIT_SLOTS rows")
	_chk(st["port"].get("variants", 0) >= 1 and not st["port"].has("miss"),
		"port -> variants, no miss")
	_chk(st["mining"].get("miss", "") == "no_art_in_pack" and not st["mining"].has("variants"),
		"mining -> no_art_in_pack, no variants")

	# `glyph` must be OMITTED for a key that is not a real CIV_TRAITS entry.
	var bogus: Array = _wg.civ_trait_badge_row(1200.0, 600.0, PackedStringArray(["not_a_trait"]), 10.0, 1.0)
	print("    bogus = ", bogus)
	_chk(bogus.size() == 1 and not bogus[0].has("glyph"),
		"an unknown trait key omits `glyph` entirely")
	_chk(bogus.size() == 1 and bogus[0].get("miss", "") == "no_art_in_pack",
		"an unknown trait key still reports a miss reason")

	_wb = ClassDB.instantiate("WorldGen")
	var bp := _broken_pack()
	var bok: bool = bp != "" and _wb.load_asset_pack(bp)
	_chk(bok, "broken pack loads")
	if bok:
		var brow: Array = _wb.civ_trait_badge_row(1200.0, 600.0, PackedStringArray(TRAITS), 10.0, 1.0)
		_chk(brow.size() == 3 and brow[0].get("miss", "") == "art_failed_to_decode",
			"broken port -> miss=art_failed_to_decode (resolver still live)")
		var bst: Dictionary = _wb.pack_trait_art_status()
		_chk(bst["port"].get("miss", "") == "art_failed_to_decode",
			"status separates decode failure from never-declared")


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 600.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG -- probe did not finish")
		get_tree().quit(2))
	wd.start()

	if DisplayServer.get_name() == "headless":
		print("ABORT: pixel probe cannot run headless")
		get_tree().quit(2)
		return

	_check_signature()
	_check_funcs()

	var head := _arm("res://_vfy_head_overlay.gd")
	var live := _arm("res://map_overlay.gd")
	var ctrl := _arm("res://map_overlay.gd")
	var broken := _arm("res://map_overlay.gd")
	var packed := _arm("res://map_overlay.gd")
	_chk(not (head[1] as Control).has_method("set_trait_art_resolver"),
		"the COMMITTED overlay has no set_trait_art_resolver (baseline is really the old file)")
	_chk((live[1] as Control).has_method("set_trait_art_resolver"),
		"the working-tree overlay has set_trait_art_resolver")
	(ctrl[1] as Control).set_trait_art_resolver(Callable(self, "_stub_row"))
	if _wb != null:
		(broken[1] as Control).set_trait_art_resolver(Callable(_wb, "civ_trait_badge_row"))
	if _wg != null:
		(packed[1] as Control).set_trait_art_resolver(Callable(_wg, "civ_trait_badge_row"))

	print("")
	print("NO-ART BYTE IDENTITY vs COMMITTED map_overlay.gd (full 2400x1200 frame)")
	for fx in FIXTURES:
		var nm: String = fx[0]
		var kind: String = fx[1]
		_push(head, nm, kind)
		_push(live, nm, kind)
		_push(ctrl, nm, kind)
		_push(broken, nm, kind)
		_push(packed, nm, kind)
		await _settle()
		var i_head := _shot(head)
		var i_live := _shot(live)
		var i_ctrl := _shot(ctrl)
		var i_broken := _shot(broken)
		var i_packed := _shot(packed)
		var d_packed := _diff_win(i_head, i_packed)
		_chk(d_packed > 0, "%s: the fixture pack's `port` art reaches the pin (%d px in window)" % [nm, d_packed])
		var same_live := _same_all(i_head, i_live)
		var same_ctrl := _same_all(i_head, i_ctrl)
		var same_broken := _same_all(i_head, i_broken)
		var d_ctrl := _diff_win(i_head, i_ctrl)
		print("  %s/%s   full-frame bytes equal to HEAD: no-resolver=%s  stub-control=%s  broken-pack=%s   (control window diff = %d px)"
			% [nm, kind, same_live, same_ctrl, same_broken, d_ctrl])
		_chk(same_live, "%s: no-art path byte-identical to the committed file" % nm)
		_chk(not same_ctrl and d_ctrl > 0, "%s: positive control moves pixels" % nm)
		_chk(same_broken, "%s: art_failed_to_decode draws the same as no pack" % nm)

	print("")
	print("VERIFIER RESULT: %d FAIL" % _fails)
	get_tree().quit(1 if _fails > 0 else 0)

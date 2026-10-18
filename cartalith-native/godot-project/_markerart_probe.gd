extends Node
## PIXEL VERIFY, WINDOWED: AP-3 (`ASSET_PACK_TOOLS_PROPOSAL.md` §6, Ruling CQ
## Q2). Does a pack's settlement / POI / sea-mark / custom art reach a placed
## map icon, at the glyph's size, only while the switch is on -- and is every
## other frame byte for byte what it was?
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _markerart_probe.tscn
##
## Windowed only (`MISTAKES.md`: `ImageTexture.update()` is a no-op under
## `--headless`, so a pixel comparison there passes vacuously); it refuses to
## run headless and exits 2.
##
## **The pre-change baseline is a file.** Run once against a tree whose
## `map_overlay.gd` predates AP-3 (it has no `set_marker_art_resolver`): the
## `base` arm's frame is written to `user://markerart/pre_base.png`. Run again
## on the AP-3 tree: the fresh `base` frame must equal it byte for byte. Delete
## `user://markerart/` to re-baseline. Without the file the comparison is
## reported as not made, not as a pass.
##
## Arms, each a 2400x1200 SubViewport over a flat background (icons only; no
## settlements, so nothing else moves):
##   base        the shipping overlay, switch at its default (off), no resolver
##   nopack_on   switch ON + the real `WorldGen.icon_marker_art` with NO pack
##               loaded -- must equal `base` (AP-3: "with no pack, map_overlay
##               draws byte-identically")
##   stub_on     switch ON + a GDScript resolver handing back a flat magenta
##               square for every icon -- a positive control independent of
##               Rust, and the size measurement (sprite box vs glyph box)
##   packed_on   switch ON + `WorldGen` over a pack built here with art for
##               settlement:city, poi:ruin, seamarks:buoy, custom Naval/
##               lighthouse and icons:mountain
##   packed_off  the same pack, switch OFF -- must equal `base`
## and the stub/packed pair again inside a Node2D scaled x4 (a camera zoom),
## which also exercises a larger pre-filtered raster.
##
## Prints `MARKERART ok|FAIL <what>` rows, per-icon size lines, `IMAGES: <dir>`
## and `MARKERART RESULT: ALL PASS|<n> FAILURES`. Exit 0 / 1 / 2 (could not run).

const W := 2400
const H := 1200
const GW := 96
const GH := 48
## A light ground, so the glyph's near-black outline (`ICON_OUTLINE`) and the
## sprite's dark ring both read against it: on the dark ground the outline is
## within 0.06 of the background and a visibility threshold drops it.
const BG := Color(0.86, 0.82, 0.72)

## The icons placed in every arm: `[family, slot, set, gx, gy, scale, art?]`.
## `art` says whether the pack built here fills that slot, so whether
## `packed_on` must change it. `feature`/mountain HAS art in the pack and must
## still not change (AP-3 does not draw feature art); settlement:town has none.
const ICONS := [
	["settlement", "city", "", 10.0, 10.0, 1.0, true],
	["poi", "ruin", "", 25.0, 10.0, 1.0, true],
	["seamarks", "buoy", "", 40.0, 10.0, 1.0, true],
	["custom", "lighthouse", "Naval", 55.0, 10.0, 1.0, true],
	["feature", "mountain", "", 70.0, 10.0, 1.0, false],
	["settlement", "town", "", 85.0, 10.0, 1.0, false],
	["settlement", "city", "", 10.0, 30.0, 3.0, true],
	["poi", "ruin", "", 25.0, 30.0, 0.5, true],
	["custom", "lighthouse", "Naval", 40.0, 30.0, 2.0, true],
	["seamarks", "buoy", "", 55.0, 30.0, 2.5, true],
]

var _fails := 0
var _checks := 0
## Held as members: a `WorldGen` local dies when `_ready()` returns and the
## `Callable` bound to it goes invalid (`_traitart_probe.gd` measured that).
var _wg_nopack: Object = null
var _wg_pack: Object = null


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print("MARKERART %s  %s" % ["ok  " if cond else "FAIL", what])
	if not cond:
		_fails += 1


func _icons() -> Array:
	var out: Array = []
	for e in ICONS:
		out.append({"x": e[3], "y": e[4], "family": e[0], "slot": e[1], "set": e[2],
			"scale": e[5], "origin": "manual"})
	return out


## One arm: a SubViewport with a background, and the overlay under a Node2D of
## `zoom` scale (1 = no camera).
func _arm(zoom: float) -> Array:
	var vp := SubViewport.new()
	vp.size = Vector2i(W, H)
	vp.transparent_bg = false
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	var bg := ColorRect.new()
	bg.color = BG
	bg.size = Vector2(W, H)
	vp.add_child(bg)
	var cam := Node2D.new()
	cam.scale = Vector2(zoom, zoom)
	vp.add_child(cam)
	add_child(vp)
	var ov: Control = Control.new()
	ov.set_script(load("res://map_overlay.gd"))
	ov.size = Vector2(W, H)
	cam.add_child(ov)
	ov.set_civ_data([], [], [], GW, GH, 0.0)
	ov.set_manual_icons(_icons())
	return [vp, ov]


func _settle() -> void:
	for f in 6:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw


static func _shot(arm: Array) -> Image:
	return (arm[0] as SubViewport).get_texture().get_image()


## Where icon `i` lands on screen in an arm of `zoom`: `_point_to_screen` of
## the overlay at that zoom. Read from the overlay itself so the probe does not
## restate the transform.
static func _screen_pos(ov: Control, i: int, zoom: float) -> Vector2:
	var e: Array = ICONS[i]
	return ov.call("_point_to_screen", Vector2(e[3], e[4]), ov.call("_displayed_rect")) * zoom


## Bounding box of pixels VISIBLY differing from the background within a
## `half`-px window around `c`, as `[w, h]`, or `[]` when there are none.
## "Visibly" = some channel more than `VISIBLE` away, i.e. at least about a
## third covered by the dark outline: a vector glyph's antialiasing leaves
## single faint pixels past its edge (measured on a dark ground: a ring
## glyph's arc puts one stray pixel above and below it, reading 12x14 for a
## 12x12 mark), which are not part of the mark's on-screen size.
const VISIBLE := 0.25
static func _bbox_vs_bg(img: Image, c: Vector2, half: int) -> Array:
	var bg := img.get_pixel(0, 0)
	var x0 := 1 << 30
	var y0 := 1 << 30
	var x1 := -1
	var y1 := -1
	for y in range(maxi(0, int(c.y) - half), mini(img.get_height(), int(c.y) + half)):
		for x in range(maxi(0, int(c.x) - half), mini(img.get_width(), int(c.x) + half)):
			var q := img.get_pixel(x, y)
			if maxf(absf(q.r - bg.r), maxf(absf(q.g - bg.g), absf(q.b - bg.b))) > VISIBLE:
				x0 = mini(x0, x)
				y0 = mini(y0, y)
				x1 = maxi(x1, x)
				y1 = maxi(y1, y)
	return [] if x1 < 0 else [x1 - x0 + 1, y1 - y0 + 1]


## Pixels that differ between two frames within a window around `c`.
static func _diff_at(a: Image, b: Image, c: Vector2, half: int) -> int:
	var n := 0
	for y in range(maxi(0, int(c.y) - half), mini(a.get_height(), int(c.y) + half)):
		for x in range(maxi(0, int(c.x) - half), mini(a.get_width(), int(c.x) + half)):
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
	return n


static func _flat_tex(c: Color) -> ImageTexture:
	var img := Image.create_empty(32, 32, false, Image.FORMAT_RGBA8)
	img.fill(c)
	return ImageTexture.create_from_image(img)


var _magenta: ImageTexture = null

## The positive-control resolver: every icon of every family gets a flat
## magenta square, aspect 1 -- so the drawn box IS the sprite box.
func _stub(_family: String, _slot: String, _set_name: String, _x: float, _y: float, _px: float) -> Dictionary:
	return {"texture": _magenta, "aspect": 1.0}


## A 256² sprite: a filled disc of `fill` with a dark ring, transparent
## outside -- art with real alpha, not a flat square.
static func _disc_png(fill: Color) -> PackedByteArray:
	var img := Image.create_empty(256, 256, false, Image.FORMAT_RGBA8)
	img.fill(Color(0, 0, 0, 0))
	for y in 256:
		for x in 256:
			var d := Vector2(x + 0.5 - 128.0, y + 0.5 - 128.0).length()
			if d < 110.0:
				img.set_pixel(x, y, fill)
			elif d < 124.0:
				img.set_pixel(x, y, Color(0.05, 0.04, 0.03, 1.0))
	return img.save_png_to_buffer()


## A real pack `.zip` with art for the five slots `ICONS` names as filled;
## `with_feature` false leaves out `icons:mountain` (the app probe's frames).
static func _build_pack(with_feature: bool = true) -> String:
	var path := "user://markerart/pack.zip"
	var z := ZIPPacker.new()
	if z.open(path, ZIPPacker.APPEND_CREATE) != OK:
		return ""
	var files := {
		"structures/settlement/city_01.png": _disc_png(Color(0.85, 0.37, 0.1)),
		"structures/poi/ruin_01.png": _disc_png(Color(0.95, 0.85, 0.2)),
		"seamarks/buoy_01.png": _disc_png(Color(0.9, 0.1, 0.1)),
		"custom/naval/lighthouse_01.png": _disc_png(Color(0.3, 0.7, 0.95)),
	}
	var manifest := {"schema": 2, "name": "markerart", "license": "CC0",
		"structures": {"settlement": {"city": ["structures/settlement/city_01.png"]},
			"poi": {"ruin": ["structures/poi/ruin_01.png"]}},
		"seamarks": {"buoy": ["seamarks/buoy_01.png"]},
		"custom": {"Naval": {"lighthouse": ["custom/naval/lighthouse_01.png"]}}}
	if with_feature:
		files["icons/mountain_01.png"] = _disc_png(Color(0.6, 0.6, 0.6))
		manifest["icons"] = {"mountain": ["icons/mountain_01.png"]}
	z.start_file("pack.json")
	z.write_file(JSON.stringify(manifest).to_utf8_buffer())
	z.close_file()
	for f in files:
		z.start_file(f)
		z.write_file(files[f])
		z.close_file()
	z.close()
	return ProjectSettings.globalize_path(path)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG -- probe did not finish")
		get_tree().quit(2))
	wd.start()
	if DisplayServer.get_name() == "headless":
		print("ABORT: this probe measures pixels and cannot run headless --")
		print("  Godot_v4.7.2-stable_win64_console.exe --path . _markerart_probe.tscn")
		get_tree().quit(2)
		return
	var dir := "user://markerart"
	DirAccess.make_dir_recursive_absolute(dir)
	_magenta = _flat_tex(Color(1, 0, 1, 1))

	var base := _arm(1.0)
	var has_switch: bool = (base[1] as Control).has_method("set_marker_art_resolver")
	print("map_overlay has set_marker_art_resolver: ", has_switch)
	var engine_has: bool = ClassDB.class_has_method("WorldGen", "icon_marker_art")
	print("engine has icon_marker_art: ", engine_has)
	await _settle()
	var i_base := _shot(base)
	i_base.save_png(dir + "/base.png")
	var pre_path := dir + "/pre_base.png"
	if not has_switch:
		i_base.save_png(pre_path)
		print("BASELINE WRITTEN  %s -- this is the pre-AP-3 overlay; re-run on the AP-3 tree" % pre_path)
		print("MARKERART RESULT: BASELINE ONLY")
		get_tree().quit(0)
		return
	if FileAccess.file_exists(pre_path):
		var pre := Image.load_from_file(pre_path)
		_ok(pre != null and pre.get_data() == i_base.get_data(),
			"switch-off frame is BYTE-IDENTICAL to the pre-AP-3 overlay's own render")
	else:
		print("MARKERART note  no pre-AP-3 baseline at %s; that comparison was NOT made" % pre_path)
	_ok(not bool((base[1] as Control).get("marker_art_enabled")), "the overlay's own default is OFF")
	if not engine_has:
		print("MARKERART RESULT: CANNOT RUN (engine lacks icon_marker_art)")
		get_tree().quit(2)
		return

	# ---- arms ----
	_wg_nopack = ClassDB.instantiate("WorldGen")
	_wg_pack = ClassDB.instantiate("WorldGen")
	var pack_path := _build_pack()
	var loaded: bool = pack_path != "" and _wg_pack.load_asset_pack(pack_path)
	_ok(loaded, "the probe's pack loads (%s)" % pack_path)
	var nopack_on := _arm(1.0)
	(nopack_on[1] as Control).set_marker_art_resolver(Callable(_wg_nopack, "icon_marker_art"), true)
	var stub_on := _arm(1.0)
	(stub_on[1] as Control).set_marker_art_resolver(Callable(self, "_stub"), true)
	var packed_on := _arm(1.0)
	(packed_on[1] as Control).set_marker_art_resolver(Callable(_wg_pack, "icon_marker_art"), true)
	var packed_off := _arm(1.0)
	(packed_off[1] as Control).set_marker_art_resolver(Callable(_wg_pack, "icon_marker_art"), false)
	var base4 := _arm(4.0)
	var stub4 := _arm(4.0)
	(stub4[1] as Control).set_marker_art_resolver(Callable(self, "_stub"), true)
	var packed4 := _arm(4.0)
	(packed4[1] as Control).set_marker_art_resolver(Callable(_wg_pack, "icon_marker_art"), true)
	await _settle()
	var i_nopack := _shot(nopack_on)
	var i_stub := _shot(stub_on)
	var i_packed := _shot(packed_on)
	var i_off := _shot(packed_off)
	var i_base4 := _shot(base4)
	var i_stub4 := _shot(stub4)
	var i_packed4 := _shot(packed4)
	for pair in [["nopack_on", i_nopack], ["stub_on", i_stub], ["packed_on", i_packed],
			["packed_off", i_off], ["base_x4", i_base4], ["stub_x4", i_stub4], ["packed_x4", i_packed4]]:
		(pair[1] as Image).save_png("%s/%s.png" % [dir, pair[0]])

	# ---- byte identity where nothing may move ----
	_ok(i_nopack.get_data() == i_base.get_data(),
		"switch ON with NO pack loaded draws the whole frame byte-identically to switch off")
	_ok(i_off.get_data() == i_base.get_data(),
		"a loaded pack with the switch OFF draws the whole frame byte-identically (the switch is real)")
	_ok(i_stub.get_data() != i_base.get_data(), "positive control: the stub resolver moves pixels")

	# ---- per icon ----
	var ov: Control = base[1]
	var ov4: Control = base4[1]
	for i in ICONS.size():
		var e: Array = ICONS[i]
		var tag := "%s:%s%s x%.1f" % [e[0], e[1], ("/" + String(e[2])) if String(e[2]) != "" else "", e[5]]
		for z in [1.0, 4.0]:
			var p := _screen_pos(ov if z == 1.0 else ov4, i, z)
			var half := int(ceil(40.0 * z * float(e[5]))) + 4
			if p.x < 0 or p.y < 0 or p.x >= W or p.y >= H:
				continue
			var gl := _bbox_vs_bg(i_base if z == 1.0 else i_base4, p, half)
			var sp := _bbox_vs_bg(i_stub if z == 1.0 else i_stub4, p, half)
			if gl.is_empty() or sp.is_empty():
				_ok(false, "%s at x%d: glyph %s / sprite %s box measurable" % [tag, int(z), gl, sp])
				continue
			if String(e[0]) != "feature":
				_ok(absi(int(sp[1]) - int(gl[1])) <= 1 and absi(int(sp[0]) - int(gl[0])) <= 1,
					"%s at x%d: sprite box %dx%d vs glyph box %dx%d (within 1 px)" % [tag, int(z), sp[0], sp[1], gl[0], gl[1]])
			var moved := _diff_at(i_packed if z == 1.0 else i_packed4, i_base if z == 1.0 else i_base4, p, half)
			if bool(e[6]):
				_ok(moved > 0, "%s at x%d: the pack's art replaces the glyph (%d px changed)" % [tag, int(z), moved])
			else:
				_ok(moved == 0, "%s at x%d: no AP-3 art, so the glyph is unchanged (%d px changed)" % [tag, int(z), moved])

	print("IMAGES: ", ProjectSettings.globalize_path(dir))
	print("MARKERART checks=%d failures=%d" % [_checks, _fails])
	print("MARKERART RESULT: ", "ALL PASS" if _fails == 0 else "%d FAILURES" % _fails)
	get_tree().quit(1 if _fails > 0 else 0)

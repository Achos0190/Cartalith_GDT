extends Node
## PIXEL VERIFY, WINDOWED: does an imported pack's `structures.trait` art reach
## a settlement pin, and is the no-art path still byte for byte what it was?
##
##   Godot_v4.7.1-stable_win64.exe --path . _traitart_probe.tscn
##
## `MISTAKES.md`: *"`ImageTexture.update()` is a no-op under `--headless`"*, so
## this runs windowed and refuses to run otherwise.
##
## **The baseline is the file, not a subclass.** `_traitbadge_probe.gd` proves
## its "before" with a `_PreChange` subclass of the shipping overlay; that
## cannot prove *this* change, because the change is inside
## `_draw_trait_badges` itself and a subclass overriding it would test the
## override. So the baseline here is a PNG of the real file, captured by
## running this probe against the tree BEFORE the edit:
##
##   run 1 (pre-edit)   `user://traitart/pre_<kind>.png` does not exist -> written
##   run 2 (post-edit)  it does -> the fresh no-resolver frame must equal it
##                      byte for byte, or this probe fails
##
## Delete `user://traitart/` to re-baseline. The path is printed at the end.
##
## Four arms, three fixtures each:
##
##   noresolver  shipping overlay, traits pushed, NO art resolver installed --
##               the state every world is in until a pack is imported
##   stub        + a GDScript resolver handing back flat magenta squares --
##               a positive control that must move pixels, independent of Rust
##   packed      + `WorldGen.civ_trait_badge_row` over the real
##               `reference_pack.zip`, which has art for `port` and for
##               nothing else -- so the `port` badge must change and the
##               `mining`/`military` badges must not
##   broken      + the same over a pack built here whose declared trait PNG is
##               not a PNG -- the `art_failed_to_decode` state, which no real
##               fixture can express
##
## and a set of non-pixel assertions on what the two new `#[func]`s return,
## because the map draws the same fallback for both miss reasons on purpose
## and pixels therefore cannot tell them apart.

const W := 2400
const H := 1200
const GW := 96
const GH := 48
const CELL := Vector2(48, 24)

## The scan window in screen pixels: the pin and the badge row below it.
const SX := 1130
const SY := 598
const SW := 180
const SH := 110

## `port` first because it is the one slot `reference_pack.zip` has art for.
const TRAITS := ["port", "mining", "military"]
const GLYPHS := {"port": "⚓", "mining": "⚒", "military": "⚔"}

## `[name, kind]`. Tier sets the pin radius, so the badge radius and the row
## width -- `MISTAKES.md`: *"one world is one sample"*. Named in the output.
const FIXTURES := [["Ka", "city"], ["Vandermeer", "metropolis"], ["Ost", "town"]]

const PACK_ZIP := "../crates/cartalith-assets/tests/fixtures/reference_pack.zip"

var _fails := 0

## `WorldGen` is `RefCounted`. Held as members and not as `_ready()` locals:
## a local's last reference dies when `_ready()` returns, the `Callable`
## bound to it goes invalid, and every art arm then silently draws the
## fallback -- which reads exactly like "the wiring does not work". Measured,
## not anticipated: the first run of this probe reported 0 px changed on all
## three fixtures for that reason and nothing else.
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


func _arm(with_script: bool) -> Array:
	var vp := SubViewport.new()
	vp.size = Vector2i(W, H)
	vp.transparent_bg = false
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	var bg := ColorRect.new()
	bg.color = Color(0.09, 0.10, 0.11)
	bg.size = Vector2(W, H)
	vp.add_child(bg)
	add_child(vp)
	if not with_script:
		return [vp, null]
	var ov: Control = Control.new()
	ov.set_script(load("res://map_overlay.gd"))
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
	for f in 4:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw


## Pixels inside the scan window where two frames differ.
static func _diff(a: Image, b: Image) -> int:
	var n := 0
	for j in SH:
		for i in SW:
			if a.get_pixel(SX + i, SY + j) != b.get_pixel(SX + i, SY + j):
				n += 1
	return n


## `[x_left, x_right]` of the columns where two frames differ, absolute, or an
## empty array when they do not differ at all -- a result, not a failure.
static func _xspan(a: Image, b: Image) -> Array:
	var x0 := SW
	var x1 := -1
	for j in SH:
		for i in SW:
			if a.get_pixel(SX + i, SY + j) != b.get_pixel(SX + i, SY + j):
				x0 = mini(x0, i)
				x1 = maxi(x1, i)
	if x1 < 0:
		return []
	return [SX + x0, SX + x1]


## A flat square, so a stub badge's own geometry adds nothing to what is being
## measured: `dw == dh == 2r` and the drawn box is exactly the disc's box.
static func _flat_tex(c: Color) -> ImageTexture:
	var img := Image.create_empty(16, 16, false, Image.FORMAT_RGBA8)
	img.fill(c)
	return ImageTexture.create_from_image(img)


## A resolver with the shape `map_overlay.gd` expects, built entirely here:
## the reference's own layout arithmetic, so this control arm depends on no
## Rust at all.
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


## A real `.zip` whose manifest declares `structures.trait.port` art and whose
## payload at that path is not a PNG -- `TraitArtMiss::ArtFailedToDecode`,
## which the reference fixture cannot express because a fixture holding a
## broken PNG is a broken fixture.
static func _broken_pack() -> String:
	var path := "user://traitart_broken.zip"
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
		print("  Godot_v4.7.1-stable_win64.exe --path . _traitart_probe.tscn")
		get_tree().quit(2)
		return

	var dir := "user://traitart"
	DirAccess.make_dir_recursive_absolute(dir)
	var wired: bool = ClassDB.class_has_method("WorldGen", "civ_trait_badge_row")
	print("engine has civ_trait_badge_row: ", wired)

	# ---- the two `#[func]`s, checked on their own terms before any pixels ----
	if wired:
		_check_funcs()

	var noresolver := _arm(true)
	var stub := _arm(true)
	var packed := _arm(true)
	var broken := _arm(true)
	var overlay_wired: bool = (noresolver[1] as Control).has_method("set_trait_art_resolver")
	print("map_overlay has set_trait_art_resolver: ", overlay_wired)

	var wg_ok := false
	var wg_broken_ok := false
	if overlay_wired:
		(stub[1] as Control).set_trait_art_resolver(Callable(self, "_stub_row"))
		if wired:
			_wg = ClassDB.instantiate("WorldGen")
			wg_ok = _wg.load_asset_pack(ProjectSettings.globalize_path("res://" + PACK_ZIP))
			if wg_ok:
				(packed[1] as Control).set_trait_art_resolver(Callable(_wg, "civ_trait_badge_row"))
			_wb = ClassDB.instantiate("WorldGen")
			var bp := _broken_pack()
			wg_broken_ok = bp != "" and _wb.load_asset_pack(bp)
			if wg_broken_ok:
				(broken[1] as Control).set_trait_art_resolver(Callable(_wb, "civ_trait_badge_row"))
	print("reference_pack.zip loaded: ", wg_ok, "   broken pack loaded: ", wg_broken_ok)

	for fx in FIXTURES:
		var nm: String = fx[0]
		var kind: String = fx[1]
		_push(noresolver, nm, kind)
		_push(stub, nm, kind)
		_push(packed, nm, kind)
		_push(broken, nm, kind)
		await _settle()

		var i_none := _shot(noresolver)
		var i_stub := _shot(stub)
		var i_packed := _shot(packed)
		var i_broken := _shot(broken)

		print("")
		print("================ fixture: \"%s\", a %s ================" % [nm, kind])

		# ---- rule 2: the no-art path is byte for byte what it was ----
		var base_png := "%s/pre_%s.png" % [dir, kind]
		if not FileAccess.file_exists(base_png):
			i_none.save_png(base_png)
			print("  BASELINE WRITTEN  %s -- re-run to compare" % base_png)
		else:
			var pre := Image.load_from_file(base_png)
			_chk(pre != null and i_none.get_data() == pre.get_data(),
				"no-resolver frame is BYTE-IDENTICAL to the pre-change file's own render")

		if not overlay_wired:
			print("  overlay has no resolver slot; the art arms are not measurable yet")
			continue

		# ---- positive control: a resolver moves pixels at all ----
		var n_stub := _diff(i_stub, i_none)
		_chk(n_stub > 0, "positive control: a stub resolver MOVES pixels (%d px)" % n_stub)
		var sp_stub := _xspan(i_stub, i_none)
		if sp_stub.is_empty():
			print("  the stub arm moved nothing; the row span is not measurable")
			continue
		if not sp_stub.is_empty():
			print("  stub art spans x %d..%d" % [sp_stub[0], sp_stub[1]])

		if not wg_ok:
			print("  reference pack did not load; the real-art arm is not measurable")
			continue

		# ---- the real pack: `port` has art, `mining`/`military` do not ----
		var n_packed := _diff(i_packed, i_none)
		_chk(n_packed > 0, "an imported pack's OWN art reaches the pin (%d px changed)" % n_packed)
		var sp := _xspan(i_packed, i_none)
		if not sp.is_empty():
			print("  pack art spans x %d..%d, of a %d..%d row"
				% [sp[0], sp[1], sp_stub[0], sp_stub[1]])
			# `port` is the first of three badges, so its art must sit in the
			# LEFT third of the row the stub filled end to end. If the port
			# sprite were painted at the wrong badge, or the two `mining`/
			# `military` badges were painted with it, this span would widen.
			var third: float = float(int(sp_stub[0]) + int(sp_stub[1])) / 2.0
			_chk(sp[1] < third,
				"and only the FIRST badge changed -- `mining`/`military` have no art in this pack and kept their disc (right edge %d < row midpoint %.1f)"
				% [sp[1], third])

		if wg_broken_ok:
			# 0 px is the *expected* answer here, so it has to be shown that
			# the resolver was actually consulted -- otherwise a dead
			# `Callable` passes this check for the wrong reason, which is
			# exactly how the pack arm failed on this probe's first run.
			var brow: Array = _wb.civ_trait_badge_row(0.0, 0.0, PackedStringArray(TRAITS), 10.0, 2.0)
			_chk(brow.size() == 3 and String((brow[0] as Dictionary).get("miss", "")) == "art_failed_to_decode",
				"the broken-pack arm's resolver is live and reports `art_failed_to_decode`")
			_chk(_diff(i_broken, i_none) == 0,
				"and a pack whose trait PNG will not decode draws the SAME fallback as no pack at all")

	print("")
	print("IMAGES: ", ProjectSettings.globalize_path(dir))
	print("TRAITART RESULT: ", "ALL PASS" if _fails == 0 else "%d FAILURES" % _fails)
	get_tree().quit(1 if _fails > 0 else 0)


## What the two new `#[func]`s return, independent of any drawing: the map
## draws the same disc for both miss reasons on purpose, so no pixel
## comparison can separate them.
func _check_funcs() -> void:
	print("")
	print("================ `civ_trait_badge_row` / `pack_trait_art_status` ================")
	var wg: Object = ClassDB.instantiate("WorldGen")

	_chk((wg.civ_trait_badge_row(100.0, 100.0, PackedStringArray(TRAITS), 10.0, 2.0) as Array).is_empty(),
		"with NO pack loaded the row is empty -- not three `no_art_in_pack` badges")
	_chk((wg.pack_trait_art_status() as Dictionary).is_empty(),
		"and the status readout is empty for the same reason")

	_chk(wg.load_asset_pack(ProjectSettings.globalize_path("res://" + PACK_ZIP)),
		"reference_pack.zip loads")
	_chk((wg.civ_trait_badge_row(100.0, 100.0, PackedStringArray(), 10.0, 2.0) as Array).is_empty(),
		"a settlement with no traits gets an empty row")

	# The reference's own arithmetic, written out here rather than read back
	# from the engine: `r = max(2.2, sz*0.42)`, `gap = r*2.35`, row centred on
	# `px`, `cy = py + sz + r + 1.2*sc`.
	var px := 100.0
	var py := 100.0
	var sz := 10.0
	var sc := 2.0
	var r := maxf(2.2, sz * 0.42)
	var row: Array = wg.civ_trait_badge_row(px, py, PackedStringArray(TRAITS), sz, sc)
	_chk(row.size() == 3, "three traits lay out three badges (got %d)" % row.size())
	if row.size() == 3:
		var b0: Dictionary = row[0]
		var b1: Dictionary = row[1]
		var b2: Dictionary = row[2]
		_chk(is_equal_approx(float(b0["r"]), r), "badge radius is max(2.2, sz*0.42) = %.3f" % r)
		_chk(is_equal_approx(float(b1["cx"]) - float(b0["cx"]), r * 2.35),
			"badges are spaced r*2.35 = %.3f apart" % (r * 2.35))
		_chk(is_equal_approx((float(b0["cx"]) + float(b2["cx"])) / 2.0, px),
			"the row is centred on the pin")
		_chk(is_equal_approx(float(b0["cy"]), py + sz + r + 1.2 * sc),
			"and sits at py + sz + r + 1.2*sc = %.3f" % (py + sz + r + 1.2 * sc))

		_chk(b0.has("texture") and not b0.has("miss"),
			"`port` -- the one slot this pack has art for -- carries a texture and no miss")
		_chk(b0.has("dx") and b0.has("dy") and b0.has("dw") and b0.has("dh"),
			"and its destination rect")
		if b0.has("dw"):
			# `port_01.png` is 256x256, so `dw = dh = r*2` exactly.
			_chk(is_equal_approx(float(b0["dw"]), r * 2.0) and is_equal_approx(float(b0["dh"]), r * 2.0),
				"a square source fills the badge's own r*2 box (%.3f)" % (r * 2.0))
			_chk(is_equal_approx(float(b0["dx"]) + float(b0["dw"]) / 2.0, float(b0["cx"])),
				"centre-anchored on the badge, not bottom-anchored")
		_chk(not b1.has("texture") and String(b1.get("miss", "")) == "no_art_in_pack",
			"`mining` is absent from this pack's manifest -> `no_art_in_pack`")
		_chk(String(b1.get("glyph", "")) != "",
			"and still carries its glyph, so the caller can draw the reference's fallback")

	var st: Dictionary = wg.pack_trait_art_status()
	_chk(st.size() == 7, "the status readout has one row per PACK_TRAIT_SLOTS member (got %d)" % st.size())
	var port: Dictionary = st.get("port", {})
	_chk(int(port.get("variants", 0)) == 1 and not port.has("miss"),
		"`port`: 1 variant, no miss")
	var mining: Dictionary = st.get("mining", {})
	_chk(String(mining.get("miss", "")) == "no_art_in_pack" and not mining.has("variants"),
		"`mining`: `no_art_in_pack`, and no `variants: 0` standing in for it")

	# What one redraw of a busy map costs, since this is called once per
	# settlement that carries a trait. 200 pins x 3 badges, five runs, median
	# with the spread -- `MISTAKES.md`: no point estimates.
	var runs: Array[float] = []
	for t in 5:
		var t0 := Time.get_ticks_usec()
		for n in 200:
			wg.civ_trait_badge_row(float(n), py, PackedStringArray(TRAITS), sz, sc)
		runs.append(float(Time.get_ticks_usec() - t0) / 1000.0)
	runs.sort()
	print("  200 pins x 3 badges: median %.3f ms (%.3f .. %.3f over 5 runs)"
		% [runs[2], runs[0], runs[4]])

	# The third state, which needs a pack that declares art it cannot decode.
	var bp := _broken_pack()
	var wb: Object = ClassDB.instantiate("WorldGen")
	if bp != "" and wb.load_asset_pack(bp):
		var bst: Dictionary = wb.pack_trait_art_status()
		var bport: Dictionary = bst.get("port", {})
		_chk(String(bport.get("miss", "")) == "art_failed_to_decode",
			"a declared-but-undecodable `port` reports `art_failed_to_decode`, NOT `no_art_in_pack`")
		var brow: Array = wb.civ_trait_badge_row(px, py, PackedStringArray(["port"]), sz, sc)
		_chk(brow.size() == 1 and not (brow[0] as Dictionary).has("texture")
			and String((brow[0] as Dictionary).get("miss", "")) == "art_failed_to_decode",
			"and the badge row says the same thing per badge")
	else:
		_chk(false, "the broken-pack fixture could not be built; the third state is unmeasured")

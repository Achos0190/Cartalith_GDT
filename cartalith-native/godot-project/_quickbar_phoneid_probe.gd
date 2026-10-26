extends Node
## Pixel-identity probe for the PHONE screens with an EMPTY quick bar and the assistant off -- the
## identity bar of milestone CQ-6 of `CUSTOM_QUICK_BAR_SCOPE.md` (Ruling DH Q9): "with an EMPTY quick bar
## and the assistant off the phone screens must be pixel-identical to the pristine tree".
##
##   capture: godot --path . --resolution 393x852 _quickbar_phoneid_probe.tscn -- --force-touch --out <dir>
##   compare: godot --path . --resolution 393x852 _quickbar_phoneid_probe.tscn -- --force-touch --base <dirA> --base2 <dirB> [--shots <dir>]
##   other size: the same with `--resolution 1080x2340`
##
## **Windowed, never `--headless`** (`ImageTexture.update()` is a no-op headless, so every comparison would
## pass vacuously -- `MISTAKES.md` "Run a pixel probe"). It reads only `--out`, `--base`, `--base2`,
## `--shots` and `--force-touch`; anything else aborts with exit 2 (`MISTAKES.md` "Write a probe's usage
## header"). Run it from a scratch worktree with its own `config/name` and a wiped user dir.
##
## **It touches only APIs that exist on the tree BEFORE CQ-6** (`_pick_phone_tab`, `set_phone_detent`, the
## MORE overlay's `_push_screen`), so the SAME file runs on the pristine tree to write the baselines and on
## the changed tree to compare -- a baseline is only a baseline if the code that wrote it is the code being
## replaced. Method as `_quickbar_ringid_probe.gd` (CQ-4, CQ-7): the baseline is captured TWICE on the
## pristine tree (`--base`, `--base2`); a pixel that differs between those two runs is DYNAMIC and is
## masked and counted; every `Label` and `RichTextLabel` is made transparent before each capture (layout
## untouched) so the generation timings and the clock text drop out, which leaves the shell's geometry and
## every non-text drawing to be compared.
##
## Screens captured: the MAP sheet at peek, half and full; the GENERATE sheet at half and full; MORE's root.
## **MORE's Preferences screen is captured and reported but is NOT part of the bar**: CQ-6 gives the phone
## the `Preferences > Quick bar...` row (the one way to a first pin when nothing is pinned, since an empty bar
## draws no row and no section), so that one screen differs by that row on purpose.
## Controls (all must hold or the run is not evidence): DETERMINISM (one state captured twice in one run),
## SENSITIVITY (peek and half differ by many pixels), EMPTY STORE (no pins, assistant off at launch).
## Exit 0 identical / written, 1 differs, 2 bad argument or watchdog, 3 control failed.

const SEED := 483920
## Tags whose difference is expected and reported, never counted (see above).
const EXPECTED_DIFF := ["more_prefs"]

var _out := ""
var _base := ""
var _base2 := ""
var _shots := ""
var _app: Node
var _images := {}
var _bad := 0


func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame


func _freeze_chrome() -> void:
	for n in _app.find_children("*", "Label", true, false):
		(n as CanvasItem).modulate.a = 0.0
	for n in _app.find_children("*", "RichTextLabel", true, false):
		(n as CanvasItem).modulate.a = 0.0
	## The shell's toasts (`_show_phone_toast`: z_index 50, a child of `_phone_root`) slide and fade on a timer, so
	## their position at the instant of a capture differs from run to run (the first compare run differed by
	## 15 296 px in a 248 x 98 box that was a coach mark mid-slide, in `gen_half`, `gen_full` and `more_root`,
	## measured against two baselines whose own difference there was 7 032 px). They are not what is compared,
	## so they are hidden; layout is untouched.
	if _app._phone_root != null:
		for c in _app._phone_root.get_children():
			if c is PanelContainer and (c as Control).z_index == 50:
				(c as Control).visible = false


func _grab(tag: String) -> Image:
	_freeze_chrome()
	await _settle(4)
	var img: Image = get_viewport().get_texture().get_image()
	img.convert(Image.FORMAT_RGBA8)
	_images[tag] = img
	if _shots != "":
		img.save_png(_shots.path_join("phoneid_%s.png" % tag))
	return img


static func _diff_px(a: Image, b: Image, mask: Image = null) -> int:
	if a.get_size() != b.get_size():
		return a.get_width() * a.get_height()
	var da := a.get_data()
	var db := b.get_data()
	if da == db:
		return 0
	var dm := mask.get_data() if mask != null else PackedByteArray()
	var px := a.get_width() * a.get_height()
	var stride := da.size() / px
	var n := 0
	for i in px:
		if not dm.is_empty() and dm[i] != 0:
			continue
		var o := i * stride
		for c in stride:
			if da[o + c] != db[o + c]:
				n += 1
				break
	return n


static func _mask_of(a: Image, b: Image) -> Image:
	var da := a.get_data()
	var db := b.get_data()
	var px := a.get_width() * a.get_height()
	var stride := da.size() / px
	var out := PackedByteArray()
	out.resize(px)
	if da != db:
		for i in px:
			var o := i * stride
			for c in stride:
				if da[o + c] != db[o + c]:
					out[i] = 255
					break
	return Image.create_from_data(a.get_width(), a.get_height(), false, Image.FORMAT_L8, out)


static func _mask_count(m: Image) -> int:
	var n := 0
	for v in m.get_data():
		if v != 0:
			n += 1
	return n


static func _bbox(a: Image, b: Image) -> Rect2i:
	var da := a.get_data()
	var db := b.get_data()
	if da == db or a.get_size() != b.get_size():
		return Rect2i()
	var w := a.get_width()
	var px := w * a.get_height()
	var stride := da.size() / px
	var x0 := 1 << 30
	var y0 := 1 << 30
	var x1 := -1
	var y1 := -1
	for i in px:
		var o := i * stride
		for c in stride:
			if da[o + c] != db[o + c]:
				var x := i % w
				var y := i / w
				x0 = mini(x0, x)
				y0 = mini(y0, y)
				x1 = maxi(x1, x)
				y1 = maxi(y1, y)
				break
	return Rect2i(x0, y0, x1 - x0 + 1, y1 - y0 + 1)


## Puts the phone shell on `tab` at `det` and waits out the detent tween (0.28 s) plus a few frames.
func _show(tab: String, det: String) -> void:
	if _app._phone_menu != null and _app._phone_menu.is_open():
		_app._phone_menu.close()
	_app._pick_phone_tab(tab)
	_app.set_phone_detent(det)
	await get_tree().create_timer(0.6).timeout
	await _settle(4)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := args[i]
		if a == "--force-touch":
			pass
		elif a == "--out" and i + 1 < args.size():
			i += 1
			_out = args[i]
		elif a == "--base" and i + 1 < args.size():
			i += 1
			_base = args[i]
		elif a == "--base2" and i + 1 < args.size():
			i += 1
			_base2 = args[i]
		elif a == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --out, --base, --base2, --shots and --force-touch only" % a)
			get_tree().quit(2)
			return
		i += 1
	if (_out == "") == (_base == "") or (_base != "" and _base2 == ""):
		print("ABORT: give exactly one of --out <dir> (capture) or --base <dirA> --base2 <dirB> (compare)")
		get_tree().quit(2)
		return
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	## EMPTY STORE control (CQ-1's accessor and the LLM section both exist on the pristine tree, origin/main
	## at 31a5f6e9, so the same file runs on both).
	if DccSettings.quick_bar_value("bar") != [] or bool(DccSettings.llm_value("enabled")):
		print("CONTROL FAIL: the quick bar is not empty or the assistant is on at launch (wipe the user dir)")
		get_tree().quit(3)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	if not _app.is_phone():
		print("ABORT: this probe is for the phone form (--resolution 393x852 -- --force-touch)")
		get_tree().quit(2)
		return
	print("== phone, viewport %s ==" % str(get_viewport().get_visible_rect().size))
	var bridge = _app.bridge
	bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(8)

	await _show("map", "peek")
	await _grab("map_peek")
	var again := await _grab("map_peek_again")
	var dd := _diff_px(again, _images["map_peek"])
	print("ID determinism: the same state captured twice differs by %d px" % dd)
	if dd > 2000:
		print("CONTROL FAIL: the same state differs by %d px in one run" % dd)
		get_tree().quit(3)
		return
	await _show("map", "half")
	await _grab("map_half")
	var sens := _diff_px(_images["map_half"], _images["map_peek"])
	print("ID sensitivity: peek vs half moves %d px" % sens)
	if sens < 5000:
		print("CONTROL FAIL: peek and half moved only %d px -- the comparison cannot see the sheet" % sens)
		get_tree().quit(3)
		return
	await _show("map", "full")
	await _grab("map_full")
	await _show("gen", "half")
	await _grab("gen_half")
	await _show("gen", "full")
	await _grab("gen_full")
	await _show("map", "peek")
	_app._pick_phone_tab("more")
	await get_tree().create_timer(0.6).timeout
	await _settle(4)
	await _grab("more_root")
	_app._phone_menu._push_screen("prefs")
	await _settle(6)
	await _grab("more_prefs")
	_app._phone_menu.close()
	await _settle(2)

	if _out != "":
		DirAccess.make_dir_recursive_absolute(_out)
		for tag in _images:
			(_images[tag] as Image).save_png(_out.path_join("%s.png" % tag))
		print("RESULT: WROTE %d captures to %s" % [_images.size(), _out])
		get_tree().quit(0)
		return
	var same := 0
	for tag in _images:
		if tag == "map_peek_again":
			continue
		var ia := Image.load_from_file(_base.path_join("%s.png" % tag))
		var ib := Image.load_from_file(_base2.path_join("%s.png" % tag))
		if ia == null or ib == null:
			print("ID MISSING %s" % tag)
			_bad += 1
			continue
		ia.convert(Image.FORMAT_RGBA8)
		ib.convert(Image.FORMAT_RGBA8)
		var mask := _mask_of(ia, ib)
		var d := _diff_px(_images[tag], ia, mask)
		var line := "ID %s %-12s %d differing px outside the dynamic mask (%d masked), bbox %s" % [
			"same  " if d == 0 else "DIFFER", tag, d, _mask_count(mask), str(_bbox(_images[tag], ia))]
		if EXPECTED_DIFF.has(tag):
			print(line + "  [expected to differ: reported, not counted]")
			continue
		print(line)
		if d == 0:
			same += 1
		else:
			_bad += 1
	print("RESULT: %s %d/%d screens (0 differing pixels required)" % ["IDENTICAL" if _bad == 0 else "DIFFERS", same, same + _bad])
	get_tree().quit(0 if _bad == 0 else 1)

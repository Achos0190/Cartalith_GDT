extends Node
## Pixel-identity probe for the cartwheel with NOTHING pinned -- the bar of milestone CQ-4 of
## `CUSTOM_QUICK_BAR_SCOPE.md` (Ruling DH): "with no pins and no assistant the ring renders
## pixel-identical to today's (framebuffer diff 0 differing pixels, windowed)".
##
##   capture: godot --path . --resolution 1600x1000 _quickbar_ringid_probe.tscn -- --out <dir>
##   compare: godot --path . --resolution 1600x1000 _quickbar_ringid_probe.tscn -- --base <dirA> --base2 <dirB> [--shots <dir>]
##   tablet / phone: the same with `--resolution 1280x800` / `393x852` and `-- --force-touch`
##
## **Windowed, never `--headless`** (`ImageTexture.update()` is a no-op headless, so every comparison
## would pass vacuously). It reads only `--out`, `--base`, `--base2`, `--shots` and `--force-touch`;
## any other argument aborts with exit 2. Run from a scratch worktree with its own `config/name` and a
## wiped user dir. **It touches only APIs that exist on the tree BEFORE CQ-4** (the broker's `ring_*`
## calls and the ring's `debug_state()`), so the SAME file runs on the pristine tree to write the
## baselines and on the changed tree to compare -- a baseline is only a baseline if the code that
## wrote it is the code being replaced (`MISTAKES.md`: a probe earns authority by running against the
## state before the change).
##
## What it captures: the whole window (the shell, the map and the ring) as a PNG, for 5 ring states in
## each of the three domains -- just opened, a slot hovered, a sub-ring open, the ring 10 px from the
## bottom edge and from the top edge. A whole-window HASH was tried first and differed in every state
## even between two runs of the SAME code: the status bar prints the generation time and the repaint
## cost. So the baseline is captured TWICE on the pristine tree (`--base`, `--base2`); a pixel that
## differs between those two runs is DYNAMIC (clock text, a cursor readout) and is excluded from the
## comparison, and is counted and reported so the exclusion cannot hide the ring. Two samples are not
## enough to find every moving digit (the first compare run still differed by 9 to 326 pixels, ALL in
## the status bar and the left dock's pipeline timings, measured by bounding box), so the bar is
## the count of differing pixels INSIDE the ring's own region -- its centre plus the main ring, the
## sub-ring's reach, the caption pill and the room under it where a dock would draw -- between this
## tree and the first baseline, outside the dynamic mask. The whole-window count and its bounding box are
## printed beside it, so a difference outside the region is visible, not hidden.
## The cause is then removed rather than masked: `_freeze_chrome()` makes every `Label` and
## `RichTextLabel` of the shell transparent before each capture (layout untouched, so both trees lay
## out identically), which takes the generation timings, the repaint cost and the pipeline status text
## out of the picture. What is compared is then the shell's own geometry, the map, the scrim, the disc,
## the slots, the pill and the sub-ring.
## Controls (all must hold or the run is not evidence):
##   * DETERMINISM: the first state is captured twice in one run and must be near pixel-equal there;
##   * SENSITIVITY: a hover on another slot must differ by many pixels (a comparison that cannot
##     move proves nothing);
##   * EMPTY STORE: the dock/share store must be empty at launch.
## Prints one `ID` row per state and `RESULT: IDENTICAL n/n` or `DIFFERS`.
## Exit 0 identical / written, 1 differs, 2 bad argument or watchdog, 3 control failed.

const SEED := 483920
const DOMAINS := ["world", "civilization", "cartography"]

var _out := ""
var _base := ""
var _base2 := ""
var _shots := ""
var _app: Node
var _images := {}   ## tag -> Image
var _rois := {}     ## tag -> Rect2i, the ring's own region at that capture
var _bad := 0


func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame


func _grab(tag: String) -> Image:
	_freeze_chrome()
	await _settle(4)
	var img: Image = get_viewport().get_texture().get_image()
	img.convert(Image.FORMAT_RGBA8)
	_images[tag] = img
	var st: Dictionary = _app.context_broker.ring.debug_state() if _app.context_broker.ring != null else {}
	if bool(st.get("active", false)):
		var c: Vector2 = st["centre"]
		var reach := float(st["radius"]) + float(st["sub_disc_radius"]) + 24.0
		_rois[tag] = Rect2i(Vector2i(int(c.x - reach), int(c.y - reach)), Vector2i(int(2.0 * reach), int(2.0 * reach + 60.0)))
	if _shots != "":
		img.save_png(_shots.path_join("ringid_%s.png" % tag.replace("/", "_")))
	return img


## Number of pixels at which `a` and `b` differ, ignoring any pixel set in `mask` (when given).
static func _diff_px(a: Image, b: Image, mask: Image = null, roi: Rect2i = Rect2i()) -> int:
	if a.get_size() != b.get_size():
		return a.get_width() * a.get_height()
	var da := a.get_data()
	var db := b.get_data()
	if da == db:
		return 0   ## byte-equal: nothing differs whatever the mask says (also the fast path)
	var dm := mask.get_data() if mask != null else PackedByteArray()
	var n := 0
	var w := a.get_width()
	var px := w * a.get_height()
	var stride := da.size() / px
	var r := Rect2i(Vector2i.ZERO, a.get_size())
	if roi.size != Vector2i.ZERO:
		r = r.intersection(roi)
	for y in range(r.position.y, r.end.y):
		for x in range(r.position.x, r.end.x):
			var i := y * w + x
			if not dm.is_empty() and dm[i] != 0:
				continue
			var o := i * stride
			for c in stride:
				if da[o + c] != db[o + c]:
					n += 1
					break
	return n


## Bounding box of every differing pixel (whole image), for the informational line.
static func _diff_bbox(a: Image, b: Image) -> Rect2i:
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


## A mask image: 255 where `a` and `b` differ, else 0 (single channel FORMAT_L8).
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


## Makes every text label of the shell invisible without moving anything. Run after a domain change
## (the docks rebuild their rows) and before every capture.
func _freeze_chrome() -> void:
	for n in _app.find_children("*", "Label", true, false):
		(n as CanvasItem).modulate.a = 0.0
	for n in _app.find_children("*", "RichTextLabel", true, false):
		(n as CanvasItem).modulate.a = 0.0


func _local(global: Vector2) -> Vector2:
	return _app.viewport.overlay.get_global_transform_with_canvas().affine_inverse() * global


func _open_at(global: Vector2) -> Dictionary:
	var b = _app.context_broker
	b.ring_close()
	await _settle(2)
	b.ring_key_press(_local(global))
	await _settle(2)
	b.ring_key_release()
	await _settle(2)
	return b.ring.debug_state()


func _slot_global(st: Dictionary, dir: String) -> Vector2:
	var ang := deg_to_rad(float({"N": -90.0, "NE": -45.0, "E": 0.0, "SE": 45.0, "S": 90.0, "SW": 135.0, "W": 180.0, "NW": -135.0}[dir]))
	return (st["centre"] as Vector2) + Vector2(cos(ang), sin(ang)) * float(st["radius"])


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
			print("ABORT: unrecognised argument %s -- this probe reads --out, --base, --shots and --force-touch only" % a)
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
	if DccSettings.quick_bar_value("dock") != [] or DccSettings.quick_bar_value("share") != {}:
		print("CONTROL FAIL: the dock/share store is not empty at launch (wipe the user dir)")
		get_tree().quit(3)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	var mode := "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	var vp := get_viewport().get_visible_rect().size
	print("== mode %s, viewport %s ==" % [mode, vp])
	var bridge = _app.bridge
	bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(8)

	var ov_rect: Rect2 = _app.viewport.overlay.get_global_rect()
	var mid := ov_rect.get_center()
	var opened_any := false
	for d in DOMAINS:
		_app.select_domain(d)
		await _settle(10)
		_freeze_chrome()
		await _settle(2)
		var st := await _open_at(mid)
		if bool(st.get("active", false)):
			opened_any = true
		await _grab("%s/open" % d)
		if d == DOMAINS[0]:
			## DETERMINISM control: the same state again, same run.
			var again := await _grab("%s/open_again" % d)
			var dd := _diff_px(again, _images["%s/open" % d])
			print("ID determinism: the same state captured twice differs by %d px" % dd)
			if dd > 2000:
				print("CONTROL FAIL: the same ring state differs by %d px in one run -- the scene is not stable" % dd)
				get_tree().quit(3)
				return
		if bool(st.get("active", false)):
			_app.context_broker.ring_pointer(_local(_slot_global(st, "E")))
			await _settle(3)
			await _grab("%s/hover_E" % d)
			if d == DOMAINS[0]:
				## SENSITIVITY control: another slot must hash differently.
				_app.context_broker.ring_pointer(_local(_slot_global(st, "W")))
				await _settle(3)
				var hw := await _grab("%s/hover_W" % d)
				var sens := _diff_px(hw, _images["%s/hover_E" % d])
				print("ID sensitivity: hovering W instead of E moves %d px" % sens)
				if sens < 500:
					print("CONTROL FAIL: hovering E and W moved only %d px -- the comparison cannot see the ring" % sens)
					get_tree().quit(3)
					return
			## A sub-ring: a diagonal that carries children opens on a click into it.
			var sub_dir := "NE"
			_app.context_broker.ring_pointer(_local(_slot_global(st, sub_dir)))
			await _settle(2)
			_app.context_broker.ring_click(_local(_slot_global(st, sub_dir)))
			await _settle(3)
			var st2: Dictionary = _app.context_broker.ring.debug_state()
			if bool(st2.get("sub_open", false)):
				await _grab("%s/sub_%s" % [d, sub_dir])
			else:
				print("NOTE: no sub-ring opened for %s %s" % [d, sub_dir])
		for edge in ["bottom", "top"]:
			var p := Vector2(mid.x, ov_rect.end.y - 10.0 if edge == "bottom" else ov_rect.position.y + 10.0)
			var ste := await _open_at(p)
			await _grab("%s/edge_%s" % [d, edge])
			if bool(ste.get("active", false)):
				print("ID edge %s %s centre=%s" % [d, edge, str(ste["centre"])])
		_app.context_broker.ring_close()
		await _settle(2)
	_app.select_domain("world")
	if not opened_any:
		print("NOTE: the ring never became active at this form (%s); the captures are of the plain shell" % mode)

	if _out != "":
		DirAccess.make_dir_recursive_absolute(_out)
		for tag in _images:
			(_images[tag] as Image).save_png(_out.path_join("%s.png" % tag.replace("/", "_")))
		print("RESULT: WROTE %d captures to %s (mode %s, ring opened: %s)" % [_images.size(), _out, mode, str(opened_any)])
		get_tree().quit(0)
		return
	var same := 0
	for tag in _images:
		if tag.ends_with("open_again"):
			continue
		var fa := _base.path_join("%s.png" % tag.replace("/", "_"))
		var fb := _base2.path_join("%s.png" % tag.replace("/", "_"))
		var ia := Image.load_from_file(fa)
		var ib := Image.load_from_file(fb)
		if ia == null or ib == null:
			print("ID MISSING %s" % tag)
			_bad += 1
			continue
		ia.convert(Image.FORMAT_RGBA8)
		ib.convert(Image.FORMAT_RGBA8)
		var cur: Image = _images[tag]
		var mask := _mask_of(ia, ib)
		var mc := _mask_count(mask)
		var roi: Rect2i = _rois.get(tag, Rect2i())
		var d := _diff_px(cur, ia, mask, roi)
		var whole := _diff_px(cur, ia, mask)
		if d == 0:
			same += 1
		else:
			_bad += 1
		print("ID %s %-28s ring region %s: %d differing px | whole window outside the mask: %d, bbox %s | dynamic mask %d px" % ["same  " if d == 0 else "DIFFER", tag, str(roi), d, whole, str(_diff_bbox(cur, ia)), mc])
	var total := same + _bad
	print("RESULT: %s %d/%d states (0 differing pixels required; this mode: %s)" % ["IDENTICAL" if _bad == 0 else "DIFFERS", same, total, mode])
	get_tree().quit(0 if _bad == 0 else 1)

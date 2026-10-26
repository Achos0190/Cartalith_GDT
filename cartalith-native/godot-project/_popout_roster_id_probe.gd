extends Node
## Pixel-identity probe for the faction roster with the pop-out setting OFF -- the pristine-tree bar of
## milestone PW-2 of `POPOUT_WINDOWS_SCOPE.md` (Ruling DK): "with the setting off the roster must be
## pixel-identical to the pristine tree" (the CQ-4 / CQ-7 method).
##
##   capture: godot --path . --resolution 1400x900 _popout_roster_id_probe.tscn -- --out <dir>
##   compare: godot --path . --resolution 1400x900 _popout_roster_id_probe.tscn -- --base <dirA> --base2 <dirB> [--shots <dir>]
##
## **Windowed, never `--headless`** (`ImageTexture.update()` is a no-op headless, so every comparison
## would pass vacuously). It reads only `--out`, `--base`, `--base2` and `--shots`; anything else aborts,
## exit 2. Run from a scratch worktree with its own `config/name` and a wiped user dir. **It touches only
## APIs that exist on the tree BEFORE PW-2** (`FactionRosterWindow.open(faction, tab)`, `bridge.generate`)
## plus an optional lookup by NAME of the per-window button (`PopoutButton`) that only the changed tree
## has, so the SAME file writes the baselines on the pristine tree and compares on the changed one: a
## baseline is only a baseline if the code that wrote it is the code being replaced (`MISTAKES.md`).
##
## What it captures: the roster window's own rectangle (the dialog is embedded and centred at this
## resolution) for six states -- the Identity, Territory, Settlements, Economy, Relations and Military
## tabs of faction 2 -- on a generated world of seed 483920. The baseline is captured TWICE on the pristine
## tree; a pixel that differs between those two runs is DYNAMIC and is excluded and counted.
## What it reports on the changed tree (setting off):
##   * the differing pixels over the whole window rectangle, with their bounding box;
##   * the same count OUTSIDE the per-window button's rectangle -- the intended change is exactly that
##     button, so the bar is 0 outside it;
##   * the same count with the button HIDDEN (the flip-the-flag test): 0 over the whole window means the
##     only difference between the trees is that one control;
##   * the OK ("Close") button's rectangle on both trees (it must not have moved).
## Controls: DETERMINISM (the first state captured twice in one run, 0 differing); SENSITIVITY (a second
## faction's Identity tab must differ by many pixels, so the comparison can move).
## Prints one `ID` row per state and `RESULT: IDENTICAL|DIFFERS|WRITTEN`. Exit 0 identical / written,
## 1 differs, 2 bad argument or watchdog, 3 a control failed.

const SEED := 483920
const TABS := ["identity", "territory", "settlements", "economy", "relations", "military"]

var _out := ""
var _base := ""
var _base2 := ""
var _shots := ""
var _app: Node
var _images := {}   ## tag -> Image (the window rectangle)
var _meta := {}     ## tag -> Dictionary(window rect, ok rect, button rect or empty)
var _bad := 0
var _ctl := 0


func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a in ["--out", "--base", "--base2", "--shots"] and i + 1 < args.size():
			match a:
				"--out": _out = String(args[i + 1])
				"--base": _base = String(args[i + 1])
				"--base2": _base2 = String(args[i + 1])
				"--shots": _shots = String(args[i + 1])
			i += 2
		else:
			print("ABORT: unknown argument %s" % a)
			get_tree().quit(2)
			return
	if _out == "" and (_base == "" or _base2 == ""):
		print("ABORT: --out <dir>, or --base <dir> --base2 <dir>")
		get_tree().quit(2)
		return
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	var bridge = _app.bridge
	bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _settle(10)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	if not bridge.has_world:
		print("ABORT: no world")
		get_tree().quit(2)
		return
	var w: Window = _app.faction_roster_window
	for tab in TABS:
		await _capture("t_" + tab, w, 2, tab)
	await _capture("t_identity_again", w, 2, "identity")
	await _capture("t_other_faction", w, 1, "identity")
	w.hide()
	if _out != "":
		DirAccess.make_dir_recursive_absolute(_out)
		for tag in _images:
			(_images[tag] as Image).save_png(_out.path_join(tag + ".png"))
			var f := FileAccess.open(_out.path_join(tag + ".json"), FileAccess.WRITE)
			f.store_string(JSON.stringify(_meta[tag]))
		print("RESULT: WRITTEN %d states to %s" % [_images.size(), _out])
		get_tree().quit(0)
		return
	_compare()


func _rect_dict(r: Rect2i) -> Dictionary:
	return {"x": r.position.x, "y": r.position.y, "w": r.size.x, "h": r.size.y}


func _capture(tag: String, w: Window, fid: int, tab: String) -> void:
	w.open(fid, tab)
	await _settle(8)
	var win := Rect2i(w.position, w.size)
	var ok := Rect2i()
	var okb: Button = w.get_ok_button()
	if okb != null:
		ok = Rect2i(Vector2i(okb.get_global_rect().position) + w.position, Vector2i(okb.get_global_rect().size))
	var btn := Rect2i()
	var pb := w.find_child("PopoutButton", true, false)
	if pb != null and (pb as Control).visible:
		btn = Rect2i(Vector2i((pb as Control).get_global_rect().position) + w.position, Vector2i((pb as Control).get_global_rect().size))
	var img := get_viewport().get_texture().get_image()
	_images[tag] = img.get_region(win)
	_meta[tag] = {"win": _rect_dict(win), "ok": _rect_dict(ok), "btn": _rect_dict(btn), "has_btn": pb != null}
	if pb != null and _out == "":
		(pb as Control).visible = false
		var bal := w.find_child("PopoutButtonBalance", true, false)
		if bal != null:
			(bal as Control).visible = false
		await _settle(4)
		_images[tag + "@hidden"] = get_viewport().get_texture().get_image().get_region(win)
		(pb as Control).visible = true
		if bal != null:
			(bal as Control).visible = true
		await _settle(2)


func _load(dir: String, tag: String) -> Image:
	var img := Image.load_from_file(ProjectSettings.globalize_path(dir.path_join(tag + ".png")))
	return img


func _diff(a: Image, b: Image, mask: Image = null, skip: Rect2i = Rect2i()) -> Dictionary:
	var n := 0
	var bb := Rect2i()
	var first := true
	if a == null or b == null or a.get_size() != b.get_size():
		return {"n": -1, "bb": Rect2i(), "note": "size differs"}
	a.convert(Image.FORMAT_RGBA8)
	b.convert(Image.FORMAT_RGBA8)
	for y in a.get_height():
		for x in a.get_width():
			if skip.size != Vector2i.ZERO and skip.has_point(Vector2i(x, y)):
				continue
			if mask != null and mask.get_pixel(x, y).r > 0.5:
				continue
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
				var p := Rect2i(x, y, 1, 1)
				bb = p if first else bb.merge(p)
				first = false
	return {"n": n, "bb": bb}


func _dyn_mask(a: Image, b: Image) -> Image:
	var m := Image.create(a.get_width(), a.get_height(), false, Image.FORMAT_R8)
	for y in a.get_height():
		for x in a.get_width():
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				m.set_pixel(x, y, Color(1, 0, 0))
	return m


func _compare() -> void:
	var fails := 0
	# control: determinism within this run
	var d0 := _diff(_images["t_identity"], _images["t_identity_again"])
	print("CONTROL determinism: %d differing (want 0)" % d0["n"])
	if int(d0["n"]) != 0:
		_ctl += 1
	var ds := _diff(_images["t_identity"], _images["t_other_faction"])
	print("CONTROL sensitivity: %d differing between faction 2 and faction 1 (want > 500)" % ds["n"])
	if int(ds["n"]) <= 500:
		_ctl += 1
	var total_outside := 0
	var total_hidden := 0
	var total_dyn := 0
	var moved_ok := 0
	for tag in _images:
		if tag.ends_with("@hidden") or tag in ["t_identity_again", "t_other_faction"]:
			continue
		var b1 := _load(_base, tag)
		var b2 := _load(_base2, tag)
		if b1 == null or b2 == null:
			print("ABORT: baseline %s missing" % tag)
			get_tree().quit(2)
			return
		var mask := _dyn_mask(b1, b2) if b1.get_size() == b2.get_size() else null
		var dyn: int = _diff(b1, b2)["n"]
		total_dyn += maxi(dyn, 0)
		var m: Dictionary = _meta[tag]
		var btn := Rect2i(m["btn"]["x"], m["btn"]["y"], m["btn"]["w"], m["btn"]["h"])
		var win := Rect2i(m["win"]["x"], m["win"]["y"], m["win"]["w"], m["win"]["h"])
		var btn_local := Rect2i(btn.position - win.position, btn.size) if btn.size != Vector2i.ZERO else Rect2i()
		if _shots != "":
			DirAccess.make_dir_recursive_absolute(_shots)
			(_images[tag] as Image).save_png(_shots.path_join(tag + ".png"))
		var all := _diff(b1, _images[tag], mask)
		var outside := _diff(b1, _images[tag], mask, btn_local)
		total_outside += maxi(int(outside["n"]), 0) if int(outside["n"]) >= 0 else 1000000
		var hid := {"n": 0}
		if _images.has(tag + "@hidden"):
			hid = _diff(b1, _images[tag + "@hidden"], mask)
			total_hidden += maxi(int(hid["n"]), 0) if int(hid["n"]) >= 0 else 1000000
		# the OK button must not move
		var bm := _read_json(_base.path_join(tag + ".json"))
		if bm.is_empty() or not _same_rect(bm["ok"], m["ok"]):
			moved_ok += 1
		print("ID %-18s whole=%s bb=%s outside_button=%s outside_bb=%s hidden=%s dyn=%d ok=%s btn=%s" % [tag, all["n"], str(all.get("bb", "")), outside["n"], str(outside.get("bb", "")), hid["n"], dyn, str(m["ok"]), str(m["btn"])])
	print("TOTAL outside the button: %d, with the button hidden: %d, dynamic pixels excluded: %d, OK buttons moved: %d" % [total_outside, total_hidden, total_dyn, moved_ok])
	if _ctl > 0:
		print("RESULT: CONTROL FAILED")
		get_tree().quit(3)
		return
	var identical := total_outside == 0 and total_hidden == 0 and moved_ok == 0
	print("RESULT: %s" % ("IDENTICAL" if identical else "DIFFERS"))
	get_tree().quit(0 if identical else 1)


func _read_json(path: String) -> Dictionary:
	var f := FileAccess.open(ProjectSettings.globalize_path(path), FileAccess.READ)
	if f == null:
		return {}
	var v: Variant = JSON.parse_string(f.get_as_text())
	return v if v is Dictionary else {}


## JSON round-trips ints as floats, so two rectangles are compared by value, not by Variant type.
func _same_rect(a: Dictionary, b: Dictionary) -> bool:
	for k in ["x", "y", "w", "h"]:
		if int(a.get(k, -1)) != int(b.get(k, -2)):
			return false
	return true

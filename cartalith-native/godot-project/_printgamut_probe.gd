extends Node
## **ET-E3's print-gamut warning in `DccWidgets.confirmable_color`, driven.**
## `EDITING_TOOLS_SCOPE.md` ET-E3 acceptance, three states:
##
##   A. no `EngineBridge` in the tree           -> row hidden, nothing asked;
##   B. a bridge whose `color_print_gamut` says `{}` (the "no CMYK profile"
##      answer)                                  -> row hidden for blue too;
##   C. the real `EngineBridge` over the loaded engine:
##      grey (128,128,128) answers in gamut and the row stays hidden;
##      blue (0,0,255) answers out of gamut and the row is DRAWN (visible in
##      tree, non-zero size, inside the popup); a real mouse click on the
##      swatch sets the picker to `nearest`, the call site's live handler
##      receives it, and `nearest` re-checks in gamut.
##
## Positive control: state C's blue MUST show the row, so a widget that never
## draws cannot pass A and B vacuously. Exits non-zero on any failure, and when
## the loaded binary has no `color_print_gamut` (it cannot answer, so it must
## not report a pass).
##
## Draws and clicks, so WINDOWED, never `--headless`:
##   godot --path . _printgamut_probe.tscn
## Flags: none. Any `--flag` after `--` aborts rather than being ignored.

var _fail := 0
var _live: Array[Color] = []

func _check(ok: bool, what: String) -> void:
	print("%s %s" % ["PASS" if ok else "FAIL", what])
	if not ok:
		_fail += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## A fresh picker under `host`, upgraded by `confirmable_color`, with a live
## handler that records what the call site would apply.
func _make_picker(host: Control) -> ColorPickerButton:
	var pick := ColorPickerButton.new()
	pick.custom_minimum_size = Vector2(80, 30)
	pick.color = Color(0.5, 0.5, 0.5)
	host.add_child(pick)
	pick.color_changed.connect(func(c: Color): _live.append(c))
	DccWidgets.confirmable_color(pick, func(_c: Color): pass)
	return pick

func _row(pick: ColorPickerButton) -> HBoxContainer:
	return pick.get_popup().find_child("PrintGamut", true, false) as HBoxContainer

## Opens the popup the way the button does, then sets a colour as a drag would
## (assign + `color_changed`, which is what `ColorPicker` emits on a drag).
func _open_and_set(pick: ColorPickerButton, c: Color) -> void:
	var pop := pick.get_popup()
	pop.popup()
	await _frames(3)
	pick.color = c
	pick.color_changed.emit(c)
	await _frames(3)

func _drawn(r: Control) -> bool:
	return r != null and r.is_visible_in_tree() and r.size.x > 0.0 and r.size.y > 0.0

func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--"):
			push_error("unknown flag %s" % a)
			get_tree().quit(2)
			return
	var host := Control.new()
	host.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(host)
	var blue := Color(0, 0, 1)
	var grey := Color8(128, 128, 128)

	# -- A: no bridge at all ---------------------------------------------------
	var pa := _make_picker(host)
	await _open_and_set(pa, blue)
	_check(DccWidgets._print_gamut_bridge(pa) == null, "A: no EngineBridge in the tree -> nothing to ask")
	_check(not _drawn(_row(pa)), "A: blue with no bridge -> glyph not drawn")
	pa.get_popup().hide()
	await _frames(2)

	# -- B: a bridge that answers {} (no CMYK profile) ------------------------
	var stub_src := GDScript.new()
	stub_src.source_code = "extends Node\nfunc color_print_gamut(_c: Color) -> Dictionary:\n\treturn {}\n"
	stub_src.reload()
	var stub := Node.new()
	stub.set_script(stub_src)
	stub.name = "EngineBridge"
	get_tree().root.add_child(stub)
	var pb := _make_picker(host)
	await _open_and_set(pb, blue)
	_check(DccWidgets._print_gamut_bridge(pb) == stub, "B: the stub bridge is found")
	_check((stub.call("color_print_gamut", blue) as Dictionary).is_empty(), "B: the bridge dictionary is empty")
	_check(not _drawn(_row(pb)), "B: empty answer -> glyph not drawn")
	pb.get_popup().hide()
	await _frames(2)
	get_tree().root.remove_child(stub)
	stub.free()
	await _frames(1)

	# -- C: the real bridge and engine ----------------------------------------
	var bridge := EngineBridge.new()
	bridge.name = "EngineBridge"
	get_tree().root.add_child(bridge)
	await _frames(2)
	if not bridge.world_gen.has_method("color_print_gamut"):
		print("FAIL C: the loaded binary has no WorldGen.color_print_gamut -- cannot answer")
		get_tree().quit(3)
		return
	var dg: Dictionary = bridge.color_print_gamut(grey)
	var db: Dictionary = bridge.color_print_gamut(blue)
	print("C: grey -> %s" % dg)
	print("C: blue -> %s" % db)
	_check(dg.has("in_gamut") and bool(dg["in_gamut"]), "C: grey (128,128,128) is in gamut")
	_check(db.has("in_gamut") and not bool(db["in_gamut"]), "C: blue (0,0,255) is flagged")
	_check(String(db.get("source", "")) == "own_icc", "C: source names the engine's own ICC profile")

	var pc := _make_picker(host)
	await _open_and_set(pc, grey)
	_check(not _drawn(_row(pc)), "C: grey -> glyph not drawn")
	pc.color = blue
	pc.color_changed.emit(blue)
	await _frames(4)
	var row := _row(pc)
	_check(_drawn(row), "C: blue -> glyph and swatch drawn (positive control)")
	var glyph: Control = (row.get_node("PrintGamutGlyph") as Control) if row != null else null
	_check(_drawn(glyph), "C: the warning glyph itself is drawn")

	# A real click on the swatch, through the root viewport's input.
	var sw: Button = (row.get_node("PrintGamutNearest") as Button) if row != null else null
	var pop := pc.get_popup()
	_live.clear()
	if sw != null:
		var at: Vector2 = Vector2(pop.position) + sw.get_global_rect().get_center()
		for pressed in [true, false]:
			var ev := InputEventMouseButton.new()
			ev.button_index = MOUSE_BUTTON_LEFT
			ev.pressed = pressed
			ev.position = at
			ev.global_position = at
			get_viewport().push_input(ev)
			await _frames(2)
	await _frames(3)
	var near: Color = db.get("nearest", Color(-1, -1, -1))
	var close := func(a: Color, b: Color) -> bool:
		return absf(a.r - b.r) < 1e-4 and absf(a.g - b.g) < 1e-4 and absf(a.b - b.b) < 1e-4
	_check(db.has("nearest") and close.call(pc.color, near), "C: click sets the picker to nearest (%s -> %s)" % [pc.color, near])
	_check(_live.size() > 0 and close.call(_live[-1], near), "C: the call site's live handler received nearest")
	var dn: Dictionary = bridge.color_print_gamut(pc.color)
	_check(dn.has("in_gamut") and bool(dn["in_gamut"]), "C: nearest re-checks in gamut (dE00 %s)" % dn.get("delta_e", "-"))
	_check(not _drawn(_row(pc)), "C: after the click the warning is gone")

	print("PROBE-RESULT: %s (fail=%d)" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)

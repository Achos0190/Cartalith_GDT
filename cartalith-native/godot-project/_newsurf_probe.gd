extends Node
## TEMPORARY, untracked probe for the 2026-08-25 "is every control wired" pass.
##
## The surfaces that landed last are the least verified, so they get driven
## rather than read: the trade-flow match (§42 IN-13), the undo ledger (§42
## ED-02), contested borders (§41 CV-23), the vault window (§42 VA-01), and
## every entry in the Layers popover -- that last one measured in **pixels**,
## because RD-02's finding was five controls that all drew the same colour and
## no amount of reading node text would have caught it.
##
## Every check also asks RF-01's question: after a fresh generate, does the
## control go back to a state that matches the new world, or keep the old one's?
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _newsurf_probe.tscn

var _app: Node
var _bridge
var _fail := 0


func _p(s: String) -> void:
	print("NEWSURF  %s" % s)


func _bad(s: String) -> void:
	_fail += 1
	print("NEWSURF  FAIL  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


func _texts(root: Node) -> String:
	if root == null:
		return ""
	var all: Array = []
	_walk(root, all)
	var parts := PackedStringArray()
	for n in all:
		if n is Label:
			parts.append((n as Label).text)
		elif n is RichTextLabel:
			parts.append((n as RichTextLabel).get_parsed_text())
		elif n is Button:
			parts.append("%s|%s" % [(n as Button).text, str((n as Button).disabled)])
	return "\n".join(parts)


func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null


func _find_button(n: Node, needle: String) -> Button:
	if n is Button and String((n as Button).text).findn(needle) >= 0:
		return n as Button
	for c in n.get_children(true):
		var r := _find_button(c, needle)
		if r != null:
			return r
	return null


func _all_categories(ws: Node) -> Array:
	var out: Array = []
	out.append_array(ws.categories)
	for extra in ["_infra", "_render"]:
		if ws.get(extra) != null:
			out.append_array((ws.get(extra) as Node).categories)
	return out


func _cat(ws: Node, title: String) -> Control:
	var cats := _all_categories(ws)
	for e in cats:
		var d: Dictionary = e
		if String(d["title"]) == title:
			for e2 in cats:
				((e2 as Dictionary)["body"] as Control).visible = false
			(d["body"] as Control).visible = true
			return d["body"]
	return null


func _layer_names() -> PackedStringArray:
	var out := PackedStringArray()
	var nodes: Array = []
	_walk(_app.layers_popover, nodes)
	for n in nodes:
		if n is Button and not (n is CheckBox) and not (n is OptionButton) \
				and (n as Button).text.strip_edges() != "" and not (n as Button).disabled:
			out.append((n as Button).text)
	return out


func _layer_button(label: String) -> Button:
	_app.layers_popover.visible = true
	if _app.layers_popover.has_method("rebuild"):
		_app.layers_popover.call("rebuild")
	var nodes: Array = []
	_walk(_app.layers_popover, nodes)
	for n in nodes:
		if n is Button and (n as Button).text == label and not (n as Button).disabled:
			return n
	return null


## The map area only, downscaled to 192 px wide. `get_pixel()` in GDScript is
## far too slow to walk a 1152x648 frame 70 times (the first cut of this probe
## ran past a 1500 s watchdog on the layer sweep alone); `Image.resize()` is
## native, and an average-filtered 192 px raster still separates a layer that
## repaints the whole map from one that changes nothing.
func _shot() -> Image:
	await RenderingServer.frame_post_draw
	var full := get_viewport().get_texture().get_image()
	var w := full.get_width()
	var h := full.get_height()
	var x0 := int(w * 0.30)
	var crop := full.get_region(Rect2i(x0, 0, w - x0, h))
	crop.resize(192, 108, Image.INTERPOLATE_BILINEAR)
	return crop


## Fraction of pixels that differ, over the map area only (the left 60 % of the
## frame is dock, so the whole-frame figure would be diluted by chrome).
func _differ(a: Image, b: Image) -> float:
	if a == null or b == null or a.get_size() != b.get_size():
		return -1.0
	var w := a.get_width()
	var h := a.get_height()
	var n := 0
	var d := 0
	for y in h:
		for x in w:
			n += 1
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				d += 1
	return 0.0 if n == 0 else 100.0 * float(d) / float(n)


func _generate(seed: int, gw: int, gh: int, km: float) -> void:
	_bridge.generate({
		"seed": seed, "width_km": km, "grid_w": gw, "grid_h": gh,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 1500.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	await _generate(483920, 384, 288, 2400.0)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(8)
	var civ := _find(_app, "civilization_workspace.gd")
	var carto := _find(_app, "cartography_workspace.gd")
	_p("world: %d settlements, %d ways, %d factions" % [
		_bridge.settlements().size(), _bridge.roads().size(), _bridge.get_factions().size()])

	# ======================================================= IN-13 trade flows
	_p("=== IN-13 : CIVIL ▸ Trade ▸ Match trade flows ===")
	_app._select_domain("civilization")
	await _frames(3)
	var trade := _cat(civ, "Trade")
	await _frames(4)
	var match_btn := _find_button(trade, "Match trade flows")
	if match_btn == null:
		_bad("no Match trade flows button")
	else:
		_p("before: disabled=%s; store empty=%s" % [
			str(match_btn.disabled), str(TradeStore.last().is_empty())])
		if match_btn.disabled:
			_bad("Match is disabled over a live world -- RF-01 again")
		var t0 := Time.get_ticks_msec()
		match_btn.pressed.emit()
		await _frames(10)
		var d := TradeStore.last()
		_p("after press: %d ms, flow_count=%d, importing=%d, supplied=%d" % [
			Time.get_ticks_msec() - t0, int(d.get("flow_count", 0)),
			int(d.get("importing", 0)), int(d.get("supplied", 0))])
		if d.is_empty() or int(d.get("flow_count", 0)) <= 0:
			_bad("the match produced no flows")
		var body_txt := _texts(_cat(civ, "Trade"))
		await _frames(3)
		if body_txt.find("Not matched yet") >= 0:
			_bad("the Flows body still says 'Not matched yet' after a real match")

		## The overlay's own half: CARTO ▸ Roads & routes ▸ Trade load, measured
		## in pixels rather than believed.
		_app._select_domain("cartography")
		await _frames(3)
		var rr := _cat(carto, "Roads & routes")
		await _frames(6)
		var off_img := await _shot()
		var toggles: Array = []
		_walk(rr, toggles)
		var load_cb: CheckBox = null
		for n in toggles:
			if n is CheckBox:
				var p := (n as Node).get_parent()
				for s in p.get_children():
					if s is Label and String((s as Label).text).findn("Thicken ways") >= 0:
						load_cb = n
		if load_cb == null:
			_bad("no 'Trade load' toggle in CARTO ▸ Roads & routes")
		else:
			## RF-01's exact shape: this toggle is BUILT at launch, before any
			## world, and disables itself when `has_trade_load()` is false. The
			## match that makes it valid happens in a different workspace.
			_p("Trade load toggle after a real match: disabled=%s  overlay has data=%s" % [
				str(load_cb.disabled), str(_app.viewport.overlay.has_trade_load())])
			if load_cb.disabled and _app.viewport.overlay.has_trade_load():
				_bad("RF-01: the Trade load toggle is still disabled after a real "
					+ "match produced way volumes -- nothing re-enables it")
			load_cb.button_pressed = true
			load_cb.toggled.emit(true)
			await _frames(8)
			var on_img := await _shot()
			var moved := _differ(off_img, on_img)
			_p("Trade load ON moved %.4f %% of map pixels" % moved)
			if moved <= 0.0001:
				_bad("Trade load changed no pixels at all")
			load_cb.button_pressed = false
			load_cb.toggled.emit(false)
			await _frames(8)
			var back_img := await _shot()
			var back := _differ(off_img, back_img)
			_p("Trade load OFF returned to %.4f %% differing" % back)
			if back > 0.05:
				_bad("Trade load OFF did not return the map (%.4f %%)" % back)

		## RF-01: a new world renumbers every settlement and way the match names.
		await _generate(771155, 256, 192, 900.0)
		await _frames(8)
		_app._select_domain("civilization")
		await _frames(3)
		var trade2 := _cat(civ, "Trade")
		await _frames(5)
		var txt2 := _texts(trade2)
		var m2 := _find_button(trade2, "Match trade flows")
		_p("after regenerate: store empty=%s, body says 'Not matched yet'=%s, button disabled=%s" % [
			str(TradeStore.last().is_empty()),
			str(txt2.find("Not matched yet") >= 0),
			str(m2.disabled) if m2 != null else "?"])
		if not TradeStore.last().is_empty():
			_bad("the previous world's trade match survived a generate")
		if txt2.find("Not matched yet") < 0:
			_bad("the Flows body still shows the previous world's numbers")
		if m2 != null and m2.disabled:
			_bad("Match is disabled over the new world -- RF-01")

	# ========================================================= ED-02 the ledger
	_p("=== ED-02 : right dock ▸ History ===")
	_app.right_dock_ctrl.show_history()
	await _frames(8)
	var hist := _texts(_app.right_dock_body)
	var rows := 0
	for line in hist.split("\n"):
		if String(line).find("▲") >= 0 or String(line).find("·") >= 0 or String(line).find("◼") >= 0:
			rows += 1
	_p("history rows carrying a state glyph: %d" % rows)
	if hist.strip_edges() == "":
		_bad("the History dock drew nothing")
	## The floor row must name the world actually on screen, not the last one --
	## §42 found exactly that (`seed 0` against a status bar reading 483920).
	if hist.find("771155") < 0:
		_p("   note: history does not name seed 771155; dumping rows")
		for line in hist.split("\n"):
			if String(line).strip_edges() != "":
				_p("   hist> %s" % line)
	else:
		_p("PASS  the ledger floor names the live seed 771155")

	# ================================================== CV-23 contested borders
	_p("=== CV-23 : CIVIL ▸ Territories ▸ Borders & influence, then the layer ===")
	_app._select_domain("civilization")
	await _frames(3)
	var terr := _cat(civ, "Territories")
	await _frames(5)
	var bi := _find_button(terr, "Borders & influence")
	if bi == null:
		bi = _find_button(terr, "influence")
	if bi == null:
		_p("   (no explicit Borders & influence button -- listing Territories buttons)")
		var tb: Array = []
		_walk(terr, tb)
		for n in tb:
			if n is Button:
				_p("   btn> '%s' disabled=%s" % [(n as Button).text, str((n as Button).disabled)])
	else:
		_p("Borders & influence: disabled=%s" % str(bi.disabled))
		if bi.disabled:
			_bad("Borders & influence is disabled over a live world")
		else:
			bi.pressed.emit()
			await _frames(20)
			_p("after press: %s" % _texts(_cat(civ, "Territories")).substr(0, 300).replace("\n", " / "))

	# ============================================ the Layers popover, in pixels
	_p("=== Layers popover : every entry, measured in pixels ===")
	_app.layers_popover.visible = true
	if _app.layers_popover.has_method("rebuild"):
		_app.layers_popover.call("rebuild")
	await _frames(6)
	var names := _layer_names()
	_p("%d layer entries" % names.size())
	_app.layers_popover.visible = false
	await _frames(4)
	var base := await _shot()
	var same_as_base := 0
	var seen := {}
	for label in names:
		## Re-found by TEXT every iteration: pressing one entry rebuilds the
		## popover, which frees every Button in it -- a cached list makes the
		## sweep silently cover exactly one layer and report the other 34 as
		## `is_instance_valid == false`.
		var b := _layer_button(label)
		if b == null:
			_p("  %-34s (gone after rebuild)" % label)
			continue
		b.pressed.emit()
		await _frames(10)
		_app.layers_popover.visible = false
		await _frames(4)
		var img := await _shot()
		var moved2 := _differ(base, img)
		## Also fingerprint the drawn frame itself, so two DIFFERENT layers that
		## draw the SAME picture are caught -- RD-02's actual failure.
		## Hash the whole 192x108 crop natively rather than sampling a grid of
		## points: the first cut sampled 45 points and every one of the 35 layers
		## hashed identically, because the points happened to land in the 59 % of
		## the frame that never moves. RD-02's failure -- two layers drawing the
		## same picture -- is only catchable on the whole raster.
		var key := "%d" % hash(img.get_data())
		_p("  %-34s moved %6.3f %%  key=%s" % [label, moved2, key])
		if moved2 <= 0.0001:
			same_as_base += 1
			_p("     >> %s drew the same frame as the base map" % label)
		if seen.has(key):
			_bad("layer '%s' draws a pixel-identical frame to '%s'" % [label, seen[key]])
		else:
			seen[key] = label
	_p("%d of %d layer entries changed nothing" % [same_as_base, names.size()])

	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

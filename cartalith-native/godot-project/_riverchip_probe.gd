extends Node
## Lane B, 2026-09-05: the GIS INCLUDE row's `rivers` chip now carries a real
## count instead of a dash whose stated reason had gone false.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _riverchip_probe.tscn
##
## Three things are under test and they are separable:
##
##   1. `EngineBridge.rivers(min_order)` exists and forwards to the bound
##      `WorldGen::get_rivers`;
##   2. its count at order 2 equals the number of `properties.layer == "river"`
##      features in the document `export_geojson()` actually writes -- the
##      chip agreeing with the receipt is the whole point of the change;
##   3. the order argument is **load-bearing**. Item 2 would pass vacuously if
##      `min_order` were ignored, so the probe also measures order 1 and
##      requires it to disagree with the document. `EXPORT_MIN_RIVER_ORDER` is
##      carried as a literal here rather than read from the code under test.
##
## Both branches of the chip are exercised, because an inverted pair of
## reasons is exactly the defect this batch exists to stop: the live path with
## a generated world, and the empty path by holding `EngineBridge.generating`
## true, which is one of the three absences the dash text names.

const EXPORT_MIN_RIVER_ORDER := 2   ## geojson_bridge.rs's own literal, carried
const HEADWATER_ORDER := 1          ## the control: every trickle, not the export set

var app: Node
var dm: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ck(name: String, cond: bool, detail: String = "") -> void:
	print("  %s %s%s" % ["ok  " if cond else "FAIL", name,
		("   -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _eq(name: String, got, want) -> void:
	_ck(name, str(got) == str(want), "got=%s want=%s" % [str(got), str(want)])

## Every `properties.layer == "river"` feature in a written document.
## `-1` for text that is not a FeatureCollection, so an empty export cannot
## masquerade as a measured zero.
func _doc_rivers(gj: String) -> int:
	if gj.strip_edges() == "":
		return -1
	var parsed = JSON.parse_string(gj)
	if typeof(parsed) != TYPE_DICTIONARY or not (parsed as Dictionary).has("features"):
		return -1
	var n := 0
	for f in (parsed as Dictionary)["features"]:
		if String((f as Dictionary).get("properties", {}).get("layer", "")) == "river":
			n += 1
	return n

func _labels(n: Node, out: Array = []) -> Array:
	for c in n.get_children():
		if c is Label:
			out.append(c)
		_labels(c, out)
	return out

## The drawn chip whose text starts with `rivers`, as `{text, tip}` -- read off
## the scene, not off `_gis_count()`, so a bug in the helper cannot hide here.
func _rivers_chip() -> Dictionary:
	for l in _labels(dm._pane_body):
		var t := String((l as Label).text)
		if t.begins_with("rivers"):
			var wrap := (l as Label).get_parent() as Control
			return {"text": t, "tip": "" if wrap == null else wrap.tooltip_text}
	return {}

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	DccTheme.apply_theme(false)   ## force LIGHT; this machine boots light anyway

	print("[0] surface")
	_ck("EngineBridge.rivers() exists", app.bridge.has_method("rivers"))
	_ck("the loaded cdylib exports WorldGen.get_rivers",
		app.bridge.world_gen.has_method("get_rivers"))

	await _generate()

	print("[1] the count and the document")
	var n2: int = app.bridge.rivers(EXPORT_MIN_RIVER_ORDER).size()
	var n1: int = app.bridge.rivers(HEADWATER_ORDER).size()
	var gj: String = app.bridge.export_geojson()
	var n_doc := _doc_rivers(gj)
	_ck("export_geojson wrote a FeatureCollection", n_doc >= 0,
		"%d bytes" % gj.to_utf8_buffer().size())
	_ck("the document carries rivers at all", n_doc > 0, "river features=%d" % n_doc)
	_eq("rivers(%d).size() == river features in the file" % EXPORT_MIN_RIVER_ORDER,
		n2, n_doc)
	_ck("min_order is load-bearing, not ignored", n1 != n2,
		"order %d = %d runs, order %d = %d runs" % [HEADWATER_ORDER, n1,
			EXPORT_MIN_RIVER_ORDER, n2])
	_ck("passing the headwater order WOULD disagree with the file", n1 != n_doc,
		"order %d = %d vs %d in the file" % [HEADWATER_ORDER, n1, n_doc])

	print("[2] the chip on screen")
	dm = app.data_manager_window
	dm.open_route("export_gis")
	await _frames(4)
	var chip := _rivers_chip()
	_ck("a rivers chip is drawn", not chip.is_empty())
	_eq("its text is the real count", String(chip.get("text", "")),
		"rivers %d" % n_doc)
	_ck("it is not dashed", String(chip.get("text", "")).find("—") == -1,
		String(chip.get("text", "")))
	var tip := String(chip.get("tip", ""))
	_ck("its tooltip no longer claims there is no forwarder",
		tip.find("no forwarder") == -1 and tip.find("One line away") == -1,
		tip.substr(0, 90))

	print("[3] both branches of _gis_count")
	var live: Dictionary = dm._gis_count("rivers")
	_ck("live branch returns a count", live.has("count"))
	_eq("and it is the document's number", int(live.get("count", -1)), n_doc)
	_ck("with a how, not a why", live.has("how") and not live.has("why"))

	## The empty path, driven through the forwarder's own mid-generation
	## refusal -- one of the three absences the dash text names.
	app.bridge.generating = true
	var dashed: Dictionary = dm._gis_count("rivers")
	app.bridge.generating = false
	_ck("empty branch omits the key rather than returning 0",
		not dashed.has("count"), str(dashed.get("count", "<absent>")))
	_ck("and gives a reason", dashed.has("why"))
	## The retired claim, not the word: the new text says the forwarder
	## *refuses* mid-generation, which is true and uses the same noun.
	var why := String(dashed.get("why", ""))
	_ck("whose reason is not the retired one",
		why.find("has no forwarder") == -1 and why.find("One line away") == -1,
		why.substr(0, 90))
	_ck("and names an absence it can actually be in",
		why.find("generation is in flight") != -1, why.substr(0, 90))
	_eq("the live branch still works after restoring generating",
		int(dm._gis_count("rivers").get("count", -1)), n_doc)

	print("=== RIVER CHIP ", "OK" if _fail == 0 else "FAILED (%d)" % _fail, " ===")
	get_tree().quit(0 if _fail == 0 else 1)

func _generate() -> void:
	app.bridge.generate({"seed": 40417, "width_km": 2000.0, "grid_w": 256,
		"grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while app.bridge.generating and waited < 6000:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	_ck("world generated", app.bridge.has_world,
		"%d settlements" % app.bridge.settlements().size())

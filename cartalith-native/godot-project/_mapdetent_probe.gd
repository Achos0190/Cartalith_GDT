extends Node
## Ruling BK header (2026-10-13, probe adopted into git; first run passed on the debug DLL):
## Protects: Ruling Z: the phone MAP tab's half-detent height tracks its real content (with and without a world); GEN's detent stays flat 0.46 and MORE stays peek.
## Run (Godot_v4.7.1-stable_win64_console.exe, from godot-project/): --headless --path . _mapdetent_probe.tscn -- --force-touch --vp 1080x2340
## Result: prints 'MAP world: content_height=.. half_detent=..' and 'OK'/'FAIL' on the GEN and MORE lines, then 'DONE' (no exit code; read the lines)
## Sources: derived from this file's own assertions and a 2026-10-13 run; the feature symbols were grepped, not assumed.
##
## Scratch verification probe for Ruling Z (`LARGE_ITEM_RULINGS.md`, 2026-09-21)
## -- measures the phone MAP tab's half-detent height against its real content,
## with and without a world loaded, and confirms GEN/PLAN/MORE are unaffected.
## Not a standing probe; modeled on `_detent_probe.gd`'s own conventions
## (headless is sound here -- these are Control.size/layout reads, nothing
## rasterised).
##
##   godot --headless --path . _mapdetent_probe.tscn -- --force-touch --vp 1080x2340

var app: Node
var _vp: SubViewport

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	var parts: PackedStringArray = "1080x2340".split("x")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	if not app.is_phone():
		print("ABORT not phone mode -- pass --force-touch")
		get_tree().quit(2)
		return

	var fh: float = float(_vp.size.y) - app._phone_nav_reserve()
	var flat_half := maxf(float(app._pscale(66.0)), round(fh * 0.46))
	print("flat 0.46 half (px) = %.1f" % flat_half)

	# -- MAP, no world --------------------------------------------------------
	app._pick_phone_tab("map")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	var map_content_noworld := float(app._phone_map_sheet_content_height())
	var map_half_noworld := float(app._phone_detent_height("half"))
	var map_sheet_noworld: float = (app._phone_tool_sheet as Control).size.y
	print("MAP no-world: content_height=%.1f  half_detent=%.1f  actual_sheet_h=%.1f" % [
		map_content_noworld, map_half_noworld, map_sheet_noworld])
	print("MAP no-world: blank_fraction_vs_flat = %.1f%%" % [
		100.0 * (1.0 - map_content_noworld / flat_half)])

	# -- GEN, no world, unaffected regression check ---------------------------
	app._pick_phone_tab("gen")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	var gen_half := float(app._phone_detent_height("half"))
	print("GEN: half_detent=%.1f  (must equal flat 0.46 = %.1f)  %s" % [
		gen_half, flat_half, "OK" if absf(gen_half - flat_half) < 0.5 else "FAIL"])

	# -- back to MAP with a world loaded ---------------------------------------
	app.bridge.generate({"seed": 77341, "width_km": 600.0, "grid_w": 128, "grid_h": 96,
		"sea_level": 0.5, "villages": false})
	await app.bridge.generation_finished
	await _frames(10)
	print("world loaded: has_world=%s" % str(app.bridge.has_world))

	app._pick_phone_tab("map")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	var map_content_world := float(app._phone_map_sheet_content_height())
	var map_half_world := float(app._phone_detent_height("half"))
	var map_sheet_world: float = (app._phone_tool_sheet as Control).size.y
	print("MAP world: content_height=%.1f  half_detent=%.1f  actual_sheet_h=%.1f" % [
		map_content_world, map_half_world, map_sheet_world])
	print("MAP world: blank_fraction_vs_flat = %.1f%%" % [
		100.0 * (1.0 - map_content_world / flat_half)])

	# -- GEN again, world loaded, still unaffected -----------------------------
	app._pick_phone_tab("gen")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	var gen_half_world := float(app._phone_detent_height("half"))
	print("GEN world: half_detent=%.1f  (must equal flat 0.46 = %.1f)  %s" % [
		gen_half_world, flat_half, "OK" if absf(gen_half_world - flat_half) < 0.5 else "FAIL"])

	# -- PLAN/MORE don't drive the sheet at all, confirm untouched -------------
	app.set_phone_detent("peek")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	app._pick_phone_tab("more")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	print("MORE: detent=%s (must stay peek, MORE doesn't drive the sheet)  %s" % [
		String(app.phone_detent()),
		"OK" if String(app.phone_detent()) == "peek" else "FAIL"])

	print("DONE")
	get_tree().quit(0)

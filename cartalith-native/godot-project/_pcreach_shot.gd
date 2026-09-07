extends Node
## **Lane PC-REACH capture leg (2026-09-07).** Reaches the four desktop states
## the previous capture lane recorded as "no shipped side": the CIVIL and CARTO
## rail domains (pairs P03/P04), the CIVIL > Journey planner panel that
## `_ds03fit_probe` reports at 404 px, and the light theme at 1920 (P05).
##
## Headed, not `--headless`: a `SubViewport` texture is null under the dummy
## rasteriser, which is why `_menuconf_probe`'s shots are empty in a headless
## run. Read-only apart from the PNGs it writes to `--out`.
##
##   Godot_v4.7.1 --path . _pcreach_shot.tscn -- --vp 1920x1080 --out <dir>

const SEED := 483920

var app: Node
var _vp: SubViewport
var _dir := "user://pcreach"


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _shot(nm: String) -> void:
	await _frames(6)
	var img := _vp.get_texture().get_image()
	if img == null:
		print("  shot FAILED (null texture) ", nm)
		return
	img.save_png("%s/%s.png" % [_dir, nm])
	print("  shot -> %s.png  %dx%d" % [nm, img.get_width(), img.get_height()])


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var w := 1920
	var h := 1080
	var i := args.find("--vp")
	if i >= 0 and i + 1 < args.size():
		var p: PackedStringArray = String(args[i + 1]).split("x")
		w = int(p[0]); h = int(p[1])
	var j := args.find("--out")
	if j >= 0 and j + 1 < args.size():
		_dir = String(args[j + 1])
	DirAccess.make_dir_recursive_absolute(_dir)

	_vp = SubViewport.new()
	_vp.size = Vector2i(w, h)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(30)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(10)
	print("[MODE] vp=%s theme=%s docks=%s/%s" % [_vp.size,
		("light" if DccTheme.c("bg").r > 0.5 else "dark"),
		app.get("_left_width"), app.get("_right_width")])

	var bridge = app.bridge
	bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.2).timeout
	print("[WORLD] generated")

	var states := [
		["world_a", "world", "a"],
		["civil_landmarks", "civilization", "landmarks"],
		["civil_factions", "civilization", "factions"],
		["civil_planner", "civilization", "planner"],
		["carto_style", "cartography", "style"],
		["carto_terrain", "cartography", "terrain"],
	]
	for s in states:
		app.call("_on_rail_node_pressed", String(s[1]), String(s[2]))
		await _frames(20)
		await _shot(String(s[0]))

	print("### DONE ###")
	get_tree().quit()

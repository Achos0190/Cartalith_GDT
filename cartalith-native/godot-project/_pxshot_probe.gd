extends Node
## Pixel-identity probe for the Pen link (Ruling EK): boots the shell, generates one fixed world, and saves the
## SubViewport as a PNG so two trees can be compared byte for byte. Asserts nothing itself.
## Args: --vp WxH --out PATH
func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	return String(args[k + 1]) if k >= 0 and k + 1 < args.size() else dflt
func _ready() -> void:
	var parts := _arg("--vp", "1600x1000").split("x")
	var vp := SubViewport.new()
	vp.size = Vector2i(int(parts[0]), int(parts[1]))
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	app.bridge.generate({"seed": 24601, "width_km": 1200.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.42})
	while app.bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if "open_project_dialog" in app and app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	for k in 30:
		await get_tree().process_frame
	await get_tree().create_timer(1.0).timeout
	var img := vp.get_texture().get_image()
	img.save_png(_arg("--out", "C:/cw/shot.png"))
	print("SHOT ", img.get_size())
	get_tree().quit(0)

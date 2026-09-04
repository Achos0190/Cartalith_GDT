extends Node
## Constraint 2 of the batch-24 Lane A brief: *desktop must come out
## byte-identical except where it was already rendering outside its slot*.
## Reasoning from the node tree cannot show that (MISTAKES.md: "flip the flag
## and diff the framebuffer"), so this renders the right dock with the Journey
## section appended at a width where EVERY row already fitted before the
## reflow -- 620 px, comfortably past the 456 px the panel used to demand --
## crops the dock's own rect and writes it as a PNG for the caller to hash.
## The 620 px it asks for is not what it gets: at `--resolution 1600x1000` the
## row settles the dock at **597 px** (printed with every capture), which is
## still past 456 and is the width the identical hashes were taken at.
##
## Identical hashes before and after the change is the proof, and the rect
## printed at the shipped width is its positive control -- 456 x 867 at HEAD,
## 280 x 867 with the reflow, so a run that produced identical hashes is known
## to have been capable of showing a difference. Two runs of the same code gave
## identical hashes on all three seeds, so the comparison is not merely noise.
## The minimum-size figures are a separate probe (`_jpwidth_probe.gd`).
##
## Windowed, not headless -- `--headless` renders nothing to diff.
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _jpwshot_probe.tscn

const SEEDS := [483920, 77021, 4242]
const WIDE := 620.0

var app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	var bridge = app.bridge
	var rd = app.right_dock_ctrl
	var shipped_w: float = (app.right_dock as Control).custom_minimum_size.x

	for seed_v in SEEDS:
		bridge.generate({
			"seed": seed_v, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
			"archetype": "", "villages": true, "sea_level": 0.45,
		})
		var waited := 0
		while bridge.generating and waited < 3000:
			await get_tree().process_frame
			waited += 1
		await _frames(10)
		if not bridge.has_world:
			print("JPS seed %d: generate FAILED" % seed_v)
			continue

		var gs: Vector2i = bridge.grid_size()
		bridge.route_begin("mixed")
		bridge.route_append_stop(gs.x * 0.20, gs.y * 0.30)
		bridge.route_append_stop(gs.x * 0.55, gs.y * 0.50)
		bridge.route_append_stop(gs.x * 0.82, gs.y * 0.72)
		bridge.route_commit()

		var settlements: Array = bridge.settlements()
		app.select_domain("civilization")
		await _frames(3)
		app.arm_tool("inspect")
		if not settlements.is_empty():
			rd.on_settlement_selected(settlements[0], 0)
		await _frames(4)
		app.arm_tool("journey")
		await _frames(10)

		## The positive control. At the shipped dock width the geometry is the
		## thing under test, so record the RENDERED rect before widening -- if
		## this number does not move, the harness cannot see the change at all
		## and the identical hashes below would mean nothing.
		var nat: Rect2 = (app.right_dock as Control).get_global_rect()
		print("JPS seed %d  dock rect at shipped width = %.0f x %.0f  (x=%.0f)"
			% [seed_v, nat.size.x, nat.size.y, nat.position.x])

		## Wide enough that nothing in the panel had to reflow before the
		## change -- the widest measured demand was 456 px.
		(app.right_dock as Control).custom_minimum_size.x = WIDE
		await _frames(12)

		var r: Rect2 = (app.right_dock as Control).get_global_rect()
		var img: Image = get_viewport().get_texture().get_image()
		if img == null:
			print("JPS  !! no framebuffer -- run windowed, not --headless")
			get_tree().quit(2)
			return
		var crop := Rect2i(int(r.position.x), int(r.position.y), int(r.size.x), int(r.size.y))
		crop = crop.intersection(Rect2i(Vector2i.ZERO, img.get_size()))
		var sub := img.get_region(crop)
		var out := "user://jps_%d.png" % seed_v
		sub.save_png(out)
		print("JPS seed %d  dock rect=%s  -> %s" % [seed_v, str(crop), ProjectSettings.globalize_path(out)])

		## Put the dock back before the next seed -- the WIDE override is not
		## the shell's own width, and leaving it set made every seed after the
		## first report 620 for its "shipped width" control.
		(app.right_dock as Control).custom_minimum_size.x = shipped_w
		rd.on_settlement_selected(null, -1)
		app.arm_tool("inspect")
		await _frames(4)

	print("JPS DONE")
	get_tree().quit(0)

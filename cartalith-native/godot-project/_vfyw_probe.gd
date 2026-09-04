extends Node
## VERIFIER batch 24, lane A. Independent width + inventory measurement.
## Own seeds, deliberately NOT the lane's 483920 / 77021 / 4242.
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _vfyw_probe.tscn

const SEEDS := [7, 131313, 20260904]

var app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## Every Control in the subtree, with its text if it has one, its class and its
## own minimum x. Visibility printed beside every width (MISTAKES candidate #1
## from the lane's own report: a hidden Control reports a stale minimum).
func _inventory(root: Node, out: Array) -> void:
	for c in root.get_children():
		if c is Control:
			var t := ""
			if c is Button:
				t = (c as Button).text
			elif c is Label:
				t = (c as Label).text
			out.append({
				"k": c.get_class(), "t": t,
				"w": (c as Control).get_combined_minimum_size().x,
				"vis": (c as Control).is_visible_in_tree(),
			})
		_inventory(c, out)

func _expand_all_groups(root: Node) -> int:
	var opened := 0
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is MarginContainer and n.get_child_count() == 1:
			var gb = n.get_child(0)
			if gb is Control and not (gb as Control).visible:
				(gb as Control).visible = true
				opened += 1
		for c in n.get_children():
			stack.append(c)
	return opened

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	var bridge = app.bridge
	var rd = app.right_dock_ctrl
	var body: Control = app.right_dock_body

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
			print("VFYW seed %d: generate FAILED" % seed_v)
			continue

		var gs: Vector2i = bridge.grid_size()
		bridge.route_begin("mixed")
		bridge.route_append_stop(gs.x * 0.20, gs.y * 0.30)
		bridge.route_append_stop(gs.x * 0.55, gs.y * 0.50)
		bridge.route_append_stop(gs.x * 0.82, gs.y * 0.72)
		var ridx: int = bridge.route_commit()
		var settlements: Array = bridge.settlements()
		var sel: Dictionary = settlements[0] if not settlements.is_empty() else {}

		app.select_domain("civilization")
		await _frames(3)
		app.arm_tool("inspect")
		if not sel.is_empty():
			rd.on_settlement_selected(sel, 0)
		await _frames(6)
		app.arm_tool("journey")
		await _frames(14)

		var opened := _expand_all_groups(body)
		await _frames(8)

		var inv: Array = []
		_inventory(body, inv)
		var n_btn := 0
		var n_lbl := 0
		var texts: Array = []
		for e in inv:
			var d: Dictionary = e
			if String(d["k"]) == "Button":
				n_btn += 1
			if String(d["k"]) == "Label":
				n_lbl += 1
			if String(d["t"]) != "":
				texts.append("%s|%s" % [String(d["k"]), String(d["t"])])
		texts.sort()
		var sig := "".join(texts)

		print("VFYW ===== seed %d  route=%d  settlements=%d  groups_opened=%d =====" % [seed_v, ridx, settlements.size(), opened])
		print("VFYW   body min.x=%.0f  scroll min.x=%.0f  right_dock min.x=%.0f  (custom_min=%.0f)"
			% [body.get_combined_minimum_size().x,
			(app._right_dock_scroll as Control).get_combined_minimum_size().x,
			(app.right_dock as Control).get_combined_minimum_size().x,
			(app.right_dock as Control).custom_minimum_size.x])
		print("VFYW   controls=%d  buttons=%d  labels=%d  texted=%d  SIG=%s"
			% [inv.size(), n_btn, n_lbl, texts.size(), sig.md5_text()])
		inv.sort_custom(func(a, b): return float(a["w"]) > float(b["w"]))
		for i in mini(6, inv.size()):
			var f: Dictionary = inv[i]
			print("VFYW   widest %6.0f  vis=%-5s %-16s [%s]" % [float(f["w"]), str(bool(f["vis"])), String(f["k"]), String(f["t"])])
		# every texted control, so a dropped one is detectable across runs
		for t in texts:
			print("VFYW   TXT %s" % t)

	print("VFYW DONE")
	get_tree().quit()

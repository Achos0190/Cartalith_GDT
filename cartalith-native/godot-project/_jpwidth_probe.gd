extends Node
## Lane A / batch 24 measuring harness: how wide is the appended Journey
## results section, and WHICH node inside it demands that width.
##
## Three seeds, because the row this closes was mis-filed from ONE (an empty
## plan at 190 px reported as "no overflow"). Run:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _jpwidth_probe.tscn

const SEEDS := [483920, 77021, 4242]

var app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## Deepest single node accountable for `root`'s combined-minimum x, plus the
## chain that carried it up. Reports the node, not just the number, because
## "351 px" alone cannot tell you what to reflow.
func _blame(root: Control) -> Array:
	var chain: Array = []
	var n: Control = root
	while n != null:
		chain.append("%s%s=%.0f" % [n.get_class(), _tx(n), n.get_combined_minimum_size().x])
		var next: Control = null
		var best := -1.0
		var distributes := (n is HBoxContainer) or (n is HFlowContainer) or (n is GridContainer)
		var sum := 0.0
		for c in n.get_children():
			if not (c is Control) or not (c as Control).visible:
				continue
			var w := (c as Control).get_combined_minimum_size().x
			sum += w
			if w > best:
				best = w
				next = c
		if next == null:
			break
		## For a width-distributing container the blame is the SUM, so name it
		## as such rather than descending into one child and understating it.
		if distributes and n.get_child_count() > 1:
			chain.append("  ^ distributes across %d children, sum=%.0f" % [n.get_child_count(), sum])
		n = next
	return chain

## The text a control carries, so a width has a name instead of an object id.
func _tx(n: Node) -> String:
	if n is Button:
		return "[%s]" % (n as Button).text
	if n is Label:
		return "[%s]" % (n as Label).text
	return ""

## Every leaf whose own minimum x exceeds `floor`, named by its text.
func _fat_leaves(root: Node, floor_px: float, out: Array) -> void:
	for c in root.get_children():
		if c is Control and (c as Control).visible:
			var w := (c as Control).get_combined_minimum_size().x
			var leaf := (c is Label) or (c is Button)
			if leaf and w >= floor_px:
				out.append({"w": w, "t": _tx(c), "k": c.get_class()})
		_fat_leaves(c, floor_px, out)

func _widest_children(body: Control) -> Array:
	var out: Array = []
	for c in body.get_children():
		if not (c is Control):
			continue
		out.append({"w": (c as Control).get_combined_minimum_size().x, "n": c})
	out.sort_custom(func(a, b): return float(a["w"]) > float(b["w"]))
	return out

## Every `_kv_row` in the tree: which side got the wrapping/expanding slot.
## Two Labels in an HBox is that row's exact shape and nothing else in the
## panel has it.
func _kv_rows(root: Node, out: Array) -> void:
	for c in root.get_children():
		if c is HBoxContainer:
			var labels: Array = []
			for gc in c.get_children():
				if gc is Label:
					labels.append(gc)
			if labels.size() == 2 and c.get_child_count() == 2:
				var a: Label = labels[0]
				var b: Label = labels[1]
				var which := "label" if a.autowrap_mode != TextServer.AUTOWRAP_OFF else ("value" if b.autowrap_mode != TextServer.AUTOWRAP_OFF else "NEITHER")
				out.append({"w": c.get_combined_minimum_size().x, "l": a.text, "v": b.text, "which": which,
					"lw": a.get_combined_minimum_size().x, "vw": b.get_combined_minimum_size().x,
					"vis": c.is_visible_in_tree()})
		_kv_rows(c, out)

## Click every collapsed group open, because a user will.
func _expand_all(root: Node, out: Array) -> void:
	for c in root.get_children():
		if c is Button and String((c as Button).text).begins_with(DccIcons.SYMBOLS["expand"]):
			out.append(c)
		_expand_all(c, out)

func _headers() -> Array:
	var out: Array = []
	var stack: Array = [app.right_dock_body]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Label and String((n as Label).text).begins_with("§ "):
			out.append(String((n as Label).text).substr(2))
		for c in n.get_children():
			stack.append(c)
	return out

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
	print("JPW dock role w_right_dock=%d  laptop_override=%s  W_DOCK_TABLET=%d"
		% [DccTheme.role_px("w_right_dock"), DccTheme.is_laptop(), DccTheme.W_DOCK_TABLET])

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
			print("JPW seed %d: generate FAILED" % seed_v)
			continue

		var gs: Vector2i = bridge.grid_size()
		bridge.route_begin("mixed")
		bridge.route_append_stop(gs.x * 0.20, gs.y * 0.30)
		bridge.route_append_stop(gs.x * 0.55, gs.y * 0.50)
		bridge.route_append_stop(gs.x * 0.82, gs.y * 0.72)
		var ridx: int = bridge.route_commit()

		var settlements: Array = bridge.settlements()
		var sel: Dictionary = settlements[0] if not settlements.is_empty() else {}

		# -- selection alone -------------------------------------------------
		app.select_domain("civilization")
		await _frames(3)
		app.arm_tool("inspect")
		if not sel.is_empty():
			rd.on_settlement_selected(sel, 0)
		await _frames(6)
		var w_sel := body.get_combined_minimum_size().x

		# -- Journey armed on top --------------------------------------------
		app.arm_tool("journey")
		await _frames(14)
		var w_both := body.get_combined_minimum_size().x
		var w_scroll: float = (app._right_dock_scroll as Control).get_combined_minimum_size().x
		var w_dock: float = (app.right_dock as Control).get_combined_minimum_size().x

		print("JPW ===== seed %d  route=%d  settlements=%d ============" % [seed_v, ridx, settlements.size()])
		print("JPW   headers=%s" % [_headers()])
		print("JPW   body min.x  selection-only=%.0f   selection+journey=%.0f" % [w_sel, w_both])
		print("JPW   scroll min.x=%.0f   right_dock min.x=%.0f   (custom_min=%.0f)"
			% [w_scroll, w_dock, (app.right_dock as Control).custom_minimum_size.x])
		var tops := _widest_children(body)
		for i in mini(4, tops.size()):
			var e: Dictionary = tops[i]
			print("JPW   top%d  %.0f  %s(%s)" % [i, float(e["w"]), (e["n"] as Node).get_class(), (e["n"] as Node).name])
		if not tops.is_empty():
			for line in _blame(tops[0]["n"]):
				print("JPW     | %s" % line)
		var fat: Array = []
		_fat_leaves(body, 150.0, fat)
		fat.sort_custom(func(a, b): return float(a["w"]) > float(b["w"]))
		for i in mini(12, fat.size()):
			var f: Dictionary = fat[i]
			print("JPW   fat %6.0f  %-6s %s" % [float(f["w"]), String(f["k"]), String(f["t"])])

		var rows: Array = []
		_kv_rows(body, rows)
		rows.sort_custom(func(a, b): return float(a["w"]) > float(b["w"]))
		var neither := 0
		for r in rows:
			if String((r as Dictionary)["which"]) == "NEITHER":
				neither += 1
		print("JPW   kv rows=%d  none-bound=%d" % [rows.size(), neither])
		for i in mini(4, rows.size()):
			var r2: Dictionary = rows[i]
			print("JPW   kv %6.0f  wrap=%-6s vis=%s  l=%.0f v=%.0f  [%s] / [%s]" % [float(r2["w"]), String(r2["which"]),
				str(bool(r2["vis"])), float(r2["lw"]), float(r2["vw"]), String(r2["l"]), String(r2["v"])])

		# -- every collapsible group opened, because a user will open them ----
		var btns: Array = []
		_expand_all(body, btns)
		var opened := 0
		for b2 in btns:
			var par: Node = (b2 as Node).get_parent()
			var idx: int = (b2 as Node).get_index()
			if idx + 1 < par.get_child_count():
				var pad2: Node = par.get_child(idx + 1)
				if pad2 is MarginContainer and pad2.get_child_count() == 1:
					var gb: Control = pad2.get_child(0)
					if not gb.visible:
						gb.visible = true
						opened += 1
		await _frames(8)
		print("JPW   EXPANDED (%d groups, %d were closed):  body min.x=%.0f  scroll=%.0f  right_dock=%.0f"
			% [btns.size(), opened, body.get_combined_minimum_size().x,
			(app._right_dock_scroll as Control).get_combined_minimum_size().x,
			(app.right_dock as Control).get_combined_minimum_size().x])
		var tops2 := _widest_children(body)
		for i in mini(3, tops2.size()):
			var e2: Dictionary = tops2[i]
			print("JPW   exp-top%d  %.0f  %s(%s)" % [i, float(e2["w"]), (e2["n"] as Node).get_class(), (e2["n"] as Node).name])
		var fat2: Array = []
		_fat_leaves(body, 200.0, fat2)
		fat2.sort_custom(func(a, b): return float(a["w"]) > float(b["w"]))
		for i in mini(6, fat2.size()):
			var f2: Dictionary = fat2[i]
			print("JPW   exp-fat %6.0f  %-6s %s" % [float(f2["w"]), String(f2["k"]), String(f2["t"])])

		rd.on_settlement_selected(null, -1)
		app.arm_tool("inspect")
		await _frames(4)

	print("JPW DONE")
	get_tree().quit(0)

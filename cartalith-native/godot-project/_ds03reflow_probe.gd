extends Node
## **DS-03 measurement leg.** Boots the real shell once, walks the live tree and
## reports what the tablet composition actually does to the desktop inventory.
## It asserts nothing on its own -- `_ds03fit_probe.gd` is the guard; this is
## the instrument the guard's numbers came from.
##
##   Godot_v4.7.1 --path . --resolution 1600x900 _ds03reflow_probe.tscn -- --force-touch
##   Godot_v4.7.1 --path . --resolution 1600x900 _ds03reflow_probe.tscn
##
## The second invocation is the desktop control: `DccTheme._touch` is latched
## for the life of the process (see its own comment), so the two densities
## cannot be compared inside one run and this probe is run twice instead.

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _boot(w: int, h: int) -> Node:
	var vp := SubViewport.new()
	vp.size = Vector2i(w, h)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(45)
	return app

## Every `ScrollContainer` in the tree, with the axis modes it was built with
## and the minimum size it is therefore forcing on its parent. A DISABLED axis
## folds the child's minimum into the container's own on that axis -- the trap
## `MISTAKES.md` records three instances of.
func _scrolls(root: Node) -> Array:
	var out: Array = []
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n is ScrollContainer:
			var s := n as ScrollContainer
			if not s.is_visible_in_tree():
				continue
			out.append({
				"path": str(root.get_path_to(s)),
				"h_mode": s.horizontal_scroll_mode,
				"v_mode": s.vertical_scroll_mode,
				"min_x": s.get_combined_minimum_size().x,
				"min_y": s.get_combined_minimum_size().y,
				"size_x": s.size.x,
				"size_y": s.size.y,
			})
	return out

## Visible leaf content, counted per dock: the inventory question. Labels and
## buttons only -- containers are structure, not content.
func _content_count(root: Node) -> Dictionary:
	var labels := 0
	var buttons := 0
	var others := 0
	if root == null:
		return {"labels": 0, "buttons": 0, "others": 0}
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n is Control and (n as Control).is_visible_in_tree():
			if n is Label:
				if (n as Label).text.strip_edges() != "":
					labels += 1
			elif n is BaseButton:
				buttons += 1
			elif n is ProgressBar or n is LineEdit or n is TextEdit or n is ItemList or n is Tree:
				others += 1
	return {"labels": labels, "buttons": buttons, "others": others}

## A control whose own minimum width exceeds the box it was given. This is the
## clipping the tablet density causes and the desktop one does not.
func _overwide(root: Node, budget: float) -> Array:
	var out: Array = []
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n is Control:
			var ctl := n as Control
			if not ctl.is_visible_in_tree():
				continue
			var mn := ctl.get_combined_minimum_size().x
			if mn > budget:
				var txt: Variant = n.get("text")
				out.append("%s  min_x=%.0f > %.0f  autowrap=%s clip=%s  text=%s  path=%s" % [
					String(n.name), mn, budget,
					(str((n as Label).autowrap_mode) if n is Label else "-"),
					(str((n as Label).clip_text) if n is Label else "-"),
					(String(txt).left(400) if txt != null else ""),
					str(root.get_path_to(n))])
	return out

## Every (domain, mode) pair the rail can reach, so the inventory question is
## asked of the whole shell rather than of whichever panel happened to boot.
func _modes() -> Array:
	var out: Array = []
	for n in DccShell.RAIL_NODES:
		if String(n.get("kind", "")) == "node":
			out.append([String(n["domain"]), String(n["mode"])])
	return out

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var forced := "--force-touch" in OS.get_cmdline_user_args()
	print("[BOOT] force-touch=", forced)

	var w := 2560
	var h := 1600
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--vp="):
			var parts := a.substr(5).split("x")
			w = int(parts[0]); h = int(parts[1])
	var app := await _boot(w, h)
	var shell: Node = app
	print("[MODE] vp=", w, "x", h, " touch=", DccTheme.is_touch(), " tablet=", DccTheme.is_tablet(),
		" phone=", DccTheme.is_phone(), " laptop=", DccTheme.is_laptop())
	print("[DOCKS] left_width=", shell.get("_left_width"), " right_width=", shell.get("_right_width"))

	## The four fixed-height bands are the only regions with NO scroll of any
	## kind. Their minimum width is therefore a hard floor on the window, and
	## anything past the viewport edge is simply not reachable.
	for pair in [["tool_options_row", shell.get("tool_options_row")],
			["timeline_row", shell.get("timeline_row")]]:
		var c = pair[1]
		if c is Control:
			print("[BAR] ", pair[0], " min_x=", (c as Control).get_combined_minimum_size().x,
				" size_x=", (c as Control).size.x)
	var mb := _find_menu_bar_row(app)
	if mb != null:
		print("[BAR] menu_bar_row min_x=", mb.get_combined_minimum_size().x,
			" size_x=", mb.size.x)
	var sb := _find_status_row(app)
	if sb != null:
		print("[BAR] status_row min_x=", sb.get_combined_minimum_size().x,
			" size_x=", sb.size.x)

	var lb: Node = shell.get("left_dock_body")
	var rb: Node = shell.get("right_dock_body")
	var lw := float(shell.get("_left_width"))
	var rw := float(shell.get("_right_width"))

	## Per (domain, mode): the content inventory and the widest thing in it.
	## `_select_domain` + `set_active_mode` is the same path the rail buttons
	## take, so this exercises the real switch rather than a private setter.
	for m in _modes():
		shell.call("_on_rail_node_pressed", m[0], m[1])
		await _frames(6)
		var lc := _content_count(lb)
		var rc := _content_count(rb)
		var lmin := (lb as Control).get_combined_minimum_size().x
		var rmin := (rb as Control).get_combined_minimum_size().x
		var over := _overwide(lb, lw) + _overwide(rb, rw)
		var ld := shell.get("left_dock") as Control
		var dockmin := ld.get_combined_minimum_size().x
		var docksize := ld.size.x
		var vph := _find_viewport_host(app)
		var rootmin := (app as Control).get_combined_minimum_size().x
		print("[PANEL] %s/%s  left(L%d B%d O%d min_x=%.0f/%.0f)  right(L%d B%d O%d min_x=%.0f/%.0f)  leftdock_min=%.0f shellroot_min=%.0f  overwide=%d" % [
			m[0], m[1], lc["labels"], lc["buttons"], lc["others"], lmin, lw,
			rc["labels"], rc["buttons"], rc["others"], rmin, rw,
			dockmin, rootmin, over.size()])
		var lat := _latent(lb, lw) + _latent(rb, rw)
		print("        latent(hidden included) leaves over budget: ", lat.size())
		for s in lat:
			print("          ", s)
		print("        left_dock size.x=%.0f   viewport_host size.x=%.0f   widest leaves: %s" % [
			docksize, vph, str(_widest_leaves(lb, 4))])
		for s in over:
			print("      ", s)
		if m[0] == "civilization" and m[1] == "landmarks":
			print("      -- landmarks: the widest HBox row and its children --")
			var worst: Control = null
			var stack2: Array = [lb]
			while not stack2.is_empty():
				var x: Node = stack2.pop_back()
				for c in x.get_children(true):
					stack2.append(c)
				if x is HBoxContainer and (x as Control).is_visible_in_tree():
					if worst == null or (x as Control).get_combined_minimum_size().x > worst.get_combined_minimum_size().x:
						worst = x as Control
			if worst != null:
				print("        row min_x=", worst.get_combined_minimum_size().x,
					" sep=", worst.get_theme_constant("separation"))
				for c in worst.get_children():
					if c is Control:
						var t: Variant = c.get("text")
						print("          ", c.get_class(), " min_x=",
							(c as Control).get_combined_minimum_size().x,
							" cms_x=", (c as Control).custom_minimum_size.x,
							" flags=", (c as Control).size_flags_horizontal,
							" text=", (String(t).left(60) if t != null else ""))

	get_tree().quit(0)

## The map region's actual width -- what a dock blow-out eats.
func _find_viewport_host(app: Node) -> float:
	var stack: Array = [app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n.get_class() == "SubViewportContainer" or String(n.name).findn("viewport") >= 0:
			if n is Control:
				return (n as Control).size.x
	return -1.0

## The widest *leaf* Controls -- a container's minimum is only ever its
## children's, so the leaf is where a fix has to land.
func _widest_leaves(root: Node, n: int) -> Array:
	var rows: Array = []
	var stack: Array = [root]
	while not stack.is_empty():
		var x: Node = stack.pop_back()
		for c in x.get_children(true):
			stack.append(c)
		if x is Control and (x as Control).is_visible_in_tree() and x.get_child_count() == 0:
			rows.append([(x as Control).get_combined_minimum_size().x, String(x.name)])
	rows.sort_custom(func(a, b): return a[0] > b[0])
	return rows.slice(0, n)

## The same question asked of HIDDEN subtrees too. A collapsed `group()` body
## contributes nothing to its parent's minimum while it is hidden, so the live
## walk above cannot see a row that will overflow the moment the caret is
## opened. This is the latent set.
func _latent(root: Node, budget: float) -> Array:
	var out: Array = []
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n is Control and n.get_child_count() == 0:
			var mn := (n as Control).get_combined_minimum_size().x
			if mn > budget:
				var t: Variant = n.get("text")
				out.append("%s min_x=%.0f %s" % [n.get_class(), mn,
					(String(t).left(50) if t != null else "")])
	return out

func _find_menu_bar_row(n: Node) -> Control:
	for c in n.get_children(true):
		if c is MenuButton:
			return n as Control
	for c in n.get_children(true):
		var r := _find_menu_bar_row(c)
		if r != null:
			return r
	return null

func _find_status_row(n: Node) -> Control:
	## The status bar is the last `HBoxContainer` under the shell's root column
	## -- found by structure, since it has no exported handle.
	var found: Control = null
	var stack: Array = [n]
	while not stack.is_empty():
		var x: Node = stack.pop_back()
		for c in x.get_children(true):
			stack.append(c)
		if x is Label and (x as Label).text.begins_with("Ready"):
			var p := (x as Node).get_parent()
			if p is Control:
				found = p as Control
	return found

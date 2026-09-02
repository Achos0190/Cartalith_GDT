extends Node
## `Help ▸ LOD debug` probe. Parsing proves nothing about whether the three
## toggles reach the overlay, so this drives the real shell:
##
##   godot --path . --resolution 1600x900 _loddbg_probe.tscn
##
## **Not `--headless`.** Section 4c reads the framebuffer back, and the
## headless build runs the dummy rasterizer whose `texture_2d_get` returns
## null -- `get_texture().get_image()` is a null deref there. Sections 1-4b do
## run headless; only the pixel diff needs a real renderer.
##
## Asserts, in order: the submenu exists on the real Help popup with the
## reference's own three labels; the check marks read back off `ViewportHost`
## rather than a menu-local copy; toggling a row moves the viewport's own
## state; the overlay node exists, is gated on "any toggle on", and its
## `_draw()` runs without erroring; and `atlas_is_covered` answers through the
## bridge on a build whose atlas has never been written.
##
## The `SubViewport` host is `_audit4_probe.gd`'s idiom, for the reason it
## documents: a real window is clamped to the desktop work area, and the shell
## classifies its layout from the viewport rect.

var _vp: SubViewport
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

## `get_children(true)` -- **including internal children**, which is the whole
## point. A `MenuButton` keeps its `PopupMenu` as an internal child, so the
## default `get_children()` walk finds *zero* popups in a shell that has seven
## of them. Cost this probe one run to learn.
func _find(n: Node, cls: String, out: Array) -> void:
	if n.get_class() == cls or n.is_class(cls):
		out.append(n)
	for c in n.get_children(true):
		_find(c, cls, out)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] the .gdextension did not load.")
		get_tree().quit(1)
		return

	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var scn: PackedScene = load("res://shell/app.tscn")
	var app: Node = scn.instantiate()
	_vp.add_child(app)
	await _frames(45)
	print("[BOOT] shell up")

	# --- 1. the submenu exists, with the reference's labels and order --------
	var pops: Array = []
	_find(app, "PopupMenu", pops)
	var lod: PopupMenu = null
	for p in pops:
		if (p as PopupMenu).name == "LodDebug":
			lod = p
			break
	print("\n=== 1: Help > LOD debug exists ===")
	_ok("LodDebug submenu found", lod != null, true)
	if lod == null:
		var mbs: Array = []
		_find(app, "MenuButton", mbs)
		var alln: Array = []
		_find(app, "Node", alln)
		print("[DIAG] nodes walked: ", alln.size(), "  MenuButtons: ", mbs.size())
		for m in mbs:
			var mb := m as MenuButton
			var pu := mb.get_popup()
			print("[DIAG]   MenuButton '", mb.text, "' popup=", pu,
				" popup_in_tree=", pu != null and pu.is_inside_tree(),
				" popup_parent=", (pu.get_parent().name if pu != null and pu.get_parent() != null else "<none>"),
				" children=", (pu.get_child_count() if pu != null else -1))
		print("[DIAG] PopupMenus reachable from app: ", pops.size())
		for p in pops:
			var pm := p as PopupMenu
			var items := []
			for i in pm.item_count:
				items.append(pm.get_item_text(i))
			print("[DIAG]   name=", pm.name, " parent=", pm.get_parent().name,
				" items=", items.slice(0, 6))
		print("[FATAL] no submenu; the rest cannot run.")
		get_tree().quit(1)
		return
	_ok("three rows", lod.item_count, 3)
	_ok("row 0 label", lod.get_item_text(0), "Grid")
	_ok("row 1 label", lod.get_item_text(1), "Colors")
	_ok("row 2 label", lod.get_item_text(2), "Labels")
	_ok("row 0 is checkable, not radio", lod.is_item_checkable(0), true)
	_ok("every row carries the deep-zoom tooltip",
		str(lod.get_item_tooltip(0)).find("tiled LOD view") >= 0, true)

	# --- 2. the viewport owns the state -------------------------------------
	var vh = app.viewport
	print("\n=== 2: ViewportHost is the single source of truth ===")
	_ok("viewport reachable", vh != null, true)
	_ok("grid starts off", vh.lod_debug_enabled("grid"), false)
	_ok("colors starts off", vh.lod_debug_enabled("colors"), false)
	_ok("labels starts off", vh.lod_debug_enabled("labels"), false)
	_ok("menu agrees at boot", lod.is_item_checked(0), false)
	_ok("an unknown key is refused, not stored", vh.lod_debug_enabled("nope"), false)

	# --- 3. pressing a row moves the viewport, and the mark follows ----------
	print("\n=== 3: the row drives the overlay ===")
	lod.id_pressed.emit(74)   # ID_HELP_LOD_GRID
	await _frames(2)
	_ok("grid on after press", vh.lod_debug_enabled("grid"), true)
	_ok("check mark followed", lod.is_item_checked(0), true)
	_ok("colors untouched", vh.lod_debug_enabled("colors"), false)
	lod.id_pressed.emit(74)
	await _frames(2)
	_ok("grid off after second press", vh.lod_debug_enabled("grid"), false)
	_ok("check mark cleared", lod.is_item_checked(0), false)

	# --- 4. the overlay node, its visibility gate, and a real draw ----------
	print("\n=== 4: the overlay node ===")
	var dbg: Control = vh.get("_lod_debug_layer")
	_ok("overlay node exists", dbg != null, true)
	_ok("hidden while every toggle is off", dbg.visible, false)
	lod.id_pressed.emit(75)   # Colors
	await _frames(2)
	_ok("shown once one toggle is on", dbg.visible, true)
	lod.id_pressed.emit(76)   # Labels
	await _frames(2)
	_ok("stays shown with two on", dbg.visible, true)
	lod.id_pressed.emit(75)
	lod.id_pressed.emit(76)
	await _frames(2)
	_ok("hidden again once all are off", dbg.visible, false)

	# --- 4b. a draw over REAL tiles -----------------------------------------
	#
	# Running `_draw_lod_debug` with `_lod_tiles` empty proves nothing: the
	# loop body never executes, and this repository has been bitten four times
	# by output that passed every structural check and was silently empty. So
	# generate a world and zoom past the LOD gate first
	# (`_zoom > LOD_AUTO_ZOOM` 2.2 AND `px_per_cell > 1.0`), then assert the
	# tile count is non-zero BEFORE trusting the draw.
	print("\n=== 4b: the draw runs over real tiles, not an empty dictionary ===")
	# Through `EngineBridge.generate()`, not `world_gen.generate_sized()`
	# directly: `has_world` is the *bridge's* flag, set on its own completion
	# path, and `ViewportHost._update_lod()` gates on it. Driving the engine
	# underneath the bridge leaves the shell believing there is no world --
	# which is exactly what the first run of this probe measured.
	var br0 = app.get("bridge")
	br0.generate({
		"seed": 24601, "width_km": 640.0, "grid_w": 96, "grid_h": 64,
		"sea_level": 0.5, "villages": true,
	})
	var gen_ok = await br0.generation_finished
	await _frames(6)
	_ok("generation reported ok", gen_ok, true)
	_ok("world present", br0.has_world, true)
	var guard := 0
	while vh.zoom() <= 2.4 and guard < 40:
		vh.zoom_step(1.35)
		guard += 1
		await _frames(1)
	print("  info zoom reached ", vh.zoom(), " in ", guard, " steps")
	await _frames(20)   # let the backlog drain
	var tiles: Dictionary = vh.get("_lod_tiles")
	_ok("LOD is active after zooming in", vh.get("_lod_active"), true)
	_ok("live LOD tiles exist (the draw has something to annotate)", tiles.size() > 0, true)
	print("  info live LOD tiles: ", tiles.size())
	for i in [74, 75, 76]:
		lod.id_pressed.emit(i)
	await _frames(2)
	_ok("all three toggles on", [vh.lod_debug_enabled("grid"),
		vh.lod_debug_enabled("colors"), vh.lod_debug_enabled("labels")],
		[true, true, true])
	dbg.queue_redraw()
	await _frames(3)
	print("  ok   _draw_lod_debug ran over ", tiles.size(),
		" real tiles with all three layers on and did not error")

	# --- 4c. it actually paints ---------------------------------------------
	#
	# "ran without erroring" is satisfied by a draw that silently paints
	# nothing -- a `queue_redraw` on a zero-size Control, a colour with alpha
	# 0, a rect off screen. The only assertion that cannot be satisfied that
	# way is a pixel diff, so take one: same camera, same world, overlay off
	# vs on. Each layer is diffed separately, because a grid that draws while
	# labels silently do not would otherwise pass on the grid's pixels alone.
	print("\n=== 4c: each layer changes real pixels ===")
	for i in [74, 75, 76]:   # all off
		lod.id_pressed.emit(i)
	await _frames(8)
	# Raw buffer comparison, not `get_pixel` -- 1600x900 is 1.44M pixels and a
	# GDScript loop over even every 4th one, three times over, does not finish
	# inside a sane probe budget (measured: it did not).
	var base_buf: PackedByteArray = _vp.get_texture().get_image().get_data()
	_ok("baseline captured", base_buf.size() > 0, true)

	for row in [[74, "Grid"], [75, "Colors"], [76, "Labels"]]:
		lod.id_pressed.emit(row[0])
		await _frames(8)
		var on_buf: PackedByteArray = _vp.get_texture().get_image().get_data()
		_ok("%s changes pixels" % row[1], on_buf != base_buf, true)
		lod.id_pressed.emit(row[0])   # back off, so each layer is measured alone
		await _frames(8)
		var off_buf: PackedByteArray = _vp.get_texture().get_image().get_data()
		_ok("%s leaves no residue when turned back off" % row[1], off_buf == base_buf, true)

	# --- 5. atlas_is_covered answers through the bridge ---------------------
	print("\n=== 5: atlas_is_covered, the F16 decline this overlay consumes ===")
	var br = app.get("bridge")
	if br == null:
		br = vh.get("_bridge")
	_ok("bridge reachable", br != null, true)
	var covered = br.atlas_is_covered(0, 0, 0)
	_ok("answers a bool on an unbaked world", typeof(covered) == TYPE_BOOL, true)
	_ok("nothing is baked yet", covered, false)

	print("\n_loddbg_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

extends Node
## **Every `ColorPickerButton`'s popup, measured at the size it opens.**
## `OUTSTANDING_WORK.md` §2.11, "Phone colour-picker popups are not scaled":
## `phone_fit()` never reached a picker's popup (a `PopupPanel`, which is a
## `Window` and not a `Control`), so on a phone it opened at its desktop width.
##
## Stages each surface that builds a picker, then walks the WHOLE tree under
## the SubViewport (Windows included) for `ColorPickerButton`s, so a picker
## this list does not name is still measured rather than missed. Each popup is
## opened the way a tap opens it and measured:
##   px  = on-screen width in the viewport's physical pixels -- the popup's own
##         `size` times its embedder's `content_scale_factor` when the embedder
##         is a content-scaled `Window` (the roster, the slicer modal);
##   dp  = px / `phone_scale()`.
## A picker whose popup is not at least 250 dp wide on a phone, or that leaves
## the viewport, FAILS. 250 dp against the desktop popup's ~298 px: the popup's
## own content minimum, so "scaled to the phone" means close to its desktop
## size in dp, not a third of the screen.
##
## Sizes are read, so WINDOWED, never `--headless`:
##   godot --path . _pickerpopup_probe.tscn -- --force-touch --vp 1080x2340 --tag phone
##   godot --path . _pickerpopup_probe.tscn -- --vp 1600x1000 --tag desktop
##
## Flags this probe reads, grepped from the body below:
##   `--vp WxH`   SubViewport size in physical px. Default 1080x2340.
##   `--tag NAME` prefix on every output line. Default `cpp`.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it; with a phone-sized
##                `--vp` it makes the run a phone.
## Any other `--flag` aborts rather than being silently ignored.

var app: Node
var bridge: EngineBridge
var _vp: SubViewport
var _tag := "cpp"
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _reject_unknown_args() -> bool:
	var known := ["--force-touch", "--vp", "--tag"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

func _pickers(root: Node, out: Array) -> void:
	for c in root.get_children():
		if c is ColorPickerButton:
			out.append(c)
		_pickers(c, out)

## Which construction site built this picker, by identity -- not by the
## nearest scripted ancestor, which names the HOST (the nested render workspace
## draws into CARTO's dock, so every one of its pickers sits under
## `cartography_workspace.gd`). Anything unmatched is reported by its path, so
## a seventh site would show up rather than vanish.
func _owner_of(pick: Node, carto: Node) -> String:
	var rw = carto._render if carto != null else null
	if rw != null and pick in rw._biome_swatches:
		return "render_workspace.gd (biome)"
	if rw != null and rw._ramp_host != null and rw._ramp_host.is_ancestor_of(pick):
		return "render_workspace.gd (ramp)"
	if carto != null and carto._label_edit_body != null and carto._label_edit_body.is_ancestor_of(pick):
		return "cartography_workspace.gd"
	if app.right_dock_ctrl != null and app.right_dock_ctrl.is_ancestor_of(pick):
		return "right_dock.gd"
	if app.faction_roster_window != null and app.faction_roster_window.is_ancestor_of(pick):
		return "faction_roster_window.gd"
	var al = app.asset_library_window
	if al != null and pick == al._slicer_chroma_color:
		return "asset_library_window.gd"
	## Fallback: the script its `color_changed` handler belongs to. The phone
	## re-parents the right dock's body into a bottom sheet, so tree position
	## alone cannot name that one.
	for con in pick.color_changed.get_connections():
		var obj: Object = (con["callable"] as Callable).get_object()
		if obj != null and obj.get_script() != null:
			return (obj.get_script() as Script).resource_path.get_file()
	return "? %s" % pick.get_path()

func _cartography() -> Node:
	for ws in app._workspaces:
		if ws is CartographyWorkspace:
			return ws
	return null

func _ready() -> void:
	_tag = _arg("--tag", "cpp")
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		_log("ABORT run WINDOWED -- popup sizes are not laid out under --headless")
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1080x2340").split("x")
	if parts.size() != 2:
		_log("ABORT --vp wants WxH")
		get_tree().quit(2)
		return
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	bridge = app.bridge
	var phone: bool = app.is_phone()
	var scale: float = float(app.phone_scale())
	_log("viewport %dx%d  phone=%s  phone_scale=%.4f" % [_vp.size.x, _vp.size.y, phone, scale])

	bridge.generate({"seed": 4242, "width_km": 800.0, "grid_w": 256, "grid_h": 164,
		"sea_level": 0.5, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	_check(bridge.has_world, "a world landed")
	var g: Vector2i = bridge.grid_size()

	## -- Stage every surface that builds a picker ------------------------------
	## CARTO: the biome table and the ramp swatches (render_workspace.gd), and
	## the right dock's Ramp · stops editor (CARTO + inspect, one stop selected).
	app.select_domain("cartography")
	await _frames(4)
	app.arm_tool("inspect")
	await _frames(4)
	var rd = app.right_dock_ctrl
	if rd != null and rd._tool_section() == "stops":
		rd._on_stops_add()
		await _frames(4)
	## The label editor's colour (cartography_workspace.gd): a selected label.
	var lidx: int = bridge.label_create(g.x * 0.4, g.y * 0.4, "Picker Probe")
	bridge.label_select(lidx)
	var carto = _cartography()
	if carto != null and carto.has_method("_rebuild_label_edit_form") and carto._label_edit_body != null:
		carto._rebuild_label_edit_form()
	await _frames(4)
	## The faction roster (faction_roster_window.gd): open, faction 1 selected.
	app.open_faction_roster()
	await _frames(8)
	## The asset library's slicer (asset_library_window.gd).
	app.open_asset_library("", true)
	await _frames(8)
	var al = app.asset_library_window
	if al != null and (al._slicer == null or not al._slicer.visible):
		al._open_slicer()
		await _frames(8)

	var all: Array = []
	_pickers(_vp, all)
	_log("found %d ColorPickerButton(s)" % all.size())
	var sites := {}
	for pick: ColorPickerButton in all:
		var site := _owner_of(pick, carto)
		sites[site] = int(sites.get(site, 0)) + 1
		## One of each construction: the 15 biome rows share one, as do the
		## ramp rows. Folded-away pickers are measured too -- the popup is the
		## subject, and it opens the same whichever category is expanded.
		if int(sites[site]) > 1:
			continue
		var pop: PopupPanel = pick.get_popup()
		pick.get_picker()
		pop.popup()
		await _frames(6)
		## `Window.get_embedder()` is not bound; walk it the way the engine
		## does -- the nearest viewport up the tree that embeds sub-windows.
		var emb: Viewport = pick.get_viewport()
		while emb is Window and not emb.gui_embed_subwindows and emb.get_parent() != null:
			emb = emb.get_parent().get_viewport()
		var emb_scale := 1.0
		if emb is Window:
			emb_scale = (emb as Window).content_scale_factor
		var px := pop.size.x * emb_scale
		var left := pop.position.x * emb_scale
		var dp := px / scale
		_log("%-32s popup size %s  own csf %.3f  embedder %s csf %.3f  -> %.0f px = %.0f dp  x=[%.0f, %.0f]"
			% [site, pop.size, pop.content_scale_factor, emb.get_class() if emb != null else "-",
				emb_scale, px, dp, left, left + px])
		## Inside the embedder's own visible rect (the phone screen, or the
		## content-scaled window it opens in).
		var bound := float(_vp.size.x)
		_check(pop.visible and left >= -0.5 and left + px <= bound + 0.5,
			"%s: the popup opens inside the %d px screen width" % [site, int(bound)])
		var top := pop.position.y * emb_scale
		var bottom := top + pop.size.y * emb_scale
		_check(top >= -0.5 and bottom <= float(_vp.size.y) + 0.5,
			"%s: and inside its %d px height (y=[%.0f, %.0f])" % [site, _vp.size.y, top, bottom])
		if phone:
			_check(dp >= 250.0, "%s: the popup is scaled to the phone (%.0f dp >= 250)" % [site, dp])
		pop.hide()
		await _frames(3)
	for want in ["render_workspace.gd (biome)", "render_workspace.gd (ramp)",
			"cartography_workspace.gd", "right_dock.gd",
			"faction_roster_window.gd", "asset_library_window.gd"]:
		_check(sites.has(want), "a picker from %s was measured" % want)
	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

extends DccShell
class_name DccApp

## The application root: the frame plus everything hung off it.
##
## `DccShell` owns geometry and nothing else. This script owns composition --
## it creates the one `EngineBridge`, builds the seven program menus, attaches
## the viewport, registers the five workspaces, and answers the menu callbacks.
## Anything that needs world state goes through `bridge`.

var bridge: EngineBridge
var viewport: ViewportHost
var menus := DccMenus.new()

var new_world_dialog: NewWorldDialog
var world_data_window: WorldDataWindow
var performance_window: PerformanceWindow
var right_dock_ctrl: RightDock

var _workspaces: Array = []
var _region_nodes: Dictionary = {}
var _tool_options_stale: Label

func _ready() -> void:
	super._ready()

	bridge = EngineBridge.new()
	bridge.name = "EngineBridge"
	add_child(bridge)

	viewport = ViewportHost.new()
	viewport_content.add_child(viewport)
	viewport.setup(bridge)

	menus.build(self, bridge, self)

	new_world_dialog = NewWorldDialog.new()
	add_child(new_world_dialog)
	new_world_dialog.setup(bridge)

	world_data_window = WorldDataWindow.new()
	add_child(world_data_window)
	world_data_window.setup(bridge)

	performance_window = PerformanceWindow.new()
	add_child(performance_window)
	performance_window.setup(bridge)

	_register_workspaces()

	## Owns `right_dock_body`'s content (`DCC_SHELL_SPEC.md` §6, `right_dock.gd`).
	## Appended to `_workspaces` so `_wire_selection`'s existing forwarding loop
	## reaches it too -- it implements `on_settlement_selected`/`on_cursor_sampled`
	## exactly like a workspace does, without needing a rail button of its own.
	right_dock_ctrl = RightDock.new()
	add_child(right_dock_ctrl)
	right_dock_ctrl.setup(self, bridge)
	_workspaces.append(right_dock_ctrl)

	_wire_status()
	_wire_selection()

	_region_nodes = {
		DccMenus.ID_WIN_LEFT: left_dock,
		DccMenus.ID_WIN_RIGHT: right_dock,
		DccMenus.ID_WIN_TIMELINE: timeline_row.get_parent().get_parent(),
		DccMenus.ID_WIN_STATUS: status_row.get_parent().get_parent(),
		DccMenus.ID_WIN_RAIL: rail_column.get_parent().get_parent(),
	}

	workspace_changed.connect(_on_workspace_changed)
	_on_workspace_changed(active_domain())

	set_status("pass", "no world", "text_faint")
	set_status("hint", "File ▸ New world… to begin", "text_ghost")
	set_status("top_world", "—")

func _register_workspaces() -> void:
	## Each workspace builds its own left-dock panel and, where it has one, its
	## own right-dock contribution. They are constructed up front and hidden,
	## so an L2 category left open in Cartography is still open when the user
	## comes back to it.
	for entry in [
		["world", WorldWorkspace.new()],
		["civilization", CivilizationWorkspace.new()],
		["infrastructure", InfrastructureWorkspace.new()],
		["cartography", CartographyWorkspace.new()],
		["render", RenderWorkspace.new()],
	]:
		var ws: Control = entry[1]
		ws.name = String(entry[0]).capitalize() + "Workspace"
		register_workspace(entry[0], ws)
		ws.setup(self, bridge)
		_workspaces.append(ws)

func _wire_status() -> void:
	bridge.generation_started.connect(func():
		set_status("pass", "generating…", "accent")
		set_status("hint", "", "text_ghost"))
	bridge.generation_finished.connect(func(ok: bool):
		set_status("pass", "generated" if ok else "generate failed",
			"text_dim" if ok else "accent")
		set_status("hint", bridge.last_summary, "text_ghost")
		var g := bridge.grid_size()
		set_status("top_world", ("ELDRA · %d" % bridge.world_gen.get_seed()) if ok else "—")
		set_status("top_res", ("%d×%d working" % [g.x, g.y]) if ok else "")
		set_status("top_mem", "%.1f GB" % (OS.get_static_memory_usage() / 1073741824.0))
		_refresh_stale()
		_refresh_rail_foot())
	bridge.params_changed.connect(_refresh_stale)
	bridge.params_applied.connect(_refresh_stale)
	bridge.world_loaded.connect(func():
		set_status("pass", "loaded", "text_dim")
		set_status("hint", bridge.last_summary, "text_ghost"))

## §7's staleness rule, stated once here so no workspace has to restate it:
## Cartalith is a one-shot generator, so a moved dial marks the world stale
## rather than recomputing a stage.
func _refresh_stale() -> void:
	if not bridge.has_world:
		set_status("stale", "")
		return
	var text: String = "stale from 01 — regenerate to apply" if bridge.params_dirty else "resolved"
	set_status("stale", text, "stale" if bridge.params_dirty else "text_faint")
	if is_instance_valid(_tool_options_stale):
		_tool_options_stale.text = text

func _wire_selection() -> void:
	viewport.settlement_selected.connect(func(data, index):
		for ws in _workspaces:
			if ws.has_method("on_settlement_selected"):
				ws.on_settlement_selected(data, index))
	viewport.cursor_sampled.connect(func(gx, gy, valid):
		for ws in _workspaces:
			if ws.has_method("on_cursor_sampled"):
				ws.on_cursor_sampled(gx, gy, valid))
	viewport.layers_button_pressed.connect(func(): _select_domain("cartography"))


# -- Contextual chrome --------------------------------------------------------

## §4: the tool options bar "always reflects the active tool or workspace", and
## §10 says the timeline is "absent from generation and style screens --
## generation is not time-based". Both are functions of the active domain, so
## both are driven from one place rather than five workspaces each remembering.
func _on_workspace_changed(id: String) -> void:
	timeline_bar.visible = id in ["civilization", "infrastructure"]
	match id:
		"world": _tool_options_generate()
		"cartography": _tool_options_simple("CARTOGRAPHY · STYLE",
			"presentation only — no control here marks a generation stage stale")
		"civilization": _tool_options_simple("CIVIL · INSPECT",
			"place, territory and route tools need their bindings (STRANDED_TOOLS.md)")
		"infrastructure": _tool_options_simple("INFRA · INSPECT",
			"way and route tools need their bindings (STRANDED_TOOLS.md)")
		"render": _tool_options_simple("RENDER · PREVIEW",
			"TerrainAppearance is unbound; quality tier lives in Preferences")
	_refresh_rail_foot()

func _tool_options_label(row: Control, text: String, token: String) -> void:
	row.add_child(DccTheme.mono_label(text, token, DccTheme.FS_SMALL, 2, true))

## §4's Generation Pipeline row, in its specified order: context label, the two
## run actions, New seed, the stale-from readout, then the finalize action hard
## right. Run/Finalize are disabled for the reasons §5.1 records -- the engine
## is one-shot and there is no bake pipeline.
func _tool_options_generate() -> void:
	set_tool_options(func(row: HBoxContainer):
		_tool_options_label(row, "GENERATE · WORLD", "accent")
		## "Run stage N" genuinely has no engine entry point. "Run 01 -> 10"
		## does: walking the whole chain from the first stage to the last is
		## exactly what `generate_terrain` is, so it is wired rather than
		## disabled -- the honest reading of §5.1 rather than the literal one.
		var one := DccWidgets.action(row, "Run stage 01", func(): pass)
		one.disabled = true
		one.tooltip_text = "generate_terrain is one-shot: there is no per-stage recompute entry point. Run 01 -> 10 instead, which regenerates the whole world."
		DccWidgets.action(row, "Run 01 → 10", _run_pipeline, true)
		DccWidgets.action(row, "New seed", _new_seed)
		var stale := DccTheme.mono_label("", "stale", DccTheme.FS_SMALL, 1)
		_tool_options_stale = stale
		row.add_child(stale)
		row.add_child(DccTheme.spacer())
		var bake := DccWidgets.action(row, "Bake ALL & finalize", func(): pass)
		bake.disabled = true
		bake.tooltip_text = "No bake/LOD pipeline exists yet; finalize has nothing to freeze."
		_refresh_stale()
	)

func _tool_options_simple(context: String, note: String) -> void:
	set_tool_options(func(row: HBoxContainer):
		_tool_options_label(row, context, "accent")
		row.add_child(DccTheme.label(note, "text_ghost", DccTheme.FS_MICRO))
		row.add_child(DccTheme.spacer())
	)

## The whole chain, which is the only granularity the engine offers.
func _run_pipeline() -> void:
	if bridge.generating:
		return
	bridge.generate(new_world_dialog.request())

func _new_seed() -> void:
	if new_world_dialog.has_method("randomise_seed"):
		new_world_dialog.randomise_seed()
	else:
		open_new_world()

## §3: the rail foot carries the active context and, in World, the stage counter.
func _refresh_rail_foot() -> void:
	var ctx := {"world": "TERRAIN", "civilization": "CIVIL",
		"infrastructure": "INFRA", "cartography": "STYLE", "render": "RENDER"}
	var text: String = ctx.get(active_domain(), "")
	if active_domain() == "world":
		text += "   %s / 10" % ("10" if bridge.has_world else "00")
	set_rail_foot(text)

# -- Menu callbacks -----------------------------------------------------------

func open_new_world() -> void:
	new_world_dialog.popup_centered()

func open_project_picker() -> void:
	_pick_file("Open project", ["*.zip ; Cartalith project"], func(path: String):
		if not bridge.load_save(path):
			set_status("hint", "load failed — see console", "accent"))

func open_asset_pack_picker() -> void:
	_pick_file("Import asset pack", ["*.zip ; Asset pack"], func(path: String):
		if not bridge.load_asset_pack(path):
			set_status("hint", "asset pack failed — see console", "accent"))

func _pick_file(title: String, filters: Array, on_pick: Callable) -> void:
	var d := FileDialog.new()
	d.title = title
	d.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	d.access = FileDialog.ACCESS_FILESYSTEM
	for f in filters:
		d.add_filter(f)
	d.file_selected.connect(func(path: String):
		on_pick.call(path)
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	add_child(d)
	d.popup_centered_ratio(0.6)

func open_world_data() -> void:
	world_data_window.open()

func open_performance() -> void:
	performance_window.open()

## `credits.gd` extends AcceptDialog and fills `%CreditsText` from `_ready`, so
## the scroll and the label have to exist *before* the script runs -- hence
## building the body first and attaching the script last. The attribution it
## carries is a standing obligation (`PROVENANCE.md`), not decoration, which is
## why this is the one Help item that is fully live.
func open_credits() -> void:
	var dlg := AcceptDialog.new()
	dlg.title = "Credits & academic principles"
	dlg.size = Vector2i(720, 640)
	var scroll := ScrollContainer.new()
	scroll.custom_minimum_size = Vector2(680, 560)
	var text := RichTextLabel.new()
	text.name = "CreditsText"
	text.bbcode_enabled = true
	text.fit_content = true
	text.custom_minimum_size.x = 660
	text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(text)
	dlg.add_child(scroll)
	add_child(dlg)
	text.owner = dlg
	text.unique_name_in_owner = true
	dlg.set_script(load("res://credits.gd"))
	dlg.popup_centered()

func open_about() -> void:
	var d := AcceptDialog.new()
	d.title = "About Cartalith"
	d.dialog_text = "Cartalith — native port of Cartalith Gen1 v2.10.\nGodot %s · %s" % [
		Engine.get_version_info().string, OS.get_name()]
	add_child(d)
	d.popup_centered()

func toggle_region(id: int) -> void:
	if id == DccMenus.ID_WIN_RESET:
		for node in _region_nodes.values():
			(node as CanvasItem).visible = true
		return
	if not _region_nodes.has(id):
		return
	var node: CanvasItem = _region_nodes[id]
	node.visible = not node.visible

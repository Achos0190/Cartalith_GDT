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
		set_status("top_world", "seed %d · %d×%d" % [
			bridge.world_gen.get_seed(), g.x, g.y] if ok else "—")
		_refresh_stale())
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
	set_status("stale",
		"parameters changed — regenerate to apply" if bridge.params_dirty else "up to date",
		"stale" if bridge.params_dirty else "text_faint")

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

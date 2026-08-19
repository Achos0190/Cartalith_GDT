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
var journey_planner_view: JourneyPlannerView
var layers_popover: LayersPopover
var right_dock_ctrl: RightDock
var data_manager_window: DataManagerWindow
var asset_library_window: AssetLibraryWindow

## The path `bridge.load_save(path)` last succeeded with, remembered here
## (Godot-side only, no Rust change) so File ▸ Show project on disk and
## Data ▸ Recent worlds have something real to act on --
## `DCC_SHELL_SPEC.md` §2.1. Empty until a project has been opened.
var current_project_path := ""

var _workspaces: Array = []
var _region_nodes: Dictionary = {}
var _tool_options_stale: Label

# -- §4.5 Tool palette ---------------------------------------------------------
#
# One shared `ButtonGroup` across every domain's TOOLS block is the entire
# mechanism `UI_SHELL_DESIGN.md`'s "one tool is armed at a time, globally" and
# "switching workspace never disarms it" both need -- see `DccWidgets.
# tool_button()`'s own doc comment for why. A domain workspace never talks to
# another domain's tool; it registers a click/drag handler under its own tool
# id and never learns whether anyone else's tool exists.

signal tool_armed(id: String)

var tool_group := ButtonGroup.new()
var armed_tool := "inspect"

## tool id -> Callable(gx: float, gy: float). Populated by each workspace's
## own `setup()`. A click/drag while Inspect (or an unregistered id) is armed
## does nothing here -- Inspect's own behaviour is `overlay`'s unconditional
## `settlement_selected` emission, already wired in `_wire_selection()`, not a
## registered handler.
var _click_handlers: Dictionary = {}
var _drag_handlers: Dictionary = {}
var _release_handlers: Dictionary = {}   ## Callable(gx, gy, valid) -- a drag gesture's end.
## tool id -> Callable(), called on Escape instead of the default disarm, for
## a multi-click tool that needs Escape to commit in-progress geometry first
## (§4.5.6: Way, Route). Returning nothing and not disarming is up to the
## handler; most tools don't need one and just fall through to the default.
var _escape_handlers: Dictionary = {}

func arm_tool(id: String) -> void:
	if armed_tool == id:
		return
	armed_tool = id
	tool_armed.emit(id)
	set_status("hint", "" if id == "inspect" else "%s armed — Esc to release" % id.capitalize(), "text_ghost")

func register_tool_click_handler(id: String, handler: Callable) -> void:
	_click_handlers[id] = handler

func register_tool_drag_handler(id: String, handler: Callable) -> void:
	_drag_handlers[id] = handler

func register_tool_escape_handler(id: String, handler: Callable) -> void:
	_escape_handlers[id] = handler

func register_tool_release_handler(id: String, handler: Callable) -> void:
	_release_handlers[id] = handler

func _on_map_clicked(gx: float, gy: float) -> void:
	if _click_handlers.has(armed_tool):
		_click_handlers[armed_tool].call(gx, gy)

func _on_map_dragged(gx: float, gy: float) -> void:
	if _drag_handlers.has(armed_tool):
		_drag_handlers[armed_tool].call(gx, gy)

func _on_map_released(gx: float, gy: float, valid: bool) -> void:
	if _release_handlers.has(armed_tool):
		_release_handlers[armed_tool].call(gx, gy, valid)

## §4.5.6: "Escape commits an in-progress multi-click tool... and otherwise
## disarms back to Inspect." A key, not a mouse button, so it belongs on
## `_unhandled_key_input` regardless of which control has focus.
func _unhandled_key_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		if _escape_handlers.has(armed_tool):
			_escape_handlers[armed_tool].call()
		else:
			var btn: BaseButton = tool_group.get_pressed_button()
			if btn != null:
				btn.button_pressed = false
			arm_tool("inspect")
		get_viewport().set_input_as_handled()

func _ready() -> void:
	super._ready()

	bridge = EngineBridge.new()
	bridge.name = "EngineBridge"
	add_child(bridge)

	viewport = ViewportHost.new()
	viewport_content.add_child(viewport)
	viewport.setup(bridge)

	## Phone chrome sits on top of an edge-to-edge map (§13); `ViewportHost`'s
	## own corner chrome needs to know where that chrome's edges are so it
	## doesn't draw under the app bar/rail/tool sheet. Never runs off the
	## phone path -- `_phone` is false on desktop/tablet. The first call is
	## deferred a frame so the tool sheet has already had its one layout pass
	## when `phone_content_insets()` reads its real height
	## (`_phone_bottom_reserve()`); `phone_insets_changed` covers every later
	## change (rotation, and a domain switch resizing the sheet).
	if _phone:
		(func(): viewport.set_safe_insets(phone_content_insets())).call_deferred()
		phone_insets_changed.connect(func(): viewport.set_safe_insets(phone_content_insets()))

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

	data_manager_window = DataManagerWindow.new()
	add_child(data_manager_window)
	data_manager_window.setup(self, bridge)

	asset_library_window = AssetLibraryWindow.new()
	add_child(asset_library_window)
	asset_library_window.setup(self, bridge)

	layers_popover = LayersPopover.new()
	add_child(layers_popover)
	layers_popover.setup(bridge, viewport)

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
	GlobalTools.install(self)

	_region_nodes = {
		DccMenus.ID_WIN_LEFT: left_dock,
		DccMenus.ID_WIN_RIGHT: right_dock,
		DccMenus.ID_WIN_TIMELINE: timeline_row.get_parent().get_parent(),
		DccMenus.ID_WIN_STATUS: status_row.get_parent().get_parent(),
		DccMenus.ID_WIN_RAIL: rail_column.get_parent().get_parent(),
	}

	workspace_changed.connect(_on_workspace_changed)
	_on_workspace_changed(active_domain())

	## Built and connected last, deliberately after the `workspace_changed`
	## connection two lines up: `JourneyPlannerView` listens to that same
	## signal to reclaim the tool options bar / dock swap after a domain
	## switch (`journey_planner_view.gd`'s own class doc), and GDScript signal
	## handlers run in connection order -- so this must connect after
	## `_on_workspace_changed` for its own re-application to win rather than
	## be immediately overwritten by `_on_workspace_changed`'s INFRA branch.
	journey_planner_view = JourneyPlannerView.new()
	add_child(journey_planner_view)
	journey_planner_view.setup(self, bridge)

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

## There is no staleness state to report: verified live against the reference
## (Playwright, 2026-08-19) that every generation control regenerates the whole
## world automatically on release (`tparam()`'s `change` handler, `withBusy
## ('generating…', generate)`) -- the same mechanism `world_workspace.gd`'s
## slider/toggle rows now trigger. So the only two states worth telling anyone
## about are "a regenerate is running" and "here's how long the last one
## took" -- both live signals, neither an invented dirty flag.
func _wire_status() -> void:
	bridge.generation_started.connect(func():
		set_status("pass", "generating…", "accent")
		set_status("hint", "", "text_ghost")
		if is_instance_valid(_tool_options_stale):
			_tool_options_stale.text = "generating…")
	bridge.generation_finished.connect(func(ok: bool):
		set_status("pass", "generated" if ok else "generate failed",
			"text_dim" if ok else "accent")
		set_status("hint", bridge.last_summary, "text_ghost")
		set_status("stale", ("%.1fs" % (bridge.last_generate_ms / 1000.0)) if ok else "", "text_faint")
		var g := bridge.grid_size()
		set_status("top_world", ("ELDRA · %d" % bridge.world_gen.get_seed()) if ok else "—")
		set_status("top_res", ("%d×%d working" % [g.x, g.y]) if ok else "")
		set_status("top_mem", "%.1f GB" % (OS.get_static_memory_usage() / 1073741824.0))
		if is_instance_valid(_tool_options_stale):
			_tool_options_stale.text = ""
		_refresh_rail_foot())
	bridge.world_loaded.connect(func():
		set_status("pass", "loaded", "text_dim")
		set_status("hint", bridge.last_summary, "text_ghost"))

func _wire_selection() -> void:
	viewport.settlement_selected.connect(func(data, index):
		for ws in _workspaces:
			if ws.has_method("on_settlement_selected"):
				ws.on_settlement_selected(data, index))
	viewport.cursor_sampled.connect(func(gx, gy, valid):
		for ws in _workspaces:
			if ws.has_method("on_cursor_sampled"):
				ws.on_cursor_sampled(gx, gy, valid))
	## §9's layers button opens the canvas Layers popover (the reference's own
	## `#layersPopover`). It used to select the Cartography domain instead --
	## a stand-in for this, since that workspace's left dock is where the only
	## layer controls lived. Nothing it reached is gone: those toggles are
	## still on the rail, and the popover's own footer points at them.
	viewport.layers_button_pressed.connect(func(): layers_popover.open())
	viewport.map_clicked.connect(_on_map_clicked)
	viewport.map_dragged.connect(_on_map_dragged)
	viewport.map_released.connect(_on_map_released)


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
		## Settlement/POI/Territory (civ_tools_bridge.rs) and Way/Route/Measure/
		## Region (infra_tools_bridge.rs) are bound and tested as of 2026-08-19,
		## and §4.5's TOOLS block that arms them now exists in both docks
		## (`civilization_workspace.gd`/`infrastructure_workspace.gd`'s own
		## `_build_tools()`). The earlier wording here claimed the palette "is
		## not built yet" and was stale the moment those two files shipped --
		## both of them say so in their own comments. These strings are only the
		## idle default a domain switch lands on; each workspace reclaims the bar
		## with its own richer row the moment one of its tools arms.
		"civilization": _tool_options_simple("CIVIL · INSPECT",
			"Settlement and Territory tools are armed from the TOOLS block in the dock. POI has no engine call (civ_tools_bridge.rs) and is not offered.")
		"infrastructure": _tool_options_simple("INFRA · INSPECT",
			"Way, Route and Journey are armed from the TOOLS block in the dock; Measure and Region select are global tools.")
		"render": _tool_options_simple("RENDER · PREVIEW",
			"TerrainAppearance is unbound; quality tier lives in Preferences")
	_refresh_rail_foot()

func _tool_options_label(row: Control, text: String, token: String) -> void:
	row.add_child(DccTheme.mono_label(text, token, DccTheme.FS_SMALL, 2, true))

## §4's Generation Pipeline row, in its specified order: context label, the two
## run actions, New seed, the stale-from readout, then the finalize action hard
## right. Run/Finalize are disabled for the reasons §5.1 records -- the engine
## is one-shot and there is no bake pipeline.
## Matches the reference exactly rather than the DCC mockup's own prose, on
## direct owner instruction (2026-08-19) after a live Playwright check of
## `Cartalith Gen1 v2.10.html`: two global buttons (`#genBtn` "Generate
## world", `#reseedBtn` "New seed"), no per-stage anything -- `grep`-checked,
## zero buttons anywhere in the DOM match `/run stage|run \d+.*→/i`. The
## mockup's "Run stage 04 · Run 04 → 10 · stale from 04 Tectonics" describes a
## partial-recompute capability that exists in neither the reference app nor
## this engine; building disabled buttons for it was clutter implying a
## capability that will never exist, not honesty about a gap. Recorded as a
## correction for the design end in `DCC_SHELL_SPEC.md`'s header, the same
## treatment §5.2's stale commit-prose correction already got.
func _tool_options_generate() -> void:
	set_tool_options(func(row: HBoxContainer):
		_tool_options_label(row, "GENERATE · WORLD", "accent")
		DccWidgets.action(row, "Generate world", _run_pipeline, true)
		DccWidgets.action(row, "New seed", _new_seed)
		var busy := DccTheme.mono_label("", "text_ghost", DccTheme.FS_SMALL, 1)
		_tool_options_stale = busy
		row.add_child(busy)
		row.add_child(DccTheme.spacer())
		var bake := DccWidgets.action(row, "Bake ALL & finalize", func(): pass)
		bake.disabled = true
		bake.tooltip_text = "No bake/LOD pipeline exists yet; finalize has nothing to freeze."
		busy.text = "generating…" if bridge.generating else ""
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
		_load_project(path))

## Shared by the file-picker path above and `Data ▸ Recent worlds` / the
## Data manager window's own Import ▸ World Data route -- one place remembers
## `current_project_path` and updates the recent-projects list
## (`DCC_SHELL_SPEC.md` §2.1) so neither caller has to duplicate the
## bookkeeping.
func _load_project(path: String) -> void:
	if bridge.load_save(path):
		current_project_path = path
		DccSettings.remember_project(path)
	else:
		set_status("hint", "load failed — see console", "accent")

## `Data ▸ Recent worlds` submenu entries all call this (`menus.gd`'s
## `_on_recent_world`) -- the exact same load path `open_project_picker()`'s
## own callback uses, just without the file dialog in front of it.
func open_recent_project(path: String) -> void:
	_load_project(path)

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

## `Data`'s five group items (`menus.gd`) all converge here -- `group` is one
## of the five `DataManagerWindow.GROUP_ORDER` names, empty opens the window
## on its first route.
func open_data_manager(group: String = "") -> void:
	data_manager_window.open(group)

## Assets ▸ ⧉ Asset library / ▦ Sprite sheet slicer, and the Icon families ▸ /
## Texture sets ▸ submenus (`menus.gd`) all converge here. `family_key` scopes
## the family rail's selection; `open_slicer` opens the slicer modal on top,
## per §2.3's "opens the library window with the slicer modal already open."
func open_asset_library(family_key: String = "", open_slicer: bool = false) -> void:
	asset_library_window.open(family_key, open_slicer)

# -- Storage locations (`DCC_SHELL_SPEC.md` §2.1, §2.5's "Same modal as File") --

## File ▸ Storage locations and Preferences ▸ Application ▸ Storage
## locations… both call this -- the spec's own "Same modal as File".
##
## Originally two separate dialogs (a read-only list, and a second "Change
## locations…" item with the actual Browse buttons) -- merged into one on
## owner feedback (2026-08-19): showing the same four rows twice across two
## menu items was redundant menu surface, not two distinct capabilities.
## One dialog, one row per root, each with its own Browse… button that
## writes back to `DccSettings` immediately on pick -- no separate confirm
## step, the readout itself is the committed value.
func open_storage_locations() -> void:
	var d := AcceptDialog.new()
	d.title = "Storage locations"
	d.size = Vector2i(680, 340)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 8)
	add_child(d)
	d.add_child(body)

	body.add_child(DccTheme.label(
		"One folder picker per root. Each change saves immediately.", "text_ghost", DccTheme.FS_MICRO))
	body.add_child(DccTheme.rule())

	for key in DccSettings.ROOT_KEYS:
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		row.custom_minimum_size.y = 24
		var lbl := DccTheme.mono_label(String(DccSettings.ROOT_LABELS[key]), "text_dim", DccTheme.FS_SMALL)
		lbl.custom_minimum_size.x = 140
		row.add_child(lbl)
		var readout := DccTheme.mono_label(DccSettings.storage_root(key), "text", DccTheme.FS_SMALL)
		readout.clip_text = true
		readout.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		row.add_child(readout)
		DccWidgets.action(row, "Browse…", func(): _browse_root(key, readout))
		body.add_child(row)
		if key == "atlas_cache":
			## §2.1: "Moving the atlas root invalidates the cache." No tile
			## atlas/cache concept exists in this port yet (Preferences ▸
			## Tiled LOD is itself still _todo, `menus.gd`), so there is
			## nothing to invalidate -- said plainly rather than inventing
			## cache-invalidation logic for a cache that isn't built.
			var cache_note := DccTheme.label(
				"No tile atlas cache exists yet (Preferences ▸ Tiled LOD is not built) -- moving this root has nothing to invalidate.",
				"text_ghost", DccTheme.FS_MICRO)
			cache_note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
			cache_note.custom_minimum_size.x = 560
			body.add_child(cache_note)

	var footnote := DccTheme.label(
		"Defaults derive from OS.get_user_data_dir() -- §2.1's own \"~/Cartalith/...\" paths are macOS-flavored prose that does not hold on every platform this shell runs on.",
		"text_ghost", DccTheme.FS_MICRO)
	footnote.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	footnote.custom_minimum_size.x = 620
	body.add_child(footnote)

	d.popup_centered()

func _browse_root(key: String, readout: Label) -> void:
	var fd := FileDialog.new()
	fd.title = "Choose folder — %s" % String(DccSettings.ROOT_LABELS[key])
	fd.file_mode = FileDialog.FILE_MODE_OPEN_DIR
	fd.access = FileDialog.ACCESS_FILESYSTEM
	var current := DccSettings.storage_root(key)
	if DirAccess.dir_exists_absolute(current):
		fd.current_dir = current
	fd.dir_selected.connect(func(path: String):
		DccSettings.set_storage_root(key, path)
		readout.text = path
		fd.queue_free())
	fd.canceled.connect(func(): fd.queue_free())
	add_child(fd)
	fd.popup_centered_ratio(0.6)

## File ▸ Show project on disk (`DCC_SHELL_SPEC.md` §2.1) -- reveals
## `current_project_path`'s containing folder in the real OS file manager.
## `OS.shell_show_in_file_manager` (Godot 4.4+) is preferred; a version that
## predates it falls back to `shell_open` on the folder URI.
func show_project_on_disk() -> void:
	if current_project_path == "":
		return
	if OS.has_method("shell_show_in_file_manager"):
		OS.shell_show_in_file_manager(current_project_path)
	else:
		OS.shell_open("file://" + current_project_path.get_base_dir())

## `DCC_SHELL_SPEC.md` §4.5.4's 2026-08-19 addition: Journey is an INFRA tool
## takeover, not a dialog -- this arms it exactly like any other tool
## (`journey_planner_view.gd` does the actual region swap, listening to
## `tool_armed`). Both real entry points converge here: `Data ▸ Journey
## planner… ⇧J` (`menus.gd`) and the INFRA dock's own Logistics button
## (`infrastructure_workspace.gd`).
func open_journey_planner() -> void:
	journey_planner_view.open()

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

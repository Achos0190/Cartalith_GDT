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
var resource_overlay: ResourceOverlay
var gen_info_dialog: GenInfoDialog
var journey_planner_view: JourneyPlannerView
var layers_popover: LayersPopover
var right_dock_ctrl: RightDock
var data_manager_window: DataManagerWindow
var asset_library_window: AssetLibraryWindow
var travel_library_window: TravelLibraryWindow
## `GUI_GAP_REGISTER.md` UM-02, the reference's `cityViewerModal`. Long-lived
## for the same reason as its neighbours here -- it keeps a settlement picker
## and a canvas pan/zoom worth holding between opens.
var city_viewer_window: CityViewerWindow
## `placeEditPopup` / `_civPopulatePlaceEditor` and `civFactionsModal` /
## `_civOpenFactionsModal` (`PARITY_AUDIT.md` §5 items 3, 9, 10).
var place_editor_window: PlaceEditorWindow
var faction_roster_window: FactionRosterWindow
## Long-lived, unlike `DccBrowseDialog` (which spawns and frees per pick):
## the gallery holds a scope chip and a search query worth keeping between
## opens, exactly like every other window on this list.
var open_project_dialog: OpenProjectDialog

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

## `_civCtxShow`'s right click. Unlike the three tool primitives above this
## is NOT dispatched by armed tool -- the reference's own menu opens
## regardless of which civ tool is armed (its only gate is "a civ-capable
## tab is open"), so it is broadcast to every workspace that wants it, the
## same shape `settlement_selected`/`cursor_sampled` already use.
func _on_map_right_clicked(gx: float, gy: float, hit: int, screen_pos: Vector2) -> void:
	for ws in _workspaces:
		if ws.has_method("on_map_right_clicked"):
			ws.on_map_right_clicked(gx, gy, hit, screen_pos)

## §4.5.6: "Escape commits an in-progress multi-click tool... and otherwise
## disarms back to Inspect." A key, not a mouse button, so it belongs on
## `_unhandled_key_input` regardless of which control has focus.
##
## Delete joins it for the same reason (`PARITY_AUDIT.md` §5 item 4,
## reference block 2's own keydown at line 26096: Delete removes the
## selected place). Broadcast rather than dispatched, like the right click
## above -- and deliberately *after* the `LineEdit`/`TextEdit` guard below,
## because `_unhandled_key_input` still fires for a focused text field on
## some platforms and deleting a settlement while the user is editing its
## name would be the worst possible surprise.
func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey and event.pressed):
		return
	if event.keycode == KEY_ESCAPE:
		if _escape_handlers.has(armed_tool):
			_escape_handlers[armed_tool].call()
		else:
			var btn: BaseButton = tool_group.get_pressed_button()
			if btn != null:
				btn.button_pressed = false
			arm_tool("inspect")
		get_viewport().set_input_as_handled()
	elif event.keycode == KEY_DELETE:
		var focused := get_viewport().gui_get_focus_owner()
		if focused is LineEdit or focused is TextEdit or focused is SpinBox:
			return
		for ws in _workspaces:
			if ws.has_method("on_delete_key"):
				if ws.on_delete_key():
					get_viewport().set_input_as_handled()
					return

func _ready() -> void:
	super._ready()

	bridge = EngineBridge.new()
	bridge.name = "EngineBridge"
	add_child(bridge)

	viewport = ViewportHost.new()
	viewport_content.add_child(viewport)
	viewport.setup(bridge)

	## `PARITY_AUDIT.md` §5 item 5, the reference's `#resOverlay` (Shift+D) --
	## a top-right diagnostics HUD, not part of `ViewportHost`'s own "exactly
	## five things" chrome budget (that file's own header comment), so it is
	## a second, independent child of `viewport_content` ("the map surface;
	## overlays are children" -- `dcc_shell.gd`), added after `viewport` so
	## it draws on top. Anchored/offset in its own script; static insets, not
	## `set_safe_insets()`-aware -- ponytail: fine for a debug HUD, revisit if
	## phone chrome ever needs it collision-free.
	resource_overlay = ResourceOverlay.new()
	resource_overlay.set_anchors_and_offsets_preset(Control.PRESET_TOP_RIGHT)
	resource_overlay.offset_left = -260
	resource_overlay.offset_top = 34
	resource_overlay.offset_right = -10
	resource_overlay.offset_bottom = 150
	viewport_content.add_child(resource_overlay)
	resource_overlay.setup(bridge)

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

	city_viewer_window = CityViewerWindow.new()
	add_child(city_viewer_window)
	city_viewer_window.setup(bridge)

	## `placeEditPopup` and `civFactionsModal` (`PARITY_AUDIT.md` §5 items 3,
	## 9, 10). Long-lived like their neighbours -- both keep a selection
	## worth holding between opens, and the place editor in particular is
	## reopened from four different places (the context menu, the Delete-key
	## path's confirm, the roster's settlement sublist, and the CIVIL dock).
	place_editor_window = PlaceEditorWindow.new()
	add_child(place_editor_window)
	place_editor_window.setup(self, bridge)

	faction_roster_window = FactionRosterWindow.new()
	add_child(faction_roster_window)
	faction_roster_window.setup(self, bridge)

	performance_window = PerformanceWindow.new()
	add_child(performance_window)
	performance_window.setup(bridge)

	gen_info_dialog = GenInfoDialog.new()
	add_child(gen_info_dialog)
	gen_info_dialog.setup(self, bridge)

	data_manager_window = DataManagerWindow.new()
	add_child(data_manager_window)
	data_manager_window.setup(self, bridge)

	asset_library_window = AssetLibraryWindow.new()
	add_child(asset_library_window)
	asset_library_window.setup(self, bridge)

	travel_library_window = TravelLibraryWindow.new()
	add_child(travel_library_window)
	travel_library_window.setup(self, bridge)

	open_project_dialog = OpenProjectDialog.new()
	add_child(open_project_dialog)
	open_project_dialog.setup(self)

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
		## Named, not walked to: the phone bottom bar nests differently and
		## wants a different node hidden -- see `DccShell.rail_region()`.
		DccMenus.ID_WIN_RAIL: rail_region(),
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

	## The cold start. The reference opens onto a mandatory setup gate whose
	## intro step is exactly three buttons -- "🌍 Generate a world",
	## "📂 Load project (.zip)", "🗻 Import a heightmap" (reference HTML lines
	## 657-666) -- and nothing simulates until one is chosen. This port shows
	## the same three choices, with two differences, both deliberate:
	##
	## - **It is not a gate.** The reference's own comment calls its gate
	##   mandatory ("No Skip"); here Escape or Cancel closes the prompt and
	##   leaves the empty shell exactly as it was, with the status hint above
	##   still standing. A DCC shell with a populated menu bar, a rail and a
	##   viewport has somewhere to go with no world open; a single-file web
	##   page with a blank canvas does not. The prompt is a convenience, not
	##   a lock.
	## - **It is deferred one frame.** `popup_centered` before the shell has
	##   laid out once centres against a window that has not been sized yet.
	##
	## Suppressed entirely when a world already exists -- nothing does yet at
	## this point in `_ready`, but the guard keeps this honest if a future
	## autoload restores a session.
	if not bridge.has_world:
		_open_welcome_when_drawn()

## `call_deferred` was not enough, and the difference is a hard crash rather
## than a cosmetic one. It runs at the end of the *current* idle frame, which
## is still before the renderer has finished standing up: a `Window` (which an
## `AcceptDialog` is) needs its own render target, and asking GL Compatibility
## for one that early fails outright --
## `_update_render_target_color: Could not create render target, status: 0`,
## preceded by `texture_free_data` on an id the GLES3 allocs cache never got,
## and followed a few frees later by a signal-11 crash inside
## `update_texture_atlas`. Reproduced on a real launch (AMD RX 7800 XT,
## OpenGL 3.3 Core) with the backtrace pointing straight at
## `open_project_dialog.gd`'s `popup_centered()`.
##
## Awaiting `frame_post_draw` puts the popup after a frame has actually been
## drawn, by which point the render target it needs can be created. Two
## `process_frame`s first so layout has settled and `popup_centered` centres
## against a real window size -- the reason the old comment gave for
## deferring at all, which still holds.
func _open_welcome_when_drawn() -> void:
	await get_tree().process_frame
	await get_tree().process_frame
	await RenderingServer.frame_post_draw
	if is_inside_tree() and not bridge.has_world:
		open_welcome()

## Three rail buttons, not five (`dcc_shell.gd`'s own `DOMAINS` doc comment:
## 2026-08-20 domain merge). `InfrastructureWorkspace` and `RenderWorkspace`
## still exist and still build their own real content -- they are just
## composed *into* `CivilizationWorkspace`/`CartographyWorkspace` now
## (`civilization_workspace.gd`'s own `_infra` field,
## `cartography_workspace.gd`'s own `_render` field) rather than getting a
## `register_workspace()` call of their own here.
func _register_workspaces() -> void:
	## Each workspace builds its own left-dock panel and, where it has one, its
	## own right-dock contribution. They are constructed up front and hidden,
	## so an L2 category left open in Cartography is still open when the user
	## comes back to it.
	for entry in [
		["world", WorldWorkspace.new()],
		["civilization", CivilizationWorkspace.new()],
		["cartography", CartographyWorkspace.new()],
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
	viewport.map_right_clicked.connect(_on_map_right_clicked)


# -- Contextual chrome --------------------------------------------------------

## §4: the tool options bar "always reflects the active tool or workspace", and
## §10 says the timeline is "absent from generation and style screens --
## generation is not time-based". Both are functions of the active domain, so
## both are driven from one place rather than five workspaces each remembering.
func _on_workspace_changed(id: String) -> void:
	## Was `id in ["civilization", "infrastructure"]` -- INFRA merged into
	## CIVIL 2026-08-20 (`dcc_shell.gd`'s `DOMAINS` doc comment), so the one
	## surviving id already covers both.
	timeline_bar.visible = id == "civilization"
	if id != "world":
		right_dock_ctrl.leave_sculpt_context()
	match id:
		"world": _tool_options_generate()
		## CARTO absorbed RENDER's one subject (terrain appearance, unbound)
		## the same pass -- see the note appended below.
		"cartography": _tool_options_simple("CARTOGRAPHY · STYLE",
			"presentation only — no control here marks a generation stage stale. Terrain appearance (formerly the RENDER domain) is real in render.rs but unbound to Godot; quality tier lives in Preferences.")
		## Settlement/POI/Territory (civ_tools_bridge.rs) and Way/Route/Measure/
		## Region (infra_tools_bridge.rs) are bound and tested as of 2026-08-19,
		## and §4.5's TOOLS block that arms them now exists in this dock
		## (`civilization_workspace.gd`'s own `_build_tools()`, which composes
		## `infrastructure_workspace.gd`'s Way/Route buttons into the same row
		## since the 2026-08-20 domain merge). The earlier wording here claimed
		## the palette "is not built yet" and was stale the moment that file
		## shipped -- it says so in its own comments. These strings are only the
		## idle default a domain switch lands on; each workspace reclaims the bar
		## with its own richer row the moment one of its tools arms.
		"civilization": _tool_options_simple("CIVIL · INSPECT",
			"Settlement, Territory, Way and Route tools are armed from the TOOLS block in the dock. POI has no engine call (civ_tools_bridge.rs) and is not offered.")
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
		## The reference's third button in this same group (`#centerBtn`).
		## Live since 2026-08-23 (`GUI_GAP_REGISTER.md` MS-01): the centring
		## pass is a real, golden-verified port now
		## (`cartalith_engine::center::center_landmasses`), so this calls it
		## instead of disclosing its absence.
		var centre := DccWidgets.action(row, "Center landmasses", _center_landmasses)
		centre.tooltip_text = "The reference's #centerBtn. Rotates the world in longitude so the emptiest meridian sits at the map edge, then feathers the join it moved into the interior -- the wrapped map has no natural origin, so this is an equivalent world with the seam relocated into open ocean. Whole-world mode only; Region edges are hard borders. The civilisation layer is dropped, since its coordinates would no longer line up."
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

## The reference's `#centerBtn`. Every outcome is reported in the status bar
## rather than silently: "already centred" (offset 0) is a real answer, and
## so is the Region-mode refusal the reference raises an `alert()` for.
func _center_landmasses() -> void:
	if not bridge.has_world:
		set_status("hint", "no world to centre", "accent")
		return
	var r: Dictionary = bridge.center_landmasses()
	if not bool(r.get("ok", false)):
		set_status("hint", String(r.get("reason", "centring unavailable")), "accent")
		return
	var off := int(r.get("offset", 0))
	if off == 0:
		set_status("hint", "already centred — the emptiest meridian is at the edge", "text_ghost")
		return
	set_status("hint", "centred: rotated %d columns, seam feathered at column %d" %
		[off, int(r.get("seam_column", 0))], "text_ghost")

func _new_seed() -> void:
	if new_world_dialog.has_method("randomise_seed"):
		new_world_dialog.randomise_seed()
	else:
		open_new_world()

## §3: the rail foot carries the active context and, in World, the stage counter.
func _refresh_rail_foot() -> void:
	var ctx := {"world": "TERRAIN", "civilization": "CIVIL", "cartography": "STYLE"}
	var text: String = ctx.get(active_domain(), "")
	if active_domain() == "world":
		text += "   %s / 10" % ("10" if bridge.has_world else "00")
	set_rail_foot(text)

# -- Menu callbacks -----------------------------------------------------------

func open_new_world() -> void:
	new_world_dialog.popup_centered()

## File ▸ Open project… (`DCC_SHELL_SPEC.md` §2.1). The generic Godot
## `FileDialog` this used to pop is gone: `design/Cartalith DCC Shell.dc.html`
## gained an "Open project dialog 1920" screen that is a world *gallery* --
## recents, seeds, edit times, a `CURRENT` badge -- not a file tree, and
## `open_project_dialog.gd` draws exactly that. Its own dashed import tile is
## the route to a `.zip` sitting somewhere else on disk.
func open_project_picker() -> void:
	open_project_dialog.open()

## The welcome prompt, shown once on a cold start (see `_ready`). Same
## dialog as `open_project_picker()`, in its welcome mode -- see
## `open_project_dialog.gd`'s header for why this is one screen with three
## actions rather than a second dialog in front of it.
func open_welcome() -> void:
	open_project_dialog.open_welcome()

## Import ▸ Load heightmap… — the reference's third route into a world
## (`#loadBtn` + `#inferTectBtn`, reference HTML lines 534-535). Picks a PNG,
## then hands it to the engine, which resamples it, takes it as the elevation
## field and infers a tectonic substrate underneath so every downstream layer
## has something to read.
##
## The scale settings come from `new_world_dialog.request()` rather than from
## a form of their own, and that mirrors the reference exactly: its own import
## path reopens the setup gate in `calibrate` mode, which is the *same* form
## as the new-world one with resolution and extent omitted (`_suCalSync` is
## literally `_suGenSync`). Grid *height* is the one thing not taken from it —
## the engine derives that from the image's own aspect ratio.
func open_heightmap_import() -> void:
	if not bridge.import_api:
		set_status("hint", "this build's engine has no heightmap import", "accent")
		return
	if bridge.generating:
		return
	DccBrowseDialog.choose_file(self, "Import heightmap — browse", PackedStringArray(["png"]),
		DccSettings.storage_root("projects"),
		"PNG heightmaps, white = high. Scale comes from New world…'s width and peak.",
		func(path: String):
			set_status("hint", "importing %s…" % path.get_file(), "text_ghost")
			bridge.import_heightmap(path, new_world_dialog.request()))

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

## Assets ▸ Import pack… Deliberately *not* the gallery above: the mockup's
## Open-project screen is world-shaped throughout (it captions tiles with a
## seed and an edit time, it is titled "choose a world to continue", its foot
## names the projects root), and an asset pack is none of those things. It
## gets the other new screen instead -- `DccBrowseDialog` in file mode, which
## is the "Select folder dialog 1920" browser with its file rows live rather
## than dimmed -- so no stock `FileDialog` survives on this path either.
func open_asset_pack_picker() -> void:
	DccBrowseDialog.choose_file(self, "Import asset pack", PackedStringArray(["zip"]),
		DccSettings.storage_root("asset_packs"),
		"asset packs read from %s" % DccSettings.storage_root("asset_packs"),
		func(path: String):
			if not bridge.load_asset_pack(path):
				set_status("hint", "asset pack failed — see console", "accent"))

## `tab`, if given, opens `world_data_window` scoped straight to that tab
## ("Settlements" / "Provinces" / "Economy") -- `right_dock.gd`'s RD-03
## Settlement ▸ Economy button is the one caller that needs this; every
## other caller (the Data menu) still gets the default first-tab open.
func open_world_data(tab: String = "") -> void:
	world_data_window.open(tab)

## The City Viewer, on one settlement (`GUI_GAP_REGISTER.md` UM-02).
## `index` is an index into `bridge.settlements()`; the window's own picker
## can move off it once open.
func open_city_viewer(index: int) -> void:
	city_viewer_window.open(index)

## The place editor on one settlement (`PARITY_AUDIT.md` §5 item 3).
## `index` is an index into `bridge.settlements()`.
func open_place_editor(index: int) -> void:
	place_editor_window.open_for(index)

## The Faction Roster modal (`civOpenFactionsBtn`).
func open_faction_roster() -> void:
	faction_roster_window.open()

func open_performance() -> void:
	performance_window.open()

func open_gen_info() -> void:
	gen_info_dialog.open()

func toggle_resource_overlay() -> void:
	resource_overlay.toggle()

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

## Data ▸ Travel library… (⇧L, `TRAVEL_LIBRARY_SPEC.md`). `kind` optionally
## scopes the initial tab ("animal"/"vehicle"/"vessel"/"preset"), empty opens
## on whichever tab was last selected (or Animals & mounts, the first time).
func open_travel_library(kind: String = "") -> void:
	travel_library_window.open(kind)

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

## The storage-locations rows' own Browse… button. This is the call site
## "Select folder dialog 1920" was drawn for -- the mockup even titles itself
## "Select markdown vault folder", one of the roots this shell will grow
## (`MARKDOWN_VAULT_INTEGRATION.md`), and its foot states where the chosen
## folder's contents will be written. `DccBrowseDialog.choose_folder`'s
## callback takes the same single absolute path `FileDialog.dir_selected` did,
## so this function's body is otherwise unchanged.
func _browse_root(key: String, readout: Label) -> void:
	DccBrowseDialog.choose_folder(self,
		"Select %s folder" % String(DccSettings.ROOT_LABELS[key]).to_lower(),
		DccSettings.storage_root(key),
		"%s currently written to %s" % [String(DccSettings.ROOT_LABELS[key]),
			DccSettings.storage_root(key)],
		func(path: String):
			DccSettings.set_storage_root(key, path)
			readout.text = path)

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

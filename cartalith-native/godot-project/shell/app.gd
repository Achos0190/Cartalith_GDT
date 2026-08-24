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
## The cross-section's bottom strip (`design/Cartalith Measurement Toolbar
## .dc.html` state 2). A `viewport_content` overlay rather than a shell
## region -- see `section_strip.gd`'s own header for why. Hidden unless the
## Measure tool's Cross-section mode has a reading.
var section_strip: SectionStrip
## The unified Sculpt/Paint/Measure tool bar (`design/Cartalith Paint
## Toolbar.dc.html`). Not a `Node` -- it owns no scene state of its own, only
## the callback that fills `tool_options_row`.
var tool_bar: DccToolBar
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
## The Markdown Vault panel (`MARKDOWN_VAULT_SCOPE.md` milestone 1). Opened
## scoped to a settlement, province or continent, or on its overview.
var vault_window: VaultWindow
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
## The tool-options bar's copy of the WORLD dock's bake button. Rebuilt every
## time `_tool_options_generate()` runs, so always reached through
## `is_instance_valid`.
var _tool_options_bake: Button

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
## tool id -> Callable(), called on Backspace. Added for the measurement
## toolbar's own `⌫ drop last` (`design/Cartalith Measurement Toolbar.dc.html`
## state 1's modifier row) -- the same shape as the four dictionaries above,
## rather than a keyboard listener inside `global_tools.gd`, because Escape
## and Delete already arrive through `_unhandled_key_input` here and a second
## key path would be a second place to look when one of them stops working.
var _backspace_handlers: Dictionary = {}

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

func register_tool_backspace_handler(id: String, handler: Callable) -> void:
	_backspace_handlers[id] = handler

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
		_escape_action()
		get_viewport().set_input_as_handled()
	elif event.keycode == KEY_BACKSPACE:
		## Same text-field guard Delete needs below, and for the same reason:
		## Backspace inside a `LineEdit` means "delete a character", never
		## "drop the last measured point".
		var typing := get_viewport().gui_get_focus_owner()
		if typing is LineEdit or typing is TextEdit or typing is SpinBox:
			return
		if _backspace_handlers.has(armed_tool):
			_backspace_handlers[armed_tool].call()
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

## Escape's body, named because Android's back gesture means the same thing at
## the point it runs out of surfaces to leave (`_back_exhausted()`), and a phone
## has no Escape key to press.
##
## `force_disarm` is the one place the two diverge, and it is load-bearing.
## `GlobalTools._measure_escape()` deliberately clears the chain and leaves
## Measure **armed** -- correct for a pointer user, whose next action is
## overwhelmingly another measurement. Back inheriting that would make the
## gesture a no-op forever: every press would clear an already-clear chain,
## `armed_tool` would never reach `inspect`, and the exit below would be
## unreachable with Measure armed. A back press must always leave a level, so
## it runs the handler's cleanup *and then* disarms.
func _escape_action(force_disarm := false) -> void:
	if _escape_handlers.has(armed_tool):
		_escape_handlers[armed_tool].call()
		if not force_disarm:
			return
	var btn: BaseButton = tool_group.get_pressed_button()
	if btn != null:
		btn.button_pressed = false
	arm_tool("inspect")

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

	## The Markdown Vault panel (`MARKDOWN_VAULT_SCOPE.md` milestone 1).
	## Long-lived like its neighbours: it holds an entity scope, an open
	## reader and a checkbox set worth keeping between opens, and it is
	## reached from three different places (the place editor's KNOWLEDGE
	## section, and the Civilization dock's province and continent rows).
	##
	## `store_changed` -> `VaultStore.save_from` is the only writer of
	## `user://markdown_vault.json`. The window never touches the disk; it
	## says what changed and this owns when that is persisted.
	vault_window = VaultWindow.new()
	add_child(vault_window)
	vault_window.setup(self, bridge)
	vault_window.store_changed.connect(func(): VaultStore.save_from(bridge))
	VaultStore.load_into(bridge)

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

	## The cross-section strip. A second overlay child of `viewport_content`,
	## added after `viewport` so it draws on top of the map, and after
	## `resource_overlay` so the two never contend for the same corner (the
	## HUD is top-right; this is the full bottom edge). Anchored, not laid
	## out -- see `section_strip.gd`.
	section_strip = SectionStrip.new()
	viewport_content.add_child(section_strip)
	section_strip.setup(self)

	_wire_status()
	_wire_selection()
	GlobalTools.install(self)
	## Built after `GlobalTools.install` because its Measure row reads
	## `GlobalTools.measure_mode()`, and after `_register_workspaces` because
	## its Paint row reads `WorldWorkspace`'s own brush dictionary.
	tool_bar = DccToolBar.install(self, bridge)

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

	_setup_autosave()

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

## Corrected 2026-08-24 (`GUI_GAP_REGISTER.md` SG-01). This used to read "there
## is no staleness state to report", on the grounds -- verified live against
## the reference (Playwright, 2026-08-19) -- that every generation control
## regenerates the whole world on release (`tparam()`'s `change` handler), so
## the dials can never leave anything behind. That half is still true and
## `world_workspace.gd` still works that way.
##
## What it missed is that the *tools* leave real staleness behind, deliberately:
## a sculpt or a carve settles hydrology and climate but leaves the civ layer
## for `recompute_civilisation` to catch up (SG-02's measured reason), and a
## hand-dropped or deleted settlement leaves everything derived from the
## roster. That state is the engine's own -- `stale_stages()` reads
## `cartalith_spatial::StageGraph` -- not an invented dirty flag, which is the
## objection the old comment was really making. The `stale` status slot the
## shell has reserved since it was built is where it goes.
##
## The generation duration that used to occupy that slot moves into `pass`
## ("generated · 3.2s"), which is where the rest of the last run's outcome
## already is.
func _wire_status() -> void:
	bridge.generation_started.connect(func():
		set_status("pass", "generating…", "accent")
		set_status("hint", "", "text_ghost")
		if is_instance_valid(_tool_options_stale):
			_tool_options_stale.text = "generating…")
	bridge.generation_finished.connect(func(ok: bool):
		set_status("pass", ("generated · %.1fs" % (bridge.last_generate_ms / 1000.0)) if ok
			else "generate failed", "text_dim" if ok else "accent")
		set_status("hint", bridge.last_summary, "text_ghost")
		var g := bridge.grid_size()
		set_status("top_world", ("ELDRA · %d" % bridge.world_gen.get_seed()) if ok else "—")
		set_status("top_res", ("%d×%d working" % [g.x, g.y]) if ok else "")
		set_status("top_mem", "%.1f GB" % (OS.get_static_memory_usage() / 1073741824.0))
		if is_instance_valid(_tool_options_stale):
			_tool_options_stale.text = ""
		_refresh_rail_foot()
		_refresh_world_dependent())
	bridge.world_loaded.connect(func():
		set_status("pass", "loaded", "text_dim")
		set_status("hint", bridge.last_summary, "text_ghost")
		_refresh_world_dependent())
	_setup_staleness()

## SG-01's clock. A `Timer` and not a `_process` tick, and not a signal either:
## staleness is produced by half a dozen unrelated `#[func]`s (every commit
## path, every place edit, every marked `set_params`), and wiring a
## notification into each of them would be six couplings for a readout that is
## a plain query. One second is well under the time it takes to notice, and
## `stale_stages()` recomputes nothing -- `StageGraph`'s accessors all take
## `&self`.
var _stale_timer: Timer

func _setup_staleness() -> void:
	_stale_timer = Timer.new()
	_stale_timer.name = "StalenessPoll"
	_stale_timer.wait_time = 1.0
	_stale_timer.one_shot = false
	_stale_timer.timeout.connect(refresh_staleness)
	add_child(_stale_timer)
	_stale_timer.start()

## Reads the engine's stage graph and writes the `stale` status slot. Public
## because a control that has just cleared staleness (the Civilization dock's
## Recompute button) should show that immediately rather than up to a second
## later.
##
## Names the stale stages, then the most-upstream reason once -- "climate ·
## civ — sculpt" rather than repeating the cause per stage, since the graph
## reports the same origin all the way down a chain by design.
func refresh_staleness() -> void:
	var stale: Dictionary = bridge.stale_stages()
	if stale.is_empty():
		set_status("stale", "", "text_faint")
		return
	var names := PackedStringArray()
	var reason := ""
	for stage in stale:
		names.append(String(stage))
		if reason.is_empty():
			var e: Dictionary = stale[stage]
			reason = String(e.get("reason", ""))
			if reason.is_empty():
				reason = String(e.get("origin", ""))
	set_status("stale", "stale: %s — %s" % [" · ".join(names), reason], "stale")

## `GUI_GAP_REGISTER.md` **SH-07**: the status bar's `atlas` slot, which
## `dcc_shell.gd` has built since the shell shipped and nothing ever wrote.
##
## The reference's `updateAtlasStatus` (line 10748) is a chunk *count*; this
## adds the two numbers a user actually decides on — how deep the bake goes and
## what it costs on disk — plus the finalize state, since a locked world's
## single most important fact is that it is locked.
##
## Blank for a world with nothing baked: an empty slot is the honest reading of
## "no atlas", and a permanent "Atlas: empty" would spend a status slot saying
## nothing. Called after every bake, clear, finalize and generate.
## What has to be re-read when the world itself changes identity — a generate
## or a save load. Both the atlas slot and the Finalize foot describe *this*
## world's atlas, and a new world has a different `atlas_world_key()`, so both
## are stale the instant generation finishes.
##
## This existed as a gap rather than a decision: `refresh_atlas_status()`'s own
## doc already claimed it was "called after every bake, clear, finalize and
## generate", and the generate call site was never written. The visible symptom
## was a dead end rather than a stale readout — `_refresh_finalize()` runs when
## the workspace is *built*, which is before any world exists, so it left
## "Bake ALL levels & finalize" disabled, and nothing else re-enabled it. The
## only paths that called it back were the bake and clear buttons, one of which
## was the disabled one. Found by `_bakeui_shot.gd` pressing the real button.
func _refresh_world_dependent() -> void:
	refresh_atlas_status()
	for ws in _workspaces:
		if ws.has_method("on_world_changed"):
			ws.on_world_changed()

## The WORLD workspace, found by capability rather than by index — `_workspaces`
## also carries the right dock, which is not one.
func _world_workspace() -> WorldWorkspace:
	for ws in _workspaces:
		if ws is WorldWorkspace:
			return ws
	return null

## Called by `world_workspace._refresh_finalize()`, the single owner of whether
## baking is currently possible. See `_tool_options_generate()` for why the
## header copy is pushed to rather than computed twice.
func set_bake_shortcut(shown: bool, is_disabled: bool, tip: String) -> void:
	if not is_instance_valid(_tool_options_bake):
		return
	_tool_options_bake.visible = shown
	_tool_options_bake.disabled = is_disabled
	_tool_options_bake.tooltip_text = tip

func refresh_atlas_status() -> void:
	var st: Dictionary = bridge.atlas_status()
	if st.is_empty() or int(st.get("chunks", 0)) == 0:
		set_status("atlas", "", "text_faint")
		return
	var text := "atlas: %d chunks · LOD 0–%d · %s" % [
		int(st.get("chunks", 0)), int(st.get("deepest_level", 0)),
		String(st.get("bytes_text", ""))]
	if bool(st.get("finalized", false)):
		set_status("atlas", text + " · FINALIZED", "accent")
	else:
		set_status("atlas", text, "text_dim")

func _wire_selection() -> void:
	viewport.settlement_selected.connect(func(data, index):
		for ws in _workspaces:
			if ws.has_method("on_settlement_selected"):
				ws.on_settlement_selected(data, index))
	viewport.cursor_sampled.connect(func(gx, gy, valid):
		for ws in _workspaces:
			if ws.has_method("on_cursor_sampled"):
				ws.on_cursor_sampled(gx, gy, valid))
	## The Wildlife debug view's own click behaviour (the reference's
	## `showWildInfo`/`hideWildInfo` pair, HTML 9785-9791): while that view
	## is the one actually drawn, a map click picks the nearest ecoregion
	## marker and hands its roster to the right dock; a click that misses
	## every marker clears it back to Sample. Gated on `debug_view()` rather
	## than on a tool being armed, exactly as the reference gates on
	## `state.debug === 'wildlife'` -- so this adds no behaviour at all to
	## any other view, and no new tool to the rail.
	viewport.map_clicked.connect(func(gx, gy):
		if viewport.debug_view() != "wildlife":
			return
		if right_dock_ctrl.has_method("show_wildlife"):
			right_dock_ctrl.show_wildlife(bridge.wildlife_region_at(gx, gy)))
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
	_fill_timeline_strip()
	if id != "world":
		right_dock_ctrl.leave_sculpt_context()
	match id:
		"world": _tool_options_generate()
		## CARTO absorbed RENDER's one subject (terrain appearance) the same
		## pass; that subject is bound as of the map-coloration pass, so this
		## caption no longer claims it is not.
		"cartography": _tool_options_simple("CARTOGRAPHY · STYLE",
			"presentation only — no control here marks a generation stage stale. Map view, Map style and Rendering-advanced drive render.rs's TerrainAppearance live; the quality tier those values start from lives in Preferences.")
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

## §10's reserved timeline strip, which shows only in CIVIL — **and which was
## 70 px of blank panel across the whole window in that domain** (measured
## live 2026-08-24: `visible=true, children=0, height=70`). The controls it
## reserves space for deliberately live in the CIVIL dock's own Timeline
## category instead, on `TIMELINE_SCOPE.md` §4's "default to your own
## dedicated panel rather than risking the wrong shell region" — but *leaving
## the region on screen and empty* was never part of that decision, and
## `Window ▸ Timeline` toggles it, so a user can also turn an empty band on
## and off.
##
## Given a pointer rather than the controls: the strip now says where the
## timeline is and takes you there, which is this shell's established answer
## for a surface whose capability lives elsewhere (the tool-options bar's
## shortcut onto the WORLD dock's bake button, immediately below). Nothing is
## duplicated — `open_timeline_category()` presses the dock's own header.
func _fill_timeline_strip() -> void:
	if timeline_row == null:
		return
	for c in timeline_row.get_children():
		timeline_row.remove_child(c)
		c.queue_free()
	if not timeline_bar.visible:
		return
	_tool_options_label(timeline_row, "TIMELINE", "text_dim")
	## `clip_text` is load-bearing here for the same reason `tool_bar.gd`'s own
	## `_note()` documents: a `Label` reports its full text width as its
	## minimum size, and this row sits in the shell's top-level
	## `VBoxContainer`, so one long sentence raises the whole window's minimum
	## width and pushes the right dock off the screen.
	var hint := DccTheme.label(
		"Years, playback and per-year filters are in CIVIL ▸ Politics; the collapse/recovery simulation is CIVIL ▸ Simulation.",
		"text_ghost", DccTheme.FS_MICRO)
	hint.clip_text = true
	hint.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	timeline_row.add_child(hint)
	var go := DccWidgets.action(timeline_row, "Open Timeline", func():
		var ws = _workspace_panels.get("civilization")
		if ws != null and ws.has_method("open_timeline_category"):
			ws.open_timeline_category())
	go.tooltip_text = ("Opens CIVIL ▸ Politics in the left dock -- the recorded years, the "
		+ "scrubber, playback and the existence filters, which v3 names 'political change "
		+ "over time'. This strip is §10's reserved region; v3 calls the scrubber program "
		+ "scope and it belongs here, but the controls need a year-pill list, an add-year "
		+ "field and three filter checkboxes, which one fixed-height row cannot hold "
		+ "(TIMELINE_SCOPE.md §4, GUI_GAP_REGISTER.md CV-24).")
	## No trailing spacer: `hint` already expands, so the action lands hard
	## right the way every other bar in this shell puts its action. Adding one
	## made both grow and parked the button in the middle of the strip.

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
		## §4's tool-options bar carries a second copy of the WORLD dock's bake
		## control — `GUI_GAP_REGISTER.md` WW-01's own last open item, and until
		## now a dead placeholder whose tooltip ("No bake/LOD pipeline exists
		## yet") stopped being true the day WW-01 shipped.
		##
		## Wired as a *shortcut*, not a second source of truth: it presses the
		## same `_on_bake_all` the dock foot does, and `_refresh_finalize()` —
		## which already runs on every generate, bake, finalize and clear —
		## pushes its enabled/visible state here through `set_bake_shortcut`.
		## Two buttons with two independent state computations is exactly the
		## drift this shell has been bitten by before.
		var bake := DccWidgets.action(row, "Bake ALL & finalize", func():
			var ws: WorldWorkspace = _world_workspace()
			if ws != null:
				ws.bake_and_finalize())
		_tool_options_bake = bake
		var ws0: WorldWorkspace = _world_workspace()
		if ws0 != null:
			ws0.on_world_changed()
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
	## PH-06: on a phone the dialog fills the screen instead, and it has to be
	## *opened* by that call rather than sized by it -- `DccWidgets
	## .phone_present()`'s own doc comment carries why (an `AcceptDialog` lays
	## its content out from a resize notification, so a hidden resize followed
	## by `popup_centered()` leaves the body at its desktop rect and the form
	## overflows instead of scrolling).
	if not DccWidgets.phone_present(new_world_dialog, self):
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

# -- §2.1 Project lifecycle ----------------------------------------------------
#
# Save / Save as… / Autosave / Revert / Close, all of which were disabled
# menu items with "requires a save writer" on them until `cartalith-io` grew
# one (`GUI_GAP_REGISTER.md` FI-01, `SAVEFILE_COMPAT.md`).
#
# One rule runs through all five, and it is the same one `_load_project`
# already followed: `current_project_path` is the single piece of project
# bookkeeping this shell keeps, and every route through here maintains it or
# does nothing.

## The autosave clock. A `Timer` rather than a `_process` accumulator because
## the interval is minutes and the work is a file write -- nothing here wants
## per-frame resolution.
var _autosave_timer: Timer

func _setup_autosave() -> void:
	_autosave_timer = Timer.new()
	_autosave_timer.name = "Autosave"
	_autosave_timer.one_shot = false
	_autosave_timer.timeout.connect(_autosave_tick)
	add_child(_autosave_timer)
	## The status slot is one of the four the shell already reserves
	## (`dcc_shell.gd`'s `_build_status_bar`) and has been empty since it was
	## built -- this is what it was for.
	bridge.dirty_changed.connect(func(_d: bool): _refresh_save_status())
	bridge.project_saved.connect(func(_p: String): _refresh_save_status())
	apply_autosave_setting()

## Reads `DccSettings.autosave_enabled()` and starts or stops the clock.
## Called at startup and whenever the menu toggle flips.
func apply_autosave_setting() -> void:
	if _autosave_timer == null:
		return
	if DccSettings.autosave_enabled():
		_autosave_timer.wait_time = DccSettings.autosave_minutes() * 60.0
		_autosave_timer.start()
	else:
		_autosave_timer.stop()
	_refresh_save_status()

## Autosave writes **beside** the project, never over it: a background writer
## that silently replaces the file the user last chose to keep is how an
## autosave feature destroys work instead of protecting it. `world.zip`
## autosaves to `world.autosave.zip`, and recovering is File ▸ Open project…
## on that file.
##
## Skipped when there is nothing to write (no world), nowhere to write it
## (the project has never been saved, so there is no folder the user has
## chosen), or nothing new to write (`bridge.world_dirty` -- see its own doc
## comment for what that does and does not see).
func _autosave_tick() -> void:
	if not bridge.save_api or not bridge.has_world or not bridge.world_dirty:
		return
	if current_project_path == "":
		return
	var target := current_project_path.get_basename() + ".autosave.zip"
	## Deliberately does **not** clear the dirty flag: the project itself is
	## still unsaved, and an autosave that made File ▸ Save look unnecessary
	## would be worse than no autosave.
	var was_dirty := bridge.world_dirty
	if bridge.world_gen.save_project(target):
		set_status("autosave", "autosaved %s" % Time.get_time_string_from_system().substr(0, 5), "text_faint")
	else:
		set_status("autosave", "autosave failed", "accent")
	bridge.world_dirty = was_dirty

func _refresh_save_status() -> void:
	if not DccSettings.autosave_enabled():
		set_status("autosave", "" if not bridge.world_dirty else "unsaved changes",
			"text_faint" if not bridge.world_dirty else "accent")
	elif current_project_path == "":
		set_status("autosave", "autosave waiting for a saved project", "text_ghost")
	else:
		set_status("autosave", "autosave every %d min" % DccSettings.autosave_minutes(), "text_faint")

## File ▸ Save project. Falls through to Save as… when the world has never
## been written anywhere -- the behaviour every application has, and the
## reason there is no separate "Save" disabled state to explain.
func save_project() -> void:
	if not bridge.has_world:
		set_status("hint", "no world to save", "accent")
		return
	if current_project_path == "":
		save_project_as()
		return
	_write_project(current_project_path)

## File ▸ Save as… Uses the shell's own browser in its save mode rather than
## a stock `FileDialog`, for the same reason `open_project_picker()` uses the
## gallery: this shell draws its own chrome.
func save_project_as(then: Callable = Callable()) -> void:
	if not bridge.has_world:
		set_status("hint", "no world to save", "accent")
		return
	var suggested := current_project_path.get_file()
	if suggested == "":
		## The reference names its own exports `world_<seed>_<size>.zip`
		## (reference HTML's `exportZip`); the seed half is the part that
		## identifies the world, and the bake size means nothing here.
		suggested = "world_%d.zip" % bridge.world_gen.get_seed()
	var start := current_project_path.get_base_dir()
	if start == "":
		start = DccSettings.storage_root("projects")
	DccBrowseDialog.choose_save_path(self, "Save project as", "zip", start,
		"", suggested, func(path: String):
			if FileAccess.file_exists(path):
				_confirm(
					"Overwrite %s?" % path.get_file(),
					"That file already exists. Saving replaces it.",
					"Overwrite", func(): _write_project(path, then))
			else:
				_write_project(path, then))

## The one place a project is actually written. Everything above routes here
## so the bookkeeping -- `current_project_path`, the recents list, the status
## line, the optional continuation -- happens once.
func _write_project(path: String, then: Callable = Callable()) -> void:
	if not bridge.save_project(path):
		set_status("hint", "save failed — see console", "accent")
		return
	current_project_path = path
	DccSettings.remember_project(path)
	set_status("hint", "saved %s" % path.get_file(), "text_ghost")
	_refresh_save_status()
	if then.is_valid():
		then.call()

## File ▸ Revert to last save. Throws the in-memory world away and reloads
## the file, which is exactly `_load_project` on the current path -- the
## confirm in front of it is the whole feature, since the discard is
## irreversible and the button sits two rows under Save.
func revert_to_saved() -> void:
	if current_project_path == "":
		set_status("hint", "this world has never been saved", "accent")
		return
	_confirm(
		"Revert to last save?",
		"Everything since the last save of %s is discarded." % current_project_path.get_file(),
		"Revert", func(): _load_project(current_project_path))

## File ▸ Close project.
func close_project() -> void:
	if not bridge.has_world:
		return
	confirm_unsaved_world("Close project", "Close it?", "Discard and close",
		"Save and close", _close_world)

## The unsaved-work gate. **The only prompt of its kind in the shell** -- File ▸
## Close project, Android's back gesture at the end of its navigation
## (`DccShell._back_exhausted()`) and the desktop window's system close
## (`DccShell._close_requested()`), both overridden below, all come here, rather
## than each growing a second, subtly different prompt of its own.
##
## Prompts **whenever a world exists**, not only when `bridge.world_dirty` is
## set -- see that flag's own doc comment for what it cannot see, and why the
## last moment before work is destroyed is the wrong one to under-report.
##
## Three answers, because the third button is the whole reason this prompt could
## not be built before there was a writer: with no Save to offer it would have
## been "discard or cancel", which is not a choice.
##
## Returns the dialog, because `_close_requested()` has to be able to *check*
## that it really went up -- see there.
func confirm_unsaved_world(prompt_title: String, question: String,
		discard_text: String, save_text: String, then: Callable) -> ConfirmationDialog:
	var body := "This world has unsaved changes." if bridge.world_dirty \
		else "Tool edits made since the last save are not tracked, so save if in doubt."
	var dlg := ConfirmationDialog.new()
	dlg.title = prompt_title
	## Phone treatment, and only on a phone: `DccWidgets.phone_window()` clears
	## `wrap_controls` unconditionally, which is right for a window with a
	## scrolling body and wrong for a text-sized prompt -- with it cleared and
	## no size set, `popup_centered()` collapses the dialog onto its minimum and
	## clips the question. A prompt nobody can read is not a fix, and a prompt
	## whose buttons land at ~5 dp (the recorded "desktop pixels on a phone"
	## bug class) is not one either -- which is what `phone_present()`'s
	## content scale is here to prevent.
	var phone := is_phone()
	if phone:
		DccWidgets.phone_window(dlg, self)
	## `phone_window()` drops the title bar, so on a phone the title has to live
	## in the body instead of vanishing with the decoration.
	dlg.dialog_text = ("%s\n\n" % prompt_title if phone else "") \
		+ "%s\n\n%s" % [body, question]
	dlg.ok_button_text = discard_text
	var save_btn := dlg.add_button(save_text, true, "save")
	save_btn.pressed.connect(func():
		dlg.hide()
		if current_project_path == "":
			save_project_as(then)
		else:
			_write_project(current_project_path, then))
	dlg.confirmed.connect(then)
	dlg.visibility_changed.connect(func(): if not dlg.visible: dlg.queue_free())
	add_child(dlg)
	if not DccWidgets.phone_present(dlg, self):
		dlg.popup_centered()
	if phone:
		## AFTER the popup, and re-applied on every rotation -- see
		## `_floor_prompt_buttons()` for why neither is optional. The relay
		## is guarded and self-releasing for the same reason
		## `DccWidgets.phone_window()`'s is: this dialog frees itself on close,
		## and a rotation afterwards would otherwise touch a freed object.
		_floor_prompt_buttons(dlg, save_btn)
		var refloor := func():
			if is_instance_valid(dlg) and dlg.visible:
				_floor_prompt_buttons(dlg, save_btn)
		phone_insets_changed.connect(refloor)
		dlg.tree_exiting.connect(func():
			if phone_insets_changed.is_connected(refloor):
				phone_insets_changed.disconnect(refloor))
	return dlg

## Floor `ConfirmationDialog`'s three stock answers at §13's 44 dp tap minimum.
##
## `DccShell.phone_fit()` cannot do it: it walks `get_children()`, and
## `AcceptDialog` parents its whole button bar as an **internal** child, so the
## stock row is outside every fit this shell performs and measured 29 dp.
## Everywhere else that has mattered little, because a window's real controls
## live in its content child. Here the three buttons *are* the dialog, and one
## of them destroys a world.
##
## Two measured traps, both of which silently produced 29 dp buttons on the way
## to this working:
##
##   1. `b.custom_minimum_size.y = 44` through an **untyped** loop element
##      writes to a temporary copy of the vector and is lost. Hence the typed
##      `for b: Button` and the whole-`Vector2` assignment.
##   2. **`Window.popup()` clears it.** Isolated in a two-node scene: the value
##      survives `content_scale_*`, `min_size` and `max_size`, and is `(0, 0)`
##      the instant the window is shown, because `AcceptDialog` re-lays its
##      internal button bar on popup. So this must run AFTER the popup -- and
##      again after every re-popup, which is what a rotation is.
func _floor_prompt_buttons(dlg: ConfirmationDialog, extra: Button) -> void:
	for b: Button in [dlg.get_ok_button(), dlg.get_cancel_button(), extra]:
		b.custom_minimum_size = Vector2(0.0, DccTheme.PHONE_TAP_MIN)

## Android's back gesture has run out of things to leave (`DccShell`'s chain).
##
## One level remains before the app itself: an armed tool is a *mode*, and
## leaving a mode is exactly what back means -- so this does what Escape does,
## including letting a multi-click tool commit through its own handler.
##
## Then the exit gate, and deliberately **not** a "press back again to exit"
## timer. That pattern earns its place in an app whose back stack is one level
## deep, where a stray edge swipe is the only thing it can be; here back already
## walks four real levels (dialog, menu level, overlay, tool) before it can ever
## reach this function, so a press that arrives here is a considered one. It
## also has nowhere to draw its hint on the phone composition, where the status
## bar is parked hidden as the phone menu's model. The prompt is the guard where
## there is something to lose; where there is not, back exits at once, which is
## the platform convention and the only way out of a full-screen app.
func _back_exhausted() -> void:
	if armed_tool != "inspect":
		_escape_action(true)
		return
	if not bridge.has_world:
		get_tree().quit()
		return
	confirm_unsaved_world("Exit Cartalith", "Exit the app?", "Discard and exit",
		"Save and exit", func(): get_tree().quit())

## The exit prompt raised by the system close, while it is outstanding. `null`
## once it resolves; `_quit_asked` records that we *tried*, and is set before the
## attempt so that it survives an attempt that fails halfway.
var _quit_prompt: ConfirmationDialog = null
var _quit_asked := false

## The desktop title bar's ×, Alt+F4, the taskbar's Close (BK-02). Same fault as
## BK-01 and the same fix: `DccShell._ready()` turns `auto_accept_quit` off so
## the request reaches code, and the request goes through the one shared
## `confirm_unsaved_world()` gate rather than a second prompt of its own.
##
## **Why this cannot leave the app un-closeable**, which is the reason the fix
## was deferred when BK-02 was registered. `auto_accept_quit = false` means the
## only way out is our own `quit()`, so the invariant this function keeps is:
##
##   *every close request either quits, or leaves a VISIBLE prompt on screen
##   whose three answers all resolve.*
##
## Each branch, in order:
##
##   1. The prompt is up and visible -> re-raise it, do not stack a second and
##      do not quit. Quitting on a double-click of × would destroy the world the
##      first click just asked about, which is the bug, not a fallback.
##   2. We already asked and there is no visible prompt -> **quit**. This is the
##      escape hatch, and it covers the failure the deferral was about: if the
##      prompt ever fails to appear -- a script error mid-way through building
##      it, a Window that never shows -- `_quit_asked` is already true, so the
##      next × ends the process unconditionally.
##   3. Nothing to lose -> quit at once.
##   4. Otherwise prompt, then **verify it is actually visible**, and quit
##      immediately if it is not. The first × is enough even for a prompt that
##      silently fails on its very first use.
##
## Resolution closes the loop from the other side: Cancel hides the dialog, which
## frees it (`visibility_changed` in `confirm_unsaved_world()`), which clears
## both flags here; Discard quits; Save writes and then quits through the same
## continuation. Nothing leaves the flags set with the app still running and
## nothing on screen.
func _close_requested() -> void:
	if is_instance_valid(_quit_prompt) and _quit_prompt.visible:
		_quit_prompt.grab_focus()
		return
	if _quit_asked or not bridge.has_world:
		get_tree().quit()
		return
	_quit_asked = true
	_quit_prompt = confirm_unsaved_world("Exit Cartalith", "Exit the app?",
		"Discard and exit", "Save and exit", func(): get_tree().quit())
	if not (is_instance_valid(_quit_prompt) and _quit_prompt.visible):
		get_tree().quit()
		return
	_quit_prompt.tree_exiting.connect(func():
		_quit_prompt = null
		_quit_asked = false)

func _close_world() -> void:
	bridge.close_world()
	current_project_path = ""
	set_status("pass", "no world", "text_faint")
	set_status("hint", "File ▸ New world… to begin", "text_ghost")
	set_status("top_world", "—")
	_refresh_save_status()

## A yes/no prompt with the shell's own wording rules: the destructive answer
## is named after what it does, never "OK".
func _confirm(prompt_title: String, body: String, ok_text: String, on_ok: Callable) -> void:
	var dlg := ConfirmationDialog.new()
	dlg.title = prompt_title
	dlg.dialog_text = body
	dlg.ok_button_text = ok_text
	dlg.confirmed.connect(on_ok)
	dlg.visibility_changed.connect(func(): if not dlg.visible: dlg.queue_free())
	add_child(dlg)
	dlg.popup_centered()

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

## The Markdown Vault panel, scoped to one entity
## (`MARKDOWN_VAULT_INTEGRATION.md` §28: the vault belongs in the entity's own
## information panel, not in an isolated utility). `kind` is `"settlement"`,
## `"province"` or `"continent"`; `entity_id` is that kind's own id — a
## settlement's **tid**, not its index into `bridge.settlements()`.
func open_vault(kind: String, entity_id: int, label: String) -> void:
	vault_window.open_for(kind, entity_id, label)

## The same panel with no entity scope: the whole link store.
func open_vault_overview() -> void:
	vault_window.open_overview()

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
			## §2.1: "Moving the atlas root invalidates the cache." As of
			## 2026-08-24 there is a real cache here -- the persistent tile
			## atlas WORLD ▸ Generate ▸ Finalize ▸ Bake writes (`bake_bridge.rs`), which
			## `EngineBridge.atlas_ready()` points at exactly this root. Moving
			## it does not delete anything: the chunks stay where they are and
			## the new root simply starts empty, which is the honest behaviour
			## and the one a browser cache pane has.
			var cache_note := DccTheme.label(
				"The tile atlas baked by WORLD ▸ Generate ▸ Finalize lives here. Moving this root leaves the existing chunks in place and starts the new location empty; clear the old one from Preferences ▸ Memory ▸ Clear caches before you move it if you want the space back.",
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
## `Edit ▸ Undo` / Ctrl+Z — the reference's `undoBtn` (`undoLast`), register
## ED-01. Pops one committed height field back off the engine's bounded stack.
##
## Repaints by writing `map_view.texture` directly rather than calling
## `ViewportHost.refresh()`, which would also reset the camera to fit: undoing
## an edit should leave you looking at exactly where you were looking. That is
## the same reasoning (and the same two lines) `world_workspace.gd`'s
## `_on_sculpt_commit` already uses for the commit this reverses.
##
## The engine deliberately does not re-run flow, rivers or climate here -- see
## `undo.rs` -- so the status hint says which stages are now behind the height
## field rather than leaving that to be discovered.
func undo_last() -> void:
	var label := bridge.undo_last()
	if label == "":
		set_status("hint", "Nothing to undo.", "text_ghost")
		return
	if viewport != null:
		viewport.map_view.texture = bridge.color_texture()
		viewport.set_preview_texture(null)
	var stats: Dictionary = bridge.undo_stats()
	set_status("pass", "undid %s" % label.to_lower(), "text_dim")
	set_status("hint", "%d undo step%s left · flow, rivers and climate are not re-run" % [
		int(stats.get("depth", 0)), "" if int(stats.get("depth", 0)) == 1 else "s"], "text_ghost")

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
##
## **The domain switch is load-bearing** (`GUI_GAP_REGISTER.md` IN-10, owner
## report 2026-08-24: "there is no way to plan a Journey"). Journey's takeover
## only paints when CIVIL is the active domain -- `journey_planner_view.gd`'s
## `_recompute_visibility()` requires `app.active_domain() == "civilization"`,
## correctly, since the view swaps *that domain's* region and would otherwise
## paint over WORLD's or CARTO's dock. But the shell opens on WORLD, and the
## two entry points that can be reached from anywhere (`Data ▸ Journey
## planner… ⇧J` and the right dock's own "Plan a journey") only armed the
## tool. From WORLD or CARTO the result was a menu item that changed nothing
## on screen at all: the tool armed, the status line said so in ghost text at
## the far corner, and every other pixel stayed put. Verified live from a
## fresh launch before the fix. Selecting the domain first is what the INFRA
## dock button had implicitly all along (you can only click it from inside
## CIVIL), so this makes every entry point behave like the one that worked.
func open_journey_planner() -> void:
	select_domain("civilization")
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
	## `Window ▸ Timeline` can turn the strip on from any domain, including the
	## two `_on_workspace_changed` hides it in — and its contents are built by
	## that same handler, so without this it comes back blank.
	if id == DccMenus.ID_WIN_TIMELINE:
		_fill_timeline_strip()

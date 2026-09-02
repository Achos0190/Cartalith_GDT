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
var shortcuts_dialog: ShortcutsDialog
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
## scoped to one entity, or on its overview. The kinds it can be scoped to are
## `EngineBridge.vault_entity_kinds()`, never a list written here -- that list
## grew from three to five on 2026-08-25 and the transcriptions did not.
var vault_window: VaultWindow
## Long-lived, unlike `DccBrowseDialog` (which spawns and frees per pick):
## the gallery holds a scope chip and a search query worth keeping between
## opens, exactly like every other window on this list.
var open_project_dialog: OpenProjectDialog
## The phone's own entry screen (`phone_project_picker.gd`'s own header for
## the full reasoning) -- `null` on desktop and tablet, never constructed
## there at all, so `is_phone()` is what every reader below has to check
## rather than a visibility flag on a node that might not exist.
var phone_project_picker: PhoneProjectPicker

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
	## `BUILD_ANSWERS.md` §4's "tool arm", 10 ms. Inside the early-return guard
	## on purpose: re-arming the tool already armed is a no-op everywhere else
	## in this function and buzzing for it would be the one part that fired.
	## `_haptic()` (`DccShell`) is itself a no-op off Android/iOS, so nothing
	## about the desktop path changes.
	_haptic("tool_arm")
	tool_armed.emit(id)
	## **The timeline strip's hand-back (JP-13).** `journey_planner_view.gd`
	## borrows `timeline_row` while the Journey tool is armed and empties it
	## again on disarm -- in `_recompute_visibility()`, which the emit above
	## has just run synchronously. Its comment there says it is clearing the
	## row "back to that empty state", and that premise stopped being true
	## when `_fill_timeline_strip()` gave CIVIL an owner for the row: the
	## strip simply stayed blank for the rest of the session.
	##
	## Refilling here rather than in that file keeps the borrow one-sided --
	## the view takes the row and gives it back, and this is the giving back.
	## Gated on the row actually being empty and the tool no longer being
	## Journey, so re-arming Journey (which fills the row itself, a few frames
	## into `_compute()`) is never fought over.
	if id != "journey" and timeline_row != null and timeline_bar != null and timeline_bar.visible \
			and timeline_row.get_child_count() == 0:
		_fill_timeline_strip()
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
		if delete_selection():
			get_viewport().set_input_as_handled()
			return

## Delete whatever is selected, in whichever workspace owns a selection.
##
## Extracted from the `KEY_DELETE` branch above so `Edit ▸ Delete` can reach
## the same path. It was reachable only from the keyboard until 2026-08-30,
## while the menu row beside it said deletion was impossible -- see
## `menus.gd`'s Edit block for what that row used to claim.
##
## Returns whether anything was actually deleted, so a caller can tell "there
## was nothing selected" from "it happened".
func delete_selection() -> bool:
	for ws in _workspaces:
		if ws.has_method("on_delete_key"):
			if ws.on_delete_key():
				return true
	return false

## Seed a fresh world's lighting from `DccSettings.lighting_defaults()`.
##
## Every key is a real `render.rs` tunable (`sun_az_deg`, `sun_alt_deg`,
## `relief_ambient`, `relief_lights`), so this is one `set_appearance()` call
## and no new rendering -- which is what the menu row's own reason said was the
## only thing missing.
##
## `set_appearance` returns how many keys it accepted; an older cdylib that
## does not publish one of the four simply takes fewer, which is the same
## degrade `render_workspace.gd` already relies on. Silent by design: this runs
## on every Generate and a status line saying "lighting applied" every time
## would be noise, not information.
func _apply_lighting_defaults() -> void:
	if not bridge.has_method("set_appearance"):
		return
	bridge.set_appearance(DccSettings.lighting_defaults())

## Clear every selection the shell holds -- `Edit > Deselect` (⌘D,
## `DCC_SHELL_SPEC.md` §2.2, "Select all / Deselect ... Scoped to the active
## layer").
##
## The row was disabled until 2026-08-30 with the reason *"no shared way to
## clear them: Escape disarms the active tool without touching what is
## selected"*. Both halves of that were true and only the second is now: this
## is the shared way, and Escape still deliberately does not call it -- Escape
## means "put the tool down", which is a different act from "keep the tool,
## forget what it was pointed at".
##
## Three owners, because there are exactly three selections in this shell and
## none of them knew about the others:
##
##   - **settlements** -- `_selected_index` on `CivilizationWorkspace`, reached
##     through `on_deselect()` the same way Delete reaches `on_delete_key()`
##   - **labels** -- `label_select(-1)`, which the engine has always accepted
##     as "select nothing"
##   - **icons** -- `icon_deselect()`, which it did NOT have until this change;
##     that asymmetry is what made the row's reason true in the first place
##
## "Scoped to the active layer" is honoured by the workspace guard rather than
## by a layer test: `on_deselect()` refuses unless its own domain is active,
## and labels and icons are both Cartography's, so a Deselect in World clears
## nothing. Returns whether anything was actually cleared.
func clear_selection() -> bool:
	var any := false
	for ws in _workspaces:
		if ws.has_method("on_deselect"):
			if ws.on_deselect():
				any = true
	if bridge.label_get_selected() >= 0:
		bridge.label_select(-1)
		any = true
	if bridge.icon_get_selected() >= 0:
		bridge.icon_deselect()
		any = true
	if any:
		## The two on-canvas handle sets belong to whatever was selected, so
		## they go with it -- a resize handle floating over nothing is the
		## same class of lie as a menu row that claims a capability.
		viewport.tool_overlay.set_handles([])
		viewport.refresh_annotations()
		set_status("hint", "selection cleared", "text_dim")
	return any

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

	## `GUI_GAP_REGISTER.md` **RF-04**, and it is a signal-ORDER bug rather than
	## a missing connection -- the first of those this register has had.
	##
	## `_refresh_world_dependent()` below already calls `TradeStore.clear()`, and
	## `infrastructure_workspace.gd`'s own comment says the Flows body "refills
	## from whatever `TradeStore` holds, which `app.gd` has just cleared on this
	## same world change". It had not. Godot delivers a signal in connection
	## order, `_register_workspaces()` runs at line 313 and `_wire_status()` at
	## 333, so on every generate the INFRA refill read the **previous** world's
	## match, redrew its flow count, its timing and its settlement names, and
	## only then did the store get dropped -- with nothing left to re-run the
	## fill. Measured: after regenerating under a live 624-flow match, CIVIL ▸
	## Trade ▸ Flows still showed 624 while `TradeStore.last()` was empty, so the
	## dock and its two fellow readers (the place editor's ledger, the way-load
	## overlay) disagreed about whether a match existed at all.
	##
	## Connected here, before anything else subscribes to either signal, so the
	## store is empty by the time any reader is asked to redraw. The call in
	## `_refresh_world_dependent()` stays: it costs nothing and keeps that
	## function's stated ownership of "the world changed identity" true.
	bridge.generation_finished.connect(func(ok: bool): if ok: TradeStore.clear())
	bridge.world_loaded.connect(func(): TradeStore.clear())

	viewport = ViewportHost.new()
	viewport_content.add_child(viewport)
	viewport.setup(bridge)
	## §2.5's Tiled LOD mode is a preference, so it survives a restart -- and a
	## preference the shell forgets on boot is the same lie as a menu row with
	## nothing behind it. The default is `true`, so this is a no-op for anyone
	## who never touched it.
	viewport.set_lod_auto(DccSettings.lod_auto())
	## §2.5 Graphics ▸ Lighting rig defaults, applied to each FRESH world.
	##
	## Deliberately `generation_finished` and **not** `world_loaded`: a project
	## archive carries its own `appearance.json` slot (`project_bridge.rs:92`),
	## so seeding a preference over a world someone saved would be this
	## preference destroying their work. A Generate has no such record to
	## respect -- it is the moment a new world's look is decided, which is
	## exactly what a default is for.
	bridge.generation_finished.connect(func(ok: bool):
		if ok:
			_apply_lighting_defaults())

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
	shortcuts_dialog = ShortcutsDialog.new()
	add_child(shortcuts_dialog)
	shortcuts_dialog.setup(self)

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

	## Phone only, and not merely hidden the rest of the time -- see
	## `phone_project_picker.gd`'s own header for why the node not existing at
	## all is load-bearing rather than a style choice. `is_phone()` is already
	## final by this point: `super._ready()` two lines into this function ran
	## `DccShell._ready()`, which calls `_compute_layout_mode()` before
	## returning.
	if is_phone():
		phone_project_picker = PhoneProjectPicker.new()
		add_child(phone_project_picker)
		phone_project_picker.setup(self)

	## The Markdown Vault panel (`MARKDOWN_VAULT_SCOPE.md` milestone 1).
	## Long-lived like its neighbours: it holds an entity scope, an open
	## reader and a checkbox set worth keeping between opens, and it is
	## reached from three different places (the place editor's KNOWLEDGE
	## section, and the Civilization dock's province and continent rows).
	##
	## `store_changed` -> `VaultStore.save_from` is the only writer of
	## `user://markdown_vault.json`. The window never touches the disk; it
	## says what changed and this owns when that is persisted.
	##
	## **Two things happen here, and the second is milestone 3's**
	## (`MARKDOWN_VAULT_SCOPE.md`). Since the project archive carries the links
	## (`project_save_with_documents` writes `vault.json`), a vault mutation is
	## an unsaved change to the *project*, exactly like a moved label or a
	## painted stroke. `mark_world_dirty()` is what makes File ▸ Save offer
	## itself and the autosave pick them up; without it the archive silently
	## fell behind the panel. It is a no-op with no world, so a link attached
	## before anything is generated marks nothing.
	##
	## `current_project_path != ""` is the flag `save_from` needs to know which
	## of the two writers owns the links this session — see `vault_store.gd`'s
	## own header for the rule and for what happens to a sidecar written
	## before it.
	vault_window = VaultWindow.new()
	add_child(vault_window)
	vault_window.setup(self, bridge)
	vault_window.store_changed.connect(func():
		VaultStore.save_from(bridge, current_project_path != "")
		bridge.mark_world_dirty())
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

	## The three regions that ARE a node whose `visible` is the whole truth, in
	## every composition. The other two are phone-conditional and live in
	## `toggle_region()` instead -- see its own header for why writing
	## `node.visible` was wrong for both of them on a handset.
	_region_nodes = {
		DccMenus.ID_WIN_TIMELINE: timeline_row.get_parent().get_parent(),
		## Named, not walked to: the phone bottom bar nests differently and
		## wants a different node hidden -- see `DccShell.rail_region()`.
		DccMenus.ID_WIN_RAIL: rail_region(),
	}
	if not is_phone():
		_region_nodes[DccMenus.ID_WIN_LEFT] = left_dock
		_region_nodes[DccMenus.ID_WIN_RIGHT] = right_dock
		_region_nodes[DccMenus.ID_WIN_STATUS] = status_row.get_parent().get_parent()

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
	##
	## **On phone, `open_welcome()` (called below via `_open_welcome_when_drawn`)
	## opens `phone_project_picker.gd` instead of this dialog** -- the locked
	## Android spec's own entry decision, not a variant of this one. See that
	## file's header for the full reasoning and `open_welcome()`'s own comment
	## for the one-line branch.
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
	## `statusMid`'s two moving parts. `generation_stage(index, name, total)` is
	## the engine's own per-stage tick -- `world_workspace.gd`'s ten state
	## labels are driven by the same signal -- so the stage name printed here is
	## the engine's, never a copy of a stage table kept in the shell.
	##
	## The signal fires when a stage *starts*, so mid-run the last **resolved**
	## stage is the one before it; at `generation_finished` the running one has
	## resolved too and becomes the last.
	##
	## **Residual gap, disclosed rather than silently left**: this remembers
	## only the immediately-preceding tick, and `EngineBridge._process()`
	## (`engine_bridge.gd:216-231`) emits on *change*, polled once a frame --
	## an index that jumps by more than one between two frames resolves
	## without ever being individually named here. `cartalith-engine/src/
	## progress.rs`'s own doc comment says stages 8 and 9 (Ecology & biomes,
	## Resources & soils) have no code and "tick through together right
	## before this function returns", which is exactly the shape that can
	## race a frame poll. This can only ever UNDERstate -- `_mid_stage` only
	## ever promotes from a tick that has genuinely started a later stage, or
	## from a successful `generation_finished` -- never claims a stage
	## resolved before it did. Fixing the understatement precisely would need
	## a stage-name-by-index table this file does not have and should not
	## grow: `world_workspace.gd`'s `_assert_stage_names()` exists specifically
	## because a duplicated copy of `progress.rs::STAGE_NAMES` drifted from it
	## once already (this function's own opening paragraph is the fix for
	## that, for this reader -- read the name off the signal, never off a
	## second table).
	bridge.generation_stage.connect(func(index: int, stage_name: String, _total: int):
		_mid_stage = _mid_running_stage
		## `BUILD_ANSWERS.md` §2.2's fixed string reads `09 Ecology`, not
		## `09 Ecology & biomes` -- the mockup names truncate at " &"
		## (`name.split(' &')[0]`, `Cartalith DCC Environment.dc.html:1984`).
		## `cartalith-engine/src/progress.rs::STAGE_NAMES` carries the full
		## names (three of the ten have a " & "), so this shell truncates the
		## same way rather than showing an ampersand the reference never does.
		_mid_running_stage = "%02d %s" % [index + 1, stage_name.split(" &")[0]]
		_refresh_status_mid())
	bridge.generation_started.connect(func():
		set_status("pass", "generating…", "accent")
		set_status("hint", "", "text_ghost")
		_mid_stage = ""
		_mid_running_stage = ""
		_refresh_status_mid()
		if is_instance_valid(_tool_options_stale):
			_tool_options_stale.text = "generating…")
	bridge.generation_finished.connect(func(ok: bool):
		if ok:
			_mid_stage = _mid_running_stage
		_mid_running_stage = ""
		_refresh_status_mid()
		set_status("pass", ("generated · %.1fs" % (bridge.last_generate_ms / 1000.0)) if ok
			else "generate failed", "text_dim" if ok else "accent")
		set_status("hint", bridge.last_summary, "text_ghost")
		## The first of `UNWIRED_FUNCTIONS.md`'s three phone-invisible failures:
		## a generate that fails wrote `pass` and `hint`, both of which live in
		## the hidden `PhoneMenuModel` host on a handset, so the only symptom
		## was a map that never changed. `_report_failure()` rewrites the same
		## hint and adds the toast; it runs after the line above so its sentence
		## is the one that stands.
		if not ok:
			_report_failure("Generation failed. Nothing was changed on the map."
				if bridge.last_summary == ""
				else "Generation failed — %s" % bridge.last_summary)
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
		## **No document restore here.** `world_loaded` fires for a centring
		## pass, a fjord carve, an applied asset pack and a close as well as
		## for an open, while `bridge.last_documents` still holds the last
		## OPENED archive's text on every one of them -- so restoring from
		## this handler replayed the file's journeys over every journey the
		## user had planned since, on every one of those actions. The restore
		## lives in `_restore_project_documents()`, called from
		## `_load_project()`, which is the only place a new set of documents
		## actually arrives.
		## A load never goes through `generation_stage`, so there is no stage to
		## name and the composite says so rather than reprinting the stage of
		## whatever was generated before it.
		_mid_stage = ""
		_mid_running_stage = ""
		_refresh_status_mid()
		_refresh_world_dependent())
	_setup_staleness()

# -- §11 `statusMid` -----------------------------------------------------------
#
# `BUILD_ANSWERS.md` §2.2, fixed verbatim:
#   `last pass 09 Ecology · 101 ms · repaint 84 ms · autosave 14:02`
# "The stage name is the last stage whose `stageState()` is `resolved`, so the
# middle of the bar always says what the map currently rests on. Autosave shows
# `off` when autosave is off."
#
# **`repaint NN ms` is deliberately absent, and this is the comment that owes
# the reader why.** It is blocked on the owner's open question 2 in
# `UNWIRED_FUNCTIONS.md`: the prototype composites in one canvas pass and can
# time it; this shell composites through `ViewportHost` plus
# `map_overlay.gd` plus whatever overlays are live, and has no equivalent
# single-pass timer to read. `grep -rn repaint shell/*.gd` finds prose and
# nothing else. The field goes here, between the pass duration and the autosave
# clock, once question 2 says what it should measure.
#
# Everything else here comes from writers that already existed -- the pass
# duration is `bridge.last_generate_ms`, the same figure the `pass` slot prints
# as `generated · 1.4s`, and the autosave state is the same
# `DccSettings.autosave_enabled()` / last-autosave pair the `autosave` slot's
# four writers read. That slot is now registered and not drawn
# (`DccShell._build_status_bar()`), so the clock appears once.

var _mid_stage := ""           ## The last stage that has RESOLVED, `NN Name`.
var _mid_running_stage := ""   ## The one currently running; not yet resolved.
var _mid_autosave_at := ""     ## `HH:MM` of the last successful autosave.

func _refresh_status_mid() -> void:
	if not bridge.has_world and not bridge.generating:
		set_status("mid", "", "text_ghost")
		return
	var parts := PackedStringArray()
	if _mid_stage != "":
		parts.append("last pass %s" % _mid_stage)
		## Only alongside a stage, and only once the run that produced it has
		## actually settled. `last_generate_ms` is written by
		## `EngineBridge._finish()` right as a run ends (`engine_bridge.gd:515`/
		## `:584`, both before that run's own `generation_finished.emit`), so
		## it is correct the instant a stage resolves via `generation_finished`
		## -- but mid-*regenerate*, while stages are still resolving one at a
		## time, it is still the PREVIOUS run's total. Showing it there
		## attributes an old run's duration to the stage this run just
		## resolved, which is a wrong number, not an approximate one. There is
		## no per-stage figure on the bridge either way -- `world_workspace.gd`'s
		## own per-stage timings are its private `_stage_elapsed_ms`, not shared
		## state -- so this waits for the run to finish rather than guess.
		if not bridge.generating and bridge.last_generate_ms > 0.0:
			parts.append("%d ms" % int(round(bridge.last_generate_ms)))
	elif bridge.has_world and not bridge.generating:
		## A loaded world: resolved, with no run behind it to name. The
		## `not bridge.generating` half of this guard is load-bearing, not
		## defensive: `generation_started` resets `_mid_stage` to "" before the
		## first stage tick of a *regenerate* lands, and `has_world` is still
		## true from the run before it (`engine_bridge.gd` never clears it at
		## the start of `generate()`) -- so without this guard the composite
		## spent the opening stretch of every regenerate reading "loaded — no
		## generation this session" in the same breath the `pass` slot beside
		## it read "generating…".
		parts.append("loaded — no generation this session")
	parts.append("autosave %s" % _status_mid_autosave())
	set_status("mid", " · ".join(parts), "text_ghost")

## §2.2's autosave field. `off` is fixed by the answer; the other two states
## are this shell's, and each is a fact rather than a promise -- a clock only
## once an autosave has actually written one, and the interval before that.
func _status_mid_autosave() -> String:
	if not DccSettings.autosave_enabled():
		return "off"
	if _mid_autosave_at != "":
		return _mid_autosave_at
	return "every %d min" % DccSettings.autosave_minutes()

## SG-01's clock. A `Timer` and not a `_process` tick, and not a signal either:
## staleness is produced by half a dozen unrelated `#[func]`s (every commit
## path, every place edit, every marked `set_params`), and wiring a
## notification into each of them would be six couplings for a readout that is
## a plain query. One second is well under the time it takes to notice, and
## `stale_stages()` recomputes nothing -- `StageGraph`'s accessors all take
## `&self`.
var _stale_timer: Timer

## `PARITY_AUDIT.md` §23 F16's first item: the shell has shown staleness since
## SG-01 and had no way to settle it on purpose. `recompute_stale_stages` --
## the `#[func]` that recomputes whatever the graph currently reports stale and
## answers `{recomputed, still_stale, ms}` -- was named by a probe and by
## nothing else. Every commit path (sculpt, carve, paint) already calls it
## internally, so the readout does clear eventually; what was missing is the
## explicit "recompute now" the reference offers.
##
## It lives in `status_row`, beside the readout it acts on, and it is driven by
## the same one-second poll rather than by a signal -- `_setup_staleness()`'s
## own reasoning above, unchanged: the button's visibility is a second reader
## of `stale_stages()`, not a seventh coupling into the six `#[func]`s that
## produce staleness. Hidden when nothing is stale, which is also its whole
## disabled state.
##
## **Desktop and tablet only, disclosed rather than hidden.** §13 parks the
## status bar in an invisible model host on the phone and `phone_menu.gd`
## re-presents the *readouts* as list rows (`status_slot_text()`), so a control
## added to `status_row` is not drawn there. The phone therefore still shows
## staleness and still settles it the way it always has -- as a side effect of
## the commit paths -- and reaching this button from `phone_menu.gd`'s root
## screen is that file's change to make, not this one's.
var _stale_recompute: Button

func _setup_staleness() -> void:
	_stale_timer = Timer.new()
	_stale_timer.name = "StalenessPoll"
	_stale_timer.wait_time = 1.0
	_stale_timer.one_shot = false
	_stale_timer.timeout.connect(refresh_staleness)
	add_child(_stale_timer)
	_stale_timer.start()

	_stale_recompute = DccWidgets.action(status_row, "Recompute", _recompute_stale)
	_stale_recompute.visible = false
	_stale_recompute.tooltip_text = ("Re-runs the stages the graph reports stale right now, "
		+ "and nothing else. The civilisation layer is deliberately not cascaded per edit "
		+ "(UNIFIED_TOOL_PLAN.md milestone C measured why), so \"civ\" usually stays -- "
		+ "Civilization ▸ Settlements ▸ Recompute civilisation is the one that clears it.")
	## `dcc_shell.gd`'s `_build_status_bar()` fills this row in its own fixed
	## order -- pass, stale, autosave, atlas, spacer, hint -- so index 2 is
	## immediately after the `stale` label. Appended and moved rather than built
	## into that function, because the row is shell geometry and this is
	## composition, which is the split `app.gd`'s own header states.
	status_row.move_child(_stale_recompute, 2)

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
		if is_instance_valid(_stale_recompute):
			_stale_recompute.visible = false
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
	## Only offered on a binary that can act on it: an older extension answers
	## `stale_stages()` and not this, and a button that silently does nothing is
	## worse than no button (`GUI_GAP_REGISTER.md`'s own standing rule).
	if is_instance_valid(_stale_recompute):
		_stale_recompute.visible = bridge.world_gen != null \
			and bridge.world_gen.has_method("recompute_stale_stages")

## The "Recompute" button beside the `stale` slot. Synchronous, with no
## progress signal to subscribe to, so it does what `civilization_workspace.
## gd`'s `_recompute_civ()` does for the same reason: relabel, disable, let two
## frames actually paint that, then block.
func _recompute_stale() -> void:
	if bridge.generating or bridge.world_gen == null:
		return
	var b := _stale_recompute
	if is_instance_valid(b):
		b.text = "Recomputing…"
		b.disabled = true
		await get_tree().process_frame
		await get_tree().process_frame
	var r: Dictionary = bridge.world_gen.recompute_stale_stages()
	if is_instance_valid(b):
		b.disabled = false
		b.text = "Recompute"
	var ran := PackedStringArray(r.get("recomputed", PackedStringArray()))
	var left := PackedStringArray(r.get("still_stale", PackedStringArray()))
	if ran.is_empty():
		## `still_stale` without `recomputed` is the normal answer after a
		## terrain edit, not a failure: `civ` is re-derived by a full recompute
		## of its own and this pass deliberately leaves it alone.
		set_status("hint", ("Nothing recomputed. Still stale: %s." % " · ".join(left))
			if not left.is_empty() else "Nothing was stale.", "text_ghost")
	else:
		set_status("hint", "Recomputed %s in %.0f ms.%s" % [" · ".join(ran), float(r.get("ms", 0.0)),
			(" Still stale: %s." % " · ".join(left)) if not left.is_empty() else ""], "text_ghost")
		## The height/hydrology/climate fields moved, so the textures drawn from
		## them are a frame behind. Same direct call `render_workspace.gd` makes
		## after a live appearance change -- not `world_loaded`, which means "a
		## different world" and would clear the trade match with it.
		if viewport != null:
			viewport.refresh()
	refresh_staleness()

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
	## `GUI_GAP_REGISTER.md` **IN-13**. The trade match is keyed to settlement
	## indices and way indices, both of which a new world renumbers, so a stale
	## match is not merely out of date — it names the wrong places. Cleared
	## here, in the one function that already owns "the world changed
	## identity", rather than by each of its three readers.
	TradeStore.clear()
	if viewport != null and viewport.overlay != null:
		viewport.overlay.set_trade_load(PackedFloat32Array())
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
		## **The second half of the caption is the MODE, not a fixed word.**
		## It read "CARTOGRAPHY · STYLE" unconditionally until stage 2, which
		## was true while CARTO had one destination and became a plain
		## contradiction the moment it had four: the bar said STYLE over an open
		## Labels panel with the rail's `Labels` node lit and the rail foot
		## reading LABELS. Three surfaces agreeing and one disagreeing is worse
		## than four saying nothing. `_tool_options_*` is otherwise stage 5's
		## rewrite (blocked on the prototype's truncated `tbLabel`); this is the
		## one word of it stage 2 is obliged to fix, because stage 2 is what
		## made it wrong.
		"cartography": _tool_options_simple("CARTOGRAPHY · " + active_mode("cartography").to_upper(),
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

# -- §10 The timeline strip -------------------------------------------------
#
# **What this replaces, and why the old reason is gone rather than argued
# with.** Until 2026-08-31 this strip held a `TIMELINE` label, a clipped
# sentence and an `Open Timeline` button that pressed the CIVIL Politics
# accordion. The reason given was `TIMELINE_SCOPE.md` §4's "default to your own
# dedicated panel rather than risking the wrong shell region", plus the
# observation that "one fixed-height row" could not hold a year-pill list, an
# add-year field and three filter checkboxes. That was true, and it predates the
# specification this now builds: `01-frame-and-tokens.md` §3.7 and
# `05-right-dock-and-bars.md` §4.2 author the strip in full, and the strip they
# author is **not** the dock's panel moved down here. It is a transport, a
# speed pill group, a state readout, six layer toggles, a scrub track and a
# footer -- and the container is explicitly *auto height*, not one fixed row.
# So the old reason is not overruled; the thing it declined is still in the
# dock, and this is a different control.
#
# **One cursor.** The year lives in `DccShell`'s §10a block, which reads and
# writes the engine's `CivData::year` through `civ_goto_year()`. The CIVIL
# dock's year pills are the first view of it, this strip is the second and the
# phone sim strip is the third. Nothing here caches a year.
#
# **Collapsed and expanded.** §3.7 gives both forms; `tlOpen` defaults to
# `false`, so the strip opens collapsed -- `TIMELINE · {year} · ▴ expand` --
# and the whole row is the expand target. `Window ▸ Timeline` still toggles the
# region itself, which is a different question and stays where it was.
#
# **`timeline_row` is shared.** `journey_planner_view.gd` takes this same row
# over while the Journey tool is armed (JP-13) and empties it again on disarm.
# `arm_tool()` refills it after the `tool_armed` emit that ran that disarm, so
# the borrow is one-sided and the strip comes back rather than leaving the
# region blank for the rest of the session. `_repaint_timeline()`'s rebuild
# guard is the second line of defence for the same case, and it tests the row's
# child count because the handles this file holds survive the planner emptying
# the row underneath them.

## `tlOpen` (§4.3), false by default.
var _tl_expanded := false

## **Held so the cursor moving does not rebuild the row.** A full rebuild on
## every `timeline_changed` is what this file did first, and it broke the one
## control that emits the signal continuously: the scrub track queue-freed
## itself out from under a drag on the first motion event, so a drag moved the
## year exactly once and then died. `_repaint_timeline()` writes these instead,
## and only falls back to a rebuild when they are gone -- which is the normal
## state whenever the journey planner has taken `timeline_row` over.
var _tl_year_labels: Array[Label] = []
var _tl_state_label: Label
var _tl_play_button: Button
var _tl_speed_segments: Dictionary = {}   ## multiplier -> Button
var _tl_track: Control
var _tl_head: ColorRect
var _tl_phone_button: Button
## Everything `DccShell.tl_available()` gates -- the three transport squares,
## the three speed pills and the scrub track. **Not** the six layer toggles:
## those are a persisted shell preference with no engine behind them either way,
## so whether a world exists makes them neither more nor less real.
var _tl_transport: Array[Control] = []

func _fill_timeline_strip() -> void:
	if timeline_row == null:
		return
	for c in timeline_row.get_children():
		timeline_row.remove_child(c)
		c.queue_free()
	if not timeline_bar.visible:
		return
	## Connected once, on the first fill: `timeline_changed` is emitted by every
	## control in every view of the cursor, and this is the desktop view's
	## repaint. `CONNECT_REFERENCE_COUNTED` is not used -- a plain guard is
	## cheaper and says what it means.
	if not timeline_changed.is_connected(_repaint_timeline):
		timeline_changed.connect(_repaint_timeline)
	_tl_year_labels = []
	_tl_state_label = null
	_tl_play_button = null
	_tl_speed_segments = {}
	_tl_track = null
	_tl_head = null
	_tl_phone_button = null
	_tl_transport = []
	if is_phone():
		_fill_phone_timeline_row()
		return
	if _tl_expanded:
		_build_timeline_expanded()
	else:
		_build_timeline_collapsed()

## §4.1: `TIMELINE` (`--m2` mono, `.16em`, `var(--faint)`) · `{{ tlYearLabel }}`
## (`--m1`, `var(--sec)`) · spacer · `▴ expand` (`--m2`, `var(--faint)`), and
## "the whole strip is clickable → `hTlExpand`".
func _build_timeline_collapsed() -> void:
	var hit := Button.new()
	hit.flat = false
	hit.focus_mode = Control.FOCUS_NONE
	hit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	hit.tooltip_text = "Expand the timeline: transport, speed, the year scrub and the six simulation layers."
	hit.add_theme_stylebox_override("normal", DccTheme.empty())
	hit.add_theme_stylebox_override("focus", DccTheme.empty())
	hit.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	hit.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("line_soft")))
	hit.pressed.connect(func():
		_tl_expanded = true
		_fill_timeline_strip())
	timeline_row.add_child(hit)

	var row := HBoxContainer.new()
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_theme_constant_override("separation", 12)
	row.add_child(DccTheme.mono_label("TIMELINE", "text_faint", DccTheme.role_px("fs_timeline"), 2))
	var year_lbl := DccTheme.mono_label(
		_tl_year_label() if tl_available() else "no world", "text_secondary",
		DccTheme.role_px("fs_timeline"), 0)
	_tl_year_labels.append(year_lbl)
	row.add_child(year_lbl)
	row.add_child(DccTheme.spacer())
	## `▴` U+25B4 -- not in `DccIcons.SYMBOLS`, and the table is not this
	## file's to extend. Same `SystemFont` fallback as every other missing mark.
	row.add_child(DccTheme.mono_label("\u25b4 expand", "text_faint",
		DccTheme.role_px("fs_timeline"), 0))
	hit.add_child(row)
	row.set_anchors_preset(Control.PRESET_FULL_RECT)

## §4.2's three rows, plus a fourth this build owes the reader -- see
## `_build_timeline_layers()`.
func _build_timeline_expanded() -> void:
	var col := VBoxContainer.new()
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col.add_theme_constant_override("separation", 6)
	timeline_row.add_child(col)

	# Row 1 -- transport.
	var t := HBoxContainer.new()
	t.add_theme_constant_override("separation", 10)
	col.add_child(t)

	_tl_play_button = _tl_square(t, DccIcons.SYMBOLS["play"], "accent", tl_toggle_play)
	## `◀` U+25C0. `SYMBOLS` carries `play` (`▶`) and no left-pointing twin;
	## the table is not this file's to extend, so the literal stands beside it.
	var back := _tl_square(t, "\u25c0", "text_secondary", func(): tl_step(-1))
	back.tooltip_text = "Step the year cursor back by the selected speed."
	var fwd := _tl_square(t, DccIcons.SYMBOLS["play"], "text_secondary", func(): tl_step(1))
	fwd.tooltip_text = "Step the year cursor forward by the selected speed."

	## The speed pill group. `DccWidgets.segment()`/`set_segment_on()` is this
	## shell's own lit-one-of-a-set control, so the group reads like every other
	## segmented row rather than like a second vocabulary invented here.
	for mult in TL_SPEEDS:
		var seg := DccWidgets.segment(t, "×%d" % mult, tl_set_speed.bind(mult))
		seg.tooltip_text = ("How far the year cursor moves per step, and per 600 ms of "
			+ "playback: %d year%s." % [mult, "" if mult == 1 else "s"])
		_tl_speed_segments[mult] = seg
		_tl_transport.append(seg)

	_tl_state_label = DccTheme.mono_label("", "accent", DccTheme.role_px("fs_timeline"), 0)
	t.add_child(_tl_state_label)
	t.add_child(DccTheme.spacer())
	_build_timeline_layers(t)
	## `⌄` collapse, `color:var(--faint)`, `padding:0 4px`.
	var collapse := DccWidgets.text_button(t, DccIcons.SYMBOLS["chevron"], func():
		_tl_expanded = false
		_fill_timeline_strip())
	collapse.tooltip_text = "Collapse the timeline back to one row."

	# Row 2 -- the scrub track.
	col.add_child(_build_timeline_scrub())

	# Row 3 -- the track scale, `justify-content:space-between`.
	var foot := HBoxContainer.new()
	col.add_child(foot)
	foot.add_child(DccTheme.mono_label("YEAR %d" % TL_YEAR_MIN, "text_ghost",
		DccTheme.role_px("fs_timeline"), 0))
	foot.add_child(DccTheme.spacer())
	var foot_year := DccTheme.mono_label(_tl_year_label(), "text_secondary",
		DccTheme.role_px("fs_timeline"), 0)
	_tl_year_labels.append(foot_year)
	foot.add_child(foot_year)
	foot.add_child(DccTheme.spacer())
	foot.add_child(DccTheme.mono_label("YEAR %d" % TL_YEAR_MAX, "text_ghost",
		DccTheme.role_px("fs_timeline"), 0))

	# Row 4 -- the note the six toggles owe the reader. See below.
	var note := DccTheme.mono_label("Simulation layers — %s." % TL_LAYER_NOTE,
		"text_ghost", DccTheme.FS_MICRO, 0)
	note.clip_text = true
	note.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col.add_child(note)

	## The transport, the lit speed and the state string are all "current
	## state", so they are written by the same function that will write them
	## again on the next `timeline_changed` rather than a second time here.
	_repaint_timeline()

## §4.2's transport square: `var(--ctl)`, radius 8, `background:var(--ins)`.
func _tl_square(parent: Control, glyph: String, token: String, on_press: Callable) -> Button:
	var b := _menu_square(glyph, on_press)
	b.add_theme_color_override("font_color", DccTheme.c(token))
	## §57's tier A: transport is one of the three controls the tablet artboard
	## floors at 44. `role_px` answers 0 on the desktop, which means "the design
	## states no constraint" -- see `ROLE`'s own note -- so this only ever grows.
	b.custom_minimum_size.y = maxf(b.custom_minimum_size.y,
		float(DccTheme.role_px("btn_min_h")))
	parent.add_child(b)
	_tl_transport.append(b)
	return b

## The six simulation layer toggles — `Climate · Population · Economy ·
## Politics · Infrastructure · Warfare` (`01-frame-and-tokens.md` §3.7, the
## "State defaults" line -- §3.7's own `(line 1204)` points into the prototype
## markup it documents, not into that .md, which is only 680 lines long). `grep -n "Warfare" shell/` found nothing at all before
## this pass.
##
## **They record a choice and render nothing, and they say so.**
## `BUILD_ANSWERS.md` §3 classes them *intended* rather than an oversight and
## fixes the disclosure verbatim -- `DccShell.TL_LAYER_NOTE`, quoted rather than
## reworded -- and rules that it must sit **on the timeline**, which is why the
## toggles and this strip had to be one pass: until the strip existed the note
## had nowhere to go. It is drawn as row 4 above and repeated on every pill.
##
## Enabled rather than `_todo`-disabled, which is the one place this departs
## from the shell's usual "nothing behind it, draw it dead" rule, and it departs
## on the owner's own ruling: there *is* something behind them -- the choice
## persists (`DccShell._tl_save_layers()`) -- and what is missing is the
## renderer, which the note names.
## **What this row costs in width.** Six pills whose labels are their ids adds
## about 410 px of un-clippable minimum to the transport row (a `Button` reports
## its text width as its minimum unless `clip_text` is set, and these are
## labelled controls -- clipping them would leave "Infrastr..." on a toggle
## whose whole job is to name a layer). With the transport, the speeds and the
## state readout the row measures around 850 px, which is above this shell's
## other minimums (rail 40 + docks 372 + 304 = 716) and therefore becomes the
## window's floor. That is the row both the w1366 and the w1920 artboards draw,
## so it is the design's figure rather than an accident -- recorded because the
## previous occupant of this strip was `clip_text`ed for exactly this reason and
## the next reader will otherwise wonder why this one is not.
func _build_timeline_layers(parent: Control) -> void:
	for row in TL_LAYERS:
		var id := String(row[0])
		var on: bool = bool(tl_layers.get(id, row[1]))
		var pill := DccWidgets.segment(parent, id, func(): tl_toggle_layer(id))
		DccWidgets.set_segment_on(pill, on)
		## The note **verbatim**, not `capitalize()`d: `BUILD_ANSWERS.md` §3
		## fixes the wording, and GDScript's `capitalize()` is title case, which
		## would have rewritten a quotation into "They Record Which Layer You
		## Want; No Layer Renders Yet".
		pill.tooltip_text = "%s: %s — %s." % [id, "on" if on else "off", TL_LAYER_NOTE]

## §4.2 row 2: `height:16px`, rail `flex:1; height:3px; background:var(--ins)`,
## playhead `width:2px; height:13px; top:-5px; background:var(--acc)`, and
## `hTlScrub` on pointer-down, and on motion with the button still held --
## §4.2 binds `onPointerDown` only, but a 3 px track that answers a click and
## refuses a drag is not a scrubber. See `_place_timeline_head()` for where the
## playhead's fraction comes from.
func _build_timeline_scrub() -> Control:
	var track := Control.new()
	track.custom_minimum_size.y = DccTheme.role_px("timeline_track_h")
	track.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	track.tooltip_text = ("Drag to move the CIVIL year cursor (civ_goto_year) anywhere in "
		+ "-400..1200. The map's territory changes only at the years CIVIL > Politics has "
		+ "recorded a snapshot for; between them the cursor moves and the territory holds.")

	var rail := ColorRect.new()
	rail.color = DccTheme.c("sunken")
	rail.set_anchors_preset(Control.PRESET_CENTER_LEFT)
	rail.anchor_right = 1.0
	rail.offset_left = 0
	rail.offset_right = 0
	rail.offset_top = -1
	rail.offset_bottom = 2
	rail.mouse_filter = Control.MOUSE_FILTER_IGNORE
	track.add_child(rail)

	var head := ColorRect.new()
	head.color = DccTheme.c("accent")
	head.mouse_filter = Control.MOUSE_FILTER_IGNORE
	head.custom_minimum_size = Vector2(2, 13)
	track.add_child(head)
	_tl_track = track
	_tl_head = head
	_tl_transport.append(track)
	## Positioned on every resize as well as on every cursor move: the fraction
	## is a function of the year, but the pixel it lands on is a function of the
	## width, and this row sits between two resizable docks.
	track.resized.connect(_place_timeline_head)
	_place_timeline_head.call_deferred()

	track.gui_input.connect(func(ev: InputEvent):
		var at := -1.0
		if ev is InputEventMouseButton and ev.pressed \
				and ev.button_index == MOUSE_BUTTON_LEFT:
			at = ev.position.x
		elif ev is InputEventMouseMotion and (ev.button_mask & MOUSE_BUTTON_MASK_LEFT) != 0:
			at = ev.position.x
		if at < 0.0 or track.size.x <= 0.0:
			return
		var f: float = clampf(at / track.size.x, 0.0, 1.0)
		tl_set_year(TL_YEAR_MIN + int(round(f * float(TL_YEAR_MAX - TL_YEAR_MIN)))))
	return track

## The phone has no room for §4.2's row, so its timeline region carries the
## collapsed form only and the transport lives in `06-phone.md` §6.2's floating
## sim strip -- which `DccShell` builds and which this row is the only way in
## to. `phone_menu.gd` routes MORE ▸ Simulation to the CIVIL *Simulation*
## category, which is the collapse/recovery model and a different destination;
## it is left where it is.
func _fill_phone_timeline_row() -> void:
	var open := is_phone_sim_strip_open()
	var b := DccWidgets.text_button(timeline_row,
		"TIMELINE · %s" % (_tl_year_label() if tl_available() else "no world"),
		func(): set_phone_sim_strip_open(not is_phone_sim_strip_open()))
	_tl_phone_button = b
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.tooltip_text = "Close the year controls" if open else "Play, scrub and set the speed of the CIVIL year cursor."
	timeline_row.add_child(DccTheme.mono_label(
		DccIcons.SYMBOLS["chevron"] if open else "\u25b4", "text_faint", DccTheme.FS_MICRO, 0))

## `tlYearLabel`'s format is `UNSPECIFIED` in the prototype (§4.3 lists three
## candidates and settles none). This uses the CIVIL dock's own
## `_civFormatYear` grammar -- `412 AD`, `-400` as `400 BC` -- because that is
## already what the year pills beside it print, and two spellings of one year in
## one shell is worse than either spelling.
func _tl_year_label() -> String:
	var y := tl_year()
	return ("%d BC" % -y) if y < 0 else ("%d AD" % y)

## Everything on the strip that follows the cursor, the run state or the speed,
## written in place. Wired to `timeline_changed`, which every view of the cursor
## emits -- including the scrub track a few lines up, mid-drag, which is why
## this is a repaint and not a rebuild.
func _repaint_timeline() -> void:
	if timeline_row == null or not timeline_bar.visible:
		return
	## The row is shared with the journey planner (JP-13), which empties it
	## whole. Nothing held below survives that, and the strip has to be built
	## again rather than written to.
	##
	## **Testing the handles for emptiness could not see that happen.** The
	## planner removes and `queue_free`s the row's children without touching
	## this file's arrays, so `_tl_year_labels` stayed exactly as long as it
	## was -- holding Labels that were on their way out of the tree -- and the
	## guard written for precisely this case never fired once. The row's own
	## child count is the fact both owners agree on; the handle check stays
	## beside it for the case where the row was refilled by someone who did
	## not go through `_fill_timeline_strip()`.
	var have_handles := is_instance_valid(_tl_phone_button)
	if not have_handles:
		for l in _tl_year_labels:
			if is_instance_valid(l):
				have_handles = true
				break
	if timeline_row.get_child_count() == 0 or not have_handles:
		_fill_timeline_strip()
		return
	## **The gate.** `DccShell.tl_available()` is false until a world exists, and
	## before one `civ_goto_year` is a no-op -- so every transport control here
	## would answer a click with nothing at all. Drawn dead with the reason
	## instead, which is this shell's standing rule and, in this case, a rule a
	## probe run caught being broken: the strip accepted three cursor writes on a
	## world-less shell and read `0` back all three times.
	var live: bool = tl_available()
	for c in _tl_transport:
		if not is_instance_valid(c):
			continue
		if c is Button:
			(c as Button).disabled = not live
		if not live:
			c.tooltip_text = TL_UNAVAILABLE
	var label := _tl_year_label() if live else "no world"
	for l in _tl_year_labels:
		if is_instance_valid(l):
			l.text = label
	if is_instance_valid(_tl_phone_button):
		_tl_phone_button.text = "TIMELINE · %s" % label
	if is_instance_valid(_tl_state_label):
		_tl_state_label.text = tl_state_text() if live else "no world"
	if is_instance_valid(_tl_play_button):
		_tl_play_button.text = DccIcons.SYMBOLS["pause"] if tl_playing \
			else DccIcons.SYMBOLS["play"]
		if live:
			_tl_play_button.tooltip_text = ("Pause" if tl_playing else "Play") \
				+ " — %s. Each 600 ms tick moves the year cursor by the selected speed." % tl_state_text()
	for mult in _tl_speed_segments:
		var seg: Button = _tl_speed_segments[mult]
		if is_instance_valid(seg):
			DccWidgets.set_segment_on(seg, int(mult) == tl_speed)
	_place_timeline_head()

## `tlPct` is `UNSPECIFIED` in the prototype; §4.3 records `(tlYear + 400) /
## 1600` as the obvious candidate and states that it is not written down. It is
## used here because it is the only formula a fixed -400..1200 track admits, and
## it is named as recovered rather than as read.
func _place_timeline_head() -> void:
	if not is_instance_valid(_tl_track) or not is_instance_valid(_tl_head):
		return
	var f := float(tl_year() - TL_YEAR_MIN) / float(TL_YEAR_MAX - TL_YEAR_MIN)
	_tl_head.position = Vector2(_tl_track.size.x * f - 1.0, _tl_track.size.y * 0.5 - 6.5)
	_tl_head.size = Vector2(2, 13)

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
		centre.tooltip_text = CENTRE_TOOLTIP
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

## One text for both buttons that raise this pass -- the tool-options row above
## and `world_workspace.gd`'s WORLD-dock row, which calls straight into
## `_center_landmasses()`. Two copies of a disclosure is two chances for one of
## them to go stale.
##
## Everything after the first sentence is read off `lib.rs::center_landmasses`
## as it stands today, not off the design: it rotates ~20 grids in one pass and
## records a `ledger` row of kind `Recorded`, which is the ledger's own word for
## "seen in the history, not revertible" -- there is no height snapshot, because
## restoring the field alone would leave it misaligned with the nineteen grids
## beside it. `civ` and `sculpt` are set to `None` and the landmark store is
## invalidated. `labels`, `icons`, `infra` and the paint layers are **not**
## touched by that function, and every one of them is a grid coordinate into
## the grid that just rotated, so they are named here rather than left for the
## user to discover on the map.
const CENTRE_TOOLTIP := "The reference's #centerBtn. Rotates the world in longitude so the emptiest meridian sits at the map edge, then feathers the join it moved into the interior -- the wrapped map has no natural origin, so this is an equivalent world with the seam relocated into open ocean. Whole-world mode only; Region edges are hard borders, and a loaded save has no tectonic substrate to rotate.\n\nCannot be undone: the pass moves about twenty grids at once and only the height field can be snapshot, so Edit ▸ Undo records it without being able to revert it.\n\nDrops the civilisation layer and the sculpt draft, whose coordinates would no longer line up. Labels, map icons, infrastructure and painted layers are kept as they are and do NOT rotate with the terrain, so re-check anything you placed by hand."

## The reference's `#centerBtn`. Every outcome is reported in the status bar
## rather than silently: "already centred" (offset 0) is a real answer, and
## so is the Region-mode refusal the reference raises an `alert()` for.
##
## **Confirmed first.** This is the one field operation in the shell with no
## way back -- `carve_fjords` pushes a height snapshot and `Edit ▸ Undo`
## reverts it; this one records a `Recorded` ledger entry precisely because it
## cannot be reverted -- and it silently discards the civilisation layer and
## the sculpt draft on the way. A single unlabelled press of a toolbar button
## is not consent for that, and the prompt is the whole difference between an
## irreversible action and an accident.
func _center_landmasses() -> void:
	if not bridge.has_world:
		set_status("hint", "no world to centre", "accent")
		return
	_confirm("Center landmasses?",
		"This cannot be undone.\n\nThe civilisation layer and the sculpt draft are discarded. Labels, map icons, infrastructure and painted layers stay where they are and will no longer line up with the terrain.",
		"Rotate the world", _do_center_landmasses)

func _do_center_landmasses() -> void:
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
##
## **Re-based on `railFoot` (`ENV:1932`), stage 2.** That binding reads, in full:
##
##     s.domain==='WORLD' ? (wm==='b' ? 'SCULPT' : ('0'+(s.staleFrom||10)).slice(-2)+' / 10')
##   : s.domain==='CIVIL' ? this.cc().toUpperCase()
##   :                      this.ct().toUpperCase()
##
## i.e. the foot names the **mode**, not the domain -- which is the whole point
## of it now that each domain has one. The shipped version named the domain with
## three fixed strings ("TERRAIN" for WORLD, "CIVIL" for CIVIL, "STYLE" for
## CARTO), two of which were already lies about where the user was: CARTO said
## STYLE while the dock sat on Labels, and WORLD said TERRAIN while the pipeline
## was open.
##
## Two divergences from `ENV:1932`, both stated rather than silently taken:
##
## - **`staleFrom` has no counterpart here.** The prototype counts down from the
##   first stale stage, so its WORLD foot reads `03 / 10` after a stage-3 edit.
##   `bridge.stale_stages()` does expose that (`app.gd`'s own `SG-01` note), but
##   the foot is 84 px of rotated 9 px type that already carries the mode word,
##   and wiring a second live readout into it is dock work, not rail work. The
##   shipped `10`/`00` reading survives: "does a world exist at all".
## - **The mode word for WORLD `a` is the counter, not a word.** That is the
##   prototype's own asymmetry (`'SCULPT'` versus `'NN / 10'`), kept because the
##   counter is the more useful of the two and the node label already says
##   "Generation pipeline".
func _refresh_rail_foot() -> void:
	var domain := active_domain()
	var mode := active_mode(domain)
	var text := ""
	if domain == "world":
		text = "SCULPT" if mode == "b" else "%s / 10" % ("10" if bridge.has_world else "00")
	else:
		## `cc()`/`ct()` upper-cased, verbatim -- `landmarks` → `LANDMARKS`,
		## `style` → `STYLE`. Mode ids are already the prototype's own words.
		text = mode.to_upper()
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
##
## On phone this is also the *only* route back to `phone_project_picker.gd`
## after its own cold-start showing -- no second "Open project" entry was
## added to the File menu or the MORE tab; both already called this function,
## so branching here is the whole change (`phone_project_picker.gd`'s own
## header, "Dismissal").
func open_project_picker() -> void:
	if is_phone() and phone_project_picker != null:
		phone_project_picker.open()
	else:
		open_project_dialog.open()

## The welcome prompt, shown once on a cold start (see `_ready`). Desktop and
## tablet get `open_project_dialog.gd`'s welcome mode, unchanged -- see that
## file's header for why it is one screen with three actions rather than a
## second dialog in front of it. Phone gets its own entry screen instead
## (`phone_project_picker.gd`'s own header for the full reasoning: same
## recents, same New-world/Open-.zip flows, the locked spec's one-column card
## layout rather than the desktop gallery scaled down).
func open_welcome() -> void:
	if is_phone() and phone_project_picker != null:
		phone_project_picker.open()
	else:
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
			## `NewWorldDialog.heightmap_grid_summary()` -- `2048 x 1311 ->
			## working grid 1024 x 656` -- written for exactly this call site
			## and never called from it. Without it the first time a person
			## learns their image was resampled is when the world comes back
			## at a resolution they did not choose, and by then the import has
			## already committed. It returns `""` for every case where it has
			## no honest answer (no import API, unreadable file, an engine too
			## old to compute the grid), which is why there is no second guard
			## here: the empty string simply leaves the old sentence standing.
			var summary := new_world_dialog.heightmap_grid_summary(path)
			set_status("hint",
				("importing %s…" % path.get_file()) if summary == ""
					else "importing %s — %s" % [path.get_file(), summary],
				"text_ghost")
			bridge.import_heightmap(path, new_world_dialog.request()))

## The other end of `_project_documents()`: puts the archive's caller-owned
## documents back, and returns the sentences the person is owed about whatever
## did **not** come back.
##
## Called from `_load_project()` and from nowhere else, deliberately -- see the
## note in the `bridge.world_loaded` handler for the bug that put it there.
##
## Three of the five slots restore, and the shape of each answer is the
## engine's, not this function's:
##
##   * `entities/journeys.json` -- the shell's own; a wholesale replacement of
##     the planner's list, which is right here and only here, because this is
##     the moment the list is supposed to become the file's.
##   * `library/assets.json` -- comes back as pack info, collections, custom
##     slots, slot metadata and scatter rules. The item **images do not**, and
##     cannot: `project_bridge.rs::asset_library_document_json` states that the
##     record carries each item's `img` index while the bytes it points at have
##     no channel in the project writer. `items` is therefore 0 by design, and
##     that is a sentence, not a silence.
##   * `library/travel.json` -- replaces the custom half of every set and
##     leaves the stock entries alone, so one project's pack mule cannot
##     survive into the next.
##
## Two do not, and say so instead of pretending:
##
##   * `drafts/sculpt.json` -- carried in the archive and **not re-applied**.
##     The engine drops the Sculpt editor on every load (a save carries no
##     `river_mask`/`river_floor` for the draft's water hooks to adopt), so
##     `sculpt_restore_document` answers `ok == false` with that reason. The
##     stamps are still in the file; the person who drew them is owed the
##     sentence rather than an empty stamp list.
##   * `drafts/paint.json` -- carried, with no restore binding at all, for the
##     same reason: a project opened from disk has no `PaintEditor` to restore
##     into.
##
## Returns the notes rather than drawing them so `_load_project()` can compose
## one line with the format warning it already had.
func _restore_project_documents() -> PackedStringArray:
	var notes := PackedStringArray()
	var docs: Dictionary = bridge.last_documents
	if journey_planner_view != null:
		journey_planner_view.restore_journeys_document(
			String(docs.get("entities/journeys.json", "")))

	var travel := String(docs.get("library/travel.json", ""))
	if travel != "":
		var tr: Dictionary = bridge.travel_library_restore_document(travel)
		if not bool(tr.get("ok", false)):
			notes.append("the travel library could not be restored (%s)"
				% String(tr.get("error", "unknown")))
		else:
			var rejected: int = (tr.get("rejected", PackedStringArray()) as PackedStringArray).size()
			if rejected > 0:
				notes.append("%d travel-library field(s) in this archive are not recognised by this build" % rejected)

	var assets := String(docs.get("library/assets.json", ""))
	if assets != "":
		var ar: Dictionary = bridge.asset_library_restore_document(assets)
		if not bool(ar.get("ok", false)):
			notes.append("the asset library could not be restored (%s)"
				% String(ar.get("error", "unknown")))
		elif int(ar.get("slots", 0)) > 0 and int(ar.get("items", 0)) == 0:
			notes.append("the asset library came back as %d slot definition(s) without their images — a project archive carries the records, not the pixels, so re-import the source files to place them"
				% int(ar.get("slots", 0)))

	var sculpt := String(docs.get("drafts/sculpt.json", ""))
	if sculpt != "":
		var sr: Dictionary = bridge.sculpt_restore_document(sculpt)
		if not bool(sr.get("ok", false)):
			notes.append("the sculpt draft is still in this archive but was not re-applied (%s)"
				% String(sr.get("error", "unknown")))
		elif int(sr.get("stamps", 0)) > 0:
			notes.append("%d sculpt stamp(s) restored" % int(sr.get("stamps", 0)))

	if String(docs.get("drafts/paint.json", "")) != "":
		notes.append("the painted layers are carried in this archive but this build has no route to re-apply them to a loaded world")
	return notes

## Shared by the file-picker path above and `Data ▸ Recent worlds` / the
## Data manager window's own Import ▸ World Data route -- one place remembers
## `current_project_path` and updates the recent-projects list
## (`DCC_SHELL_SPEC.md` §2.1) so neither caller has to duplicate the
## bookkeeping.
func _load_project(path: String) -> void:
	if bridge.load_save(path):
		current_project_path = path
		DccSettings.remember_project(path)
		var notes := _restore_project_documents()
		## Say which format was read when it was the OLD one. The conversion is
		## one-way -- this build reads the reference app's flat layout and
		## writes only the tree -- so a user who round-trips through the
		## browser app needs to know before they save over it, not after.
		## `engine_bridge.gd`'s own comment has stated that rule since
		## 2026-08-26; it was never said to the person it affects.
		##
		## Ahead of the restore notes, not instead of them: a flat archive
		## carries no caller-owned documents at all, so the two sentences are
		## about different halves of the same open and both are owed.
		if bridge.last_open_layout == "flat":
			notes.insert(0, "the older flat format — saving converts it to the project format, which the browser app cannot reopen")
		elif not bridge.last_open_warnings.is_empty():
			notes.insert(0, String(bridge.last_open_warnings[0]))
		if not notes.is_empty():
			var line := "opened %s — %s" % [path.get_file(), " · ".join(notes)]
			set_status("hint", line, "accent")
			## The same phone-invisibility this file's three other failure
			## reports were fixed for: `hint` lives in the hidden
			## `PhoneMenuModel` host on a handset, and "your sculpt draft did
			## not come back" is precisely the sentence a person must not have
			## to go looking for.
			_show_phone_toast(line, null, 5.0)
		return
	## `GUI_GAP_REGISTER.md` **FI-04**. "see console" names somewhere the person
	## running an exported build cannot look, and it was the *only* thing said
	## about the commonest failure by far: `File ▸ Recent worlds` remembers a
	## path, not a file, so every row in it can outlive what it points at.
	## Measured on all three rows of a real recent list, every one naming a save
	## a previous session had since deleted -- the row attempted, failed and
	## reported a reason that was true of none of them.
	##
	## The distinction is one `file_exists` and it is worth making, because the
	## two have different answers: a missing file is the list's problem and a
	## refused one is the save's.
	if not FileAccess.file_exists(path):
		set_status("hint", "%s is no longer on disk" % path.get_file(), "accent")
	else:
		set_status("hint", "could not open %s — the engine refused the save" % path.get_file(), "accent")

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
	## **Never while a generate is in flight.** This is a background timer, so
	## unlike every other writer it can fire at any instant -- including while
	## the worker thread holds the engine object. Both halves of the write
	## take `&mut self` on the Rust side (`project_engine_built_documents`
	## builds four documents, `project_save_with_documents` serialises the
	## world), and reaching either through a `Gd<T>::bind()` the worker
	## already holds is the failure `EngineBridge.stale_stages()` refuses for
	## the same reason. The tick is skipped rather than deferred: the next one
	## is minutes away and the world is still dirty, so nothing is lost.
	if bridge.generating:
		return
	var target := current_project_path.get_basename() + ".autosave.zip"
	## Deliberately does **not** clear the dirty flag: the project itself is
	## still unsaved, and an autosave that made File ▸ Save look unnecessary
	## would be worse than no autosave.
	var was_dirty := bridge.world_dirty
	## Same writer AND the same contents as File ▸ Save: `_project_documents()`
	## is shared with `_write_project()`, and `project_save_with_documents` is
	## what `EngineBridge.save_project()` itself calls once there is anything
	## to carry.
	##
	## It used to be `bridge.world_gen.project_save(target)` -- the raw engine
	## call, chosen so the dirty flag survived -- and the comment above it
	## claimed that was "the same writer as File ▸ Save". It was not:
	## `project_save` is `project_save_with_documents(path, {})`, so every
	## autosave dropped the journeys, the paint layers, the sculpt draft and
	## both libraries, and an autosave written in a different format from the
	## manual save is exactly the trap that comment was warning about.
	##
	## `EngineBridge.project_save_with_documents()` rather than
	## `save_project()`, still deliberately: this is the one caller that must
	## NOT clear the dirty flag or emit `project_saved`, because the project
	## the user chose to keep is still unwritten. That wrapper's own doc names
	## this call site as its reason for existing.
	if bool(bridge.project_save_with_documents(target, _project_documents()).get("ok", false)):
		_mid_autosave_at = Time.get_time_string_from_system().substr(0, 5)
		set_status("autosave", "autosaved %s" % _mid_autosave_at, "text_faint")
	else:
		set_status("autosave", "autosave failed", "accent")
		## The third of the three phone-invisible failures. The `autosave` slot
		## is a status readout like the other two, so on a handset this said
		## nothing at all -- and the whole value of an autosave is that the
		## person believes it is running. Kept as a toast rather than routed
		## through `_report_failure()`: the `hint` slot belongs to whatever the
		## user last did by hand, and a background writer must not overwrite it.
		_show_phone_toast(
			"Autosave failed — %s could not be written. Your project itself is untouched."
				% target.get_file(), null, 4.5)
	_refresh_status_mid()
	## Restored through `mark_world_dirty()` rather than by writing the field,
	## so `dirty_changed` fires and the status bar cannot drift out of step
	## with the flag. A no-op on today's code path -- nothing between here and
	## `was_dirty` clears it -- and that is the point: it stays correct if
	## anything on that path ever starts clearing it.
	if was_dirty:
		bridge.mark_world_dirty()

func _refresh_save_status() -> void:
	if not DccSettings.autosave_enabled():
		set_status("autosave", "" if not bridge.world_dirty else "unsaved changes",
			"text_faint" if not bridge.world_dirty else "accent")
	elif current_project_path == "":
		set_status("autosave", "autosave waiting for a saved project", "text_ghost")
	else:
		set_status("autosave", "autosave every %d min" % DccSettings.autosave_minutes(), "text_faint")
	_refresh_status_mid()

## A failure the person has to see, on **every** composition.
##
## `set_status("hint", …)` on its own is a desktop-only report, and
## `UNWIRED_FUNCTIONS.md` registered exactly that: on a phone the status bar is
## parked inside the permanently hidden `PhoneMenuModel` host
## (`DccShell._build_phone_shell()`), so a hint written there is reachable only
## by opening MORE and going looking for it -- which nobody does after an action
## they believed had worked. `_show_phone_toast()` is the shell's own
## already-built pill (its three callers before this one were the undo chip's
## feedback and the two coach marks) and returns immediately off the phone, so
## one call site serves both compositions rather than a branch at each site.
##
## `null` for the anchor, deliberately: a failed generate or a refused write is
## not attached to any control, and `_position_phone_toast()` centres a toast
## that has nothing to point at.
##
## 4.5 s rather than the 2.8 s default -- these are sentences to read and act
## on, not the one-word acknowledgement the undo chip shows.
func _report_failure(text: String) -> void:
	set_status("hint", text, "accent")
	_show_phone_toast(text, null, 4.5)

## A conservative upper bound on the bytes a project save is about to need,
## read off what the writer actually stores rather than guessed.
##
## `project_save_with_documents` (`crates/cartalith-godot/src/project_bridge.rs`)
## hands `cartalith_io::SaveFields` five `Vec<f32>` rasters -- heightmap,
## temperature, rainfall, volcanic_field, impact_field -- plus one `u8`
## `strahler_order`, every one of them `gw * gh` long. That is 21 bytes per cell,
## and it over-states the file rather than under-stating it, because
## `cartalith-io`'s `zip_opts()` writes every entry `Deflated`.
##
## Over-stating is the right direction for a refusal. The archive is built whole
## in memory and written once -- that crate's own guarantee that a full disk
## cannot leave a half-written save over a good one -- so the peak demand really
## is the entire archive at once, and a save refused just under the line costs a
## second attempt while one allowed just over it costs the write.
##
## The 4 MiB is a flat allowance for the project layer: the civ roster, the
## timeline, the vault and the caller's own documents are JSON of no fixed size
## and are not modelled here. Named as an allowance rather than dressed up as a
## measurement.
const SAVE_BYTES_PER_CELL := 21
const SAVE_PROJECT_LAYER_ALLOWANCE := 4 * 1024 * 1024

func _save_bytes_needed() -> int:
	var g := bridge.grid_size()
	return g.x * g.y * SAVE_BYTES_PER_CELL + SAVE_PROJECT_LAYER_ALLOWANCE

## The storage-full guard. `grep` for disk / space / ENOSPC / free-bytes across
## this workspace found nothing at all before this, so a save onto a full volume
## failed inside the engine and surfaced as *"save failed -- see console"* on a
## handset that has no console.
##
## Returns `true` when the write should be refused, having already said why.
##
## **`-1` never blocks.** `EngineBridge.disk_free_bytes()` returns `-1` for "I
## do not know" -- an unmounted volume, a platform whose query refused, or a
## GDExtension older than the binding -- and treating that as "zero free" would
## refuse every save on any build that cannot answer. When it is unknown the
## save proceeds and the real failure surfaces through `_report_failure()`,
## which is the whole reason that path was made visible on the phone first.
func _save_blocked_by_space(path: String) -> bool:
	var free := bridge.disk_free_bytes(path)
	if free < 0:
		return false
	var need := _save_bytes_needed()
	if free >= need:
		return false
	var where := path.get_base_dir()
	_report_failure("Not enough space to save: about %s needed, %s free in %s. Free some space, or use File ▸ Save as… somewhere else."
		% [String.humanize_size(need), String.humanize_size(free),
			where if where != "" else path])
	return true

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

## Every caller-owned document that belongs in the archive, `{slot: json_text}`
## over the slots `bridge.project_document_slots()` registers.
##
## **Both writers call this** -- `_write_project()` (File ▸ Save / Save as…)
## and `_autosave_tick()`. They used to differ: the manual path collected
## journeys and the autosave path called the raw engine `project_save`, which
## `project_bridge.rs` implements as `project_save_with_documents(path, {})`
## -- an empty document map. So every autosave silently wrote an archive with
## no journeys, no paint layers, no sculpt draft and neither library in it,
## and the one archive a person reaches for after a crash was the lossy one.
##
## Two owners, merged here and nowhere else:
##
##   * the ENGINE's four (`drafts/paint.json`, `drafts/sculpt.json`,
##     `library/assets.json`, `library/travel.json`) via
##     `project_engine_built_documents()`, whose own doc calls itself "the
##     call a Save command should make"; a slot with nothing to write is
##     absent rather than empty, so this cannot pad the archive;
##   * the SHELL's one, `entities/journeys.json` -- a saved journey is a route
##     index plus a party form, both of which the engine deliberately does not
##     model.
##
## The two sets never collide (`project_engine_built_documents()`'s own
## guarantee: none of its four is a slot GDScript writes), so this is a merge
## and not a precedence question.
func _project_documents() -> Dictionary:
	var documents: Dictionary = bridge.project_engine_built_documents()
	if journey_planner_view != null:
		var doc := journey_planner_view.journeys_document()
		if doc != "":
			documents["entities/journeys.json"] = doc
	return documents

## The one place a project is actually written. Everything above routes here
## so the bookkeeping -- `current_project_path`, the recents list, the status
## line, the optional continuation -- happens once.
func _write_project(path: String, then: Callable = Callable()) -> void:
	var documents := _project_documents()
	## Checked before the writer is called, not after it fails: the engine
	## assembles the whole archive in memory first, so a doomed save on a full
	## volume costs a full serialisation before anyone hears about it.
	if _save_blocked_by_space(path):
		return
	if not bridge.save_project(path, documents):
		## **Was "save failed -- see console".** That named somewhere an exported
		## Android build has no way to reach, which is the half of
		## `BUILD_ANSWERS.md` §4's storage/failure gap that cost nothing to fix.
		## What replaces it says the one thing that is certain -- the engine
		## builds the archive whole and writes it once, so an existing file at
		## this path is untouched (`project_bridge.rs`'s own guarantee) -- and
		## then names the two things a person can actually do. The specific
		## engine-side reason is pushed as a warning by
		## `EngineBridge.save_project()` and is not returned to this caller;
		## see the note in the return report for the one-line change that would
		## let this sentence carry it.
		_report_failure("Could not save %s. Nothing already on disk was changed. Check the folder still exists and can be written to, or use File ▸ Save as… to pick another."
			% path.get_file())
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
## Branches 2 and 3 above, as one predicate, so a test can exercise the real
## condition instead of restating it.
##
## `_backnav_probe.gd`'s escape-hatch checks used to assign `_quit_asked` and
## `_quit_prompt` and then assert on an expression they had just written by
## hand -- a tautology that passed whatever `_close_requested()` actually did,
## and would have gone on passing if this condition were inverted. There is
## exactly one copy of it now, and it is this one.
##
## **Precondition: the caller has already ruled out branch 1.** A visible
## prompt is handled before this is asked, so "we already asked" here always
## means "and nothing is on screen".
func _would_quit_immediately() -> bool:
	return _quit_asked or not bridge.has_world

func _close_requested() -> void:
	if is_instance_valid(_quit_prompt) and _quit_prompt.visible:
		_quit_prompt.grab_focus()
		return
	if _would_quit_immediately():
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
## information panel, not in an isolated utility). `kind` is one of
## `bridge.vault_entity_kinds()` -- `settlement`, `province`, `continent`,
## `faction` and `culture` on this build, and read from the engine rather than
## transcribed, because that list has already grown twice. `entity_id` is that
## kind's own id — a settlement's **tid**, not its index into
## `bridge.settlements()`.
func open_vault(kind: String, entity_id: int, label: String) -> void:
	vault_window.open_for(kind, entity_id, label)

## The same panel with no entity scope: the whole link store.
func open_vault_overview() -> void:
	vault_window.open_overview()

func open_performance() -> void:
	performance_window.open()

func open_gen_info() -> void:
	gen_info_dialog.open()

## `Help ▸ Keyboard shortcuts`. The dialog reads the accelerators off the live
## menus every time it opens, so there is no table here to keep in step.
func open_shortcuts() -> void:
	shortcuts_dialog.open()

func toggle_resource_overlay() -> void:
	resource_overlay.toggle()

## `Data ▸ ⧉ Data manager` and the Region-select handoff converge here --
## `group` is one of the four `DataManagerWindow.GROUP_ORDER` names, empty
## opens the window on its first route.
func open_data_manager(group: String = "") -> void:
	data_manager_window.open(group)

## The Data dropdown's fourteen route rows (`menus.gd::_data()`, rebuilt
## against `DCC shell tablet 2560` on 2026-08-25). A group name picks that
## group's first route; this picks the exact one the row names.
func open_data_manager_route(route_id: String) -> void:
	data_manager_window.open_route(route_id)

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
	## ED-02: the ledger lost a row, so the panel showing it is stale. A no-op
	## unless History is the live context.
	if right_dock_ctrl != null:
		right_dock_ctrl.refresh_history()

## The mirror of `undo_last()`, and the destination the menu bar's `↷` square
## calls (`DccShell._menu_bar_redo()`). Same two lines of repaint, same reason:
## write `map_view.texture` directly rather than calling `ViewportHost.refresh()`,
## which would also reset the camera.
##
## Guarded on the binding rather than assumed present, exactly as `menus.gd`'s
## own `Redo` row is: `EngineBridge.redo_last()` answers `false` on a cdylib that
## predates the global redo cursor, and the square in front of this is already
## drawn disabled with that reason when it does.
##
## `menus.gd::_redo_last()` is the same operation written a second time, in the
## file that owns `Edit ▸ Redo`. It should delegate here so there is one redo
## path the way there is one undo path; that file is not this pass's to edit.
func redo_last() -> void:
	if not bridge.redo_available():
		set_status("hint", "Nothing to redo.", "text_ghost")
		return
	var label := bridge.redo_label()
	if not bridge.redo_last():
		set_status("hint", "Nothing to redo.", "text_ghost")
		return
	if viewport != null:
		viewport.map_view.texture = bridge.color_texture()
		viewport.set_preview_texture(null)
	set_status("pass", "redid %s" % label.to_lower(), "text_dim")
	## ED-02: the ledger cursor moved, so the panel showing it is stale. A no-op
	## unless History is the live right-dock context.
	if right_dock_ctrl != null:
		right_dock_ctrl.refresh_history()

## `Edit ▸ Undo history…` (`GUI_GAP_REGISTER.md` **ED-02**). A right-dock
## context, per `DCC_SHELL_SPEC.md` §7.1 proposal 3 -- selection-adjacent, and
## the dock is already the context-driven surface. Not a window: a window for
## a list that is read *against* the map would cover the map.
func open_undo_history() -> void:
	if right_dock_ctrl != null:
		right_dock_ctrl.show_history()

func show_project_on_disk() -> void:
	reveal_on_disk(current_project_path)

## Show `path` in the desktop file manager. Extracted from
## `show_project_on_disk()` so an export can end somewhere rather than in
## silence: `data_manager_window.gd` reported how many pixels and how many
## seconds and never where the file went.
##
## **Desktop only, and that is deliberate rather than an omission.** Android
## has no file manager to hand a path to -- `DisplayServer` reports the
## capability but the intent lands nowhere useful -- so on a phone or tablet
## the caller's status line, which names the file, is the whole answer.
## Returns whether it actually did anything, so a caller can word its status
## accordingly instead of promising a window that never opens.
func reveal_on_disk(path: String) -> bool:
	if path == "" or DccTheme.is_touch():
		return false
	if OS.has_method("shell_show_in_file_manager"):
		OS.shell_show_in_file_manager(path)
	else:
		OS.shell_open("file://" + path.get_base_dir())
	return true

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
## **The body was empty, and the reason is the order of the last four lines.**
## `_ready()` fires when a node ENTERS THE TREE, and `add_child(dlg)` is what
## puts it there -- so by the time `set_script()` attached `credits.gd`, that
## script's `_ready()` had already missed its only chance to run. Attaching a
## script to a node already inside the tree does not re-run it. The window
## therefore opened with a correctly built, correctly named, entirely blank
## `RichTextLabel` (parallel phone sweep, 2026-08-25, and true on the desktop
## build just as much -- this was never a phone bug).
##
## Fixed by attaching the script BEFORE the node enters the tree, which also
## means `owner`/`unique_name_in_owner` -- the two lines `%CreditsText` needs --
## have to be set before that. The comment this replaces had the dependency
## exactly backwards: the body has to exist before the script runs, which it
## does either way, because both are built here and neither needs the tree.
##
## Held in `_credits_dialog` rather than dropped on the floor: a second Help ▸
## Credits used to build a second dialog and leave the first one parented
## forever, and nothing in this file could close either.
var _credits_dialog: AcceptDialog = null

func open_credits() -> void:
	if is_instance_valid(_credits_dialog):
		_present_credits()
		return
	var dlg := AcceptDialog.new()
	dlg.title = "Credits & academic principles"
	dlg.size = Vector2i(720, 640)
	dlg.ok_button_text = "Close"
	## PH-12. A phone gets the shared treatment: borderless, a content-scaled
	## fill and the tap floor. The two authored minimums go with the desktop
	## branch -- 680x560 and a 660 px measure inside a 393 dp column would widen
	## the window past the screen, and a `Window` cannot be narrower than its
	## content's minimum. `phone_window()` also turns `wrap_controls` off, which
	## a `fit_content` `RichTextLabel` of this length badly needed.
	var phone := DccWidgets.phone_window(dlg, self)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var text := RichTextLabel.new()
	text.name = "CreditsText"
	text.bbcode_enabled = true
	text.fit_content = true
	text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(text)
	if phone:
		var outer := VBoxContainer.new()
		outer.add_theme_constant_override("separation", 0)
		DccWidgets.phone_head(outer, "Credits", "attribution & academic principles")
		outer.add_child(scroll)
		dlg.add_child(outer)
	else:
		scroll.custom_minimum_size = Vector2(680, 560)
		text.custom_minimum_size.x = 660
		dlg.add_child(scroll)
	text.owner = dlg
	text.unique_name_in_owner = true
	dlg.set_script(load("res://credits.gd"))
	_credits_dialog = dlg
	add_child(dlg)
	if phone:
		phone_fit(dlg, 1.0)
	_present_credits()

func _present_credits() -> void:
	if DccWidgets.phone_present(_credits_dialog, self):
		return
	_credits_dialog.popup_centered()

func open_about() -> void:
	var d := AcceptDialog.new()
	d.title = "About Cartalith"
	d.dialog_text = "Cartalith — native port of Cartalith Gen1 v2.10.\nGodot %s · %s" % [
		Engine.get_version_info().string, OS.get_name()]
	add_child(d)
	d.popup_centered()

## The five `Window ▸` region rows plus `Reset layout`.
##
## **Three of the six needed a phone branch, and `UNWIRED_FUNCTIONS.md`
## registered all three as dangerous-class: drawn live, doing the wrong thing.**
## The desktop model -- a region is a node and the row flips its `visible` --
## holds for the timeline and the domain rail in every composition, and breaks
## for the other three on a handset:
##
##   - **Left dock / Right dock.** On the phone these two nodes are the sheets,
##     and a sheet's truth is not its `visible` flag but `DccShell`'s
##     `_left_sheet_open`/`_right_sheet_open`, which the system-back chain
##     (`DccShell._phone_back()`) tests to decide whether back has a level to
##     leave. Writing `visible` behind the flag's back put a full-screen sheet
##     on the map that **back would not close** -- the one gesture Android users
##     have no alternative to. Routed through `_set_sheet_open()`, which owns
##     both halves and also closes whatever overlay was in front of it.
##   - **Status bar.** The desktop status strip is parked inside the hidden
##     `PhoneMenuModel` host on a phone, so the row moved a check mark over a
##     node that could never become visible. See
##     `DccShell.set_status_region_shown()` for what the row acts on instead and
##     why that surface is a setter pair rather than a node.
##
## `Reset layout` compounded all three: it re-showed every mapped node, which on
## a phone meant both sheets drawn at once (they are mutually exclusive) with
## both open-flags still `false`. Its phone form closes them instead, which is
## the layout the phone boots into.
func toggle_region(id: int) -> void:
	if id == DccMenus.ID_WIN_RESET:
		if is_phone():
			_set_sheet_open("left", false)
			_set_sheet_open("right", false)
			set_status_region_shown(true)
		for node in _region_nodes.values():
			(node as CanvasItem).visible = true
		## `resetlayout` (`ENV:2052`) writes `railExp:false` alongside the five
		## region flags, and it has to: the expansion column is not one of the
		## regions this loop re-shows, so a reset that left it open would leave
		## the one piece of chrome the user cannot reach from this menu in
		## whatever state they had put it in. Cheap, and it makes "reset" mean
		## the layout the shell boots into.
		set_rail_expanded(false)
		return
	## The three phone-conditional rows. `_region_nodes` carries no entry for
	## them on a phone (see where it is built), so this is the only path they
	## can take there and the desktop path is untouched.
	if is_phone():
		if id == DccMenus.ID_WIN_LEFT:
			_set_sheet_open("left", not _left_sheet_open)
			return
		if id == DccMenus.ID_WIN_RIGHT:
			_set_sheet_open("right", not _right_sheet_open)
			return
		if id == DccMenus.ID_WIN_STATUS:
			set_status_region_shown(not is_status_region_shown())
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

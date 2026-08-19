extends RefCounted
class_name DccMenus

## The seven program menus (`DCC_SHELL_SPEC.md` §2).
##
## File · Edit · Assets · Data · Preferences · Window · Help -- and nothing
## else. Generate, Simulate, Render and View were menus in the previous shell
## and are workspaces on the domain rail here; that is the structural change
## this revision makes, and the reason there is no eighth entry.
##
## Honesty rule, inherited from the old `main.gd`: an item with no engine
## behind it is added **disabled**, with a tooltip that says what is missing.
## It is never added enabled and silently inert, and never omitted -- the menu
## is also the map of what the port still owes.

const ID_NEW_WORLD := 10
const ID_OPEN_PROJECT := 11
const ID_SAVE := 12
const ID_SAVE_AS := 13
const ID_REVERT := 15
const ID_CLOSE := 16
const ID_STORAGE := 17
const ID_SHOW_ON_DISK := 19

const ID_UNDO := 20
const ID_REDO := 21

const ID_ASSET_LIBRARY := 30
const ID_SLICER := 31
const ID_IMPORT_IMAGE := 32
const ID_IMPORT_PACK := 33
const ID_APPLY_LIBRARY := 36
const ID_CLEAR_LIBRARY := 37

const ID_DATA_MANAGER := 40
const ID_JOURNEY_PLANNER := 41
const ID_DATA_MGR_IMPORT := 42
const ID_DATA_MGR_EXPORT := 43
const ID_DATA_MGR_SOURCES := 44
const ID_DATA_MGR_CONVERSION := 45
const ID_DATA_MGR_VALIDATION := 46

const ID_PREF_GPU := 50
const ID_PREF_THEME_DARK := 51
const ID_PREF_THEME_LIGHT := 52
const ID_PREF_QUALITY := 53
const ID_PREF_UNITS_KM := 54
const ID_PREF_UNITS_MI := 55
const ID_PREF_STORAGE := 56

const ID_WIN_LEFT := 60
const ID_WIN_RIGHT := 61
const ID_WIN_TIMELINE := 62
const ID_WIN_STATUS := 63
const ID_WIN_RAIL := 64
const ID_WIN_RESET := 65

const ID_HELP_CREDITS := 70
const ID_HELP_ABOUT := 71
const ID_HELP_SHORTCUTS := 72

var _shell: DccShell
var _bridge: EngineBridge
var _host: Node                 ## Where dialogs are parented and callbacks live.
var _quality_popup: PopupMenu
var _recent_popup: PopupMenu
var _icon_families_popup: PopupMenu
var _texture_sets_popup: PopupMenu

func build(shell: DccShell, bridge: EngineBridge, host: Node) -> void:
	_shell = shell
	_bridge = bridge
	_host = host
	shell.add_menu("File", _file)
	shell.add_menu("Edit", _edit)
	shell.add_menu("Assets", _assets)
	shell.add_menu("Data", _data)
	shell.add_menu("Preferences", _preferences)
	shell.add_menu("Window", _window)
	shell.add_menu("Help", _help)

## Add an item the port cannot yet honour. Disabled, with the reason attached,
## so the menu never promises behaviour that does not exist.
func _todo(p: PopupMenu, text: String, why: String) -> void:
	p.add_item(text)
	var i := p.item_count - 1
	p.set_item_disabled(i, true)
	p.set_item_tooltip(i, why)

func _live(p: PopupMenu, text: String, id: int, accel: Key = KEY_NONE) -> void:
	p.add_item(text, id)
	if accel != KEY_NONE:
		p.set_item_accelerator(p.item_count - 1, accel)

# -- §2.1 File ----------------------------------------------------------------

func _file(p: PopupMenu) -> void:
	_live(p, "New world…", ID_NEW_WORLD, KEY_MASK_CTRL | KEY_N)
	_live(p, "Open project…", ID_OPEN_PROJECT, KEY_MASK_CTRL | KEY_O)

	## A real submenu, per §2.1: "last 10 projects, path shown as secondary
	## text." Rebuilt on every `about_to_popup` -- unlike `_quality_popup`'s
	## fixed tier list, the recent list changes as projects are opened, so
	## the cached-once-at-build-time pattern those other submenus use would
	## go stale.
	_recent_popup = PopupMenu.new()
	_recent_popup.name = "RecentWorlds"
	_shell.style_popup(_recent_popup)
	p.add_child(_recent_popup)
	p.add_submenu_item("Recent worlds", "RecentWorlds")
	_recent_popup.id_pressed.connect(_on_recent_world)

	p.add_separator()
	_todo(p, "Save project", "The engine reads .zip saves but does not write them yet (cartalith-io is read-only).")
	_todo(p, "Save as…", "Same: no save writer yet.")
	_todo(p, "Autosave", "Requires a save writer.")
	_todo(p, "Revert to last save", "Requires a save writer.")
	p.add_separator()
	_todo(p, "Close project", "No project lifecycle yet; the shell holds one world at a time.")
	p.add_separator()

	## One item, one dialog with an inline Browse… per root (`DccApp.
	## open_storage_locations()`) -- was two items (a read-only list plus a
	## separate "Change locations…" item) opening two dialogs that showed the
	## same four rows; merged on owner feedback (2026-08-19) as redundant
	## menu surface, not two distinct capabilities.
	_live(p, "Storage locations", ID_STORAGE)
	_live(p, "Show project on disk", ID_SHOW_ON_DISK)
	var show_idx := p.item_count - 1
	p.set_item_tooltip(show_idx,
		"Reveals the project's folder in the OS file manager. Disabled until a project has been opened this session.")

	p.add_separator()
	## §2.1's static note: imports do not live in File.
	p.add_item("Imports live under Data ▸ Import; asset packs under Assets")
	p.set_item_disabled(p.item_count - 1, true)

	p.about_to_popup.connect(func():
		_refresh_recent_worlds()
		p.set_item_disabled(show_idx, _host.current_project_path == ""))
	p.id_pressed.connect(_on_file)

func _refresh_recent_worlds() -> void:
	_recent_popup.clear()
	var recents: Array = DccSettings.recent_projects()
	if recents.is_empty():
		_recent_popup.add_item("No recent projects")
		_recent_popup.set_item_disabled(0, true)
		_recent_popup.set_item_tooltip(0, "Projects are opened by path -- this fills in as File ▸ Open project… is used.")
		return
	for i in recents.size():
		var path := String(recents[i])
		_recent_popup.add_item(path.get_file(), i)
		_recent_popup.set_item_tooltip(i, path)

func _on_recent_world(id: int) -> void:
	var recents: Array = DccSettings.recent_projects()
	if id >= 0 and id < recents.size():
		_host.open_recent_project(String(recents[id]))

func _on_file(id: int) -> void:
	match id:
		ID_NEW_WORLD: _host.open_new_world()
		ID_OPEN_PROJECT: _host.open_project_picker()
		ID_STORAGE: _host.open_storage_locations()
		ID_SHOW_ON_DISK: _host.show_project_on_disk()

# -- §2.2 Edit ----------------------------------------------------------------

func _edit(p: PopupMenu) -> void:
	_todo(p, "Undo", "No undo stack yet -- generation is one-shot and sculpt has no Godot binding.")
	_todo(p, "Redo", "Same.")
	_todo(p, "Undo history…", "Same.")
	p.add_separator()
	_todo(p, "Cut", "Nothing is selectable for editing yet beyond settlements, which are read-only.")
	_todo(p, "Copy", "Same.")
	_todo(p, "Paste", "Same.")
	_todo(p, "Delete", "Same.")
	p.add_separator()
	_todo(p, "Select all", "Same.")
	_todo(p, "Deselect", "Same.")
	p.add_separator()
	_todo(p, "Find on map…", "No search index yet; settlement search lives in the Data manager.")

# -- §2.3 Assets --------------------------------------------------------------

func _assets(p: PopupMenu) -> void:
	_live(p, "%s Asset library" % DccIcons.SYMBOLS["panels"], ID_ASSET_LIBRARY, KEY_MASK_SHIFT | KEY_A)
	_live(p, "%s Sprite sheet slicer" % DccIcons.SYMBOLS["panels"], ID_SLICER)
	p.add_separator()
	_todo(p, "Import image…",
		"The asset-library window (§8) is built, but landing a loose image in an Unassigned-imports custom slot needs AssetDB::addCustomSlot -- no #[func] exposes it.")
	_live(p, "Import asset pack .zip…", ID_IMPORT_PACK)
	p.add_separator()

	## §2.3: "Submenu listing the 24 families with filled/capacity counts."
	## `cartalith-assets` ships EIGHT families, not 24 -- verified reading
	## `slots.rs`/`library.rs` directly (`ASSET_LIBRARY_SCOPE.md` §1 already
	## recorded this: "eight families, seven of them closed vocabularies").
	## These two submenus list the real eight, split the way the crate itself
	## splits them (`Family::is_texture()`); each entry is a real scoped-open
	## shortcut into `AssetLibraryWindow` -- capacity is real (the frozen slot
	## count), fill count is not shown because no query for it is exposed.
	_icon_families_popup = PopupMenu.new()
	_icon_families_popup.name = "IconFamilies"
	_shell.style_popup(_icon_families_popup)
	p.add_child(_icon_families_popup)
	p.add_submenu_item("Icon families", "IconFamilies")
	var icon_keys: Array[String] = []
	for fam in AssetLibraryWindow.FAMILIES:
		if not bool(fam.get("texture", false)):
			_icon_families_popup.add_item(
				"%s (%d)" % [String(fam["title"]), (fam["slots"] as Array).size()], icon_keys.size())
			icon_keys.append(String(fam["key"]))
	_icon_families_popup.id_pressed.connect(func(i: int): _host.open_asset_library(icon_keys[i]))

	_texture_sets_popup = PopupMenu.new()
	_texture_sets_popup.name = "TextureSets"
	_shell.style_popup(_texture_sets_popup)
	p.add_child(_texture_sets_popup)
	p.add_submenu_item("Texture sets", "TextureSets")
	var tex_keys: Array[String] = []
	for fam in AssetLibraryWindow.FAMILIES:
		if bool(fam.get("texture", false)):
			_texture_sets_popup.add_item(
				"%s (%d)" % [String(fam["title"]), (fam["slots"] as Array).size()], tex_keys.size())
			tex_keys.append(String(fam["key"]))
	_texture_sets_popup.id_pressed.connect(func(i: int): _host.open_asset_library(tex_keys[i]))

	p.add_separator()
	_todo(p, "Apply library to map",
		"Verified against the live engine (2026-08-19): cartalith-godot exposes load_asset_pack(path) only -- loading a pack FROM DISK. There is no in-memory library-editing session on the Godot side to compile and apply.")
	_todo(p, "Clear library…",
		"Same verification: no AssetDB.clear() equivalent is exposed -- there is no live library state here to clear.")
	p.id_pressed.connect(_on_assets)

func _on_assets(id: int) -> void:
	match id:
		ID_IMPORT_PACK: _host.open_asset_pack_picker()
		ID_ASSET_LIBRARY: _host.open_asset_library()
		ID_SLICER: _host.open_asset_library("", true)

# -- §2.4 Data ----------------------------------------------------------------
#
# §2.4: the dropdown mirrors the Data manager window's five groups and is a
# shortcut into it, never a second implementation. The window (`§9`,
# `data_manager_window.gd`) now exists -- each group item below opens it
# scoped to that group's first route. Most routes inside are still disclosed
# gaps (see that file's own header comment for the full breakdown); what
# changed here is that the item now opens a real window that is honest about
# which of its own routes work, rather than being disabled at the menu level.

## §2.4's own 2026-08-19 addition (`DCC_SHELL_SPEC.md`, reconciled from
## `JOURNEY_PLANNER_SPEC.md`): Journey planner sits above the five Data
## manager groups, alongside World data tables, and arms the INFRA JOURNEY
## tool takeover rather than opening a window -- Travel library (⇧L) is the
## sibling addition that DOES stay a real window, and is separate, later work
## per this port's own scope note; not added here.
func _data(p: PopupMenu) -> void:
	_live(p, "World data tables…", ID_DATA_MANAGER)
	_live(p, "Journey planner…", ID_JOURNEY_PLANNER, KEY_MASK_SHIFT | KEY_J)
	p.add_separator()
	_live(p, "Import ▸ Maps · Heightmaps · GIS · World data", ID_DATA_MGR_IMPORT)
	_live(p, "Export ▸ Maps · GIS · World data · Asset pack", ID_DATA_MGR_EXPORT)
	_live(p, "Sources ▸ External · Connected · Registry", ID_DATA_MGR_SOURCES)
	_live(p, "Conversion ▸ Coordinate systems · Formats", ID_DATA_MGR_CONVERSION)
	_live(p, "Validation ▸ Check data · Repair", ID_DATA_MGR_VALIDATION)
	p.id_pressed.connect(func(id: int) -> void:
		match id:
			ID_DATA_MANAGER: _host.open_world_data()
			ID_JOURNEY_PLANNER: _host.open_journey_planner()
			ID_DATA_MGR_IMPORT: _host.open_data_manager("Import")
			ID_DATA_MGR_EXPORT: _host.open_data_manager("Export")
			ID_DATA_MGR_SOURCES: _host.open_data_manager("Sources")
			ID_DATA_MGR_CONVERSION: _host.open_data_manager("Conversion")
			ID_DATA_MGR_VALIDATION: _host.open_data_manager("Validation")
	)

# -- §2.5 Preferences ---------------------------------------------------------

func _preferences(p: PopupMenu) -> void:
	## `use_gpu` is a plain entry in the engine's own flat parameter table
	## (`params.rs`) -- `bridge.param_set("use_gpu", ...)` is the whole
	## mechanism, identical to every slider in the Generation Pipeline. It was
	## shipped disabled here on the mistaken assumption that it needed its own
	## binding; it does not, and disabling a control that already works is
	## worse than the gap it was meant to be honest about.
	##
	## Defaulting it ON in the shell (see `EngineBridge._ready`) is a UI-layer
	## decision, not an engine one: `WorldParams::default()` stays `false` in
	## Rust so golden-parity tests keep pinning the CPU path exactly as
	## `GPU_LAYER_INTEGRATION_SCOPE.md` requires. The tooltip below is the
	## "honest 'this may produce a different world' messaging" that same
	## document says a GPU toggle needs before it can be user-facing at all
	## (`DECISIONS.md` §7c: GPU-path noise is genuinely different, not
	## tolerance-different, for the same seed).
	p.add_check_item("GPU acceleration", ID_PREF_GPU)
	var gpu_idx := p.item_count - 1
	p.set_item_checked(gpu_idx, bool(_bridge.param_get("use_gpu")))
	p.set_item_tooltip(gpu_idx,
		"Runs domain warp, crustal heterogeneity, plate assignment and flow accumulation on the GPU. A given seed produces a genuinely different (not just faster) world with this on vs. off -- both are valid, but they don't match each other. Takes effect on the next generate.")
	p.about_to_popup.connect(func():
		p.set_item_checked(gpu_idx, bool(_bridge.param_get("use_gpu"))))
	_todo(p, "Devices", "Multi-GPU device selection is not exposed by the engine yet.")
	_todo(p, "Multi-GPU mode", "Same.")
	_todo(p, "CPU worker threads", "Rayon sizes its own pool; no override is exposed.")
	_todo(p, "VRAM budget", "Not exposed.")
	p.add_separator()

	## Render quality is real: the engine ships named tiers and recommends one.
	_quality_popup = PopupMenu.new()
	_quality_popup.name = "QualityTiers"
	var tiers := _bridge.quality_tiers()
	var current := _bridge.quality_tier()
	for i in tiers.size():
		_quality_popup.add_radio_check_item(tiers[i], i)
		_quality_popup.set_item_checked(i, tiers[i] == current)
	_quality_popup.id_pressed.connect(_on_quality)
	_shell.style_popup(_quality_popup)
	p.add_child(_quality_popup)
	p.add_submenu_item("Render quality", "QualityTiers")

	_todo(p, "Anti-aliasing · anisotropy", "The 2D map path does not sample-antialias; this belongs to the 3D viewport, which is not built.")
	_todo(p, "Colour management", "The renderer is sRGB-only today.")
	_todo(p, "3D viewport defaults", "No 3D viewport yet.")
	_todo(p, "Lighting rig defaults", "No lighting rig yet.")
	p.add_separator()
	_todo(p, "Tiled LOD · tile size · atlas cache", "No tile atlas yet.")
	_todo(p, "Undo history", "No undo stack yet.")
	p.add_separator()

	## §2.5's Application group: "Storage locations… — Same modal as File."
	## Genuinely the same dialog `File ▸ Storage locations` opens --
	## `DccApp.open_storage_locations()` is the one method both call.
	_live(p, "Storage locations…", ID_PREF_STORAGE)

	var theme_menu := PopupMenu.new()
	theme_menu.name = "ThemeChoice"
	theme_menu.add_radio_check_item("Dark", ID_PREF_THEME_DARK)
	theme_menu.add_radio_check_item("Light", ID_PREF_THEME_LIGHT)
	theme_menu.set_item_checked(0, DccTheme.is_dark())
	theme_menu.set_item_checked(1, not DccTheme.is_dark())
	theme_menu.set_item_disabled(1, true)
	theme_menu.set_item_tooltip(1,
		"The light palette is defined (DccTheme.LIGHT) but the shell builds its styleboxes once at startup; live swapping needs a rebuild pass.")
	_shell.style_popup(theme_menu)
	p.add_child(theme_menu)
	p.add_submenu_item("Theme", "ThemeChoice")
	_todo(p, "Units", "The shell is km-only; the reference's mi toggle is not ported.")
	_todo(p, "Keyboard shortcuts…", "No shortcut table yet.")
	p.id_pressed.connect(_on_preferences.bind(p))

func _on_preferences(id: int, p: PopupMenu) -> void:
	if id == ID_PREF_STORAGE:
		_host.open_storage_locations()
		return
	if id != ID_PREF_GPU:
		return
	var idx := p.get_item_index(ID_PREF_GPU)
	var on := not bool(_bridge.param_get("use_gpu"))
	if _bridge.param_set("use_gpu", on):
		p.set_item_checked(idx, on)

func _on_quality(id: int) -> void:
	var tiers := _bridge.quality_tiers()
	if id < 0 or id >= tiers.size():
		return
	if _bridge.set_quality_tier(tiers[id]):
		for i in tiers.size():
			_quality_popup.set_item_checked(i, i == id)
		_bridge.mark_dirty()

# -- §2.6 Window --------------------------------------------------------------

func _window(p: PopupMenu) -> void:
	for entry in [
		["Left dock", ID_WIN_LEFT], ["Right dock", ID_WIN_RIGHT],
		["Timeline", ID_WIN_TIMELINE], ["Status bar", ID_WIN_STATUS],
		["Domain rail", ID_WIN_RAIL],
	]:
		p.add_check_item(entry[0], entry[1])
		p.set_item_checked(p.item_count - 1, true)
	p.add_separator()
	_live(p, "Reset layout", ID_WIN_RESET)
	_todo(p, "Save layout as…", "No layout store yet.")
	p.id_pressed.connect(func(id: int): _host.toggle_region(id))

# -- §2.7 Help ----------------------------------------------------------------

func _help(p: PopupMenu) -> void:
	_todo(p, "Documentation", "No in-app documentation yet; the repository docs are the reference.")
	_todo(p, "Keyboard shortcuts", "No shortcut table yet.")
	_live(p, "Credits & academic principles", ID_HELP_CREDITS)
	_todo(p, "Report an issue", "No issue route wired.")
	_live(p, "About", ID_HELP_ABOUT)
	p.id_pressed.connect(_on_help)

func _on_help(id: int) -> void:
	match id:
		ID_HELP_CREDITS: _host.open_credits()
		ID_HELP_ABOUT: _host.open_about()

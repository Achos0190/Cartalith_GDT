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
const ID_AP_VALIDATE := 38
const ID_AP_EXPORT := 39
const ID_AP_PACK_META := 49

const ID_DATA_MANAGER := 40
const ID_JOURNEY_PLANNER := 41
const ID_DATA_MGR_IMPORT := 42
const ID_DATA_MGR_EXPORT := 43
const ID_DATA_MGR_SOURCES := 44
const ID_DATA_MGR_VALIDATION := 46
const ID_TRAVEL_LIBRARY := 48

const ID_PREF_GPU := 50
const ID_PREF_THEME_DARK := 51
const ID_PREF_THEME_LIGHT := 52
const ID_PREF_QUALITY := 53
const ID_PREF_UNITS_KM := 54
const ID_PREF_UNITS_MI := 55
const ID_PREF_STORAGE := 56
const ID_PREF_WORKING_SET := 57
const ID_PREF_THEME_SYSTEM := 58

const ID_WIN_LEFT := 60
const ID_WIN_RIGHT := 61
const ID_WIN_TIMELINE := 62
const ID_WIN_STATUS := 63
const ID_WIN_RAIL := 64
const ID_WIN_RESET := 65
const ID_WIN_DIAG_OVERLAY := 66

const ID_HELP_CREDITS := 70
const ID_HELP_ABOUT := 71
const ID_HELP_SHORTCUTS := 72
const ID_HELP_GEN_INFO := 73

var _shell: DccShell
var _bridge: EngineBridge
var _host: Node                 ## Where dialogs are parented and callbacks live.
var _quality_popup: PopupMenu
var _recent_popup: PopupMenu
var _icon_families_popup: PopupMenu
var _texture_sets_popup: PopupMenu
## AS-13 / omission O2: `Assets ▸ Asset pack ▸`, `DCC_CONTROL_INDEX.md` §2.3.1.
var _asset_pack_popup: PopupMenu
var _ap_edit_popup: PopupMenu
var _ap_batch_popup: PopupMenu
var _ap_build_popup: PopupMenu
var _ap_stats_idx: int = -1
## §2.5 Performance ▸ the four multi-GPU rows (`GUI_GAP_REGISTER.md`
## PR-01/PR-02/PR-04/PR-05). `_gpu_devices` is cached rather than
## re-enumerated on every popup: enumeration opens a `wgpu` instance and walks
## every backend, which is far too much work for a menu that opens on hover.
## `Rescan devices…` is the explicit refresh, and is the honest place to put
## the cost.
var _gpu_devices_popup: PopupMenu
var _gpu_mode_popup: PopupMenu
var _gpu_vram_popup: PopupMenu
var _gpu_fallback_popup: PopupMenu
var _gpu_devices: Array = []
## Whether `gpu_devices()` has run this session. See `_on_gpu_devices_about_to_popup`.
var _gpu_enumerated := false
## The Preferences rows that must go dark while a generation owns the engine
## (`engine_bridge.gd`'s `gpu_settings_locked`), with the tooltips to put back
## afterwards. A submenu row carries no id, so it is tracked by index.
var _gpu_pref_rows: Array[int] = []
var _gpu_pref_tips: Array[String] = []
const GPU_DEV_AUTO := 0
const GPU_DEV_RESCAN := 1
const GPU_DEV_FIRST := 100      ## device i is GPU_DEV_FIRST + i
## §2.5's "VRAM budget · GB". A fixed ladder rather than a spinner: the value
## only ever gates whole grid sizes, so a free-form GB field would offer a
## precision the decision does not have.
const GPU_VRAM_CHOICES: Array[float] = [0.0, 1.0, 2.0, 4.0, 8.0, 12.0, 16.0, 24.0]
## The GPU-acceleration row's own tooltip, named because `about_to_popup` has
## to put it back after a generation released the row.
const GPU_TOGGLE_TIP := "Runs domain warp, crustal heterogeneity, plate assignment and flow accumulation on the GPU. A given seed produces a genuinely different (not just faster) world with this on vs. off -- both are valid, but they don't match each other. Takes effect on the next generate."

var _theme_popup: PopupMenu
var _theme_mode := "dark"  ## "dark" / "light" / "system" -- which of the three
	## radio rows shows checked. Not persisted (`DccSettings` carries no theme
	## key yet): §2.5's "follow system" is explicitly a one-shot resolve, not a
	## live subscription, so there is no ongoing mode to save beyond the plain
	## dark/light bit `DccTheme.is_dark()` already is.
var _workspace_popup: PopupMenu
var _windows_popup: PopupMenu
var _open_windows: Array = []  ## the live `AcceptDialog`s behind `_windows_popup`, index-parallel to its items

func build(shell: DccShell, bridge: EngineBridge, host: Node) -> void:
	_shell = shell
	_bridge = bridge
	_host = host
	_theme_mode = "dark" if DccTheme.is_dark() else "light"
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
	## §2.3's own table: "⧉ Asset library (⇧A)" / "⧉ Sprite sheet slicer (▦)"
	## -- ⧉ is the "opens a dedicated window" marker (§2, §12: "the ⧉ window
	## marker in menus"), not the phone app-bar's "panels" glyph (▤) these two
	## items were built with; the literal Unicode "⧉" is already how every
	## window this shell opens marks its own title (`asset_library_window.gd`'s
	## "⧉ ASSET LIBRARY", `data_manager_window.gd`'s "⧉ DATA MANAGER"), so this
	## matches that convention rather than reaching for `DccIcons`, whose own
	## `PATHS` has no path a `PopupMenu` text item could render anyway --
	## every glyph elsewhere in this menu is plain text, never `add_icon_item`.
	_live(p, "⧉ Asset library", ID_ASSET_LIBRARY, KEY_MASK_SHIFT | KEY_A)
	_live(p, "⧉ Sprite sheet slicer (▦)", ID_SLICER)
	p.add_separator()
	## `AssetDB::add_item`/`raster::decode_png` are real and bound now
	## (`as_import_item`) -- the window's own slot grid is where a target
	## slot gets focused, so this opens straight to it rather than
	## duplicating slot-selection at the menu level.
	_live(p, "Import image…", ID_IMPORT_IMAGE)
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
	_live(p, "Apply library to map", ID_APPLY_LIBRARY)
	_live(p, "Clear library…", ID_CLEAR_LIBRARY)
	p.add_separator()
	_build_asset_pack_submenu(p)
	p.id_pressed.connect(_on_assets)

## AS-13 / omission O2: the `Assets ▸ Asset pack ▸` submenu
## `DCC_CONTROL_INDEX.md` §2.3.1 describes (24 controls, "19 backed-unwired
## against 1 engine gap") -- most of it real once `asset_bridge.rs`'s
## session exists. Laid out in its own four groups (Active pack / Edit /
## Batch / Build) the same way §2.3.1's own table does.
##
## The Edit ▸ and Batch ▸ groups need a *selected slot* (Edit) or a
## *multi-selection* (Batch) neither of which a flat `PopupMenu` item has --
## both open the real window, where that context lives, rather than
## duplicating slot/selection state at the menu level (real navigation to a
## real control, not a `_todo()` gap). Build ▸'s four items and the
## top-level Pack metadata/Clear library actions below need no such context,
## so they call straight into the engine.
func _build_asset_pack_submenu(p: PopupMenu) -> void:
	_asset_pack_popup = PopupMenu.new()
	_asset_pack_popup.name = "AssetPack"
	_shell.style_popup(_asset_pack_popup)
	p.add_child(_asset_pack_popup)
	p.add_submenu_item("Asset pack ▸", "AssetPack")

	var ap := _asset_pack_popup
	## Active pack -- name/author/license/schema/filled-slots, live values
	## refreshed on every `about_to_popup` (the same pattern `_quality_popup`'s
	## own live-check row and `_refresh_recent_worlds()` already use).
	ap.add_item("Active pack")
	ap.set_item_disabled(ap.item_count - 1, true)
	_ap_stats_idx = ap.item_count
	ap.add_item("— loading —")
	ap.set_item_disabled(ap.item_count - 1, true)
	ap.add_item("Schema 2 · STORED zip (frozen timestamps, byte-reproducible)")
	ap.set_item_disabled(ap.item_count - 1, true)
	ap.add_separator()
	ap.add_item("Pack metadata… (name / author / license)", ID_AP_PACK_META)
	ap.add_separator()

	_ap_edit_popup = PopupMenu.new()
	_ap_edit_popup.name = "APEdit"
	_shell.style_popup(_ap_edit_popup)
	ap.add_child(_ap_edit_popup)
	ap.add_submenu_item("Edit", "APEdit")
	for row in [
		["Open library workspace", ID_ASSET_LIBRARY],
		["Import image into slot…", ID_IMPORT_IMAGE],
		["Sprite sheet slicer…", ID_SLICER],
		["Add variant to slot", ID_IMPORT_IMAGE],
		["Replace / delete slot art", ID_ASSET_LIBRARY],
		["Slot transform (scale · fit · reset)", -1],
		["Preview background", ID_ASSET_LIBRARY],
	]:
		var wid: int = row[1]
		if wid < 0:
			_todo(_ap_edit_popup, String(row[0]),
				"ItemTransform is real and shown in the inspector now, but no as_set_item_transform #[func] exists yet to write a new scale/pan back -- reading it is done, editing it is a smaller follow-on.")
		else:
			_ap_edit_popup.add_item(String(row[0]), wid)
			_ap_edit_popup.set_item_tooltip(_ap_edit_popup.item_count - 1,
				"Opens the Asset Library window -- every Edit control needs a focused slot, which only the window's own grid provides.")
	_ap_edit_popup.id_pressed.connect(_on_ap_edit)

	_ap_batch_popup = PopupMenu.new()
	_ap_batch_popup.name = "APBatch"
	_shell.style_popup(_ap_batch_popup)
	ap.add_child(_ap_batch_popup)
	ap.add_submenu_item("Batch", "APBatch")
	for label in ["Tag…", "Collect into set…", "Rename…", "Duplicate", "Delete ⌫"]:
		_ap_batch_popup.add_item(label, ID_ASSET_LIBRARY)
		_ap_batch_popup.set_item_tooltip(_ap_batch_popup.item_count - 1,
			"Opens the Asset Library window -- every batch op needs a multi-selection, which only the window's own grid (⇧-click ranges, ⌘/Ctrl-click adds) provides. All five are real there.")
	_ap_batch_popup.id_pressed.connect(func(_id: int): _host.open_asset_library())

	_ap_build_popup = PopupMenu.new()
	_ap_build_popup.name = "APBuild"
	_shell.style_popup(_ap_build_popup)
	ap.add_child(_ap_build_popup)
	ap.add_submenu_item("Build", "APBuild")
	_ap_build_popup.add_item("Validate pack (warning count)", ID_AP_VALIDATE)
	_ap_build_popup.add_item("Apply to map", ID_APPLY_LIBRARY)
	_ap_build_popup.add_item("Import pack .zip…", ID_IMPORT_PACK)
	_ap_build_popup.add_item("Export pack .zip… ⌘⇧P", ID_AP_EXPORT)
	_ap_build_popup.set_item_accelerator(_ap_build_popup.item_count - 1, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_P)
	_ap_build_popup.id_pressed.connect(_on_assets)

	ap.about_to_popup.connect(_refresh_asset_pack_stats)

func _refresh_asset_pack_stats() -> void:
	if _ap_stats_idx < 0:
		return
	var info: Dictionary = _bridge.as_pack_info()
	var total := int(info.get("total_items", 0))
	var pn := String(info.get("name", ""))
	_asset_pack_popup.set_item_text(_ap_stats_idx, "%s · %s · %s · %d item%s" % [
		pn if pn != "" else "(unnamed)",
		String(info.get("author", "")) if String(info.get("author", "")) != "" else "(no author)",
		String(info.get("license", "")) if String(info.get("license", "")) != "" else "(no license)",
		total, "" if total == 1 else "s"])

func _on_ap_edit(id: int) -> void:
	match id:
		ID_SLICER: _host.open_asset_library("", true)
		_: _host.open_asset_library()

func _on_assets(id: int) -> void:
	match id:
		ID_IMPORT_PACK: _host.open_asset_pack_picker()
		ID_IMPORT_IMAGE: _host.open_asset_library()
		ID_ASSET_LIBRARY: _host.open_asset_library()
		ID_SLICER: _host.open_asset_library("", true)
		ID_APPLY_LIBRARY: _host.asset_library_window.apply_to_map_now()
		ID_CLEAR_LIBRARY: _host.asset_library_window.clear_library_now()
		ID_AP_VALIDATE: _host.asset_library_window.validate_now()
		ID_AP_EXPORT: _host.asset_library_window.export_pack_now()
		ID_AP_PACK_META: _host.open_asset_library()
		ID_SLICER: _host.open_asset_library("", true)

# -- §2.4 Data ----------------------------------------------------------------
#
# §2.4: the dropdown mirrors the Data manager window's four groups and is a
# shortcut into it, never a second implementation. The window (`§9`,
# `data_manager_window.gd`) now exists -- each group item below opens it
# scoped to that group's first route. Most routes inside are still disclosed
# gaps (see that file's own header comment for the full breakdown); what
# changed here is that the item now opens a real window that is honest about
# which of its own routes work, rather than being disabled at the menu level.

## §2.4's own 2026-08-19 addition (`DCC_SHELL_SPEC.md`, reconciled from
## `JOURNEY_PLANNER_SPEC.md`): Journey planner sits above the five Data
## manager groups, alongside World data tables, and arms the INFRA JOURNEY
## tool takeover rather than opening a window. Travel library (⇧L,
## `TRAVEL_LIBRARY_SPEC.md`) is the sibling addition that DOES open its own
## window -- `2a`'s own mockup places it directly below Journey planner,
## in its own bracket, above the Data manager's four groups.
## **Conversion is deliberately absent** (owner decision, 2026-08-20). The
## row used to read "Conversion ▸ Coordinate systems · Formats" and open the
## Data manager on a three-route group of disclosed gaps. `GUI_GAP_REGISTER
## .md` §7.4's research found that no serious GIS or mapping application
## carries a top-level Conversion route -- reprojection and format handling
## belong to import/export, where the file is actually being read or
## written -- so the group was removed outright rather than left as three
## permanently-empty rows promising a shape the product will not take. See
## `DCC_SHELL_SPEC.md` §2.4's correction note.
func _data(p: PopupMenu) -> void:
	_live(p, "World data tables…", ID_DATA_MANAGER)
	_live(p, "Journey planner…", ID_JOURNEY_PLANNER, KEY_MASK_SHIFT | KEY_J)
	_live(p, "Travel library…", ID_TRAVEL_LIBRARY, KEY_MASK_SHIFT | KEY_L)
	p.add_separator()
	_live(p, "Import ▸ Maps · Heightmaps · GIS · World data", ID_DATA_MGR_IMPORT)
	_live(p, "Export ▸ Maps · GIS · World data · Asset pack", ID_DATA_MGR_EXPORT)
	_live(p, "Sources ▸ External · Connected · Registry", ID_DATA_MGR_SOURCES)
	_live(p, "Validation ▸ Check data · Repair", ID_DATA_MGR_VALIDATION)
	p.id_pressed.connect(func(id: int) -> void:
		match id:
			ID_DATA_MANAGER: _host.open_world_data()
			ID_JOURNEY_PLANNER: _host.open_journey_planner()
			ID_TRAVEL_LIBRARY: _host.open_travel_library()
			ID_DATA_MGR_IMPORT: _host.open_data_manager("Import")
			ID_DATA_MGR_EXPORT: _host.open_data_manager("Export")
			ID_DATA_MGR_SOURCES: _host.open_data_manager("Sources")
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
	_gpu_pref_rows.clear()
	_gpu_pref_tips.clear()
	p.add_check_item("GPU acceleration", ID_PREF_GPU)
	var gpu_idx := p.item_count - 1
	p.set_item_checked(gpu_idx, bool(_bridge.param_get("use_gpu")))
	p.set_item_tooltip(gpu_idx, GPU_TOGGLE_TIP)
	## Every row in this group reaches a `WorldGen` `#[func]`, and a generation
	## in flight holds that object mutably borrowed on its worker thread --
	## reaching it anyway is the `Gd<T>::bind() failed, already bound` failure
	## `engine_bridge.gd`'s multi-GPU block documents. The bridge refuses those
	## calls; this is where the refusal is made visible, because a control that
	## silently no-ops is exactly what this file's own honesty rule forbids.
	p.about_to_popup.connect(func():
		p.set_item_checked(gpu_idx, bool(_bridge.param_get("use_gpu")))
		var busy := _bridge.gpu_settings_locked()
		var why := "A generation is running. Every setting in this group takes effect on the next generate anyway, and the engine object belongs to the worker thread until this one finishes."
		p.set_item_disabled(gpu_idx, busy)
		p.set_item_tooltip(gpu_idx, why if busy else GPU_TOGGLE_TIP)
		for i in _gpu_pref_rows.size():
			var row: int = _gpu_pref_rows[i]
			if row < p.item_count:
				p.set_item_disabled(row, busy)
				p.set_item_tooltip(row, why if busy else _gpu_pref_tips[i]))
	## PR-01/PR-02/PR-04/PR-05: the four §2.5 Performance rows the engine now
	## backs. Each is a submenu rather than a dialog -- every one of them is a
	## small fixed choice, and a modal for four radio lists would be more
	## chrome than content.
	if _bridge.gpu_api:
		_build_gpu_devices_menu(p)
		_build_gpu_mode_menu(p)
	else:
		_todo(p, "Devices", "This GDExtension build predates the multi-GPU API (WorldGen.gpu_enumerate_devices is missing).")
		_todo(p, "Multi-GPU mode", "Same.")
	_todo(p, "CPU worker threads", "Rayon sizes its own pool; no override is exposed.")
	if _bridge.gpu_api:
		_build_gpu_vram_menu(p)
		_build_gpu_fallback_menu(p)
	else:
		_todo(p, "VRAM budget", "Same -- this build predates the multi-GPU API.")
		_todo(p, "Fallback when VRAM full", "Same.")
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
	## The one row `world_workspace.gd`'s "Not a generation stage" note points
	## at for chunk debug, so its tooltip has to actually name that -- it did
	## not, which made the pointer dangle (2026-08-20 menu-structure audit).
	_todo(p, "Tiled LOD · tile size · atlas cache · chunk debug",
		"Deep-zoom LOD tiling is live and automatic (lod_synthesize_tile/lod_tile_cells, driven by viewport_host.gd) -- what does not exist is any of §2.5's controls over it: no auto/manual switch (#lodAutoChk), no tile-size or LOD-level choice, no Refine detail for the current view (#lodRefineBtn), and no persistent atlas cache to bake into, cap or clear (#lodBakeBtn/#lodClearAtlasBtn -- tiles are synthesized on demand and never written to disk). The reference's two per-tile refinement passes, Burn rivers into tiles and Micro-erode tiles, have no cartalith-engine equivalent either: lod_synthesize_tile resamples the existing field and runs no erosion or river burn-in of its own. The chunk debug overlay (#lodDbgSeg grid / colors / off) and Show tile borders have no draw path -- viewport_host.gd composites LOD tiles into the map layer with no debug visualisation of the tile grid.")
	_todo(p, "Undo history", "No undo stack yet.")
	## §2.5's Memory group has three items -- Undo history (above, a real
	## gap), Working set and Clear caches -- but only the first ever made it
	## into this menu; the other two were missing outright, not even as
	## honest `_todo()`s, found in the 2026-08-19 GUI audit alongside the
	## orphaned `PerformanceWindow` this now opens. Working set is real:
	## `OS.get_static_memory_usage()`, the same source the menu bar's own
	## `top_mem` readout already uses (`app.gd`'s `_wire_status()`).
	_live(p, "Working set…", ID_PREF_WORKING_SET)
	_todo(p, "Clear caches…", "No atlas or field cache exists yet to clear (Preferences ▸ Tiled LOD is itself not built).")
	p.add_separator()

	## §2.5's Application group: "Storage locations… — Same modal as File."
	## Genuinely the same dialog `File ▸ Storage locations` opens --
	## `DccApp.open_storage_locations()` is the one method both call.
	_live(p, "Storage locations…", ID_PREF_STORAGE)

	## PR-13/PR-14: `DccTheme.LIGHT` was always fully defined -- §11's own
	## light token column -- the blocker was that `DccShell` built every
	## stylebox once at startup with no rebuild path. `DccShell.rebuild_theme()`
	## is that path now, so Light is live rather than disabled, and Follow
	## system (previously absent, §2.5's third choice) resolves the OS
	## preference once through it too.
	_theme_popup = PopupMenu.new()
	_theme_popup.name = "ThemeChoice"
	_theme_popup.add_radio_check_item("Dark", ID_PREF_THEME_DARK)
	_theme_popup.add_radio_check_item("Light", ID_PREF_THEME_LIGHT)
	_theme_popup.add_radio_check_item("Follow system", ID_PREF_THEME_SYSTEM)
	if not DisplayServer.is_dark_mode_supported():
		_theme_popup.set_item_disabled(2, true)
		_theme_popup.set_item_tooltip(2,
			"This platform/build reports no OS dark-mode preference (DisplayServer.is_dark_mode_supported() is false).")
	_refresh_theme_menu()
	_theme_popup.id_pressed.connect(_on_theme_choice)
	_shell.style_popup(_theme_popup)
	p.add_child(_theme_popup)
	p.add_submenu_item("Theme", "ThemeChoice")
	_todo(p, "Units", "The shell is km-only; the reference's mi toggle is not ported.")
	_todo(p, "Keyboard shortcuts…", "No shortcut table yet.")
	p.id_pressed.connect(_on_preferences.bind(p))

# -- §2.5 Performance ▸ multi-GPU ---------------------------------------------

## Record the row `add_submenu_item` just appended, and give it its tooltip, so
## `_preferences`' `about_to_popup` can darken it for the duration of a
## generation and put the real text back afterwards. A submenu row carries no
## id, so the index is the only handle there is.
func _track_gpu_pref_row(p: PopupMenu, tip: String) -> void:
	var i := p.item_count - 1
	p.set_item_tooltip(i, tip)
	_gpu_pref_rows.append(i)
	_gpu_pref_tips.append(tip)

## §2.5: "Expands to a per-device checklist with live utilisation ... Unchecking
## a device excludes it from dispatch."
##
## The checklist is real. The **utilisation percentage is not, and is not
## faked**: `wgpu` 30 reports no system-wide GPU utilisation and no VRAM size
## on any backend, so `GPU 0 · discrete 16 GB 71%` cannot be produced honestly.
## What each row shows instead is what is genuinely knowable -- class, backend
## and driver -- and the footer shows the one real memory number there is:
## this application's own allocation total from the last GPU generation.
func _build_gpu_devices_menu(p: PopupMenu) -> void:
	_gpu_devices_popup = PopupMenu.new()
	_gpu_devices_popup.name = "GpuDevices"
	_shell.style_popup(_gpu_devices_popup)
	_gpu_devices_popup.id_pressed.connect(_on_gpu_device_choice)
	_gpu_devices_popup.about_to_popup.connect(_on_gpu_devices_about_to_popup)
	p.add_child(_gpu_devices_popup)
	p.add_submenu_item("Devices", "GpuDevices")
	_track_gpu_pref_row(p, "Which physical GPU(s) the four GPU-eligible substrate stages dispatch to. Takes effect on the next generate.")
	## Deliberately NOT enumerated here. `build()` runs inside `DccApp._ready`,
	## and `gpu_devices()` stands up a `wgpu::Instance` and walks its backends
	## -- doing that while Godot's own GL Compatibility renderer is still
	## coming up corrupts its GLES3 resource caches and takes the process down
	## with a signal-11 inside `update_texture_atlas`, no GDScript frame in the
	## backtrace. Bisected against a real launch (AMD RX 7800 XT, OpenGL 3.3
	## Core): stubbing this one call out made the launch clean, and it is the
	## reason "launching from Godot's launcher stopped working" (owner,
	## 2026-08-20). Restricting `wgpu` to non-GL backends was tried first and
	## is *not* sufficient on its own -- kept anyway, in `multi.rs`, because a
	## second GL context in this process is wrong regardless.
	##
	## `about_to_popup` above already refreshes, so the list fills the first
	## time the submenu is actually opened -- by which point the renderer is
	## long since up. This also happens to be what this file's own
	## `_gpu_devices` doc comment asked for: enumeration is "far too much work
	## for a menu that opens on hover", and now nobody pays it until they ask.
	_refresh_gpu_devices_menu()

## The only place a *first* enumeration is allowed to happen: a real user
## opening the submenu, long after the renderer is up. Kept separate from
## `_refresh_gpu_devices_menu` because that one also runs at build time, where
## enumerating is exactly the crash `_build_gpu_devices_menu` documents.
## `_gpu_enumerated` rather than `_gpu_devices.is_empty()` so a machine that
## genuinely has no adapters does not re-enumerate on every hover.
## A generation in flight owns the engine object outright (see
## `engine_bridge.gd`'s `_gpu_read` note), so `gpu_devices()` would answer
## from an empty cache. Latching `_gpu_enumerated` on that answer is what
## left the submenu permanently reading "No GPU detected" for the rest of the
## session -- so the latch only closes over a reply that was really enumerated.
func _on_gpu_devices_about_to_popup() -> void:
	if not _gpu_enumerated and not _bridge.gpu_settings_locked():
		_gpu_enumerated = true
		_gpu_devices = _bridge.gpu_devices()
	_refresh_gpu_devices_menu()

func _refresh_gpu_devices_menu() -> void:
	var pm := _gpu_devices_popup
	pm.clear()
	var selected := _bridge.gpu_selected_devices()

	pm.add_check_item("Automatic (highest-performance GPU)", GPU_DEV_AUTO)
	pm.set_item_checked(pm.item_count - 1, selected.is_empty())
	pm.set_item_tooltip(pm.item_count - 1,
		"The default, and what this port always did: one PowerPreference::HighPerformance adapter. Check a device below to override it.")
	pm.add_separator()

	if _gpu_devices.is_empty():
		pm.add_item("No GPU detected")
		pm.set_item_disabled(pm.item_count - 1, true)
		pm.set_item_tooltip(pm.item_count - 1,
			"wgpu enumerated no adapters. Generation runs entirely on the CPU, which is the reference path and produces correct worlds -- just slower on the four GPU-eligible substrate stages.")
	for i in _gpu_devices.size():
		var d: Dictionary = _gpu_devices[i]
		var label := "%s · %s · %s" % [String(d.get("name", "?")), String(d.get("kind", "?")), String(d.get("backend", "?"))]
		pm.add_check_item(label, GPU_DEV_FIRST + i)
		var idx := pm.item_count - 1
		pm.set_item_checked(idx, String(d.get("key", "")) in selected)
		var alts: PackedStringArray = d.get("alt_backends", PackedStringArray())
		var alt_txt := (" Also reachable over %s." % ", ".join(alts)) if not alts.is_empty() else ""
		if bool(d.get("software", false)):
			pm.set_item_disabled(idx, true)
			pm.set_item_tooltip(idx,
				"A software rasterizer, not a GPU. Never dispatched to: it would be slower than the CPU path it is pretending to accelerate. Listed rather than hidden because it is genuinely what the system enumerates.")
		else:
			pm.set_item_tooltip(idx,
				"driver %s %s.%s Max buffer %d MB.%s" % [
					String(d.get("driver", "?")), String(d.get("driver_info", "")), alt_txt,
					int(d.get("max_buffer_mb", 0)),
					"" if bool(d.get("compute", true)) else " No compute-shader support -- cannot run this pipeline."])

	pm.add_separator()
	## The honest memory readout, in place of §2.5's un-obtainable percentage.
	var usage := _bridge.gpu_last_device_usage()
	if usage.is_empty():
		pm.add_item("Memory: not measured yet")
		pm.set_item_disabled(pm.item_count - 1, true)
		pm.set_item_tooltip(pm.item_count - 1,
			"Measured during a GPU generation, not polled. Generate once with GPU acceleration on and this shows real numbers.")
	else:
		for u in usage:
			pm.add_item("%s: %d MB allocated (%d MB reserved)" % [
				String(u.get("name", "?")), int(u.get("allocated_mb", 0)), int(u.get("reserved_mb", 0))])
			pm.set_item_disabled(pm.item_count - 1, true)
			pm.set_item_tooltip(pm.item_count - 1,
				"This application's own GPU memory at the end of the last GPU generation, from wgpu's allocator. Not system-wide VRAM use, and not a utilisation percentage -- neither is queryable through wgpu on any backend. Read the two together: every dispatch frees its buffers as it returns, so allocated falls back to the idle baseline while reserved is what the allocator still holds from the driver -- reserved is the number that answers how much of the card this app is occupying.")
	pm.add_separator()
	pm.add_item("Rescan devices…", GPU_DEV_RESCAN)

func _on_gpu_device_choice(id: int) -> void:
	if id == GPU_DEV_RESCAN:
		_gpu_devices = _bridge.gpu_devices()
		_refresh_gpu_devices_menu()
		return
	if id == GPU_DEV_AUTO:
		_bridge.gpu_set_selected_devices(PackedStringArray())
		_refresh_gpu_devices_menu()
		return
	var i := id - GPU_DEV_FIRST
	if i < 0 or i >= _gpu_devices.size():
		return
	var key := String((_gpu_devices[i] as Dictionary).get("key", ""))
	var selected := _bridge.gpu_selected_devices()
	var at := Array(selected).find(key)
	if at >= 0:
		selected.remove_at(at)
	else:
		selected.append(key)
	_bridge.gpu_set_selected_devices(selected)
	_refresh_gpu_devices_menu()

## §2.5: "`split tiles` (default) · `alternate frames` · `single device`".
##
## Two deliberate departures, both disclosed in the rows themselves:
## `alternate frames` is **disabled** (§2.5's own note is that it only helps
## the 3D viewport, and there is no 3D viewport), and the default here is
## `single device` rather than `split tiles`, because splitting was measured
## on real hardware and only pays above 2048² -- the tooltip carries the
## numbers rather than asserting a benefit.
func _build_gpu_mode_menu(p: PopupMenu) -> void:
	_gpu_mode_popup = PopupMenu.new()
	_gpu_mode_popup.name = "GpuMultiMode"
	_shell.style_popup(_gpu_mode_popup)
	_gpu_mode_popup.add_radio_check_item("Single device", 0)
	_gpu_mode_popup.set_item_tooltip(0,
		"Everything on the one selected (or automatic) device. The default.")
	_gpu_mode_popup.add_radio_check_item("Split tiles", 1)
	_gpu_mode_popup.set_item_tooltip(1,
		"Partitions the domain-warp stage into row bands across every selected device, sized by measured per-device throughput. Only that one stage: it is the only GPU stage here whose kernel reads nothing outside its own cell -- blur needs a halo, plate assignment and flow accumulation read across the whole grid. Measured on this machine (RX 7800 XT + integrated Radeon): 1.2-1.5x at 4096 squared, but 0.7-0.8x at 2048 squared and below, where the second device's fixed cost exceeds what it contributes. Needs two devices checked above.")
	_gpu_mode_popup.add_radio_check_item("Alternate frames", 2)
	_gpu_mode_popup.set_item_disabled(2, true)
	_gpu_mode_popup.set_item_tooltip(2,
		"Not built. The spec's own note is that alternate-frame rendering only helps the 3D viewport, and this port has no 3D viewport (DECISIONS.md section 4 defers 3D to Phase 3). Selecting it would be indistinguishable from Single device while implying otherwise.")
	_gpu_mode_popup.id_pressed.connect(_on_gpu_mode_choice)
	_gpu_mode_popup.about_to_popup.connect(_refresh_gpu_mode_menu)
	_shell.style_popup(_gpu_mode_popup)
	p.add_child(_gpu_mode_popup)
	p.add_submenu_item("Multi-GPU mode", "GpuMultiMode")
	_track_gpu_pref_row(p, "How work is divided when more than one device is selected. Takes effect on the next generate.")
	_refresh_gpu_mode_menu()

func _refresh_gpu_mode_menu() -> void:
	var mode := _bridge.gpu_multi_mode()
	_gpu_mode_popup.set_item_checked(0, mode == "single_device")
	_gpu_mode_popup.set_item_checked(1, mode == "split_tiles")
	_gpu_mode_popup.set_item_checked(2, mode == "alternate_frames")

func _on_gpu_mode_choice(id: int) -> void:
	var names := ["single_device", "split_tiles", "alternate_frames"]
	if id < 0 or id >= names.size():
		return
	if _bridge.gpu_set_multi_mode(names[id]):
		_refresh_gpu_mode_menu()

## §2.5: "VRAM budget — GB, default 75 % of the smallest active device."
##
## The cap is real; **that default is not implementable and is not faked**.
## `wgpu` 30 reports no VRAM size for an adapter at all, so there is no
## quantity to take 75 % of -- the default is "no cap", and the row says so.
func _build_gpu_vram_menu(p: PopupMenu) -> void:
	_gpu_vram_popup = PopupMenu.new()
	_gpu_vram_popup.name = "GpuVramBudget"
	_shell.style_popup(_gpu_vram_popup)
	for i in GPU_VRAM_CHOICES.size():
		var gb: float = GPU_VRAM_CHOICES[i]
		_gpu_vram_popup.add_radio_check_item("No cap" if gb <= 0.0 else "%d GB" % int(gb), i)
	_gpu_vram_popup.set_item_tooltip(0,
		"The default. The spec asks for 75 percent of the smallest active device, which cannot be computed: wgpu exposes no VRAM size for an adapter on any backend, so there is nothing to take a percentage of.")
	_gpu_vram_popup.id_pressed.connect(_on_gpu_vram_choice)
	_gpu_vram_popup.about_to_popup.connect(_refresh_gpu_vram_menu)
	p.add_child(_gpu_vram_popup)
	p.add_submenu_item("VRAM budget", "GpuVramBudget")
	_track_gpu_pref_row(p, "A cap on the GPU working set this pipeline may allocate for a grid. Takes effect on the next generate.")
	_refresh_gpu_vram_menu()

func _refresh_gpu_vram_menu() -> void:
	var cur := _bridge.gpu_vram_budget_gb()
	for i in GPU_VRAM_CHOICES.size():
		_gpu_vram_popup.set_item_checked(i, is_equal_approx(GPU_VRAM_CHOICES[i], cur))
	## The estimate is the whole reason the cap is meaningful, so it is shown
	## against the *current* grid rather than left for the user to guess.
	var est := _bridge.gpu_vram_estimate()
	var idx := _gpu_vram_popup.get_item_index(GPU_VRAM_CHOICES.size())
	if idx >= 0:
		_gpu_vram_popup.remove_item(idx)
	if not est.is_empty():
		## `gw`/`gh` are 0 until the first generate -- say that, rather than
		## printing "0x0 needs about 0 MB", which reads like a measurement.
		var label := "Current grid %dx%d needs about %d MB" % [
			int(est.get("gw", 0)), int(est.get("gh", 0)), int(est.get("estimate_mb", 0))]
		if int(est.get("gw", 0)) <= 0:
			label = "Estimate available after the first generate"
		_gpu_vram_popup.add_item(label, GPU_VRAM_CHOICES.size())
		var i2 := _gpu_vram_popup.item_count - 1
		_gpu_vram_popup.set_item_disabled(i2, true)
		_gpu_vram_popup.set_item_tooltip(i2,
			"An upper bound on what this pipeline's own GPU buffers need, not a measurement of card occupancy: ten f32 grids, the count the heaviest stage (plate assignment) binds plus its staging buffers. Over budget, the fallback below decides what happens.")

func _on_gpu_vram_choice(id: int) -> void:
	if id < 0 or id >= GPU_VRAM_CHOICES.size():
		return
	_bridge.gpu_set_vram_budget_gb(GPU_VRAM_CHOICES[id])
	_refresh_gpu_vram_menu()

## §2.5: "`CPU tile pass` (default) · `reduce working res` · `fail with error`".
## The first is already exactly what this port does whenever the GPU path is
## unavailable, so wiring it discloses existing behaviour; the third is real;
## the middle one has no implementation and is disabled rather than accepted.
func _build_gpu_fallback_menu(p: PopupMenu) -> void:
	_gpu_fallback_popup = PopupMenu.new()
	_gpu_fallback_popup.name = "GpuVramFallback"
	_shell.style_popup(_gpu_fallback_popup)
	_gpu_fallback_popup.add_radio_check_item("CPU tile pass", 0)
	_gpu_fallback_popup.set_item_tooltip(0,
		"The default, and already what happens on any GPU failure: the stage runs on the CPU instead. The CPU path is this port's reference implementation, so the world stays correct -- only slower.")
	_gpu_fallback_popup.add_radio_check_item("Reduce working res", 1)
	_gpu_fallback_popup.set_item_disabled(1, true)
	_gpu_fallback_popup.set_item_tooltip(1,
		"Not built. Nothing in this pipeline computes a stage at a reduced grid and resamples back up -- LOD tile synthesis resamples an already-finished field, which is a different operation. Left visible so the gap is stated rather than silently missing.")
	_gpu_fallback_popup.add_radio_check_item("Fail with error", 2)
	_gpu_fallback_popup.set_item_tooltip(2,
		"Refuse to generate rather than quietly dropping to the CPU. Useful when a benchmark run must be GPU or nothing.")
	_gpu_fallback_popup.id_pressed.connect(_on_gpu_fallback_choice)
	_gpu_fallback_popup.about_to_popup.connect(_refresh_gpu_fallback_menu)
	p.add_child(_gpu_fallback_popup)
	p.add_submenu_item("Fallback when VRAM full", "GpuVramFallback")
	_track_gpu_pref_row(p, "What happens when a grid is over the budget above. Takes effect on the next generate.")
	_refresh_gpu_fallback_menu()

func _refresh_gpu_fallback_menu() -> void:
	var cur := _bridge.gpu_vram_fallback()
	_gpu_fallback_popup.set_item_checked(0, cur == "cpu_tile_pass")
	_gpu_fallback_popup.set_item_checked(1, cur == "reduce_working_res")
	_gpu_fallback_popup.set_item_checked(2, cur == "fail_with_error")

func _on_gpu_fallback_choice(id: int) -> void:
	var names := ["cpu_tile_pass", "reduce_working_res", "fail_with_error"]
	if id < 0 or id >= names.size():
		return
	if _bridge.gpu_set_vram_fallback(names[id]):
		_refresh_gpu_fallback_menu()

func _on_preferences(id: int, p: PopupMenu) -> void:
	if id == ID_PREF_STORAGE:
		_host.open_storage_locations()
		return
	if id == ID_PREF_WORKING_SET:
		_host.open_performance()
		return
	if id != ID_PREF_GPU:
		return
	var idx := p.get_item_index(ID_PREF_GPU)
	var on := not bool(_bridge.param_get("use_gpu"))
	if _bridge.param_set("use_gpu", on):
		p.set_item_checked(idx, on)

func _refresh_theme_menu() -> void:
	_theme_popup.set_item_checked(0, _theme_mode == "dark")
	_theme_popup.set_item_checked(1, _theme_mode == "light")
	_theme_popup.set_item_checked(2, _theme_mode == "system")

## Dark/Light set the mode and the palette directly; Follow system resolves
## `DisplayServer`'s own preference once and applies it -- §2.5's "dark ·
## light · follow system" as three discrete choices, not a live subscription:
## nothing here re-checks it if the OS preference changes later, matching the
## owner's own "does not need to live-watch it".
func _on_theme_choice(id: int) -> void:
	var want_dark: bool
	match id:
		ID_PREF_THEME_DARK:
			_theme_mode = "dark"
			want_dark = true
		ID_PREF_THEME_LIGHT:
			_theme_mode = "light"
			want_dark = false
		ID_PREF_THEME_SYSTEM:
			_theme_mode = "system"
			want_dark = DisplayServer.is_dark_mode() if DisplayServer.is_dark_mode_supported() \
				else DccTheme.is_dark()
		_:
			return
	_refresh_theme_menu()
	if want_dark != DccTheme.is_dark():
		var was_dark := DccTheme.is_dark()
		DccTheme.apply_theme(want_dark)
		_shell.rebuild_theme(was_dark)

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

	## `PARITY_AUDIT.md` §5 item 5: the reference's `#resOverlay` (Shift+D) --
	## a top-right diagnostics HUD (`resource_overlay.gd`'s own header
	## comment explains why "Resource" is a misnomer). Off by default,
	## unlike the five region checks above -- this is a debug aid, not a
	## layout region a "Reset layout" click should ever have to re-show.
	p.add_check_item("Diagnostics overlay", ID_WIN_DIAG_OVERLAY)
	var diag_idx := p.item_count - 1
	p.set_item_accelerator(diag_idx, KEY_MASK_SHIFT | KEY_D)
	p.id_pressed.connect(func(id: int):
		if id == ID_WIN_DIAG_OVERLAY:
			_host.toggle_resource_overlay()
			p.set_item_checked(diag_idx, _host.resource_overlay.visible))
	p.add_separator()

	## WI-02: a real submenu over `DccShell.DOMAINS` -- the list itself is
	## fixed so it's built once, like `_quality_popup`'s own tier list;
	## `about_to_popup` only refreshes which row shows as the active domain,
	## the same split the GPU checkbox above already uses for "static rows,
	## live check".
	_workspace_popup = PopupMenu.new()
	_workspace_popup.name = "WorkspaceList"
	_shell.style_popup(_workspace_popup)
	p.add_child(_workspace_popup)
	p.add_submenu_item("Workspace", "WorkspaceList")
	for i in DccShell.DOMAINS.size():
		_workspace_popup.add_radio_check_item(String(DccShell.DOMAINS[i].label), i)
	_workspace_popup.about_to_popup.connect(func():
		for i in DccShell.DOMAINS.size():
			_workspace_popup.set_item_checked(i, DccShell.DOMAINS[i].id == _shell.active_domain()))
	_workspace_popup.id_pressed.connect(func(i: int):
		_shell.select_domain(String(DccShell.DOMAINS[i].id)))

	## WI-03: `DccApp`'s own `AcceptDialog`s, listed live while `visible` and
	## cleared+rebuilt on every `about_to_popup` -- the same rebuild-on-popup
	## convention `_refresh_recent_worlds()` already uses above, because
	## unlike the workspace list this one genuinely changes between opens.
	_windows_popup = PopupMenu.new()
	_windows_popup.name = "OpenWindows"
	_shell.style_popup(_windows_popup)
	p.add_child(_windows_popup)
	p.add_submenu_item("Open windows", "OpenWindows")
	_windows_popup.about_to_popup.connect(_refresh_open_windows)
	_windows_popup.id_pressed.connect(_on_open_window)

	p.add_separator()
	_live(p, "Reset layout", ID_WIN_RESET)
	_todo(p, "Save layout as…", "No layout store yet.")
	p.id_pressed.connect(func(id: int):
		_host.toggle_region(id)
		_sync_region_checks(p, id))

## The five region rows were checked once at build time and never again, so a
## toggle left the checkmark saying the opposite of the truth until the next
## restart. Not noticed on desktop, where the mark is a small tick a pointer
## user glances past; unmissable on the phone, where `phone_menu.gd` draws the
## same state as a full 40 dp switch that visibly refused to move. Fixed here
## rather than there, because the stale state is the popup's, not the
## presentation's -- the desktop menu was wrong too.
##
## `Reset layout` re-shows every region (`DccApp.toggle_region`'s own
## `ID_WIN_RESET` branch), so it re-checks all five rather than flipping one.
const WIN_REGION_IDS: Array[int] = [ID_WIN_LEFT, ID_WIN_RIGHT, ID_WIN_TIMELINE,
	ID_WIN_STATUS, ID_WIN_RAIL]

func _sync_region_checks(p: PopupMenu, id: int) -> void:
	if id == ID_WIN_RESET:
		for rid in WIN_REGION_IDS:
			var ri := p.get_item_index(rid)
			if ri >= 0:
				p.set_item_checked(ri, true)
		return
	if not WIN_REGION_IDS.has(id):
		return
	var i := p.get_item_index(id)
	if i >= 0:
		p.set_item_checked(i, not p.is_item_checked(i))

## `_host` is `DccApp` (`app.gd`); these five fields are its own public
## `AcceptDialog`s, none reached through any new API. Grepped against
## `app.gd` rather than trusted from memory -- the list has grown before.
func _refresh_open_windows() -> void:
	_windows_popup.clear()
	_open_windows.clear()
	for entry in [
		["New world…", _host.new_world_dialog],
		["World data tables", _host.world_data_window],
		["Performance", _host.performance_window],
		["Generation info", _host.gen_info_dialog],
		["Data manager", _host.data_manager_window],
		["Asset library", _host.asset_library_window],
		["Travel library", _host.travel_library_window],
	]:
		var dlg: Window = entry[1]
		if dlg != null and dlg.visible:
			_windows_popup.add_check_item(String(entry[0]), _open_windows.size())
			_windows_popup.set_item_checked(_open_windows.size(), true)
			_open_windows.append(dlg)
	if _open_windows.is_empty():
		_windows_popup.add_item("No windows open")
		_windows_popup.set_item_disabled(0, true)

## Brings an already-open window to front. `popup_centered()` on a `Window`
## that's already visible re-centres and raises it -- there is no separate
## "just raise, don't move" call on `AcceptDialog`.
func _on_open_window(id: int) -> void:
	if id >= 0 and id < _open_windows.size():
		(_open_windows[id] as Window).popup_centered()

# -- §2.7 Help ----------------------------------------------------------------

func _help(p: PopupMenu) -> void:
	_todo(p, "Documentation", "No in-app documentation yet; the repository docs are the reference.")
	_todo(p, "Keyboard shortcuts", "No shortcut table yet.")
	_live(p, "Credits & academic principles", ID_HELP_CREDITS)
	## `PARITY_AUDIT.md` §5 item 6: the reference's ℹ️ `#genInfoBtn` --
	## dumps every generation parameter as plain text, a bug-report
	## affordance distinct from "Report an issue" below (which still has no
	## actual issue-filing route).
	_live(p, "Generation info…", ID_HELP_GEN_INFO)
	_todo(p, "Report an issue", "No issue route wired.")
	_live(p, "About", ID_HELP_ABOUT)
	p.id_pressed.connect(_on_help)

func _on_help(id: int) -> void:
	match id:
		ID_HELP_CREDITS: _host.open_credits()
		ID_HELP_GEN_INFO: _host.open_gen_info()
		ID_HELP_ABOUT: _host.open_about()

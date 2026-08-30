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
## `14` was the one free id in §2.1's block; Autosave is a checkbox item, not
## an action, so it toggles `DccSettings.autosave_enabled()` and re-arms the
## shell's clock rather than doing anything itself.
const ID_AUTOSAVE := 14
const ID_REVERT := 15
const ID_CLOSE := 16
const ID_STORAGE := 17
const ID_SHOW_ON_DISK := 19
## `File ▸ Autosave interval` (§2.1: "Toggle + interval submenu (off, 1, 5,
## 15 min). Default 5 min."). `Off` writes `DccSettings.set_autosave_enabled
## (false)` -- the same bit the `Autosave` check item above it flips, one
## setting with two entry points, exactly as `Storage locations…` is reached
## from both File and Preferences. The three minute values are §2.1's own.
const ID_AUTOSAVE_OFF := 130
const ID_AUTOSAVE_FIRST := 131      ## interval i is ID_AUTOSAVE_FIRST + i
const AUTOSAVE_MINUTES: Array[int] = [1, 5, 15]

const ID_UNDO := 20
const ID_REDO := 21
## `GUI_GAP_REGISTER.md` **ED-02**. Its own id and not `ID_UNDO`'s: the panel
## and the action are different things, and routing both through one id is the
## shape this shell keeps having to undo.
const ID_DELETE := 77
const ID_DESELECT := 82
const ID_UNDO_HISTORY := 121
const ID_PREF_UNDO_CLEAR := 22
## `Edit ▸ Find on map…`. Was a `_todo` row (no id, since `_todo` never
## assigns one) claiming "the entity index and its pan-to-hit are both still
## owed" -- `PlaceSearch` (shell/place_search.gd) is that index, built now.
const ID_FIND_ON_MAP := 23

## `Preferences ▸ Memory ▸ Undo history` (PR-11). Budgets, not step counts --
## see the submenu's own comment in `_preferences` for why. 256 MB is
## `undo::DEFAULT_BUDGET_BYTES`; the engine floors any budget at 4 MB.
const UNDO_BUDGETS_MB: Array[int] = [64, 128, 256, 512, 1024]

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
## 42/43/44/46 held one id per Data-manager *group* while the dropdown drew one
## row per group. It now draws one row per route (see `_data()`), so the group
## ids are gone and `ID_DATA_ROUTE_FIRST` below replaces them. Left out rather
## than left dangling: an unused id in a menu file is how a row ends up wired
## to the wrong thing later.
const ID_TRAVEL_LIBRARY := 48
const ID_VAULT := 45
const ID_WORLD_DATA := 47
## One id per Data-manager route, allocated above every other id in this file.
## The Data dropdown draws all fourteen routes the canvas draws (see `_data()`),
## and each one is its own destination rather than a group's first.
const ID_DATA_ROUTE_FIRST := 400
var _data_route_ids: Array[String] = []

const ID_PREF_GPU := 50
const ID_PREF_THEME_DARK := 51
const ID_PREF_THEME_LIGHT := 52
const ID_PREF_QUALITY := 53
const ID_PREF_UNITS_KM := 54
const ID_PREF_UNITS_MI := 55
const ID_PREF_STORAGE := 56
const ID_PREF_WORKING_SET := 57
const ID_PREF_THEME_SYSTEM := 58
const ID_PREF_CLEAR_CACHES := 59

const ID_WIN_LEFT := 60
const ID_WIN_RIGHT := 61
const ID_WIN_TIMELINE := 62
const ID_WIN_STATUS := 63
const ID_WIN_RAIL := 64
const ID_WIN_RESET := 65
const ID_WIN_DIAG_OVERLAY := 66

## `Preferences ▸ Tiles & LOD ▸ Tile size` -- §2.5's "256/512/1024".
const ID_LOD_TILE_FIRST := 140      ## size i is ID_LOD_TILE_FIRST + i
const ATLAS_TILE_SIZES: Array[int] = [256, 512, 1024]
## `Preferences ▸ Tiles & LOD ▸ Tile size · LOD levels ▸ LOD levels` --
## §2.5's "levels 0-8". Depth d is ID_LOD_LEVEL_FIRST + d.
const ID_LOD_LEVEL_FIRST := 150   ## depths 0..8 occupy 150-158
const ID_LOD_MODE_AUTO := 160
const ID_LOD_MODE_MANUAL := 161
const ID_LOD_ENTER_NOW := 162
const ID_LOD_LEAVE_NOW := 163
const ID_LOD_CLEAR_ATLAS := 149

const ID_HELP_DOCS := 78
const ID_HELP_CREDITS := 70
const ID_HELP_ABOUT := 71
const ID_HELP_SHORTCUTS := 72
const ID_HELP_GEN_INFO := 73
## `Preferences ▸ Tiles & LOD ▸ Chunk debug overlay` -- the reference's
## `lodDbgSeg` trio (line 1266). Named `ID_HELP_*` while the submenu lived
## under Help; renamed with it when §2.5's own placement was restored.
const ID_LOD_DBG_GRID := 74
const ID_LOD_DBG_COLORS := 75
const ID_LOD_DBG_LABELS := 76
## 80/81, not 78/79: `ID_HELP_DOCS` already holds 78. Godot dispatches
## `id_pressed` per PopupMenu, so two popups sharing an id do not cross-fire --
## but `get_item_index(id)` does not know that, and a probe or a later reader
## looking a row up by number would find the wrong menu's. Cheap to avoid.
const ID_LOD_TILE_BORDERS := 80
const ID_LOD_REFINE_VIEW := 81

var _shell: DccShell
var _bridge: EngineBridge
var _host: Node                 ## Where dialogs are parented and callbacks live.
var _quality_popup: PopupMenu
var _recent_popup: PopupMenu
var _autosave_popup: PopupMenu
var _icon_families_popup: PopupMenu
var _texture_sets_popup: PopupMenu
## Family keys, index-parallel to the two submenus above. Members rather than
## locals because §2.3's "filled/capacity counts" have to be re-read on every
## popup -- an import changes them, and the submenus are built once.
var _icon_family_keys: Array[String] = []
var _texture_family_keys: Array[String] = []
## AS-13 / omission O2: `Assets ▸ Asset pack ▸`, `DCC_CONTROL_INDEX.md` §2.3.1.
var _asset_pack_popup: PopupMenu
## The `Edit ▸`/`Batch ▸`/`Build ▸` child popups these three held are gone
## (2026-08-25): the canvas draws one panel with four labelled bands, so
## `_build_asset_pack_submenu()` builds bands rather than submenus.
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

var _lod_debug_popup: PopupMenu   ## `Help ▸ LOD debug` -- see
	## `_build_lod_debug_submenu()`. Holds no state of its own; the check
	## marks are read back off `ViewportHost` each time it opens.
var _theme_popup: PopupMenu
var _theme_mode := "dark"  ## "dark" / "light" / "system" -- which of the three
	## radio rows shows checked. Not persisted (`DccSettings` carries no theme
	## key yet): §2.5's "follow system" is explicitly a one-shot resolve, not a
	## live subscription, so there is no ongoing mode to save beyond the plain
	## dark/light bit `DccTheme.is_dark()` already is.
## §2.5 Tiles & LOD. `_atlas_popup`'s first row is a live status readout, so
## its index is kept the way `_ap_stats_idx` keeps the Asset pack one.
var _tile_size_popup: PopupMenu
var _lod_levels_popup: PopupMenu
var _tiled_lod_popup: PopupMenu
var _atlas_popup: PopupMenu
var _atlas_stats_idx: int = -1
## §2.5 Memory: "Working set — read-only, `1.6 GB of 12 GB`." A disabled row
## refreshed in `about_to_popup`, tracked by index because it carries no id.
var _working_set_row := -1
var _undo_budget_popup: PopupMenu
var _undo_pref_row := -1   ## `Preferences ▸ Memory ▸ Undo history`'s own row; a
	## submenu row carries no id, so its index is the only handle there is --
	## the same reason `_track_gpu_pref_row` below keeps indices.
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
	## All five were `_todo` rows reading "requires a save writer" until
	## `cartalith-io` grew one (`GUI_GAP_REGISTER.md` FI-01). They stay
	## honest by a different mechanism now: enabled only against a build of
	## the extension that actually has `save_project`, and only while there
	## is a world to save, both refreshed in `about_to_popup` below.
	_live(p, "Save project", ID_SAVE, KEY_MASK_CTRL | KEY_S)
	var save_idx := p.item_count - 1
	_live(p, "Save as…", ID_SAVE_AS, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_S)
	var save_as_idx := p.item_count - 1
	p.add_check_item("Autosave", ID_AUTOSAVE)
	var autosave_idx := p.item_count - 1
	## Its tooltip carries the live interval and is therefore set in
	## `about_to_popup` below, not here: the submenu directly underneath can
	## change that number while this same menu is open.
	##
	## **§2.1's interval submenu**, which this menu shipped without: the row was
	## a bare check item and the interval was read-only, taken from
	## `DccSettings.autosave_minutes()` for the tooltip and settable nowhere.
	## `set_autosave_minutes()` was already there; nothing called it.
	##
	## §2.1's four values verbatim -- off, 1, 5, 15 min -- as radio rows, `Off`
	## writing the same `autosave_enabled` bit the check item above flips. Two
	## entry points onto one setting, the shape `Storage locations…` already has
	## in File and Preferences, rather than two settings that can disagree.
	##
	## §2.1's "Default 5 min" is the spec's; this install's store defaults to 10
	## (`dcc_settings.gd`), which is not on the ladder -- a stored value that is
	## not one of the four leaves every row unchecked and is reported on the
	## parent row instead of being silently rounded into one of them.
	_autosave_popup = PopupMenu.new()
	_autosave_popup.name = "AutosaveInterval"
	_shell.style_popup(_autosave_popup)
	_autosave_popup.add_radio_check_item("Off", ID_AUTOSAVE_OFF)
	for i in AUTOSAVE_MINUTES.size():
		_autosave_popup.add_radio_check_item("Every %d min" % AUTOSAVE_MINUTES[i],
			ID_AUTOSAVE_FIRST + i)
	_autosave_popup.id_pressed.connect(_on_autosave_interval)
	_autosave_popup.about_to_popup.connect(_refresh_autosave_menu)
	p.add_child(_autosave_popup)
	p.add_submenu_item("Autosave interval", "AutosaveInterval")
	var autosave_int_idx := p.item_count - 1
	_refresh_autosave_menu()
	_live(p, "Revert to last save", ID_REVERT)
	var revert_idx := p.item_count - 1

	## **The canvas's own band, and its own order** (`DCC Cartography style
	## 1920`, the one artboard that draws File open). Storage sits *between*
	## Revert and Close, under a `STORAGE LOCATIONS` label, with the four roots
	## listed read-only above the two actions -- and `Close project ⌘W` is the
	## last item in the menu, not the middle one. The shell had Close directly
	## after Revert and the storage pair below it, which is the canvas's order
	## inverted.
	p.add_separator("STORAGE LOCATIONS")
	## `padding:2px 14px 8px;font:10px 'IBM Plex Mono';color:#6f7478` with the
	## path in `#8d9296`: four read-only rows, not a control. Rebuilt on every
	## popup because Change locations… can move any of them mid-session.
	var root_rows: Array[int] = []
	for key in DccSettings.ROOT_KEYS:
		p.add_item("")
		p.set_item_disabled(p.item_count - 1, true)
		root_rows.append(p.item_count - 1)
	## One item, one dialog with an inline Browse… per root (`DccApp.
	## open_storage_locations()`) -- was two items (a read-only list plus a
	## separate "Change locations…" item) opening two dialogs that showed the
	## same four rows; merged on owner feedback (2026-08-19) as redundant
	## menu surface, not two distinct capabilities. The canvas's own label for
	## the surviving one is `Change locations…`, which is what it does.
	_live(p, "Change locations…", ID_STORAGE)
	_live(p, "Show project on disk", ID_SHOW_ON_DISK)
	var show_idx := p.item_count - 1
	p.set_item_tooltip(show_idx,
		"Reveals the project's folder in the OS file manager. Disabled until a project has been opened this session.")

	p.add_separator()
	_live(p, "Close project", ID_CLOSE, KEY_MASK_CTRL | KEY_W)
	var close_idx := p.item_count - 1
	## §2.1's static note: imports do not live in File. The canvas sets it in
	## `9.5px 'IBM Plex Mono';color:#5f6468;line-height:1.6` and lets it **wrap
	## over two lines**; a `PopupMenu` item cannot wrap, and a `PopupMenu` sizes
	## itself to its widest item, so as one row this sentence alone made the
	## File menu 380 px wide against the canvas's 298. Two disabled rows are the
	## canvas's own two lines, at the canvas's own width.
	for line in ["imports live under Data ▸ Import",
			"asset packs under Assets"]:
		p.add_item(line)
		p.set_item_disabled(p.item_count - 1, true)

	p.about_to_popup.connect(func():
		_refresh_recent_worlds()
		for ri in root_rows.size():
			var key := String(DccSettings.ROOT_KEYS[ri])
			var full := DccSettings.storage_root(key)
			p.set_item_text(root_rows[ri], "%s   %s" % [
				String(DccSettings.ROOT_LABELS[key]).to_lower(), _tail(full)])
			p.set_item_tooltip(root_rows[ri], full)
		p.set_item_disabled(show_idx, _host.current_project_path == "")
		var can_write: bool = _bridge.save_api
		var has_world: bool = _bridge.has_world
		for idx in [save_idx, save_as_idx]:
			p.set_item_disabled(idx, not (can_write and has_world))
			if not can_write:
				p.set_item_tooltip(idx, "This build of the engine has no save writer.")
			elif not has_world:
				p.set_item_tooltip(idx, "No world to save yet.")
			else:
				p.set_item_tooltip(idx, "")
		p.set_item_disabled(autosave_idx, not can_write)
		p.set_item_checked(autosave_idx, DccSettings.autosave_enabled())
		p.set_item_tooltip(autosave_idx,
			"Writes a backup beside the project (world.zip -> world.autosave.zip) every %d minutes while it has unsaved changes. Never overwrites the project itself."
				% DccSettings.autosave_minutes())
		p.set_item_disabled(autosave_int_idx, not can_write)
		p.set_item_text(autosave_int_idx, "Autosave interval   %s" % _autosave_summary())
		p.set_item_tooltip(autosave_int_idx,
			"How often the backup above is written. SS2.1 offers off / 1 / 5 / 15 min and names 5 as the default; this install's stored value is %d min."
				% DccSettings.autosave_minutes())
		## Revert reloads the file on disk, so it needs one -- a world that
		## has never been saved has nothing to revert *to*.
		p.set_item_disabled(revert_idx, _host.current_project_path == "")
		p.set_item_tooltip(revert_idx,
			"" if _host.current_project_path != "" else "This world has never been saved.")
		p.set_item_disabled(close_idx, not has_world))
	p.id_pressed.connect(_on_file)

## The canvas prints a storage root as `~/Cartalith/Worlds` -- three segments,
## because the mockup's roots live under a home directory. This port's real
## roots are `OS.get_user_data_dir()`-relative (`dcc_settings.gd`'s own header
## says why), which on Windows is six segments and ~70 characters, and a
## `PopupMenu` sizes itself to its widest item -- four of those would have made
## the File menu wider than the left dock. Last two segments, elided, with the
## whole path on the tooltip.
static func _tail(path: String) -> String:
	var norm := path.replace("\\", "/")
	var base := OS.get_user_data_dir().replace("\\", "/")
	if base != "" and norm.begins_with(base):
		return "…" + norm.substr(base.length())
	var parts := norm.split("/", false)
	if parts.size() <= 2:
		return path
	return "…/%s/%s" % [parts[parts.size() - 2], parts[parts.size() - 1]]

## What the `Autosave interval` row prints beside its own label -- the same
## live-state-in-the-label pattern `Edit ▸ Undo` uses. Says `off` when the
## toggle is off, because an interval that is not running is not a truth worth
## printing on its own.
func _autosave_summary() -> String:
	if not DccSettings.autosave_enabled():
		return "off"
	return "every %d min" % DccSettings.autosave_minutes()

func _refresh_autosave_menu() -> void:
	var on := DccSettings.autosave_enabled()
	var mins := DccSettings.autosave_minutes()
	_autosave_popup.set_item_checked(0, not on)
	for i in AUTOSAVE_MINUTES.size():
		_autosave_popup.set_item_checked(i + 1, on and AUTOSAVE_MINUTES[i] == mins)

func _on_autosave_interval(id: int) -> void:
	if id == ID_AUTOSAVE_OFF:
		DccSettings.set_autosave_enabled(false)
	else:
		var i := id - ID_AUTOSAVE_FIRST
		if i < 0 or i >= AUTOSAVE_MINUTES.size():
			return
		DccSettings.set_autosave_minutes(AUTOSAVE_MINUTES[i])
		DccSettings.set_autosave_enabled(true)
	## The clock is re-armed here, not on the next popup: `apply_autosave_setting()`
	## is what File ▸ Autosave already calls, and a changed interval that only
	## takes effect after the next toggle is the silently-inert shape this file
	## forbids.
	_host.apply_autosave_setting()
	_refresh_autosave_menu()

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
		ID_SAVE: _host.save_project()
		ID_SAVE_AS: _host.save_project_as()
		ID_AUTOSAVE:
			DccSettings.set_autosave_enabled(not DccSettings.autosave_enabled())
			_host.apply_autosave_setting()
		ID_REVERT: _host.revert_to_saved()
		ID_CLOSE: _host.close_project()
		ID_STORAGE: _host.open_storage_locations()
		ID_SHOW_ON_DISK: _host.show_project_on_disk()

# -- §2.2 Edit ----------------------------------------------------------------

func _edit(p: PopupMenu) -> void:
	## Global heightmap undo -- the reference's `undoBtn`/`undoLast`, register
	## ED-01. Covers the two destructive height operations this port has
	## bound (Sculpt ▸ Commit to map, and Carve fjords), which is the same
	## set the reference's own `pushUndo()` guards minus the eight erosion
	## passes it has and this port does not yet run.
	##
	## Not the same control as the right dock's Sculpt ▸ Undo, which steps
	## back through an *uncommitted* draft's stamps. The reference draws the
	## identical line and routes Ctrl+Z to whichever is in context; here the
	## accelerator stays on this one, since the draft's own Undo sits two
	## clicks away in the dock beside the stack it edits.
	_live(p, "Undo", ID_UNDO, KEY_MASK_CTRL | KEY_Z)
	var undo_idx := p.item_count - 1
	_todo(p, "Redo",
		"Global undo has no redo, in this port or the reference: undoLast() pops the snapshot " +
		"rather than moving a cursor through a history, so an undone step is gone. " +
		"The Sculpt draft's own stamp history (right dock, while the Sculpt tool is active) " +
		"does have a real Redo.")
	## `GUI_GAP_REGISTER.md` **ED-02**, built 2026-08-25 as the *ledger* §7.1
	## asked for rather than the five-row list a previous pass declined to
	## ship. It records every commit and reverses the ones it can, saying per
	## row which is which.
	_live(p, "Undo history…", ID_UNDO_HISTORY)
	p.set_item_tooltip(p.item_count - 1,
		"Opens the history ledger in the right dock: every committed operation this "
		+ "session, newest first, with the open Sculpt draft above them as its own "
		+ "tier. A row marked ▲ still has a height snapshot and can be reverted to; "
		+ "one marked · happened and cannot be walked back, and says exactly what is "
		+ "not retained; ◼ is a generate or a load, where history starts. Reverting "
		+ "is linear -- it discards everything after the row -- and asks first when "
		+ "it would discard more than one step.")
	p.add_separator()
	## **These six rows used to say "Nothing is selectable for editing yet
	## beyond settlements, which are read-only." Both halves of that were
	## false**, and had been since well before 2026-08-30 when a Nortantis
	## comparison happened to read them next to the bindings.
	##
	## Three independent selections exist and this shell calls all three:
	## icons (`icon_get_selected` / `icon_hit_test` / `icon_delete`, reached
	## from `cartography_workspace.gd`), labels (`label_select` /
	## `label_get_selected` / `label_delete`, same file), and settlements --
	## which are not read-only either: `app.gd`'s `KEY_DELETE` branch has
	## routed to `civilization_workspace.on_delete_key()` →
	## `place_editor_window.confirm_delete()` the whole time.
	##
	## So Delete already worked from the keyboard while the menu row beside it
	## said deletion could not exist. That is `PARITY_AUDIT.md` §23's defect
	## class with the polarity reversed -- the *disclosure* was the stale
	## artefact, not the wiring -- which is exactly why `audit_wiring.py`
	## cannot see it: every one of those `#[func]`s scores as reached.
	##
	## Delete is now live and goes through the same `app.delete_selection()`
	## the key does. The other five keep a `_todo`, with the reason replaced by
	## one that is true: what is missing is not selectability, it is a
	## clipboard model and a multi-selection -- `PARITY_AUDIT.md` §20 classes
	## ED-03/ED-04 as large for exactly that reason.
	_todo(p, "Cut",
		"No clipboard model exists. Icons, labels and settlements are three unrelated " +
		"single-item selections with no common representation to cut into one buffer " +
		"(PARITY_AUDIT.md §20, ED-03/ED-04).")
	_todo(p, "Copy", "Same -- no clipboard model.")
	_todo(p, "Paste", "Same -- nothing can be on a clipboard to paste.")
	## §2.2 prints this row's shortcut as `⌫`. That glyph is the Mac name for
	## the key `app.gd` already binds -- `KEY_DELETE`, routed to
	## `delete_selection()` through every workspace's `on_delete_key()` -- so the
	## accelerator names the real key rather than the spec's glyph. The row had
	## none at all, which made it the one live Edit item whose shortcut column
	## was blank while the key worked.
	##
	## Safe to put on the menu even though `app.gd` also handles it: Godot
	## delivers `_gui_input` to a focused `Control` **before** `_shortcut_input`
	## runs the popup accelerators, so a `LineEdit` still eats its own Delete.
	_live(p, "Delete", ID_DELETE, KEY_DELETE)
	p.set_item_tooltip(p.item_count - 1,
		"Delete the current selection. The Delete key does the same thing; this row " +
		"exists because a keyboard-only capability is not a discoverable one.")
	p.add_separator()
	_todo(p, "Select all",
		"Every selection in the shell holds exactly one item -- icon_get_selected and " +
		"label_get_selected each return a single index -- so there is no multi-selection " +
		"for this to select into.")
	## §2.2's other half, live since 2026-08-30. Its reason used to be "no
	## shared way to clear them", which was true of all three selections at
	## once: settlements had no `on_deselect`, and icons had no engine call to
	## clear with (labels always did -- `label_select(-1)`). `DccApp.
	## clear_selection()` is that shared way and `icon_deselect()` is the
	## binding it needed.
	##
	## **Select all above stays disabled, and the two are not the same job.**
	## Every selection here holds exactly one item, so there is nothing for
	## Select all to select INTO -- that needs a multi-selection model first.
	## Clearing one item needs no such model.
	_live(p, "Deselect", ID_DESELECT, KEY_MASK_CTRL | KEY_D)
	p.set_item_tooltip(p.item_count - 1,
		"Clears the settlement, label or icon selection in the active domain. Not the same as Escape, which puts the tool down and deliberately leaves the selection alone.")
	p.add_separator()
	## §2.2: "Search places, labels, factions, routes; result pans the
	## viewport." Both halves now exist: `PlaceSearch` (shell/place_search.gd)
	## builds the entity index off `EngineBridge.settlements()` /
	## `get_factions()` / `label_list()` / `roads()` / `sea_routes()`, and
	## `ViewportHost.move_view_to()` was always the pan half. This row only
	## opens the picker -- `_host.open_find_on_map()`, guarded the way every
	## `_host` call this file did not itself add already is, since `_host`
	## (`dcc_shell.gd`) is owned by a different pass and may not carry the
	## method yet.
	_live(p, "Find on map…", ID_FIND_ON_MAP, KEY_MASK_CTRL | KEY_F)

	## The row's label carries the operation name and the live cost, the way
	## the reference's own header pairs "↩ Undo" with `#undoMem`'s
	## "N steps saved · M MB". Rebuilt on every popup because both change
	## with every commit -- there is no signal to subscribe to.
	p.about_to_popup.connect(func():
		var stats: Dictionary = _bridge.undo_stats()
		var can: bool = _bridge.can_undo()
		p.set_item_disabled(undo_idx, not can)
		if can:
			p.set_item_text(undo_idx, "Undo %s" % _bridge.undo_label())
			p.set_item_tooltip(undo_idx, "%s · %s held" % [
				_undo_depth_text(stats), _mb(int(stats.get("bytes", 0)))])
		else:
			p.set_item_text(undo_idx, "Undo")
			p.set_item_tooltip(undo_idx, _undo_empty_reason()))
	p.id_pressed.connect(_on_edit)

## Why there is nothing to undo -- three genuinely different situations, and
## saying "nothing to undo" for all three would hide the interesting one (a
## generate cleared the stack, which is a deliberate divergence from the
## reference, not a missing feature).
func _undo_empty_reason() -> String:
	if not _bridge.has_world:
		return "Nothing to undo: no world yet."
	return ("Nothing to undo. The stack holds up to 5 snapshots of the height field, pushed by " +
		"Sculpt ▸ Commit to map and by Carve fjords, and is cleared by every Generate " +
		"(a snapshot of the previous world is the wrong content, and at a different " +
		"resolution the wrong length). Cost and depth: Preferences ▸ Memory ▸ Undo history.")

func _undo_depth_text(stats: Dictionary) -> String:
	var d := int(stats.get("depth", 0))
	return "%d step%s saved" % [d, "" if d == 1 else "s"]

## Bytes as the coarse MB the reference's own `#undoMem` prints (`toFixed(0)`),
## with a sub-MB floor so a small world does not read as "0 MB".
func _mb(bytes: int) -> String:
	if bytes <= 0:
		return "0 MB"
	if bytes < 1048576:
		return "<1 MB"
	return "%d MB" % int(round(float(bytes) / 1048576.0))

func _on_edit(id: int) -> void:
	match id:
		ID_UNDO: _host.undo_last()
		ID_UNDO_HISTORY: _host.open_undo_history()
		## Same path the Delete key takes -- see the Edit block's own comment
		## on why this row was a `_todo` claiming deletion was impossible while
		## the key it duplicates already worked.
		ID_DELETE: _host.delete_selection()
		ID_DESELECT: _host.clear_selection()
		## `_host` is `dcc_shell.gd`, owned by a different pass -- guarded
		## rather than called bare so a build that has not yet grown
		## `open_find_on_map()` still opens every other Edit row cleanly
		## instead of crashing the whole popup's `id_pressed` dispatch.
		ID_FIND_ON_MAP:
			if _host.has_method("open_find_on_map"):
				_host.open_find_on_map()

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
	##
	## The canvas runs these four as **one unbroken block** -- there is no rule
	## between the slicer and Import image in `DCC shell 1920` -- and puts its
	## only two rules after Texture sets and before Apply. The shell had three
	## rules here and `Asset pack ▸` alone at the very foot, below Clear
	## library…, where the canvas has it directly above Icon families.
	_live(p, "⧉ Asset library", ID_ASSET_LIBRARY, KEY_MASK_SHIFT | KEY_A)
	_live(p, "⧉ Sprite sheet slicer (▦)", ID_SLICER)
	## `AssetDB::add_item`/`raster::decode_png` are real and bound now
	## (`as_import_item`) -- the window's own slot grid is where a target
	## slot gets focused, so this opens straight to it rather than
	## duplicating slot-selection at the menu level.
	_live(p, "Import image…", ID_IMPORT_IMAGE)
	_live(p, "Import asset pack .zip…", ID_IMPORT_PACK)
	p.add_separator()
	_build_asset_pack_submenu(p)

	## §2.3: "Submenu listing the 24 families with filled/capacity counts."
	## `cartalith-assets` ships EIGHT families, not 24 -- verified reading
	## `slots.rs`/`library.rs` directly (`ASSET_LIBRARY_SCOPE.md` §1 already
	## recorded this: "eight families, seven of them closed vocabularies").
	## These two submenus list the real eight, split the way the crate itself
	## splits them (`Family::is_texture()`); each entry is a real scoped-open
	## shortcut into `AssetLibraryWindow`.
	##
	## **Both halves of §2.3's "filled/capacity counts" are drawn now.** The
	## rows used to print capacity alone, on the stated grounds that "no query
	## for it is exposed" -- `as_family_slots()` has always returned a per-slot
	## `filled` flag, and `asset_library_window.gd`'s own rail counts it that
	## way (`_refresh_rail_counts`). That was a gap in this file, not in the
	## engine. Counted on every popup rather than at build time, because an
	## import between two opens changes it.
	_icon_families_popup = PopupMenu.new()
	_icon_families_popup.name = "IconFamilies"
	_shell.style_popup(_icon_families_popup)
	p.add_child(_icon_families_popup)
	p.add_submenu_item("Icon families", "IconFamilies")
	p.set_item_tooltip(p.item_count - 1, AssetLibraryWindow.FAMILIES_NOTE)
	_icon_family_keys.clear()
	for fam in AssetLibraryWindow.FAMILIES:
		if not bool(fam.get("texture", false)):
			_icon_families_popup.add_item(String(fam["title"]), _icon_family_keys.size())
			_icon_family_keys.append(String(fam["key"]))
	_icon_families_popup.about_to_popup.connect(func():
		_refresh_family_counts(_icon_families_popup, _icon_family_keys))
	_icon_families_popup.id_pressed.connect(
		func(i: int): _host.open_asset_library(_icon_family_keys[i]))
	_refresh_family_counts(_icon_families_popup, _icon_family_keys)

	_texture_sets_popup = PopupMenu.new()
	_texture_sets_popup.name = "TextureSets"
	_shell.style_popup(_texture_sets_popup)
	p.add_child(_texture_sets_popup)
	p.add_submenu_item("Texture sets", "TextureSets")
	p.set_item_tooltip(p.item_count - 1, AssetLibraryWindow.FAMILIES_NOTE)
	_texture_family_keys.clear()
	for fam in AssetLibraryWindow.FAMILIES:
		if bool(fam.get("texture", false)):
			_texture_sets_popup.add_item(String(fam["title"]), _texture_family_keys.size())
			_texture_family_keys.append(String(fam["key"]))
	_texture_sets_popup.about_to_popup.connect(func():
		_refresh_family_counts(_texture_sets_popup, _texture_family_keys))
	_texture_sets_popup.id_pressed.connect(
		func(i: int): _host.open_asset_library(_texture_family_keys[i]))
	_refresh_family_counts(_texture_sets_popup, _texture_family_keys)

	p.add_separator()
	_live(p, "Apply library to map", ID_APPLY_LIBRARY)
	_live(p, "Clear library…", ID_CLEAR_LIBRARY)
	p.id_pressed.connect(_on_assets)

## §2.3's "filled/capacity counts", per family, on one of the two family
## submenus. The capacity rule is `asset_library_window.gd`'s own
## (`_refresh_rail_counts`): a custom family's capacity is however many slots
## it currently has, a frozen one's is the count the crate froze -- the two
## differ, and taking `as_family_slots().size()` for both would report a
## custom family as permanently full.
func _refresh_family_counts(pm: PopupMenu, keys: Array[String]) -> void:
	for i in keys.size():
		var key: String = keys[i]
		var fam := _family_by_key(key)
		if fam.is_empty():
			continue
		var slots: Array = _bridge.as_family_slots(key)
		var filled := 0
		for slot in slots:
			if bool((slot as Dictionary).get("filled", false)):
				filled += 1
		var capacity: int = slots.size() if bool(fam.get("custom", false)) 			else (fam["slots"] as Array).size()
		var idx := pm.get_item_index(i)
		if idx < 0:
			continue
		pm.set_item_text(idx, "%s   %d/%d" % [String(fam["title"]), filled, capacity])
		pm.set_item_tooltip(idx,
			"%d of %d slots filled. Opens the Asset library scoped to this family." % [
				filled, capacity])

func _family_by_key(key: String) -> Dictionary:
	for fam in AssetLibraryWindow.FAMILIES:
		if String((fam as Dictionary)["key"]) == key:
			return fam
	return {}

## AS-13 / omission O2: the `Assets ▸ Asset pack ▸` submenu
## `DCC_CONTROL_INDEX.md` §2.3.1 describes (24 controls, "19 backed-unwired
## against 1 engine gap") -- most of it real once `asset_bridge.rs`'s
## session exists.
##
## **Flattened 2026-08-25.** §2.3.1's four groups were built as three *nested
## submenus* (`Edit ▸`, `Batch ▸`, `Build ▸`) hanging off a fourth popup. The
## canvas draws one 306 px panel with four **labelled bands** and every row
## visible at once -- `ACTIVE PACK`, `EDIT`, `BATCH · 12 SELECTED`, `BUILD`,
## each `font:9px 'IBM Plex Mono';letter-spacing:.18em;color:#5f6468` -- and
## `Clear library…` at its foot. Three popups became zero, which also removes
## three more places for MN-10's trap (a submenu's `id_pressed` does not bubble
## to its parent in Godot 4) to reappear: there is now one popup and one
## handler.
##
## The Edit and Batch rows still need a *selected slot* / *multi-selection*
## that no `PopupMenu` item has, so they still open the real window where that
## context lives, and still say so in their tooltips. Build's four rows and
## Pack metadata need no such context and call straight into the engine.
func _build_asset_pack_submenu(p: PopupMenu) -> void:
	_asset_pack_popup = PopupMenu.new()
	_asset_pack_popup.name = "AssetPack"
	_shell.style_popup(_asset_pack_popup)
	p.add_child(_asset_pack_popup)
	p.add_submenu_item("Asset pack", "AssetPack")

	var ap := _asset_pack_popup
	## Active pack -- name/author/license/schema/filled-slots, live values
	## refreshed on every `about_to_popup` (the same pattern `_quality_popup`'s
	## own live-check row and `_refresh_recent_worlds()` already use).
	ap.add_separator("ACTIVE PACK")
	_ap_stats_idx = ap.item_count
	ap.add_item("— loading —")
	ap.set_item_disabled(ap.item_count - 1, true)
	## `schema   2 · STORED zip` is the canvas's own row, verbatim. The longer
	## "(frozen timestamps, byte-reproducible)" gloss this used to carry made
	## this popup 512 px wide against the canvas's 306 -- a `PopupMenu` sizes
	## itself to its widest item, and that one row was setting the width of the
	## whole panel. The gloss is on the tooltip.
	ap.add_item("schema   2 · STORED zip")
	ap.set_item_disabled(ap.item_count - 1, true)
	ap.set_item_tooltip(ap.item_count - 1,
		"Frozen timestamps, byte-reproducible: the same library exports to the same bytes.")
	ap.add_item("Pack metadata…   name · author · license", ID_AP_PACK_META)
	ap.set_item_tooltip(ap.item_count - 1,
		"SS2.3.1: a modal editing name / author / license. Writes straight through as_set_pack_info(); the three values above are the same record, read back.")

	ap.add_separator("EDIT")
	for row in [
		["Open library workspace   ▤", ID_ASSET_LIBRARY],
		["Import image into slot…", ID_IMPORT_IMAGE],
		["Sprite sheet slicer…   cols · rows · margin", ID_SLICER],
		["Add variant to slot   + variant", ID_IMPORT_IMAGE],
		["Replace · delete slot art", ID_ASSET_LIBRARY],
		["Slot transform   scale · fit · reset", -1],
		["Preview background   checker", ID_ASSET_LIBRARY],
	]:
		var wid: int = row[1]
		if wid < 0:
			_todo(ap, String(row[0]),
				"ItemTransform is real and shown in the inspector now, but no as_set_item_transform #[func] exists yet to write a new scale/pan back -- reading it is done, editing it is a smaller follow-on.")
		else:
			ap.add_item(String(row[0]), wid)
			ap.set_item_tooltip(ap.item_count - 1,
				"Opens the Asset Library window -- every Edit control needs a focused slot, which only the window's own grid provides.")

	## `Delete ⌫` used to carry the canvas's own accelerator glyph inside its
	## label, and MN-09 removed it for two reasons. **One of the two is now
	## false**: since 2026-08-24 `asset_library_window.gd` does bind Delete and
	## Backspace, to its own confirmed batch delete. The other still stands and
	## is why the glyph stays off — *this row does not delete anything*. Every
	## one of the five opens the window, as its own tooltip says, and the
	## binding lives in the window that has a selection to act on. A shortcut
	## printed on a row that is not the action is still a promise this build
	## cannot keep.
	##
	## The canvas's band reads `BATCH · 12 SELECTED`, a live count. This build
	## has no selection at the menu level to count -- the selection lives in the
	## window -- so the band is `BATCH` and the count is not faked.
	ap.add_separator("BATCH")
	for label in ["Tag…", "Collect into set…", "Rename…", "Duplicate", "Delete"]:
		ap.add_item(label, ID_ASSET_LIBRARY)
		ap.set_item_tooltip(ap.item_count - 1,
			"Opens the Asset Library window -- every batch op needs a multi-selection, which only the window's own grid (Shift-click ranges, Ctrl-click adds) provides. All five are real there, and Delete/Backspace there is the same confirmed batch delete.")

	ap.add_separator("BUILD")
	ap.add_item("Validate pack   warning count", ID_AP_VALIDATE)
	ap.add_item("Apply to map   compile & load", ID_APPLY_LIBRARY)
	ap.add_item("Import pack .zip…", ID_IMPORT_PACK)
	## The canvas draws `⌘⇧P` in the popup's own *accelerator column*, right
	## of the label — not inside the label. Baking it into the text put the
	## shortcut on the row twice, `Export pack .zip… ⌘⇧P    Ctrl+Shift+P`, one
	## of them naming a modifier key that exists on neither platform this port
	## ships on. `set_item_accelerator` alone renders exactly the canvas's
	## layout, in the notation the machine actually has.
	ap.add_item("Export pack .zip…", ID_AP_EXPORT)
	ap.set_item_accelerator(ap.item_count - 1, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_P)

	ap.add_separator()
	ap.add_item("Clear library…   destructive", ID_CLEAR_LIBRARY)

	## `GUI_GAP_REGISTER.md` **MN-10**. The three child popups this used to
	## build each connected their own `id_pressed` and this one never did, so
	## `Pack metadata…` -- the one live row the parent owned -- was enabled,
	## carried an id, had a written handler branch, and could not reach it.
	## With the children gone there is one connection and one place to lose.
	ap.id_pressed.connect(_on_assets)
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

## §2.3.1: "Pack metadata… — modal editing name / author / license."
##
## **This row was the file's own honesty rule broken from the other side.** It
## was enabled, carried an id, had a handler branch -- and the branch called
## `open_asset_library()`, so the one item §2.3.1 describes as a modal opened a
## window that edits something else. The write binding it needed
## (`as_set_pack_info`) had been on the bridge the whole time; nothing called
## it. Three fields, the modal the spec asks for, and the same
## `ConfirmationDialog`-with-a-body shape `asset_library_window.gd`'s
## `_prompt_text()` uses for its batch prompts.
func _open_pack_metadata() -> void:
	var info: Dictionary = _bridge.as_pack_info()
	var d := ConfirmationDialog.new()
	d.title = "Pack metadata"
	d.min_size = Vector2i(380, 0)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 8)
	var fields: Array[LineEdit] = []
	for spec in [["Name", "name"], ["Author", "author"], ["License", "license"]]:
		body.add_child(DccTheme.label(String(spec[0]), "text_dim", DccTheme.FS_SMALL))
		var le := LineEdit.new()
		le.text = String(info.get(String(spec[1]), ""))
		le.select_all_on_focus = true
		DccWidgets.well(le)
		body.add_child(le)
		fields.append(le)
	## The one thing the three fields do NOT cover, said rather than implied:
	## schema and the STORED-zip packaging are the exporter's, not the pack's.
	DccWidgets.note(body,
		"Written into pack.json on the next Export pack .zip. Schema and packaging are the exporter's and are not editable here.")
	d.add_child(body)
	d.ok_button_text = "Save"
	d.confirmed.connect(func():
		var ok := _bridge.as_set_pack_info(
			fields[0].text.strip_edges(), fields[1].text.strip_edges(),
			fields[2].text.strip_edges())
		_host.set_status("hint",
			"pack metadata updated" if ok else "this build has no as_set_pack_info()",
			"accent" if ok else "text_dim")
		_refresh_asset_pack_stats()
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	if _host.is_phone():
		DccWidgets.phone_window(d, _host)
	_host.add_child(d)
	if not DccWidgets.phone_present(d, _host):
		d.popup_centered()
	fields[0].grab_focus.call_deferred()

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
		ID_AP_PACK_META: _open_pack_metadata()

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
## **Rebuilt against the canvas 2026-08-25.** `DCC shell tablet 2560` is the one
## artboard that draws this menu open, and it draws it as four *labelled bands*
## carrying **fourteen indented rows** -- `Maps`, `Heightmaps · PNG · TIFF`,
## `GIS / GeoJSON`, `World Data · .zip · fields`, `Assets · → Assets`, and so on
## down through Export, Sources and Validation. The shell had collapsed each
## band into a single `Import ▸ Maps · Heightmaps · GIS · World data` row that
## opened the group's *first* route, so four of the canvas's fourteen
## destinations were reachable from the menu and ten were not.
##
## The rows are generated from `DataManagerWindow.ROUTES` rather than retyped:
## that table already carries the canvas's own label and badge for every route,
## and a second copy here is a second thing to keep in step.
##
## One thing the canvas has that a `PopupMenu` cannot draw: its badge is a
## right-aligned second column in `13px 'IBM Plex Mono';color:#6f7478`. Godot's
## only right column is the accelerator, which renders a real keystroke or
## nothing, so the badge is appended to the label instead. Content matches;
## the two-column alignment does not, and inventing a custom-drawn item to get
## it would be a new widget for one menu.
func _data(p: PopupMenu) -> void:
	## The canvas's own head row: `⧉ DATA MANAGER`, the window this whole menu
	## is a shortcut into. It had no row at all here -- the four group rows were
	## the only way in.
	_live(p, "⧉ Data manager", ID_DATA_MANAGER)
	_live(p, "World data tables…", ID_WORLD_DATA)
	_live(p, "Journey planner…", ID_JOURNEY_PLANNER, KEY_MASK_SHIFT | KEY_J)
	_live(p, "Travel library…", ID_TRAVEL_LIBRARY, KEY_MASK_SHIFT | KEY_L)

	_data_route_ids.clear()
	for group in DataManagerWindow.GROUP_ORDER:
		p.add_separator(String(group).to_upper())
		for r in DataManagerWindow.ROUTES:
			var route: Dictionary = r
			if String(route["group"]) != group:
				continue
			var badge := String(route.get("badge", ""))
			p.add_item("%s%s" % [String(route["label"]),
				("   " + badge) if badge != "" else ""],
				ID_DATA_ROUTE_FIRST + _data_route_ids.size())
			p.set_item_tooltip(p.item_count - 1, String(route.get("sub", "")))
			_data_route_ids.append(String(route["id"]))

	_build_vault_rows(p)
	p.id_pressed.connect(func(id: int) -> void:
		if id >= ID_DATA_ROUTE_FIRST:
			var i := id - ID_DATA_ROUTE_FIRST
			if i < _data_route_ids.size():
				_host.open_data_manager_route(_data_route_ids[i])
			return
		match id:
			ID_DATA_MANAGER: _host.open_data_manager()
			ID_WORLD_DATA: _host.open_world_data()
			ID_JOURNEY_PLANNER: _host.open_journey_planner()
			ID_TRAVEL_LIBRARY: _host.open_travel_library()
			ID_VAULT: _host.open_vault_overview()
	)

## **The Markdown vault's program-scope entry point** (2026-08-24, `design/
## Cartalith Menu Structure v3.dc.html`).
##
## v3 draws `▾ VAULT` as its own top-bar menu. **It is not one here** (owner,
## 2026-08-24: *"the vault menu can be shoved into data"*) — a vault is a
## folder of files this app reads and writes, which is what every other row
## in this menu already is, and an eighth top-level menu for one window is
## the kind of bar growth `DCC_SHELL_SPEC.md` §2 exists to prevent.
##
## One live row, because there is exactly one window behind it
## (`vault_window.gd`, `MARKDOWN_VAULT_SCOPE.md` milestone 1) and a second
## row onto the same window is the duplicate-owner shape this shell keeps
## having to undo. Three of v3's seven rows are that window's own content and
## say so in the tooltip; the other three have no implementation at all and
## are `_todo`, not invented.
func _build_vault_rows(p: PopupMenu) -> void:
	p.add_separator()
	_live(p, "Markdown vault ▸ Connect · Browse · Links", ID_VAULT)
	p.set_item_tooltip(p.item_count - 1,
		"Connect or re-link a vault folder (any folder of .md files -- Obsidian is one, "
		+ "and nothing here requires it), browse it, attach a settlement, province or "
		+ "continent to a note section, and see every link in this world. Also carries "
		+ "v3's frontmatter mapping and sync direction, as a per-write choice rather "
		+ "than a global setting: the field-fill picker chooses which derived keys go "
		+ "in, fills only empty ones by default, previews every write, and refuses if "
		+ "the note changed since the preview. Cartalith never rewrites a note's body.")
	_live(p, "Create a note from a template…", ID_VAULT)
	p.set_item_tooltip(p.item_count - 1,
		"Opens the same vault panel: pick any entity's Linked notes and its "
		+ "New note from a template block writes Settlements/{name}.md (or the "
		+ "matching folder for a province, continent or faction) from one of your "
		+ "own templates. A template is any .md in the vault with \"template\" in "
		+ "its path -- Cartalith ships none of its own, and copies yours verbatim "
		+ "with only the entity's name substituted. It refuses an existing path "
		+ "rather than overwriting a note. GUI_GAP_REGISTER.md VA-02.")
	## `GUI_GAP_REGISTER.md` **VA-01**, built 2026-08-25. The register's own
	## framing -- an on-demand index that stalls versus a persistent one that
	## goes stale -- was a false pair: a `stat` is not a read, so the index is
	## persisted AND kept correct per file by `(modified, len)`.
	_live(p, "Vault index ▸ Backlinks · missing & orphan notes…", ID_VAULT)
	p.set_item_tooltip(p.item_count - 1,
		"Opens the vault panel's Index section: build or refresh the reverse index, "
		+ "then see which links point at a note that does not exist and which notes "
		+ "nothing links to. Building reads every note once; a refresh re-opens only "
		+ "the files whose size or modified time changed, so ten edits in Obsidian "
		+ "cost ten reads and not the whole vault. Per note it keeps the links, the "
		+ "Cartalith blocks and a 64-bit word fingerprint -- never the prose. Backlinks "
		+ "and unlinked mentions for one entity are on that entity's own panel. "
		+ "GUI_GAP_REGISTER.md VA-01.")

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
	## **The five bands are §2.5's own five groups**, drawn as the canvas draws
	## a group inside a menu: a labelled band, `font:9px 'IBM Plex Mono';
	## letter-spacing:.18em;color:#5f6468`. No artboard draws Preferences open,
	## so the *names* come from §2.5's table and the *treatment* from the three
	## menus that are drawn (File's `STORAGE LOCATIONS`, Assets' `ACTIVE PACK`/
	## `EDIT`/`BATCH`/`BUILD`, Data's `IMPORT`/`EXPORT`/`SOURCES`/`VALIDATION`).
	## Until now this menu had four unlabelled rules for five groups -- Tiles &
	## LOD and Memory shared one band, so `Tiled LOD` and `Undo history` read as
	## the same subject.
	p.add_separator("PERFORMANCE")
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
		var use_gpu := bool(_bridge.param_get("use_gpu"))
		p.set_item_checked(gpu_idx, use_gpu)
		## §2.5: "Toggle + backend readout (`WebGPU · on`)." The row carried no
		## backend at all. See `_active_backend()` for why it can be blank.
		var backend := _active_backend()
		p.set_item_text(gpu_idx, "GPU acceleration" if backend == ""
			else "GPU acceleration   %s · %s" % [backend, "on" if use_gpu else "off"])
		var busy := _bridge.gpu_settings_locked()
		var why := "A generation is running. Every setting in this group takes effect on the next generate anyway, and the engine object belongs to the worker thread until this one finishes."
		p.set_item_disabled(gpu_idx, busy)
		p.set_item_tooltip(gpu_idx, why if busy else GPU_TOGGLE_TIP)
		for i in _gpu_pref_rows.size():
			var row: int = _gpu_pref_rows[i]
			if row < p.item_count:
				p.set_item_disabled(row, busy)
				p.set_item_tooltip(row, why if busy else _gpu_pref_tips[i])
		if _undo_pref_row >= 0 and _undo_pref_row < p.item_count:
			p.set_item_tooltip(_undo_pref_row, _undo_pref_tip())
		_refresh_working_set_row(p))
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
	_todo(p, "CPU worker threads",
		"SS2.5: an integer from 1 to the logical core count, default cores - 4 (its own example is 12 of 16). Rayon builds its global pool implicitly on first use and this port never calls ThreadPoolBuilder, so there is no #[func] to set it and no pool init to set it at. A submenu ladder (1 / quarter / half / cores-4 / all) over one binding closes it, and cores-4 is the spec's own default to honour rather than a number to pick.")
	if _bridge.gpu_api:
		_build_gpu_vram_menu(p)
		_build_gpu_fallback_menu(p)
	else:
		_todo(p, "VRAM budget", "Same -- this build predates the multi-GPU API.")
		_todo(p, "Fallback when VRAM full", "Same.")
	p.add_separator("GRAPHICS")

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

	_todo(p, "Anti-aliasing · anisotropy",
		"SS2.5 asks for off / MSAA 2x / 4x / 8x and anisotropy 1-16. Both are 3D-viewport settings and there is no 3D viewport (DECISIONS.md section 4 defers it to Phase 3). The 2D map path composites whole rasters, where a sample count means nothing -- bolting MSAA onto it would be a control with no effect.")
	_todo(p, "Colour management",
		"SS2.5 asks for sRGB / Display P3 / linear. The renderer is sRGB-only end to end: render.rs writes 8-bit sRGB bytes and nothing carries a colour space through to the texture. A three-row radio that always resolves to sRGB is exactly the enabled-and-inert row this menu forbids.")
	_todo(p, "3D viewport defaults",
		"SS2.5's four parameters verbatim -- relief exaggeration, detail, light, flatten oceans -- replacing the reference's #genV3dSec, and exempt from the finalize lock. There is no 3D viewport to give defaults to.")
	_todo(p, "Lighting rig defaults",
		"SS2.5 asks for azimuth, elevation, ambient and multidirectional on/off. The values themselves already ship, per layer, in SS7's Layer properties LIGHT group (azimuth 315 deg, elevation 45 deg, strength 0.62, multidirectional 8 lights) in the Cartography workspace. What is missing is only the project-level default store those per-layer values would seed from -- a settings key, not new rendering.")
	p.add_separator("TILES & LOD")
	## **§2.5's Tiles & LOD group is four items, and this menu shipped all four
	## behind one disabled row** reading `Tiled LOD · tile size · LOD level ·
	## chunk debug`. That row was scrupulously honest about what was missing and
	## wrong about the shape: two of the four are real. `atlas_tile_size()` /
	## `atlas_set_tile_size()` have been on the bridge since the atlas landed
	## and the row itself said "nothing in Preferences calls it"; the chunk-debug
	## overlay was fully built and sitting under `Help ▸ LOD debug`. Folding a
	## live control in with a gap is how a built thing stays unreachable, which
	## is the defect class `PARITY_AUDIT.md` §23 exists for. Split here into
	## §2.5's own four rows, each carrying its own status.

	## §2.5: "Tiled LOD — `auto on zoom` (default) · `manual`. Replaces
	## #lodAutoChk." Live since 2026-08-30.
	##
	## The reason that stood here was accurate and stopped one step early: there
	## WAS no public suppressor, so a manual row "would be the second half of a
	## radio pair that does nothing". The missing half was three functions on
	## `ViewportHost`, not a design problem.
	##
	## **Manual ships with its own way in, and that is not optional.** The
	## reference gates only the WHEEL handler on `state.lodAuto` (line 13986)
	## and keeps `enterLodFromView` as the explicit route; a suppressor without
	## one would make deep detail unreachable, which is worse than not offering
	## the choice at all. `Enter deep detail now` is that route, and it reports
	## when the camera is not far enough in rather than appearing inert.
	_build_tiled_lod_menu(p)

	_build_atlas_tiles_menu(p)
	_build_atlas_cache_menu(p)
	## §2.5 puts the chunk-debug overlay in **this** group. It was built under
	## `Help ▸ LOD debug`, on the stated reasoning that "this shell has no Atlas
	## panel" and a developer overlay belongs beside `Generation info…`. The
	## first half stopped being true one row above this line; the second is a
	## preference for Help over the spec, and `CLAUDE.md`'s own rule is that an
	## owner decision or the newer canvas wins over the shell's improvisation.
	## Moved, not duplicated -- Help no longer carries it.
	_build_lod_debug_submenu(p)
	## PR-11, live. §2.5 asked for a depth control; the engine's bound is a
	## **byte budget** rather than a step count, because one height field is
	## 16 MB at 2048² and 256 MB at 8192² -- a flat "5 deep" would commit to
	## 1.25 GB of undo buffer on the largest world this shell offers. The
	## step count (5, the reference's own `MAX_UNDO`) is still the ceiling;
	## the budget is what actually binds on a big world. Rows are budgets,
	## and each shows how many steps that buys at the current resolution,
	## which is the honest way to present a bound whose depth is
	## resolution-dependent (`undo.rs`).
	p.add_separator("MEMORY")
	_undo_budget_popup = PopupMenu.new()
	_undo_budget_popup.name = "UndoBudget"
	for i in UNDO_BUDGETS_MB.size():
		_undo_budget_popup.add_radio_check_item("%d MB" % UNDO_BUDGETS_MB[i], i)
	_undo_budget_popup.add_separator()
	_undo_budget_popup.add_item("Clear undo history now", ID_PREF_UNDO_CLEAR)
	_undo_budget_popup.id_pressed.connect(_on_undo_budget)
	_undo_budget_popup.about_to_popup.connect(_refresh_undo_budget_menu)
	_shell.style_popup(_undo_budget_popup)
	p.add_child(_undo_budget_popup)
	p.add_submenu_item("Undo history", "UndoBudget")
	_undo_pref_row = p.item_count - 1
	## §2.5's Memory group has three items -- Undo history (above, a real
	## gap), Working set and Clear caches -- but only the first ever made it
	## into this menu; the other two were missing outright, not even as
	## honest `_todo()`s, found in the 2026-08-19 GUI audit alongside the
	## orphaned `PerformanceWindow` this now opens. Working set is real:
	## `OS.get_static_memory_usage()`, the same source the menu bar's own
	## `top_mem` readout already uses (`app.gd`'s `_wire_status()`).
	## §2.5: "Working set — read-only, `1.6 GB of 12 GB`." The spec asks for an
	## inline readout; this menu had only the dialog below it, so the one number
	## §2.5 wanted on the row itself was two clicks away. Both now: the row is
	## the spec's line, the dialog is `PerformanceWindow`, which carries more.
	p.add_item("Working set")
	_working_set_row = p.item_count - 1
	p.set_item_disabled(_working_set_row, true)
	_refresh_working_set_row(p)
	_live(p, "Working set…", ID_PREF_WORKING_SET)
	## PR-12, live 2026-08-24. There is now a real cache to clear: the
	## persistent tile atlas (`bake_bridge.rs`), written by WORLD ▸ Generate ▸ Finalize ▸
	## Bake. Clearing un-finalizes too -- a lock protecting nothing would
	## strand the world read-only for no reason.
	_live(p, "Clear caches…", ID_PREF_CLEAR_CACHES)
	p.add_separator("APPLICATION")

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
	_todo(p, "Units",
		"SS2.5 asks for km / mi (the reference's #calUnitSeg). The shell is km-only, and the work is not this row: every readout that prints km would have to go through one formatter first -- the status bar's cursor coordinates, the scale bar, Sculpt's brush km equivalent (#sBrushKm), Measure's running total and per-segment lengths, and Region select's km column. A setting here with five call sites still printing km would be worse than no setting.")
	## **This row's reason was stale.** It said "No shortcut table yet" after
	## `Help ▸ Keyboard shortcuts…` shipped a live one (`shortcuts_dialog.gd`,
	## which walks these menus). The gap §2.5 names is a different one and is
	## real: Help's list is read-only, and this row asks for an **editable,
	## per-context** table -- one that writes.
	_todo(p, "Keyboard shortcuts…",
		"SS2.5 asks for an editable, per-context table. Help > Keyboard shortcuts... already lists every binding, read-only, by walking these menus -- so the list exists and what is missing is rebinding: a per-context store in DccSettings that both the menu accelerators here and app.gd's own key handlers read back instead of hard-coding.")
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

## §2.5's "backend readout (`WebGPU · on`)" beside the GPU-acceleration
## toggle. Three deliberate departures, all forced:
##
##   1. The spec's literal `WebGPU` is the *browser* reference's backend. This
##      port is native, so what gets printed is whatever `wgpu` really chose --
##      `Vulkan`, `Dx12`, `Metal`, `Gl` -- rather than the spec's example
##      string, which would be false on every machine this ships to.
##   2. Read off `_gpu_devices` **only**, never by enumerating. `gpu_devices()`
##      stands up a `wgpu::Instance` and walks every backend, and
##      `_build_gpu_devices_menu()` documents at length the signal-11 that
##      caused. So until `Devices ▸` has been opened once, this is blank --
##      honest, because nothing has asked the driver yet.
##   3. Blank again when the selected devices disagree on a backend, or when
##      selection is automatic and the enumerated adapters do not all share
##      one: `PowerPreference::HighPerformance` picks the adapter, not this
##      code, so naming one of several would be a guess printed as a fact.
func _active_backend() -> String:
	var sel := _bridge.gpu_selected_devices()
	var found := ""
	for d in _gpu_devices:
		var dd: Dictionary = d
		if bool(dd.get("software", false)):
			continue
		if not sel.is_empty() and not (String(dd.get("key", "")) in sel):
			continue
		var b := String(dd.get("backend", ""))
		if found == "":
			found = b
		elif found != b:
			return ""
	return found

## §2.5 Memory: "Working set — read-only, `1.6 GB of 12 GB`."
##
## Numerator is `OS.get_static_memory_usage()`, the same source the menu bar's
## `top_mem` slot and `PerformanceWindow` already read, so the three cannot
## disagree. Denominator is `OS.get_memory_info()["physical"]`, which is `-1`
## on any platform that will not report it -- in which case only the
## numerator is printed rather than a fabricated total.
func _refresh_working_set_row(p: PopupMenu) -> void:
	if _working_set_row < 0 or _working_set_row >= p.item_count:
		return
	var used := OS.get_static_memory_usage()
	var info := OS.get_memory_info()
	var total := int(info.get("physical", -1))
	if total > 0:
		p.set_item_text(_working_set_row, "Working set   %s of %s" % [_gb(used), _gb(total)])
		p.set_item_tooltip(_working_set_row,
			"This process's own allocations against the machine's physical RAM. Not GPU memory -- that is under Performance > Devices. Working set... below opens the full breakdown.")
	else:
		p.set_item_text(_working_set_row, "Working set   %s" % _gb(used))
		p.set_item_tooltip(_working_set_row,
			"This process's own allocations. This platform reports no physical-RAM total (OS.get_memory_info() physical is -1), so the of-N half of SS2.5's line is left off rather than invented.")

## Coarse GB/MB for the two memory readouts. Same floor logic as `_mb()`, one
## unit up, because a working set is gigabytes and `1638 MB` is harder to read
## against a 12 GB machine than `1.6 GB`.
static func _gb(bytes: int) -> String:
	if bytes <= 0:
		return "0 MB"
	if bytes < 1073741824:
		return "%d MB" % int(round(float(bytes) / 1048576.0))
	return "%.1f GB" % (float(bytes) / 1073741824.0)

# -- §2.5 Tiles & LOD -----------------------------------------------------------

## §2.5's Tiled LOD mode, plus the manual mode's own entry point.
func _build_tiled_lod_menu(p: PopupMenu) -> void:
	_tiled_lod_popup = PopupMenu.new()
	_tiled_lod_popup.name = "TiledLod"
	_shell.style_popup(_tiled_lod_popup)
	_tiled_lod_popup.add_radio_check_item("Auto on zoom", ID_LOD_MODE_AUTO)
	_tiled_lod_popup.set_item_tooltip(0,
		"The default, and the reference's own: zooming past roughly one screen pixel per grid cell brings the tile pyramid up by itself.")
	_tiled_lod_popup.add_radio_check_item("Manual", ID_LOD_MODE_MANUAL)
	_tiled_lod_popup.set_item_tooltip(1,
		"Zooming in never enters the pyramid; use Enter deep detail now below. Panning a deep view stays cheap either way -- the mode is about entering, not about staying.")
	_tiled_lod_popup.add_separator()
	_tiled_lod_popup.add_item("Enter deep detail now", ID_LOD_ENTER_NOW)
	_tiled_lod_popup.add_item("Leave deep detail", ID_LOD_LEAVE_NOW)
	_tiled_lod_popup.id_pressed.connect(_on_tiled_lod)
	_tiled_lod_popup.about_to_popup.connect(_refresh_tiled_lod_menu)
	p.add_child(_tiled_lod_popup)
	p.add_submenu_item("Tiled LOD", "TiledLod")

func _refresh_tiled_lod_menu() -> void:
	if _tiled_lod_popup == null or _host == null or _host.viewport == null:
		return
	var vp = _host.viewport
	if not vp.has_method("lod_auto"):
		return
	var auto: bool = vp.lod_auto()
	_tiled_lod_popup.set_item_checked(0, auto)
	_tiled_lod_popup.set_item_checked(1, not auto)
	var up: bool = vp.lod_active()
	var enter_i := _tiled_lod_popup.get_item_index(ID_LOD_ENTER_NOW)
	var leave_i := _tiled_lod_popup.get_item_index(ID_LOD_LEAVE_NOW)
	## Both rows say why they cannot run, rather than only greying out.
	_tiled_lod_popup.set_item_disabled(enter_i, up)
	_tiled_lod_popup.set_item_tooltip(enter_i, "Already in the deep-detail view." if up
		else "Brings the tile pyramid up at the current camera. Needs the camera zoomed past the detail threshold -- it will say so if it is not.")
	## Leaving under Auto would be undone by the next camera move, so it is
	## refused with that as the reason instead of being offered and reverting.
	_tiled_lod_popup.set_item_disabled(leave_i, not up or auto)
	_tiled_lod_popup.set_item_tooltip(leave_i,
		"Not in the deep-detail view." if not up
		else ("Auto mode would bring it straight back on the next camera move. Switch to Manual first." if auto
			else "Drops back to the base raster without moving the camera."))

func _on_tiled_lod(id: int) -> void:
	if _host == null or _host.viewport == null:
		return
	var vp = _host.viewport
	if not vp.has_method("set_lod_auto"):
		return
	match id:
		ID_LOD_MODE_AUTO, ID_LOD_MODE_MANUAL:
			var auto := id == ID_LOD_MODE_AUTO
			DccSettings.set_lod_auto(auto)
			vp.set_lod_auto(auto)
			_host.set_status("hint", "tiled LOD: %s" % (
				"auto on zoom" if auto else "manual — use Enter deep detail now"), "text_dim")
		ID_LOD_ENTER_NOW:
			if vp.request_lod_entry():
				_host.set_status("hint", "deep detail on", "text_dim")
			else:
				_host.set_status("hint",
					"not zoomed in far enough for deep detail — zoom in, then try again", "text_dim")
		ID_LOD_LEAVE_NOW:
			vp.release_lod_entry()
			_host.set_status("hint", "deep detail off", "text_dim")
	_refresh_tiled_lod_menu()

## §2.5: "Tile size · LOD levels — 256/512/1024; levels 0–8 (#lodMaxLevel)."
##
## The tile size is real and was never called from here: `atlas_tile_size()`
## and `atlas_set_tile_size()` are both on the bridge, and the collapsed row
## this replaces said so in its own tooltip for six days.
##
## **The levels half is live since 2026-08-30, and the thing that unblocked it
## was a settings key, not an engine call.** The reason that stood here was
## right about the blocker: bake depth already had ONE owner, the WORLD dock's
## Finalize foot, and it was a private field, so a ladder here would have been
## a copy free to disagree with the number `bake_all()` is actually called
## with. Promoting it to `DccSettings.bake_depth()` -- read by BOTH surfaces,
## re-read by the dock on every atlas refresh -- removes the objection rather
## than working around it.
##
## The range is §2.5's own 0-8. The engine's ceiling is higher
## (`lod_bridge::MAX_LEVEL` is 10), so the clamp is the spec's, not a
## capability limit. Each rung carries its tile count, because depth 8 is
## 87 381 tiles and a synchronous bake that deep is a decision rather than a
## click -- the same reason the dock already prints the count before the user
## commits.
func _build_atlas_tiles_menu(p: PopupMenu) -> void:
	_tile_size_popup = PopupMenu.new()
	_tile_size_popup.name = "AtlasTileSize"
	_shell.style_popup(_tile_size_popup)
	for i in ATLAS_TILE_SIZES.size():
		_tile_size_popup.add_radio_check_item("%d px" % ATLAS_TILE_SIZES[i],
			ID_LOD_TILE_FIRST + i)
	_tile_size_popup.add_separator()
	_lod_levels_popup = PopupMenu.new()
	_lod_levels_popup.name = "AtlasLodLevels"
	_shell.style_popup(_lod_levels_popup)
	for d in range(0, 9):
		_lod_levels_popup.add_radio_check_item("LOD 0–%d   %s tile%s" % [
			d, _pyramid_tiles(d), "" if d == 0 else "s"], ID_LOD_LEVEL_FIRST + d)
	_lod_levels_popup.id_pressed.connect(_on_lod_levels)
	_lod_levels_popup.about_to_popup.connect(_refresh_lod_levels_menu)
	_tile_size_popup.add_child(_lod_levels_popup)
	_tile_size_popup.add_submenu_item("LOD levels", "AtlasLodLevels")
	_tile_size_popup.set_item_tooltip(_tile_size_popup.item_count - 1,
		"How deep Bake ALL levels & finalize goes. The same setting as WORLD > Finalize > Bake depth -- one store, two entry points.")
	_tile_size_popup.id_pressed.connect(_on_tile_size)
	_tile_size_popup.about_to_popup.connect(_refresh_tile_size_menu)
	p.add_child(_tile_size_popup)
	p.add_submenu_item("Tile size · LOD levels", "AtlasTileSize")
	_refresh_tile_size_menu()

## Tiles in a pyramid of `depth` levels: (4^(depth+1) - 1) / 3, thousands
## separated. Duplicated from `world_workspace.gd::_pyramid_tiles` on purpose
## and it is four lines of exact arithmetic, not a shared value: making this a
## helper on a third object to serve two label builders is the abstraction the
## `/ponytail` rule exists to refuse. The NUMBER that must not diverge is the
## depth, and that is a settings key, not this.
func _pyramid_tiles(depth: int) -> String:
	var n := 0
	for z in range(0, depth + 1):
		n += (1 << z) * (1 << z)
	var out := ""
	var txt := str(n)
	for i in txt.length():
		if i > 0 and (txt.length() - i) % 3 == 0:
			out += " "
		out += txt[i]
	return out

func _refresh_lod_levels_menu() -> void:
	if _lod_levels_popup == null:
		return
	var cur := DccSettings.bake_depth()
	for d in range(0, 9):
		_lod_levels_popup.set_item_checked(d, d == cur)

func _on_lod_levels(id: int) -> void:
	var d := id - ID_LOD_LEVEL_FIRST
	if d < 0 or d > 8:
		return
	DccSettings.set_bake_depth(d)
	_refresh_lod_levels_menu()
	## The dock re-reads the key on its own atlas refresh, but nudging it here
	## means the change is visible immediately rather than at the next bake or
	## cache change -- the same courtesy `_on_tile_size` already does.
	if _host != null and _host.has_method("refresh_atlas_status"):
		_host.refresh_atlas_status()
	_host.set_status("hint", "bake depth set to LOD 0–%d (%s tile%s)" % [
		d, _pyramid_tiles(d), "" if d == 0 else "s"], "text_dim")

## Locked once anything is baked, and this is a real constraint rather than
## caution: `atlas_set_tile_size()` writes `bake.tile_size` and nothing else
## (`lib.rs` line 7273), so chunks already on disk keep the size they were
## written at. Changing it with a populated atlas mixes two tile sizes under
## one world key. Clearing the cache first is the way through, which is why
## `Atlas cache ▸ Clear` sits directly below this row.
func _refresh_tile_size_menu() -> void:
	var st: Dictionary = _bridge.atlas_status()
	var cur := int(st.get("tile_size", _bridge.atlas_tile_size()))
	var baked := int(st.get("chunks", 0))
	for i in ATLAS_TILE_SIZES.size():
		_tile_size_popup.set_item_checked(i, ATLAS_TILE_SIZES[i] == cur)
		_tile_size_popup.set_item_disabled(i, baked > 0)
		_tile_size_popup.set_item_tooltip(i, "" if baked <= 0 else
			"%d chunk%s already baked at %d px. Clear the atlas cache below first -- a size change does not rewrite tiles already on disk." % [
				baked, "" if baked == 1 else "s", cur])

func _on_tile_size(id: int) -> void:
	var i := id - ID_LOD_TILE_FIRST
	if i < 0 or i >= ATLAS_TILE_SIZES.size():
		return
	_bridge.set_atlas_tile_size(ATLAS_TILE_SIZES[i])
	_refresh_tile_size_menu()
	if _host.has_method("refresh_atlas_status"):
		_host.refresh_atlas_status()
	_host.set_status("hint", "tile size %d px — applies to the next bake" % ATLAS_TILE_SIZES[i],
		"text_dim")

## §2.5: "Atlas cache — size cap in GB + Clear (#lodBakeBtn,
## #lodClearAtlasBtn)."
##
## Clear is the same action as `Memory ▸ Clear caches…` and goes through the
## same `_clear_caches()` -- one implementation, two entry points, the shape
## `Storage locations…` already has in File and Preferences, not a second
## clearer. The cap and the reference's Refine pass are both real gaps and say
## which kind of gap each is.
func _build_atlas_cache_menu(p: PopupMenu) -> void:
	_atlas_popup = PopupMenu.new()
	_atlas_popup.name = "AtlasCache"
	_shell.style_popup(_atlas_popup)
	_atlas_stats_idx = 0
	_atlas_popup.add_item("— loading —")
	_atlas_popup.set_item_disabled(0, true)
	_todo(_atlas_popup, "Size cap · GB",
		"The store is real and measured (atlas_status() reports chunks and bytes), but nothing evicts: bake_bridge writes chunks and only atlas_clear() removes them, all of them at once. A cap needs an eviction policy -- which chunk goes when the cap is hit -- and there is no access order or level priority recorded to choose by. The GB ladder itself would mirror Performance > VRAM budget, which already solves the presentation half.")
	## The reference's `#lodRefineBtn`. Live since 2026-08-30: the accessor
	## that was owed -- `ViewportHost.visible_grid_rect()` -- now returns the
	## grid rectangle the camera is showing plus the pyramid level that
	## rectangle resolves to, which is exactly `bake_visible`'s five
	## arguments and nothing more. Measured on a 256x192 world at 6x zoom:
	## 16 chunks baked in 0.26 s.
	_atlas_popup.add_item("Refine detail for the current view", ID_LOD_REFINE_VIEW)
	_atlas_popup.set_item_tooltip(_atlas_popup.item_count - 1,
		"Bakes the pyramid chunks the current view touches, at the level this zoom resolves to, into the atlas cache -- so panning back over this area reads from disk instead of synthesizing. Zoom in first: at a fitted view the pyramid is not up and there is nothing to refine.")
	_atlas_popup.add_separator()
	_atlas_popup.add_item("Clear atlas cache now…", ID_LOD_CLEAR_ATLAS)
	_atlas_popup.set_item_tooltip(_atlas_popup.item_count - 1,
		"The same action as Preferences > Memory > Clear caches..., confirmation and all -- one clearer with two entry points, since SS2.5 lists it in both groups.")
	_atlas_popup.id_pressed.connect(func(id: int):
		if id == ID_LOD_CLEAR_ATLAS:
			_clear_caches()
		elif id == ID_LOD_REFINE_VIEW:
			_refine_current_view())
	_atlas_popup.about_to_popup.connect(_refresh_atlas_cache_menu)
	p.add_child(_atlas_popup)
	p.add_submenu_item("Atlas cache", "AtlasCache")
	_refresh_atlas_cache_menu()

## The live store, from `atlas_status()` -- the same dictionary the status
## bar's `atlas` slot already reads, so the menu cannot report a different
## cache than the bar does.
func _refresh_atlas_cache_menu() -> void:
	if _atlas_stats_idx < 0:
		return
	var st: Dictionary = _bridge.atlas_status()
	if st.is_empty():
		_atlas_popup.set_item_text(_atlas_stats_idx, "No atlas in this build")
		_atlas_popup.set_item_tooltip(_atlas_stats_idx,
			"This GDExtension build has no atlas_status().")
		return
	var chunks := int(st.get("chunks", 0))
	var deepest := int(st.get("deepest_level", -1))
	_atlas_popup.set_item_text(_atlas_stats_idx, "%d chunk%s · %s · %s" % [
		chunks, "" if chunks == 1 else "s", String(st.get("bytes_text", "0 B")),
		"empty" if deepest < 0 else "to LOD %d" % deepest])
	_atlas_popup.set_item_tooltip(_atlas_stats_idx,
		"%s\nRoot: %s" % [String(st.get("text", "")), String(st.get("root", ""))])
	var clear_idx := _atlas_popup.get_item_index(ID_LOD_CLEAR_ATLAS)
	if clear_idx >= 0:
		_atlas_popup.set_item_disabled(clear_idx, chunks <= 0)

## §2.5: "Clear caches… — **Confirmation**; clears atlas + field caches, never
## project data."
##
## **The confirmation was missing.** The row fired the moment it was clicked,
## and it is genuinely destructive twice over: it deletes every baked chunk
## (minutes of bake time, and `bake_all` at depth 5 is 1365 tiles) and it
## clears the finalize lock, so a finalized world silently becomes editable
## again. `Assets ▸ Clear library…` is marked destructive and does confirm;
## this one is at least as destructive and did not. The freed-bytes figure the
## old handler only printed *afterwards* is what the prompt shows *first*.
func _clear_caches() -> void:
	var st: Dictionary = _bridge.atlas_status()
	var chunks := int(st.get("chunks", 0))
	var freed := String(st.get("bytes_text", "0 B"))
	if chunks <= 0:
		_host.set_status("hint", "nothing baked for this world — no cache to clear", "text_dim")
		return
	var d := ConfirmationDialog.new()
	d.title = "Clear cached tiles?"
	d.dialog_text = ("Deletes %d baked chunk%s (%s) for this world.\n\n"
		+ "The world itself, its parameters and every edit are untouched -- only the "
		+ "rendered tile pyramid goes, and it can be baked again from WORLD > Finalize. "
		+ "%s") % [chunks, "" if chunks == 1 else "s", freed,
			"This world is finalized; clearing releases that lock, because a lock protecting nothing would strand it read-only."
				if bool(st.get("finalized", false)) else ""]
	d.ok_button_text = "Clear %s" % freed
	d.confirmed.connect(func():
		var n := _bridge.atlas_clear()
		_host.set_status("hint", "cleared %d baked chunk%s (%s)" % [
			n, "" if n == 1 else "s", freed], "text_dim")
		if _host.has_method("refresh_atlas_status"):
			_host.refresh_atlas_status()
		_refresh_atlas_cache_menu()
		_refresh_tile_size_menu()
		d.queue_free())
	d.canceled.connect(func(): d.queue_free())
	if _host.is_phone():
		DccWidgets.phone_window(d, _host)
	_host.add_child(d)
	if not DccWidgets.phone_present(d, _host):
		d.popup_centered()

func _on_preferences(id: int, p: PopupMenu) -> void:
	if id == ID_PREF_STORAGE:
		_host.open_storage_locations()
		return
	if id == ID_PREF_WORKING_SET:
		_host.open_performance()
		return
	if id == ID_PREF_CLEAR_CACHES:
		_clear_caches()
		return
	if id != ID_PREF_GPU:
		return
	var idx := p.get_item_index(ID_PREF_GPU)
	var on := not bool(_bridge.param_get("use_gpu"))
	if _bridge.param_set("use_gpu", on):
		p.set_item_checked(idx, on)
		## §2.5's backend readout carries the on/off half of `WebGPU · on`, so
		## it has to move with the check mark rather than waiting for the next
		## `about_to_popup` -- the popup stays open after a toggle, and a row
		## reading `Vulkan · off` beside a ticked box is the kind of
		## disagreement this file's own region-check bug already cost once.
		var backend := _active_backend()
		p.set_item_text(idx, "GPU acceleration" if backend == ""
			else "GPU acceleration   %s · %s" % [backend, "on" if on else "off"])

# -- §2.5 Memory ▸ Undo history (PR-11) ---------------------------------------

## The parent row's own tooltip: the live cost, and -- the number that makes
## the budget legible -- what one step costs at *this* resolution.
func _undo_pref_tip() -> String:
	var s: Dictionary = _bridge.undo_stats()
	if s.is_empty():
		return "This build's engine has no undo bindings."
	var step := int(s.get("step_bytes", 0))
	var head := "%s · %s of a %s budget." % [
		_undo_depth_text(s), _mb(int(s.get("bytes", 0))), _mb(int(s.get("budget_bytes", 0)))]
	if step <= 0:
		return head + " No world yet, so a step costs nothing."
	return head + (" One step costs %s at this resolution, so the budget holds %d of the %d " +
		"steps the reference keeps. Cleared by every Generate.") % [
			_mb(step), _steps_affordable(s), int(s.get("max_steps", 5))]

## How many steps the current budget actually buys at the current step size --
## the engine's own eviction rule (`undo.rs`: both bounds, floor of one),
## restated so the menu can show it before a push proves it.
func _steps_affordable(s: Dictionary) -> int:
	var step := int(s.get("step_bytes", 0))
	var max_steps := int(s.get("max_steps", 5))
	if step <= 0:
		return max_steps
	return maxi(1, mini(max_steps, int(s.get("budget_bytes", 0)) / step))

func _refresh_undo_budget_menu() -> void:
	var s: Dictionary = _bridge.undo_stats()
	var budget_mb := int(s.get("budget_bytes", 0)) / 1048576
	var step := int(s.get("step_bytes", 0))
	for i in UNDO_BUDGETS_MB.size():
		var mb: int = UNDO_BUDGETS_MB[i]
		_undo_budget_popup.set_item_checked(i, mb == budget_mb)
		if step > 0:
			var steps: int = maxi(1, mini(int(s.get("max_steps", 5)), (mb * 1048576) / step))
			_undo_budget_popup.set_item_text(i, "%d MB — %d step%s here" % [
				mb, steps, "" if steps == 1 else "s"])
		else:
			_undo_budget_popup.set_item_text(i, "%d MB" % mb)
	var clear_idx := _undo_budget_popup.get_item_index(ID_PREF_UNDO_CLEAR)
	if clear_idx >= 0:
		_undo_budget_popup.set_item_disabled(clear_idx, not _bridge.can_undo())
		_undo_budget_popup.set_item_tooltip(clear_idx,
			"Frees %s immediately. The next destructive edit starts a new stack." % _mb(int(s.get("bytes", 0))))

func _on_undo_budget(id: int) -> void:
	if id == ID_PREF_UNDO_CLEAR:
		_bridge.clear_undo()
		return
	if id >= 0 and id < UNDO_BUDGETS_MB.size():
		_bridge.set_undo_budget_mb(UNDO_BUDGETS_MB[id])
		_refresh_undo_budget_menu()

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
	_todo(p, "Save layout as…",
		"SS2.6 lists it beside Reset layout. Every ingredient already exists and none of them is collected: dock widths, each dock's collapsed state, the five region toggles above and the active domain are all live on DccShell, and DccSettings already persists machine-scoped state (storage roots, GPU selection, autosave). What is owed is a named-preset section over data the shell is already holding, plus the read side that applies one.")
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
	## §2.7 lists Documentation first. There is no in-app manual and the spec
	## names no URL, so inventing one would be the worst kind of gap-filling --
	## a row that opens something that may not exist. What *does* exist is the
	## repository the tooltip has always pointed at, and opening a real folder
	## is a real behaviour. Resolved once, at build time (the filesystem does
	## not move under a running app), and left disabled with the true reason
	## when it is not there -- which is every exported build, where `res://`
	## lives inside the `.pck` and the repository is not shipped beside it.
	var docs := _docs_dir()
	if docs == "":
		_todo(p, "Documentation",
			"No in-app documentation exists, and SS2.7 names no URL to open instead. The reference is the repository's own documents (README.md, DECISIONS.md, ARCHITECTURE.md and the scope documents), which are not present beside this build -- an exported build ships res:// inside the .pck and nothing else. Running from the repository enables this row.")
	else:
		_live(p, "Documentation", ID_HELP_DOCS)
		p.set_item_tooltip(p.item_count - 1,
			"Opens the repository's documents in the OS file manager: %s. There is no in-app manual, and SS2.7 names no URL -- this is the reference CLAUDE.md itself calls the reading entry point." % docs)
	## Was a `_todo` reading "No shortcut table yet." There is no table now
	## either, and that is the point: `ShortcutsDialog` walks these very menus
	## and reports what it finds, so the list cannot disagree with the app.
	_live(p, "Keyboard shortcuts…", ID_HELP_SHORTCUTS)
	_live(p, "Credits & academic principles", ID_HELP_CREDITS)
	## `PARITY_AUDIT.md` §5 item 6: the reference's ℹ️ `#genInfoBtn` --
	## dumps every generation parameter as plain text, a bug-report
	## affordance distinct from "Report an issue" below (which still has no
	## actual issue-filing route).
	_live(p, "Generation info…", ID_HELP_GEN_INFO)
	## `LOD debug ▸` used to be built here. §2.5 puts the chunk-debug overlay
	## under `Preferences ▸ Tiles & LOD`, and that is where it is now -- see the
	## note at that call site for why the reason it was put in Help stopped
	## being true.
	_todo(p, "Report an issue",
		"SS2.7 lists it and names no destination, which is the whole blocker: there is no issue tracker, support address or crash endpoint in this port to send to, and picking one would be inventing a route. The content is already solved -- Generation info... above dumps every generation parameter as plain text, and pairing that with the version and build string from About is exactly the body a report wants.")
	_live(p, "About", ID_HELP_ABOUT)
	p.id_pressed.connect(_on_help)

## §2.5's `Preferences ▸ Tiles & LOD ▸ Chunk debug overlay` -- the reference's
## own overlay, whose three toggles live on a `seg sm` segmented control
## (`lodDbgSeg`, reference line 1266) inside an "Atlas cache ▸ Chunk debug
## overlay" accordion.
##
## **Built under `Help ▸ LOD debug` and moved here 2026-08-30.** The reason it
## was in Help was that "this shell has no Atlas panel" -- it has one now, the
## `Atlas cache ▸` row directly above this call, so the reference's own
## accordion placement and §2.5's group are the same place again. Help keeps
## `Generation info…`, which is a dump rather than an overlay.
##
## The three rows keep the reference's own labels and order: Grid, Colors,
## Labels.
##
## Checkable rows rather than a segmented control: a `PopupMenu` has no
## segment, and a check mark is what "this overlay is on" looks like in a
## menu. They are independent toggles in the reference too -- three separate
## booleans, any combination legal -- so check items, not radio items.
func _build_lod_debug_submenu(p: PopupMenu) -> void:
	_lod_debug_popup = PopupMenu.new()
	_lod_debug_popup.name = "LodDebug"
	_lod_debug_popup.add_check_item("Grid", ID_LOD_DBG_GRID)
	_lod_debug_popup.add_check_item("Colors", ID_LOD_DBG_COLORS)
	_lod_debug_popup.add_check_item("Labels", ID_LOD_DBG_LABELS)
	## Reference: the overlay draws only under the tiled LOD view
	## (`drawLODChunkDebug` is called from `drawLODView`'s tail, and the
	## handler re-renders only `if(_lodOn)`). Said in the tooltip rather than
	## disabling the rows, because deep zoom is a camera state the user
	## reaches by scrolling, not a mode they switch on -- greying these out
	## would read as "not built".
	for i in 3:
		_lod_debug_popup.set_item_tooltip(i,
			"Chunk-debug overlay for the deep-zoom tile pyramid. Draws only while the tiled LOD view is up -- zoom in past the threshold to see it.")
	## §2.5's fourth element of this row: "`off · grid · colours` (#lodDbgSeg)
	## **+ tile borders**".
	##
	## **It is not a fourth chunk-debug toggle, and this file said it was.**
	## The reason that stood here until 2026-08-30 read "this would outline
	## each composited LOD TILE's own edge ... `_lod_sprite_rect()` already
	## returns exactly the rect one would need". That was inferred from the
	## spec's wording and is wrong on both halves. The reference settles it:
	##
	##   - The control is `#lodShowGrid`, labelled "Show tile borders on the
	##     map" (reference line 1281). Its handler sets `_showExportGrid`
	##     (line 13880) -- not one of `_lodGrid`/`_lodChunkCol`/`_lodLabels`.
	##   - Its draw is `drawExportTileGrid()` (line 9602): a dashed
	##     `refCols` x `refRows` split of the WHOLE map, where those two are
	##     the tile-export block's Cols/Rows fields (lines 1276-1277).
	##   - Its call site is `if(_showExportGrid && !_lodOn)` (line 8658) --
	##     it draws when the pyramid is DOWN, the opposite of the three
	##     toggles above, which draw only when it is up.
	##
	## So it is an export preview that happens to be filed under the same
	## accordion, and §2.5 lists it here because the reference PANEL groups
	## them, not because they are the same feature. It ships as a live row
	## against `ViewportHost.set_export_tile_grid()`, drawing the split
	## `DataManagerWindow` will actually export, and the mismatch is left in
	## this comment rather than corrected in the spec, which is the owner's
	## to change.
	_lod_debug_popup.add_separator()
	_lod_debug_popup.add_check_item("Show tile borders on the map", ID_LOD_TILE_BORDERS)
	_lod_debug_popup.set_item_tooltip(_lod_debug_popup.item_count - 1,
		"The export tile split (Data > Export > Maps > Tile grid), dashed over the map -- NOT the chunk overlay above. Draws while the deep-zoom pyramid is down, since the split is taken off the full-resolution grid.")
	_refresh_lod_debug_menu()
	_lod_debug_popup.id_pressed.connect(_on_lod_debug)
	_shell.style_popup(_lod_debug_popup)
	p.add_child(_lod_debug_popup)
	p.add_submenu_item("Chunk debug overlay", "LodDebug")

## Mirrors `ViewportHost`'s own three booleans onto the check marks. The
## viewport is the single source of truth -- this menu keeps no copy, which is
## the bug shape `PARITY_AUDIT.md` §23 F14 found in the three world-dialog
## getters (shell and engine each holding a copy, free to disagree after a
## load).
func _refresh_lod_debug_menu() -> void:
	if _lod_debug_popup == null or _host == null or _host.viewport == null:
		return
	_lod_debug_popup.set_item_checked(0, _host.viewport.lod_debug_enabled("grid"))
	_lod_debug_popup.set_item_checked(1, _host.viewport.lod_debug_enabled("colors"))
	_lod_debug_popup.set_item_checked(2, _host.viewport.lod_debug_enabled("labels"))
	## Index 4, past the separator the three toggles are followed by. Found by
	## id rather than by counting, so inserting a row above cannot silently
	## check the wrong one.
	var bi := _lod_debug_popup.get_item_index(ID_LOD_TILE_BORDERS)
	if bi >= 0 and _host.viewport.has_method("export_tile_grid_enabled"):
		_lod_debug_popup.set_item_checked(bi, _host.viewport.export_tile_grid_enabled())

## `#lodRefineBtn` -- bake just what is on screen.
##
## Every argument comes from `ViewportHost.visible_grid_rect()`, which is
## `_update_lod()`'s own camera math published; nothing here re-derives a
## rectangle the viewport would compute differently. The three refusals are
## distinct on purpose and each says which one it is, because "nothing
## happened" is the failure mode this repository keeps finding:
##
##   - no world, or the camera is off the map entirely -> the rect is not ok
##   - the pyramid is down (a fitted view) -> there is no level to refine to,
##     and baking level 0 would silently do something other than what the row
##     says
##   - no atlas directory -> `bake_visible` refuses, and its own message says so
func _refine_current_view() -> void:
	if _host == null or _host.viewport == null:
		return
	if not _host.viewport.has_method("visible_grid_rect"):
		_host.set_status("hint", "this build has no visible_grid_rect() — refine needs it", "text_dim")
		return
	var r: Dictionary = _host.viewport.visible_grid_rect()
	if not bool(r.get("ok", false)):
		_host.set_status("hint", "nothing to refine — no world is on screen", "text_dim")
		return
	if not bool(r.get("lod_active", false)):
		_host.set_status("hint", "nothing to refine at this zoom — the tile pyramid is not up; zoom in first", "text_dim")
		return
	var t0 := Time.get_ticks_msec()
	var res: Dictionary = _bridge.bake_visible(int(r.get("z", 0)),
		float(r["x0"]), float(r["y0"]), float(r["x1"]), float(r["y1"]))
	if not bool(res.get("ok", false)):
		_host.set_status("hint", "refine failed — %s" % String(res.get("error", "unknown")), "accent")
		return
	_host.set_status("hint", "refined LOD %d — %d chunk%s baked, %d already cached, %.1f s" % [
		int(r.get("z", 0)), int(res.get("baked", 0)),
		"" if int(res.get("baked", 0)) == 1 else "s",
		int(res.get("skipped", 0)), float(Time.get_ticks_msec() - t0) / 1000.0], "text_dim")

func _on_lod_debug(id: int) -> void:
	if _host == null or _host.viewport == null:
		return
	if id == ID_LOD_TILE_BORDERS:
		if not _host.viewport.has_method("set_export_tile_grid"):
			return
		_host.viewport.set_export_tile_grid(
			not _host.viewport.export_tile_grid_enabled())
		_refresh_lod_debug_menu()
		return
	var which := ""
	match id:
		ID_LOD_DBG_GRID: which = "grid"
		ID_LOD_DBG_COLORS: which = "colors"
		ID_LOD_DBG_LABELS: which = "labels"
		_: return
	_host.viewport.set_lod_debug(which, not _host.viewport.lod_debug_enabled(which))
	_refresh_lod_debug_menu()

## Where `Help ▸ Documentation` points, or `""` when nothing is there.
##
## `res://` is the Godot project directory, so the repository root is two
## levels up and `cartalith-native/` is one. The root is preferred because
## `CLAUDE.md` names its `README.md` as the reading entry point; presence of
## that file is also how this tells a source checkout from an exported build,
## rather than trusting `OS.has_feature("editor")` -- an exported build run
## from inside the repository has the documents too.
static func _docs_dir() -> String:
	var base := ProjectSettings.globalize_path("res://")
	for rel in ["../..", ".."]:
		var dir := base.path_join(rel).simplify_path()
		if FileAccess.file_exists(dir.path_join("README.md")):
			return dir
	return ""

func _on_help(id: int) -> void:
	match id:
		## `file://` rather than the bare path: `OS.shell_open` takes a URI on
		## every platform this ships to, and a bare Windows path with a drive
		## letter is not one.
		ID_HELP_DOCS: OS.shell_open("file://" + _docs_dir())
		ID_HELP_CREDITS: _host.open_credits()
		ID_HELP_SHORTCUTS: _host.open_shortcuts()
		ID_HELP_GEN_INFO: _host.open_gen_info()
		ID_HELP_ABOUT: _host.open_about()

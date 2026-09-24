extends Control
class_name PhoneMenu

## The phone menu, built to `design/Cartalith Android Phone.dc.html`
## ("ANDROID PHONE · FULL MENU", 412 x 892 dp, all five disclosure levels).
##
## This replaces `DccShell._build_phone_overflow()`, which reparented the
## *desktop* menu bar into a 220 px sheet -- `GUI_GAP_REGISTER.md` §15's four
## faults: nothing phone-scaled, desktop status chrome squeezing the row into a
## strip, no touch response, and ~41 items behind 15 hover-opened submenus.
##
## ## The 2026-09-05 ruling: five bespoke screens, not seven menu drills
##
## `LARGE_ITEM_RULINGS.md`, owner, 2026-09-05: *"Phone MORE — build the bespoke
## screens per `06-phone.md` §6.6. The shell's re-presentation of the desktop
## popups is superseded. Five purpose-built screens: Project, Civilization,
## Data, Simulation, Preferences."* Also recorded there, and repeated here so a
## reader of this file alone does not go looking: `docs/ANDROID_UI_SPEC.md`,
## which the paragraphs below still cite, **is not in this repository** -- it
## lives in the owner's design project. `06-phone.md` is what was ruled on, and
## it is what every screen below is built from.
##
## ## It still re-presents `menus.gd`; what changed is who chooses the order
##
## Before this pass a screen *was* a `PopupMenu`: the root listed the seven
## program menus and everything below it was that menu's own items in that
## menu's own order. Now the five ruled screens carry **§6.6's** rows in
## **§6.6's** order -- but each row still resolves to a real `PopupMenu` item
## (by menu name and item id, or by a submenu's node name) and still fires
## through `_activate()`, which emits `id_pressed`/`index_pressed` exactly as a
## pointer would. **No handler, callback or menu id is reimplemented here.** A
## row `menus.gd` deletes stops resolving and draws its absence
## (`_missing_row()`) rather than silently vanishing.
##
## `about_to_popup` is emitted before any popup is read, so the rows that
## rebuild themselves on open (Recent worlds, the autosave interval, the CPU
## thread pool state, the undo budget's step counts) are as live here as they
## are on desktop. It is emitted **once per menu per screen render**, and only
## for the menus that screen actually reads -- so entering a screen costs
## exactly what opening that menu costs on desktop, which is the budget
## `command_index.gd`'s own header explains must not be exceeded.
##
## ## The 2026-09-06 pass: nine more screens, and two that are not built
##
## §6.6 defines **seventeen** screens; the ruling built six. Eleven remained --
## not the twelve the backlog row says, counted off §6.6's own `_moreTitle()`
## table rather than off the row. Nine of the eleven are now built
## (`assets`, `assets-grid`, `asset-slot`, `travel`, `travel-item`,
## `landmarks`, `lm-fam`, `help`, `gestures`), each over state this build
## actually holds. **`data-tiles` followed on 2026-09-23** (`_fill_data_tiles()`),
## once XYZ/TMS/WMTS addressing existed to drive it. **`data-io` is not built**:
## it is a mock in the specification itself whose real equivalent the `data`
## screen already draws -- see the head of the sub-screen section below.
##
## ## Reachability is the constraint, and it is checked, not assumed
##
## §6.6's `help` screen states the rule this build is held to: *"The phone
## reorganises rather than truncates: every desktop function is reachable
## through MAP · GENERATE · PLAN · MORE."* Fifteen screens cannot carry 361
## menu rows one for one, so the coverage is explicit:
##
## | Program menu | Where it is reached on the phone |
## |---|---|
## | `File` | the `project` screen -- §6.6's four acts and its autosave block first, then **every remaining File row in File's own order** (`_rest_of()`) |
## | `Data` | the `data` screen -- the whole Data popup, whose own `IMPORT`/`EXPORT`/`SOURCES`/`VALIDATION` separators are already §6.6's bands |
## | `Preferences` | the `prefs` screen -- Theme and Units lifted to the top per §6.6, then the rest of the popup in its own order |
## | `Assets` | the `assets` screen -- §6.6's family rows over `as_family_slots()`, then **the whole Assets popup** (`_rest_of()`), which is what carries `Landmark types ▸`, the icon families and the slicer |
## | `Help` | the `help` screen -- this build's own runtime identity, the gesture drill, then **the whole Help popup** (`_rest_of()`) |
## | `Edit`, `Window` | the root's last band. §6.6's root table has no row for either, and dropping them would make Undo history, Find on map, Reset one stage, the dock toggles, Workspace, Open windows and Layouts unreachable on a handset |
##
## `_phonemore_reach_probe.gd` renders each built screen and reads back the
## strings it drew. It asserts (a) all eight of §6.6's root rows plus the two
## fallback rows are on the root, (b) every `MenuButton` on the live menu bar is
## reached by one of them, and (c) every **top-level** row of File, Data and
## Preferences is drawn on its screen -- counting a submenu as drawn when its
## own text appears OR, for one expanded into chips, when a child's does.
## Deeper levels are `_popup_row()`'s ordinary drill and are not re-walked.
##
## ## Row types
##
## §6.6's own vocabulary (`head`, `nav`, `act`, `tog`, `seg`, `range`, `read`,
## `info`), mapped onto what a `PopupMenu` item already is:
##
## | §6.6 | Here |
## |---|---|
## | `head` | `_band()` -- also what a labelled `add_separator()` renders as |
## | `nav` | `_row()` with a chevron: another screen, a program menu, or a submenu |
## | `act` | a plain item, fired by `_activate()` |
## | `tog` | a `p.is_item_checkable()` item, drawn with `_switch()` |
## | `seg` | a submenu of radio items, **expanded inline as chips** -- see `_expandable()` |
## | `range` | only where a continuous quantity really exists: the `sim` screen's Year, over `DccShell.TL_YEAR_MIN..TL_YEAR_MAX`. Everywhere §6.6 draws a range over what this port models as a fixed set (CPU threads, undo budget, zoom levels) the set is drawn, because inventing intermediate values would offer settings the engine has no call for |
## | `read` | a `menus.gd::_readout()` row, or a `DccShell` status slot |
## | `info` | a `menus.gd::_signpost()` row, or a stated absence |
##
## ## Theme
##
## Everything is written from `DccTheme.c()` tokens through `font_color` and the
## `panel`/`normal`/`hover` styleboxes -- the exact override names
## `DccShell._recolor_subtree()` walks -- so a dark/light switch repaints this
## surface with no second code path. There is one deliberate literal: the
## sheet's dim scrim, which is `Color(c("bg"), 0.72)`, an alpha derivative
## `DccTheme.remap()` resolves by its RGB half.

## A step on the drill path. `popup` is null only for the root.
##
## `level` is the disclosure level, and it is what decides the presentation:
## 4 is a sheet, everything else is a screen.
class _Step:
	var popup: PopupMenu
	## One of `SCREEN_IDS` when this step is a bespoke §6.6 screen, `""` when it
	## is a popup being re-presented. Exactly one of `popup` / `screen` is set;
	## the root is the `"more"` screen and so has neither a popup nor a parent.
	var screen: String
	## The one thing a screen is *about*, when a screen id alone does not say:
	## which asset family `assets-grid` is showing, which slot uid `asset-slot`
	## is inspecting, which `kind|id` pair `travel-item` opened, which landmark
	## family `lm-fam` drew. Empty for every screen that needs no argument.
	##
	## A plain `String` rather than a Dictionary because every consumer wants
	## exactly one identifier and the one pair (`travel-item`) splits on `|` --
	## a two-field payload is not worth a second type, and a String keeps the
	## whole navigation stack printable, which is what the probes read.
	var arg: String
	var title: String
	var trail: String
	var level: int

	func _init(p: PopupMenu, t: String, tr: String, lv: int, scr: String = "",
			ar: String = "") -> void:
		popup = p
		title = t
		trail = tr
		level = lv
		screen = scr
		arg = ar

	func is_sheet() -> bool:
		return level == 4

## §6.6's `_moreTitle()` table -- the title and subtitle for every screen this
## file builds, quoted from the spec's own two columns.
##
## Two subtitles deviate, and both are the spec being stale rather than this
## file taking a liberty:
##
##   - **`data`** is `import · export · sources · conversion · validation` in
##     §6.6. `Data ▸ Conversion` was **removed by owner decision, 2026-08-20**
##     (the root `CLAUDE.md` records it as the one case where a canvas is the
##     stale party), and `menus.gd::_data()` accordingly builds four bands, not
##     five. Naming a fifth in the subtitle would advertise a band the screen
##     cannot contain.
##   - **`root`** is `program · data · preferences` there; `MORE`'s own bottom
##     bar cell already says MORE, so the subtitle carries the three §6.6 words
##     unchanged.
##
## `civ`'s subtitle is §6.6's verbatim, **including "POI"**, which this port has
## no tool for -- see `_fill_civ()`, which draws that absence as a row rather
## than quietly shortening the promise.
const SCREEN_TITLES: Dictionary = {
	"more": ["More", "program · data · preferences"],
	"project": ["Project", "files · autosave · storage"],
	"civ": ["Civilization", "settlement · POI · way tools"],
	"data": ["Data manager", "import · export · sources · validation"],
	"sim": ["Simulation", "timeline · layers"],
	"prefs": ["Preferences", "application · performance · graphics"],
	## -- The 2026-09-06 pass: §6.6's sub-screens, minus the two that are mock --
	##
	## §6.6 defines seventeen screens; the ruling built six of them (`root`
	## through `prefs` above). **Eleven remained, not the twelve the backlog row
	## and this batch's brief both say** -- counted off §6.6's own
	## `_moreTitle()` table, which is the list directly above: `data-tiles`,
	## `data-io`, `assets`, `assets-grid`, `asset-slot`, `travel`,
	## `travel-item`, `landmarks`, `lm-fam`, `help`, `gestures`. Nine landed
	## 2026-09-06 and `data-tiles` 2026-09-23; `data-io` is **not built**, and
	## the reason is at the head of the sub-screen section rather than in a
	## title nobody would reach.
	##
	## Two subtitles deviate from §6.6 and both are this port having more than
	## the prototype did rather than less:
	##
	##   - **`assets`** is `24 families · 72 of 113 filled` there -- a written
	##     count over a family list the prototype invented. This build's
	##     families are `AssetLibraryWindow.FAMILIES` (eight, see that constant's
	##     `FAMILIES_NOTE` for why the canvas's 24 do not exist in any Rust type)
	##     and its fill counts are real, so the subtitle names the source and the
	##     screen counts.
	##   - **`help`** is `Cartalith Mobile 0.9` there. There is no product
	##     version, build number or "Cartalith Mobile" in this port -- see
	##     `_fill_help()`, which draws what this build CAN answer for itself.
	"assets": ["Asset library", "families · slots · fill"],
	"assets-grid": ["Asset library", ""],
	"asset-slot": ["Slot inspector", ""],
	"travel": ["Travel library", "classifications & constraints — feeds the planner"],
	"travel-item": ["Travel library", ""],
	"landmarks": ["Landmarks", "caps · spacing · one run"],
	"lm-fam": ["", "zero = off · a cap is a ceiling, not a quota"],
	"help": ["Help & about", "version · engine · gestures · credits"],
	"gestures": ["Gesture reference", "the touch vocabulary this build has"],
	## §6.6: `EXPORT ▸ MAPS ▸ TILE PYRAMID` / `leaflet · XYZ · baked atlas
	## L0–L3`. The subtitle drops "baked atlas": `slippy_export_tiles`
	## synthesises every tile rather than reading the atlas
	## (`cartalith_engine::slippy_export`'s module doc says why).
	"data-tiles": ["Tile pyramid", "leaflet · XYZ · TMS · WMTS · retina"],
}

## §6.6's `root` table, in its order, with its glyphs and its sub text.
##
## `t` is what the row goes to: `screen` a bespoke screen id, or `menu` a
## program menu re-presented whole. **Every row is a `screen` today**; `menu`
## is kept because `_named_in_root()` reads it and because a future root row
## may want a plain drill, and a third kind (`call`, straight to a window)
## existed until 2026-09-06 and was removed with its last row -- the Travel
## library window is now opened from inside the `travel` screen instead.
##
## **All eight are bespoke screens as of 2026-09-06.** Until then three were
## not: the 2026-09-05 ruling named five screens, and Assets, Travel and Help
## drilled the real popup or opened the real window instead. Each of those three
## turned out to have real state behind it -- `as_family_slots()` reports every
## family's slots with true fill flags, `tl_list()` reports the four travel
## kinds, and the build can answer its own identity -- so the three rows now
## reach screens that show it. **Nothing stopped being reachable:**
## `_fill_assets()` and `_fill_help()` both end in `_rest_of()` over the same
## program popup the row used to drill, so every Assets and Help row is still
## drawn, in the popup's own order, under §6.6's rows. Travel is not a program
## menu at all -- `Data ▸ Travel library…` is its menu row, and the `data`
## screen draws that -- so it loses nothing either, and its screen ends with an
## act that opens the same window the row used to.
const ROOT_ROWS: Array = [
	{"t": "screen", "id": "project", "glyph": "⧉", "label": "Project",
		"sub": "save · recent · storage"},
	{"t": "screen", "id": "civ", "glyph": "◍", "label": "Civilization",
		"sub": "settlement · POI · way tools"},
	{"t": "screen", "id": "data", "glyph": "⇅", "label": "Data manager",
		"sub": "import · export · sources · validation"},
	{"t": "screen", "id": "assets", "glyph": "▦", "label": "Asset library",
		"sub": "families · slots · packs · landmark types"},
	{"t": "screen", "id": "travel", "glyph": "≋", "label": "Travel library",
		"sub": "animals · vehicles · vessels · parties"},
	{"t": "screen", "id": "sim", "glyph": "◷", "label": "Simulation",
		"sub": ""},
	{"t": "screen", "id": "prefs", "glyph": "⚙", "label": "Preferences",
		"sub": "theme · units · performance · graphics"},
	{"t": "screen", "id": "help", "glyph": "?", "label": "Help & about",
		"sub": "documentation · shortcuts · credits · about"},
]

## The two program menus §6.6's root table has no row for.
##
## Dropping them is what the ruling does **not** authorise: `Edit` owns Undo
## history, Delete, Deselect, Find on map, Reset generation parameters and
## Reset one stage; `Window` owns the four dock/bar toggles, the diagnostics
## overlay, Workspace, Open windows, Reset layout and Layouts. None of those is
## carried by any of the five bespoke screens, and the phone has no menu bar --
## so without this band they would be reachable nowhere at all. §6.6's own
## `help` screen is the authority for keeping them: *"The phone reorganises
## rather than truncates: every desktop function is reachable through MAP ·
## GENERATE · PLAN · MORE."*
##
## Appended **after** §6.6's last root row (the `STATUS` block) rather than
## inserted among the eight, so the spec's own order is intact above it.
const ROOT_REST: Array = ["Edit", "Window"]

## Submenus `_expandable()` must never expand inline, whatever their shape.
##
## Expanding a submenu means emitting its `about_to_popup`, and these two
## handlers are not observers:
##
##   - `AtlasCache` -> `menus.gd::_refresh_atlas_cache_menu()`, which calls
##     `_enforce_atlas_cap()` and **evicts baked chunks**.
##   - `GpuDevices` -> `_on_gpu_devices_about_to_popup()`, which enumerates
##     `wgpu` adapters -- the cost `menus.gd::_build_gpu_devices_menu()`
##     documents as the crash it was restructured to put behind a first open.
##
## `command_index.gd`'s header records the same hazard for the same two
## handlers and the same reason. Both are excluded **by name** even though
## neither satisfies `_expandable()` today (one nests a submenu, the other has
## no radio items): that is a fact about their current shape, not a property
## anyone maintaining `menus.gd` has agreed to preserve.
const NO_EXPAND: Array = ["AtlasCache", "GpuDevices"]

## The status readouts, in the order they read best as a list. Keys are
## `DccShell`'s own status slots; `set_status()` keeps them live and this
## re-reads them every time the root screen is drawn.
##
## Two changes this pass, both from reading the shipped root screen rather than
## the code:
##
##   - **"Pass" is gone.** The row read `Pass — no world`. "Pass" is what the
##     engine calls one stage run; it is not a word this app has ever shown a
##     user, and nothing in either spec uses it. The *slot* is worth keeping --
##     it is the only report that a generation happened and what it cost -- so
##     it is relabelled **Generator**, which is the phone's own name for that
##     pipeline (`PHONE_TABS`' second tab is GENERATE), and every value
##     `app.gd` writes into it then reads as a state of one: `no world`,
##     `generating…`, `generated · 1.4s`, `loaded`.
##   - **`hint` is new here, and first.** It is the only slot carrying an
##     instruction rather than a measurement -- `app.gd` writes "File ▸ New
##     world… to begin" the moment the app opens with no world -- and on a
##     phone the status bar it normally lives in is parked in a hidden host, so
##     it was reaching nobody at all. That is the row a user can act on, which
##     is what the em-dash row it replaces was not. Drawn wrapped rather than
##     as a right-hand readout, because it is a sentence (`_note_row()`).
##
## `World —` was not a labelling fault and is not fixed here: see
## `_status_rows()`, which stopped drawing rows whose value is the shell's own
## em-dash placeholder for "nothing yet".
const STATUS_ROWS: Array = [
	["hint", "Next"],
	["top_world", "World"],
	["top_res", "Resolution"],
	["pass", "Generator"],
	["stale", "Stale"],
	["autosave", "Autosave"],
	["atlas", "Atlas"],
	["top_cpu", "CPU"],
	["top_gpu", "GPU"],
	["top_mem", "Memory"],
]

var _shell: DccShell
var _scale := 1.0
var _stack: Array = []  ## of `_Step`; untyped because a typed `Array[_Step]`
	## over an inner class is not portable across GDScript versions.

var _screen: PanelContainer
var _screen_head_title: Label
var _screen_head_trail: Label
var _screen_head_meta: Label   ## The root's right-hand `ELDRA · 1.6 GB`.
var _screen_back: Button
var _screen_close: Button
var _screen_scroll: ScrollContainer
var _screen_body: VBoxContainer

var _sheet_scrim: ColorRect
var _sheet: PanelContainer
var _sheet_head_title: Label
var _sheet_head_trail: Label
var _sheet_scroll: ScrollContainer
var _sheet_body: VBoxContainer

# -- Geometry ----------------------------------------------------------------
#
# The same two helpers `DccShell` uses for its own phone chrome, over the same
# `phone_scale()`. Duplicated as three lines rather than reached for across the
# file boundary because `_pscale`/`_ptap` are private there and this is the only
# thing this file needs from them.

func _ps(px: float) -> int:
	return maxi(1, int(round(px * _scale)))

func _pt(px: float) -> int:
	return maxi(DccTheme.PHONE_TAP_MIN, _ps(px))

# -- Build --------------------------------------------------------------------

func setup(shell: DccShell) -> void:
	_shell = shell
	_scale = shell.phone_scale()
	## `set_anchors_and_offsets_preset`, not `set_anchors_preset`: this node is
	## already in the tree under a parent that has its real size, and the
	## anchors-only call *preserves the current rect* by writing compensating
	## offsets -- which for a freshly-added control means offsets of
	## (0, 0, -width, -height) and a permanently zero-sized overlay. Measured,
	## not guessed: the first run of the capture harness drew the whole menu as
	## a 181x57 box in the top-left corner.
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	## **Always `IGNORE`, open or closed.** This node is full-rect and its
	## children are inset (`apply_insets()`), so a `STOP` here -- which is what
	## `open()` used to set -- made the *whole screen* pick, including the strip
	## below the screen where the bottom nav lives. Found on the handset: with
	## MORE open, tapping MORE again did nothing and tapping WORLD did nothing,
	## because neither tap ever reached the bar. Blocking is the job of `_screen`
	## and `_sheet_scrim`, which cover exactly the rect the menu occupies.
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	visible = false

	_screen = _build_screen()
	add_child(_screen)

	## Canvas "04 · L4 SHEET" draws the region above the sheet as
	## `rgba(8,9,9,.72)` -- the screen it was opened from, veiled, not removed.
	_sheet_scrim = ColorRect.new()
	_sheet_scrim.color = Color(DccTheme.c("bg"), 0.72)
	_sheet_scrim.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_sheet_scrim.mouse_filter = Control.MOUSE_FILTER_STOP
	_sheet_scrim.gui_input.connect(_on_scrim_input)
	_sheet_scrim.visible = false
	add_child(_sheet_scrim)

	_sheet = _build_sheet()
	add_child(_sheet)

func _build_screen() -> PanelContainer:
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DccTheme.panel("bg"))
	panel.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	## Explicit, because this is what stops a tap on the menu reaching the map
	## now that the node above it is `IGNORE` -- a `Container` defaults to `PASS`,
	## which happens to block too, but relying on a default for a thing that
	## matters is how the fault above got in.
	panel.mouse_filter = Control.MOUSE_FILTER_STOP

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	panel.add_child(col)

	## Two headers in one row, switched by level in `_render()`:
	##
	##   - **L2, the root** is canvas "07 More": `height:56px;padding:0 16px;
	##     gap:12px`, a `500 12px Plex/.22em` title taking the full width and a
	##     `10px Plex #6f7478` readout on the right (`ELDRA · 1.6 GB`). **No
	##     back button and no close.** This screen used to carry a `✕` on each
	##     side of its title -- two buttons for one action, which is what a
	##     menu-by-menu walk against this canvas found first. The bottom nav is
	##     visible beneath the menu, so tapping any tab (MORE included, which is
	##     a toggle now) leaves; so does system back.
	##   - **L3+** is canvas "02"/"03": `←` in a 40 dp cell, title over a
	##     breadcrumb, and a slot on the right the canvas fills with `⋮`. That
	##     `⋮` is a per-screen overflow this shell has nothing to put behind, so
	##     the slot carries a close instead -- `←` leaves one level, `✕` leaves
	##     the menu, and neither is decorative.
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", _ps(12))
	head.custom_minimum_size.y = _pt(DccTheme.H_PHONE_APP_BAR)

	_screen_back = _icon_button(DccIcons.SYMBOLS["collapse"], "Back", _go_back_pressed)
	head.add_child(_screen_back)

	var titles := VBoxContainer.new()
	titles.add_theme_constant_override("separation", _ps(2))
	titles.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	titles.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	titles.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_screen_head_title = DccTheme.mono_label("", "text_bright", _ps(12), 2, true)
	_screen_head_trail = DccTheme.mono_label("", "text_faint", _ps(10), 0)
	titles.add_child(_screen_head_title)
	titles.add_child(_screen_head_trail)
	head.add_child(titles)

	## The root's right-hand readout, in the canvas's own position and type.
	_screen_head_meta = DccTheme.mono_label("", "text_faint", _ps(10), 0)
	_screen_head_meta.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	_screen_head_meta.mouse_filter = Control.MOUSE_FILTER_IGNORE
	head.add_child(_screen_head_meta)

	_screen_close = _icon_button(DccIcons.SYMBOLS["cross"], "Close menu", close)
	head.add_child(_screen_close)

	var hp := MarginContainer.new()
	hp.add_theme_constant_override("margin_left", _ps(16))
	hp.add_theme_constant_override("margin_right", _ps(6))
	hp.add_child(head)
	col.add_child(hp)
	col.add_child(DccTheme.rule())

	_screen_scroll = ScrollContainer.new()
	_screen_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_screen_scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	_screen_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_screen_scroll.add_theme_stylebox_override("panel", DccTheme.empty())
	_screen_body = VBoxContainer.new()
	_screen_body.add_theme_constant_override("separation", 0)
	_screen_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_screen_scroll.add_child(_screen_body)
	col.add_child(_screen_scroll)
	return panel

## Canvas "SHEETS STOP AT 60% HEIGHT".
##
## **`open_sheet()` is now its only caller.** Until the 2026-09-05 ruling this
## also drew every submenu one level down (L4 in the old grammar); §6.6 makes
## the MORE tab a single navigation stack, so `SCREEN_LEVEL` sends those to the
## screen instead. What is left here is the map context menu, which really is an
## overlay over an unrelated surface and is what the scrim and the 60% cap are
## for.
##
## Expressed as anchors (top 0.4) rather
## than a pixel offset computed from `size`, because at build time this node has
## no size yet and a rotation changes it afterwards -- an anchor is correct in
## both cases with nothing to re-apply.
func _build_sheet() -> PanelContainer:
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DccTheme.panel("raised", {"top": 1}))
	panel.anchor_left = 0.0
	panel.anchor_right = 1.0
	panel.anchor_top = 0.4
	panel.anchor_bottom = 1.0
	panel.visible = false

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	panel.add_child(col)

	var handle_wrap := Control.new()
	handle_wrap.custom_minimum_size.y = _ps(18)
	handle_wrap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var handle := ColorRect.new()
	## Token-derived, never a literal white: see the tool sheet's own handle in
	## `dcc_shell.gd` for the light-theme reason.
	handle.color = Color(DccTheme.c("text_ghost"), 0.55)
	var hw := _ps(34)
	var hh := _ps(4)
	handle.set_anchors_preset(Control.PRESET_CENTER)
	handle.size = Vector2(hw, hh)
	handle.position = Vector2(-hw / 2.0, -hh / 2.0)
	handle.mouse_filter = Control.MOUSE_FILTER_IGNORE
	handle_wrap.add_child(handle)
	col.add_child(handle_wrap)

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", _ps(12))
	head.custom_minimum_size.y = _pt(48)
	var titles := VBoxContainer.new()
	titles.add_theme_constant_override("separation", _ps(2))
	titles.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	titles.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	titles.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_sheet_head_title = DccTheme.mono_label("", "text_bright", _ps(12), 3, true)
	_sheet_head_trail = DccTheme.mono_label("", "text_faint", _ps(9), 1)
	titles.add_child(_sheet_head_title)
	titles.add_child(_sheet_head_trail)
	head.add_child(titles)
	head.add_child(_icon_button(DccIcons.SYMBOLS["cross"], "Close", _go_back_pressed))

	var hp := MarginContainer.new()
	hp.add_theme_constant_override("margin_left", _ps(16))
	hp.add_theme_constant_override("margin_right", _ps(6))
	hp.add_child(head)
	col.add_child(hp)
	col.add_child(DccTheme.rule())

	_sheet_scroll = ScrollContainer.new()
	_sheet_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_sheet_scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	_sheet_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_sheet_scroll.add_theme_stylebox_override("panel", DccTheme.empty())
	_sheet_body = VBoxContainer.new()
	_sheet_body.add_theme_constant_override("separation", 0)
	_sheet_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_sheet_scroll.add_child(_sheet_body)
	col.add_child(_sheet_scroll)
	return panel

## Canvas TARGETS: "44 dp icon buttons".
func _icon_button(glyph: String, tip: String, on_press: Callable) -> Button:
	var b := Button.new()
	b.text = glyph
	## "normal" below is genuinely empty (correct, glyph-only at rest) but
	## "hover" and "pressed" are real fills -- flat suppresses all three
	## alike, so this 44 dp target had no touch/press feedback whatever state
	## it was in. Every phone-menu icon button shares this one factory.
	b.flat = false
	b.focus_mode = Control.FOCUS_NONE
	b.tooltip_text = tip
	b.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	b.custom_minimum_size = Vector2(_pt(44), _pt(44))
	b.add_theme_font_size_override("font_size", _ps(15))
	b.add_theme_font_override("font", DccTheme.mono(0))
	b.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	b.add_theme_stylebox_override("normal", DccTheme.empty())
	b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	b.add_theme_stylebox_override("pressed", DccTheme.active_row(false))
	b.pressed.connect(on_press)
	return b

# -- Insets -------------------------------------------------------------------

## Called by `DccShell` on build and on every `phone_insets_changed`. The menu
## sits *over* the app bar (the canvas's L2/L3 screens carry their own header
## and replace it), but never over the status safe area, the landscape side
## safe area, or the bottom gesture inset.
func apply_insets(top: float, left: float, bottom: float) -> void:
	if _screen == null:
		return
	_screen.offset_left = left
	_screen.offset_top = top
	_screen.offset_right = 0
	_screen.offset_bottom = -bottom
	_sheet.offset_left = left
	_sheet.offset_right = 0
	_sheet.offset_bottom = -bottom
	_sheet_scrim.offset_left = left
	_sheet_scrim.offset_top = top
	_sheet_scrim.offset_right = 0
	_sheet_scrim.offset_bottom = -bottom

# -- Navigation ---------------------------------------------------------------

func is_open() -> bool:
	return visible

func open() -> void:
	_stack.clear()
	## L1 is the bottom bar itself, so the first screen this file owns is L2.
	## `MORE` is the canvas's own word for this screen and for the bar cell that
	## opens it; it read `MENU` until the 412 migration.
	_stack.append(_Step.new(null, _screen_title("more"), "", 2, "more"))
	visible = true
	_render()

## §6.6's `_moreTitle()` is a lookup for most screens and a **template** for
## four of them (`{assetFam}`, `{selected entry}`, `{family label}`,
## `{assetFam} · slot {n+1}`). Those four resolve against the step's `arg`, and
## every one of them resolves against LIVE state rather than a stored copy: an
## asset family's title comes from `AssetLibraryWindow.FAMILIES`, a travel
## entry's name from `tl_get()`, a landmark family's from the engine's own key.
##
## An unresolvable arg falls back to the arg itself rather than to a plausible
## name -- a header reading `poi` is a header saying "this is the poi family and
## nothing pretty was found for it", which is true; one reading `Points of
## interest` for a family that no longer exists would not be.
func _screen_title(id: String, arg: String = "") -> String:
	if id == "lm-fam" and arg != "":
		return CivilizationWorkspace._lm_pretty(arg).capitalize()
	var row: Array = SCREEN_TITLES.get(id, [])
	return String(row[0]) if row.size() == 2 and String(row[0]) != "" else id

func _screen_sub(id: String, arg: String = "") -> String:
	match id:
		"assets-grid":
			return "family · %s" % _asset_family_title(arg)
		"asset-slot":
			return _slot_subtitle(arg)
		"travel-item":
			return _travel_entry_name(arg)
	var row: Array = SCREEN_TITLES.get(id, [])
	return String(row[1]) if row.size() == 2 else ""

## Phone sheet header support: `dcc_shell.gd::_refresh_phone_sheet_header()`'s
## own 2026-09-13 doc comment named the same gap for MORE that PLAN had --
## "`phone_menu.gd`'s `_stack`/`_screen_title()` are both private with no
## public getter" -- and this closes it, the same shape as
## `journey_planner_view.gd::phone_header_info()` added the same pass. `.title`
## on the top step is already the live, resolved string `_push_screen()` wrote
## it with (`_screen_title(id, arg)` at push time, not a static table read
## fresh), so this is a read, not a second computation.
##
## **Built and NOT wired into that header, and verified why rather than left
## unstated:** `_screen` -- the L2+ page this menu draws while `is_open()` --
## is a `PanelContainer` on `DccTheme.panel("bg")` (opaque, not a wash) at
## `PRESET_FULL_RECT` (`_build_screen()`'s own construction above), and this
## whole node is added to `_phone_root` in `dcc_shell.gd::_build_phone_shell()`
## strictly AFTER `chrome` (the tool sheet's own parent, added a few dozen
## lines earlier in that same function) -- ordinary Godot canvas ordering
## then draws this menu's opaque full screen OVER the sheet header on every
## frame it is open. A caller could point `_phone_sheet_title.text` at this
## and it would be correct and permanently invisible in the same edit. Left
## available rather than wired to a dead consumer, for whatever later change
## (a peek-height MORE, or a header this menu draws itself) could actually
## show it.
func current_page_title() -> String:
	if _stack.is_empty():
		return _screen_title("more")
	return (_stack[_stack.size() - 1] as _Step).title

## The canvas's `ELDRA · 1.6 GB` -- the world's name beside what it costs.
## Read off the live status slots rather than stored: `top_world` is written by
## `app.gd::_world_pill_text()` as `"<name> · <seed>"` when the world has a
## generated name and bare `"<seed>"` when it does not (a save loaded from
## before `world.name` existed -- `WorldGen::get_world_name()`'s own note), so
## the name is the head ONLY when a `" · "` separator is actually present; the
## bare-seed case has no name half to take, and `split(" · ")[0]` on a string
## with no separator returns the whole string, which would misread a seed as
## a name if taken unconditionally. `top_mem` is the figure
## `app.gd::_wire_status` writes into `top_mem`. (**This said "the
## Performance window's own figure" until 2026-09-06; that window is deleted and
## the attribution was wrong anyway — `app.gd:739`'s `set_status("top_mem", ...)`
## has always been the writer.**) Either half may be empty before a world
## exists, and an empty readout is drawn as nothing rather than as `· `.
func _root_meta() -> String:
	var parts := PackedStringArray()
	var world := _slot("top_world")
	if world != "":
		var world_halves := world.split(" · ")
		if world_halves.size() > 1:
			parts.append(world_halves[0])
	var mem := _slot("top_mem")
	if mem != "":
		parts.append(mem)
	return " · ".join(parts)

## A status slot's text, with the shell's own em-dash placeholder read as
## "nothing yet". Before a world exists `top_world` is `–`, and a header that
## reads `–` is worse than one that reads nothing.
func _slot(key: String) -> String:
	var t := _shell.status_slot_text(key).strip_edges()
	return "" if t == "" or t == "–" or t == "—" or t == "-" else t

## Present ONE arbitrary `PopupMenu` as an L4 sheet with nothing behind it but
## the veiled map -- the map context menu's phone form
## (`civilization_workspace.gd`'s `_ctx_menu`, opened by a press-and-hold
## rather than a right click).
##
## Everything the canvas asks of a sheet is already built above and none of it
## is re-implemented here: the 60%-height cap, the grab handle, the scrim that
## dismisses on tap, 52 dp rows, the disabled-row reason drawn as a second
## line, and `_activate()`'s two signals -- so the caller builds its menu
## exactly as it does for desktop and this re-presents it, the same contract
## the rest of this file has with `menus.gd`. A context menu with a submenu
## would drill to a full screen from here, which is `_render()`'s existing
## behaviour and needs nothing special.
##
## Deliberately NOT `PopupMenu.popup()` on a phone: a stock popup draws
## ~20 px rows sized for a pointer, and one opened at a finger near the screen
## edge is clipped by the window rather than nudged into it.
func open_sheet(popup: PopupMenu, title: String, trail: String) -> void:
	_stack.clear()
	_stack.append(_Step.new(popup, title, trail, 4))
	visible = true
	_render()

func close() -> void:
	visible = false
	_stack.clear()

## Canvas BACK: "System back leaves a sheet, then the L2 screen, then the
## viewport -- never the app." Returns true when it consumed the gesture, so
## `DccShell._notification()` knows whether to let the request fall through.
func go_back() -> bool:
	if not visible:
		return false
	if _stack.size() <= 1:
		close()
		return true
	_stack.pop_back()
	_render()
	return true

func _go_back_pressed() -> void:
	go_back()

func _on_scrim_input(ev: InputEvent) -> void:
	var tapped: bool = (ev is InputEventMouseButton and ev.pressed) \
		or (ev is InputEventScreenTouch and ev.pressed)
	if tapped:
		go_back()

## Push one level. `popup` is read (and `about_to_popup` fired) at render time,
## not here, so a submenu that rebuilds itself on open is rebuilt on every visit
## rather than once.
func _push(popup: PopupMenu, title: String, level: int) -> void:
	var trail := PackedStringArray()
	for s in _stack:
		trail.append(s.title)
	## The breadcrumb is where the user is, not how deep the code thinks it is.
	## This used to append " · L%d" -- so the File screen's header read
	## "More · L3", printing an internal disclosure level to somebody who has no
	## idea the levels exist. `level` still drives layout; it just stopped
	## being copy.
	_stack.append(_Step.new(popup, title, " · ".join(trail), level))
	_render()

## Push one of §6.6's bespoke screens. Same stack, same back button, same
## breadcrumb -- §6.6 calls it `moreStack` and gives it exactly this behaviour:
## *"A navigation stack (`moreStack`, starts `['root']`). The back button pops
## one level."*
##
## Level 3, so it draws as a **screen** and not as the 60%-cap sheet: §5.1 makes
## the sheet the container for the whole MORE tab, and §6.6's sub-screens are
## pages *inside* it, not a second surface stacked over it. `is_sheet()` (level
## 4) stays what `open_sheet()` uses for the map context menu, which is the one
## thing in this file that really is an overlay over an unrelated surface.
func _push_screen(id: String, arg: String = "") -> void:
	var trail := PackedStringArray()
	for s in _stack:
		trail.append(s.title)
	_stack.append(_Step.new(null, _screen_title(id, arg), " · ".join(trail),
		SCREEN_LEVEL, id, arg))
	_render()

# -- Render -------------------------------------------------------------------

func _render() -> void:
	if _stack.is_empty():
		return
	var top: _Step = _stack[_stack.size() - 1]

	## The deepest *screen* step. When the top step is a sheet this is the step
	## behind it, which is exactly what the canvas veils rather than replaces.
	##
	## `open_sheet()` pushes a sheet as the *base* step, so there may be no
	## screen at all -- and then the thing behind the scrim is the map, which is
	## precisely what a context menu should veil. Guarded rather than defaulted
	## to `_stack[0]`, which in that case is the sheet itself and would draw the
	## sheet's own rows full-screen underneath it.
	var screen_step: _Step = null
	for s in _stack:
		if not s.is_sheet():
			screen_step = s

	_screen.visible = screen_step != null
	if screen_step != null:
		var root: bool = screen_step.level == 2
		_screen_head_title.text = screen_step.title.to_upper()
		## §6.6's `_moreTitle()` gives every bespoke screen its OWN subtitle
		## ("files · autosave · storage"), which is what §5.4's subtitle slot
		## draws. A breadcrumb is the fallback for a re-presented popup, where
		## no such subtitle exists and the trail is the only orientation there
		## is. The root keeps neither: its right-hand readout is §6.6's `root`
		## header and a subtitle under `MORE` would push it off.
		var sub := _screen_sub(screen_step.screen, screen_step.arg) \
			if screen_step.screen != "" else screen_step.trail
		_screen_head_trail.text = "" if root else sub
		_screen_head_trail.visible = not root and sub != ""
		## Canvas "07 More": the root's header is a title and a readout, with no
		## button on either side. Everything deeper is "02"/"03": `←` and a
		## right-hand slot.
		_screen_back.visible = not root
		_screen_close.visible = not root
		_screen_head_meta.visible = root
		if root:
			_screen_head_meta.text = _root_meta()
		_fill(_screen_body, screen_step)
		_screen_scroll.scroll_vertical = 0

	var sheet_open: bool = top.is_sheet()
	_sheet_scrim.visible = sheet_open
	_sheet.visible = sheet_open
	if sheet_open:
		_sheet_head_title.text = top.title.to_upper()
		_sheet_head_trail.text = top.trail
		_fill(_sheet_body, top)
		_sheet_scroll.scroll_vertical = 0

func _clear(body: VBoxContainer) -> void:
	for child in body.get_children():
		body.remove_child(child)
		child.queue_free()

func _fill(body: VBoxContainer, step: _Step) -> void:
	_clear(body)
	_refreshed.clear()
	if step.screen != "":
		_fill_screen(body, step.screen, step.arg)
	elif step.popup == null:
		## Unreachable through `open()` / `_push_screen()` / `_push()`, all of
		## which set one of the two. Drawn rather than returning silently, for
		## the same reason `_missing_row()` exists: an empty screen with a
		## working back button is indistinguishable from a screen whose rows all
		## went away.
		body.add_child(_missing_row("This screen has no content source.",
			"Neither a bespoke screen id nor a PopupMenu reached _fill()."))
	else:
		_fill_popup(body, step)
	## Canvas "02"/"03"/"07" all close their list with a hairline and a padded
	## tail so the last row is not flush against the gesture inset.
	body.add_child(DccTheme.rule())
	var tail := Control.new()
	tail.custom_minimum_size.y = _ps(24)
	tail.mouse_filter = Control.MOUSE_FILTER_IGNORE
	body.add_child(tail)

# -- Resolving a §6.6 row to a real menu item ---------------------------------

## Which menus have had `about_to_popup` emitted during the screen currently
## being filled. Cleared by `_fill()`, so entering a screen refreshes each menu
## it reads exactly once -- see the header on why "once per screen" is the
## budget rather than "once per row" or "never".
var _refreshed: Dictionary = {}

func _menu_buttons() -> Array:
	var out: Array = []
	if _shell.menu_bar_row == null:
		return out
	for child in _shell.menu_bar_row.get_children():
		if child is MenuButton:
			out.append(child)
	return out

## A program menu's `PopupMenu` by the title `DccShell.add_menu()` gave it, or
## `null`.
##
## `refresh` fires `about_to_popup` -- once per menu per screen fill, tracked in
## `_refreshed`. The root passes `false` for its two drill rows: it only needs
## the popup OBJECT to push, and refreshing a menu the user is merely looking at
## the name of is the cost `_menu_row()`'s own header refuses. A screen that
## reads a menu's *items* passes `true`, which is the same refresh opening that
## menu performs on desktop.
func _menu_popup(title: String, refresh: bool = true) -> PopupMenu:
	for mb in _menu_buttons():
		if String(mb.text) == title:
			var p := (mb as MenuButton).get_popup()
			if refresh and not _refreshed.has(title):
				_refreshed[title] = true
				p.about_to_popup.emit()
			return p
	return null

## First index in `p` carrying `id`, or -1.
##
## **First, deliberately, and it is not arbitrary.** Item ids are unique per
## *command*, not per popup: `menus.gd::_todo()` and `_signpost()` call
## `add_item(text)` with no id, so Godot assigns the item's index -- which
## collides with the small explicit ids at the top of the same menu. Measured
## on the built tree by `_phonemore_inv_probe.gd`: File carries id 10 twice
## (`New world…` and the `projects` storage readout), 11, 12, 13 and 19 likewise,
## and Data carries id 45 three times. In every collision the **named command
## comes first**, because `menus.gd` adds it before the readout block. Rows here
## therefore resolve to the command and never to the readout that shadows it.
func _find_id(p: PopupMenu, id: int) -> int:
	for i in p.item_count:
		if not p.is_item_separator(i) and p.get_item_id(i) == id:
			return i
	return -1

## Index in `p` whose submenu is the node named `node`, or -1. Submenu node
## names are `menus.gd`'s own literals (`"ThemeChoice"`, `"AutosaveInterval"`,
## …) and are stable in a way a submenu row's *id* is not -- `add_submenu_item`
## takes no id, so those ids are indices and move whenever a row above them is
## added or removed.
func _find_sub(p: PopupMenu, node: String) -> int:
	for i in p.item_count:
		if p.get_item_submenu(i) == node:
			return i
	return -1

## A row for a menu item this build does not have.
##
## Drawn, not skipped. A screen assembled from ids in another file will one day
## name an id that file no longer has, and the failure mode of skipping is a
## screen that is quietly one row shorter than the spec it was built from --
## which is the class of defect this whole surface exists to remove.
func _missing_row(label: String, why: String) -> Control:
	return _row(label, why, null, null, Callable(), true)

# -- The screens --------------------------------------------------------------

func _fill_screen(body: VBoxContainer, id: String, arg: String = "") -> void:
	match id:
		"more": _fill_root(body)
		"project": _fill_project(body)
		"civ": _fill_civ(body)
		"data": _fill_data(body)
		"sim": _fill_sim(body)
		"prefs": _fill_prefs(body)
		"assets": _fill_assets(body)
		"assets-grid": _fill_assets_grid(body, arg)
		"asset-slot": _fill_asset_slot(body, arg)
		"travel": _fill_travel(body)
		"travel-item": _fill_travel_item(body, arg)
		"landmarks": _fill_landmarks(body)
		"lm-fam": _fill_lm_family(body, arg)
		"help": _fill_help(body)
		"gestures": _fill_gestures(body)
		"data-tiles": _fill_data_tiles(body)
		_:
			body.add_child(_missing_row("Unknown screen '%s'." % id,
				"No builder in _fill_screen()."))

## §6.6 `root`: eight nav rows, then `STATUS`, then the readouts.
func _fill_root(body: VBoxContainer) -> void:
	for spec in ROOT_ROWS:
		var label := String(spec.label)
		var sub := String(spec.sub)
		var glyph := _glyph(String(spec.glyph))
		match String(spec.t):
			"screen":
				var target := String(spec.id)
				if sub == "":
					sub = _live_root_sub(target)
				## §6.6 badges its `Asset library` row `72 / 113`. This build
				## can answer that for real -- see `_asset_fill_badge()`, which
				## counts `as_family_slots()` rather than printing a figure.
				var badge: Control = _root_badge(target)
				_add(body, _row(label, sub, badge, _chevron(),
					func(): _push_screen(target), false, glyph))
			"menu":
				var p := _menu_popup(String(spec.id), false)
				if p == null:
					_add(body, _missing_row(label,
						"The %s menu is not on this build's menu bar." % spec.id))
				else:
					## §6.6 badges its `Asset library` row `72 / 113`. This
					## build has no filled/total to report, so the badge is the
					## popup's own row count -- read off the menu, never written
					## down, exactly as `_menu_row()` trails it.
					var n := 0
					for i in p.item_count:
						if not p.is_item_separator(i):
							n += 1
					_add(body, _row(label, sub, _trail_label("%d" % n), _chevron(),
						func(): _push(p, label, SCREEN_LEVEL), false, glyph))

	## §15 fault 2: the desktop readout cluster used to be reparented into the
	## sheet whole -- a 150 px wordmark and five labels that are empty before a
	## generation, eating most of the surface. The readouts themselves are worth
	## having; the desktop chrome around them is not. They are rows now.
	##
	## §6.6's `root` table ends on exactly this block: `head STATUS`, then
	## `WORLD`, `STATE`, `LAST AUTOSAVE` and `UNDO DEPTH`. `STATUS_ROWS` is this
	## shell's own slot list rather than those four, because these are the slots
	## `app.gd` actually writes -- a `read` row for a value nothing produces is
	## the "no value drawn as a plausible value" defect, not conformance.
	var status := _status_rows()
	if not status.is_empty():
		_head(body, "Status")
		for entry in status:
			## `entry[2]` is set only for the one slot that holds a sentence.
			_add(body, _note_row(String(entry[0]), String(entry[1])) if bool(entry[2])
				else _value_row(String(entry[0]), String(entry[1])))
		## The only status row a user can *act* on, and until 2026-09-01 the
		## phone showed the readout and offered nothing. `app.gd` puts a
		## Recompute button beside the desktop `stale` slot and its own
		## comment says why the handset does not get it -- SS13 parks the
		## status bar in an invisible model host here, so a control added to
		## `status_row` is never drawn -- and ends by naming this file as
		## where the phone half belongs. This is that row: the same act, on
		## the same two preconditions, presented as a list row.
		if _can_recompute_stale():
			_add(body, _row("Recompute stale stages",
				"Re-runs only the stages the graph reports stale. The civilisation "
					+ "layer is deliberately not cascaded per edit, so \"civ\" usually "
					+ "stays -- Civilization ▸ Settlements ▸ Recompute civilisation is "
					+ "the one that clears it.",
				null, null, _go_recompute_stale, false))

	## Everything §6.6's root table does not name -- see `ROOT_REST`.
	var rest: Array = []
	for mb in _menu_buttons():
		var name_ := String(mb.text)
		if ROOT_REST.has(name_) or not _named_in_root(name_):
			rest.append(mb)
	if not rest.is_empty():
		_head(body, "Not on the MORE list")
		for mb in rest:
			_add(body, _menu_row(mb))

## Is this program menu already reached by one of `ROOT_ROWS`, or by one of the
## bespoke screens? Drives the fallback band, so a menu can never go silently
## missing from the phone -- including an eighth one a future `menus.gd` adds,
## which lands in `Not on the MORE list` rather than nowhere.
##
## The screen-owned menus are named here rather than derived, because the
## coverage lives in `_fill_project()` / `_fill_data()` / `_fill_prefs()` /
## `_fill_assets()` / `_fill_help()` as row calls and there is nothing to read
## it off. `_phonemore_reach_probe.gd` is what checks the claim either way.
##
## **`Assets` and `Help` joined this list on 2026-09-06**, when their root rows
## stopped being `menu` drills and became bespoke screens. The comment here used
## to say `Assets` was deliberately absent *"because the root already drills
## it"*; it does not any more, and leaving it out would have put the whole
## Assets popup in the `Not on the MORE list` band as well as on its own screen
## -- two routes to one menu, which is exactly the duplication this predicate
## exists to prevent.
func _named_in_root(menu_title: String) -> bool:
	if ["File", "Data", "Preferences", "Assets", "Help"].has(menu_title):
		return true
	for spec in ROOT_ROWS:
		if String(spec.t) == "menu" and String(spec.id) == menu_title:
			return true
	return false

## §6.6's root glyph column, with a coverage check.
##
## The eight glyphs are literal characters in the spec (`⧉ ◍ ⇅ ▦ ≋ ◷ ⚙ ?`) and
## this shell's mono face is IBM Plex Mono, which does not carry all of them. A
## glyph the font has no outline for draws as a notdef box -- a row that reads
## as broken rather than as decorated -- so an absent one falls back to the
## submenu arrow every other drill row in this file already uses.
##
## `Font.has_char()` is asked at draw time rather than a list being written
## down, so swapping the face (or adding a fallback to it) changes the answer
## with no edit here.
func _glyph(ch: String) -> Label:
	var f := DccTheme.mono(0)
	var text := ch
	if ch == "" or (f != null and not f.has_char(ch.unicode_at(0))):
		text = DccIcons.SYMBOLS["submenu"]
	var l := DccTheme.mono_label(text, "text_faint", _ps(12), 0)
	l.custom_minimum_size.x = _ps(18)
	l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	l.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return l

## §6.6 `project`: the spec's four acts and its autosave block in the spec's
## order, then every remaining `File` row in `File`'s own order.
##
## The tail is `_rest_of()` rather than a written list, and that is what makes
## this screen a re-presentation of File rather than a second copy of it: Revert
## to last save, the `STORAGE LOCATIONS` band and its four readouts, Change
## locations…, Show project on disk, Close project and File's two signposts all
## arrive because they are in the popup, not because they are named here.
##
## §6.6 also draws a `RECENT WORLDS` head with two worlds under it as ordinary
## rows. `_inline_sub()` does that with the real `RecentWorlds` submenu, so the
## list is however many projects have actually been opened -- including the
## "No recent projects" readout `menus.gd` puts there when none have.
func _fill_project(body: VBoxContainer) -> void:
	var p := _menu_popup("File")
	if p == null:
		_add(body, _missing_row("Project", "The File menu is not on this build's menu bar."))
		return
	var drawn := {}
	_act(body, p, DccMenus.ID_SAVE, drawn)
	_act(body, p, DccMenus.ID_SAVE_AS, drawn)
	_act(body, p, DccMenus.ID_NEW_WORLD, drawn)
	_act(body, p, DccMenus.ID_OPEN_PROJECT, drawn)

	_head(body, "Recent worlds")
	_inline_sub(body, p, "RecentWorlds", drawn)

	_head(body, "Autosave")
	_act(body, p, DccMenus.ID_AUTOSAVE, drawn)
	_expand_sub(body, p, "AutosaveInterval", "Interval", drawn)

	_rest_of(body, p, drawn)

## §6.6 `civ`. The spec's row list is `Settlement · Point of interest · Way`,
## then Landmark generation, then the journey planner.
##
## **This port's CIVIL tool set is six, and one of the spec's three is not in
## it.** `civilization_workspace.gd::_build_tools()` builds Settlement,
## Territory, Territory lasso, Way, Route and Conflict (SP-4); the comment directly above it says of POI: *"no
## function anywhere in this workspace drops one, so there is nothing an armed
## POI tool could call. Arming a button with no engine behind it would be the
## fake control this port's own discipline exists to avoid, so it is omitted
## rather than built disabled or wired to a stub."* That reasoning decides this
## screen too -- the row is drawn with the reason, and arms nothing.
##
## `CIV_TOOLS` mirrors that block **by hand**. There is no accessor on the
## workspace to read the list from -- `_build_tools()` passes its six
## dictionaries straight to `DccWidgets.tools_block()` and keeps nothing -- so
## the ids here are a copy and can go stale. `_phonemore_act_probe.gd` arms all
## six through the drawn rows and reads `app.armed_tool` back, which is what
## turns the copy into a checked claim.
const CIV_TOOLS: Array = [
	{"id": "settlement", "label": "Settlement",
		"sub": "tap drops a place · class and snapping from the CIVIL dock"},
	{"id": "territory", "label": "Territory",
		"sub": "drag to claim · commit or discard from the tool options"},
	{"id": "territory_lasso", "label": "Territory lasso",
		"sub": "taps place a ring · commit assigns its inside to the faction"},
	{"id": "way", "label": "Way", "sub": "taps append waypoints · commit from the dock"},
	{"id": "route", "label": "Route", "sub": "sea and river legs between two ports"},
	{"id": "conflict", "label": "Conflict",
		"sub": "taps draw a front, arrow, siege or battle · commit, then fill it in the dock"},
]

func _fill_civ(body: VBoxContainer) -> void:
	_info(body, "Arming a tool closes this screen — tap the map to place. "
		+ "Esc, or arming another tool, disarms.")
	for t in CIV_TOOLS:
		var id := String(t.id)
		var armed: bool = String(_shell.get("armed_tool")) == id
		## §6.6's `nav` row spec: *"Badge `9px mono` at its own colour"*, and for
		## this row *"badge `ARMED` in `var(--acc)`"*. `_trail_label()` is
		## `text_dim`, which is the count colour, so the badge gets its own.
		_add(body, _row(String(t.label), String(t.sub), null,
			_badge("ARMED") if armed else _chevron(),
			func(): _arm_civ_tool(id), false))
	## §6.6's third row, and the one thing on its `civ` screen this port has no
	## engine for. See `CIV_TOOLS`' header for the workspace's own wording.
	_add(body, _missing_row("Point of interest",
		"No POI placement tool exists in this port: nothing in the civilisation "
			+ "workspace drops one, so an armed tool would have nothing to call. "
			+ "POI art is an icon family — Assets ▸ Icon families ▸ Points of interest."))

	## §6.6's `civ` row goes to its `landmarks` screen, and as of 2026-09-06 so
	## does this one. **It drilled `Assets ▸ Landmark types ▸` until then**, and
	## that submenu is a *read-only* one -- `menus.gd::_build_landmark_family()`
	## ends every family with a signpost saying so in its own words: *"Rows here
	## read state; they do not arm a type."* So the row promised landmark
	## generation and reached a report of it. It now reaches the caps, the
	## spacing dial and the run, over the same `landmark_*` bridge the CIVIL
	## dock uses. The submenu is still reachable, on the `assets` screen, where
	## `_rest_of()` draws it as itself.
	##
	## The subtitle stays `_landmark_sub()`'s count, which is read off that same
	## submenu -- a type or a family added to `menus.gd` still moves this line.
	var assets := _menu_popup("Assets")
	var lm_sub := ""
	if assets != null and _find_sub(assets, "LandmarkTypes") >= 0:
		lm_sub = _landmark_sub(assets.get_node_or_null(NodePath("LandmarkTypes")) as PopupMenu)
	_add(body, _row("Landmark generation",
		lm_sub if lm_sub != "" else "Caps, spacing and one run.",
		null, _chevron(), func(): _push_screen("landmarks"), false))

	_add(body, _row("Open journey planner", "Party, season, carriage, stages and cost.",
		null, _chevron(), _go_journey_planner, false))
	_add(body, _row("Civilization dock",
		"Settlements, factions, provinces, trade, roads, sea routes and journeys.",
		null, _chevron(), _go_civilization, false))

## `49 types · 6 families` in §6.6 -- counted off the real submenu rather than
## written down, so a family or a type added to `menus.gd` moves this line.
func _landmark_sub(sub: PopupMenu) -> String:
	if sub == null:
		return ""
	var fams := 0
	var types := 0
	for i in sub.item_count:
		var node := sub.get_item_submenu(i)
		if node == "":
			continue
		fams += 1
		var fp := sub.get_node_or_null(NodePath(node)) as PopupMenu
		if fp == null:
			continue
		for j in fp.item_count:
			## The family popups end in an "Open … in the dock" command and a
			## signpost; a *type* is a plain enabled row before those.
			if fp.is_item_separator(j) or fp.get_item_id(j) >= DccMenus.ID_LM_FAMILY_FIRST:
				continue
			if typeof(fp.get_item_metadata(j)) == TYPE_STRING:
				continue
			types += 1
	return "%d types · %d families · cap + spacing" % [types, fams]

## §6.6 `data`. The Data popup already carries §6.6's own band names as labelled
## separators (`IMPORT`, `EXPORT`, `SOURCES`, `VALIDATION`) in §6.6's own order,
## so this screen is the popup re-presented with §6.6's row types and nothing
## re-ordered. The fifth band §6.6 draws, `CONVERSION`, is not built: see
## `SCREEN_TITLES` for the owner decision that removed it.
##
## **One row is not the popup's**: §6.6's EXPORT band opens with `nav Maps ·
## tile pyramid` → `data-tiles`, so that row is drawn first under the popup's
## own `EXPORT` separator. The popup is split there with `_rest_of()`'s own
## `drawn` set rather than re-walked, so every popup row still arrives in the
## popup's order; the popup's `Maps` route row (the desktop pane, which also
## has the marquee grid export) stays below it. No `EXPORT` separator -> the
## row goes last under its own band rather than nowhere.
func _fill_data(body: VBoxContainer) -> void:
	var p := _menu_popup("Data")
	if p == null:
		_add(body, _missing_row("Data manager",
			"The Data menu is not on this build's menu bar."))
		return
	var cut := p.item_count - 1
	for i in p.item_count:
		if p.is_item_separator(i) and _clean(p.get_item_text(i)).to_upper() == "EXPORT":
			cut = i
			break
	var head := {}
	var tail := {}
	for i in p.item_count:
		if i > cut:
			tail[i] = true
		else:
			head[i] = true
	_rest_of(body, p, tail)
	if cut == p.item_count - 1 and not p.is_item_separator(cut):
		_head(body, "Export")
	_add(body, _row("Maps · tile pyramid", "leaflet · XYZ · TMS · WMTS · retina",
		## §6.6 writes `{zmax} levels`; `0..zmax` is zmax + 1 of them.
		_badge("%d levels" % (_dt_zmax + 1)), _chevron(),
		func(): _push_screen("data-tiles"), false, _glyph("▦")))
	_rest_of(body, p, head)

## §6.6 `sim`. Every quantity on this screen is `DccShell`'s own timeline model
## -- `TL_YEAR_MIN..TL_YEAR_MAX`, `TL_SPEEDS` and `TL_LAYERS` -- and each of the
## three matches §6.6's table exactly (−400…1200; ×1 · ×10 · ×100; Climate on,
## Population on, Economy off, Politics on, Infrastructure off, Warfare off).
## Nothing here is a written copy of the spec's numbers.
##
## **One row is an addition, not §6.6's**: `Playback`. The spec puts the
## transport in the on-map strip and gives this screen a rate but no way to
## start anything, so Speed here would set a number nothing consumed until the
## user found the strip. `DccShell.tl_toggle_play()` is the same call the
## strip's own button makes.
func _fill_sim(body: VBoxContainer) -> void:
	var strip_on := _shell.is_phone_sim_strip_open()
	_add(body, _row("Transport strip on map", "Mini scrub above the nav bar.",
		null, _switch(strip_on), func(): _toggle_sim_strip(not strip_on), false))

	var live := _shell.tl_available()
	if not live:
		_add(body, _missing_row("Year", DccShell.TL_UNAVAILABLE))
	else:
		_add(body, _slider_row("Year", "YEAR %d" % _shell.tl_year(),
			DccShell.TL_YEAR_MIN, DccShell.TL_YEAR_MAX, _shell.tl_year(),
			func(v: float): _shell.tl_set_year(int(v))))
		var speeds: Array = []
		for mult in DccShell.TL_SPEEDS:
			var m := int(mult)
			speeds.append({"label": "×%d" % m, "on": _shell.tl_speed == m,
				"press": func(): _set_sim_speed(m)})
		_chips(body, "Speed", speeds)
		_add(body, _row("Playback", _shell.tl_state_text(), null,
			_switch(_shell.tl_playing), func(): _toggle_sim_play(), false))

	_head(body, "Simulation layers")
	for row in DccShell.TL_LAYERS:
		var lid := String(row[0])
		var on: bool = bool(_shell.tl_layers.get(lid, bool(row[1])))
		_add(body, _row(lid, "", null, _switch(on), func(): _toggle_sim_layer(lid), false))
	## §6.6's info row is *"Generation is not time-based — the timeline drives
	## simulation layers only."* Its first clause is true here; its second is
	## not, and `DccShell.TL_LAYER_NOTE` is this shell's own wording for what
	## these six toggles do instead. Quoted rather than paraphrased, the same
	## way `app.gd::_build_timeline_layers()` quotes it.
	_info(body, "Generation is not time-based. Simulation layers — %s."
		% DccShell.TL_LAYER_NOTE)
	_add(body, _row("Simulation model", "Collapse and recovery, over the recorded years.",
		null, _chevron(), _go_simulation, false))

## §6.6 `prefs`. Theme and Units are lifted to the top -- §6.6 opens with them
## and its subtitle names `application` first -- and the rest of the Preferences
## popup follows in the popup's own order, which already carries §6.6's
## `PERFORMANCE`, `GRAPHICS`, `TILES & LOD` and `MEMORY` heads as labelled
## separators.
##
## §6.6's `TOUCH` head and its `Gesture reference` drill are built as of
## 2026-09-06. **This comment said they were "not built" and gave the reason:**
## the screen is nine claims about what a gesture does on this shell, and
## writing them down without testing each against this build is the "prose that
## describes behaviour nobody checked" defect. That reason still stands and is
## what `_fill_gestures()` is: each of the nine was checked at a symbol, three
## turned out to be wrong about this build and say what it does instead, and two
## do not exist here and are drawn as absent.
func _fill_prefs(body: VBoxContainer) -> void:
	var p := _menu_popup("Preferences")
	if p == null:
		_add(body, _missing_row("Preferences",
			"The Preferences menu is not on this build's menu bar."))
		return
	var drawn := {}
	_head(body, "Application")
	_expand_sub(body, p, "ThemeChoice", "Theme", drawn)
	_expand_sub(body, p, "UnitsChoice", "Units", drawn)
	_rest_of(body, p, drawn)
	## §6.6 closes this screen on `head TOUCH` / `nav ☰ Gesture reference`.
	## After `_rest_of()`, so the Preferences popup's own bands are not
	## interrupted by a band that belongs to neither of them.
	_head(body, "Touch")
	_add(body, _row("Gesture reference", "The whole touch vocabulary this build "
		+ "has, each row checked at its symbol.", null, _chevron(),
		func(): _push_screen("gestures"), false, _glyph("☰")))

# -- §6.6's sub-screens (2026-09-06) ------------------------------------------
#
# §6.6 defines seventeen screens. Six shipped 2026-09-05; **eleven remained**
# (`data-tiles`, `data-io`, `assets`, `assets-grid`, `asset-slot`, `travel`,
# `travel-item`, `landmarks`, `lm-fam`, `help`, `gestures` -- counted off
# §6.6's own `_moreTitle()` table, which is the enumeration, not off a backlog
# row, which says twelve and lists eleven). Nine are below, and `data-tiles`
# (2026-09-23) after them. The one that is not built is not built for the
# reason in the next paragraph, and is not drawn as an empty frame:
#
#   - **`data-io`** is a mock in the specification itself. Its own first row is
#     *"Route configuration is desktop-parity mock in this prototype. The route
#     exists so nothing on the phone is unreachable"*, and §9 item 16 records
#     that all eight of its callers land on one screen whose title never
#     changes because `this.state.dataRoute` is declared and never assigned.
#     This port has the real thing instead -- fourteen `DataManagerWindow.
#     ROUTES`, each opened by `open_data_manager_route()` -- and the `data`
#     screen already draws every one of those rows. Building `data-io` here
#     would add a screen whose only honest content is a copy of the rows one
#     level above it.
#
# Everything below reads live state. Where a figure does not exist this build
# draws no figure: no invented byte sizes, no `0.9 · build 2611`, no fill
# counts written down. Where §6.6's own row is *wrong about this build* -- three
# of `gestures`' nine are -- the row says what this build does.

## The live `EngineBridge`, or `null`.
##
## `bridge` lives on `DccApp`, not on `DccShell`, so it is fetched by name for
## the reason `_can_recompute_stale()` already gives: `DccApp extends DccShell`
## and `DccShell` builds this file, so a typed access would close a class cycle,
## and the capture probes instantiate `DccShell` bare where the property does
## not exist at all.
func _engine():
	return _shell.get("bridge")

## The bridge, or `null` **while the engine is busy as well as when it is
## absent**.
##
## `generate()` and `landmark_run()` both hold `world_gen` mutably borrowed on a
## worker thread, and any `#[func]` reached from the main thread meanwhile fails
## its own `Gd<T>::bind()` -- a Rust panic per call, which `engine_bridge.gd`'s
## landmark block documents with its 360-panic measurement. Every `landmark_*`
## wrapper carries that guard itself; **`as_family_slots`, `as_slot_summary`,
## `as_item_summary`, `as_pack_info` and every `tl_*` wrapper do not** -- they
## guard only on `_has()`. So this is where the guard lives for the asset and
## travel screens, and `_busy_why()` is what those screens draw instead of a
## count they could not take.
func _engine_idle():
	var br = _engine()
	if br == null or bool(br.get("generating")):
		return null
	return br

func _busy_why() -> String:
	var br = _engine()
	if br == null:
		return "This build has no engine bridge, so there is nothing to read."
	if bool(br.get("generating")):
		return ("A generation or landmark pass holds the engine. These counts are "
			+ "read live and cannot be taken while it is running — reopen this "
			+ "screen when the status bar clears.")
	return "The engine reported nothing for this screen."

# -- assets / assets-grid / asset-slot ----------------------------------------

## `AssetLibraryWindow.FAMILIES`' row for `key`, or an empty Dictionary.
##
## The family table is read off that window rather than copied here, and that
## is the whole reason these three screens are a re-presentation rather than a
## second asset library: a family added there (or its slot list changed)
## appears here with no edit, exactly as a `menus.gd` row does.
func _asset_family(key: String) -> Dictionary:
	for f in AssetLibraryWindow.FAMILIES:
		if String((f as Dictionary).get("key", "")) == key:
			return f
	return {}

func _asset_family_title(key: String) -> String:
	var f := _asset_family(key)
	return String(f.get("title", key)) if not f.is_empty() else key

## `{filled, total}` over one family's real slots, or an empty Dictionary when
## the engine could not be read. **Empty, not `{0, 0}`** -- "no art anywhere"
## and "could not ask" are different answers and a zero would print as the
## first.
func _asset_family_fill(family_key: String) -> Dictionary:
	var br = _engine_idle()
	if br == null or not br.has_method("as_family_slots"):
		return {}
	var rows: Array = br.as_family_slots(family_key)
	if rows.is_empty():
		return {}
	var filled := 0
	for r in rows:
		if bool((r as Dictionary).get("filled", false)):
			filled += 1
	return {"filled": filled, "total": rows.size()}

## §6.6 badges the root's `Asset library` row `72 / 113`. That is a written
## figure over an invented family list; this one is `as_family_slots()` counted
## across `AssetLibraryWindow.FAMILIES`, and it is **absent rather than zero**
## when the engine is busy or the binding is missing.
##
## **Eight crossings of the gdext boundary on every root render**, so it gets a
## number rather than an assumption: `0.412 ms` median (`0.400..0.796`) over 9
## against a `5.28 ms` (`5.25..6.08`) whole-root render -- measured 2026-09-06
## by `_phonemore2_probe.gd` at 1080x2400, the badge **timed on its own** rather
## than inferred from the render, since a whole-render figure cannot say whose
## milliseconds it is. A second process re-ran it at `0.452` (`0.436..0.560`),
## inside the first bracket, so the order of magnitude holds. `_menu_row()`'s
## header refuses a preview that would fire one `about_to_popup`; this is a
## different cost -- a registry walk, not a menu handler -- and small enough to
## keep.
func _root_badge(screen_id: String) -> Control:
	if screen_id != "assets":
		return null
	var filled := 0
	var total := 0
	var any := false
	for f in AssetLibraryWindow.FAMILIES:
		var n := _asset_family_fill(String((f as Dictionary).get("key", "")))
		if n.is_empty():
			continue
		any = true
		filled += int(n["filled"])
		total += int(n["total"])
	return _badge("%d / %d" % [filled, total]) if any else null

## §6.6 `assets`. Eleven family nav rows badged `filled / total`, then a
## `COLLECTIONS` block.
##
## **Eight families, not eleven, and not the canvas's 24.**
## `AssetLibraryWindow.FAMILIES_NOTE` is the authority and states the reason in
## its own words: *"frozen against the reference engine
## (cartalith-assets::slots / library) -- not the design canvas's own 24. The
## canvas subdivides more finely (splitting e.g. 'Feature icons' into 'Trees &
## cover' / 'Rock & scree'); no Rust type draws that line"*. Drawing §6.6's
## eleven would mean inventing three families and splitting one that is a single
## slot registry in the engine.
##
## §6.6's `COLLECTIONS` block is **not** carried: it is `read UNASSIGNED
## IMPORTS 3` plus an info line, and per-slot collection membership is a
## `as_slot_summary()`-per-slot walk (`asset_library_window.gd`'s own comment
## names that cost). What replaces it is the real pack line, which is one call.
##
## Ends in `_rest_of()` over the Assets popup. That is what keeps the Assets
## menu reachable now that the root row pushes this screen instead of drilling
## it -- Asset library…, Sprite sheet slicer…, Icon families ▸, Texture sets ▸,
## Landmark types ▸ and the rest all arrive because `menus.gd` put them there.
func _fill_assets(body: VBoxContainer) -> void:
	var br = _engine_idle()
	for f in AssetLibraryWindow.FAMILIES:
		var fam: Dictionary = f
		var key := String(fam.get("key", ""))
		var title := String(fam.get("title", key))
		var n := _asset_family_fill(key)
		var sub := "%s · %s" % [String(fam.get("group", "")),
			"%d px seamless tile" % int(fam.get("size", 0)) if bool(fam.get("texture", false))
				else "%d px RGBA icon" % int(fam.get("size", 0))]
		if n.is_empty():
			## A family whose count could not be taken still drills -- the grid
			## re-asks, and by then the pass may have finished.
			_add(body, _row(title, sub, null, _chevron(),
				func(): _push_screen("assets-grid", key), false, _glyph("▦")))
			continue
		_add(body, _row(title, sub,
			_badge("%d / %d" % [int(n["filled"]), int(n["total"])]), _chevron(),
			func(): _push_screen("assets-grid", key), false, _glyph("▦")))
	if br == null:
		_info(body, _busy_why())

	_head(body, "Pack")
	if br != null and br.has_method("as_pack_info"):
		var pack: Dictionary = br.as_pack_info()
		## Every field here is optional in the reply and each is omitted rather
		## than defaulted: an unnamed pack is a pack with no name, and printing
		## `—` for `name` would read as a pack called that.
		for pair in [["Name", "name"], ["Author", "author"], ["Licence", "license"]]:
			var text := String(pack.get(String(pair[1]), "")).strip_edges()
			if text != "":
				_add(body, _value_row(String(pair[0]), text))
		if pack.has("total_items"):
			_add(body, _value_row("Stored items", "%d" % int(pack["total_items"])))
	else:
		_add(body, _missing_row("Pack metadata", _busy_why()))

	var p := _menu_popup("Assets")
	if p == null:
		_add(body, _missing_row("Assets menu",
			"The Assets menu is not on this build's menu bar."))
		return
	_rest_of(body, p, {})

## §6.6 `assets-grid`: *"No list rows -- this screen renders only the slot
## grid"*, four columns, one cell per slot, tapping a cell pushes `asset-slot`.
##
## §6.6 draws **12 cells always** and fills `Places → 10`, `Trees & cover → 11`,
## every other family `7` -- a written fill pattern over an invented slot count.
## This draws one cell per **real** slot (`as_family_slots()`: 7 for
## `textures`, 15 for `biomes`, 10 for `icons`, …) with each cell's own real
## `filled` flag, so an empty family draws an empty grid rather than seven
## imaginary filled cells.
##
## The `custom` family is the one that can legitimately have no slots at all,
## and its empty state says so rather than drawing nothing.
func _fill_assets_grid(body: VBoxContainer, family_key: String) -> void:
	var fam := _asset_family(family_key)
	if fam.is_empty():
		_add(body, _missing_row(family_key,
			"No such family in AssetLibraryWindow.FAMILIES."))
		return
	var br = _engine_idle()
	if br == null or not br.has_method("as_family_slots"):
		_add(body, _missing_row("%s slots" % String(fam.get("title", family_key)),
			_busy_why()))
		return
	var rows: Array = br.as_family_slots(family_key)

	## §6.6's header row: the family and `tap a slot` on the right.
	var filled := 0
	for r in rows:
		if bool((r as Dictionary).get("filled", false)):
			filled += 1
	_head(body, "%s · slots" % String(fam.get("title", family_key)))
	_add(body, _value_row("Filled", "%d of %d" % [filled, rows.size()]))

	if rows.is_empty():
		_info(body, ("This family has no slots yet. %s"
			% ("Custom icons are created by importing an image with no slot focused "
				+ "(as_add_custom_slot); the eight registry families are fixed."
				if bool(fam.get("custom", false))
				else "The engine reported an empty slot registry for it.")))
	else:
		var grid := GridContainer.new()
		grid.columns = ASSET_GRID_COLS
		grid.add_theme_constant_override("h_separation", _ps(8))
		grid.add_theme_constant_override("v_separation", _ps(8))
		var wrap := MarginContainer.new()
		wrap.add_theme_constant_override("margin_left", _ps(16))
		wrap.add_theme_constant_override("margin_right", _ps(16))
		wrap.add_theme_constant_override("margin_top", _ps(8))
		wrap.add_theme_constant_override("margin_bottom", _ps(8))
		wrap.add_child(grid)
		for i in rows.size():
			grid.add_child(_asset_cell(fam, rows[i], i))
		_add(body, wrap)

	_add(body, _row("Open %s in the asset library" % String(fam.get("title", family_key)),
		"Import, slice, tag, collect and apply — the whole window, on this family.",
		null, _chevron(), func(): _go_asset_library(family_key), false))

## §6.6's grid geometry is `repeat(4,1fr)`, and **that spec is the whole reason
## for the value** -- 4 is what the design draws.
##
## **The touch floor does NOT pin it, and an earlier version of this comment
## claimed it did.** That claim said "a fifth column would take it under on a
## narrow handset"; a verifier set the constant to 5 and re-ran at both ends
## (540x1200 and 1080x2400) and got `0 failure(s)` both times -- five columns at
## 412 dp is about 71 dp square, still well clear of `DccTheme.PHONE_TAP_MIN`.
## `fits` does not catch it either: `assets-grid body_min_w` goes 819 -> 1008
## against a 1080 screen.
##
## So **no assertion currently covers this value** -- it is held by the design
## spec alone. Said plainly rather than left as a coverage claim that would not
## have gone red. Still true on 2026-09-06: `grep -rn ASSET_GRID_COLS` over the
## whole `godot-project` tree, probes and scenes included, returns this line and
## its one use above, and nothing else.
##
## **Where the assertion goes when someone writes it**, so the next pass does
## not have to re-derive it: `_phonemore2_probe.gd`'s section 4 already opens
## `assets-grid` on a family discovered live and holds the drawn body
## (`pm._screen_body`) -- it asserts one cell per slot and stops there. The
## missing half is `repeat(4,1fr)` itself, and it is two properties off the
## drawing rather than a re-statement of this constant: the `GridContainer`
## under that body has `columns == 4`, and every cell under it carries
## `SIZE_EXPAND_FILL` (the `1fr`, which is what makes four columns four *equal*
## columns rather than four content-sized ones). Asserting `ASSET_GRID_COLS ==
## 4` here instead would be the constant-against-itself shape `MISTAKES.md`
## already has an entry for -- the literal `4` has to come from §6.6, and the
## columns have to be read off the built grid.
const ASSET_GRID_COLS := 4

## One slot cell. A `Button`, deliberately -- **not** the `PanelContainer` +
## `_row_input()` pair the list rows use.
##
## `_row_input()`'s own header records why that pair exists and what it costs:
## a `PanelContainer` is not a `BaseButton`, so it never gets the free
## press-cancel a `ScrollContainer` gives a real button when a drag passes
## `scroll_deadzone`, and PH-15 had to reimplement it. A grid cell needs no
## wrapped second line, which is the only thing the pair buys, so it takes the
## `BaseButton` and gets that cancellation for nothing.
func _asset_cell(fam: Dictionary, row: Dictionary, index: int) -> Control:
	var uid := String(row.get("uid", ""))
	var filled := bool(row.get("filled", false))
	var code := "%s-%02d" % [String(fam.get("code", "?")), index + 1]
	var slot_name := String(row.get("name", row.get("id", "")))

	var b := Button.new()
	b.focus_mode = Control.FOCUS_NONE
	b.custom_minimum_size = Vector2(_pt(64), _pt(64))
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.tooltip_text = "%s · %s · %s" % [code, slot_name,
		"%d variant(s)" % int(row.get("item_count", 0)) if filled else "empty"]
	## §6.6's own filled/empty distinction, onto the two stylebox factories this
	## shell already has: a filled slot is a raised surface (`panel_alt`), an
	## empty one is the bordered-but-transparent `pill(false)` -- "there is a
	## slot here and nothing in it". §6.6 hatches the empty one with a 45°
	## repeating gradient, which is not something a `StyleBoxFlat` draws, so the
	## outline carries that job instead.
	var sb: StyleBox = DccTheme.flat(DccTheme.c("panel_alt"), _ps(14)) if filled \
		else DccTheme.pill(false, _ps(14), _ps(6), _ps(6))
	b.add_theme_stylebox_override("normal", sb)
	b.add_theme_stylebox_override("hover", sb)
	b.add_theme_stylebox_override("disabled", sb)
	b.add_theme_stylebox_override("pressed", DccTheme.pill(true, _ps(14), _ps(6), _ps(6)))
	if uid != "":
		b.pressed.connect(func(): _push_screen("asset-slot", uid))
	else:
		b.disabled = true

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", _ps(2))
	col.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	col.alignment = BoxContainer.ALIGNMENT_CENTER
	col.mouse_filter = Control.MOUSE_FILTER_IGNORE
	b.add_child(col)

	var name_l := DccTheme.mono_label(slot_name, "text" if filled else "text_ghost",
		_ps(9), 0)
	name_l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	name_l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	name_l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	col.add_child(name_l)

	var code_l := DccTheme.mono_label(code, "accent" if filled else "text_ghost",
		_ps(8.5), 1)
	code_l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	code_l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	col.add_child(code_l)
	return b

## §6.6 `asset-slot`'s subtitle is `{assetFam} · slot {n+1}`. The slot's real
## family comes back from `as_slot_summary()`, so this is the family it is in
## rather than the family the user happened to arrive from.
func _slot_subtitle(uid: String) -> String:
	var br = _engine_idle()
	if br == null or not br.has_method("as_slot_summary"):
		return ""
	var s: Dictionary = br.as_slot_summary(uid)
	if not bool(s.get("ok", false)):
		return ""
	return _asset_family_title(String(s.get("family", "")))

## §6.6 `asset-slot`. **§9 item 20 records that the prototype's own version of
## this screen is a fixed placeholder** -- *"`asset-slot` shows the same fixed
## placeholder (`capital-star.png · 512×512 · 84 KB`, `118% · fit · reset`,
## anchor `base`, `×3` variants) for every slot in every family"*. Every figure
## below is instead the engine's, and one of the prototype's five is **not
## drawn at all**: the engine reports no stored byte size, which
## `asset_library_window.gd` already found and states in its own words
## (*"`as_item_summary` carries name/transform/decoded size/hash and nothing
## else, so the last field is dropped rather than invented"*).
func _fill_asset_slot(body: VBoxContainer, uid: String) -> void:
	var br = _engine_idle()
	if br == null or not br.has_method("as_slot_summary"):
		_add(body, _missing_row("Slot", _busy_why()))
		return
	var s: Dictionary = br.as_slot_summary(uid)
	if not bool(s.get("ok", false)):
		_add(body, _missing_row("Slot",
			"This slot no longer exists in the live session — a batch delete or "
				+ "rename removed it while this screen was open."))
		return
	var fam := _asset_family(String(s.get("family", "")))
	var item_count := int(s.get("item_count", 0))

	_add(body, _value_row("Slot", String(s.get("name", s.get("id", uid)))))
	if not fam.is_empty():
		_add(body, _value_row("Family", String(fam.get("title", ""))))
	_add(body, _value_row("Variants", "%d" % item_count))

	if item_count > 0 and br.has_method("as_item_summary"):
		var item: Dictionary = br.as_item_summary(uid, 0)
		if bool(item.get("ok", false)):
			## `w`/`h` are the DECODED size, which is what the window's own
			## readout prints. No byte figure follows it: see this function's
			## header.
			_add(body, _value_row("File", "%s · %d × %d · PNG" % [
				String(item.get("name", "")), int(item.get("w", 0)), int(item.get("h", 0))]))
			if item.has("scale"):
				_add(body, _value_row("Scale", "%d%%" % int(roundf(float(item["scale"]) * 100.0))))
			if item.has("pan_x") and item.has("pan_y"):
				_add(body, _value_row("Pan", "%.0f, %.0f" % [
					float(item["pan_x"]), float(item["pan_y"])]))
			if item.has("hash"):
				_add(body, _value_row("Content hash", String(item["hash"])))
		var png: PackedByteArray = br.as_thumbnail_png(uid, 0, 256) \
			if br.has_method("as_thumbnail_png") else PackedByteArray()
		var preview := _asset_preview(png)
		if preview != null:
			_add(body, preview)
	else:
		_info(body, "No art in this slot yet. Import one from the asset library — "
			+ "the phone screens read the library, they do not write to it.")

	if not fam.is_empty():
		## §6.6 draws `ANCHOR base` as an editable-looking field. It is not
		## per-slot here and saying so is the point: `asset_library_window.gd`
		## states it as *"Anchor is fixed by the family
		## (cartalith-assets::Family), not a per-slot setting"*, and its own
		## chips are drawn disabled for that reason.
		_add(body, _note_row("Anchor", "%s — fixed by the family, not per slot."
			% String(fam.get("anchor", "center"))))
		_add(body, _value_row("Bakes to", "%d px %s" % [int(fam.get("size", 0)),
			"opaque, seamless tile" if bool(fam.get("texture", false))
				else "RGBA, straight alpha"]))

	var tags: PackedStringArray = s.get("tags", PackedStringArray())
	if tags.size() > 0:
		_add(body, _value_row("Tags", " · ".join(tags)))
	if bool(s.get("has_dupe", false)):
		_add(body, _note_row("Duplicate art",
			"Another slot in this library holds a byte-identical image."))

	var fam_key := String(s.get("family", ""))
	_add(body, _row("Open in the asset library",
		"Replace, add a variant, tag, or move this slot into a collection.",
		null, _chevron(), func(): _go_asset_library(fam_key), false))

## A decoded thumbnail, or `null` when there is nothing to draw.
##
## Null rather than an empty frame: a bordered blank square is indistinguishable
## from art that failed to decode, and this screen already says "no art in this
## slot" above in the case where there is none.
func _asset_preview(png: PackedByteArray) -> Control:
	if png.is_empty():
		return null
	var img := Image.new()
	if img.load_png_from_buffer(png) != OK:
		return null
	var tr := TextureRect.new()
	tr.texture = ImageTexture.create_from_image(img)
	tr.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	tr.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	tr.custom_minimum_size.y = _ps(160)
	tr.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var wrap := MarginContainer.new()
	wrap.add_theme_constant_override("margin_left", _ps(16))
	wrap.add_theme_constant_override("margin_right", _ps(16))
	wrap.add_theme_constant_override("margin_top", _ps(10))
	wrap.add_theme_constant_override("margin_bottom", _ps(10))
	wrap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.add_child(tr)
	return wrap

## Closed first, and every window opener in this file does the same: the window
## opens over the map, and leaving this full-screen overlay underneath it puts
## the user behind it the moment they dismiss the window.
##
## Reached by name rather than by type -- `open_asset_library()` lives on
## `DccApp`, the subclass that owns the windows, and a typed call from a file
## `DccShell` builds would close a class cycle. Guarded, because `DccShell` is
## also instantiated bare by the capture probes.
func _go_asset_library(family_key: String) -> void:
	close()
	if _shell.has_method("open_asset_library"):
		_shell.call("open_asset_library", family_key, false)

# -- travel / travel-item -----------------------------------------------------

## Which of `TravelLibraryWindow.KINDS` the `travel` screen's `seg` has
## selected. §6.6 defaults it to `ANIMALS`; `KINDS[0]` is `animal`, so the
## default is read off that table rather than written here.
var _travel_kind := ""

func _travel_kinds() -> Array:
	return TravelLibraryWindow.KINDS

func _travel_current_kind() -> String:
	if _travel_kind == "" and not _travel_kinds().is_empty():
		_travel_kind = String((_travel_kinds()[0] as Dictionary).get("key", ""))
	return _travel_kind

## §6.6 `travel`. A `seg` over the four types, then one nav row per entry.
##
## §6.6 lists **sixteen entries by name with their sub text written out**
## (`Horse` / `mount · pack 90 kg · 24 km/d base · grazing normal`, …). Not one
## of them is written here: `tl_list(kind)` returns the live library, and each
## row carries its own `subtitle` (`travel_bridge.rs`), so a row edited in the
## Travel library window — or added there, or duplicated — moves this screen.
## That also means the count is whatever the project holds, not sixteen.
func _fill_travel(body: VBoxContainer) -> void:
	var kind := _travel_current_kind()
	var chips: Array = []
	for k in _travel_kinds():
		var kd: Dictionary = k
		var key := String(kd.get("key", ""))
		chips.append({"label": String(kd.get("label", key)), "on": key == kind,
			"press": func(): _set_travel_kind(key)})
	_chips(body, "Type", chips)

	var br = _engine_idle()
	if br == null or not br.has_method("tl_list"):
		_add(body, _missing_row("Travel entries", _busy_why()))
		return
	var rows: Array = br.tl_list(kind)
	if rows.is_empty():
		_add(body, _missing_row("Travel entries",
			## Parenthesised, because `%` binds tighter than `+`: without them
			## the format would apply to the last literal alone, which carries
			## no placeholder.
			("This build's library holds no entries of this type. "
				+ "tl_list(\"%s\") returned nothing — an older extension has no "
				+ "tl_* bindings at all, and a newer one ships stock rows.") % kind))
	for r in rows:
		var row: Dictionary = r
		var id := String(row.get("id", ""))
		var origin := String(row.get("origin", ""))
		## `origin` is `stock` / `custom`. Badged only when it is `custom`,
		## because "this one was edited or added here" is the state worth
		## marking; badging every stock row would make the badge mean nothing.
		var mark: Control = _badge("CUSTOM") if origin == "custom" else _chevron()
		_add(body, _row(String(row.get("name", id)), String(row.get("subtitle", "")),
			null, mark, func(): _push_screen("travel-item", "%s|%s" % [kind, id]),
			false, _glyph("≋")))

	_info(body, "An information layer only — entries become selectable options in "
		+ "the journey planner's party form.")
	_add(body, _row("Open the travel library",
		"Add, duplicate, edit and validate entries — the whole window.",
		null, _chevron(), func(): _go_travel_library_kind(kind), false))

func _set_travel_kind(kind: String) -> void:
	_travel_kind = kind
	_render()

func _go_travel_library_kind(kind: String) -> void:
	close()
	if _shell.has_method("open_travel_library"):
		_shell.call("open_travel_library", kind)

## `"kind|id"` -> the entry's live name, for the header subtitle.
func _travel_entry_name(arg: String) -> String:
	var parts := arg.split("|")
	if parts.size() != 2:
		return ""
	var br = _engine_idle()
	if br == null or not br.has_method("tl_get"):
		return ""
	var e: Dictionary = br.tl_get(String(parts[0]), String(parts[1]))
	return String(e.get("name", "")) if bool(e.get("ok", false)) else ""

## §6.6 `travel-item`: `ENTRY` / `CLASS` / `CONSTRAINTS` / `SOURCE`, plus
## `LOAD INTO PLANNER ➔` for the party set-ups.
##
## §6.6 also writes out three party presets as a table of nine columns each
## (`Merchant caravan` groupSize 12, pace `Steady`, …). Those are the
## prototype's own fixtures; here the presets are `tl_list("preset")` rows and
## the planner reads them itself, so the table is not copied and cannot drift
## from the library.
func _fill_travel_item(body: VBoxContainer, arg: String) -> void:
	var parts := arg.split("|")
	if parts.size() != 2:
		_add(body, _missing_row("Travel entry",
			"Malformed screen argument '%s' — expected \"kind|id\"." % arg))
		return
	var kind := String(parts[0])
	var id := String(parts[1])
	var br = _engine_idle()
	if br == null or not br.has_method("tl_get"):
		_add(body, _missing_row("Travel entry", _busy_why()))
		return
	var e: Dictionary = br.tl_get(kind, id)
	if not bool(e.get("ok", false)):
		_add(body, _missing_row("Travel entry",
			"tl_get(\"%s\", \"%s\") reports this entry no longer exists." % [kind, id]))
		return

	_add(body, _value_row("Entry", String(e.get("name", id))))
	var kind_label := kind
	for k in _travel_kinds():
		if String((k as Dictionary).get("key", "")) == kind:
			kind_label = String((k as Dictionary).get("label", kind))
	_add(body, _value_row("Class", kind_label))
	var subtitle := String(e.get("subtitle", ""))
	if subtitle != "":
		_add(body, _note_row("Constraints", subtitle))
	_add(body, _value_row("Source",
		"stock definition" if String(e.get("origin", "")) == "stock" else "project data"))

	## §3's validation state, drawn because it is the one thing on this entry
	## that can be WRONG -- a row missing a required field is selectable in the
	## planner and then fails there instead of here.
	var vstate := String(e.get("validation_state", ""))
	if vstate != "" and vstate != "ok":
		var missing: PackedStringArray = e.get("validation_missing", PackedStringArray())
		_add(body, _note_row("Validation", "%s%s" % [vstate,
			(" — missing %s" % " · ".join(missing)) if missing.size() > 0 else ""]))

	## Usage, and both halves only when the bridge sent them: a `0` here means
	## "nothing uses it", which is a real and useful answer, but an ABSENT key
	## means the binding did not report and must not print as zero.
	var used := PackedStringArray()
	if e.has("usage_presets"):
		used.append("%d party set-up(s)" % int(e["usage_presets"]))
	if e.has("usage_journeys"):
		used.append("%d journey(s)" % int(e["usage_journeys"]))
	if used.size() > 0:
		_add(body, _value_row("Used by", " · ".join(used)))

	if kind == "preset":
		_add(body, _row("Load into planner",
			"Writes this set-up's party fields into the journey planner and opens it.",
			null, _chevron(), func(): _load_preset_into_planner(id), false))
	else:
		_info(body, "Selectable as a mount, vehicle or vessel in the planner's "
			+ "party form.")
	_add(body, _row("Open in the travel library",
		"Edit, duplicate or validate this entry.", null, _chevron(),
		func(): _go_travel_library_kind(kind), false))

## §6.6's `LOAD INTO PLANNER ➔`.
##
## The planner is opened **first**, because `_apply_preset()` calls
## `_rebuild_party_form()` and `_compute()` on the view, and doing that before
## the view has been presented rebuilds a form nobody is looking at and then
## rebuilds it again on open.
##
## `_apply_preset` is reached **by name**, which is the same route
## `_go_recompute_stale()` takes to `DccShell._recompute_stale()` and for the
## same reason -- but unlike that one it crosses into another file's private
## method, and that is worth saying plainly rather than burying: a public entry
## point on `JourneyPlannerView` would be better, and this row is the caller
## that would justify adding one. Guarded, so an older or refactored planner
## leaves the user in the planner rather than doing nothing silently.
func _load_preset_into_planner(id: String) -> void:
	close()
	if _shell.has_method("open_journey_planner"):
		_shell.call("open_journey_planner")
	var view = _shell.get("journey_planner_view")
	if view != null and view.has_method("_apply_preset"):
		view.call("_apply_preset", id)
	elif _shell.has_method("set_status"):
		_shell.call("set_status", "hint",
			"This build's journey planner has no preset entry point — pick the "
				+ "set-up from the planner's own party row.", "warn")

# -- landmarks / lm-fam -------------------------------------------------------

## §6.6's landmark screens are the one place it writes out a whole model: six
## families, 49 types, a cap and a `was` per type, a placement formula and a
## `setInterval` run mock. **None of that is copied.** This engine has the real
## thing behind `EngineBridge`'s `landmark_*` block -- `landmark_kinds()` is the
## type registry, `landmark_settings()` the caps and armed flags,
## `landmark_headroom()` §6.6's own info line (`caps_total` / `room_estimate` /
## `last_placed`), `landmark_funnels()` the per-type limiting reason, and
## `landmark_run()` a real threaded pass -- and `CivilizationWorkspace` is the
## desktop panel over exactly those calls.
##
## The ladder, the class labels, the Crowding range and the limiting-reason
## vocabulary are read from that class's own constants rather than restated, so
## a ladder rung added there appears here. That is deliberate and it is the same
## contract this file has with `menus.gd`: one definition, two presentations.
##
## §6.6's `nav ×6` count is not written down either -- the families are the
## engine's, first-seen out of `landmark_kinds()`, which is the order
## `menus.gd::_build_landmark_types_menu()` also uses.
func _lm_kinds() -> Array:
	var br = _engine()
	if br == null or not br.has_method("landmark_kinds"):
		return []
	return br.landmark_kinds()

func _lm_settings() -> Dictionary:
	var br = _engine()
	if br == null or not br.has_method("landmark_settings"):
		return {}
	return br.landmark_settings()

## `kind key -> funnel row`, so a type's last-run result is one lookup.
func _lm_funnels() -> Dictionary:
	var br = _engine()
	if br == null or not br.has_method("landmark_funnels"):
		return {}
	var out: Dictionary = {}
	for f in br.landmark_funnels():
		out[String((f as Dictionary).get("kind", ""))] = f
	return out

## The engine's families, in the engine's own first-seen order.
func _lm_families() -> Array:
	var order: Array = []
	for k in _lm_kinds():
		var fam := String((k as Dictionary).get("family", "other"))
		if not order.has(fam):
			order.append(fam)
	return order

func _fill_landmarks(body: VBoxContainer) -> void:
	var br = _engine()
	var kinds := _lm_kinds()
	if br == null or kinds.is_empty():
		_add(body, _missing_row("Landmark types",
			"This build's extension returned no landmark vocabulary. "
				+ "landmark_kinds() is the type registry and it came back empty, "
				+ "which is a missing binding rather than an empty world — "
				+ "rebuild the native library."))
		return
	var st := _lm_settings()
	var caps: Dictionary = st.get("caps", {})
	var armed: Dictionary = st.get("armed", {})
	var funnels := _lm_funnels()

	## §6.6's info row is one sentence of three figures. **Two of the three are
	## not always figures**, and this is the one place on the screen where a
	## zero would read as a measurement:
	##
	##   - `room_estimate` is `0` **whenever there is no generated world**, not
	##     because nothing fits. `lib.rs::landmark_headroom()` computes it only
	##     for `WorldSource::Generated` and its other arm is a literal `_ => 0`.
	##     With 384 of armed caps standing, *"room for about 0 at this spacing"*
	##     is the app reporting a world it has not got.
	##   - `last_placed` is `0` until the first pass, because
	##     `landmark_store.last` is `None` and `map_or(0, …)` flattens that to a
	##     count. *"last run placed 0"* before any run reads as a run that
	##     placed nothing.
	##
	## So each clause is emitted only when it is one, and what is missing says
	## why. `caps_total` is exact at all times (`LandmarkStore::caps_total()`
	## over the armed kinds) and is always drawn.
	var head: Dictionary = br.landmark_headroom() if br.has_method("landmark_headroom") else {}
	if head.is_empty():
		_info(body, "This build's bridge reports no headroom estimate, so neither "
			+ "the packing figure nor the last run's total is shown. The caps and "
			+ "the spacing below are live.")
	else:
		var line := PackedStringArray(["caps total %d" % int(head.get("caps_total", 0))])
		var world: bool = bool(br.get("has_world"))
		if world:
			line.append("room for about %d at this spacing" % int(head.get("room_estimate", 0)))
		var ran: bool = not (br.landmark_funnels() as Array).is_empty()
		if ran:
			line.append("last run placed %d" % int(head.get("last_placed", 0)))
		var why := PackedStringArray()
		if not world:
			why.append("the packing estimate needs a generated world")
		if not ran:
			why.append("no landmark pass has run yet")
		_info(body, " · ".join(line)
			+ ("" if why.is_empty() else " — " + ", and ".join(why) + "."))

	## §6.6's `range Crowding`, over `CivilizationWorkspace`'s own dial range and
	## step, with §4.1's second line: `× 1.00` is arithmetic, `34 km` is a fact
	## about the map. `_lm_radius_in_force()` DIVIDES -- the engine's
	## `radius_km` is `base / crowding`, and the desktop panel multiplied here
	## until 2026-09-03, so this calls that helper rather than doing the sum.
	var crowd := float(st.get("crowding", 1.0))
	var radii: Array = st.get("class_radius_km", [])
	var qi: int = CivilizationWorkspace.LM_QUOTED_CLASS
	var base_km: float = float(radii[qi]) if qi < radii.size() else 0.0
	var cname := String(CivilizationWorkspace.LM_CLASS_LABEL.get(
		CivilizationWorkspace.LM_CLASSES[qi], "")).to_lower()
	##
	## The readout is `× 1.00` alone and the sentence is a wrapping row under
	## it, which is the desktop panel's own structure (`_lm_crowd_readout` and
	## `_lm_crowd_note` are two labels) and is here for a measured reason: a
	## `_slider_row()` display is a `mono_label` that neither wraps nor clips, so
	## its full text width is the row's minimum width. With the sentence inline
	## this screen measured **1035 px of minimum width against a 1080 px
	## screen** — 96% of it, the widest of the nine — and the sentence grows as
	## Crowding falls (`× 0.25` puts a three-digit km in it), so the tightest
	## reading was not the worst case.
	_add(body, _slider_row("Crowding", "× %.2f" % crowd,
		CivilizationWorkspace.LM_CROWDING_MIN, CivilizationWorkspace.LM_CROWDING_MAX,
		crowd, Callable(), CivilizationWorkspace.LM_CROWDING_STEP,
		func(v: float): _lm_write("landmark_set_crowding", [v], true),
		func(v: float): return "× %.2f" % v))
	_info(body, "a %s landmark keeps %.0f km clear · sparse → dense" % [cname,
		CivilizationWorkspace._lm_radius_in_force(base_km, crowd)])

	var compete := bool(st.get("cross_type_competition", true))
	_add(body, _row("Types compete with each other",
		"Off lets a shrine sit beside a waterfall. On keeps every landmark clear "
			+ "of every other one.", null, _switch(compete),
		func(): _lm_write("landmark_set_cross_competition", [not compete], true), false))

	_head(body, "Families")
	for fam in _lm_families():
		var n_armed := 0
		var n_total := 0
		var n_placed := 0
		for k in kinds:
			var kd: Dictionary = k
			if String(kd.get("family", "other")) != fam:
				continue
			n_total += 1
			if bool(armed.get(String(kd.get("key", "")), false)):
				n_armed += 1
			var fn: Dictionary = funnels.get(String(kd.get("key", "")), {})
			n_placed += int(fn.get("placed", 0))
		var fk: String = fam
		_add(body, _row(CivilizationWorkspace._lm_pretty(fk).capitalize(),
			"%d of %d armed · %d placed" % [n_armed, n_total, n_placed],
			null, _chevron(), func(): _push_screen("lm-fam", fk), false))

	var busy: bool = bool(br.get("generating"))
	if busy:
		_add(body, _missing_row("Run landmark pass",
			"A generation or landmark pass is already running. The pass is "
				+ "threaded and only one may hold the engine at a time."))
	else:
		_add(body, _row("Run landmark pass",
			"Places every armed type, once, against the current terrain.",
			null, _chevron(), _lm_run, false))
	## §6.6's closing info, verbatim on its second clause because it is the
	## sentence the whole screen exists to make true.
	_info(body, "a cap is a ceiling, not a quota — the spacing calculation gives "
		+ "the restraint")

## One landmark setting write, then a redraw.
##
## `emit_after` exists because this screen's own rows are its listeners: a
## redraw before the engine call would repaint from the value the setting had
## BEFORE the write, which is the ten-emitters-fired-early defect
## `MISTAKES.md` records. The write happens first, always; only the redraw is
## conditional, and it is skipped for the slider (which redraws on release
## instead, so the body is not rebuilt under a moving finger).
func _lm_write(method: String, args: Array, redraw: bool = false) -> void:
	var br = _engine()
	if br == null or not br.has_method(method):
		return
	br.callv(method, args)
	if redraw:
		_render()

## `landmark_run()` is `await`-able and threaded (`engine_bridge.gd`: it sets
## the same `generating` flag a generate does, so neither can start while the
## other is in flight). The screen is left open across it -- unlike
## `_go_recompute_stale()`, which closes because that call blocks the main
## thread -- and redrawn when the pass returns, so the family rows' `placed`
## counts and the headroom line update in place.
func _lm_run() -> void:
	var br = _engine()
	if br == null or not br.has_method("landmark_run"):
		return
	if _shell.has_method("set_status"):
		_shell.call("set_status", "hint", "Landmark pass running…", "accent")
	var result: Dictionary = await br.landmark_run()
	if _shell.has_method("set_status"):
		_shell.call("set_status", "hint",
			"Landmark pass — %d placed." % int(result.get("placed", 0))
				if bool(result.get("ok", false))
				else String(result.get("error", "The landmark pass did not run.")),
			"accent" if bool(result.get("ok", false)) else "warn")
	if visible:
		_render()

## §6.6 `lm-fam`: one `range` per type over the cap ladder, and a `read` under
## each armed one carrying the last run's placed count and limiting reason.
##
## The ladder is `CivilizationWorkspace.LM_LADDER` and the slider carries its
## **index**, never the cap -- that class's own comment explains why (for a
## Continental type one versus two is the design of the world and 120 versus
## 200 means nothing). Dragging to index 0 disarms and the row then prints what
## the cap was, because the store keeps `armed` and `cap` apart on purpose.
func _fill_lm_family(body: VBoxContainer, family: String) -> void:
	var kinds := _lm_kinds()
	if kinds.is_empty():
		_add(body, _missing_row(family, "landmark_kinds() returned no types."))
		return
	var st := _lm_settings()
	var caps: Dictionary = st.get("caps", {})
	var armed_map: Dictionary = st.get("armed", {})
	var funnels := _lm_funnels()
	var mine: Array = []
	for k in kinds:
		if String((k as Dictionary).get("family", "other")) == family:
			mine.append(k)
	if mine.is_empty():
		_add(body, _missing_row(family,
			"No landmark type reports this family. The families are the engine's "
				+ "own, first-seen out of landmark_kinds()."))
		return

	var ladder: Array = CivilizationWorkspace.LM_LADDER
	var any_viewshed := false
	for k in mine:
		var kd: Dictionary = k
		var key := String(kd.get("key", ""))
		var label := String(kd.get("label", key))
		var cls := String(kd.get("class", ""))
		var buildable := bool(kd.get("buildable", true))
		var needs_vs := bool(kd.get("needs_viewshed", false))
		if needs_vs:
			any_viewshed = true
		var cap := int(caps.get(key, int(kd.get("default_cap", 0))))
		var is_armed: bool = buildable and bool(armed_map.get(key, false))
		var rung: int = CivilizationWorkspace._lm_rung(cap) if is_armed else 0

		## The desktop row's `[viewshed]` tag, as a caption suffix: driven by
		## `needs_viewshed`, which the engine pins to the kinds whose scorer
		## reads the viewshed (see `CivilizationWorkspace._lm_types()`). It
		## used to read "· no viewshed", which was false once the viewshed
		## existed.
		var caption := "%s · %s%s" % [label,
			String(CivilizationWorkspace.LM_CLASS_LABEL.get(cls, cls)).to_lower(),
			" · viewshed" if needs_vs else ""]
		if not buildable:
			## An unbuildable type is drawn, disabled, with the engine's own
			## per-type reason (`landmark_kinds()`' `not_built`) -- `menus.gd`'s
			## honesty rule, and the desktop panel does the same rather than
			## hiding the row.
			_add(body, _missing_row(caption,
				CivilizationWorkspace._lm_not_built_why(kd)))
			continue
		var kk := key
		var kept := cap
		_add(body, _slider_row(caption,
			"%d max" % cap if is_armed else "off · was %d" % cap,
			0.0, float(ladder.size() - 1), float(rung),
			Callable(), 1.0,
			func(v: float): _lm_set_rung(kk, int(round(v))),
			## §6.6's own two readouts: `{cap} max` when armed, `off · was
			## {cap}` at the zero stop. The remembered number is `kept` -- the
			## cap the engine holds right now -- so the zero stop says what the
			## user is about to get back rather than `off · was 0`.
			func(v: float):
				var i: int = clampi(int(round(v)), 0, ladder.size() - 1)
				return "off · was %d" % kept if i == 0 else "%d max" % int(ladder[i])))
		if is_armed and funnels.has(key):
			var fn: Dictionary = funnels[key]
			_add(body, _value_row("↳ last run", "%d placed · %s" % [
				int(fn.get("placed", 0)),
				CivilizationWorkspace._lm_limit_word(String(fn.get("limit", "")))]))

	var fam := family
	_add(body, _row("Arm every type in this family",
		"Each type resumes the cap it was last set to.", null, _chevron(),
		func(): _lm_bulk(fam, true), false))
	_add(body, _row("Turn every type off",
		"Each row keeps its number and says so, so this is reversible.",
		null, _chevron(), func(): _lm_bulk(fam, false), false))
	_info(body, "The slider is one gesture — zero disarms the type and remembers "
		+ "its number; drag up and it resumes. The track is a 1-2-3-5 ladder, "
		+ "not a linear count."
		+ (" · viewshed = " + CivilizationWorkspace.LM_VIEWSHED_WHY.to_lower()
			if any_viewshed else ""))

## Index 0 is the detented zero stop: it disarms and **never writes a cap of
## 0**, so the row keeps its number. That is the desktop panel's own rule
## (`_lm_type_row()`: *"disarming writes `landmark_set_armed(false)` and never
## `landmark_set_cap(0)`, and the row prints `was 40`"*), and getting it wrong
## here would silently destroy a setting on the way past zero.
## Called once, on release -- never per drag frame. See `_slider_row()`'s
## `on_release` note for what per-frame writing would do on the way past zero.
func _lm_set_rung(key: String, rung: int) -> void:
	var ladder: Array = CivilizationWorkspace.LM_LADDER
	var i: int = clampi(rung, 0, ladder.size() - 1)
	if i == 0:
		_lm_write("landmark_set_armed", [key, false], true)
		return
	_lm_write("landmark_set_cap", [key, int(ladder[i])])
	_lm_write("landmark_set_armed", [key, true], true)

func _lm_bulk(family: String, on: bool) -> void:
	for k in _lm_kinds():
		var kd: Dictionary = k
		if String(kd.get("family", "other")) != family:
			continue
		if on and not bool(kd.get("buildable", true)):
			continue
		_lm_write("landmark_set_armed", [String(kd.get("key", "")), on])
	_render()

# -- help / gestures ----------------------------------------------------------

## §6.6 `help`: `read VERSION` `Cartalith Mobile 0.9 · build 2611`, `read ENGINE`
## `shared with desktop · WebGPU`, the gesture-reference drill, and two acts.
##
## **There is no product version in this port and none is invented.** No
## `VERSION` constant exists in the shell, no "Cartalith Mobile" exists at all,
## and `app.gd::open_about()`'s own dialog reports the Godot version and the OS
## rather than a build number. So the VERSION row is not drawn with a made-up
## figure and it is not drawn blank either: what replaces it is the two
## identities this build **can** answer for itself — the engine it is running
## on, and the GPU state the shell already measures.
##
## Ends in `_rest_of()` over the real Help popup, which is what keeps
## Documentation, Keyboard shortcuts…, Credits & academic principles,
## Generation info…, Save diagnostic report and About reachable now that the
## root row pushes this screen instead of drilling the menu. §6.6's `CREDITS &
## ACADEMIC PRINCIPLES` and `REPORT AN ISSUE` acts are two of those six, so they
## arrive as themselves rather than as re-labelled copies.
func _fill_help(body: VBoxContainer) -> void:
	_add(body, _value_row("Runtime", "Godot %s · %s"
		% [Engine.get_version_info().string, OS.get_name()]))
	var gpu := _slot("top_gpu")
	if gpu != "":
		_add(body, _value_row("Engine", gpu))
	_add(body, _row("Gesture reference", "The touch vocabulary this build has.",
		null, _chevron(), func(): _push_screen("gestures"), false, _glyph("☰")))

	var p := _menu_popup("Help")
	if p == null:
		_add(body, _missing_row("Help menu",
			"The Help menu is not on this build's menu bar."))
	else:
		_rest_of(body, p, {})

	## §6.6's own closing line, and the rule this whole surface is held to. It
	## is quoted in this file's header for the same reason.
	_info(body, "The phone reorganises rather than truncates: every desktop "
		+ "function is reachable through MAP · GENERATE · PLAN · MORE.")

## §6.6's nine `read` rows, **checked against this build one at a time**, which
## is what the 2026-09-05 pass declined to do and said so
## (`_fill_prefs()` carried the note: *"writing them down without testing each
## one against this build is exactly the 'prose that describes behaviour nobody
## checked' defect. Reported as outstanding rather than guessed at."*). That
## note is now stale and has been removed from that function.
##
## **Three of §6.6's nine are wrong about this build, and each says what this
## build does instead:**
##
##   - `TWO FINGERS · rotate` — there is no map rotation anywhere in
##     `viewport_host.gd`; what two fingers produce is
##     `InputEventPanGesture`, which pans.
##   - `LONG-PRESS · sample terrain → pin + chip` — `map_overlay.gd`'s
##     `_TOUCH_HOLD_MS` (500 ms) turns a hold into `map_right_clicked`, which
##     `civilization_workspace.gd::on_map_right_clicked()` presents as the L4
##     sheet. Its `Info here` row is the sampling half; there is no pin-and-chip.
##   - `TAB RE-TAP · close the sheet` — `DccShell._pick_phone_tab()` collapses
##     it to **peek** instead, and its own comment says why: this sheet is the
##     tool options bar and it has no "gone" state on the other two form
##     factors.
##
## **Two more of the nine do not exist and are drawn as absent, not omitted:**
## there is no double-tap handler on the map (`viewport_host.gd` has no
## `double_click` branch at all), and no edge-swipe inspector — the only edge
## swipe this shell reads is Android's own back gesture, which
## `DccShell._phone_back()` handles and which is listed here as itself.
##
## Every row below names the symbol it was checked at, in its second line, so
## the next reader can re-check it rather than believing this list.
const GESTURES: Array = [
	["Drag", "Pans the map.", "viewport_host.gd — the pan branch of _input()"],
	["Pinch", "Zooms at the pinch centre.",
		"viewport_host.gd — InputEventMagnifyGesture"],
	["Two fingers", "Pans. This build has no map rotation.",
		"viewport_host.gd — InputEventPanGesture; nothing rotates the view"],
	["Press and hold", "Opens the map menu: edit, move the viewer, delete, drop a "
		+ "settlement here, or read the terrain here.",
		"map_overlay.gd — _TOUCH_HOLD_MS 500 → map_right_clicked → "
			+ "civilization_workspace.gd::on_map_right_clicked()"],
	["Sheet handle", "Drags the sheet between peek, half and full.",
		"dcc_shell.gd — _on_phone_sheet_grab_input() / _set_phone_detent()"],
	["Tab re-tap", "Collapses the sheet to peek. It does not close: the sheet is "
		+ "the tool options bar and has no closed state on desktop or tablet.",
		"dcc_shell.gd — _pick_phone_tab()"],
	["Undo chip", "Tap undoes one step; hold opens the step history.",
		"dcc_shell.gd — PHONE_UNDO_HOLD_SEC 0.45 → the undo_ledger() popover"],
	["Back", "Leaves a sheet, then this screen, then the overlay — never the app. "
		+ "Android's edge swipe and the hardware key are the same gesture.",
		"dcc_shell.gd — _phone_back(), and PhoneMenu.go_back()"],
]

## The two §6.6 rows this build does not have. Drawn, with the true reason,
## rather than dropped: a gesture reference that silently omits two of the nine
## a user may have read about elsewhere teaches them the list is complete.
const GESTURES_ABSENT: Array = [
	["Double-tap", "No double-tap handler exists on the map. viewport_host.gd has "
		+ "no double_click branch; pinch and the zoom buttons are the zoom paths."],
	["Edge-swipe inspector", "No inspector drawer is bound to an edge swipe. The "
		+ "right dock is a sheet reached from the bottom bar, and the only edge "
		+ "swipe this shell reads is Android's back gesture, listed above."],
]

func _fill_gestures(body: VBoxContainer) -> void:
	for g in GESTURES:
		_add(body, _row(String(g[0]), "%s  ·  %s" % [String(g[1]), String(g[2])],
			null, null, Callable(), false))
	_head(body, "Not in this build")
	for g in GESTURES_ABSENT:
		_add(body, _missing_row(String(g[0]), String(g[1])))
	_info(body, "Each row above names the symbol it was checked at — re-check it "
		+ "rather than trusting this list, which is prose about behaviour and "
		+ "goes stale the way prose does.")

# -- data-tiles ---------------------------------------------------------------

## §6.6 `data-tiles` state. Defaults are §6.6's (`scheme:'XYZ'`, `size:'256'`,
## `retina:true`, `zmax:5`). Its fifth, `skip:true`, has no engine behind it --
## see `_fill_data_tiles()`.
var _dt_scheme := "xyz"
var _dt_tile := 256
var _dt_zmax := 5
var _dt_retina := true
## The last export from this screen -- `{key, bytes, secs, path}` -- or empty.
## `key` is the settings it ran with, so SIZE and RENDER TIME are only drawn
## against the settings that produced them.
var _dt_last: Dictionary = {}

## `slippy_export_tiles` clamps `max_z` to `bake_bridge::MAX_BAKE_DEPTH` (6);
## §6.6's range runs to 8. The set drawn is the set the engine honours --
## `DataManagerWindow.ZOOM_NOTE` is the reason the rest are refused.
const DT_ZMAX_CHOICES: Array[int] = [1, 2, 3, 4, 5, 6]
const DT_TILE_SIZES: Array[int] = [256, 512]

func _dt_key() -> String:
	return "%s|%d|%d|%s" % [_dt_scheme, _dt_tile, _dt_zmax, _dt_retina]

## Exact, not §6.6's `4^zmax × 1.37` model: `(4^(N+1) − 1) / 3` per density,
## `cartalith_engine::slippy_export::slippy_tile_count`'s own sum.
func _dt_tile_count() -> int:
	var n := 0
	for z in _dt_zmax + 1:
		n += 1 << (2 * z)
	return n * (2 if _dt_retina else 1)

func _dt_set(field: String, value) -> void:
	set(field, value)
	_render()

## §6.6 `data-tiles`: Scheme, Tile size, Zoom levels, Retina, Skip all-ocean,
## ESTIMATE, the EXPORT act and the destination info -- in §6.6's order.
##
## Drives the same `slippy_export_tiles` binding the desktop Export ▸ Maps pane
## does (`DataManagerWindow._run_export()`), so the archive is identical for the
## same settings. What §6.6 draws and this build does not have is drawn as
## absent with its reason: all-ocean skipping, and a SIZE / RENDER TIME model
## (the desktop pane refuses one too -- both are measured by a real run).
func _fill_data_tiles(body: VBoxContainer) -> void:
	var chips: Array = []
	for s in ["xyz", "tms", "wmts"]:
		var sk: String = s
		chips.append({"label": sk.to_upper(), "on": _dt_scheme == sk,
			"press": func(): _dt_set("_dt_scheme", sk)})
	_chips(body, "Scheme", chips)
	chips = []
	for n in DT_TILE_SIZES:
		var nn: int = n
		chips.append({"label": "%d" % nn, "on": _dt_tile == nn,
			"press": func(): _dt_set("_dt_tile", nn)})
	_chips(body, "Tile size", chips)
	chips = []
	for n in DT_ZMAX_CHOICES:
		var nn: int = n
		chips.append({"label": "0 – %d" % nn, "on": _dt_zmax == nn,
			"press": func(): _dt_set("_dt_zmax", nn)})
	_chips(body, "Zoom levels 0 → N", chips)
	_add(body, _row("Retina @2x", "doubles render cost", null, _switch(_dt_retina),
		func(): _dt_set("_dt_retina", not _dt_retina), false))
	_add(body, _missing_row("Skip all-ocean tiles",
		"Not built: slippy_export_tiles writes every tile of every level, and a "
			+ "skipped tile would reach a web client as a 404 rather than sea."))

	_head(body, "Estimate")
	var tiles := _dt_tile_count()
	_add(body, _value_row("Tiles", "%d" % tiles))
	var measured: bool = not _dt_last.is_empty() and String(_dt_last.get("key", "")) == _dt_key()
	if measured:
		_add(body, _value_row("Size", "%.1f MB" % (float(_dt_last["bytes"]) / 1048576.0)))
		## Short on purpose: `_trail_label()` neither wraps nor clips, so a long
		## value here is this screen's minimum width (`_note_row()`'s header).
		_add(body, _value_row("Render time", "%.1f s" % float(_dt_last["secs"])))
	else:
		_add(body, _missing_row("Size · render time",
			"Measured by an export at these settings, not modelled — this build has "
				+ "no size model and does not invent one."))

	var br = _engine_idle()
	var dest := DccSettings.storage_root("exports").path_join(_dt_file_name())
	if br == null or not br.has_method("slippy_export_tiles"):
		_add(body, _missing_row("Export %d tiles" % tiles, _busy_why()))
	elif not bool(br.get("has_world")):
		_add(body, _missing_row("Export %d tiles" % tiles,
			"There is no world to export. Generate or open one first."))
	else:
		_add(body, _row("Export %d tiles" % tiles, "Writes %s" % dest.get_file(),
			null, null, _dt_export, false))
	_info(body, "Destination %s · one stored .zip holding the tiles, tiles.json and "
		% DccSettings.storage_root("exports")
		+ "leaflet-preview.html — unzip it and open the page in a browser (Leaflet "
		+ "loads from unpkg.com, so the preview needs a connection). style.json "
		+ "and attribution are not written: this build has no style model to emit.")

func _dt_file_name() -> String:
	return "tile-pyramid-%s-z0-%d%s.zip" % [_dt_scheme, _dt_zmax, "@2x" if _dt_retina else ""]

## Synchronous, as the desktop pane's is: `slippy_export_tiles` returns the
## whole archive. The screen stays open and redraws with the measured figures.
## ponytail: blocks the main thread for the export; a worker thread when a
## depth-6 retina export on a handset is measured to need one.
func _dt_export() -> void:
	var br = _engine_idle()
	if br == null or not br.has_method("slippy_export_tiles"):
		return
	var dir := DccSettings.storage_root("exports")
	DirAccess.make_dir_recursive_absolute(dir)
	var path := dir.path_join(_dt_file_name())
	var t0 := Time.get_ticks_msec()
	var bytes: PackedByteArray = br.slippy_export_tiles({"scheme": _dt_scheme,
		"max_z": _dt_zmax, "tile_size": _dt_tile, "retina": _dt_retina})
	var secs := float(Time.get_ticks_msec() - t0) / 1000.0
	var msg := ""
	var ok := false
	if bytes.is_empty():
		msg = "Tile export failed — slippy_export_tiles returned no bytes (see the Godot log)."
	else:
		var f := FileAccess.open(path, FileAccess.WRITE)
		if f == null:
			msg = "Tile export failed — could not open %s for writing." % path
		else:
			f.store_buffer(bytes)
			f.close()
			ok = true
			_dt_last = {"key": _dt_key(), "bytes": bytes.size(), "secs": secs, "path": path}
			msg = "Export complete · %s · leaflet-preview.html inside" % path.get_file()
	if _shell.has_method("set_status"):
		_shell.call("set_status", "hint", msg, "accent" if ok else "warn")
	if visible:
		_render()

## The live status rows, as `[label, value, wraps]`.
##
## **`_slot()`, not `status_slot_text()`.** The raw read only skips a slot that
## was never written, and before a world exists `app.gd` writes `top_world` as
## the shell's em-dash placeholder -- so the very first thing the MORE screen
## drew was a row reading `World —`, a label with a dash where its value should
## be. `_slot()` is the reader that already treats that placeholder as "nothing
## yet" (`_root_meta()` has used it since the header was built), and a readout
## with nothing to say is a row that should not exist rather than a row that
## says nothing.
func _status_rows() -> Array:
	var out: Array = []
	for entry in STATUS_ROWS:
		var key := String(entry[0])
		var text := _slot(key)
		if text != "":
			out.append([String(entry[1]), text, key == "hint"])
	return out

## Both halves of the condition `app.gd` puts on its own Recompute button:
## something is actually stale, and this GDExtension build can act on it.
## An older extension answers `stale_stages()` and not
## `recompute_stale_stages()`, and a row that silently does nothing is
## worse than no row -- so the row is absent rather than drawn dead, which
## is the same call the desktop button makes (it hides itself).
##
## `bridge` lives on `DccApp`, not on `DccShell`, so it is fetched by name
## for the same reason `_engine()` and `_go_travel_library_kind()` do:
## `DccApp extends DccShell` and `DccShell` builds this file, so a
## typed access here would close a class cycle -- and the capture probes
## instantiate `DccShell` bare, where the property does not exist at all.
func _can_recompute_stale() -> bool:
	if _slot("stale") == "":
		return false
	if not _shell.has_method("_recompute_stale"):
		return false
	var br = _shell.get("bridge")
	if br == null or br.world_gen == null:
		return false
	return br.world_gen.has_method("recompute_stale_stages")

## Closed **before** the call, not after. `_recompute_stale()` yields two
## frames to paint its busy state and then blocks the main thread for the
## whole recompute (it is synchronous and has no progress signal), so
## leaving a full-screen overlay up across that freeze is the one
## presentation this must not have. Closing also matches `_activate()`'s
## own default for every non-checkable row.
##
## The result lands in the `stale` and `hint` slots, which this screen
## re-reads from scratch on its next open -- the same freshness every other
## row in the Status band has.
func _go_recompute_stale() -> void:
	if not _shell.has_method("_recompute_stale"):
		return
	close()
	_shell.call("_recompute_stale")

## Civilization is a *domain*, not a menu: the desktop reaches it by clicking
## CIVIL on the rail, and `select_domain()` is that same `_select_domain()`
## made public for exactly this kind of cross-surface jump.
##
## Selecting it is not enough on a phone, and that is the whole point of this
## function. A domain decides what the left dock *would* show, and on a phone
## the dock is a sheet that is closed -- so a bare `select_domain()` swaps the
## tool-options sheet and otherwise leaves the user looking at the same map they
## were looking at, which is a menu row that appears to do nothing. Opening the
## left sheet lands them on the CIVIL dock's own TOOLS block, which is what the
## spec's parenthesis ("settlement/POI/way tools arm & place on map") names.
func _go_civilization() -> void:
	_shell.select_domain("civilization")
	_open_left_sheet()

## Simulation is not a domain of its own, and since Ruling L not a category
## either: `civilization_workspace.gd` builds the collapse/recovery model as the
## closed `Simulate collapse / recovery` expander inside CIVIL ▸ "Timeline"
## (`left_rail_tree_resorted.md` L236-247, was the "Simulation" category), so
## this row lands on Timeline with that expander closed.
## `select_domain_category()` is the shell's one call for "switch domain and
## open this category", and it `push_warning`s rather than failing silently if
## the category is ever renamed out from under this row.
func _go_simulation() -> void:
	_shell.select_domain_category("civilization", "Timeline")
	_open_left_sheet()

## `_set_sheet_open()` is the phone's dock-sheet opener, and its first act is
## `_close_all_phone_overlays()` -- which closes this menu. So there is no
## `close()` call in the two functions above and there must not be one: it
## would run after the sheet opened and take the sheet down with it.
func _open_left_sheet() -> void:
	_shell._set_sheet_open("left", true)

## §6.6 `civ`'s `OPEN JOURNEY PLANNER ➔`. Reached by name for the same class
## cycle reason `_go_travel_library_kind()` gives, and it is the same call
## `DccShell._pick_phone_tab()` makes for the PLAN tab -- so this row and that
## tab land on one view, not two.
##
## **No `_open_left_sheet()` here, unlike `_go_civilization()` and
## `_go_simulation()` above, and the omission is deliberate as of 2026-09-07.**
## It used to be an oversight with the same symptom their headers describe: the
## planner's whole control column lives in `app.left_dock_body`, so this row and
## the PLAN tab both landed on `_center_panel` alone -- four result groups all
## reading "no committed route selected" and nothing to select one with. The fix
## went into `journey_planner_view.gd::open()` instead -- the one function all
## seven entry points converge on, this row included -- so adding a second
## opener here would only close and reopen the sheet it has already put up.
func _go_journey_planner() -> void:
	close()
	if _shell.has_method("open_journey_planner"):
		_shell.call("open_journey_planner")

## §6.6 `civ`: *"Arming a tool closes this sheet — tap the map to place."*
##
## Domain first, then arm, then close, and the order is the whole of it. A tool
## armed while another domain is selected leaves the dock and the tool options
## describing something else; `civilization_workspace.gd::_on_civ_tool_armed()`
## fills the tool-options row from the `tool_armed` emit, so the domain has to
## already be CIVIL when that fires. Closing last, because `close()` only hides
## this overlay and nothing downstream re-opens it.
##
## Deliberately does **not** open the left sheet, unlike `_go_civilization()`:
## the point of arming from here is to get to the map, and the dock sheet would
## cover the thing the user is about to tap.
func _arm_civ_tool(id: String) -> void:
	_shell.select_domain("civilization")
	if _shell.has_method("arm_tool"):
		_shell.call("arm_tool", id)
	close()

## §6.6's root row for Simulation reads `year {n} · running|paused`. Before a
## generate there is no cursor -- `DccShell.tl_year()` returns `0`, which is a
## legal year and would print as though measured -- so the row says what is
## missing instead of printing the placeholder.
func _live_root_sub(screen_id: String) -> String:
	if screen_id != "sim":
		return ""
	if not _shell.tl_available():
		return "no cursor yet — generate a world first"
	return "year %d · %s" % [_shell.tl_year(), _shell.tl_state_text()]

func _toggle_sim_strip(on: bool) -> void:
	_shell.set_phone_sim_strip_open(on)
	## Turning the strip ON and staying on a full-screen overlay would show the
	## user nothing: the strip draws over the map, which this screen covers.
	## §6.6's own toast for this row says the same thing -- *"close this sheet
	## to scrub"* -- so the close is the instruction carried out rather than
	## printed.
	if on:
		close()
	else:
		_render()

func _toggle_sim_play() -> void:
	_shell.tl_toggle_play()
	_render()

func _set_sim_speed(mult: int) -> void:
	_shell.tl_set_speed(mult)
	_render()

func _toggle_sim_layer(id: String) -> void:
	_shell.tl_toggle_layer(id)
	_render()

# -- §6.6 row types over a real popup -----------------------------------------

## Append a control, with the hairline every list in this file puts between its
## rows. A band supplies its own leading gap, so no rule is drawn against one.
func _add(body: VBoxContainer, ctrl: Control) -> void:
	var n := body.get_child_count()
	if n > 0 and not bool(body.get_child(n - 1).get_meta(_META_BAND, false)):
		body.add_child(DccTheme.rule())
	body.add_child(ctrl)

const _META_BAND := "phone_band"

func _head(body: VBoxContainer, title: String) -> void:
	var b := _band(title)
	b.set_meta(_META_BAND, true)
	body.add_child(b)

## §6.6's `info` row: prose, not a control. Same wrapped second-line treatment a
## disabled row gets, with no title above it.
func _info(body: VBoxContainer, text: String) -> void:
	var wrap := MarginContainer.new()
	wrap.add_theme_constant_override("margin_left", _ps(16))
	wrap.add_theme_constant_override("margin_right", _ps(16))
	wrap.add_theme_constant_override("margin_top", _ps(8))
	wrap.add_theme_constant_override("margin_bottom", _ps(8))
	wrap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var l := DccTheme.mono_label(text, "text_ghost", _ps(9.5), 0)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.add_child(l)
	_add(body, wrap)

## One named item of `p`, drawn by whatever kind of item it turned out to be --
## §6.6's `act`, `tog` or `read` all come out of `_popup_row()`, which already
## reads checkable/radio/disabled/submenu off the item itself.
##
## **§6.6's primary/secondary `act` styling is not carried**, and that is
## stated rather than approximated. Its primary act is a filled `var(--acc)`
## pill; the only filled-accent surface in this screen's vocabulary is the
## ARMED badge, and giving Save project the same fill as an armed tool would
## make two unrelated things look like one state. Reported as outstanding.
func _act(body: VBoxContainer, p: PopupMenu, id: int, drawn: Dictionary) -> void:
	var i := _find_id(p, id)
	if i < 0:
		_add(body, _missing_row("Menu item %d" % id,
			"%s has no item with this id in this build." % p.name))
		return
	drawn[i] = true
	_add(body, _popup_row(p, i))

## Draw a submenu's items **inline**, as rows of the screen, rather than as a
## drill. §6.6 does this for `RECENT WORLDS`, whose entries are the screen's own
## content and not a level below it.
func _inline_sub(body: VBoxContainer, p: PopupMenu, node: String,
		drawn: Dictionary) -> void:
	var i := _find_sub(p, node)
	if i < 0:
		_add(body, _missing_row(node, "No such submenu in %s." % p.name))
		return
	drawn[i] = true
	var sub := p.get_node_or_null(NodePath(node)) as PopupMenu
	if sub == null:
		_add(body, _missing_row(node, "The submenu node is not under %s." % p.name))
		return
	sub.about_to_popup.emit()
	for j in sub.item_count:
		if sub.is_item_separator(j):
			continue
		_add(body, _popup_row(sub, j))

## §6.6's `seg`: a submenu of radio items drawn as a chip row on this screen
## instead of a level below it, with anything in the submenu that is *not* a
## radio drawn as an ordinary row underneath.
##
## Falls back to a drill row when `_expandable()` says no, so the caller never
## has to know which submenus qualify.
func _expand_sub(body: VBoxContainer, p: PopupMenu, node: String, label: String,
		drawn: Dictionary) -> void:
	var i := _find_sub(p, node)
	if i < 0:
		_add(body, _missing_row(label, "No submenu '%s' in %s." % [node, p.name]))
		return
	drawn[i] = true
	var sub := p.get_node_or_null(NodePath(node)) as PopupMenu
	if sub == null:
		_add(body, _missing_row(label, "The submenu node is not under %s." % p.name))
		return
	if not _expandable(sub, node):
		_add(body, _popup_row(p, i))
		return
	sub.about_to_popup.emit()
	var chips: Array = []
	var rest: Array = []
	for j in sub.item_count:
		if sub.is_item_separator(j):
			continue
		if sub.is_item_radio_checkable(j) and not sub.is_item_disabled(j):
			var idx := j
			chips.append({"label": _clean(sub.get_item_text(j)),
				"on": sub.is_item_checked(j),
				"press": func(): _activate(sub, idx, true)})
		else:
			rest.append(j)
	_chips(body, _unstamped(label, chips), chips)
	for j in rest:
		_add(body, _popup_row(sub, j))

## `label` with the value the chip row below it is about to light removed.
##
## **The same value was being drawn twice.** `menus.gd::_stamp_pref_values()`
## rewrites every Preferences submenu ROW to `base + PREF_VALUE_SEP + picked`,
## so a desktop popup can show a group's setting without opening it. `_rest_of()`
## then hands that rewritten text straight down as this screen's caption -- and
## the chip row underneath already draws `picked` in the accent. The screen read
## `Tiled LOD   Auto on zoom` over an accented `Auto on zoom` chip, for eight of
## the ten groups on it.
##
## **Theme and Units escaped it, and that escape is the shape of this fix.**
## `_fill_prefs()` calls `_expand_sub()` for those two with a caption of its own
## -- `"Theme"`, `"Units"` -- and never reads the popup's text at all. Everything
## else arrives through `_rest_of()`, whose whole point is that it does not name
## the rows it draws, so it cannot be the place that knows which suffix is a
## readout. Stripping here, where the chips are already in hand, keeps that
## property: `_rest_of()` stays a walk over `menus.gd`'s own order.
##
## Keyed to `PREF_VALUE_SEP` **and** to an ON chip's own label, taken from the
## `chips` array rather than re-read from the submenu -- so what is stripped
## cannot disagree with what is drawn. A group with no checked chip
## (`Relief exaggeration`, whose chips are commands rather than a radio set) is
## stamped by nothing and so trimmed by nothing.
##
## **The match is a prefix in EITHER direction, and that is not defensive
## slack.** `_stamp_pref_values()` runs on the Preferences popup's own
## `about_to_popup`, which `_menu_popup()` fires *before* `_expand_sub()` fires
## the submenu's -- so on the first render of a screen the parent row is stamped
## from submenu text that the submenu's own refresher has not written yet. With
## a world loaded, `menus.gd::_refresh_undo_budget_menu()` rewrites `256 MB` to
## `256 MB — 5 steps here`, and the caption read `Undo history   256 MB` over a
## lit `256 MB — 5 steps here` chip: the same value twice, in two different
## wordings, which an equality test cannot see. Found by generating a world in
## the probe rather than by reading the code -- with no world the refresher
## takes its `step <= 0` branch, writes the short form, and the defect is
## invisible. The reverse (a stamp longer than the chip, after a world is
## dropped) is the same fault mirrored, hence `or`.
##
## Not `_rest_of()`'s `_popup_row()` path: a submenu drawn as an ordinary drill
## row has no chips under it, so there the stamped value is the only readout
## there is and removing it would delete information rather than deduplicate it.
func _unstamped(label: String, chips: Array) -> String:
	var cut := label.rfind(DccMenus.PREF_VALUE_SEP)
	if cut <= 0:
		return label
	var tail := label.substr(cut + DccMenus.PREF_VALUE_SEP.length())
	if tail == "":
		return label
	for c in chips:
		if not bool(c["on"]):
			continue
		var picked := String(c["label"])
		if picked.begins_with(tail) or tail.begins_with(picked):
			return label.substr(0, cut)
	return label

## May this submenu be drawn inline as chips?
##
## Two derived conditions and one written exclusion, named separately because
## they are not the same kind of fact:
##
##   - **derived** — at least two enabled radio items, and no nested submenu. A
##     nested submenu has nowhere to go on a chip row, and a submenu with fewer
##     than two radios is a list of commands rather than a choice.
##   - **written** — `NO_EXPAND`, whose two entries have `about_to_popup`
##     handlers with side effects. See that constant for what each one does.
func _expandable(sub: PopupMenu, node: String) -> bool:
	## `_rest_of()` passes the result of a `get_node_or_null()` straight in, so
	## a submenu row whose node is missing arrives here as null. Not expandable
	## rather than a crash -- `_popup_row()` then draws it as an ordinary row,
	## which is what a submenu with nothing behind it should look like.
	if sub == null or NO_EXPAND.has(node):
		return false
	var radios := 0
	for j in sub.item_count:
		if sub.is_item_separator(j):
			continue
		if sub.get_item_submenu(j) != "":
			return false
		if sub.is_item_radio_checkable(j) and not sub.is_item_disabled(j):
			radios += 1
	return radios >= 2

## Every item of `p` not already in `drawn`, in `p`'s own order.
##
## This is what keeps a bespoke screen a re-presentation rather than a second
## copy: §6.6 decides the order of the rows it names, and everything else
## arrives because `menus.gd` put it in the popup. A row added there appears
## here with no edit to this file, which is the property the whole surface had
## before the ruling and the one thing worth carrying across it.
func _rest_of(body: VBoxContainer, p: PopupMenu, drawn: Dictionary) -> void:
	for i in p.item_count:
		if drawn.has(i):
			continue
		if p.is_item_separator(i):
			var cap := _clean(p.get_item_text(i))
			if cap != "":
				_head(body, cap)
			continue
		var node := p.get_item_submenu(i)
		if node != "" and _expandable(p.get_node_or_null(NodePath(node)) as PopupMenu, node):
			_expand_sub(body, p, node, _clean(p.get_item_text(i)), drawn)
			continue
		_add(body, _popup_row(p, i))

## §6.6's `seg` control: a label over a wrapped row of chips.
##
## Chips are `_pt(44)` tall, not §6.6's `min-height:40px`. 44 dp is the TARGETS
## card's own floor and the one this shell is held to; `DccWidgets.action`'s
## 39 px and `DccTheme.role_px("chip_min_h")`'s 34 are both filed standing
## issues, so neither is the number to copy.
func _chips(body: VBoxContainer, label: String, entries: Array) -> void:
	if entries.is_empty():
		return
	var wrap := MarginContainer.new()
	wrap.add_theme_constant_override("margin_left", _ps(16))
	wrap.add_theme_constant_override("margin_right", _ps(16))
	wrap.add_theme_constant_override("margin_top", _ps(8))
	wrap.add_theme_constant_override("margin_bottom", _ps(6))
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", _ps(7))
	wrap.add_child(col)
	var cap := DccTheme.mono_label(label, "text_dim", _ps(10), 0)
	cap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	col.add_child(cap)
	var flow := HFlowContainer.new()
	flow.add_theme_constant_override("h_separation", _ps(6))
	flow.add_theme_constant_override("v_separation", _ps(6))
	col.add_child(flow)
	for e in entries:
		flow.add_child(_chip(String(e["label"]), bool(e["on"]), e["press"]))
	_add(body, wrap)

func _chip(text: String, on: bool, on_press: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_NONE
	b.custom_minimum_size.y = _pt(44)
	## **Medium when selected.** The owner asked for the chosen option to read
	## bold in the overview, and this was answered "not achievable" -- an answer
	## that measured the DESKTOP `PopupMenu`, where it is true: none of its 25
	## `set_item_*` methods carries a font, a weight or a style. This screen is
	## not a `PopupMenu`. It is a `Button` that already sets its own face, and
	## `DccTheme.mono()`'s second argument has selected `FONT_MONO_MED` since it
	## was written, so the whole change is `on` in the line below.
	##
	## Weight is the half of the cue that costs no layout, and that is why it
	## survived the ruling below: **Plex Mono Medium's advance is identical to
	## Regular's**, so a group whose chips wrap onto two lines (Undo history's
	## five, VRAM budget's eight) does not re-flow when the selection moves.
	## Measured rather than asserted -- `_chipink_probe.gd` reads
	## `get_string_size()` for both faces over all 38 chip labels this screen
	## draws, flips each live chip's face under it, and builds a
	## selected/unselected pair over one identical string: 0 px on every one.
	b.add_theme_font_override("font", DccTheme.mono(0, on))
	b.add_theme_font_size_override("font_size", _ps(10))
	## **The accent, and it is not `accent_ink` -- `LARGE_ITEM_RULINGS.md`
	## ruling B, owner, 2026-09-07.** Recorded at the call site because the
	## ruling says to: *"Record it at the call site or a later conformance pass
	## will read the accent as drift and remove it."*
	##
	## The owner asked for the selected option to read **bold**. True Bold (700)
	## means a fourth face and a wider advance, which would re-flow every chip
	## row -- and phone chip rows are where this project's clipping defects have
	## appeared. So the ruling **keeps Medium and adds the accent ink**:
	## *"Colour costs no layout at all, so it buys legibility for free."*
	##
	## **Which token: `accent`, not `accent_ink`.** The canvas's own chip helper
	## for these screens gives a selected chip accent TYPE over a wash, never
	## reversed type over a slab -- `design/mcp-2026-09-07/Cartalith
	## Android.dc.html:1356`:
	##
	##     const chip=(on)=>({bord:on?'var(--acc)':'var(--hair)',
	##                        col:on?'var(--acc)':'var(--sec)',
	##                        bg:on?'var(--wash)':'transparent'});
	##
	## consumed at `AND:1362` (`opts:(r.opts||[]).map(v=>({v,...chip(v===r.cur)}))`)
	## as the MORE screens' `seg` options, which is exactly this row. `--acc` is
	## `#e0a34a` / `#a4650f` (`AND:31` / `AND:1469`) and `--accInk` is declared
	## in those same two lines -- so the canvas has both and spends `--accInk`
	## somewhere other than a chip's text.
	##
	## **The fill had to move with it, and that is a measurement rather than a
	## preference:** accent type on an accent slab is **1.00:1**. The ink cannot
	## change without the ground under it -- see `_chip_box()`.
	var ink := DccTheme.c("accent") if on else DccTheme.c("text_dim")
	b.add_theme_color_override("font_color", ink)
	b.add_theme_color_override("font_hover_color",
		ink if on else DccTheme.c("text_bright"))
	## **The press flash is the on-state now, for both chips.** It used to be
	## `pill(true)` + `accent_ink` whatever the chip's state, so tapping an
	## unselected chip showed a filled amber slab for the press and then settled
	## into a washed chip -- two different appearances for one selection.
	## Pressing now previews the state the tap is about to produce.
	##
	## **`pill(true)` is not gone from this file**, and the claim is worth being
	## exact about rather than sweeping: `_asset_cell()` still presses an asset
	## slot into a filled pill. That is a momentary press state on a tile, not a
	## `seg` on-state, so §7 does not reach it and this pass leaves it alone.
	b.add_theme_color_override("font_pressed_color", DccTheme.c("accent"))
	var pad := _ps(13)
	b.add_theme_stylebox_override("normal", _chip_box(on, pad))
	b.add_theme_stylebox_override("hover", _chip_box(on, pad))
	b.add_theme_stylebox_override("pressed", _chip_box(true, pad))
	b.pressed.connect(on_press)
	return b

## The chip's box: `DccTheme.pill()`'s geometry, the **washed** on-state's
## colours.
##
## `DccWidgets.set_segment_on()` has built exactly this for every desktop and
## tablet segment since 2026-09-06 -- `accent_wash_2` fill, `accent` ink,
## `accent` border -- under the owner's §7 ruling in `LARGE_ITEM_RULINGS.md`:
## *"Resolved to the WASHED treatment — `accent_wash_2` fill, `accent` ink,
## border"*. This file was the one `seg` surface left drawing the **filled**
## alternative, which is what `GUI_GAP_REGISTER.md` §48 (DS-02) removed
## shell-wide and what `dcc_widgets.gd::set_mode_segment_on()` reserves for the
## tool bar's three SCULPT / PAINT / MEASURE segments *"and nothing else"*.
##
## **Not a call to `set_segment_on()`.** That function carries its own 8/3
## padding and its square-cornered `box()`; a phone chip is a fully rounded
## 44 dp target at radius `_ps(20)`, and the ruling's whole point is that
## nothing about the chip's geometry moves. Only the three colours are shared.
##
## **`accent_wash_2` (.16) and not `accent_wash` (.09)**, because this is a
## segment on-state, which is the line `dcc_theme.gd`'s own token comment draws
## between the two: `var(--wash2)` is 36 uses in the desktop prototype and every
## one a segment or toggle on-state, `var(--wash)` is 13 and every one a hover
## or a list-row selection. **The Android canvas cannot settle that on its own**
## -- it declares a single `--wash`, `rgba(224,163,74,.14)` at `AND:31` and
## `rgba(164,101,15,.10)` at `AND:1469`, and spends it on both jobs because it
## has no `--wash2` to spend. The shell's two weights bracket those two figures.
##
## The fill is transparent-black rather than absent on the off chip, and the
## border is `c("border")` there, both straight from `pill(false)`.
func _chip_box(on: bool, pad: int) -> StyleBoxFlat:
	var sb := DccTheme.pill(false, _ps(20), pad, _ps(9))
	if on:
		sb.bg_color = DccTheme.c("accent_wash_2")
		sb.border_color = DccTheme.c("accent")
	return sb

## §6.6's `range`: a label, a right-hand display, and a full-width slider with
## no steppers. Used **only** where the underlying quantity really is
## continuous -- see the row-type table in this file's header.
##
## `step` defaults to 1, which is what the Year row and every ladder-index row
## want. It is a **parameter** rather than a constant because Crowding is
## `CivilizationWorkspace.LM_CROWDING_STEP` (0.05) and a slider stepping by 1
## over a 0.25..2.00 range has three usable positions -- the same defect class
## as drawing a range over a set, one layer down.
##
## **`on_release` is the write; `fmt` is what moves under the finger.**
## `DccWidgets.slider()` has had that split since it was built and the landmark
## rows need it for a reason stronger than cost: `landmark_set_cap()` and
## `landmark_set_armed()` on every drag frame means a drag from rung 5 to rung 8
## writes six intermediate caps and, if it passes the zero stop, disarms and
## re-arms the type on the way. `fmt` is called with the slider's value and
## returns the readout text, so the number tracks the finger with no engine
## write at all; `on_release` is called once, with the final value.
##
## `on_change` stays for the one caller whose write really is per-frame -- the
## `sim` screen's Year cursor, where the map is meant to follow the drag.
func _slider_row(label: String, display: String, lo: float, hi: float, value: float,
		on_change: Callable, step: float = 1.0,
		on_release: Callable = Callable(), fmt: Callable = Callable()) -> Control:
	var wrap := MarginContainer.new()
	wrap.add_theme_constant_override("margin_left", _ps(16))
	wrap.add_theme_constant_override("margin_right", _ps(16))
	wrap.add_theme_constant_override("margin_top", _ps(8))
	wrap.add_theme_constant_override("margin_bottom", _ps(4))
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", _ps(6))
	wrap.add_child(col)
	var line := HBoxContainer.new()
	line.add_theme_constant_override("separation", _ps(10))
	col.add_child(line)
	var cap := DccTheme.mono_label(label, "text_dim", _ps(10), 0)
	cap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	line.add_child(cap)
	var val := DccTheme.mono_label(display, "text_bright", _ps(11), 0)
	val.mouse_filter = Control.MOUSE_FILTER_IGNORE
	line.add_child(val)
	var s := HSlider.new()
	s.min_value = lo
	s.max_value = hi
	s.step = step
	s.value = value
	s.focus_mode = Control.FOCUS_NONE
	## The grab region, not the drawn track: a 4 dp rail is not a touch target,
	## and the slider's own height is what the finger has to find.
	s.custom_minimum_size.y = _pt(44)
	## **And the gesture arbitration, because this screen scrolls.** Every row
	## this function builds lands in `_screen_scroll`, whose
	## `vertical_scroll_mode` is `SCROLL_MODE_AUTO` -- so without this a vertical
	## swipe that begins on the Year cursor, the Crowding dial or a landmark cap
	## rewrites that value instead of scrolling, and Godot's `Slider::gui_input`
	## does it on the touch-DOWN, before there is any motion to classify.
	##
	## Attached here rather than by `DccShell.phone_fit()`, which is where the
	## other 242 hazardous sliders get it: `PhoneMenu` is parented to
	## `_phone_root` and not to `left_dock`/`right_dock`, so
	## `_on_phone_node_added()` never routes it to `_run_phone_dock_fit()` and
	## no `phone_fit()` call site names this file. Checked rather than assumed --
	## `grep -n phone_fit shell/phone_menu.gd` is empty, 2026-09-07.
	##
	## `_ps(8)`, not `_pt(8)`: 8 dp of TRAVEL in this screen's own pixels. `_pt`
	## is the tap-target floor and applying it to a slop would demand 44 dp of
	## movement before a drag was believed.
	DccWidgets.touch_slider(s, float(_ps(8)))
	s.value_changed.connect(func(v: float):
		if fmt.is_valid():
			val.text = String(fmt.call(v))
		if on_change.is_valid():
			on_change.call(v))
	if on_release.is_valid():
		## `drag_ended`, not `mouse_exited`: on the handset a finger produces
		## `InputEventScreenTouch` and `HSlider` reports the end of that drag
		## through `drag_ended` only. It fires for the mouse too, so one
		## connection covers both and a second would double the write.
		##
		## `s` is captured so the final value is read off the control rather
		## than off `drag_ended`'s argument, which is a *changed* flag and not a
		## value.
		s.drag_ended.connect(func(_changed: bool): on_release.call(s.value))
	col.add_child(s)
	return wrap

## Deliberately does **not** fire `about_to_popup` to build the preview. A
## preview needs names and a count, both static; firing it would run that
## menu's handler on every render of the screen the row is on -- and
## `Preferences ▸ Devices` walks every `wgpu` backend in its own, which is the
## enumeration cost `menus.gd` moved behind a first-open for a reason. The
## refresh happens when the row is *entered* (`_fill_popup`), which is when the
## desktop does it too.
##
## Since the 2026-09-05 ruling this draws only the root's `Not on the MORE list`
## band -- `Edit` and `Window`. **It had two other callers until 2026-09-06**,
## the root's `Asset library` and `Help & about` drill rows; both are bespoke
## screens now, so this is the fallback band's builder and nothing else.
func _menu_row(mb: MenuButton) -> Control:
	var popup := mb.get_popup()
	var title := String(mb.text)
	var count := 0
	var names := PackedStringArray()
	for i in popup.item_count:
		if popup.is_item_separator(i):
			continue
		count += 1
		if names.size() < 3 and not popup.is_item_disabled(i):
			names.append(_clean(popup.get_item_text(i)))
	var subtitle := " · ".join(names)
	return _row(title, subtitle, _trail_label("%d" % count), _chevron(),
		func(): _push(popup, title, SCREEN_LEVEL), false)

# -- One popup, re-presented --------------------------------------------------

func _fill_popup(body: VBoxContainer, step: _Step) -> void:
	var p := step.popup
	## Live exactly as the desktop menu is live: Recent worlds, GPU devices,
	## Open windows and the Preferences busy-lock all populate here.
	p.about_to_popup.emit()

	var first := true
	for i in p.item_count:
		if p.is_item_separator(i):
			## Canvas: "L3 stays a titled band, never a disclosure." Every
			## separator `menus.gd` writes is unlabelled today, so this is the
			## rule-and-gap the desktop draws; give one text and it becomes a
			## caption with no change here.
			if first:
				continue
			body.add_child(_band(_clean(p.get_item_text(i))))
			continue
		if not first:
			body.add_child(DccTheme.rule())
		first = false
		body.add_child(_popup_row(p, i))

## **Every drill inside the MORE stack pushes `SCREEN_LEVEL`, not one more than
## its parent.** The 2026-08-25 grammar this file shipped with gave L4 -- one
## submenu deep -- a 60%-cap sheet over a scrim, and L5 a full screen again, so
## walking Preferences > Lighting rig defaults > Azimuth changed presentation
## twice on the way down. §5 makes the sheet the container for the *whole* MORE
## tab and §6.6 makes everything inside it one navigation stack (*"`moreStack`,
## starts `['root']`. The back button pops one level."*), so every level below
## the root now draws the same way.
##
## The 60%-cap sheet is not gone: `open_sheet()` still pushes level 4 for the
## map context menu, which really is an overlay over an unrelated surface. That
## is the one caller left, and `_build_sheet()` exists for it.
const SCREEN_LEVEL := 3

func _popup_row(p: PopupMenu, i: int) -> Control:
	var text := _clean(p.get_item_text(i))
	var disabled := p.is_item_disabled(i)
	var sub_name := p.get_item_submenu(i)

	if sub_name != "" and not disabled:
		var sub := p.get_node_or_null(NodePath(sub_name)) as PopupMenu
		if sub != null:
			## No `about_to_popup` here either -- see `_menu_row()`.
			var names := PackedStringArray()
			var count := 0
			for j in sub.item_count:
				if sub.is_item_separator(j):
					continue
				count += 1
				if names.size() < 3 and not sub.is_item_disabled(j):
					names.append(_clean(sub.get_item_text(j)))
			var subtitle := " · ".join(names)
			if subtitle == "":
				subtitle = _count_label(count)
			return _row(text, subtitle, _trail_label("%d" % count), _chevron(),
				func(): _push(sub, text, SCREEN_LEVEL), false)

	## An item the port cannot honour is added disabled with the reason in its
	## tooltip (`menus.gd`'s own honesty rule). A phone has no hover, so the
	## reason is drawn as the row's second line instead of being unreachable --
	## this surface is *more* legible than the desktop one, not less.
	if disabled:
		## ...but a disabled *radio* row is still one of a group, and dropping
		## its mark demotes "unselected choice" to "dim line of text" -- a
		## reading the desktop popup never gives, since it keeps drawing the
		## bullet either way. Draw the mark first, then dim. Radio only: a
		## disabled `_switch()` pill reads as an operable toggle, which is the
		## opposite of what the honesty rule above is for.
		var off_mark: Control = _radio(p.is_item_checked(i)) if p.is_item_radio_checkable(i) else null
		return _row(text, p.get_item_tooltip(i), null, off_mark, Callable(), true)

	if p.is_item_checkable(i) or p.is_item_radio_checkable(i):
		var on := p.is_item_checked(i)
		var mark: Control = _radio(on) if p.is_item_radio_checkable(i) else _switch(on)
		return _row(text, p.get_item_tooltip(i), null, mark,
			func(): _activate(p, i, true), false)

	return _row(text, p.get_item_tooltip(i), null, null,
		func(): _activate(p, i, false), false)

## Fire the item exactly as a pointer would: the two signals `menus.gd` is
## connected to, with the item's own id, so every handler, `bind` and
## `set_item_checked()` write in that file runs unchanged.
##
## **Not** `PopupMenu.activate_item()`. That was the first implementation and it
## is a Godot 3 method -- Godot 4 removed it, and nothing said so until the
## build was on a real handset: navigation rows (which never touch this
## function) worked, every *action* row silently did nothing, and the only
## evidence was `adb logcat`'s "Invalid call. Nonexistent function
## 'activate_item' in base 'PopupMenu'". Recorded because it is precisely the
## failure a headless or editor-only check cannot produce.
##
## `stay` keeps the menu open for a checkable row, so the toggle is *seen* to
## move; everything else closes, matching a desktop popup dismissing on
## selection (and, for the many items that open a dialog, getting the menu out
## of the way of it).
func _activate(p: PopupMenu, index: int, stay: bool) -> void:
	if index < 0 or index >= p.item_count or p.is_item_disabled(index) 			or p.is_item_separator(index):
		return
	p.id_pressed.emit(p.get_item_id(index))
	p.index_pressed.emit(index)
	if stay:
		_render()
	else:
		close()

# -- Row construction ---------------------------------------------------------
#
# A row is a `PanelContainer` with its own `gui_input`, not a `Button` with an
# anchored child. The difference matters: a disabled item draws its reason as a
# wrapped second line, and only a container-parented label makes the row grow to
# fit it -- a `Button` sizes from its own text and would clip.

const _META_PRESSED := "pressed"
const _META_PRESS_POS := "press_pos"

## `lead` is §6.6's `nav` glyph column -- *"Glyph `12px mono`, `width:18px`,
## centred"* -- and it is a seventh, optional argument rather than a reuse of
## `trail` because the two sit on opposite sides of the row: the glyph opens the
## line, the count and the chevron close it.
func _row(title: String, subtitle: String, trail: Control, mark: Control,
		on_press: Callable, dim: bool, lead: Control = null) -> Control:
	var row := PanelContainer.new()
	row.add_theme_stylebox_override("panel", DccTheme.empty())
	## Canvas TARGETS: "52 dp list rows". A row grows past it when its text
	## wraps; it never shrinks below it.
	row.custom_minimum_size.y = _pt(DccTheme.H_PHONE_ROW)
	row.tooltip_text = subtitle
	if on_press.is_valid():
		row.mouse_filter = Control.MOUSE_FILTER_STOP
		row.gui_input.connect(_row_input.bind(row, on_press))
	else:
		row.mouse_filter = Control.MOUSE_FILTER_IGNORE

	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _ps(16))
	pad.add_theme_constant_override("margin_right", _ps(16))
	pad.add_theme_constant_override("margin_top", _ps(9))
	pad.add_theme_constant_override("margin_bottom", _ps(9))
	pad.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(pad)

	var line := HBoxContainer.new()
	line.add_theme_constant_override("separation", _ps(12))
	line.mouse_filter = Control.MOUSE_FILTER_IGNORE
	pad.add_child(line)

	if lead != null:
		line.add_child(lead)

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", _ps(3))
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	col.mouse_filter = Control.MOUSE_FILTER_IGNORE
	## Sentence case and the prose face, not the rail's tracked mono: the canvas
	## sets its list rows in the UI sans ("Tectonics", "Droplet hydraulic") and
	## keeps mono for the meta line. Upper-casing here would also destroy the
	## real menu text, which is the one thing this file must not touch.
	var t := DccTheme.label(title, "text_ghost" if dim else "text", _ps(13))
	t.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	col.add_child(t)
	if subtitle.strip_edges() != "" and not _is_bare_path(subtitle):
		## `font:9.5px 'IBM Plex Mono';color:#5f6468` -- `text_ghost`, one step
		## quieter than the `text_faint` this used, on every drill row the canvas
		## draws a second line on.
		var s := DccTheme.mono_label(_shorten(subtitle), "text_ghost", _ps(9.5), 0)
		s.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		col.add_child(s)
	line.add_child(col)

	if trail != null:
		line.add_child(trail)
	if mark != null:
		line.add_child(mark)
	return row

## Press feedback without a `Button`: the same accent wash `DccTheme.active_row`
## gives every other pressed surface in the shell, applied to the row's own
## `panel` stylebox (one of the names `_recolor_subtree()` walks, so it stays
## correct across a theme switch).
##
## **PH-15, registered 2026-08-25, fixed 2026-09-03.** A real `BaseButton`
## cancels its own pending press when a `ScrollContainer` recognises a drag
## past its `scroll_deadzone` -- `dcc_shell.gd::phone_fit()`'s own PH-05
## comment names the notification that does it. This row is not a
## `BaseButton` (this function's own header explains why), so it never got
## that for free: a flick that started on a row fired `on_press` regardless of
## how far the finger had travelled by the time it lifted, three device
## flicks in a row opening *Working set…* (`GUI_GAP_REGISTER.md` §50/§52).
## The sheet **does** scroll with the row still at `MOUSE_FILTER_STOP`
## (measured on the handset both times), and `dcc_shell.gd::phone_fit()`'s
## own comment says a row of this class must keep `STOP` regardless -- so the
## fix stays inside this handler rather than touching `row.mouse_filter`.
## `InputEventScreenDrag`/a left-button `InputEventMouseMotion` were simply
## never read before; now, while a press is pending, one that has moved past
## `DccShell.PHONE_SCROLL_DEADZONE` (the same threshold the sheet's own
## `ScrollContainer` scales) cancels the press and clears the tint without
## calling `on_press` -- the eventual `up` then finds `_META_PRESSED` already
## false and falls through the same early-out a genuine tap never reaches.
func _row_input(ev: InputEvent, row: PanelContainer, on_press: Callable) -> void:
	var down := false
	var up := false
	var pos := Vector2.ZERO
	if ev is InputEventMouseButton and (ev as InputEventMouseButton).button_index == MOUSE_BUTTON_LEFT:
		down = (ev as InputEventMouseButton).pressed
		up = not down
		pos = (ev as InputEventMouseButton).position
	elif ev is InputEventScreenTouch:
		down = (ev as InputEventScreenTouch).pressed
		up = not down
		pos = (ev as InputEventScreenTouch).position
	elif row.get_meta(_META_PRESSED, false) and (ev is InputEventScreenDrag
			or (ev is InputEventMouseMotion
				and ((ev as InputEventMouseMotion).button_mask & MOUSE_BUTTON_MASK_LEFT) != 0)):
		pos = (ev as InputEventScreenDrag).position if ev is InputEventScreenDrag \
			else (ev as InputEventMouseMotion).position
		var origin: Vector2 = row.get_meta(_META_PRESS_POS, pos)
		if pos.distance_to(origin) > _ps(DccShell.PHONE_SCROLL_DEADZONE):
			row.set_meta(_META_PRESSED, false)
			row.add_theme_stylebox_override("panel", DccTheme.empty())
		return
	else:
		return
	if down:
		row.set_meta(_META_PRESSED, true)
		row.set_meta(_META_PRESS_POS, pos)
		row.add_theme_stylebox_override("panel", DccTheme.active_row(false))
		return
	if not up or not row.get_meta(_META_PRESSED, false):
		return
	row.set_meta(_META_PRESSED, false)
	row.add_theme_stylebox_override("panel", DccTheme.empty())
	on_press.call()

## `padding:13px 16px 5px;font:9.5px 'IBM Plex Mono';letter-spacing:.2em;
## color:#6f7478` -- the canvas's band on every one of its eight screens.
##
## **Built here rather than through `DccTheme.header()`**, which is a desktop
## helper: its `FS_HEADER` is a raw 9, and the main viewport has no content
## scale, so every band caption in this menu was drawing at 9 *physical* pixels
## -- about half a millimetre on a 510 ppi panel, and legible in a capture only
## if you already knew what it said. Measured at 1080x2400 before this pass:
## STATUS / PROJECT / CONTENT / SYSTEM were four grey smudges.
func _band(title: String) -> Control:
	var wrap := MarginContainer.new()
	wrap.add_theme_constant_override("margin_left", _ps(16))
	wrap.add_theme_constant_override("margin_right", _ps(16))
	wrap.add_theme_constant_override("margin_top", _ps(13))
	wrap.add_theme_constant_override("margin_bottom", _ps(5))
	wrap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	## An unlabelled `add_separator()` has no caption to draw, so the band is the
	## gap plus the hairline the list already puts between rows. Every group in
	## the drawn menus carries its canvas name as of `GUI_GAP_REGISTER.md` MN-14,
	## which is what turned this file's stated shortfall into a caption.
	## `.2em` of 9.5 px is 1.9, and `spacing_glyph` is whole pixels.
	var l := DccTheme.mono_label(title.strip_edges().to_upper(), "text_faint",
		_ps(9.5), 2, true)
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.add_child(l)
	return wrap

func _value_row(title: String, value: String) -> Control:
	return _row(title, "", _trail_label(value), null, Callable(), false)

## A status row whose value is a sentence rather than a readout: it wraps under
## the title instead of being pinned to the right in a mono label.
##
## `_trail_label()` neither wraps nor clips, and a `Label` reports its full text
## width as its minimum size -- so putting the `hint` slot's sentence through
## `_value_row()` would set this row's minimum width past the screen, inside a
## `ScrollContainer` whose horizontal scrolling is disabled. The subtitle slot
## already autowraps and already carries a sentence on every drill row.
##
## That sentence used to be quoted here as `"File ▸ New world… to begin"`. It is
## `DccShell.new_world_route()` as of 2026-09-06 and reads **`MORE ▸ Project ▸
## New world… to begin`** on this composition -- ten characters longer (17 -> 27), and on
## the wrapping side of exactly the trap above. Re-measured rather than argued:
## `_emptyphone_probe.tscn -- --force-touch --dismiss --tab more` reports zero
## descendants of `_screen` whose combined minimum width exceeds 1080.
func _note_row(title: String, text: String) -> Control:
	return _row(title, text, null, null, Callable(), false)

func _trail_label(text: String) -> Label:
	var l := DccTheme.mono_label(text, "text_dim", _ps(11), 0)
	l.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return l

## §6.6's `nav` badge slot: a short word at its own colour, accent by default
## because the only badge this screen set draws is `ARMED`. Separate from
## `_trail_label()` so the count and the badge cannot drift into one another --
## a count is a quantity and a badge is a state.
func _badge(text: String, token: String = "accent") -> Label:
	var l := DccTheme.mono_label(text, token, _ps(9), 1)
	l.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return l

func _chevron() -> Label:
	var l := DccTheme.mono_label(DccIcons.SYMBOLS["expand"], "text_ghost", _ps(14), 0)
	l.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return l

func _radio(on: bool) -> Label:
	var l := DccTheme.mono_label(DccIcons.SYMBOLS["on"] if on else DccIcons.SYMBOLS["off"],
		"accent" if on else "text_ghost", _ps(13), 0)
	l.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return l

## The canvas's own toggle: a 40 x 22 dp pill with an 18 dp knob, accent when
## on. Built from two `PanelContainer`s with rounded `flat()` styleboxes rather
## than a `CheckButton`, because Godot's stock switch texture is a bitmap this
## shell cannot recolour for the light palette.
func _switch(on: bool) -> Control:
	var track := PanelContainer.new()
	track.custom_minimum_size = Vector2(_ps(40), _ps(22))
	track.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	track.mouse_filter = Control.MOUSE_FILTER_IGNORE
	track.add_theme_stylebox_override("panel",
		DccTheme.flat(DccTheme.c("accent") if on else DccTheme.c("line"), _ps(11)))

	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _ps(2))
	pad.add_theme_constant_override("margin_right", _ps(2))
	pad.add_theme_constant_override("margin_top", _ps(2))
	pad.add_theme_constant_override("margin_bottom", _ps(2))
	pad.mouse_filter = Control.MOUSE_FILTER_IGNORE
	track.add_child(pad)

	var line := HBoxContainer.new()
	line.add_theme_constant_override("separation", 0)
	line.mouse_filter = Control.MOUSE_FILTER_IGNORE
	pad.add_child(line)

	var knob := PanelContainer.new()
	knob.custom_minimum_size = Vector2(_ps(18), _ps(18))
	knob.mouse_filter = Control.MOUSE_FILTER_IGNORE
	knob.add_theme_stylebox_override("panel",
		DccTheme.flat(DccTheme.c("bg") if on else DccTheme.c("text_dim"), _ps(9)))

	if on:
		line.add_child(DccTheme.spacer())
		line.add_child(knob)
	else:
		line.add_child(knob)
		line.add_child(DccTheme.spacer())
	return track

## One `menus.gd` submenu label is written `"Asset pack ▸"`, carrying the
## desktop popup's own submenu arrow inside the text. A drill row already draws
## a chevron, so a *trailing* arrow is dropped for display only -- the popup's
## text is never modified, so the desktop menu is untouched.
##
## Two things this deliberately does not do, both found by reading the result on
## the device rather than by reasoning:
##   - It does not strip `&`. An earlier revision did, assuming a mnemonic
##     marker, and turned "Credits & academic principles" into "Credits
##     academic principles". Godot popups have no `&` mnemonics.
##   - It does not strip `▸` anywhere but the end. File's own static note reads
##     "Imports live under Data ▸ Import; asset packs under Assets", where the
##     arrow is a path separator in prose, and a global strip mangled it.
func _clean(text: String) -> String:
	var out := text.strip_edges()
	if out.ends_with("▸"):
		out = out.substr(0, out.length() - 1).strip_edges()
	return out

## A few `menus.gd` tooltips are paragraph-length (the GPU-acceleration row's
## runs to ~300 characters). They earn their place as a row's second line -- a
## phone has no hover, so this is the only place the reason is readable at all
## -- but not at the cost of a row taller than a third of the screen. Cut on a
## word boundary; the full text stays on the row's own `tooltip_text` for a
## desktop-with-a-mouse run of the same build.
const _SUBTITLE_MAX := 150

## "1 item", not "1 items" -- a submenu whose every row is disabled (Recent
## worlds before a project has ever been opened) falls back to this.
func _count_label(n: int) -> String:
	return "%d item" % n if n == 1 else "%d items" % n

## Is this "subtitle" just a filesystem path?
##
## File's STORAGE LOCATIONS rows carry the full root as their tooltip while the
## row TEXT already carries the readable tail (`projects   .../Worlds`). On
## desktop that is right: hover to see where it really is. On a phone the
## tooltip becomes permanent body copy, so the screen printed four lines of
## `/data/data/org.cartalith.walkingskeleton/files/...` under four rows that had
## already said which folder they meant -- a value repeated as an explanation,
## in a location the user cannot open anyway.
##
## Matching a leading `/` or a `C:\` drive letter, not the app id, so it stays
## true if the package name or the storage root ever moves.
func _is_bare_path(text: String) -> bool:
	var t := text.strip_edges()
	if t.find(" ") >= 0:
		return false   ## a sentence that mentions a path is still a sentence
	return t.begins_with("/") or (t.length() > 2 and t[1] == ":" and t[2] == "\\")

## A subtitle for a phone row, from a string written to be a desktop tooltip.
##
## **These are two different kinds of text and were being treated as one.**
## A tooltip is read on demand, in full, hovering; a phone subtitle is read at a
## glance, always, under a title. Dumping the first into the second produced the
## File screen's Autosave row: three dense lines including
## "(world.zip → world.autosave.zip)", cut mid-sentence at "Never overwrites the
## project…", with the sentence that actually mattered lost past the cut.
##
## So take the FIRST SENTENCE rather than the first N characters. A tooltip's
## opening sentence is almost always its summary -- the rest is the caveat a
## hover-reader wanted -- and a whole sentence never ends mid-word. The length
## cut stays as a backstop for a tooltip with no sentence break in it, and only
## then does an ellipsis appear.
func _shorten(text: String) -> String:
	var t := text.strip_edges()
	if t.length() <= _SUBTITLE_MAX:
		return t
	## First sentence, if there is one that is not absurdly long. `. ` rather
	## than `.` so "world.zip" and "0.42" do not read as sentence ends.
	var stop := t.find(". ")
	if stop > 0 and stop <= _SUBTITLE_MAX:
		return t.substr(0, stop + 1)
	var cut := t.substr(0, _SUBTITLE_MAX)
	var space := cut.rfind(" ")
	if space > _SUBTITLE_MAX / 2:
		cut = cut.substr(0, space)
	return cut + "…"

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
## ## Reachability is the constraint, and it is checked, not assumed
##
## §6.6's `help` screen states the rule this build is held to: *"The phone
## reorganises rather than truncates: every desktop function is reachable
## through MAP · GENERATE · PLAN · MORE."* Five screens cannot carry 361 menu
## rows, so the coverage is explicit:
##
## | Program menu | Where it is reached on the phone |
## |---|---|
## | `File` | the `project` screen -- §6.6's four acts and its autosave block first, then **every remaining File row in File's own order** (`_rest_of()`) |
## | `Data` | the `data` screen -- the whole Data popup, whose own `IMPORT`/`EXPORT`/`SOURCES`/`VALIDATION` separators are already §6.6's bands |
## | `Preferences` | the `prefs` screen -- Theme and Units lifted to the top per §6.6, then the rest of the popup in its own order |
## | `Assets` | the root's `Asset library` row (a drill into the real popup) **and** the `civ` screen's `Landmark generation` row (`Assets ▸ Landmark types`) |
## | `Help` | the root's `Help & about` row |
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
	var title: String
	var trail: String
	var level: int

	func _init(p: PopupMenu, t: String, tr: String, lv: int, scr: String = "") -> void:
		popup = p
		title = t
		trail = tr
		level = lv
		screen = scr

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
}

## §6.6's `root` table, in its order, with its glyphs and its sub text.
##
## `t` is what the row goes to: `screen` a bespoke screen id, `menu` a program
## menu re-presented whole, `call` one of `_root_action()`'s destinations.
##
## **Three of the eight are not bespoke screens, and that is the ruling, not a
## shortfall.** The owner named five (Project, Civilization, Data, Simulation,
## Preferences); §6.6's `assets`/`assets-grid`/`asset-slot` and its `help`/
## `gestures` screens were not among them, and this port already has a real
## Asset Library window and a real Help menu behind those two rows. The Travel
## library row likewise opens the real `travel_library_window.gd`, which is a
## whole window rather than §6.6's two mock screens.
const ROOT_ROWS: Array = [
	{"t": "screen", "id": "project", "glyph": "⧉", "label": "Project",
		"sub": "save · recent · storage"},
	{"t": "screen", "id": "civ", "glyph": "◍", "label": "Civilization",
		"sub": "settlement · POI · way tools"},
	{"t": "screen", "id": "data", "glyph": "⇅", "label": "Data manager",
		"sub": "import · export · sources · validation"},
	{"t": "menu", "id": "Assets", "glyph": "▦", "label": "Asset library",
		"sub": "families · slots · packs · landmark types"},
	{"t": "call", "id": "travel_library", "glyph": "≋", "label": "Travel library",
		"sub": "animals · vehicles · vessels · parties"},
	{"t": "screen", "id": "sim", "glyph": "◷", "label": "Simulation",
		"sub": ""},
	{"t": "screen", "id": "prefs", "glyph": "⚙", "label": "Preferences",
		"sub": "theme · units · performance · graphics"},
	{"t": "menu", "id": "Help", "glyph": "?", "label": "Help & about",
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
	b.flat = true
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

func _screen_title(id: String) -> String:
	var row: Array = SCREEN_TITLES.get(id, [])
	return String(row[0]) if row.size() == 2 else id

func _screen_sub(id: String) -> String:
	var row: Array = SCREEN_TITLES.get(id, [])
	return String(row[1]) if row.size() == 2 else ""

## The canvas's `ELDRA · 1.6 GB` -- the world's name beside what it costs.
## Read off the live status slots rather than stored: `top_world` is written as
## `"ELDRA · <seed>"` by `app.gd`, so the name is its head, and `top_mem` is the
## Performance window's own figure. Either half may be empty before a world
## exists, and an empty readout is drawn as nothing rather than as `· `.
func _root_meta() -> String:
	var parts := PackedStringArray()
	var world := _slot("top_world")
	if world != "":
		parts.append(world.split(" · ")[0])
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
func _push_screen(id: String) -> void:
	var trail := PackedStringArray()
	for s in _stack:
		trail.append(s.title)
	_stack.append(_Step.new(null, _screen_title(id), " · ".join(trail), SCREEN_LEVEL, id))
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
		var sub := _screen_sub(screen_step.screen) if screen_step.screen != "" \
			else screen_step.trail
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
		_fill_screen(body, step.screen)
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

func _fill_screen(body: VBoxContainer, id: String) -> void:
	match id:
		"more": _fill_root(body)
		"project": _fill_project(body)
		"civ": _fill_civ(body)
		"data": _fill_data(body)
		"sim": _fill_sim(body)
		"prefs": _fill_prefs(body)
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
				_add(body, _row(label, sub, null, _chevron(),
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
			"call":
				_add(body, _root_action(String(spec.id), label, sub, glyph))

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
## The three screen-owned menus are named here rather than derived, because the
## coverage lives in `_fill_project()` / `_fill_data()` / `_fill_prefs()` as row
## calls and there is nothing to read it off. `Assets` is deliberately NOT among
## them: the root already drills it (`ROOT_ROWS`), so the loop below finds it,
## and listing it twice would hide a future edit that removed that row.
## `_phonemore_reach_probe.gd` is what checks the claim either way.
func _named_in_root(menu_title: String) -> bool:
	if ["File", "Data", "Preferences"].has(menu_title):
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
## **This port's CIVIL tool set is four, and one of the spec's three is not in
## it.** `civilization_workspace.gd::_build_tools()` builds Settlement,
## Territory, Way and Route; the comment directly above it says of POI: *"no
## function anywhere in this workspace drops one, so there is nothing an armed
## POI tool could call. Arming a button with no engine behind it would be the
## fake control this port's own discipline exists to avoid, so it is omitted
## rather than built disabled or wired to a stub."* That reasoning decides this
## screen too -- the row is drawn with the reason, and arms nothing.
##
## `CIV_TOOLS` mirrors that block **by hand**. There is no accessor on the
## workspace to read the list from -- `_build_tools()` passes its four
## dictionaries straight to `DccWidgets.tools_block()` and keeps nothing -- so
## the ids here are a copy and can go stale. `_phonemore_act_probe.gd` arms all
## four through the drawn rows and reads `app.armed_tool` back, which is what
## turns the copy into a checked claim.
const CIV_TOOLS: Array = [
	{"id": "settlement", "label": "Settlement",
		"sub": "tap drops a place · class and snapping from the CIVIL dock"},
	{"id": "territory", "label": "Territory",
		"sub": "drag to claim · commit or discard from the tool options"},
	{"id": "way", "label": "Way", "sub": "taps append waypoints · commit from the dock"},
	{"id": "route", "label": "Route", "sub": "sea and river legs between two ports"},
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

	var assets := _menu_popup("Assets")
	if assets == null:
		_add(body, _missing_row("Landmark generation",
			"The Assets menu is not on this build's menu bar."))
	else:
		var li := _find_sub(assets, "LandmarkTypes")
		if li < 0:
			_add(body, _missing_row("Landmark generation",
				"Assets ▸ Landmark types is not in this build."))
		else:
			var sub := assets.get_node_or_null(NodePath("LandmarkTypes")) as PopupMenu
			_add(body, _row("Landmark generation", _landmark_sub(sub), null, _chevron(),
				func(): _push(sub, "Landmark types", SCREEN_LEVEL), false))

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
func _fill_data(body: VBoxContainer) -> void:
	var p := _menu_popup("Data")
	if p == null:
		_add(body, _missing_row("Data manager",
			"The Data menu is not on this build's menu bar."))
		return
	_rest_of(body, p, {})

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
## §6.6's `TOUCH` head and its `Gesture reference` screen are **not built**.
## That screen is nine claims about what a gesture does on this shell (pan,
## pinch, rotate, double-tap, long-press sample, edge-swipe inspector, sheet
## handle, tab re-tap, undo chip), and writing them down without testing each
## one against this build is exactly the "prose that describes behaviour nobody
## checked" defect. Reported as outstanding rather than guessed at.
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

## A `ROOT_ROWS` entry whose destination is neither a bespoke screen nor a
## program menu: a real window or view, opened by the same call the desktop
## opens it by. Nothing here is a stub and nothing duplicates a `menus.gd`
## handler.
func _root_action(id: String, label: String, sub: String, glyph: Control) -> Control:
	match id:
		"travel_library":
			return _row(label, sub, null, _chevron(), _go_travel_library, false, glyph)
	## Only reachable if `ROOT_ROWS` names a destination this match has no case
	## for. Drawn disabled with the reason, not skipped -- the same honesty rule
	## `menus.gd` follows for an item the port cannot honour, and the
	## alternative is a destination that vanishes with nothing said.
	return _row(label, "No destination is wired to this row.", null, null, Callable(), true, glyph)

## Both halves of the condition `app.gd` puts on its own Recompute button:
## something is actually stale, and this GDExtension build can act on it.
## An older extension answers `stale_stages()` and not
## `recompute_stale_stages()`, and a row that silently does nothing is
## worse than no row -- so the row is absent rather than drawn dead, which
## is the same call the desktop button makes (it hides itself).
##
## `bridge` lives on `DccApp`, not on `DccShell`, so it is fetched by name
## for the same reason `_go_travel_library()` reaches `open_travel_library`
## that way: `DccApp extends DccShell` and `DccShell` builds this file, so a
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

## Simulation is a category *inside* the CIVIL dock, not a domain of its own:
## `civilization_workspace.gd` builds it as "Simulation" (the collapse/recovery
## model), and `app.gd`'s own timeline strip already points at it by that name.
## `select_domain_category()` is the shell's one call for "switch domain and
## open this category", and it `push_warning`s rather than failing silently if
## the category is ever renamed out from under this row.
func _go_simulation() -> void:
	_shell.select_domain_category("civilization", "Simulation")
	_open_left_sheet()

## `_set_sheet_open()` is the phone's dock-sheet opener, and its first act is
## `_close_all_phone_overlays()` -- which closes this menu. So there is no
## `close()` call in the two functions above and there must not be one: it
## would run after the sheet opened and take the sheet down with it.
func _open_left_sheet() -> void:
	_shell._set_sheet_open("left", true)

## The Travel library window (`TRAVEL_LIBRARY_SPEC.md`; the desktop reaches it
## from Data ▸ Travel library…, ⇧L). `open_travel_library()` lives on `DccApp`,
## the subclass that owns the windows, and is reached **by name** rather than by
## type for the same reason `DccShell._pick_phone_tab()` reaches
## `open_journey_planner()` that way: `DccApp extends DccShell` and `DccShell`
## builds this file, so a typed call here would close a class cycle. Guarded
## because `DccShell` is also instantiated bare by the capture probes.
##
## Closed first: the window opens over the map, and leaving the menu underneath
## it would put the user behind a full-screen overlay when they dismiss it.
func _go_travel_library() -> void:
	close()
	if _shell.has_method("open_travel_library"):
		_shell.call("open_travel_library")

## §6.6 `civ`'s `OPEN JOURNEY PLANNER ➔`. Reached by name for the same class
## cycle reason as `_go_travel_library()` above, and it is the same call
## `DccShell._pick_phone_tab()` makes for the PLAN tab -- so this row and that
## tab land on one view, not two.
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
	_chips(body, label, chips)
	for j in rest:
		_add(body, _popup_row(sub, j))

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
	b.add_theme_font_override("font", DccTheme.mono(0))
	b.add_theme_font_size_override("font_size", _ps(10))
	## `accent_ink`, not `bg`: the on-chip's background IS `c("accent")`, and
	## `DccTheme.pill()`'s own comment names `accent_ink` as the ink that reads
	## on it in both palettes. The off-chip sits on the screen's `bg` panel and
	## so takes an ordinary text token.
	b.add_theme_color_override("font_color",
		DccTheme.c("accent_ink" if on else "text_dim"))
	b.add_theme_color_override("font_hover_color",
		DccTheme.c("accent_ink" if on else "text_bright"))
	b.add_theme_color_override("font_pressed_color",
		DccTheme.c("accent_ink" if on else "text_bright"))
	var pad := _ps(13)
	b.add_theme_stylebox_override("normal", DccTheme.pill(on, _ps(20), pad, _ps(9)))
	b.add_theme_stylebox_override("hover", DccTheme.pill(on, _ps(20), pad, _ps(9)))
	b.add_theme_stylebox_override("pressed", DccTheme.pill(true, _ps(20), pad, _ps(9)))
	b.pressed.connect(on_press)
	return b

## §6.6's `range`: a label, a right-hand display, and a full-width slider with
## no steppers. Used **only** where the underlying quantity really is
## continuous -- see the row-type table in this file's header.
func _slider_row(label: String, display: String, lo: float, hi: float, value: float,
		on_change: Callable) -> Control:
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
	s.step = 1.0
	s.value = value
	s.focus_mode = Control.FOCUS_NONE
	## The grab region, not the drawn track: a 4 dp rail is not a touch target,
	## and the slider's own height is what the finger has to find.
	s.custom_minimum_size.y = _pt(44)
	s.value_changed.connect(func(v: float): on_change.call(v))
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
## band -- `Edit` and `Window`. The root's two §6.6 drill rows build their own
## row and pass `refresh:false` to `_menu_popup()` for exactly the same reason.
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
## width as its minimum size -- so putting "File ▸ New world… to begin" through
## `_value_row()` would set this row's minimum width past the screen, inside a
## `ScrollContainer` whose horizontal scrolling is disabled. The subtitle slot
## already autowraps and already carries a sentence on every drill row.
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

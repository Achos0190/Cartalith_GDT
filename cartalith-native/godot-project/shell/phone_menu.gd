extends Control
class_name PhoneMenu

## The phone menu, built to `design/Cartalith Android Phone.dc.html` and, since MS-4, to the mobile shell mockup
## (`design/mobile-shell-2026-10-31/`, `MOBILE_SHELL_PROPOSAL.md` §2).
##
## This replaces `DccShell._build_phone_overflow()`, which reparented the *desktop* menu bar into a 220 px sheet --
## `GUI_GAP_REGISTER.md` §15's four faults: nothing phone-scaled, desktop status chrome squeezing the row into a
## strip, no touch response, and ~41 items behind 15 hover-opened submenus.
##
## ## MENU is the desktop's eight menus, row for row (MS-4; Ruling DT Q2, Ruling DX Q13)
##
## Until MS-4 this file carried §6.6's bespoke screens (the 2026-09-05 ruling: Project, Civilization, Data,
## Simulation, Preferences, then nine more on 2026-09-06 and Tile pyramid on 2026-09-23), a root that opened them
## in §6.6's order, and a fallback band for the menus they did not name. Ruling DX Q13 (owner, 2026-11-01) chose the
## mockup over them: the phone's MENU repeats the desktop menus row for row, so the root (`_fill_root()`) lists the
## live menu bar's buttons in the bar's order and each opens its own popup through `_fill_popup()`. The fourteen
## screens whose only doors were the old root and each other were deleted in MS-4 (see `SCREEN_TITLES`), with every
## row they drew re-homed or dropped by a stated decision in `cartalith-native/drafts/mobile-shell/README.md`.
##
## **No handler, callback or menu id is reimplemented here.** Each row resolves to a real `PopupMenu` item and fires
## through `_activate()`, which emits `id_pressed`/`index_pressed` exactly as a pointer would. A menu `menus.gd` adds
## appears with no edit here, because the rows are read off the live menu bar, never copied.
##
## `about_to_popup` is emitted before any popup is read, so the rows that rebuild themselves on open (Recent worlds,
## the autosave interval, the CPU thread pool state, the undo budget's step counts) are as live here as they are on
## desktop. It is emitted when a menu is *entered* (`_fill_popup()`), never by the root that only names the menus, so
## entering a screen costs what opening that menu costs on desktop, which is the budget `command_index.gd`'s own
## header explains must not be exceeded.
##
## ## What else lives here
##
## The STORY tab's screens (`story-root`, `story`), the touch vocabulary (`gestures`, reached from Help ▸ Gesture
## reference…), the Quick section and the Status band of MENU's root, and the map context card's peek panel and
## sheet. `_phonemore_reach_probe.gd` and `_ms4_probe.gd` read back what the root draws.
##
## ## Row types
##
## §6.6's own vocabulary (`head`, `nav`, `act`, `tog`, `read`, `info`; `seg`, the inline chip row, was retired with the
## Preferences screen by MS-4), mapped onto what a `PopupMenu` item
## already is:
##
## | §6.6 | Here |
## |---|---|
## | `head` | `_band()` -- also what a labelled `add_separator()` renders as |
## | `nav` | `_row()` with a chevron: another screen, a program menu, or a submenu |
## | `act` | a plain item, fired by `_activate()` |
## | `tog` | a `p.is_item_checkable()` item, drawn with `_switch()` |
## | `seg` | retired (MS-4): a radio submenu is an ordinary drill row now (`_popup_row()`) |
## | `read` | a `menus.gd::_readout()` row, or a `DccShell` status slot |
## | `info` | a `menus.gd::_signpost()` row, or a stated absence |
##
## ## Theme
##
## Everything is written from `DccTheme.c()` tokens through `font_color` and the `panel`/`normal`/`hover` styleboxes
## -- the exact override names `DccShell._recolor_subtree()` walks -- so a dark/light switch repaints this surface
## with no second code path. There is one deliberate literal: the sheet's dim scrim, which is `Color(c("bg"), 0.72)`,
## an alpha derivative `DccTheme.remap()` resolves by its RGB half.

## A step on the drill path. `popup` is null only for the root.
##
## `level` is the disclosure level, and it is what decides the presentation:
## 4 is a sheet, everything else is a screen.
class _Step:
	var popup: PopupMenu
	## One of `SCREEN_TITLES`' ids when this step is a bespoke screen, `""` when it
	## is a popup being re-presented. Exactly one of `popup` / `screen` is set;
	## the root is the `"more"` screen and so has neither a popup nor a parent.
	var screen: String
	## The one thing a screen is *about*, when a screen id alone does not say: which `STORY_ROWS` band `story` is
	## showing. Empty for every screen that needs no argument. A plain `String` so the whole navigation stack stays
	## printable, which is what the probes read.
	var arg: String
	var title: String
	var trail: String
	var level: int

	## Records one navigation step: the popup being re-presented (null for a bespoke screen),
	## its title and breadcrumb trail, its level, and the screen id plus argument. Besides the
	## root, exactly one of `popup` / `screen` should be set. Holds data only; draws nothing.
	func _init(p: PopupMenu, t: String, tr: String, lv: int, scr: String = "",
			ar: String = "") -> void:
		popup = p
		title = t
		trail = tr
		level = lv
		screen = scr
		arg = ar

	## True for the 60%-cap sheet presentation (level 4), which only `open_sheet()` pushes
	## (the map context menu). Every MORE screen uses `SCREEN_LEVEL` and is not a sheet.
	func is_sheet() -> bool:
		return level == 4

## The title and subtitle of every screen this file still builds (§6.6's `_moreTitle()` columns, trimmed by MS-4 to the
## three screens that remain). `more` is MENU's root: its subtitle names what the root now holds.
const SCREEN_TITLES: Dictionary = {
	## MS-4 (Ruling DT Q2, Ruling DX Q13): the bar's last tab is MENU, and its root is the eight desktop menus row
	## for row (`_fill_root()`). The key stays `more` so `open()` and every probe that pushes it keep working.
	"more": ["Menu", "the desktop menus · quick · status"],
	## MS-2: the STORY tab's screen (draft 05's `story-root`), the four `STORY_ROWS` bands.
	"story-root": ["Story", "time · journeys · conflicts · notes"],
	## Help ▸ Gesture reference… (`menus.gd::_build_gesture_reference_row()`, draft 09) is this screen's only door.
	## The other fourteen §6.6 screens (`project`, `civ`, `data`, `data-tiles`, `sim`, `prefs`, `assets`, `assets-grid`,
	## `asset-slot`, `travel`, `travel-item`, `landmarks`, `lm-fam`, `help`) were deleted by MS-4, each row of them
	## re-homed in the desktop menus (`drafts/mobile-shell/README.md`, "Where each phone-only row goes").
	"gestures": ["Gesture reference", "the touch vocabulary this build has"],
}



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

## The `DccShell` this menu serves, assigned in setup; the source of `phone_scale()` and of every action a row calls.
var _shell: DccShell
## The shell's `phone_scale()` captured at setup; `_ps()` and `_pt()` multiply dp values by it.
var _scale := 1.0
var _stack: Array = []  ## of `_Step`; untyped because a typed `Array[_Step]`
	## over an inner class is not portable across GDScript versions.

## The full-screen drill-down panel (the menu root and its child screens).
var _screen: PanelContainer
## The screen header's title label.
var _screen_head_title: Label
## The screen header's breadcrumb line; shown only below the root.
var _screen_head_trail: Label
var _screen_head_meta: Label   ## The root's right-hand `ELDRA · 1.6 GB`.
## The header's back button; visible only below the root.
var _screen_back: Button
## The header's close button; visible only below the root.
var _screen_close: Button
## The scroller around `_screen_body`.
var _screen_scroll: ScrollContainer
## The column rebuilt for each screen.
var _screen_body: VBoxContainer

## The scrim behind the bottom sheet; shared with the peek card (see the CM-5 block below).
var _sheet_scrim: ColorRect
## The bottom sheet panel, used for popup menus in the stack.
var _sheet: PanelContainer
## The sheet header's title label.
var _sheet_head_title: Label
## The sheet header's trail label (text from the top `_Step`'s `trail`).
var _sheet_head_trail: Label
## The scroller around `_sheet_body`.
var _sheet_scroll: ScrollContainer
## The column rebuilt for each sheet.
var _sheet_body: VBoxContainer

## CM-5 (`MAP_CONTEXT_SCOPE.md` §8.1, §9.1): the phone's own noun-surface
## sheet -- peek (chip row) / half (the full sectioned card, 46 dp rows).
## Shares `_sheet_scrim` above rather than a second scrim: `_close_all_phone_
## overlays()` already guarantees only one of `_sheet` / `_peek_panel` is ever
## shown at a time (`phone_present_peek_card()`, `dcc_shell.gd`).
var _peek_panel: PanelContainer
## The peek card's grab handle; its `gui_input` runs `_on_peek_grab_input`, and its rect is what the broker may ask for.
var _peek_grab: Control
## The peek card's title label.
var _peek_title: Label
## The peek card's second line (for example `Map · cell x, y`).
var _peek_trail: Label
## Margin wrapper for the chip row; visible only at the `peek` detent.
var _peek_chip_wrap: MarginContainer
## The chip row of primary actions shown at the `peek` detent.
var _peek_chip_row: HBoxContainer
## The scroller around `_peek_full_col`; visible only at the `half` detent.
var _peek_full_scroll: ScrollContainer
## The full sectioned action list (or, in select mode, the hit list).
var _peek_full_col: VBoxContainer
## True while the peek card is shown (set when it opens, cleared in dismissal).
var _peek_open := false
var _peek_detent := "peek"          ## "peek" or "half".
var _peek_drag: Dictionary = {}     ## the grab handle's own press/move state.
## The running detent-height tween, if any; killed before a new tween or a drag starts.
var _peek_tween: Tween
## The action list of the current request, kept so the full rows can be rebuilt on a detent change.
var _peek_actions: Array = []
## Callback passed in by the context broker; called with a hit when the user taps one in select mode.
var _peek_reselect: Callable = Callable()
## Callback the broker passes in to be told when the card closes; called once and then cleared.
var _peek_dismiss_cb: Callable = Callable()
## CM-5 residual (`OUTSTANDING_WORK.md`'s "CM-5 residuals" row): §8.1.4's
## multi-hit "Select ▸" chip. `_peek_all_hits` is `req.get("all_hits", ...)`,
## the same field `context_card.gd::open()` reads for its own Select ▸ list;
## `_peek_primary` is `req["hits"][0]`, marking which hit the sheet is
## currently about. `_peek_select_mode` swaps `_peek_full_col` from the
## action rows to the hit list -- forced to the `half` detent, since a
## `peek`-height sheet has no room for a list.
var _peek_all_hits: Array = []
## The hit the card is currently about (`req["hits"][0]`), per the block above; empty when none.
var _peek_primary: Dictionary = {}
## True while `_peek_full_col` shows the hit list instead of the action rows.
var _peek_select_mode := false
## `context_card.gd`'s own SELECT_CAP (§9.1: "nearest first, capped at 8") --
## read off that script rather than a re-declared literal, so the two lists
## cannot drift apart.
const _ContextCard := preload("res://shell/context_card.gd")
## `06-phone.md` §5.2's own peek figure -- the same 66 dp the tool sheet's
## `DccShell.PHONE_DETENT_PEEK` uses, kept as this file's own constant rather
## than a cross-file reference so this sheet does not depend on the tool
## sheet's detent system, which it does not share (this one has two detents,
## not three, and sizes "half" to the card's own content instead of a screen
## fraction).
const _PEEK_CARD_PEEK_DP := 66.0
## "Sheets stop at 60% height" -- this file's own established rule
## (`_build_sheet()` above draws it as a fixed `anchor_top = 0.4`); the half
## detent borrows it as a ceiling instead of a fixed height, so a short list
## does not leave a wall of empty sheet under it.
const _PEEK_CARD_HALF_CAP_FRAC := 0.6
const _PEEK_CARD_ANIM := 0.22  ## Seconds for the detent height tween (no source found at this line).
const _PEEK_CARD_MIN_DRAG_DP := 24.0  ## Declared with a 24 dp value and not referenced anywhere in this file; intent unverified (presumably a drag threshold).

# -- Geometry ----------------------------------------------------------------
#
# The same two helpers `DccShell` uses for its own phone chrome, over the same
# `phone_scale()`. Duplicated as three lines rather than reached for across the
# file boundary because `_pscale`/`_ptap` are private there and this is the only
# thing this file needs from them.

## A dp value scaled by the shell's phone scale (`_scale`), at least 1 px. For things that are only
## drawn; use `_pt` for anything a finger must hit.
func _ps(px: float) -> int:
	return maxi(1, int(round(px * _scale)))

## Like `_ps` but never below `DccTheme.PHONE_TAP_MIN`: use it for the height of anything
## a finger must hit, and `_ps` for everything that is only drawn.
func _pt(px: float) -> int:
	return maxi(DccTheme.PHONE_TAP_MIN, _ps(px))

# -- Build --------------------------------------------------------------------

## Binds the MORE overlay to its `DccShell`, captures `phone_scale()` into `_scale`, makes the node
## full-rect with an IGNORE mouse filter while closed and builds the screen, sheet and peek panel. Called
## once from `dcc_shell.gd`. Opening is `open()`.
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

	_peek_panel = _build_peek_panel()
	add_child(_peek_panel)

## Builds the full-screen MORE panel once: a header (back, title over breadcrumb, the
## root's right-hand readout, close) above a hairline and a vertical-only scroll area
## whose body `_render()` refills. Its mouse filter is STOP so a tap cannot fall through
## to the map. It holds no content of its own.
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

## CM-5 (`MAP_CONTEXT_SCOPE.md` §8.1, §9.1): the phone's own noun-surface
## sheet, built the same way `_build_sheet()` above is (a `PanelContainer`
## anchored to the bottom edge), except its height is not a fixed anchor -- it
## is an `offset_top` this file animates and drags between the two detents,
## so a grab-drag can track a finger the same way `dcc_shell.gd`'s tool sheet
## already does for its own three detents (`_on_phone_sheet_grab_input()`,
## read for the pattern this reuses).
func _build_peek_panel() -> PanelContainer:
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DccTheme.panel("raised", {"top": 1}))
	panel.anchor_left = 0.0
	panel.anchor_right = 1.0
	panel.anchor_top = 1.0
	panel.anchor_bottom = 1.0
	panel.offset_bottom = 0.0
	panel.offset_top = -_ps(_PEEK_CARD_PEEK_DP)
	panel.visible = false

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	panel.add_child(col)

	_peek_grab = Control.new()
	## Tighter than `_build_sheet()`'s own 18 dp handle strip: at `peek`'s 66 dp
	## total, the grab strip, the header and the chip row all have to fit in
	## the SAME band the tool sheet gives a title and one options row (see
	## `_phone_map_sheet_content_height()`'s own comment in `dcc_shell.gd`) --
	## there is no room here for a tap-target-sized handle, only a grabbable one.
	_peek_grab.custom_minimum_size.y = _ps(14)
	_peek_grab.mouse_filter = Control.MOUSE_FILTER_STOP
	_peek_grab.gui_input.connect(_on_peek_grab_input)
	var handle := ColorRect.new()
	handle.color = Color(DccTheme.c("text_ghost"), 0.55)
	var hw := _ps(34)
	var hh := _ps(4)
	handle.set_anchors_preset(Control.PRESET_CENTER)
	handle.size = Vector2(hw, hh)
	handle.position = Vector2(-hw / 2.0, -hh / 2.0)
	handle.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_peek_grab.add_child(handle)
	col.add_child(_peek_grab)

	## No forced `custom_minimum_size` here (unlike `_build_sheet()`'s own
	## `_pt(48)` head, which has a whole 60% sheet to spend it in): at `peek`
	## this row sizes to its two lines of text and nothing more, or the grab
	## strip + header + chip row together overrun `peek`'s 66 dp budget and
	## the panel silently grows past it (measured: it did, by 9 px at
	## 1080x2340, the first time this was built with a `_pt(32)` floor here).
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", _ps(12))
	var titles := VBoxContainer.new()
	titles.add_theme_constant_override("separation", _ps(1))
	titles.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	titles.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	titles.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_peek_title = DccTheme.mono_label("Here", "text_bright", _ps(11), 2, true)
	_peek_trail = DccTheme.mono_label("", "text_faint", _ps(8), 0)
	titles.add_child(_peek_title)
	titles.add_child(_peek_trail)
	head.add_child(titles)
	var hp := MarginContainer.new()
	hp.add_theme_constant_override("margin_left", _ps(16))
	hp.add_theme_constant_override("margin_right", _ps(12))
	hp.add_theme_constant_override("margin_top", _ps(2))
	hp.add_theme_constant_override("margin_bottom", _ps(2))
	hp.add_child(head)
	col.add_child(hp)

	## §8.1.2: "one horizontal row of up to four action chips" -- shown at
	## `peek`, hidden at `half`, where the full sectioned list below replaces it
	## ("one provider list and one definition in two presentations", PH-02's own
	## rule, carried forward from the desktop card to this sheet).
	_peek_chip_row = HBoxContainer.new()
	_peek_chip_row.add_theme_constant_override("separation", _ps(8))
	_peek_chip_wrap = MarginContainer.new()
	_peek_chip_wrap.add_theme_constant_override("margin_left", _ps(16))
	_peek_chip_wrap.add_theme_constant_override("margin_right", _ps(16))
	_peek_chip_wrap.add_theme_constant_override("margin_bottom", _ps(4))
	_peek_chip_wrap.add_child(_peek_chip_row)
	col.add_child(_peek_chip_wrap)

	## §8.1.3: "the full card appears, in the same sections, as 46 dp rows".
	_peek_full_scroll = ScrollContainer.new()
	_peek_full_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_peek_full_scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	_peek_full_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_peek_full_scroll.add_theme_stylebox_override("panel", DccTheme.empty())
	_peek_full_scroll.visible = false
	_peek_full_col = VBoxContainer.new()
	_peek_full_col.add_theme_constant_override("separation", 0)
	_peek_full_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_peek_full_scroll.add_child(_peek_full_col)
	col.add_child(_peek_full_scroll)
	return panel

## Whether the peek/half card is up -- read by `dcc_shell.gd::
## phone_present_peek_card()` so a re-resolve (the pin dragged to a new spot)
## updates this SAME sheet's content in place, at whichever detent the user
## already has it, instead of tearing it down and reopening at `peek` under
## the finger's drag.
func peek_card_is_open() -> bool:
	return _peek_open

## Probe seam, all read-only -- `_ctxphone_probe.gd`'s own introspection,
## the same reason `_card().panel_rect()`/`drawn_rows()` exist on the desktop
## card and `RadialRing.debug_state()` exists on the ring.
func peek_detent() -> String:
	return _peek_detent

## Probe seam (read-only): the global rect of the peek/half card, or an empty Rect2 before
## it is built.
func peek_panel_rect() -> Rect2:
	return _peek_panel.get_global_rect() if _peek_panel != null else Rect2()

## Probe seam (read-only): the global rect of the card's grab handle, or an empty Rect2.
func peek_grab_rect() -> Rect2:
	return _peek_grab.get_global_rect() if _peek_grab != null else Rect2()

## Probe seam (read-only): the card's title text, or "" before it is built.
func peek_title_text() -> String:
	return _peek_title.text if _peek_title != null else ""

## Probe seam (read-only): the labels of the chip buttons shown at the `peek` detent.
func peek_chip_labels() -> PackedStringArray:
	var out := PackedStringArray()
	if _peek_chip_row == null:
		return out
	for c in _peek_chip_row.get_children():
		if c is Button:
			out.append((c as Button).text)
	return out

## Probe seam: whether the sheet is showing §8.1.4's hit list rather than the
## ordinary action rows.
func peek_in_select() -> bool:
	return _peek_select_mode

## Probe seam (read-only): the labels of the row buttons in the `half` detent's list (the
## action rows, or the hit list in select mode). A disabled row's reason line is a Label,
## not a Button, so it is not included.
func peek_full_row_labels() -> PackedStringArray:
	var out := PackedStringArray()
	if _peek_full_col == null:
		return out
	for wrap in _peek_full_col.get_children():
		if wrap is VBoxContainer:
			for c in (wrap as VBoxContainer).get_children():
				if c is Button:
					out.append((c as Button).text)
	return out

## Probe seam: drives the grab handle's own drag exactly as a real finger
## would (`_on_peek_grab_input()` is private, and the handle itself is not a
## public property to push an `InputEvent` through, unlike a `Control` with
## its own `_gui_input`).
func peek_grab_input(event: InputEvent) -> void:
	_on_peek_grab_input(event)

## CM-5's own presenter. `req`/`actions` are `context_broker.gd`'s merged
## rows (`MAP_CONTEXT_SCOPE.md` §3/§9.1's Action shape). `reselect` is
## `context_broker.gd::reselect()`, called from `_reselect_from_peek()` when
## the user picks one entry off §8.1.4's Select ▸ list (`_open_peek_select()`,
## `_build_peek_select_row()`) -- the phone form of the desktop card's own
## Select ▸ submenu, built as a §8.1.3 half-detent row list rather than a
## card-in-card submenu, since the sheet has no second disclosure level to
## put one in. `on_dismiss` fires exactly once, when the sheet closes by the
## user's own action (scrim tap, system back, drag below `peek`) -- never when
## another
## phone overlay simply replaces this one (`close()` above skips it).
func peek_card(req: Dictionary, actions: Array, reselect: Callable,
		on_dismiss: Callable = Callable()) -> void:
	var was_open := _peek_open
	_peek_reselect = reselect
	_peek_dismiss_cb = on_dismiss
	## CM-5 residual: this used to stop at a provider's own `header` string,
	## reading "Here" whenever nothing set one -- fine for a single settlement
	## hit (its rows always set `header`), wrong the moment §8.1.4's Select ▸
	## list (below) re-resolves to a label or icon, whose rows
	## (`cartography_workspace.gd`'s `carto.label_edit` etc.) set no `header`
	## at all. Falls back to the picked hit's own name -- `req["hits"][0]`,
	## which `context_broker.gd::reselect()` narrows to exactly the chosen
	## object -- rather than reading "Here" for a re-resolve that plainly did
	## land on something.
	var hits: Array = req.get("hits", [])
	var title := "Here"
	for a in actions:
		if (a as Dictionary).has("header"):
			title = String(a["header"])
			break
	if title == "Here" and not hits.is_empty():
		title = String(hits[0].get("label", "Here"))
	_peek_title.text = title
	_peek_trail.text = "Map · cell %d, %d" % [int(req.get("gx", 0.0)), int(req.get("gy", 0.0))]
	_peek_actions = actions
	## Every fresh resolve (a new drop, or a Select ▸ pick re-resolving) lands
	## back in the action rows -- `_reselect_from_peek()` below is the only
	## other writer of `_peek_select_mode` and it always clears it itself, but
	## a re-drop while a select list happened to be open must not leave a
	## stale one showing for the NEW hit's own `all_hits`.
	_peek_select_mode = false
	_peek_all_hits = req.get("all_hits", req.get("hits", []))
	_peek_primary = hits[0] if not hits.is_empty() else {}
	_rebuild_peek_chip_row(actions)
	_rebuild_peek_full_rows(actions)
	visible = true
	_peek_open = true
	_sheet_scrim.visible = true
	_peek_panel.visible = true
	## CM-2 residual (`MAP_CONTEXT_SCOPE.md` §9.4: "the card, ring, peek sheet
	## and thumb fan all lack it"). Guarded on `was_open` so a re-resolve (the
	## pin dragged to a new spot, same sheet) does not pause a second time --
	## `_shell` is typed `DccShell`, which does not declare these methods
	## (they are `app.gd`'s `DccApp`, the one subclass that ever constructs
	## this control), so `has_method`/`call` reach them dynamically rather
	## than widening the base class for one caller.
	if not was_open and _shell.has_method("pause_for_context_surface"):
		_shell.call("pause_for_context_surface")
	## A fresh drop always opens at `peek` (§8.1.1/.2); a re-resolve (the pin
	## dragged to a new spot) keeps whatever detent the user already had it at
	## -- re-applied rather than left alone, since `half`'s own height is sized
	## to the NEW content and may have changed.
	_set_peek_detent(_peek_detent if was_open else "peek", false)

## §8.1.4 (`design/map-context-2026-09-25/Phone.dc.html`'s own
## `head.multi`/`openSelect` chip): a multi-hit drop shows *Select ▸ N* as the
## FIRST chip, ahead of the ordinary action chips, which yield one slot for it
## (3 of them, not 4) so the row still fits the same peek band. A single-hit
## drop is unchanged -- the chip is absent and all 4 slots go to actions,
## exactly as before this residual.
func _rebuild_peek_chip_row(actions: Array) -> void:
	for c in _peek_chip_row.get_children():
		c.queue_free()
	var enabled: Array = actions.filter(func(a): return bool((a as Dictionary).get("enabled", true)))
	## SM-4 (`STORY_MENUS_PROPOSAL.md` §6, A8): a row may carry `chip` (its
	## place on this sheet) and `chip_label` (the pill's shorter text). Rows
	## with a `chip` come first in that order, then the rest in card order --
	## so a town's sheet shows Open note · Add event · Journey from here · City
	## viewer rather than Edit · Move viewer · …. Stable: the rest keep their
	## relative order. The full (half-detent) list is untouched.
	var ranked: Array = []
	for n in enabled.size():
		var a: Dictionary = enabled[n]
		ranked.append([int(a.get("chip", 1000)), n, a])
	ranked.sort_custom(func(x, y): return x[0] < y[0] or (x[0] == y[0] and x[1] < y[1]))
	enabled = ranked.map(func(x): return x[2])
	var take := 4
	if _peek_all_hits.size() >= 2:
		take = 3
		DccWidgets.chip(_peek_chip_row, "Select ▸ %d" % _peek_all_hits.size(),
			_open_peek_select, true)
	for a in enabled.slice(0, take):
		var row: Dictionary = a
		var cb: Callable = row.get("callable", Callable())
		DccWidgets.chip(_peek_chip_row, String(row.get("chip_label", row.get("label", ""))), func() -> void:
			_dismiss_peek_card()
			if cb.is_valid():
				cb.call())
	## Four chips share the row's width, eliding their text, rather than
	## running off the right edge: before this, four full-width labels at
	## 393 dp pushed the fourth chip off screen (seen on SM-4's own capture,
	## `_storymenus_probe.gd` phone `peek`; the old Edit / Move viewer labels
	## overran too). §8.1.2 keeps "one horizontal row", so the row stays one
	## row. Every chip expands equally, so none collapses to nothing beside a
	## wider sibling (`MISTAKES.md`, clip_text beside an EXPAND_FILL sibling);
	## the full label stays on the tooltip.
	for c in _peek_chip_row.get_children():
		var b := c as Button
		if b == null or b.is_queued_for_deletion():
			continue
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.clip_text = true
		b.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		b.tooltip_text = b.text

## Rebuilds the `half` detent's list. In select mode: a Back row, up to
## `_ContextCard.SELECT_CAP` nearest hits and a disabled "... and N more" row. Otherwise
## one row per action, with a hairline wherever the action's `section` changes. Clears the
## old rows first, so it is safe to call whenever the card's content changes.
func _rebuild_peek_full_rows(actions: Array) -> void:
	for c in _peek_full_col.get_children():
		c.queue_free()
	if _peek_select_mode:
		_peek_full_col.add_child(_build_peek_back_row())
		var cap: int = _ContextCard.SELECT_CAP
		for n in mini(_peek_all_hits.size(), cap):
			_peek_full_col.add_child(_build_peek_select_row(_peek_all_hits[n]))
		if _peek_all_hits.size() > cap:
			_peek_full_col.add_child(_build_peek_row({
				"label": "… and %d more" % (_peek_all_hits.size() - cap),
				"enabled": false,
				"reason": "the list shows the %d nearest -- zoom in to separate the rest" % cap,
			}))
		return
	var prev := ""
	for n in actions.size():
		var a: Dictionary = actions[n]
		var sec := String(a.get("section", ""))
		if n > 0 and sec != prev:
			_peek_full_col.add_child(DccTheme.rule())
		prev = sec
		_peek_full_col.add_child(_build_peek_row(a))

## §8.1.4's own row, the phone-sheet form of `context_card.gd`'s Select ▸
## submenu (`_activate_focused()`'s `"select"` arm there): capped list of
## the hits under the drop, nearest first, the currently-shown one marked.
## Tapping one calls `_peek_reselect` (`context_broker.gd::reselect()`),
## which re-resolves the whole request narrowed to that hit and re-opens this
## SAME sheet (`peek_card()` above) with the new content -- there is nothing
## for this row to close itself, unlike an ordinary action row.
func _build_peek_select_row(h: Dictionary) -> Control:
	var wrap := VBoxContainer.new()
	wrap.add_theme_constant_override("separation", 0)
	var b := Button.new()
	var mark := " %s" % DccIcons.SYMBOLS["on"] if _same_peek_hit(h, _peek_primary) else ""
	b.text = "%s · %s%s" % [String(h.get("label", "(unnamed)")), String(h.get("kind", "")), mark]
	b.custom_minimum_size.y = _pt(46)
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.focus_mode = Control.FOCUS_NONE
	b.flat = true
	b.add_theme_font_size_override("font_size", _ps(12))
	b.pressed.connect(func() -> void:
		_reselect_from_peek(h))
	wrap.add_child(b)
	return wrap

## The list's own `‹ Back` -- returns to the action rows without picking, the
## phone form of the desktop card's `back` row (`_activate_focused()`'s
## `"back"` arm). Distinct from the sheet's scrim/drag dismissal: this leaves
## the sheet open, on the SAME hit, and only steps out of the sub-list.
func _build_peek_back_row() -> Control:
	var wrap := VBoxContainer.new()
	wrap.add_theme_constant_override("separation", 0)
	var b := Button.new()
	b.text = "‹ Back"
	b.custom_minimum_size.y = _pt(46)
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.focus_mode = Control.FOCUS_NONE
	b.flat = true
	b.add_theme_font_size_override("font_size", _ps(12))
	b.add_theme_color_override("font_color", DccTheme.c("accent"))
	b.pressed.connect(func() -> void:
		_peek_select_mode = false
		_rebuild_peek_full_rows(_peek_actions))
	wrap.add_child(b)
	return wrap

## Enters the card's hit-list (select) mode and raises it to the `half` detent, which is the
## only detent that can show the list. Leaves the card open on the same hit.
func _open_peek_select() -> void:
	_peek_select_mode = true
	_rebuild_peek_full_rows(_peek_actions)
	_set_peek_detent("half")

## A hit-list row was tapped: leave select mode and hand the hit to the broker's reselect
## callback (which re-resolves the card narrowed to it and re-presents this same sheet).
## Does not close the card itself; a callback that is not valid is skipped.
func _reselect_from_peek(h: Dictionary) -> void:
	_peek_select_mode = false
	if _peek_reselect.is_valid():
		_peek_reselect.call(h)

## Whether two hit dictionaries are the same entity (same `kind` and `id`). False when `b`
## is empty. A missing id defaults to different sentinels on each side, so two hits with no
## id never compare equal.
static func _same_peek_hit(a: Dictionary, b: Dictionary) -> bool:
	return not b.is_empty() and a.get("kind") == b.get("kind") and int(a.get("id", -1)) == int(b.get("id", -2))

## §8.1.3's 46 dp row: label left, and a disabled row's reason drawn as a
## second line under it -- `MAP_CONTEXT_SCOPE.md` §4.2's disabled-with-reason
## rule, which the phone's card owes the same as the desktop one.
func _build_peek_row(a: Dictionary) -> Control:
	var enabled: bool = bool(a.get("enabled", true))
	var wrap := VBoxContainer.new()
	wrap.add_theme_constant_override("separation", 0)
	var b := Button.new()
	b.text = String(a.get("label", ""))
	b.custom_minimum_size.y = _pt(46)
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.focus_mode = Control.FOCUS_NONE
	b.flat = true
	b.disabled = not enabled
	b.add_theme_font_size_override("font_size", _ps(12))
	if enabled:
		var cb: Callable = a.get("callable", Callable())
		b.pressed.connect(func() -> void:
			_dismiss_peek_card()
			if cb.is_valid():
				cb.call())
	wrap.add_child(b)
	if not enabled:
		var reason := DccTheme.mono_label(String(a.get("reason", "")), "text_faint", _ps(9), 0)
		## Wrapped: a reason is a sentence, and unwrapped its one-line minimum
		## width widened the whole sheet past the screen (SM-4's "Journey to
		## here" reason measured the half card at 1598 px on a 1080 px screen,
		## `_ctxphone_probe.gd` leg R). The VBox gives it the sheet's width.
		reason.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		reason.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		var rp := MarginContainer.new()
		rp.add_theme_constant_override("margin_left", _ps(16))
		rp.add_theme_constant_override("margin_right", _ps(16))
		rp.add_theme_constant_override("margin_bottom", _ps(6))
		rp.add_child(reason)
		wrap.add_child(rp)
	return wrap

## `peek`'s own fixed height, or `half`'s. `half` is NOT content-measured:
## `_peek_full_col` sits inside `_peek_full_scroll`, a `ScrollContainer`, and a
## `ScrollContainer` deliberately does not fold its child's minimum size into
## its own -- that is the one thing that makes it scroll instead of grow -- so
## reading `get_combined_minimum_size()` back up through it would report a
## small constant no matter how many rows the card has, not the card's real
## height. `half` is instead this file's own established "sheets stop at 60%
## height" rule (`_build_sheet()` above draws the SAME rule as a fixed
## `anchor_top = 0.4`), applied as a height rather than an anchor because this
## panel's own height is a drag target and an anchor cannot be tweened.
func _peek_detent_height(det: String) -> float:
	var peek := float(_ps(_PEEK_CARD_PEEK_DP))
	if det == "peek":
		return peek
	return maxf(peek, get_viewport_rect().size.y * _PEEK_CARD_HALF_CAP_FRAC)

## Moves the card to `peek` (chips only) or `half` (scrolling list): swaps which content is
## visible, kills any running tween, then animates `offset_top` to the detent height
## (or jumps there when `animate` is false). Never changes the card's content.
func _set_peek_detent(det: String, animate: bool = true) -> void:
	_peek_detent = det
	_peek_chip_wrap.visible = (det == "peek")
	_peek_full_scroll.visible = (det != "peek")
	var target := _peek_detent_height(det)
	if _peek_tween != null and _peek_tween.is_valid():
		_peek_tween.kill()
	if not animate:
		_peek_panel.offset_top = -target
		return
	_peek_tween = create_tween()
	_peek_tween.set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	_peek_tween.tween_property(_peek_panel, "offset_top", -target, _PEEK_CARD_ANIM)

## The grab handle's own drag -- `dcc_shell.gd::_on_phone_sheet_grab_input()`'s
## press/move/release triple, over this sheet's two detents instead of three.
## Positions are global `y`, the same reason that function gives: the panel's
## own origin moves as its height changes, so a local `y` would drift under a
## finger that has not moved.
func _on_peek_grab_input(event: InputEvent) -> void:
	var press_state := 0
	var moved := false
	var local_y := 0.0
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index != MOUSE_BUTTON_LEFT:
			return
		press_state = 1 if mb.pressed else -1
		local_y = mb.position.y
	elif event is InputEventScreenTouch:
		var st := event as InputEventScreenTouch
		press_state = 1 if st.pressed else -1
		local_y = st.position.y
	elif event is InputEventMouseMotion:
		moved = true
		local_y = (event as InputEventMouseMotion).position.y
	elif event is InputEventScreenDrag:
		moved = true
		local_y = (event as InputEventScreenDrag).position.y
	else:
		return
	var gy: float = _peek_grab.global_position.y + local_y
	if press_state == 1:
		if _peek_tween != null and _peek_tween.is_valid():
			_peek_tween.kill()
		_peek_drag = {"y0": gy, "h0": -_peek_panel.offset_top}
		_peek_grab.accept_event()
		return
	if _peek_drag.is_empty():
		return
	var lo := float(_ps(_PEEK_CARD_PEEK_DP))
	var hi := _peek_detent_height("half")
	if moved:
		var h: float = clampf(float(_peek_drag["h0"]) - (gy - float(_peek_drag["y0"])), lo, maxf(lo, hi))
		_peek_drag["h"] = h
		_peek_panel.offset_top = -h
		## Track which content is drawn mid-drag by nearest detent, so the sheet
		## does not show an empty gap between the chip row's own height and the
		## full list's while the finger is between the two.
		var at_peek := absf(h - lo) <= absf(h - hi)
		_peek_chip_wrap.visible = at_peek
		_peek_full_scroll.visible = not at_peek
		_peek_grab.accept_event()
		return
	var final_h: float = float(_peek_drag.get("h", _peek_drag["h0"]))
	_peek_drag = {}
	_set_peek_detent("half" if absf(final_h - hi) < absf(final_h - lo) else "peek")

## Closes the card by the user's own action -- scrim tap, system back, or a
## drag down to `peek` and then further (there is nowhere lower than `peek`,
## so a drag never dismisses; only the scrim and back do). Fires
## `_peek_dismiss_cb` exactly once, which is how `context_broker.gd` clears
## `map_overlay.gd`'s sample pin when its own sheet goes away.
func _dismiss_peek_card() -> void:
	if not _peek_open:
		return
	_peek_open = false
	_peek_panel.visible = false
	_sheet_scrim.visible = false
	visible = false
	## CM-2 residual (§9.4) -- the resume matching `peek_card()`'s own pause.
	if _shell.has_method("resume_from_context_surface"):
		_shell.call("resume_from_context_surface")
	var cb := _peek_dismiss_cb
	_peek_dismiss_cb = Callable()
	if cb.is_valid():
		cb.call()

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
	_peek_panel.offset_left = left
	_peek_panel.offset_right = 0
	_peek_panel.offset_bottom = -bottom

# -- Navigation ---------------------------------------------------------------

## True while the MORE overlay is visible. Read by `dcc_shell.gd`, for example to close it when a phone tab is chosen.
func is_open() -> bool:
	return visible

## Opens the MORE root: resets the navigation stack to the single "more" screen at level 2,
## shows the overlay and renders. Level 1 is the bottom bar, which this file does not own.
## Does not close any other overlay; the shell does that before calling.
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
	if id == "story":
		return String(_story_row(arg).get("label", "Story"))
	var row: Array = SCREEN_TITLES.get(id, [])
	return String(row[0]) if row.size() == 2 and String(row[0]) != "" else id

## The breadcrumb/subtitle for a screen id: a live value for the three screens that depend
## on an argument (asset family, asset slot, travel entry), otherwise the second column of
## `SCREEN_TITLES`, or "" when the id has none.
func _screen_sub(id: String, arg: String = "") -> String:
	match id:
		"story":
			return String(_story_row(arg).get("sub", ""))
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
## (`context_broker.gd`'s `popup` since CM-1, opened by a press-and-hold
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

## Hides the overlay and clears the stack. If the peek/half card is up it is also torn down,
## WITHOUT calling its dismiss callback, because this is the shell swapping one overlay for
## another rather than the user dismissing the card; the shell is told to resume instead.
func close() -> void:
	visible = false
	_stack.clear()
	## CM-5: a fresh `open()`/`open_sheet()` while the peek/half card is up
	## (`_close_all_phone_overlays()` calls this before every phone overlay,
	## `phone_present_peek_card()` included) must not leave it drawn behind
	## whatever opens next. Not routed through `_dismiss_peek_card()`: this is
	## the shell replacing one overlay with another, not the user closing the
	## card, so its own dismiss callback (map's sample pin) does not fire.
	if _peek_open:
		_peek_open = false
		_peek_panel.visible = false
		_sheet_scrim.visible = false
		_peek_dismiss_cb = Callable()
		## CM-2 residual (§9.4) -- the sheet is genuinely going away here too,
		## same resume as `_dismiss_peek_card()`'s.
		if _shell.has_method("resume_from_context_surface"):
			_shell.call("resume_from_context_surface")

## Canvas BACK: "System back leaves a sheet, then the L2 screen, then the
## viewport -- never the app." Returns true when it consumed the gesture, so
## `DccShell._notification()` knows whether to let the request fall through.
##
## CM-5: the peek/half card is checked first and dismissed on its own --
## `_dismiss_peek_card()`, not `close()`, so its dismiss callback (clearing the
## map's sample pin) actually fires.
func go_back() -> bool:
	if _peek_open:
		_dismiss_peek_card()
		return true
	if not visible:
		return false
	if _stack.size() <= 1:
		close()
		return true
	_stack.pop_back()
	_render()
	return true

## Signal adapter for the header's back button: discards `go_back()`'s result, which only
## matters to the system-back caller.
func _go_back_pressed() -> void:
	go_back()

## A press (mouse or touch) on the sheet scrim goes back one level, like system back.
## Releases and motion are ignored so one tap is one step.
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

## Repaints from the navigation stack `_stack`: shows the deepest screen step (if any) in the
## full-screen pane with its title and head variant (root versus drilled), and shows the top step in the
## sheet when it is a sheet, veiling the screen behind it. Does nothing on an empty stack. The place the
## stack becomes nodes.
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

## Removes and frees every child of a screen body. Removed first so a stale row cannot be
## hit-tested or counted while it waits to be freed.
func _clear(body: VBoxContainer) -> void:
	for child in body.get_children():
		body.remove_child(child)
		child.queue_free()

## Refills a screen body for one step: clears it and the per-screen `_refreshed` set, then
## draws a bespoke screen (`step.screen`) or re-presents a popup's items, or a labelled
## "no content source" row when the step has neither, and closes the list with a hairline
## and a padded tail. Must not leave a screen silently empty.
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
		## Unreachable through `open()` / `_push_screen()` / `_push()` --
		## neither a dedicated screen id nor a menu reached `_fill()`.
		body.add_child(_missing_row("This screen has no content source.",
			"Nothing was set up to fill this screen with rows."))
	else:
		_fill_popup(body, step)
	## Canvas "02"/"03"/"07" all close their list with a hairline and a padded
	## tail so the last row is not flush against the gesture inset.
	body.add_child(DccTheme.rule())
	var tail := Control.new()
	tail.custom_minimum_size.y = _ps(24)
	tail.mouse_filter = Control.MOUSE_FILTER_IGNORE
	body.add_child(tail)
	_let_drags_reach_scroller(body)

## Phone fix 2026-10-26 (owner finding, OnePlus 6T: the MORE list only scrolled by dragging its scrollbar
## edge; measured with `adb shell input swipe`: a swipe that began on a band caption scrolled, one that began
## on any row did not). A control with `MOUSE_FILTER_STOP` ends Godot's bubbling of a touch or drag event, so
## the `ScrollContainer` above it never saw the drag. `PASS` still delivers every event to the row's own
## `gui_input` (the press, the release and `_row_input`'s deadzone test are unchanged) and then lets it
## bubble. Sliders are left alone: they have their own touch arbitration (`DccWidgets.touch_slider`), and a
## drag that starts on one is meant to move it. Walks the freshly filled body once (`_fill`); a row a screen adds
## later on its own refresh is NOT covered, and keeps STOP until the next `_fill`.
func _let_drags_reach_scroller(root: Node) -> void:
	for ch in root.get_children():
		if ch is Control:
			var c := ch as Control
			if c.mouse_filter == Control.MOUSE_FILTER_STOP and (c is BaseButton or c is PanelContainer) 					and not (c is Slider):
				c.mouse_filter = Control.MOUSE_FILTER_PASS
		_let_drags_reach_scroller(ch)

# -- Resolving a §6.6 row to a real menu item ---------------------------------

## Which menus have had `about_to_popup` emitted during the screen currently
## being filled. Cleared by `_fill()`, so entering a screen refreshes each menu
## it reads exactly once -- see the header on why "once per screen" is the
## budget rather than "once per row" or "never".
var _refreshed: Dictionary = {}

## The shell's top-level program-menu buttons, or an empty array before the menu bar exists.
## `_menu_popup` walks these to find real popups; this creates nothing.
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
## `_refreshed`. A caller that only needs
## the popup OBJECT passes `false`, and refreshing a menu the user is merely looking at
## the name of is the cost `_menu_root_row()` refuses. A screen that
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



## A row for a menu item this build does not have.
##
## Drawn, not skipped. A screen assembled from ids in another file will one day
## name an id that file no longer has, and the failure mode of skipping is a
## screen that is quietly one row shorter than the spec it was built from --
## which is the class of defect this whole surface exists to remove.
func _missing_row(label: String, why: String) -> Control:
	return _row(label, why, null, null, Callable(), true)

# -- The screens --------------------------------------------------------------

## Dispatches a bespoke screen id (`more`, `project`, `civ`, `data`, `sim`, `prefs`, `assets`,
## `travel`, `landmarks`, `help`, ...) to its `_fill_*` builder, passing `arg` to the ones that take one.
## An unknown id draws a labelled "no content" row rather than an empty screen.
func _fill_screen(body: VBoxContainer, id: String, arg: String = "") -> void:
	match id:
		"more": _fill_root(body)
		"story-root": _fill_story_rows(body)
		"gestures": _fill_gestures(body)
		"story": _fill_story(body, arg)
		_:
			## No case for this id in `_fill_screen()`.
			body.add_child(_missing_row("Unknown screen '%s'." % id,
				"There is no content defined for this screen."))

## MENU's root (MS-4; Ruling DT Q2, Ruling DX Q13: "the phone's MENU follows the mockup"): Quick, the MENUS band,
## the desktop's eight menus in the menu bar's order and under its names, then Status. Each menu row opens that
## menu's popup row for row (`_menu_root_row()`, `_fill_popup()`), so MENU draws exactly the rows the desktop builds.
## The menu rows are read off the live menu bar (`_menu_buttons()`), never a list written here, so a menu added to
## `menus.gd` appears with no edit (`_ms4_probe.gd` proves the parity row by row). Must never route a menu through a
## bespoke screen: a phone-only row belongs in the menus, marked phone only, or is dropped by a stated decision
## (`drafts/mobile-shell/README.md`, "Where each phone-only row goes"). Transplanted from draft 05.
func _fill_root(body: VBoxContainer) -> void:
	_fill_quick(body)
	_head(body, "Menus")
	for mb in _menu_buttons():
		_add(body, _menu_root_row(mb as MenuButton))
	_fill_status(body)

## The subtitle under each menu's root row: the mockup's own, `menuTree()`'s `sub` for that menu
## (`design/mobile-shell-2026-10-31/Shell.dc.html`). A label, not a list: the rows below it are the menu's own.
## Keyed by the menu's title (Ruling DY's rename put "Records" where "World" was; the rows did not change, so
## neither did the subtitle). A title missing here draws no subtitle rather than a wrong one.
const MENU_SUBS: Dictionary = {
	"File": "new · open · save · export · storage",
	"Edit": "undo · clipboard · find · reset",
	"Assets": "library · packs · families · landmark types",
	"Records": "data manager · records · technical",
	"Story": "time · journeys · conflicts · notes",
	"Preferences": "performance · graphics · LOD · memory · app",
	"Window": "regions · session · 3D · workspace · layouts",
	"Help": "documentation · shortcuts · credits · about",
}

## Lead glyphs of the eight menu rows (the mockup's; `_glyph()` falls back to the submenu arrow for a glyph the mono
## font lacks). Records keeps the glyph World had: Ruling DY renamed the menu, not its rows.
const MENU_GLYPHS: Dictionary = {
	"File": "⧉", "Edit": "✎", "Assets": "▦", "Records": "◍", "Story": "➔",
	"Preferences": "⚙", "Window": "▢", "Help": "?",
}

## One MENU root row: the menu's name, the mockup's subtitle for it, the count of its rows and its glyph. Pressing it
## pushes the menu's popup, which `_fill_popup()` draws row for row with every live refresh the desktop runs
## (`about_to_popup`). Never refreshes the popup itself: the root only names the menus, and refreshing a menu the
## user is merely looking at the name of is the cost the old `_menu_row()` refused (`AtlasCache` evicts baked
## chunks, `GpuDevices` enumerates adapters). Replaces `_menu_row()` (draft 05).
func _menu_root_row(mb: MenuButton) -> Control:
	var popup := mb.get_popup()
	var title := String(mb.text)
	var count := 0
	for i in popup.item_count:
		if not popup.is_item_separator(i):
			count += 1
	return _row(title, String(MENU_SUBS.get(title, "")), _trail_label("%d" % count), _chevron(),
		func(): _push(popup, title, SCREEN_LEVEL), false, _glyph(String(MENU_GLYPHS.get(title, "▸"))))

## The Status band, moved out of the old `_fill_root()` without change (draft 05's `_fill_status()`): the readouts
## that hold a real value (`STATUS_ROWS`, `_status_rows()`), and "Recompute stale stages" when a recompute is
## possible. `app.gd` puts a Recompute button beside the desktop `stale` slot and its own comment says why the
## handset does not get it (SS13 parks the status bar in an invisible model host, so a control added there is never
## drawn); this row is that act, on the same two preconditions. §15 fault 2: the desktop readout cluster is rows
## here, not the reparented chrome.
func _fill_status(body: VBoxContainer) -> void:
	var status := _status_rows()
	if not status.is_empty():
		_head(body, "Status")
		for entry in status:
			## `entry[2]` is set only for the one slot that holds a sentence.
			_add(body, _note_row(String(entry[0]), String(entry[1])) if bool(entry[2])
				else _value_row(String(entry[0]), String(entry[1])))
		if _can_recompute_stale():
			_add(body, _row("Recompute stale stages",
				"Re-runs only the stages the graph reports stale. The civilisation "
					+ "layer is deliberately not cascaded per edit, so \"civ\" usually "
					+ "stays -- the Civilization dock's Recompute civilisation is "
					+ "the one that clears it.",
				null, null, _go_recompute_stale, false))

## CQ-6 (`CUSTOM_QUICK_BAR_SCOPE.md` s.3.2, Ruling DH Q9): the MORE root's `Quick` section, FIRST in the list
## so a pinned command is two taps from anywhere with the sheet closed. One row per entry of
## `QuickBarPhone.entries` -- the Assistant row first when its rule shows it, then the pins in order -- and a
## `Customise...` row last. A row that cannot run is dim and says WHY on its second line (a tooltip is
## invisible on a phone); tapping it toasts the same reason. A live row closes this overlay and then runs, so
## the command's own window is not left under MORE.
##
## **Draws NOTHING when the bar is empty and the assistant is off** -- no band, no row, no gap -- which is
## what keeps MORE's root pixel-identical to the tree before this milestone (`_quickbar_phoneid_probe.gd`).
## The way to a first pin is then `Preferences > Quick bar...`, a row of the Preferences popup the phone
## already lists. Rows are `_row()`'s 52 dp (the phone spec's list-row height), above the scope's 46 dp.
func _fill_quick(body: VBoxContainer) -> void:
	var es := QuickBarPhone.entries(_shell)
	if es.is_empty():
		return
	_head(body, "Quick")
	for e in es:
		var entry: Dictionary = e
		var live := bool(entry["live"])
		var sub := ""
		if not live:
			sub = String(entry["reason"])
		elif String(entry["kind"]) == "assistant":
			sub = "Open the assistant panel"
		elif String(entry["group"]) != "":
			sub = "%s menu" % String(entry["group"])
		var row := _row(String(entry["title"]), sub, null, null, _press_quick.bind(entry), not live)
		var key := String(entry["key"]).replace(":", "_").replace("/", "_")
		row.name = "QuickRow_" + (key if String(entry["kind"]) == "pin" else "assistant")
		_add(body, row)
	_add(body, _row("Customise…", "Pin, unpin and reorder commands", null, _chevron(), _quick_customise, false))

## A tap on a MORE ▸ Quick row. Close first, then run, so a command that opens a window does not open it under
## this overlay; a row that cannot run stays open and toasts its reason (`QuickBarPhone.press`).
func _press_quick(entry: Dictionary) -> void:
	if bool(entry["live"]):
		close()
	QuickBarPhone.press(_shell, entry, null)

## `MORE ▸ Quick ▸ Customise…`: the Customise window, in its phone layout.
func _quick_customise() -> void:
	close()
	if _shell.has_method("open_quick_bar_window"):
		_shell.open_quick_bar_window()

## SM-2 (`STORY_MENUS_PROPOSAL.md` §6, mockup A8): the MORE root's STORY
## section, one row per band of the desktop `Story` menu (`menus.gd::_story()`).
## `band` is that menu's separator caption, which is how `_fill_story()` finds
## the band's rows -- so the rows each screen draws are read off the live popup,
## never copied here. `label` and `glyph` are A8's; `sub` names the band's own
## rows (A8 wrote `vault` for Notes, which this build's band does not hold, so
## the subtitle says what it does hold).
const STORY_ROWS: Array = [
	{"band": "TIME", "glyph": "◷", "label": "Timeline",
		"sub": "timeline · political · faction history"},
	{"band": "JOURNEYS", "glyph": "→", "label": "Journeys",
		"sub": "planner · saved journeys · travel library"},
	{"band": "CONFLICTS & CAMPAIGNS", "glyph": "◇", "label": "Conflicts & campaigns",
		"sub": "list · draw · this year"},
	{"band": "NOTES", "glyph": "▤", "label": "Notes",
		"sub": "this place · places without notes"},
]

## The `STORY_ROWS` entry for `band`, or `{}`.
func _story_row(band: String) -> Dictionary:
	for r in STORY_ROWS:
		if String(r["band"]) == band:
			return r
	return {}

## The Story screen's body (`story-root`, the STORY tab through `open_story()`): the four band rows of
## `STORY_ROWS`, each opening its band (`_fill_story()`). MENU ▸ Story is the Story popup itself, row for row
## (Ruling DX Q13); this screen is the STORY tab's. (MS-4 retired `_fill_story_section()`, which drew these rows
## under the old MORE root's "Story" and "World" heads.)
func _fill_story_rows(body: VBoxContainer) -> void:
	for r in STORY_ROWS:
		var band := String(r["band"])
		_add(body, _row(String(r["label"]), String(r["sub"]), null, _chevron(),
			func(): _push_screen("story", band), false, _glyph(String(r["glyph"]))))

## The STORY tab (Ruling DT Q1; draft 05). Opens this sheet on the Story screen, or straight on one band
## ("TIME", "JOURNEYS", "CONFLICTS & CAMPAIGNS" or "NOTES", the `STORY_ROWS` bands). System back from a band
## returns to the Story screen, and from there closes the sheet. Does not close other overlays; the shell
## does that first. Must never leave the planner unreachable: JOURNEYS holds it (the PLAN tab's old door).
func open_story(band: String = "") -> void:
	_stack.clear()
	_stack.append(_Step.new(null, _screen_title("story-root"), "", 2, "story-root"))
	visible = true
	if band != "":
		_push_screen("story", band)
	else:
		_render()

## Opens MENU on its root and pushes screen `id` over it, so system back returns to MENU. For a row a program menu
## carries whose content is a phone screen: Help ▸ Gesture reference… (`gestures`), reached through
## `DccShell.phone_open_menu_screen()` (draft 09). Must not be given a screen MS-4 deleted.
func open_screen(id: String) -> void:
	_stack.clear()
	_stack.append(_Step.new(null, _screen_title("more"), "", 2, "more"))
	visible = true
	_push_screen(id)

## One Story band as a screen: the `Story` popup's rows between the separator
## captioned `band` and the next separator, drawn by `_rest_of()` in the popup's
## own order (so a disabled row arrives with its reason, and a readout as a
## readout). Refreshes the popup once (`_menu_popup`), which is what fills its
## "this place" rows and the campaigns count.
func _fill_story(body: VBoxContainer, band: String) -> void:
	var p := _menu_popup("Story")
	if p == null:
		_add(body, _missing_row("Story", "The Story menu is not on this build's menu bar."))
		return
	var start := -1
	var stop := p.item_count
	for i in p.item_count:
		if not p.is_item_separator(i):
			continue
		if start < 0 and _clean(p.get_item_text(i)) == band:
			start = i
		elif start >= 0:
			stop = i
			break
	if start < 0:
		_add(body, _missing_row(band.capitalize(),
			"The Story menu has no %s band in this build." % band))
		return
	var drawn := {}
	for i in p.item_count:
		if i <= start or i >= stop:
			drawn[i] = true
	_rest_of(body, p, drawn)


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


# -- What is left of §6.6's sub-screens ----------------------------------------
#
# MS-4 deleted fourteen of §6.6's screens (`project`, `civ`, `data`, `data-tiles`, `sim`, `prefs`, `assets`,
# `assets-grid`, `asset-slot`, `travel`, `travel-item`, `landmarks`, `lm-fam`, `help`) with their builders and
# helpers: MENU is the eight desktop menus row for row now (`_fill_root()`), and each row those screens drew has a
# stated home or a stated drop in `cartalith-native/drafts/mobile-shell/README.md`, "Where each phone-only row
# goes". `data-io` was never built (a mock in the specification itself; this port's `DataManagerWindow.ROUTES` are
# the real thing, reached from Records). What stays is `gestures`, the touch vocabulary, whose door is Help ▸
# Gesture reference… (draft 09), and the STORY tab's `story-root` and `story`.
#
# Everything below reads live state. Where a figure does not exist this build draws no figure: no invented byte
# sizes, no `0.9 · build 2611`, no fill counts written down. Where §6.6's own row is *wrong about this build* --
# three of `gestures`' nine are -- the row says what this build does.


# -- help / gestures ----------------------------------------------------------


## §6.6's nine `read` rows, **checked against this build one at a time**, which
## is what the 2026-09-05 pass declined to do and said so
## (the pre-MS-4 Preferences screen carried the note: *"writing them down
## without testing each one against this build is exactly the 'prose that
## describes behaviour nobody checked' defect"*). That note is long stale.
##
## **Three of §6.6's nine are wrong about this build, and each says what this
## build does instead:**
##
##   - `TWO FINGERS · rotate` — there is no map rotation anywhere in
##     `viewport_host.gd`; what two fingers produce is
##     `InputEventPanGesture`, which pans.
##   - `LONG-PRESS · sample terrain → pin + chip` — `map_overlay.gd`'s
##     `_TOUCH_HOLD_MS` (500 ms) turns a hold into `context_requested`, which
##     `context_broker.gd` presents as the L4 sheet (CIVIL's rows come from
##     `civilization_workspace.gd::context_actions()`). Its `Info here` row is the sampling half; there is no pin-and-chip.
##   - `TAB RE-TAP · close the sheet` — `DccShell._pick_phone_tab()` collapses
##     it to **peek** instead, and its own comment says why: this sheet is the
##     tool options bar and it has no "gone" state on the other two form
##     factors.
##
## **One more of the nine does not exist and is drawn as absent, not omitted:**
## no edge-swipe inspector — the only edge swipe this shell reads is Android's
## own back gesture, which `DccShell._phone_back()` handles and which is listed
## here as itself. (The double-tap was absent here until MS-5 built it, with
## thumb zoom beside it; both are `GESTURES` rows now.)
##
## Every row below names the symbol it was checked at, in its second line, so
## the next reader can re-check it rather than believing this list.
## Each entry's third field is a plain confirmation line; the exact code
## location it was checked against is noted in the comment above that entry,
## for a future engineer re-verifying this list, not for the on-screen text.
const GESTURES: Array = [
	## Checked at `viewport_host.gd`'s `_touch_motion()` (the `_TP_CANDIDATE` branch: a drag past
	## `map_overlay.gd::TOUCH_SLOP` with Inspect or a placing tool pans) and `_input()`'s latch branch (with a drawing
	## tool armed, one finger draws and the latch hands it to the map). MS-5.
	["Drag", "Pans the map with Inspect or a placing tool. With a drawing tool armed one "
		+ "finger draws, and the hand button gives it to the map.",
		"Confirmed in this build."],
	## Checked at `viewport_host.gd`'s `_touch_press()` / `_touch_release()` / `_double_tap_zoom()` (MS-5):
	## x1.7 about the finger over 240 ms, on with Inspect or the hand button, off with a placing or drawing tool.
	["Double-tap", "Zooms in about the finger. On with Inspect, or with a drawing tool while "
		+ "the hand button is on; off with a placing tool, where a tap places.",
		"Confirmed in this build."],
	## Checked at `viewport_host.gd`'s `_touch_motion()` `_TP_THUMB` branch (MS-5, Ruling EB item 3).
	["Double-tap, hold, slide", "Zooms about the finger with one thumb: slide down to zoom in, "
		+ "up to zoom out. Lift to stop. The held press opens no card.",
		"Confirmed in this build."],
	## Checked at `viewport_host.gd`'s `InputEventMagnifyGesture` handling.
	["Pinch", "Zooms at the pinch centre.",
		"Confirmed in this build."],
	## Checked at `viewport_host.gd`'s `InputEventPanGesture` handling;
	## nothing rotates the view.
	["Two fingers", "Pans. This build has no map rotation.",
		"Confirmed in this build."],
	## Checked at `map_overlay.gd`'s `_TOUCH_HOLD_MS` (500) -> `context_requested`
	## -> `context_broker.gd` -> `civilization_workspace.gd::context_actions()`.
	["Press and hold", "Opens the map menu: edit, move the viewer, delete, drop a "
		+ "settlement here, or read the terrain here.",
		"Confirmed in this build."],
	## Checked at `dcc_shell.gd`'s `_on_phone_sheet_grab_input()` /
	## `_set_phone_detent()`.
	["Sheet handle", "Drags the sheet between peek, half and full.",
		"Confirmed in this build."],
	## Checked at `dcc_shell.gd`'s `_pick_phone_tab()`.
	["Tab re-tap", "Collapses the sheet to peek. It does not close: the sheet is "
		+ "the tool options bar and has no closed state on desktop or tablet.",
		"Confirmed in this build."],
	## Checked at `dcc_shell.gd`'s `PHONE_UNDO_HOLD_SEC` (0.45) -> the
	## `undo_ledger()` popover.
	["Undo chip", "Tap undoes one step; hold opens the step history.",
		"Confirmed in this build."],
	## Checked at `dcc_shell.gd`'s `_phone_back()`, and `PhoneMenu.go_back()`.
	["Back", "Leaves a sheet, then this screen, then the overlay — never the app. "
		+ "Android's edge swipe and the hardware key are the same gesture.",
		"Confirmed in this build."],
]

## The §6.6 row this build does not have. Drawn, with the true reason,
## rather than dropped: a gesture reference that silently omits one of the nine
## a user may have read about elsewhere teaches them the list is complete.
const GESTURES_ABSENT: Array = [
	["Edge-swipe inspector", "No inspector drawer is bound to an edge swipe. The "
		+ "right dock is a sheet reached from the bottom bar, and the only edge "
		+ "swipe this shell reads is Android's back gesture, listed above."],
]

## The gestures reference screen: one read-only row per `GESTURES` entry, then the
## `GESTURES_ABSENT` rows as missing, then a note that each row names the symbol it was
## checked at. Prose about behaviour, so it can go stale; it triggers nothing.
func _fill_gestures(body: VBoxContainer) -> void:
	for g in GESTURES:
		_add(body, _row(String(g[0]), String(g[1]),
			null, null, Callable(), false))
	_head(body, "Not available")
	for g in GESTURES_ABSENT:
		_add(body, _missing_row(String(g[0]), String(g[1])))


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
## for the same reason as other by-name reads of `DccApp` members:
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


# -- §6.6 row types over a real popup -----------------------------------------

## Append a control, with the hairline every list in this file puts between its
## rows. A band supplies its own leading gap, so no rule is drawn against one.
func _add(body: VBoxContainer, ctrl: Control) -> void:
	var n := body.get_child_count()
	if n > 0 and not bool(body.get_child(n - 1).get_meta(_META_BAND, false)):
		body.add_child(DccTheme.rule())
	body.add_child(ctrl)

const _META_BAND := "phone_band"  ## Meta key set on a band row; `_add()` draws no hairline after a row carrying it.

## Appends a section band, tagged so `_add` draws no hairline against it (a band supplies
## its own leading gap).
func _head(body: VBoxContainer, title: String) -> void:
	var b := _band(title)
	b.set_meta(_META_BAND, true)
	body.add_child(b)





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
		_add(body, _popup_row(p, i))





# -- One popup, re-presented --------------------------------------------------

## Re-presents a desktop `PopupMenu` (`step.popup`) as rows: emits its `about_to_popup` first so
## dynamic menus (recent worlds, GPU devices, open windows) fill, then draws each item via `_popup_row`
## with hairlines between them and a titled band for any labelled separator. A leading separator is skipped
## only when it has no caption. Since MS-4 this draws all eight of MENU's menus and every submenu drilled from
## them (draft 05; Ruling DX Q13).
##
## **One test differs from the pre-MS-4 version**: it skipped every leading separator, captioned or not, and its
## comment said every separator `menus.gd` writes is unlabelled. Many are labelled now (RECORDS, FOR THE TABLE,
## SOURCES, ACTIVE PACK), and a leading one lost its caption. A leading plain separator is still skipped: it would
## draw a gap above the first row.
func _fill_popup(body: VBoxContainer, step: _Step) -> void:
	var p := step.popup
	## Live exactly as the desktop menu is live: Recent worlds, GPU devices,
	## Open windows and the Preferences busy-lock all populate here.
	p.about_to_popup.emit()

	var first := true
	for i in p.item_count:
		if p.is_item_separator(i):
			## Canvas: "L3 stays a titled band, never a disclosure." A labelled
			## separator is a caption, wherever it sits; an unlabelled one is the
			## rule-and-gap the desktop draws, except at the very top.
			if first and _clean(p.get_item_text(i)) == "":
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

## Draws popup item `i` as a row: a drill row for an enabled submenu, a disabled row showing
## the tooltip as its reason, a checkable row with a radio or switch mark that stays open
## through `_activate`, or a plain action row. Reads the item's state at draw time.
func _popup_row(p: PopupMenu, i: int) -> Control:
	var text := _clean(p.get_item_text(i))
	var disabled := p.is_item_disabled(i)
	var sub_name := p.get_item_submenu(i)

	if sub_name != "" and not disabled:
		var sub := p.get_node_or_null(NodePath(sub_name)) as PopupMenu
		if sub != null:
			## No `about_to_popup` here either -- see `_menu_root_row()`.
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

const _META_PRESSED := "pressed"  ## Meta key on a row: true between press and release; cleared once the pointer moves past `DccShell.PHONE_SCROLL_DEADZONE`.
const _META_PRESS_POS := "press_pos"  ## Meta key on a row: where the press began, the origin for the deadzone test.

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
		## 2026-10-26: compared in VIEWPORT coordinates. `ev.position` is local to the row, and once the
		## ScrollContainer starts scrolling the row moves with the finger, so the local distance stayed under the
		## deadzone forever and a scroll that began on a row opened it on release (reproduced by
		## `_phonescroll_probe.gd`: the first MORE row, "New world...", ran after a 110 px drag).
		var origin: Vector2 = row.get_meta(_META_PRESS_POS, row.get_global_transform_with_canvas() * pos)
		if (row.get_global_transform_with_canvas() * pos).distance_to(origin) > _ps(DccShell.PHONE_SCROLL_DEADZONE):
			row.set_meta(_META_PRESSED, false)
			row.add_theme_stylebox_override("panel", DccTheme.empty())
		return
	else:
		return
	if down:
		row.set_meta(_META_PRESSED, true)
		row.set_meta(_META_PRESS_POS, row.get_global_transform_with_canvas() * pos)
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

## A title with a short value on the right. `_trail_label` neither wraps nor clips, so a
## long value becomes the screen's minimum width inside a scroll area with horizontal scroll
## disabled; keep values short and use `_note_row` for prose.
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
## (MS-4 made the phrase `MENU ▸ File ▸ New world… to begin`, the File popup's own first row; the same length class.)
func _note_row(title: String, text: String) -> Control:
	return _row(title, text, null, null, Callable(), false)

## The right-hand count/value label of a row: dim mono, vertically centred, ignoring the
## mouse so the row still receives the tap.
func _trail_label(text: String) -> Label:
	var l := DccTheme.mono_label(text, "text_dim", _ps(11), 0)
	l.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return l


## The drill-in marker drawn on the right of a navigation row; ignores the mouse.
func _chevron() -> Label:
	var l := DccTheme.mono_label(DccIcons.SYMBOLS["expand"], "text_ghost", _ps(14), 0)
	l.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return l

## A radio mark: the filled glyph in the accent colour when on, the empty one in a ghost
## colour when off. A mark only; the row owns the tap.
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

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
## **It re-presents `menus.gd`, it does not reimplement it.** Every row here is
## read off the real `PopupMenu` objects `DccShell.add_menu()` already built,
## and every tap goes back out through `PopupMenu.activate_item()` -- the
## engine's own activation path, which emits `id_pressed`/`index_pressed` to
## whatever `menus.gd` connected. No menu id, callback or label is duplicated in
## this file; add an item to `menus.gd` and it appears here with no change.
## `about_to_popup` is emitted before a popup is read, so the rows that rebuild
## themselves on open (Recent worlds, GPU devices, Open windows, the Preferences
## busy-lock) are as live here as they are on desktop.
##
## ## The five levels, and where each one really occurs
##
## The canvas's own PHONE RULES: "L1 is the bottom bar, L2 a drill screen, L3 a
## titled band, L4 a sheet, L5 a full screen", and "Drilling replaces rather
## than stacks: at most one L2 screen and one sheet exist at a time."
##
## | Level | Canvas treatment | What it is here |
## |---|---|---|
## | L1 | bottom bar | `DccShell._build_phone_menu_bar()` -- WORLD/CIVIL/CARTO/PANELS/MENU. Not this file. |
## | L2 | drill screen | This file's **root**: the seven program menus as a grouped list, plus the live status readouts. |
## | L3 | titled band | One program menu's items. Its `add_separator()` groups are the bands. |
## | L4 | sheet, 60% cap | A submenu (Recent worlds, Theme, Devices, Asset pack, Workspace, ...). |
## | L5 | full screen | A submenu *inside* a submenu -- really three of them: `Assets ▸ Asset pack ▸ Edit / Batch / Build`. |
##
## Levels past 5 (none exist in the shipped tree today) keep the L5 treatment
## rather than nesting sheets, which is what the canvas's "at most one sheet"
## rule requires.
##
## ## The one honest shortfall
##
## The canvas draws L3 bands with titles ("§ HYDRAULIC PASSES"). Every
## `add_separator()` in `menus.gd` is **unlabelled**, so a band here draws as
## the hairline-plus-gap the desktop menu itself draws. The moment a separator
## is given text it becomes a titled band with no change to this file -- but
## today it is a rule, not a caption, and that is stated rather than faked with
## invented headings.
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
	var title: String
	var trail: String
	var level: int

	func _init(p: PopupMenu, t: String, tr: String, lv: int) -> void:
		popup = p
		title = t
		trail = tr
		level = lv

	func is_sheet() -> bool:
		return level == 4

## The L2 grouping. `menus.gd`'s seven menus in the order §2 declares them,
## banded the way the canvas's own "07 · MORE" artboard bands its list
## (PROJECT / ASSETS / VIEW). Any menu bar entry that matches no group falls
## into a trailing "Other" band rather than disappearing, so an eighth program
## menu can never go silently missing from the phone.
const GROUPS: Array = [
	{"title": "Project", "menus": ["File", "Edit"]},
	{"title": "Content", "menus": ["Assets", "Data"]},
	{"title": "System", "menus": ["Preferences", "Window", "Help"]},
]

## The status readouts, in the order they read best as a list. Keys are
## `DccShell`'s own status slots; `set_status()` keeps them live and this
## re-reads them every time the root screen is drawn.
const STATUS_ROWS: Array = [
	["top_world", "World"],
	["top_res", "Resolution"],
	["pass", "Pass"],
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

## Canvas "SHEETS STOP AT 60% HEIGHT". Expressed as anchors (top 0.4) rather
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
	_stack.append(_Step.new(null, "More", "", 2))
	visible = true
	_render()

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
		_screen_head_trail.text = "" if root else screen_step.trail
		_screen_head_trail.visible = not root
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
	if step.popup == null:
		_fill_root(body)
	else:
		_fill_popup(body, step)
	## Canvas "02"/"03"/"07" all close their list with a hairline and a padded
	## tail so the last row is not flush against the gesture inset.
	body.add_child(DccTheme.rule())
	var tail := Control.new()
	tail.custom_minimum_size.y = _ps(24)
	tail.mouse_filter = Control.MOUSE_FILTER_IGNORE
	body.add_child(tail)

# -- L2: the root screen ------------------------------------------------------

func _menu_buttons() -> Array:
	var out: Array = []
	if _shell.menu_bar_row == null:
		return out
	for child in _shell.menu_bar_row.get_children():
		if child is MenuButton:
			out.append(child)
	return out

func _fill_root(body: VBoxContainer) -> void:
	## §15 fault 2: the desktop readout cluster used to be reparented into the
	## sheet whole -- a 150 px wordmark and five labels that are empty before a
	## generation, eating most of the surface. The readouts themselves are worth
	## having; the desktop chrome around them is not. They are rows now.
	var status := _status_rows()
	if not status.is_empty():
		body.add_child(_band("Status"))
		for pair in status:
			body.add_child(DccTheme.rule())
			body.add_child(_value_row(String(pair[0]), String(pair[1])))

	var buttons := _menu_buttons()
	var placed := {}
	for group in GROUPS:
		var rows: Array = []
		for wanted in group.menus:
			for mb in buttons:
				if String(mb.text) == String(wanted):
					rows.append(mb)
					placed[mb] = true
		if rows.is_empty():
			continue
		body.add_child(_band(String(group.title)))
		for mb in rows:
			body.add_child(DccTheme.rule())
			body.add_child(_menu_row(mb))

	var rest: Array = []
	for mb in buttons:
		if not placed.has(mb):
			rest.append(mb)
	if not rest.is_empty():
		body.add_child(_band("Other"))
		for mb in rest:
			body.add_child(DccTheme.rule())
			body.add_child(_menu_row(mb))

func _status_rows() -> Array:
	var out: Array = []
	for entry in STATUS_ROWS:
		var text := _shell.status_slot_text(String(entry[0]))
		if text.strip_edges() != "":
			out.append([String(entry[1]), text])
	return out

## Deliberately does **not** fire `about_to_popup` to build the preview. A
## preview needs names and a count, both static; firing it would run all seven
## handlers on every render of this screen -- and `Preferences ▸ Devices` walks
## every `wgpu` backend in its own, which is the enumeration cost
## `menus.gd` moved behind a first-open for a reason. The refresh happens when
## the row is *entered* (`_fill_popup`), which is when the desktop does it too.
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
		func(): _push(popup, title, 3), false)

# -- L3/L4/L5: one popup ------------------------------------------------------

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
		body.add_child(_popup_row(p, i, step.level))

func _popup_row(p: PopupMenu, i: int, level: int) -> Control:
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
				func(): _push(sub, text, level + 1), false)

	## An item the port cannot honour is added disabled with the reason in its
	## tooltip (`menus.gd`'s own honesty rule). A phone has no hover, so the
	## reason is drawn as the row's second line instead of being unreachable --
	## this surface is *more* legible than the desktop one, not less.
	if disabled:
		return _row(text, p.get_item_tooltip(i), null, null, Callable(), true)

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

func _row(title: String, subtitle: String, trail: Control, mark: Control,
		on_press: Callable, dim: bool) -> Control:
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
func _row_input(ev: InputEvent, row: PanelContainer, on_press: Callable) -> void:
	var down := false
	var up := false
	if ev is InputEventMouseButton and (ev as InputEventMouseButton).button_index == MOUSE_BUTTON_LEFT:
		down = (ev as InputEventMouseButton).pressed
		up = not down
	elif ev is InputEventScreenTouch:
		down = (ev as InputEventScreenTouch).pressed
		up = not down
	else:
		return
	if down:
		row.set_meta(_META_PRESSED, true)
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

func _trail_label(text: String) -> Label:
	var l := DccTheme.mono_label(text, "text_dim", _ps(11), 0)
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

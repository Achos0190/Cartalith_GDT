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
var _screen_back: Button
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

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	panel.add_child(col)

	## Canvas "02"/"03": ← 40 dp, title over a breadcrumb line, ⋮ on the right.
	## The right-hand slot is a close here rather than the canvas's overflow
	## dot-column: there is no third menu behind this menu to put there, and a
	## decorative one would be the "connected affordance with nothing behind it"
	## the gap register exists to catch.
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", _ps(12))
	head.custom_minimum_size.y = _pt(56)

	_screen_back = _icon_button(DccIcons.SYMBOLS["collapse"], "Back", _go_back_pressed)
	head.add_child(_screen_back)

	var titles := VBoxContainer.new()
	titles.add_theme_constant_override("separation", _ps(2))
	titles.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	titles.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	titles.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_screen_head_title = DccTheme.mono_label("", "text_bright", _ps(12), 3, true)
	_screen_head_trail = DccTheme.mono_label("", "text_faint", _ps(9), 1)
	titles.add_child(_screen_head_title)
	titles.add_child(_screen_head_trail)
	head.add_child(titles)

	head.add_child(_icon_button(DccIcons.SYMBOLS["cross"], "Close menu", close))

	var hp := MarginContainer.new()
	hp.add_theme_constant_override("margin_left", _ps(6))
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
	_stack.append(_Step.new(null, "Menu", "Cartalith", 2))
	visible = true
	mouse_filter = Control.MOUSE_FILTER_STOP
	_render()

func close() -> void:
	visible = false
	mouse_filter = Control.MOUSE_FILTER_IGNORE
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
	_stack.append(_Step.new(popup, title, " · ".join(trail) + " · L%d" % level, level))
	_render()

# -- Render -------------------------------------------------------------------

func _render() -> void:
	if _stack.is_empty():
		return
	var top: _Step = _stack[_stack.size() - 1]

	## The deepest *screen* step. When the top step is a sheet this is the step
	## behind it, which is exactly what the canvas veils rather than replaces.
	var screen_step: _Step = _stack[0]
	for s in _stack:
		if not s.is_sheet():
			screen_step = s

	_screen_head_title.text = screen_step.title.to_upper()
	_screen_head_trail.text = screen_step.trail
	_screen_back.text = DccIcons.SYMBOLS["cross"] if screen_step.level == 2 \
		else DccIcons.SYMBOLS["collapse"]
	_screen_back.tooltip_text = "Close menu" if screen_step.level == 2 else "Back"
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
	row.custom_minimum_size.y = _pt(52)
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
	if subtitle.strip_edges() != "":
		var s := DccTheme.mono_label(_shorten(subtitle), "text_faint", _ps(9), 0)
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

func _band(title: String) -> Control:
	var wrap := MarginContainer.new()
	wrap.add_theme_constant_override("margin_left", _ps(16))
	wrap.add_theme_constant_override("margin_right", _ps(16))
	wrap.add_theme_constant_override("margin_top", _ps(16))
	wrap.add_theme_constant_override("margin_bottom", _ps(6))
	wrap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	## An unlabelled `add_separator()` has no caption to draw, so the band is the
	## gap plus the hairline the list already puts between rows. Stated in this
	## file's header as the one place the canvas is not fully met.
	var l := DccTheme.header(title, "") if title.strip_edges() != "" \
		else DccTheme.header("", "")
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

func _shorten(text: String) -> String:
	if text.length() <= _SUBTITLE_MAX:
		return text
	var cut := text.substr(0, _SUBTITLE_MAX)
	var space := cut.rfind(" ")
	if space > _SUBTITLE_MAX / 2:
		cut = cut.substr(0, space)
	return cut + "…"

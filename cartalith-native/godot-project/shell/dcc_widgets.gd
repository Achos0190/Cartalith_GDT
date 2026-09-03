extends RefCounted
class_name DccWidgets

## The disclosure grammar (`UI_SHELL_DESIGN.md`) and the row vocabulary every
## dock is built from.
##
## Five levels, no deeper:
##
## | L1 | domain            | owns a workspace, never a mode |
## | L2 | ▾ category        | one open at a time, state persists per domain |
## | L3 | § section         | always expanded, a titled band of rows |
## | L4 | › group           | one pass or one tool; its action button sits inside |
## | L5 | + advanced        | expert dials, closed by default, defaults correct |
##
## A sixth level means the L2 category is wrong and should be split. A group
## gated by a checkbox renders at L4 and is **hidden, not disabled**, when off.
## These functions are the only sanctioned way to draw those levels, so the
## rule is enforced by there being nothing deeper to call.

# -- L2 category --------------------------------------------------------------

## A collapsible category. Returns the body VBox; `header_extra` may add a
## readout to the right of the caret. Categories are accordion siblings --
## `group` ties them together so opening one closes the rest.
static func category(parent: Control, title: String, group: Array,
		open: bool = false) -> VBoxContainer:
	var wrap := VBoxContainer.new()
	wrap.add_theme_constant_override("separation", 0)
	wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(wrap)

	var btn := Button.new()
	btn.flat = true
	btn.focus_mode = Control.FOCUS_NONE
	btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
	## A category header is itself a dock row (an L2 disclosure header), so it
	## takes the same `row_min_h`/`fs_readout` pair `_row()` takes for its own
	## label -- `fs_readout` rather than `fs_prose` because this label is set in
	## Plex (`mono(1)` two lines down), matching §57's "sans and mono take
	## different multipliers off the same rung" finding.
	var cat_tablet := DccTheme.is_tablet()
	btn.custom_minimum_size.y = DccTheme.role_px("row_min_h") if cat_tablet else 30
	btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	btn.add_theme_font_size_override("font_size",
		DccTheme.role_px("fs_readout") if cat_tablet else DccTheme.FS_SMALL)
	btn.add_theme_font_override("font", DccTheme.mono(1))
	btn.add_theme_color_override("font_color", DccTheme.c("text_bright"))
	btn.add_theme_stylebox_override("normal", DccTheme.inset(12, 0, 12, 0))
	btn.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	btn.add_theme_stylebox_override("pressed", DccTheme.inset(12, 0, 12, 0))

	## **Phone only: the per-row control count.**
	## `design/Cartalith Android Phone.dc.html`'s `02 Domain` screen puts a
	## `10px 'IBM Plex Mono';color:#6f7478` number at the end of every category
	## row -- "the count is the number of controls inside, so depth is legible
	## before the tap" -- and no desktop or tablet artboard draws one, so the
	## header stays a bare `Button` everywhere else and this costs those two
	## compositions exactly one `size_flags` assignment.
	var head: Control = btn
	var count_label: Label = null
	if DccTheme.is_phone():
		var hbox := HBoxContainer.new()
		hbox.add_theme_constant_override("separation", 4)
		hbox.add_child(btn)
		count_label = DccTheme.mono_label("", "text_faint", DccTheme.FS_TINY, 0)
		count_label.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		var cpad := MarginContainer.new()
		cpad.add_theme_constant_override("margin_right", 12)
		cpad.add_child(count_label)
		hbox.add_child(cpad)
		head = hbox
	wrap.add_child(head)

	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 0)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.visible = open
	wrap.add_child(body)
	wrap.add_child(DccTheme.rule())

	var entry := {"button": btn, "body": body, "title": title}
	group.append(entry)
	btn.text = "%s  %s" % [DccIcons.SYMBOLS["caret"] if open else DccIcons.SYMBOLS["submenu"], title]
	btn.pressed.connect(func(): _toggle_category(entry, group))
	## Deferred because the body is empty right now: the caller fills it with the
	## VBox this returns, synchronously, in the same call stack. `call_deferred`
	## runs at the end of the frame's idle pass, by which time it is populated --
	## and it works on a node that is not yet in the tree, which matters because a
	## workspace builds its whole panel before `register_workspace()` attaches it.
	if count_label != null:
		_fill_category_count.bind(body, count_label).call_deferred()
	return body

## How many controls a category holds, for the phone drill row's count column.
##
## Counts *controls*, not nodes: a `SpinBox` and an `OptionButton` are each one
## control made of several `BaseButton`s and a `LineEdit`, so the walk counts
## them and stops rather than descending. Everything else that a finger can move
## -- a slider, a checkbox, an action, a text field -- counts once.
static func _fill_category_count(body: Node, label: Label) -> void:
	if not is_instance_valid(body) or not is_instance_valid(label):
		return
	var n := _count_controls(body)
	label.text = "%d" % n if n > 0 else ""

static func _count_controls(node: Node) -> int:
	var n := 0
	for child in node.get_children():
		if child is SpinBox or child is OptionButton:
			n += 1
			continue
		if child is BaseButton or child is Range or child is LineEdit or child is TextEdit:
			n += 1
			continue
		n += _count_controls(child)
	return n

static func _toggle_category(entry: Dictionary, group: Array) -> void:
	var opening: bool = not (entry["body"] as Control).visible
	for e in group:
		var on: bool = e == entry and opening
		(e["body"] as Control).visible = on
		var b: Button = e["button"]
		b.text = "%s  %s" % [
			DccIcons.SYMBOLS["caret"] if on else DccIcons.SYMBOLS["submenu"], e["title"]]
		b.add_theme_color_override("font_color",
			DccTheme.c("accent") if on else DccTheme.c("text_bright"))

## A numbered, stateful L2 category -- the Generation Pipeline's own stage row
## (`DCC_SHELL_SPEC.md` §5.1: "number, state dot, name, state label,
## disclosure chevron").
##
## **No current caller** (2026-08-24): v3 replaced WORLD's numbered ten-stage
## list with nine subject categories, and the stages became L3 sections inside
## them (`world_workspace.gd`'s `CATEGORIES`). Kept rather than deleted because
## it is the only row type that can carry a marker changing *after* the row is
## built, which is a real capability and not a v3-specific one -- and because
## deleting it would take §5.1's reasoning below with it. If nothing has
## claimed it by the time WORLD is next reworked, delete it then.
##
## `category()`'s single-string title can't host a state
## marker that changes after the row is built -- a bridge signal can flip a
## stage stale once the dock already exists -- so this is a genuine second row
## type, not `category()` restyled. The accordion contract (one open at a
## time, sharing `group` with any other `category()`/`stage_category()` calls
## on the same panel) is identical, which is why it reuses `_toggle_category`
## rather than a parallel implementation.
## Returns `{"body": VBoxContainer, "state_label": Label}` -- the caller owns
## updating `state_label.text` / its font colour as the world's state changes.
static func stage_category(parent: Control, number: String, title: String,
		group: Array, open: bool = false) -> Dictionary:
	var wrap := VBoxContainer.new()
	wrap.add_theme_constant_override("separation", 0)
	wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(wrap)

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 4)
	wrap.add_child(head)

	var btn := Button.new()
	btn.flat = true
	btn.focus_mode = Control.FOCUS_NONE
	btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
	btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## See `category()`'s own comment -- same row, same role pair.
	var stage_tablet := DccTheme.is_tablet()
	btn.custom_minimum_size.y = DccTheme.role_px("row_min_h") if stage_tablet else 30
	btn.add_theme_font_size_override("font_size",
		DccTheme.role_px("fs_readout") if stage_tablet else DccTheme.FS_SMALL)
	btn.add_theme_font_override("font", DccTheme.mono(1))
	btn.add_theme_color_override("font_color", DccTheme.c("text_bright"))
	btn.add_theme_stylebox_override("normal", DccTheme.inset(12, 0, 0, 0))
	btn.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	btn.add_theme_stylebox_override("pressed", DccTheme.inset(12, 0, 0, 0))
	head.add_child(btn)

	var state_label := DccTheme.mono_label("", "text_dim", DccTheme.FS_MICRO, 1)
	var state_pad := MarginContainer.new()
	state_pad.add_theme_constant_override("margin_right", 12)
	state_pad.add_child(state_label)
	head.add_child(state_pad)

	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 0)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.visible = open
	wrap.add_child(body)
	wrap.add_child(DccTheme.rule())

	var entry := {"button": btn, "body": body, "title": "%s  %s" % [number, title]}
	group.append(entry)
	btn.text = "%s  %s  %s" % [
		DccIcons.SYMBOLS["caret"] if open else DccIcons.SYMBOLS["submenu"], number, title]
	btn.pressed.connect(func(): _toggle_category(entry, group))
	return {"body": body, "state_label": state_label}

# -- L3 section ---------------------------------------------------------------

## A titled band of rows, always expanded. Returns the body VBox.
static func section(parent: Control, title: String) -> VBoxContainer:
	var head := DccTheme.header(title)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_top", 10)
	pad.add_theme_constant_override("margin_bottom", 4)
	pad.add_child(head)
	parent.add_child(pad)

	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 2)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var bpad := MarginContainer.new()
	bpad.add_theme_constant_override("margin_left", 14)
	bpad.add_theme_constant_override("margin_right", 12)
	bpad.add_theme_constant_override("margin_bottom", 6)
	bpad.add_child(body)
	parent.add_child(bpad)
	return body

# -- L4 group -----------------------------------------------------------------

## One pass or one tool. Its action button belongs inside the returned body,
## never in the section around it.
static func group(parent: Control, title: String, open: bool = true,
		sigil: String = "") -> VBoxContainer:
	var mark: String = sigil if sigil != "" else DccIcons.SYMBOLS["expand"]
	var btn := Button.new()
	btn.flat = true
	btn.focus_mode = Control.FOCUS_NONE
	btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
	## An L4 group header is a dock row too (§57's "row_min_h" -- "Dock list
	## rows and menu items"); its own type is `FS_HEADER` (9), which is exactly
	## `ROLE`'s `fs_dock_header` desktop figure (`[9, 11]`), not `fs_readout`.
	var grp_tablet := DccTheme.is_tablet()
	btn.custom_minimum_size.y = DccTheme.role_px("row_min_h") if grp_tablet else 22
	btn.text = "%s %s" % [mark, title.to_upper()]
	btn.add_theme_font_size_override("font_size",
		DccTheme.role_px("fs_dock_header") if grp_tablet else DccTheme.FS_HEADER)
	btn.add_theme_font_override("font", DccTheme.mono(2, true))
	btn.add_theme_color_override("font_color", DccTheme.c("text_faint"))
	btn.add_theme_stylebox_override("normal", DccTheme.empty())
	btn.add_theme_stylebox_override("hover", DccTheme.empty())
	btn.add_theme_stylebox_override("pressed", DccTheme.empty())
	parent.add_child(btn)

	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 2)
	body.visible = open
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 10)
	pad.add_child(body)
	parent.add_child(pad)
	btn.pressed.connect(func():
		body.visible = not body.visible
		btn.text = "%s %s" % [mark, title.to_upper()])
	return body

# -- L5 advanced --------------------------------------------------------------

## Expert dials, closed by default. If a value in here has to be changed for a
## normal result, the default above it is wrong -- fix the default instead.
static func advanced(parent: Control, title: String = "advanced") -> VBoxContainer:
	return group(parent, title, false, "+")

# -- Rows ---------------------------------------------------------------------

const ROW_LABEL_W := 132
## `width:44px;text-align:right` on the canvas's value column. 56 was 27 % wide
## and stole the slack the label needed.
const ROW_VALUE_W := 44
## `width:78px;height:2px` -- the canvas's parameter track, fixed for every row
## in every dock. The tool options bar draws the same control at 70 px; 78 is
## the dock figure and this constant only serves dock rows.
const TRACK_W := 78


## §11: "no fills on panels; regions are separated by hairlines only. Radius 0
## everywhere." A slider follows the same rule -- a 2 px rule, the travelled
## part in accent, and **no grabber**. Godot's default is a thick track with a
## round knob, which reads as a web form rather than a tool.
##
## `StyleBoxFlat.content_margin_*` is how thin the drawn bar reads: with no
## `custom_minimum_size` of its own, a StyleBox's minimum size *is* its content
## margins, and Godot centres that minimum inside the control's real height --
## so `top=1,bottom=1` here draws a 2 px line centred in the 14 px control, not
## a 14 px slab. `role_px("slider_track_h")` (`[2, 3]`, §57's own measured
## pair) is that same total split as evenly as an odd tablet figure allows.
static func _style_slider(s: HSlider) -> void:
	var thickness := DccTheme.role_px("slider_track_h") if DccTheme.is_tablet() else 2
	var track := StyleBoxFlat.new()
	track.bg_color = DccTheme.c("line")
	track.content_margin_top = thickness / 2
	track.content_margin_bottom = thickness - thickness / 2
	s.add_theme_stylebox_override("slider", track)
	var filled := StyleBoxFlat.new()
	filled.bg_color = DccTheme.c("accent")
	s.add_theme_stylebox_override("grabber_area", filled)
	s.add_theme_stylebox_override("grabber_area_highlight", filled)
	## An empty texture is how a grabber is removed; setting a size of zero
	## still draws the theme default.
	s.add_theme_icon_override("grabber", ImageTexture.new())
	s.add_theme_icon_override("grabber_highlight", ImageTexture.new())
	s.add_theme_icon_override("grabber_disabled", ImageTexture.new())
	s.add_theme_constant_override("center_grabber", 1)

## `GUI_GAP_REGISTER.md` §57 / `UNWIRED_FUNCTIONS.md` "the tablet interior
## walk": resolved here, at the one place every dock parameter row passes
## through, rather than by a class-dispatched walk over the finished tree --
## §57's refutation #2 is exactly that a walk cannot tell this row apart from
## any other `HBoxContainer` once built, but this factory always knows it is
## building a dock row. `role_px("row_min_h")` is `ROLE`'s own "Dock list rows
## and menu items" pair (`[0, 44]` -- the desktop `0` means "no constraint",
## not "floor to zero", so the literal `24` stays the desktop figure and only
## tablet reads the table) and `"fs_prose"` is the row label's own pair
## (`[11, 14]`, matching the `FS_SMALL` this replaced on tablet only).
static func _row(parent: Control, label_text: String, tooltip: String) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = DccTheme.role_px("row_min_h") if DccTheme.is_tablet() else 24
	row.tooltip_text = tooltip
	## `font-family:'Helvetica Neue';font-size:11px;color:#a9adb0` on every
	## parameter row in the canvas's left dock -- prose, not Plex, and one ink
	## step brighter than `text_dim`. Only the *value* on the right is Plex.
	## This row is the single most repeated thing in the shell, so drawing its
	## label in mono put a monospaced texture across every dock in the app that
	## the reference does not have anywhere.
	var label_fs := DccTheme.role_px("fs_prose") if DccTheme.is_tablet() else DccTheme.FS_SMALL
	var l := DccTheme.label(label_text, "text_secondary", label_fs)
	l.custom_minimum_size.x = ROW_LABEL_W
	l.clip_text = true
	## GUI_GAP_REGISTER.md phone residue: "World data ▸ Economy rows end
	## `…silver, clay, buildst` -- a hard cut at the panel edge with no
	## affordance." This is that row's own builder -- the single most
	## repeated thing in the shell, per this function's own header, so every
	## caller inherits the fix from here rather than each patching its own
	## copy. `dcc_shell.gd::phone_fit()`'s generalised Label pass does not
	## reach this one: it only trims a `Label` sized by `SIZE_EXPAND`, and
	## this one is sized by the fixed `ROW_LABEL_W` above instead -- the same
	## shape `right_dock.gd` and this file's own `_project_picker` header
	## already fix with this exact pair of properties.
	l.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	row.add_child(l)
	parent.add_child(row)
	return row

## A continuous value: slider plus a right-aligned numeric readout. `on_change`
## is called with every value, including mid-drag -- cheap writes only, per
## `on_release`'s own doc below. Returns a Dictionary of the parts so a caller
## can refresh the row when the engine's value changes underneath it.
##
## `on_release`, if given, fires once when a drag ends (`Slider.drag_ended`):
## the `input` vs `change` split the reference's own `tparam()` uses --
## `input` (every tick) writes the value and updates the label, `change`
## (release) is where expensive work belongs. `HSlider` has no equivalent
## one-shot signal for keyboard-driven changes (arrow keys fire `value_changed`
## per press with no drag to end), so `on_release` does not fire from the
## keyboard today -- a real, minor gap against a mouse/touch drag, not solved
## here rather than papered over with a guess at "was this really a release."
static func slider(parent: Control, label_text: String, minimum: float, maximum: float,
		step: float, value: float, unit: String, on_change: Callable,
		tooltip: String = "", on_release: Callable = Callable()) -> Dictionary:
	var row := _row(parent, label_text, tooltip)
	var s := HSlider.new()
	s.min_value = minimum
	s.max_value = maximum
	s.step = step
	s.value = value
	## `width:78px;height:2px` -- a fixed track, not an expanding one. The
	## canvas gives the *label* the slack and keeps every track in a dock the
	## same length, so the five steering dials read as one column of bars. An
	## expanding track measured 128 px here at a 372 px dock and grew with the
	## dock, which is why long parameter names ("Enable continental shelves")
	## were clipping while the bar beside them had room to spare.
	s.size_flags_horizontal = Control.SIZE_SHRINK_END
	s.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	## `role_px("slider_track_w")` (`[70, 90]`) on tablet only -- `TRACK_W` (78)
	## is the dock's own desktop figure and stays put; the control's own 14 px
	## height is an interaction floor already comfortably above the 2-3 px
	## visual line `_style_slider()` draws, so it is left alone here rather than
	## resized to a figure that would make the touch target *smaller*.
	var track_w := DccTheme.role_px("slider_track_w") if DccTheme.is_tablet() else TRACK_W
	s.custom_minimum_size = Vector2(track_w, 14)
	s.focus_mode = Control.FOCUS_NONE
	_style_slider(s)
	row.add_child(DccTheme.spacer())
	row.add_child(s)
	var readout_fs := DccTheme.role_px("fs_readout") if DccTheme.is_tablet() else DccTheme.FS_SMALL
	var readout := DccTheme.mono_label("", "text", readout_fs, 0)
	readout.custom_minimum_size.x = ROW_VALUE_W
	readout.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	row.add_child(readout)

	var fmt := func(v: float) -> String:
		var digits := 0 if step >= 1.0 else (1 if step >= 0.1 else 2)
		return ("%.*f%s" % [digits, v, unit])
	readout.text = fmt.call(value)
	s.value_changed.connect(func(v: float):
		readout.text = fmt.call(v)
		on_change.call(v))
	if on_release.is_valid():
		s.drag_ended.connect(func(_value_changed: bool): on_release.call())
	return {"row": row, "slider": s, "readout": readout, "format": fmt}

## `CheckBox`/`OptionButton` are both `BaseButton` in Godot 4, so they are real
## tap targets in their own right, not just row furniture -- floored to
## `role_px("btn_min_h")` on tablet for the same reason `action()` is (tier A:
## a single discrete tap, not one of a lit set).
static func toggle(parent: Control, label_text: String, value: bool,
		on_change: Callable, tooltip: String = "") -> CheckBox:
	var row := _row(parent, label_text, tooltip)
	var cb := CheckBox.new()
	cb.button_pressed = value
	cb.focus_mode = Control.FOCUS_NONE
	var fs := DccTheme.role_px("fs_prose") if DccTheme.is_tablet() else DccTheme.FS_SMALL
	cb.add_theme_font_size_override("font_size", fs)
	if DccTheme.is_tablet():
		cb.custom_minimum_size.y = DccTheme.role_px("btn_min_h")
	cb.toggled.connect(func(v: bool): on_change.call(v))
	## **The spacer goes BEFORE the box, and the label is allowed to grow.**
	##
	## `_row` clips its label to `ROW_LABEL_W` (132) and every other control
	## this file builds — `slider`, `choice`, `value` — puts an EXPANDING
	## control after it, so the label's clip is invisible: the row is full and
	## the value sits on the right edge, which is what `_row`'s own comment
	## describes ("Only the *value* on the right").
	##
	## A toggle had neither. The check box is intrinsically sized, so it hugged
	## the clipped label and left the rest of the row empty — and any label
	## longer than 132 px was cut mid-word for no reason, with the space to fix
	## it sitting unused two pixels to the right. Measured on the OnePlus 6T:
	## `Types compete with each o…` with the box on top of the cut and half the
	## row blank.
	##
	## Letting the label expand takes the slack, and moving the spacer ahead of
	## the box right-aligns it into the value column where every other control
	## already lives. 27 toggles across nine files get their full label and a
	## consistent right edge.
	var lbl := row.get_child(0) as Control
	if lbl != null:
		lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(DccTheme.spacer())
	row.add_child(cb)
	return cb

static func choice(parent: Control, label_text: String, options: Array, selected: int,
		on_change: Callable, tooltip: String = "") -> OptionButton:
	var row := _row(parent, label_text, tooltip)
	var ob := OptionButton.new()
	ob.focus_mode = Control.FOCUS_NONE
	ob.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var fs := DccTheme.role_px("fs_prose") if DccTheme.is_tablet() else DccTheme.FS_SMALL
	ob.add_theme_font_size_override("font_size", fs)
	if DccTheme.is_tablet():
		ob.custom_minimum_size.y = DccTheme.role_px("btn_min_h")
	for o in options:
		ob.add_item(String(o))
	ob.selected = selected
	ob.item_selected.connect(func(i: int): on_change.call(i))
	style_popup(ob.get_popup())
	row.add_child(ob)
	return ob

## The canvas's own menu panel, read off `DCC shell 1920`'s open Assets menu,
## `DCC Cartography style 1920`'s open File menu and `DCC shell tablet 2560`'s
## open Data menu:
##
##   background:#121314; border:1px solid rgba(255,255,255,.14);
##   box-shadow:0 14px 34px rgba(0,0,0,.55); padding:5px 0
##   item     padding:6px 14px               (tablet: 9px 18px, min-height 44)
##   label    font-size:11.5px prose         (tablet: 14px)
##   band     font:9px Plex;.18em;#5f6468    (tablet: 11px)
##   rule     rgba(255,255,255,.09), margin:5px 0
##   open row background:rgba(224,163,74,.10);color:#e8ebec
##
## Lives here rather than on `DccShell` because it serves every `PopupMenu` in
## the shell, not only the seven program menus: `choice()` above opens one on
## every dropdown, and until 2026-08-25 those were stock Godot.
static func style_popup(popup: PopupMenu) -> void:
	var touch := DccTheme.is_touch()
	var panel := DccTheme.panel("panel",
		{"left": 1, "right": 1, "top": 1, "bottom": 1})
	panel.border_color = DccTheme.c("border")
	panel.shadow_color = Color(0, 0, 0, 0.55) if DccTheme.is_dark() \
		else Color(0.137, 0.141, 0.122, 0.16)
	panel.shadow_size = 34
	panel.shadow_offset = Vector2(0, 14)
	var pad_y := DccTheme.menu("pad_y", touch)
	var pad_x := DccTheme.menu("pad_x", touch)
	panel.content_margin_top = pad_y
	panel.content_margin_bottom = pad_y
	## `padding:6px 14px` on the canvas's item is horizontal padding on the
	## *row*, and `PopupMenu` has no such constant -- it draws from the panel's
	## own content margin plus `item_start_padding`. Both were 0/2, which is
	## why every menu in the shell sat about 10 px tighter to its edge than the
	## canvas draws it.
	panel.content_margin_left = pad_x
	panel.content_margin_right = pad_x
	popup.add_theme_stylebox_override("panel", panel)
	popup.add_theme_color_override("font_color", DccTheme.c("text"))
	popup.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	popup.add_theme_color_override("font_accelerator_color", DccTheme.c("text_faint"))
	## Item labels are prose (`font-size:11.5px`); only the shortcut column is
	## Plex, and `PopupMenu` draws that column from the same font, so this
	## follows the label rather than the shortcut.
	var fs := DccTheme.menu("fs_item", touch)
	popup.add_theme_font_size_override("font_size", fs)
	## **Row pitch.** `PopupMenu` sizes a row to its font height and nothing
	## else -- there is no per-item minimum -- so the canvas's `padding:6px 14px`
	## (a 28.7 px row) and the tablet's stated `min-height:44px` can only be
	## reached through `v_separation`. That constant is dead space *between*
	## rows, though, and the `hover` box is drawn on the row rect alone, so a
	## bare separation would give a tall menu with a short highlight bar. The
	## box's `expand_margin` claims the gap back, and the two together draw the
	## canvas's full-bleed padded row.
	##
	## Measured before this change: a desktop row was 21 px against the
	## canvas's 28.7, and a **tablet** row was the same 21 px against a stated
	## floor of 44 -- every menu row on the tablet was less than half a target.
	var f: Font = popup.get_theme_font("font")
	var line: float = f.get_height(fs) if f != null else float(fs)
	var gap: int = maxi(2, DccTheme.menu("pitch", touch) - int(ceil(line)))
	popup.add_theme_constant_override("v_separation", gap)
	## The highlighted item. Godot's stock `hover` box is a blue selection bar,
	## which is what a real menu capture showed 2026-08-25 -- the one saturated
	## colour anywhere in a shell whose entire palette is greys plus one amber.
	## The canvas's own hovered row is `background:rgba(224,163,74,.10);
	## color:#e8ebec` (and `rgba(164,101,15,.10)` / `#111210` in light) -- see
	## `DccTheme.menu_highlight()` for why that is not `accent_wash`.
	var hov := DccTheme.flat(DccTheme.menu_highlight())
	hov.expand_margin_top = gap / 2.0
	hov.expand_margin_bottom = gap / 2.0
	popup.add_theme_stylebox_override("hover", hov)
	popup.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	## **A labelled separator is the canvas's group band**, not a rule with a
	## caption: `padding:9px 14px 4px;font:9px 'IBM Plex Mono';
	## letter-spacing:.18em;color:#5f6468` over `STORAGE LOCATIONS`,
	## `ACTIVE PACK`, `EDIT`, `BATCH · n SELECTED`, `BUILD`, `IMPORT`, `EXPORT`,
	## `SOURCES` and `VALIDATION`. Godot draws that from `font_separator`,
	## `font_separator_size` and `font_separator_color`, none of which was set:
	## the band inherited the prose face at 13 px in `text_faint`, one size up
	## and one step bright, and every `add_separator()` in `menus.gd` was
	## unlabelled anyway.
	##
	## `phone_menu.gd`'s header names this as its own one honest shortfall --
	## "the moment a separator is given text it becomes a titled band with no
	## change to this file" -- so labelling them fixes the phone's L3 bands in
	## the same stroke.
	popup.add_theme_font_override("font_separator", DccTheme.mono(2))
	popup.add_theme_font_size_override("font_separator_size",
		DccTheme.menu("fs_group", touch))
	popup.add_theme_color_override("font_separator_color", DccTheme.c("text_ghost"))
	## `height:1px;background:rgba(255,255,255,.09);margin:5px 0` on the
	## canvas's own menu rules. `StyleBoxLine`, not `StyleBoxFlat`: a Flat box
	## in the `separator` slot fills the separator's whole reserved band.
	var sep := StyleBoxLine.new()
	sep.color = DccTheme.c("line_soft")
	sep.thickness = 1
	popup.add_theme_stylebox_override("separator", sep)
	_style_popup_marks(popup, fs)

## **The check column.** `GUI_GAP_REGISTER.md` §51 row 70: the canvas marks a
## chosen row with a typographic `●` and an unchosen one with `○`
## (`DccIcons.SYMBOLS["on"]`/`["off"]`, the same pair `phone_menu.gd` already
## draws in its own rows), and the shell was leaving Godot's stock radio and
## check icons -- a blue-tinted disc and a boxed tick from the engine's default
## theme, which is the last stock artwork left in a shell whose palette is greys
## plus one amber.
##
## Godot draws that column from four **theme icons**, so this is the one place
## a typographic mark has to arrive as a texture. Drawn rather than rasterised
## out of the font: a filled disc *is* `●` and a hairline ring *is* `○`, at the
## exact ink the palette says, repainted on a theme switch because
## `_recolor_subtree()` cannot reach inside a `Texture2D`.
##
## Sized to the item's own type (`fs`), so the marks scale with the tablet's
## 14 px rows the same way the labels beside them do.
static func _style_popup_marks(popup: PopupMenu, fs: int) -> void:
	var px := maxi(6, int(round(fs * 0.62)))
	var on := _round_dot(px, DccTheme.c("accent"))
	var off := _round_ring(px, DccTheme.c("text_ghost"))
	popup.add_theme_icon_override("radio_checked", on)
	popup.add_theme_icon_override("radio_unchecked", off)
	popup.add_theme_icon_override("checked", on)
	popup.add_theme_icon_override("unchecked", off)
	## The disabled pair exists too, and left alone it falls back to the stock
	## artwork -- the same "one row in twenty still draws the engine's own icon"
	## trap `style_popup()` itself was written to close.
	var dim := _round_dot(px, DccTheme.c("text_ghost"))
	popup.add_theme_icon_override("radio_checked_disabled", dim)
	popup.add_theme_icon_override("radio_unchecked_disabled", off)
	popup.add_theme_icon_override("checked_disabled", dim)
	popup.add_theme_icon_override("unchecked_disabled", off)

## `○` -- a hairline ring, the outlined twin of `_round_dot()`.
static func _round_ring(px: int, color: Color) -> ImageTexture:
	var img := Image.create(px, px, false, Image.FORMAT_RGBA8)
	img.fill(Color(color.r, color.g, color.b, 0.0))
	var c := (px - 1) * 0.5
	var w: float = maxf(1.0, px / 9.0)  ## The stroke, ~1.2 px at 11 px type.
	for y in px:
		for x in px:
			var d := Vector2(x - c, y - c).length()
			## Coverage of a `w`-wide annulus whose outer edge is the disc rim.
			var a: float = minf(clampf(c - d + 0.5, 0.0, 1.0),
				clampf(d - (c - w) + 0.5, 0.0, 1.0))
			if a > 0.0:
				img.set_pixel(x, y, Color(color.r, color.g, color.b, a * color.a))
	return ImageTexture.create_from_image(img)

static func number(parent: Control, label_text: String, minimum: float, maximum: float,
		step: float, value: float, on_change: Callable, tooltip: String = "") -> SpinBox:
	var row := _row(parent, label_text, tooltip)
	var sb := SpinBox.new()
	sb.min_value = minimum
	sb.max_value = maximum
	sb.step = step
	sb.value = value
	sb.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if DccTheme.is_tablet():
		sb.custom_minimum_size.y = DccTheme.role_px("btn_min_h")
	sb.value_changed.connect(func(v: float): on_change.call(v))
	row.add_child(sb)
	return sb

## The action a group commits with. §4 and §7 both put it *inside* the group it
## belongs to, never floating at the panel foot.
## An action: "Run stage 04", "commit pass", "Apply", "New seed".
##
## **Outlined, never filled** -- corrected 2026-08-25 against the canvas rather
## than against this file's own previous belief. `modal_button()` below used to
## justify itself by saying a dock action "draws a filled accent slab, which is
## the left dock's own run-this-pass affordance"; a search of
## `design/Cartalith DCC Shell.dc.html` for `background:#e0a34a` returns
## exactly one non-slider hit in the whole 1920-wide document, and it is a
## *selected layer row* in the layers popover, not a button. Every action in
## every artboard is the same chip:
##
##   padding:4px 10px; border:1px solid #e0a34a; color:#e0a34a   (primary)
##   padding:4px 10px; border:1px solid rgba(255,255,255,.16)    (secondary)
##
## So the distinction this helper drew against `modal_button()` was a
## distinction the design does not make. What survives is the padding: a dock
## action is `4px 10px`, a modal's is `8px 18px`. Radius was 2 and is now 0
## per §11's "Radius 0 everywhere".
## Marks an `action()` button so `DccShell.phone_fit()` can find it and swap the
## desktop chip for the 412 canvas's 48 dp pill. The *primary* flag rides along
## because the pill's two variants differ by fill, not by size.
const ACTION_META := "dcc_action_primary"

## §57's tier A: "commit/discard, transport, speed" -- the single factory
## behind all of them, so `role_px("btn_min_h")` (`[0, 44]`) resolved here
## reaches every one at once rather than needing a per-call-site fix. Padding
## grows with it (`btn_pad_x`/`btn_pad_y`, `[11, 18]`/`[3, 9]`) so a 44 px-tall
## button does not read as a tiny label adrift in a tall box; the desktop
## figures (10/4) are the pre-existing literals, kept as-is since they are not
## quite the same as `ROLE`'s own desktop pair and this pass changes tablet
## only.
static func action(parent: Control, text: String, on_press: Callable,
		primary: bool = false) -> Button:
	var b := Button.new()
	b.text = text
	## DS-03's reflow. A `Button`'s minimum width is the width of its whole
	## label, and eight `action()` call sites carry a *sentence* -- the
	## cross-reference signposts, "Claim hatching and the influence ramp ->
	## Layers > Claim hatch" and its seven siblings. Measured, that one is
	## 753 px of minimum inside a 400 px dock; the dock's `ScrollContainer`
	## has `horizontal_scroll_mode = SCROLL_MODE_DISABLED`, which folds the
	## child's minimum into the container's own, so the number propagated all
	## the way out and the left dock *grew* to swallow the map (measured
	## 400 -> 555 px on CIVIL > Factions, 400 -> 1589 px on CARTO > Labels).
	## That is the fourth instance of `MISTAKES.md`'s disabled-axis trap.
	##
	## Wrapping rather than clipping, because the owner's DS-03 ruling is
	## "keep everything, reflow only": an ellipsis would delete the half of
	## the sentence that names the destination. In a column a wrapped button
	## still draws at the full dock width -- it is `SIZE_FILL` there and
	## autowrap only lowers its *minimum* -- so the eight that did not fit move
	## and nothing else does.
	##
	## **Only in a column, and this guard was added after measuring the damage
	## without it.** `set_tool_options()` hands this factory the tool-options
	## bar's own `HBoxContainer` (`app.gd::_tool_options_generate()` builds
	## five buttons straight into `row`), and there a collapsed minimum is
	## exactly the wrong answer: the row shares its width between children, so
	## every label wrapped and the 40 px band grew to **265 px**, taking 225 px
	## off the map on both WORLD modes. Measured, not reasoned -- the first
	## version of this line had no guard and `_ds03shot_probe.gd` caught it.
	## This is `MISTAKES.md`'s `clip_text` entry in its other form: a text
	## control's minimum width is load-bearing wherever a sibling competes for
	## the same axis.
	## `GridContainer` was missing from the first version of this guard and a
	## verifier caught it: a grid shares its width between COLUMNS exactly as an
	## `HBoxContainer` shares it between children, but it is neither a
	## `BoxContainer` nor an `HFlowContainer`, so its buttons wrapped. The test
	## is "does a sibling compete for my width", not "which class is my parent",
	## and `GridContainer` answers yes.
	var horizontal_parent := (parent is BoxContainer and not (parent as BoxContainer).vertical) 		or parent is HFlowContainer 		or parent is GridContainer
	if not horizontal_parent:
		b.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	b.focus_mode = Control.FOCUS_NONE
	b.set_meta(ACTION_META, primary)
	var act_tablet := DccTheme.is_tablet()
	b.custom_minimum_size.y = DccTheme.role_px("btn_min_h") if act_tablet else 26
	b.add_theme_font_size_override("font_size",
		DccTheme.role_px("fs_readout") if act_tablet else DccTheme.FS_SMALL)
	b.add_theme_font_override("font", DccTheme.mono(1))
	var edge := "accent" if primary else "border"
	b.add_theme_color_override("font_color",
		DccTheme.c("accent") if primary else DccTheme.c("text"))
	b.add_theme_color_override("font_hover_color", DccTheme.c(
		"accent_hover" if primary else "text_bright"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	var pad_x := DccTheme.role_px("btn_pad_x") if act_tablet else 10
	var pad_y := DccTheme.role_px("btn_pad_y") if act_tablet else 4
	var rest := DccTheme.outline(edge)
	rest.content_margin_left = pad_x
	rest.content_margin_right = pad_x
	rest.content_margin_top = pad_y
	rest.content_margin_bottom = pad_y
	b.add_theme_stylebox_override("normal", rest)
	b.add_theme_stylebox_override("disabled", rest)
	var lit := DccTheme.outline(edge, "accent_wash" if primary else "line_soft")
	lit.content_margin_left = pad_x
	lit.content_margin_right = pad_x
	lit.content_margin_top = pad_y
	lit.content_margin_bottom = pad_y
	b.add_theme_stylebox_override("hover", lit)
	b.add_theme_stylebox_override("pressed", lit)
	b.pressed.connect(on_press)
	parent.add_child(b)
	return b

## The button a *modal* commits or dismisses with -- "Cancel" / "Open
## selected" / "Use this folder" on the two file-dialog screens in
## `design/Cartalith DCC Shell.dc.html`.
##
## The same outline `action()` now draws, at the modal's own larger padding:
## `padding:8px 18px;border:1px solid #e0a34a;color:#e0a34a;font-size:12px` on
## "Open selected", `rgba(255,255,255,.16)` on "Cancel". The paragraph that
## used to sit here claimed a dock action was a filled accent slab and that
## keeping the two apart was the point -- see `action()` above for why that was
## wrong about the canvas. The two are the same chip at two sizes.
## §57 tier A: a modal's Open/Cancel pair is exactly "commit/discard" at a
## larger size, so it takes `role_px("btn_min_h")` the same way `action()`
## does. `open_project_dialog.gd`'s Welcome gate is the one modal the tablet
## probe actually opens by default, and its "Open selected"/"Continue without
## a world" pair (30 px) was two of the small number of genuine violations
## left standing after every dock fix, because the factory itself, not a call
## site this pass owns, was where the figure lived.
static func modal_button(parent: Control, text: String, on_press: Callable,
		primary: bool = false) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_NONE
	var modal_h := float(DccTheme.role_px("btn_min_h")) if DccTheme.is_tablet() else 30.0
	b.custom_minimum_size = Vector2(0, modal_h)
	b.add_theme_font_size_override("font_size", DccTheme.FS_BODY)
	var token := "accent" if primary else "border"
	var fg := "accent" if primary else "text"
	b.add_theme_color_override("font_color", DccTheme.c(fg))
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	var rest := DccTheme.outline(token)
	rest.content_margin_left = 18
	rest.content_margin_right = 18
	rest.content_margin_top = 8
	rest.content_margin_bottom = 8
	b.add_theme_stylebox_override("normal", rest)
	b.add_theme_stylebox_override("pressed", rest)
	b.add_theme_stylebox_override("disabled", rest)
	var hover := DccTheme.outline(token, "accent_wash" if primary else "line_soft")
	hover.content_margin_left = 18
	hover.content_margin_right = 18
	hover.content_margin_top = 8
	hover.content_margin_bottom = 8
	b.add_theme_stylebox_override("hover", hover)
	b.pressed.connect(on_press)
	parent.add_child(b)
	return b

## §4.5's tool palette. One square icon button, toggle-style, joined into
## `group` (a shared `ButtonGroup` -- `DccApp.tool_group`, the SAME instance
## across every domain's TOOLS block) so arming a tool anywhere disarms it
## everywhere else, and switching domains never loses the armed state: the
## button that's actually pressed simply isn't present in whichever domain's
## dock isn't currently visible. That is the whole mechanism `UI_SHELL_DESIGN
## .md`'s "one tool is armed at a time, globally" needs -- no extra
## bookkeeping beyond every tool button belonging to one group.
##
## The two metas are the phone's half of this widget, and they live here
## because this is the file that knows the glyph's *name* -- an `ImageTexture`
## rasterised at 15 px cannot be grown afterwards without resampling, so
## `DccShell.phone_fit()` has to re-render from the SVG, and for that it needs
## the name back. `TOOL_CAPTION_META` is set by `tools_block()` only; see there
## for why the feature picker does not get one.
const TOOL_GLYPH_META := "dcc_tool_glyph"
const TOOL_CAPTION_META := "dcc_tool_caption"

static func tool_button(parent: Control, glyph: String, label_text: String,
		group: ButtonGroup, on_armed: Callable) -> Button:
	var b := Button.new()
	b.toggle_mode = true
	b.button_group = group
	b.focus_mode = Control.FOCUS_NONE
	b.tooltip_text = label_text
	b.set_meta(TOOL_GLYPH_META, glyph)
	## A square icon tool button is a discrete single-tap target -- tier A --
	## the same as `action()`, floored to `role_px("btn_min_h")` on both
	## dimensions on tablet rather than the desktop's fixed `30x30`.
	var tb_size := DccTheme.role_px("btn_min_h") if DccTheme.is_tablet() else 30
	b.custom_minimum_size = Vector2(tb_size, tb_size)
	b.icon = DccIcons.get_icon(glyph, 15)
	b.expand_icon = false
	b.add_theme_stylebox_override("normal", DccTheme.empty())
	## Radius 0 per §11, like everything else. Was 2.
	b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	b.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("accent_wash")))
	b.add_theme_color_override("icon_normal_color", DccTheme.c("text_dim"))
	b.add_theme_color_override("icon_hover_color", DccTheme.c("text_bright"))
	b.add_theme_color_override("icon_pressed_color", DccTheme.c("accent"))
	b.toggled.connect(func(on: bool): if on: on_armed.call())
	parent.add_child(b)
	return b

## The TOOLS block itself (§4.5: "every left dock opens with a TOOLS block:
## first the four global tools, then that domain's own"). `global_only` skips
## the domain-specific row for a caller with none yet. `entries` is
## `[{glyph, label, id}, ...]`; arming calls `app.arm_tool(id)`.
static func tools_block(parent: Control, app, group: ButtonGroup,
		domain_entries: Array = []) -> void:
	var sec := section(parent, "Tools")
	sec.add_child(_tools_row(GLOBAL_TOOL_ENTRIES, app, group))
	if not domain_entries.is_empty():
		sec.add_child(_tools_row(domain_entries, app, group))
	parent.add_child(DccTheme.rule())

## `HFlowContainer`, not `HBoxContainer`, and only because of the phone: on a
## handset `DccShell.phone_fit()` puts each tool's *name* under its glyph
## (they are otherwise unlabelled marks, and touch has no hover to name them
## with), which makes a row several times wider than the four 30 px squares it
## was authored as. A `BoxContainer` handed more minimum width than it has does
## not clip -- it *overlaps*, so the last tool would sit on top of its
## neighbour rather than moving to a second line. Nothing changes on desktop:
## a flow container with room for every child lays it out identically.
static func _tools_row(entries: Array, app, group: ButtonGroup) -> Control:
	var row := HFlowContainer.new()
	row.add_theme_constant_override("h_separation", 2)
	row.add_theme_constant_override("v_separation", 2)
	for e in entries:
		if e.has("legend"):
			_tool_legend(row, String(e["glyph"]), String(e["label"]), String(e["legend"]))
			continue
		var b := tool_button(row, e["glyph"], e["label"], group, func(): app.arm_tool(e["id"]))
		## The TOOLS block is the one caller whose labels are short enough to
		## draw under a glyph. `world_workspace.gd`'s feature picker uses the
		## same widget with a whole hint sentence appended, in a 5-column
		## grid, so it gets the touch size and the border and no caption.
		b.set_meta(TOOL_CAPTION_META, tool_caption(String(e["label"])))
		## `GUI_GAP_REGISTER.md` IN-11. Every one of these labels has advertised
		## a letter since the TOOLS block was first built -- "Way (W)",
		## "Route (⇧R)", "Label (L)", "Biome paint (B)" -- and until now not one
		## of them was bound to anything: no `_unhandled_key_input` branch, no
		## `Shortcut`, nothing anywhere in `shell/` matched a bare letter. The
		## tooltip was the whole feature. That is exactly the fake control this
		## port's discipline exists to avoid, and it is a plausible half of the
		## owner's own "there is no way to draw a route" (2026-08-24).
		##
		## A `Shortcut` on the button rather than a key table on `app.gd`, for
		## one reason that is not style: `BaseButton::shortcut_input` fires only
		## when the button `is_visible_in_tree()` and is not disabled. Only the
		## active domain's panel is visible (`DccShell._select_domain`), so `W`
		## arms Way exactly when CIVIL is showing and is inert in WORLD --
		## which is the rule we want and would otherwise have to re-derive by
		## hand. It also lands *after* GUI input, so a focused `LineEdit` eats
		## its own letters first and typing a settlement name never arms a tool.
		##
		## `shortcut_in_tooltip` off: the tooltip already spells the key in the
		## mockup's own notation (`⇧R`), and Godot would append a second,
		## differently-spelled copy ("Shift+R") under it.
		var sc := _tool_shortcut(String(e["label"]))
		if sc != null:
			b.shortcut = sc
			b.shortcut_in_tooltip = false
	return row

## The fourth global cell. `02-rail-and-domains.md` §4d and
## `01-frame-and-tokens.md` §3.6c both draw `pan` in the same four-square row
## as the three real tools and both say what it is: *"the pan button is
## permanently `bg:var(--ins)` / `col:var(--dis)` -- it is a legend, not a
## button"*. It arms nothing and joins no `ButtonGroup`, because panning is
## never armed: it is on the wheel, the middle drag and the pinch at all times,
## in every domain, whatever tool is live. Drawn here so the palette *says*
## that -- the desktop shell says it nowhere today (there is a pan mode on the
## touch navpad, `viewport_host.gd`, and no pan cell in any TOOLS block), which
## leaves "how do I pan?" answerable only by trying it.
##
## `disabled` is what makes it inert, and it is also what paints it: Godot uses
## the `disabled` stylebox and `icon_disabled_color`, which is exactly the
## `ins` ground / `dis` ink pair the spec asks for, in one state that no hover
## or press can move off. The tooltip carries the spec's own sentence, so the
## reason it cannot be armed is where a user meets it -- `menus.gd`'s rule for
## every other inert control in this shell.
##
## It still takes `TOOL_GLYPH_META`/`TOOL_CAPTION_META`: `DccShell.phone_fit()`
## re-rasterises every tool glyph from its name and captions it, and a legend
## that stayed a 15 px mark beside four captioned squares would read as a
## rendering fault rather than as the row's fourth member.
static func _tool_legend(parent: Control, glyph: String, caption: String, tip: String) -> Button:
	var b := Button.new()
	b.disabled = true
	b.focus_mode = Control.FOCUS_NONE
	b.tooltip_text = tip
	b.set_meta(TOOL_GLYPH_META, glyph)
	b.set_meta(TOOL_CAPTION_META, caption)
	var tb_size := DccTheme.role_px("btn_min_h") if DccTheme.is_tablet() else 30
	b.custom_minimum_size = Vector2(tb_size, tb_size)
	b.icon = DccIcons.get_icon(glyph, 15)
	b.expand_icon = false
	b.add_theme_stylebox_override("disabled", DccTheme.flat(DccTheme.c("sunken")))
	b.add_theme_color_override("icon_disabled_color", DccTheme.c("text_ghost"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	parent.add_child(b)
	return b

## `"Route (⇧R)"` -> a `Shortcut` for Shift+R; `"Inspect (V)"` -> plain V.
## Returns `null` for any label whose parenthetical is not a single A-Z letter
## with an optional `⇧`, so a caller that writes something else gets no
## shortcut rather than a wrong one.
static func _tool_shortcut(label_text: String) -> Shortcut:
	var open_i := label_text.find(" (")
	var close_i := label_text.rfind(")")
	if open_i < 0 or close_i <= open_i + 2:
		return null
	var body := label_text.substr(open_i + 2, close_i - open_i - 2)
	var shift := body.begins_with("⇧")
	if shift:
		body = body.substr(1)
	if body.length() != 1:
		return null
	var code := body.to_upper().unicode_at(0)
	if code < KEY_A or code > KEY_Z:
		return null
	var ev := InputEventKey.new()
	ev.keycode = code as Key
	ev.shift_pressed = shift
	var sc := Shortcut.new()
	sc.events = [ev]
	return sc

## `"Region select (R)"` -> `"Region select"`. The entry's `label` is written
## for a tooltip: it carries a keyboard shortcut, and the device that needs the
## caption is the one with no keyboard to press it on.
static func tool_caption(label_text: String) -> String:
	var s := label_text.split(" -- ")[0]
	var paren := s.find(" (")
	return (s.substr(0, paren) if paren > 0 else s).strip_edges()

## §4.5.1 -- present in every domain, identical everywhere. Four cells, and
## the fourth is not a tool: an entry carrying `legend` is drawn by
## `_tool_legend()` instead of `tool_button()` -- see there.
const GLOBAL_TOOL_ENTRIES: Array = [
	{"id": "inspect", "glyph": "tool_inspect", "label": "Inspect (V)"},
	{"id": "measure", "glyph": "tool_measure", "label": "Measure (M)"},
	{"id": "region", "glyph": "tool_region", "label": "Region select (R)"},
	{"id": "pan", "glyph": "tool_pan", "label": "Pan",
		"legend": "Pan / zoom — always available"},
]

## Prose that explains a rule rather than labelling a control. Kept narrow so a
## dock at its minimum width still wraps sensibly -- but narrow enough to fit
## *inside* that minimum, unlike the 240 (272 with margins) this used to carry
## against the right dock's own documented floor, `DccTheme.W_RIGHT_DOCK_MIN`
## (260, `right_dock.gd`'s many `note()` calls). `section()`'s own padding
## above takes 26 px off that floor (14 left + 12 right) and a `group()`
## nested inside one more section takes 10 more, leaving 223 px in the
## tightest real case (`right_dock.gd`'s Measure ▸ Actions, a note inside a
## group inside a section) -- so 190 keeps clearance for the right dock's
## ScrollContainer to grow a vertical scrollbar without re-opening the same
## fixed-floor-fights-the-dock bug PARITY_AUDIT.md's pass 2 (F8) found here,
## and 695821f fixed one call site up in `_field()`'s value labels.
static func note(parent: Control, text: String) -> Label:
	## `role_px("fs_prose")` on tablet -- a note is prose (`DccTheme.label()`,
	## no font override) like any other dock row, so it takes the same floor
	## `_row()`'s own label does. Resolved here rather than left to
	## `DccShell.tablet_fit()`'s walk for the same reason `DccTheme.header()`
	## now is: `right_dock.gd`'s `note()` calls sit in a dock that walk never
	## reaches.
	var fs := DccTheme.role_px("fs_prose") if DccTheme.is_tablet() else DccTheme.FS_MICRO
	var l := DccTheme.label(text, "text_ghost", fs)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.custom_minimum_size.x = 190
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(l)
	return l

## The mark a stage carries when something upstream changed. Non-destructive by
## default: editing a stage marks everything downstream stale rather than
## silently invalidating it.
##
## **No caller** (checked repo-wide 2026-09-01), on exactly the terms
## `stage_category()` above states for itself: this is the mark that row
## type carries, v3 replaced the numbered ten-stage list it belonged to, and
## the two are dead together rather than separately. Staleness is disclosed
## today by the status bar's own `stale` slot (`app.gd`'s
## `refresh_staleness()`), which is a different presentation of the same
## fact and does not use this. Same expiry as its row type: if nothing has
## claimed either by the time WORLD is next reworked, delete both then.
static func stale_mark(parent: Control) -> Label:
	var l := DccTheme.label("stale", "stale", DccTheme.FS_TINY)
	l.visible = false
	parent.add_child(l)
	return l

# ---------------------------------------------------------------------------
# The design canvas's *window* vocabulary
#
# `design/Cartalith DCC Shell.dc.html`'s workspace-window screens (`Asset
# library window 1920`, `Data manager window 1920`) draw exactly five controls,
# and none of them is a stock Godot widget:
#
#   chip         `padding:4px 9px; border:1px solid rgba(255,255,255,.16)`
#   segment      the narrower `padding:3px 8px` variant; one of a set is lit
#   well         a bordered text field, Plex Mono at 10-10.5 px
#   text button  borderless, ghost -- the grid header's batch verbs
#   band         a 28 px column header: ground, bottom hairline, padded row
#
# These were written as private statics in `asset_library_window.gd` during
# that window's 2026-08-20 rebuild, with the note *"built here rather than in
# `dcc_widgets.gd` because nothing else in the shell draws them yet; if a
# second window needs them, they move."* The Data manager rebuild is that
# second window, so they moved. `asset_library_window.gd` keeps its private
# names as one-line delegators, so none of its 74 call sites changed.
#
# The docks above use `section`/`group`/`row`; these are for a window's own
# chrome. Nothing here computes -- every one is presentation only.
# ---------------------------------------------------------------------------

static func box(border_token: String, bg_token: String, px: int, py: int) -> StyleBoxFlat:
	var sb := DccTheme.outline(border_token, bg_token)
	sb.content_margin_left = px
	sb.content_margin_right = px
	sb.content_margin_top = py
	sb.content_margin_bottom = py
	return sb

## The canvas's ubiquitous outline chip. `accent` swaps both border and text.
static func chip(parent: Control, text: String, on_press: Callable,
		accent: bool = false, px: int = 9, py: int = 4) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_NONE
	b.add_theme_font_override("font", DccTheme.mono(0))
	b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	## `.16`, the control edge -- not `line` (`.10`), which is the *region*
	## hairline. The header block above quotes the canvas correctly
	## (`border:1px solid rgba(255,255,255,.16)`) and then the code used the
	## wrong token, so every chip in every window was drawn 6 points fainter
	## than the design and read as an outline that wasn't quite there.
	var token := "accent" if accent else "border"
	b.add_theme_color_override("font_color", DccTheme.c("accent") if accent else DccTheme.c("text"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	var rest := box(token, "", px, py)
	b.add_theme_stylebox_override("normal", rest)
	b.add_theme_stylebox_override("pressed", rest)
	b.add_theme_stylebox_override("disabled", box("line_soft", "", px, py))
	b.add_theme_stylebox_override("hover",
		box(token, "accent_wash" if accent else "line_soft", px, py))
	if on_press.is_valid():
		b.pressed.connect(on_press)
	parent.add_child(b)
	return b

## The narrower `padding:3px 8px` chip -- the canvas's Scheme / Zoom range /
## CRS / Packaging rows, where one of a set is lit and the rest are quiet.
##
## §57's tier B: "mode chips (raise/lower/smooth)". `chip()` below is shared
## with dozens of call sites this pass does not own (window batch-action
## chips that have no artboard evidence they should grow), so the tier-B floor
## is applied here, one level up, where the caller has told us -- by calling
## `segment()` rather than `chip()` -- that this button *is* one of a lit set.
## `role_px("chip_min_h")` (`[0, 34]`) and `chip_pad_x`/`chip_pad_y`
## (`[9, 16]`/`[3, 9]`) are the pair §57 measured for exactly this control.
static func segment(parent: Control, text: String, on_press: Callable) -> Button:
	var seg_tablet := DccTheme.is_tablet()
	var px := DccTheme.role_px("chip_pad_x") if seg_tablet else 8
	var py := DccTheme.role_px("chip_pad_y") if seg_tablet else 3
	var b := chip(parent, text, on_press, false, px, py)
	b.add_theme_font_size_override("font_size",
		DccTheme.role_px("fs_readout") if seg_tablet else DccTheme.FS_TINY)
	b.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	if seg_tablet:
		b.custom_minimum_size.y = DccTheme.role_px("chip_min_h")
	return b

## A lit segment has to survive being `disabled` -- Godot resolves the
## `disabled` stylebox and `font_disabled_color` ahead of the `normal` pair, so
## a lit-but-disabled segment (the Data manager draws several: the one real
## scheme among three impossible ones) would otherwise be painted exactly like
## the impossible ones.
## **Padding matches `segment()`'s own tablet figure, not a bare `8, 3`.**
## `add_theme_stylebox_override` *replaces* the box `segment()` built, so
## calling this afterward (the normal pattern -- every real caller does) used
## to silently put the desktop padding back on a tablet chip, leaving the
## height floor `segment()` set the only surviving part of the fix.
static func set_segment_on(b: Button, on: bool) -> void:
	var token := "accent" if on else "border"
	var fg := DccTheme.c("accent") if on else DccTheme.c("text_dim")
	var seg_px := DccTheme.role_px("chip_pad_x") if DccTheme.is_tablet() else 8
	var seg_py := DccTheme.role_px("chip_pad_y") if DccTheme.is_tablet() else 3
	## **A lit segment carries the accent wash behind its border.** `Cartalith
	## Paint Toolbar.dc.html`'s `Sculpt raise 1920` draws the armed feature as
	## `border:1px solid #e0a34a;color:#e0a34a;background:rgba(224,163,74,.10)`
	## and every unlit sibling as `border:1px solid rgba(255,255,255,.16)` with
	## no fill. The border and the ink were already right; the wash was missing,
	## which is why "which one is armed" read as a hairline colour change on a
	## row of eight identical chips. Not a filled surface -- see
	## `set_mode_segment_on()` below for the one segment that is.
	var wash := "accent_wash" if on else ""
	for sb_name in ["normal", "pressed", "disabled"]:
		b.add_theme_stylebox_override(sb_name, box(token, wash, seg_px, seg_py))
	b.add_theme_color_override("font_color", fg)
	b.add_theme_color_override("font_disabled_color",
		fg if on else DccTheme.c("text_ghost"))

## The tool bar's three **mode** segments -- SCULPT / PAINT / MEASURE -- and
## nothing else.
##
## `GUI_GAP_REGISTER.md` §48 (DS-02) removed every filled amber slab in the
## shell after finding that a search of `DCC shell 1920` for
## `background:#e0a34a` returns slider fills and one selected layer row, and
## that is still true of that artboard. The **Paint Toolbar** canvas is a later
## artboard of a component that one does not draw, and it fills exactly one
## thing: `padding:5px 12px;border:1px solid #e0a34a;color:#141617;
## background:#e0a34a;letter-spacing:.12em` on the active mode. Reversed
## paper-coloured type on accent is §11's own rule for a filled accent surface,
## so this is the design's grammar rather than an exception to it.
##
## Kept as its own call, not a flag on `set_segment_on()`, so the fill cannot
## spread back to the 141 call sites DS-02 cleared.
static func set_mode_segment_on(b: Button, on: bool) -> void:
	if not on:
		set_segment_on(b, false)
		return
	var seg_px := DccTheme.role_px("chip_pad_x") if DccTheme.is_tablet() else 8
	var seg_py := DccTheme.role_px("chip_pad_y") if DccTheme.is_tablet() else 3
	var filled := box("accent", "accent", seg_px, seg_py)
	for sb_name in ["normal", "pressed", "disabled", "hover"]:
		b.add_theme_stylebox_override(sb_name, filled)
	## See `accent_ink`'s own comment in `dcc_theme.gd`: this was `c("bg")`
	## until the 2026-08-31 re-base, i.e. #0d0e0f ink on an #e0a34a fill.
	b.add_theme_color_override("font_color", DccTheme.c("accent_ink"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("accent_ink"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("accent_ink"))

## The phone form of `action()`: `design/Cartalith Android Phone.dc.html`'s
## `height:48px;border-radius:24px`, primary filled `#e0a34a` with `#141617`
## type, secondary the same box outlined at `rgba(255,255,255,.16)`, both
## `font:500 11px 'IBM Plex Mono';letter-spacing:.16em` in upper case.
##
## Called from `DccShell.phone_fit()` and nowhere else, so a desktop or tablet
## build never sees a rounded button and the 141 call sites `GUI_GAP_REGISTER.md`
## §48 (DS-02) cleared of accent fills stay cleared -- the fill here is the
## phone canvas's own, on the phone only.
##
## `px` is `phone_fit()`'s unit: what one authored pixel is worth in this
## subtree's space. Everything below is authored in 412 dp and multiplied by it.
static func phone_pill(b: Button, unit: float) -> void:
	var primary: bool = bool(b.get_meta(ACTION_META, false))
	var h := int(round(DccTheme.H_PHONE_PILL * unit))
	var r := int(round(DccTheme.H_PHONE_PILL * 0.5 * unit))
	var pad_x := int(round(16.0 * unit))
	b.custom_minimum_size.y = maxf(b.custom_minimum_size.y, float(h))
	b.text = b.text.to_upper()
	b.add_theme_font_override("font", DccTheme.mono(maxi(1, int(round(2.0 * unit))), true))
	b.add_theme_font_size_override("font_size", maxi(1, int(round(11.0 * unit))))
	var rest := DccTheme.pill(primary, r, pad_x, 0)
	var lit := DccTheme.pill(primary, r, pad_x, 0)
	if primary:
		lit.bg_color = DccTheme.c("accent_hover")
	else:
		lit.bg_color = DccTheme.c("line_soft")
	for sb_name in ["normal", "disabled"]:
		b.add_theme_stylebox_override(sb_name, rest)
	for sb_name in ["hover", "pressed"]:
		b.add_theme_stylebox_override(sb_name, lit)
	b.add_theme_stylebox_override("focus", DccTheme.empty())
	## Reversed paper ink on the filled pill -- `c("accent_ink")` since the
	## 2026-08-31 re-base, and `c("bg")` before it, which was the literal
	## `#141617` before *that*. Each step is the same correction taken one
	## notch further: a theme switch has to repaint it, AND it has to be a
	## colour chosen to sit on amber rather than one that happens to be dark.
	## `#c8cbcd` on the outlined one, unchanged.
	var fg := DccTheme.c("accent_ink") if primary else DccTheme.c("text")
	b.add_theme_color_override("font_color", fg)
	b.add_theme_color_override("font_hover_color", fg)
	b.add_theme_color_override("font_pressed_color", fg)
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))

## The phone form of a `slider()` track: `height:3px` with a `22x22` round
## accent thumb in a `32px` row.
##
## The dock's slider deliberately has **no grabber** -- §11's "a 2 px rule, the
## travelled part in accent, and no grabber", which is right for a pointer and
## is what `_style_slider()` builds. The 412 phone canvas draws a thumb on every
## slider it has, in the position the value is at, because a finger has no
## cursor to tell it where the handle is. Rasterised as a circle rather than
## taken from Godot's stock grabber, which is a fixed bitmap this shell cannot
## recolour for the light palette -- the same reason `phone_menu.gd` builds its
## switch out of two rounded styleboxes.
static func phone_slider(s: HSlider, unit: float) -> void:
	var thumb := maxi(4, int(round(DccTheme.PHONE_SLIDER_THUMB * unit)))
	var track := maxi(1, int(round(DccTheme.PHONE_SLIDER_TRACK * unit)))
	s.custom_minimum_size.y = maxf(s.custom_minimum_size.y,
		float(maxi(thumb, int(round(DccTheme.PHONE_SLIDER_ROW * unit)))))
	var bar := StyleBoxFlat.new()
	bar.bg_color = DccTheme.c("line")
	bar.content_margin_top = track / 2
	bar.content_margin_bottom = track - track / 2
	s.add_theme_stylebox_override("slider", bar)
	var filled := StyleBoxFlat.new()
	filled.bg_color = DccTheme.c("accent")
	s.add_theme_stylebox_override("grabber_area", filled)
	s.add_theme_stylebox_override("grabber_area_highlight", filled)
	var tex := _round_dot(thumb, DccTheme.c("accent"))
	s.add_theme_icon_override("grabber", tex)
	s.add_theme_icon_override("grabber_highlight", tex)
	s.add_theme_icon_override("grabber_disabled", _round_dot(thumb, DccTheme.c("text_ghost")))
	s.add_theme_constant_override("center_grabber", 1)

## A filled circle as an `ImageTexture`, drawn rather than loaded because this
## shell ships no bitmaps and a theme switch has to be able to redraw it.
## Antialiased by a one-pixel coverage ramp at the rim; anything cheaper reads
## as a polygon at 22 dp on a 510 ppi panel.
static func _round_dot(px: int, color: Color) -> ImageTexture:
	var img := Image.create(px, px, false, Image.FORMAT_RGBA8)
	img.fill(Color(color.r, color.g, color.b, 0.0))
	var c := (px - 1) * 0.5
	for y in px:
		for x in px:
			var d := Vector2(x - c, y - c).length()
			var a: float = clampf(c - d + 0.5, 0.0, 1.0)
			if a > 0.0:
				img.set_pixel(x, y, Color(color.r, color.g, color.b, a * color.a))
	return ImageTexture.create_from_image(img)

## An outlined text field -- the canvas's Tile size / World bounds /
## Destination wells and the Asset library's search.
static func well(le: Control, px: int = 9, py: int = 4, accent: bool = false) -> void:
	var token := "accent" if accent else "border"
	le.add_theme_stylebox_override("normal", box(token, "", px, py))
	le.add_theme_stylebox_override("focus", box("accent", "", px, py))
	le.add_theme_stylebox_override("read_only", box("line_soft", "", px, py))
	le.add_theme_font_override("font", DccTheme.mono(0))
	le.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	le.add_theme_color_override("font_color", DccTheme.c("text"))
	le.add_theme_color_override("font_placeholder_color", DccTheme.c("text_ghost"))
	le.add_theme_color_override("font_uneditable_color", DccTheme.c("text_ghost"))
	le.add_theme_color_override("caret_color", DccTheme.c("accent"))

## Borderless, ghost -- the only place in a window a button carries no outline.
static func text_button(parent: Control, text: String, on_press: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.add_theme_font_override("font", DccTheme.mono(0))
	b.add_theme_font_size_override("font_size", DccTheme.FS_MICRO)
	b.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("accent"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	b.add_theme_stylebox_override("normal", DccTheme.empty())
	b.add_theme_stylebox_override("hover", DccTheme.empty())
	b.add_theme_stylebox_override("pressed", DccTheme.empty())
	b.add_theme_stylebox_override("disabled", DccTheme.empty())
	b.pressed.connect(on_press)
	## **The phone tap floor, applied where the target is MADE.**
	##
	## `text_button` is borderless and sized by its own text, so at `FS_MICRO`
	## it lands around 28 x 13 px -- measured by `_phonechrome_probe.gd` on
	## `SectionStrip`'s "close", which is the Measure tool's only way out of
	## the profile strip and the smallest tappable thing in the phone shell.
	##
	## `DccShell._ptap()` cannot reach it: that is an instance method on the
	## shell and this is a static factory. `DccTheme.is_phone()` and
	## `phone_scale()` are published as statics for exactly this case -- see
	## their own comments.
	##
	## `DCC_SHELL_SPEC.md` §13: "Minimum target 44 px, measured inside the safe
	## area, with no exceptions." Scaled, because 44 is a reference-unit figure
	## and the phone composition is drawn at `_pscale`'s factor; an unscaled 44
	## is 16 dp on the 6T, which is the same mistake `_ptap()` carried until
	## this session.
	if DccTheme.is_phone():
		var tap := int(round(DccTheme.PHONE_TAP_MIN * DccTheme.phone_scale()))
		b.custom_minimum_size = Vector2(tap, tap)
	parent.add_child(b)
	return b

## A column header band: ground, a bottom hairline, and a horizontally padded
## row centred in it. Returns the row to fill. 28 px on both canvas screens.
static func band(parent: Control, pad_x: int, gap: int = 14, height: int = 28) -> HBoxContainer:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"bottom": 1}))
	wrap.custom_minimum_size.y = height
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", pad_x)
	pad.add_theme_constant_override("margin_right", pad_x)
	wrap.add_child(pad)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", gap)
	row.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	pad.add_child(row)
	parent.add_child(wrap)
	return row

# ---------------------------------------------------------------------------
# Phone treatment for a free-floating window
#
# Three windows landed this session as plain `AcceptDialog`s authored at
# desktop sizes -- 400x640, 880x620, 940x660 -- on a shell whose phone screen
# is 393 dp wide. Both halves of this shell's twice-recorded window bug class
# were present in all three, so both are fixed in one place rather than three:
#
#   1. `wrap_controls`, which `AcceptDialog` turns ON in its constructor. The
#      window then grows to fit its content instead of letting the content
#      scroll, and walks off the bottom of the screen -- taking the buttons
#      with it (`GUI_GAP_REGISTER.md`, and the same fix in
#      `asset_library_window.gd` and `data_manager_window.gd`). Wrong on every
#      platform, so it is applied unconditionally.
#   2. Desktop pixels on a phone. `open_project_dialog.gd` established the
#      answer and its reasoning holds unchanged here: fill the screen and let
#      `content_scale_factor` map the desktop-authored composition onto the
#      canvas's own 393 dp reference, so one layout serves both form factors
#      instead of a second set of constants per window.
#
# What that precedent does NOT solve is touch target size -- a content scale
# maps 24 authored px onto 24 *dp*, which is still half of §13's floor. That
# is `DccShell.phone_fit(dlg, 1.0)`'s job, called by each window after its
# body is built; `1.0` because the compositor has already applied the scale
# once and applying it again here would square it.
# ---------------------------------------------------------------------------

## Once, from `setup()`. Returns whether this is a phone, so the caller can
## build a stacked layout instead of a side-by-side one -- the one thing a
## content scale cannot fix, since a 264 px companion column beside a 393 dp
## body leaves the body 129 px no matter what it is scaled by.
static func phone_window(dlg: AcceptDialog, host) -> bool:
	dlg.wrap_controls = false
	if host == null or not host.has_method("is_phone") or not host.is_phone():
		return false
	## The embedded window's own title bar is drawn by the PARENT viewport, at
	## the parent's scale -- so it does not grow with `content_scale_factor` and
	## its close box lands at about 5 dp. Dropping the decoration entirely is
	## the same call `open_project_dialog.gd` made, for the same reason; each
	## window carries its own titled header inside the content, which does
	## scale, and `ok_button_text` gives the explicit way out.
	dlg.borderless = true
	dlg.ok_button_text = "Close"
	## A rotation changes both the screen this fills and the scale it fills it
	## at. `phone_insets_changed` is the shell's own "the phone layout moved"
	## signal, already emitted by `_apply_phone_orientation()`.
	##
	## Guarded and self-disconnecting, because `browse_dialog.gd` (PH-06) is
	## the first caller that does **not** live for the session: it spawns per
	## pick and frees itself on close. This lambda is created in a `static`
	## function, so it has no owning instance for Godot to auto-disconnect it
	## from -- without the guard a rotation after the dialog closed would
	## touch a freed object, and without the release every browse would leave
	## a dead connection on the shell.
	var relay := func():
		if is_instance_valid(dlg) and dlg.visible:
			phone_present(dlg, host)
	host.phone_insets_changed.connect(relay)
	dlg.tree_exiting.connect(func():
		if host.phone_insets_changed.is_connected(relay):
			host.phone_insets_changed.disconnect(relay))
	return true

## Opens the window, phone-shaped. Returns **false** on desktop and tablet,
## where the caller should go on and `popup_centered()` as it always has --
## so a call site is two lines and carries no `is_phone()` branch of its own.
##
## It opens the window rather than only sizing it, and that is load-bearing
## rather than convenience. `AcceptDialog` lays its content child out from a
## resize *notification*; assigning `size` while the dialog is hidden and then
## calling `popup_centered()` produces no such notification, so the child keeps
## the rect it last had at its desktop size -- measured at 377 x 2602 inside a
## 393 x 852 window. The visible symptom is not a cropped window but a body
## that **overflows instead of scrolling**: a `ScrollContainer` handed 2 602 px
## of height has nothing to scroll, so the bottom two thirds of a form are
## simply unreachable. `child_controls_changed()` does not fix it, and neither
## does re-setting the child's rect afterwards -- the next layout pass puts it
## back. `Window.popup(rect)` is the engine's own sized-popup entry point: it
## sets position and size *as part of* showing the window, so the notification
## arrives and `_update_child_rects()` runs against the real size.
##
## Re-run on every open (and on rotation, via `phone_insets_changed`) because
## the viewport it measures changes with both.
static func phone_present(dlg: Window, host) -> bool:
	if host == null or not host.has_method("is_phone") or not host.is_phone():
		return false
	var screen: Vector2 = host.get_viewport_rect().size
	dlg.content_scale_mode = Window.CONTENT_SCALE_MODE_CANVAS_ITEMS
	dlg.content_scale_aspect = Window.CONTENT_SCALE_ASPECT_IGNORE
	dlg.content_scale_factor = host.phone_scale()
	## Both authored for desktop, and both would otherwise fight the fill:
	## `min_size` refuses a window narrower than 620-880 px, which every phone
	## in portrait is, and `max_size` -- which two of these three windows set,
	## precisely to stop `wrap_controls` running them off a 1080p screen --
	## caps the height at 700-760 and leaves a band of map showing under the
	## window. With `wrap_controls` off above, the cap has nothing left to
	## treat, so it goes rather than cropping the fill.
	dlg.min_size = Vector2i.ZERO
	dlg.max_size = Vector2i.ZERO
	## §13: "Bottom 26 px is the gesture inset -- no tappable target inside it.
	## Timeline and sheets stop above it." A window filling the whole screen put
	## its `AcceptDialog` OK button -- which on four of these windows is the only
	## way out -- squarely in it: measured 846 dp on an 864.6 dp screen, where
	## the inset begins at 838.6. Stopping the window above the inset is what
	## the canvas draws (the map keeps bleeding under it) and costs nothing but
	## the 26 dp the system was going to take anyway.
	var gesture := int(round(DccTheme.H_PHONE_GESTURE * host.phone_scale()))
	var target := Vector2i(int(screen.x), maxi(1, int(screen.y) - gesture))
	## One pixel short on purpose -- `_floor_dialog_bar()` below restores it, and
	## that restore is the only thing that makes `AcceptDialog` re-lay its button
	## bar. See that function for the measurement.
	dlg.popup(Rect2i(Vector2i.ZERO, target - Vector2i(0, 1)))
	## `AcceptDialog` parents its whole button bar as an **internal** child, so
	## `DccShell.phone_fit()` -- which walks `get_children()` -- has never once
	## reached it. Measured 29 dp on every window whose only way out is that
	## button (`gen_info_dialog.gd`, `performance_window.gd`,
	## `world_data_window.gd`, the credits sheet), which is two thirds of §13's
	## floor on the one control that closes the window. Flooring it, and then
	## seating the bar that holds it, is `_floor_dialog_bar()` below -- it runs
	## after the `popup()` and not before, because **`Window.popup()` clears
	## `custom_minimum_size`** when it re-lays that bar on show.
	_floor_dialog_bar(dlg, target)
	oversample(dlg)
	return true

## **Flooring the button bar and *seating* it are two different jobs, and only
## the first was done.** `AcceptDialog::_update_child_rects()` puts the bar at
## `size.y - buttons_minsize.height - margin` and takes that minimum from the
## layout pass in progress -- the one `popup()` just ran, with the stock 29 dp
## buttons still in place. Raising `custom_minimum_size` afterwards therefore
## grows each button **downwards from a position computed for the old height**,
## through the window's bottom edge, where the subwindow clips it.
##
## Measured on a OnePlus 6T (1080 x 2340, `phone_scale` 2.748) -- the real
## handset, not a `SubViewport` harness: the amber Close border on World data,
## Gen info, Performance and the credits sheet all ran y 2185-2268 against a
## window ending at 2269. **84 px of the 121 px the floor asks for: 5.31 mm of
## 7.65.** The glyph sat at 2245, the centre of the *full* 121 px box, which is
## what proved it was clipped rather than merely short. New World's
## Cancel/Create pair was the same defect at 78 px.
##
## `Window.child_controls_changed()` does **not** fix it, and that was tried on
## the device first: it defers to `Window::_update_window_size()`, which with
## `wrap_controls` off (which `phone_window()` sets, deliberately) finds the
## size unchanged and raises no notification, so `AcceptDialog` never re-lays.
## Measured after that attempt: still 82 px.
##
## **Three ways of asking `AcceptDialog` to re-lay were tried on the handset and
## all three measured identically**, which is what moved this from "find the
## right API" to "do the arithmetic here":
##
## - `Window.child_controls_changed()` -> 82 px. It defers to
##   `Window::_update_window_size()`, which with `wrap_controls` off (which
##   `phone_window()` sets, deliberately) finds the size unchanged and raises
##   nothing.
## - `size` assigned immediately after the floor -> 84 px.
##   `Control.custom_minimum_size` does not publish synchronously, it queues
##   `update_minimum_size()`, so a same-call relay is still told the stock size.
## - the same assignment via `set_deferred()` -> 84 px again. Queue order was the
##   wrong theory too.
##
## So the bar is seated **once**, by `popup()`, and nothing this function can
## reach makes it happen a second time. Fine: the geometry is fully known at this
## point and does not need the engine's help. `hbox.size.y` still holds the stock
## height here -- that staleness is the input, not the obstacle -- so the
## shortfall is `PHONE_TAP_MIN - hbox.size.y`, the bar moves up by it, and the
## content child above shrinks by the same amount so the two do not overlap.
##
## Everything is in the window's own content-scale units, which is why the
## 44 in `PHONE_TAP_MIN` can be compared with `hbox.size.y` directly.
## `ad.get_children()` skips internal children, so `bg_panel` and the button bar
## itself are not in that loop; the content child is the only thing it touches.
##
## Idempotent, and self-healing if a future engine version does relay: a second
## call finds `hbox.size.y` already at or above the floor and returns.
##
## Measured on a OnePlus 6T (1080 x 2340, `phone_scale` 2.748) -- the real
## handset, not a `SubViewport` harness. Before: the amber Close border on World
## data, Gen info, Performance and the credits sheet all ran y 2185-2268 against
## a window ending at 2269, **84 px of the 121 px the floor asks for, 5.31 mm of
## 7.65**, with the glyph at 2245 -- the centre of the *full* 121 px box, which
## is what proved it was clipped rather than merely short. New World's
## Cancel/Create pair was the same defect at 78 px.
static func _floor_dialog_bar(dlg: Window, target: Vector2i) -> void:
	dlg.size = target
	if not (dlg is AcceptDialog):
		return
	var ad := dlg as AcceptDialog
	## `get_cancel_button()` is `ConfirmationDialog`'s, not `AcceptDialog`'s --
	## asked for by name rather than assumed, so a plain `AcceptDialog` does not
	## take a "method not found" here.
	var bar: Array[Button] = [ad.get_ok_button()]
	if ad.has_method("get_cancel_button"):
		bar.append(ad.call("get_cancel_button"))
	## An **untyped** loop element writes to a temporary copy of the vector and
	## is lost -- `app.gd::_floor_prompt_buttons()`'s first trap, which applies
	## here verbatim.
	for b: Button in bar:
		if b != null and b.visible:
			b.custom_minimum_size = Vector2(0.0, DccTheme.PHONE_TAP_MIN)
	var ok: Button = ad.get_ok_button()
	if ok == null or ok.get_parent() == null:
		return
	var hbox := ok.get_parent() as Control
	if hbox == null:
		return
	var short := float(DccTheme.PHONE_TAP_MIN) - hbox.size.y
	if short <= 0.5:
		return
	## **Plus a foot, because the bar had none.** Seating the 44 dp button
	## exactly where the 29 dp one ended still measured as clipped on the
	## handset (2144-2268 against a window whose own bottom border is 2266-2268):
	## `AcceptDialog` gives the bar no bottom margin at all, so the button's
	## border and the window's border were the same two pixels. 12 dp is the
	## shell's own standard inset -- `category()`'s `DccTheme.inset(12, 0, 12, 0)`
	## -- rather than a number chosen to make this screenshot look right.
	short += 12.0
	hbox.position.y -= short
	hbox.size.y = float(DccTheme.PHONE_TAP_MIN)
	for child in ad.get_children():
		var c := child as Control
		if c != null and c != hbox and c.visible:
			c.size.y = maxf(1.0, c.size.y - short)

## **A content scale does not scale the font raster, and this engine does not
## work it out on its own** (`GUI_GAP_REGISTER.md` HD-01). Everything above maps
## a desktop-authored composition onto 393 dp and lets the compositor do the
## rest -- true of geometry, false of type. Godot 4.5 introduced dynamic font
## oversampling and 4.7.1 has it on by default, but `Viewport.get_oversampling()`
## inside a `CONTENT_SCALE_MODE_CANVAS_ITEMS` sub-Window whose
## `content_scale_factor` is 3.664 returns **1.0**: the automatic value does not
## account for a Window's own content scale, so a 12 dp label is rasterised at
## 12 texels and the canvas transform magnifies that bitmap.
##
## Measured on this exact build rather than inferred from a version number, two
## windows drawing the same physical glyph height:
##   factor 3.664 / font 12 -> max adjacent-pixel |dLum| 0.2667, 0 hard edges
##   factor 1.000 / font 44 -> max 0.9843, 722 hard edges
## 0.2667 is 1/3.75: a resampled bitmap cannot produce a step steeper than its
## own magnification allows, which is what makes this a measurement rather than
## an impression. Turning `Viewport.oversampling` **off** changed nothing (the
## same 0.2667 to four places), so the boolean is not the lever.
## `oversampling_override` is -- with it the same window measures 0.9804 and 518
## hard edges, the native control's own numbers.
##
## **It has to be set once the window is in the tree.** Assigned in a
## constructor the property reads back the value and `get_oversampling()`
## ignores it; that was the first cut of this fix and it measured exactly as if
## absent. Hence a call at the end of `phone_present()`, after `popup()`, rather
## than beside `content_scale_factor` above -- and re-applied on every present,
## since a rotation re-enters that path.
##
## Read off the window rather than recomputed where it can be:
## `content_scale_factor` is a float32 property, so 1440/393 stores as
## 3.66412210464478 and not the 3.66412213740458 that was assigned, and the two
## must not disagree by that last ulp. `scale` is for the callers that have no
## such property to read -- an embedded `PopupMenu` is drawn inside its parent
## window's canvas and so inherits the parent's content scale without ever
## carrying it, and its own `content_scale_factor` reads a flat 1.0.
##
## **A resize clears it, and the value it reverts to is 1.0.** Measured, in
## isolation: set on a content-scaled `Window` the override survives eleven
## frames, a `popup()` and a hide/show cycle unchanged, and then reads back 1.0
## the frame after `size` is assigned -- reassigning `content_scale_factor`
## afterwards does not bring it back. That is the same trap the `AcceptDialog`
## button bar above already carries ("`Window.popup()` clears the value, because
## the bar is re-laid on show"), and it is why the first cut of this measured
## exactly as if it were absent: `phone_present()` sets it, and one of the
## layout passes that follow a fill-the-screen popup silently drops it. So it is
## re-applied from `size_changed` as well as set here, guarded by a meta flag
## because this is a `static` function with no owning instance for Godot to
## auto-disconnect -- the connection belongs to the window and dies with it, and
## the handler re-checks `is_instance_valid` regardless.
const _OVERSAMPLE_META := "_dcc_oversample"

static func oversample(w: Window, scale: float = 0.0) -> void:
	if w == null:
		return
	if scale <= 0.0:
		scale = w.content_scale_factor
	if scale <= 1.0:
		return
	w.set_meta(_OVERSAMPLE_META, scale)
	w.oversampling_override = scale
	if not w.size_changed.is_connected(_reoversample):
		w.size_changed.connect(_reoversample.bind(w))

static func _reoversample(w: Window) -> void:
	if is_instance_valid(w) and w.has_meta(_OVERSAMPLE_META):
		w.oversampling_override = float(w.get_meta(_OVERSAMPLE_META))

## The header a borderless phone window draws in place of the title bar it
## gave up: the canvas's 56 dp app-bar row, in dp because the window that
## hosts it is content-scaled. Returns the title `Label` so a window whose
## title tracks its subject can keep writing to it.
static func phone_head(parent: Control, title: String, subtitle: String) -> Label:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"bottom": 1}))
	## 44 dp of keep-clear above the 56 dp bar. §13, verbatim: "Top 44 px is a
	## keep-clear safe area: status glyphs only… Nothing is centred there." A
	## full-bleed phone window starts at y = 0, so until 2026-08-25 every one of
	## these headers put its title **20 dp up inside the punch-hole lane** --
	## measured on the OnePlus 12 capture, `ASSET LIBRARY` with its cap height
	## at 20 dp and `WORLD DATA` at 28. `DccShell` reserves this for its own app
	## bar and no window ever did.
	wrap.custom_minimum_size.y = DccTheme.H_PHONE_TOP_SAFE + 56
	var m := MarginContainer.new()
	m.add_theme_constant_override("margin_left", 16)
	m.add_theme_constant_override("margin_right", 16)
	m.add_theme_constant_override("margin_top", DccTheme.H_PHONE_TOP_SAFE)
	wrap.add_child(m)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 2)
	col.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	m.add_child(col)
	var t := DccTheme.mono_label(title.to_upper(), "text_bright", 12, 3, true)
	col.add_child(t)
	if subtitle != "":
		var sub := DccTheme.mono_label(subtitle, "text_faint", 9, 1)
		## **Clipped, because a subtitle here is often a path.**
		## `phone_project_picker.gd` passes the projects root, and on Android
		## that is an absolute app-private path -- measured on the OnePlus 6T,
		## `worlds on this device · /data/data/org.cartalith.walkingskeleton/…`
		## ran straight off the right edge of the screen with no ellipsis,
		## because a `Label` in a `MarginContainer` grows past its parent
		## rather than truncating. One line, clipped, with the tail cut: a
		## header is a header, not a place to read a filesystem path.
		sub.clip_text = true
		sub.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		sub.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		col.add_child(sub)
	parent.add_child(wrap)
	parent.move_child(wrap, 0)
	return t

static func pad(parent: Control, l: int, t: int, r: int, b: int) -> MarginContainer:
	var m := MarginContainer.new()
	m.add_theme_constant_override("margin_left", l)
	m.add_theme_constant_override("margin_top", t)
	m.add_theme_constant_override("margin_right", r)
	m.add_theme_constant_override("margin_bottom", b)
	parent.add_child(m)
	return m

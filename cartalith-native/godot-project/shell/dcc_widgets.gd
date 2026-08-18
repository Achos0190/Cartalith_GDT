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
	btn.custom_minimum_size.y = 30
	btn.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	btn.add_theme_font_override("font", DccTheme.mono(1))
	btn.add_theme_color_override("font_color", DccTheme.c("text_bright"))
	btn.add_theme_stylebox_override("normal", DccTheme.inset(12, 0, 12, 0))
	btn.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	btn.add_theme_stylebox_override("pressed", DccTheme.inset(12, 0, 12, 0))
	wrap.add_child(btn)

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
	return body

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
## disclosure chevron"). `category()`'s single-string title can't host a state
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
	btn.custom_minimum_size.y = 30
	btn.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
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
	btn.custom_minimum_size.y = 22
	btn.text = "%s %s" % [mark, title.to_upper()]
	btn.add_theme_font_size_override("font_size", DccTheme.FS_HEADER)
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
	return group(parent, title, false, "＋")

# -- Rows ---------------------------------------------------------------------

const ROW_LABEL_W := 132
const ROW_VALUE_W := 56


## §11: "no fills on panels; regions are separated by hairlines only. Radius 0
## everywhere." A slider follows the same rule -- a 2 px rule, the travelled
## part in accent, and **no grabber**. Godot's default is a thick track with a
## round knob, which reads as a web form rather than a tool.
static func _style_slider(s: HSlider) -> void:
	var track := StyleBoxFlat.new()
	track.bg_color = DccTheme.c("line")
	track.content_margin_top = 1
	track.content_margin_bottom = 1
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

static func _row(parent: Control, label_text: String, tooltip: String) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = 24
	row.tooltip_text = tooltip
	var l := DccTheme.mono_label(label_text, "text_dim", DccTheme.FS_SMALL, 0)
	l.custom_minimum_size.x = ROW_LABEL_W
	l.clip_text = true
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
	s.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	s.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	s.custom_minimum_size.y = 14
	s.focus_mode = Control.FOCUS_NONE
	_style_slider(s)
	row.add_child(s)
	var readout := DccTheme.mono_label("", "text", DccTheme.FS_SMALL, 0)
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

static func toggle(parent: Control, label_text: String, value: bool,
		on_change: Callable, tooltip: String = "") -> CheckBox:
	var row := _row(parent, label_text, tooltip)
	var cb := CheckBox.new()
	cb.button_pressed = value
	cb.focus_mode = Control.FOCUS_NONE
	cb.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	cb.toggled.connect(func(v: bool): on_change.call(v))
	row.add_child(cb)
	row.add_child(DccTheme.spacer())
	return cb

static func choice(parent: Control, label_text: String, options: Array, selected: int,
		on_change: Callable, tooltip: String = "") -> OptionButton:
	var row := _row(parent, label_text, tooltip)
	var ob := OptionButton.new()
	ob.focus_mode = Control.FOCUS_NONE
	ob.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	ob.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	for o in options:
		ob.add_item(String(o))
	ob.selected = selected
	ob.item_selected.connect(func(i: int): on_change.call(i))
	row.add_child(ob)
	return ob

static func number(parent: Control, label_text: String, minimum: float, maximum: float,
		step: float, value: float, on_change: Callable, tooltip: String = "") -> SpinBox:
	var row := _row(parent, label_text, tooltip)
	var sb := SpinBox.new()
	sb.min_value = minimum
	sb.max_value = maximum
	sb.step = step
	sb.value = value
	sb.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	sb.value_changed.connect(func(v: float): on_change.call(v))
	row.add_child(sb)
	return sb

## The action a group commits with. §4 and §7 both put it *inside* the group it
## belongs to, never floating at the panel foot.
static func action(parent: Control, text: String, on_press: Callable,
		primary: bool = false) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_NONE
	b.custom_minimum_size.y = 26
	b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	b.add_theme_font_override("font", DccTheme.mono(1))
	if primary:
		b.add_theme_color_override("font_color", DccTheme.c("bg"))
		b.add_theme_stylebox_override("normal", DccTheme.flat(DccTheme.c("accent"), 2))
		b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("accent").lightened(0.1), 2))
		b.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("accent_dim"), 2))
	else:
		b.add_theme_color_override("font_color", DccTheme.c("text"))
		b.add_theme_stylebox_override("normal", DccTheme.flat(DccTheme.c("sunken"), 2))
		b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("raised"), 2))
	b.pressed.connect(on_press)
	parent.add_child(b)
	return b

## Prose that explains a rule rather than labelling a control. Kept narrow so a
## dock at its minimum width still wraps sensibly.
static func note(parent: Control, text: String) -> Label:
	var l := DccTheme.label(text, "text_ghost", DccTheme.FS_MICRO)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.custom_minimum_size.x = 240
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(l)
	return l

## The mark a stage carries when something upstream changed. Non-destructive by
## default: editing a stage marks everything downstream stale rather than
## silently invalidating it.
static func stale_mark(parent: Control) -> Label:
	var l := DccTheme.label("stale", "stale", DccTheme.FS_TINY)
	l.visible = false
	parent.add_child(l)
	return l

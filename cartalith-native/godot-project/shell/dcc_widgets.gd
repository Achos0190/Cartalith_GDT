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
	## `font-family:'Helvetica Neue';font-size:11px;color:#a9adb0` on every
	## parameter row in the canvas's left dock -- prose, not Plex, and one ink
	## step brighter than `text_dim`. Only the *value* on the right is Plex.
	## This row is the single most repeated thing in the shell, so drawing its
	## label in mono put a monospaced texture across every dock in the app that
	## the reference does not have anywhere.
	var l := DccTheme.label(label_text, "text_secondary", DccTheme.FS_SMALL)
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
	## `width:78px;height:2px` -- a fixed track, not an expanding one. The
	## canvas gives the *label* the slack and keeps every track in a dock the
	## same length, so the five steering dials read as one column of bars. An
	## expanding track measured 128 px here at a 372 px dock and grew with the
	## dock, which is why long parameter names ("Enable continental shelves")
	## were clipping while the bar beside them had room to spare.
	s.size_flags_horizontal = Control.SIZE_SHRINK_END
	s.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	s.custom_minimum_size = Vector2(TRACK_W, 14)
	s.focus_mode = Control.FOCUS_NONE
	_style_slider(s)
	row.add_child(DccTheme.spacer())
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
static func action(parent: Control, text: String, on_press: Callable,
		primary: bool = false) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_NONE
	b.custom_minimum_size.y = 26
	b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	b.add_theme_font_override("font", DccTheme.mono(1))
	var edge := "accent" if primary else "border"
	b.add_theme_color_override("font_color",
		DccTheme.c("accent") if primary else DccTheme.c("text"))
	b.add_theme_color_override("font_hover_color", DccTheme.c(
		"accent_hover" if primary else "text_bright"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	var rest := DccTheme.outline(edge)
	rest.content_margin_left = 10
	rest.content_margin_right = 10
	rest.content_margin_top = 4
	rest.content_margin_bottom = 4
	b.add_theme_stylebox_override("normal", rest)
	b.add_theme_stylebox_override("disabled", rest)
	var lit := DccTheme.outline(edge, "accent_wash" if primary else "line_soft")
	lit.content_margin_left = 10
	lit.content_margin_right = 10
	lit.content_margin_top = 4
	lit.content_margin_bottom = 4
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
static func modal_button(parent: Control, text: String, on_press: Callable,
		primary: bool = false) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_NONE
	b.custom_minimum_size = Vector2(0, 30)
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
	b.custom_minimum_size = Vector2(30, 30)
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

## §4.5.1 -- present in every domain, identical everywhere.
const GLOBAL_TOOL_ENTRIES: Array = [
	{"id": "inspect", "glyph": "tool_inspect", "label": "Inspect (V)"},
	{"id": "measure", "glyph": "tool_measure", "label": "Measure (M)"},
	{"id": "region", "glyph": "tool_region", "label": "Region select (R)"},
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
	var l := DccTheme.label(text, "text_ghost", DccTheme.FS_MICRO)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.custom_minimum_size.x = 190
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
static func segment(parent: Control, text: String, on_press: Callable) -> Button:
	var b := chip(parent, text, on_press, false, 8, 3)
	b.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	b.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	return b

## A lit segment has to survive being `disabled` -- Godot resolves the
## `disabled` stylebox and `font_disabled_color` ahead of the `normal` pair, so
## a lit-but-disabled segment (the Data manager draws several: the one real
## scheme among three impossible ones) would otherwise be painted exactly like
## the impossible ones.
static func set_segment_on(b: Button, on: bool) -> void:
	var token := "accent" if on else "border"
	var fg := DccTheme.c("accent") if on else DccTheme.c("text_dim")
	for sb_name in ["normal", "pressed", "disabled"]:
		b.add_theme_stylebox_override(sb_name, box(token, "", 8, 3))
	b.add_theme_color_override("font_color", fg)
	b.add_theme_color_override("font_disabled_color",
		fg if on else DccTheme.c("text_ghost"))

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
	dlg.popup(Rect2i(Vector2i.ZERO,
		Vector2i(int(screen.x), maxi(1, int(screen.y) - gesture))))
	## `AcceptDialog` parents its whole button bar as an **internal** child, so
	## `DccShell.phone_fit()` -- which walks `get_children()` -- has never once
	## reached it. Measured 29 dp on every window whose only way out is that
	## button (`gen_info_dialog.gd`, `performance_window.gd`,
	## `world_data_window.gd`, the credits sheet), which is two thirds of §13's
	## floor on the one control that closes the window.
	##
	## `app.gd::_floor_prompt_buttons()` found this first for the quit prompt
	## and records the two traps it costs, both of which apply here verbatim:
	## an **untyped** loop element writes to a temporary copy of the vector and
	## is lost, and **`Window.popup()` clears the value**, because the bar is
	## re-laid on show. So this runs after the `popup()` above, not before --
	## and again on every rotation, since the relay re-enters here.
	## `get_cancel_button()` is `ConfirmationDialog`'s, not `AcceptDialog`'s --
	## asked for by name rather than assumed, so a plain `AcceptDialog` does not
	## take a "method not found" here.
	if dlg is AcceptDialog:
		var ad := dlg as AcceptDialog
		var bar: Array[Button] = [ad.get_ok_button()]
		if ad.has_method("get_cancel_button"):
			bar.append(ad.call("get_cancel_button"))
		for b: Button in bar:
			if b != null and b.visible:
				b.custom_minimum_size = Vector2(0.0, DccTheme.PHONE_TAP_MIN)
	oversample(dlg)
	return true

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
		col.add_child(DccTheme.mono_label(subtitle, "text_faint", 9, 1))
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

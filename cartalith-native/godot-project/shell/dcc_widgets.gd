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
	## `false`, not the `true` this carried before: "normal"/"pressed" below
	## are `inset()` (a real `StyleBoxEmpty`, margin only, so `flat` costs it
	## nothing) but "hover" is a genuine `line_soft` wash, and a flat `Button`
	## draws no stylebox in any state -- so every L2 category header in every
	## dock had no hover feedback at all. `layers_popover.gd`'s own finding,
	## reproduced independently on an isolated probe before this changed.
	btn.flat = false
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
	## See `category()`'s own comment: not flat, for the same reason -- its
	## "hover" override is a real fill this shared shape needs drawn.
	btn.flat = false
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
##
## `elide: true` on the `header()` call, lane GRID 2026-09-13: safe here and
## only here among that function's callers, because `head` below is the SOLE
## child of `pad`, a `MarginContainer` that sizes an only child to its own
## full inner rect regardless of the child's minimum -- unlike
## `left_dock_title`/`right_dock_title` (`dcc_shell.gd`), which sit beside a
## `SIZE_EXPAND_FILL` spacer and collapsed to 1 px when a first cut of this
## made `header()` clip unconditionally. See `DccTheme.header()`'s own doc for
## why the choice moved to the caller.
static func section(parent: Control, title: String) -> VBoxContainer:
	var head := DccTheme.header(title, "§", true)
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
	## DS-03's reflow, extended to the L4 header. A `Button`'s minimum width is
	## the width of its whole label, and this header had neither autowrap nor a
	## parent guard while `action()` -- the sibling factory ten screens down --
	## carried both. That asymmetry was not a decision; nothing above this line
	## ever said anything about width.
	##
	## **Measured before it was changed** (`_grphdr_probe.gd`, `_grpfloor_probe.gd`;
	## seeds 483920 / 77021 / 4242, which agreed on every figure below; laptop
	## density, left dock 330 / right dock 280). 79 `group()` call sites plus the
	## four `advanced()` ones. **141 surface states per seed** -- ten rail nodes,
	## every L2 category inside each, and every arm of
	## `RightDock._tool_section()` in the domain that arm requires. In 69 of
	## those 141 a dock could not be dragged down to its documented floor, and a
	## group header was the node holding it open in **6 of the 69** -- two
	## distinct headers, each appearing in several rail-node contexts:
	##
	##   CIVIL ▸ Military   `› WHO THE BANDS ARE MEASURED AGAINST` (265 px)
	##                      floor 300, drag stopped at 306
	##   RIGHT ▸ Journey    `› VESSEL REFERENCE · SPEED BY WATER` (258 px)
	##                      floor 260, drag stopped at 273
	##
	## Two headers is a thin case for re-basing a widget with 83 call sites, so
	## the cost was measured rather than argued (`_grpwrap_shot.gd`, **windowed**
	## 1600x1000, light palette, four surfaces): at the shipped dock widths the
	## change moves **0 pixels of 1 600 000**, against a positive control -- one
	## group toggled -- of 37 428. Re-run at the desktop pair (372/304) and at
	## tablet (`--force-touch`, 400/400, where the header takes the 11 px
	## `fs_dock_header` instead of `FS_HEADER`): 0 there too.
	##
	## It is free because a wrapped `Button` in a column still DRAWS at the full
	## dock width -- autowrap only lowers its minimum, and it lowers it to **0**,
	## not to the widest word. `_grpguard_probe.gd` asserts both halves of that
	## on the live header, because a ~0 minimum is one step from `MISTAKES.md`'s
	## `clip_text` trap: measured `min.x = 0.0` while `size.x = 265` in its own
	## column, text intact and `clip_text` false.
	##
	## Afterwards the full 141-state sweep re-runs with **0 regressions**: 135
	## states unchanged, 6 improved, and no group header binds any dock above
	## its floor. The remaining 65 states that still cannot reach their floor
	## are held open by `Label`s (41), `OptionButton`s (10), `RichTextLabel`s
	## (8), `HSlider` rows (5) and one `TextureRect` -- none of which this
	## change touches. CIVIL ▸ Landmarks still stops at 328, CARTO ▸ Layers &
	## style at 333. **The dock floors are aspirational for reasons this change
	## does not address**; it removes group headers as one of the causes.
	##
	## The guard is `action()`'s, and its question is **"does a sibling compete
	## for my width"**, not "which class is my parent" -- a `GridContainer`
	## shares width across columns exactly as an `HBoxContainer` shares it
	## across children, and it was the arm a verifier had to add to `action()`.
	## No group header live in the shell sits in a width-distributing parent
	## today: 0 of 75 / 92 / 90 headers (one figure per seed) across 16 surface
	## states each, asserted over the whole `app` subtree by
	## `_grpguard_probe.gd` rather than inferred from the call sites. The guard
	## is here anyway because `action()`'s own note records `set_tool_options()`
	## handing that factory an `HBoxContainer` and costing 225 px of map.
	var grp_shares_width := (parent is BoxContainer and not (parent as BoxContainer).vertical) \
		or parent is HFlowContainer \
		or parent is GridContainer
	if not grp_shares_width:
		btn.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	## An L4 group header is a dock row too (§57's "row_min_h" -- "Dock list
	## rows and menu items"); its own type is `FS_HEADER` (9), which is exactly
	## `ROLE`'s `fs_dock_header` desktop figure (`[9, 11]`), not `fs_readout`.
	##
	## `custom_minimum_size.y` stays a FLOOR, not a fixed height: with autowrap
	## above it, a header that wraps to two lines in a narrowed dock reports a
	## content minimum taller than this and grows. That is the intent -- the
	## owner's DS-03 ruling is "keep everything, reflow only", so a header that
	## no longer fits gets a second line rather than an ellipsis that would
	## delete half of what it names.
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

# ---------------------------------------------------------------------------
# **The inputs had no styling of their own until 2026-09-07**, which is the
# input half of the owner's "all buttons and inputs are visually not the same".
# `choice()`, `toggle()` and `number()` each called `OptionButton.new()` /
# `CheckBox.new()` / `SpinBox.new()`, set a font size, and stopped -- not one
# `add_theme_stylebox_override` between the three -- so fill, border, corner
# radius and padding all came from `res://theme/dark_theme.tres`, whose own
# header dates it **2026-08-17** and says it was "authored against the mockup".
# That is the same mechanism the owner named for the buttons, one layer down.
#
# Measured against `design/mcp-2026-09-07/`, that resource's `SB_FieldNormal`
# is wrong in four ways at once:
#
# | | `dark_theme.tres` | canvas |
# |---|---|---|
# | fill | `#1a1b1d` | `var(--ins)`: `#191c1e` dark, `#eceae4` light |
# | border | 1 px `rgba(255,255,255,.14)` | **none** |
# | radius | 4 | `8px` pointer (`ENV:522`), `--rCtl:12px` tablet |
# | padding | `10px 6px` | `3px 9px` (`ENV:522`) |
#
# The fill is the one that is not merely off by three values. `#1a1b1d` is
# **not a `DccTheme` token** -- `sunken` is `#191c1e` -- so
# `DccShell._recolor_project_theme()`'s reverse lookup returns null for it and
# leaves it alone on a palette flip. Every field in the shell therefore stayed
# a near-black slab on the **light** palette, which is the preference this
# machine runs.
#
# Giving the widgets their own boxes retires that resource for them without
# touching `project.godot`, which this pass may not edit. Ground comes from
# `DccTheme.field_box()`; see its header for why it is a sibling of
# `button_box()` rather than an extension of it, and `DccTheme.outline()` is
# not touched here for the same reason the button pass did not touch it.
# ---------------------------------------------------------------------------

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
	## **Paint.** `dark_theme.tres` points all four `CheckBox` styleboxes at
	## `SB_FieldDisabled` -- one bordered, rounded, 10x6-padded slab, identical
	## in every state -- so a toggle carried a *field's* chrome it is not a
	## field, and had no hover feedback at all because normal and hover were
	## the same resource. The canvas draws no box behind a toggle row.
	##
	## `StyleBoxEmpty` rather than a transparent `StyleBoxFlat`: it holds no
	## colour, so it cannot go stale on a palette flip and needs no entry in
	## `DccShell._THEME_STYLEBOX_OVERRIDES`, which this pass may not edit.
	for state in ["normal", "hover", "pressed", "hover_pressed", "disabled", "focus"]:
		cb.add_theme_stylebox_override(state, DccTheme.empty())
	_palette_watch(cb, func() -> void: _paint_switch(cb))
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
	## Lane GRID 2026-09-13: the spacer is inert once the label above is
	## already `SIZE_EXPAND_FILL` -- both would share the row's leftover
	## width, and the checkbox's final left edge is `label_final + sep +
	## spacer_final + sep`, a sum invariant to how that split lands (the
	## spacer draws nothing and holds no minimum of its own), so dropping it
	## moves not one visible pixel on any geometry with slack to give. What
	## it costs with NO slack is one more `separation` (8) baked into the
	## row's own MINIMUM-size sum for nothing rendered -- measured as a
	## portrait-dock driver: a toggle row (label 132 + 2 seps + the
	## intrinsically-sized `CheckBox`) held at 188 against a compacted
	## slider row's 172 (`_wrap_slider_cell`, `world_workspace.gd`), which
	## was this dock's own remaining reason Terrain/Climate missed 232 px
	## after that reflow and `DccTheme.header()`'s own elide fix -- measured
	## by `_worldportraitgrid_probe.gd`'s driver walk, not guessed. Tablet
	## portrait only, matching every other change of this shape here:
	## desktop, laptop, landscape tablet and phone toggles keep the spacer
	## and are BYTE-IDENTICAL to HEAD, same child count, same nodes.
	if not DccTheme.is_tablet_portrait():
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
	## **Paint.** A closed dropdown is the canvas's `--ins` chip exactly --
	## `ENV:522` is `min-height:var(--ctl);border-radius:8px;
	## background:var(--ins);padding:3px 9px` -- so its ground is
	## `DccTheme.field_box()` and its padding is `chip_pad_x`/`chip_pad_y`,
	## whose pointer half (9/3) is that declaration verbatim and whose tablet
	## half (16/9) is already the canvas-derived touch figure.
	##
	## **Ink is one step brighter than a button's, and that is the design.** A
	## dropdown shows a *value* (`color:var(--ink)` on the chip's own label
	## span) where a secondary button shows a *label* (`color:var(--sec)`).
	## Hover moves to `var(--acc)`, the `style-hover` the canvas puts on every
	## `--ins` chip, and disabled drops to `var(--dis)` over a held ground --
	## the undo/redo pair's own measured behaviour.
	##
	## **The caret is the stock `arrow` under `modulate_arrow`, not a drawn
	## texture, and the trade is stated.** That constant makes `OptionButton`
	## paint its arrow in the current draw mode's *font colour*, so the caret
	## follows `font_color`/`font_hover_color` -- both names
	## `DccShell._recolor_subtree()` walks -- and survives a palette flip with
	## no repaint hook at all. What it costs: the canvas draws the caret one
	## step dimmer than the value (`color:var(--faint)`, `ENV:525`) and this
	## draws them at the same ink. An `ImageTexture` caret would match that and
	## then go stale, exactly as `_style_popup_marks()`'s marks do.
	var chip_x := DccTheme.role_px("chip_pad_x")
	var chip_y := DccTheme.role_px("chip_pad_y")
	for state in ["normal", "hover", "pressed", "disabled", "focus"]:
		ob.add_theme_stylebox_override(state,
			DccTheme.field_box(state, chip_x, chip_y))
	ob.add_theme_color_override("font_color", DccTheme.c("text_bright"))
	ob.add_theme_color_override("font_hover_color", DccTheme.c("accent"))
	ob.add_theme_color_override("font_pressed_color", DccTheme.c("accent"))
	ob.add_theme_color_override("font_focus_color", DccTheme.c("text_bright"))
	ob.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	ob.add_theme_constant_override("modulate_arrow", 1)
	## **`fit_to_longest_item` is not touched.** `DccShell.dock_fit()` and
	## `phone_fit()` clear it on expanding dropdowns inside a dock and leave it
	## set everywhere else; it is load-bearing in three places and none of them
	## is paint.
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
	## **Paint.** A `SpinBox` is the worst-hit of the three, because it stacks
	## *two* of the stale theme's boxes: `SB_FieldNormal` on its `LineEdit` and
	## another behind each arrow, so a number row drew a bordered slab inside a
	## bordered slab. It was also the only control in `_row()` with **no font
	## size set at all**, so its field alone took the resource's 13 px while
	## every sibling in the same row is 11.
	##
	## `get_line_edit()` is the field, and touching only its theme is
	## deliberate: `DccWidgets.touch_focus_field()` attaches a script and
	## rewires `focus_mode` on that same node from `DccShell`, and the two are
	## disjoint -- no property either one sets is read by the other.
	##
	## The arrows get `StyleBoxEmpty`, which is both the canvas's composition
	## (`ENV:348` draws its stepper glyph on the `--ins` ground, not on a box
	## of its own) and the palette-safe choice, since an empty box holds no
	## colour to go stale. Their ink rides on `*_icon_modulate` -- `var(--sec)`
	## at rest, `var(--acc)` engaged, `var(--dis)` disabled, all off that same
	## canvas row -- and those eight names are *not* in
	## `DccShell._THEME_COLOR_OVERRIDES`, which is what `_palette_watch()` is
	## for.
	##
	## **No `SpinBox` constant is touched.** `field_and_buttons_separation`,
	## `buttons_width` and `set_min_buttons_width_from_icons` are the three
	## that would move the field's rect, and this pass is paint.
	style_spin(sb)
	row.add_child(sb)
	return sb

## Paint a `SpinBox` the way the canvas draws one. Extracted from `number()`,
## which was its only caller and therefore its only beneficiary.
##
## **The extraction is the fix.** `journey_planner_view.gd` builds four bare
## `SpinBox.new()`s inside custom rows -- a checkbox beside a field, which
## `number()`'s own row shape cannot express -- so they never reached this
## paint and took the stale project theme instead: measured at 81x29 on a
## `#f4f2ee` ground beside the 170x24 `#eceae4` chips this function produces.
##
## That is the third instance of one mistake in this shell: `_picker_button()`,
## the private window constructions, and now these. **"Every X" keeps meaning
## "every X that goes through the factory we knew about"**, and the remedy is
## the same each time -- make the paint callable without the layout.
static func style_spin(sb: SpinBox) -> void:
	var spin_x := DccTheme.role_px("chip_pad_x")
	var spin_y := DccTheme.role_px("chip_pad_y")
	var le := sb.get_line_edit()
	for state in ["normal", "focus", "read_only"]:
		le.add_theme_stylebox_override(state,
			DccTheme.field_box(state, spin_x, spin_y))
	le.add_theme_font_override("font", DccTheme.mono(0))
	le.add_theme_font_size_override("font_size",
		DccTheme.role_px("fs_readout") if DccTheme.is_tablet() else DccTheme.FS_TINY)
	le.add_theme_color_override("font_color", DccTheme.c("text_bright"))
	le.add_theme_color_override("font_uneditable_color", DccTheme.c("text_ghost"))
	le.add_theme_color_override("font_placeholder_color", DccTheme.c("text_faint"))
	le.add_theme_color_override("caret_color", DccTheme.c("accent"))
	## See `well()` for why `font_selected_color` is set by hand: the default
	## theme's white glyph on the light palette's `#d3bb99` selection band
	## measured 1.26:1.
	_palette_watch(le, func() -> void:
		le.add_theme_color_override("font_selected_color",
			DccTheme.c("text_bright")))
	## **Left, and `ENV:351` is why it is not right.** This line used to read
	## `le.alignment = HORIZONTAL_ALIGNMENT_RIGHT`, justified by "`text-align:
	## right` on the canvas's numeric readout (`ENV:351`)". That declaration is
	## real and the borrowing was not: `ENV:351` is
	##
	##   width:52px;flex:none;text-align:right;font:var(--m1) 'IBM Plex Mono'
	##
	## -- a **52 px fixed-width readout span with no ground at all**, sitting
	## between a slider and the row edge. Right-aligning a 52 px span moves its
	## digits by a few pixels. This field is a `SIZE_EXPAND_FILL` input:
	## measured in the New World dialog it is **388 px wide**, so the same
	## declaration moved a 36 px number 343 px away from its own label and left
	## the box it sits in empty. That is the owner's "minuscule glyph in the
	## bottom-right corner of an oversized empty box", and the frame that was
	## reported lost is the ground of a chip with nothing in it but corner.
	##
	## What a `SpinBox` *is* on the canvas is the `--ins` chip its dropdown
	## sibling is (`ENV:522`), whose value sits at the left and whose caret
	## sits at the right; the canvas's two real `<input>`s (`ENV:222`,
	## `ENV:498`) set no `text-align` at all. `DccWidgets.well()` -- every
	## other text field in this shell -- likewise leaves it default. So the
	## number now starts where the dropdown's value starts, one row above it,
	## and `_numglass_probe` asserts exactly that relationship rather than a
	## pixel column that any width change would move.
	le.alignment = HORIZONTAL_ALIGNMENT_LEFT
	for side in ["up", "down"]:
		for slot in ["_background", "_background_hovered", "_background_pressed",
				"_background_disabled"]:
			sb.add_theme_stylebox_override(side + slot, DccTheme.empty())
	_palette_watch(sb, func() -> void: _paint_spin_arrows(sb))

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
## figures (14/4) are this factory's own literals, independent of `ROLE`'s
## tablet pair. Both are canvas figures: the `--btnH` census (14 nodes,
## `LARGE_ITEM_RULINGS.md`'s "Ruling G, made and WITHDRAWN the same hour")
## found y=4 on 10 of 14 and x=14 the only plurality, 5 of 14. y was already
## this value; x moved from the prior 10 to match, 2026-09-08 -- guarded by
## `_ds03fit_probe`/`_ds03shot_probe` since it is a reflow change (see the
## DS-03 paragraph below).
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
	## **Filled, rounded, four distinct states** -- this block was the whole of
	## the owner's 2026-09-07 "all buttons and inputs are visually not the same".
	##
	## What stood here drew `DccTheme.outline(edge)`: a hairline rectangle with
	## `Color(0, 0, 0, 0)` for a fill and, because `outline()` never calls
	## `set_corner_radius_*`, square corners. The canvas draws neither. It fills
	## (`var(--acc)` primary, `var(--ins)` secondary), it rounds (8 px pointer,
	## `--rCtl:12px` tablet), and it puts **no border on either variant** -- 0 of
	## its `--ins` chips carry a `border:` declaration. See
	## `DccTheme.button_box()` for every measurement and for why the fix could
	## not go in `outline()`, which 30 sites share and only four of them buttons.
	##
	## The colour half lives here rather than in the factory because Godot takes
	## font colours as per-state overrides on the `Button`; the two halves are
	## one design and must be read together.
	##
	## Ink follows the fill, which is the actual repair: primary text was
	## `c("accent")` -- amber on nothing -- and is now `accent_ink` on the amber
	## ground, the pairing the canvas writes as `color:var(--accInk)` and which
	## measures **8.60:1** dark / 4.30:1 light. Secondary is `--sec` =
	## `text_secondary` (7.58:1 / 8.87:1). The hover ink is measured too, not
	## invented: the canvas's `style-hover` on an `--ins` chip is
	## `color:var(--acc)`, so a secondary hovers to accent and a primary holds
	## `accent_ink` while its *fill* lifts.
	##
	## **`disabled` no longer shares a box with `normal`.** That sharing is why a
	## disabled action read as an enabled one, which is a correctness defect
	## rather than a cosmetic one: nothing on screen distinguished a control you
	## could press from one you could not.
	b.add_theme_color_override("font_color",
		DccTheme.c("accent_ink") if primary else DccTheme.c("text_secondary"))
	b.add_theme_color_override("font_hover_color",
		DccTheme.c("accent_ink") if primary else DccTheme.c("accent"))
	b.add_theme_color_override("font_pressed_color",
		DccTheme.c("accent_ink") if primary else DccTheme.c("accent"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	var pad_x := DccTheme.role_px("btn_pad_x") if act_tablet else 14
	var pad_y := DccTheme.role_px("btn_pad_y") if act_tablet else 4
	for state in ["normal", "hover", "pressed", "disabled"]:
		b.add_theme_stylebox_override(state,
			DccTheme.button_box(primary, state, pad_x, pad_y))
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
	## Same repair as `action()`, and for the same reason: the paragraph above
	## already says these two are "the same chip at two sizes", so when `action()`
	## became a filled rounded button this had to move with it or the modals would
	## have become the only square hairline buttons left in the shell. It keeps
	## its own 18/8 padding -- that is the "two sizes" half of the claim and it is
	## unchanged -- and takes the shared fill, radius and four-state model.
	b.add_theme_color_override("font_color",
		DccTheme.c("accent_ink") if primary else DccTheme.c("text_secondary"))
	b.add_theme_color_override("font_hover_color",
		DccTheme.c("accent_ink") if primary else DccTheme.c("accent"))
	b.add_theme_color_override("font_pressed_color",
		DccTheme.c("accent_ink") if primary else DccTheme.c("accent"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	for state in ["normal", "hover", "pressed", "disabled"]:
		b.add_theme_stylebox_override(state,
			DccTheme.button_box(primary, state, 18, 8))
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
## first the four global tools, then that domain's own"). `entries` is
## `[{glyph, label, id}, ...]`; arming calls `app.arm_tool(id)`. An entry may
## carry `modes` -- the domain modes it is shown in (WORLD's Biome paint is
## `["b"]`, Ruling L); only the top-bar strip reads it, the phone's dock copy
## is gated by `world_workspace.gd::_refresh_paint_tool_row()` instead.
##
## **Off the phone this draws nothing in the dock.** Owner, 2026-09-23: *"the
## tools should be in a horizontal toolbar that sits on top"* -- an owner
## decision, which outranks `04-left-dock.md` §2.4's dock placement
## (`ENV:316-327`). The entries are handed to `DccApp.set_domain_tools()` and
## drawn by `tool_strip()` at the left of the top tool bar, for whichever domain
## is active. The phone keeps the dock block: its own spec (`06-phone.md` §6.3)
## has no top bar, and that request was about the desktop shell.
static func tools_block(parent: Control, app, group: ButtonGroup,
		domain_entries: Array = []) -> void:
	if not DccTheme.is_phone() and app != null and app.has_method("set_domain_tools"):
		app.set_domain_tools(String(parent.get("domain_id")), domain_entries)
		return
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
		_tool_entry(row, e, app, group)
	return row

## Where the tool palette is, for prose that sends a reader to it: the top
## palette bar off the phone, the dock's own TOOLS block on it (`tools_block()`).
static func tools_home() -> String:
	if DccTheme.is_phone():
		return "the TOOLS block at the top of this dock"
	return "the tool bar at the top of the window"

## The top palette bar's copy of the TOOLS block (`DccApp._install_tool_
## palette_bar()`): the four global cells, the canvas's divider (`ENV:323`,
## `width:1px;height:var(--tool);background:var(--div);margin:0 3px`), then the
## active domain's own tools, laid out as one `HBoxContainer` rather than
## `_tools_row()`'s `HFlowContainer` -- a flow container inside a bar's
## `HBoxContainer` reports its widest child as its minimum and wraps to a
## second line. `ENV:319`'s `gap:5px` between cells. Each button is lit from
## `app.armed_tool` as it is built, because the palette is rebuilt on every
## domain/mode change and a new button knows nothing of the last one's pressed
## state; `DccApp._sync_tool_strip()` keeps it lit between rebuilds.
static func tool_strip(parent: Control, app, group: ButtonGroup, domain_entries: Array) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 5)
	for e in GLOBAL_TOOL_ENTRIES:
		_tool_entry(row, e, app, group)
	if not domain_entries.is_empty():
		var div := ColorRect.new()
		div.color = DccTheme.c("line_soft")
		div.custom_minimum_size = Vector2(1, 30)
		div.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		var m := MarginContainer.new()
		m.add_theme_constant_override("margin_left", 3)
		m.add_theme_constant_override("margin_right", 3)
		m.add_child(div)
		row.add_child(m)
		for e in domain_entries:
			_tool_entry(row, e, app, group)
	for b in row.find_children("*", "Button", true, false):
		if (b as Button).toggle_mode:
			(b as Button).set_pressed_no_signal(String(b.get_meta(TOOL_ID_META, "")) == String(app.armed_tool))
	parent.add_child(row)
	return row

## The tool id a palette button arms -- read back by `tool_strip()` and
## `DccApp._sync_tool_strip()` to light the armed one.
const TOOL_ID_META := "dcc_tool_id"

static func _tool_entry(row: Control, e: Dictionary, app, group: ButtonGroup) -> void:
	if e.has("legend"):
		_tool_legend(row, String(e["glyph"]), String(e["label"]), String(e["legend"]))
		return
	var b := tool_button(row, e["glyph"], e["label"], group, func(): app.arm_tool(e["id"]))
	b.set_meta(TOOL_ID_META, String(e["id"]))
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
	## active domain's tools are in the top bar (`DccApp._tool_strip()`; on a
	## phone, only the active domain's dock is visible), so `W`
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
	##
	## **The wash is `accent_wash_2` (.16), not `accent_wash` (.09), since
	## 2026-09-06.** The `.10` quoted above is real and is the *older* Paint
	## Toolbar canvas; the newer `design/dcc-environment-2026-08-31/Cartalith
	## DCC Environment.dc.html` separates the two weights and spends them on two
	## different jobs, which is the whole reason `accent_wash_2` exists:
	##
	## - `var(--wash2)` -- **36 uses, every one a segment or toggle on-state**:
	##   `ldSwABg`/`ldSwBBg`, `measSegBg`/`measPathBg`, `terrAddBg`/`terrSubBg`,
	##   `bpEraseBg`/`bpLandBg`, `layersBtnBg`, `bakeBg`, `waySnapBg`, and the
	##   `X===id?'var(--wash2)'` chip rows (tool, ramp, shape, op, fall,
	##   anchor, iconFam, sizeMode, interp, cls, kind, tlSpeed, inspFilter).
	## - `var(--wash)` -- **13 uses, every one a hover or a list-row
	##   selection**: the two `style-hover="background:var(--wash)"` menu rows,
	##   `ca.sel===l.id`, `cv.sel===i`, `i===sc.sel`, `sel===i`, `bp.value===v`,
	##   `s.domain===id` (the domain cell, "merely current"), `finBg`.
	##
	## Counted with `grep -o "[A-Za-z_]*:[^,;{}]\{0,90\}var(--wash2)"` and its
	## `--wash` twin over that file. So this was never a choice between two
	## alphas for one token: the shell had both tokens and was spending the
	## row-selection one on the segment class. `accent_wash` keeps .09 and keeps
	## its own consumers -- menu highlight, `active_row()`, the selected-row
	## fills in the asset library and data manager, the chart fills.
	##
	## The owner's ruling names the token outright (`LARGE_ITEM_RULINGS.md` §6
	## and §7, hedge retired 2026-09-06): *"Resolved to the WASHED treatment --
	## `accent_wash_2` fill, `accent` ink, border"*, with the 0.16 move filed as
	## outstanding work precisely because it re-bases every call site here.
	##
	## **Measured before it was moved**, because the row that scheduled it asked
	## for that rather than for an opinion -- `_washstep_probe.gd`, windowed,
	## both palettes, real chips on their real grounds. The .09 -> .16 step is
	## **14-16 / 255** on the worst channel, against **19-26 / 255** for the
	## whole ground-to-lit signal this design already ships as legible. It is
	## roughly three quarters of the entire on/off cue, not a rounding step.
	var wash := "accent_wash_2" if on else ""
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

# -- The touch-gesture arbiter every scrolling slider needs ---------------------

## **A drag that starts on a slider belongs to the axis it is actually moving
## along, and until this class existed the slider took it either way.**
##
## Found on glass on the first swipe of a verification pass: Ocean depth went
## `0.60` -> `0.14` in one vertical gesture, stage 03 was marked stale, and
## nothing on screen said so.
##
## Two mechanisms combine, and the first is the one that surprises:
##
## 1. **Godot's `Slider` writes the value on touch-DOWN, not on drag.**
##    `Slider::gui_input` calls `set_as_ratio()` from the press position and
##    only then arms its grab, so the jump had already happened before there
##    was any motion to classify. Withholding the press is therefore
##    load-bearing here, not a refinement of the drag handling -- classifying
##    the drag alone would have left the touch-down jump exactly as it was.
## 2. A slider inside a scroller is picked `MOUSE_FILTER_STOP`, which is
##    correct for the horizontal drag and wrong for the vertical one -- and the
##    44 dp touch floor widened the band that consumes it from 32 dp to 44 dp.
##    A floor is a hit area, and a bigger hit area catches more than you meant,
##    so raising it obliged this. **The answer is not to shrink the row back
##    under the floor**: that trades a silent data change for a control a
##    finger cannot hit, and both are defects.
##
## The rule is Android's own: hold the gesture until it has travelled `slop`,
## then give it to whichever axis it travelled furthest along. Vertical scrolls
## and **never touches `value`**; horizontal writes exactly as before; a tap
## that never resolves keeps Godot's own jump-to-the-tap, applied at release
## once it is known to be a tap rather than before it is known to be anything.
##
## **This class lived inside `world_workspace.gd` until 2026-09-07 and covered
## two construction sites.** It was moved here because a live census of the
## phone tree at 1080x2340 (`_rangeswipe_probe.gd --census-only`) counted **247
## `Range` nodes, of which 245 are writable and sit inside a live vertical
## scroller -- and exactly 3 of those 245 arbitrated the gesture**, the `_pg_*`
## sliders. The other **242** are `DccWidgets.slider()` rows in the left-dock
## sheet (214), `DccWidgets.number()` spin boxes in the journey planner's form
## (12), and 16 more across four phone-presented windows. A defect class this
## broad cannot be closed at a construction site, so `touch_slider()` below is
## the attachment point and `DccShell.phone_fit()` is the walk that calls it.
##
## `_gui_input` is the seam that makes withholding possible.
## `Control::_call_gui_input` runs the script's `_gui_input` **before** the C++
## `Slider::gui_input`, and `accept_event()` aborts the rest of that chain --
## so this subclass can decide whether the slider it is attached to ever sees
## the event. Every pointer event it recognises is accepted, and `_apply_x()`
## below is what stands in for the suppressed built-in.
##
## The scroll is driven by writing the ancestor `ScrollContainer.scroll_vertical`
## rather than by letting the event propagate up, and that follows from the
## same withholding: `ScrollContainer`'s touch drag arms on the
## `InputEventScreenTouch` press, which by classification time has been
## swallowed, so forwarding only the later drags would scroll nothing. The cost
## is stated rather than hidden -- a fling that **begins on a slider** does not
## carry inertia. Every other pixel of the sheet still does.
##
## `_family` exists because `project.godot` leaves
## `input_devices/pointing/emulate_mouse_from_touch` at its default `true`
## (checked 2026-09-07; the file's own comment says so), so one finger delivers
## `InputEventScreenTouch`/`ScreenDrag` **and** an emulated
## `InputEventMouseButton`/`MouseMotion`. Latching to the family that opened
## the gesture is what stops every delta being counted twice.
##
## **Three gates decide whether this class does anything at all**, and each one
## exists because broadening the population from 3 sliders to 242 made it a
## question the two-site version never had to answer:
##
## * `editable == false` -> stock behaviour. Four surfaces disable a slider
##   (`cartography_workspace.gd:2632`, `civilization_workspace.gd:4589`,
##   `world_workspace.gd:1471` and `:1555`, plus `dcc_shell.gd`'s simulate
##   strip), and the original class checked nothing -- so attaching it broadly
##   without this would have made a *disabled* slider writable by a drag.
## * no vertical-scrolling ancestor -> stock behaviour. There is nothing to
##   arbitrate against, so a vertical drag should still mean "move this
##   slider". The phone's simulate-strip slider and one asset-library slider
##   are in exactly that position (census: `scroller=none`), and they keep
##   Godot's own handling untouched.
## * a non-left mouse button, or a wheel -> stock behaviour, as before.
class PgSlider extends HSlider:
	## Android's own `ViewConfiguration.getScaledTouchSlop()` is 8 dp, in the
	## pixels the surface lays out in.
	##
	## **Deliberately carries no default, and `PgField.slop` lost the identical
	## one in the same change.** Every way in supplies it: `touch_slider()`
	## below takes it as a required argument, and the two `PgSlider.new()` sites
	## -- `world_workspace.gd::_pg_range_field()` and `::_pg_sculpt_slider()` --
	## each assign `_pg_px(8)` on the line after the constructor. So a declared
	## `:= 8.0` was never reached by anything, and read as coverage it did not
	## give: mutated to `400.0` AND to `0.0` it left `_rangeswipe_probe.gd`
	## green both ways, and `PgField`'s left `_gestclass_probe.gd` green both
	## ways (measured 2026-09-07, four runs).
	##
	## **What replaces it is an assertion, not a quieter fallback**, because
	## `8.0` was the wrong kind of safety net -- it would have absorbed a
	## construction site that forgot, and reported nothing.
	## `_gestclass_probe.gd::_slop_walk()` walks the live phone tree and
	## requires every attached `PgSlider` and `PgField` to carry `slop >= 1.0`,
	## so a site that forgets now fails a probe. The cost of the removal is
	## stated rather than hidden: an unset `slop` is `0.0`, which is stock
	## first-pixel classification, which is the defect this class exists to
	## close -- that is exactly what `_slop_walk()` is looking for.
	##
	## **The two arrangements were measured against each other**, because
	## "the default hides a forgetful site" is a claim and not an argument.
	## Deleting `touch_slider()`'s own `s.set("slop", ...)` line -- one
	## construction path that forgets -- fails the walk with 218 of 239
	## unconfigured. Re-running that IDENTICAL mutation with `:= 8.0` put back
	## passes, `fail=0`, with the walk reporting `8.00=235`: every phone slider
	## silently arbitrating at 8 unscaled px instead of the 21 it was fitted
	## for, and nothing on screen or in any probe saying so.
	##
	## **Pinned from below as well as above.** `_rangeswipe_probe.gd`'s
	## `_jitter()` leg swipes vertically with the sideways wobble a real thumb
	## makes in its first three samples; at `slop = 0` that wobble is the whole
	## gesture the classifier sees, so the first pixel picks the horizontal axis
	## and the parameter is written. A pure-vertical swipe cannot see that --
	## which is why `_nwsize_probe.gd`'s five checks all survive `slop = 0`.
	var slop: float

	var _scroller: ScrollContainer
	var _looked := false
	var _family := 0            ## 0 idle, 1 touch, 2 mouse.
	var _verdict := 0           ## 0 undecided, 1 slider, -1 scroller.
	var _origin := Vector2.ZERO ## Press point, scroll-compensated (see `_track`).
	var _origin_scroll := 0
	var _press_value := 0.0
	var _started := false       ## Whether `drag_started` has been emitted.

	func _gui_input(event: InputEvent) -> void:
		## A disabled slider is not ours to write, and a slider with nothing to
		## scroll has no second axis to lose the gesture to. Returning WITHOUT
		## `accept_event()` is what hands the event back to `Slider::gui_input`
		## unchanged -- these two lines are the difference between "arbitrate"
		## and "replace".
		if not editable or _scroll() == null:
			return
		var family := 0
		var kind := 0           ## 1 press, 2 move, 3 release.
		var pos := Vector2.ZERO
		if event is InputEventScreenTouch:
			family = 1
			kind = 1 if (event as InputEventScreenTouch).pressed else 3
			pos = (event as InputEventScreenTouch).position
		elif event is InputEventScreenDrag:
			family = 1
			kind = 2
			pos = (event as InputEventScreenDrag).position
		elif event is InputEventMouseButton:
			var mb := event as InputEventMouseButton
			## Wheel, and every other button, stay the base class's business --
			## returning without accepting lets `Slider::gui_input` run.
			if mb.button_index != MOUSE_BUTTON_LEFT:
				return
			family = 2
			kind = 1 if mb.pressed else 3
			pos = mb.position
		elif event is InputEventMouseMotion:
			var mm := event as InputEventMouseMotion
			if (mm.button_mask & MOUSE_BUTTON_MASK_LEFT) == 0:
				return
			family = 2
			kind = 2
			pos = mm.position
		else:
			return
		accept_event()
		if _family != 0 and family != _family:
			return              ## The emulated twin of the gesture in progress.
		if kind == 1:
			_family = family
			_verdict = 0
			_started = false
			_press_value = value
			_origin_scroll = _scroll_now()
			_origin = _track(pos)
			return
		if _family == 0:
			return              ## A drag or a release with no press of ours.
		var d := _track(pos) - _origin
		if kind == 2:
			if _verdict == 0:
				if maxf(absf(d.x), absf(d.y)) < slop:
					return
				_verdict = -1 if absf(d.y) > absf(d.x) else 1
				if _verdict > 0:
					_begin_drag()
			if _verdict > 0:
				_apply_x(pos.x)
			elif _scroll() != null:
				_scroller.scroll_vertical = _origin_scroll - int(round(d.y))
			return
		if _verdict == 0:
			_begin_drag()
			_apply_x(pos.x)     ## A tap: Godot's own jump-to-the-tap, deferred.
		## Only a gesture that actually moved the value announces itself.
		## `_pg_range_field()` and `_pg_sculpt_slider()` both write the engine
		## from `drag_ended`, and `_pg_after_param_write()` marks the stage
		## stale off the same signal -- so a scroll, and a tap that lands on the
		## value the slider already held, now write nothing and mark nothing.
		##
		## **Deliberately unbalanced against `drag_started`.** Godot's own
		## `Slider` pairs the two on every press/release; here a horizontal drag
		## that lands back on the value it started from emits `drag_started` and
		## no `drag_ended`. That is the cheaper of two wrong answers: the only
		## `drag_started` consumer in the shell is
		## `civilization_workspace.gd::_lm_drag_start()`, which sets a flag the
		## next `drag_started` overwrites, while a spurious `drag_ended` would
		## re-post the parameter and mark a generation stage stale -- the exact
		## defect this class exists to prevent.
		if _verdict >= 0 and not is_equal_approx(value, _press_value):
			drag_ended.emit(true)
		_family = 0
		_verdict = 0
		_started = false

	## `Slider::gui_input` emits `drag_started` when it arms its grab, and this
	## class suppresses that press -- so the signal has to be re-emitted at the
	## moment the gesture is *known* to belong to the slider.
	## `civilization_workspace.gd:4590` connects it to record whether a landmark
	## cap drag began from the `off` stop, and without this that row's
	## "drag up from off resumes at 40" would silently stop working.
	func _begin_drag() -> void:
		if not _started:
			_started = true
			drag_started.emit()

	## `Slider::gui_input`'s own arithmetic, since this class is what replaces
	## it: the grabber's width is dead travel, half of it at each end.
	func _apply_x(x: float) -> void:
		var g := 0.0
		var tex: Texture2D = get_theme_icon("grabber")
		if tex != null:
			g = float(tex.get_width())
		var area := size.x - g
		if area <= 0.0:
			return
		set_as_ratio(clampf((x - g * 0.5) / area, 0.0, 1.0))

	## The finger's position in a frame that does not move when the scroller
	## does. `event.position` is local to this control, and this control slides
	## up the screen as the scroll it is driving advances -- so a delta taken
	## from raw local coordinates feeds itself and the list runs away under the
	## finger. Subtracting the scroll offset cancels exactly that term: the
	## control's global y is `C - scroll` for a constant `C`, so `pos.y - scroll`
	## is `finger_y - C` and the difference of two of them is pure finger travel.
	##
	## **Both axes, not just y.** The two `_pg_*` sliders this class was written
	## for sit in a scroller whose horizontal axis is `DISABLED`, so x needed no
	## treatment there; the phone tool sheet's own `ScrollContainer`
	## (`dcc_shell.gd`, `horizontal_scroll_mode = SCROLL_MODE_AUTO`) is not that
	## scroller, and neither is every window this now attaches to.
	func _track(pos: Vector2) -> Vector2:
		var sc := _scroll()
		if sc == null:
			return pos
		return Vector2(pos.x - float(sc.scroll_horizontal),
			pos.y - float(sc.scroll_vertical))

	## Resolved on first use rather than in `_ready()`, so it cannot depend on
	## whether this node was parented before or after its own ancestors were.
	## The lookup itself is `DccWidgets.vertical_scroller_above()` -- shared
	## with `touch_release_button()` below, because two gesture fixes that
	## disagree about what counts as "a scroller above me" is exactly the drift
	## this file keeps finding.
	func _scroll() -> ScrollContainer:
		if not _looked:
			_looked = true
			_scroller = DccWidgets.vertical_scroller_above(self)
		return _scroller

	func _scroll_now() -> int:
		return _scroll().scroll_vertical if _scroll() != null else 0

## Give an already-built slider the arbitration above.
##
## Returns whether it was attached, so a walk can count what it changed rather
## than assert that it walked. **Skipped for a slider that already carries a
## script** -- `PgSlider` itself, or any other subclass a surface deliberately
## gave it -- because `set_script()` would silently replace whatever that was.
##
## `set_script()` rather than constructing a `PgSlider` at each factory: the 242
## hazardous sliders the census found are built by seven different files, four
## of which no single lane owns, and `DccShell.phone_fit()` already walks every
## one of them. Attaching after construction also leaves every theme override,
## `custom_minimum_size` and signal connection exactly as its builder left them
## -- `set_script` replaces the script instance, not the object.
##
## `slop_px` is a distance TRAVELLED, in whatever pixels the surface lays out
## in, so it scales with `phone_fit()`'s own `unit` and is not floored at a tap
## target the way a hit area is.
static func touch_slider(s: HSlider, slop_px: float) -> bool:
	if s == null or s.get_script() != null:
		return false
	s.set_script(PgSlider)
	s.set("slop", maxf(1.0, slop_px))
	return true

## **The nearest ancestor `ScrollContainer` that actually scrolls vertically**,
## or `null`. Not merely the nearest one: a `ScrollContainer` with
## `vertical_scroll_mode` DISABLED has no vertical gesture to claim, and
## treating it as one would take a vertical drag away from a control and give
## it nowhere to go.
##
## Shared by `PgSlider._scroll()` and `touch_release_button()` below -- the one
## question both fixes ask, asked once. `_gestclass_probe.gd` and
## `_rangeswipe_probe.gd` each restate it independently, deliberately, so a
## census does not depend on the code under test to describe itself.
static func vertical_scroller_above(n: Node) -> ScrollContainer:
	var p: Node = n.get_parent()
	while p != null:
		if p is ScrollContainer and (p as ScrollContainer).vertical_scroll_mode \
				!= ScrollContainer.SCROLL_MODE_DISABLED:
			return p as ScrollContainer
		p = p.get_parent()
	return null

## **A dropdown opens its popup on touch-DOWN, and on a phone that means a
## vertical swipe that happens to begin on one opens the popup instead of
## scrolling -- and then the same gesture picks an item out of it.**
##
## Measured 2026-09-07 with `_gestclass_probe.gd` at 1080x2340, before this
## function existed: a jittered vertical swipe starting on a left-dock
## `DccWidgets.choice()` row took `sel=7 -> sel=3` with the sheet not moving a
## pixel, and the same swipe on the New World card's Archetype dropdown left
## its six-item popup standing open. It is the §1.14 slider defect one class
## over -- silent, not merely annoying -- and the main loop found it on glass
## on exactly the surface that must be scrolled to reach CREATE WORLD.
##
## **No second gesture arbiter, and that is the point.** `PgSlider` exists
## because `Slider` has no say in when it acts: `Slider::gui_input` calls
## `set_as_ratio()` from the press and `MOUSE_FILTER_PASS` does not stop it,
## since a `PASS` control is still picked and still runs its own handler. A
## `BaseButton` is not in that position -- **`action_mode` is the engine's own
## supported way to say "act on release"** -- so the fix here is a property and
## a mouse filter, not a subclass. Reusing `PgSlider`'s classification would
## have meant a second arbiter that can drift from the first, and would have
## cost the native fling, which this does not.
##
## Both halves are needed and neither is sufficient:
##
## * `ACTION_MODE_BUTTON_RELEASE` stops the popup opening under the finger, so
##   there is a gesture left to classify at all.
## * `MOUSE_FILTER_PASS` lets the press reach the `ScrollContainer` above, so
##   it arms its own touch drag -- and past the deadzone that scroll posts
##   `NOTIFICATION_SCROLL_BEGIN`, which is what cancels the button's pending
##   press. That cooperation is stock Godot and is already load-bearing here
##   for every plain `Button` in the left sheet (PH-05).
##
## **The in-tree control that proves the shape**: `CheckBox` is already
## `action_mode == ACTION_MODE_BUTTON_RELEASE` and already `PASS`, and the same
## probe leg measures a jittered vertical swipe starting on one leaving
## `button_pressed` alone, scrolling the sheet `222 -> 1020`, and a tap at the
## identical point still toggling it. That is this combination, measured, on
## this build, on a control nobody had to change.
##
## **Two gates**, both carried over from `PgSlider`'s own widening:
##
## * already `ACTION_MODE_BUTTON_RELEASE` -> nothing to do, and say so by
##   returning `false`. `ColorPickerButton` measures `1` (RELEASE) on 4.7.1 and
##   is not this defect; whether its `MOUSE_FILTER_STOP` blocks a scroll is a
##   separate question, unmeasured here and deliberately not changed.
## * no vertical-scrolling ancestor -> stock behaviour. There is nothing to
##   arbitrate against, so a press-to-open dropdown should keep opening on
##   press.
##
##   **This gate is load-bearing and it is NOT what leaves the menu bar
##   alone** -- that attribution was written here and is wrong. Measured
##   2026-09-07: **0 of the 7 `MenuButton`s carry `_phone_fitted`**, so
##   `phone_fit()` never walks the menu bar and this function is never called
##   on one; inverting the gate leaves all seven stock anyway. What the gate
##   actually protects is `asset_library_window.gd`'s `scroller=none`
##   dropdown, which the inverted mutation wrongly converts. The `scroller=none`
##   census row for the `MenuButton`s is true and was doing no work in the
##   argument -- two facts standing next to each other read as cause and
##   effect.
##
## **Pinned from both directions**, 2026-09-07, by a Python harness that
## replaces exact literals here, restores in a `finally` and hashes the file
## before and after (`SAME`, no residue):
##
## | mutation | `_gestclass_probe` at 1080x2340 |
## |---|---|
## | shipped | GREEN, census hazard 4 |
## | `action_mode` back to `..._PRESS` | 4 FAIL, hazard 22 -- the whole defect |
## | `mouse_filter` back to `STOP` | 2 FAIL, hazard 4 -- the value is safe and **the sheet still does not scroll** |
## | the scroller gate inverted (`!= null`) | 4 FAIL, hazard 22 |
##
## The middle row is the one worth reading: it is why both writes are here.
##
## Returns whether it changed anything, so a walk can count what it converted.
static func touch_release_button(b: BaseButton) -> bool:
	if b == null or b.action_mode == BaseButton.ACTION_MODE_BUTTON_RELEASE:
		return false
	if vertical_scroller_above(b) == null:
		return false
	b.action_mode = BaseButton.ACTION_MODE_BUTTON_RELEASE
	if b.mouse_filter == Control.MOUSE_FILTER_STOP:
		b.mouse_filter = Control.MOUSE_FILTER_PASS
	return true

## **A text field is the third member of the touch-DOWN family, and neither of
## the two fixes above reaches it.** Measured by a verifier on the New World
## card: a jittered vertical swipe on the **Seed** field gives scroll `0 -> 0`
## with its internal `SpinBoxLineEdit` focused, while the label column at the
## same `y` scrolls `0 -> 62`. On Android that focus raises the soft keyboard
## over the sheet the swipe was trying to scroll.
##
## **`action_mode` is not the switch and neither is `mouse_filter`, and both
## were measured rather than reasoned about.** A `LineEdit` is not a
## `BaseButton`, so it has no `action_mode` at all; and the census rows that
## matter most -- `SpinBoxLineEdit` in `AcceptDialog`, `new_world_dialog.gd`
## and `asset_library_window.gd` -- are **already `MOUSE_FILTER_PASS` and still
## eat the swipe**. A five-way table on a stock `LineEdit` in a live
## `ScrollContainer`, 4.7.1, one jittered vertical swipe each, each field
## scrolled into view first so the push is known to pick it:
##
## | configuration | scroll | focused by the swipe |
## |---|---|---|
## | stock | no | **yes** -- the defect |
## | `accept_event()` in `_gui_input` | no | **yes** |
## | `mouse_filter = PASS` alone | no | **yes** |
## | `focus_mode = FOCUS_NONE` | no | no |
## | `FOCUS_NONE` + this class driving the scroll | **yes** (200 -> 400) | no |
##
## Row two is the one that decides the design. **`accept_event()` cannot stop
## the focus**, because `Viewport::_gui_input_event` grabs focus for the
## control under a left press *before* it calls `_gui_call_input` -- so the
## grab has already happened by the time any `_gui_input` runs, script or C++.
## `focus_mode` is what the viewport reads there, so `focus_mode` is the lever.
## Row three is why `MOUSE_FILTER_PASS` is not: `LineEdit::gui_input` calls
## `accept_event()` on every left press (the engine's own comment says it is
## handled "even when the LineEdit is not editable"), and `PASS` forwards only
## what a control does **not** accept -- which is exactly why the `PASS`
## `SpinBoxLineEdit` rows in the census are as stuck as the `STOP` ones.
##
## So this class is `PgSlider`'s shape with a different lever: it withholds the
## press, classifies the gesture after `slop`, drives the ancestor
## `ScrollContainer` itself on a vertical, and takes focus only once the
## gesture is known to be a tap. `focus_mode` is parked at `FOCUS_NONE` between
## gestures and restored for exactly as long as the field is focused.
##
## **Two costs, stated rather than hidden**, both the same shape as
## `PgSlider`'s lost fling:
##
## * **A tap puts the caret at the end of the text, not under the finger.**
##   The press that would have positioned it is the press this class swallows,
##   and 4.7.1 exposes no pixel-to-column call to put it back
##   (`ClassDB.class_get_method_list("LineEdit")` has `set_caret_column` and
##   `get_scroll_offset` and nothing between them). End-of-text is the
##   non-destructive choice: it is where a keyboard-focused field already puts
##   it, and typing appends rather than overwrites. A field that asks for
##   `select_all_on_focus` keeps that instead -- `grab_focus()` does it and
##   this leaves the selection alone.
## * **Drag-to-select inside the field is not available on the phone.** A
##   horizontal verdict resolves to a tap rather than to a selection drag,
##   because the press it would have started was withheld.
##
## **Gated on a vertical-scrolling ancestor, and that gate is load-bearing
## here in a way it is not for the two fixes above**: `focus_mode` is written
## at attach time, so a field with nothing to arbitrate against must not be
## attached at all -- parking `FOCUS_NONE` on it would leave a field that
## cannot be focused by anything. `touch_focus_field()` below is where that is
## checked. The `scroller=none` text fields the census counts (the phone root,
## `open_project_dialog.gd`, `travel_library_window.gd`,
## `world_data_window.gd`, `layers_popover.gd`, one in
## `asset_library_window.gd`, and 47 engine-internal ones) are left stock.
##
## **`editable` is deliberately NOT a gate**, and that is a difference from
## `PgSlider` rather than an oversight. There the harm is writing a disabled
## control's value; here the harm is eating the scroll, which a read-only field
## does exactly as much as an editable one -- and a read-only field raises no
## keyboard, so converting it is strictly an improvement. The two `live=false`
## `LineEdit`s the census finds in an `AcceptDialog` are in that position.
class PgField extends LineEdit:
	## Android's `ViewConfiguration.getScaledTouchSlop()`, in the pixels the
	## surface lays out in. Set by `touch_focus_field()`, which is the only way
	## a field becomes a `PgField` and which takes the value as a required
	## argument -- so **this carries no default either**. It was removed in the
	## same change as `PgSlider.slop`'s and for the same reason, which is the
	## point: house style had the identical dead default in both classes, and
	## fixing one would have left the next reader believing the other was
	## deliberate. A declared `:= 8.0` here was unreachable -- mutated to
	## `400.0` and to `0.0` it left `_gestclass_probe.gd` green both ways
	## (2026-09-07). `_slop_walk()` in that probe is what asserts it now; read
	## `PgSlider.slop` for why an assertion beats a fallback here.
	var slop: float
	## `focus_mode` as the field's builder left it, restored for the duration
	## of a focus and parked at `FOCUS_NONE` again on `focus_exited`.
	var stock_focus := Control.FOCUS_ALL

	var _scroller: ScrollContainer
	var _looked := false
	var _family := 0            ## 0 idle, 1 touch, 2 mouse. See `PgSlider`.
	var _verdict := 0           ## 0 undecided, 1 field, -1 scroller.
	var _origin := Vector2.ZERO
	var _origin_scroll := 0

	func _gui_input(event: InputEvent) -> void:
		## Nothing to arbitrate against -> hand the event back untouched. The
		## attacher refuses this case, so reaching it means the tree changed
		## under us; stock behaviour is the safe answer either way.
		if _scroll() == null:
			return
		var family := 0
		var kind := 0           ## 1 press, 2 move, 3 release.
		var pos := Vector2.ZERO
		if event is InputEventScreenTouch:
			family = 1
			kind = 1 if (event as InputEventScreenTouch).pressed else 3
			pos = (event as InputEventScreenTouch).position
		elif event is InputEventScreenDrag:
			family = 1
			kind = 2
			pos = (event as InputEventScreenDrag).position
		elif event is InputEventMouseButton:
			var mb := event as InputEventMouseButton
			## Right-click is the context menu and the wheel is the wheel;
			## returning without accepting lets `LineEdit::gui_input` have them.
			if mb.button_index != MOUSE_BUTTON_LEFT:
				return
			family = 2
			kind = 1 if mb.pressed else 3
			pos = mb.position
		elif event is InputEventMouseMotion:
			var mm := event as InputEventMouseMotion
			if (mm.button_mask & MOUSE_BUTTON_MASK_LEFT) == 0:
				return
			family = 2
			kind = 2
			pos = mm.position
		else:
			## Keys, IME and everything else stay the base class's business --
			## a focused field must still type.
			return
		accept_event()
		if _family != 0 and family != _family:
			return              ## The emulated twin of the gesture in progress.
		if kind == 1:
			_family = family
			_verdict = 0
			_origin_scroll = _scroll_now()
			_origin = _track(pos)
			return
		if _family == 0:
			return
		var d := _track(pos) - _origin
		if kind == 2:
			if _verdict == 0:
				if maxf(absf(d.x), absf(d.y)) < slop:
					return
				_verdict = -1 if absf(d.y) > absf(d.x) else 1
			if _verdict < 0:
				_scroller.scroll_vertical = _origin_scroll - int(round(d.y))
			return
		## A tap, or a horizontal that resolved to the field: this is the
		## deferred half of the press that was withheld.
		if _verdict >= 0:
			_take_focus()
		_family = 0
		_verdict = 0

	func _take_focus() -> void:
		if stock_focus == Control.FOCUS_NONE:
			return              ## Its builder did not want it focusable.
		focus_mode = stock_focus
		grab_focus()
		## `grab_focus()` runs `select_all_on_focus` itself; overwriting the
		## caret afterwards would silently undo it.
		if not select_all_on_focus:
			caret_column = text.length()

	## Park `focus_mode` again the moment the field stops being focused, so the
	## next swipe that begins on it is arbitrated rather than focused. Connected
	## by `touch_focus_field()`, because `set_script()` on a node already in the
	## tree does not re-run `_ready()`.
	func _relock() -> void:
		focus_mode = Control.FOCUS_NONE

	## Identical to `PgSlider._track()` and for the identical reason: this
	## control slides up the screen as the scroll it is driving advances, so a
	## delta taken from raw local coordinates feeds itself.
	func _track(pos: Vector2) -> Vector2:
		var sc := _scroll()
		if sc == null:
			return pos
		return Vector2(pos.x - float(sc.scroll_horizontal),
			pos.y - float(sc.scroll_vertical))

	func _scroll() -> ScrollContainer:
		if not _looked:
			_looked = true
			_scroller = DccWidgets.vertical_scroller_above(self)
		return _scroller

	func _scroll_now() -> int:
		return _scroll().scroll_vertical if _scroll() != null else 0

## Give an already-built text field the arbitration above.
##
## Returns whether it was attached, so a walk can count what it changed.
## Skipped for a field that already carries a script, exactly as
## `touch_slider()` is, because `set_script()` would replace it.
##
## **The scroller is checked HERE and not lazily**, which is the one place this
## differs from `touch_slider()`. `PgSlider` can defer the question to its
## first event because everything it does happens inside `_gui_input`; this
## class writes `focus_mode` at attach time, and a field parked at
## `FOCUS_NONE` with no scroller to arbitrate against would simply be a field
## that can no longer be focused.
##
## `SpinBox`'s field is an **internal** child, so `DccShell.phone_fit()`'s
## `get_children()` walk never reaches it -- the `SpinBox` call site passes
## `get_line_edit()` explicitly. **12 of the 16 hazardous fields at boot**
## are in that position -- 20 of 24 at maximum, with a world loaded. This
## said "12 of 34"; **no state produces 34**. That figure was 16 plus the
## 18 hidden `PopupMenu` incremental-search fields, which are unreachable
## by any gesture and which this same change taught the census to report
## separately rather than count.
static func touch_focus_field(le: LineEdit, slop_px: float) -> bool:
	if le == null or le.get_script() != null:
		return false
	if vertical_scroller_above(le) == null:
		return false
	var stock: int = le.focus_mode
	le.set_script(PgField)
	le.set("slop", maxf(1.0, slop_px))
	le.set("stock_focus", stock)
	le.focus_mode = Control.FOCUS_NONE
	if not le.focus_exited.is_connected(Callable(le, "_relock")):
		le.focus_exited.connect(Callable(le, "_relock"))
	return true

## **Repaint-on-palette-flip**, for the two inputs whose appearance is carried
## by something `DccShell`'s recolour walks cannot reach: an `ImageTexture`
## (nothing can reach inside one -- `_style_popup_marks()` above carries the
## same exposure and says so) and the `*_icon_modulate` colour family, which is
## not in `_THEME_COLOR_OVERRIDES`. Both lists live in `dcc_shell.gd`, which
## this pass does not own.
##
## `Theme.emit_changed()` is the last thing `DccShell._recolor_project_theme()`
## does, so every `Control` in the tree gets `theme_changed` on a flip, and
## this is the cheapest hook that already exists.
##
## **The guard is the palette itself rather than a re-entrancy flag**, and the
## order below is the whole reason it terminates: `painter` sets theme
## overrides and *each one re-emits `theme_changed`*, so the handler is
## guaranteed to be called from inside itself. Writing the meta before the
## paint means the re-entrant call finds it already equal to the live palette
## and returns. The first paint runs before the connection exists, so it cannot
## recurse at all.
const PALETTE_META := "dcc_palette_dark"

static func _palette_watch(ctl: Control, painter: Callable) -> void:
	ctl.set_meta(PALETTE_META, DccTheme.is_dark())
	painter.call()
	ctl.theme_changed.connect(func() -> void:
		if bool(ctl.get_meta(PALETTE_META, DccTheme.is_dark())) == DccTheme.is_dark():
			return
		ctl.set_meta(PALETTE_META, DccTheme.is_dark())
		painter.call())

static func _paint_switch(cb: CheckBox) -> void:
	var touch := DccTheme.is_touch()
	var w := 40 if touch else 30
	var h := 22 if touch else 17
	var knob := 18 if touch else 13
	cb.add_theme_icon_override("checked", _switch(w, h, knob, true))
	cb.add_theme_icon_override("unchecked", _switch(w, h, knob, false))
	cb.add_theme_icon_override("checked_disabled", _switch(w, h, knob, true, false))
	cb.add_theme_icon_override("unchecked_disabled", _switch(w, h, knob, false, false))
	## **Identity modulate, and it has to be per-instance.** Godot 4.7's
	## `CheckBox` tints its icon with `checkbox_checked_color` /
	## `checkbox_unchecked_color`, which `res://theme/dark_theme.tres` sets and
	## which therefore reached these four textures *after* they were drawn --
	## so `e830112`'s stylebox proof measured zero movement while the switch on
	## the screen was the wrong colour. Measured by `_ckpix_probe`: the ON knob
	## arrived `#cc9644` instead of the canvas `--ink` `#e8ebec`, and the
	## disabled knob `#53401e` instead of `#5f6468`.
	##
	## **Two names cover four slots.** The same probe's `[5]` block forces only
	## this pair on a *disabled* switch and gets `#5f6468` back exactly, so
	## `checked_disabled` and `unchecked_disabled` modulate from these two as
	## well and there is nothing to add for them.
	##
	## **Not a theme item, and that is not a style preference.** Two reasons,
	## both measured. (1) Seven bare `CheckBox.new()` sites --
	## `journey_planner_view.gd` 2444/2482/2487 and `travel_library_window.gd`
	## 844/868/900/931 -- draw Godot's own check glyph and *need* the tint
	## kept; one theme item cannot be both an identity and a tint. (2)
	## `_cklight_probe`'s `[4]` block runs the shipping
	## `DccShell._recolor_project_theme()` path over pure white and watches it
	## come back rewritten, because white has no exact-RGBA token and falls
	## into the RGB-only pass against `line` = `Color(1,1,1,.10)`. A white
	## entry in the resource would not survive one palette flip. Here it is a
	## literal on the instance, so `_palette_watch()` re-asserts it unchanged.
	cb.add_theme_color_override("checkbox_checked_color", Color(1, 1, 1))
	cb.add_theme_color_override("checkbox_unchecked_color", Color(1, 1, 1))

static func _paint_spin_arrows(sb: SpinBox) -> void:
	for side in ["up", "down"]:
		sb.add_theme_color_override(side + "_icon_modulate",
			DccTheme.c("text_secondary"))
		sb.add_theme_color_override(side + "_hover_icon_modulate",
			DccTheme.c("accent"))
		sb.add_theme_color_override(side + "_pressed_icon_modulate",
			DccTheme.c("accent"))
		sb.add_theme_color_override(side + "_disabled_icon_modulate",
			DccTheme.c("text_ghost"))

## **The canvas's toggle is a switch, not a check box.** Read off the PC
## artboard, which draws it in three places (`ENV:76`, `ENV:365`, `ENV:846`)
## with one state table (`ENV:1354`, `ENV:1838`):
##
##   track  width:30px;height:17px;border-radius:999px;background:{{ togBg }}
##   knob   width:13px;height:13px;border-radius:50%;top:2px;left:{{ togX }}px
##   togBg  v ? var(--acc) : var(--sur)        togX  v ? 15 : 2
##   knob   background:var(--ink)
##
## `CheckBox` draws its state from four theme *icons*, so the switch arrives as
## a texture the same way `_style_popup_marks()`'s marks do. Nothing about the
## node changes: still a `BaseButton`, still the tap target `toggle()`'s header
## sizes, still `button_pressed`. Only what it paints.
##
## **The touch pair is borrowed, and labelled as borrowed.**
## `Cartalith Tablet.dc.html` contains no switch at all (`grep togBg` returns
## nothing in it), so there is no tablet literal to read;
## `Cartalith Android.dc.html` draws `width:40px;height:22px` with an `18px`
## knob at `top:2px` (`AND:311`, `AND:440`, `AND:521`). **Geometry only.** That
## canvas's own colours -- `--chip` off, `--accInk`/`--sec` knob -- are not
## borrowed with it, because the PC canvas is the one `DccTheme`'s palette is
## derived from and mixing the two gives a toggle that matches neither.
##
## `left:2px` / `left:15px` on a 30 px track with a 13 px knob is 2 px of inset
## at both ends, which is what generalises to the 40/18 pair rather than the
## two literals themselves.
static func _switch(w: int, h: int, knob: int, on: bool,
		enabled: bool = true) -> ImageTexture:
	var track := DccTheme.c("accent") if on else DccTheme.c("bg")
	var pin := DccTheme.c("text_bright")
	if not enabled:
		track = DccTheme.c("sunken")
		pin = DccTheme.c("text_ghost")
	var img := Image.create(w, h, false, Image.FORMAT_RGBA8)
	img.fill(Color(0, 0, 0, 0))
	var r := h * 0.5
	var half := Vector2(w, h) * 0.5
	var kx: float = (w - 2.0 - knob * 0.5) if on else (2.0 + knob * 0.5)
	var kc := Vector2(kx, h * 0.5)
	for y in h:
		for x in w:
			var p := Vector2(x + 0.5, y + 0.5)
			## Signed distance to a rounded rectangle, so the pill's ends are
			## antialiased by the same one-pixel coverage ramp `_round_dot()`
			## uses. `half - r` collapses to the spine when `r == h / 2`.
			var q := Vector2(absf(p.x - half.x) - (half.x - r),
				absf(p.y - half.y) - (half.y - r))
			var d: float = Vector2(maxf(q.x, 0.0), maxf(q.y, 0.0)).length() \
				+ minf(maxf(q.x, q.y), 0.0) - r
			var cover: float = clampf(0.5 - d, 0.0, 1.0)
			if cover <= 0.0:
				continue
			var kcov: float = clampf(knob * 0.5 - (p - kc).length() + 0.5, 0.0, 1.0)
			var rgb := track.lerp(pin, kcov)
			img.set_pixel(x, y, Color(rgb.r, rgb.g, rgb.b, cover))
	return ImageTexture.create_from_image(img)

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

## A text field -- the canvas's Tile size / World bounds / Destination wells
## and the Asset library's search.
##
## **Filled, borderless, rounded** -- corrected 2026-09-07 with the rest of the
## inputs. What stood here was `box(border, "", px, py)`: a hairline rectangle
## over nothing, square-cornered because `DccTheme.outline()` never sets a
## corner radius. The canvas draws the opposite of all three, in four places:
##
##   ENV:222 / ENV:498 / ENV:1101 / ENV:1118
##   background:var(--ins);border:none;border-radius:8px;
##   min-height:var(--ctl);padding:2..4px 11px;color:var(--ink);outline:none
##
## **This one can be fixed in place where `DccTheme.outline()` could not**, and
## the difference is the whole test the button pass used: `grep -rn
## "DccWidgets.well("` returns 12 call sites and all 12 are `LineEdit`s. There
## is no non-field caller to repaint by accident, so the blast radius is the
## control this change is about.
##
## `px`/`py` are untouched on purpose -- every caller either takes the 9/4
## default or passes its own, and moving them moves layout at all 12 sites.
## The canvas's `11px` is a horizontal figure this pass does not spend its
## invariance budget on.
##
## Ink moves `text` -> `text_bright`, which is the canvas's `color:var(--ink)`
## on every one of those four inputs; `text` is `--body`, one step down.
static func well(le: Control, px: int = 9, py: int = 4, accent: bool = false) -> void:
	var norm := DccTheme.field_box("normal", px, py)
	if accent:
		norm.border_color = DccTheme.c("accent")
		norm.set_border_width_all(1)
	le.add_theme_stylebox_override("normal", norm)
	le.add_theme_stylebox_override("focus", DccTheme.field_box("focus", px, py))
	le.add_theme_stylebox_override("read_only",
		DccTheme.field_box("read_only", px, py))
	le.add_theme_font_override("font", DccTheme.mono(0))
	le.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	le.add_theme_color_override("font_color", DccTheme.c("text_bright"))
	le.add_theme_color_override("font_placeholder_color", DccTheme.c("text_ghost"))
	le.add_theme_color_override("font_uneditable_color", DccTheme.c("text_ghost"))
	le.add_theme_color_override("caret_color", DccTheme.c("accent"))
	## **Selected text, and the eighth colour item.** `selection_color` comes
	## from `dark_theme.tres` as `accent` at `a=0.35` and does follow the
	## palette; `font_selected_color` does **not** -- it is `#ffffff` out of
	## Godot's own default theme, which nothing in this shell remaps. On the
	## light palette that puts white glyphs on the pale tan band the accent
	## composites to: `_cklight_probe` measured the band at `#d3bb99` and the
	## brightest glyph at `#f6f1ea`, **1.26:1**, i.e. selecting text made it
	## disappear. Holding the field's own ink through the selection is the
	## same move the canvas makes for `::selection` nowhere at all -- it draws
	## no selection state -- so this is derived, and derived the cheap way:
	## the ink that already reads on this ground keeps reading on a wash of it.
	_palette_watch(le, func() -> void:
		le.add_theme_color_override("font_selected_color",
			DccTheme.c("text_bright")))

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
	## `+ kb`: the on-screen keyboard draws over the frame instead of resizing
	## it (`dcc_shell.gd`'s own `_process()` comment), so a window sized only to
	## the gesture inset puts this dialog's OK/Cancel bar -- its one bottom-
	## docked control -- under the IME exactly as the shell's own docked chrome
	## would without `_phone_kb_height`. Read rather than re-polled: the shell
	## already owns the one `DisplayServer.virtual_keyboard_get_height()` poll
	## (polling a second time risks a second "not supported" warning on a
	## display with no IME feature), and `phone_insets_changed` -- which
	## `phone_window()`'s `relay` already re-runs this function from -- fires on
	## every change to it, keyboard included. `+`, not `max()`, to match how
	## `_apply_phone_nav_orientation()`'s own `gesture` and
	## `phone_content_insets()`'s own `bottom` already fold this height in.
	## Reached directly rather than through a new getter: `dcc_shell.gd` is a
	## concurrent lane's file this batch, so a `phone_kb_height()` accessor
	## belongs there next to `is_phone()`/`phone_scale()` -- which its own
	## comment already flags as read-only-by-convention -- once it is free to
	## edit again; this keeps that read to the one call site meanwhile.
	var kb := int(host._phone_kb_height) if host is DccShell else 0
	var target := Vector2i(int(screen.x), maxi(1, int(screen.y) - gesture - kb))
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
## Gen info, the (since-deleted) Performance window and the credits sheet all ran
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
## data, Gen info, the (since-deleted) Performance window and the credits sheet
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
	## is lost. That was `app.gd::_floor_prompt_buttons()`'s first trap; that
	## function was deleted on 2026-09-05 when `confirm_unsaved_world()` moved
	## to `modal_card()`, and the trap is recorded here because it applies
	## verbatim to this loop and to any other written against a button bar.
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

# ---------------------------------------------------------------------------
# Grading a window against the phone protocol -- and the two helpers that make
# a call site pass it by construction
#
# Reaching the phone correctly was three calls spread across build time and
# open time: `phone_window()` before the body, `DccShell.phone_fit(dlg, 1.0)`
# after it, `phone_present()` instead of `popup_centered()`. A call site can
# complete none of them, some of them, or -- twice in this tree -- the WRONG
# ones: `vault_window.gd::_compare_dialog` and `::_preview_dialog` both call
# `phone_fit()` without `phone_window()`, so the fit they perform is wrong
# rather than absent, and nothing caught it.
#
# **The rule encoded below is deliberately NOT "all three calls ran".** It is
# the three things a hand can feel, measured off the drawn window:
#
#   1. phone-SHAPED     -- `wrap_controls` off and the unscaled title bar gone
#                          (what `phone_window()` does)
#   2. phone-PRESENTED  -- filling the screen above the gesture inset, at the
#                          handset content scale, with the desktop
#                          `min_size`/`max_size` cleared (what
#                          `phone_present()` does)
#   3. every tappable control at or above §13's 44 dp floor **and inside the
#      window's own client rect**
#
# Stating (3) as the floor rather than as "`phone_fit()` ran" is the whole
# reason this grades correctly, and it is why `modal_present()` can go on
# skipping `phone_fit()`: a `modal_card()`'s buttons are floored at
# construction by `_modal_btn()`, so the card clears the floor by a different
# route. A conformance check written against the call list would have failed
# the one path that is already right.
#
# `_phoneproto_probe.gd` is the proof this discriminates: it builds one
# specimen per known-bad shape found in the census and asserts each is graded
# `fails:` before asserting the two helpers below are graded `ok`.
# ---------------------------------------------------------------------------

## `DccShell.phone_fit()`'s own idempotence flag, by its literal name. Read
## here only to report *how* a window got its tap floor -- never to decide
## whether it has one. The probe checks the literal still matches by calling
## `phone_fit()` and watching the flag appear, not by comparing it against the
## constant it was copied from.
const PHONE_FIT_META := "_phone_fitted"

## Every tappable descendant, INCLUDING internal children. `get_children()`
## skips `AcceptDialog`'s own button bar, which is exactly how a shipped OK
## button stayed 29 dp through four windows.
static func _tappables(n: Node, out: Array, fitted: Array) -> void:
	for c in n.get_children(true):
		if c is Control and (c as Control).has_meta(PHONE_FIT_META):
			fitted.append(c)
		if c is BaseButton or c is LineEdit or c is SpinBox or c is TextEdit:
			out.append(c)
		_tappables(c, out, fitted)

## Grades an **open, laid-out** window against the three points above.
##
## `verdict` is `desktop` when the host is not a phone (there is nothing to
## grade), `gone` when the window has been freed, `ok`, or `fails:` followed by
## the failing legs -- `shape`, `present`, `tap` -- comma separated.
##
## `backwards` is the partly-right case the census found and the reason this
## function exists: `phone_fit()` ran but the window was never phone-shaped.
static func phone_protocol_grade(dlg: Window, host) -> Dictionary:
	var out := {"verdict": "desktop", "shaped": false, "presented": false,
		"tap_ok": false, "fit_ran": false, "backwards": false,
		"worst": 0.0, "worst_name": "", "offscreen": 0, "tappables": 0}
	if dlg == null or not is_instance_valid(dlg):
		out["verdict"] = "gone"
		return out
	if host == null or not host.has_method("is_phone") or not host.is_phone():
		return out

	## Both halves, because either alone has another explanation: `modal_card()`
	## sets `borderless` on every platform, and a caller can turn
	## `wrap_controls` off by hand for the growth bug without ever meeting the
	## phone. Only the pair is `phone_window()`'s signature.
	out["shaped"] = (not dlg.wrap_controls) and dlg.borderless

	var scale: float = host.phone_scale()
	var screen: Vector2 = host.get_viewport_rect().size
	var gesture := int(round(DccTheme.H_PHONE_GESTURE * scale))
	## `- kb`: mirrors `phone_present()`'s own target-size formula. Left out
	## once, this grader reported a correctly-presented dialog as broken the
	## moment the IME was up -- found by an adversarial verifier, not by a
	## device report; the only two callers today (`_phoneproto_probe.gd`,
	## `_vfyproto_probe.gd`) run at kb=0, so it was latent, not yet observed.
	var kb := int(host._phone_kb_height) if host is DccShell else 0
	out["presented"] = dlg.position == Vector2i.ZERO \
		and dlg.size.x == int(screen.x) \
		and dlg.size.y == maxi(1, int(screen.y) - gesture - kb) \
		and absf(dlg.content_scale_factor - scale) < 0.001 \
		and dlg.min_size == Vector2i.ZERO and dlg.max_size == Vector2i.ZERO

	## The window's own client rect, in the CONTENT units its children are laid
	## out in -- `Window.size` is physical and `content_scale_factor` divides
	## it. `get_global_rect()` is unclipped, so a window that popped wider than
	## the viewport would otherwise report every control "shown" while it sits
	## off the screen.
	var csf := maxf(0.001, dlg.content_scale_factor)
	var client := Rect2(Vector2.ZERO, Vector2(dlg.size) / csf)
	var tapped: Array = []
	var fitted: Array = []
	_tappables(dlg, tapped, fitted)
	out["fit_ran"] = not fitted.is_empty()
	out["backwards"] = bool(out["fit_ran"]) and not bool(out["shaped"])
	var worst := 1e9
	var worst_name := ""
	var off := 0
	var seen := 0
	for c in tapped:
		var ctl := c as Control
		if ctl == null or not ctl.is_visible_in_tree():
			continue
		seen += 1
		if ctl.size.y < worst:
			worst = ctl.size.y
			worst_name = String(ctl.name)
			if ctl is Button and (ctl as Button).text != "":
				worst_name = (ctl as Button).text
		if not client.encloses(ctl.get_global_rect()):
			off += 1
	out["tappables"] = seen
	out["offscreen"] = off
	out["worst"] = 0.0 if worst > 1e8 else worst
	out["worst_name"] = worst_name
	## No tappable control at all is not a pass by default -- it means the
	## window was measured before it laid out, which is a probe defect and must
	## not read as conformance.
	out["tap_ok"] = seen > 0 and worst >= float(DccTheme.PHONE_TAP_MIN) and off == 0

	if bool(out["shaped"]) and bool(out["presented"]) and bool(out["tap_ok"]):
		out["verdict"] = "ok"
	else:
		var miss := PackedStringArray()
		if not bool(out["shaped"]):
			miss.append("shape")
		if not bool(out["presented"]):
			miss.append("present")
		if not bool(out["tap_ok"]):
			miss.append("tap")
		out["verdict"] = "fails:" + ",".join(miss)
	return out

## One line for a log. `phone_protocol_grade()` returns the numbers; this is
## the only place they are formatted, so a probe and a future call site cannot
## disagree about what a grade reads like.
static func phone_protocol_line(tag: String, g: Dictionary) -> String:
	return "%-26s %-22s shaped=%d present=%d tap=%d fit_ran=%d backwards=%d worst=%.0fdp(%s) off=%d n=%d" % [
		tag, String(g.get("verdict", "?")), int(bool(g.get("shaped", false))),
		int(bool(g.get("presented", false))), int(bool(g.get("tap_ok", false))),
		int(bool(g.get("fit_ran", false))), int(bool(g.get("backwards", false))),
		float(g.get("worst", 0.0)), String(g.get("worst_name", "")),
		int(g.get("offscreen", 0)), int(g.get("tappables", 0))]

## Frees the dialog once, on either answer, and runs the caller's callback
## after it -- the shape all six text-only sites write out by hand, three of
## them with a subtly different one (`visibility_changed`, a bare `queue_free`
## bind, a `confirmed` that frees only on the success branch).
static func _wire_dismiss(dlg: AcceptDialog, on_confirm: Callable,
		on_cancel: Callable) -> void:
	var dismiss := func():
		if is_instance_valid(dlg) and not dlg.is_queued_for_deletion():
			dlg.queue_free()
	dlg.confirmed.connect(func():
		dismiss.call()
		if on_confirm.is_valid():
			on_confirm.call())
	dlg.canceled.connect(func():
		dismiss.call()
		if on_cancel.is_valid():
			on_cancel.call())

## The borderless phone body a `phone_window()`ed dialog needs: the header it
## traded its title bar for, and a scroller under it. Returns the column a
## caller fills.
##
## The scroller's horizontal axis is DISABLED, which folds its child's minimum
## width into its own -- right here, because a 393 dp column is the whole width
## there is, and the column below it is the only sibling, so there is nothing
## beside it to overflow.
static func _phone_dialog_body(dlg: AcceptDialog, title: String) -> VBoxContainer:
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	dlg.add_child(col)
	phone_head(col, title, "")
	var m := pad(col, 16, 14, 16, 14)
	m.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	m.add_child(scroll)
	var inner := VBoxContainer.new()
	inner.add_theme_constant_override("separation", 8)
	inner.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(inner)
	return inner

## The text-only confirmation, protocol-complete.
##
## Six sites in this shell hand-build exactly this shape and **not one of the
## six reaches the phone at all**: `faction_roster_window.gd::_confirm_remove`,
## `place_editor_window.gd::confirm_delete`, `right_dock.gd::_confirm_revert`,
## `civilization_workspace.gd::_confirm_destructive` and `::_tl_show_confirm`,
## `world_workspace.gd::_confirm_discard`.
##
## Still a stock `ConfirmationDialog` and not a `modal_card()`, deliberately.
## `menus.gd::_open_pack_metadata`'s own audit note settles that question for
## this shell: converting OS chrome to the card "changes what a user sees,
## which is an owner call, not an audit's". Nothing here changes what a
## **desktop** user sees; the phone body is the defect being fixed.
##
## Two traps a hand-written call site cannot see, both absorbed here:
##   * `phone_window()` assigns `ok_button_text = "Close"`, which is right for
##     a one-way window and wrong for a two-way question -- so `ok_text` is
##     written AFTER it, never before. Every one of the six names its answer
##     ("Remove", "Delete", "Revert", "Overwrite"), and that wording rule is
##     the first thing a naive conversion loses.
##   * `phone_window()` drops the decoration, taking the OS title bar and with
##     it `title`. On a phone the question therefore carries its own
##     `phone_head()`; on desktop the stock `dialog_text` path is untouched.
static func confirm(host: Node, title: String, text: String, ok_text: String,
		on_confirm: Callable, on_cancel: Callable = Callable(),
		width: int = 380) -> ConfirmationDialog:
	var dlg := ConfirmationDialog.new()
	dlg.title = title
	var phone := phone_window(dlg, host)
	dlg.ok_button_text = ok_text
	if phone:
		modal_prose(_phone_dialog_body(dlg, title), text)
	else:
		dlg.dialog_text = text
		dlg.min_size = Vector2i(width, 0)
	host.add_child(dlg)
	_wire_dismiss(dlg, on_confirm, on_cancel)
	if phone and host.has_method("phone_fit"):
		host.phone_fit(dlg, 1.0)
	if not phone_present(dlg, host):
		dlg.popup_centered()
	return dlg

## The one-field prompt, protocol-complete. Three sites hand-build it and none
## of the three reaches the phone: `journey_planner_view.gd::_save_journey` and
## `::_capture_preset`, `cartography_workspace.gd::_prompt_label_name`.
##
## Returns `{"dialog", "field", "body"}`. `body` is handed back rather than
## kept private because two of the three sites append a row this helper has no
## business knowing about -- a live name-clash warning, a persistence note --
## and a helper that made them give those up would not be adopted.
##
## `on_submit` receives the **trimmed** text and runs only when it is not
## empty, which is what all three sites already do by hand. Enter in the field
## commits, which two of the three had and one did not.
static func prompt(host: Node, title: String, field_label: String,
		initial: String, ok_text: String, on_submit: Callable,
		hint: String = "", width: int = 360) -> Dictionary:
	var dlg := ConfirmationDialog.new()
	dlg.title = title
	var phone := phone_window(dlg, host)
	dlg.ok_button_text = ok_text
	var body: VBoxContainer
	if phone:
		body = _phone_dialog_body(dlg, title)
	else:
		body = VBoxContainer.new()
		body.add_theme_constant_override("separation", 6)
		dlg.add_child(body)
		dlg.min_size = Vector2i(width, 0)
	if field_label != "":
		body.add_child(DccTheme.label(field_label, "text_dim", DccTheme.FS_SMALL))
	var le := LineEdit.new()
	le.text = initial
	le.select_all_on_focus = true
	well(le)
	body.add_child(le)
	if hint != "":
		note(body, hint)
	host.add_child(dlg)

	var submit := func():
		var typed := le.text.strip_edges()
		if is_instance_valid(dlg) and not dlg.is_queued_for_deletion():
			dlg.hide()
			dlg.queue_free()
		if typed != "" and on_submit.is_valid():
			on_submit.call(typed)
	dlg.confirmed.connect(submit)
	## A focused `LineEdit` consumes Enter before the dialog's own default
	## button ever sees it -- the same reason `modal_choices()` stamps
	## `MODAL_DEFAULT_META` for a card with a field in it.
	le.text_submitted.connect(func(_t: String): submit.call())
	dlg.canceled.connect(func():
		if is_instance_valid(dlg) and not dlg.is_queued_for_deletion():
			dlg.queue_free())

	if phone and host.has_method("phone_fit"):
		host.phone_fit(dlg, 1.0)
	if not phone_present(dlg, host):
		dlg.popup_centered()
	le.grab_focus.call_deferred()
	return {"dialog": dlg, "field": le, "body": body}

# ---------------------------------------------------------------------------
# The modal card
#
# `design/proposed-2026-09-05/Modal.dc.html`, approved by the owner 2026-09-05
# ("I like the layouts as proposed, implement those"). One pattern, three
# variants, drawn side by side in that artboard:
#
#   A CONFIRM      Cancel / Discard / Save and close -- a question with a safe
#                  answer, so the safe answer is the filled one and sits last.
#   B DESTRUCTIVE  Cancel / Clear 128 packs -- the destructive act IS the
#                  dialog's purpose, so it sits last, in a block wash, and
#                  NEVER in the accent fill.
#   C WARNINGS     Copy report / Done -- a result, not a question, so there is
#                  no Cancel.
#
# **The ordering rule, which is the whole point of the pattern and is enforced
# by `modal_choices()` rather than left to a call site:** where a safe action
# exists it is the filled one and sits last, and the destructive action beside
# it is text-only, never filled and never the rightmost. Where the destructive
# action is the only action (B) it is last -- there is nothing safe to put
# after it -- but it takes the block wash, because the accent fill is what
# tells a reader "this is the safe answer".
#
# Radius. §11's "radius 0 everywhere" is a rule from the 2026-08 desktop
# canvases; this artboard is newer and draws `border-radius:10px` on the card
# and `8px` on its insets and buttons, so the owner's 2026-08-25 ruling ("when
# two design canvases disagree, the newer one wins") settles it for a floating
# surface. Nothing that is not a modal changes: `DccTheme.outline()` still
# returns radius 0 and every existing caller of it is untouched.
#
# Colour tokens, resolved out of `dcc_theme.gd` rather than pasted from the
# artboard's hex (each was matched by grepping the hex in `dcc_theme.gd`):
#   `--pan`  #121314        -> `panel`          (the card ground)
#   `--ins`  #191c1e        -> `sunken`         (the stat block, the list)
#   `--bor`  rgba(255,.16)  -> `border`         ("anything that floats")
#   `--div`  rgba(255,.07)  -> `line_soft`      (the header rule, list rules)
#   `--faint`#6f7478        -> `text_faint`     (title, keys, the closer)
#   `--body` #c8cbcd        -> `text`           (the prose line)
#   `--sec`  #a9adb0        -> `text_secondary` (values, list text)
#   `--dim`  #8d9296        -> `text_dim`
#   `--dis`  #5f6468        -> `text_ghost`     (the foot note, and dashes)
#   `--acc`  #e0a34a        -> `accent`
#   `--accInk` #141005      -> `accent_ink`
#   `--block`#c96a5a        -> `block`
# `rgba(201,106,90,.16)` on B's button is `block` at the same alpha
# `accent_wash_2` carries for `accent`; there is no `block_wash` token, so it
# is derived from `c("block")` here rather than written as a hex.
#
# Metrics. `--pad:14px` is `role_px("bar_pad_x")` (`[14, 22]`) -- the same
# chrome inset every bar in the shell takes, and the only `ROLE` entry whose
# desktop figure is the artboard's. `--btnH:28px`, `--ctl:24px` and the card
# width have no `ROLE` counterpart at all (`btn_min_h` is `[0, 44]`, whose
# desktop `0` means "no constraint"), so they are named constants below and
# the tablet answer still comes from `btn_min_h`.
# ---------------------------------------------------------------------------

enum { MODAL_CONFIRM, MODAL_DESTRUCTIVE, MODAL_WARNINGS }

const MODAL_RADIUS := 10        ## The card. `border-radius:10px`.
const MODAL_INSET_RADIUS := 8   ## Stat block, list, buttons. `border-radius:8px`.
const MODAL_BTN_H := 28         ## `--btnH`.
const MODAL_CTL := 24           ## `--ctl`, the header's close box.
## The artboard cards are 300 px in a 300 px column; 340 is that plus the two
## 14 px pads and a little slack. It is a MINIMUM -- `wrap_controls` grows the
## card for a longer question rather than clipping it.
const MODAL_W := 340
const MODAL_STAT_H := 19        ## The stat rows' own `min-height:19px`.

## `--shadow:0 14px 34px rgba(0,0,0,.55)`. No palette token carries a shadow,
## so it is a constant here. Drawn onto the dialog's own `panel` stylebox,
## which is what `asset_library_window.gd::_build_slicer_modal()` already does
## for the one other floating card in this shell.
const MODAL_SHADOW_COLOR := Color(0, 0, 0, 0.55)
const MODAL_SHADOW_SIZE := 34
const MODAL_SHADOW_OFFSET := Vector2(0, 14)

## Meta on the list container: which dot tokens `modal_list_row()` has actually
## used. `modal_legend()` reads it, so the legend can only ever name classes
## that are really on screen -- the artboard's "wire the legend to the same
## source as the dots", made structural rather than a convention.
const MODAL_DOTS_META := "dcc_modal_dot_tokens"

## The floating card. Returns
## `{"dialog": AcceptDialog, "root": VBoxContainer, "body": VBoxContainer,
##   "title": Label, "close": Button, "variant": int}`.
##
## **One content child, deliberately.** `AcceptDialog` hands its FIRST content
## child the whole rect and lays every later sibling out on top of it; this
## shell has been bitten by that, so everything below hangs off `root`.
##
## Keyboard, and this is a capability the stock dialogs give away for free --
## losing it to a restyle would be a regression wearing conformance clothes:
##   * **Esc** cancels. `AcceptDialog` already routes `ui_cancel` to its own
##     `canceled` signal; `modal_choices()` connects that to the dismissal.
##   * **Enter** fires the last button, exactly as a stock
##     `ConfirmationDialog`'s focused OK does. Wired in `modal_choices()`
##     through `window_input`, because every button in this shell is
##     `FOCUS_NONE` and there is therefore no focused control to press.
##   * **Modality** is `AcceptDialog`'s own `exclusive`, re-asserted here after
##     the reparent so it cannot depend on a constructor default.
static func modal_card(host: Node, title: String, variant: int = MODAL_CONFIRM,
		width: int = MODAL_W) -> Dictionary:
	var dlg := AcceptDialog.new()
	dlg.title = title
	dlg.borderless = true
	dlg.wrap_controls = true
	dlg.get_ok_button().hide()
	dlg.add_theme_constant_override("buttons_min_height", 0)
	dlg.add_theme_constant_override("margin", 0)
	var card := DccTheme.outline("border", "panel")
	card.set_corner_radius_all(MODAL_RADIUS)
	card.shadow_color = MODAL_SHADOW_COLOR
	card.shadow_size = MODAL_SHADOW_SIZE
	card.shadow_offset = MODAL_SHADOW_OFFSET
	dlg.add_theme_stylebox_override("panel", card)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 0)
	root.custom_minimum_size.x = width
	dlg.add_child(root)

	var pad_x := DccTheme.role_px("bar_pad_x")
	var head_pad := pad(root, pad_x, 9, pad_x, 8)
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 8)
	head_pad.add_child(head)
	## Tracked caps in the faint ink -- and in `block` for the destructive
	## variant, which tints its title and (via `modal_inset()`) its inset's
	## left rule, and nothing else. The header divider below stays `line_soft`
	## in all three variants, which is what the artboard draws.
	var t := DccTheme.mono_label(title.to_upper(),
		"block" if variant == MODAL_DESTRUCTIVE else "text_faint",
		DccTheme.FS_MICRO, 2)
	t.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(t)
	var closer := Button.new()
	closer.text = "✕"
	closer.focus_mode = Control.FOCUS_NONE
	## **`--ctl:24px` is the artboard's desktop figure, and on a phone it is 24
	## *dp* against §13's 44 dp floor** -- the same trap `_modal_btn()` already
	## handles for the action row, on the one control in this card nobody gave
	## the same treatment. Nothing else was ever going to reach it either:
	## `modal_present()` deliberately does not call `DccShell.phone_fit()`,
	## because the card's buttons are floored at construction instead, and this
	## box was not one of them. Measured at **24 dp** on a 1080 x 2400 handset
	## by `_phoneproto_probe.gd` before this line existed -- the card was the
	## only "conformant" path in the census that the probe graded `fails:tap`.
	##
	## Phone only. `_modal_btn()` has a third, tablet density
	## (`role_px("btn_min_h")`); a square icon box is not a text button and
	## that figure has not been measured here, so the tablet keeps the
	## artboard's 24 rather than inheriting an untested number.
	var closer_px := DccTheme.PHONE_TAP_MIN if DccTheme.is_phone() else MODAL_CTL
	closer.custom_minimum_size = Vector2(closer_px, closer_px)
	closer.add_theme_font_override("font", DccTheme.mono(0))
	closer.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	closer.add_theme_color_override("font_color", DccTheme.c("text_faint"))
	closer.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	closer.add_theme_stylebox_override("normal", DccTheme.empty())
	closer.add_theme_stylebox_override("pressed", DccTheme.empty())
	closer.add_theme_stylebox_override("hover",
		DccTheme.flat(DccTheme.c("line_soft"), MODAL_INSET_RADIUS))
	head.add_child(closer)

	var rule := ColorRect.new()
	rule.color = DccTheme.c("line_soft")
	rule.custom_minimum_size.y = DccTheme.role_px("hairline")
	root.add_child(rule)

	var body_pad := pad(root, pad_x, 13, pad_x, 14)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 11)
	body_pad.add_child(body)

	if host != null:
		host.add_child(dlg)
	dlg.transient = true
	dlg.exclusive = true
	return {"dialog": dlg, "root": root, "body": body, "title": t,
		"close": closer, "variant": variant}

## The card's prose line -- `color:var(--body);line-height:1.5`, wrapped.
static func modal_prose(parent: Control, text: String) -> Label:
	var l := DccTheme.label(text, "text", DccTheme.FS_BODY)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	parent.add_child(l)
	return l

## The `--ins` block the stat rows sit in. `tinted` draws the destructive
## variant's `border-left:2px solid var(--block)`.
static func modal_inset(parent: Control, tinted: bool = false) -> VBoxContainer:
	var pc := PanelContainer.new()
	var sb := StyleBoxFlat.new()
	sb.bg_color = DccTheme.c("sunken")
	sb.set_corner_radius_all(MODAL_INSET_RADIUS)
	sb.content_margin_left = 11
	sb.content_margin_right = 11
	sb.content_margin_top = 9
	sb.content_margin_bottom = 9
	if tinted:
		sb.border_color = DccTheme.c("block")
		sb.border_width_left = 2
	pc.add_theme_stylebox_override("panel", sb)
	parent.add_child(pc)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	pc.add_child(col)
	return col

static func _modal_stat_row(parent: Control, key: String) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	row.custom_minimum_size.y = MODAL_STAT_H
	parent.add_child(row)
	var k := DccTheme.mono_label(key.to_upper(), "text_faint", DccTheme.FS_MICRO, 1)
	k.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(k)
	return row

## One `KEY ..... value` row inside a `modal_inset()`.
static func modal_stat(parent: Control, key: String, value: String) -> HBoxContainer:
	var row := _modal_stat_row(parent, key)
	var v := DccTheme.mono_label(value, "text_secondary", DccTheme.FS_TINY)
	v.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	row.add_child(v)
	return row

## The same row for a figure **the engine cannot supply**. Draws an em dash in
## the ghost ink and puts `why` on the row's tooltip, so the reason travels
## with the dash instead of living only in a source comment.
##
## This exists because the artboards were drawn to settle layout and their
## contents are illustrative: a plausible number in one of these slots is a
## worse outcome than a dash, and a factory makes the honest form the easy one.
static func modal_stat_absent(parent: Control, key: String, why: String) -> HBoxContainer:
	var row := _modal_stat_row(parent, key)
	var v := DccTheme.mono_label("—", "text_ghost", DccTheme.FS_TINY)
	v.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	v.mouse_filter = Control.MOUSE_FILTER_STOP
	row.add_child(v)
	row.tooltip_text = why
	v.tooltip_text = why
	return row

## The micro foot -- "the autosave is a separate slot", "source files on disk
## are not touched". A destructive confirm states what is NOT destroyed here.
static func modal_foot(parent: Control, text: String) -> Label:
	var l := DccTheme.mono_label(text, "text_ghost", DccTheme.FS_MICRO)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	parent.add_child(l)
	return l

## Variant C's list container. Rows go in through `modal_list_row()`, which is
## also what records the dot tokens `modal_legend()` names.
static func modal_list(parent: Control) -> VBoxContainer:
	var pc := PanelContainer.new()
	var sb := StyleBoxFlat.new()
	sb.bg_color = DccTheme.c("sunken")
	sb.set_corner_radius_all(MODAL_INSET_RADIUS)
	pc.add_theme_stylebox_override("panel", sb)
	parent.add_child(pc)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	col.set_meta(MODAL_DOTS_META, PackedStringArray())
	pc.add_child(col)
	return col

## One list row. `dot_token` empty draws the artboard's quiet "2 more" tail
## row, which carries no dot and therefore contributes nothing to the legend.
static func modal_list_row(list: VBoxContainer, dot_token: String,
		text: String) -> HBoxContainer:
	if list.get_child_count() > 0:
		var sep := ColorRect.new()
		sep.color = DccTheme.c("line_soft")
		sep.custom_minimum_size.y = DccTheme.role_px("hairline")
		list.add_child(sep)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 0)
	list.add_child(row)
	var m := MarginContainer.new()
	m.add_theme_constant_override("margin_left", 11)
	m.add_theme_constant_override("margin_right", 11)
	m.add_theme_constant_override("margin_top", 6)
	m.add_theme_constant_override("margin_bottom", 6)
	m.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(m)
	var inner := HBoxContainer.new()
	inner.add_theme_constant_override("separation", 9)
	m.add_child(inner)
	if dot_token != "":
		inner.add_child(DccTheme.mono_label("●", dot_token, DccTheme.FS_TINY))
		var seen: PackedStringArray = list.get_meta(MODAL_DOTS_META, PackedStringArray())
		if not seen.has(dot_token):
			seen.append(dot_token)
			list.set_meta(MODAL_DOTS_META, seen)
	var l := DccTheme.mono_label(text,
		"text_secondary" if dot_token != "" else "text_ghost",
		DccTheme.FS_TINY if dot_token != "" else DccTheme.FS_MICRO)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	inner.add_child(l)
	return row

## The legend under a `modal_list()`. `captions` maps a dot token to its
## caption; **only tokens the list actually drew are named**, read back off
## `MODAL_DOTS_META`, so a class the data never produced can never appear in
## the key. A caption with no matching dot is silently absent, which is the
## honest answer -- not a legend entry for a category that is not on screen.
static func modal_legend(parent: Control, list: VBoxContainer,
		captions: Dictionary) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 12)
	parent.add_child(row)
	var seen: PackedStringArray = list.get_meta(MODAL_DOTS_META, PackedStringArray())
	for token in seen:
		if not captions.has(token):
			continue
		var cell := HBoxContainer.new()
		cell.add_theme_constant_override("separation", 5)
		row.add_child(cell)
		cell.add_child(DccTheme.mono_label("●", token, DccTheme.FS_MICRO))
		cell.add_child(DccTheme.mono_label(String(captions[token]),
			"text_ghost", DccTheme.FS_MICRO))
	return row

## The action row. Right-aligned, `gap:8px`.
static func modal_actions(parent: Control) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.alignment = BoxContainer.ALIGNMENT_END
	parent.add_child(row)
	return row

const MODAL_KIND_META := "dcc_modal_button_kind"
## The card's Enter action, stamped on the dialog by `modal_choices()`.
const MODAL_DEFAULT_META := "dcc_modal_default"

static func _modal_btn(row: Control, text: String, on_press: Callable,
		kind: String, bg: Color, hover_bg: Color, ink: String,
		ink_hover: String) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_NONE
	b.set_meta(MODAL_KIND_META, kind)
	## Three densities, and the phone one is not optional: these buttons are
	## ordinary content children (unlike `AcceptDialog`'s internal button bar,
	## which is why `app.gd::_floor_prompt_buttons()` had to exist), so nothing
	## else floors them and a 28 px answer on a handset is the recorded
	## "desktop pixels on a phone" bug. `is_tablet()` is deliberately false on
	## a phone -- see its own comment -- so the two cases are separate.
	var h := MODAL_BTN_H
	if DccTheme.is_tablet():
		h = DccTheme.role_px("btn_min_h")
	elif DccTheme.is_phone():
		h = DccTheme.PHONE_TAP_MIN
	b.custom_minimum_size.y = h
	b.add_theme_font_override("font", DccTheme.mono(0))
	b.add_theme_font_size_override("font_size",
		DccTheme.role_px("fs_readout") if DccTheme.is_tablet() else DccTheme.FS_TINY)
	b.add_theme_color_override("font_color", DccTheme.c(ink))
	b.add_theme_color_override("font_hover_color", DccTheme.c(ink_hover))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	var pad_x := DccTheme.role_px("btn_pad_x") if DccTheme.is_tablet() else 14
	var rest := DccTheme.flat(bg, MODAL_INSET_RADIUS)
	rest.content_margin_left = pad_x
	rest.content_margin_right = pad_x
	b.add_theme_stylebox_override("normal", rest)
	b.add_theme_stylebox_override("disabled", rest)
	var hover := DccTheme.flat(hover_bg, MODAL_INSET_RADIUS)
	hover.content_margin_left = pad_x
	hover.content_margin_right = pad_x
	b.add_theme_stylebox_override("hover", hover)
	b.add_theme_stylebox_override("pressed", hover)
	b.pressed.connect(on_press)
	row.add_child(b)
	return b

## `.btn2` -- the quiet dismissal. `background:var(--ins);color:var(--sec)`.
static func modal_quiet(row: Control, text: String, on_press: Callable) -> Button:
	return _modal_btn(row, text, on_press, "quiet", DccTheme.c("sunken"),
		DccTheme.c("raised"), "text_secondary", "text_bright")

## The destructive answer **beside a safe one** -- `.btn2`'s quiet box with the
## block ink. Text-only in the sense that matters: it never takes a fill of its
## own, never the accent, and never the rightmost slot.
static func modal_text_destructive(row: Control, text: String, on_press: Callable) -> Button:
	return _modal_btn(row, text, on_press, "destructive_text",
		DccTheme.c("sunken"), DccTheme.c("raised"), "block", "block")

## The destructive answer when it is the ONLY answer (variant B). Block wash at
## `accent_wash_2`'s alpha, block ink -- never the accent fill.
static func modal_destructive(row: Control, text: String, on_press: Callable) -> Button:
	var wash := DccTheme.c("block")
	wash.a = DccTheme.c("accent_wash_2").a
	var lit := DccTheme.c("block")
	lit.a = min(1.0, wash.a * 1.7)
	return _modal_btn(row, text, on_press, "destructive_filled", wash, lit,
		"block", "text_bright")

## `.btn` -- the safe, filled answer. `background:var(--acc);color:var(--accInk)`.
static func modal_safe(row: Control, text: String, on_press: Callable) -> Button:
	return _modal_btn(row, text, on_press, "safe", DccTheme.c("accent"),
		DccTheme.c("accent_hover"), "accent_ink", "accent_ink")

## Builds the action row for a `modal_card()` and **enforces the artboard's
## ordering**, so no call site can put the destructive answer last beside a
## safe one or draw it in the accent fill.
##
## `spec` is read with `has()` throughout -- an absent key is an absent button,
## never a button with an empty label:
##   `cancel`      String -- the quiet dismissal, always first.
##   `destructive` {"text": String, "on": Callable}
##   `safe`        {"text": String, "on": Callable}
##
## Returns `{"row", "cancel"?, "destructive"?, "safe"?, "default"?}`; the keys
## for buttons that were not asked for are omitted.
##
## Keyboard, restored rather than lost:
##   * Esc -> `canceled` -> the card closes, and the caller's `on_cancel` runs.
##   * Enter -> the LAST button, which is what a stock `ConfirmationDialog`'s
##     focused OK does. Fired from `window_input` because every button here is
##     `FOCUS_NONE`, so there is no focused control for `ui_accept` to reach.
##   * The header close box and the window's own close both take the Esc path.
static func modal_choices(card: Dictionary, spec: Dictionary) -> Dictionary:
	var dlg: AcceptDialog = card["dialog"]
	var row := modal_actions(card["body"])
	var out := {"row": row}
	var default_action := Callable()

	var has_safe: bool = spec.has("safe")
	var has_destructive: bool = spec.has("destructive")
	var on_cancel: Callable = spec["on_cancel"] if spec.has("on_cancel") else Callable()

	var dismiss := func():
		if is_instance_valid(dlg) and not dlg.is_queued_for_deletion():
			dlg.hide()
			dlg.queue_free()

	var cancel_action := func():
		var was_live: bool = is_instance_valid(dlg) and not dlg.is_queued_for_deletion()
		dismiss.call()
		if was_live and on_cancel.is_valid():
			on_cancel.call()

	if spec.has("cancel"):
		out["cancel"] = modal_quiet(row, String(spec["cancel"]), cancel_action)

	if has_destructive:
		var d: Dictionary = spec["destructive"]
		var d_on: Callable = d["on"]
		var run_destructive := func():
			dismiss.call()
			d_on.call()
		if has_safe:
			## Text-only, and NOT the rightmost -- the safe answer follows it.
			out["destructive"] = modal_text_destructive(row, String(d["text"]),
				run_destructive)
		else:
			out["destructive"] = modal_destructive(row, String(d["text"]),
				run_destructive)
			default_action = run_destructive

	if has_safe:
		var s: Dictionary = spec["safe"]
		var s_on: Callable = s["on"]
		var run_safe := func():
			dismiss.call()
			s_on.call()
		out["safe"] = modal_safe(row, String(s["text"]), run_safe)
		default_action = run_safe

	if default_action.is_valid():
		out["default"] = default_action
		## Also stamped on the dialog, because a caller that puts a `LineEdit`
		## in the body needs to route `text_submitted` to the same action: a
		## focused `LineEdit` consumes Enter before `window_input` sees it, so
		## the card-level handler below cannot be the field's route too.
		dlg.set_meta(MODAL_DEFAULT_META, default_action)

	## Esc. `AcceptDialog` routes `ui_cancel` here on its own; connecting it is
	## what makes the restyle keep the capability the stock dialog had.
	dlg.canceled.connect(cancel_action)
	dlg.close_requested.connect(cancel_action)
	(card["close"] as Button).pressed.connect(cancel_action)

	## Enter.
	if default_action.is_valid():
		var fire := default_action
		dlg.window_input.connect(func(ev: InputEvent):
			if ev.is_action_pressed("ui_accept", false, true):
				dlg.set_input_as_handled()
				fire.call())
	return out

## Opens a `modal_card()`, phone-shaped where the host is a phone. Mirrors the
## two-line `phone_present()` / `popup_centered()` pair every window in this
## shell already uses.
static func modal_present(card: Dictionary, host) -> void:
	var dlg: AcceptDialog = card["dialog"]
	if DccTheme.is_phone():
		phone_window(dlg, host)
	if not phone_present(dlg, host):
		dlg.popup_centered()

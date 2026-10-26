extends ScrollContainer
class_name QuickBarPhoneRow

## The pinned-chip row at the top of the phone's MAP sheet (`CUSTOM_QUICK_BAR_SCOPE.md` s.3.2, milestone CQ-6,
## Ruling DH Q9). One chip per entry of [`QuickBarPhone.entries`] -- the Assistant chip first when its rule
## shows it, then the user's pins in order -- and `All...` last, which opens the full list.
##
## ## Size: 48 dp chips, a hard floor of 44 dp on BOTH axes
##
## Every chip is `QuickBarPhone.CHIP_DP` (48) dp tall and at least that wide, passed through `DccShell._ptap`,
## which raises anything below `DccTheme.PHONE_TAP_MIN` (44) dp to it. The chip's width follows its label (a
## title is cut to 16 characters in code, never with `clip_text`, which collapses a Label's minimum to 1;
## the full title is in the entry and in the `All...` list).
##
## ## It scrolls sideways and can never widen the sheet
##
## A `ScrollContainer` with `horizontal_scroll_mode = SHOW_NEVER` takes its width from its parent and folds
## nothing of its content's width into its own minimum, so 24 chips cannot push the sheet (or the shell) past
## the screen: they scroll, and `All...` lists them all. The VERTICAL axis is `DISABLED` on purpose: that is
## the one axis whose content minimum IS folded into the container's, which is what gives the row its height
## (`MISTAKES.md`, "Read a layout that overflows the screen": walk the row for the minimum WIDTH, which is
## what `_quickbar_phone_probe.gd` does). `SHOW_NEVER`, not `AUTO`, because a visible scrollbar adds its own
## height to the minimum and the row's height is part of the MAP sheet's detent budget.
##
## ## A chip that cannot run is drawn dim, not disabled
##
## A disabled `Button` swallows its tap, and on a phone there is no tooltip and no status bar to say why. So
## an unavailable chip keeps its enabled state but wears the disabled look, and a tap toasts the entry's
## reason through [`QuickBarPhone.press`].

var _shell
var _ctl: QuickBarPhone
var _box: HBoxContainer
var _chips: Array[Button] = []
## The entries last drawn, in order (for tests).
var entries: Array = []
## The `All...` chip (for tests).
var all_chip: Button

## Builds the empty row for `shell` (the app) and its controller `ctl`. Call once, before `set_entries`.
func setup(shell, ctl: QuickBarPhone) -> void:
	_shell = shell
	_ctl = ctl
	name = "QuickBarPhoneRow"
	horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_NEVER
	vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_theme_stylebox_override("panel", DccTheme.empty())
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", shell.phone_px(QuickBarPhone.ROW_SIDE_DP))
	pad.add_theme_constant_override("margin_right", shell.phone_px(QuickBarPhone.ROW_SIDE_DP))
	pad.add_theme_constant_override("margin_top", shell.phone_px(QuickBarPhone.ROW_TOP_DP))
	pad.add_theme_constant_override("margin_bottom", shell.phone_px(QuickBarPhone.ROW_BOTTOM_DP))
	add_child(pad)
	_box = HBoxContainer.new()
	_box.add_theme_constant_override("separation", shell.phone_px(QuickBarPhone.ROW_GAP_DP))
	pad.add_child(_box)

## Redraws the chips for `es` (the result of [`QuickBarPhone.entries`]). The previous chips are freed.
func set_entries(es: Array) -> void:
	entries = es
	for c in _chips:
		_box.remove_child(c)
		c.queue_free()
	_chips.clear()
	for e in es:
		_chips.append(_chip(String(e["label"]), func(): QuickBarPhone.press(_shell, e, _chips_near()), e))
	all_chip = _chip(QuickBarPhone.ALL_LABEL, func(): _ctl.open_list(), {"live": true, "reason": ""})
	all_chip.name = "QuickBarPhoneAll"
	_chips.append(all_chip)

## The chips in order, `All...` last (for tests).
func chips() -> Array[Button]:
	return _chips

## One chip. `e` carries `live` and `reason`; an entry that cannot run is drawn in the disabled look.
func _chip(text: String, on_press: Callable, e: Dictionary) -> Button:
	var b := DccWidgets.chip(_box, text, on_press, false, _shell.phone_px(10), _shell.phone_px(3))
	b.add_theme_font_size_override("font_size", _shell.phone_font(10))
	var tap := float(_shell.phone_tap(QuickBarPhone.CHIP_DP))
	b.custom_minimum_size = Vector2(tap, tap)
	b.tooltip_text = String(e.get("reason", "")) if not bool(e.get("live", true)) else text
	b.set_meta("live", bool(e.get("live", true)))
	b.set_meta("reason", String(e.get("reason", "")))
	if not bool(e.get("live", true)):
		## The disabled look on an enabled button (module header): the same styleboxes and ink the
		## shell's own disabled chip uses, swapped into every state a tap can reach.
		var dim: StyleBox = b.get_theme_stylebox("disabled")
		for state in ["normal", "pressed", "hover"]:
			b.add_theme_stylebox_override(state, dim)
		b.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
		b.add_theme_color_override("font_hover_color", DccTheme.c("text_ghost"))
	return b

## Where a toast should appear: over the row.
func _chips_near() -> Control:
	return self

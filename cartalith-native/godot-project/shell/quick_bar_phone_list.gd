extends AcceptDialog
class_name QuickBarPhoneList

## The phone's `All...` sheet: every pinned command, one full-width row each, plus a `Customise...` row
## (`CUSTOM_QUICK_BAR_SCOPE.md` s.3.2, milestone CQ-6). The chip row scrolls sideways and can hold 24 pins; this
## is where all 24 are readable at once, and where a pin that cannot run shows WHY in the row itself (a
## tooltip is invisible on a phone).
##
## ## Built like the phone's other full-height windows
##
## `DccWidgets.phone_window` / `phone_present` make an `AcceptDialog` fill the screen above the gesture inset,
## borderless, scaled by `phone_scale()`; its own button (`Close`) is the way out. The window therefore lays
## out in **dp** (the content scale factor does the physical scaling), so every size below is a plain dp
## number, not a `_pscale` result (that would scale twice -- `MISTAKES.md`, "Judge a measurement against a
## floor": name the density beside every number).
##
## ## Rows
##
## 52 dp tall (the phone spec's list-row height; the scope's 46 dp is the floor it clears), full width, left
## aligned, the label on the first line and, on a second, the menu the command lives in or the reason it
## cannot run. A tap runs the entry and closes the sheet, so the command's own window is not left under it;
## a row that cannot run toasts its reason and stays open. Only the list is a `ScrollContainer`, with the
## horizontal axis `DISABLED` and every row a width-bounded Button, so nothing here widens the window.

## One list row's height in dp (see "Rows").
const ROW_DP := 52.0

var _app
var _body: VBoxContainer
var _scroll: ScrollContainer
var _count: Label
## The rows last drawn, in order (for tests): the entry rows, then the Customise row.
var rows: Array[Button] = []

## Builds the empty sheet. Call once, after `add_child`, and only when first needed (`QuickBarPhone.open_list`).
func setup(app) -> void:
	_app = app
	name = "QuickBarPhoneList"
	title = "Quick bar"
	DccWidgets.phone_window(self, app)
	get_ok_button().text = "Close"
	var pad := MarginContainer.new()
	for side in ["left", "right"]:
		pad.add_theme_constant_override("margin_" + side, 14)
	pad.add_theme_constant_override("margin_top", 10)
	pad.add_theme_constant_override("margin_bottom", 6)
	add_child(pad)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 6)
	pad.add_child(col)
	col.add_child(DccTheme.mono_label("QUICK BAR", "accent", DccTheme.FS_HEADER, 2, true))
	_count = DccTheme.label("", "text_secondary", DccTheme.FS_SMALL)
	_count.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_count.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## An autowrap label measures its minimum height at its current width, 1 px before the first layout;
	## a width floor stops the window growing tall (`quick_bar_window.gd`'s `_builtin_lbl`).
	_count.custom_minimum_size = Vector2(240, 0)
	col.add_child(_count)
	_scroll = ScrollContainer.new()
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col.add_child(_scroll)
	_body = VBoxContainer.new()
	_body.add_theme_constant_override("separation", 4)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_scroll.add_child(_body)

## Shows the sheet on `es` ([`QuickBarPhone.entries`]).
func open(es: Array) -> void:
	show_entries(es)
	if not DccWidgets.phone_present(self, _app):
		popup_centered(Vector2i(420, 640))

## Redraws the rows for `es` without moving the window (used while the sheet is open and a pin changed).
func show_entries(es: Array) -> void:
	for c in _body.get_children():
		_body.remove_child(c)
		c.queue_free()
	rows.clear()
	var pins := 0
	for e in es:
		if String(e["kind"]) == "pin":
			pins += 1
	_count.text = "%d / %d pinned. Tap a command to run it." % [pins, DccSettings.QUICK_BAR_MAX_BAR]
	for e in es:
		rows.append(_row(String(e["title"]), _sub_for(e), bool(e["live"]), func(): _tap(e)))
	rows.append(_row("Customise…", "Pin, unpin and reorder commands", true, _customise))

## The second line of an entry's row: the menu it lives in, or the reason it cannot run.
static func _sub_for(e: Dictionary) -> String:
	if not bool(e["live"]):
		return String(e["reason"])
	if String(e["kind"]) == "assistant":
		return "Open the assistant panel"
	return "%s menu" % String(e["group"]) if String(e["group"]) != "" else ""

## One full-width row. Dim (the disabled look) when `live` is false, but still tappable so the tap can say why.
func _row(title_text: String, sub: String, live: bool, on_press: Callable) -> Button:
	var b := Button.new()
	b.flat = false
	b.focus_mode = Control.FOCUS_NONE
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.text = title_text if sub == "" else "%s\n%s" % [title_text, sub]
	b.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.custom_minimum_size = Vector2(0, ROW_DP)
	b.add_theme_font_override("font", DccTheme.mono(0))
	b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	b.add_theme_color_override("font_color", DccTheme.c("text") if live else DccTheme.c("text_ghost"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright") if live else DccTheme.c("text_ghost"))
	var rest := DccWidgets.box("line_soft" if live else "line", "", 12, 6)
	for state in ["normal", "pressed", "disabled", "focus"]:
		b.add_theme_stylebox_override(state, rest)
	b.add_theme_stylebox_override("hover", DccWidgets.box("border", "line_soft", 12, 6))
	b.set_meta("live", live)
	b.pressed.connect(on_press)
	_body.add_child(b)
	return b

## A tap on an entry's row: run it and close, or say why not and stay.
func _tap(e: Dictionary) -> void:
	if not bool(e["live"]):
		QuickBarPhone.press(_app, e, _scroll)
		return
	hide()
	QuickBarPhone.press(_app, e, null)

func _customise() -> void:
	hide()
	if _app != null and _app.has_method("open_quick_bar_window"):
		_app.open_quick_bar_window()

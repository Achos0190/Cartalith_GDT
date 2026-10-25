extends AcceptDialog
class_name QuickBarWindow

## **`Preferences ▸ Quick bar…`** -- the Customise window for the user's pinned commands
## (`CUSTOM_QUICK_BAR_SCOPE.md` s.3.5, milestone CQ-3, Ruling DH).
##
## ## What it is
##
## Two columns. Left: the searchable catalogue of every command the live menus offer
## (`CommandIndex`, so it is built from the app and never hand-kept, scope risk 5). Right: the
## ordered pins with up, down and remove buttons, a count against the cap, and a confirmed
## "Reset to empty". It reads and writes only through `DccSettings` (`quick_bar_value`,
## `quick_bar_add`, `quick_bar_remove`, `quick_bar_move`, `set_quick_bar_value`), whose
## allow-list, caps and refusal sentences are CQ-1's; this file adds none of its own, so the cap
## and the order logic exist once. After every change it emits `pins_changed`, which the shell
## connects to `DccQuickBar.refresh()`, so the cluster follows within the same frame.
##
## ## No drag, and no pointer needed (scope s.3.5)
##
## Reorder and remove are buttons AND keys: Enter (or a double click) on a catalogue row adds it;
## on a pin, Delete removes and Alt+Up / Alt+Down move. A drag is not offered at all in this
## milestone: a drag-only reorder is unusable on a phone and invisible to a keyboard, and the
## buttons already cover both. Every button here is focusable (`DccWidgets.chip` is not: it sets
## `FOCUS_NONE`), so Tab reaches search, the unpinnable toggle, the catalogue, Add, the pins,
## the three pin buttons, Reset and Close in reading order.
##
## ## Honesty about what cannot be pinned (scope s.3.3, Ruling DH Q8/Q10)
##
## An unpinnable row is shown **disabled with its `why_not` written in the row itself** (not only
## in a tooltip, which a keyboard and a touch screen never see). They are hidden by default
## behind "Show commands that cannot be pinned" because they outnumber the pinnable ones nine to
## one; the toggle makes each one's reason readable. The ~264 rows with an EMPTY group are the
## preset menus of embedded `ColorPicker`s (`CommandIndex` marks them unpinnable and the scope's
## s.11 records them as search noise): they are left out of this catalogue altogether and the
## footer counts them, so the list is not padded with colour swatches. A deliberate divergence
## from "every unpinnable row appears", disclosed here and in the scope.
##
## ## What this milestone does not build (CQ-4 and later)
##
## The scope draws three tabs (Bar, Ring dock, Ring slots). Only **Bar** exists here, as a plain
## page rather than a one-tab `TabContainer`: the other two edit lists that nothing draws until
## CQ-4 puts the dock and the shared sub-rings on the ring, and a control that edits an invisible
## list is the dead control this project forbids. CQ-4 adds the tabs with their consumers. The
## phone gets no row for this window (`menus.gd`) until CQ-6 gives it a cluster to edit.

## A pin was added, removed, moved or the list was reset.
signal pins_changed

## Longest reason shown inline before it is cut (the full text is the row's tooltip).
const MAX_REASON_CHARS := 90

var app
## The command index of the last `_rebuild_all()`; rebuilt on open and after every change.
var _idx: CommandIndex
var _search: LineEdit
var _show_unpinnable: CheckBox
var _catalogue: ItemList
var _pins_list: ItemList
var _add_btn: Button
var _up_btn: Button
var _down_btn: Button
var _remove_btn: Button
var _reset_btn: Button
var _count: Label
var _foot: Label
var _msg: Label
var _detail: Label
var _reset_dlg: ConfirmationDialog
## Row dictionaries parallel to `_catalogue`'s items (`{}` for a placeholder line).
var _cat_rows: Array = []
## Pin keys parallel to `_pins_list`'s items.
var _pin_keys: Array = []
## Rows with an empty group left out of the catalogue on the last render.
var hidden_picker_rows := 0

# -- pure logic (static so a probe and a mutation run can reach it) ------------------------

## The rows the catalogue shows from `rows`: never a row with an empty group (an embedded
## ColorPicker's preset menu, see the module header), and a row that cannot be pinned only when
## `show_unpinnable` is on. Order is preserved.
static func filter_rows(rows: Array, show_unpinnable: bool) -> Array:
	var out: Array = []
	for r in rows:
		if String(r.get("group", "")) == "":
			continue
		if not show_unpinnable and not bool(r.get("pinnable", false)):
			continue
		out.append(r)
	return out

## How many of `rows` the catalogue leaves out for having no group.
static func count_picker_rows(rows: Array) -> int:
	var n := 0
	for r in rows:
		if String(r.get("group", "")) == "":
			n += 1
	return n

## The text of one catalogue row: `Group › Title`, then `· on the bar` when pinned, or the reason
## it cannot be pinned (cut to `MAX_REASON_CHARS`; the whole reason is the tooltip).
static func row_text(row: Dictionary, is_pinned: bool) -> String:
	var t := "%s  ›  %s" % [String(row["group"]), String(row["title"])]
	if not bool(row.get("pinnable", false)):
		var why := String(row.get("why_not", ""))
		if why.length() > MAX_REASON_CHARS:
			why = why.substr(0, MAX_REASON_CHARS - 1) + "…"
		return "%s  --  %s" % [t, why]
	return t + ("  ·  on the bar" if is_pinned else "")

## Why `row` cannot be added right now, or "" when it can. Only the row's OWN state is judged
## here (a placeholder, or unpinnable); "already pinned" and "the bar is full" are
## `DccSettings.quick_bar_add`'s sentences and are shown verbatim, so they exist once.
static func add_refusal(row: Dictionary) -> String:
	if row.is_empty() or not row.has("pin_key"):
		return "Select a command first."
	if not bool(row.get("pinnable", false)):
		return "Cannot be pinned: " + String(row.get("why_not", "this command cannot be run from a chip."))
	return ""

## The label of one pin in the right-hand list: the live title, or the title it had when pinned
## plus the reason it is not live (scope s.0 rule 1: a pin whose command is gone is shown, never
## dropped). `row` is the live index row or `{}`.
static func pin_text(row: Dictionary, stored_title: String, pin_key: String) -> String:
	if row.is_empty():
		return "%s  --  not in this build" % (stored_title if stored_title != "" else pin_key)
	if not bool(row.get("pinnable", false)):
		return "%s  --  unavailable: %s" % [String(row["title"]), String(row.get("why_not", ""))]
	return String(row["title"])

# -- construction --------------------------------------------------------------------------

## Builds the window once; `open()` fills it. Call after `add_child`.
func setup(a) -> void:
	app = a
	title = "Quick bar"
	size = Vector2i(820, 580)
	min_size = Vector2i(660, 440)
	DccWidgets.phone_window(self, a)
	get_ok_button().text = "Close"
	var tablet := DccTheme.is_tablet()
	## Inset the content so it does not touch the dialog's own edge (the first screenshot showed it flush).
	var pad := _inset()
	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 8)
	pad.add_child(root)
	var lead := DccTheme.label("Pin commands to the bar beside the tool palette. Pins belong to this device and are never saved in a world, an export or a project file.", "text_secondary", DccTheme.FS_SMALL)
	lead.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	lead.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	root.add_child(lead)
	var cols := HBoxContainer.new()
	cols.add_theme_constant_override("separation", 14)
	cols.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_child(cols)

	var left := VBoxContainer.new()
	left.add_theme_constant_override("separation", 6)
	left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left.size_flags_vertical = Control.SIZE_EXPAND_FILL
	left.size_flags_stretch_ratio = 1.25
	cols.add_child(left)
	left.add_child(DccTheme.mono_label("COMMANDS", "text_dim", DccTheme.FS_HEADER, 1))
	_search = LineEdit.new()
	_search.name = "Search"
	_search.placeholder_text = "Search commands"
	_search.clear_button_enabled = true
	_search.text_changed.connect(func(_t: String): _render_catalogue())
	_search.custom_minimum_size.y = DccTheme.role_px("btn_min_h") if tablet else 26
	left.add_child(_search)
	_show_unpinnable = CheckBox.new()
	_show_unpinnable.name = "ShowUnpinnable"
	_show_unpinnable.text = "Show commands that cannot be pinned (with the reason)"
	_show_unpinnable.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	_show_unpinnable.toggled.connect(func(_on: bool): _render_catalogue())
	left.add_child(_show_unpinnable)
	_catalogue = _make_list("Catalogue")
	_catalogue.item_selected.connect(func(_i: int): _on_catalogue_selected())
	_catalogue.item_activated.connect(func(_i: int): _add_selected())
	left.add_child(_catalogue)
	_detail = DccTheme.label("", "text_ghost", DccTheme.FS_MICRO)
	_detail.name = "Detail"
	_detail.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_detail.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left.add_child(_detail)
	_add_btn = _button(left, "＋ Add to bar", _add_selected)
	_add_btn.name = "Add"
	_foot = DccTheme.label("", "text_ghost", DccTheme.FS_MICRO)
	_foot.name = "Foot"
	_foot.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_foot.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left.add_child(_foot)

	var right := VBoxContainer.new()
	right.add_theme_constant_override("separation", 6)
	right.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right.size_flags_vertical = Control.SIZE_EXPAND_FILL
	cols.add_child(right)
	_count = DccTheme.mono_label("", "text_dim", DccTheme.FS_HEADER, 1)
	_count.name = "Count"
	right.add_child(_count)
	_pins_list = _make_list("Pins")
	_pins_list.item_selected.connect(func(_i: int): _sync_buttons())
	_pins_list.gui_input.connect(_pins_key)
	right.add_child(_pins_list)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	right.add_child(row)
	_up_btn = _button(row, "▲ Up", func(): _move(-1))
	_up_btn.name = "Up"
	_down_btn = _button(row, "▼ Down", func(): _move(1))
	_down_btn.name = "Down"
	_remove_btn = _button(row, "✕ Remove", _remove_selected)
	_remove_btn.name = "Remove"
	_reset_btn = _button(right, "Reset to empty", _ask_reset)
	_reset_btn.name = "Reset"
	_msg = DccTheme.label("", "warn", DccTheme.FS_SMALL)
	_msg.name = "Message"
	_msg.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_msg.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	root.add_child(_msg)

	_reset_dlg = ConfirmationDialog.new()
	_reset_dlg.title = "Reset the quick bar"
	_reset_dlg.ok_button_text = "Remove all pins"
	## A width, so the sentence wraps inside the dialog (the shell's own `DccWidgets.confirm` does the same).
	_reset_dlg.min_size = Vector2i(420, 0)
	_reset_dlg.confirmed.connect(_do_reset)
	add_child(_reset_dlg)

## The content inset, added to the dialog itself (`DccWidgets.pad` needs a `Control` parent and a
## dialog is a `Window`, so this builds the same four-margin container directly).
func _inset() -> MarginContainer:
	var m := MarginContainer.new()
	for side in ["left", "right"]:
		m.add_theme_constant_override("margin_" + side, 14)
	for side in ["top", "bottom"]:
		m.add_theme_constant_override("margin_" + side, 10)
	add_child(m)
	return m

## A themed, focusable list. `ItemList` rather than a column of buttons so that arrow keys move
## through hundreds of rows with ONE Tab stop, and Enter activates.
func _make_list(node_name: String) -> ItemList:
	var l := ItemList.new()
	l.name = node_name
	l.select_mode = ItemList.SELECT_SINGLE
	l.allow_reselect = true
	l.size_flags_vertical = Control.SIZE_EXPAND_FILL
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.custom_minimum_size = Vector2(200, 160)
	l.auto_height = false
	l.focus_mode = Control.FOCUS_ALL
	l.add_theme_font_override("font", DccTheme.mono(0))
	l.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	l.add_theme_color_override("font_color", DccTheme.c("text"))
	l.add_theme_color_override("font_hovered_color", DccTheme.c("text_bright"))
	l.add_theme_color_override("font_selected_color", DccTheme.c("text_bright"))
	l.add_theme_color_override("guide_color", DccTheme.c("line_soft"))
	l.add_theme_stylebox_override("panel", DccTheme.field_box("normal", 4, 4))
	l.add_theme_stylebox_override("focus", DccTheme.empty())
	l.add_theme_stylebox_override("selected", DccTheme.active_row())
	l.add_theme_stylebox_override("selected_focus", DccTheme.active_row())
	l.add_theme_stylebox_override("hovered", DccTheme.flat(DccTheme.c("line_soft")))
	l.add_theme_constant_override("v_separation", 8 if DccTheme.is_tablet() else 3)
	return l

## A focusable text button, floored to the touch size on a tablet.
func _button(parent: Control, text: String, on_press: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_ALL
	b.add_theme_font_override("font", DccTheme.mono(0))
	b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	b.add_theme_color_override("font_color", DccTheme.c("text"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	var rest := DccWidgets.box("border", "", 10, 4)
	b.add_theme_stylebox_override("normal", rest)
	b.add_theme_stylebox_override("pressed", rest)
	b.add_theme_stylebox_override("focus", DccWidgets.box("accent", "", 10, 4))
	b.add_theme_stylebox_override("disabled", DccWidgets.box("line_soft", "", 10, 4))
	b.add_theme_stylebox_override("hover", DccWidgets.box("border", "line_soft", 10, 4))
	if DccTheme.is_tablet():
		b.custom_minimum_size = Vector2(DccTheme.role_px("btn_min_h"), DccTheme.role_px("btn_min_h"))
	b.pressed.connect(on_press)
	parent.add_child(b)
	return b

# -- opening and rendering -----------------------------------------------------------------

## Opens the window on the stored state, with the first control focused.
func open() -> void:
	## A fresh look each time: the previous query and the unpinnable toggle belong to the last visit,
	## and a stale filter makes a catalogue that "has nothing in it" (found by the probe).
	_msg.text = ""
	_search.text = ""
	_show_unpinnable.set_pressed_no_signal(false)
	_rebuild_all()
	popup_centered()
	_search.grab_focus.call_deferred()

## Rebuilds the index and both lists. The index is rebuilt (not cached) so a pin's "not in this
## build" state and every row's availability are those of the moment, not of the last open.
func _rebuild_all() -> void:
	_idx = CommandIndex.new()
	_idx.build(app, app.bridge)
	_render_catalogue()
	_render_pins()

func _pinned_keys() -> Dictionary:
	var out := {}
	for p in DccSettings.quick_bar_value("bar"):
		out[String(p["k"])] = true
	return out

## Fills the catalogue from the search box and the unpinnable toggle, keeping the selected
## command selected when it is still listed.
func _render_catalogue() -> void:
	var keep := ""
	var sel := _catalogue.get_selected_items()
	if sel.size() > 0 and sel[0] < _cat_rows.size():
		keep = String(_cat_rows[sel[0]].get("pin_key", ""))
	var all_rows: Array = _idx.search(_search.text)
	hidden_picker_rows = count_picker_rows(_idx.all())
	var rows := filter_rows(all_rows, _show_unpinnable.button_pressed)
	var pinned := _pinned_keys()
	_catalogue.clear()
	_cat_rows.clear()
	var reselect := -1
	for r in rows:
		var is_pinned := pinned.has(String(r["pin_key"]))
		var i := _catalogue.add_item(row_text(r, is_pinned))
		_cat_rows.append(r)
		var pinnable := bool(r.get("pinnable", false))
		_catalogue.set_item_disabled(i, not pinnable)
		_catalogue.set_item_tooltip(i, String(r.get("blurb", "")) if pinnable else String(r.get("why_not", "")))
		if not pinnable:
			_catalogue.set_item_custom_fg_color(i, DccTheme.c("text_ghost"))
		if keep != "" and String(r["pin_key"]) == keep:
			reselect = i
	if rows.is_empty():
		var i := _catalogue.add_item("No command matches \"%s\"." % _search.text if _search.text.strip_edges() != "" else "Nothing to list.")
		_catalogue.set_item_disabled(i, true)
		_cat_rows.append({})
	if reselect >= 0:
		_catalogue.select(reselect)
	var pinnable_n := 0
	for r in rows:
		if bool(r.get("pinnable", false)):
			pinnable_n += 1
	_foot.text = "%d listed, %d pinnable. %d colour-picker preset rows are left out." % [rows.size(), pinnable_n, hidden_picker_rows]
	_on_catalogue_selected()

## Fills the pins list from the store, selecting `select_key` (else keeping the selection).
func _render_pins(select_key: String = "") -> void:
	var keep := select_key
	if keep == "":
		var sel := _pins_list.get_selected_items()
		if sel.size() > 0 and sel[0] < _pin_keys.size():
			keep = String(_pin_keys[sel[0]])
	var pins: Array = DccSettings.quick_bar_value("bar")
	_pins_list.clear()
	_pin_keys.clear()
	var at := -1
	for p in pins:
		var k := String(p["k"])
		var row := _idx.find_pin(k)
		var i := _pins_list.add_item(pin_text(row, String(p["t"]), k))
		_pin_keys.append(k)
		if not row.is_empty() and bool(row.get("pinnable", false)):
			_pins_list.set_item_tooltip(i, String(row.get("blurb", "")))
		else:
			_pins_list.set_item_tooltip(i, "This pin is kept. Remove it with Remove or Delete.")
		if k == keep:
			at = i
	if at >= 0:
		_pins_list.select(at)
	_count.text = "ON THE BAR  %d / %d" % [pins.size(), DccSettings.QUICK_BAR_MAX_BAR]
	_sync_buttons()

## The detail line under the catalogue and the Add button's state follow the selected row.
func _on_catalogue_selected() -> void:
	var row := _selected_row()
	if row.is_empty():
		_detail.text = ""
	else:
		_detail.text = String(row.get("blurb", ""))
	_sync_buttons()

func _selected_row() -> Dictionary:
	var sel := _catalogue.get_selected_items()
	if sel.size() == 0 or sel[0] >= _cat_rows.size():
		return {}
	return _cat_rows[sel[0]]

func _selected_pin_index() -> int:
	var sel := _pins_list.get_selected_items()
	return -1 if sel.size() == 0 else int(sel[0])

## Enables each button exactly when it can do something. Add stays enabled on a pinnable row
## even when the bar is full or the row is pinned: pressing it then SAYS why it did not add
## (scope s.3.5: "refused with the reason, not silently dropped"), which a greyed button cannot.
func _sync_buttons() -> void:
	var row := _selected_row()
	_add_btn.disabled = row.is_empty() or not bool(row.get("pinnable", false))
	var i := _selected_pin_index()
	var n := _pin_keys.size()
	_up_btn.disabled = i <= 0
	_down_btn.disabled = i < 0 or i >= n - 1
	_remove_btn.disabled = i < 0
	_reset_btn.disabled = n == 0

# -- actions -------------------------------------------------------------------------------

func _say(text: String, token: String = "warn") -> void:
	_msg.text = text
	_msg.add_theme_color_override("font_color", DccTheme.c(token))

## Adds the selected catalogue row. Answers "" when added, else the sentence shown.
func _add_selected() -> String:
	var row := _selected_row()
	return add_row(row)

## Adds `row`; the entry point the probe and the Enter / double-click handlers share.
func add_row(row: Dictionary) -> String:
	var why := add_refusal(row)
	if why == "":
		why = DccSettings.quick_bar_add("bar", String(row["pin_key"]), String(row["title"]))
	if why != "":
		_say(why)
		return why
	_say("Pinned \"%s\"." % String(row["title"]), "text_secondary")
	_changed(String(row["pin_key"]))
	return ""

func _move(delta: int) -> bool:
	var i := _selected_pin_index()
	if i < 0:
		return false
	var key := String(_pin_keys[i])
	if not DccSettings.quick_bar_move("bar", key, delta):
		return false
	_say("", "text_secondary")
	_changed(key)
	return true

func _remove_selected() -> bool:
	var i := _selected_pin_index()
	if i < 0:
		return false
	var key := String(_pin_keys[i])
	var title := String(_pins_list.get_item_text(i))
	if not DccSettings.quick_bar_remove("bar", key):
		return false
	_say("Removed %s." % title.get_slice("  --  ", 0), "text_secondary")
	## Select the neighbour that slides into the gap, so repeated Delete clears the list
	## top to bottom without the focus going stale.
	## (`_pin_keys` still holds the list as rendered before the removal.)
	_changed(_neighbour_after_remove(i))
	return true

## The key that will occupy the removed pin's place: the next one, else the previous.
func _neighbour_after_remove(i: int) -> String:
	if i + 1 < _pin_keys.size():
		return String(_pin_keys[i + 1])
	if i - 1 >= 0:
		return String(_pin_keys[i - 1])
	return ""

func _ask_reset() -> void:
	var n := _pin_keys.size()
	if n == 0:
		_say("The quick bar is already empty.")
		return
	_reset_dlg.dialog_text = "Remove all %d pins from the quick bar? The commands themselves are not affected." % n
	_reset_dlg.popup_centered()

## Writes the EMPTY list (not "no write": clearing the last pin must persist, or an old list
## reappears on reload, scope s.3.4). Reached only through the confirmation dialog.
func _do_reset() -> void:
	if DccSettings.set_quick_bar_value("bar", []):
		_say("The quick bar is empty.", "text_secondary")
		_changed("")

## Common tail of every change: re-render both lists, tell the shell.
func _changed(select_key: String) -> void:
	_render_catalogue()
	_render_pins(select_key)
	pins_changed.emit()

## Keys on the pins list: Delete removes, Alt+Up / Alt+Down reorder (scope s.3.5).
func _pins_key(ev: InputEvent) -> void:
	if not (ev is InputEventKey) or not (ev as InputEventKey).pressed:
		return
	var k := ev as InputEventKey
	if k.keycode == KEY_DELETE:
		_remove_selected()
		_pins_list.accept_event()
	elif k.alt_pressed and k.keycode == KEY_UP:
		_move(-1)
		_pins_list.accept_event()
	elif k.alt_pressed and k.keycode == KEY_DOWN:
		_move(1)
		_pins_list.accept_event()

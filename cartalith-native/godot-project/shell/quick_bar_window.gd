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
## ## Three tabs, one catalogue (CQ-4)
##
## The scope draws three tabs (s.3.5). Milestone CQ-3 built only **Bar** because the other two edit
## lists nothing drew; CQ-4 puts the dock and the shared sub-rings on the cartwheel
## (`radial_ring.gd`, `quick_bar_ring.gd`), so it adds them here together with their consumers.
## The tabs are a `TabBar` over ONE catalogue and ONE pins list: the tab only decides which stored
## list the right-hand column edits and which cap applies.
##
##  * **Bar** -- the quick cluster, cap 24.
##  * **Ring dock** -- up to 4 chips beneath the cartwheel, cap 4.
##  * **Ring slots** -- a chooser over every diagonal of every domain. A diagonal that opens a
##    sub-ring (`children`) accepts pins after its built-in children, capped so built-ins plus pins
##    never exceed 9; the built-ins are listed read-only, read from the LIVE `ring_slots()` (so
##    this tab needs a loaded world: before one, the real count is unknown and the tab says so
##    instead of guessing). A diagonal that is a single tool (CARTO Label, CIVIL Route) is listed
##    disabled with the reason it cannot share.
##
## ## The phone layout (CQ-6)
##
## On a phone the window fills the screen (`DccWidgets.phone_window` / `phone_present`) and lays out in dp. The
## phone has no cartwheel, so the Ring dock and Ring slots tabs would edit lists nothing there draws: the tab
## strip and its note are hidden and only the Bar list is editable. The two columns do not fit 412 dp side by
## side, so the left (catalogue) and right (pins) columns are stacked under a two-segment strip, `Commands` and
## `Pinned`, one visible at a time. Every button, check box and field is floored to 44 dp on both axes and the
## list rows are given the separation that makes a row 44 dp tall (measured by `_quickbar_phone_probe.gd` by
## sampling `ItemList.get_item_at_position`, not read from a constant). All of it is built on the **first
## `open()`**, never in `setup()`: creating controls in a never-shown window at boot moved 26 014 pixels of the
## phone's Cartography sheet in CQ-4 and CQ-7, and the phone is exactly where this window is not yet open.

## A pin was added, removed, moved or the list was reset.
signal pins_changed

## The phone's tap floor in dp (`DccTheme.PHONE_TAP_MIN`, DESIGN_HANDOFF.md: 44, no exceptions). The window lays out in
## dp because `phone_present` sets its content scale factor to `phone_scale()`.
const PHONE_FLOOR_DP := 44.0
## The `v_separation` that, added to a one-line row of the small mono face, reaches 44 dp. A labelled starting
## value: the probe measures the real hit height and fails if it is under the floor, so a font change shows up.
const PHONE_LIST_SEPARATION := 30
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
## The tab strip (Bar / Ring dock / Ring slots) and the list the right column edits.
var _tabs: TabBar
## "bar", "dock" or "share" -- which stored list `_pins_now()` reads and every action writes.
var _target := "bar"
## The one-line explanation under the tabs.
var _tab_note: Label
## Ring slots tab only: the diagonal chooser, the read-only list of that diagonal's built-in items,
## and the rows behind the chooser's entries.
var _diag: OptionButton
var _builtin_lbl: Label
var _diag_rows: Array = []
## The selected diagonal's `<domain>.<DIR>` key (or "" with none), and how many built-in children it
## has now (-1 for a single-tool diagonal).
var _share_key := ""
var _share_builtin := -1
var _foot: Label
var _msg: Label
var _detail: Label
var _reset_dlg: ConfirmationDialog
## The two columns and the box that holds them, kept so the phone layout can restack them (CQ-6).
var _cols: Control
var _left: VBoxContainer
var _right: VBoxContainer
## The phone's `Commands` / `Pinned` strip, built on the first phone open; null elsewhere.
var _pages: TabBar
## Whether the phone layout has been applied (once per window).
var _phone_ready := false
## The user's switch for the conditional Assistant chip (CQ-7, Ruling DH Q6) and the line saying what
## the chip is doing right now and why.
var _assistant_chk: CheckBox
var _assistant_note: Label
## The dialog's content column, kept so the Assistant switch can be added on first open (below).
var _root_box: VBoxContainer
## The inset container holding everything; sized by `_fit_content`.
var _content: MarginContainer
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

## The text of one catalogue row: `Group › Title`, then `· on the <where>` when pinned in the list
## being edited (`where` is "bar", "ring dock" or "sub-ring"), or the reason it cannot be pinned (cut
## to `MAX_REASON_CHARS`; the whole reason is the tooltip).
static func row_text(row: Dictionary, is_pinned: bool, where: String = "bar") -> String:
	var t := "%s  ›  %s" % [String(row["group"]), String(row["title"])]
	if not bool(row.get("pinnable", false)):
		var why := String(row.get("why_not", ""))
		if why.length() > MAX_REASON_CHARS:
			why = why.substr(0, MAX_REASON_CHARS - 1) + "…"
		return "%s  --  %s" % [t, why]
	return t + (("  ·  on the %s" % where) if is_pinned else "")

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
	## Taller than CQ-3's 580 x 440: the tab strip and its note add about 70 px, and at 580 the Add
	## button and the footer fell below the window's bottom edge (the CQ-4 probe's first screenshot).
	size = Vector2i(820, 660)
	min_size = Vector2i(660, 520)
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
	_root_box = root
	_tabs = TabBar.new()
	_tabs.name = "Tabs"
	_tabs.add_tab("Bar")
	_tabs.add_tab("Ring dock")
	_tabs.add_tab("Ring slots")
	_tabs.focus_mode = Control.FOCUS_ALL
	if tablet:
		_tabs.custom_minimum_size.y = DccTheme.role_px("btn_min_h")
	_tabs.tab_changed.connect(func(_i: int): _on_tab_changed())
	root.add_child(_tabs)
	_tab_note = DccTheme.label("", "text_secondary", DccTheme.FS_SMALL)
	_tab_note.name = "TabNote"
	_tab_note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_tab_note.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## Width floor: an autowrap label measures its minimum height at its current width (1 px before the
	## first layout) -- see `_builtin_lbl`.
	_tab_note.custom_minimum_size = Vector2(320, 0)
	root.add_child(_tab_note)
	var cols := HBoxContainer.new()
	cols.add_theme_constant_override("separation", 14)
	cols.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_child(cols)
	_cols = cols

	var left := VBoxContainer.new()
	left.add_theme_constant_override("separation", 6)
	left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left.size_flags_vertical = Control.SIZE_EXPAND_FILL
	left.size_flags_stretch_ratio = 1.25
	cols.add_child(left)
	_left = left
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
	_right = right
	_diag = OptionButton.new()
	_diag.name = "Diagonal"
	_diag.focus_mode = Control.FOCUS_ALL
	_diag.add_theme_font_override("font", DccTheme.mono(0))
	_diag.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	_diag.fit_to_longest_item = false
	_diag.clip_text = false
	_diag.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if tablet:
		_diag.custom_minimum_size.y = DccTheme.role_px("btn_min_h")
	_diag.item_selected.connect(func(_i: int): _on_diagonal_selected())
	_diag.visible = false
	right.add_child(_diag)
	_builtin_lbl = DccTheme.label("", "text_ghost", DccTheme.FS_MICRO)
	_builtin_lbl.name = "Builtin"
	_builtin_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_builtin_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## An autowrap label reports its minimum height at its CURRENT width, which is 1 px until it is laid
	## out: a long built-in list then measured 1 152 px and the dialog grew to 3 700 px, pushing Add and
	## the footer out of sight (found by the CQ-4 probe). A width floor stops that.
	_builtin_lbl.custom_minimum_size = Vector2(240, 0)
	_builtin_lbl.visible = false
	right.add_child(_builtin_lbl)
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
	_content = m
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
	_tabs.set_current_tab(0)
	_target = "bar"
	## CQ-7: the Assistant chip's switch follows the store, and its note follows every setting change
	## while the window is open. Connected here, not in `setup()` (the phone lesson below).
	if not DccSettings.assistant_hub.changed.is_connected(_sync_assistant_switch):
		DccSettings.assistant_hub.changed.connect(_sync_assistant_switch)
	_ensure_assistant_switch()
	_sync_assistant_switch()
	_ensure_phone_layout()
	_rebuild_all()
	## A phone fills the screen (`phone_present` opens the window itself); desktop and tablet centre it.
	if not DccWidgets.phone_present(self, app):
		popup_centered()
	_search.grab_focus.call_deferred()
	## Follow the user's own resizes of the dialog from now on. Connected here, not in `setup()`:
	## connected at boot it changed the PHONE's CARTO layout (the `_quickbar_ringid_probe.gd` phone run
	## differed by 26 014 px, and by none with this one line removed), for a window the phone never opens.
	if not size_changed.is_connected(_fit_content):
		size_changed.connect(_fit_content)
	## After the dialog's own first layout (a deferred call ran too early and was overwritten).
	await get_tree().process_frame
	_fit_content()

## Builds the Assistant chip's switch and its note on the FIRST `open()`, below the lead paragraph. **Not in
## `setup()`**: creating these two controls at boot changed 26 014 pixels of the phone's Cartography sheet
## in `_quickbar_ringid_probe.gd` (desktop and tablet were untouched), on a phone that never opens this
## window -- the same unexplained shape as the `size_changed` connection that `open()` already avoids (see
## its comment). Deferring them to the first open removes the difference (measured, section 15 of the
## scope). Why a never-shown dialog's children moved pixels elsewhere was not traced.
func _ensure_assistant_switch() -> void:
	if _assistant_chk != null:
		return
	_assistant_chk = CheckBox.new()
	_assistant_chk.name = "AssistantChipSwitch"
	_assistant_chk.text = "Show the Assistant chip when the assistant is ready"
	_assistant_chk.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	_assistant_chk.tooltip_text = "The chip appears under the cartwheel and at the start of the bar once the assistant is enabled and a provider is set. This switch hides it; it is not one of your pins."
	_assistant_chk.toggled.connect(func(on: bool): DccSettings.set_quick_bar_value("assistant_button", on))
	_root_box.add_child(_assistant_chk)
	_assistant_note = DccTheme.label("", "text_ghost", DccTheme.FS_MICRO)
	_assistant_note.name = "AssistantChipNote"
	_assistant_note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_assistant_note.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_assistant_note.custom_minimum_size = Vector2(320, 0)
	_root_box.add_child(_assistant_note)
	## Directly under the lead paragraph (child 0), above the tabs.
	_root_box.move_child(_assistant_chk, 1)
	_root_box.move_child(_assistant_note, 2)

## Applies the phone layout once, on the first `open()` on a phone (module header, "The phone layout"):
## restacks the two columns under a `Commands` / `Pinned` strip, hides the ring tabs the phone cannot use,
## and floors every control to 44 dp. A no-op off the phone and after the first call.
func _ensure_phone_layout() -> void:
	if _phone_ready or app == null or not app.has_method("is_phone") or not app.is_phone():
		return
	_phone_ready = true
	## The lead paragraph names where the pins appear; on a phone that is not "beside the tool palette".
	(_root_box.get_child(0) as Label).text = "Pin commands to the chip row in the MAP sheet and the Quick section in MORE. Pins belong to this device and are never saved in a world, an export or a project file."
	_tabs.visible = false
	_tab_note.visible = false
	## A `HBoxContainer` cannot change orientation (`BoxContainer.is_fixed`), so the columns move into a
	## `VBoxContainer` that takes the old box's place.
	var stack := VBoxContainer.new()
	stack.name = "PhoneColumns"
	stack.add_theme_constant_override("separation", 8)
	stack.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var at := _cols.get_index()
	_root_box.add_child(stack)
	_root_box.move_child(stack, at)
	_left.reparent(stack, false)
	_right.reparent(stack, false)
	_left.size_flags_stretch_ratio = 1.0
	_cols.queue_free()
	_cols = stack
	## The stacked layout is taller than a landscape phone (373 dp: measured, Add and the pinned list fell
	## below the window's edge), so the whole column scrolls vertically. Horizontal stays DISABLED, which folds
	## the column's minimum WIDTH into the scroller's (the trap in MISTAKES.md): every label in the column is
	## width-bounded, and `_quickbar_phone_probe.gd` walks the window for a minimum wider than the window.
	## The column keeps `EXPAND_FILL`, so in portrait, where it fits, the lists still take the spare height.
	var scroll := ScrollContainer.new()
	scroll.name = "PhoneScroll"
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_content.add_child(scroll)
	_root_box.reparent(scroll, false)
	_root_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_root_box.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_pages = TabBar.new()
	_pages.name = "PhonePages"
	_pages.add_tab("Commands")
	_pages.add_tab("Pinned")
	_pages.focus_mode = Control.FOCUS_ALL
	_pages.custom_minimum_size.y = PHONE_FLOOR_DP
	_pages.tab_changed.connect(func(_i: int): _show_phone_page())
	_root_box.add_child(_pages)
	_root_box.move_child(_pages, stack.get_index())
	_show_phone_page()
	## Floors on both axes. A check box's long label would otherwise be the window's minimum width (the
	## overflow family in MISTAKES.md), so those wrap.
	for n in find_children("*", "BaseButton", true, false):
		var b := n as BaseButton
		b.custom_minimum_size = Vector2(maxf(b.custom_minimum_size.x, PHONE_FLOOR_DP), maxf(b.custom_minimum_size.y, PHONE_FLOOR_DP))
		if b is Button and (b as Button).text.length() > 24:
			(b as Button).autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
			b.custom_minimum_size.x = PHONE_FLOOR_DP
	_search.custom_minimum_size.y = PHONE_FLOOR_DP
	_diag.custom_minimum_size.y = PHONE_FLOOR_DP
	## A list row is its text plus `v_separation`, and a tap is hit-tested over the row PLUS the separation, so
	## the separation is what makes the row tall enough to touch.
	for l in [_catalogue, _pins_list]:
		(l as ItemList).add_theme_constant_override("v_separation", PHONE_LIST_SEPARATION)

## Shows the `Commands` or the `Pinned` column on the phone, by the strip's current segment.
func _show_phone_page() -> void:
	if _pages == null:
		return
	_left.visible = _pages.current_tab == 0
	_right.visible = _pages.current_tab == 1

## Brings the Assistant chip's checkbox and its note up to date with the store. The note names the
## chip's actual state in words, with the gate's own reason when it is greyed, so a user who sees no
## chip can read why (switch off, assistant off, no provider or model).
func _sync_assistant_switch() -> void:
	if _assistant_chk == null:
		return
	_assistant_chk.set_pressed_no_signal(bool(DccSettings.quick_bar_value("assistant_button")))
	var st := AssistantChip.state_for_app(app)
	match String(st["state"]):
		AssistantChip.LIVE:
			_assistant_note.text = "The Assistant chip is shown and opens the assistant panel."
		AssistantChip.DISABLED:
			_assistant_note.text = "The Assistant chip is shown greyed: %s" % String(st["reason"])
		_:
			_assistant_note.text = "The Assistant chip is hidden: it needs the assistant switched on in Preferences, a provider and a model, and this switch on."

## Sizes the content to the dialog. The content is first laid out while its autowrap labels are 1 px
## wide, which measures them thousands of pixels tall, and `AcceptDialog` then leaves it at that size
## (measured on `open()` with the full catalogue: content 3 147 px tall in a 660 px window, so the Add
## button and the footer sat at y = 3 073, below the window's bottom edge, out of sight and out of
## reach of a click; the very same happens in CQ-3's 580 px window, which its probe never saw because
## it searched first, leaving a short list). The content's minimum is only 439 px; setting its size
## by hand is what fixes it (measured: Add at y = 486). It stops at the top of the Close row.
func _fit_content() -> void:
	if _content == null or not visible:
		return
	var row: Control = get_ok_button().get_parent() as Control
	## The window's `size` is in physical pixels; its children lay out in logical units, which differ by
	## `content_scale_factor` (1.0 on desktop and tablet, `phone_scale()` on a phone, set by `phone_present`).
	var logical := Vector2(size) / maxf(content_scale_factor, 0.01)
	var bottom := row.position.y if row != null else logical.y
	_content.size = Vector2(logical.x - _content.position.x * 2.0, bottom - _content.position.y)

## Rebuilds the index and both lists. The index is rebuilt (not cached) so a pin's "not in this
## build" state and every row's availability are those of the moment, not of the last open.
func _rebuild_all() -> void:
	_idx = CommandIndex.new()
	_idx.build(app, app.bridge)
	_render_diagonals()
	_apply_target_chrome()
	_render_catalogue()
	_render_pins()

# -- the three targets (CQ-4) ---------------------------------------------------------------

## The stored list the right-hand column is editing, in order.
func _pins_now() -> Array:
	match _target:
		"dock":
			return DccSettings.quick_bar_value("dock")
		"share":
			return DccSettings.quick_bar_share_pins(_share_key)
	return DccSettings.quick_bar_value("bar")

## The word the catalogue puts after `on the` for a command already in the edited list.
func _where() -> String:
	match _target:
		"dock":
			return "ring dock"
		"share":
			return "sub-ring"
	return "bar"

## The cap of the edited list right now (24, 4, or the diagonal's allowance).
func _cap_now() -> int:
	match _target:
		"dock":
			return DccSettings.QUICK_BAR_MAX_DOCK
		"share":
			return DccSettings.quick_bar_share_allowance(maxi(0, _share_builtin)) if _share_builtin >= 0 else 0
	return DccSettings.QUICK_BAR_MAX_BAR

## A tab was chosen: switch the edited list, redraw both columns.
func _on_tab_changed() -> void:
	_target = ["bar", "dock", "share"][clampi(_tabs.current_tab, 0, 2)]
	_msg.text = ""
	_render_diagonals()
	_apply_target_chrome()
	_render_catalogue()
	_render_pins()

## Shows or hides the diagonal chooser and writes the explanation line for the current tab.
func _apply_target_chrome() -> void:
	_diag.visible = _target == "share"
	_builtin_lbl.visible = _target == "share"
	match _target:
		"bar":
			_tab_note.text = "Pins on the bar beside the tool palette."
		"dock":
			_tab_note.text = "Up to %d chips under the cartwheel, beside its eight slots (never replacing one). A quick flick never selects them." % DccSettings.QUICK_BAR_MAX_DOCK
		"share":
			_tab_note.text = "Pins go after a diagonal's built-in items (a sub-ring places at most %d). Single-tool diagonals cannot share." % DccSettings.QUICK_BAR_MAX_SHARE
	_tab_note.visible = not _phone_ready

## Fills the diagonal chooser from the live rings: one entry per domain and diagonal, a single-tool
## diagonal or one that cannot be read yet disabled with the reason in its own text. Keeps the
## selected diagonal selected when it is still listed, else picks the first that can share.
func _render_diagonals() -> void:
	var keep := _share_key
	_diag.clear()
	_diag_rows.clear()
	var world: bool = app.bridge != null and app.bridge.has_world
	for d in QuickBarRing.DOMAINS:
		var built: Dictionary = QuickBarRing.builtin_diagonals(app, d)
		for dir in QuickBarRing.DIAGONALS:
			var key := QuickBarRing.share_key(d, dir)
			var head := "%s · %s" % [String(QuickBarRing.DOMAIN_NAMES.get(d, d)), dir]
			var row := {"key": key, "domain": d, "dir": dir, "children": -2, "label": dir}
			var text := ""
			var usable := false
			if not built.has(dir):
				text = "%s  --  %s" % [head, "load a world to read this ring" if not world else "no ring here"]
			else:
				var b: Dictionary = built[dir]
				row["label"] = String(b["label"])
				row["children"] = int(b["children"])
				if int(b["children"]) < 0:
					text = "%s · %s  --  a single tool with no sub-ring: cannot share" % [head, String(b["label"])]
				else:
					var n := DccSettings.quick_bar_share_pins(key).size()
					text = "%s · %s  --  %d built-in, %d of %d pinned" % [head, String(b["label"]), int(b["children"]), n, DccSettings.quick_bar_share_allowance(int(b["children"]))]
					usable = true
			var i := _diag.item_count
			_diag.add_item(text)
			_diag.set_item_disabled(i, not usable)
			_diag_rows.append(row)
	var at := -1
	for i in _diag_rows.size():
		if _diag.is_item_disabled(i):
			continue
		if at < 0:
			at = i
		if String(_diag_rows[i]["key"]) == keep:
			at = i
			break
	if at >= 0:
		_diag.select(at)
		_share_key = String(_diag_rows[at]["key"])
		_share_builtin = int(_diag_rows[at]["children"])
	else:
		_diag.select(-1)
		_share_key = ""
		_share_builtin = -1
	_update_builtin_label()

## The chooser changed.
func _on_diagonal_selected() -> void:
	var i := _diag.selected
	if i < 0 or i >= _diag_rows.size():
		return
	_share_key = String(_diag_rows[i]["key"])
	_share_builtin = int(_diag_rows[i]["children"])
	_msg.text = ""
	_update_builtin_label()
	_render_catalogue()
	_render_pins()

## The read-only list of the selected diagonal's built-in children (or why there is none).
func _update_builtin_label() -> void:
	if _share_key == "":
		var world: bool = app.bridge != null and app.bridge.has_world
		_builtin_lbl.text = "Load or generate a world first: a diagonal's built-in items are read from the live ring, and how many more fit depends on them." if not world \
			else "No diagonal can share a sub-ring here."
		return
	var parts := _share_key.split(".")
	var kids := QuickBarRing.builtin_child_labels(app, parts[0], parts[1])
	_builtin_lbl.text = "Built in, always first (%d): %s" % [kids.size(), ", ".join(PackedStringArray(kids))]

func _pinned_keys() -> Dictionary:
	var out := {}
	for p in _pins_now():
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
		var i := _catalogue.add_item(row_text(r, is_pinned, _where()))
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
	var pins: Array = _pins_now()
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
	match _target:
		"dock":
			_count.text = "ON THE RING DOCK  %d / %d" % [pins.size(), _cap_now()]
		"share":
			_count.text = ("ON %s  %d / %d" % [_share_key.to_upper(), pins.size(), _cap_now()]) if _share_key != "" else "NO DIAGONAL"
		_:
			_count.text = "ON THE BAR  %d / %d" % [pins.size(), _cap_now()]
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
		why = _add_to_target(String(row["pin_key"]), String(row["title"]))
	if why != "":
		_say(why)
		return why
	_say("Pinned \"%s\"." % String(row["title"]), "text_secondary")
	_changed(String(row["pin_key"]))
	return ""

## Appends to the edited list through `DccSettings`, whose caps and sentences are the store's: ""
## on success, else the sentence to show. A diagonal that cannot share is refused here with the
## chooser's own reason (the store refuses a leaf too, but the window knows which one is chosen).
func _add_to_target(pin_key: String, title: String) -> String:
	match _target:
		"dock":
			return DccSettings.quick_bar_add("dock", pin_key, title)
		"share":
			if _share_key == "":
				return "Load a world, then choose a diagonal that opens a sub-ring."
			return DccSettings.quick_bar_share_add(_share_key, pin_key, title, _share_builtin)
	return DccSettings.quick_bar_add("bar", pin_key, title)

## `quick_bar_move` / `_remove` for the edited list.
func _move_in_target(key: String, delta: int) -> bool:
	if _target == "share":
		return DccSettings.quick_bar_share_move(_share_key, key, delta)
	return DccSettings.quick_bar_move("dock" if _target == "dock" else "bar", key, delta)

func _remove_in_target(key: String) -> bool:
	if _target == "share":
		return DccSettings.quick_bar_share_remove(_share_key, key)
	return DccSettings.quick_bar_remove("dock" if _target == "dock" else "bar", key)

func _move(delta: int) -> bool:
	var i := _selected_pin_index()
	if i < 0:
		return false
	var key := String(_pin_keys[i])
	if not _move_in_target(key, delta):
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
	if not _remove_in_target(key):
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
		_say("The %s is already empty." % ("quick bar" if _target == "bar" else _where()))
		return
	_reset_dlg.dialog_text = "Remove all %d pins from the %s? The commands themselves are not affected." % [n, "quick bar" if _target == "bar" else ("ring dock" if _target == "dock" else "sub-ring %s" % _share_key)]
	_reset_dlg.popup_centered()

## Writes the EMPTY list (not "no write": clearing the last pin must persist, or an old list
## reappears on reload, scope s.3.4). Reached only through the confirmation dialog.
func _do_reset() -> void:
	var ok := false
	match _target:
		"dock":
			ok = DccSettings.set_quick_bar_value("dock", [])
		"share":
			ok = DccSettings.quick_bar_share_clear(_share_key)
		_:
			ok = DccSettings.set_quick_bar_value("bar", [])
	if ok:
		_say("The %s is empty." % ("quick bar" if _target == "bar" else _where()), "text_secondary")
		_changed("")

## Common tail of every change: re-render both lists, tell the shell.
func _changed(select_key: String) -> void:
	if _target == "share":
		_render_diagonals()
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

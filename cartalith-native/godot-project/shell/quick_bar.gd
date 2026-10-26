extends Container
class_name DccQuickBar

## The quick cluster: the user's pinned commands, drawn at the right end of the tool
## palette bar (`CUSTOM_QUICK_BAR_SCOPE.md` s.3.1, milestone CQ-2, Ruling DH Q2).
##
## ## A pin is a pointer, never a second implementation (scope s.0 rule 1, s.3.6)
##
## A chip stores only a `pin_key` (`DccSettings` `[quick_bar]`, CQ-1). Pressing it builds a
## fresh `CommandIndex`, finds the live row for that key and calls `CommandIndex.run_row()`,
## which emits the owning `PopupMenu`'s own `id_pressed` -- the signal the menu click's real
## handler (`_on_file`, `_on_edit`, ...) is connected to. Result and side effects are those of
## the menu click because it IS the menu click. Nothing here knows what any command does.
##
## A pin whose command is gone from this build stays: a greyed chip titled with the title it
## had when pinned, saying why. It is never deleted and never re-pointed (scope s.0 rule 1).
##
## ## Why this is a Container with a CONSTANT minimum width
##
## The palette bar was introduced as a bar of its own because prefixing the options bar pushed
## the window's minimum width past 1 600 px (`app.gd::_install_tool_palette_bar`). A row of up
## to 24 chips would do the same, and `MISTAKES.md` records the shape three times ("Read a
## layout that overflows the screen"). So this node reports a minimum that does NOT depend on
## how many pins it holds -- the `＋` chip's width -- takes whatever width the bar has left
## (`SIZE_EXPAND_FILL`), shows the pins that fit and puts the rest behind a `⋯` chip's
## `PopupMenu`, in pin order. The shown count is computed from the width this node was GIVEN,
## not from the window's. No chip label uses `clip_text` (it collapses a Label's minimum to 1);
## a long title is cut in code (`MAX_LABEL_CHARS`) and the full title is the tooltip.
##
## ## Where pins are edited (CQ-3)
##
## All adding happens in the Customise window (`QuickBarWindow`, `Preferences ▸ Quick bar…`),
## which the `＋` chip, the `⋯` menu's last row and a chip's right-click menu all open. The
## interim grouped menu CQ-2 hung off `＋` is gone: the window lists the same pinnable rows, with
## search and the reasons for the rest, so keeping both would be two ways to do one thing. A
## chip's right-click menu still removes or moves it in place.
##
## ## What is not here (the milestones after it)
##
## The ring dock and the shared sub-rings are `quick_bar_ring.gd`'s (CQ-4); it reuses `run_pin_on`,
## `accelerator_text` and `label_for` from here. The phone has no palette bar and so no cluster: its
## chip row and Quick section are `quick_bar_phone.gd`'s (CQ-6), reading the same store. Tools and check/radio rows are not pinnable in v1 (Ruling DH Q8, Q10); `CommandIndex` says
## why per row.
##
## ## The Assistant chip (CQ-7, Ruling DH Q6)
##
## The first chip of the cluster, when `AssistantChip.state_for_app` says it is shown (the assistant
## is enabled, a provider is configured and the user has not hidden it; greyed with the gate's own
## reason when the gate refuses). It is a built-in conditional item, NOT one of the stored pins: it
## is not in `_chips`, is not counted by `pin_count()`, and is not part of the 24-pin cap. It is
## re-evaluated, with no index build, whenever `DccSettings.assistant_hub` announces a change, so it
## follows a setting inside the same call. It does not change `_get_minimum_size()`: when the bar is
## too narrow for it, it moves into the `⋯` menu's first row rather than widening the shell.

## The gap between chips: the palette strip's own `separation` (`DccWidgets.tool_strip`).
const GAP := 5
## Longest chip label in characters before it is cut with an ellipsis. A labelled judgement:
## about the width of "Journey planner" in the small mono face; the full title is the tooltip.
const MAX_LABEL_CHARS := 16
## The id of the `Customise…` row in the `⋯` and chip menus. Far above any pin index (the `⋯`
## menu numbers its pins 0..23) so the two never collide.
const ID_CUSTOMISE := 1000
## The `⋯` menu's Assistant row and the Assistant chip's menu rows (CQ-7); above any pin index.
const ID_ASSISTANT := 1001
const ID_ASSISTANT_HIDE := 1002
## Ruling DQ: the Assistant chip's menu lists the profiles; row `i` carries id `ID_PROFILE_BASE + i`.
## Above every other id in this file and well below Godot's menu id limits; 16 profiles at most.
const ID_PROFILE_BASE := 1100
## Reference to the owning `DccApp` for `bridge` and the menu walk; untyped for a harness stub.
var _app
## Pinned chips in pin order, one per stored pin (hidden ones are overflow).
var _chips: Array[Button] = []
## The `＋` chip: always present, last.
var _plus: Button
## The `⋯` overflow chip: shown only while at least one pin does not fit.
var _more: Button
## The conditional Assistant chip (CQ-7): always a child, visible only while `_assistant_state` is not
## hidden and it fits. NOT a pin (module header).
var _assistant: Button
## The last `AssistantChip.state_for_app` result: `{state, reason}`.
var _assistant_state := {"state": AssistantChip.HIDDEN, "reason": ""}
## The menu the Assistant chip's right-click pops up (Hide, Customise).
var _assistant_menu: PopupMenu
## The menus the `⋯` chip and a pin's right-click pop up.
var _more_menu: PopupMenu
var _chip_menu: PopupMenu
## The pin the open chip context menu belongs to.
var _chip_menu_key := ""
## How many pins were shown, in the last layout.
var _shown := 0
## The index of the last `refresh()`; rebuilt for every press so a press is never stale.
var _index: CommandIndex
## Milliseconds the last index build took (reported by `_quickbar_probe.gd`).
var last_index_ms := 0.0

## Emitted whenever the stored pin list changed through this node.
signal pins_changed

func _init() -> void:
	name = "QuickBar"
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	clip_contents = true
	## Created BEFORE the `＋` chip so it is the first child and the first thing drawn (scope s.5:
	## "first chip of the quick cluster"); pins are inserted before `＋`, hence after it.
	_assistant = _make_chip(AssistantChip.LABEL, _on_assistant_pressed)
	_assistant.name = "QuickBarAssistant"
	_assistant.visible = false
	_assistant.gui_input.connect(_assistant_input)
	_plus = _make_chip("＋", _on_plus_pressed)
	_plus.name = "QuickBarPlus"
	_nudge(_plus, 1)
	_more = _make_chip("⋯", _on_more_pressed)
	_more.name = "QuickBarMore"
	_nudge(_more, -1)
	_more.visible = false
	_more_menu = PopupMenu.new()
	_chip_menu = PopupMenu.new()
	_assistant_menu = PopupMenu.new()
	for m in [_more_menu, _chip_menu, _assistant_menu]:
		add_child(m)
	_assistant_menu.id_pressed.connect(_on_assistant_menu_item)
	_more_menu.id_pressed.connect(_on_more_item)
	_chip_menu.id_pressed.connect(_on_chip_menu_item)

## Binds the owning app and draws the pins. Call once, after the shell is built.
func setup(app) -> void:
	_app = app
	## Follow the settings that decide the Assistant chip (CQ-7). Connected once; the hub is a static
	## on `DccSettings`, so a connection to a freed bar is dropped by the engine.
	if not DccSettings.assistant_hub.changed.is_connected(refresh_assistant):
		DccSettings.assistant_hub.changed.connect(refresh_assistant)
	refresh()

## The chip height every chip is floored to: the palette cell's. Desktop 30 (the palette's own
## `tool_button` figure), tablet `role_px("btn_min_h")` (44, the touch floor).
static func cell_px() -> int:
	return DccTheme.role_px("btn_min_h") if DccTheme.is_tablet() else 30

## One chip: the shell's outline chip, floored on BOTH axes to the cell size (a tablet chip is
## at least 44 x 44; `MISTAKES.md` "Floor a control that a rebuild creates").
func _make_chip(text: String, on_press: Callable) -> Button:
	var tablet := DccTheme.is_tablet()
	var px := DccTheme.role_px("chip_pad_x") if tablet else 8
	var py := DccTheme.role_px("chip_pad_y") if tablet else 3
	var b := DccWidgets.chip(self, text, on_press, false, px, py)
	b.custom_minimum_size = Vector2(cell_px(), cell_px())
	return b

## Shifts a chip's text down by `dy` px (up when negative) by moving its content box, on every
## stylebox state once (`normal` and `pressed` are one shared instance). Used for the two symbol
## chips only: the `＋` and `⋯` glyphs have different vertical bearings in the mono face, so a
## geometrically centred chip draws them 1.5 px high and 1 px low (measured by
## `_alignment_probe.gd`, desktop, owner finding 2026-10-26). A labelled judgement, re-measured
## per mode by that probe, not a font metric. A content box moves by half the margin difference,
## so the margin change is twice the shift.
static func _nudge(b: Button, dy: int) -> void:
	var seen := {}
	for st in ["normal", "pressed", "hover", "disabled"]:
		var sb: StyleBox = b.get_theme_stylebox(st)
		if seen.has(sb.get_instance_id()):
			continue
		seen[sb.get_instance_id()] = true
		if dy > 0:
			sb.content_margin_top += 2.0 * dy
		else:
			sb.content_margin_bottom += -2.0 * dy

## `title` as a chip label: cut to `MAX_LABEL_CHARS` with an ellipsis (never `clip_text`).
static func label_for(title: String) -> String:
	return title if title.length() <= MAX_LABEL_CHARS else title.substr(0, MAX_LABEL_CHARS - 1) + "…"

# -- state ----------------------------------------------------------------------------

## Number of pins stored (the number of chips, shown or not).
func pin_count() -> int:
	return _chips.size()

## Number of pin chips the last layout drew.
func shown_count() -> int:
	return _shown

## The titles of the pins behind the `⋯` chip, in pin order, as its menu lists them.
func overflow_titles() -> PackedStringArray:
	var out := PackedStringArray()
	for c in _chips:
		if not c.visible:
			out.append(String(c.get_meta("title", "")))
	return out

## The pin chips in pin order (shown or not), for tests.
func chips() -> Array[Button]:
	return _chips

func plus_chip() -> Button:
	return _plus

func more_chip() -> Button:
	return _more

## Rebuilds every chip from the store and a fresh index. Cheap enough to call on every
## workspace change and world load (the cost is `last_index_ms`); loses no pin because the
## pins live in `DccSettings`, not here.
func refresh() -> void:
	if _app == null:
		return
	var t0 := Time.get_ticks_usec()
	_index = CommandIndex.new()
	_index.build(_app, _app.bridge)
	last_index_ms = float(Time.get_ticks_usec() - t0) / 1000.0
	for c in _chips:
		remove_child(c)
		c.queue_free()
	_chips.clear()
	var pins: Array = DccSettings.quick_bar_value("bar")
	for p in pins:
		_chips.append(_chip_for(String(p["k"]), String(p["t"])))
	var full := pins.size() >= DccSettings.QUICK_BAR_MAX_BAR
	_plus.disabled = false
	_plus.tooltip_text = ("Customise the quick bar. It is full (%d / %d): remove a pin to add another." % [pins.size(), DccSettings.QUICK_BAR_MAX_BAR]) if full \
		else "Customise the quick bar: pin a command (%d / %d)" % [pins.size(), DccSettings.QUICK_BAR_MAX_BAR]
	refresh_assistant()
	update_minimum_size()
	queue_sort()

## Re-evaluates the Assistant chip from the settings and re-lays the bar. No index build, no network:
## cheap enough for every settings write (the hub calls it synchronously, so the chip is current in
## the same frame as the setting).
func refresh_assistant() -> void:
	if _app == null:
		return
	_assistant_state = AssistantChip.state_for_app(_app)
	var st := String(_assistant_state["state"])
	_assistant.disabled = st != AssistantChip.LIVE
	_assistant.tooltip_text = AssistantChip.tooltip_for(_assistant_state)
	queue_sort()

## The Assistant chip's state, `hidden`, `disabled` or `live`, as last evaluated (for tests).
func assistant_state() -> String:
	return String(_assistant_state["state"])

## The Assistant chip, for tests. It is visible only when shown AND it fits the bar's width.
func assistant_chip() -> Button:
	return _assistant

## Builds the chip for one stored pin against the live index row (or "not in this build").
func _chip_for(pin_key: String, stored_title: String) -> Button:
	var row := _index.find_pin(pin_key)
	var title := String(row.get("title", "")) if not row.is_empty() else stored_title
	if title == "":
		title = pin_key
	var b := _make_chip(label_for(title), func(): _press(pin_key))
	b.name = "Pin_" + pin_key.replace(":", "_").replace("/", "_")
	b.set_meta("pin_key", pin_key)
	b.set_meta("title", title)
	b.gui_input.connect(func(ev: InputEvent): _chip_input(ev, b, pin_key))
	move_child(b, _plus.get_index())
	var tip := title
	if row.is_empty():
		b.disabled = true
		b.tooltip_text = "Not in this build: \"%s\". The pin is kept; right-click to remove it." % title
		return b
	var accel := accelerator_text(row)
	if accel != "":
		tip += "  (%s)" % accel
	## **A chip is live exactly when the index calls the row pinnable now** (`CommandIndex.run_row`
	## refuses anything else). That covers three cases with one rule: a command disabled in
	## the menu's built state (its own reason is shown), a row that is no longer a command
	## a chip may run (a check or radio row, a generated row: only a hand-edited file can
	## pin one), and the live case. A row whose menu merely REFRESHES its enabled state when
	## it opens (`refreshes_on_open`) and was built enabled stays live: the chip does not
	## copy a state it cannot see, and the shared handler answers (scope s.3.3).
	var usable := bool(row.get("pinnable", false))
	b.disabled = not usable
	if not usable:
		tip += " -- unavailable: " + String(row.get("why_not", ""))
	b.tooltip_text = tip
	return b

## The accelerator the menu row currently shows ("Ctrl+S"), or "". Read from the popup, which
## `menus.gd` binds from `DccSettings.shortcut_binding`, so a rebind shows here. Static since CQ-4
## so the ring dock's tooltip and caption (`quick_bar_ring.gd`) read it the same way a chip does.
static func accelerator_text(row: Dictionary) -> String:
	var popup = row.get("_popup")
	if popup == null or not is_instance_valid(popup):
		return ""
	var idx: int = (popup as PopupMenu).get_item_index(int(row.get("_id", -1)))
	if idx < 0:
		return ""
	var accel: int = (popup as PopupMenu).get_item_accelerator(idx)
	return OS.get_keycode_string(accel) if accel != 0 else ""

# -- running a pin --------------------------------------------------------------------

## A chip press: runs the pin and, when it could not run, says why in the status bar.
func _press(pin_key: String) -> void:
	var why := run_pin(pin_key)
	if why != "":
		_say(why)

## Runs the command pinned as `pin_key`. Answers "" when it ran, else the reason it did not
## (also shown in the status line by the caller). Resolves against a FRESH index, so a command
## re-homed or removed since the last refresh is found or refused honestly.
func run_pin(pin_key: String) -> String:
	return DccQuickBar.run_pin_on(_app, pin_key)

## The body of `run_pin`, static since CQ-4 (`CUSTOM_QUICK_BAR_SCOPE.md` s.4) so the ring dock and a
## shared sub-ring child run a pin through the SAME code a chip does -- one resolution, one refusal
## sentence each, no second implementation of "press a pin".
static func run_pin_on(app, pin_key: String) -> String:
	if app == null:
		return "The quick bar is not attached to the app."
	var idx := CommandIndex.new()
	idx.build(app, app.bridge)
	var row := idx.find_pin(pin_key)
	if row.is_empty():
		return "Not in this build."
	if not bool(row.get("pinnable", false)):
		return "Unavailable: " + String(row.get("why_not", "this command cannot be run from a chip."))
	return "" if idx.run_row(row) else "Could not run it."

# -- layout ---------------------------------------------------------------------------

func _get_minimum_size() -> Vector2:
	## CONSTANT: the `＋` chip. Never a function of the pin count (the module header).
	if _plus == null:
		return Vector2.ZERO
	return Vector2(_plus.get_combined_minimum_size().x, float(cell_px()))

func _notification(what: int) -> void:
	if what == NOTIFICATION_SORT_CHILDREN:
		_layout()

## Places the pins left to right that fit in the width this node has, then `⋯` (when any did
## not fit), then `＋`. A hidden pin stays a child (so `refresh()` keeps owning it) and shows
## in the `⋯` menu.
func _layout() -> void:
	var w := size.x
	## Chips are the cell SQUARE tall (`cell_px()`), centred in the height this node was given. The
	## bar stretches this node to its own full height (39 px at desktop), and filling it made every
	## chip a tall box around a symbol that sat 1-1.5 px off the middle (`_alignment_probe.gd`, owner
	## finding 2026-10-26). `round()` keeps the chip on whole pixels so its 1 px border stays crisp.
	var h := float(cell_px())
	var y0 := roundf((size.y - h) * 0.5)
	var pw := _plus.get_combined_minimum_size().x
	var ow := _more.get_combined_minimum_size().x
	## The Assistant chip leads the row when it is wanted and the bar can hold it beside `＋`. When it
	## is wanted and does not fit it is shown in the `⋯` menu instead, so the bar's own minimum
	## width never depends on it (module header).
	var aw := _assistant.get_combined_minimum_size().x
	var wanted := String(_assistant_state["state"]) != AssistantChip.HIDDEN
	var lead := aw + GAP if wanted and aw + GAP + pw <= w else 0.0
	var assistant_fits := lead > 0.0
	var natural := 0.0
	for c in _chips:
		natural += c.get_combined_minimum_size().x + GAP
	## `⋯` is needed when a pin does not fit OR the Assistant chip was moved into its menu.
	var all_fit := lead + natural + pw <= w
	var assistant_in_more := wanted and not assistant_fits
	var avail := (w - lead - pw - GAP) if all_fit and not assistant_in_more else (w - lead - pw - ow - 2 * GAP)
	var x := lead
	var shown := 0
	var overflowed := false
	if _assistant.visible != assistant_fits:
		_assistant.visible = assistant_fits
	if assistant_fits:
		fit_child_in_rect(_assistant, Rect2(0.0, y0, aw, h))
	for c in _chips:
		var cw := c.get_combined_minimum_size().x
		var fits := not overflowed and x + cw <= avail
		if not fits:
			overflowed = true
		if c.visible != fits:
			c.visible = fits
		if fits:
			fit_child_in_rect(c, Rect2(x, y0, cw, h))
			x += cw + GAP
			shown += 1
	_shown = shown
	overflowed = overflowed or assistant_in_more
	if _more.visible != overflowed:
		_more.visible = overflowed
	if overflowed:
		fit_child_in_rect(_more, Rect2(x, y0, ow, h))
		x += ow + GAP
	fit_child_in_rect(_plus, Rect2(x, y0, pw, h))

# -- the ＋ chip: add a pin -----------------------------------------------------------

## Opens the Customise window (CQ-3). The `＋` chip is the window's entry point on the bar;
## it stays live when the bar is full, because the window is also where a pin is removed and it
## states the cap's reason when an add is refused (scope s.3.5).
func _on_plus_pressed() -> void:
	open_customise()

## Opens `Preferences ▸ Quick bar…` through the app, the one method every entry point calls
## (`menus.gd`'s row, this node's `＋`, `⋯ ▸ Customise…` and a pin's right-click), so they cannot
## open different windows. A stub app without it says so in the status line, not silently.
func open_customise() -> void:
	if _app != null and _app.has_method("open_quick_bar_window"):
		_app.open_quick_bar_window()
	else:
		_say("The Customise window is not available here.")

# -- the ⋯ chip: pins that did not fit ------------------------------------------------

func _on_more_pressed() -> void:
	_more_menu.clear()
	## A wanted Assistant chip that did not fit the bar is the menu's first row (CQ-7), with its
	## own state: greyed with the gate's reason when the gate refuses.
	if String(_assistant_state["state"]) != AssistantChip.HIDDEN and not _assistant.visible:
		_more_menu.add_item(AssistantChip.LABEL, ID_ASSISTANT)
		_more_menu.set_item_disabled(_more_menu.get_item_index(ID_ASSISTANT), _assistant.disabled)
		_more_menu.set_item_tooltip(_more_menu.get_item_index(ID_ASSISTANT), _assistant.tooltip_text)
	var i := 0
	for c in _chips:
		if c.visible:
			continue
		_more_menu.add_item(String(c.get_meta("title", "")), i)
		## The ITEM INDEX, not the id: the Assistant row (CQ-7) may already sit at index 0, so a pin's
		## id (0..23) is no longer its position. Using `i` here set the wrong rows' state.
		var at := _more_menu.item_count - 1
		_more_menu.set_item_metadata(at, String(c.get_meta("pin_key", "")))
		_more_menu.set_item_disabled(at, c.disabled)
		_more_menu.set_item_tooltip(at, c.tooltip_text)
		i += 1
	_more_menu.add_separator()
	_more_menu.add_item("Customise…", ID_CUSTOMISE)
	DccWidgets.style_popup(_more_menu)
	_popup_under(_more_menu, _more)

func _on_more_item(id: int) -> void:
	if id == ID_CUSTOMISE:
		open_customise()
		return
	if id == ID_ASSISTANT:
		_on_assistant_pressed()
		return
	_press(String(_more_menu.get_item_metadata(_more_menu.get_item_index(id))))

# -- a chip's own right-click menu ----------------------------------------------------

func _chip_input(ev: InputEvent, chip: Button, pin_key: String) -> void:
	if not (ev is InputEventMouseButton) or not (ev as InputEventMouseButton).pressed:
		return
	if (ev as InputEventMouseButton).button_index != MOUSE_BUTTON_RIGHT:
		return
	open_chip_menu(chip, pin_key)
	chip.accept_event()

## Pops the chip's context menu (Remove, Move left, Move right). Public so a probe can reach it
## through a real right-click on the chip and also assert its rows.
func open_chip_menu(chip: Button, pin_key: String) -> void:
	_chip_menu_key = pin_key
	var pins: Array = DccSettings.quick_bar_value("bar")
	var at := -1
	for i in pins.size():
		if String(pins[i]["k"]) == pin_key:
			at = i
	_chip_menu.clear()
	_chip_menu.add_item("Remove from quick bar", 0)
	_chip_menu.add_item("Move left", 1)
	_chip_menu.add_item("Move right", 2)
	_chip_menu.add_separator()
	_chip_menu.add_item("Customise…", ID_CUSTOMISE)
	_chip_menu.set_item_disabled(1, at <= 0)
	_chip_menu.set_item_disabled(2, at < 0 or at >= pins.size() - 1)
	DccWidgets.style_popup(_chip_menu)
	_popup_under(_chip_menu, chip)

func _on_chip_menu_item(id: int) -> void:
	if id == ID_CUSTOMISE:
		open_customise()
		return
	var ok := false
	match id:
		0:
			ok = DccSettings.quick_bar_remove("bar", _chip_menu_key)
		1:
			ok = DccSettings.quick_bar_move("bar", _chip_menu_key, -1)
		2:
			ok = DccSettings.quick_bar_move("bar", _chip_menu_key, 1)
	if ok:
		refresh()
		pins_changed.emit()

# -- the Assistant chip (CQ-7) --------------------------------------------------------

## A press opens the assistant panel, and does nothing else: showing the chip makes no call, and
## neither does pressing it (the panel's Send is the only thing that does, through the gate). A chip
## that is not live is disabled and cannot be pressed; `AssistantChip.open_panel` re-checks anyway.
func _on_assistant_pressed() -> void:
	if not AssistantChip.open_panel(_app):
		_say(AssistantChip.tooltip_for(_assistant_state))

## The Assistant chip's right-click menu: the user's switch to hide it (Ruling DH Q6) and Customise.
## The same switch is a checkbox in the Customise window, where it can be turned back on.
func _assistant_input(ev: InputEvent) -> void:
	if not (ev is InputEventMouseButton) or not (ev as InputEventMouseButton).pressed:
		return
	if (ev as InputEventMouseButton).button_index != MOUSE_BUTTON_RIGHT:
		return
	_assistant_menu.clear()
	_fill_profile_rows(_assistant_menu)
	_assistant_menu.add_item("Hide the Assistant chip", ID_ASSISTANT_HIDE)
	_assistant_menu.add_separator()
	_assistant_menu.add_item("Customise…", ID_CUSTOMISE)
	DccWidgets.style_popup(_assistant_menu)
	_popup_under(_assistant_menu, _assistant)
	_assistant.accept_event()

## Adds one radio row per profile (the active one checked) and a separator, to `menu`. Nothing is added
## when profiles are unavailable. A pick calls `LlmProfiles.switch_to`, which applies the whole profile
## and shows the consent card first when the new endpoint has none.
func _fill_profile_rows(menu: PopupMenu) -> void:
	var set := LlmProfiles.load_set()
	if not bool(set.get("ok", false)):
		return
	var active := LlmProfiles.active_id()
	var rows: Array = set["profiles"]
	for i in rows.size():
		menu.add_radio_check_item("Profile: " + String(rows[i]["name"]), ID_PROFILE_BASE + i)
		menu.set_item_checked(menu.item_count - 1, String(rows[i]["id"]) == active)
		menu.set_item_metadata(menu.item_count - 1, String(rows[i]["id"]))
	menu.add_separator()

func _on_assistant_menu_item(id: int) -> void:
	if id >= ID_PROFILE_BASE and id < ID_PROFILE_BASE + 16:
		var idx := _assistant_menu.get_item_index(id)
		var r := LlmProfiles.switch_to(String(_assistant_menu.get_item_metadata(idx)))
		if not bool(r.get("ok", false)):
			_say(String(r.get("reason", "Not switched.")))
		return
	if id == ID_CUSTOMISE:
		open_customise()
	elif id == ID_ASSISTANT_HIDE:
		## The hub's announcement re-evaluates this chip; nothing else to do.
		DccSettings.set_quick_bar_value("assistant_button", false)

# -- helpers --------------------------------------------------------------------------

func _popup_under(menu: PopupMenu, anchor: Control) -> void:
	var r := anchor.get_global_rect()
	menu.popup(Rect2i(Vector2i(int(r.position.x), int(r.end.y)), Vector2i.ZERO))

## One warning line in the status bar's hint slot (`DccShell.set_status`), the same place a
## refused tool arm reports (`app.gd`'s "Not done: ..." line).
func _say(text: String) -> void:
	DccQuickBar.say_on(_app, text)

## `_say` for a caller that holds only the app (the ring rows, `quick_bar_ring.gd`).
static func say_on(app, text: String) -> void:
	if app != null and app.has_method("set_status"):
		app.set_status("hint", text, "warn")

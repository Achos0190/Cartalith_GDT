extends PanelContainer
## Ruling EO (2026-11-11): **on touch, a tap on a map element selects it and shows its action chips** -- Move, Rename,
## Note and Delete, as they apply to that element. A long press keeps the full context card (`MAP_CONTEXT_SCOPE.md`
## fork F1). `EDIT_FROM_CONTEXT_AUDIT.md` is the design.
##
## **The chips are the card's own rows, never a second set** (Ruling EO item 3): `DccApp._on_element_tapped` asks the
## broker for the rows of the tapped point (`ContextBroker.collect`, the same merge a long press presents) and hands
## every row that carries a `tap` key to `show_for`. Pressing a chip runs that row's own `callable`, so each action is
## the existing path with the existing confirm (Delete asks first: labels, icons and places have no undo history).
##
## A strip of 44 dp targets at the top centre of the map: the bottom holds the phone's sheet and the map's control
## row. It never blocks the map -- a tap anywhere else hides it (`hide_chips`), as does a selection change, a domain
## change, an armed tool and pressing a chip. Must never present on desktop (no tap gesture there) or while a tool
## other than Inspect is armed (a tap then places something).

## Chip order, left to right, by the row's `tap` word: the verbs the owner named, safest first, Delete last.
const ORDER := ["Move", "Rename", "Note", "Delete"]

## The 44 dp touch floor (`06-phone.md` section 13), in UI units.
const MIN_TARGET := 44.0

var _row: HBoxContainer
## The title of the element the chips act on (shown as the strip's first label).
var _title: Label


func _init() -> void:
	name = "ElementChips"
	set_meta("map_chrome", true)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	_row = HBoxContainer.new()
	_row.add_theme_constant_override("separation", 6)
	add_child(_row)
	var sb := StyleBoxFlat.new()
	sb.bg_color = DccTheme.c("panel")
	sb.set_corner_radius_all(10)
	sb.set_content_margin_all(6)
	sb.border_color = DccTheme.c("border")
	sb.set_border_width_all(1)
	add_theme_stylebox_override("panel", sb)


## Shows the chips for `rows` (the broker's merged rows for the tapped point) under `title`. Only a row with a `tap` key
## becomes a chip, at most one per word, in `ORDER`; no chip, no strip. `after` runs once a chip has been pressed.
func show_for(title: String, rows: Array, after: Callable = Callable()) -> void:
	for c in _row.get_children():
		_row.remove_child(c)
		c.queue_free()
	var by_word := {}
	for r in rows:
		var w := String((r as Dictionary).get("tap", ""))
		if w != "" and not by_word.has(w) and bool((r as Dictionary).get("enabled", true)):
			by_word[w] = r
	if by_word.is_empty():
		hide_chips()
		return
	_title = DccTheme.label(title, "text_dim", DccTheme.FS_SMALL)
	_title.custom_minimum_size = Vector2(0, MIN_TARGET)
	_title.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	_title.clip_text = true
	_title.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	_title.custom_minimum_size.x = 0
	_row.add_child(_title)
	for w in ORDER:
		if not by_word.has(w):
			continue
		var row: Dictionary = by_word[w]
		var b := Button.new()
		b.name = "Chip" + w
		b.text = w
		b.focus_mode = Control.FOCUS_NONE
		b.custom_minimum_size = Vector2(MIN_TARGET, MIN_TARGET)
		b.set_meta("row_id", String(row.get("id", "")))
		if bool(row.get("danger", false)):
			b.add_theme_color_override("font_color", DccTheme.c("accent"))
		var cb: Callable = row.get("callable", Callable())
		b.pressed.connect(func() -> void:
			hide_chips()
			if cb.is_valid():
				cb.call()
			if after.is_valid():
				after.call())
		_row.add_child(b)
	visible = true
	reset_size()


## Hides the strip. Safe to call any time.
func hide_chips() -> void:
	visible = false


## The chip words currently drawn, in order. For probes.
func chip_words() -> Array:
	var out: Array = []
	if not visible:
		return out
	for c in _row.get_children():
		if c is Button:
			out.append((c as Button).text)
	return out


## The chip button for `word`, or null. For probes.
func chip_button(word: String) -> Button:
	for c in _row.get_children():
		if c is Button and (c as Button).text == word:
			return c
	return null

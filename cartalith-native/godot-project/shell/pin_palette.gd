extends VBoxContainer
## The pin palette (Ruling FD, `PIN_PALETTE_SCOPE.md` PP-3/PP-4/PP-5): the right dock's body while CARTO's Pins tool (tool id
## `poi`, CL-11's Point of interest tool) is armed. A search field, a recents row (the last 8, per user), collapsible
## categories and an icon grid, in the current marker style's own pictures. Picking an icon ARMS it (the workspace's
## `_pin_state["kind"]`); a click on the map drops it; dragging the icon onto the map drops it too (a Godot drag with
## `{"pin_kind": name}` as its data, taken by `map_overlay.gd::_drop_data`).
##
## **What it is not.** It places nothing and calls no engine function: it reports a pick through its signals and the
## workspace (`cartography_workspace.gd::build_pin_palette`) does every write, so there is exactly one drop path. It holds no
## state of its own that survives a rebuild: the dock rebuilds its body on every selection change, so everything that must
## persist (the armed kind, the search text, which categories are closed, the free label) lives in the `state` dictionary
## the workspace passes in and this node mutates in place.
##
## ## The one table
##
## `CATEGORIES` is the single owner of which pin belongs where. Eight categories as the owner listed them, each pin in
## exactly one, derived from the glyph set's colour rows (gold settlements, blue built features, green nature, orange
## hazards and land) and the painted set's frames, with the military, sacred and water groups split out of the blue and
## green rows by what the picture is. A labelled judgement, checked by `_pinpalette_probe.gd`: the union of the table is
## exactly the two icon folders' union (62 names), no name twice.
##
## ## What each style shows
##
## The grid draws the pictures of the marker style's icon set (`DccSettings.marker_icon_set()`: Icons = glyph, Painted =
## painted). With **Pins** (drawn) or **Circles** there is no set; the palette then shows the painted pictures and says so,
## naming the style beside them (the scope's design), because those two styles draw a pictogram of the nearest kind, not
## the picture the palette shows. A name that has no file in the shown set (`rockfall` is glyph-only, `hazard` and `inn`
## painted-only) is drawn from the other set so the grid is complete and the pin can still be picked.
##
## ## PP-6: the touch forms
##
## **Tablet (>= 900 dp, touch):** the same dock section, verified with touch. Every tap target is at least 44 dp tall (the tiles
## are 44 x 52 dp; the category heads, the search field and the free-label buttons are floored at `TOUCH_MIN_DP`); a tap on a tile
## arms and a tap on the map drops (the ordinary click path, `cartography_workspace.gd::_on_poi_click`). On a touch device a drag
## from a tile only starts after a LONG PRESS (`LONG_PRESS_MS`, the quick bar's 0.4 s), so a finger that starts on a tile and
## moves at once scrolls the dock instead of dragging a pin; the drag itself is the same Godot drag as the desktop's.
##
## **Phone (< 900 dp):** the palette is the body of the phone tool sheet (`cartography_workspace.gd::_build_poi_tool_options_row`
## builds it into the sheet's options row, `DccShell.set_tool_options` phone-sizes it). Top to bottom: the armed line, the
## category chips (one line, scrolls sideways; "All" or one category), the search field, the grid, the free label. **The recents
## row is not in the sheet: it is the tool strip's second row (`tool_strip.gd`), the surface that stays on screen above the sheet
## at its PEEK detent, whose body is hidden there.** Tap a tile, then tap the map. Search spans every category and ignores the chip. Sizes are authored in dp here and scaled by `DccShell.phone_fit` (each fresh host is fitted by
## `_finish_form`, because the sheet's own walk runs only once, when the row is set).
##
## Must never: write a world, call the engine, or keep a reference to a node of an old dock body.

## `kind_picked(kind)`: the user armed (or, in change mode, chose) a picture kind. `free_picked(mark, label)`: they armed or chose
## a free-text kind. `change_cancelled`: they left change mode without choosing.
signal kind_picked(kind: String)
signal free_picked(mark: String, label: String)
signal change_cancelled

## The eight categories, in the order the dock lists them. `{id, title, kinds}`.
const CATEGORIES := [
	{"id": "settlements", "title": "Settlements", "kinds": ["capital", "city", "town", "village", "hamlet"]},
	{"id": "buildings", "title": "Buildings & trade", "kinds": ["tavern", "inn", "market", "smithy", "guild", "mine", "quarry",
		"salt_works", "trade_depot", "caravan_station"]},
	{"id": "military", "title": "Military", "kinds": ["castle", "fort", "watchtower", "fortified_pass", "fortified_crossing",
		"gate", "battlefield", "border_marker", "camp"]},
	{"id": "sacred", "title": "Sacred & historic", "kinds": ["temple", "shrine", "monument", "ruin", "tomb", "sacred_grove",
		"sacred_mountain", "ceremonial_site", "ancient_road", "historic_crossing"]},
	{"id": "water", "title": "Water", "kinds": ["harbour", "lake", "waterfall", "spring", "well", "river_confluence", "delta",
		"ford", "portage", "bridge"]},
	{"id": "terrain", "title": "Terrain", "kinds": ["mountain", "mountain_pass", "ridge", "cliff", "gorge", "rock_formation",
		"glacier", "volcano", "cave", "forest"]},
	{"id": "land", "title": "Land & hazards", "kinds": ["desert", "farmland", "rockfall", "hazard", "wildlife"]},
	{"id": "other", "title": "Other", "kinds": ["other", "notice", "road_junction"]},
]

## The pin pictures (`icon_texture`); preloaded, no `class_name`, like every other caller of this file.
const PinMarker := preload("res://shell/pin_marker.gd")

## The icon height the grid draws at (source px; the cached bucket at least this tall is used). 48 keeps a 40 px tile sharp
## on a 125 % display without keeping the 128 px source.
const TILE_ICON_H := 48.0
## One tile's size on a pointer form, px (a pin is 107-110 wide by 128 tall at source, so the ratio holds). The phone and
## tablet forms floor this at 44 dp (PP-6).
const TILE_W := 44.0
const TILE_H := 52.0
## The touch floor for every button and field of the palette, dp (`DccTheme.PHONE_TAP_MIN`'s 44, the canvas's TARGETS card).
const TOUCH_MIN_DP := 44.0
## A touch drag from a tile starts only after the finger has been down this long, ms (`quick_bar.gd::LONG_PRESS_SEC`, 0.4 s). A
## labelled judgement: shorter than the map's own hold (480 / 500 ms) so the two never race, long enough that a scroll flick is
## not a drag.
const LONG_PRESS_MS := 400

var _app
var _state: Dictionary = {}
## The grid host the search rebuilds in place, so typing never rebuilds the dock (and never loses the field's focus).
var _grid_host: VBoxContainer
var _recent_host: Container
var _chip_host: HBoxContainer
var _armed_label: Label
## True on the phone form (the sheet layout) and on any touch form (the 44 dp floors and the long-press drag).
var _phone := false
var _touch := false


## Every pin name in the table, in category order.
static func all_kinds() -> Array:
	var out: Array = []
	for c in CATEGORIES:
		out.append_array(c["kinds"])
	return out


## The category id of `kind`, or `""` for a name the table does not hold (a legacy `shop`, or a future icon): never a guess.
static func category_of(kind: String) -> String:
	for c in CATEGORIES:
		if (c["kinds"] as Array).has(kind):
			return String(c["id"])
	return ""


## The words for a pin name: `mountain_pass` is "Mountain pass". The same rule as `poi::kind_label_for` for names outside the
## owner's twelve; the palette is the only caller that shows a name that has no card row yet.
static func label_for(kind: String) -> String:
	var t := kind.replace("_", " ")
	return t.substr(0, 1).to_upper() + t.substr(1)


## The categories that hold a name matching `query` (a case-insensitive substring of the name's words OR of the category's
## title), as `[{id, title, kinds}]` with only the matching kinds. An empty query returns every category whole. A category
## whose title matches keeps all its kinds (typing "water" lists the Water category).
static func filter(query: String) -> Array:
	var q := query.strip_edges().to_lower()
	var out: Array = []
	for c in CATEGORIES:
		if q == "":
			out.append(c)
			continue
		var kinds: Array = []
		if String(c["title"]).to_lower().contains(q):
			kinds = (c["kinds"] as Array).duplicate()
		else:
			for k in c["kinds"]:
				if label_for(String(k)).to_lower().contains(q) or String(k).contains(q):
					kinds.append(k)
		if not kinds.is_empty():
			out.append({"id": c["id"], "title": c["title"], "kinds": kinds})
	return out


## The icon set the grid shows: the style's own, else the painted pictures (Pins and Circles have none).
static func shown_set() -> String:
	var s := DccSettings.marker_icon_set()
	return s if s != "" else "painted"


## The picture for `kind` in the shown set, falling back to the other set when the shown one has no file of that name; null
## only when neither has (cannot happen for a name in the table, which the probe asserts).
static func tile_texture(kind: String) -> Texture2D:
	var s := shown_set()
	var t := PinMarker.icon_texture(s, kind, TILE_ICON_H)
	if t == null:
		t = PinMarker.icon_texture("glyph" if s == "painted" else "painted", kind, TILE_ICON_H)
	return t


## Builds the palette into this node. `state` is the workspace's dictionary (keys: `kind`, `mark`, `free_label`, `query`,
## `closed`, `change_idx`, `change_name`); it is read and written in place. Call once, after adding the node to the tree is
## not required.
func setup(app, state: Dictionary) -> void:
	_app = app
	_state = state
	_phone = DccTheme.is_phone()
	_touch = DccTheme.is_touch()
	name = "PinPalette"
	add_theme_constant_override("separation", 6)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_build()


func _build() -> void:
	if _changing():
		var banner := DccTheme.label("Choose a new kind for “%s”" % String(_state.get("change_name", "this pin")), "accent", DccTheme.FS_SMALL)
		banner.name = "ChangeBanner"
		banner.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		add_child(banner)
		var cancel := DccWidgets.action(self, "Cancel (keep the current kind)", func() -> void: change_cancelled.emit())
		cancel.name = "ChangeCancel"
	if _phone:
		## The sheet is as wide as the screen but its options row is a horizontal scroller that sizes its child to the
		## child's own minimum, so without a minimum the palette is as wide as its widest control (~400 px of 1080).
		var host_w := 0.0
		if _app != null and _app.get("_phone_tool_sheet") != null:
			host_w = (_app.get("_phone_tool_sheet") as Control).size.x
		if host_w <= 0.0:
			host_w = float(DisplayServer.window_get_size().x)
		custom_minimum_size.x = maxf(0.0, host_w - 2.0 * (_app.phone_px(14.0) if _app != null and _app.has_method("phone_px") else 0))
		## Already in physical px: the sheet's own `phone_fit` walk (which meets this node first) must not scale it again.
		set_meta("_phone_fitted", true)
	var style := DccSettings.marker_style()
	var cap := "Pins on the map are drawn in the %s style." % style
	if DccSettings.marker_icon_set() == "":
		cap = "Map style: %s. It has no pictures of its own, so the palette shows the painted ones." % style.capitalize()
	var note := DccTheme.label(cap, "text_ghost", DccTheme.FS_MICRO)
	note.name = "StyleNote"
	note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	if not _phone:
		add_child(note)

	var search := LineEdit.new()
	search.name = "PinSearch"
	search.placeholder_text = "Search pins"
	search.clear_button_enabled = true
	search.text = String(_state.get("query", ""))
	DccWidgets.well(search)
	search.text_changed.connect(func(t: String) -> void:
		_state["query"] = t
		_fill_grid())

	_armed_label = DccTheme.label("", "text_secondary", DccTheme.FS_SMALL)
	_armed_label.name = "ArmedPin"
	_armed_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	if _phone:
		## The sheet's order (see the file header): recents, armed line, chips, search, grid. The recents row and the chips
		## are built by `_build_phone_top`; the armed line goes between them, so it is the first thing at PEEK when there
		## are no recents yet.
		add_child(_armed_label)
		_refresh_armed()
		_build_chips()
		add_child(search)
		add_child(note)
		_grid_host = VBoxContainer.new()
		_grid_host.name = "Categories"
		_grid_host.add_theme_constant_override("separation", 4)
		add_child(_grid_host)
		_fill_recents()
		_fill_grid()
		if not _changing_landmark():
			_build_free()
		_finish_form(self)
		return
	add_child(search)
	add_child(_armed_label)
	_refresh_armed()

	var rh := DccTheme.label("RECENT", "text_faint", DccTheme.FS_HEADER)
	rh.name = "RecentHeader"
	add_child(rh)
	var rflow := HFlowContainer.new()
	rflow.add_theme_constant_override("h_separation", 4)
	rflow.add_theme_constant_override("v_separation", 4)
	_recent_host = rflow
	_recent_host.name = "Recents"
	add_child(_recent_host)
	_fill_recents()

	_grid_host = VBoxContainer.new()
	_grid_host.name = "Categories"
	_grid_host.add_theme_constant_override("separation", 4)
	add_child(_grid_host)
	_fill_grid()
	if not _changing_landmark():
		_build_free()
	_finish_form(self)


## Change mode for a user pin (`change_idx`) or a generated landmark (`change_lm`, Ruling FD item 3): the next pick CHANGES
## something instead of arming the next drop, so the armed line is hidden and the Cancel row shows.
func _changing() -> bool:
	return int(_state.get("change_idx", -1)) >= 0 or _changing_landmark()


## Change mode for a generated landmark. Its kind must be a picture kind: the free-text label (a `?` or `!` with the user's
## words) is a user pin's own, so the free-label group is not built for a landmark.
func _changing_landmark() -> bool:
	return int(_state.get("change_lm", -1)) >= 0


## One authored dp in this node's own pixels: the phone's scale on the phone form (the sheet lays out in physical pixels), 1 on
## the desktop and tablet (the tablet's viewport is content-scaled, so a dp is a layout unit).
func _unit() -> float:
	if _phone and _app != null and _app.has_method("phone_scale"):
		return float(_app.phone_scale())
	return 1.0


## Floors every button and field under `node` at `TOUCH_MIN_DP` tall when the form is a touch one, and on the phone hands the
## subtree to `DccShell.phone_fit` (idempotent by its own meta flag, so the sheet's later walk skips what is already fitted).
func _finish_form(node: Node) -> void:
	if _touch:
		_floor_touch(node)
	if _phone and _app != null and _app.has_method("phone_fit"):
		_app.phone_fit(node, _unit(), true)


func _floor_touch(node: Node) -> void:
	for c in node.get_children():
		if c is BaseButton or c is LineEdit:
			var ctl := c as Control
			if ctl.custom_minimum_size.y < TOUCH_MIN_DP and not ctl.has_meta("_phone_fitted"):
				ctl.custom_minimum_size.y = TOUCH_MIN_DP
		_floor_touch(c)


## The phone's category chips: "All" and the eight categories, one line that scrolls sideways. A chip narrows the grid to that
## category (search ignores it and spans every category).
func _build_chips() -> void:
	var sc := ScrollContainer.new()
	sc.name = "CatChipsScroll"
	sc.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	sc.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	sc.add_theme_stylebox_override("panel", DccTheme.empty())
	_chip_host = HBoxContainer.new()
	_chip_host.name = "CatChips"
	_chip_host.add_theme_constant_override("separation", 4)
	sc.add_child(_chip_host)
	add_child(sc)
	_fill_chips()


func _fill_chips() -> void:
	for c in _chip_host.get_children():
		_chip_host.remove_child(c)
		c.queue_free()
	var cur := String(_state.get("cat", "all"))
	var defs: Array = [{"id": "all", "title": "All"}]
	defs.append_array(CATEGORIES)
	for d in defs:
		var cid := String(d["id"])
		var b := Button.new()
		b.name = "Chip_" + cid
		b.text = String(d["title"])
		b.focus_mode = Control.FOCUS_NONE
		b.toggle_mode = false
		b.custom_minimum_size = Vector2(TOUCH_MIN_DP, TOUCH_MIN_DP)
		b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
		var on := cid == cur
		b.add_theme_color_override("font_color", DccTheme.c("accent") if on else DccTheme.c("text"))
		var sb := DccWidgets.box("accent" if on else "border", "accent_wash" if on else "", 10, 6)
		b.add_theme_stylebox_override("normal", sb)
		b.add_theme_stylebox_override("pressed", sb)
		b.add_theme_stylebox_override("hover", sb)
		b.pressed.connect(func() -> void:
			_state["cat"] = cid
			_fill_chips()
			_fill_grid())
		_chip_host.add_child(b)
	_finish_form(_chip_host)


func _refresh_armed() -> void:
	if _armed_label == null:
		return
	var k := String(_state.get("kind", ""))
	## In change mode the pick CHANGES a pin, it does not arm the next drop, so the "Armed" line would mislead: it is hidden.
	_armed_label.visible = not _changing()
	var verb := "tap" if _touch else "click"
	var stop := "" if _touch else " · Esc stops"
	if k == "":
		_armed_label.text = "Pick a pin, then %s the map to drop it%s" % [verb, stop]
	elif String(_state.get("free_label", "")) != "" and (k == "other" or k == "notice"):
		_armed_label.text = "Armed: “%s” (%s)" % [String(_state["free_label"]), "?" if k == "other" else "!"]
	else:
		_armed_label.text = "Armed: %s · %s the map to drop it%s" % [label_for(k), verb, stop]


## The recents row: up to 8 tiles from `DccSettings.pin_recents()`; hidden (header and all) when there are none.
func _fill_recents() -> void:
	if _recent_host == null:
		return   # the phone form: its recents are the tool strip's row, not the sheet's
	for c in _recent_host.get_children():
		_recent_host.remove_child(c)
		c.queue_free()
	var rec: Array = DccSettings.pin_recents()
	var hdr := get_node_or_null("RecentHeader")
	if hdr != null:
		(hdr as Control).visible = not rec.is_empty()
	_recent_host.visible = not rec.is_empty()
	for k in rec:
		_recent_host.add_child(_tile(String(k), "Recent_"))
	_finish_form(_recent_host)


## Rebuilds the category grid for the current query. A category with no match is absent; with a query active every matching
## category is open (the point of searching is to see the match), otherwise the user's own open/closed choice holds.
func _fill_grid() -> void:
	for c in _grid_host.get_children():
		_grid_host.remove_child(c)
		c.queue_free()
	var q := String(_state.get("query", "")).strip_edges()
	var closed: Dictionary = _state.get("closed", {})
	var only := String(_state.get("cat", "all")) if _phone and q == "" else "all"
	for c in filter(q):
		var cid := String(c["id"])
		if only != "all" and cid != only:
			continue
		var is_open: bool = (q != "") or not closed.has(cid) or only != "all"
		var head := Button.new()
		head.name = "Cat_" + cid
		head.flat = true
		head.focus_mode = Control.FOCUS_NONE
		head.alignment = HORIZONTAL_ALIGNMENT_LEFT
		head.add_theme_font_size_override("font_size", DccTheme.FS_HEADER)
		head.add_theme_font_override("font", DccTheme.mono(2, true))
		head.add_theme_color_override("font_color", DccTheme.c("text_faint"))
		head.add_theme_stylebox_override("normal", DccTheme.empty())
		head.add_theme_stylebox_override("hover", DccTheme.empty())
		head.add_theme_stylebox_override("pressed", DccTheme.empty())
		head.text = "%s %s (%d)" % [DccIcons.SYMBOLS["caret"] if is_open else DccIcons.SYMBOLS["expand"], String(c["title"]).to_upper(), (c["kinds"] as Array).size()]
		if only != "all":
			head.visible = false   # one category chosen by its chip: the chip is the heading
		_grid_host.add_child(head)
		var flow := HFlowContainer.new()
		flow.name = "Grid_" + cid
		flow.add_theme_constant_override("h_separation", 4)
		flow.add_theme_constant_override("v_separation", 4)
		flow.visible = is_open
		for k in c["kinds"]:
			flow.add_child(_tile(String(k), "Pin_"))
		_grid_host.add_child(flow)
		head.pressed.connect(func() -> void:
			if q != "":
				return   # a search shows every match open; the toggle applies to the unfiltered list
			var cl: Dictionary = _state.get("closed", {})
			if cl.has(cid):
				cl.erase(cid)
			else:
				cl[cid] = true
			_state["closed"] = cl
			_fill_grid())
	if _grid_host.get_child_count() == 0:
		var none := DccTheme.label("No pin matches “%s”." % q, "text_ghost", DccTheme.FS_SMALL)
		none.name = "NoMatch"
		_grid_host.add_child(none)
	_finish_form(_grid_host)


## One icon tile: a flat button holding the picture, lit while armed, with the name as its tooltip, that drags (`{"pin_kind"}`)
## onto the map. The button's `name` is `prefix + kind`, which the probes press by.
func _tile(kind: String, prefix: String) -> Button:
	var b := Button.new()
	b.name = prefix + kind
	b.toggle_mode = false
	b.focus_mode = Control.FOCUS_NONE
	b.flat = true
	b.icon = tile_texture(kind)
	b.expand_icon = true
	b.icon_alignment = HORIZONTAL_ALIGNMENT_CENTER
	b.vertical_icon_alignment = VERTICAL_ALIGNMENT_CENTER
	b.custom_minimum_size = Vector2(TILE_W, TILE_H)
	b.tooltip_text = label_for(kind)
	b.set_meta("pin_kind", kind)
	var armed := String(_state.get("kind", "")) == kind and String(_state.get("free_label", "")) == ""
	var sb := DccWidgets.box("accent" if armed else "border", "accent_wash" if armed else "", 3, 3)
	b.add_theme_stylebox_override("normal", sb)
	b.add_theme_stylebox_override("pressed", sb)
	b.add_theme_stylebox_override("hover", DccWidgets.box("accent", "line_soft", 3, 3))
	b.pressed.connect(func() -> void: kind_picked.emit(kind))
	## Touch only: remember when the finger went down, so the drag below can tell a long press from a scroll flick.
	b.gui_input.connect(func(ev: InputEvent) -> void:
		if ev is InputEventMouseButton and (ev as InputEventMouseButton).button_index == MOUSE_BUTTON_LEFT \
				and (ev as InputEventMouseButton).pressed:
			b.set_meta("press_ms", Time.get_ticks_msec()))
	var touch := _touch
	var drag := func(_at: Variant) -> Variant:
		## On a touch device the drag needs a long press (`LONG_PRESS_MS`); a mouse drags at once.
		if touch and Time.get_ticks_msec() - int(b.get_meta("press_ms", 0)) < LONG_PRESS_MS:
			return null
		var prev := TextureRect.new()
		prev.texture = b.icon
		prev.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		prev.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
		var tsz := Vector2(TILE_W, TILE_H) * _unit()
		prev.custom_minimum_size = tsz
		prev.size = tsz
		prev.modulate = Color(1, 1, 1, 0.85)
		var holder := Control.new()
		holder.add_child(prev)
		prev.position = Vector2(-tsz.x * 0.5, -tsz.y)   # the tip under the pointer
		b.set_drag_preview(holder)
		return {"pin_kind": kind}
	b.set_drag_forwarding(drag, Callable(), Callable())
	return b


## The free-text kind: a label and the choice of mark (a question mark or an exclamation mark), Ruling FD item 2. "Use" arms it
## for the next drop, or, in change mode, applies it to the pin being changed.
func _build_free() -> void:
	var grp := DccWidgets.group(self, "Your own label", false)
	grp.name = "FreeKind"
	var le := LineEdit.new()
	le.name = "FreeLabel"
	le.placeholder_text = "e.g. Beehives"
	le.text = String(_state.get("free_label", ""))
	DccWidgets.well(le)
	grp.add_child(le)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)
	grp.add_child(row)
	var marks := {}
	for m in [["other", "?  question mark"], ["notice", "!  exclamation mark"]]:
		var mb := DccWidgets.segment(row, String(m[1]), Callable())
		mb.name = "Mark_" + String(m[0])
		marks[String(m[0])] = mb
	var paint := func() -> void:
		for mk in marks:
			(marks[mk] as Button).add_theme_color_override("font_color", DccTheme.c("accent") if String(_state.get("mark", "other")) == mk else DccTheme.c("text"))
	for mk in marks:
		(marks[mk] as Button).pressed.connect(func() -> void:
			_state["mark"] = mk
			paint.call())
	paint.call()
	var use := DccWidgets.action(grp, "Use this label", func() -> void:
		var t := le.text.strip_edges()
		if t == "":
			return
		_state["free_label"] = t
		free_picked.emit(String(_state.get("mark", "other")), t))
	use.name = "FreeUse"


## Repaints the armed readout, the recents and the tiles' lit state after the workspace changed `state`.
func refresh() -> void:
	_refresh_armed()
	_fill_recents()
	_fill_grid()

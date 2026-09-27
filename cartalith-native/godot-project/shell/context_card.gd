extends PopupPanel
## `MAP_CONTEXT_SCOPE.md` §4 and §9.1 -- **the context card**, CM-2. The
## desktop and tablet presenter for `context_broker.gd`'s merged rows; it
## replaced CX-01's `PopupMenu` there. The phone keeps that `PopupMenu`,
## re-presented as its L4 sheet (`phone_present_popup`) -- the phone's own
## noun surface is CM-5.
##
## **Why a `PopupPanel` and not a `PopupMenu`** (§9.1): a `PopupMenu` cannot
## hold a header readout, a disabled row's reason on the row itself, or an
## inline parameter row. Every one of those is a §4.1 requirement.
##
## ## Structure, top to bottom (§4.1's order, which never changes)
##
##   header    the object the verbs act on (`Vhal Serai · town`), or `Here`;
##             second line: elevation · biome · cell → km (`sample_cell`)
##   filter    only while the user is typing: the text, and nothing else
##   Select ▸  when two or more things are under the pointer; opens the hit
##             list (the nearest `SELECT_CAP`) in place of the sections
##   sections  DRAFT · TOOL · OBJECT · PLACE HERE · GO / MEASURE · INFO,
##             each only when it has a row. Within a section a provider's
##             order is kept, except that `danger` rows sort last behind a
##             rule (§3's `danger`)
##
## A row with `children` opens **one** submenu in place: the card's body is
## replaced by a `‹ BACK · <title>` band and the children (§4.2, "no sixth
## level"). Select ▸ is the same mechanism with the hit list as children.
##
## ## Rows
##
##   action    label left, hint right (`▸` for a submenu, or the row's
##             `shortcut`). Click, or Enter on the highlighted row
##   disabled  label in `text_dim`, the reason on its own line under it in
##             `accent_hover` -- always drawn, never only a tooltip (§4.2).
##             Not reachable by the keyboard: there is nothing to run
##   param     label, `−`, value, `+` (§4.1's Tool section, inline). The
##             action's `param` is `{"value": Callable -> String, "step":
##             Callable(dir: int)}`; Left / Right step the highlighted one
##
## ## Keyboard (§4.2)
##
## Up / Down move the highlight over the rows that can act; Enter runs it
## (or opens its submenu); Right opens a submenu, Left / Backspace-on-an-empty-
## filter goes back from one; Left / Right step a param row; any printable
## character filters the rows by label (a submenu row also matches on its
## children's labels); Backspace edits the filter; Esc closes (the `Popup`'s
## own `ui_cancel`). Handled on `window_input`, the event path an embedded
## popup actually receives.
##
## ## Palette
##
## Built fresh on every `open()` from `DccTheme.c()`, so it always matches the
## palette at the moment it is seen (the reason `style_popup()` restyles on
## `about_to_popup`). The design canvas (`design/map-context-2026-09-25/
## Main.dc.html`, the card's `<div ... width: 286px; background: #16181a`)
## is dark-only; the token for each of its literals is chosen for the pair
## that matters, text against the card's own ground, and measured in both
## palettes by `_ctxcard_probe.gd`:
##
##   canvas             token           why
##   #16181a ground     `raised`        #17191a dark -- one step from the canvas
##   #555b60 border     `text_ghost`    #5f6468; `border` (16 % white) is the
##                                      dimmer outline the owner's contrast
##                                      pass replaced
##   #e8ebec / #eef0f1  `text_bright`   title and row labels
##   #b8bdc1 / #b0b5b9  `text_secondary` subtitle, section bands, hints
##   #a0a6aa disabled   `text_dim`      still >= 4.5 : 1 on the ground
##   #ebc080 reason     `accent_hover`
##   #f29c8c danger     `block`
##   #e0a34a Select/BACK `accent` on dark, `accent_hover` on light
##                                      (`_accent_ink()` has the measurement)
##   #3a3e42 rules      `line`

## §9.1's cap on the Select ▸ list: "nearest first, capped at 8".
const SELECT_CAP := 8

const SECTION_TITLES := {
	"draft": "DRAFT", "tool": "TOOL", "object": "OBJECT",
	"place": "PLACE HERE", "go": "GO / MEASURE", "info": "INFO",
}

var app
## What `open()` was given. `_all_hits` is every hit under the pointer, which
## Select ▸ lists even after a pick narrowed the request's own `hits` to one.
var _req: Dictionary = {}
var _actions: Array = []
var _all_hits: Array = []
## Called with the chosen hit when a Select ▸ entry is run; the broker
## re-resolves for that object and calls `open()` again.
var _on_select: Callable = Callable()
var _anchor := Vector2.ZERO
## CM-4 (`MAP_CONTEXT_SCOPE.md` §7.1): `""` (default) keeps `_target_rect()`'s
## plain pointer-relative placement (right of the anchor, flipped only at a
## screen edge) -- every presentation except the tablet's touch-hold. `"left"`
## / `"right"` is the side away from the dominant hand for that one gesture;
## still flips to the other side rather than clipping if THAT side does not
## fit either, so the edge-flip guarantee holds regardless of hand.
var _dock_side := ""
## Coordinator review, 2026-09-27 -- see `open()`'s own doc. 0.0 outside the
## ring+card-together presentations.
var _ring_clear := 0.0
## Coordinator review, 2026-09-27 (overlap follow-up) -- see `set_ring_clear()`'s
## own doc. `_sub_x` is the open SUB-ring's own centre-x, in the same space as
## `_anchor`; `_sub_clear` is its `radius + slot * 0.5`. Both 0.0 whenever no
## sub-ring is open beside this card.
var _sub_x := 0.0
var _sub_clear := 0.0
## `{}` for the sections; else `{"title": String, "rows": Array}` -- the one
## submenu level §4.2 allows.
var _sub: Dictionary = {}
var _filter := ""
## The rows the keyboard can reach, in drawn order: `{node, kind, action}`.
var _nav: Array = []
var _focus := -1
var _body: VBoxContainer
var _scroll: ScrollContainer


func setup(p_app) -> void:
	app = p_app
	name = "ContextCard"
	wrap_controls = true
	transparent_bg = true
	_scroll = ScrollContainer.new()
	## Horizontal off, so the body's width IS the card's width (MISTAKES.md:
	## a disabled axis folds the child's minimum into the container's -- here
	## that is wanted); vertical on, so a long card scrolls inside the screen.
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	## Empty, or the shell theme's own `ScrollContainer` box paints over the
	## card's ground (measured: the card drew `bg` #0d0e0f, not `raised`).
	_scroll.add_theme_stylebox_override("panel", DccTheme.empty())
	add_child(_scroll)
	_body = VBoxContainer.new()
	_body.add_theme_constant_override("separation", 0)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_scroll.add_child(_body)
	window_input.connect(_on_window_input)


## Accent-coloured TEXT (Select ▸, BACK, a param's value). `accent` itself
## on dark -- the canvas's #e0a34a, 8.7 : 1 on the card -- but `accent_hover`
## on light, because the light `accent` #a4650f measured **4.22 : 1** on the
## light card (`_ctxcard_probe.gd` leg P), under the 4.5 floor. The light
## `accent_hover` #8a5309 is the same hue a step darker.
static func _accent_ink() -> String:
	return "accent" if DccTheme.is_dark() else "accent_hover"


func _touch() -> bool:
	return DccTheme.is_tablet()


## The figures the canvas authors per density: `Main.dc.html` (CW 286, ROWH
## 28, 12 px rows, 12 px padding) and `Tablet.dc.html` (CW 320, ROWH 44, 14 px
## rows, 14 px padding).
func _m(key: String) -> int:
	var t := _touch()
	match key:
		"w": return 320 if t else 286
		"row": return 44 if t else 28
		"param_row": return 48 if t else 28
		"pad": return 14 if t else 12
		"fs_row": return 14 if t else 12
		"fs_title": return 16 if t else 13
		"fs_sub": return 13 if t else 11
		"fs_band": return 11 if t else 9
		"fs_hint": return 13 if t else 10
		"step_btn": return 36 if t else 22
	return 0


## `anchor` is in the space of the viewport this card pops into (the one
## `app` is in). `actions` is the broker's merge, already in §4.1's order.
## `ring_clear` (coordinator review, 2026-09-27): how far, in px, `_target_rect()`
## must additionally clear `anchor` before it starts offsetting the card --
## `context_broker.gd::present()`'s own header has the full reasoning. 0.0
## (default) is the plain-click case with no ring open beside this card,
## which keeps `_target_rect()`'s original 10/6 px offset exactly as it was.
func open(req: Dictionary, actions: Array, anchor: Vector2, on_select: Callable,
		dock_side: String = "", ring_clear: float = 0.0, sub_x: float = 0.0,
		sub_clear: float = 0.0) -> void:
	_req = req
	_actions = actions
	_all_hits = req.get("all_hits", req.get("hits", []))
	_on_select = on_select
	_anchor = anchor
	_dock_side = dock_side
	_ring_clear = ring_clear
	_sub_x = sub_x
	_sub_clear = sub_clear
	_sub = {}
	_filter = ""
	_rebuild()
	_place()


## Coordinator review, 2026-09-27 (overlap follow-up). A sub-ring re-centres
## on its own parent slot (`radial_ring.gd::_open_sub()`), which can reach
## past the top-level ring's own footprint toward this card -- `open()`'s
## `ring_clear` alone only ever widened the dock offset for the MAIN ring,
## centred at `_anchor`. `context_broker.gd` calls this the moment a sub-ring
## opens while the card is already showing (the only transition that matters:
## `radial_ring.gd` never closes a sub-ring without closing the whole ring, so
## there is no "sub closed, ring still open" case to react to). `_reclamp()`
## re-targets the ALREADY-OPEN card at its own current size -- a translate,
## not a rebuild -- so a call that does not actually need to move the card
## (the sub landed on a side the card was never near) is a no-op in practice,
## not just skipped by this early-return on an unchanged input.
func set_ring_clear(ring_clear: float, sub_x: float, sub_clear: float) -> void:
	if _ring_clear == ring_clear and _sub_x == sub_x and _sub_clear == sub_clear:
		return
	_ring_clear = ring_clear
	_sub_x = sub_x
	_sub_clear = sub_clear
	if visible:
		_reclamp()


# -- Probe-facing readouts ----------------------------------------------------

## The drawn rows, top to bottom, as `{kind, text, ...}` -- read off the
## nodes themselves, not off `_actions`, so a row that was dropped or
## mis-drawn is visible here. `kind` is `title`, `subtitle`, `filter`,
## `select`, `back`, `band`, `rule`, `action`, `disabled` (with `reason`) or
## `param` (with `value`).
func drawn_rows() -> Array:
	var out: Array = []
	for n in _body.get_children():
		var d: Dictionary = n.get_meta("card_row", {})
		if d.is_empty():
			continue
		var e := d.duplicate()
		e["node"] = n
		out.append(e)
	return out


func focused_row() -> Control:
	return _nav[_focus]["node"] if _focus >= 0 and _focus < _nav.size() else null


func in_submenu() -> bool:
	return not _sub.is_empty()


# -- Build --------------------------------------------------------------------

func _rebuild() -> void:
	for c in _body.get_children():
		_body.remove_child(c)
		c.queue_free()
	_nav.clear()
	_focus = -1
	var sb := StyleBoxFlat.new()
	sb.bg_color = DccTheme.c("raised")
	sb.border_color = DccTheme.c("text_ghost")
	sb.set_border_width_all(1)
	sb.shadow_color = Color(0, 0, 0, 0.5) if DccTheme.is_dark() else Color(0.137, 0.141, 0.122, 0.16)
	sb.shadow_size = 18
	sb.shadow_offset = Vector2(0, 10)
	sb.content_margin_left = 1
	sb.content_margin_right = 1
	sb.content_margin_top = 1
	sb.content_margin_bottom = 1
	add_theme_stylebox_override("panel", sb)
	_body.custom_minimum_size.x = _m("w") - 2

	_build_header()
	if not _filter.is_empty():
		var f := _text_row("filter", "filter · %s" % _filter, "text_bright", _m("fs_hint"), true)
		f.set_meta("card_row", {"kind": "filter", "text": _filter})
	if _sub.is_empty():
		_build_main()
	else:
		_build_sub()
	if not _nav.is_empty():
		_set_focus(0)


func _build_header() -> void:
	var head := _header()
	var box := MarginContainer.new()
	box.add_theme_constant_override("margin_left", _m("pad"))
	box.add_theme_constant_override("margin_right", _m("pad"))
	box.add_theme_constant_override("margin_top", 10)
	box.add_theme_constant_override("margin_bottom", 8)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 4)
	box.add_child(col)
	var t := DccTheme.label(String(head["title"]), "text_bright", _m("fs_title"))
	t.name = "Title"
	col.add_child(t)
	## Two lines, not one: the canvas's readout is a single line because its
	## numbers are short; this one carries a grid cell and two distances, and
	## on one line it set the card's width (measured 452 px against 286).
	var lines: Array = head["subtitle"]
	for i in lines.size():
		var s := DccTheme.mono_label(String(lines[i]), "text_secondary", _m("fs_sub"))
		s.name = "Subtitle" if i == 0 else "Subtitle%d" % (i + 1)
		col.add_child(s)
	box.set_meta("card_row", {"kind": "header", "text": head["title"], "subtitle": " · ".join(lines)})
	_body.add_child(box)


func _build_main() -> void:
	if _all_hits.size() >= 2 and _filter.is_empty():
		_rule()
		var n := _all_hits.size()
		var sel := _action_row("Select — %d objects here" % n, DccIcons.SYMBOLS["submenu"], _accent_ink(), true)
		sel.set_meta("card_row", {"kind": "select", "text": "Select — %d objects here" % n})
		_nav.append({"node": sel, "kind": "select"})
	var by_section: Dictionary = {}
	var order: Array = []
	for a in _actions:
		var sec := String((a as Dictionary).get("section", ""))
		if not by_section.has(sec):
			by_section[sec] = []
			order.append(sec)
		by_section[sec].append(a)
	for sec in order:
		var rows: Array = []
		for a in by_section[sec]:
			if _matches(a):
				rows.append(a)
		if rows.is_empty():
			continue
		## Danger last, stably: §3's `danger` "sort last in their section".
		var safe: Array = rows.filter(func(a): return not bool(a.get("danger", false)))
		var risky: Array = rows.filter(func(a): return bool(a.get("danger", false)))
		_rule()
		_band(_section_title(sec))
		for a in safe:
			_row_for(a)
		if not risky.is_empty():
			if not safe.is_empty():
				_rule(true)
			for a in risky:
				_row_for(a)


func _build_sub() -> void:
	_rule()
	var back := _action_row("‹ BACK  ·  %s" % String(_sub["title"]), "", _accent_ink(), true, true)
	back.set_meta("card_row", {"kind": "back", "text": String(_sub["title"])})
	_nav.append({"node": back, "kind": "back"})
	for a in _sub["rows"]:
		if _matches(a):
			_row_for(a, true)


func _section_title(sec: String) -> String:
	var t := String(SECTION_TITLES.get(sec, sec.to_upper()))
	if sec == "tool":
		t += " · " + String(_req.get("armed_tool", "")).to_upper()
	## The canvas's own CARTO band: its View field ▸ and Style preset ▸ sit
	## with the place-here rows, because in CARTO "place" means "show".
	if sec == "place" and String(_req.get("domain", "")) == "cartography":
		t = "PLACE & SHOW"
	return t


func _matches(a: Dictionary) -> bool:
	if _filter.is_empty():
		return true
	var needle := _filter.to_lower()
	if String(a.get("label", "")).to_lower().contains(needle):
		return true
	for c in a.get("children", []):
		if String((c as Dictionary).get("label", "")).to_lower().contains(needle):
			return true
	return false


func _row_for(a: Dictionary, indent: bool = false) -> void:
	var label := String(a.get("label", ""))
	if a.has("param"):
		_param_row(a)
		return
	if not bool(a.get("enabled", true)):
		_disabled_row(a)
		return
	var hint := String(a.get("shortcut", ""))
	if a.has("children"):
		hint = DccIcons.SYMBOLS["submenu"]
	var ink := "block" if bool(a.get("danger", false)) else "text_bright"
	var row := _action_row(label, hint, ink, false, false, indent)
	row.set_meta("card_row", {"kind": "action", "text": label, "id": String(a.get("id", "")),
		"submenu": a.has("children")})
	_nav.append({"node": row, "kind": "action", "action": a})


func _rule(soft: bool = false) -> void:
	var r := ColorRect.new()
	r.color = DccTheme.c("line_soft" if soft else "line")
	r.custom_minimum_size = Vector2(0, 1)
	r.mouse_filter = Control.MOUSE_FILTER_IGNORE
	r.set_meta("card_row", {"kind": "rule", "text": ""})
	_body.add_child(r)


func _band(text: String) -> void:
	var m := MarginContainer.new()
	m.add_theme_constant_override("margin_left", _m("pad"))
	m.add_theme_constant_override("margin_top", 6)
	m.add_theme_constant_override("margin_bottom", 3)
	var l := DccTheme.mono_label(text, "text_secondary", _m("fs_band"), 2, true)
	m.add_child(l)
	m.set_meta("card_row", {"kind": "band", "text": text})
	_body.add_child(m)


func _text_row(_kind: String, text: String, token: String, fs: int, mono: bool) -> Control:
	var m := MarginContainer.new()
	m.add_theme_constant_override("margin_left", _m("pad"))
	m.add_theme_constant_override("margin_right", _m("pad"))
	m.add_theme_constant_override("margin_top", 4)
	m.add_theme_constant_override("margin_bottom", 4)
	m.add_child(DccTheme.mono_label(text, token, fs) if mono else DccTheme.label(text, token, fs))
	_body.add_child(m)
	return m


## One clickable row. A `PanelContainer` whose `panel` box is swapped for the
## highlight: the row is a hit area and a ground, not a `Button` -- a Button
## cannot right-align a hint, and `flat` would void the box (MISTAKES.md).
func _action_row(text: String, hint: String, ink: String, mono: bool,
		band: bool = false, indent: bool = false) -> PanelContainer:
	var row := PanelContainer.new()
	row.custom_minimum_size.y = _m("row")
	row.mouse_filter = Control.MOUSE_FILTER_STOP
	row.set_meta("rest", DccTheme.flat(DccTheme.c("sunken")) if band else DccTheme.empty())
	row.add_theme_stylebox_override("panel", row.get_meta("rest"))
	var h := HBoxContainer.new()
	h.add_theme_constant_override("separation", 8)
	var m := MarginContainer.new()
	m.add_theme_constant_override("margin_left", _m("pad") + (10 if indent else 0))
	m.add_theme_constant_override("margin_right", _m("pad"))
	m.mouse_filter = Control.MOUSE_FILTER_IGNORE
	m.add_child(h)
	row.add_child(m)
	var l: Label = DccTheme.mono_label(text, ink, _m("fs_hint") + 1, 0, true) if mono \
		else DccTheme.label(text, ink, _m("fs_row"))
	l.name = "Text"
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	h.add_child(l)
	if hint != "":
		var hl := DccTheme.mono_label(hint, _accent_ink() if mono else "text_secondary", _m("fs_hint"))
		hl.name = "Hint"
		hl.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		hl.mouse_filter = Control.MOUSE_FILTER_IGNORE
		h.add_child(hl)
	row.mouse_entered.connect(func() -> void: _hover(row))
	row.gui_input.connect(func(ev: InputEvent) -> void:
		var mb := ev as InputEventMouseButton
		if mb != null and mb.button_index == MOUSE_BUTTON_LEFT and not mb.pressed:
			## Accepted first: activating can rebuild the card, which frees
			## this very row.
			row.accept_event()
			_hover(row)
			_activate_focused())
	_body.add_child(row)
	return row


func _disabled_row(a: Dictionary) -> void:
	var m := MarginContainer.new()
	m.add_theme_constant_override("margin_left", _m("pad"))
	m.add_theme_constant_override("margin_right", _m("pad"))
	m.add_theme_constant_override("margin_top", 5)
	m.add_theme_constant_override("margin_bottom", 5)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 2)
	m.add_child(col)
	var l := DccTheme.label(String(a.get("label", "")), "text_dim", _m("fs_row"))
	l.name = "Text"
	col.add_child(l)
	## §4.2: the reason is **required** when a row is disabled. An empty one is
	## a provider bug, and is drawn as one rather than as a blank line.
	var why := String(a.get("reason", ""))
	var r := DccTheme.mono_label(why if why != "" else "(no reason given -- provider bug)",
		"accent_hover", _m("fs_hint"))
	r.name = "Reason"
	r.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	r.custom_minimum_size.x = _m("w") - 2 * _m("pad") - 2
	col.add_child(r)
	m.tooltip_text = why
	m.set_meta("card_row", {"kind": "disabled", "text": String(a.get("label", "")),
		"reason": why, "id": String(a.get("id", ""))})
	_body.add_child(m)


func _param_row(a: Dictionary) -> void:
	var row := PanelContainer.new()
	row.custom_minimum_size.y = _m("param_row")
	row.mouse_filter = Control.MOUSE_FILTER_STOP
	row.set_meta("rest", DccTheme.empty())
	row.add_theme_stylebox_override("panel", row.get_meta("rest"))
	var m := MarginContainer.new()
	m.add_theme_constant_override("margin_left", _m("pad"))
	m.add_theme_constant_override("margin_right", _m("pad"))
	var h := HBoxContainer.new()
	h.add_theme_constant_override("separation", 6)
	m.add_child(h)
	row.add_child(m)
	var l := DccTheme.label(String(a.get("label", "")), "text", _m("fs_row"))
	l.name = "Text"
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	h.add_child(l)
	var v := DccTheme.mono_label("", _accent_ink(), _m("fs_hint") + 1)
	v.name = "Value"
	v.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	v.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	v.custom_minimum_size.x = 64
	var minus := _step_button("−", "Decrease")
	var plus := _step_button("+", "Increase")
	h.add_child(minus)
	h.add_child(v)
	h.add_child(plus)
	var entry := {"node": row, "kind": "param", "action": a, "value_label": v}
	minus.pressed.connect(func() -> void: _step(entry, -1))
	plus.pressed.connect(func() -> void: _step(entry, 1))
	row.mouse_entered.connect(func() -> void: _hover(row))
	_body.add_child(row)
	_nav.append(entry)
	_refresh_param(entry)


func _step_button(glyph: String, tip: String) -> Button:
	var b := Button.new()
	b.text = glyph
	b.tooltip_text = tip
	b.focus_mode = Control.FOCUS_NONE
	var s := _m("step_btn")
	b.custom_minimum_size = Vector2(s, s)
	b.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	b.add_theme_font_size_override("font_size", _m("fs_row"))
	for st in ["normal", "hover", "pressed", "focus"]:
		var box := DccTheme.outline("border", "sunken")
		if st == "hover" or st == "pressed":
			box = DccTheme.outline("accent", "sunken")
		b.add_theme_stylebox_override(st, box)
	b.add_theme_color_override("font_color", DccTheme.c("text_bright"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	return b


func _refresh_param(entry: Dictionary) -> void:
	var p: Dictionary = entry["action"]["param"]
	var text := String((p["value"] as Callable).call())
	(entry["value_label"] as Label).text = text
	(entry["node"] as Control).set_meta("card_row", {"kind": "param",
		"text": String(entry["action"].get("label", "")), "value": text,
		"id": String(entry["action"].get("id", ""))})


func _step(entry: Dictionary, dir: int) -> void:
	((entry["action"]["param"] as Dictionary)["step"] as Callable).call(dir)
	_refresh_param(entry)


# -- The header ---------------------------------------------------------------

## §4.1 row 0. Title: the object the rows act on -- the first hit a row names
## in its `header` (CIVIL's rows name their settlement; that is the object the
## verbs are about even when a label sits nearer the pointer), else the
## nearest hit, else `Here`. Kind: a settlement's class, else the hit kind.
## Subtitle: what `sample_cell` knows here, each part omitted when the cell
## does not carry it (never zero-filled -- `sample_cell`'s own contract).
func _header() -> Dictionary:
	var hits: Array = _req.get("hits", [])
	var named := ""
	for a in _actions:
		if (a as Dictionary).has("header"):
			named = String(a["header"])
			break
	var primary: Dictionary = {}
	if named != "":
		for h in hits:
			if String((h as Dictionary).get("label", "")) == named:
				primary = h
				break
	if primary.is_empty() and not hits.is_empty():
		primary = hits[0]
	var title := "Here"
	if not primary.is_empty():
		title = "%s · %s" % [String(primary.get("label", "(unnamed)")), _kind_text(primary)]
	elif named != "":
		title = named
	return {"title": title, "subtitle": _readout()}


func _kind_text(h: Dictionary) -> String:
	var kind := String(h.get("kind", ""))
	if kind == "settlement" and app != null and app.bridge != null:
		var ss: Array = app.bridge.settlements()
		var i := int(h.get("id", -1))
		if i >= 0 and i < ss.size() and (ss[i] as Dictionary).has("kind"):
			return String(ss[i]["kind"])
	return kind


## `[what is here, where it is]` -- either line omitted when empty.
func _readout() -> Array:
	if app == null or app.bridge == null or not app.bridge.has_world:
		return []
	var gx := float(_req.get("gx", 0.0))
	var gy := float(_req.get("gy", 0.0))
	var parts: Array = []
	var cell: Dictionary = app.bridge.sample_cell(int(gx), int(gy))
	if cell.has("elevation_m"):
		var w := String(cell.get("water", ""))
		parts.append("%.0f m%s" % [float(cell["elevation_m"]), (" · " + w) if w == "ocean" or w == "lake" else ""])
	if cell.has("biome") and String(cell["biome"]) != "":
		parts.append(String(cell["biome"]).capitalize())
	var crs: Dictionary = app.bridge.world_crs()
	var where := "cell %d, %d" % [int(gx), int(gy)]
	if crs.has("cell_km") and float(crs["cell_km"]) > 0.0:
		var k := float(crs["cell_km"])
		where += " → %s E · %s S" % [DccUnits.format(gx * k), DccUnits.format(gy * k)]
	var out: Array = []
	if not parts.is_empty():
		out.append(" · ".join(parts))
	out.append(where)
	return out


# -- Interaction --------------------------------------------------------------

func _hover(row: Control) -> void:
	for i in _nav.size():
		if _nav[i]["node"] == row:
			_set_focus(i)
			return


func _set_focus(i: int) -> void:
	if _focus >= 0 and _focus < _nav.size():
		var old: Control = _nav[_focus]["node"]
		if is_instance_valid(old):
			old.add_theme_stylebox_override("panel", old.get_meta("rest"))
	_focus = i
	if i < 0 or i >= _nav.size():
		return
	var n: Control = _nav[i]["node"]
	n.add_theme_stylebox_override("panel", DccTheme.flat(DccTheme.menu_highlight()))
	if _scroll != null and n.is_inside_tree():
		_scroll.ensure_control_visible(n)


func _activate_focused() -> void:
	if _focus < 0 or _focus >= _nav.size():
		return
	var e: Dictionary = _nav[_focus]
	match String(e["kind"]):
		"select":
			var rows: Array = []
			var primary := _primary_hit()
			for n in mini(_all_hits.size(), SELECT_CAP):
				var h: Dictionary = _all_hits[n]
				rows.append({"id": "select.%s.%d" % [h.get("kind", ""), int(h.get("id", -1))],
					"label": "%s · %s" % [String(h.get("label", "(unnamed)")), _kind_text(h)],
					"enabled": true, "shortcut": DccIcons.SYMBOLS["on"] if _same_hit(h, primary) else "",
					"select_hit": h})
			if _all_hits.size() > SELECT_CAP:
				rows.append({"id": "select.more", "label": "… and %d more" % (_all_hits.size() - SELECT_CAP),
					"enabled": false, "reason": "the list shows the %d nearest — zoom in to separate the rest" % SELECT_CAP})
			_open_sub("Objects here", rows)
		"back":
			_sub = {}
			_filter = ""
			_rebuild()
			_place()
		"param":
			pass
		_:
			var a: Dictionary = e["action"]
			if a.has("children"):
				_open_sub(String(a.get("label", "")), a["children"])
				return
			if a.has("select_hit"):
				if _on_select.is_valid():
					_on_select.call(a["select_hit"])
				return
			if not bool(a.get("enabled", true)):
				return
			hide()
			(a["callable"] as Callable).call()


func _primary_hit() -> Dictionary:
	var hits: Array = _req.get("hits", [])
	return hits[0] if not hits.is_empty() else {}


static func _same_hit(a: Dictionary, b: Dictionary) -> bool:
	return not b.is_empty() and a.get("kind") == b.get("kind") and int(a.get("id", -1)) == int(b.get("id", -2))


func _open_sub(title: String, rows: Array) -> void:
	_sub = {"title": title, "rows": rows}
	_filter = ""
	_rebuild()
	_place()


func _on_window_input(event: InputEvent) -> void:
	var k := event as InputEventKey
	if k == null or not k.pressed:
		return
	if _key(k):
		set_input_as_handled()


## One key, returns whether it was used. Esc is not here: the `Popup` closes
## itself on `ui_cancel`, which is exactly §4.2's "Esc closes" and §9.4's
## "never commits or discards a draft by being dismissed".
func _key(k: InputEventKey) -> bool:
	match k.keycode:
		KEY_DOWN:
			if not _nav.is_empty():
				_set_focus((_focus + 1) % _nav.size())
			return true
		KEY_UP:
			if not _nav.is_empty():
				_set_focus((_focus - 1 + _nav.size()) % _nav.size())
			return true
		KEY_ENTER, KEY_KP_ENTER:
			_activate_focused()
			return true
		KEY_RIGHT, KEY_LEFT:
			var right := k.keycode == KEY_RIGHT
			if _focus >= 0 and _focus < _nav.size():
				var e: Dictionary = _nav[_focus]
				if e["kind"] == "param":
					_step(e, 1 if right else -1)
					return true
				if right and (e["kind"] == "select" or (e.has("action") and e["action"].has("children"))):
					_activate_focused()
					return true
			if not right and not _sub.is_empty():
				_sub = {}
				_filter = ""
				_rebuild()
				_place()
			return true
		KEY_BACKSPACE:
			if not _filter.is_empty():
				_filter = _filter.left(_filter.length() - 1)
				_rebuild()
				_place()
			elif not _sub.is_empty():
				_sub = {}
				_rebuild()
				_place()
			return true
	if k.unicode >= 32 and not k.ctrl_pressed and not k.alt_pressed and not k.meta_pressed:
		_filter += char(k.unicode)
		_rebuild()
		_place()
		return true
	return false


# -- Placement ----------------------------------------------------------------

## At the pointer, offset as the canvas offsets it (`cx=x+10, cy=y+6`),
## **flipped** to the other side of the pointer when it would cross the right
## or bottom edge, then clamped inside the viewport's visible rect with an 8 px
## margin -- so it stays on screen at all four edges. Taller than the screen:
## the body scrolls inside a card capped at the visible height.
##
## **The window is not the card.** An embedded `PopupPanel` whose box casts a
## shadow grows its window by the shadow on every side (measured: 18 px left,
## 8 px above for an 18 px shadow offset 10 down), and draws the panel inset
## by that much. So the placement is solved for the *panel* -- `panel_rect()`,
## measured off the drawn content -- and the window is moved by whatever
## separates the two, once now and once after layout settles.
func _place() -> void:
	var bounds := _bounds()
	var margin := 8.0
	_scroll.custom_minimum_size = Vector2.ZERO
	var content: Vector2 = _body.get_combined_minimum_size()
	var avail_h := bounds.size.y - 2.0 * margin - 2.0
	_scroll.custom_minimum_size = Vector2(0.0, minf(content.y, avail_h))
	var want := Vector2(minf(content.x + 2.0, bounds.size.x - 2.0 * margin), minf(content.y, avail_h) + 2.0)
	var t := _target_rect(want)
	if not visible:
		popup(Rect2i(Vector2i(t.position), Vector2i(t.size.ceil())))
	_reclamp()
	_reclamp.call_deferred()


func _bounds() -> Rect2:
	var host_vp: Viewport = app.get_viewport() if app != null else get_viewport()
	return host_vp.get_visible_rect()


## Where a panel of `panel_size` goes: the canvas's offset, flipped, clamped.
## `_dock_side` (CM-4, §7.1) picks which side is tried FIRST -- "left"/"right"
## for the tablet's away-from-the-hand dock, "" for every other presenter's
## plain pointer-relative offset (unchanged from CM-2). Either way the other
## side is the fallback when the preferred one does not fit, so the on-screen
## guarantee is the same regardless of hand.
##
## `gap` (coordinator review, 2026-09-27) replaces the bare `10.0` the canvas's
## own offset names: `_ring_clear` widens it whenever a ring is open beside
## this card (`open()`'s own doc), so the card starts clear of the ring's own
## footprint instead of overlapping its lower slots. **X alone is enough**:
## `radius + slot * 0.5` (what `_ring_clear` carries) is the farthest any of
## the ring's 8 slot squares reaches from the anchor in ANY direction -- the
## ring's own bounding shape is therefore fully inside the square
## `anchor ± _ring_clear` on both axes. Once the card's entire x-range sits
## outside `anchor.x ± gap`, every point in the card is farther than `gap`
## from `anchor` (distance >= |dx| >= gap) for ANY y, so the y offset below
## (unchanged since CM-2) never needs its own widening.
##
## Overlap follow-up, same review: a SUB-ring re-centres on its own parent
## slot (`radial_ring.gd::_open_sub()`), a different point up to the ring's
## own radius away from `_anchor` -- `_sub_x`/`_sub_clear` (`set_ring_clear()`'s
## own doc) are ITS centre-x and `radius + slot * 0.5`, zero whenever none is
## open. `left_edge`/`right_edge` below are the widest either obstacle's own
## `x ± (10 + clear)` square reaches; two rectangles can only overlap if BOTH
## their x-ranges and y-ranges intersect, so keeping the card's entire x-range
## outside `[left_edge, right_edge]` clears BOTH obstacles' shapes for any y --
## the same proof above, generalised from one obstacle to the union of two.
func _target_rect(panel_size: Vector2) -> Rect2:
	var bounds := _bounds()
	var margin := 8.0
	var w := panel_size.x
	var h := panel_size.y
	var left_edge := _anchor.x - 10.0 - _ring_clear
	var right_edge := _anchor.x + 10.0 + _ring_clear
	if _sub_clear > 0.0:
		left_edge = minf(left_edge, _sub_x - 10.0 - _sub_clear)
		right_edge = maxf(right_edge, _sub_x + 10.0 + _sub_clear)
	var x: float
	if _dock_side == "left":
		x = left_edge - w
		if x < bounds.position.x + margin:
			x = right_edge
	elif _dock_side == "right":
		x = right_edge
		if x + w > bounds.end.x - margin:
			x = left_edge - w
	else:
		x = right_edge
		if x + w > bounds.end.x - margin:
			x = left_edge - w
	var y := _anchor.y + 6.0
	if y + h > bounds.end.y - margin:
		y = _anchor.y - 6.0 - h
	x = clampf(x, bounds.position.x + margin, maxf(bounds.position.x + margin, bounds.end.x - margin - w))
	y = clampf(y, bounds.position.y + margin, maxf(bounds.position.y + margin, bounds.end.y - margin - h))
	return Rect2(Vector2(x, y), panel_size)


## The drawn card -- its border included -- in the space the window is
## positioned in. Public: the probe measures THIS against the screen, not the
## window, whose shadow margin may hang off an edge harmlessly.
func panel_rect() -> Rect2:
	var r := _scroll.get_global_rect()
	return Rect2(Vector2(position) + r.position - Vector2.ONE, r.size + Vector2(2, 2))


func _reclamp() -> void:
	if not visible or not is_instance_valid(_scroll):
		return
	var pr := panel_rect()
	var t := _target_rect(pr.size)
	var delta := (t.position - pr.position).round()
	if delta != Vector2.ZERO:
		position += Vector2i(delta)

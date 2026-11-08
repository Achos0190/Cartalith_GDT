extends AcceptDialog

## `Assets ▸ Landmark types…` -- Ruling EH item 2 (`LARGE_ITEM_RULINGS.md`, owner 2026-11-08).
##
## Replaces the last three-level menu path (finding M7 of `PC_SHELL_PROPOSAL.md`; `5c1d178f`
## flattened the other four): `Assets ▸ Landmark types ▸ <family> ▸ <type>`, about 49 read-only
## engine-state rows whose per-family ids (0..n) collided when the phone's MENU or the searchable
## index flattened them. It is now one list in a window, and everything the cascade showed is here:
##
## | the cascade had                              | this window has                                     |
## |----------------------------------------------|-----------------------------------------------------|
## | one family row, `N of M armed · P placed`    | a family header row with the same text              |
## | one leaf per type, `cap · placed · reason`   | one row per type, text from `row_text()` (moved)    |
## | `Open <Family> in the dock` per family       | the same button under each family                   |
## | the "Rows here read state" signpost          | a note under the readout                            |
## | `Placed by hand   N`                         | a button at the foot with the same text             |
## | `Landmark icons…`, `Landmark label style…`   | two buttons at the foot                             |
## | `No landmark types` (older cdylib)           | the same sentence as a note                         |
##
## **Source of the rows: the engine, read at its own symbols, and nothing else.** The families and
## types come from `EngineBridge.landmark_kinds()` (the engine's first-seen order), the caps and
## armed flags from `landmark_settings()`, placed counts and the limiting reason from
## `landmark_funnels()`. The window carries no family, class or type name of its own (the
## hand-maintained catalogue `command_index.gd`'s header calls "the most reliably stale document a
## project can own"). A type the engine cannot place is never counted as armed, whatever the settings
## say, exactly as the dock forces it (`civilization_workspace.gd::_lm_type_row`): two readouts of one
## fact must not disagree.
##
## **What it must never do.** Arm, disarm or cap a type (the slider that does lives in CIVIL ▸
## Landmarks and only there: a row here that armed one would be a second store to keep in sync with
## it); invent a count it did not read; call the engine while `bridge.generating`.
##
## **Desktop (PC-7): a non-blocking window.** `ContextWindow.make_context()` clears `exclusive`, so the
## map and the CIVIL dock stay usable while the list is open; it has no selection subject, so no
## FOLLOW chip. **Phone and tablet:** the phone shows it as a full-screen sheet
## (`DccWidgets.phone_present` / `phone_fit`) and both keep it modal; a destination button closes it
## first so the dock it opens is not hidden behind the sheet.
##
## Preloaded by `menus.gd` (`LANDMARK_TYPES_WINDOW`), no `class_name`, so adding it needs no
## `--headless --import` pass (`CLAUDE.md`: that pass strips `project.godot`'s comments).

## PC-7's non-blocking rules.
const ContextWindow := preload("res://shell/context_window.gd")

## The signpost the cascade carried under every family (kept word for word).
const NOTE_READ_ONLY := "Rows here read state; they do not arm a type."
## Shown instead of an empty list when the native library predates `landmark_kinds()`.
const NOTE_NO_KINDS := ("No landmark types -- this version of the app returned no landmark vocabulary; "
	+ "an update is needed to add it. CIVIL ▸ Landmarks says the same thing at greater length. "
	+ "Neither invents a list to fill the gap.")
## A type row's tooltip (the cascade's own).
const TIP_TYPE := "Read-only. The slider that arms this type is in CIVIL ▸ Landmarks."

## The shell (`DccApp`): `phone_fit`, `is_phone`.
var _host
## The `EngineBridge`.
var _bridge
## Destinations, handed in by `menus.gd` so the dock-opening code stays one implementation there:
## `dock` (Callable(family: String)), `icons`, `labels`.
var _go: Dictionary = {}
## The refilled list.
var _body: VBoxContainer
## "6 families · 49 types · 17 armed · 59 placed".
var _readout: Label
## Phone composition (`DccWidgets.phone_window`).
var _phone := false

## Builds the frame once. `go` carries the destination Callables (see `_go`). Call once before
## `open()`; it appends to `self`.
func setup(host, bridge, go: Dictionary) -> void:
	_host = host
	_bridge = bridge
	_go = go
	title = "Landmark types"
	## A labelled judgement: the longest row ("Name   40 max · 11 placed · spacing") on one line.
	size = Vector2i(620, 600)
	min_size = Vector2i(420, 360)
	_phone = DccWidgets.phone_window(self, host)
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)
	if _phone:
		DccWidgets.phone_head(outer, "Landmark types", "every family and type · read-only")
	else:
		var head_row := HBoxContainer.new()
		head_row.add_theme_constant_override("separation", 12)
		DccWidgets.pad(outer, 12, 6, 12, 6).add_child(head_row)
		head_row.add_child(DccTheme.mono_label("◇ LANDMARK TYPES", "accent", DccTheme.FS_HEADER, 2, true))
		head_row.add_child(DccTheme.label("Assets ▸ Landmark types", "text_ghost", DccTheme.FS_TINY))
		outer.add_child(DccTheme.rule())
	var top := VBoxContainer.new()
	top.add_theme_constant_override("separation", 4)
	DccWidgets.pad(outer, 14, 8, 14, 6).add_child(top)
	_readout = DccTheme.mono_label("", "text_dim", DccTheme.FS_SMALL, 0)
	_readout.name = "LtReadout"
	top.add_child(_readout)
	var note := DccWidgets.note(top, NOTE_READ_ONLY)
	note.name = "LtSignpost"
	var scroll := ScrollContainer.new()
	scroll.name = "LtScroll"
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(scroll)
	var pad := MarginContainer.new()
	for side in ["left", "right", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 14)
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(pad)
	_body = VBoxContainer.new()
	_body.name = "LtRows"
	_body.add_theme_constant_override("separation", 0)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_body)
	## A world swap, a finished generation and a finished landmark pass each change a count; the cascade
	## re-counted on every popup, so this re-reads on each of them while visible (and on every open).
	bridge.generation_finished.connect(func(_ok: bool): if visible: _refresh())
	bridge.world_loaded.connect(func(): if visible: _refresh())
	bridge.landmark_finished.connect(func(_r: Dictionary): if visible: _refresh())
	bridge.landmark_refine_finished.connect(func(_r: Dictionary): if visible: _refresh())
	ContextWindow.make_context(self, host, false)

## Shows the window with a fresh read: a phone sheet on a phone, centred elsewhere.
func open() -> void:
	_refresh()
	if not DccWidgets.phone_present(self, _host):
		popup_centered()
	if _phone:
		_host.phone_fit(self, 1.0)

## The text of one type row, exactly the cascade's (moved here from `menus.gd::_refresh_landmark_menu`,
## where it was inline). Pure so a probe can pin it. `funnel` is the type's `landmark_funnels()` row, or
## `{}` when it has not run -- which is not the same as having placed zero, so it prints no reason.
static func row_text(label: String, buildable: bool, armed: bool, cap: int, funnel: Dictionary) -> String:
	if not buildable:
		return "%s   not buildable" % label
	if armed:
		## Normalised, not looked up raw: the engine spells its tokens with spaces
		## (`CivilizationWorkspace._lm_limit_word`).
		var word := CivilizationWorkspace._lm_limit_word(String(funnel.get("limit", "")))
		var placed := int(funnel.get("placed", 0))
		return ("%s   %d max · %d placed · %s" % [label, cap, placed, word]
			if word != "" else "%s   %d max" % [label, cap])
	if cap > 0:
		return "%s   off · was %d" % [label, cap]
	return "%s   off" % label

## The family header's text, the cascade's own: `Title   n of m armed · p placed`.
static func family_text(title_text: String, n_armed: int, n_types: int, n_placed: int) -> String:
	return "%s   %d of %d armed · %d placed" % [title_text, n_armed, n_types, n_placed]

## The per-family model the list is drawn from: an Array of
## `{key, title, rows: [{key, label, text}], armed, total, placed}` in the engine's first-seen family
## order. An empty Array means the binding is missing (an older cdylib), which the caller says in
## words rather than draw an empty list. Reads only the bridge.
func model() -> Array:
	var kinds: Array = []
	if _bridge != null and _bridge.has_method("landmark_kinds"):
		kinds = _bridge.landmark_kinds()
	var st: Dictionary = {}
	if _bridge != null and _bridge.has_method("landmark_settings"):
		st = _bridge.landmark_settings()
	var caps: Dictionary = st.get("caps", {})
	var armed_map: Dictionary = st.get("armed", {})
	var funnels := {}
	if _bridge != null and _bridge.has_method("landmark_funnels"):
		for f in _bridge.landmark_funnels():
			funnels[String((f as Dictionary).get("kind", ""))] = f
	var order: Array[String] = []
	var by_fam := {}
	for k in kinds:
		var f := String((k as Dictionary).get("family", "other"))
		if not by_fam.has(f):
			by_fam[f] = []
			order.append(f)
		(by_fam[f] as Array).append(k)
	var out: Array = []
	for fam in order:
		var rec := {"key": fam, "title": fam.replace("_", " ").capitalize(), "rows": [],
			"armed": 0, "total": (by_fam[fam] as Array).size(), "placed": 0}
		for k in by_fam[fam]:
			var kd: Dictionary = k
			var key := String(kd.get("key", ""))
			var label := String(kd.get("label", key))
			var buildable := bool(kd.get("buildable", true))
			## A type the engine cannot place is never counted as armed.
			var on := buildable and bool(armed_map.get(key, false))
			var fun: Dictionary = funnels.get(key, {})
			if on:
				rec["armed"] = int(rec["armed"]) + 1
				rec["placed"] = int(rec["placed"]) + int(fun.get("placed", 0))
			(rec["rows"] as Array).append({"key": key, "label": label,
				"text": row_text(label, buildable, on, int(caps.get(key, 0)), fun)})
		out.append(rec)
	return out

## Rebuilds the readout and the list from `model()`, or draws the state that stops it.
func _refresh() -> void:
	for c in _body.get_children():
		_body.remove_child(c)
		c.queue_free()
	if _bridge != null and bool(_bridge.generating):
		_readout.text = "Generating"
		DccWidgets.note(_body, "A generation is running; the landmark vocabulary is read when it finishes (the engine object belongs to the worker until then).")
		return
	var fams := model()
	if fams.is_empty():
		_readout.text = "No landmark types"
		var n := DccWidgets.note(_body, NOTE_NO_KINDS)
		n.name = "LtNoKinds"
	else:
		var types := 0
		var armed := 0
		var placed := 0
		for f in fams:
			types += int((f as Dictionary)["total"])
			armed += int((f as Dictionary)["armed"])
			placed += int((f as Dictionary)["placed"])
		_readout.text = "%d families · %d types · %d armed · %d placed" % [fams.size(), types, armed, placed]
		for f in fams:
			_add_family(f as Dictionary)
	_add_foot()
	if _phone:
		_host.phone_fit(self, 1.0)

## One family: the header (counts), one row per type, the dock destination, a rule.
func _add_family(f: Dictionary) -> void:
	var fam := String(f["key"])
	var head := DccTheme.mono_label(family_text(String(f["title"]), int(f["armed"]), int(f["total"]),
		int(f["placed"])), "accent", DccTheme.FS_SMALL, 1, true)
	head.name = "LtFamily"
	head.set_meta("family", fam)
	var hp := MarginContainer.new()
	hp.add_theme_constant_override("margin_top", 10)
	hp.add_theme_constant_override("margin_bottom", 2)
	_body.add_child(hp)
	hp.add_child(head)
	for r in f["rows"]:
		var rd: Dictionary = r
		var b := _row_button(String(rd["text"]))
		b.name = "LtType"
		b.set_meta("lt_role", "type")  ## Godot renames a repeated sibling name, so a probe finds rows by this.
		b.set_meta("family", fam)
		b.set_meta("type_key", String(rd["key"]))
		b.tooltip_text = TIP_TYPE
		## Every leaf was a destination in the cascade, so which one was picked did not change what
		## happened; kept.
		b.pressed.connect(_dock.bind(fam))
		_body.add_child(b)
	var open := _row_button("Open %s in the dock" % String(f["title"]), true)
	open.name = "LtOpenDock"
	open.set_meta("lt_role", "open")
	open.set_meta("family", fam)
	open.tooltip_text = "→ CIVIL ▸ Landmarks, opened at this family."
	open.pressed.connect(_dock.bind(fam))
	_body.add_child(open)
	_body.add_child(DccTheme.rule())

## The foot: hand-placed count and the two real destinations (the cascade's fixed rows).
func _add_foot() -> void:
	var hand := "Placed by hand"
	if _bridge != null and _bridge.has_method("icon_list"):
		hand = "Placed by hand   %d" % (_bridge.icon_list() as Array).size()
	var b := _row_button(hand, true)
	b.name = "LtHand"
	b.tooltip_text = ("Icons stamped on the map by hand. Annotation, not entities: no causal "
		+ "chain and no emergent importance, so they are never counted into a family above.")
	b.pressed.connect(_dock.bind(""))
	_body.add_child(b)
	var ic := _row_button("Landmark icons…", true)
	ic.name = "LtIcons"
	ic.tooltip_text = ("Opens the Asset library on the points-of-interest family. Landmarks draw "
		+ "from eight of its slots (Lake and Bridge are never used on the map): a "
		+ "type uses the slot of the same name, peaks use Mountain peak, ancient "
		+ "forests Named forest, ruins Ruin, and every other type uses Other.")
	ic.pressed.connect(_dest.bind("icons"))
	_body.add_child(ic)
	var lb := _row_button("Landmark label style…", true)
	lb.name = "LtLabels"
	lb.tooltip_text = ("→ Cartography ▸ Labels. A landmark's icon and label style are "
		+ "presentation and stay where presentation lives; its cap and its "
		+ "spacing are world model and stay in CIVIL.")
	lb.pressed.connect(_dest.bind("labels"))
	_body.add_child(lb)

## A flat full-width row button; `accent` inks a destination.
func _row_button(text: String, accent: bool = false) -> Button:
	var b := Button.new()
	b.text = text
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.focus_mode = Control.FOCUS_NONE
	b.flat = true
	b.add_theme_color_override("font_color", DccTheme.c("accent" if accent else "text"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("accent"))
	b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if DccTheme.is_tablet():
		b.custom_minimum_size.y = DccTheme.role_px("row_min_h")
	return b

## Closes a modal window (phone, tablet) so the destination is not hidden behind it; the desktop's stays.
func _leave() -> void:
	if not ContextWindow.applies(_host):
		hide()

## Opens CIVIL ▸ Landmarks at `family` (`""` = the hand-placed icons), through `menus.gd`.
func _dock(family: String) -> void:
	_leave()
	if _go.has("dock"):
		(_go["dock"] as Callable).call(family)

## Runs the named destination (`icons` or `labels`).
func _dest(which: String) -> void:
	_leave()
	if _go.has(which):
		(_go[which] as Callable).call()

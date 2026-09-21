extends AcceptDialog
class_name SettlementTypesWindow

## `lazy-riding-piglet.md` Batch D -- artboard `1f`
## (`design/settlement-editor-2026-09-21/Cartalith Settlement Editor.dc.html`,
## `id="1f"`, "SETTLEMENT TYPES · templates, and a default per faction").
##
## A **separate popup**, not a Place editor tab -- the canvas's own framing,
## and the right shape here too: a type is a library entry a faction reads
## from, not a fact about one settlement. `AcceptDialog` is this shell's
## established free-floating-window vocabulary
## (`place_editor_window.gd`/`faction_roster_window.gd`/
## `culture_profiles_window.gd`), and this window follows
## `culture_profiles_window.gd`'s three-pane shape most closely: a library
## list on the left, a selected item's editable detail in the centre, a
## per-faction column on the right.
##
## ## Storage
##
## Reads and writes nothing of its own -- every field lives in
## `SettlementTypeStore` (a static store, same shape as `trade_store.gd`), so
## this window is a pure UI over state that survives it being closed. See
## that file's own top-of-file doc for the document-slot registration
## (`library/settlement_types.json`, caller-owned) and the wiring that
## applies a default type on settlement drop
## (`civilization_workspace.gd::_settlement_click`).
##
## ## Phone
##
## One `if _phone: ... else: ...` branch in `_rebuild()`, matching this
## shell's established convention (confirmed in `faction_roster_window.gd`
## and `culture_profiles_window.gd`) over a second per-platform file: the
## three columns stack into three sections in list order (library, detail,
## faction defaults) rather than a `TabContainer` or a bespoke pane switcher,
## since none of the three needs to be hidden from the others the way
## `culture_profiles_window.gd`'s own phone switcher hides its panes -- a
## type's detail and the faction column are both short enough to read
## together on a scroll.

var app                       ## `DccApp`
var bridge: EngineBridge

var _body: VBoxContainer
var _phone := false
var _phone_title: Label
var _rebuilding := false

## Selected library entry's id, `""` when the library is empty.
var _selected_id := ""

const KIND_ORDER := SettlementTypeStore.KIND_ORDER


func setup(a, b: EngineBridge) -> void:
	app = a
	bridge = b
	title = "Settlement types"
	size = Vector2i(900, 560)
	min_size = Vector2i(640, 420)
	max_size = Vector2i(1180, 780)
	_phone = DccWidgets.phone_window(self, a)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 0)
	add_child(root)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_child(scroll)
	if _phone:
		_phone_title = DccWidgets.phone_head(root, "Settlement types",
			"library · base kind · traits · per-faction defaults")
	var pad := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 12)
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(pad)
	_body = VBoxContainer.new()
	_body.add_theme_constant_override("separation", 4)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_body)

	## The library is project data, not settlement data -- a generate or a
	## load does not touch it (`SettlementTypeStore` is restored only from
	## `app.gd::_restore_project_documents()`, on an actual project open).
	## This window still rebuilds on both, matching every other window here,
	## because the right-hand faction column reads `get_factions()` and a
	## generate/load can change the roster out from under an open window.
	bridge.generation_finished.connect(func(ok: bool): if ok and visible: _rebuild())
	bridge.world_loaded.connect(func(): if visible: _rebuild())


func open() -> void:
	var types := SettlementTypeStore.types()
	if _selected_id == "" or SettlementTypeStore.get_type(_selected_id).is_empty():
		_selected_id = String((types[0] as Dictionary).get("id", "")) if not types.is_empty() else ""
	_rebuild()
	if not DccWidgets.phone_present(self, app):
		popup_centered()


func _clear() -> void:
	_rebuilding = true
	for c in _body.get_children():
		_body.remove_child(c)
		c.queue_free()
	_rebuilding = false


func _rebuild() -> void:
	_clear()
	var types := SettlementTypeStore.types()
	if _selected_id != "" and SettlementTypeStore.get_type(_selected_id).is_empty():
		_selected_id = ""
	if _selected_id == "" and not types.is_empty():
		_selected_id = String((types[0] as Dictionary).get("id", ""))

	if _phone:
		_build_list(_body, types)
		if _selected_id != "":
			_build_detail(_body)
		_build_faction_defaults(_body)
	else:
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 12)
		row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		var left := VBoxContainer.new()
		left.custom_minimum_size.x = 220
		_build_list(left, types)
		row.add_child(left)
		var center := VBoxContainer.new()
		center.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		if _selected_id != "":
			_build_detail(center)
		else:
			DccWidgets.note(center, "No type yet -- \"+ New type\" on the left to create one.")
		row.add_child(center)
		var right := VBoxContainer.new()
		right.custom_minimum_size.x = 240
		_build_faction_defaults(right)
		row.add_child(right)
		_body.add_child(row)

	if _phone:
		app.phone_fit(self, 1.0)


# -- Library list -------------------------------------------------------------

func _build_list(parent: Control, types: Array) -> void:
	var sec := DccWidgets.section(parent, "Library · %d" % types.size())
	if types.is_empty():
		DccWidgets.note(sec, "No settlement types yet.")
	for t in types:
		var d: Dictionary = t
		var id := String(d.get("id", ""))
		var used := int(d.get("usage_count", 0))
		var label := "%s -- %s · %s (used %d)" % [
			String(d.get("name", "")),
			String(d.get("kind", "town")).capitalize(),
			String(d.get("specialisation", "none")).capitalize(),
			used]
		DccWidgets.chip(sec, label, func():
			_selected_id = id
			_rebuild(), id == _selected_id)
	DccWidgets.action(sec, "+ New type", func():
		_selected_id = SettlementTypeStore.new_type("New type")
		_rebuild())
	DccWidgets.note(sec, "Types live in the project file, not preferences.")


# -- Detail ---------------------------------------------------------------------

func _build_detail(parent: Control) -> void:
	var id := _selected_id
	var t := SettlementTypeStore.get_type(id)
	if t.is_empty():
		return

	var sec := DccWidgets.section(parent, "Type")
	var name_row := HBoxContainer.new()
	name_row.add_theme_constant_override("separation", 6)
	var name_edit := LineEdit.new()
	name_edit.text = String(t.get("name", ""))
	name_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## Same commit-on-loss-of-focus-or-Enter discipline as
	## `place_editor_window.gd::_build_identity`'s name field, and the same
	## `_rebuilding` guard so a rebuild's own teardown does not write the
	## stale text of a field about to be discarded.
	name_edit.text_submitted.connect(func(v: String):
		SettlementTypeStore.set_field(id, "name", v)
		_rebuild())
	name_edit.focus_exited.connect(func():
		if _rebuilding or not is_instance_valid(name_edit):
			return
		SettlementTypeStore.set_field(id, "name", name_edit.text)
		_rebuild())
	name_row.add_child(name_edit)
	sec.add_child(name_row)

	var actions_row := HBoxContainer.new()
	actions_row.add_theme_constant_override("separation", 10)
	DccWidgets.action(actions_row, "Duplicate", func():
		var new_id := SettlementTypeStore.duplicate_type(id)
		if new_id != "":
			_selected_id = new_id
		_rebuild())
	var del := DccWidgets.action(actions_row, "Delete", func():
		SettlementTypeStore.delete_type(id)
		_selected_id = ""
		_rebuild())
	del.add_theme_color_override("font_color", DccTheme.c("accent"))
	sec.add_child(actions_row)

	# -- Base kind ------------------------------------------------------------
	DccWidgets.choice(sec, "Base kind", KIND_ORDER.map(func(k): return String(k).capitalize()),
		maxi(0, KIND_ORDER.find(String(t.get("kind", "town")))),
		func(i: int):
			SettlementTypeStore.set_field(id, "kind", KIND_ORDER[i])
			_rebuild(),
		"Six fixed values. A type pins one of them; it cannot add a seventh. Base population follows the kind, not the type.")

	# -- Specialisation ---------------------------------------------------------
	var specs := bridge.civ_specialisation_vocabulary()
	if specs.is_empty():
		DccWidgets.note(sec, "No specialisation vocabulary -- the engine build is older than civ_specialisation_vocabulary().")
	else:
		var keys: Array = []
		var labels: Array = []
		for e in specs:
			var d: Dictionary = e
			keys.append(String(d.get("key", "none")))
			labels.append(String(d.get("label", "?")))
		DccWidgets.choice(sec, "Specialisation", labels,
			maxi(0, keys.find(String(t.get("specialisation", "none")))),
			func(i: int):
				SettlementTypeStore.set_field(id, "specialisation", keys[i])
				_rebuild())

	_build_traits_chips(sec, id, t)

	# -- Walls / Age policy -----------------------------------------------------
	var walls := int(t.get("walls", -1))
	DccWidgets.choice(sec, "Walls", ["Auto", "No fortifications", "Fortified"],
		0 if walls < 0 else (1 if walls == 0 else 2),
		func(i: int):
			SettlementTypeStore.set_field(id, "walls", (-1 if i == 0 else (0 if i == 1 else 1)))
			_rebuild())
	var age_mode := String(t.get("age_mode", "auto"))
	DccWidgets.choice(sec, "Age policy", ["Auto", "Fixed"],
		0 if age_mode == "auto" else 1,
		func(i: int):
			SettlementTypeStore.set_field(id, "age_mode", "auto" if i == 0 else "fixed")
			_rebuild())
	if age_mode == "fixed":
		DccWidgets.number(sec, "Age (yr)", 30.0, 1000.0, 1.0, float(t.get("age_years", 100)),
			func(v: float): SettlementTypeStore.set_field(id, "age_years", int(v)),
			"Clamped to 30..1000 on apply, matching the settlement's own Age field (place_editor_window.gd::_build_urban).")

	# -- Name pool ----------------------------------------------------------------
	var pool := DccWidgets.group(sec, "Name pool")
	DccWidgets.note(pool,
		"Inherit the faction's culture -- the only functional option. Naming is a "
		+ "faction-level property: civ_reroll_settlement_name keys a re-roll off the "
		+ "settlement's own faction, never a per-type pool. Overriding it per type would "
		+ "need a generation-side hook that does not exist.")

	# -- Placement rules (dashed) -------------------------------------------------
	var plc := DccWidgets.section(sec, "Placement rules")
	DccWidgets.note(plc,
		"Terrain preference, minimum spacing and river/coast requirements would make a "
		+ "type influence WHERE generation puts a settlement. Placement reads suitability "
		+ "and faction, not types -- this needs a generation-side hook that does not "
		+ "exist. Dashed until it does.")


## The exact toggle-chip pattern `place_editor_window.gd::_build_traits`
## already established for a closed-vocabulary chip row, reused verbatim
## down to the styling -- only the write target differs (`SettlementTypeStore
## .toggle_trait` in place of `bridge.civ_settlement_toggle_trait`, since a
## type's traits are authored data, not a live settlement's).
func _build_traits_chips(parent: Control, id: String, t: Dictionary) -> void:
	var sec := DccWidgets.section(parent, "Traits applied on drop")
	var vocab := bridge.civ_trait_vocabulary()
	if vocab.is_empty():
		DccWidgets.note(sec, "No trait vocabulary -- the engine build is older than civ_trait_vocabulary().")
		return
	var on: Array = t.get("traits", [])
	var flow := HFlowContainer.new()
	flow.add_theme_constant_override("h_separation", 4)
	flow.add_theme_constant_override("v_separation", 4)
	for e in vocab:
		var d: Dictionary = e
		var key := String(d.get("key", ""))
		var is_on: bool = on.has(key)
		var b := Button.new()
		b.text = "%s %s" % [String(d.get("glyph", "")), String(d.get("label", key))]
		b.flat = false
		b.focus_mode = Control.FOCUS_NONE
		b.custom_minimum_size.y = 22
		b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
		b.add_theme_color_override("font_color",
			DccTheme.c("accent_ink") if is_on else DccTheme.c("text_dim"))
		b.add_theme_stylebox_override("normal",
			DccTheme.flat(DccTheme.c("accent") if is_on else DccTheme.c("sunken")))
		b.add_theme_stylebox_override("hover",
			DccTheme.flat(DccTheme.c("accent").lightened(0.1)) if is_on
			else DccTheme.outline("border", "sunken"))
		b.pressed.connect(func():
			SettlementTypeStore.toggle_trait(id, key)
			_rebuild())
		flow.add_child(b)
	sec.add_child(flow)


# -- Faction defaults column ---------------------------------------------------

func _build_faction_defaults(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Default type per faction")
	var factions := bridge.get_factions()
	if factions.is_empty():
		DccWidgets.note(sec, "No factions -- generate a world first.")
		return
	var types := SettlementTypeStore.types()
	var type_labels: Array = ["None"]
	var type_ids: Array = [""]
	for t in types:
		var d: Dictionary = t
		type_labels.append(String(d.get("name", "Type")))
		type_ids.append(String(d.get("id", "")))
	for f in factions:
		var fd: Dictionary = f
		var fid := int(fd.get("id", 1))
		var cur := SettlementTypeStore.faction_default(fid)
		var idx := maxi(0, type_ids.find(cur))
		DccWidgets.choice(sec, String(fd.get("name", "Faction %d" % fid)), type_labels, idx,
			func(i: int): SettlementTypeStore.set_faction_default(fid, String(type_ids[i])))
	DccWidgets.note(sec,
		"This column is the whole reason the library exists: the settlement tool reads "
		+ "the armed faction's default type, so dropping a place in that faction's "
		+ "territory applies its bundle without setting five fields by hand. A faction "
		+ "left on None behaves exactly as today.")

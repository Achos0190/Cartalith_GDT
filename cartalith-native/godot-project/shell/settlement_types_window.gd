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
## the now-retired `culture_profiles_window.gd` (FH-2)), and this window follows
## the now-retired `culture_profiles_window.gd` (FH-2)), and this window follows
## the now-retired `culture_profiles_window.gd` (FH-2)'s pane shape: a library
## list on the left and a selected item's editable detail in the centre. A third,
## per-faction column used to sit on the right; **FH-3 moved it into the Factions
## hub** (Identity tab, "Default settlement type") and left a one-line link in
## its place (`_build_faction_defaults_link`). This window is now the library only.
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
## and the now-retired `culture_profiles_window.gd` (FH-2)) over a second per-platform file: the
## two panes stack into two sections in list order (library, detail), then the
## faction-defaults link, rather than a `TabContainer` or a bespoke pane switcher,
## since neither pane needs to be hidden from the other the way
## the now-retired `culture_profiles_window.gd` (FH-2)'s own phone switcher hides its panes -- a
## type's detail is short enough to read under the list on a scroll.

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
	## No `max_size` (2026-09-24, the vault window's `3736fe7` fix repeated). The
	## cap treated `wrap_controls` growing the window to its content, which
	## `phone_window()` below turns off at the cause; all the cap still did was
	## stop a user from making the window bigger.
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
			"library · base kind · traits · applied on drop")
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
	## This window still rebuilds on both, matching every other window here:
	## a project open replaces the library through `restore_document()` and an
	## open window would otherwise keep listing the outgoing project's types.
	## (Until FH-3 the per-faction column also read `get_factions()` here; that
	## column now lives in the Factions hub and no longer reads the roster.)
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
		_body.add_child(row)
	## Last on both forms (FH-3): the faction-defaults link, in place of the
	## third column / third phone section the per-faction defaults used to be.
	_build_faction_defaults_link(_body)

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


# -- Faction defaults: moved to the Factions hub (FH-3) ----------------------

## The old "Default type per faction" column, replaced by a one-line pointer
## (FH-3, `FACTION_HUB_DESIGN.md` §3.5): a faction's default settlement type is a
## fact about the faction, so its picker is now the Identity tab's "Default
## settlement type" row (`faction_roster_window.gd::_default_type_choice`), where
## the faction's other identity fields already are. This window stays the
## **library** -- a type is a library entry, not a faction field.
##
## The button opens the hub on Identity with no faction named (`-1`: the hub's
## last-selected faction) and closes this window first, so two exclusive popups
## are never stacked on desktop and the phone's full-screen windows do not pile
## up. Nothing is lost by hiding: the library holds no unsaved edit -- every field
## writes `SettlementTypeStore` live, and the one text field (the type name)
## is committed by releasing focus before the hide (its `focus_exited` write).
##
## **It must not touch `SettlementTypeStore`'s defaults.** The data, its
## `document()` form and `apply_default_to_settlement` (the drop tool's reader)
## are unchanged; only where the choice is edited moved.
func _build_faction_defaults_link(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Faction defaults")
	var link := DccWidgets.action(sec, "Faction defaults now live in Factions ▸ Identity", func():
		_release_focus_in_body()
		hide()
		app.open_faction_roster(-1, "identity"))
	link.name = "FactionDefaultsLink"
	link.tooltip_text = "Opens the Factions window on its Identity tab, where each faction's Default settlement type is chosen. A faction left on None behaves exactly as today."


## Releases keyboard focus if it sits inside this window's body, which fires the
## type-name field's `focus_exited` commit (`_build_detail`) *before* the window
## hides -- hiding alone is not relied on to deliver that signal. A no-op when
## nothing in the body is focused.
func _release_focus_in_body() -> void:
	var vp := _body.get_viewport()
	if vp == null:
		return
	var fo := vp.gui_get_focus_owner()
	if fo != null and _body.is_ancestor_of(fo):
		fo.release_focus()

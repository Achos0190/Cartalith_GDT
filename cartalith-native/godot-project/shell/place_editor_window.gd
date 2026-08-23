extends AcceptDialog
class_name PlaceEditorWindow

## `placeEditPopup` / `_civPopulatePlaceEditor` (reference 16694) --
## `PARITY_AUDIT.md` §5 item 3, `GUI_GAP_REGISTER.md` ED-03.
##
## The audit's own words: `civ_drop_settlement` created a settlement and
## nothing edited, moved or deleted one, so "a user can add a settlement they
## can never fix or undo." This is the missing editor: name (with the
## reference's own culture-aware re-roll), class, polity, population,
## economy, traits, age and walls overrides, history, focus-camera, and
## Delete.
##
## ## A window, not a right-dock context
##
## `right_dock.gd`'s Settlement context renders every field as a read-only
## `Label` and is a *summary* surface -- `civilization_workspace.gd`'s own
## `_settlement_click` already notes it has "no name field to focus". Rather
## than convert a read-only dock into an editor (and inherit its
## rebuild-on-every-selection lifecycle for a form that must not lose
## half-typed text), this follows the reference, which puts the place editor
## in its own floating popup anchored at the place. `AcceptDialog` is this
## shell's established free-floating-window vocabulary
## (`world_data_window.gd`, `travel_library_window.gd`).
##
## ## What is deliberately absent
##
## - **Category (settlement ↔ POI).** POI is not a ported concept
##   (`civ_tools_bridge.rs`'s module doc, `GUI_GAP_REGISTER.md` CV-01) -- the
##   selector would have exactly one option.
## - **`peCityPreview`** (the town-layout *thumbnail* inline in the popup,
##   register UM-03): needs a rendered layout at icon size. Its launcher
##   `peCityOpen` IS wired -- the Actions section opens `CityViewerWindow`.
## - **`_civPeConnect` ("Connect to road network")**: no
##   `civ_connect_place_to_network` binding exists; roads are produced inside
##   `generate()` and never mutated afterwards.
## - **Econ. importance / Trade volume**: `NamedSettlement` carries neither,
##   and `FactionPlace` reads both as zero. Editing a number nothing stores
##   would be a fake control.
##
## Each of those is stated in the window's own footer rather than only here,
## so a user of the product sees the same list a reader of this file does.

var app                       ## `DccApp`
var bridge: EngineBridge

var _index := -1
var _body: VBoxContainer
var _name_edit: LineEdit
var _footer: Label

## Emitted after any change that moves map data, so the caller can refresh
## the overlay without this file knowing how (`civilization_workspace.gd`
## owns `_refresh_civ_data`).
signal place_changed
signal place_deleted

const KIND_ORDER := ["metropolis", "capital", "city", "town", "village", "hamlet"]


func setup(a, b: EngineBridge) -> void:
	app = a
	bridge = b
	title = "Place"
	size = Vector2i(400, 640)
	min_size = Vector2i(340, 420)
	## The body scrolls; the window must not grow to fit it (same reason
	## `faction_roster_window.gd` caps its own).
	max_size = Vector2i(560, 760)
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	add_child(scroll)
	var pad := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 12)
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(pad)
	_body = VBoxContainer.new()
	_body.add_theme_constant_override("separation", 4)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_body)


## `_civOpenPlacePopup`: opens on the settlement at `index` into
## `get_settlements()`'s own array.
func open_for(index: int) -> void:
	_index = index
	_rebuild()
	if _index < 0:
		return
	popup_centered()
	if _name_edit != null:
		## §4.5.3's own right-dock column asks for "the new settlement's
		## inspector, live, focused on the name field" -- which nothing in
		## this shell could honour until this window existed.
		_name_edit.grab_focus()
		_name_edit.select_all()


func _clear() -> void:
	for c in _body.get_children():
		_body.remove_child(c)
		c.queue_free()
	_name_edit = null


func _settlement() -> Dictionary:
	var all := bridge.settlements()
	if _index < 0 or _index >= all.size():
		return {}
	return all[_index]


func _rebuild() -> void:
	_clear()
	var s := _settlement()
	if s.is_empty():
		DccWidgets.note(_body, "That settlement no longer exists.")
		_index = -1
		return
	var details := bridge.civ_settlement_details(_index)
	title = "Place — %s" % String(s.get("name", "(unnamed)"))

	_build_identity(s)
	_build_class_and_polity(s)
	_build_economy(s, details)
	_build_traits(details)
	_build_urban(details)
	_build_history(details)
	_build_actions(s)
	_build_footer()


# -- Name + re-roll ---------------------------------------------------------

func _build_identity(s: Dictionary) -> void:
	var sec := DccWidgets.section(_body, "Identity")
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	_name_edit = LineEdit.new()
	_name_edit.text = String(s.get("name", ""))
	_name_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## Commit on focus-loss/Enter rather than per keystroke: the engine call
	## is a full `Dictionary` round trip, and a rebuild mid-word would steal
	## focus -- the same reasoning `civilization_workspace.gd`'s own
	## `_settlement_name_field` gives for not rebuilding on `text_changed`.
	_name_edit.text_submitted.connect(func(t: String): _apply({"name": t}))
	_name_edit.focus_exited.connect(func(): _apply({"name": _name_edit.text}))
	row.add_child(_name_edit)
	var roll := Button.new()
	roll.text = "⟳"
	roll.tooltip_text = "Re-roll a name from this settlement's own faction naming culture (reference v1.07: a rename matches the polity it belongs to, not the global pool)."
	roll.focus_mode = Control.FOCUS_NONE
	roll.pressed.connect(func():
		var n := bridge.civ_reroll_settlement_name(_index)
		if n != "":
			place_changed.emit()
			_rebuild())
	row.add_child(roll)
	sec.add_child(row)
	DccWidgets.note(sec, "Cell (%d, %d) · tid %d" % [
		int(s.get("x", 0)), int(s.get("y", 0)), int(s.get("tid", 0))])


# -- Class + polity ---------------------------------------------------------

func _build_class_and_polity(s: Dictionary) -> void:
	var sec := DccWidgets.section(_body, "Classification")
	var kind := String(s.get("kind", "town"))
	DccWidgets.choice(sec, "Class", KIND_ORDER.map(func(k): return String(k).capitalize()),
		maxi(0, KIND_ORDER.find(kind)),
		func(i: int): _apply({"kind": KIND_ORDER[i]}); _rebuild(),
		"Metropolis is selectable here even though the Settlement tool refuses it: promoting an existing settlement is exactly what _civSelectMetropolises does. Changing class also sets/clears the capital flag, which is the same fact stored twice.")

	var factions := bridge.get_factions()
	if factions.is_empty():
		DccWidgets.note(sec, "No factions -- generate a world first.")
		return
	var ids: Array = []
	var labels: Array = []
	for f in factions:
		var d: Dictionary = f
		ids.append(int(d.get("id", 1)))
		labels.append("%d · %s" % [int(d.get("id", 1)), String(d.get("name", "?"))])
	DccWidgets.choice(sec, "Polity", labels, maxi(0, ids.find(int(s.get("faction", 1)))),
		func(i: int): _apply({"faction": ids[i]}); _rebuild(),
		"The reference's own Polity picker. Territory is NOT repainted -- assign_territory runs inside generate() and no #[func] re-runs it (GUI_GAP_REGISTER.md, Politics ▸ Recalculate territories).")


# -- Population + economy ---------------------------------------------------

func _build_economy(s: Dictionary, details: Dictionary) -> void:
	var sec := DccWidgets.section(_body, "Population & economy")
	## Step 1, not the reference's own `step="500"`: an HTML number input
	## only steps its ARROWS by that, while Godot's `SpinBox` snaps the
	## displayed value to `min + k*step` -- which would show a population of
	## 4500 for a settlement the engine holds at 4321, and write the lie back
	## on the next interaction. Correctness over arrow ergonomics.
	DccWidgets.number(sec, "Population", 0.0, 100000000.0, 1.0, float(int(s.get("population", 0))),
		func(v: float): _apply({"population": int(v)}))
	var specs := bridge.civ_specialisation_vocabulary()
	if not specs.is_empty():
		var keys: Array = []
		var labels: Array = []
		for e in specs:
			var d: Dictionary = e
			keys.append(String(d.get("key", "none")))
			labels.append(String(d.get("label", "?")))
		var cur := String(details.get("specialisation", "none"))
		DccWidgets.choice(sec, "Economy", labels, maxi(0, keys.find(cur)),
			func(i: int): _apply({"specialisation": keys[i]}),
			"CIV_SPECIALISATIONS, the reference's own vocabulary. Stored on the settlement but NOT fed back into civ_faction_aggregates' sector output -- doing so would change already-golden economy numbers on a user edit. See GUI_GAP_REGISTER.md ED-03.")


# -- Traits -----------------------------------------------------------------

func _build_traits(details: Dictionary) -> void:
	var sec := DccWidgets.section(_body, "Traits")
	var vocab := bridge.civ_trait_vocabulary()
	if vocab.is_empty():
		DccWidgets.note(sec, "No trait vocabulary -- the engine build is older than civ_trait_vocabulary().")
		return
	var on: PackedStringArray = details.get("traits", PackedStringArray())
	var flow := HFlowContainer.new()
	flow.add_theme_constant_override("h_separation", 4)
	flow.add_theme_constant_override("v_separation", 4)
	for e in vocab:
		var d: Dictionary = e
		var key := String(d.get("key", ""))
		var is_on := on.has(key)
		var b := Button.new()
		b.text = "%s %s" % [String(d.get("glyph", "")), String(d.get("label", key))]
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.custom_minimum_size.y = 22
		b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
		b.add_theme_color_override("font_color", DccTheme.c("bg") if is_on else DccTheme.c("text_dim"))
		b.add_theme_stylebox_override("normal",
			DccTheme.flat(DccTheme.c("accent") if is_on else DccTheme.c("sunken"), 3))
		b.add_theme_stylebox_override("hover",
			DccTheme.flat(DccTheme.c("accent").lightened(0.1) if is_on else DccTheme.c("raised"), 3))
		b.pressed.connect(func():
			bridge.civ_settlement_toggle_trait(_index, key)
			_rebuild())
		flow.add_child(b)
	sec.add_child(flow)
	DccWidgets.note(sec,
		"Map-glyph badges, deliberately overlapping Economy on mining/trade hub -- the "
		+ "reference keeps both vocabularies on purpose. Nothing in this port draws them on "
		+ "the map yet (map_overlay.gd has no per-trait glyph pass).")


# -- Age + walls ------------------------------------------------------------

func _build_urban(details: Dictionary) -> void:
	var sec := DccWidgets.section(_body, "Settlement fabric")
	var age := int(details.get("age", -1))
	## Step 1 for the same reason Population above uses it -- see there.
	DccWidgets.number(sec, "Age (yr)", -1.0, 1000.0, 1.0, float(age),
		func(v: float): _apply({"age": int(v)}),
		"-1 = auto (the reference infers age from population via _umInferAge). Any other value is clamped to 30..1000, exactly as the reference's own field does.")
	var walls := int(details.get("walls", -1))
	DccWidgets.choice(sec, "Walls", ["Auto", "No fortifications", "Fortified"],
		0 if walls < 0 else (1 if walls == 0 else 2),
		func(i: int): _apply({"walls": (-1 if i == 0 else (0 if i == 1 else 1))}),
		"The reference's own checkbox cannot return to Auto once clicked (native checkboxes have no third state); this picker can.")
	DccWidgets.note(sec,
		"Both overrides are stored and neither is consumed: their only readers are "
		+ "_umInferAge/_umWallSpec in the urban-morphology layer, which milestones 8-17 have "
		+ "not ported (URBAN_MORPHOLOGY_SCOPE.md). Recorded honestly rather than hidden.")


# -- History ----------------------------------------------------------------

func _build_history(details: Dictionary) -> void:
	var sec := DccWidgets.section(_body, "History")
	var te := TextEdit.new()
	te.text = String(details.get("history", ""))
	te.placeholder_text = "Lore, founding, notable events…"
	te.custom_minimum_size.y = 84
	te.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	te.focus_exited.connect(func(): _apply({"history": te.text}))
	sec.add_child(te)


# -- Actions ----------------------------------------------------------------

func _build_actions(s: Dictionary) -> void:
	var sec := DccWidgets.section(_body, "Actions")
	DccWidgets.action(sec, "Focus camera on settlement", func():
		app.viewport.move_view_to(float(int(s.get("x", 0))), float(int(s.get("y", 0)))))
	## `peCityOpen` (`GUI_GAP_REGISTER.md` UM-03). That row predicted "a popup
	## can call it in one line" once a popup existed; this is the line.
	##
	## `has_method`-guarded because `CityViewerWindow` is a separate, parallel
	## piece of work: this window degrades to a disabled button rather than a
	## runtime error against a shell that does not carry it -- the same guard
	## discipline `engine_bridge.gd` applies to every `#[func]` it calls.
	var idx := _index
	var cv := DccWidgets.action(sec, "Open in City Viewer", func():
		if app.has_method("open_city_viewer"):
			app.open_city_viewer(idx))
	cv.disabled = not app.has_method("open_city_viewer")
	cv.tooltip_text = "The reference's peCityOpen. peCityPreview (the layout thumbnail inline in this popup) is still open -- it needs a rendered layout at icon size, not a modal."
	var del := DccWidgets.action(sec, "Delete place", func(): confirm_delete(_index))
	del.add_theme_color_override("font_color", DccTheme.c("accent"))
	del.tooltip_text = "The reference confirms first (its own v1.24 data-loss fix); so does this."


## Shared with the map context menu and the Delete key, so all three
## destructive paths ask exactly once, the same way.
func confirm_delete(index: int) -> void:
	var all := bridge.settlements()
	if index < 0 or index >= all.size():
		return
	var name := String((all[index] as Dictionary).get("name", "this place"))
	var dlg := ConfirmationDialog.new()
	dlg.title = "Delete place?"
	dlg.dialog_text = "Delete %s?\n\nProvinces, trade balances, roads and territory were computed before this edit and are not recomputed." % name
	dlg.get_ok_button().text = "Delete"
	dlg.confirmed.connect(func():
		if bridge.civ_delete_settlement(index):
			place_deleted.emit()
			if index == _index:
				_index = -1
				hide()
			else:
				_rebuild()
		dlg.queue_free())
	dlg.canceled.connect(dlg.queue_free)
	app.add_child(dlg)
	dlg.popup_centered()


func _build_footer() -> void:
	_footer = DccWidgets.note(_body,
		"Not built here, each for a stated reason: Category (settlement ↔ POI) -- POI is not a "
		+ "ported concept (GUI_GAP_REGISTER.md CV-01); the inline town-layout thumbnail (UM-03) "
		+ "-- no layout renders at icon size yet, though its City Viewer launcher above is live; "
		+ "\"Connect to road network\" -- no civ_connect_place_to_network binding; Econ. "
		+ "importance and Trade volume -- NamedSettlement carries neither field.")


## One engine call, then a refresh. `civ_edit_settlement` is all-or-nothing:
## a rejected value applies nothing, so a `false` here means the form and the
## engine disagree and the form must be rebuilt from the engine's truth,
## never left showing a value that was refused.
func _apply(fields: Dictionary) -> void:
	if _index < 0:
		return
	if not bridge.civ_edit_settlement(_index, fields):
		app.set_status("hint", "Edit refused — a value was outside the engine's own vocabulary.", "accent")
		_rebuild()
		return
	place_changed.emit()

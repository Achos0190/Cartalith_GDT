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
## Phone (§13). `DccWidgets.phone_window()`'s header comment carries the whole
## treatment and why; here it only decides whether the in-content header below
## exists and whether each rebuild is re-fitted for touch.
var _phone := false
var _phone_title: Label

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
	## Also turns `wrap_controls` off -- which this window shipped with on,
	## the third instance of that bug class in this shell.
	_phone = DccWidgets.phone_window(self, a)

	## One column rather than the scroll directly, so the phone header has
	## somewhere to sit. An `AcceptDialog` gives its *first* content child the
	## whole rect, so a second sibling would simply overlap this one.
	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 0)
	add_child(root)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_child(scroll)
	if _phone:
		_phone_title = DccWidgets.phone_head(root, "Place", "settlement editor")
	var pad := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 12)
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(pad)
	_body = VBoxContainer.new()
	_body.add_theme_constant_override("separation", 4)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_body)

	## `GUI_GAP_REGISTER.md` **RF-02**. This window is built on `open_for()`, not
	## at launch, so §23's sweep did not reach it -- but it is keyed to a
	## settlement **index**, and a generate renumbers every index there is.
	## Left open across one it went on showing the old world's place: measured
	## `Sevjuniana` pop 19 332 at (142, 14) while the engine's settlement 0 was
	## pop 19 774 at (208, 183), the form character-for-character identical.
	## Every field here writes through `civ_edit_settlement(_index, …)`, so a
	## commit from that pane would have written the previous world's name, kind
	## and traits onto whatever now sits at the index -- PE-01's failure with a
	## generate as the trigger.
	##
	## Rebuilt rather than closed, which is the shape three other windows in this
	## shell already use for the same pair of signals (`city_viewer_window`,
	## `world_data_window`, `performance_window` all `if visible: reload`).
	## Closing would also be wrong for the ambiguous half of `world_loaded`:
	## `load_asset_pack` and `as_apply_to_map` emit it without touching a single
	## settlement.
	##
	## `_rebuild()` and not `open_for(_index)`: `open_for` commits the focused
	## field first, and committing this form against the world that has just
	## replaced the one it was typed for is the bug, not the fix. `_clear()`'s
	## `_rebuilding` guard drops it instead.
	bridge.generation_finished.connect(func(ok: bool): if ok and visible: _rebuild())
	bridge.world_loaded.connect(func(): if visible: _rebuild())


## `_civOpenPlacePopup`: opens on the settlement at `index` into
## `get_settlements()`'s own array.
func open_for(index: int) -> void:
	## PE-01: flush whatever the currently-open form has focused, against the
	## settlement it belongs to, before `_index` moves. Without this an editor
	## re-opened on a different place from the map committed the old form's
	## text onto the new one.
	_commit_focused_field()
	_index = index
	_rebuild()
	if _index < 0:
		return
	if not DccWidgets.phone_present(self, app):
		popup_centered()
	## §4.5.3 asks for the editor to open "focused on the name field", and on a
	## pointer that costs nothing. On a phone a focused `LineEdit` raises the
	## IME, which on the device pass covered the form from Traits down before
	## the user had asked to rename anything -- so phone opens on the whole
	## form and the field takes focus when it is tapped, like every other field
	## here. Measured on the handset, not assumed.
	if _phone:
		return
	if _name_edit != null:
		## §4.5.3's own right-dock column asks for "the new settlement's
		## inspector, live, focused on the name field" -- which nothing in
		## this shell could honour until this window existed.
		_name_edit.grab_focus()
		_name_edit.select_all()


## `_rebuilding` guards the name field's `focus_exited` commit against its own
## teardown (`GUI_GAP_REGISTER.md` PE-01).
##
## §4.5.3 has `open_for()` focus the name field, so on desktop the field holds
## focus for the whole session. `_clear()` below removes it from the tree,
## which releases focus and fires `focus_exited` **synchronously** -- and that
## handler writes the field's text back through `civ_edit_settlement`. Any
## rebuild triggered by something that changed the name therefore had the OLD
## name written back over the new one before the rebuilt form ever read it.
##
## Measured, not reasoned: the ⟳ re-roll button left the engine name at
## `Yusnashharwell` across a real press with the field focused, and changed it
## to `Abedomarmarch` on the identical press with `release_focus()` called
## first. Every press after the first worked, because only `open_for()` grabs
## focus -- which is exactly why this survived to be found by a probe rather
## than by the eye.
var _rebuilding := false

## Commit whatever field is focused **before** `_index` moves, so a half-typed
## rename lands on the settlement it was typed for instead of being dropped by
## the guard. Releasing focus is what makes the `focus_exited` commit fire
## here, against the right index, rather than inside `_clear()`.
func _commit_focused_field() -> void:
	if _body == null:
		return
	var vp := _body.get_viewport()
	if vp == null:
		return
	var fo := vp.gui_get_focus_owner()
	if fo != null and _body.is_ancestor_of(fo):
		fo.release_focus()

func _clear() -> void:
	_rebuilding = true
	## Cleared BEFORE the removal as well, so a handler that reads it during
	## teardown sees no field rather than a dying one.
	_name_edit = null
	for c in _body.get_children():
		_body.remove_child(c)
		c.queue_free()
	_rebuilding = false


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
	if _phone_title != null:
		## The window is borderless on a phone, so the title bar this used to
		## be read from is gone -- the in-content header is where it lives now.
		_phone_title.text = String(s.get("name", "(unnamed)")).to_upper()

	_build_identity(s)
	_build_class_and_polity(s)
	_build_economy(s, details)
	_build_trade()
	_build_traits(details)
	_build_urban(details)
	_build_history(details)
	_build_knowledge(s)
	_build_actions(s)
	_build_footer()
	## Every row above comes from `dcc_widgets.gd`, which is authored in desktop
	## pixels; this floors the tappable ones at §13's 44 dp. Re-run per rebuild
	## because a rebuild makes fresh nodes, and safe to re-run because the walk
	## is idempotent (`DccShell.phone_fit`'s own meta-flag).
	if _phone:
		app.phone_fit(self, 1.0)


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
	## `_rebuilding` / null guard: this fires during `_clear()`'s own teardown
	## too, where the "edit" it would commit is the stale text of a field that
	## is being thrown away. See `_rebuilding`'s doc comment (PE-01).
	_name_edit.focus_exited.connect(func():
		if _rebuilding or _name_edit == null:
			return
		_apply({"name": _name_edit.text}))
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
		"The reference's own Polity picker. Territory is NOT repainted -- assign_territory runs inside generate() and no #[func] re-runs it (GUI_GAP_REGISTER.md CV-20). Civilization ▸ Territories ▸ Recalculate territories is the shortcut that does re-run it.")


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


# -- Trade ------------------------------------------------------------------

## `_civGoodReach`'s bulk branch, in the one sentence a reader needs: what a
## settlement's own water lets its heavy goods actually reach. Keyed by
## `cartalith_civ::trade::NavKind`'s own strings.
const REACH_BY_WATER := {
	"sea": "long -- bulk goods reach anywhere",
	"river": "regional -- bulk goods reach the river's own network",
	"stream": "local -- a headwater stream carries no cargo",
	"none": "local -- bulk goods stop at 50 km",
}

## `GUI_GAP_REGISTER.md` **IN-13**'s per-settlement half: *"imports/exports
## per settlement as a ledger"*.
##
## Here rather than in a second settlement list in the dock, because this
## window is already where a settlement's own facts live — and because a
## partner is a *name*, which only means something next to the place it
## belongs to.
##
## Reads `TradeStore`, never the engine: the match is one computation shared
## by three surfaces, and re-running it on every place-editor open is exactly
## the cost `civ_trade_flows()`'s own doc comment explains this design avoids.
## When no match has run the section says so and points at the one control
## that runs it — a section that silently showed nothing would be
## indistinguishable from a settlement that trades nothing.
func _build_trade() -> void:
	var sec := DccWidgets.section(_body, "Trade")
	if not TradeStore.is_matched():
		DccWidgets.note(sec,
			"No trade match on this world yet. Civilization ▸ Trade ▸ Match trade flows "
			+ "computes who supplies whom; it is derived on demand and held nowhere, so a "
			+ "generate clears it.")
		return

	var led := TradeStore.ledger(_index)
	var imports: Array = led.get("imports", [])
	var exports: Array = led.get("exports", [])
	var unmet := TradeStore.unmet_for(_index)

	if imports.is_empty() and exports.is_empty() and unmet.is_empty():
		DccWidgets.note(sec,
			"No trade relationship. Its hinterland is close enough to the world mean on every "
			+ "resource that nothing reads as a surplus or a shortage.")
	if not imports.is_empty():
		var g := DccWidgets.group(sec, "Imports")
		for f in imports:
			_trade_row(g, f, "from_name", "←")
	if not exports.is_empty():
		var g2 := DccWidgets.group(sec, "Exports")
		for f in exports:
			_trade_row(g2, f, "to_name", "→")
	if not unmet.is_empty():
		DccWidgets.note(sec,
			"Needs %s and nothing in reach supplies it. That is a dependency the world cannot "
			% ", ".join(unmet)
			+ "carry, not an import relationship -- the reference's own distinction.")

	var n := TradeStore.navigability(_index)
	if not n.is_empty():
		var kind := String(n.get("kind", "none"))
		var reach := String(REACH_BY_WATER.get(kind, "local"))
		DccWidgets.note(sec, "Water: %s (%s). Bulk reach is %s."
			% [kind, String(n.get("basis", "?")), reach])
	_food_shed_note(sec)
	_smelting_salt_note(sec)

## `ECONOMY_SCOPE.md` milestone 2's per-settlement half -- `_civFoodShed`, run
## for every settlement by `TradeStore.refresh()` in the same pass as the trade
## match above. Here, beside Water, because both answer the same question about
## this one place: what can actually reach it. The import term is even routed
## over the navigability `n` just read.
##
## **Every figure is a headcount, not a tonnage.** `civ_food_shed` scales each
## catchment's yield by that culture's farmers-per-urbanite ratio before summing
## (`cartalith_civ::trade::food_surplus_ratio`), so `supported` is the *urban
## population* this settlement's food logistics can carry -- which is why it is
## directly comparable to `pop`, and why the label says "people".
##
## Diagnostic only, and said so in the note: the reference reconciles the
## overshoot in `_civApplyFoodShedCeilings`, and **that function is not ported**
## (checked repo-wide 2026-09-01 -- `cartalith-civ` names it in a comment and
## nothing implements it). Nothing here clamps a population, so a reader must
## not take an unsustainable settlement for one the engine will shrink.
func _food_shed_note(sec: Control) -> void:
	var shed := TradeStore.food_shed_for(_index)
	if shed.is_empty():
		DccWidgets.note(sec, "Food shed: — no row for this settlement. civ_food_shed() "
			+ "returned nothing, which is what an engine build older than ECONOMY_SCOPE.md "
			+ "milestone 2 does.")
		return
	var supported := FactionRosterWindow._thousands(int(round(float(shed.get("supported", 0.0)))))
	var local := FactionRosterWindow._thousands(int(round(float(shed.get("local_capacity", 0.0)))))
	var hinter := FactionRosterWindow._thousands(int(round(float(shed.get("hinterland_capacity", 0.0)))))
	var suppliers := int(shed.get("suppliers", 0))
	## `best_mode` is `land` whether or not anything was imported, so it is only
	## reported when a supplier actually contributed -- the engine's own caveat.
	var import_clause := "nothing imported -- no settlement with spare capacity is in reach"
	if suppliers > 0:
		import_clause = "%s imported from %d settlement%s over %s" % [
			FactionRosterWindow._thousands(int(round(float(shed.get("import_capacity", 0.0))))),
			suppliers, "" if suppliers == 1 else "s", String(shed.get("best_mode", "land"))]
	DccWidgets.note(sec, "Food shed: supports %s people -- %s from its own catchment, %s from the "
		% [supported, local, hinter]
		+ "countryside within land reach, %s. Limited by %s." % [import_clause,
			String(shed.get("limited_by", "local"))])
	var pop := FactionRosterWindow._thousands(int(shed.get("pop", 0)))
	if bool(shed.get("sustainable", true)):
		DccWidgets.note(sec, "Population %s when the match ran, inside that ceiling." % pop)
	else:
		DccWidgets.note(sec, "Population %s when the match ran, over the ceiling by %s. "
			% [pop, FactionRosterWindow._thousands(int(round(float(shed.get("over_by", 0.0)))))]
			+ "A diagnostic, not a correction -- the reference's _civApplyFoodShedCeilings is "
			+ "not ported, so nothing here shrinks the settlement to fit.")

## `ECONOMY_SCOPE.md` EC-2/EC-7 -- `_civPlaceSmelting`/`_civSaltAccess`, run for
## every settlement by `TradeStore.refresh()` in the same pass as everything
## above. Beside the food shed for the same reason that sits beside Water: all
## three answer what this one place can actually reach, from its own ground.
##
## Smelting is gated by **fuel, not ore** -- the reference's own point, kept in
## the note rather than buried: `limited_by` names whichever of the two
## catchment budgets binds, and `fuel_poor`/`ore_rich` flag the two lopsided
## cases (ore to spare with no wood to fire it, and the converse -- a charcoal
## exporter with nothing to smelt).
func _smelting_salt_note(sec: Control) -> void:
	var smelt := TradeStore.smelting_for(_index)
	if smelt.is_empty():
		DccWidgets.note(sec, "Smelting: — no row for this settlement. civ_place_smelting() "
			+ "returned nothing, the same defensive case civ_food_shed()'s own reader states above.")
	elif float(smelt.get("iron_kg_yr", 0.0)) <= 0.0:
		DccWidgets.note(sec, "Smelting: none possible here -- ore, fuel, or both are absent from "
			+ "this catchment.")
	else:
		var iron := FactionRosterWindow._thousands(int(round(float(smelt.get("iron_kg_yr", 0.0)))))
		var ore := FactionRosterWindow._thousands(int(round(float(smelt.get("ore_kg_yr", 0.0)))))
		var charcoal := FactionRosterWindow._thousands(int(round(float(smelt.get("charcoal_kg_yr", 0.0)))))
		var woodland := FactionRosterWindow._thousands(int(round(float(smelt.get("woodland_ha", 0.0)))))
		var flag := ""
		if bool(smelt.get("fuel_poor", false)):
			flag = " Fuel-poor: ore to spare, not enough woodland to fire it."
		elif bool(smelt.get("ore_rich", false)):
			flag = " A charcoal exporter: more fuel here than this catchment's ore can use."
		DccWidgets.note(sec, ("Smelting: %s kg iron/yr, limited by %s -- %s kg ore/yr, %s kg "
			+ "charcoal/yr from %s ha of woodland.%s") % [iron, String(smelt.get("limited_by", "ore")),
				ore, charcoal, woodland, flag])

	var salt := TradeStore.salt_access_for(_index)
	if salt.is_empty():
		DccWidgets.note(sec, "Salt: — no row for this settlement. civ_salt_access() "
			+ "returned nothing, the same defensive case civ_food_shed()'s own reader states above.")
	elif bool(salt.get("has", false)):
		DccWidgets.note(sec, "Salt: yes, from %s." % String(salt.get("source", "?")))
	else:
		DccWidgets.note(sec, "Salt: none in reach -- no sea route, salt deposit or salt lake.")

func _trade_row(parent: Control, flow: Variant, name_key: String, arrow: String) -> void:
	var f: Dictionary = flow
	var partner := int(f.get("from" if name_key == "from_name" else "to", -1))
	var b := DccWidgets.action(parent, "%s %s %s -- %s, %s %d km" % [
		String(f.get("good", "?")), arrow, String(f.get(name_key, "?")),
		FactionRosterWindow._thousands(int(round(float(f.get("volume", 0.0))))),
		String(f.get("mode", "land")), int(round(float(f.get("distance_km", 0.0))))],
		func():
			if partner >= 0:
				app.open_place_editor(partner))
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.tooltip_text = ("%s reach; %d%% of the partner's scale survives the carriage. "
		+ "Opens the partner's own place editor.") % [
			String(f.get("reach", "?")),
			int(round(100.0 * float(f.get("deliverable", 0.0))))]


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
		## `accent_ink` on the filled half since the 2026-08-31 re-base -- see
		## the token's own comment; `c("bg")` here was near-black on amber.
		b.add_theme_color_override("font_color",
			DccTheme.c("accent_ink") if is_on else DccTheme.c("text_dim"))
		b.add_theme_stylebox_override("normal",
			DccTheme.flat(DccTheme.c("accent") if is_on else DccTheme.c("sunken")))
		## **The off-chip's hover was inverted by the 2026-08-31 token re-base.**
		## It was `raised`, which is a lift over the old `sunken` and is not one
		## over the new: `--ins` moved #101112 -> #191c1e, so on dark the hover
		## went from *lighter by (7,8,8)* to *darker by (2,3,4)* -- a hover that
		## reads as a press, at a delta too small to read as anything.
		##
		## `outline("border", "sunken")` keeps the well and adds the hairline edge
		## the shell already uses for a chip hover (`dcc_widgets.gd`'s `outline(...)`
		## pair), so the affordance is an edge rather than a fill and works the
		## same in both palettes -- `border` is rgba(255,255,255,.16) on dark and
		## rgba(0,0,0,.20) on light. Both are real tokens, so `DccTheme.remap()`
		## repaints them on a theme flip; a `lightened()`/blended literal would not
		## be matched by either of its passes.
		b.add_theme_stylebox_override("hover",
			DccTheme.flat(DccTheme.c("accent").lightened(0.1)) if is_on
			else DccTheme.outline("border", "sunken"))
		b.pressed.connect(func():
			bridge.civ_settlement_toggle_trait(_index, key)
			## The map's badge row is joined by `tid` from
			## `civ_settlement_details()`, so it only changes when something
			## re-reads that -- a toggle here is the one edit that moves it.
			if app != null and app.viewport != null and app.viewport.has_method("refresh_settlement_traits"):
				app.viewport.refresh_settlement_traits()
			_rebuild())
		flow.add_child(b)
	sec.add_child(flow)
	DccWidgets.note(sec,
		"Map-glyph badges, deliberately overlapping Economy on mining/trade hub -- the "
		+ "reference keeps both vocabularies on purpose. Toggling one draws it under the "
		+ "settlement's pin immediately (map_overlay.gd's `_draw_trait_badges`, the "
		+ "reference's own no-art disc-and-glyph branch; no asset-pack sprite reaches a "
		+ "Godot draw call yet).")


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
	## **Corrected 2026-09-03.** This note used to give the reason as
	## "_umInferAge/_umWallSpec ... which milestones 8-17 have not ported". Both
	## are ported -- `cartalith_civ::urban_adapter::um_infer_age` and
	## `cartalith_civ::military::um_wall_spec`, the latter reading
	## `walls_override` and `age_override` at its first two branches. The
	## conclusion held and the reason did not. Re-cut 2026-09-03: the reason
	## given here was itself falsified in the same batch that wrote it --
	## `urban_adapter`'s `WallPlace` now reads both off a `PlaceOverrides`
	## struct. The one remaining gap is delivery at a single call site,
	## `urban_bridge.rs`, which still calls `settlement_layout()` (supplying
	## `PlaceOverrides::default()`) rather than `settlement_layout_with()`.
	DccWidgets.note(sec,
		"Both overrides are stored and neither reaches the layout yet -- but every "
		+ "piece below them is now built. cartalith-civ's um_infer_age and "
		+ "um_wall_spec are live, um_wall_spec branches on walls_override and "
		+ "age_override before anything else, and the adapter's WallPlace reads "
		+ "them straight off a PlaceOverrides struct (urban_adapter.rs) rather "
		+ "than hardcoding None. What is missing is one call site: "
		+ "urban_bridge.rs still calls settlement_layout(), the entry point that "
		+ "supplies PlaceOverrides::default(), instead of the _with() variant "
		+ "that would carry this editor's values. Recorded honestly rather than "
		+ "hidden.")


# -- History ----------------------------------------------------------------

func _build_history(details: Dictionary) -> void:
	var sec := DccWidgets.section(_body, "History")
	var te := TextEdit.new()
	te.text = String(details.get("history", ""))
	te.placeholder_text = "Lore, founding, notable events…"
	te.custom_minimum_size.y = 84
	te.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	## Same guard as the name field above (PE-01). The cross-settlement case is
	## the dangerous one: `open_for()` sets `_index` and only then rebuilds, so
	## an unguarded teardown commit writes THIS settlement's history onto the
	## one the user just opened.
	te.focus_exited.connect(func():
		if _rebuilding:
			return
		_apply({"history": te.text}))
	sec.add_child(te)


# -- Knowledge (Markdown vault) ---------------------------------------------

## `MARKDOWN_VAULT_INTEGRATION.md` §28's KNOWLEDGE block, which the design
## sketches inside exactly this panel: *"The Markdown functionality should
## appear in entity information panels rather than as an isolated utility."*
##
## What lives here is the affordance and the status; the browser, the reader
## and the two write actions live in `vault_window.gd`, which this opens
## already scoped to this settlement.
##
## **Keyed by `tid`, not by `_index`.** A settlement's index shifts every time
## an earlier one is deleted, and a knowledge link that followed the index
## would silently re-point at a different town. `tid` is this port's own
## stable id and is what the engine stores.
func _build_knowledge(s: Dictionary) -> void:
	var sec := DccWidgets.section(_body, "Knowledge")
	var tid := int(s.get("tid", 0))
	if tid == 0:
		DccWidgets.note(sec, "This settlement has no stable id yet, so nothing can be linked to it.")
		return
	var name := String(s.get("name", "this settlement"))
	var summary := bridge.vault_entity_summary("settlement", tid)
	var n := int(summary.get("link_count", 0))
	if n == 0:
		DccWidgets.note(sec, "No Markdown notes linked.")
	else:
		DccWidgets.note(sec, "%d linked note%s — %s" % [n, "" if n == 1 else "s",
			String(VaultWindow.STATUS_TEXT.get(String(summary.get("status", "")), ""))])
	var open := DccWidgets.action(sec, "Linked notes…" if n > 0 else "Attach a Markdown note…", func():
		app.open_vault("settlement", tid, name))
	open.tooltip_text = ("Links this settlement to a note in an external Markdown vault — any folder "
		+ "of .md files, Obsidian's included, and nothing here requires Obsidian. Cartalith reads on "
		+ "demand and writes only on an explicit, previewed action.")
	_build_backlinks(sec, "settlement", tid, name)


## `GUI_GAP_REGISTER.md` **VA-01**: what points *at* this entity, as opposed to
## what it points at.
##
## Three states, kept apart because on screen they mean different things:
##
## - **not indexed** — say so and offer the one control that fixes it. An
##   empty list here would read as "nothing references this place", which is a
##   claim the shell has no basis for until the index exists.
## - **backlinks** — exact references. A `block` row is a note carrying this
##   entity's own Cartalith block, which finds it even when it has no note of
##   its own; it is not a link and is labelled differently.
## - **unlinked mentions** — a name in prose, and a *guess*. Visually
##   subordinate, with the excerpt shown so the reader can judge it, because a
##   place called Nareth and a person called Nareth read identically to a
##   substring match.
##
## Static-shaped enough to live in the base window rather than in each caller:
## the faction roster and the province rows call it with their own kind.
func _build_backlinks(sec: Control, kind: String, entity_id: int, name: String) -> void:
	var stats := bridge.vault_backlink_stats()
	if not bool(stats.get("built", false)):
		var note := DccWidgets.note(sec,
			"Backlinks are not indexed for this vault yet. Building the index reads every "
			+ "note once; after that a refresh only re-opens the files that changed.")
		note.tooltip_text = "Data ▸ Vault index… builds it."
		return

	var back: Array = bridge.vault_entity_backlinks(kind, entity_id)
	if back.is_empty():
		DccWidgets.note(sec, "No note references this %s." % kind)
	else:
		var g := DccWidgets.group(sec, "Backlinks (%d)" % back.size())
		for row in back:
			var d: Dictionary = row
			var rel := String(d.get("rel", ""))
			var form := String(d.get("form", "wiki"))
			var count := int(d.get("count", 1))
			var b := DccWidgets.action(g, "%s%s" % [rel, "" if count < 2 else "  ×%d" % count],
				func(): app.open_vault_overview())
			b.alignment = HORIZONTAL_ALIGNMENT_LEFT
			b.tooltip_text = {
				"wiki": "A [[wikilink]] in this note points here.",
				"markdown": "A [markdown](link) in this note points here.",
				"block": "This note carries a Cartalith block naming %s:%d directly, so the reference survives a rename of the note and of the %s." % [kind, entity_id, kind],
			}.get(form, "") + " Opens the vault panel."

	var mentions: Array = bridge.vault_entity_mentions(kind, entity_id, name, 8)
	if mentions.is_empty():
		return
	var mg := DccWidgets.group(sec, "Unlinked mentions (%d)" % mentions.size(), false)
	DccWidgets.note(mg,
		"A guess: these notes contain the name and do not link here. Cartalith opened only "
		+ "the files its index said could match, and it changed none of them.")
	for row in mentions:
		var d: Dictionary = row
		DccWidgets.note(mg, "%s
    %s" % [String(d.get("rel", "")), String(d.get("excerpt", ""))])


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
	dlg.dialog_text = "Delete %s?\n\nProvinces, trade balances, roads and territory were computed before this edit and are not recomputed by the delete itself. The Civilization dock's Settlements ▸ Recompute civilisation rebuilds them against the current roster and terrain." % name
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

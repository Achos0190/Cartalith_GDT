extends AcceptDialog
class_name FactionRosterWindow

## `civOpenFactionsBtn` → `civFactionsModal` (`_civOpenFactionsModal`,
## reference 16177; `_civRenderFactionList`; `_civPopulateFactionEditor`,
## 16247) -- `PARITY_AUDIT.md` §5 items 9 and 10, `GUI_GAP_REGISTER.md`
## CV-07 / MS-13.
##
## The register said add/remove faction was absent and `CIV_FACTION_COUNT`
## "a compile-time constant … so there is no roster to add to or remove
## from -- get_factions() enumerates a fixed set, it does not own one." That
## is no longer true: `CivData::faction_roster` owns one, and this window is
## the reference's three-part modal over it -- world overview, faction list,
## and the Inspector drawer with its five editable fields, its procedural
## banner, its Territory-fit verdict and its settlement sublist.
##
## ## What is real, and what is not
##
## Real: name/culture/religion/government/ag-tech editing (all five persist
## and all five are validated against the engine's own vocabularies),
## add/remove faction (with the reference's own revert-to-Unclaimed side
## effect), the procedural banner (a port of `_civFactionBannerCanvas`'s
## actual composition, not a redesign), Territory fit (a real
## `civ_culture_terrain_fit` verdict over a real `civ_faction_aggregates`
## terrain mix), settlement count / population / territory km² / capital.
##
## Not built, and said so in-window rather than only here: the reference's
## **Power breakdown** (five axes) and **Economy** block (food production,
## tax, trade, exports/imports, strategic resources, craft share). Both come
## from `_civFactionAggregates`' resource- and density-fed half, and
## `compute_civilisation` frees the resource rasters and never retains a
## population-density field for this -- surfacing them means a memory
## decision (`MEMORY_OPTIMIZATION_SCOPE.md`) and an `ECONOMY_SCOPE.md`
## milestone, not a widget. **Diplomacy** has no model at all, in either
## codebase; the reference's own inspector says "not yet implemented" there
## and so does this.

var app                       ## `DccApp`
var bridge: EngineBridge

var _selected := 1
var _list_body: VBoxContainer
var _inspector_body: VBoxContainer
var _overview: Label
var _fits: Array = []         ## `civ_faction_terrain_fits()`, cached per open.

## Emitted whenever the roster changes in a way that moves map data (a
## removed faction reverts settlements and territory to Unclaimed).
signal roster_changed


func setup(a, b: EngineBridge) -> void:
	app = a
	bridge = b
	title = "Faction roster"
	size = Vector2i(880, 620)
	min_size = Vector2i(620, 420)
	## Both panes scroll, so the window must not grow to fit their content --
	## without this the inspector's own prose pushes the dialog past the
	## viewport and pushes the Add/Remove row off the bottom of the screen.
	max_size = Vector2i(1000, 700)

	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 6)
	add_child(outer)

	_overview = DccTheme.label("", "text_dim", DccTheme.FS_SMALL)
	_overview.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	outer.add_child(_overview)
	outer.add_child(DccTheme.rule())

	var split := HBoxContainer.new()
	split.add_theme_constant_override("separation", 10)
	split.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(split)

	var left := VBoxContainer.new()
	left.custom_minimum_size.x = 250
	left.add_theme_constant_override("separation", 4)
	split.add_child(left)
	var list_scroll := ScrollContainer.new()
	list_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	list_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	left.add_child(list_scroll)
	_list_body = VBoxContainer.new()
	_list_body.add_theme_constant_override("separation", 2)
	_list_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	list_scroll.add_child(_list_body)

	var roster_row := HBoxContainer.new()
	roster_row.add_theme_constant_override("separation", 6)
	DccWidgets.action(roster_row, "+ Add faction", _add_faction)
	DccWidgets.action(roster_row, "− Remove last", _confirm_remove)
	left.add_child(roster_row)

	split.add_child(DccTheme.rule(true))

	var right_scroll := ScrollContainer.new()
	right_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	right_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	split.add_child(right_scroll)
	_inspector_body = VBoxContainer.new()
	_inspector_body.add_theme_constant_override("separation", 4)
	_inspector_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_scroll.add_child(_inspector_body)


func open() -> void:
	## Cached once per open, not per faction row: the underlying pass is
	## O(cells) and rebuilds a biome raster and an ocean-distance field --
	## see `civ_faction_terrain_fits`' own Rust doc comment.
	_fits = bridge.civ_faction_terrain_fits()
	_rebuild()
	popup_centered()


func _clear(node: Control) -> void:
	for c in node.get_children():
		node.remove_child(c)
		c.queue_free()


func _rebuild() -> void:
	_rebuild_overview()
	_rebuild_list()
	_rebuild_inspector()


# -- World overview (`_civRenderFactionsWorldOverview`) ----------------------

func _rebuild_overview() -> void:
	var factions := bridge.get_factions()
	if factions.is_empty():
		_overview.text = "Generate a world to see a faction summary here."
		return
	var total_pop := 0
	var total_settle := 0
	var total_cells := 0
	var top_name := ""
	var top_pop := -1
	for f in factions:
		var d: Dictionary = f
		var pop := int(d.get("population", 0))
		total_pop += pop
		total_settle += int(d.get("settlement_count", 0))
		total_cells += int(d.get("claimed_cells", 0))
		if pop > top_pop:
			top_pop = pop
			top_name = String(d.get("name", "?"))
	var agr := bridge.civ_agrarian_regional_total()
	var land_line := ""
	if not agr.is_empty():
		land_line = "  ·  Land sustains ≈ %s across %s km²" % [
			_thousands(int(agr.get("sustains", 0))), _thousands(int(agr.get("land_km2", 0)))]
	_overview.text = "%d factions  ·  %s total settled population  ·  %d settlements  ·  %d claimed cells%s\nLargest by population: %s (%s)" % [
		factions.size(), _thousands(total_pop), total_settle, total_cells, land_line,
		top_name, _thousands(top_pop)]


# -- Faction list (`_civRenderFactionList`) ---------------------------------

func _rebuild_list() -> void:
	_clear(_list_body)
	var factions := bridge.get_factions()
	if factions.is_empty():
		DccWidgets.note(_list_body, "No world generated -- File ▸ New world… to begin.")
		return
	for f in factions:
		var d: Dictionary = f
		var fid := int(d.get("id", 1))
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)
		## The banner is a `Control` with its own `_draw()`, so it goes
		## beside the button rather than into `Button.icon` (which wants a
		## `Texture2D` and would need a `SubViewport` round trip to get one).
		var banner := FactionBanner.new()
		banner.configure(fid, _color_of(d), 22)
		row.add_child(banner)
		var b := Button.new()
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.custom_minimum_size.y = 30
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.text = "%s — %d settlements, %s pop" % [
			String(d.get("name", "?")), int(d.get("settlement_count", 0)),
			_thousands(int(d.get("population", 0)))]
		if fid == _selected:
			b.add_theme_stylebox_override("normal", DccTheme.flat(DccTheme.c("sunken"), 3))
		b.pressed.connect(func(): _selected = fid; _rebuild_list(); _rebuild_inspector())
		row.add_child(b)
		_list_body.add_child(row)


# -- Inspector (`_civPopulateFactionEditor`) --------------------------------

func _rebuild_inspector() -> void:
	_clear(_inspector_body)
	var d := _faction(_selected)
	if d.is_empty():
		DccWidgets.note(_inspector_body, "Select a faction.")
		return

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 8)
	var banner := FactionBanner.new()
	banner.configure(_selected, _color_of(d), 48)
	head.add_child(banner)
	var name_edit := LineEdit.new()
	name_edit.text = String(d.get("name", ""))
	name_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_edit.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	name_edit.text_submitted.connect(func(t: String): _set_field("name", t))
	name_edit.focus_exited.connect(func(): _set_field("name", name_edit.text))
	head.add_child(name_edit)
	_inspector_body.add_child(head)

	var sec := DccWidgets.section(_inspector_body, "Identity")
	_vocab_choice(sec, "Government", bridge.civ_government_vocabulary(),
		String(d.get("government", "monarchy")), "government")
	_culture_choice(sec, String(d.get("culture", "common")))
	_vocab_choice(sec, "Religion", bridge.civ_religion_vocabulary(),
		String(d.get("religion", "none")), "religion")
	_ag_tech_choice(sec, String(d.get("ag_tech", "traditionalAgrarian")))

	_build_terrain_fit()
	_build_overview_block(d)
	_build_settlement_sublist()
	_build_gaps()


func _vocab_choice(parent: Control, label_text: String, vocab: Array, current: String, key: String) -> void:
	if vocab.is_empty():
		return
	var keys: Array = []
	var labels: Array = []
	for e in vocab:
		var d: Dictionary = e
		keys.append(String(d.get("key", "")))
		labels.append(String(d.get("label", "?")))
	DccWidgets.choice(parent, label_text, labels, maxi(0, keys.find(current)),
		func(i: int): _set_field(key, keys[i]))


## Cultures come back as bare keys -- the reference's own `CIV_CULTURES`
## carries no display label, so capitalising the key is the honest render,
## not an invented label table.
func _culture_choice(parent: Control, current: String) -> void:
	var keys := bridge.civ_culture_vocabulary()
	if keys.is_empty():
		return
	var labels: Array = []
	for k in keys:
		labels.append(String(k).capitalize())
	DccWidgets.choice(parent, "Culture", labels, maxi(0, Array(keys).find(current)),
		func(i: int): _set_field("culture", String(keys[i])),
		"Naming culture -- the pool _civSettleName draws this faction's settlement names from. Also what Territory fit below judges the land against.")


func _ag_tech_choice(parent: Control, current: String) -> void:
	var vocab := bridge.civ_ag_tech_vocabulary()
	if vocab.is_empty():
		return
	var keys: Array = []
	var labels: Array = []
	var hint := ""
	for e in vocab:
		var d: Dictionary = e
		keys.append(String(d.get("key", "")))
		labels.append(String(d.get("label", "?")))
		if String(d.get("key", "")) == current:
			hint = String(d.get("hint", ""))
	DccWidgets.choice(parent, "Ag. technology", labels, maxi(0, keys.find(current)),
		func(i: int): _set_field("ag_tech", keys[i]),
		"Stored and validated, but consumed by nothing here: its only readers in the reference are _civFoodShed/foodSurplusRatio, neither of which is ported. Recorded rather than hidden.")
	if hint != "":
		DccWidgets.note(parent, hint)


# -- Territory fit (`_civTerrainFitHtml`) -----------------------------------

func _build_terrain_fit() -> void:
	var sec := DccWidgets.section(_inspector_body, "Territory fit")
	var fit := _fit_for(_selected)
	if fit.is_empty():
		DccWidgets.note(sec, "Reopen the roster to recompute terrain composition.")
		return
	var mix: Dictionary = fit.get("mix", {})
	var parts: Array[String] = []
	for k in ["river", "coast", "arid", "forest", "hills"]:
		parts.append("%s %d%%" % [String(k).capitalize(), int(round(float(mix.get(k, 0.0)) * 100.0))])
	if bool(fit.get("has_verdict", false)):
		var verdict := String(fit.get("verdict", "typical"))
		var word := "a strong match" if verdict == "match" else ("a mismatch" if verdict == "mismatch" else "roughly typical")
		var line := DccTheme.label("%s territory: %d%% vs. world average %d%% — %s for a %s culture." % [
			String(fit.get("key", "?")).capitalize(),
			int(round(float(fit.get("value", 0.0)) * 100.0)),
			int(round(float(fit.get("world_mean", 0.0)) * 100.0)),
			word, String(fit.get("culture", "?")).capitalize()], "text", DccTheme.FS_SMALL)
		line.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		if verdict == "match":
			line.add_theme_color_override("font_color", Color(0.48, 0.78, 0.49))
		elif verdict == "mismatch":
			line.add_theme_color_override("font_color", DccTheme.c("accent"))
		sec.add_child(line)
	else:
		DccWidgets.note(sec,
			"%s culture has no terrain theme — composition shown for reference only. The engine "
			% String(fit.get("culture", "?")).capitalize()
			+ "returns no verdict for common/imperial rather than fabricating one, which is the "
			+ "reference's own discipline.")
	DccWidgets.note(sec, " · ".join(parts))


# -- Overview block ---------------------------------------------------------

func _build_overview_block(d: Dictionary) -> void:
	var sec := DccWidgets.section(_inspector_body, "Overview")
	var stats := bridge.civ_faction_territory_stats(_selected)
	var cap := _capital_of(_selected)
	var cap_name: String = String(cap.get("name", "")) if not cap.is_empty() else "none"
	DccWidgets.note(sec, "Capital: %s   ·   Settlements: %d   ·   Settled population: %s" % [
		cap_name, int(d.get("settlement_count", 0)), _thousands(int(d.get("population", 0)))])
	if not stats.is_empty():
		DccWidgets.note(sec, "Territory: %s km² over %d claimed cells (%d contested)" % [
			_thousands(int(float(stats.get("area_km2", 0.0)))),
			int(stats.get("claimed_cells", 0)), int(stats.get("contested_cells", 0))])
	if not cap.is_empty():
		DccWidgets.action(sec, "Focus camera on capital", func():
			app.viewport.move_view_to(float(int(cap.get("x", 0))), float(int(cap.get("y", 0))))
			hide())


# -- Settlement sublist (`_civRenderFactionSettlementSublist`) --------------

func _build_settlement_sublist() -> void:
	var all := bridge.settlements()
	var mine: Array = []
	for i in all.size():
		var s: Dictionary = all[i]
		if int(s.get("faction", 0)) == _selected:
			mine.append({"index": i, "data": s})
	var grp := DccWidgets.group(_inspector_body, "Settlements (%d)" % mine.size(), false)
	if mine.is_empty():
		DccWidgets.note(grp, "No settlements yet. Paint territory or drop one with the Settlement tool.")
		return
	mine.sort_custom(func(a, b): return int(a.data.population) > int(b.data.population))
	for e in mine:
		var s: Dictionary = e.data
		var idx: int = e.index
		var b := DccWidgets.action(grp, "%s — %s, %s" % [
			String(s.get("name", "?")), String(s.get("kind", "?")).capitalize(),
			_thousands(int(s.get("population", 0)))],
			func():
				app.viewport.move_view_to(float(int(s.get("x", 0))), float(int(s.get("y", 0))))
				hide()
				app.open_place_editor(idx))
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.tooltip_text = "Centre the map on it and open its editor -- the reference's own sublist row action."


func _build_gaps() -> void:
	var sec := DccWidgets.section(_inspector_body, "Not built")
	DccWidgets.note(sec,
		"The reference's Power breakdown (military/economic/political/cultural/religious) and its "
		+ "Economy block (food production and surplus, tax income, trade income, primary exports "
		+ "and imports, strategic resources, craft share) both read _civFactionAggregates' "
		+ "resource- and density-fed half. compute_civilisation frees the 15 resource rasters "
		+ "after trade balances and retains no population-density field, so surfacing those is a "
		+ "memory decision (MEMORY_OPTIMIZATION_SCOPE.md) plus an ECONOMY_SCOPE.md milestone, not "
		+ "a widget. Diplomatic relations has no model in either codebase -- the reference's own "
		+ "inspector says \"not yet implemented\" there too.")


# -- Roster mutation --------------------------------------------------------

func _add_faction() -> void:
	var id := bridge.civ_add_faction()
	if id < 0:
		app.set_status("hint", "Generate a world before adding a faction.", "accent")
		return
	_selected = id
	_fits = bridge.civ_faction_terrain_fits()
	_rebuild()
	app.set_status("hint",
		"Faction %d added — it owns nothing until you paint territory or reassign a settlement." % id,
		"text_ghost")


## The reference confirms first, and names the consequence in the prompt
## ("Any settlements/territory using it will revert to Unclaimed") -- both
## kept, because both are the point.
func _confirm_remove() -> void:
	if bridge.civ_faction_count() <= 1:
		app.set_status("hint", "One faction plus Unclaimed is the floor — nothing to remove.", "accent")
		return
	var dlg := ConfirmationDialog.new()
	dlg.title = "Remove the last faction?"
	dlg.dialog_text = "Remove faction %d?\n\nAny settlements and territory using it will revert to Unclaimed." % bridge.civ_faction_count()
	dlg.get_ok_button().text = "Remove"
	dlg.confirmed.connect(func():
		if bridge.civ_remove_faction():
			_selected = mini(_selected, bridge.civ_faction_count())
			_fits = bridge.civ_faction_terrain_fits()
			roster_changed.emit()
			_rebuild()
		dlg.queue_free())
	dlg.canceled.connect(dlg.queue_free)
	app.add_child(dlg)
	dlg.popup_centered()


func _set_field(key: String, value: String) -> void:
	if not bridge.civ_set_faction_field(_selected, key, value):
		app.set_status("hint", "Rejected — %s is not a value the engine recognises." % value, "accent")
	if key == "culture":
		## The verdict is a function of culture; nothing else here changes it.
		_rebuild_inspector()
	elif key == "name":
		_rebuild_list()
		_rebuild_overview()
		roster_changed.emit()


# -- Helpers ----------------------------------------------------------------

func _faction(fid: int) -> Dictionary:
	for f in bridge.get_factions():
		if int((f as Dictionary).get("id", -1)) == fid:
			return f
	return {}


func _fit_for(fid: int) -> Dictionary:
	for f in _fits:
		if int((f as Dictionary).get("faction", -1)) == fid:
			return f
	return {}


func _capital_of(fid: int) -> Dictionary:
	## `_civFactionCapital`: the highest-pop capital/metropolis, else the
	## highest-pop settlement of any kind -- fully derived from kind/pop, no
	## override field, exactly as the reference derives it.
	var best := {}
	var best_pop := -1
	var best_seat := false
	for s in bridge.settlements():
		var d: Dictionary = s
		if int(d.get("faction", 0)) != fid:
			continue
		var kind := String(d.get("kind", ""))
		var seat := kind == "capital" or kind == "metropolis"
		var pop := int(d.get("population", 0))
		if (seat and not best_seat) or (seat == best_seat and pop > best_pop):
			best = d
			best_pop = pop
			best_seat = seat
	return best


func _color_of(d: Dictionary) -> Color:
	return Color8(int(d.get("color_r", 150)), int(d.get("color_g", 150)), int(d.get("color_b", 150)))


static func _thousands(n: int) -> String:
	var s := str(absi(n))
	var out := ""
	var c := 0
	for i in range(s.length() - 1, -1, -1):
		out = s[i] + out
		c += 1
		if c % 3 == 0 and i > 0:
			out = " " + out
	return ("-" if n < 0 else "") + out

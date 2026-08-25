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
## `civ_military_summary()`, cached per open for the same reason `_fits` is:
## one call carries `power.military`, the fortification counts and the whole
## of `cartalith_civ::manpower`'s answer for every faction at once.
var _military: Dictionary = {}

## Phone (§13). The window's own treatment is
## `DccWidgets.phone_window()`'s; what is specific to *this* window is that
## master-detail does not survive the width. The list pane is 250 px and the
## inspector wants the rest, which at a 393 dp reference leaves the inspector
## 140 -- narrower than a single one of its own vocabulary pickers. So phone
## runs the classic master-*then*-detail: the list is a screen until a faction
## is picked, after which it folds into a 52 dp bar carrying that faction's
## banner and name, and the bar is what reopens it.
var _phone := false
var _phone_list_pane: Control
var _phone_list_bar: PanelContainer
var _phone_bar_name: Label
var _phone_bar_banner: Control
## The inspector head's own banner, held so the colour picker can repaint it
## live without rebuilding the inspector under the open picker (CV-21).
var _head_banner: FactionBanner

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
	## Also turns `wrap_controls` off -- which this window shipped with on,
	## despite its own `max_size` comment describing exactly the symptom
	## ("pushes the Add/Remove row off the bottom of the screen"). `max_size`
	## treated it; this is the cause.
	_phone = DccWidgets.phone_window(self, a)

	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 6)
	## Phone: **one** scrolling column, not two nested scrolling panes inside a
	## third expanding one. Desktop's shape -- a `SIZE_EXPAND_FILL` split holding
	## two `SIZE_EXPAND_FILL` `ScrollContainer`s -- does not survive an
	## `AcceptDialog` on a phone: the dialog laid this column out at 377 x 2 619
	## inside a 393 x 852 window, so the panes had nothing to scroll and their
	## lower two thirds were simply off the screen. The place editor, whose body
	## is a single scroll with no nesting, comes out at 377 x 797 in the same
	## window from the same helper -- which is what identified the nesting as the
	## cause rather than the window size. Phone follows that shape.
	##
	## It is also the better phone design independently: nested scroll regions
	## on a touch screen make every drag ambiguous, and the design canvas's own
	## roster artboard is one column from the overview to the settlement list.
	if _phone:
		var root := ScrollContainer.new()
		root.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		root.size_flags_vertical = Control.SIZE_EXPAND_FILL
		add_child(root)
		outer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		root.add_child(outer)
	else:
		add_child(outer)

	_overview = DccTheme.label("", "text_dim", DccTheme.FS_SMALL)
	_overview.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	outer.add_child(_overview)
	outer.add_child(DccTheme.rule())

	if _phone:
		_phone_list_bar = _build_phone_list_bar()
		outer.add_child(_phone_list_bar)

	## Side by side on a pointer; stacked on a phone, where only one of the two
	## panes is ever visible at a time.
	var split: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	split.add_theme_constant_override("separation", 10)
	if not _phone:
		split.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(split)

	var left := VBoxContainer.new()
	if not _phone:
		left.custom_minimum_size.x = 250
	left.add_theme_constant_override("separation", 4)
	split.add_child(left)
	_phone_list_pane = left
	var list_host: Control = left
	if not _phone:
		var list_scroll := ScrollContainer.new()
		list_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		list_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
		left.add_child(list_scroll)
		list_host = list_scroll
	_list_body = VBoxContainer.new()
	_list_body.add_theme_constant_override("separation", 2)
	_list_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	list_host.add_child(_list_body)

	var roster_row := HBoxContainer.new()
	roster_row.add_theme_constant_override("separation", 6)
	DccWidgets.action(roster_row, "+ Add faction", _add_faction)
	DccWidgets.action(roster_row, "− Remove last", _confirm_remove)
	## Stays under the list it acts on, on both form factors. Pinning it to the
	## window foot on the phone was tried first and is what found the layout
	## trap underneath: an `AcceptDialog` sizes its content child from a resize
	## notification, so anything laid out *after* a `SIZE_EXPAND_FILL` pane can
	## be pushed past the bottom of a window that was resized while hidden --
	## measured at 2 611 px of content in an 852 px window. Below the list is
	## also where these two belong: they change the roster, and the roster is
	## what the list is.
	left.add_child(roster_row)
	if not _phone:
		split.add_child(DccTheme.rule(true))

	var inspector_host: Control = split
	if not _phone:
		var right_scroll := ScrollContainer.new()
		right_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		right_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		right_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
		split.add_child(right_scroll)
		inspector_host = right_scroll
	_inspector_body = VBoxContainer.new()
	_inspector_body.add_theme_constant_override("separation", 4)
	_inspector_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	inspector_host.add_child(_inspector_body)

	if _phone:
		DccWidgets.phone_head(outer, "Faction roster", "world politics")
		_set_phone_list_open(true)

	## `GUI_GAP_REGISTER.md` **RF-03**. §23 asked "what re-runs this, and on
	## which signal?" of every panel built at launch; this window is built on
	## `open()` instead, which is correct only if nothing can change while it is
	## up. A world can. Left open across a generate the roster went on showing
	## the previous world's factions -- measured Aurelia:27 / Veldmark:49 /
	## Mirelle:57 against a live engine reading Aurelia:57 / Veldmark:27 /
	## Mirelle:7 -- and every editable control in it writes by faction **id**,
	## which the new world reuses. So a rename or a culture change committed
	## from that stale pane lands on a different faction than the one on screen:
	## FR-02's data-corruption mode with a generate as the trigger instead of a
	## click.
	bridge.generation_finished.connect(func(ok: bool): if ok and visible: _on_world_changed())
	bridge.world_loaded.connect(func(): if visible: _on_world_changed())


## Both cached fields are per-world and cached per `open()`, so both have to be
## re-taken here rather than only rebuilding the panes over stale numbers.
## `_selected` is reset because it is an id into a roster the new world has
## replaced -- the same reason `civilization_workspace._on_world_changed()`
## resets its own `_selected_index`.
##
## Nothing commits on the way out: `_rebuild()` goes through `_clear()`, whose
## `_rebuilding` guard is exactly what stops a dying focused field writing its
## text into the world that just replaced the one it was typed for (FR-02).
func _on_world_changed() -> void:
	_fits = bridge.civ_faction_terrain_fits()
	_military = bridge.civ_military_summary()
	_selected = 1
	_rebuild()


# -- Phone: the folded master pane -------------------------------------------

## The bar the list folds into: the selected faction's own banner, its name,
## its position in the roster, and a chevron. 52 dp, the canvas's list-row
## height, because it *is* a list row -- the one row of the list still worth
## showing once a choice has been made.
func _build_phone_list_bar() -> PanelContainer:
	var bar := PanelContainer.new()
	bar.add_theme_stylebox_override("panel", DccTheme.panel("raised", {"bottom": 1}))
	bar.custom_minimum_size.y = 52
	bar.mouse_filter = Control.MOUSE_FILTER_STOP
	bar.gui_input.connect(func(ev: InputEvent):
		var tapped: bool = (ev is InputEventMouseButton and (ev as InputEventMouseButton).pressed) \
			or (ev is InputEventScreenTouch and (ev as InputEventScreenTouch).pressed)
		if tapped:
			_set_phone_list_open(true))

	var m := MarginContainer.new()
	m.add_theme_constant_override("margin_left", 14)
	m.add_theme_constant_override("margin_right", 14)
	m.mouse_filter = Control.MOUSE_FILTER_IGNORE
	bar.add_child(m)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	row.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	m.add_child(row)

	_phone_bar_banner = FactionBanner.new()
	_phone_bar_banner.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(_phone_bar_banner)
	_phone_bar_name = DccTheme.mono_label("", "text_bright", DccTheme.FS_SMALL, 0)
	_phone_bar_name.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_phone_bar_name.clip_text = true
	_phone_bar_name.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(_phone_bar_name)
	var chev := DccTheme.mono_label(DccIcons.SYMBOLS["expand"], "text_ghost", DccTheme.FS_SMALL, 0)
	chev.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(chev)
	return bar

## Exactly one of the two panes is on screen. The bar exists only while the
## list is folded, so it is never a second copy of a row already visible.
func _set_phone_list_open(open: bool) -> void:
	if not _phone:
		return
	_phone_list_pane.visible = open
	_phone_list_bar.visible = not open
	if not open:
		var d := _faction(_selected)
		_phone_bar_name.text = String(d.get("name", "?")) if not d.is_empty() else "—"
		if not d.is_empty():
			(_phone_bar_banner as FactionBanner).configure(_selected, _color_of(d), 22)

func open() -> void:
	## Cached once per open, not per faction row: the underlying pass is
	## O(cells) and rebuilds a biome raster and an ocean-distance field --
	## see `civ_faction_terrain_fits`' own Rust doc comment.
	_fits = bridge.civ_faction_terrain_fits()
	## Cached on the same schedule and for the same reason: one call rebuilds
	## the biome/lithology/resource passes `civ_faction_aggregates` needs, and
	## every faction's row comes out of that one answer.
	_military = bridge.civ_military_summary()
	_rebuild()
	## Reopens on the master, the way a phone list screen does -- picking up
	## mid-inspector on a faction chosen in a previous session would hide the
	## only control that says which faction this is.
	_set_phone_list_open(true)
	if not DccWidgets.phone_present(self, app):
		popup_centered()


## Set for the duration of a pane teardown. The inspector's name field commits
## on `focus_exited`, and removing a focused `Control` from the tree fires that
## signal **synchronously** -- so a rebuild was itself an "edit"
## (`GUI_GAP_REGISTER.md` FR-02). Measured before it was believed: with
## Aurelia's name field focused, clicking Veldmark in the list left the roster
## reading `1:Aurelia, 2:Aurelia` -- `_selected` is reassigned before
## `_rebuild_inspector()`, so the dying field's text was written to the faction
## the user had just switched TO.
var _rebuilding := false

func _clear(node: Control) -> void:
	var was := _rebuilding
	_rebuilding = true
	for c in node.get_children():
		node.remove_child(c)
		c.queue_free()
	_rebuilding = was

## Commit whatever the inspector's focused field holds **before** the caller
## changes `_selected`, so a half-typed rename lands on the faction it was
## typed for rather than being dropped by the guard above. Returns having
## released focus, which is what makes the `focus_exited` commit fire here,
## against the right id, instead of during the teardown.
func _commit_focused_field() -> void:
	var vp := _inspector_body.get_viewport()
	if vp == null:
		return
	var fo := vp.gui_get_focus_owner()
	if fo != null and _inspector_body.is_ancestor_of(fo):
		fo.release_focus()


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
		b.pressed.connect(func():
			## FR-02: flush the inspector's pending edit against the faction it
			## was typed for, before `_selected` moves. These list rows are
			## `FOCUS_NONE`, so without this the name field keeps focus right up
			## until `_rebuild_inspector()` frees it -- under the new id.
			_commit_focused_field()
			_selected = fid
			_rebuild_list()
			_rebuild_inspector()
			## Phone: the pick IS the navigation. Desktop leaves both panes up.
			_set_phone_list_open(false))
		row.add_child(b)
		_list_body.add_child(row)
	## `_set_field("name")` rebuilds this list on its own, so it needs its own
	## fit rather than relying on the inspector's.
	if _phone:
		app.phone_fit(_list_body, 1.0)


# -- Inspector (`_civPopulateFactionEditor`) --------------------------------

func _rebuild_inspector() -> void:
	_clear(_inspector_body)
	var d := _faction(_selected)
	if d.is_empty():
		DccWidgets.note(_inspector_body, "Select a faction.")
		return

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 8)
	_head_banner = FactionBanner.new()
	_head_banner.configure(_selected, _color_of(d), 48)
	head.add_child(_head_banner)
	var name_edit := LineEdit.new()
	name_edit.text = String(d.get("name", ""))
	name_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_edit.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	name_edit.text_submitted.connect(func(t: String): _set_field("name", t))
	## Guarded against its own teardown -- see `_rebuilding` (FR-02).
	name_edit.focus_exited.connect(func():
		if _rebuilding:
			return
		_set_field("name", name_edit.text))
	head.add_child(name_edit)
	_inspector_body.add_child(head)

	var sec := DccWidgets.section(_inspector_body, "Identity")
	_colour_row(sec, d)
	_vocab_choice(sec, "Government", bridge.civ_government_vocabulary(),
		String(d.get("government", "monarchy")), "government",
		"Live since 2026-08-25, and this is its first consumer in either codebase — the reference's own comment says no simulation reads it there. It sets how much of the surplus this faction's state can actually capture, which drives the standing army and half of the mobilization reach (Military, below).")
	_culture_choice(sec, String(d.get("culture", "common")))
	_vocab_choice(sec, "Religion", bridge.civ_religion_vocabulary(),
		String(d.get("religion", "none")), "religion")
	_ag_tech_choice(sec, String(d.get("ag_tech", "traditionalAgrarian")))

	_build_terrain_fit()
	_build_overview_block(d)
	_build_military_block()
	_build_settlement_sublist()
	_build_gaps()
	## Both panes are rebuilt from scratch here and in `_rebuild_list()`, so the
	## touch fit is re-applied over the window each time; idempotent, per
	## `DccShell.phone_fit`.
	if _phone:
		app.phone_fit(self, 1.0)


## The faction's identity colour — `GUI_GAP_REGISTER.md` **CV-21**, and v3's
## "CIVIL owns the colour, CARTO owns the paint" split at the CIVIL end.
##
## Registered as unbacked during the v3 pass, on the reading that
## "`FactionRoster` stores no colour field". It stored one already; what it
## had no way to do was let anyone *set* it, and nothing read it — the
## renderers went to `FACTION_RGB` by index. `civ_set_faction_color` writes
## the override and the three surfaces that draw a faction (territory wash,
## Political-control field, this banner) all read `CivData::faction_rgb`.
##
## `color_changed` rather than `popup_closed`: the map updates while the
## wheel is dragged, which is the whole point of a colour picker over a map.
func _colour_row(parent: Control, d: Dictionary) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = 24
	row.tooltip_text = "The colour this faction is drawn in: its territory wash on the map, the Political control analysis field, and its banner here. Unset, it takes the palette's own colour for this index."
	var l := DccTheme.mono_label("Colour", "text_dim", DccTheme.FS_SMALL, 0)
	l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
	l.clip_text = true
	row.add_child(l)

	var picker := ColorPickerButton.new()
	picker.custom_minimum_size = Vector2(64, 20)
	picker.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	picker.color = _color_of(d)
	picker.edit_alpha = false
	## Live, not on close: `roster_changed` repaints the territory wash, so
	## dragging the wheel drags the map's colour with it.
	picker.color_changed.connect(func(c: Color):
		if _rebuilding:
			return
		if bridge.civ_set_faction_color(_selected, c):
			_repaint_banners(c)
			roster_changed.emit())
	row.add_child(picker)

	var reset := DccWidgets.text_button(row, "Reset", func():
		if bridge.civ_clear_faction_color(_selected):
			roster_changed.emit()
			_rebuild())
	reset.disabled = not bool(d.get("color_custom", false))
	reset.tooltip_text = ("Back to the palette colour for faction %d." % _selected) if not reset.disabled \
		else "Already on the palette colour — nothing to reset."
	parent.add_child(row)


## The two banners on screen for the selected faction (the inspector head and,
## on phone, the bar) repainted in place. Rebuilding the whole inspector on
## every wheel movement would tear the picker down mid-drag.
func _repaint_banners(c: Color) -> void:
	if _head_banner != null and is_instance_valid(_head_banner):
		_head_banner.configure(_selected, c, 48)
	if _phone_bar_banner != null and is_instance_valid(_phone_bar_banner):
		(_phone_bar_banner as FactionBanner).configure(_selected, c, 22)


func _vocab_choice(parent: Control, label_text: String, vocab: Array, current: String,
		key: String, tip: String = "") -> void:
	if vocab.is_empty():
		return
	var keys: Array = []
	var labels: Array = []
	for e in vocab:
		var d: Dictionary = e
		keys.append(String(d.get("key", "")))
		labels.append(String(d.get("label", "?")))
	DccWidgets.choice(parent, label_text, labels, maxi(0, keys.find(current)),
		func(i: int): _set_field(key, keys[i]), tip)


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
		"Live since 2026-08-25: farmersPerUrbanite is the agricultural labour ratio the manpower model runs on (Military, below), so changing this moves this faction's standing army, field army, emergency levy and war duration. It is deliberately NOT the driver — it enters as one of five variables, and government, roads, water and the land itself move the answer as much.")
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


# -- Military (`GUI_GAP_REGISTER.md` CV-25 / `MILITARY_MANPOWER_SCOPE.md`) --
#
# This is the block the roster's own "Not built" note used to disclaim. The
# reference's Power breakdown had no reader anywhere in this port until CV-25
# landed, and its manpower half had no model in either codebase until this
# pass built one.
#
# **The two numbers here answer different questions and are labelled as such.**
# `power.military` is the reference's own 0-100 heuristic -- a comparison
# against the other factions on THIS map, explicitly derived and never
# presented as simulated. The four headcounts are absolute and have no
# reference at all. Neither is a rescaling of the other, and presenting one as
# the other would be the easiest wrong thing to do here.

func _military_row(faction_id: int) -> Dictionary:
	for r in (_military.get("factions", []) as Array):
		if int((r as Dictionary).get("faction", -1)) == faction_id:
			return r
	return {}

func _build_military_block() -> void:
	var row := _military_row(_selected)
	if row.is_empty():
		return
	var sec := DccWidgets.section(_inspector_body, "Military")
	DccWidgets.note(sec, "Power: %d/100 relative to the other factions   ·   %d of %d settlements fortified (%d stone, %d palisade, %d ditch)" % [
		int(round(float(row.get("military", 0.0)))), int(row.get("fortified_count", 0)),
		int(row.get("settlement_count", 0)), int(row.get("walled_stone", 0)),
		int(row.get("walled_palisade", 0)), int(row.get("walled_ditch", 0))])

	var m: Dictionary = row.get("manpower", {})
	if m.is_empty():
		return
	DccWidgets.note(sec, "Standing army %s (professional core %s)   ·   sustainable field army %s   ·   emergency levy %s" % [
		_thousands(int(round(float(m.get("standing_army", 0.0))))),
		_thousands(int(round(float(m.get("professional_core", 0.0))))),
		_thousands(int(round(float(m.get("field_army", 0.0))))),
		_thousands(int(round(float(m.get("emergency_mobilization", 0.0)))))])
	DccWidgets.note(sec, "Out of a total population of %s (%.0f%% in farming), of whom %s are of military age. A field army stays out %d days; a full levy %d." % [
		_thousands(int(round(float(m.get("total_population", 0.0))))),
		100.0 * float(m.get("agricultural_labour_ratio", 0.0)),
		_thousands(int(round(float(m.get("mobilization_pool", 0.0))))),
		int(round(float(m.get("field_duration_days", 0.0)))),
		int(round(float(m.get("emergency_duration_days", 0.0))))])
	## The era bands are shares of the CITIZEN / FREE population, not of the
	## total (owner ruling, 2026-08-25 -- the supplied specification's own
	## Republican Rome figure is quoted as "17-29 % of its citizen
	## population"). So that body is named on the line above the verdict
	## rather than being an invisible divisor: a reader has to be able to see
	## what the percentage is a percentage of, and how the government produced
	## it.
	DccWidgets.note(sec, "Citizen / free population %s — %.0f%% of the total, the share a %s confers. This is what the era bands below are measured against, not the whole population." % [
		_thousands(int(round(float(m.get("citizen_population", 0.0))))),
		100.0 * float(m.get("citizen_fraction", 0.0)),
		String(m.get("government", "?")).replace("_", " ")])
	DccWidgets.note(sec, "Reads as a %s (%s). Standing %.2f%% of citizens — %s that era's %.1f–%.1f%% band; mobilization %.1f%% — %s its %.0f–%.0f%%." % [
		String(m.get("era", "?")), String(m.get("era_constraint", "")),
		100.0 * float(m.get("standing_citizen_share", 0.0)),
		String(m.get("era_standing_verdict", "?")),
		100.0 * float(m.get("era_standing_lo", 0.0)), 100.0 * float(m.get("era_standing_hi", 0.0)),
		100.0 * float(m.get("emergency_citizen_share", 0.0)),
		String(m.get("era_mobilization_verdict", "?")),
		100.0 * float(m.get("era_mobilization_lo", 0.0)),
		100.0 * float(m.get("era_mobilization_hi", 0.0))])
	## Closes the modal on the way, the same shape `_build_settlement_sublist`
	## already uses -- leaving a roster window open over the category it just
	## navigated to would hide the thing it sent the reader to look at.
	var go := DccWidgets.action(sec, "Full breakdown → Civilization ▸ Military",
		func():
			hide()
			app.select_domain_category("civilization", "Military"))
	go.alignment = HORIZONTAL_ALIGNMENT_LEFT


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
		"The Power breakdown's four remaining axes (economic, political, cultural, religious) "
		+ "and the Economy block (food production and surplus, tax income, trade income, primary "
		+ "exports and imports, strategic resources, craft share). The military axis and the "
		+ "manpower model above are live; these read the same _civFactionAggregates pass and are "
		+ "a widget away rather than a model away -- see Civilization ▸ Economy and ▸ Trade for "
		+ "the parts that already have their own category.\n"
		+ "Diplomatic relations exists now (Civilization ▸ Relationships, CV-26): a derived, "
		+ "recomputed value per faction pair. What is still absent there is anything that ACTS "
		+ "-- treaties, vassalage, war declarations, change over time.")


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

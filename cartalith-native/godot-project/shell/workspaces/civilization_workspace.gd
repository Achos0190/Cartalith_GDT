extends Workspace
class_name CivilizationWorkspace

## CIVIL domain (§3): settlements, population, economy, politics, culture,
## and (§4.5.3) the Settlement and Territory tools.
##
## The engine backs four of the five browsing categories as *readable* data
## (`get_settlements`, `get_provinces`, `get_trade_balances`). Culture has no
## GDExtension binding at all -- `cartalith-civ` computes culture profiles
## internally (`STATUS.md`'s "generation rules and culture profiles"
## milestone) but nothing exports them.
##
## `civ_tools_bridge.rs` (`UNIFIED_TOOL_PLAN.md` milestone F) now backs
## placing a settlement and painting territory too -- `STRANDED_TOOLS.md`
## rows 10 and 12, closed by the TOOLS block below. POI is the one third of
## §4.5.3's own table that stays unbuilt: `_civDropPOI` has no Rust
## counterpart anywhere in this workspace (`civ_tools_bridge.rs`'s own doc
## comment), so there is no `civ_drop_poi` to arm a button against -- see
## `_build_tools()`'s own comment for where that button would have gone.
##
## The five browsing categories below (Settlements/Population/Economy/
## Politics/Culture) stay read-only -- clicking a settlement or faction row
## pins it into the right dock (`right_dock.gd`'s Settlement/Faction
## contexts) exactly like clicking it on the map does. Only the TOOLS block
## writes to `bridge`.

const KIND_ORDER := ["capital", "city", "town", "village", "hamlet"]
const KIND_PLURAL := {
	"capital": "capitals", "city": "cities", "town": "towns",
	"village": "villages", "hamlet": "hamlets",
}

## Which of our two tools (if either) is armed -- tracked locally for the
## same reason `infrastructure_workspace.gd`'s own `_active_infra_tool` is:
## `app.tool_armed` fires with the new id already written into `app.armed_
## tool`, so there is no other way to learn what is being armed away FROM.
var _active_civ_tool := ""

## -- Settlement tool state (§4.5.3's own options row). Persists across
## re-arms/rebuilds of the tool options bar so the row always reflects the
## shell's last choice, the same reasoning `cartography_workspace.gd`'s own
## `_icon_*` state vars follow. --
var _settlement_kind := "town"
var _settlement_faction := 1
## Cleared back to "" after every successful placement (`_settlement_click`)
## -- a name left in this field would otherwise get stamped onto every
## settlement dropped afterward, verbatim (`manual_settlement_name` only
## generates one when the given name is blank).
var _settlement_name := ""
var _settlement_snap_water := false

## -- Territory tool state. `_territory_radius`'s default is the reference's
## own `_civTerRadius` initial value (`TERRITORY_BRUSH_RADIUS`, `tools.rs`
## line 102), not an arbitrary number. --
var _territory_faction := 1
var _territory_radius := 5.0
var _territory_subtract := false


func _build() -> void:
	_build_tools()
	_build_settlements()
	_build_population()
	_build_economy()
	_build_politics()
	_build_culture()

# -- Tools (§4.5.3: Settlement, Territory) -------------------------------

## §4.5's TOOLS block: the three global tools (`GlobalTools.install`) plus
## this domain's own two, built first -- matching `infrastructure_workspace
## .gd`'s own ordering (§4.5: "every left dock opens with a TOOLS block").
##
## POI is not a third button here: §4.5.3's own table lists it ("Click drops
## a point of interest, `_civDropPOI`"), but `civ_tools_bridge.rs`'s own
## module doc says outright that POI "is not a ported concept" -- no Rust
## function anywhere in this workspace drops one, so there is nothing an
## armed POI tool could call. Arming a button with no engine behind it would
## be the fake control this port's own discipline (`DECISIONS.md`) exists to
## avoid, so it is omitted rather than built disabled or wired to a stub.
func _build_tools() -> void:
	DccWidgets.tools_block(self, app, app.tool_group, [
		{"id": "settlement", "glyph": "tool_settlement", "label": "Settlement (S)"},
		{"id": "territory", "glyph": "tool_territory", "label": "Territory (T)"},
	])
	app.register_tool_click_handler("settlement", func(gx, gy): _settlement_click(gx, gy))
	app.register_tool_drag_handler("territory", func(gx, gy): _territory_drag(gx, gy))
	app.tool_armed.connect(_on_civ_tool_armed)

## Reacts to ANY tool arming anywhere in the app (`app.tool_armed` is one
## shared signal across every domain), not just ours -- see `_active_civ_
## tool`'s own doc for why a local flag, not `app.armed_tool`, is what tells
## us whether *we* were the one just disarmed.
##
## Deliberately does NOT auto-commit an in-progress Territory stroke on
## switching away, unlike `infrastructure_workspace.gd`'s Way/Route (whose
## own `_on_infra_tool_armed` commits on switch). Territory's engine state
## (`civ_tools_bridge::CivTools`) is modelled on Paint's `PassBuffer`/
## `PaintLayer` draft, not Way/Route's click-chain (that module's own doc
## comment: "Milestone C's own precedent -- territory paint is PaintStamp/
## PaintLayer, unchanged"), and Paint's own precedent
## (`world_workspace.gd`'s `_on_tool_armed`) leaves its draft pending across
## a tool switch too, trusting the explicit Commit/Discard buttons rather
## than silently baking in whatever was mid-stroke when the user clicked
## away. A stray auto-commit of a half-finished territory claim would be a
## worse surprise than a draft that just waits.
func _on_civ_tool_armed(id: String) -> void:
	match id:
		"settlement":
			_active_civ_tool = "settlement"
			_tool_options_settlement()
		"territory":
			_active_civ_tool = "territory"
			_tool_options_territory()
		_:
			if _active_civ_tool != "":
				_active_civ_tool = ""
				## Matches `infrastructure_workspace.gd`'s own reasoning:
				## only reclaim the options bar for the plain "back to
				## Inspect" case -- Measure/Region set no options-bar content
				## of their own, so guessing at something to show while one
				## of those is armed would just be a different flavour of
				## stale.
				if id == "inspect":
					_tool_options_civ_idle()

## §10's brush ring, wired from `on_cursor_sampled` per the tool-arming
## substrate's own instructions (`app.gd`'s `_wire_selection` forwards every
## viewport cursor sample to any workspace that implements this method).
##
## Deliberately has no `else: hide` branch. `world_workspace.gd`'s own
## `on_cursor_sampled` (registered first -- `app.gd`'s `_register_
## workspaces` builds `["world", "civilization", ...]` in that order, and
## `_wire_selection`'s forwarding loop runs in the same order) already hides
## the brush ring for every tool that isn't sculpt/paint, Territory included
## -- so this only ever needs to say when to SHOW it; the "otherwise hide"
## half is already handled upstream. Adding a redundant `else` here would
## instead race it: this file's handler runs AFTER world's in that same
## loop, so an unconditional hide here would win the frame and blank the
## sculpt/paint ring the instant this file is present at all.
func on_cursor_sampled(gx: float, gy: float, valid: bool) -> void:
	if app.armed_tool == "territory":
		app.viewport.tool_overlay.set_brush_cursor(valid, gx, gy, _territory_radius)

func _tool_options_label(row: HBoxContainer, text: String) -> void:
	row.add_child(DccTheme.mono_label(text, "accent", DccTheme.FS_SMALL, 2, true))

## Reclaims the options bar once Settlement/Territory hands it back to plain
## Inspect. Same in-session-only caveat `infrastructure_workspace.gd`'s own
## twin carries: `app.gd`'s domain-switch default is stale the moment this
## file ships, but `app.gd` isn't ours to touch this pass.
func _tool_options_civ_idle() -> void:
	app.set_tool_options(func(row: HBoxContainer):
		_tool_options_label(row, "CIVIL · INSPECT")
		row.add_child(DccTheme.label("Settlement and Territory tools are armed from the TOOLS block above.", "text_ghost", DccTheme.FS_MICRO))
		row.add_child(DccTheme.spacer())
	)

## Shared by Settlement's and Territory's own options rows -- both need "pick
## a faction id", and `bridge.get_factions()` (`CIV_FACTION_COUNT` real ids,
## 1-based) is the one live source for what a valid faction actually is; a
## bare number spinbox would let either tool arm against an id the engine
## has never heard of. Renders a plain note instead of the dropdown before
## any world exists (`get_factions()` is empty then -- `lib.rs`'s own guard,
## no `civ` to read faction ids from yet).
func _faction_choice(row: HBoxContainer, current: int, on_change: Callable) -> void:
	var factions := bridge.get_factions()
	if factions.is_empty():
		row.add_child(DccTheme.label("no world generated -- factions unknown", "text_ghost", DccTheme.FS_MICRO))
		return
	var ids: Array = []
	var labels: Array = []
	for f in factions:
		var d: Dictionary = f
		var fid := int(d.get("id", 1))
		ids.append(fid)
		labels.append("%d · %s" % [fid, String(d.get("culture", "?")).capitalize()])
	DccWidgets.choice(row, "Faction", labels, maxi(0, ids.find(current)),
		func(i: int): on_change.call(ids[i]))

## `DccWidgets` has no bare-text-field row builder (`choice`/`slider`/
## `toggle`/`number`/`action` cover every other row shape this shell uses,
## per that file's own L4 disclosure-grammar comment) -- built by hand here
## rather than adding a sixth for this one call site, matching its private
## `_row()`'s own label+control layout (`DccWidgets.ROW_LABEL_W`) so it reads
## as one more row in the bar, not a visual outlier. `text_changed` updates
## the state var in place with no row rebuild, unlike a family/mode change
## elsewhere in this file -- rebuilding on every keystroke would steal focus
## out of the field mid-word.
func _settlement_name_field(row: HBoxContainer) -> void:
	var mini := HBoxContainer.new()
	mini.add_theme_constant_override("separation", 8)
	mini.custom_minimum_size.y = 24
	var l := DccTheme.mono_label("Name", "text_dim", DccTheme.FS_SMALL, 0)
	l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
	mini.add_child(l)
	var edit := LineEdit.new()
	edit.text = _settlement_name
	edit.placeholder_text = "(blank = generated)"
	edit.custom_minimum_size.x = 130
	edit.text_changed.connect(func(t: String): _settlement_name = t)
	mini.add_child(edit)
	row.add_child(mini)

## §4.5.3's Settlement options row: `CIVIL · SETTLEMENT` · class · faction ·
## name · snap to water. §4.5.3's own class list ("metropolis / city / town /
## village / hamlet") is one tier wider than the engine actually models --
## `civ_tools_bridge::kind_from_str` accepts exactly the five real tiers
## `SettlementKind` has and rejects "metropolis" like any other unknown
## string (that module's own doc comment). This dropdown offers only the
## five it can actually place, reusing `KIND_ORDER` (already this file's own
## capital-first tier order, `_build_settlements()` above).
##
## No pick-radius control: `civ_drop_settlement` (`lib.rs`) computes its own
## pick radius internally (`cartalith_civ::tools::civ_place_pick_radius(gw)`)
## and takes no radius argument at all, so a slider here would be decoration
## with nothing behind it -- the same kind of stale-spec-vs-engine gap this
## task's own brief names for Sculpt, treated the same way.
func _tool_options_settlement() -> void:
	app.set_tool_options(func(row: HBoxContainer):
		_tool_options_label(row, "CIVIL · SETTLEMENT")
		DccWidgets.choice(row, "Class", KIND_ORDER.map(func(k): return String(k).capitalize()),
			KIND_ORDER.find(_settlement_kind), func(i: int): _settlement_kind = KIND_ORDER[i])
		_faction_choice(row, _settlement_faction, func(fid: int): _settlement_faction = fid)
		_settlement_name_field(row)
		DccWidgets.toggle(row, "Snap to water", _settlement_snap_water,
			func(v: bool): _settlement_snap_water = v)
		row.add_child(DccTheme.spacer())
	)

## One click, one drop (`DCC_SHELL_SPEC.md` §4.5.3: "Click drops a place").
func _settlement_click(gx: float, gy: float) -> void:
	var idx := bridge.civ_drop_settlement(gx, gy, _settlement_kind, _settlement_faction, _settlement_name, _settlement_snap_water)
	if idx < 0:
		app.set_status("hint", "Settlement refused -- out of bounds, or water without Snap to water.", "accent")
		return
	_settlement_name = ""
	_refresh_civ_data()
	## §4.5.3's own right-dock column: "The new settlement's inspector, live,
	## focused on the name field." `right_dock.gd`'s Settlement context
	## (`_build_settlement`) renders every field, name included, as a plain
	## read-only Label, not a LineEdit -- there is no name field to focus,
	## and `right_dock.gd` is explicitly not this pass's to change. The
	## closest honest equivalent is selecting the new settlement the same
	## way clicking one on the map already does (`map_overlay.gd`'s own
	## `settlement_selected` -> `right_dock.gd`'s `on_settlement_selected`).
	var settlements := bridge.settlements()
	if idx < settlements.size():
		app.right_dock_ctrl.on_settlement_selected(settlements[idx], idx)
	## §4.5.6: "A tool that writes world data ... reports its staleness
	## consequence in the status bar the moment it commits." Not `bridge.
	## mark_dirty()` -- that flags the GENERATION PARAMETERS stale, prompting
	## a full regenerate, which would rebuild `civ.settlements` from scratch
	## and silently discard this manual drop. The real consequence is
	## narrower: provinces/trade balances/roads were computed before this
	## edit and won't reflect it until the next full regenerate, which is
	## what the hint below actually says.
	app.set_status("hint",
		"Settlement placed -- provinces/trade/roads were computed before this edit.", "text_ghost")
	if _active_civ_tool == "settlement":
		_tool_options_settlement()

## Repopulates `map_overlay.gd`'s settlement layer without the full
## `ViewportHost.refresh()` this same data normally goes through after a
## generate -- that call also runs `reset_view()` (recentres/rezooms the
## camera), which would be actively hostile here and worse for Territory
## (`_commit_territory` below calls this too, and a camera snap on every
## paint commit would be disorienting mid-session). `viewport.overlay` is a
## public field (`ViewportHost.overlay: Control`) for exactly this -- the
## same call `refresh()` itself makes for civ data, just without the camera
## reset wrapped around it.
func _refresh_civ_data() -> void:
	var g := bridge.grid_size()
	app.viewport.overlay.set_civ_data(bridge.settlements(), bridge.roads(),
		bridge.sea_routes(), g.x, g.y, bridge.border_inset_frac())

## §4.5.3's Territory options row: `CIVIL · TERRITORY` · faction · radius ·
## add/subtract · a live stats readout · Commit/Discard. No "respect
## coastlines" toggle: `civ_territory_paint_at` (`civ_tools_bridge::CivTools
## ::paint_at`) always pushes an ungated circular dab (`PaintStamp::
## ungated`) with no coastline mask behind it, so there is nothing to wire a
## toggle to -- the same disclosed-gap treatment this file already gives
## Settlement's missing pick-radius control above.
##
## The stats readout (`civ_faction_territory_stats`) reads the COMMITTED
## `civ.territory`, not the in-progress draft (`CivTools::paint_at` only
## touches `territory_draft`, baked in on `commit()` -- that module's own
## doc comment) -- so like Paint's own painted-cell counts (`world_workspace
## .gd`'s `_build_paint`), this is only "live" per commit, not per dab. Same
## reason there is no live preview texture here either: nothing in `civ_
## tools_bridge.rs`/`lib.rs` builds one for Territory (checked against every
## `civ_*` `#[func]` in `lib.rs` -- `build_sculpt_preview_texture`/`build_
## paint_preview_texture` exist, no Territory equivalent does).
func _tool_options_territory() -> void:
	app.set_tool_options(func(row: HBoxContainer):
		_tool_options_label(row, "CIVIL · TERRITORY")
		_faction_choice(row, _territory_faction, func(fid: int): _territory_faction = fid)
		DccWidgets.slider(row, "Radius", 1.0, 20.0, 1.0, _territory_radius, " c",
			func(v: float): _territory_radius = v)
		DccWidgets.choice(row, "Mode", ["Add", "Subtract (⇧)"], 1 if _territory_subtract else 0,
			func(i: int): _territory_subtract = (i == 1))
		var stats := bridge.civ_faction_territory_stats(_territory_faction)
		if not stats.is_empty():
			row.add_child(DccTheme.mono_label(
				"%d cells · %.0f km² · %d contested" % [
					int(stats.get("claimed_cells", 0)), float(stats.get("area_km2", 0.0)),
					int(stats.get("contested_cells", 0))],
				"text_dim", DccTheme.FS_MICRO))
		row.add_child(DccTheme.spacer())
		DccWidgets.action(row, "✓ Commit", _commit_territory, true)
		DccWidgets.action(row, "Discard", _discard_territory)
	)

## Paints on every drag sample (§4.5.3: "Drag paints the armed faction's
## claim"). ⇧ is a momentary modifier ORed with the Mode choice's own
## Subtract state, not a replacement for it -- matching Paint's own "⇧
## erases" precedent (`world_workspace.gd`'s `_paint_apply_dab`).
func _territory_drag(gx: float, gy: float) -> void:
	var subtract := _territory_subtract or Input.is_key_pressed(KEY_SHIFT)
	bridge.civ_territory_paint_at(gx, gy, _territory_faction, _territory_radius, subtract)

func _commit_territory() -> void:
	bridge.civ_territory_commit()
	## Same reasoning as `_refresh_civ_data()`'s own doc comment: the direct
	## field write `ViewportHost.refresh()` itself would make for the
	## territory texture, without that call's camera-resetting side effects.
	app.viewport.territory_view.texture = bridge.territory_texture()
	## §4.5.3's own right-dock column: "Faction inspector with live area,
	## claimed-cell count, and contested-cell warning." `right_dock.gd`'s
	## Faction context (`_build_faction`) predates `civ_faction_territory_
	## stats` and still says outright "no per-faction cell count or area
	## query exists" -- true when that sentence was written, no longer true,
	## but `right_dock.gd` is explicitly not this pass's to change. Pinning
	## the faction is still the closest honest equivalent (same "select what
	## was just edited" pattern `_settlement_click` above uses); the live
	## cells/area/contested numbers §4.5.3 actually wants live in THIS file's
	## own tool options row instead (above), which can read the real binding.
	app.right_dock_ctrl.show_faction(_territory_faction)
	app.set_status("hint",
		"Territory committed -- provinces/trade were computed before this edit.", "text_ghost")
	if _active_civ_tool == "territory":
		_tool_options_territory()

func _discard_territory() -> void:
	bridge.civ_territory_discard()
	if _active_civ_tool == "territory":
		_tool_options_territory()


# -- Settlements --------------------------------------------------------

func _build_settlements() -> void:
	var cat := DccWidgets.category(self, "Settlements", categories, true)
	var sec := DccWidgets.section(cat, "Roster")
	var settlements := bridge.settlements()
	if settlements.is_empty():
		DccWidgets.note(sec, "No settlements -- generate a world first (World ▸ Generation Pipeline).")
		return

	var counts := {}
	for s in settlements:
		var kind := String((s as Dictionary).get("kind", "?"))
		counts[kind] = int(counts.get(kind, 0)) + 1
	var parts: Array[String] = []
	for kind in KIND_ORDER:
		if counts.has(kind):
			parts.append("%d %s" % [counts[kind], kind if int(counts[kind]) == 1 else KIND_PLURAL[kind]])
	DccWidgets.note(sec, "%d settlements -- %s." % [settlements.size(), ", ".join(parts)])

	var by_pop := DccWidgets.group(sec, "Largest, by population")
	var ranked: Array = []
	for i in range(settlements.size()):
		ranked.append({"index": i, "data": settlements[i]})
	ranked.sort_custom(func(a, b): return int(a.data.population) > int(b.data.population))
	for i in range(mini(8, ranked.size())):
		_settlement_row(by_pop, ranked[i].data, ranked[i].index)

func _settlement_row(parent: Control, data: Dictionary, index: int) -> void:
	var text := "%s -- %s, pop %d" % [data.get("name", "?"), String(data.get("kind", "?")).capitalize(), int(data.get("population", 0))]
	var b := DccWidgets.action(parent, text, func(): app.right_dock_ctrl.on_settlement_selected(data, index))
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.tooltip_text = "Pin this settlement in the right dock (same as clicking it on the map)."

# -- Population -----------------------------------------------------------

func _build_population() -> void:
	var cat := DccWidgets.category(self, "Population", categories)
	var sec := DccWidgets.section(cat, "Totals")
	var settlements := bridge.settlements()
	if settlements.is_empty():
		DccWidgets.note(sec, "No settlements -- generate a world first.")
		return

	var total := 0
	var largest_name := ""
	var largest_pop := -1
	for s in settlements:
		var d: Dictionary = s
		var pop := int(d.get("population", 0))
		total += pop
		if pop > largest_pop:
			largest_pop = pop
			largest_name = String(d.get("name", "?"))
	DccWidgets.note(sec, "Total population %d across %d settlements. Largest: %s (%d)." % [
		total, settlements.size(), largest_name, largest_pop])
	DccWidgets.note(sec,
		"Per-settlement population is real (get_settlements()'s own field). Faction- or " +
		"province-level population aggregation has no binding -- see Politics below for " +
		"what get_provinces() does carry.")

# -- Economy ----------------------------------------------------------------

func _build_economy() -> void:
	var cat := DccWidgets.category(self, "Economy", categories)
	var sec := DccWidgets.section(cat, "Trade balance")
	var settlements := bridge.settlements()
	var balances := bridge.trade_balances()
	if balances.is_empty():
		DccWidgets.note(sec, "No trade balances -- generate a world first.")
		return

	var exports := {}
	var imports := {}
	var trading := 0
	for i in range(balances.size()):
		var t: Dictionary = balances[i]
		var ex: PackedStringArray = t.get("exports", PackedStringArray())
		var im: PackedStringArray = t.get("imports", PackedStringArray())
		if ex.size() > 0 or im.size() > 0:
			trading += 1
		for r in ex:
			exports[r] = int(exports.get(r, 0)) + 1
		for r in im:
			imports[r] = int(imports.get(r, 0)) + 1
	DccWidgets.note(sec, "%d of %d settlements carry a trade relationship (surplus or deficit)." % [
		trading, settlements.size()])
	DccWidgets.note(sec, "Most-exported: %s. Most-imported: %s." % [
		_top_key(exports), _top_key(imports)])
	DccWidgets.note(sec,
		"This is the hinterland term only (civ_resource_trade_balance) -- the full " +
		"faction-level aggregation (population, tax, the five-axis power heuristic) is " +
		"real future scope per ECONOMY_SCOPE.md, not yet computed.")

func _top_key(counts: Dictionary) -> String:
	if counts.is_empty():
		return "none"
	var best_key := ""
	var best_n := -1
	for k in counts:
		if int(counts[k]) > best_n:
			best_n = int(counts[k])
			best_key = String(k)
	return "%s (%d settlements)" % [best_key, best_n]

# -- Politics -----------------------------------------------------------

func _build_politics() -> void:
	var cat := DccWidgets.category(self, "Politics", categories)
	var sec := DccWidgets.section(cat, "Factions")
	var provinces := bridge.provinces()
	var settlements := bridge.settlements()
	if provinces.is_empty():
		DccWidgets.note(sec, "No provinces -- generate a world first.")
	else:
		var by_faction := {}
		for p in provinces:
			var d: Dictionary = p
			var f := int(d.get("faction", 0))
			if not by_faction.has(f):
				by_faction[f] = []
			by_faction[f].append(d)
		var factions: Array = by_faction.keys()
		factions.sort()
		var roster := DccWidgets.group(sec, "Roster, by province count")
		for f in factions:
			var provs: Array = by_faction[f]
			var cap_name := "—"
			if not provs.is_empty():
				var cap_idx := int((provs[0] as Dictionary).get("capital_settlement_index", -1))
				if cap_idx >= 0 and cap_idx < settlements.size():
					cap_name = String((settlements[cap_idx] as Dictionary).get("name", "—"))
			var text := "Faction %d -- %d provinces, capital %s" % [f, provs.size(), cap_name]
			var b := DccWidgets.action(roster, text, func(): app.right_dock_ctrl.show_faction(f))
			b.alignment = HORIZONTAL_ALIGNMENT_LEFT
			b.tooltip_text = "Open this faction in the right dock."

	DccWidgets.note(sec,
		"Territory paint and settlement placement (STRANDED_TOOLS.md rows 10, 12) are wired " +
		"now -- see the TOOLS block at the top of this dock (§4.5.3's Settlement and " +
		"Territory tools).")

# -- Culture ----------------------------------------------------------------

func _build_culture() -> void:
	var cat := DccWidgets.category(self, "Culture", categories)
	var sec := DccWidgets.section(cat, "Profiles")
	DccWidgets.note(sec,
		"cartalith-civ generates culture profiles internally (generation rules + culture " +
		"profiles milestone, STATUS.md), but no GDExtension method exports them -- " +
		"get_settlements()/get_provinces() carry no culture field, and there is no " +
		"get_cultures() to read one from. Nothing here can be honest until that binding " +
		"lands.")

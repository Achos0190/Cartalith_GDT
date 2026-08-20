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
##
## **Domain merge (2026-08-20, owner instruction: "Infra can be dropped as a
## name and can be absorbed by civil").** This dock now also carries the
## former INFRA domain's five subjects and its Way/Route tools, via `_infra`
## below -- a real `InfrastructureWorkspace` instance, unmodified in what it
## does, appended as a nested `VBoxContainer` after this file's own six
## categories. `_build_tools()` draws ONE combined TOOLS block (Settlement ·
## Territory · Way · Route) rather than two stacked ones; `_infra`'s own
## `_build_tools()` (called from its own `setup()`, `_nested = true`) skips
## drawing its half of that row but still registers the Way/Route click/drag/
## escape handlers, since those don't care which file drew the button. See
## `InfrastructureWorkspace`'s own class doc for the mechanism, and
## `DCC_SHELL_SPEC.md`'s correction notice for the disclosure.

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

## -- Timeline state (`TIMELINE_SCOPE.md` milestone 6). See the Timeline
## section's own header comment, below `_build_culture()`, for why this is a
## sixth `DccWidgets.category()` here rather than a new `right_dock.gd` CTX_*
## context. --
var _tl_body: VBoxContainer
var _tl_add_year := 100                 ## Reference default (`#civTlYear` value="100").
var _tl_playing := false
var _tl_play_timer: Timer
var _tl_sim_mode := "collapse"          ## "collapse" | "recovery"
var _tl_sim_character := "mixed"        ## "mixed" | "trade" | "disease" | "conflict"
var _tl_sim_severity := 0.5             ## fraction [0,1] -- reference slider/100
var _tl_sim_rate := 0.01                ## fraction/yr -- reference slider(tenths-%)/1000
var _tl_sim_start_year := 0
var _tl_sim_duration := 100
var _tl_sim_step_years := 10
var _tl_filter_exist_only := false
var _tl_filter_ghost := false
var _tl_filter_highlight := false
var _tl_sim_out := ""

## The former INFRA domain, nested into this dock -- see this file's own
## class doc and `InfrastructureWorkspace`'s own class doc for the mechanism.
var _infra: InfrastructureWorkspace


func _build() -> void:
	_infra = InfrastructureWorkspace.new()
	_infra._nested = true

	_build_tools()
	_build_settlements()
	_build_population()
	_build_economy()
	_build_politics()
	_build_culture()
	_build_timeline()

	## Appended last, after CIVIL's own six categories -- `_infra.setup()`
	## calls its own `_build()`, which adds its five categories (Roads/
	## Rivers/Ports/Trade/Logistics) as children of `_infra` itself, and
	## registers the Way/Route handlers `_build_tools()` above already drew
	## buttons for. One rule marks the seam so the merge reads as two grouped
	## subjects, not one undifferentiated list.
	add_child(DccTheme.rule())
	add_child(_infra)
	_infra.setup(app, bridge)

# -- Tools (§4.5.3: Settlement, Territory -- and, since the 2026-08-20 merge,
# §4.5.4: Way, Route) -----------------------------------------------------

## §4.5's TOOLS block: the three global tools (`GlobalTools.install`) plus
## this domain's own four, built first (§4.5: "every left dock opens with a
## TOOLS block"). One combined row rather than two stacked ones -- Settlement
## and Territory are this file's own; Way and Route belong to `_infra`
## (`InfrastructureWorkspace`, composed in via `_build()` above since the
## 2026-08-20 domain merge) and are registered by its own `_build_tools()`
## when `_infra.setup()` runs, `_nested = true` so it draws no second row.
##
## POI is not a fifth button here: §4.5.3's own table lists it ("Click drops
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
		{"id": "way", "glyph": "tool_way", "label": "Way (W)"},
		{"id": "route", "glyph": "tool_route", "label": "Route (⇧R)"},
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
		_build_settlement_gaps(cat)
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

	_build_settlement_gaps(cat)


## The reference's own settlement-population operations, which this shell has
## no equivalent of because the split is different, not because they were
## forgotten: `generate()` populates the world as part of the one-shot chain
## (`compute_civilisation`), so there is no separate "populate now" step to
## press and nothing that clears just the civ layer without re-running the
## whole pipeline. Said out loud so a reader of this dock can tell that apart
## from an unfinished panel.
func _build_settlement_gaps(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Not built")
	var pop := DccWidgets.action(sec, "Auto-populate world", func(): pass)
	pop.disabled = true
	pop.tooltip_text = "The reference's #civAutoPopulateBtn, plus its capitals / towns / hamlets count sliders. In this port settlement placement is not a separate pass: compute_civilisation runs inside generate() and there is no civ_populate #[func] to call on its own, nor any parameter for the three counts (params.rs has 58 entries, none of them civ). Re-generate from World ▸ Generation pipeline to re-place everything."
	var clear := DccWidgets.action(sec, "Clear places & routes", func(): pass)
	clear.disabled = true
	clear.tooltip_text = "The reference's #civClearPlacesBtn. Same shape: no civ_clear_places #[func] exists, and CivData is rebuilt wholesale by generate() rather than mutated in place, so there is no partial teardown to expose. Individual manual drops can still be undone by re-generating."
	DccWidgets.note(sec,
		"The placement model's own dials are equally internal: biome carrying-capacity "
		+ "and the imperial-seat (metropolis) tier are computed inside cartalith-civ with "
		+ "no parameters, and urban morphology layouts are a separate unported subsystem "
		+ "(URBAN_MORPHOLOGY_SCOPE.md, Phase 5, in progress). Village seeding is the one "
		+ "of the four that IS exposed -- as a toggle in File ▸ New world.")

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

	var gaps := DccWidgets.section(cat, "Not built")
	var recalc := DccWidgets.action(gaps, "Recalculate territories", func(): pass)
	recalc.disabled = true
	recalc.tooltip_text = "The reference's territory recompute. assign_territory() runs inside compute_civilisation as part of generate(); no #[func] re-runs it against edited settlements, so a manual drop does not redraw the claim map until the next full re-generate (which is what the Settlement tool's own status hint already says). Painting a claim by hand is the wired alternative -- the Territory tool above."
	var clear_ter := DccWidgets.action(gaps, "Clear territory", func(): pass)
	clear_ter.disabled = true
	clear_ter.tooltip_text = "Same: CivData::territory is rebuilt wholesale by generate() and there is no civ_clear_territory #[func]. The Territory tool's own Discard reverts an uncommitted draft only, not the committed claim map."
	var gen_prov := DccWidgets.action(gaps, "Generate provinces", func(): pass)
	gen_prov.disabled = true
	gen_prov.tooltip_text = "The reference's province generator. Provinces are produced inside generate() and only read out (get_provinces()); no #[func] regenerates them. Their map tint IS live -- Cartography ▸ Layers ▸ Political — provinces."
	var add_fac := DccWidgets.action(gaps, "Add / remove faction", func(): pass)
	add_fac.disabled = true
	add_fac.tooltip_text = "GUI_GAP_REGISTER.md CV-07. CIV_FACTION_COUNT is a compile-time constant in cartalith-civ and factions have no persistent identity across a re-generate, so there is no roster to add to or remove from -- get_factions() enumerates a fixed set, it does not own one."
	DccWidgets.note(gaps,
		"Diplomatic relations (the design's own per-faction sub-list) is new work with no " +
		"reference behaviour behind it either -- cartalith-civ models no inter-faction " +
		"relation of any kind.")

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

# -- Timeline (`TIMELINE_SCOPE.md` milestone 6) --------------------------------
#
# Built as a sixth L2 category in THIS workspace's left dock, alongside
# Settlements/Population/Economy/Politics/Culture above, rather than a new
# `right_dock.gd` CTX_* context. `right_dock.gd`'s own CTX_SCULPT/CTX_JOURNEY
# (the precedent this milestone's own brief pointed at) are both driven by an
# actual map TOOL arming (`app.tool_armed`) tied to viewport interaction --
# Sculpt shows the live stamp stack while the Sculpt tool is armed or a
# stroke just ended; Journey swaps the whole INFRA region while the JOURNEY
# tool is armed. Timeline has no map click of its own: add year / goto year /
# run simulation are all pure state edits with no tool to arm, exactly like
# THIS FILE's own Settlements/Population/Politics categories already are
# (click a row, pin something, done). Given that, `DccWidgets.category()` --
# this file's own established vocabulary, used five times already above -- is
# the correctly-scoped precedent, not `right_dock.gd`'s tool-armed dispatch.
#
# Also deliberately NOT wired into `dcc_shell.gd`'s own `timeline_bar`/
# `timeline_row` (`_build_timeline()` there -- the empty bottom strip
# `DCC_CONTROL_INDEX.md` §10 reserves, shown for civilization (was
# civilization/infrastructure before the 2026-08-20 domain merge -- one
# surviving id already covers both), `app.gd`'s `_on_workspace_changed`).
# `TIMELINE_SCOPE.md` §4's own
# words: "if you're unsure whether a given shell region is the discrete
# scrub mechanism vs the continuous six-toggle feature, stop and default to
# building your own dedicated panel rather than risking the wrong one." That
# bar is one fixed-height HBox row -- no room for a year-pill list, an add-
# year field, three filter checkboxes AND the whole collapse-sim form -- and
# §10 never disambiguates whether ITS OWN scrub track is meant for this
# discrete `civTimeline` or the still-open six-toggle continuous simulation.
# Left untouched rather than risk building into the wrong one; this category
# is the "dedicated panel" that note calls for instead.
#
# Real, disclosed gap (`_build_timeline_filters` below carries the same note
# in-product): `get_settlements()` (`lib.rs`) carries no `tid` field even
# though `NamedSettlement` gained one at the Rust level (`TIMELINE_SCOPE.md`
# milestone 1, `timeline_bridge.rs`'s own top-of-file doc comment says so
# outright). So the three filter checkboxes below drive `civ_year_diff()` for
# real -- the present/removed/added counts they show are live engine output
# -- but nothing on the Godot side can tell which drawn settlement PIN any of
# those tids refers to, so exist-only/ghost/highlight cannot filter or style
# individual pins on the map yet. That is real Rust-side work
# (`get_settlements()`'s own `#[func]`), out of scope for this GDScript-only
# milestone -- disclosed here and in `CHANGELOG.md`, not silently faked.

func _build_timeline() -> void:
	var cat := DccWidgets.category(self, "Timeline", categories)
	_tl_body = cat
	bridge.generation_finished.connect(func(ok: bool): if ok: _tl_on_world_changed())
	bridge.world_loaded.connect(func(): _tl_on_world_changed())
	_rebuild_timeline()

## A fresh generate/loaded save invalidates any in-flight playback and the
## last simulation's own readout -- same reasoning `right_dock.gd`'s
## `_rebuild()` already applies on the same two signals.
func _tl_on_world_changed() -> void:
	_tl_playing = false
	if _tl_play_timer != null:
		_tl_play_timer.stop()
	_tl_sim_out = ""
	_rebuild_timeline()

## Tears down and rebuilds just this category's body -- the same "whole-
## section rebuild on every action" discipline `right_dock.gd`'s own
## `_rebuild()`/`show_sculpt_stack()` pair already establishes, scoped to
## `_tl_body` rather than the whole workspace so Settlements/Population/etc.
## above are untouched.
func _rebuild_timeline() -> void:
	if _tl_body == null:
		return
	for c in _tl_body.get_children():
		_tl_body.remove_child(c)
		c.queue_free()
	if not bridge.has_world:
		DccWidgets.note(_tl_body, "Generate a world first.")
		return
	_build_timeline_years(_tl_body)
	_build_timeline_scrub(_tl_body)
	_build_timeline_playback(_tl_body)
	_build_timeline_filters(_tl_body)
	_build_timeline_sim(_tl_body)

## `_civFormatYear` (reference line 20644), ported verbatim: negative years
## are BC.
static func _tl_format_year(year: int) -> String:
	return ("%d BC" % -year) if year < 0 else ("%d AD" % year)

## `get_civ_timeline_years()` is already ascending (`lib.rs`'s own doc
## comment) -- re-sorted defensively rather than trusted blindly, since
## nothing here is on a hot path.
func _tl_years() -> Array:
	var out: Array = Array(bridge.get_civ_timeline_years())
	out.sort()
	return out

func _tl_nearest_year(years: Array, target: int) -> int:
	var nearest: int = years[0]
	for y in years:
		if absi(int(y) - target) < absi(nearest - target):
			nearest = int(y)
	return nearest

## Same reasoning as `_refresh_civ_data()`/`_commit_territory()` above: every
## call that moves the timeline cursor (`civGotoYear`) reloads `territory`
## engine-side but does not itself touch the rendered texture -- this is the
## direct field write that does, without `ViewportHost.refresh()`'s camera-
## reset side effect.
func _tl_refresh_territory_view() -> void:
	if app != null and app.viewport != null and app.viewport.territory_view != null:
		app.viewport.territory_view.texture = bridge.territory_texture()

func _tl_goto_year(year: int) -> void:
	bridge.civ_goto_year(year)
	_tl_refresh_territory_view()

func _tl_add_year_action() -> void:
	bridge.civ_add_year(_tl_add_year)
	_tl_refresh_territory_view()
	_rebuild_timeline()

func _tl_remove_year_action(year: int) -> void:
	bridge.civ_remove_year(year)
	_tl_refresh_territory_view()
	_rebuild_timeline()

# -- Years pill row + Add year (Cluster A: civAddYear/civRemoveYear/civGotoYear) --

func _build_timeline_years(body: Control) -> void:
	var sec := DccWidgets.section(body, "Years")
	var add_row := HBoxContainer.new()
	add_row.add_theme_constant_override("separation", 6)
	var yi := SpinBox.new()
	yi.min_value = -9999
	yi.max_value = 9999
	yi.step = 1
	yi.value = _tl_add_year
	yi.custom_minimum_size.x = 90
	yi.value_changed.connect(func(v: float): _tl_add_year = int(v))
	add_row.add_child(yi)
	DccWidgets.action(add_row, "Add year", _tl_add_year_action)
	sec.add_child(add_row)

	var years := _tl_years()
	if years.is_empty():
		DccWidgets.note(sec, "No years added yet.")
	else:
		var flow := HFlowContainer.new()
		flow.add_theme_constant_override("h_separation", 4)
		flow.add_theme_constant_override("v_separation", 4)
		var cur := bridge.get_civ_year()
		for y in years:
			flow.add_child(_tl_year_pill(int(y), int(y) == cur))
		sec.add_child(flow)
		DccWidgets.note(sec, "Active year: %s" % _tl_format_year(cur))
	DccWidgets.note(sec,
		"Each year stores a snapshot of the territory paint plus which settlements/roads " +
		"existed then. Negative years are BC. Tap a pill to jump to that era, or %s to remove it." %
			DccIcons.SYMBOLS["cross"])

func _tl_year_pill(year: int, active: bool) -> Control:
	var pill := HBoxContainer.new()
	pill.add_theme_constant_override("separation", 1)
	var go := Button.new()
	go.text = _tl_format_year(year)
	go.flat = true
	go.focus_mode = Control.FOCUS_NONE
	go.custom_minimum_size.y = 22
	go.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	go.add_theme_font_override("font", DccTheme.mono(1))
	go.add_theme_color_override("font_color", DccTheme.c("bg") if active else DccTheme.c("text"))
	go.add_theme_color_override("font_hover_color", DccTheme.c("bg") if active else DccTheme.c("text_bright"))
	go.add_theme_stylebox_override("normal", DccTheme.flat(DccTheme.c("accent") if active else DccTheme.c("sunken"), 2))
	go.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("accent").lightened(0.1) if active else DccTheme.c("raised"), 2))
	go.tooltip_text = "Jump to %s (civ_goto_year)." % _tl_format_year(year)
	go.pressed.connect(func(): _tl_goto_year(year); _rebuild_timeline())
	pill.add_child(go)
	var rm := Button.new()
	rm.text = DccIcons.SYMBOLS["cross"]
	rm.flat = true
	rm.focus_mode = Control.FOCUS_NONE
	rm.custom_minimum_size = Vector2(18, 22)
	rm.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	rm.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
	rm.add_theme_color_override("font_hover_color", DccTheme.c("accent"))
	rm.tooltip_text = "Remove %s (civ_remove_year)." % _tl_format_year(year)
	rm.pressed.connect(func(): _tl_remove_year_action(year))
	pill.add_child(rm)
	return pill

# -- Scrub track (Cluster C: _civWireYearSlider, v0.91 real time-scale) --------

func _build_timeline_scrub(body: Control) -> void:
	var years := _tl_years()
	if years.size() < 2:
		return   ## Gated exactly like the reference's #explTimelineSliderRow (>1 recorded year).
	var sec := DccWidgets.section(body, "Scrub")
	var lo: int = years[0]
	var hi: int = years[years.size() - 1]
	var cur := bridge.get_civ_year()
	var caps := HBoxContainer.new()
	caps.add_child(DccTheme.mono_label(_tl_format_year(lo), "text_ghost", DccTheme.FS_TINY))
	caps.add_child(DccTheme.spacer())
	caps.add_child(DccTheme.mono_label(_tl_format_year(hi), "text_ghost", DccTheme.FS_TINY))
	sec.add_child(caps)
	DccWidgets.slider(sec, "Year", float(lo), float(hi), 1.0, float(cur), "",
		func(v: float): _tl_goto_year(_tl_nearest_year(years, int(round(v)))),
		"Real time-scale -- min/max/value are the actual recorded years, not a snapshot-count " +
			"index (reference v0.91). Dragging snaps to the nearest recorded year.",
		func(): _rebuild_timeline())
	DccWidgets.note(sec, "Active year: %s" % _tl_format_year(cur))

# -- Playback transport (Cluster C: _civTlStartPlay/_civTlStopPlay, 1200ms) ----

func _build_timeline_playback(body: Control) -> void:
	var years := _tl_years()
	if years.size() < 2:
		return   ## Same gate as the scrub row -- reference shares one markup section for both.
	var sec := DccWidgets.section(body, "Playback")
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	var label_text := ("%s Stop" % DccIcons.SYMBOLS["pause"]) if _tl_playing else ("%s Animate" % DccIcons.SYMBOLS["play"])
	DccWidgets.action(row, label_text, func(): _tl_stop_play() if _tl_playing else _tl_start_play())
	DccWidgets.action(row, "Step", _tl_step)
	sec.add_child(row)

func _tl_ensure_timer() -> void:
	if _tl_play_timer == null:
		_tl_play_timer = Timer.new()
		_tl_play_timer.wait_time = 1.2   ## Reference's own 1200ms interval, verbatim.
		_tl_play_timer.one_shot = false
		_tl_play_timer.timeout.connect(_tl_on_play_tick)
		add_child(_tl_play_timer)

func _tl_start_play() -> void:
	var years := _tl_years()
	if years.size() < 2:
		return
	_tl_ensure_timer()
	_tl_playing = true
	_tl_play_timer.start()
	_rebuild_timeline()

func _tl_stop_play() -> void:
	_tl_playing = false
	if _tl_play_timer != null:
		_tl_play_timer.stop()
	_rebuild_timeline()

func _tl_on_play_tick() -> void:
	var years := _tl_years()
	if years.size() < 2:
		_tl_stop_play()
		return
	var cur := bridge.get_civ_year()
	var idx := years.find(cur)
	var next: int = (idx + 1) if idx >= 0 else 0
	if next >= years.size():
		_tl_stop_play()
		return
	_tl_goto_year(int(years[next]))
	if next == years.size() - 1:
		_tl_stop_play()
	else:
		_rebuild_timeline()

func _tl_step() -> void:
	var years := _tl_years()
	if years.size() < 2:
		return
	var cur := bridge.get_civ_year()
	var idx := years.find(cur)
	var next: int = (idx + 1) if idx >= 0 else 0
	if next >= years.size():
		return
	_tl_goto_year(int(years[next]))
	_rebuild_timeline()

# -- Filters (Cluster C: explTlExistOnly/Ghost/Highlight, _civYearDiff) --------

func _build_timeline_filters(body: Control) -> void:
	var sec := DccWidgets.section(body, "Filters")
	DccWidgets.toggle(sec, "Exist only", _tl_filter_exist_only,
		func(v: bool): _tl_filter_exist_only = v,
		"Reference: hide anything not present in the selected year (civ_year_diff().present).")
	DccWidgets.toggle(sec, "Ghost removed", _tl_filter_ghost,
		func(v: bool): _tl_filter_ghost = v,
		"Reference: fade objects removed since the previous recorded year (civ_year_diff().removed).")
	DccWidgets.toggle(sec, "Highlight new", _tl_filter_highlight,
		func(v: bool): _tl_filter_highlight = v,
		"Reference: halo objects added since the previous recorded year (civ_year_diff().added).")
	var years := _tl_years()
	if not years.is_empty():
		var diff: Dictionary = bridge.civ_year_diff(bridge.get_civ_year())
		var present: int = (diff.get("present", PackedInt64Array()) as PackedInt64Array).size()
		var removed: int = (diff.get("removed", PackedInt64Array()) as PackedInt64Array).size()
		var added: int = (diff.get("added", PackedInt64Array()) as PackedInt64Array).size()
		DccWidgets.note(sec,
			"%d present, %d removed, %d added vs. the previous recorded year -- real, live civ_year_diff() output." %
				[present, removed, added])
	DccWidgets.note(sec,
		"Not yet wired to the map: get_settlements() carries no tid even though " +
		"NamedSettlement gained one in Rust (TIMELINE_SCOPE.md milestone 1) -- nothing on " +
		"this side can tell which drawn pin any of civ_year_diff()'s tids refers to, so " +
		"these checkboxes read real state above but cannot filter/ghost/highlight " +
		"individual settlement pins on the map yet. A real, disclosed Rust-side gap " +
		"(get_settlements()'s own #[func], lib.rs), not faked here.")

# -- Collapse / recovery simulator form (Cluster B/impure wiring) -------------

func _build_timeline_sim(body: Control) -> void:
	var grp := DccWidgets.group(body, "Simulate collapse / recovery", false)
	DccWidgets.choice(grp, "Mode", ["Collapse (decline + migration)", "Recovery (regrowth)"],
		0 if _tl_sim_mode == "collapse" else 1,
		func(i: int): _tl_sim_mode = ("collapse" if i == 0 else "recovery"); _rebuild_timeline())
	if _tl_sim_mode == "collapse":
		var chars := ["mixed", "trade", "disease", "conflict"]
		DccWidgets.choice(grp, "Character",
			["Mixed (default)", "Trade -- hubs fall hardest", "Disease -- dense/connected fall first",
				"Conflict -- undefended fall first"],
			maxi(0, chars.find(_tl_sim_character)),
			func(i: int): _tl_sim_character = chars[i])
		DccWidgets.slider(grp, "Severity", 0.0, 100.0, 1.0, _tl_sim_severity * 100.0, "%",
			func(v: float): _tl_sim_severity = v / 100.0)
	else:
		DccWidgets.slider(grp, "Regrowth rate", 0.1, 3.0, 0.1, _tl_sim_rate * 100.0, "%/yr",
			func(v: float): _tl_sim_rate = v / 100.0)
	DccWidgets.number(grp, "Start year", -9999.0, 9999.0, 1.0, float(_tl_sim_start_year),
		func(v: float): _tl_sim_start_year = int(v))
	DccWidgets.number(grp, "Duration (yr)", 1.0, 1000000.0, 1.0, float(_tl_sim_duration),
		func(v: float): _tl_sim_duration = int(v))
	DccWidgets.number(grp, "Step years", 1.0, 1000000.0, 1.0, float(_tl_sim_step_years),
		func(v: float): _tl_sim_step_years = int(v))
	DccWidgets.action(grp, "Simulate", func(): _tl_run_simulation(false), true)
	DccWidgets.note(grp,
		"Runs a year-by-year simulation from the CURRENT settlements and writes one " +
		"timeline entry per step. Collapse: each settlement's stress (trade/density/ " +
		"undefended exposure, weighted by Character) drives mortality and gravity-model " +
		"out-migration each step; a nucleus below its tier's floor demotes or is " +
		"abandoned. Recovery: logistic regrowth toward each settlement's catchment " +
		"ceiling. See docs/research/collapse-timeline-dynamics.md.")
	if _tl_sim_out != "":
		DccWidgets.note(grp, _tl_sim_out)

func _tl_run_simulation(confirm_overwrite: bool) -> void:
	var request := {
		"mode": _tl_sim_mode,
		"character": _tl_sim_character,
		"severity": _tl_sim_severity,
		"rate": _tl_sim_rate,
		"start_year": _tl_sim_start_year,
		"duration": _tl_sim_duration,
		"step_years": _tl_sim_step_years,
		"confirm_overwrite": confirm_overwrite,
	}
	var result: Dictionary = bridge.civ_run_collapse_simulation(request)
	if not bool(result.get("ok", false)):
		if bool(result.get("needs_confirm", false)):
			_tl_show_confirm(result)
			return
		_tl_sim_out = String(result.get("error", "Simulation failed."))
		_rebuild_timeline()
		return
	_tl_sim_out = _tl_format_sim_result(result)
	_tl_refresh_territory_view()
	_rebuild_timeline()

## `_civRunCollapseSimulation`'s own blocking `confirm()` (reference line
## 24911), reimplemented as a real Yes/No dialog -- this shell's own
## precedent is `AcceptDialog` built by hand and `add_child`ed onto `app`
## (`app.gd`'s `open_storage_locations`/`open_credits`, `cartography_
## workspace.gd`'s `_prompt_label_name`), none of which needed a Cancel path.
## `ConfirmationDialog` (a built-in `AcceptDialog` subclass adding exactly
## that Cancel/close-as-No button) is the closest match to that convention
## for a real two-way choice.
func _tl_show_confirm(result: Dictionary) -> void:
	var years: PackedInt64Array = result.get("clobber_years", PackedInt64Array())
	var parts: Array[String] = []
	for y in years:
		parts.append(_tl_format_year(int(y)))
	var dlg := ConfirmationDialog.new()
	dlg.title = "Overwrite recorded years?"
	dlg.dialog_text = "Simulation will overwrite %d existing timeline year%s (%s).\n\nContinue?" % [
		years.size(), "" if years.size() == 1 else "s", ", ".join(parts)]
	dlg.get_ok_button().text = "Overwrite"
	dlg.confirmed.connect(func(): _tl_run_simulation(true); dlg.queue_free())
	dlg.canceled.connect(dlg.queue_free)
	app.add_child(dlg)
	dlg.popup_centered()

## Reference's own `civSimOut` innerHTML (lines 24940-24949), ported field-
## for-field -- `fmt()`'s k/M abbreviation is skipped (this port's numbers
## are small enough in practice, and DccTheme has no established large-number
## abbreviation convention elsewhere to match).
func _tl_format_sim_result(r: Dictionary) -> String:
	var steps := int(r.get("steps", 0))
	var end_year := int(r.get("end_year", 0))
	var final_n := int(r.get("final_settlements", 0))
	var head := "Simulated %d step%s (%d yr each), %s -> %s. %d settlements remain." % [
		steps, "" if steps == 1 else "s", _tl_sim_step_years,
		_tl_format_year(_tl_sim_start_year), _tl_format_year(end_year), final_n]
	if _tl_sim_mode == "recovery":
		return head
	var died := int(r.get("died", 0))
	var migrated := int(r.get("migrated", 0))
	var unplaced := int(r.get("unplaced", 0))
	var failed := int(r.get("failed", 0))
	return "%s %d died, %d migrated (%d lost in transit/diaspora), %d settlement%s failed/abandoned." % [
		head, died, migrated, unplaced, failed, "" if failed == 1 else "s"]

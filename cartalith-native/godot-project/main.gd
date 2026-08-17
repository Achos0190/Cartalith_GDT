extends Control
## GUI shell (GUI_SHELL_SCOPE.md milestone 1): top bar with 7 domain menus,
## left workspace navigator, a second panel that swaps with the navigator
## selection, centre mode bar + viewport, right context inspector, bottom
## timeline bar. Desktop (1920x1080) dark theme only this pass -- light
## theme, panel collapse, and responsive breakpoints are explicitly deferred
## follow-up milestones (`GUI_SHELL_SCOPE.md`'s own scope).
##
## Every real, working control from the prior MVP shell is re-parented here
## unchanged -- same %unique_name node names, same signals, same Rust calls.
## Godot's `%Name` lookup resolves by unique name regardless of tree
## position, so re-parenting a node in the .tscn never breaks an existing
## `@onready var x = %Name` reference in this script, as long as the name
## and `unique_name_in_owner` are preserved (verified against every %ref
## below). Controls for features with no engine backing yet (Simulate's
## year-by-year playback, Warfare, full Politics/Trade, tile/LOD, 2D/3D)
## are real nodes, visibly present, `disabled = true` -- not hidden, not
## deleted -- per the owner's own explicit "build the full shell now, wire
## it up later" decision (GUI_SHELL_SCOPE.md).
##
## Generation still runs on a background Thread (unchanged from the prior
## shell -- godot-shell skill: "a frozen window during it is the difference
## between a tool and a toy"). `WorldGen.generate()`/`generate_world_
## structure()` are pure Rust computation over plain WorldState, safe off
## -thread; `build_color_texture()` and every scene-tree write happen back
## on the main thread via `call_deferred`.

@onready var seed_input: SpinBox = %SeedInput
@onready var resolution_input: OptionButton = %ResolutionInput
@onready var width_input: SpinBox = %WidthInput
@onready var sea_level_input: SpinBox = %SeaLevelInput
@onready var world_shape_input: OptionButton = %WorldShapeInput
@onready var dynamic_lithology_check: CheckBox = %DynamicLithologyCheck
@onready var volc_provinces_check: CheckBox = %VolcProvincesCheck
@onready var wind_deflection_check: CheckBox = %WindDeflectionCheck
@onready var ocean_currents_check: CheckBox = %OceanCurrentsCheck
@onready var generate_button: Button = %GenerateButton
@onready var load_save_button: Button = %LoadSaveButton
@onready var status_label: Label = %StatusLabel
@onready var map_view: TextureRect = %MapView
@onready var territory_view: TextureRect = %TerritoryView
@onready var province_boundary_view: TextureRect = %ProvinceBoundaryView
@onready var map_overlay: Control = %MapOverlay
@onready var civ_layer_check: CheckBox = %CivLayerCheck
@onready var territory_layer_check: CheckBox = %TerritoryLayerCheck
@onready var province_layer_check: CheckBox = %ProvinceLayerCheck
@onready var villages_check: CheckBox = %VillagesCheck
@onready var load_save_dialog: FileDialog = %LoadSaveDialog
@onready var credits_button: Button = %CreditsButton
@onready var credits_dialog: AcceptDialog = %CreditsDialog

## New this milestone: shell chrome.
@onready var readout_label: Label = %ReadoutLabel
@onready var project_menu: MenuButton = %ProjectMenu
@onready var world_menu: MenuButton = %WorldMenu
@onready var generate_menu: MenuButton = %GenerateMenu
@onready var simulate_menu: MenuButton = %SimulateMenu
@onready var map_menu: MenuButton = %MapMenu
## Not a MenuButton: the reference app's own `#assetsHeaderBtn` is a plain
## mode-switch toggle (flips its own label to "← Map" on exit), not a
## dropdown of operations -- Assets is a full-viewport mode, not an
## operations list. Converted from MenuButton to Button in this
## decluttering pass (GUI_SHELL_SCOPE.md); stays `disabled` since no
## Assets-mode viewport exists in this port yet.
@onready var assets_menu: Button = %AssetsMenu
@onready var view_menu: MenuButton = %ViewMenu
@onready var navigator_vbox: VBoxContainer = %NavigatorVBox
@onready var second_panel_header: Label = %SecondPanelHeader
@onready var overview_content: Control = %OverviewContent
@onready var placeholder_content: Control = %PlaceholderContent
@onready var placeholder_label: Label = %PlaceholderLabel
@onready var scale_bar_label: Label = %ScaleBarLabel
@onready var inspector_header: Label = %InspectorHeader
@onready var inspector_body: RichTextLabel = %InspectorBody
## §3 fix: `FooterVBox` (Generate/Load Save/Status) must be scoped to
## `WORLD:Overview` only -- it previously persisted, visible, on all 20 nav
## subjects regardless of which one was selected.
@onready var footer_separator: HSeparator = %FooterSeparator
@onready var footer_vbox: VBoxContainer = %FooterVBox

var world_gen: WorldGen = WorldGen.new()
var _gen_thread: Thread
var _generating := false
var _last_width_km := 0.0

## Index into WorldShapeInput -> the archetype name WorldGen.
## generate_world_structure expects (reference HTML `ARCHETYPES`). Index 0
## ("Classic") isn't an archetype at all -- it's World-Structure disabled,
## the plain `generate()` path.
const WORLD_SHAPES: Array[String] = ["", "earth", "supercontinent", "archipelago", "volcanic", "rift"]

## Display labels shown in the dropdown, same order/index as WORLD_SHAPES.
const WORLD_SHAPE_LABELS: Array[String] = ["Classic", "Earth-like", "Supercontinent", "Archipelago", "Volcanic", "Rift"]

## Reference HTML's real "Working resolution" presets (`#resSeg` buttons:
## 512/1K/2K/4K/8K, default 2K) -- this port previously capped at 512 via a
## 32-512 SpinBox, far below what the reference actually offers by default.
const RESOLUTION_PRESETS: Array[int] = [512, 1024, 2048, 4096, 8192]
const RESOLUTION_LABELS: Array[String] = ["512", "1K", "2K", "4K", "8K"]
const RESOLUTION_DEFAULT_INDEX := 2 ## 2K, matching the reference's own default.

## Workspace navigator, GUI decluttering pass (2026-08-17): every group and
## subject below is grounded in the reference app's real IA (`Cartalith
## Gen1 v2.10.html`'s Generate->{World, Civilization, Cartography, Sculpt}
## branches plus Explore, the reference's real second mode) -- not the
## prior pass's invented `INFRASTRUCTURE` bucket (Roads/Rivers/Ports/Trade/
## Logistics), which had zero grounding there and is replaced by `EXPLORE`.
## `CARTOGRAPHY:Layers` is also gone as a nav subject: it and the debug/
## analysis layer picker were two surfaces for one thing (the reference
## app's own admission); consolidated into `LayersPanel` alone (always
## visible, not a destination), freeing CARTOGRAPHY's 5th slot for `Paint`
## -- the reference's real brush bucket, previously homeless. Only
## WORLD:Overview has a real parameter panel (`OverviewContent` in the
## .tscn); every other subject swaps in `PlaceholderContent`, now with a
## subject-specific honest hint (`NAV_SUBJECT_HINTS` below) instead of one
## generic message -- per GUI_SHELL_SCOPE.md's own inventory, none of them
## have engine backing yet, but they're at least named and grouped where
## the reference app actually puts them.
const NAV_GROUPS := {
	"WORLD": ["Overview", "Terrain", "Water", "Climate", "Ecology", "Sculpt"],
	"CIVILIZATION": ["Settlements", "Factions", "Economy", "Generation", "Statistics"],
	"CARTOGRAPHY": ["Map Style", "Labels", "Icons", "Map View", "Paint"],
	"EXPLORE": ["Tools", "Timeline", "Info", "Journeys", "Journey Planner"],
}
## The only subject with real *parameter-panel* content this milestone --
## everything else in NAV_GROUPS falls through to the placeholder, worded
## per `NAV_SUBJECT_HINTS`.
const NAV_REAL_SUBJECTS := ["WORLD:Overview"]

## Per-subject honest placeholder text, reference-grounded (this plan's
## own §2/§6): each names the *real* controls that subject corresponds to
## in the reference app, not a generic "not wired yet" -- a concrete
## honesty improvement even before any of them get engine backing.
## `CIVILIZATION:Settlements` is the one exception with real backing today
## (settlements render, hover works) -- its hint is a redirect to where
## that real data already lives, matching the pattern `LAYERS` established,
## not the generic "not wired" text, which would be actively misleading.
const NAV_SUBJECT_HINTS := {
	"WORLD:Terrain": "Tectonics (plates, drift, warp, uplift, erosion age) and volcanism & impact controls, matching the reference app's Geology stage. Not wired yet -- the four experimental flags on Overview are the only exposed knobs so far.",
	"WORLD:Water": "Droplet / hillslope / stream-power / velocity erosion, evolve & sediment, glacial, and coastal controls, matching the reference app's Hydrology stage. Not wired yet.",
	"WORLD:Climate": "Climate & biome classification and the weather / rainfall simulator, matching the reference app's Climate stage. Not wired yet.",
	"WORLD:Ecology": "River-in-biome, river density, minimum stream order, sharper biomes, and lakes-as-water controls, matching the reference app's Ecology stage. Not wired yet.",
	"WORLD:Sculpt": "The reference app's 4th Generate branch: stamp editor (feature type, presets, brush & noise, feature params, stamp stack, commit/discard). No engine backing in this port yet.",
	"CIVILIZATION:Settlements": "Settlements have real engine backing -- they render on the map and respond to hover. Hover a marker to see its data and causal \"why here?\" chain in the INSPECTOR panel to the right. A dedicated searchable/sortable table here is not yet built.",
	"CIVILIZATION:Factions": "Faction pills, add/remove, and an \"Open Faction Roster...\" modal, matching the reference app's faction management. Not wired yet.",
	"CIVILIZATION:Economy": "Aggregate faction economics report, matching the reference app's Economy stage. Not wired yet (see ECONOMY_SCOPE.md).",
	"CIVILIZATION:Generation": "Step 1 populate (counts, Auto-populate, toggles) -> Step 2 roads (Generate Roads, ways list) -> Step 3 territories/provinces, plus a display fold (settlement/way scale, opacity) -- the reference app's real Civilization generation sequence. Not wired yet; see the Generate menu for the same three steps as menu actions.",
	"CIVILIZATION:Statistics": "World and per-faction statistics report, matching the reference app's Culture stage. Not wired yet.",
	"CARTOGRAPHY:Map Style": "Presets (Default/Antique/Ink/Watercolor/Print), a rendering-advanced fold, the painter NPR fold, and overlays, matching the reference app's map styling. Not wired yet (see TERRAIN_APPEARANCE_SCOPE.md).",
	"CARTOGRAPHY:Labels": "Region name labels, matching the reference app's Labels stage. Not wired yet.",
	"CARTOGRAPHY:Icons": "Manual icon placement: category, gallery, density brush, and the placed-icons list, matching the reference app's Assets stage. Not wired yet.",
	"CARTOGRAPHY:Map View": "Mode segment (Biome/Relief/Height/Shade), relief<->biome blend, exaggeration, sun angle, and hillshade toggle. Not wired yet.",
	"CARTOGRAPHY:Paint": "Paint toggle, layer segment (Biome/Splat/Terrain), value, radius, erase, texture strength, clear, and pack gallery -- the reference app's Paint brush. Not wired yet.",
	"EXPLORE:Tools": "Info / route tool palette, snap-to-places-and-ways toggle, and Commit route. Not wired yet -- the engine is a one-shot static generator, not a continuous simulation (HARDWARE_ACCELERATION.md).",
	"EXPLORE:Timeline": "Add-year input and pills, \"show only objects in selected year\" / Ghost / Highlight toggles, and a nested Simulate collapse/recovery fold -- year management and simulation, distinct from the BottomBar's own play/pause/step/speed scrub controls. Not wired yet.",
	"EXPLORE:Info": "Click-to-inspect readout feeding the Inspector panel; on a settlement hit, opens the full-screen City Viewer in the reference app. Settlement hover already feeds the Inspector today (see CIVILIZATION > Settlements) -- the rest is not wired yet.",
	"EXPLORE:Journeys": "Journey list, matching the reference app's Explore > Journeys. Not wired yet.",
	"EXPLORE:Journey Planner": "Compact summary plus \"Edit route...\" opening the full-screen Route Editor, matching the reference app's Journey Planner. Not wired yet.",
}

var _nav_buttons: Dictionary = {} ## "GROUP:Subject" -> Button, for active-state styling


func _ready() -> void:
	## Was never populated at all (scene nor script) -- OptionButton.selected
	## defaults to -1 with no items, and GDScript's negative indexing meant
	## `WORLD_SHAPES[world_shape_input.selected]` silently resolved to the
	## LAST entry ("rift") instead of erroring or defaulting to Classic.
	## Caught by hands-on testing (a real generate() run), not by review.
	for label in WORLD_SHAPE_LABELS:
		world_shape_input.add_item(label)
	world_shape_input.selected = 0

	for label in RESOLUTION_LABELS:
		resolution_input.add_item(label)
	resolution_input.selected = RESOLUTION_DEFAULT_INDEX

	generate_button.pressed.connect(_on_generate_pressed)
	load_save_button.pressed.connect(_on_load_save_pressed)
	load_save_dialog.file_selected.connect(_on_save_file_selected)
	credits_button.pressed.connect(func(): credits_dialog.popup_centered())
	civ_layer_check.toggled.connect(func(pressed: bool): map_overlay.visible = pressed)
	territory_layer_check.toggled.connect(func(pressed: bool): territory_view.visible = pressed)
	province_layer_check.toggled.connect(func(pressed: bool): province_boundary_view.visible = pressed)
	map_overlay.settlement_hovered.connect(_on_settlement_hovered)

	_build_navigator()
	_build_menus()
	_select_nav_subject("WORLD", "Overview")


## Builds the 4-group workspace navigator (`design/cartalith-menu-structure.md`
## §"Shell regions") from `NAV_GROUPS` -- 20 subject rows would be tedious
## and error-prone to hand-author as individual .tscn nodes, so they're
## generated here instead. Each row is a real, clickable flat Button;
## `_nav_buttons` keeps a flat "GROUP:Subject" -> Button map so
## `_select_nav_subject` can restyle whichever row is active without a tree
## walk.
func _build_navigator() -> void:
	for group_name in NAV_GROUPS:
		var group_header := Label.new()
		group_header.text = group_name
		group_header.add_theme_color_override("font_color", Color(0.552941, 0.576471, 0.588235))
		group_header.add_theme_font_size_override("font_size", 9)
		navigator_vbox.add_child(group_header)

		var group_box := VBoxContainer.new()
		navigator_vbox.add_child(group_box)

		for subject in NAV_GROUPS[group_name]:
			var btn := Button.new()
			btn.text = subject
			btn.flat = true
			btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
			btn.custom_minimum_size = Vector2(0, 30)
			btn.add_theme_color_override("font_color", Color(0.784314, 0.796078, 0.803922))
			var key := "%s:%s" % [group_name, subject]
			btn.pressed.connect(_select_nav_subject.bind(group_name, subject))
			_nav_buttons[key] = btn
			group_box.add_child(btn)


## Swaps the second panel's content and restyles the active navigator row.
## Per `design/cartalith-menu-structure.md`'s own architectural rule: "A
## navigator node never swaps the viewport or the application -- it swaps
## the tool palette and the inspector around it." Only the viewport itself
## (and inspector's live selection state) are untouched by this.
func _select_nav_subject(group_name: String, subject: String) -> void:
	var key := "%s:%s" % [group_name, subject]
	for k in _nav_buttons:
		var btn: Button = _nav_buttons[k]
		btn.add_theme_color_override("font_color",
			Color(0.878431, 0.639216, 0.290196) if k == key else Color(0.784314, 0.796078, 0.803922))

	second_panel_header.text = "%s · %s" % [group_name, subject.to_upper()]
	overview_content.visible = false
	placeholder_content.visible = false
	## §3 fix: the footer (Generate/Load Save/Status) is `WORLD:Overview`-
	## only chrome -- it used to persist, visible, across all 20 subjects.
	var is_overview := key == "WORLD:Overview"
	footer_separator.visible = is_overview
	footer_vbox.visible = is_overview
	if is_overview:
		overview_content.visible = true
	else:
		placeholder_content.visible = true
		placeholder_label.text = NAV_SUBJECT_HINTS.get(key, "This workspace subject isn't wired to the engine yet.")


## Populates the 6 remaining top-bar domain popups (Assets is a plain
## toggle `Button`, not a `MenuButton` -- see its `@onready` doc comment)
## per this decluttering pass's §1: real header/Generate-branch content
## from the reference app, not the prior pass's invented items. Real items
## call an existing, already-wired action; disabled items are present and
## readable, not a functional no-op silently doing nothing, per
## GUI_SHELL_SCOPE.md's "visibly present but honestly inert" rule.
func _build_menus() -> void:
	## ProjectMenu: `New world...` / `Save project` deleted outright -- zero
	## reference grounding (the reference has no such File▾ entries; "save"
	## *is* Export .zip, world creation is the onboarding gate, not a menu
	## action). Replaced with an honest, disabled Import group (Load
	## heightmap / Infer tectonics / Import asset pack -- none exist in the
	## Rust core yet) and an honest Export group (Export .zip / GeoJSON).
	var project_popup := project_menu.get_popup()
	project_popup.add_item("Open project (.zip)...")
	project_popup.add_item("Credits")
	project_popup.add_separator()
	for item in ["Load heightmap...", "Infer tectonics from heightmap...", "Import asset pack..."]:
		project_popup.add_item(item)
		project_popup.set_item_disabled(project_popup.item_count - 1, true)
	project_popup.add_separator()
	for item in ["Export .zip", "Export GeoJSON"]:
		project_popup.add_item(item)
		project_popup.set_item_disabled(project_popup.item_count - 1, true)
	project_popup.id_pressed.connect(_on_project_menu_id)

	## WorldMenu: same two real items as before (call the same functions as
	## Overview's own footer buttons -- a legitimate header shortcut, not a
	## duplicate implementation). `Planet settings...`/`Coordinate system...`
	## stay disabled, but their *future* behavior is now a jump-to-section
	## contract (scroll/expand Overview's Planet subsection once wired), not
	## a second settings surface -- avoids building a duplicate modal later.
	var world_popup := world_menu.get_popup()
	world_popup.add_item("Generate World")
	world_popup.add_item("New seed")
	world_popup.add_separator()
	world_popup.add_item("Planet settings...")
	world_popup.set_item_disabled(world_popup.item_count - 1, true)
	world_popup.add_item("Coordinate system / projection...")
	world_popup.set_item_disabled(world_popup.item_count - 1, true)
	world_popup.id_pressed.connect(_on_world_menu_id)

	## GenerateMenu: replaces the prior flat 11-stage placeholder list (it
	## implied one monolithic pipeline that doesn't match the reference's
	## real branch structure) with the reference's actual Civilization
	## Step 1->2->3 sequence, the single largest piece of invented structure
	## in the prior shell.
	var generate_popup := generate_menu.get_popup()
	for item in ["Auto-populate world (Step 1)", "Generate Roads (Step 2)", "Recalculate Territories (Step 3)"]:
		generate_popup.add_item(item)
		generate_popup.set_item_disabled(generate_popup.item_count - 1, true)
	generate_popup.add_separator()
	generate_popup.add_item("Generate Provinces")
	generate_popup.set_item_disabled(generate_popup.item_count - 1, true)

	## SimulateMenu: renamed to the reference's real Explore/Timeline
	## operations (same 4-item slot count as before).
	var simulate_popup := simulate_menu.get_popup()
	for item in ["Add year...", "Animate playback", "Simulate collapse / recovery...",
			"(Explore phase — not yet implemented)"]:
		simulate_popup.add_item(item)
		simulate_popup.set_item_disabled(simulate_popup.item_count - 1, true)

	## MapMenu: same 3 items, but their future behavior is now a
	## navigator-jump contract -- each maps 1:1 to a real CARTOGRAPHY
	## subject (Map View / Map Style's NPR fold / Labels) instead of a
	## future modal, preventing a future duplicate-surface bug. No "Layers"
	## item here: `LayersPanel` is the one real layer surface, always
	## visible, not a nav destination a menu item could echo.
	var map_popup := map_menu.get_popup()
	map_popup.add_item("Terrain appearance...")  # -> jump to CARTOGRAPHY:Map View
	map_popup.set_item_disabled(map_popup.item_count - 1, true)
	map_popup.add_item("Painter styles (NPR)")  # -> jump to CARTOGRAPHY:Map Style's NPR fold
	map_popup.set_item_disabled(map_popup.item_count - 1, true)
	map_popup.add_item("Labels & annotation")  # -> jump to CARTOGRAPHY:Labels
	map_popup.set_item_disabled(map_popup.item_count - 1, true)

	## AssetsMenu has no popup to build -- it's a plain toggle `Button` now,
	## see its `@onready` doc comment.

	## ViewMenu: renamed to real reference-grounded items (same 4-item slot
	## count).
	var view_popup := view_menu.get_popup()
	for item in ["3D view dials... (relief exaggeration / detail / light / flatten oceans)",
			"Focus Layers panel", "Tiled LOD view (cartalith-spatial, unintegrated)", "Atlas cache"]:
		view_popup.add_item(item)
		view_popup.set_item_disabled(view_popup.item_count - 1, true)


func _on_project_menu_id(id: int) -> void:
	match id:
		0: _on_load_save_pressed()
		1: credits_dialog.popup_centered()


func _on_world_menu_id(id: int) -> void:
	match id:
		0: _on_generate_pressed()
		1: seed_input.value = randi() % 1000000


## Noun phrases for each suitability term key returned by
## `WorldGen.explain_settlement()`. Wording lives here, not in Rust: the
## engine supplies facts, the UI phrases them (ARCHITECTURE.md -- Godot
## computes nothing beyond layout, and wording is layout).
const SUIT_TERM_LABELS := {
	"carrying_capacity": "fertile land",
	"water_access": "fresh water",
	"gentle_slope": "gentle terrain",
	"terrain_form": "terrain form",
	"coastal_access": "coastal access",
	"river": "river access",
	"lake": "lakeside",
	"minerals": "mineral deposits",
	"route_corridor": "natural route corridor",
	"farmland": "farmland",
	"buildable_ground": "buildable ground",
	"flood_risk": "flood risk",
	"islet_penalty": "isolation",
	"water_bonus": "water",
}

## Qualifies a term by its own raw 0..1 reading, so the chain stays honest:
## a settlement placed on mediocre soil reads as "weak farmland", not as a
## flattering "farmland". Deliberately describes the reading, not the rank.
func _term_strength(value: float) -> String:
	if value >= 0.75:
		return "strong"
	if value >= 0.45:
		return "moderate"
	if value > 0.05:
		return "weak"
	return "negligible"


func _describe_term(t: Dictionary) -> String:
	var key := String(t["key"])
	var label: String = SUIT_TERM_LABELS.get(key, key.replace("_", " "))
	return "%s %s (%.2f)" % [_term_strength(float(t["value"])), label, float(t["value"])]


## Updates the Inspector panel (right) on hover: the settlement's own data
## plus a real "why here?" causal chain, decomposed from the very
## suitability score that placed it (`WorldGen.explain_settlement`,
## VISION.md). `data` is `null` and `index` `-1` on hover-exit.
func _on_settlement_hovered(data: Variant, index: int) -> void:
	if data == null:
		inspector_header.text = "INSPECTOR · NO SELECTION"
		inspector_body.text = "No selection.\n\nHover a settlement marker on the map to inspect it. A full per-cell inspector (elevation, slope, aspect, drainage, etc. at the cursor) needs a new engine query this milestone doesn't add -- see GUI_SHELL_SCOPE.md."
		return
	var s: Dictionary = data
	inspector_header.text = "INSPECTOR · SETTLEMENT"
	var kind_label: String = String(s["kind"]).capitalize()
	var lines := [
		"[b]%s[/b] (%s)" % [s["name"], kind_label],
		"Population: %s" % s["population"],
		"Faction: %d" % s["faction"],
		"Coastal: %s" % ("yes" if s["coastal"] else "no"),
		"Capital: %s" % ("yes" if s["capital"] else "no"),
	]

	var why: Dictionary = world_gen.explain_settlement(index)
	if not why.is_empty():
		lines.append("")
		lines.append("[b]WHY HERE?[/b]")
		if why.has("excluded"):
			# A placed settlement shouldn't sit on an excluded cell; if it
			# ever does, say so plainly rather than inventing a rationale.
			lines.append("Cell excluded from suitability (%s)." % why["excluded"])
		else:
			var terms: Array = why["terms"]
			# `terms` arrives sorted most-decisive-first. Positives are the
			# reasons it's here; negatives are what it was placed in spite of.
			var positives: Array[String] = []
			var negatives: Array[String] = []
			for t: Dictionary in terms:
				var c := float(t["contribution"])
				if c > 0.005 and positives.size() < 3:
					positives.append(_describe_term(t))
				elif c < -0.005 and negatives.size() < 2:
					negatives.append(_describe_term(t))
			if positives.is_empty():
				lines.append("No single factor stands out -- placed on broadly average ground.")
			else:
				lines.append(" → ".join(positives))
			if not negatives.is_empty():
				lines.append("Despite: %s" % ", ".join(negatives))
			lines.append("Suitability %.2f" % float(why["score"]))

		lines.append("")
		var ord_i := int(why["river_order"])
		var river_txt := ("Strahler %d" % ord_i) if ord_i > 0 else "none"
		var coast_cells := float(why["coast_dist_cells"])
		lines.append("River: %s · flow %.0f" % [river_txt, float(why["flow"])])
		# Spelled out as distance-to-water rather than "coast", because the
		# settlement's own `Coastal` flag above uses a much wider radius
		# (max(6, GW/60) cells -- port eligibility) than the suitability
		# coast bonus does (a 5-cell falloff). A settlement can honestly be
		# coastal AND have earned no coastal bonus; labelling this "coast"
		# made those two lines read as a contradiction when they aren't.
		lines.append("Distance to water: %.1f cells" % coast_cells)
		lines.append("Elevation: %.3f (normalised)" % float(why["elevation"]))
		lines.append("Travel cost: %.2f" % float(why["travel_cost"]))

	inspector_body.text = "\n".join(lines)


func _on_generate_pressed() -> void:
	if _generating:
		return
	_generating = true
	generate_button.disabled = true
	status_label.text = "generating..."
	readout_label.text = "generating..."

	var seed_value := int(seed_input.value)
	var resolution := RESOLUTION_PRESETS[resolution_input.selected]
	var width_km := width_input.value
	var archetype := WORLD_SHAPES[world_shape_input.selected]

	## All four golden-verified against the real JS engine
	## (cartalith-native/docs/CHANGELOG.md). Still exposed as toggles --
	## default checked state matches each one's real JS default.
	world_gen.set_experimental_flags(
		dynamic_lithology_check.button_pressed,
		volc_provinces_check.button_pressed,
		wind_deflection_check.button_pressed,
		ocean_currents_check.button_pressed,
	)
	## Reference `_civVillages` default OFF (Phase 2 milestone 15) --
	## gated separately from the four flags above since it's civ-layer,
	## not terrain-substrate.
	world_gen.set_villages_enabled(villages_check.button_pressed)
	## `MVP_SCOPE.md` point 9 / reference `state.seaLevel`. UI is a 0-100%
	## SpinBox (matching the reference's own `#seaV` slider convention);
	## WorldGen.set_sea_level expects the raw [0,1] fraction.
	world_gen.set_sea_level(sea_level_input.value / 100.0)

	_gen_thread = Thread.new()
	_gen_thread.start(_generate_worker.bind(seed_value, width_km, resolution, archetype))


## Runs off the main thread. Touches only `world_gen` (plain Rust state),
## never a node -- see the class doc comment above. `generate()` and
## `generate_world_structure()` are both full, equally expensive
## `generate_terrain()` calls that mutate the same `world_gen` state --
## this must be the ONE call site. `archetype` empty == Classic
## (World-Structure disabled).
func _generate_worker(seed_value: int, width_km: float, resolution: int, archetype: String) -> void:
	var ok := true
	if archetype.is_empty():
		world_gen.generate(seed_value, width_km, resolution)
	else:
		ok = world_gen.generate_world_structure(seed_value, width_km, resolution, archetype)
	_on_generate_done.call_deferred(seed_value, width_km, ok)


## Deferred back to the main thread: joins the worker, then does the one
## Rust call that builds a Godot resource (`build_color_texture`) and every
## scene-tree write. `ok` is false only for an unrecognized archetype
## string (defensive -- `WORLD_SHAPES` only ever supplies known values).
func _on_generate_done(seed_value: int, width_km: float, ok: bool) -> void:
	_gen_thread.wait_to_finish()
	_gen_thread = null

	if not ok:
		status_label.text = "generate failed — see console"
		readout_label.text = "generate failed"
		generate_button.disabled = false
		_generating = false
		return

	var tex: ImageTexture = world_gen.build_color_texture()
	if tex:
		map_view.texture = tex
		## Phase 2 civilisation layer (cartalith-civ): computed automatically
		## by generate()/generate_world_structure() itself (see
		## cartalith-godot's WorldGen), so it's already ready here -- just
		## fetch and hand to the overlay. No civ data for a loaded save
		## (see WorldGen.load_save's own doc comment), only a fresh generate.
		var settlements := world_gen.get_settlements()
		var roads := world_gen.get_roads()
		var sea_routes := world_gen.get_sea_routes()
		map_overlay.set_civ_data(settlements, roads, sea_routes, world_gen.get_width(), world_gen.get_height())
		territory_view.texture = world_gen.build_territory_texture()
		## Province boundaries (Phase 2, civ_generate_provinces): thin lines
		## only, drawn on top of territory's own per-faction fill -- see
		## `build_province_boundary_texture`'s own doc comment for why this
		## isn't a per-province fill colour (province count is unbounded,
		## unlike CIV_FACTION_COUNT).
		province_boundary_view.texture = world_gen.build_province_boundary_texture()

		_last_width_km = width_km
		_update_scale_bar()

		var shape_label := world_shape_input.get_item_text(world_shape_input.selected)
		var civ_note := ", %d settlements" % settlements.size() if not settlements.is_empty() else ""
		status_label.text = "%dx%d, seed %d, %.0f km, %s%s" % [
			world_gen.get_width(), world_gen.get_height(), seed_value, width_km, shape_label, civ_note
		]
		readout_label.text = "seed %d · %dx%d · %.0f km" % [seed_value, world_gen.get_width(), world_gen.get_height(), width_km]
	else:
		status_label.text = "generate failed — see console"
		readout_label.text = "generate failed"

	generate_button.disabled = false
	_generating = false


## Top-bar readout has no live CPU/GPU/memory number wired this milestone
## (`GUI_SHELL_SCOPE.md`'s own "ambiguous, verify before building" note --
## Godot's `Performance` singleton has some of this natively but wiring it
## honestly wasn't this pass's focus); the scale bar is real, computed from
## the actual generated map width in km against the viewport's own map-view
## pixel width.
func _update_scale_bar() -> void:
	if _last_width_km <= 0.0 or map_view.size.x <= 0.0:
		scale_bar_label.text = ""
		return
	var km_per_px := _last_width_km / map_view.size.x
	var bar_km := 100.0
	if km_per_px > 0.0:
		bar_km = roundf(100.0 * km_per_px) / km_per_px / 100.0 * 100.0
	scale_bar_label.text = "%.0f km across viewport" % _last_width_km


func _on_load_save_pressed() -> void:
	load_save_dialog.popup_centered()


## `MVP_SCOPE.md` criterion 7: opens a real HTML-app `.zip` and renders that
## save's terrain. `WorldGen.load_save` (`cartalith-io`) reads the save's
## own stored fields directly -- no `generate()` call involved, so whatever
## grid size/seed/map width the save was exported at is what's shown.
func _on_save_file_selected(path: String) -> void:
	status_label.text = "loading %s..." % path.get_file()
	if not world_gen.load_save(path):
		status_label.text = "load failed — see console"
		return
	var tex: ImageTexture = world_gen.build_color_texture()
	if tex:
		map_view.texture = tex
		## No civ data in a loaded save (WorldGen.load_save's own doc
		## comment) -- clear any settlements/roads/territory left over
		## from a previous generate() so a stale overlay doesn't linger.
		map_overlay.set_civ_data([], [], [], world_gen.get_width(), world_gen.get_height())
		territory_view.texture = null
		province_boundary_view.texture = null
		status_label.text = "loaded %s (%dx%d)" % [
			path.get_file(), world_gen.get_width(), world_gen.get_height()
		]
		readout_label.text = "loaded %s" % path.get_file()
	else:
		status_label.text = "load succeeded but render failed — see console"

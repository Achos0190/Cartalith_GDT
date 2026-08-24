extends Workspace
class_name RenderWorkspace

## Terrain appearance groups; right dock preview & quality -- formerly the
## standalone RENDER rail domain (§3).
##
## Mirrors the reference's own Cartography ▸ **Map view** and **Map style**
## blocks (reference HTML lines 1706-1783), in the reference's own order:
## Map view, then the style presets, then Rendering-advanced, then the Painter
## (NPR) styles, then the overlays.
##
## `render.rs`'s `TerrainAppearance` used to be settable in Rust and bound to
## nothing, which is what the "not built" note here used to say; it is now
## bound through `WorldGen::{get_appearance, set_appearance,
## list_appearance_tunables, reset_appearance}` and driven by `_build_map_view`
## / `_build_appearance` below. The non-photorealistic half (the ten "Painter"
## styles, the coastal wave lines, animated water and multi-sun) came live one
## pass earlier -- see `_build_npr`.
##
## **Domain merge (2026-08-20, owner instruction: "And render into carto").**
## RENDER no longer has its own rail button; this one-subject class is now
## composed into `CartographyWorkspace` (`cartography_workspace.gd`'s own
## `_render` field) as a nested `VBoxContainer` appended after CARTO's own
## three categories. `_nested` (set true by `cartography_workspace.gd` before
## `setup()`) skips this file's own TOOLS block -- CARTO's own already covers
## the three global tools (plus Icon/Label), and RENDER never had a
## domain-specific tool of its own to add to it. This also directly resolves
## the CA-01/RN-01 ambiguity `GUI_GAP_REGISTER.md` §8.6 flagged: CARTO and
## RENDER were both proposing to own the same future `set_appearance()`
## binding; merging the domains removes the split.
var _nested := false

## The ten Painter styles, in the reference's own application order, each with
## the label and the one-line description the reference's own hint text gives
## it (reference HTML line 1763). The keys are `WorldGen::set_npr`'s, and a key
## it does not recognise is a no-op there (`set_npr` returns 0 and `_push`
## does nothing), so a drift between the two shows as a dead row rather than
## as a wrong picture.
const STYLES := [
	["watercolor", "Watercolor", "Pigment pooling, paper granulation and edge blooms."],
	["contours", "Contour veins", "Constant-width elevation isolines; every fifth is an index line."],
	["ink", "Ink linework", "Pen outlines on strong landform edges, with hand-drawn weight wobble."],
	["hachure", "Hachure", "Downslope hatching, denser and darker on steeper ground."],
	["cel", "Cel / toon", "Posterized flat colour bands."],
	["crosshatch", "Engraving", "Antique cross-hatch; more hatch directions as the ground darkens."],
	["stipple", "Stipple", "Pen dot-density shading, denser in darker regions."],
	["sepia", "Sepia", "Antique warm brown toning."],
	["risograph", "Risograph", "Indigo-to-amber duotone screen print with halftone dots."],
	["pointillism", "Pointillism", "Seurat-style coloured dot field."],
]

## `preload`, not the `WaterAnimLayer` global class name -- the same reason
## `layers_popover.gd` preloads `wind_fx_layer.gd`: a global name resolves only
## once the editor has rescanned and written the class cache, so a fresh clone
## or an editor-less capture run would fail to parse this file.
const WATER_ANIM_SCRIPT := preload("res://shell/water_anim_layer.gd")

## The reference's five Map-style presets (reference HTML line 12850's
## `STYLE_PRESETS`), each an **absolute bundle**: every managed key first goes
## back to its own default, then the preset's overrides apply. That is what
## makes "Default" reproduce the base look exactly rather than approximately.
##
## Two of the reference's own preset keys have no counterpart here yet, and
## the panel says so rather than quietly dropping them:
##
## * `parchment` (antique 0.6, watercolor 0.2). The reference's parchment is
##   off by default; **this port's paper ground is on at 0.85** and is what
##   `VISION.md` asks for, so mapping the reference's number in would *reduce*
##   the parchment on the very preset that wanted more of it. Parchment is a
##   live slider in Rendering-advanced below instead, and the presets leave it
##   where the quality tier put it.
## * `icons` (antique true) -- the stylized mountain/hill/tree glyph layer,
##   which this port has not built at all.
const STYLE_PRESETS := [
	["Default", {}],
	["Antique", {"sepia": 0.35}],
	["Ink", {"ink": 0.6, "contours": 0.35}],
	["Watercolor", {"watercolor": 0.65}],
	["Print", {"risograph": 0.5, "contours": 0.25}],
]

## What a preset resets before applying itself -- the reference's own
## `STYLE_MANAGED_NUM`/`STYLE_MANAGED_BOOL`, intersected with what this port
## binds. `contour_m` is 0 = the reference's own automatic interval.
const STYLE_MANAGED := {
	"watercolor": 0.0, "contours": 0.0, "contour_m": 0.0, "ink": 0.0,
	"hachure": 0.0, "cel": 0.0, "crosshatch": 0.0, "stipple": 0.0,
	"sepia": 0.0, "risograph": 0.0, "pointillism": 0.0,
	"waves": false, "multi_sun": false,
}

## Which appearance keys go in which group, in panel order. Keys the running
## cdylib does not publish are skipped, so an older binary loses rows rather
## than drawing dead ones -- the same degrade `appearance_api` already gives.
const APPEARANCE_VIEW := ["exag", "sun_az_deg", "sun_alt_deg", "bio_blend"]
const APPEARANCE_GROUPS := [
	["Relief & light", ["relief_lights", "relief_directionality", "relief_ambient",
		"relief_gain", "ao_strength", "ao_radius_frac"]],
	["The sheet", ["paper_strength", "paper_grain", "paper_mottle", "paper_wash",
		"stipple_strength", "border_width_frac"]],
	["Materials", ["litho_strength", "litho_exposure", "hydro_wet_strength",
		"local_contrast", "splat_strength"]],
]

## Per-key presentation only: the step, the unit, and a display `scale` for the
## four keys the engine holds as a fraction of grid width (0.014 reads as
## nothing on a slider; 1.4% reads as a number). Anything absent gets the
## 0..1 default of step 0.01, no unit, scale 1.
const APPEARANCE_UI := {
	"exag": {"step": 0.1, "unit": "x"},
	"sun_az_deg": {"step": 5.0, "unit": "deg"},
	"sun_alt_deg": {"step": 1.0, "unit": "deg"},
	"relief_lights": {"step": 1.0, "unit": ""},
	"relief_gain": {"step": 0.02, "unit": ""},
	"ao_radius_frac": {"step": 0.1, "unit": "%", "scale": 100.0},
	"border_width_frac": {"step": 0.1, "unit": "%", "scale": 100.0},
	"paper_grain": {"step": 0.5, "unit": "%", "scale": 100.0},
	"paper_mottle": {"step": 0.5, "unit": "%", "scale": 100.0},
}

## One line per control, in the panel. The engine publishes a label but not a
## reason, and a slider called "Directionality" teaches nobody anything.
const APPEARANCE_HELP := {
	"exag": "Vertical exaggeration of the relief the hillshade is computed from. The reference's own Relief slider.",
	"sun_az_deg": "Compass bearing the light comes from. 315deg (north-west) is the cartographic convention -- lighting from the south-east makes ridges read as valleys.",
	"sun_alt_deg": "How high the sun sits. Low angles lengthen the shading and exaggerate small landforms.",
	"bio_blend": "How far the biome colour pulls away from grey relief. 0 is a pure grey shaded-relief map; 1 is full biome colour.",
	"relief_lights": "Hillshade light directions, evenly spaced from the sun azimuth. 1 is the reference's exact single-sun shading; more reveals ridges that run parallel to the primary sun.",
	"relief_directionality": "How far the primary sun dominates the others. 1 is near-single-light; 0 is fully omnidirectional, which flattens the relief completely.",
	"relief_ambient": "Floor of the light curve -- how bright fully-shadowed ground stays.",
	"relief_gain": "Gain of the light curve, above the ambient floor.",
	"ao_strength": "Ambient occlusion: darkens enclosed valleys and gorges that see less sky. The reference's own Ambient occlusion slider.",
	"ao_radius_frac": "How wide the occlusion samples, as a share of map width -- a basin scale, not a pixel scale.",
	"paper_strength": "The paper/vellum ground: fibre, tooth, ageing and a warm tint. The reference's Parchment slider, on by default here because this port's base look is an atlas plate.",
	"paper_grain": "Amplitude of the sheet's fibre and laid lines.",
	"paper_mottle": "Amplitude of the broad ageing/staining blotches, at sheet scale.",
	"paper_wash": "How far the colour is muted toward a paper-coloured grey of the same luminance. Costs chroma only, never relief or biome legibility.",
	"stipple_strength": "Forest stippling -- canopy texture driven by the real canopy fraction, applied zero-mean so the forest gains texture without darkening.",
	"border_width_frac": "Width of the plate frame (bare-paper margin plus neatlines), as a share of map width. 0 removes the frame.",
	"litho_strength": "Geology tint: how far exposed rock moves from the climate heuristic toward the palette of the rock actually underneath. The reference's Geology materials slider. Inert on a loaded save, which stores no lithology.",
	"litho_exposure": "How strongly bedrock shows through the soil cover, from slope, vegetation and moisture.",
	"hydro_wet_strength": "Wetness: darkens and cools persistently saturated ground near channels. The reference's own Wetness slider. Measured near-invisible at working resolution -- the tint's log-flow gate leaves it under 0.001% of pixels at 2048 wide, so this row is honest about the engine and the engine needs a retune (GUI_GAP_REGISTER.md CA-11).",
	"local_contrast": "Adds band-limited detail back after the paper wash. The gain falls to zero on strong edges, so coastlines and snowlines cannot halo.",
	"splat_strength": "How strongly a loaded asset pack's ground textures blend in. Inert with no pack loaded. The reference's Texture strength.",
}

var _water_anim: Control
var _anim_check: CheckBox

## key -> the `DccWidgets.slider` handle, so a preset can move the control the
## user is looking at rather than only the value behind it.
var _npr_rows: Dictionary = {}
var _app_rows: Dictionary = {}
var _preset_chips: Array[Button] = []
var _custom_note: Label
## True only while `_apply_preset` runs, so the preset's own writes do not
## trip the "Custom" mark the reference flips on any manual edit.
var _applying := false

func _build() -> void:
	## §4.5.1's three global tools -- RENDER owns no tool of its own (no
	## §4.5.x section names one), so this was always the whole TOOLS block.
	## Skipped when nested -- see this file's own class doc.
	if not _nested:
		DccWidgets.tools_block(self, app, app.tool_group)
	_build_map_view()
	_build_map_style()
	_build_appearance()
	_build_npr()
	_build_owed_inventory()

# -- The reference's Map view block (reference HTML 1706-1717) -----------------

## The reference's four Map-view rows, minus its `modeSeg` (Biome / Relief /
## Height / Shade) and `shadeOnHypso`: those select a *base render mode*, and
## this port resolves base-mode switching through the Layers popover's own
## view list (`layers_popover.gd`'s own note on the split) rather than a second
## competing picker. Everything else here is one-to-one.
func _build_map_view() -> void:
	if not bridge.appearance_api:
		return
	var body := DccWidgets.section(self, "Map view")
	for key in APPEARANCE_VIEW:
		_appearance_slider(body, key)

# -- The reference's Map style presets (reference HTML 1719-1729) --------------

func _build_map_style() -> void:
	if not bridge.npr_api:
		return
	var body := DccWidgets.section(self, "Map style")
	var row := HFlowContainer.new()
	row.add_theme_constant_override("h_separation", 4)
	row.add_theme_constant_override("v_separation", 4)
	body.add_child(row)
	for i in STYLE_PRESETS.size():
		var chip := DccWidgets.segment(row, String(STYLE_PRESETS[i][0]),
			_apply_preset.bind(i))
		DccWidgets.set_segment_on(chip, i == 0)
		_preset_chips.append(chip)
	_custom_note = DccWidgets.note(body,
		"Custom -- controls edited since the last preset.")
	_custom_note.visible = false
	DccWidgets.note(body,
		"Each preset is an absolute bundle: every Painter style and overlay it "
		+ "manages goes back to off first, so Default reproduces this port's base "
		+ "look exactly. Parchment is left where the quality tier put it -- see "
		+ "this file's STYLE_PRESETS for why the reference's own number would "
		+ "reduce it. The reference's Antique also turns on a stylized "
		+ "mountain/hill/tree glyph layer, which this port has not built.")

func _apply_preset(index: int) -> void:
	var values: Dictionary = STYLE_MANAGED.duplicate()
	for key in Dictionary(STYLE_PRESETS[index][1]):
		values[key] = STYLE_PRESETS[index][1][key]
	_applying = true
	if bridge.set_npr(values) > 0:
		_refresh_map()
	for key in values:
		if not _npr_rows.has(key):
			continue
		var row: Dictionary = _npr_rows[key]
		if row.has("slider"):
			row["slider"].value = float(values[key])
		elif row.has("check"):
			row["check"].set_pressed_no_signal(bool(values[key]))
	_applying = false
	for i in _preset_chips.size():
		DccWidgets.set_segment_on(_preset_chips[i], i == index)
	if _custom_note != null:
		_custom_note.visible = false

## The reference flips the row to "Custom" on any manual edit inside the Map
## style section; here that means any Painter or appearance write that did not
## come from `_apply_preset`.
func _mark_custom() -> void:
	if _applying:
		return
	for chip in _preset_chips:
		DccWidgets.set_segment_on(chip, false)
	if _custom_note != null:
		_custom_note.visible = true

# -- The reference's Rendering-advanced block (reference HTML 1731-1751) -------

## Built from `list_appearance_tunables()` -- the engine's own key/range/label
## table -- rather than from a second copy of those ranges here, so a slider
## cannot offer a value `set_appearance` would clamp.
func _build_appearance() -> void:
	if not bridge.appearance_api:
		return
	var body := DccWidgets.section(self, "Rendering - advanced")
	for entry in APPEARANCE_GROUPS:
		var g := DccWidgets.group(body, String(entry[0]), false)
		for key in entry[1]:
			_appearance_slider(g, String(key))
	DccWidgets.action(body, "Reset to quality tier", func():
		if bridge.reset_appearance() > 0:
			_sync_appearance()
			_refresh_map()
			_mark_custom())
	DccWidgets.note(body,
		"These are the quality tier's own values (Preferences > Render quality), "
		+ "editable. An edit survives a later tier change; Reset hands every one "
		+ "of them back to the tier. All presentation -- nothing here marks a "
		+ "generation stage stale.")
	DccWidgets.note(body,
		"Not bound, because the engine has no such stage: ridge crests, slope "
		+ "rock, surface texture, minor channels, ridged relief, sky view factor, "
		+ "cast shadows, curvature shading, season blend and the three SDF "
		+ "layers (coastlines, river bands, biome blend). Those are reference "
		+ "render stages this port has not ported, not bindings it is missing.")

## One row, with the engine's range and this file's own step/unit/scale.
func _appearance_slider(parent: Control, key: String) -> void:
	var spec := _tunable(key)
	if spec.is_empty():
		return
	var ui: Dictionary = APPEARANCE_UI.get(key, {})
	var scale := float(ui.get("scale", 1.0))
	var step := float(ui.get("step", 0.01))
	var unit := String(ui.get("unit", ""))
	var value := float(bridge.appearance().get(key, spec["min"]))
	var pending := [value]
	var handle := DccWidgets.slider(parent, String(spec["label"]),
		float(spec["min"]) * scale, float(spec["max"]) * scale, step,
		value * scale, unit,
		func(v: float): pending[0] = v / scale,
		String(APPEARANCE_HELP.get(key, "")),
		func():
			if bridge.set_appearance({key: pending[0]}) > 0:
				_mark_custom()
				_refresh_map())
	_app_rows[key] = {"handle": handle, "scale": scale, "pending": pending}

## Pull every appearance row back from the engine -- after a Reset, where the
## engine's values changed without any slider moving.
func _sync_appearance() -> void:
	var live: Dictionary = bridge.appearance()
	for key in _app_rows:
		if not live.has(key):
			continue
		var row: Dictionary = _app_rows[key]
		row["pending"][0] = float(live[key])
		row["handle"]["slider"].value = float(live[key]) * float(row["scale"])

## `[key, min, max, label]` for one key, or `{}` if this cdylib has no such
## tunable.
func _tunable(key: String) -> Dictionary:
	for row in bridge.appearance_tunables():
		if String(row[0]) == key:
			return {"min": float(row[1]), "max": float(row[2]), "label": String(row[3])}
	return {}

# -- The reference's NPR block (`GUI_GAP_REGISTER.md` RN-01) -------------------

## The ten Painter styles plus the three toggles the reference keeps beside
## them, all live against `WorldGen::set_npr`.
##
## **Presentation only, and it says so.** None of these marks a generation
## stage stale: each one calls `app.viewport.refresh()`, which re-runs
## `build_color_texture()` over the world that is already there. That is why
## nothing here touches `bridge.mark_dirty()`.
##
## Sliders commit on release rather than on every value change: a full-map
## re-render at the app's own 2048x1311 is not a per-drag-pixel operation, and
## `DccWidgets.slider` already carries an `on_release` for exactly this.
func _build_npr() -> void:
	if not bridge.npr_api:
		return
	var npr: Dictionary = bridge.npr_settings()

	var body := DccWidgets.section(self, "Painter styles")
	DccWidgets.note(body,
		"Hand-drawn non-photorealistic styles, each with its own intensity. "
		+ "Land only, all default off -- stack them freely.")

	for entry in STYLES:
		_npr_slider(body, entry[1], entry[0], 1.0, 0.01, "", entry[2], npr)

	var adv := DccWidgets.advanced(body, "contour interval")
	_npr_slider(adv, "Interval", "contour_m", 50.0, 5.0, " m",
		"Metres between contour veins. 0 = the reference's own automatic "
		+ "interval (1/20th of the world's relief). Only affects Contour veins.",
		npr)

	var water := DccWidgets.section(self, "Water & light")
	_npr_rows["waves"] = {"check": DccWidgets.toggle(water, "Coastal wave lines",
		bool(npr.get("waves", false)),
		func(v: bool): _push({"waves": v}),
		"Foam contours hugging the shore and fading into deep water.")}
	## `wave_dist` 0 *is* 1× in the engine (the reference's own
	## `waveDist>0?waveDist:1`), so the row opens at 1.00× rather than showing a
	## 0.00× that would render as 1× anyway. The range is the reference's own.
	if float(npr.get("wave_dist", 0.0)) <= 0.0:
		npr["wave_dist"] = 1.0
	_npr_slider(water, "Wave reach", "wave_dist", 3.0, 0.05, "×",
		"How far the foam reaches offshore. 1× is the reference's own reach.",
		npr, 0.25)
	_anim_check = DccWidgets.toggle(water, "Animate water", bool(npr.get("animate_water", false)),
		_on_animate_water,
		"A travelling shimmer along river channels. Drawn as a shader overlay "
		+ "on the map, not baked into the raster -- see water_anim_layer.gd.")
	_npr_rows["multi_sun"] = {"check": DccWidgets.toggle(water, "Multi-sun lighting",
		bool(npr.get("multi_sun", false)),
		func(v: bool): _push({"multi_sun": v}),
		"The reference's softer four-light painterly relief: a primary sun, a "
		+ "fill sun 90° round, a zenith light and an ambient floor.")}

func _npr_slider(parent: Control, label_text: String, key: String, maximum: float,
		step: float, unit: String, tooltip: String, npr: Dictionary,
		minimum := 0.0) -> void:
	var pending := [float(npr.get(key, 0.0))]
	var handle := DccWidgets.slider(parent, label_text, minimum, maximum, step,
		pending[0], unit,
		func(v: float): pending[0] = v,
		tooltip,
		func(): _push({key: pending[0]}))
	_npr_rows[key] = {"slider": handle["slider"]}

## One changed key at a time -- `set_npr` treats every key as optional, so the
## panel never has to send (or hold) the whole block.
func _push(values: Dictionary) -> void:
	if bridge.set_npr(values) > 0:
		_mark_custom()
		_refresh_map()

## Re-render the raster over the world that is already there. Deliberately not
## `bridge.mark_dirty()`: nothing here invalidates a generation stage.
func _refresh_map() -> void:
	if app != null and app.viewport != null:
		app.viewport.refresh()

## Animated water is the one member of the block this renderer does not draw:
## it is per-frame, so it lives on its own shader overlay over the map. The
## layer is created the first time it is asked for and parented under the map
## overlay -- the same slot, and for the same reason, `wind_fx_layer.gd` uses.
func _on_animate_water(on: bool) -> void:
	## Straight to the engine, not through `_push`: the flag is carried with
	## the rest of the NPR block so it round-trips in one place, but nothing
	## in the raster reads it, so re-rendering the map would be pure waste.
	bridge.set_npr({"animate_water": on})
	if app == null or app.viewport == null or app.viewport.overlay == null:
		return
	if _water_anim == null:
		var existing: Node = app.viewport.overlay.get_node_or_null("WaterAnimLayer")
		if existing != null:
			_water_anim = existing
		elif on:
			_water_anim = WATER_ANIM_SCRIPT.new()
			_water_anim.setup(bridge)
			app.viewport.overlay.add_child(_water_anim)
	if _water_anim == null:
		return
	## `false` means this world carries no discharge field to animate. Untick
	## rather than leave a checkbox on over an effect that is not running --
	## the same honesty rule `wind_fx_layer.gd`'s own `_refused` follows.
	if not _water_anim.set_enabled(on):
		if _anim_check != null:
			_anim_check.set_pressed_no_signal(false)
		bridge.set_npr({"animate_water": false})
		push_warning("Animate water: this world has no river discharge field to animate.")

# -- What is still owed -------------------------------------------------------

## What the design still asks for beyond what is now live above. Enumerated
## rather than left implicit, so a reader of the dock can tell which of the
## designed ~60 controls landed and which did not.
## `design/Cartalith Menu Structure v2.dc.html` (MAP ▸ MAP STYLE and MAP ▸
## TERRAIN APPEARANCE) is the authoritative content list; `TERRAIN_APPEARANCE_
## SCOPE.md` carries the milestones. Prose, not disabled rows: a wall of
## disabled sliders would imply that many separate gaps, when what remains is a
## handful of real ones.
func _build_owed_inventory() -> void:
	var sec := DccWidgets.section(self, "Still owed")
	DccWidgets.note(sec,
		"Colour relief as an editable ramp: the gradient editor with draggable "
		+ "stops, add / delete / duplicate / reverse, per-stop elevation and "
		+ "hex/RGBA, interpolation mode, elevation domain, Auto Fit / Auto "
		+ "Breakpoints, and the 16 named ramp presets. render.rs holds colour as "
		+ "a 31-entry material palette, not an elevation-keyed breakpoint ramp, so "
		+ "this is a renderer change and not a binding (GUI_GAP_REGISTER.md CA-02).")
	DccWidgets.note(sec,
		"Colour grading (vibrancy, saturation, contrast, brightness, gamma, "
		+ "temperature, tint and the four field-influence weights) · Material "
		+ "exposure per class (vegetation / rock / soil and their slope, curvature "
		+ "and wetness modulation -- only the lithology pair is live above) · "
		+ "Detail & atmosphere (macro / meso / micro intensity, distance and "
		+ "elevation haze) · Preview (on-off, Compare current / previous / split).")
	DccWidgets.note(sec,
		"Saving a look: save preset / save as theme. TerrainAppearance does not "
		+ "derive Serialize, and nothing writes appearance into the project file "
		+ "yet, so an edited style is per-session (GUI_GAP_REGISTER.md CA-08).")
	DccWidgets.note(sec,
		"None of it is presentation-unsafe: every control here updates visible "
		+ "tiles only and never marks a generation stage stale.")

extends Workspace
class_name RenderWorkspace

## Terrain appearance groups; right dock preview & quality -- formerly the
## standalone RENDER rail domain (§3).
##
## `render.rs` carries a full `TerrainAppearance` structure that is settable
## in Rust and bound to nothing -- its own doc comment says so. Until it is
## bound, quality tier (Preferences) is the only live control **for the
## colour/relief half**; the reference's non-photorealistic half (the ten
## "Painter" styles, the coastal wave lines, animated water and multi-sun) is
## live here as of the NPR pass -- see `_build_npr` below.
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

var _water_anim: Control
var _anim_check: CheckBox

func _build() -> void:
	## §4.5.1's three global tools -- RENDER owns no tool of its own (no
	## §4.5.x section names one), so this was always the whole TOOLS block.
	## Skipped when nested -- see this file's own class doc.
	if not _nested:
		DccWidgets.tools_block(self, app, app.tool_group)
	_build_npr()
	_not_built("Terrain appearance",
		"render.rs's TerrainAppearance is real but unbound to Godot; until it is, Preferences ▸ Render quality is the live control.")
	_build_owed_inventory()

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
	DccWidgets.toggle(water, "Coastal wave lines", bool(npr.get("waves", false)),
		func(v: bool): _push({"waves": v}),
		"Foam contours hugging the shore and fading into deep water.")
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
	DccWidgets.toggle(water, "Multi-sun lighting", bool(npr.get("multi_sun", false)),
		func(v: bool): _push({"multi_sun": v}),
		"The reference's softer four-light painterly relief: a primary sun, a "
		+ "fill sun 90° round, a zenith light and an ambient floor.")

func _npr_slider(parent: Control, label_text: String, key: String, maximum: float,
		step: float, unit: String, tooltip: String, npr: Dictionary,
		minimum := 0.0) -> void:
	var pending := [float(npr.get(key, 0.0))]
	DccWidgets.slider(parent, label_text, minimum, maximum, step, pending[0], unit,
		func(v: float): pending[0] = v,
		tooltip,
		func(): _push({key: pending[0]}))

## One changed key at a time -- `set_npr` treats every key as optional, so the
## panel never has to send (or hold) the whole block.
func _push(values: Dictionary) -> void:
	if bridge.set_npr(values) > 0:
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

## What the one umbrella note above actually covers, enumerated rather than
## left as "terrain appearance" -- a reader of the dock could not otherwise
## learn that ~60 designed controls sit behind that single sentence.
## `design/Cartalith Menu Structure v2.dc.html` (MAP ▸ MAP STYLE and MAP ▸
## TERRAIN APPEARANCE) is the authoritative content list; `TERRAIN_APPEARANCE_
## SCOPE.md` carries the milestones. Prose, not disabled rows: sixty disabled
## sliders would be clutter implying sixty separate gaps, when there is one --
## the missing `set_appearance()`-shaped #[func].
func _build_owed_inventory() -> void:
	var sec := DccWidgets.section(self, "What that covers")
	DccWidgets.note(sec,
		"Preset (16 named ramps, save preset / save as theme / reset) · Colour relief "
		+ "(gradient editor with draggable stops, add/delete/duplicate/reverse, "
		+ "per-stop elevation + hex/RGBA, interpolation mode, elevation domain, "
		+ "Auto Fit / Auto Breakpoints) · Colour (vibrancy, saturation, contrast, "
		+ "brightness, gamma, temperature, tint, and the four field-influence "
		+ "weights) · Material (vegetation / rock / soil exposure and their slope, "
		+ "curvature and wetness modulation).")
	DccWidgets.note(sec,
		"Relief (multidirectional 8-light hillshade, strength / directionality / "
		+ "softness, the light rig, AO strength / radius / contrast, slope, "
		+ "curvature and local contrast) · Detail & atmosphere (macro / meso / "
		+ "micro intensity, distance and elevation haze) · Preview & quality "
		+ "(preview on-off, Compare current / previous / split, the four quality "
		+ "tiers -- of which only the tiers are live, in Preferences).")
	DccWidgets.note(sec,
		"And the reference's own Rendering-advanced block, which folds into this "
		+ "same subsystem rather than staying a separate menu: parchment, surface "
		+ "texture, sky view factor, ridge crests, ridged relief, slope rock, "
		+ "geology materials, cast shadows, curvature shading, minor channels, "
		+ "wetness, season, SDF coastlines / river bands / biome blend, and "
		+ "stylized icons. (The Painter styles, coastal wave lines, animated "
		+ "water and multi-sun lighting that used to be listed here are built "
		+ "-- see above.)")
	DccWidgets.note(sec,
		"None of it is presentation-unsafe: every control here updates visible "
		+ "tiles only and never marks a generation stage stale.")

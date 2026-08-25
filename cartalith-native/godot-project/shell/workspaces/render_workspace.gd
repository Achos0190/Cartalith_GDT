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
## **The elevation colour ramp and saved looks** (2026-08-24, CA-02 and CA-08)
## are `_build_ramp` and `_build_look_presets`. Both needed a renderer change
## rather than a binding -- there was no breakpoint ramp in `render.rs` at all,
## and `TerrainAppearance` did not derive `Serialize` -- which is why they
## trailed the sliders by a commit.
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

## **Where this workspace's sections attach** (2026-08-24, `design/Cartalith
## Menu Structure v3.dc.html`).
##
## v3 splits CARTO into ten named L2 categories, and four of them are this
## file's content: Map style (map view + presets + painter styles), Terrain
## appearance (the ramp and the ground/relief groups), Colours (the grade and
## its field weights) and Map presets (saved looks). Before v3 all of it was
## one flat run of L3 sections appended below CARTO's own categories, which is
## what `_nested` used to mean.
##
## Rather than move the builders into `cartography_workspace.gd` -- they carry
## the ramp editor, the preset table and every appearance sync in this file --
## each one now draws into `_h()` instead of `self`, and CARTO calls the four
## `build_*_into()` entry points below with its own category bodies. **The
## builders are unchanged**; only where they attach moved. Standalone
## (non-nested) use is untouched: `_host` stays null and `_h()` returns `self`.
var _host: Control = null

func _h() -> Control:
	return _host if _host != null else self

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
##
## **2026-08-24: each preset now also names a *look*.** A look
## (`WorldGen::list_looks`) is the engine's colour/chroma/light-shaping/grade
## layer over the quality tier; the Painter dictionary beside it is the NPR
## block as before. Splitting the two is what lets "Ink" put pen lines over the
## shipped vibrant base rather than over the reference's muted one, and what
## makes "Default" mean the tier's own image rather than "vibrant minus the
## styles". A preset whose look this cdylib does not have simply keeps the look
## that is already selected -- `EngineBridge.set_look` returns false and the
## Painter half still applies.
const STYLE_PRESETS := [
	["Natural Vibrant", "Natural Vibrant", {"multi_sun": true}],
	["Default", "Quality tier", {}],
	["Antique", "Antique Parchment", {"sepia": 0.35, "multi_sun": true}],
	["Ink", "Natural Vibrant", {"ink": 0.6, "contours": 0.35, "multi_sun": true}],
	["Watercolor", "Natural Vibrant", {"watercolor": 0.65, "multi_sun": true}],
	["Print", "Natural Vibrant", {"risograph": 0.5, "contours": 0.25}],
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
		"relief_gain", "relief_chroma", "ao_strength", "ao_radius_frac",
		"crest_strength", "curve_shade", "ridged_strength"]],
	["The sheet", ["paper_strength", "paper_grain", "paper_mottle", "paper_wash",
		"stipple_strength", "border_width_frac"]],
	["Materials", ["biome_sat", "tex_strength", "litho_strength", "litho_exposure",
		"hydro_wet_strength", "local_contrast", "splat_strength"]],
	["Atmosphere", ["haze_strength"]],
	## The colour grade -- a presentation-only post-process over the finished
	## terrain raster, before rivers, labels and icons draw. Its own group
	## because it is a different kind of control from everything above it:
	## nothing here describes the ground, it describes the print.
	["Colour grade", ["grade_exposure", "grade_gamma", "grade_contrast",
		"grade_saturation", "grade_temperature", "grade_shadow_tint",
		"grade_highlight_tint"]],
	## The four field-influence weights, in their own group because they are
	## not axes: each one scales the grade above by an underlying field, so all
	## four are inert while every slider in "Colour grade" sits at zero. The
	## design draws them nested under COLOUR for the same reason
	## (`design/Cartalith Menu Structure v2.dc.html`, "+ Field influence
	## weights"); this shell has no nesting inside a group, so a named group
	## adjacent to the grade is the closest honest arrangement.
	["Grade field influence", ["grade_field_biome", "grade_field_elevation",
		"grade_field_moisture", "grade_field_geology"]],
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
	"hydro_wet_strength": "Wetness: darkens and cools persistently saturated ground near channels. The reference's own Wetness slider. Gated on real upstream drainage area, so it marks the same rivers whatever the map's resolution (GUI_GAP_REGISTER.md CA-11 -- it used to fade out as the grid got finer, and at 2048 wide it moved nothing at all).",
	"ramp_strength": "How far the colour relief ramp takes over from the material colour. 0 is the material model alone (climate, slope, relief); 1 is a full hypsometric tint. The ramp is applied before the light, so the hillshade, occlusion and paper still read through it at any strength.",
	"local_contrast": "Adds band-limited detail back after the paper wash. The gain falls to zero on strong edges, so coastlines and snowlines cannot halo.",
	"splat_strength": "How strongly a loaded asset pack's ground textures blend in. Inert with no pack loaded. The reference's Texture strength.",
	"relief_chroma": "How far the relief lighting keeps the map's colour instead of fading it toward grey. 0 is the reference exactly -- shaded ground is pulled toward one fixed neutral, which costs value as well as chroma. At 1 the shading desaturates about each pixel's own luminance, and shadow cools while sunlight warms, the way a real scene's sky-lit shadow and warm sun differ.",
	"crest_strength": "Thin bright strokes along convex, steep ridge lines -- the reference's Ridge crests. Costs one whole-grid pass when on and nothing when off.",
	"curve_shade": "Sun-independent lighting straight from the surface curvature: convex ridges brighten, concave valleys darken. Keeps a landform legible where it happens to run parallel to the sun. The reference's Curvature shading.",
	"ridged_strength": "Folded creases from a ridged multifractal, weighted by elevation squared so they concentrate in the highlands and leave the lowlands alone. The reference's Ridged relief.",
	"biome_sat": "How colourful the material mix is, about its own luminance -- so it can never make one material lighter or darker relative to its neighbour, only more or less saturated. Negative is toward grey. No reference counterpart: the reference's only chroma control is Relief <-> biome, which pulls toward a fixed grey and therefore flattens as it desaturates.",
	"tex_strength": "A three-frequency fine surface modulation over the material colour -- the reference's Surface texture. Evaluated in map coordinates, so a tiled export stays seamless.",
	"haze_strength": "Atmospheric perspective: how far the plate fades toward sky at its edges. The reference's own fixed 0.18, made adjustable; the shipped look uses 0.09, which reads as air rather than as a vignette.",
	"grade_exposure": "Overall brightness of the finished map, as a linear gain. The grade is a post-process on the rendered image and touches no world data at all.",
	"grade_contrast": "Contrast about mid-grey, applied after exposure so the pivot sits where exposure put the image.",
	"grade_saturation": "Saturation of the finished map, about its own luminance -- exactly luminance-preserving, so it cannot flatten the relief.",
	"grade_temperature": "Colour temperature on a blue-to-amber axis. Luminance-compensated: warming the map does not also brighten it.",
	"grade_shadow_tint": "The same blue-to-amber axis, weighted toward the dark half of the image only.",
	"grade_highlight_tint": "The same blue-to-amber axis, weighted toward the bright half of the image only.",
	"grade_gamma": "Midtone bend, as a symmetric power curve applied straight after exposure. Pure black and pure white do not move; positive lifts the midtones, negative sinks them, and +k then -k returns the original image.",
	"grade_field_biome": "How far the grade above follows biome vegetation cover -- bare ice and desert least, closed forest most. Negative reverses it. Does nothing while the whole grade is at rest.",
	"grade_field_elevation": "How far the grade above follows relative land elevation, from 0 at the coast to 1 at the highest ground. Water takes the low end. Does nothing while the whole grade is at rest.",
	"grade_field_moisture": "How far the grade above follows the rainfall field. Does nothing while the whole grade is at rest.",
	"grade_field_geology": "How far the grade above follows the lightness of the rock underneath, in the current lithology palette -- dark basalt and pale limestone at opposite ends. Inert on a loaded save, which stores no lithology, and while the whole grade is at rest.",
}

var _water_anim: Control
var _anim_check: CheckBox

## key -> the `DccWidgets.slider` handle, so a preset can move the control the
## user is looking at rather than only the value behind it.
var _npr_rows: Dictionary = {}
var _app_rows: Dictionary = {}
var _preset_chips: Array[Button] = []
var _custom_note: Label
## The base-look picker, kept so a Map-style chip can move it rather than
## leaving it naming a look that is not the one drawing the map.
var _look_pick: OptionButton
var _look_names: Array = []
## True only while `_apply_preset` runs, so the preset's own writes do not
## trip the "Custom" mark the reference flips on any manual edit.
var _applying := false

func _build() -> void:
	## Nested under CARTO, this node draws nothing itself: it holds the state
	## (the ramp, the preset chips, the appearance rows, the water-anim layer)
	## while `cartography_workspace.gd` calls the four `build_*_into()` entry
	## points below with v3's own category bodies. See `_host`.
	if _nested:
		return
	## §4.5.1's three global tools -- RENDER owns no tool of its own (no
	## §4.5.x section names one), so this was always the whole TOOLS block.
	DccWidgets.tools_block(self, app, app.tool_group)
	_build_map_view()
	_build_map_style()
	_build_ramp()
	_build_appearance(APPEARANCE_GROUPS)
	_build_look_presets()
	_build_npr()
	_build_owed_inventory()

# -- v3 entry points ----------------------------------------------------------
#
# One per CARTO category that draws this file's content. Each sets `_host`,
# runs the unchanged builders, and hands `_host` back so a later standalone
# call still draws into `self`.

## v3 CARTO ▸ MAP STYLE: style preset · mode · relief↔biome mix · sun, then
## `+ Painter styles (NPR)`.
func build_map_style_into(parent: Control) -> void:
	_host = parent
	_build_map_style()
	_build_map_view()
	_build_npr()
	_host = null

## v3 CARTO ▸ TERRAIN APPEARANCE: `§ Colour relief ramp`, then the relief,
## sheet, material and atmosphere groups. v3 keeps this category "whole,
## unchanged in scope" -- its migration audit's own words -- so the split
## below is only the grade leaving for COLOURS, which v3 does ask for.
func build_terrain_appearance_into(parent: Control) -> void:
	_host = parent
	_build_ramp()
	_build_appearance(APPEARANCE_GROUPS.slice(0, 4))
	_host = null

## v3 CARTO ▸ COLOURS: vibrancy/saturation/contrast/brightness/gamma/temp/tint
## (the colour grade) and `+ Field influence weights`.
func build_colours_into(parent: Control) -> void:
	_host = parent
	_build_appearance(APPEARANCE_GROUPS.slice(4), "Colour grade")
	_host = null

## v3 CARTO ▸ MAP PRESETS: the saved-look library, plus the inventory of what
## this dock still owes.
func build_presets_into(parent: Control) -> void:
	_host = parent
	_build_look_presets()
	_build_owed_inventory()
	_host = null

# -- The reference's Map view block (reference HTML 1706-1717) -----------------

## The reference's four Map-view rows, minus its `modeSeg` (Biome / Relief /
## Height / Shade) and `shadeOnHypso`: those select a *base render mode*, and
## this port resolves base-mode switching through the Layers popover's own
## view list (`layers_popover.gd`'s own note on the split) rather than a second
## competing picker. Everything else here is one-to-one.
func _build_map_view() -> void:
	if not bridge.appearance_api:
		return
	var body := DccWidgets.section(_h(), "Map view")
	for key in APPEARANCE_VIEW:
		_appearance_slider(body, key)

# -- The reference's Map style presets (reference HTML 1719-1729) --------------

func _build_map_style() -> void:
	if not bridge.npr_api:
		return
	var body := DccWidgets.section(_h(), "Map style")
	var row := HFlowContainer.new()
	row.add_theme_constant_override("h_separation", 4)
	row.add_theme_constant_override("v_separation", 4)
	body.add_child(row)
	## Which chip opens lit is read from the engine, not assumed: the shipped
	## default is Natural Vibrant and a hard-coded `i == 0` would go on lying
	## the day that changes.
	var live_look := bridge.look()
	## The first chip whose look matches. On a fresh session that is Natural
	## Vibrant, which is exactly what is on screen; `0` is the fallback for a
	## cdylib with no look API at all, where the old "Default is lit" behaviour
	## is still the truthful one.
	var lit := 1 if not bridge.look_api else 0
	for i in STYLE_PRESETS.size():
		if String(STYLE_PRESETS[i][1]) == live_look:
			lit = i
			break
	for i in STYLE_PRESETS.size():
		var chip := DccWidgets.segment(row, String(STYLE_PRESETS[i][0]),
			_apply_preset.bind(i))
		DccWidgets.set_segment_on(chip, i == lit)
		_preset_chips.append(chip)

	## The look on its own, because it is the engine's own list and a user may
	## want a base without a Painter bundle over it.
	var look_names: Array = bridge.looks()
	_look_names = look_names
	if not look_names.is_empty():
		_look_pick = DccWidgets.choice(body, "Base look", look_names,
			maxi(look_names.find(live_look), 0),
			func(i: int): _on_look(String(look_names[i])),
			"The colour, chroma, light shaping and grade the map is built on, "
			+ "layered over the quality tier -- the tier decides what the "
			+ "renderer spends, the look decides what the picture is, and a "
			+ "phone answers only the first differently. Quality tier is the "
			+ "identity. Natural Vibrant is the shipped default. Changing it "
			+ "moves the Rendering-advanced values below, since that is where "
			+ "they come from.")
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
	for key in Dictionary(STYLE_PRESETS[index][2]):
		values[key] = STYLE_PRESETS[index][2][key]
	_applying = true
	## The look first, so the one re-render below carries both halves. It moves
	## the Rendering-advanced values too (a look is where ambient occlusion,
	## wetness, haze and the grade come from), so those rows are pulled back
	## from the engine rather than left showing the previous look's numbers --
	## the desync this register keeps finding one control at a time.
	bridge.set_look(String(STYLE_PRESETS[index][1]))
	_sync_look_pick()
	bridge.set_npr(values)
	_sync_appearance()
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
	## `GUI_GAP_REGISTER.md`'s top-right-readout note, now taken: the viewport's
	## own chrome has no idea which style preset is active, so this is pushed in
	## rather than polled -- `ViewportHost.set_style_readout()`'s own comment.
	if app != null and app.viewport != null:
		app.viewport.set_style_readout(String(STYLE_PRESETS[index][0]))

## Pick a base look on its own. Marks Custom, because the chips above name a
## look *and* a Painter bundle and only one half moved.
func _on_look(name: String) -> void:
	if not bridge.set_look(name):
		return
	_sync_appearance()
	_refresh_map()
	_mark_custom()

## Re-select the picker after a Map-style chip changed the look underneath it.
func _sync_look_pick() -> void:
	if _look_pick == null:
		return
	var i: int = _look_names.find(bridge.look())
	if i >= 0 and _look_pick.selected != i:
		_look_pick.select(i)

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
	if app != null and app.viewport != null:
		app.viewport.set_style_readout("Custom")

# -- The reference's Rendering-advanced block (reference HTML 1731-1751) -------

## Built from `list_appearance_tunables()` -- the engine's own key/range/label
## table -- rather than from a second copy of those ranges here, so a slider
## cannot offer a value `set_appearance` would clamp.
## `groups` is a slice of `APPEARANCE_GROUPS`: v3 draws the first four under
## CARTO ▸ Terrain appearance and the last two under CARTO ▸ Colours, so the
## section takes its title and its Reset scope from whichever half it is. The
## Reset button itself is engine-wide (`reset_appearance()` hands *every*
## tunable back), so it is drawn once, with the terrain half.
func _build_appearance(groups: Array, title: String = "Rendering - advanced") -> void:
	if not bridge.appearance_api:
		return
	var body := DccWidgets.section(_h(), title)
	for entry in groups:
		var g := DccWidgets.group(body, String(entry[0]), false)
		for key in entry[1]:
			_appearance_slider(g, String(key))
	if title != "Colour grade":
		DccWidgets.action(body, "Reset to quality tier", func():
			if bridge.reset_appearance() > 0:
				_sync_appearance()
				_refresh_map()
				_mark_custom())
		DccWidgets.note(body,
			"These are the quality tier's own values (Preferences > Render quality) "
			+ "as the base look above reshapes them, editable. An edit survives a "
			+ "later tier or look change; Reset hands every one of them back -- "
			+ "including the colour grade under Colours, which is the same "
			+ "appearance record. Reset "
			+ "deliberately leaves the base look alone -- that picker is above, and "
			+ "a button in this section silently moving it is the desync this dock "
			+ "keeps having to fix. All presentation -- nothing here marks a "
			+ "generation stage stale.")
		DccWidgets.note(body,
			"Not bound, because the engine has no such stage: slope rock, minor "
			+ "channels, sky view factor, cast shadows, season blend and the three "
			+ "SDF layers (coastlines, river bands, biome blend). Those are "
			+ "reference render stages this port has not ported, not bindings it is "
			+ "missing. Ridge crests, surface texture, ridged relief and curvature "
			+ "shading left this list on 2026-08-24 and are live above.")
	else:
		DccWidgets.note(body,
			"A post-process over the finished terrain raster, before rivers, labels "
			+ "and icons draw -- nothing here describes the ground, it describes the "
			+ "print. Every weight in Grade field influence is inert while every "
			+ "slider above it sits at rest. Reset to quality tier, under Terrain "
			+ "appearance, hands these back too: it is one appearance record.")
		## `GUI_GAP_REGISTER.md` **CA-19**, corrected 2026-08-25: the table has
		## been *readable* all along -- `debug_layers()` carries all fifteen
		## classes, name and swatch, as the Biomes field's own legend, and the
		## paint palette reads the same constant. What it is not is *writable*.
		var n := 0
		for g in bridge.debug_layers():
			for it in (g as Dictionary).get("items", []):
				if String((it as Dictionary).get("id", "")) == "bclass":
					n = ((it as Dictionary).get("legend", []) as Array).size()
		var see := DccWidgets.action(body,
			"Biome colour table (%d classes) → Layers ▸ Biomes" % n,
			func():
				app.viewport.set_debug_layer("bclass")
				app.layers_popover.open())
		see.alignment = HORIZONTAL_ALIGNMENT_LEFT
		see.tooltip_text = "CART_BIOME_COLS, the reference's own fifteen-class table, rendered as the Biomes field's legend -- one picker, not a second copy of the list."
		DccWidgets.note(body,
			"A writable biome table  ·  costs a re-baseline\n"
			+ "All fifteen classes are readable today; making one writable is what "
			+ "is not built (GUI_GAP_REGISTER.md CA-19). It is a frozen "
			+ "reference constant compiled into render.rs, which five test targets "
			+ "include standalone, and it is what a painted biome cell blends "
			+ "toward -- so a rewritable palette is a field threaded through "
			+ "RenderCtx and re-baselined golden expectations, not a picker. The "
			+ "four field weights above are the influence half of v3's category "
			+ "and are live.")

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

# -- Colour relief: the elevation ramp (`GUI_GAP_REGISTER.md` CA-02) -----------

## `DCC_SHELL_SPEC.md` §7's Colour ramp popover + Stop editor, in the left dock
## rather than split across a popover and the right dock. The split the spec
## draws is a three-pane style editor this shell does not have; folding the two
## into one block loses nothing (the popover is a list of named ramps and the
## stop editor is the list of stops) and costs no navigation.
##
## What is here: the nine named ramps, the three interpolation modes, a live
## gradient bar, one row per stop with its colour, its elevation, its alpha and
## a delete, plus Add stop and Reverse, and the strength slider that blends the
## whole thing against the material colour.
##
## **2026-08-24: the two axes CA-02 shipped without.** Its own note here used to
## say per-stop alpha and Linear/Ease/Step "are not built"; both are, and both
## needed `render.rs` rather than a binding. The mode is the **ramp's**, not a
## stop's -- `DCC_SHELL_SPEC.md` §7 draws one picker above the stop list, and
## "banded" is a statement about the whole plate.
var _ramp: Array = []
var _ramp_host: VBoxContainer
var _ramp_bar: TextureRect
var _ramp_gradient: Gradient
## The engine's own mode name, cached so `_update_ramp_bar` (which runs on every
## drag frame) is not an engine call per frame.
var _ramp_mode := ""
var _ramp_modes: Array = []
var _ramp_mode_pick: OptionButton

func _build_ramp() -> void:
	if not bridge.ramp_api:
		return
	var body := DccWidgets.section(_h(), "Colour relief")
	_appearance_slider(body, "ramp_strength")

	var names: Array = bridge.ramp_presets()
	if not names.is_empty():
		DccWidgets.choice(body, "Ramp", names, -1,
			func(i: int): _on_ramp_preset(String(names[i])),
			"Named elevation ramps. Picking one replaces every stop below; edit "
			+ "from there and the picker no longer describes what is on screen.")

	_ramp_modes = bridge.ramp_modes()
	var modes: Array = _ramp_modes
	if not modes.is_empty():
		_ramp_mode_pick = DccWidgets.choice(body, "Blend", modes, maxi(modes.find(bridge.ramp_mode()), 0),
			func(i: int): _on_ramp_mode(String(modes[i])),
			"How the colour crosses from one stop to the next. Linear is a "
			+ "straight blend. Ease flattens the ramp at each stop and puts the "
			+ "change in the middle of the interval, so the ramp reads as broad "
			+ "bands with soft joins. Step draws flat bands with a hard edge on "
			+ "each stop -- the classic banded hypsometric plate. This belongs "
			+ "to the ramp rather than to a stop, and survives every stop edit.")

	_ramp_gradient = Gradient.new()
	var tex := GradientTexture1D.new()
	tex.gradient = _ramp_gradient
	tex.width = 256
	_ramp_bar = TextureRect.new()
	_ramp_bar.texture = tex
	_ramp_bar.stretch_mode = TextureRect.STRETCH_SCALE
	_ramp_bar.custom_minimum_size.y = 16
	_ramp_bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(_ramp_bar)

	_ramp_host = VBoxContainer.new()
	_ramp_host.add_theme_constant_override("separation", 2)
	body.add_child(_ramp_host)

	DccWidgets.action(body, "Add stop", _on_add_stop)
	DccWidgets.action(body, "Reverse", _on_reverse_ramp)
	DccWidgets.note(body,
		"Stops are keyed to relative land elevation -- 0 at the shoreline, 1 at "
		+ "the world's highest point -- so a saved ramp means the same picture on "
		+ "a world with a different peak. The metre readout is that fraction of "
		+ "this world's own relief. Order is the position: drag a stop past its "
		+ "neighbour and it takes its place.")
	DccWidgets.note(body,
		"Blending is flat beyond the end stops in every mode -- there is no "
		+ "neighbour out there to blend towards. A stop's alpha is how far that "
		+ "band takes part: it multiplies the strength above, so 0 leaves the "
		+ "material colour showing through at that elevation whatever the "
		+ "slider says. Water is untouched -- the sea has its own depth ramp.")
	_sync_ramp()

## Pull the engine's ramp and rebuild both the bar and the rows.
func _sync_ramp() -> void:
	_ramp.clear()
	## The `Color`'s alpha is the stop's own opacity, not a placeholder -- see
	## `EngineBridge.color_ramp()`.
	for row in bridge.color_ramp():
		_ramp.append([float(row[0]), row[1] as Color])
	_ramp_mode = bridge.ramp_mode()
	## Re-select the picker, or a look loaded with a different mode would leave
	## the row naming a mode that is not the one drawing the map -- the exact
	## failure this register keeps finding, one control at a time.
	if _ramp_mode_pick != null:
		var mi: int = _ramp_modes.find(_ramp_mode)
		if mi >= 0 and _ramp_mode_pick.selected != mi:
			_ramp_mode_pick.select(mi)
	_rebuild_ramp_rows()
	_update_ramp_bar()

func _on_ramp_mode(name: String) -> void:
	if not bridge.set_ramp_mode(name):
		return
	_ramp_mode = name
	_update_ramp_bar()
	_mark_custom()
	_refresh_map()

## The gradient bar, straight from `_ramp` -- redrawn on every value change so
## dragging a stop shows where it is going, even though the engine is only told
## on release.
func _update_ramp_bar() -> void:
	if _ramp_gradient == null:
		return
	var offsets := PackedFloat32Array()
	var colors := PackedColorArray()
	var sorted := _ramp.duplicate()
	sorted.sort_custom(func(a, b): return float(a[0]) < float(b[0]))
	for s in sorted:
		offsets.append(clampf(float(s[0]), 0.0, 1.0))
		colors.append(s[1])
	if offsets.is_empty():
		return
	## `Gradient` rejects an empty pair and duplicates confuse its own
	## interpolation, so a one-stop ramp is drawn as a flat two-stop one.
	if offsets.size() == 1:
		offsets.append(1.0)
		colors.append(colors[0])
	## Step is exact here; **Ease is an approximation**, and saying so is
	## cheaper than a hand-baked 256-sample texture. `Gradient` offers cubic,
	## not smoothstep, so the bar shows an eased join where the renderer's own
	## `k^2(3-2k)` is a slightly different curve. The bar is a preview of the
	## ramp, and `render.rs` remains the thing that draws the map.
	match _ramp_mode:
		"Step":
			_ramp_gradient.interpolation_mode = Gradient.GRADIENT_INTERPOLATE_CONSTANT
		"Ease":
			_ramp_gradient.interpolation_mode = Gradient.GRADIENT_INTERPOLATE_CUBIC
		_:
			_ramp_gradient.interpolation_mode = Gradient.GRADIENT_INTERPOLATE_LINEAR
	_ramp_gradient.offsets = offsets
	_ramp_gradient.colors = colors

## Metres above sea level a relative position stands for on *this* world.
## `0` before the parameter table is read, which reads as `0 m` on every row --
## honest, and better than a metre figure derived from a peak nobody has yet.
func _ramp_metres(at: float) -> float:
	if not bridge.params_available():
		return 0.0
	var peak = bridge.param_get("peak_m")
	return at * (float(peak) if peak != null else 0.0)

func _rebuild_ramp_rows() -> void:
	if _ramp_host == null:
		return
	for c in _ramp_host.get_children():
		c.queue_free()
	for i in _ramp.size():
		## Captured by value into every lambda in this row, so the closures keep
		## pointing at their own stop rather than at the loop's last one.
		var idx: int = i
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 4)
		_ramp_host.add_child(row)

		var swatch := ColorPickerButton.new()
		## Shown opaque on purpose: this control owns the **hue**, the slider
		## further along the row owns the alpha, and a swatch drawn at 20%
		## would read as a colour choice nobody made.
		var stop_col := Color(_ramp[idx][1])
		swatch.color = Color(stop_col.r, stop_col.g, stop_col.b, 1.0)
		swatch.custom_minimum_size = Vector2(30, 18)
		swatch.focus_mode = Control.FOCUS_NONE
		swatch.edit_alpha = false
		swatch.color_changed.connect(func(c: Color):
			## `edit_alpha = false` makes the picker emit an **opaque** colour,
			## so the stop's own alpha has to be carried across by hand -- take
			## `c` whole and every colour edit silently resets the alpha to 1.
			_ramp[idx][1] = Color(c.r, c.g, c.b, Color(_ramp[idx][1]).a)
			_update_ramp_bar()
			_push_ramp(false))
		row.add_child(swatch)

		var pos := HSlider.new()
		pos.min_value = 0.0
		pos.max_value = 1.0
		pos.step = 0.005
		pos.value = float(_ramp[idx][0])
		pos.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		pos.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		pos.custom_minimum_size.y = 14
		pos.focus_mode = Control.FOCUS_NONE
		row.add_child(pos)

		var readout := DccTheme.mono_label("", "text", DccTheme.FS_SMALL, 0)
		readout.custom_minimum_size.x = 62
		readout.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		readout.text = "%d m" % int(round(_ramp_metres(float(_ramp[idx][0]))))
		row.add_child(readout)

		pos.value_changed.connect(func(v: float):
			_ramp[idx][0] = v
			readout.text = "%d m" % int(round(_ramp_metres(v)))
			_update_ramp_bar())
		## Commit on release, like every other row in this dock: a full-map
		## re-render is not a per-drag-pixel operation.
		pos.drag_ended.connect(func(_changed: bool): _push_ramp(true))

		var alpha := HSlider.new()
		alpha.min_value = 0.0
		alpha.max_value = 1.0
		alpha.step = 0.01
		alpha.value = stop_col.a
		alpha.custom_minimum_size = Vector2(48, 14)
		alpha.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		alpha.focus_mode = Control.FOCUS_NONE
		alpha.tooltip_text = "Alpha -- how far this stop takes part, multiplied " \
			+ "into the strength above. 0 leaves the material colour showing " \
			+ "through at this elevation; it interpolates towards its " \
			+ "neighbours exactly as the colour does."
		row.add_child(alpha)
		alpha.value_changed.connect(func(v: float):
			var c := Color(_ramp[idx][1])
			c.a = v
			_ramp[idx][1] = c
			_update_ramp_bar())
		## `false`: alpha cannot reorder anything, so re-reading the engine
		## would rebuild these rows out from under the slider being dragged.
		alpha.drag_ended.connect(func(_changed: bool): _push_ramp(false))

		var del := DccWidgets.text_button(row, "x", func(): _delete_stop(idx))
		del.tooltip_text = "Delete this stop"
		## A one-stop ramp is legal; a zero-stop one is not (the engine refuses
		## it), so the last delete is disabled rather than silently ignored.
		del.disabled = _ramp.size() <= 1

func _delete_stop(idx: int) -> void:
	if _ramp.size() <= 1 or idx < 0 or idx >= _ramp.size():
		return
	_ramp.remove_at(idx)
	_push_ramp(true)

## A new stop in the widest gap, coloured with what the ramp already shows
## there -- so adding one changes nothing until it is moved, which is what
## makes it an edit rather than a surprise.
func _on_add_stop() -> void:
	var sorted := _ramp.duplicate()
	sorted.sort_custom(func(a, b): return float(a[0]) < float(b[0]))
	var at := 0.5
	var col := Color(0.5, 0.5, 0.5)
	if sorted.size() >= 2:
		var best := -1.0
		for i in sorted.size() - 1:
			var gap: float = float(sorted[i + 1][0]) - float(sorted[i][0])
			if gap > best:
				best = gap
				at = (float(sorted[i][0]) + float(sorted[i + 1][0])) * 0.5
				col = Color(sorted[i][1]).lerp(Color(sorted[i + 1][1]), 0.5)
	elif sorted.size() == 1:
		at = clampf(float(sorted[0][0]) + 0.25, 0.0, 1.0)
		col = sorted[0][1]
	_ramp.append([at, col])
	_push_ramp(true)

## The design's own Reverse: the same colours, top to bottom.
func _on_reverse_ramp() -> void:
	for s in _ramp:
		s[0] = 1.0 - float(s[0])
	_push_ramp(true)

func _on_ramp_preset(name: String) -> void:
	if bridge.load_ramp_preset(name):
		_sync_ramp()
		_mark_custom()
		_refresh_map()

## Send the whole list; the engine sorts it, so this is add, delete and reorder
## in one call. `rebuild` re-reads the engine afterwards, which is what makes a
## reorder show up in the rows -- skipped on a colour change, where the rows are
## already right and rebuilding would close the colour picker the user is in.
func _push_ramp(rebuild: bool) -> void:
	if bridge.set_color_ramp(_ramp) <= 0:
		return
	_mark_custom()
	_refresh_map()
	if rebuild:
		_sync_ramp()

# -- Saving a look (`GUI_GAP_REGISTER.md` CA-08) -------------------------------

var _preset_name: LineEdit
var _preset_pick: OptionButton
var _preset_slugs: Array = []

func _build_look_presets() -> void:
	if not bridge.preset_api:
		return
	var body := DccWidgets.section(_h(), "Saved looks")
	_preset_name = LineEdit.new()
	_preset_name.placeholder_text = "Name this look"
	_preset_name.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	DccWidgets.well(_preset_name)
	body.add_child(_preset_name)
	DccWidgets.action(body, "Save look", _on_save_look, true)
	_preset_pick = DccWidgets.choice(body, "Saved", [], -1, func(_i: int): pass,
		"Looks saved on this machine.")
	DccWidgets.action(body, "Load look", _on_load_look)
	DccWidgets.note(body,
		"A look is every value in this panel -- Map view, Rendering-advanced, the "
		+ "ramp and the Painter styles -- written to its own small JSON file "
		+ "beside the project, not into the world .zip. It is reusable across "
		+ "worlds, which is the whole reason to save one, and the .zip is the "
		+ "reference app's format rather than this port's to extend.")
	DccWidgets.note(body,
		"Loading replaces the quality tier as the starting point, so a look saved "
		+ "at Ultra renders at Ultra wherever it is opened. Reset to quality tier "
		+ "above hands it all back.")
	_refresh_preset_list()

func _refresh_preset_list() -> void:
	if _preset_pick == null:
		return
	_preset_slugs.clear()
	_preset_pick.clear()
	for entry in bridge.appearance_presets():
		_preset_pick.add_item(String(entry[0]))
		_preset_slugs.append(String(entry[1]))
	_preset_pick.disabled = _preset_slugs.is_empty()

func _on_save_look() -> void:
	var name := _preset_name.text.strip_edges() if _preset_name != null else ""
	if name == "":
		push_warning("Save look: name the look first.")
		return
	if not bridge.save_appearance_preset(name):
		push_warning("Save look: the engine could not write the preset.")
		return
	_refresh_preset_list()
	for i in _preset_slugs.size():
		if String(_preset_pick.get_item_text(i)) == name:
			_preset_pick.selected = i

func _on_load_look() -> void:
	if _preset_pick == null or _preset_pick.selected < 0:
		return
	if not bridge.load_appearance_preset(String(_preset_slugs[_preset_pick.selected])):
		push_warning("Load look: the preset could not be read.")
		return
	_sync_appearance()
	_sync_ramp()
	_mark_custom()
	_refresh_map()

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

	var body := DccWidgets.section(_h(), "Painter styles")
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

	var water := DccWidgets.section(_h(), "Water & light")
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
	var sec := DccWidgets.section(_h(), "Still owed")
	DccWidgets.note(sec,
		"Of the ramp editor (CA-02, now live above): stop duplicate, an absolute "
		+ "elevation domain, and Auto Fit / Auto Breakpoints. Per-stop alpha and "
		+ "the Linear / Ease / Step modes landed 2026-08-24 and are above; the "
		+ "mode is the ramp's rather than a stop's, which is how the design draws "
		+ "it. The renderer's ramp is keyed to relative land elevation.")
	DccWidgets.note(sec,
		"Of colour grading (2026-08-24, now live above as its own group): "
		+ "exposure, contrast, saturation, temperature, the two tints, gamma "
		+ "(2026-08-24) and the four field-influence weights (2026-08-24, "
		+ "their own group below the grade) are real; the two tints are still "
		+ "a blue-to-amber axis rather than free colour pickers. "
		+ "Still owed: Material exposure per class (vegetation / rock / soil and "
		+ "their slope, curvature and wetness modulation -- only the lithology "
		+ "pair and the new curvature/crest stages are live) · Detail & "
		+ "atmosphere (macro / meso / micro intensity separately, and elevation "
		+ "haze -- only distance haze is live) · Preview (on-off, Compare "
		+ "current / previous / split).")
	DccWidgets.note(sec,
		"Of saving a look (CA-08, now live above): rename and delete a saved look, "
		+ "a thumbnail per look, and sharing one between machines. A look is a "
		+ "named JSON file under user://appearance_presets and nothing collects "
		+ "them into a theme.")
	DccWidgets.note(sec,
		"None of it is presentation-unsafe: every control here updates visible "
		+ "tiles only and never marks a generation stage stale.")

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
	## `svf_strength` and `shadow_strength` joined this group 2026-09-03
	## (`OUTSTANDING_WORK.md` §2.5) and sit **beside** the two occlusion rows
	## rather than in a group of their own, because the engine multiplies all
	## three into one field (`render.rs`'s `fold_lighting_fields`, which is the
	## reference's own `aoC`). Both are `0.0` in the shipped default.
	["Relief & light", ["relief_lights", "relief_directionality", "relief_ambient",
		"relief_gain", "relief_chroma", "ao_strength", "ao_radius_frac",
		"svf_strength", "shadow_strength",
		"crest_strength", "curve_shade", "ridged_strength"]],
	["The sheet", ["paper_strength", "paper_grain", "paper_mottle", "paper_wash",
		"stipple_strength", "border_width_frac"]],
	## `rock_slope`, `wetness` and `sea_grain_warp` joined this group 2026-09-03
	## (`OUTSTANDING_WORK.md` §2.5); `geo_micro` and `sdf_coast` joined it later
	## the same day, and `sdf_rivers`/`sdf_biomes` on the pass after that. The
	## first two are the reference's own last two unported `landColorCore`
	## colour stages; `sea_grain_warp` is not a reference row at all but the
	## flag over the reference's ocean noise lattice -- see its help line.
	## `geo_micro` sits directly under the two geology rows because the
	## reference drives all three from one `geologyR` slider, and the three SDF
	## rows sit together after the two wetness rows, in the reference's own
	## panel order, because they are the land tints keyed on distance rather
	## than on climate. All seven are `0.0` in the shipped default, so the
	## panel opens on the same picture it always did.
	##
	## **Derived from the `sdf_coast` row, not designed.** Owner ruling
	## 2026-08-25: where no canvas exists, derive from the DCC vocabulary. The
	## sibling row *is* that vocabulary -- same widget, same 0-1 range from the
	## engine's own `TUNABLE` table, same help-line shape (what it draws, what
	## it costs when off). `ui-ux-pro-max` returned no verified match for
	## "another row in an existing slider group", which is what its own
	## skip-this-for-non-visual-work rule predicts.
	["Materials", ["biome_sat", "tex_strength", "rock_slope",
		"litho_strength", "litho_exposure", "geo_micro",
		"hydro_wet_strength", "wetness",
		"sdf_coast", "sdf_rivers", "sdf_biomes",
		"local_contrast", "splat_strength", "sea_grain_warp"]],
	## §19 (`TERRAIN_APPEARANCE_RESEARCH.md`, `OUTSTANDING_WORK.md` §2.5): the
	## other two atmospheric-perspective axes research named, over the same
	## plate-edge distance `haze_strength` already reads -- there is no
	## camera, so "far" is this plate-relative distance for all three.
	["Atmosphere", ["haze_strength", "atmo_desaturation", "atmo_contrast"]],
	## §16 (`TERRAIN_APPEARANCE_SCOPE.md`, `OUTSTANDING_WORK.md` §2.5): the
	## three hillshade bands `land_color` already blends -- macro, meso and a
	## per-pixel micro jitter -- as their own named weights instead of the
	## hardcoded 0.40/0.40/0.20 they used to be. A relief group, not a
	## materials one: nothing here reads climate, biome or geology, only the
	## same shading `land_color` computes for "Relief & light" above.
	["Multi-scale detail", ["detail_macro_weight", "detail_meso_weight", "detail_micro_weight"]],
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
	"detail_macro_weight": "Weight of the broad relief silhouette in the shading blend -- the same multidirectional hillshade the Relief & light group above lights.",
	"detail_meso_weight": "Weight of the same hillshade at a smoothed, coarser scale -- keeps large landforms legible once the macro band starts responding to per-cell noise.",
	"detail_micro_weight": "Weight of a per-pixel high-frequency jitter of the macro band -- fine surface grain, distinct from the sheet's paper grain, which textures colour rather than light.",
	"ao_strength": "Ambient occlusion: darkens enclosed valleys and gorges that see less sky. The reference's own Ambient occlusion slider.",
	"ao_radius_frac": "How wide the occlusion samples, as a share of map width -- a basin scale, not a pixel scale.",
	"svf_strength": "Sky view factor: how much of the sky hemisphere each cell can see, so enclosed valleys and gorge floors lose the diffuse skylight open ridgetops keep. A different measurement from Ambient occlusion above, not a second dial on it -- occlusion compares a cell against a blurred copy of the terrain (does this sit in a hollow), while this ray-casts eight directions for the horizon (how high does the land rise around it). A broad shallow basin is a strong hollow and a weak enclosure; a narrow gorge between two walls is the reverse. The reference multiplies both into one field and so does this. Costs one whole-grid pass when on and nothing when off.",
	"shadow_strength": "Cast shadows: marches each cell toward the sun and darkens it where terrain rises above the sun ray -- the long soft shadows a mountain range throws, which is the one relief cue a hillshade cannot give however many light directions it is handed, because a hillshade only ever asks about the local surface angle. The sun elevation here is the reference's own fixed 20deg rather than the Sun elevation slider above: a horizon shadow only exists at a low sun, and at this map's 40deg default almost nothing on a real heightfield rises above the ray. Costs one whole-grid pass when on and nothing when off.",
	"paper_strength": "The paper/vellum ground: fibre, tooth, ageing and a warm tint. The reference's Parchment slider, on by default here because this port's base look is an atlas plate.",
	"paper_grain": "Amplitude of the sheet's fibre and laid lines.",
	"paper_mottle": "Amplitude of the broad ageing/staining blotches, at sheet scale.",
	"paper_wash": "How far the colour is muted toward a paper-coloured grey of the same luminance. Costs chroma only, never relief or biome legibility.",
	"stipple_strength": "Forest stippling -- canopy texture driven by the real canopy fraction, applied zero-mean so the forest gains texture without darkening.",
	"border_width_frac": "Width of the plate frame (bare-paper margin plus neatlines), as a share of map width. 0 removes the frame.",
	"litho_strength": "Geology tint: how far exposed rock moves from the climate heuristic toward the palette of the rock actually underneath. The reference's Geology materials slider. Inert on a loaded save, which stores no lithology.",
	"litho_exposure": "How strongly bedrock shows through the soil cover, from slope, vegetation and moisture.",
	"geo_micro": "Rock microtexture: per-rock-type surface detail over the geology tint above -- granite mineral speckle and fracture creases, basalt lava-field patchiness, andesite ash and cinder, limestone karst pitting, sandstone and shale strata banded by elevation, metamorphic folded gneiss -- plus the wind-ripple banding the same slider gives gentle sandy ground. The reference's Geology materials slider drives its texture and its colour from one number; the colour half already arrives here from Geology tint and Bedrock exposure above, over an editable palette the reference does not have, so this row is the texture only rather than a second geology colour disagreeing with the first. Inert on a loaded save, which stores no lithology, and on ground the soil still covers.",
	"hydro_wet_strength": "Darkens and cools ground near real channels -- a soft halo around the drainage network, applied to the finished pixel. Not the reference's Wetness slider, which this row was mislabelled as until 2026-09-03 and which is the separate Wet ground (TWI) row below. Gated on real upstream drainage area, so it marks the same rivers whatever the map's resolution (GUI_GAP_REGISTER.md CA-11 -- it used to fade out as the grid got finer, and at 2048 wide it moved nothing at all).",
	"ramp_strength": "How far the colour relief ramp takes over from the material colour. 0 is the material model alone (climate, slope, relief); 1 is a full hypsometric tint. The ramp is applied before the light, so the hillshade, occlusion and paper still read through it at any strength.",
	"local_contrast": "Adds band-limited detail back after the paper wash. The gain falls to zero on strong edges, so coastlines and snowlines cannot halo.",
	"splat_strength": "How strongly a loaded asset pack's ground textures blend in. Inert with no pack loaded. The reference's Texture strength.",
	"relief_chroma": "How far the relief lighting keeps the map's colour instead of fading it toward grey. 0 is the reference exactly -- shaded ground is pulled toward one fixed neutral, which costs value as well as chroma. At 1 the shading desaturates about each pixel's own luminance, and shadow cools while sunlight warms, the way a real scene's sky-lit shadow and warm sun differ.",
	"crest_strength": "Thin bright strokes along convex, steep ridge lines -- the reference's Ridge crests. Costs one whole-grid pass when on and nothing when off.",
	"curve_shade": "Sun-independent lighting straight from the surface curvature: convex ridges brighten, concave valleys darken. Keeps a landform legible where it happens to run parallel to the sun. The reference's Curvature shading.",
	"ridged_strength": "Folded creases from a ridged multifractal, weighted by elevation squared so they concentrate in the highlands and leave the lowlands alone. The reference's Ridged relief.",
	"biome_sat": "How colourful the material mix is, about its own luminance -- so it can never make one material lighter or darker relative to its neighbour, only more or less saturated. Negative is toward grey. No reference counterpart: the reference's only chroma control is Relief <-> biome, which pulls toward a fixed grey and therefore flattens as it desaturates.",
	"tex_strength": "A three-frequency fine surface modulation over the material colour -- the reference's Surface texture. Evaluated in map coordinates, so a tiled export stays seamless.",
	"rock_slope": "Extra rock exposure on steep ground -- the reference's Slope rock. A tint over the finished material mix rather than a change to it, so the climate/slope blend underneath is untouched. Its steepness threshold is the reference's own, which is measured per cell: a cliff crosses it at any map size, but how much of the middle ground qualifies falls as the grid gets finer.",
	"wetness": "Darkens and cools ground with a high topographic wetness index -- valley bottoms, seeps and saturated soils. This is the reference's Wetness slider, and a different stage from the near-channel wetness directly above: unblurred, keyed on terrain shape rather than on river flow, and applied to the material before the light rather than to the finished pixel. Both can be on.",
	"sdf_coast": "Coast bands: a bright wet shore-sand band and a lusher coastal plain behind it, keyed on distance from the coastline itself rather than on elevation, so both hold a constant width whatever the relief does and at whatever resolution the map was generated. The reference's SDF coastlines, and the first of its three distance-keyed rows -- the two below are its siblings. Costs one whole-grid distance transform when on and nothing when off.",
	"sdf_rivers": "River bands: a damp bank, a wetland green behind it and a wide pale floodplain behind that, in three widening rings out from every watercourse. Measured from the channels themselves rather than from height or rainfall, so a river reads as a valley floor at any resolution instead of a coloured line. Same family as Coast bands above, keyed on rivers instead of the shore. Nothing on a loaded save, which stores no flow field and therefore knows where no rivers are. Costs one whole-grid distance transform when on and nothing when off.",
	"sdf_biomes": "Biome blend: widens the noise that ragged the boundary between two biomes, in proportion to how close the ground is to that boundary -- so grassland dissolves into forest over a band rather than at a line, while each biome's interior stays as crisp as it was. Not a tint: it moves where the materials meet, not what colour they are. Costs a water-body pass and one whole-grid distance transform when on, and nothing when off.",
	"sea_grain_warp": "Breaks up the rectangular quilt visible in open ocean at low zoom -- squares about eighty cells across, caused by the sea's colour noise being sampled on a grid-aligned lattice. 0 is the reference's exact lattice, artifact included, and is what the shipped default uses: the blockiness is inherited from the reference HTML rather than introduced here, so removing it is a deliberate divergence and this slider is where you opt into it.",
	"haze_strength": "Atmospheric perspective: how far the plate fades toward sky at its edges. The reference's own fixed 0.18, made adjustable; the shipped look uses 0.09, which reads as air rather than as a vignette.",
	"atmo_desaturation": "How far the outer plate loses colour toward the haze above, same distance as the sky tint. 0 keeps material colour equally saturated everywhere.",
	"atmo_contrast": "How far the outer plate's material contrast flattens toward the haze above. 0 keeps full material contrast everywhere; the centre is always unaffected.",
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

## The Colour management picker and the names behind it, retained for
## `_sync_color_space()` -- see `_build_color_management()` for why a control
## that is deliberately *not* part of the look still has to be re-read when the
## world changes.
var _color_space_pick: OptionButton
var _color_space_names: Array = []
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
	## Beside the ramp rather than anywhere else: the ramp *is* the Colour
	## relief row's content, and this is the stack that orders it.
	_build_layer_stack()
	_build_appearance(APPEARANCE_GROUPS)
	_build_color_management()
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
## sheet, material, atmosphere and multi-scale-detail groups. v3 keeps this
## category "whole, unchanged in scope" -- its migration audit's own words --
## so the split below is only the grade leaving for COLOURS, which v3 does
## ask for. (§16's group joined the other five here 2026-09-02, after v3's
## own migration -- a relief control, not a colour one, so it grew this slice
## rather than the other.)
func build_terrain_appearance_into(parent: Control) -> void:
	_host = parent
	_build_ramp()
	_build_appearance(APPEARANCE_GROUPS.slice(0, 5))
	_host = null

## v3 CARTO ▸ LAYERS: the terrain raster's three separable categories
## (`GUI_GAP_REGISTER.md` CA-03/CA-04). Drawn into CARTO's own Layers category
## rather than into Terrain appearance, because §7 draws one layer list and this
## is the part of it that is a stack rather than a set of switches -- the eight
## rows `cartography_workspace.gd` already builds there are whole overlays with
## nothing to order.
##
## It lives in *this* file all the same, with the ramp and the tunables: the
## stack is `TerrainAppearance` state, it is what `_mark_custom()` and
## `_refresh_map()` are for, and `on_world_changed()` has to re-read it beside
## everything else that a project open can move.
func build_layer_stack_into(parent: Control) -> void:
	_host = parent
	_build_layer_stack()
	_host = null

## v3 CARTO ▸ COLOURS: vibrancy/saturation/contrast/brightness/gamma/temp/tint
## (the colour grade), `+ Field influence weights`, then colour management.
##
## Colour management sits **after** the grade rather than before it, and the
## order is the pipeline's: the grade decides what the picture is, the output
## space decides how the finished picture is encoded for the panel it is going
## to. Reading the section top to bottom is reading `build_color_texture` in
## order.
func build_colours_into(parent: Control) -> void:
	_host = parent
	_build_appearance(APPEARANCE_GROUPS.slice(5), "Colour grade")
	_build_color_management()
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
				## `reset_appearance()` hands back **every** tunable, and the
				## layer stack is one of them (`appearance_layers` is dropped
				## by it). Without this the engine returns to the default
				## order while the panel keeps drawing the user's arrangement,
				## and the next unrelated rebuild silently adopts whichever
				## the panel happened to hold. Found by a verifier 2026-09-03,
				## measured as a real divergence rather than reasoned about.
				_sync_layer_stack()
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
		## **This note has now been wrong twice in one day, in the same
		## way** -- `MISTAKES.md`'s "prose that describes the old behaviour",
		## in the surface a user actually reads. The second time it said the
		## two remaining SDF legs "wait on one thing, the distance transform
		## their coast sibling already has"; they did not wait on anything.
		## `build_coast_sdf` **is** that transform over a mask, which is how
		## the reference writes `buildRiverSDF` too, so both shipped by
		## composition (`render.rs::build_river_sdf`,
		## `build_biome_boundary_dist`). What is left is checked rather than
		## inherited: `minorStreams` and `season` still return zero hits in
		## `render.rs`.
		DccWidgets.note(body,
			"Not bound, because the engine has no such stage: minor "
			+ "channels and season blend. Those are reference render stages this "
			+ "port has not ported, not bindings it is missing. Ridge crests, "
			+ "surface texture, ridged relief and curvature shading left this list "
			+ "on 2026-08-24, slope rock and the reference's own TWI wetness on "
			+ "2026-09-03, sky view factor, cast shadows and SDF coastlines later "
			+ "the same day, and the river-band and biome-blend legs on the pass "
			+ "after that; all eleven are live above, alongside the rock "
			+ "microtexture, which this note never listed.")
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


# -- The raster layer stack (`GUI_GAP_REGISTER.md` CA-03/CA-04) ----------------
#
# `DCC_SHELL_SPEC.md` §7's layer list, for the one part of it that is a *stack*:
# Terrain, Colour relief and Hillshade, the three categories `render::LayerStack`
# separates inside `land_color`. §6's Layers context names the row grammar --
# "visibility dot, name, opacity bar, blend mode" -- and this is that row, plus
# the reorder §6 calls "nested children under Terrain" and §7 draws as a
# draggable stack.
#
# **Three rules this panel is built around, each of which is a way to get it
# wrong:**
#
# 1. **Never write a stack the user has not asked for.** Nothing here calls
#    `set_layer_stack` on build, on sync, or on a world change -- only a real
#    gesture writes. `LayerStack::is_default()` is a structural comparison, so an
#    explicitly-default write would still render byte-identically; what it would
#    do is leave `WorldGen::appearance_layers` holding `Some(...)`, which
#    `reset_appearance()` counts as an override the user never made.
# 2. **An absent key means unchanged, never default.** `set_layer_stack` merges
#    each row over the entry that layer has now, so `_push_layer` sends the id
#    plus *only* the key the gesture moved. Restating every value would make this
#    panel overwrite whatever a second writer had just changed.
# 3. **Reorder is data, not redraw order.** `_move_layer` sends a new id order
#    and then re-reads the engine; the rows are rebuilt from what came back. The
#    list never reorders its own children and hopes the composite agrees.

## The engine's own rows, top-first, as `get_layer_stack()` handed them over.
## A cache for `_push_layer`/`_move_layer` to build their id lists from, never a
## draft: every write is followed by a re-read.
var _layers: Array = []
var _layer_host: VBoxContainer
var _blend_names: Array = []

func _build_layer_stack() -> void:
	if not bridge.layer_stack_api:
		return
	var body := DccWidgets.section(_h(), "Terrain raster")
	_blend_names = bridge.blend_modes()
	_layer_host = VBoxContainer.new()
	_layer_host.add_theme_constant_override("separation", 4)
	body.add_child(_layer_host)
	DccWidgets.note(body,
		"The three categories the terrain raster separates into, top of the list "
		+ "drawn last. They composite inside one pass over the same colour every "
		+ "consumer reads, so the map, every PNG export and every region crop all "
		+ "get the arrangement below -- there is no path that can miss it.")
	DccWidgets.note(body,
		"The composite starts from a white ground, not black. Hide Terrain and "
		+ "Hillshade's Multiply leaves the grey relief plate a reader means by "
		+ "\"hillshade alone\", which is the intended reading rather than a bug: "
		+ "white is Multiply's identity, and Normal ignores the backdrop at full "
		+ "alpha, so those are the only two operators reachable with nothing "
		+ "underneath.")
	DccWidgets.note(body,
		"Presentation only, and session-scoped with one exception: Save look "
		+ "(Map presets) writes the arrangement into the look file, and Reset to "
		+ "quality tier drops it. A project .zip does NOT carry it -- the saved "
		+ "appearance document round-trips the tier, the look, the overrides, the "
		+ "ramp and the Painter block, and has no layers field -- so this is one "
		+ "control in the dock that File - Save does not preserve.")
	## The one funnel. Every accepted write -- this panel's or the right dock's
	## appended Layers section -- lands here, so the rows, the Custom mark and
	## the repaint happen once each and in one place. `_apply_layer_stack` below
	## therefore does none of the three itself.
	bridge.layer_stack_changed.connect(func():
		_sync_layer_stack()
		_mark_custom()
		_refresh_map())
	_sync_layer_stack()

## Re-read the engine and rebuild every row from it.
##
## A full teardown rather than an in-place update, for `_rebuild_ramp_rows()`'s
## own reason: a reorder changes which row is which, and there is no per-row
## identity to update against once the order has moved.
func _sync_layer_stack() -> void:
	if _layer_host == null:
		return
	_layers = bridge.layer_stack()
	for child in _layer_host.get_children():
		_layer_host.remove_child(child)
		child.queue_free()
	if _layers.is_empty():
		DccWidgets.note(_layer_host, "The engine returned no layers.")
		return
	for i in _layers.size():
		_layer_row(_layer_host, _layers[i] as Dictionary, i)

## One layer: the header row (dot, name, reorder), then its opacity and blend.
##
## Two lines and not one because the dock is 304-372 px wide and §7's own row
## carries four controls; a slider and a dropdown crammed beside a 132 px label
## column would clip both. The header is the row that drags and the row §6's
## collapsed readout counts.
func _layer_row(parent: Control, d: Dictionary, index: int) -> void:
	## Every key is read through `has()`. `get_layer_stack()` sets all five
	## today, but a row that arrived short must say which field is missing
	## rather than render a plausible `false`/`0.0`/`"Normal"` -- a hidden layer
	## and an unreported one look identical once a default has been invented.
	var missing: Array = []
	for k in ["id", "label", "visible", "opacity", "blend"]:
		if not d.has(k):
			missing.append(k)
	if not missing.is_empty():
		DccWidgets.note(parent,
			"Layer row %d is unreadable - the engine sent no %s."
			% [index, ", ".join(missing)])
		return

	var id := String(d["id"])
	var visible := bool(d["visible"])

	## **A row can be wired and still move no pixels, and saying so is the whole
	## point of this block.** `TerrainAppearance::ramp_strength` ships at `0.0`,
	## and `LayerStack::composite` skips Colour relief entirely when the ramp
	## contributes nothing (`None => continue`). So at the shipped default this
	## row's dot, opacity, blend and reorder are all live controls over a layer
	## that draws nothing — measured by a verifier as a byte-identical
	## hillshade/colour-relief swap.
	##
	## Disclosed rather than disabled: the controls still *work*, and they take
	## effect the moment the ramp has a strength, so greying them out would be
	## the opposite lie. This is the "dash it with its reason" idiom applied to
	## a whole row instead of a field.
	if id == "colour_relief" and float(bridge.appearance().get("ramp_strength", 0.0)) <= 0.0:
		DccWidgets.note(parent,
			"Colour relief draws nothing right now - Ramp strength is 0, so the "
			+ "ramp contributes no colour and this row's settings have no visible "
			+ "effect. Raise Ramp strength (Rendering - advanced) to see them.")

	var tablet := DccTheme.is_tablet()
	var readout_fs := DccTheme.role_px("fs_readout") if tablet else DccTheme.FS_SMALL

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", DccTheme.role_px("dock_row_gap"))
	head.custom_minimum_size.y = DccTheme.role_px("row_min_h") if tablet else 22
	## The row is the drag source, so it must actually receive the press --
	## its children (three buttons) still get input first, which is why the
	## dot and the two reorder buttons keep working.
	head.mouse_filter = Control.MOUSE_FILTER_STOP

	## §7's visibility dot. A button rather than the `_sculpt_stamp_row` mark it
	## borrows its glyphs from, since here it is the control and not a readout.
	var dot := Button.new()
	dot.flat = true
	dot.focus_mode = Control.FOCUS_NONE
	dot.text = DccIcons.SYMBOLS["on"] if visible else DccIcons.SYMBOLS["off"]
	dot.add_theme_font_override("font", DccTheme.mono(0))
	dot.add_theme_font_size_override("font_size", readout_fs)
	dot.add_theme_color_override("font_color",
		DccTheme.c("text") if visible else DccTheme.c("text_ghost"))
	dot.tooltip_text = "%s this layer. Hiding Terrain leaves the white ground the other two composite over, not black." \
		% ("Hide" if visible else "Show")
	if tablet:
		dot.custom_minimum_size = Vector2(DccTheme.role_px("row_min_h"), DccTheme.role_px("row_min_h"))
	dot.pressed.connect(func(): _push_layer(id, "visible", not visible))
	head.add_child(dot)

	var name_label := DccTheme.mono_label(String(d["label"]),
		"text" if visible else "text_ghost", readout_fs)
	name_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_label.clip_text = true
	head.add_child(name_label)

	## WCAG 2.2 SC 2.5.7: a pointer alternative to the drag below, and the
	## alternative the guideline names by example. Same vocabulary as the sculpt
	## stamp stack's own Move up / Move down, spelt out rather than arrowed --
	## `DccIcons.SYMBOLS` has no vertical arrow, and an unlabelled glyph button
	## is the anti-pattern this shell keeps out of its docks.
	var up := _reorder_button(head, "Up", index > 0, readout_fs, tablet,
		"Move this layer one place up the stack - it then composites later, over its old neighbour.")
	up.pressed.connect(func(): _move_layer(index, index - 1))
	var down := _reorder_button(head, "Down", index < _layers.size() - 1, readout_fs, tablet,
		"Move this layer one place down the stack - it then composites earlier, under its old neighbour.")
	down.pressed.connect(func(): _move_layer(index, index + 1))
	parent.add_child(head)

	## §7's drag reorder, over the same `_move_layer` the buttons call, so the
	## two cannot disagree about what a move means.
	head.set_drag_forwarding(
		func(_at: Vector2) -> Variant:
			var ghost := DccTheme.mono_label(String(d["label"]), "accent", readout_fs)
			head.set_drag_preview(ghost)
			return {"cartalith_layer_row": index},
		func(_at: Vector2, data: Variant) -> bool:
			return data is Dictionary and (data as Dictionary).has("cartalith_layer_row"),
		func(_at: Vector2, data: Variant) -> void:
			_move_layer(int((data as Dictionary)["cartalith_layer_row"]), index))

	var pad := DccWidgets.pad(parent, 14, 0, 0, 0)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	pad.add_child(col)
	## Engine write on release only, on `_build_npr`'s own reasoning: a full-map
	## re-render is not a per-drag-pixel operation. The readout follows the
	## handle live because `DccWidgets.slider` updates it itself.
	var pending := [float(d["opacity"])]
	DccWidgets.slider(col, "Opacity", 0.0, 1.0, 0.01, pending[0], "",
		func(v: float): pending[0] = v,
		"How much of this layer takes part. It multiplies whatever alpha the category already carries, so Colour relief at 0.5 here is half of ramp strength times the stop's own alpha.",
		func(): _push_layer(id, "opacity", pending[0]))
	var bi: int = _blend_names.find(String(d["blend"]))
	DccWidgets.choice(col, "Blend", _blend_names, bi,
		func(i: int): _push_layer(id, "blend", String(_blend_names[i])),
		"How this layer combines with everything under it. Hillshade opens on Multiply, which is exactly the c * light the renderer has always drawn; the other two open on Normal.")
	if bi < 0:
		## The engine named a mode this build's own list does not carry. Said
		## rather than silently shown as the first entry, which would claim the
		## picture is Normal when it is not.
		DccWidgets.note(col, "Blend mode \"%s\" is not in this build's picker." % String(d["blend"]))

func _reorder_button(parent: Control, text: String, enabled: bool, fs: int,
		tablet: bool, tip: String) -> Button:
	var b := Button.new()
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.text = text
	b.disabled = not enabled
	b.tooltip_text = tip
	b.add_theme_font_size_override("font_size", fs)
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	if tablet:
		b.custom_minimum_size.y = DccTheme.role_px("btn_min_h")
	parent.add_child(b)
	return b

## One gesture, one key. See this section's header, rule 2: the row carries its
## id and nothing but the field that moved, so every value this panel did not
## touch is left to the engine's own current entry.
func _push_layer(id: String, key: String, value: Variant) -> void:
	var rows: Array = []
	for r in _layers:
		var d: Dictionary = r
		var row := {"id": String(d.get("id", ""))}
		if String(d.get("id", "")) == id:
			row[key] = value
		rows.append(row)
	_apply_layer_stack(rows)

## Rule 3: the new order, sent as data. The engine is the only thing that
## decides what the stack is; the rows are rebuilt from its answer.
func _move_layer(from: int, to: int) -> void:
	if from == to or from < 0 or from >= _layers.size() or to < 0 or to >= _layers.size():
		return
	var ids: Array = []
	for r in _layers:
		ids.append(String((r as Dictionary).get("id", "")))
	var moved: String = ids[from]
	ids.remove_at(from)
	ids.insert(to, moved)
	var rows: Array = []
	for id in ids:
		rows.append({"id": id})
	_apply_layer_stack(rows)

## 3 or 0 -- all three rows or none. A refusal changed nothing, so the rows on
## screen are still right and re-reading would be waste; it is still reported,
## because a silent no-op is how a panel starts lying about the map. On success
## the rebuild, the Custom mark and the repaint all arrive through
## `layer_stack_changed` (see `_build_layer_stack`), which is also the path the
## right dock's writes take.
##
## Returns whether it was accepted, for `RightDock._write_layers()`'s reason:
## with no return value the refusal branch's only effect is a log line, and
## mutation testing scored it SURVIVED.
func _apply_layer_stack(rows: Array) -> bool:
	if bridge.set_layer_stack(rows) != 3:
		push_warning("Layers: the engine refused the stack; nothing changed.")
		return false
	return true

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
	## Picking is deliberately not loading: a look replaces every value in this
	## panel, so it is not something to trigger by scrolling a dropdown past the
	## wrong entry, and `_on_save_look` selects the look it just wrote without
	## wanting to re-apply it. What the two-step owed the user was saying so --
	## the picker moved and nothing happened, with no hint that a second press
	## finishes the job. The tooltip goes on the `OptionButton` as well as its
	## row, since `_row` puts it on the row and the button covers most of it.
	const PICK_TIP := "Looks saved on this machine. Picking one only selects it -- press Load look below to apply it."
	_preset_pick = DccWidgets.choice(body, "Saved", [], -1, func(_i: int): pass, PICK_TIP)
	_preset_pick.tooltip_text = PICK_TIP
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
	if bridge.layer_stack_api:
		## Verified against `WorldGen::load_appearance_preset`, which clears the
		## override map and the ramp override and does NOT clear
		## `appearance_layers`; `appearance()` then lets that session stack win
		## over the loaded preset's own. Saving is whole either way --
		## `save_appearance_preset` writes the merged appearance, layers
		## included. Stated rather than worked around: the shell cannot read a
		## preset's stack without loading it, so there is nothing here to
		## restore it from.
		DccWidgets.note(body,
			"The layer arrangement (Layers - Terrain raster) is saved into a look "
			+ "and comes back with it -- unless you have moved a layer since this "
			+ "session started, in which case your arrangement stays and the "
			+ "look's is ignored. Reset to quality tier first to load a look's "
			+ "layers exactly.")
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
	## Re-read the layer rows for the same reason every other block above is
	## re-read: the loaded look carries a stack. **Whether the rows then change
	## depends on the engine, not on this call** -- see the note in
	## `_build_look_presets()` for the one case where they do not.
	if _layer_host != null:
		_sync_layer_stack()
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

# -- Colour management (`LARGE_ITEM_RULINGS.md`, owner-ruled build) ------------

## The output colour space, as **one axis and one stated fact** rather than the
## three-row radio `OUTSTANDING_WORK.md` §2.5 draws.
##
## `GUI_GAP_REGISTER.md` §7.6 read Blender 4.x's own Color Management panel and
## found §2.5's row (`sRGB · Display P3 · linear`) offers one control where
## there are two axes: sRGB and Display P3 are **display devices**; linear is a
## **working space**. Its words: shipping the three as one dropdown *"would be a
## category error that becomes very expensive to unpick later."* So the picker
## carries the two display devices, and the working space is a stated fact with
## its reason -- which is also what this shell does everywhere else rather than
## draw an enabled control that resolves to one value.
##
## No `_mark_custom()` and no sync in `on_world_changed()`, and both omissions
## are deliberate: this is the one control in this file that is **not part of
## the look**. It describes the monitor, not the map, so it is not saved into a
## look preset and not written to the project. See `WorldGen::color_space`'s own
## doc for why carrying a display device in a document is the wrong shape.
##
## **It still needs syncing, and the reason this comment used to give for not
## syncing it was wrong.** It read *"opening a project therefore cannot leave
## this row stale, because nothing underneath it moved"*. Nothing in the
## *document* moves, which is true and is not the question: `File ▸ Close
## project` runs `EngineBridge.close_world()`, which does `world_gen =
## WorldGen.new()`, and the fresh handle re-initialises `color_space` to `Srgb`.
## The engine moved even though the document did not, so the picker was left
## reading Display P3 over an sRGB engine -- measured in the real shell before
## this was fixed. `_sync_color_space()` below closes it.
func _build_color_management() -> void:
	if not bridge.color_space_api:
		return
	var spaces: Array = bridge.color_spaces()
	if spaces.is_empty():
		return
	var body := DccWidgets.section(_h(), "Colour management")
	_color_space_names = spaces
	_color_space_pick = DccWidgets.choice(body, "Display", spaces, maxi(spaces.find(bridge.color_space()), 0),
		func(i: int): _on_color_space(String(spaces[i])),
		"Which display the finished map is encoded for. sRGB is the default and "
		+ "is what almost every monitor expects. Choose Display P3 only for a "
		+ "wide-gamut screen that is NOT colour-managing sRGB input for itself: "
		+ "on one of those, sRGB numbers read oversaturated and these read "
		+ "correct. On an ordinary sRGB screen it is the other way round.")
	DccWidgets.note(body,
		"Working space: sRGB, 8 bits per channel. A fact, not a choice -- the "
		+ "renderer composites into an 8-bit buffer, and 8-bit linear is not a "
		+ "usable working space at that depth: it loses a significant amount of "
		+ "its information in the darks, which is what Godot's own documentation "
		+ "warns about too. A linear option would have to wait for the "
		+ "high-precision pipeline, and it would arrive as a View Transform "
		+ "beside the grade above, not as a third entry in this picker.")
	DccWidgets.note(body,
		"Display P3 re-encodes the map raster only. The overlays drawn over it "
		+ "-- rivers, labels, settlement markers, territory, the scale bar -- and "
		+ "the interface around it stay sRGB, because Godot's compatibility "
		+ "renderer does no colour management and there is no hook to convert a "
		+ "colour on its way to the screen. Exports are unaffected and stay sRGB, "
		+ "which is right: an image file with no profile is read as sRGB.")
	DccWidgets.note(body,
		"Display P3 is a real change to the numbers, not a tag: measured on a "
		+ "2048-wide map it moves 87% of the bytes, by up to 31 levels. Greys, "
		+ "the paper ground and the neatlines are the exception and do not move "
		+ "at all. It also costs gradient resolution -- re-encoding 8-bit sRGB "
		+ "into the wider P3 container collapses about 48% of the 16.7 million "
		+ "codes, so the smoothest washes (deep sea, edge haze) can lose a "
		+ "level. That is the 8-bit buffer's cost, and the high-precision "
		+ "pipeline is what buys it back.")

func _on_color_space(name: String) -> void:
	if not bridge.set_color_space(name):
		return
	_refresh_map()

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


# -- A world arriving underneath these panels ---------------------------------

## Re-read every control in this file from the engine, because a project open
## replaced what the engine holds without any control here moving.
##
## **This was a real data loss, not a cosmetic staleness** (`GUI_GAP_REGISTER`
## shape, found 2026-09-01). `project_bridge.rs`'s `AppearanceDoc` round-trips
## six things -- quality, look, territory opacity, the appearance overrides,
## the colour ramp and the NPR block -- and restores all six into the engine on
## File ▸ Open. This file draws five of them and subscribed to nothing, so
## every row still showed launch-time values afterwards.
##
## The ramp made that destructive rather than merely wrong. `_ramp` is a
## shell-side copy, and `_push_ramp()` sends **the whole list**: with `_ramp`
## still holding the launch-time ramp, the first colour swatch, stop drag,
## delete, Add stop or Reverse after a project open would overwrite the ramp
## that was just restored from the file with the one from before it was opened.
## Nothing warned, and the file on disk still held the good ramp until the next
## save wrote the bad one over it.
##
## Named `on_world_changed` to match the method `app.gd`'s own
## `_refresh_world_dependent()` broadcasts over the registered workspaces, so a
## future standalone registration of this class needs no new wiring. Today it
## is nested, so `cartography_workspace.gd`'s `_on_world_changed()` -- already
## connected to both `generation_finished` and `world_loaded` -- forwards it.
##
## Every half is guarded on whether that block was actually drawn rather than
## on the binary's `*_api` flags, which is the stricter of the two: an older
## cdylib leaves a block undrawn, and so does a future reordering of
## `_build()`, and only the node itself knows.
func on_world_changed() -> void:
	if not _app_rows.is_empty():
		_sync_appearance()
	if _look_pick != null:
		_sync_look_pick()
	## `_ramp_host` rather than `bridge.ramp_api`: it is non-null exactly when
	## `_build_ramp()` actually drew the editor, which is the thing `_sync_ramp`
	## refills.
	if _ramp_host != null:
		_sync_ramp()
	## `_layer_host` for `_ramp_host`'s reason: non-null exactly when
	## `_build_layer_stack()` actually drew the rows. **A read, never a write** --
	## `_sync_layer_stack` only calls `get_layer_stack`, so a world arriving
	## under this panel cannot push a stack the user never set.
	if _layer_host != null:
		_sync_layer_stack()
	if not _npr_rows.is_empty():
		_sync_npr()
	_sync_color_space()

## Re-read the engine's colour space into the picker.
##
## `select()`, not the handler: writing the value back into the engine it was
## just read from is the asymmetry `_sync_appearance`'s own note describes, and
## here it would also be a no-op write on every world change.
func _sync_color_space() -> void:
	if _color_space_pick == null or not bridge.color_space_api:
		return
	var i: int = _color_space_names.find(bridge.color_space())
	if i >= 0 and _color_space_pick.selected != i:
		_color_space_pick.select(i)

## The Painter block's own half of `on_world_changed()`.
##
## `set_pressed_no_signal` / a plain `value` write, deliberately asymmetric:
## a checkbox's handler calls `_push()`, which would write the value straight
## back to the engine it was just read from and mark the look Custom, so the
## two toggles are set silently. A slider's `value_changed` handler only
## updates the closure's own `pending` array -- the engine write is on
## `drag_ended` -- so letting it fire is not merely harmless, it is required:
## `pending` is unreachable from here, and leaving it stale would make the
## next drag of a *different* row push this row's old value back.
func _sync_npr() -> void:
	var npr: Dictionary = bridge.npr_settings()
	if npr.is_empty():
		return
	## Matching `_build_npr`: 0 *is* 1x in the engine (the reference's own
	## `waveDist>0?waveDist:1`), so the row shows the reach that is actually
	## drawn rather than a 0.00x that renders as 1x.
	if float(npr.get("wave_dist", 0.0)) <= 0.0:
		npr["wave_dist"] = 1.0
	for key in _npr_rows:
		if not npr.has(key):
			continue
		var row: Dictionary = _npr_rows[key]
		if row.has("check"):
			(row["check"] as CheckBox).set_pressed_no_signal(bool(npr[key]))
		elif row.has("slider"):
			(row["slider"] as Range).value = float(npr[key])
	## Not in `_npr_rows`: `_build_npr` keeps this one out of the table because
	## its handler is `_on_animate_water`, which owns the shader overlay as well
	## as the flag. Silent for the same reason the two toggles above are.
	if _anim_check != null:
		_anim_check.set_pressed_no_signal(bool(npr.get("animate_water", false)))

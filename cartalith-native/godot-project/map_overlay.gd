extends Control
## Phase 2 civilisation-layer overlay (cartalith-civ): settlements + roads
## drawn on top of `%MapView`'s terrain texture. Godot computes nothing
## here beyond screen-space placement -- every value drawn (position,
## faction, tier, name, population) comes straight from `WorldGen.
## get_settlements()`/`get_roads()` (ARCHITECTURE.md: "Godot computes
## nothing beyond layout").
##
## Sibling of `%MapView` under the same `MarginContainer` (`MapMargin`),
## so both share an identical laid-out rect -- this control's own `size`
## is what `%MapView`'s texture is drawn into. `%MapView` uses
## `stretch_mode = 5` (`STRETCH_KEEP_ASPECT_CENTERED`), so the texture
## itself is letterboxed within that rect; `_displayed_rect()` reproduces
## that same fit/centering math so markers land on the real pixels they
## represent, not on the raw control bounds.

## Okabe-Ito colourblind-safe qualitative palette (Okabe & Ito, 2008) --
## 6 mutually distinct hues, matching `CIV_FACTION_COUNT` in
## `cartalith-godot` exactly (faction ids are 1-based, index with `- 1`).
## Chosen deliberately independent of the UI's own light-parchment theme:
## this is data-driven map content (which faction owns this settlement),
## not UI chrome -- the same reasoning the terrain renderer's own biome
## colours already follow (theme-independent, see CHANGELOG's UI-reskin
## entries).
const FACTION_COLORS: Array[Color] = [
	Color(0.902, 0.624, 0.0),   # orange
	Color(0.337, 0.706, 0.914), # sky blue
	Color(0.0, 0.620, 0.451),   # bluish green
	Color(0.941, 0.894, 0.259), # yellow
	Color(0.0, 0.447, 0.698),   # blue
	Color(0.835, 0.369, 0.0),   # vermillion
]

## Reference's `CIV_SETTLEMENT_CLASSES` (line 14674), restricted to the six
## tiers `SettlementKind` -- and so `get_settlements()`'s own `kind` field --
## can actually produce. `metropolis` (rank 5, glyph ★) joined the list on
## 2026-08-20 when `_civSelectMetropolises` was ported: a promoted imperial
## seat is a real settlement kind now, not just a manual-icon slot. The
## monastery/fortress/university/industrial special kinds are still never
## assigned to a real settlement.
## `rank` drives `_civDrawSettlementPin`'s own size formula (`(4+klass.rank)
## *sc`, reference line 15166) exactly; `glyph` is the same per-tier
## character the reference draws centred on the pin (line 15180), reused
## here now that this control draws more than a flat circle. Replaces the
## old hardcoded `TIER_RADIUS` px dict -- radius is now derived below
## (`_settlement_pin_radius`), not looked up from an arbitrary constant.
const SETTLEMENT_CLASS := {
	"hamlet":  {"rank": 0, "glyph": "⌂"},
	"village": {"rank": 1, "glyph": "●"},
	"town":    {"rank": 2, "glyph": "◉"},
	"city":    {"rank": 3, "glyph": "⬣"},
	"capital": {"rank": 4, "glyph": "✦"},
	"metropolis": {"rank": 5, "glyph": "★"},
}

## `CIV_LOD_PLACE` (reference line 15373): the minimum RAW camera zoom
## (`_camera_zoom` itself -- the same un-clamped scale `_civZoomK()`/
## `_civZoomRaw()` both start from before either clamps or inverts it,
## reference line 15003) at which a tier's full pin+glyph+label shows.
## Below it the reference does not hide the place outright -- it draws a
## small faction-tinted dot instead (reference lines 15744-15757: "render a
## small faction-tinted dot instead of hiding the place entirely -- road
## endpoints stay anchored to a visible marker at any zoom"), so a capital
## or city (threshold 0) is always full-size, town needs a little zoom-in,
## and village/hamlet need progressively more -- restricted here to the
## six tiers `SettlementKind` (and so `SETTLEMENT_CLASS` above) actually
## produces; the reference's own dict also carries monastery/fortress/
## ruin/etc. this port's settlements never have. `metropolis` is `0` in the
## reference's own dict (line 15374), the same as capital and city.
##
## This is not a port invention: it is the same population/importance-
## tiered LOD reveal real map renderers use for POI/place density --
## OpenStreetMap Carto renders cities/towns from ~z9, villages from ~z12,
## hamlets from ~z14 (github.com/gravitystorm/openstreetmap-carto), and the
## Mapbox/MapLibre style spec's `minzoom`/`maxzoom` per symbol layer is the
## same mechanism generalised (maplibre.org/maplibre-style-spec/layers/).
const SETTLEMENT_LOD := {
	"metropolis": 0.0,
	"capital": 0.0,
	"city":    0.0,
	"town":    0.4,
	"village": 0.7,
	"hamlet":  1.4,
}
## Reference's own low-zoom fallback dot (line 15752-15756):
## `dr=(isPoi?1.4:1.9)*lsc` plus a `+0.6*lsc` dark outline ring -- `lsc`
## there is this file's own per-frame `sc`.
const LOD_DOT_RADIUS_SC := 1.9
const LOD_DOT_OUTLINE_SC := 0.6
## `sc`, screen px per size-formula rank-unit. Reference: `sc=max(1,GW/512)*
## civZoomK()*civIconScale()` (line 15165) -- a canvas-resolution term times
## an inverse-CSS-zoom term times a user icon-scale, so a pin holds roughly
## the same ON-SCREEN size regardless of grid resolution or camera zoom
## (`_civZoomK`'s own comment, line 14976-14978: "shrinking in canvas-space
## as you zoom in so the on-screen size ... stays roughly constant" -- the
## exact mechanism, verified against the real function rather than trusted
## from the paraphrase, `_civZoomK()` at line 14980-14983:
## `1/clamp(viewT.scale, 0.35, 5)`).
##
## Correction (2026-08-19, the LOD/settlement-fidelity bug pass): an earlier
## version of this comment argued the inverse-zoom term could be dropped
## entirely because "this control has no separate canvas/CSS-zoom split to
## reproduce... camera zoom is a plain transform `ViewportHost` applies to
## this whole control, the same role the reference's CSS transform plays
## over its canvas." That description of the transform was correct and the
## conclusion drawn from it was backwards: the reference needs `_civZoomK()`
## *precisely because* its own CSS transform plays that exact role -- the
## whole job of `civZoomK()` is shrinking the pin in canvas-space so that
## when the CSS transform (there) / `_camera.scale` (here, `ViewportHost.
## zoom()`) multiplies the rendered size back out, the two cancel and the
## ON-SCREEN size stays constant. Without an equivalent term here,
## `_camera.scale` alone was doing the reference's zoom-transform half of
## that cancellation with nothing supplying the other half, so a pin's
## on-screen size grew linearly with `ViewportHost.zoom()` instead of
## holding steady -- confirmed numerically before this fix (a settlement
## pin at `zoom=1.0` vs. the same pin at `zoom=4.0` measured 4x the on-screen
## radius, not the same one). `_civ_zoom_k()` below is the missing term,
## using the reference's own clamp bounds (0.35-5.0) rather than this port's
## own, wider `ViewportHost.ZOOM_MIN`/`ZOOM_MAX` (0.4-8.0) -- deliberately:
## the clamp exists purely so a pin never vanishes at extreme zoom-in nor
## dominates at extreme zoom-out, a readability bound with no reason to
## track wherever this port's own pan/zoom range happens to sit.
##
## The resolution term is still ported the same way this comment always
## described: tying `sc` to `rect.size.x` ALONE (not `rect.size.x/_gw`,
## unlike `tool_overlay.gd`'s brush-cursor radius) keeps a pin's on-screen
## size independent of grid resolution, matching what the reference's own
## `GW/512` term is actually for; a literal `radius_cells*(rect.size.x/_gw)`
## port would instead shrink pins on a bigger grid, which is right for a
## brush (a real world-space distance) but wrong for a pin's glyph size (not
## a world-space quantity at all). `PIN_SCALE_REF_PX` is a tuned constant
## this port chose (not a reference value -- there is no equivalent number
## to port), sized so a typical dock-width viewport lands close to the
## pre-formula `TIER_RADIUS` constants it replaces.
const PIN_SCALE_REF_PX := 1400.0
const CAPITAL_RING_WIDTH := 2.5

## Soft drop-shadow under each full-tier pin (2026-08-19, "settlement
## rendering could be made graphically more interesting" pass, explicit
## owner latitude to improve past the reference's own plain filled-circle
## baseline -- `DECISIONS.md` §7a "principled equivalence" room, same as
## this session's own Asset Library/Travel Library work). A flat faction-
## coloured fill alone reads ambiguously against some of this renderer's
## own biome colours (pale sand, snow, light grassland) without a dark halo
## under it; the reference (`_civDrawSettlementPin`) draws no shadow at
## all, so this is a genuine addition, not a port of anything.
const PIN_SHADOW_COLOR := Color(0.0, 0.0, 0.0, 0.28)
const PIN_SHADOW_OFFSET_SC := 0.6

## A small water-blue "harbour" badge at a coastal pin's lower-right --
## grounded in real, derived engine data (`get_settlements()`'s own
## `coastal` field, `civ_is_coastal` in `cartalith-civ`, re-verified against
## final placement geometry by this same pass's Part 1 fix), not invented
## decoration. River adjacency has no equivalent per-settlement flag
## anywhere in `cartalith-civ` (`NamedSettlement` carries no river bool),
## so only the coastal case gets a badge -- a river indicator would have no
## real data backing it.
const COASTAL_BADGE_COLOR := Color(0.337, 0.706, 0.914, 0.95) ## FACTION_COLORS[1]'s sky blue -- reads as "water" at a glance
const COASTAL_BADGE_OUTLINE := Color(0.051, 0.043, 0.031, 0.9)
const COASTAL_BADGE_R_SC := 0.55
## Reference's settlement-label fill (`#f6ecd4`, line 15206) -- close enough
## to `LABEL_STROKE_COLOR` below's own outline colour (`rgba(8,6,4,.85)`,
## line 15198) that this control's existing region-label palette already
## matches it; only the fill needed a name of its own.
const SETTLEMENT_LABEL_FILL := Color(0.965, 0.925, 0.831)
## By `way_type` (`cartalith_civ::WayType`, peak-corridor-usage
## classification, Phase 2 milestone 14) -- a highway should read as more
## prominent than a track, the same "tier implies visual weight" principle
## `TIER_RADIUS` already applies to settlements.
const ROAD_COLOR := Color(0.36, 0.29, 0.16, 0.55)
## `ancient` is not a `WayType`: it is the fourth `ManualWayType`, arriving
## here since `get_roads()` began appending hand-drawn ways (IN-02). The
## reference strokes it at the same 1.1×rsc it gives `track` (line ~15516),
## so it takes that width rather than this dictionary's 1.6 `road` default.
## Its distinct grey dashed *colour* there is not reproduced -- this control
## strokes every land way in `ROAD_COLOR` regardless of type, which predates
## and is unaffected by this addition.
const ROAD_WIDTH_BY_TYPE := {"highway": 2.6, "regional": 2.0, "road": 1.6, "track": 1.1, "ancient": 1.1}
const MARKER_OUTLINE := Color(0.101, 0.070, 0.023, 0.85) ## matches PrimaryButton's ink tone
const HOVER_RADIUS_PAD := 4.0 ## extra hit-test slack (px) beyond the drawn marker radius

## Sea-lane style: reference's own convention (reference HTML line ~15511)
## is a dark navy solid underlayer plus a lighter dashed overlay -- not the
## `ROAD_COLOR`/`ROAD_WIDTH_BY_TYPE` land-road styling, so sea routes read
## as visually distinct (shipping lanes, not roads) at a glance.
const SEA_ROUTE_UNDERLAY := Color(0.039, 0.118, 0.235, 0.4)
const SEA_ROUTE_UNDERLAY_WIDTH := 1.5
const SEA_ROUTE_DASH_COLOR := Color(0.118, 0.510, 0.784, 0.7)
const SEA_ROUTE_DASH_WIDTH := 0.85
const SEA_ROUTE_DASH_LENGTH := 2.6

## §4.5.4's Route tool: a committed *route* is not infrastructure. It is a
## solved journey across the existing network (`route_commit` ->
## `civ_join_dijkstra_segs`, mixed land/sea), and the reference draws it as
## its own layer on top of the ways it follows (`drawCivLayer` block 2b,
## reference HTML lines 15552-15560, `civJourneys.forEach`) -- a dark
## underlayer stroke, then a dashed amber overlay. These four constants are
## that block's own unselected-journey values converted to `Color`:
## `rgba(40,25,5,.5)` at `lineWidth 3`, then `rgba(200,160,60,.85)` at
## `lineWidth 1.5` with `setLineDash([5,3])`.
##
## Reusing `ROAD_COLOR` would make a route invisible, which is exactly what
## happened before this existed: a committed 578 km route drew nothing at all
## (`GUI_GAP_REGISTER.md` IN-09). Selection state (the reference's brighter,
## thicker `sel` branch) has no counterpart here yet -- there is no route
## selection in this shell, only a read-only list.
const MANUAL_ROUTE_UNDERLAY := Color(0.157, 0.098, 0.020, 0.5)
const MANUAL_ROUTE_UNDERLAY_WIDTH := 3.0
const MANUAL_ROUTE_COLOR := Color(0.784, 0.627, 0.235, 0.85)
const MANUAL_ROUTE_WIDTH := 1.5
const MANUAL_ROUTE_DASH := 5.0
const MANUAL_ROUTE_GAP := 3.0

## §4.5.5's Icon tool markers, by `icon_dict`'s `family` key
## (`cartalith_assets::manual::ManualIconFamily::key()`). No texture atlas
## from the asset pack is wired into Godot yet (`icon_bridge.rs`'s art is
## rasterised only into the baked terrain texture, never exposed as
## individually-addressable sprites here) -- these are honest placeholder
## glyphs distinguishing family and marking real placed positions, not a
## stand-in for the pack's actual per-slot art.
const ICON_FAMILY_COLORS := {
	"settlement": Color(0.835, 0.369, 0.0),
	"feature": Color(0.0, 0.620, 0.451),
	"poi": Color(0.941, 0.894, 0.259),
	"custom": Color(0.337, 0.706, 0.914),
}
const ICON_BASE_RADIUS := 5.5
const ICON_OUTLINE := Color(0.051, 0.043, 0.031, 0.9)

## §4.5.5's Label tool. `color`/`font` are always the label's *effective*
## value (`label_dict` calls `color_or_default`/`font_or_default`), so no
## further fallback is needed here. `font` is a CSS font-family list (e.g.
## `"Georgia, serif"`) -- there is no web-font fallback chain in Godot, so
## the theme's own default font is used regardless of that string; only
## size/angle/arc/color are real per-label rendering.
const LABEL_STROKE_COLOR := Color(0.031, 0.024, 0.016, 0.8)
const LABEL_ZOOM_BASE_PX_PER_CELL := 2.0 ## tuning constant, see `_label_font_px`
const LABEL_FONT_PX_MIN := 8.0
const LABEL_FONT_PX_MAX := 96.0

## Emitted whenever the hovered settlement changes -- `null` on hover-exit.
## Lets `main.gd`'s Sample dock show the same data this overlay's own
## `_draw_hover_card` already draws on-canvas, without duplicating the
## hit-test logic in `_gui_input` below.
## `index` is the position in `get_settlements()`'s own array (`-1` on
## hover-exit) -- the same index `WorldGen.explain_settlement()` keys on, so
## the dock can ask why that settlement is there without re-running any hit
## test.
signal settlement_hovered(data: Variant, index: int)

## DCC shell milestone 1 (DCC_SHELL_SCOPE.md), click-to-pin (GUI_FEATURE_
## PARITY_SCOPE.md Category-1 item #10): emitted on a left click that hits
## a settlement, or on a click that hits nothing (`null`, `-1`) to unpin.
## Independent of `settlement_hovered` above -- the Properties dock holds
## this until the next click, unlike Sample's transient hover state.
signal settlement_selected(data: Variant, index: int)

## DCC shell milestone 1: emitted on every mouse motion with the grid-cell
## position under the cursor (`valid` false when the cursor is off the
## plate interior or nothing has been generated yet). Feeds the viewport's
## corner coordinate readout and the Sample dock -- real grid coordinates
## only, never a fabricated per-cell field (elevation/slope/biome) the
## engine doesn't expose per-cell yet.
signal cursor_sampled(gx: float, gy: float, valid: bool)

## §4.5's tool palette: the two primitives every armed tool needs, and
## nothing more specific than that -- a click-placement tool (Settlement,
## POI, Icon, Label, a Way/Route waypoint) wants `map_clicked`; a
## drag-painting tool (Biome paint, Territory, a Sculpt/Freehand stroke)
## wants `map_dragged`, which fires on every motion sample while the left
## button is held so a caller can accumulate a stroke or dab a brush
## continuously. Both fire in real grid-cell coordinates, the same values
## `cursor_sampled` already computes -- `_grid_point()` below is the one
## place that math lives now, shared by all three signals.
##
## This control stays tool-agnostic on purpose: it always emits both the
## selection signals above AND these two, on every click/drag, regardless of
## which tool is armed. The dispatch decision -- "is Inspect armed, so treat
## this as a selection, or is Settlement armed, so treat it as a placement"
## -- belongs to whoever owns the armed-tool state (`DccApp`), not here. That
## keeps this file ignorant of the tool system entirely, the same boundary
## `cursor_sampled`'s own doc comment already draws for per-cell fields.
signal map_clicked(gx: float, gy: float)
signal map_dragged(gx: float, gy: float)
signal map_released(gx: float, gy: float, valid: bool)   ## LMB release, ends a drag gesture.

## The reference's `contextmenu` handler on `view` (line 25888) ->
## `_civCtxShow` (25857). `PARITY_AUDIT.md` §5 item 2: no `MOUSE_BUTTON_RIGHT`
## handler existed anywhere under `godot-project/`, which left the reference's
## only path to Move-viewer-to and Delete-nearest-place with no counterpart.
##
## Stays as tool-agnostic as the three primitives above: this control reports
## "a right click landed at this grid cell, and the nearest settlement to it
## is `hit`", and nothing about what should happen next. The reference does
## its own nearest-place hit test inside the handler with the *same* radius
## `_civSelectPlaceAt` uses; here `_hit_test_settlement` already is that one
## shared definition, so the hit travels with the signal rather than being
## re-derived by whoever builds the menu.
##
## `screen_pos` is this control's local position, which is what a `PopupMenu`
## needs converted to global -- the receiver owns that conversion, since only
## it knows which window it is popping into.
signal map_right_clicked(gx: float, gy: float, hit: int, screen_pos: Vector2)

# ── Touch: press-and-hold IS the right click ─────────────────────────────────
#
# A finger has no second button, so on Android the context menu above had no
# route at all -- the one capability from the civ-interaction pass that phone
# could not reach by any path (`GUI_GAP_REGISTER.md`, and the phone canvas's
# own TARGETS rule says nothing about it because right click is not a phone
# gesture). Press-and-hold is the platform's answer, and this is where it
# belongs: the same control that owns the right click owns its touch twin, so
# `civilization_workspace.gd` receives one signal and never learns which
# pointer produced it.
#
# **The hard part is not the timer, it is the click that already fired.**
# `input_devices/pointing/emulate_mouse_from_touch` is on (project.godot), so a
# finger-down arrives here as a left `InputEventMouseButton` *immediately* --
# and `_gui_input`'s press branch emits `map_clicked` on press, which with the
# Settlement tool armed drops a settlement. Holding to open a menu would place
# a town first and then offer to edit a different one. So a touch press is
# **withheld** until the gesture says what it is:
#
#   drifts past `_TOUCH_SLOP`  -> it was a drag: release the press now, so the
#                                 sculpt/paint stroke starts from its real origin
#   lifts before the deadline  -> it was a tap: release the press, then release
#   reaches the deadline       -> it was a hold: discard the press entirely and
#                                 emit `map_right_clicked`; the lift is swallowed
#
# Driven off the *emulated mouse* stream rather than `InputEventScreenTouch`,
# deliberately: the emulated events are the ones this control is already known
# to receive, and `device < 0` (`InputEvent.DEVICE_ID_EMULATION`) is Godot's own
# marker for them, so a real mouse -- every desktop run -- takes none of this
# path and behaves exactly as it did.
#
# `OS.has_feature("mobile")` is ORed in as a second, coarser gate. Both were
# needed to get this working the first time and neither was individually
# provable on the handset without a build cycle per guess, so both stayed: the
# device id is the precise signal, the feature flag is the guarantee that an
# Android build takes this path even if a future engine stamps its emulated
# events differently. A physical mouse plugged into an Android device is the
# one case the flag over-claims, and it still resolves correctly -- the press
# is released on the lift or the first motion, one frame later than it would
# have been.
const _TOUCH_HOLD_MS := 500
## Physical pixels, not dp: this control is laid out in the main viewport,
## which carries no content scale. 28 px is ~10 dp on a 400 ppi handset, which
## is a finger's idle wobble and comfortably under the distance a deliberate
## drag covers in half a second.
const _TOUCH_SLOP := 28.0

var _touch_armed := false        ## a withheld press is outstanding
var _touch_swallow_up := false   ## the hold fired; the coming lift is not a release
var _touch_ms := 0
var _touch_pos := Vector2.ZERO
var _touch_press: Dictionary = {}

var _settlements: Array = []
var _roads: Array = []
var _sea_routes: Array = []
var _gw := 0
var _gh := 0
var _hover_index := -1
## Layer-granularity split (GUI_FEATURE_PARITY_SCOPE.md Category-1 item
## #9): the old shell had one checkbox hiding this whole control (and with
## it, hover input) -- these three flags let Settlements/Roads/Sea routes
## toggle independently while the control itself, and hover/click input on
## settlements, stay live regardless of which are on.
var _show_settlements := true
var _show_roads := true
var _show_sea_routes := true
## Per-class / per-way-type filters -- the reference's own
## `#explSettlementFilterList` and the by-way-type half of `#explShowRoads`
## (`design/Cartalith Menu Structure v2.dc.html`, MAP > LAYERS). Stored as
## *hidden* sets rather than shown sets so an empty dictionary means "show
## everything", which is what an untouched shell and a freshly-loaded world
## both are; a shown-set would have to be seeded from a settlement roster
## that does not exist until the first generate.
##
## Purely a draw filter: `_hit_test`/hover/click still see every settlement,
## the same independence `_show_settlements` above already keeps for the
## whole layer, because hiding a class is a cartographic choice and should
## not make a place unselectable.
var _hidden_settlement_kinds: Dictionary = {}
var _hidden_way_types: Dictionary = {}
## Plate-frame width as a fraction of the terrain texture's own width
## (`WorldGen.get_border_inset_frac()`, Phase 3 milestone 4). `0.0` when the
## renderer draws no frame, which makes every use of it below an exact no-op.
var _border_frac := 0.0

## The live camera zoom (`ViewportHost.zoom()`/`_zoom`), pushed in by
## `ViewportHost._zoom_at()`/`reset_view()` every time it changes --
## `PIN_SCALE_REF_PX`'s own doc comment covers why this is needed at all.
## Default `1.0` matches `ViewportHost`'s own default zoom, so a pin looks
## right even in the one frame before the first zoom/reset call ever runs.
var _camera_zoom := 1.0

func set_camera_zoom(z: float) -> void:
	if is_equal_approx(z, _camera_zoom):
		return
	_camera_zoom = z
	## Unlike `set_civ_data`/`set_show_settlements`, nothing about `_draw()`'s
	## own *content* changed here -- only how big the already-drawn geometry
	## needs to be next time, which only a fresh `_draw()` call can apply.
	## Godot does not re-run a `CanvasItem`'s draw commands just because an
	## ancestor's `scale` changed; it only rescales the cached ones, which is
	## exactly the bug `PIN_SCALE_REF_PX`'s own doc comment fixes -- without
	## this `queue_redraw()`, `_civ_zoom_k()` would compute the right radius
	## but never actually get drawn with it until some unrelated redraw
	## happened to fire.
	queue_redraw()

## The reference's own `_civZoomK()` (reference line 14980-14983), applied to
## `sc` so a settlement pin holds a roughly constant ON-SCREEN size across
## camera zoom -- see `PIN_SCALE_REF_PX`'s own doc comment for the full
## derivation of why this term is needed at all.
func _civ_zoom_k() -> float:
	return 1.0 / clampf(_camera_zoom, 0.35, 5.0)

## §4.5.5's Icon and Label tools place these; `bridge.icon_list()`/
## `bridge.label_list()` are this data's only source, both already bound
## (`icon_bridge.rs`/`label_bridge.rs`) and both wrapped in `engine_bridge.gd`.
## Set by `ViewportHost.refresh_annotations()`, a lighter call than the full
## `refresh()` -- placing one icon shouldn't re-fetch the terrain texture.
var _manual_icons: Array = []
var _labels: Array = []

## §4.5.4's Route tool. Each entry is one `route_get(i)` dictionary --
## `{points: PackedVector2Array, brks: PackedInt32Array, km, mode,
## unreachable_legs}` -- so `brks` is honoured exactly the way `_sea_routes`'
## own breaks are: a break ends one stroke and starts the next rather than
## drawing a straight line across the gap. Its own array rather than a third
## entry in `set_civ_data` because a committed route is not part of
## `get_roads()`/`get_sea_routes()` at all (`route_commit` stores into
## `InfraTools::routes`, a separate list from `InfraTools::ways` that
## `GUI_GAP_REGISTER.md` IN-02's fix appended to the two network getters).
var _manual_routes: Array = []

func set_manual_icons(icons: Array) -> void:
	_manual_icons = icons
	queue_redraw()

func set_labels(labels: Array) -> void:
	_labels = labels
	queue_redraw()

func set_manual_routes(routes: Array) -> void:
	_manual_routes = routes
	queue_redraw()


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_STOP
	resized.connect(func(): queue_redraw())


## Called by `main.gd` right after a successful `generate()`/
## `generate_world_structure()`. `settlements` is `WorldGen.get_settlements()`'s
## `Array[Dictionary]`, `roads` is `get_roads()`'s `Array[Dictionary]` --
## each entry `{points: PackedVector2Array, brks: PackedInt32Array,
## way_type: String, name: String}` (Phase 2 milestone 14's own
## consolidated/smoothed/classified output, not raw grid-cell indices --
## `points` are already continuous full-resolution coordinates, drawn via
## `_point_to_screen`, distinct from `_cell_to_screen`'s settlement-marker
## `+0.5` cell-centering, which would be wrong here). `sea_routes` is
## `get_sea_routes()`'s `Array[Dictionary]` -- same `{points, brks, name}`
## shape minus `way_type` (Phase 2 milestone 13, `SeaRoute` has no
## highway/regional/road/track tier). Both also carry `km: float` and
## `manual: bool`; this control reads neither. `manual` marks a way the user
## drew with the Way tool (`GUI_GAP_REGISTER.md` IN-02) and is deliberately
## NOT consulted while drawing -- the reference keeps hand-drawn and
## generated ways in one array and styles both by `way_type` alone, so a
## hand-drawn `road` is meant to be indistinguishable from a generated one.
## It is there for lists and filters, not for this `_draw()`.
## Screen-space conversion happens every
## frame from the current control size, so this stays correct across
## window resizes without needing to be told again.
## `border_frac` is `WorldGen.get_border_inset_frac()` -- the plate frame the
## terrain raster itself now carries, as a fraction of texture width. Passed
## in rather than hardcoded here because `render.rs` owns that geometry (see
## `_interior_rect`); defaults to `0.0` so a caller that doesn't know about
## the frame simply gets the old, uninset behaviour.
func set_civ_data(settlements: Array, roads: Array, sea_routes: Array, gw: int, gh: int, border_frac: float = 0.0) -> void:
	_settlements = settlements
	_roads = roads
	_sea_routes = sea_routes
	_gw = gw
	_gh = gh
	_border_frac = border_frac
	_hover_index = -1
	## A settlement roster this control is handed afresh invalidates every
	## cached town layout by index -- see `clear_urban_layouts()`.
	clear_urban_layouts()
	queue_redraw()


func set_show_settlements(shown: bool) -> void:
	_show_settlements = shown
	queue_redraw()


func set_show_roads(shown: bool) -> void:
	_show_roads = shown
	queue_redraw()


func set_show_sea_routes(shown: bool) -> void:
	_show_sea_routes = shown
	queue_redraw()


## One settlement tier (`capital`/`city`/`town`/`village`/`hamlet` -- the
## engine's own five, `get_settlements()`'s `kind`). Independent of
## `set_show_settlements`, which gates the whole layer: turning the layer on
## restores whatever per-class state was last set, matching how the
## reference's own filter popover and its master Settlements checkbox behave.
func set_settlement_kind_visible(kind: String, shown: bool) -> void:
	if shown:
		_hidden_settlement_kinds.erase(kind)
	else:
		_hidden_settlement_kinds[kind] = true
	queue_redraw()


## One way type (`road`/`track`/`sea_lane`/`ancient` -- the engine's own four,
## `infra_tools_bridge::parse_way_type`). Sea lanes are drawn from
## `_sea_routes`, not `_roads`, so this filter only ever reaches the three
## land types in practice; it is keyed by the same `way_type` string
## `get_roads()` returns rather than a separate vocabulary.
func set_way_type_visible(way_type: String, shown: bool) -> void:
	if shown:
		_hidden_way_types.erase(way_type)
	else:
		_hidden_way_types[way_type] = true
	queue_redraw()


## Reproduces `%MapView`'s own `STRETCH_KEEP_ASPECT_CENTERED` fit math so
## grid-cell coordinates map onto the same pixels the terrain texture
## actually occupies (this control's `size` is identical to `%MapView`'s,
## both being plain siblings under one `MarginContainer` -- see this
## script's own top comment). Returns `Rect2(origin, displayed_size)`, or
## a zero rect if there's nothing to fit yet.
func _displayed_rect() -> Rect2:
	if _gw <= 0 or _gh <= 0 or size.x <= 0.0 or size.y <= 0.0:
		return Rect2()
	var scale := minf(size.x / float(_gw), size.y / float(_gh))
	var displayed_size := Vector2(_gw, _gh) * scale
	var origin := (size - displayed_size) * 0.5
	return Rect2(origin, displayed_size)

## Public alias of the above -- `ViewportHost`'s tool overlays (the Measure
## ruler, the Region marquee) need the exact same fit rect this control's own
## drawing already computes, and duplicating the letterbox math a second place
## would be the kind of drift `js_hypot`-style bugs come from. Grid-to-screen,
## not the reverse -- see `_grid_point()` for the inverse.
func displayed_rect() -> Rect2:
	return _displayed_rect()


## The plate *interior*: `_displayed_rect()` minus the frame the terrain
## raster draws over its own outermost cells (paper margin + thick and thin
## neatlines, `render.rs`'s `apply_border`, Phase 3 milestone 4). Everything
## outside this is bare paper with no map under it at all -- the terrain
## there is covered, not shown -- so nothing drawn from world data belongs
## on it.
##
## The frame is inset by the same number of *cells* on all four sides and
## `_displayed_rect()`'s fit scale is uniform, so one pixel inset serves both
## axes. `_border_frac` is a fraction of texture width rather than a cell
## count precisely so this needs no resolution knowledge.
func _interior_rect(rect: Rect2) -> Rect2:
	if _border_frac <= 0.0:
		return rect
	var inset := minf(_border_frac * rect.size.x, minf(rect.size.x, rect.size.y) * 0.45)
	return rect.grow(-inset)


func _cell_to_screen(cell: Vector2, rect: Rect2) -> Vector2:
	return rect.position + Vector2((cell.x + 0.5) / _gw, (cell.y + 0.5) / _gh) * rect.size


## Roads' own `points` are already continuous full-resolution coordinates
## (Catmull-Rom-smoothed, not raw cell indices) -- no `+0.5` centering,
## unlike `_cell_to_screen`'s settlement markers.
func _point_to_screen(p: Vector2, rect: Rect2) -> Vector2:
	return rect.position + Vector2(p.x / _gw, p.y / _gh) * rect.size


## True below `kind`'s own `SETTLEMENT_LOD` threshold -- the raw camera
## zoom, not `_civ_zoom_k()`'s clamped/inverted screen-size compensation,
## matching the reference's own `zoom<lodMin` test (`zoom` there is
## `_civZoomRaw()`, the un-clamped `viewT.scale`). An unrecognised kind
## defaults to `0.5` (town/village straddle), same fallback
## `CIV_LOD_PLACE[p.kind]!=null?...:0.5` uses in the reference.
func _settlement_below_lod(kind: String) -> bool:
	return _camera_zoom < float(SETTLEMENT_LOD.get(kind, 0.5))


## `(4+klass.rank)*sc` -- the reference's own settlement-pin size formula
## (`_civDrawSettlementPin`, line 15166), `sc` per `PIN_SCALE_REF_PX`'s own
## doc comment above. Shared by `_draw()`'s settlement loop and `_hit_test_
## settlement` so the drawn pin and its click/hover target never disagree.
## Below the tier's own zoom threshold this instead returns the small
## fallback-dot radius (`LOD_DOT_RADIUS_SC`) -- the pin `_draw()` actually
## renders at low zoom is the dot, so the hit target must shrink with it,
## the same "drawn size is picked size" invariant this function already
## keeps for the full pin.
func _settlement_pin_radius(kind: String, rect: Rect2) -> float:
	var sc: float = (rect.size.x / PIN_SCALE_REF_PX) * _civ_zoom_k()
	if _settlement_below_lod(kind):
		return LOD_DOT_RADIUS_SC * sc
	var klass: Dictionary = SETTLEMENT_CLASS.get(kind, SETTLEMENT_CLASS["town"])
	return (4.0 + float(klass["rank"])) * sc


## Reference lines 15716-15721 (`lblCandidates`): the four label positions a
## settlement name is tried at, above -> below -> right -> left, as screen-
## space boxes centred on the pin. `gap` matches the reference's own `2*sc`
## clearance between the pin edge and the label box.
func _settlement_label_candidates(pos: Vector2, radius: float, sc: float, w: float, h: float) -> Array[Rect2]:
	var gap := 2.0 * sc
	return [
		Rect2(pos - Vector2(w / 2.0, radius + gap + h), Vector2(w, h)),  # above
		Rect2(pos + Vector2(-w / 2.0, radius + gap), Vector2(w, h)),     # below
		Rect2(pos + Vector2(radius + gap, -h / 2.0), Vector2(w, h)),     # right
		Rect2(pos - Vector2(radius + gap + w, h / 2.0), Vector2(w, h)),  # left
	]


## Pre-seeds this frame's label-occupancy set with every manual icon's and
## user-authored label's own approximate footprint, so a settlement's
## auto-placed name never overlaps annotation the user placed deliberately
## -- the same reasoning the reference reserves region-label boxes before
## its own settlement loop runs for (line 15722-15728: "user-placed region
## names are DELIBERATE cartography -- they must not be silently suppressed
## by an auto-placed settlement label"). Approximate rather than exact for
## icons (whose real footprint depends on family/shape, drawn in
## `_draw_manual_icons`, not recomputed here) -- close enough to keep a
## label from visibly overlapping an icon, which is all this simplified
## system claims to do; see `_draw()`'s own settlement-loop comment for the
## full scope of the simplification against the reference's real occupancy
## grid (`_civLblOcc`).
func _seed_label_occupancy(rect: Rect2) -> Array[Rect2]:
	var boxes: Array[Rect2] = []
	var font := get_theme_default_font()
	for lb: Dictionary in _labels:
		var text: String = lb["text"]
		if text.is_empty():
			continue
		var pos := _point_to_screen(Vector2(lb["x"], lb["y"]), rect)
		var font_px := _label_font_px(lb, rect)
		var w := font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px).x
		var h := float(font_px) * 1.3
		boxes.append(Rect2(pos - Vector2(w, h) / 2.0, Vector2(w, h)))
	for ic: Dictionary in _manual_icons:
		var pos2 := _point_to_screen(Vector2(ic["x"], ic["y"]), rect)
		var r: float = ICON_BASE_RADIUS * maxf(0.2, float(ic["scale"]))
		boxes.append(Rect2(pos2 - Vector2(r, r), Vector2(r, r) * 2.0))
	return boxes


func _draw() -> void:
	if (_settlements.is_empty() and _roads.is_empty() and _sea_routes.is_empty()
			and _manual_icons.is_empty() and _labels.is_empty()
			and _manual_routes.is_empty()):
		return
	var rect := _displayed_rect()
	if rect.size.x <= 0.0:
		return
	var interior := _interior_rect(rect)

	# Linear features (roads, sea lanes) are *clipped* at the neatline: a
	# road that runs off the plate genuinely continues past the sheet edge,
	# and cutting it there is what an atlas plate does. Point symbols are
	# handled the opposite way below -- placed or not placed, never sliced.
	#
	# One scissor rect for the whole canvas item rather than hand-clipping
	# four different primitive types. `Control` re-sets both of these from
	# its own rect on every `NOTIFICATION_DRAW`, which fires immediately
	# before `_draw()`, so this override lasts exactly one frame and needs
	# no restore.
	if _border_frac > 0.0:
		var ci := get_canvas_item()
		RenderingServer.canvas_item_set_custom_rect(ci, true, interior)
		RenderingServer.canvas_item_set_clip(ci, true)

	if _show_sea_routes:
		for route: Dictionary in _sea_routes:
			var points: PackedVector2Array = route["points"]
			if points.size() < 2:
				continue
			var brks: PackedInt32Array = route["brks"]
			var start := 0
			for cut in brks:
				_draw_sea_route_segment(points, start, cut, rect)
				start = cut
			_draw_sea_route_segment(points, start, points.size(), rect)

	if _show_roads:
		for way: Dictionary in _roads:
			var points: PackedVector2Array = way["points"]
			if points.size() < 2:
				continue
			if _hidden_way_types.has(way["way_type"]):
				continue
			var width: float = ROAD_WIDTH_BY_TYPE.get(way["way_type"], 1.6)
			var brks: PackedInt32Array = way["brks"]
			# `brks` marks indices where this way's own path has a real gap
			# (two disjoint consolidated runs sharing one `Way`) -- draw each
			# run between breaks as its own stroke, not one polyline straight
			# through the gap.
			var start2 := 0
			for cut in brks:
				_draw_way_segment(points, start2, cut, rect, width)
				start2 = cut
			_draw_way_segment(points, start2, points.size(), rect, width)

	## Committed Route-tool routes, drawn after both network layers so a route
	## that runs along an existing road is still visible on top of it. Shares
	## the "Ways & routes" visibility toggle (`set_show_roads`) because that is
	## the layer row the CARTO dock actually labels "Ways & routes" -- there is
	## no separate routes checkbox to gate against.
	if _show_roads:
		for r: Dictionary in _manual_routes:
			var rpts: PackedVector2Array = r.get("points", PackedVector2Array())
			if rpts.size() < 2:
				continue
			var rbrks: PackedInt32Array = r.get("brks", PackedInt32Array())
			var rstart := 0
			for cut in rbrks:
				_draw_manual_route_segment(rpts, rstart, cut, rect)
				rstart = cut
			_draw_manual_route_segment(rpts, rstart, rpts.size(), rect)

	## Town layouts sit above the ways -- a town's own high street IS the
	## through-road, so it must overlay it -- and *replace* the pin of every
	## place they actually draw (`_urban_revealed`, the reference's
	## `_umRevealedSet`). See this file's "Urban layouts" block at the foot
	## for the reveal gate and why it is not `_umLayoutAlpha`'s km band.
	_urban_revealed.clear()
	if _show_urban_layouts:
		_draw_urban_layouts(rect, interior)

	if _show_settlements:
		## Same formula `_settlement_pin_radius()` uses -- kept as one inline
		## `sc` here (rather than calling that function per-settlement) since
		## `_draw()`'s loop already reuses `sc` for the glyph and label sizing
		## below it too, exactly like the reference's own `sc` feeds icon,
		## way and label sizing from one shared value (reference line 15165).
		var sc: float = (rect.size.x / PIN_SCALE_REF_PX) * _civ_zoom_k()
		var font := get_theme_default_font()
		# Trait badges (§4.5.3's own reference behaviour, `_civDrawTraitBadges`,
		# reference line 15101) are a disclosed gap, not an oversight: `get_
		# settlements()` (`lib.rs`) emits {x, y, name, population, kind,
		# faction, capital, coastal} only -- no `traits` field -- and nothing
		# in `cartalith-civ`'s own `NamedSettlement` models a trait list at
		# all (grepped: the string "traits" appears exactly once in that
		# crate, in a doc comment quoting the REFERENCE's own JS object
		# shape). There is no data to draw a badge from, so none are drawn.
		#
		# Auto-label placement below is a deliberately simplified stand-in
		# for the reference's real system (`_civLblOcc`, an occupancy GRID
		# tested/marked per label-sized bucket, reference lines 15668-15781):
		# a plain per-frame `Array[Rect2]` of already-placed boxes, tested by
		# `Rect2.intersects` rather than a spatial grid. Fine at settlement-
		# roster scale (dozens to a few hundred -- an occupancy grid exists to
		# make THOUSANDS cheap, which no generated world here produces), and
		# it reproduces the essential behaviour: higher tiers are drawn (and
		# so win label placement) first, the same four candidate positions in
		# the same above/below/right/left order, and a label that fits
		# nowhere is dropped -- but its pin is always still drawn (once past
		# its own `SETTLEMENT_LOD` threshold -- see the dot-fallback branch
		# below). This is a different LOD from `LOD_TILING_INTEGRATION_SCOPE
		# .md` milestone M1 (that one raster-tiles the TERRAIN at deep zoom;
		# this one is `CIV_LOD_PLACE`, the reference's own zoom-gated
		# settlement-pin importance tiering, owner-requested 2026-08-19).
		var occupied: Array[Rect2] = _seed_label_occupancy(rect)
		var draw_order := range(_settlements.size())
		draw_order.sort_custom(func(a, b):
			var ra: int = SETTLEMENT_CLASS.get(_settlements[a]["kind"], SETTLEMENT_CLASS["town"])["rank"]
			var rb: int = SETTLEMENT_CLASS.get(_settlements[b]["kind"], SETTLEMENT_CLASS["town"])["rank"]
			return ra > rb)

		for i in draw_order:
			var s: Dictionary = _settlements[i]
			## Per-class filter, tested before any geometry so a hidden tier
			## costs nothing and, more importantly, never reserves label
			## occupancy that a *visible* place would then be pushed out of.
			if _hidden_settlement_kinds.has(s["kind"]):
				continue
			## The reference's `_umRevealedSet` (line 22753): a place whose own
			## generated layout was actually drawn this frame gives up its pin
			## to it. The reference crossfades the two across its km band; with
			## no band here (see the "Urban layouts" block for why) this is the
			## end state of that fade, applied at the same moment. Without it
			## the pin -- deliberately sized to hold constant on screen -- sits
			## squarely over the market anchor and the densest streets, which
			## is exactly what it is drawn on top of.
			if _urban_revealed.has(i):
				continue
			var pos := _cell_to_screen(Vector2(s["x"], s["y"]), rect)
			# A settlement whose cell is under the frame has no visible terrain
			# beneath it at all, so a marker there points at nothing -- it is off
			# the plate, and off-plate detail is omitted rather than trimmed to a
			# half-disc against the neatline. The clip above then trims the one
			# remaining case: a settlement just *inside* the interior whose
			# radius overhangs it (the actual defect this fixes -- markers
			# landing partly on the margin, seen in both test worlds).
			if not interior.has_point(pos):
				continue
			var faction: int = s["faction"]
			var color: Color = FACTION_COLORS[(faction - 1) % FACTION_COLORS.size()] if faction > 0 else Color(0.5, 0.5, 0.5)
			var kind: String = s["kind"]

			# `CIV_LOD_PLACE` (reference line 15373, see `SETTLEMENT_LOD`'s own
			# doc comment above for the full derivation and real-world-mapping
			# citations): below this tier's own zoom threshold, draw a small
			# faction-tinted dot instead of the full pin -- never hide the
			# place outright (a road still needs a visible anchor at any
			# zoom), but keep a low-zoom view from drowning in city-sized
			# hamlet icons and labels. No glyph, no name label, no hover-
			# radius bump, no capital ring -- all reference behaviour for the
			# dot branch (reference lines 15747-15756 draw nothing else for
			# it either).
			if _settlement_below_lod(kind):
				var dot_r: float = LOD_DOT_RADIUS_SC * sc
				draw_circle(pos, dot_r + LOD_DOT_OUTLINE_SC * sc, Color(0.047, 0.039, 0.027, 0.65), true, -1.0, true)
				draw_circle(pos, dot_r, color, true, -1.0, true)
				continue

			var klass: Dictionary = SETTLEMENT_CLASS.get(kind, SETTLEMENT_CLASS["town"])
			var radius: float = (4.0 + float(klass["rank"])) * sc
			if i == _hover_index:
				radius += 1.5

			## `antialiased` (the 6th positional arg) defaults to `false` in
			## Godot 4 -- left implicit here would draw a visibly jagged
			## circle, worse the more a pin is magnified by camera zoom
			## (`PIN_SCALE_REF_PX`'s own doc comment covers the zoom side of
			## "not sharp"; this is the antialiasing side). `filled=true,
			## width=-1.0` spelled out are just `draw_circle`'s own defaults,
			## kept explicit because GDScript has no keyword-argument syntax
			## to set only the trailing one.
			##
			## Shadow first (drawn behind, offset straight down) so the fill
			## and outline composite over it exactly like the reference's own
			## layering, just with one extra pass underneath.
			draw_circle(pos + Vector2(0, PIN_SHADOW_OFFSET_SC * sc), radius, PIN_SHADOW_COLOR, true, -1.0, true)
			draw_circle(pos, radius, color, true, -1.0, true)
			draw_arc(pos, radius, 0, TAU, 24, MARKER_OUTLINE, 1.2, true)
			if s["capital"]:
				draw_arc(pos, radius + CAPITAL_RING_WIDTH, 0, TAU, 28, color, CAPITAL_RING_WIDTH, true)
			if s.get("coastal", false):
				var badge_r: float = COASTAL_BADGE_R_SC * sc
				var badge_pos := pos + Vector2(radius, radius) * 0.62
				draw_circle(badge_pos, badge_r + 0.5, COASTAL_BADGE_OUTLINE, true, -1.0, true)
				draw_circle(badge_pos, badge_r, COASTAL_BADGE_COLOR, true, -1.0, true)

			# Glyph (reference: `ctx.fillText(klass.glyph,px,py)`, line 15178-
			# 15180) -- the one per-tier visual distinguisher beyond size,
			# centred on the pin exactly like `_draw_labels`' own straight-text
			# path centres a region label (`v_center` trick, same reasoning).
			# Godot's built-in theme font may not carry every one of these five
			# dingbat glyphs (⌂●◉⬣✦) -- a missing one falls back to whatever
			# Godot's own tofu/replacement glyph is, same disclosed limitation
			# `_draw_labels`' own doc comment already accepts for `font` (no
			# web-font fallback chain exists in Godot either).
			var glyph: String = klass["glyph"]
			var glyph_px: int = maxi(8, int(radius + 2.0 * sc))
			var glyph_w := font.get_string_size(glyph, HORIZONTAL_ALIGNMENT_LEFT, -1, glyph_px).x
			var glyph_v_center: float = (font.get_ascent(glyph_px) - font.get_descent(glyph_px)) / 2.0
			draw_string(font, pos + Vector2(-glyph_w / 2.0, glyph_v_center), glyph,
				HORIZONTAL_ALIGNMENT_LEFT, -1, glyph_px, Color.WHITE)

			# Auto-placed name label -- see this block's own top comment for
			# the simplified-occupancy-set reasoning.
			var name: String = s.get("name", "")
			if not name.is_empty():
				var label_px: int = maxi(9, int(radius + sc))
				var lw := font.get_string_size(name, HORIZONTAL_ALIGNMENT_LEFT, -1, label_px).x
				var lh := float(label_px) * 1.3
				for box in _settlement_label_candidates(pos, radius, sc, lw, lh):
					var fits := true
					for occ in occupied:
						if occ.intersects(box):
							fits = false
							break
					if not fits:
						continue
					occupied.append(box)
					var v_center: float = (font.get_ascent(label_px) - font.get_descent(label_px)) / 2.0
					var draw_pos := Vector2(box.position.x, box.position.y + box.size.y / 2.0 + v_center)
					var outline_w: int = maxi(1, int(2.5 * sc))
					draw_string_outline(font, draw_pos, name, HORIZONTAL_ALIGNMENT_LEFT, -1, label_px, outline_w, LABEL_STROKE_COLOR)
					draw_string(font, draw_pos, name, HORIZONTAL_ALIGNMENT_LEFT, -1, label_px, SETTLEMENT_LABEL_FILL)
					break

		if _hover_index >= 0 and _hover_index < _settlements.size():
			_draw_hover_card(_settlements[_hover_index], rect, interior)

	# Manual annotations (§4.5.5) are independent of the Settlements/Roads/Sea
	# routes toggles above -- they have no layer-visibility flag of their own
	# in `DCC_SHELL_SPEC.md`, so they always draw once placed, same as the
	# Measure/Region tool overlays in `tool_overlay.gd` always draw once armed.
	_draw_manual_icons(rect, interior)
	_draw_labels(rect, interior)


## §4.5.5's Icon tool: placed markers, by `family` (`icon_dict`'s
## `{x, y, family, slot, set, scale}`). Positions are continuous
## full-resolution coordinates (a placement click's own `gx, gy`, not a
## cell index) -- `_point_to_screen`, not `_cell_to_screen`, matching roads'
## own reasoning in this file's `set_civ_data` doc comment.
func _draw_manual_icons(rect: Rect2, interior: Rect2) -> void:
	for ic: Dictionary in _manual_icons:
		var pos := _point_to_screen(Vector2(ic["x"], ic["y"]), rect)
		if not interior.has_point(pos):
			continue
		var color: Color = ICON_FAMILY_COLORS.get(ic["family"], Color(0.7, 0.7, 0.7))
		var r: float = ICON_BASE_RADIUS * maxf(0.2, float(ic["scale"]))
		match ic["family"]:
			"settlement":
				var half := r * 0.85
				draw_rect(Rect2(pos - Vector2(half, half), Vector2(half, half) * 2.0), color, true)
				draw_rect(Rect2(pos - Vector2(half, half), Vector2(half, half) * 2.0), ICON_OUTLINE, false, 1.2)
			"feature":
				var pts := PackedVector2Array([
					pos + Vector2(0, -r), pos + Vector2(r * 0.87, r * 0.5), pos + Vector2(-r * 0.87, r * 0.5)])
				draw_colored_polygon(pts, color)
				draw_polyline(PackedVector2Array([pts[0], pts[1], pts[2], pts[0]]), ICON_OUTLINE, 1.2, true)
			"poi":
				var pts2 := PackedVector2Array([
					pos + Vector2(0, -r), pos + Vector2(r, 0), pos + Vector2(0, r), pos + Vector2(-r, 0)])
				draw_colored_polygon(pts2, color)
				draw_polyline(PackedVector2Array([pts2[0], pts2[1], pts2[2], pts2[3], pts2[0]]), ICON_OUTLINE, 1.2, true)
			_: ## "custom", or any future family this build doesn't recognise yet.
				draw_circle(pos, r, color, true, -1.0, true)   ## See the settlement pin's own antialiasing comment above.
				draw_arc(pos, r, 0, TAU, 20, ICON_OUTLINE, 1.2, true)
				draw_arc(pos, r * 0.4, 0, TAU, 12, ICON_OUTLINE, 1.0, true)


## §4.5.5's Label tool: user-authored region-name text, angled/arched in the
## label's own font/color. Ports the reference's `drawArcLabel` (reference
## HTML line ~15244) character-for-character -- same per-glyph placement on
## a circle of radius `R`, same `|arc| < 0.01` straight-line fast path --
## rather than approximating curved text, since a region label's curve is
## itself user-authored content (dragged into shape via the arc handle),
## not decoration.
func _draw_labels(rect: Rect2, interior: Rect2) -> void:
	if _labels.is_empty():
		return
	var font := get_theme_default_font()
	for lb: Dictionary in _labels:
		var text: String = lb["text"]
		if text.is_empty():
			continue
		var pos := _point_to_screen(Vector2(lb["x"], lb["y"]), rect)
		if not interior.has_point(pos):
			continue
		var font_px := _label_font_px(lb, rect)
		var fill: Color = Color(String(lb["color"]))
		var outline_w: int = maxi(1, int(font_px * 0.16))
		var v_center: float = (font.get_ascent(font_px) - font.get_descent(font_px)) / 2.0
		var th: float = deg_to_rad(float(lb["angle"]))
		var a: float = clampf(float(lb["arc"]), -1.0, 1.0)

		if absf(a) < 0.01:
			var full_w := font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px).x
			var local_pos := Vector2(-full_w / 2.0, v_center)
			draw_set_transform(pos, th, Vector2.ONE)
			draw_string_outline(font, local_pos, text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px, outline_w, LABEL_STROKE_COLOR)
			draw_string(font, local_pos, text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px, fill)
			continue

		var total_w := font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px).x
		var radius: float = maxf(font_px * 1.2, total_w / (2.2 * absf(a)))
		var dir_sign: float = 1.0 if a > 0.0 else -1.0
		var acc := -total_w / 2.0
		for ch in text:
			var w := font.get_string_size(ch, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px).x
			var mid := acc + w / 2.0
			var theta := mid / radius
			var glyph_local := Vector2(radius * sin(theta), dir_sign * radius * (1.0 - cos(theta)))
			var world_pt := pos + glyph_local.rotated(th)
			var local_pos2 := Vector2(-w / 2.0, v_center)
			draw_set_transform(world_pt, th + dir_sign * theta, Vector2.ONE)
			draw_string_outline(font, local_pos2, ch, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px, outline_w, LABEL_STROKE_COLOR)
			draw_string(font, local_pos2, ch, HORIZONTAL_ALIGNMENT_LEFT, -1, font_px, fill)
			acc += w
	draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)


## `size_mode == "fixed"` holds a constant on-screen size regardless of
## zoom (reference: drops its own `_civZoomK()` factor); `"zoom"` grows with
## the terrain, tied to the current px-per-cell fit (`rect.size.x / _gw`) the
## same way `tool_overlay.gd`'s brush cursor radius already scales. Clamped
## to stay legible at extreme zoom in either direction.
func _label_font_px(lb: Dictionary, rect: Rect2) -> int:
	var size: float = float(lb["size"])
	var px: float
	if lb["size_mode"] == "fixed":
		px = size
	else:
		px = size * (rect.size.x / float(_gw)) / LABEL_ZOOM_BASE_PX_PER_CELL
	return int(clampf(px, LABEL_FONT_PX_MIN, LABEL_FONT_PX_MAX))


## Draws `points[start:end]` (exclusive) as one stroke, converted to
## screen space. `end - start < 2` is a real, legitimate no-op (a run with
## a single point either side of a break contributes nothing to draw).
func _draw_way_segment(points: PackedVector2Array, start: int, end: int, rect: Rect2, width: float) -> void:
	if end - start < 2:
		return
	var screen_points := PackedVector2Array()
	screen_points.resize(end - start)
	for i in range(start, end):
		screen_points[i - start] = _point_to_screen(points[i], rect)
	draw_polyline(screen_points, ROAD_COLOR, width, true)


## Sea lane, reference's own two-pass style (reference HTML line ~15511):
## a solid dark-navy underlayer stroke first, then a lighter dashed overlay
## on top. The underlayer is one `draw_polyline` (phase doesn't matter, it's
## solid). The dash overlay is walked manually via `_draw_dashed_polyline`
## below, NOT one `draw_dashed_line` call per vertex pair -- a smoothed
## route's points are only a few px apart, shorter than one dash+gap cycle,
## so restarting the dash phase at every vertex left every segment landing
## inside the "on" portion and rendered as a solid line, not dashed (caught
## by the real-app screenshot verification this milestone requires, not
## assumed).
func _draw_sea_route_segment(points: PackedVector2Array, start: int, end: int, rect: Rect2) -> void:
	if end - start < 2:
		return
	var screen_points := PackedVector2Array()
	screen_points.resize(end - start)
	for i in range(start, end):
		screen_points[i - start] = _point_to_screen(points[i], rect)
	draw_polyline(screen_points, SEA_ROUTE_UNDERLAY, SEA_ROUTE_UNDERLAY_WIDTH, true)
	_draw_dashed_polyline(screen_points, SEA_ROUTE_DASH_COLOR, SEA_ROUTE_DASH_WIDTH, SEA_ROUTE_DASH_LENGTH)


## One committed Route-tool route, `points[start:end]` (exclusive). The
## reference's own two-pass journey stroke (block 2b, lines 15555-15559):
## solid dark underlayer first, dashed amber on top. Same structure as
## `_draw_sea_route_segment`, and dashed for the same reason it is -- the
## overlay walk keeps the dash phase continuous across vertices, which a
## per-vertex `draw_dashed_line` would not (see `_draw_dashed_polyline`).
func _draw_manual_route_segment(points: PackedVector2Array, start: int, end: int, rect: Rect2) -> void:
	if end - start < 2:
		return
	var screen_points := PackedVector2Array()
	screen_points.resize(end - start)
	for i in range(start, end):
		screen_points[i - start] = _point_to_screen(points[i], rect)
	draw_polyline(screen_points, MANUAL_ROUTE_UNDERLAY, MANUAL_ROUTE_UNDERLAY_WIDTH, true)
	_draw_dashed_polyline(screen_points, MANUAL_ROUTE_COLOR, MANUAL_ROUTE_WIDTH,
		MANUAL_ROUTE_DASH, MANUAL_ROUTE_GAP)


## Draws `points` as a dashed line with the dash phase carried continuously
## across every vertex -- unlike `draw_dashed_line` per-segment, a dash or
## gap can span a vertex instead of always restarting "on" there.
##
## `gap_len` defaults to `dash_len` (the equal on/off the sea-lane overlay
## has always used, unchanged). The Route layer passes an unequal pair
## because the reference's journey stroke is `setLineDash([5,3])`, not
## `[5,5]`.
func _draw_dashed_polyline(points: PackedVector2Array, color: Color, width: float, dash_len: float, gap_len: float = -1.0) -> void:
	if gap_len < 0.0:
		gap_len = dash_len
	var period := dash_len + gap_len
	var phase := 0.0
	for i in range(points.size() - 1):
		var p0 := points[i]
		var p1 := points[i + 1]
		var seg_vec := p1 - p0
		var seg_len := seg_vec.length()
		if seg_len <= 0.0:
			continue
		var dir := seg_vec / seg_len
		var traveled := 0.0
		while traveled < seg_len:
			var cycle_pos := fmod(phase, period)
			var on := cycle_pos < dash_len
			var remaining_in_state := (dash_len - cycle_pos) if on else (period - cycle_pos)
			# `remaining_in_state` is mathematically > 0 whenever the loop is
			# reached, but `phase` accumulates across every vertex of a long
			# route, and float drift can land `cycle_pos` close enough to a
			# `dash_len`/`period` boundary that subtraction rounds to exactly
			# 0.0 -- `step` then never advances `traveled`, spinning forever
			# and flooding the renderer's draw-command buffer until it
			# overflows (crashed a real run, not a hypothetical). Floor
			# `step` to a sub-pixel epsilon so every iteration makes forward
			# progress; the resulting overshoot is invisible.
			var step := maxf(minf(remaining_in_state, seg_len - traveled), 0.001)
			if on:
				draw_line(p0 + dir * traveled, p0 + dir * (traveled + step), color, width, true)
			traveled += step
			phase += step


func _draw_hover_card(s: Dictionary, rect: Rect2, interior: Rect2) -> void:
	var pos := _cell_to_screen(Vector2(s["x"], s["y"]), rect)
	var kind_label: String = String(s["kind"]).capitalize()
	var lines := ["%s (%s)" % [s["name"], kind_label], "Population %s" % _format_pop(s["population"])]
	var font := get_theme_default_font()
	var font_size := 13
	var line_h := font.get_height(font_size)
	var w := 0.0
	for line in lines:
		w = maxf(w, font.get_string_size(line, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x)
	var pad := 8.0
	var card_size := Vector2(w + pad * 2, line_h * lines.size() + pad * 2)
	var card_pos := pos + Vector2(10, -card_size.y - 10)
	# Clamped into the plate interior, not into this control's full bounds:
	# `_draw`'s scissor is still in force here, so a card clamped to the
	# control edge would be sliced by the neatline. Keeping it inside the
	# interior keeps it whole *and* keeps it on the map, which is where a
	# tooltip about map content belongs anyway.
	card_pos.x = clampf(card_pos.x, interior.position.x, maxf(interior.position.x, interior.end.x - card_size.x))
	card_pos.y = clampf(card_pos.y, interior.position.y, maxf(interior.position.y, interior.end.y - card_size.y))

	# Dark-viewport-compatible palette (GUI decluttering pass, 2026-08-17) --
	# was a light-parchment cream/brown card, the fourth stray light-styled
	# surface even though this control's independence from the Theme
	# resource is legitimate (map content, not chrome, see this file's own
	# top comment). Matches the shell's own surface/border/emphasis tokens
	# (`theme/dark_theme.tres`): near-black card fill, accent border,
	# emphasis-toned text.
	draw_rect(Rect2(card_pos, card_size), Color(0.051, 0.055, 0.059, 0.96), true)
	draw_rect(Rect2(card_pos, card_size), Color(0.878, 0.639, 0.290, 1.0), false, 1.5)
	for j in lines.size():
		var text_pos := card_pos + Vector2(pad, pad + line_h * (j + 1) - font.get_descent(font_size))
		draw_string(font, text_pos, lines[j], HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color(0.910, 0.922, 0.925))


func _format_pop(pop: int) -> String:
	if pop >= 1000:
		return "%.1fk" % (pop / 1000.0)
	return str(pop)


## Nearest settlement whose marker is within its own hit radius of `mouse`,
## or `-1`. Shared by hover (`_gui_input`'s motion branch) and click-to-pin
## (its button branch) so both use exactly one hit-test definition.
func _hit_test_settlement(mouse: Vector2, interior: Rect2, rect: Rect2) -> int:
	var closest := -1
	var closest_dist := INF
	for i in _settlements.size():
		var s: Dictionary = _settlements[i]
		var pos := _cell_to_screen(Vector2(s["x"], s["y"]), rect)
		# Same predicate `_draw` uses: an off-plate settlement has no
		# marker, so it must not have a hit target either -- otherwise a
		# hover/click would fill in dock data from what looks like blank
		# paper.
		if not interior.has_point(pos):
			continue
		var radius: float = _settlement_pin_radius(s["kind"], rect) + HOVER_RADIUS_PAD
		var d := mouse.distance_to(pos)
		if d <= radius and d < closest_dist:
			closest = i
			closest_dist = d
	return closest


## Returns `{valid, gx, gy}` -- the inverse of `_cell_to_screen`, shared by
## `cursor_sampled`, `map_clicked` and `map_dragged` so the coordinate math
## exists in exactly one place. `valid` is false outside the plate interior
## (bare paper, no world coordinate under it) or before anything has
## generated.
func _grid_point(mouse: Vector2, rect: Rect2, interior: Rect2) -> Dictionary:
	if _gw <= 0 or _gh <= 0 or not interior.has_point(mouse):
		return {"valid": false, "gx": 0.0, "gy": 0.0}
	return {
		"valid": true,
		"gx": (mouse.x - rect.position.x) / rect.size.x * _gw,
		"gy": (mouse.y - rect.position.y) / rect.size.y * _gh,
	}

func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var rect := _displayed_rect()
		if rect.size.x <= 0.0:
			cursor_sampled.emit(0.0, 0.0, false)
			return
		var interior := _interior_rect(rect)
		var mm := event as InputEventMouseMotion
		var mouse: Vector2 = mm.position

		var closest := _hit_test_settlement(mouse, interior, rect)
		if closest != _hover_index:
			_hover_index = closest
			queue_redraw()
			settlement_hovered.emit(_settlements[closest] if closest != -1 else null, closest)

		var p := _grid_point(mouse, rect, interior)
		cursor_sampled.emit(p["gx"], p["gy"], p["valid"])
		## A withheld touch press that has now travelled is a drag, not a hold:
		## let it through *before* the first `map_dragged`, so a stroke's origin
		## is the point the finger went down on rather than wherever it had got
		## to by the time the slop was exceeded.
		if _touch_armed and mouse.distance_to(_touch_pos) > _TOUCH_SLOP:
			_release_touch_press()
		if p["valid"] and (mm.button_mask & MOUSE_BUTTON_MASK_LEFT) != 0:
			map_dragged.emit(p["gx"], p["gy"])

	elif event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index == MOUSE_BUTTON_RIGHT:
			## Press, not release -- the reference opens its menu from
			## `contextmenu`, which fires on press. `accept_event()` so the
			## click cannot fall through to anything behind this control,
			## matching the reference's own `e.preventDefault()` ("the canvas
			## has no useful native menu; ours only opens in civ-capable
			## tabs").
			if not mb.pressed:
				return
			var r := _displayed_rect()
			if r.size.x <= 0.0:
				return
			var inter := _interior_rect(r)
			var pt := _grid_point(mb.position, r, inter)
			if not pt["valid"]:
				return
			map_right_clicked.emit(pt["gx"], pt["gy"],
				_hit_test_settlement(mb.position, inter, r), mb.position)
			accept_event()
			return
		if mb.button_index != MOUSE_BUTTON_LEFT:
			return
		var rect := _displayed_rect()
		if rect.size.x <= 0.0:
			return
		var interior := _interior_rect(rect)
		if mb.pressed:
			var hit := _hit_test_settlement(mb.position, interior, rect)
			var p := _grid_point(mb.position, rect, interior)
			## Touch: hold the press back until the gesture identifies itself.
			## See the `_TOUCH_HOLD_MS` block above for the four outcomes.
			if mb.device < 0 or OS.has_feature("mobile"):
				_touch_armed = true
				_touch_swallow_up = false
				_touch_ms = Time.get_ticks_msec()
				_touch_pos = mb.position
				_touch_press = {"hit": hit, "point": p, "pos": mb.position}
				set_process(true)
				return
			settlement_selected.emit(_settlements[hit] if hit != -1 else null, hit)
			if p["valid"]:
				map_clicked.emit(p["gx"], p["gy"])
		else:
			if _touch_swallow_up:
				## The hold already answered this gesture; the lift is its end,
				## not a drag's. Emitting `map_released` here would hand a
				## latched tool (Region select's marquee origin) a commit it
				## never got an origin for.
				_touch_swallow_up = false
				_touch_armed = false
				set_process(false)
				return
			if _touch_armed:
				## A tap: short, and it never travelled. The press it withheld
				## is due now, immediately before the release that ends it.
				_release_touch_press()
			## §4.5.1's Region select needs the release, not the press --
			## `map_dragged` already reported every point along the way, but
			## nothing marked the gesture's *end*, which is where a marquee
			## commits. `p["valid"]` is intentionally not required here: a
			## drag that ends off-plate still has to end the drag, or a
			## caller's own latched origin (`GlobalTools._region_origin`)
			## would never clear.
			var p := _grid_point(mb.position, rect, interior)
			map_released.emit(p["gx"], p["gy"], p["valid"])


## Let a withheld touch press through, unchanged and in its original order --
## the selection first, then the click, exactly as the mouse branch emits them
## and from the point the finger actually went down on, not from where it is
## now. Clears the arming, so a second call is a no-op.
func _release_touch_press() -> void:
	if not _touch_armed:
		return
	_touch_armed = false
	set_process(false)
	var hit: int = int(_touch_press.get("hit", -1))
	var p: Dictionary = _touch_press.get("point", {})
	settlement_selected.emit(_settlements[hit] if hit != -1 else null, hit)
	if bool(p.get("valid", false)):
		map_clicked.emit(p["gx"], p["gy"])

## Runs only while a touch press is outstanding (`set_process` is turned on by
## the press and off by every one of the four exits), so this costs nothing on
## a desktop run and nothing between gestures on a phone.
func _process(_delta: float) -> void:
	if not _touch_armed:
		set_process(false)
		return
	if Time.get_ticks_msec() - _touch_ms < _TOUCH_HOLD_MS:
		return
	## The hold. The withheld press is **discarded**, never emitted: that is the
	## whole point -- opening the menu must not also fire the armed tool.
	_touch_armed = false
	_touch_swallow_up = true
	set_process(false)
	var p: Dictionary = _touch_press.get("point", {})
	if not bool(p.get("valid", false)):
		return
	map_right_clicked.emit(p["gx"], p["gy"], int(_touch_press.get("hit", -1)),
		_touch_press.get("pos", Vector2.ZERO))

func _notification(what: int) -> void:
	if what == NOTIFICATION_MOUSE_EXIT:
		cursor_sampled.emit(0.0, 0.0, false)
		if _hover_index != -1:
			_hover_index = -1
			queue_redraw()
			settlement_hovered.emit(null, -1)


# ── Urban layouts ────────────────────────────────────────────────────────────
#
# `civUrbanLayoutsChk` (`GUI_GAP_REGISTER.md` UM-01): the reference's own
# deep-zoom town-layout layer, `_umDrawLayout` called from `drawCivLayer`'s
# §2.5. This port draws the same layer from the same kind of data, restricted
# to what `URBAN_MORPHOLOGY_SCOPE.md` milestones 1-7 actually generate — a
# street skeleton on a real site. Blocks, buildings and the wall circuit are
# milestones 10-13 and are not drawn, not stubbed; see `urban_layout_draw.gd`.
#
# **The reveal gate is deliberately NOT the reference's `_umLayoutAlpha`.**
# That function crossfades pins into layouts across a 24 km → 10 km viewport
# span, which works there because its LOD region window lets the camera reach
# a few-km span. This port's camera clamps at `ViewportHost.ZOOM_MAX` (8.0),
# so on a default 800 km world the closest reachable span is ~100 km and a
# ported 24 km threshold would never once fire — a toggle that silently draws
# nothing on the default world is worse than a different, stated rule. The
# gate here is the thing that actually matters for whether a town is worth
# drawing at all: how many screen pixels its 1.7 km site box covers. A town
# under `URBAN_MIN_BOX_PX` is a smudge and is skipped, and no layout is even
# requested for it.
## `preload`, not the `UrbanLayoutDraw` global class name -- see
## `city_viewer_window.gd`'s own `DRAW` const for why.
const URBAN_DRAW := preload("res://shell/urban_layout_draw.gd")
const URBAN_MIN_BOX_PX := 16.0
## The site box, in km — `UME.SITE_WM`/`1000` (`urban_adapter::SITE_WM`).
const URBAN_SITE_BOX_KM := 1.7
## Requested per frame, and the number is the reference's own
## `_UM_MODEL_CACHE_MAX` (line 22684): as many towns as it was willing to hold
## generated at once. Each is a few milliseconds of real generation on the
## main thread, so the cap is what keeps a pan at deep zoom from stalling.
const URBAN_BATCH_MAX := 24

## Emitted when the layer is on, the zoom is deep enough, and these settlement
## indices have no layout yet. `ViewportHost` answers with
## `set_urban_layouts()`. Deliberately a signal rather than a direct bridge
## call: this control holds no `EngineBridge` and every other value it draws
## is pushed into it, not pulled.
signal urban_layouts_needed(indices: PackedInt32Array)

var _show_urban_layouts := false
## settlement index -> layout Dictionary, or `null` for an index the engine
## refused (a settlement in open water — `_umModelFor`'s own refusal). Both
## are "answered", so neither is requested twice.
var _urban_layouts: Dictionary = {}
var _urban_pending := false
var _map_width_km := 0.0
## `_umRevealedSet` (reference line 22753): the settlement indices whose layout
## was actually drawn this frame, so the pin loop can stand down for them.
## Rebuilt every `_draw()`, never persisted -- the reference rebuilds its own
## per frame for the same reason (a place mid-generation must not lose its pin
## on the strength of the toggle alone).
var _urban_revealed: Dictionary = {}


func set_show_urban_layouts(shown: bool) -> void:
	_show_urban_layouts = shown
	queue_redraw()


## The real map width, needed to size a town against the grid. Pushed from
## `ViewportHost.refresh()` alongside the civ data.
func set_map_width_km(km: float) -> void:
	if is_equal_approx(_map_width_km, km):
		return
	_map_width_km = km
	_urban_layouts.clear()
	queue_redraw()


## `requested` is the batch that was asked for; `layouts` is what came back,
## which is shorter whenever the engine refused one. Recording the whole
## requested set is what stops a refused settlement being re-requested every
## frame forever.
func set_urban_layouts(requested: PackedInt32Array, layouts: Array) -> void:
	for i in requested:
		_urban_layouts[i] = null
	for l: Dictionary in layouts:
		_urban_layouts[int(l["index"])] = l
	_urban_pending = false
	queue_redraw()


## Dropped wholesale when the world changes — `ViewportHost.refresh()` calls
## `set_civ_data`, which calls this.
func clear_urban_layouts() -> void:
	_urban_layouts.clear()
	_urban_pending = false


## Screen pixels per model metre, at the current fit and camera zoom. Widths
## drawn through this scale with the camera exactly as positions do, which is
## right for a town's streets (a real world-space width) and is the opposite
## of what `_settlement_pin_radius` wants for a pin (not a world-space
## quantity at all).
func _urban_m_scale(rect: Rect2) -> float:
	if _map_width_km <= 0.0 or rect.size.x <= 0.0:
		return 0.0
	return rect.size.x / (_map_width_km * 1000.0)


func _draw_urban_layouts(rect: Rect2, interior: Rect2) -> void:
	var m_scale := _urban_m_scale(rect)
	if m_scale <= 0.0:
		return
	## The camera scales this whole control, so a box that measures
	## `box_px` here lands `box_px * _camera_zoom` wide on screen.
	var box_px := URBAN_SITE_BOX_KM * 1000.0 * m_scale * _camera_zoom
	if box_px < URBAN_MIN_BOX_PX:
		return

	## The camera scales and offsets this whole control, so `interior` alone is
	## "on the plate", not "on screen" -- at deep zoom that is nearly the
	## entire settlement roster, and generating a town for each is real work.
	## The viewport rect pulled back through this control's own global
	## transform is the actual visible area in the space `rect` is measured in.
	var visible_area := get_global_transform().affine_inverse() * get_viewport_rect()
	visible_area = visible_area.intersection(interior)
	if visible_area.size.x <= 0.0 or visible_area.size.y <= 0.0:
		return
	## Half a box of slack, so a town whose centre is just off-screen still
	## draws the half of itself that is on-screen.
	visible_area = visible_area.grow(box_px * 0.5 / maxf(0.001, _camera_zoom))

	var need := PackedInt32Array()
	for i in _settlements.size():
		var s: Dictionary = _settlements[i]
		if _hidden_settlement_kinds.has(s["kind"]):
			continue
		var pos := _cell_to_screen(Vector2(s["x"], s["y"]), rect)
		if not visible_area.has_point(pos):
			continue
		if not _urban_layouts.has(i):
			if need.size() < URBAN_BATCH_MAX:
				need.append(i)
			continue
		var layout = _urban_layouts[i]
		if layout == null:
			continue
		## `_umDrawLayout`'s own transform: local model metres, measured from
		## the market anchor, rotated by the layout's terrain orientation, then
		## scaled into grid units and projected like any other map point — so
		## the market lands exactly on the settlement's real position and an
		## injected real road overlays the map road it came from.
		var anchor: Vector2 = layout.get("market", Vector2.ZERO)
		var rot: float = float(layout.get("orient", 0.0))
		var cth := cos(rot)
		var sth := sin(rot)
		var grid_per_meter := float(_gw) / (_map_width_km * 1000.0)
		var to_screen := func(mp: Vector2) -> Vector2:
			var l := mp - anchor
			return _point_to_screen(Vector2(
				float(s["x"]) + 0.5 + (l.x * cth - l.y * sth) * grid_per_meter,
				float(s["y"]) + 0.5 + (l.x * sth + l.y * cth) * grid_per_meter), rect)
		## `px_floor` = one screen pixel in this control's own space: the camera
		## scales this whole control by `_camera_zoom`, so a stroke floored at
		## a literal 1.0 here would land `_camera_zoom` px thick on screen.
		URBAN_DRAW.draw_layout(self, layout, to_screen, m_scale,
			1.0 / maxf(0.001, _camera_zoom), 1.0, false)
		_urban_revealed[i] = true

	if need.size() > 0 and not _urban_pending:
		_urban_pending = true
		urban_layouts_needed.emit.call_deferred(need)

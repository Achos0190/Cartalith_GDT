extends Control
class_name ViewportHost

## The map surface (`DCC_SHELL_SPEC.md` §9).
##
## §9 lets the viewport carry exactly five things beyond the map itself: the
## brush cursor, the layers button, the scale bar, the projection/zoom readout
## and the cursor coordinates. Anything else that could live in a dock does.
## This node builds those and nothing more.
##
## The three raster layers stack in draw order -- terrain, territory fill,
## province boundaries -- and `map_overlay.gd` draws the vector layers
## (settlements, roads, sea routes) on top from world data, re-fitting itself
## every frame so a window resize needs no notification.

signal settlement_selected(data: Variant, index: int)
signal settlement_hovered(data: Variant, index: int)
signal cursor_sampled(gx: float, gy: float, valid: bool)
signal layers_button_pressed()
signal map_clicked(gx: float, gy: float)   ## §4.5 tool click-placement primitive.
signal map_dragged(gx: float, gy: float)   ## §4.5 tool drag-paint primitive.
signal map_released(gx: float, gy: float, valid: bool)   ## §4.5 tool drag-end primitive.
## `_civCtxShow`'s right-click (`map_overlay.gd`'s own signal, re-emitted).
signal map_right_clicked(gx: float, gy: float, hit: int, screen_pos: Vector2)

const OVERLAY_SCRIPT := preload("res://map_overlay.gd")
## Deep-zoom tile compositing -- see `_build_lod_tile()` and the shader's own
## header. A tile texture is a relief-detail shade ratio, not a picture; this
## is what turns it back into map pixels using `map_view`'s own colours.
const LOD_TILE_SHADER := preload("res://shell/lod_tile.gdshader")

var map_view: TextureRect
var territory_view: TextureRect
var province_view: TextureRect
var overlay: Control
var tool_overlay: ToolOverlay
var _preview_layer: TextureRect   ## Sculpt/Paint's live draft raster. See `set_preview_texture()`.
var _debug_layer: TextureRect     ## The Layers popover's field raster. See `set_debug_layer()`.
var _debug_view := "off"          ## Which view `_debug_layer` currently holds.

var _scale_label: Label
var _readout_label: Label
## `render_workspace.gd`'s active Map style preset name (or "Custom" once a
## manual edit or a loaded named look diverges from all five), pushed in via
## `set_style_readout()` -- see that method's own doc comment for why this
## file does not read it itself. "Default" until the workspace's first build,
## matching `_build_map_style()`'s own initial chip selection.
var _style_readout := "Default"
var _coords_label: Label
var _layers_btn: Button
## The touch navpad (`GUI_GAP_REGISTER.md` SH-14) and its one stateful member.
## `null` on desktop -- see `_build_navpad()` for the reachability call.
var _navpad: VBoxContainer
var _pan_btn: Button
var _bridge: EngineBridge
var _width_km := 0.0

## Default matches the original hardcoded 10 px inset every corner label and
## the layers button used before phone chrome existed. `DccShell` overrides
## this (`set_safe_insets()`) once it knows its own chrome's edges -- on
## phone the map (and this node with it) is edge-to-edge behind that chrome,
## so a flat 10 px would land this node's own chrome under the app bar/rail/
## tool sheet instead of in the visible gap between them.
var _safe_insets := {"left": 10.0, "top": 10.0, "right": 10.0, "bottom": 10.0}
## The last scale `_apply_touch_scale()` applied, so a re-inset on rotation --
## which does not change it -- does no work. See that function for HD-03.
var _touch_scale := 1.0
## The navpad glyph's name, kept for the same reason `DccWidgets
## .TOOL_GLYPH_META` keeps a tool's: an `ImageTexture` in hand cannot be grown
## without resampling it.
const NAVPAD_GLYPH_META := "dcc_navpad_glyph"

## §13's 44 px floor applies here too -- this button is real, on-screen, and
## tappable on phone, not workspace/dock content this file is exempt from
## touching. Tablet gets the same floor (its own target range is "44-52 px");
## only pointer-first Windows keeps the original compact 26 px hit box.
##
## `--force-touch` is `DccShell._ready()`'s own testing-only override, adopted
## here verbatim for the same reason it exists there: no dev/CI box has touch
## hardware, so without it `_build_navpad()` below is unreachable outside a
## real device and could only ever be verified on one.
var _touch := (DisplayServer.is_touchscreen_available() and OS.has_feature("mobile")) \
	or "--force-touch" in OS.get_cmdline_user_args()

# -- Camera (§4.5.1 Pan / zoom) ------------------------------------------------
#
# `LOD_TILING_BASE_SCOPE.md` (2026-08-17) deliberately kept `cartalith-spatial`'s
## tiling standalone -- "no camera... revisit when a concrete need appears
## rather than building it speculatively." That need was named directly
## (owner, 2026-08-19: "don't forget to wire zoom/pan"), and §4.5.1 has always
## specified it as a global tool, present in every domain. This is that wiring
## -- a camera transform over the single existing raster, nothing about tiled
## LOD, which is its own, separately scoped decision
## (`LOD_TILING_INTEGRATION_SCOPE.md`).
var _camera: Control    ## Hosts map_view/territory_view/province_view/overlay;
	## `scale`/`position` on THIS node is the whole transform. Chrome
	## (`_scale_label` etc.) stays a direct child of `self`, outside it, so it
	## is never itself panned or zoomed.
const ZOOM_MIN := 0.4
## `lodMaxZoom()` (reference 10672) -- **not** a constant, and the fix for the
## owner's 2026-08-24 "LOD zooming doesn't seem to go that deep either".
##
## This file used to cap zoom at a flat `8.0`, which is the reference's cap on
## `viewT.scale` (13381) and nothing else. But the reference *hands off* at
## 2.2x: `enterLodFromView` (13953) pins `viewT.scale` back to 1 and gives the
## camera to the tiled-LOD viewer, whose own zoom `_lodZoom` runs to
## `lodMaxZoom()` -- and that function exists because of an owner report with
## the same shape as this one. Its comment (v0.88) says so outright: *"highest
## zoom stops at 20km, I'd like to drop down to 5km ... Scale the cap so a
## real-world span of <=5km is always reachable, never less generous than the
## old x64 for small/default maps."*
##
## So the reachable depth is a property of the **map's real width**, not a
## screen-space constant. Measured before changing anything, on a default 800 km
## world: this port stopped at `z8` = a **100 km** visible span, where the
## reference reaches `z160` = **5 km**. Twenty times short, and visibly so --
## the deepest view this port could reach was a smooth blur.
##
## `_zoom` here means exactly what `_lodZoom` means there (the map's full width
## divided by the visible span), so the number ports directly.
const ZOOM_MAX_FLOOR := 64.0    ## `Math.max(64, ...)` -- never less generous
	## than the pre-v0.88 hardcoded x64 for small or default maps.
const ZOOM_TARGET_SPAN_KM := 5.0   ## `(state.mapWidthKm||800)/5` -- the span
	## the owner asked to be able to reach on any map size.
var _zoom_max := ZOOM_MAX_FLOOR   ## `lodMaxZoom()` for the live world;
	## recomputed in `refresh()`, floored until one exists.
const ZOOM_WHEEL_STEP := 1.15   ## Multiplicative per wheel notch.
var _zoom := 1.0
var _panning := false
var _pan_last_screen := Vector2.ZERO
## The navpad's ✋ (`GUI_GAP_REGISTER.md` SH-14) -- the reference's `panMode`
## (9582), which despite the ✋ glyph is a **latching toggle**, not a
## press-and-hold: `panBtn`'s whole handler is `panMode=!panMode` (13963).
## Checked before assuming, because "hold to pan" is what the button looks
## like it means. While it is on, the reference gives a plain button-0
## pointerdown to the pan drag (9623) and suppresses the armed tool
## (13924) -- both of which fall out of `_panning` below for free.
var _pan_mode := false

# -- Deep-zoom tile compositing (`LOD_TILING_INTEGRATION_SCOPE.md` milestone M1) ---
#
## The gap M1 exists to close: `_raster()` below sets `TEXTURE_FILTER_NEAREST`
## on `map_view`, so the moment a cell occupies more than one screen pixel the
## base raster shows visibly blocky single-cell squares -- the reference's own
## owner complaint the scope document quotes verbatim (`docs/HANDOFF.md`,
## "There is still a certain pixilated quality to the map when we zoom").
## `_update_lod()` below is what switches the affected screen region over to
## `amplify_region`/`refine_tile`-synthesized tiles (already ported and
## golden-tested, `cartalith-terrain::amplify`/`tile_render` -- this file adds
## no new numerical logic of its own, only screen<->grid geometry, the same
## kind `_zoom_at` below already does) once that threshold is actually
## crossed. Below the threshold this is a complete no-op and Z1 (the
## coordinating session's zoom/pan) is entirely unaffected -- `_lod_active`
## starts `false` and every early-return path above leaves it that way.
##
## Threshold: "more than roughly one screen pixel per grid cell" (the
## milestone's own wording) is exactly `native_px_per_cell * _zoom > 1.0`,
## where `native_px_per_cell` is the *unzoomed* fit scale `_update_lod()`
## computes the same way `map_overlay.gd`'s own `_displayed_rect()` does
## (`min(control_px / grid_cells)` per axis) -- `_zoom` alone is not
## sufficient, since a small preset (512) already exceeds one px/cell at
## `_zoom == 1.0` in most windows, while a large one (8192) needs real zoom
## to reach it. `LOD_PX_PER_CELL_THRESHOLD` names that "roughly one" plainly
## rather than burying `1.0` in the formula below.
const LOD_PX_PER_CELL_THRESHOLD := 1.0
## ...and a second gate the reference has and this file was missing: the
## camera must actually be zoomed IN. `LOD_AUTO_SCALE = 2.2` (reference
## 13952), tested against `viewT.scale` in the wheel handler (reference
## 13986), is a pure camera-zoom threshold, independent of grid resolution.
## Without it the px-per-cell test alone turns deep zoom on at the *fit* view
## for any grid narrower than the viewport -- a 512-cell world in a 900 px map
## rect is already at 1.7 px/cell before the user touches the wheel -- so the
## LOD layer was live on every freshly generated world, which is half of why
## the owner's "a zoom action exposes the underlying heightmap" was visible
## with no zoom action at all. Both conditions must hold, as in the reference:
## px-per-cell says the detail is *resolvable*, this says it was *asked for*.
const LOD_AUTO_ZOOM := 2.2
## A real bound, not a cache: right at the threshold crossing is where the
## *most* tiles are visible at once (zooming in further shows fewer, larger
## ones), and synthesizing an unbounded burst in one call the instant a fast
## wheel-scroll crosses the line would stall a frame for no good reason.
## This bounds only the *synthesis* calls a single `_update_lod()` invocation
## may issue -- an already-built tile that is still wanted is never freed or
## re-synthesized just because it falls outside the closest-`N`-to-centre
## set, and whatever misses this call's budget is queued in `_lod_backlog`
## rather than dropped: `_process()` below drains a few more of it every
## frame until either the backlog empties or the next `_update_lod()` call
## replaces it with a freshly-computed one.
##
## Before `_lod_backlog` existed, a viewport wanting more than this many
## *new* tiles at once (a large window, or a fast zoom crossing the
## threshold) left the excess permanently missing: `_apply_lod_tiles`'s
## reconciliation only ever acted on the current `wanted` vs. `_lod_tiles`,
## and a camera that stops moving never calls `_update_lod()` again to give
## the dropped tiles a second chance -- confirmed with a real headless
## repro (grid 512, a 1920x1080 viewport, 64 tiles wanted at the default
## zoom) before this comment was updated: `built` stuck at exactly 48 across
## five redundant `_update_lod()` calls with nothing about the camera
## changing between them. Background synthesis is exactly what
## `_lod_backlog`/`_process()` now are; the one piece still out of this
## milestone's scope is a persistent bake across sessions (the Z5 atlas
## cache, `LOD_TILING_INTEGRATION_SCOPE.md` M3).
const MAX_LOD_TILES_PER_UPDATE := 48
## How many backlog tiles `_process()` synthesizes per frame once
## `_update_lod()` couldn't fit everything into one call's
## `MAX_LOD_TILES_PER_UPDATE` budget. Small on purpose -- this runs every
## frame while a backlog exists (not once per input event like the cap
## above), so it only needs to be large enough to drain a few hundred queued
## tiles over a couple of seconds without itself becoming a per-frame stall.
const MAX_LOD_TILES_PER_CATCHUP := 6

var _lod_layer: Control   ## Child of `_camera`, drawn directly above
	## `map_view` and below every overlay (territory, provinces, the Layers
	## debug raster, previews, vectors) -- it replaces the base raster at deep
	## zoom rather than sitting on top of the stack. See its `_ready()`
	## construction site for why that distinction is load-bearing.
var _lod_debug_layer: Control   ## Child of `_lod_layer`, above its tiles --
	## the chunk-debug overlay. See its construction site in `_ready()`.
var _lod_dbg_grid := false      ## The reference's `_lodGrid`,
var _lod_dbg_colors := false    ## `_lodChunkCol` and
var _lod_dbg_labels := false    ## `_lodLabels` (reference line 10933).
var _lod_tiles: Dictionary = {}   ## `"%d,%d,%d" % [z, col, row]` -> the live
	## `Sprite2D` showing that pyramid chunk, so a pan/zoom that doesn't
	## touch a given chunk leaves its node (and the Rust call that built it)
	## alone -- see `_update_lod()`'s own doc comment on why this is what
	## keeps calling it once per mouse-motion sample affordable.
	##
	## The level is **in the key** since 2026-08-24, which retired a whole
	## parallel `_lod_tile_detail` dictionary: at the old fixed 64-cell tile
	## grid an index meant the same ground at every detail tier, so the tier
	## had to be tracked beside it and compared; under the pyramid a chunk
	## index only means anything *with* its level, and a level change makes
	## every old key simply absent from `wanted`, which the reconciliation in
	## `_apply_lod_tiles` already frees.
var _lod_backlog: Dictionary = {}   ## `"%d,%d,%d"` -> `Vector3i(z, col, row)`
	## -- chunks the most recent `_update_lod()` wanted but couldn't fit inside
	## `MAX_LOD_TILES_PER_UPDATE`'s per-call synthesis budget. Replaced
	## wholesale on every `_update_lod()` call (never merged), so a camera
	## move that stops wanting a backlogged tile drops it for free the next
	## time `_update_lod()` runs, rather than `_process()` wastefully
	## building it anyway. See `MAX_LOD_TILES_PER_UPDATE`'s own doc comment
	## for why this queue exists at all.
var _lod_backlog_n := 0   ## Tiles per axis at the level `_lod_backlog`'s
var _lod_backlog_grid := Vector2i.ZERO   ## entries belong to, and the geometry
var _lod_backlog_origin := Vector2.ZERO   ## they were computed against --
var _lod_backlog_size := Vector2.ZERO   ## `_process()` reuses it rather than
	## re-deriving it from the camera, since it is exactly what the
	## `_update_lod()` call that filled the backlog already had on hand, and is
	## guaranteed current: any camera motion that would invalidate it also
	## calls `_update_lod()`, which replaces the whole backlog (and this
	## geometry) before `_process()` next runs.
var _lod_active := false

## **§2.5's "Tiled LOD — `auto on zoom` (default) · `manual`", the reference's
## `state.lodAuto` (line 13479).**
##
## The reference gates exactly one thing on it: the wheel handler's
## `if(state.lodAuto && !_lodOn && e.deltaY<0 && ...) enterLodFromView(...)`
## (line 13986). It never prevents the LOD view from being entered, only from
## being entered *by zooming*. That distinction is why `manual` is a usable
## mode there and would be a trap here without the request below -- a
## suppressor with no way in makes deep detail unreachable, which is worse
## than not offering the choice.
##
## So: `_lod_auto` false suppresses automatic entry, and
## `request_lod_entry()` is the way in. Once entered, manual mode keeps the
## pyramid up while the camera stays past the threshold and drops it on the
## way out, exactly as auto does -- the mode is about *entering*, not about
## staying.
var _lod_auto := true
var _lod_manual_request := false

## The export tile-border preview (`#lodShowGrid` / `drawExportTileGrid`).
## `_export_grid_cols`/`_rows` mirror `data_manager_window.gd`'s `_tx_cols`/
## `_tx_rows`; that window pushes them here whenever they change, so this is a
## cache of one authority rather than a second one.
var _export_grid_layer: Control
var _export_grid_on := false
var _export_grid_cols := 4
var _export_grid_rows := 4

func setup(bridge: EngineBridge) -> void:
	_bridge = bridge
	bridge.generation_finished.connect(func(ok: bool): if ok: refresh())
	bridge.world_loaded.connect(refresh)
	## The landmark pass's placements reach the map through here and nowhere
	## else. Connected in the ONE place that owns `overlay` rather than left to
	## whoever presses the button: `CivilizationWorkspace._lm_run()` was the
	## only caller of `landmark_run()` and it did not push the result at the
	## map, which is the second half of the owner's 2026-09-01 report -- the
	## pass placed 239 landmarks and `MapOverlay._landmarks` stayed `[]`
	## forever, so the rings only ever appeared if the user happened to go to
	## Cartography afterwards and drag an icon. A caller that forgets is the
	## bug; a caller that cannot forget is the fix.
	bridge.landmark_finished.connect(func(_r: Dictionary): refresh_annotations())

func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_PASS

	## `_camera`'s own logical `size` always equals the viewport's real size
	## (full-rect, same as before) -- `scale`/`position` handle zoom/pan on top
	## of that, which is why `overlay`'s existing `_displayed_rect()` fit-scale
	## math (written entirely in its own local space) needed no changes at all
	## to keep working under a scaled parent.
	_camera = Control.new()
	_camera.set_anchors_preset(Control.PRESET_FULL_RECT)
	_camera.mouse_filter = Control.MOUSE_FILTER_PASS
	add_child(_camera)

	map_view = _raster()
	_camera.add_child(map_view)

	## Deep-zoom tile overlay (`LOD_TILING_INTEGRATION_SCOPE.md` milestone
	## M1): directly over `map_view` and under *every* overlay, because a
	## refined tile stands in for the blocky base pixels -- it is the base
	## map at deep zoom, not something drawn on top of it. Starts fully
	## transparent and empty; `_update_lod()` is the only thing that ever
	## adds children to it.
	##
	## Since 2026-08-23 a tile *literally* is the base map: its shader
	## samples `map_view`'s own texture for colour and uses the tile texture
	## only as a relief-detail shade ratio (`_build_lod_tile()`,
	## `lod_bridge.rs`). Before that it carried
	## `render_height_tile_rgba`'s hypsometric ramp, which is the reference's
	## *Relief* view mode, and covering the biome-coloured plate with it is
	## exactly the owner's "a zoom action exposes the underlying heightmap".
	##
	## **This node's position in the stack is load-bearing, and once got it
	## wrong.** It used to be added after `territory_view`, `province_view`
	## and `_debug_layer`, on the reading that "above the base raster" meant
	## "above everything raster." It does not: those three are overlays that
	## happen to be rasters. `_update_lod()` turns this layer on whenever the
	## fit scale exceeds one screen pixel per cell -- which a 384x256 grid in
	## a 900px-wide viewport already does at `_zoom == 1.0`, no zooming
	## needed -- and at `modulate.a == 1.0` its opaque tiles then covered the
	## faction fill, the province boundaries and every one of the Layers
	## popover's field views completely. The popover still highlighted the
	## picked row, `debug_view()` still echoed it back, and the map did not
	## change: the owner's "a host of options such as layers dont work"
	## (2026-08-20), reproduced live and fixed by these four lines moving up.
	_lod_layer = Control.new()
	_lod_layer.set_anchors_preset(Control.PRESET_FULL_RECT)
	_lod_layer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_lod_layer.modulate.a = 0.0
	_camera.add_child(_lod_layer)

	## Chunk-debug overlay -- the reference's `drawLODChunkDebug` (line 10946),
	## reached there from `drawLODView`'s tail and gated on the same three
	## toggles. A CHILD of `_lod_layer` rather than a sibling, deliberately:
	## it inherits the `modulate:a` fade `_set_lod_active()` tweens, so the
	## overlay appears and leaves exactly with the tiles it annotates instead
	## of popping a frame early. `z_index` puts it above the `Sprite2D` tiles
	## added to the same parent later; `_clear_lod_tiles()` only frees nodes
	## it tracks in `_lod_tiles`, so this one survives a level change.
	_lod_debug_layer = Control.new()
	_lod_debug_layer.set_anchors_preset(Control.PRESET_FULL_RECT)
	_lod_debug_layer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_lod_debug_layer.z_index = 1
	_lod_debug_layer.visible = false
	_lod_debug_layer.draw.connect(_draw_lod_debug)
	_lod_layer.add_child(_lod_debug_layer)

	territory_view = _raster()
	province_view = _raster()
	territory_view.visible = false
	province_view.visible = false
	_camera.add_child(territory_view)
	_camera.add_child(province_view)

	## The Layers popover's field raster (`sample_bridge.rs`'s debug views),
	## directly over the three base rasters and under everything vector --
	## the reference's own debug overlay sits in exactly that slot, blended
	## over the base map by its `dbgOpacity` slider rather than replacing it.
	## Empty and fully opaque by default; nothing shows until a view is
	## picked, and `modulate.a` is what the popover's opacity slider drives.
	_debug_layer = _raster()
	_camera.add_child(_debug_layer)

	## One shared draft-preview layer for every tool whose result is a full
	## raster (`build_sculpt_preview_texture`, `build_paint_preview_texture`)
	## rather than vector geometry `tool_overlay` already covers -- so Sculpt/
	## Paint (and anything landing later) never need their own `TextureRect`
	## wired into this file. Above `_lod_layer` and below `overlay`, matching
	## the same "raster first, vectors on top" order the base layers already
	## use. Empty texture = invisible; nothing shows until a caller sets one.
	_preview_layer = _raster()
	_camera.add_child(_preview_layer)

	overlay = Control.new()
	overlay.set_script(OVERLAY_SCRIPT)
	overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	_camera.add_child(overlay)
	overlay.settlement_selected.connect(func(d, i): settlement_selected.emit(d, i))
	overlay.settlement_hovered.connect(_on_hovered)
	overlay.cursor_sampled.connect(_on_sampled)
	overlay.map_clicked.connect(func(gx, gy): map_clicked.emit(gx, gy))
	overlay.map_dragged.connect(func(gx, gy): map_dragged.emit(gx, gy))
	overlay.map_released.connect(func(gx, gy, valid): map_released.emit(gx, gy, valid))
	overlay.map_right_clicked.connect(func(gx, gy, hit, pos): map_right_clicked.emit(gx, gy, hit, pos))
	## The town-layout layer pulls its own data, one deferred batch at a time,
	## because generating a town is real engine work and only the overlay knows
	## which towns are on screen and large enough to be worth drawing. This is
	## the whole of the reference's `_umModelFor` scheduling queue that this
	## port keeps -- `URBAN_MORPHOLOGY_SCOPE.md` puts its LRU and its
	## `setTimeout(...,0)` pump explicitly out of scope (a workaround for the
	## browser's single thread), and the overlay's own index-keyed dictionary,
	## dropped whole on every `set_civ_data`, is the only invalidation this
	## shell needs.
	overlay.urban_layouts_needed.connect(_on_urban_layouts_needed)

	## §4.5.1's tool feedback (Region marquee, Measure ruler) -- above
	## `overlay` in draw order so it's never hidden behind terrain/vector
	## data, `MOUSE_FILTER_IGNORE` always so it never competes with
	## `overlay`'s own hit-testing for the click that feeds it.
	tool_overlay = ToolOverlay.new()
	tool_overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	tool_overlay.overlay = overlay
	_camera.add_child(tool_overlay)

	## **Export tile borders** -- the reference's `#lodShowGrid`, "Show tile
	## borders on the map" (reference line 1281), whose handler sets
	## `_showExportGrid` and whose draw is `drawExportTileGrid()` (line 9602).
	##
	## `DCC_SHELL_SPEC.md` §2.5 lists it beside the off/grid/colours segment,
	## under "Chunk debug overlay ... + tile borders", and reading the spec
	## alone makes it look like a fourth chunk-debug toggle. **It is not**, and
	## `UNWIRED_FUNCTIONS.md` said so wrongly before the reference was read:
	## the spec groups the two because the reference PANEL puts the checkbox
	## under the same accordion, but the two are different features over
	## different data.
	##
	##   - The chunk-debug segment annotates live pyramid tiles, and its draw
	##     runs inside `drawLODView`'s tail -- LOD ON.
	##   - This draws the **export** split: `refCols` x `refRows` (reference
	##     lines 1276-1277, the Cols/Rows number fields of the tile-export
	##     block), dashed, over the whole map, and its call site is guarded
	##     `if(_showExportGrid && !_lodOn)` at line 8658 -- LOD **OFF**.
	##
	## So it belongs here, a sibling of the overlays and NOT a child of
	## `_lod_layer`, and its cols/rows come from `DataManagerWindow`'s own
	## `_tx_cols`/`_tx_rows` (the same two numbers the export writes, defaulting
	## 4x4 there against the reference's 2x2) rather than from a second store
	## free to disagree with what Export actually emits.
	_export_grid_layer = Control.new()
	_export_grid_layer.set_anchors_preset(Control.PRESET_FULL_RECT)
	_export_grid_layer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_export_grid_layer.z_index = 2
	_export_grid_layer.visible = false
	_export_grid_layer.draw.connect(_draw_export_tile_grid)
	_camera.add_child(_export_grid_layer)

	## §9's chrome, all corner-anchored so it survives any dock width.
	_scale_label = _chrome(Control.PRESET_BOTTOM_LEFT, HORIZONTAL_ALIGNMENT_LEFT)
	_readout_label = _chrome(Control.PRESET_TOP_RIGHT, HORIZONTAL_ALIGNMENT_RIGHT)
	_coords_label = _chrome(Control.PRESET_BOTTOM_RIGHT, HORIZONTAL_ALIGNMENT_RIGHT)

	_layers_btn = Button.new()
	_layers_btn.flat = true
	_layers_btn.focus_mode = Control.FOCUS_NONE
	_layers_btn.icon = DccIcons.get_icon("layers", 15)
	## Icon-only, so `icon_alignment`'s `LEFT` default would hang the glyph off
	## the left edge of its own hit box. `_apply_touch_scale()` below already
	## sets this -- but only when the scale actually *changes*, and it opens
	## `if is_equal_approx(scale, _touch_scale)` against a `_touch_scale` that
	## initialises to `1.0`. So a touch device running at scale 1.0 (a tablet)
	## never reached that line and kept the misaligned default. Set at
	## construction, where it is true of every device; the copy in the scale
	## pass is left alone because it is idempotent and its comment carries the
	## OnePlus 6T history that found this class of bug.
	_layers_btn.icon_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_layers_btn.tooltip_text = "Layers"
	_layers_btn.modulate = DccTheme.c("text_dim")
	_layers_btn.set_anchors_and_offsets_preset(Control.PRESET_TOP_LEFT)
	var hit := 44 if _touch else 26
	_layers_btn.custom_minimum_size = Vector2(hit, hit)
	_layers_btn.add_theme_stylebox_override("normal",
		DccTheme.flat(DccTheme.c("panel"), 3))
	_layers_btn.pressed.connect(func(): layers_button_pressed.emit())
	add_child(_layers_btn)

	_build_navpad()

	## A resize changes the native fit rect `_update_lod()` positions every
	## tile against (`displayed_origin`/`displayed_size`, computed from
	## `size`) -- without this, resizing the window while zoomed in past the
	## threshold would leave existing tiles positioned against the old
	## window size until the next zoom/pan event.
	resized.connect(_update_lod)
	## ...and the navpad is the one piece of chrome positioned absolutely
	## against the *right/bottom* edge rather than anchored there, so unlike
	## the three labels it does not track a resize on its own.
	resized.connect(_apply_safe_insets)

	## `_process()` (deep-zoom backlog catch-up, see `_lod_backlog`'s own doc
	## comment) has nothing to do until `_update_lod()` first populates a
	## backlog -- disabled here rather than paying a per-frame call for a
	## dictionary that starts, and usually stays, empty.
	set_process(false)

	_apply_safe_insets()

## `_input`, not `_gui_input` or `_unhandled_input` -- verified empirically,
## not assumed: `overlay` sits in front with `MOUSE_FILTER_STOP` for its own
## hover/click handling, and Godot's real dispatch order is `_input` (every
## node, tree order) -> GUI dispatch (`_gui_input`, front-to-back, stops at
## the first `MOUSE_FILTER_STOP` control) -> `_unhandled_input`. A first
## attempt at `_unhandled_input` logged the wheel event but never the MMB
## press or its motion -- GUI dispatch had already consumed them, since
## `MOUSE_FILTER_STOP` swallows every mouse event over a control's rect for
## dispatch purposes, not only the ones its own script chooses to act on.
## `_input` runs before that dispatch, so it sees everything unconditionally;
## `set_input_as_handled()` is then called only for the specific events this
## node actually consumes (wheel, MMB, magnify, and a motion event only while
## actively panning), so plain hover motion still reaches `overlay`'s own
## `_gui_input` exactly as before -- no change needed to `map_overlay.gd`.
##
## Pan is a held modifier (MMB, or Space+LMB), per §4.5.1 exactly: *"Pan / zoom
## | Space (held), MMB | Hand | Always available as a modifier, even with
## another tool armed."* Deliberately not bare LMB-drag -- that would collide
## with every future tool that drags with the primary button (Sculpt, Biome
## paint, Territory), which is precisely why the spec calls it a modifier and
## not a default drag.
## Running before GUI dispatch is what makes the handler above work, and it is
## also its one hazard: `_input` fires for **every** node on **every** event,
## wherever the cursor is. Until 2026-08-24 the wheel branches below took that
## literally -- a notch anywhere in the shell zoomed the map and then called
## `set_input_as_handled()`, which cancels GUI dispatch entirely. So **no
## `ScrollContainer` in the application could be scrolled with the wheel**: the
## left dock (836 px of content in a 774 px window, measured), the right dock,
## every popover and every dialog body. Reported by the owner as "rail scrolling
## doesn't work on mouse hover" and reproduced live at three separate hover
## points, all reading `scroll_vertical == 0` after five notches.
##
## The fix is the guard the LMB branch below already carries for the navpad,
## generalised: a press only belongs to the camera when it lands on this node's
## own rect. `_input` still *sees* everything -- that is still required, for the
## reason the comment above gives -- it just stops *claiming* everything.
## Releases are deliberately exempt: a pan that began on the map and ended over
## a dock must still clear `_panning`, or the camera sticks to the cursor.
func _input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		var over_map := get_global_rect().has_point(mb.position)
		if mb.button_index == MOUSE_BUTTON_WHEEL_UP and mb.pressed and over_map:
			_zoom_at(mb.position - global_position, ZOOM_WHEEL_STEP)
			get_viewport().set_input_as_handled()
		elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN and mb.pressed and over_map:
			_zoom_at(mb.position - global_position, 1.0 / ZOOM_WHEEL_STEP)
			get_viewport().set_input_as_handled()
		elif mb.button_index == MOUSE_BUTTON_MIDDLE:
			if mb.pressed and not over_map:
				return
			_panning = mb.pressed
			_pan_last_screen = mb.position
			if mb.pressed:
				get_viewport().set_input_as_handled()
		elif mb.button_index == MOUSE_BUTTON_LEFT and _pan_mode:
			## The navpad's ✋ is on, so the primary button *is* the pan drag --
			## exactly the reference's `if(e.button===1||panMode||spaceDown)`
			## (9623). Reuses `_panning`, so the MouseMotion branch below needs
			## no change at all; and handling the press here, before GUI
			## dispatch, is what keeps the armed tool from also seeing it
			## (the reference's own `!panMode` tool guard, 13924).
			##
			## On the phone this is a single finger: Godot's
			## `emulate_mouse_from_touch` (default, and left alone by
			## `project.godot`'s pinch note) delivers a one-finger drag as
			## LEFT press + motion with the LEFT mask set. Two fingers keep
			## going to the magnify/pan gesture pair below, unaffected.
			##
			## One guard the reference does not need: its buttons live outside
			## the canvas element, while this column sits *over* the map. This
			## handler runs before GUI dispatch, so without it a tap on ✋ or ⟳
			## would start a pan drag as well as press the button.
			## `get_global_rect()` rather than `get_rect()` -- the former
			## carries every parent offset, so it is right whether or not this
			## control happens to start at the viewport origin.
			## `over_map` extends that same guard to the docks and bars, which
			## `_input` reaches just as freely as the navpad -- see this
			## function's own header.
			if mb.pressed and (not over_map or (_navpad != null \
					and _navpad.get_global_rect().has_point(mb.position))):
				return
			_panning = mb.pressed
			_pan_last_screen = mb.position
			get_viewport().set_input_as_handled()
	elif event is InputEventMouseMotion:
		var mm := event as InputEventMouseMotion
		var space_held := Input.is_key_pressed(KEY_SPACE)
		if _panning or (space_held and (mm.button_mask & MOUSE_BUTTON_MASK_LEFT) != 0):
			_camera.position += mm.position - _pan_last_screen
			_pan_last_screen = mm.position
			get_viewport().set_input_as_handled()
			## Every motion sample, not just on release: `_update_lod()`'s
			## own doc comment explains why this is cheap even at that
			## frequency (only a newly-revealed tile index triggers a real
			## Rust call) -- and updating live means a fast pan reveals new
			## deep-zoom tiles as it crosses into them, not only once the
			## drag ends.
			_update_lod()
		else:
			_pan_last_screen = mm.position
	elif event is InputEventMagnifyGesture:
		## Two-finger pinch, synthesized by the platform on touch. Single-
		## finger drag-to-pan is deliberately not wired on touch: it would
		## collide with an armed tool's own drag the same way bare LMB-drag
		## would on desktop, and the reference makes exactly the same call --
		## its own touch handler comments *"one finger keeps painting/drawing"*
		## (HTML line 13988) and gives the single finger to the tool, never to
		## the camera. Two fingers have no such conflict, so both halves of the
		## gesture pair are safe, and the reference drives both from that one
		## handler: zoom about the centroid **and** pan by the centroid's own
		## delta, in the same `touchmove` (HTML lines 14014-14015).
		var mg := event as InputEventMagnifyGesture
		_zoom_at(mg.position - global_position, mg.factor)
		get_viewport().set_input_as_handled()
	elif event is InputEventPanGesture:
		## The pan half of that pair, and until it existed a phone could not
		## move the camera *at all*: pan above is MMB or Space+LMB, and a
		## handheld has neither -- verified on the real device, where a
		## single-finger `input swipe` across the map moved only the hover
		## cursor and left every map pixel identical.
		##
		## Emitted by the same Android gesture detector `SH-10` turned on
		## (`input_devices/pointing/android/enable_pan_and_scale_gestures`) --
		## the setting name is not incidental, it gates *pan and scale*
		## together, and the shipped APK's own `GodotGestureHandler` carries
		## `handlePanEvent`/`setPanEvent` beside the `onScale` pair SH-10
		## already confirmed. So this arrives for free on a build that already
		## pinches; nothing new had to be enabled.
		##
		## Sign: Android's `GestureDetector` defines `distanceX/Y` as *old
		## focus minus new*, so the delta points opposite the fingers --
		## subtracting it makes the map travel *with* them, which is what
		## every map application does and what the reference's own
		## `viewT.panX += cx - pinch.cx` does with its own (already-inverted)
		## convention. Measured on device, not reasoned into place: the same
		## injected two-finger drag moves the map the way the fingers went.
		##
		## `_zoom` is deliberately untouched -- a pan is not a zoom, and the
		## magnify branch above already owns that half of the gesture, so a
		## real pinch (which fires both) still zooms and pans exactly once
		## each, like the reference's single `touchmove` does.
		var pg := event as InputEventPanGesture
		_camera.position -= pg.delta
		_update_lod()
		get_viewport().set_input_as_handled()

## Zooms so the world point under `screen_pt` stays under it: with `pan` the
## camera's position and `zoom` its scale, a local point maps to screen space
## as `screen = pan + local * zoom`. Solving for the new pan that keeps the
## same local point fixed under `screen_pt` after rescaling gives this directly
## -- no pivot_offset needed, which sidesteps Control's own layout-vs-transform
## interactions around pivot entirely.
##
## **`screen_pt` is THIS NODE's local space, not the viewport's**
## (`GUI_GAP_REGISTER.md` SH-11). `_camera` is a child of `ViewportHost`, so
## `_camera.position` is measured from this node's own origin; an
## `InputEvent.position` is measured from the window's. Subtracting one from
## the other put the pivot out by exactly `global_position * (1/z0 - 1/z1)` --
## **measured at 32.59 px per wheel notch** on the desktop layout
## (`global_position` (412, 70), zoom 1.6727 -> 1.9236), the same drift at
## every probe point, which is the signature of a constant offset rather than
## a pivot error. `zoom_step()` below was always correct because `size * 0.5`
## is already local, and measured 0.00 px on the same run. So the two `_input`
## call sites convert; this function's own maths never needed changing.
func _zoom_at(screen_pt: Vector2, factor: float) -> void:
	var new_zoom: float = clampf(_zoom * factor, ZOOM_MIN, _zoom_max)
	if is_equal_approx(new_zoom, _zoom):
		return
	var local_pt := (screen_pt - _camera.position) / _zoom
	_camera.position = screen_pt - local_pt * new_zoom
	_zoom = new_zoom
	_camera.scale = Vector2(_zoom, _zoom)
	_update_zoom_readout()
	_update_scale_bar()   ## `lodSpanKm()` -- see its own doc comment.
	## `overlay`'s settlement pins are re-scaled to compensate for
	## `_camera.scale` above (`map_overlay.gd`'s `_civ_zoom_k()`, see
	## `PIN_SCALE_REF_PX`'s own doc comment for why) -- that compensation is
	## baked into `_draw()`'s cached draw commands, so it needs telling every
	## time zoom actually changes, not just once.
	overlay.set_camera_zoom(_zoom)
	_update_lod()

## The navpad's + / − (`GUI_GAP_REGISTER.md` SH-14). The reference's own two
## buttons are `zoomAt(viewCenter(), 1.35)` and its inverse (13464-13465) --
## the *centre* of the viewport, not the last-touched point, because a button
## press carries no map position of its own. That is `_zoom_at` above with one
## argument filled in; no second zoom path, and every consequence `_zoom_at`
## already handles (the readout, `overlay`'s pin compensation, the deep-zoom
## tile pass) follows from calling it.
const ZOOM_BUTTON_STEP := 1.35   ## Reference 13464. Deliberately coarser than
	## `ZOOM_WHEEL_STEP` (1.15): a wheel notch is cheap and repeatable, a
	## 44 px tap is neither.

func zoom_step(factor: float) -> void:
	_zoom_at(size * 0.5, factor)

## The navpad's ✋. See `_pan_mode`'s own doc comment for why this latches
## rather than holding. Dropping `_panning` on the way out matters: turning
## the mode off mid-drag would otherwise leave a pan running with no button
## down to end it, since the release would no longer take this file's branch.
func set_pan_mode(on: bool) -> void:
	if _pan_mode == on:
		return
	_pan_mode = on
	if not on:
		_panning = false
	if _pan_btn != null:
		_pan_btn.set_pressed_no_signal(on)
		## Accent fill, dark glyph -- the phone canvas's own on-toggle idiom
		## (`#e0a34a` track, `#141617` knob), not a border or a tint, so the
		## one latched control in the column is unmistakable at arm's length.
		_navpad_paint(_pan_btn,
			DccTheme.c("accent") if on else DccTheme.c("panel"),
			DccTheme.c("bg") if on else DccTheme.c("text"))

func pan_mode() -> bool:
	return _pan_mode

## `_civMoveViewTo(x, y)` -- the reference's context-menu "📍 Move viewer to"
## op and the Faction Roster's "(focus camera)" link. Centres grid cell
## `(gx, gy)` in the viewport at the current zoom; no zoom change, matching
## the reference, which pans only.
##
## The camera's own contract (`_zoom_at` above) is `screen = position +
## local * zoom`, where `local` is `overlay`'s unscaled control space --
## `overlay` is a FULL_RECT child of `_camera`, so `displayed_rect()` is
## already in exactly that space. Solving that for "put this local point at
## the viewport centre" is the one line below; no other pan/zoom state is
## touched, so a following wheel-zoom still pivots correctly.
func move_view_to(gx: float, gy: float) -> void:
	var rect: Rect2 = overlay.displayed_rect()
	if rect.size.x <= 0.0:
		return
	var g := _bridge.grid_size()
	if g.x <= 0 or g.y <= 0:
		return
	var local := rect.position + Vector2((gx + 0.5) / float(g.x), (gy + 0.5) / float(g.y)) * rect.size
	_camera.position = size * 0.5 - local * _zoom
	_update_lod()

## `_viewFill()` (reference 13294) -- the default/reset view, which in this app
## means **cover**, not fit: the map fills the display and whichever axis has
## slack loses it off the edges. Called from `refresh()` so a new world never
## opens scrolled off into whatever corner the previous one was zoomed into,
## and from the navpad's ⟳, which is what the reference's own `zoomReset`
## calls (13466) -- deliberately *not* `resetView()` (13390, `scale=1, pan=0`),
## which has not been what that button does since the reference's v1.13.
##
## This used to be plain fit (`_zoom = 1`, `position = ZERO`), which is
## `_camera` at identity over a `STRETCH_KEEP_ASPECT_CENTERED` raster --
## visibly the letterboxed state the reference's own v1.01 was raised to fix
## ("eliminate unused letterbox space"), and on a portrait phone against a
## square world those dead bands are most of the screen. Owner decision,
## 2026-08-23: reset matches the reference and covers.
##
## The scale: `_viewCoverScale()` is `max(1, availW/natW, availH/natH)` over
## the canvas's *natural* size. This camera's `_zoom == 1` is already the
## letterbox-fit rect rather than a natural pixel size, so the same quantity
## here is the fit rect's own shortfall against the viewport -- and it is
## `>= 1` by construction, which is the reference's `max(1, ...)` floor for
## free. `overlay.displayed_rect()` is that rect, reused rather than
## recomputing the fit math a third time.
##
## The pan: the reference sets `panX/panY = 0` and lets `_viewClampFill()`
## (13295) settle it. Worked through, that lands the map exactly aligned on
## the tight axis and **asymmetrically cropped on the loose one** -- an
## artifact of `transform-origin: 0 0` over a flex-centred `.canvas-wrap`,
## not an intent; the reference's own comment at 13290 says "cover scale,
## centred". Centred is what this does, so the crop is even on both edges.
## Deviation recorded rather than taken silently (`CLAUDE.md`), and it is the
## only one -- the scale is the reference's, exactly.
##
## The reference's standing pan clamp is deliberately **not** ported. It runs
## on every `applyView()`, not just reset, so it is a change to all four pan
## paths (MMB, Space+LMB, pan gesture, navpad pan mode) rather than to this
## function -- and it would fight `ZOOM_MIN = 0.4`, which lets this camera
## zoom below fit where the reference floors at fit. Reset restores a known
## view, which is the whole point of the button; recorded as open in
## `GUI_GAP_REGISTER.md` rather than bundled in here.
func reset_view() -> void:
	var cover := 1.0
	var origin := Vector2.ZERO
	var rect: Rect2 = overlay.displayed_rect()
	if rect.size.x > 0.0 and rect.size.y > 0.0 and size.x > 0.0 and size.y > 0.0:
		cover = clampf(maxf(size.x / rect.size.x, size.y / rect.size.y), 1.0, _zoom_max)
		origin = size * 0.5 - rect.get_center() * cover
	_zoom = cover
	_camera.scale = Vector2(_zoom, _zoom)
	_camera.position = origin
	## `zoomReset` clears `panMode` as well (13466) -- a reset that left the
	## hand latched would put the view back but not the input mode, which is
	## half a reset.
	set_pan_mode(false)
	_update_zoom_readout()
	_update_scale_bar()   ## `lodSpanKm()` -- see its own doc comment.
	overlay.set_camera_zoom(_zoom)   ## See `_zoom_at()`'s own comment on why this call exists.
	## Every existing deep-zoom tile belongs to whatever world/size was live
	## before this reset -- `refresh()` calls this on every new
	## `generate()`/`load_save()`, so a stale tile from a *different* world
	## must not survive to be shown (or, worse, silently reused because its
	## `(tx, ty)` key happens to overlap) against the new one.
	## `_update_lod()` below repopulates from scratch if the new view still
	## qualifies for deep zoom (a small preset can, even at `_zoom == 1.0` --
	## see this file's own `LOD_PX_PER_CELL_THRESHOLD` doc comment).
	_clear_lod_tiles()
	_update_lod()

## `GUI_GAP_REGISTER.md`'s top-right-readout note: `design/Cartalith DCC Shell
## .dc.html` draws `2D · equirect · z 5.2` over `relief · atlas preset` here --
## projection and the active style preset, not grid size and extent (those are
## `_bridge.grid_size()`/`last_width_km`/`last_height_km`, and already have a
## home: the WORLD dock readout and the Sample panel). "2D" and "equirect" are
## not live lookups; they are honest constants, not filler -- this port has no
## camera projection to switch and works in one flat km grid throughout
## (`DCC_SHELL_SPEC.md` §2.4's own "this port works in one flat km projection
## throughout"). Only the zoom and the style name are runtime state.
## **Is it safe to ask the engine anything right now?**
##
## Seven functions in this file used to open `if _bridge == null or not
## _bridge.has_world`, and every one of them could still reach a `#[func]` on
## `WorldGen` during a generate. `EngineBridge.generate()` hands the object to a
## worker `Thread` that holds it mutably borrowed until `_finish` (which is why
## `_params_cache` exists there at all), so any `#[func]` reached meanwhile
## aborts with `Gd<T>::bind() failed, already bound`.
##
## `has_world` does not cover it: it is false only until the FIRST world lands.
## On a re-generate it stays true for the whole run, so a mouse move over the
## viewport (`_coords_text`), a camera nudge (`_update_lod`, `_update_zoom_
## readout`) or a resize was enough. Surfaced by `_genstage_probe.gd`, which ran
## a generate with the LOD view up and logged the panic repeatedly at what was
## then line 1384 -- `grid_size()` inside `_update_lod`.
##
## Both conditions mean the same thing to every caller: there is no world this
## frame that can be read. So they take the branch they already had for the
## no-world case rather than needing a second one.
func _engine_readable() -> bool:
	return _bridge != null and _bridge.has_world and not _bridge.generating

func _update_zoom_readout() -> void:
	if not _engine_readable():
		return
	_readout_label.text = "2D · equirect · z%.1f\n%s" % [_zoom, _style_readout]

## `render_workspace.gd` owns the Map style preset (the five chips plus
## "Custom") as its own UI-only state -- nothing in the engine tracks "the
## active look" (`GUI_GAP_REGISTER.md`'s own gap note on this readout: "the
## style preset [is] simply not surfaced on the map at all"). Pushed here
## rather than read from `app` on every frame, matching `set_camera_zoom()`'s
## push-not-poll pattern on `overlay` and keeping this leaf node ignorant of
## which workspace owns which dock.
func set_style_readout(name: String) -> void:
	_style_readout = name
	_update_zoom_readout()

## Phone chrome (`DccShell._build_phone_shell()`) sits on top of this node's
## own edges once the map is edge-to-edge behind it (inset rule "DRAW
## EDGE-TO-EDGE, PAD BY INSET") -- without this, the layers button and the
## coordinate/scale-bar labels would land under the app bar, the rail or the
## tool sheet instead of in the visible gap between them. Desktop/tablet never
## call this, so `_safe_insets` stays at its flat 10 px default there.
func set_safe_insets(insets: Dictionary) -> void:
	_safe_insets = insets
	_apply_touch_scale(float(insets.get("scale", 1.0)))
	_apply_safe_insets()

## HD-03. This node lives in the MAIN viewport, which has no content scale
## (`project.godot`'s `[display]` block records why that is deliberate), so
## every constant in this file is a real device pixel. `NAVPAD_HIT`'s own
## comment asserted the opposite -- "the shipped phone's viewport is ~393 px,
## where that scale is 1.0" -- and it is simply wrong: measured on the real
## composition, `get_viewport_rect()` reports 1080 x 2400 and 1440 x 3168 and
## `DccShell._phone_scale` reports 2.748 and 3.664. A raw 44 px pill is
## therefore 2.83 mm on a 395 ppi panel and **2.19 mm** on the OnePlus 12's
## 510 ppi one, against roughly 7 mm for the 44 dp this shell floors every
## other target at. Not blurry, but the same "unpolished" complaint.
##
## The glyphs are re-rasterised rather than stretched, for the reason
## `DccIcons`' own header gives: growing a 17 px bitmap to 62 px is the smear
## this whole pass exists to remove.
func _apply_touch_scale(scale: float) -> void:
	scale = maxf(1.0, scale)
	if is_equal_approx(scale, _touch_scale) or _layers_btn == null or not _touch:
		return
	_touch_scale = scale
	var hit := maxi(DccTheme.PHONE_TAP_MIN, int(round(44.0 * scale)))
	_layers_btn.custom_minimum_size = Vector2(hit, hit)
	_layers_btn.icon = DccIcons.get_icon("layers", maxi(1, int(round(15.0 * scale))))
	## **The hit rect grew and the paint did not, and on a real handset that is
	## the difference between a control and a smudge.** Two things were wrong
	## together, both found on a OnePlus 6T rather than in a harness:
	##
	## 1. `Button.icon_alignment` defaults to `LEFT`, so the glyph sat in the
	##    top-left *corner* of a 121 px box positioned at the map's own left
	##    edge (`DccShell.phone_content_insets()` returns `left = 0` in
	##    portrait, deliberately -- the map is edge-to-edge). Measured: the
	##    36 px glyph occupied x 2-37, y 307-342, i.e. flush against the panel
	##    edge with the remaining 84 px of its own target empty to the right.
	##    It reads as a clipped icon, which is what it was reported as.
	## 2. `flat = true` **suppresses the background stylebox entirely** -- the
	##    exact trap `_navpad_button()`'s own comment below records paying for
	##    once already ("the first cut was flat, and the pills were invisible
	##    over the terrain with only their glyphs showing"). The Layers button
	##    still had it, so `text_dim` grey drew straight onto whatever biome
	##    happened to be under the top-left corner; over this world's coastal
	##    scrub, with a settlement pin behind it, it was unreadable.
	##
	## Fixed together and only on touch, so the desktop 26 px flat glyph in the
	## dock-framed viewport is byte-identical: the same 92 %-alpha scrim pill
	## the navpad already uses, at the same radius, with the glyph centred in
	## it. `modulate` goes back to white because `_navpad_paint()` tints through
	## `icon_*_color` -- leaving both would multiply the tint twice.
	_layers_btn.icon_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_layers_btn.flat = false
	_layers_btn.modulate = Color.WHITE
	_navpad_paint(_layers_btn, DccTheme.c("panel"), DccTheme.c("text"))
	for state in ["normal", "hover", "pressed"]:
		var lsb: StyleBox = _layers_btn.get_theme_stylebox(state)
		if lsb is StyleBoxFlat:
			(lsb as StyleBoxFlat).set_corner_radius_all(int(hit / 2.0))
	if _navpad == null:
		return
	_navpad.add_theme_constant_override("separation",
		maxi(1, int(round(NAVPAD_GAP * scale))))
	for child in _navpad.get_children():
		if not (child is Button) or not child.has_meta(NAVPAD_GLYPH_META):
			continue
		var b := child as Button
		b.custom_minimum_size = Vector2(hit, hit)
		b.icon = DccIcons.get_icon(String(b.get_meta(NAVPAD_GLYPH_META)),
			maxi(1, int(round(17.0 * scale))))
		for state in ["normal", "hover", "pressed"]:
			var sb: StyleBox = b.get_theme_stylebox(state)
			if sb is StyleBoxFlat:
				(sb as StyleBoxFlat).set_corner_radius_all(int(hit / 2.0))
	_navpad.reset_size()

## Sets offsets directly rather than `.position` -- found by screenshot that
## `.position` doesn't hold up here: it's computed against the control's
## *current* size, and for `_readout_label`/`_scale_label` that's still (0, 0)
## the first time this runs (before `refresh()` has ever set their text).
## `grow_horizontal`/`grow_vertical` then auto-expand the rect the next time
## the text changes, growing from whatever edge was implied by that stale
## zero-size baseline -- which, worked through by hand, is exactly the wrong
## edge for a right/bottom-anchored label, and landed the readout ~139 px off
## the left edge of a 393 px screen. Computing both edges from
## `get_minimum_size()` every call sidesteps the growth direction machinery
## entirely instead of trying to keep it fed a correct baseline.
func _apply_safe_insets() -> void:
	if _scale_label == null:
		return  ## Not built yet -- `_ready()` applies the default once itself.
	var l := float(_safe_insets.get("left", 10.0))
	var t := float(_safe_insets.get("top", 10.0))
	var r := float(_safe_insets.get("right", 10.0))
	var b := float(_safe_insets.get("bottom", 10.0))

	var scale_size := _scale_label.get_minimum_size()
	_scale_label.offset_left = l
	_scale_label.offset_right = l + scale_size.x
	_scale_label.offset_top = -b - scale_size.y
	_scale_label.offset_bottom = -b

	var readout_size := _readout_label.get_minimum_size()
	_readout_label.offset_right = -r
	_readout_label.offset_left = -r - readout_size.x
	_readout_label.offset_top = t
	_readout_label.offset_bottom = t + readout_size.y

	var coords_size := _coords_label.get_minimum_size()
	_coords_label.offset_right = -r
	_coords_label.offset_left = -r - coords_size.x
	_coords_label.offset_top = -b - coords_size.y
	_coords_label.offset_bottom = -b

	## `NAVPAD_EDGE` as a floor here for the same reason the navpad takes it
	## below: in portrait `DccShell.phone_content_insets()` returns `left = 0`
	## deliberately (the map is edge-to-edge behind the chrome), and with the
	## button now painting a real 121 px pill rather than a bare glyph, a left
	## offset of 0 makes that pill *tangent to the panel edge*. Measured on the
	## handset after the paint fix: the disc ran x 0-121 with the screen at 0.
	## The navpad's `right:14px` is the canvas's own value for exactly this
	## relationship, and this is the same relationship on the other side.
	## Gated on `_touch` rather than applied unconditionally the way the navpad
	## applies it, because on desktop `l` is 10 and the floor would move a glyph
	## §46 and §48 both measured -- and there it is a flat 26 px flat button with
	## no pill to sit tangent to anything.
	_layers_btn.position = Vector2(maxf(l, float(NAVPAD_EDGE)) if _touch else l, t)

	## The navpad rides the same insets, so it clears the app bar, the bottom
	## bar, the timeline and the gesture strip without a second set of
	## numbers -- and stacks *above* `_coords_label`, which owns this corner.
	if _navpad != null:
		var pad := _navpad.get_combined_minimum_size()
		_navpad.position = Vector2(size.x - maxf(r, float(NAVPAD_EDGE)) - pad.x,
			size.y - b - coords_size.y - float(NAVPAD_GAP) - pad.y)

func _raster() -> TextureRect:
	var t := TextureRect.new()
	t.set_anchors_preset(Control.PRESET_FULL_RECT)
	t.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	## Keep-aspect-centred, so a non-square map is letterboxed rather than
	## stretched -- the whole point of the sized API.
	t.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	t.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	t.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return t

## The touch navpad -- this port's answer to the reference's mobile-only
## `#zoomOverlay` (HTML 749-754), and `GUI_GAP_REGISTER.md` SH-14's whole
## content. Designed rather than transliterated (owner decision, 2026-08-23):
## the reference draws four bare floating web buttons, a mobile-web idiom this
## shell uses nowhere else, so this is the same four functions in the phone
## canvas's own language -- one right-edge column of 44 dp pills, exactly the
## floating cluster `design/Cartalith Android Phone.dc.html`'s artboard
## "01 · VIEWPORT" already puts at `right:14px` with a 10 px gap. Nothing here
## is a new mechanism; it is `_layers_btn` four more times.
##
## **Reachability: every touch device, not phones only.** `_touch` is the gate
## the reference's own `isMobile` gate means -- what that gate is really
## testing is "there is no wheel, no middle button and no space bar", which is
## as true of a tablet as of a phone. `DccShell._phone` would be the wrong
## gate: it is an *aspect-ratio* test (`_PHONE_ASPECT_MAX`) that exists to
## pick a layout, and a tablet fails it and takes the desktop shell -- desktop
## chrome with no mouse, which is precisely the case that needs this most.
## Desktop is excluded because it already has all four (wheel, MMB/Space,
## and now this file's `reset_view()` from the same call).
const NAVPAD_HIT := 44     ## §13's floor. Raw, not `_phone_scale`d, for the
	## same reason `_layers_btn` above is: the shipped phone's viewport is
	## ~393 px (see `_apply_safe_insets()`'s own note), where that scale is
	## 1.0 -- and `_safe_insets`, which positions this, arrives already
	## scaled from `DccShell.phone_content_insets()`.
const NAVPAD_GAP := 10     ## The canvas's own column gap, and above the 8 px
	## adjacent-target minimum.
const NAVPAD_EDGE := 14    ## The canvas's own `right:14px`, used as a *floor*
	## on the safe inset rather than instead of it. Portrait phone reports
	## `right: 0.0` (`DccShell.phone_content_insets()`) because no chrome
	## occupies that edge -- correct for a text readout, wrong for a round
	## 44 px target, which would sit against the bezel with half its area in
	## the palm-rejection zone.

func _build_navpad() -> void:
	if not _touch:
		return
	_navpad = VBoxContainer.new()
	_navpad.add_theme_constant_override("separation", NAVPAD_GAP)
	_navpad.set_anchors_and_offsets_preset(Control.PRESET_TOP_LEFT)
	_navpad.mouse_filter = Control.MOUSE_FILTER_IGNORE   ## The buttons pick;
		## the column itself must not, or it would eat the map between them.
	add_child(_navpad)

	_navpad.add_child(_navpad_button("zoom_in", "Zoom in",
		func(): zoom_step(ZOOM_BUTTON_STEP)))
	_navpad.add_child(_navpad_button("zoom_out", "Zoom out",
		func(): zoom_step(1.0 / ZOOM_BUTTON_STEP)))

	_pan_btn = _navpad_button("tool_pan", "Pan mode", Callable())
	_pan_btn.toggle_mode = true
	## `set_pan_mode()` writes `button_pressed` back (⟳ clears the mode), which
	## would re-enter this handler -- it uses `set_pressed_no_signal()` for
	## exactly that, and this stays the only caller that comes from a finger.
	_pan_btn.toggled.connect(set_pan_mode)
	_navpad.add_child(_pan_btn)

	_navpad.add_child(_navpad_button("view_fill", "Reset view", reset_view))

func _navpad_button(glyph: String, tip: String, on_press: Callable) -> Button:
	var b := Button.new()
	## Deliberately **not** `flat`, unlike `_layers_btn` and every other button
	## in this shell: `Button.flat` suppresses the background stylebox
	## entirely, so a flat button with a `normal` override draws the override
	## nowhere. Caught by screenshot -- the first cut was flat, and the pills
	## were invisible over the terrain with only their glyphs showing.
	b.focus_mode = Control.FOCUS_NONE
	b.set_meta(NAVPAD_GLYPH_META, glyph)
	b.icon = DccIcons.get_icon(glyph, 17)
	## **`Button.icon_alignment` defaults to `LEFT`, and these are icon-only.**
	## Without this the 17 px glyph sits flush against the left edge of its own
	## 44 px pill instead of in the middle of it. Measured off a 2560x1600
	## device capture before the fix: all four glyph centres 12.5-13.0 px left
	## of their circle centre, dy 0 -- so the pills read as four rings with
	## something stuck to one side rather than as buttons.
	##
	## `_apply_touch_scale()` above records this exact trap, found on a real
	## OnePlus 6T, and set `icon_alignment` on `_layers_btn` -- and only on
	## `_layers_btn`. The four pills built here were never given the same line,
	## so the bug it documents was half-fixed for a year of screenshots. It
	## belongs at construction rather than in the touch-scale pass, because it
	## is true of the button on every device, not only a scaled one.
	b.icon_alignment = HORIZONTAL_ALIGNMENT_CENTER
	## Icon-only, so the tooltip is the only accessible name it has.
	b.tooltip_text = tip
	b.custom_minimum_size = Vector2(NAVPAD_HIT, NAVPAD_HIT)
	_navpad_paint(b, DccTheme.c("panel"), DccTheme.c("text"))
	if on_press.is_valid():
		b.pressed.connect(on_press)
	return b

## One pill's fill and glyph colour, across all three states. Tinting the
## glyph through `icon_*_color` rather than `modulate` is what lets the fill
## and the glyph carry different colours -- `modulate` multiplies the whole
## control, so an accent pill would drag its own glyph to accent with it.
func _navpad_paint(b: Button, fill: Color, ink: Color) -> void:
	var pill := StyleBoxFlat.new()
	## `background:rgba(20,22,23,.92)` on every floating map control in
	## `design/Cartalith Android Phone.dc.html` -- a scrim the map shows
	## through, not an opaque disc. Painted fully opaque until 2026-08-25,
	## which over a bright desert or an ice cap read as four black holes
	## punched in the terrain rather than as chrome sitting on top of it.
	pill.bg_color = Color(fill, 0.92)
	pill.set_corner_radius_all(NAVPAD_HIT / 2)
	## A hairline, because unlike every other button in this shell these float
	## over the *map*: `panel` against dark terrain reads as a shape, against
	## bright desert or ice it does not. The canvas's own floating chips carry
	## the same `rgba(255,255,255,.12)` edge for the same reason.
	pill.set_border_width_all(1)
	pill.border_color = DccTheme.c("line")
	b.add_theme_stylebox_override("normal", pill)
	## A visible press, which a 44 px target with no hover state on a
	## touchscreen otherwise has no feedback at all for. Lightened toward the
	## ink rather than swapped for a token, so it works for both fills.
	var down := pill.duplicate() as StyleBoxFlat
	down.bg_color = fill.lerp(ink, 0.22)
	b.add_theme_stylebox_override("hover", down)
	b.add_theme_stylebox_override("pressed", down)
	for state in ["normal", "hover", "pressed"]:
		b.add_theme_color_override("icon_%s_color" % state, ink)

func _chrome(preset: int, align: int) -> Label:
	var l := DccTheme.label("", "text_faint", DccTheme.FS_SMALL)
	l.horizontal_alignment = align
	## PRESET_MODE_MINSIZE bakes offsets from the control's size at call time --
	## zero, during `_ready` -- and nothing recomputes them on resize, which put
	## the right-hand readouts off the edge of the viewport. Anchor without
	## baking, and grow away from the anchored edge.
	l.set_anchors_preset(preset, true)
	var right: bool = align == HORIZONTAL_ALIGNMENT_RIGHT
	var bottom: bool = preset in [Control.PRESET_BOTTOM_LEFT, Control.PRESET_BOTTOM_RIGHT]
	l.grow_horizontal = Control.GROW_DIRECTION_BEGIN if right else Control.GROW_DIRECTION_END
	l.grow_vertical = Control.GROW_DIRECTION_BEGIN if bottom else Control.GROW_DIRECTION_END
	## Positioned properly by `_apply_safe_insets()`, called once at the end of
	## `_ready()` (and again by `set_safe_insets()` on phone rotation) -- left
	## at the anchor's own baseline here rather than duplicating the inset math.
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(l)
	return l

# -- Refresh ------------------------------------------------------------------

func refresh() -> void:
	if not _engine_readable():
		return
	map_view.texture = _bridge.color_texture()
	territory_view.texture = _bridge.territory_texture()
	province_view.texture = _bridge.province_boundary_texture()
	## A picked view survives a regenerate: same view id, new world's data.
	## `set_debug_layer` clears itself if the new world cannot answer it.
	set_debug_layer(_debug_view)
	var g := _bridge.grid_size()
	overlay.set_civ_data(_bridge.settlements(), _bridge.roads(),
		_bridge.sea_routes(), g.x, g.y, _bridge.border_inset_frac())
	refresh_faction_colors()
	## A regenerate empties `InfraTools::routes`, so this also *clears* the
	## route layer rather than leaving the previous world's routes drawn over
	## the new one.
	overlay.set_manual_routes(manual_routes())
	## A town is sized in real metres, so the layout layer needs the map's own
	## km extent to know how many pixels 1.7 km is worth.
	overlay.set_map_width_km(_bridge.last_width_km)
	tool_overlay.set_grid(g.x, g.y)
	_width_km = _bridge.last_width_km
	## `lodMaxZoom()` -- see `ZOOM_MAX_FLOOR`'s own doc comment. Recomputed per
	## world because it is a function of the map's real width in km, which is a
	## generation parameter: an 800 km world reaches z160, a 4000 km one z800,
	## and both land on the same 5 km closest span.
	_zoom_max = maxf(ZOOM_MAX_FLOOR, ceilf(maxf(_width_km, 0.0) / ZOOM_TARGET_SPAN_KM))
	reset_view()   ## Also sets `_readout_label`'s text and the scale bar.
	## Belt-and-suspenders after the two lines above change `_scale_label`'s
	## and `_readout_label`'s text (and so their minimum size): the fixed
	## edge each already has should carry Godot's own grow-direction resize
	## correctly on its own (see `_apply_safe_insets()`'s comment), but a full
	## recompute against the now-current size costs nothing and doesn't rely
	## on trusting that reasoning held.
	_apply_safe_insets()
	## The annotation layers, for the same reason the `set_manual_routes` line
	## above is here: a regenerate must *clear* what the previous world left
	## drawn, not leave it over the new terrain. `WorldGen::landmark_run`'s
	## store is invalidated by `generate`, `absorb`, the rotate path and the
	## project loader -- so `landmarks()` correctly answers `[]` here -- but
	## `MapOverlay._landmarks` is shell-side state and nothing was clearing it,
	## which left world A's rings drawn over world B while the Landmarks panel
	## reported no run. Subsumes the `set_manual_routes` call above; that one
	## stays because its comment is the reasoning for this one.
	refresh_annotations()

## §4.5.5's Icon/Label tools (`UNIFIED_TOOL_PLAN.md` milestone F) -- lighter
## than `refresh()`: placing/editing one icon or label shouldn't re-fetch the
## terrain texture or civ data, only push the updated lists into `overlay`'s
## own annotation layers. See `map_overlay.gd`'s own `_manual_icons`/`_labels`
## doc comment, which already names this method.
func refresh_annotations() -> void:
	if not _engine_readable():
		return
	## Deliberately does NOT re-pull the faction swatches, unlike `refresh()`.
	## `cartography_workspace.gd` registers this as the *drag* handler's tail
	## for both the Icon and Label tools, so it runs once per mouse-motion
	## event; `get_factions()` scans the whole territory raster once per
	## faction (`claimed_cells`), which would put the first O(gw*gh) term into
	## a path whose existing work is all list marshalling. Nothing about a
	## faction's colour can change while a label is being dragged.
	overlay.set_manual_icons(_bridge.icon_list())
	## Generated landmarks (`LANDMARK_GENERATION_SCOPE.md`). Guarded on the
	## method rather than assumed: the landmark bridge is newer than this call
	## site, and an older cdylib must lose the overlay rather than the whole
	## annotation refresh — the same degrade `_has()` gives every other binding
	## in `engine_bridge.gd`.
	if _bridge.has_method("landmarks") and overlay.has_method("set_landmarks"):
		overlay.set_landmarks(_bridge.landmarks())
	overlay.set_labels(_bridge.label_list())
	overlay.set_manual_routes(manual_routes())

## Push the engine's own faction swatches into `map_overlay.gd`, which drew
## its settlement pins from a frozen six-entry copy of them until
## 2026-09-01.
##
## `CivData::faction_rgb` is the single authority the territory wash, the
## Political-control analysis field and `get_factions()`'s roster/banner
## swatch all go through, and its whole reason for existing is that those
## surfaces "cannot disagree". The pins were the fourth surface and were not
## going through it: they indexed `FACTION_COLORS` with a `% 6` wrap, so a
## colour the user set in the Faction roster never reached them, and on a
## seven-faction world faction 7 got faction 1's colour on the pin while the
## wash under it drew `civ_faction_color`'s golden-angle hue -- the exact
## divergence `faction_rgb`'s own doc comment records having already fixed
## once, for the Political-control field.
##
## `color_r`/`color_g`/`color_b` are 0-255 and are documented as "the exact
## swatch `build_territory_texture` paints this faction's cells in", so the
## pin and the wash are now the same number rather than two derivations of
## it. `get_factions()` enumerates `1..=roster.len()`, so the array covers
## every faction that exists, including appended ones past the base six.
##
## Called from `refresh()` -- generate, load, and every other full rebuild --
## and from nowhere hotter, for the reason `refresh_annotations()` states at
## its own head.
##
## **One path is still open and is not this pass's file.** A roster edit that
## changes only a colour goes through `civilization_workspace.gd`'s
## `_on_roster_changed` -> `_refresh_civ_data`, which calls
## `overlay.set_civ_data()` directly and touches nothing here, so the pins
## keep the previous swatch until the next `refresh()`. One added line there
## -- `app.viewport.refresh_faction_colors()`, beside the territory-texture
## write that handler already does for exactly this reason -- closes it, and
## this method is public so that line is the whole change.
func refresh_faction_colors() -> void:
	if _bridge == null or not overlay.has_method("set_faction_colors"):
		return
	var out: Array[Color] = []
	for f in _bridge.get_factions():
		var d: Dictionary = f
		out.append(Color8(int(d.get("color_r", 128)), int(d.get("color_g", 128)),
			int(d.get("color_b", 128))))
	overlay.set_faction_colors(out)

## Every committed Route-tool route, in `route_get`'s own dictionary shape.
## `route_count`/`route_get` are the only readback the Route tool has (see
## `route_count`'s own doc comment in `lib.rs`) -- there is no bulk getter,
## so the loop lives here rather than in `map_overlay.gd`, which is handed
## finished data and never calls the bridge itself. Cheap: a session's routes
## number in the handful, and this runs only on a commit or a regenerate.
func manual_routes() -> Array:
	var out: Array = []
	if _bridge == null:
		return out
	for i in _bridge.route_count():
		var r := _bridge.route_get(i)
		if not r.is_empty():
			out.append(r)
	return out

## The current pan/zoom camera scale (`_zoom`, `_zoom_at`'s own factor) --
## read-only exposure for `label_handles(index, zoom)` callers (`DCC_SHELL_
## SPEC.md` §4.5.5's Label tool), which need the real camera zoom so a
## selected label's on-canvas handles size and stem-offset consistently with
## how large the label itself is currently drawn.
func zoom() -> float:
	return _zoom

func set_layer_visible(layer: String, shown: bool) -> void:
	match layer:
		"territory": territory_view.visible = shown
		"provinces": province_view.visible = shown
		"settlements": overlay.set_show_settlements(shown)
		"roads": overlay.set_show_roads(shown)
		"sea_routes": overlay.set_show_sea_routes(shown)
		## Guarded on the method, unlike its neighbours: this arm is newer than
		## `map_overlay.gd`'s own landmark block in some checkouts, and a match
		## arm that calls a method the overlay does not have takes the whole
		## `set_layer_visible` dispatch down with it -- including the five rows
		## that do work.
		"landmarks":
			if overlay.has_method("set_landmarks_visible"):
				overlay.set_landmarks_visible(shown)
		## `civUrbanLayoutsChk` (`GUI_GAP_REGISTER.md` UM-01). Reveals only
		## once a town's 1.7 km site box is worth pixels -- `map_overlay.gd`'s
		## own "Urban layouts" block owns that gate and states why it is not
		## the reference's `_umLayoutAlpha` km band.
		"urban_layouts": overlay.set_show_urban_layouts(shown)
		_: push_error("ViewportHost: unknown layer '%s'" % layer)

## Answers `map_overlay.gd`'s one-batch-at-a-time request for town layouts.
## Synchronous, on the main thread: generating a town is a few milliseconds,
## the batch is capped by the overlay, and this port has no worker path from
## GDScript into a `#[func]` -- said plainly rather than hidden behind a
## thread that does not exist.
func _on_urban_layouts_needed(indices: PackedInt32Array) -> void:
	if not _engine_readable():
		overlay.set_urban_layouts(indices, [])
		return
	overlay.set_urban_layouts(indices, _bridge.urban_layouts(indices))

## The per-class / per-way-type half of the reference's own layer filters
## (`#explSettlementFilterList`, and `#explShowRoads`'s by-way-type list --
## `design/Cartalith Menu Structure v2.dc.html`, MAP ▸ LAYERS). Passed
## straight through to `map_overlay.gd`, which owns the draw-time test; kept
## as separate entry points from `set_layer_visible` above because these take
## a *sub*-key, not a layer id, and folding them into one string namespace
## would make "settlements" and "settlements/hamlet" collide.
func set_settlement_kind_visible(kind: String, shown: bool) -> void:
	overlay.set_settlement_kind_visible(kind, shown)

func set_way_type_visible(way_type: String, shown: bool) -> void:
	overlay.set_way_type_visible(way_type, shown)

## Sculpt/Paint's shared draft-preview raster (`_preview_layer`, built in
## `_ready()`). Pass `null` to clear it -- an armed tool's own disarm handler
## is expected to do this, the same way `GlobalTools` clears `tool_overlay`'s
## geometry on disarm, so a stale draft never lingers after switching tools.
func set_preview_texture(tex: Texture2D) -> void:
	_preview_layer.texture = tex

## The Layers popover's field raster. `"off"` (or a view this world has no
## input for) clears it -- the popover reads `debug_view()` back to keep its
## own selection honest, so a view that could not be drawn does not stay
## highlighted as though it had been.
func set_debug_layer(view: String) -> void:
	if _bridge == null or view == "off" or not _bridge.has_world:
		_debug_view = "off"
		_debug_layer.texture = null
		return
	var tex := _bridge.debug_texture(view)
	_debug_layer.texture = tex
	_debug_view = view if tex != null else "off"

## Which view `set_debug_layer` actually managed to draw.
func debug_view() -> String:
	return _debug_view

## The reference's own `#dbgOpacity` (0-100%): blends the active field
## raster over the base map so terrain reads through it.
func set_debug_opacity(a: float) -> void:
	_debug_layer.modulate.a = clampf(a, 0.0, 1.0)

func debug_opacity() -> float:
	return _debug_layer.modulate.a

## Screen-space rect of the layers button, so a popover can anchor itself to
## it rather than guessing at the viewport's corner inset (which changes with
## `set_safe_insets()` on phone).
func layers_button_rect() -> Rect2i:
	return Rect2i(Vector2i(_layers_btn.global_position), Vector2i(_layers_btn.size))

## `lodSpanKm()` (reference 10675) -- *"the real-world km currently spanned by
## the LOD viewport (full map width when off/zoom=1) -- the single source of
## truth for both the scale bar and any future 'current view width' readout"*,
## which is `mapWidthKm / max(1, _lodZoom)` there and the same divide here.
##
## This used to print `_width_km` flat at every zoom, so the deepest view the
## camera could reach still read "800 km across" -- the one readout that would
## have told the owner how deep they actually were, saying the same thing at
## z1 and at the cap. Called from `_zoom_at`/`reset_view` now, not only from
## `refresh()`.
func _update_scale_bar() -> void:
	if _width_km <= 0.0:
		_scale_label.text = ""
		return
	var span := _width_km / maxf(1.0, _zoom)
	var gw := _bridge.grid_size().x
	if gw <= 0:
		_scale_label.text = "%s km across" % _fmt_km(span)
		return
	## Cells are square in km, so one quotient describes both axes.
	var per_cell := _width_km / float(gw)
	var cell_text := "%.2f" % per_cell if per_cell < 10.0 else "%.0f" % per_cell
	_scale_label.text = "%s km across  ·  %s km / cell" % [_fmt_km(span), cell_text]

## Enough decimals to stay informative once the span is small -- the whole
## point of showing it is that it changes, and `%.0f` would print "5 km" for
## everything from 4.5 to 5.5 at the deepest zoom this camera now reaches.
func _fmt_km(km: float) -> String:
	if km >= 100.0:
		return "%.0f" % km
	if km >= 10.0:
		return "%.1f" % km
	return "%.2f" % km

func _on_hovered(data: Variant, index: int) -> void:
	settlement_hovered.emit(data, index)

func _on_sampled(gx: float, gy: float, valid: bool) -> void:
	_coords_label.text = _coords_text(gx, gy) if valid else ""
	cursor_sampled.emit(gx, gy, valid)

## §10's bottom-right readout (`4 812 km E · 1 093 km N · 1 462 m`): a real
## world-space position from the same km-per-cell conversion
## `_update_scale_bar()` already trusts ("cells are square in km, so one
## quotient describes both axes"), plus the *committed* elevation at that
## cell (`EngineBridge.sample_cell()`'s `elevation_m`). Grid row 0 is the
## map's north edge (screen-up, same convention `map_overlay.gd` draws
## against), so "km N" counts up from the *south* edge -- `(gh - gy)`, not
## `gy` -- the way a real northing reads on a map, increasing northward.
## `""` before any world, matching this label's own previous behaviour.
##
## **The design's own `→ 1 582 m` draft-stamp suffix
## (`DCC_SHELL_SPEC.md` §10) is deliberately not built here.**
## `sample_cell()` reads only `WorldState::field` -- `sample_bridge.rs`'s own
## `sample_refs()` wires `field: &ws.field` and nothing from `self.sculpt`'s
## draft `PassBuffer` -- so it can never see an uncommitted stroke.
## `build_sculpt_preview_texture()` does composite the draft, but only into a
## full-grid *colourised* RGB texture (`RenderCtx::with_appearance`'s
## hillshade/AO/appearance pipeline runs first); there is no `#[func]` that
## returns the draft's raw elevation at one cell, and reverse-engineering an
## elevation out of already-shaded pixel colour is not a real reading. This
## needs one new Rust entry point (something like `sample_bridge::
## sample_cell` but reading `scratch` -- the sculpt draft's own
## `preview_into` output -- instead of `ws.field`); `GUI_GAP_REGISTER.md`
## classified SH-06 (A) on the premise that this call already existed. It
## doesn't -- corrected there and in `CHANGELOG.md`.
func _coords_text(gx: float, gy: float) -> String:
	if not _engine_readable():
		return ""
	var g := _bridge.grid_size()
	if g.x <= 0 or g.y <= 0 or _width_km <= 0.0:
		return ""
	var per_cell := _width_km / float(g.x)
	var east_km := gx * per_cell
	var north_km := float(g.y - gy) * per_cell
	var text := "%s km E  ·  %s km N" % [_fmt_thousands(east_km, 0), _fmt_thousands(north_km, 0)]
	var cell := _bridge.sample_cell(int(round(gx)), int(round(gy)))
	if cell.has("elevation_m"):
		text += "  ·  %s m" % _fmt_thousands(float(cell["elevation_m"]), 0)
	return text

## Space-grouped thousands ("4 812"), matching the design mockup's own
## formatting. The same small helper as `journey_planner_view.gd`'s own
## `_fmt_thousands` -- duplicated rather than shared, matching this
## project's existing pattern of a private per-file formatter rather than a
## new cross-file utility for one call site each.
func _fmt_thousands(v: float, decimals: int) -> String:
	var s := ("%.*f" % [decimals, v])
	var neg := s.begins_with("-")
	if neg:
		s = s.substr(1)
	var dot := s.find(".")
	var int_part := s if dot < 0 else s.substr(0, dot)
	var frac_part := "" if dot < 0 else s.substr(dot)
	var out := ""
	var count := 0
	for i in range(int_part.length() - 1, -1, -1):
		out = int_part[i] + out
		count += 1
		if count % 3 == 0 and i > 0:
			out = " " + out
	return ("-" if neg else "") + out + frac_part

# -- Deep-zoom tile compositing (`LOD_TILING_INTEGRATION_SCOPE.md` M1) --------

## Recomputes which deep-zoom tiles belong on screen and asks
## `EngineBridge.lod_synthesize_tile` for any not already displayed. Called
## from every place the camera or the control's own size can change what's
## visible: `_zoom_at` (wheel/pinch), the pan branch of `_input`'s
## `MouseMotion` handling, `reset_view` (new/loaded world), and the
## `resized` signal (`_ready`).
##
## Deliberately **not** debounced to "once per frame" or "only on pan-end":
## the real cost here is the Rust synthesis call, and that only happens for
## a chunk this method has not already built a `Sprite2D` for
## (`_lod_tiles`, keyed by `(z, col, row)`) -- a pan or zoom that doesn't
## cross a chunk boundary touches no new chunks and makes no Rust call at all,
## so running this once per input event costs only the screen<->grid geometry
## below, not a synthesis pass. That stays true at every depth, which is the
## other half of what the pyramid buys: a chunk's *screen* size is roughly
## `lod_bridge::TILE_PX` at any level, so the number on screen is bounded and
## the cost of a viewful does not grow with zoom.
##
## Which tiles are "visible" is resolved with plain arithmetic here, in
## GDScript, rather than via `cartalith_spatial::QuadTree::query_region` --
## see `lod_bridge.rs`'s own module doc for why: building a quadtree over
## the live height field just to answer a pure index-range question would
## cost a real O(field size) scan for a query whose real cost, done
## directly, is O(tiles on screen). This file already owns the
## camera<->grid transform (`_zoom_at`, `map_overlay.gd`'s
## `_displayed_rect()`/`_cell_to_screen`) that the same arithmetic needs, so
## nothing new is invented here, only reused.
func _update_lod() -> void:
	if not _engine_readable():
		_set_lod_active(false)
		return

	var g := _bridge.grid_size()
	if g.x <= 1 or g.y <= 1 or size.x <= 0.0 or size.y <= 0.0:
		_set_lod_active(false)
		return

	## The native (`_zoom == 1.0`) fit rect -- identical math to
	## `map_overlay.gd`'s own `_displayed_rect()`, computed against `size`
	## (this control's own, which `_camera`'s always equals -- see its own
	## doc comment above) since that is the space every tile is positioned
	## in: `_camera`'s `scale`/`position` transform is what pan/zoom
	## actually is, so a tile positioned once in native space tracks the
	## camera for free, the same way `map_view` itself already does.
	var native_scale := minf(size.x / float(g.x), size.y / float(g.y))
	if native_scale <= 0.0:
		_set_lod_active(false)
		return
	var screen_px_per_cell := native_scale * _zoom
	if screen_px_per_cell <= LOD_PX_PER_CELL_THRESHOLD or _zoom <= LOD_AUTO_ZOOM:
		## Below the threshold the pyramid goes down in either mode, and the
		## manual request goes with it -- otherwise a request made once would
		## silently re-enter on every later zoom, which is auto by another name.
		_lod_manual_request = false
		_set_lod_active(false)
		return
	## Manual mode: past the threshold, but nobody asked. The reference's own
	## gate, one level up from the wheel handler because this file has no
	## single zoom entry point to hang it on.
	if not _lod_auto and not _lod_active and not _lod_manual_request:
		_set_lod_active(false)
		return

	## `pyramidLevelForZoom` against this world's own width, decided in Rust
	## (`lod_bridge::level_for_zoom`) so this file cannot pick a level the
	## synthesis would resolve differently. `<= 0` tiles per axis means a
	## binary built before this milestone: stay Z1-only, exactly as the
	## retired `lod_tile_cells() == 0` check did.
	var z: int = _bridge.lod_level_for_zoom(screen_px_per_cell)
	## `PARITY_AUDIT.md` §23 F14: `lod_level_for_zoom` already clamps to
	## `lod_bridge::MAX_LEVEL` internally today, so this is currently a
	## no-op in practice -- but nothing on this side enforced it, and the
	## engine's own ceiling (`lod_max_level()`, real and callable, just
	## unread until now) is the honest source of truth for it rather than a
	## number this file would otherwise have to assume. `> 0` guard: a
	## binary built before `lod_max_level` existed answers `0`, which would
	## otherwise clamp every level to Z0 forever.
	var max_z := _bridge.lod_max_level()
	if max_z > 0:
		z = mini(z, max_z)
	var n: int = _bridge.lod_tiles_per_axis(z)
	if n <= 0:
		_set_lod_active(false)
		return

	var displayed_size := Vector2(g.x, g.y) * native_scale
	var displayed_origin := (size - displayed_size) * 0.5

	## Screen rect -> `_camera`-local rect, inverting `_zoom_at`'s own
	## `screen = position + local * zoom` relation.
	var local_top_left := (Vector2.ZERO - _camera.position) / _zoom
	var local_bottom_right := (size - _camera.position) / _zoom

	## Local rect -> grid-cell rect: the inverse of `map_overlay.gd`'s
	## `_cell_to_screen` mapping, minus its marker-placement `+0.5`
	## centering (irrelevant to the cell/pixel correspondence itself).
	## Clamped to the real grid -- a non-square map's own letterbox bars,
	## and panning past the map's edge, both map outside `[0, gw] x [0, gh]`
	## otherwise.
	var gx0 := clampf((local_top_left.x - displayed_origin.x) / displayed_size.x * g.x, 0.0, float(g.x))
	var gy0 := clampf((local_top_left.y - displayed_origin.y) / displayed_size.y * g.y, 0.0, float(g.y))
	var gx1 := clampf((local_bottom_right.x - displayed_origin.x) / displayed_size.x * g.x, 0.0, float(g.x))
	var gy1 := clampf((local_bottom_right.y - displayed_origin.y) / displayed_size.y * g.y, 0.0, float(g.y))
	if gx1 <= gx0 or gy1 <= gy0:
		_set_lod_active(false)
		return

	## Which chunks of level `z` that rect touches. The pyramid splits the
	## *sample* range `[0, gw-1]` into `n` steps (`pyramid_tile_bounds`), and
	## a cell's sample coordinate is its texel centre, `cell - 0.5` in the
	## `[0, gw]` span the rect above is written in -- hence the shift, which
	## is a whole tile's worth at the deepest levels and cannot be dropped.
	var step := _lod_step(g, n)
	var c0 := clampi(int(floor((gx0 - 0.5) / step.x)), 0, n - 1)
	var c1 := clampi(int(floor((gx1 - 0.5) / step.x)), 0, n - 1)
	var r0 := clampi(int(floor((gy0 - 0.5) / step.y)), 0, n - 1)
	var r1 := clampi(int(floor((gy1 - 0.5) / step.y)), 0, n - 1)

	var wanted: Dictionary = {}   ## key -> Vector3i(z, col, row)
	for row in range(r0, r1 + 1):
		for col in range(c0, c1 + 1):
			wanted["%d,%d,%d" % [z, col, row]] = Vector3i(z, col, row)

	## Only the tiles this call would actually have to *synthesize* --
	## already-built ones cost nothing but a reposition below, so they stay
	## in `wanted` (and so never get freed) regardless of whether they'd have
	## made the closest-`N` cut; only the genuinely-missing set below competes
	## for the per-call budget.
	var missing: Dictionary = {}
	for key in wanted.keys():
		if not _lod_tiles.has(key):
			missing[key] = wanted[key]

	var build_keys: Dictionary = missing
	var trimmed := false
	if missing.size() > MAX_LOD_TILES_PER_UPDATE:
		build_keys = _nearest_tiles(missing, Vector2((c0 + c1) * 0.5, (r0 + r1) * 0.5))
		trimmed = true

	## Whatever didn't make this call's budget -- replaces the previous
	## backlog wholesale rather than merging into it, so a tile that scrolled
	## back out of `wanted` since the last call is dropped here for free
	## instead of `_process()` wastefully building it later. See
	## `_lod_backlog`'s own doc comment.
	_lod_backlog.clear()
	if trimmed:
		for key in missing.keys():
			if not build_keys.has(key):
				_lod_backlog[key] = missing[key]
	_lod_backlog_n = n
	_lod_backlog_grid = g
	_lod_backlog_origin = displayed_origin
	_lod_backlog_size = displayed_size
	set_process(not _lod_backlog.is_empty())

	_apply_lod_tiles(wanted, build_keys, n, g, displayed_origin, displayed_size)
	_set_lod_active(true)

## `pyramid_tile_bounds`' own step: the *sample* range `[0, gw-1] x [0, gh-1]`
## divided `n` ways per axis. Stated once because four call sites need it and
## the `- 1` is the whole reason adjacent chunks share an edge sample.
func _lod_step(g: Vector2i, n: int) -> Vector2:
	return Vector2(float(g.x - 1) / float(n), float(g.y - 1) / float(n))

## Trims a candidate set (key -> `Vector3i` chunk id -- `_update_lod()`'s own
## `missing`) to `MAX_LOD_TILES_PER_UPDATE` entries closest to `centre`
## (in chunk-index space) -- see `MAX_LOD_TILES_PER_UPDATE`'s own doc comment
## for why this bound exists at all rather than synthesizing every candidate
## unconditionally.
func _nearest_tiles(wanted: Dictionary, centre: Vector2) -> Dictionary:
	var keys: Array = wanted.keys()
	keys.sort_custom(func(a, b) -> bool:
		var ai: Vector3i = wanted[a]
		var bi: Vector3i = wanted[b]
		return Vector2(ai.y, ai.z).distance_squared_to(centre) \
			< Vector2(bi.y, bi.z).distance_squared_to(centre))
	var trimmed: Dictionary = {}
	for i in range(MAX_LOD_TILES_PER_UPDATE):
		trimmed[keys[i]] = wanted[keys[i]]
	return trimmed

## Reconciles `_lod_tiles` against `wanted` (key -> `Vector3i` chunk id):
## frees what is no longer visible, repositions what is, and only calls
## `EngineBridge.lod_synthesize_tile` for a key in `build_keys` -- everything
## else missing from `wanted` was already queued in `_lod_backlog` by
## `_update_lod()`, and `_process()` builds it shortly.
##
## A tile at the wrong *level* needs no special handling since 2026-08-24:
## the level is part of the key, so a level change leaves every old key out
## of `wanted` and the free loop below collects them.
func _apply_lod_tiles(wanted: Dictionary, build_keys: Dictionary, n: int, g: Vector2i, displayed_origin: Vector2, displayed_size: Vector2) -> void:
	for key in _lod_tiles.keys().duplicate():
		if not wanted.has(key):
			(_lod_tiles[key] as Sprite2D).queue_free()
			_lod_tiles.erase(key)

	for key in wanted.keys():
		var idx: Vector3i = wanted[key]
		if _lod_tiles.has(key):
			var existing := _lod_tiles[key] as Sprite2D
			var rect = _lod_tile_rect(idx, existing.texture, n, g, displayed_origin, displayed_size)
			if rect != null:
				_place_lod_tile(existing, rect)
			continue
		if build_keys.has(key):
			_build_lod_tile(key, idx, n, g, displayed_origin, displayed_size)
		## else: outside this call's synthesis budget, already queued in
		## `_lod_backlog` by `_update_lod()`, built by `_process()` shortly.

## The screen rect one chunk occupies, in `_camera`-local space.
##
## Not simply "the chunk's cell footprint mapped through the map rect": the
## two grids use different conventions and at deep zoom the difference is
## most of a screen. `amplify_region` maps output texel `ox` to sample
## coordinate `b.x + ox/(tw-1) * b.w` -- endpoints inclusive, texel *centres*
## on sample coordinates -- while the base raster's own texel `i` covers the
## span `[i, i+1)` cells, so sample coordinate `c` sits at cell `c + 0.5`. A
## sprite of width `W` puts its texel `j`'s centre at `(j+0.5)/tw * W`.
## Solving those three for the rect that lands every tile texel centre on its
## own ground gives the half-texel inset below; at the deepest level, where a
## chunk covers half a cell, dropping it would offset the tile by more than
## its own width.
##
## Adjacent chunks therefore overlap by exactly one texel, which is correct
## and not a seam: the pyramid has them *share* that edge sample
## (`pyramid_tile_bounds`, `w = (gw-1)/n`), so both draw the same value there.
##
## `null` (not a `Rect2`) for a degenerate texture, the same "no tile" answer
## `_build_lod_tile` gives for a `null` synthesis result.
func _lod_tile_rect(idx: Vector3i, tex: Texture2D, n: int, g: Vector2i, displayed_origin: Vector2, displayed_size: Vector2) -> Variant:
	if tex == null:
		return null
	var tw := tex.get_width()
	var th := tex.get_height()
	if tw < 2 or th < 2:
		return null
	if n <= 0:
		return null
	var step := _lod_step(g, n)
	var b := Vector2(idx.y * step.x, idx.z * step.y)
	## Cell-space span of the drawn rect: half a texel out from the first and
	## last sample, and hence `tw/(tw-1)` of the chunk's own sample span.
	var half := Vector2(0.5 * step.x / float(tw - 1), 0.5 * step.y / float(th - 1))
	var c0 := b + Vector2(0.5, 0.5) - half
	var span := Vector2(step.x * tw / float(tw - 1), step.y * th / float(th - 1))
	var scale_v := displayed_size / Vector2(g)
	return Rect2(displayed_origin + c0 * scale_v, span * scale_v)

## Puts one tile sprite over `rect`, which is in `_camera`-local space.
##
## **A `Sprite2D`, not the `TextureRect` this layer used until 2026-08-24**, and
## the reason is the whole difference between deep zoom working and not.
## `gui/common/snap_controls_to_pixels` is on by default and rounds a
## `Control`'s position and size to whole *local* pixels -- and `_camera`'s
## local pixel is `_zoom` screen pixels, so at the depths this camera now
## reaches (z160: the map is 5.5 local px wide) a tile 1.74 local px across was
## snapped to 1 or 2, i.e. 160 or 320 screen px instead of 278. Measured, not
## guessed: a diff of the same frame with the layer shown and hidden came back
## with 40 px vertical and 120 px horizontal bands that the layer changed not
## at all, in a set of tiles whose own arithmetic covered the screen with a
## one-pixel overlap. `Node2D` carries a float transform and is never snapped.
## Nothing else about the tile changes: `Sprite2D` is a `CanvasItem` like the
## `TextureRect` was, sits under the same `_lod_layer` `modulate` fade, takes
## the same `texture_filter`, and hands the shader the same `UV`.
##
## The old code got away with it because a tile was then a fixed 64 coarse
## cells -- several hundred local px at `ZOOM_MAX = 8` -- where one pixel of
## snapping is invisible.
func _place_lod_tile(sprite: Sprite2D, rect: Rect2) -> void:
	var tex_size := sprite.texture.get_size()
	sprite.position = rect.position
	sprite.scale = rect.size / Vector2(maxf(tex_size.x, 1.0), maxf(tex_size.y, 1.0))

## Synthesizes one chunk and adds its `Sprite2D` to `_lod_layer`, recording
## it in `_lod_tiles`. Shared by `_apply_lod_tiles`'s own build loop and
## `_process()`'s backlog drain, which reach it from two different budgets
## (`MAX_LOD_TILES_PER_UPDATE` per input event, or
## `MAX_LOD_TILES_PER_CATCHUP` per idle frame).
func _build_lod_tile(key: String, idx: Vector3i, n: int, g: Vector2i, displayed_origin: Vector2, displayed_size: Vector2) -> void:
	var tex := _bridge.lod_synthesize_tile(idx.x, idx.y, idx.z)
	if tex == null:
		return
	## After synthesis, not before: the rect is derived from the texture's own
	## texel count (`_lod_tile_rect`), which for a non-square map is not
	## `TILE_PX` on both axes.
	var rect = _lod_tile_rect(idx, tex, n, g, displayed_origin, displayed_size)
	if rect == null:
		return
	var tile_node := Sprite2D.new()
	tile_node.texture = tex
	tile_node.centered = false
	## `LINEAR`, not `_raster()`'s `NEAREST` -- the entire reason this
	## milestone exists is to stop showing blocky single-cell squares at
	## deep zoom, so the tile that replaces them must not reintroduce
	## the same artifact at its own, finer texel size.
	tile_node.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
	_place_lod_tile(tile_node, rect)
	## `tex` is a relief-detail shade ratio, not a picture (`lod_bridge.rs`,
	## "What a tile actually contains"). The shader multiplies it into the
	## base map's own colour, sampled from `map_view.texture` over this
	## tile's footprint -- so a deep-zoom tile can no longer disagree with
	## the map it sits on, which is what the pre-2026-08-23 hypsometric
	## tile did at every pixel. The UVs are the same rect in `[0,1]` map
	## space, so the two agree by construction rather than by a second
	## derivation.
	var mat := ShaderMaterial.new()
	mat.shader = LOD_TILE_SHADER
	mat.set_shader_parameter("base_tex", map_view.texture)
	mat.set_shader_parameter("base_uv0", (rect.position - displayed_origin) / displayed_size)
	mat.set_shader_parameter("base_uv1", (rect.end - displayed_origin) / displayed_size)
	tile_node.material = mat
	_lod_layer.add_child(tile_node)
	_lod_tiles[key] = tile_node

## Backlog catch-up (`_lod_backlog`'s own doc comment): drains up to
## `MAX_LOD_TILES_PER_CATCHUP` entries per frame, reusing the geometry the
## `_update_lod()` call that filled the backlog already computed -- safe to
## reuse because any camera motion that would make it stale also triggers a
## fresh `_update_lod()` call first, which replaces the whole backlog (and
## this geometry) before `_process()` next runs. Disables its own per-frame
## processing the moment the backlog empties, so this costs nothing once
## everything wanted is on screen.
func _process(_delta: float) -> void:
	if _lod_backlog.is_empty():
		set_process(false)
		return
	var n := 0
	for key in _lod_backlog.keys().duplicate():
		if n >= MAX_LOD_TILES_PER_CATCHUP:
			break
		var idx: Vector3i = _lod_backlog[key]
		_lod_backlog.erase(key)
		if not _lod_tiles.has(key):   ## Not already built by a call in between.
			_build_lod_tile(key, idx, _lod_backlog_n, _lod_backlog_grid,
				_lod_backlog_origin, _lod_backlog_size)
		n += 1
	if _lod_backlog.is_empty():
		set_process(false)

## Frees every live tile (they belong to whatever world/size was live before)
## and whenever `_set_lod_active(false)` turns the layer off.
func _clear_lod_tiles() -> void:
	for key in _lod_tiles.keys():
		(_lod_tiles[key] as Sprite2D).queue_free()
	_lod_tiles.clear()
	_lod_backlog.clear()
	set_process(false)

## Fades `_lod_layer` in or out and frees its tiles on the way out. A no-op
## when `active` already matches `_lod_active` -- called from several
## early-return paths in `_update_lod()`, most of which run on every camera
## move.
## The grid-cell rectangle the viewport is currently showing, plus the pyramid
## level that rectangle resolves to -- everything `EngineBridge.bake_visible(z,
## x0, y0, x1, y1)` wants and nothing it does not.
##
## That binding is real, callable and, until now, called by nothing: the rect it
## needs was computed inside `_update_lod()` and published nowhere, so
## `Preferences > Tiles & LOD > Atlas cache > Refine detail for the current
## view` (the reference's `#lodRefineBtn`) had no way to say *which* view.
## `UNWIRED_FUNCTIONS.md` called that "the cheapest remaining win in this
## group"; this is it.
##
## The math is `_update_lod()`'s, deliberately duplicated rather than cached
## off it: a cache would be stale for exactly one frame after every pan, which
## is the frame a user who just moved the camera is most likely to press the
## button in. Returns `ok: false` with nothing else when there is no world, a
## degenerate grid, or the camera is off the map entirely.
func visible_grid_rect() -> Dictionary:
	var disp := _map_display_rect()
	if disp.size.x <= 0.0 or disp.size.y <= 0.0:
		return {"ok": false}
	var g := _bridge.grid_size()
	var local_tl := (Vector2.ZERO - _camera.position) / _zoom
	var local_br := (size - _camera.position) / _zoom
	var x0 := clampf((local_tl.x - disp.position.x) / disp.size.x * g.x, 0.0, float(g.x))
	var y0 := clampf((local_tl.y - disp.position.y) / disp.size.y * g.y, 0.0, float(g.y))
	var x1 := clampf((local_br.x - disp.position.x) / disp.size.x * g.x, 0.0, float(g.x))
	var y1 := clampf((local_br.y - disp.position.y) / disp.size.y * g.y, 0.0, float(g.y))
	if x1 <= x0 or y1 <= y0:
		return {"ok": false}
	## Same level `_update_lod()` would pick for this camera, through the same
	## engine call, clamped by the same `lod_max_level()` guard -- so Refine
	## bakes the level the view is about to ask for rather than one it will not
	## read. `screen_px_per_cell` is the native fit scale times the zoom.
	var px_per_cell := (disp.size.x / float(g.x)) * _zoom
	var z: int = _bridge.lod_level_for_zoom(px_per_cell)
	var max_z := _bridge.lod_max_level()
	if max_z > 0:
		z = mini(z, max_z)
	return {"ok": true, "z": z, "x0": x0, "y0": y0, "x1": x1, "y1": y1,
		"px_per_cell": px_per_cell, "lod_active": _lod_active}

## §2.5's Tiled LOD mode. `true` is "auto on zoom" (the default and the
## reference's own); `false` is "manual", where the pyramid is entered only
## through `request_lod_entry()`.
func set_lod_auto(on: bool) -> void:
	if on == _lod_auto:
		return
	_lod_auto = on
	if on:
		_lod_manual_request = false
	_update_lod()

func lod_auto() -> bool:
	return _lod_auto

## Enter the deep-detail view now, if the camera is far enough in for there to
## be one. The reference's `enterLodFromView`. Returns whether the pyramid is
## up afterwards, so a caller can say "you are not zoomed in far enough" rather
## than appear to do nothing -- the failure mode this repository keeps finding.
func request_lod_entry() -> bool:
	_lod_manual_request = true
	_update_lod()
	if not _lod_active:
		## Nothing came up, so the request was against a camera that has no
		## deep detail to show. Drop it rather than leaving it armed to fire on
		## some later zoom the user did not connect to this.
		_lod_manual_request = false
	return _lod_active

## Leave the deep-detail view without moving the camera. Only meaningful in
## manual mode: in auto the next `_update_lod()` would bring it straight back,
## and saying so is better than a control that undoes itself.
func release_lod_entry() -> void:
	_lod_manual_request = false
	if not _lod_auto:
		_set_lod_active(false)

func lod_active() -> bool:
	return _lod_active

func _set_lod_active(active: bool) -> void:
	if active == _lod_active:
		return
	_lod_active = active
	if not active:
		_clear_lod_tiles()
	var tw := create_tween()
	tw.tween_property(_lod_layer, "modulate:a", 1.0 if active else 0.0, 0.15)
	## The export preview is of the full-resolution split and hides while the
	## pyramid is up (reference line 8658's `&& !_lodOn`), so it has to be
	## re-evaluated here and not only when the checkbox moves.
	if _export_grid_layer != null:
		_export_grid_layer.visible = _export_grid_on and not active

# -- Chunk-debug overlay ------------------------------------------------------
#
# The reference's `drawLODChunkDebug` (line 10946) and its three toggles
# (`_lodGrid`/`_lodChunkCol`/`_lodLabels`, line 10933), driven there by the
# `lodDbgSeg` segmented control (line 1266) under an "Atlas cache ▸ Chunk debug
# overlay" accordion. This port hangs them on `Help ▸ LOD debug` instead --
# the shell has no Atlas panel, and a developer overlay is what Help's own
# `Generation info…` row already is.
#
# **Two of the reference's four chunk states are not reachable here, and are
# omitted rather than faked.** `CHUNK_STATE_COL` there is
# `baked / edited / cached / unexplored`. This overlay can only annotate chunks
# that exist as live tiles, so `unexplored` can never be drawn; and `edited`
# reads the reference's `_lodEdits` store, which this port has no equivalent of
# (`composeTileEdits`/`composeEditInto` are unported).
#
# **What the third state measures was corrected on 2026-08-31.** It used to be
# labelled `baked` / `cached`, and described here as "is this chunk served from
# the baked atlas, or synthesized" -- a distinction this port cannot draw,
# because `_build_lod_tile()` calls `lod_synthesize_tile()` unconditionally and
# no shell file calls `atlas_tile_png()`. *Every* drawn tile is synthesized, so
# a legend answering that question had one correct answer and was reading the
# wrong flag to give it. What `atlas_is_covered()` actually answers is store
# coverage -- is this chunk, or an ancestor of it, in the on-disk atlas -- which
# is a real thing to want to see while baking, and is what the two labels now
# say. Restore the provenance wording in the same commit that makes
# `_build_lod_tile` try the atlas first; `menus.gd`'s `_build_atlas_cache_menu`
# header carries the shader constraint that stops that being a one-line branch.

## Reference `LOD_LEVEL_COLS` (line 10934), verbatim -- red/blue/green/yellow/
## cyan/magenta, indexed `z % 6`, so adjacent pyramid levels never share a
## grid colour.
const LOD_LEVEL_COLS: Array[Color] = [
	Color8(235, 80, 80), Color8(80, 140, 235), Color8(80, 205, 120),
	Color8(235, 205, 72), Color8(80, 215, 225), Color8(215, 90, 215),
]

## Reference `CHUNK_STATE_COL` (line 10935), the two entries this port can
## actually answer for. See the note above on the two that are omitted, and on
## what these two now mean: atlas *coverage*, not the drawn tile's provenance.
const CHUNK_COL_IN_ATLAS := Color8(80, 200, 110)
const CHUNK_COL_NOT_IN_ATLAS := Color8(90, 140, 210)

## Turn one chunk-debug layer on or off. `which` is `"grid"`, `"colors"` or
## `"labels"` -- the reference's own `data-g` values (`grid`/`col`/`lbl`),
## spelled out here because a menu row is not a three-character dataset key.
func set_lod_debug(which: String, on: bool) -> void:
	match which:
		"grid": _lod_dbg_grid = on
		"colors": _lod_dbg_colors = on
		"labels": _lod_dbg_labels = on
		_: return
	## The reference re-renders only when the LOD view is up (`if(_lodOn)
	## renderNow()`); here the layer is a child of `_lod_layer`, so an
	## inactive LOD view draws nothing regardless -- but keeping the node
	## hidden when every toggle is off means `_draw_lod_debug` is not even
	## queued on a camera move.
	_lod_debug_layer.visible = _lod_dbg_grid or _lod_dbg_colors or _lod_dbg_labels
	_lod_debug_layer.queue_redraw()

func lod_debug_enabled(which: String) -> bool:
	match which:
		"grid": return _lod_dbg_grid
		"colors": return _lod_dbg_colors
		"labels": return _lod_dbg_labels
	return false

## Stable per-chunk hue -- the reference's `chunkColorHash` (line 10938),
## `hsl(hash(col,row,z+1), 0.5, 0.55)`. The hash itself is *not* ported
## bit-for-bit: this is a debug tint, `DECISIONS.md` §7d's contract is
## behaviour rather than pixels, and no golden covers it. What is preserved is
## the property the overlay is *for* -- that the tint is stable per chunk and
## uncorrelated between neighbours, so a duplicated or misindexed chunk shows
## up as a repeated colour.
func _chunk_hue(z: int, col: int, row: int) -> Color:
	var h := int(col) * 73856093 ^ int(row) * 19349663 ^ int(z + 1) * 83492791
	return Color.from_hsv(float(absi(h) % 3600) / 3600.0, 0.5, 0.55)

## The rect a live tile actually occupies, read back off the `Sprite2D` rather
## than recomputed from `_lod_tile_rect()`'s inputs. Deliberate: a
## recomputation is a second implementation of the half-texel inset that
## function's own 20-line comment exists to justify, and a debug overlay that
## disagrees with the tiles it annotates is worse than no overlay.
func _lod_sprite_rect(sprite: Sprite2D) -> Rect2:
	var tex_size := sprite.texture.get_size() if sprite.texture != null else Vector2.ONE
	return Rect2(sprite.position, tex_size * sprite.scale)

## The map's own rect in `_camera`-local space -- `_update_lod()`'s own
## `native_scale` / `displayed_size` / `displayed_origin` math, lifted out so
## the export-grid overlay and `visible_grid_rect()` cannot drift from the
## tiles. Returns a zero-size rect when there is no world to measure.
func _map_display_rect() -> Rect2:
	if not _engine_readable():
		return Rect2()
	var g := _bridge.grid_size()
	if g.x <= 1 or g.y <= 1 or size.x <= 0.0 or size.y <= 0.0:
		return Rect2()
	var native_scale := minf(size.x / float(g.x), size.y / float(g.y))
	if native_scale <= 0.0:
		return Rect2()
	var displayed_size := Vector2(g.x, g.y) * native_scale
	return Rect2((size - displayed_size) * 0.5, displayed_size)

## Turn the export tile-border preview on or off, and tell it the split to
## draw. `data_manager_window.gd` owns `cols`/`rows`; this only caches them.
func set_export_tile_grid(on: bool, cols: int = -1, rows: int = -1) -> void:
	if cols > 0:
		_export_grid_cols = cols
	if rows > 0:
		_export_grid_rows = rows
	_export_grid_on = on
	## `if(_showExportGrid && !_lodOn)` (reference line 8658) -- the preview is
	## of the *export* split, which is taken off the full-resolution grid, so
	## drawing it over deep-zoom pyramid tiles would annotate the wrong thing.
	_export_grid_layer.visible = on and not _lod_active
	_export_grid_layer.queue_redraw()

func export_tile_grid_enabled() -> bool:
	return _export_grid_on

## `drawExportTileGrid` (reference line 9602), in this space rather than the
## reference's grid-pixel canvas: a dashed border around the whole map plus
## `cols - 1` verticals and `rows - 1` horizontals at the even split. Colour
## and the dash cadence are the reference's own -- `rgba(255,210,60,0.6)`, dash
## `max(3, GW/120)` -- with `GW` reading as the drawn width here, since this
## overlay is measured in screen-space pixels and the reference's was measured
## in grid cells at 1:1.
func _draw_export_tile_grid() -> void:
	var r := _map_display_rect()
	if r.size.x <= 0.0 or r.size.y <= 0.0:
		return
	var col := Color(1.0, 0.824, 0.235, 0.6)
	var lw := maxf(1.0, r.size.x / 640.0)
	var dash := maxf(3.0, r.size.x / 120.0)
	_dashed_rect(r, col, lw, dash)
	for c in range(1, maxi(1, _export_grid_cols)):
		var x := r.position.x + roundf(r.size.x * float(c) / float(_export_grid_cols))
		_dashed_line(Vector2(x, r.position.y), Vector2(x, r.end.y), col, lw, dash)
	for rw in range(1, maxi(1, _export_grid_rows)):
		var y := r.position.y + roundf(r.size.y * float(rw) / float(_export_grid_rows))
		_dashed_line(Vector2(r.position.x, y), Vector2(r.end.x, y), col, lw, dash)

## Godot's `draw_line` has no dash pattern, so the dashes are drawn. Kept
## private and used only by the overlay above; `draw_dashed_line` exists in
## Godot 4.7 but takes an *aligned* dash length that snaps the last segment,
## which visibly shortens one edge of a rect at these lengths.
func _dashed_line(a: Vector2, b: Vector2, col: Color, width: float, dash: float) -> void:
	var span := a.distance_to(b)
	if span <= 0.0 or dash <= 0.0:
		return
	var dir := (b - a) / span
	var t := 0.0
	while t < span:
		var seg := minf(dash, span - t)
		_export_grid_layer.draw_line(a + dir * t, a + dir * (t + seg), col, width)
		t += dash * 2.0

func _dashed_rect(r: Rect2, col: Color, width: float, dash: float) -> void:
	_dashed_line(r.position, Vector2(r.end.x, r.position.y), col, width, dash)
	_dashed_line(Vector2(r.end.x, r.position.y), r.end, col, width, dash)
	_dashed_line(r.end, Vector2(r.position.x, r.end.y), col, width, dash)
	_dashed_line(Vector2(r.position.x, r.end.y), r.position, col, width, dash)

func _draw_lod_debug() -> void:
	if _lod_tiles.is_empty():
		return
	var font := ThemeDB.fallback_font
	var fs := maxi(9, int(round(float(_lod_debug_layer.size.x) / 110.0)))
	var lh := float(fs) * 1.25
	var lw := maxf(1.5, float(_lod_debug_layer.size.x) / 360.0)

	for key in _lod_tiles.keys():
		var sprite := _lod_tiles[key] as Sprite2D
		if sprite == null or not is_instance_valid(sprite):
			continue
		var parts := String(key).split(",")
		if parts.size() != 3:
			continue
		var z := int(parts[0])
		var col := int(parts[1])
		var row := int(parts[2])
		var r := _lod_sprite_rect(sprite)
		var lc: Color = LOD_LEVEL_COLS[z % LOD_LEVEL_COLS.size()]

		if _lod_dbg_colors:
			var c := _chunk_hue(z, col, row)
			c.a = 0.32
			_lod_debug_layer.draw_rect(r, c, true)

		if _lod_dbg_grid:
			## Faint child-quadrant guides first -- the next level's split, so
			## the tiling still reads when one chunk fills the whole view.
			var faint := lc
			faint.a = 0.30
			var mid := r.position + r.size * 0.5
			_lod_debug_layer.draw_line(
				Vector2(mid.x, r.position.y), Vector2(mid.x, r.end.y), faint, maxf(1.0, lw * 0.5))
			_lod_debug_layer.draw_line(
				Vector2(r.position.x, mid.y), Vector2(r.end.x, mid.y), faint, maxf(1.0, lw * 0.5))
			var edge := lc
			edge.a = 0.95
			_lod_debug_layer.draw_rect(r, edge, false, lw)

		## The reference's own legibility gate: a chunk narrower than about
		## eight glyphs gets no label rather than an unreadable smear.
		if _lod_dbg_labels and r.size.x > float(fs) * 7.0:
			## Store coverage, not provenance: the tile drawn under this
			## label was synthesized either way (see the header note).
			var in_atlas := _bridge.atlas_is_covered(z, col, row)
			var state_txt := "in atlas" if in_atlas else "not in atlas"
			var state_col := CHUNK_COL_IN_ATLAS if in_atlas else CHUNK_COL_NOT_IN_ATLAS
			var box := Rect2(r.position + Vector2(2, 2), Vector2(float(fs) * 8.5, lh * 3.0 + 6.0))
			_lod_debug_layer.draw_rect(box, Color(0.031, 0.039, 0.063, 0.62), true)
			var tp := r.position + Vector2(5, 5 + float(fs))
			_lod_debug_layer.draw_string(font, tp,
				"LOD%d %d,%d" % [z, col, row], HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Color8(221, 255, 238))
			var par := "root" if z <= 0 else "par %d,%d" % [col >> 1, row >> 1]
			_lod_debug_layer.draw_string(font, tp + Vector2(0, lh),
				par, HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Color8(153, 187, 221))
			_lod_debug_layer.draw_string(font, tp + Vector2(0, lh * 2.0),
				state_txt, HORIZONTAL_ALIGNMENT_LEFT, -1, fs, state_col)

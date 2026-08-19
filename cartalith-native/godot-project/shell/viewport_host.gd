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

const OVERLAY_SCRIPT := preload("res://map_overlay.gd")

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
var _coords_label: Label
var _layers_btn: Button
var _bridge: EngineBridge
var _width_km := 0.0

## Default matches the original hardcoded 10 px inset every corner label and
## the layers button used before phone chrome existed. `DccShell` overrides
## this (`set_safe_insets()`) once it knows its own chrome's edges -- on
## phone the map (and this node with it) is edge-to-edge behind that chrome,
## so a flat 10 px would land this node's own chrome under the app bar/rail/
## tool sheet instead of in the visible gap between them.
var _safe_insets := {"left": 10.0, "top": 10.0, "right": 10.0, "bottom": 10.0}

## §13's 44 px floor applies here too -- this button is real, on-screen, and
## tappable on phone, not workspace/dock content this file is exempt from
## touching. Tablet gets the same floor (its own target range is "44-52 px");
## only pointer-first Windows keeps the original compact 26 px hit box.
var _touch := DisplayServer.is_touchscreen_available() and OS.has_feature("mobile")

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
const ZOOM_MAX := 8.0
const ZOOM_WHEEL_STEP := 1.15   ## Multiplicative per wheel notch.
var _zoom := 1.0
var _panning := false
var _pan_last_screen := Vector2.ZERO

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
## `detail_level` doubles the synthesized tile's own resolution once per
## extra octave past the threshold (`lod_bridge::tile_px_for_level`,
## 256/512/1024px) -- 2 matches that function's own `MAX_DETAIL_LEVEL`
## clamp, so this file's idea of "how many tiers exist" cannot drift from
## the Rust side's.
const LOD_MAX_DETAIL_LEVEL := 2
## A real bound, not a cache: right at the threshold crossing is where the
## *most* tiles are visible at once (zooming in further shows fewer, larger
## ones), and synthesizing an unbounded burst in one call the instant a fast
## wheel-scroll crosses the line would stall a frame for no good reason.
## Capped closest-tile-to-centre-first in `_update_lod()`; a real fix
## (background synthesis, or the Z5 atlas cache) is out of this milestone's
## scope (`LOD_TILING_INTEGRATION_SCOPE.md` M3).
const MAX_LOD_TILES_PER_UPDATE := 48

var _lod_layer: Control   ## Child of `_camera`, drawn above map/territory/
	## province and below `overlay` -- same z-order the base raster already
	## has relative to the vector layer, just with a refined terrain image
	## underneath at deep zoom instead of the blocky one.
var _lod_tile_cells := 0   ## `EngineBridge.lod_tile_cells()`, fetched once
	## per world (`0` before any world, or against a binary built before
	## this milestone -- both make `_update_lod()` a no-op, degrading
	## cleanly to Z1-only).
var _lod_tiles: Dictionary = {}   ## `"%d,%d" % [tx, ty]` -> the live
	## `TextureRect` showing that tile, so a pan/zoom that doesn't touch a
	## given tile's index leaves its node (and the Rust call that built it)
	## alone -- see `_update_lod()`'s own doc comment on why this is what
	## keeps calling it once per mouse-motion sample affordable.
var _lod_active := false

func setup(bridge: EngineBridge) -> void:
	_bridge = bridge
	bridge.generation_finished.connect(func(ok: bool): if ok: refresh())
	bridge.world_loaded.connect(refresh)

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
	territory_view = _raster()
	province_view = _raster()
	territory_view.visible = false
	province_view.visible = false
	_camera.add_child(map_view)
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

	## Deep-zoom tile overlay (`LOD_TILING_INTEGRATION_SCOPE.md` milestone
	## M1) -- above the raster/territory/province layers so a refined tile
	## covers the blocky base pixels it's standing in for, below `overlay`
	## so settlements/roads/sea-lanes stay on top exactly as they already
	## are over the plain raster. Starts fully transparent and empty;
	## `_update_lod()` is the only thing that ever adds children to it.
	_lod_layer = Control.new()
	_lod_layer.set_anchors_preset(Control.PRESET_FULL_RECT)
	_lod_layer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_lod_layer.modulate.a = 0.0
	_camera.add_child(_lod_layer)

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

	## §4.5.1's tool feedback (Region marquee, Measure ruler) -- above
	## `overlay` in draw order so it's never hidden behind terrain/vector
	## data, `MOUSE_FILTER_IGNORE` always so it never competes with
	## `overlay`'s own hit-testing for the click that feeds it.
	tool_overlay = ToolOverlay.new()
	tool_overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	tool_overlay.overlay = overlay
	_camera.add_child(tool_overlay)

	## §9's chrome, all corner-anchored so it survives any dock width.
	_scale_label = _chrome(Control.PRESET_BOTTOM_LEFT, HORIZONTAL_ALIGNMENT_LEFT)
	_readout_label = _chrome(Control.PRESET_TOP_RIGHT, HORIZONTAL_ALIGNMENT_RIGHT)
	_coords_label = _chrome(Control.PRESET_BOTTOM_RIGHT, HORIZONTAL_ALIGNMENT_RIGHT)

	_layers_btn = Button.new()
	_layers_btn.flat = true
	_layers_btn.focus_mode = Control.FOCUS_NONE
	_layers_btn.icon = DccIcons.get_icon("layers", 15)
	_layers_btn.tooltip_text = "Layers"
	_layers_btn.modulate = DccTheme.c("text_dim")
	_layers_btn.set_anchors_and_offsets_preset(Control.PRESET_TOP_LEFT)
	var hit := 44 if _touch else 26
	_layers_btn.custom_minimum_size = Vector2(hit, hit)
	_layers_btn.add_theme_stylebox_override("normal",
		DccTheme.flat(DccTheme.c("panel"), 3))
	_layers_btn.pressed.connect(func(): layers_button_pressed.emit())
	add_child(_layers_btn)

	## A resize changes the native fit rect `_update_lod()` positions every
	## tile against (`displayed_origin`/`displayed_size`, computed from
	## `size`) -- without this, resizing the window while zoomed in past the
	## threshold would leave existing tiles positioned against the old
	## window size until the next zoom/pan event.
	resized.connect(_update_lod)

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
func _input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index == MOUSE_BUTTON_WHEEL_UP and mb.pressed:
			_zoom_at(mb.position, ZOOM_WHEEL_STEP)
			get_viewport().set_input_as_handled()
		elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN and mb.pressed:
			_zoom_at(mb.position, 1.0 / ZOOM_WHEEL_STEP)
			get_viewport().set_input_as_handled()
		elif mb.button_index == MOUSE_BUTTON_MIDDLE:
			_panning = mb.pressed
			_pan_last_screen = mb.position
			if mb.pressed:
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
		## finger drag-to-pan is deliberately not wired on touch yet: it would
		## collide with a future armed tool's own drag the same way bare
		## LMB-drag would on desktop, and there is no tool palette yet to
		## arm/disarm Pan explicitly the way §4.5.1 intends. Pinch has no such
		## conflict -- nothing else uses two fingers -- so it is safe now.
		var mg := event as InputEventMagnifyGesture
		_zoom_at(mg.position, mg.factor)
		get_viewport().set_input_as_handled()

## Zooms so the world point under `screen_pt` stays under it: with `pan` the
## camera's position and `zoom` its scale, a local point maps to screen space
## as `screen = pan + local * zoom`. Solving for the new pan that keeps the
## same local point fixed under `screen_pt` after rescaling gives this directly
## -- no pivot_offset needed, which sidesteps Control's own layout-vs-transform
## interactions around pivot entirely.
func _zoom_at(screen_pt: Vector2, factor: float) -> void:
	var new_zoom: float = clampf(_zoom * factor, ZOOM_MIN, ZOOM_MAX)
	if is_equal_approx(new_zoom, _zoom):
		return
	var local_pt := (screen_pt - _camera.position) / _zoom
	_camera.position = screen_pt - local_pt * new_zoom
	_zoom = new_zoom
	_camera.scale = Vector2(_zoom, _zoom)
	_update_zoom_readout()
	_update_lod()

## Back to fit, matching the letterboxed `STRETCH_KEEP_ASPECT_CENTERED` view
## every fresh generate/load already rendered before this camera existed --
## called from `refresh()` so a new world never opens scrolled off into
## whatever corner the previous one was zoomed into.
func reset_view() -> void:
	_zoom = 1.0
	_camera.scale = Vector2.ONE
	_camera.position = Vector2.ZERO
	_update_zoom_readout()
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

func _update_zoom_readout() -> void:
	if _bridge == null or not _bridge.has_world:
		return
	var g := _bridge.grid_size()
	_readout_label.text = "%d x %d  ·  %.0f x %.0f km  ·  z%.1f" % [
		g.x, g.y, _bridge.last_width_km, _bridge.last_height_km, _zoom]

## Phone chrome (`DccShell._build_phone_shell()`) sits on top of this node's
## own edges once the map is edge-to-edge behind it (inset rule "DRAW
## EDGE-TO-EDGE, PAD BY INSET") -- without this, the layers button and the
## coordinate/scale-bar labels would land under the app bar, the rail or the
## tool sheet instead of in the visible gap between them. Desktop/tablet never
## call this, so `_safe_insets` stays at its flat 10 px default there.
func set_safe_insets(insets: Dictionary) -> void:
	_safe_insets = insets
	_apply_safe_insets()

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

	_layers_btn.position = Vector2(l, t)

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
	if _bridge == null or not _bridge.has_world:
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
	tool_overlay.set_grid(g.x, g.y)
	_width_km = _bridge.last_width_km
	_update_scale_bar()
	reset_view()   ## Also sets `_readout_label`'s text -- see its own doc.
	## Belt-and-suspenders after the two lines above change `_scale_label`'s
	## and `_readout_label`'s text (and so their minimum size): the fixed
	## edge each already has should carry Godot's own grow-direction resize
	## correctly on its own (see `_apply_safe_insets()`'s comment), but a full
	## recompute against the now-current size costs nothing and doesn't rely
	## on trusting that reasoning held.
	_apply_safe_insets()

## §4.5.5's Icon/Label tools (`UNIFIED_TOOL_PLAN.md` milestone F) -- lighter
## than `refresh()`: placing/editing one icon or label shouldn't re-fetch the
## terrain texture or civ data, only push the updated lists into `overlay`'s
## own annotation layers. See `map_overlay.gd`'s own `_manual_icons`/`_labels`
## doc comment, which already names this method.
func refresh_annotations() -> void:
	if _bridge == null or not _bridge.has_world:
		return
	overlay.set_manual_icons(_bridge.icon_list())
	overlay.set_labels(_bridge.label_list())

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
		_: push_error("ViewportHost: unknown layer '%s'" % layer)

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

func _update_scale_bar() -> void:
	if _width_km <= 0.0:
		_scale_label.text = ""
		return
	var gw := _bridge.grid_size().x
	if gw <= 0:
		_scale_label.text = "%.0f km across" % _width_km
		return
	## Cells are square in km, so one quotient describes both axes.
	var per_cell := _width_km / float(gw)
	var cell_text := "%.2f" % per_cell if per_cell < 10.0 else "%.0f" % per_cell
	_scale_label.text = "%.0f km across  ·  %s km / cell" % [_width_km, cell_text]

func _on_hovered(data: Variant, index: int) -> void:
	settlement_hovered.emit(data, index)

func _on_sampled(gx: float, gy: float, valid: bool) -> void:
	_coords_label.text = "%d, %d" % [int(gx), int(gy)] if valid else ""
	cursor_sampled.emit(gx, gy, valid)

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
## a tile index this method has not already built a `TextureRect` for
## (`_lod_tiles`, keyed by tile index) -- a pan or zoom that doesn't cross a
## tile boundary (`_lod_tile_cells` grid cells, which at any zoom past the
## threshold is already several hundred screen px) touches no new tiles and
## makes no Rust call at all, so running this once per input event costs
## only the screen<->grid geometry below, not a synthesis pass.
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
	if _bridge == null or not _bridge.has_world:
		_set_lod_active(false)
		return
	if _lod_tile_cells <= 0:
		_lod_tile_cells = _bridge.lod_tile_cells()
	if _lod_tile_cells <= 0:
		return   ## Binary built before this milestone -- stay Z1-only.

	var g := _bridge.grid_size()
	if g.x <= 0 or g.y <= 0 or size.x <= 0.0 or size.y <= 0.0:
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
	if screen_px_per_cell <= LOD_PX_PER_CELL_THRESHOLD:
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

	var tx0 := int(floor(gx0 / _lod_tile_cells))
	var ty0 := int(floor(gy0 / _lod_tile_cells))
	var tx1 := int(floor(maxf(gx0, gx1 - 0.001) / _lod_tile_cells))
	var ty1 := int(floor(maxf(gy0, gy1 - 0.001) / _lod_tile_cells))

	## One detail tier per doubling past the threshold, capped at
	## `LOD_MAX_DETAIL_LEVEL` to match `lod_bridge::MAX_DETAIL_LEVEL` --
	## `log(x)/log(2.0)` is GDScript's `log2`, since `log()` here is natural
	## log.
	var detail_level := clampi(
		int(floor(log(screen_px_per_cell / LOD_PX_PER_CELL_THRESHOLD) / log(2.0))),
		0, LOD_MAX_DETAIL_LEVEL)

	var wanted: Dictionary = {}   ## key -> Vector2i tile index
	for ty in range(ty0, ty1 + 1):
		for tx in range(tx0, tx1 + 1):
			wanted["%d,%d" % [tx, ty]] = Vector2i(tx, ty)

	if wanted.size() > MAX_LOD_TILES_PER_UPDATE:
		wanted = _nearest_tiles(wanted, Vector2((tx0 + tx1) * 0.5, (ty0 + ty1) * 0.5))

	_apply_lod_tiles(wanted, detail_level, g, displayed_origin, displayed_size)
	_set_lod_active(true)

## Trims `wanted` (key -> `Vector2i` tile index) to `MAX_LOD_TILES_PER_UPDATE`
## entries closest to `centre` (in tile-index space) -- see
## `MAX_LOD_TILES_PER_UPDATE`'s own doc comment for why this bound exists at
## all rather than synthesizing every visible tile unconditionally.
func _nearest_tiles(wanted: Dictionary, centre: Vector2) -> Dictionary:
	var keys: Array = wanted.keys()
	keys.sort_custom(func(a, b) -> bool:
		var ai: Vector2i = wanted[a]
		var bi: Vector2i = wanted[b]
		return Vector2(ai).distance_squared_to(centre) < Vector2(bi).distance_squared_to(centre))
	var trimmed: Dictionary = {}
	for i in range(MAX_LOD_TILES_PER_UPDATE):
		trimmed[keys[i]] = wanted[keys[i]]
	return trimmed

## Reconciles `_lod_tiles` against `wanted` (key -> `Vector2i` tile index):
## frees whatever is no longer wanted, repositions whatever already exists
## (cheap, and correct after a resize -- see `_update_lod`'s own native-space
## reasoning), and only calls `EngineBridge.lod_synthesize_tile` for a key
## that isn't in `_lod_tiles` yet.
func _apply_lod_tiles(wanted: Dictionary, detail_level: int, g: Vector2i, displayed_origin: Vector2, displayed_size: Vector2) -> void:
	for key in _lod_tiles.keys().duplicate():
		if not wanted.has(key):
			(_lod_tiles[key] as TextureRect).queue_free()
			_lod_tiles.erase(key)

	for key in wanted.keys():
		var idx: Vector2i = wanted[key]
		var origin_x := idx.x * _lod_tile_cells
		var origin_y := idx.y * _lod_tile_cells
		## Clipped exactly like `lod_bridge::tile_bounds` on the Rust side --
		## both compute `min(TILE_CELLS, remaining)` from the same `gw`/`gh`,
		## so this can never disagree with what a tile request actually
		## returns (this file's own module-doc note on why that's the one
		## piece of tile-bounds arithmetic duplicated here at all).
		var tile_w := mini(_lod_tile_cells, g.x - origin_x)
		var tile_h := mini(_lod_tile_cells, g.y - origin_y)
		if tile_w <= 0 or tile_h <= 0:
			continue
		var rect := Rect2(
			displayed_origin + Vector2(origin_x, origin_y) / Vector2(g.x, g.y) * displayed_size,
			Vector2(tile_w, tile_h) / Vector2(g.x, g.y) * displayed_size)

		if _lod_tiles.has(key):
			var existing := _lod_tiles[key] as TextureRect
			existing.position = rect.position
			existing.size = rect.size
			continue

		var tex := _bridge.lod_synthesize_tile(idx.x, idx.y, detail_level)
		if tex == null:
			continue
		var tile_node := TextureRect.new()
		tile_node.texture = tex
		tile_node.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		tile_node.stretch_mode = TextureRect.STRETCH_SCALE
		## `LINEAR`, not `_raster()`'s `NEAREST` -- the entire reason this
		## milestone exists is to stop showing blocky single-cell squares at
		## deep zoom, so the tile that replaces them must not reintroduce
		## the same artifact at its own, finer texel size.
		tile_node.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
		tile_node.mouse_filter = Control.MOUSE_FILTER_IGNORE
		tile_node.position = rect.position
		tile_node.size = rect.size
		_lod_layer.add_child(tile_node)
		_lod_tiles[key] = tile_node

## Frees every currently-displayed deep-zoom tile. Called on every world
## reset (`reset_view`, since a stale tile belongs to whatever world/size
## was live before) and whenever `_set_lod_active(false)` actually changes
## state (crossing back below the threshold).
func _clear_lod_tiles() -> void:
	for key in _lod_tiles.keys():
		(_lod_tiles[key] as TextureRect).queue_free()
	_lod_tiles.clear()

## Fades `_lod_layer` in or out and frees its tiles on the way out. A no-op
## when `active` already matches `_lod_active` -- called from several
## early-return paths in `_update_lod()` that would otherwise re-trigger the
## tween on every single call while already below the threshold.
func _set_lod_active(active: bool) -> void:
	if active == _lod_active:
		return
	_lod_active = active
	if not active:
		_clear_lod_tiles()
	var tw := create_tween()
	tw.tween_property(_lod_layer, "modulate:a", 1.0 if active else 0.0, 0.15)

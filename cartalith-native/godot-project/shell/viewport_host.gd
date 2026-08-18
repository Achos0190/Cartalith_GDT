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

const OVERLAY_SCRIPT := preload("res://map_overlay.gd")

var map_view: TextureRect
var territory_view: TextureRect
var province_view: TextureRect
var overlay: Control

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

	overlay = Control.new()
	overlay.set_script(OVERLAY_SCRIPT)
	overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	_camera.add_child(overlay)
	overlay.settlement_selected.connect(func(d, i): settlement_selected.emit(d, i))
	overlay.settlement_hovered.connect(_on_hovered)
	overlay.cursor_sampled.connect(_on_sampled)

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

## Back to fit, matching the letterboxed `STRETCH_KEEP_ASPECT_CENTERED` view
## every fresh generate/load already rendered before this camera existed --
## called from `refresh()` so a new world never opens scrolled off into
## whatever corner the previous one was zoomed into.
func reset_view() -> void:
	_zoom = 1.0
	_camera.scale = Vector2.ONE
	_camera.position = Vector2.ZERO
	_update_zoom_readout()

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
	var g := _bridge.grid_size()
	overlay.set_civ_data(_bridge.settlements(), _bridge.roads(),
		_bridge.sea_routes(), g.x, g.y, _bridge.border_inset_frac())
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

func set_layer_visible(layer: String, shown: bool) -> void:
	match layer:
		"territory": territory_view.visible = shown
		"provinces": province_view.visible = shown
		"settlements": overlay.set_show_settlements(shown)
		"roads": overlay.set_show_roads(shown)
		"sea_routes": overlay.set_show_sea_routes(shown)
		_: push_error("ViewportHost: unknown layer '%s'" % layer)

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

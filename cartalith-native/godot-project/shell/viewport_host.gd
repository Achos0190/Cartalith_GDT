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
var _bridge: EngineBridge
var _width_km := 0.0

func setup(bridge: EngineBridge) -> void:
	_bridge = bridge
	bridge.generation_finished.connect(func(ok: bool): if ok: refresh())
	bridge.world_loaded.connect(refresh)

func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_PASS

	map_view = _raster()
	territory_view = _raster()
	province_view = _raster()
	territory_view.visible = false
	province_view.visible = false
	add_child(map_view)
	add_child(territory_view)
	add_child(province_view)

	overlay = Control.new()
	overlay.set_script(OVERLAY_SCRIPT)
	overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(overlay)
	overlay.settlement_selected.connect(func(d, i): settlement_selected.emit(d, i))
	overlay.settlement_hovered.connect(_on_hovered)
	overlay.cursor_sampled.connect(_on_sampled)

	## §9's chrome, all corner-anchored so it survives any dock width.
	_scale_label = _chrome(Control.PRESET_BOTTOM_LEFT, HORIZONTAL_ALIGNMENT_LEFT)
	_readout_label = _chrome(Control.PRESET_TOP_RIGHT, HORIZONTAL_ALIGNMENT_RIGHT)
	_coords_label = _chrome(Control.PRESET_BOTTOM_RIGHT, HORIZONTAL_ALIGNMENT_RIGHT)

	var layers_btn := Button.new()
	layers_btn.flat = true
	layers_btn.focus_mode = Control.FOCUS_NONE
	layers_btn.icon = DccIcons.get_icon("layers", 15)
	layers_btn.tooltip_text = "Layers"
	layers_btn.modulate = DccTheme.c("text_dim")
	layers_btn.set_anchors_and_offsets_preset(Control.PRESET_TOP_LEFT)
	layers_btn.position = Vector2(10, 10)
	layers_btn.custom_minimum_size = Vector2(26, 26)
	layers_btn.add_theme_stylebox_override("normal",
		DccTheme.flat(DccTheme.c("panel"), 3))
	layers_btn.pressed.connect(func(): layers_button_pressed.emit())
	add_child(layers_btn)

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
	l.position += Vector2(-10.0 if right else 10.0, -10.0 if bottom else 10.0)
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
	_readout_label.text = "%d x %d  ·  %.0f x %.0f km" % [
		g.x, g.y, _bridge.last_width_km, _bridge.last_height_km]

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

extends ColorRect
class_name WaterAnimLayer

## Animated water (`GUI_GAP_REGISTER.md` RN-01) -- the reference's own
## `state.viz.waterAnim` flow-map shimmer over river channels (`waterAnimFrame`,
## reference HTML lines 8667-8690), which this port had not carried across.
##
## ## Why this is a shader and the Painter styles are not
##
## Everything else in the reference's NPR block is a pure function of one
## finished pixel, so it ports literally into `render.rs`'s `apply_npr` and
## rides the raster pass that already exists. This one is a function of one
## pixel *and the clock*: the reference rebuilds a `GW x GH` `ImageData` and
## `putImageData`s it on every animation frame, and caps itself at
## `GW*GH <= 400000` (line 8670) precisely because that does not scale. Baking
## a per-frame effect into `build_color_texture()` would mean re-running the
## whole appearance pipeline sixty times a second for a shimmer.
##
## So the model is ported and the technique is not -- `DECISIONS.md` §7a's
## principled-equivalence carve-out, and the same call `wind_fx_layer.gd`
## already made for the streak trails. The constants (`SCALE`, `F`, the 2.4 s
## two-stream crossfade, the `rip - 0.5` threshold, the 240/150 alpha scale
## and cap, the colour) are the reference's; the noise hash and the fact that
## it runs in a fragment shader are not. See `water_anim.gdshader`.
##
## **Dropping the reference's own resolution cap is deliberate**: the cap
## exists to protect a JavaScript pixel loop, and there is no JavaScript pixel
## loop here. A full 2048x1311 world animates.
##
## ## Nothing runs while it is off
##
## `visible` false means Godot never draws the rect and never runs the shader;
## `_process` is disabled with it, so the node costs one idle object. The flow
## raster is fetched on the first enable and dropped on a regenerate or load,
## the same "derive when picked, keep nothing after" rule `sample_bridge.rs`'s
## own rasters and `wind_fx_layer.gd` both follow.

const SHADER := preload("res://shell/water_anim.gdshader")

var _bridge: EngineBridge
var _time := 0.0
var _grid := Vector2i.ZERO
var _refused := false    ## This world could not answer the field; retried only after a regenerate.

func setup(b: EngineBridge) -> void:
	_bridge = b
	name = "WaterAnimLayer"
	color = Color(1, 1, 1, 1)      ## Unused: the shader writes COLOR outright.
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	visible = false
	set_process(false)
	var mat := ShaderMaterial.new()
	mat.shader = SHADER
	material = mat
	## A regenerate or a load invalidates the channel field this holds (and can
	## change the grid it lives on), so drop it and let the next enable refetch.
	b.generation_finished.connect(func(_ok: bool): _drop())
	b.world_loaded.connect(_drop)

func _drop() -> void:
	_grid = Vector2i.ZERO
	_refused = false
	(material as ShaderMaterial).set_shader_parameter("flow_tex", null)

## The one entry point. `false` from `set_enabled(true)` means this world
## cannot answer the field -- the caller reports that rather than leaving a
## checkbox ticked over an effect that is not running.
func set_enabled(on: bool) -> bool:
	if not on:
		visible = false
		set_process(false)
		return true
	if _grid == Vector2i.ZERO:
		if _refused or not _fetch():
			_refused = true
			visible = false
			set_process(false)
			return false
	visible = true
	set_process(true)
	return true

func is_enabled() -> bool:
	return visible

func _fetch() -> bool:
	if _bridge == null:
		return false
	var tex: Texture2D = _bridge.debug_texture("waterfx")
	if tex == null or tex.get_width() <= 0:
		return false
	_grid = Vector2i(tex.get_width(), tex.get_height())
	var mat := material as ShaderMaterial
	mat.set_shader_parameter("flow_tex", tex)
	mat.set_shader_parameter("grid_size", Vector2(_grid))
	return true

## Track the map's own letterbox rect rather than the whole viewport, exactly
## as `wind_fx_layer.gd` does and for the same reason: the parent
## (`map_overlay.gd`) already publishes the fit, so pan and zoom need no code
## here at all.
func _process(delta: float) -> void:
	_time += delta
	(material as ShaderMaterial).set_shader_parameter("anim_time", _time)
	var parent := get_parent()
	if parent == null or not parent.has_method("displayed_rect"):
		return
	var rect: Rect2 = parent.displayed_rect()
	if rect.size.x <= 0.0 or rect.size.y <= 0.0:
		visible = false
		return
	visible = true
	position = rect.position
	size = rect.size

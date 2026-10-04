extends RefCounted

## Ruling BR (`LARGE_ITEM_RULINGS.md`, 2026-09-28): while a paint tool is armed
## on a raster layer, show that layer's ORIGINAL -- as the generator made it,
## before any hand paint -- over the map, at an opacity the painter sets. A
## comparison aid for "what did I change", nothing more.
##
## **What "original" is, and is not.** The paint layers (`biome`, `terrain`) are
## override arrays that `build_color_texture` blends over the generated field at
## `landColorCore`'s 0.60 weight. The engine already exposes the UNPAINTED
## classification as two debug views: `bclass` (the 15-class biome grid) and
## `cterrain` (the 13-class terrain grid), both built from the world's own
## height/climate/water and neither reading the override arrays
## (`sample_bridge.rs`). So the original needs no Rust: this asks
## `EngineBridge.debug_texture()` for the view and hands it to
## `ViewportHost.set_paint_original()`. Two limits, both disclosed rather than
## hidden:
##   - It is re-derived from the CURRENT height/climate, so a committed Sculpt
##     edit moves it. A snapshot of the pre-sculpt world is not retained
##     anywhere; keeping one would need Rust (OUTSTANDING_WORK.md row, Ruling
##     BR). Hand paint is the only thing it excludes.
##   - `splat` has no generated raster at all (it forces a pack ground texture),
##     so that layer shows a one-line reason instead of a slider.
##
## **What this must never do.** Write world data, the project save or an
## export; call anything on the bridge but the read-only `debug_layers()` /
## `debug_texture()`; keep a texture alive while no paint tool is armed (it is
## a grid-sized RGBA, freed on disarm); or draw anything per frame (the layer
## is a static `TextureRect`, hidden when off, so an idle frame is unchanged).
##
## **State** is shared by the desktop Paint tool bar and the phone GENERATE sheet's
## Paint column (both write it only through `set_percent()`), and is session-only,
## like `world_workspace.gd`'s `_paint_brush` which it sits beside: `opacity` starts at 0 (off), survives
## layer switches and disarm/re-arm, and is gone at restart. Nothing is written
## to `user://cartalith_settings.cfg`.
##
## Not a `class_name` (the shell already registers too many; project rule).
## `app.gd` preloads it, so no other file needs to know its path.

## The two layers that have a generated raster, and the engine debug view each
## one's original is. Source: `sample_bridge.rs` `LAYER_GROUPS` ("bclass" is
## `buildCartBiome()`'s grid, "cterrain" is `buildCartTerrain()`'s) -- the same
## grids `paint_bridge.rs` overrides, so the two line up cell for cell.
const ORIGINAL_VIEW := {"biome": "bclass", "terrain": "cterrain"}

## Slider range and step, in percent. 0 = off; 5 % steps are fine enough to
## read an overlay's strength by eye and coarse enough for a thumb.
const PCT_MAX := 100.0
const PCT_STEP := 5.0

## The slider's tooltip: what it shows, and that it is a viewing aid only.
const TIP := "Show this layer as the generator made it, under your paint, so you can compare. " \
	+ "A viewing aid: it never changes the world, a save or an export."

var app: DccApp
var bridge: EngineBridge

## Slider value as a 0..1 fraction. 0 means off. Session-only (see header).
var opacity := 0.0

var _tex: Texture2D = null
var _tex_layer := ""
## True when `_tex` may be stale. Set by every `color_texture_rebuilt` (a new
## world, a Sculpt/Paint commit), cleared when the texture is rebuilt.
var _dirty := true

## Wire to the shell: re-evaluate on every tool arm and on every map rebuild.
## Called once from `DccApp._ready()` after `bridge` exists.
func setup(a: DccApp, b: EngineBridge) -> void:
	app = a
	bridge = b
	app.tool_armed.connect(func(_id: String): sync())
	if bridge.has_signal("color_texture_rebuilt"):
		bridge.color_texture_rebuilt.connect(_on_map_rebuilt)

## A rebuild of the map (new world, commit) can change the unpainted field too
## (Sculpt moves height), so the cached texture is no longer trusted. Rebuilt
## lazily by `sync()` -- and only when something is actually showing it.
func _on_map_rebuilt() -> void:
	_dirty = true
	if _showing():
		sync()

## The armed paint layer's key (`biome`/`terrain`/`splat`), read from the same
## place `tool_bar.gd::_paint_state()` reads it. `""` when the WORLD workspace
## is not registered yet -- "no value" is not encoded as `biome`.
func _layer() -> String:
	var ws = app._workspace_panels.get("world")
	if ws != null and "_paint_layer" in ws:
		return String(ws._paint_layer)
	return ""

## Whether the original layer is currently on screen.
func _showing() -> bool:
	return app.viewport != null and bool(app.viewport.paint_original_state().get("visible", false))

## The engine's reason this layer has no usable original, or `""` when it has
## one. Used for the bar's hint; `bridge.debug_layers()` is the engine's
## own availability answer (`sample_bridge.rs::layer_available`).
func unavailable_reason(layer: String) -> String:
	if layer == "":
		return "no paint layer"
	var view := String(ORIGINAL_VIEW.get(layer, ""))
	if view == "":
		return "%s has no generated raster" % layer.capitalize()
	if not bridge.has_world:
		return "no world yet"
	for g in bridge.debug_layers():
		for item in (g as Dictionary).get("items", []):
			var d: Dictionary = item
			if String(d.get("id", "")) == view:
				return "" if bool(d.get("available", true)) else "the engine cannot answer %s for this world" % view
	return "the engine has no %s view" % view

## Bring the on-screen layer in line with the current state: shown iff a paint
## tool is armed, the slider is above 0 and the layer has an original. Every
## other path frees the texture, so nothing is held while the aid is off. Cheap
## when nothing changed (one dictionary read and a visibility set).
func sync() -> void:
	if app == null or app.viewport == null:
		return
	var layer := _layer()
	var armed: bool = app.armed_tool == "paint"
	if not armed or opacity <= 0.0 or unavailable_reason(layer) != "":
		_release()
		return
	if _tex == null or _tex_layer != layer or _dirty:
		_tex = bridge.debug_texture(String(ORIGINAL_VIEW[layer]))
		_tex_layer = layer
		_dirty = false
	## A null texture (the engine declined) hides the layer rather than showing
	## a stale one; `set_paint_original` treats null as off.
	app.viewport.set_paint_original(_tex, opacity)

## Set the strength from a percent value and re-evaluate the layer. The ONE
## writer of `opacity`: the tool bar's "Original" slider (`decorate()` below)
## and the phone GENERATE sheet's Paint column (`world_workspace.gd::_pg_paint`)
## both call it, so a thumb on the phone and a mouse on the bar move the same
## state and neither can drift from the other. `percent` is on the slider's own
## 0..`PCT_MAX` scale (0 = off); out-of-range input is clamped, never wrapped.
## Must never write anything but `opacity` and the on-screen layer; it is as
## cheap as `sync()` (one dictionary read, plus one texture build the first
## time a layer is shown).
func set_percent(percent: float) -> void:
	opacity = clampf(percent / PCT_MAX, 0.0, 1.0)
	sync()

## Hide the layer and drop the texture.
func _release() -> void:
	_tex = null
	_tex_layer = ""
	_dirty = true
	if app.viewport != null:
		app.viewport.set_paint_original(null, 0.0)

## Append the Original control to the Paint tool bar. `build` is
## the callable `DccApp.set_tool_options()` was given; this only acts when it
## is the `DccToolBar` in Paint mode, so another workspace's options row (or a
## Sculpt/Measure bar) is never touched. Called inside the row's own build, i.e.
## BEFORE `set_tool_options()` runs `phone_fit()`/`tablet_fit()`, so the control
## is sized to the touch floor by the same pass as every sibling.
##
## Layout: it goes on the bar's FIRST row, after the Erase button and before the
## row's right-hand note, beside the layer picker it qualifies. Not on the
## options row: measured at 1600x1000, that row's minimum width is 930 px and
## adding the slider made it 1144 (a bar that fit a ~960 px window then clips),
## while the tools row is 408 px and takes it with room to spare. The slider
## uses the bar's own narrowed label/track/readout widths (`DccToolBar.BAR_*`),
## so it is the same shape as the Size slider. A layer with no original (Splat,
## or an engine that cannot answer) gets a dim one-line reason instead of a dead
## slider.
func decorate(row: HBoxContainer, build: Callable) -> void:
	var obj := build.get_object()
	if not (obj is DccToolBar) or (obj as DccToolBar).mode != "paint":
		return
	if row.get_child_count() == 0:
		return
	var col := row.get_child(0) as VBoxContainer
	if col == null or col.get_child_count() == 0:
		return
	var tools := col.get_child(0) as HBoxContainer
	var options := col.get_child(col.get_child_count() - 1) as HBoxContainer
	## `_build_paint_options` returns early with an empty row when there is no
	## world or no palette; there is nothing to compare against then.
	if tools == null or options == null or options.get_child_count() == 0:
		return
	var why := unavailable_reason(_layer())
	if why != "":
		var hint := DccTheme.mono_label("no original · %s" % why, "text_ghost", DccTheme.FS_SMALL)
		hint.tooltip_text = TIP
		## Expands and clips, like `tool_bar.gd::_note`: a zero-minimum label
		## that does not expand is given zero width by the row, which is what
		## the first version of this did (the reason was in the tree and
		## invisible on screen). Sharing the slack with the row's spacer, it is
		## readable at any ordinary width and clips -- tooltip intact -- below.
		hint.clip_text = true
		hint.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		tools.add_child(hint)
		tools.move_child(hint, _spacer_index(tools))
		return
	var parts := DccWidgets.slider(tools, "Original", 0.0, PCT_MAX, PCT_STEP,
		opacity * 100.0, "%", func(v: float): set_percent(v), TIP)
	var r: Control = parts["row"]
	tools.move_child(r, _spacer_index(tools))
	var label := r.get_child(0) as Control
	label.custom_minimum_size.x = DccToolBar.BAR_LABEL_W
	if label is Label:
		(label as Label).add_theme_font_override("font", DccTheme.mono(0))
		(label as Label).add_theme_color_override("font_color", DccTheme.c("text_dim"))
	(parts["readout"] as Control).custom_minimum_size.x = DccToolBar.BAR_VALUE_W
	(parts["slider"] as Control).custom_minimum_size.x = DccToolBar.BAR_CONTROL_W

## Where to slot the control: just before the row's right-hand spacer, so it
## sits with the brush controls (Size, Land only) and the "N painted" readout
## and Commit/Discard chips stay hard right. The spacer is the one plain
## `Control` (not a container, not a label) that expands horizontally --
## `DccTheme.spacer()`. Falls back to the end of the row if it is not found.
func _spacer_index(row: HBoxContainer) -> int:
	for i in row.get_child_count():
		var c := row.get_child(i)
		if c.get_class() == "Control" and ((c as Control).size_flags_horizontal & Control.SIZE_EXPAND) != 0:
			return i
	return row.get_child_count() - 1

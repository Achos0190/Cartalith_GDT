extends RefCounted
## LOD-D7 (Ruling AV) -- deep-zoom tiles for the Layers popover's info views.
##
## WHAT IT IS. A small manager, owned by `viewport_host.gd`, that draws a
## per-tile raster of the CURRENT info view (temperature, rainfall, flow, crust
## age, rock resistance, elevation, slope, aspect) over the deep-zoom terrain
## tiles, in place of the map-resolution `_debug_layer` raster that goes blocky
## once a map cell is many screen pixels wide. Tiles are made by
## `info_tile.rs` on the SAME worker as the terrain tiles (same snapshot, same
## generation counter, same in-flight cap), so nothing here synthesises on the
## main thread and nothing queues beyond the tier's budget.
##
## WHAT IT MUST NEVER DO.
##  - Show one view's tile under another view. Every tile arrives with
##    `{view, tag}`; a tile whose view is not the live view, or whose tag is not
##    the snapshot's CURRENT tag for that view, is dropped, never installed. The
##    parked-tile cache is keyed on the tag (which contains the view id), so a
##    switch cannot restore a different view's pixels either.
##  - Hold more than the tier's budget. Parked tiles are capped at
##    `cache_tiles / PARK_DIVISOR`; live tiles are the wanted set plus, at most
##    one level change's worth of retained coverage.
##  - Touch the terrain tiles, `_lod_tiles` or the base raster. This layer is a
##    separate child of the camera, above `_debug_layer`.
##
## THE DEBUG RASTER STAYS UNDERNEATH as the fallback, and is hidden only once
## this layer has covered the whole wanted set at least once (`engaged`), so the
## half-opaque raster never ghosts through a tile the way two stacked
## translucent layers would. Until then it is what the viewer sees, which is
## the pre-D7 behaviour.
##
## Not a `class_name`: `viewport_host.gd` preloads this file, so no global class
## is registered (and `project.godot` is not touched).

## Parked tiles are capped at the tier's `cache_tiles` divided by this. A
## labelled judgement: info views are a secondary overlay, and the terrain cache
## already holds `cache_tiles` tiles of the same 256 KiB each, so an overlay
## that may park as many again would double the pyramid's tile memory. Half
## keeps the worst case at 1.5x.
const PARK_DIVISOR := 2

var _camera: Node
var _bridge
var _rect_fn: Callable          ## host `_lod_tile_rect(idx, tex, g, origin, size)`
var _layer: Control
var _view := ""                 ## the view being drawn; "" = none
var _tag := ""                  ## the snapshot's tag for `_view`, re-read each update
var _level := -1                ## the level currently wanted
var _engaged := false           ## the wanted set has been fully covered once
var _tiles: Dictionary = {}     ## "z,col,row" -> Sprite2D (live)
var _pending: Dictionary = {}   ## "z,col,row" -> Vector3i (asked of the worker)
var _backlog: Dictionary = {}   ## "z,col,row" -> Vector3i (refused or over budget; retried)
var _wanted: Dictionary = {}    ## "z,col,row" -> Vector3i
var _cache: Dictionary = {}     ## "tag|z,col,row" -> Sprite2D (parked, insertion = LRU order)
var _cache_cap := 0
var _geom_grid := Vector2i.ZERO
var _geom_origin := Vector2.ZERO
var _geom_size := Vector2.ZERO
var _opacity := 1.0
var _installed_total := 0
var _dropped_stale := 0
var _bytes_live := 0

## Wire up the manager. Call once after the camera exists.
##
## `above` is the node the layer is inserted after (the debug raster), so the
## sibling order is terrain tiles, debug raster, info tiles, overlay.
func setup(camera: Node, above: Node, bridge, rect_fn: Callable) -> void:
	_camera = camera
	_bridge = bridge
	_rect_fn = rect_fn
	_layer = Control.new()
	_layer.name = "InfoTileLayer"
	_layer.set_anchors_preset(Control.PRESET_FULL_RECT)
	_layer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_layer.visible = false
	camera.add_child(_layer)
	camera.move_child(_layer, above.get_index() + 1)

## Late-bind the engine bridge (the host's `setup()` can run after `_ready()`).
func bind(bridge) -> void:
	_bridge = bridge

## The layer node, for probes that read the pixels it draws.
func layer() -> Control:
	return _layer

## Whether the loaded extension can draw info tiles at all.
func available() -> bool:
	return _bridge != null and _bridge.info_tiles_available()

## Which view is being drawn ("" when none).
func view() -> String:
	return _view

## True once the whole wanted set has been covered at least once for the live
## view -- the moment the host hides its map-resolution raster.
func engaged() -> bool:
	return _engaged

## Match the debug raster's opacity slider.
func set_opacity(a: float) -> void:
	_opacity = clampf(a, 0.0, 1.0)
	if _layer != null:
		_layer.modulate.a = _opacity

## True while a request is out or waiting to be asked again: the host keeps its
## `_process` running for exactly this long, then lets it idle.
func busy() -> bool:
	return not _pending.is_empty() or not _backlog.is_empty()

## Forget every live and pending tile (a view switch, a world change, the
## pyramid coming down). Parked tiles are kept ONLY across a view switch, and
## `drop_cache()` frees them. The pending set is forgotten, not cancelled --
## nothing can cancel a running job -- and its late arrivals are dropped by the
## view and tag check in `process_frame()`.
func clear() -> void:
	for key in _tiles.keys():
		_free_tile(_tiles[key])
	_tiles.clear()
	_pending.clear()
	_backlog.clear()
	_wanted.clear()
	_view = ""
	_tag = ""
	_level = -1
	_engaged = false
	_bytes_live = 0
	if _layer != null:
		_layer.visible = false

## Free every parked tile. A world change or an appearance change: the parked
## tiles belong to a snapshot that no longer exists.
func drop_cache() -> void:
	for key in _cache.keys():
		_free_tile(_cache[key])
	_cache.clear()

## Point the layer at `view` and at this call's wanted set. `wanted` is
## `"z,col,row" -> Vector3i` for the DRAWN level only (no parent level: the
## fallback under a missing tile is the debug raster, not another level).
##
## A view whose tag is empty (not tileable, no snapshot yet, an input missing)
## draws nothing and reports `false`, so the host keeps the raster. A changed
## view or tag clears everything live first; that is the whole of "a view switch
## never shows another view's tile" on the live side.
func update(view_id: String, wanted: Dictionary, level: int, g: Vector2i, origin: Vector2, disp_size: Vector2, budget: Dictionary) -> bool:
	if not available() or view_id == "":
		clear()
		return false
	var tag: String = _bridge.info_tile_tag(view_id)
	if tag == "":
		clear()
		return false
	_cache_cap = maxi(0, int(floor(float(budget.get("cache_tiles", 0)) / float(PARK_DIVISOR))))
	if view_id != _view or tag != _tag:
		clear()
		_view = view_id
		_tag = tag
	_geom_grid = g
	_geom_origin = origin
	_geom_size = disp_size
	_level = level
	_wanted = wanted
	_layer.visible = true
	_layer.modulate.a = _opacity
	# Re-place what is live (a resize or a letterbox change moves the fit rect).
	for key in _tiles.keys():
		var s: Sprite2D = _tiles[key]
		_place(s, s.get_meta("info_idx"))
	# Free what is neither wanted nor still needed as coverage.
	_retire_unneeded()
	# Ask for what is missing, nearest first is the host's job via `wanted` order.
	var asked := 0
	var per_call := maxi(1, int(budget.get("tiles_per_update", 8)))
	for key in wanted.keys():
		if _tiles.has(key) or _pending.has(key):
			continue
		var idx: Vector3i = wanted[key]
		if _restore(key, idx):
			continue
		if asked >= per_call:
			_backlog[key] = idx
			continue
		_ask(key, idx)
		asked += 1
	_update_engaged()
	return true

## One frame of work: collect finished tiles, then retry the backlog. Returns
## whether anything was installed (the host re-runs its blend pass if so).
func process_frame(budget: Dictionary) -> bool:
	if _view == "":
		return false
	var installed := false
	var per_frame := maxi(1, int(budget.get("tiles_per_catchup", 4)))
	if not _pending.is_empty():
		for entry in _bridge.info_take_ready_tiles(per_frame):
			var d: Dictionary = entry
			var idx := Vector3i(int(d.get("z", -1)), int(d.get("col", 0)), int(d.get("row", 0)))
			var key := "%d,%d,%d" % [idx.x, idx.y, idx.z]
			# THE VIEW AND TAG CHECK. A tile for another view (asked before a
			# switch and landing after it) or for an older snapshot is dropped.
			if str(d.get("view", "")) != _view or str(d.get("tag", "")) != _tag:
				_dropped_stale += 1
				continue
			_pending.erase(key)
			if _tiles.has(key) or not _wanted.has(key):
				continue
			_install(key, idx, d.get("tex", null) as Texture2D)
			installed = true
		_reconcile_pending()
	var n := 0
	for key in _backlog.keys().duplicate():
		if n >= per_frame:
			break
		var idx: Vector3i = _backlog[key]
		_backlog.erase(key)
		if _tiles.has(key) or _pending.has(key) or not _wanted.has(key):
			continue
		if not _restore(key, idx):
			_ask(key, idx)
		n += 1
	if installed:
		_retire_unneeded()
		_update_engaged()
	return installed

## Counters for the probe and the diagnostics: tile counts, the memory they
## hold (RGBA8, so 4 bytes a texel) and the stale-arrival count.
func stats() -> Dictionary:
	var parked_bytes := 0
	for key in _cache.keys():
		parked_bytes += _sprite_bytes(_cache[key])
	return {
		"view": _view, "tag": _tag, "level": _level, "engaged": _engaged,
		"live": _tiles.size(), "parked": _cache.size(),
		"pending": _pending.size(), "backlog": _backlog.size(),
		"live_bytes": _bytes_live, "parked_bytes": parked_bytes,
		"installed_total": _installed_total, "dropped_stale": _dropped_stale,
	}

## The live tile at `key` (`"z,col,row"`), or null. For probes.
func tile_at(key: String) -> Sprite2D:
	return _tiles.get(key, null)

# -- internals ---------------------------------------------------------------

func _ask(key: String, idx: Vector3i) -> void:
	if _bridge.info_request_tile(_view, idx.x, idx.y, idx.z):
		_pending[key] = idx
	else:
		# Refused: the shared cap is full or the snapshot is still building.
		# Not an error, not a drop -- asked again next frame.
		_backlog[key] = idx

func _place(s: Sprite2D, idx: Vector3i) -> void:
	var rect = _rect_fn.call(idx, s.texture, _geom_grid, _geom_origin, _geom_size)
	if rect == null:
		return
	var ts := s.texture.get_size()
	s.position = rect.position
	s.scale = rect.size / Vector2(maxf(ts.x, 1.0), maxf(ts.y, 1.0))

func _install(key: String, idx: Vector3i, tex: Texture2D) -> void:
	if tex == null or tex.get_width() < 2 or tex.get_height() < 2:
		return
	var s := Sprite2D.new()
	s.texture = tex
	s.centered = false
	# LINEAR for the same reason as the terrain tiles: the point of the layer is
	# to stop showing single-cell squares.
	s.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
	s.set_meta("info_idx", idx)
	s.set_meta("info_tag", _tag)
	_place(s, idx)
	_layer.add_child(s)
	# Coarser levels sit behind finer ones while a level change is retained.
	if idx.x < _level:
		_layer.move_child(s, 0)
	_tiles[key] = s
	_bytes_live += _sprite_bytes(s)
	_installed_total += 1

func _sprite_bytes(s: Sprite2D) -> int:
	return s.texture.get_width() * s.texture.get_height() * 4 if s.texture != null else 0

## Take a parked tile back if its TAG matches the live one (it contains the
## view id, so another view's tile can never match).
func _restore(key: String, idx: Vector3i) -> bool:
	var ck := _tag + "|" + key
	if not _cache.has(ck):
		return false
	var s: Sprite2D = _cache[ck]
	_cache.erase(ck)
	if str(s.get_meta("info_tag", "")) != _tag:
		_free_tile(s)
		return false
	_layer.add_child(s)
	_tiles[key] = s
	_place(s, idx)
	if idx.x < _level:
		_layer.move_child(s, 0)
	_bytes_live += _sprite_bytes(s)
	return true

## Move a live tile into the parked cache (or free it when the cache is off),
## evicting the oldest past the cap.
func _park(key: String, s: Sprite2D) -> void:
	_bytes_live -= _sprite_bytes(s)
	_layer.remove_child(s)
	if _cache_cap <= 0:
		s.queue_free()
		return
	_cache[_tag + "|" + key] = s
	while _cache.size() > _cache_cap:
		var oldest: String = _cache.keys()[0]
		_free_tile(_cache[oldest])
		_cache.erase(oldest)

func _free_tile(s: Sprite2D) -> void:
	if s.get_parent() != null:
		s.get_parent().remove_child(s)
	s.queue_free()

## Free the live tiles that are not wanted -- except coverage kept on purpose.
## While the drawn level is not fully covered, tiles of OTHER levels stay so a
## level change never opens a hole; once it is covered they go.
func _retire_unneeded() -> void:
	var covered := true
	for key in _wanted.keys():
		if not _tiles.has(key):
			covered = false
			break
	for key in _tiles.keys().duplicate():
		if _wanted.has(key):
			continue
		var idx: Vector3i = (_tiles[key] as Sprite2D).get_meta("info_idx")
		if idx.x != _level and not covered:
			continue
		var s: Sprite2D = _tiles[key]
		_tiles.erase(key)
		_park(key, s)

func _update_engaged() -> void:
	if _engaged or _wanted.is_empty():
		return
	for key in _wanted.keys():
		if not _tiles.has(key):
			return
	_engaged = true

## The engine's own book-keeping: nothing in flight and nothing waiting means
## every request this side holds has been answered one way or another (a
## refused job is counted as dropped and never handed back).
func _reconcile_pending() -> void:
	if _pending.is_empty():
		return
	var st: Dictionary = _bridge.info_worker_stats()
	if st.is_empty():
		return
	if int(st.get("in_flight", 1)) == 0 and int(st.get("waiting", 1)) == 0:
		_pending.clear()

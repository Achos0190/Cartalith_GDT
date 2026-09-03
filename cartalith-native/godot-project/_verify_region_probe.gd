extends SceneTree

# Adversarial verification of Lane B: WorldGen::region_new_world.
# Drives the REAL cdylib. Nothing here is a stub.

var g: RefCounted
var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _dims() -> Vector2i:
	var tex: ImageTexture = g.build_color_texture()
	if tex == null:
		return Vector2i(-1, -1)
	return Vector2i(tex.get_width(), tex.get_height())

func _initialize() -> void:
	g = ClassDB.instantiate("WorldGen")
	if g == null:
		print("FATAL: WorldGen not registered"); quit(1); return

	print("== 0. refusals BEFORE any world exists ==")
	_ok(g.region_new_world(128, false, 0.0, 0.0) == false, "refuses with no world")
	print("      reason: %s" % g.region_new_world_error())
	_ok(String(g.region_new_world_error()) != "", "refusal reason is non-empty")

	print("\n== 1. parent world ==")
	g.generate_sized(4242, 400.0, 256, 160)
	var d0 := _dims()
	var w0: float = g.get_map_width_km()
	var seed0: int = g.get_seed()
	var set0: Array = g.get_settlements()
	print("  parent: dims=%s  map_width_km=%.4f  seed=%d  settlements=%d" % [d0, w0, seed0, set0.size()])
	_ok(d0 == Vector2i(256, 160), "parent grid is 256x160")

	# refusal 2: world exists, no marquee
	_ok(g.region_new_world(128, false, 0.0, 0.0) == false, "refuses with world but no marquee")
	print("      reason: %s" % g.region_new_world_error())

	print("\n== 2. dirty the world with everything a replacement must clear ==")
	var ic: int = g.icon_place(40.0, 40.0)
	var lb: int = g.label_create(50.0, 50.0, "PARENT LABEL")
	print("  icon_place -> %d, label_create -> %d" % [ic, lb])
	var icons0: Array = g.icon_list()
	var labels0: Array = g.label_list()
	_ok(icons0.size() > 0, "parent has %d icon(s)" % icons0.size())
	_ok(labels0.size() > 0, "parent has %d label(s)" % labels0.size())
	var undo0: bool = g.can_undo()
	print("  can_undo before = %s" % undo0)
	g.region_set(64.0, 40.0, 128.0, 80.0)
	var reg0: Dictionary = g.region_get()
	print("  region_get = %s" % reg0)
	_ok(not reg0.is_empty(), "marquee set")

	print("\n== 3. region_new_world(128) ==")
	var ok: bool = g.region_new_world(128, false, 0.0, 0.0)
	print("  returned %s, error=\"%s\"" % [ok, g.region_new_world_error()])
	_ok(ok, "region_new_world returned true")
	_ok(String(g.region_new_world_error()) == "", "error string cleared on success")

	var d1 := _dims()
	var w1: float = g.get_map_width_km()
	var seed1: int = g.get_seed()
	print("  child : dims=%s  map_width_km=%.4f  seed=%d" % [d1, w1, seed1])

	print("\n== 4. CARRY-OVER HUNT ==")
	_ok(d1 != d0, "grid dims changed (%s -> %s)" % [d0, d1])
	_ok(d1.x == 128, "long edge is tile_size 128 (got %d)" % d1.x)
	# reference: newMapWidthKm = max(1, mapWidthKm * sel.w / GW_old_new)... the
	# port's own rule: map_width_km * sel.w / new_gw
	var expect_w: float = maxf(1.0, w0 * 128.0 / float(d1.x))
	print("  expected map_width_km = %.4f (w0=%.4f * sel.w=128 / gw=%d)" % [expect_w, w0, d1.x])
	_ok(absf(w1 - expect_w) < 1e-9, "map_width_km rescaled to the selection's share")
	_ok(seed1 == seed0, "seed inherited (%d)" % seed1)

	var reg1: Dictionary = g.region_get()
	_ok(reg1.is_empty(), "MARQUEE cleared (region_get = %s)" % reg1)
	var icons1: Array = g.icon_list()
	_ok(icons1.is_empty(), "ICONS cleared (%d left)" % icons1.size())
	var labels1: Array = g.label_list()
	_ok(labels1.is_empty(), "LABELS cleared (%d left)" % labels1.size())
	_ok(g.can_undo() == false, "UNDO cleared")
	_ok(g.is_finalized() == false, "bake.finalized cleared")
	var set1: Array = g.get_settlements()
	print("  child settlements = %d (parent had %d)" % [set1.size(), set0.size()])
	_ok(set1.size() > 0, "CIV LAYER recomputed over the new terrain, not empty")

	# an icon placed at a coordinate only valid in the parent grid must not
	# have survived into a smaller child grid
	_ok(icons1.is_empty(), "no parent-grid icon coordinate leaked into the child")

	print("\n== 5. the sub-4 refusal floor (extreme aspect) ==")
	g.generate_sized(7, 400.0, 256, 160)
	g.region_set(0.0, 0.0, 256.0, 2.0)   # 128:1 aspect
	var ok2: bool = g.region_new_world(256, false, 0.0, 0.0)
	print("  returned %s, error=\"%s\"" % [ok2, g.region_new_world_error()])
	_ok(ok2 == false, "refuses a sub-4-cell short axis")
	_ok(String(g.region_new_world_error()).contains("4"), "reason names the 4-cell floor")
	# and the parent must be untouched by the refusal
	_ok(_dims() == Vector2i(256, 160), "parent grid untouched by the refusal")
	_ok(not g.region_get().is_empty(), "marquee still set after the refusal")

	print("\n== 6. tile_size <= 0 defaults to 1024 (reference refSize) ==")
	g.generate_sized(9, 400.0, 128, 80)
	g.region_set(0.0, 0.0, 128.0, 80.0)
	var ok3: bool = g.region_new_world(0, false, 0.0, 0.0)
	print("  returned %s, dims=%s" % [ok3, _dims()])
	_ok(ok3, "tile_size 0 accepted")
	_ok(_dims().x == 1024, "defaulted to 1024, not 512 (got %d)" % _dims().x)

	print("\n%s (%d failure%s)" % ["ALL PASS" if fails == 0 else "FAILURES", fails, "" if fails == 1 else "s"])
	quit(1 if fails > 0 else 0)

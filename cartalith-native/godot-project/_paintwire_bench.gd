extends Node

## What one paint dab costs the SHELL, before and after the bounded upload --
## `world_workspace.gd::_paint_show_preview()`'s two branches, timed through a
## real `EngineBridge` and a real `ViewportHost` rather than as engine calls in
## isolation.
##
## **Runs WINDOWED on purpose and refuses otherwise.** Under `--headless` the
## dummy driver makes `ImageTexture.update()` a no-op and `get_image()` a
## dictionary lookup, so the two costs this compares both collapse to nothing
## and the answer would be a fiction.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _paintwire_bench.tscn

const SIZES := [512, 1024, 2048]
const DABS := 20
const RADIUS := 40.0    ## The brush ceiling: the largest footprint a dab can make.

var bridge: EngineBridge
var host: ViewportHost
var full_hits := 0


func _ready() -> void:
	var probe := Image.create_empty(2, 2, false, Image.FORMAT_RGBA8)
	probe.fill(Color(0, 0, 0, 0))
	var t := ImageTexture.create_from_image(probe)
	probe.set_pixel(1, 1, Color(1, 0, 0, 1))
	t.update(probe)
	if t.get_image().get_pixel(1, 1).a == 0.0:
		print("BENCH REFUSED: no real renderer (%s). Re-run without --headless."
			% DisplayServer.get_name())
		get_tree().quit(2)
		return

	bridge = EngineBridge.new()
	add_child(bridge)
	host = ViewportHost.new()
	add_child(host)
	host.setup(bridge)
	await get_tree().process_frame
	for g in SIZES:
		await _bench(int(g))
	get_tree().quit(0)


func _bench(g: int) -> void:
	bridge.world_gen.generate_sized(24601, 640.0, g, g)
	bridge.world_gen.paint_set_layer("biome")
	bridge.world_gen.paint_set_brush(3, RADIUS, 1.0, 0.0, false, false)

	var before := await _pass(g, false)
	var after := await _pass(g, true)
	var d: Dictionary = bridge.build_paint_preview_patch()
	print("%d^2  window %dx%d = %.2f%% of the grid  (%d of %d dabs fell back to a full raster)" % [
		g, int(d["w"]), int(d["h"]), 100.0 * float(int(d["w"]) * int(d["h"])) / float(g * g),
		full_hits, DABS])
	print("    full raster (before)  ", _line(before))
	print("    bounded patch (after) ", _line(after))


## One 20-dab drag. `patched == false` is the shell as it stood at 74141fe;
## `true` is `_paint_show_preview()` as it stands now, fallback included.
func _pass(g: int, patched: bool) -> Array:
	bridge.world_gen.paint_discard()
	host.set_preview_texture(null)
	full_hits = 0
	var us: Array = []
	for i in range(DABS):
		bridge.world_gen.paint_stroke_at(float(g) * 0.5 + float(i) * 3.0, float(g) * 0.5 + float(i) * 2.0)
		var t0 := Time.get_ticks_usec()
		if patched:
			if not host.set_preview_patch(bridge.build_paint_preview_patch()):
				host.set_preview_texture(bridge.build_paint_preview_texture(), true)
				full_hits += 1
		else:
			host.set_preview_texture(bridge.build_paint_preview_texture())
		us.append(Time.get_ticks_usec() - t0)
		await RenderingServer.frame_post_draw
	return us


func _line(a: Array) -> String:
	a.sort()
	return "%6.3f ms  (%.3f..%.3f)" % [
		float(a[a.size() / 2]) / 1000.0, float(a[0]) / 1000.0, float(a[-1]) / 1000.0]

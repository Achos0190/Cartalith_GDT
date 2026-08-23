extends PanelContainer
class_name ResourceOverlay

## The reference's `#resOverlay` (`updateResOverlay`/`toggleResOverlay`,
## Shift+D, reference lines 10182-10229) — `PARITY_AUDIT.md` §5 item 5.
##
## Read the reference before building: despite the id, this is not a
## "resource-potential at the cursor" readout (`GUI_GAP_REGISTER.md`'s own
## description guessed that, reasonably, from the name alone). The real
## `updateResOverlay()` reads `GW`/`GH`, an approximate memory total, GPU
## status, IndexedDB/Worker availability, LOD state, a handful of boolean
## "active feature" flags and the last generate/render timings — a small
## top-right **engine/perf diagnostics HUD**, refreshed after each render,
## not on mouse motion. "res" is short for "resolution", not "resource". It
## is unrelated to the Resources debug *layer* `layers_popover.gd` already
## draws (a coloured raster) — this is text, and the two can be on at once.
##
## Ported field-for-field where this native port has a real equivalent, and
## honestly dropped where it doesn't (never invented):
## - IndexedDB / Web Worker availability are browser concepts with no native
##   meaning here.
## - The reference's `PERF.gen`/`PERF.render` per-stage millisecond timings
##   have no Rust-side collector anywhere in `cartalith-godot` — adding one
##   is real engine work, out of this ticket's "presentation only" scope.
## - "Seasons"/"Geoid"/"Tides" are reference `state.*` flags with no matching
##   `WorldParams` field in `params.rs` at all (this port never exposed
##   those knobs) — omitted rather than guessed at.
## ponytail: refreshes on a 0.5s Timer while visible (cheapest stand-in for
## the reference's "after every render" hook, which would need a render-
## pipeline callback this port doesn't have) plus on generate/load. Add a
## real render-completion signal if 0.5s latency ever matters.

var _bridge: EngineBridge
var _label: Label
var _timer: Timer

func setup(bridge: EngineBridge) -> void:
	_bridge = bridge
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	clip_contents = true   ## Fixed-width box (app.gd); clip rather than
		## overflow onto the map if a line ever runs long.
	var sb := DccTheme.flat(Color(0.039, 0.047, 0.067, 0.55), 5)
	sb.content_margin_left = 8
	sb.content_margin_right = 8
	sb.content_margin_top = 6
	sb.content_margin_bottom = 6
	add_theme_stylebox_override("panel", sb)

	_label = DccTheme.mono_label("", "text_dim", DccTheme.FS_TINY)
	add_child(_label)

	_timer = Timer.new()
	_timer.wait_time = 0.5
	_timer.timeout.connect(_refresh)
	add_child(_timer)

	bridge.generation_finished.connect(func(_ok: bool): if visible: _refresh())
	bridge.world_loaded.connect(func(): if visible: _refresh())
	visibility_changed.connect(_on_visibility_changed)
	visible = false

func toggle() -> void:
	visible = not visible

func _on_visibility_changed() -> void:
	if visible:
		_refresh()
		_timer.start()
	else:
		_timer.stop()

func _refresh() -> void:
	if _bridge == null or not _bridge.has_world:
		_label.text = "No world generated yet."
		return

	var g := _bridge.grid_size()
	var lines: Array[String] = []
	lines.append("%d × %d  ·  %.2f MP" % [g.x, g.y, (g.x * g.y) / 1.0e6])
	lines.append("Working set: %.1f MB" % (OS.get_static_memory_usage() / 1048576.0))

	var gpu_on := bool(_bridge.param_get("use_gpu"))
	var used := _bridge.gpu_stages_used()
	var gpu_str: String
	if not gpu_on:
		gpu_str = "off"
	elif used.is_empty():
		gpu_str = "on · no stages dispatched"
	else:
		gpu_str = "on · %d stage%s" % [used.size(), "" if used.size() == 1 else "s"]
	lines.append("GPU: %s" % gpu_str)
	lines.append("Quality: %s" % _bridge.quality_tier())

	## Only the flags that exist as real `params.rs` entries -- see this
	## file's own header comment on Seasons/Geoid/Tides.
	if _bridge.world_gen.has_method("get_params"):
		var params: Dictionary = _bridge.world_gen.get_params()
		var feats: Array[String] = []
		if bool(params.get("tect.dynamic_lithology", false)):
			feats.append("DynLith")
		if bool(params.get("climate.currents", false)):
			feats.append("Currents")
		if bool(params.get("volc.provinces", false)):
			feats.append("VolcProv")
		if not feats.is_empty():
			lines.append("Active: %s" % ", ".join(feats))

	_label.text = "\n".join(lines)

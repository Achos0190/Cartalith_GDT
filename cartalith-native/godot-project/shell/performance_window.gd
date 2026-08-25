extends AcceptDialog
class_name PerformanceWindow

## Preferences ▸ Memory ▸ Working set… (`DCC_SHELL_SPEC.md` §2.5's Memory
## group, and the Performance group's own "GPU acceleration... backend
## readout" line).
##
## Was a placeholder ("Being ported from main.gd's performance dialog") with
## `open_performance()` wired in `app.gd` but no menu item anywhere calling
## it -- a real window nothing could reach, found during the 2026-08-19 GUI
## audit. What it now shows is exactly what is real and already unwired:
## `EngineBridge.gpu_stages_used()` (backed by the real `#[func]`
## `get_gpu_stages_used`, which stages the *last* generate actually
## dispatched to the GPU -- not which ones merely could be, per that
## binding's own doc comment), `quality_tier()`/`quality_tiers()`/
## `recommended_quality_tier()` (four real `#[func]`s, matching
## `DCC_SHELL_SPEC.md` §2.5's tier names exactly, already used by
## `menus.gd`'s Preferences ▸ Render quality submenu -- this window is the
## second, read-only place the same live state is worth showing), and
## Godot's own `OS.get_static_memory_usage()` (the same source `app.gd`'s
## `_wire_status()` already feeds into the menu bar's `top_mem` readout).
##
## §2.5's "Devices" checklist, per-device utilisation and VRAM budget are not
## rebuilt here -- `menus.gd`'s own `_todo()` entries for those already carry
## an accurate reason (`cartalith_gpu::init_gpu()` requests one adapter, no
## enumeration exists), and duplicating that disclosure in a second place
## would be the two-views-of-one-gap problem this audit's own brief warns
## against, not a second finding.

var bridge: EngineBridge
var _body: VBoxContainer

## Phone (§13) -- PH-07. This window had none of the shell's phone treatment, so
## it opened as a 560x420 desktop card in the middle of a 1440x3168 panel with
## its only way out -- `AcceptDialog`'s own OK button -- measured at 29 dp.
## Nothing here needs stacking: it is one column of prose. What it needs is the
## content scale, the tap floor, and somewhere for six autowrapping notes to
## scroll once they are set to a 393 dp measure instead of a 560 px one.
var _phone := false

func setup(b: EngineBridge) -> void:
	bridge = b
	title = "Performance"
	size = Vector2i(560, 420)
	ok_button_text = "Close"
	## `get_parent()` is the shell: `app.gd` adds this window to itself and then
	## calls `setup()`. Asked for that way rather than added to the signature,
	## because the parent is already the right object at every call site.
	_phone = DccWidgets.phone_window(self, get_parent())
	_body = VBoxContainer.new()
	_body.add_theme_constant_override("separation", 4)
	if _phone:
		## The head sits OUTSIDE the scroll, not inside `_body`: `_rebuild()`
		## clears every child of `_body` on each refresh, and a header parented
		## there would be destroyed by the first one.
		var outer := VBoxContainer.new()
		outer.add_theme_constant_override("separation", 0)
		add_child(outer)
		DccWidgets.phone_head(outer, "Performance", "gpu · quality · memory")
		var scroll := ScrollContainer.new()
		scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
		outer.add_child(scroll)
		_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		scroll.add_child(_body)
	else:
		add_child(_body)
	bridge.generation_finished.connect(func(_ok: bool): if visible: _rebuild())
	bridge.world_loaded.connect(func(): if visible: _rebuild())
	_rebuild()

func _rebuild() -> void:
	for c in _body.get_children():
		_body.remove_child(c)
		c.queue_free()

	var gpu := DccWidgets.section(_body, "GPU")
	var gpu_on := bool(bridge.param_get("use_gpu"))
	DccWidgets.note(gpu, "GPU acceleration: %s (Preferences ▸ GPU acceleration)." % ("on" if gpu_on else "off"))
	if not bridge.has_world:
		DccWidgets.note(gpu, "No generate yet -- stages actually dispatched are only known after one.")
	else:
		var used := bridge.gpu_stages_used()
		if not gpu_on:
			DccWidgets.note(gpu, "Last generate ran entirely on CPU (GPU acceleration was off).")
		elif used.is_empty():
			DccWidgets.note(gpu, "GPU acceleration is on, but the last generate reported no stages dispatched to it -- likely no eligible adapter (cartalith_gpu::init_gpu() found none).")
		else:
			DccWidgets.note(gpu, "Stages the last generate actually ran on the GPU: %s." % ", ".join(used))

	var quality := DccWidgets.section(_body, "Render quality")
	var tiers := bridge.quality_tiers()
	var current := bridge.quality_tier()
	var recommended := bridge.recommended_quality_tier()
	DccWidgets.note(quality, "Current: %s · recommended for this machine: %s." % [current, recommended])
	DccWidgets.note(quality, "All tiers: %s -- set from Preferences ▸ Render quality." % ", ".join(tiers))

	var mem := DccWidgets.section(_body, "Memory")
	var used_gb := OS.get_static_memory_usage() / 1073741824.0
	DccWidgets.note(mem, "Working set: %.2f GB (Godot's own OS.get_static_memory_usage()). No portable total-system-memory query exists to show it as \"of N GB\", per §2.5's own reading -- reported alone rather than paired with a guessed denominator.")

	DccWidgets.note(_body, "Devices, multi-GPU mode and VRAM budget: see Preferences ▸ Performance -- no per-device enumeration exists in cartalith-gpu (GPU_LAYER_INTEGRATION_SCOPE.md).")

	## PH-07: every row above is a fresh node, and a generate finishing while
	## this window is open rebuilds them behind the one-shot fit `open()` did.
	## Idempotent by meta-flag, so this only touches what was just made.
	if _phone and get_parent() != null and get_parent().has_method("phone_fit"):
		get_parent().phone_fit(self, 1.0)

func open() -> void:
	_rebuild()
	if DccWidgets.phone_present(self, get_parent()):
		## `1.0`: the scale is already applied once as `content_scale_factor`.
		## After every `_rebuild()`, because that replaces the whole body.
		get_parent().phone_fit(self, 1.0)
		return
	popup_centered()

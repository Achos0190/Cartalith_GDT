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
## ## The open question, and it is not this file's to answer
##
## **`DCC_SHELL_SPEC.md` designs a menu ROW, not a window.** §2.5's Memory
## group has `Working set | Read-only, \`1.6 GB of 12 GB\``, and the Performance
## group has `GPU acceleration | Toggle + backend readout`. Both are rows in a
## Preferences dropdown. No section of the spec draws a diagnostics window: §8
## (Asset library) and §9 (Data manager) are the only two windows it specifies,
## and §2.6's Window menu lists exactly those two as the ones that "appear here
## while open". So *whether a diagnostics window should exist at all* is a
## designer's decision that has never been taken, and taking it here would be
## inventing design authority -- the failure `world_data_window.gd`'s header
## documents in its own false citation.
##
## What this pass does instead: leaves the decision open, states it here so it
## reaches `DESIGN_HANDOFF.md` rather than dying in a diff, and brings the
## chrome of the window that already exists into the DCC vocabulary the rest of
## the shell uses -- `DccWidgets.section()` bands with `§` headers, notes in the
## dock's own prose voice, and nothing invented beyond that.
##
## ## Measured 2026-09-05, three named densities, `_lanea_probe.gd`
##
## The content did not fit the window, at any of them, and there was no scroll
## on the pointer path to reveal it. Laid-out extent against the declared
## `560x420`:
##
##                                          before    after this pass
##   desktop  1920x1080 (pointer, base)     545 px      420 px
##   laptop   1600x900  (`is_laptop()`)     545 px      420 px
##   tablet   2560x1600 (`is_tablet()`)    1088 px      420 px
##
## 420 is the declared height exactly, which is what a scroll should read
## as: the column now ends at the box instead of past it. The `before`
## column was taken at HEAD, before the `Devices` section this pass also
## adds -- that section makes the column taller still, and the scroll is
## why it costs nothing.
##
## The tablet figure is not a surprise once stated: `DccWidgets.note()` resolves
## `role_px("fs_prose")`, which is 11 on a pointer and **14** on touch, and this
## window is nothing but autowrapping notes -- **eight of them on screen at once**
## (twelve `note()` call sites, of which the five GPU-branch ones are mutually
## exclusive) under four `section()` headers. The `ScrollContainer` that answers
## it was already here; it was just inside `if _phone`. It is now unconditional,
## which is the whole fix -- no size, no font and no note text changed with it.
##
## §2.5's "Devices" checklist, per-device utilisation and VRAM budget are not
## rebuilt here -- `menus.gd`'s own `_todo()` entries for those already carry
## an accurate reason (`cartalith_gpu::init_gpu()` requests one adapter, no
## enumeration exists), and duplicating that disclosure in a second place
## would be the two-views-of-one-gap problem this audit's own brief warns
## against, not a second finding.

var bridge: EngineBridge
var _body: VBoxContainer

## Phone (§13) -- PH-12. This window had none of the shell's phone treatment, so
## it opened as a 560x420 desktop card in the middle of a 1440x3168 panel with
## its only way out -- `AcceptDialog`'s own OK button -- measured at 29 dp.
## Nothing here needs stacking: it is one column of prose. What it needs is the
## content scale and the tap floor. *It also used to be the only density with
## somewhere for the notes to scroll; since 2026-09-05 every density has that,
## so this flag no longer selects the scroll -- see `setup()`.*
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
	## The head sits OUTSIDE the scroll, not inside `_body`: `_rebuild()` clears
	## every child of `_body` on each refresh, and a header parented there would
	## be destroyed by the first one.
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)
	if _phone:
		DccWidgets.phone_head(outer, "Performance", "gpu · quality · memory")
	## **Unconditional since 2026-09-05.** This whole block was inside
	## `if _phone`, and the pointer path was a bare `add_child(_body)` -- so the
	## one density whose type is *largest* (`fs_prose` 14 on touch, against 11
	## on a pointer) was the one with no way to reach the bottom of the column.
	## See the measurement table in this file's header for the three figures.
	##
	## `SCROLL_MODE_DISABLED` on the horizontal axis is deliberate and is also
	## the axis to watch: a disabled axis folds the child's minimum width into
	## the container's own, so an over-wide leaf would push the *dialog* wider
	## with no scrollbar to show for it (`MISTAKES.md`'s disabled-axis row).
	## Measured at 216 px against a 560 px window, all three densities, and
	## `_lanea_probe.gd` asserts it rather than leaving it to be rediscovered --
	## every note here is autowrapping with `DccWidgets.note()`'s own 190 px
	## floor, which is what keeps the number that low.
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(scroll)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_body)
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
		## `WorldGen.gpu_last_backend` -- the backend the last generate really
		## opened, recorded by `cartalith_gpu::record_opened_backend` at the
		## moment `generate_terrain` decides. Not `menus.gd::_active_backend()`,
		## which reads the device enumeration and reports what a request *would*
		## prefer: that answer is identical whether the request succeeded, landed
		## on another backend, or opened nothing at all, so it cannot be used to
		## check a claim about what ran.
		var backend := bridge.gpu_last_backend()
		if not gpu_on:
			DccWidgets.note(gpu, "Last generate ran entirely on CPU (GPU acceleration was off).")
		elif backend == "":
			## Was "likely no eligible adapter", hedged because nothing in the
			## app could tell. It can now: an empty backend means the device was
			## never opened, which is a different fact from every stage falling
			## back off one that was.
			DccWidgets.note(gpu, "GPU acceleration is on, but the last generate opened no GPU device at all -- no eligible adapter, over the VRAM budget, or an adapter that cannot bind a grid this size. It ran on the CPU.")
		elif used.is_empty():
			DccWidgets.note(gpu, "Backend the last generate actually opened: %s. No stage was dispatched to it, though -- every one fell back to the CPU." % backend)
		else:
			DccWidgets.note(gpu, "Backend the last generate actually opened: %s. Stages that ran on it: %s." % [backend, ", ".join(used)])

	var quality := DccWidgets.section(_body, "Render quality")
	var tiers := bridge.quality_tiers()
	var current := bridge.quality_tier()
	var recommended := bridge.recommended_quality_tier()
	DccWidgets.note(quality, "Current: %s · recommended for this machine: %s." % [current, recommended])
	DccWidgets.note(quality, "All tiers: %s -- set from Preferences ▸ Render quality." % ", ".join(tiers))

	var mem := DccWidgets.section(_body, "Memory")
	var used_gb := OS.get_static_memory_usage() / 1073741824.0
	## The `%` operator was missing, so this window has been shipping the
	## literal string `%.2f` where the number goes -- caught by screenshot in
	## the 2026-08-25 conformance sweep, not by any test, because a `#[func]`
	## returning a figure proves nothing about whether it reaches a Label.
	DccWidgets.note(mem, "Working set: %.2f GB (Godot's own OS.get_static_memory_usage()). No portable total-system-memory query exists to show it as \"of N GB\", per §2.5's own reading -- reported alone rather than paired with a guessed denominator." % used_gb)

	## `GUI_GAP_REGISTER.md` §50 registered the one honest defect in the row
	## above: on the handset it read **0.2 GB** while `dumpsys meminfo` reported
	## **818 MB** of TOTAL PSS for the same process at the same moment. Neither
	## the Rust allocations nor the GPU's own textures live inside Godot's
	## static heap, so the figure on screen was never the figure that gets the
	## app killed. These are the parts the renderer does know, reported beside
	## it rather than instead of it -- and they are what the 2026-08-25 memory
	## diagnosis measured the hi-DPI pass against.
	DccWidgets.note(mem, "Video memory: %s -- textures %s, buffers %s (Godot's own render monitors; outside the working-set figure above)." % [
		String.humanize_size(int(Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED))),
		String.humanize_size(int(Performance.get_monitor(Performance.RENDER_TEXTURE_MEM_USED))),
		String.humanize_size(int(Performance.get_monitor(Performance.RENDER_BUFFER_MEM_USED))),
	])
	var icons := DccIcons.cache_stats()
	DccWidgets.note(mem, "Glyph raster cache: %d entries, %s. Last frame: %d draw calls over %d objects." % [
		int(icons["entries"]), String.humanize_size(int(icons["bytes"])),
		int(Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME)),
		int(Performance.get_monitor(Performance.RENDER_TOTAL_OBJECTS_IN_FRAME)),
	])

	## Build-conditional, not a permanent absence: `menus.gd:_build_gpu_devices_menu`
	## draws those three rows when `EngineBridge.gpu_api` is true and falls back to
	## a `_todo` naming the missing binding when it is not. This note said "no
	## per-device enumeration exists in cartalith-gpu" until 2026-09-03; it does --
	## `cartalith_gpu::enumerate_devices` (`multi.rs`), bound as
	## `WorldGen::gpu_enumerate_devices`.
	## **Was parented to `_body` directly**, which put it outside every section:
	## no `§` header over it, and `section()`'s own 14 px left margin missing, so
	## the one note in the window carrying a cross-reference sat a step to the
	## left of the seven above it and read as a footer nobody had styled. Its own
	## section now, titled from `DCC_SHELL_SPEC.md` §2.5's own row name --
	## `| | Devices | Expands to a per-device checklist with live utilisation`
	## -- rather than a title invented for it.
	var devices := DccWidgets.section(_body, "Devices")
	DccWidgets.note(devices, "Devices, multi-GPU mode and VRAM budget: see Preferences ▸ Performance. Per-device enumeration is cartalith-gpu's enumerate_devices, bound as WorldGen.gpu_enumerate_devices; those rows are live whenever the loaded GDExtension build exposes that binding, and say so on hover when it does not (GPU_LAYER_INTEGRATION_SCOPE.md).")

	## PH-12: every row above is a fresh node, and a generate finishing while
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

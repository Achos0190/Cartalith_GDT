extends RefCounted
class_name DiagnosticReport

## `Help ▸ Save diagnostic report` -- `LARGE_ITEM_RULINGS.md`'s "Build" ruling
## on the reference's `Report an issue` row (§2.7): *"Replace with a local
## diagnostic dump -- rename to a save diagnostic report action writing
## generation info, missing bindings, project format version, GPU state and
## the last error to a file the user attaches themselves. No endpoint
## required."* The old row is `menus.gd`'s own comment on why it could not be
## more than a `_todo`: SS2.7 names no issue tracker, support address or crash
## endpoint this port could send to, and inventing one would be worse than
## leaving it disabled. This never sends anything anywhere -- it writes one
## text file and tells the user where, exactly like `data_manager_window.gd`'s
## existing exports (`_host.reveal_on_disk()` / `_host.set_status()`).
##
## Three of the five named readouts already existed as trivial calls and are
## reused rather than re-implemented: `GenInfoDialog._dump_text()` already
## dumps generation info, `EngineBridge.missing_bindings()` and
## `EngineBridge.project_format_version()` inside it -- `menus.gd`'s own
## comment on the old `_todo` row said as much ("pairing that with the version
## and build string from About is exactly the body a report wants").
## GPU state and the last error did not exist anywhere and are built here:
## GPU state from the multi-GPU/`RenderingServer` accessors `engine_bridge.gd`
## and `menus.gd` already expose for their own Preferences rows; the last
## error from a small retention `engine_bridge.gd` gained alongside this
## file (`EngineBridge.note_error()` / `.last_error()`) -- nothing in the
## codebase retained one before, confirmed by grepping the whole repository
## for `last_error` and finding nothing but this addition.

## Builds the report, writes it under the same storage root
## `data_manager_window.gd`'s exports already use, and tells the user where --
## `app.reveal_on_disk()` opens the OS file manager on desktop (the same
## fallback-aware call every export already goes through), and the status
## line always carries the full path too, since `reveal_on_disk()` is a
## desktop-only no-op on a phone or tablet.
static func write(app: Node, bridge: EngineBridge) -> void:
	var dir := DccSettings.storage_root("exports")
	## Not return-checked: `engine_bridge.gd`'s own `_preset_path()` calls this
	## the same bare way and lets the write below be the real failure signal --
	## `make_dir_recursive_absolute` does not error when the directory is
	## already there, which is the common case after the first report.
	DirAccess.make_dir_recursive_absolute(dir)
	## Filesystem-safe stamp: `Time`'s own separator is `:`, illegal in a
	## Windows filename. One report per call rather than one fixed name, so
	## filing several reports in a session (plausible -- a report is exactly
	## what a user reaches for right after something goes wrong more than
	## once) does not silently overwrite the previous one's evidence.
	var stamp: String = Time.get_datetime_string_from_system().replace(":", "-")
	var path := dir.path_join("cartalith_diagnostic_report_%s.txt" % stamp)
	var f := FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		app.set_status("hint", "could not write diagnostic report (%s)"
			% error_string(FileAccess.get_open_error()), "accent")
		return
	f.store_string(_build(app, bridge))
	f.close()
	var shown: bool = app.reveal_on_disk(path)
	app.set_status("hint", "diagnostic report saved -> %s"
		% (path.get_file() if shown else path), "accent")

static func _build(app: Node, bridge: EngineBridge) -> String:
	var lines: Array[String] = []
	lines.append("Cartalith diagnostic report")
	lines.append("Written %s local time -- Godot %s · %s"
		% [Time.get_datetime_string_from_system(), Engine.get_version_info().string, OS.get_name()])
	if bridge != null and bridge.generating:
		lines.append("A generation is running right now -- the GPU/device readouts below are whatever was last measured, not this run's.")
	lines.append("")
	## Requirement #2 (see the ruling row): redact nothing, but a path is not
	## something to ship unremarked either. Every path this file can contain
	## -- the storage roots below, and anything a captured error names --
	## comes straight from `OS.get_user_data_dir()` or a user-chosen save
	## location, and on Windows that ordinarily embeds the Windows account
	## name. Said once, up front, rather than guessed-at per line.
	lines.append("This file may contain local filesystem paths. On Windows those")
	lines.append("normally embed your Windows account name (e.g. C:\\Users\\<name>\\...).")
	lines.append("Nothing below is redacted -- review before attaching to a public issue")
	lines.append("if that matters to you.")
	lines.append("")

	lines.append("== Generation info · missing bindings · project format version ==")
	lines.append(_gen_info_section(app))
	lines.append("")

	lines.append("== GPU state ==")
	lines.append(_gpu_section(bridge))
	lines.append("")

	lines.append("== Last error ==")
	lines.append(_last_error_section(bridge))

	return "\n".join(lines)

## Readouts 1-3. `GenInfoDialog._dump_text()` already IS this -- grid, seed,
## quality tier, a GPU on/off line (the GPU section below goes much further),
## `missing_bindings()` and `project_format_version()`, then the full
## generation-parameter dump. Reused rather than re-implemented: duplicating
## its sorted-keys/JSON.stringify loop here would be the same list drifting
## out of step with itself the moment one of them is edited.
static func _gen_info_section(app: Node) -> String:
	return String(app.gen_info_dialog._dump_text())

## Readout 4. Two different questions this project has already found can
## disagree (`STATUS.md`, 2026-09-02: `forward_plus`/vulkan loses the device
## on generate while `gl_compatibility` is clean on the same machine) --
## which renderer Godot itself is drawing the shell with, and what state the
## separate wgpu compute pipeline the four GPU-eligible substrate stages
## dispatch to is in. Answered separately rather than folded into one line.
static func _gpu_section(bridge: EngineBridge) -> String:
	var lines: Array[String] = []
	lines.append("Godot renderer:")
	lines.append("  configured rendering method: %s"
		% String(ProjectSettings.get_setting("rendering/renderer/rendering_method", "unknown")))
	var rd := RenderingServer.get_rendering_device()
	lines.append("  RenderingDevice: %s -- %s" % ["present" if rd != null else "null",
		"Vulkan/D3D12-class device open this run" if rd != null
			else "no RD-backed device open -- expected under gl_compatibility, and under --headless"])
	var adapter := RenderingServer.get_video_adapter_name()
	if adapter == "":
		lines.append("  video adapter: unknown (RenderingServer reported nothing -- expected under --headless)")
	else:
		lines.append("  video adapter: %s · %s · driver API %s · %s"
			% [adapter, RenderingServer.get_video_adapter_vendor(),
				RenderingServer.get_video_adapter_api_version(), _adapter_type_name(RenderingServer.get_video_adapter_type())])

	lines.append("Compute GPU (wgpu, the four GPU-eligible substrate stages):")
	if bridge == null or not bridge.gpu_api:
		lines.append("  unavailable -- this build predates the multi-GPU API (WorldGen.gpu_enumerate_devices/gpu_set_multi_mode missing).")
		return "\n".join(lines)
	lines.append("  requested (Preferences > GPU acceleration): %s" % ("on" if bool(bridge.param_get("use_gpu")) else "off"))
	var backend := bridge.gpu_last_backend()
	if backend != "":
		lines.append("  backend of the last generate (measured): %s" % backend)
	elif "gpu_last_backend" in bridge.missing_bindings():
		lines.append("  backend of the last generate: unavailable -- this build predates gpu_last_backend().")
	else:
		lines.append("  backend of the last generate: not measured yet -- no GPU generate has completed this session.")

	var devices: Array = bridge.gpu_devices()
	if devices.is_empty():
		lines.append("  devices: none enumerated -- wgpu found no adapters, or Preferences > GPU > Devices has never been opened this session. Generation runs on the CPU either way.")
	else:
		var selected := bridge.gpu_selected_devices()
		lines.append("  devices: %d enumerated (%s selected)" % [devices.size(),
			"automatic, highest-performance" if selected.is_empty() else ", ".join(selected)])
		for d in devices:
			var dd: Dictionary = d
			lines.append("    - %s · %s · %s%s" % [String(dd.get("name", "?")), String(dd.get("kind", "?")),
				String(dd.get("backend", "?")), "  [software rasterizer, never dispatched to]" if bool(dd.get("software", false)) else ""])
	lines.append("  multi-GPU mode: %s" % bridge.gpu_multi_mode())
	## `0` is the sentinel for **no cap**, not a measured budget of zero, and it
	## is the shipping default (`GPU_VRAM_CHOICES[0] == 0.0`), so printing it as
	## `0.0 GB` told every reader of every stock-install report that the GPU path
	## was budget-refused. `cartalith-gpu/src/multi.rs::vram_verdict_for` returns
	## `Ok` unconditionally when `budget_bytes == 0`, and `menus.gd:2391` labels
	## the same value "No cap". `menus.gd` avoids this exact shape three lines
	## from there -- "rather than printing '0x0 needs about 0 MB', which reads
	## like a measurement" -- and that care was not carried here until now.
	var _vram: float = bridge.gpu_vram_budget_gb()
	var _vram_text := "— no cap set (the default)" if _vram <= 0.0 else "%.1f GB" % _vram
	lines.append("  VRAM budget: %s · fallback when full: %s" % [_vram_text, bridge.gpu_vram_fallback()])
	lines.append("  readback failures banned this session: %s" % ("yes -- Preferences > GPU > Try the GPU again to clear" if bridge.gpu_readback_failed() else "none"))
	var usage: Array = bridge.gpu_last_device_usage()
	if usage.is_empty():
		lines.append("  memory at end of last GPU generation: not measured yet")
	else:
		for u in usage:
			var uu: Dictionary = u
			lines.append("    - %s: %d MB allocated, %d MB reserved" % [String(uu.get("name", "?")), int(uu.get("allocated_mb", 0)), int(uu.get("reserved_mb", 0))])
	return "\n".join(lines)

static func _adapter_type_name(t: int) -> String:
	match t:
		RenderingDevice.DEVICE_TYPE_INTEGRATED_GPU: return "integrated GPU"
		RenderingDevice.DEVICE_TYPE_DISCRETE_GPU: return "discrete GPU"
		RenderingDevice.DEVICE_TYPE_VIRTUAL_GPU: return "virtual GPU"
		RenderingDevice.DEVICE_TYPE_CPU: return "CPU (software)"
		_: return "other/unknown (%d)" % t

## Readout 5. `EngineBridge._last_error` (see its own doc comment) -- scoped to
## what that file can see and already has a real reason string for: a failed
## generate/import/VRAM-refusal, and a failed project save/load. Not every
## `_report_failure()` in `app.gd` -- said plainly rather than implied, since
## a report that cannot say what it does and does not cover is worse than one
## that says so.
static func _last_error_section(bridge: EngineBridge) -> String:
	if bridge == null:
		return "(no engine bridge on this build's app root.)"
	var e := bridge.last_error()
	if e.is_empty():
		return ("none this session (covers a failed generate, heightmap import, "
			+ "or project save/load -- not every local refusal the UI shows, "
			+ "such as \"nothing to undo\")")
	return "%s -- %s" % [String(e.get("at", "?")), String(e.get("text", ""))]

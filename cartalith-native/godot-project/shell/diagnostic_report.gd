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
## **`DiagnosticReviewDialog` shows the user every row before any of this
## runs.** The round-3 canvas's own contribution -- the part of it that was
## right, against an `OPEN TRACKER` button that contradicts the ruling -- is
## that a pre-filled report is a claim about what the app knows, so the rows
## are reviewable first. `manifest()` below is that table, and `build_text()`
## writes the same array, which is what makes the panel and the file agree by
## construction rather than by two code paths being kept in step.
##
## Three of the five named readouts already existed as trivial calls and are
## reused rather than re-implemented: `GenInfoDialog` holds generation info,
## `EngineBridge.missing_bindings()` and `EngineBridge.project_format_version()`
## -- `menus.gd`'s own comment on the old `_todo` row said as much ("pairing
## that with the version and build string from About is exactly the body a
## report wants"). Those three arrived here as one `_dump_text()` call until
## 2026-09-06; the review panel needs them as separate rows with separate
## sources, and needs the seed and parameter dump separable from the summary,
## so `GenInfoDialog` gained six statics (`grid_text`, `seed_text`,
## `quality_text`, `format_version_text`, `bindings_text`, `params_text`) that
## its own `_dump_text()` now composes. One implementation, two callers -- the
## drift the old note guarded against is still guarded against.
## GPU state and the last error did not exist anywhere and are built here:
## GPU state from the multi-GPU/`RenderingServer` accessors `engine_bridge.gd`
## and `menus.gd` already expose for their own Preferences rows; the last
## error from a small retention `engine_bridge.gd` gained alongside this
## file (`EngineBridge.note_error()` / `.last_error()`) -- nothing in the
## codebase retained one before, confirmed by grepping the whole repository
## for `last_error` and finding nothing but this addition.

## The two toggles the review panel offers. `Help ▸ Save diagnostic report`
## opens that panel; `write()` stays callable on its own (that is what
## `_diagreport_probe.gd` exercises) and then uses `default_options()`.
const OPT_SEED_PARAMS := "seed_params"
const OPT_PROJECT_PATH := "project_path"

## Seed and parameters **on**: a world is reproducible from its seed, so a
## report without them can rarely be acted on. Project path **off**: it is a
## filesystem path, and on Windows it ordinarily embeds the account name --
## the one field where the safe default and the useful default disagree.
static func default_options() -> Dictionary:
	return {OPT_SEED_PARAMS: true, OPT_PROJECT_PATH: false}

## Every value the report can carry, in the order the file writes it.
##
## One entry per attached value: `key`, the `label` both surfaces show, the
## `source` (the symbol the value was read from, not a description of it), a
## one-line `preview` for the panel, and the `body` the file gets. `included`
## false carries `why` -- either a toggle the user turned off, or a value this
## build genuinely cannot supply, which is dashed with the reason rather than
## written as an empty section.
##
## **`why` is present exactly when `included` is false**, so a reader takes it
## with `r["included"]` rather than testing a sentinel -- there is no "no
## reason" string, and an included row carries no `why` key at all.
##
## The two optional rows are `seed` and `path`; the panel knows which by key
## because it owns the toggles, and nothing here encodes a gate the caller
## would have to keep in step.
static func manifest(app: Node, bridge: EngineBridge, opts: Dictionary) -> Array[Dictionary]:
	var rows: Array[Dictionary] = []
	var seed_on: bool = bool(opts.get(OPT_SEED_PARAMS, true))
	var path_on: bool = bool(opts.get(OPT_PROJECT_PATH, false))
	var has_world: bool = bridge != null and bridge.has_world

	rows.append(_row("session", "Session & platform",
		"Engine.get_version_info() · OS.get_name() · Time.get_datetime_string_from_system()",
		_session_section(bridge)))

	if has_world:
		rows.append(_row("generation", "Generation info",
			"EngineBridge.grid_size() / .quality_tier() / .param_get(\"use_gpu\"), via GenInfoDialog",
			GenInfoDialog.grid_text(bridge) + "\n" + GenInfoDialog.quality_text(bridge)))
	else:
		rows.append(_absent("generation", "Generation info",
			"EngineBridge.has_world", GenInfoDialog.NO_WORLD))

	## The one row the seed toggle gates. Kept as its own section rather than
	## folded into Generation info precisely so it can be dropped without
	## rewriting anything else -- see `GenInfoDialog._dump_text()`'s own note.
	if not seed_on:
		rows.append(_absent("seed", "Seed & generation parameters",
			"WorldGen.get_seed() · WorldGen.get_params()",
			"you turned this off -- the report will not say which world this is or how to rebuild it"))
	elif not has_world:
		rows.append(_absent("seed", "Seed & generation parameters",
			"WorldGen.get_seed() · WorldGen.get_params()", GenInfoDialog.NO_WORLD))
	else:
		var seed_line := GenInfoDialog.seed_text(bridge)
		rows.append(_row("seed", "Seed & generation parameters",
			"WorldGen.get_seed() · WorldGen.get_params()",
			(seed_line + "\n" if seed_line != "" else "")
				+ GenInfoDialog.PARAMS_HEADING + "\n" + GenInfoDialog.params_text(bridge)))

	if bridge == null:
		rows.append(_absent("bindings", "Missing bindings", "EngineBridge.missing_bindings()",
			"no engine bridge on this build's app root"))
		rows.append(_absent("format", "Project format version",
			"EngineBridge.project_format_version()", "no engine bridge on this build's app root"))
	else:
		rows.append(_row("bindings", "Missing bindings",
			"EngineBridge.missing_bindings()", GenInfoDialog.bindings_text(bridge)))
		rows.append(_row("format", "Project format version",
			"EngineBridge.project_format_version()", GenInfoDialog.format_version_text(bridge)))

	rows.append(_row("gpu", "GPU state",
		"RenderingServer.* · EngineBridge.gpu_devices() / .gpu_last_backend() / .gpu_stages_used()",
		_gpu_section(bridge)))
	rows.append(_row("error", "Last error this session",
		"EngineBridge.last_error()", _last_error_section(bridge)))
	rows.append(_log_row())
	rows.append(_project_path_row(app, path_on))
	return rows

static func _row(key: String, label: String, source: String, body: String) -> Dictionary:
	return {"key": key, "label": label, "source": source, "body": body,
		"included": true, "preview": _preview(body)}

static func _absent(key: String, label: String, source: String, why: String) -> Dictionary:
	return {"key": key, "label": label, "source": source, "body": "", "included": false,
		"preview": "", "why": why}

## The panel's right-hand column: the value's first line, elided. Never a
## substitute for the value -- the file gets `body` whole.
static func _preview(body: String) -> String:
	var first := body.split("\n")[0]
	var extra := body.split("\n").size() - 1
	if first.length() > 46:
		first = first.substr(0, 45) + "…"
	return first if extra <= 0 else "%s  (+%d line%s)" % [first, extra, "" if extra == 1 else "s"]

static func _session_section(bridge: EngineBridge) -> String:
	var lines: Array[String] = []
	lines.append("Written %s local time" % Time.get_datetime_string_from_system())
	lines.append("Godot %s · %s" % [Engine.get_version_info().string, OS.get_name()])
	if bridge != null and bridge.generating:
		lines.append("A generation is running right now -- the GPU/device readouts below are whatever was last measured, not this run's.")
	return "\n".join(lines)

## **The one readout the round-3 canvas got wrong, and it was measured rather
## than believed.** The canvas dashed a log tail as "there is no log file". On
## desktop there is: `debug/file_logging/enable_file_logging` reads `false`,
## and **Godot enables file logging by default on desktop regardless**, so it writes
## `user://logs/godot.log` and rotates five deep -- confirmed 2026-09-06 by
## printing a unique marker and finding it in the file the same run.
##
## **An earlier version of this comment credited a `.pc` feature override in
## `project.godot`. There is none** -- `grep file_logging project.godot`
## returns nothing. The dash reason the user actually sees always said
## "Godot's file logging is a desktop-only default", which was right while
## this explanation of it was wrong.
##
## The test below is the open, not the setting: `get_setting()` on the base key
## returns `false` while logging is on, because the logger resolves the feature
## override during early boot through a path the base lookup does not see. So
## the honest question is "can this build read its own log", and that is also
## the right answer on Android, where the `pc` tag does not apply, nothing
## writes a log, and the row dashes with that as its reason.
const LOG_TAIL_LINES := 40
const LOG_TAIL_BYTES := 65536

static func _log_row() -> Dictionary:
	var path := String(ProjectSettings.get_setting("debug/file_logging/log_path",
		"user://logs/godot.log"))
	var label := "Log tail (last %d lines)" % LOG_TAIL_LINES
	var source := "%s -- Godot's own file log" % path
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		return _absent("log", label, source,
			"nothing to read at %s: Godot's file logging is a desktop-only default (enable_file_logging.pc), so this platform writes no log" % path)
	## Tail, not the whole file: a long session's log is unbounded and a bug
	## report wants the end of it. Read through `get_buffer` after a seek --
	## `get_as_text()` returns the file from the beginning whatever the cursor
	## is, which would defeat the seek silently.
	var length := f.get_length()
	var start: int = max(0, length - LOG_TAIL_BYTES)
	f.seek(start)
	var text := f.get_buffer(length - start).get_string_from_utf8()
	f.close()
	var all := text.split("\n")
	var keep: Array[String] = []
	for i in range(max(0, all.size() - LOG_TAIL_LINES), all.size()):
		keep.append(all[i])
	var body := "\n".join(keep).strip_edges()
	if body == "":
		return _absent("log", label, source, "the log file at %s is empty" % path)
	return _row("log", label, source, body)

static func _project_path_row(app: Node, on: bool) -> Dictionary:
	var label := "Project file path"
	var source := "app.current_project_path"
	if not on:
		return _absent("path", label, source,
			"off by default -- a filesystem path, and on Windows it embeds your account name")
	if app == null or not ("current_project_path" in app):
		return _absent("path", label, source, "this build's app root holds no current_project_path")
	var p := String(app.current_project_path)
	if p == "":
		return _absent("path", label, source, "no project is open -- this world has never been saved or loaded")
	return _row("path", label, source, p)

## Builds the report, writes it under the same storage root
## `data_manager_window.gd`'s exports already use, and tells the user where --
## `app.reveal_on_disk()` opens the OS file manager on desktop (the same
## fallback-aware call every export already goes through), and the status
## line always carries the full path too, since `reveal_on_disk()` is a
## desktop-only no-op on a phone or tablet.
##
## Returns the path written, or `""` on failure, so the review panel can say
## which file it produced instead of repeating the status line's guesswork.
static func write(app: Node, bridge: EngineBridge, opts: Dictionary = {}) -> String:
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
	##
	## **That guarantee did not hold, and the fix is the loop below.** The
	## stamp resolves to one second and `Time` offers nothing finer, so two
	## reports written inside the same second landed on the same filename and
	## the second silently replaced the first -- the exact outcome this comment
	## claimed to prevent. Found by `_diagreview_probe.gd`, which writes one
	## report per toggle case: the second case reported "Save wrote no new
	## file" because it had overwritten the first. Real for a user too, and for
	## the same reason the comment gives -- a report is what you reach for when
	## something has just gone wrong twice in a row.
	var stamp: String = Time.get_datetime_string_from_system().replace(":", "-")
	var path := dir.path_join("cartalith_diagnostic_report_%s.txt" % stamp)
	var n := 2
	while FileAccess.file_exists(path) and n <= 99:
		path = dir.path_join("cartalith_diagnostic_report_%s-%d.txt" % [stamp, n])
		n += 1
	var f := FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		app.set_status("hint", "could not write diagnostic report (%s)"
			% error_string(FileAccess.get_open_error()), "accent")
		return ""
	f.store_string(build_text(app, bridge, opts))
	f.close()
	var shown: bool = app.reveal_on_disk(path)
	app.set_status("hint", "diagnostic report saved -> %s"
		% (path.get_file() if shown else path), "accent")
	return path

## The file, built from the same array the panel drew. The
## `WHAT THIS FILE CONTAINS` block is not decoration: it is `manifest()`
## verbatim, so a reader can check the file against what they were shown and
## `_diagreview_probe.gd` can assert the two agree row for row without parsing
## prose. Every section below it is one included row, in the same order.
static func build_text(app: Node, bridge: EngineBridge, opts: Dictionary = {}) -> String:
	var rows := manifest(app, bridge,
		default_options() if opts.is_empty() else opts)
	var lines: Array[String] = []
	lines.append("Cartalith diagnostic report")
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
	## **What the toggles do NOT guarantee, measured rather than assumed.** A
	## row turned off is absent as a SECTION; it is not scrubbed from the file,
	## because the log tail is whatever the application printed and nothing
	## filters it. `_diagreview_probe.gd` caught this the honest way: with the
	## seed row off, its own printed output had put the seed string back into
	## the log, and the probe failed a check that was written as though the
	## toggle were a redaction. Filtering the log would be guessing at what to
	## redact and would cost the log its value, so this says so instead.
	lines.append("The Log tail section is verbatim application output and is not filtered")
	lines.append("by the choices above: a value you excluded can still appear there if the")
	lines.append("app happened to print it.")
	lines.append("")
	lines.append("== WHAT THIS FILE CONTAINS ==")
	for r in rows:
		lines.append("%s %s  [%s]" % ["[x]" if bool(r["included"]) else "[ ]",
			String(r["label"]), String(r["source"])])
		if not bool(r["included"]):
			lines.append("      not included: %s" % String(r["why"]))
	lines.append("")
	for r in rows:
		if not bool(r["included"]):
			continue
		lines.append("== %s ==" % String(r["label"]))
		lines.append("source: %s" % String(r["source"]))
		lines.append(String(r["body"]))
		lines.append("")
	return "\n".join(lines)

## Readout 4. Two different questions this project has already found can
## disagree (`STATUS.md`, 2026-09-02: `forward_plus`/vulkan loses the device
## on generate while `gl_compatibility` is clean on the same machine) --
## which renderer Godot itself is drawing the shell with, and what state the
## separate wgpu compute pipeline the GPU-accelerated stages dispatch to is
## in (generation's seven, the civ layer's two, and Erode's thermal passes --
## `menus.gd::GPU_TOGGLE_TIP`; it said "four" until 2026-09-24). Answered separately rather than folded into one line.
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

	lines.append("Compute GPU (wgpu -- generation, civilisation-layer and Erode stages):")
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

	## **The stage NAMES, which had exactly one home and lost it.** The deleted
	## Performance window (owner ruling 19, 2026-09-06) was the only surface that
	## listed them; `resource_overlay.gd` renders the COUNT only ("on · 4 stages")
	## and this section carried the backend but never the list. A verifier called
	## that the one real capability the ruling costs, so it lands here rather than
	## being dropped -- a report is where you look when the count is surprising.
	var stages: Array = bridge.gpu_stages_used()
	if not stages.is_empty():
		lines.append("  stages dispatched to it: %s" % ", ".join(stages))
	elif backend != "":
		lines.append("  stages dispatched to it: none -- the backend opened and no stage was sent to it.")

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

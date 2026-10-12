extends Control
## The busy overlay: a modal, input-swallowing layer shown while the engine is busy, with one line
## of wit per step (`loading_messages.gd`) over a relief-trace progress bar.
##
## **What.** A full-viewport `Control` (mounted by `app.gd::_mount_busy_overlay` on its own
## `CanvasLayer`, `CANVAS_LAYER`, as the app's last child) that draws everything itself in `_draw`:
## a scrim, a card, the action's title, the RELIEF TRACE (a mountain-range profile swept left to
## right like a heart monitor, its "beats" being ridges -- the section/measure profile's own
## vocabulary from `section_strip.gd::_draw_plot`: accent line, accent-wash fill, dashed level
## line, ticks), the stage line ("stage 04 of 10 · Tectonics"), the message and a foot line.
##
## **Why.** Owner, 2026-10-11: loading messages "per step ... with a pulse style loading bar ...
## like a heart monitor but instead of the heartbeat more a mountain range style relief" and "That
## way we can block/save guard the UI." So this is also the UI GUARD: while `EngineBridge.generating`
## is true the worker thread owns `WorldGen`, and every shell call that reaches it is the
## `Gd<T>::bind() failed` panic the per-handler `generating` guards exist for. This layer makes
## those handlers unreachable through real input (defence in depth: the guards all stay).
##
## **How input is swallowed.** `_input` runs BEFORE GUI dispatch, shortcut dispatch (`MenuBar`
## accelerators) and `_unhandled_key_input` (`app.gd`'s Esc/Delete/Ctrl+Y/Q/Shift+J), and Godot
## calls `_input` on the LAST node in tree order first -- so `_enter` moves this layer to the end
## of the app's children, ahead of `ViewportHost._input` and `layers_popover.gd::_input`, and
## `_input` marks EVERY event handled while `is_blocking()`. `mouse_filter = STOP` over the whole
## viewport is the second layer for the GUI path. An embedded dialog or popup open during the run
## receives events BEFORE any `_input`, so those are muted separately (`_mute_subwindows`).
## Window-manager actions (close, resize, move) are not `InputEvent`s and are never touched.
##
## **Busy sources and the depth counter.** `show_busy`/`hide_busy` are the HTML's
## `showBusy`/`hideBusy` (v1.24's nesting counter: a second source never hides it early, a stray
## hide is clamped at 0 and never "owed"). The ENGINE is one source, acquired on
## `generation_started` (generate, heightmap import, region resample) or, for the landmark pass and
## the viewshed refine (which emit no start signal), on the first frame `bridge.generating` is seen
## true; released on `generation_finished` (ok or failed), `landmark_finished`,
## `landmark_refine_finished`. `_engine_held` makes that source exactly one depth unit, so an
## unpaired `generation_finished(false)` (the VRAM refusal emits one without a start) cannot steal
## another source's count. The WATCHDOG releases a source the engine no longer backs: the engine
## source after `WATCHDOG_FRAMES` frames of `generating == false` with no finish signal, and any
## other depth after the same run of frames with the engine idle. The second kind of source is
## `busy_run` (2026-10-12): the synchronous main-thread actions -- save, open, export, bake, the
## droplet erosion pass, the stale-stage and civilisation recomputes -- show, paint, block, settle
## and hide inside one coroutine, well inside the watchdog's frames (`app.gd::busy_run` lists the
## call sites and the ones deliberately left out).
## Nothing else calls `show_busy` directly; a new caller should go through `busy_run` too, since a
## bare `show_busy` before a synchronous call blocks before anything is drawn.
##
## **Messages follow the real stage.** `generation_stage(index, name, total)` -- the bridge's ONE
## poll of `GenerationProgress`; this file reads only that signal, `bridge.generating` and
## `bridge.busy_kind`, and never calls `WorldGen` -- picks the stage's category and a fresh line at
## once; inside a long stage a new line every `ROTATE_S`, never the previous one.
##
## **Never.** Never call a `world_gen` method or `GenerationProgress` from here. Never offer a
## cancel (the engine has no cancel entry point; `world_workspace.gd::_pg_progress_card` says the
## same). Never hold a non-engine source across more frames than `WATCHDOG_FRAMES` (the watchdog
## will release it; `busy_run` holds `PAINT_FRAMES + SETTLE_FRAMES`). Never draw progress the engine did not report: the trace's head (`_head`) eases
## toward the completed-stages fraction (`_solid`) and never past it, so it is monotonic and stops
## at the last tick the engine reported; the glint running along it is animation, not a measurement.

## The message tables and pickers (`category_for_label`, `stage_category`, `pick_entry`); this file only draws what it returns.
const LoadingMessages := preload("res://shell/loading_messages.gd")

## The `CanvasLayer.layer` `app.gd` mounts this on. Above every canvas item the shell draws (the
## shell itself adds no other `CanvasLayer`; checked 2026-10-11 by grepping `CanvasLayer` in
## `shell/`). A labelled judgement: any value above 1 would do; 100 leaves room either side.
## Embedded windows (dialogs, popups) still draw ABOVE every canvas layer -- see `_mute_subwindows`.
const CANVAS_LAYER := 100

## Frames of `bridge.generating == false` (with a source still held) before the watchdog releases
## it and logs. A labelled judgement: 20 frames is ~0.33 s at 60 Hz -- long enough that no real
## finish path (each clears `generating` and emits its signal in the same call) can trip it, short
## enough that a missed signal costs the user a blink, not a locked window.
const WATCHDOG_FRAMES := 20
## Seconds before a fresh line inside one long stage. The owner brief's "~4 s"; a judgement.
const ROTATE_S := 4.0
## Seconds for the heartbeat glint to run from the start of the trace to the pen (it then starts
## again). A labelled judgement chosen to read as a pulse, not as progress.
const SWEEP_S := 1.6
## How fast the trace's head sweeps toward a newly reported stage: at least this fraction of the
## range per second, faster for a bigger jump (`_advance_head`). A labelled judgement: one stage
## segment (0.1) in about 0.3 s.
const HEAD_RATE := 0.33
## Seconds for the indeterminate pulse to travel the whole range once. A labelled judgement.
const PULSE_S := 2.4
## Width of the indeterminate pulse, as a fraction of the range. A labelled judgement.
const PULSE_W := 0.12
## Seconds a successful run's completed range stays after the finish signal (the head completes in
## the first 40 %, the second half fades). A labelled judgement; 0 under `reduce_motion`. Input is
## released at once, before any of it: the flourish is drawn with `mouse_filter` IGNORE and
## `is_blocking()` false.
const FINISH_S := 0.5
## Seconds a completed stage's ridge takes to rise (the "beat"). A labelled judgement.
const BEAT_S := 0.45
## Profile samples per stage segment. A labelled judgement: 24 x 10 = 240 vertices is smooth at
## the 520 px desktop bar (about 2 px per vertex) and cheap at phone width.
const SAMPLES_PER_STAGE := 24
## Midpoint-displacement roughness: each octave's jitter is this times the previous one. A labelled
## judgement: 0.55 gives a ridgeline (rough, not white noise). Cheap fBm, not geology.
const ROUGHNESS := 0.55
## Card geometry in reference px (multiplied by the phone scale on a phone). Judgements, matching
## the DCC modal's own scale (`DccWidgets.MODAL_W` family) and the shell's 16 px phone gutter.
const CARD_W := 560.0
const GUTTER := 16.0  ## Minimum side gap between card and viewport edge in reference px (a judgement, per the line above); multiplied by the phone scale.
const PAD := 20.0  ## Card inner padding in reference px (a judgement, per the line above); multiplied by the phone scale.
const BAR_H := 78.0  ## Relief-trace bar height in reference px (a judgement, per the line above); multiplied by the phone scale.
## Scrim opacity over the `bg` token. A labelled judgement: dark enough that the map clearly reads
## as "not available", light enough to see it is still there.
const SCRIM_ALPHA := 0.78
## The stage count assumed before the first `generation_stage` tick (the engine has ten,
## `progress.rs::STAGE_COUNT`); replaced by the signal's own `total` as soon as one arrives.
const DEFAULT_STAGES := 10

## The bridge whose `generating` flag and signals drive the engine source. Set by `setup`.
var bridge: Node
## Honoured if the shell grows a reduced-motion preference (none exists today, grepped
## 2026-10-11): no sweep, no travelling pulse, no fade, no ridge rise. Messages still rotate.
var reduce_motion := false
## The timing control for `_busyov_probe.gd` leg 8 ("does the overlay cost generate time"):
## `false` makes the overlay inert (never shown, never blocking). Nothing in the shell writes it.
var enabled := true

## Probe-visible counters. `swallowed` counts every event `_input` consumed;
## `watchdog_releases` counts every release the watchdog made (each also `push_warning`s).
var swallowed := 0
## Probe-visible: `swallowed` broken down by `InputEvent.get_class()`.
var swallowed_by_class: Dictionary = {}
## Probe-visible count of watchdog releases (each also calls `push_warning`).
var watchdog_releases := 0
## The trace's head as a fraction of the range, as last DRAWN (`-1` while indeterminate).
var drawn_extent := -1.0

## Busy-source nesting count (the HTML's `showBusy` counter); blocking and visible while above 0.
var _depth := 0
## True while the engine is holding exactly one `_depth` unit, so an unpaired finish cannot release another source's count.
var _engine_held := false
## The engine source's kind (`bridge.busy_kind`, or "generate" by default); empty when none is held.
var _engine_kind := ""
## The title line, set by `show_busy` for a fresh run or while no stages are reporting.
var _label := ""
## The current message category ("init" until a label or stage sets one).
var _category := "init"
## The line of wit currently drawn.
var _message := ""
## The tier of the current message entry ("cat" until `pick_entry` returns one); exposed by `message_tier()`.
var _tier := "cat"
## RNG for message picks; seeded from the clock in `_init` and carried across runs so no line repeats back to back. A probe re-seeds it.
var _msg_rng := RandomNumberGenerator.new()
## RNG for the trace's mountain-range profile (`_rebuild_profile`); seeded from the clock; a probe may re-seed it.
var _range_rng := RandomNumberGenerator.new()
## Seconds since the last message pick; a fresh line is picked at `ROTATE_S`.
var _since_rotate := 0.0
## Seconds since this run began; the clock the ridge-rise animation reads.
var _t := 0.0
## True once the engine reports stages (`generation_stage`); a generate starts in stage mode, other kinds stay indeterminate.
var _stage_mode := false
## Last stage index reported; -1 before any.
var _stage_index := -1
## Stage total, `DEFAULT_STAGES` until a signal's own `total` replaces it.
var _stage_count := DEFAULT_STAGES
## Last stage name reported; empty before any.
var _stage_name := ""
## Completed-stages fraction of the range, only ever raised; the trace's head eases toward it and never past it.
var _solid := 0.0
## The trace head's fraction of the range, eased by `_advance_head`; reset to 0 for each run.
var _head := 0.0
## The head's value when the finish flourish began; the flourish interpolates from it to 1.
var _finish_from := 0.0
## Per stage, the `_t` at which that stage completed (the ridge's rise start); -1.0 while not yet completed.
var _beats: PackedFloat32Array = PackedFloat32Array()
## The mountain-range height samples (`_stage_count * SAMPLES_PER_STAGE + 1`), 0.05 to 0.60 of the bar, rebuilt by `_rebuild_profile`.
var _profile: PackedFloat32Array = PackedFloat32Array()
## True while a successful run's completed range is still on screen (the flourish); input is already released.
var _finishing := false
## Seconds into the flourish; compared with `FINISH_S`.
var _finish_t := 0.0
## Consecutive frames of `bridge.generating == false` with the engine source held; the watchdog trips at `WATCHDOG_FRAMES`.
var _idle_frames := 0
## Consecutive frames a non-engine source has outlived an idle engine; the watchdog trips at `WATCHDOG_FRAMES`.
var _orphan_frames := 0

## Builds the inert overlay: hidden, full-rect, input-transparent until a source arrives. Seeds the
## message and range RNGs from the clock (a probe re-seeds `_msg_rng`/`_range_rng` for repeatability).
func _init() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	focus_mode = Control.FOCUS_NONE
	process_mode = Node.PROCESS_MODE_ALWAYS
	visible = false
	_msg_rng.seed = Time.get_ticks_usec()
	_range_rng.seed = Time.get_ticks_usec() ^ 0x5bd1e995

## Connects the engine source to `b`'s signals. Called once by `app.gd` after the bridge exists;
## a null bridge leaves the overlay usable through `show_busy`/`hide_busy` only.
func setup(b: Node) -> void:
	bridge = b
	if bridge == null:
		return
	bridge.generation_started.connect(_on_generation_started)
	bridge.generation_stage.connect(_on_stage)
	bridge.generation_finished.connect(func(ok: bool): _release_engine(ok))
	bridge.landmark_finished.connect(func(r: Dictionary): _release_engine(bool(r.get("ok", true))))
	bridge.landmark_refine_finished.connect(func(r: Dictionary): _release_engine(bool(r.get("ok", true))))

# -- Public API (the HTML's showBusy / hideBusy) ---------------------------------------------------

## `showBusy(label)`: one more busy source. The first one shows the overlay and starts blocking;
## a nested one updates the label and message unless a generation is reporting stages (the stage
## readout outranks a nested label). A no-op while `enabled` is false.
func show_busy(label: String = "working…") -> void:
	if not enabled:
		return
	_depth += 1
	var fresh := not visible or _finishing
	if fresh:
		_reset_for_run()
	if fresh or not _stage_mode:
		_label = label
		_set_category(LoadingMessages.category_for_label(label))
	_enter()

## `hideBusy()`: one source fewer, clamped at 0 (a stray hide is never owed later). At 0 the
## overlay stops blocking at once and hides.
func hide_busy() -> void:
	_depth = maxi(0, _depth - 1)
	if _depth == 0:
		_leave(false)

## Process frames `busy_run` waits after `show_busy` before it starts the work. Two, not one: a single
## `process_frame` await resumes before the frame that drew the change has reached the screen -- the
## shell's own measured rule, stated at `civilization_workspace.gd::_recompute_civ` and
## `_populate_world_now`, which used exactly this pair to paint a button label before blocking.
const PAINT_FRAMES := 2
## Process frames `busy_run` keeps blocking AFTER the work returns, before it hides. One: input the OS
## queued while the main thread was frozen is delivered at the start of the next loop iteration
## (`DisplayServer.process_events`, which flushes buffered input), before that iteration's
## `process_frame` -- so holding the overlay through exactly one more frame is what lets `_input`
## swallow the clicks and keys a user mashed during the freeze, instead of replaying them into the
## shell the instant it unfreezes (the gap STATUS.md's 2026-10-11 entry recorded as "input during
## them is queued and delivered after"). A labelled judgement on the count; the mechanism is
## Godot's main-loop order. `_busysync_probe.gd` models it with an event deferred out of the work.
const SETTLE_FRAMES := 1

## Emitted by `busy_run` immediately before / after it calls `work`, with the run's label. Probe
## hooks only (`_busysync_probe.gd` reads the overlay's state and the action's on-disk effect from
## inside the blocked call through them); nothing in the shell connects to them, and a listener must
## never call the engine (the work is about to hold, or has just released, the main thread).
signal work_started(label: String)
## Emitted by `busy_run` just after `work` returns (see `work_started` above).
signal work_finished(label: String)

## **Runs one SYNCHRONOUS main-thread job under the overlay** (save, open, export, bake, erosion
## pass, civilisation recompute): the try/finally GDScript does not have.
##
## **What.** `show_busy(label)` (one depth unit; the category is `label`'s, through
## `LoadingMessages.LABEL_HINT`), `PAINT_FRAMES` frames so the card is actually on screen, then
## `work.call()` -- which blocks the main thread, so the card stays frozen on screen for the whole
## call -- then `SETTLE_FRAMES` frames still blocking, then `hide_busy()`. Returns whatever `work`
## returned (null for an invalid `work`).
##
## **Why here and not at each call site.** A synchronous engine call inside a button handler blocks
## before anything can paint, so every site needs the same show / await / work / await / hide
## sequence; written out at each site the hide is the line an early `return` skips. Here `work`'s own
## early returns and failure branches only return from `work`, and a GDScript runtime error inside it
## aborts that call with a null result and resumes HERE -- so every exit path reaches the single
## `hide_busy()` below. The coroutine also lives on this node, so a caller freed mid-await (a dock
## rebuilt during the paint frames) cannot strand a held source.
##
## **Never.** Never pass `work` that awaits: it would return at its first `await` and the overlay
## would hide while it continued (nothing here can see that). Never call it from code that must have
## the result before it returns -- it is a coroutine; keep the synchronous function and give the
## button a wrapper that awaits this. Never use it for the generation thread's runs (those hold the
## ENGINE source through the bridge's signals) or for timer-fired work: its blocking would swallow
## the release of a drag already in progress.
func busy_run(label: String, work: Callable) -> Variant:
	show_busy(label)
	for i in PAINT_FRAMES:
		await get_tree().process_frame
	work_started.emit(label)
	var result: Variant = work.call() if work.is_valid() else null
	work_finished.emit(label)
	for i in SETTLE_FRAMES:
		await get_tree().process_frame
	hide_busy()
	return result

## True while input is being swallowed: some source is held and the overlay is enabled.
func is_blocking() -> bool:
	return enabled and _depth > 0

## The current nesting depth (probe-visible; never negative).
func depth() -> int:
	return _depth

## The line on screen now.
func message() -> String:
	return _message

## The category the current line was asked for (a stage category during a generation).
func message_category() -> String:
	return _category

## `"cat"`, `"rare"` or `"xrare"`: which tier the current line came from.
func message_tier() -> String:
	return _tier

## The engine stage index shown (`-1` before the first tick or for an unstaged run).
func stage_index() -> int:
	return _stage_index

## The engine's own name for the stage shown (empty before the first tick).
func stage_name() -> String:
	return _stage_name

## Completed stages as a fraction of the range: the SOLID trace. Monotonic within one run.
func solid_extent() -> float:
	return _solid

## What kind of engine run holds the overlay (`bridge.busy_kind` at acquire time), or "".
func engine_kind() -> String:
	return _engine_kind

## The laid-out geometry `_draw` uses, for layout probes: `card`, `bar`, `title_rect`, `stage_rect`,
## `message_rect`, `foot_rect` (Rect2, overlay-local), `scale`, `message_lines` and `inner_w`.
func layout() -> Dictionary:
	return _layout()

# -- Engine source ----------------------------------------------------------------------------------

## `generation_started`: a generate, a heightmap import or a region resample took the engine.
func _on_generation_started() -> void:
	_acquire_engine(_bridge_kind("generate"))

## `bridge.busy_kind`, or `fallback` on a bridge without the field or with it empty.
func _bridge_kind(fallback: String) -> String:
	if bridge == null:
		return fallback
	var k = bridge.get("busy_kind")
	return fallback if k == null or String(k) == "" else String(k)

## Takes the engine source (one depth unit, at most one at a time). A generate starts in STAGE mode
## (the engine reports stages); every other kind is indeterminate.
func _acquire_engine(kind: String) -> void:
	if _engine_held or not enabled:
		return
	_engine_held = true
	_engine_kind = kind
	_idle_frames = 0
	show_busy(_label_for_kind(kind))
	_stage_mode = kind == "generate"
	if _stage_mode:
		_set_category(LoadingMessages.stage_category(0, ""))

## Gives the engine source back. `ok` on a successful generate completes the range and lets it fade
## out (`FINISH_S`); a failure hides at once so the existing failure report (status bar, phone
## toast) is what the user sees. A no-op when the engine source is not held.
func _release_engine(ok: bool) -> void:
	if not _engine_held:
		return
	_engine_held = false
	_idle_frames = 0
	if ok and _stage_mode:
		_solid = 1.0
		for j in _beats.size():
			if _beats[j] < 0.0:
				_beats[j] = _t
	_depth = maxi(0, _depth - 1)
	if _depth == 0:
		_leave(ok and not reduce_motion)

## `generation_stage`: moves the readout to the engine's CURRENT stage and shows a fresh line from
## its category at once. Ignored when no generate holds the overlay, and for an index lower than the
## one shown (the solid trace never walks back). A jump marks every skipped stage complete.
func _on_stage(index: int, stage_name: String, total: int) -> void:
	if not _engine_held or not _stage_mode:
		return
	if index < _stage_index:
		return
	if total > 0 and total != _stage_count:
		_stage_count = total
		_rebuild_profile()
	for j in range(maxi(0, _stage_index), mini(index, _beats.size())):
		if _beats[j] < 0.0:
			_beats[j] = _t
	var moved := index != _stage_index
	_stage_index = index
	_stage_name = stage_name
	_solid = maxf(_solid, float(index) / float(maxi(1, _stage_count)))
	if moved:
		_set_category(LoadingMessages.stage_category(index, stage_name))

# -- Show / hide ------------------------------------------------------------------------------------

## A fresh run: new range, new stage state. The message RNG carries on (no-repeat spans runs).
func _reset_for_run() -> void:
	_finishing = false
	_finish_t = 0.0
	modulate.a = 1.0
	_t = 0.0
	_stage_mode = false
	_stage_index = -1
	_stage_name = ""
	_stage_count = DEFAULT_STAGES
	_solid = 0.0
	_head = 0.0
	drawn_extent = -1.0
	_rebuild_profile()

## Becomes visible and blocking, and moves the layer to the end of its parent so this `_input` runs
## before every other `_input` in the app (see the file header).
func _enter() -> void:
	visible = true
	mouse_filter = Control.MOUSE_FILTER_STOP
	var layer := get_parent()
	if layer != null and layer.get_parent() != null:
		layer.get_parent().move_child(layer, -1)
	_mute_subwindows()
	queue_redraw()

## **Embedded windows are the one hole `_input` cannot close.** A dialog or popup that is open when a
## run starts is an embedded sub-window: the viewport forwards events to it BEFORE any `_input`
## (`Viewport.push_input` -> `_sub_windows_forward`), and it is drawn above every `CanvasLayer`.
## Found by `_busyov_probe.gd` 2026-10-11: a "New settlement label" dialog left open stayed usable over
## the overlay, and its Create button writes the engine. So every visible embedded window gets
## `gui_disable_input` while blocking (re-checked every frame, for one that opens mid-run) and exactly
## those get it back on release. Hiding them instead was rejected: several dialogs treat a hide as a
## cancel and clean up (or call the engine) on it. They stay visible; they just stop listening.
var _muted_windows: Array = []

## Disables input on every visible embedded window that still accepts it, remembering which.
func _mute_subwindows() -> void:
	var vp := get_viewport()
	if vp == null:
		return
	for w in vp.get_embedded_subwindows():
		if w is Window and not (w as Window).gui_disable_input:
			(w as Window).gui_disable_input = true
			_muted_windows.append(w)

## Gives input back to exactly the windows `_mute_subwindows` took it from (freed ones are skipped).
func _unmute_subwindows() -> void:
	for w in _muted_windows:
		if is_instance_valid(w):
			(w as Window).gui_disable_input = false
	_muted_windows.clear()

## Stops blocking at once; hides now, or after the `FINISH_S` fade when `flourish`.
func _leave(flourish: bool) -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_unmute_subwindows()
	if flourish:
		_finishing = true
		_finish_t = 0.0
		_finish_from = _head
		queue_redraw()
		return
	_finishing = false
	modulate.a = 1.0
	visible = false

## Switches to `category` and draws a fresh line from it at once (the stage-change path).
func _set_category(category: String) -> void:
	_category = category
	_new_message()

## Draws a fresh line for the current category, never the previous one, and restarts the rotation.
func _new_message() -> void:
	var e: Dictionary = LoadingMessages.pick_entry(_category, _msg_rng, _message)
	_message = String(e["text"])
	_tier = String(e["tier"])
	_since_rotate = 0.0

## The HTML-style busy label for an engine kind; `category_for_label` maps it to its category.
static func _label_for_kind(kind: String) -> String:
	match kind:
		"generate": return "generating…"
		"import": return "importing heightmap…"
		"resample": return "resampling region…"
		"landmarks": return "placing landmarks…"
		"viewshed": return "refining viewshed…"
	return "working…"

# -- Frame loop: watchdog, rotation, animation -------------------------------------------------------

## Polls `bridge.generating` (acquire for unsignalled runs; the watchdog), rotates the line inside a
## long stage, advances the animation and the finish fade. Never touches the engine.
func _process(delta: float) -> void:
	if bridge != null and enabled:
		var busy := bool(bridge.generating)
		if busy and not _engine_held:
			_acquire_engine(_bridge_kind("landmarks"))
		elif not busy and _engine_held:
			_idle_frames += 1
			if _idle_frames >= WATCHDOG_FRAMES:
				push_warning("BusyOverlay watchdog: the engine has been idle for %d frames and no finish signal arrived -- releasing the UI." % _idle_frames)
				watchdog_releases += 1
				_release_engine(false)
		else:
			_idle_frames = 0
		if _depth > 0 and not _engine_held and not busy:
			_orphan_frames += 1
			if _orphan_frames >= WATCHDOG_FRAMES:
				push_warning("BusyOverlay watchdog: %d busy source(s) outlived the engine by %d frames -- releasing the UI." % [_depth, _orphan_frames])
				watchdog_releases += 1
				_depth = 0
				_leave(false)
		else:
			_orphan_frames = 0
	if not visible:
		return
	if is_blocking():
		_mute_subwindows()
	_t += delta
	_since_rotate += delta
	if _depth > 0 and _since_rotate >= ROTATE_S:
		_new_message()
	if _finishing:
		_finish_t += delta
		## The completed range sweeps to its end in the first 40 % of the flourish at full opacity,
		## holds, and only the second half fades -- so the whole range is seen drawn before it goes.
		_head = lerpf(_finish_from, 1.0, clampf(_finish_t / (FINISH_S * 0.4), 0.0, 1.0))
		modulate.a = clampf(2.0 - 2.0 * _finish_t / FINISH_S, 0.0, 1.0)
		if _finish_t >= FINISH_S:
			_leave(false)
			return
	else:
		_advance_head(delta)
	queue_redraw()

## Eases `_head` toward `_solid` (never past it, never back): at least `HEAD_RATE` of the range per
## second, faster when the engine jumped several stages. Snaps under `reduce_motion`.
func _advance_head(delta: float) -> void:
	if reduce_motion:
		_head = maxf(_head, _solid)
		return
	if _head < _solid:
		_head = minf(_solid, _head + maxf(HEAD_RATE * delta, (_solid - _head) * minf(1.0, 6.0 * delta)))

## Swallows EVERY input event while blocking (mouse, touch, wheel, drag, gestures, keys, actions),
## before GUI, shortcut and unhandled dispatch. Counts what it took for the probes.
func _input(event: InputEvent) -> void:
	if not is_blocking():
		return
	swallowed += 1
	var k := event.get_class()
	swallowed_by_class[k] = int(swallowed_by_class.get(k, 0)) + 1
	get_viewport().set_input_as_handled()

# -- The relief range --------------------------------------------------------------------------------

## A new seeded range for this run: 1D midpoint displacement (cheap fBm; a labelled judgement, not
## geology) on 2^8+1 points, normalised to 0.05..0.60 of the bar height, resampled to
## `SAMPLES_PER_STAGE` per stage. Also resets the per-stage beat clocks (`-1` = not yet completed).
func _rebuild_profile() -> void:
	var m := 257
	var h := PackedFloat32Array()
	h.resize(m)
	h[0] = _range_rng.randf_range(0.1, 0.3)
	h[m - 1] = _range_rng.randf_range(0.1, 0.3)
	var step := m - 1
	var amp := 0.5
	while step > 1:
		var half := step / 2
		for i in range(half, m, step):
			h[i] = (h[i - half] + h[i + half]) * 0.5 + _range_rng.randf_range(-amp, amp)
		amp *= ROUGHNESS
		step = half
	var lo := INF
	var hi := -INF
	for v in h:
		lo = minf(lo, v)
		hi = maxf(hi, v)
	var span := maxf(1e-6, hi - lo)
	var n := _stage_count * SAMPLES_PER_STAGE + 1
	_profile.resize(n)
	for i in n:
		var u := float(i) / float(n - 1) * float(m - 1)
		var i0 := mini(int(u), m - 2)
		var f := u - float(i0)
		var v := lerpf(h[i0], h[i0 + 1], f)
		_profile[i] = 0.05 + 0.55 * (v - lo) / span
	_beats.resize(_stage_count)
	for j in _stage_count:
		_beats[j] = -1.0

## Height (0..1 of the bar) at sample `i`: the base range plus every completed stage's ridge -- the
## "beat": a bump centred on that stage's segment, taller the later the stage (a crescendo), rising
## over `BEAT_S` from the moment the stage completed.
func _height_at(i: int) -> float:
	var n := _profile.size()
	var x := float(i) / float(maxi(1, n - 1))
	var y := _profile[i]
	var seg := 1.0 / float(maxi(1, _stage_count))
	for j in _beats.size():
		if _beats[j] < 0.0:
			continue
		var grow := 1.0 if reduce_motion else clampf((_t - _beats[j]) / BEAT_S, 0.0, 1.0)
		grow = grow * grow * (3.0 - 2.0 * grow)
		var c := (float(j) + 0.5) * seg
		var d := (x - c) / (seg * 0.22)
		y += grow * (0.16 + 0.26 * float(j + 1) / float(_stage_count)) * exp(-d * d)
	return clampf(y, 0.0, 1.0)

# -- Layout and drawing ---------------------------------------------------------------------------

## The phone composition's scale (`DccTheme.phone_scale()`), 1.0 elsewhere.
static func _scale() -> float:
	return DccTheme.phone_scale() if DccTheme.is_phone() else 1.0

## The title line: the busy label without its ellipsis, upper-cased.
func _title_text() -> String:
	return _label.replace("…", "").strip_edges().to_upper()

## "stage 04 of 10 · Tectonics" while a generate reports stages; "starting" before its first
## tick; "working -- this step reports no stages" for an unstaged run (the honest dash).
func _stage_text() -> String:
	if _stage_mode and _stage_index >= 0:
		return "stage %02d of %02d · %s" % [_stage_index + 1, _stage_count, _stage_name]
	if _stage_mode:
		return "starting · %02d stages" % _stage_count
	return "working · this step reports no stages"

## The foot line: what is paused and why, and that there is no cancel.
const FOOT := "Input is paused while the engine has the world. A run can't be cancelled once it starts; it ends on its own."

## Computes every rect `_draw` uses (see `layout()`).
func _layout() -> Dictionary:
	var s := _scale()
	var vs := size
	var gutter := GUTTER * s
	var card_w := minf(CARD_W * s, maxf(1.0, vs.x - 2.0 * gutter))
	var pad := PAD * s
	var inner_w := maxf(1.0, card_w - 2.0 * pad)
	var mono := DccTheme.mono(1)
	var body := get_theme_default_font()
	var title_fs := int(round(10.0 * s))
	var stage_fs := int(round(11.0 * s))
	var msg_fs := int(round(17.0 * s))
	var foot_fs := int(round(10.0 * s))
	var title_h := mono.get_height(title_fs)
	var bar_h := BAR_H * s
	var stage_h := mono.get_multiline_string_size(_stage_text(), HORIZONTAL_ALIGNMENT_LEFT, inner_w, stage_fs).y
	var msg_sz := body.get_multiline_string_size(_message, HORIZONTAL_ALIGNMENT_LEFT, inner_w, msg_fs)
	var foot_sz := mono.get_multiline_string_size(FOOT, HORIZONTAL_ALIGNMENT_LEFT, inner_w, foot_fs)
	var gap := 12.0 * s
	var card_h := pad * 2.0 + title_h + gap + bar_h + gap + stage_h + gap * 1.4 + msg_sz.y + gap * 1.4 + foot_sz.y
	var card := Rect2(Vector2((vs.x - card_w) * 0.5, maxf(gutter, (vs.y - card_h) * 0.5)), Vector2(card_w, card_h))
	var x := card.position.x + pad
	var y := card.position.y + pad
	var title_rect := Rect2(x, y, inner_w, title_h)
	y += title_h + gap
	var bar := Rect2(x, y, inner_w, bar_h)
	y += bar_h + gap
	var stage_rect := Rect2(x, y, inner_w, stage_h)
	y += stage_h + gap * 1.4
	var msg_rect := Rect2(x, y, inner_w, msg_sz.y)
	y += msg_sz.y + gap * 1.4
	var foot_rect := Rect2(x, y, inner_w, foot_sz.y)
	return {
		"scale": s, "card": card, "bar": bar, "inner_w": inner_w,
		"title_rect": title_rect, "stage_rect": stage_rect, "message_rect": msg_rect, "foot_rect": foot_rect,
		"message_lines": int(round(msg_sz.y / maxf(1.0, body.get_height(msg_fs)))),
		"fonts": {"title": title_fs, "stage": stage_fs, "message": msg_fs, "foot": foot_fs},
	}

## Draws the scrim, the card, the title, the relief trace, the stage line, the message and the foot.
## Colours are read from `DccTheme` on every draw, so a palette flip lands on the next frame.
func _draw() -> void:
	var L := _layout()
	var s: float = L["scale"]
	draw_rect(Rect2(Vector2.ZERO, size), Color(DccTheme.c("bg"), SCRIM_ALPHA))
	var card: Rect2 = L["card"]
	draw_rect(card, DccTheme.c("panel"))
	draw_rect(card, DccTheme.c("border"), false, maxf(1.0, s))
	var mono := DccTheme.mono(1)
	var mono_med := DccTheme.mono(1, true)
	var body := get_theme_default_font()
	var fonts: Dictionary = L["fonts"]
	var tr: Rect2 = L["title_rect"]
	draw_string(DccTheme.mono(2), Vector2(tr.position.x, tr.position.y + mono.get_ascent(fonts["title"])),
		_title_text(), HORIZONTAL_ALIGNMENT_LEFT, tr.size.x, fonts["title"], DccTheme.c("text_faint"))
	_draw_relief(L["bar"], s)
	var sr: Rect2 = L["stage_rect"]
	draw_multiline_string(mono_med, Vector2(sr.position.x, sr.position.y + mono_med.get_ascent(fonts["stage"])),
		_stage_text(), HORIZONTAL_ALIGNMENT_LEFT, sr.size.x, fonts["stage"], -1, DccTheme.c("accent"))
	var mr: Rect2 = L["message_rect"]
	draw_multiline_string(body, Vector2(mr.position.x, mr.position.y + body.get_ascent(fonts["message"])),
		_message, HORIZONTAL_ALIGNMENT_LEFT, mr.size.x, fonts["message"], -1, DccTheme.c("text_bright"))
	var fr: Rect2 = L["foot_rect"]
	draw_multiline_string(mono, Vector2(fr.position.x, fr.position.y + mono.get_ascent(fonts["foot"])),
		FOOT, HORIZONTAL_ALIGNMENT_LEFT, fr.size.x, fonts["foot"], -1, DccTheme.c("text_ghost"))

## The relief trace inside `r`: level line, stage ticks, the range ahead (faint), the swept trace
## (older part fading behind the pen), its accent-wash fill, and the pen. Indeterminate runs get the
## whole range faint with a travelling pulse instead. Records `drawn_extent`.
func _draw_relief(r: Rect2, s: float) -> void:
	var n := _profile.size()
	if n < 2:
		return
	var tick_h := 12.0 * s
	var base_y := r.end.y - tick_h
	var top := r.position.y + 2.0 * s
	var amp := base_y - top
	var count := maxi(1, _stage_count)
	# Gridlines and the dashed level line -- `section_strip.gd`'s vocabulary.
	for g in [1.0 / 3.0, 2.0 / 3.0]:
		var gy: float = base_y - amp * g
		draw_line(Vector2(r.position.x, gy), Vector2(r.end.x, gy), DccTheme.c("line_soft"), 1.0)
	draw_dashed_line(Vector2(r.position.x, base_y), Vector2(r.end.x, base_y), DccTheme.c("text_faint"), maxf(1.0, s), 5.0 * s)
	var pts := PackedVector2Array()
	pts.resize(n)
	for i in n:
		pts[i] = Vector2(r.position.x + r.size.x * float(i) / float(n - 1), base_y - amp * _height_at(i))
	# Stage ticks along the baseline, completed ones in accent.
	var done_ticks := int(round(_solid * count)) if _stage_mode else -1
	for k in count + 1:
		var tx := r.position.x + r.size.x * float(k) / float(count)
		var tc := DccTheme.c("accent_dim") if k <= done_ticks else DccTheme.c("text_ghost")
		draw_line(Vector2(tx, base_y), Vector2(tx, base_y + 5.0 * s), tc, maxf(1.0, s))
	var width := 1.6 * s
	if not _stage_mode:
		_draw_pulse(pts, base_y, width, s)
		return
	var head := _head
	drawn_extent = head
	var hi := clampi(int(round(head * float(n - 1))), 0, n - 1)
	# The range still ahead: faint, so the user sees what is left to cross.
	if hi < n - 1:
		draw_polyline(pts.slice(hi), DccTheme.c("line_soft"), maxf(1.0, s), true)
	if hi < 1:
		draw_circle(pts[0], 3.0 * s, DccTheme.c("accent"), true, -1.0, true)
		return
	var trace := pts.slice(0, hi + 1)
	var fill := trace.duplicate()
	fill.append(Vector2(trace[trace.size() - 1].x, base_y))
	fill.append(Vector2(trace[0].x, base_y))
	draw_colored_polygon(fill, DccTheme.c("accent_wash"))
	# Older trace fades behind the pen (full ink over the last 35 % of the range, 30 % before it),
	# and the heartbeat: a glint that runs from the start of the trace to the pen every `SWEEP_S`.
	var acc := DccTheme.c("accent")
	var glint := -1.0 if reduce_motion else fmod(_t, SWEEP_S) / SWEEP_S * head
	var cols := PackedColorArray()
	cols.resize(trace.size())
	for i in trace.size():
		var x := float(i) / float(n - 1)
		var a := 0.3 + 0.7 * clampf(1.0 - (head - x) / 0.35, 0.0, 1.0)
		if glint >= 0.0:
			var d := (x - glint) / 0.04
			a = maxf(a, exp(-d * d))
		cols[i] = Color(acc, a)
	draw_polyline_colors(trace, cols, width, true)
	var pen := trace[trace.size() - 1]
	var glow := 7.0 * s * (1.0 if reduce_motion else (0.85 + 0.15 * sin(_t * TAU / SWEEP_S)))
	draw_circle(pen, glow, Color(acc, 0.22), true, -1.0, true)
	draw_circle(pen, 3.0 * s, acc, true, -1.0, true)

## The indeterminate trace: the whole range faint, a bright window travelling along it (static and
## evenly inked under `reduce_motion`).
func _draw_pulse(pts: PackedVector2Array, base_y: float, width: float, s: float) -> void:
	var n := pts.size()
	var acc := DccTheme.c("accent")
	drawn_extent = -1.0
	if reduce_motion:
		draw_polyline(pts, Color(acc, 0.6), width, true)
		return
	var p := fmod(_t, PULSE_S) / PULSE_S * (1.0 + 2.0 * PULSE_W) - PULSE_W
	var cols := PackedColorArray()
	cols.resize(n)
	for i in n:
		var x := float(i) / float(n - 1)
		var d := (x - p) / PULSE_W
		cols[i] = Color(acc, 0.22 + 0.78 * exp(-d * d))
	draw_polyline_colors(pts, cols, width, true)
	var pi := clampi(int(round(p * float(n - 1))), 0, n - 1)
	if p >= 0.0 and p <= 1.0:
		draw_circle(pts[pi], 6.0 * s, Color(acc, 0.22), true, -1.0, true)
		draw_circle(pts[pi], 3.0 * s, acc, true, -1.0, true)

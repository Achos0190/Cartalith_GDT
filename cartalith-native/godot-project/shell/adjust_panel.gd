extends VBoxContainer
## ET-C4 (`EDITING_TOOLS_SCOPE.md` section 5; Ruling BX item 4): the per-layer adjustment
## stack panel. One ordered, toggleable list of adjustments for each of three layers --
## Terrain, Water (both run in Rust inside `finish_rgb`, ET-C2) and Labels & overlays (the
## GDScript port, ET-C3, `shell/overlay_adjust.gd`) -- with add, enable, opacity, reorder,
## delete and a parameter editor per kind (Levels, Curves, Hue/Saturation/Lightness,
## Gradient map), a "Reset layer" action and plain-language notices.
##
## No `class_name` on purpose: `render_workspace.gd` loads it with
## `preload("res://shell/adjust_panel.gd")` and calls `.new()`, so registering it never needs
## the `godot --import` pass that strips `project.godot`'s `;` comment block (CLAUDE.md, last
## working rule), the same precedent as `look_history.gd` and `ramp_bar.gd`.
##
## # What this panel is, and is not
##
## **A view and an editor over state it does not own.** The engine owns the two engine stacks
## (`appearance_get_adjust` / `appearance_set_adjust`) and `overlay_adjust.gd` owns the third;
## `_doc` is only a parsed copy of the layer on show, re-read after any refusal, undo, world
## change or look load. It must never become the source of truth: a refused write leaves the
## owner unchanged and the copy is then re-synced FROM the owner, so the widgets can never keep
## showing a stack the renderer does not hold.
##
## # Rules this file is built around (each is a way to get it wrong)
##
## 1. **An invalid value is never sent.** Every slider's range is the engine's own validated
##    range (`crates/cartalith-godot/src/adjust.rs::AdjustKind::validate`, mirrored in the
##    constants below), and the cross-field rules (`in_black < in_white`, curve `x` strictly
##    increasing) are held by narrowing the neighbouring sliders' limits as one moves, so the
##    out-of-range value cannot be entered at all. If the owner still refuses, its error text
##    is shown VERBATIM (`_notice`) and nothing is clamped to make it pass.
## 2. **Never write a stack the user has not asked for.** Nothing here writes on build, sync,
##    world change or look load; only a gesture writes. An untouched project therefore keeps
##    `Option::None` stacks and its `appearance.json` byte-identical.
## 3. **Never touch the engine while it generates.** `EngineBridge.adjust_get/adjust_set` are
##    gated on `generating`. While a generate runs, EVERY layer's controls are DISABLED
##    (`_locked`), not queued: a queued edit would silently apply to a world that did not exist
##    when it was made, and refusing with a visible message is the honest option (decided here,
##    ET-C4). The overlay layer is locked too although its stack never reaches the engine,
##    because an edit ends in a look-history commit and that commit's listeners
##    (`menus.gd::refresh_undo_rows`, which calls `EngineBridge.undo_stats()` unguarded) DO call
##    the engine: measured as a `Gd<T>::bind() failed` panic in the first probe run that edited
##    the overlay layer mid-generate. `_write_layer` refuses for the same reason, so a caller
##    that bypasses the disabled widgets still never commits.
## 4. **One look-history step per gesture** (Ruling BY item 2): a slider drag, a ramp-bar
##    drag, a colour dialog and a button press are one entry each. This file owns no history;
##    it brackets writes with the host's `look_begin` / `look_commit` callables.
## 5. **Never free a live colour sample.** Only structural changes (add, delete, reorder,
##    reset, undo, layer switch) rebuild the list, and those cannot happen while the gradient
##    map's colour popup is open. A live colour tick (`final == false`) never rebuilds
##    (MISTAKES.md: a dock rebuild during a live colour sample frees the picker).
## 6. **Controls on touch are at least 44 px on BOTH axes**, floored at the call site because
##    this list is rebuilt after `phone_fit()` / `tablet_fit()` have already run once
##    (MISTAKES.md: a rebuild creates controls that escape both fitters).
##
## # The Look entry (ET-F1, `EDITING_TOOLS_SCOPE.md` section 7)
##
## The Look sliders (`render_workspace.gd`, section "Look") write ONE Hue/Sat/Light entry
## pinned at the top of each selected layer's stack. It is an ordinary entry -- the saved form
## has no field to tag it with (`adjust.rs::Adjustment` and `overlay_adjust.gd::_parse` both
## drop unknown keys), so it is identified by POSITION and KIND alone: **the Look entry is
## entry 0 when entry 0 is `hsl`** (`is_look_entry()`, the one definition both files read).
## This panel therefore lists it once, as row 1 tagged "Look", still editable (one state, two
## views: an edit here emits `stack_written`, and the Look sliders re-read), and keeps it on
## top: its Down and the next row's Up are disabled, so no other entry can be moved above it.
## Consequence, stated rather than hidden: a Hue/Sat/Light added here to an EMPTY layer is
## entry 0 and so is that layer's Look entry -- which is exactly what it does to the picture.

## Emitted after every successful write this panel makes (a gesture tick, a structural edit),
## with the layer id. The Look sliders listen so they never show a stack the owner no longer
## holds. Not emitted by `write_layer()` (the Look sliders' own path), so it cannot loop.
signal stack_written(layer: String)

const _OA := preload("res://shell/overlay_adjust.gd")
const _RampBar := preload("res://shell/ramp_bar.gd")

## The id of the labels/overlays layer. The two engine layers' ids come from
## `EngineBridge.adjust_layers()` (`list_adjust_layers`), never from a second copy here.
const OVERLAY := "overlay"
## Display names, keyed by layer id. An id the engine adds later falls back to its own spelling.
const LAYER_LABELS := {"terrain": "Terrain", "water": "Water", "overlay": "Labels & overlays"}
## Adjustment kinds in the engine's saved-form spelling (`AdjustKind`, snake_case), in menu order.
const KIND_ORDER := ["levels", "curves", "hsl", "gradient_map"]
## Button captions for adding a kind (the row's own title is `KIND_SHORT`).
const KIND_ADD := {"levels": "Levels", "curves": "Curves", "hsl": "Hue / Sat / Light", "gradient_map": "Gradient map"}
const KIND_SHORT := {"levels": "Levels", "curves": "Curves", "hsl": "Hue / Sat / Light", "gradient_map": "Gradient map"}
const KIND_TIP := {
	"levels": "Levels: map an input range to an output range with a midtone gamma, per channel. Gamma above 1 brightens the midtones.",
	"curves": "Curves: a tone curve through control points (input to output, 0 to 255), smoothed so it never overshoots. Applied to each channel.",
	"hsl": "Hue / Saturation / Lightness in OkLCh: hue rotates without shifting perceived lightness; saturation scales chroma; lightness lifts or lowers.",
	"gradient_map": "Gradient map: replace each colour by this gradient sampled at its brightness. Drag stops, click the bar to add one, drag one off to delete it.",
}

## Most entries a layer's stack may hold: `adjust.rs::MAX_ENTRIES`, a labelled judgement there.
const MAX_ENTRIES := 32
## Fewest and most Curves control points: `adjust.rs::MIN_CURVE_POINTS` / `MAX_CURVE_POINTS`.
const MIN_CURVE_POINTS := 2
const MAX_CURVE_POINTS := 16
## Levels gamma range, `adjust.rs::GAMMA_RANGE` (a labelled judgement there).
const GAMMA_MIN := 0.1
const GAMMA_MAX := 10.0
## Hue rotation range in degrees, `adjust.rs::HUE_RANGE_DEG`.
const HUE_MAX := 360.0
## Byte scale of Levels and Curves, `adjust.rs` (`in_range(.., 0.0, 255.0)`).
const BYTE_MAX := 255.0
## The blend modes a gradient map's ramp takes, `render::ElevationRamp`'s own three
## (`overlay_adjust.gd::_read_params` refuses any other).
const RAMP_MODES := ["Linear", "Ease", "Step"]

## Minimum milliseconds between live writes during a drag: the ramp bar's own `LIVE_MS`
## (a labelled judgement there: about 30 Hz). The release always writes, unthrottled.
const LIVE_MS := 33
## The shell's touch floor (`DccTheme` / `ramp_bar.gd::TOUCH_PX`), both axes.
const TOUCH_PX := 44.0

var app: Node
var bridge: Node
## `Callable(parts: Array)` / `Callable(label: String, parts: Array)`: the host's look-history
## bracket (`RenderWorkspace.look_begin` / `look_commit`).
var _look_begin: Callable
var _look_commit: Callable
## `Callable(layer: String)`: the host marks the look Custom and re-renders the map. Called
## after every successful ENGINE write (an overlay write redraws itself).
var _edited: Callable

## The layer on show: an engine layer id or `OVERLAY`.
var _layer := "terrain"
var _layer_ids: Array = []
## Parsed saved form of the layer on show, `{"entries": [...]}`. See the header: a copy.
var _doc: Dictionary = {"entries": []}
## True while the layer on show cannot be read (an engine layer during a generate).
var _unknown := false
## True while a generate runs: every layer's controls are disabled (rule 3).
var _locked := false
## A gesture has WRITTEN since `_begin_gesture()`. Not "a gesture is open": see `_begin_gesture`.
var _gest_dirty := false
var _gest_label := "Adjustment"
## A slider drag is in progress (its release closes the gesture).
var _dragging := false
var _pending := false
var _last_write := 0
var _rebuild_queued := false

var _layer_pick: OptionButton
var _list: VBoxContainer
var _add_host: HFlowContainer
var _reset_btn: Button
var _notice: Label
var _guard: Label
## Says, on the Labels & overlays layer only, when this build's project file cannot store that stack.
var _persist: Label


## Build the panel. `begin`/`commit`/`edited` are the host's callables (see the fields).
## Called once, after `add_child`, with a live `EngineBridge`.
func setup(app_ref: Node, bridge_ref: Node, begin: Callable, commit: Callable, edited: Callable) -> void:
	app = app_ref
	bridge = bridge_ref
	_look_begin = begin
	_look_commit = commit
	_edited = edited
	add_theme_constant_override("separation", 4)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	set_process(false)
	_layer_ids = bridge.adjust_layers() if bridge.adjust_api else []
	_layer_ids.append(OVERLAY)
	var names: Array = []
	for id in _layer_ids:
		names.append(String(LAYER_LABELS.get(id, String(id).capitalize())))
	_layer = String(_layer_ids[0])
	_layer_pick = DccWidgets.choice(self, "Layer", names, 0, _on_layer_picked,
		"Which layer the adjustments below act on. Terrain and Water act on the rendered map; "
		+ "Labels & overlays acts on roads, labels, icons, journeys, faction colours and borders.")
	_layer_pick.name = "AdjLayerPick"
	_floor_touch(_layer_pick)
	_list = VBoxContainer.new()
	_list.name = "AdjList"
	_list.add_theme_constant_override("separation", 8)
	add_child(_list)
	_add_host = HFlowContainer.new()
	_add_host.name = "AdjAddRow"
	_add_host.add_theme_constant_override("h_separation", 6)
	_add_host.add_theme_constant_override("v_separation", 6)
	add_child(_add_host)
	for kind in KIND_ORDER:
		var k := String(kind)
		var b := _action(_add_host, "+ " + String(KIND_ADD[k]), func(): _on_add(k), String(KIND_TIP[k]))
		b.name = "AdjAdd_" + k
	_reset_btn = _action(self, "Reset layer", _on_reset,
		"Remove every adjustment on this layer. One undo step.")
	_reset_btn.name = "AdjReset"
	_notice = DccWidgets.note(self, "")
	_notice.name = "AdjNotice"
	_notice.visible = false
	_guard = DccWidgets.note(self, "")
	_guard.name = "AdjGuard"
	_guard.visible = false
	_persist = DccWidgets.note(self, "")
	_persist.name = "AdjPersist"
	_persist.visible = false
	DccWidgets.note(self,
		"Adjustments never change the map's data or the colours you set: they are applied to "
		+ "the picture, in order, after the colour grade. Terrain and Water are saved with the "
		+ "project. Undo (Ctrl+Z) steps one gesture at a time.")
	if bridge.has_signal("generation_started"):
		bridge.generation_started.connect(_on_generation_started)
		bridge.generation_finished.connect(_on_generation_finished)
	sync()


## The layer ids this panel offers, in picker order. Read by the probe.
func layer_ids() -> Array:
	return _layer_ids.duplicate()


## The layer on show. Read by the probe.
func current_layer() -> String:
	return _layer


## Select a layer programmatically (the same path as the picker). Used by `on_look_restored`
## callers and the probe.
func select_layer(id: String) -> void:
	var i := _layer_ids.find(id)
	if i < 0:
		return
	_layer_pick.select(i)
	_on_layer_picked(i)


## Re-read the layer on show from its owner and rebuild the list. Never writes. Safe at any
## time, including during a generate (an engine layer is then shown as unknown).
func sync() -> void:
	_reload()
	_rebuild()


# -- reading and writing the owner -------------------------------------------------------

## Parse the layer's saved form into `_doc`. `_unknown` is set (and `_doc` emptied) when an
## engine layer cannot be read: an unreadable stack is NOT shown as an empty one.
func _reload() -> void:
	_unknown = false
	var d = _read_layer(_layer)
	if d == null:
		_unknown = true
		_doc = {"entries": []}
		return
	_doc = d


## The layer's stack as a parsed Dictionary, or `null` when it cannot be read (engine layer
## while generating, binding absent, or an unparseable document).
func _read_layer(layer: String) -> Variant:
	var text := ""
	if layer == OVERLAY:
		text = _OA.get_stack_json()
	else:
		var g: Dictionary = bridge.adjust_get(layer)
		if not bool(g.get("ok", false)) or not g.has("stack"):
			return null
		text = String(g["stack"])
	var p := JSON.new()
	if p.parse(text) != OK or not (p.data is Dictionary):
		return null
	var d: Dictionary = p.data
	if not (d.get("entries") is Array):
		d["entries"] = []
	return d


## Write `doc` to `layer`'s owner: `{ok: true, ...}` or `{ok: false, error}` (the owner's own
## text). The overlay goes through `ViewportHost.set_overlay_adjust`, which also re-applies the
## province-border shader; an engine layer goes through the generating-gated bridge wrapper.
func _write_layer(layer: String, doc: Dictionary) -> Dictionary:
	## Refused for EVERY layer while a generate runs (rule 3): the widgets are disabled, this is
	## the net under a caller that reaches `_apply()` anyway, and it is why no history entry is
	## ever committed mid-generate.
	if bridge.generating:
		return {"ok": false, "error": "the world is generating; try again when it finishes"}
	var text := JSON.stringify(doc)
	if layer == OVERLAY:
		if app == null or app.get("viewport") == null:
			return {"ok": false, "error": "no map overlay to hold the stack"}
		var r: Dictionary = app.viewport.set_overlay_adjust(text)
		if bool(r.get("ok", false)):
			bridge.mark_world_dirty()
		return r
	return bridge.adjust_set(layer, text)


## Push the current `_doc` to the owner now. On success tell the host (map refresh) and refresh
## the guard notice; on refusal show the owner's text, re-sync the copy from the owner and
## rebuild the widgets (deferred: this can run inside one of their own handlers).
func _apply() -> bool:
	_pending = false
	_last_write = Time.get_ticks_msec()
	var r := _write_layer(_layer, _doc)
	if not bool(r.get("ok", false)):
		_show_notice("Refused, nothing changed: %s" % String(r.get("error", "unknown reason")))
		_reload()
		_queue_rebuild()
		return false
	_gest_dirty = true
	_clear_notice()
	if _layer != OVERLAY and _edited.is_valid():
		_edited.call(_layer)
	_refresh_guard()
	stack_written.emit(_layer)
	return true


## True when `entries` (a stack's saved-form entry list) holds a Look entry: entry 0 is a
## Hue/Sat/Light. The single definition of the ET-F1 convention (see the header): position and
## kind, because the saved form has nowhere to carry a tag.
static func is_look_entry(entries: Array) -> bool:
	if entries.is_empty() or not (entries[0] is Dictionary):
		return false
	var k = (entries[0] as Dictionary).get("kind")
	return k is Dictionary and (k as Dictionary).has("hsl")


## Public read of any layer's stack for the Look sliders: the panel's own `_read_layer` (parsed
## `{"entries": [...]}`, or `null` when it cannot be read -- never an empty stack in its place).
func read_layer(layer: String) -> Variant:
	return _read_layer(layer)


## Public write for the Look sliders: the panel's own generating-gated `_write_layer`, so the
## Look writes the engine and the overlay by exactly the path a panel edit takes. Does NOT emit
## `stack_written`, re-sync or touch history: the caller brackets and refreshes.
func write_layer(layer: String, doc: Dictionary) -> Dictionary:
	return _write_layer(layer, doc)


## A parameter changed under a drag or a keypress: record the gesture's start, then write now
## or, inside the throttle window, schedule the remainder for `_process`.
func _changed_live(label: String) -> void:
	_gest_label = label
	_begin_gesture()
	if Time.get_ticks_msec() - _last_write >= LIVE_MS:
		_apply()
	else:
		_pending = true
		set_process(true)
	if not _dragging:
		call_deferred("_end_idle")


## Flush a throttled write once the interval has passed (the ramp bar's own pattern).
func _process(_d: float) -> void:
	if not _pending:
		set_process(false)
		return
	if Time.get_ticks_msec() - _last_write >= LIVE_MS:
		_apply()
		set_process(false)


## Deferred close of a gesture that no drag will end (a keypress, a click-less value change).
func _end_idle() -> void:
	if _gest_dirty or _pending:
		if not _dragging:
			_end_gesture(_gest_label)


# -- look history ------------------------------------------------------------------------

## The look-history part this layer's snapshots use (`look_history.gd`).
func _parts() -> Array:
	return ["adjust_" + _layer]


## Snapshot BEFORE the first write of a gesture. Re-snapshots unless the gesture has already
## written: a press that never wrote (a ramp-bar click with no movement, a slider grabbed and
## released unchanged) leaves a stale pending snapshot in the history, and `begin_gesture()`
## replaces it, so the next real edit is never paired with an old "before".
func _begin_gesture() -> void:
	if _gest_dirty:
		return
	if _look_begin.is_valid():
		_look_begin.call(_parts())


## Close the gesture: flush any throttled write, then push ONE entry (the history skips it when
## the data did not change, so a grabbed-and-released slider pushes nothing).
func _end_gesture(label: String) -> void:
	if _pending:
		_apply()
	var wrote := _gest_dirty
	_gest_dirty = false
	_dragging = false
	if wrote and _look_commit.is_valid():
		_look_commit.call(label, _parts())


## A discrete, one-step edit: snapshot, mutate the copy, write, close the gesture, rebuild.
func _structural(label: String, mutate: Callable) -> void:
	_end_gesture(_gest_label)
	_gest_label = label
	_begin_gesture()
	mutate.call()
	_apply()
	_end_gesture(label)
	_queue_rebuild()


# -- structural actions ------------------------------------------------------------------

## The entries array of the parsed copy `_doc` (live reference: editors append and remove in place).
func _entries() -> Array:
	return _doc["entries"]


## The saved-form kind name of entry `e` (`levels`, `curves`, `hsl`, `gradient_map`): the one key of its externally tagged `kind` object.
func _kind_of(e: Dictionary) -> String:
	return String((e["kind"] as Dictionary).keys()[0])


## The kind's parameter body of entry `e` (the dictionary the editors mutate in place).
func _body(e: Dictionary) -> Dictionary:
	return (e["kind"] as Dictionary)[_kind_of(e)]


## A fresh entry of `kind`, enabled at full opacity. Levels, Curves and Hue/Sat/Light start at
## their NEUTRAL parameters (skipped by control flow in the renderer, so adding one changes
## nothing until it is edited); a gradient map is never neutral, so it starts as a plain
## black-to-white ramp (a greyscale map) -- the least surprising non-neutral default.
static func new_entry(kind: String) -> Dictionary:
	var body := {}
	match kind:
		"levels":
			body = {"in_black": 0.0, "in_white": BYTE_MAX, "gamma": 1.0, "out_black": 0.0, "out_white": BYTE_MAX}
		"curves":
			body = {"points": [[0.0, 0.0], [BYTE_MAX, BYTE_MAX]]}
		"hsl":
			body = {"hue_deg": 0.0, "saturation": 0.0, "lightness": 0.0}
		"gradient_map":
			body = {"ramp": {"stops": [
				{"at": 0.0, "col": [0.0, 0.0, 0.0], "a": 1.0},
				{"at": 1.0, "col": [BYTE_MAX, BYTE_MAX, BYTE_MAX], "a": 1.0}], "mode": "Linear"}}
	return {"kind": {kind: body}, "enabled": true, "opacity": 1.0}


## Add button handler: append a fresh `kind` entry (one history step). Refused with a notice at `MAX_ENTRIES`, never silently dropped.
func _on_add(kind: String) -> void:
	if _entries().size() >= MAX_ENTRIES:
		_show_notice("A layer holds at most %d adjustments." % MAX_ENTRIES)
		return
	_structural("%s: add %s" % [_layer_label(), String(KIND_SHORT[kind])],
		func(): _entries().append(new_entry(kind)))


## Reset layer: empty the stack (one history step). Does nothing on an already empty layer, so it pushes no no-op entry.
func _on_reset() -> void:
	if _entries().is_empty():
		return
	_structural("%s: reset adjustments" % _layer_label(), func(): _doc = {"entries": []})


## Delete button handler for entry `i` (one history step).
func _on_delete(i: int) -> void:
	_structural("%s: remove %s" % [_layer_label(), String(KIND_SHORT[_kind_of(_entries()[i])])],
		func(): _entries().remove_at(i))


## Swap entry `i` with its neighbour `i + dir` (-1 up, +1 down). Order is the order applied.
func _on_move(i: int, dir: int) -> void:
	var j := i + dir
	if j < 0 or j >= _entries().size():
		return
	## ET-F1: nothing swaps with a pinned Look entry (the buttons are disabled; this is the net).
	if is_look_entry(_entries()) and (i == 0 or j == 0):
		return
	_structural("%s: reorder adjustments" % _layer_label(), func():
		var e: Array = _entries()
		var t = e[i]
		e[i] = e[j]
		e[j] = t)


## Enable checkbox handler for entry `i`: a disabled entry is kept but skipped by the renderer (one history step).
func _on_toggle(i: int, on: bool) -> void:
	_structural("%s: %s %s" % [_layer_label(), "enable" if on else "disable", String(KIND_SHORT[_kind_of(_entries()[i])])],
		func(): _entries()[i]["enabled"] = on)


## Layer picker handler: close any open gesture on the old layer, then show the new one read fresh from its owner. Never writes.
func _on_layer_picked(idx: int) -> void:
	_end_gesture(_gest_label)
	_layer = String(_layer_ids[idx])
	_clear_notice()
	sync()


## Display name of the layer on show, for history labels and messages.
func _layer_label() -> String:
	return String(LAYER_LABELS.get(_layer, _layer.capitalize()))


# -- generation lock ---------------------------------------------------------------------

## A generate started: the worker owns the engine, so every layer's controls are disabled
## (rule 3, including why the overlay layer is not exempt).
func _on_generation_started() -> void:
	_end_gesture(_gest_label)
	_locked = true
	_apply_lock()
	_show_notice("Paused while the world generates. Adjustments are available again when it finishes.")


## The generate finished: read the layer again (a new world may carry different stacks).
func _on_generation_finished(_ok: bool) -> void:
	_locked = false
	_clear_notice()
	sync()


## Disable (or leave enabled) every control that edits a stack. Called after every rebuild and
## on a generate start; a rebuild at the end of a generate restores the states.
func _apply_lock() -> void:
	var off := _locked
	_walk_lock(_list, off)
	_walk_lock(_add_host, off)
	_reset_btn.disabled = off or _entries().is_empty()


## Recursively set every button disabled / every slider read-only (`off`), except the buttons that are disabled for another reason (meta `adj_off`: Up on the first entry, Down on the last, a full add row).
func _walk_lock(n: Node, off: bool) -> void:
	for c in n.get_children():
		if c is BaseButton:
			(c as BaseButton).disabled = off or (c as BaseButton).has_meta("adj_off")
		elif c is Range:
			(c as Range).editable = not off
		_walk_lock(c, off)


# -- building the list -------------------------------------------------------------------

## Rebuild the list at the end of this frame, once, however many callers ask. Deferred because callers run inside handlers of the very widgets a rebuild frees.
func _queue_rebuild() -> void:
	if _rebuild_queued:
		return
	_rebuild_queued = true
	call_deferred("_rebuild_now")


## Deferred body of `_queue_rebuild()`.
func _rebuild_now() -> void:
	_rebuild_queued = false
	_rebuild()


## Tear down and redraw the whole list from `_doc`. A full rebuild rather than in-place updates
## because a reorder or a delete changes which row is which (the ramp rows' own reason). Never
## called from inside a colour dialog's callback (rule 5).
func _rebuild() -> void:
	for c in _list.get_children():
		_list.remove_child(c)
		c.queue_free()
	var show_controls := not _unknown
	_add_host.visible = show_controls
	_reset_btn.visible = show_controls
	if _unknown:
		_show_notice("This layer is read from the engine, which is busy generating. It appears again when the world is ready.")
	else:
		var n := _entries().size()
		for i in n:
			_build_entry(i, n)
		if n == 0:
			var l := DccWidgets.note(_list, "No adjustments on this layer.")
			l.name = "AdjEmpty"
	for b in _add_host.get_children():
		if b is Button:
			var full := not _unknown and _entries().size() >= MAX_ENTRIES
			if full:
				(b as Button).set_meta("adj_off", true)
			elif (b as Button).has_meta("adj_off"):
				(b as Button).remove_meta("adj_off")
			(b as Button).disabled = full
	_apply_lock()
	_refresh_guard()
	_refresh_persist_note()


## One adjustment's card: title row (enable, name), reorder and delete buttons, opacity, then
## the kind's parameter editors.
func _build_entry(i: int, n: int) -> void:
	var e: Dictionary = _entries()[i]
	var kind := _kind_of(e)
	var card := VBoxContainer.new()
	card.name = "AdjEntry_%d" % i
	card.add_theme_constant_override("separation", 2)
	card.set_meta("adj_kind", kind)
	_list.add_child(card)
	## ET-F1: the Look entry (see the header) is tagged and pinned, never listed a second time.
	var pinned := is_look_entry(_entries())
	var is_look := pinned and i == 0
	card.set_meta("adj_look", is_look)
	var title := "%d  %s" % [i + 1, String(KIND_SHORT[kind])]
	var tip := String(KIND_TIP[kind]) + " Untick to skip it without deleting it."
	if is_look:
		title = "%d  Look: %s" % [i + 1, String(KIND_SHORT[kind])]
		tip += " This is the Look entry: the Hue, Saturation and Brightness sliders under Look write it, and it stays first. Setting all three to 0 removes it."
	var cb := DccWidgets.toggle(card, title, bool(e.get("enabled", true)),
		func(on: bool): _on_toggle(i, on), tip)
	cb.name = "AdjEnable_%d" % i
	_floor_touch(cb)
	var bar := HBoxContainer.new()
	bar.add_theme_constant_override("separation", 6)
	card.add_child(bar)
	var up := _action(bar, "Up", func(): _on_move(i, -1), "Apply this adjustment earlier.")
	up.name = "AdjUp_%d" % i
	## Row 2 cannot move above a pinned Look entry.
	var up_off := i == 0 or (pinned and i == 1)
	up.disabled = up_off
	if up_off:
		up.set_meta("adj_off", true)
	var down := _action(bar, "Down", func(): _on_move(i, 1), "Apply this adjustment later.")
	down.name = "AdjDown_%d" % i
	## The Look entry itself cannot move down.
	var down_off := i == n - 1 or is_look
	down.disabled = down_off
	if down_off:
		down.set_meta("adj_off", true)
	var del := _action(bar, "Delete", func(): _on_delete(i), "Remove this adjustment.")
	del.name = "AdjDel_%d" % i
	for b in [up, down, del]:
		(b as Button).size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_slider(card, "Opacity", 0.0, 1.0, 0.01, float(e.get("opacity", 1.0)), "opacity", "How strongly this adjustment applies, 0 to 1.",
		func(v: float): e["opacity"] = v)
	match kind:
		"levels":
			_build_levels(card, e)
		"curves":
			_build_curves(card, i, e)
		"hsl":
			_build_hsl(card, e)
		"gradient_map":
			_build_gradient(card, i, e)


## Levels: five sliders. `in_black` and `in_white` narrow each other's limits as one moves, so
## `in_black < in_white` (the engine's rule) can never be violated by a drag.
func _build_levels(card: Control, e: Dictionary) -> void:
	var p := _body(e)
	var ib := _slider(card, "In black", 0.0, float(p["in_white"]) - 1.0, 1.0, float(p["in_black"]), "in_black",
		"Input value that becomes black, 0 to 255, always below In white.", func(v: float): p["in_black"] = v)
	var iw := _slider(card, "In white", float(p["in_black"]) + 1.0, BYTE_MAX, 1.0, float(p["in_white"]), "in_white",
		"Input value that becomes white, 0 to 255, always above In black.", func(v: float): p["in_white"] = v)
	ib["slider"].value_changed.connect(func(v: float): (iw["slider"] as Slider).min_value = v + 1.0)
	iw["slider"].value_changed.connect(func(v: float): (ib["slider"] as Slider).max_value = v - 1.0)
	_slider(card, "Gamma", GAMMA_MIN, GAMMA_MAX, 0.01, float(p["gamma"]), "gamma",
		"Midtone exponent, 0.1 to 10. Above 1 brightens the midtones.", func(v: float): p["gamma"] = v)
	_slider(card, "Out black", 0.0, BYTE_MAX, 1.0, float(p["out_black"]), "out_black",
		"Output for black, 0 to 255. Above Out white inverts.", func(v: float): p["out_black"] = v)
	_slider(card, "Out white", 0.0, BYTE_MAX, 1.0, float(p["out_white"]), "out_white",
		"Output for white, 0 to 255.", func(v: float): p["out_white"] = v)


## Hue / Saturation / Lightness: three sliders, each over the engine's validated range (`adjust.rs::HUE_RANGE_DEG`, -1..=1, -1..=1).
func _build_hsl(card: Control, e: Dictionary) -> void:
	var p := _body(e)
	_slider(card, "Hue", -HUE_MAX, HUE_MAX, 1.0, float(p["hue_deg"]), "hue_deg",
		"Hue rotation in degrees, -360 to 360.", func(v: float): p["hue_deg"] = v)
	_slider(card, "Saturation", -1.0, 1.0, 0.01, float(p["saturation"]), "saturation",
		"Chroma scale, -1 (grey) to 1 (double).", func(v: float): p["saturation"] = v)
	_slider(card, "Lightness", -1.0, 1.0, 0.01, float(p["lightness"]), "lightness",
		"Lightness shift, -1 (toward black) to 1 (toward white).", func(v: float): p["lightness"] = v)


## Curves as numeric rows, an "in" and an "out" slider per control point (the scope allows
## numeric rows where a point editor is not practical; the ramp bar's drag gestures do not map
## onto a two-axis curve). Each point's `in` slider is limited to strictly between its
## neighbours', so `x` stays strictly increasing (the engine's rule) and is narrowed live as a
## neighbour moves.
func _build_curves(card: Control, i: int, e: Dictionary) -> void:
	var pts: Array = _body(e)["points"]
	var xs: Array = []
	for j in pts.size():
		var lo := 0.0 if j == 0 else float(pts[j - 1][0]) + 1.0
		var hi := BYTE_MAX if j == pts.size() - 1 else float(pts[j + 1][0]) - 1.0
		var pt: Array = pts[j]
		var hx := _slider(card, "Point %d in" % (j + 1), lo, hi, 1.0, float(pt[0]), "x%d" % j,
			"Where this control point sits on the input axis, 0 to 255, strictly between its neighbours.",
			func(v: float): pt[0] = v)
		xs.append(hx)
		_slider(card, "Point %d out" % (j + 1), 0.0, BYTE_MAX, 1.0, float(pt[1]), "y%d" % j,
			"What this control point maps to, 0 to 255.", func(v: float): pt[1] = v)
	for j in xs.size():
		var s: Slider = xs[j]["slider"]
		s.value_changed.connect(func(v: float):
			if j > 0:
				(xs[j - 1]["slider"] as Slider).max_value = v - 1.0
			if j < xs.size() - 1:
				(xs[j + 1]["slider"] as Slider).min_value = v + 1.0)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	card.add_child(row)
	var add := _action(row, "Add point", func(): _on_add_point(i), "Insert a control point in the widest gap.")
	add.name = "AdjAddPoint_%d" % i
	var rem := _action(row, "Remove point", func(): _on_remove_point(i), "Remove the last control point.")
	rem.name = "AdjRemPoint_%d" % i
	if pts.size() >= MAX_CURVE_POINTS:
		add.disabled = true
		add.set_meta("adj_off", true)
	if pts.size() <= MIN_CURVE_POINTS:
		rem.disabled = true
		rem.set_meta("adj_off", true)
	for b in [add, rem]:
		(b as Button).size_flags_horizontal = Control.SIZE_EXPAND_FILL


## Insert a point at the middle of the widest gap, at the height the curve's own line segment
## would have there (so the shape barely changes). Refused, with a message, when every gap is
## narrower than 2 (no integer `x` strictly between two neighbours).
func _on_add_point(i: int) -> void:
	var pts: Array = _body(_entries()[i])["points"]
	if pts.size() >= MAX_CURVE_POINTS:
		return
	var best := -1
	var gap := 0.0
	for j in range(pts.size() - 1):
		var g := float(pts[j + 1][0]) - float(pts[j][0])
		if g > gap:
			gap = g
			best = j
	if best < 0 or gap < 2.0:
		_show_notice("No room for another point: every gap between points is already 1 wide.")
		return
	_structural("%s: add curve point" % _layer_label(), func():
		var a: Array = pts[best]
		var b: Array = pts[best + 1]
		var x := float(a[0]) + floorf(gap / 2.0)
		var t := (x - float(a[0])) / (float(b[0]) - float(a[0]))
		pts.insert(best + 1, [x, roundf(float(a[1]) + (float(b[1]) - float(a[1])) * t)]))


## Remove the last curve control point of entry `i` (one history step), never below `MIN_CURVE_POINTS`.
func _on_remove_point(i: int) -> void:
	var pts: Array = _body(_entries()[i])["points"]
	if pts.size() <= MIN_CURVE_POINTS:
		return
	_structural("%s: remove curve point" % _layer_label(), func(): pts.pop_back())


## Gradient map: a blend picker and the SAME `ramp_bar.gd` the elevation ramp uses, driven
## through a small adapter that presents this entry's stops as that bar's "engine ramp".
func _build_gradient(card: Control, i: int, e: Dictionary) -> void:
	var ramp: Dictionary = _body(e)["ramp"]
	var mode := String(ramp.get("mode", "Linear"))
	var pick := DccWidgets.choice(card, "Blend", RAMP_MODES, maxi(RAMP_MODES.find(mode), 0),
		func(k: int): _on_ramp_mode(i, String(RAMP_MODES[k])),
		"How the gradient crosses from one stop to the next: Linear, Ease (soft joins) or Step (flat bands).")
	pick.name = "AdjBlend_%d" % i
	_floor_touch(pick)
	var ad := _GradAdapter.new()
	ad.panel = self
	ad.entry = e
	var rb = _RampBar.new()
	rb.name = "AdjRamp_%d" % i
	rb.setup(ad, func(_s: Array, _sel: int, final: bool): _on_ramp_edited(final), Callable(),
		_on_ramp_begin)
	card.add_child(rb)
	rb.set_stops(ad.color_ramp(), mode)


## The ramp bar's gesture-start hook (a stop press, a bar click, a colour dialog opening):
## snapshot BEFORE the first write.
func _on_ramp_begin() -> void:
	_gest_label = "%s: gradient map" % _layer_label()
	_begin_gesture()


## The ramp bar's callback: it has already written through the adapter (so `_apply` ran). The
## one `final` call per gesture closes the history step. A non-final call is a live tick or an
## unchanged release; neither rebuilds anything (rule 5).
func _on_ramp_edited(final: bool) -> void:
	if final:
		_end_gesture(_gest_label)


## Blend picker handler of gradient-map entry `i` (one history step).
func _on_ramp_mode(i: int, mode: String) -> void:
	_structural("%s: gradient blend %s" % [_layer_label(), mode], func():
		(_body(_entries()[i])["ramp"] as Dictionary)["mode"] = mode)


## Presents one gradient-map entry's stops to `ramp_bar.gd` as if they were the engine's
## elevation ramp: `color_ramp()` is `[[offset, Color]]`, `set_color_ramp()` returns how many
## stops were accepted (0 = refused, the bar then writes nothing back). The stop alpha rides in
## the Color's alpha, the same convention the elevation ramp has.
class _GradAdapter extends RefCounted:
	var panel
	var entry: Dictionary

	## The entry's stops as the ramp bar reads them: `[[offset, Color]]`, channels 0..1 (the saved form is 0..255), stop alpha in the Color's alpha.
	func color_ramp() -> Array:
		var out: Array = []
		for s in (entry["kind"]["gradient_map"]["ramp"]["stops"] as Array):
			var c: Array = s["col"]
			out.append([float(s["at"]), Color(float(c[0]) / 255.0, float(c[1]) / 255.0, float(c[2]) / 255.0, float(s["a"]))])
		return out

	## The ramp bar's write: store and push the stops, returning how many the owner accepted (0 = refused, so the bar reports nothing).
	func set_color_ramp(stops: Array) -> int:
		return panel._grad_write(entry, stops)


## Convert the ramp bar's `[[offset, Color]]` into the saved form (sorted by offset, channels on
## the 0..255 scale, clamped into range so rounding noise can never produce an invalid stop),
## store it in the entry and write. Returns the stop count accepted, 0 on a refusal.
func _grad_write(entry: Dictionary, stops: Array) -> int:
	if stops.is_empty():
		return 0
	var list: Array = []
	for s in stops:
		var c: Color = s[1]
		list.append({"at": clampf(float(s[0]), 0.0, 1.0),
			"col": [clampf(c.r * BYTE_MAX, 0.0, BYTE_MAX), clampf(c.g * BYTE_MAX, 0.0, BYTE_MAX), clampf(c.b * BYTE_MAX, 0.0, BYTE_MAX)],
			"a": clampf(c.a, 0.0, 1.0)})
	list.sort_custom(func(a, b): return float(a["at"]) < float(b["at"]))
	((entry["kind"]["gradient_map"]["ramp"]) as Dictionary)["stops"] = list
	_pending = false
	return list.size() if _apply() else 0


# -- widget helpers ----------------------------------------------------------------------

## A slider row over `DccWidgets.slider`, wired to the gesture: `drag_started` snapshots,
## every change writes (throttled), `drag_ended` closes ONE history step. `setter` stores the
## value into the entry (`_doc`) before the write. The returned Dictionary is the widget
## handle; the slider carries meta `adj_param` for probes.
func _slider(parent: Control, label_text: String, mn: float, mx: float, step: float, value: float,
		param: String, tip: String, setter: Callable) -> Dictionary:
	var h := DccWidgets.slider(parent, label_text, mn, mx, step, value, "",
		func(v: float):
			setter.call(v)
			_changed_live("%s: %s" % [_layer_label(), label_text.to_lower()]),
		tip,
		func():
			_dragging = false
			_end_gesture(_gest_label))
	var s: HSlider = h["slider"]
	s.set_meta("adj_param", param)
	s.drag_started.connect(func():
		_dragging = true
		_gest_label = "%s: %s" % [_layer_label(), label_text.to_lower()]
		_begin_gesture())
	if DccTheme.is_touch():
		(h["row"] as Control).custom_minimum_size.y = maxf((h["row"] as Control).custom_minimum_size.y, TOUCH_PX)
		s.custom_minimum_size.y = maxf(s.custom_minimum_size.y, TOUCH_PX)
	return h


## A button floored to the touch minimum on BOTH axes at the call site (rule 6).
func _action(parent: Control, text: String, on_press: Callable, tip: String = "") -> Button:
	var b := DccWidgets.action(parent, text, on_press)
	b.tooltip_text = tip
	_floor_touch(b)
	return b


## On touch, raise `c` to the 44 px floor on BOTH axes at the call site (rule 6 in the header). A no-op with a pointer.
func _floor_touch(c: Control) -> void:
	if DccTheme.is_touch():
		c.custom_minimum_size = Vector2(maxf(c.custom_minimum_size.x, TOUCH_PX), maxf(c.custom_minimum_size.y, TOUCH_PX))


# -- notices -----------------------------------------------------------------------------

## Show `text` in the notice line (the engine's refusal text, the generating pause, a limit reached). Empty text hides it.
func _show_notice(text: String) -> void:
	_notice.text = text
	_notice.visible = text != ""


## Hide the notice line.
func _clear_notice() -> void:
	_notice.text = ""
	_notice.visible = false


## The overlay contrast guard's plain-language outcome, shown only on the Labels & overlays
## layer. `MapOverlay.adjusted_factions_mode()` records what the FACTION-colour guard settled
## on (`"stack"` met the guard as it was; `"guarded"`, `"clamped"` and `"original"` each mean it
## held the adjustment back). The label and halo guard (`guard_ink_pair`) is a pure function per
## ink and records nothing, so its outcome has no state to read; the note says it is applied.
func _refresh_guard() -> void:
	if _guard == null:
		return
	var text := ""
	if _layer == OVERLAY and not _unknown and _OA.is_live():
		var ov = null
		if app != null and app.get("viewport") != null:
			ov = app.viewport.get("overlay")
		var mode := "none"
		if ov != null and ov.has_method("adjusted_factions_mode"):
			mode = String(ov.adjusted_factions_mode())
		match mode:
			"guarded":
				text = "The contrast guard moved some faction colours so they stay tell-apart-able and readable on the map. Labels and halos are held readable the same way."
			"clamped":
				text = "The contrast guard reduced the strength of this stack on faction colours so they stay tell-apart-able. Labels and halos are held readable the same way."
			"original":
				text = "The contrast guard held this stack back from faction colours entirely: they are drawn in their original colours. Labels and halos are held readable the same way."
			_:
				text = "Labels and halos are held readable (contrast of at least 4.5, or the preset's own if lower) and faction colours stay distinct: the guard may soften a strong stack."
	_guard.text = text
	_guard.visible = text != ""

## The labels/overlays stack is saved with the project only when the engine registers its document slot
## (`DccApp.overlay_adjust_persists()`); on a build that does not, say so on that layer instead of letting
## the person believe an edit is kept. Hidden on the engine layers (their stacks ride in `appearance.json`).
func _refresh_persist_note() -> void:
	if _persist == null:
		return
	var text := ""
	if _layer == OVERLAY and app != null and app.has_method("overlay_adjust_persists_state") and int(app.overlay_adjust_persists_state()) == 0:
		text = "These adjustments last for this session only: this build's project file has no place to store them yet, so saving does not keep them."
	_persist.text = text
	_persist.visible = text != ""

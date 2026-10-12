extends RefCounted
## One shared undo/redo history for LOOK edits (owner Ruling BY item 2,
## 2026-10-10): elevation-ramp stops and blend mode (dock stops section, CARTO
## "Colour relief" editor, the interactive ramp bar), the biome colour table,
## a Look/Map-style preset or base-look pick, and (ET-C4, `EDITING_TOOLS_SCOPE.md`
## section 5) the three per-layer adjustment stacks (Terrain, Water, Labels &
## overlays). One entry per gesture: a slider drag is one entry.
##
## ET-F1's Look sliders (Hue, Saturation, Brightness) need no part of their own: they write
## one Hue/Sat/Light entry pinned at the top of the selected layers' stacks, so their
## gestures are ordinary `adjust_*` parts, labelled with `LOOK_LABEL`.
##
## No `class_name` on purpose: hosts load it with
## `preload("res://shell/look_history.gd")`, so registering it never needs the
## `godot --import` pass that strips `project.godot`'s `;` comment block
## (CLAUDE.md, last working rule).
##
## **Separate from the heightmap undo, and it must stay separate.** The engine's
## `can_undo`/`undo_last` stack is heightmap (and way-delete) only and this file
## never touches it. What is shared is the *front door*: `DccApp.undo_last()` /
## `redo_last()` (Edit > Undo/Redo, Ctrl+Z, Ctrl+Shift+Z) ask `wants_undo()` /
## `wants_redo()` which history the most recent edit belonged to.
##
## **"Last edited domain" is derived, not tracked.** Nothing hooks the engine's
## height commits, so each look edit records `height_sig()` (the engine undo
## stats plus the redo flag). While the signature still equals the one stored at
## the last look edit, no height step has been committed, undone or redone since,
## so the look history owns the next Ctrl+Z. Any height commit/undo/redo changes
## the signature (depth, bytes, label or redo flag moves), handing the next press
## back to the height stack; when one stack is empty the other answers.
##
## **Entry shape**: `{"label": String, "before": Dictionary, "after": Dictionary}`
## where each snapshot holds only the parts the gesture could move (see
## `snapshot()`). `cursor` splits applied entries `[0, cursor)` from the redo tail
## `[cursor, size)`; a new entry drops the tail.
##
## **Session-only**: never saved with a project, and `clear()`ed whenever a
## different world is generated or loaded (`DccApp._refresh_world_dependent()`).
##
## What it does NOT cover (nothing here can read it back): Painter/NPR/appearance
## SLIDER edits, the layer stack, colour space, saved-look load, and the quality
## tier. A preset entry restores the look name, the NPR block, the preset's own
## appearance keys and the ramp -- exactly what `_apply_preset` writes -- and (ET-F3,
## `PRESET_PART`) which preset and variant was applied.

## Longest history kept. A labelled judgement: a session of colour tuning is a few
## dozen gestures, a snapshot is a few hundred bytes (15 colours + ~10 stops + one
## small dictionary), so 100 entries is tens of KB and far past any realistic walk
## back. Oldest entries fall off first.
const MAX_ENTRIES := 100
## Biome classes the engine's table holds (`engine_bridge.gd`: "index 1..15").
const BIOME_N := 15
## The `parts` names of the adjustment stacks (ET-C4) and the snapshot key each one
## writes: the part `"adjust_terrain"` stores the layer's stack as its saved-form
## JSON TEXT under the key `"adjust_terrain"`, and so on. Text, not a parsed
## dictionary, so `_same()` compares two snapshots by value and an undo restores
## exactly the document the engine (or `overlay_adjust.gd`) reported, never a
## re-serialisation of it. The first two are engine layers (`adjust_get/set`);
## `adjust_overlay` is GDScript state (`shell/overlay_adjust.gd`).
const ADJUST_ENGINE_PARTS := {"adjust_terrain": "terrain", "adjust_water": "water"}
## The `parts` name (and snapshot key) of the labels/overlays adjustment stack, which is GDScript
## state in `overlay_adjust.gd`; `snapshot` stores its JSON text and the render workspace's
## `on_look_restored` checks for it.
const ADJUST_OVERLAY_PART := "adjust_overlay"
## `shell/overlay_adjust.gd`, the labels/overlays stack (static state, no engine).
const _OA := preload("res://shell/overlay_adjust.gd")
## Snapshot key holding presentation chrome (active tile, Custom note, readout,
## overlay ink). Restored with the data but excluded from the "did anything
## change" comparison, so a Cancel that reverts the data to equal pushes nothing
## even though the live ticks already flipped the readout to "Custom".
const CHROME := "chrome"
## ET-F3 (`EDITING_TOOLS_SCOPE.md` section 7): the snapshot part naming which style preset
## and which of its variants is applied, `{"index": int, "variant": String}` (`""` = the base
## preset, `index` -1 = Custom). Only `RenderWorkspace._apply_preset` asks for it, so a preset
## press that changes nothing but the variant (a cdylib with no appearance API, where the
## variant moves only the overlay inks) is still ONE entry, and an undo across a variant
## switch restores the variant as data rather than as chrome. It is a DATA part (compared by
## `_same()`), unlike `CHROME`, so it must never join a gesture that can be cancelled back to
## equal data -- a Cancel after a live tick flipped the gallery to Custom would then push an
## entry; that is why no other gesture snapshots it. `apply()` writes nothing for it: the
## engine half of a variant is the `style` part's appearance keys; `RenderWorkspace.
## on_look_restored()` relights the gallery from this part and pushes the overlay inks
## recorded in `CHROME`.
const PRESET_PART := "preset"
## Label prefix of an ET-F1 Look-slider gesture (`"Look: saturation"`), as Edit > Undo shows it.
## The control's name in the shell (section "Look"), so the undo row names what was moved.
const LOOK_LABEL := "Look"

## Emitted whenever the entries or cursor change (push, undo, redo, clear), so
## an Edit menu or toolbar can refresh its enabled state.
signal changed

## The recorded gestures, oldest first; see the header for the entry shape.
## Treat as read-only outside this file.
var entries: Array = []
## Entries `[0, cursor)` are applied; `[cursor, size)` is the redo tail.
var cursor := 0
## `height_sig()` at the most recent look push/undo/redo; see the header.
var _sig := ""
## Snapshot taken when the current gesture began; `{}` when none is open.
var _open: Dictionary = {}

## Forget everything (a different world was generated or opened).
func clear() -> void:
	entries.clear()
	cursor = 0
	_sig = ""
	_open = {}
	changed.emit()

## Whether an applied entry exists to step back over.
func can_undo() -> bool:
	return cursor > 0

## Whether a redo tail exists.
func can_redo() -> bool:
	return cursor < entries.size()

## Label of the entry `undo` would step over, or `""` when there is none.
func undo_label() -> String:
	return String(entries[cursor - 1]["label"]) if can_undo() else ""

## Label of the entry `redo` would re-apply, or `""` when there is none.
func redo_label() -> String:
	return String(entries[cursor]["label"]) if can_redo() else ""

## Remember the state a gesture is about to change. Call BEFORE the first engine
## write of the gesture; a later `commit()` pairs it with the result. Calling it
## again replaces the pending snapshot (a press that never committed is dropped).
func begin_gesture(snap: Dictionary) -> void:
	_open = snap

## Close the gesture opened by `begin_gesture()` (or pass `before` explicitly).
## Pushes ONE entry unless the data parts are equal -- a Cancel, a no-op press, a
## refused write. Returns whether an entry was pushed.
func commit(label: String, after: Dictionary, bridge: Object, before: Dictionary = {}) -> bool:
	var b: Dictionary = before if not before.is_empty() else _open
	_open = {}
	if b.is_empty() or after.is_empty() or _same(b, after):
		return false
	entries.resize(cursor)   ## a new edit drops the redo tail
	entries.append({"label": label, "before": b, "after": after})
	if entries.size() > MAX_ENTRIES:
		entries.pop_front()
	cursor = entries.size()
	_sig = height_sig(bridge)
	changed.emit()
	return true

## Step back one entry; returns it (apply its `before`), or `{}` when empty.
func undo(bridge: Object) -> Dictionary:
	if not can_undo():
		return {}
	cursor -= 1
	_sig = height_sig(bridge)
	changed.emit()
	return entries[cursor]

## Step forward one entry; returns it (apply its `after`), or `{}` when none.
func redo(bridge: Object) -> Dictionary:
	if not can_redo():
		return {}
	var e: Dictionary = entries[cursor]
	cursor += 1
	_sig = height_sig(bridge)
	changed.emit()
	return e

## True when the next Undo belongs here rather than to the heightmap stack: this
## history has a step and either no height edit has happened since the last look
## edit, or the height stack has nothing to undo.
func wants_undo(bridge: Object) -> bool:
	return can_undo() and (height_sig(bridge) == _sig or not bridge.can_undo())

## The Redo mirror of `wants_undo()`.
func wants_redo(bridge: Object) -> bool:
	return can_redo() and (height_sig(bridge) == _sig or not bridge.redo_available())

## Cheap fingerprint of the height/way undo stack: any commit, undo or redo moves
## at least one field. Empty-build safe (`undo_stats()` is `{}` on an old cdylib);
## guarded on `generating` because the worker holds the engine then and a read
## would panic (`engine_bridge.gd::redo_available`'s own note).
static func height_sig(bridge: Object) -> String:
	if bridge == null or bridge.generating:
		return "generating"
	return "%s|%s|%s" % [str(bridge.undo_stats()), str(bridge.can_undo()), str(bridge.redo_available())]

## Data parts equal (the chrome key is ignored; see `CHROME`).
static func _same(a: Dictionary, b: Dictionary) -> bool:
	var x: Dictionary = a.duplicate()
	var y: Dictionary = b.duplicate()
	x.erase(CHROME)
	y.erase(CHROME)
	return x == y

## Read the requested parts back from the engine. `parts` is any of:
##   "ramp"   -> `ramp` (sorted `[[offset, Color], ...]`) and `mode`
##   "biomes" -> `biomes`: 15 colours, index 0 = class 1
##   "style"  -> `style`: the base look name, the NPR block, and the effective
##              values of `appearance_keys` (the keys a preset overrides)
##   "adjust_terrain" / "adjust_water" -> that engine layer's stack, saved-form
##              JSON text (ET-C4); omitted while a generate runs (the engine is
##              the worker's) or when the cdylib has no adjustment stacks
##   "adjust_overlay" -> the labels/overlays stack, saved-form JSON text (static
##              GDScript state, readable at any time)
##   "preset" -> (ET-F3, `PRESET_PART`) a copy of `preset_state`, the caller's
##              `{"index", "variant"}`; omitted when the caller passes none (it is
##              GDScript gallery state the bridge cannot read)
## Parts the running cdylib cannot read are omitted, never defaulted (MISTAKES:
## absent is not a plausible value) -- in particular a stack that could not be
## READ is never recorded as the empty stack, which would make an undo CLEAR it.
static func snapshot(bridge: Object, parts: Array, appearance_keys: PackedStringArray = PackedStringArray(), preset_state: Dictionary = {}) -> Dictionary:
	var s := {}
	if PRESET_PART in parts and not preset_state.is_empty():
		s[PRESET_PART] = preset_state.duplicate()
	if "ramp" in parts and bridge.ramp_api:
		s["ramp"] = bridge.color_ramp()
		if bridge.ramp_mode_api:
			s["mode"] = bridge.ramp_mode()
	if "biomes" in parts and bridge.biome_colors_api and not bridge.generating:
		var cols: Array = []
		for i in range(1, BIOME_N + 1):
			cols.append(bridge.biome_color(i))
		s["biomes"] = cols
	if "style" in parts:
		var st := {}
		if bridge.look_api:
			st["look"] = bridge.look()
		if bridge.npr_api:
			st["npr"] = bridge.npr_settings()
		if bridge.appearance_api:
			var ap: Dictionary = bridge.appearance()
			var sub := {}
			for k in appearance_keys:
				if ap.has(k):
					sub[k] = ap[k]
			st["appearance"] = sub
		if not st.is_empty():
			s["style"] = st
	## ET-C4: the adjustment stacks. A failed or absent read leaves the key out.
	for part in ADJUST_ENGINE_PARTS:
		if part in parts and bridge.adjust_api and not bridge.generating:
			var got: Dictionary = bridge.adjust_get(String(ADJUST_ENGINE_PARTS[part]))
			if bool(got.get("ok", false)) and got.has("stack"):
				s[part] = String(got["stack"])
	if ADJUST_OVERLAY_PART in parts:
		s[ADJUST_OVERLAY_PART] = _OA.get_stack_json()
	return s

## Write a snapshot back through the same setters the editors use. Does NOT
## refresh any widget -- the caller does (`RenderWorkspace.on_look_restored`).
## `appearance_keys` must be the set the snapshot was taken with.
##
## The two ENGINE stacks are written here (`adjust_set`, which refuses while a
## generate runs, so a restore then is skipped rather than panicking). The
## labels/overlays stack is NOT: its setter that also re-applies the province-border
## shader is `ViewportHost.set_overlay_adjust`, which this static function has no
## handle to, so `RenderWorkspace.on_look_restored()` writes `adjust_overlay`.
static func apply(bridge: Object, snap: Dictionary, appearance_keys: PackedStringArray = PackedStringArray()) -> void:
	if snap.has("style"):
		var st: Dictionary = snap["style"]
		if st.has("look"):
			bridge.set_look(String(st["look"]))
		if st.has("npr"):
			bridge.set_npr(st["npr"])
		if st.has("appearance"):
			## Drop the preset's pins first so a key that was merely tier/look
			## derived goes back to following them; then pin only the keys whose
			## value still differs (it was a real override before).
			bridge.drop_appearance_overrides(appearance_keys)
			var live: Dictionary = bridge.appearance()
			var diff := {}
			for k in Dictionary(st["appearance"]):
				if not live.has(k) or not is_equal_approx(float(live[k]), float(st["appearance"][k])):
					diff[k] = st["appearance"][k]
			if not diff.is_empty():
				bridge.set_appearance(diff)
	for part in ADJUST_ENGINE_PARTS:
		if snap.has(part):
			bridge.adjust_set(String(ADJUST_ENGINE_PARTS[part]), String(snap[part]))
	if snap.has("ramp"):
		bridge.set_color_ramp(snap["ramp"])
	if snap.has("mode") and String(snap["mode"]) != "":
		bridge.set_ramp_mode(String(snap["mode"]))
	if snap.has("biomes"):
		var cols: Array = snap["biomes"]
		for i in cols.size():
			if bridge.biome_color(i + 1) != (cols[i] as Color):
				bridge.set_biome_color(i + 1, cols[i] as Color)

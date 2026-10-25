extends RefCounted
## LLM-5 (`LLM_ASSISTANT_SCOPE.md` s.2, s.3, s.5b.4; Ruling DD Q10): the
## **grouped undo**. Several edits applied as one change (an Assistant "Apply"
## of several model-proposed changes, later an edit action) undo in ONE
## Edit > Undo and redo in ONE Edit > Redo, and a failure part-way through any
## of the three (apply, undo, redo) rolls the whole group back.
##
## What it is: a thin layer over the histories that already exist. It keeps no
## snapshots of its own except the parameter values, because each member kind
## is already undoable on its own stack:
##
##   member      history it lives on                   identity the group keeps
##   height      engine `HeightUndo` + ledger row      the row's ledger `seq`
##   way (drag)  engine way store + ledger row         the row's ledger `seq`
##   appearance  `look_history.gd` (LookHistory)       the entry's serial `"n"`
##   colour paint `colour_paint_tool.gd`'s own stack   the entry's serial `"n"`
##   parameter   **none existed** (s.5b.4: `param_set` records nothing), so
##               the group keeps `{key, old, new}` itself and writes them back
##               through `bridge.param_write`, the setter every dialog uses
##
## Undoing a group takes its members' steps on their own stacks (engine
## `undo_last`, `LookHistory.undo` + the app's look restore, the paint tool's
## step, the old parameter values), so a single-tool undo and a group undo can
## never disagree about what a step is.
##
## **How it knows a group is on top** (so Ctrl+Z belongs to it): structurally,
## per history, at the moment of the press -- never by remembering who pushed
## last. For undo: the next `n` reversible ledger rows (`undo_ledger()`'s
## `steps` 1..n) are the group's seqs, the newest `n` applied look entries and
## paint steps are the group's serials, and a history the group did not touch
## still has the top it had when the group was committed (its "floor"). For
## redo: every history's top is back at the floor, and the next redo offers
## (`redo_offer_ids()`, look's and paint's redo tails) are exactly what they
## were right after the group was undone. Any later commit anywhere changes one
## of those, which is the linear-history rule (a new edit drops redo) without
## this file having to observe every commit. Parameters never block ownership:
## undo writes the old values back even if a slider moved one since, which is
## what "undo the Assistant's change" means for settings that have no history.
##
## **What it must never do:**
## - Regenerate, refresh or recompute anything. Undoing a parameter restores
##   the SETTING; the world on screen stays the one generated (s.5b.4 item 3).
##   After a generate or open, a group keeps only its parameter members and
##   says so (`world_changed`).
## - Leave a half-applied group. Every failure runs the inverse of each step
##   already taken, newest first; if an inverse itself fails, the result says
##   so in words rather than claiming a clean rollback.
## - Let one member of a group be undone or redone alone. Members carry a
##   `"group"` tag (look and paint entries) or are listed here (engine seqs and
##   redo ids); `blocks_single_undo()` / `blocks_single_redo()` /
##   `blocks_revert_to()` refuse the per-tool paths that would split one, with
##   a reason. A group whose structure was destroyed by something outside any
##   history (Preferences > Memory > Clear undo history) is **dissolved**: its
##   remaining members become ordinary single steps and the status says so.
## - Touch the existing single-step behaviour when no group exists: every
##   public check here answers "not mine" / "" on an empty list.
##
## No `class_name` (preloaded by `app.gd`, like `look_history.gd`), so adding it
## never needs the `godot --import` pass that strips `project.godot`'s comments.

## Largest number of members one group may hold. A labelled judgement: the
## parameter validator's own cap is `MAX_PARAM_CHANGES` = 12 (s.5b.2 V3), and
## an edit action (LLM-7) adds a handful of strokes and commits; 24 is twice
## the parameter cap. A bigger group is refused at commit and rolled back.
const MAX_MEMBERS := 24

## Emitted when the group list changes (commit, undo, redo, dissolve, world
## change), for a menu that wants to refresh its Undo row.
signal changed

## The `DccApp` (untyped: this script must not depend on a `class_name`).
## Read for `bridge`, `look_history`, `_restore_look`, `_workspaces`.
var app = null

## The group stack, oldest first: `[0, cursor)` are applied, `[cursor, size)`
## are undone (redoable while they still own the redo). Entry shape:
##   id: int, label: String, world_changed: bool,
##   engine: Array[int]   ledger seqs, oldest first (rewritten after a redo)
##   engine_kinds: Array[String]  "height" / "way", same order
##   look: Array[int], paint: Array[int]  serials, oldest first
##   params: Array[Dictionary]  {key, old, new}, in write order
##   floor: {engine: int, look: int, paint: int}  the tops below the group
##   after_undo: {offers: PackedInt64Array, look: int, paint: int}  redo-side
##       tops captured right after the group was undone
var groups: Array = []
var cursor := 0
## Undone groups a later group replaced: their members may still sit on a redo
## tail, so they keep guarding them (a lone redo would half-redo the change).
var _spent: Array = []
var _next_id := 1
## `_world_marker()` at the last `on_world_changed`; "" before the first.
var _world_seen := ""
## The open group (between `begin` and `commit`/`abort`); `{}` when none.
var _open: Dictionary = {}
## Why the last group was dissolved, for the caller's status line; "" when none.
var last_note := ""
## The reason of the last `abort()` (including one `write_param` or `commit`
## ran itself), for a caller that got `false` back; "" until one happens.
var last_reason := ""

## **Test seam, probe-only.** When >= 0, the member step with this index
## (0-based, counted in the next undo, redo or rollback pass) reports failure,
## so a probe can prove the rollback without breaking an engine. Reset to -1
## after it fires.
var fail_at_step := -1
var _step_count := 0
## True while `_run` is taking a group's steps. The steps refresh editors and
## menus, which ask `wants_undo()` / `undo_label()` re-entrantly; mid-run a
## group is neither applied nor undone, so `_prune()` must not judge it (it
## would dissolve it) and no group may claim a press.
var _busy := false

# -- opening, writing and closing a group -------------------------------------

## Whether a group is open (between `begin` and `commit`/`abort`).
func is_open() -> bool:
	return not _open.is_empty()

## Open a group named `label` (the Edit menu shows "Undo <label>"). Returns ""
## on success or the reason it was refused: a group already open, no engine,
## a generate running, or an engine build without `undo_commit_seq` (whose
## commits the group could then not account for).
func begin(label: String) -> String:
	if is_open():
		return "another grouped change is still being applied"
	var b = _bridge()
	if b == null:
		return "no engine"
	if b.generating:
		return "the world is generating"
	var e0: int = b.undo_commit_seq()
	if e0 < 0:
		return "this engine build cannot account for grouped undo (no undo_commit_seq)"
	var tool = _paint_tool()
	_open = {
		"label": label,
		"e0": e0,
		"l0": int(app.look_history.serial),
		"p0": int(tool.serial) if tool != null else 0,
		"floor": _tops(),
		"params": [],
		"heights": 0,
	}
	return ""

## How many more HEIGHT steps the open group can take before the engine's
## bounded stack would evict the group's own first one: the stack keeps
## `min(max_steps, budget_bytes / step_bytes)` steps, at least one (`undo.rs`).
## Derived from `undo_stats()`, never a literal. An edit action (LLM-7) asks
## this before each height edit; `commit()` also catches an eviction after the
## fact. `0` with no open group.
func height_room() -> int:
	if not is_open():
		return 0
	var st: Dictionary = _bridge().undo_stats()
	var step := maxi(int(st.get("step_bytes", 0)), 1)
	var cap := maxi(1, mini(int(st.get("max_steps", 0)), int(st.get("budget_bytes", 0)) / step))
	var used := 0
	for r in _new_engine_rows(int(_open["e0"])):
		if String(r.get("kind", "")) == "height":
			used += 1
	return maxi(cap - used, 0)

## Write one parameter as a member of the open group, through
## `bridge.param_write` (the same setter `param_set` and every dialog use).
## Reads the old value first. A rejected or **clamped** write is a failure
## (s.5b.4 item 2: a model's number is never silently changed), and a failure
## rolls the WHOLE open group back (`abort`). Returns whether it landed.
## Writing a key twice keeps the first `old`, so undo restores the value from
## before the group.
func write_param(key: String, value) -> bool:
	if not is_open():
		return false
	var b = _bridge()
	var old = b.param_get(key)
	if old == null:
		abort("unknown setting %s" % key)
		return false
	var r: Dictionary = b.param_write(key, value)
	if not bool(r.get("ok", false)) or bool(r.get("clamped", false)):
		## A clamped write landed a different value; it is in the table now,
		## so record it as a member before rolling back, or the rollback would
		## leave it behind.
		if bool(r.get("ok", false)):
			_note_param(key, old, b.param_get(key))
		abort("%s was %s" % [key, "clamped to its range" if bool(r.get("clamped", false)) else "refused by the engine"])
		return false
	_note_param(key, old, b.param_get(key))
	return true

func _note_param(key: String, old, new_value) -> void:
	for m in _open["params"]:
		if String(m["key"]) == key:
			m["new"] = new_value
			return
	(_open["params"] as Array).append({"key": key, "old": old, "new": new_value})

## Close the open group. Every edit made since `begin` (on the engine's
## stack, the look history and the colour-paint stack) plus the parameter
## writes becomes ONE undo step. Returns `{ok, reason, members}`; on any
## failure the group is rolled back first (`ok` false, `reason` in words):
## an edit that cannot be undone (a recorded-only engine row), a member the
## bounded stacks already evicted, or more than `MAX_MEMBERS` members.
## A group with no members pushes nothing (`members` 0, `ok` true).
func commit() -> Dictionary:
	if not is_open():
		return {"ok": false, "reason": "no grouped change is open", "members": 0}
	var c := _collect()
	if c.has("error"):
		return abort(String(c["error"]))
	var g: Dictionary = c["group"]
	var n := _member_count(g)
	if n == 0:
		_open = {}
		return {"ok": true, "reason": "", "members": 0}
	if n > MAX_MEMBERS:
		return abort("%d changes in one step is more than the %d one undo step may hold" % [n, MAX_MEMBERS])
	g["id"] = _next_id
	_next_id += 1
	_tag(g, int(g["id"]))
	## A new change drops the redo side, as every linear history does: undone
	## groups stop being redoable but keep guarding their members.
	for i in range(cursor, groups.size()):
		_spent.append(groups[i])
	groups.resize(cursor)
	groups.append(g)
	cursor = groups.size()
	_open = {}
	changed.emit()
	return {"ok": true, "reason": "", "members": n}

## Roll the open group back (every member taken so far, inverse order) and
## close it. Returns `{ok: false, reason, members: 0, rolled_back: bool}`;
## `rolled_back` is false only when an inverse step itself failed, and the
## reason then says what is left applied.
func abort(reason: String) -> Dictionary:
	if not is_open():
		return {"ok": false, "reason": reason, "members": 0, "rolled_back": true}
	var c := _collect()
	var g: Dictionary = c["group"]
	_open = {}
	var r := _run(g, true)
	last_reason = reason
	var out := {"ok": false, "reason": reason, "members": 0, "rolled_back": bool(r["ok"])}
	if c.has("error"):
		out["reason"] = "%s; %s" % [reason, String(c["error"])] if String(c["error"]) != reason else reason
	if not bool(r["ok"]):
		out["reason"] = "%s; and the rollback failed: %s" % [out["reason"], String(r["reason"])]
	last_reason = String(out["reason"])
	return out

# -- Edit > Undo / Redo --------------------------------------------------------

## True when the next Edit > Undo belongs to a group: the newest applied group
## is structurally on top of every history (header). Prunes dead groups first.
func wants_undo() -> bool:
	if _busy:
		return false
	_prune()
	return cursor > 0 and _owns_undo(groups[cursor - 1])

## The Redo mirror of `wants_undo()`.
func wants_redo() -> bool:
	if _busy:
		return false
	_prune()
	return cursor < groups.size() and _owns_redo(groups[cursor])

## Label of the group `undo()` would take, or "" when `wants_undo()` is false.
func undo_label() -> String:
	return _display_label(groups[cursor - 1]) if wants_undo() else ""

## Label of the group `redo()` would take, or "".
func redo_label() -> String:
	return _display_label(groups[cursor]) if wants_redo() else ""

## Undo the top group in one step. Returns `{ok, reason, label, height, way,
## look, paint, params, world_changed}` (the member counts let the app repaint
## only what moved). On failure nothing is left changed (each step taken is
## re-applied) and the group stays applied.
func undo() -> Dictionary:
	if not wants_undo():
		return {"ok": false, "reason": "no grouped change is on top"}
	var g: Dictionary = groups[cursor - 1]
	var r := _run(g, true)
	if bool(r["ok"]):
		cursor -= 1
		g["after_undo"] = _redo_tops()
		changed.emit()
	else:
		## The rollback re-did the engine steps it had undone, and a redo
		## records fresh ledger rows: re-read the group's seqs so it still
		## owns the next press.
		_reread_engine(g)
	return _result(g, r)

## After the engine re-applied a group's steps (a redo, or a failed undo's
## rollback), its ledger rows are new: the group's seqs are the top `n` again.
func _reread_engine(g: Dictionary) -> void:
	var n: int = (g["engine"] as Array).size()
	if n > 0:
		var seqs := _next_engine_seqs(n)
		seqs.reverse()
		g["engine"] = seqs

## Redo the next undone group in one step; same contract as `undo()`. The
## engine records a fresh ledger row per redone step, so the group's seqs are
## re-read afterwards.
func redo() -> Dictionary:
	if not wants_redo():
		return {"ok": false, "reason": "no grouped change to redo"}
	var g: Dictionary = groups[cursor]
	var r := _run(g, false)
	if bool(r["ok"]):
		_reread_engine(g)
		cursor += 1
		changed.emit()
	else:
		## The rollback undid again what the redo had re-applied, which puts
		## fresh step ids on the engine's redo tail: re-capture the redo side
		## so the group still owns the next Redo.
		g["after_undo"] = _redo_tops()
	return _result(g, r)

func _result(g: Dictionary, r: Dictionary) -> Dictionary:
	var kinds: Array = g["engine_kinds"]
	return {
		"ok": bool(r["ok"]), "reason": String(r.get("reason", "")), "label": _display_label(g),
		"height": kinds.count("height"), "way": kinds.count("way"),
		"look": (g["look"] as Array).size(), "paint": (g["paint"] as Array).size(),
		"params": (g["params"] as Array).size(), "world_changed": bool(g["world_changed"]),
	}

# -- guards for the single-step paths --------------------------------------------

## The reason a plain (single-history) Undo must not run now, or "" when it may.
## `look_branch` is the app's own routing answer (`look_history.wants_undo`):
## true means the press would take the newest look entry, false the engine's
## next step. Refuses exactly when that step is a member of a group (which, by
## the time this is asked, does not own the press).
func blocks_single_undo(look_branch: bool) -> String:
	if groups.is_empty() and _spent.is_empty():
		return ""
	_prune()
	if look_branch:
		var lh = app.look_history
		if lh.cursor > 0 and (lh.entries[lh.cursor - 1] as Dictionary).has("group"):
			return _split_reason(int(lh.entries[lh.cursor - 1]["group"]), "Undo")
		return ""
	var next := _next_engine_seqs(1)
	if next.is_empty() or int(next[0]) <= 0:
		return ""
	for g in groups.slice(0, cursor):
		if int(next[0]) in (g["engine"] as Array):
			return _split_reason(int(g["id"]), "Undo")
	return ""

## The Redo mirror: refuses when the look history's next redo entry is tagged,
## or the engine's next redo offer is a group's undone step.
func blocks_single_redo(look_branch: bool) -> String:
	if groups.is_empty() and _spent.is_empty():
		return ""
	_prune()
	if look_branch:
		var lh = app.look_history
		if lh.cursor < lh.entries.size() and (lh.entries[lh.cursor] as Dictionary).has("group"):
			return _split_reason(int(lh.entries[lh.cursor]["group"]), "Redo")
		return ""
	return blocks_engine_redo(1)

## Refuses (with a reason) a walk of `steps` engine redos that would re-apply a
## group's undone engine step on its own (`right_dock.gd::_redo_to`, Redo).
func blocks_engine_redo(steps: int) -> String:
	if groups.is_empty() and _spent.is_empty():
		return ""
	var offers: PackedInt64Array = _bridge().redo_offer_ids() if _bridge() != null else PackedInt64Array()
	for i in mini(steps, offers.size()):
		for g in groups.slice(cursor) + _spent:
			var ids: PackedInt64Array = (g.get("after_undo", {}) as Dictionary).get("offers", PackedInt64Array())
			if (g["engine"] as Array).size() > 0 and offers[i] in ids:
				return _split_reason(int(g["id"]), "Redo")
	return ""

## Refuses (with a reason) a History-panel revert to ledger row `seq`, which
## pops every reversible row at or above it, when one of those is a member of
## an applied group (`right_dock.gd::_do_revert`, the phone history popover).
func blocks_revert_to(seq: int) -> String:
	if groups.is_empty():
		return ""
	_prune()
	for g in groups.slice(0, cursor):
		for s in g["engine"]:
			if int(s) >= seq:
				return "Not done: reverting there would take part of \"%s\" and leave the rest. Undo it with Edit > Undo first." % _display_label(g)
	return ""

## The label of group `id` (applied, undone or spent), for a refusal message;
## "a grouped change" when it is no longer known.
func label_of(id: int) -> String:
	for g in groups + _spent:
		if int(g["id"]) == id:
			return _display_label(g)
	return "a grouped change"

func _split_reason(id: int, what: String) -> String:
	return "Not done: that step is part of \"%s\", which is one step. Edit > %s takes it whole once the later edits are %s." % [
		label_of(id), what, "undone" if what == "Undo" else "redone"]

func _display_label(g: Dictionary) -> String:
	return String(g["label"]) + (" (settings only)" if bool(g["world_changed"]) else "")

# -- world changes ---------------------------------------------------------------

## `DccApp._refresh_world_dependent` ran. That handler runs on every
## `world_loaded`, which fires for a generate and an open but ALSO for a fjord
## carve, a centring pass and an applied asset pack (its own header says so),
## and it clears the look history and the paint stack on all of them (an
## existing behaviour this milestone does not change). So the first question is
## whether the world was really replaced: only a generate or an open records a
## new floor row in the engine ledger (or replaces the engine object). If not,
## nothing here changes; a group whose look or paint members that handler just
## cleared is dissolved by `_prune()` at the next press, and the open group
## (a carve inside it) stays open.
##
## When the world WAS replaced, the engine stack, the look history and the
## paint stack are all empty, so a group keeps only its parameter members (the
## scope's rule: undo restores settings, not a world) and is marked
## `world_changed`; a group left with nothing is dropped. The floors and redo
## tops are re-read from the emptied histories. An open group is closed
## without rollback: its other members no longer exist to roll back.
func on_world_changed() -> void:
	var marker := _world_marker()
	if marker == _world_seen:
		return
	_world_seen = marker
	_open = {}
	_spent.clear()
	var keep: Array = []
	var new_cursor := 0
	for i in groups.size():
		var g: Dictionary = groups[i]
		if (g["params"] as Array).is_empty():
			continue
		g["engine"] = []
		g["engine_kinds"] = []
		g["look"] = []
		g["paint"] = []
		g["floor"] = _tops()
		g["after_undo"] = _redo_tops()
		keep.append(g)
		if i < cursor:
			new_cursor += 1
	## Parameter-only groups were never world-dependent; a regenerate still
	## means the values on screen came from a generate, so every survivor is
	## settings-only from here on.
	for g in keep:
		g["world_changed"] = true
	groups = keep
	cursor = new_cursor
	changed.emit()

# -- internals -------------------------------------------------------------------

## Identity of the world the histories describe: the engine object plus the
## newest floor row of its ledger (a generate or open records one and clears
## the rows before it; a carve or a paint commit never does).
func _world_marker() -> String:
	var b = _bridge()
	if b == null or b.generating:
		return _world_seen
	var floor_seq := -1
	for r in b.undo_ledger():
		if String((r as Dictionary).get("kind", "")) == "floor":
			floor_seq = int(r["seq"])
	return "%d|%d" % [b.world_gen.get_instance_id() if b.world_gen != null else 0, floor_seq]

func _bridge():
	return app.bridge if app != null else null

## The colour-paint tool (`CartographyWorkspace.colour_paint`), or null when
## no workspace built one.
func _paint_tool():
	if app == null:
		return null
	for ws in app._workspaces:
		if ws != null and "colour_paint" in ws and ws.colour_paint != null:
			return ws.colour_paint
	return null

func _member_count(g: Dictionary) -> int:
	return (g["engine"] as Array).size() + (g["look"] as Array).size() + (g["paint"] as Array).size() + (g["params"] as Array).size()

## Seqs of the engine's next `n` undo steps, in the order `undo_last()` takes
## them (`undo_ledger()` rows by `steps` 1..n); shorter when fewer exist.
func _next_engine_seqs(n: int) -> Array:
	var by_step := {}
	for r in _bridge().undo_ledger():
		var d: Dictionary = r
		if bool(d.get("reversible", false)) and int(d.get("steps", 0)) > 0:
			by_step[int(d["steps"])] = int(d["seq"])
	var out: Array = []
	for k in range(1, n + 1):
		if not by_step.has(k):
			break
		out.append(by_step[k])
	return out

## Ledger rows recorded after commit counter `e0`, oldest first.
func _new_engine_rows(e0: int) -> Array:
	var out: Array = []
	for r in _bridge().undo_ledger():
		if int((r as Dictionary).get("seq", 0)) > e0:
			out.append(r)
	return out

## The top of each history right now: the engine's next undo seq, the newest
## applied look entry's serial, the newest paint step's serial; 0 for "none".
func _tops() -> Dictionary:
	var next := _next_engine_seqs(1)
	var lh = app.look_history
	var tool = _paint_tool()
	return {
		"engine": int(next[0]) if not next.is_empty() else 0,
		"look": int((lh.entries[lh.cursor - 1] as Dictionary).get("n", 0)) if lh.cursor > 0 else 0,
		"paint": int((tool._undo.back() as Dictionary).get("n", 0)) if tool != null and not tool._undo.is_empty() else 0,
	}

## The redo side of each history right now: the engine's redo offer ids, the
## look history's next redo serial, the paint tool's next redo serial.
func _redo_tops() -> Dictionary:
	var lh = app.look_history
	var tool = _paint_tool()
	return {
		"offers": _bridge().redo_offer_ids(),
		"look": int((lh.entries[lh.cursor] as Dictionary).get("n", 0)) if lh.cursor < lh.entries.size() else 0,
		"paint": int((tool._redo.back() as Dictionary).get("n", 0)) if tool != null and not tool._redo.is_empty() else 0,
	}

## Builds the group record from what happened since `begin`. Members that
## cannot be undone are reported in `error` but the reversible ones are still
## collected, so an abort's rollback takes what it can.
func _collect() -> Dictionary:
	var b = _bridge()
	var e0: int = int(_open["e0"])
	var engine: Array = []
	var kinds: Array = []
	var error := ""
	var rows := _new_engine_rows(e0)
	for r in rows:
		var d: Dictionary = r
		var kind := String(d.get("kind", ""))
		if not bool(d.get("reversible", false)) or not (kind == "height" or kind == "way"):
			error = "\"%s\" cannot be undone (%s)" % [String(d.get("label", "")), String(d.get("reason", "")) if String(d.get("reason", "")) != "" else kind]
			continue
		engine.append(int(d["seq"]))
		kinds.append(kind)
	var recorded: int = b.undo_commit_seq() - e0
	if error == "" and recorded != rows.size():
		error = "%d engine steps were recorded but %d are still listed" % [recorded, rows.size()]
	## The engine steps must be the newest ones (nothing of anyone else's above).
	var top := _next_engine_seqs(engine.size())
	top.reverse()
	if error == "" and top != engine:
		error = "the engine's newest undo steps are not this change's"
	var lh = app.look_history
	var look: Array = []
	for i in lh.cursor:
		var nn := int((lh.entries[i] as Dictionary).get("n", 0))
		if nn > int(_open["l0"]):
			look.append(nn)
	if error == "" and look.size() != int(lh.serial) - int(_open["l0"]):
		error = "a look step of this change was already dropped or undone"
	var tool = _paint_tool()
	var paint: Array = []
	if tool != null:
		for e in tool._undo:
			var nn := int((e as Dictionary).get("n", 0))
			if nn > int(_open["p0"]):
				paint.append(nn)
		if error == "" and paint.size() != int(tool.serial) - int(_open["p0"]):
			error = "a colour-paint step of this change was dropped by the paint stack's bound or undone"
	var g := {
		"id": 0, "label": String(_open["label"]), "world_changed": false,
		"engine": engine, "engine_kinds": kinds, "look": look, "paint": paint,
		"params": (_open["params"] as Array).duplicate(true),
		"floor": _open["floor"], "after_undo": {},
	}
	var out := {"group": g}
	if error != "":
		out["error"] = error
	return out

## Tags the group's look and paint entries with its id.
func _tag(g: Dictionary, id: int) -> void:
	for e in app.look_history.entries:
		if int((e as Dictionary).get("n", 0)) in g["look"]:
			e["group"] = id
	var tool = _paint_tool()
	if tool != null:
		for e in tool._undo + tool._redo:
			if int((e as Dictionary).get("n", 0)) in g["paint"]:
				e["group"] = id

## Removes the group tag from whatever entries still carry it (a dissolve).
func _untag(g: Dictionary) -> void:
	var id := int(g["id"])
	for e in app.look_history.entries:
		if int((e as Dictionary).get("group", -1)) == id:
			(e as Dictionary).erase("group")
	var tool = _paint_tool()
	if tool != null:
		for e in tool._undo + tool._redo:
			if int((e as Dictionary).get("group", -1)) == id:
				(e as Dictionary).erase("group")

## Undo ownership (header): every history's top is the group's members, or the
## group's floor where it has none.
func _owns_undo(g: Dictionary) -> bool:
	var t := _tops()
	var engine: Array = g["engine"]
	if engine.is_empty():
		if int(t["engine"]) != int(g["floor"]["engine"]):
			return false
	else:
		var next := _next_engine_seqs(engine.size())
		next.reverse()
		if next != engine:
			return false
	var lh = app.look_history
	var look: Array = g["look"]
	if look.is_empty():
		if int(t["look"]) != int(g["floor"]["look"]):
			return false
	else:
		if lh.cursor < look.size():
			return false
		for i in look.size():
			if int((lh.entries[lh.cursor - look.size() + i] as Dictionary).get("n", 0)) != int(look[i]):
				return false
	var paint: Array = g["paint"]
	if paint.is_empty():
		return int(t["paint"]) == int(g["floor"]["paint"])
	var tool = _paint_tool()
	if tool == null or tool._undo.size() < paint.size():
		return false
	for i in paint.size():
		if int((tool._undo[tool._undo.size() - paint.size() + i] as Dictionary).get("n", 0)) != int(paint[i]):
			return false
	return true

## Redo ownership (header): every history's top is back at the floor and the
## redo side is exactly as the group's undo left it.
func _owns_redo(g: Dictionary) -> bool:
	if not g.has("after_undo") or (g["after_undo"] as Dictionary).is_empty():
		return false
	var t := _tops()
	for k in ["engine", "look", "paint"]:
		if int(t[k]) != int(g["floor"][k]):
			return false
	var now := _redo_tops()
	var then: Dictionary = g["after_undo"]
	return now["offers"] == then["offers"] and int(now["look"]) == int(then["look"]) and int(now["paint"]) == int(then["paint"])

## Drops groups that can no longer be taken whole. An applied group whose
## members or floor left their histories (Clear undo history, a History-panel
## revert, eviction) is **dissolved**: untagged, so what is left of it is
## ordinary single steps, with `last_note` saying so. An undone group whose
## redo was lost is moved to `_spent` (it keeps guarding its tagged entries).
func _prune() -> void:
	if _busy or app == null or _bridge() == null or _bridge().generating:
		return
	var i := 0
	while i < groups.size():
		var g: Dictionary = groups[i]
		var alive := _alive_applied(g) if i < cursor else _alive_undone(g)
		if alive:
			i += 1
			continue
		groups.remove_at(i)
		if i < cursor:
			cursor -= 1
			_untag(g)
			last_note = "\"%s\" can no longer be undone as one step (part of it left the undo history), so what is left of it is ordinary undo steps." % _display_label(g)
		else:
			_spent.append(g)
		changed.emit()
	## Spent groups whose tagged entries are all gone guard nothing.
	_spent = _spent.filter(func(s): return _has_tagged(s))

func _alive_applied(g: Dictionary) -> bool:
	var live := {}
	for r in _bridge().undo_ledger():
		var d: Dictionary = r
		live[int(d["seq"])] = bool(d.get("reversible", false))
	for s in g["engine"]:
		if not bool(live.get(int(s), false)):
			return false
	var fe := int(g["floor"]["engine"])
	if fe != 0 and not live.has(fe):
		return false
	var lh = app.look_history
	var applied_look: Array = []
	for k in lh.cursor:
		applied_look.append(int((lh.entries[k] as Dictionary).get("n", 0)))
	for s in g["look"]:
		if not (int(s) in applied_look):
			return false
	var fl := int(g["floor"]["look"])
	if fl != 0 and not (fl in applied_look):
		return false
	var tool = _paint_tool()
	var applied_paint: Array = []
	if tool != null:
		for e in tool._undo:
			applied_paint.append(int((e as Dictionary).get("n", 0)))
	for s in g["paint"]:
		if not (int(s) in applied_paint):
			return false
	var fp := int(g["floor"]["paint"])
	return fp == 0 or fp in applied_paint

func _alive_undone(g: Dictionary) -> bool:
	var then: Dictionary = g.get("after_undo", {})
	if then.is_empty():
		return false
	var offers: PackedInt64Array = _bridge().redo_offer_ids()
	var ids: PackedInt64Array = then.get("offers", PackedInt64Array())
	if (g["engine"] as Array).size() > 0 and (ids.is_empty() or not (ids[0] in offers)):
		return false
	var lh = app.look_history
	for s in g["look"]:
		var found := false
		for k in range(lh.cursor, lh.entries.size()):
			if int((lh.entries[k] as Dictionary).get("n", 0)) == int(s):
				found = true
		if not found:
			return false
	var tool = _paint_tool()
	for s in g["paint"]:
		if tool == null:
			return false
		var found := false
		for e in tool._redo:
			if int((e as Dictionary).get("n", 0)) == int(s):
				found = true
		if not found:
			return false
	return true

func _has_tagged(g: Dictionary) -> bool:
	var id := int(g["id"])
	for e in app.look_history.entries:
		if int((e as Dictionary).get("group", -1)) == id:
			return true
	var tool = _paint_tool()
	if tool != null:
		for e in tool._undo + tool._redo:
			if int((e as Dictionary).get("group", -1)) == id:
				return true
	if (g["engine"] as Array).is_empty():
		return false
	var ids: PackedInt64Array = (g.get("after_undo", {}) as Dictionary).get("offers", PackedInt64Array())
	return not ids.is_empty() and ids[0] in _bridge().redo_offer_ids()

## One member step for the test seam: false exactly at `fail_at_step`.
func _tick() -> bool:
	var ok := _step_count != fail_at_step
	_step_count += 1
	if not ok:
		fail_at_step = -1
	return ok

## Runs every member's undo (`undo_side`) or redo, engine first (the step most
## likely to be refused, so the fewest to roll back), then look, paint and
## parameters. On a failure, runs the inverse of every step already taken,
## newest first, and returns `{ok: false, reason}` -- with the inverse's own
## failure appended if one happens. Never regenerates.
func _run(g: Dictionary, undo_side: bool) -> Dictionary:
	_busy = true
	var r := _run_steps(g, undo_side)
	_busy = false
	return r

func _run_steps(g: Dictionary, undo_side: bool) -> Dictionary:
	_step_count = 0
	var b = _bridge()
	var lh = app.look_history
	var tool = _paint_tool()
	var done: Array = []   ## inverse callables, in the order the steps ran
	var fail := ""
	for _i in (g["engine"] as Array).size():
		if not _tick():
			fail = "an engine step was refused (test seam)"
			break
		var ok: bool = b.undo_last() != "" if undo_side else b.redo_last()
		if not ok:
			fail = "the engine refused to %s one of its steps" % ("undo" if undo_side else "redo")
			break
		done.append(func(): return b.redo_last() if undo_side else b.undo_last() != "")
	if fail == "":
		for _i in (g["look"] as Array).size():
			if not _tick():
				fail = "a look step was refused (test seam)"
				break
			var e: Dictionary = lh.undo(b) if undo_side else lh.redo(b)
			if e.is_empty():
				fail = "the look history had no step to %s" % ("undo" if undo_side else "redo")
				break
			_restore_look(e["before"] if undo_side else e["after"])
			done.append(func():
				var back: Dictionary = lh.redo(b) if undo_side else lh.undo(b)
				if back.is_empty():
					return false
				_restore_look(back["after"] if undo_side else back["before"])
				return true)
	if fail == "":
		for _i in (g["paint"] as Array).size():
			if not _tick():
				fail = "a colour-paint step was refused (test seam)"
				break
			if tool == null or not tool.step_for_group(undo_side):
				fail = "the colour-paint step could not be %s" % ("undone" if undo_side else "redone")
				break
			done.append(func(): return tool.step_for_group(not undo_side))
	if fail == "":
		var ps: Array = (g["params"] as Array).duplicate()
		if undo_side:
			ps.reverse()
		for m in ps:
			if not _tick():
				fail = "a setting write was refused (test seam)"
				break
			var to = m["old"] if undo_side else m["new"]
			var back = m["new"] if undo_side else m["old"]
			if not _param_exact(String(m["key"]), to):
				fail = "the setting %s could not be written back" % String(m["key"])
				break
			done.append(func(): return _param_exact(String(m["key"]), back))
	if fail == "":
		return {"ok": true}
	var broken := ""
	for k in range(done.size() - 1, -1, -1):
		if not bool((done[k] as Callable).call()):
			broken = "step %d of %d could not be put back" % [k + 1, done.size()]
			break
	return {"ok": false, "reason": fail + ("; " + broken if broken != "" else "")}

## Writes one parameter and accepts only an exact landing (not clamped, not
## rejected). Values written here were read from the engine, so a clamp means
## the range moved under the group.
func _param_exact(key: String, value) -> bool:
	var r: Dictionary = _bridge().param_write(key, value)
	return bool(r.get("ok", false)) and not bool(r.get("clamped", false))

## Writes a look snapshot back the way the app's own look undo does
## (`DccApp._restore_look`, which also refreshes the editors), or straight
## through `LookHistory.apply` when no render workspace exists to refresh.
func _restore_look(snap: Dictionary) -> void:
	var has_render := false
	for ws in app._workspaces:
		if ws is CartographyWorkspace and ws._render != null:
			has_render = true
	if has_render:
		app._restore_look(snap)
	else:
		app.look_history.apply(_bridge(), snap, RenderWorkspace._preset_appearance_keys())

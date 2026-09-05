extends Node
## Committed verification harness for the vault panel's **three snapshot
## states** — `vault_window.gd::_build_snapshots`, 2026-09-05.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _vaultgone_probe.tscn
##
## **Windowed, not `--headless`.** Nothing here samples pixels, so the
## `ImageTexture.update()` trap does not apply — but the panel it reads is
## built by a real `AcceptDialog` popping up, and running it the way the user
## does costs nothing and removes the question.
##
## ## What it pins
##
## `WorldGen::vault_snapshot_radii` keeps three states apart and **omits** its
## `missing` key rather than sending `false`, because the absence of the key
## means *unknown* and not *fine*. The panel drew its tick from `path` alone,
## so a snapshot deleted from outside Cartalith still read as generated and
## still offered *Regenerate*. This probe deletes the PNG **behind the
## panel's back** — `DirAccess.remove_absolute`, so nothing tells the store —
## and asserts the row changes its mark, its sentence and its button.
##
## The other half matters as much: **a fix that changes all three states is
## not a fix.** The never-generated and present renderings are asserted
## against the literal strings they had before this pass (`○ not generated`
## with *Generate local*; `✓ <rel>` with *Regenerate local*), so a later edit
## that "improves" them has to do it deliberately.
##
## Row labels are `caption + "\n" + status`, so the marks are matched with a
## *contains* and the buttons with an *exact* string — which is also what
## keeps *Generate local* and *Generate local again* apart.
##
## ## It was shown to fail (2026-09-05)
##
## A green probe proves nothing until it has been made red. `_build_snapshots`'
## `if d.has("missing"):` was mutated to a key that never exists — reverting
## the row to the old two-state rendering, restored in a `finally`, the file
## hashing identical afterwards. **5 FAILED**, counted from the run: the four
## state-3 checks — the tick that should have gone, the sentence, the button
## that should not say *Regenerate*, and the *Generate … again* that was
## counted 0 — plus *"three states, three renderings"*, which reported
## `○3 ✓0 ✕0 | ○2 ✓1 ✕0 | ○2 ✓1 ✕0` and named the collapse directly. States 1
## and 2 and every engine-side check stayed green. That is the shape a real
## regression would have.
##
## Committed like every probe scene in this folder — `STATUS.md`'s F8 row.

const SEED := 483920
const SNAP_PX := 128

var _app: Node
var _bridge
var _vw
var _root := ""
var _fails: Array = []
## The real profile's vault sidecars, put back exactly as found: this probe
## runs against the same `user://` a real session uses.
var _saved: Dictionary = {}


func _ok(label: String, cond: bool, detail: String = "") -> void:
	if cond:
		print("VGONE    OK  %s" % label)
	else:
		_fails.append(label)
		print("VGONE    !!  %s   %s" % [label, detail])


func _read(path: String) -> String:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		return ""
	var s := f.get_as_text()
	f.close()
	return s


func _stash(path: String) -> void:
	_saved[path] = _read(path) if FileAccess.file_exists(path) else null
	if FileAccess.file_exists(path):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(path))


func _restore() -> void:
	for path in _saved:
		if FileAccess.file_exists(path):
			DirAccess.remove_absolute(ProjectSettings.globalize_path(path))
		if _saved[path] != null:
			var f := FileAccess.open(path, FileAccess.WRITE)
			f.store_string(String(_saved[path]))
			f.close()


## Every `Label` and `Button` string in the window, in tree order. A collapsed
## group keeps its children in the tree, so a folded "Map snapshot" is still
## readable here — which is what makes reading the rendered text a fair test
## of what the panel says rather than of what it happened to expand.
func _texts(n: Node, out: Array) -> Array:
	if n is Label:
		out.append(String((n as Label).text))
	elif n is Button:
		out.append(String((n as Button).text))
	for c in n.get_children():
		_texts(c, out)
	return out


## Counted, not `has()`: `has` would pass just as happily if a rebuild
## appended a second copy of a row instead of replacing it, and `_clear()`
## failing that way is a real shape in this shell.
func _exact(texts: Array, s: String) -> int:
	var n := 0
	for t in texts:
		if String(t) == s:
			n += 1
	return n


func _has(texts: Array, s: String) -> int:
	var n := 0
	for t in texts:
		if String(t).find(s) >= 0:
			n += 1
	return n


func _offers(tid: int, key: String) -> bool:
	for fd in _bridge.vault_export_fields("settlement", tid):
		if String((fd as Dictionary).get("key", "")) == key:
			return true
	return false


## The `path`/`missing` pair the panel branches on, read back from the engine
## so a failure says which half moved.
func _row(tid: int, radius: String) -> Dictionary:
	for r in _bridge.vault_snapshot_radii("settlement", tid):
		var d: Dictionary = r
		if String(d.get("radius", "")) == radius:
			return d
	return {}


## The three marks, counted. `"✓ ."` and not `"✕ "`-style bare marks because
## the connection section's own line is `✓ Connected — …`; a snapshot row's
## path always begins with the leading dot of `.cartalith/maps`, which is the
## default folder this probe leaves in the field.
func _marks(texts: Array) -> String:
	return "○%d ✓%d ✕%d" % [_has(texts, "○ not generated"), _has(texts, "✓ ."), _has(texts, "✕ .")]


func _ready() -> void:
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(0.8).timeout
	_bridge = _app.bridge

	## Exit 2, never 0: a `.dll` older than the shell is the condition that has
	## twice made a whole verification pass here meaningless.
	if not _bridge.world_gen.has_method("vault_snapshot_radii"):
		print("VGONE    ABORT: vault_snapshot_radii absent -- the loaded extension "
			+ "predates the snapshot surface; rebuild before believing this probe")
		get_tree().quit(2)
		return

	_stash(VaultStore.PATH)
	_stash(VaultStore.PRE_PROJECT_PATH)

	_bridge.generate({
		"seed": SEED, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().process_frame

	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vaultgone-probe"
	if DirAccess.dir_exists_absolute(_root):
		OS.move_to_trash(ProjectSettings.globalize_path(_root))
	DirAccess.make_dir_recursive_absolute(_root + "/Locations")
	var f := FileAccess.open(_root + "/Locations/Nareth.md", FileAccess.WRITE)
	f.store_string("# Nareth\n\nA river town at the third ford.\n")
	f.close()

	var conn: Dictionary = _bridge.vault_connect(_root, "GoneProbeVault")
	_ok("connect: the scratch vault binds", bool(conn.get("ok", false)), String(conn.get("error", "")))

	var settlements: Array = _bridge.settlements()
	if settlements.is_empty():
		_ok("world: settlements exist to snapshot", false)
		_finish()
		return
	var s: Dictionary = settlements[0]
	var tid := int(s.get("tid", 0))
	var sname := String(s.get("name", ""))

	_app.open_vault("settlement", tid, sname)
	await get_tree().process_frame
	## `_app.vault_window` rather than a walk for the `VaultWindow` class: the
	## app holds the instance by name, so there is nothing to search for.
	_vw = _app.vault_window
	## **A null here is "could not run", not a failed check** — exit 2, the
	## same rule this probe's `vault_snapshot_radii` guard follows. Measured
	## 2026-09-05: three runs reported *"the vault window exists"* red while a
	## concurrent lane's half-saved `shell/menus.gd` was refusing to compile
	## (`Identifier "_ap_stats_idx" not declared`), which takes `app.gd` down
	## with it and leaves `_app` scriptless. Nothing about this window was
	## wrong, and a red check said it was.
	if _vw == null:
		print("VGONE    ABORT: the shell did not build -- read the SCRIPT ERROR lines "
			+ "above, and re-run once before concluding anything about this window")
		_finish_abort()
		return

	# -- State 1: never generated -------------------------------------------
	var t1: Array = _texts(_vw, [])
	_ok("panel: the Map snapshot section is built", _has(t1, "MAP SNAPSHOT") == 1, str(t1))
	_ok("state 1: all three radii read \"not generated\"",
		_has(t1, "○ not generated") == 3, _marks(t1))
	_ok("state 1: the button says Generate, once per radius",
		_exact(t1, "Generate immediate") == 1 and _exact(t1, "Generate local") == 1
			and _exact(t1, "Generate regional") == 1,
		"%d/%d/%d" % [_exact(t1, "Generate immediate"), _exact(t1, "Generate local"),
			_exact(t1, "Generate regional")])
	_ok("state 1: nothing says Regenerate and nothing is marked gone",
		_exact(t1, "Regenerate local") == 0 and _has(t1, "✕") == 0
			and _exact(t1, "Generate local again") == 0, _marks(t1))

	# -- State 2: generated, and the image is there --------------------------
	var snap: Dictionary = _bridge.vault_snapshot("settlement", tid, "local", "", SNAP_PX)
	_ok("write: the local map is written", bool(snap.get("ok", false)), String(snap.get("error", "")))
	var rel := String(snap.get("rel", ""))
	var abs_png := _root + "/" + rel
	_ok("write: the file is on disk", FileAccess.file_exists(abs_png), abs_png)
	_ok("write: the engine reports it present -- no `missing` key",
		String(_row(tid, "local").get("path", "")) == rel and not _row(tid, "local").has("missing"),
		str(_row(tid, "local")))
	_ok("write: and the Map checkbox is offered", _offers(tid, "map_local"))

	_vw._rebuild()
	await get_tree().process_frame
	var t2: Array = _texts(_vw, [])
	_ok("state 2: the generated row ticks, and carries its path",
		_has(t2, "✓ %s" % rel) == 1, _marks(t2))
	_ok("state 2: its button says Regenerate",
		_exact(t2, "Regenerate local") == 1 and _exact(t2, "Generate local") == 0,
		"%d / %d" % [_exact(t2, "Regenerate local"), _exact(t2, "Generate local")])
	_ok("state 2: the two ungenerated radii are untouched",
		_has(t2, "○ not generated") == 2 and _exact(t2, "Generate immediate") == 1
			and _exact(t2, "Generate regional") == 1, _marks(t2))
	_ok("state 2: nothing is marked gone", _has(t2, "✕") == 0, _marks(t2))

	# -- State 3: generated, then deleted from outside Cartalith -------------
	##
	## **Behind the panel's back.** `DirAccess.remove_absolute` is what a file
	## manager does: the `LinkStore` still holds the path, so a panel reading
	## `path` alone cannot tell this apart from state 2 — which is exactly the
	## defect. The two asserts under the delete are its positive control: the
	## store must be unchanged and the file must be gone, or the state under
	## test was never reached and everything below is vacuous.
	var removed := DirAccess.remove_absolute(ProjectSettings.globalize_path(abs_png))
	_ok("delete: the PNG is removed from under the store",
		removed == OK and not FileAccess.file_exists(abs_png), "err=%d" % removed)
	_ok("delete: and nothing told the store -- the path is still filed",
		String(_row(tid, "local").get("path", "")) == rel, str(_row(tid, "local")))
	_ok("delete: the engine now flags it, with `missing` present and true",
		_row(tid, "local").has("missing") and bool(_row(tid, "local").get("missing", false)),
		str(_row(tid, "local")))
	_ok("delete: and the Map checkbox is withdrawn", not _offers(tid, "map_local"),
		"a deleted image is still offered for the note")

	_vw._rebuild()
	await get_tree().process_frame
	var t3: Array = _texts(_vw, [])
	_ok("state 3: the row no longer ticks",
		_has(t3, "✓ %s" % rel) == 0, "the deleted snapshot still reads as present")
	_ok("state 3: it is marked gone, and says so in words",
		_has(t3, "✕ %s — written before, and not in the vault now." % rel) == 1, str(t3))
	_ok("state 3: the button is not Regenerate -- there is nothing to replace",
		_exact(t3, "Regenerate local") == 0, "still offering Regenerate for a file that is gone")
	_ok("state 3: it offers to generate it again, once",
		_exact(t3, "Generate local again") == 1,
		"counted %d" % _exact(t3, "Generate local again"))
	_ok("state 3: and the two ungenerated radii still read exactly as they did",
		_has(t3, "○ not generated") == 2 and _exact(t3, "Generate immediate") == 1
			and _exact(t3, "Generate regional") == 1, _marks(t3))

	## The three renderings must be three. A panel that answered the deleted
	## case by dropping the tick everywhere would pass every check above.
	_ok("three states, three renderings",
		_has(t1, "○ not generated") == 3 and _has(t1, "✓ .") == 0 and _has(t1, "✕ .") == 0
			and _has(t2, "○ not generated") == 2 and _has(t2, "✓ .") == 1 and _has(t2, "✕ .") == 0
			and _has(t3, "○ not generated") == 2 and _has(t3, "✓ .") == 0 and _has(t3, "✕ .") == 1,
		"%s | %s | %s" % [_marks(t1), _marks(t2), _marks(t3)])

	# -- Width: the new label is the longest one in the section --------------
	##
	## *Generate local again* is four characters longer than *Regenerate
	## local*, and this window's `ScrollContainer` has its horizontal axis
	## DISABLED — the trap that folds a child's minimum into the container's
	## own and pushes it out to every ancestor with no scrollbar to reveal it.
	## Measured at three widths rather than one, because a panel's minimum is
	## content-dependent and one sample has given the wrong answer twice in
	## this shell. `_body`'s minimum must stay inside the window's own
	## `min_size.x` less the 12 px padding on each side.
	for w in [380, 560, 760]:
		_vw.size = Vector2i(w, _vw.size.y)
		_vw._rebuild()
		await get_tree().process_frame
		var min_x: float = _vw._body.get_combined_minimum_size().x
		## Printed whether or not it passes: a verdict with no number cannot be
		## compared against the next run's, and this is the figure a later
		## widening of the label has to be checked against.
		print("VGONE    ..  width @%d: body min.x = %.1f (budget %d)" % [w, min_x, w - 24])
		_ok("width @%d: the body's minimum stays inside the window" % w, min_x <= float(w - 24),
			"body min.x = %.1f against %d" % [min_x, w - 24])

	# -- Back again: regenerating clears the mark ---------------------------
	##
	## The state machine has to close. A row that latched "gone" would look
	## right in every check above and never recover.
	var again: Dictionary = _bridge.vault_snapshot("settlement", tid, "local", "", SNAP_PX)
	_ok("recover: generating again writes the same path",
		bool(again.get("ok", false)) and String(again.get("rel", "")) == rel,
		String(again.get("error", "")))
	_vw._rebuild()
	await get_tree().process_frame
	var t4: Array = _texts(_vw, [])
	_ok("recover: the row goes back to state 2 exactly",
		_has(t4, "✓ %s" % rel) == 1 and _exact(t4, "Regenerate local") == 1
			and _has(t4, "✕") == 0 and _exact(t4, "Generate local again") == 0, _marks(t4))
	_ok("recover: and the Map checkbox comes back", _offers(tid, "map_local"))

	_finish()


## "Could not run", kept apart from "ran and failed" so a broken tree cannot
## be mistaken for a defect in the panel — and cannot be mistaken for a pass
## either, which is why it is 2 and not 0.
func _finish_abort() -> void:
	_cleanup()
	get_tree().quit(2)


func _cleanup() -> void:
	if _root != "" and DirAccess.dir_exists_absolute(_root):
		OS.move_to_trash(ProjectSettings.globalize_path(_root))
	_restore()


func _finish() -> void:
	_cleanup()
	if _fails.is_empty():
		print("VGONE    ALL CHECKS PASSED")
	else:
		print("VGONE    %d FAILED: %s" % [_fails.size(), ", ".join(PackedStringArray(_fails))])
	get_tree().quit(0 if _fails.is_empty() else 1)

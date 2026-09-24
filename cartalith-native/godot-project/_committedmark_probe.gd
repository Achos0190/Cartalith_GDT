extends Node
## HISTORY's `COMMITTED` boundary **across a process boundary** -- the third
## clause of `design/proposed-2026-09-05/Main.dc.html`'s footnote, and the one
## case the shell could not place until 2026-09-06: *a project saved in an
## earlier session*.
##
## ## Why this probe is two processes and not two phases of one
##
## The claim under test is that the boundary is readable from the **file**, not
## from anything this session remembers. A single-process test cannot refute
## that: the session that saved is the session that reads back, so a boundary
## carried in a member variable passes it. So the harness runs this scene
## twice, and the second run is a different process with a different
## `EngineBridge`, a different `RightDock` and a different world:
##
##     Godot_v4.7.1-stable_win64_console.exe --path . _committedmark_probe.tscn -- --phase save
##     touch -d '3 days ago' <the .zip>          # see "the age is the file's" below
##     Godot_v4.7.1-stable_win64_console.exe --path . _committedmark_probe.tscn -- --phase open
##
## and a third phase over an archive **an earlier build wrote**, which is the
## backward-compatibility case (`--archive` accepts `res://`, `user://` or a
## native path):
##
##     ... _committedmark_probe.tscn -- --phase legacy --archive res://../crates/cartalith-io/tests/fixtures/real_export_seed24601.zip
##
## `--phase` and `--archive` are the only arguments this file reads
## (`_phase()`), and an argument it does not understand **aborts** rather than
## defaulting -- a probe that quietly runs the wrong phase reports a pass for a
## test it never performed.
##
## **Windowed, not `--headless`.** Every assertion below reads a `Label` that
## the dock drew, and the dock is only laid out and drawn with a real display.
##
## ## The age is the file's, and the harness proves it by moving the file
##
## Between the two runs the harness back-dates the archive's mtime. If the age
## readout came from this process's clock it would still say *"saved just
## now"*; it says *"saved 3 d ago"*, and `saved_at_ms` matches Godot's own
## `FileAccess.get_modified_time()` -- an oracle read on this side of the
## boundary, not the engine's number compared against itself.
##
## ## What each phase asserts
##
## | `--phase save` | |
## |---|---|
## | before any save | **no** COMMITTED rule; `undo_stats()` omits `saved_seq` and `saved_at_ms`; the note names *never saved* |
## | after `save_project()` | the rule is drawn; `saved_seq` is the newest ledger row |
## | one edit later | the rule sits between two rows |
## | reverting above it | the rule survives (the negative control) |
## | reverting *to* it | the rule goes, and the note names the revert -- not "never saved" |
##
## | `--phase open` | |
## |---|---|
## | fresh session, generated world | no rule again -- this process has saved nothing |
## | after opening the archive | the rule **is** drawn, at the `Open project` floor, with the file's own age |
## | `saved_at_ms` | equals Godot's own `FileAccess.get_modified_time()` |
## | the ledger itself | did **not** survive the reload -- one floor row is all there is |
## | the confirmation's predicate | true at the boundary row, false above it |
##
## **The invalidation half is in phase SAVE**, where it was written when a
## reopened project refused every height edit. Since owner Ruling AR
## (2026-09-24) a project saved with its world substrate (`SAVEFILE_COMPAT.md`
## §8.3) reopens as the complete world and takes one, so phase OPEN now also
## makes the edit across the process boundary: `carve_fjords()` is accepted,
## records one ledger row above the `Open project` floor, and leaves
## `saved_seq` where the file put it.
##
## | `--phase legacy --archive <path>` | |
## |---|---|
## | an archive an **earlier build** wrote | the rule is drawn, at that load's floor, with that file's own mtime |
##
## Nothing was added to the save format for this boundary, which is what makes
## that phase's claim strong rather than circular: the fixture is the previous
## writer's own output and it needs no field the previous writer never emitted.

var _app: Node
var _bridge: Node
var _rd: Node
var _fail := 0
var _phase_name := ""
var _archive := ""

const ZIP := "user://_committedmark_probe.zip"

func _p(s: String) -> void:
	print("CMK %s" % s)

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("CMK %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## `--phase save` / `--phase open`, and nothing else. An unrecognised argument
## is fatal on purpose (`MISTAKES.md`: a usage header is a claim about the
## body, and a default that swallows a typo measures the same thing twice).
func _phase() -> String:
	var args := OS.get_cmdline_user_args()
	var want := ""
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--phase" and i + 1 < args.size():
			want = String(args[i + 1])
			i += 2
			continue
		if a == "--archive" and i + 1 < args.size():
			_archive = String(args[i + 1])
			i += 2
			continue
		_p("!! unrecognised argument '%s' -- this probe reads `--phase save|open|legacy`"
			% a + " and `--archive <path>` (legacy only)")
		return ""
	if want != "save" and want != "open" and want != "legacy":
		_p("!! --phase must be `save`, `open` or `legacy`, got '%s'" % want)
		return ""
	if want == "legacy" and _archive == "":
		_p("!! --phase legacy needs `--archive <path>` -- an archive written by an "
			+ "EARLIER build, not one this run produced")
		return ""
	return want

# -- Reading the drawn dock back ----------------------------------------------

func _walk(n: Node, out: Array) -> void:
	for c in n.get_children():
		out.append(c)
		_walk(c, out)

func _labels() -> Array:
	var all: Array = []
	_walk(_app.right_dock_body, all)
	var out: Array = []
	for n in all:
		if n is Label:
			out.append(n)
	return out

func _texts() -> Array:
	var out: Array = []
	for l in _labels():
		out.append((l as Label).text)
	return out

func _has_text(t: String) -> bool:
	return _texts().has(t)

## The trailing readout beside the rule -- any drawn label beginning `saved `.
## Returned rather than asserted so the caller can say what it found.
func _age_text() -> String:
	for t in _texts():
		if String(t).begins_with("saved "):
			return String(t)
	return ""

## The panel's own explanation for an absent rule, or "" when it drew none.
func _no_rule_note() -> String:
	for t in _texts():
		if String(t).find("No COMMITTED rule") >= 0:
			return String(t)
	return ""

func _history() -> void:
	_rd.show_history()
	await _frames(6)

func _stats() -> Dictionary:
	return _bridge.undo_stats()

func _newest_seq() -> int:
	var rows: Array = _bridge.undo_ledger()
	if rows.is_empty():
		return -1
	return int((rows[rows.size() - 1] as Dictionary).get("seq", -1))

func _generate(seed: int) -> void:
	_bridge.generate({
		"seed": seed, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout

# -- The never-saved case, asserted the same way in both phases ---------------

## Shared because it is the same claim in two processes: **absent is absent**.
## Both keys are missing rather than zero, no rule is drawn, and the sentence
## the panel gives names *this* reason and not the revert one.
func _assert_never_saved(where: String) -> void:
	var st := _stats()
	_check("%s: undo_stats() omits saved_seq entirely" % where,
		not st.has("saved_seq"), "stats keys=%s" % str(st.keys()))
	_check("%s: and omits saved_at_ms" % where, not st.has("saved_at_ms"))
	_check("%s: saved_reverted_past is false -- never saved is not reverted past" % where,
		not bool(st.get("saved_reverted_past", true)))
	_check("%s: no COMMITTED rule is drawn" % where, not _has_text("COMMITTED"),
		"texts=%s" % str(_texts().slice(0, 12)))
	var note := _no_rule_note()
	_check("%s: the panel says why there is none" % where, note != "")
	_check("%s: and the reason is 'has not been saved', not a revert" % where,
		note.find("has not been saved") >= 0 and note.find("revert") < 0,
		"note='%s'" % note)

# ================================================================ phases =====

func _phase_save() -> void:
	_p("--- phase SAVE (writes the archive the second process reads) ---")
	await _generate(4471)
	await _history()
	_assert_never_saved("fresh world")

	## One real edit above the floor, so the archive carries a two-row history
	## and the rule has somewhere to sit that is not the bottom of the list.
	var r: Dictionary = _bridge.carve_fjords()
	_p("carve_fjords -> %s" % str(r))
	await _frames(4)
	await _history()

	var path := ProjectSettings.globalize_path(ZIP)
	var ok: bool = _bridge.save_project(path)
	_p("save_project('%s') -> %s" % [path, ok])
	if not ok:
		_check("the save the whole probe depends on succeeded", false, path)
		return
	await _frames(6)
	await _history()

	var st := _stats()
	var newest := _newest_seq()
	_check("a save draws the COMMITTED rule", _has_text("COMMITTED"),
		"texts=%s" % str(_texts().slice(0, 12)))
	_check("saved_seq is present after a save", st.has("saved_seq"),
		"stats keys=%s" % str(st.keys()))
	_check("and it is the newest ledger row", int(st.get("saved_seq", -1)) == newest,
		"saved_seq=%s newest=%d" % [str(st.get("saved_seq", "(absent)")), newest])
	_check("saved_at_ms is present after a save", st.has("saved_at_ms"))
	_check("the age readout is drawn", _age_text() != "", "age='%s'" % _age_text())
	_p("archive written: %s  (mtime %d)" % [path, FileAccess.get_modified_time(path)])

	# -- invalidation, here rather than in phase OPEN, and for a real reason --
	#
	# `carve_fjords()` is the cheapest operation that pushes a height snapshot,
	# and it **refuses a loaded world**: `WorldGen::carve_fjords` matches
	# `WorldSource::Generated` and otherwise returns
	# `"Carve fjords needs a generated world; a loaded save carries no
	# lithology inputs (crust_field/age_field) to derive the mask from."`
	# So the boundary's *invalidation* rules are exercised on this side, where
	# a generated world can still be edited; phase OPEN measures the placement,
	# which is the half that needed a second process at all.
	var saved_seq := int(st.get("saved_seq", -1))
	var r2: Dictionary = _bridge.carve_fjords()
	_p("post-save carve -> %s" % str(r2))
	await _frames(4)
	await _history()
	var above := _newest_seq()
	_check("an edit after a save lands above the rule", above > saved_seq,
		"newest=%d saved_seq=%d" % [above, saved_seq])
	_check("and the rule is drawn BETWEEN the two rows", _has_text("COMMITTED"),
		"texts=%s" % str(_texts().slice(0, 12)))

	## Negative control first: unwinding only the unsaved row leaves the file's
	## own state exactly where it was, so the rule must survive.
	var n_above: int = _bridge.undo_revert_to(above)
	_p("undo_revert_to(%d) [above the rule] -> %d steps" % [above, n_above])
	await _frames(6)
	await _history()
	_check("a revert above the rule keeps it",
		_has_text("COMMITTED") and int(_stats().get("saved_seq", -1)) == saved_seq,
		"saved_seq=%s drawn=%s" % [str(_stats().get("saved_seq", "(absent)")),
			_has_text("COMMITTED")])

	## ...and reverting *to* the boundary row unwinds the row the file holds.
	_check("the confirmation counts this as reverting past COMMITTED (the <= fix)",
		_rd._reverts_past_committed(saved_seq), "saved_seq=%d" % saved_seq)
	var n_at: int = _bridge.undo_revert_to(saved_seq)
	_p("undo_revert_to(%d) [the boundary row itself] -> %d steps" % [saved_seq, n_at])
	await _frames(6)
	await _history()
	var st3 := _stats()
	_check("reverting past the rule removes it", not st3.has("saved_seq"),
		"saved_seq=%s" % str(st3.get("saved_seq", "(absent)")))
	_check("no COMMITTED rule is drawn afterwards", not _has_text("COMMITTED"),
		"texts=%s" % str(_texts().slice(0, 12)))
	_check("saved_reverted_past is true", bool(st3.get("saved_reverted_past", false)))
	var note := _no_rule_note()
	_check("and the note blames the revert, not a project that was never saved",
		note.find("revert") >= 0 and note.find("has not been saved") < 0,
		"note='%s'" % note)
	## The archive on disk is untouched by any of that -- it was written before
	## this block ran, and phase OPEN reads it.
	_check("the archive phase OPEN reads is still on disk", FileAccess.file_exists(path))

func _phase_open() -> void:
	_p("--- phase OPEN (a different process; nothing here saved anything) ---")
	var path := ProjectSettings.globalize_path(ZIP)
	if not FileAccess.file_exists(path):
		_check("the archive phase SAVE wrote is on disk", false, path)
		return

	## A world of this session's own, so the ledger below is not empty and the
	## "never saved" claim is about a session that really has a history.
	await _generate(9182)
	await _history()
	_assert_never_saved("fresh session, before the open")

	## Godot's own stat of the same file -- the independent oracle for the age.
	## Seconds; the engine reports milliseconds.
	var mtime_s := FileAccess.get_modified_time(path)
	var now_s := int(Time.get_unix_time_from_system())
	_p("file mtime=%d  now=%d  (%d s old)" % [mtime_s, now_s, now_s - mtime_s])

	var ok: bool = _bridge.load_save(path)
	_p("load_save('%s') -> %s  layout=%s" % [path, ok, _bridge.last_open_layout])
	if not ok:
		_check("the archive opened", false, path)
		return
	await _frames(8)
	await _history()

	var st := _stats()
	var rows: Array = _bridge.undo_ledger()
	var floor_seq := int((rows[0] as Dictionary).get("seq", -1)) if not rows.is_empty() else -1
	_p("after the open: ledger=%d rows, first='%s', stats saved_seq=%s saved_at_ms=%s" % [
		rows.size(),
		String((rows[0] as Dictionary).get("label", "?")) if not rows.is_empty() else "-",
		str(st.get("saved_seq", "(absent)")), str(st.get("saved_at_ms", "(absent)"))])

	# -- THE ROW: a project saved in an earlier session places the boundary ----
	_check("REOPEN: the COMMITTED rule is drawn for a file this session never wrote",
		_has_text("COMMITTED"), "texts=%s" % str(_texts().slice(0, 12)))
	_check("REOPEN: no 'no rule' note is drawn beside it", _no_rule_note() == "",
		"note='%s'" % _no_rule_note())
	_check("REOPEN: saved_seq is present", st.has("saved_seq"),
		"stats keys=%s" % str(st.keys()))
	_check("REOPEN: it sits at the Open project floor", int(st.get("saved_seq", -1)) == floor_seq,
		"saved_seq=%s floor=%d" % [str(st.get("saved_seq", "(absent)")), floor_seq])
	_check("REOPEN: the ledger did NOT survive the reload -- one floor row is all there is",
		rows.size() == 1, "%d rows: %s" % [rows.size(), str(rows)])

	# -- the age comes from the file, measured against Godot's own stat --------
	_check("REOPEN: saved_at_ms is present", st.has("saved_at_ms"))
	var at_ms := int(st.get("saved_at_ms", 0))
	_check("REOPEN: saved_at_ms matches FileAccess.get_modified_time to the second",
		absi(int(at_ms / 1000) - mtime_s) <= 1,
		"engine=%d s, Godot=%d s" % [int(at_ms / 1000), mtime_s])
	var age := _age_text()
	_p("age readout: '%s'" % age)
	_check("REOPEN: an age is drawn", age != "")
	## The harness back-dates the archive between the two runs, so a readout
	## sourced from this process's clock would say "just now". This is the
	## whole cross-session claim in one string.
	if now_s - mtime_s > 86400:
		_check("REOPEN: the age is the FILE's, not this session's",
			age.find(" d ago") >= 0, "age='%s' for a file %d s old" % [age, now_s - mtime_s])
	else:
		_p("   (archive is < 1 day old -- run the harness's back-dating step to test that)")

	# -- the confirmation's own condition, both sides of the boundary ---------
	##
	## A pure predicate, so it needs no edit to exercise. Phase SAVE exercises
	## the invalidation half; the edit below is the same half across the
	## process boundary, which a reopened project can take since Ruling AR.
	var saved_seq := int(st.get("saved_seq", -1))
	_check("reverting TO the boundary row counts as past it (the <= fix)",
		_rd._reverts_past_committed(saved_seq), "saved_seq=%d" % saved_seq)
	_check("reverting above it does not",
		not _rd._reverts_past_committed(saved_seq + 1), "saved_seq=%d" % saved_seq)
	var carved: Dictionary = _bridge.carve_fjords()
	_p("carve_fjords() on the reopened project -> %s" % str(carved))
	await _frames(4)
	await _history()
	_check("a reopened project (saved with its substrate) takes a height edit",
		bool(carved.get("ok", false)), str(carved))
	_check("the edit is one ledger row above the floor, and the rule stays at the saved row",
		_newest_seq() > floor_seq and int(_stats().get("saved_seq", -1)) == saved_seq,
		"newest=%d floor=%d saved_seq=%s" % [_newest_seq(), floor_seq,
			str(_stats().get("saved_seq", "(absent)"))])

func _phase_legacy() -> void:
	## **The backward-compatibility case, with the previous writer's own
	## output.** `--archive` is a file some earlier build wrote -- this port's
	## tree writer from a previous session, or a genuine HTML-app export from
	## `crates/cartalith-io/tests/fixtures/` that this port never wrote at all.
	##
	## Nothing was added to the save format for the `COMMITTED` boundary, so
	## the claim being tested is the strong one: an archive written before the
	## boundary existed places it correctly, because the placement reads the
	## floor row this load creates and the age reads the file's own mtime.
	_p("--- phase LEGACY (an archive an earlier build wrote) ---")
	var path := ProjectSettings.globalize_path(_archive) if _archive.begins_with("res://") 		or _archive.begins_with("user://") else _archive
	if not FileAccess.file_exists(path):
		_check("the legacy archive exists", false, path)
		return
	var mtime_s := FileAccess.get_modified_time(path)
	var now_s := int(Time.get_unix_time_from_system())
	_p("archive=%s  mtime=%d  (%d s / %.1f d old)"
		% [path, mtime_s, now_s - mtime_s, float(now_s - mtime_s) / 86400.0])

	await _generate(2024)
	await _history()
	_assert_never_saved("fresh session, before the legacy open")

	var ok: bool = _bridge.load_save(path)
	_p("load_save -> %s  layout=%s  warnings=%s"
		% [ok, _bridge.last_open_layout, str(_bridge.last_open_warnings)])
	if not ok:
		_check("the legacy archive opened", false, path)
		return
	await _frames(8)
	await _history()

	var st := _stats()
	var rows: Array = _bridge.undo_ledger()
	var floor_seq := int((rows[0] as Dictionary).get("seq", -1)) if not rows.is_empty() else -1
	_check("LEGACY: the COMMITTED rule is drawn for an archive written before it existed",
		_has_text("COMMITTED"), "texts=%s" % str(_texts().slice(0, 12)))
	_check("LEGACY: saved_seq sits at the Open project floor",
		st.has("saved_seq") and int(st.get("saved_seq", -1)) == floor_seq,
		"saved_seq=%s floor=%d" % [str(st.get("saved_seq", "(absent)")), floor_seq])
	_check("LEGACY: saved_at_ms is the file's own mtime",
		st.has("saved_at_ms") and absi(int(int(st.get("saved_at_ms", 0)) / 1000) - mtime_s) <= 1,
		"engine=%s ms, Godot=%d s" % [str(st.get("saved_at_ms", "(absent)")), mtime_s])
	_p("age readout: '%s'" % _age_text())
	_check("LEGACY: an age is drawn", _age_text() != "")
	_check("LEGACY: and it is not 'just now' for a file days old",
		(now_s - mtime_s) < 86400 or _age_text().find("just now") < 0,
		"age='%s' for a file %d s old" % [_age_text(), now_s - mtime_s])

func _ready() -> void:
	_phase_name = _phase()
	if _phase_name == "":
		get_tree().quit(2)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await _frames(3)
	_bridge = _app.bridge
	_rd = _app.right_dock_ctrl

	## Every number below is a text, not a pixel, so the palette does not gate
	## it -- but the density does gate the layout the dock builds, so it is
	## named beside the results rather than left to be guessed.
	_p("phase=%s  window=%s  dark=%s  tablet=%s" % [
		_phase_name, str(DisplayServer.window_get_size()),
		DccTheme.is_dark(), DccTheme.is_tablet()])
	_p("headless=%s (pixel-free probe, but the dock still has to be laid out)"
		% str(DisplayServer.get_name() == "headless"))

	if _phase_name == "save":
		await _phase_save()
	elif _phase_name == "legacy":
		await _phase_legacy()
	else:
		await _phase_open()

	print("CMK %s -- %d failure%s" % [
		"PASS" if _fail == 0 else "FAIL", _fail, "" if _fail == 1 else "s"])
	get_tree().quit(0 if _fail == 0 else 1)

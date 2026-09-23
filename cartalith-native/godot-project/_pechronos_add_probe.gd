extends Node
## SP-3 / Ruling AM write path: the Settlement Editor's "Add event" form must
## compose an exact Chronos line and write it into the settlement's vault note,
## driven through the real input fields and the real Add event button.
##   1. a note with NO ```chronos block gets one created, holding the line;
##   2. a second add appends into that same block (still one block);
##   3. the unmodified parser (vault_entity_chronos) reads both back exactly;
##   4. the drawn list shows them after the add;
##   5. a note edited on disk after the list was drawn refuses the add (nothing
##      written, a conflict message drawn), and the retry then succeeds;
##   6. a missing name is refused with a message, nothing written.
## Windowed (reads real drawn Label text):
##   Godot_v4.7.1-stable_win64_console.exe --path . _pechronos_add_probe.tscn
## Exit 0 pass, 1 real failure, 2 could not run.

const VAULT_DIR := "user://_pechronos_add_vault"
const NOTE := VAULT_DIR + "/Settlements/Note.md"
var _app: Node
var _bridge
var _fail := 0


func _p(s: String) -> void:
	print("PEADD  %s" % s)


func _check(name: String, cond: bool, detail: String = "") -> void:
	_p("%s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _labels(n: Node, out: Array) -> Array:
	if n is Label:
		out.append((n as Label).text)
	for c in n.get_children(true):
		_labels(c, out)
	return out


func _has_sub(labels: Array, sub: String) -> int:
	for i in labels.size():
		if String(labels[i]).find(sub) >= 0:
			return i
	return -1


func _find(n: Node, pred: Callable) -> Node:
	if pred.call(n):
		return n
	for c in n.get_children(true):
		var r := _find(c, pred)
		if r != null:
			return r
	return null


func _press_tab(pe: Node, text: String) -> void:
	var b = _find(pe, func(n): return n is Button and (n as Button).text == text)
	if b != null:
		(b as Button).pressed.emit()


func _read_note() -> String:
	return FileAccess.get_file_as_string(NOTE)


## Types into the real form: the LineEdit found by its placeholder, the Year
## SpinBox inside the same "Add event" group, then presses the real button.
func _fill_and_add(pe: Node, year: int, fields: Dictionary) -> void:
	var btn: Button = _find(pe, func(n): return n is Button and (n as Button).text == "Add event")
	if btn == null:
		_check("Add event button present", false)
		return
	var form: Node = btn.get_parent().get_parent()
	var sb: SpinBox = _find(form, func(n): return n is SpinBox)
	sb.value = year
	for ph in fields:
		var le: LineEdit = _find(form, func(n): return n is LineEdit and (n as LineEdit).placeholder_text == ph)
		if le == null:
			_check("field '%s' present" % ph, false)
			continue
		le.text = String(fields[ph])
		le.text_changed.emit(le.text)
	btn.pressed.emit()
	await _frames(6)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	if not _bridge.world_gen.has_method("vault_add_chronos_event"):
		_p("ABORT: no vault_add_chronos_event on this binary -- stale .dll?")
		get_tree().quit(2)
		return

	_bridge.generate({"seed": 5521, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)

	var places: Array = _bridge.settlements()
	var idx := -1
	for i in places.size():
		if int((places[i] as Dictionary).get("tid", 0)) > 0:
			idx = i
			break
	if idx == -1:
		_p("ABORT: no settlement with a tid")
		get_tree().quit(2)
		return
	var s: Dictionary = places[idx]
	var tid := int(s.get("tid", 0))
	var name := String(s.get("name", "?"))
	_p("settlement idx=%d tid=%d name=%s" % [idx, tid, name])

	# A real note with no chronos block at all.
	DirAccess.make_dir_recursive_absolute(VAULT_DIR + "/Settlements")
	var original := "# %s\n\nA river town.\n\n## History\n\nOlder than the walls.\n" % name
	var f := FileAccess.open(NOTE, FileAccess.WRITE)
	f.store_string(original)
	f.close()
	var info: Dictionary = _bridge.vault_connect(ProjectSettings.globalize_path(VAULT_DIR), "Probe vault")
	_check("vault connected", bool(info.get("ok", false)), String(info.get("error", "")))
	var att: Dictionary = _bridge.vault_attach("settlement", tid, name, "Settlements/Note.md", "")
	_check("note attached", bool(att.get("ok", false)), String(att.get("error", "")))

	var pe = _app.place_editor_window
	pe.open_for(idx)
	await _frames(6)
	_press_tab(pe, "Political history")
	await _frames(6)
	var l0 := _labels(pe, [])
	_check("no-events state drawn", _has_sub(l0, "has no ```chronos block") >= 0)

	# -- 1. first add: creates the block -------------------------------------
	await _fill_and_add(pe, -250, {"blank = a single year": "-100", "required": "The long siege",
		"optional": "walls held", "optional: red, blue, ff8800": "#red"})
	# "optional" matches Description, the first field with that placeholder;
	# Group shares it, so step 2 fills Group by position.
	var t1 := _read_note()
	_p("note after add 1:\n" + t1)
	_check("original text kept byte-for-byte as a prefix", t1.begins_with(original))
	_check("a chronos block was created", t1.count("```chronos") == 1)
	_check("exact Chronos line written",
		t1.find("\n- [-250~-100] #red The long siege | walls held\n") >= 0)

	# -- 2. second add, with a group: appended into the same block -----------
	var btn: Button = _find(pe, func(n): return n is Button and (n as Button).text == "Add event")
	var form: Node = btn.get_parent().get_parent()
	var les: Array = []
	var stack: Array = [form]
	while not stack.is_empty():
		var n: Node = stack.pop_front()
		if n is LineEdit and not (n.get_parent() is SpinBox):
			les.append(n)
		stack.append_array(n.get_children(true))
	_check("five text fields in the form (end, name, description, colour, group)", les.size() == 5, str(les.size()))
	var cleared := les.size() == 5 and (les[1] as LineEdit).text == "" and (les[2] as LineEdit).text == ""
	_check("name and description cleared after a successful add", cleared)
	if les.size() == 5:
		(les[4] as LineEdit).text = "Charters"
		(les[4] as LineEdit).text_changed.emit("Charters")
	await _fill_and_add(pe, 1200, {"required": "Charter granted"})
	var t2 := _read_note()
	_check("still exactly one block", t2.count("```chronos") == 1)
	_check("second line appended after the first, inside the block",
		t2.find("- [-250~-100] #red The long siege | walls held\n- [1200] #red {Charters} Charter granted\n```") >= 0,
		t2)

	# -- 3. the unmodified parser reads both back ----------------------------
	var got: Dictionary = _bridge.vault_entity_chronos("settlement", tid)
	var evs: Array = got.get("events", [])
	_check("parser reads 2 events", evs.size() == 2, str(evs.size()))
	if evs.size() == 2:
		var a: Dictionary = evs[0]
		_check("event 1 fields", int(a.get("start")) == -250 and int(a.get("end")) == -100
			and a.get("color") == "red" and a.get("name") == "The long siege"
			and a.get("description") == "walls held" and not a.has("group"), str(a))
		var b: Dictionary = evs[1]
		_check("event 2 fields", int(b.get("start")) == 1200 and not b.has("end")
			and b.get("group") == "Charters" and b.get("name") == "Charter granted"
			and not b.has("description"), str(b))
	var skipped: int = (got.get("notes", [])[0] as Dictionary).get("skipped", PackedStringArray()).size()
	_check("no skipped lines", skipped == 0)

	# -- 4. the drawn list shows them ----------------------------------------
	var l1 := _labels(pe, [])
	_check("range drawn", l1.has("-250 – -100"))
	_check("first event drawn", _has_sub(l1, "The long siege") >= 0)
	_check("grouped event drawn", _has_sub(l1, "Charters · Charter granted") >= 0)
	_check("success message drawn", _has_sub(l1, "Added \"Charter granted\"") >= 0)
	get_viewport().get_texture().get_image().save_png("user://_pechronos_add_after.png")
	_p("screenshot %s" % ProjectSettings.globalize_path("user://_pechronos_add_after.png"))

	# -- 5. concurrent edit: refused, then the retry lands --------------------
	var theirs := t2.replace("Older than the walls.", "Older than the walls, says the author.")
	f = FileAccess.open(NOTE, FileAccess.WRITE)
	f.store_string(theirs)
	f.close()
	await _fill_and_add(pe, 1300, {"required": "Ghost event"})
	_check("conflicting add wrote nothing", _read_note() == theirs)
	var l2 := _labels(pe, [])
	_check("conflict message drawn", _has_sub(l2, "changed outside Cartalith") >= 0)
	btn = _find(pe, func(n): return n is Button and (n as Button).text == "Add event")
	btn.pressed.emit()
	await _frames(6)
	var t3 := _read_note()
	_check("retry after the re-read lands", t3.find("- [1300] #red {Charters} Ghost event\n```") >= 0)
	_check("the author's concurrent edit survived", t3.find("says the author.") >= 0)

	# -- 6. no name: refused with a message ----------------------------------
	await _fill_and_add(pe, 5, {"required": ""})
	_check("nameless add wrote nothing", _read_note() == t3)
	_check("nameless add refused with a message", _has_sub(_labels(pe, []), "Not added") >= 0)

	_bridge.vault_disconnect()
	DirAccess.remove_absolute(ProjectSettings.globalize_path(NOTE))
	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

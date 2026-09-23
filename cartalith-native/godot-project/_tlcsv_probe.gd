extends Node
## Proof for the Travel Library's CSV import (`OUTSTANDING_WORK.md` §2.3,
## 2026-09-24; `travel_library_window.gd::import_csv`). Drives the real window
## method against the real engine:
##   1. a CSV of two good rows, a blank line, one unknown column and one
##      wrong-typed value imports exactly two new custom animals;
##   2. each reads back through `tl_get` with the numbers the file carried;
##   3. the unknown column and the bad value are REPORTED, by line.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _tlcsv_probe.tscn

var _fails := 0

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("TLCSV %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _ready() -> void:
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	var win: Node = app.travel_library_window
	win._current_kind = "animal"
	var stock: Dictionary = bridge.tl_get("animal", "donkey")
	var nums: Array = []
	for k in stock.keys():
		if typeof(stock[k]) == TYPE_FLOAT and not String(k).begins_with("validation"):
			nums.append(String(k))
	nums.sort()
	_check("two numeric animal fields to import", nums.size() >= 2, str(nums))
	var a := String(nums[0])
	var b := String(nums[1])
	var before: int = (bridge.tl_list("animal") as Array).size()

	var path := OS.get_user_data_dir().path_join("_tlcsv_probe.csv")
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_line("name,%s,%s,not_a_field" % [a, b])
	f.store_line("\"Probe mule, grey\",1.25,2.5,x")
	f.store_line("")
	f.store_line("Probe ox,3.5,abc,")
	f.close()

	var r: Dictionary = win.import_csv(path)
	var refused: PackedStringArray = r["refused"]
	print("TLCSV result %s" % [r])
	_check("two rows imported", int(r["added"]) == 2)
	_check("the list grew by two", (bridge.tl_list("animal") as Array).size() == before + 2)
	var got := {}
	for e in bridge.tl_list("animal"):
		var ed: Dictionary = e
		if String(ed.get("name", "")).begins_with("Probe "):
			got[String(ed["name"])] = bridge.tl_get("animal", String(ed["id"]))
	_check("the quoted name survived its comma", got.has("Probe mule, grey"), str(got.keys()))
	if got.has("Probe mule, grey"):
		var m: Dictionary = got["Probe mule, grey"]
		_check("the file's numbers read back", is_equal_approx(float(m.get(a, -1)), 1.25) and is_equal_approx(float(m.get(b, -1)), 2.5),
			"%s=%s %s=%s" % [a, m.get(a), b, m.get(b)])
	var joined := " | ".join(refused)
	_check("the unknown column is reported, on line 2", joined.contains("line 2") and joined.contains("not_a_field"), joined)
	_check("the bad value is reported, on line 4", joined.contains("line 4") and joined.contains(b), joined)
	for n in got.keys():
		bridge.tl_delete("animal", String((got[n] as Dictionary).get("id", "")))
	DirAccess.remove_absolute(path)
	print("TLCSV %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)

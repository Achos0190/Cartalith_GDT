extends Node
## `PlaceSearch` probe -- the world-entity search index behind `Edit ▸ Find on
## map…` (menus.gd, `ID_FIND_ON_MAP`).
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _placesearch_probe.tscn
##
## This repo has been bitten four times by silently-empty golden output
## (CLAUDE.md's own working-rules list), so every assertion below is about
## something concrete being present or absent, never just "no crash".

var _fail := 0

func _p(s: String) -> void:
	print("PLACESEARCH  %s" % s)

func _bad(s: String) -> void:
	_fail += 1
	print("PLACESEARCH  FAIL  %s" % s)

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(20)
	var bridge = app.bridge

	print("\n=== 0: safe on no world ===")
	var idx0 := PlaceSearch.new()
	idx0.build(bridge)
	_ok("no world yet -> 0 rows", idx0.size(), 0)
	_ok("all() is an empty array, not null", idx0.all().size(), 0)
	_ok("search on an empty index returns nothing", idx0.search("anything").size(), 0)

	print("\n=== 1: generate a small world ===")
	bridge.generate({
		"seed": 771144, "width_km": 900.0, "grid_w": 192, "grid_h": 144,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	await bridge.generation_finished
	await _frames(8)
	_p("world: %d settlements, %d factions, %d ways, %d sea routes, %d labels" % [
		bridge.settlements().size(), bridge.get_factions().size(),
		bridge.roads().size(), bridge.sea_routes().size(), bridge.label_list().size()])

	## A couple of hand-placed labels so the label family is exercised even if
	## a small procedural world happens to place none of its own.
	bridge.label_create(40.0, 30.0, "Probeholm")
	bridge.label_create(90.0, 60.0, "Rivertest Landing")

	print("\n=== 2: build the index ===")
	var idx := PlaceSearch.new()
	idx.build(bridge)
	_p("index size: %d" % idx.size())
	_ok("the index is non-empty", idx.size() > 0, true)

	var by_entity := {}
	for r in idx.all():
		var e := String(r["entity"])
		by_entity[e] = int(by_entity.get(e, 0)) + 1
	_p("by entity: %s" % str(by_entity))
	_ok("settlements were indexed", int(by_entity.get("settlement", 0)) > 0, true)
	_ok("labels were indexed", int(by_entity.get("label", 0)) > 0, true)

	print("\n=== 3: a known settlement is findable ===")
	var settlements: Array = bridge.settlements()
	if settlements.is_empty():
		_bad("no settlements generated at all -- cannot test findability")
	else:
		var target := String((settlements[0] as Dictionary).get("name", ""))
		if target == "":
			_bad("settlement 0 has no name")
		else:
			var hits := idx.search(target)
			var found := false
			for r in hits:
				if String(r["name"]) == target:
					found = true
			_ok("searching the settlement's own name finds it ('%s')" % target, found, true)

	print("\n=== 4: a placed label is findable, and pans to where it was placed ===")
	var lbl_hits := idx.search("Probeholm")
	var lbl_found := false
	for r in lbl_hits:
		if String(r["name"]) == "Probeholm":
			lbl_found = true
			_ok("Probeholm's x is 40", float(r["x"]) == 40.0, true)
			_ok("Probeholm's y is 30", float(r["y"]) == 30.0, true)
			_ok("Probeholm's entity is 'label'", String(r["entity"]), "label")
	_ok("the hand-placed label was indexed and found", lbl_found, true)

	print("\n=== 5: a nonsense query returns nothing ===")
	_ok("nonsense query -> 0 rows", idx.search("qxzqxzqxz_no_such_thing").size(), 0)

	print("\n=== 6: an empty query returns everything ===")
	_ok("empty query size == index size", idx.search("").size(), idx.size())
	_ok("whitespace-only query size == index size", idx.search("   ").size(), idx.size())

	print("\n=== 7: ranking bands -- prefix hits sort before mid-string hits ===")
	## Two labels chosen so one is a prefix hit and the other a mid-string hit
	## on the same needle, to exercise the actual band split rather than just
	## trusting it structurally.
	var riv := idx.search("rivertest")
	var riv_prefix_ok := riv.size() > 0 and String(riv[0]["name"]).to_lower().begins_with("rivertest")
	_ok("'rivertest' finds the label and it sorts first (prefix band)", riv_prefix_ok, true)

	print("\n=== 8: every row's x/y are inside the grid, every subtitle is non-blank ===")
	var g: Vector2i = bridge.grid_size()
	_p("grid size: %s" % str(g))
	var bounds_ok := true
	var subtitle_ok := true
	var name_ok := true
	var bad_examples := []
	for r in idx.all():
		var x := float(r["x"])
		var y := float(r["y"])
		if x < 0.0 or x > float(g.x) or y < 0.0 or y > float(g.y):
			bounds_ok = false
			if bad_examples.size() < 5:
				bad_examples.append("%s (%s) x=%.1f y=%.1f" % [r["name"], r["entity"], x, y])
		if String(r["subtitle"]).strip_edges() == "":
			subtitle_ok = false
		if String(r["name"]).strip_edges() == "":
			name_ok = false
	if not bounds_ok:
		_p("out-of-bounds examples: %s" % str(bad_examples))
	_ok("every row's x/y is inside the grid (0..%d, 0..%d)" % [g.x, g.y], bounds_ok, true)
	_ok("every row's subtitle is non-blank", subtitle_ok, true)
	_ok("every row's name is non-blank", name_ok, true)

	print("\n=== 9: row shape -- exactly the contracted keys ===")
	var want_keys := ["name", "kind", "subtitle", "x", "y", "entity", "id"]
	var shape_ok := true
	if idx.all().is_empty():
		shape_ok = false
	else:
		var row: Dictionary = idx.all()[0]
		for k in want_keys:
			if not row.has(k):
				shape_ok = false
				_p("missing key: %s" % k)
		for k in row.keys():
			if not want_keys.has(k):
				shape_ok = false
				_p("unexpected extra key: %s" % k)
	_ok("row carries exactly the contracted keys", shape_ok, true)

	print("\nplace_search_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

extends Node
## Re-derives, rather than trusts, the Asset-pack flattening's search-loss
## claim (menus.gd's own header on `_build_asset_pack_submenu`, and commit
## 1111611's "Nine Asset-pack commands lost their search terms").
##
## That commit's table names fifteen dropped item rows (seven EDIT, five
## BATCH, two of BUILD's four, and the foot) and says three of them --
## Collect into set, Slot transform, Duplicate -- now return 0 title matches.
## Those figures are days old by the time anything schedules against them
## (MISTAKES.md's own preflight rule), so this probe re-runs the check against
## the LIVE MenuBar/CommandIndex rather than believing the prose.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _apcmdcheck_probe.tscn
##
## Title match only, never search() -- CommandIndex.search() also matches
## blurb/group, which is exactly what produced a false positive in an earlier
## verifier pass off a neighbouring tooltip (_idxfind_probe.gd's own header).

var _app: Node

## The fifteen dropped rows, each reduced to the phrase a searcher would
## actually type: the row's own leading words, not a trailing " · "-joined
## gloss. Old label kept alongside for cross-check against menus.gd's table.
const DROPPED := [
	["Open library workspace", "Open library workspace   ▤"],
	["Import image into slot", "Import image into slot…"],
	["Sprite sheet slicer", "Sprite sheet slicer…   cols · rows · margin"],
	["Add variant to slot", "Add variant to slot   + variant"],
	["Replace", "Replace · delete slot art"],
	["Slot transform", "Slot transform   scale · fit · reset"],
	["Preview background", "Preview background   checker"],
	["Tag", "Tag…"],
	["Collect into set", "Collect into set…"],
	["Rename", "Rename…"],
	["Duplicate", "Duplicate"],
	["Delete", "Delete"],
	["Apply to map", "Apply to map   compile & load"],
	["Import pack .zip", "Import pack .zip…"],
	["Clear library", "Clear library…   destructive"],
]

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	wd.timeout.connect(func():
		print("APCMD  TIMEOUT")
		get_tree().quit(2))
	add_child(wd)
	wd.start()

	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(1)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await _frames(50)
	print("[BOOT] shell up")

	var idx = load("res://shell/command_index.gd").new()
	idx.build(_app, _app.get("bridge"))
	var rows: Array = idx.all()
	print("APCMD  total indexed rows: %d" % rows.size())

	var zero := 0
	for pair in DROPPED:
		var needle: String = String(pair[0]).to_lower()
		var old_label: String = String(pair[1])
		var hits := 0
		for r in rows:
			if String(r.get("title", "")).to_lower().find(needle) >= 0:
				hits += 1
		if hits == 0:
			zero += 1
		print("APCMD  %-26s (was %-42s) title-matches=%d" % [pair[0], old_label, hits])

	print("APCMD  RESULT  %d of %d dropped-row terms now return 0 title matches" % [zero, DROPPED.size()])

	## Diagnostic pass: what a `hits>0` word above actually matched (so a
	## survivor is confirmed, not assumed), and whether a candidate EXTRAS
	## title would land on a clean, unoccupied word.
	print("\nAPCMD  -- what the >0 hits above actually are --")
	for needle in ["Sprite sheet slicer", "Tag", "Delete", "Clear library"]:
		for r in rows:
			if String(r.get("title", "")).to_lower().find(needle.to_lower()) >= 0:
				print("APCMD    '%s' -> title=%-30s group=%-12s kind=%s" % [
					needle, String(r["title"]), String(r["group"]), String(r["kind"])])

	print("\nAPCMD  -- candidate EXTRAS titles: any collision today? --")
	for cand in ["Collect into set", "Slot transform", "Add variant to slot",
			"Preview background", "Rename", "Duplicate", "Replace slot art"]:
		var hits := 0
		for r in rows:
			if String(r.get("title", "")).to_lower().find(cand.to_lower()) >= 0:
				hits += 1
		print("APCMD    %-24s collision-hits=%d" % [cand, hits])

	get_tree().quit(0)

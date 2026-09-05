extends SceneTree
## Confirms the two commands the 2026-09-05 moves dropped out of search are back.
## A verifier measured 0 title matches out of 361 rows for both, after the rows
## moved off the menu bar. Title match only -- CommandIndex.search() also matches
## blurbs, and that is exactly what produced a false positive for the verifier's
## own first pass (two atlas rows whose tooltips mention "Refine detail").
func _init() -> void:
	var idx = load("res://shell/command_index.gd").new()
	idx.build(null, null)
	var want := ["Journey planner", "Refine detail for the current view"]
	var rows: Array = idx._rows
	var fail := 0
	for w in want:
		var hits := 0
		for r in rows:
			if String(r.get("title", "")) == w:
				hits += 1
		print("IDXFIND  %-38s title-matches=%d" % [w, hits])
		if hits != 1:
			fail += 1
	print("IDXFIND  rows=%d  failures=%d" % [rows.size(), fail])
	quit(1 if fail else 0)

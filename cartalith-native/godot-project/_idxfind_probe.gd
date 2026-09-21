extends SceneTree
## Confirms the command the 2026-09-05 move dropped out of search is back.
## A verifier measured 0 title matches out of 361 rows for it, after the row
## moved off the menu bar. Title match only -- CommandIndex.search() also matches
## blurbs, and that is exactly what produced a false positive for the verifier's
## own first pass (two atlas rows whose tooltips mention "Refine detail").
##
## **`Refine detail for the current view` dropped out of `want` 2026-09-21,
## Ruling U.** It is not an `EXTRAS` row any more -- it went back to being a
## real `Preferences ▸ Tiles & LOD ▸ Atlas cache` `PopupMenu` row
## (`menus.gd::_build_atlas_cache_menu()`), which this probe's `idx.build(null,
## null)` cannot see: `_add_menu_commands()` returns immediately when `_app`
## is null, so this harness only ever exercised `EXTRAS`, never the live menu
## walk. `_vfy_batch0905_probe.gd` builds `CommandIndex` against a real `app`
## and still asserts `Refine detail` is title-findable there.
func _init() -> void:
	var idx = load("res://shell/command_index.gd").new()
	idx.build(null, null)
	var want := ["Journey planner"]
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

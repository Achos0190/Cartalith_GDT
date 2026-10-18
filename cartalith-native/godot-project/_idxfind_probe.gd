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
##
##
## **`Journey planner` left `EXTRAS` 2026-10-18, SM-2 (Ruling CK)**, for the same
## reason: it is a real `Story ▸ JOURNEYS` menu row again, which the live walk
## indexes. So this probe now asserts the OPPOSITE for it -- zero `EXTRAS`
## title matches -- because an `EXTRAS` copy beside the menu row would make the
## command appear twice in search (`command_index.gd`'s own rule: a title the
## walk owns must not be a second, driftable copy). `_vfy_batch0905_probe.gd`
## asserts the live, menu-walk side (`A: command index still finds ...`).
##
## Run:    Godot_v4.7.1-stable_win64_console.exe --headless --path . --script _idxfind_probe.gd
##         (a SceneTree script, no .tscn; it reads no pixels)
## Result: one `IDXFIND  <query>  title-matches=N` line per query, then
##         `IDXFIND  rows=N  failures=M`; exit 1 when M > 0, else 0 (each query must match
##         exactly its expected count of `EXTRAS` titles).
func _init() -> void:
	var idx = load("res://shell/command_index.gd").new()
	idx.build(null, null)
	## query -> how many `EXTRAS` titles must match it exactly.
	var want := {"Journey planner": 0, "Theme": 1}
	var rows: Array = idx._rows
	var fail := 0
	for w in want:
		var hits := 0
		for r in rows:
			if String(r.get("title", "")) == w:
				hits += 1
		print("IDXFIND  %-38s title-matches=%d (want %d)" % [w, hits, int(want[w])])
		if hits != int(want[w]):
			fail += 1
	print("IDXFIND  rows=%d  failures=%d" % [rows.size(), fail])
	quit(1 if fail else 0)

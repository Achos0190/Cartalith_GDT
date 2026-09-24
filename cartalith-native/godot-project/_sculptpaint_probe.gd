extends SceneTree

## Ruling AS (2026-09-24): a sculpt commit clears the painted override cells it
## covers. The Rust tests pin `SculptStamp::footprint` and
## `PaintEditor::clear_cells_under`; this is the WIRING, through the real
## `WorldGen::sculpt_commit` over the gdext boundary, which no unit test reaches.
##
## Paints one radius-1 biome dab (5 cells, `hypot <= 1`) under a sculpt tap and
## one far outside it, commits the paint, taps a Mountains stamp at the first
## and commits the sculpt. Asserts: the commit reports 5 biome cells cleared;
## the layer drops from 10 painted cells to 5; the ledger carries a recorded
## "Sculpt cleared paint" row; and Edit > Undo (`undo_last`) restores height
## but NOT the paint, with that row still listed -- the disclosed behaviour.
##
##   godot --headless --path godot-project -s _sculptpaint_probe.gd
## Run `cargo build -p cartalith-godot` first: this loads the DLL.

var fails := 0

func check(ok: bool, what: String) -> void:
	if not ok:
		fails += 1
	print("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _ledger_has(wg: WorldGen, label: String) -> Dictionary:
	for r in wg.undo_ledger():
		if String(r["label"]) == label:
			return r
	return {}

func _init() -> void:
	var wg: WorldGen = WorldGen.new()
	wg.generate_sized(12345, 800.0, 256, 192)
	var cx := 128.0
	var cy := 96.0

	wg.paint_set_layer("biome")
	## land_only false: the fixture must not depend on where the sea fell.
	wg.paint_set_brush(3, 1.0, 1.0, 0.0, false, false)
	wg.paint_stroke_at(cx, cy)
	wg.paint_stroke_at(6.0, 6.0)
	wg.paint_commit()
	var before := int(wg.paint_painted_counts()["total"])
	check(before == 10, "fixture: two 5-cell dabs painted (%d)" % before)

	wg.sculpt_set_feature("mountains")
	wg.sculpt_begin_stroke()
	wg.sculpt_add_point(cx, cy)
	wg.sculpt_end_stroke()
	var h_before: float = float(wg.sample_cell(int(cx), int(cy)).get("elevation", -1.0))
	var summary: Dictionary = wg.sculpt_commit("probe")
	check(int(summary.get("stamps_applied", 0)) == 1, "one stamp applied")
	var pc: Dictionary = summary.get("paint_cleared", {})
	check(int(pc.get("biome", -1)) == 5 and int(pc.get("terrain", -1)) == 0 and int(pc.get("splat", -1)) == 0,
		"sculpt_commit reports paint_cleared %s (expect biome 5, terrain 0, splat 0)" % str(pc))
	var after := int(wg.paint_painted_counts()["total"])
	check(after == 5, "the dab under the stamp is gone and the far dab kept (%d painted)" % after)
	var row := _ledger_has(wg, "Sculpt cleared paint")
	check(not row.is_empty() and String(row.get("kind", "")) == "recorded",
		"the ledger records the clear as a non-reversible row: %s" % str(row))
	if not row.is_empty():
		print("    reason: %s" % String(row["reason"]))

	var undone := String(wg.undo_last())
	print("  undo_last -> `%s`" % undone)
	check(undone != "", "undo_last reverted something")
	check(int(wg.paint_painted_counts()["total"]) == 5,
		"undo restores height only: paint stays cleared (%d painted)" % int(wg.paint_painted_counts()["total"]))
	check(not _ledger_has(wg, "Sculpt cleared paint").is_empty(),
		"after undo the 'Sculpt cleared paint' row is still listed, so the loss stays disclosed")
	check(_ledger_has(wg, "Sculpt commit").is_empty(), "after undo the sculpt's own height row is gone")
	if h_before >= 0.0:
		var h_now: float = float(wg.sample_cell(int(cx), int(cy)).get("elevation", -1.0))
		check(is_equal_approx(h_now, h_before), "undo restored the height under the tap (%.5f vs %.5f)" % [h_now, h_before])

	## A sculpt with nothing painted anywhere clears nothing and writes no row.
	var wg2: WorldGen = WorldGen.new()
	wg2.generate_sized(12345, 800.0, 256, 192)
	wg2.sculpt_set_feature("mountains")
	wg2.sculpt_begin_stroke()
	wg2.sculpt_add_point(cx, cy)
	wg2.sculpt_end_stroke()
	var s2: Dictionary = wg2.sculpt_commit("probe")
	var pc2: Dictionary = s2.get("paint_cleared", {})
	check(int(pc2.get("biome", -1)) == 0 and _ledger_has(wg2, "Sculpt cleared paint").is_empty(),
		"no paint: nothing cleared, no row (%s)" % str(pc2))

	print("RESULT fail=%d" % fails)
	quit(1 if fails else 0)

extends Node
## ET-D2/D3 follow-up (2026-11-23): an uncommitted paint stroke is taken back by the SHELL's Undo
## (Edit > Undo, Ctrl+Z and the toolbar squares all end in `DccApp.undo_last()`), one whole stroke per
## press, and only while the draft holds one.
##
##   godot --headless --path . res://_paintundo_probe.tscn
##
## Checks (each says what it protects):
##  1. Premise: a world generated, two strokes drafted (positive control: the draft is not empty).
##  2. `can_undo_any()` and the row's label say "Paint stroke" while a draft stroke exists.
##  3. One `undo_last()` removes exactly ONE stroke (the first stays), the second removes the other.
##  4. With the draft empty again, `can_undo_any()` no longer answers for paint (nothing to undo).
## Exit 0 held, 1 a check failed, 2 could not run. Mutant: delete the `bridge.paint_can_undo()` branch of
## `app.gd::_undo_last_impl` (check 3 goes red).

var _fail := 0

func _ok(c: bool, what: String) -> void:
	print(("  PASS  %s" if c else "  FAIL  %s") % what)
	if not c:
		_fail += 1

func _ready() -> void:
	get_tree().create_timer(300.0).timeout.connect(func() -> void: get_tree().quit(2))
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var br: Node = app.bridge
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	br.generate({"seed": 4242, "width_km": 800.0, "grid_w": 256, "grid_h": 164, "sea_level": 0.5, "villages": true})
	while br.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().process_frame
	br.paint_set_layer("biome")
	br.paint_set_brush(1, 6.0, 1.0, 0.0, false, false)
	for row in 2:
		_ok(br.paint_stroke_begin({}), "stroke %d opens" % row)
		for k in 8:
			br.paint_stroke_sample(60.0 + 6.0 * k, 50.0 + 20.0 * row, null, null, k * 8)
		br.paint_stroke_end()
	var drafted: int = br.paint_draft_count()
	_ok(drafted > 0, "premise: two strokes left %d dabs in the draft" % drafted)
	_ok(app.can_undo_any() and app.undo_target_label() == "Paint stroke", "Undo is offered, labelled Paint stroke (%s)" % app.undo_target_label())
	app.undo_last()
	var after1: int = br.paint_draft_count()
	_ok(after1 > 0 and after1 < drafted, "one Undo takes back ONE stroke, not both and not none (%d -> %d)" % [drafted, after1])
	app.undo_last()
	_ok(br.paint_draft_count() == 0, "the second Undo takes back the other (%d left)" % br.paint_draft_count())
	_ok(not br.paint_can_undo() and app.undo_target_label() != "Paint stroke", "with the draft empty the paint step is gone (label %s)" % app.undo_target_label())
	print("== %s: %d failure(s) ==" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(1 if _fail > 0 else 0)

extends Node

## **Does a control created AFTER boot, inside a dock, get floored?**
##
## `tablet_fit()` ran once, from the deferred pass in `register_workspace()`, so
## anything a rebuild created afterwards was never reached. Three consecutive
## batches found that shape and fixed the instances it produced — an `edit`
## button at 44×29, its `×` at 27×29, a colour well at 60×24, all against the
## 44 dp floor. **Fixing instances does not stop a boot-only fitter from missing
## the next rebuild**, so `_run_tablet_dock_arbitrate()` now calls
## `tablet_fit(dock)` beside its arbitration.
##
## That change is the kind that is easy to prove SAFE and easy to leave INERT:
## the three known defects were already fixed per-site, so no existing probe
## moves whether the hook fires or not. This one creates the situation instead
## of waiting for it — a deliberately undersized `Button` parented into the
## right dock after the shell is up, which is exactly what a rebuild does.
##
## Run WINDOWED at tablet: `--force-touch --vp 1600x1000`. It refuses anywhere
## else rather than passing vacuously, because `tablet_fit()` returns
## immediately unless `DccTheme.is_tablet()` and a green run off-tablet would
## assert nothing at all.
##
##   godot --headless _tabrefit_probe.tscn -- --force-touch --vp 1600x1000
##
## Exit 0 all held · 1 an assertion failed · 2 could not run here.

const UNDERSIZED_H := 12.0   ## Well under any floor, so the assertion is unambiguous.

var _fail := 0

func _log(s: String) -> void:
	print("[tabrefit] ", s)

func _ok(what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fail += 1
	_log(("  ok   " if cond else "  FAIL ") + what + ("  " + detail if detail != "" else ""))

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _ready() -> void:
	## Boot the shell the way every sibling probe does: a `SubViewport` sized to
	## the composition under test, with `app.tscn` inside it.
	var vp := SubViewport.new()
	vp.size = Vector2i(1600, 1000)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(30)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(30)

	if not DccTheme.is_tablet():
		_log("ABORT this run is not the tablet composition — tablet_fit() returns")
		_log("  immediately off-tablet, so every assertion below would hold for")
		_log("  the wrong reason. Re-run with `-- --force-touch --vp 1600x1000`.")
		get_tree().quit(2)
		return

	var dock: Control = app.right_dock
	if dock == null:
		_log("ABORT right_dock is null")
		get_tree().quit(2)
		return

	## The floor the fitter applies, read from the same role the fitter reads —
	## pinned as a relationship rather than as a literal, because a role change
	## should move both sides together and a wrong role should fail here.
	var floor_h := float(DccTheme.role_px("btn_min_h"))
	_ok("the tablet floor is a real height", floor_h >= 40.0,
		"btn_min_h=%.1f" % floor_h)

	## Now do what a rebuild does: parent a new, undersized control into the
	## dock once the shell is already up.
	var late := Button.new()
	late.text = "late"
	late.custom_minimum_size = Vector2(60, UNDERSIZED_H)
	dock.add_child(late)
	_ok("the control really was undersized when added",
		is_equal_approx(late.custom_minimum_size.y, UNDERSIZED_H),
		"h=%.1f" % late.custom_minimum_size.y)

	## `_on_tablet_node_added` defers through `_tablet_arb_pending`, so the fit
	## lands on a later frame — waiting on frames rather than asserting at once
	## is the difference between testing the hook and testing the timing.
	await _frames(8)

	_ok("a control added AFTER boot is floored by the rebuild hook",
		late.custom_minimum_size.y >= floor_h,
		"got %.1f want >= %.1f" % [late.custom_minimum_size.y, floor_h])
	_ok("the fitter marked it, so a re-walk will skip it",
		late.has_meta("_tablet_fitted"))

	late.queue_free()
	_log("=== end tabrefit fails=%d ===" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

extends Node
## Lane labelcue -- OUTSTANDING_WORK.md "Make the need to run CARTO ▸ Generate
## labels visible on screen" (Ruling AZ).
##
## Run WINDOWED, not `--headless` -- the dock has to lay out for
## `_open_labels_category()` to reach anything, the same reason
## `_labelroles_probe.gd` (this file's own template) runs windowed.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x900 _labelgencue_probe.tscn
##
## What this checks, in order:
##
##  1. No world yet -- the cue stays hidden. `_label_class_summary` already
##     carries the engine's own refusal sentence
##     (`label_bridge/generate.rs::labels_generate`'s `ok: false` reason), and a
##     Generate button would have nothing to run against.
##  2. A fresh generate -- POSITIVE CONTROL. `_regenerate_labels()` already runs
##     automatically (`call_deferred` at dock build, `generation_finished`/
##     `world_loaded`), so a world with real settlements/continents to name
##     produces `_label_gen_ran == true` and the cue is hidden without anything
##     pressed.
##  3. The "never generated" branch, exercised at the symbol -- `_label_gen_ran`
##     and `_label_terrain_stale` are the two flags `_refresh_label_gen_cue()`
##     reads, and setting the first false and calling the refresh directly is
##     the honest way to reach a state that (per `engine_bridge.gd`'s own
##     forwarder audit on `labels_clear_generated`) nothing in the shipped UI
##     can otherwise produce for a world that already exists -- it is a startup
##     race, not a reachable click path.
##  4. The real, reachable trigger -- `sculpt_draft_changed`, which
##     `engine_bridge.gd`'s audit names as the one path that leaves a stale run:
##     emitting it (the same signal a real sculpt commit emits) must set
##     `_label_terrain_stale` and show the cue with its own, different sentence.
##  5. Pressing the cue's own "Generate labels" button must clear both flags,
##     hide the cue, and leave `_label_gen_ran` true -- proving the button is
##     wired to `_regenerate_labels()` and not decorative.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row.

const SEED := 483920

var app: Node
var ws: Node
var bridge
var fails := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

## `_labelroles_probe.gd`'s own helper, copied rather than shared across two
## probe files with no common base -- it walks the dock for the accordion
## header ending in "Labels" and presses it if its body is still hidden.
func _open_labels_category() -> bool:
	var found: Button = null
	var stack: Array = [ws]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children():
			stack.append(c)
			if c is Button and String((c as Button).text).ends_with("Labels"):
				found = c
	if found == null:
		return false
	var pad := found.get_parent().get_child(found.get_index() + 1) if found.get_parent() != null else null
	if pad is Control and (pad as Control).visible:
		return true
	found.emit_signal("pressed")
	return true

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	print("LGC shell build %s" % DccShell.build_id())

	ws = app.left_dock_body.get_node_or_null("CartographyWorkspace")
	if ws == null:
		print("LGC FATAL: no CartographyWorkspace under left_dock_body")
		get_tree().quit(1)
		return
	bridge = app.bridge

	app.select_domain_mode("cartography", "labels")
	await _frames(10)
	_ok(_open_labels_category(), "the Labels category opens")
	await _frames(5)

	# -- 1. no world yet: the cue is gated off -----------------------------------
	_ok(not bridge.has_world, "no world exists yet (precondition)")
	_ok(ws._label_gen_cue != null and is_instance_valid(ws._label_gen_cue),
		"the cue row was built")
	_ok(not ws._label_gen_cue.visible, "cue hidden before any world exists")

	# -- 2. fresh generate: POSITIVE CONTROL -------------------------------------
	bridge.generate({"seed": SEED, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(15)
	if not bridge.has_world:
		print("LGC FATAL: generate failed")
		get_tree().quit(1)
		return
	_ok(bool(ws._label_gen_ran), "labels_generate ran automatically over the fresh world")
	_ok(not bool(ws._label_terrain_stale), "a fresh world is not flagged terrain-stale")
	_ok(not ws._label_gen_cue.visible,
		"POSITIVE CONTROL: cue hidden once labels exist for this world")

	# -- 3. "never generated", exercised at the symbol ---------------------------
	# Not reachable through the shipped UI for a world that already exists (see
	# this file's header) -- set the flag `_refresh_label_gen_cue()` reads and
	# call it directly, the same honest substitute
	# `engine_bridge.gd`'s own audit prescribes for an unwired path.
	ws._label_gen_ran = false
	ws._refresh_label_gen_cue()
	await _frames(2)
	_ok(ws._label_gen_cue.visible, "cue VISIBLE: no generated labels for this world")
	_ok(String(ws._label_gen_cue_note.text).findn("generated") >= 0,
		"cue names the missing state in words (\"%s\")" % ws._label_gen_cue_note.text)
	_ok(ws._label_gen_cue_btn != null and ws._label_gen_cue_btn.is_visible_in_tree(),
		"the Generate action sits beside the message")

	# Press the cue's own action -- proves it is wired to _regenerate_labels(),
	# not decorative.
	ws._label_gen_cue_btn.emit_signal("pressed")
	await _frames(6)
	_ok(bool(ws._label_gen_ran), "pressing Generate labels re-runs the pass")
	_ok(not ws._label_gen_cue.visible, "cue disappears once labels exist again")

	# -- 4. the real, reachable trigger: a sculpt commit -------------------------
	bridge.sculpt_draft_changed.emit()
	await _frames(2)
	_ok(bool(ws._label_terrain_stale), "sculpt_draft_changed marks the run stale")
	_ok(ws._label_gen_cue.visible, "cue VISIBLE: terrain changed since the last run")
	_ok(String(ws._label_gen_cue_note.text).findn("terrain") >= 0,
		"cue's stale sentence names what changed (\"%s\")" % ws._label_gen_cue_note.text)

	# -- 5. the cue's own action clears staleness too ----------------------------
	ws._label_gen_cue_btn.emit_signal("pressed")
	await _frames(6)
	_ok(not bool(ws._label_terrain_stale), "pressing Generate labels clears the stale flag")
	_ok(bool(ws._label_gen_ran), "labels_generate ran again")
	_ok(not ws._label_gen_cue.visible, "cue disappears once the run is current again")

	print("LGC DONE fails=%d" % fails)
	get_tree().quit(1 if fails > 0 else 0)

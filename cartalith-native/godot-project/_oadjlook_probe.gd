extends Node
## Two backlog rows, one probe (OUTSTANDING_WORK.md, 2026-10-12):
##
## 1. "The adjustment panel does not say when a contrast guard held an adjustment back".
##    Protects `AdjustPanel.guard_text` (what the "AdjGuard" line shows): it must name, per
##    layer, what the guards did, from the guards' OWN reports -- the faction guard's
##    `adjusted_factions_report()` and the label/halo guard's `label_guard_summary()`
##    (`guard_ink_pair_report`). The label counts are checked against an INDEPENDENT oracle
##    built from what is drawn (`_stacked_pair` against the raw pair and the stacked ink), not
##    against the summary's own loop. Positive controls: the label guard both moves a fill and
##    reverts a pair under at least one (preset x stack); a mild stack says nothing.
## 2. "Overlay stack in saved looks and generate inconsistency". The rule decided: a look
##    carries ALL THREE stacks; a generate keeps ALL THREE (the engine's two already did);
##    a close or open resets all three. Protects: a saved look round-trips the overlay stack
##    through its sidecar; a look without one clears the live stack; re-saving over a look
##    removes its stale sidecar; an invalid sidecar refuses the whole look (engine stack
##    unchanged too); the sidecar is never listed as a look; a second generate leaves the
##    overlay stack in place beside the terrain stack; a close empties the overlay stack.
##
## MUST run WINDOWED from a scratch copy with its own config/name (it writes looks):
##   Godot_v4.7.x_win64_console.exe --path <copy> res://_oadjlook_probe.tscn -- --out DIR
## Exit 0 = every check held; 1 = a check failed; 2 = could not run.
## Result: an assertion, windowed only. Command per the header: `res://_oadjlook_probe.tscn -- --out DIR` (the code reads
## `--out` only as the first two user arguments). Prints `OADJ  ok|FAIL <what>`, `OADJ  MISMATCH ...` / `OADJ  SAID-WRONG
## ...` detail lines, `OADJ  faction modes seen: ...` and `OADJ  RESULT checks=<n> fail=<n>`; stderr carries
## `PROBE-CANNOT-RUN: ...` (headless, 1500 s watchdog, no world, no render workspace or adjust/preset API). It writes a
## sidecar file under `--out`. Exit 0 = every check held, 1 = a check failed, 2 = could not run.

const AP := preload("res://shell/adjust_panel.gd")
const OA := preload("res://shell/overlay_adjust.gd")
const PRESETS := ["Default", "Antique", "Ink", "Watercolor", "Print", "Atlas", "Blueprint", "Woodcut", "Night", "Nautical"]
const SWATCHES := [Color8(230, 159, 0), Color8(86, 180, 233), Color8(0, 158, 115),
	Color8(240, 228, 66), Color8(0, 114, 178), Color8(213, 94, 0)]
const CUSTOM := [false, false, true, false, false, false]

var _out := ""
var _fail := 0
var _checks := 0
var _app: Node
var _vh: Control
var _br: Node
var _ov: Control
var _rw: Node


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fail += 1
	print("OADJ  %-4s %s" % ["ok" if cond else "FAIL", what])


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null


func _entry(kind: Dictionary) -> Dictionary:
	return {"kind": kind, "enabled": true, "opacity": 1.0}


func _json(entries: Array) -> String:
	return JSON.stringify({"entries": entries})


func _hsl(h: float, s: float, l: float) -> Dictionary:
	return {"hsl": {"hue_deg": h, "saturation": s, "lightness": l}}


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- run windowed.")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	if args.size() >= 2 and args[0] == "--out":
		_out = args[1]
	get_tree().create_timer(1500.0).timeout.connect(func() -> void:
		printerr("PROBE-CANNOT-RUN: watchdog"); get_tree().quit(2))
	DisplayServer.window_set_size(Vector2i(1600, 1000))
	await _frames(2)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	_ov = _vh.overlay
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await _generate(483920)
	if _br.generating or _br.settlements().is_empty():
		printerr("PROBE-CANNOT-RUN: no generated world")
		get_tree().quit(2)
		return
	_br.labels_generate({"cull": {"on": false}})
	_vh.refresh_annotations()
	_app.select_domain_category("cartography", "Style")
	await _frames(8)
	_rw = _find(_app, "render_workspace.gd")
	if _rw == null or not _br.adjust_api or not _br.preset_api:
		printerr("PROBE-CANNOT-RUN: no render workspace or no adjust/preset API")
		get_tree().quit(2)
		return
	_ov.set_faction_custom(CUSTOM)
	_ov.set_faction_colors(SWATCHES)
	_leg_guard_text()
	await _leg_saved_look()
	await _leg_generate_and_close()
	print("OADJ  RESULT checks=%d fail=%d" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)


func _generate(seed_: int) -> void:
	_br.generate({"seed": seed_, "width_km": 1200.0, "grid_w": 512, "grid_h": 328,
		"archetype": "", "villages": true, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout


## The label guard's outcome per distinct raw pair, from what is DRAWN: `stack` when the drawn
## fill is the stacked fill, `original` when the drawn pair is the raw one, `guarded` between.
func _oracle() -> Dictionary:
	var seen := {}
	var out := {"pairs": 0, "guarded": 0, "original": 0}
	var raws: Array = [_ov._settlement_name_inks_preset()]
	for lb in _ov.get("_labels"):
		raws.append(_ov._label_inks_preset(lb))
	for raw: Array in raws:
		var key := [raw[0], raw[1]]
		if seen.has(key):
			continue
		seen[key] = true
		var stacked: Color = OA.color(raw[0])
		var drawn: Array = _ov._stacked_pair(raw)
		out["pairs"] += 1
		if drawn[0] == stacked and drawn[1] == OA.color(raw[1]):
			continue
		if drawn[0] == raw[0] and drawn[1] == raw[1]:
			out["original"] += 1
		else:
			out["guarded"] += 1
	return out


func _leg_guard_text() -> void:
	# Engine layers: no guard; plain statement only while the stack has an entry.
	_ok(AP.guard_text("terrain", null, 0, false) == "", "terrain, empty stack: says nothing")
	_ok(AP.guard_text("terrain", null, 2, false).contains("No guard holds this layer back"), "terrain with entries: states that no guard applies")
	_ok(AP.guard_text("water", null, 1, true) == "", "an unknown (unreadable) layer says nothing")
	_vh.set_overlay_adjust('{"entries":[]}')
	_ok(AP.guard_text("overlay", _ov, 0, false) == "", "overlay, no live stack: says nothing")
	# A mild stack: no label pair held back, no faction mode other than stack.
	_vh.set_overlay_adjust(_json([_entry(_hsl(10.0, 0.05, 0.0))]))
	_ok(AP.guard_text("overlay", _ov, 1, false) == "", "mild overlay stack: nothing held back, nothing said")
	var stacks := {
		"all white": [_entry(_hsl(0.0, 0.0, 1.0))],
		"all black": [_entry(_hsl(0.0, 0.0, -1.0))],
		"invert": [_entry({"levels": {"in_black": 0.0, "in_white": 255.0, "gamma": 1.0, "out_black": 255.0, "out_white": 0.0}})],
		"desaturate+hue": [_entry(_hsl(180.0, -1.0, 0.0))],
		"flat grey": [_entry({"gradient_map": {"stops": [{"at": 0.0, "col": [119.0, 119.0, 119.0], "a": 1.0}], "mode": "Linear"}})],
	}
	var seen_modes := {}
	var label_guarded := 0
	var label_original := 0
	var mismatch := 0
	var said_wrong := 0
	for pname in PRESETS:
		var t := RenderWorkspace.overlay_treatment_for(pname) if pname != "Default" else {}
		_vh.set_overlay_treatment(t)
		_ov.set_faction_custom(CUSTOM)
		_ov.set_faction_colors(SWATCHES)
		for sname in stacks:
			_vh.set_overlay_adjust(_json(stacks[sname]))
			var o := _oracle()
			var s: Dictionary = _ov.label_guard_summary()
			if int(s["pairs"]) != int(o["pairs"]) or int(s["guarded"]) != int(o["guarded"]) or int(s["original"]) != int(o["original"]):
				mismatch += 1
				print("OADJ  MISMATCH %s/%s summary=%s oracle=%s" % [pname, sname, str(s), str(o)])
			label_guarded += int(o["guarded"])
			label_original += int(o["original"])
			var txt := AP.guard_text("overlay", _ov, 1, false)
			var mode: String = _ov.adjusted_factions_mode()
			seen_modes[mode] = true
			var want_f: bool = mode in ["guarded", "clamped", "original"]
			var want_l: bool = int(o["guarded"]) + int(o["original"]) > 0
			if txt.contains("Faction colours:") != want_f or txt.contains("Labels and halos:") != want_l:
				said_wrong += 1
				print("OADJ  SAID-WRONG %s/%s mode=%s oracle=%s text=%s" % [pname, sname, mode, str(o), txt])
			if mode == "original" and not txt.contains("original colours"):
				said_wrong += 1
			if int(o["original"]) > 0 and not txt.contains("reverted"):
				said_wrong += 1
			if int(o["guarded"]) > 0 and not txt.contains("moved the text colour"):
				said_wrong += 1
	_ok(mismatch == 0, "label_guard_summary equals the independent oracle on every (preset x stack): %d mismatches" % mismatch)
	_ok(said_wrong == 0, "the guard text names exactly the guards that held something back: %d wrong" % said_wrong)
	_ok(label_guarded > 0, "positive control: some label pair has its fill moved (%d)" % label_guarded)
	## `original` is the guard's defensive last branch: `_pin_contrast_fix` can always reach 4.5
	## against any ground (black or white contrasts >= 4.58 with every ground), so no real
	## (preset x stack) reaches it -- measured 0 here. Its wording is therefore driven through a
	## stub carrying the guards' documented report shapes, so the sentence is not unseen code.
	print("OADJ  label pairs reverted to the preset's across the sweep: %d (defensive branch)" % label_original)
	var stub := _StubOv.new()
	var st := AP.guard_text("overlay", stub, 1, false)
	_ok(st.contains("held this stack back entirely") and st.contains("original colours")
		and st.contains("reverted 2 of 5 ink pairs") and st.contains("contrast 3.2 against a required 4.5")
		and st.contains("at least 20.0 apart") and st.contains("sits at 12.5"),
		"stubbed original outcomes: both sentences quote the guards' own numbers")
	print("OADJ  faction modes seen: %s" % str(seen_modes.keys()))
	_ok(seen_modes.size() >= 2, "positive control: the faction guard reports more than one outcome across the sweep")
	# The numbers quoted are the guard's own.
	for sname in stacks:
		_vh.set_overlay_treatment({})
		_vh.set_overlay_adjust(_json(stacks[sname]))
		var rep: Dictionary = _ov.adjusted_factions_report()
		if String(rep.get("mode", "")) in ["guarded", "clamped", "original"]:
			var txt2 := AP.guard_text("overlay", _ov, 1, false)
			_ok(txt2.contains("%.1f" % float(rep["required_de"])) and txt2.contains("%.1f" % float(rep["min_de"])),
				"faction text quotes the guard's own required and measured delta-E (%s, %s)" % [sname, rep["mode"]])
			break
	_vh.set_overlay_adjust('{"entries":[]}')
	_vh.set_overlay_treatment({})


func _look_names() -> Array:
	var out: Array = []
	for r in _br.appearance_presets():
		out.append(String(r[1]))
	return out


func _select_look(slug: String) -> void:
	var pick: OptionButton = _rw.get("_preset_pick")
	_rw._refresh_preset_list()
	var slugs: Array = _rw.get("_preset_slugs")
	pick.selected = slugs.find(slug)


func _leg_saved_look() -> void:
	var s_over := _json([_entry(_hsl(30.0, 0.3, 0.1))])
	var s_terr := _json([_entry(_hsl(-20.0, 0.2, 0.0))])
	_vh.set_overlay_adjust(s_over)
	_ok(_br.adjust_set("terrain", s_terr).get("ok", false), "terrain stack set")
	var nm: LineEdit = _rw.get("_preset_name")
	nm.text = "oadj one"
	_rw._on_save_look()
	var sidecar: String = _br.preset_overlay_path("oadj one")
	_ok(FileAccess.file_exists(sidecar), "saving a look with a live overlay stack writes its sidecar")
	_ok(not _look_names().has("oadj_one.overlay_adjust") and _look_names().has("oadj_one"), "the look is listed once; the sidecar is never offered as a look")
	# Clear both, load: both return.
	_vh.set_overlay_adjust('{"entries":[]}')
	_br.adjust_set("terrain", '{"entries":[]}')
	_select_look("oadj_one")
	_rw._on_load_look()
	_ok(OA.get_stack_json() == JSON.stringify(JSON.parse_string(s_over)), "loading the look restores the overlay stack (the engine's Terrain stack rode in its own file)")
	_ok(int(_br.adjust_get("terrain").get("entries", 0)) == 1, "...and the terrain stack returns with it")
	# A look saved with an EMPTY overlay stack clears the live one on load.
	_vh.set_overlay_adjust('{"entries":[]}')
	nm.text = "oadj two"
	_rw._on_save_look()
	_ok(not FileAccess.file_exists(_br.preset_overlay_path("oadj two")), "an empty overlay stack writes no sidecar")
	_vh.set_overlay_adjust(s_over)
	_select_look("oadj_two")
	_rw._on_load_look()
	_ok(OA.entry_count() == 0, "loading a look that carries no overlay stack clears the live one (as it resets the engine stacks)")
	# Re-saving over a look removes its stale sidecar.
	_vh.set_overlay_adjust(s_over)
	nm.text = "oadj two"
	_rw._on_save_look()
	_ok(FileAccess.file_exists(_br.preset_overlay_path("oadj two")), "re-saving with a live stack writes the sidecar")
	_vh.set_overlay_adjust('{"entries":[]}')
	_rw._on_save_look()
	_ok(not FileAccess.file_exists(_br.preset_overlay_path("oadj two")), "re-saving over it with an empty stack removes the stale sidecar")
	# An invalid sidecar refuses the whole look: nothing changes, engine stack included.
	var f := FileAccess.open(sidecar, FileAccess.WRITE)
	f.store_string('{"entries":[{"kind":{"hsl":{"hue_deg":9999.0,"saturation":0.0,"lightness":0.0}},"enabled":true,"opacity":1.0}]}')
	f.close()
	_br.adjust_set("terrain", '{"entries":[]}')
	_vh.set_overlay_adjust(_json([_entry(_hsl(5.0, 0.0, 0.0))]))
	var before := OA.get_stack_json()
	_select_look("oadj_one")
	_rw._on_load_look()
	_ok(OA.get_stack_json() == before and int(_br.adjust_get("terrain").get("entries", 0)) == 0,
		"an invalid sidecar refuses the whole look: overlay and terrain stacks unchanged")
	# A pre-sidecar look (no file at all) is the same as "no overlay stack".
	DirAccess.remove_absolute(ProjectSettings.globalize_path(sidecar))
	_vh.set_overlay_adjust(s_over)
	_select_look("oadj_one")
	_rw._on_load_look()
	_ok(OA.entry_count() == 0 and int(_br.adjust_get("terrain").get("entries", 0)) == 1, "a look saved before sidecars loads: overlay cleared, terrain restored")
	for n in ["oadj_one", "oadj_two"]:
		DirAccess.remove_absolute(ProjectSettings.globalize_path("%s/%s.json" % [_br.PRESET_DIR, n]))
		DirAccess.remove_absolute(ProjectSettings.globalize_path(_br.preset_overlay_path(n)))


func _leg_generate_and_close() -> void:
	var s_over := _json([_entry(_hsl(30.0, 0.3, 0.1))])
	_vh.set_overlay_adjust(s_over)
	_br.adjust_set("terrain", _json([_entry(_hsl(-20.0, 0.2, 0.0))]))
	await _generate(777001)
	_ok(not _br.generating and not _br.settlements().is_empty(), "second generate finished")
	_ok(OA.entry_count() == 1, "a generate keeps the overlay stack")
	_ok(int(_br.adjust_get("terrain").get("entries", 0)) == 1, "...as it keeps the engine's terrain stack (the rule: all three agree)")
	_app._close_world()
	await _frames(4)
	_ok(OA.entry_count() == 0, "closing the world empties the overlay stack")


## Stands in for the live `MapOverlay` with the two guards' documented report shapes
## (`guard_adjusted_factions`' result and `label_guard_summary`'s), for the outcome the real
## overlay cannot reach.
class _StubOv:
	extends RefCounted

	func adjusted_factions_report() -> Dictionary:
		return {"mode": "original", "min_de": 12.5, "min_contrast": 2.0, "required_de": 20.0, "required_contrast": 0.0}

	func label_guard_summary() -> Dictionary:
		return {"pairs": 5, "bar": 4.5, "guarded": 0, "original": 2, "worst_required": 4.5, "worst_before": 3.2}

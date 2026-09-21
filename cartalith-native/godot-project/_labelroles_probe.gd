extends Node
## Lane LabelRoles -- CARTO > Labels' per-role editor, its role defaults, the
## per-role count of hand-placed labels, and the rule that a role default never
## overwrites a per-label value on its own.
##
## Run WINDOWED, not `--headless`. Nothing here reads a pixel, but the shell has
## to be composited for the dock, the sheet and the label tool to lay out, and
## `MISTAKES.md`'s own rule is to run a probe in the mode its header prescribes
## rather than to mandate one -- so this header prescribes windowed and every
## number below was taken that way.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1080x2400 _labelroles_probe.tscn -- --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _labelroles_probe.tscn -- --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x900  _labelroles_probe.tscn
##
## **The density is read, never assumed from the flag.** `--resolution` is
## clamped to the monitor work area (`_phonesweep_probe.gd` measured a
## 1440x3168 request boot a 1440x1031 window), and `_compute_layout_mode()`
## decides phone-vs-tablet off the size it actually got. So the probe prints the
## viewport it has and names the leg PHONE / TABLET / POINTER from
## `DccTheme.is_phone()` / `is_tablet()`.
##
## **The touch floor is a floor on the TARGET, both axes.** `phone_fit()` floors
## width and height for every `BaseButton` under a dock; `tablet_fit()` floors
## height only, so a width defect at tablet density is invisible to the fitter
## and has to be measured here. At pointer density there is no touch floor at
## all and the sizes are reported without a verdict.
##
## Three seeds, because a dock's laid widths are content-dependent and one world
## is one sample (`MISTAKES.md`, "Report a layout measurement").
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them.

const SEEDS := [483920, 77021, 4242]

var app: Node
var ws: Node
var fails := 0
var density := "?"
var floor_px := 0.0
var _spans: Dictionary = {}   ## name -> [min_w, max_w, min_h, max_h]

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

## One tappable control against the floor, on BOTH axes.
func _target(tag: String, c: Control) -> void:
	if c == null or not is_instance_valid(c):
		fails += 1
		print("  FAIL  %s: control missing" % tag)
		return
	var s := c.size
	_span(tag, s)
	if floor_px <= 0.0:
		print("  ----  %s laid %.0f x %.0f (no touch floor at POINTER density)" % [tag, s.x, s.y])
		return
	_ok(s.x >= floor_px and s.y >= floor_px,
		"%s laid %.0f x %.0f >= %.0f x %.0f floor (%s)" % [tag, s.x, s.y, floor_px, floor_px, density])

func _span(tag: String, s: Vector2) -> void:
	if _spans.has(tag):
		var e: Array = _spans[tag]
		_spans[tag] = [minf(e[0], s.x), maxf(e[1], s.x), minf(e[2], s.y), maxf(e[3], s.y)]
	else:
		_spans[tag] = [s.x, s.x, s.y, s.y]

## The CARTO panel's own "Labels" L2 category header -- the accordion row that
## has to be pressed before anything below it is laid out at all.
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
	## `category()` keeps at most one open, so pressing an already-open one
	## would close it. Press only when its body is hidden.
	var pad := found.get_parent().get_child(found.get_index() + 1) if found.get_parent() != null else null
	if pad is Control and (pad as Control).visible:
		return true
	found.emit_signal("pressed")
	return true

func _dead_rows() -> Dictionary:
	var out: Dictionary = {}
	var stack: Array = [ws]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children():
			stack.append(c)
			if c is HBoxContainer and (c as Control).get_child_count() > 0:
				var first = (c as Control).get_child(0)
				if first is Label and ["Font family", "Weight", "Case"].has(String((first as Label).text)):
					out[String((first as Label).text)] = c
	return out

## The workspace's own `_prompt_label_name()` dialog, driven the way a user
## drives it -- typed name, Create pressed -- so the role and the two role
## defaults it applies at creation are exercised, not simulated.
func _create_label(nm: String, gx: float, gy: float) -> void:
	ws._prompt_label_name(gx, gy)
	await _frames(3)
	var dlg: AcceptDialog = null
	for c in app.get_children():
		if c is AcceptDialog and String((c as AcceptDialog).title).begins_with("New "):
			dlg = c
	if dlg == null:
		fails += 1
		print("  FAIL  create %s: no New-label dialog appeared" % nm)
		return
	var edit: LineEdit = null
	var stack: Array = [dlg]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children():
			stack.append(c)
			if c is LineEdit:
				edit = c
	if edit == null:
		fails += 1
		print("  FAIL  create %s: dialog has no LineEdit" % nm)
		return
	edit.text = nm
	dlg.confirmed.emit()
	await _frames(4)

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	var vp: Vector2 = get_viewport().get_visible_rect().size
	density = "PHONE" if DccTheme.is_phone() else ("TABLET" if DccTheme.is_tablet() else "POINTER")
	## PHONE: `phone_fit(dock, _phone_scale)` floors at `round(44 * unit)` in
	## laid pixels. TABLET: `role_px("btn_min_h")`. POINTER: none.
	if DccTheme.is_phone():
		floor_px = round(DccTheme.PHONE_TAP_MIN * app.phone_scale())
	elif DccTheme.is_tablet():
		floor_px = float(DccTheme.role_px("btn_min_h"))
	print("LR shell build %s" % DccShell.build_id())
	print("LR density=%s viewport=%.0fx%.0f is_touch=%s is_phone=%s is_tablet=%s phone_scale=%.3f floor=%.0f"
		% [density, vp.x, vp.y, DccTheme.is_touch(), DccTheme.is_phone(),
			DccTheme.is_tablet(), app.phone_scale(), floor_px])

	if DccTheme.is_phone():
		app._set_sheet_open("left", true)
		await _frames(6)

	ws = app.left_dock_body.get_node_or_null("CartographyWorkspace")
	if ws == null:
		print("LR FATAL: no CartographyWorkspace under left_dock_body")
		get_tree().quit(1)
		return

	var bridge = app.bridge
	for seed_v in SEEDS:
		print("LR ================ seed %d (%s) ================" % [seed_v, density])
		bridge.generate({"seed": seed_v, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
			"archetype": "", "villages": true, "sea_level": 0.45})
		var waited := 0
		while bridge.generating and waited < 3000:
			await get_tree().process_frame
			waited += 1
		await _frames(10)
		if not bridge.has_world:
			print("LR seed %d: generate FAILED" % seed_v)
			continue
		app.select_domain_mode("cartography", "labels")
		await _frames(10)
		_ok(_open_labels_category(), "the Labels category opens")
		await _frames(10)
		await _measure(seed_v == SEEDS[0], bridge)

	print("LR ---- laid-size spans across %d seeds (%s) ----" % [SEEDS.size(), density])
	for k in _spans:
		var e: Array = _spans[k]
		print("LR   %-28s w %.0f..%.0f  h %.0f..%.0f" % [k, e[0], e[1], e[2], e[3]])
	print("LR DONE %s fails=%d" % [density, fails])
	get_tree().quit(1 if fails > 0 else 0)


func _measure(behaviour: bool, bridge) -> void:
	# -- 1. the five role rows, and the route in ------------------------------
	var rows: Dictionary = ws._label_role_rows
	_ok(rows.size() == 5, "five role rows built (%d)" % rows.size())
	for key in ["continental", "region", "settlement", "water", "landmark"]:
		var parts: Dictionary = rows.get(key, {})
		var btn: Button = parts.get("btn")
		_ok(btn != null and is_instance_valid(btn) and btn.is_visible_in_tree(),
			"role row %s is in the tree and visible" % key)
		if btn == null:
			continue
		_target("role row %s" % key, btn)
		## Every cell inside the row must ignore the mouse, or it eats the press
		## over its own width and leaves a dead strip in the middle of the
		## control that selects the role.
		var bad := 0
		var stack: Array = [btn]
		while not stack.is_empty():
			var n: Node = stack.pop_back()
			for c in n.get_children():
				stack.append(c)
				if c is Control and (c as Control).mouse_filter != Control.MOUSE_FILTER_IGNORE:
					bad += 1
		_ok(bad == 0, "role row %s: every cell inside ignores the mouse (%d do not)" % [key, bad])

	# -- 2. pressing a row selects that role ----------------------------------
	for key in ["water", "landmark", "continental", "settlement"]:
		var btn: Button = (rows[key] as Dictionary)["btn"]
		btn.emit_signal("pressed")
		await _frames(3)
		_ok(String(ws._label_class) == key, "pressing the %s row selects it" % key)
		var mark: ColorRect = (rows[key] as Dictionary)["mark"]
		_ok(mark.color.a > 0.0, "the %s row's selection bar is drawn" % key)
		var others := 0
		for k2 in rows:
			if k2 != key and ((rows[k2] as Dictionary)["mark"] as ColorRect).color.a > 0.0:
				others += 1
		_ok(others == 0, "no other row's bar is drawn while %s is selected" % key)
		_ok(String(ws._label_class_title.text).begins_with(key.to_upper()),
			"the Type title follows the row press (%s)" % ws._label_class_title.text)


		## **The dropdown is a SECOND VIEW of the same state and must follow the
		## row press.** Until 2026-09-06 `DccWidgets.choice()`'s return was
		## discarded, which was harmless while the dropdown was the only route in;
		## adding the row press gave the panel two routes that disagreed on screen
		## -- pressing Water moved the marks, the title, the three dials and Apply
		## while this still read Settlement. Found by a verifier, not by this probe,
		## which asserted every other consumer and not this one.
		var picker: OptionButton = ws._label_class_picker
		if is_instance_valid(picker):
			var shown := String(picker.get_item_text(picker.selected)) if picker.selected >= 0 else "<none>"
			var want := String(ws._label_class_spec(key).get("label", ""))
			_ok(shown == want,
				"the Class dropdown follows the row press (reads %s, want %s)" % [shown, want])
		else:
			_ok(false, "the Class dropdown was not retained -- it cannot follow anything")
	# -- 3. the role-defaults controls, both axes -----------------------------
	_target("role Size mode choice", ws._label_role_mode)
	_target("role Apply button", ws._label_role_apply)
	_ok(ws._label_role_line != null and is_instance_valid(ws._label_role_line),
		"the per-role tally line exists")

	# -- 4. halo and tracking are LIVE, not dashed ----------------------------
	var dials: Array = ws._label_class_fields
	_ok(dials.size() == 3, "three class dials")
	for pair in [[0, "size"], [1, "halo"], [2, "tracking"]]:
		var s: HSlider = (dials[int(pair[0])] as Dictionary)["slider"]
		_ok(s.editable, "the %s dial is editable -- it is bound, not dashed" % pair[1])

	# -- 5. the three fields that really are absent ---------------------------
	var dead := _dead_rows()
	for nm in ["Font family", "Weight", "Case"]:
		var row: Control = dead.get(nm)
		_ok(row != null, "%s is drawn as a dashed row" % nm)
		if row == null:
			continue
		_ok(row.modulate.a < 1.0, "%s is dimmed (%.2f)" % [nm, row.modulate.a])
		_ok(not String(row.tooltip_text).is_empty(), "%s carries its reason" % nm)
		var dash: Label = null
		for c in row.get_children():
			if c is Label and String((c as Label).text) == "--":
				dash = c
		_ok(dash != null, "%s shows an em dash, not a plausible value" % nm)
	_ok(not dead.has("Halo") and not dead.has("Tracking"),
		"halo and tracking are NOT among the dashed rows")

	if not behaviour:
		return

	# -- 6. the disagreement rule, driven end to end --------------------------
	print("LR -- role defaults: creation, non-overwrite, explicit apply --")
	bridge.label_clear_all()
	ws._rebuild_label_panel()
	await _frames(3)

	## Select Water, set its mode to Fixed, and create two labels in it.
	((rows["water"] as Dictionary)["btn"] as Button).emit_signal("pressed")
	await _frames(2)
	ws._label_role_mode.selected = 0
	ws._label_role_mode.item_selected.emit(0)
	await _frames(2)
	var water_base := float(ws._label_class_spec("water").get("size", -1.0))
	var gs: Vector2i = bridge.grid_size()
	await _create_label("Inner Sea", gs.x * 0.30, gs.y * 0.40)
	await _create_label("Long Lake", gs.x * 0.55, gs.y * 0.45)

	## And one in Region, at that role's own default mode, to prove the
	## defaults are per role rather than global. NOT Landmark: since
	## 2026-09-21 `LABEL_PRACTICAL_SIZE_MODE_BY_CLASS` seeds Landmark (and
	## Settlement) to "fixed" out of the box -- the same practical default
	## `map_overlay.gd` applies to the GENERATED pass's own labels of those
	## roles -- so it would no longer diverge from Water's explicitly-set
	## "fixed" here. Region keeps the untouched "zoom" default and still
	## proves the point.
	((rows["region"] as Dictionary)["btn"] as Button).emit_signal("pressed")
	await _frames(2)
	await _create_label("Old Spire", gs.x * 0.70, gs.y * 0.60)

	var list: Array = bridge.label_list()
	_ok(list.size() == 3, "three labels created (%d)" % list.size())
	var by_name: Dictionary = {}
	for e in list:
		by_name[String((e as Dictionary).get("text", ""))] = e
	for nm in ["Inner Sea", "Long Lake"]:
		var d: Dictionary = by_name.get(nm, {})
		var idx := int(d.get("index", -1))
		_ok(String(bridge.label_class_of(idx)) == "water",
			"%s was created in the selected role (water, got %s)" % [nm, bridge.label_class_of(idx)])
		_ok(is_equal_approx(float(d.get("size", -1.0)), water_base),
			"%s took the role's base size %.0f (got %.1f)" % [nm, water_base, float(d.get("size", -1.0))])
		_ok(String(d.get("size_mode", "")) == "fixed",
			"%s took the role's size mode fixed (got %s)" % [nm, d.get("size_mode", "")])
	var spire: Dictionary = by_name.get("Old Spire", {})
	_ok(String(bridge.label_class_of(int(spire.get("index", -1)))) == "region",
		"Old Spire was created in region, not in water")
	_ok(String(spire.get("size_mode", "")) == "zoom",
		"Old Spire took region's own mode (zoom), so the mode is per role (got %s)" % spire.get("size_mode", ""))

	# -- 6b. the label-list row this pass adds a cell to ----------------------
	##
	## Not a new control, and measured anyway: this pass puts the `role · base`
	## cell into that row, and `MISTAKES.md`'s floor rule is a floor on the
	## TARGET, both axes. `tablet_fit()` floors height only and runs once, from
	## `register_workspace()`'s deferred pass -- these rows are built later, by
	## `_rebuild_label_panel()`, so at tablet density they are reached by
	## neither fitter and nothing else would ever measure them.
	for c in ws._label_list_body.get_children():
		if not (c is HBoxContainer):
			continue
		for b in (c as Control).get_children():
			if b is Button:
				_target("label row '%s'" % String((b as Button).text), b)
		break   ## One row is the shape of all of them.
	## The per-label edit form's colour well, for the same reason: a
	## `ColorPickerButton` is a `BaseButton`, it is built by
	## `_rebuild_label_edit_form()` after the one deferred `tablet_fit()`, and it
	## sits on the surface this pass edits.
	var stack2: Array = [ws._label_edit_body]
	while not stack2.is_empty():
		var n: Node = stack2.pop_back()
		for c in n.get_children():
			stack2.append(c)
			if c is ColorPickerButton:
				_target("label form colour well", c)

	# -- 7. the per-role placed count ----------------------------------------
	var cells: Dictionary = ws._label_class_count_cells
	_ok(String((cells["water"] as Dictionary)["placed"].text) == "2",
		"water's placed column reads 2 (got %s)" % (cells["water"] as Dictionary)["placed"].text)
	_ok(String((cells["region"] as Dictionary)["placed"].text) == "1",
		"region's placed column reads 1 (got %s)" % (cells["region"] as Dictionary)["placed"].text)
	_ok(String((cells["landmark"] as Dictionary)["placed"].text) == "0",
		"landmark's placed column reads a real 0, not a dash (got %s)" % (cells["landmark"] as Dictionary)["placed"].text)

	# -- 8. a per-label edit is NOT overwritten by the role default -----------
	var inner: int = int((by_name["Inner Sea"] as Dictionary).get("index", -1))
	bridge.label_set(inner, {"size": 41.0})
	ws._rebuild_label_panel()
	await _frames(3)
	((rows["water"] as Dictionary)["btn"] as Button).emit_signal("pressed")
	await _frames(2)
	## Move the role's base. The label that was edited by hand must not follow.
	var size_dial: HSlider = (ws._label_class_fields[0] as Dictionary)["slider"]
	var moved: float = clampf(water_base + 4.0, size_dial.min_value, size_dial.max_value)
	size_dial.value = moved
	ws._regenerate_labels()
	await _frames(6)
	_ok(is_equal_approx(float(bridge.label_get(inner).get("size", -1.0)), 41.0),
		"moving the role's base did NOT overwrite the hand-edited label (still %.1f)"
			% float(bridge.label_get(inner).get("size", -1.0)))
	var long_lake: int = int((by_name["Long Lake"] as Dictionary).get("index", -1))
	_ok(not is_equal_approx(float(bridge.label_get(long_lake).get("size", -1.0)), moved),
		"and did not overwrite the untouched one either -- a default is not a fallback")

	## The list has to SAY which labels are at the base and which are not.
	var marks := _role_marks()
	_ok(marks.has("water · %dpx" % 41) or marks.has("water · 41px"),
		"the list marks the hand-edited label with its own size (%s)" % str(marks.keys()))
	_ok(marks.size() >= 2, "the list marks every label with its role (%d rows)" % marks.size())

	# -- 9. the explicit apply, and only it, overwrites -----------------------
	var before := String(ws._label_role_apply.text)
	_ok(before.contains("(2)"), "Apply names the count it will change (%s)" % before)
	ws._label_role_apply.emit_signal("pressed")
	await _frames(6)
	for idx in [inner, long_lake]:
		_ok(is_equal_approx(float(bridge.label_get(idx).get("size", -1.0)), moved),
			"Apply set label %d to the role base %.0f (got %.1f)"
				% [idx, moved, float(bridge.label_get(idx).get("size", -1.0))])
	_ok(is_equal_approx(float(bridge.label_get(int(spire.get("index", -1))).get("size", -1.0)),
			float(ws._label_class_spec("region").get("size", -1.0))),
		"Apply left the region label alone -- it reaches one role, not every label")
	_ok(ws._label_role_apply.disabled,
		"Apply disables itself once every label in the role is at the base")
	_ok(String(ws._label_role_line.text).contains("2 already at the base"),
		"the tally line says how many are at the base (%s)" % ws._label_role_line.text)
	var after := _role_marks()
	var at_base := 0
	for k in after:
		if String(k).ends_with("· base"):
			at_base += 1
	_ok(at_base == 2, "both water labels now read `water · base` (%d)" % at_base)

	bridge.label_clear_all()
	ws._rebuild_label_panel()
	await _frames(3)
	_ok(String((cells["water"] as Dictionary)["placed"].text) == "0",
		"clearing every label takes the placed column back to 0")

## Every `role · …` cell in the Region labels list, as text -> count.
func _role_marks() -> Dictionary:
	var out: Dictionary = {}
	var stack: Array = [ws._label_list_body]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children():
			stack.append(c)
			if c is Label:
				var t := String((c as Label).text)
				if t.contains(" · ") and (t.ends_with("base") or t.ends_with("px")):
					out[t] = int(out.get(t, 0)) + 1
	return out

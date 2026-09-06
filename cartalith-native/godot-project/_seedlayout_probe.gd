extends Node
## `Window ▸ Layouts` -- WI-01's first-run seeding, and the save modal's live
## readout. `design/round3-corrected/SaveLayout.dc.html` C2 and C3.
##
## **Two processes, on purpose.** "Seed once, not every launch" is a claim
## about what survives a relaunch, and one process cannot make it: calling the
## seeder twice in a row proves the in-memory `ConfigFile` remembers, which is
## not the question. So phase 1 wipes the config to a genuine first run, boots
## `app.tscn`, checks the four seeds, forgets one, and **leaves the config on
## disk**; phase 2 is a second Godot process that boots the same app over that
## file and checks the forgotten one stayed forgotten.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _seedlayout_probe.tscn
##   Godot_v4.7.1-stable_win64_console.exe --path . _seedlayout_probe.tscn -- phase2
##
## **Windowed, not `--headless`.** Nothing here samples pixels, but the modal
## and the menus are laid out for real and `phone_present()`/`popup_centered()`
## want a display. Density is reported in the banner and is `pointer` on this
## machine.
##
## The install's own `cartalith_settings.cfg` is copied to `BACKUP` before
## phase 1 writes anything and restored by phase 2, which then deletes the
## copy. If phase 2 never runs, `BACKUP` is the file to put back.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row.

const BACKUP := "user://_seedlayout_probe_backup.bin"

var _app: DccApp
var _fail := 0
var _phase2 := false


func _p(s: String) -> void:
	print("SEEDLAYOUT  %s" % s)


func _ok(name: String, got: Variant, want: Variant) -> void:
	var pass_ := str(got) == str(want)
	if not pass_:
		_fail += 1
	print("SEEDLAYOUT  %s  %-46s got=%s want=%s"
		% ["ok  " if pass_ else "FAIL", name, str(got), str(want)])


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


## Depth-first by node name and class, so nothing here reaches into a private
## field: `_layouts_popup.name` is `"Layouts"` and the Window `MenuButton`'s
## text is `"Window"`, both set by the code under test.
func _find(n: Node, pred: Callable) -> Node:
	if pred.call(n):
		return n
	for c in n.get_children(true):
		var hit := _find(c, pred)
		if hit != null:
			return hit
	return null


func _popup_titles(pm: PopupMenu) -> Array:
	var out: Array = []
	for i in pm.item_count:
		if not pm.is_item_separator(i):
			out.append(pm.get_item_text(i))
	return out


func _layouts_popup() -> PopupMenu:
	return _find(_app, func(n): return n is PopupMenu and n.name == "Layouts") as PopupMenu


func _window_popup() -> PopupMenu:
	var mb := _find(_app, func(n): return n is MenuButton and n.text == "Window") as MenuButton
	return mb.get_popup() if mb != null else null


## Every `KEY -> value` pair a `_layout_readout()` block drew, read back off the
## rendered labels rather than off the dictionary that produced them -- a
## readout that agrees with its own input proves nothing about what is on
## screen.
func _readout_rows(root: Node) -> Dictionary:
	var out := {}
	## Breadth-first in child order, so the returned Dictionary's key order is
	## the order the rows are drawn in -- `pop_back()` would reverse it and the
	## literal-key assertion below would be pinning an artefact of the walk.
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_front()
		if n is HBoxContainer and n.get_child_count() == 2 \
				and n.get_child(0) is Label and n.get_child(1) is Label:
			out[(n.get_child(0) as Label).text] = (n.get_child(1) as Label).text
		for c in n.get_children(true):
			stack.append(c)
	return out


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_phase2 = Array(OS.get_cmdline_user_args()).has("phase2")
	var cfg := DccSettings.CONFIG_PATH
	if not _phase2:
		## Save the install's file, then hand the app an empty one: an absent
		## `[layout] seeded` key IS the first-run state, and this is the only
		## way to reach it on a machine that has already run the shell.
		## **Refuse to overwrite an existing backup.** A phase 1 re-run after a
		## phase 2 that never happened would otherwise back up the probe's own
		## wiped config over the install's real one and lose it silently.
		if FileAccess.file_exists(BACKUP):
			_p("ABORT  %s already exists -- a previous phase 1 was never closed by a phase 2."
				% ProjectSettings.globalize_path(BACKUP))
			_p("       Copy it over cartalith_settings.cfg and delete it, then re-run.")
			get_tree().quit(2)
			return
		var original := FileAccess.get_file_as_bytes(cfg)
		var b := FileAccess.open(BACKUP, FileAccess.WRITE)
		b.store_buffer(original)
		b.close()
		var f := FileAccess.open(cfg, FileAccess.WRITE)
		f.store_string("")
		f.close()
		_p("phase 1 -- config wiped to first run (backed up %d bytes)" % original.size())
	else:
		_p("phase 2 -- reopened over the config phase 1 left behind")

	_app = load("res://shell/app.tscn").instantiate() as DccApp
	add_child(_app)
	await get_tree().create_timer(1.5).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(4)

	_p("density: touch=%s phone=%s tablet=%s -> pointer=%s"
		% [str(DccTheme.is_touch()), str(DccTheme.is_phone()),
			str(DccTheme.is_tablet()),
			str(not DccTheme.is_touch() and not DccTheme.is_phone())])

	var lp := _layouts_popup()
	if lp == null:
		_p("FAIL  no PopupMenu named \"Layouts\" under the app")
		_finish()
		return
	lp.about_to_popup.emit()
	await _frames(2)
	var titles := _popup_titles(lp)
	_p("Window ▸ Layouts = %s" % str(titles))

	if _phase2:
		await _phase_two(titles)
	else:
		await _phase_one(titles)
	_finish()


# -- phase 1 ------------------------------------------------------------------

func _phase_one(titles: Array) -> void:
	# 1 -- the four seeds exist on a first run, and Default still leads.
	_ok("seeded flag set after first boot", DccSettings.layouts_seeded(), true)
	_ok("first row is still the built-in", titles[0], "Default (reset layout)")
	for row in DccMenus.SEED_LAYOUTS:
		_ok("submenu lists \"%s\"" % String(row["name"]),
			titles.has(String(row["name"])), true)
	_ok("no sixth row was invented", titles.size(),
		DccMenus.SEED_LAYOUTS.size() + 3)  # Default + 4 seeds + Save + Forget

	# 1b -- the five region rows, as LITERALS read off the live Window menu.
	#       `WIN_REGION_LABELS` cannot be checked against itself, and this is
	#       the finding the artboard got wrong: the fifth region is the Domain
	#       rail. There is no "Tool options" toggle in this menu.
	var wp0 := _window_popup()
	var region_rows: Array = []
	for i in wp0.item_count:
		if wp0.is_item_checkable(i) and DccMenus.WIN_REGION_IDS.has(wp0.get_item_id(i)):
			region_rows.append(wp0.get_item_text(i))
	_ok("Window menu's five region rows", str(region_rows),
		str(["Left dock", "Right dock", "Timeline", "Status bar", "Domain rail"]))

	# 2 -- every seed names a REAL rail node, checked against RAIL_NODES' own
	#      labels rather than against the constant that produced it. This is
	#      the assertion the artboard's `pipeline`/`sculpt` would have failed.
	var want_labels := {
		"Generate": "Generation pipeline", "Sculpt": "Sculpt",
		"Cartography": "Layers & style", "Journey": "Journey planner",
	}
	for row in DccMenus.SEED_LAYOUTS:
		var name := String(row["name"])
		var stored := DccSettings.layout(name)
		var node := DccShell.rail_node(String(stored.get("domain", "")),
			String(stored.get("mode", "")))
		_ok("%s -> a real RAIL_NODES node" % name, not node.is_empty(), true)
		_ok("%s -> node label" % name, String(node.get("label", "")), want_labels[name])
		_ok("%s stores five region flags" % name,
			(stored.get("regions", {}) as Dictionary).size(), DccMenus.WIN_REGION_IDS.size())
		_ok("%s stores rail collapsed" % name, bool(stored.get("rail_expanded", true)), false)

	# 3 -- and applying one actually moves the shell. A seed whose strings are
	#      wrong restores nothing and says nothing, so this is the check that
	#      a mutation of SEED_LAYOUTS' mode strings cannot survive.
	var lpop := _layouts_popup()
	var idx := _popup_titles(lpop).find("Sculpt")
	_app.select_domain_mode("cartography", "labels")
	await _frames(2)
	lpop.id_pressed.emit(DccMenus.ID_WIN_LAYOUT_FIRST + idx)
	await _frames(4)
	_ok("applying Sculpt selects its domain", _app.active_domain(), "world")
	_ok("applying Sculpt selects its mode", _app.active_mode(), "b")

	# 4 -- the live readout, against a state this probe sets by hand.
	await _readout_checks()

	# 5 -- forget one, and leave the config for phase 2.
	DccSettings.forget_layout("Sculpt")
	lpop.about_to_popup.emit()
	await _frames(2)
	_ok("Sculpt gone from the submenu immediately",
		_popup_titles(lpop).has("Sculpt"), false)
	_ok("...and the seeded flag is still set", DccSettings.layouts_seeded(), true)
	_p("phase 1 done -- run again with `-- phase2` to check it stays forgotten")


# -- phase 2 ------------------------------------------------------------------

func _phase_two(titles: Array) -> void:
	_ok("Sculpt STILL absent after a reopen", titles.has("Sculpt"), false)
	_ok("...and was not re-seeded into the store",
		DccSettings.layouts().has("Sculpt"), false)
	for name in ["Generate", "Cartography", "Journey"]:
		_ok("%s survived the reopen" % name, titles.has(name), true)
	_ok("seeded flag persisted to disk", DccSettings.layouts_seeded(), true)

	## Put the install back exactly as phase 1 found it.
	var original := FileAccess.get_file_as_bytes(BACKUP)
	var f := FileAccess.open(DccSettings.CONFIG_PATH, FileAccess.WRITE)
	f.store_buffer(original)
	f.close()
	_ok("install config restored byte-for-byte",
		FileAccess.get_file_as_bytes(DccSettings.CONFIG_PATH) == original, true)
	DirAccess.remove_absolute(ProjectSettings.globalize_path(BACKUP))


# -- the readout --------------------------------------------------------------

func _readout_checks() -> void:
	var menus: DccMenus = _app.menus
	var ids := DccMenus.WIN_REGION_IDS

	# 4a -- a hand-made snapshot: every row's live branch, including a hidden
	#       region, so "shown" is not the only answer the block can give.
	var host := VBoxContainer.new()
	add_child(host)
	var regions := {}
	for rid in ids:
		regions[rid] = true
	regions[DccMenus.ID_WIN_RIGHT] = false
	menus._layout_readout(host, {
		"regions": regions, "domain": "civilization", "mode": "planner",
		"rail_expanded": true, "detent": "half",
	})
	await _frames(2)
	var rows := _readout_rows(host)
	_ok("readout: left dock", rows.get("LEFT DOCK", ""), "shown")
	_ok("readout: right dock reads hidden", rows.get("RIGHT DOCK", ""), "hidden")
	_ok("readout: domain · mode", rows.get("DOMAIN · MODE", ""), "CIVIL · planner")
	_ok("readout: rail", rows.get("RAIL", ""), "expanded")
	_ok("readout: dock widths dashed", rows.get("DOCK WIDTHS", ""), "—")
	## `detent` is in the snapshot and the row is still absent, because this is
	## not the phone composition. The row is not "missing" -- it is scoped.
	_ok("readout: no tool-sheet row at pointer density",
		rows.has("TOOL SHEET"), false)
	host.queue_free()

	# 4b -- the other branch of every dashable row: an empty snapshot. Each
	#       dash must carry a reason, and the reason is what a reader gets.
	var host2 := VBoxContainer.new()
	add_child(host2)
	menus._layout_readout(host2, {})
	await _frames(2)
	var rows2 := _readout_rows(host2)
	for rid in ids:
		_ok("empty snap: %s dashed" % String(DccMenus.WIN_REGION_LABELS[rid]).to_upper(),
			rows2.get(String(DccMenus.WIN_REGION_LABELS[rid]).to_upper(), ""), "—")
	## Literal keys, not `WIN_REGION_LABELS` upper-cased: the loop above reads
	## the constant that produced the rows, so only this can catch a rename.
	_ok("readout keys are the five region names", str(rows2.keys()),
		str(["LEFT DOCK", "RIGHT DOCK", "TIMELINE", "STATUS BAR", "DOMAIN RAIL",
			"DOMAIN · MODE", "RAIL", "DOCK WIDTHS"]))
	_ok("empty snap: domain · mode dashed", rows2.get("DOMAIN · MODE", ""), "—")
	_ok("empty snap: rail dashed", rows2.get("RAIL", ""), "—")
	var reasoned := 0
	var stack: Array = [host2]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is HBoxContainer and n.tooltip_text.strip_edges() != "":
			reasoned += 1
		for c in n.get_children(true):
			stack.append(c)
	_ok("every dash carries a reason", reasoned, ids.size() + 3)  # 5 + domain + rail + widths
	host2.queue_free()

	# 4c -- the live path: a state set on the shell, read back through the
	#       modal the user actually opens.
	var wp := _window_popup()
	wp.about_to_popup.emit()
	await _frames(2)
	wp.id_pressed.emit(DccMenus.ID_WIN_RIGHT)      # hide the right dock
	## Domain first, rail second, and the order is not arbitrary:
	## `DccShell._select_domain()` collapses the rail on every domain change
	## (`ENV:2054`'s `{domain:d, railExp:false}`), so expanding first and
	## selecting after would measure the collapse, not the readout. This is
	## also why `_apply_layout()` restores `rail_expanded` LAST -- it would
	## otherwise undo itself on every layout that changes domain.
	_app.select_domain("civilization")
	_app.select_domain_mode("civilization", "planner")
	await _frames(4)
	_app.set_rail_expanded(true)
	await _frames(4)
	_ok("rail really is expanded before the modal opens",
		_app.is_rail_expanded(), true)
	menus._prompt_save_layout()
	await _frames(6)
	var dlg := _find(_app, func(n): return n is ConfirmationDialog \
		and (n as ConfirmationDialog).title == "Save layout as") as ConfirmationDialog
	if dlg == null:
		_ok("save modal opened", false, true)
		return
	var live := _readout_rows(dlg)
	_ok("live: right dock hidden as set", live.get("RIGHT DOCK", ""), "hidden")
	_ok("live: left dock still shown", live.get("LEFT DOCK", ""), "shown")
	_ok("live: rail expanded as set", live.get("RAIL", ""), "expanded")
	_ok("live: domain · mode as set", live.get("DOMAIN · MODE", ""), "CIVIL · planner")

	# 4d -- the collision line and the Replace wording.
	var le := _find(dlg, func(n): return n is LineEdit) as LineEdit
	## Four seeds are in the store and none of them is called "Layout 1", so
	## the first free name is still 1 -- the artboard's own pre-fill. A
	## `count + 1` counter would offer "Layout 5" here.
	_ok("pre-fill is the first FREE Layout N", le.text, "Layout 1")
	var warn := _find(dlg, func(n): return n is Label \
		and (n as Label).text.begins_with("\"")) as Label
	le.text = "Generate"
	le.text_changed.emit("Generate")
	await _frames(2)
	_ok("existing name -> Replace", dlg.ok_button_text, "Replace")
	warn = _find(dlg, func(n): return n is Label and (n as Label).visible \
		and (n as Label).text.contains("already exists")) as Label
	_ok("existing name -> warning shown", warn != null, true)
	le.text = "Nonesuch"
	le.text_changed.emit("Nonesuch")
	await _frames(2)
	_ok("free name -> Save", dlg.ok_button_text, "Save")
	var still := _find(dlg, func(n): return n is Label and (n as Label).visible \
		and (n as Label).text.contains("already exists"))
	_ok("free name -> warning hidden", still == null, true)
	dlg.hide()
	dlg.queue_free()
	await _frames(2)

	## Restore the two regions this block moved, so phase 1's later checks see
	## the shell it started with.
	wp.id_pressed.emit(DccMenus.ID_WIN_RIGHT)
	_app.set_rail_expanded(false)
	await _frames(2)


func _finish() -> void:
	_p("%s (%d failed)" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(_fail)

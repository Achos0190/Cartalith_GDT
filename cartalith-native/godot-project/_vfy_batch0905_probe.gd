extends Node
## INDEPENDENT verifier for the 2026-09-05 three-lane batch. Written to REFUTE,
## not to reproduce the lanes' own probes: every locator, every literal and
## every deletion here is derived from the shipped source rather than copied
## from `_structmove_probe.gd`, `_lanebpickers_probe.gd` or `_vaultgone_probe.gd`.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _vfy_batch0905_probe.tscn

const SEED := 771203

var app: Node
var _fail := 0
var _n := 0

func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("VFY  ok    %s" % label)
	else:
		_fail += 1
		print("VFY  FAIL  %s   %s" % [label, detail])

func _frames(k: int) -> void:
	for i in k:
		await get_tree().process_frame

func _gather(n: Node, out: Array, cls: String) -> void:
	if n.is_class(cls) or (cls == "MenuButton" and n is MenuButton) \
			or (cls == "PopupMenu" and n is PopupMenu):
		out.append(n)
	for c in n.get_children(true):
		_gather(c, out, cls)

func _menu(title: String) -> PopupMenu:
	var out: Array = []
	_gather(app, out, "MenuButton")
	for mb in out:
		if String((mb as MenuButton).text) == title:
			return (mb as MenuButton).get_popup()
	return null

func _popup_named(nm: String) -> PopupMenu:
	var out: Array = []
	_gather(app, out, "PopupMenu")
	for p in out:
		if String((p as Node).name) == nm:
			return p
	return null

## Every row text in a popup and, recursively, in the popups its submenu items
## name. Written fresh so a row that merely moved into a child popup is still
## found -- the failure mode a flat "rows of this popup" check would miss.
func _rows_deep(p: PopupMenu, out: Array) -> Array:
	if p == null:
		return out
	for i in p.item_count:
		out.append(p.get_item_text(i))
		var sub := p.get_item_submenu(i)
		if sub != "":
			_rows_deep(_popup_named(sub), out)
	return out

func _count_containing(rows: Array, needle: String) -> int:
	var k := 0
	for r in rows:
		if String(r).find(needle) >= 0:
			k += 1
	return k

func _texts(n: Node, out: Array) -> Array:
	if n is Label:
		out.append(String((n as Label).text))
	elif n is Button:
		out.append(String((n as Button).text))
	for c in n.get_children(true):
		_texts(c, out)
	return out

func _nodes_of(root: Node, out: Array, pred: Callable) -> Array:
	if pred.call(root):
		out.append(root)
	for c in root.get_children(true):
		_nodes_of(c, out, pred)
	return out

func _browsers(host: Node) -> Array:
	return _nodes_of(host, [], func(n): return n is DccBrowseDialog)

func _confirms(host: Node) -> Array:
	return _nodes_of(host, [], func(n): return n is ConfirmationDialog)

func _button_named(root: Node, text: String) -> Button:
	for b in _nodes_of(root, [], func(n): return n is Button):
		if String((b as Button).text) == text:
			return b
	return null


func _title_hits(idx: CommandIndex, q: String) -> int:
	var k := 0
	for r in idx.search(q):
		if String(r["title"]).to_lower().find(q) >= 0:
			k += 1
	return k

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	if app.get_script() == null or not app.has_method("open_journey_planner"):
		print("VFY  ABORT: the shell did not build")
		get_tree().quit(2)
		return
	var bridge = app.bridge
	bridge.generate({
		"seed": SEED, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(3)

	# =====================================================================
	# LANE A
	# =====================================================================
	var data := _menu("Data")
	_ok("A: a Data menu exists at all", data != null)
	var data_rows := _rows_deep(data, [])
	_ok("A1 gone: no `Journey planner` row anywhere under Data (incl. submenus)",
		_count_containing(data_rows, "Journey planner") == 0, str(data_rows))
	_ok("A1 control: Data still carries World data tables",
		_count_containing(data_rows, "World data") >= 1)

	## The whole MenuBar, deep. A row that moved to another menu instead of the
	## rail would show up here and would NOT be a vanished command.
	var all_menu_rows: Array = []
	for t in ["File", "Edit", "View", "Data", "Assets", "Generate", "Preferences", "Window", "Help"]:
		var m := _menu(t)
		if m != null:
			_rows_deep(m, all_menu_rows)
	_ok("A1 gone (whole menu bar): `Journey planner` is in no menu at all",
		_count_containing(all_menu_rows, "Journey planner") == 0)
	_ok("A2 gone (whole menu bar): `Refine detail` is in no menu at all",
		_count_containing(all_menu_rows, "Refine detail") == 0)

	## --- the command index, built the way the search field builds it -------
	var idx := CommandIndex.new()
	idx.build(app, bridge)
	## **TITLE matches only.** `CommandIndex.search()` also matches a row's
	## blurb, and counting those reported a false PASS for `Refine detail`: the
	## two hits were `Clear atlas cache now` and `Export atlas`, whose tooltips
	## were rewritten in this same pass to say *"or Refine detail beside it on
	## the WORLD tool-options bar"*. Neither is the command.
	_ok("A: command index still finds `Journey planner`", _title_hits(idx, "journey planner") > 0,
		"%d title matches" % _title_hits(idx, "journey planner"))
	_ok("A: command index still finds `Refine detail`", _title_hits(idx, "refine detail") > 0,
		"%d title matches" % _title_hits(idx, "refine detail"))
	for kept in ["validate pack", "export pack", "pack metadata", "clear library"]:
		_ok("A control: `%s` is still indexed" % kept, _title_hits(idx, kept) > 0)
	## Nine EDIT/BATCH search terms the flatten dropped. Sampled, not all nine.
	for term in ["sprite sheet slicer", "collect into set", "slot transform", "duplicate"]:
		_ok("A3 index: `%s` still findable" % term, _title_hits(idx, term) > 0,
			"%d title matches" % _title_hits(idx, term))

	## --- Help > Keyboard shortcuts -----------------------------------------
	app.shortcuts_dialog.open()
	await _frames(2)
	var sc_texts := _texts(app.shortcuts_dialog, [])
	_ok("A: Keyboard shortcuts lists Shift+J",
		_count_containing(sc_texts, "Shift+J") > 0 or _count_containing(sc_texts, "⇧J") > 0,
		"not listed among %d strings" % sc_texts.size())
	_ok("A control: it does list Esc", _count_containing(sc_texts, "Esc") > 0)
	app.shortcuts_dialog.hide()
	await _frames(1)

	## --- Shift+J still opens the planner ------------------------------------
	app.arm_tool("inspect")
	var ev := InputEventKey.new()
	ev.keycode = KEY_J
	ev.pressed = true
	ev.shift_pressed = true
	app._unhandled_key_input(ev)
	await _frames(3)
	_ok("A1 accelerator: Shift+J arms the journey tool",
		String(app.armed_tool) == "journey", "armed_tool=%s" % String(app.armed_tool))
	_ok("A1 accelerator: and it lit the CIVIL rail node it now lives on",
		String(app._domain_mode.get("civilization", "")) == "planner",
		str(app._domain_mode))

	## negative control -- bare J must arm nothing
	app.arm_tool("inspect")
	var ev2 := InputEventKey.new()
	ev2.keycode = KEY_J
	ev2.pressed = true
	app._unhandled_key_input(ev2)
	await _frames(2)
	_ok("A1 control: bare J arms nothing", String(app.armed_tool) == "inspect",
		String(app.armed_tool))

	## --- the rail node itself ----------------------------------------------
	app.select_domain("world")
	await _frames(2)
	app._on_rail_node_pressed("civilization", "planner")
	await _frames(3)
	_ok("A1 new home: the CIVIL `planner` rail node opens the planner",
		String(app.armed_tool) == "journey" and String(app._active_domain) == "civilization",
		"tool=%s domain=%s" % [String(app.armed_tool), String(app._active_domain)])

	## --- the sibling-node release Lane A claims to have fixed --------------
	app._on_rail_node_pressed("civilization", "landmarks")
	await _frames(3)
	_ok("A1 release: a sibling CIVIL node releases the journey takeover",
		String(app.armed_tool) != "journey", "still armed_tool=%s" % String(app.armed_tool))

	## --- Move 2: Refine detail at its new home ------------------------------
	var atlas := _popup_named("AtlasCache")
	_ok("A2: the Atlas cache submenu still exists", atlas != null)
	if atlas != null:
		var arows := _rows_deep(atlas, [])
		_ok("A2 gone: no Refine row in Atlas cache", _count_containing(arows, "Refine") == 0, str(arows))
		_ok("A2 control: Atlas cache still has Export/Import/Clear",
			_count_containing(arows, "Export") >= 1 and _count_containing(arows, "Import") >= 1
				and _count_containing(arows, "Clear") >= 1, str(arows))

	app.select_domain_mode("world", "generate")
	await _frames(3)
	var refine := _button_named(app, "Refine detail")
	_ok("A2 new home: a `Refine detail` button is on the WORLD tool-options bar", refine != null)
	if refine != null:
		## Literals, never `== DccMenus.REFINE_TOOLTIP`: asserting a constant
		## against itself passes however the constant is mutated. Shown red by
		## replacing "does NOT speed up panning back" in `menus.gd` with a
		## sentinel (file restored, sha256 identical, zero residue).
		_ok("A2 new home: its tooltip is the shared refine text",
			String(refine.tooltip_text).find("does NOT speed up panning back") >= 0
				and String(refine.tooltip_text).find("Zoom in first") >= 0,
			String(refine.tooltip_text).substr(0, 60))
		_ok("A2 touch target: >= 44 px tall", refine.size.y >= 44.0,
			"%.1f x %.1f px" % [refine.size.x, refine.size.y])
		refine.pressed.emit()
		await _frames(3)
		var st := _texts(app, [])
		_ok("A2 works: pressing it reaches menus.refine_current_view()",
			_count_containing(st, "nothing to refine at this zoom") > 0
				or _count_containing(st, "refined") > 0
				or _count_containing(st, "no world on screen") > 0,
			"no refine status sentence on screen")

	## --- Move 3: the flat nine ----------------------------------------------
	var ap := _popup_named("AssetPack")
	_ok("A3: the Asset pack submenu exists", ap != null)
	if ap != null:
		ap.about_to_popup.emit()
		await _frames(1)
		var aprows: Array = []
		for i in ap.item_count:
			aprows.append(ap.get_item_text(i))
		_ok("A3: exactly nine rows", ap.item_count == 9, "%d rows: %s" % [ap.item_count, str(aprows)])
		_ok("A3: no MB/GB invented on the FILLED row",
			_count_containing(aprows, "MB") == 0 and _count_containing(aprows, "GB") == 0, str(aprows))
		_ok("A3: FILLED counts slots", _count_containing(aprows, "slots") > 0, str(aprows))

	## --- the owner's binding consequence -------------------------------------
	var assets := _menu("Assets")
	var arows2 := _rows_deep(assets, [])
	_ok("A4: exactly ONE `Clear library` row in the whole Assets menu",
		_count_containing(arows2, "Clear library") == 1,
		"%d -- %s" % [_count_containing(arows2, "Clear library"), str(arows2)])
	_ok("A4: it is marked destructive",
		_count_containing(arows2, "Clear library…   destructive") == 1, str(arows2))
	## reachable and confirming, driven through the real dispatch
	var before_items := int(bridge.as_pack_info().get("total_items", -1))
	var ci := -1
	for i in assets.item_count:
		if String(assets.get_item_text(i)).find("Clear library") >= 0:
			ci = i
	_ok("A4: the row is enabled", ci >= 0 and not assets.is_item_disabled(ci), "index %d" % ci)
	if ci >= 0:
		assets.id_pressed.emit(assets.get_item_id(ci))
		await _frames(3)
		var cd := _confirms(app.asset_library_window)
		_ok("A4: pressing it raises a ConfirmationDialog", cd.size() >= 1, "%d dialogs" % cd.size())
		_ok("A4: and clears NOTHING until it is answered",
			int(bridge.as_pack_info().get("total_items", -1)) == before_items,
			"%d -> %d" % [before_items, int(bridge.as_pack_info().get("total_items", -1))])
		for d in cd:
			(d as ConfirmationDialog).hide()
		await _frames(2)
		_ok("A4: dismissing it still clears nothing",
			int(bridge.as_pack_info().get("total_items", -1)) == before_items)

	# =====================================================================
	# LANE B -- save mode only. Open mode is the easy half.
	# =====================================================================
	var dm = app.data_manager_window
	var scratch := OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vfy-0905"
	if DirAccess.dir_exists_absolute(scratch):
		for f in DirAccess.get_files_at(scratch):
			DirAccess.remove_absolute(scratch + "/" + f)
	DirAccess.make_dir_recursive_absolute(scratch)

	## The five writing pickers, by the function that raises each.
	var save_sites := [
		["heightmap", func(): dm._pick_heightmap_destination(), "png", "heightmap.png", dm],
		["tiles .zip", func(): dm._pick_destination(), "zip", "region-tiles.zip", dm],
		["geojson", func(): dm._pick_geojson_destination(), "geojson", "world.geojson", dm],
		["map raster", func(): dm._pick_raster_destination(), "png", "map.png", dm],
		["asset pack .zip", func(): app.asset_library_window._on_export_pack(), "zip", "", app.asset_library_window],
	]
	dm._wd_tiled = false
	## `_on_export_pack()` returns before raising a picker while
	## `as_export_pack_bytes()` says *Library is empty*, so the eighth site is
	## unreachable without one imported item. Measured: without this the site
	## reports "0 browsers" and looks like a defect that is not there.
	var _img := Image.create(32, 32, false, Image.FORMAT_RGBA8)
	_img.fill(Color(0.2, 0.6, 0.9, 1.0))
	var _png := scratch + "/vfy_asset.png"
	_img.save_png(_png)
	var _slots: Array = bridge.as_family_slots("textures")
	if not _slots.is_empty():
		bridge.as_import_item(String((_slots[0] as Dictionary).get("uid", "")), "vfy",
			FileAccess.get_file_as_bytes(_png))
	_ok("B fixture: the library is non-empty, so the pack export can raise a picker",
		bool(bridge.as_export_pack_bytes().get("ok", false)),
		String(bridge.as_export_pack_bytes().get("error", "")))
	for site in save_sites:
		var label: String = site[0]
		var host: Node = site[4]
		for b in _browsers(host):
			(b as Node).free()
		(site[1] as Callable).call()
		await _frames(3)
		var bs := _browsers(host)
		_ok("B %s: exactly one browser opened" % label, bs.size() == 1, "%d" % bs.size())
		if bs.size() != 1:
			continue
		var d = bs[0]
		_ok("B %s: SAVE mode" % label, d._mode == DccBrowseDialog.PickKind.SAVE, str(d._mode))
		_ok("B %s: a filename field exists" % label, d._name_edit != null)
		_ok("B %s: the extension filter survived (%s)" % [label, site[2]],
			d._extensions.size() == 1 and String(d._extensions[0]) == String(site[2]),
			str(d._extensions))
		if String(site[3]) != "":
			_ok("B %s: the default name survived" % label,
				String(d._name_edit.text) == String(site[3]), String(d._name_edit.text))
		_ok("B %s: the foot hint is on screen and wider than 100 px" % label,
			d._foot_note != null and d._foot_note.visible and d._foot_note.size.x > 100.0,
			"note=%s w=%s" % [str(d._foot_note != null),
				("%.1f" % d._foot_note.size.x) if d._foot_note != null else "-"])
		## extension auto-append
		d.navigate(scratch)
		await _frames(2)
		d._name_edit.text = "vfy_noext"
		var appended: String = d._save_path()
		_ok("B %s: a name with no extension gets .%s appended" % [label, site[2]],
			appended.ends_with("." + String(site[2])), appended)
		## empty name -> Save disabled, and _confirm() returns without calling back
		d._name_edit.text = ""
		d._refresh_primary()
		_ok("B %s: an empty name disables Save" % label, d._primary.disabled)
		(d as Node).free()
		await _frames(1)

	## --- CANCEL must not hand the caller a path ------------------------------
	var runs_before: int = dm._runs.size()
	dm._pick_geojson_destination()
	await _frames(3)
	var bs2 := _browsers(dm)
	if bs2.size() == 1:
		var dd = bs2[0]
		dd.navigate(scratch)
		await _frames(2)
		var cancel := _button_named(dd, "Cancel")
		_ok("B cancel: a Cancel button exists", cancel != null)
		if cancel != null:
			cancel.pressed.emit()
			await _frames(3)
		_ok("B cancel: nothing was exported", dm._runs.size() == runs_before,
			"%d -> %d" % [runs_before, dm._runs.size()])
		_ok("B cancel: no file appeared in the scratch dir",
			DirAccess.get_files_at(scratch).size() == 0,
			str(DirAccess.get_files_at(scratch)))
		_ok("B cancel: the browser freed itself", _browsers(dm).size() == 0)

	## --- OVERWRITE: an existing file must interpose a confirm ---------------
	var victim := scratch + "/vfy_overwrite.geojson"
	var vf := FileAccess.open(victim, FileAccess.WRITE)
	vf.store_string("ORIGINAL-CONTENT-MUST-SURVIVE")
	vf.close()
	_ok("B overwrite: the victim file was written", FileAccess.file_exists(victim))
	var runs_b2: int = dm._runs.size()
	dm._pick_geojson_destination()
	await _frames(3)
	var bs3 := _browsers(dm)
	if bs3.size() == 1:
		var d3 = bs3[0]
		d3.navigate(scratch)
		await _frames(2)
		d3._name_edit.text = "vfy_overwrite"
		d3._refresh_primary()
		d3._confirm()
		await _frames(4)
		var cds := _confirms(app)
		var overwrite_prompt: ConfirmationDialog = null
		for c in cds:
			if String((c as ConfirmationDialog).title).find("Overwrite") >= 0:
				overwrite_prompt = c
		_ok("B overwrite: a confirm interposed", overwrite_prompt != null,
			"%d ConfirmationDialogs on the app, none titled Overwrite" % cds.size())
		_ok("B overwrite: the export did NOT run yet", dm._runs.size() == runs_b2,
			"%d -> %d" % [runs_b2, dm._runs.size()])
		var still := FileAccess.open(victim, FileAccess.READ)
		var body := still.get_as_text()
		still.close()
		_ok("B overwrite: the file is untouched while the prompt is up",
			body == "ORIGINAL-CONTENT-MUST-SURVIVE", body.substr(0, 40))
		if overwrite_prompt != null:
			_ok("B overwrite: its OK button says Overwrite",
				String(overwrite_prompt.ok_button_text) == "Overwrite",
				String(overwrite_prompt.ok_button_text))
			## dismiss -> nothing happens
			overwrite_prompt.hide()
			await _frames(3)
			var s2 := FileAccess.open(victim, FileAccess.READ)
			var b2 := s2.get_as_text()
			s2.close()
			_ok("B overwrite: dismissing writes nothing",
				b2 == "ORIGINAL-CONTENT-MUST-SURVIVE" and dm._runs.size() == runs_b2)
		## now confirm for real
		dm._pick_geojson_destination()
		await _frames(3)
		var bs4 := _browsers(dm)
		if bs4.size() == 1:
			var d4 = bs4[0]
			d4.navigate(scratch)
			await _frames(2)
			d4._name_edit.text = "vfy_overwrite"
			d4._refresh_primary()
			d4._confirm()
			await _frames(4)
			var pr: ConfirmationDialog = null
			for c in _confirms(app):
				if String((c as ConfirmationDialog).title).find("Overwrite") >= 0:
					pr = c
			if pr != null:
				pr.confirmed.emit()
				await _frames(5)
			var s3 := FileAccess.open(victim, FileAccess.READ)
			var b3 := s3.get_as_text()
			s3.close()
			_ok("B overwrite: confirming DOES replace it", b3 != "ORIGINAL-CONTENT-MUST-SURVIVE",
				"file unchanged after confirming")

	# =====================================================================
	# LANE C -- three snapshot states, deleting the PNG myself
	# =====================================================================
	var vroot := OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vfy-vault"
	if DirAccess.dir_exists_absolute(vroot):
		OS.move_to_trash(ProjectSettings.globalize_path(vroot))
	DirAccess.make_dir_recursive_absolute(vroot + "/Locations")
	var nf := FileAccess.open(vroot + "/Locations/VfyTown.md", FileAccess.WRITE)
	nf.store_string("# VfyTown\n")
	nf.close()
	var conn: Dictionary = bridge.vault_connect(vroot, "VfyVault")
	_ok("C: the scratch vault binds", bool(conn.get("ok", false)), String(conn.get("error", "")))
	var setts: Array = bridge.settlements()
	if setts.is_empty():
		_ok("C: settlements exist", false)
		_finish()
		return
	var tid := int((setts[0] as Dictionary).get("tid", 0))
	app.open_vault("settlement", tid, String((setts[0] as Dictionary).get("name", "")))
	await _frames(3)
	var vw = app.vault_window
	if vw == null:
		print("VFY  ABORT: vault window absent")
		_finish_code(2)
		return

	var t1 := _texts(vw, [])
	var c1_none := _count_containing(t1, "○ not generated")
	var c1_tick := _count_containing(t1, "✓ .")
	var c1_gone := _count_containing(t1, "✕ .")
	_ok("C state 1 (never generated): every radius reads ○, none ticks, none gone",
		c1_none == 3 and c1_tick == 0 and c1_gone == 0,
		"○%d ✓%d ✕%d" % [c1_none, c1_tick, c1_gone])

	var snap: Dictionary = bridge.vault_snapshot("settlement", tid, "local", "", 128)
	_ok("C: the local snapshot writes", bool(snap.get("ok", false)), String(snap.get("error", "")))
	var rel := String(snap.get("rel", ""))
	var abs_png := vroot + "/" + rel
	_ok("C: the PNG is on disk", FileAccess.file_exists(abs_png), abs_png)
	vw._rebuild()
	await _frames(2)
	var t2 := _texts(vw, [])
	var c2_none := _count_containing(t2, "○ not generated")
	var c2_tick := _count_containing(t2, "✓ .")
	var c2_gone := _count_containing(t2, "✕ .")
	_ok("C state 2 (present): one ticks, two still ○, none gone",
		c2_none == 2 and c2_tick == 1 and c2_gone == 0,
		"○%d ✓%d ✕%d" % [c2_none, c2_tick, c2_gone])
	_ok("C state 2: its button says Regenerate local",
		_count_containing(t2, "Regenerate local") == 1, str(t2))

	## MY OWN deletion, through DirAccess, behind the panel's back.
	var rc := DirAccess.remove_absolute(ProjectSettings.globalize_path(abs_png))
	_ok("C: I deleted the PNG myself", rc == OK and not FileAccess.file_exists(abs_png), "err=%d" % rc)
	vw._rebuild()
	await _frames(2)
	var t3 := _texts(vw, [])
	var c3_none := _count_containing(t3, "○ not generated")
	var c3_tick := _count_containing(t3, "✓ .")
	var c3_gone := _count_containing(t3, "✕ .")
	_ok("C state 3 (deleted): the tick is gone and the row is marked ✕",
		c3_none == 2 and c3_tick == 0 and c3_gone == 1,
		"○%d ✓%d ✕%d" % [c3_none, c3_tick, c3_gone])
	_ok("C state 3: the button is no longer Regenerate",
		_count_containing(t3, "Regenerate local") == 0, str(t3))
	_ok("C state 3: it offers `Generate local again`, exactly once",
		_count_containing(t3, "Generate local again") == 1,
		"%d" % _count_containing(t3, "Generate local again"))
	_ok("C: three states are three DISTINGUISHABLE renderings",
		("%d%d%d" % [c1_none, c1_tick, c1_gone]) != ("%d%d%d" % [c2_none, c2_tick, c2_gone])
			and ("%d%d%d" % [c2_none, c2_tick, c2_gone]) != ("%d%d%d" % [c3_none, c3_tick, c3_gone])
			and ("%d%d%d" % [c1_none, c1_tick, c1_gone]) != ("%d%d%d" % [c3_none, c3_tick, c3_gone]),
		"%d%d%d | %d%d%d | %d%d%d" % [c1_none, c1_tick, c1_gone, c2_none, c2_tick, c2_gone,
			c3_none, c3_tick, c3_gone])
	## and the Map checkbox is withdrawn
	var offered := false
	for fd in bridge.vault_export_fields("settlement", tid):
		if String((fd as Dictionary).get("key", "")) == "map_local":
			offered = true
	_ok("C: the Map checkbox is withdrawn for a deleted image", not offered)

	_finish()


func _finish() -> void:
	_finish_code(1 if _fail > 0 else 0)

func _finish_code(code: int) -> void:
	print("VFY  DONE  %d checks, %d FAILED" % [_n, _fail])
	get_tree().quit(code)

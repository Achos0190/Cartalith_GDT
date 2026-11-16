extends Node
## VE-6's template chooser (`VAULT_EDITOR_PROPOSAL.md` VE-6; `shell/vault_template_chooser.gd`). Windowed (refuses --headless),
## wiped user directory (`APPDATA=<empty dir>`), a throw-away vault in TEMP with three templates.
##
##   godot --path . _ve6tpl_probe.tscn -- --vp 1600x900 --out <dir>                                  (desktop)
##   ... -- --vp 1700x1000 --density=1.0 --force-touch --out <dir>                                  (tablet)
##   ... -- --vp 1080x2340 --density 2.625 --force-touch --out <dir>                                (phone)
##
## What it protects (literals written here, the fixture's own text, never a constant read back from the code):
##   W  the vault window's place panel (Ruling FJ, `VaultWindow._build_create`): the chooser lists the vault's three templates then
##      `(blank note)` last, defaults to the first template, and PREVIEWS the chosen one; picking the blank note previews `# <name>`;
##      a 30-line template previews at most 8 lines and ends `...`; the preview sits inside the window and does not widen it;
##      Create from a picked template writes that template's text (its own marker line), not another's;
##   N  (desktop, tablet) the Context panel's Note tab: the same chooser, blank first and the default, preview following the pick;
##      Create from a picked template writes that template's marker line;
##   X  a vault with NO templates: the chooser still lists `(blank note)`, previews its title line and Create makes a blank note.
## Exit 0 = held, 1 = a check failed, 2 = could not run.

var app: Node
var _vp: SubViewport
var _bridge
var _checks := 0
var _fails := 0
var _out := ""
var _root := ""


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("VE6TPL %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("VE6TPL %s" % s)


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _read(path: String) -> String:
	return FileAccess.get_file_as_string(path) if FileAccess.file_exists(path) else ""


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join(name))
	_p("shot %s" % name)


func _vis(n: Node, name: String) -> Control:
	if n.name == name and n is Control and (n as Control).is_visible_in_tree():
		return n
	for c in n.get_children():
		var r := _vis(c, name)
		if r != null:
			return r
	return null


func _btn_prefix(n: Node, prefix: String) -> Button:
	if n is Button and (n as Button).text.to_lower().begins_with(prefix.to_lower()) and (n as Button).is_visible_in_tree():
		return n
	for c in n.get_children():
		var r := _btn_prefix(c, prefix)
		if r != null:
			return r
	return null


## A user's pick in a dropdown: the selection and the signal its popup raises.
func _pick(ob: OptionButton, text: String) -> bool:
	for i in ob.item_count:
		if ob.get_item_text(i) == text:
			ob.select(i)
			ob.item_selected.emit(i)
			return true
	return false


func _items(ob: OptionButton) -> Array:
	var out: Array = []
	for i in ob.item_count:
		out.append(ob.get_item_text(i))
	return out


func _chooser_checks(tag: String, where: Node, blank_first: bool, name_: String, shots: String) -> OptionButton:
	var ob := _vis(where, "NoteTemplate") as OptionButton
	var pv := _vis(where, "TemplatePreview") as Label
	_ok("%s T0: the chooser and its preview are on screen" % tag, ob != null and pv != null)
	if ob == null or pv == null:
		return null
	var items := _items(ob)
	var want := ["(blank note)", "Templates ▸ Long", "Templates ▸ Person", "Templates ▸ Place"] if blank_first else ["Templates ▸ Long", "Templates ▸ Person", "Templates ▸ Place", "(blank note)"]
	_ok("%s T1: lists the three templates and the blank note (%s)" % [tag, "blank first" if blank_first else "blank last"], items == want, str(items))
	_ok("%s T2: default is %s" % [tag, "the blank note" if blank_first else "the first template"], ob.selected == (0 if blank_first else 0)
		and ob.get_item_text(ob.selected) == want[0], str(ob.selected))
	if blank_first:
		_ok("%s T3: the blank note previews its own title line" % tag, pv.text == "# %s" % name_, pv.text)
	else:
		_ok("%s T3: the first template previews its text" % tag, pv.text.contains("LONGMARK 1") and pv.text.contains("line 7"), pv.text)
	_ok("%s T4: Person previews its own marker line" % tag, _pick(ob, "Templates ▸ Person") and pv.text.contains("## Allegiance") and not pv.text.contains("LONGMARK"), pv.text)
	_ok("%s T5: Place previews its own marker line" % tag, _pick(ob, "Templates ▸ Place") and pv.text.contains("**Location:**") and not pv.text.contains("Allegiance"), pv.text)
	_pick(ob, "Templates ▸ Long")
	var lines := pv.text.split("\n")
	_ok("%s T6: a 30-line template previews at most 8 lines and ends with ..." % tag, lines.size() == 9 and lines[8] == "..." and not pv.text.contains("line 20"), "%d lines" % lines.size())
	_ok("%s T7: the blank note previews `# %s`" % [tag, name_], _pick(ob, "(blank note)") and pv.text == "# %s" % name_, pv.text)
	var vr := _vp.get_visible_rect()
	var gr := pv.get_global_rect()
	_ok("%s T8: the preview sits on screen, inside the viewport and below half its height in size" % tag, gr.size.y > 8.0 and gr.size.x > 100.0 and gr.end.x <= vr.size.x + 1.0
		and gr.size.y < vr.size.y * 0.5, str(gr))
	if shots != "":
		_pick(ob, "Templates ▸ Person")
		await _frames(4)
		await _shot(shots)
	return ob


func _run() -> void:
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-ve6tpl-%d" % Time.get_ticks_msec()
	var long_text := ""
	for i in range(1, 31):
		long_text += ("LONGMARK 1\n" if i == 1 else "line %d\n" % i)
	_write(_root + "/Templates/Long.md", long_text)
	_write(_root + "/Templates/Person.md", "---\ntype: person\n---\n# {{title}}\n## Allegiance\n")
	_write(_root + "/Templates/Place.md", "# {{title}}\n\n**Type:** City\n**Location:**\n")
	var s: Dictionary = _bridge.settlements()[0]
	var tid := int(s.get("tid", 0))
	var nm := String(s.get("name", ""))
	var s2: Dictionary = _bridge.settlements()[1]
	var tid2 := int(s2.get("tid", 1))
	var nm2 := String(s2.get("name", ""))
	_ok("setup: the throw-away vault binds", bool(_bridge.vault_connect(_root, "Chronicle").get("ok", false)))
	await _frames(4)
	var vw = app.vault_window
	# W: the vault window's place panel.
	_p("== W. the vault window's place panel ==")
	vw.open_for("settlement", tid, nm)
	await _frames(10)
	var ob := await _chooser_checks("W", vw, false, nm, "ve6_W_window_person.png")
	if ob != null:
		_pick(ob, "Templates ▸ Person")
		var cb := _btn_prefix(vw, "Create ")
		_ok("W C0: the Create button is there", cb != null)
		if cb != null:
			var sug: String = _bridge.vault_suggested_path("settlement", nm)
			cb.pressed.emit()
			await _frames(8)
			var made := _read(_root + "/" + sug)
			_ok("W C1: Create wrote the PICKED template (Person), title filled", made.contains("## Allegiance") and made.contains("# " + nm) and not made.contains("LONGMARK"), made.left(120))
	# N: the Note tab (the dock exists on desktop and tablet).
	if not DccTheme.is_phone():
		_p("== N. the Context panel's Note tab ==")
		vw.hide()
		await _frames(4)
		app.select("settlement", 1)
		await _frames(6)
		var tab := _vis(app, "CtxTabNote") as Button
		_ok("N0: the Note tab exists", tab != null)
		if tab != null:
			tab.pressed.emit()
			await _frames(8)
			var ob2 := await _chooser_checks("N", app, true, nm2, "ve6_N_notetab_person.png")
			if ob2 != null:
				_pick(ob2, "Templates ▸ Place")
				var nc := _vis(app, "NoteCreate") as Button
				_ok("N C0: Create is there", nc != null)
				if nc != null:
					var sug2: String = _bridge.vault_suggested_path("settlement", nm2)
					nc.pressed.emit()
					await _frames(8)
					var made2 := _read(_root + "/" + sug2)
					_ok("N C1: Create wrote the PICKED template (Place)", made2.contains("**Location:**") and not made2.contains("Allegiance"), made2.left(120))
	# X: a vault with no templates.
	_p("== X. a vault with no templates ==")
	var root2 := _root + "-none"
	_write(root2 + "/Lore/Old.md", "# Old\n")
	_ok("X0: the empty-template vault binds", bool(_bridge.vault_connect(root2, "Bare").get("ok", false)))
	await _frames(4)
	var s3: Dictionary = _bridge.settlements()[2]
	vw.open_for("settlement", int(s3.get("tid", 2)), String(s3.get("name", "")))
	await _frames(10)
	var obx := _vis(vw, "NoteTemplate") as OptionButton
	var pvx := _vis(vw, "TemplatePreview") as Label
	_ok("X1: only the blank note is listed, previewing its title line", obx != null and pvx != null and _items(obx) == ["(blank note)"]
		and pvx.text == "# %s" % String(s3.get("name", "")), str(_items(obx)) if obx != null else "none")
	await _shot("ve6_X_no_templates.png")
	var cbx := _btn_prefix(vw, "Create ")
	if cbx != null:
		var sugx: String = _bridge.vault_suggested_path("settlement", String(s3.get("name", "")))
		cbx.pressed.emit()
		await _frames(8)
		_ok("X2: Create makes the blank note", _read(root2 + "/" + sugx).strip_edges() == "# %s" % String(s3.get("name", "")), _read(root2 + "/" + sugx))
	else:
		_ok("X2: Create is there", false)


func _ready() -> void:
	var vp_size := Vector2i(1600, 900)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--vp":
			var wh := String(args[i + 1]).split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a == "--density":
			i += 2
		elif a.begins_with("--density="):
			i += 1
		elif a == "--out":
			_out = String(args[i + 1])
			i += 2
		elif a == "--force-touch":
			i += 1
		else:
			_p("ABORT unknown argument %s" % a)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		_p("ABORT run windowed")
		get_tree().quit(2)
		return
	if _out != "":
		DirAccess.make_dir_recursive_absolute(_out)
	get_tree().create_timer(900.0).timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)
	_p("form phone=%s tablet=%s vp=%s" % [DccTheme.is_phone(), DccTheme.is_tablet(), str(vp_size)])
	_bridge = app.bridge
	_bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if not _bridge.has_world:
		_ok("a world generated", false)
		get_tree().quit(2)
		return
	var t0 := Time.get_ticks_msec()
	while app.busy_overlay != null and app.busy_overlay.visible and Time.get_ticks_msec() - t0 < 30000:
		await _frames(10)
	await _frames(20)
	await _run()
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)

extends Node
## Ruling FJ (owner, 2026-11-16): the information card shows ONE note row for the selected thing, and the vault, focused on the
## place, holds create, attach, unlink and the place's linked notes. Windowed (refuses --headless), under a wiped user directory
## (`APPDATA=<empty dir>`), a throw-away vault in TEMP.
##
##   godot --path . _notefj_probe.tscn -- --vp 1600x900 --out <dir> [--tag <name>]
##   ... -- --vp 1700x1000 --density=1.0 --force-touch --out <dir>        (the tablet)
##   ... -- --vp 1080x2340 --density 2.625 --force-touch --out <dir>      (the phone)
##
## What it protects, per kind (settlement, landmark, point of interest, label, icon):
##   R  the card's rows hold exactly ONE note row (`note.open`), labelled `Note…` while none is linked and `Open note` once one
##      is, and none of the retired rows (`Create note…`, `Attach note to …`, `Open note for …`, any `el.*_note`/`civ.vault_note`);
##      the drawn card (desktop, tablet) or the peek sheet (phone) shows the same single row, and the empty map point shows none;
##   O  pressing the row opens the vault window scoped to THIS place (kind, entity id, name), not the overview;
##   C  the vault's place panel (`VaultPlaceNotes`) creates a note through the existing template form and its folder chooser: the file
##      exists and the link is bound to the place;
##   L  the panel lists the place's linked notes, and Unlink removes the link only (the file stays);
##   A  Attach an existing note: picking a note in the tree then Attach to <place> links it;
##   and after each of C, L, A the card's row reads what the store says (Open note / Note…).
## Mutants (named in the commit message) are driven by `--tag`; a mutated source makes a named check red.
## Exit 0 = held, 1 = a check failed, 2 = could not run.

var app: Node
var _vp: SubViewport
var _bridge
var _checks := 0
var _fails := 0
var _out := ""
var _tag := "base"
var _root := ""
var _phone := false


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("NOTEFJ %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("NOTEFJ %s" % s)


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _finish() -> void:
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join("%s_%s" % [_tag, name]))
	_p("shot %s_%s" % [_tag, name])


## Presents the card for `hits` the way the overlay's gesture would (a request the broker builds and merges).
func _card_for(hits: Array) -> void:
	app.context_broker.resolve({"hits": hits, "screen_pos": Vector2(420, 300), "source": "touch" if _phone else "click",
		"all_hits": hits})
	await _frames(6)


func _note_rows() -> Array:
	var out: Array = []
	for r in app.context_broker.last_actions:
		var d: Dictionary = r
		var id := String(d.get("id", ""))
		if String(d.get("section", "")) == "note" or id.ends_with("_note") or id.begins_with("note."):
			out.append(d)
	return out


## Every action label on the card that reads like the retired rows.
func _retired_labels() -> Array:
	var out: Array = []
	for r in app.context_broker.last_actions:
		var l := String((r as Dictionary).get("label", ""))
		if l.begins_with("Create note") or l.begins_with("Attach note") or l.begins_with("Open note for") or l.begins_with("Open note ·"):
			out.append(l)
	return out


## The drawn card's (or phone sheet's) note rows: desktop/tablet read `drawn_rows`, the phone reads the sheet's labels.
func _drawn_note_labels() -> Array:
	var out: Array = []
	if _phone:
		var pm = app._phone_menu
		for l in pm.peek_full_row_labels():
			if String(l) in ["Note…", "Open note"] or String(l).begins_with("Attach note") or String(l).begins_with("Create note"):
				out.append(String(l))
	else:
		var c = app.context_broker.card
		if c != null and c.visible:
			for r in c.drawn_rows():
				var id := String((r as Dictionary).get("id", ""))
				if id.begins_with("note.") or id.ends_with("_note"):
					out.append(String((r as Dictionary).get("text", "")))
	return out


func _close_card() -> void:
	var c = app.context_broker.card
	if c != null and c.visible:
		c.hide()
	if _phone:
		var pm = app.get("_phone_menu")
		if pm != null and pm.peek_card_is_open():
			await pm._dismiss_peek_card()
		app.phone_hide_right_sheet()
		app._close_all_phone_overlays()
	await _frames(3)


func _vw():
	return app.vault_window


func _press_named(node_name: String) -> bool:
	var n: Node = _vw().find_child(node_name, true, false)
	if n == null or not (n is Button):
		return false
	(n as Button).pressed.emit()
	return true


func _button_starting(prefix: String) -> Button:
	return _find_button(_vw(), prefix)


func _all_buttons(root: Node) -> Array:
	var out: Array = []
	for c in root.find_children("*", "Button", true, false):
		out.append("%s%s" % [(c as Button).text, "" if (c as Button).is_visible_in_tree() else "(hidden)"])
	return out


func _find_button(root: Node, prefix: String) -> Button:
	for c in root.get_children():
		## The phone draws button text in capitals, so match without case.
		if c is Button and String((c as Button).text).to_lower().begins_with(prefix.to_lower()) and (c as Button).is_visible_in_tree():
			return c
		var r := _find_button(c, prefix)
		if r != null:
			return r
	return null


## One kind, through R, O, C, L, A. `vk` is the vault's kind word, `vid` the entity id the vault binds.
func _leg(label: String, hits: Array, vk: String, vid: int, nm: String, shots: bool) -> void:
	_p("== %s ==" % label)
	await _card_for(hits)
	var rows := _note_rows()
	_ok("%s R1: exactly one note row, with no vault note yet" % label, rows.size() == 1 and String(rows[0].get("id", "")) == "note.open"
		and String(rows[0].get("label", "")) == "Note…", str(rows.map(func(r): return r.get("label"))))
	_ok("%s R2: none of the retired rows" % label, _retired_labels().is_empty(), str(_retired_labels()))
	var drawn := _drawn_note_labels()
	_ok("%s R3: the %s shows that one row" % [label, "peek sheet" if _phone else "card"], drawn.size() == 1 and drawn[0].begins_with("Note…"), str(drawn))
	if shots:
		await _shot("card_none_%s.png" % label)
	if rows.is_empty():
		return
	(rows[0]["callable"] as Callable).call()
	await _frames(8)
	var vw = _vw()
	_ok("%s O1: the row opens the vault window" % label, vw.visible)
	_ok("%s O2: scoped to this place (kind, id, name)" % label, String(vw._kind) == vk and int(vw._entity_id) == vid and String(vw._entity_label) == nm,
		"%s %s %s" % [vw._kind, vw._entity_id, vw._entity_label])
	_ok("%s O3: the place's notes panel is on screen" % label, vw.find_child("VaultPlaceNotes", true, false) != null)
	if shots:
		await _shot("vault_empty_%s.png" % label)
	# C: create through the vault's own form and folder chooser.
	var cb := _button_starting("Create ")
	_ok("%s C0: the create form's button is there" % label, cb != null, "" if cb != null else str(_all_buttons(vw)))
	if cb != null:
		cb.pressed.emit()
		await _frames(8)
	var links: Array = _bridge.vault_links_for(vk, vid)
	_ok("%s C1: creating binds one note to the place" % label, links.size() == 1, str(links))
	var made := String((links[0] as Dictionary).get("path", "")) if links.size() == 1 else ""
	_ok("%s C2: the file exists in the vault" % label, made != "" and FileAccess.file_exists(_root + "/" + made), made)
	_ok("%s C3: the panel lists it" % label, _listed(vw) == [made.get_file().to_lower()], str(_listed(vw)))
	if shots:
		await _shot("vault_linked_%s.png" % label)
	await _close_card()
	await _card_for(hits)
	rows = _note_rows()
	_ok("%s C4: the card's one row now reads Open note" % label, rows.size() == 1 and String(rows[0].get("label", "")) == "Open note"
		and _drawn_note_labels().size() == 1 and _drawn_note_labels()[0].begins_with("Open note"), str(rows.map(func(r): return r.get("label"))))
	_ok("%s C5: still none of the retired rows" % label, _retired_labels().is_empty(), str(_retired_labels()))
	if shots:
		await _shot("card_note_%s.png" % label)
	await _close_card()
	# L: unlink removes the link only.
	_ok("%s L1: Unlink is pressed" % label, _press_named("PlaceNoteUnlink"))
	await _frames(6)
	_ok("%s L2: the link is gone and the file stays" % label, _bridge.vault_links_for(vk, vid).is_empty() and FileAccess.file_exists(_root + "/" + made))
	_ok("%s L3: the panel lists no note" % label, _listed(vw).is_empty(), str(_listed(vw)))
	await _card_for(hits)
	rows = _note_rows()
	_ok("%s L4: the card's row is back to Note…" % label, rows.size() == 1 and String(rows[0].get("label", "")) == "Note…")
	await _close_card()
	# A: attach an existing note: pick it in the tree, then Attach to <place>.
	_ok("%s A0: Attach an existing note… is offered" % label, _vw().find_child("PlaceAttachExisting", true, false) != null)
	_press_named("PlaceAttachExisting")
	await _frames(6)
	vw._select_file("Lore/Old.md")
	await _frames(8)
	var ab := _button_starting("Attach to %s" % nm)
	_ok("%s A1: picking a note offers Attach to the place" % label, ab != null)
	if ab != null:
		ab.pressed.emit()
		await _frames(8)
	links = _bridge.vault_links_for(vk, vid)
	_ok("%s A2: attaching links Lore/Old.md" % label, links.size() == 1 and String((links[0] as Dictionary).get("path", "")) == "Lore/Old.md", str(links))
	_ok("%s A3: the panel lists it" % label, _listed(vw) == ["old.md"], str(_listed(vw)))
	await _card_for(hits)
	rows = _note_rows()
	_ok("%s A4: the card reads Open note" % label, rows.size() == 1 and String(rows[0].get("label", "")) == "Open note")
	await _close_card()
	# leave the place unlinked for the next leg's R1.
	_press_named("PlaceNoteUnlink")
	await _frames(4)
	vw.hide()
	await _frames(4)


func _listed(vw) -> Array:
	var out: Array = []
	var sec: Node = vw.find_child("VaultPlaceNotes", true, false)
	if sec == null:
		return out
	for n in sec.find_children("PlaceNoteOpen", "Button", true, false):
		out.append(String((n as Button).text).to_lower())   ## the phone draws capitals
	return out


func _run() -> void:
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-notefj-%d" % Time.get_ticks_msec()
	_write(_root + "/Lore/Old.md", "# Old\n")
	_write(_root + "/Templates/Place.md", "# {{title}}\n")
	var settlements: Array = _bridge.settlements()
	var si := 0
	var s: Dictionary = settlements[si]
	var tid := int(s.get("tid", si))
	var sname := String(s.get("name", ""))
	# no vault yet: the one row is still one row, and still enabled (the vault window is where a folder is connected).
	await _card_for([{"kind": "settlement", "id": si, "dist": 0.0, "label": sname}])
	var nv := _note_rows()
	_ok("N0: with no vault folder the card still has exactly one note row, enabled", nv.size() == 1 and bool(nv[0].get("enabled", false))
		and String(nv[0].get("label", "")) == "Note…", str(nv))
	await _close_card()
	_ok("N1: the throw-away vault binds", bool(_bridge.vault_connect(_root, "Chronicle").get("ok", false)))
	await _frames(4)
	await _leg("settlement", [{"kind": "settlement", "id": si, "dist": 0.0, "label": sname}], "settlement", tid, sname, true)
	# landmark
	var lms: Array = _bridge.landmarks()
	if not lms.is_empty():
		var lm: Dictionary = lms[0]
		var lname := String((_bridge.landmark_names()[0] as Dictionary).get("name", ""))
		var got: Dictionary = _bridge.vault_landmark_entity_id("%s@%d,%d" % [String(lm.get("kind", "")), int(lm.get("x", 0)), int(lm.get("y", 0))])
		await _leg("landmark", [{"kind": "landmark", "id": 0, "dist": 0.0, "label": lname}], "landmark", int(got.get("entity_id", -1)), lname, false)
	else:
		_ok("landmark: the world has landmarks", false)
	# point of interest
	var pi: int = _bridge.poi_add(float(s["x"]) + 8.5, float(s["y"]) + 8.5, "shop", "Fj Shop")
	_ok("a point of interest placed", pi >= 0)
	var peid: Dictionary = _bridge.vault_poi_entity_id(pi)
	await _leg("poi", [{"kind": "poi", "id": pi, "dist": 0.0, "label": "Fj Shop"}], "poi", int(peid.get("entity_id", -1)), String(peid.get("label", "Fj Shop")), false)
	# label
	var li: int = _bridge.label_create(float(s["x"]) + 14.5, float(s["y"]) + 14.5, "Fj label")
	_ok("a label placed", li >= 0)
	var leid: Dictionary = _bridge.vault_label_entity_id(li)
	await _leg("label", [{"kind": "label", "id": li, "dist": 0.0, "label": "Fj label"}], "label", int(leid.get("entity_id", -1)), String(leid.get("label", "Fj label")), false)
	# icon (needs an asset pack, as `_edfc_probe` loads)
	var pack: String = ProjectSettings.globalize_path("res://") + "../crates/cartalith-assets/tests/fixtures/reference_pack.zip"
	if _bridge.world_gen.load_asset_pack(pack):
		var ii := -1
		## Land is not guaranteed at a fixed offset: try the settlement's own cell neighbourhood until the engine accepts one.
		for off in [Vector2(0.5, 0.5), Vector2(2.5, 0.5), Vector2(-2.5, 0.5), Vector2(0.5, 3.5), Vector2(5.5, 5.5), Vector2(-5.5, 5.5)]:
			_bridge.icon_arm("feature", 0, 1.0, 0.0, 0.0)
			ii = _bridge.icon_place(float(s["x"]) + off.x, float(s["y"]) + off.y)
			_bridge.icon_disarm()
			if ii >= 0:
				break
		_ok("an icon placed", ii >= 0)
		var ieid: Dictionary = _bridge.vault_icon_entity_id(ii)
		await _leg("icon", [{"kind": "icon", "id": ii, "dist": 0.0, "label": "icon"}], "icon", int(ieid.get("entity_id", -1)), String(ieid.get("label", "icon")), false)
	else:
		_ok("the fixture asset pack loaded (icons)", false)
	# an empty map point: no hit, no note row.
	await _card_for([])
	_ok("E1: an empty map point offers no note row", _note_rows().is_empty())
	await _close_card()


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
		elif a == "--tag":
			_tag = String(args[i + 1])
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
	_phone = DccTheme.is_phone()
	_p("form phone=%s tablet=%s vp=%s" % [_phone, DccTheme.is_tablet(), str(vp_size)])
	_bridge = app.bridge
	_bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if not _bridge.has_world:
		_ok("a world generated", false)
		_finish()
		return
	await _bridge.landmark_run()
	await _frames(6)
	var t0 := Time.get_ticks_msec()
	while app.busy_overlay != null and app.busy_overlay.visible and Time.get_ticks_msec() - t0 < 30000:
		await _frames(10)
	await _frames(20)
	if _phone:
		app.element_chips.hide_chips()
		app._close_all_phone_overlays()
		await _frames(10)
	await _run()
	_finish()

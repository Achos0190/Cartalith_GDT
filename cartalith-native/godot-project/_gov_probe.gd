extends Node
## Protects: G1-3, the faction hub's grouped Government picker
## (`GOVERNMENTS_SCOPE.md` §6 and §8 G1-3, Ruling CZ Q9), read off the live
## window on a generated world:
##
##   VOCAB   `civ_government_vocabulary()` holds the 17 forms in stored order,
##           nine marked `reference`, five display groups, Unclaimed alone
##           ungrouped, every appended form naming its anchor row
##   PICKER  the Identity tab's "GovernmentPicker" lists every form once plus
##           one disabled header per group (22 rows), headers not choosable
##   PICK    every one of the 17 forms is choosable through the picker and
##           persists: `get_factions()` reads the key back (17 of 17), and the
##           "GovernmentHint" line under the picker follows the pick
##   DOCK    the right dock's Faction card shows the vocabulary label
##   MIL     a faction set to an added form gets the Military tab's
##           authored-ordering line naming its anchor; a reference form does not;
##           the khanate states its levy bonus (G1-5)
##   UNKNOWN a stored key this build does not know ("future_form") shows its
##           own name, is not choosable, and 0 rows read "None / Unclaimed"
##           as the selection; the dock names it the same way
##   PHONE   (phone only) the picker is at least 44 px on BOTH axes, the popup's
##           row pitch is at least 44 px, the Identity pane's combined minimum
##           width fits the screen, and no container scrolls horizontally
##   TABLET  (tablet only) the picker is at least `role_px("btn_min_h")` tall
##
## Windowed, never `--headless` (layout is the subject, `MISTAKES.md`):
##   Godot_v4.7.1-stable_win64_console.exe --path . _gov_probe.tscn -- --out <dir>
##   ... -- --out <dir> --force-touch --vp 2560x1600     (tablet)
##   ... -- --out <dir> --force-touch --vp 1080x2340     (phone)
## `--out <dir>` receives PNG captures of the live viewport (Identity pane,
## picker open, Military tab). Prints `GOV ok|FAIL <name>`, `PROBE-FAIL <name>`
## per miss, and `GOV RESULT PASS|FAIL checks=<n> fails=<n>`; exits 0 only with
## no failure, 1 otherwise, 2 for an unknown argument or a world it cannot
## use, 3 from the watchdog.

var _app: Node
var _bridge
var _fr
var _vp: SubViewport = null
var _out := ""
var _checks := 0
var _fails := 0
var _phone := false
var _tablet := false


func _ok(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if cond:
		print("GOV ok    %s" % name)
	else:
		_fails += 1
		print("GOV FAIL  %s  -- %s" % [name, detail])
		print("PROBE-FAIL %s" % name)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _shot(tag: String) -> void:
	if _out == "":
		return
	await _frames(4)
	var form := "phone" if _phone else ("tablet" if _tablet else "desktop")
	var tex := _vp.get_texture() if _vp != null else get_viewport().get_texture()
	tex.get_image().save_png(_out.path_join("gov_%s_%s.png" % [form, tag]))


func _count(node: Node, pred: Callable) -> int:
	var n := 1 if pred.call(node) else 0
	for c in node.get_children(true):
		n += _count(c, pred)
	return n


func _texts(node: Node, out: PackedStringArray) -> void:
	if node is Label:
		out.append((node as Label).text)
	elif node is Button:
		out.append((node as Button).text)
	for c in node.get_children():
		_texts(c, out)


func _faction(f: int) -> Dictionary:
	for e in _bridge.get_factions():
		if int(e.get("id", -1)) == f:
			return e
	return {}


func _picker() -> OptionButton:
	return _fr.find_child("GovernmentPicker", true, false) as OptionButton


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("GOV WATCHDOG"); get_tree().quit(3))
	wd.start()
	var vp_size := Vector2i.ZERO
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--vp" and i + 1 < args.size():
			var wh := args[i + 1].split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
			continue
		if a == "--out" and i + 1 < args.size():
			_out = args[i + 1]
			DirAccess.make_dir_recursive_absolute(_out)
			i += 2
			continue
		if a == "--force-touch":
			i += 1
			continue
		print("GOV ABORT: unknown argument '%s'" % a)
		get_tree().quit(2)
		return
	if vp_size != Vector2i.ZERO:
		_vp = SubViewport.new()
		_vp.size = vp_size
		_vp.transparent_bg = false
		_vp.gui_embed_subwindows = true
		_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(_vp)
		_app = load("res://shell/app.tscn").instantiate()
		_vp.add_child(_app)
	else:
		_app = load("res://shell/app.tscn").instantiate()
		add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)
	_phone = DccTheme.is_phone()
	_tablet = DccTheme.is_tablet()
	print("GOV form=%s" % ("phone" if _phone else ("tablet" if _tablet else "desktop")))
	_fr = _app.faction_roster_window
	if _bridge.get_factions().size() < 3:
		print("GOV ABORT: fewer than two real factions")
		get_tree().quit(2)
		return
	await _run()
	print("GOV RESULT %s  checks=%d fails=%d" % ["PASS" if _fails == 0 else "FAIL", _checks, _fails])
	get_tree().quit(0 if _fails == 0 else 1)


func _run() -> void:
	# -- VOCAB -------------------------------------------------------------------
	var vocab: Array = _bridge.civ_government_vocabulary()
	_ok("VOCAB 17 forms", vocab.size() == 17, str(vocab.size()))
	var refs := 0
	var groups := {}
	var ungrouped: Array = []
	var anchors_ok := true
	var labels := {}
	for e in vocab:
		var v: Dictionary = e
		labels[String(v.get("key"))] = String(v.get("label"))
		if bool(v.get("reference", false)):
			refs += 1
			if v.has("anchor"):
				anchors_ok = false
		elif not v.has("anchor"):
			anchors_ok = false
		if v.has("group"):
			groups[String(v.get("group"))] = true
		else:
			ungrouped.append(String(v.get("key")))
		if String(v.get("hint", "")) == "":
			anchors_ok = false
	_ok("VOCAB nine reference forms", refs == 9, str(refs))
	_ok("VOCAB five groups, Unclaimed alone ungrouped", groups.size() == 5 and ungrouped == ["none"],
		"%s / %s" % [groups.keys(), ungrouped])
	_ok("VOCAB appended forms (only) name an anchor; every hint non-empty", anchors_ok)

	# -- PICKER ------------------------------------------------------------------
	var f := 1
	_app.open_faction_roster(f, "identity")
	await _frames(10)
	var ob := _picker()
	_ok("PICKER found on the Identity tab", ob != null and ob.is_visible_in_tree())
	if ob == null:
		return
	await _shot("identity")
	var headers := 0
	var rows := {}
	for k in ob.item_count:
		if ob.is_item_disabled(k):
			headers += 1
		else:
			rows[ob.get_item_text(k)] = k
	_ok("PICKER 22 rows: 17 forms and 5 disabled headers", ob.item_count == 22 and headers == 5,
		"items=%d disabled=%d" % [ob.item_count, headers])
	var all_listed := true
	for key in labels:
		if not rows.has(labels[key]):
			all_listed = false
	_ok("PICKER every form's label is a choosable row", all_listed and rows.size() == 17, str(rows.size()))

	# -- PICK --------------------------------------------------------------------
	var persisted := 0
	var hinted := 0
	var miss := ""
	for e in vocab:
		var v: Dictionary = e
		var key := String(v.get("key"))
		ob = _picker()
		var idx: int = -1
		for k in ob.item_count:
			if ob.get_item_text(k) == String(v.get("label")) and not ob.is_item_disabled(k):
				idx = k
		if idx < 0:
			miss += key + " "
			continue
		ob.select(idx)
		ob.item_selected.emit(idx)
		await _frames(3)
		if String(_faction(f).get("government", "")) == key:
			persisted += 1
		else:
			miss += "%s(read %s) " % [key, _faction(f).get("government", "?")]
		var hl := _fr.find_child("GovernmentHint", true, false) as Label
		if hl != null and hl.text == String(v.get("hint")):
			hinted += 1
	_ok("PICK 17 of 17 forms persist through get_factions()", persisted == 17, "%d; %s" % [persisted, miss])
	_ok("PICK the hint line follows every pick", hinted == 17, str(hinted))

	# -- MIL and DOCK: an added form, then a reference form ------------------------
	_bridge.civ_set_faction_field(f, "government", "merchant_republic")
	_app.open_faction_roster(f, "military")
	await _frames(12)
	var mil := PackedStringArray()
	_texts(_fr, mil)
	var mil_text := "\n".join(mil)
	_ok("MIL an added form names its authored ordering and anchor",
		mil_text.contains("Merchant Republic is one of the added forms") and mil_text.contains("the share a Republic confers"),
		mil_text.substr(0, 200))
	await _shot("military_merchant_republic")
	_bridge.civ_set_faction_field(f, "government", "nomad_khanate")
	_app.open_faction_roster(f, "military")
	await _frames(12)
	mil = PackedStringArray()
	_texts(_fr, mil)
	_ok("MIL the khanate states its levy bonus (G1-5)",
		"\n".join(mil).contains("Its levy reaches further than its extraction alone gives"))
	_bridge.civ_set_faction_field(f, "government", "monarchy")
	_app.open_faction_roster(f, "military")
	await _frames(12)
	mil = PackedStringArray()
	_texts(_fr, mil)
	_ok("MIL a reference form gets no added-form line", not "\n".join(mil).contains("is one of the added forms"))
	var dock = _app.get("right_dock_ctrl")
	if dock != null and dock.has_method("_government_label"):
		_ok("DOCK label for city_state is the vocabulary's", dock._government_label({"government": "city_state"}) == "City-State",
			dock._government_label({"government": "city_state"}))
		_ok("DOCK label for an added form", dock._government_label({"government": "holy_order"}) == "Holy Order")
		var u: String = dock._government_label({"government": "future_form"})
		_ok("UNKNOWN the dock names an unknown key", u.contains("future_form") and not u.contains("Unclaimed"), u)
		_ok("UNKNOWN the dock dashes an absent key", dock._government_label({}).begins_with("—"))
	else:
		_ok("DOCK right dock reachable", false, "no right_dock with _government_label")

	# -- UNKNOWN: the picker over a stored key this build does not know ----------
	var box := VBoxContainer.new()
	_fr.add_child(box)
	_fr._government_choice(box, {"government": "future_form"})
	await _frames(3)
	var uob: OptionButton = null
	for c in box.find_children("*", "OptionButton", true, false):
		uob = c
	_ok("UNKNOWN a picker is drawn for an unknown key", uob != null)
	if uob != null:
		var shown := uob.get_item_text(uob.selected) if uob.selected >= 0 else ""
		_ok("UNKNOWN the selection names the key", shown.contains("future_form"), shown)
		_ok("UNKNOWN 0 rows show None / Unclaimed as the selection", not shown.contains("Unclaimed"), shown)
		_ok("UNKNOWN the unknown entry cannot be picked", uob.is_item_disabled(uob.selected))
	box.queue_free()
	var box2 := VBoxContainer.new()
	_fr.add_child(box2)
	_fr._government_choice(box2, {})
	await _frames(2)
	for c in box2.find_children("*", "OptionButton", true, false):
		var o2: OptionButton = c
		_ok("UNKNOWN an absent key is dashed, not shown as a form", o2.get_item_text(o2.selected).begins_with("—"), o2.get_item_text(o2.selected))
	box2.queue_free()

	# -- Back to Identity with the popup open, for the eye and the phone legs -----
	_app.open_faction_roster(f, "identity")
	await _frames(10)
	ob = _picker()
	if _phone or _tablet:
		var floor_px := float(DccTheme.PHONE_TAP_MIN) if _phone else float(DccTheme.role_px("btn_min_h"))
		_ok("TOUCH the picker is at least the touch floor on both axes (%.0f)" % floor_px,
			ob.size.y + 0.5 >= floor_px and ob.size.x + 0.5 >= floor_px, str(ob.size))
	if _phone:
		var pane: Control = _fr._tab_panes.get("identity", null)
		var screen_w: float = float(_fr.size.x)
		var minw: float = pane.get_combined_minimum_size().x if pane != null else 1e9
		_ok("PHONE the Identity pane's minimum width fits the window", minw <= screen_w + 0.5,
			"min %.0f vs %.0f" % [minw, screen_w])
		var hs := _count(_fr, func(n): return n is ScrollContainer and n.is_visible_in_tree() \
			and (n as ScrollContainer).horizontal_scroll_mode != ScrollContainer.SCROLL_MODE_DISABLED \
			and (n as ScrollContainer).get_h_scroll_bar().visible)
		_ok("PHONE no horizontal scrollbar", hs == 0, str(hs))
	ob.show_popup()
	await _frames(6)
	var pop := ob.get_popup()
	var fsz := pop.get_theme_font_size("font_size")
	var line := pop.get_theme_font("font").get_height(fsz)
	var pitch := line + float(pop.get_theme_constant("v_separation"))
	print("GOV popup row pitch %.1f (line %.1f + v_separation %d), %d items, popup %s" % [
		pitch, line, pop.get_theme_constant("v_separation"), pop.item_count, pop.size])
	if _phone:
		_ok("PHONE the popup's row pitch is at least 44 px", pitch + 0.5 >= float(DccTheme.PHONE_TAP_MIN), "%.1f" % pitch)
	await _shot("picker_open")
	pop.hide()
	await _frames(2)

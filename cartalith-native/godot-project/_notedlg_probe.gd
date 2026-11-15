extends Node
## The card's Create note dialog (`ContextNote.ask_create`, with `VaultFolderPick`'s tree) laid out on every form and in a
## City Viewer pop-out. Owner report: in the pop-out the dialog was 380x100, the name field not visible, the tree squeezed
## to ~50 px and Create/Cancel drawn over the tree's last row.
##
##   godot --path . _notedlg_probe.tscn -- --form desktop|tablet|phone --vp 1600x900 [--force-touch --density 1.0]
##                                          [--small] [--shots <dir> --tag <name>]
##   godot --path . _notedlg_probe.tscn -- --popout --shots <dir> --tag <name>        (WINDOWED, opens a second OS window)
##
## Every check reads laid-out `get_global_rect()` after settled frames, never a requested size. Exit 0 held, 1 failed, 2 abort.
## What it protects (each leg, on the form named):
##   V  the name field is visible in the tree, focused, inside the dialog and has a real height;
##   T  the folder tree is at least `MIN_ROWS` rows tall (a row = the tree's own first-row height, 44 on touch) -- or, with
##      `--small` (a host too short for that), at least 2 rows and the dialog fills the host's room instead;
##   B  Create and Cancel are inside the dialog and do not intersect the tree (nor the name field);
##   F  the dialog lies wholly inside its host window (the pop-out, or the app's viewport).

const VAULT_FOLDERS := ["Lore", "Lore/Settlements", "Lore/Factions", "Lore/People", "People", "Places", "Places/North", "Places/South", "Sessions", "Maps", "Maps/Old"]
const MIN_ROWS := 6

var _fails := 0
var _checks := 0
var _shots := ""
var _tag := "base"
var _form := "desktop"
var _vp_size := Vector2i(1600, 900)
var _popout := false
var _small := false
var _app: Node
var _bridge
var _reached_end := false


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(1 if _fails > 0 else 0)


func _shot(w: Viewport, name: String) -> void:
	if _shots == "":
		return
	await RenderingServer.frame_post_draw
	w.get_texture().get_image().save_png(_shots.path_join("notedlg_%s_%s.png" % [_tag, name]))


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--shots" and i + 1 < args.size():
			_shots = String(args[i + 1]); i += 2
		elif a == "--tag" and i + 1 < args.size():
			_tag = String(args[i + 1]); i += 2
		elif a == "--form" and i + 1 < args.size():
			_form = String(args[i + 1]); i += 2
		elif a == "--vp" and i + 1 < args.size():
			var wh := String(args[i + 1]).split("x")
			_vp_size = Vector2i(int(wh[0]), int(wh[1])); i += 2
		elif a == "--density" and i + 1 < args.size():
			i += 2
		elif a == "--popout":
			_popout = true; i += 1
		elif a == "--small":
			_small = true; i += 1
		elif a == "--force-touch":
			i += 1
		else:
			print("ABORT: unknown argument %s" % a)
			get_tree().quit(2)
			return
	if _popout and DisplayServer.get_name() == "headless":
		print("ABORT: --popout needs a windowed run")
		get_tree().quit(2)
		return
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	var host_parent: Node = self
	var vp: SubViewport = null
	if not _popout:
		vp = SubViewport.new()
		vp.size = _vp_size
		vp.gui_embed_subwindows = true
		vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(vp)
		host_parent = vp
	_app = load("res://shell/app.tscn").instantiate()
	host_parent.add_child(_app)
	await get_tree().create_timer(1.4).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	var phone: bool = _app.is_phone()
	var tablet: bool = DccTheme.is_tablet()
	_ok((_form == "phone") == phone and (_form == "tablet") == tablet, "the shell booted into the %s form (phone %s, tablet %s)" % [_form, str(phone), str(tablet)])
	DccSettings.set_popout_enabled(false)
	PopoutHost.forget_choices(get_tree())
	_bridge = _app.bridge
	_bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 512, "grid_h": 384, "archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _settle(20)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	if not _bridge.has_world:
		_ok(false, "a world generated")
		_finish()
		return
	await _run(vp)
	_ok(_reached_end, "the probe ran to its last check")
	_finish()


func _town() -> int:
	var places: Array = _bridge.settlements()
	for k in places.size():
		if String((places[k] as Dictionary).get("kind", "")) in ["city", "capital", "town", "metropolis"]:
			var lay: Array = _bridge.urban_layouts(PackedInt32Array([k]))
			if lay.size() > 0 and (lay[0] as Dictionary).has("market"):
				return k
	return -1


func _vault() -> void:
	var root := OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-notedlg-%d" % Time.get_ticks_msec()
	for f in VAULT_FOLDERS:
		DirAccess.make_dir_recursive_absolute(root + "/" + f)
		var fa := FileAccess.open(root + "/" + f + "/Old.md", FileAccess.WRITE)
		fa.store_string("# Old\n")
		fa.close()
	_ok(bool(_bridge.vault_connect(root, "Chronicle").get("ok", false)), "a throw-away vault binds (%d folders)" % VAULT_FOLDERS.size())


func _mb(local: Vector2, down: bool) -> InputEventMouseButton:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = down
	ev.position = local
	ev.global_position = local
	ev.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
	return ev


func _click_in(w: Window, at: Vector2) -> void:
	for down in [true, false]:
		var ev := _mb(at, down)
		w.push_input(ev)
		await _settle(2)
	await _settle(8)


## The dialog's rect in its host's coordinates. An embedded dialog's `position`/`size` are in the host's logical pixels.
func _dlg_rect(dlg: Window) -> Rect2:
	return Rect2(Vector2(dlg.position), Vector2(dlg.size))


## Lays out the four legs against the open dialog `d`, whose host's visible rect is `host`.
func _check(d: Dictionary, host: Rect2, label: String) -> void:
	var dlg: Window = d["dialog"]
	await _settle(8)
	_ok(dlg.visible, "%s: the dialog is showing" % label)
	var local := Rect2(Vector2.ZERO, Vector2(dlg.size))
	var edit: LineEdit = d["field"]
	var tree: Tree = d["chooser"]["tree"]
	var ok_b: Button = (dlg as ConfirmationDialog).get_ok_button()
	var cancel_b: Button = (dlg as ConfirmationDialog).get_cancel_button()
	var er := edit.get_global_rect()
	var tr := tree.get_global_rect()
	var okr := ok_b.get_global_rect()
	var cr := cancel_b.get_global_rect()
	print("  [%s] diag size %s min %s max %s contents_min %s wrap %s body_min %s visible %s unresizable %s" % [label, str(dlg.size), str(dlg.min_size), str(dlg.max_size), str(dlg.get_contents_minimum_size()), str(dlg.wrap_controls), str(d["body"].get_combined_minimum_size()), str(dlg.visible), str(dlg.unresizable)])
	print("  [%s] dialog %s host %s | name %s tree %s ok %s cancel %s" % [label, str(_dlg_rect(dlg)), str(host), str(er), str(tr), str(okr), str(cr)])
	# V
	_ok(edit.is_visible_in_tree() and er.size.y >= 20.0 and local.encloses(er), "%s V: the name field is visible, has a height and lies inside the dialog" % label)
	var fo = dlg.gui_get_focus_owner() if dlg.has_method("gui_get_focus_owner") else null
	print("  focus owner: %s / visible %s / editable %s / focus_mode %d" % [str(fo), str(edit.is_visible_in_tree()), str(edit.editable), edit.focus_mode])
	if DccTheme.is_phone():
		# The phone's own convention (`DccWidgets.touch_focus_field`, `PgField`): a field in a scrolling sheet is parked at
		# FOCUS_NONE until tapped, so a dialog does not raise the soft keyboard over its own folder tree on open.
		_ok(not edit.has_focus() and edit.get_script() != null, "%s V: on the phone the field is a tap-to-focus PgField, not auto-focused (the keyboard stays down)" % label)
	else:
		_ok(edit.has_focus(), "%s V: the name field is focused" % label)
	# T
	var row_h := 44.0 if DccTheme.is_touch() else float(tree.get_item_area_rect(tree.get_root()).size.y)
	row_h = maxf(row_h, 18.0)
	var want := 2.0 * row_h if _small else float(MIN_ROWS) * row_h
	_ok(tr.size.y >= want, "%s T: the tree is %d px tall, at least %d (%s rows of %d px)" % [label, int(tr.size.y), int(want), "2" if _small else str(MIN_ROWS), int(row_h)])
	_ok(local.encloses(tr), "%s T: the tree lies inside the dialog" % label)
	# B
	_ok(local.encloses(okr) and local.encloses(cr), "%s B: Create and Cancel lie inside the dialog" % label)
	_ok(not okr.intersects(tr) and not cr.intersects(tr), "%s B: and do not touch the tree (ok %s, cancel %s, tree %s)" % [label, str(okr), str(cr), str(tr)])
	_ok(not er.intersects(tr), "%s B: nor does the name field overlap the tree" % label)
	# F
	## An embedded dialog's title bar is drawn ABOVE its client rect, so the whole visible window is the rect grown upward.
	var bar := 0.0 if dlg.borderless else float(dlg.get_theme_constant("title_height", "Window"))
	var whole := Rect2(Vector2(dlg.position) - Vector2(0.0, bar), Vector2(dlg.size) + Vector2(0.0, bar))
	## A floating dialog keeps a little air from the host's edge (`PopoutHost.DIALOG_MARGIN` is 16; 4 is asserted so a
	## centred dialog's rounding cannot fail it). The phone's sheet is full-screen by design and keeps none.
	var air := 0.0 if DccTheme.is_phone() else 4.0
	_ok(host.grow(-air).encloses(whole), "%s F: the dialog, title bar included, is wholly inside its host window with %d px of air (%s in %s)" % [label, int(air), str(whole), str(host)])


func _run(vp: SubViewport) -> void:
	var si := _town()
	_ok(si >= 0, "a town with a layout exists")
	if si < 0:
		return
	var s: Dictionary = _bridge.settlements()[si]
	_vault()
	if _app.context_broker._note == null:   # the broker builds it lazily, on the first card
		_app.context_broker._note = preload("res://shell/context_note.gd").new(_app)
	var note = _app.context_broker._note
	var r := {"name": String(s.get("name", "Testtown")), "vk": "settlement", "vid": si}
	if not _popout:
		var host := Rect2(Vector2.ZERO, vp.get_visible_rect().size)
		note.ask_create(r)
		await _check(note._last_dialog, host, _form)
		await _shot(vp, "%s_create" % _form)
		((note._last_dialog["dialog"]) as ConfirmationDialog).canceled.emit()
		await _settle(3)
		_reached_end = true
		return
	# pop-out: the viewer popped out, a card open inside it, then Create note
	var cv: CityViewerWindow = _app.city_viewer_window
	_app.arm_tool("inspect")
	_app.open_city_viewer(si)
	await _settle(14)
	await PopoutHost.toggle(cv)
	await _settle(10)
	_ok(not cv.is_embedded() and cv.visible, "the viewer popped out")
	var vis: Rect2 = cv.get_visible_rect()
	var k := -1
	var pins: Array = cv._pins()
	for q in pins.size():
		if not (pins[q] as Dictionary).has("poi"):
			k = q
			break
	var pr: float = cv.PIN_HEAD_R * cv._pin_unit()
	const Pin := preload("res://shell/pin_marker.gd")
	var head: Vector2 = cv._canvas.get_global_transform_with_canvas() * Pin.geometry(cv.pin_tip(k), pr, cv.pin_tilt(k))["head"]
	_app.select("settlement", -1)
	await _click_in(cv, head)
	var c = _app.context_broker.card
	_ok(c != null and c.visible and c.get_parent() == cv, "a card is open inside the pop-out")
	note.ask_create(r)
	var dlg: Window = note._last_dialog["dialog"]
	_ok(dlg.get_parent() == cv, "the dialog opened in the pop-out (parent %s)" % str(dlg.get_parent().name))
	await _check(note._last_dialog, vis, "popout")
	await _shot(cv, "popped_create")
	_reached_end = true

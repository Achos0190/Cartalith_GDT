extends Node
## PH-16 probe -- the Journey Planner on the phone.
##
## `GUI_GAP_REGISTER.md` §22 PH-16 measured, on a OnePlus 6T: "a uniform
## `panel` `#121314` field from y = 265 to y = 1 699 -- 61 % of the screen,
## 91 mm -- in which not one pixel exceeds RGB(23, 23, 23)", with the map gone
## because `journey_planner_view.gd::_show()` sets `app.viewport.visible =
## false`. It closes by naming the two-minute diagnosis it deliberately did not
## run: "`_ph9_probe.gd` at `--vp 1080x2400 --force-touch`, dumping the rects of
## `_center_panel` / `col` / `map_row_pad`". This is that dump, plus the
## assertions the fix has to hold afterwards.
##
## `spec/06-phone.md` §6.5 is the authority for what the phone should do
## instead: PLAN is a SHEET over a live map ("The route line and the selected
## stage are highlighted on the map behind this sheet"), never a centre-panel
## takeover that hides the viewport.
##
## Run:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _ph16_probe.tscn -- --force-touch
##
## SubViewport-sized rather than `--resolution`, for `_phonechrome_probe.gd`'s
## reason: `DccShell._compute_layout_mode()` reads `get_viewport_rect().size`,
## which inside a SubViewport is that viewport's size and is not clamped to the
## dev monitor the way `--resolution` is. 1080x2340 = the OnePlus 6T PH-16 was
## measured on.

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	if not ("--force-touch" in OS.get_cmdline_user_args()):
		print("[FATAL] run with `-- --force-touch`"); get_tree().quit(1); return

	var vp := SubViewport.new()
	vp.size = Vector2i(1080, 2340)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(60)
	print("[BOOT] phone=", app.is_phone(), " scale=", app.phone_scale())
	if not bool(app.is_phone()):
		print("[FATAL] not the phone composition"); get_tree().quit(1); return

	var jp: Node = app.journey_planner_view
	print("\n=== BEFORE: viewport state with PLAN closed ===")
	print("  viewport.visible=", app.viewport.visible)

	print("\n=== tap PLAN ===")
	app.call("_pick_phone_tab", "plan")
	await _frames(40)

	print("\n=== rects ===")
	var cp: Control = jp._center_panel
	print("  _center_panel visible=", (cp.visible if cp != null else "NULL"),
		" rect=", (cp.get_global_rect() if cp != null else Rect2()))
	if cp != null:
		for c in cp.get_children():
			var cc := c as Control
			if cc != null:
				print("    child ", cc.get_class(), " visible=", cc.visible,
					" rect=", cc.get_global_rect())
				for g in cc.get_children():
					var gc := g as Control
					if gc != null:
						print("      ", gc.get_class(), " vis=", gc.visible,
							" rect=", gc.get_global_rect(),
							" min=", gc.get_combined_minimum_size())
	print("  viewport.visible=", app.viewport.visible)
	print("  viewport_content rect=", app.viewport_content.get_global_rect())
	var civ: Control = app._workspace_panels.get("civilization")
	print("  civ panel visible=", (civ.visible if civ != null else "NULL"))
	print("  armed_tool=", app.armed_tool, " domain=", app.active_domain())
	print("  left sheet open=", app.call("_is_sheet_open", "left"),
		"  right sheet open=", app.call("_is_sheet_open", "right"))

	print("\n=== assertions the PH-16 fix has to hold ===")
	## 06-phone.md §6.5: the map is behind the sheet, not replaced by it.
	_ok("the map stays visible with PLAN open", app.viewport.visible, true)
	_ok("the desktop centre-panel takeover does not paint on a phone",
		(cp != null and cp.visible), false)
	_ok("PLAN opens a sheet", app.call("_is_sheet_open", "left"), true)

	print("\n_ph16_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

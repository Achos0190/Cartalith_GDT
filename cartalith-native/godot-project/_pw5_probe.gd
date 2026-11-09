extends Node
## PW-5 of `POPOUT_WINDOWS_SCOPE.md` (Android and tablet gating) and the "never off-screen, never a silent
## screen 0" half of PW-1's restore, against the real shell, the real `PopoutHost`, `PopoutGeometry` and
## the real Preferences menu.
##
##   godot --path . --resolution 1400x900 _pw5_probe.tscn -- --form desktop
##   godot --path . --resolution 1280x800 _pw5_probe.tscn -- --form tablet --force-touch
##   godot --path . --resolution  393x852 _pw5_probe.tscn -- --form phone  --force-touch
##   godot --headless --path . _pw5_probe.tscn -- --pure          (the literal tables only, no window)
##
## **Windowed** (the desktop leg opens a real native window and reads where the OS put it). Run from a scratch
## tree with a wiped user dir (`APPDATA` pointed at an empty directory): section 0 fails if it was not clean.
## Exit 0 pass, 1 fail, 2 bad argument / watchdog.
##
## What it protects, and what it does NOT:
##   tables   `PopoutHost.unsupported_reason()` and `reason_text()` by literal rows: each of the four terms
##            changes some row, and the FORM terms outrank the PLATFORM terms (a tablet on Android says
##            "tablet"). The platform facts (`FEATURE_SUBWINDOWS`, `OS.has_feature("mobile")`) are passed in
##            as literals: **this is the simulation of an Android process** -- this machine is Windows, so no
##            run of this probe has an Android display server. The real OnePlus 6T pass is owner-run.
##   menu     the row, by title, in the real Preferences popup: desktop = an enabled check item; tablet = DISABLED
##            with the literal reason; phone = ABSENT everywhere (every popup under the shell, and the command
##            index).
##   zero     with the stored setting ON (an old or synced setting) every listed window opens embedded in every
##            form that cannot pop out: window count stays 1, no `force_native`, no button, `prepare()` false.
##   geo      (pure, SIMULATED screens) a window stored on a monitor that is gone is clamped onto a screen that
##            exists, labelled by the rule that chose it, never screen 0 by default; and (desktop, REAL) the
##            roster stored on a monitor that does not exist opens inside this machine's one screen.
## One monitor is assumed; the second-monitor halves are owner-run and listed in the scope.

const ROW := "Open panels in their own window"

var _fails := 0
var _checks := 0
var _form := ""
var _app: Node
var _bridge


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame


func _until(cond: Callable, ms: int = 8000) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


func _scr(r: Rect2i, usable: Rect2i = Rect2i()) -> Dictionary:
	return {"rect": r, "usable": r if usable.size == Vector2i.ZERO else usable}


func _geo(x: int, y: int, w: int, h: int, s: Rect2i) -> Dictionary:
	return {"x": x, "y": y, "w": w, "h": h, "sx": s.position.x, "sy": s.position.y, "sw": s.size.x, "sh": s.size.y}


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--form" and i + 1 < args.size():
			_form = String(args[i + 1])
			i += 2
		elif a == "--force-touch":
			i += 1
		elif a == "--pure":
			# the literal tables only (no window, runs headless): the mutation runs on the gate and the geometry use it
			_tables()
			_geo_pure()
			print("RESULT: %s (%d failed, %d checks) pure" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
			get_tree().quit(0 if _fails == 0 else 1)
			return
		else:
			print("ABORT: unknown argument %s" % a)
			get_tree().quit(2)
			return
	if not _form in ["desktop", "tablet", "phone"]:
		print("ABORT: --form desktop|tablet|phone is required")
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		print("ABORT: run windowed")
		get_tree().quit(2)
		return
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	print("== 0. clean user dir; screens on this machine: %d ==" % DisplayServer.get_screen_count())
	_ok(not FileAccess.file_exists(ProjectSettings.globalize_path(DccSettings.CONFIG_PATH)), "no cartalith_settings.cfg at start (the user dir was wiped)")
	_tables()
	_geo_pure()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	var got := "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_ok(got == _form, "the shell booted into the %s form (got %s)" % [_form, got])
	_bridge = _app.bridge
	_menu()
	await _generate()
	await _zero_native()
	if _form == "desktop":
		await _geo_live()
	print("RESULT: %s (%d failed, %d checks) form=%s" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks, _form])
	get_tree().quit(0 if _fails == 0 else 1)


# -- tables ---------------------------------------------------------------------------------------

func _tables() -> void:
	print("== 1. the gate, by literal rows (platform facts SIMULATED: this machine is not Android) ==")
	# [has_subwindows, mobile_os, phone, tablet] -> reason
	var rows := [
		[true, false, false, false, ""],              # a desktop: the one case that pops out
		[true, false, true, false, "phone"],
		[true, false, false, true, "tablet"],
		[false, true, false, true, "tablet"],         # a tablet ON Android: the form term outranks the OS term
		[false, true, true, false, "phone"],          # the 6T: Android AND phone form
		[true, true, true, true, "phone"],            # phone outranks tablet
		[true, true, false, false, "mobile_os"],      # Android in a desktop-shaped form
		[false, true, false, false, "mobile_os"],     # ... and the display server agrees; the OS reason is named first
		[false, false, false, false, "no_subwindows"],
		[false, false, true, false, "phone"],
		[true, true, false, true, "tablet"],
	]
	for r in rows:
		var got := PopoutHost.unsupported_reason(r[0], r[1], r[2], r[3])
		_ok(got == String(r[4]), "unsupported_reason(sub=%s, mobile=%s, phone=%s, tablet=%s) == '%s' (got '%s')" % [r[0], r[1], r[2], r[3], r[4], got])
	_ok(PopoutHost.reason_text("tablet").begins_with("Not available on a tablet"), "the tablet reason names the tablet")
	_ok(PopoutHost.reason_text("mobile_os").contains("Android"), "the OS reason names Android")
	_ok(PopoutHost.reason_text("no_subwindows").contains("display"), "the display reason names the display")
	_ok(PopoutHost.reason_text("phone") == "", "the phone has no row, so no reason text")
	_ok(PopoutHost.reason_text("") == "", "no reason, no text")
	_ok(PopoutHost.reason_text("tablet") != PopoutHost.reason_text("mobile_os") and PopoutHost.reason_text("mobile_os") != PopoutHost.reason_text("no_subwindows"), "the three reasons are three sentences")
	var want := PopoutHost.unsupported_reason(DisplayServer.has_feature(DisplayServer.FEATURE_SUBWINDOWS), OS.has_feature("mobile"), DccTheme.is_phone(), DccTheme.is_tablet()) == ""
	_ok(PopoutHost.supported() == want, "supported() is exactly 'no reason' for this process")


# -- geometry, pure -------------------------------------------------------------------------------

func _geo_pure() -> void:
	print("== 2. a window stored on a monitor that is gone (SIMULATED screens, pure) ==")
	var m1 := Rect2i(0, 0, 1920, 1080)
	var m2 := Rect2i(1920, 0, 2560, 1440)
	var left := Rect2i(-1920, 0, 1920, 1080)
	var u1 := Rect2i(0, 0, 1920, 1040)
	var r := PopoutGeometry.resolve(_geo(2400, 200, 880, 620, m2), [_scr(m1, u1)], 0)
	_ok(bool(r["placed"]) and String(r["how"]) == "primary" and int(r["screen"]) == 0, "monitor 2 unplugged: placed on the primary by the 'primary' rule (%s)" % str(r))
	_ok(bool(r["placed"]) and u1.encloses(r["rect"]) and PopoutGeometry.title_visible_px(r["rect"], [_scr(m1, u1)]) >= 64, "... fully inside monitor 1 with its title row reachable")
	r = PopoutGeometry.resolve(_geo(-1500, 100, 880, 620, left), [_scr(m1, u1)], 0)
	_ok(bool(r["placed"]) and String(r["how"]) == "primary" and u1.encloses(r["rect"]), "a negative-coordinate monitor gone: onto the primary, inside it (%s)" % str(r))
	# the primary index names a screen that is NOT usable: the first usable one is used, and SAID so
	var dead := _scr(m1)
	dead["usable"] = Rect2i(0, 0, 0, 0)
	r = PopoutGeometry.resolve(_geo(9000, 0, 880, 620, Rect2i(8000, 0, 1920, 1080)), [dead, _scr(m2)], 0)
	_ok(bool(r["placed"]) and String(r["how"]) == "first_usable" and int(r["screen"]) == 1, "primary has no usable area: 'first_usable', screen 1, not a silent 0 (%s)" % str(r))
	r = PopoutGeometry.resolve(_geo(9000, 0, 880, 620, Rect2i(8000, 0, 1920, 1080)), [_scr(m1, u1), _scr(m2)], -1)
	_ok(bool(r["placed"]) and String(r["how"]) == "first_usable" and int(r["screen"]) == 0, "no primary given (-1): 'first_usable', never labelled 'primary' (%s)" % str(r))
	r = PopoutGeometry.resolve(_geo(5000, 0, 880, 620, m2), [_scr(m1, u1), _scr(m2)], 1)
	_ok(String(r["how"]) == "same_screen" or String(r["how"]) == "overlap" or String(r["how"]) == "primary", "a stored rect on a screen that exists is matched by a real rule (%s)" % str(r))
	# stored on a screen that is gone, primary index 1: honoured, not replaced by screen 0
	r = PopoutGeometry.resolve(_geo(9000, 0, 880, 620, Rect2i(8000, 0, 1920, 1080)), [_scr(m1, u1), _scr(m2)], 1)
	_ok(String(r["how"]) == "primary" and int(r["screen"]) == 1, "a usable primary index 1 is honoured, not replaced by screen 0 (%s)" % str(r))
	r = PopoutGeometry.resolve(_geo(0, 0, 880, 620, m1), [], 0)
	_ok(not bool(r["placed"]) and String(r["why"]) == "no screen reported", "zero screens: nothing placed, and the reason says so (%s)" % str(r))
	r = PopoutGeometry.resolve({}, [_scr(m1, u1)], 0)
	_ok(not bool(r["placed"]) and not r.has("rect"), "nothing stored: no rectangle at all, not a default one")


# -- the menu ---------------------------------------------------------------------------------------

func _row_in(idx: CommandIndex) -> Dictionary:
	for r in idx.all():
		if String(r["title"]) == ROW:
			return r
	return {}


func _popups_with_row() -> int:
	var n := 0
	for p in _app.find_children("*", "PopupMenu", true, false):
		for k in (p as PopupMenu).item_count:
			if (p as PopupMenu).get_item_text(k) == ROW:
				n += 1
	return n


func _menu() -> void:
	print("== 3. the Preferences row on the %s ==" % _form)
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	var row := _row_in(idx)
	match _form:
		"desktop":
			_ok(not row.is_empty() and bool(row["available"]), "desktop: the row exists and is available")
			if not row.is_empty():
				var p: PopupMenu = row["_popup"]
				var at := -1
				for k in p.item_count:
					if p.get_item_id(k) == int(row["_id"]):
						at = k
				_ok(at >= 0 and p.is_item_checkable(at) and not p.is_item_disabled(at), "desktop: a live check item")
		"tablet":
			_ok(not row.is_empty(), "tablet: the row is PRESENT (disabled-with-reason, not omitted)")
			if not row.is_empty():
				_ok(not bool(row["available"]), "tablet: it is unavailable")
				_ok(String(row["why"]).begins_with("Not available on a tablet: every panel opens inside this window"), "tablet: the reason is the literal sentence: '%s'" % String(row["why"]))
				var p: PopupMenu = row["_popup"]
				var at := -1
				for k in p.item_count:
					if p.get_item_text(k) == ROW:
						at = k
				_ok(at >= 0 and p.is_item_disabled(at) and not p.is_item_checkable(at), "tablet: the popup item is disabled and has no tick to flip")
				_ok(at >= 0 and p.get_item_tooltip(at) == String(row["why"]), "tablet: the tooltip carries the reason")
		"phone":
			_ok(row.is_empty(), "phone: the command index has no such row")
			_ok(_popups_with_row() == 0, "phone: no PopupMenu under the shell has the row (absent, not disabled)")
	_ok(not DccSettings.popout_enabled(), "looking at the row stored nothing")


# -- zero native windows ------------------------------------------------------------------------------

func _generate() -> void:
	_bridge.generate({"seed": 483920, "width_km": 1200.0, "grid_w": 160, "grid_h": 120, "archetype": "", "villages": true})
	while _bridge.generating:
		await get_tree().create_timer(0.1).timeout
	await _settle(10)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()


func _nwin() -> int:
	return DisplayServer.get_window_list().size()


func _zero_native() -> void:
	print("== 4. a stored setting that is ON, on a form that cannot pop out: zero native windows ==")
	if _form == "desktop":
		print("  (desktop pops out by design; the zero-native bar is the other two forms')")
		return
	# an old or synced setting: written straight to the store, as the row cannot
	DccSettings.set_popout_value("enabled", true)
	_ok(DccSettings.popout_enabled(), "the stored setting is ON (written past the row)")
	_ok(not PopoutHost.supported(), "supported() is false")
	var wins: Array = [_app.faction_roster_window, _app.settlement_types_window, _app.place_editor_window,
		_app.generation_rules_window, _app.city_viewer_window, _app.vault_window, _app.assistant_window]
	_app.faction_roster_window.open(1, "identity")
	_app.settlement_types_window.open()
	_app.place_editor_window.open_for(0)
	_app.generation_rules_window.open()
	_app.city_viewer_window.open(0)
	_app.vault_window.open_browse()
	_app.assistant_window.open_window()
	await _settle(8)
	_ok(_nwin() == 1, "seven listed windows open: OS window count is %d (want 1)" % _nwin())
	var native := 0
	var buttons := 0
	for w in wins:
		if (w as Window).force_native:
			native += 1
		if (w as Window).find_child(PopoutHost.BUTTON_NAME, true, false) != null:
			buttons += 1
		_ok((w as Window).is_embedded() or not (w as Window).visible, "%s is embedded" % (w as Window).title)
	_ok(native == 0, "no listed window has force_native (%d)" % native)
	_ok(buttons == 0, "no Pop out button on any of them (%d)" % buttons)
	for w in wins:
		(w as Window).hide()
	var d := AcceptDialog.new()
	d.title = "roster"
	add_child(d)
	_ok(PopoutHost.prepare(d, PackedStringArray([PopoutHost.key_for(d)])) == false and not d.force_native, "prepare() on a window whose key is live still answers embedded")
	_ok(PopoutHost.add_toggle_button(d) == null, "add_toggle_button() adds nothing")
	d.queue_free()
	DccSettings.set_popout_value("enabled", false)
	await _settle(2)
	_ok(_nwin() == 1, "still one OS window at the end")


# -- geometry, live (desktop) ---------------------------------------------------------------------

func _geo_live() -> void:
	print("== 5. desktop, REAL screen: a roster stored on a monitor that does not exist ==")
	var n := DisplayServer.get_screen_count()
	print("  screens attached: %d (the second-monitor halves stay owner-run)" % n)
	var scr0 := Rect2i(DisplayServer.screen_get_position(0), DisplayServer.screen_get_size(0))
	var usable := DisplayServer.screen_get_usable_rect(0)
	var ghost := Rect2i(scr0.end.x, 0, 2560, 1440)
	var key := "faction_roster_window"
	DccSettings.set_popout_enabled(true)
	DccSettings.set_popout_geometry(key, _geo(ghost.position.x + 300, 200, 880, 620, ghost))
	_app.faction_roster_window.open(1, "identity")
	await _until(func() -> bool: return _nwin() == 2)
	await _settle(8)
	var w: Window = _app.faction_roster_window
	_ok(not w.is_embedded() and _nwin() == 2, "the roster opened as a native window (count %d)" % _nwin())
	var rect := Rect2i(w.position, w.size)
	_ok(usable.encloses(rect), "its rectangle %s is inside this machine's usable area %s (not on the ghost monitor)" % [str(rect), str(usable)])
	var vis := PopoutGeometry.title_visible_px(rect, PopoutGeometry.screens_now())
	_ok(vis >= 64, "its title row is reachable: %d px visible" % vis)
	_ok(w.current_screen == DisplayServer.get_primary_screen() or n == 1, "it is on the primary screen (%d)" % w.current_screen)
	w.hide()
	await _settle(4)
	var g := DccSettings.popout_geometry(key)
	_ok(not g.is_empty() and Rect2i(g["sx"], g["sy"], g["sw"], g["sh"]) == scr0, "closing stored the real screen rectangle, not the ghost: %s" % str(g))
	DccSettings.set_popout_enabled(false)

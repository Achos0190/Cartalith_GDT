extends Node
## Probe for PW-1 of `POPOUT_WINDOWS_SCOPE.md` (Ruling DK): the pop-out SETTING, the GEOMETRY
## store and the ZERO-COST-WHEN-OFF guarantee, against the real `DccSettings`, `PopoutGeometry`,
## `PopoutHost` and the real shell.
##
##   desktop: godot --path . --resolution 1400x900 _popout_settings_probe.tscn -- --form desktop [--setting on] [--shots <dir>] [--pure]
##   tablet:  godot --path . --resolution 1280x800 _popout_settings_probe.tscn -- --form tablet  [--setting on] --force-touch
##   phone:   godot --path . --resolution 393x852  _popout_settings_probe.tscn -- --form phone   [--setting on] --force-touch
##
## **Windowed, never `--headless`** (it generates a world, exports a PNG and opens a real native
## window). Arguments: `--form desktop|tablet|phone` (required; the probe checks the shell booted
## into that form, so a wrong `--resolution` cannot read as a pass), `--setting on|off` (default
## off: what the pop-out setting holds BEFORE the shell boots, so the windows are prepared under
## it), `--force-touch` (the shell's own flag) and `--shots <dir>`. Anything else aborts, exit 2.
## **Run it from a scratch worktree with its own `config/name` and a wiped `app_userdata/<name>`**
## (`MISTAKES.md`: a probe that reads `user://` is green for reasons unrelated to the code);
## section 0 fails if the directory was not clean. Only one monitor is assumed, and only the
## single-monitor halves of the bars are claimed.
##
## What it protects (the PW-1 bars):
##   0. clean user dir; the shipped default is OFF; reading writes nothing.
##   1. the STORE: round trip through a reload; REFUSALS write nothing (byte-identical file); a
##      hand-edited file reads safe (junk dropped, over-cap cut, wrong types default); a newer
##      `version` reads as the defaults and is not rewritten by reading; a real PRIOR document
##      (written before `[popout]` existed) still resolves and keeps its bytes.
##   2. the RESTORE TABLE: `PopoutGeometry.resolve()` over six literal cases (same screen, screen 2
##      absent, resolution smaller, negative coordinates, zero screens, window larger than the
##      usable rect) plus nothing-stored and malformed, each with at least 64 px of title row
##      visible by an INDEPENDENT oracle (`title_visible_px`), and the decision table of
##      `PopoutHost.wants_native()` (every term of the conjunction flipped alone).
##   3. THE SESSION: boot, generate, open every listed window, hide it, save, export -- and at
##      EVERY step `DisplayServer.get_window_list().size() == 1` and no shell window has
##      `force_native` (with the setting off AND with it on, because `LIVE_KEYS` is empty in PW-1
##      and tablet/phone never pop out).
##   4. (desktop) THE NATIVE BRANCH on a scratch dialog the probe allows through `live`: setting
##      off keeps it embedded and stores nothing; setting on makes it native, transient,
##      non-exclusive, the window count 2; its rectangle and screen are stored on hide and put
##      back by the real `popup_centered()` path; a stored rectangle off every screen opens
##      clamped inside the usable area with the title row visible.
##   5. (desktop) THE ROW: `Preferences > Open panels in their own window` exists, toggles the
##      stored setting, and is absent on tablet and phone.
##   6. (desktop) LEAK: with geometry stored, a saved project (every zip entry), the raw zip and an
##      exported PNG hold none of `[popout]`, the window key or the stored numbers; a positive
##      control (the zip with a `[popout]` entry appended) must trip the detector.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument/watchdog.

const FIXTURE := "res://../docs/evidence/quick-bar-2026-10-26/prior_cartalith_settings.cfg"
const KEY := "pw1_scratch"
const ROW := "Open panels in their own window"

var _fails := 0
var _checks := 0
var _form := ""
var _setting := false
var _shots := ""
## `--pure`: run only sections 0-2 (store and tables, no shell, no window) and finish. The fast leg the
## mutation run uses for the logic that needs no window; never counted as the PW-1 session bar.
var _pure := false
var _app: Node
var _bridge
var _steps: Array = []


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _cfg_path() -> String:
	return ProjectSettings.globalize_path(DccSettings.CONFIG_PATH)


## Forgets the in-memory ConfigFile so the next accessor re-reads the file, as a relaunch does.
func _reload() -> void:
	DccSettings._loaded = false
	DccSettings._cfg = null


func _text(path: String) -> String:
	var f := FileAccess.open(path, FileAccess.READ)
	return f.get_as_text().replace("\r\n", "\n") if f != null else ""


func _bytes(path: String) -> PackedByteArray:
	var f := FileAccess.open(path, FileAccess.READ)
	return f.get_buffer(f.get_length()) if f != null else PackedByteArray()


func _write_text(path: String, text: String) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _contains(hay: PackedByteArray, needle: String) -> bool:
	var n := needle.to_utf8_buffer()
	if n.is_empty() or hay.size() < n.size():
		return false
	var first := n[0]
	for i in range(0, hay.size() - n.size() + 1):
		if hay[i] != first:
			continue
		var hit := true
		for j in range(1, n.size()):
			if hay[i + j] != n[j]:
				hit = false
				break
		if hit:
			return true
	return false


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks) form=%s setting=%s" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks, _form, "on" if _setting else "off"])
	get_tree().quit(0 if _fails == 0 else 1)


func _geo(x: int, y: int, w: int, h: int, s: Rect2i) -> Dictionary:
	return {"x": x, "y": y, "w": w, "h": h, "sx": s.position.x, "sy": s.position.y, "sw": s.size.x, "sh": s.size.y}


func _scr(r: Rect2i, usable: Rect2i = Rect2i()) -> Dictionary:
	return {"rect": r, "usable": r if usable.size == Vector2i.ZERO else usable}


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--form" and i + 1 < args.size():
			_form = String(args[i + 1])
			i += 2
		elif a == "--setting" and i + 1 < args.size():
			_setting = String(args[i + 1]) == "on"
			if String(args[i + 1]) != "on" and String(args[i + 1]) != "off":
				_form = ""
			i += 2
		elif a == "--shots" and i + 1 < args.size():
			_shots = String(args[i + 1])
			i += 2
		elif a == "--pure":
			_pure = true
			i += 1
		elif a == "--force-touch":
			i += 1
		else:
			print("ABORT: unknown argument %s" % a)
			get_tree().quit(2)
			return
	if not _form in ["desktop", "tablet", "phone"]:
		print("ABORT: --form desktop|tablet|phone is required")
		get_tree().quit(2)
		return
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	var path := _cfg_path()

	print("== 0. clean user dir, shipped default OFF, reads do not write ==")
	_ok(not FileAccess.file_exists(path), "no cartalith_settings.cfg at start (the user dir was wiped): %s" % path)
	_ok(DccSettings.popout_enabled() == false, "the pop-out setting defaults to OFF")
	_ok(DccSettings.popout_value("geometry") == {}, "geometry defaults to the empty map")
	_ok(DccSettings.popout_value("version") == 1, "version reads 1")
	_ok(DccSettings.popout_value("nope") == null, "an unknown key reads null, not a value")
	_ok(DccSettings.popout_geometry("anything") == {}, "a window with nothing stored reads {} (absence, not a zero rectangle)")
	_ok(not DccSettings.has_popout_value("enabled"), "a never-stored switch is 'never configured'")
	_ok(not FileAccess.file_exists(path), "reading every key wrote nothing")

	if _form == "desktop" and not _setting:
		_store()
		_tables()
	if _pure:
		_finish()
		return
	_reload()
	_write_text(path, "")
	_reload()
	if _setting:
		_ok(DccSettings.set_popout_enabled(true), "the setting is stored ON before the shell boots")

	print("== 3. the session (form %s, setting %s) ==" % [_form, "on" if _setting else "off"])
	await _session()
	if _form == "desktop":
		await _native_branch()
		await _row()
		await _leak()
	else:
		await _row()
	_finish()


# -- 1 ------------------------------------------------------------------------------------

func _store() -> void:
	print("== 1. the store ==")
	var path := _cfg_path()
	var s := Rect2i(0, 0, 1920, 1080)
	var ga := _geo(100, 80, 880, 620, s)
	var gb := _geo(-1500, 40, 700, 500, Rect2i(-1920, 0, 1920, 1080))
	_ok(DccSettings.set_popout_enabled(true), "enabled stored")
	_ok(DccSettings.set_popout_geometry("faction_roster_window", ga), "geometry for the roster stored")
	_ok(DccSettings.set_popout_geometry("vault_window", gb), "geometry for the vault stored (negative x)")
	_reload()
	_ok(DccSettings.popout_enabled() == true, "enabled true survives a reload")
	_ok(DccSettings.popout_geometry("faction_roster_window") == ga, "roster geometry equals what was stored after a reload")
	_ok(DccSettings.popout_geometry("vault_window").get("x") == -1500, "negative x is stored literally (-1500)")
	_ok(DccSettings.popout_geometry("faction_roster_window").get("w") == 880 and DccSettings.popout_geometry("faction_roster_window").get("sw") == 1920, "w and the screen width read back as 880 and 1920 (literals)")
	_ok(DccSettings.has_popout_value("enabled") and DccSettings.has_popout_value("geometry"), "stored keys report as stored")
	_ok(DccSettings.set_popout_enabled(false), "enabled can be turned off again")
	_reload()
	_ok(DccSettings.popout_enabled() == false and DccSettings.has_popout_value("enabled"), "off is a stored value, distinct from never configured")
	var many := {}
	for n in 32:
		many["w%02d" % n] = ga
	_ok(DccSettings.set_popout_value("geometry", many), "exactly 32 windows are accepted")
	_ok(not DccSettings.set_popout_geometry("w_33rd", ga), "a 33rd window is refused")
	_ok(DccSettings.set_popout_value("geometry", {"faction_roster_window": ga}), "back to one window")

	print("== 1b. refusals write nothing ==")
	var before := _bytes(path)
	_ok(before.size() > 0, "the file exists to compare against (%d bytes)" % before.size())
	var bad_missing := ga.duplicate()
	bad_missing.erase("sh")
	var bad_type := ga.duplicate()
	bad_type["x"] = "100"
	var bad_w := ga.duplicate()
	bad_w["w"] = 0
	var bad_sw := ga.duplicate()
	bad_sw["sw"] = -5
	var bad_float := ga.duplicate()
	bad_float["y"] = 80.5
	var refusals := {
		"unknown key": ["weird", true],
		"version is the format's, not the user's": ["version", 99],
		"enabled given an int": ["enabled", 1],
		"enabled given a String": ["enabled", "true"],
		"geometry given an Array": ["geometry", [ga]],
		"an empty window key": ["geometry", {"": ga}],
		"a non-String window key": ["geometry", {7: ga}],
		"an entry that is not a Dictionary": ["geometry", {"a": 5}],
		"an entry missing sh": ["geometry", {"a": bad_missing}],
		"an entry with a String x": ["geometry", {"a": bad_type}],
		"an entry with w = 0": ["geometry", {"a": bad_w}],
		"an entry with sw < 0": ["geometry", {"a": bad_sw}],
		"an entry with a float y": ["geometry", {"a": bad_float}],
		"33 windows": ["geometry", many.merged({"w_33rd": ga})],
	}
	for what in refusals:
		var r: Array = refusals[what]
		_ok(not DccSettings.set_popout_value(String(r[0]), r[1]), "refused: %s" % what)
	_ok(not DccSettings.set_popout_geometry("", ga), "set_popout_geometry refuses an empty key")
	_ok(not DccSettings.set_popout_geometry("a", bad_w), "set_popout_geometry refuses a zero-width entry")
	_ok(_bytes(path) == before, "the file is byte-identical after all %d refusals" % (refusals.size() + 2))

	print("== 1c. a hand-edited file reads safe ==")
	var hand := "[popout]\n\nversion=1\nenabled=\"yes\"\ngeometry={\"good\": {\"x\": 1, \"y\": 2, \"w\": 3, \"h\": 4, \"sx\": 0, \"sy\": 0, \"sw\": 800, \"sh\": 600}, \"\": {\"x\": 1, \"y\": 2, \"w\": 3, \"h\": 4, \"sx\": 0, \"sy\": 0, \"sw\": 800, \"sh\": 600}, \"nodict\": 5, \"zero_w\": {\"x\": 1, \"y\": 2, \"w\": 0, \"h\": 4, \"sx\": 0, \"sy\": 0, \"sw\": 800, \"sh\": 600}, \"floaty\": {\"x\": 1.5, \"y\": 2, \"w\": 3, \"h\": 4, \"sx\": 0, \"sy\": 0, \"sw\": 800, \"sh\": 600}, \"missing\": {\"x\": 1}}\nextra_key=5\n"
	_write_text(path, hand)
	_reload()
	_ok(DccSettings.popout_enabled() == false, "a wrong-typed switch reads as its default (off)")
	var hg: Dictionary = DccSettings.popout_value("geometry")
	_ok(hg.keys() == ["good"], "only the one well-formed entry survives: %s" % str(hg.keys()))
	_ok(DccSettings.popout_value("extra_key") == null, "an unknown stored key is not exposed")
	var over := "[popout]\n\nversion=1\ngeometry={"
	for n in 40:
		over += "\"k%02d\": {\"x\": 1, \"y\": 2, \"w\": 3, \"h\": 4, \"sx\": 0, \"sy\": 0, \"sw\": 800, \"sh\": 600}%s" % [n, ", " if n < 39 else ""]
	over += "}\n"
	_write_text(path, over)
	_reload()
	var og: Dictionary = DccSettings.popout_value("geometry")
	_ok(og.size() == 32 and og.has("k00") and og.has("k31") and not og.has("k32"), "40 stored windows are cut to the first 32 by key order: %d" % og.size())
	_write_text(path, "[popout]\n\nversion=1\nenabled=true\ngeometry=[1, 2]\n")
	_reload()
	_ok(DccSettings.popout_enabled() == true and DccSettings.popout_value("geometry") == {}, "a wrong-typed geometry reads empty without throwing; a good switch still reads")
	var newer := "[popout]\n\nversion=2\nenabled=true\nfuture_key=1\n"
	_write_text(path, newer)
	_reload()
	_ok(DccSettings.popout_enabled() == false and DccSettings.popout_value("geometry") == {}, "a version-2 section reads as the defaults (the switch is NOT on)")
	_ok(not DccSettings.has_popout_value("enabled"), "and reports nothing stored")
	_ok(_text(path) == newer, "reading it did not rewrite the file")
	_write_text(path, "[popout]\n\nversion=\"two\"\nenabled=true\n")
	_reload()
	_ok(DccSettings.popout_enabled() == false, "a non-int version also reads as the defaults")
	_write_text(path, newer)
	_reload()
	_ok(DccSettings.set_popout_enabled(true), "the first user edit is accepted")
	_reload()
	_ok(DccSettings.popout_enabled() == true and DccSettings.popout_value("version") == 1 and not _text(path).contains("future_key"), "it replaced the section: version 1, the new value, the newer build's key dropped (documented)")

	print("== 1d. backward compatibility: a real prior document ==")
	var fixture := _text(ProjectSettings.globalize_path(FIXTURE))
	_ok(fixture.length() > 400 and fixture.contains("[handedness]") and fixture.contains("[llm]") and not fixture.contains("[popout]"),
		"the fixture was written by the pre-PW-1 setters: %d chars, no [popout]" % fixture.length())
	_write_text(path, fixture)
	_reload()
	_ok(DccSettings.dominant_hand() == "left" and DccSettings.theme_mode() == "light" and DccSettings.units_mode() == "mi", "handedness, theme and units resolve from the prior file")
	_ok(DccSettings.popout_enabled() == false and not DccSettings.has_popout_value("enabled"), "a file with no [popout] reads as OFF and never configured")
	_ok(DccSettings.set_popout_enabled(true) and DccSettings.set_popout_geometry("vault_window", gb), "[popout] keys are written into it")
	var after := _text(path)
	var cut := after.find("[popout]")
	_ok(cut > 0 and after.substr(0, cut).strip_edges() == fixture.strip_edges(), "everything before [popout] is byte-identical to the fixture (%d chars)" % cut)
	_reload()
	_ok(DccSettings.dominant_hand() == "left" and DccSettings.popout_geometry("vault_window") == gb, "and both still resolve after a reload")

	print("== 1e. the allow-list ==")
	_ok(DccSettings.POPOUT_DEFAULTS.size() == 3, "three keys: version + enabled + geometry (literal)")
	_ok(DccSettings.POPOUT_DEFAULTS["enabled"] == false, "the shipped default of enabled is false (Ruling DK Q1)")
	_ok(DccSettings.POPOUT_MAX_WINDOWS == 32, "the window cap is 32 (literal)")
	for k in DccSettings.POPOUT_DEFAULTS:
		_ok(not String(k).contains("secret") and k != "key" and k != "api_key" and k != "token", "allow-list entry '%s' is not a secret slot" % k)


# -- 2 ------------------------------------------------------------------------------------

func _check_case(name: String, saved: Dictionary, screens: Array, primary: int, want_rect: Rect2i, want_screen: int, want_how: String) -> void:
	var r := PopoutGeometry.resolve(saved, screens, primary)
	_ok(bool(r.get("placed", false)), "%s: placed" % name)
	if not bool(r.get("placed", false)):
		return
	var rect: Rect2i = r["rect"]
	_ok(rect == want_rect, "%s: rect %s == %s" % [name, str(rect), str(want_rect)])
	_ok(int(r["screen"]) == want_screen and String(r["how"]) == want_how, "%s: screen %d by '%s' (want %d '%s')" % [name, int(r["screen"]), String(r["how"]), want_screen, want_how])
	var u: Rect2i = screens[int(r["screen"])]["usable"]
	_ok(u.encloses(rect), "%s: the window lies inside the usable area %s" % [name, str(u)])
	_ok(PopoutGeometry.title_visible_px(rect, screens) >= 64, "%s: >= 64 px of title row visible by the independent oracle (%d)" % [name, PopoutGeometry.title_visible_px(rect, screens)])


func _tables() -> void:
	print("== 2. the restore table (literals) ==")
	var s0 := Rect2i(0, 0, 1920, 1080)
	var u0 := Rect2i(0, 0, 1920, 1040)
	var s1 := Rect2i(1920, 0, 2560, 1440)
	var u1 := Rect2i(1920, 0, 2560, 1400)
	var two := [_scr(s0, u0), _scr(s1, u1)]
	# 1 same screen: on monitor 2, restored byte-for-byte
	_check_case("same screen (monitor 2)", _geo(2400, 200, 880, 620, s1), two, 0, Rect2i(2400, 200, 880, 620), 1, "same_screen")
	# 2 screen 2 absent: only monitor 1 now; centred in its usable area below the title row
	_check_case("screen 2 absent", _geo(2400, 200, 880, 620, s1), [_scr(s0, u0)], 0, Rect2i(520, 32 + (1040 - 32 - 620) / 2, 880, 620), 0, "primary")
	# 3 resolution smaller: the screen is now 1280x720 at the same origin; shrunk and clamped
	var small := Rect2i(0, 0, 1280, 720)
	_check_case("resolution smaller", _geo(1500, 900, 800, 600, s0), [_scr(small)], 0, Rect2i(480, 120, 800, 600), 0, "same_origin")
	# 4a negative coordinates, the screen still exists to the left of the primary
	var left := Rect2i(-1920, 0, 1920, 1080)
	_check_case("negative coordinates (screen present)", _geo(-1500, 100, 700, 500, left), [_scr(left), _scr(Rect2i(0, 0, 1920, 1080))], 1, Rect2i(-1500, 100, 700, 500), 0, "same_screen")
	# 4b negative coordinates, that screen is gone: centred on the primary
	_check_case("negative coordinates (screen gone)", _geo(-1500, 100, 700, 500, left), [_scr(Rect2i(0, 0, 1920, 1080))], 0, Rect2i(610, 32 + (1080 - 32 - 500) / 2, 700, 500), 0, "primary")
	# 5 zero screens reported: nothing placed, and NO rectangle in the answer
	var none := PopoutGeometry.resolve(_geo(100, 100, 800, 600, s0), [], 0)
	_ok(not bool(none["placed"]) and not none.has("rect") and not none.has("screen"), "zero screens: placed false and no rect/screen key (absence, not (0,0,w,h))")
	# 6 window larger than the usable rect: shrunk to usable width and (height - title row)
	_check_case("window larger than the usable rect", _geo(0, 0, 3000, 2000, s0), [_scr(s0, u0)], 0, Rect2i(0, 32, 1920, 1008), 0, "same_screen")
	# extras: nothing stored, malformed, the window half off the right edge, the stored screen moved index
	_ok(not bool(PopoutGeometry.resolve({}, two, 0)["placed"]), "nothing stored: not placed")
	var mal := _geo(1, 1, 1, 1, s0)
	mal.erase("sw")
	_ok(not bool(PopoutGeometry.resolve(mal, two, 0)["placed"]), "a malformed entry: not placed")
	_check_case("half off the right edge of monitor 1", _geo(1800, 300, 700, 500, s0), [_scr(s0, u0)], 0, Rect2i(1220, 300, 700, 500), 0, "same_screen")
	_check_case("above the usable top (title bar would be off-screen)", _geo(100, 0, 700, 500, s0), [_scr(s0, u0)], 0, Rect2i(100, 32, 700, 500), 0, "same_screen")
	_check_case("screens reordered: monitor 2 is now index 0", _geo(2400, 200, 880, 620, s1), [_scr(s1, u1), _scr(s0, u0)], 1, Rect2i(2400, 200, 880, 620), 0, "same_screen")
	# the oracle itself must be able to say NO
	_ok(PopoutGeometry.title_visible_px(Rect2i(5000, 5000, 700, 500), two) == 0, "oracle control: a window far off every screen shows 0 px")
	_ok(PopoutGeometry.title_visible_px(Rect2i(100, 0, 700, 500), [_scr(s0, u0)]) == 0, "oracle control: a title bar above the top edge shows 0 px")
	_ok(PopoutGeometry.title_visible_px(Rect2i(1900, 100, 700, 500), [_scr(s0, u0)]) == 20, "oracle control: 20 px of a window overhanging the right edge shows 20 px")

	print("== 2b. the decision table of PopoutHost.wants_native ==")
	var live := PackedStringArray(["a", "b"])
	var rows := [
		[true, true, "a", true], [true, true, "b", true],
		[false, true, "a", false],   # setting off
		[true, false, "a", false],   # platform cannot
		[true, true, "c", false],    # key not live
		[false, false, "c", false],
		[true, true, "", false],     # empty key
	]
	for r in rows:
		_ok(PopoutHost.wants_native(r[0], r[1], r[2], live) == r[3], "wants_native(setting=%s, platform=%s, key='%s') == %s" % [r[0], r[1], r[2], r[3]])
	_ok(PopoutHost.wants_native(true, true, "a", PackedStringArray()) == false, "an empty live list lets nothing out")
	_ok(PopoutHost.LIVE_KEYS.is_empty(), "PW-1 ships LIVE_KEYS empty (PW-2 adds the roster)")


# -- 3 ------------------------------------------------------------------------------------

func _step(label: String) -> void:
	var n := DisplayServer.get_window_list().size()
	_steps.append([label, n])
	if n != 1:
		print("  step '%s': window count %d" % [label, n])


func _native_shell_windows() -> Array:
	var out: Array = []
	for w in _app.find_children("*", "Window", true, false):
		if (w as Window).force_native:
			out.append(String(w.name))
	return out


func _session() -> void:
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	var got := "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_ok(got == _form, "the shell booted into the %s form (got %s, viewport %s)" % [_form, got, str(get_viewport().get_visible_rect().size)])
	_ok(PopoutHost.supported() == (_form == "desktop"), "PopoutHost.supported() is true only on the desktop form (%s)" % str(PopoutHost.supported()))
	_ok(DccSettings.popout_enabled() == _setting, "the setting held what --setting asked for (%s)" % str(DccSettings.popout_enabled()))
	_bridge = _app.bridge
	_step("boot")
	_bridge.generate({"seed": 9137, "width_km": 1200.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true})
	var polls := 0
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
		polls += 1
		if DisplayServer.get_window_list().size() != 1:
			_step("during generate")
			break
	_step("generate")
	await _settle(10)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	_ok(_bridge.has_world, "a world exists (%d polls)" % polls)
	_step("world ready")

	var listed := {
		"faction_roster_window": _app.faction_roster_window, "settlement_types_window": _app.settlement_types_window,
		"place_editor_window": _app.place_editor_window, "generation_rules_window": _app.generation_rules_window,
		"city_viewer_window": _app.city_viewer_window, "vault_window": _app.vault_window,
		"assistant_window": _app.assistant_window,
	}
	var also := {
		"world_data_window": _app.world_data_window, "quick_bar_window": _app.quick_bar_window,
		"assistant_settings_window": _app.assistant_settings_window, "data_manager_window": _app.data_manager_window,
		"asset_library_window": _app.asset_library_window, "travel_library_window": _app.travel_library_window,
	}
	_ok(listed.size() == 7, "seven listed windows (the v1 roster, scope s.4): %d" % listed.size())
	for k in listed:
		_ok(listed[k] != null and PopoutHost.key_for(listed[k]) != "", "'%s' exists, key '%s'" % [k, PopoutHost.key_for(listed[k]) if listed[k] != null else ""])
	for name in listed:
		var w: Window = listed[name]
		match name:
			"place_editor_window": w.open_for(0)
			"city_viewer_window": w.open(0)
			"vault_window": w.open_overview()
			_: w.open()
		await _settle(5)
		_ok(w.visible, "%s opened" % name)
		_ok(w.is_embedded(), "%s is an embedded window (not native)" % name)
		_step("open " + name)
		w.hide()
		await _settle(2)
	for name in also:
		var w: Window = also[name]
		w.open()
		await _settle(5)
		_step("open " + name)
		w.hide()
		await _settle(2)
	_ok(_native_shell_windows().is_empty(), "no shell window has force_native set: %s" % str(_native_shell_windows()))

	var sdir := ProjectSettings.globalize_path("user://_popout_settings_probe")
	DirAccess.make_dir_recursive_absolute(sdir)
	var zpath := sdir.path_join("session.zip")
	_ok(_bridge.save_project(zpath), "the project saved")
	_step("save")
	var dlg: AcceptDialog = _app.open_export_dialog()
	await _settle(4)
	_step("export dialog open")
	dlg.set_destination(sdir.path_join("session.png"))
	dlg.start_export()
	var res: Dictionary = await dlg.export_finished
	_ok(bool(res.get("ok", false)), "a PNG export ran: %s" % str(res.get("ok", res)))
	dlg.hide()
	await _settle(3)
	_step("export")
	var bad := _steps.filter(func(s): return int(s[1]) != 1)
	_ok(bad.is_empty(), "DisplayServer.get_window_list().size() == 1 at all %d steps%s" % [_steps.size(), "" if bad.is_empty() else ": " + str(bad)])
	_ok(_native_shell_windows().is_empty(), "still no force_native shell window after the session: %s" % str(_native_shell_windows()))


# -- 4 ------------------------------------------------------------------------------------

func _scratch() -> AcceptDialog:
	var d := AcceptDialog.new()
	d.title = KEY
	d.size = Vector2i(500, 400)
	add_child(d)
	return d


func _native_branch() -> void:
	print("== 4. the native branch (scratch dialog, one monitor) ==")
	var live := PackedStringArray([KEY])
	# setting OFF: embedded and stores nothing
	DccSettings.set_popout_enabled(false)
	var e := _scratch()
	_ok(PopoutHost.prepare(e, live) == false and e.force_native == false, "setting off: prepare() leaves the scratch embedded")
	e.popup_centered()
	await _settle(4)
	_ok(e.is_embedded() and DisplayServer.get_window_list().size() == 1, "setting off: embedded, window count 1")
	_ok(PopoutGeometry.capture(e).is_empty(), "capture() refuses an embedded window (its position is relative to the main window, meaningless on a screen)")
	e.hide()
	await _settle(2)
	_ok(DccSettings.popout_geometry(KEY).is_empty(), "an embedded window stores no geometry on hide")
	e.queue_free()
	await _settle(2)
	# setting ON but key not live: still embedded
	DccSettings.set_popout_enabled(true)
	var nl := _scratch()
	_ok(PopoutHost.prepare(nl, PackedStringArray(["other"])) == false and nl.force_native == false, "setting on, key not live: embedded")
	nl.queue_free()
	await _settle(2)
	# setting ON and live: native
	var d := _scratch()
	_ok(PopoutHost.prepare(d, live), "setting on and the key live: native")
	_ok(d.force_native and d.transient and not d.exclusive, "force_native + transient, not exclusive")
	d.popup_centered()
	await _settle(6)
	_ok(not d.is_embedded() and DisplayServer.get_window_list().size() == 2, "a second OS window exists: embedded=%s count=%d" % [str(d.is_embedded()), DisplayServer.get_window_list().size()])
	var usable := DisplayServer.screen_get_usable_rect(0)
	var scr := Rect2i(DisplayServer.screen_get_position(0), DisplayServer.screen_get_size(0))
	d.position = Vector2i(200, 150)
	d.size = Vector2i(520, 410)
	await _settle(4)
	_ok(d.position == Vector2i(200, 150) and d.size == Vector2i(520, 410), "the test moved it to (200,150) 520x410: %s %s" % [str(d.position), str(d.size)])
	d.hide()
	await _settle(3)
	var g := DccSettings.popout_geometry(KEY)
	_ok(g.get("x") == 200 and g.get("y") == 150 and g.get("w") == 520 and g.get("h") == 410, "hiding stored x=200 y=150 w=520 h=410: %s" % str(g))
	_ok(g.get("sx") == scr.position.x and g.get("sy") == scr.position.y and g.get("sw") == scr.size.x and g.get("sh") == scr.size.y, "and the screen rectangle it was on %s" % str(scr))
	_ok(DisplayServer.get_window_list().size() == 1, "the OS window is gone after hide (count 1)")
	d.popup_centered()
	await _settle(6)
	_ok(d.position == Vector2i(200, 150) and d.size == Vector2i(520, 410), "the real popup_centered() path restores the stored rectangle: %s %s (centred would differ)" % [str(d.position), str(d.size)])
	d.hide()
	await _settle(3)
	# a stored rectangle off every screen opens clamped inside the usable area
	DccSettings.set_popout_geometry(KEY, _geo(5000, 5000, 520, 410, scr))
	d.popup_centered()
	await _settle(6)
	var r := Rect2i(d.position, d.size)
	_ok(usable.encloses(r), "off-screen geometry opens clamped inside the usable area: %s in %s" % [str(r), str(usable)])
	_ok(PopoutGeometry.title_visible_px(r, PopoutGeometry.screens_now()) >= 64, "with >= 64 px of title row visible (%d)" % PopoutGeometry.title_visible_px(r, PopoutGeometry.screens_now()))
	d.hide()
	await _settle(2)
	# a larger-than-screen stored size shrinks
	DccSettings.set_popout_geometry(KEY, _geo(0, 0, 9000, 9000, scr))
	d.popup_centered()
	await _settle(6)
	r = Rect2i(d.position, d.size)
	_ok(usable.encloses(r) and r.size.x <= usable.size.x and r.size.y <= usable.size.y - PopoutGeometry.TITLE_ROW, "a 9000x9000 stored window is shrunk to fit: %s" % str(r))
	d.hide()
	d.queue_free()
	await _settle(3)
	_ok(DisplayServer.get_window_list().size() == 1, "no OS window left behind (count 1)")


# -- 5 ------------------------------------------------------------------------------------

func _row() -> void:
	print("== 5. the Preferences row ==")
	var idx := CommandIndex.new()
	idx.build(_app, _bridge)
	var row := {}
	for r in idx.all():
		if String(r["title"]) == ROW:
			row = r
	if _form != "desktop":
		_ok(row.is_empty(), "no '%s' row on the %s (it stays embedded; the tablet's disabled row is PW-5's)" % [ROW, _form])
		return
	_ok(not row.is_empty(), "CommandIndex finds the Preferences row BY TITLE on the desktop")
	if row.is_empty():
		return
	var popup: PopupMenu = row["_popup"]
	var id := int(row["_id"])
	var at := -1
	for k in popup.item_count:
		if popup.get_item_id(k) == id:
			at = k
	_ok(at >= 0 and popup.is_item_checkable(at), "it is a check item")
	DccSettings.set_popout_enabled(false)
	popup.set_item_checked(at, false)
	popup.id_pressed.emit(id)
	_ok(DccSettings.popout_enabled() == true and popup.is_item_checked(at), "choosing it turns the stored setting ON and ticks the row")
	popup.id_pressed.emit(id)
	_ok(DccSettings.popout_enabled() == false and not popup.is_item_checked(at), "choosing it again turns it OFF and unticks the row")
	_reload()
	_ok(DccSettings.popout_enabled() == false, "OFF survives a reload")
	_ok(popup.get_item_tooltip(at).contains("none does") or popup.get_item_tooltip(at).contains("today"), "the tooltip states honestly that no window honours it yet")
	if _shots != "":
		DccSettings.set_popout_enabled(true)
		popup.set_item_checked(at, true)
		popup.popup(Rect2i(Vector2i(40, 40), Vector2i(420, 700)))
		await _settle(6)
		popup.scroll_to_item(at)
		await _settle(4)
		get_viewport().get_texture().get_image().save_png(_shots.path_join("pw1_preferences_row_desktop.png"))
		print("SHOT pw1_preferences_row_desktop.png")
		popup.hide()
		DccSettings.set_popout_enabled(false)
		popup.set_item_checked(at, false)
		await _settle(2)


# -- 6 ------------------------------------------------------------------------------------

func _leak() -> void:
	print("== 6. LEAK: nothing reaches a world, an export or an archive ==")
	var scr := Rect2i(DisplayServer.screen_get_position(0), DisplayServer.screen_get_size(0))
	DccSettings.set_popout_enabled(true)
	DccSettings.set_popout_geometry("PWLEAK_window", _geo(913371, 727319, 515153, 434347, Rect2i(717173, 636361, 818183, 919193)))
	var needles := ["[popout]", "popout", "PWLEAK_window", "913371", "727319", "515153", "434347", "717173", "636361", "818183", "919193"]
	var sdir := ProjectSettings.globalize_path("user://_popout_settings_probe")
	var zpath := sdir.path_join("leak.zip")
	_ok(_bridge.save_project(zpath), "the project is saved while geometry is stored")
	var zr := ZIPReader.new()
	_ok(zr.open(zpath) == OK, "the saved project opens as a zip")
	var entries := zr.get_files()
	var leaked := PackedStringArray()
	for e in entries:
		var b := zr.read_file(e)
		# the numbers can occur by chance inside binary entries; only the text needles are searched in
		# every entry, and the numeric ones only in the text entries (json/txt/md)
		for n in needles:
			var numeric: bool = n.is_valid_int()
			if numeric and not (e.ends_with(".json") or e.ends_with(".txt") or e.ends_with(".md")):
				continue
			if _contains(b, n):
				leaked.append("%s in %s" % [n, e])
	_ok(entries.size() >= 5 and leaked.is_empty(), "%d zip entries searched for %d needles: %s" % [entries.size(), needles.size(), str(leaked)])
	var raw := _bytes(zpath)
	var raw_leak := PackedStringArray()
	for n in ["[popout]", "popout", "PWLEAK_window"]:
		if _contains(raw, n):
			raw_leak.append(n)
	_ok(raw.size() > 1000 and raw_leak.is_empty(), "the raw zip (%d bytes) holds none of the text needles: %s" % [raw.size(), str(raw_leak)])
	var cpath := sdir.path_join("control.zip")
	var packer := ZIPPacker.new()
	packer.open(cpath)
	for e in entries:
		packer.start_file(e)
		packer.write_file(zr.read_file(e))
		packer.close_file()
	packer.start_file("settings_leak.txt")
	packer.write_file("[popout]\ngeometry=PWLEAK_window".to_utf8_buffer())
	packer.close_file()
	packer.close()
	var zc := ZIPReader.new()
	zc.open(cpath)
	var hits := 0
	for e in zc.get_files():
		if _contains(zc.read_file(e), "[popout]"):
			hits += 1
	_ok(hits == 1, "positive control: a [popout] entry smuggled into the zip is detected (%d hit)" % hits)
	zc.close()
	zr.close()
	var png := _bytes(sdir.path_join("session.png"))
	var png_leak := PackedStringArray()
	for n in ["[popout]", "popout", "PWLEAK_window"]:
		if _contains(png, n):
			png_leak.append(n)
	_ok(png.size() > 1000 and png_leak.is_empty(), "the exported PNG (%d bytes) holds none of the text needles: %s" % [png.size(), str(png_leak)])
	_ok(_contains(_bytes(_cfg_path()), "PWLEAK_window") and _contains(_bytes(_cfg_path()), "[popout]"), "control: the settings file itself holds the geometry")
	DccSettings.set_popout_enabled(false)

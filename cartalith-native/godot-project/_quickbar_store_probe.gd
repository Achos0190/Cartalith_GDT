extends Node
## Probe for the quick bar's STORE (`CUSTOM_QUICK_BAR_SCOPE.md` milestone CQ-1, Ruling DH):
## `DccSettings`' allow-listed `[quick_bar]` section, and the bar that nothing of it ever
## reaches a world file, an export or a project archive.
##
##   godot --path . --resolution 1280x800 _quickbar_store_probe.tscn
##
## **Windowed, never `--headless`** (the leak section generates a world and exports a PNG).
## It reads no arguments and aborts with exit 2 on any (a usage header that names a flag the
## body never reads is a recorded mistake). **Run it from a scratch worktree with its own
## `config/name` and a wiped `app_userdata/<name>` directory** (`MISTAKES.md`: a probe that
## reads `user://` goes green for reasons unrelated to the code); section 0 fails if the
## directory was not clean.
##
## What it protects (the CQ-1 bars, each against the real `DccSettings`):
##   0. the user dir is clean, and a fresh read answers the shipped defaults WITHOUT writing;
##   1. round trip: bar, dock, share and the flag survive a reload from disk, in order;
##   2. REFUSALS write nothing: unknown key, `version`, wrong type, malformed pin, repeated
##      pin, 25th bar pin, 5th dock pin, 10th share pin, bad share key (the file is compared
##      byte for byte before and after every refusal);
##   3. a hand-edited file reads safe: over-cap lists are cut, malformed entries dropped,
##      wrong-typed values empty, a NEWER `version` reads empty and is not rewritten by
##      reading, and the first edit replaces it;
##   4. an unknown `pin_key` survives a load/save of other keys (a pin for a command this
##      build lacks is never deleted);
##   5. clearing the last pin PERSISTS the empty list (`bar=[]` on disk, `has_quick_bar_value`
##      true), against a control that would resurrect the old list;
##   6. backward compatibility from a REAL prior document: `prior_cartalith_settings.cfg`
##      (written by the pre-CQ-1 `DccSettings` setters at 5047cc99, not an empty fixture)
##      opens, every other section resolves to what was written, and after quick-bar writes
##      the file's text up to `[quick_bar]` is byte-identical to the fixture;
##   7. LEAK: after pinning, a generated world is saved to a `.zip` (every entry searched), an
##      exported PNG and the raw zip are searched, for `quick_bar` and for each pin's key and
##      title; a positive control (the same zip with a quick-bar entry appended) must trip the
##      detector, so a broken search cannot read as "0 leaks";
##   8. the allow-list: no `[quick_bar]` default key is a secret slot, and the section is
##      written ONLY through `set_quick_bar_value` (the table is the whole list).
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const FIXTURE := "res://../docs/evidence/quick-bar-2026-10-26/prior_cartalith_settings.cfg"
const GHOST_KEY := "menu:99999"
const GHOST_TITLE := "Ghost command QBPROBE"

var _fails := 0
var _checks := 0
var _app: Node
var _bridge


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


## Forgets the in-memory ConfigFile so the next accessor re-reads the file, exactly what a
## relaunch does.
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


func _pins(n: int, prefix: String = "menu:") -> Array:
	var out: Array = []
	for i in n:
		out.append({"k": "%s%d" % [prefix, 1000 + i], "t": "Pin %d" % i})
	return out


func _keys(pins: Array) -> Array:
	var out: Array = []
	for e in pins:
		out.append(String(e["k"]))
	return out


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _ready() -> void:
	if not OS.get_cmdline_user_args().is_empty():
		print("ABORT: this probe reads no arguments")
		get_tree().quit(2)
		return
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	var path := _cfg_path()

	print("== 0. clean user dir, shipped defaults, reads do not write ==")
	_ok(not FileAccess.file_exists(path), "no cartalith_settings.cfg at start (the user dir was wiped): %s" % path)
	_ok(DccSettings.quick_bar_value("bar") == [], "bar defaults to the empty list")
	_ok(DccSettings.quick_bar_value("dock") == [], "dock defaults to the empty list")
	_ok(DccSettings.quick_bar_value("share") == {}, "share defaults to the empty map")
	_ok(DccSettings.quick_bar_value("assistant_button") == true, "assistant_button defaults to true")
	_ok(DccSettings.quick_bar_value("version") == 1, "version reads 1")
	_ok(DccSettings.quick_bar_value("nope") == null, "an unknown key reads null, not a value")
	_ok(not DccSettings.has_quick_bar_value("bar"), "a never-stored list is 'never configured', distinct from an empty stored one")
	_ok(not FileAccess.file_exists(path), "reading every key wrote nothing")
	var d: Array = DccSettings.quick_bar_value("bar")
	d.append({"k": "x", "t": "y"})
	_ok(DccSettings.quick_bar_value("bar") == [], "the value is a copy: mutating it changes nothing stored")

	print("== 1. round trip through a reload ==")
	var bar := _pins(5)
	var dock := _pins(2, "extra:")
	var share := {"cartography.NE": _pins(3, "menu:"), "world.NW": _pins(1, "extra:")}
	_ok(DccSettings.set_quick_bar_value("bar", bar), "bar stored")
	_ok(DccSettings.set_quick_bar_value("dock", dock), "dock stored")
	_ok(DccSettings.set_quick_bar_value("share", share), "share stored")
	_ok(DccSettings.set_quick_bar_value("assistant_button", false), "assistant_button stored")
	_reload()
	_ok(DccSettings.quick_bar_value("bar") == bar, "bar equals what was stored, in order, after a reload")
	_ok(_keys(DccSettings.quick_bar_value("bar")) == ["menu:1000", "menu:1001", "menu:1002", "menu:1003", "menu:1004"], "bar order is the pin order (literal)")
	_ok(DccSettings.quick_bar_value("dock") == dock, "dock equals what was stored after a reload")
	_ok(DccSettings.quick_bar_value("share") == share, "share equals what was stored after a reload")
	_ok(DccSettings.quick_bar_value("assistant_button") == false, "assistant_button false survives a reload")
	_ok(DccSettings.has_quick_bar_value("bar") and DccSettings.has_quick_bar_value("assistant_button"), "stored keys report as stored")
	_ok(DccSettings.set_quick_bar_value("bar", _pins(24)), "exactly 24 bar pins are accepted")
	_ok(DccSettings.set_quick_bar_value("dock", _pins(4)), "exactly 4 dock pins are accepted")
	_ok(DccSettings.set_quick_bar_value("share", {"cartography.SW": _pins(9)}), "exactly 9 share pins are accepted")
	_ok((DccSettings.quick_bar_value("bar") as Array).size() == 24, "24 read back")
	_ok(DccSettings.quick_bar_add("bar", "menu:5", "x") != "", "the 25th bar pin is refused WITH a reason: '%s'" % DccSettings.quick_bar_add("bar", "menu:5", "x"))
	_ok("24" in DccSettings.quick_bar_add("bar", "menu:5", "x"), "the reason names the cap")
	_ok(DccSettings.quick_bar_add("dock", "menu:5", "x") != "" and "4" in DccSettings.quick_bar_add("dock", "menu:5", "x"), "the 5th dock pin is refused with its cap named")
	_ok(DccSettings.quick_bar_add("bar", "menu:1000", "x") == "Already pinned.", "a repeated pin is refused with a reason")
	_ok(DccSettings.set_quick_bar_value("bar", bar), "back to 5 pins")
	_ok(DccSettings.quick_bar_add("bar", "menu:77", "Seventy-seven") == "", "quick_bar_add appends")
	_ok(_keys(DccSettings.quick_bar_value("bar")).back() == "menu:77", "the new pin is last")
	_ok(DccSettings.quick_bar_move("bar", "menu:77", -2), "quick_bar_move moves by 2")
	_ok(_keys(DccSettings.quick_bar_value("bar")) == ["menu:1000", "menu:1001", "menu:1002", "menu:77", "menu:1003", "menu:1004"], "the moved pin sits two places earlier than the end (literal order)")
	_ok(not DccSettings.quick_bar_move("bar", "menu:1000", -1), "moving the first pin left does nothing and says so")
	_ok(not DccSettings.quick_bar_move("bar", "menu:absent", 1), "moving an absent pin does nothing")
	_ok(DccSettings.quick_bar_remove("bar", "menu:77"), "quick_bar_remove removes")
	_ok(DccSettings.quick_bar_value("bar") == bar, "bar is back to its five pins")

	print("== 2. refusals write nothing ==")
	var before := _bytes(path)
	_ok(before.size() > 0, "the file exists to compare against (%d bytes)" % before.size())
	var refusals := {
		"unknown key": ["weird", [1]],
		"version is the format's, not the user's": ["version", 99],
		"bar given a String": ["bar", "menu:1"],
		"bar given a Dictionary": ["bar", {"k": "a", "t": "b"}],
		"dock given an int": ["dock", 3],
		"share given an Array": ["share", [{"k": "a", "t": "b"}]],
		"assistant_button given an int": ["assistant_button", 1],
		"assistant_button given a String": ["assistant_button", "true"],
		"a pin that is not a Dictionary": ["bar", ["menu:1"]],
		"a pin with no k": ["bar", [{"t": "x"}]],
		"a pin with an empty k": ["bar", [{"k": "", "t": "x"}]],
		"a pin with a non-String k": ["bar", [{"k": 7, "t": "x"}]],
		"a pin with no t": ["bar", [{"k": "menu:1"}]],
		"a repeated k": ["bar", [{"k": "menu:1", "t": "a"}, {"k": "menu:1", "t": "b"}]],
		"25 bar pins": ["bar", _pins(25)],
		"5 dock pins": ["dock", _pins(5)],
		"10 share pins": ["share", {"cartography.NE": _pins(10)}],
		"a cardinal share key": ["share", {"cartography.N": _pins(1)}],
		"a share key with no domain": ["share", {".NE": _pins(1)}],
		"an upper-case share domain": ["share", {"Cartography.NE": _pins(1)}],
		"a share key with three parts": ["share", {"a.b.NE": _pins(1)}],
		"a share value that is not an Array": ["share", {"cartography.NE": "menu:1"}],
	}
	for what in refusals:
		var r: Array = refusals[what]
		_ok(not DccSettings.set_quick_bar_value(String(r[0]), r[1]), "refused: %s" % what)
	_ok(_bytes(path) == before, "the file is byte-identical after all %d refusals" % refusals.size())
	_ok(DccSettings.quick_bar_value("bar") == bar, "and the in-memory bar is unchanged")

	print("== 3. a hand-edited file reads safe ==")
	var over := ""
	for i in 30:
		over += '{"k": "menu:%d", "t": "P%d"}, ' % [i + 1, i]
	var hand := "[quick_bar]\n\nversion=1\nbar=[%s7, \"stray\", {\"t\": \"no key\"}, {\"k\": \"\", \"t\": \"e\"}, {\"k\": 5, \"t\": \"n\"}, {\"k\": \"menu:1\", \"t\": \"dup\"}]\ndock=[%s]\nshare={\"cartography.NE\": [%s], \"bad key\": [{\"k\": \"a\", \"t\": \"b\"}], \"world.N\": [{\"k\": \"a\", \"t\": \"b\"}]}\nassistant_button=\"yes\"\nextra_key=5\n" % [over, over, over]
	_write_text(path, hand)
	_reload()
	var hb: Array = DccSettings.quick_bar_value("bar")
	_ok(hb.size() == 24, "30 valid + junk pins are cut to the 24 cap: %d" % hb.size())
	_ok(_keys(hb)[0] == "menu:1" and _keys(hb)[23] == "menu:24", "the cut keeps the FIRST 24, in order, and drops the repeat")
	_ok((DccSettings.quick_bar_value("dock") as Array).size() == 4, "dock cut to 4")
	var hs: Dictionary = DccSettings.quick_bar_value("share")
	_ok(hs.size() == 1 and hs.has("cartography.NE") and (hs["cartography.NE"] as Array).size() == 9, "share keeps only the valid key, cut to 9")
	_ok(DccSettings.quick_bar_value("assistant_button") == true, "a wrong-typed flag reads as its default")
	_ok(DccSettings.quick_bar_value("extra_key") == null, "an unknown stored key is not exposed")
	_write_text(path, "[quick_bar]\n\nversion=1\nbar=\"menu:1\"\ndock=5\nshare=[1, 2]\n")
	_reload()
	_ok(DccSettings.quick_bar_value("bar") == [] and DccSettings.quick_bar_value("dock") == [] and DccSettings.quick_bar_value("share") == {}, "wrong-typed lists read as empty, without throwing")
	# malformed entries at the FRONT of the list (a cut at the cap would otherwise hide them)
	_write_text(path, "[quick_bar]

version=1
bar=[{\"k\": \"\", \"t\": \"empty\"}, 7, \"stray\", {\"t\": \"no key\"}, {\"k\": 5, \"t\": \"n\"}, {\"k\": \"menu:1\", \"t\": \"ok\"}, {\"k\": \"menu:1\", \"t\": \"dup\"}, {\"k\": \"menu:2\"}]
")
	_reload()
	_ok(DccSettings.quick_bar_value("bar") == [{"k": "menu:1", "t": "ok"}, {"k": "menu:2", "t": ""}], "malformed entries (empty k, non-dict, no k, non-String k, a repeat) are dropped; a missing title reads as '': %s" % str(DccSettings.quick_bar_value("bar")))
	# a NEWER version: reads empty, is not rewritten by reading, replaced by the first edit.
	var newer := "[quick_bar]\n\nversion=2\nbar=[{\"k\": \"menu:45\", \"t\": \"Vault\"}]\nfuture_key=1\n"
	_write_text(path, newer)
	_reload()
	_ok(DccSettings.quick_bar_value("bar") == [] and DccSettings.quick_bar_value("share") == {}, "a version-2 section reads as empty")
	_ok(not DccSettings.has_quick_bar_value("bar"), "and reports nothing stored")
	_ok(_text(path) == newer.replace("\r\n", "\n"), "reading it did not rewrite the file")
	_write_text(path, "[quick_bar]\n\nversion=\"two\"\nbar=[{\"k\": \"menu:45\", \"t\": \"Vault\"}]\n")
	_reload()
	_ok(DccSettings.quick_bar_value("bar") == [], "a non-int version also reads as empty")
	_write_text(path, newer)
	_reload()
	_ok(DccSettings.set_quick_bar_value("bar", [{"k": "menu:45", "t": "Vault"}]), "the first user edit is accepted")
	_reload()
	_ok(DccSettings.quick_bar_value("version") == 1 and _keys(DccSettings.quick_bar_value("bar")) == ["menu:45"], "it replaced the section: version 1 and the new pin")
	_ok(not _text(path).contains("future_key"), "the newer build's other keys were dropped by that edit (documented)")

	print("== 4. an unknown pin_key survives ==")
	_write_text(path, "")
	_reload()
	_ok(DccSettings.set_quick_bar_value("bar", [{"k": GHOST_KEY, "t": GHOST_TITLE}, {"k": "menu:45", "t": "Markdown vault"}]), "a pin for a command this build lacks is storable")
	_ok(DccSettings.set_quick_bar_value("assistant_button", true), "another key is written")
	_ok(DccSettings.quick_bar_add("dock", "menu:726", "Journey planner") == "", "another list is written")
	_reload()
	var gb: Array = DccSettings.quick_bar_value("bar")
	_ok(_keys(gb) == [GHOST_KEY, "menu:45"] and String(gb[0]["t"]) == GHOST_TITLE, "the ghost pin and its title survive a load/save of other keys")

	print("== 5. clearing the last pin persists ==")
	_ok(DccSettings.set_quick_bar_value("bar", [{"k": "menu:45", "t": "Markdown vault"}]), "one pin")
	_ok(DccSettings.quick_bar_remove("bar", "menu:45"), "remove the last pin")
	_ok(_text(path).contains("bar=[]"), "the file now says bar=[]")
	_reload()
	_ok(DccSettings.quick_bar_value("bar") == [] and DccSettings.has_quick_bar_value("bar"), "after a reload the bar is empty AND stored (not 'never configured')")
	_ok(not DccSettings.quick_bar_remove("bar", "menu:45"), "removing a pin that is not there says false")

	print("== 6. backward compatibility: a real prior document ==")
	var fixture_path := ProjectSettings.globalize_path(FIXTURE)
	var fixture := _text(fixture_path)
	_ok(fixture.length() > 400 and fixture.contains("[handedness]") and fixture.contains("[llm]") and not fixture.contains("quick_bar"),
		"the fixture was written by the pre-CQ-1 setters: %d chars, 15 real sections, no [quick_bar]" % fixture.length())
	_write_text(path, fixture)
	_reload()
	_ok(DccSettings.dominant_hand() == "left" and DccSettings.theme_mode() == "light" and DccSettings.units_mode() == "mi", "handedness, theme and units resolve from the prior file")
	_ok(DccSettings.recent_projects().size() == 2 and DccSettings.autosave_minutes() == 7 and DccSettings.cpu_thread_count() == 6, "recent, autosave and cpu resolve")
	_ok(DccSettings.bake_depth() == 4 and is_equal_approx(DccSettings.atlas_cap_gb(), 12.5) and not DccSettings.landmarks_visible(), "tiles, atlas and landmark layer resolve")
	_ok(DccSettings.layout_names() == ["My layout"] and DccSettings.asset_favourites().size() == 1, "layouts and asset favourites resolve")
	_ok(DccSettings.llm_value("enabled") == true and DccSettings.llm_value("model") == "llama3", "the [llm] section resolves")
	_ok(DccSettings.shortcut_binding(DccSettings.SHORTCUT_CONTEXT_MENU, 45, 0) == 4194326, "a shortcut override resolves")
	_ok(DccSettings.quick_bar_value("bar") == [] and not DccSettings.has_quick_bar_value("bar"), "a file with no [quick_bar] reads as never configured")
	_ok(DccSettings.set_quick_bar_value("bar", [{"k": "menu:45", "t": "Markdown vault"}]) and DccSettings.set_quick_bar_value("share", {"world.NW": _pins(2)}), "quick-bar keys are written into it")
	var after := _text(path)
	var cut := after.find("[quick_bar]")
	_ok(cut > 0, "[quick_bar] was appended after the existing sections")
	_ok(after.substr(0, cut).strip_edges() == fixture.strip_edges(), "everything before [quick_bar] is byte-identical to the fixture (%d chars)" % cut)
	_reload()
	_ok(DccSettings.dominant_hand() == "left" and DccSettings.layout_names() == ["My layout"], "and it still resolves after a reload")

	print("== 7. LEAK: nothing reaches a world, an export or an archive ==")
	await _leak_checks()

	print("== 8. the allow-list ==")
	for k in DccSettings.QUICK_BAR_DEFAULTS:
		_ok(not String(k).contains("secret") and k != "key" and k != "api_key" and k != "token", "allow-list entry '%s' is not a secret slot" % k)
	_ok(DccSettings.QUICK_BAR_DEFAULTS.size() == 5, "five keys: version + four user-settable (literal)")
	_ok(DccSettings.QUICK_BAR_MAX_BAR == 24 and DccSettings.QUICK_BAR_MAX_DOCK == 4 and DccSettings.QUICK_BAR_MAX_SHARE == 9, "caps are 24 / 4 / 9 (Ruling DH Q5, literal)")
	_finish()


## The leak greps over a saved project (every zip entry), an exported PNG and the raw zip.
func _leak_checks() -> void:
	# Pin things whose text is unmistakable, so a hit cannot be a coincidence.
	DccSettings.set_quick_bar_value("bar", [{"k": "menu:45", "t": "QBLEAK Markdown vault"}, {"k": GHOST_KEY, "t": GHOST_TITLE}])
	DccSettings.set_quick_bar_value("dock", [{"k": "menu:726", "t": "QBLEAK Journey planner"}])
	DccSettings.set_quick_bar_value("share", {"cartography.NE": [{"k": "menu:731", "t": "QBLEAK shared"}]})
	var needles := ["quick_bar", "QBLEAK", GHOST_TITLE, GHOST_KEY]
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_bridge = _app.bridge
	var sdir := ProjectSettings.globalize_path("user://_quickbar_store_probe")
	DirAccess.make_dir_recursive_absolute(sdir)
	_bridge.generate({"seed": 9137, "width_km": 1200.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _settle(10)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	_ok(_bridge.has_world, "a world exists to save")
	var zpath := sdir.path_join("leak.zip")
	_ok(_bridge.save_project(zpath), "the project is saved while the pins are stored")
	var zr := ZIPReader.new()
	_ok(zr.open(zpath) == OK, "the saved project opens as a zip")
	var entries := zr.get_files()
	var leaked := PackedStringArray()
	for e in entries:
		var b := zr.read_file(e)
		for n in needles:
			if _contains(b, n):
				leaked.append("%s in %s" % [n, e])
	_ok(entries.size() >= 5 and leaked.is_empty(), "%d zip entries searched for %d needles: %s" % [entries.size(), needles.size(), str(leaked)])
	var raw := _bytes(zpath)
	var raw_leak := PackedStringArray()
	for n in needles:
		if _contains(raw, n):
			raw_leak.append(n)
	_ok(raw.size() > 1000 and raw_leak.is_empty(), "the raw zip (%d bytes) holds none of them: %s" % [raw.size(), str(raw_leak)])
	# Positive control: the same zip with a quick-bar entry appended MUST trip the detector.
	var cpath := sdir.path_join("control.zip")
	var packer := ZIPPacker.new()
	packer.open(cpath)
	for e in entries:
		packer.start_file(e)
		packer.write_file(zr.read_file(e))
		packer.close_file()
	packer.start_file("settings_leak.txt")
	packer.write_file("[quick_bar]\nbar=QBLEAK".to_utf8_buffer())
	packer.close_file()
	packer.close()
	var zc := ZIPReader.new()
	zc.open(cpath)
	var control_hits := 0
	for e in zc.get_files():
		if _contains(zc.read_file(e), "quick_bar"):
			control_hits += 1
	_ok(control_hits == 1, "positive control: a quick_bar entry smuggled into the zip is detected (%d hit)" % control_hits)
	zc.close()
	zr.close()
	# Export a PNG and search it.
	var dlg: AcceptDialog = _app.open_export_dialog()
	await _settle(4)
	dlg.set_destination(sdir.path_join("session.png"))
	dlg.start_export()
	var res: Dictionary = await dlg.export_finished
	_ok(bool(res.get("ok", false)), "a 2048 PNG export ran: %s" % str(res.get("ok", res)))
	dlg.hide()
	var png := _bytes(sdir.path_join("session.png"))
	var png_leak := PackedStringArray()
	for n in needles:
		if _contains(png, n):
			png_leak.append(n)
	_ok(png.size() > 1000 and png_leak.is_empty(), "the exported PNG (%d bytes) holds none of them: %s" % [png.size(), str(png_leak)])
	# And the settings file DOES hold them (the search is over something real).
	_ok(_contains(_bytes(_cfg_path()), "QBLEAK"), "control: the settings file itself holds the pins' text")

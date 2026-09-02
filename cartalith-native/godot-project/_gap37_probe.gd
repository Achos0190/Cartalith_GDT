extends Node
## Committed probe for the §37 backable-items pass.
##
## Drives, windowed, against a real world:
##   WW-14  WORLD ▸ Ecology is a live readout (NPP + ecoregions), the `npp`
##          analysis field draws, and the two jump buttons really switch it.
##   CV-21  a faction identity colour is settable, reaches the territory wash
##          and the Political-control field, and Reset puts it back.
##   CA-17  territory fill opacity moves the wash's alpha.
##   CA-16  way width / way opacity move the drawn ways; the LOD ladder hides
##          minor ways when zoomed out.
##   CV-22  faction notes reach the vault.
##   VA-02  create-a-note-from-template writes a real file.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _gap37_probe.tscn
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _app: Node
var _bridge
var _fail := 0


func _p(s: String) -> void:
	print("GAP37  %s" % s)

func _bad(s: String) -> void:
	_fail += 1
	print("GAP37  FAIL  %s" % s)

func _ok(s: String) -> void:
	print("GAP37  ok    %s" % s)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)

func _all(root: Node) -> Array:
	var out: Array = []
	_walk(root, out)
	return out

func _texts(root: Node) -> String:
	var parts: Array[String] = []
	for n in _all(root):
		if n is Label:
			parts.append((n as Label).text)
		elif n is Button:
			parts.append((n as Button).text)
		elif n is RichTextLabel:
			parts.append((n as RichTextLabel).get_parsed_text())
	return "\n".join(parts)

func _button(root: Node, needle: String) -> Button:
	for n in _all(root):
		if n is Button and needle in (n as Button).text:
			return n
	return null

func _tex_stats(t: Texture2D) -> Dictionary:
	if t == null:
		return {}
	var img := t.get_image()
	var acc := {"n": 0, "r": 0.0, "g": 0.0, "b": 0.0, "a": 0.0}
	var step := maxi(1, img.get_width() / 96)
	for y in range(0, img.get_height(), step):
		for x in range(0, img.get_width(), step):
			var c := img.get_pixel(x, y)
			if c.a <= 0.002:
				continue
			acc.n += 1
			acc.r += c.r; acc.g += c.g; acc.b += c.b; acc.a += c.a
	if acc.n == 0:
		return {"n": 0}
	return {"n": acc.n, "r": acc.r / acc.n, "g": acc.g / acc.n,
		"b": acc.b / acc.n, "a": acc.a / acc.n}


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 420.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

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
	_p("world: %d settlements, %d factions, %d ways" % [
		_bridge.settlements().size(), _bridge.civ_faction_count(),
		_bridge.roads().size()])

	await _ww14()
	await _cv21()
	await _ca17()
	await _ca16()
	await _cv22()
	await _va02()
	await _ww15()

	_check_bindings()
	_p("=== %s ===" % ("PASS" if _fail == 0 else "%d FAILURES" % _fail))
	get_tree().quit(0 if _fail == 0 else 1)


# ------------------------------------------------------------------- WW-14
func _ww14() -> void:
	_p("=== WW-14 : WORLD ▸ Ecology ===")
	if not _bridge._has("ecology_summary"):
		_bad("ecology_summary missing from the loaded extension -- stale .dll?")
		return
	var eco: Dictionary = _bridge.ecology_summary()
	_p("summary: npp_mean=%.1f npp_max=%.1f land=%d regions=%d species=%d" % [
		float(eco.get("npp_mean", -1.0)), float(eco.get("npp_max", -1.0)),
		int(eco.get("land_cells", -1)), int(eco.get("region_count", -1)),
		int(eco.get("species_total", -1))])
	if float(eco.get("npp_mean", 0.0)) <= 0.0:
		_bad("mean NPP is not positive on a land-bearing world")
	if float(eco.get("npp_max", 0.0)) > 3000.001:
		_bad("NPP exceeds the Miami ceiling")
	if int(eco.get("region_count", 0)) <= 0:
		_bad("no ecoregions on a generated world")
	if int(eco.get("species_total", 0)) <= 0:
		_bad("no species records -- fauna distribution is the half WW-14 denied existed")
	if (eco.get("regions", []) as Array).is_empty():
		_bad("regions list empty")

	_app.select_domain_category("world", "Ecology")
	await _frames(8)
	var ws = _app._world_workspace()
	var body := _texts(ws)
	for needle in ["Net primary productivity averages", "ecoregions carrying",
			"Show productivity on the map", "Show fauna on the map"]:
		if needle in body:
			_ok("Ecology renders: %s" % needle)
		else:
			_bad("Ecology is missing: %s" % needle)
	if "do not exist in this port or in the reference" in body:
		_bad("the stale WW-14 denial is still on screen")

	## The `npp` analysis field must be offered, available, and draw.
	var offered := false
	for g in _bridge.debug_layers():
		for it in (g as Dictionary).get("items", []):
			if String((it as Dictionary).get("id", "")) == "npp":
				offered = true
				if not bool((it as Dictionary).get("available", false)):
					_bad("npp is offered but unavailable on a generated world")
				if (it as Dictionary).get("legend", []).is_empty():
					_bad("npp has no legend")
	if not offered:
		_bad("npp is not in debug_layers()")

	var b := _button(ws, "Show productivity on the map")
	if b == null:
		_bad("no productivity jump button")
	else:
		b.pressed.emit()
		await _frames(6)
		if _app.viewport.debug_view() != "npp":
			_bad("productivity button did not set the npp view (got %s)" % _app.viewport.debug_view())
		else:
			var s := _tex_stats(_app.viewport._debug_layer.texture)
			_p("npp raster: %d sampled px, mean rgb (%.2f, %.2f, %.2f)" % [
				int(s.get("n", 0)), float(s.get("r", 0.0)), float(s.get("g", 0.0)), float(s.get("b", 0.0))])
			if int(s.get("n", 0)) == 0:
				_bad("npp raster is empty")
			else:
				_ok("npp raster drew")
	var b2 := _button(ws, "Show fauna on the map")
	if b2 != null:
		b2.pressed.emit()
		await _frames(6)
		if _app.viewport.debug_view() != "wildlife":
			_bad("fauna button did not set the wildlife view")
		else:
			_ok("fauna button set the wildlife view")
	_app.viewport.set_debug_layer("off")
	await _frames(2)


# ------------------------------------------------------------------- CV-21
func _cv21() -> void:
	_p("=== CV-21 : faction identity colour ===")
	if not _bridge._has("civ_set_faction_color"):
		_bad("civ_set_faction_color missing -- stale .dll?")
		return
	var before: Array = _bridge.get_factions()
	if before.is_empty():
		_bad("no factions")
		return
	var f1: Dictionary = before[0]
	var fid := int(f1.get("id", 1))
	_p("faction %d default swatch (%d,%d,%d) custom=%s" % [fid,
		int(f1.get("color_r", -1)), int(f1.get("color_g", -1)), int(f1.get("color_b", -1)),
		str(f1.get("color_custom", null))])
	if bool(f1.get("color_custom", true)):
		_bad("a fresh roster reports a custom colour")

	var wash0 := _tex_stats(_bridge.territory_texture())
	_app.viewport.set_debug_layer("control")
	await _frames(4)
	var ctrl0 := _tex_stats(_app.viewport._debug_layer.texture)
	_p("before: wash mean rgb (%.3f, %.3f, %.3f) a=%.3f over %d px" % [
		float(wash0.get("r", 0.0)), float(wash0.get("g", 0.0)), float(wash0.get("b", 0.0)),
		float(wash0.get("a", 0.0)), int(wash0.get("n", 0))])

	## A colour nothing in either default palette is near: pure magenta.
	if not _bridge.civ_set_faction_color(fid, Color(1.0, 0.0, 1.0)):
		_bad("civ_set_faction_color refused")
	var after: Array = _bridge.get_factions()
	var f2: Dictionary = after[0]
	if not (int(f2.get("color_r", 0)) == 255 and int(f2.get("color_g", 0)) == 0 and int(f2.get("color_b", 0)) == 255):
		_bad("get_factions did not read the new colour back: (%d,%d,%d)" % [
			int(f2.get("color_r", 0)), int(f2.get("color_g", 0)), int(f2.get("color_b", 0))])
	else:
		_ok("get_factions reads the identity colour back")
	if not bool(f2.get("color_custom", false)):
		_bad("color_custom did not flip")

	var wash1 := _tex_stats(_bridge.territory_texture())
	_app.viewport.set_debug_layer("control")
	await _frames(4)
	var ctrl1 := _tex_stats(_app.viewport._debug_layer.texture)
	_p("after:  wash mean rgb (%.3f, %.3f, %.3f)   control mean rgb (%.3f, %.3f, %.3f)" % [
		float(wash1.get("r", 0.0)), float(wash1.get("g", 0.0)), float(wash1.get("b", 0.0)),
		float(ctrl1.get("r", 0.0)), float(ctrl1.get("g", 0.0)), float(ctrl1.get("b", 0.0))])
	if abs(float(wash1.get("b", 0.0)) - float(wash0.get("b", 0.0))) < 0.01:
		_bad("the territory wash did not move -- CIVIL's colour has no renderer")
	else:
		_ok("the territory wash moved with the identity colour")
	if abs(float(ctrl1.get("b", 0.0)) - float(ctrl0.get("b", 0.0))) < 0.01:
		_bad("the Political-control field did not move")
	else:
		_ok("the Political-control field moved too")

	## Through the real window, not just the binding.
	_app.faction_roster_window.open()
	await _frames(10)
	var pickers: Array = []
	for n in _all(_app.faction_roster_window):
		if n is ColorPickerButton:
			pickers.append(n)
	if pickers.is_empty():
		_bad("the roster inspector has no colour picker")
	else:
		_ok("roster inspector has a colour picker, showing %s" % str((pickers[0] as ColorPickerButton).color))
		(pickers[0] as ColorPickerButton).color_changed.emit(Color(0.0, 1.0, 0.0))
		await _frames(6)
		var g: Array = _bridge.get_factions()
		if int((g[_app.faction_roster_window._selected - 1] as Dictionary).get("color_g", 0)) != 255:
			_bad("the picker did not write through")
		else:
			_ok("the picker writes through to the engine")
	var reset := _button(_app.faction_roster_window, "Reset")
	if reset == null:
		_bad("no Reset row")
	elif reset.disabled:
		_bad("Reset is dead on a faction that has a custom colour")
	else:
		reset.pressed.emit()
		await _frames(8)
		var back: Array = _bridge.get_factions()
		var fb: Dictionary = back[_app.faction_roster_window._selected - 1]
		if bool(fb.get("color_custom", true)):
			_bad("Reset did not clear the override")
		else:
			_ok("Reset returns the faction to the palette rule")
	_app.faction_roster_window.hide()
	_bridge.civ_clear_faction_color(fid)
	_app.viewport.set_debug_layer("off")
	await _frames(4)


# ------------------------------------------------------------------- CA-17
func _ca17() -> void:
	_p("=== CA-17 : territory fill opacity ===")
	if not _bridge._has("set_territory_opacity"):
		_p("skipped -- not built in this pass")
		return
	var a0 := _tex_stats(_bridge.territory_texture())
	_bridge.set_territory_opacity(1.0)
	var a1 := _tex_stats(_bridge.territory_texture())
	_bridge.set_territory_opacity(0.1)
	var a2 := _tex_stats(_bridge.territory_texture())
	_p("alpha at default=%.3f, 1.00=%.3f, 0.10=%.3f" % [
		float(a0.get("a", 0.0)), float(a1.get("a", 0.0)), float(a2.get("a", 0.0))])
	if not (float(a2.get("a", 0.0)) < float(a0.get("a", 0.0)) and float(a0.get("a", 0.0)) < float(a1.get("a", 0.0))):
		_bad("territory opacity is not monotone in the slider")
	else:
		_ok("territory opacity moves the wash")
	_bridge.set_territory_opacity(-1.0)


# ------------------------------------------------------------------- CA-16
func _ca16() -> void:
	_p("=== CA-16 / CA-18 : way width, opacity, LOD ladder ===")
	var ov = _app.viewport.overlay
	if not ov.has_method("set_way_scale"):
		_bad("map_overlay has no set_way_scale -- CA-16 not built")
		return
	_p("defaults: scale=%.2f opacity=%.2f lod=%s" % [ov.way_scale(), ov.way_opacity(), str(ov.way_lod())])
	if not is_equal_approx(ov.way_scale(), 1.0) or not is_equal_approx(ov.way_opacity(), 1.0):
		_bad("the defaults are not the reference's identity pair")

	## Pixel evidence: the map with ways at 2.5x is not the map with ways off.
	var base := await _shot()
	ov.set_way_opacity(0.0)
	await _frames(6)
	var off := await _shot()
	ov.set_way_opacity(1.0)
	ov.set_way_scale(2.5)
	await _frames(6)
	var fat := await _shot()
	var d_off := _diff(base, off)
	var d_fat := _diff(base, fat)
	_p("pixels differing from the default frame: opacity 0 -> %.3f%%, width 2.5x -> %.3f%%" % [d_off * 100.0, d_fat * 100.0])
	if d_off <= 0.0001:
		_bad("way opacity 0 changed nothing on screen")
	else:
		_ok("way opacity moves real pixels")
	if d_fat <= 0.0001:
		_bad("way width 2.5x changed nothing on screen")
	else:
		_ok("way width moves real pixels")
	ov.set_way_scale(1.0)
	await _frames(4)

	## The LOD ladder: `track`/`ancient` below 0.7, nothing else.
	var by_type := {}
	for w in _bridge.roads():
		var k := String((w as Dictionary).get("way_type", "?"))
		by_type[k] = int(by_type.get(k, 0)) + 1
	_p("way types in this world: %s" % str(by_type))
	var minor := 0
	for k in by_type:
		if float(ov.WAY_LOD_MIN.get(k, ov.WAY_LOD_DEFAULT)) > 0.4:
			minor += int(by_type[k])
	_p("ways the ladder drops below 0.7x zoom: %d of %d" % [minor, _bridge.roads().size()])
	_app.viewport.zoom_step(0.5 / _app.viewport.zoom())
	await _frames(6)
	var out_on := await _shot()
	ov.set_way_lod(false)
	await _frames(6)
	var out_off := await _shot()
	ov.set_way_lod(true)
	_app.viewport.zoom_step(1.0 / _app.viewport.zoom())
	await _frames(6)
	var d_lod := _diff(out_on, out_off)
	_p("zoomed to 0.5x, ladder on vs off: %.4f%% of sampled pixels differ (one track of 35 ways)" % [d_lod * 100.0])
	if minor > 0 and d_lod <= 0.0:
		_bad("the LOD ladder drops %d ways but the screen is identical" % minor)
	elif minor > 0:
		_ok("the LOD ladder is visible")
	else:
		_p("(this world has no track/ancient ways, so the ladder has nothing to drop)")


func _shot() -> Image:
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()


## Fraction of sampled pixels that differ by more than one level.
func _diff(a: Image, b: Image) -> float:
	if a == null or b == null or a.get_width() != b.get_width():
		return -1.0
	var n := 0
	var hit := 0
	var step := maxi(1, a.get_width() / 200)
	for y in range(0, a.get_height(), step):
		for x in range(0, a.get_width(), step):
			n += 1
			var ca := a.get_pixel(x, y)
			var cb := b.get_pixel(x, y)
			if absf(ca.r - cb.r) > 0.004 or absf(ca.g - cb.g) > 0.004 or absf(ca.b - cb.b) > 0.004:
				hit += 1
	return 0.0 if n == 0 else float(hit) / float(n)


# ------------------------------------------------------------------- CV-22
func _cv22() -> void:
	_p("=== CV-22 : faction notes in the vault ===")
	if not _bridge._has("vault_entity_kinds"):
		_bad("vault_entity_kinds missing -- stale .dll?")
		return
	var kinds: PackedStringArray = _bridge.vault_entity_kinds()
	_p("entity kinds: %s" % str(kinds))
	if not ("faction" in kinds):
		_bad("faction is not an addressable vault entity")
		return
	_ok("faction is an addressable vault entity")

	var fields: Array = _bridge.vault_export_fields("faction", 1)
	var keys: Array[String] = []
	for f in fields:
		keys.append(String((f as Dictionary).get("key", "")))
	_p("offered fields for faction 1: %s" % str(keys))
	for want in ["name", "entity_type", "culture", "government", "population", "settlements", "area"]:
		if want in keys:
			_ok("offers %s" % want)
		else:
			_bad("faction 1 does not offer %s" % want)

	var vals: Dictionary = _bridge.vault_entity_values("faction", 1)
	_p("values: %s" % str(vals))
	if String(vals.get("entity_type", "")) != "faction":
		_bad("entity_type is not faction")
	if String(vals.get("name", "")).is_empty():
		_bad("no faction name")
	var body: String = _bridge.vault_block_body("faction", 1, PackedStringArray(keys))
	_p("block body (%d chars):
%s" % [body.length(), body])
	if body.length() < 20:
		_bad("the machine block for a faction is empty")
	else:
		_ok("a faction produces a real Cartalith block")

	## Unknown ids must be refused, not defaulted.
	if not _bridge.vault_entity_values("faction", 0).is_empty():
		_bad("faction 0 (Unclaimed) returned values")
	if not _bridge.vault_entity_values("faction", 999).is_empty():
		_bad("a nonexistent faction returned values")

	## And the dock row is drawn.
	_app.select_domain_category("civilization", "Factions")
	await _frames(8)
	var t := _texts(_app).to_lower()
	if "linked notes" in t and "history, notes and lore" in t:
		_ok("CIVIL ▸ Factions draws a Linked notes section")
	else:
		_bad("CIVIL ▸ Factions has no Linked notes section")
	if "is not an addressable entity there yet" in t:
		_bad("the stale CV-22 denial is still on screen")
	if "identity colour" in t:
		_ok("CIVIL ▸ Factions draws the identity-colour block (CV-21)")
	else:
		_bad("CIVIL ▸ Factions has no identity-colour block")


# ------------------------------------------------------------------- VA-02
func _va02() -> void:
	_p("=== VA-02 : create a note from a template ===")
	if not _bridge._has("vault_create_from_template"):
		_bad("vault_create_from_template missing -- stale .dll?")
		return
	## A real folder with the owner's own template shape in it.
	var root := OS.get_user_data_dir().path_join("_gap37_vault")
	var da := DirAccess.open(OS.get_user_data_dir())
	if DirAccess.dir_exists_absolute(root):
		_rmtree(root)
	DirAccess.make_dir_recursive_absolute(root)
	var f := FileAccess.open(root.path_join("Settlement Template.md"), FileAccess.WRITE)
	f.store_string("## Settlement Profile: [Name]

**Former Names:** [If applicable]

### History

[Key events.]
")
	f.close()
	var r: Dictionary = _bridge.vault_connect(root, "Gap37")
	if not bool(r.get("ok", false)):
		_bad("could not connect the probe vault: %s" % str(r))
		return
	_ok("connected a probe vault at %s" % root)

	var ts: Array = _bridge.vault_templates()
	_p("templates found: %s" % str(ts))
	if ts.is_empty():
		_bad("the template was not discovered")
		return
	_ok("template discovered: %s" % String((ts[0] as Dictionary).get("label", "")))

	var s0: Dictionary = _bridge.settlements()[0]
	var sname := String(s0.get("name", "Place"))
	var path: String = _bridge.vault_suggested_path("settlement", sname)
	_p("suggested path for %s: %s" % [sname, path])
	if not path.begins_with("Settlements/"):
		_bad("the path convention is not Settlements/{name}.md")

	var made: Dictionary = _bridge.vault_create_from_template(String((ts[0] as Dictionary).get("rel", "")), path, sname)
	if not bool(made.get("ok", false)):
		_bad("create refused: %s" % String(made.get("error", "")))
		return
	var text := String(made.get("text", ""))
	_p("created %s (%d chars)" % [String(made.get("path", "")), text.length()])
	if not text.begins_with("## Settlement Profile: %s" % sname):
		_bad("the name was not substituted")
	else:
		_ok("the name was substituted")
	if not ("[If applicable]" in text):
		_bad("the author's own prompt was rewritten")
	else:
		_ok("the author's prompts survive verbatim")
	var on_disk: String = FileAccess.get_file_as_string(root.path_join(path))
	if on_disk != text:
		_bad("what was returned is not what is on disk")
	else:
		_ok("the file is really on disk, byte for byte")

	var again: Dictionary = _bridge.vault_create_from_template(String((ts[0] as Dictionary).get("rel", "")), path, "Someone Else")
	if bool(again.get("ok", false)):
		_bad("an existing note was overwritten")
	else:
		_ok("an existing note is refused: %s" % String(again.get("error", "")))
	if FileAccess.get_file_as_string(root.path_join(path)) != text:
		_bad("the refused write still changed the file")

	## And through the window, on a real entity.
	_app.open_vault("settlement", int(s0.get("tid", 1)), sname)
	await _frames(10)
	var wt: String = _texts(_app.vault_window)
	if "new note from a template" in wt.to_lower():
		_ok("the vault window draws the create block")
	else:
		_bad("the vault window has no create block")
	_app.vault_window.hide()
	_bridge.vault_disconnect()
	_rmtree(root)


# ------------------------------------------------------------------- WW-15
func _ww15() -> void:
	_p("=== WW-15 : coordinate system ===")
	if not _bridge._has("world_crs"):
		_bad("world_crs missing -- stale .dll?")
		return
	var crs: Dictionary = _bridge.world_crs()
	_p("crs: %s" % str(crs))
	if crs.is_empty():
		_bad("no CRS record on a generated world")
		return
	for k in ["world", "frame", "lat_n", "lat_s", "cell_km", "deg_per_row", "export_note"]:
		if not crs.has(k):
			_bad("crs is missing %s" % k)
	if float(crs.get("cell_km", 0.0)) <= 0.0:
		_bad("cell size is not positive")
	if float(crs.get("deg_per_row", 0.0)) <= 0.0:
		_bad("degrees-per-row is not positive")
	if String(crs.get("export_note", "")).is_empty():
		_bad("the export declares no coordinate note")
	else:
		_ok("the export's own CRS note is readable in-app")
	_app.select_domain_category("world", "World data")
	await _frames(8)
	var wtxt := _texts(_app._world_workspace()).to_lower()
	if "coordinate system" in wtxt and "km on a side" in wtxt:
		_ok("WORLD ▸ World data draws the coordinate frame")
	else:
		_bad("WORLD ▸ World data has no coordinate-frame readout")
	if "are not modelled" in wtxt:
		_bad("the stale WW-15 denial is still on screen")


func _rmtree(path: String) -> void:
	var d := DirAccess.open(path)
	if d == null:
		return
	d.list_dir_begin()
	var n := d.get_next()
	while n != "":
		if d.current_is_dir():
			_rmtree(path.path_join(n))
		else:
			DirAccess.remove_absolute(path.path_join(n))
		n = d.get_next()
	d.list_dir_end()
	DirAccess.remove_absolute(path)


## The staleness fingerprint, read off the shell instead of guessed at.
##
## `EngineBridge._has()` (`shell/engine_bridge.gd`) is the one choke point
## every binding guard in the shell goes through, and it records the name of
## each method the shell asked for that this build does not export;
## `EngineBridge.missing_bindings()` hands back the set. Nothing in this probe
## suite read it -- and a stale `target/debug/cartalith_godot.dll` has twice
## sent every `_has()` guard in a run down its degraded-fallback branch, which
## turns a whole sweep into a clean report over code that was never exercised.
## That is the failure mode this suite is least able to notice on its own, and
## the shell was already carrying the answer.
##
## Called last, after every surface this run drives has been driven: the set
## only fills as guards are reached, so an early read reports an empty one.
func _check_bindings() -> void:
	var mb: PackedStringArray = _bridge.missing_bindings()
	if mb.is_empty():
		return
	_bad("stale extension -- the shell asked for %d binding(s) this build "
		% mb.size()
		+ "does not export (%s). " % ", ".join(mb)
		+ "Every result above was measured against a degraded shell; rebuild "
		+ "the crates and re-run before believing any of it.")

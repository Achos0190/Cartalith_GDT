extends Node
## Protects: ET-F3 (`EDITING_TOOLS_SCOPE.md` section 7) -- the style-preset VARIANT recorded
## in the look history, the project archive (`appearance_style.json`, caller-owned, registered
## in `cartalith_io::DOCUMENT_SLOTS`) and a saved look (the file's top-level `style` member),
## each OMITTED when there is no variant. Through the real shell on a real generated world:
##
##   A. RULE (pure): `style_record()` is `{}` for Custom, no variant, an unknown variant and an
##      out-of-range index, and the two literal names otherwise; `preset_index()` inverts it.
##   B. HISTORY: Antique base -> Antique Dark is one entry whose snapshots carry the `preset`
##      data part; Undo restores the map texture BYTE FOR BYTE and every adjustment stack's
##      saved-form text, relights the base tile with no chip and puts the base overlay inks
##      back; Redo restores the variant's bytes and chip; Dark -> Light then Undo is Dark again.
##   C. SAVE -> OPEN: a project saved with Antique Dark lit carries `appearance_style.json` =
##      `{"preset":"Antique","variant":"dark"}`; opened over a different preset, it relights
##      Antique + the Dark chip, the readout and the variant's overlay inks, and the map texture
##      is the saved one byte for byte.
##   D. NO VARIANT: a project saved on a base preset has no such entry; opening it while a
##      variant is lit drops the variant (so the next save cannot write one into it). With
##      `--oldzip PATH` (an archive written by the PRE-ET-F3 tree, `--identity` below), that real
##      prior-format file opens with no style note and no variant, and re-saving it writes the
##      same entry set with the same bytes (every entry but `preview.png`).
##   E. UNKNOWN NAMES: `{"preset":"Antique","variant":"sepia"}` in an archive opens with no note
##      as the base preset; an unknown preset or unreadable text is a note, never an error.
##   F. SAVED LOOK: a look saved on Antique Dark carries `style` = the two literal names, one saved
##      on a base preset carries exactly the four pre-ET-F3 keys; loading the styled one relights
##      Antique Dark with its inks, the plain one is Custom (as before), a `sepia` one is the base.
##      The map texture after a look load is PRINTED, not asserted: a saved look does not
##      reproduce its texture even without a variant (a pre-existing gap; see the leg).
##
## `--identity` instead runs one leg only, for the byte-identity comparison ACROSS TREES: it
## generates the world, saves an untouched project, applies Antique (base) and saves again, and
## writes each archive's entry list with SHA-256s to `--out`. Run it on the pre-change and the
## post-change tree and diff the two lists.
##
## MUST run WINDOWED, from a SCRATCH COPY of the project with its own `config/name` (a run
## writes projects and looks under that copy's own user dir):
##
##   Godot_v4.7.x_win64_console.exe --path <copy> res://_etf3_probe.tscn -- --out DIR [--oldzip P]
##   Godot_v4.7.x_win64_console.exe --path <copy> res://_etf3_probe.tscn -- --out DIR --identity
##
## Arguments: --out DIR (required), --seed N (483920), --grid WxH (512x328), --oldzip PATH,
## --identity. An unknown argument refuses to run.
## Exit 0 = every check held; 1 = a check failed; 2 = could not run (headless, no world).
## Result: an assertion, windowed only. Command per the header: `res://_etf3_probe.tscn -- --out DIR [--oldzip P]` or `--
## --out DIR --identity` (`--out` required; `--seed N` default 483920, `--grid WxH` default 512x328, `--oldzip PATH`,
## `--identity`; an unknown argument refuses to run). Prints `ETF3  ok    <what>`, `PROBE-FAIL ETF3  <what>` per miss,
## `ETF3  note` / `ETF3  info` lines and `ETF3  RESULT checks=<n> fail=<n>`; stderr carries `PROBE-CANNOT-RUN: ...`
## (headless, bad argument, no world, no RenderWorkspace, 1200 s watchdog). `--identity` writes `entries.txt` (each
## archive entry with its SHA-256) into `--out` for a comparison across trees. Exit 0 = every check held, 1 = a check
## failed, 2 = could not run.

const RW := preload("res://shell/workspaces/render_workspace.gd")
const SLOT := "appearance_style.json"

var GRID := Vector2i(512, 328)
var _out := ""
var _seed := 483920
var _oldzip := ""
var _identity := false
var _fail := 0
var _checks := 0
var _app: Node
var _vh: Control
var _br: Node
var _ov: Control
var _rw: Node
var _tiles: Array = []


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if cond:
		print("ETF3  ok    " + what)
	else:
		_fail += 1
		print("PROBE-FAIL ETF3  " + what)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null


func _parse_args() -> bool:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out":
				_out = args[i + 1]; i += 1
			"--seed":
				_seed = int(args[i + 1]); i += 1
			"--grid":
				var p := args[i + 1].split("x"); GRID = Vector2i(int(p[0]), int(p[1])); i += 1
			"--oldzip":
				_oldzip = args[i + 1]; i += 1
			"--identity":
				_identity = true
			_:
				printerr("PROBE-CANNOT-RUN: unknown argument %s" % args[i])
				return false
		i += 1
	if _out == "":
		printerr("PROBE-CANNOT-RUN: --out DIR is required")
		return false
	DirAccess.make_dir_recursive_absolute(_out)
	return true


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- run windowed.")
		get_tree().quit(2)
		return
	if not _parse_args():
		get_tree().quit(2)
		return
	get_tree().create_timer(1200.0).timeout.connect(func() -> void:
		printerr("PROBE-CANNOT-RUN: watchdog"); get_tree().quit(2))
	DisplayServer.window_set_size(Vector2i(1600, 1000))
	await _frames(2)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	_ov = _vh.overlay
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	_br.generate({"seed": _seed, "width_km": 1200.0, "grid_w": GRID.x, "grid_h": GRID.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	if _br.generating or _br.settlements().is_empty():
		printerr("PROBE-CANNOT-RUN: no generated world")
		get_tree().quit(2)
		return
	_app.select_domain_category("cartography", "Style")
	await _frames(8)
	_rw = _find(_app, "render_workspace.gd")
	if _rw == null:
		printerr("PROBE-CANNOT-RUN: no RenderWorkspace")
		get_tree().quit(2)
		return
	_tiles = _rw.get("_preset_tiles")
	_vh.reset_view()
	await _settle()
	if _identity:
		await _leg_identity()
	else:
		_leg_rule()
		await _leg_history()
		await _leg_save_open()
		await _leg_no_variant()
		await _leg_unknown()
		await _leg_saved_look()
	_finish()


func _settle() -> void:
	await _frames(8)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.3).timeout
	await _frames(4)


func _press_base(i: int) -> void:
	(_tiles[i]["button"] as Button).emit_signal("pressed")
	await _settle()


func _press_chip(i: int, v: String) -> void:
	(_tiles[i]["chips"][v] as Button).emit_signal("pressed")
	await _settle()


func _idx(n: String) -> int:
	for i in RW.STYLE_PRESETS.size():
		if String(RW.STYLE_PRESETS[i][0]) == n:
			return i
	return -1


## The engine's map texture, RGB8 bytes (`_etf1_probe.gd`'s measure).
func _tex_bytes() -> PackedByteArray:
	var t: Texture2D = _vh.map_view.texture
	if t == null:
		return PackedByteArray()
	var im := t.get_image()
	im.convert(Image.FORMAT_RGB8)
	return im.get_data()


## Every adjustment stack's saved-form text: Terrain and Water from the engine, the
## labels/overlays one from `overlay_adjust.gd` -- what "stack text" means in the acceptance.
func _stacks() -> String:
	var out := []
	for layer in ["terrain", "water"]:
		var g: Dictionary = _br.adjust_get(layer)
		out.append(String(g.get("stack", "<unreadable>")))
	out.append(String(preload("res://shell/overlay_adjust.gd").get_stack_json()))
	return " | ".join(out)


## A chip is lit when `DccWidgets.set_segment_on()` gave it the accent ink (`_etf2_probe.gd`).
func _chip_lit(b: Button) -> bool:
	return b.get_theme_color("font_color") == DccTheme.c("accent")


## The gallery state: [active index, variant, readout, overlay treatment == expected].
func _state_is(i: int, v: String) -> bool:
	var name := String(RW.STYLE_PRESETS[i][0])
	var shown := name if v == "" else "%s (%s)" % [name, String(RW.VARIANT_LABEL[v])]
	var chips: Dictionary = _tiles[i]["chips"]
	var lit_ok := true
	for cv in chips:
		lit_ok = lit_ok and (_chip_lit(chips[cv]) == (String(cv) == v))
	return int(_rw.get("_active_preset")) == i and String(_rw.get("_active_variant")) == v \
		and String(_vh._style_readout) == shown and lit_ok \
		and _ov.style_treatment() == RW.variant_treatment_for(name, v)


func _zip_entries(path: String) -> Dictionary:
	var z := ZIPReader.new()
	var out := {}
	if z.open(path) != OK:
		return out
	for f in z.get_files():
		out[f] = z.read_file(f)
	z.close()
	return out


func _save(path: String) -> bool:
	if FileAccess.file_exists(path):
		DirAccess.remove_absolute(path)
	_app._write_project(path)
	await _frames(2)
	return FileAccess.file_exists(path)


func _open(path: String) -> bool:
	var ok: bool = _app._load_project(path)
	await get_tree().create_timer(0.5).timeout
	await _settle()
	return ok


# -- A ---------------------------------------------------------------------------------

func _leg_rule() -> void:
	var a := _idx("Antique")
	_ok(a >= 0, "A: the gallery has an Antique tile")
	_ok(_eq(_callm(_rw, "style_record", [-1, "dark"]), {}), "A: Custom records nothing")
	_ok(_eq(_callm(_rw, "style_record", [a, ""]), {}), "A: a base preset records nothing")
	_ok(_eq(_callm(_rw, "style_record", [a, "sepia"]), {}), "A: an unknown variant records nothing")
	_ok(_eq(_callm(_rw, "style_record", [RW.STYLE_PRESETS.size(), "dark"]), {}), "A: an out-of-range index records nothing")
	_ok(_eq(_callm(_rw, "style_record", [a, "dark"]), {"preset": "Antique", "variant": "dark"}),
		"A: Antique Dark records the two literal names")
	_ok(_eq(_callm(_rw, "preset_index", ["Antique"]), a) and _eq(_callm(_rw, "preset_index", ["No such style"]), -1),
		"A: preset_index inverts the name; an unknown name is -1")


# -- B ---------------------------------------------------------------------------------

var _t_dark := PackedByteArray()

func _leg_history() -> void:
	var a := _idx("Antique")
	await _press_base(a)
	var t0 := _tex_bytes()
	var s0 := _stacks()
	await _press_chip(a, "dark")
	var t1 := _tex_bytes()
	_t_dark = t1
	_ok(not t0.is_empty() and t1 != t0, "B: positive control -- the Dark chip moved the map texture")
	var lh = _app.look_history
	var e: Dictionary = lh.entries[lh.cursor - 1]
	_ok(Dictionary(e["after"]).get("preset", {}) == {"index": a, "variant": "dark"}
		and Dictionary(e["before"]).get("preset", {}) == {"index": a, "variant": ""},
		"B: the entry's snapshots carry the preset part (before %s, after %s)"
			% [str(Dictionary(e["before"]).get("preset")), str(Dictionary(e["after"]).get("preset"))])
	_app.look_undo()
	await _settle()
	_ok(_tex_bytes() == t0, "B: undo restores the map texture byte for byte")
	_ok(_stacks() == s0, "B: undo leaves every adjustment stack's text as it was")
	_ok(_state_is(a, ""), "B: undo relights Antique with no chip, readout and base inks")
	_app.look_redo()
	await _settle()
	_ok(_tex_bytes() == t1, "B: redo restores the Dark bytes")
	_ok(_state_is(a, "dark"), "B: redo relights the Dark chip, readout and inks")
	await _press_chip(a, "light")
	_ok(_state_is(a, "light"), "B: Dark -> Light")
	_app.look_undo()
	await _settle()
	_ok(_state_is(a, "dark") and _tex_bytes() == t1, "B: undo across Dark -> Light is Dark again, bytes and chip")


# -- C ---------------------------------------------------------------------------------

func _leg_save_open() -> void:
	var a := _idx("Antique")
	_ok(_state_is(a, "dark"), "C: precondition -- Antique Dark lit")
	var p := _out.path_join("styled.zip")
	_ok(await _save(p), "C: the project saved")
	var ents := _zip_entries(p)
	var rec = JSON.parse_string((ents.get(SLOT, PackedByteArray()) as PackedByteArray).get_string_from_utf8()) if ents.has(SLOT) else null
	_ok(rec is Dictionary and rec == {"preset": "Antique", "variant": "dark"},
		"C: the archive's %s is {preset: Antique, variant: dark} (%s)" % [SLOT, str(rec)])
	var ink := _idx("Ink")
	await _press_base(ink)
	_ok(_state_is(ink, ""), "C: moved off to Ink before the open")
	_ok(await _open(p), "C: the project opened")
	_ok(_state_is(a, "dark"), "C: open relights Antique, the Dark chip, readout and the variant's inks")
	_ok(_tex_bytes() == _t_dark, "C: the opened map texture is the saved Dark one byte for byte")


# -- D ---------------------------------------------------------------------------------

func _leg_no_variant() -> void:
	var a := _idx("Antique")
	await _press_base(a)
	var p := _out.path_join("base.zip")
	_ok(await _save(p), "D: the base-preset project saved")
	var ents := _zip_entries(p)
	_ok(not ents.is_empty() and not ents.has(SLOT), "D: a base preset writes no %s (%d entries)" % [SLOT, ents.size()])
	await _press_chip(a, "pastel")
	_ok(_state_is(a, "pastel"), "D: Pastel lit before opening the variant-less project")
	_ok(await _open(p), "D: the variant-less project opened")
	_ok(String(_rw.get("_active_variant")) == "" and String(_callm(_app, "style_document", [])) == "",
		"D: the session's variant was dropped, so the next save writes no slot")
	if _oldzip == "":
		print("ETF3  note  D: no --oldzip given; the real prior-format file leg was skipped")
		return
	var old := _zip_entries(_oldzip)
	_ok(not old.is_empty() and not old.has(SLOT), "D: the pre-ET-F3 archive reads (%d entries, no %s)" % [old.size(), SLOT])
	_ok(await _open(_oldzip), "D: the pre-ET-F3 archive opened")
	_ok(String(_rw.get("_active_variant")) == "" and String(_callm(_app, "style_document", [])) == "",
		"D: ... with no variant")
	var p2 := _out.path_join("old_resaved.zip")
	_ok(await _save(p2), "D: ... and re-saved")
	var back := _zip_entries(p2)
	var same_set := back.keys().size() == old.keys().size()
	var differ := []
	for k in old:
		if not back.has(k):
			same_set = false
		elif k != "preview.png" and back[k] != old[k]:
			differ.append(k)
	_ok(same_set and differ.is_empty(), "D: the re-saved old archive has the same entries with the same bytes (preview.png aside); differing: %s" % str(differ))


# -- E ---------------------------------------------------------------------------------

func _leg_unknown() -> void:
	var a := _idx("Antique")
	var docs: Dictionary = _app._project_documents()
	docs[SLOT] = "{\"preset\":\"Antique\",\"variant\":\"sepia\"}"
	var p := _out.path_join("sepia.zip")
	if FileAccess.file_exists(p):
		DirAccess.remove_absolute(p)
	_ok(bool(_br.save_project(ProjectSettings.globalize_path(p) if p.begins_with("user://") else p, docs)),
		"E: an archive with an unknown variant saved")
	await _press_base(_idx("Ink"))
	_ok(await _open(p), "E: it opened (no refusal: '%s')" % String(_app.last_open_refusal))
	_ok(_state_is(a, ""), "E: an unknown variant ('sepia') is the base preset")
	_ok(String(_callm(_rw, "restore_style_document", ["{\"preset\":\"Antique\",\"variant\":\"sepia\"}"])) == "",
		"E: ... and produces no note")
	_ok(String(_callm(_rw, "restore_style_document", ["{\"preset\":\"No such style\",\"variant\":\"dark\"}"])) != ""
		and String(_callm(_rw, "restore_style_document", ["[1, 2]"])) != "",
		"E: an unknown preset or non-object text is a note, not an error")


# -- F ---------------------------------------------------------------------------------

func _look_file(name: String) -> String:
	return "user://appearance_presets/%s.json" % _br.call("_preset_slug", name)


func _leg_saved_look() -> void:
	var a := _idx("Antique")
	var pick: OptionButton = _rw.get("_preset_pick")
	if pick == null:
		_ok(false, "F: the saved-looks section is built")
		return
	await _press_chip(a, "dark")
	var t_dark := _tex_bytes()
	_rw.get("_preset_name").text = "etf3 styled"
	_rw.call("_on_save_look")
	var styled = JSON.parse_string(FileAccess.get_file_as_string(_look_file("etf3 styled")))
	_ok(styled is Dictionary and Dictionary(styled).get("style") == {"preset": "Antique", "variant": "dark"},
		"F: a look saved on Antique Dark records style {Antique, dark} (%s)" % str(Dictionary(styled).get("style") if styled is Dictionary else null))
	await _press_base(_idx("Ink"))
	var t_ink := _tex_bytes()
	_rw.get("_preset_name").text = "etf3 plain"
	_rw.call("_on_save_look")
	var plain = JSON.parse_string(FileAccess.get_file_as_string(_look_file("etf3 plain")))
	var keys: Array = Dictionary(plain).keys() if plain is Dictionary else []
	keys.sort()
	_ok(keys == ["appearance", "format", "name", "v"], "F: a look saved on a base preset has exactly the four pre-ET-F3 keys (%s)" % str(keys))
	## Unknown variant: the styled file with its variant renamed, written by hand.
	var sep_text := FileAccess.get_file_as_string(_look_file("etf3 styled")).replace("\"variant\": \"dark\"", "\"variant\": \"sepia\"").replace("\"name\": \"etf3 styled\"", "\"name\": \"etf3 sepia\"")
	var f := FileAccess.open(_look_file("etf3 sepia"), FileAccess.WRITE)
	f.store_string(sep_text)
	f.close()
	_ok(sep_text.contains("\"variant\": \"sepia\""), "F: the sepia look file was written")
	_rw.call("_refresh_preset_list")
	await _press_base(_idx("Default"))
	_ok(await _load_look(pick, "etf3 styled"), "F: the styled look loaded")
	_ok(_state_is(a, "dark"), "F: loading the styled look relights Antique Dark, readout and inks")
	## NOT asserted: a saved look does not reproduce its map texture even with no variant
	## (2026-10-12, this probe: Ink base saved then reloaded over Default differs in ~71 % of
	## bytes, the same as the styled look) -- a pre-existing gap in what a saved look carries,
	## outside ET-F3, which records only the names. Printed beside the plain control.
	print("ETF3  info  F: styled look reload vs its save: %d of %d bytes differ" % [_ndiff(_tex_bytes(), t_dark), t_dark.size()])
	_ok(await _load_look(pick, "etf3 plain"), "F: the plain look loaded")
	print("ETF3  info  F: plain (Ink base) look reload vs its save: %d of %d bytes differ" % [_ndiff(_tex_bytes(), t_ink), t_ink.size()])
	_ok(int(_rw.get("_active_preset")) == -1 and String(_vh._style_readout) == "Custom",
		"F: a look with no style loads as Custom, as before ET-F3")
	_ok(await _load_look(pick, "etf3 sepia"), "F: the sepia look loaded")
	_ok(_state_is(a, ""), "F: a look with an unknown variant loads as the base preset")
	for n in ["etf3 styled", "etf3 plain", "etf3 sepia"]:
		DirAccess.remove_absolute(ProjectSettings.globalize_path(_look_file(n)))


func _load_look(pick: OptionButton, name: String) -> bool:
	for i in pick.item_count:
		if pick.get_item_text(i) == name:
			pick.select(i)
			_rw.call("_on_load_look")
			await _settle()
			return true
	return false


# -- identity (run on both trees) --------------------------------------------------------

func _leg_identity() -> void:
	var lines := PackedStringArray()
	var p0 := _out.path_join("untouched.zip")
	_ok(await _save(p0), "I: the untouched project saved")
	await _press_base(_idx("Antique"))
	var p1 := _out.path_join("antique_base.zip")
	_ok(await _save(p1), "I: the Antique (base) project saved")
	for p in [p0, p1]:
		var ents := _zip_entries(p)
		_ok(not ents.is_empty() and not ents.has(SLOT), "I: %s has %d entries and no %s" % [p.get_file(), ents.size(), SLOT])
		var names: Array = ents.keys()
		names.sort()
		for k in names:
			lines.append("%s %s %s" % [p.get_file(), k, ("size=%d" % (ents[k] as PackedByteArray).size()) if k == "preview.png" else _sha(ents[k])])
		lines.append("%s FILE %s" % [p.get_file(), FileAccess.get_sha256(p)])
	var f := FileAccess.open(_out.path_join("entries.txt"), FileAccess.WRITE)
	f.store_string("\n".join(lines) + "\n")
	f.close()


## Count of differing bytes (size mismatch counts every byte of the longer).
func _ndiff(a: PackedByteArray, b: PackedByteArray) -> int:
	if a.size() != b.size():
		return maxi(a.size(), b.size())
	var n := 0
	for i in a.size():
		if a[i] != b[i]:
			n += 1
	return n


func _sha(b: PackedByteArray) -> String:
	var h := HashingContext.new()
	h.start(HashingContext.HASH_SHA256)
	h.update(b)
	return h.finish().hex_encode()


## `obj.callv(name, args)` when `obj` has the method, else `"<missing:name>"` -- so on the
## PRE-ET-F3 tree (where these methods do not exist) the checks FAIL cleanly instead of a
## script error halting the leg.
## `a == b` only when both are the same Variant type (GDScript refuses `==` across some
## type pairs, and a `"<missing:...>"` from `_callm` must read as a plain failure).
func _eq(a: Variant, b: Variant) -> bool:
	return typeof(a) == typeof(b) and a == b


func _callm(obj: Object, name: String, args: Array) -> Variant:
	if obj == null or not obj.has_method(name):
		return "<missing:%s>" % name
	return obj.callv(name, args)


func _finish() -> void:
	var code := 1 if _fail > 0 else 0
	print("ETF3  RESULT checks=%d fail=%d" % [_checks, _fail])
	get_tree().quit(code)

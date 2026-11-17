extends AcceptDialog

## Records ▸ Name styles… -- Ruling FZ (owner, 2026-11-17), name styles from CSV.
##
## The owner asked for `Data > Name styles…` (the Data menu is `Records` since Ruling DY, which kept its Technical
## band; this row sits in it beside `World data tables…`). One window, three forms (desktop, tablet, phone), one
## builder: wide windows lay the styles list and the selected style's detail side by side, a phone stacks them in
## one scrolling column (the same arrangement `saved_journeys_window.gd` and `settlements_window` take).
##
## **What it holds** (`cartalith_civ::name_style` is the format; `name_style_bridge.rs` the engine side):
##  * a list of styles: the seven built-in cultures (read-only) and the user's imported ones
##    (`name_style_library.gd`, per device, usable in every world);
##  * a drop zone and **Import CSV...** (Android raises the document picker through `DccBrowseDialog.choose_file`),
##    **Export template...** (any built-in culture in the exact import format) and **Export .csv** (a library style);
##  * a **preview** per section with a reseedable sample, drawn by the engine from the same code that names the map;
##  * a **check panel** listing every row error of the last import by line number (a bad row is reported and
##    skipped, never repaired);
##  * **assignment** as the world default and per faction (the faction roster's culture picker lists the library too);
##  * **Duplicate** and **Delete** (Delete removes the library file only: a world that used the style keeps its copy);
##  * **Rename all**: one undoable step that renames the generated names with the chosen style and **never** a
##    hand-given one (`PlaceExtras::name_given`). A world saved before that record existed has no name it may assume
##    generated, so Rename all says so and waits for the user's tick.
##
## **What it must never do**: call `WorldGen` while `bridge.generating`, rename without the confirmation that says how
## many places and how many hand-given names it will leave, or write a style into a world other than the open one.
## Preloaded by `app.gd` (`NAME_STYLES_WINDOW`) rather than given a `class_name` (no `--import` pass: CLAUDE.md).

const NameStyleLibrary := preload("res://shell/name_style_library.gd")

## The shell (`DccApp`) and the engine bridge.
var _host
var _bridge: EngineBridge
var _phone := false
var _body: VBoxContainer
var _status: Label
## Library entries (`name_style_library.gd::entries`), refilled by `_refresh()`.
var _entries: Array = []
## The selected style's key: a built-in culture key or a `style:` key.
var _selected := ""
## Section being previewed and the sample's seed (Reseed counts it up).
var _section := "Settlements"
var _seed := 1
## The last import's check: `{source, ok, issues}`; empty before any import.
var _check: Dictionary = {}
## Ticked by the user to let Rename all treat the names of an older save as generated.
var _include_unknown := false
## The faction the assign row targets (roster id).
var _faction := 1
## Section keys and labels, read once from the engine.
var _sections: Array = []
var _file_drop_wired := false

func setup(host, bridge: EngineBridge) -> void:
	_host = host
	_bridge = bridge
	title = "Name styles"
	size = Vector2i(1060, 700)
	min_size = Vector2i(640, 420)
	_phone = DccWidgets.phone_window(self, host)
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)
	if _phone:
		DccWidgets.phone_head(outer, "Name styles", "import, preview and assign naming styles")
	else:
		var head_row := HBoxContainer.new()
		head_row.add_theme_constant_override("separation", 12)
		DccWidgets.pad(outer, 12, 6, 12, 6).add_child(head_row)
		head_row.add_child(DccTheme.mono_label("→ NAME STYLES", "accent", DccTheme.FS_HEADER, 2, true))
		head_row.add_child(DccTheme.label("Records ▸ Name styles", "text_ghost", DccTheme.FS_TINY))
		head_row.add_child(DccTheme.spacer())
		head_row.add_child(DccTheme.label("a CSV per style · weights · a section per kind of name", "text_ghost", DccTheme.FS_TINY))
		outer.add_child(DccTheme.rule())
	_status = DccTheme.mono_label("", "text_dim", DccTheme.FS_SMALL, 0)
	_status.name = "NsStatus"
	_status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	DccWidgets.pad(outer, 14, 8, 14, 4).add_child(_status)
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(scroll)
	var pad := MarginContainer.new()
	for side in ["left", "right", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 14)
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(pad)
	_body = VBoxContainer.new()
	_body.name = "NsBody"
	_body.add_theme_constant_override("separation", 10)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_body)
	bridge.generation_finished.connect(func(_ok: bool): if visible: _refresh())
	bridge.world_loaded.connect(func(): if visible: _refresh())

func open() -> void:
	if not _file_drop_wired:
		_file_drop_wired = true
		## An OS file drop reaches `Window.files_dropped`, never a Control (`asset_library_window.gd`'s own note): this
		## window and the root both listen, because which one Godot picks depends on the embedding.
		files_dropped.connect(_on_files_dropped)
		get_tree().root.files_dropped.connect(func(f: PackedStringArray): if visible: _on_files_dropped(f))
	_refresh()
	if not DccWidgets.phone_present(self, _host):
		popup_centered()
	if _phone:
		_host.phone_fit(self, 1.0)

func _say(text: String, token: String = "text_dim") -> void:
	_status.text = text
	_status.add_theme_color_override("font_color", DccTheme.c(token))

func _clear(n: Node) -> void:
	for c in n.get_children():
		n.remove_child(c)
		c.queue_free()

## The label of a style key: the library's name, the built-in key capitalised, or the raw key for a style this device
## does not hold (a world's copy).
func _label_of(key: String) -> String:
	for e in _entries:
		if String(e["key"]) == key:
			return String(e["name"])
	return key.capitalize() if not key.begins_with("style:") else "a style not in this library (%s)" % key.trim_prefix("style:").left(6)

func _is_builtin(key: String) -> bool:
	return not key.begins_with("style:")

## Rebuilds the whole window from the engine and the library.
func _refresh() -> void:
	_clear(_body)
	if bool(_bridge.generating):
		_say("Generating")
		DccWidgets.note(_body, "A generation is running; name styles are read when it finishes (the engine object belongs to the worker until then).")
		return
	_entries = NameStyleLibrary.register_all(_bridge)
	if _sections.is_empty():
		_sections = _bridge.ns_sections()
	if _sections.is_empty():
		_say("This build of the engine has no name styles.")
		return
	var builtins: PackedStringArray = _bridge.civ_culture_vocabulary()
	if _selected == "" and builtins.size() > 0:
		_selected = String(builtins[0])
	var split: BoxContainer = VBoxContainer.new() if _phone or size.x < 820 else HBoxContainer.new()
	split.add_theme_constant_override("separation", 18)
	split.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body.add_child(split)
	var left := VBoxContainer.new()
	left.add_theme_constant_override("separation", 8)
	left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left.size_flags_stretch_ratio = 0.8
	split.add_child(left)
	var right := VBoxContainer.new()
	right.add_theme_constant_override("separation", 8)
	right.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right.size_flags_stretch_ratio = 1.2
	split.add_child(right)
	_build_import(left)
	_build_list(left, builtins)
	_build_detail(right)
	if _phone:
		_host.phone_fit(self, 1.0)

# -- import / export ---------------------------------------------------------------------------------------------------

func _build_import(parent: Control) -> void:
	var zone := PanelContainer.new()
	zone.name = "NsDropZone"
	zone.add_theme_stylebox_override("panel", DccTheme.outline("border", "", 1))
	parent.add_child(zone)
	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", 6)
	DccWidgets.pad(zone, 10, 8, 10, 8).add_child(v)
	v.add_child(DccTheme.label("Drop a .csv here, or", "text_dim", DccTheme.FS_SMALL))
	var imp := DccWidgets.action(v, "Import CSV…", _pick_import, true)
	imp.name = "NsImport"
	imp.tooltip_text = "Reads one name style from a CSV file (Name style, Syllables, then a [Section] per kind of name). A row that is wrong is listed below by its line number and skipped; the rest imports."
	## Export template: any built-in culture, in the exact import format, as a worked example.
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	v.add_child(row)
	var pick := OptionButton.new()
	pick.name = "NsTemplateCulture"
	pick.focus_mode = Control.FOCUS_NONE
	pick.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	for k in _bridge.civ_culture_vocabulary():
		pick.add_item(String(k).capitalize())
		pick.set_item_metadata(pick.item_count - 1, String(k))
	if DccTheme.is_tablet() or _phone:
		pick.custom_minimum_size.y = DccTheme.role_px("btn_min_h")
	row.add_child(pick)
	var exp := DccWidgets.action(row, "Export template…", func(): _export_template(String(pick.get_item_metadata(maxi(0, pick.selected)))))
	exp.name = "NsExportTemplate"
	exp.tooltip_text = "Writes the chosen built-in culture in the exact import format: today's syllables in Start, Middle and End, repeats as weights, every section with its built-in templates. Import it unchanged and it names exactly what the culture names."
	_check_panel(parent)

func _pick_import() -> void:
	DccBrowseDialog.choose_file(_host, "Import name style", PackedStringArray(["csv"]),
		DccSettings.storage_root("exports"), "a name-style .csv", func(path: String): _import_path(path))

func _on_files_dropped(files: PackedStringArray) -> void:
	for f in files:
		if String(f).get_extension().to_lower() == "csv":
			_import_path(String(f))
		else:
			_say("%s is not a .csv; only name-style CSV files import." % String(f).get_file(), "accent")

func _import_path(path: String) -> void:
	if not FileAccess.file_exists(path):
		_say("Could not read %s." % path.get_file(), "accent")
		return
	import_text(FileAccess.get_file_as_string(path), path.get_file())

## Parses `text`, adds a good style to the library and selects it; the check panel gets every issue. Public so the
## probes drive the same path a drop or the picker takes.
func import_text(text: String, source: String) -> Dictionary:
	var r: Dictionary = _bridge.ns_parse(text, true)
	var issues: Array = r.get("issues", [])
	var ok := bool(r.get("ok", false))
	_check = {"source": source, "ok": ok, "issues": issues}
	if ok:
		_selected = NameStyleLibrary.add(r)
		var errs := int(r.get("errors", 0))
		_say("Imported %s from %s%s." % [String(r["name"]), source,
			"" if errs == 0 else " (%d row%s skipped, listed below)" % [errs, "" if errs == 1 else "s"]],
			"text_dim" if errs == 0 else "accent")
	else:
		_say("%s was not imported: %s" % [source, String((issues[0] as Dictionary).get("message", "unreadable")) if not issues.is_empty() else "unreadable"], "accent")
	_refresh()
	return r

func _export_template(culture: String) -> void:
	var csv := _bridge.ns_builtin_csv(culture)
	if csv == "":
		_say("No built-in culture called %s." % culture, "accent")
		return
	DccBrowseDialog.choose_save_path(_host, "Export template", "csv", DccSettings.storage_root("exports"),
		"the %s culture as a name-style CSV" % culture.capitalize(), "%s_template.csv" % culture,
		func(path: String): _write_text(path, csv, "Exported the %s template" % culture.capitalize()))

func _export_selected() -> void:
	var csv := _bridge.ns_builtin_csv(_selected) if _is_builtin(_selected) else _csv_of(_selected)
	if csv == "":
		_say("Nothing to export for %s." % _label_of(_selected), "accent")
		return
	DccBrowseDialog.choose_save_path(_host, "Export name style", "csv", DccSettings.storage_root("exports"),
		"%s as a name-style CSV" % _label_of(_selected), "%s.csv" % _label_of(_selected).to_snake_case(),
		func(path: String): _write_text(path, csv, "Exported %s" % _label_of(_selected)))

func _csv_of(key: String) -> String:
	for e in _entries:
		if String(e["key"]) == key:
			return String(e["csv"])
	return _bridge.ns_style_csv(key)

func _write_text(path: String, text: String, said: String) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		_say("Could not write %s." % path, "accent")
		return
	f.store_string(text)
	f.close()
	_say("%s to %s." % [said, path.get_file()])

# -- the check panel -----------------------------------------------------------------------------------------------------

func _check_panel(parent: Control) -> void:
	var box := VBoxContainer.new()
	box.name = "NsCheck"
	box.add_theme_constant_override("separation", 2)
	parent.add_child(box)
	box.add_child(DccTheme.mono_label("→ CHECK", "text_faint", DccTheme.FS_MICRO, 2))
	if _check.is_empty():
		DccWidgets.note(box, "Nothing imported yet. After an import every row that was wrong is listed here with its line number; a skipped row is never repaired.")
		return
	var issues: Array = _check.get("issues", [])
	var errs := 0
	for i in issues:
		if bool((i as Dictionary).get("error", false)):
			errs += 1
	box.add_child(DccTheme.label("%s: %s" % [String(_check.get("source", "")),
		"%d row%s with a problem" % [errs, "" if errs == 1 else "s"] if errs > 0 else "every row is valid"],
		"accent" if errs > 0 else "text", DccTheme.FS_SMALL))
	for i in issues:
		var d: Dictionary = i
		var line := int(d.get("line", 0))
		var l := DccTheme.mono_label("%s %s" % [("line %d:" % line) if line > 0 else "file:", String(d.get("message", ""))],
			"accent" if bool(d.get("error", false)) else "text_dim", DccTheme.FS_TINY, 0)
		l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		l.custom_minimum_size.x = 200
		l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		box.add_child(l)

# -- the list --------------------------------------------------------------------------------------------------------------

func _build_list(parent: Control, builtins: PackedStringArray) -> void:
	var box := VBoxContainer.new()
	box.name = "NsList"
	box.add_theme_constant_override("separation", 2)
	parent.add_child(box)
	box.add_child(DccTheme.mono_label("→ IMPORTED", "text_faint", DccTheme.FS_MICRO, 2))
	if _entries.is_empty():
		DccWidgets.note(box, "No imported style on this device yet.")
	for e in _entries:
		_style_row(box, String(e["key"]), String(e["name"]), "imported")
	box.add_child(DccTheme.mono_label("→ BUILT IN (read-only)", "text_faint", DccTheme.FS_MICRO, 2))
	for k in builtins:
		_style_row(box, String(k), String(k).capitalize(), "built in")

func _style_row(parent: Control, key: String, nm: String, kind: String) -> void:
	var b := Button.new()
	b.name = "NsRow_%s" % key.replace(":", "_")
	b.text = "%s   · %s" % [nm, kind]
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.focus_mode = Control.FOCUS_NONE
	b.flat = true
	b.toggle_mode = false
	var sel := key == _selected
	b.add_theme_color_override("font_color", DccTheme.c("accent" if sel else "text"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("accent"))
	b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	if DccTheme.is_tablet() or _phone:
		b.custom_minimum_size.y = DccTheme.role_px("row_min_h") if DccTheme.is_tablet() else 44
	b.pressed.connect(func():
		_selected = key
		_refresh())
	parent.add_child(b)

# -- the selected style ------------------------------------------------------------------------------------------------------

func _build_detail(parent: Control) -> void:
	var box := VBoxContainer.new()
	box.name = "NsDetail"
	box.add_theme_constant_override("separation", 8)
	parent.add_child(box)
	if _selected == "":
		return
	var builtin := _is_builtin(_selected)
	box.add_child(DccTheme.mono_label("→ %s" % _label_of(_selected).to_upper(), "accent", DccTheme.FS_HEADER, 1, true))
	box.add_child(DccTheme.label("Built-in culture: read-only. Duplicate it to edit a copy." if builtin else "Imported style, kept on this device for every world.", "text_ghost", DccTheme.FS_TINY))
	var acts := HFlowContainer.new()
	acts.add_theme_constant_override("h_separation", 6)
	acts.add_theme_constant_override("v_separation", 6)
	box.add_child(acts)
	DccWidgets.action(acts, "Export .csv", _export_selected).name = "NsExport"
	var dup := DccWidgets.action(acts, "Duplicate", _duplicate_selected)
	dup.name = "NsDuplicate"
	var del := DccWidgets.action(acts, "Delete", _delete_selected)
	del.name = "NsDelete"
	del.disabled = builtin
	del.tooltip_text = "Built-in cultures cannot be deleted." if builtin else "Removes the style from this device's library. A world that uses it keeps its own copy."
	_build_preview(box)
	_build_assign(box)
	_build_rename(box)

func _duplicate_selected() -> void:
	var csv := _bridge.ns_builtin_csv(_selected) if _is_builtin(_selected) else _csv_of(_selected)
	if csv == "":
		return
	var copy_name := "%s copy" % _label_of(_selected)
	var r: Dictionary = _bridge.ns_parse(NameStyleLibrary.with_name(csv, copy_name), true)
	if not bool(r.get("ok", false)):
		_say("Could not duplicate %s." % _label_of(_selected), "accent")
		return
	_selected = NameStyleLibrary.add(r)
	_say("Duplicated as %s." % copy_name)
	_refresh()

func _delete_selected() -> void:
	if _is_builtin(_selected):
		return
	var nm := _label_of(_selected)
	NameStyleLibrary.remove(_selected)
	_selected = ""
	_say("Deleted %s from this device's library. Worlds that used it keep their copy." % nm)
	_refresh()

## The preview: one section at a time, reseedable. Drawn by the engine from the code that names the map.
func _build_preview(parent: Control) -> void:
	var box := VBoxContainer.new()
	box.name = "NsPreview"
	box.add_theme_constant_override("separation", 4)
	parent.add_child(box)
	box.add_child(DccTheme.mono_label("→ PREVIEW", "text_faint", DccTheme.FS_MICRO, 2))
	var labels: Array = []
	var idx := 0
	for i in _sections.size():
		var s: Dictionary = _sections[i]
		labels.append(String(s["label"]))
		if String(s["key"]) == _section:
			idx = i
	var ob := DccWidgets.choice(box, "Section", labels, idx, func(i: int):
		_section = String((_sections[i] as Dictionary)["key"])
		_refresh(), "Which kind of name to sample. A section the style does not define borrows [Settlements]'s fragments and the built-in templates.")
	ob.name = "NsSection"
	var flow := HFlowContainer.new()
	flow.name = "NsSamples"
	flow.add_theme_constant_override("h_separation", 12)
	flow.add_theme_constant_override("v_separation", 4)
	box.add_child(flow)
	var names: PackedStringArray = _bridge.ns_preview(_selected, _section, _seed, 12)
	for n in names:
		flow.add_child(DccTheme.label(String(n), "text", DccTheme.FS_SMALL))
	if names.is_empty():
		DccWidgets.note(box, "No sample: this build of the engine does not know that style.")
	var re := DccWidgets.action(box, "Reseed", func():
		_seed += 1
		_refresh())
	re.name = "NsReseed"
	re.tooltip_text = "Draws a new sample. The same seed always gives the same names."

## Assignment: the world default, and per faction (the roster's culture picker lists the library too).
func _build_assign(parent: Control) -> void:
	var box := VBoxContainer.new()
	box.name = "NsAssign"
	box.add_theme_constant_override("separation", 4)
	parent.add_child(box)
	box.add_child(DccTheme.mono_label("→ ASSIGN", "text_faint", DccTheme.FS_MICRO, 2))
	var st: Dictionary = _bridge.ns_world_state()
	var has_world := bool(_bridge.has_world)
	var cur := String(st.get("world_default", ""))
	box.add_child(DccTheme.label("World default: %s" % (_label_of(cur) if cur != "" else "none (the built-in cultures)"), "text", DccTheme.FS_SMALL))
	if not has_world:
		DccWidgets.note(box, "No world is open. Generate or open one to assign a style.")
		return
	var row := HFlowContainer.new()
	row.add_theme_constant_override("h_separation", 6)
	row.add_theme_constant_override("v_separation", 6)
	box.add_child(row)
	var use := DccWidgets.action(row, "Use as world default", func():
		if _bridge.ns_set_world_default(_selected):
			_say("%s is the world default: it names landmasses no faction holds, lakes, the world and (when on) rivers, mounts, cliffs and holy sites. It renames nothing already named." % _label_of(_selected))
		else:
			_say("Could not set the default.", "accent")
		_refresh())
	use.name = "NsUseDefault"
	var clr := DccWidgets.action(row, "Clear default", func():
		_bridge.ns_set_world_default("")
		_say("The world has no default style.")
		_refresh())
	clr.name = "NsClearDefault"
	clr.disabled = cur == ""
	var factions: Array = _bridge.get_factions()
	var labels: Array = []
	var ids: Array = []
	var pick_idx := 0
	for f in factions:
		var d: Dictionary = f
		var fid := int(d.get("id", 0))
		if fid < 1:
			continue
		if fid == _faction:
			pick_idx = ids.size()
		ids.append(fid)
		labels.append("%d · %s · %s" % [fid, String(d.get("name", "")), _label_of(String(d.get("culture", "")))])
	if ids.is_empty():
		return
	var fc := DccWidgets.choice(box, "Faction", labels, pick_idx, func(i: int): _faction = int(ids[i]),
		"The faction whose naming culture the next button sets. It names that faction's next settlements, re-rolls and continents; it renames none already placed.")
	fc.name = "NsFaction"
	var arow := HFlowContainer.new()
	arow.add_theme_constant_override("h_separation", 6)
	arow.add_theme_constant_override("v_separation", 6)
	box.add_child(arow)
	var one := DccWidgets.action(arow, "Give to this faction", func():
		var ok: bool = _bridge.ns_assign_faction(_faction, _selected)
		_say("Faction %d now names in %s." % [_faction, _label_of(_selected)] if ok else "Could not assign.", "text_dim" if ok else "accent")
		_refresh())
	one.name = "NsAssignOne"
	var all := DccWidgets.action(arow, "Give to every faction", func():
		var n := 0
		for fid in ids:
			if _bridge.ns_assign_faction(int(fid), _selected):
				n += 1
		_say("%d faction%s now name in %s." % [n, "" if n == 1 else "s", _label_of(_selected)])
		_refresh())
	all.name = "NsAssignAll"

## Rename all: one undoable step, never a hand-given name.
func _build_rename(parent: Control) -> void:
	var box := VBoxContainer.new()
	box.name = "NsRename"
	box.add_theme_constant_override("separation", 4)
	parent.add_child(box)
	box.add_child(DccTheme.mono_label("→ RENAME ALL", "text_faint", DccTheme.FS_MICRO, 2))
	if not bool(_bridge.has_world):
		DccWidgets.note(box, "No world is open.")
		return
	var st: Dictionary = _bridge.ns_world_state()
	var gen := int(st.get("generated", 0))
	var hand := int(st.get("hand_given", 0))
	var unk := int(st.get("unknown", 0))
	DccWidgets.note(box, "Renames the generated places, their provinces and the continents in %s. A name you gave by hand is never touched. %d generated · %d given by hand%s." % [
		_label_of(_selected), gen, hand, (" · %d of unknown origin (a save from before names were tracked)" % unk) if unk > 0 else ""])
	if unk > 0:
		var cb := CheckBox.new()
		cb.name = "NsIncludeUnknown"
		cb.text = "Also rename the %d names of unknown origin" % unk
		cb.button_pressed = _include_unknown
		cb.focus_mode = Control.FOCUS_NONE
		cb.toggled.connect(func(on: bool): _include_unknown = on)
		box.add_child(cb)
	var row := HFlowContainer.new()
	row.add_theme_constant_override("h_separation", 6)
	row.add_theme_constant_override("v_separation", 6)
	box.add_child(row)
	var go := DccWidgets.action(row, "Rename all…", _confirm_rename, true)
	go.name = "NsRenameAll"
	var undo := DccWidgets.action(row, "Undo rename all", func():
		_say("Rename all undone." if _bridge.ns_rename_undo() else "Nothing to undo, or a renamed place changed since.", "text_dim")
		_refresh())
	undo.name = "NsRenameUndo"
	undo.disabled = not bool(st.get("can_undo", false))

func _confirm_rename() -> void:
	var st: Dictionary = _bridge.ns_world_state()
	var will := int(st.get("generated", 0)) + (int(st.get("unknown", 0)) if _include_unknown else 0)
	var dlg := ConfirmationDialog.new()
	dlg.name = "NsRenameConfirm"
	dlg.title = "Rename all"
	dlg.dialog_text = "Rename %d place%s in %s?\n%d name%s you gave by hand will not be touched.%s\nOne step: Undo rename all puts every name back." % [
		will, "" if will == 1 else "s", _label_of(_selected), int(st.get("hand_given", 0)), "" if int(st.get("hand_given", 0)) == 1 else "s",
		"" if _include_unknown or int(st.get("unknown", 0)) == 0 else "\n%d names of unknown origin stay as they are." % int(st.get("unknown", 0))]
	dlg.confirmed.connect(do_rename_all)
	dlg.canceled.connect(dlg.queue_free)
	dlg.confirmed.connect(dlg.queue_free)
	add_child(dlg)
	dlg.popup_centered()

## The rename itself (public so a probe can drive it without the dialog).
func do_rename_all() -> Dictionary:
	var r: Dictionary = _bridge.ns_rename_all(_selected, _include_unknown)
	if bool(r.get("ok", false)):
		_say("Renamed %d place%s (%d given by hand left alone, %d of unknown origin left), %d province%s and %d continent%s." % [
			int(r["renamed"]), "" if int(r["renamed"]) == 1 else "s", int(r["kept_hand"]), int(r["kept_unknown"]),
			int(r["provinces"]), "" if int(r["provinces"]) == 1 else "s", int(r["continents"]), "" if int(r["continents"]) == 1 else "s"])
		## The signal every place rename emits (`global_tools.gd::_settlement_rows`): the map, the lists and the docks re-read.
		if _host != null and _host.get("place_editor_window") != null:
			_host.place_editor_window.place_changed.emit()
	else:
		_say(String(r.get("reason", "Could not rename.")), "accent")
	_refresh()
	return r

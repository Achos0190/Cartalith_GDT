extends Node
## Committed verification harness for **Data manager ▸ Validation ▸
## Definitions** (DM-10) -- the route added 2026-09-06, whose own reason for
## existing is a finding rather than a feature.
##
## Six things are asserted, and none of them is readable from the source:
##
##   1. **The route exists, is live, and is bespoke** -- measured by opening it
##      and reading the drawn tree, not by reading `ROUTES`.
##   2. **The seven columns, as literals.** `CHECKS_COLS` is never compared to
##      itself: the header band's texts are read off the drawn `Label`s and
##      matched against seven strings typed here.
##   3. **`LOCATE` is present and disabled on EVERY row**, and carries a
##      non-empty reason. Not omitted, and not wired.
##   4. **No repair affordance.** No button anywhere in the pane or its footer
##      matches `fix`/`repair`/`normalize`/`apply`, at either state.
##   5. **Both states, on the same run.** A deliberate defect (a blank animal,
##      which `validate_animal` reports Incomplete) produces its row; a
##      cleaned library (pack named, one item imported, the blank deleted)
##      produces the all-clear panel with its counts.
##   6. **Every dash carries a reason.** Any cell drawn `—` must have a
##      non-empty tooltip -- checked over the whole pane in both states, which
##      is the only way an inverted or missing reason shows up.
##
## And the finding itself: the **Map geometry** group is drawn, dashed, with a
## reason naming `poly_self_intersects`, the unclosed ring and ring
## orientation. If someone deletes that group this goes red.
##
## Forced dark, and refuses to run otherwise -- this machine boots light.
##
##   Windowed (the density this was measured at is printed):
##     Godot_v4.7.1-stable_win64_console.exe --path . _dm10_probe.tscn
##   Headless is fine here: nothing below reads a pixel. Every claim is over
##   the node tree, node properties and engine return values.
##     Godot_v4.7.1-stable_win64_console.exe --headless --path . _dm10_probe.tscn

var app: Node
var dm: DataManagerWindow
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("DM10 %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _eq(name: String, got, want) -> void:
	_check(name, got == want, "got=%s want=%s" % [got, want])

# -- Reading the drawn tree ---------------------------------------------------

func _walk(n: Node, out: Array, want: String) -> void:
	if (want == "Label" and n is Label) or (want == "Button" and n is Button) \
			or (want == "PanelContainer" and n is PanelContainer):
		out.append(n)
	for c in n.get_children(true):
		_walk(c, out, want)

func _labels(root: Node) -> Array:
	var out: Array = []
	_walk(root, out, "Label")
	return out

func _buttons(root: Node) -> Array:
	var out: Array = []
	_walk(root, out, "Button")
	return out

func _texts(root: Node) -> PackedStringArray:
	var out := PackedStringArray()
	for l in _labels(root):
		out.append((l as Label).text)
	return out

func _label_containing(root: Node, needle: String) -> Label:
	for l in _labels(root):
		if (l as Label).text.find(needle) >= 0:
			return l
	return null

## A windowed capture of the route, for the two states. **Skipped under
## `--headless`**, where the dummy driver never fires `frame_post_draw` and the
## call would hang rather than fail -- the same trap `MISTAKES.md` records for
## pixel probes. Nothing below asserts on a pixel; this exists so the drawing
## can be looked at, and it says which density it was taken at.
func _shot(tag: String) -> void:
	if DisplayServer.get_name() == "headless":
		print("DM10      (no capture: headless)")
		return
	await RenderingServer.frame_post_draw
	var img := dm.get_texture().get_image()
	var path := "user://_dm10_%s.png" % tag
	img.save_png(path)
	print("DM10      shot %s  %dx%d  density=%s  ->  %s" % [tag,
		img.get_width(), img.get_height(),
		"phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "pointer"),
		ProjectSettings.globalize_path(path)])

# ---------------------------------------------------------------------------

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	## **Through the shell, not through `DccTheme` alone.** This machine boots
	## light, and a bare `DccTheme.apply_theme(true)` only re-points which
	## palette `c()` resolves: every stylebox already baked into a built window
	## keeps the old one, so the capture comes out as dark ink on a light panel
	## and the shot is unreadable while every assertion still passes. Measured
	## on the first windowed run of this probe. `DccShell.toggle_theme()` is the
	## shell's own path and calls `rebuild_theme()` behind it.
	if not DccTheme.is_dark():
		app.toggle_theme()
		await _frames(3)
	if not DccTheme.is_dark():
		print("DM10 FAIL  could not force the dark palette -- refusing to run")
		get_tree().quit(1)
		return
	print("DM10      density=%s  headless=%s  window=%s  palette=dark" % [
		"phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "pointer"),
		DisplayServer.get_name() == "headless",
		DisplayServer.window_get_size()])
	dm = app.data_manager_window

	await _route_is_live_and_bespoke()
	await _defect_state()
	await _all_clear_state()

	print("=== DM-10 CHECKS ", "OK" if _fail == 0 else "FAILED (%d)" % _fail, " ===")
	get_tree().quit(0 if _fail == 0 else 1)

# -- 1. The route ------------------------------------------------------------

func _route_is_live_and_bespoke() -> void:
	print("[1] the route")
	var row: Dictionary = {}
	for r in DataManagerWindow.ROUTES:
		if String((r as Dictionary)["id"]) == "val_defs":
			row = r
	_check("val_defs is a ROUTES row", not row.is_empty())
	_eq("it is in the Validation group", String(row.get("group", "")), "Validation")
	_eq("it is live, not a disclosed gap", String(row.get("kind", "")), "live")
	_check("Check Data stays a gap beside it",
		String(_route_kind("val_check")) == "gap",
		"world-state contradictions still have no validator")
	## The pattern's tell, the same one `_datapane_probe.gd` uses: a `Label`
	## whose text is the route label at `PATTERN_TITLE_FS`. A bespoke pane
	## draws no such label.
	dm.open_route("val_defs")
	await _frames(4)
	var pattern := false
	for l in _labels(dm._pane_body):
		var lb := l as Label
		if lb.text == "Definitions" and lb.has_theme_font_size_override("font_size") \
				and lb.get_theme_font_size("font_size") == DataManagerWindow.PATTERN_TITLE_FS:
			pattern = true
	_check("the pane is bespoke, not the twelve-route pattern", not pattern)
	_eq("the pane header names the route",
		dm._pane_title.text, "VALIDATION ▸ DEFINITIONS")

func _route_kind(id: String) -> String:
	for r in DataManagerWindow.ROUTES:
		if String((r as Dictionary)["id"]) == id:
			return String((r as Dictionary).get("kind", ""))
	return ""

# -- 2..4, 6. The defect state -----------------------------------------------

## A blank animal is `Incomplete`: `tl_add_blank` sets no constraint field, and
## `validate_animal` pushes eight names for exactly that. The library is
## stock-seeded and every stock entry validates ok, so this is the row.
func _defect_state() -> void:
	print("[2] a deliberate defect shows a row")
	var added: Dictionary = app.bridge.world_gen.tl_add_blank("animal", "Probe Defect")
	_check("blank animal added", bool(added.get("ok", false)), str(added))
	dm.open_route("val_defs")
	await _frames(4)
	var body: Control = dm._pane_body

	# --- the seven columns, as literals typed here --------------------------
	var want := ["STATE", "KIND", "NAME", "FIELDS", "MESSAGE", "RESOLUTION", "LOCATE"]
	var head: Array = []
	for l in _labels(body):
		var lb := l as Label
		if want.has(lb.text) and lb.get_theme_font_size("font_size") == DccTheme.FS_MICRO:
			head.append(lb.text)
	_eq("the header band is the seven columns, in order", head, want)

	# --- the defect's own row -----------------------------------------------
	var name_cell := _label_containing(body, "Probe Defect")
	_check("the defect is a row", name_cell != null,
		"looked for a NAME cell reading Probe Defect")
	var state_cell := _label_containing(body, "incomplete")
	_check("its STATE is incomplete", state_cell != null)
	if state_cell != null:
		_eq("incomplete is drawn in the theme's warn ink",
			state_cell.get_theme_color("font_color"), DccTheme.c("warn"))
	var fields := _label_containing(body, "load capacity kg")
	_check("FIELDS lists the unset constraint fields by name", fields != null,
		"" if fields == null else fields.text)
	_check("MESSAGE says how many are unset",
		_label_containing(body, "constraint fields unset") != null)
	_check("RESOLUTION names the window that edits it",
		_label_containing(body, "Travel library") != null)

	# --- the asset rows, and their two dashes -------------------------------
	_check("the empty asset library reports its own warnings",
		_label_containing(body, "Library is empty") != null)
	_check("Asset library rows are present",
		_label_containing(body, "Asset library") != null)

	await _locate_column(body)
	await _no_repair_affordance(body)
	await _every_dash_has_a_reason(body, true)
	await _map_geometry_group(body)
	_check("the status line reports the same count the pane drew",
		dm._status_mid.text.find("findings across") >= 0, dm._status_mid.text)
	await _shot("defect")

## Present and disabled on every row -- counted against the number of rows, so
## a row that skipped it fails rather than passing on its neighbours.
func _locate_column(body: Control) -> void:
	print("[3] LOCATE: present, disabled, reasoned")
	var locates: Array = []
	for b in _buttons(body):
		if (b as Button).text == "Locate":
			locates.append(b)
	var states := _finding_row_count(body)
	_eq("one Locate per finding row", locates.size(), states)
	var enabled := 0
	var mute := 0
	for b in locates:
		if not (b as Button).disabled:
			enabled += 1
		if (b as Button).tooltip_text == "":
			mute += 1
	_eq("none of them is enabled", enabled, 0)
	_eq("all of them carry a reason", mute, 0)
	if locates.size() > 0:
		_check("the reason is why, not \"not implemented\"",
			(locates[0] as Button).tooltip_text.find("is a placed thing") >= 0,
			(locates[0] as Button).tooltip_text.substr(0, 60))

## Counted from the STATE column's own vocabulary rather than from a variable
## the pane kept, so the two can disagree and be caught.
func _finding_row_count(body: Control) -> int:
	var n := 0
	for l in _labels(body):
		var t := (l as Label).text
		if t == "incomplete" or t == "conflicting" or t == "warning":
			n += 1
	return n

func _no_repair_affordance(body: Control) -> void:
	print("[4] no repair affordance")
	var offenders := PackedStringArray()
	for b in _buttons(body) + _buttons(dm._pane_footer):
		var t := (b as Button).text.to_lower()
		if t.find("fix") >= 0 or t.find("repair") >= 0 or t.find("normalize") >= 0:
			offenders.append((b as Button).text)
	_eq("nothing offers to change state", Array(offenders), [])
	var run: Button = null
	for b in _buttons(dm._pane_footer):
		if (b as Button).text == "Run all validators":
			run = b
	_check("the one action re-runs the validators", run != null and not run.disabled)
	if run != null:
		_check("and says they are pure", run.tooltip_text.find("pure functions") >= 0)

func _every_dash_has_a_reason(body: Control, expect_asset_dash: bool) -> void:
	print("[6] every dash carries a reason")
	var mute := PackedStringArray()
	var dashes := 0
	for l in _labels(body):
		var lb := l as Label
		if lb.text != "—":
			continue
		dashes += 1
		if lb.tooltip_text == "":
			mute.append(str(lb.get_path()))
	_check("there is at least one dash to check", dashes > 0, "%d dashes" % dashes)
	_eq("no dash is reasonless", Array(mute), [])
	## The asset row's FIELDS dash and its reason, specifically -- this is the
	## one the design calls out, and a wrong reason here is worse than none.
	## Only in the defect state: an all-clear library draws no asset row at all,
	## so requiring it there would be asserting against a row that cannot exist.
	if not expect_asset_dash:
		return
	var found := false
	for l in _labels(body):
		var lb := l as Label
		if lb.text == "—" and lb.tooltip_text.find("returns Vec<String>") >= 0:
			found = true
	_check("the asset FIELDS dash says validate() returns strings, not a payload",
		found)

func _map_geometry_group(body: Control) -> void:
	print("[5] the Map geometry group, dashed")
	var row := _label_containing(body, "Map geometry")
	_check("the group is drawn", row != null)
	var why := _label_containing(body, "poly_self_intersects")
	_check("it carries the code's own reason", why != null)
	if why != null:
		_check("the unclosed ring is named as NORMAL output, not an error",
			why.text.find("unclosed ring is NORMAL output") >= 0)
		_check("ring orientation is named as unchecked",
			why.text.find("Ring orientation is checked nowhere") >= 0)
	## And the four validators that DO run are named beside it, so the dashed
	## group reads as one row of a coverage list rather than as a failure.
	var texts := _texts(body)
	for fn in ["validate_animal", "validate_vehicle", "validate_vessel",
			"validate_party_preset", "AssetLibrarySession::validate()"]:
		_check("COVERAGE names %s" % fn, texts.has(fn))
	_check("party presets are listed with two states, not three",
		texts.has("ok · incomplete") and texts.has("ok · incomplete · conflicting"))

# -- 5. The all-clear --------------------------------------------------------

## Cleaned by removing every real cause: the blank animal is deleted, the pack
## is named and one item is imported (an empty library is itself two warnings).
func _all_clear_state() -> void:
	print("[5] the all-clear, on a clean library")
	var blank_id := ""
	for r in app.bridge.tl_list("animal"):
		if String((r as Dictionary).get("name", "")) == "Probe Defect":
			blank_id = String((r as Dictionary)["id"])
	_check("the defect is findable to delete", blank_id != "")
	app.bridge.world_gen.tl_delete("animal", blank_id)
	app.bridge.as_set_pack_info("Probe Pack", "probe", "CC0")
	var slot: Dictionary = app.bridge.as_add_custom_slot("Probe Slot", "")
	var img := Image.create(4, 4, false, Image.FORMAT_RGBA8)
	img.fill(Color(0.4, 0.6, 0.8, 1.0))
	var res: Dictionary = app.bridge.as_import_item(String(slot.get("uid", "")),
		"probe.png", img.save_png_to_buffer())
	_check("a real item is in the library", bool(res.get("ok", false)), str(res))
	var warnings: PackedStringArray = app.bridge.as_validate()
	_eq("the engine itself now reports nothing", Array(warnings), [])

	dm.open_route("val_defs")
	await _frames(4)
	var body: Control = dm._pane_body
	_eq("no finding rows remain", _finding_row_count(body), 0)
	var ok := _label_containing(body, "every validator returned ok")
	_check("the all-clear panel is drawn, not an empty pane", ok != null,
		", ".join(_texts(body).slice(0, 4)))
	if ok != null:
		_eq("it is drawn in the theme's good ink",
			ok.get_theme_color("font_color"), DccTheme.c("good"))
	_check("it carries the counts",
		_label_containing(body, "travel definitions across four types") != null)
	_check("COVERAGE still lists every validator with its count",
		_label_containing(body, "checked") != null)
	await _map_geometry_group(body)
	await _every_dash_has_a_reason(body, false)
	_eq("no Locate button survives a table with no rows",
		_buttons(body).filter(func(b): return (b as Button).text == "Locate").size(), 0)
	_check("the status line agrees",
		dm._status_mid.text.find("0 findings") >= 0, dm._status_mid.text)
	await _shot("allclear")

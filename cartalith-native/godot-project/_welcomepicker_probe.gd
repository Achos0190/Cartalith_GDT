extends Node
## Measures `open_project_dialog.gd`'s **welcome mode** after it was rebuilt as
## the 2026-08-31 environment canvas's `state.scr = 'picker'` screen -- a
## centred wordmark column, up to three world cards, three peer actions and one
## foot line -- instead of the gallery with different words in its head.
##
## Run **windowed**, not `--headless`: this reads laid-out `Control.size`, which
## needs a real layout pass, and `MISTAKES.md`'s pixel-probe row applies to
## anything that reads back from the compositor.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _welcomepicker_probe.tscn
##
## Nine measurements: three named densities x three content samples (an empty
## profile, five worlds, and five worlds under a deep storage root -- one
## sample of a layout is the error `MISTAKES.md` names twice).
## `desktop-1920` is the base pointer set, `laptop-1366` the `LAPTOP` override
## layer (`DccTheme.is_laptop()` = narrow and not touch), `tablet-touch` the
## `ROLE` tablet column -- and the last one is measured at the dialog's
## `min_size` floor (880 x 560), which is the tightest rect the composition can
## legally be asked for.
##
## `_gallery_check()` runs once on top of that. `File ▸ Open project…` is not
## this lane's screen, but `_build()` was split underneath it, so its own
## artboard figures are re-asserted rather than assumed.
##
## **The profile is staged in memory only.** `DccSettings._cfg` is written and
## `_save()` is never called, so `user://cartalith_settings.cfg` is untouched;
## the five staged `.zip` files live under a probe-owned directory that
## `_teardown()` removes. Positive control: the populated case must show
## exactly `PICKER_MAX_TILES` of five, and the empty case exactly zero with the
## row hidden -- a probe that could only ever report "no overflow" is the
## failure `MISTAKES.md` calls a check that cannot fail.

var app: Node
var _fail := 0
var _root_empty := ""
var _root_full := ""
var _root_long := ""

## A storage root a user can really have -- a deep synced folder -- and the one
## thing on this screen that can push the composition past the window: the foot
## line is the truncation notice plus this path. `MISTAKES.md`'s layout row
## wants three densities; this is the third *content* sample, and it is the one
## that would have hidden the opt-out before the foot was capped at two lines.
const LONG_DIR := "Dropbox synced worlds/Cartalith campaign archive/second age/northern reaches/saves"
var _keep_recent: Array = []
var _keep_root = null

const DENSITIES := [
	{"name": "desktop-1920", "touch": false, "narrow": false, "w": 1180, "h": 760},
	{"name": "laptop-1366", "touch": false, "narrow": true, "w": 1180, "h": 760},
	{"name": "tablet-touch", "touch": true, "narrow": false, "w": 880, "h": 560},
]

## Five, so the cap at three is visible and the "3 of 5" foot line has
## something to count. Deliberately mixed lengths: a card is a fixed 252 px and
## the long name must clip rather than widen the row.
const STAGED := [
	"Eldra.zip",
	"Vharen Reach.zip",
	"Kessa.zip",
	"An extremely long world name that must not widen the card.zip",
	"Sea of Veld.zip",
]

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(label: String, cond: bool, detail: String = "") -> void:
	print("WP %s  %s%s" % ["ok  " if cond else "FAIL", label,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

# ── Profile staging ──────────────────────────────────────────────────────────

func _stage() -> void:
	## Touch the config once so `_cfg` exists, then write it in memory only.
	DccSettings.recent_projects()
	_keep_recent = DccSettings._cfg.get_value("recent", "paths", [])
	_keep_root = DccSettings._cfg.get_value("storage_roots", "projects", null)
	var base := OS.get_user_data_dir().path_join("_welcomepicker_probe")
	_root_empty = base.path_join("empty")
	_root_full = base.path_join("full")
	_root_long = base.path_join(LONG_DIR)
	for d in [_root_empty, _root_full, _root_long]:
		DirAccess.make_dir_recursive_absolute(d)
	for root in [_root_full, _root_long]:
		for name in STAGED:
			var f := FileAccess.open(String(root).path_join(String(name)), FileAccess.WRITE)
			## Not a real archive: `project_meta()` falls back to "seed unread"
			## and omits the format, which is the widest caption the card can be
			## asked to draw and therefore the right one to measure.
			f.store_string("not-a-zip")
			f.close()
	DccSettings._cfg.set_value("recent", "paths", [])

func _use(root: String) -> void:
	DccSettings._cfg.set_value("storage_roots", "projects", root)

func _teardown() -> void:
	DccSettings._cfg.set_value("recent", "paths", _keep_recent)
	if _keep_root == null:
		DccSettings._cfg.erase_section_key("storage_roots", "projects")
	else:
		DccSettings._cfg.set_value("storage_roots", "projects", _keep_root)
	for root in [_root_full, _root_long]:
		for name in STAGED:
			DirAccess.remove_absolute(String(root).path_join(String(name)))
	var base := OS.get_user_data_dir().path_join("_welcomepicker_probe")
	var deep := _root_long
	while deep.length() > base.length():
		DirAccess.remove_absolute(deep)
		deep = deep.get_base_dir()
	DirAccess.remove_absolute(_root_full)
	DirAccess.remove_absolute(_root_empty)
	DirAccess.remove_absolute(base)
	_check("the probe left nothing behind", not DirAccess.dir_exists_absolute(base),
		"base dir %s" % base)

# ── Walking a built picker ───────────────────────────────────────────────────

func _buttons(node: Node, out: Array) -> void:
	for c in node.get_children():
		if c is Button and (c as Button).visible:
			out.append(c)
		_buttons(c, out)

func _labels(node: Node, out: Array) -> void:
	for c in node.get_children():
		if c is Label and (c as Label).visible:
			out.append((c as Label).text)
		_labels(c, out)

func _measure(dlg, dens: Dictionary, state: String, expect_tiles: int) -> void:
	dlg.open_welcome()
	dlg.size = Vector2i(int(dens["w"]), int(dens["h"]))
	await _frames(6)

	var picker: Control = dlg._picker
	var tiles: HFlowContainer = dlg._picker_tiles
	var cards := tiles.get_child_count() if tiles.visible else 0
	var pmin := picker.get_combined_minimum_size()
	var outer: Control = dlg.get_child(0) as Control
	var omin := outer.get_combined_minimum_size()
	var texts: Array = []
	_labels(picker, texts)
	var btns: Array = []
	_buttons(picker, btns)
	var btn_h := 0.0
	var btn_names := PackedStringArray()
	for b in btns:
		btn_h = maxf(btn_h, (b as Button).size.y)
		btn_names.append((b as Button).text)
	var card_w := 0.0
	if cards > 0:
		card_w = (tiles.get_child(0) as Control).size.x

	var note_lines: int = (dlg._picker_note as Label).get_line_count()
	var tag := "%s/%s" % [dens["name"], state]
	print("WP [%s] dialog=%dx%d  picker.min=%.0fx%.0f  content.min=%.0fx%.0f  laid=%.0fx%.0f  cards=%d  card_w=%.0f  btn_h=%.0f  foot_lines=%d" \
		% [tag, dlg.size.x, dlg.size.y, pmin.x, pmin.y, omin.x, omin.y,
			picker.size.x, picker.size.y, cards, card_w, btn_h, note_lines])
	print("WP [%s] actions=%s" % [tag, btn_names])
	print("WP [%s] foot='%s'" % [tag, dlg._picker_note.text])

	_check("%s: the gallery is not on screen" % tag, not dlg._gallery.visible)
	_check("%s: the picker is" % tag, picker.visible)
	_check("%s: %d card(s)" % [tag, expect_tiles], cards == expect_tiles,
		"got %d, row visible=%s" % [cards, tiles.visible])
	## The whole composition, against the rect it is actually in. `min_size` is
	## 880 x 560 and `wrap_controls` is false, so this is the check that the
	## dialog cannot be shrunk into clipping the opt-out off the bottom.
	_check("%s: the composition fits the window" % tag,
		omin.x <= dlg.size.x and omin.y <= dlg.size.y,
		"min=%.0fx%.0f window=%dx%d" % [omin.x, omin.y, dlg.size.x, dlg.size.y])
	## The one block that can grow, capped -- see `_build_picker()`.
	_check("%s: the foot never takes a third line" % tag, note_lines <= 2,
		"measured %d lines" % note_lines)
	_check("%s: the wordmark is drawn" % tag, texts.has("CARTALITH"),
		"labels=%s" % [texts])
	## Create / open-from-disk / import-a-heightmap: losing any one of them is
	## losing a route into the app, which is worse than the drift being fixed.
	_check("%s: all three routes are present" % tag, btns.size() >= 3,
		"buttons=%s" % [btn_names])
	_check("%s: every action clears the 44 px target floor" % tag, btn_h >= 44.0,
		"tallest measured %.0f" % btn_h)
	if cards > 0:
		_check("%s: the card is the canvas's 252 px" % tag, is_equal_approx(card_w, 252.0),
			"measured %.0f" % card_w)
	dlg.hide()
	await _frames(2)

## `File ▸ Open project…` is the composition this lane was told **not** to
## touch, and `_build()` was restructured underneath it, so it is re-verified
## against its own artboard ("Open project dialog 1920",
## `design/Cartalith DCC Shell.dc.html`): the 1180 x 760 modal, the 4-column
## grid of `TILE_MIN` cells, the dashed import tile first, the three scope
## chips with `Shared` disabled, and the `Open selected` button that starts
## disabled with a reason on hover.
func _gallery_check(dlg) -> void:
	_use(_root_full)
	dlg.open()
	dlg.size = Vector2i(1180, 760)
	await _frames(6)
	var grid: GridContainer = dlg._grid
	var chips: Dictionary = dlg._scope_buttons
	var recent_cells := grid.get_child_count()
	print("WP [gallery] size=%dx%d columns=%d recent_cells=%d tile_min=%s chips=%s" \
		% [dlg.size.x, dlg.size.y, grid.columns, recent_cells, dlg.TILE_MIN, chips.keys()])
	_check("gallery: it is the composition on screen", dlg._gallery.visible and not dlg._picker.visible)
	_check("gallery: the modal is still 1180 x 760", dlg.size == Vector2i(1180, 760))
	_check("gallery: four columns", grid.columns == 4, "got %d" % grid.columns)
	_check("gallery: 232 x 186 cells", dlg.TILE_MIN == Vector2(232, 186),
		"got %s" % dlg.TILE_MIN)
	## The staged profile has an empty recents list, so `Recent` is the dashed
	## import tile alone -- and that IS the assertion: the picker's own list is
	## the union of both chips, so the two screens must disagree here.
	_check("gallery: Recent is empty on the staged profile", recent_cells == 1,
		"got %d cells" % recent_cells)
	dlg._set_scope("all")
	await _frames(4)
	var all_cells := grid.get_child_count()
	print("WP [gallery] all_worlds_cells=%d" % all_cells)
	## The import tile plus five staged worlds -- the gallery has no cap, which
	## is the difference the picker's foot line points at.
	_check("gallery: All worlds lists every one, uncapped",
		all_cells == 1 + STAGED.size(),
		"got %d cells for %d worlds" % [all_cells, STAGED.size()])
	dlg._set_scope("recent")
	await _frames(2)
	_check("gallery: three scope chips", chips.size() == 3, "got %s" % [chips.keys()])
	_check("gallery: Shared is disabled and says why",
		(chips["shared"] as Button).disabled and (chips["shared"] as Button).tooltip_text != "")
	_check("gallery: Open selected starts disabled with a reason",
		dlg._open_btn.disabled and dlg._open_btn.tooltip_text != "",
		"tooltip='%s'" % dlg._open_btn.tooltip_text)
	dlg.hide()
	await _frames(2)

func _run_density(dens: Dictionary) -> void:
	DccTheme.set_phone(false)
	DccTheme.set_touch(bool(dens["touch"]))
	DccTheme.set_narrow(bool(dens["narrow"]))
	## A fresh dialog per density so `_build()` actually runs under it -- the
	## shell builds its own once, in `app.gd::_ready`, and re-flipping the
	## density afterwards would measure a composition built at another one.
	var dlg = OpenProjectDialog.new()
	app.add_child(dlg)
	dlg.setup(app)
	await _frames(2)

	_use(_root_empty)
	await _measure(dlg, dens, "empty", 0)
	_use(_root_full)
	await _measure(dlg, dens, "populated", 3)
	_use(_root_long)
	await _measure(dlg, dens, "populated-deep-root", 3)
	## Once, on the base density: this lane owns the picker, and the gallery is
	## here only to prove `_build()`'s split did not disturb it.
	if String(dens["name"]) == "desktop-1920":
		await _gallery_check(dlg)

	dlg.queue_free()
	await _frames(2)

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	_stage()
	_check("staging wrote five worlds", DirAccess.get_files_at(_root_full).size() == 5,
		"found %s" % [DirAccess.get_files_at(_root_full)])
	_check("the empty profile really is empty",
		DirAccess.get_files_at(_root_empty).is_empty() and DccSettings.recent_projects().is_empty())

	for d in DENSITIES:
		await _run_density(d)

	_teardown()
	print("WP done -- %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

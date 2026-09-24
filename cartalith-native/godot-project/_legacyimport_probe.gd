extends Node
## `OUTSTANDING_WORK.md` §2.11, "Import legacy flat `.zip` settlements, labels
## and icons" (owner Ruling AU, 2026-09-24). Through the SHELL's own open path
## -- `app._load_project`, which File ▸ Open, Recent worlds and the phone picker
## all call -- does a legacy HTML-app export now come back with its
## settlements, labels and icons, do they DRAW, can they be edited, and does
## the world still refuse what needs the substrate?
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _legacyimport_probe.tscn
##
## WINDOWED: a pixel probe run headless passes vacuously (MISTAKES.md), so it
## aborts under the headless display server.
##
## How "draws" is measured: the frame is captured twice -- as the shell drew
## it, and again with the overlay's settlement, label and icon lists emptied --
## and each mark's neighbourhood is compared between the two. Ink the marks
## put down is the difference. POSITIVE CONTROL: the same measurement on a
## freshly GENERATED world, whose settlements the shell has always drawn,
## must find them. NEGATIVE CONTROL: points on the legacy map far from every
## mark must show no difference.
##
## Fixtures (both real HTML-app exports, `crates/cartalith-io/tests/fixtures/`):
## `real_export_seed24601.zip` carries no records (the "opens as before"
## case); `legacy_records_seed24601.zip` carries 24 places, 2 labels and 3
## icons (`tools/legacy_records_capture.js`).

var app: Node
var _fails := 0
const HALF := 7

func _p(s: String) -> void:
	print("LEGACYIMPORT " + s)

func _check(ok: bool, what: String, detail: String = "") -> void:
	_p("%s  %s%s" % ["ok  " if ok else "FAIL", what, ("  -- " + detail) if detail != "" else ""])
	if not ok:
		_fails += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _grab() -> Image:
	await _frames(3)
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()

func _fixture(name: String) -> String:
	return ProjectSettings.globalize_path("res://").path_join("../crates/cartalith-io/tests/fixtures/" + name).simplify_path()

## Screen (framebuffer) position of a grid point, through the overlay's own
## fit rect and canvas transform. `cell` true adds the settlement markers'
## +0.5 centring; labels and icons are continuous grid coordinates.
func _fb(ov: Control, p: Vector2, cell: bool) -> Vector2:
	var rect: Rect2 = ov._displayed_rect()
	var local: Vector2 = ov._cell_to_screen(p, rect) if cell else ov._point_to_screen(p, rect)
	return ov.get_global_transform_with_canvas() * local

func _diff(a: Image, b: Image, at: Vector2) -> float:
	var t := 0.0
	for y in range(int(at.y) - HALF, int(at.y) + HALF + 1):
		for x in range(int(at.x) - HALF, int(at.x) + HALF + 1):
			if x < 0 or y < 0 or x >= a.get_width() or y >= a.get_height():
				continue
			var p := a.get_pixel(x, y)
			var q := b.get_pixel(x, y)
			t += absf(p.r - q.r) + absf(p.g - q.g) + absf(p.b - q.b)
	return t

## Captures with and without the three mark lists, restoring them after.
func _with_and_without(ov: Control) -> Array:
	var shown := await _grab()
	var s: Array = ov._settlements
	var l: Array = ov._labels
	var i: Array = ov._manual_icons
	ov._settlements = []
	ov._labels = []
	ov._manual_icons = []
	ov.queue_redraw()
	var hidden := await _grab()
	ov._settlements = s
	ov._labels = l
	ov._manual_icons = i
	ov.queue_redraw()
	return [shown, hidden]

## Whether a framebuffer point is on the visible map: inside the window and
## inside the viewport host, which clips the camera's content.
func _visible(at: Vector2) -> bool:
	var host: Rect2 = app.viewport.get_global_rect()
	return host.grow(-HALF - 2).has_point(at)

## Whether the overlay draws anything at this grid point at all: it skips
## every mark on the plate's printed frame (`map_overlay.gd`'s
## `_interior_rect`, "nothing drawn from world data belongs on it"), for
## generated and imported worlds alike. On this 96-cell world that frame is
## about ten cells deep.
func _in_interior(ov: Control, p: Vector2, cell: bool) -> bool:
	var rect: Rect2 = ov._displayed_rect()
	var local: Vector2 = ov._cell_to_screen(p, rect) if cell else ov._point_to_screen(p, rect)
	return ov._interior_rect(rect).has_point(local)

## `[inked, checkable, framed]` over `points`: how many of those on screen and
## inside the plate interior show ink (difference above `floor`), and how
## many sit on the frame, where the overlay draws nothing by design.
func _inked(pair: Array, ov: Control, points: Array, cell: bool, floor: float) -> Array:
	var n := 0
	var seen := 0
	var framed := 0
	for p: Vector2 in points:
		var at := _fb(ov, p, cell)
		if not _in_interior(ov, p, cell):
			framed += 1
			continue
		if not _visible(at):
			continue
		seen += 1
		if _diff(pair[0], pair[1], at) > floor:
			n += 1
	return [n, seen, framed]

## The shell opens a world at "cover" zoom, which on a small window puts much
## of the map outside the viewport. Measured at the contain fit instead: the
## camera at zoom 1, origin 0, which is `overlay.displayed_rect()` itself.
func _contain() -> void:
	var vh = app.viewport
	vh._zoom = 1.0
	vh._camera.scale = Vector2.ONE
	vh._camera.position = Vector2.ZERO
	vh.overlay.set_camera_zoom(1.0)
	await _frames(4)

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		_p("ABORT: pixel probe, run windowed")
		get_tree().quit(2)
		return
	var wd := get_tree().create_timer(600.0)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))
	for f in ["real_export_seed24601.zip", "legacy_records_seed24601.zip"]:
		if not FileAccess.file_exists(_fixture(f)):
			_p("ABORT: fixture missing: " + _fixture(f))
			get_tree().quit(2)
			return

	DisplayServer.window_set_size(Vector2i(1800, 1100))
	await _frames(2)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	var bridge = app.bridge
	var ov: Control = app.viewport.overlay
	const FLOOR := 0.5

	## ---- POSITIVE CONTROL: a generated world's settlements draw ------------
	bridge.generate({"seed": 24601, "width_km": 800.0, "grid_w": 96, "grid_h": 61,
		"archetype": "", "villages": false, "sea_level": 0.45})
	var waited := 0
	while bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	await _contain()
	var gen_pts: Array = []
	for s: Dictionary in ov._settlements:
		gen_pts.append(Vector2(float(s["x"]), float(s["y"])))
	var gpair := await _with_and_without(ov)
	var g_ink := _inked(gpair, ov, gen_pts, true, FLOOR)
	_check(g_ink[1] * 2 > gen_pts.size() and g_ink[0] == g_ink[1],
		"POSITIVE CONTROL: a generated world's settlements show ink by this measure",
		"%d of %d on screen in the interior (%d placed, %d on the frame)" % [g_ink[0], g_ink[1], gen_pts.size(), g_ink[2]])

	## ---- 1. an export with no records opens as before --------------------
	_check(app._load_project(_fixture("real_export_seed24601.zip")), "the record-less export opens")
	await _frames(8)
	_check(bridge.last_open_layout == "flat", "read as flat", bridge.last_open_layout)
	_check(bridge.settlements().is_empty() and bridge.label_list().is_empty() and bridge.icon_list().is_empty(),
		"it brings no settlement, label or icon",
		"%d/%d/%d" % [bridge.settlements().size(), bridge.label_list().size(), bridge.icon_list().size()])
	_check(bridge.last_open_warnings.is_empty(), "and reports nothing", str(bridge.last_open_warnings))

	## ---- 2. the export with records ---------------------------------------
	_check(app._load_project(_fixture("legacy_records_seed24601.zip")), "the records export opens")
	await _frames(12)
	await _contain()
	_check(bridge.last_open_layout == "flat", "read as flat", bridge.last_open_layout)
	var sets: Array = bridge.settlements()
	var labels: Array = bridge.label_list()
	var icons: Array = bridge.icon_list()
	_p("counts: settlements %d, labels %d, icons %d" % [sets.size(), labels.size(), icons.size()])
	## Literals: the capture script placed 21 by Auto-populate + 1 by the
	## Settlement tool, 2 labels, 3 icons.
	_check(sets.size() == 22, "22 settlements came back", str(sets.size()))
	_check(labels.size() == 2, "2 labels came back", str(labels.size()))
	_check(icons.size() == 3, "3 icons came back", str(icons.size()))
	if labels.size() == 2:
		_check(String(labels[0].get("text", "")) == "Vharen Reach", "label 0 is \"Vharen Reach\"", str(labels[0]))
	if icons.size() == 3:
		_check(String(icons[1].get("family", "")) == "custom" and String(icons[1].get("set", "")) == "my_set",
			"icon 1 is the custom one from set my_set", str(icons[1]))
	_p("report (%d lines):" % bridge.last_open_warnings.size())
	for w in bridge.last_open_warnings:
		_p("   | " + String(w))
	_check(not bridge.last_open_warnings.is_empty(), "what did not map is reported")

	## The overlay has them.
	_check(ov._settlements.size() == 22, "the overlay holds 22 settlements", str(ov._settlements.size()))
	var hand_labels := 0
	for l: Dictionary in ov._labels:
		if not bool(l.get("generated", false)):
			hand_labels += 1
	_check(hand_labels == 2, "the overlay holds the 2 hand labels", str(hand_labels))
	_check(ov._manual_icons.size() == 3, "the overlay holds 3 icons", str(ov._manual_icons.size()))

	## They draw.
	var pair := await _with_and_without(ov)
	if OS.get_cmdline_user_args().has("--shots"):
		pair[0].save_png("res://_legacyimport_shown.png")
		pair[1].save_png("res://_legacyimport_hidden.png")
	var s_pts: Array = []
	for s: Dictionary in sets:
		s_pts.append(Vector2(float(s["x"]), float(s["y"])))
	var s_ink := _inked(pair, ov, s_pts, true, FLOOR)
	_check(s_ink[1] * 2 > s_pts.size() and s_ink[0] == s_ink[1], "the imported settlements draw",
		"%d of %d on screen in the interior show ink (%d imported, %d on the frame)" % [s_ink[0], s_ink[1], s_pts.size(), s_ink[2]])
	var l_pts: Array = []
	for l: Dictionary in labels:
		l_pts.append(Vector2(float(l["x"]), float(l["y"])))
	var l_ink := _inked(pair, ov, l_pts, false, FLOOR)
	_check(l_ink[1] >= 1 and l_ink[0] == l_ink[1], "every imported label in the interior draws",
		"%d of %d (%d on the frame)" % [l_ink[0], l_ink[1], l_ink[2]])
	var i_pts: Array = []
	for ic: Dictionary in icons:
		i_pts.append(Vector2(float(ic["x"]), float(ic["y"])))
	var i_ink := _inked(pair, ov, i_pts, false, FLOOR)
	_check(i_ink[1] == 3 and i_ink[0] == 3, "all three imported icons draw", "%d of %d (%d on the frame)" % [i_ink[0], i_ink[1], i_ink[2]])

	## NEGATIVE CONTROL: far from every mark, nothing changes.
	var marks: Array = []
	for p in s_pts:
		marks.append(_fb(ov, p, true))
	for p in l_pts + i_pts:
		marks.append(_fb(ov, p, false))
	var quiet := 0
	var probed := 0
	for gy in range(6, 61, 9):
		for gx in range(6, 96, 9):
			var fb := _fb(ov, Vector2(gx, gy), true)
			if not _visible(fb):
				continue
			var near := false
			for m: Vector2 in marks:
				if m.distance_to(fb) < 90.0:
					near = true
					break
			if near:
				continue
			probed += 1
			if _diff(pair[0], pair[1], fb) <= FLOOR:
				quiet += 1
	_check(probed > 0 and quiet == probed, "NEGATIVE CONTROL: points far from every mark show no difference",
		"%d of %d quiet" % [quiet, probed])

	## ---- 3. editable, and the world is still `Loaded` ---------------------
	_check(bridge.civ_edit_settlement(0, {"name": "Renamed Here"}), "an imported settlement can be renamed")
	_check(String(bridge.settlements()[0].get("name", "")) == "Renamed Here", "and the rename holds",
		String(bridge.settlements()[0].get("name", "")))
	var r: Dictionary = bridge.world_gen.recompute_civilisation()
	var reason := String(r.get("reason", ""))
	_check(not bool(r.get("ok", true)) and reason.contains("does not carry its hydrology and tectonic rasters"),
		"Recompute civilisation refuses with NEEDS_SUBSTRATE", reason.left(160))
	_check(bridge.settlements().size() == 22, "and the refusal left the settlements alone", str(bridge.settlements().size()))

	_p("---- %s ----" % ("PASS" if _fails == 0 else "%d FAILED" % _fails))
	get_tree().quit(1 if _fails > 0 else 0)

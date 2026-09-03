extends Node
## GUI replacement stage 5 — the viewport furniture and the two right-dock
## footnotes the 2026-08-31 re-export unblocked.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . \
##       --resolution 1600x900 --rendering-driver opengl3 _stage5hud_probe.tscn
##
## **Why a probe and not a unit test.** Every assertion below reads a real
## `ViewportHost` / `RightDock` inside a booted `app.tscn`. `WorldGen` is a
## cdylib `GodotClass` and cannot be constructed in a `cargo test`, and the three
## things this stage added — a `StyleBox` override on a `Label`, a chip whose
## string is composed from domain + mode + engine state, and a footnote rewritten
## in place rather than at build time — are all node state, not values.
##
## **World-less on purpose.** `spec/05-right-dock-and-bars.md` §5.2's chip has
## four arms and three of them (`GENERATING`, `SCULPT · DRAFT`, the domain name)
## are reachable with no world at all, which is also the state a fresh boot is
## in. The fourth — `EDITED` / `RESOLVED` — needs `stale_stages()` and therefore
## a generate; §2 asserts the world-less branch says so instead, which is the
## branch a user actually meets first.

var _vp: SubViewport
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _truthy(name: String, got: bool) -> void:
	if not got:
		_fail += 1
	print("  ", "ok  " if got else "FAIL", " ", name, "   got=", got, " want=true")

## Every `Label` under `root`, depth first. The dock rebuilds its whole body on
## every context change, so nothing here may hold a node reference across one.
func _labels(root: Node, out: Array) -> Array:
	for c in root.get_children():
		if c is Label:
			out.append(c as Label)
		_labels(c, out)
	return out

func _has_label(root: Node, text: String) -> bool:
	for l in _labels(root, []):
		if l.text == text:
			return true
	return false

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(50)
	print("[BOOT] shell up")

	var host: ViewportHost = app.get("viewport")
	var dock: RightDock = app.get("right_dock_ctrl")

	# =====================================================================
	print("\n=== 1: §5.2/5.3/5.5 the HUD scrim, and §5.4's exemption ===")
	## `scrimBg` is `DccTheme.hud_scrim`, whose own comment said "nothing
	## consumes it yet". Asserted against the token rather than against a literal
	## colour, because a literal here would survive a re-base that moved the
	## token and leave the HUD painted the old value with nothing red.
	var scrim: Color = DccTheme.c("hud_scrim")
	for pair in [["_readout_label", true], ["_coords_label", true], ["_scale_label", false]]:
		var l: Label = host.get(String(pair[0]))
		var want_box: bool = bool(pair[1])
		var sb: StyleBox = l.get_theme_stylebox("normal")
		var painted: bool = sb is StyleBoxFlat and (sb as StyleBoxFlat).bg_color == scrim
		_ok("%s carries the scrim" % pair[0], painted, want_box)
		## §5.2-5.5 set every one of the three in `var(--dim)`, including the
		## scale bar the pill skips.
		_ok("%s ink is text_dim" % pair[0],
			l.get_theme_color("font_color"), DccTheme.c("text_dim"))
	var box: StyleBoxFlat = host.get("_readout_label").get_theme_stylebox("normal")
	## `content_margin_*` are floats; §0.1's figures are whole pixels.
	_ok("scrim padding-x", int(box.content_margin_left), 9)
	_ok("scrim padding-y", int(box.content_margin_top), 4)
	_ok("scrim radius", box.corner_radius_top_left, 6)

	# =====================================================================
	print("\n=== 2: §5.2's context chip, three of its four arms ===")
	## Arm 4 with no world. The chip must name the domain and say why there is
	## no verdict beside it, rather than reading `WORLD · RESOLVED` over a shell
	## that has generated nothing.
	app.select_domain_mode("world", "a")
	await _frames(3)
	var chip: Label = host.get("_vp_context")
	_truthy("chip is visible", chip.visible)
	_ok("WORLD, pipeline, no world", chip.text, "WORLD")
	_truthy("...and says why there is no verdict", chip.tooltip_text.contains("No world yet"))

	## Arm 2, verbatim from the re-export.
	app.select_domain_mode("world", "b")
	await _frames(3)
	_ok("WORLD, sculpt mode", chip.text, "SCULPT · DRAFT")

	## Arm 4 for the two folded domains — the rail's own labels, not the
	## workspace ids.
	app.select_domain("civilization")
	await _frames(3)
	_ok("CIVIL", chip.text, "CIVIL")
	app.select_domain("cartography")
	await _frames(3)
	_ok("CARTO", chip.text, "CARTO")

	## The chip sits in §5.2's cluster: right of the Layers button, 8 px gap.
	var btn: Button = host.get("_layers_btn")
	_ok("cluster gap", int(round(chip.position.x - (btn.position.x + btn.size.x))), 8)

	# =====================================================================
	print("\n=== 3: §5.3's vpField ===")
	## `off` is the base map, and the base map is the relief render.
	host.set_debug_layer("off")
	await _frames(2)
	_ok("no overlay reads relief", host.get("_vp_field"), "relief")

	# =====================================================================
	print("\n=== 4: §1.6's region rows and footnote ===")
	## A synthetic `region_get()` answer — the marquee tool needs a world and a
	## drag, and this is the dock's own entry point either way.
	dock.show_region({
		"x": 0, "y": 512, "w": 1024, "h": 768,
		"x_km": 0.0, "y_km": 1280.0, "w_km": 2560.0, "h_km": 1920.0,
		"cell_count": 786432, "tile_estimates": [],
	})
	await _frames(3)
	var body: Control = app.get("right_dock_body")
	## The origin row is the one this panel never had. `0 · 512` also proves the
	## `has()` read rather than `get(k, 0)`: an x of exactly 0 is a legal corner.
	_truthy("origin row present", _has_label(body, "Origin"))
	_truthy("origin value is the corner", _has_label(body, "0 · 512 cells"))
	## §1.6's `CELLS` row is `toLocaleString('en-US')`.
	_truthy("cells grouped", _has_label(body, "786,432"))
	_truthy("footnote verbatim",
		_has_label(body, "the marquee and the export route are two views of one rect"))

	# =====================================================================
	print("\n=== 5: §1.4's stale footnote ===")
	## Called directly: staleness needs a generate to produce, and what this
	## stage added is the *sentence*, not the staleness. Three inputs — none,
	## one the panel owns rows for, and one it does not.
	_ok("nothing stale -> no sentence", dock._stale_footnote_text({}), "")
	_ok("a stage the panel reads",
		dock._stale_footnote_text({"climate": {"origin": "sculpt", "reason": "sculpt"}}),
		"fields owned by stale stages read — (climate)")
	_ok("two stages, sorted",
		dock._stale_footnote_text({"hydrology": {}, "civ": {}}),
		"fields owned by stale stages read — (civ, hydrology)")
	## A stale stage that owns no row in this panel must not produce a sentence
	## about dashes the reader cannot find.
	_ok("a stage no row reads -> no sentence",
		dock._stale_footnote_text({"tectonics": {}}), "")

	print("\n[RESULT] ", "PASS" if _fail == 0 else "FAIL", "  failures=", _fail)
	get_tree().quit(1 if _fail > 0 else 0)

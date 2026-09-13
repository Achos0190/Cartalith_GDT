extends Node
## Part A gate proof, 2026-09-13: dumps `tool_options_row`'s drawn geometry at
## WORLD/pipeline so a HEAD run and a fixed-code run can be diffed byte-for-
## number outside this process (GDScript cannot load two versions of the same
## class in one run). Not a pass/fail probe itself -- `--dump` writes JSON,
## and the comparison lives in the harness script that runs this twice.
##
##   Godot_v4.7.1 --headless --path . _toolbargeom_probe.tscn -- --force-touch --vp 2560x1600 --dump <path>
##
## Headless is correct here (`MISTAKES.md`: "headless for logic and layout") --
## every number below is a Control rect, nothing rasterised.

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _ready() -> void:
	var parts: PackedStringArray = _arg("--vp", "2560x1600").split("x")
	var vw := int(parts[0])
	var vh := int(parts[1])
	var dump_path := _arg("--dump", "")
	var vp := SubViewport.new()
	vp.size = Vector2i(vw, vh)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(30)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(10)
	## Force WORLD/pipeline explicitly rather than trust the boot default, so
	## this reproduces the verifier's own Bake/Refine row regardless of what
	## domain/mode the shell happens to boot into.
	app.call("_on_rail_node_pressed", "world", "a")
	await _frames(10)

	var shell: Node = app
	var tor := shell.get("tool_options_row") as Control
	var band := shell.get("tool_options_bar") as Control
	var out := {
		"vp": "%dx%d" % [vw, vh],
		"band_rect": _rect(band),
		"row_rect": _rect(tor),
		"children": [],
	}
	if tor != null:
		for c in tor.get_children():
			if c is Control:
				var ctl := c as Control
				var is_spacer := not (ctl is BaseButton) and not (ctl is Label) \
					and (ctl.size_flags_horizontal & Control.SIZE_EXPAND) != 0
				out["children"].append({
					"name": String(c.name),
					"class": c.get_class(),
					"text": String(c.get("text")) if c.get("text") != null else "",
					"spacer": is_spacer,
					"rect": _rect(ctl),
				})

	## Part C, 2026-09-13: the phone bottom-nav tabs' own drawn rects. Added
	## alongside the tool-options-row dump above rather than in a second probe
	## file, and gated on phone mode so a tablet run does not print an empty
	## section. Gate C1's whole claim -- a tab's BUTTON rect is unaffected by
	## swapping its glyph from an SVG `TextureRect` to a mono `Label` -- lives
	## here: `_phone_bar_cell()`'s own header explains why structurally (the
	## button's size comes from `custom_minimum_size`/`SIZE_EXPAND_FILL`, and
	## `col` fills the button via `PRESET_FULL_RECT` regardless of what its own
	## children measure), and this dump is what proves it rather than argues it.
	if DccTheme.is_phone():
		out["phone_tabs"] = []
		var cells: Dictionary = shell.get("_phone_tab_cells")
		for key in cells.keys():
			var cell: Dictionary = cells[key]
			var btn := cell.get("button") as Control
			var icon := cell.get("icon") as Control
			out["phone_tabs"].append({
				"id": String(key),
				"button_rect": _rect(btn),
				"icon_class": icon.get_class() if icon != null else "",
				"icon_text": String(icon.get("text")) if icon != null and icon.get("text") != null else "",
			})

	print("=== toolbargeom vp=", out["vp"], " band=", out["band_rect"], " row=", out["row_rect"], " ===")
	for c in out["children"]:
		print("  ", c)
	if out.has("phone_tabs"):
		print("  -- phone bottom-nav tabs --")
		for t in out["phone_tabs"]:
			print("  ", t)

	if dump_path != "":
		var f := FileAccess.open(dump_path, FileAccess.WRITE)
		f.store_string(JSON.stringify(out, "  "))
		f.close()
		print("[DUMP] ", dump_path)

	get_tree().quit(0)

func _rect(c: Control) -> Dictionary:
	if c == null:
		return {}
	var r := c.get_global_rect()
	return {"x": r.position.x, "y": r.position.y, "w": r.size.x, "h": r.size.y}

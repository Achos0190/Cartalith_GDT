extends Node
## WINDOWED, IN THE REAL SHELL: AP-7 and AP-6 (`ASSET_PACK_TOOLS_PROPOSAL.md`
## §6, Ruling CQ Q7) -- the Asset library inspector's REFINE verbs, each
## writing a NEW variant with an Undo, Fit all (placement only), the family
## rules that disable a verb with its reason, and the pack report's
## near-duplicate row.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _refine_probe.tscn
##   phone composition: ... _refine_probe.tscn -- --vp 1080x2400 --force-touch
##
## `--vp WxH` hosts the app in a SubViewport of that size (`_seamfix_probe`'s
## pattern); any other argument aborts (exit 2). Windowed only: it saves
## screenshots for a human to read.
##
## Steps, each asserted:
##   1. a 200x160 orange disc on transparency, imported into Feature icons ▸
##      Mountain through the real `as_import_item`, shows REFINE with the seam
##      rows hidden and all eight verb chips enabled, each with a tooltip;
##   2. for each of the seven pixel verbs, pressing its chip (the button's own
##      `pressed`) adds exactly one variant, previews it, leaves variant 1's
##      content hash and thumbnail as they were, and offers "Undo <verb>";
##      pressing Undo removes it and nothing else;
##   3. Harmonise's own answer meets AP-7's bar (mean hue within 10° of the
##      target swatch) on the Look the app opened with;
##   4. Fit all on a variant moved off its fit puts it back, writes no pixel,
##      and its Undo restores the old placement;
##   5. a biome tile disables Trim, Pad, Outline and Shadow, and a splat
##      texture disables Tint and Harmonise, each tooltip giving the reason;
##   6. the disc and its mirror in two slots give one near-duplicate row in
##      the pack report (AP-6), naming both.
## Prints `REFINE ok|FAIL <what>`, `REFINE saved <png>` and
## `REFINE checks=<n> failures=<n>`. Exit 0 / 1 / 2.

var app: Node
var _view: Viewport
var _fails := 0
var _checks := 0


func _ok(cond: bool, what: String, detail: String = "") -> void:
	_checks += 1
	print("REFINE %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _snap(tag: String) -> void:
	await RenderingServer.frame_post_draw
	var img := _view.get_texture().get_image()
	DirAccess.make_dir_recursive_absolute("user://refine")
	var path := "user://refine/%s_%s.png" % [tag, "phone" if DccTheme.is_phone() else "desktop"]
	img.save_png(path)
	print("REFINE saved %s" % ProjectSettings.globalize_path(path))


## The fixture sprite: an orange disc shaded left to right with a soft edge,
## off centre on a 200x160 transparent canvas (a margin to trim, a hue to
## harmonise, an edge to outline). `mirror` draws it left-right mirrored.
static func _disc_png(mirror: bool = false) -> PackedByteArray:
	var img := Image.create_empty(200, 160, false, Image.FORMAT_RGBA8)
	for y in 160:
		for x in 200:
			var d := Vector2(x - 90, y - 70).length()
			var a := clampf(50.0 - d, 0.0, 1.0)
			var sx := 199 - x if mirror else x
			img.set_pixel(sx, y, Color8(150 + x / 3, 60 + x / 6, 30, int(round(a * 255.0))))
	return img.save_png_to_buffer()


static func _flat_png(c: Color) -> PackedByteArray:
	var img := Image.create_empty(512, 512, false, Image.FORMAT_RGBA8)
	img.fill(c)
	return img.save_png_to_buffer()


## Bring `c` into view in its enclosing ScrollContainer, the way a user
## scrolls the inspector pane (REFINE is below the fold on a phone).
func _scroll_to(c: Control) -> void:
	var p: Node = c.get_parent()
	while p != null and not (p is ScrollContainer):
		p = p.get_parent()
	if p != null:
		(p as ScrollContainer).ensure_control_visible(c)
		await _frames(3)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	var vp := Vector2i.ZERO
	while i < args.size():
		if args[i] == "--vp" and i + 1 < args.size():
			var parts := String(args[i + 1]).split("x")
			vp = Vector2i(int(parts[0]), int(parts[1]))
			i += 2
			continue
		if args[i] == "--force-touch":
			i += 1
			continue
		print("REFINE ABORT: unknown argument '%s'" % args[i])
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		print("REFINE REFUSED: headless (it saves screenshots; run windowed)")
		get_tree().quit(2)
		return
	app = load("res://shell/app.tscn").instantiate()
	_view = get_viewport()
	if vp != Vector2i.ZERO:
		var sv := SubViewport.new()
		sv.size = vp
		sv.gui_embed_subwindows = true
		sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(sv)
		Input.set_emulate_touch_from_mouse(true)
		sv.add_child(app)
		_view = sv
	else:
		add_child(app)
	await get_tree().create_timer(1.0).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
		await _frames(2)
	var b = app.bridge
	if not b.world_gen.has_method("as_refine") or not b.world_gen.has_method("as_fit_all"):
		print("REFINE CANNOT RUN: this DLL has no as_refine / as_fit_all")
		get_tree().quit(2)
		return
	print("REFINE view=%s palette=%s phone=%s" % [_view.get_visible_rect().size,
		"dark" if DccTheme.is_dark() else "light", DccTheme.is_phone()])
	b.as_clear_library()
	app.open_asset_library()
	await _frames(6)
	var w: AssetLibraryWindow = app.asset_library_window
	const UID := "icons:mountain"

	# -- 1 ------------------------------------------------------------------
	var imp: Dictionary = b.as_import_item(UID, "disc.png", _disc_png())
	_ok(bool(imp.get("ok", false)), "the disc imports into Feature icons ▸ Mountain", str(imp))
	_ok(w.focus_slot(UID), "focus_slot reaches the Mountain slot")
	await _frames(4)
	var refine: Control = w.get("_insp_refine")
	var seam_box: Control = w.get("_insp_seam_box")
	var btns: Dictionary = w.get("_refine_btns")
	var undo: Button = w.get("_insp_refine_undo_btn")
	_ok(refine.is_visible_in_tree() and not seam_box.is_visible_in_tree(), "REFINE shows for a sprite, without the seam rows")
	var all_on := true
	for k in ["fit_all", "trim", "pad", "flip", "tint", "outline", "shadow", "harmonise"]:
		var bt: Button = btns.get(k)
		if bt == null or bt.disabled or bt.tooltip_text == "":
			all_on = false
			print("REFINE   chip %s: %s" % [k, "missing" if bt == null else ("disabled: " + bt.tooltip_text if bt.disabled else "no tooltip")])
	_ok(all_on, "all eight verb chips are enabled on a sprite, each with a tooltip")
	var h0 := String(b.as_item_summary(UID, 0).get("hash", ""))
	var t0: PackedByteArray = b.as_thumbnail_png(UID, 0, 256)

	# -- 2 ------------------------------------------------------------------
	for k in ["trim", "pad", "flip", "tint", "outline", "shadow", "harmonise"]:
		(btns[k] as Button).pressed.emit()
		await _frames(4)
		var n := int(b.as_slot_summary(UID).get("item_count", 0))
		_ok(n == 2, "%s added exactly one variant" % k, "items=%d" % n)
		_ok(int(w.get("_preview_index")) == 1, "%s: the new variant is previewed" % k)
		_ok(String(b.as_item_summary(UID, 0).get("hash", "")) == h0 and b.as_thumbnail_png(UID, 0, 256) == t0,
			"%s: variant 1 is untouched (hash and pixels)" % k)
		_ok(undo.is_visible_in_tree() and undo.text == "Undo %s" % k, "%s: Undo is offered by name" % k, undo.text)
		if k == "outline" or k == "shadow":
			## The green preview ground (a terrain-like colour): the outline
			## and shadow are dark, so the default dark ground would hide them.
			w.call("_pick_preview_bg", 3)
			await _scroll_to(refine)
			await _snap("01_%s" % k)
			w.call("_pick_preview_bg", 1)
		undo.pressed.emit()
		await _frames(4)
		_ok(int(b.as_slot_summary(UID).get("item_count", 0)) == 1 and String(b.as_item_summary(UID, 0).get("hash", "")) == h0,
			"%s: Undo removed the new variant and kept the original" % k)

	# -- 3 ------------------------------------------------------------------
	var hr: Dictionary = b.as_refine(UID, 0, "harmonise")
	var dh := fposmod(float(hr.get("hue_after", 999.0)) - float(hr.get("target_hue", 0.0)) + 180.0, 360.0) - 180.0
	_ok(bool(hr.get("changed", false)) and absf(dh) <= 10.0, "harmonise lands within 10° of the Look's nearest swatch", str(hr))
	b.as_remove_item(UID, 1)
	w.focus_slot(UID)
	await _frames(3)

	# -- 4 ------------------------------------------------------------------
	var fitted: Dictionary = b.as_item_summary(UID, 0)
	b.as_set_item_transform(UID, 0, 1.5, 9.0, -4.0)
	w.focus_slot(UID)
	await _frames(3)
	(btns["fit_all"] as Button).pressed.emit()
	await _frames(4)
	var after: Dictionary = b.as_item_summary(UID, 0)
	_ok(is_equal_approx(float(after["scale"]), float(fitted["scale"])) and is_equal_approx(float(after["pan_y"]), float(fitted["pan_y"]))
		and is_equal_approx(float(after["pan_x"]), float(fitted["pan_x"])), "Fit all put the variant back on its fit", str(after))
	_ok(int(b.as_slot_summary(UID).get("item_count", 0)) == 1 and String(after.get("hash", "")) == h0, "Fit all wrote no pixel and added no variant")
	_ok(undo.is_visible_in_tree() and undo.text == "Undo fit all", "Fit all offers its Undo", undo.text)
	undo.pressed.emit()
	await _frames(4)
	var back: Dictionary = b.as_item_summary(UID, 0)
	_ok(is_equal_approx(float(back["scale"]), 1.5) and is_equal_approx(float(back["pan_x"]), 9.0) and is_equal_approx(float(back["pan_y"]), -4.0),
		"Undo fit all restored the old placement", str(back))

	# -- 5 ------------------------------------------------------------------
	b.as_import_item("biomes:jungle", "j.png", _flat_png(Color(0.2, 0.5, 0.2)))
	w.focus_slot("biomes:jungle")
	await _frames(4)
	var ground_ok := true
	for k in ["trim", "pad", "outline", "shadow"]:
		var bt: Button = btns[k]
		if not bt.disabled or not bt.tooltip_text.contains("tiled edge to edge"):
			ground_ok = false
	_ok(ground_ok and not (btns["flip"] as Button).disabled and not (btns["harmonise"] as Button).disabled,
		"a biome tile disables Trim, Pad, Outline and Shadow with the reason, and keeps Flip and Harmonise")
	_ok(seam_box.is_visible_in_tree(), "the seam rows are back for a ground tile")
	b.as_import_item("textures:grass", "g.png", _flat_png(Color(0.3, 0.5, 0.2)))
	w.focus_slot("textures:grass")
	await _frames(4)
	_ok((btns["tint"] as Button).disabled and (btns["tint"] as Button).tooltip_text.contains("plat")
		and (btns["harmonise"] as Button).disabled and not (btns["flip"] as Button).disabled,
		"a splat texture disables Tint and Harmonise with the reason", (btns["tint"] as Button).tooltip_text)
	await _scroll_to(refine)
	await _snap("02_splat_rules")

	# -- 6 ------------------------------------------------------------------
	b.as_import_item("icons:hill", "mirror.png", _disc_png(true))
	var rep: Dictionary = b.as_pack_report()
	var near: Array = []
	for f in rep.get("findings", []):
		if String((f as Dictionary).get("kind", "")) == "near_duplicate":
			near.append(f)
	_ok(near.size() == 1 and String((near[0] as Dictionary).get("text", "")).contains("Mountain #1")
		and String((near[0] as Dictionary).get("text", "")).contains("Hill #1"),
		"the disc and its mirror are one near-duplicate row in the pack report", str(near))
	w.call("_on_report")
	await _frames(10)
	await _snap("03_report_near")

	print("REFINE checks=%d failures=%d" % [_checks, _fails])
	get_tree().quit(1 if _fails > 0 else 0)

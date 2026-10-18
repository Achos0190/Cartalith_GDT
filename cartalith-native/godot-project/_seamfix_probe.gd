extends Node
## WINDOWED, IN THE REAL SHELL: AP-4 (`ASSET_PACK_TOOLS_PROPOSAL.md` §6, Ruling
## CQ Q7) -- the Asset library inspector's REFINE section: the seam readout
## and Make seamless, which writes a NEW variant, and its Undo.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _seamfix_probe.tscn
##   phone composition: ... _seamfix_probe.tscn -- --vp 1080x2400 --force-touch
##
## `--vp WxH` hosts the app in a SubViewport of that size (`_packopen_probe`'s
## pattern); any other argument aborts (exit 2). Windowed only: it saves
## screenshots for a human to read.
##
## Steps, each asserted:
##   1. a 512² left-to-right ramp (a ground tile with a real seam) imported into
##      Splat ▸ grass through the real `as_import_item`; focusing it shows the
##      REFINE section with a seam readout in words and Make seamless enabled;
##   2. a sprite slot hides the seam rows (Make seamless has nothing to do
##      there); since AP-7 and AP-10 REFINE itself stays, for the sprite
##      verbs and the line-art block;
##   3. pressing Make seamless (the button's own `pressed` signal) adds exactly
##      one variant, leaves variant 1's bytes and content hash as they were,
##      previews the new one, which reads "wraps cleanly" with the button
##      disabled and its reason in the tooltip, and shows Undo;
##   4. Undo removes the new variant and nothing else;
##   5. a flat (already seamless) tile reads "wraps cleanly", Make seamless is
##      disabled, and the engine call adds nothing.
## Prints `SEAMFIX ok|FAIL <what>`, `SEAMFIX saved <png>` and
## `SEAMFIX checks=<n> failures=<n>`. Exit 0 / 1 / 2.

var app: Node
var _view: Viewport
var _fails := 0
var _checks := 0


func _ok(cond: bool, what: String, detail: String = "") -> void:
	_checks += 1
	print("SEAMFIX %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _snap(tag: String) -> void:
	await RenderingServer.frame_post_draw
	var img := _view.get_texture().get_image()
	DirAccess.make_dir_recursive_absolute("user://seamfix")
	var path := "user://seamfix/%s_%s.png" % [tag, "phone" if DccTheme.is_phone() else "desktop"]
	img.save_png(path)
	print("SEAMFIX saved %s" % ProjectSettings.globalize_path(path))


static func _ramp_png() -> PackedByteArray:
	var img := Image.create_empty(512, 512, false, Image.FORMAT_RGBA8)
	for y in 512:
		for x in 512:
			var t := float(x) / 511.0
			img.set_pixel(x, y, Color(0.25 + 0.5 * t, 0.45 + 0.2 * t, 0.2, 1.0))
	return img.save_png_to_buffer()


static func _flat_png() -> PackedByteArray:
	var img := Image.create_empty(512, 512, false, Image.FORMAT_RGBA8)
	img.fill(Color(0.4, 0.5, 0.3, 1.0))
	return img.save_png_to_buffer()


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
		print("SEAMFIX ABORT: unknown argument '%s'" % args[i])
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		print("SEAMFIX REFUSED: headless (it saves screenshots; run windowed)")
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
	if not b.world_gen.has_method("as_make_seamless"):
		print("SEAMFIX CANNOT RUN: this DLL has no as_make_seamless")
		get_tree().quit(2)
		return
	print("SEAMFIX view=%s palette=%s phone=%s" % [_view.get_visible_rect().size,
		"dark" if DccTheme.is_dark() else "light", DccTheme.is_phone()])
	b.as_clear_library()
	var w: AssetLibraryWindow = app.asset_library_window

	# -- 1 ------------------------------------------------------------------
	var imp: Dictionary = b.as_import_item("textures:grass", "ramp.png", _ramp_png())
	_ok(bool(imp.get("ok", false)), "the ramp tile imports into Splat ▸ grass", str(imp))
	_ok(w.focus_slot("textures:grass"), "focus_slot reaches the grass slot")
	await _frames(4)
	var refine: Control = w.get("_insp_refine")
	var label: Label = w.get("_insp_seam_label")
	var btn: Button = w.get("_insp_seam_btn")
	var undo: Button = w.get("_insp_refine_undo_btn")
	var seam_box: Control = w.get("_insp_seam_box")
	_ok(refine != null and refine.is_visible_in_tree(), "REFINE is shown for a ground texture")
	_ok(label.text.contains("visible line"), "the readout says the tile has a seam", label.text)
	_ok(not btn.disabled, "Make seamless is enabled on a seam")
	var h0 := String(b.as_item_summary("textures:grass", 0).get("hash", ""))
	var t0: PackedByteArray = b.as_thumbnail_png("textures:grass", 0, 256)
	await _snap("01_seam")

	# -- 2 ------------------------------------------------------------------
	b.as_import_item("icons:mountain", "m.png", _flat_png())
	w.focus_slot("icons:mountain")
	await _frames(3)
	## Since AP-7 and AP-10 a sprite slot shows REFINE for its verbs and
	## line-art block (`_refine_probe`, `_lineart_probe`); what must still
	## hold is that the seam readout and Make seamless are not offered.
	_ok(not seam_box.is_visible_in_tree() and not label.is_visible_in_tree() and not btn.is_visible_in_tree(),
		"the seam rows are hidden for a sprite family")
	_ok(refine.is_visible_in_tree(), "REFINE itself stays, for the sprite verbs")
	w.focus_slot("textures:grass")
	await _frames(3)

	# -- 3 ------------------------------------------------------------------
	btn.pressed.emit()
	await _frames(4)
	var s: Dictionary = b.as_slot_summary("textures:grass")
	_ok(int(s.get("item_count", 0)) == 2, "Make seamless added exactly one variant", str(s.get("item_count")))
	_ok(String(b.as_item_summary("textures:grass", 0).get("hash", "")) == h0
		and b.as_thumbnail_png("textures:grass", 0, 256) == t0, "variant 1 is untouched (hash and pixels)")
	_ok(int(w.get("_preview_index")) == 1, "the new variant is previewed")
	_ok(label.text.contains("wraps cleanly"), "the new variant reads as wrapping cleanly", label.text)
	_ok(btn.disabled and btn.tooltip_text.contains("already wraps"), "and Make seamless is disabled, with the reason")
	_ok(undo.is_visible_in_tree(), "Undo is offered")
	var sc: Dictionary = b.as_seam_check("textures:grass", 1)
	_ok(bool(sc.get("ok", false)) and float(sc["edge"]) <= 1.25 * float(sc["interior"]) + 1e-9,
		"AP-4's bar on the stored variant: wrap <= 1.25 x centre", str(sc))
	## On a phone the inspector is a scrolling pane and REFINE sits below the
	## fold; bring it into view the way a user scrolls, so the frame shows it.
	var sc_parent: Node = refine.get_parent()
	while sc_parent != null and not (sc_parent is ScrollContainer):
		sc_parent = sc_parent.get_parent()
	if sc_parent != null:
		(sc_parent as ScrollContainer).ensure_control_visible(refine)
		await _frames(3)
	await _snap("02_made")

	# -- 4 ------------------------------------------------------------------
	undo.pressed.emit()
	await _frames(4)
	s = b.as_slot_summary("textures:grass")
	_ok(int(s.get("item_count", 0)) == 1 and String(b.as_item_summary("textures:grass", 0).get("hash", "")) == h0,
		"Undo removed the new variant and kept the original")
	_ok(not undo.is_visible_in_tree(), "Undo is withdrawn after use")

	# -- 5 ------------------------------------------------------------------
	b.as_import_item("biomes:jungle", "flat.png", _flat_png())
	w.focus_slot("biomes:jungle")
	await _frames(3)
	_ok(label.text.contains("wraps cleanly") and btn.disabled, "a flat tile reads as wrapping cleanly; the button is disabled", label.text)
	var r: Dictionary = b.as_make_seamless("biomes:jungle", 0)
	_ok(bool(r.get("ok", false)) and not bool(r.get("changed", true)) and not r.has("index")
		and int(b.as_slot_summary("biomes:jungle").get("item_count", 0)) == 1,
		"the engine leaves an already-seamless tile untouched (no variant added)", str(r))
	await _snap("03_clean")

	print("SEAMFIX checks=%d failures=%d" % [_checks, _fails])
	get_tree().quit(1 if _fails > 0 else 0)

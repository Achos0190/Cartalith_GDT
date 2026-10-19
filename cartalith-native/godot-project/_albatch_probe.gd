extends Node
## Windowed proof that the Asset library's SLOTS-pane batch verbs (Tag,
## Collect, Rename, Duplicate, Delete) take the 44 dp tap floor once on a
## phone, not twice (2026-10-19; filed with AP-9 as "~115 dp tall").
##
##   Godot_v4.7.2-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _albatch_probe.tscn -- --nowelcome
##   phone composition: ... _albatch_probe.tscn -- --nowelcome --force-touch --vp 1080x2340
## `--vp WxH` hosts the app in a SubViewport (`_packopen_probe`'s pattern);
## `--force-touch` is not the real device (`MISTAKES.md`).
##
## Opens the library from the Assets menu row (its popup's own `id_pressed`)
## and, on a phone, presses the SLOTS segment, then measures every batch
## verb's laid size in the window's own units, which are dp on a phone (the
## window is content-scaled by `phone_scale()`), on BOTH axes:
##   phone:   each verb 44 dp tall and at least 44 dp wide, at most 46 dp
##            tall (the bug drew 44 * phone_scale(), 115 dp at 1080 wide);
##   desktop: no floor at all (each shorter than 44 px), and visible.
## Also checks the app turned on Ruling CR's two sprite-chain switches
## (`WorldGen::sprite_area_blit`, `as_premultiplied_bake`; engine default off).
## Fails on the pre-fix `asset_library_window.gd` (checked 2026-10-19).
## Prints `ALBATCH ok|FAIL` rows and `ALBATCH checks=<n> failures=<n>`;
## saves `user://albatch/slots_<density>.png`. Exit 0 pass, 1 a check
## failed, 2 cannot run (headless, or no library row).

const LIB_ROW := "⧉ Asset library"
const VERBS := ["tag", "collect", "rename", "duplicate", "delete"]

var app: Node
var _view: Viewport
var _fails := 0
var _checks := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("ALBATCH %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _walk(n: Node, out: Array) -> void:
	for c in n.get_children(true):
		out.append(c)
		_walk(c, out)

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("ALBATCH REFUSED: headless (run windowed)")
		get_tree().quit(2)
		return
	app = load("res://shell/app.tscn").instantiate()
	_view = get_viewport()
	var args := OS.get_cmdline_user_args()
	var vp_at := args.find("--vp")
	if vp_at >= 0 and vp_at + 1 < args.size():
		var parts := String(args[vp_at + 1]).split("x")
		var sv := SubViewport.new()
		sv.size = Vector2i(int(parts[0]), int(parts[1]))
		sv.gui_embed_subwindows = true
		sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(sv)
		Input.set_emulate_touch_from_mouse(true)
		sv.add_child(app)
		_view = sv
	else:
		add_child(app)
	await get_tree().create_timer(1.0).timeout
	if "--nowelcome" in args and app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
		await _frames(2)
	var phone := DccTheme.is_phone()
	print("ALBATCH view=%s phone=%s phone_scale=%.3f" % [_view.get_visible_rect().size, phone, DccTheme.phone_scale()])

	## Ruling CR's two sprite-chain switches, as the app sets them
	## (`viewport_host.gd` -> `engine_bridge.gd::_apply_sprite_filters`): the
	## engine defaults are off, so reading them ON proves the app's wiring.
	var wg: Object = app.bridge.world_gen
	if wg.has_method("sprite_area_blit") and wg.has_method("as_premultiplied_bake"):
		_ok("the app turned the area blit ON (PACK_SPRITE_AREA_BLIT)", wg.sprite_area_blit())
		_ok("the app turned the premultiplied bake ON (PACK_PREMULTIPLIED_BAKE)", wg.as_premultiplied_bake())
	else:
		_ok("this DLL has the sprite-chain switches", false)

	var all: Array = []
	_walk(app, all)
	var menu: PopupMenu = null
	var idx := -1
	for n in all:
		if n is PopupMenu:
			for i in (n as PopupMenu).item_count:
				if (n as PopupMenu).get_item_text(i) == LIB_ROW:
					menu = n
					idx = i
	if menu == null:
		print("ALBATCH CANNOT RUN: no '%s' menu row" % LIB_ROW)
		get_tree().quit(2)
		return
	menu.id_pressed.emit(menu.get_item_id(idx))
	await _frames(12)
	var win: Window = app.asset_library_window
	_ok("the Assets menu row opens the library window", win != null and win.visible)
	if win == null or not win.visible:
		print("ALBATCH checks=%d failures=%d" % [_checks, _fails])
		get_tree().quit(1)
		return
	if phone:
		var seg: Button = (win.get("_phone_pane_buttons") as Dictionary).get("slots")
		_ok("the SLOTS segment is there to press", seg != null and seg.is_visible_in_tree())
		if seg != null:
			seg.pressed.emit()
		await _frames(8)
	var buttons: Dictionary = win.get("_batch_buttons")
	var scale: float = win.content_scale_factor
	for key in VERBS:
		var b: Button = buttons.get(key)
		if b == null:
			_ok("verb %s exists" % key, false)
			continue
		var s := b.size
		var detail := "%s: %.1f x %.1f window units (%.1f x %.1f px at content scale %.3f)" % [b.text, s.x, s.y, s.x * scale, s.y * scale, scale]
		_ok("%s is drawn" % b.text, b.is_visible_in_tree(), detail)
		if phone:
			_ok("%s takes the 44 dp floor once (44 <= h <= 46, w >= 44)" % b.text, s.y >= 44.0 and s.y <= 46.0 and s.x >= 44.0, detail)
		else:
			_ok("%s has no touch floor on the desktop (h < 44)" % b.text, s.y < 44.0, detail)
	await RenderingServer.frame_post_draw
	var img := _view.get_texture().get_image()
	DirAccess.make_dir_recursive_absolute("user://albatch")
	var path := "user://albatch/slots_%s.png" % ("phone" if phone else "desktop")
	img.save_png(path)
	print("ALBATCH saved %s" % ProjectSettings.globalize_path(path))
	print("ALBATCH checks=%d failures=%d" % [_checks, _fails])
	get_tree().quit(1 if _fails > 0 else 0)

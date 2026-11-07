extends Node
## MS-1 identity probe (`MOBILE_SHELL_MS1_SPEC.md` B2, B9, B10's "same class"): boots the real shell in a
## `--vp`-sized host `SubViewport`, makes every text node transparent, captures the host's framebuffer and
## prints its hash and the class the shell chose. Two captures of the pristine tree by THIS file are the
## baseline; a stage passes B2/B9 at a size when its capture hashes the same as that baseline.
##
##   godot --path . _ms1ident_probe.tscn -- --vp 1920x1080 --out C:/abs/dir --tag desk1920
##   godot --path . _ms1ident_probe.tscn -- --force-touch --vp 1080x2340 --out C:/abs/dir --tag phone1080
##   ... -- --force-touch --vp 2388x1668 --density 2 --out C:/abs/dir --tag tab_d2    (after S1 only)
##   positive control (the hash must move): append  --control
##
## **Windowed, never `--headless`**: the framebuffer is not drawn there (`MISTAKES.md`, "Run a pixel
## probe"). Run with a project `override.cfg` naming its own `config/name`, and that app_userdata dir
## wiped first: a boot with an empty user dir opens the project picker, which this probe hides.
##
## Arguments read by THIS file: `--vp WxH`, `--out <dir>`, `--tag <name>`, `--control`. Arguments it
## accepts and passes through untouched, because the shell reads them: `--force-touch`, `--density D`,
## `--ui-scale S`. Anything else aborts with exit 2 (`MISTAKES.md`, "Write a probe's usage header").
##
## Output lines: `MS1IDENT CLASS tag=… form=… phone_scale=… factor=…` and
## `MS1IDENT HASH tag=… <sha256 of the RGBA8 bytes> <w>x<h>`. Exit 0 when a capture was taken, 1 when it was
## empty, 2 on a bad argument, 3 on the watchdog.

## Text is the one thing that differs between two boots of an identical tree (the status clock, the
## build id), so every text-drawing node is made transparent before the capture: the spec's identity
## method (CQ-4/CQ-6/PW-2). Geometry, colour fills and icons stay.
const TEXT_CLASSES := ["Label", "RichTextLabel", "LineEdit", "TextEdit"]

var _vp: SubViewport
var _app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

## Every argument this probe or the shell under it reads; anything else is a typo that would otherwise
## silently measure the default (`MISTAKES.md`, "Write a probe's usage header").
func _args_ok() -> bool:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a in ["--force-touch", "--control"]:
			i += 1
		elif a in ["--vp", "--out", "--tag", "--density", "--ui-scale"]:
			i += 2
		else:
			print("MS1IDENT FATAL unknown argument ", a)
			return false
	return true

## Makes a node and every descendant's text invisible. `self_modulate` alpha 0 hides a Label's glyphs
## and its own stylebox but not its children; the walk includes INTERNAL children (`MISTAKES.md`, "Walk
## a Godot scene tree"), so a SpinBox's LineEdit is reached too.
func _hide_text(n: Node) -> void:
	for c in n.get_children(true):
		if c is CanvasItem and c.get_class() in TEXT_CLASSES:
			(c as CanvasItem).self_modulate = Color(1, 1, 1, 0)
		elif c is Button:
			## A Button's text is drawn by the button itself, not a child: transparent font colours.
			for k in ["font_color", "font_hover_color", "font_pressed_color", "font_focus_color",
					"font_hover_pressed_color", "font_disabled_color"]:
				(c as Button).add_theme_color_override(k, Color(0, 0, 0, 0))
		_hide_text(c)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 120.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("MS1IDENT WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	if not _args_ok():
		get_tree().quit(2)
		return
	var tag := _arg("--tag", "run")
	var out := _arg("--out", "")
	var parts: PackedStringArray = _arg("--vp", "").split("x")
	if parts.size() != 2 or out == "":
		print("MS1IDENT FATAL needs --vp WxH and --out <dir>")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute(out)
	var vw := int(parts[0])
	var vh := int(parts[1])
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	## `size_2d_override` is the probe host's stand-in for the root's content scale (spec §3.3): a
	## factor of 1 reads (0, 0) here, a dp unit reads the dp size.
	print("MS1IDENT CLASS tag=%s vp=%dx%d form=%s phone_scale=%.4f override=%s root_factor=%.3f" % [
		tag, vw, vh, form, DccTheme.phone_scale(), str(_vp.size_2d_override),
		get_tree().root.content_scale_factor])
	_hide_text(_vp)
	if "--control" in OS.get_cmdline_user_args():
		## Positive control: one 4 x 4 block that must move the hash.
		var r := ColorRect.new()
		r.color = Color(1, 0, 0, 1)
		r.position = Vector2(10, 10)
		r.size = Vector2(4, 4)
		_vp.add_child(r)
	await _frames(8)
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	if img == null or img.is_empty():
		print("MS1IDENT FATAL empty capture")
		get_tree().quit(1)
		return
	img.convert(Image.FORMAT_RGBA8)
	var path := out.path_join(tag + ".png")
	img.save_png(path)
	var ctx := HashingContext.new()
	ctx.start(HashingContext.HASH_SHA256)
	ctx.update(img.get_data())
	print("MS1IDENT HASH tag=%s %s %dx%d" % [tag, ctx.finish().hex_encode(), img.get_width(), img.get_height()])
	get_tree().quit(0)

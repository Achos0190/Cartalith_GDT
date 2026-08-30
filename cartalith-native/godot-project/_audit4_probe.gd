extends Node
## Audit pass 4 boot probe. Confirms the extension the shell actually loads is
## the one the crates were just built from, and boots the real shell windowed.
##
##   godot --path . --resolution 1600x900 _audit4_probe.tscn
##
## Hosted in a `SubViewport` for the reason `_hidpi_probe.gd` documents: the
## real window is clamped to the desktop work area, so any measurement taken
## through `--resolution` is wrong. Nothing here is measured in pixels, but the
## shell classifies its layout from the viewport rect, so the same idiom keeps
## the desktop composition from being misread as a tablet.

var _vp: SubViewport

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	await _frames(2)

	# 1. The extension, before anything else touches it.
	var have := ClassDB.class_exists("WorldGen")
	print("[EXT] WorldGen registered: ", have)
	if not have:
		print("[EXT] FATAL - the .gdextension did not load.")
		get_tree().quit(1)
		return
	var gen: Object = ClassDB.instantiate("WorldGen")
	var methods: Array = []
	for m in gen.get_method_list():
		var n: String = str(m.get("name", ""))
		if n.begins_with("_"):
			continue
		methods.append(n)
	methods.sort()
	print("[EXT] WorldGen public methods visible to GDScript: ", methods.size())

	# 2. The real shell, in a viewport that is not clamped.
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var scn: PackedScene = load("res://shell/app.tscn")
	if scn == null:
		print("[BOOT] FATAL - shell/app.tscn did not load.")
		get_tree().quit(1)
		return
	var app: Node = scn.instantiate()
	_vp.add_child(app)
	await _frames(45)
	print("[BOOT] shell instantiated, node count: ", _count(app))

	# 3. Dead-control census: every disabled Button/PopupMenu item still in the
	#    shipped shell, which is the register's own unit of "outstanding".
	var buttons: Array = []
	_walk(app, buttons)
	var disabled := 0
	var with_tip := 0
	for b in buttons:
		if b is BaseButton and (b as BaseButton).disabled:
			disabled += 1
			if str((b as Control).tooltip_text).strip_edges() != "":
				with_tip += 1
	print("[GAP] BaseButtons total: ", buttons.size())
	print("[GAP] disabled: ", disabled, "  of which disclosed by tooltip: ", with_tip)
	for b in buttons:
		if b is BaseButton and (b as BaseButton).disabled:
			var lbl := ""
			if b is Button:
				lbl = (b as Button).text
			print("[DEAD] ", lbl, " || ", str((b as Control).tooltip_text).replace("\n", " "))

	var img: Image = _vp.get_texture().get_image()
	img.save_png("user://_audit4_boot.png")
	print("[SHOT] user://_audit4_boot.png ", img.get_width(), "x", img.get_height())
	get_tree().quit(0)

func _count(n: Node) -> int:
	var c := 1
	for k in n.get_children():
		c += _count(k)
	return c

func _walk(n: Node, out: Array) -> void:
	if n is BaseButton:
		out.append(n)
	for k in n.get_children():
		_walk(k, out)

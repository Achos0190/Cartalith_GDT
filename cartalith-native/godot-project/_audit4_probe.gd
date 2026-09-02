extends Node
## Audit pass 4 boot probe. Boots the real shell, checks the extension it loads
## is not older than the shell, and takes a census of the dead controls left in
## it.
##
##   godot --path . --resolution 1600x900 _audit4_probe.tscn
##
## **The freshness claim used to be a sentence, not a test.** This header said
## it confirmed the loaded extension was "the one the crates were just built
## from", and the whole implementation of that was
## `ClassDB.class_exists("WorldGen")` plus a printed method count -- which is
## satisfied by any `.dll` that ever exported the class, including one 21
## commits behind. That is the exact condition this project has twice had
## silently invalidate a verification pass, so a probe whose stated job was to
## catch it and did not was worse than no probe.
##
## It is a test now, and the evidence is the shell's own:
## `EngineBridge._has()` records every binding the shell asked for and this
## build does not export, and `missing_bindings()` returns the set. A non-empty
## set is a `.dll` older than the shell. That is the only freshness evidence
## available without a build stamp compiled into the crate, and it is checked
## after the shell has booted and its guards have run, so the set is filled.
##
## What this probe still cannot see: a `.dll` that is stale in an *implementation*
## the shell never guards on -- same bindings, different behaviour behind them.
## Nothing here detects that; a `build_stamp()` `#[func]` on `WorldGen` compared
## against a value the caller passes in would, and would need a crate change.
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

	var fails := 0

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

	## 4. Freshness, checked last so every `_has()` guard the boot runs has
	##    already been asked. See the header for why this is the check and the
	##    class-exists test above is not.
	var bridge = _app_bridge(app)
	var mb: PackedStringArray
	if bridge == null:
		mb = PackedStringArray(["<no EngineBridge on the shell -- app.bridge is null>"])
	else:
		mb = bridge.missing_bindings()
	if mb.is_empty():
		print("[EXT] bindings missing: none -- the library is not older than the shell")
	else:
		fails += 1
		print("[EXT] FAIL stale extension: the shell asked for %d binding(s) this "
			% mb.size()
			+ "build does not export (%s). " % ", ".join(mb)
			+ "The census below was taken against a degraded shell.")

	var img: Image = _vp.get_texture().get_image()
	img.save_png("user://_audit4_boot.png")
	print("[SHOT] user://_audit4_boot.png ", img.get_width(), "x", img.get_height())
	## Was `quit(0)` unconditionally, which is what let the freshness sentence
	## above stand unchallenged for as long as it did.
	print("[EXT] DONE fail=%d" % fails)
	get_tree().quit(1 if fails > 0 else 0)


## `app.bridge` without assuming it: the shell builds it in `_ready`, and a
## boot that failed early leaves it null, which is a different report from
## "no bindings missing".
func _app_bridge(app: Node):
	return app.get("bridge")

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

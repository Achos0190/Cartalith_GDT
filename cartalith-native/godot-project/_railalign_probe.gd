extends Node
## Rail + navpad alignment probe.
##
##   godot --path . --resolution 2560x1600 _railalign_probe.tscn
##
## Owner report: "Alligntment for text and buttons in the rail and the icons
## and circles in the main viewing pane don't match."
##
## Measured off a real 2560x1600 device capture first, which is why this probe
## exists rather than a guess: all four navpad glyph centres sat 12.5-13.0 px
## LEFT of their circle centres, and the active rail label sat at the top of
## its band with the lower ~55 px empty. This reports the same two quantities
## from the live scene, so a fix can be verified rather than eyeballed.
##
## Reports rather than asserts for the rail, because the correct rail geometry
## is what this probe is being used to establish.

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

func _find(n: Node, cls: String, out: Array) -> void:
	if n.is_class(cls):
		out.append(n)
	for c in n.get_children(true):
		_find(c, cls, out)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return

	_vp = SubViewport.new()
	_vp.size = Vector2i(2560, 1600)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(50)
	print("[BOOT] shell up at 2560x1600")

	var shell = app.get("shell")
	if shell == null:
		var shells: Array = []
		_find(app, "Control", shells)
		for s in shells:
			if s.has_method("rail_region"):
				shell = s
				break
	print("[BOOT] shell node: ", shell)

	# --- 1. the rail: is each label centred in its own button? --------------
	print("\n=== 1: rail label vs its button ===")
	var marks = shell.get("_domain_marks")
	var btns = shell.get("_domain_buttons")
	if marks == null or btns == null:
		print("[FATAL] could not reach _domain_marks/_domain_buttons")
		get_tree().quit(1); return
	for id in btns.keys():
		var b: Button = btns[id]
		var lbl: Control = marks[id]["label"]
		var brect := b.get_global_rect()
		## `get_global_rect()` is NOT rotation-aware -- it returns position and
		## size, ignoring `rotation`. These labels are rotated -90 degrees, so
		## the visual box has to be built from the transformed corners or every
		## number below describes a rectangle that is not on screen.
		var xf := lbl.get_global_transform()
		var ls := lbl.size
		var pts: Array[Vector2] = [xf * Vector2(0, 0), xf * Vector2(ls.x, 0),
					xf * Vector2(0, ls.y), xf * Vector2(ls.x, ls.y)]
		var vmin: Vector2 = pts[0]
		var vmax: Vector2 = pts[0]
		for p in pts:
			vmin = Vector2(minf(vmin.x, p.x), minf(vmin.y, p.y))
			vmax = Vector2(maxf(vmax.x, p.x), maxf(vmax.y, p.y))
		var above: float = vmin.y - brect.position.y
		var below: float = brect.end.y - vmax.y
		print("  %-13s button y[%.0f..%.0f] h=%.0f | VISUAL label y[%.0f..%.0f] h=%.0f w=%.0f"
			% [id, brect.position.y, brect.end.y, brect.size.y,
			   vmin.y, vmax.y, vmax.y - vmin.y, vmax.x - vmin.x])
		print("      gap above=%.1f  gap below=%.1f  -> %s   | x: left=%.1f right=%.1f"
			% [above, below,
			   ("CENTRED" if absf(above - below) <= 1.0 else "OFF BY %.1f px" % absf(above - below)),
			   vmin.x - brect.position.x, brect.end.x - vmax.x])
		## The suspected root cause: `get_minimum_size()` is read BEFORE the
		## label is added to the tree, so it may be measured without theme/font
		## context. Compare what it says now (in-tree, themed) against the
		## button height that was derived from the early read.
		var now := lbl.get_minimum_size()
		print("      in-tree get_minimum_size()=%s  -> implied button h = %.0f, actual %.0f%s"
			% [now, now.x + 24.0, brect.size.y,
			   ("" if absf(now.x + 24.0 - brect.size.y) <= 1.0 else "   <-- MISMATCH")])

	# --- 2. the navpad: is each glyph centred in its pill? ------------------
	print("\n=== 2: navpad glyph vs its pill ===")
	var vh = app.get("viewport")
	var navpad = vh.get("_navpad")
	if navpad == null:
		print("  info no navpad on this composition (desktop build has none --")
		print("       _build_navpad() opens `if not _touch: return`).")
		print("       Reporting icon_alignment on the constructed buttons instead.")
	var btns2: Array = []
	_find(vh, "Button", btns2)
	var checked := 0
	for b in btns2:
		var bb := b as Button
		if not bb.has_meta("dcc_navpad_glyph"):
			continue
		checked += 1
		_ok("navpad %s icon_alignment is CENTER" % str(bb.get_meta("dcc_navpad_glyph")),
			bb.icon_alignment, HORIZONTAL_ALIGNMENT_CENTER)
	var lb = vh.get("_layers_btn")
	if lb != null:
		print("  info _layers_btn icon_alignment = ", lb.icon_alignment,
			"  (CENTER is ", HORIZONTAL_ALIGNMENT_CENTER, ")")
	print("  info navpad buttons found: ", checked)
	if checked == 0:
		print("  info none built -- this composition is not touch; the fix is still")
		print("       asserted at construction, so a touch build gets it.")

	print("\n_railalign_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

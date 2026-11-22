extends RefCounted
## Item 3 of `OUTSTANDING_WORK.md` (2026-11-22): the vault window's editor toolbar was clipped at the right edge at
## 1600x900. Run per form: `python -I ... fennara_drive.py GODOT --shell desktop|tablet|phone res://tools/vault_toolbar_drive.gd`.
## Opens a note in the Vault window of a scratch vault (under the scratch APPDATA's user dir) and checks that every
## visible toolbar button lies inside the editor's own rect (wrapped or overflowed into a menu, never clipped), then
## captures the frame. Reads only; the vault is a temp dir.
var _fails := 0


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _find(ctx, t: String, cls: String = "") -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
		if cls == "" or e["class"] == cls:
			return e
	return {}


func _wait_for(ctx, t: String, secs: float = 20.0) -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		var e := _find(ctx, t)
		if not e.is_empty():
			return e
		await ctx.wait(0.25)
	return {}


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


func run(ctx) -> void:
	var app = ctx.get_scene_root()
	if not app.has_method("open_vault_browse"):
		app = ctx.get_scene_root().find_child("App", true, false)
	var phone: bool = ctx.get_scene_root().get_viewport().get_visible_rect().size.x < 700.0
	if phone:
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		if not nw.is_empty():
			await _click_entry(ctx, nw)
			var cancel := await _wait_for(ctx, "CANCEL")
			if not cancel.is_empty():
				await _click_entry(ctx, cancel)
	else:
		var cont := await _wait_for(ctx, "Continue without a world")
		if not cont.is_empty():
			await _click_entry(ctx, cont)
	await ctx.wait(1.0)
	var dir := ProjectSettings.globalize_path("user://small4_vault")
	DirAccess.make_dir_recursive_absolute(dir)
	var f := FileAccess.open(dir + "/Test.md", FileAccess.WRITE)
	f.store_string("# Test\n\nSome **text** for the toolbar.\n")
	f.close()
	var r: Dictionary = app.bridge.vault_connect(dir, "scratch")
	ctx.log("vault_connect: %s" % str(r))
	app.vault_window.open_note("Test.md")
	await ctx.wait(2.0)
	var tb: Control = null
	for n in app.vault_window.find_children("EditorToolbar*", "Container", true, false):
		if (n as Control).is_visible_in_tree():
			tb = n
			break
	## The phone's toolbar rides on the keyboard (VE-8), so it is on screen only while one is up.
	if not phone:
		_check(ctx, "the editor toolbar is on screen", tb != null)
	if tb != null:
		var win: Window = app.vault_window
		var vp: Rect2 = ctx.get_scene_root().get_viewport().get_visible_rect()
		var edge := minf(vp.end.x, DriveCtx.rect_of(tb.get_parent() as Control).end.x)
		var worst := 0.0
		var seen := 0
		for b in tb.find_children("*", "Button", true, false):
			var bc := b as Control
			if not bc.is_visible_in_tree():
				continue
			seen += 1
			var br := DriveCtx.rect_of(bc)
			worst = maxf(worst, br.end.x - edge)
		_check(ctx, "toolbar shows buttons (positive control)", seen > 5, str(seen))
		_check(ctx, "no toolbar button reaches past the editor's right edge (worst overshoot %.0f px)" % worst, worst <= 1.0, "window %s toolbar %s" % [str(win.size), str(DriveCtx.rect_of(tb))])
		_check(ctx, "the toolbar itself fits its parent", DriveCtx.rect_of(tb).end.x <= DriveCtx.rect_of(tb.get_parent() as Control).end.x + 1.0, "%s in %s" % [str(DriveCtx.rect_of(tb)), str(DriveCtx.rect_of(tb.get_parent() as Control))])
	await ctx.capture("vault_toolbar")
	## The same editor docked in the Context panel's Note tab (the narrowest place it lives; desktop and tablet).
	if not phone:
		app.vault_window.hide()
		await ctx.wait(0.5)
		app.bridge.generate({"seed": 483920, "width_km": 800.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": true, "sea_level": 0.45})
		var t0 := Time.get_ticks_msec()
		await ctx.wait(0.2)
		while bool(app.bridge.generating) and Time.get_ticks_msec() - t0 < 300000:
			await ctx.wait(0.25)
		await ctx.wait(2.0)
		app.select("settlement", 0)
		await ctx.wait(1.0)
		app.open_context_tab("note")
		await ctx.wait(1.0)
		var cb := await _wait_for(ctx, "Create ", 5.0)
		if not cb.is_empty():
			await _click_entry(ctx, cb)
			await ctx.wait(2.0)
		var dtb: Control = null
		for n in app.find_children("EditorToolbar*", "Container", true, false):
			if (n as Control).is_visible_in_tree():
				dtb = n
				break
		_check(ctx, "docked: the editor toolbar is on screen", dtb != null)
		if dtb != null:
			var vp2: Rect2 = ctx.get_scene_root().get_viewport().get_visible_rect()
			var edge2 := minf(vp2.end.x, DriveCtx.rect_of(dtb.get_parent() as Control).end.x)
			var worst2 := 0.0
			for b in dtb.find_children("*", "Button", true, false):
				if (b as Control).is_visible_in_tree():
					worst2 = maxf(worst2, DriveCtx.rect_of(b as Control).end.x - edge2)
			_check(ctx, "docked: no toolbar button past the right edge (worst %.0f px)" % worst2, worst2 <= 1.0, str(DriveCtx.rect_of(dtb)))
		await ctx.capture("vault_toolbar_docked")
	ctx.log("VAULT TOOLBAR %s" % ("FAIL: %d check(s)" % _fails if _fails > 0 else "PASS"))

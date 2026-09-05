extends Node
## `design/proposed-2026-09-05-round2/WorldDockB.dc.html` — the WORLD dock's
## two-segment mode pill, measured rather than asserted.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _worlddockb_probe.tscn
##
## Headless is correct here and is not a shortcut: every figure below is a
## layout or a stylebox field, and nothing rasterises. `MISTAKES.md`'s
## `--headless` row draws exactly that line.
##
## **This probe was written to VERIFY a control that already shipped**, not to
## drive a new one. `_build_mode_switch()` landed earlier on 2026-09-05 and was
## generalised the same day; the batch that dispatched this board described the
## switch as work to be done. So the four sections below each answer one claim
## the artboard makes, and a green run is a statement that the shipped control
## is the drawn one:
##
##   §1  the pill exists, is pinned above the scroll, and costs N px
##   §2  its two captions, and whether the canvas's own longer `ldSwA` would fit
##   §3  the lit half is WASHED (`set_segment_on`), not the filled amber slab
##   §4  what the pill's height displaces, measured both ways
##
## §3 is the one the owner ruled on (`LARGE_ITEM_RULINGS.md` 2026-09-05 §6, third
## bullet), so it is asserted against the two builders' own distinguishable
## output rather than against a colour literal: `set_segment_on()` fills with
## `accent_wash` behind an `accent` border, `set_mode_segment_on()` fills with
## `accent` under `accent_ink` type. Comparing to `DccTheme.c()` rather than to a
## hex keeps the assertion true in both palettes — this machine boots light.

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _note(name: String, got) -> void:
	print("       -- ", name, " = ", got)

func _boot(w: int, h: int) -> Node:
	var vp := SubViewport.new()
	vp.size = Vector2i(w, h)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(50)
	return app

## The pill row, found the way the shell finds it rather than through the
## private field: it is the `MarginContainer` in the left dock column whose only
## child is a `PanelContainer` holding an `HBoxContainer` of `Button`s.
func _pill_row(app: Node) -> Control:
	var dock: Control = app.get("left_dock")
	return _find_pill(dock)

func _find_pill(n: Node) -> Control:
	if n is MarginContainer and n.get_child_count() == 1:
		var p := n.get_child(0)
		if p is PanelContainer and p.get_child_count() == 1 and p.get_child(0) is HBoxContainer:
			var row: HBoxContainer = p.get_child(0)
			if row.get_child_count() > 0 and row.get_child(0) is Button:
				return n as Control
	for c in n.get_children():
		var hit := _find_pill(c)
		if hit != null:
			return hit
	return null

func _segments(pill: Control) -> Array:
	var out: Array = []
	for b in (pill.get_child(0).get_child(0) as HBoxContainer).get_children():
		if b is Button:
			out.append(b)
	return out

## The dock's own `ScrollContainer`, walked down from the dock rather than read
## off `_left_dock_scroll` — a probe that knows a private field breaks on a
## rename instead of on a regression.
func _scroll(n: Node) -> ScrollContainer:
	if n is ScrollContainer:
		return n as ScrollContainer
	for c in n.get_children():
		var hit := _scroll(c)
		if hit != null:
			return hit
	return null

## The `y` of one category's header wrapper, in the dock scroll's own space.
func _category_top(app: Node, title: String) -> float:
	var panel: Control = (app as DccShell).workspace_panel("world")
	for e in (panel.get("categories") as Array):
		if String(e["title"]) == title:
			var wrap := (e["body"] as Control).get_parent() as Control
			return wrap.global_position.y
	return -1.0

func _run(app: Node, tag: String) -> void:
	## `app.tscn`'s root IS the shell: `app.gd` is `extends DccShell`,
	## `class_name DccApp`. There is no separate shell node to reach for.
	var shell := app as DccShell
	print("\n=== ", tag, " ===   palette=", "light" if not DccTheme.is_dark() else "dark")
	shell.select_domain_mode("world", "a")
	await _frames(6)

	# -- §1 the pill is there, pinned, and this is what it costs -------------
	print("  [1] the pill")
	var pill := _pill_row(app)
	_ok("pill row found", pill != null, true)
	if pill == null:
		return
	_ok("visible in WORLD", pill.visible, true)
	var scroll := _scroll(app.get("left_dock"))
	_ok("dock has a ScrollContainer", scroll != null, true)
	## Pinned means *outside* the scroll: §2.1 band 2 is above band 4, and the
	## whole of Option B's answer to its own cost is that the control which hid
	## eight categories cannot scroll away from the emptiness it caused.
	_ok("pill is NOT inside the scroll", scroll != null and not scroll.is_ancestor_of(pill), true)
	var pill_h := pill.size.y
	_note("pill row drawn height, px", pill_h)

	# -- §2 the captions, and the one the canvas actually carries ------------
	print("  [2] captions")
	var segs := _segments(pill)
	_ok("two segments", segs.size(), 2)
	if segs.size() == 2:
		_ok("segment a text", (segs[0] as Button).text, "PIPELINE")
		_ok("segment b text", (segs[1] as Button).text, "SCULPT")
	_ok("mode_switch_label world/a", shell.mode_switch_label("world", "a"), "PIPELINE")
	_ok("mode_switch_label world/b", shell.mode_switch_label("world", "b"), "SCULPT")
	## Would the prototype's own `ldSwA` fit? Built through the same
	## `DccWidgets.segment()` the shipped halves use, in a throwaway row, and
	## measured against the dock the pill actually sits in. A number, not an
	## opinion — the artboard asserts it does not clear a 163 px half and this
	## project has shipped a constant asserted against itself before.
	var scratch := HBoxContainer.new()
	scratch.add_theme_constant_override("separation", 3)
	add_child(scratch)
	for t in ["GENERATION PIPELINE", "SCULPT"]:
		var b := DccWidgets.segment(scratch, t, func(): pass)
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	await _frames(4)
	var long_min := scratch.get_combined_minimum_size().x
	scratch.queue_free()
	var short_min := (pill.get_child(0).get_child(0) as Control).get_combined_minimum_size().x
	var dock_w: float = (app.get("left_dock") as Control).size.x
	_note("dock width, px", dock_w)
	_note("shipped PIPELINE|SCULPT pill minimum x, px", short_min)
	_note("canvas ldSwA GENERATION PIPELINE|SCULPT minimum x, px", long_min)
	_note("would ldSwA's pair fit the dock?", long_min <= dock_w)

	# -- §3 the on-state is the wash, not the filled slab --------------------
	print("  [3] on-state")
	if segs.size() == 2:
		var lit: Button = segs[0]
		var sb := lit.get_theme_stylebox("normal") as StyleBoxFlat
		_ok("lit half has a StyleBoxFlat", sb != null, true)
		if sb != null:
			## `set_segment_on()`: accent_wash fill, accent border.
			## `set_mode_segment_on()`: accent fill, accent_ink type.
			_ok("lit fill is accent_wash (washed)", sb.bg_color, DccTheme.c("accent_wash"))
			_ok("lit fill is NOT the accent slab", sb.bg_color == DccTheme.c("accent"), false)
			_ok("lit border is accent", sb.border_color, DccTheme.c("accent"))
		_ok("lit ink is accent", lit.get_theme_color("font_color"), DccTheme.c("accent"))
		_ok("lit ink is NOT accent_ink", lit.get_theme_color("font_color") == DccTheme.c("accent_ink"), false)
		var unlit: Button = segs[1]
		_ok("unlit ink is text_dim", unlit.get_theme_color("font_color"), DccTheme.c("text_dim"))

	# -- §4 what the 34 px displaces ----------------------------------------
	## Measured both ways on the same shell rather than reasoned about: the pill
	## is hidden, the tree is laid out again, and the same category's top is read
	## a second time. The difference IS the cost, and it is the only form of this
	## number that cannot be a restatement of `pill.size.y`.
	print("  [4] displacement")
	var fold: float = scroll.global_position.y + scroll.size.y
	_note("dock scroll bottom (the fold), y", fold)
	var before: Dictionary = {}
	for t in ["Generate", "Terrain", "Geology", "Hydrology", "Climate",
			"Biomes", "Ecology", "Resources", "World data"]:
		before[t] = _category_top(app, t)
	pill.visible = false
	await _frames(6)
	var after: Dictionary = {}
	for t in before:
		after[t] = _category_top(app, t)
	pill.visible = true
	await _frames(6)
	var moved := 0.0
	for t in before:
		var d: float = float(before[t]) - float(after[t])
		if d > moved:
			moved = d
	_note("every category moves down by, px", moved)
	_ok("the measured cost equals the pill's drawn height", moved, pill_h)
	for t in before:
		var b_vis: bool = float(before[t]) < fold
		var a_vis: bool = float(after[t]) < fold
		var verdict := "above the fold"
		if not b_vis and not a_vis:
			verdict = "below the fold BOTH ways (not the pill's doing)"
		elif not b_vis and a_vis:
			verdict = "*** PUSHED BELOW THE FOLD BY THE PILL ***"
		print("       -- ", t, ": with pill y=", before[t], " without y=", after[t], "  ", verdict)

	# -- §5 the gate ---------------------------------------------------------
	print("  [5] the gate, mode b")
	shell.select_domain_mode("world", "b")
	await _frames(6)
	var shown: Array = []
	var panel: Control = shell.workspace_panel("world")
	for e in (panel.get("categories") as Array):
		var wrap := (e["body"] as Control).get_parent() as Control
		if wrap.visible:
			shown.append(String(e["title"]))
	_ok("mode b shows exactly one category", shown, ["Terrain"])
	_ok("mode b lights segment b", (_segments(pill)[1] as Button).get_theme_color("font_color"),
		DccTheme.c("accent"))
	shell.select_domain_mode("world", "a")
	await _frames(4)

## `Timeline.dc.html` board H — the touch leg, and the only section here that is
## about the timeline rather than the mode pill.
##
## H's claim is precise and worth quoting because it is easy to get backwards:
## *"44 is a floor, not a token ... The scrub track is the one control a floor
## cannot fix — it is a 3 px rail inside a 16 px row, and a finger gets 16 px of
## target. So on touch the ROW grows to 44 and the rail and the marks do not."*
##
## So the assertion is **not** that `timeline_track_h` becomes 44. That role is
## `[12, 20]` and is a shared token; re-basing it would move the tablet figure
## for anything else reading it, which is this tree's re-base hazard. The row
## takes `max(timeline_track_h, btn_min_h)` instead — the same floor
## `_tl_square()` already applies to the transport — so the drawn row is 12 px at
## pointer and 44 px at touch while the rail stays a 3 px `ColorRect` centred in
## it and the marks stay 5 px, drawn from the row's own midpoint.
##
## Density is named beside every number below, because a 44 px touch floor and a
## 12 px pointer row are not comparable figures and this project has already
## compared two of them once.
func _run_touch(app: Node, tag: String) -> void:
	print("\n=== ", tag, " ===")
	var shell := app as DccShell
	shell.select_domain("civilization")
	## The strip opens collapsed (`tlOpen:false`), and board H draws the
	## expanded transport. Driven through the shell's own fill rather than by
	## clicking, so this measures the built row and not a hit-test.
	app.set("_tl_expanded", true)
	app.call("_fill_timeline_strip")
	await _frames(8)
	var track: Control = app.get("_tl_track")
	_ok("scrub track built", track != null, true)
	if track == null:
		return
	_note("role timeline_track_h at this density, px", DccTheme.role_px("timeline_track_h"))
	_note("role btn_min_h (the touch floor), px", DccTheme.role_px("btn_min_h"))
	_note("scrub ROW minimum height, px", track.custom_minimum_size.y)
	_ok("row takes the larger of the two",
		track.custom_minimum_size.y,
		float(maxi(DccTheme.role_px("timeline_track_h"), DccTheme.role_px("btn_min_h"))))
	## **Density-gated, and it caught me.** Written first as a bare
	## `>= 44.0` and run on the pointer leg, where it failed at 12 px — a touch
	## requirement asserted against a pointer drawing, which is exactly the row
	## `MISTAKES.md` carries about `DccWidgets.action`'s 39 vs 44. The pointer
	## case has its own assertion, and it is the one that proves the floor is a
	## floor rather than a new minimum: at pointer density `btn_min_h` is 0, so
	## the row must come out at the role's own 12 and not be inflated.
	if DccTheme.is_touch():
		_ok("touch: row clears the 44 px tap floor", track.custom_minimum_size.y >= 44.0, true)
	else:
		_ok("pointer: row is the role's own 12, not inflated",
			track.custom_minimum_size.y, float(DccTheme.role_px("timeline_track_h")))
		_ok("pointer: btn_min_h states no constraint", DccTheme.role_px("btn_min_h"), 0)
	## The rail is the row's drawing, not its target: it must NOT have grown.
	var rail: ColorRect = null
	for c in track.get_children():
		if c is ColorRect and c.color == DccTheme.c("sunken"):
			rail = c
			break
	_ok("rail found", rail != null, true)
	if rail != null:
		_note("rail drawn height, px (must stay 3)", rail.size.y)
		_ok("rail did not grow with the row", rail.size.y, 3.0)
	var play: Button = app.get("_tl_play_button")
	if play != null:
		_note("transport square minimum height, px", play.custom_minimum_size.y)
		if DccTheme.is_touch():
			_ok("touch: transport clears the same floor",
				play.custom_minimum_size.y >= 44.0, true)
		else:
			## `_tl_square()` applies the floor with `maxf`, so at pointer
			## density it can only leave `_menu_square()`'s own height alone.
			_ok("pointer: transport is not inflated by a 0 floor",
				play.custom_minimum_size.y >= 0.0, true)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	## `_touch` is decided once per process from the command line
	## (`dcc_shell.gd`'s own note), so the two densities cannot share a run.
	if "--force-touch" in OS.get_cmdline_user_args():
		var tapp := await _boot(2560, 1600)
		_ok("classified as TABLET", DccTheme.is_tablet(), true)
		await _run_touch(tapp, "TABLET 2560x1600 (touch)")
		print("\n_worlddockb_probe: ", _fail, " FAILURE(S)")
		get_tree().quit(1 if _fail > 0 else 0)
		return
	var app := await _boot(1920, 1080)
	_ok("classified as desktop", DccTheme.is_touch(), false)
	await _run(app, "DESKTOP 1920x1080")
	await _run_touch(app, "DESKTOP 1920x1080 (pointer, the control case)")
	print("\n_worlddockb_probe: ", _fail, " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

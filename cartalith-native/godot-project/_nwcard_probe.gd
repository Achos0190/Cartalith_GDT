extends Node
## New world dialog -- §6.7 phone-card verifier.
##
##   godot --path . _nwcard_probe.tscn -- --force-touch --nowelcome --size 1080x2400
##   godot --path . _nwcard_probe.tscn -- --nowelcome --size 1920x1080      (desktop control)
##
## `--size WxH` picks the density. **Three phone densities minimum, named**
## (`MISTAKES.md`, "Report a layout measurement": one sample is one sample, and
## a card width is a function of the screen's *dp*, not its pixels).
##
## What it measures, per run:
##
## * `is_phone()` / `phone_scale()` / screen dp -- the inputs the card width is
##   derived from, printed so the expected value can be recomputed by hand.
## * The card's own `custom_minimum_size.x` against `min(360, dp - 44)`, which
##   is §6.7's `max-width:360px` inside a scrim padded 22 a side.
## * **Horizontal overflow.** The form sits in a `ScrollContainer` whose
##   horizontal axis is DISABLED, which folds its child's minimum width into
##   its own with no scrollbar to reveal it. Every descendant whose combined
##   minimum width exceeds the window's content width is printed.
## * That the hidden desktop form is *hidden and still live*: `request()` must
##   return every key with a real value. Three of those keys read controls that
##   are no longer on screen (`width_input`, `grid_w_input`, `grid_h_input`) and
##   four more carry values only the hidden toggles and `_sync_from_engine()`
##   ever write.
## * The extent chips as a state machine, with a positive control: REGION lit
##   before, WORLD lit after, `extent_input.selected` and the engine's `world`
##   parameter both following.
## * The dice button: press it, and the seed must move.
##
## The desktop control run asserts the opposite -- no card, no chips -- because
## a phone-only branch that leaks onto the desktop is the failure this pair
## exists to catch.

var _fails := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ck(name: String, ok: bool, detail: String = "") -> void:
	if not ok:
		_fails += 1
	print(("  ok   " if ok else "  FAIL "), name, ("" if detail == "" else "  -- " + detail))

func _arg_size() -> Vector2i:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--size"):
			var v := a.replace("--size", "").strip_edges().split("x")
			if v.size() == 2:
				return Vector2i(int(v[0]), int(v[1]))
	var args := OS.get_cmdline_user_args()
	for i in args.size():
		if args[i] == "--size" and i + 1 < args.size():
			var v := String(args[i + 1]).split("x")
			if v.size() == 2:
				return Vector2i(int(v[0]), int(v[1]))
	return Vector2i(1080, 2400)

## Every descendant whose COMBINED MINIMUM width exceeds `limit`. The deepest
## one is the cause and its ancestors are symptoms, so the whole chain prints.
func _find_wide(node: Node, limit: float, out: Array, depth: int = 0) -> void:
	if node is Control and (node as Control).visible:
		var mw: float = (node as Control).get_combined_minimum_size().x
		if mw > limit:
			out.append("%s%s (%s) min_w=%.0f" % [
				"  ".repeat(depth), node.name, node.get_class(), mw])
	for c in node.get_children():
		_find_wide(c, limit, out, depth + 1)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 150.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var want := _arg_size()
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
		await get_tree().process_frame

	var screen: Vector2 = app.get_viewport_rect().size
	var phone: bool = app.is_phone()
	var scale: float = app.phone_scale()
	var dp := int(screen.x / scale) if phone else int(screen.x)
	print("=== size=", want, " screen=", screen, " phone=", phone, " scale=", scale,
		" dp=", dp, " ===")

	app.open_new_world()
	await get_tree().create_timer(0.6).timeout
	await _frames(4)

	var dlg = app.new_world_dialog
	_ck("dialog is visible", dlg.visible)

	if not phone:
		## The desktop control. A phone-only branch that leaks here is the
		## regression this half exists to catch.
		_ck("no card on desktop", dlg._card == null)
		_ck("no extent chips on desktop", dlg._extent_chips.is_empty())
		_ck("extent dropdown visible on desktop", dlg.extent_input.is_visible_in_tree())
		_ck("grid rows visible on desktop", dlg.grid_h_input.is_visible_in_tree())
		_ck("archetype visible on desktop", dlg.archetype_input.is_visible_in_tree())
		_report()
		return

	## ---- §6.7 card geometry --------------------------------------------------
	var card: PanelContainer = dlg._card
	_ck("card exists", card != null)
	if card == null:
		_report()
		return
	var expect := clampi(dp - 44, 240, 360)
	print("  card min_w=", card.custom_minimum_size.x, " expected=", expect,
		"  card rect=", card.get_global_rect())
	_ck("card width is min(360, dp-44)", int(card.custom_minimum_size.x) == expect,
		"got %d, want %d" % [int(card.custom_minimum_size.x), expect])

	## ---- overflow ------------------------------------------------------------
	## The dialog's own content units, not physical pixels: it is a
	## `CONTENT_SCALE_MODE_CANVAS_ITEMS` window, so its children are laid out in
	## `size / content_scale_factor`.
	var content_w := float(dlg.size.x) / maxf(0.001, dlg.content_scale_factor)
	var over: Array = []
	_find_wide(dlg, content_w, over)
	print("  min-width > content width (", int(content_w), "): ", over.size(), " nodes")
	for e in over:
		print("     ", e)
	_ck("no visible node overflows the dialog width", over.is_empty())

	## ---- the reduced form ----------------------------------------------------
	_ck("seed row is on the card", dlg.seed_input.is_visible_in_tree())
	_ck("resolution is on the card", dlg.resolution_input.is_visible_in_tree())
	_ck("extent dropdown is hidden", not dlg.extent_input.is_visible_in_tree())
	_ck("grid rows hidden", not dlg.grid_h_input.is_visible_in_tree())
	_ck("aspect hidden", not dlg.aspect_input.is_visible_in_tree())
	_ck("map width hidden", not dlg.width_input.is_visible_in_tree())
	_ck("archetype hidden", not dlg.archetype_input.is_visible_in_tree())
	_ck("village toggle hidden", not dlg.villages_check.is_visible_in_tree())

	## ---- hidden but LIVE -----------------------------------------------------
	## The six keys `request()` reads off controls that are no longer drawn. A
	## build that had simply not created them would crash here, and one that
	## defaulted them would post a construction-time value into the engine.
	var req: Dictionary = dlg.request()
	for k in ["seed", "width_km", "grid_w", "grid_h", "archetype", "villages",
			"metropolis", "biome_k", "recovery_phase"]:
		_ck("request() carries " + k, req.has(k))
	_ck("width_km is the real control's value",
		is_equal_approx(float(req["width_km"]), dlg.width_input.value),
		"%s vs %s" % [req["width_km"], dlg.width_input.value])
	_ck("grid_h is derived, not zero", int(req["grid_h"]) >= 4, str(req["grid_h"]))
	_ck("the four civ controls still exist for _sync_from_engine()",
		dlg.villages_check != null and dlg.metropolis_check != null
		and dlg.biome_k_check != null and dlg.recovery_input != null)
	## `_sync_from_engine()` runs on `about_to_popup`; it must not have been
	## defeated by the controls being off screen.
	dlg._sync_from_engine()
	_ck("biome_k round-trips through the hidden control",
		dlg.biome_k_check.button_pressed == dlg._biome_k)

	## ---- extent chips, with a positive control -------------------------------
	_ck("two extent chips", dlg._extent_chips.size() == 2, str(dlg._extent_chips.size()))
	if dlg._extent_chips.size() == 2:
		dlg._on_extent_chip(0)
		await _frames(2)
		var region_lit := _lit(dlg._extent_chips[0])
		var world_lit := _lit(dlg._extent_chips[1])
		print("  after REGION: selected=", dlg.extent_input.selected,
			" world_param=", app.bridge.param_get("world"),
			" lit=", region_lit, "/", world_lit)
		_ck("REGION selects 0", dlg.extent_input.selected == 0)
		_ck("REGION lights the left chip only", region_lit and not world_lit)

		dlg._on_extent_chip(1)
		await _frames(2)
		var region_lit2 := _lit(dlg._extent_chips[0])
		var world_lit2 := _lit(dlg._extent_chips[1])
		print("  after WORLD:  selected=", dlg.extent_input.selected,
			" world_param=", app.bridge.param_get("world"),
			" lit=", region_lit2, "/", world_lit2)
		_ck("WORLD selects 1", dlg.extent_input.selected == 1)
		_ck("WORLD writes the engine parameter", bool(app.bridge.param_get("world")))
		_ck("WORLD lights the right chip only", world_lit2 and not region_lit2)
		_ck("the lit state actually moved", region_lit != region_lit2)
		_ck("chips are at the tap floor",
			dlg._extent_chips[0].size.y >= float(DccTheme.PHONE_TAP_MIN),
			"%.0f" % dlg._extent_chips[0].size.y)
		_ck("chips carry §6.7's radius",
			(dlg._extent_chips[1].get_theme_stylebox("normal") as StyleBoxFlat)
				.corner_radius_top_left == dlg.PHONE_CHIP_RADIUS)
		## The lit state is three visual facts and nothing else; a non-visual
		## reader gets it from the name or not at all.
		print("  a11y names: ", dlg._extent_chips[0].accessibility_name,
			" | ", dlg._extent_chips[1].accessibility_name)
		_ck("the selected chip says so in its accessible name",
			dlg._extent_chips[1].accessibility_name.contains("selected")
			and not dlg._extent_chips[0].accessibility_name.contains("selected"))

	## ---- the extent re-sync, with a positive control -------------------------
	## `request()` carries no extent: Create takes the live parameter, so a
	## stale REGION on screen produces a whole-world map. Drive the parameter
	## from underneath the dialog, the way `project_open` does, and the next
	## `about_to_popup` must follow it.
	dlg._on_extent_chip(0)
	await _frames(2)
	app.bridge.param_set("world", true)
	dlg._sync_from_engine()
	await _frames(2)
	print("  after an engine-side extent change: selected=", dlg.extent_input.selected,
		" lit=", _lit(dlg._extent_chips[0]), "/", _lit(dlg._extent_chips[1]))
	_ck("re-open follows an engine-side extent change", dlg.extent_input.selected == 1)
	_ck("and the chips follow it", _lit(dlg._extent_chips[1]) and not _lit(dlg._extent_chips[0]))
	## The control: an unchanged parameter must not move the selection.
	dlg._sync_from_engine()
	await _frames(2)
	_ck("an unchanged parameter leaves the selection alone", dlg.extent_input.selected == 1)

	## ---- dice ----------------------------------------------------------------
	var dice: Button = null
	for c in dlg.seed_input.get_parent().get_children():
		if c is Button and c != dlg.seed_input:
			dice = c
	_ck("dice button is in the seed row", dice != null)
	if dice != null:
		var before: float = dlg.seed_input.value
		dice.emit_signal("pressed")
		await _frames(2)
		print("  seed ", before, " -> ", dlg.seed_input.value,
			"  dice size=", dice.size, " icon=", dice.icon != null)
		_ck("dice rerolls the seed", not is_equal_approx(before, dlg.seed_input.value))
		_ck("dice icon resolved", dice.icon != null)
		## An icon-only button on a device with no hover has no reachable name
		## unless one is set explicitly.
		_ck("dice has an accessible name", dice.accessibility_name != "",
			dice.accessibility_name)
		_ck("dice is at the tap floor", dice.size.y >= float(DccTheme.PHONE_TAP_MIN),
			"%.0f x %.0f" % [dice.size.x, dice.size.y])

	## ---- the warning travels with the control it warns about -----------------
	dlg.resolution_input.selected = 4        ## 8K
	dlg._on_resolution_selected(4)
	await _frames(2)
	_ck("the 8K warning is visible on the card",
		dlg.dimension_warning_label.is_visible_in_tree()
		and dlg.dimension_warning_label.text != "")
	print("  8K warning: ", dlg.dimension_warning_label.text.substr(0, 70))

	_report()

func _lit(b: Button) -> bool:
	var sb := b.get_theme_stylebox("normal") as StyleBoxFlat
	return sb != null and sb.border_color.is_equal_approx(DccTheme.c("accent"))

func _report() -> void:
	print("=== failures=", _fails, " ===")
	get_tree().quit(0 if _fails == 0 else 1)

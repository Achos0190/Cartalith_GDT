extends Node
## **`File ▸ New world from selection…` end to end, over the real `app.tscn`.**
##
## Nothing here is a stub and nothing re-implements the shell: the marquee is
## armed through `GlobalTools`' own drag/release handlers (the same statics
## `app.register_tool_*_handler` binds for the Region tool), the row is read off
## the live `MenuBar`'s File `PopupMenu` after its own `about_to_popup` refresh,
## the row is fired by emitting the popup's own `id_pressed` (the signal a
## click emits, and the one `_file()` connects `_on_file` to -- `activate_item`
## is not exposed to script in Godot 4), and the confirm is answered by
## pressing the real `Button` in the real card.
##
## Four things it proves:
##   1. the row exists in File, and its four gates report the right reason --
##      including the marquee one, which is the reference's own;
##   2. the CANCEL path changes nothing: no `generation_started`, the grid,
##      seed, extent, icons, labels and the marquee itself all survive;
##   3. the CONFIRM path actually runs `region_new_world` -- `generation_started`
##      then `generation_finished(true)`, and the world state really changed;
##   4. `app.gd`'s own pre-flight guard refuses with no marquee and puts up no
##      dialog, which is the path a command-index hit takes (that index is built
##      from the BUILT menu state, before `about_to_popup` disables anything).
##
## Run: godot4 --headless --path . _regionnewworld_probe.tscn
##
## Headless is sound here: every assertion is over engine state, node text and
## signal order. Nothing is sampled off the framebuffer.

var _app: Node
var _fails := 0
var _started := 0
var _finished: Array = []

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		_fails += 1
		print("  FAIL  %s" % what)

func _eq(what: String, got, want) -> void:
	_ok(got == want, "%s (got %s, want %s)" % [what, got, want])

## `CommandIndex._gather_menu_buttons`' own walk -- a `MenuButton` keeps its
## popup as an INTERNAL child, so `get_children(true)` is required.
func _menu_buttons(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_menu_buttons(c, out)

func _file_popup() -> PopupMenu:
	var mbs: Array = []
	_menu_buttons(_app, mbs)
	for mb in mbs:
		if String((mb as MenuButton).text) == "File":
			return (mb as MenuButton).get_popup()
	return null

func _row_index(p: PopupMenu, id: int) -> int:
	return p.get_item_index(id)

## Every live `AcceptDialog` under the app whose title matches, so the probe
## can tell "no dialog went up" from "a dialog went up".
func _cards(title: String) -> Array:
	var out: Array = []
	for c in _app.get_children(true):
		if c is AcceptDialog and String((c as AcceptDialog).title) == title \
				and not c.is_queued_for_deletion():
			out.append(c)
	return out

func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)

func _button(root: Node, text: String) -> Button:
	var all: Array = []
	_walk(root, all)
	for n in all:
		if n is Button and String((n as Button).text) == text:
			return n
	return null

func _all_prose(root: Node) -> String:
	var all: Array = []
	_walk(root, all)
	var s := ""
	for n in all:
		if n is Label:
			s += String((n as Label).text) + "\n"
	return s

func _snapshot() -> Dictionary:
	var g = _app.bridge.world_gen
	return {
		"w": int(g.get_width()), "h": int(g.get_height()),
		"seed": int(g.get_seed()),
		"km": float(g.get_map_width_km()),
		"region": _app.bridge.region_get(),
		"icons": (g.icon_list() as Array).size(),
		"labels": (g.label_list() as Array).size(),
	}

## The real marquee, through the real handlers. `_region_drag` latches the
## origin on its first call and `_region_release` is what actually calls
## `bridge.region_set()` -- exactly the path a mouse drag takes.
func _arm_marquee(x0: float, y0: float, x1: float, y1: float) -> void:
	_app.arm_tool("region")
	GlobalTools._region_drag(_app, x0, y0)
	GlobalTools._region_drag(_app, x1, y1)
	GlobalTools._region_release(_app, x1, y1, true)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 600.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout

	_app.bridge.generation_started.connect(func(): _started += 1)
	_app.bridge.generation_finished.connect(func(ok: bool): _finished.append(ok))

	var p := _file_popup()
	if p == null:
		print("FATAL: no File menu on the live MenuBar"); get_tree().quit(1); return
	var id: int = DccMenus.ID_NEW_WORLD_FROM_SELECTION
	var idx := _row_index(p, id)
	print("== 0. the row exists, in File ==")
	_ok(idx >= 0, "File carries ID_NEW_WORLD_FROM_SELECTION (index %d)" % idx)
	if idx < 0:
		get_tree().quit(1); return
	_eq("its label", String(p.get_item_text(idx)), "New world from selection…")
	## The built state, which is what `CommandIndex` reads. A silent disabled
	## row is dropped from the index entirely, so the tooltip must be there
	## before any popup has happened.
	_ok(String(p.get_item_tooltip(idx)).strip_edges() != "",
		"carries a tooltip at BUILD time (CommandIndex reads the built state)")

	print("\n== 1. the gates, in order ==")
	p.about_to_popup.emit()
	_ok(p.is_item_disabled(idx), "disabled with no world")
	_ok(String(p.get_item_tooltip(idx)) == DccMenus.REGION_NEW_WORLD_NO_WORLD,
		"and the reason is the no-world one: \"%s\"" % p.get_item_tooltip(idx))

	_app.bridge.generate({
		"seed": 4242, "width_km": 400.0, "grid_w": 256, "grid_h": 160,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _app.bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.5).timeout
	_ok(_app.bridge.has_world, "a parent world generated")
	_started = 0
	_finished = []

	p.about_to_popup.emit()
	_ok(p.is_item_disabled(idx), "still disabled with a world but NO marquee")
	_ok(String(p.get_item_tooltip(idx)) == DccMenus.REGION_NEW_WORLD_NO_REGION,
		"and the reason names the Region select tool: \"%s\"" % p.get_item_tooltip(idx))

	print("\n== 2. app.gd's own guard refuses with no marquee, and puts up NO dialog ==")
	_app.new_world_from_selection()
	await get_tree().process_frame
	_eq("no confirm card went up", _cards("New world from selection").size(), 0)
	_eq("and nothing was started", _started, 0)

	print("\n== 3. arm a real marquee through GlobalTools' own handlers ==")
	## Hand-authored work the replacement must be seen to destroy.
	##
	## **Labels only, and that is the engine's rule rather than a shortcut.**
	## `icon_place` goes through `IconBridge::place`, whose first line is
	## `let armed = self.armed.as_ref()?`, and `icon_arm` returns `false`
	## without a loaded asset pack -- so with no pack there is no armed icon
	## and `icon_place` is a no-op that returns `-1`. Loading a pack to place
	## one icon would be a second subsystem inside this probe; the icon clear
	## is already asserted against the real cdylib by
	## `_verify_region_probe.gd`, and the count is printed here rather than
	## asserted so a future build that *does* place one is not silently
	## re-read as a pass.
	var g = _app.bridge.world_gen
	g.icon_place(40.0, 40.0)
	g.label_create(50.0, 50.0, "PARENT LABEL")
	_arm_marquee(64.0, 40.0, 192.0, 120.0)
	var before := _snapshot()
	print("  parent: %s" % before)
	_ok(not (before["region"] as Dictionary).is_empty(), "the marquee reached the engine")
	_ok(int(before["labels"]) > 0,
		"the parent holds %d label(s) (icons: %d, see above)" % [before["labels"], before["icons"]])
	p.about_to_popup.emit()
	_ok(not p.is_item_disabled(idx), "the row is now ENABLED")
	_ok(String(p.get_item_tooltip(idx)) == DccMenus.REGION_NEW_WORLD_TIP,
		"and its tooltip is back to what the command does")

	print("\n== 4. CANCEL changes nothing ==")
	## The shipped dispatch, minus the mouse: `id_pressed` is what a click on
	## this row emits and what `_file()` connects `_on_file` to. A real press
	## is what the device pass owes.
	p.id_pressed.emit(id)
	await get_tree().process_frame
	await get_tree().process_frame
	var cards := _cards("New world from selection")
	_eq("the confirm card went up", cards.size(), 1)
	if cards.is_empty():
		print("FATAL: no card to answer"); get_tree().quit(1); return
	var card: AcceptDialog = cards[0]
	var prose := _all_prose(card)
	print("  ---- card prose ----\n%s  --------------------" % prose)
	## The task the wrapper's own doc sets: the dialog must NAME what it
	## destroys, not ask "are you sure". Each clause is asserted separately so a
	## reworded sentence that drops one cannot pass.
	for word in ["civilisation", "labels", "icons", "ways", "routes", "paint",
			"sculpt", "journeys", "measurements", "undo", "vault link",
			"cannot be undone"]:
		_ok(prose.to_lower().contains(word.to_lower()),
			"the prose names \"%s\"" % word)
	_ok(prose.contains("Save project"), "the foot points at File ▸ Save project")
	_ok(_button(card, "Replace the world") != null,
		"the answer is named after what it does, not \"OK\"")
	var cancel := _button(card, "Cancel")
	_ok(cancel != null, "there is a Cancel")
	cancel.pressed.emit()
	await get_tree().process_frame
	await get_tree().create_timer(0.3).timeout
	_eq("nothing was started", _started, 0)
	_eq("nothing finished", _finished.size(), 0)
	var after_cancel := _snapshot()
	_eq("grid unchanged", Vector2i(int(after_cancel["w"]), int(after_cancel["h"])),
		Vector2i(int(before["w"]), int(before["h"])))
	_eq("seed unchanged", after_cancel["seed"], before["seed"])
	_eq("extent unchanged", after_cancel["km"], before["km"])
	_eq("icons untouched", after_cancel["icons"], before["icons"])
	_eq("labels untouched", after_cancel["labels"], before["labels"])
	_ok(not (after_cancel["region"] as Dictionary).is_empty(),
		"the marquee is STILL SET after a cancel")
	_eq("the card was dismissed", _cards("New world from selection").size(), 0)

	print("\n== 5. CONFIRM actually runs region_new_world ==")
	p.about_to_popup.emit()
	_ok(not p.is_item_disabled(idx), "the row is still enabled after the cancel")
	p.id_pressed.emit(id)
	await get_tree().process_frame
	await get_tree().process_frame
	var cards2 := _cards("New world from selection")
	_eq("the confirm card went up again", cards2.size(), 1)
	if cards2.is_empty():
		print("FATAL: no card to confirm"); get_tree().quit(1); return
	var go := _button(cards2[0], "Replace the world")
	if go == null:
		print("FATAL: no destructive button"); get_tree().quit(1); return
	go.pressed.emit()
	await get_tree().process_frame
	_eq("generation_started fired", _started, 1)
	_ok(_app.bridge.generating, "the bridge is generating (threaded, as import_heightmap is)")
	var spins := 0
	while _app.bridge.generating and spins < 400:
		spins += 1
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.5).timeout
	_eq("generation_finished fired once", _finished.size(), 1)
	_ok(_finished.size() == 1 and bool(_finished[0]),
		"and it reported ok=true (last_summary: \"%s\")" % _app.bridge.last_summary)

	var after := _snapshot()
	print("  child : %s" % after)
	_ok(Vector2i(int(after["w"]), int(after["h"]))
			!= Vector2i(int(before["w"]), int(before["h"])),
		"the WORLD CHANGED -- grid %dx%d -> %dx%d" % [
			before["w"], before["h"], after["w"], after["h"]])
	_eq("the long edge is the engine's default tile size", int(after["w"]), 1024)
	_eq("the seed is inherited", after["seed"], before["seed"])
	## `region_as_new_world`'s own rule, the reference's
	## `mapWidthKm * sel.w / GW`: the marquee is 128 of 256 cells wide here, so
	## the child is exactly half the parent's width in km.
	var want_km: float = maxf(1.0, float(before["km"])
		* float(int((before["region"] as Dictionary).get("w", 0))) / float(int(before["w"])))
	_ok(absf(float(after["km"]) - want_km) < 1e-6,
		"the extent rescaled to the selection's share (%.4f km, want %.4f)" % [after["km"], want_km])
	_ok((after["region"] as Dictionary).is_empty(), "the marquee was cleared")
	_eq("icons cleared", after["icons"], 0)
	_eq("labels cleared", after["labels"], 0)
	_eq("undo cleared", _app.bridge.world_gen.can_undo(), false)

	print("\n== 6. and the row gates itself again afterwards ==")
	p.about_to_popup.emit()
	_ok(p.is_item_disabled(idx), "disabled again -- the replacement cleared the marquee")
	_ok(String(p.get_item_tooltip(idx)) == DccMenus.REGION_NEW_WORLD_NO_REGION,
		"with the no-region reason")

	print("\n%s (%d failure%s)" % ["ALL PASS" if _fails == 0 else "FAILURES",
		_fails, "" if _fails == 1 else "s"])
	get_tree().quit(1 if _fails > 0 else 0)

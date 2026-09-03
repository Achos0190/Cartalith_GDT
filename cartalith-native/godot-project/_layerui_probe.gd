extends SceneTree

# Lane A -- the layer-stack UI (GUI_GAP_REGISTER.md CA-03/CA-04, RD-10).
#
# `_layerstack_probe.gd` covers the BINDING. This covers the two surfaces built
# over it, against the real `EngineBridge`, the real `RenderWorkspace` builders
# and the real `RightDock._append_layers`, because none of it is reachable from
# `cargo test`: `WorldGen` is a cdylib GodotClass and `DccWidgets` needs a live
# scene tree.
#
# What it asserts, in the order the brief names the risks:
#   0. the panel writes NOTHING at build -- the default stack stays the default
#   1. the rows are the engine's own, top-first
#   2. a visibility gesture sends ONE key and leaves every other value alone
#   3. reorder is data: the engine's order moves, and the rows follow IT
#   4. both docks stay in step through `layer_stack_changed`
#   5. an older cdylib (no bindings) disables the panel instead of erroring
#
#   godot --headless --script _layerui_probe.gd

var fails := 0
var pushes := 0

func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		fails += 1
		print("  FAIL  %s" % what)

func _ids(rows: Array) -> Array:
	var out: Array = []
	for r in rows:
		out.append(String((r as Dictionary).get("id", "?")))
	return out

func _row(rows: Array, id: String) -> Dictionary:
	for r in rows:
		if String((r as Dictionary).get("id", "")) == id:
			return r
	return {}

## Every header row `_layer_row()` built, in order: `_layer_host`'s children
## alternate header (HBoxContainer) / body (MarginContainer).
func _heads(host: Node) -> Array:
	var out: Array = []
	for c in host.get_children():
		if c is HBoxContainer:
			out.append(c)
	return out

## Every descendant whose class is `cls`, depth-first in draw order.
func _find(node: Node, cls: String) -> Array:
	var out: Array = []
	for c in node.get_children():
		if c.is_class(cls):
			out.append(c)
		out.append_array(_find(c, cls))
	return out

func _buttons(row: Node) -> Array:
	var out: Array = []
	for c in row.get_children():
		if c is Button:
			out.append(c)
	return out

func _initialize() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("FAIL  WorldGen is not registered -- the GDExtension did not load")
		quit(1)
		return

	var bridge: Node = load("res://shell/engine_bridge.gd").new()
	## `_ready()` by hand: `SceneTree._initialize()` runs before the tree is
	## live, so `root.add_child()` here does not put a node *inside* the tree and
	## `_ready` never fires -- measured, `is_inside_tree()` comes back false. The
	## capability probing under test lives in `_ready`, so it is called directly.
	bridge._ready()
	_ok(bridge.layer_stack_api, "engine_bridge._ready() probed the three methods and enabled the panel")
	if not bridge.layer_stack_api:
		quit(1)
		return
	# Connected BEFORE anything is built, so a build-time write cannot hide.
	bridge.layer_stack_changed.connect(func(): pushes += 1)

	var wg = bridge.world_gen
	var shipped := _ids(bridge.layer_stack())

	print("\n== 0. the panel is built and writes NOTHING ==")
	var ws: Node = load("res://shell/workspaces/render_workspace.gd").new()
	ws._nested = true
	ws.bridge = bridge
	root.add_child(ws)
	var host := VBoxContainer.new()
	root.add_child(host)
	ws.build_layer_stack_into(host)
	_ok(pushes == 0, "building the Layers panel fired 0 set_layer_stack calls (got %d)" % pushes)
	# The direct measurement of "the default stayed the default": `reset_appearance`
	# counts the four override authorities, and `appearance_layers` is one of them.
	_ok(wg.reset_appearance() == 0,
		"after the build the engine holds ZERO appearance overrides -- no default stack was pushed")
	_ok(_ids(bridge.layer_stack()) == shipped, "the stack is untouched: %s" % str(shipped))

	print("\n== 1. the rows are the engine's own, top-first ==")
	var heads := _heads(ws._layer_host)
	_ok(heads.size() == 3, "three header rows (got %d)" % heads.size())
	var drawn: Array = []
	for h in heads:
		for c in h.get_children():
			if c is Label:
				drawn.append(String((c as Label).text))
				break
	_ok(drawn == ["Hillshade", "Colour relief", "Terrain"],
		"drawn top-first in the engine's own order: %s" % str(drawn))
	_ok(_buttons(heads[0]).size() == 3, "each row carries dot + Up + Down (got %d buttons)" % _buttons(heads[0]).size())
	_ok(_buttons(heads[0])[1].disabled, "the top row's Up is disabled")
	_ok(_buttons(heads[2])[2].disabled, "the bottom row's Down is disabled")

	print("\n== 2. a visibility gesture sends ONE key, over a STALE cache ==")
	# Terrain is the bottom row. The other two are moved behind the panel's back
	# -- straight on `WorldGen`, so the bridge emits no `layer_stack_changed`
	# and `_layers` stays deliberately stale.
	#
	# **That staleness is the whole fixture.** It is what a second writer looks
	# like, and the only condition under which "an absent key means unchanged"
	# is observable at all: against a fresh cache a panel that restated every
	# value would send the values already there and nothing would show.
	# Mutation-tested -- with `ws._sync_layer_stack()` here instead, a
	# `_push_layer` that restated `visible`/`opacity`/`blend` SURVIVED.
	wg.reset_appearance()
	ws._sync_layer_stack()
	wg.set_layer_stack([
		{"id": "hillshade", "opacity": 0.4, "blend": "Screen"},
		{"id": "colour_relief"},
		{"id": "terrain"},
	])
	pushes = 0
	_buttons(_heads(ws._layer_host)[2])[0].pressed.emit()    # Terrain's dot
	var after: Array = bridge.layer_stack()
	_ok(pushes == 1, "one accepted write, one signal (got %d)" % pushes)
	_ok(not bool(_row(after, "terrain").get("visible", true)), "Terrain is hidden")
	_ok(abs(float(_row(after, "hillshade").get("opacity", -1.0)) - 0.4) < 1e-6,
		"a value the panel had never READ survived a gesture that never mentioned it")
	_ok(String(_row(after, "hillshade").get("blend", "")) == "Screen",
		"and so did the blend -- the write carried an id and one key, not a copy of the row")
	_ok(bool(_row(after, "colour_relief").get("visible", false)), "colour relief is untouched")

	print("\n== 2b. the blend picker and the opacity release ==")
	# Both of these free the very widget whose signal is mid-emission, because
	# an accepted write rebuilds the rows. Exercised rather than reasoned about:
	# the reorder buttons above do not cover them (`MISTAKES.md`, "covering some
	# inputs of a thing, not all of them").
	wg.reset_appearance()
	ws._sync_layer_stack()
	var picks: Array = _find(ws._layer_host, "OptionButton")
	_ok(picks.size() == 3, "one blend picker per row (got %d)" % picks.size())
	var modes: Array = bridge.blend_modes()
	_ok(modes.size() == 5, "the picker is built from the engine's own five modes (got %d)" % modes.size())
	## Each picker OPENS on its own row's mode. A picker stuck on the first
	## entry would claim every layer draws Normal while Hillshade multiplies --
	## the "a control that disagrees with its own engine" defect this register
	## keeps finding one row at a time.
	var opened: Array = []
	for pk in picks:
		opened.append(String(pk.get_item_text(pk.selected)) if pk.selected >= 0 else "<none>")
	_ok(opened == ["Multiply", "Normal", "Normal"],
		"each picker opens on ITS row's mode, not on the list's first entry: %s" % str(opened))
	# Row 0 is Hillshade, which opens on Multiply; drive it to Screen.
	picks[0].item_selected.emit(modes.find("Screen"))
	_ok(String(_row(bridge.layer_stack(), "hillshade").get("blend", "")) == "Screen",
		"picking a blend mode reached the engine")
	_ok(_find(ws._layer_host, "OptionButton").size() == 3,
		"the rows survived being rebuilt from inside the picker's own signal")

	var sliders: Array = _find(ws._layer_host, "HSlider")
	_ok(sliders.size() == 3, "one opacity slider per row (got %d)" % sliders.size())
	## The two signals are emitted by hand rather than by writing `.value`.
	## **Measured, not assumed:** a `Range` outside a live scene tree does not
	## emit `value_changed` from a code assignment at all, and
	## `SceneTree._initialize()` runs before the tree is live -- so setting
	## `.value` here would drive nothing and the assertion below would pass on
	## `1.0 == 1.0` without the handler ever running. Emitting is what the widget
	## itself does, and it is the wiring, not `Range`, that is under test.
	sliders[0].value_changed.emit(0.25)
	_ok(abs(float(_row(bridge.layer_stack(), "hillshade").get("opacity", -1.0)) - 1.0) < 1e-6,
		"dragging writes NOTHING -- a full-map re-render is not a per-tick operation")
	sliders[0].drag_ended.emit(true)
	_ok(abs(float(_row(bridge.layer_stack(), "hillshade").get("opacity", -1.0)) - 0.25) < 1e-6,
		"releasing the handle is what writes")
	_ok(String(_row(bridge.layer_stack(), "hillshade").get("blend", "")) == "Screen",
		"and the blend the opacity gesture never mentioned survived it")
	_ok(_find(ws._layer_host, "HSlider").size() == 3,
		"the rows survived being rebuilt from inside the slider's own release")

	print("\n== 3. reorder is DATA, and the rows follow the engine ==")
	wg.reset_appearance()
	ws._sync_layer_stack()
	_buttons(_heads(ws._layer_host)[0])[2].pressed.emit()    # top row, Down
	var moved := _ids(bridge.layer_stack())
	_ok(moved == ["colour_relief", "hillshade", "terrain"],
		"the ENGINE's order moved: %s" % str(moved))
	var redrawn: Array = []
	for h in _heads(ws._layer_host):
		for c in h.get_children():
			if c is Label:
				redrawn.append(String((c as Label).text))
				break
	_ok(redrawn == ["Colour relief", "Hillshade", "Terrain"],
		"the rows were rebuilt from the engine's answer, not reordered in place: %s" % str(redrawn))
	# ...and a move the stack cannot make changes nothing.
	var before_ids := _ids(bridge.layer_stack())
	ws._move_layer(0, 9)
	_ok(_ids(bridge.layer_stack()) == before_ids, "an out-of-range move is a no-op")

	print("\n== 3b. a row that arrived short is reported, not defaulted ==")
	# `get_layer_stack()` sets all five keys today, so this branch is
	# unreachable from a real engine and mutation testing scored it SURVIVED.
	# Driven directly instead of deleted: it is the degrade-not-crash contract
	# for a cdylib whose row shape differs, and an untested guard that invents
	# `false`/`0.0`/`"Normal"` is exactly the "encode no value as a plausible
	# value" defect this repository has recorded five times.
	var short_host := VBoxContainer.new()
	root.add_child(short_host)
	ws._layer_row(short_host, {"id": "terrain", "label": "Terrain"}, 0)
	var said := ""
	for l in _find(short_host, "Label"):
		said += String((l as Label).text)
	_ok(short_host.get_child_count() == 1, "a short row draws one note, not a control (got %d children)" % short_host.get_child_count())
	_ok(said.contains("visible") and said.contains("opacity") and said.contains("blend"),
		"the note names every key that was missing: %s" % said)
	_ok(_find(short_host, "HSlider").is_empty() and _find(short_host, "Button").is_empty(),
		"and it offers no control over a value it does not have")

	print("\n== 4. the right dock's appended section, and the two docks in step ==")
	wg.reset_appearance()
	ws._sync_layer_stack()
	var app: Node = load("res://shell/app.gd").new()
	app._active_domain = "cartography"
	var rd: Node = load("res://shell/right_dock.gd").new()
	rd.app = app
	rd.bridge = bridge
	var dock_body := VBoxContainer.new()
	root.add_child(dock_body)
	rd._append_layers(dock_body)
	var dock_heads := _heads_deep(dock_body)
	_ok(dock_heads.size() == 3, "the right dock drew three layer rows (got %d)" % dock_heads.size())
	_ok(_buttons(dock_heads[0]).size() == 3, "dot + Up + Down in the right dock too")
	var bars := 0
	for h in dock_heads:
		for c in h.get_children():
			if c is ProgressBar:
				bars += 1
	_ok(bars == 3, "section 6's opacity bar is drawn on every row (got %d)" % bars)

	# A right-dock write must reach the LEFT dock's rows, which is the whole
	# reason `layer_stack_changed` exists.
	rd._move_layer(0, 1)
	var both := _ids(bridge.layer_stack())
	_ok(both == ["colour_relief", "hillshade", "terrain"],
		"the right dock's reorder reached the engine: %s" % str(both))
	var synced: Array = []
	for h in _heads(ws._layer_host):
		for c in h.get_children():
			if c is Label:
				synced.append(String((c as Label).text))
				break
	_ok(synced == ["Colour relief", "Hillshade", "Terrain"],
		"the LEFT dock followed a write it did not make: %s" % str(synced))

	# The refusal branch. Both docks report it rather than assume success --
	# without a return value the branch's only effect is a log line, and
	# mutation testing scored `!= 3` SURVIVED in both files.
	var pristine := _ids(bridge.layer_stack())
	_ok(not rd._write_layers([{"id": "terrain"}]), "the right dock reports a refused stack")
	_ok(not ws._apply_layer_stack([{"id": "terrain"}, {"id": "terrain"}, {"id": "hillshade"}]),
		"the left dock reports a refused stack")
	_ok(_ids(bridge.layer_stack()) == pristine, "and both refusals left the stack alone")
	_ok(rd._write_layers([{"id": "terrain"}, {"id": "colour_relief"}, {"id": "hillshade"}]),
		"an accepted stack reports true")
	_ok(ws._apply_layer_stack([{"id": "hillshade"}, {"id": "colour_relief"}, {"id": "terrain"}]),
		"and the left dock reports an accepted stack true as well")

	# The same short-row guard the left dock has, in the right dock's own
	# `_layer_row` -- unreachable from a real engine, so driven directly.
	var short_dock := VBoxContainer.new()
	root.add_child(short_dock)
	rd._layer_row(short_dock, {"id": "terrain", "opacity": 1.0}, 0, 3)
	var dock_said := ""
	for l in _find(short_dock, "Label"):
		dock_said += String((l as Label).text)
	_ok(dock_said.contains("label") and dock_said.contains("visible") and dock_said.contains("blend"),
		"the right dock names every key that was missing: %s" % dock_said)
	_ok(_find(short_dock, "ProgressBar").is_empty() and _find(short_dock, "Button").is_empty(),
		"and draws no dot, bar or reorder over a row it could not read")

	# And the section is drawn only in CARTO.
	app._active_domain = "world"
	var elsewhere := VBoxContainer.new()
	root.add_child(elsewhere)
	rd._append_layers(elsewhere)
	_ok(elsewhere.get_child_count() == 0, "outside CARTO the section is not appended")

	print("\n== 5. an older cdylib loses the panel instead of erroring ==")
	var old: Node = load("res://shell/engine_bridge.gd").new()
	old._ready()
	old.layer_stack_api = false
	_ok(old.layer_stack() == [], "layer_stack() degrades to []")
	_ok(old.blend_modes() == [], "blend_modes() degrades to []")
	_ok(old.set_layer_stack([{"id": "terrain"}]) == 0, "set_layer_stack() degrades to 0")
	var ws2: Node = load("res://shell/workspaces/render_workspace.gd").new()
	ws2._nested = true
	ws2.bridge = old
	root.add_child(ws2)
	var host2 := VBoxContainer.new()
	root.add_child(host2)
	ws2.build_layer_stack_into(host2)
	_ok(host2.get_child_count() == 0, "the left dock draws no Layers section at all")
	var rd2: Node = load("res://shell/right_dock.gd").new()
	rd2.app = app
	rd2.bridge = old
	app._active_domain = "cartography"
	var body2 := VBoxContainer.new()
	root.add_child(body2)
	rd2._append_layers(body2)
	_ok(body2.get_child_count() == 0, "and neither does the right dock")

	wg.reset_appearance()
	print("")
	print("%s -- %d failure(s)" % ["ALL PASS" if fails == 0 else "FAILURES", fails])
	quit(1 if fails > 0 else 0)

## The right dock nests its rows one level deeper (`DccWidgets.section` returns
## the section's body), so this walks rather than reading direct children.
func _heads_deep(node: Node) -> Array:
	var out: Array = []
	for c in node.get_children():
		if c is HBoxContainer and _buttons(c).size() == 3:
			out.append(c)
		else:
			out.append_array(_heads_deep(c))
	return out

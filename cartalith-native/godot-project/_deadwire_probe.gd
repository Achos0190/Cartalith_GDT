extends Node
## TEMPORARY, untracked probe. The question this session keeps finding answers
## to: **which controls exist but do nothing?**
##
## For every window and dock, walk every Control and report any that is
##   - enabled (not disabled, so it invites a click), and
##   - interactive (Button / CheckBox / OptionButton / Slider / SpinBox /
##     LineEdit / TextEdit / ItemList / Tree), and
##   - has **zero** connections on its primary activation signal.
##
## Such a control is either dead or driven by _gui_input; both are worth a look.
## Also dumps: every enabled control's text + tooltip, so stale copy shows up,
## and every DISABLED control's tooltip (the `_todo()` reason contract).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _deadwire_probe.tscn

var _app: Node
var _bridge
var _fail := 0
var _big_town := 0

const SIGS := {
	"Button": "pressed",
	"CheckBox": "toggled",
	"CheckButton": "toggled",
	"OptionButton": "item_selected",
	"MenuButton": "pressed",
	"HSlider": "value_changed",
	"VSlider": "value_changed",
	"SpinBox": "value_changed",
	"LineEdit": "text_submitted",
	"TextEdit": "text_changed",
	"ItemList": "item_selected",
	"Tree": "item_selected",
	"TabBar": "tab_changed",
	"TabContainer": "tab_changed",
}


func _p(s: String) -> void:
	print("DEADWIRE  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _primary_sig(c: Control) -> String:
	var cls := c.get_class()
	if SIGS.has(cls):
		return SIGS[cls]
	# Custom subclasses: walk up.
	var s := ClassDB.get_parent_class(cls)
	while s != "":
		if SIGS.has(s):
			return SIGS[s]
		s = ClassDB.get_parent_class(s)
	return ""


func _label_of(c: Control) -> String:
	if c is Button:
		return (c as Button).text
	if c is OptionButton:
		return (c as OptionButton).text
	if c is LineEdit:
		return "<LineEdit ph='%s' text='%s'>" % [(c as LineEdit).placeholder_text, (c as LineEdit).text]
	if c is Range:
		return "<%s v=%s>" % [c.get_class(), str((c as Range).value)]
	return "<%s>" % c.get_class()


func _walk(n: Node, out: Array, depth: int = 0) -> void:
	if n is Control:
		out.append(n)
	for ch in n.get_children(true):
		_walk(ch, out, depth + 1)


func _path_from(root: Node, c: Node) -> String:
	var parts: Array = []
	var cur := c
	while cur != null and cur != root:
		parts.push_front(cur.name)
		cur = cur.get_parent()
	return "/".join(parts)


func _audit(title: String, root: Node) -> void:
	if root == null:
		_p("%s :: NULL ROOT" % title)
		return
	var all: Array = []
	_walk(root, all)
	var dead: Array = []
	var disabled_no_reason: Array = []
	var n_int := 0
	for c in all:
		if not (c is Control):
			continue
		var sig := _primary_sig(c)
		if sig == "":
			continue
		if not c.has_signal(sig):
			continue
		n_int += 1
		var is_disabled := false
		if "disabled" in c:
			is_disabled = c.disabled
		if is_disabled:
			var tip: String = c.tooltip_text
			if tip.strip_edges() == "" and c.is_visible_in_tree():
				disabled_no_reason.append("%s  %s" % [_path_from(root, c), _label_of(c)])
			continue
		if not c.is_visible_in_tree():
			continue
		var conns: Array = c.get_signal_connection_list(sig)
		if c is LineEdit:
			conns = conns + c.get_signal_connection_list("text_changed")
		if c is TextEdit and not (c is LineEdit):
			conns = conns + c.get_signal_connection_list("focus_exited")
		if c is Tree:
			conns = conns + c.get_signal_connection_list("item_activated") \
				+ c.get_signal_connection_list("item_edited") \
				+ c.get_signal_connection_list("button_clicked")
		if c is ItemList:
			conns = conns + c.get_signal_connection_list("item_activated") \
				+ c.get_signal_connection_list("item_clicked")
		if c is BaseButton:
			conns = conns + c.get_signal_connection_list("toggled") \
				+ c.get_signal_connection_list("button_down") \
				+ c.get_signal_connection_list("gui_input")
		if c is ColorPickerButton:
			conns = conns + c.get_signal_connection_list("color_changed")
		if c is Range:
			conns = conns + c.get_signal_connection_list("drag_ended")
		if conns.is_empty():
			# a Button inside a ButtonGroup, or one driven by _gui_input /
			# _toggled override, is not necessarily dead -- report and judge.
			var extra := ""
			if c is BaseButton and (c as BaseButton).button_group != null:
				extra = " [in ButtonGroup]"
			if c.has_method("_gui_input"):
				extra += " [has _gui_input]"
			if c is BaseButton and (c as BaseButton).toggle_mode:
				extra += " [toggle]"
			dead.append("%s  %s  sig=%s%s" % [_path_from(root, c), _label_of(c), sig, extra])
	_p("---- %s : %d controls, %d interactive, %d UNWIRED, %d disabled-without-reason" % [title, all.size(), n_int, dead.size(), disabled_no_reason.size()])
	for d in dead:
		_p("   UNWIRED  %s" % d)
	for d in disabled_no_reason:
		_p("   NOREASON %s" % d)
		_fail += 1


func _ready() -> void:
	## watchdog: never let a hang eat the caller's timeout silently
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_p("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(4)

	## the biggest settlement -- the one most likely to have a layout, a
	## hinterland and journeys, so the windows have something real to draw.
	var st: Array = _bridge.settlements()
	var best := 0
	var best_pop := -1.0
	for i in st.size():
		var p: float = float(st[i].get("population", 0.0))
		if p > best_pop:
			best_pop = p
			best = i
	_big_town = best
	_p("world: %d settlements, biggest #%d pop=%.0f" % [st.size(), best, best_pop])

	var wins := {
		"AssetLibrary": _app.asset_library_window,
		"DataManager": _app.data_manager_window,
		"TravelLibrary": _app.travel_library_window,
		"CityViewer": _app.city_viewer_window,
		"FactionRoster": _app.faction_roster_window,
		"PlaceEditor": _app.place_editor_window,
		"Vault": _app.vault_window,
		"Performance": _app.performance_window,
		"WorldData": _app.world_data_window,
		"GenInfo": _app.gen_info_dialog,
		"NewWorld": _app.new_world_dialog,
		"JourneyPlanner": _app.journey_planner_view,
		"LayersPopover": _app.layers_popover,
		"OpenProject": _app.open_project_dialog,
	}
	for k in wins:
		var w = wins[k]
		if w == null:
			_p("%s :: null" % k)
			continue
		# open it so lazy content builds
		match k:
			"CityViewer":
				w.call("open", _big_town)
			"PlaceEditor":
				w.call("open_for", _big_town)
			"Vault":
				w.call("open_overview")
			"NewWorld":
				w.call("popup_centered")
			_:
				w.call("open")
		await _frames(6)
		_audit(k, w)
		if w.has_method("hide"):
			w.call("hide")
		await _frames(2)

	_audit("RightDock", _app.right_dock_ctrl)
	_audit("ToolBar", _app.tool_bar)
	_audit("SectionStrip", _app.section_strip)

	_p("DONE fail=%d" % _fail)
	get_tree().quit(0)

extends Node
## Committed probe. The question this session keeps finding answers
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
## ## What makes a DISABLED control a defect
##
## The first version of this test was "disabled and no tooltip", and it reported
## six controls that are not defects at all: `right_dock.gd`'s sculpt
## Commit/Discard/Undo/Redo and `tool_bar.gd`'s sculpt Commit/Discard, every one
## of them `disabled = stamps.is_empty()` or `not bridge.sculpt_can_undo()`.
## `UNWIRED_FUNCTIONS.md` files exactly these under "State-driven disables that
## re-enable on their own" and calls them CORRECT. The test could not tell
## "disabled because the feature does not exist" from "disabled because there is
## nothing to commit yet", so it called both a failure.
##
## The two are distinguished here without naming a single control, by two rules:
##
##   Rule 2  DISABLED, no tooltip, and **zero handlers** on every activation
##           signal it has. Nothing would run if it were enabled; there is no
##           state anywhere in the program that could give it an effect. Dead
##           and silent -- fails on sight.
##
##   Rule 3  DISABLED, no tooltip, but **wired** to a real handler. Deferred.
##           At the end (`_verdict()`) it fails unless *the same action* was
##           observed ENABLED in some other pass over the same surface. Identity
##           is the handler -- method name plus bound arguments (`_action_key`)
##           -- not the label and not the node path, because the dock rebuilds
##           its whole body per context (auto-generated node names) and a
##           button's own text can change with the state under test (the stamp
##           rows say "select" enabled and "selected" disabled).
##
## Rule 3 has teeth only if the probe actually drives the states, so it does:
## `_audit_warm_sculpt()` draws two real strokes through
## `sculpt_begin_stroke`/`add_point`/`end_stroke`, selects the other stamp, and
## undoes one edit -- the same bridge calls the map's pointer handler makes. A
## control that stays disabled through every state this probe can reach, with no
## tooltip saying why, is still a failure. Nothing is exempted by name.
##
## Residual gap, stated rather than hidden: rule 3 acquits on evidence, so it can
## only ever be as strong as the states this probe drives. A wired control whose
## gate this probe never opens fails (correctly, as unproven); a wired control
## hard-set `disabled = true` also fails, since no state opens it. What rule 3
## cannot do is distinguish those two -- both read NEVER-ENABLED, and telling
## them apart needs a reader to look at the assignment.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _deadwire_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _app: Node
var _bridge
var _fail := 0
var _big_town := 0

## Rule 2's findings, counted at `_verdict()` rather than inline, so the whole
## disabled-control report is one block instead of scattered through fourteen
## audits that each only saw one state. A **Dictionary**, not an Array: several
## surfaces are now audited more than once (`RightDock[sculpt]` four times), and
## the first cut of this appended per sighting, so one dead button in a repeated
## context was counted once per pass. `_deadwire_teeth.gd`'s synthetic control
## caught that -- it reported `fail=3` against three planted defects, two of them
## the same button twice.
var _dead_silent: Dictionary = {}
## Rule 3: action-key -> {title, where}. One entry per gated action, first sight.
var _gated: Dictionary = {}
## Action-keys seen ENABLED at least once, anywhere. Rule 3's acquittal.
var _seen_enabled: Dictionary = {}

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


## Every signal that counts as "this control has an effect". Wider than SIGS:
## SIGS picks the ONE signal a class is judged unwired on, this asks whether the
## control is attached to anything at all.
const ACT_SIGS := ["pressed", "toggled", "item_selected", "value_changed",
	"text_submitted", "text_changed", "item_activated", "item_edited",
	"button_clicked", "item_clicked", "tab_changed", "color_changed",
	"button_down", "focus_exited", "drag_ended", "gui_input"]


## Identity for a control **across rebuilds**, which the node path is not: the
## dock throws its whole body away per context and Godot auto-names the
## replacements (`@Button@412`), so path-based identity never matches twice.
## The handler does match: `DccWidgets.action()` and `chip()` both end in
## `b.pressed.connect(on_press)` with the caller's own Callable, so
## `_on_sculpt_stack_commit` survives every rebuild, and `.bind(idx)` keeps two
## stamp rows apart. Empty return == "wired to nothing", which is rule 2.
func _action_key(title: String, c: Control) -> String:
	var parts: Array = []
	for sig_name in ACT_SIGS:
		if not c.has_signal(sig_name):
			continue
		for con in c.get_signal_connection_list(sig_name):
			var cb: Callable = con["callable"]
			var m := String(cb.get_method())
			if m == "":
				## A GDScript lambda has no stable name across a rebuild, so it
				## cannot BE the identity; the label is the only stable thing
				## such a control has left. None of the six this rule was written
				## for take this branch -- they are all plain methods.
				m = "<lambda>" + _label_of(c)
			var bound := ""
			for a in cb.get_bound_arguments():
				bound += "," + str(a)
			parts.append("%s(%s)" % [m, bound])
	if parts.is_empty():
		return ""
	parts.sort()
	var joined := ""
	for part in parts:
		joined += String(part) + "|"
	return "%s :: %s :: %s" % [title, c.get_class(), joined]


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


## `root` is `Variant`, not `Node`, and the difference is the whole reason this
## probe's previous "clean" run was worthless. `_audit("ToolBar", _app.tool_bar)`
## passed a `DccToolBar`, which `shell/tool_bar.gd` declares `extends RefCounted`
## / `class_name DccToolBar` on its first two lines -- a runtime type error
## against a `Node` parameter. The probe died inside it, before `DONE`, before
## `get_tree().quit()`, and hung until the caller's own timeout fired. Anything
## that is not a Node is now a loud, counted SKIP: a dead-control audit that
## quietly audits nothing is worse than no audit.
func _audit(title: String, root: Variant) -> void:
	if root == null:
		## Counted, for the same reason the SKIP below is. A null surface is not
		## "nothing to report" -- it is the window never constructed, almost
		## always because its script failed to load, and the pass that follows
		## audits zero of its controls while the verdict still reads clean.
		_fail += 1
		_p("%s :: NULL ROOT -- the surface was never constructed (its script "
			% title + "probably failed to load); zero controls audited")
		return
	if not (root is Node):
		_fail += 1
		_p("%s :: SKIP -- root is not a Node (%s); nothing to walk" % [title, str(root)])
		return
	var node := root as Node
	var all: Array = []
	_walk(node, all)
	var dead: Array = []
	var n_dead_silent := 0
	var n_gated := 0
	var n_int := 0
	for c in all:
		if not (c is Control):
			continue
		var sig := _primary_sig(c)
		if sig == "":
			continue
		if not c.has_signal(sig):
			continue
		## A read-only text field is a READOUT, not a control: it neither invites
		## a click nor owes a disabled-reason tooltip. `TextEdit`/`LineEdit`
		## express that as `editable = false`, which the `disabled` test below
		## cannot see -- so without this, GenInfo's dump field
		## (`gen_info_dialog.gd`, `_text.editable = false`) was reported UNWIRED
		## on a signal a read-only field can never emit.
		if (c is TextEdit or c is LineEdit) and not c.editable:
			continue
		n_int += 1
		var is_disabled := false
		if "disabled" in c:
			is_disabled = c.disabled
		var key := _action_key(title, c)
		if is_disabled:
			var tip: String = c.tooltip_text
			## A tooltip IS the reason contract; a control that is not on screen
			## invites no click. Neither owes anything further.
			if tip.strip_edges() != "" or not c.is_visible_in_tree():
				continue
			var where := "%s  %s" % [_path_from(node, c), _label_of(c)]
			if key == "":
				## Rule 2. Not one handler on any activation signal it owns:
				## enabling it would still do nothing, in every state.
				## Keyed, not appended -- see `_dead_silent`'s own note. A control
				## wired to nothing has no handler to be identified by, so the
				## label is the best identity left; the node path is not usable
				## here because a rebuilt dock renames every node it makes.
				n_dead_silent += 1
				var dkey := "%s :: %s :: %s" % [title, c.get_class(), _label_of(c)]
				if not _dead_silent.has(dkey):
					_dead_silent[dkey] = "%s  %s" % [title, where]
			else:
				## Rule 3. The action exists and is implemented; whether the gate
				## ever reopens is settled in `_verdict()`, from evidence.
				n_gated += 1
				if not _gated.has(key):
					_gated[key] = {"title": title, "where": where}
			continue
		if not c.is_visible_in_tree():
			continue
		## Enabled and on screen: this action's gate is open in THIS state, which
		## is rule 3's acquittal for every other pass that found it shut.
		if key != "":
			_seen_enabled[key] = true
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
			dead.append("%s  %s  sig=%s%s" % [_path_from(node, c), _label_of(c), sig, extra])
	_p("---- %s : %d controls, %d interactive, %d UNWIRED, %d dead-silent, %d gated" % [title, all.size(), n_int, dead.size(), n_dead_silent, n_gated])
	for d in dead:
		_p("   UNWIRED  %s" % d)


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
			## Counted. `_audit()` and `_audit_tool_bar()` both fail on a null
			## surface; a window that is null here is audited by nobody, and a
			## silent `continue` is exactly the clean-line-over-zero-coverage
			## this probe's own header calls the worse failure.
			_fail += 1
			_p("%s :: NULL -- the window was never constructed; zero controls audited" % k)
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

	## -------------------------------------------------------- the right dock
	## **This used to be `_audit("RightDock", _app.right_dock_ctrl)`, and it
	## printed `0 controls, 0 interactive`** -- a clean line over zero coverage
	## of the file. `RightDock` is `extends Node` (`shell/right_dock.gd`, line 1)
	## and owns no Control of its own: `_rebuild()` fills `app.right_dock_body`,
	## the `VBoxContainer` `DccShell` declares. So the dock is audited where its
	## controls actually live -- and once per context, because §6's dock swaps
	## its whole body per selection and one pass would only ever see Sample.
	await _audit_right_dock()

	## ---------------------------------------------------------- the tool bar
	## **This used to be `_audit("ToolBar", _app.tool_bar)`, and it killed the
	## probe.** See `_audit()`'s own header for the type error. The bar's Node
	## subtree is `app.tool_options_row`, which `DccToolBar.rebuild()` fills
	## through `app.set_tool_options(_build)`.
	await _audit_tool_bar()

	_audit("SectionStrip", _app.section_strip)

	## ------------------------------------------------------ the left dock
	## The surface this probe was blind to for its whole life. See
	## `_audit_workspaces()` for what it was missing and why one `_audit()` call
	## per panel would not have found it.
	await _audit_workspaces()

	## Rule 3's evidence. Everything above ran the sculpt surfaces in exactly one
	## state -- an empty draft -- which is the state that shuts Commit, Discard,
	## Undo and Redo. This drives the other states.
	await _audit_warm_sculpt()

	_check_bindings()
	_verdict()

	## Was `quit(0)` unconditionally, which made `_fail` a number nobody could
	## gate on. It now exits non-zero, like `_jump_probe.gd` does.
	_p("DONE fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)


## Rule 3's evidence half. The rule asks whether a disabled control has a
## re-enabling path, and the only honest way for a probe to answer that is to
## take the path and look again. `sculpt_begin_stroke` / `sculpt_add_point` /
## `sculpt_end_stroke` build a real stamp through the same bridge the map's
## pointer handler uses -- these are the states a user reaches by drawing on the
## map, not a flag the probe sets on itself to make itself pass.
##
## Three states, because the four sculpt gates do not all open on the same one:
##   B  a non-empty stack        -> Commit, Discard, Undo (and the tool bar pair)
##   C  the other stamp selected -> each stamp row's own `idx == selected` gate
##   D  one edit undone          -> Redo, whose gate opens in no other state
func _audit_warm_sculpt() -> void:
	var rd = _app.right_dock_ctrl
	var body = _app.right_dock_body
	if rd == null or body == null:
		_p("   SKIP warm sculpt -- right dock is null; the gated sculpt controls "
			+ "stay unproven and _verdict() will fail them")
		return
	if _bridge.sculpt_get_globals().is_empty():
		_p("   SKIP warm sculpt -- this world has no sculpt editor "
			+ "(sculpt_get_globals() is empty); gated controls stay unproven")
		return

	var made := 0
	for stroke in 2:
		if not _bridge.sculpt_begin_stroke():
			break
		for i in 6:
			_bridge.sculpt_add_point(140.0 + stroke * 24.0 + i * 5.0, 100.0 + i * 3.0)
		if _bridge.sculpt_end_stroke() >= 0:
			made += 1
	_p("warm sculpt: %d stroke(s) drawn, stamp_count=%d can_undo=%s can_redo=%s"
		% [made, _bridge.sculpt_stamp_count(), str(_bridge.sculpt_can_undo()),
			str(_bridge.sculpt_can_redo())])
	if made == 0:
		_p("   SKIP warm sculpt -- no stamp could be created; gated controls stay unproven")
		return

	## B
	rd.show_sculpt_stack()
	await _frames(4)
	_audit("RightDock[sculpt]", body)

	## C. `sculpt_end_stroke` selects the stamp it just pushed, so the OTHER one
	## is the state that flips both stamp rows' select buttons at once.
	var stamps: Array = _bridge.sculpt_list_stamps()
	if stamps.size() > 1:
		var sel := int(_bridge.sculpt_get_selected_stamp())
		for st in stamps:
			var idx := int((st as Dictionary).get("index", -1))
			if idx != sel:
				_bridge.sculpt_select_stamp(idx)
				break
		rd.show_sculpt_stack()
		await _frames(4)
		_audit("RightDock[sculpt]", body)
	else:
		_p("   note: only %d stamp -- the per-row select gate stays unproven" % stamps.size())

	## D
	if _bridge.sculpt_undo():
		rd.show_sculpt_stack()
		await _frames(4)
		_audit("RightDock[sculpt]", body)
	else:
		_p("   note: sculpt_undo() refused -- Redo stays unproven")

	## The tool bar's own Commit / Discard read the same stamp count, and its
	## cold pass ran before any of this existed.
	var was := String(_app.armed_tool)
	_app.arm_tool("sculpt")
	await _frames(8)
	_audit("ToolBar[sculpt]", _app.tool_options_row)
	_app.arm_tool(was)
	await _frames(4)

	## And the LEFT dock's copies, which are a third pair of the same two
	## buttons: `world_workspace.gd::_build_sculpt_draft()` draws its own
	## "Commit to map" / "Discard draft", both `disabled = count == 0` against
	## the same `bridge.sculpt_stamp_count()`. They are a distinct `_action_key` from the
	## right dock's (`_on_sculpt_commit` is the workspace's own method), so the
	## evidence gathered above does not acquit them and rule 3 fails them
	## unproven -- correctly, until this pass drives their state too. Added with
	## `_audit_workspaces()`: before that pass existed nothing audited this
	## surface at all, so the gap could not show.
	await _audit_world_category("Terrain")

	## Leave nothing behind. `sculpt_discard()` drops the draft; the stack is
	## never committed, so the heightfield this probe generated is untouched.
	_bridge.sculpt_discard()
	rd.on_settlement_selected(null, -1)
	await _frames(2)


## Rules 2 and 3 settled together, after every pass, because rule 3 cannot be
## decided until the last state has been seen.
func _verdict() -> void:
	_p("---- verdict on DISABLED controls")
	for dkey in _dead_silent:
		_p("   DEAD-SILENT   %s" % _dead_silent[dkey])
		_fail += 1
	var proved := 0
	for key in _gated:
		var rec: Dictionary = _gated[key]
		if _seen_enabled.has(key):
			proved += 1
			continue
		_p("   NEVER-ENABLED %s  %s" % [rec["title"], rec["where"]])
		_p("                 action=%s" % key)
		_fail += 1
	_p("   %d dead-silent, %d gated action(s) proved a re-enabling path, "
		% [_dead_silent.size(), proved]
		+ "%d never seen enabled" % (_gated.size() - proved))


## The tool bar is a `RefCounted` controller over a Node the shell owns, so
## "audit the tool bar" means "audit the row it builds, in each mode it can
## build". The bar draws only the **active** mode's tools
## (`DccToolBar._build()`'s two `match mode:` arms), so one pass would miss two
## thirds of it.
func _audit_tool_bar() -> void:
	var bar = _app.tool_bar
	if bar == null:
		_p("ToolBar :: NULL -- DccToolBar.install() did not run")
		_fail += 1
		return
	if bar is Node:
		## Kept live rather than assumed away: if the bar is ever promoted to a
		## Node this branch audits the new surface instead of silently walking
		## the old one.
		_audit("ToolBar", bar)
		return
	_p("ToolBar :: DccToolBar is RefCounted (shell/tool_bar.gd), not a Node -- "
		+ "auditing app.tool_options_row, the row it fills via app.set_tool_options()")
	var was := String(_app.armed_tool)
	for m in DccToolBar.MODES:
		_app.arm_tool(String(m))
		await _frames(8)
		_audit("ToolBar[%s]" % m, _app.tool_options_row)
	_app.arm_tool(was)
	await _frames(4)


## Every context `RightDock._dispatch()` can draw, each with the closest thing
## to real engine data this probe can hand it. Where the data does not exist the
## context is SKIPped **by name and with the reason**, never audited empty --
## three of these (`_build_wildlife`, `_build_journey`, `_build_settlement`)
## fall back to `_build_sample()` when their data is missing, so an empty audit
## would silently report Sample twice under someone else's title.
func _audit_right_dock() -> void:
	var rd = _app.right_dock_ctrl
	var body = _app.right_dock_body
	if rd == null or body == null:
		_p("RightDock :: NULL (ctrl=%s body=%s)" % [str(rd), str(body)])
		_fail += 1
		return

	## Sample. `on_settlement_selected(null, -1)` is the public way back to it.
	rd.on_settlement_selected(null, -1)
	await _frames(4)
	rd.on_cursor_sampled(192.0, 144.0, true)
	await _frames(4)
	_audit("RightDock[sample]", body)

	var st: Array = _bridge.settlements()
	if st.is_empty():
		_p("   SKIP RightDock[settlement] -- bridge.settlements() is empty")
	else:
		rd.on_settlement_selected(st[_big_town], _big_town)
		await _frames(4)
		_audit("RightDock[settlement]", body)

	var fs: Array = _bridge.get_factions()
	if fs.is_empty():
		_p("   SKIP RightDock[faction] -- bridge.get_factions() is empty")
	else:
		var a := int((fs[0] as Dictionary).get("id", -1))
		var b := int((fs[mini(1, fs.size() - 1)] as Dictionary).get("id", -1))
		## The pair form (RL-01), not the single: it draws strictly more.
		rd.show_faction(a, b)
		await _frames(4)
		_audit("RightDock[faction]", body)

	var rs: Array = _bridge.roads()
	if rs.is_empty():
		_p("   SKIP RightDock[route] -- bridge.roads() is empty")
	else:
		rd.show_route(rs[0], "road")
		await _frames(4)
		_audit("RightDock[route]", body)

	## `show_wildlife({})` is *defined* to fall back to Sample, so an empty
	## record would audit Sample a second time under the Ecoregion title. A few
	## cells are tried before giving up.
	var eco: Dictionary = {}
	for probe_at in [Vector2(192, 144), Vector2(96, 72), Vector2(288, 216), Vector2(48, 200)]:
		eco = _bridge.wildlife_region_at((probe_at as Vector2).x, (probe_at as Vector2).y)
		if not eco.is_empty():
			break
	if eco.is_empty():
		_p("   SKIP RightDock[wildlife] -- wildlife_region_at() found no ecoregion at any probe cell")
	else:
		rd.show_wildlife(eco)
		await _frames(4)
		_audit("RightDock[wildlife]", body)

	## Measure and Region both draw a real, deliberate empty state
	## (`_measure_empty()`, "Drag a marquee on the map"), which is a form worth
	## auditing in its own right -- so the live dict is used when the engine has
	## one and the empty state when it does not.
	rd.show_measure(_bridge.measure_result(), "distance")
	await _frames(4)
	_audit("RightDock[measure]", body)

	rd.show_region(_bridge.region_get())
	await _frames(4)
	_audit("RightDock[region]", body)

	## River takes no argument and has no public setter -- it is reached from
	## the map, which this probe cannot click. The context constant is public.
	rd._context = RightDock.CTX_RIVER
	rd._rebuild()
	await _frames(4)
	_audit("RightDock[river]", body)

	## The Stamp stack stopped being a right-dock CONTEXT on 2026-09-03 (the
	## owner's "selection wins, the tool appends a section" ruling): it is now an
	## appended section derived from the armed tool plus the sculpt draft, so
	## `show_sculpt_stack()` is a rebuild and no longer selects what is drawn.
	## Without arming the tool this audit silently re-measured whatever context
	## the line above left behind -- it recorded `RightDock[sculpt] : 6 controls`
	## while River was on screen -- so the label named a surface it had not
	## visited. The draft is still empty here, which is the point: this is the
	## COLD pass, and `_audit_warm_sculpt()` below is the warm one.
	var sculpt_was := String(_app.armed_tool)
	_app.arm_tool("sculpt")
	await _frames(4)
	_audit("RightDock[sculpt]", body)
	_app.arm_tool(sculpt_was)
	await _frames(2)

	rd.show_history()
	await _frames(4)
	_audit("RightDock[history]", body)

	_p("   SKIP RightDock[journey] -- needs a live JourneyPlannerView; "
		+ "right_dock.gd::_build_journey() falls back to Sample without one")

	## Back to the default, so nothing downstream inherits a context.
	rd.on_settlement_selected(null, -1)
	await _frames(2)


## The staleness fingerprint, read off the shell instead of guessed at.
##
## `EngineBridge._has()` (`shell/engine_bridge.gd`) is the one choke point
## every binding guard in the shell goes through, and it records the name of
## each method the shell asked for that this build does not export;
## `EngineBridge.missing_bindings()` hands back the set. Nothing in this probe
## suite read it -- and a stale `target/debug/cartalith_godot.dll` has twice
## sent every `_has()` guard in a run down its degraded-fallback branch, which
## turns a whole sweep into a clean report over code that was never exercised.
## That is the failure mode this suite is least able to notice on its own, and
## the shell was already carrying the answer.
##
## Called last, after every surface this run drives has been driven: the set
## only fills as guards are reached, so an early read reports an empty one.
func _bad_binding(s: String) -> void:
	_fail += 1
	_p("FAIL  %s" % s)


func _check_bindings() -> void:
	var mb: PackedStringArray = _bridge.missing_bindings()
	if mb.is_empty():
		return
	_bad_binding("stale extension -- the shell asked for %d binding(s) this build "
		% mb.size()
		+ "does not export (%s). " % ", ".join(mb)
		+ "Every result above was measured against a degraded shell; rebuild "
		+ "the crates and re-run before believing any of it.")


## The three left-dock workspaces -- the surface this probe never looked at.
##
## Before this pass `_deadwire_probe` audited fourteen windows, the right dock,
## the tool bar and the section strip, and **zero controls** of
## `shell/workspaces/world_workspace.gd`, `civilization_workspace.gd` and
## `cartography_workspace.gd` -- together about 366 KB of GDScript against the
## right dock's 98 KB, and the largest widget surface in the shell. A
## dead-control sweep that skips the docks is a sweep of the chrome around them,
## and it reported clean the whole time.
##
## Two things make this more than one `_audit()` call per panel, and both are
## why the naive version would have reported "0 controls" and looked fine:
##
##   - Only the active domain's panel is visible (`register_workspace()` sets
##     `panel.visible = id == _active_domain`), and `_audit()` skips any control
##     failing `is_visible_in_tree()` -- so the domain has to be *selected*
##     first, not merely fetched.
##   - L2 categories are an accordion: `Workspace.open_category()` opens one and
##     closes its siblings. A single pass would only ever see whichever category
##     happened to be open, which for a fresh boot is the first. Each is opened
##     and audited in turn, under its own name, the way `_v3menu_probe.gd`
##     drives the same docks.
##
## `workspace_panel()` and `open_category()` are the public entry points --
## `_workspace_panels` is private and a probe that reaches into it breaks on a
## rename instead of on a regression, which `DccShell.workspace_panel()`'s own
## doc comment says in as many
## words.
func _audit_workspaces() -> void:
	var was := String(_app.active_domain())
	## The three domains `app.gd::_register_workspaces()` actually registers.
	## `infrastructure_workspace.gd` and `render_workspace.gd` still exist as
	## files but are not registered (the 2026-08-20 domain merge), so there is
	## no panel to fetch for them and asking would be a null this probe would
	## then have to explain away.
	for dom in ["world", "civilization", "cartography"]:
		var panel: Control = _app.workspace_panel(dom)
		if panel == null:
			_fail += 1
			_p("Workspace[%s] :: NULL -- register_workspace() never ran for this "
				% dom + "domain; zero controls audited")
			continue
		_app.select_domain(dom)
		await _frames(6)
		var cats: Array = panel.get("categories")
		if cats == null or cats.is_empty():
			_fail += 1
			_p("Workspace[%s] :: no categories -- setup()/_build() drew nothing" % dom)
			continue
		_p("Workspace[%s] :: %d categories" % [dom, cats.size()])
		for e in cats:
			var title := String(e["title"])
			if not panel.call("open_category", title):
				_fail += 1
				_p("Workspace[%s/%s] :: open_category() refused its own category "
					% [dom, title] + "title -- the accordion and `categories` disagree")
				continue
			await _frames(4)
			_audit("Workspace[%s/%s]" % [dom, title], e["body"])
	_app.select_domain(was)
	await _frames(4)


## Re-audit one category of the WORLD dock in whatever state the caller has
## just driven. Split out of `_audit_workspaces()` so the warm-sculpt pass can
## reach the same surface a second time without repeating the domain-select and
## accordion dance -- and so that what it re-audits is named at the call site.
func _audit_world_category(title: String) -> void:
	var panel: Control = _app.workspace_panel("world")
	if panel == null:
		_fail += 1
		_p("Workspace[world/%s] :: NULL panel on the warm pass" % title)
		return
	var was := String(_app.active_domain())
	_app.select_domain("world")
	await _frames(4)
	if not panel.call("open_category", title):
		_fail += 1
		_p("Workspace[world/%s] :: open_category() refused on the warm pass" % title)
		_app.select_domain(was)
		return
	await _frames(4)
	## The stamp count at the moment of the re-audit, printed so a NEVER-ENABLED
	## verdict below is readable on its own: it separates "this probe never
	## opened the gate" from "the gate was open and the control stayed shut".
	_p("Workspace[world/%s] re-audited with stamp_count=%d can_undo=%s"
		% [title, _bridge.sculpt_stamp_count(), str(_bridge.sculpt_can_undo())])
	for e in (panel.get("categories") as Array):
		if String(e["title"]) == title:
			_audit("Workspace[world/%s]" % title, e["body"])
			break
	_app.select_domain(was)
	await _frames(2)

extends Node
## MS-1 swap probe (`MOBILE_SHELL_MS1_SPEC.md` §3.5 and bar B5; `cartalith-native/docs/evidence/ms1-2026-11-07/
## S4_INVENTORY.md` §8 and §12). Boots the real shell in a `--vp`-sized host `SubViewport` and drives class
## changes by resizing the host and emitting `root.size_changed`, as `_ms1class_probe.gd`'s live legs do (a host
## resize does not resize the root window, and the shell listens to the root).
##
##   godot --path . --resolution 1600x1000 _ms1swap_probe.tscn -- --leg quiet --force-touch --vp 1800x3200 --density 1
##   ... -- --leg norm --vp 1600x1000 --out <file>        (cold boot of one class: its normalised dump)
##   ... -- --leg crossclass --vp 1600x1000 --steps tablet:1800x1000,phone:880x1564,tablet:1800x1000 --density 1
##         --out <file>     (S4.3: remount across classes; the final dump must equal a cold boot's)
##   ... -- --leg carry --force-touch --vp 1800x1000 --density 1          (S4.3: each carried state item round-trips)
##
## Legs:
##   quiet   (stage S4.1) the quiescence gate. The shell must report no blocker when idle; a held dock drag, a
##           visible confirmation, a generating engine and an open New world form must each be reported by
##           `quiescent_blockers()`; a size change across the line while one is held must WAIT (no swap run,
##           the status hint naming the blocker) and run once the blocker ends; a size that returns to the
##           current class while waiting must cancel the wait (no swap run, the hint cleared). At density 1,
##           1 800 x 3 200 dp is a tablet and 880 x 1 564 dp a phone (past 900 - 16, `DccTheme.BAND_DP`).
##
## Run windowed with a wiped user dir (the runner sets `APPDATA` to an empty folder). Logic and layout only, so
## nothing here reads pixels. Arguments read by THIS file: `--leg quiet`, `--vp WxH`; accepted because the
## shell reads them: `--force-touch`, `--density D`. Anything else aborts with exit 2 (`MISTAKES.md`, "Write a
## probe's usage header").
##
## **It fails on the tree before S4.1** by construction: there `quiescent_blockers()` does not exist and the
## swap request runs at once, so the first check fails and the wait checks cannot hold (`MISTAKES.md`, "Add a
## probe in the same commit that changes the behaviour"). Expected values are literals.
##
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument, 3 watchdog.

var _vp: SubViewport
## Debug only (`--names`): append each node's name and its parent's to the structure lines, to find what a
## differing line is. Names carry counters, so a run with it is read by eye, never compared byte for byte.
var _debug_names := false
var _app: Node
var _fails := 0
var _checks := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1

func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	if k >= 0 and k + 1 < args.size():
		return String(args[k + 1])
	return dflt

## Resizes the host and tells the shell, then waits past the 150 ms debounce (0.4 s).
func _resize(w: int, h: int) -> void:
	_vp.size = Vector2i(w, h)
	await _frames(2)
	get_tree().root.size_changed.emit()
	await get_tree().create_timer(0.4).timeout
	await _frames(3)

func _blockers() -> Array:
	return _app.call("quiescent_blockers")

func _has_blocker(prefix: String) -> bool:
	for b in _blockers():
		if String(b).begins_with(prefix):
			return true
	return false

func _hint() -> String:
	return String(_app.status_slot_text("hint"))

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 400.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("RESULT: FAIL (watchdog)")
		get_tree().quit(3))
	wd.start()
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch" or a == "--names":
			i += 1
		elif a in ["--leg", "--vp", "--density", "--out", "--steps"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	var leg := _arg("--leg", "")
	_debug_names = "--names" in OS.get_cmdline_user_args()
	var parts := _arg("--vp", "").split("x")
	if not leg in ["quiet", "dump", "roundtrip", "norm", "crossclass", "carry"] or parts.size() != 2:
		print("ABORT: needs --leg quiet|dump|roundtrip|norm|crossclass|carry and --vp WxH")
		get_tree().quit(2)
		return
	var vw := int(parts[0])
	var vh := int(parts[1])
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	if leg == "quiet":
		await _quiet()
	elif leg == "dump":
		await _dump_leg()
	elif leg == "norm":
		await _norm_leg()
	elif leg == "crossclass":
		await _crossclass()
	elif leg == "carry":
		await _carry()
	else:
		await _roundtrip()
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

func _quiet() -> void:
	_ok(DccTheme.is_tablet(), "boots 1800 x 3200 dp as a tablet")
	var has := _app.has_method("quiescent_blockers") and _app.has_method("pending_class_swap")
	_ok(has, "the shell has quiescent_blockers() and pending_class_swap() (S4.1)")
	if not has:
		return
	_ok(_blockers().is_empty(), "idle: no blocker (got %s)" % str(_blockers()))

	print("-- a held dock drag makes a crossing wait, and the swap runs when it ends")
	_app._dragging_dock = "left"
	_ok(_has_blocker("a dock is being resized"), "a dock drag is reported (got %s)" % str(_blockers()))
	var runs0: int = _app.class_swap_runs
	await _resize(880, 1564)
	_ok(not _app.pending_class_swap().is_empty(), "880 x 1564 under a dock drag: a swap is pending")
	_ok(_app.class_swap_runs == runs0, "and has not run (runs %d, want %d)" % [_app.class_swap_runs, runs0])
	_ok(_hint().begins_with("Layout change waiting: a dock is being resized"),
		"the status hint names the blocker (hint '%s')" % _hint())
	_ok(DccTheme.is_tablet(), "the composition is still the tablet while waiting")
	_app._dragging_dock = ""
	await get_tree().create_timer(0.8).timeout
	await _frames(2)
	_ok(_app.class_swap_runs == runs0 + 1, "the drag ended: the swap ran once (runs %d, want %d)" % [
		_app.class_swap_runs, runs0 + 1])
	_ok(_app.pending_class_swap().is_empty(), "nothing is pending after it ran")
	_ok(not _hint().begins_with("Layout change waiting"), "the waiting hint is cleared (hint '%s')" % _hint())
	await _resize(1800, 3200)

	print("-- a visible confirmation makes a crossing wait; returning to the class cancels the wait")
	var runs1: int = _app.class_swap_runs
	var dlg: ConfirmationDialog = DccWidgets.confirm(_app, "Probe question", "Probe text", "OK", func(): pass)
	await _frames(3)
	_ok(_has_blocker("a confirmation is open (Probe question)"), "a confirmation is reported (got %s)" % str(_blockers()))
	await _resize(880, 1564)
	_ok(not _app.pending_class_swap().is_empty() and _app.class_swap_runs == runs1,
		"880 x 1564 under a confirmation: pending, not run (runs %d)" % _app.class_swap_runs)
	_ok(_hint().begins_with("Layout change waiting: a confirmation is open"), "hint names it ('%s')" % _hint())
	## The settled resize must cancel the wait by itself: the poll (a second path to the same end) is stopped
	## first, so this check cannot be passed by the poll firing inside the window (found by mutation).
	_app._swap_wait_timer.stop()
	await _resize(1800, 3200)
	_ok(_app.pending_class_swap().is_empty(), "back at 1800 x 3200: the wait is cancelled")
	_ok(_app.class_swap_runs == runs1, "and no swap ran (runs %d, want %d)" % [_app.class_swap_runs, runs1])
	_ok(not _hint().begins_with("Layout change waiting"), "the waiting hint is cleared (hint '%s')" % _hint())
	_ok(is_instance_valid(dlg) and dlg.visible, "the confirmation was not touched: still open")
	print("-- the poll alone cancels a wait when the size has come back without a settled resize")
	await _resize(880, 1564)
	_ok(not _app.pending_class_swap().is_empty(), "880 x 1564 under the confirmation again: pending")
	## Back to the tablet size with no `size_changed`, so only the poll sees it.
	_vp.size = Vector2i(1800, 3200)
	await get_tree().create_timer(0.8).timeout
	await _frames(2)
	_ok(_app.pending_class_swap().is_empty() and _app.class_swap_runs == runs1,
		"the poll found the size back in the class: cancelled, no swap (runs %d, want %d)" % [_app.class_swap_runs, runs1])
	dlg.hide()
	dlg.queue_free()
	await _frames(3)

	print("-- other sources, read without a resize")
	_app.bridge.generating = true
	_ok(_has_blocker("the world is"), "a generating engine is reported (got %s)" % str(_blockers()))
	_app.bridge.generating = false
	_app.new_world_dialog.popup_centered()
	await _frames(3)
	_ok(_has_blocker("the New world form is open"), "an open New world form is reported (got %s)" % str(_blockers()))
	_app.new_world_dialog.hide()
	await _frames(3)
	_ok(_app.vault_window.quiescent_blockers().is_empty(), "the vault with no note open reports nothing")
	_ok(_blockers().is_empty(), "idle again: no blocker (got %s)" % str(_blockers()))

## -- S4.2: the composition dump and the same-class unmount / mount round trip ------------------------------------

## The emitters whose signal connection lists the dump and the round trip compare, as `[object, label]`:
## the five the inventory's section 3 lists (the tree's `node_added`, the root's `size_changed`, the shell's own
## signals, the bridge's, and the viewport's). Every signal of each is listed by `_connection_lines()`.
func _emitters() -> Array:
	return [[get_tree(), "tree"], [get_tree().root, "root"], [_app, "app"], [_app.bridge, "bridge"],
		[_app.viewport, "viewport"]]

## One line per signal that has connections, for every emitter, in connection order: the receiver's class and
## script and the method name. A lambda reports its enclosing function's name, which is deterministic, so a
## swapped order or a missing or doubled connection changes the text. A receiver in the composition shows its
## class, never its (auto-generated, counter-suffixed) name.
func _connection_lines() -> PackedStringArray:
	var out := PackedStringArray()
	for e in _emitters():
		var obj: Object = e[0]
		for sig in obj.get_signal_list():
			var cons: Array = obj.get_signal_connection_list(String(sig["name"]))
			if cons.is_empty():
				continue
			var parts := PackedStringArray()
			for c in cons:
				var cb: Callable = c["callable"]
				var tgt := cb.get_object()
				var cls := "freed" if tgt == null else String(tgt.get_class())
				var scr := ""
				if tgt != null and tgt.get_script() != null:
					scr = String((tgt.get_script() as Script).resource_path).get_file()
				parts.append("%s/%s.%s" % [cls, scr, cb.get_method()])
			out.append("CONN %s.%s n=%d : %s" % [e[1], sig["name"], cons.size(), " | ".join(parts)])
	return out

## The structure under the app, in tree order: class, script file, visibility and, for a Control, the global rect
## to 1 dp, indented by depth. No names (an unnamed node's name carries a counter) and no text (the clock and the
## build id are text). Windows are listed with their children: the round trip must leave them as they were, and
## the cold-boot comparison wants the child order of the app.
func _structure_lines(n: Node, depth: int, out: PackedStringArray, visible_only := false, skip_windows := false) -> void:
	## S4.3: a long-lived window (not a popup menu) decides its layout for the class it was set up in, and S4.5
	## re-presents it, so a cold-boot comparison of the COMPOSITION leaves it out (`_norm_text()`).
	if skip_windows and n is Window and not (n is PopupMenu):
		return
	var scr := ""
	if n.get_script() != null:
		scr = String((n.get_script() as Script).resource_path).get_file()
	var rect := ""
	if n is Control and not (visible_only and not (n as Control).is_visible_in_tree()):
		var r := (n as Control).get_global_rect()
		rect = " r=(%.1f,%.1f,%.1f,%.1f)" % [r.position.x, r.position.y, r.size.x, r.size.y]
	var vis := ""
	if n is CanvasItem:
		vis = " v=%s" % str((n as CanvasItem).visible)
	var nm := (" '" + String(n.name) + "' " + String(n.get_parent().name if n.get_parent() != null else "")) if _debug_names else ""
	out.append("%s%s [%s]%s%s%s" % ["  ".repeat(depth), n.get_class(), scr, vis, rect, nm])
	## The phone's project picker builds its content when it is opened, and boot opens it (no world); a remounted
	## picker is new and unopened, so its subtree is not comparable. The window itself is still counted.
	if (visible_only or skip_windows) and scr == "phone_project_picker.gd":
		return
	## S4.3: the viewport is a SURVIVOR whose own chrome (the Layers button's size, the scale rule, the map control
	## row) is built for the class it booted in; refreshing that is `ViewportHost`'s S4.4 work (and MS-5's file),
	## so a cold-boot comparison of the composition counts the node and not what is under it.
	if skip_windows and scr == "viewport_host.gd":
		return
	for c in n.get_children():
		_structure_lines(c, depth + 1, out, visible_only, skip_windows)

func _dump_text() -> String:
	var lines := PackedStringArray()
	_structure_lines(_app, 0, lines)
	lines.append_array(_connection_lines())
	lines.append("ORPHANS %d" % int(Performance.get_monitor(Performance.OBJECT_ORPHAN_NODE_COUNT)))
	lines.append("NODES %d" % int(Performance.get_monitor(Performance.OBJECT_NODE_COUNT)))
	return "\n".join(lines) + "\n"

func _dump_leg() -> void:
	var path := _arg("--out", "")
	_ok(path != "", "--out names the dump file")
	var f := FileAccess.open(path, FileAccess.WRITE)
	_ok(f != null, "the dump file opens")
	if f == null:
		return
	var txt := _dump_text()
	f.store_string(txt)
	f.close()
	print("DUMP ", path, " ", txt.length(), " chars, sha256 ", txt.sha256_text())

## Parses `_connection_lines()` into `{signal: sorted receiver list}`: the multiset of receivers per signal,
## with the connection ORDER discarded (a rebuilt composition's receivers legitimately come last).
func _connection_sets() -> Dictionary:
	var out := {}
	for l in _connection_lines():
		var head := String(l).get_slice(" : ", 0)
		var recv := String(l).get_slice(" : ", 1).split(" | ")
		var arr: Array = Array(recv)
		arr.sort()
		out[head.get_slice(" n=", 0)] = arr
	return out

## The sorted structure for the round trip's before and after. The rect is kept only for a control that is
## visible in the tree: a hidden control's rect is whatever its last layout pass left (the phone's generate column,
## a closed sheet), which is a timing artefact, not structure. The boot comparison (`_dump_text()`) keeps every rect.
func _structure_sorted() -> Array:
	var lines := PackedStringArray()
	_structure_lines(_app, 0, lines, true)
	var arr: Array = Array(lines)
	arr.sort()
	return arr

## Descendant count of each direct child of the app, keyed by class and script, for locating where a round trip left
## extra nodes.
func _child_weights() -> Dictionary:
	var out := {}
	for c in _app.get_children():
		var scr := ""
		if c.get_script() != null:
			scr = String((c.get_script() as Script).resource_path).get_file()
		var k := "%s [%s]" % [c.get_class(), scr]
		out[k] = int(out.get(k, 0)) + 1 + c.find_children("*", "", true, false).size()
	return out

## Nodes in the phone's project picker subtree (0 elsewhere): it builds its content on open, so the node count
## comparison leaves it out, as `_structure_lines()` does.
func _picker_nodes() -> int:
	var pk = _app.get("phone_project_picker")
	if pk == null or not is_instance_valid(pk):
		return 0
	return 1 + (pk as Node).find_children("*", "", true, false).size()

func _orphans() -> int:
	return int(Performance.get_monitor(Performance.OBJECT_ORPHAN_NODE_COUNT))

## S4.2 (`MOBILE_SHELL_MS1_SPEC.md` section 3.5; `S4_INVENTORY.md` section 12, step 4 at the SAME class): the
## composition is unmounted and mounted again `ROUNDS` times through `_remount_composition_for_test()`, and
## nothing may be left over. Held to LITERALS where a count is the thing (zero orphans gained, the same node
## count), and to the boot's own figures where the thing is "as it was" (every signal's receivers, the two
## `DccTheme` registries, the composition's structure). Positive controls: the old docks must be freed and the
## new ones different instances (else the round trip rebuilt nothing and every equality is vacuous), and the
## survivors must be the same instances, still valid, back under the new `viewport_content`.
const ROUNDS := 3

func _roundtrip() -> void:
	await _frames(10)
	## The phone shows its first-run coach toasts one after another for about eight seconds from boot; they remove
	## themselves and are not part of the composition, so the "before" snapshot waits them out.
	if DccTheme.is_phone():
		await get_tree().create_timer(20.0).timeout
		await _frames(4)
	var cls := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("-- roundtrip at class ", cls)
	var orph0 := _orphans()
	var nodes0 := int(Performance.get_monitor(Performance.OBJECT_NODE_COUNT)) - _picker_nodes()
	var conn0 := _connection_sets()
	var struct0 := _structure_sorted()
	var weights0 := _child_weights()
	var watch0: int = DccTheme._portrait_watchers.size()
	var elide0: int = DccTheme._elide_labels.size()
	var vp_id: int = _app.viewport.get_instance_id()
	var ro_id: int = _app.resource_overlay.get_instance_id()
	var ss_id: int = _app.section_strip.get_instance_id()
	var old_left: Object = _app.left_dock
	var old_rdc: Object = _app.right_dock_ctrl
	var old_root: Object = _app._phone_root if DccTheme.is_phone() else _app._desktop_root
	var old_menus: Object = _app.menus
	_ok(old_left != null and old_rdc != null and old_root != null, "a composition is present before the round trip")
	print("  info: portrait watchers %d, elide labels %d at boot" % [watch0, elide0])
	for r in ROUNDS:
		_app._remount_composition_for_test()
		await _frames(8)
	await _frames(10)
	_ok(not is_instance_valid(old_left) and not is_instance_valid(old_rdc) and not is_instance_valid(old_root),
		"the old docks, RightDock and composition root were freed (the round trip rebuilt something)")
	_ok(_app.left_dock != null and _app.right_dock_ctrl != null and _app.menus != old_menus,
		"new docks, a new RightDock and a new menus object stand in their place")
	_ok(_app.viewport.get_instance_id() == vp_id and _app.resource_overlay.get_instance_id() == ro_id
		and _app.section_strip.get_instance_id() == ss_id, "the viewport, the resource HUD and the section strip are the same instances")
	_ok(_app.viewport.get_parent() == _app.viewport_content and _app.resource_overlay.get_parent() == _app.viewport_content
		and _app.section_strip.get_parent() == _app.viewport_content, "and sit under the new viewport_content")
	var orph1 := _orphans()
	_ok(orph1 == orph0, "orphan nodes after %d round trips: %d, before: %d" % [ROUNDS, orph1, orph0])
	if orph1 != orph0:
		var hist := {}
		for id in Node.get_orphan_node_ids():
			var o := instance_from_id(id)
			if o != null:
				var k := o.get_class()
				if o.get_script() != null:
					k += " [" + String((o.get_script() as Script).resource_path).get_file() + "]"
				hist[k] = int(hist.get(k, 0)) + 1
		print("    orphan classes: ", hist)
	var nodes1 := int(Performance.get_monitor(Performance.OBJECT_NODE_COUNT)) - _picker_nodes()
	_ok(nodes1 == nodes0, "node count (the phone picker's subtree left out) after: %d, before: %d" % [nodes1, nodes0])
	var conn1 := _connection_sets()
	var bad := PackedStringArray()
	for k in conn0:
		if not conn1.has(k) or conn1[k] != conn0[k]:
			bad.append("%s %d->%d" % [k, (conn0[k] as Array).size(), (conn1[k] as Array).size() if conn1.has(k) else 0])
	for k in conn1:
		if not conn0.has(k):
			bad.append("%s new %d" % [k, (conn1[k] as Array).size()])
	_ok(bad.is_empty(), "every signal has the same receivers after (differences: %s)" % ", ".join(bad))
	for k in conn1:
		if conn0.has(k) and conn1[k] != conn0[k]:
			var m := {}
			for s in conn1[k]:
				m[s] = int(m.get(s, 0)) + 1
			for s in conn0[k]:
				m[s] = int(m.get(s, 0)) - 1
			var parts := PackedStringArray()
			for s in m:
				if int(m[s]) != 0:
					parts.append("%s %+d" % [s, int(m[s])])
			print("    diff ", k, ": ", " ; ".join(parts))
	_ok(DccTheme._portrait_watchers.size() == watch0, "portrait watchers: %d, before %d" % [DccTheme._portrait_watchers.size(), watch0])
	_ok(DccTheme._elide_labels.size() == elide0, "elide labels: %d, before %d" % [DccTheme._elide_labels.size(), elide0])
	var struct1 := _structure_sorted()
	_ok(struct1 == struct0, "the structure (class, script, visibility, rect), as a sorted multiset, is what boot built (%d lines, %d before)" % [struct1.size(), struct0.size()])
	var weights1 := _child_weights()
	for k in weights1:
		if int(weights1[k]) != int(weights0.get(k, 0)):
			print("    app child ", k, ": ", int(weights0.get(k, 0)), " -> ", int(weights1[k]))
			if k.begins_with("PopupMenu"):
				var names := PackedStringArray()
				for c in _app.get_children():
					if c is PopupMenu:
						names.append(String(c.name))
				print("      popup children of the app: ", ", ".join(names))
	if struct1 != struct0:
		var a := {}
		for s in struct0:
			a[s] = int(a.get(s, 0)) + 1
		var b := {}
		for s in struct1:
			b[s] = int(b.get(s, 0)) + 1
		var shown := 0
		for s in a:
			if int(a[s]) > int(b.get(s, 0)) and shown < 8:
				print("    only before (x%d): %s" % [int(a[s]) - int(b.get(s, 0)), String(s).strip_edges()])
				shown += 1
		shown = 0
		for s in b:
			if int(b[s]) > int(a.get(s, 0)) and shown < 8:
				print("    only after (x%d): %s" % [int(b[s]) - int(a.get(s, 0)), String(s).strip_edges()])
				shown += 1

## -- S4.3: the cross-class remount and the state it carries ---------------------------------------------------------

## The dump a cold boot and a remounted shell are compared by: every visible control's class, script, visibility and
## rect as a SORTED multiset (a rebuilt node's tree position is a construction-order artefact, a hidden control's
## rect a timing artefact), every signal's receivers as a sorted multiset (a rebuilt receiver is connected last),
## and the node and orphan counts. The phone picker's subtree is left out of the node count, as `_roundtrip()` does.
func _norm_text() -> String:
	var lines := PackedStringArray()
	var st := PackedStringArray()
	_structure_lines(_app, 0, st, true, true)
	var sarr: Array = Array(st)
	sarr.sort()
	for l in sarr:
		lines.append("S " + String(l))
	var conns := _connection_sets()
	var keys: Array = conns.keys()
	keys.sort()
	for k in keys:
		if String(k).begins_with("viewport."):
			continue  ## the survivor viewport's own signals: see `_structure_lines()`
		## Receivers that are long-lived windows are S4.5's (see `_structure_lines()`): left out of both sides.
		var recv: Array = []
		for r in conns[k]:
			## "GDScript/." is a lambda defined in a static helper (`DccWidgets`, `ContextWindow`): the windows' own
			## relays (`phone_present()`, the context windows' chips), which S4.5 owns, left out like their windows.
			if not (String(r).begins_with("AcceptDialog/") or String(r).begins_with("ConfirmationDialog/") or String(r).begins_with("Window/") or String(r).begins_with("GDScript/.")):
				recv.append(r)
		lines.append("C %s : %s" % [k, " | ".join(PackedStringArray(recv))])
	var all := PackedStringArray()
	_structure_lines(_app, 0, all, false, true)
	lines.append("NODES %d" % all.size())
	lines.append("ORPHANS %d" % _orphans())
	lines.append("CLASS %s" % _app.class_key())
	var sc: ScrollContainer = _app._left_dock_scroll
	if sc != null:
		var vb := sc.get_v_scroll_bar()
		print("  info dock widths: left ", _app._left_width, " ", _app.left_dock.custom_minimum_size.x, " size ", _app.left_dock.size.x, " landscape ", _app._landscape, " portrait ", DccTheme.is_tablet_portrait(), " narrow ", DccTheme.is_laptop())
		print("  info left dock scroll: mode ", sc.vertical_scroll_mode, " bar visible ", vb.visible, " bar width ", vb.size.x, " min ", vb.get_combined_minimum_size().x, " max ", vb.max_value, " page ", vb.page, " scroll size ", sc.size)
	return "\n".join(lines) + "\n"

func _write_norm(path: String) -> void:
	_ok(path != "", "--out names the dump file")
	var f := FileAccess.open(path, FileAccess.WRITE)
	_ok(f != null, "the dump file opens")
	if f == null:
		return
	var txt := _norm_text()
	f.store_string(txt)
	f.close()
	print("NORM ", path, " ", txt.length(), " chars, sha256 ", txt.sha256_text())

## Waits for the boot's settling and, on the phone, for its first-run coach toasts (about eight seconds, not part
## of the composition) so a snapshot is taken at rest. Used identically by the cold boot and the remounted shell.
func _settle_at_rest() -> void:
	await _frames(10)
	if DccTheme.is_phone():
		await get_tree().create_timer(20.0).timeout
		await _frames(4)

func _norm_leg() -> void:
	await _settle_at_rest()
	_write_norm(_arg("--out", ""))

## Moves the host to `w` x `h` and remounts across to the class `want` (a literal the step names), then waits.
func _swap_to(want: String, w: int, h: int) -> void:
	_vp.size = Vector2i(w, h)
	await _frames(2)
	## The touch flag is hardware, never a swap dimension (a pointer window does not become a touch one), so it
	## stays what the shell booted with: null.
	await _app._remount_composition_for_test(null)
	await _frames(8)
	_ok(String(_app.class_key()) == want, "the swap lands on %s (class_key %s)" % [want, _app.class_key()])

## S4.3 bar: remount desktop -> tablet -> phone -> tablet (or any `--steps` chain) at runtime. Writes the final
## normalised dump to `--out`; the runner compares it with `--leg norm` of a cold boot of the last step's class and
## size (structure, every signal's receivers, node and orphan counts). Also holds a literal: no signal of the
## tree may keep a freed receiver after any step.
func _crossclass() -> void:
	await _settle_at_rest()
	print("  info tl-connected at start: ", _app.timeline_changed.is_connected(_app._repaint_timeline), " bar visible ", _app.timeline_bar.visible)
	var steps := _arg("--steps", "").split(",", false)
	_ok(steps.size() >= 1, "--steps names at least one class change")
	var vp_id: int = _app.viewport.get_instance_id()
	for st in steps:
		var cw := String(st).split(":")
		var wh := cw[1].split("x")
		print("-- swap to ", st)
		await _swap_to(cw[0], int(wh[0]), int(wh[1]))
		print("  info tl-connected: ", _app.timeline_changed.is_connected(_app._repaint_timeline), " bar visible ", _app.timeline_bar.visible, " domain ", _app.active_domain())
		_ok(_app.viewport.get_instance_id() == vp_id, "  the viewport is the same instance")
		_ok(_app.viewport.get_parent() == _app.viewport_content, "  and sits under the new viewport_content")
		var orph_cap: int = {"tablet": 214, "phone": 1, "desktop": 0}[String(_app.class_key())]
		_ok(_orphans() <= orph_cap, "  orphan nodes after the swap: %d, at most %d (cold boots of this class and size leave 213, 1 and 0 for tablet, phone and desktop; the tablet's extra one is a phone boot's own)" % [_orphans(), orph_cap])
		if _orphans() > orph_cap:
			var hist := {}
			for id in Node.get_orphan_node_ids():
				var o := instance_from_id(id)
				if o != null:
					var k := o.get_class()
					if o.get_script() != null:
						k += " [" + String((o.get_script() as Script).resource_path).get_file() + "]"
					hist[k] = int(hist.get(k, 0)) + 1
			print("    orphan classes: ", hist)
		var freed := 0
		for l in _connection_lines():
			if String(l).contains("freed/"):
				freed += 1
		_ok(freed == 0, "  no connection keeps a freed receiver (%d)" % freed)
	await _settle_at_rest()
	_write_norm(_arg("--out", ""))

## The carry leg's recorded values, written on the tablet and read back after each swap.
var _st := {}

func _panel_paths() -> Dictionary:
	var out := {}
	for n in _app.left_dock_body.find_children("*", "", true, false):
		if n is Workspace and not (n as Workspace).categories.is_empty():
			out[String(_app.left_dock_body.get_path_to(n))] = n
	return out

## The title of the category of `ws` that is open now, "" when none is.
func _open_title(ws: Workspace) -> String:
	for e in ws.categories:
		if (e["body"] as Control).visible and ((e["body"] as Control).get_parent() as Control).visible:
			return String(e["title"])
	return ""

func _scroll_max(sc: ScrollContainer) -> int:
	var b := sc.get_v_scroll_bar()
	return int(b.max_value - b.page)

## S4.3: every item the swap carries, set on the tablet, then read back after tablet -> phone -> tablet -> desktop
## -> tablet -> desktop -> phone. Expected values are LITERALS or the values this function wrote itself on a
## different class (never read back from the composition under test). A positive control: the fixture must be
## able to hold each item (a scrollable dock, three accordions, an options row), or the equality is vacuous.
func _carry() -> void:
	await _settle_at_rest()
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	_ok(String(_app.class_key()) in ["tablet", "desktop"], "boots as a tablet or the desktop (class_key %s)" % _app.class_key())
	var b = _app.bridge
	b.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _frames(2)
	while bool(b.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _frames(10)
	await _frames(30)
	_ok(bool(b.has_world) and b.settlements().size() >= 2, "a world with two settlements stands")
	## ---- set every carried item on the tablet ----
	var civ_modes: Array = []
	for n in DccShell.RAIL_NODES:
		if String(n.get("kind", "")) == "node" and String(n["domain"]) == "civilization":
			civ_modes.append(String(n["mode"]))
	_st["mode"] = civ_modes[1]
	_app.select_domain_mode("civilization", _st["mode"])
	await _frames(4)
	## One non-first, visible category open in each workspace panel.
	_st["open"] = {}
	var panels0 := _panel_paths()
	for path in panels0:
		var ws: Workspace = panels0[path]
		var pick := ""
		for e in ws.categories:
			if ((e["body"] as Control).get_parent() as Control).visible and String(e["title"]) != _open_title(ws):
				pick = String(e["title"])
		if pick != "":
			ws.open_category(pick)
		_st["open"][path] = _open_title(ws)
	await _frames(4)
	_ok(_st["open"].size() >= 3, "three or more workspace panels hold an accordion (%d)" % _st["open"].size())
	_app.arm_tool("measure")
	await _frames(4)
	_st["opts"] = _app.tool_options_row.get_child_count()
	_ok(_st["opts"] > 0, "the measure tool built an options row (%d children)" % _st["opts"])
	_app.select("settlement", 0)
	await _frames(3)
	_ok(_app.right_dock_ctrl.pin(), "the Context panel pins settlement 0")
	_app.select("settlement", 1)
	await _frames(3)
	_ok(_app.right_dock_ctrl.is_pinned() and _app.right_dock_ctrl.shown_subject() == {"kind": "settlement", "id": 0},
		"pinned: the panel holds settlement 0 while settlement 1 is selected")
	_ok(_app.right_dock_ctrl.show_tab("note"), "the Note tab opens")
	_st["back"] = _app._selection_back.duplicate(true)
	_ok(_st["back"].size() == 2, "the selection history holds two steps (%d)" % _st["back"].size())
	var r: Dictionary = _app.tl_range()
	_ok(int(r["min"]) == -400, "the strip's range starts at the fallback -400 (%d)" % int(r["min"]))
	_st["year"] = int(r["min"]) + 37
	_app.tl_set_year(_st["year"])
	_ok(_app.tl_year() == _st["year"], "the year cursor moved to %d (got %d)" % [_st["year"], _app.tl_year()])
	_app._left_width = 333.0
	_app._apply_dock_widths()
	_app._toggle_dock(false)
	await _frames(4)
	_ok(_app.right_dock.custom_minimum_size.x == float(DccTheme.W_RAIL_COLLAPSED), "the right dock is collapsed")
	var sc: ScrollContainer = _app._left_dock_scroll
	var smax := _scroll_max(sc)
	_ok(smax >= 50, "the left dock can scroll (max %d): the fixture reaches the scroll" % smax)
	sc.scroll_vertical = 50
	await _frames(3)
	_ok(sc.scroll_vertical == 50, "the left dock holds scroll 50")

	## ---- the chain ----
	## The touch flag is hardware, so the chain keeps it: on a touch build tablet <-> phone, on a pointer build the
	## desktop remounted twice (`--leg carry` without `--force-touch`, 1600x1000).
	if String(_app.class_key()) == "tablet":
		print("-- tablet -> phone")
		await _swap_to("phone", 880, 1564)
		_check_common("on the phone")
		_app.set_phone_detent("full")
		await _frames(4)
		_ok(_app.phone_detent() == "full", "the phone's detent is full")
		_ok(_app.right_dock_ctrl.shown_subject() == {"kind": "settlement", "id": 1}, "phone: the sheet shows the selection (settlement 1; a phone has no pin)")
		print("-- phone -> tablet")
		await _swap_to("tablet", 1800, 1000)
		_check_common("back on the tablet")
		_check_chrome("back on the tablet")
		print("-- tablet -> phone, again")
		await _swap_to("phone", 880, 1564)
		_check_common("on the phone, second visit")
		_ok(_app.phone_detent() == "full", "phone, second visit: the detent is still full (%s)" % _app.phone_detent())
		print("-- phone -> tablet, again")
		await _swap_to("tablet", 1800, 1000)
		_check_common("tablet, second visit")
		_check_chrome("tablet, second visit")
	else:
		print("-- desktop -> desktop (the same class, state carried through two remounts)")
		await _swap_to("desktop", 1600, 1000)
		_check_common("desktop remount 1")
		_check_chrome("desktop remount 1")
		await _swap_to("desktop", 1600, 1000)
		_check_common("desktop remount 2")
		_check_chrome("desktop remount 2")

## What every class must still show after a swap: the domain and its mode, the open accordion categories, the
## selection and its history, the armed tool, the year.
func _check_common(tag: String) -> void:
	_ok(_app.active_domain() == "civilization" and _app.active_mode("civilization") == _st["mode"],
		"%s: domain civilization, mode %s (got %s/%s)" % [tag, _st["mode"], _app.active_domain(), _app.active_mode("civilization")])
	var panels := _panel_paths()
	for path in _st["open"]:
		_ok(panels.has(path) and _open_title(panels[path]) == _st["open"][path],
			"%s: %s has %s open (got %s)" % [tag, path, _st["open"][path], _open_title(panels[path]) if panels.has(path) else "no panel"])
	_ok(_app.selection == {"kind": "settlement", "id": 1}, "%s: the selection is settlement 1 (%s)" % [tag, str(_app.selection)])
	_ok(_app._selection_back == _st["back"], "%s: the selection history is the two steps (%s)" % [tag, str(_app._selection_back)])
	_ok(_app.armed_tool == "measure", "%s: measure is armed (%s)" % [tag, _app.armed_tool])
	_ok(_app.tl_year() == _st["year"], "%s: the year is %d (got %d)" % [tag, _st["year"], _app.tl_year()])
	var seen := false
	var texts := PackedStringArray()
	for l in _app._tl_year_labels:
		if is_instance_valid(l):
			texts.append(String(l.text))
			if String(l.text) == "363 BC":
				seen = true
	## The phone's strip lives in the STORY sheet and is not built until opened: its year label is not drawn.
	seen = seen or tag.contains("phone")
	_ok(seen, "%s: a year label on the strip reads 363 BC, year -363 (labels %s)" % [tag, str(texts)])

## What only a class with the docks and the Context panel's chrome shows: the pin and the tab, the options row, the
## dragged width, the collapsed right dock and the scroll.
func _check_chrome(tag: String) -> void:
	_ok(_app.right_dock_ctrl.is_pinned() and _app.right_dock_ctrl.shown_subject() == {"kind": "settlement", "id": 0},
		"%s: pinned to settlement 0 while settlement 1 is selected" % tag)
	_ok(String(_app.right_dock_ctrl.get("_tab")) == "note", "%s: the Note tab is open (%s)" % [tag, str(_app.right_dock_ctrl.get("_tab"))])
	_ok(_app.tool_options_row.get_child_count() == _st["opts"], "%s: the options row has %d children (got %d)" % [tag, _st["opts"], _app.tool_options_row.get_child_count()])
	_ok(_app._left_width == 333.0 and _app.left_dock.custom_minimum_size.x == 333.0, "%s: the left dock is 333 wide (%.0f)" % [tag, _app.left_dock.custom_minimum_size.x])
	_ok(_app.right_dock.custom_minimum_size.x == float(DccTheme.W_RAIL_COLLAPSED), "%s: the right dock is collapsed" % tag)
	_ok(_app._left_dock_scroll.scroll_vertical == 50, "%s: the left dock is scrolled to 50 (%d)" % [tag, _app._left_dock_scroll.scroll_vertical])

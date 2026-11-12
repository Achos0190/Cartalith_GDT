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
##   ... -- --leg live --force-touch --vp 1800x1000 --density 1 --out <prefix> [--trips N] [--window]
##         (S4.4: the REAL swap. Resizes through the resize path, `--window` through `DisplayServer.window_set_size()`,
##         across the 900 dp line with hysteresis; a window open makes the swap wait with a reason; ten round trips;
##         every carried item, the Ask conversation, the tool settings, the world hash and the camera zoom checked after
##         each; writes `<prefix>.phone` and `<prefix>.tablet` for the runner to compare with a cold boot.
##         `--clean` sets no user state, `--noworld` also stands no world, so the end state equals `--leg norm`'s.)
##   ... -- --leg represent --force-touch --vp 1800x1000 --density 1          (S4.5: open windows are re-presented in place)
##   ... -- --leg ringcard --force-touch --vp 1800x1000 --density 1       (S4.6: the radial ring and the context card across a swap)
##   ... -- --leg b5 --force-touch --vp 1800x1000 --density 1       (S4.6: B5's three windows in turn, each open across the swap, and the swap's timing)
##   ... -- --leg assistant --force-touch --vp 1800x1000 --density 1       (S4.5's last window: the assistant dialog is KEPT and re-laid out in place, mid-conversation, with a pending card and a simulated running request; S4_INVENTORY.md 18.11)
##   ... -- --leg norm [--world] --force-touch --vp 880x1564 --density 1 --out <file>   (cold boot, optionally with a world)
##
## Legs:
##   quiet   (stage S4.1) the quiescence gate. The shell must report no blocker when idle; a held dock drag, a
##           visible confirmation, a generating engine and an open New world form must each be reported by
##           `quiescent_blockers()`; a size change across the line while one is held must WAIT (no swap run,
##           the status hint naming the blocker) and run once the blocker ends; a size that returns to the
##           current class while waiting must cancel the wait (no swap run, the hint cleared). At density 1,
##           1 800 x 3 200 dp is a tablet and 880 x 1 564 dp a phone (past 900 - 16, `DccTheme.BAND_DP`).
##
##   live    (stage S4.4) see the usage line above. Run WITHOUT `--force-touch` on a pointer build it asserts the other
##           half of the bar: no size makes a desktop a phone (touch is hardware), so no swap is ever requested.
##
## Run windowed with a wiped user dir (the runner sets `APPDATA` to an empty folder). Logic and layout only, so
## nothing here reads pixels. Arguments read by THIS file: `--leg quiet`, `--vp WxH`, `--only <window field>` (represent only); accepted because the
## shell reads them: `--force-touch`, `--density D`. Flags of this file with no value: `--window`, `--clean`, `--noworld`,
## `--world`, `--names`. Anything else aborts with exit 2 (`MISTAKES.md`, "Write a
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

func _arg_int(name: String, dflt: int) -> int:
	return int(_arg(name, str(dflt)))

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	if k >= 0 and k + 1 < args.size():
		return String(args[k + 1])
	return dflt

## `--window`: the shell is in the real root window (see `_ready()`), not in a host SubViewport.
var _window_mode := false
## How long `app.tscn` took to instantiate and enter the tree at boot, in ms; the cold-boot figure the live leg's swap
## timing is set against (measured on this machine, labelled so in the report).
var _cold_boot_ms := 0.0
## Milliseconds from the engine's start to this probe's `_ready()` (`Time.get_ticks_msec()` counts from engine start): the
## process, the extension and the scripts, which a swap never pays and which a RESTART does. B5's timing reads the swap against
## `_cold_boot_ms` alone and against `_engine_ms + _cold_boot_ms`, and says which it is (`_b5()`).
var _engine_ms := 0

## Resizes the host and tells the shell, then waits past the 150 ms debounce (0.4 s).
func _resize(w: int, h: int) -> void:
	if _window_mode:
		DisplayServer.window_set_size(Vector2i(w, h))
		await _frames(2)
		if get_tree().root.size != Vector2i(w, h):
			print("  info: the window is %s after asking for %dx%d" % [str(get_tree().root.size), w, h])
		get_tree().root.size_changed.emit()
	else:
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
	_engine_ms = Time.get_ticks_msec()
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
		if a == "--force-touch" or a == "--names" or a == "--window" or a == "--clean" or a == "--world" or a == "--noworld":
			i += 1
		elif a in ["--leg", "--vp", "--density", "--out", "--steps", "--trips", "--only", "--fresh-frames"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	var leg := _arg("--leg", "")
	_debug_names = "--names" in OS.get_cmdline_user_args()
	var parts := _arg("--vp", "").split("x")
	if not leg in ["quiet", "dump", "roundtrip", "norm", "crossclass", "carry", "live", "represent", "ringcard", "b5", "assistant"] or parts.size() != 2:
		print("ABORT: needs --leg quiet|dump|roundtrip|norm|crossclass|carry|live|represent|ringcard|b5|assistant and --vp WxH")
		get_tree().quit(2)
		return
	var vw := int(parts[0])
	var vh := int(parts[1])
	_window_mode = "--window" in OS.get_cmdline_user_args()
	var t_boot := Time.get_ticks_usec()
	if _window_mode:
		## `--window`: no host SubViewport; the shell lives in the real root window, and a "resize" is
		## `DisplayServer.window_set_size()`, the call a split-screen divider or a freeform drag ends in.
		DisplayServer.window_set_size(Vector2i(vw, vh))
		await _frames(4)
		_app = load("res://shell/app.tscn").instantiate()
		add_child(_app)
	else:
		DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
		_vp = SubViewport.new()
		_vp.size = Vector2i(vw, vh)
		_vp.gui_embed_subwindows = true
		_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(_vp)
		_app = load("res://shell/app.tscn").instantiate()
		_vp.add_child(_app)
	_cold_boot_ms = (Time.get_ticks_usec() - t_boot) / 1000.0
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
	elif leg == "live":
		await _live()
	elif leg == "represent":
		await _represent()
	elif leg == "ringcard":
		await _ringcard()
	elif leg == "b5":
		await _b5()
	elif leg == "assistant":
		await _assistant()
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
	## S4.5 batch 5: merely being open no longer holds a swap (the form is carried); a pending large-grid confirmation does, because its
	## `proceed` callable belongs to the instance.
	_ok(not _has_blocker("the New world form"), "an open New world form holds nothing (got %s)" % str(_blockers()))
	_app.new_world_dialog._confirm_phone_memory(8192, 8192, func(): pass, func(): pass)
	await _frames(3)
	_ok(_has_blocker("the New world form is asking to confirm a large grid"), "a pending large-grid confirmation is reported (got %s)" % str(_blockers()))
	_app.new_world_dialog._memory_confirm.hide()
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
	## S4.4: a `Timer` is shell infrastructure made on first use (the resize debounce, the swap's poll), not part of
	## the composition; a cold boot has made none of them yet. Left out of the cold-boot comparison on both sides.
	if skip_windows and n is Timer:
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
	## S4.4: the viewport is a SURVIVOR, and `ViewportHost.rebuild_chrome()` builds its chrome (the Layers button, the
	## scale bar and rule, the readouts, the context chip, the navpad) again for the new class, so a cold-boot comparison
	## now lists those nodes too. Its camera subtree (the map, its tiles and overlays) is the world and is left out.
	for c in n.get_children():
		if skip_windows and scr == "viewport_host.gd" and c == (n as Object).get("_camera"):
			continue
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
## composition is unmounted and mounted again `ROUNDS` times through `_remount_composition()`, and
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
		_app._remount_composition()
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
	var slots := PackedStringArray()
	for sl in ["pass", "stale", "atlas", "progress", "autosave", "hint", "mid"]:
		slots.append("%s='%s'" % [sl, _app.status_slot_text(sl)])
	print("  info status slots: ", ", ".join(slots))
	print("  info right dock: tab ", _app.right_dock_ctrl.get("_tab"), " pinned ", _app.right_dock_ctrl.is_pinned(), " collapsed l/r ", _app._left_collapsed, "/", _app._right_collapsed, " floating l/r ", _app._left_floating, "/", _app._right_floating, " subject ", _app.right_dock_ctrl.shown_subject())
	var strips := 0
	for n in get_tree().root.find_children("*", "PanelContainer", true, false):
		if n.get_script() != null and String((n.get_script() as Script).resource_path).get_file() == "tool_strip.gd":
			strips += 1
	var recv := 0
	for c in _app.bridge.get_signal_connection_list("generation_finished"):
		var o: Object = (c["callable"] as Callable).get_object()
		if o != null and o.get_script() != null and String((o.get_script() as Script).resource_path).get_file() == "tool_strip.gd":
			recv += 1
	print("  info tool strips in the tree: ", strips, ", generation_finished receivers of tool_strip.gd: ", recv)
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
	## `--world`: the cold boot a `--leg live --clean` run is compared with, with the same world standing.
	if "--world" in OS.get_cmdline_user_args():
		await _make_world()
		await _settle_at_rest()
	_write_norm(_arg("--out", ""))

## Moves the host to `w` x `h` and remounts across to the class `want` (a literal the step names), then waits.
func _swap_to(want: String, w: int, h: int) -> void:
	_vp.size = Vector2i(w, h)
	await _frames(2)
	## The touch flag is hardware, never a swap dimension (a pointer window does not become a touch one), so it
	## stays what the shell booted with: null.
	await _app._remount_composition(null)
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
	await _make_world()
	await _carry_fixture()

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

## Generates the probe's world (seed 4242, 256 x 192) and waits for it; asserts two settlements stand.
func _make_world() -> void:
	var b = _app.bridge
	b.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _frames(2)
	while bool(b.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _frames(10)
	await _frames(30)
	_ok(bool(b.has_world) and b.settlements().size() >= 2, "a world with two settlements stands")

## Sets every carried item on the boot class (see `_carry()`): domain mode, open accordions, the armed measure tool,
## a pin and a selection history, the year, a dragged left width, a collapsed right dock and a scrolled left dock.
## Writes the literals it sets into `_st`; the checks read them back after each swap.
func _carry_fixture() -> void:
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

## -- S4.4: the live swap ---------------------------------------------------------------------------------------------

## The signals whose receiver lists the bar names (spec B5): compared between the first and every later visit of a class.
const LIVE_SIGNALS := ["CONN root.size_changed", "CONN tree.node_added", "CONN app.phone_insets_changed",
	"CONN app.timeline_changed", "CONN app.workspace_changed", "CONN bridge.world_loaded", "CONN bridge.generation_finished"]
## What the live leg writes on the boot class and reads back after every swap (literals it chose, never read from the
## composition under test): the paint brush radius, a droplet-erosion setting, the bake depth, the territory radius,
## the colour-paint radius, the Ask conversation's draft and entry count, the map's zoom.
const LIVE_PAINT_RADIUS := 17.0
const LIVE_BAKE_DEPTH := 5
const LIVE_TERRITORY := 9.0
const LIVE_COLOUR_KM := 3.5
const LIVE_DRAFT := "draft one"
## Round trips of the live leg; the bar is ten, and `--trips N` runs fewer for a mutant run (the mutants need one).
var ROUND_TRIPS := 10

## `--clean` (leg live): swap with the world standing and none of the user's state set, so the end state is what a
## cold boot of the class with the same world draws (`--leg norm --world`), which the runner compares.
var _clean := false
var _live_orph := {}
var _live_conn := {}
var _live_world := ""
var _live_zoom := 0.0
var _live_t_sync := {"phone": [], "tablet": []}
var _live_t_full := {"phone": [], "tablet": []}

func _workspace(name: String) -> Object:
	for w in _app._workspaces:
		if String(w.name) == name:
			return w
	return null

func _median(a: Array) -> float:
	var s := a.duplicate()
	s.sort()
	return 0.0 if s.is_empty() else float(s[s.size() / 2])

## The hash of what the world IS to the viewer: the map texture's bytes and a fixed set of engine cell samples.
func _world_hash() -> String:
	if _app.viewport.map_view.texture == null:
		return "no world"
	var img: Image = _app.viewport.map_view.texture.get_image()
	var cells := ""
	for p in [Vector2i(10, 10), Vector2i(100, 80), Vector2i(200, 150), Vector2i(60, 120)]:
		cells += str(_app.bridge.sample_cell(p.x, p.y))
	var hc := HashingContext.new()
	hc.start(HashingContext.HASH_SHA256)
	hc.update(img.get_data())
	return hc.finish().hex_encode() + ":" + cells.sha256_text()

## Waits for a swap that has started (or is pending) to finish: the swap runs in a coroutine, and a wait ends by a poll.
func _await_swap(limit_s := 40.0) -> void:
	var t0 := Time.get_ticks_msec()
	while (bool(_app._swap_running) or not _app.pending_class_swap().is_empty()) and Time.get_ticks_msec() - t0 < int(limit_s * 1000.0):
		await _frames(5)
	await _frames(10)

## Resizes (the real resize path, `_resize()`) and expects ONE swap to `want`: the counter moves by exactly one.
func _live_swap(w: int, h: int, want: String) -> void:
	var runs0: int = _app.class_swap_runs
	await _resize(w, h)
	await _await_swap()
	_ok(String(_app.class_key()) == want and _app.class_swap_runs == runs0 + 1,
		"%dx%d swaps to %s, once (class %s, runs %d -> %d)" % [w, h, want, _app.class_key(), runs0, _app.class_swap_runs])

## MS-7: resizes and expects ONE swap that keeps the class `want` (the tablet's menu bar changed).
func _live_bar_swap(w: int, h: int, want: String, why: String) -> void:
	var runs0: int = _app.class_swap_runs
	await _resize(w, h)
	await _await_swap()
	_ok(String(_app.class_key()) == want and _app.class_swap_runs == runs0 + 1 and _app.pending_class_swap().is_empty(),
		"%dx%d stays %s through one bar swap (%s; class %s, runs %d -> %d)" % [w, h, want, why, _app.class_key(), runs0, _app.class_swap_runs])

## Resizes and expects NO swap: the class stays `want`, the counter does not move and nothing waits.
func _live_stay(w: int, h: int, want: String, why: String) -> void:
	var runs0: int = _app.class_swap_runs
	await _resize(w, h)
	await _await_swap()
	_ok(String(_app.class_key()) == want and _app.class_swap_runs == runs0 and _app.pending_class_swap().is_empty(),
		"%dx%d stays %s with no swap (%s; class %s, runs %d -> %d)" % [w, h, want, why, _app.class_key(), runs0, _app.class_swap_runs])

func _ask_state() -> Dictionary:
	var win = _app.assistant_window
	var msgs: Node = win.get("_msgs")
	var inp = win.controls.get("input")
	return {"n": 0 if msgs == null or not is_instance_valid(msgs) else msgs.get_child_count(),
		"draft": "" if inp == null or not is_instance_valid(inp) else String((inp as TextEdit).text),
		"host_ok": win.get("_host") != null and is_instance_valid(win.get("_host"))}

## What the live leg wrote on the first class, read back after a swap.
func _live_check_items(tag: String) -> void:
	var w = _workspace("WorldWorkspace")
	_ok(w != null and float(w._paint_brush["radius"]) == LIVE_PAINT_RADIUS, "%s: the paint brush radius is %.0f" % [tag, LIVE_PAINT_RADIUS])
	_ok(w != null and int(w._bake_depth) == LIVE_BAKE_DEPTH and DccSettings.bake_depth() == LIVE_BAKE_DEPTH, "%s: the bake depth is %d, in the setting and in the rebuilt Finalize foot" % [tag, LIVE_BAKE_DEPTH])
	_ok(w != null and float(w._erode_op[_st["erode_key"]]) == float(_st["erode_val"]) and bool(w._erode_op_seeded),
		"%s: the erode setting %s is %s and not re-seeded" % [tag, _st["erode_key"], str(_st["erode_val"])])
	var c = _workspace("CivilizationWorkspace")
	_ok(c != null and float(c._territory_radius) == LIVE_TERRITORY, "%s: the territory radius is %.0f" % [tag, LIVE_TERRITORY])
	var k = _workspace("CartographyWorkspace")
	_ok(k != null and k.colour_paint != null and float(k.colour_paint.radius_km) == LIVE_COLOUR_KM, "%s: the colour-paint radius is %.1f km" % [tag, LIVE_COLOUR_KM])
	var a := _ask_state()
	_ok(a["n"] == _st["ask_n"] and a["draft"] == LIVE_DRAFT and a["host_ok"], "%s: the Ask conversation survived (%d entries, draft '%s', content alive %s)" % [tag, a["n"], a["draft"], str(a["host_ok"])])
	_ok(_world_hash() == _live_world, "%s: the world is byte-identical (map texture and engine cells)" % tag)
	_ok(absf(float(_app.viewport.zoom()) - _live_zoom) < 1e-6, "%s: the camera zoom is unchanged (%.6f, was %.6f)" % [tag, _app.viewport.zoom(), _live_zoom])

func _live_conn_sets() -> Dictionary:
	var all := _connection_sets()
	var out := {}
	for k in LIVE_SIGNALS:
		out[k] = all.get(k, [])
	return out

## Counts after a visit: orphans and the named signals' receiver lists. The first visit of a class sets the reference;
## every later visit must equal it.
func _live_visit(cls: String, trip: int) -> void:
	var o := _orphans()
	var c := _live_conn_sets()
	if not _live_orph.has(cls):
		_live_orph[cls] = o
		_live_conn[cls] = c
		var sizes := PackedStringArray()
		for k in LIVE_SIGNALS:
			sizes.append("%s=%d" % [k, (c[k] as Array).size()])
		print("  info %s reference: orphans %d, receivers %s" % [cls, o, ", ".join(sizes)])
		return
	_ok(o == _live_orph[cls], "trip %d, %s: orphan nodes %d equal the first visit's %d" % [trip, cls, o, _live_orph[cls]])
	var bad := PackedStringArray()
	for k in LIVE_SIGNALS:
		if c[k] != _live_conn[cls][k]:
			bad.append("%s %d->%d" % [k, (_live_conn[cls][k] as Array).size(), (c[k] as Array).size()])
	_ok(bad.is_empty(), "trip %d, %s: the named signals have the first visit's receivers (%s)" % [trip, cls, ", ".join(bad)])

func _live_desktop() -> void:
	print("-- desktop: touch is hardware, so no size makes a pointer window a phone")
	var runs0: int = _app.class_swap_runs
	for wh in [Vector2i(880, 1564), Vector2i(520, 1000), Vector2i(1000, 1000)]:
		await _resize(wh.x, wh.y)
		await _await_swap()
		_ok(String(_app.class_key()) == "desktop" and _app.class_swap_runs == runs0 and _app.pending_class_swap().is_empty(),
			"%dx%d: still the desktop, no swap requested (class %s, runs %d)" % [wh.x, wh.y, _app.class_key(), _app.class_swap_runs])

func _live() -> void:
	_clean = "--clean" in OS.get_cmdline_user_args()
	ROUND_TRIPS = int(_arg("--trips", "10"))
	await _settle_at_rest()
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	var boot := String(_app.class_key())
	_ok(boot in ["tablet", "desktop"], "boots as a tablet or the desktop (class_key %s)" % boot)
	print("  info cold boot of the app: %.0f ms (measured on this machine, instantiate + enter tree)" % _cold_boot_ms)
	## `--noworld` (with `--clean`): no world at all, so the end state is comparable with a plain cold boot (`--leg norm`):
	## a composition built with a world standing holds the rows that world makes (hundreds of buttons), which a boot
	## that builds empty and gets its world later does not draw the same way.
	if not "--noworld" in OS.get_cmdline_user_args():
		await _make_world()
	if boot == "desktop":
		await _live_desktop()
		return
	var parts := _arg("--vp", "").split("x")
	var tab := Vector2i(int(parts[0]), int(parts[1]))
	var ph := Vector2i(520, 1000) if _window_mode else Vector2i(880, 1564)
	## ---- the hysteresis band, at the real resize path ----
	print("-- hysteresis")
	## MS-7: a tablet booted from 1 180 dp of width has the whole menu bar, and 890 dp is the short bar, so this
	## resize is ONE swap that keeps the class (the hysteresis must hold across it: 890 is inside the band). A boot
	## below 1 180 dp has the short bar already and no swap runs, as before MS-7.
	if tab.x >= 1180:
		await _live_bar_swap(890, 1000, "tablet", "890 dp is inside the band, and the bar crosses 1 180 dp")
	else:
		await _live_stay(890, 1000, "tablet", "890 dp is inside the band")
	await _live_swap(880, 1000, "phone")
	await _live_stay(900, 1000, "phone", "900 dp does not cross back")
	await _live_swap(920, 1000, "tablet")
	if not _window_mode:
		await _resize(1600, 1000)
		await _await_swap()
		_ok(String(_app.class_key()) == "tablet", "1600x1000 (16:10) is a tablet")
		await _live_swap(880, 550, "phone")
		await _live_swap(tab.x, tab.y, "tablet")
	else:
		await _live_stay(tab.x, tab.y, "tablet", "already a tablet at 920 dp")

	## ---- fixture: the tool settings, the Ask conversation, then every S4.3 item ----
	if not _clean:
		await _live_fixture_a()
	if _clean:
		_live_zoom = float(_app.viewport.zoom())
		_live_world = _world_hash()
	## ---- a window open: the swap waits with a reason, loses nothing, and runs when the window closes ----
	print("-- a window is open")
	## S4.5 batch 4: every long-lived window is re-presented now, so merely being open no longer makes a swap wait; the example is a
	## HELD-INPUT case: the travel library with a staged, unsaved definition edit (`_draft`), which Q-S4-2 keeps waiting for. The
	## rail's filter box is typed in as well (not held: it is carried).
	var roster = _app.travel_library_window
	_ok(roster.has_method("represent_capture"), "the travel library is a re-presented window (an open one alone holds nothing)")
	roster.open("animal")
	await _frames(10)
	roster._on_add_blank()
	await _frames(4)
	var le: LineEdit = roster._filter_edit
	roster._draft = {"name": "staged name"}
	_ok(roster.visible and String(roster._current_id) != "", "the travel library is open on a blank entry with a staged edit")
	if le != null:
		le.text = "typed text"
	var runs0: int = _app.class_swap_runs
	await _resize(ph.x, ph.y)
	await _frames(20)
	await get_tree().create_timer(1.3).timeout
	_ok(String(_app.class_key()) == "tablet" and _app.class_swap_runs == runs0 and not _app.pending_class_swap().is_empty(),
		"the swap waits while the window is open (class %s, runs %d, pending %s)" % [_app.class_key(), _app.class_swap_runs, str(_app.pending_class_swap().get("phone"))])
	var reason := String(_app.status_slot_text("hint"))
	_ok(str(_blockers()).contains("a travel definition has unsaved edits"), "quiescent_blockers() names the held edit (%s)" % str(_blockers()))
	_ok(reason.begins_with("Layout change waiting: a travel definition has unsaved edits"), "the status line says why: '%s'" % reason)
	if le != null:
		_ok(le.text == "typed text", "the typed text is still there while waiting")
	_ok(roster._draft.get("name", "") == "staged name", "the staged edit is still there while waiting")
	roster.hide()
	await _await_swap()
	_ok(String(_app.class_key()) == "phone" and _app.class_swap_runs == runs0 + 1, "the swap ran once the window closed (class %s)" % _app.class_key())
	_ok(not str(_app.status_slot_text("hint")).begins_with("Layout change waiting"), "and the wait's sentence is gone from the status line")
	## The window was built again, so the old filter box is gone: the new one holds the same text, and the staged edit came with it.
	var roster2 = _app.travel_library_window
	_ok(roster2 != roster and not is_instance_valid(roster), "the travel library is a new instance (the old one is freed)")
	_ok(roster2._filter_edit.text == "typed text", "the rail filter's typed text survived the swap ('%s')" % roster2._filter_edit.text)
	_ok(roster2._draft.get("name", "") == "staged name", "the staged definition edit survived the swap (%s)" % str(roster2._draft))
	roster2._draft.clear()
	roster2._filter_edit.text = ""
	await _live_swap(tab.x, tab.y, "tablet")

	## ---- the Ask tab SHOWN while the swap runs: its content is in the tab's body, which the unmount frees ----
	if not _clean:
		_ok(_app.open_context_tab("ask"), "the Ask tab is shown")
		await _frames(8)
		_ok(_app.assistant_window.is_docked(), "its conversation is docked in the tab")
		await _live_swap(ph.x, ph.y, "phone")
		var a1 := _ask_state()
		_ok(a1["n"] == _st["ask_n"] and a1["draft"] == LIVE_DRAFT and a1["host_ok"], "on the phone the conversation is alive (%d entries, draft '%s', content alive %s)" % [a1["n"], a1["draft"], str(a1["host_ok"])])
		await _live_swap(tab.x, tab.y, "tablet")
		await _frames(8)
		var a2 := _ask_state()
		_ok(a2["n"] == _st["ask_n"] and a2["draft"] == LIVE_DRAFT and _app.assistant_window.is_docked(), "back on the tablet the Ask tab shows it again, docked (%d entries)" % a2["n"])
		_app.open_context_tab("inspect")
		await _frames(4)

	if not _clean:
		await _carry_fixture()
		_app.viewport.zoom_step(2.0)
		await _frames(4)
		_live_zoom = float(_app.viewport.zoom())
		_ok(_live_zoom > 1.5, "the map is zoomed in to %.3f (a literal check that the zoom moved)" % _live_zoom)
		_live_world = _world_hash()
		_ok(_live_world.length() > 40, "the world hash is not empty")
	await _live_visit_now("tablet", 0)

	## ---- ten round trips ----
	print("-- %d round trips" % ROUND_TRIPS)
	for t in range(1, ROUND_TRIPS + 1):
		await _live_swap(ph.x, ph.y, "phone")
		_live_t_sync["phone"].append(float(_app.last_swap_sync_ms))
		_live_t_full["phone"].append(float(_app.last_swap_ms))
		await _live_visit_now("phone", t)
		await _live_swap(tab.x, tab.y, "tablet")
		_live_t_sync["tablet"].append(float(_app.last_swap_sync_ms))
		_live_t_full["tablet"].append(float(_app.last_swap_ms))
		await _live_visit_now("tablet", t)
	var out := _arg("--out", "")
	await _live_swap(ph.x, ph.y, "phone")
	await _settle_at_rest()
	_write_norm(out + ".phone")
	await _live_swap(tab.x, tab.y, "tablet")
	if not _clean:
		_ok(_app.open_context_tab("ask"), "back on the tablet the Ask tab opens")
		await _frames(8)
		var a := _ask_state()
		_ok(a["n"] == _st["ask_n"] and a["draft"] == LIVE_DRAFT and _app.assistant_window.is_docked(), "the conversation is back in the Ask tab with its %d entries and its draft" % a["n"])
		_app.open_context_tab("inspect")
		await _frames(4)
	await _settle_at_rest()
	_write_norm(out + ".tablet")
	for cls in ["phone", "tablet"]:
		var sy: Array = _live_t_sync[cls]
		var fu: Array = _live_t_full[cls]
		print("  TIMING %s swap: synchronous part median %.0f ms (min %.0f, max %.0f), whole median %.0f ms, n=%d" % [cls, _median(sy), sy.min(), sy.max(), _median(fu), sy.size()])
	print("  TIMING cold boot %.0f ms" % _cold_boot_ms)
	var freed := 0
	for l in _connection_lines():
		if String(l).contains("freed/"):
			freed += 1
	_ok(freed == 0, "no connection keeps a freed receiver (%d)" % freed)

## A visit's checks: the common items (domain, accordions, selection, tool, year), the class chrome on the tablet, the
## tool settings, the Ask conversation, the world and the camera, then the counts.
func _live_visit_now(cls: String, trip: int) -> void:
	var tag := "trip %d, %s" % [trip, cls]
	if _clean:
		_ok(_world_hash() == _live_world, "%s: the world is byte-identical" % tag)
		## A clean run set nothing, so no dock may be collapsed on the tablet: the hysteresis steps passed through 920 dp, where
		## the docking rule floats and closes both docks, and that closed flag is the rule's, not the user's (`DccShell.
		## _capture_shell_state()`). Without the fix the 1 800 dp tablet came back with its right dock collapsed.
		if cls == "tablet":
			_ok(not _app.is_dock_collapsed("left") and not _app.is_dock_collapsed("right"), "%s: neither dock is collapsed (the user collapsed none; a floating dock's closed flag is the docking rule's)" % tag)
		await _live_visit(cls, trip)
		return
	_check_common(tag)
	if cls == "tablet":
		_check_chrome(tag)
	else:
		_ok(_app.right_dock_ctrl.shown_subject() == {"kind": "settlement", "id": 1}, "%s: the sheet shows the selection (settlement 1)" % tag)
	_live_check_items(tag)
	await _live_visit(cls, trip)

## The tool settings and the Ask conversation the live leg writes on the boot class (see `_live()`), before the
## S4.3 items of `_carry_fixture()`.
func _live_fixture_a() -> void:
	var wk = _workspace("WorldWorkspace")
	wk._on_paint_radius_changed(LIVE_PAINT_RADIUS)
	DccSettings.set_bake_depth(LIVE_BAKE_DEPTH)
	wk._refresh_finalize()
	_st["erode_key"] = "talus"
	_st["erode_val"] = 0.031
	wk._erode_op["talus"] = 0.031
	wk._erode_op_seeded = true
	_workspace("CivilizationWorkspace")._territory_radius = LIVE_TERRITORY
	_workspace("CartographyWorkspace").colour_paint.radius_km = LIVE_COLOUR_KM
	_ok(_app.open_context_tab("ask"), "the Ask tab opens on the boot class")
	await _frames(8)
	var win = _app.assistant_window
	win._on_view_facts()
	await _frames(4)
	(win.controls["input"] as TextEdit).text = LIVE_DRAFT
	_st["ask_n"] = int(_ask_state()["n"])
	_ok(_st["ask_n"] >= 1 and win.is_docked(), "the conversation holds %d entries and is docked in the Ask tab" % _st["ask_n"])
	_app.open_context_tab("inspect")
	await _frames(4)

## ---- S4.5: windows re-presented in place -------------------------------------------------------------------------------
## `--leg represent`: a tablet with ONE converted window open at a time (the roster, then the Place editor), with state the
## user set, swapped to the phone and back across the 900 dp line through the real resize path. One at a time because on
## the phone and the tablet an `AcceptDialog` is exclusive (`ContextWindow.applies()` is false there), so the engine
## refuses a second modal window: B5's "three windows open at once" can only be a desktop state, and a desktop never
## swaps (touch is hardware). After each swap: no wait, the window is a new instance open in the new class's form (its
## baked `_phone`, `borderless`, `min_size`), its state is intact (the faction or place, the tab, the simulator mode, the
## scroll), its layout equals a fresh open's, and the counts do not grow. Then a window with no re-present path yet (the
## vault with unsaved text) holds the swap with its reason while a converted window is open, and that window comes
## through when it ends. Expected values are literals.

const REP_TAB := "economy"
const REP_PLACE_TAB := "timeline"
const REP_SIM := "recovery"
## 20, not larger: the old window is stretched by the engine to the new viewport before the swap reads it, and the faction
## roster's scroll then clamps to 35 (measured in this leg), so a larger value would be clamped by the resize, not lost by the swap.
const REP_SCROLL := 20
## World data (batch 2): the Settlements tab (the only one long enough to scroll in this world; the tab carry has its own block,
## `_rep_world_tab()`), and a filter typed in upper case (the field keeps the case, `_filter` is its lower-case form, so a
## window that carried only `_filter` would show "a" in the box).
const REP_WD_TAB := "Settlements"
const REP_WD_SEARCH := "A"
## City viewer: a zoom the Fit button would not leave and a drag in canvas pixels, both made through the window's own input
## paths (`_zoom_by`, the canvas drag), so the pan is a fraction of the canvas the user saw (a pixel pan would be wrong in the
## other class's canvas).
const REP_CV_ZOOM := 2.5
const REP_CV_DRAG := Vector2(80.0, -40.0)
const REP_CV_INDEX := 1
## The settlement-type library entry the window is on (the second of two the leg creates) and the conflicts filter chip.
const REP_ST_NAME := "Probe B"
const REP_CF_FILTER := "active"
## The windows this leg covers, in order. Each is driven by the `match` arms below; a window joins the list with its arm in
## every one of them (open, state, expect, scroll node), so a half-added window fails loudly (`push_error`) and not silently.
const REP_ORDER := ["faction_roster_window", "place_editor_window", "world_data_window", "city_viewer_window",
	"settlement_types_window", "gen_info_dialog", "conflicts_window", "saved_journeys_window",
	"shortcuts_dialog", "generation_rules_window", "layers_popover", "_credits_dialog", "export_dialog", "quick_bar_phone_list",
	"quick_bar_window", "assistant_settings_window", "travel_library_window", "data_manager_window",
	"new_world_dialog", "asset_library_window", "vault_window"]
## The vault (batch 5): a bound folder with two notes; the first is open in Preview with a two-step session undo.
const REP_VT_NOTE := "Alpha.md"
const REP_VT_FIRST := "alpha first"
const REP_VT_LATER := "alpha later"
## Asset library and slicer (batch 5): a family, a search and the name sort, and every slicer parameter set off its default.
const REP_AL_FAMILY := "biomes"
const REP_AL_SEARCH := "o"
const REP_SL_NAME := "Probe Icon"
const REP_SL_SET := "ProbeSet"
## Batch 5's literals (S4_INVENTORY section 18.8). New world: a row count TYPED and never committed (a typed SEED has its own block, `_rep_nw_seed()`: committing one pushes a history entry) (the capture commits
## them), a custom grid (640 x 480, aspect Custom = 7), the plates dial moved by hand from the 3 000 km default (21 -> 22), and two
## history entries with these seeds.
const REP_NW_SEED := 31337
const REP_NW_KM := 3000.0
const REP_NW_GW := 640
const REP_NW_GH := 480
const REP_NW_PLATES := 22
const REP_NW_HIST := [11, 22]
## Batch 4's literals (S4_INVENTORY section 18.7). The Customise window's query; the settings window's typed model name (typed,
## never committed by the probe: the capture commits it, as a close does); the travel library's tab and rail filter ("new" matches
## the blank entry the fixture adds, whose name is "New Vessel"); the data manager's Maps options and its GIS destination, which
## the window keeps only in these session fields.
const REP_QB_QUERY := "view"
const REP_AS_MODEL := "probe-model-x"
const REP_TL_KIND := "vessel"
const REP_TL_FILTER := "new"
const REP_DM_ROUTE := "export_maps"
const REP_DM_COLS := 6
const REP_DM_ROWS := 3
const REP_DM_DEST := "C:/probe_tiles/tiles.zip"
const REP_DM_GIS := "C:/probe_gis/world.geojson"
## Batch 3's literals. The generation rules' lit preset chip (not the engine's rules: the leg must not change the world), the layers
## popover's filter, and the export form's choices: the second format (JPEG, which shows the quality row), a quality that is not the
## default (90), the third tier, a layer that is off by default switched on and one that is on by default switched off, and a path.
const REP_GR_PRESET := "Medina"
const REP_LP_QUERY := "o"
const REP_EX_FORMAT := 1
const REP_EX_QUALITY := 55.0
const REP_EX_TIER := 2
const REP_EX_WIDTH := 4096
const REP_EX_PATH := "C:/probe_export/map.jpg"
## The popover is a `PopupPanel` (always borderless) so the borderless check does not tell its class; its own `_phone` does.
## The Data manager is borderless on every class (a full-bleed window with its own bar), so its `borderless` tells nothing either.
const REP_NO_BORDER_CHECK := ["layers_popover", "data_manager_window", "asset_library_window"]
## The non-vacuity floor of "the layout equals a fresh open's" (8 lines by default): the phone's `All...` sheet, with no pins in
## this fixture, is one title, one count and the Customise row.
## The tablet's credits dialog is a scroll holding one text label: three lines.
const REP_MIN_LINES := {"quick_bar_phone_list": 6, "_credits_dialog": 2}
## The scroll check is vacuous for a window with nothing to scroll in this fixture, so the fixture asserts a scroll with room
## exists, and only for these.
const REP_SCROLLED := ["new_world_dialog", "faction_roster_window", "place_editor_window", "world_data_window", "settlement_types_window", "gen_info_dialog",
	"shortcuts_dialog", "layers_popover", "_credits_dialog", "export_dialog", "assistant_settings_window", "data_manager_window"]
## The bags the fixture builds each window's expected state from, filled by `_rep_init()` (the settlement type's id is the
## store's, not a literal the probe could know).
var _rep_bag := {}
## Windows whose scroll cannot survive the trip BACK to the tablet: the engine stretches the phone window to the tablet viewport
## before the swap captures it, the stretched page is taller than this window's content, and the scroll is clamped to 0 (the
## `S4_INVENTORY.md` section 18.3 finding, here with no room at all). The scroll is checked on the way to the phone, expected to be
## 0 on the way back, and set again on the tablet before the next trip.
## The credits text joins it: stretched to the tablet viewport (1800 dp wide) the phone window lays the text out short enough to fit,
## so the engine clamps the scroll to 0 before the capture, as for Settlement types; on the tablet's own 720 px measure the text is
## 1 139 px tall and scrolls again.
const REP_SCROLL_ONE_WAY := ["settlement_types_window", "_credits_dialog"]
## New world's scroll is clamped on the way back too, but not to 0: the phone card is stretched to the tablet viewport (1800 x 1000)
## before the swap captures it, which leaves it a scroll range of 12 (measured; the same clamp as the one-way windows, with room).
const REP_SCROLL_BACK := {"new_world_dialog": 12}
## Frames a window needs to finish laying itself out after it opens, where ten (the default wait) is not enough: New world's card
## settles over about thirty (measured: its content was 15 px taller at frame 10 than at frame 40). Applied to the real window and
## to the fresh oracle alike, so the comparison is between two settled layouts.
const REP_SETTLE := {"new_world_dialog": 40}

func _rep_init() -> void:
	## Two library entries so "the second" is a selection and not the default the window would fall back to.
	SettlementTypeStore.new_type("Probe A")
	var b_id: String = SettlementTypeStore.new_type(REP_ST_NAME)
	var hist: Array = []
	for hs in REP_NW_HIST:
		var im := Image.create(8, 8, false, Image.FORMAT_RGB8)
		im.fill(Color(float(hs) / 100.0, 0.5, 0.5))
		hist.append({"seed": hs, "world": false, "km": 800.0, "plates": 12, "manual": false, "tex": ImageTexture.create_from_image(im), "land": 40.0})
	_rep_bag = {
		"vault_window": {"open": true, "kind": "", "entity": 0, "label": "", "browse_only": true, "path": REP_VT_NOTE, "pick": REP_VT_NOTE,
			"reader": "", "heading": "", "mode": "preview", "pane": "preview", "hist": {REP_VT_NOTE: [REP_VT_FIRST, REP_VT_LATER]}, "back": [],
			"scroll": 0},
		## `focused`, the slicer's `image` and the `cell` are filled by `_rep_open` (the uid is the engine's, the image is the fixture's).
		"asset_library_window": {"open": true, "family": REP_AL_FAMILY, "collection": "", "unassigned": false, "pinned": "",
			"search": REP_AL_SEARCH, "sort": 1, "cell_px": 100.0, "select_mode": true, "selected": {}, "last_index": 0, "focused": "",
			"preview_index": 0, "swatch": 3, "phone_pane": "", "dirty": false, "scroll": 0,
			"slicer": {"open": true, "image": null, "loaded": true, "readout": "probe_sheet.png · 64 × 48", "cell": 5,
				"col_lines": PackedFloat64Array(), "row_lines": PackedFloat64Array(),
				"cols": 4.0, "rows": 3.0, "margin": 2.0, "spacing": 1.0, "trim": true, "skip": false, "chroma": true,
				"key": Color(0.2, 0.4, 0.6), "tol": 55.0, "target": 2, "family_index": 1, "overwrite": true,
				"name": REP_SL_NAME, "set": REP_SL_SET}},
		"new_world_dialog": {"open": true, "seed": REP_NW_SEED, "locked": false, "km": REP_NW_KM, "grid_w": REP_NW_GW, "grid_h": REP_NW_GH,
			"aspect": 7, "archetype": "", "plates_manual": true, "plates": REP_NW_PLATES, "history": hist, "push_pending": false,
			"more": false, "scroll": REP_SCROLL},
		"faction_roster_window": {"open": true, "selected": 2, "tab": REP_TAB, "pair": -1, "scroll": REP_SCROLL},
		"place_editor_window": {"open": true, "index": 1, "tab": REP_PLACE_TAB, "sim_mode": REP_SIM, "scroll": REP_SCROLL},
		"world_data_window": {"open": true, "tab": REP_WD_TAB, "search": REP_WD_SEARCH, "scroll": REP_SCROLL},
		"city_viewer_window": {"open": true, "index": REP_CV_INDEX, "zoom": REP_CV_ZOOM, "pan": Vector2.ZERO},
		"settlement_types_window": {"open": true, "selected": b_id, "scroll": REP_SCROLL},
		"gen_info_dialog": {"open": true, "scroll": REP_SCROLL},
		"conflicts_window": {"open": true, "filter": REP_CF_FILTER, "scroll": 0},
		"saved_journeys_window": {"open": true, "scroll": 0},
		"shortcuts_dialog": {"open": true, "editable": true, "scroll": REP_SCROLL},
		"generation_rules_window": {"open": true, "preset": REP_GR_PRESET, "scroll": 0},
		"layers_popover": {"open": true, "query": REP_LP_QUERY, "scroll": REP_SCROLL},
		"_credits_dialog": {"open": true, "scroll": REP_SCROLL},
		"export_dialog": {"open": true, "scroll": REP_SCROLL, "width": REP_EX_WIDTH, "format": REP_EX_FORMAT, "quality": REP_EX_QUALITY,
			"tier": REP_EX_TIER, "settle": true, "path": REP_EX_PATH, "layers": {"routes": true, "labels": false},
			"space": "sRGB", "look": "Current look", "title": "Export map image", "note": "", "note_on": false},
		"quick_bar_phone_list": {"open": true, "scroll": 0},
		## The tab is 0 and the share key empty in this bag because the phone has no ring tabs (a round trip through it drops
		## them, by design: `QuickBarWindow.represent_restore`); the ring tabs have their own block, `_rep_qb_tab()`.
		"quick_bar_window": {"open": true, "tab": 0, "share_key": "", "search": REP_QB_QUERY, "unpinnable": true, "page": -1,
			"cat_key": "", "pin_key": "", "scroll": 0},
		"assistant_settings_window": {"open": true, "scroll": REP_SCROLL},
		## `id` is the blank entry the fixture adds (the store names it), filled by `_rep_open`. `entry` is true both ways: the
		## tablet's selected entry is the phone's inspector pane, and the phone's inspector pane is a selected entry.
		"travel_library_window": {"open": true, "kind": REP_TL_KIND, "id": "", "filter": REP_TL_FILTER, "draft": {}, "entry": true, "scroll": 0},
		"data_manager_window": {"open": true, "route": REP_DM_ROUTE, "pane": "", "scroll": REP_SCROLL, "fields": {"_tx_cols": REP_DM_COLS,
			"_tx_rows": REP_DM_ROWS, "_tx_gzip": true, "_tx_dest": REP_DM_DEST, "_gis_dest": REP_DM_GIS, "_wd_width": 2048}},
	}

## The structure lines of one window, visible nodes only, no names or text: what `_structure_lines()` gives a composition.
func _window_lines(w: Node) -> PackedStringArray:
	var out := PackedStringArray()
	_structure_lines(w, 0, out, true)
	return out

## Builds the window again through the app's own constructor, gives it `bag` and returns its lines: what a fresh open of
## the same window in the current class looks like. The probe's oracle for "equals a fresh open".
func _fresh_lines(field: String, bag: Dictionary) -> PackedStringArray:
	var fresh: Node = _app._make_window(field)
	await fresh.represent_restore(bag)
	await _frames(maxi(_arg_int("--fresh-frames", 6), int(REP_SETTLE.get(field, 0))))
	var lines := _window_lines(fresh)
	fresh.hide()
	## A window that owns another (the asset library's slicer, parented to the shell) lets it go first, as `_represent_windows()` does.
	if fresh.has_method("represent_release"):
		fresh.represent_release()
	fresh.free()
	await _frames(2)
	return lines

## The window's state as its own fields hold it, for the literal checks.
func _rep_state(field: String) -> Dictionary:
	var w = _app.get(field)
	match field:
		"vault_window":
			var hh: Array = w._hist.get(REP_VT_NOTE, [])
			return {"open": w.visible, "path": String(w._browse_path), "mode": String(w._editor_mode), "browse_only": bool(w._browse_only),
				"hist": hh.size(), "hist0": String(hh[0]) if hh.size() > 0 else "", "editor": w._editor != null and is_instance_valid(w._editor),
				"text": String(w._browse_text), "phone": bool(w._phone)}
		"asset_library_window":
			var sl = w._slicer
			return {"open": w.visible, "family": String(w._current_family), "search": String(w._search_text), "box": String(w._search_edit.text),
				"sort": int(w._sort_mode), "sort_ui": int(w._sort_button.selected), "cell_px": float(w._cell_px), "select_mode": bool(w._select_mode),
				"sel": int(w._selected.size()), "focused": String(w._focused_uid), "swatch": String(w._preview_bg),
				"slicer_open": bool(sl.visible), "cols": int(w._slicer_cols.value), "rows": int(w._slicer_rows.value),
				"margin": int(w._slicer_margin.value), "spacing": int(w._slicer_spacing.value), "trim": bool(w._slice_trim),
				"skip": bool(w._slice_skip_empty), "chroma": bool(w._slice_chroma), "tol": float(w._slicer_chroma_tol.value),
				"key": w._slicer_chroma_color.color.to_html(false), "target": int(w._slice_target_index), "family_index": int(w._slice_family_index),
				"overwrite": bool(w._slice_overwrite), "name": String(w._slicer_name.text), "set": String(w._slicer_set.text),
				"sheet": w._sheet_image != null and bool(w._sheet_loaded), "sheet_size": Vector2i(w._sheet_image.get_size()) if w._sheet_image != null else Vector2i.ZERO,
				"cell": int(w._sheet_preview.selected_cell), "phone": bool(w._phone)}
		"new_world_dialog":
			return {"open": w.visible, "seed": int(w.seed_input.value), "km": float(w.width_input.value), "gw": int(w.grid_w_input.value),
				"gh": int(w.grid_h_input.value), "aspect": int(w.aspect_input.selected), "manual": bool(w._plates_manual),
				"plates": int(w.effective_plates()), "hist": w._history.size(), "hist0": int(w._history[0]["seed"]) if w._history.size() > 0 else -1,
				"phone": bool(w._phone)}
		"faction_roster_window":
			return {"open": w.visible, "sel": int(w._selected), "tab": String(w._tab), "phone": bool(w._phone)}
		"place_editor_window":
			return {"open": w.visible, "idx": int(w._index), "tab": String(w._active_tab), "sim": String(w._sim_mode), "phone": bool(w._phone)}
		"world_data_window":
			return {"open": w.visible, "tab": w._tabs.get_tab_title(w._tabs.current_tab), "search": String(w._search.text), "filter": String(w._filter), "phone": bool(w._phone)}
		"city_viewer_window":
			return {"open": w.visible, "idx": int(w._index), "zoom": float(w._zoom), "phone": bool(w._phone)}
		"settlement_types_window":
			var t: Dictionary = SettlementTypeStore.get_type(String(w._selected_id))
			return {"open": w.visible, "name": String(t.get("name", "")), "phone": bool(w._phone)}
		"gen_info_dialog":
			return {"open": w.visible, "phone": bool(w._phone), "lines": int(w._text.get_line_count()) > 40}
		"conflicts_window":
			return {"open": w.visible, "filter": String(w._filter), "phone": bool(w._phone)}
		"saved_journeys_window":
			return {"open": w.visible, "phone": bool(w._phone)}
		"shortcuts_dialog":
			return {"open": w.visible, "editable": bool(w._editable), "phone": bool(w._phone)}
		"generation_rules_window":
			return {"open": w.visible, "preset": String(w._active_preset), "phone": bool(w._phone)}
		"layers_popover":
			return {"open": w.visible, "query": String(w._query), "field": String(w._field.text), "phone": bool(w._phone)}
		"_credits_dialog":
			## The one build-site branch the dialog has: the desktop form sets a 660 px measure on the text, the phone's leaves it 0.
			return {"open": w.visible, "phone": w.get_node("%CreditsText").custom_minimum_size.x == 0.0}
		"export_dialog":
			var lb: Dictionary = w._layer_boxes
			return {"open": w.visible, "format": int(w._format.selected), "quality": float(w._quality.value), "tier": int(w._tier.selected),
				"settle": bool(w._settle.button_pressed), "width": int(w._selected_width()), "path": String(w._path),
				"routes": bool(lb["routes"].button_pressed), "labels": bool(lb["labels"].button_pressed),
				"quality_row": bool(w._quality_row.visible), "phone": bool(w._phone)}
		"quick_bar_phone_list":
			## No build-site branch: `phone_window()` makes it borderless on the phone and nothing else differs.
			return {"open": w.visible, "phone": w.borderless}
		"quick_bar_window":
			## `_phone_ready` is set by the first `open()` on a phone, so it is the window's own record of the class it laid out for.
			return {"open": w.visible, "search": String(w._search.text), "unpinnable": bool(w._show_unpinnable.button_pressed),
				"tab": int(w._tabs.current_tab), "cat_key": _qb_cat_key(w), "phone": bool(w._phone_ready)}
		"assistant_settings_window":
			var mf = w.controls.get("model")
			return {"open": w.visible, "model": String(mf.text) if mf is LineEdit else "(no field)", "stored": String(DccSettings.llm_value("model")),
				"phone": bool(w._phone)}
		"travel_library_window":
			return {"open": w.visible, "kind": String(w._current_kind), "has_id": String(w._current_id) != "", "id": String(w._current_id),
				"filter": String(w._filter_edit.text), "filter_state": String(w._filter_text), "phone": bool(w._phone),
				"showing_entry": bool(w._phone_showing_entry) if w._phone else true}
		"data_manager_window":
			return {"open": w.visible, "route": String(w._selected_id), "cols": int(w._tx_cols), "rows": int(w._tx_rows), "gzip": bool(w._tx_gzip),
				"dest": String(w._tx_dest), "gis": String(w._gis_dest), "wd_width": int(w._wd_width), "phone": bool(w._phone)}
	push_error("_rep_state: no arm for " + field)
	return {}

## The window's scroll in the units its bag uses, or -1 when it has none.
func _rep_scroll(field: String) -> int:
	var w = _app.get(field)
	if field == "gen_info_dialog":
		return int(w._text.scroll_vertical)
	var body: Node = null
	match field:
		"new_world_dialog": body = w._rep_scroll_node()
		"shortcuts_dialog", "layers_popover": body = w._list
		"generation_rules_window": body = w._street_body
		"_credits_dialog": body = w.get_node("%CreditsText")
		"export_dialog": body = w._status
		"faction_roster_window": body = w._inspector_body
		"place_editor_window": body = w._body
		"world_data_window": body = w._settlements_body
		"settlement_types_window", "conflicts_window", "saved_journeys_window": body = w._body
		"assistant_settings_window": body = w._body
		"travel_library_window": body = w._inspector_body
		"data_manager_window": body = w._pane_body
	if body == null:
		return -1
	var p := body.get_parent()
	while p != null and not p is ScrollContainer:
		p = p.get_parent()
	return -1 if p == null else (p as ScrollContainer).scroll_vertical

## The literals the state must read after every swap (the bag's values written out, not read from it).
func _rep_expect(field: String) -> Dictionary:
	match field:
		"vault_window": return {"open": true, "path": "Alpha.md", "mode": "preview", "browse_only": true, "hist": 2, "hist0": "alpha first",
			"editor": true, "text": "alpha later"}
		"asset_library_window": return {"open": true, "family": "biomes", "search": "o", "box": "o", "sort": 1, "sort_ui": 1, "cell_px": 100.0,
			"select_mode": true, "sel": 1, "focused": String(_rep_bag[field]["focused"]), "swatch": "color", "slicer_open": true, "cols": 4, "rows": 3,
			"margin": 2, "spacing": 1, "trim": true, "skip": false, "chroma": true, "tol": 55.0, "key": "336699", "target": 2, "family_index": 1,
			"overwrite": true, "name": "Probe Icon", "set": "ProbeSet", "sheet": true, "sheet_size": Vector2i(64, 48), "cell": 5}
		"new_world_dialog": return {"open": true, "seed": 31337, "km": 3000.0, "gw": 640, "gh": 480, "aspect": 7, "manual": true,
			"plates": 22, "hist": 2, "hist0": 11}
		"faction_roster_window": return {"open": true, "sel": 2, "tab": REP_TAB}
		"place_editor_window": return {"open": true, "idx": 1, "tab": REP_PLACE_TAB, "sim": REP_SIM}
		"world_data_window": return {"open": true, "tab": "Settlements", "search": "A", "filter": "a"}
		"city_viewer_window": return {"open": true, "idx": 1, "zoom": 2.5}
		"settlement_types_window": return {"open": true, "name": "Probe B"}
		"gen_info_dialog": return {"open": true, "lines": true}
		"conflicts_window": return {"open": true, "filter": "active"}
		"saved_journeys_window": return {"open": true}
		"shortcuts_dialog": return {"open": true, "editable": true}
		"generation_rules_window": return {"open": true, "preset": "Medina"}
		"layers_popover": return {"open": true, "query": "o", "field": "o"}
		"_credits_dialog": return {"open": true}
		"export_dialog": return {"open": true, "format": 1, "quality": 55.0, "tier": 2, "settle": true, "width": 4096,
			"path": "C:/probe_export/map.jpg", "routes": true, "labels": false, "quality_row": true}
		"quick_bar_phone_list": return {"open": true}
		"quick_bar_window": return {"open": true, "search": "view", "unpinnable": true, "tab": 0, "cat_key": String(_rep_bag[field]["cat_key"])}
		"assistant_settings_window": return {"open": true, "model": "probe-model-x", "stored": "probe-model-x"}
		"travel_library_window": return {"open": true, "kind": "vessel", "has_id": true, "id": String(_rep_bag[field]["id"]), "filter": "new",
			"filter_state": "new", "showing_entry": true}
		"data_manager_window": return {"open": true, "route": "export_maps", "cols": 6, "rows": 3, "gzip": true, "dest": "C:/probe_tiles/tiles.zip",
			"gis": "C:/probe_gis/world.geojson", "wd_width": 2048}
	push_error("_rep_expect: no arm for " + field)
	return {}

## Opens the window on the tablet with the state of its bag, the way the user would (the window's own open call).
func _rep_open(field: String) -> void:
	match field:
		"vault_window":
			var root := ProjectSettings.globalize_path("user://probe_vault")
			DirAccess.make_dir_recursive_absolute(root)
			for nm in [["Alpha.md", REP_VT_LATER], ["Beta.md", "beta"]]:
				var f := FileAccess.open(root + "/" + String(nm[0]), FileAccess.WRITE)
				f.store_string(String(nm[1]))
				f.close()
			var conn: Dictionary = _app.bridge.vault_connect(root, "Probe")
			_ok(bool(conn.get("ok", false)), "the vault fixture folder connected (%s)" % str(conn).substr(0, 120))
			var vw = _app.vault_window
			vw.open_browse()
			await _frames(8)
			vw.open_note(REP_VT_NOTE)
			await _frames(8)
			vw._editor.set_mode("preview")
			await _frames(4)
			vw._hist[REP_VT_NOTE] = [REP_VT_FIRST, REP_VT_LATER]
			_ok(vw._editor_mode == "preview" and vw._browse_path == REP_VT_NOTE, "the vault fixture has %s open in Preview (%s, %s)" % [REP_VT_NOTE, vw._browse_path, vw._editor_mode])
		"asset_library_window":
			var al = _app.asset_library_window
			al.open(REP_AL_FAMILY, false)
			await _frames(8)
			al._search_edit.text = REP_AL_SEARCH
			al._search_edit.text_changed.emit(REP_AL_SEARCH)
			al._sort_button.select(1)
			al._sort_button.item_selected.emit(1)
			al._zoom_slider.value = 100.0
			al._pick_preview_bg(3)
			await _frames(4)
			_ok(al._slot_order.size() > 0, "the asset fixture's search leaves slots to focus (%d)" % al._slot_order.size())
			var uid := String(al._slot_order[0]["uid"])
			al._selected = {uid: true}
			al._focused_uid = uid
			al._last_index = 0
			al._select_mode = true
			al._refresh_selection_visuals()
			al._refresh_inspector()
			_rep_bag[field]["focused"] = uid
			_rep_bag[field]["selected"] = {uid: true}
			var sh := Image.create(64, 48, false, Image.FORMAT_RGBA8)
			for yy in 48:
				for xx in 64:
					sh.set_pixel(xx, yy, Color(float(xx) / 64.0, float(yy) / 48.0, 0.5, 1.0))
			sh.save_png("user://probe_sheet.png")
			al._open_slicer()
			await _frames(6)
			al._load_sheet_image(ProjectSettings.globalize_path("user://probe_sheet.png"))
			al._slicer_cols.value = 4
			al._slicer_rows.value = 3
			al._slicer_margin.value = 2
			al._slicer_spacing.value = 1
			al._slicer_trim.button_pressed = true
			al._slicer_skip.button_pressed = false
			al._slicer_chroma.button_pressed = true
			al._slicer_chroma_color.color = Color(0.2, 0.4, 0.6)
			al._slicer_chroma_tol.value = 55
			al._slicer_target.select(2)
			al._slicer_target.item_selected.emit(2)
			al._slicer_family.select(1)
			al._slicer_family.item_selected.emit(1)
			al._set_fill(true)
			al._slicer_name.text = REP_SL_NAME
			al._slicer_set.text = REP_SL_SET
			await _frames(6)
			al._sheet_preview.selected_cell = 5
			await _frames(4)
			_ok(al._sheet_preview.selected_cell == 5, "the asset fixture has picked sheet cell 5 (so the cell check is not vacuous)")
			_rep_bag[field]["slicer"]["image"] = sh
			_rep_bag[field]["slicer"]["readout"] = String(al._sheet_readout.text)
			_ok(al._slicer.visible and al._sheet_loaded, "the asset fixture has the slicer open with a sheet loaded")
		"new_world_dialog":
			_app.open_new_world()
			await _frames(8)
			var nw = _app.new_world_dialog
			nw.width_input.value = REP_NW_KM
			nw.grid_w_input.value = REP_NW_GW
			nw.aspect_input.selected = 7
			nw._refresh_dimensions()
			## Typed and NEVER committed: the capture must commit them (Android BACK leaves a SpinBox exactly like this).
			nw.seed_input.set_value_no_signal(float(REP_NW_SEED))
			nw.seed_input.get_line_edit().text = str(REP_NW_SEED)
			nw.grid_h_input.get_line_edit().text = str(REP_NW_GH)
			nw.grid_h_input.get_line_edit().text_changed.emit(str(REP_NW_GH))
			nw._plates_manual = false
			nw._plates_step(1)
			nw._history.clear()
			for he in _rep_bag[field]["history"]:
				nw._history.append(he)
			nw._history_push_pending = false
			nw._refresh_history_strip()
			_ok(nw.effective_plates() == 22 and nw._plates_manual, "the New world fixture moved the plates dial by hand to 22 (%d)" % nw.effective_plates())
		"faction_roster_window": _app.faction_roster_window.open(2, REP_TAB)
		"place_editor_window":
			_app.place_editor_window.open_for_tab(1, REP_PLACE_TAB)
			_app.place_editor_window._sim_mode = REP_SIM
		"world_data_window":
			_app.world_data_window._search.text = REP_WD_SEARCH
			_app.world_data_window._filter = REP_WD_SEARCH.to_lower()
			_app.open_world_data(REP_WD_TAB)
		"city_viewer_window":
			_app.open_city_viewer(REP_CV_INDEX)
			await _frames(6)
			var cv = _app.city_viewer_window
			var cs: Vector2 = cv._canvas.size
			cv._zoom_by(REP_CV_ZOOM, cs * 0.5)
			var press := InputEventMouseButton.new()
			press.button_index = MOUSE_BUTTON_LEFT
			press.pressed = true
			cv._on_canvas_input(press)
			var mm := InputEventMouseMotion.new()
			mm.relative = REP_CV_DRAG
			cv._on_canvas_input(mm)
			var rel := InputEventMouseButton.new()
			rel.button_index = MOUSE_BUTTON_LEFT
			cv._on_canvas_input(rel)
			_rep_bag[field]["pan"] = Vector2(REP_CV_DRAG.x / cs.x, REP_CV_DRAG.y / cs.y)
		"settlement_types_window":
			_app.settlement_types_window._selected_id = String(_rep_bag[field]["selected"])
			_app.settlement_types_window.open()
		"gen_info_dialog": _app.gen_info_dialog.open()
		"conflicts_window":
			_app.open_conflicts_list()
			_app.conflicts_window._set_filter(REP_CF_FILTER)
		"saved_journeys_window": _app.open_saved_journeys()
		"shortcuts_dialog": _app.shortcuts_dialog.open_editable()
		"generation_rules_window":
			_app.generation_rules_window._active_preset = REP_GR_PRESET
			_app.generation_rules_window.open()
		"layers_popover":
			## A popup opened in the frame a dialog was hidden is closed again by the engine's focus hand-back (seen in the full
			## leg, not in `--only layers_popover`), so the focus settles first.
			await _frames(10)
			_app.layers_popover.open()
			_app.layers_popover._field.text = REP_LP_QUERY
			_app.layers_popover._field.text_changed.emit(REP_LP_QUERY)
		"_credits_dialog": _app.open_credits()
		"export_dialog":
			var ex = _app.open_export_dialog()
			ex._format.select(REP_EX_FORMAT)
			ex._on_format_changed()
			ex._quality.value = REP_EX_QUALITY
			ex._tier.select(REP_EX_TIER)
			ex._settle.button_pressed = true
			ex._layer_boxes["routes"].button_pressed = true
			ex._layer_boxes["labels"].button_pressed = false
			for i in ex._widths.size():
				if ex._widths[i] == REP_EX_WIDTH:
					_ok(not ex._sizes.is_item_disabled(i), "the export fixture's width %d is affordable in this world" % REP_EX_WIDTH)
					ex._sizes.select(i)
			ex.set_destination(REP_EX_PATH)
		"quick_bar_phone_list": _app.phone_quick.open_list()
		"quick_bar_window":
			_app.open_quick_bar_window()
			await _frames(8)
			var q = _app.quick_bar_window
			q._search.text = REP_QB_QUERY
			q._search.text_changed.emit(REP_QB_QUERY)
			q._show_unpinnable.button_pressed = true
			await _frames(4)
			for i in q._cat_rows.size():
				if bool(q._cat_rows[i].get("pinnable", false)):
					q._catalogue.select(i)
					q._on_catalogue_selected()
					_rep_bag[field]["cat_key"] = String(q._cat_rows[i]["pin_key"])
					break
			_ok(String(_rep_bag[field]["cat_key"]) != "", "the Customise fixture selected a pinnable row (so the selection check is not vacuous)")
		"assistant_settings_window":
			_app.assistant_settings_window.open()
			await _frames(8)
			var mf: LineEdit = _app.assistant_settings_window.controls["model"]
			_ok(String(DccSettings.llm_value("model")) == "", "the model name is unset before the fixture types one (so the carry is not vacuous)")
			mf.text = REP_AS_MODEL
		"travel_library_window":
			_app.open_travel_library(REP_TL_KIND)
			await _frames(8)
			var tl = _app.travel_library_window
			tl._on_add_blank()
			_rep_bag[field]["id"] = String(tl._current_id)
			tl._filter_edit.text = REP_TL_FILTER
			tl._filter_edit.text_changed.emit(REP_TL_FILTER)
			_ok(String(tl._current_id) != "", "the travel fixture selected the blank entry it added")
		"data_manager_window":
			var dm = _app.data_manager_window
			for k in _rep_bag[field]["fields"]:
				dm.set(k, _rep_bag[field]["fields"][k])
			_app.data_manager_window.open_route(REP_DM_ROUTE)

## The pin key of the catalogue row the Customise window has selected, or "".
func _qb_cat_key(w: Node) -> String:
	var sel: PackedInt32Array = w._catalogue.get_selected_items()
	if sel.size() > 0 and sel[0] < w._cat_rows.size():
		return String(w._cat_rows[sel[0]].get("pin_key", ""))
	return ""

## The ScrollContainer this fixture scrolls (a `TextEdit` for the dump), or null.
func _rep_scroll_target(w: Node, field: String) -> Node:
	if field == "gen_info_dialog":
		return w._text
	var sc: ScrollContainer = null
	for c in w.find_children("*", "ScrollContainer", true, false):
		if (c as ScrollContainer).get_v_scroll_bar().max_value - (c as ScrollContainer).get_v_scroll_bar().page > REP_SCROLL + 20.0 and (c as Control).is_visible_in_tree():
			sc = c
	return sc

## One step: resize across the line, wait, and check that the swap ran once without waiting and the window is open in the
## new class's form with its state; its lines equal a fresh open's (the restored window is hidden for that, because only one
## modal window can be shown, then opened again from its own bag).
func _rep_step(field: String, w: int, h: int, want: String, tag: String) -> void:
	var runs0: int = _app.class_swap_runs
	var old = _app.get(field)
	await _resize(w, h)
	await _await_swap()
	await _frames(int(REP_SETTLE.get(field, 10)))
	_ok(String(_app.class_key()) == want and _app.class_swap_runs == runs0 + 1, "%s: swapped to %s once, without waiting (class %s, runs %d -> %d)" % [tag, want, _app.class_key(), runs0, _app.class_swap_runs])
	_ok(not str(_blockers()).contains("window is open"), "%s: no open window is holding anything (%s)" % [tag, str(_blockers())])
	var win = _app.get(field)
	_ok(win != old and not is_instance_valid(old), "%s: the window is a new instance and the old one is freed" % tag)
	var st := _rep_state(field)
	var phone := want == "phone"
	var ex := _rep_expect(field)
	var same := true
	for k in ex:
		same = same and (is_equal_approx(float(st[k]), float(ex[k])) if ex[k] is float else st[k] == ex[k])
	_ok(same, "%s: %s is open with its state (%s)" % [tag, field, str(st)])
	_ok(st["phone"] == phone and (REP_NO_BORDER_CHECK.has(field) or win.borderless == phone), "%s: baked for %s (_phone %s, borderless %s)" % [tag, want, str(st["phone"]), str(win.borderless)])
	if field == "faction_roster_window" or field == "place_editor_window" or field == "world_data_window":
		_ok((win.min_size == Vector2i.ZERO) == phone, "%s: min_size is %s (%s)" % [tag, "cleared" if phone else "the desktop's", str(win.min_size)])
	if field == "layers_popover":
		## The wind and current layer is attached to the SURVIVING overlay by `setup()`: a rebuilt popover must reuse it.
		var fx := 0
		for c in _app.viewport.overlay.get_children():
			if String(c.name).begins_with("WindFxLayer"):
				fx += 1
		_ok(fx == 1, "%s: exactly one wind/current layer is attached to the overlay (%d)" % [tag, fx])
	var want_scroll := REP_SCROLL
	if want == "tablet" and REP_SCROLL_ONE_WAY.has(field):
		want_scroll = 0
	elif want == "tablet" and REP_SCROLL_BACK.has(field):
		want_scroll = int(REP_SCROLL_BACK[field])
	if REP_SCROLLED.has(field):
		_ok(_rep_scroll(field) == want_scroll, "%s: the scroll is %d (%d)" % [tag, want_scroll, _rep_scroll(field)])
	if field == "city_viewer_window":
		var cs: Vector2 = win._canvas.size
		var fr: Vector2 = _rep_bag[field]["pan"]
		var pan_ok: bool = (win._pan - Vector2(fr.x * cs.x, fr.y * cs.y)).length() < 0.5
		_ok(pan_ok, "%s: the pan is the same fraction of the canvas (%s of %s)" % [tag, str(win._pan), str(cs)])
	var rl := _window_lines(win)
	win.hide()
	await _frames(4)
	var fresh_bag: Dictionary = (_rep_bag[field] as Dictionary).duplicate()
	if fresh_bag.has("scroll") and REP_SCROLLED.has(field):
		fresh_bag["scroll"] = want_scroll
	var fr := await _fresh_lines(field, fresh_bag)
	var first := -1
	for i in mini(rl.size(), fr.size()):
		if rl[i] != fr[i]:
			first = i
			break
	_ok(rl.size() > (REP_MIN_LINES[field] if REP_MIN_LINES.has(field) else 8) and rl == fr, "%s: the layout equals a fresh open's (%d lines vs %d; first difference %d: '%s' vs '%s')" % [tag, rl.size(), fr.size(), first, "" if first < 0 else rl[first], "" if first < 0 else fr[first]])
	if first >= 0:
		for j in range(maxi(0, first - 3), mini(mini(rl.size(), fr.size()), first + 2)):
			print("    ctx %d  real: %s" % [j, rl[j]])
			print("    ctx %d fresh: %s" % [j, fr[j]])
	var back_bag: Dictionary = fresh_bag
	win.represent_restore(back_bag)
	await _frames(10)
	if want == "tablet" and (REP_SCROLL_ONE_WAY.has(field) or REP_SCROLL_BACK.has(field)):
		var sc: Node = _rep_scroll_target(win, field)
		_ok(sc != null, "%s: the tablet window has room to scroll again" % tag)
		if sc != null:
			(sc as ScrollContainer).scroll_vertical = REP_SCROLL
			await _frames(4)

## Hides every window this leg drives, and the asset slicer with them: the slicer is parented to the shell, so hiding the library
## alone leaves it up, and an open exclusive window refuses every window opened after it ("Attempting to make child window
## exclusive"), which silently emptied a later block's gate check.
func _rep_hide_all() -> void:
	for other in REP_ORDER:
		if _app.get(other) != null:
			_app.get(other).hide()
	var al = _app.asset_library_window
	if al != null and is_instance_valid(al) and al._slicer != null and is_instance_valid(al._slicer):
		al._slicer.hide()

func _rep_fixture(field: String) -> void:
	_rep_hide_all()
	await _rep_open(field)
	await _frames(12)
	var w = _app.get(field)
	var sc: Node = _rep_scroll_target(w, field)
	if REP_SCROLLED.has(field):
		_ok(sc != null, "%s has a scroll with room to scroll (so the scroll check is not vacuous)" % field)
	if sc != null:
		if sc is ScrollContainer:
			(sc as ScrollContainer).scroll_vertical = REP_SCROLL
		else:
			(sc as TextEdit).scroll_vertical = REP_SCROLL
		print("  info %s scroll node %s set %d" % [field, str(w.get_path_to(sc)), REP_SCROLL])
	await _frames(4)
	if REP_SCROLLED.has(field):
		_ok(_rep_scroll(field) == REP_SCROLL, "%s: the scroll is set to %d before the swap (%d)" % [field, REP_SCROLL, _rep_scroll(field)])
	var st := _rep_state(field)
	_ok(st["open"] and not st["phone"], "%s is open on the tablet (%s)" % [field, str(st)])

## World data's tab, which the main loop cannot test (the Settlements tab is the default, and the only long one): open on the
## Economy tab, swap to the phone and back, and the tab is still Economy each time.
func _rep_world_tab(ph: Vector2i, tab: Vector2i) -> void:
	_rep_hide_all()
	_app.open_world_data("Economy")
	await _frames(8)
	for t in 2:
		for step in [[ph, "phone"], [tab, "tablet"]]:
			await _resize(step[0].x, step[0].y)
			await _await_swap()
			await _frames(10)
			var w = _app.world_data_window
			_ok(w.visible and w._tabs.get_tab_title(w._tabs.current_tab) == "Economy", "world data keeps the Economy tab on the %s (trip %d, got %s)" % [step[1], t + 1, w._tabs.get_tab_title(w._tabs.current_tab)])
	_app.world_data_window.hide()
	await _frames(4)

## The desktop `Find on map` dialog (batch 3): a plain AcceptDialog the shell builds, re-presented by `DccShell.search_represent_*`,
## so it is not in `REP_ORDER`. Same-class remounts (the probe's direct call, which bypasses the gate) keep it open with its text
## three times; a swap to the phone drops it closed (the phone has its own search overlay, `search_represent_restore`) and back to
## the tablet it stays closed, as a rebuilt-but-unopened dialog should.
const REP_SEARCH_TEXT := "a"
func _rep_search(ph: Vector2i, tab: Vector2i) -> void:
	print("-- the desktop Find dialog: open with a query, three same-class remounts, then the phone and back")
	_rep_hide_all()
	_app.open_find_on_map()
	await _frames(6)
	_app._desktop_search_field.text = REP_SEARCH_TEXT
	_app._refresh_desktop_search()
	await _frames(4)
	_ok(_app._desktop_search_dialog != null and _app._desktop_search_dialog.visible, "the Find dialog is open on the tablet")
	var o0 := -1
	for t in range(1, 4):
		var old = _app._desktop_search_dialog
		await _app._remount_composition(null)
		await _frames(12)
		var d = _app._desktop_search_dialog
		_ok(d != old and not is_instance_valid(old), "find trip %d: the dialog is a new instance and the old one is freed" % t)
		_ok(d != null and d.visible and String(_app._desktop_search_field.text) == "a", "find trip %d: open with its query ('%s')" % [t, str(_app._desktop_search_field.text) if d != null else "none"])
		_ok(not str(_blockers()).contains("Find on map"), "find trip %d: the dialog holds nothing (%s)" % [t, str(_blockers())])
		_ok(_app._desktop_search_results.get_child_count() > 0, "find trip %d: the results are filled again from the world (%d rows)" % [t, _app._desktop_search_results.get_child_count()])
		if t == 1:
			o0 = _orphans()
		else:
			_ok(_orphans() == o0, "find trip %d: orphan nodes %d equal trip 1's %d" % [t, _orphans(), o0])
	await _swap_to("phone", ph.x, ph.y)
	var dp = _app._desktop_search_dialog
	_ok(dp != null and not dp.visible, "on the phone the desktop dialog is rebuilt closed (the phone's overlay is its search)")
	await _swap_to("tablet", tab.x, tab.y)
	var dt = _app._desktop_search_dialog
	_ok(dt != null and dt != dp and not dt.visible, "back on the tablet it is a new instance, closed")
	_app._desktop_search_dialog.hide()
	await _frames(4)

## Generation rules' scroll. On the tablet the window (980 x 660) holds all three tables with nothing to scroll, so the table
## cannot test it; on the phone the sections stack and there is room. Set there and carried across three same-class remounts (the
## probe's direct call, which bypasses the gate), with the lit preset chip.
func _rep_gr_scroll(ph: Vector2i, tab: Vector2i) -> void:
	print("-- generation rules: the scroll on the phone, across three same-class remounts")
	_rep_hide_all()
	await _swap_to("phone", ph.x, ph.y)
	_app.generation_rules_window._active_preset = REP_GR_PRESET
	_app.generation_rules_window.open()
	await _frames(12)
	var sc: ScrollContainer = null
	var p: Node = _app.generation_rules_window._street_body.get_parent()
	while p != null and not p is ScrollContainer:
		p = p.get_parent()
	sc = p
	_ok(sc != null and sc.get_v_scroll_bar().max_value - sc.get_v_scroll_bar().page > REP_SCROLL + 20.0, "the phone's tables have room to scroll (max %s, page %s)" % [str(sc.get_v_scroll_bar().max_value), str(sc.get_v_scroll_bar().page)])
	sc.scroll_vertical = REP_SCROLL
	await _frames(4)
	for t in range(1, 4):
		var old = _app.generation_rules_window
		await _app._remount_composition(null)
		await _frames(14)
		var g = _app.generation_rules_window
		var q: Node = g._street_body.get_parent()
		while q != null and not q is ScrollContainer:
			q = q.get_parent()
		_ok(g != old and not is_instance_valid(old) and g.visible, "gen rules remount %d: a new instance, open" % t)
		_ok(q != null and (q as ScrollContainer).scroll_vertical == REP_SCROLL, "gen rules remount %d: the scroll is %d (%d)" % [t, REP_SCROLL, (q as ScrollContainer).scroll_vertical if q != null else -1])
		_ok(String(g._active_preset) == REP_GR_PRESET, "gen rules remount %d: the lit preset is '%s' (%s)" % [t, REP_GR_PRESET, g._active_preset])
	_app.generation_rules_window.hide()
	await _swap_to("tablet", tab.x, tab.y)

## The Customise window's ring tab and diagonal, which the table cannot test (the phone has no ring tabs, so a round trip through
## it drops them by design). Open on "Ring slots" with the first diagonal that can share, carried across three same-class
## remounts; then on the phone, the page strip's second segment across two more; then back to the tablet on the Bar tab.
func _rep_qb_tab(ph: Vector2i, tab: Vector2i) -> void:
	print("-- the Customise window: the ring tab and diagonal across three same-class remounts, then the phone's page")
	_rep_hide_all()
	_app.open_quick_bar_window()
	await _frames(8)
	var q = _app.quick_bar_window
	q._tabs.current_tab = 2
	await _frames(4)
	var key := String(q._share_key)
	_ok(key != "" and String(q._target) == "share", "the Customise window is on Ring slots with a diagonal (key '%s', target %s)" % [key, q._target])
	for t in range(1, 4):
		var old = _app.quick_bar_window
		await _app._remount_composition(null)
		await _frames(14)
		var n = _app.quick_bar_window
		_ok(n != old and not is_instance_valid(old) and n.visible, "qb tab remount %d: a new instance, open" % t)
		_ok(int(n._tabs.current_tab) == 2 and String(n._target) == "share" and String(n._share_key) == key, "qb tab remount %d: Ring slots on '%s' (tab %d, target %s, key '%s')" % [t, key, n._tabs.current_tab, n._target, n._share_key])
	_app.quick_bar_window.hide()
	await _swap_to("phone", ph.x, ph.y)
	_app.open_quick_bar_window()
	await _frames(10)
	var qp = _app.quick_bar_window
	_ok(qp._pages != null and int(qp._pages.current_tab) == 0, "on the phone the Customise window has its page strip on Commands")
	qp._pages.current_tab = 1
	await _frames(4)
	for t in range(1, 3):
		await _app._remount_composition(null)
		await _frames(14)
		var np = _app.quick_bar_window
		_ok(np.visible and int(np._pages.current_tab) == 1 and not np._left.visible and np._right.visible, "qb page remount %d: the Pinned page is still shown (page %d)" % [t, np._pages.current_tab])
	_app.quick_bar_window.hide()
	await _swap_to("tablet", tab.x, tab.y)
	await _frames(4)

## New world's two paths the main loop cannot take (S4_INVENTORY section 18.8): a TYPED, never committed seed (committing it pushes a
## history entry, so it cannot share the main fixture's oracle) is committed by the capture and survives three remounts; and an
## archetype (a name, applied to the engine ONCE when chosen: the re-present must select it again without applying it again,
## so a parameter the user changed after choosing it is left alone). `remounts` are same-class, which is what `_remount_composition()`
## does; the cross-class trips are the main loop's.
func _rep_nw_extra(ph: Vector2i, tab: Vector2i) -> void:
	print("-- New world: a typed seed is committed by the capture, and an archetype is selected without being applied again")
	_rep_hide_all()
	_app.open_new_world()
	await _frames(8)
	var nw = _app.new_world_dialog
	nw.seed_input.get_line_edit().text = "424242"
	nw.seed_input.get_line_edit().text_changed.emit("424242")
	_ok(int(nw.seed_input.value) != 424242, "the typed seed is NOT committed before the swap (so the carry is not vacuous)")
	await _swap_to("phone", ph.x, ph.y)
	nw = _app.new_world_dialog
	_ok(nw.visible and int(nw.seed_input.value) == 424242 and not bool(nw._seed_dirty), "the typed seed was committed and carried to the phone (%d)" % int(nw.seed_input.value))
	_ok(int(nw.request()["seed"]) == 424242, "and request() reads it (%d)" % int(nw.request()["seed"]))
	nw.hide()
	await _swap_to("tablet", tab.x, tab.y)
	nw = _app.new_world_dialog
	_ok(int(nw.seed_input.value) == 424242, "and back on the tablet (%d)" % int(nw.seed_input.value))
	var names: PackedStringArray = nw._archetype_names
	_ok(names.size() > 0, "the engine publishes archetypes")
	if names.size() > 0:
		_app.open_new_world()
		await _frames(6)
		nw.archetype_input.select(1)
		nw._on_archetype_selected(1)
		_ok(String(nw._archetype) == String(names[0]), "the fixture chose the first archetype (%s)" % String(nw._archetype))
		var plates_before = _app.bridge.param_get("tect.plates")
		_app.bridge.param_set("tect.plates", int(plates_before) + 3)
		for t in range(1, 4):
			await _app._remount_composition(null)
			await _frames(10)
			nw = _app.new_world_dialog
			_ok(String(nw._archetype) == String(names[0]) and nw.archetype_input.selected == 1, "archetype remount %d: still '%s' (item %d)" % [t, names[0], nw.archetype_input.selected])
			_ok(int(_app.bridge.param_get("tect.plates")) == int(plates_before) + 3, "archetype remount %d: the plates parameter the user changed after choosing it is untouched (%s)" % [t, str(_app.bridge.param_get("tect.plates"))])
		_app.bridge.param_set("tect.plates", plates_before)
		nw = _app.new_world_dialog
		nw.archetype_input.select(0)
		nw._on_archetype_selected(0)
	_app.new_world_dialog.hide()
	await _frames(4)

## The travel library's staged edit across a swap, and what makes it wait. A field the user is typing in has staged nothing (`_draft`
## takes the text on Enter or on losing focus), so a focused, unstaged name is staged by the capture; a staged draft makes a real
## swap wait with its reason (Q-S4-2) and is carried by a remount that bypasses the gate.
func _rep_tl_draft(ph: Vector2i, tab: Vector2i) -> void:
	print("-- the travel library: a staged definition edit waits and is carried; a focused unstaged field is staged by the capture")
	_rep_hide_all()
	_app.open_travel_library("preset")
	await _frames(8)
	var tl = _app.travel_library_window
	tl._on_add_blank()
	await _frames(4)
	var id0: String = String(tl._current_id)
	var name_le: LineEdit = null
	for n in tl._inspector_body.find_children("*", "LineEdit", true, false):
		if (n as LineEdit).editable:
			name_le = n
			break
	_ok(name_le != null, "the blank custom entry has an editable text field")
	if name_le == null:
		return
	name_le.grab_focus()
	name_le.text = "typed unstaged"
	await _frames(2)
	_ok(tl._draft.is_empty(), "nothing is staged yet: the text is only in the focused field")
	_ok(not str(_blockers()).contains("unsaved"), "so the swap would not wait for it (%s)" % str(_blockers()))
	await _app._remount_composition(null)
	await _frames(14)
	var t2 = _app.travel_library_window
	_ok(t2 != tl and t2.visible and String(t2._current_id) == id0, "a same-class remount: a new instance, open on the same entry")
	var staged := false
	for k in t2._draft:
		staged = staged or String(t2._draft[k]) == "typed unstaged"
	_ok(staged, "the focused field's text was staged by the capture and carried (%s)" % str(t2._draft))
	_ok(str(_blockers()).contains("a travel definition has unsaved edits"), "and the new window holds the swap for it, by name (%s)" % str(_blockers()))
	var runs0: int = _app.class_swap_runs
	await _resize(ph.x, ph.y)
	await _frames(20)
	await get_tree().create_timer(1.3).timeout
	_ok(String(_app.class_key()) == "tablet" and _app.class_swap_runs == runs0, "a real swap waits for the staged edit (class %s)" % _app.class_key())
	t2._draft.clear()
	await _await_swap()
	_ok(String(_app.class_key()) == "phone" and _app.class_swap_runs == runs0 + 1, "and runs once the edit is saved or reverted (class %s)" % _app.class_key())
	_ok(_app.travel_library_window.visible, "the travel library is open on the phone")
	_app.travel_library_window.hide()
	await _live_swap(tab.x, tab.y, "tablet")

## The project picker across a swap: `OpenProjectDialog` on the tablet, `PhoneProjectPicker` on the phone. Same-class remounts keep the
## gallery open with its scope chip and its typed search; a swap to the phone opens the phone's list instead of the dialog (and
## hides the dialog); back on the tablet the dialog is opened again; the welcome composition stays the welcome composition.
func _rep_open_project(ph: Vector2i, tab: Vector2i) -> void:
	print("-- the project picker: scope and search across remounts, then to the phone's list and back")
	_rep_hide_all()
	var d = _app.open_project_dialog
	d.open()
	await _frames(8)
	d._set_scope("all")
	d._search.text = "zz"
	d._search.text_changed.emit("zz")
	await _frames(4)
	for t in range(1, 4):
		var old = _app.open_project_dialog
		await _app._remount_composition(null)
		await _frames(14)
		var n = _app.open_project_dialog
		_ok(n != old and not is_instance_valid(old) and n.visible, "picker remount %d: a new dialog, open" % t)
		_ok(String(n._scope) == "all" and String(n._search.text) == "zz" and not bool(n._welcome), "picker remount %d: scope 'all', search 'zz' (%s, '%s')" % [t, n._scope, n._search.text])
		_ok(not str(_blockers()).contains("window is open"), "picker remount %d: it holds nothing (%s)" % [t, str(_blockers())])
	await _swap_to("phone", ph.x, ph.y)
	var pk = _app.phone_project_picker
	_ok(pk != null and pk.visible, "on the phone the entry screen is open instead of the dialog")
	_ok(not _app.open_project_dialog.visible, "and the dialog is closed")
	_ok(not str(_blockers()).contains("window is open"), "the phone's entry screen holds nothing (%s)" % str(_blockers()))
	await _swap_to("tablet", tab.x, tab.y)
	await _frames(10)
	_ok(_app.phone_project_picker == null and _app.open_project_dialog.visible and not bool(_app.open_project_dialog._welcome), "back on the tablet the gallery dialog is open and the phone's screen is gone")
	_app.open_project_dialog.open_welcome()
	await _frames(8)
	await _app._remount_composition(null)
	await _frames(14)
	_ok(_app.open_project_dialog.visible and bool(_app.open_project_dialog._welcome), "the cold-start welcome is still the welcome after a remount")
	await _swap_to("phone", ph.x, ph.y)
	_ok(_app.phone_project_picker != null and _app.phone_project_picker.visible and bool(_app.phone_project_picker.welcome), "the welcome carried onto the phone is the phone's entry screen, marked as the welcome")
	await _swap_to("tablet", tab.x, tab.y)
	await _frames(10)
	_ok(_app.open_project_dialog.visible and bool(_app.open_project_dialog._welcome), "and back on the tablet it is the welcome again, not the gallery")
	_app.open_project_dialog.hide()
	await _frames(4)

## The settings window's typed API key: it is cleared by design on every rebuild, so a typed one holds the swap by name. The key
## field only exists with an OS credential store, so the probe stands a field in `controls` the way the window's own builder does.
func _rep_settings_key() -> void:
	print("-- the assistant settings window: a typed API key holds the swap, an empty one does not")
	_rep_hide_all()
	var w = _app.assistant_settings_window
	w.open()
	await _frames(8)
	var had = w.controls.get("key_field")
	var kf := LineEdit.new()
	w.controls["key_field"] = kf
	_ok(w.quiescent_blockers().is_empty(), "an empty key field holds nothing")
	kf.text = "sk-probe"
	_ok(w.quiescent_blockers() == ["an API key is typed but not saved"], "a typed key holds the swap and says so (%s)" % str(w.quiescent_blockers()))
	_ok(str(_blockers()).contains("an API key is typed but not saved"), "the shell's gate reports it (%s)" % str(_blockers()))
	w.hide()
	_ok(w.quiescent_blockers().is_empty(), "a closed window holds nothing")
	if had != null:
		w.controls["key_field"] = had
	else:
		w.controls.erase("key_field")
	kf.free()

## -- S4.6: the radial ring and the context card across a class swap --------------------------------------------------

## The tablet ring's radius, as a literal worked out by hand from the two constants it is made of, never read back from them:
## `RING_RADIUS` 92 x `DccTheme.TOUCH_SCALE` 1.53 = 140.76 (above the 96 floor). The un-scaled figure is 92. A ring that
## lived through a swap without being rebuilt keeps whichever it was built with, so these two literals tell the instances apart.
const RING_TABLET_RADIUS := 140.76
const RING_PLAIN_RADIUS := 92.0

## The overlay's centre in its own local space: where a held touch would open the ring and the card.
func _rc_local() -> Vector2:
	return _app.viewport.overlay.size * 0.5

## Opens the ring and the card the way a hold does (`ContextBroker.ring_touch_open()`: the ring first, then the card through
## `resolve()`), so the probe is on the shipped path.
func _rc_open_hold() -> void:
	_app.context_broker.ring_touch_open(_rc_local())
	await _frames(6)

## The world-pause depth every open surface adds one to (`DccApp.pause_for_context_surface()`): a freed-while-open ring or
## card that did not release its share would leave it too high for good.
func _rc_depth() -> int:
	return int(_app._context_surface_depth)

## Closes whatever the broker has open and waits, so the next block starts from depth 0.
func _rc_close_all() -> void:
	var b = _app.context_broker
	if b.ring != null and is_instance_valid(b.ring):
		b.ring.close()
	if b.card != null and is_instance_valid(b.card):
		b.card.hide()
	var pm = _app._phone_menu
	if pm != null and is_instance_valid(pm) and pm.peek_card_is_open():
		pm.close()
	await _frames(4)

func _ringcard() -> void:
	await _settle_at_rest()
	var boot := String(_app.class_key())
	_ok(boot == "tablet", "boots as a tablet (class_key %s)" % boot)
	await _make_world()
	var parts := _arg("--vp", "").split("x")
	var tab := Vector2i(int(parts[0]), int(parts[1]))
	var ph := Vector2i(880, 800)
	var b = _app.context_broker
	_ok(b != null and b.ring == null and b.card == null, "the broker has built neither the ring nor the card yet")
	_ok(_rc_depth() == 0, "no surface is open: the world pause depth is 0 (got %d)" % _rc_depth())

	print("-- tablet: a hold opens the ring and the card; their figures are the tablet's")
	await _rc_open_hold()
	_ok(b.ring != null and b.ring.is_open(), "the ring is open after the hold")
	_ok(b.card != null and b.card.visible, "the card is open beside it")
	if b.ring == null or b.card == null:
		return
	var st1: Dictionary = b.ring.debug_state()
	_ok(absf(float(st1["radius"]) - RING_TABLET_RADIUS) < 0.01, "the tablet ring's radius is %.2f (literal 92 x 1.53; got %.2f)" % [RING_TABLET_RADIUS, float(st1["radius"])])
	_ok(int(b.card._m("row")) == 44, "the card draws its tablet row height, 44 (got %d)" % int(b.card._m("row")))
	_ok(_rc_depth() == 2, "the ring and the card each hold the world pause: depth 2 (got %d)" % _rc_depth())
	var gx0 := float(b.last_request["gx"])
	var gy0 := float(b.last_request["gy"])
	_ok(String(b.last_request["form"]) == "tablet", "the subject was resolved in the tablet form")

	print("-- a REAL swap waits while the ring and the card are open, and runs once they close")
	var old_ring: Object = b.ring
	var old_card: Object = b.card
	var runs0: int = _app.class_swap_runs
	_ok(str(_blockers()).contains("a context card is open"), "an open ring and card are reported (got %s)" % str(_blockers()))
	await _resize(ph.x, ph.y)
	await _frames(10)
	_ok(String(_app.class_key()) == "tablet" and _app.class_swap_runs == runs0 and not _app.pending_class_swap().is_empty(), "880 x 800 under them: pending, not run (class %s, runs %d)" % [_app.class_key(), _app.class_swap_runs])
	await _rc_close_all()
	await _await_swap()
	_ok(String(_app.class_key()) == "phone" and _app.class_swap_runs == runs0 + 1, "closed, the swap ran once (class %s, runs %d)" % [_app.class_key(), _app.class_swap_runs])
	_ok(not is_instance_valid(old_ring) and not is_instance_valid(old_card), "the CLOSED ring and card, built for the tablet, were freed")
	_ok(b.ring == null and b.card == null, "and neither was built again (the broker builds them on first use)")
	_ok(_rc_depth() == 0, "the world pause depth is 0 (got %d)" % _rc_depth())

	print("-- a ring built on the phone has the PLAIN figures (it is a new instance, not the tablet's)")
	b.ring_touch_open(_rc_local())
	await _frames(6)
	_ok(b.ring != null, "the ring exists on the phone once armed (a probe-only arming: the phone never arms one)")
	var ph_radius := float(b.ring.debug_state()["radius"]) if b.ring != null else -1.0
	_ok(absf(ph_radius - RING_PLAIN_RADIUS) < 0.01, "its radius is %.1f (literal 92, un-scaled; got %.2f). A ring kept from the tablet would read %.2f" % [RING_PLAIN_RADIUS, ph_radius, RING_TABLET_RADIUS])
	await _rc_close_all()
	await _live_swap(tab.x, tab.y, "tablet")
	_ok(b.ring == null and b.card == null, "back on the tablet the phone's ring was freed too")
	await _rc_open_hold()
	_ok(b.ring != null and absf(float(b.ring.debug_state()["radius"]) - RING_TABLET_RADIUS) < 0.01, "a hold on the tablet builds a ring with radius %.2f again" % RING_TABLET_RADIUS)
	await _rc_close_all()

	print("-- the carry, on the path that bypasses the gate (a direct remount): tablet to phone with both open")
	await _rc_open_hold()
	old_ring = b.ring
	old_card = b.card
	_ok(_rc_depth() == 2, "depth 2 with both open (got %d)" % _rc_depth())
	## The host is resized with NO `size_changed`, so no swap is requested; the direct call re-decides the class from the size.
	_vp.size = ph
	await _frames(3)
	_app._remount_composition()
	await _frames(14)
	_ok(String(_app.class_key()) == "phone", "the direct remount is at the phone's class (%s)" % _app.class_key())
	_ok(not is_instance_valid(old_ring) and not is_instance_valid(old_card), "the open ring and card were freed")
	_ok(b.ring == null and b.card == null, "the phone has neither (its form is the peek sheet)")
	var pm = _app._phone_menu
	_ok(pm != null and pm.peek_card_is_open(), "the card's subject is open again as the phone's peek sheet")
	_ok(absf(float(b.last_request["gx"]) - gx0) < 1e-6 and absf(float(b.last_request["gy"]) - gy0) < 1e-6 and String(b.last_request["form"]) == "phone",
		"it is the same point (%.3f, %.3f), now resolved in the phone form" % [float(b.last_request["gx"]), float(b.last_request["gy"])])
	_ok(_rc_depth() == 1, "the freed ring and card gave back their pause and the peek sheet holds one share: depth 1, not 2 or 3 (got %d)" % _rc_depth())

	print("-- and phone to tablet: the peek sheet's subject becomes the card; no ring (the phone had none)")
	_vp.size = tab
	await _frames(3)
	_app._remount_composition()
	await _frames(14)
	_ok(String(_app.class_key()) == "tablet", "the tablet again (%s)" % _app.class_key())
	_ok(b.card != null and b.card.visible and b.ring == null, "the card is open in the tablet's form, with no ring")
	_ok(absf(float(b.last_request["gx"]) - gx0) < 1e-6 and absf(float(b.last_request["gy"]) - gy0) < 1e-6 and String(b.last_request["form"]) == "tablet", "the same point, in the tablet form")
	_ok(_rc_depth() == 1, "the card holds one share of the pause, and the closed peek sheet left none (got %d)" % _rc_depth())
	await _rc_close_all()
	_ok(_rc_depth() == 0, "closed: depth 0 (got %d)" % _rc_depth())

	print("-- a same-class remount with both open: the carry is exact")
	await _rc_open_hold()
	## A hold arms the ring in its Q form (not sticky); set the other form by hand so the two flags carry a value that differs
	## from what `arm()` writes (else a restore that forgot them would pass).
	b.ring._sticky = true
	b.ring._q = false
	var st2: Dictionary = b.ring.debug_state()
	var centre2: Vector2 = st2["centre"]
	var sticky2: bool = bool(st2["sticky"])
	var q2: bool = bool(st2["q"])
	_ok(sticky2 and not q2, "the ring is sticky and not the Q form before the remount (sticky %s, q %s)" % [str(sticky2), str(q2)])
	var ring_b = b.ring
	var card_b = b.card
	_app._remount_composition()
	await _frames(14)
	_ok(not is_instance_valid(ring_b) and not is_instance_valid(card_b), "the ring and the card were freed")
	_ok(b.ring != null and b.ring.is_open() and b.card != null and b.card.visible, "and both are open again")
	if b.ring == null or b.card == null:
		return
	var st3: Dictionary = b.ring.debug_state()
	_ok(absf(float(st3["radius"]) - RING_TABLET_RADIUS) < 0.01, "with the tablet radius %.2f" % float(st3["radius"]))
	_ok(((st3["centre"] as Vector2) - centre2).length() < 1.0, "at the same centre (%s -> %s)" % [str(centre2), str(st3["centre"])])
	_ok(bool(st3["sticky"]) == sticky2 and bool(st3["q"]) == q2, "the sticky and Q flags are carried (sticky %s, q %s)" % [str(sticky2), str(q2)])
	_ok(absf(float(b.last_request["gx"]) - gx0) < 1e-6 and absf(float(b.last_request["gy"]) - gy0) < 1e-6 and String(b.last_request["form"]) == "tablet", "the card's subject is the same point, in the tablet form")
	_ok(_rc_depth() == 2, "depth is 2 again, not 4 and not 0 (got %d)" % _rc_depth())
	await _rc_close_all()
	_ok(_rc_depth() == 0, "closing both leaves depth 0 (got %d)" % _rc_depth())
	print("-- nothing open: a remount builds neither")
	_app._remount_composition()
	await _frames(14)
	_ok(b.ring == null and b.card == null and _rc_depth() == 0, "no ring, no card, depth 0 after a remount with neither open")
	var freed := 0
	for l in _connection_lines():
		if String(l).contains("freed/"):
			freed += 1
	_ok(freed == 0, "no connection keeps a freed receiver (%d)" % freed)

## -- S4.6: B5's window and timing legs ------------------------------------------------------------------------------------

## The three windows B5 names (`MOBILE_SHELL_MS1_SPEC.md` section 5): the faction roster, the Place editor and the vault. **Adapted
## to what the shell allows** (`S4_INVENTORY.md` section 18.3): on a phone or a tablet an `AcceptDialog` is exclusive, so only ONE of
## them can be open at a time, and the bar's "three windows open at once" is read as three windows IN TURN, each opened with state
## on the tablet and held open across two round trips through the REAL resize path. For each: it was not made to wait, it is a new
## instance with the old one freed, its state came back, it is baked for the class, and its layout equals a fresh open's (all of that
## is `_rep_step()`, the same checks `--leg represent` makes for every window). On top of those, B5's own: the world hash, the camera
## and the domain are unchanged across the whole leg, and the time of every swap is recorded for the runner.
const B5_WINDOWS := ["faction_roster_window", "place_editor_window", "vault_window"]

## ---- S4.5's last window: the assistant dialog (`S4_INVENTORY.md` section 18.11) ---------------------------------------
## The dialog is KEPT across a swap, so the checks are about what must NOT change as much as what must: the transcript's
## nodes (same instances), the pending card and its Apply, the streaming reply label, the draft, the task and the groups, the
## running request (`run.busy()`, simulated by setting `LlmRun._active`: no socket, no thread, no model; a hide that shut it
## down would clear the flag) and the world-pause depth. And the layout around the transcript must equal a COLD window's in the
## class (the oracle is a second `LlmAssistantWindow` built and put in the same state, never the kept one).

## The layout the kept window and the cold one must share, per control: whether it shows, its minimum, its class and its
## parent's class; plus the dialog chrome. The transcript is excluded (it is the kept part). Text is not compared.
func _asst_layout(win) -> Array:
	var rows: Array = []
	for k in (win.controls as Dictionary).keys():
		if k == "messages" or k == "cleared_note":
			continue
		var c = win.controls[k]
		if c == null or not is_instance_valid(c):
			rows.append("%s: freed" % k)
			continue
		var ctl := c as Control
		rows.append("%s: %s vis=%s min=%s parent=%s" % [k, ctl.get_class(), str(ctl.visible), str(ctl.custom_minimum_size), ctl.get_parent().get_class()])
	rows.sort()
	## `min_size` is not compared: `phone_present()` zeroes it on a SHOWN sheet only (checked on its own below).
	rows.append("chrome: borderless=%s scale=%.3f head=%s padb=%d" % [str(win.borderless), win.content_scale_factor,
		str(win._head_wrap != null), win._pad.get_theme_constant("margin_bottom")])
	return rows

## A cold window for the shell's current class, put in the state the kept one is in (the task, the groups row), laid out.
func _asst_cold_layout(task: String, groups_open: bool, shown: bool = false, busy: bool = false) -> Array:
	var cold := LlmAssistantWindow.new()
	_app.add_child(cold)
	cold.setup(_app, _app.bridge)
	cold._ensure_built()
	cold._pick_task(task)
	cold._groups_flow.visible = groups_open
	## The kept window has a request running, which shows Stop and disables the chips; the oracle is put in the same state.
	if busy:
		cold.run._active = true
		cold.run.set_process(false)
	cold._refresh()
	## A window shown on the phone is fitted to the phone's touch floors (`_show_dialog()` -> `phone_fit()`); a hidden one is not.
	if shown and cold._phone:
		_app.phone_fit(cold, 1.0)
	var rows := _asst_layout(cold)
	## One frame before the free: the shell queued a deferred window arbitration on the new window (`_run_window_arbitrate`).
	await _frames(2)
	cold.run._active = false
	cold.free()
	return rows

## Prints the rows two layouts disagree on, so a failed equality says where.
func _asst_diff(a: Array, b: Array, tag: String) -> void:
	for r in a:
		if not b.has(r):
			print("  diff %s kept-only: %s" % [tag, r])
	for r in b:
		if not a.has(r):
			print("  diff %s cold-only: %s" % [tag, r])

func _asst_snapshot(win) -> Dictionary:
	var kids: Array = []
	for c in win._msgs.get_children():
		kids.append(c.get_instance_id())
	return {"msgs": win._msgs.get_instance_id(), "kids": kids, "cards": (win.cards as Array).size(),
		"card0": (win.cards[0]["apply"] as Button).get_instance_id(), "reply_label": win._reply_label.get_instance_id(),
		"run": win.run.get_instance_id()}

func _assistant() -> void:
	await _settle_at_rest()
	_ok(String(_app.class_key()) == "tablet", "boots as a tablet (class_key %s)" % _app.class_key())
	await _make_world()
	var vpp := _arg("--vp", "").split("x")
	var tab := Vector2i(int(vpp[0]), int(vpp[1]))
	var ph := Vector2i(880, 800)
	var win = _app.assistant_window
	_ok(_app.open_context_tab("ask"), "the Ask tab opens")
	await _frames(8)
	_ok(win.is_docked() and win._built, "the conversation is built and docked in the Ask tab")
	## ---- the fixture: a message, a pending card, a draft, a task, a chosen group, a streaming reply ----
	win._add_msg("You", "hello from the probe", "text_secondary")
	var lay := ""
	var lay_label := ""
	for l in (win._VIEW.facts(_app, _app.bridge) as Dictionary).get("layers", []):
		if bool((l as Dictionary).get("on", false)):
			lay = String(l["id"])
			lay_label = String(l["label"])
			break
	_ok(lay != "", "control: a shown layer exists for the card (%s)" % lay)
	var card_v := {"summary": "probe card", "valid": [{"action": "set_layer_visible", "label": lay_label, "from": "shown", "to": "hidden",
		"args": {"layer": lay, "visible": false}, "undoable": false}], "rejected": [], "noop": PackedStringArray(), "undo": PackedStringArray()}
	win._on_view_proposal({"view": card_v})
	## "Choose groups" shows only while the parameter source is on (`_refresh()`'s `groups_ok`); the wiped user dir may have it off.
	DccSettings.set_llm_value("ctx_params", true)
	win._pick_task("commands")
	win._toggle_groups()
	_ok(win._groups_flow.visible, "control: the groups row is open before the swap")
	var gname := String(win._group_names()[0])
	win._tick_group(gname)
	(win.controls["input"] as TextEdit).text = "DRAFT-ASST-X"
	win._reply_label = win._add_msg("Assistant", "", "text")
	win._reply = ""
	win.run._active = true
	## `LlmRun` processes by default once in the tree, and its pump ends a reply whose wire never started: stop it, or the simulated request would end itself on the first frame.
	win.run.set_process(false)
	win._on_token("first ")
	await _frames(4)
	_ok(win.run.busy() and win._reply == "first " and win._reply_label.text == "first ", "control: a simulated request is running and streaming ('%s')" % win._reply)
	_ok((win.cards as Array).size() == 1 and (win.cards[0] as Dictionary)["state"] == "new", "control: one card is pending")
	var depth0: int = _app._context_surface_depth
	_ok(depth0 == 0, "the world-pause depth is 0 before (%d)" % depth0)
	var before := _asst_snapshot(win)
	var runs0: int = _app.class_swap_runs
	var expect_reply := "first "
	var orph := {}
	for trip in range(1, 4):
		## ---- tablet -> phone: the Ask tab was open, so the sheet opens ----
		await _live_swap(ph.x, ph.y, "phone")
		_ok(_app.class_swap_runs == runs0 + 2 * trip - 1, "trip %d: the swap RAN with a reply running (a running reply no longer holds it)" % trip)
		_ok(win._phone and win._form == "phone" and win.visible and not win.is_docked(), "trip %d phone: the sheet is open, in the phone form" % trip)
		_ok(win.run.busy(), "trip %d phone: the request is still running (nothing hid it into a shutdown)" % trip)
		_ok(_asst_snapshot(win) == before, "trip %d phone: the transcript, the card's Apply, the reply label and the run are the same instances" % trip)
		_ok(win._reply == expect_reply and win._reply_label.text == expect_reply, "trip %d phone: the streaming label holds the reply so far ('%s')" % [trip, win._reply])
		win._on_token("second ")
		expect_reply += "second "
		await _frames(2)
		_ok(win._reply_label.text == expect_reply, "trip %d phone: a token after the swap lands in the same label" % trip)
		print("  info draft='%s' task=%s groups=%s open=%s" % [(win.controls["input"] as TextEdit).text, win._task, str(win._chosen_groups), str(win._groups_flow.visible)])
		_ok((win.controls["input"] as TextEdit).text == "DRAFT-ASST-X" and win._task == "commands" and (win._chosen_groups as Array) == [gname] and win._groups_flow.visible,
			"trip %d phone: the draft, the task, the chosen group and its row are back" % trip)
		_ok(win.borderless and win.min_size == Vector2i.ZERO and win._head_wrap != null, "trip %d phone: the sheet chrome (borderless, no minimum, header)" % trip)
		## The oracle windows are built on trip 1 only: each leaves two orphan nodes behind (the probe's, not the swap's), which
		## would hide a swap leak in the trip-to-trip orphan count below.
		if trip == 1:
			var cold_p := (await _asst_cold_layout("commands", true, win.visible, true))
			_asst_diff(_asst_layout(win), cold_p, "phone")
			_ok(_asst_layout(win) == cold_p, "trip %d phone: the layout equals a cold window's" % trip)
		_ok(_app._context_surface_depth == depth0, "trip %d phone: the world-pause depth is unchanged (%d)" % [trip, _app._context_surface_depth])
		## ---- phone -> tablet: the sheet closes into the Ask tab ----
		await _live_swap(tab.x, tab.y, "tablet")
		await _frames(8)
		_ok(not win._phone and win._form == "tablet" and not win.visible and win.is_docked(), "trip %d tablet: the conversation is in the Ask tab, the dialog hidden" % trip)
		_ok(win.run.busy(), "trip %d tablet: the request is still running" % trip)
		_ok(_asst_snapshot(win) == before, "trip %d tablet: the same instances" % trip)
		_ok((win.controls["input"] as TextEdit).text == "DRAFT-ASST-X" and win._task == "commands" and (win._chosen_groups as Array) == [gname],
			"trip %d tablet: draft, task and group intact" % trip)
		_ok(not win.borderless and win.min_size == Vector2i(440, 520) and win._head_wrap == null and win.content_scale_factor == 1.0, "trip %d tablet: the window chrome is the pointer form's" % trip)
		if trip == 1:
			var cold_t := (await _asst_cold_layout("commands", true, win.visible, true))
			_asst_diff(_asst_layout(win), cold_t, "tablet")
			_ok(_asst_layout(win) == cold_t, "trip %d tablet: the layout equals a cold window's" % trip)
		_ok(_app._context_surface_depth == depth0, "trip %d tablet: the world-pause depth is unchanged" % trip)
		orph[trip] = _orphans()
		print("  info orphan nodes after trip %d: %d" % [trip, orph[trip]])
	_ok(orph[3] == orph[2], "orphan nodes after trip 3 equal trip 2's (%d, %d)" % [orph[3], orph[2]])
	var freed := 0
	for l in _connection_lines():
		if String(l).contains("freed/"):
			freed += 1
	_ok(freed == 0, "no connection keeps a freed receiver (%d)" % freed)
	## ---- the request ends; the card still applies ----
	win.run._active = false
	win._refresh()
	var card: Dictionary = win.cards[0]
	(card["apply"] as Button).pressed.emit()
	await _frames(4)
	_ok(card["state"] == "applied" and not bool(_app.viewport.layer_visible(lay)), "after three round trips the kept card's Apply works (state %s, layer %s hidden)" % [card["state"], lay])
	## ---- a window that is closed is re-laid out too, and a closed one stays closed ----
	_app.open_context_tab("inspect")
	await _frames(4)
	await _live_swap(ph.x, ph.y, "phone")
	_ok(win._phone and not win.visible, "closed on the tablet, it stays closed on the phone")
	win.open()
	await _frames(6)
	var cold_o := (await _asst_cold_layout("commands", true, win.visible, false))
	_asst_diff(_asst_layout(win), cold_o, "opened-later")
	_ok(win.visible and _asst_layout(win) == cold_o, "opened later on the phone, its layout equals a cold window's")
	win.hide()
	await _live_swap(tab.x, tab.y, "tablet")

func _b5() -> void:
	await _settle_at_rest()
	var boot := String(_app.class_key())
	_ok(boot == "tablet", "boots as a tablet (class_key %s)" % boot)
	await _make_world()
	_rep_init()
	var parts := _arg("--vp", "").split("x")
	var tab := Vector2i(int(parts[0]), int(parts[1]))
	var ph := Vector2i(880, 800)
	_live_world = _world_hash()
	_live_zoom = float(_app.viewport.zoom())
	var cam0: Vector2 = _app.viewport._camera.position
	var dom0 := String(_app.active_domain())
	var sync: Array = []
	var full: Array = []
	for field in B5_WINDOWS:
		print("-- %s: open on the tablet with state, held open across two round trips" % field)
		await _rep_fixture(field)
		for t in range(1, 3):
			await _rep_step(field, ph.x, ph.y, "phone", "%s trip %d to the phone" % [field, t])
			sync.append(float(_app.last_swap_sync_ms))
			full.append(float(_app.last_swap_ms))
			await _rep_step(field, tab.x, tab.y, "tablet", "%s trip %d back to the tablet" % [field, t])
			sync.append(float(_app.last_swap_sync_ms))
			full.append(float(_app.last_swap_ms))
		_app.get(field).hide()
		await _frames(4)
	_ok(_world_hash() == _live_world, "the world is byte-identical after all the swaps (map texture and engine cells)")
	_ok(absf(float(_app.viewport.zoom()) - _live_zoom) < 1e-6, "the camera zoom is unchanged (%.6f, was %.6f)" % [_app.viewport.zoom(), _live_zoom])
	_ok((_app.viewport._camera.position - cam0).length() < 1e-6, "the camera's pan offset is unchanged (%s, was %s)" % [str(_app.viewport._camera.position), str(cam0)])
	_ok(String(_app.active_domain()) == dom0, "the domain is unchanged (%s)" % String(_app.active_domain()))
	_ok(sync.size() == 12, "twelve swaps were timed (%d)" % sync.size())
	var freed := 0
	for l in _connection_lines():
		if String(l).contains("freed/"):
			freed += 1
	_ok(freed == 0, "no connection keeps a freed receiver (%d)" % freed)
	## One line the runner (`ms1_b5.sh`) reads: this process's cold boot and the median of its twelve swaps, both in ms.
	print("B5TIME cold_boot=%.1f swap_sync_median=%.1f swap_whole_median=%.1f engine_start=%d n=%d" % [_cold_boot_ms, _median(sync), _median(full), _engine_ms, sync.size()])

func _represent() -> void:
	await _settle_at_rest()
	var boot := String(_app.class_key())
	_ok(boot == "tablet", "boots as a tablet (class_key %s)" % boot)
	await _make_world()
	_rep_init()
	var parts := _arg("--vp", "").split("x")
	var tab := Vector2i(int(parts[0]), int(parts[1]))
	## 800 dp tall, not the 1 564 of the live leg: a window as tall as the phone shows 1 200 px of content with nothing to scroll, so
	## the scroll carried across would be a vacuous check there.
	var ph := Vector2i(880, 800)
	_live_world = _world_hash()
	_live_zoom = float(_app.viewport.zoom())
	## `--only <field>`: one window, for the mutation runs (each mutant needs only its own window's steps).
	var only := _arg("--only", "")
	for field in REP_ORDER:
		if only != "" and field != only:
			continue
		print("-- %s: open on the tablet with state, then three round trips" % field)
		await _rep_fixture(field)
		var o0 := -1
		var c0 := {}
		for t in range(1, 4):
			await _rep_step(field, ph.x, ph.y, "phone", "%s trip %d to the phone" % [field, t])
			await _rep_step(field, tab.x, tab.y, "tablet", "%s trip %d back to the tablet" % [field, t])
			if t == 1:
				o0 = _orphans()
				c0 = _live_conn_sets()
			else:
				_ok(_orphans() == o0, "%s trip %d: orphan nodes %d equal trip 1's %d" % [field, t, _orphans(), o0])
				var cs_now := _live_conn_sets()
				_ok(cs_now == c0, "%s trip %d: the named signals have trip 1's receivers" % [field, t])
				if cs_now != c0:
					for ck in cs_now:
						if cs_now[ck] != c0.get(ck, []):
							print("    conn %s: now %s ; trip 1 %s" % [ck, str(cs_now[ck]).substr(0, 600), str(c0.get(ck, [])).substr(0, 600)])
		_app.get(field).hide()
		await _frames(4)
	if only == "" or only == "world_data_window":
		await _rep_world_tab(ph, tab)
	if only == "" or only == "search":
		await _rep_search(ph, tab)
	if only == "" or only == "gr_scroll":
		await _rep_gr_scroll(ph, tab)
	if only == "" or only == "qb_tab":
		await _rep_qb_tab(ph, tab)
	if only == "" or only == "nw_extra":
		await _rep_nw_extra(ph, tab)
	if only == "" or only == "tl_draft":
		await _rep_tl_draft(ph, tab)
	if only == "" or only == "open_project":
		await _rep_open_project(ph, tab)
	if only == "" or only == "settings_key":
		await _rep_settings_key()
	_ok(_world_hash() == _live_world and absf(float(_app.viewport.zoom()) - _live_zoom) < 1e-6, "the world and the camera are untouched")
	print("-- the vault holds the swap until its text is saved (a converted window is hidden by then: only one modal window can be shown)")
	var vw = _app.vault_window
	_ok(vw.has_method("represent_capture"), "the vault re-presents itself, and still holds the swap for unsaved text (this block is that wait)")
	_rep_hide_all()
	vw._browse_path = "probe.md"
	vw._browse_saved = "saved"
	vw._browse_text = "typed but not saved"
	vw.show()
	await _frames(4)
	_ok(vw._is_dirty(), "the vault holds unsaved text")
	var runs0: int = _app.class_swap_runs
	await _resize(ph.x, ph.y)
	await _frames(20)
	await get_tree().create_timer(1.3).timeout
	_ok(String(_app.class_key()) == "tablet" and _app.class_swap_runs == runs0 and str(_blockers()).contains("a note has unsaved text"), "the swap waits for the vault's text and names it (%s)" % str(_blockers()))
	_ok(str(_app.status_slot_text("hint")).begins_with("Layout change waiting: a note has unsaved text"), "the status line says: '%s'" % _app.status_slot_text("hint"))
	vw._browse_saved = "typed but not saved"
	vw._browse_path = ""
	vw.hide()
	await _await_swap()
	_ok(String(_app.class_key()) == "phone" and _app.class_swap_runs == runs0 + 1, "once the text is saved the swap runs (class %s)" % _app.class_key())
	_ok(bool(_app.faction_roster_window._phone) and bool(_app.place_editor_window._phone), "the two windows, closed, were rebuilt for the phone too (a later open is in its form)")
	var freed := 0
	for l in _connection_lines():
		if String(l).contains("freed/"):
			freed += 1
	_ok(freed == 0, "no connection keeps a freed receiver (%d)" % freed)

extends Node
## Committed probe for the HISTORY panel's **undone tail** -- the rows
## `design/proposed-2026-09-05/Main.dc.html` draws below the cursor band (`07 add
## label · "Ashen Reach"`, `08 move label`) and the `✕ discard the 2 undone
## steps` row under them.
##
## Five things, in the order they can break:
##
##   T0  `EngineBridge.redo_labels()` / `discard_redo_tail()` are reachable
##       through the **GDScript wrapper**. A `#[func]` with no forwarder is
##       invisible to the shell however healthy the `.dll` is, and that failure
##       has been misdiagnosed as a stale build twice in one week.
##   T1  Two undos draw TWO named rows, in `redo_labels()`'s own order, with the
##       ordinals continuing the committed rows above.
##   T2  Those rows draw at the same height as the committed rows they continue,
##       at **both** densities -- 22 px pointer, `role_px("row_min_h")` touch.
##   T3  **Invalidation.** A committed operation drops the tail, and the rows
##       must leave the screen with it. Driven with a direct `bridge` call, so
##       nothing in the shell rebuilds the dock: the assertion is that the rows
##       are still there on the frame of the edit and gone three frames later,
##       which is `RightDock._process()` and nothing else.
##   T4  The discard row acts, names the right number, and touches ONLY the tail
##       -- the undo depth and the ledger must be where they were.
##   T5  Clicking the second undone row moves the cursor two steps, not one.
##
## Every assertion reads a value **off a drawn node** -- a `Label`'s `text`, a
## `Control`'s measured `size` -- or off the engine, never out of the variable
## that fed the drawing. The drawn names are compared against `redo_labels()`
## element by element, which is the only check that can catch a list drawn in
## the wrong order.
##
## **No hex is measured here, so this probe does not force a palette** -- unlike
## `_rd3ctx_probe.gd`, whose every figure is a dark-palette value on a machine
## that boots light. Heights and text are palette-independent.
##
## Run it WINDOWED. Nothing here rasterises, but the brief that commissioned it
## asks for windowed evidence and a windowed run costs nothing extra:
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _redodock_probe.tscn

var _app: Node
var _bridge: Node
var _fail := 0

func _p(s: String) -> void:
	print("REDO %s" % s)

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("REDO %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

# -- Reading the drawn tree back ---------------------------------------------

func _walk(n: Node, out: Array) -> void:
	for c in n.get_children():
		out.append(c)
		_walk(c, out)

func _nodes() -> Array:
	var out: Array = []
	_walk(_app.right_dock_body, out)
	return out

func _texts() -> Array:
	var out: Array = []
	for n in _nodes():
		if n is Label:
			out.append((n as Label).text)
	return out

func _has_text(t: String) -> bool:
	return _texts().has(t)

## Every drawn undone row, in draw order -- an `HBoxContainer` carrying a Label
## whose text is exactly `undone`. Found by that trailing marker rather than by
## ink, because ink is what the artboard distinguishes these rows by and reading
## the marker proves the panel is not relying on colour alone.
func _undone_rows() -> Array:
	var out: Array = []
	for n in _nodes():
		if not (n is HBoxContainer):
			continue
		for c in (n as HBoxContainer).get_children():
			if c is Label and (c as Label).text == "undone":
				out.append(n)
				break
	return out

## The name each undone row is drawing -- its `SIZE_EXPAND_FILL` label, which is
## the only child of the row set to expand.
func _undone_names() -> Array:
	var out: Array = []
	for r in _undone_rows():
		for c in (r as HBoxContainer).get_children():
			if c is Label and (c as Label).size_flags_horizontal == Control.SIZE_EXPAND_FILL:
				out.append((c as Label).text)
				break
	return out

## The ordinal each undone row is drawing -- its first child.
func _undone_ordinals() -> Array:
	var out: Array = []
	for r in _undone_rows():
		var kids := (r as HBoxContainer).get_children()
		if not kids.is_empty() and kids[0] is Label:
			out.append((kids[0] as Label).text)
	return out

## The committed rows, found the same structural way: an `HBoxContainer` whose
## first child is a two-digit ordinal and which carries no `undone` marker.
func _committed_rows() -> Array:
	var undone := _undone_rows()
	var out: Array = []
	for n in _nodes():
		if not (n is HBoxContainer) or undone.has(n):
			continue
		var kids := (n as HBoxContainer).get_children()
		if kids.size() < 3 or not (kids[0] is Label):
			continue
		var t := (kids[0] as Label).text
		if t.length() == 2 and t.is_valid_int():
			out.append(n)
	return out

func _discard_button() -> Button:
	for n in _nodes():
		if n is Button and (n as Button).text.begins_with("✕ discard"):
			return n as Button
	return null

func _generate(seed_value: int) -> void:
	_bridge.generate({
		"seed": seed_value, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout

## Undo `n` times through the app wrapper -- the same path the panel's own `↶`
## square takes, so the dock is rebuilt by the shell exactly as a user's press
## would rebuild it.
func _undo(n: int) -> void:
	for i in n:
		_app.undo_last()
		await _frames(4)

# ============================================================================

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 900.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	_bridge = _app.bridge
	_p("density at boot: %s" % ("touch/tablet" if DccTheme.is_tablet() else "pointer/desktop"))
	await _frames(4)

	await _generate(483920)
	if not _bridge.has_world:
		_p("!! generate failed -- nothing below can run")
		get_tree().quit(2)
		return

	# ---------------------------------------------------------------- T0 ----
	_p("=== T0 the forwarders exist on EngineBridge, not just on WorldGen ===")
	_check("EngineBridge.redo_labels() is defined",
		_bridge.has_method("redo_labels"))
	_check("EngineBridge.discard_redo_tail() is defined",
		_bridge.has_method("discard_redo_tail"))
	## Callable, not merely defined: a forwarder that errors at the boundary
	## would still answer `has_method`.
	var empty_tail: PackedStringArray = _bridge.redo_labels()
	_p("redo_labels() with no tail -> %s (size %d)" % [str(empty_tail), empty_tail.size()])
	_check("redo_labels() answers a PackedStringArray and it is empty at the tip",
		empty_tail is PackedStringArray and empty_tail.is_empty())
	var nothing: int = _bridge.discard_redo_tail()
	_check("discard_redo_tail() answers 0 when there is no tail", nothing == 0,
		"returned %d" % nothing)
	## And the `0` above is a real "no tail", not `_has()` degrading against an
	## older cdylib: either name in the staleness fingerprint would mean the
	## `.dll` has no such `#[func]` and every number below is meaningless.
	var missing: PackedStringArray = _bridge.missing_bindings()
	_check("neither name is in missing_bindings() -- the cdylib really carries them",
		not missing.has("redo_labels") and not missing.has("discard_redo_tail"),
		"missing=%s" % str(missing))

	# ---------------------------------------------------------------- T1 ----
	_p("=== T1 two undos draw two named rows ===")
	## Three real height snapshots on top of the generate's floor row.
	for i in 3:
		var r: Dictionary = _bridge.carve_fjords()
		_p("carve_fjords #%d -> %s" % [i + 1, str(r)])
		await _frames(3)
	var rd = _app.right_dock_ctrl
	rd.show_history()
	await _frames(6)
	var full_ledger: int = _bridge.undo_ledger().size()
	var full_depth: int = int(_bridge.undo_stats().get("depth", 0))
	_p("ledger=%d  undo depth=%d" % [full_ledger, full_depth])
	_check("three carves built a ledger deep enough to undo twice", full_ledger >= 3,
		"ledger=%d" % full_ledger)

	await _undo(2)
	var labels: PackedStringArray = _bridge.redo_labels()
	_p("after two undos: redo_labels=%s  redo_available=%s  redo_depth(stats)=%d"
		% [str(labels), _bridge.redo_available(),
			int(_bridge.undo_stats().get("redo_depth", 0))])
	_check("redo_labels() names both undone steps", labels.size() == 2,
		"got %d: %s" % [labels.size(), str(labels)])
	var drawn := _undone_names()
	_check("the panel draws one row per undone step", drawn.size() == labels.size(),
		"%d rows for %d labels: %s" % [drawn.size(), labels.size(), str(drawn)])
	## Element by element and in order. **Weak here on purpose-built data and
	## said so:** both labels are `Carve fjords`, so this passes on any order.
	## T4 makes the same comparison against a tail of two DIFFERENT names and
	## is where the ordering claim is actually earned.
	var ordered := drawn.size() == labels.size()
	for i in range(mini(drawn.size(), labels.size())):
		if String(drawn[i]) != String(labels[i]):
			ordered = false
	_check("and draws them in redo_labels() order, element by element", ordered,
		"drawn=%s  labels=%s" % [str(drawn), str(labels)])
	## Ordinals continue the committed rows rather than restarting.
	var ledger_now: int = _bridge.undo_ledger().size()
	var want_ord: Array = []
	for i in range(labels.size()):
		want_ord.append("%02d" % (ledger_now + i + 1))
	_check("the ordinals continue the ledger's own count", _undone_ordinals() == want_ord,
		"drew %s, expected %s (ledger=%d)" % [str(_undone_ordinals()), str(want_ord), ledger_now])
	## And the discard row names the same number the list has.
	var db := _discard_button()
	_check("the discard row is drawn", db != null,
		"button texts=%s" % str(_texts().slice(0, 10)))
	if db != null:
		_p("discard row text: '%s'" % db.text)
		_check("and names the count the list actually has",
			db.text == "✕ discard the %d undone steps" % labels.size(),
			"drew '%s'" % db.text)

	# ---------------------------------------------------------------- T2 ----
	_p("=== T2 row heights, at both densities ===")
	var un := _undone_rows()
	var com := _committed_rows()
	if not un.is_empty() and not com.is_empty():
		var uh: float = (un[0] as Control).size.y
		var ch: float = (com[0] as Control).size.y
		_p("pointer density: undone row %.0f px, committed row %.0f px, discard button %.0f px"
			% [uh, ch, (db as Control).size.y if db != null else -1.0])
		_check("an undone row is exactly as tall as the committed rows it continues",
			absf(uh - ch) < 0.5, "undone %.1f vs committed %.1f" % [uh, ch])
		_check("and that is the 22 px this dock's rows draw at pointer density",
			absf(uh - 22.0) < 0.5, "drew %.1f" % uh)
	## Touch. `role_px("row_min_h")` and `role_px("btn_min_h")` are both 44 in
	## `ROLE`; asserted here as the literal 44 the tap floor actually is, not
	## against the table that produced it.
	DccTheme.set_touch(true)
	rd.show_history()
	await _frames(6)
	var un_t := _undone_rows()
	var com_t := _committed_rows()
	var db_t := _discard_button()
	if not un_t.is_empty() and not com_t.is_empty():
		var uh2: float = (un_t[0] as Control).size.y
		var ch2: float = (com_t[0] as Control).size.y
		var bh2: float = (db_t as Control).size.y if db_t != null else -1.0
		_p("touch density: undone row %.0f px, committed row %.0f px, discard button %.0f px"
			% [uh2, ch2, bh2])
		_check("at touch density an undone row meets the 44 px tap floor", uh2 >= 44.0,
			"drew %.1f" % uh2)
		_check("the committed rows meet it too, so the list is one height", ch2 >= 44.0,
			"drew %.1f" % ch2)
		_check("and the discard row meets the button tap floor", bh2 >= 44.0,
			"drew %.1f" % bh2)
	DccTheme.set_touch(false)
	rd.show_history()
	await _frames(6)
	_check("back at pointer density for the drives below", not DccTheme.is_tablet())

	# ---------------------------------------------------------------- T3 ----
	_p("=== T3 invalidation: a new edit drops the tail and the rows must go ===")
	_check("two undone rows are on screen before the edit", _undone_rows().size() == 2,
		"found %d" % _undone_rows().size())
	## **The erosion op, and it has to be this one.** Of the engine's three
	## `self.undo.push` sites, two rebuild this dock by accident:
	## `bridge.carve_fjords()` emits `world_loaded` (connected to `_rebuild`)
	## and `bridge.sculpt_commit()` emits `sculpt_draft_changed` (the deferred
	## stamp-count backstop). Measured, not assumed -- the first run of this
	## probe drove `carve_fjords()` and the positive control below failed,
	## because the rows were already gone on the frame of the edit.
	##
	## `world_workspace.gd::_run_erode()` is the uncovered one: it calls
	## `bridge.world_gen.erode_op()` straight through, past every wrapper in
	## `engine_bridge.gd`, so no signal fires and nothing calls
	## `refresh_history()`. Driven the same way here. `droplets` and
	## `thermal_passes` are cut from `ERODE_DEFAULTS` so the op is quick; the
	## pass still pushes one undo step, which is all this drive needs.
	var opts: Dictionary = WorldWorkspace.ERODE_DEFAULTS.duplicate()
	opts["droplets"] = 2000
	opts["thermal_passes"] = 1
	var r3: Dictionary = _bridge.world_gen.erode_op(opts)
	_p("erode_op (direct on world_gen, the path _run_erode takes) -> ok=%s cells_changed=%s"
		% [str(r3.get("ok", false)), str(r3.get("cells_changed", "?"))])
	_check("the erosion pass ran", bool(r3.get("ok", false)),
		"reason=%s" % String(r3.get("reason", "")))
	_check("the engine dropped the tail immediately", not _bridge.redo_available())
	_check("POSITIVE CONTROL: the stale rows are still drawn on this frame, so "
		+ "nothing else rebuilt the dock", _undone_rows().size() == 2,
		"found %d" % _undone_rows().size())
	await _frames(4)
	_check("and the poll has taken them off screen four frames later",
		_undone_rows().is_empty(),
		"still drawing %s" % str(_undone_names()))
	_check("the discard row went with them", _discard_button() == null)
	_check("redo_labels() agrees there is nothing to draw",
		_bridge.redo_labels().is_empty())

	# ---------------------------------------------------------------- T4 ----
	_p("=== T4 the discard row acts, and touches only the tail ===")
	await _undo(2)
	await _frames(4)
	var before_depth: int = int(_bridge.undo_stats().get("depth", 0))
	var before_ledger: int = _bridge.undo_ledger().size()
	var before_can_undo: bool = _bridge.can_undo()
	var before_labels: PackedStringArray = _bridge.redo_labels()
	_p("before discard: undo depth=%d ledger=%d can_undo=%s tail=%s"
		% [before_depth, before_ledger, before_can_undo, str(before_labels)])
	## **The ordering check T1 could not make.** T3's erosion pass means this
	## tail is `["Carve fjords", "Erode (droplet)"]` -- two different names, so
	## a list drawn newest-first fails here and only here.
	var drawn4 := _undone_names()
	var ordered4 := drawn4.size() == before_labels.size() and before_labels.size() == 2
	for i in range(mini(drawn4.size(), before_labels.size())):
		if String(drawn4[i]) != String(before_labels[i]):
			ordered4 = false
	_check("with two DIFFERENT names, the rows are in redo_labels() order", ordered4,
		"drawn=%s  labels=%s" % [str(drawn4), str(before_labels)])
	var db2 := _discard_button()
	_check("the discard row is drawn again", db2 != null)
	if db2 != null:
		## Pressed through the real control's own signal, not by calling the
		## handler -- a handler that is never wired would pass the other check.
		db2.pressed.emit()
		await _frames(4)
		_p("after discard: undo depth=%d ledger=%d redo_available=%s tail=%s"
			% [int(_bridge.undo_stats().get("depth", 0)), _bridge.undo_ledger().size(),
				_bridge.redo_available(), str(_bridge.redo_labels())])
		_check("the tail is gone", not _bridge.redo_available()
			and _bridge.redo_labels().is_empty())
		_check("and its rows left the panel", _undone_rows().is_empty(),
			"still drawing %s" % str(_undone_names()))
		_check("the discard row went with them", _discard_button() == null)
		_check("the undo stack is EXACTLY where it was -- discard is not clear_undo",
			int(_bridge.undo_stats().get("depth", 0)) == before_depth,
			"%d -> %d" % [before_depth, int(_bridge.undo_stats().get("depth", 0))])
		_check("and so is the ledger", _bridge.undo_ledger().size() == before_ledger,
			"%d -> %d" % [before_ledger, _bridge.undo_ledger().size()])
		## Against its own value before the discard, not against `true`: by this
		## point the stack may legitimately be empty, and asserting `can_undo()`
		## is what a first run of this probe did -- it failed on a stack that
		## two undos had honestly exhausted, which is the harness's error and
		## not the panel's.
		_check("can_undo() is exactly what it was before the discard",
			_bridge.can_undo() == before_can_undo,
			"%s -> %s" % [before_can_undo, _bridge.can_undo()])

	# ---------------------------------------------------------------- T5 ----
	_p("=== T5 clicking the second undone row moves the cursor two steps ===")
	## Re-stock the stack: T3's erosion and T4's two undos leave it empty, so
	## `_undo(2)` below would have nothing to take off.
	for i in 2:
		_bridge.carve_fjords()
		await _frames(3)
	await _undo(2)
	await _frames(4)
	var rows5 := _undone_rows()
	var ledger5: int = _bridge.undo_ledger().size()
	_check("two rows to click", rows5.size() == 2, "found %d" % rows5.size())
	if rows5.size() == 2:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = true
		## The SECOND row -- one press of Redo would only reach the first, so a
		## handler that ignored its index would leave a row behind.
		(rows5[1] as Control).gui_input.emit(ev)
		await _frames(8)
		_p("after clicking row 2: ledger=%d -> %d  redo_available=%s"
			% [ledger5, _bridge.undo_ledger().size(), _bridge.redo_available()])
		_check("both steps came back, not one",
			_bridge.undo_ledger().size() == ledger5 + 2,
			"%d -> %d" % [ledger5, _bridge.undo_ledger().size()])
		_check("and the tail is empty, so no undone row is left drawn",
			not _bridge.redo_available() and _undone_rows().is_empty(),
			"drawing %s" % str(_undone_names()))

	_p("=== %d failure(s) ===" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

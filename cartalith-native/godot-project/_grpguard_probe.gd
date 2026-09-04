extends Node
## Lane B / batch 25, part 4 -- verification for the `DccWidgets.group()`
## header change, written to the two hazards the change actually carries.
##
##  1. The GUARD. `MISTAKES.md`: the question is "does a sibling compete for my
##     width", not "which class is my parent". `GridContainer` was the arm a
##     verifier had to add to `action()`, so every distributing container is
##     asserted here with one real child of each -- not reasoned about.
##  2. The RE-BASE. `group()` has 83 call sites, so it is not enough that each
##     header got narrower: the RELATIONSHIPS it participates in have to still
##     hold -- the header beside its own body, and the collapsed-vs-open
##     heights. Both are asserted over a real header in a real dock.
##
## Plus the claim the new doc comment makes about the tree it leaves behind
## ("no call site hands this factory a width-distributing parent today"), which
## is asserted live rather than inferred from a grep of the call sites.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _grpguard_probe.tscn

const SEEDS := [483920, 77021, 4242]

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("GG %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

## A `DccWidgets.group()` header, and NOT merely a button whose label starts
## with the same glyph. This predicate took two corrections, both caught by
## widening the sweep rather than by reading it:
##
##   1. `begins_with(sigil + " ")` alone counted `+ Add faction` -- a chip in
##      an `HBoxContainer` -- as a group header 16 times per seed, which would
##      have shipped a false claim about no call site handing this factory a
##      distributing parent.
##   2. Adding "the text after the sigil is upper-case" fixed that and broke
##      the headers that matter most: `civilization_workspace.gd`'s
##      `_lm_refresh_group()` appends `"   %d of %d armed · %d placed"` in
##      LOWER case, so the widest header in the shell (280 px) was silently
##      excluded from every count.
##
## So the test is STRUCTURAL, off what `group()` actually builds: a flat,
## unfocusable `Button` at the group header's own font size, followed
## immediately by the `MarginContainer` holding its body `VBoxContainer`.
## Nothing else in the shell has that shape, and no amount of runtime text
## rewriting can move it.
func _is_group_header(c: Node) -> bool:
	if not (c is Button):
		return false
	var b := c as Button
	if not b.flat or b.focus_mode != Control.FOCUS_NONE:
		return false
	var t := String(b.text)
	if not (t.begins_with(DccIcons.SYMBOLS["expand"] + " ") or t.begins_with("+ ")):
		return false
	var want := DccTheme.role_px("fs_dock_header") if DccTheme.is_tablet() else DccTheme.FS_HEADER
	if b.get_theme_font_size("font_size") != want:
		return false
	var p := b.get_parent()
	if p == null or b.get_index() + 1 >= p.get_child_count():
		return false
	var pad := p.get_child(b.get_index() + 1)
	if not (pad is MarginContainer) or pad.get_child_count() == 0:
		return false
	return pad.get_child(0) is VBoxContainer

func _headers(root: Node, out: Array) -> void:
	for c in root.get_children():
		if c is Control and not (c as Control).visible:
			continue
		if _is_group_header(c):
			out.append(c)
		_headers(c, out)

func _drag_to(dock: Control, floor_px: int) -> float:
	var was := dock.custom_minimum_size.x
	dock.custom_minimum_size.x = float(floor_px)
	await _frames(4)
	var got := dock.size.x
	dock.custom_minimum_size.x = was
	await _frames(3)
	return got

## The header `group()` just appended to `parent` -- it returns the body, and
## the header is the last `Button` child, which is the same backwards search
## `civilization_workspace.gd::_lm_last_button()` already relies on.
func _last_button(parent: Control) -> Button:
	for i in range(parent.get_child_count() - 1, -1, -1):
		var c := parent.get_child(i)
		if c is Button:
			return c
	return null

func _ready() -> void:
	# == 1. the guard, one real child of every distributing container =========
	var host := VBoxContainer.new()
	add_child(host)
	var cases := [
		["VBoxContainer", VBoxContainer.new(), true],
		["HBoxContainer", HBoxContainer.new(), false],
		["HFlowContainer", HFlowContainer.new(), false],
		["GridContainer", GridContainer.new(), false],
		["MarginContainer", MarginContainer.new(), true],
		["PanelContainer", PanelContainer.new(), true],
	]
	for c in cases:
		var parent: Control = c[1]
		host.add_child(parent)
		DccWidgets.group(parent, "a title long enough to wrap somewhere", false)
		await _frames(2)
		var btn := _last_button(parent)
		var wrapped: bool = btn != null and btn.autowrap_mode != TextServer.AUTOWRAP_OFF
		_check("guard: %s -> autowrap %s" % [c[0], "ON" if bool(c[2]) else "OFF"],
			wrapped == bool(c[2]), "autowrap_mode=%d" % (btn.autowrap_mode if btn != null else -1))

	## A `GridContainer` with one column still distributes -- the guard reads
	## the class, and a grid that is one column wide today can gain a second.
	var g1 := GridContainer.new()
	g1.columns = 1
	host.add_child(g1)
	DccWidgets.group(g1, "single column grid", false)
	await _frames(2)
	var g1b := _last_button(g1)
	_check("guard: GridContainer(columns=1) is still OFF",
		g1b != null and g1b.autowrap_mode == TextServer.AUTOWRAP_OFF)

	## A VERTICAL `BoxContainer` is a column, so it wraps -- the guard must not
	## be reading "is a BoxContainer" and stopping there.
	var vb := BoxContainer.new()
	vb.vertical = true
	host.add_child(vb)
	DccWidgets.group(vb, "vertical BoxContainer", false)
	await _frames(2)
	var vbb := _last_button(vb)
	_check("guard: BoxContainer(vertical=true) wraps",
		vbb != null and vbb.autowrap_mode != TextServer.AUTOWRAP_OFF)

	# == 2. the shell ==========================================================
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	var bridge = app.bridge
	var rd = app.right_dock_ctrl

	for seed_v in SEEDS:
		bridge.generate({"seed": seed_v, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
			"archetype": "", "villages": true, "sea_level": 0.45})
		var waited := 0
		while bridge.generating and waited < 3000:
			await get_tree().process_frame
			waited += 1
		await _frames(10)
		if not bridge.has_world:
			_check("seed %d generated" % seed_v, false)
			continue
		print("GG ---------------- seed %d ----------------" % seed_v)

		# -- the doc comment's own claim, asserted live ----------------------
		var live: Array = []
		var distributing := 0
		var unwrapped := 0
		var states := 0
		## The WHOLE app subtree, not just `left_dock_body` -- the claim in
		## `group()`'s doc is about every host in `shell/`, and the right dock
		## and any open window are hosts too.
		for pair in [["world", "a"], ["world", "b"], ["civilization", "landmarks"],
				["civilization", "factions"], ["civilization", "infra"],
				["civilization", "planner"], ["cartography", "style"],
				["cartography", "labels"], ["cartography", "icons"],
				["cartography", "terrain"]]:
			app.select_domain_mode(String(pair[0]), String(pair[1]))
			await _frames(12)
			states += 1
			var got: Array = []
			_headers(app, got)
			for b in got:
				live.append(b)
				var p := (b as Node).get_parent()
				if (p is BoxContainer and not (p as BoxContainer).vertical) or (p is HFlowContainer) or (p is GridContainer):
					distributing += 1
					print("GG    distributing parent: %s under %s" % [String((b as Button).text), p.get_class()])
				if (b as Button).autowrap_mode == TextServer.AUTOWRAP_OFF:
					unwrapped += 1
		## And every arm of `RightDock._tool_section()`, in the domain each
		## arm requires -- otherwise the whole sweep above is left-dock only.
		for combo in [["world", "paint"], ["cartography", "inspect"],
				["civilization", "label"], ["civilization", "icon"],
				["civilization", "territory"], ["world", "sculpt"]]:
			app.select_domain(String(combo[0]))
			await _frames(4)
			app.arm_tool(String(combo[1]))
			await _frames(12)
			states += 1
			var got2: Array = []
			_headers(app, got2)
			for b2 in got2:
				live.append(b2)
				var p2 := (b2 as Node).get_parent()
				if (p2 is BoxContainer and not (p2 as BoxContainer).vertical) or (p2 is HFlowContainer) or (p2 is GridContainer):
					distributing += 1
					print("GG    distributing parent: %s under %s" % [String((b2 as Button).text), p2.get_class()])
				if (b2 as Button).autowrap_mode == TextServer.AUTOWRAP_OFF:
					unwrapped += 1
		print("GG    swept %d surface states, %d live headers" % [states, live.size()])
		_check("seed %d: no live group header sits in a width-distributing parent" % seed_v,
			distributing == 0, "%d of %d headers" % [distributing, live.size()])
		_check("seed %d: every live group header autowraps" % seed_v,
			unwrapped == 0 and live.size() > 0, "%d unwrapped of %d" % [unwrapped, live.size()])

		## The ONE call site that rewrites a header after `group()` built it:
		## `civilization_workspace.gd::_lm_refresh_group()` re-appends
		## `"   %d of %d armed · %d placed"` on every toggle, which is what
		## makes `› HISTORICAL …` the widest header in the shell at 280 px and
		## what a source-string sweep cannot see. Its connection is made AFTER
		## `group()`'s own toggle handler on purpose, so the counts survive a
		## click; autowrap must not have disturbed that ordering.
		## `select_domain_category`, not `select_domain_mode`: the rail node's
		## DEFAULT category is not Landmarks, and a first version of this check
		## opened the mode and found 2 headers with no counts among them.
		app.select_domain_category("civilization", "Landmarks")
		await _frames(18)
		var lm: Array = []
		_headers(app.left_dock_body, lm)
		var fam: Button = null
		for b3 in lm:
			if String((b3 as Button).text).contains(" armed · "):
				fam = b3
				break
		_check("seed %d: a landmark family header carries its counts" % seed_v,
			fam != null, "%d headers on CIVIL ▸ Landmarks%s" % [lm.size(),
				"" if fam != null else ", none carrying counts"])
		if fam != null:
			var t0 := fam.text
			fam.emit_signal("pressed")
			await _frames(8)
			var t1 := fam.text
			fam.emit_signal("pressed")
			await _frames(8)
			_check("seed %d: the counts survive a toggle and round-trip" % seed_v,
				t1.contains(" armed · ") and fam.text == t0,
				"before=%s mid=%s after=%s" % [t0, t1, fam.text])
			_check("seed %d: and that header is drawn full width too" % seed_v,
				fam.size.x > 100.0 and fam.get_combined_minimum_size().x < 1.5,
				"drawn=%.0f min.x=%.1f" % [fam.size.x, fam.get_combined_minimum_size().x])

		# -- benefit: the two floors the measurement named --------------------
		app.select_domain_category("civilization", "Military")
		await _frames(18)
		var mil: float = await _drag_to(app.left_dock, DccTheme.W_LEFT_DOCK_MIN)
		_check("seed %d: CIVIL ▸ Military now reaches the 300 floor (was 306)" % seed_v,
			mil <= float(DccTheme.W_LEFT_DOCK_MIN) + 0.5, "stops at %.0f" % mil)

		var gs: Vector2i = bridge.grid_size()
		bridge.route_begin("mixed")
		bridge.route_append_stop(gs.x * 0.20, gs.y * 0.30)
		bridge.route_append_stop(gs.x * 0.55, gs.y * 0.50)
		bridge.route_append_stop(gs.x * 0.82, gs.y * 0.72)
		bridge.route_commit()
		app.select_domain("civilization")
		await _frames(4)
		var settlements: Array = bridge.settlements()
		if not settlements.is_empty():
			rd.on_settlement_selected(settlements[0], 0)
		app.arm_tool("journey")
		await _frames(18)
		var jp: float = await _drag_to(app.right_dock, DccTheme.W_RIGHT_DOCK_MIN)
		_check("seed %d: RIGHT ▸ Journey now reaches within 1 px of the 260 floor (was 273)" % seed_v,
			jp <= float(DccTheme.W_RIGHT_DOCK_MIN) + 1.5, "stops at %.0f" % jp)

		# -- the re-base: relationships, not just widths ----------------------
		var jhdrs: Array = []
		_headers(app.right_dock_body, jhdrs)
		var target: Button = null
		for b in jhdrs:
			if String((b as Button).text).contains("VESSEL REFERENCE"):
				target = b
		_check("seed %d: the row's own header is on screen" % seed_v, target != null,
			"headers=%d" % jhdrs.size())
		if target == null:
			continue

		## Relationship A -- the header beside its own body. `group()` appends
		## the header, then a `MarginContainer` holding the body; the toggle
		## must still flip exactly that body and nothing else.
		var idx := target.get_index()
		var owner_parent := target.get_parent()
		var pad: Node = owner_parent.get_child(idx + 1) if idx + 1 < owner_parent.get_child_count() else null
		var body: Control = (pad.get_child(0) as Control) if pad != null and pad.get_child_count() > 0 else null
		_check("seed %d: header is still immediately followed by its body's pad" % seed_v,
			pad is MarginContainer and body is VBoxContainer,
			"pad=%s body=%s" % [pad.get_class() if pad != null else "<null>",
				body.get_class() if body != null else "<null>"])
		if body == null:
			continue

		## Relationship B -- collapsed vs open. Height of the header must not
		## depend on the body's visibility, and the body must round-trip.
		var was_open := body.visible
		var h_before := target.size.y
		var text_before := target.text
		target.emit_signal("pressed")
		await _frames(6)
		var h_mid := target.size.y
		var open_mid := body.visible
		target.emit_signal("pressed")
		await _frames(6)
		_check("seed %d: the toggle still flips the body" % seed_v,
			open_mid != was_open and body.visible == was_open,
			"was=%s mid=%s after=%s" % [was_open, open_mid, body.visible])
		_check("seed %d: the header text round-trips through both toggles" % seed_v,
			target.text == text_before, "before=%s after=%s" % [text_before, target.text])
		_check("seed %d: header height does not move with the body's visibility" % seed_v,
			absf(h_mid - h_before) < 0.5, "%0.f -> %.0f" % [h_before, h_mid])

		## Relationship C -- one line at the shipped width, two when squeezed.
		## This is the whole point of the change, so it is asserted in both
		## directions: a header that wrapped at the DEFAULT width would be the
		## regression, and one that refused to wrap at the floor would mean the
		## change bought nothing.
		var h_ship := target.size.y
		var was_w: float = (app.right_dock as Control).custom_minimum_size.x
		app.right_dock.custom_minimum_size.x = float(DccTheme.W_RIGHT_DOCK_MIN)
		await _frames(6)
		var h_floor := target.size.y
		app.right_dock.custom_minimum_size.x = was_w
		await _frames(4)
		var h_back := target.size.y
		_check("seed %d: one line at the shipped width (%.0f px tall)" % [seed_v, h_ship],
			h_ship <= 24.0, "h=%.0f" % h_ship)
		_check("seed %d: it reflows to a second line at the floor, not an ellipsis" % seed_v,
			h_floor > h_ship, "shipped=%.0f floor=%.0f" % [h_ship, h_floor])
		_check("seed %d: and springs back when the dock is widened again" % seed_v,
			absf(h_back - h_ship) < 0.5, "%.0f -> %.0f" % [h_ship, h_back])

		## Relationship D -- the header must still be DRAWN at full width.
		## Autowrap drops the button's minimum x to **0** (measured, not to the
		## widest word as `action()`'s note implies for its own case), which is
		## one step from `MISTAKES.md`'s `clip_text` trap: a minimum of ~0 makes
		## a text control vanish beside a `SIZE_EXPAND_FILL` sibling. The guard
		## keeps it out of such a parent; this asserts the consequence rather
		## than the guard -- in its real column it still fills the dock.
		var min_x := target.get_combined_minimum_size().x
		var parent_w := (owner_parent as Control).size.x
		_check("seed %d: autowrap drops the header's minimum to ~0 (%.0f px)" % [seed_v, min_x],
			min_x < 1.5, "min.x=%.1f" % min_x)
		_check("seed %d: yet it is still drawn at its column's full width" % seed_v,
			target.size.x >= parent_w - 0.5 and target.size.x > 100.0,
			"drawn=%.0f column=%.0f" % [target.size.x, parent_w])
		_check("seed %d: and its text is intact, not clipped or ellipsised" % seed_v,
			not target.clip_text and target.text.contains("SPEED BY WATER"),
			"clip_text=%s text=%s" % [target.clip_text, target.text])
		app.arm_tool("inspect")
		await _frames(6)

	print("GG === %d failures ===" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

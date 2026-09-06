extends Node
## **Investigation, not a guard.** `_widthfloor_probe.gd` made `phone_fit()`'s
## 44 dp tap floor unconditional on both axes and then measured a residue: a set
## of controls that now carry the floor as a `custom_minimum_size` and are still
## drawn smaller than it, on **both** axes, so setting the minimum changed
## nothing about them. This probe answers the three questions that residue
## raises and deliberately asserts almost nothing:
##
##   1. **What are they**, by name and owning script.
##   2. **Why does the parent ignore the minimum** -- which is a question about
##      classes of cause, not about 30 individual controls.
##   3. **Which of them is actually un-tappable by a finger**, which is the only
##      part that is a user-visible defect.
##
## Run (headless is correct: every number here is a laid-out size or a
## `get_combined_minimum_size()`, nothing rasterises and nothing is timed):
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _tapfloor_probe.tscn -- --force-touch
##
## `--force-touch` is required for the reason `_phonechrome_probe.gd` records:
## `_phone` can never be true in this dev environment without it.
##
## ## Density, named beside every number
##
## 1080 x 2340 portrait handset. `phone_scale` is **2.621**, so the screen is
## **412 dp** wide and the 44 dp tap floor is **115 physical px**. Every `px`
## below is physical; every `dp` is that figure divided by the control's own
## density (`_phys_scale()`, because a control inside a `content_scale_factor`
## window lays out in dp already and comparing its raw size against 115 would
## call a correctly-floored dialog button 62% undersized).
##
## ## Why "drawn smaller than its own minimum" is possible at all
##
## `Container._sort_children` distributes the space the container HAS. A
## `BoxContainer` given less than its children's combined minimum does not
## overflow and does not scroll -- it divides what it has and hands each child a
## share **below** that child's minimum. So a minimum is a request the parent
## honours only while it can afford to, and `custom_minimum_size` on a child of
## a starved container is inert. The starvation is what this probe locates: it
## walks up from each victim to the first ancestor that is at least its own
## combined minimum, and reports the boundary.

var _fail := 0
const FLOOR_DP := 44.0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## Same exclusion `_widthfloor_probe.gd` and `_phonechrome_probe.gd` make, for
## the same reason: `PhoneMenuModel` is a permanently-`visible = false` clone of
## the DESKTOP menu bar, kept only so `PhoneMenu` can read its structure.
func _collect(node: Node, out: Array) -> void:
	if node.name == "PhoneMenuModel":
		return
	if node is Control:
		out.append(node)
	for c in node.get_children():
		_collect(c, out)

## The density a control lays out in -- `_widthfloor_probe.gd::_phys_scale`,
## same walk and same reason.
func _phys_scale(c: Control) -> float:
	var n: Node = c
	while n != null:
		if n is Window:
			return maxf(0.0001, (n as Window).content_scale_factor)
		n = n.get_parent()
	return 1.0

## Nearest ancestor carrying a script -- the file that built this control.
func _owner_script(ctl: Control) -> String:
	var n: Node = ctl.get_parent()
	while n != null:
		var s: Script = n.get_script() as Script
		if s != null:
			return s.resource_path.get_file()
		n = n.get_parent()
	return "-"

## A readable trail of node NAMES from the nearest scripted ancestor down, so a
## reader can find the control in source without an @-generated path.
func _trail(ctl: Control) -> String:
	var parts: Array[String] = []
	var n: Node = ctl
	var hops := 0
	while n != null and hops < 8:
		parts.push_front(str(n.name))
		if (n.get_script() as Script) != null and n != ctl:
			break
		n = n.get_parent()
		hops += 1
	return "/".join(parts)

## The nearest ancestor with a real (non-`@`-generated) name -- the landmark a
## reader can actually find in the scene tree. `@`-names are allocated at
## runtime and mean nothing between runs.
func _named_ancestor(ctl: Control) -> String:
	var n: Node = ctl.get_parent()
	while n != null:
		if not str(n.name).begins_with("@"):
			return str(n.name)
		n = n.get_parent()
	return "(none)"


func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	if not ("--force-touch" in OS.get_cmdline_user_args()):
		print("[FATAL] run with `-- --force-touch` -- _phone can never be true without it")
		get_tree().quit(1); return

	var vp := SubViewport.new()
	vp.size = Vector2i(1080, 2340)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(50)
	if not bool(app.is_phone()):
		print("[FATAL] booted into the desktop/tablet composition, not phone")
		get_tree().quit(1); return
	var scale: float = float(app.phone_scale())
	var floor_px: float = float(app.call("_pscale", 44))
	print("[BOOT] 1080x2340 portrait, phone_scale=%.3f (%.0f dp wide), tap floor = %.0f physical px"
		% [scale, 1080.0 / scale, floor_px])

	## The same warm-up `_widthfloor_probe.gd` performs, for the reason it
	## records: a subtree that has never been visible has never had a real
	## container-sort pass, so its sizes read back as 0 and mean nothing.
	app.call("_set_overflow_open", true); await _frames(2)
	app.call("_set_sheet_open", "left", true); await _frames(2)
	app.call("_set_sheet_open", "right", true); await _frames(2)
	app.call("_close_all_phone_overlays")
	await _frames(2)
	var dom0: String = str(app.call("active_domain"))
	var mode0: String = str(app.call("active_mode", dom0))
	var tool0: String = str(app.get("armed_tool"))
	app.call("open_journey_planner")
	await _frames(20)

	var all: Array = []
	_collect(app.get("_phone_root"), all)

	## The residue, restated as its own predicate: the floor is SET (both axes,
	## in the control's own density) and the control is drawn under it on BOTH
	## axes. A control under the width floor alone is a different population --
	## `_widthfloor_probe.gd` asserts on that one and it is 0.
	var hits: Array = []
	var width_only := 0
	for ctl in all:
		var c := ctl as Control
		var ps := _phys_scale(c)
		var w := c.size.x * ps
		var h := c.size.y * ps
		if w <= 0.5 or h <= 0.5:
			continue
		var mx := c.custom_minimum_size.x * ps
		var my := c.custom_minimum_size.y * ps
		if mx < floor_px - 0.5 or w >= floor_px - 0.5:
			continue
		if my >= floor_px - 0.5 and h < floor_px - 0.5:
			hits.append(c)
		else:
			width_only += 1

	print("\n=== the residue at 1080x2340 (phone_scale %.3f, floor %.0f px) ===" % [scale, floor_px])
	print("  controls walked: ", all.size())
	print("  floored on BOTH axes and drawn under BOTH: ", hits.size(),
		"   (floored on width, drawn narrow, but tall enough: ", width_only, ")")

	## --- 1. what they are -------------------------------------------------
	var by_class := {}
	var by_script := {}
	for c in hits:
		var ctl := c as Control
		by_class[ctl.get_class()] = int(by_class.get(ctl.get_class(), 0)) + 1
		var s := _owner_script(ctl)
		by_script[s] = int(by_script.get(s, 0)) + 1
	print("\n--- 1. what they are ---")
	print("  by class:  ", by_class)
	print("  by owning script:  ", by_script)
	for c in hits:
		var ctl := c as Control
		var ps := _phys_scale(ctl)
		print("    %-13s %6.1f x %-6.1f px  = %5.1f x %-5.1f dp   min=(%.0f,%.0f)  text=%s"
			% [ctl.get_class(), ctl.size.x * ps, ctl.size.y * ps,
			   ctl.size.x * ps / scale, ctl.size.y * ps / scale,
			   ctl.custom_minimum_size.x * ps, ctl.custom_minimum_size.y * ps,
			   str(ctl.get("text"))]
			+ "   %s  [%s]" % [_trail(ctl), _owner_script(ctl)])

	## --- 2. why the parent ignores it -------------------------------------
	## Walk up to the first ancestor that is at least its own combined minimum
	## on the starved axis. Everything below that boundary is being handed less
	## than it asked for; the boundary itself is where the space runs out.
	print("\n--- 2. why the minimum is ignored: the starvation boundary ---")
	var causes := {}
	for c in hits:
		var ctl := c as Control
		var n: Node = ctl.get_parent()
		var chain: Array[String] = []
		var boundary := "(reached the root still starved)"
		var boundary_class := "(root)"
		while n != null and n is Control:
			var p := n as Control
			var cmw := p.get_combined_minimum_size().x
			var cmh := p.get_combined_minimum_size().y
			chain.append("%s[%s] size=(%.0f,%.0f) cmin=(%.0f,%.0f)%s"
				% [p.get_class(), p.name, p.size.x, p.size.y, cmw, cmh,
				   "" if p.visible else " HIDDEN"])
			if p.size.x >= cmw - 0.5 and p.size.y >= cmh - 0.5:
				boundary = chain[chain.size() - 1]
				boundary_class = "%s fits its own minimum -> the squeeze starts below it" % p.get_class()
				break
			n = p.get_parent()
		## The class of cause, as a short key a report can group on.
		var key := "%s in %s, starved by %s" % [ctl.get_class(),
			(ctl.get_parent() as Control).get_class() if ctl.get_parent() is Control else "?",
			boundary_class]
		causes[key] = int(causes.get(key, 0)) + 1
		if int(causes[key]) == 1:
			print("  cause: ", key)
			for i in mini(chain.size(), 6):
				print("      ^ ", chain[i])
	print("  cause tally: ", causes)

	## --- 3. is any of them actually un-tappable ---------------------------
	## The question that decides whether this is a defect or a bookkeeping
	## artefact. A control is only tappable if it is visible in the tree AND
	## takes input; and it is only *hard* to tap if what it draws is small.
	## Both halves are measured, neither assumed.
	print("\n--- 3. is any of them un-tappable in practice ---")
	var visible_n := 0
	var takes_input := 0
	var small_dp := 0
	var tiny_dp := 0
	for c in hits:
		var ctl := c as Control
		var ps := _phys_scale(ctl)
		var wdp := ctl.size.x * ps / scale
		var hdp := ctl.size.y * ps / scale
		var vis := ctl.is_visible_in_tree()
		var input := ctl.mouse_filter != Control.MOUSE_FILTER_IGNORE
		if vis:
			visible_n += 1
		if vis and input:
			takes_input += 1
			if minf(wdp, hdp) < FLOOR_DP:
				small_dp += 1
			if minf(wdp, hdp) < 24.0:
				tiny_dp += 1
	print("  visible in tree: ", visible_n, " of ", hits.size())
	print("  visible AND accepting input (mouse_filter != IGNORE): ", takes_input)
	print("  ...of those, smaller than the ", FLOOR_DP, " dp floor on their short axis: ", small_dp)
	print("  ...of those, under 24 dp on their short axis (a genuinely hard target): ", tiny_dp)
	for c in hits:
		var ctl := c as Control
		var ps := _phys_scale(ctl)
		print("    %-13s vis_in_tree=%-5s filter=%d  %5.1f x %-5.1f dp   %s"
			% [ctl.get_class(), str(ctl.is_visible_in_tree()), ctl.mouse_filter,
			   ctl.size.x * ps / scale, ctl.size.y * ps / scale, _trail(ctl)])

	## --- 4. the hidden ancestor, named ------------------------------------
	## "Not visible in the tree" is a fact about an ANCESTOR, not about the
	## control, and which ancestor decides whether this is a defect waiting to
	## appear or an artefact of measuring a collapsed panel. Named here rather
	## than inferred from the path.
	print("\n--- 4. which ancestor is hidden ---")
	var hidden_owners := {}
	var to_show: Array[Control] = []
	for c in hits:
		var ctl := c as Control
		var n: Node = ctl
		var first := true
		## EVERY invisible ancestor, not just the nearest: showing one and
		## leaving another hidden re-measures nothing, which is what a first
		## pass of this probe did to six of the thirty.
		while n != null:
			if n is Control and not (n as Control).visible:
				var h := n as Control
				if first:
					first = false
					var k := "%s[%s] under %s, built by %s" % [h.get_class(), h.name,
						_named_ancestor(h), _owner_script(h)]
					hidden_owners[k] = int(hidden_owners.get(k, 0)) + 1
				if not to_show.has(h):
					to_show.append(h)
			n = n.get_parent()
	print("  nearest hidden ancestor, by owner: ", hidden_owners)

	## --- 5. the experiment: show them and re-measure ----------------------
	## The question the count cannot answer on its own -- **is the floor
	## ignored, or has the subtree simply never been laid out?** A container
	## that is 0 px wide divides 0 px among its children and every child gets a
	## sliver whatever it asked for; that is not a parent disregarding a
	## minimum, it is a parent with nothing to hand out. Showing the hidden
	## ancestor forces a real container sort, and the same 30 controls are
	## re-measured against the same floor.
	print("\n--- 5. shown, then re-measured ---")
	print("  showing ", to_show.size(), " hidden ancestors")
	var was_visible := {}
	for h in to_show:
		was_visible[h] = h.visible
		h.visible = true
	await _frames(20)
	var still_under := 0
	var now_ok := 0
	var still_invisible := 0
	var worst := 999.0
	for c in hits:
		var ctl := c as Control
		var ps := _phys_scale(ctl)
		var wdp := ctl.size.x * ps / scale
		var hdp := ctl.size.y * ps / scale
		if not ctl.is_visible_in_tree():
			still_invisible += 1
			continue
		if minf(wdp, hdp) < FLOOR_DP - 0.2:
			still_under += 1
			worst = minf(worst, minf(wdp, hdp))
			print("    STILL UNDER  %-13s %5.1f x %-5.1f dp   %s"
				% [ctl.get_class(), wdp, hdp, _trail(ctl)])
		else:
			now_ok += 1
	print("  of the %d: %d now meet the %.0f dp floor, %d still under it, %d still not visible"
		% [hits.size(), now_ok, FLOOR_DP, still_under, still_invisible])
	if still_under > 0:
		print("  worst short axis once shown: %.1f dp" % worst)
	for h in to_show:
		h.visible = bool(was_visible[h])
	await _frames(5)

	## Restore, and assert the restore rather than assuming it -- the planner
	## warm-up above changed the shell's state.
	app.call("select_domain_mode", dom0, mode0)
	app.call("arm_tool", tool0)
	await _frames(10)
	if str(app.call("active_domain")) != dom0:
		_fail += 1
		print("  FAIL the planner warm-up left the domain as ", app.call("active_domain"),
			" not ", dom0)
	if str(app.get("armed_tool")) != tool0:
		_fail += 1
		print("  FAIL the planner warm-up left the armed tool as ", app.get("armed_tool"),
			" not ", tool0)

	print("\n_tapfloor_probe: ", "clean" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

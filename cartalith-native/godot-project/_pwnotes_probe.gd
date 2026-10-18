extends Node
## Places without notes probe -- milestone SM-5 of `STORY_MENUS_PROPOSAL.md`
## (§5, Ruling CK, 2026-10-18; mockup `design/story-menus-2026-10-17/` A5).
##
## Drives `vault_window.gd`'s Places without notes section on the live shell
## and asserts each of A5's states and what a row does:
##
##   unbound    no vault connected: the dashed "No vault connected" card and its
##              Connect button, and no row (the list would be the whole world)
##   no world   a vault bound, nothing generated: the "No world yet" card
##   list       a generated world: "N of M settlements have no linked note",
##              N == M with no link, one row per place up to the 40-row page,
##              Show more when N > 40, rows sorted by class then population
##   agreement  the list equals, settlement by settlement, the per-entity
##              answer `vault_entity_summary(kind, tid).link_count == 0` --
##              the independent route §5 first named -- before and after a
##              real attach (`vault_attach`), and the attached place leaves
##   filters    a class chip keeps only that class; On a journey / In a
##              conflict are disabled with a reason while the store holds none
##   all linked every continent attached: "Every place has a note"
##   rows       New from template… and Attach existing… open the place's own
##              scoped vault view (the existing VA-02 / attach flows)
##   generating opened mid-generation: the "While a generation runs" card, the
##              last list drawn with its buttons disabled, and no engine read
##   timing     `_pwn_compute` (two engine reads) against the N+1
##              `vault_entity_summary` route, median of 5 with min..max, on the
##              world the run generated; `--big WxH` adds a larger world
##
## Screenshots (`--out DIR`): the list, and the unbound / no-world / generating
## / all-linked cards, named `pwn_<form>_<state>.png`.
##
## Run WINDOWED from `godot-project/` (a capture needs a real frame):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _pwnotes_probe.tscn -- --vp 1366x768 --out C:/abs/dir
##   ... -- --vp 1920x1080 --out C:/abs/dir --big 2048x1311
##   ... -- --force-touch --vp 1600x2560 --out C:/abs/dir    (tablet)
##   ... -- --force-touch --vp 393x852 --out C:/abs/dir      (phone)
##
## `--vp WxH` sizes the host SubViewport; `--force-touch` is `dcc_shell.gd`'s
## own override; `--out DIR` is where PNGs and the probe's scratch vault go
## (required: the vault folder is `DIR/vault`); `--big WxH` times a second,
## larger world after the checks.
##
## Result: `PWN ok  |FAIL  <check>` rows, `PWN TIME ...` lines, `PWN SHOT
## <path>`, and `PWN RESULT <n> checks, <f> failed (<form>)`. Exit 0 when
## nothing failed, 1 when a check failed, 2 when the shell could not be
## measured (no extension, no `--out`), 3 on the watchdog.

var app: Node
var _vp: SubViewport
var _checks := 0
var _fails := 0
var _out := ""
var _form := "desktop"

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("PWN %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _shot(name: String) -> void:
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	if img == null:
		_check("capture %s returned an image" % name, false)
		return
	var path := _out.path_join("pwn_%s_%s.png" % [_form, name])
	img.save_png(path)
	print("PWN SHOT %s" % path)

## The vault window's state card titles currently drawn (`PwnState` panels).
func _state_titles(vw: Node) -> Array:
	var out: Array = []
	for card in vw.find_children("PwnState", "", true, false):
		var labels := (card as Node).find_children("*", "Label", true, false)
		if not labels.is_empty():
			out.append((labels[0] as Label).text)
	return out

## Rows drawn in the section, as `pwn_id` metas, in draw order. Found by the
## meta, not the node name: siblings that share a name are auto-renamed by
## `add_child`, so only the first row keeps `PwnRow`.
func _row_ids(vw: Node) -> Array:
	return _rows(vw).map(func(r): return int(r.get_meta("pwn_id", -1)))

## The row nodes (`_pwn_row`'s containers), in draw order.
func _rows(vw: Node) -> Array:
	return vw.find_children("*", "BoxContainer", true, false).filter(func(n): return n.has_meta("pwn_id"))

func _readout(vw: Node) -> String:
	var r: Node = vw.find_child("PwnReadout", true, false)
	return (r as Label).text if r != null else ""

func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 900.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("PWN WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("PWN FATAL extension did not load")
		get_tree().quit(2)
		return
	_out = _arg("--out", "")
	if _out == "":
		print("PWN FATAL --out DIR is required (the scratch vault lives there)")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	var parts: PackedStringArray = _arg("--vp", "1366x768").split("x")
	var vw_px := int(parts[0])
	var vh_px := int(parts[1])
	DisplayServer.window_set_size(Vector2i(mini(vw_px, 1600), mini(vh_px, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw_px, vh_px)
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(6)
	_form = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_form += "_%dx%d" % [vw_px, vh_px]
	print("PWN form=%s" % _form)
	var bridge = app.bridge
	var vw = app.vault_window

	## -- unbound: a clean settings dir carries no vault binding.
	if bool(bridge.vault_info().get("bound", false)):
		bridge.vault_disconnect()
	app.open_places_without_notes()
	await _frames(6)
	_check("unbound: the No vault connected card is drawn",
		_state_titles(vw).has("No vault connected"), str(_state_titles(vw)))
	_check("unbound: the card offers Connect a vault folder…",
		vw.find_child("PwnConnect", true, false) != null)
	_check("unbound: no place row is drawn", _row_ids(vw).is_empty())
	await _shot("unbound")
	vw.hide()

	## -- a scratch vault: one template (its path holds "template"), one note.
	var vault_dir := _out.path_join("vault")
	_write(vault_dir.path_join("Templates/place template.md"), "# {{title}}\n\nNotes.\n")
	_write(vault_dir.path_join("Settlements/Existing note.md"), "# Existing note\n\nText.\n")
	var con: Dictionary = bridge.vault_connect(vault_dir, "probe vault")
	_check("the scratch vault connects", bool(con.get("ok", false)), str(con))

	## -- bound, no world.
	app.open_places_without_notes()
	await _frames(6)
	_check("no world: the No world yet card is drawn",
		_state_titles(vw).has("No world yet"), str(_state_titles(vw)))
	await _shot("noworld")
	vw.hide()

	## -- a world.
	bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bool(bridge.generating):
		await get_tree().create_timer(0.25).timeout
	await _frames(4)
	var setts: Array = bridge.settlements()
	_check("the probe world has more than 40 settlements (the page is exercised)",
		setts.size() > 40, str(setts.size()))
	app.open_places_without_notes()
	await _frames(6)
	var n := setts.size()
	_check("list: readout says N of N settlements have no linked note",
		_readout(vw) == "%d of %d settlements have no linked note" % [n, n], _readout(vw))
	var ids := _row_ids(vw)
	_check("list: one 40-row page is drawn", ids.size() == mini(n, 40), str(ids.size()))
	_check("list: Show more is offered when N > 40",
		(vw.find_child("PwnMore", true, false) != null) == (n > 40))
	## Sorted by class rank, then population (largest first).
	var rank := {"metropolis": 0, "capital": 1, "city": 2, "town": 3, "village": 4, "hamlet": 5}
	var by_tid := {}
	for s in setts:
		by_tid[int(s["tid"])] = s
	var sorted_ok := true
	for i in range(1, ids.size()):
		var a: Dictionary = by_tid.get(ids[i - 1], {})
		var b: Dictionary = by_tid.get(ids[i], {})
		var ra := int(rank.get(String(a.get("kind", "")), 9))
		var rb := int(rank.get(String(b.get("kind", "")), 9))
		if ra > rb or (ra == rb and int(a.get("population", 0)) < int(b.get("population", 0))):
			sorted_ok = false
	_check("list: rows are sorted by class, then population", sorted_ok)
	await _shot("list")
	## Show more grows the page.
	var more: Button = vw.find_child("PwnMore", true, false)
	if more != null:
		more.pressed.emit()
		await _frames(4)
		_check("Show more draws the next page", _row_ids(vw).size() == mini(n, 80), str(_row_ids(vw).size()))

	## -- agreement with the per-entity route, then a real attach.
	var agree := func() -> Array:
		var got: Dictionary = vw._pwn_compute("settlement")
		var listed := {}
		for r in got["rows"]:
			listed[int(r["id"])] = true
		var bad: Array = []
		for s in bridge.settlements():
			var tid := int(s["tid"])
			var none := int(bridge.vault_entity_summary("settlement", tid).get("link_count", -1)) == 0
			if none != listed.has(tid):
				bad.append(tid)
		return bad
	var bad0: Array = agree.call()
	_check("agreement: the list equals vault_entity_summary's link_count == 0, place by place",
		bad0.is_empty(), str(bad0))
	var t0: Dictionary = setts[0]
	var att: Dictionary = bridge.vault_attach("settlement", int(t0["tid"]), String(t0["name"]),
		"Settlements/Existing note.md", "")
	_check("a real attach succeeds", bool(att.get("ok", false)), str(att))
	app.open_places_without_notes()
	await _frames(6)
	_check("after the attach: N-1 of N", _readout(vw) == "%d of %d settlements have no linked note" % [n - 1, n],
		_readout(vw))
	_check("after the attach: the attached place is not listed",
		not (vw._pwn_compute("settlement")["rows"] as Array).any(func(r): return int(r["id"]) == int(t0["tid"])))
	var bad1: Array = agree.call()
	_check("agreement holds after the attach", bad1.is_empty(), str(bad1))

	## -- filters.
	var classes := {}
	for s in setts:
		classes[String(s["kind"])] = true
	_check("no journey or conflict yet: both settlement chips are disabled with a reason",
		(vw.find_child("PwnJourneyChip", true, false) as Button).disabled
			and (vw.find_child("PwnConflictChip", true, false) as Button).disabled
			and (vw.find_child("PwnJourneyChip", true, false) as Button).tooltip_text.begins_with("No journey"))
	if classes.has("village"):
		vw._pwn_set("class", "village")
		await _frames(4)
		var vids := _row_ids(vw)
		_check("VILLAGES keeps only villages",
			not vids.is_empty() and vids.all(func(t): return String(by_tid[t]["kind"]) == "village"), str(vids.size()))
		vw._pwn_set("class", "")
		await _frames(4)
	## A conflict anchored at a listed place makes In a conflict live.
	var s1: Dictionary = setts[1]
	var ca: Dictionary = bridge.conflict_add({"name": "Probe siege", "kind": "siege",
		"start_year": int(bridge.get_civ_year()), "points": PackedVector2Array([Vector2(float(s1["x"]), float(s1["y"]))]),
		"anchor_kind": "settlement", "anchor_tid": int(s1["tid"])})
	_check("the probe's anchored conflict is recorded", bool(ca.get("ok", false)), str(ca))
	app.open_places_without_notes()
	await _frames(6)
	var cchip := vw.find_child("PwnConflictChip", true, false) as Button
	_check("with a conflict recorded, In a conflict is enabled", cchip != null and not cchip.disabled)
	if cchip != null and not cchip.disabled:
		vw._pwn_set("conflict", true)
		await _frames(4)
		var cids := _row_ids(vw)
		var touched := cids.all(func(t): return not bridge.conflicts_touching_settlement(int(t)).is_empty())
		_check("In a conflict keeps only places a conflict touches (%d rows)" % cids.size(), touched)
		vw._pwn_set("conflict", false)
		await _frames(4)

	## -- rows open the existing flows.
	var first_row: Node = _rows(vw)[0] if not _rows(vw).is_empty() else null
	var tid0 := int(first_row.get_meta("pwn_id", -1)) if first_row != null else -1
	var newb := first_row.find_child("PwnNew", true, false) as Button if first_row != null else null
	_check("New from template… is enabled (the scratch vault has a template)", newb != null and not newb.disabled,
		newb.tooltip_text if newb != null else "missing")
	if newb != null:
		newb.pressed.emit()
		await _frames(4)
		_check("New from template… opens the place's scoped view on its attach flow",
			String(vw._kind) == "settlement" and int(vw._entity_id) == tid0
				and vw.find_child("VaultAttachFlow", true, false) != null,
			"%s %d" % [vw._kind, vw._entity_id])
	app.open_places_without_notes()
	await _frames(6)
	first_row = _rows(vw)[0] if not _rows(vw).is_empty() else null
	var attb := first_row.find_child("PwnAttach", true, false) as Button if first_row != null else null
	if attb != null:
		attb.pressed.emit()
		await _frames(4)
		_check("Attach existing… opens the place's scoped view", String(vw._kind) == "settlement"
			and int(vw._entity_id) == tid0)
		if DccTheme.is_phone():
			_check("phone: Attach existing… lands on the tree pane", String(vw._browse_phone_pane) == "tree")

	## -- all linked: every continent attached.
	var conts: Array = bridge.continents()
	for c in conts:
		bridge.vault_attach("continent", int(c["id"]), String(c["name"]), "Settlements/Existing note.md", "")
	app.open_places_without_notes()
	await _frames(4)
	vw._pwn_set("kind", "continent")
	await _frames(6)
	if conts.is_empty():
		_check("this world has continents to link", false)
	else:
		_check("all linked: the Every place has a note card for continents",
			_state_titles(vw).has("Every place has a note") and _readout(vw).begins_with("0 of %d continents" % conts.size()),
			"%s / %s" % [str(_state_titles(vw)), _readout(vw)])
		await _shot("alllinked")
	vw._pwn_set("kind", "settlement")
	await _frames(4)
	vw.hide()

	## -- timing on this world.
	_time_world(vw, bridge, "256x192")

	## -- generating: open mid-generation.
	app.open_places_without_notes()
	await _frames(4)
	vw.hide()
	bridge.generate({"seed": 4243, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	await _frames(1)
	if bool(bridge.generating):
		app.open_places_without_notes()
		await _frames(2)
		_check("generating: the While a generation runs card is drawn",
			_state_titles(vw).has("While a generation runs"), str(_state_titles(vw)))
		var dashed: Array = vw.find_children("PwnNew", "", true, false)
		_check("generating: the last list is drawn with every button disabled",
			not dashed.is_empty() and dashed.all(func(b): return (b as Button).disabled), str(dashed.size()))
		await _shot("generating")
		vw.hide()
	else:
		_check("the generation was still running one frame in (gate leg reachable)", false)
	while bool(bridge.generating):
		await get_tree().create_timer(0.25).timeout
	await _frames(4)

	## -- optional larger world, timing only.
	var big := _arg("--big", "")
	if big != "":
		var bp := big.split("x")
		var t_gen := Time.get_ticks_msec()
		bridge.generate({"seed": 4242, "width_km": float(_arg("--big-km", "12000")), "grid_w": int(bp[0]), "grid_h": int(bp[1]),
			"archetype": "", "villages": true, "sea_level": 0.45})
		while bool(bridge.generating):
			await get_tree().create_timer(0.5).timeout
		await _frames(4)
		print("PWN big world %s generated in %d ms" % [big, Time.get_ticks_msec() - t_gen])
		_time_world(vw, bridge, big)
		## The whole open, as a user pays it: overview rebuild included.
		var opens: Array = []
		for k in 5:
			var ts := Time.get_ticks_usec()
			app.open_places_without_notes()
			opens.append(Time.get_ticks_usec() - ts)
			vw.hide()
			await _frames(2)
		opens.sort()
		print("PWN TIME %s open_places_without_notes (whole overview rebuild) median %.1f ms (%.1f..%.1f)" % [
			big, opens[2] / 1000.0, opens[0] / 1000.0, opens[4] / 1000.0])
		app.open_places_without_notes()
		await _frames(6)
		await _shot("big")
		vw.hide()

	print("PWN RESULT %d checks, %d failed (%s)" % [_checks, _fails, _form])
	get_tree().quit(1 if _fails > 0 else 0)

## Times the two routes to the same answer on the current world: `_pwn_compute`
## (one `vault_all_links` + one entity list) and the N+1 `vault_entity_summary`
## walk. Five runs each, median with min..max.
func _time_world(vw: Node, bridge, label: String) -> void:
	var n: int = bridge.settlements().size()
	var a: Array = []
	var b: Array = []
	for k in 5:
		var t0 := Time.get_ticks_usec()
		vw._pwn_compute("settlement")
		a.append(Time.get_ticks_usec() - t0)
		var t1 := Time.get_ticks_usec()
		var none := 0
		for s in bridge.settlements():
			if int(bridge.vault_entity_summary("settlement", int(s["tid"])).get("link_count", 0)) == 0:
				none += 1
		b.append(Time.get_ticks_usec() - t1)
	a.sort()
	b.sort()
	print("PWN TIME %s (%d settlements) _pwn_compute median %.2f ms (%.2f..%.2f); N+1 vault_entity_summary median %.2f ms (%.2f..%.2f)" % [
		label, n, a[2] / 1000.0, a[0] / 1000.0, a[4] / 1000.0, b[2] / 1000.0, b[0] / 1000.0, b[4] / 1000.0])

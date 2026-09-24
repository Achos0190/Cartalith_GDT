extends Node
## Committed verification harness for the 2026-09-05 right-dock build:
## `design/proposed-2026-09-05/SettlementExtras.dc.html`'s three appended
## sections, `DeltaVertical.dc.html`'s horizontal profile, and the removal of
## the `§ STEPS` header HISTORY drew above its own `N STEPS` row.
##
## **Every asserted figure is the artboard's or the spec's own literal**, never
## the constant that produced it -- `84` for the plan box, `121` for the
## profile samples, `6` for the adherence bar, `40`/`118`/`14` for the chart's
## three boxes and `43`/`86` for §5.7's gridlines. Batch 34 shipped a probe
## that read `DccWidgets.MODAL_RADIUS` back and compared it to itself; seven
## constants survived mutation with the label printing the wrong number.
##
## **Runs windowed and forces the dark palette.** This machine boots
## `mode="light"` (`cartalith_settings.cfg`), and part D reads the framebuffer
## back -- `ImageTexture.update()` and viewport capture are both no-ops under
## `--headless`, so a headless run of part D would pass vacuously. It refuses
## to run in light rather than measuring the wrong theme.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _setxvert_probe.tscn

const SEEDS := [483920, 77021, 4242]
## `SettlementExtras.dc.html`: `width:84px;height:84px` on the plan frame,
## `height:6px` on the adherence bar.
const ART_THUMB_PX := 84.0
const ART_FAITH_BAR_H := 6.0
## `DeltaVertical.dc.html` / `05-right-dock-and-bars.md` §5.7: a 40 px metre
## column at dock width, a 132 px chart column of which the bottom 14 px is the
## x-label strip, and two gridlines at y=43 and y=86 of a 130-unit viewBox.
const ART_Y_COL := 40.0
const ART_CHART_H := 132.0
const ART_AXIS_H := 14.0
const SPEC_GRID_Y := [43.0, 86.0]
const SPEC_VIEWBOX_H := 130.0
## §5.7 defect 2: the prototype's header says 120 where its loop
## (`for i = 0; i <= n; i++`, `n = 120`) yields 121.
const SPEC_SAMPLES := 121

var app: Node
var _fail := 0
var _sel_name := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _p(s: String) -> void:
	print("SXV %s" % s)

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("SXV %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

# -- Reading the drawn tree back, never the variables that fed it ------------

func _nodes(n: Node = null, out: Array = []) -> Array:
	var root: Node = n if n != null else app.right_dock_body
	for c in root.get_children():
		out.append(c)
		_nodes(c, out)
	return out

func _texts() -> Array:
	var out: Array = []
	for n in _nodes():
		if n is Label:
			out.append((n as Label).text)
		elif n is Button:
			out.append((n as Button).text)
	return out

func _has_text(t: String) -> bool:
	return _texts().has(t)

## Index of the first node whose text is exactly `t`, in tree order, or -1.
## A position, not a bool: "appended" is a claim about order.
func _pos(t: String) -> int:
	var ts := _texts()
	for i in ts.size():
		if String(ts[i]) == t:
			return i
	return -1

func _node_with_text(t: String) -> Node:
	for n in _nodes():
		if (n is Label and (n as Label).text == t) or (n is Button and (n as Button).text == t):
			return n
	return null

## Every `Panel` whose `custom_minimum_size` matches, to the pixel.
func _panels_sized(w: float, h: float) -> Array:
	var out: Array = []
	for n in _nodes():
		if n is Panel:
			var m: Vector2 = (n as Panel).custom_minimum_size
			if is_equal_approx(m.x, w) and is_equal_approx(m.y, h):
				out.append(n)
	return out

func _force_dark() -> bool:
	if not DccTheme.is_dark():
		DccTheme.apply_theme(true)
		app.rebuild_theme(false)
	return DccTheme.is_dark()

func _gen(seed_v: int) -> bool:
	app.bridge.generate({"seed": seed_v, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while app.bridge.generating and waited < 4000:
		await get_tree().process_frame
		waited += 1
	await _frames(10)
	return app.bridge.has_world

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	if not _force_dark():
		_p("!! refusing to run: the palette is light and every literal below is the")
		_p("!! dark artboard's. Nothing here would be measuring the theme it asserts.")
		get_tree().quit(2)
		return
	_p("palette forced dark: is_dark=%s tablet=%s" % [DccTheme.is_dark(), DccTheme.is_tablet()])

	if not await _gen(SEEDS[0]):
		_p("!! generate failed -- nothing else here can run")
		get_tree().quit(1)
		return
	var places: Array = app.bridge.settlements()
	_p("world %s, %d settlements" % [app.bridge.grid_size(), places.size()])
	if places.is_empty():
		get_tree().quit(1)
		return

	## A richer FAITH fixture than the default world gives: `civ_belief_run`
	## reports `any_faith: false` on a fresh generate, because every faction's
	## religion is None -- one segment at 100 %, which never reaches the
	## artboard's top-three-plus-remainder path. Four factions, four faiths from
	## the engine's own vocabulary, then one diffusion.
	var vocab: Array = app.bridge.civ_religion_vocabulary()
	_p("religion vocabulary: %s" % [vocab])
	var assigned := 0
	for i in vocab.size():
		var key := String((vocab[i] as Dictionary).get("key", ""))
		if key == "" or key == "none":
			continue
		if app.bridge.civ_set_faction_field(assigned + 1, "religion", key):
			assigned += 1
		if assigned >= 5:
			break
	_p("seeded %d factions with a religion" % assigned)
	if assigned > 0:
		_p("civ_belief_run(50) -> %s" % [app.bridge.civ_belief_run(50)])
		await _frames(4)

	var rd = app.right_dock_ctrl
	places = app.bridge.settlements()
	_sel_name = String((places[0] as Dictionary).get("name", ""))
	## Timed around the call itself, not around a frame wait: six
	## `await process_frame`s are ~96 ms of vsync at 60 Hz and would make every
	## figure below a measurement of the refresh rate.
	var t0 := Time.get_ticks_usec()
	rd.on_settlement_selected(places[0], 0)
	var build_ms := float(Time.get_ticks_usec() - t0) / 1000.0
	await _frames(6)
	_p("settlement panel: on_settlement_selected() returned in %.1f ms" % build_ms)

	await _part_a()
	await _part_b(places)
	await _part_c()
	await _part_d()
	await _part_e()
	await _part_f()
	await _part_g(places)

	_p("=== %s ===" % ("ALL CHECKS PASSED" if _fail == 0 else "%d CHECK(S) FAILED" % _fail))
	get_tree().quit(0 if _fail == 0 else 1)

# == A. appended, not a replacement ==========================================
#
# The artboard's binding footnote: "the four sections above are appended, not a
# replacement -- the eight rows are §1.11 unchanged."

func _part_a() -> void:
	_p("-- A. appended, and the rows above untouched --")
	var name_pos := _pos(_sel_name)
	var faith := _pos("▾ FAITH")
	var layout := _pos("▾ CITY LAYOUT")
	var why := _pos("▾ WHY HERE")
	_check("FAITH is drawn", faith >= 0, "texts=%s" % [_texts().slice(0, 10)])
	_check("CITY LAYOUT is drawn", layout >= 0)
	_check("WHY HERE is drawn", why >= 0)
	_check("all three are APPENDED, below the selection's own rows",
		name_pos >= 0 and faith > name_pos, "name@%d faith@%d" % [name_pos, faith])
	_check("they are in the artboard's order: FAITH, CITY LAYOUT, WHY HERE",
		faith >= 0 and layout > faith and why > layout,
		"faith@%d layout@%d why@%d" % [faith, layout, why])
	## §1.11's own rows, by their label text, all still there.
	for row in ["Name", "Class", "Population", "Faction", "Coastal", "Capital",
			"Water access", "Defensibility", "Routes"]:
		_check("§1.11 row survives: %s" % row, _has_text(row))
	_check("the settlement's own name is still drawn", _has_text(_sel_name))
	## The chip row is the artboard's three; the City Viewer launcher moved into
	## CITY LAYOUT, so a fourth chip would be the duplicate this pass removed.
	for chip in ["Economy", "Politics", "Logistics"]:
		_check("action chip survives: %s" % chip, _has_text(chip))
	_check("the old fourth 'City layout' chip is gone -- one launcher, not two",
		not _has_text("City layout"))
	_check("the prose 'Why here?' section header is gone", not _has_text("§ WHY HERE?"))

# == B. FAITH ================================================================

func _part_b(places: Array) -> void:
	_p("-- B. FAITH --")
	var live: Dictionary = (app.bridge.settlements()[0] as Dictionary)
	_p("settlement 0 has religion=%s adherents=%s"
		% [live.has("religion"), live.has("adherents")])
	if not live.has("adherents"):
		_p("no adherents on this world -- running a diffusion so the section has data")
		## 50 years: `civilization_workspace.gd`'s own `_religion_years` default,
		## so this measures the same state CIVIL ▸ Religion would produce.
		_p("civ_belief_run(50) -> %s" % [app.bridge.civ_belief_run(50)])
		await _frames(6)
		app.right_dock_ctrl.on_settlement_selected(app.bridge.settlements()[0], 0)
		await _frames(6)
		live = app.bridge.settlements()[0]
	if not live.has("adherents"):
		_check("FAITH draws its absent state with a reason rather than an empty section",
			_pos("▾ FAITH") >= 0 and _has_text("—"), "no belief data on this world")
		return

	var pop := int(live.get("population", 0))
	var adherents: Dictionary = live["adherents"]
	var rows: Array = CivilizationWorkspace._religion_sorted(adherents)
	_p("engine: pop=%d faiths=%d plurality=%s" % [pop, rows.size(), live.get("religion", "?")])

	## The adherence bar, by the artboard's own 6 px literal.
	var bars := _panels_sized(0.0, ART_FAITH_BAR_H)
	_check("the stacked adherence bar is drawn at the artboard's 6 px", bars.size() >= 1,
		"found %d panels of min height %.0f" % [bars.size(), ART_FAITH_BAR_H])
	if not bars.is_empty():
		var segs: Array = []
		for c in (bars[0] as Panel).get_children():
			for cc in (c as Node).get_children():
				if cc is ColorRect:
					segs.append(cc)
		_check("the bar has one segment per faith the engine listed",
			segs.size() == rows.size(), "%d segments for %d faiths" % [segs.size(), rows.size()])
		var mismatched := 0
		for i in mini(segs.size(), rows.size()):
			var want: Color = CivilizationWorkspace._religion_color(String(rows[i][1]))
			if not (segs[i] as ColorRect).color.is_equal_approx(want):
				mismatched += 1
			if not is_equal_approx((segs[i] as ColorRect).size_flags_stretch_ratio,
					float(rows[i][0])):
				mismatched += 1
		_check("every segment carries its faith's own hue and its own head-count as the ratio",
			mismatched == 0, "%d mismatches" % mismatched)

	## The top three, and no fourth.
	var shown := 0
	for r in rows:
		if _has_text(CivilizationWorkspace._religion_label(String(r[1]))):
			shown += 1
	_check("at most three faiths are listed (the artboard's top-3 plus a remainder)",
		shown <= 3, "%d of %d listed" % [shown, rows.size()])
	if rows.size() > 3:
		_check("the remainder line names the faiths beyond the third",
			_has_text_containing("%d more" % (rows.size() - 3)),
			"texts=%s" % [_texts().slice(-8, -1)])
	## The plurality is the section head's trailing note, not a row of the eight.
	_check("the plurality is drawn as FAITH's trailing note",
		_has_text(CivilizationWorkspace._religion_label(String(live["religion"]))))
	_check("no 'Faith' row was left among §1.11's rows", not _has_text("Faith"))

func _has_text_containing(frag: String) -> bool:
	for t in _texts():
		if String(t).contains(frag):
			return true
	return false

# == C. CITY LAYOUT -- UM-03's thumbnail, its three rows, its launcher =======

func _part_c() -> void:
	_p("-- C. CITY LAYOUT --")
	var got: Array = app.bridge.urban_layouts(PackedInt32Array([0]))
	if got.is_empty():
		_check("CITY LAYOUT explains an absent plan instead of drawing an empty box",
			_pos("▾ CITY LAYOUT") >= 0)
		return
	var l: Dictionary = got[0]

	var thumbs := _panels_sized(ART_THUMB_PX, ART_THUMB_PX)
	_check("the plan thumbnail is drawn at the artboard's 84 x 84", thumbs.size() == 1,
		"found %d panels of min size %.0f x %.0f" % [thumbs.size(), ART_THUMB_PX, ART_THUMB_PX])

	## WARDS -- the engine's own district tags, counted here independently of
	## the dock's own loop.
	var wards: Dictionary = {}
	for d in (l.get("parcel_district", PackedStringArray()) as PackedStringArray):
		if String(d) != "":
			wards[String(d)] = true
	_p("engine: %d district tags %s | wall_spec=%s | bridge_pt present=%s"
		% [wards.size(), wards.keys(), l.get("wall_spec", "<absent>"), l.has("bridge_pt")])
	_check("WARDS prints the engine's own distinct district count",
		_has_text(str(wards.size())), "expecting '%d'" % wards.size())
	_check("WALLS prints um_wall_spec's rung verbatim",
		_has_text(String(l.get("wall_spec", ""))), "expecting '%s'" % l.get("wall_spec", ""))

	## BRIDGES: re-pinned 2026-09-24 (`ALIGNMENT_AUDIT.md` B14b). This used to
	## require a dash, on the reading that only `bridge_pt` reached the dock;
	## `urban_bridge.rs` has emitted the validated `"bridges"`/`"ford"` since
	## 2026-09-05, and the row now prints the count ("ford"/"none" when there
	## is no bridge). An absent `"bridges"` key (older library) still dashes.
	var bridges := _node_with_text("Bridges")
	_check("a BRIDGES row exists", bridges != null)
	if bridges != null:
		var value := _row_value(bridges)
		var expect := "—"
		if l.has("bridges"):
			var nb := (l.get("bridges", PackedVector2Array()) as PackedVector2Array).size()
			expect = str(nb) if nb > 0 else ("ford" if l.has("ford") else "none")
		_check("BRIDGES prints the layout's own crossing answer",
			value == expect, "drew '%s', expected '%s'" % [value, expect])

	_check("the City Viewer launcher is here, as the artboard's link",
		_has_text_containing("open viewer"))

## The value `Label` of a `_field()` row: the sibling after the label.
func _row_value(label_node: Node) -> String:
	var row := label_node.get_parent()
	if row == null:
		return ""
	var kids := row.get_children()
	for i in kids.size():
		if kids[i] == label_node and i + 1 < kids.size() and kids[i + 1] is Label:
			return (kids[i + 1] as Label).text
	return ""

# == D. the thumbnail actually draws a plan, in pixels =======================
#
# A node of the right size proves nothing about what is inside it. This reads
# the framebuffer back and counts distinct colours in the thumbnail's own rect
# against the same-sized rect of dock beside it -- the control that makes the
# number mean something.

func _part_d() -> void:
	_p("-- D. the thumbnail's pixels --")
	var thumbs := _panels_sized(ART_THUMB_PX, ART_THUMB_PX)
	if thumbs.is_empty():
		_check("a thumbnail to capture", false)
		return
	var frame := thumbs[0] as Control
	await _scroll_into_view(frame)
	var img := await _capture()
	if img == null:
		_check("the framebuffer came back", false, "get_image() returned null")
		return
	var r := Rect2i(frame.get_global_rect())
	if not Rect2i(Vector2i.ZERO, img.get_size()).encloses(r) or r.size.x <= 0:
		_check("the thumbnail is on screen to be captured", false,
			"rect=%s viewport=%s" % [r, img.get_size()])
		return
	## **A count of warm pixels, not of distinct colours.** The dock's own inks
	## are neutral greys, so `r - b` is ~0 across every panel, label and rule in
	## it; `urban_layout_draw.gd`'s whole palette is parchment and ink brown, so
	## every pixel of a drawn plan is warm. A patch of dock the same size is the
	## control that makes the number mean something -- and it is *text* rather
	## than flat background, which is the harder control: antialiased type has
	## hundreds of distinct colours and still no warm ones.
	var plan := _warm(img, r)
	var ctl_rect := Rect2i(r.position - Vector2i(0, r.size.y + 4), r.size)
	var ctl := _warm(img, ctl_rect) if Rect2i(Vector2i.ZERO, img.get_size()).encloses(ctl_rect) else -1
	var px := r.size.x * r.size.y
	_p("warm pixels: thumbnail=%d/%d  control patch=%d  (distinct colours %d vs %d)"
		% [plan, px, ctl, _distinct(img, r), _distinct(img, ctl_rect)])
	_check("the thumbnail draws a plan, not a flat box", plan > px / 2,
		"%d of %d pixels are warm" % [plan, px])
	_check("the same-sized patch of dock beside it is not, so that count is the plan",
		ctl >= 0 and ctl < px / 20, "control=%d of %d" % [ctl, px])

## The viewport's own framebuffer, one frame after the next draw.
func _capture() -> Image:
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()

## Walks up to the nearest `ScrollContainer` and brings `c` into view, so a
## capture of a section far down a long panel is not a capture of nothing.
func _scroll_into_view(c: Control) -> void:
	var n: Node = c.get_parent()
	while n != null and not (n is ScrollContainer):
		n = n.get_parent()
	if n != null:
		(n as ScrollContainer).ensure_control_visible(c)
	await _frames(4)

## Pixels whose red channel leads their blue by more than a hair: the plan's
## parchment and ink, against a shell whose every ink is neutral.
func _warm(img: Image, r: Rect2i) -> int:
	var n := 0
	for y in range(maxi(0, r.position.y), mini(img.get_height(), r.end.y)):
		for x in range(maxi(0, r.position.x), mini(img.get_width(), r.end.x)):
			var c := img.get_pixel(x, y)
			if c.r - c.b > 0.08:
				n += 1
	return n

func _distinct(img: Image, r: Rect2i) -> int:
	var seen: Dictionary = {}
	for y in range(maxi(0, r.position.y), mini(img.get_height(), r.end.y)):
		for x in range(maxi(0, r.position.x), mini(img.get_width(), r.end.x)):
			seen[img.get_pixel(x, y).to_rgba32()] = true
	return seen.size()

# == E. WHY HERE =============================================================

func _part_e() -> void:
	_p("-- E. WHY HERE --")
	var why: Dictionary = app.bridge.explain_settlement(0)
	if why.is_empty() or why.has("excluded"):
		_check("WHY HERE explains itself when there is no ranking to draw",
			_pos("▾ WHY HERE") >= 0)
		return
	var terms: Array = why["terms"]
	var ranked: Array = terms.duplicate()
	ranked.sort_custom(func(a, b): return _contrib(a) > _contrib(b))
	var expect: Array = []
	for t in ranked:
		if absf(float((t as Dictionary)["contribution"])) > 0.005:
			expect.append(t)
	_p("engine: %d terms, %d over the 0.005 threshold" % [terms.size(), expect.size()])
	_check("the trailing note counts the factors that actually moved the score",
		_has_text("%d factor%s" % [expect.size(), "" if expect.size() == 1 else "s"]),
		"expecting '%d factors'" % expect.size())

	## Each drawn bar's fill fraction, read off the node, must equal that
	## term's own `value` -- the engine's stated 0..1 axis, which is the
	## denominator this bar claims.
	var order_ok := true
	var frac_ok := true
	var prev := INF
	for t in expect:
		var d: Dictionary = t
		var key := String(d["key"])
		var text: String = RightDock.SUIT_TERM_LABELS.get(key, key.replace("_", " "))
		var node := _node_with_text(text)
		if node == null:
			_check("a bar is drawn for %s" % text, false)
			continue
		var c := float(d["contribution"])
		if c > prev:
			order_ok = false
		prev = c
		var wrap := node.get_parent().get_parent()
		var fill := _fill_of(wrap)
		if fill < 0.0 or not is_equal_approx(fill, clampf(float(d["value"]), 0.0, 1.0)):
			frac_ok = false
			_p("   %s: drew %.4f, value is %.4f" % [text, fill, float(d["value"])])
	_check("the rows are ranked by signed contribution", order_ok)
	_check("every bar's fill is that term's own 0..1 value", frac_ok)

	## A term that scored nothing is named, not silently dropped.
	var silent := terms.size() - expect.size()
	if silent > 0:
		_check("the %d terms that scored nothing are still named" % silent,
			_has_text_containing("Scored nothing here"))
	_check("the suitability score is still reported",
		_has_text_containing("Suitability %.2f" % float(why["score"])))
	_check("the four terrain readings the prose carried are still reported",
		_has_text_containing("travel cost"))

func _contrib(t: Variant) -> float:
	return float((t as Dictionary)["contribution"])

## `_micro_bar`'s fill `anchor_right`, found under a `_factor_row` wrapper.
func _fill_of(wrap: Node) -> float:
	if wrap == null:
		return -1.0
	for n in _nodes(wrap, []):
		if n is Panel and (n as Panel).custom_minimum_size.y == 4.0:
			for c in (n as Panel).get_children():
				if c is Panel:
					return (c as Panel).anchor_right
	return -1.0

# == F. Δ vertical ===========================================================

func _part_f() -> void:
	_p("-- F. Δ vertical --")
	var g: Vector2i = app.bridge.grid_size()
	var a := Vector2(g.x * 0.22, g.y * 0.28)
	var b := Vector2(g.x * 0.71, g.y * 0.74)
	app.arm_tool("measure")
	await _frames(3)
	GlobalTools.recall_measurement(app, "vertical", PackedVector2Array([a, b]))
	await _frames(8)

	var vm: Dictionary = app.bridge.measure_vertical(a.x, a.y, b.x, b.y)
	var prof: Dictionary = app.bridge.measure_section(a.x, a.y, b.x, b.y, SPEC_SAMPLES)
	var samples: Array = prof.get("samples", [])
	var stats: Dictionary = prof.get("stats", {})
	_p("engine: delta=%.0f m  samples=%d  min=%.0f max=%.0f ascent=%.0f descent=%.0f"
		% [float(vm.get("delta_m", 0.0)), samples.size(), float(stats.get("min_m", 0.0)),
			float(stats.get("max_m", 0.0)), float(stats.get("ascent_m", 0.0)),
			float(stats.get("descent_m", 0.0))])

	_check("the engine returns 121 samples for n=121, not 120 (§5.7 defect 2)",
		samples.size() == SPEC_SAMPLES, "got %d" % samples.size())
	_check("the hero reads the vertical difference",
		_has_text("%+.0f m" % float(vm["delta_m"])), "texts=%s" % [_texts().slice(0, 8)])

	## §5.7's own geometry, pinned against the SPEC's literals typed above --
	## never against the constants that produced them. Mutating any of these in
	## `right_dock.gd` turns this red.
	_check("§5.7's viewBox height is 130", RightDock.PROFILE_VIEWBOX_H == SPEC_VIEWBOX_H,
		"code says %s" % RightDock.PROFILE_VIEWBOX_H)
	_check("§5.7's two gridlines are at y=43 and y=86",
		Array(RightDock.PROFILE_GRID_Y) == SPEC_GRID_Y,
		"code says %s" % [RightDock.PROFILE_GRID_Y])
	_check("§5.7's trace baseline is 126 and its span 118",
		RightDock.PROFILE_TRACE_BASE == 126.0 and RightDock.PROFILE_TRACE_SPAN == 118.0,
		"code says base=%s span=%s"
			% [RightDock.PROFILE_TRACE_BASE, RightDock.PROFILE_TRACE_SPAN])
	_check("the artboard's chart column is 118 + a 14 px axis strip = 132",
		float(RightDock.PROFILE_H + RightDock.PROFILE_AXIS_H) == ART_CHART_H
		and float(RightDock.PROFILE_AXIS_H) == ART_AXIS_H,
		"code says %d + %d" % [RightDock.PROFILE_H, RightDock.PROFILE_AXIS_H])
	_check("the artboard's metre column is 40 px at dock width",
		float(RightDock.PROFILE_Y_COL) == ART_Y_COL,
		"code says %d" % RightDock.PROFILE_Y_COL)
	_check("the dock asks the engine for 121 samples",
		RightDock.PROFILE_SAMPLES == SPEC_SAMPLES,
		"code says %d" % RightDock.PROFILE_SAMPLES)
	_check("the artboard's plan box is 84 px", float(RightDock.PLAN_THUMB_PX) == ART_THUMB_PX,
		"code says %d" % RightDock.PLAN_THUMB_PX)

	## The chart box, by the artboard's own 132 = 118 + 14, read off the node.
	var chart: Control = null
	for n in _nodes():
		if n.get_class() == "Control" and (n as Control).custom_minimum_size.y == ART_CHART_H:
			chart = n
			break
	_check("the profile chart is drawn at the artboard's 132 px column", chart != null,
		"found a Control of min height %.0f" % ART_CHART_H if chart != null
		else "no Control of min height %.0f" % ART_CHART_H)
	if chart != null:
		_check("its width is the dock's, not a fixed viewport figure",
			chart.size.x > ART_Y_COL, "%.0f px wide" % chart.size.x)
		## And it draws a trace rather than an empty box. Same warm-pixel
		## measure as part D and for the same reason: the fill is
		## `accent_wash` and the line `accent`, both amber, in a dock whose
		## every other ink is neutral. The control is a same-sized patch of the
		## rows above -- text, which has hundreds of distinct colours and no
		## warm ones.
		await _scroll_into_view(chart)
		var img := await _capture()
		var cr := Rect2i(chart.get_global_rect())
		var bounds := Rect2i(Vector2i.ZERO, img.get_size()) if img != null else Rect2i()
		## The patch BELOW the chart, not above it: above sits the hero, which
		## is `accent` and therefore warm by design, and a control that shares
		## the property under test is not a control. Below are `_field` rows in
		## `text`/`text_dim`, both neutral.
		var ctl_rect := Rect2i(cr.position + Vector2i(0, cr.size.y + 6), cr.size)
		if img != null and bounds.encloses(cr):
			var trace := _warm(img, cr)
			var flat := _warm(img, ctl_rect) if bounds.encloses(ctl_rect) else -1
			_p("chart rect %s: warm pixels trace=%d control=%d (of %d)"
				% [cr, trace, flat, cr.size.x * cr.size.y])
			_check("the chart draws a profile, not an empty box", trace > 200,
				"%d warm pixels" % trace)
			_check("the rows above it are not warm, so that count measures the trace",
				flat >= 0 and flat * 4 < trace, "control=%d trace=%d" % [flat, trace])
		else:
			_check("the chart is on screen to be captured", false, "rect=%s" % cr)

	_check("SAMPLES prints what came back, and it is 121",
		_has_text_containing("%d · 1 per" % samples.size()),
		"expecting '%d · 1 per ...'" % samples.size())
	_check("no '×4 exaggeration' claim is reintroduced (§5.7 defect 1)",
		not _has_text_containing("exagger"))
	_check("no '120 samples' claim is reintroduced (§5.7 defect 2)",
		not _has_text_containing("120 samples"))

	for row in ["High point", "Low point", "Total climb", "Total descent",
			"Mean grade", "Samples"]:
		_check("Δ vertical row: %s" % row, _has_text(row))
	var climb := _node_with_text("Total climb")
	if climb != null:
		## The drawn value is thousands-grouped, so the comparison strips the
		## separators rather than looking for a bare integer inside it.
		var drew := _row_value(climb).replace(",", "").replace(" ", "")
		_check("TOTAL CLIMB is the profile's ascent, not the endpoint difference",
			drew.contains("%d" % int(round(float(stats["ascent_m"])))),
			"drew '%s', ascent is %.0f" % [_row_value(climb), float(stats["ascent_m"])])
	var desc := _node_with_text("Total descent")
	if desc != null:
		var drew_d := _row_value(desc).replace(",", "").replace(" ", "")
		_check("TOTAL DESCENT is the profile's descent, already signed by the engine",
			drew_d.contains("%d" % int(round(float(stats["descent_m"])))),
			"drew '%s', descent is %.0f" % [_row_value(desc), float(stats["descent_m"])])
	## The endpoint difference and the profile's climb must be different numbers
	## on this line, or the check above cannot discriminate.
	_check("this line discriminates: ascent %.0f != |delta| %.0f"
		% [float(stats["ascent_m"]), absf(float(vm["delta_m"]))],
		absf(float(stats["ascent_m"]) - absf(float(vm["delta_m"]))) > 1.0)
	var hi := _node_with_text("High point")
	if hi != null:
		_check("HIGH POINT carries the max AND how far along it stands",
			_row_value(hi).contains("·"), "drew '%s'" % _row_value(hi))
	_check("the 2D/3D note the owner's ruling stands on survives",
		_has_text_containing("stays live in both"))

# == G. HISTORY's duplicate header, and the dock's width across three seeds ==

func _part_g(places: Array) -> void:
	_p("-- G. HISTORY, and dock width --")
	app.right_dock_ctrl.show_history()
	await _frames(6)
	_check("the N STEPS row is still drawn", _has_text("STEPS"),
		"texts=%s" % [_texts().slice(0, 8)])
	_check("the duplicate '§ STEPS' section header above it is gone",
		not _has_text("§ STEPS"), "texts=%s" % [_texts().slice(0, 8)])
	var hdrs: Array = []
	for t in _texts():
		if String(t).begins_with("§ "):
			hdrs.append(t)
	_p("HISTORY section headers now: %s" % [hdrs])

	## MISTAKES.md: one world is one sample, and panel widths are
	## content-dependent. Three seeds, and the figure that matters is the
	## SETTLEMENT panel with all three new sections open.
	var dock: Control = app.right_dock_body
	for seed_v in SEEDS:
		## Bracketed so an engine error printed on stderr can be attributed to
		## the generate or to the panel build, and not guessed at.
		_p("[seed %d] generate BEGIN" % seed_v)
		if not await _gen(seed_v):
			_p("seed %d: generate failed" % seed_v)
			continue
		await _frames(4)
		_p("[seed %d] generate END / panel build BEGIN" % seed_v)
		var ps: Array = app.bridge.settlements()
		if ps.is_empty():
			_p("seed %d: no settlement" % seed_v)
			continue
		## Median of five builds of the same panel, not one sample: this figure
		## is the whole cost of `urban_layouts()` plus `measure_section()` plus
		## the tree, and a single sample of it would be a device reading.
		var runs: Array = []
		for i in 5:
			var t0 := Time.get_ticks_usec()
			app.right_dock_ctrl.on_settlement_selected(ps[0], 0)
			runs.append(float(Time.get_ticks_usec() - t0) / 1000.0)
			await _frames(2)
		runs.sort()
		await _frames(4)
		_p("[seed %d] panel build END" % seed_v)
		var w: float = dock.get_combined_minimum_size().x
		_p("seed %6d: settlement panel min.x=%.0f px (dock %d)  build %.1f ms (%.1f..%.1f)"
			% [seed_v, w, DccTheme.W_RIGHT_DOCK, runs[2], runs[0], runs[4]])
		_check("seed %d: the panel does not force the dock past its own maximum" % seed_v,
			w <= float(DccTheme.W_RIGHT_DOCK_MAX),
			"min.x=%.0f vs W_RIGHT_DOCK_MAX %d" % [w, DccTheme.W_RIGHT_DOCK_MAX])

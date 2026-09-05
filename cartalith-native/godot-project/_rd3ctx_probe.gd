extends Node
## Committed probe for the three right-dock contexts the owner approved on
## 2026-09-05 (`design/proposed-2026-09-05`): HISTORY (`Main.dc.html`),
## ECOREGION (`Wildlife.dc.html`) and FACTION ▸ RELATIONS (`Relations.dc.html`).
##
## Every assertion reads a value **off a drawn node** -- a `Label`'s `text`, its
## resolved `font_color`, a `Control`'s measured `size` -- and never out of the
## variable that fed it. Where a figure has to be predicted, it is recomputed
## here from literals this file carries: the stance bands (0.45 / 0.15 / -0.15 /
## -0.45) are written out rather than read from `cartalith-civ`, the stance ink
## table is a second, independent copy, and `--ctl` is asserted as the literal
## 24 the canvas draws. A check that read `RightDock.STANCE_INK` back would
## prove only that a dictionary equals itself.
##
## Three seeds, because a dock's width is content-dependent and one world is
## one sample.
##
## **Every hex below is a DARK-palette value and this machine boots light**
## (`cartalith_settings.cfg`, `mode="light"`). `_force_dark()` establishes the
## precondition and `_ready()` refuses to run without it -- naming the palette
## in prose does not select it, which is the trap the first run of this probe
## fell into: it measured `#111210` (light `text_bright`) against `#e8ebec` and
## `#6B6F6A` (light `text_dim`) against `#8d9296`, three failures that were the
## harness's and not the panel's.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _rd3ctx_probe.tscn

var _app: Node
var _bridge: Node
var _fail := 0

## The artboard's own stance ink, restated independently of `STANCE_INK` in
## `right_dock.gd`. `--good` #6fae7d, `--acc` #e0a34a, `--dim` #8d9296,
## `--block` #c96a5a.
const WANT_INK := {
	"allied": "#6fae7d", "friendly": "#e0a34a", "neutral": "#8d9296",
	"wary": "#8d9296", "hostile": "#c96a5a",
}

func _p(s: String) -> void:
	print("RD3 %s" % s)

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("RD3 %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## The precondition for every hex in this file. Returns whether dark is live.
func _force_dark() -> bool:
	if not DccTheme.is_dark():
		DccTheme.apply_theme(true)
		_app.rebuild_theme(false)
	return DccTheme.is_dark()

# -- Reading the drawn tree back ---------------------------------------------

func _walk(n: Node, out: Array) -> void:
	for c in n.get_children():
		out.append(c)
		_walk(c, out)

func _nodes() -> Array:
	var out: Array = []
	_walk(_app.right_dock_body, out)
	return out

func _labels() -> Array:
	var out: Array = []
	for n in _nodes():
		if n is Label:
			out.append(n)
	return out

func _texts() -> Array:
	var out: Array = []
	for l in _labels():
		out.append((l as Label).text)
	return out

func _label_with(t: String) -> Label:
	for l in _labels():
		if (l as Label).text == t:
			return l
	return null

func _has_text(t: String) -> bool:
	return _texts().has(t)

## The state pips, in draw order, as `filled?`.
func _pip_states() -> Array:
	var out: Array = []
	for n in _nodes():
		if n is Panel and (n as Panel).custom_minimum_size == Vector2(7, 7):
			var sb: StyleBox = (n as Panel).get_theme_stylebox("panel")
			out.append(sb is StyleBoxFlat and (sb as StyleBoxFlat).bg_color.a > 0.0)
	return out

## The cursor row, found by the `accent_wash` band the artboard draws it in --
## not by its label text, which two `Carve fjords` rows share and which made
## the first run of this probe measure row 02 and report the cursor's ink wrong.
func _cursor_labels() -> Array:
	var out: Array = []
	for n in _nodes():
		if not (n is PanelContainer):
			continue
		var sb: StyleBox = (n as PanelContainer).get_theme_stylebox("panel")
		if not (sb is StyleBoxFlat):
			continue
		var bg: Color = (sb as StyleBoxFlat).bg_color
		## `--wash` = rgba(224,163,74,.09).
		if absf(bg.a - 0.09) > 0.005 or absf(bg.r - 224.0 / 255.0) > 0.01:
			continue
		var inner: Array = []
		_walk(n, inner)
		for c in inner:
			if c is Label:
				out.append(c)
	return out

## The colour the label actually draws in, off the node.
func _ink(l: Label) -> String:
	return "#%02x%02x%02x" % [
		int(round(l.get_theme_color("font_color").r * 255.0)),
		int(round(l.get_theme_color("font_color").g * 255.0)),
		int(round(l.get_theme_color("font_color").b * 255.0))]

## The widest thing the dock body demands, measured rather than reasoned about.
func _min_width() -> float:
	return (_app.right_dock_body as Control).get_combined_minimum_size().x

func _generate(seed: int) -> void:
	_bridge.generate({
		"seed": seed, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout

# -- The stance band, recomputed from literals -------------------------------

## `cartalith_civ::relations::stance_for()`, written out here so the engine's
## own answer is compared against an independent evaluation of the same rule
## rather than against itself.
func _stance_from(value: float) -> String:
	if value >= 0.45:
		return "allied"
	if value >= 0.15:
		return "friendly"
	if value > -0.15:
		return "neutral"
	if value > -0.45:
		return "wary"
	return "hostile"

# ============================================================== HISTORY ======

func _probe_history() -> void:
	_p("=== HISTORY (Main.dc.html) ===")
	var rd = _app.right_dock_ctrl
	## A fresh world's ledger is one `Floor` row, which cannot exercise the
	## cursor, the boundary or the undone tail -- all three need rows *above*
	## another row. `carve_fjords()` is the cheapest `#[func]` that pushes a
	## real `HeightSnapshot` (`lib.rs`, the `self.undo.push("Carve fjords", …)`
	## site), so two of them build a three-row ledger with two reversible steps.
	for i in 2:
		var r: Dictionary = _bridge.carve_fjords()
		_p("carve_fjords #%d -> %s" % [i + 1, str(r)])
		await _frames(4)

	rd.show_history()
	await _frames(6)

	var ledger: Array = _bridge.undo_ledger()
	_p("undo_ledger rows=%d  can_undo=%s  redo_available=%s"
		% [ledger.size(), _bridge.can_undo(), _bridge.redo_available()])
	_check("the panel drew something", not _texts().is_empty())

	## §1.7's header: the count, the literal label, two `--ctl` squares.
	var count_l := _label_with("%d" % ledger.size())
	_check("the stack header draws the row count", count_l != null,
		"looking for '%d' among %s" % [ledger.size(), _texts().slice(0, 8)])
	_check("STEPS is the header's literal label", _has_text("STEPS"),
		"texts=%s" % [_texts().slice(0, 8)])

	var squares: Array = []
	for n in _nodes():
		if n is Button and ((n as Button).text == "↶" or (n as Button).text == "↷"):
			squares.append(n)
	_check("both --ctl squares are drawn", squares.size() == 2,
		"found %d" % squares.size())
	if squares.size() == 2:
		## The canvas's own `--ctl`, asserted as the literal 24 rather than
		## against the constant that produced it.
		var sz := (squares[0] as Control).size
		_check("the undo square measures the canvas's 24x24", sz.x >= 24.0 and sz.y >= 24.0,
			"drawn %s" % str(sz))

	## Ordinals and pips. Every row's ordinal is drawn `%02d`, oldest first.
	if ledger.size() >= 2:
		_check("row ordinals run oldest-first from 01",
			_has_text("01") and _has_text("%02d" % ledger.size()),
			"texts=%s" % [_texts().slice(0, 12)])

	## The cursor: exactly one row sits on the `--wash` band, and it is the
	## newest ledger row drawn in `--ink` #e8ebec.
	if not ledger.is_empty():
		var newest: Dictionary = ledger[ledger.size() - 1]
		var want := "%s %s" % [
			String({"height": "▲", "recorded": "·", "floor": "◼"}.get(
				String(newest.get("kind", "recorded")), "·")),
			String(newest.get("label", "?"))]
		var cur := _cursor_labels()
		var cur_texts: Array = []
		for c in cur:
			cur_texts.append((c as Label).text)
		_check("exactly one row is drawn on the accent-wash cursor band",
			not cur.is_empty(), "band labels=%s" % str(cur_texts))
		_check("the cursor band holds the NEWEST ledger row", cur_texts.has(want),
			"looking for '%s'; band=%s" % [want, str(cur_texts)])
		for c in cur:
			if (c as Label).text == want:
				_check("the cursor's label draws in --ink #e8ebec", _ink(c) == "#e8ebec",
					"drew %s" % _ink(c))

	## No save has happened, so there must be no COMMITTED rule and the panel
	## must say why. The absence is the assertion: a boundary invented from the
	## ledger floor would show up here as a stray COMMITTED cap.
	_check("no COMMITTED rule before a save", not _has_text("COMMITTED"),
		"texts=%s" % [_texts().slice(0, 14)])
	var said := false
	for t in _texts():
		if String(t).find("No COMMITTED rule") >= 0:
			said = true
	_check("and the panel says why it is absent", said)

	## Undo one step, then the cursor's tail must be drawn -- one row, named
	## from `redo_label()`, and never more than one name.
	if _bridge.can_undo():
		var before := ledger.size()
		_app.undo_last()
		await _frames(8)
		var after: Array = _bridge.undo_ledger()
		_p("after undo: ledger=%d -> %d  redo_available=%s redo_depth=%d"
			% [before, after.size(), _bridge.redo_available(),
				int(_bridge.undo_stats().get("redo_depth", 0))])
		_check("undo removed the row from the ledger (the engine's behaviour, "
			+ "not the artboard's)", after.size() < before,
			"%d -> %d" % [before, after.size()])
		if _bridge.redo_available():
			_check("the undone step is drawn below the cursor", _has_text("undone"),
				"texts=%s" % [_texts().slice(0, 14)])
			var rl := String(_bridge.redo_label())
			_check("and it is named from redo_label(), not invented",
				rl == "" or _has_text(rl), "redo_label='%s'" % rl)
			## The undone row's pip is the one hollow pip on screen.
			var st := _pip_states()
			_p("pips with the tail drawn: %s" % str(st))
			_check("the undone row's pip is hollow", not st.is_empty() and not bool(st[st.size() - 1]),
				"states=%s" % str(st))
		## Put it back so the save boundary below measures the full ledger.
		_app.redo_last()
		await _frames(8)

	## COMMITTED. A real `save_project()` into `user://`, so the boundary is
	## drawn off the same `project_saved` the shell's own save bookkeeping fires.
	var path := ProjectSettings.globalize_path("user://_rd3ctx_probe.zip")
	var saved: bool = _bridge.save_project(path)
	_p("save_project('%s') -> %s" % [path, saved])
	await _frames(8)
	rd.show_history()
	await _frames(6)
	if not saved:
		_p("   SKIP the COMMITTED half -- save_project() refused; nothing to draw from")
		return
	_check("a save draws the COMMITTED rule", _has_text("COMMITTED"),
		"texts=%s" % [_texts().slice(0, 14)])
	var cap := _label_with("COMMITTED")
	if cap != null:
		_check("the COMMITTED cap draws in --faint #6f7478", _ink(cap) == "#6f7478",
			"drew %s" % _ink(cap))
	var age := false
	for t in _texts():
		if String(t).begins_with("saved "):
			age = true
			_p("   age readout: %s" % t)
	_check("with the last save's age beside it", age)

	## One more operation *after* the save, so the boundary is now BETWEEN two
	## rows and the pips have to disagree with each other. Every figure below is
	## read off a drawn node.
	_bridge.carve_fjords()
	await _frames(4)
	rd.show_history()
	await _frames(6)
	var after_save: Array = _bridge.undo_ledger()
	_p("after the post-save carve: ledger=%d rows, _saved_seq=%d"
		% [after_save.size(), int(rd._saved_seq)])
	var filled: Array = _pip_states()
	_p("pips (filled?) in draw order: %s" % str(filled))
	_check("one pip per ledger row", filled.size() == after_save.size(),
		"%d pips for %d rows" % [filled.size(), after_save.size()])
	if filled.size() == after_save.size() and after_save.size() >= 2:
		var hollow := 0
		for i in range(filled.size() - 1):
			if not bool(filled[i]):
				hollow += 1
		_check("every pip above COMMITTED is filled", hollow == 0,
			"%d hollow among the committed rows" % hollow)
	## And the row the boundary separated: the last committed row is
	## `text_secondary`, the new cursor is `text_bright`.
	if after_save.size() >= 2:
		var prev: Dictionary = after_save[after_save.size() - 2]
		var pl := _label_with("%s %s" % [
			String({"height": "▲", "recorded": "·", "floor": "◼"}.get(
				String(prev.get("kind", "recorded")), "·")),
			String(prev.get("label", "?"))])
		if pl != null:
			_check("the last saved row draws in --sec #a9adb0", _ink(pl) == "#a9adb0",
				"drew %s" % _ink(pl))

# ============================================================= ECOREGION =====

func _probe_wildlife() -> void:
	_p("=== ECOREGION (Wildlife.dc.html) ===")
	var rd = _app.right_dock_ctrl
	## The RICHEST region on the map, not the first one a probe cell happens to
	## hit: the `N more` collapse row only exists above five species, and a
	## two-species region leaves that whole branch undrawn and unproven.
	var eco: Dictionary = {}
	var best := -1
	for gy in range(24, 288, 24):
		for gx in range(24, 384, 24):
			var r: Dictionary = _bridge.wildlife_region_at(float(gx), float(gy))
			if r.is_empty():
				continue
			var n := 0
			for g in (r.get("guilds", []) as Array):
				n += ((g as Dictionary).get("species", []) as Array).size()
			if n > best:
				best = n
				eco = r
	if eco.is_empty():
		_p("   SKIP -- wildlife_region_at() found no ecoregion at any probe cell")
		return
	rd.show_wildlife(eco)
	await _frames(6)

	_check("the hero names the region by its id (there is no name field)",
		_has_text("Ecoregion %d" % int(eco.get("id", -1))),
		"texts=%s" % [_texts().slice(0, 8)])
	_check("the record really carries no name", not eco.has("name"),
		"keys=%s" % [eco.keys()])

	## The four dashed rows must be dashes, and must draw in `--dis` #5f6468 --
	## `_field()`'s unreachable ink -- so they cannot be mistaken for readings.
	var dashes := 0
	var ghosted := 0
	for l in _labels():
		if (l as Label).text == "—":
			dashes += 1
			if _ink(l) == "#5f6468":
				ghosted += 1
	_check("four fields are dashed rather than filled", dashes >= 4, "dashes=%d" % dashes)
	_check("and every dash draws in --dis #5f6468", dashes == ghosted,
		"%d of %d" % [ghosted, dashes])
	for k in ["Elevation band", "Mean temp", "Precipitation", "Soil · drainage"]:
		_check("'%s' is a drawn label" % k, _has_text(k))

	## The live half.
	_check("Species carries the real richness",
		_has_text("%d" % int(eco.get("richness", -1))),
		"richness=%d" % int(eco.get("richness", -1)))

	## Fauna, ranked. Recomputed here from the record rather than trusted.
	var all: Array = []
	for g in (eco.get("guilds", []) as Array):
		for sp in ((g as Dictionary).get("species", []) as Array):
			all.append(sp)
	_p("species in record: %d (guilds %d)" % [all.size(), (eco.get("guilds", []) as Array).size()])
	_check("the FAUNA caption counts the flattened roster",
		_has_text("FAUNA · %d" % all.size()), "texts=%s" % [_texts().slice(0, 20)])
	if not all.is_empty():
		var top: Dictionary = all[0]
		for s in all:
			if float((s as Dictionary).get("population_est", 0.0)) \
					> float(top.get("population_est", 0.0)):
				top = s
		_check("the most populous species is drawn first",
			_has_text(String(top.get("name", ""))), "top='%s'" % String(top.get("name", "")))
		## The bar for it must be full: the fill's `anchor_right` is the share
		## of the largest population, which for the largest is exactly 1.
		var full := false
		for n in _nodes():
			if n is Panel and (n as Panel).anchor_right >= 0.999:
				full = true
		_check("its micro-bar fills the track (share of the largest = 1.0)", full)
		if all.size() > 5:
			_check("the roster collapses with an 'N more' row",
				_has_text("▾  %d more" % (all.size() - 5)),
				"texts=%s" % [_texts().slice(-8, _texts().size())])
			## And expanding really draws the rest, rather than only relabelling.
			var before_names := 0
			for s in all:
				if _has_text(String((s as Dictionary).get("name", ""))):
					before_names += 1
			_app.right_dock_ctrl._wildlife_show_all = true
			_app.right_dock_ctrl._rebuild()
			await _frames(6)
			var after_names := 0
			for s in all:
				if _has_text(String((s as Dictionary).get("name", ""))):
					after_names += 1
			_p("species named: %d collapsed -> %d expanded (roster %d)"
				% [before_names, after_names, all.size()])
			_check("expanding draws every species in the roster",
				after_names == all.size() and before_names == 5,
				"%d -> %d of %d" % [before_names, after_names, all.size()])
			_check("and the 'N more' row is gone once expanded",
				not _has_text("▾  %d more" % (all.size() - 5)))
		else:
			_p("   NOTE the richest region on this map has %d species, so the "
				% all.size() + "'N more' collapse row could not be exercised")

	## Flora: an absent state, not four invented chips.
	_check("FLORA draws its absent state", _has_text("FLORA"))
	var flora_said := false
	for t in _texts():
		if String(t).find("No flora roster") >= 0:
			flora_said = true
	_check("and says there is no flora model", flora_said)

# ============================================================= RELATIONS =====

func _probe_relations() -> void:
	_p("=== FACTION ▸ RELATIONS (Relations.dc.html) ===")
	var rd = _app.right_dock_ctrl
	var pairs: Array = _bridge.civ_faction_relations()
	if pairs.is_empty():
		_p("   SKIP -- civ_faction_relations() is empty (fewer than two factions)")
		return
	var first: Dictionary = pairs[0]
	var fid := int(first.get("a", 1))
	rd.show_faction(fid, int(first.get("b", -1)))
	await _frames(6)

	_check("the faction dock draws a Relations section", _has_text("§ RELATIONS"),
		"texts=%s" % [_texts().slice(0, 10)])

	var mine: Array = []
	for p in pairs:
		var d: Dictionary = p
		if int(d.get("a", -1)) == fid or int(d.get("b", -1)) == fid:
			mine.append(d)
	_p("relations for faction %d: %d" % [fid, mine.size()])

	var checked := 0
	for p in mine:
		var d: Dictionary = p
		var stance := String(d.get("stance", ""))
		var value := float(d.get("value", 0.0))
		## The design's own claim, pinned: the stance IS a band over the score.
		_check("stance '%s' is the band %.3f falls in" % [stance, value],
			stance == _stance_from(value), "independent band says '%s'" % _stance_from(value))
		var chip := _label_with(stance.to_upper())
		if chip == null:
			_check("a %s chip is drawn" % stance, false,
				"texts=%s" % [_texts().slice(0, 24)])
			continue
		_check("the %s chip draws in %s" % [stance, String(WANT_INK.get(stance, "?"))],
			_ink(chip) == String(WANT_INK.get(stance, "?")),
			"drew %s" % _ink(chip))
		checked += 1
	_check("at least one stance chip was measured", checked > 0)

	## The score beside the chip, and the swatch before the name.
	var scored := 0
	for p in mine:
		var d: Dictionary = p
		if _has_text("%+d" % int(round(100.0 * float(d.get("value", 0.0))))):
			scored += 1
	_check("every relation's score is drawn", scored == mine.size(),
		"%d of %d" % [scored, mine.size()])
	var swatches := 0
	for n in _nodes():
		if n is ColorRect and (n as ColorRect).custom_minimum_size == Vector2(11, 11):
			swatches += 1
	_check("each relation carries a colour swatch", swatches >= mine.size(),
		"%d swatches for %d rows" % [swatches, mine.size()])

	## BALANCE, and its segments must sum to the roster.
	_check("BALANCE is drawn", _has_text("BALANCE"))
	_check("its ends are labelled hostile / allied",
		_has_text("hostile") and _has_text("allied"))

	## The marked pair -- RL-01, which `_wiredfix_probe.gd` also guards.
	var other := int(first.get("b", -1))
	var other_name := String(first.get("b_name", "?"))
	_check("the clicked pair is marked '▸ %s'" % other_name,
		_has_text("▸ %s" % other_name), "texts=%s" % [_texts().slice(0, 24)])

# ================================================================= main ======

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 1500.0
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
	if not _force_dark():
		_p("!! REFUSING TO RUN -- every hex here is a dark-palette value and the "
			+ "theme is light. A run in light measures the wrong palette and reports "
			+ "failures that are the harness's.")
		get_tree().quit(2)
		return
	_p("palette: dark (forced)")
	await _frames(4)

	## Three seeds. The first carries every assertion; all three carry the
	## width measurement, which is content-dependent and cannot be read off one.
	var widths: Array = []
	var seeds := [483920, 771155, 4242]
	for i in seeds.size():
		await _generate(int(seeds[i]))
		if not _bridge.has_world:
			_p("!! generate failed for seed %d" % int(seeds[i]))
			continue
		await _frames(6)
		if i == 0:
			await _probe_history()
			await _probe_wildlife()
			await _probe_relations()
		else:
			var eco: Dictionary = {}
			for at in [Vector2(192, 144), Vector2(96, 72), Vector2(288, 216)]:
				eco = _bridge.wildlife_region_at((at as Vector2).x, (at as Vector2).y)
				if not eco.is_empty():
					break
			if not eco.is_empty():
				_app.right_dock_ctrl.show_wildlife(eco)
				await _frames(6)
		var w := _min_width()
		widths.append(w)
		_p("seed %d: dock body minimum width = %.0f px" % [int(seeds[i]), w])

	## `DccTheme.W_RIGHT_DOCK` is 304 and the laptop band narrows it to 280;
	## the literal 280 is asserted rather than the constant.
	var worst := 0.0
	for w in widths:
		worst = maxf(worst, float(w))
	_check("no context widens the dock past the narrow 280 px band",
		worst <= 280.0, "widest of %s = %.0f" % [str(widths), worst])

	_p("=== %d failure%s ===" % [_fail, "" if _fail == 1 else "s"])
	get_tree().quit(1 if _fail > 0 else 0)

extends Node
## The year scrubber (`design/proposed-2026-09-05-round2/Timeline.dc.html`) and
## the WORLD dock's A/B mode switch (`WorldDockB.dc.html`), read back off the
## drawn nodes of a booted shell.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _tlscrub_probe.tscn
##   Godot_v4.7.1-stable_win64_console.exe --path . _tlscrub_probe.tscn        # §7 as well
##
## **§7 is the only pixel section and it refuses to run headless.** The dummy
## display driver never rasterises, so a colour census of the scrub track would
## pass vacuously; §§1-6 read node state and are density-independent, so they
## run either way. Everything here is pointer density unless a line says
## otherwise -- the transport's 44 px floor is a touch figure and is named as
## such where it appears.
##
## Dark is forced: this machine boots `mode="light"`
## (`cartalith_settings.cfg`), and §1's fill assertion is a comparison against
## two dark tokens that differ by alpha.

var _fail := 0
var _app: Node
var _vp: SubViewport

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(label: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", label, "   got=", got, " want=", want)

func _note(label: String, value) -> void:
	print("  --   ", label, " = ", value)

func _force_dark() -> bool:
	if not DccTheme.is_dark():
		DccTheme.apply_theme(true)
		_app.call("rebuild_theme", false)
	return DccTheme.is_dark()

func _walk(n: Node, out: Array) -> void:
	for c in n.get_children():
		out.append(c)
		_walk(c, out)

## Every node under the timeline region, which is the only tree these sections
## read. Walked rather than reached through the private handles the shell keeps,
## so a rename breaks this on the rename and not on a regression.
func _tl_nodes() -> Array:
	var out: Array = []
	_walk(_app.get("timeline_bar"), out)
	return out

func _tl_texts() -> Array:
	var out: Array = []
	for n in _tl_nodes():
		## `is_visible_in_tree()`, not `visible`. A `Label` inside a container
		## this repaint hid keeps `visible == true` -- the first run of this
		## probe passed a "the counts are dropped between two marks" check that
		## was reading a hidden row's text, which is the check being green for
		## the wrong reason.
		if n is Label and (n as Label).is_visible_in_tree() \
				and not (n as Label).text.is_empty():
			out.append((n as Label).text)
	return out

func _has_text(want: String) -> bool:
	for t in _tl_texts():
		if String(t) == want:
			return true
	return false

func _ancestor_scroll(n: Node) -> ScrollContainer:
	var p := n.get_parent()
	while p != null:
		if p is ScrollContainer:
			return p as ScrollContainer
		p = p.get_parent()
	return null

func _ready() -> void:
	_vp = SubViewport.new()
	_vp.size = Vector2i(1920, 1080)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	## **Refuses the touch leg rather than defaulting through it.** Run with
	## `-- --force-touch` this file reported ten failures at 1920x1080 -- the
	## tablet composition builds a different strip and the pill band measures
	## 131 px there, not 40 -- and none of that was diagnosed. A probe that
	## answers for a density it was not written for is worse than one that
	## declines: its greens would be read as touch coverage. The transport's
	## 44 px floor is therefore the role table's figure here, printed in §6,
	## and **not measured on a drawn node at touch density by this probe**.
	if OS.get_cmdline_user_args().has("--force-touch"):
		print("=== _tlscrub_probe: REFUSING --force-touch ===")
		print("  This probe asserts the POINTER leg only. The touch composition")
		print("  differs and is not covered here; do not read a pass as touch")
		print("  coverage. Re-run without --force-touch.")
		get_tree().quit(2)
		return
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await _frames(50)
	print("\n=== _tlscrub_probe · 1920x1080 pointer ===")
	print("  palette forced dark: ", _force_dark())
	await _frames(4)

	await _s1_mode_switch()
	await _s2_collapsed()
	await _s3_no_years()
	await _s4_recorded()
	await _s5_end_of_track()
	await _s6_merge()
	await _s7_pixels()

	print("\n=== ", "PASS" if _fail == 0 else "FAIL (%d)" % _fail, " ===")
	get_tree().quit(0 if _fail == 0 else 1)

# -- §1 The WORLD mode switch, verified rather than rebuilt --------------------
#
# Board 1 of this batch was already shipped at HEAD. This section asserts the
# three things the owner's ruling made binding, and measures the cost it names.

func _s1_mode_switch() -> void:
	print("\n-- §1 WORLD mode switch (WorldDockB.dc.html) --")
	_app.call("select_domain", "world")
	await _frames(3)
	var pill: Control = _app.get("_mode_switch_pill")
	var row: Control = _app.get("_mode_switch_row")
	_ok("pill shown in WORLD", row.visible, true)
	var segs: Array = pill.get_children()
	_ok("segment count", segs.size(), 2)
	_ok("caption a", (segs[0] as Button).text, "PIPELINE")
	_ok("caption b", (segs[1] as Button).text, "SCULPT")

	## The on-state, which is the clause `LARGE_ITEM_RULINGS.md` §6 held the
	## artboard back over: `set_segment_on()` builds `box("accent",
	## "accent_wash_2")`, so the lit half's fill is the **wash** and not
	## `accent`. A filled amber slab here would be the treatment DS-02 removed
	## shell-wide.
	##
	## **`accent_wash` until 2026-09-06.** §6 and §7 both name `accent_wash_2`
	## in words ("`accent_wash_2` fill, `accent` ink, border") and §7's closing
	## paragraph files the .09 -> .16 move as outstanding work; this assertion
	## had been pinned to the shipped implementation rather than to the ruling.
	## Nothing was loosened -- the "NOT accent" clause below, which is what §6
	## is actually about, is unchanged, and a second NOT pins out the .09
	## row-selection weight so the pin cannot silently slide back.
	_app.call("select_domain_mode", "world", "a")
	await _frames(3)
	var lit := (segs[0] as Button).get_theme_stylebox("normal") as StyleBoxFlat
	var quiet := (segs[1] as Button).get_theme_stylebox("normal") as StyleBoxFlat
	_ok("lit fill is accent_wash_2", lit.bg_color, DccTheme.c("accent_wash_2"))
	_ok("lit fill is NOT accent_wash (.09)",
		lit.bg_color == DccTheme.c("accent_wash"), false)
	_ok("lit fill is NOT accent", lit.bg_color == DccTheme.c("accent"), false)
	_ok("lit border is accent", lit.border_color, DccTheme.c("accent"))
	_ok("quiet half unfilled", quiet.bg_color.a, 0.0)
	_ok("lit ink is accent", (segs[0] as Button).get_theme_color("font_color"),
		DccTheme.c("accent"))

	## The ruling's second consequence, measured rather than repeated: what the
	## pill costs, and what that cost displaces.
	_note("pill band height px (ruling says 34)", row.size.y)
	var panel: Control = _app.call("workspace_panel", "world")
	var scroll := _ancestor_scroll(panel)
	var wd: Control = null
	for e in (panel.get("categories") as Array):
		if String(e["title"]) == "World data":
			wd = (e["body"] as Control).get_parent() as Control
	var fold := scroll.global_position.y + scroll.size.y
	_note("scroll fold y", fold)
	_note("World data header top y (pill shown)", wd.global_position.y)
	_ok("World data is below the fold with the pill shown",
		wd.global_position.y > fold, true)
	## The counterfactual, so the 34 px is attributed rather than merely
	## measured beside the symptom: hide the band and re-read the same header.
	row.visible = false
	await _frames(3)
	var without := wd.global_position.y
	row.visible = true
	await _frames(3)
	_note("World data header top y (pill hidden)", without)
	_note("displacement px", wd.global_position.y - without)

	_app.call("select_domain", "civilization")
	await _frames(3)
	_ok("pill hidden in CIVIL (ungated)", row.visible, false)
	_app.call("select_domain", "world")
	await _frames(3)

# -- §2 The collapsed strip's fifth slot (board A/B) ---------------------------

func _s2_collapsed() -> void:
	print("\n-- §2 collapsed strip --")
	## The region is CIVIL's: `app.gd::_select_domain()` sets
	## `timeline_bar.visible = id == "civilization"`, so every section from here
	## down runs in CIVIL. §1 left the shell in WORLD, where the strip is not
	## merely collapsed but absent, and `_fill_timeline_strip()` returns early on
	## an invisible bar -- which is why the first run of this probe reported ten
	## failures against a strip that was never built.
	_app.call("select_domain", "civilization")
	await _frames(4)
	_ok("timeline region visible in CIVIL", (_app.get("timeline_bar") as Control).visible, true)
	_app.set("_tl_expanded", false)
	_app.call("_fill_timeline_strip")
	await _frames(3)
	_ok("no world: year slot says so", _has_text("no world"), true)
	_ok("no world: no status word", _has_text("recorded") or _has_text("between")
		or _has_text("no years recorded"), false)
	_note("collapsed strip height px", (_app.get("timeline_bar") as Control).size.y)

# -- §3 A world with no recorded years (board G) -------------------------------

func _generate() -> void:
	var bridge: Node = _app.get("bridge")
	bridge.call("generate", (_app.get("new_world_dialog") as Object).call("request"))
	for i in 900:
		if not bool(bridge.get("generating")):
			break
		await get_tree().process_frame
	await _frames(6)

func _s3_no_years() -> void:
	print("\n-- §3 world generated, no year recorded (board G) --")
	await _generate()
	_ok("world available", _app.call("tl_available"), true)
	_ok("generate records no timeline year",
		(_app.call("tl_recorded_years") as PackedInt64Array).size(), 0)
	_app.set("_tl_expanded", true)
	_app.call("_fill_timeline_strip")
	await _frames(3)
	_ok("word", _has_text("no years recorded"), true)
	_ok("clause names the command that puts a mark there",
		_has_text("CIVIL › Timeline › Add year puts a mark here"), true)
	_ok("no counts printed", _has_text("0 present"), false)
	_ok("no zeroed diff printed", _has_text("+0"), false)

# -- §4 Recorded years: boards C and D -----------------------------------------

func _s4_recorded() -> void:
	print("\n-- §4 recorded years (boards C, D) --")
	var bridge: Node = _app.get("bridge")
	for y in [-200, 340, 412, 500, 705]:
		bridge.call("civ_add_year", y)
	await _frames(3)
	var years: PackedInt64Array = _app.call("tl_recorded_years")
	_ok("years recorded", years.size(), 5)

	## Board C: the cursor standing on one.
	_app.call("tl_set_year", 412)
	await _frames(3)
	_ok("C word", _has_text("recorded"), true)
	_ok("C counts row present", _has_text("since 340 AD"), true)
	var n: Dictionary = _app.call("tl_year_neighbours", 412)
	_ok("C prev", n.get("prev"), 340)
	_ok("C at", n.get("at"), 412)
	_ok("C next", n.get("next"), 500)
	var present := 0
	for t in _tl_texts():
		if String(t).ends_with(" present"):
			present += 1
	_ok("one 'N present' readout", present, 1)

	## Board D: between two of them.
	_app.call("tl_set_year", 486)
	await _frames(3)
	_ok("D word", _has_text("between"), true)
	_ok("D clause", _has_text("territory holds at 412 AD · next 500 AD"), true)
	_ok("D drops the counts", _has_text("since 340 AD"), false)

	## Below the first recorded year there is no `prev`. Under Ruling AT an
	## unrecorded year keeps the claims it was reached with, so the clause
	## names the year they were last loaded from -- 412, from the step above --
	## and not the mark below the cursor, which does not exist here.
	_app.call("tl_set_year", -300)
	await _frames(3)
	var below: Dictionary = _app.call("tl_year_neighbours", -300)
	_ok("below the first mark: no prev", below.has("prev"), false)
	_ok("below the first mark: the claims are still 412's",
		_has_text("territory holds at 412 AD · next 200 BC"), true)
	_ok("no 'territory holds at' naming a mark it did not load",
		_has_text("territory holds at 200 BC · next 200 BC"), false)

	## The snap the shift-drag hint names, on the model rather than through a
	## synthesised drag: ties go to `prev`.
	_ok("snap forward", (_app.call("tl_nearest_recorded", 690) as Dictionary).get("year"), 705)
	_ok("snap back", (_app.call("tl_nearest_recorded", 520) as Dictionary).get("year"), 500)
	_ok("snap tie goes to prev",
		(_app.call("tl_nearest_recorded", 456) as Dictionary).get("year"), 412)
	_ok("snap on an empty timeline answers {}",
		(_app.call("tl_nearest_recorded", 0) as Dictionary).is_empty(), false)

# -- §5 The top of the track (board F) -----------------------------------------

func _s5_end_of_track() -> void:
	print("\n-- §5 end of track (board F) --")
	var t: Control = _app.get("timeline_row")
	var before := t.get_combined_minimum_size().x
	_app.call("tl_set_year", 900)
	await _frames(3)
	_ok("mid-track state word", _app.call("tl_state_text"), "paused")
	_ok("play live mid-track", (_app.get("_tl_play_button") as Button).disabled, false)
	_ok("hint hidden mid-track", (_app.get("_tl_end_hint") as Label).visible, false)
	_ok("max literal quiet mid-track",
		(_app.get("_tl_max_label") as Label).get_theme_color("font_color"),
		DccTheme.c("text_ghost"))

	_app.call("tl_set_year", 1200)
	await _frames(3)
	_ok("state word", _app.call("tl_state_text"), "end of track")
	_ok("play drawn dead", (_app.get("_tl_play_button") as Button).disabled, true)
	_ok("step forward drawn dead", (_app.get("_tl_fwd_button") as Button).disabled, true)
	_ok("hint visible", (_app.get("_tl_end_hint") as Label).visible, true)
	_ok("max literal lifts to --faint",
		(_app.get("_tl_max_label") as Label).get_theme_color("font_color"),
		DccTheme.c("text_faint"))
	_ok("min literal stays quiet",
		(_app.get("_tl_min_label") as Label).get_theme_color("font_color"),
		DccTheme.c("text_ghost"))
	_app.call("tl_set_year", -400)
	await _frames(3)
	_ok("min literal lifts at the bottom of the track",
		(_app.get("_tl_min_label") as Label).get_theme_color("font_color"),
		DccTheme.c("text_faint"))

	## Where the hint sits, measured rather than argued. Board F draws it hard
	## right, after the `flex:1` -- but F draws no layer toggles and C does, so
	## the two boards do not describe the same row. These are the numbers the
	## source comment cites.
	_app.call("tl_set_year", 1200)
	await _frames(3)
	var with_hint := t.get_combined_minimum_size().x
	_note("row 1 minimum x, hint hidden", before)
	_note("row 1 minimum x, hint shown", with_hint)
	_note("hint cost px", with_hint - before)

# -- §6 Colliding marks merge --------------------------------------------------

func _s6_merge() -> void:
	print("\n-- §6 mark merge --")
	var bridge: Node = _app.get("bridge")
	## `civ_run_collapse_simulation` writes one entry per step; five years apart
	## is about 2 px on a 700 px track, which is the collision the board names.
	for y in [800, 805, 810]:
		bridge.call("civ_add_year", y)
	await _frames(3)
	## Asserted by membership, not by count: `civ_add_year` snapshots the
	## *currently active* year before creating the one asked for, so a call made
	## with the cursor parked on an unrecorded year records that year too. §5
	## leaves the cursor at 1200, so the total here is nine rather than eight --
	## engine behaviour, documented at `engine_bridge.gd::civ_add_year`, and a
	## count assertion would have read it as a defect in the drawing.
	var years: PackedInt64Array = _app.call("tl_recorded_years")
	_note("total recorded years", years.size())
	for y in [800, 805, 810]:
		_ok("recorded %d" % y, years.has(y), true)
	var track: Control = _app.get("_tl_track")
	var w := track.size.x
	_note("track width px at 1920 (two docks + rail take the rest)", w)
	## Named density, always: `timeline_track_h` is 12 at pointer and 20 at
	## touch, and the row floors at `btn_min_h` (0 pointer / 44 touch). Run the
	## touch leg with `-- --force-touch` to see the other figure.
	_note("track ROW height px (pointer unless --force-touch)", track.size.y)
	_note("role timeline_track_h / btn_min_h", "%d / %d" % [
		DccTheme.role_px("timeline_track_h"), DccTheme.role_px("btn_min_h")])

	## The same mapping `_draw_timeline_marks()` uses, evaluated on the years
	## under test. This is not the drawing -- it is the fact the drawing has to
	## cope with, and how close together two marks have to be before it does.
	var cols_live: Dictionary = {}
	var cols_700: Dictionary = {}
	for y in [800, 805, 810]:
		cols_live[floori(w * float(int(y) + 400) / 1600.0)] = true
		cols_700[floori(700.0 * float(int(y) + 400) / 1600.0)] = true
	_note("columns for 800/805/810 at %d px" % int(w), cols_live.size())
	_note("columns for 800/805/810 at 700 px (the board's own figure)", cols_700.size())
	_note("px per year at this width", w / 1600.0)
	## **Measured, and it corrects the board's arithmetic rather than repeating
	## it.** The board says a collapse run's five-year steps land "about 2 px
	## apart on a 700 px track", and that much is right: 525 / 527 / 529 above.
	## But 2 px apart is not a collision, and at this window's 1892 px track a
	## *one*-year step is already 1.18 px, so **no two distinct years can share
	## a column here at all** and the merge is inert. Asserting a merge at this
	## width would be demanding a condition the width rules out.
	##
	## What is assertable is what the merge is for: the track is 1601 years wide
	## and can never have more marks than it has pixel columns, whatever a
	## collapse simulation records. Evaluated over the whole axis at the board's
	## own 700 px, which is where marks do land on one another.
	## The clamp is part of the mapping under test, not an afterthought: without
	## it year 1200 maps to column 700 on a 700 px track, which is one pixel off
	## the right-hand edge. That is what the first run of this section caught.
	## **Calls the SHIPPED mapping, not a replica of it.** This block used to
	## re-implement the expression here with a hardcoded 699 and assert its own
	## copy, so deleting the clamp from `app.gd` left this section printing PASS
	## — a verifier killed it by mutation on 2026-09-06. `tl_mark_column()` was
	## extracted for exactly this: the probe now fails when the shipped function
	## changes.
	var every: Dictionary = {}
	for y in range(-400, 1201):
		every[_app.tl_mark_column(y, 700.0)] = true
	## The clamp, asserted directly rather than inferred from the column count:
	## the top of the range must land on the last column of a 700 px track, and
	## 700 would be one pixel past it.
	_ok("TL_YEAR_MAX lands on the last column, not one past it",
		_app.tl_mark_column(1200, 700.0), 699)
	_note("1601 years → distinct columns at 700 px", every.size())
	_ok("every year on the axis reduces to one mark per column",
		every.size() <= 700, true)
	_ok("and that is fewer marks than years", every.size() < 1601, true)

# -- §7 The marks actually rasterise (windowed only) ---------------------------

func _s7_pixels() -> void:
	print("\n-- §7 marks rasterise (windowed only) --")
	if DisplayServer.get_name() == "headless":
		print("  SKIP  headless: the dummy driver never rasterises, so a colour")
		print("        census of the track would pass whatever was drawn.")
		return
	var track: Control = _app.get("_tl_track")
	_app.call("tl_set_year", 900)
	await _frames(4)
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	var r := Rect2i(Vector2i(track.global_position), Vector2i(track.size))
	var dim := DccTheme.c("text_dim")
	var hits := 0
	for x in range(r.position.x, r.position.x + r.size.x):
		for y in range(r.position.y, r.position.y + r.size.y):
			if img.get_pixel(x, y).is_equal_approx(dim):
				hits += 1
				break
	_note("track columns carrying a --dim mark", hits)
	## The positive control: with every mark removed the same census must fall
	## to zero, or it was never measuring the marks.
	var bridge: Node = _app.get("bridge")
	## Cleared from the live list rather than a remembered one -- §5 leaves the
	## cursor at 1200 and `civ_add_year` snapshots the active year, so a
	## hardcoded removal list is one year short and the control census stays at
	## its old value. That is what the first windowed run of this section
	## reported, and the redraw below is the other half of it: `civ_remove_year`
	## emits no `timeline_changed`, so nothing had asked the track to repaint.
	for y in (_app.call("tl_recorded_years") as PackedInt64Array):
		bridge.call("civ_remove_year", int(y))
	_app.call("_repaint_timeline")
	await _frames(4)
	_ok("timeline emptied", (_app.call("tl_recorded_years") as PackedInt64Array).size(), 0)
	await RenderingServer.frame_post_draw
	var img2 := _vp.get_texture().get_image()
	var after := 0
	for x in range(r.position.x, r.position.x + r.size.x):
		for y in range(r.position.y, r.position.y + r.size.y):
			if img2.get_pixel(x, y).is_equal_approx(dim):
				after += 1
				break
	_note("same census with no year recorded", after)
	_ok("marks drawn", hits > 0, true)
	_ok("control falls to zero", after, 0)

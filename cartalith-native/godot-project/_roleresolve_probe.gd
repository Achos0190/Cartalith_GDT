extends Node
## **DS-03's invariant guard.** The role-keyed resolver (`DccTheme.ROLE` /
## `role_px()`) exists and is wired at ~40 live call sites, but until this probe
## nothing verified that it still *does the job it exists for*. Every property
## below was carried only by prose in `dcc_theme.gd`, which is the state
## `MISTAKES.md`'s "prose that describes the old behaviour" entry (x15) is about.
##
##   Godot_v4.7.1 --headless --path . _roleresolve_probe.tscn
##
## It boots no shell and needs no display: `DccTheme` is a `RefCounted` with
## static state, so the resolver can be driven directly through `set_touch()` /
## `set_phone()`. That is deliberate -- a resolver test that needed a live
## 2560x1600 `SubViewport` would be measuring the builder, not the resolver, and
## `_tabletparity_probe.gd` already measures the builder.
##
## The five properties, and what each would catch:
##
##   A. The predicate is `is_tablet()`, never `is_touch()`. `GUI_GAP_REGISTER.md`
##      section 57 refuted a role resolver on exactly this ground: `_phone`
##      requires `_touch`, so an `is_touch()` resolver hands the phone every
##      tablet figure. Nothing pinned it until now.
##   B. The collision set survives. 12 desktop integers each carry 2 or 3
##      distinct tablet answers; that is the whole reason a value-keyed table
##      could not serve. Collapsing any pair is the DS-03 regression.
##   C. `TABLET` and `ROLE` hold the relationship `ROLE`'s own header states --
##      in BOTH directions. Four rows must agree ("if one moves the other must
##      move with it"); four must NOT, because those are the collisions. A
##      one-directional test would invite someone to "fix" the disagreement.
##   D. Every figure that has a home matches the governing canvas,
##      `design/dcc-environment-2026-08-31/Cartalith DCC Environment.dc.html`
##      (`:25` pointer tokens, `:1819` `densStr` touch tokens), asserted as
##      literals read off that file rather than against the constants themselves
##      -- `MISTAKES.md`, "a test that compares a constant against itself" (x2).
##   E. The desktop column is dumped in full, so a change to the tablet half can
##      be shown to move nothing on a pointer.

var _fail := 0

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

## Every assertion below mutates `DccTheme`'s static device state, and a probe
## that left it mutated would be a trap for anything sharing the process.
func _as(touch: bool, phone: bool) -> void:
	DccTheme.set_touch(touch)
	DccTheme.set_phone(phone)
	DccTheme.set_narrow(false)

## desktop px -> the distinct tablet answers the roles at that desktop px give.
func _collisions() -> Dictionary:
	var by := {}
	for k in DccTheme.ROLE:
		var pair: Array = DccTheme.ROLE[k]
		var d: int = int(pair[0])
		if not by.has(d):
			by[d] = {}
		by[d][int(pair[1])] = true
	var out := {}
	for d in by:
		var answers: Array = (by[d] as Dictionary).keys()
		if answers.size() > 1:
			answers.sort()
			out[d] = answers
	return out

func _ready() -> void:
	print("=== DS-03 role resolver invariants ===")

	# -- A. The predicate section 57 refuted a resolver over -----------------
	print("")
	print("-- A. predicate: is_tablet(), never is_touch() --")
	_as(true, false)
	_ok("tablet is touch and not phone", DccTheme.is_tablet(), true)
	_ok("tablet takes the tablet column (row_min_h)", DccTheme.role_px("row_min_h"), 44)
	## The assertion that fails if the predicate is ever "simplified" to
	## `is_touch()`: a phone is `_touch` too, so an `is_touch()` resolver
	## answers 44 here instead of the desktop 0. This is the single line that
	## stands between the shell and section 57's refuted design.
	_as(true, true)
	_ok("phone is touch but NOT tablet", DccTheme.is_tablet(), false)
	_ok("phone does NOT take the tablet column", DccTheme.role_px("row_min_h"), 0)
	_ok("phone does NOT take the tablet chip tier", DccTheme.role_px("chip_min_h"), 0)
	_ok("phone does NOT take the tablet prose size", DccTheme.role_px("fs_prose"), 11)
	_as(false, false)
	_ok("pointer takes the desktop column", DccTheme.role_px("row_min_h"), 0)
	## `is_laptop()` is `narrow and not touch` -- the `!touch` in `ENV:1819`'s
	## own expression. A 1366-wide tablet must keep the 400 px touch docks.
	DccTheme.set_touch(true)
	DccTheme.set_phone(false)
	DccTheme.set_narrow(true)
	_ok("narrow tablet is NOT laptop", DccTheme.is_laptop(), false)
	_ok("narrow tablet keeps the 400px dock", DccTheme.role_px("w_left_dock"), 400)
	DccTheme.set_touch(false)
	DccTheme.set_narrow(true)
	_ok("narrow pointer IS laptop", DccTheme.is_laptop(), true)
	_ok("laptop override narrows the left dock", DccTheme.role_px("w_left_dock"), 330)
	_ok("laptop override leaves an unlisted role alone", DccTheme.role_px("h_menu_bar"), 36)
	_as(false, false)

	# -- B. The collision set DS-03 exists to preserve -----------------------
	print("")
	print("-- B. collision set: one desktop integer, two or three tablet answers --")
	var col := _collisions()
	var keys: Array = col.keys()
	keys.sort()
	for d in keys:
		print("     desktop %3d -> %s" % [d, str(col[d])])
	## Pinned as a literal, not as `col.size()` compared to itself. 12 was
	## measured off `ROLE` on 2026-09-03; section 57 reported "at least five",
	## which was a floor rather than a count.
	_ok("collisions in ROLE", col.size(), 12)
	## The rows that carry the argument, each asserted as the literal pair the
	## canvas draws. Merging any of these back into one answer is exactly the
	## regression that re-exhausts the key space.
	var expect := {
		0: [34, 44],        ## chip tier B / button tier A -- the touch layer
		1: [1, 2],          ## hairline pinned / open-title underline
		9: [11, 16],        ## dock row gap + header type / chip x-padding
		10: [12, 13, 14],   ## status type / timeline+shortcut+viewport / body pad
		11: [13, 14, 18],   ## mono readout / sans prose / button x-padding
		12: [15, 20],       ## wordmark / timeline scrub track
		14: [18, 22],       ## grid+rail gaps / bar x-padding
		22: [24, 26],       ## timeline gap / readout gap + hero2
		26: [30, 36],       ## hero type / status bar height
		36: [36, 52],       ## FAB pinned / menu bar height
		40: [48, 56],       ## rail width / tool-options height -- section 57's row
		70: [88, 90],       ## timeline height / slider track width
	}
	for d in expect:
		_ok("desktop %d resolves to %s" % [d, str(expect[d])],
			str(col.get(d, [])), str(expect[d]))

	# -- C. TABLET <-> ROLE, in both directions ------------------------------
	print("")
	print("-- C. the value-keyed and role-keyed tables, where they must agree --")
	## `ROLE`'s header: "These duplicate `TABLET`'s five rows on purpose ... if
	## one ever moves the other must move with it." Enforced here for the first
	## time; it was prose only.
	for row in [["h_menu_bar", 36], ["h_status", 26], ["h_timeline", 70], ["w_rail", 40]]:
		var k: String = row[0]
		var d: int = row[1]
		_ok("TABLET[%d] == ROLE[%s] tablet half" % [d, k],
			int(DccTheme.TABLET[d]), int((DccTheme.ROLE[k] as Array)[1]))
	print("")
	print("-- C2. ... and where they must NOT, because those are the collisions --")
	## The other direction, and it matters more: these four look like bugs in a
	## diff. `TABLET[40]` is the rail and `ROLE.h_tool_options` is the bar;
	## making them agree is how DS-03 gets silently undone by someone tidying.
	for row in [["h_tool_options", 40, 56], ["slider_track_w", 70, 90],
			["w_fab", 36, 36], ["fs_hero", 26, 30]]:
		var k: String = row[0]
		var d: int = row[1]
		var want: int = row[2]
		_ok("ROLE[%s] keeps its own answer, not TABLET[%d]=%d"
				% [k, d, int(DccTheme.TABLET[d])],
			int((DccTheme.ROLE[k] as Array)[1]), want)
		_ok("  ... and the two genuinely differ",
			int((DccTheme.ROLE[k] as Array)[1]) != int(DccTheme.TABLET[d]), true)

	# -- D. The governing canvas ---------------------------------------------
	print("")
	print("-- D. every homed figure against ENV:25 / ENV:1819, as literals --")
	## Read off `Cartalith DCC Environment.dc.html` on 2026-09-03: `:25` is the
	## pointer root, `:1819`'s `densStr` the touch overrides. Owner ruling
	## 2026-08-25 makes this canvas the newer authority.
	var env := [
		["--menuH", "h_menu_bar", 36, 52],
		["--tbH", "h_tool_options", 40, 56],
		["--railW", "w_rail", 40, 48],
		["--sbH", "h_status", 26, 36],
		["--tool", "h_rail_head", 30, 44],
		["--ldW", "w_left_dock", 372, 400],
		["--rdW", "w_right_dock", 304, 400],
		["--pop", "w_menu_popup", 300, 380],
		["--popW", "w_popover", 238, 300],
		["--railExpW", "w_rail_expanded", 200, 264],
		["--hero", "fs_hero", 26, 30],
		["--hero2", "fs_hero_2", 22, 26],
		["--m2", "fs_dock_header", 9, 11],
		["--m1", "fs_status", 10, 12],
		["--fs", "fs_prose", 11, 14],  ## 11.5px -> 11; Godot font sizes are ints
	]
	for e in env:
		var tok: String = e[0]
		var k: String = e[1]
		_as(false, false)
		_ok("%s pointer -> ROLE[%s]" % [tok, k], DccTheme.role_px(k), int(e[2]))
		_as(true, false)
		_ok("%s touch   -> ROLE[%s]" % [tok, k], DccTheme.role_px(k), int(e[3]))
	_as(false, false)

	# -- E. The desktop column, dumped whole ---------------------------------
	print("")
	print("-- E. desktop column (pointer, not narrow) -- diff this across a change --")
	var names: Array = DccTheme.ROLE.keys()
	names.sort()
	var acc := 0
	for k in names:
		var v: int = DccTheme.role_px(k)
		acc = (acc * 31 + v + k.length()) % 1000000007
		print("     %-18s %d" % [k, v])
	print("     DESKTOP-DIGEST %d over %d roles" % [acc, names.size()])
	print("")
	print("-- E2. tablet column --")
	_as(true, false)
	var tacc := 0
	for k in names:
		tacc = (tacc * 31 + DccTheme.role_px(k) + k.length()) % 1000000007
	print("     TABLET-DIGEST %d over %d roles" % [tacc, names.size()])
	_as(false, false)

	print("")
	print("[RESULT] failures=", _fail)
	get_tree().quit(1 if _fail > 0 else 0)

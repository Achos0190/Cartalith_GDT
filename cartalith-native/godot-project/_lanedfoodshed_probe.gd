extends Node
## Lane D, 2026-09-05. Guards `ECONOMY_SCOPE.md` milestone 2's UI surface
## against a **live** pass, which nothing did before this probe.
##
## ## The gap this closes
##
## Two probes already touch the food shed and neither can see the defect this
## one found:
##
##   * `_foodshed_probe.gd` renders `place_editor_window.gd::_food_shed_note`
##     against a **synthetic `const ROW`** typed out from the doc comment. A
##     row written from the doc cannot refute the doc -- if the binding stops
##     emitting a key the doc names, or emits one it does not, that probe
##     stays green. It is a constant asserted against itself, one layer out.
##   * `_bridgeforward_probe.gd`'s T5 asserts the row shape by `has()` on a
##     live pass -- for **smelting and salt only**. Its key lists cover
##     `civ_place_smelting` (ten keys) and `civ_salt_access` (four); it walks
##     past `civ_food_shed` entirely.
##
## So the food-shed row was the one of the three whose shape nothing checked,
## and it was the one that had drifted: `trade_store.gd::food_shed_for()`'s
## doc named nine keys, `civ_trade_bridge.rs`'s `dict!` emits **twelve**, and
## one of the three missing (`pop`) is read by the shipped readout. Corrected
## in the same change that added this probe.
##
## ## What it asserts
##
## T1  every key `civ_trade_bridge.rs::civ_food_shed`'s `dict!` writes is
##     present on a live row, by `has()` -- never `get(k, default)`, which
##     cannot tell an absent key from a real zero.
## T2  the rows array is exactly the roster's length, and the first index
##     past it answers `{}`. That is the **second** branch that produces the
##     dashed readout, and `_foodshed_probe.gd` only ever drove the first.
## T3  the readout drawn from the live row, read back off the Labels, and
##     each figure cross-checked against the row it came from.
## T4  `pop` is load-bearing: change it in the cache and the drawn line moves.
##     A key nothing reads could be dropped from the binding unnoticed; this
##     is the check that says it cannot.
## T5  the dashed branch, live -- em dash plus its reason, never a blank.
##
## The key list below is a literal transcribed from the `dict!` in
## `civ_trade_bridge.rs`, not read back from the row under test. Carrying it
## is the point: a list derived from the row holds for every row.
##
## Run:
##   godot --headless --path . _lanedfoodshed_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row.

## Transcribed from `civ_trade_bridge.rs`'s `fn civ_food_shed` `dict!` block.
## Three come from the binding (`index`, `name`, `pop`, built off
## `civ.settlements[i]`); nine from `cartalith_civ::trade::FoodShed`.
const FOOD_SHED_KEYS := [
	"index", "name", "pop",
	"local_capacity", "hinterland_capacity", "import_capacity",
	"supported", "suppliers", "best_mode", "limited_by",
	"sustainable", "over_by",
]

var app: Node
var _fails := 0


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _check(ok: bool, what: String) -> void:
	if ok:
		print("  ok: %s" % what)
	else:
		print("  FAIL: %s" % what)
		_fails += 1


func _gen(seed_value: int) -> void:
	app.bridge.generate({
		"seed": seed_value, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while app.bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(8)


## Draw `_food_shed_note` for one settlement index and return the prose it
## actually put on screen, joined by newlines. Reads the Labels back off the
## host rather than re-deriving the strings -- the whole point of a probe over
## a unit test.
##
## Never `setup()`, on `_foodshed_probe.gd`'s own precedent: `_food_shed_note`
## reads `_index` and nothing else off the window.
func _render(idx: int) -> String:
	return _draw(idx, "_food_shed_note")


## The same, for the sibling readout that carries the same two-branch dash.
func _render_ss(idx: int) -> String:
	return _draw(idx, "_smelting_salt_note")


func _draw(idx: int, method: String) -> String:
	var host := VBoxContainer.new()
	add_child(host)
	var w := PlaceEditorWindow.new()
	w._index = idx
	w.call(method, host)
	var parts: Array[String] = []
	for n in host.get_children():
		if n is Label:
			parts.append((n as Label).text)
	w.free()
	host.free()
	return "\n".join(parts)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 420.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	var bridge = app.bridge

	print("--- T0. generate, then run the real refresh ---")
	await _gen(41207)
	var settlements: Array = bridge.settlements()
	_check(settlements.size() > 0, "%d settlements generated" % settlements.size())
	if settlements.is_empty():
		print("no settlements -- civ_food_shed() returns {} by contract and nothing "
			+ "below can discriminate. Aborting.")
		get_tree().quit(1)
		return
	TradeStore.refresh(bridge)
	_check(TradeStore.is_matched(), "trade flows: a match is held")
	_check(TradeStore.is_food_shed_matched(), "food shed: a pass is held")
	if not TradeStore.is_food_shed_matched():
		print("no food-shed pass -- T1..T4 cannot run. Aborting.")
		get_tree().quit(1)
		return

	# ---- T1. the row shape, by has(), against the binding's own dict! ----
	print("--- T1. every documented key is present on a LIVE row ---")
	var row: Dictionary = TradeStore.food_shed_for(0)
	_check(not row.is_empty(), "food_shed_for(0) reads back through its real reader")
	for k in FOOD_SHED_KEYS:
		_check(row.has(k), "food-shed row 0 carries `%s`" % k)
	## Negative control: `has()` answering true for twelve keys proves nothing
	## unless it can also say no.
	_check(not row.has("no_such_food_shed_key"),
		"negative control: has() says NO to an invented key")
	## And the converse direction -- the doc must not be SHORTER than the row
	## either, which is the exact drift that made this probe necessary.
	var undocumented: Array = []
	for k in row.keys():
		if not (String(k) in FOOD_SHED_KEYS):
			undocumented.append(k)
	_check(undocumented.is_empty(),
		"the row carries no key this probe's list omits (extra: %s)" % [undocumented])
	print("  row 0 = %s, pop %d, supported %.1f, limited_by %s, sustainable %s" % [
		String(row.get("name", "?")), int(row.get("pop", -1)),
		float(row.get("supported", -1.0)), String(row.get("limited_by", "?")),
		bool(row.get("sustainable", false))])

	# ---- T2. the array length, and the SECOND dash branch ----------------
	print("--- T2. rows length == roster, and one past the end is {} ---")
	var readable := 0
	for i in settlements.size():
		if not TradeStore.food_shed_for(i).is_empty():
			readable += 1
	_check(readable == settlements.size(),
		"%d rows readable == %d settlements" % [readable, settlements.size()])
	_check(TradeStore.food_shed_for(settlements.size()).is_empty(),
		"no row at index %d -- out-of-range is the dash's second cause"
			% settlements.size())
	_check(TradeStore.food_shed_for(-1).is_empty(), "negative index is {} too")
	## Drawn, not just returned. This is the check that found the defect this
	## batch fixed: before it, index 215 drew *"civ_food_shed() returned
	## nothing, which is what an engine build older than milestone 2 does"* --
	## on a run that had just read 215 rows out of that same pass. A wrong
	## reason, stated in the voice of a checked one.
	##
	## So the assertion is two-sided. The dash must appear WITH the reason that
	## is true here (a held pass, no row at this index), and must NOT carry the
	## whole-pass reason, which belongs to the other branch alone.
	var oob := _render(settlements.size())
	_check("—" in oob, "out-of-range index still dashes rather than drawing blank")
	_check("the pass ran and covers %d settlements" % settlements.size() in oob,
		"and says the TRUE thing: the pass is held, this index is past it")
	_check(not ("engine build older" in oob),
		"and NOT the engine-too-old reason, which this same run disproves: %s"
			% JSON.stringify(oob))

	# ---- T3. the readout, drawn from the live row and read back ----------
	print("--- T3. the drawn readout, cross-checked against the row ---")
	var drawn := _render(0)
	_check(not drawn.strip_edges().is_empty(), "settlement 0 drew something")
	_check("Food shed: supports" in drawn, "it is the live readout, not the dash")
	_check(not ("—" in drawn), "no em dash on a settlement that has a row")
	var want_supported := FactionRosterWindow._thousands(
		int(round(float(row.get("supported", 0.0)))))
	_check("supports %s people" % want_supported in drawn,
		"drew `supports %s people`, recomputed from the row's own `supported`"
			% want_supported)
	var want_local := FactionRosterWindow._thousands(
		int(round(float(row.get("local_capacity", 0.0)))))
	_check("%s from its own catchment" % want_local in drawn,
		"drew `%s from its own catchment` from `local_capacity`" % want_local)
	_check("Limited by %s." % String(row.get("limited_by", "?")) in drawn,
		"drew `Limited by %s.`" % String(row.get("limited_by", "?")))
	## `best_mode` is `land` whether or not anything shipped, so it may only
	## appear when a supplier actually contributed -- `FoodShed`'s own caveat,
	## checked here against whichever case this live world produced.
	var suppliers := int(row.get("suppliers", 0))
	if suppliers > 0:
		_check("over %s" % String(row.get("best_mode", "land")) in drawn,
			"%d suppliers, so `best_mode` is reported" % suppliers)
	else:
		_check("nothing imported" in drawn and not ("over land" in drawn),
			"0 suppliers, so no mode is named -- `best_mode` suppressed")
	print("  drawn = %s" % JSON.stringify(drawn))

	# ---- T4. `pop` is load-bearing, not decoration -----------------------
	## The key whose omission from `food_shed_for()`'s doc this batch found.
	## Move it in the cache and the drawn line must move with it; if it does
	## not, the readout is reading a population from somewhere else and the
	## doc gap was harmless. It is not.
	print("--- T4. the drawn population comes from the row's `pop` ---")
	var live_pop := int(row.get("pop", 0))
	_check("Population %s when the match ran" % FactionRosterWindow._thousands(live_pop)
			in drawn,
		"drew the row's own pop (%d)" % live_pop)
	var bumped := live_pop + 777
	var saved_rows: Array = TradeStore.food_shed().get("rows", []).duplicate(true)
	var mutated: Dictionary = (row as Dictionary).duplicate(true)
	mutated["pop"] = bumped
	TradeStore._food_shed = {"rows": [mutated]}
	var after := _render(0)
	_check("Population %s when the match ran" % FactionRosterWindow._thousands(bumped)
			in after,
		"changing `pop` to %d moved the drawn line -- the key is read, not ignored"
			% bumped)

	# ---- T5. the dashed branch, live -------------------------------------
	print("--- T5. an absent pass dashes with its reason, never a blank ---")
	TradeStore._food_shed = {}
	var dashed := _render(0)
	_check(not dashed.strip_edges().is_empty(),
		"an absent pass still draws -- a blank field is what is forbidden")
	_check("—" in dashed, "the em dash is present")
	_check("civ_food_shed()" in dashed and "ECONOMY_SCOPE.md" in dashed,
		"and its reason names the symbol and the milestone: %s" % JSON.stringify(dashed))
	## The stated reason is "an engine build older than milestone 2". On THIS
	## build that cause is false, and the check that says so is the native
	## export -- the question `menus.gd::_engine_has()` splits out.
	_check(bridge.world_gen.has_method("civ_food_shed"),
		"the loaded cdylib DOES export civ_food_shed -- so the dash above is the "
			+ "forced-empty cache, not the reason it names")
	_check(not bridge.world_gen.has_method("civ_food_shed_no_such_binding"),
		"negative control: WorldGen.has_method() says NO to an invented name")

	TradeStore._food_shed = {"rows": saved_rows}

	# ---- T6. the same split, in the two sibling readouts -----------------
	## `_smelting_salt_note` carried the identical single-reason dash and was
	## fixed in the same change. Checked here rather than left to inference:
	## the defect was one shape in three readouts, so the guard has to be too.
	print("--- T6. smelting and salt split the same two branches ---")
	var ss_live := _render_ss(0)
	_check(not ss_live.strip_edges().is_empty(), "settlement 0 drew a smelting/salt readout")
	_check(not ("no row for this settlement" in ss_live),
		"live rows do not dash: %s" % JSON.stringify(ss_live))
	var ss_oob := _render_ss(settlements.size())
	_check("Smelting: — the pass ran but has no row at index %d" % settlements.size() in ss_oob,
		"smelting out-of-range says the pass is held, not that the binding is missing")
	_check("Salt: — the pass ran but has no row at index %d" % settlements.size() in ss_oob,
		"salt out-of-range says the same")
	_check(not ("returned nothing" in ss_oob),
		"and neither claims the pass returned nothing: %s" % JSON.stringify(ss_oob))
	## The other branch, and the two predicates added to reach it.
	_check(TradeStore.is_smelting_matched() and TradeStore.is_salt_matched(),
		"is_smelting_matched()/is_salt_matched() say YES while both passes are held")
	TradeStore._smelting = {}
	TradeStore._salt = {}
	_check(not TradeStore.is_smelting_matched() and not TradeStore.is_salt_matched(),
		"negative control: both say NO once the passes are dropped")
	var ss_absent := _render_ss(0)
	_check("civ_place_smelting()" in ss_absent and "civ_salt_access()" in ss_absent,
		"an absent pass names the binding instead: %s" % JSON.stringify(ss_absent))
	_check("without the binding" in ss_absent,
		"and states the cause that is actually true on that branch")
	TradeStore.refresh(bridge)
	_check(TradeStore.is_smelting_matched() and TradeStore.is_salt_matched(),
		"refresh() restored both passes -- the probe leaves the store as it found it")

	print("=== %s ===" % ("PASS" if _fails == 0 else "%d FAILURES" % _fails))
	get_tree().quit(0 if _fails == 0 else 1)

extends Node
## Lane D, 2026-09-05. Proves the two `EngineBridge` forwarders added this
## batch (`civ_place_smelting`, `civ_salt_access`) make `TradeStore.refresh()`
## complete, and that the `has_method` guards added beside them actually
## discriminate.
##
## The defect this closes: `#[func] civ_place_smelting` and `#[func]
## civ_salt_access` have been on `WorldGen` (`civ_trade_bridge.rs`) since
## 2026-09-02 and `trade_store.gd` has called them **through `EngineBridge`**
## since the same day -- but `engine_bridge.gd` had no wrapper for either, so
## every `Match trade flows` press aborted with `Nonexistent function
## 'civ_place_smelting' in base 'Node (EngineBridge)'` on the third of its four
## lines and never returned. A previous lane blamed a stale
## `target/debug/cartalith_godot.dll`; T1 below is the check that refutes that,
## and it is asked of the native library rather than of the GDScript wrapper --
## the split `menus.gd::_engine_has()` documents.
##
## Every assertion below was unreachable before this batch, by construction:
## `refresh()` could not return, so `_smelting` and `_salt` could never hold a
## row.
##
## Run:
##   godot --headless --path . _bridgeforward_probe.tscn

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

	# ---- T0. the GDScript wrapper half -----------------------------------
	print("--- T0. EngineBridge (GDScript) has a wrapper for all four ---")
	for name in ["civ_trade_flows", "civ_food_shed", "civ_place_smelting", "civ_salt_access"]:
		_check(bridge.has_method(name), "EngineBridge.%s() exists" % name)
	## Negative control: without one, `has_method` answering true proves
	## nothing -- a predicate that cannot say no is not a check.
	_check(not bridge.has_method("civ_place_smelting_no_such_method"),
		"negative control: has_method() says NO to an invented name")

	# ---- T1. the native half -- refuting the stale-dll diagnosis ---------
	print("--- T1. the loaded cdylib exports the #[func]s (NOT a stale dll) ---")
	for name in ["civ_place_smelting", "civ_salt_access"]:
		_check(bridge.world_gen.has_method(name),
			"WorldGen.%s() is in the loaded native library" % name)
	_check(not bridge.world_gen.has_method("civ_place_smelting_no_such_method"),
		"negative control: WorldGen.has_method() says NO to an invented name")
	_check(bridge.missing_bindings().is_empty(),
		"EngineBridge._has() recorded no missing binding: %s"
			% [bridge.missing_bindings()])

	# ---- T2. a world with settlements to read --------------------------
	print("--- T2. generate ---")
	await _gen(41207)
	var settlements: Array = bridge.settlements()
	_check(settlements.size() > 0, "%d settlements generated" % settlements.size())
	if settlements.is_empty():
		print("no settlements -- the four passes all return {} by contract, "
			+ "so nothing below can discriminate. Aborting.")
		get_tree().quit(1)
		return

	# ---- T3. the call that used to abort ---------------------------------
	print("--- T3. TradeStore.refresh() completes ---")
	var d = TradeStore.refresh(bridge)
	_check(typeof(d) == TYPE_DICTIONARY,
		"refresh() RETURNED (typeof=%d, TYPE_DICTIONARY=%d) -- it aborted at "
			% [typeof(d), TYPE_DICTIONARY]
			+ "line 3 of 4 before this batch and returned nothing at all")

	# ---- T4. all four stores hold a real pass ----------------------------
	print("--- T4. all four passes landed, cross-checked against the roster ---")
	_check(TradeStore.is_matched(), "trade flows: a match is held")
	_check(TradeStore.is_food_shed_matched(), "food shed: a pass is held")
	## `TradeStore` publishes no whole-dictionary reader for the smelting and
	## salt passes, so the row count is measured **through the per-settlement
	## readers the place editor actually uses** -- every index in range answers,
	## and the first index out of range does not. That is a size check made of
	## the public surface, and it is cross-checked against `get_settlements()`,
	## an independent count, rather than against the pass's own
	## `settlement_count` -- a figure compared with itself holds for every value
	## of itself.
	var smelt_n := 0
	var salt_n := 0
	for i in settlements.size():
		if not TradeStore.smelting_for(i).is_empty():
			smelt_n += 1
		if not TradeStore.salt_access_for(i).is_empty():
			salt_n += 1
	_check(smelt_n == settlements.size(),
		"smelting: %d rows readable == %d settlements" % [smelt_n, settlements.size()])
	_check(salt_n == settlements.size(),
		"salt: %d rows readable == %d settlements" % [salt_n, settlements.size()])
	_check(TradeStore.smelting_for(settlements.size()).is_empty()
			and TradeStore.salt_access_for(settlements.size()).is_empty(),
		"neither pass has a row at index %d -- the arrays are exactly the roster's length"
			% settlements.size())

	# ---- T5. the documented keys are present, by has() not get(k, default)
	print("--- T5. row shape matches civ_trade_bridge.rs's own doc ---")
	var smelt: Dictionary = TradeStore.smelting_for(0)
	_check(not smelt.is_empty(), "smelting_for(0) reads back through its real reader")
	for k in ["index", "name", "iron_kg_yr", "charcoal_kg_yr", "ore_kg_yr",
			"woodland_ha", "limited_by", "fuel_poor", "ore_rich", "coppice_ha_needed"]:
		_check(smelt.has(k), "smelting row 0 carries `%s`" % k)
	var salt: Dictionary = TradeStore.salt_access_for(0)
	_check(not salt.is_empty(), "salt_access_for(0) reads back through its real reader")
	for k in ["index", "name", "has", "source"]:
		_check(salt.has(k), "salt row 0 carries `%s`" % k)
	print("  row 0 = %s / limited_by=%s, salt=%s from %s" % [
		String(smelt.get("name", "?")), String(smelt.get("limited_by", "?")),
		bool(salt.get("has", false)), String(salt.get("source", "?"))])

	# ---- T6. the call-site guard is live, not decorative -----------------
	## Hand `refresh()` a bridge with none of the four methods. Before this
	## batch that is exactly the state `EngineBridge` was in for two of them,
	## and the call died on the spot; the guard must now degrade to `{}`.
	## Run LAST: it overwrites all four stores with the absent value.
	print("--- T6. refresh() degrades instead of aborting on a wrapperless bridge ---")
	var stub := Node.new()
	add_child(stub)
	for name in ["civ_trade_flows", "civ_food_shed", "civ_place_smelting", "civ_salt_access"]:
		_check(not stub.has_method(name), "stub bridge has no %s()" % name)
	var d2 = TradeStore.refresh(stub)
	_check(typeof(d2) == TYPE_DICTIONARY and (d2 as Dictionary).is_empty(),
		"refresh(stub) returned {} rather than aborting the caller")
	_check(TradeStore.smelting_for(0).is_empty() and TradeStore.salt_access_for(0).is_empty(),
		"the stores hold the absent value, not a minted zero row")

	print("")
	print("FAILURES: %d" % _fails)
	get_tree().quit(1 if _fails > 0 else 0)

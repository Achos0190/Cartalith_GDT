extends Node
## Committed probe for IN-13's faction-aware match, scarcity price and
## tariff (Rulings AB/AE, `LARGE_ITEM_RULINGS.md`).
##
## The load-bearing check is PARITY, against a real generated world:
## `civ_trade_flows()`'s pre-existing keys (every flow's from/to/good/mode/
## reach/distance/deliverable/volume, the per-way load, the unmet list and
## the per-good totals) hash to a digest, and that digest is printed for
##   A  -- the default 6-faction world, no tariff set,
##   B  -- the same terrain re-populated with `civ.factions = 1`.
## The two digests were recorded against the extension built from a clean
## `git worktree` at `842faf5` (before this change; two runs, identical) and
## are asserted as literals below, so the check fails if the faction-aware
## match moves a single bit of the old output.
##
## Then, only when the extension has the tariff surface:
##   * a tariff on one importer/exporter faction pair strictly lowers the
##     volume of every flow that crosses that pair, leaves every other flow
##     bit-identical, and never raises the total;
##   * clearing it restores digest A exactly.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _in13_tariff_probe.tscn
##
## Headless is correct here: nothing below reads a pixel.

const DIGEST_A := "a3bea37d327bb8ebecf219acefe58aa2"  # 233 settlements, 928 flows, total 78029.033914
const DIGEST_B := "a548698d66bb190548825477afe0ce61"  # factions [1], 947 flows, total 46830.425123

var _fail := 0

func _p(s: String) -> void:
	print("IN13T  %s" % s)

func _bad(s: String) -> void:
	_fail += 1
	print("IN13T  FAIL  %s" % s)

func _ok(s: String) -> void:
	print("IN13T  ok    %s" % s)

## The pre-change keys only, in a fixed order, at full precision.
func _digest(d: Dictionary) -> String:
	var flows: Array = []
	for f in (d.get("flows", []) as Array):
		flows.append([f["from"], f["to"], f["good"], f["mode"], f["reach"],
			f["distance_km"], f["deliverable"], f["volume"]])
	var goods: Array = []
	for g in (d.get("goods", []) as Array):
		goods.append([g["key"], g["volume"], g["exporters"], g["importers"], g["dominant_mode"]])
	var unmet: Array = []
	for u in (d.get("unmet", []) as Array):
		unmet.append([u["index"], u["goods"], u["exporter_exists"]])
	var load: Array = []
	for v in (d.get("way_load", PackedFloat32Array()) as PackedFloat32Array):
		load.append(v)
	var blob := JSON.stringify([d.get("flow_count", -1), d.get("total_volume", -1.0),
		flows, goods, unmet, load], "", false, true)
	return blob.md5_text()

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 900.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	# The engine alone, not `shell/app.tscn`: this probe asserts nothing
	# about the shell, and must not fail because a concurrent lane has the
	# shell mid-edit. The calls are exactly what `engine_bridge.generate()`
	# makes for the request `_in13_probe.gd` sends, in the same order.
	var wg := WorldGen.new()
	wg.set_experimental_flags(false, false, false, false)
	wg.set_villages_enabled(true)
	wg.set_metropolis_enabled(false)
	wg.set_recovery_phase(0)
	wg.set_biome_k_enabled(false)
	wg.set_sea_level(0.45)
	wg.generate_sized(483920, 2400.0, 384, 288)

	# ---- A: the default multi-faction world, no tariff ----
	var a: Dictionary = wg.civ_trade_flows()
	var da := _digest(a)
	_p("A  %d settlements, %d flows, total %.6f, digest %s" % [
		int(a.get("settlement_count", 0)), int(a.get("flow_count", 0)),
		float(a.get("total_volume", 0.0)), da])
	if int(a.get("flow_count", 0)) == 0:
		_bad("A has no flows -- the parity check would be vacuous")
	if da == DIGEST_A:
		_ok("A digest matches the pre-change build")
	else:
		_bad("A digest %s != recorded %s" % [da, DIGEST_A])

	var has_tariff: bool = wg.has_method("civ_set_trade_tariff")
	if has_tariff:
		await _tariff(wg, a, da)
	else:
		_p("extension has no tariff surface -- recording-only run")

	# ---- B: one faction ----
	wg.set_params({"civ.factions": 1})
	wg.civ_populate()
	var b: Dictionary = wg.civ_trade_flows()
	var db := _digest(b)
	var fs := {}
	for s in wg.get_settlements():
		fs[int((s as Dictionary).get("faction", -1))] = true
	_p("B  factions present %s, %d flows, total %.6f, digest %s" % [
		str(fs.keys()), int(b.get("flow_count", 0)), float(b.get("total_volume", 0.0)), db])
	if int(b.get("flow_count", 0)) == 0:
		_bad("B has no flows -- the parity check would be vacuous")
	if db == DIGEST_B:
		_ok("B digest matches the pre-change build")
	else:
		_bad("B digest %s != recorded %s" % [db, DIGEST_B])
	if has_tariff:
		for f in (b.get("flows", []) as Array):
			if int(f["from_faction"]) != int(f["to_faction"]):
				_bad("B: a one-faction world produced a cross-faction flow")
				break

	_p("=== %s ===" % ("PASS" if _fail == 0 else "%d FAILURES" % _fail))
	get_tree().quit(0 if _fail == 0 else 1)


func _tariff(wg, a: Dictionary, da: String) -> void:
	_p("=== tariff ===")
	var flows: Array = a["flows"]
	# The busiest cross-faction pair, so the effect is on real volume.
	var pick := {}
	var best := -1.0
	var cross := 0
	for f in flows:
		if int(f["from_faction"]) != int(f["to_faction"]) and int(f["to_faction"]) > 0:
			cross += 1
			if float(f["volume"]) > best:
				best = float(f["volume"])
				pick = f
	_p("%d of %d flows cross factions" % [cross, flows.size()])
	if pick.is_empty():
		_bad("no cross-faction flow on a 6-faction world -- nothing to tariff")
		return
	var imp := int(pick["to_faction"])
	var exp := int(pick["from_faction"])
	for f in flows:
		if float(f["price"]) <= 0.0 or float(f["price"]) >= 2.0:
			_bad("price %s outside (0, 2)" % str(f["price"]))
			break
		if float(f["tariff"]) != 0.0:
			_bad("a tariff applied with none set")
			break

	if not wg.civ_set_trade_tariff(imp, exp, 0.25):
		_bad("civ_set_trade_tariff(%d, %d, 0.25) refused" % [imp, exp])
		return
	if absf(float(wg.civ_trade_tariff(imp, exp)) - 0.25) > 1e-12:
		_bad("civ_trade_tariff did not read back 0.25")
	if float(wg.civ_trade_tariff(exp, imp)) != 0.0:
		_bad("a tariff is directional; the reverse pair must stay 0")
	if wg.civ_set_trade_tariff(imp, imp, 0.25):
		_bad("a faction taxing itself was accepted")
	if wg.civ_set_trade_tariff(0, exp, 0.25):
		_bad("Unclaimed levying a tariff was accepted")

	var t: Dictionary = wg.civ_trade_flows()
	var key := func(f) -> String: return "%d>%d:%s" % [int(f["from"]), int(f["to"]), String(f["good"])]
	var before := {}
	for f in flows:
		before[key.call(f)] = f
	var hit := 0
	var same := 0
	for f in (t["flows"] as Array):
		var o: Dictionary = before.get(key.call(f), {})
		if o.is_empty():
			_bad("tariff created a flow %s" % key.call(f))
			continue
		var taxed := int(f["to_faction"]) == imp and int(f["from_faction"]) == exp
		if taxed:
			hit += 1
			if not (float(f["volume"]) < float(o["volume"])):
				_bad("taxed flow %s did not shrink: %s -> %s" % [key.call(f), o["volume"], f["volume"]])
			if absf(float(f["volume"]) - 0.75 * float(o["volume"])) > 1e-9 * float(o["volume"]):
				_bad("taxed flow %s is not 0.75x" % key.call(f))
			if not (float(f["value"]) < float(o["value"])):
				_bad("taxed flow %s value did not shrink" % key.call(f))
		else:
			if float(f["volume"]) != float(o["volume"]) or float(f["price"]) != float(o["price"]):
				_bad("untaxed flow %s moved" % key.call(f))
			else:
				same += 1
	if (t["flows"] as Array).size() != flows.size():
		_bad("flow count moved under a tariff: %d -> %d" % [flows.size(), (t["flows"] as Array).size()])
	if not (float(t["total_volume"]) < float(a["total_volume"])):
		_bad("total volume did not fall: %f -> %f" % [a["total_volume"], t["total_volume"]])
	_p("tariff %d<-%d at 0.25: %d flows taxed, %d untouched, total %.3f -> %.3f" % [
		imp, exp, hit, same, float(a["total_volume"]), float(t["total_volume"])])
	if hit > 0:
		_ok("a tariff strictly lowers every flow it applies to and nothing else")

	wg.civ_set_trade_tariff(imp, exp, 0.0)
	var back := _digest(wg.civ_trade_flows())
	if back == da:
		_ok("clearing the tariff restores digest A exactly")
	else:
		_bad("cleared tariff digest %s != A %s" % [back, da])

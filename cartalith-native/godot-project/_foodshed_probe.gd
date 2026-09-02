extends Node
## Committed probe for `ECONOMY_SCOPE.md` milestone 2's **UI surface** -- the
## food-shed readout `place_editor_window.gd::_food_shed_note` draws in the
## Trade tab, beside the navigability row `_in13_probe.gd` already covers.
##
## Drives the four branches a `--check-only` parse cannot reach. Each is its own
## `%` format string, and a wrong argument count in one is a *runtime* error
## GDScript reports only on the frame that line executes -- so a parse-clean
## file can still crash the place editor the first time a settlement is over
## its ceiling:
##   * no row at all      -> the em dash **and its reason**, never a blank
##                           (`LARGE_ITEM_RULINGS.md`'s standing condition)
##   * supplied, in budget -> the full breakdown, thousands-separated
##   * no reachable supplier -> the "nothing imported" clause, with `best_mode`
##                           suppressed (the engine leaves it `land` whether or
##                           not anything actually shipped -- `FoodShed`'s own
##                           field doc)
##   * over the ceiling   -> the overshoot line and its not-a-correction caveat
##
## Runs against a synthetic `TradeStore` row rather than a generated world, and
## deliberately: the engine half of milestone 2 is golden-tested inside
## `cartalith-civ` (`civ_food_shed`'s own tests), the bridge half is covered by
## `_in13_probe.gd`'s live match, and what is new here is only the presentation.
## No generate, no engine call, no .dll dependency -- under a second.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _foodshed_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them.

var _fail := 0

func _p(s: String) -> void:
	print("FOODSHED  %s" % s)

func _bad(s: String) -> void:
	_fail += 1
	print("FOODSHED  FAIL  %s" % s)

func _ok(s: String) -> void:
	print("FOODSHED  ok    %s" % s)

## One `civ_food_shed()` row, in the exact shape `civ_trade_bridge.rs`'s
## `civ_food_shed` emits (its doc comment lists every key). The capacities carry
## fractions on purpose, so the readout's `round()` is exercised in both
## directions: 8100.4 must render 8 100 and 3200.6 must render 3 201.
const ROW := {
	"index": 0, "name": "Testholm", "pop": 11900,
	"local_capacity": 8100.4, "hinterland_capacity": 3200.6,
	"import_capacity": 1100.0, "supported": 12401.0,
	"suppliers": 4, "best_mode": "river", "limited_by": "trade",
	"sustainable": true, "over_by": 0.0,
}


## Render the readout against one synthetic row -- `null` for "the pass returned
## nothing", which is what an engine build without the binding produces -- and
## return the prose it drew.
func _render(row: Variant) -> String:
	TradeStore._food_shed = {} if row == null else {"rows": [row]}
	var host := VBoxContainer.new()
	add_child(host)
	## Never `setup()`: `_food_shed_note` reads `_index` and nothing else off
	## the window, so building the whole dialog (and with it a bridge, an app
	## and a phone-fit pass) would test the shell, not this readout.
	var w := PlaceEditorWindow.new()
	w._index = 0
	w._food_shed_note(host)
	var parts: Array[String] = []
	for n in host.get_children():
		if n is Label:
			parts.append((n as Label).text)
	w.free()
	host.free()
	return "\n".join(parts)


func _expect(what: String, text: String, needles: Array) -> void:
	var missing: Array = []
	for needle in needles:
		if not (String(needle) in text):
			missing.append(needle)
	if missing.is_empty():
		_ok("%s: %s" % [what, text.replace("\n", " / ")])
	else:
		_bad("%s: missing %s -- drew %s" % [what, JSON.stringify(missing),
			JSON.stringify(text)])


func _refuse(what: String, text: String, needle: String) -> void:
	if needle in text:
		_bad("%s: '%s' should not appear -- drew %s" % [what, needle, JSON.stringify(text)])
	else:
		_ok("%s: '%s' correctly absent" % [what, needle])


func _ready() -> void:
	var saved := TradeStore.food_shed()

	## 1. No row. The one case the owner's standing condition is about: dashed
	## with its reason, never blank and never silently skipped.
	var t := _render(null)
	_expect("no row", t, ["—", "civ_food_shed()", "ECONOMY_SCOPE.md"])
	if t.strip_edges().is_empty():
		_bad("no row: drew nothing at all -- a blank field is exactly what is forbidden")

	## 2. The ordinary case: four figures, thousands-separated, both rounding
	## directions, the supplier count and its best mode, and the ceiling met.
	t = _render(ROW.duplicate())
	_expect("supplied", t, [
		"Food shed: supports 12 401 people",
		"8 100 from its own catchment",
		"3 201 from the countryside within land reach",
		"1 100 imported from 4 settlements over river",
		"Limited by trade",
		"Population 11 900 when the match ran, inside that ceiling.",
	])

	## 3. One supplier, so the plural really is conditional.
	var one := ROW.duplicate()
	one["suppliers"] = 1
	one["import_capacity"] = 300.0
	_expect("one supplier", _render(one), ["300 imported from 1 settlement over river"])

	## 4. No supplier reachable. `best_mode` is `land` here whatever happened,
	## so naming a mode would be an invention -- it must not be reported.
	var alone := ROW.duplicate()
	alone["suppliers"] = 0
	alone["import_capacity"] = 0.0
	alone["best_mode"] = "land"
	alone["limited_by"] = "local"
	t = _render(alone)
	_expect("no supplier", t, ["nothing imported", "no settlement with spare capacity is in reach",
		"Limited by local"])
	_refuse("no supplier", t, "over land")

	## 5. Over the ceiling. The branch that would have crashed the editor on a
	## bad format string, since a sustainable world never reaches it.
	var over := ROW.duplicate()
	over["pop"] = 14000
	over["sustainable"] = false
	over["over_by"] = 1599.0
	_expect("unsustainable", _render(over), [
		"Population 14 000 when the match ran, over the ceiling by 1 599.",
		"A diagnostic, not a correction",
		"_civApplyFoodShedCeilings is not ported",
	])

	TradeStore._food_shed = saved
	_p("=== %s ===" % ("PASS" if _fail == 0 else "%d FAILURES" % _fail))
	get_tree().quit(0 if _fail == 0 else 1)

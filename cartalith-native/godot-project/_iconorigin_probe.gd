extends Node
## Owner ruling 14's engine half, checked where it actually has to arrive:
## **does `icon_list()`'s Dictionary carry `origin` across the gdext boundary,
## and does it carry the right value for each of the two producers?**
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _iconorigin_probe.tscn
##
## Logic only, no pixels, so `--headless` is right here and wrong for
## `_iconmerge_probe.gd` next door (`MISTAKES.md`: headless for logic and
## layout, windowed for anything that rasterises).
##
## `WorldGen` is a cdylib `GodotClass` and cannot be constructed in a Rust unit
## test, and `VarDictionary` needs the Godot runtime, so `lib.rs::icon_dict` has
## no reachable `#[cfg(test)]` coverage at all -- a probe is the only thing that
## can grade it. The `IconOrigin` values themselves are unit-tested next door in
## `icon_bridge.rs`; what is untestable in Rust, and tested here, is the
## marshalling.
##
## Both producers are exercised:
##   * `icon_generate({"family": "POI"})` -> `IconEditor::generate`, the only
##     writer of `IconOrigin::Generated`;
##   * `icon_place` after arming, -> `place_manual_icon`, which writes `Manual`.
##     Arming needs an asset pack (`icon_arm`'s own `has_asset_pack()` gate,
##     `GUI_GAP_REGISTER.md` CA-12), so the reference fixture pack is loaded
##     first. If that load fails the manual half is reported SKIPPED by name
##     rather than silently passing over zero rows.

const PACK := "res://../crates/cartalith-assets/tests/fixtures/reference_pack.zip"

var _fails := 0


func _p(s: String) -> void:
	print("ICONORIGIN  %s" % s)


func _bad(s: String) -> void:
	_fails += 1
	_p("FAIL  %s" % s)


func _ready() -> void:
	var g := WorldGen.new()
	g.set_sea_level(0.45)
	g.set_villages_enabled(true)
	g.generate_sized(483920, 2400.0, 256, 160)
	_p("grid %d x %d" % [g.get_width(), g.get_height()])

	var lr: bool = g.landmark_run()
	var lms: Array = g.landmarks()
	_p("landmark_run=%s  landmarks=%d" % [lr, lms.size()])
	if lms.is_empty():
		_bad("no landmarks: the POI pass would place nothing and every "
			+ "assertion below would be vacuous")
		get_tree().quit(1)
		return

	var rep: Dictionary = g.icon_generate({"family": "POI"})
	_p("icon_generate(POI) -> %s" % rep)
	var placed := int(rep.get("placed", 0))
	if placed <= 0:
		_bad("the POI pass placed nothing -- nothing to check the origin of")

	var rows: Array = g.icon_list()
	if rows.is_empty():
		_bad("icon_list() is empty after a pass that reported placed=%d" % placed)
		get_tree().quit(1)
		return
	if not (rows[0] as Dictionary).has("origin"):
		_bad("icon_list()'s rows carry NO `origin` key: %s" % rows[0])
		get_tree().quit(1)
		return

	var gen := 0
	var man := 0
	var other := []
	for r: Dictionary in rows:
		match String(r.get("origin", "")):
			"generated": gen += 1
			"manual": man += 1
			_: other.append(r.get("origin", "<absent>"))
	_p("after POI: rows=%d generated=%d manual=%d" % [rows.size(), gen, man])
	if not other.is_empty():
		_bad("origin values that are neither string of IconOrigin::key(): %s" % [other])
	if gen != rows.size():
		_bad("every row here came from the generated pass; %d of %d say so" % [gen, rows.size()])
	if gen != placed:
		_bad("generate reported placed=%d, icon_list shows %d generated" % [placed, gen])
	_p("sample generated row: %s" % rows[0])

	## The other producer. `icon_arm` refuses without a pack, so a failed load
	## must be reported, not worked around.
	if not g.load_asset_pack(ProjectSettings.globalize_path(PACK)):
		_p("SKIPPED  the manual half: %s did not load, so `icon_arm` is gated shut"
			% PACK)
	elif not g.icon_arm("feature", 0, 1.0, 0.0, 0.0):
		_bad("icon_arm refused with a pack loaded")
	else:
		var idx: int = g.icon_place(12.0, 9.0)
		if idx < 0:
			_bad("icon_place refused at (12, 9) on a 256x160 grid")
		else:
			var one: Dictionary = g.icon_get(idx)
			_p("hand-placed row %d: %s" % [idx, one])
			if String(one.get("origin", "<absent>")) != "manual":
				_bad("a click-placed icon reports origin=%s, not \"manual\""
					% one.get("origin", "<absent>"))
			var after: Array = g.icon_list()
			var man2 := 0
			for r: Dictionary in after:
				if String(r.get("origin", "")) == "manual":
					man2 += 1
			if man2 != 1:
				_bad("exactly one hand-placed icon exists; icon_list counts %d" % man2)
			_p("after one click: rows=%d manual=%d" % [after.size(), man2])

	_p("---- %s ----" % ("PASS" if _fails == 0 else "%d FAILED" % _fails))
	get_tree().quit(1 if _fails > 0 else 0)

extends Node
## Protects: the Journey planner's trace SOURCE column (`journey_planner_view.gd::_trace_source`) against a
## silent gap -- every term key `cartalith-civ`'s `jp_calc_land` / `jp_calc_water` emit (the `term("<key>", ...)`
## calls in `cartalith-civ/src/lib.rs`, swept 2026-10-12) must have a `_TRACE_TERMS` row, so no leg's source cell
## reads "no source mapped for <key>". The list below is that sweep, written out by hand: a NEW engine term is
## not caught by this probe until it is added here (the fallback string in `_trace_source` is what exposes it in
## the product). Run: godot --headless --path <scratch copy> res://_tracekeys_probe.tscn
## Exit 0 = held; 1 = failed.

const _EMITTED_KEYS := ["base", "hours", "terrain", "route", "slope", "group", "pace", "infra", "weather",
	"fatigue", "grazing", "foraging", "desert water", "column", "load", "sailing window"]

func _ready() -> void:
	var v = load("res://shell/journey_planner_view.gd").new()
	var bad := PackedStringArray()
	for k in _EMITTED_KEYS:
		var src: String = v._trace_source(k, 0, {})
		if src.begins_with("— no source mapped"):
			bad.append(k)
	# Positive control: a key no engine emits must hit the fallback, or the check above is vacuous.
	var ctl: String = v._trace_source("__not_a_term__", 0, {})
	var ok := bad.is_empty() and ctl.begins_with("— no source mapped") and _EMITTED_KEYS.size() == 16
	print("[tracekeys] unmapped=%s control=%s keys=%d -> %s" % [str(bad), ctl, _EMITTED_KEYS.size(), "OK" if ok else "FAIL"])
	v.free()
	get_tree().quit(0 if ok else 1)

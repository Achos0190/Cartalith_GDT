extends EngineBridge
## Probe-only: an EngineBridge that reports the ET-D4 way-stroke API absent, to drive the
## shell's click-path fallback. Shares the real bridge's WorldGen (assigned by the probe).
## Result: a helper class, not a runnable probe: it prints nothing and never quits. It extends `EngineBridge` and
## overrides `has_way_stroke_api()` to return false so that `_etd4_probe.gd` (which loads it with
## `load("res://_etd4_noapi_bridge.gd")` and calls `.new()`) can drive the shell's click-path fallback. No `.tscn` exists
## for it.
func has_way_stroke_api() -> bool:
	return false

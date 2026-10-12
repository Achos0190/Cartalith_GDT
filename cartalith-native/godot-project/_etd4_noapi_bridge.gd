extends EngineBridge
## Probe-only: an EngineBridge that reports the ET-D4 way-stroke API absent, to drive the
## shell's click-path fallback. Shares the real bridge's WorldGen (assigned by the probe).
func has_way_stroke_api() -> bool:
	return false

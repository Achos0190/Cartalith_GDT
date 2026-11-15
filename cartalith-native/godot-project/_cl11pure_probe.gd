extends SceneTree
## CL-11 and the marker-style request, the parts that need no window: `PinMarker`'s circle footprint and its style
## dispatch, the POI pictogram table, and `DccSettings`' marker style. Headless-safe (no pixels, no engine), so a mutant of any of them is
## killed in seconds.
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . --script res://_cl11pure_probe.gd
##
## Protects: a circle's footprint is its radius plus the pad and nothing else (a pin's head is NOT a circle hit); the style
## decides which footprint a hit uses and which distance ranks it; every POI kind has the pictogram the owner's list implies
## (`other` the stated dot); the marker style defaults to pins, refuses an unknown value, and reads back what was stored.
## Exit 0 = held, 1 = a check failed.

const Pin := preload("res://shell/pin_marker.gd")

var _checks := 0
var _fails := 0


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("CL11PURE %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _init() -> void:
	var c := Vector2(100.0, 100.0)
	var r := 9.0
	var pad := 2.0
	## The circle footprint: radius plus pad, measured from the centre.
	_ok("ring: the centre hits", Pin.hit_ring(c, r, c, pad))
	_ok("ring: just inside r + pad hits", Pin.hit_ring(c, r, c + Vector2(r + pad - 0.1, 0.0), pad))
	_ok("ring: just outside r + pad misses", not Pin.hit_ring(c, r, c + Vector2(r + pad + 0.1, 0.0), pad))
	_ok("ring: the pad matters (r + 1.5 hits with pad 2, misses with pad 0)", Pin.hit_ring(c, r, c + Vector2(r + 1.5, 0.0), 2.0)
		and not Pin.hit_ring(c, r, c + Vector2(r + 1.5, 0.0), 0.0))
	## A pin and a ring over the same anchor are different shapes.
	var head := c + Vector2(0.0, -Pin.STEM_RATIO * r)
	var beside := c + Vector2(r * 0.8, 0.0)
	_ok("pin: its head centre hits, a point 0.8 r beside the tip misses", Pin.hit_styled(c, r, head, pad, false) and not Pin.hit_styled(c, r, beside, pad, false))
	_ok("circle: the point 0.8 r beside the centre hits, the pin's head position misses", Pin.hit_styled(c, r, beside, pad, true) and not Pin.hit_styled(c, r, head, pad, true))
	## The rank distance: the centre in circles, the axis in pins.
	_ok("rank: circles measure to the centre", is_equal_approx(Pin.rank_distance(c, r, head, true), Pin.STEM_RATIO * r))
	_ok("rank: pins measure to the axis (0 on the head centre)", is_equal_approx(Pin.rank_distance(c, r, head, false), 0.0))
	## Pictograms: the owner's list.
	var want := {"tavern": "mug", "shop": "shop", "smithy": "anvil", "temple": "spire", "shrine": "spire", "guild": "banner",
		"gate": "gate", "well": "well", "ruin": "ruin", "camp": "tent", "mine": "pick", "other": "dot"}
	var bad: Array = []
	for k in want:
		if Pin.pictogram_for(k) != want[k]:
			bad.append("%s=%s" % [k, Pin.pictogram_for(k)])
		if not Pin.PICTOGRAMS.has(Pin.pictogram_for(k)):
			bad.append("%s has no drawn pictogram" % k)
	_ok("pictograms: every POI kind maps to the drawn pictogram it should (other is the stated dot)", bad.is_empty(), str(bad))
	_ok("pictograms: only `other` is the unmapped fallback", not Pin.is_mapped("other") and Pin.is_mapped("tavern") and Pin.is_mapped("smithy"))
	## The marker style: default, refusal, round trip.
	DccSettings._loaded = true
	DccSettings._cfg = ConfigFile.new()
	_ok("style: unset reads as pins, the default", DccSettings.marker_style() == "pins" and not DccSettings.markers_are_circles())
	DccSettings._cfg = ConfigFile.new()
	DccSettings._cfg.set_value("marker_style", "style", "squares")
	_ok("style: a corrupt stored value reads as pins", DccSettings.marker_style() == "pins")
	DccSettings._cfg = ConfigFile.new()
	DccSettings._cfg.set_value("marker_style", "style", "circles")
	_ok("style: circles reads back and the one test says so", DccSettings.marker_style() == "circles" and DccSettings.markers_are_circles())
	DccSettings._cfg.set_value("marker_style", "style", "pins")
	_ok("style: pins reads back", DccSettings.marker_style() == "pins" and not DccSettings.markers_are_circles())
	_ok("style: exactly four styles, in this order (icon sets added 2026-11-15)", DccSettings.MARKER_STYLES == ["pins", "circles", "icons", "painted"])
	print("CL11PURE ---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	quit(1 if _fails > 0 else 0)

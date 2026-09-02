extends RefCounted
class_name DccUnits

## Distance-unit conversion and display formatting -- `DCC_SHELL_SPEC.md` §2.5's
## Preferences ▸ Application ▸ Units row (`km · mi`, `#calUnitSeg`), extended to
## three per the owner's ruling on `OUTSTANDING_WORK.md`'s PR-15
## (`LARGE_ITEM_RULINGS.md`: *"Build, and add nautical miles"*) -- a deliberate
## deviation from the spec's own two-choice table, recorded here rather than
## made silently (`CLAUDE.md`).
##
## **Canonical storage stays km; this only converts what a readout shows.**
## The reference draws exactly this line itself -- v0.67's own comment:
## *"units: display-only. Canonical storage stays km (state.mapWidthKm) ...
## this only converts what the scale UI shows"* (reference 13711-13713) -- and
## every `_km` value this shell's engine hands back (`EngineBridge`, `render.rs`'s
## planar CRS) stays km internally, saved and sent back to the engine
## unconverted. `DccSettings.units_mode()` decides only what a formatter
## prints.
##
## **Round only at display.** `to_unit()` is one division, not a rounding
## step; every decimal place below is applied inside a format string as the
## very last thing that happens to the number, so nothing here rounds a value
## a caller might still do arithmetic on afterwards.
##
## Conversion factors: the reference's own where it has one, extended where
## it does not.
## - `KM_PER_MI := 1.609344` -- the reference's own `KM_PER_MI` (13714), the
##   exact international mile (1 mi = 1 609.344 m by definition).
## - `KM_PER_NMI := 1.852` -- new for this port; the reference offers no
##   nautical-mile toggle. The exact international nautical mile, 1 NM =
##   1 852 m by definition, taken directly rather than composed through miles
##   (which would stack two roundings into one).

const KM_PER_MI := 1.609344
const KM_PER_NMI := 1.852

const _SUFFIX := {"km": "km", "mi": "mi", "nmi": "nm"}
const _LABEL := {"km": "Kilometres", "mi": "Miles", "nmi": "Nautical miles"}

## `km` in whatever `DccSettings.units_mode()` currently holds -- the one
## division every formatter below builds on.
##
## Named `to_unit`, not `convert`: `@GlobalScope` already has a `convert()`
## builtin (value/type coercion), and an unqualified call inside a `static`
## function here resolves to that one, not this one -- caught by
## `godot --headless --check-only`, which reported "Too few arguments for
## convert()" pointing at this file's own call sites.
static func to_unit(km: float) -> float:
	match DccSettings.units_mode():
		"mi": return km / KM_PER_MI
		"nmi": return km / KM_PER_NMI
		_: return km

## `"km"` / `"mi"` / `"nm"` -- the abbreviation a readout appends. `"nm"`
## rather than `"nmi"` on screen: it is the unit's own standard symbol, the
## way `"mi"` and `"km"` already are, and three legible characters beat four.
static func suffix() -> String:
	return String(_SUFFIX.get(DccSettings.units_mode(), "km"))

## `"Kilometres"` / `"Miles"` / `"Nautical miles"` -- the three radio-row
## labels `menus.gd`'s Units submenu draws, kept here so the label and the
## conversion it names cannot drift into two lists that disagree.
static func label(mode: String) -> String:
	return String(_LABEL.get(mode, mode))

## Fixed decimals, unit suffix included -- `"62 mi"` at `decimals = 0`. The
## plain case: the caller already knows how many decimal places it wants and
## only needs the value converted and the right word appended.
static func format(km: float, decimals: int = 0) -> String:
	return "%.*f %s" % [decimals, to_unit(km), suffix()]

## The three-tier adaptive precision `viewport_host.gd`'s own scale-bar
## formatter used before this file existed, generalised: 2 decimals under 10,
## 1 under 100, 0 above -- so a converted value stays informative at deep zoom
## (a flat `%.0f` would print "5 km" for everything from 4.5 to 5.5 at the
## camera's own cap) without turning a three-figure span into a wall of
## decimals. Applied to the CONVERTED value, not the km one: 100 km is 62 mi
## is 54 nm, and each unit earns its own answer to "is this small yet".
static func format_adaptive(km: float) -> String:
	var v := to_unit(km)
	var mag := absf(v)
	var decimals := 0
	if mag < 10.0:
		decimals = 2
	elif mag < 100.0:
		decimals = 1
	return "%.*f %s" % [decimals, v, suffix()]

## Space-grouped thousands, no decimals, unit suffix included -- the
## `"4 812 km"` shape a large odometer-style figure wants (the cursor
## coordinate readout's own east/north distances). Grouped on the CONVERTED
## integer part: 4 812 km is 2 990 mi, and each earns its own thousands
## separator rather than inheriting the km one's.
static func format_thousands(km: float) -> String:
	var v := to_unit(km)
	var s := "%.0f" % v
	var neg := s.begins_with("-")
	if neg:
		s = s.substr(1)
	var out := ""
	var count := 0
	for i in range(s.length() - 1, -1, -1):
		out = s[i] + out
		count += 1
		if count % 3 == 0 and i > 0:
			out = " " + out
	return "%s%s %s" % ["-" if neg else "", out, suffix()]

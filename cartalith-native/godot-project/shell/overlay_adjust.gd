extends RefCounted
## ET-C3 (`EDITING_TOOLS_SCOPE.md` section 5; Ruling BX item 4): the third per-layer
## adjustment stack, for labels and overlays -- the GDScript half of milestone C.
##
## # What this file is
##
## The terrain and water stacks run in Rust inside `finish_rgb` (ET-C2, `render::adjust`).
## The overlay layer is different in kind: every ink on it is a Godot `Color` resolved at
## draw time by `map_overlay.gd` (roads, sea lanes, labels, icons, journey markers, faction
## pins) or by `viewport_host.gd` (the province-border shader), and `map_overlay.gd` is by
## its own header "handed finished data" and never calls the bridge. So the same stack
## semantics run HERE, on the same ink, by an independent port of `crates/cartalith-godot/
## src/adjust.rs`: Levels, Curves (monotone cubic Hermite, Fritsch-Carlson), Hue/Saturation/
## Lightness in OkLCh and Gradient map. This is the `river_field.rs` / `map_shore.gdshader`
## precedent: two implementations of one formula, pinned to each other by a probe
## (`_etc3_probe.gd`, section C) that drives the REAL Rust stack through the bridge as the
## oracle, over 64 colours per kind.
##
## # The one seam
##
## The stack is held in **script-level static state**, not on a node, for one reason: the
## overlay is drawn by more than one `MapOverlay` instance (the shell's, and any export
## `SubViewport` that instances its own), and an export must draw the ink the screen
## draws. State on a node would have to be pushed into every instance; static state is
## read by all of them, and `revision` is how each instance notices a change and drops its
## own caches (`map_overlay.gd::_sync_overlay_adjust`).
##
## ET-C4 will persist this exactly as `AdjustStack`'s saved form: `get_stack_json()` writes
## `{"entries":[...]}` in the shape serde writes (`kind` externally tagged by a snake_case
## name, `enabled`, `opacity`), and `set_stack_json()` reads what `appearance_get_adjust`
## returns, so one document describes a layer whichever side it lives on.
##
## # What it must never do
##
## * **Encode "no adjustment" as an identity multiply.** An empty stack, a disabled entry,
##   an opacity of exactly 0 and a parameter set at its neutral values are all SKIPPED by
##   control flow (`is_live()`, `_entry_skipped()`): `color()` returns its argument
##   untouched -- the same object, no arithmetic -- so every ink is byte-identical to the
##   shell before this file existed (MISTAKES.md, "Change generated output").
## * **Touch the engine.** Nothing here calls the bridge or `world_gen`, so it cannot be
##   reached by the "no engine call while `bridge.generating`" rule (MISTAKES.md, gating
##   engine calls); the probe asserts that this file's source names neither.
## * **Clamp silently.** `set_stack_json()` refuses an out-of-range or non-finite parameter,
##   an unknown kind or a malformed document with `{"ok": false, "error": ...}` and
##   changes nothing, as `appearance_set_adjust` does.
## * **Judge legibility.** This is the maths. The contrast guard that keeps a stack from
##   making a label unreadable or two factions alike lives in `map_overlay.gd`, next to the
##   WCAG and CIE76 helpers Ruling BX item 2's guard already uses.
## * **Change alpha.** The stack is RGB only, as in Rust; `color()` returns the input alpha.

## Most entries one stack may hold: `adjust.rs::MAX_ENTRIES`, a labelled judgement there.
const MAX_ENTRIES := 32
## Fewest and most Curves control points: `adjust.rs::MIN_CURVE_POINTS` / `MAX_CURVE_POINTS`.
const MIN_CURVE_POINTS := 2
const MAX_CURVE_POINTS := 16
## Levels gamma range: `adjust.rs::GAMMA_RANGE`, a labelled judgement there.
const GAMMA_MIN := 0.1
const GAMMA_MAX := 10.0
## Hue rotation range in degrees: `adjust.rs::HUE_RANGE_DEG`.
const HUE_MAX := 360.0
## Rec.709 luma weights, `adjust.rs::LUMA_WEIGHTS` (ITU-R BT.709), applied to the
## gamma-encoded 0..255 channels exactly as the Rust gradient map does.
const LUMA_R := 0.2126
const LUMA_G := 0.7152
const LUMA_B := 0.0722
## The most distinct source colours `color()` remembers before it empties the memo and
## starts over. A labelled judgement: a map carries a few dozen distinct inks (five label
## classes, four icon families, six factions, the road rows, the preset's pair), so 4096 is
## two orders of magnitude of headroom, and the bound only exists so a pathological caller
## (a ramp of per-pixel colours) cannot grow it without limit.
const MAX_CACHE := 4096

## Ottosson 2020, "A perceptual color space for image processing", linear sRGB to LMS --
## the same 10-decimal values as `adjust.rs::LINEAR_SRGB_TO_LMS`.
const LINEAR_SRGB_TO_LMS := [
	[0.4122214708, 0.5363325363, 0.0514459929],
	[0.2119034982, 0.6806995451, 0.1073969566],
	[0.0883024619, 0.2817188376, 0.6299787005],
]
## Cube-rooted LMS to OkLab (`adjust.rs::LMS_CUBE_ROOT_TO_OKLAB`).
const LMS_CUBE_ROOT_TO_OKLAB := [
	[0.2104542553, 0.7936177850, -0.0040720468],
	[1.9779984951, -2.4285922050, 0.4505937099],
	[0.0259040371, 0.7827717662, -0.8086757660],
]
## The EXACT inverse of the matrix above, from `adjust.rs::OKLAB_TO_LMS_CUBE_ROOT` (computed
## there in exact rational arithmetic, not the post's 10-digit inverse, which is only an
## approximate inverse of the published forward matrix and would break the hue-360 identity).
const OKLAB_TO_LMS_CUBE_ROOT := [
	[0.9999999984505198, 0.39633779217376786, 0.2158037580607588],
	[1.0000000088817609, -0.10556134232365635, -0.06385417477170591],
	[1.0000000546724108, -0.08948418209496575, -1.2914855378640917],
]
## The exact inverse of `LINEAR_SRGB_TO_LMS` (`adjust.rs::LMS_TO_LINEAR_SRGB`).
const LMS_TO_LINEAR_SRGB := [
	[4.076741661347994, -3.3077115904081933, 0.2309699287294279],
	[-1.268438004092176, 2.6097574006633715, -0.3413193963102196],
	[-0.004196086541837109, -0.7034186144594496, 1.7076147009309448],
]

## Bumped on every successful `set_stack_json()` / `clear()`. A `MapOverlay` remembers the
## value it last built its caches against and rebuilds when this has moved.
static var revision: int = 0
## The validated entries, in order, each
## `{kind: String, p: Dictionary, enabled: bool, opacity: float, m: PackedFloat64Array}`
## (`m` only for curves: the tangents, computed once at set time).
static var _entries: Array = []
## The saved-form entries (what `get_stack_json()` writes), parallel to `_entries`.
static var _doc: Array = []
## True when at least one entry is live (not skipped): the control-flow switch every
## caller tests first.
static var _live: bool = false
## Memo of `color()`: source RGB (alpha forced to 1, so alpha never splits an entry) ->
## adjusted RGB. Emptied on every stack change.
static var _cache: Dictionary = {}


# -- the seam ---------------------------------------------------------------------------

## True when the stack would change at least one colour (some entry is enabled, above
## opacity 0 and off its neutral parameters). The caller's cheap first test: `false` means
## "do nothing at all", which is what makes the empty stack byte-identical by control flow.
static func is_live() -> bool:
	return _live


## Number of entries held, live or not (a stack of disabled entries is user data).
static func entry_count() -> int:
	return _entries.size()


## The stack in `AdjustStack`'s saved form, `{"entries":[...]}`. An empty stack is
## `{"entries":[]}`: it is ET-C4's job to omit that from a document; this seam reports what
## it holds.
static func get_stack_json() -> String:
	return JSON.stringify({"entries": _doc})


## Replaces the whole stack from `AdjustStack`'s saved form. Returns
## `{ok: true, entries, identity}` or `{ok: false, error}`; on failure NOTHING changes (no
## partial or clamped stack is ever stored). `{"entries":[]}` clears the layer.
static func set_stack_json(json: String) -> Dictionary:
	var parsed := _parse(json)
	if not bool(parsed["ok"]):
		return parsed
	var entries: Array = parsed["entries"]
	var doc: Array = parsed["doc"]
	var live := false
	for e in entries:
		if not _entry_skipped(e):
			live = true
			break
	_entries = entries
	_doc = doc
	_live = live
	_cache = {}
	revision += 1
	return {"ok": true, "entries": entries.size(), "identity": not live}


## Empties the stack (`{"entries":[]}`); the same path as `set_stack_json`, so the revision
## moves and every instance's caches are dropped.
static func clear() -> void:
	set_stack_json("{\"entries\":[]}")


## The adjusted colour for ink `c`, alpha untouched. **Returns `c` itself, with no
## arithmetic, when the stack is not live** -- identity by control flow. Memoised per source
## RGB: overlay inks are resolved per draw call, so the maths runs once per distinct source
## colour per stack change, not once per call.
static func color(c: Color) -> Color:
	if not _live:
		return c
	var key := Color(c.r, c.g, c.b, 1.0)
	var hit = _cache.get(key)
	if hit != null:
		return Color(hit.r, hit.g, hit.b, c.a)
	var v := apply255(PackedFloat64Array([c.r * 255.0, c.g * 255.0, c.b * 255.0]))
	var out := Color(clampf(v[0] / 255.0, 0.0, 1.0), clampf(v[1] / 255.0, 0.0, 1.0), clampf(v[2] / 255.0, 0.0, 1.0), 1.0)
	if _cache.size() >= MAX_CACHE:
		_cache = {}
	_cache[key] = out
	return Color(out.r, out.g, out.b, c.a)


## The whole stack over one colour `[r, g, b]` on the 0..255 scale, in continuous 64-bit
## floats with no clamp or rounding between or after entries -- `AdjustStack::apply`.
## Skipped entries are not touched. Colours are `PackedFloat64Array`s, **not `Vector3`**: a
## `Vector3` holds 32-bit floats, which would put 1e-5 of a LSB of noise into a port whose
## whole point is to equal the engine's 64-bit maths (found by `_etc3_probe.gd`'s anchors).
## Public because the probe's Rust-equality leg compares THIS to the engine's pixels before
## any clamp to a `Color` hides a difference.
static func apply255(c: PackedFloat64Array) -> PackedFloat64Array:
	var v := c
	for e in _entries:
		if _entry_skipped(e):
			continue
		var adjusted := _kind_apply(e, v)
		var op := float(e["opacity"])
		if op >= 1.0:
			v = adjusted
		else:
			v = _mix(v, adjusted, op)
	return v


## Per-channel `a + (b - a) * t` (`adjust.rs::mix3`, the same expression).
static func _mix(a: PackedFloat64Array, b: PackedFloat64Array, t: float) -> PackedFloat64Array:
	return PackedFloat64Array([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t])


# -- skip rules (control flow, never arithmetic) ----------------------------------------

## `Adjustment::is_skipped`: disabled, opacity <= 0, or parameters exactly neutral.
static func _entry_skipped(e: Dictionary) -> bool:
	return (not bool(e["enabled"])) or float(e["opacity"]) <= 0.0 or _neutral(e)


## `AdjustKind::is_neutral`, exact equality on purpose: a nearly-neutral set is a real
## adjustment and only the exact one is promised to be byte-identical. Gradient map is never
## neutral.
static func _neutral(e: Dictionary) -> bool:
	var p: Dictionary = e["p"]
	match e["kind"]:
		"levels":
			return p["in_black"] == 0.0 and p["in_white"] == 255.0 and p["gamma"] == 1.0 \
				and p["out_black"] == 0.0 and p["out_white"] == 255.0
		"curves":
			for pt in p["points"]:
				if pt[0] != pt[1]:
					return false
			return true
		"hsl":
			return p["hue_deg"] == 0.0 and p["saturation"] == 0.0 and p["lightness"] == 0.0
	return false


# -- the maths (a port of adjust.rs; see there for the sources) ------------------------

## One entry at full strength, ignoring enabled/opacity/neutrality. Unclamped.
static func _kind_apply(e: Dictionary, c: PackedFloat64Array) -> PackedFloat64Array:
	var p: Dictionary = e["p"]
	match e["kind"]:
		"levels":
			return PackedFloat64Array([_levels_channel(c[0], p), _levels_channel(c[1], p), _levels_channel(c[2], p)])
		"curves":
			var m: PackedFloat64Array = e["m"]
			var pts: Array = p["points"]
			return PackedFloat64Array([_curve_eval(pts, m, c[0]), _curve_eval(pts, m, c[1]), _curve_eval(pts, m, c[2])])
		"hsl":
			return _hsl_oklch(c, float(p["hue_deg"]), float(p["saturation"]), float(p["lightness"]))
		"gradient_map":
			var y := (LUMA_R * c[0] + LUMA_G * c[1] + LUMA_B * c[2]) / 255.0
			var s := _ramp_sample(p["stops"], String(p["mode"]), clampf(y, 0.0, 1.0))
			# An empty ramp is refused at set time; leave the colour rather than index out of range.
			if s.is_empty():
				return c
			return _mix(c, PackedFloat64Array([s[0], s[1], s[2]]), float(s[3]))
	return c


## One Levels channel: `t = clip((x - in_black)/(in_white - in_black), 0, 1)`,
## `t^(1/gamma)`, `out_black + t*(out_white - out_black)`. The `t` clip is the operator's
## input range, not a byte clamp.
static func _levels_channel(x: float, p: Dictionary) -> float:
	var t := clampf((x - float(p["in_black"])) / (float(p["in_white"]) - float(p["in_black"])), 0.0, 1.0)
	t = pow(t, 1.0 / float(p["gamma"]))
	return float(p["out_black"]) + t * (float(p["out_white"]) - float(p["out_black"]))


## Fritsch-Carlson monotone tangents (Fritsch & Carlson 1980, SIAM J. Numer. Anal. 17(2)):
## interior tangent is the mean of the two secants, or 0 where they differ in sign or either
## is 0; end tangents are the end secants; then each interval's (alpha, beta) is scaled onto
## the radius-3 circle when `alpha^2 + beta^2 > 9`. The 9 and 3 are the paper's
## monotonicity region (Theorem 2), not tunables -- as in `adjust.rs::pchip_tangents`.
static func _pchip_tangents(pts: Array) -> PackedFloat64Array:
	var n := pts.size()
	var d := PackedFloat64Array()
	d.resize(n)
	var m := PackedFloat64Array()
	m.resize(n)
	for i in n - 1:
		d[i] = (float(pts[i + 1][1]) - float(pts[i][1])) / (float(pts[i + 1][0]) - float(pts[i][0]))
	m[0] = d[0]
	m[n - 1] = d[n - 2]
	for i in range(1, n - 1):
		m[i] = 0.0 if d[i - 1] * d[i] <= 0.0 else (d[i - 1] + d[i]) / 2.0
	for i in n - 1:
		if d[i] == 0.0:
			m[i] = 0.0
			m[i + 1] = 0.0
			continue
		var a := m[i] / d[i]
		var b := m[i + 1] / d[i]
		var r2 := a * a + b * b
		if r2 > 9.0:
			var tau := 3.0 / sqrt(r2)
			m[i] = tau * a * d[i]
			m[i + 1] = tau * b * d[i]
	return m


## The curve at `x` from `_pchip_tangents`' tangents; flat outside the first and last point.
static func _curve_eval(pts: Array, m: PackedFloat64Array, x: float) -> float:
	var n := pts.size()
	if x <= float(pts[0][0]):
		return float(pts[0][1])
	if x >= float(pts[n - 1][0]):
		return float(pts[n - 1][1])
	var i := 0
	while i + 2 < n and float(pts[i + 1][0]) <= x:
		i += 1
	var x0 := float(pts[i][0])
	var y0 := float(pts[i][1])
	var x1 := float(pts[i + 1][0])
	var y1 := float(pts[i + 1][1])
	var h := x1 - x0
	var t := (x - x0) / h
	var t2 := t * t
	var t3 := t * t * t
	return (2.0 * t3 - 3.0 * t2 + 1.0) * y0 + (t3 - 2.0 * t2 + t) * h * m[i] \
		+ (-2.0 * t3 + 3.0 * t2) * y1 + (t3 - t2) * h * m[i + 1]


## `ElevationRamp::sample` at `t` in 0..1: `[r, g, b, a]`, flat beyond the end stops, the
## stop pair's `k` shaped by the ramp's mode (Linear: k; Ease: k*k*(3-2k); Step: 1 once
## `k >= 1`, else 0). Empty `[]` for an empty ramp.
static func _ramp_sample(stops: Array, mode: String, t: float) -> Array:
	if stops.is_empty():
		return []
	t = clampf(t, 0.0, 1.0)
	var first: Dictionary = stops[0]
	if t <= float(first["at"]):
		return [first["col"][0], first["col"][1], first["col"][2], first["a"]]
	var last: Dictionary = stops[stops.size() - 1]
	if t >= float(last["at"]):
		return [last["col"][0], last["col"][1], last["col"][2], last["a"]]
	var hi := 0
	for i in stops.size():
		if float(stops[i]["at"]) >= t:
			hi = i
			break
	var a: Dictionary = stops[hi - 1]
	var b: Dictionary = stops[hi]
	var span := float(b["at"]) - float(a["at"])
	var k := 1.0 if span <= 0.0 else (t - float(a["at"])) / span
	match mode:
		"Ease":
			k = k * k * (3.0 - 2.0 * k)
		"Step":
			k = 1.0 if k >= 1.0 else 0.0
	return [
		float(a["col"][0]) + (float(b["col"][0]) - float(a["col"][0])) * k,
		float(a["col"][1]) + (float(b["col"][1]) - float(a["col"][1])) * k,
		float(a["col"][2]) + (float(b["col"][2]) - float(a["col"][2])) * k,
		float(a["a"]) + (float(b["a"]) - float(a["a"])) * k]


## sRGB EOTF (IEC 61966-2-1) on an encoded 0..1 value, mirrored through zero so an
## out-of-gamut channel stays continuous (`adjust.rs::eotf`).
static func _eotf(v: float) -> float:
	var a := absf(v)
	var l := a / 12.92 if a <= 0.04045 else pow((a + 0.055) / 1.055, 2.4)
	return l if v >= 0.0 else -l


## sRGB OETF, the inverse of `_eotf`, mirrored through zero (`adjust.rs::oetf`).
static func _oetf(v: float) -> float:
	var a := absf(v)
	var e := a * 12.92 if a <= 0.0031308 else 1.055 * pow(a, 1.0 / 2.4) - 0.055
	return e if v >= 0.0 else -e


## Real cube root (negative arguments allowed; `pow` of a negative base is NaN in GDScript).
static func _cbrt(x: float) -> float:
	return pow(x, 1.0 / 3.0) if x >= 0.0 else -pow(-x, 1.0 / 3.0)


## 3x3 matrix times a column vector.
static func _mat_vec(m: Array, v: PackedFloat64Array) -> PackedFloat64Array:
	return PackedFloat64Array([
		m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
		m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
		m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2]])


## A 0..255 sRGB colour `[r, g, b]` to OkLab `[L, a, b]` (Ottosson 2020).
static func srgb_to_oklab(c: PackedFloat64Array) -> PackedFloat64Array:
	var lin := PackedFloat64Array([_eotf(c[0] / 255.0), _eotf(c[1] / 255.0), _eotf(c[2] / 255.0)])
	var lms := _mat_vec(LINEAR_SRGB_TO_LMS, lin)
	return _mat_vec(LMS_CUBE_ROOT_TO_OKLAB, PackedFloat64Array([_cbrt(lms[0]), _cbrt(lms[1]), _cbrt(lms[2])]))


## OkLab back to a 0..255 sRGB colour, **unclamped**.
static func oklab_to_srgb(lab: PackedFloat64Array) -> PackedFloat64Array:
	var p := _mat_vec(OKLAB_TO_LMS_CUBE_ROOT, lab)
	var lin := _mat_vec(LMS_TO_LINEAR_SRGB, PackedFloat64Array([p[0] * p[0] * p[0], p[1] * p[1] * p[1], p[2] * p[2] * p[2]]))
	return PackedFloat64Array([255.0 * _oetf(lin[0]), 255.0 * _oetf(lin[1]), 255.0 * _oetf(lin[2])])


## Hue rotation, chroma scale and lightness shift in OkLCh; each axis only when non-zero
## (control flow). Rotating the (a, b) plane by the angle IS the hue rotation of OkLCh, with
## chroma and L untouched; saturation scales chroma by `1 + s`; lightness moves L toward 1
## (`l > 0`: `L + l(1-L)`) or toward 0 (`l < 0`: `L(1+l)`) -- `adjust.rs::hsl_oklch`.
static func _hsl_oklch(c: PackedFloat64Array, hue_deg: float, saturation: float, lightness: float) -> PackedFloat64Array:
	var lab := srgb_to_oklab(c)
	var l := lab[0]
	var a := lab[1]
	var b := lab[2]
	if hue_deg != 0.0:
		var th := deg_to_rad(hue_deg)
		var s := sin(th)
		var co := cos(th)
		var a2 := a * co - b * s
		var b2 := a * s + b * co
		a = a2
		b = b2
	if saturation != 0.0:
		var k := 1.0 + saturation
		a *= k
		b *= k
	if lightness > 0.0:
		l += lightness * (1.0 - l)
	elif lightness < 0.0:
		l *= 1.0 + lightness
	return oklab_to_srgb(PackedFloat64Array([l, a, b]))


# -- reading and validating the saved form ---------------------------------------------

## `{ok: false, error: ...}` in the wording of `AdjustError`'s Display: `adjustment 3: gamma:
## out of range`, or without the entry prefix.
static func _fail(entry: int, param: String, reason: String) -> Dictionary:
	var prefix := "adjustment %d: " % entry if entry >= 0 else ""
	return {"ok": false, "error": "%s%s: %s" % [prefix, param, reason]}


static func _is_num(v) -> bool:
	return typeof(v) == TYPE_FLOAT or typeof(v) == TYPE_INT


## True for a finite number within `lo..hi` (a NaN or infinity fails, as `adjust.rs::in_range`).
static func _in_range(v: float, lo: float, hi: float) -> bool:
	return is_finite(v) and v >= lo and v <= hi


## Parses and validates a whole stack. Returns `{ok: true, entries, doc}` or the failure.
## A document that does not parse, has an unknown kind or lacks a field is
## `document: not a valid adjustment stack` (serde's refusal in `AdjustStack::from_json`);
## then each entry is range-checked in `AdjustStack::validate`'s order.
static func _parse(json: String) -> Dictionary:
	var bad := _fail(-1, "document", "not a valid adjustment stack")
	# `JSON.new().parse()`, not `JSON.parse_string()`: the latter logs an engine ERROR for
	# every malformed document, and a refusal here is an ordinary return value, not an error.
	var parser := JSON.new()
	if parser.parse(json) != OK:
		return bad
	var root = parser.data
	if not (root is Dictionary):
		return bad
	var raw = root.get("entries", [])
	if not (raw is Array):
		return bad
	var entries: Array = []
	var doc: Array = []
	for r in raw:
		if not (r is Dictionary) or not (r.get("kind") is Dictionary):
			return bad
		var enabled = r.get("enabled", true)
		var opacity = r.get("opacity", 1.0)
		if typeof(enabled) != TYPE_BOOL or not _is_num(opacity):
			return bad
		var kd: Dictionary = r["kind"]
		if kd.size() != 1:
			return bad
		var name: String = String(kd.keys()[0])
		var body = kd[name]
		if not (body is Dictionary):
			return bad
		var p := _read_params(name, body)
		if p.is_empty():
			return bad
		var e := {"kind": name, "p": p, "enabled": enabled, "opacity": float(opacity)}
		entries.append(e)
		doc.append({"kind": {name: _doc_params(name, p)}, "enabled": enabled, "opacity": float(opacity)})
	if entries.size() > MAX_ENTRIES:
		return _fail(-1, "entries", "more than 32 adjustments")
	for i in entries.size():
		var err := _validate_entry(entries[i])
		if not err.is_empty():
			return _fail(i, err[0], err[1])
		if entries[i]["kind"] == "curves":
			entries[i]["m"] = _pchip_tangents(entries[i]["p"]["points"])
	return {"ok": true, "entries": entries, "doc": doc}


## One kind's parameters from its saved body, as floats; `{}` when the name is unknown or a
## field is missing or mistyped (the caller reports a document error).
static func _read_params(name: String, body: Dictionary) -> Dictionary:
	match name:
		"levels":
			var out := {}
			for k in ["in_black", "in_white", "gamma", "out_black", "out_white"]:
				if not _is_num(body.get(k)):
					return {}
				out[k] = float(body[k])
			return out
		"curves":
			var pts = body.get("points")
			if not (pts is Array):
				return {}
			var norm: Array = []
			for pt in pts:
				if not (pt is Array) or pt.size() != 2 or not _is_num(pt[0]) or not _is_num(pt[1]):
					return {}
				norm.append([float(pt[0]), float(pt[1])])
			return {"points": norm}
		"hsl":
			var out := {}
			for k in ["hue_deg", "saturation", "lightness"]:
				if not _is_num(body.get(k)):
					return {}
				out[k] = float(body[k])
			return out
		"gradient_map":
			var ramp = body.get("ramp")
			if not (ramp is Dictionary) or not (ramp.get("stops") is Array):
				return {}
			var mode = ramp.get("mode", "Linear")
			if typeof(mode) != TYPE_STRING or not (String(mode) in ["Linear", "Ease", "Step"]):
				return {}
			var stops: Array = []
			for s in ramp["stops"]:
				if not (s is Dictionary) or not _is_num(s.get("at")) or not (s.get("col") is Array):
					return {}
				var col: Array = s["col"]
				var a = s.get("a", 1.0)
				if col.size() != 3 or not _is_num(col[0]) or not _is_num(col[1]) or not _is_num(col[2]) or not _is_num(a):
					return {}
				stops.append({"at": float(s["at"]), "col": [float(col[0]), float(col[1]), float(col[2])], "a": float(a)})
			# An empty stop list is a VALID document that `validate` refuses by name, so the
			# parameter set must not be empty for it: carry the mode and the (empty) list.
			return {"stops": stops, "mode": String(mode)}
	return {}


## The saved form of a kind's parameters, in serde's field order.
static func _doc_params(name: String, p: Dictionary) -> Dictionary:
	match name:
		"levels":
			return {"in_black": p["in_black"], "in_white": p["in_white"], "gamma": p["gamma"],
				"out_black": p["out_black"], "out_white": p["out_white"]}
		"curves":
			return {"points": p["points"].duplicate(true)}
		"hsl":
			return {"hue_deg": p["hue_deg"], "saturation": p["saturation"], "lightness": p["lightness"]}
		"gradient_map":
			return {"ramp": {"stops": p["stops"].duplicate(true), "mode": p["mode"]}}
	return {}


## `Adjustment::validate` + `AdjustKind::validate`: `[]` when fine, else `[param, reason]`
## (never a clamped value). Same ranges and the same order as the Rust, so the first
## complaint is the same one.
static func _validate_entry(e: Dictionary) -> Array:
	if not _in_range(float(e["opacity"]), 0.0, 1.0):
		return ["opacity", "must be a finite value in 0..=1"]
	var p: Dictionary = e["p"]
	match e["kind"]:
		"levels":
			if not _in_range(p["in_black"], 0.0, 255.0):
				return ["in_black", "must be a finite value in 0..=255"]
			if not _in_range(p["in_white"], 0.0, 255.0):
				return ["in_white", "must be a finite value in 0..=255"]
			if p["in_black"] >= p["in_white"]:
				return ["in_black", "must be below in_white"]
			if not _in_range(p["gamma"], GAMMA_MIN, GAMMA_MAX):
				return ["gamma", "must be a finite value in 0.1..=10"]
			if not _in_range(p["out_black"], 0.0, 255.0):
				return ["out_black", "must be a finite value in 0..=255"]
			if not _in_range(p["out_white"], 0.0, 255.0):
				return ["out_white", "must be a finite value in 0..=255"]
		"curves":
			var pts: Array = p["points"]
			if pts.size() < MIN_CURVE_POINTS or pts.size() > MAX_CURVE_POINTS:
				return ["points", "must hold 2..=16 control points"]
			for i in pts.size():
				if not _in_range(pts[i][0], 0.0, 255.0):
					return ["points", "every x must be a finite value in 0..=255"]
				if not _in_range(pts[i][1], 0.0, 255.0):
					return ["points", "every y must be a finite value in 0..=255"]
				if i > 0 and pts[i][0] <= pts[i - 1][0]:
					return ["points", "x must be strictly increasing"]
		"hsl":
			if not _in_range(p["hue_deg"], -HUE_MAX, HUE_MAX):
				return ["hue_deg", "must be a finite value in -360..=360"]
			if not _in_range(p["saturation"], -1.0, 1.0):
				return ["saturation", "must be a finite value in -1..=1"]
			if not _in_range(p["lightness"], -1.0, 1.0):
				return ["lightness", "must be a finite value in -1..=1"]
		"gradient_map":
			var stops: Array = p["stops"]
			if stops.is_empty():
				return ["ramp", "must hold at least one stop"]
			var prev := -INF
			for s in stops:
				if not _in_range(s["at"], 0.0, 1.0):
					return ["ramp", "every stop position must be a finite value in 0..=1"]
				if s["at"] < prev:
					return ["ramp", "stops must be sorted by position"]
				prev = s["at"]
				for ch in s["col"]:
					if not _in_range(ch, 0.0, 255.0):
						return ["ramp", "every stop channel must be a finite value in 0..=255"]
				if not _in_range(s["a"], 0.0, 1.0):
					return ["ramp", "every stop alpha must be a finite value in 0..=1"]
	return []

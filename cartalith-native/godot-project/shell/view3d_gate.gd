extends Node
## No `class_name`, deliberately: every user preloads this file
## (`const View3DGate := preload(...)`), so it works in a tree whose `.godot`
## class cache predates 3D-1 without an import pass.

## **The 3D capability gate** (`3D_VIEWPORT_SCOPE.md` §4, milestone 3D-1).
##
## What it does: measures, on this device, in the real window, whether the 3D
## view can run and at which tier (Off / Basic / Standard / Full), and words the
## answer for the user with the numbers it measured (§4.3). It is the
## owner's rule made measurable: *"if a adreno 630 can't bring enough power 3D
## rendering should be disabled based on a machines power"* (§4; Ruling CS
## item 4 approved the scope, `DECISIONS.md` §4a).
##
## Why it is built before any terrain: so that "3D is unavailable, and here is
## why" works from the first day (§5, 3D-1; research §D Milestone 0). Nothing in
## this file reads a world: the controls and workloads are synthetic, sized by
## §3.12's tier table, so the answer is a property of the device and not of
## the map that happens to be open.
##
## What it must never do:
## - **Run inside a generate, or touch the 2D map.** It draws only into the
##   `SubViewport` it is handed (`run()`), which `view3d.gd` owns. The 2D path
##   never calls it, so 2D is byte-identical whether or not it has run.
## - **Force a tier.** `classify()` returns the highest tier whose controls and
##   bars ALL passed; a higher tier is only ever the user's explicit choice
##   (`check_forced()`, §4.4), never auto-applied (the `recommended_quality_tier`
##   "never auto-apply" pattern, research §tiers).
## - **Panic across the gdext boundary.** It calls no `WorldGen` method at all.
## - **Read a pass from a run that could not measure.** Headless has no
##   rasteriser (`MISTAKES.md`, "Run a pixel probe"); `run()` refuses there and
##   reports `could_not_run` instead of a tier.
##
## Shape: `run()` / `check_forced()` are the only methods that render;
## `classify()`, `disclosure()`, `stale_reason()` and `phone_class()` are pure
## and static so the acceptance probe (`_view3d_probe.gd`) can drive every
## branch, and so a mutation of any bar is visible to a test.

## Tier indices. Ordered: a higher number is a strictly larger workload (§4.1).
const TIER_OFF := 0
const TIER_BASIC := 1
const TIER_STANDARD := 2
const TIER_FULL := 3
## User-facing tier names (§4.1's column heads).
const TIER_NAMES: Array[String] = ["Off", "Basic", "Standard", "Full"]

## Bumped whenever the probe's method or bars change, so a result measured by
## an older probe is re-measured instead of trusted (§4.2: "re-runs when ... the
## app version changes"; this shell has no app-version string of its own, so
## the probe's own version plus Godot's stands in for it -- stated, not hidden).
const PROBE_VERSION := 1

## §4.1 bars, in milliseconds of frame time at the 95th percentile.
## 33 ms is LOD-D6's phone bar ("p99 frame <= 33 ms"); 16.7 ms is one 60 Hz
## frame. Both are the scope's proposed defaults, re-calibrated in 3D-7.
const BAR_30HZ_MS := 33.0
const BAR_60HZ_MS := 16.7
## §4.4: a frame over this, at any point while a workload runs, fails the
## workload outright (the "frame over 250 ms" failure class).
const FAIL_FRAME_MS := 250.0

## §4.2 step 3: 60 warm-up frames, then 180 timed frames.
const WARM_FRAMES := 60
const TIMED_FRAMES := 180
## Early stop, a labelled judgement: after this many timed frames, a workload
## whose running median is already over `EARLY_STOP_FACTOR` x its bar cannot
## pass, so the rest of its frames are skipped. On a slow phone this keeps the
## probe near §4.2's "about 3 s" instead of 3 x 240 frames at 100 ms each.
const EARLY_STOP_AFTER := 30
const EARLY_STOP_FACTOR := 2.0
## A forced tier's own check (§4.4) is shorter than the probe: it asks only
## "does this tier hold without a fatal frame", not "where does it sit".
const FORCE_WARM_FRAMES := 20
const FORCE_TIMED_FRAMES := 60

## §4.2 step 3's workloads: instance count and patch vertices per side, one
## `MultiMesh` each. Basic is §3.12's "one 257^2 mesh"; Standard is 256 x 33^2
## on a phone and 512 x 65^2 on desktop; Full is 512 x 129^2.
const WORKLOADS := {
	"basic": {"instances": 1, "n": 257},
	"standard_phone": {"instances": 256, "n": 33},
	"standard_desktop": {"instances": 512, "n": 65},
	"full": {"instances": 512, "n": 129},
}
## §3.12's layer capacity per tier, allocated and read back by step 4. Basic
## keeps one layer pair, so it has no memory step of its own.
const LAYERS := {"standard_phone": 64, "standard_desktop": 256, "full": 512}
## §3.12's layer geometry: a 256^2 RGBA8 colour tile, and a height tile padded
## by `render::tile_halo_px` (a few texels; 2 here, as in 3D-0's tiles).
const TILE_PX := 256
const HEIGHT_PAD := 2

## The positive controls' pass mark: the share of sampled pixels that must
## change when displacement is switched on (VTF) or one array layer is
## rewritten (array). A labelled judgement: §2.4 measured 24 266 moved pixels
## for the VTF control in a 1600 x 900 window, about 1.7 %; 0.2 % keeps an
## order of magnitude of margin while staying far above readback noise, which
## is zero for an unshaded scene that does not change.
const MIN_MOVED_FRACTION := 0.002
## Sampling stride of the pixel comparison (every 4th pixel each way).
const DIFF_STRIDE := 4
## Displacement switched on by the VTF control and used by every workload, in
## patch-size units: one patch width of relief. §2.4 switched `amp` 0 -> 4 over
## a larger patch; what matters is only that the change is visible, so the
## figure is a labelled judgement.
const VTF_AMP := 1.0

## **Test seams for the acceptance probe, and nothing else.** Both default to
## off and nothing in the shell sets them.
## `debug_stub_vtf`: compile the probe shader with its vertex-texture read
## replaced by a constant, which is what "VTF stubbed out" means in 3D-1's
## acceptance. The VTF and array controls must then FAIL.
var debug_stub_vtf := false
## `debug_fail_from_tier`: report every timed frame of a workload at or above
## this tier as 300 ms (over `FAIL_FRAME_MS`), and mark the result `injected`,
## so "forcing a tier then failing drops one tier" can be driven on a machine
## fast enough to pass every tier. -1 = off.
var debug_fail_from_tier := -1
## The frame time `debug_fail_from_tier` reports: a fixed 300 ms, NOT derived
## from `FAIL_FRAME_MS`, so a mutation of that limit is visible to the probe
## (a seam written as "limit + 50" moved with the limit and hid it).
const INJECTED_FRAME_MS := 300.0

## The SubViewport being rendered into, and the probe's own 3D root under it.
var _sv: SubViewport
var _root: Node3D
var _cam: Camera3D

# -- pure helpers ---------------------------------------------------------------

## Whether this device takes the phone bars (§4.1: read from `OS.get_name()`,
## because the adapter type reads 0 under Compatibility, §2.2).
static func phone_class(os_name: String) -> bool:
	return os_name == "Android" or os_name == "iOS"

## The frame bar a workload must meet, by its `WORKLOADS` key.
static func bar_for(workload: String) -> float:
	match workload:
		"basic", "standard_phone":
			return BAR_30HZ_MS
		_:
			return BAR_60HZ_MS

## The Standard workload key for this device class.
static func standard_key(phone: bool) -> String:
	return "standard_phone" if phone else "standard_desktop"

## Triangle count of one workload: instances x (n-1)^2 quads x 2.
static func triangles(workload: String) -> int:
	var w: Dictionary = WORKLOADS[workload]
	var q: int = int(w["n"]) - 1
	return int(w["instances"]) * q * q * 2

## Whether one measured workload meets its bar. A workload that was skipped,
## aborted, or has no p95 does not pass; that is never read as a pass.
static func workload_passes(m) -> bool:
	if not (m is Dictionary) or not (m as Dictionary).has("p95"):
		return false
	var d: Dictionary = m
	return not bool(d.get("aborted", false)) and float(d["p95"]) <= float(d.get("bar", 0.0))

## **The tier rule (§4.1), from a measured result alone.** Returns
## `{tier, cause}`: the highest tier whose controls and bars all passed, and the
## key of the first thing that stopped it going higher ("" at Full). Every
## requirement is checked in §4.1's order, so a later test can never rescue an
## earlier failure.
static func classify(r: Dictionary) -> Dictionary:
	if bool(r.get("could_not_run", false)):
		return {"tier": TIER_OFF, "cause": "could_not_run"}
	var phone := bool(r.get("phone", false))
	var w: Dictionary = r.get("workloads", {})
	var mem: Dictionary = r.get("memory", {})
	if not bool((r.get("vtf", {}) as Dictionary).get("pass", false)):
		return {"tier": TIER_OFF, "cause": "vtf"}
	if not workload_passes(w.get("basic")):
		return {"tier": TIER_OFF, "cause": "basic_slow"}
	if not bool((r.get("array", {}) as Dictionary).get("pass", false)):
		return {"tier": TIER_BASIC, "cause": "array"}
	var sk := standard_key(phone)
	if not workload_passes(w.get(sk)):
		return {"tier": TIER_BASIC, "cause": "standard_slow"}
	if not bool((mem.get(sk, {}) as Dictionary).get("ok", false)):
		return {"tier": TIER_BASIC, "cause": "standard_memory"}
	if not workload_passes(w.get("full")):
		return {"tier": TIER_STANDARD, "cause": "full_slow"}
	if not bool((mem.get("full", {}) as Dictionary).get("ok", false)):
		return {"tier": TIER_STANDARD, "cause": "full_memory"}
	return {"tier": TIER_FULL, "cause": ""}

## A measured frame time as text, e.g. "48" in "48 ms per frame": rounded to a
## whole millisecond above 10 ms and to 0.1 ms below, since §4.3's drafts quote
## whole numbers and a sub-10 figure would otherwise read as 0 or 1. Bars are
## printed exactly (`String.num`), so 16.7 never reads as "17".
static func _ms(v: float) -> String:
	return ("%.0f" % v) if v >= 10.0 else ("%.1f" % v)

## Thousands-grouped integer, so "131 072 triangles" reads as §4.3 drafts it.
static func _grouped(n: int) -> String:
	var s := str(n)
	var out := ""
	while s.length() > 3:
		out = " " + s.substr(s.length() - 3) + out
		s = s.substr(0, s.length() - 3)
	return s + out

## The renderer phrase §4.3 puts in brackets, read off the result.
static func _renderer(r: Dictionary) -> String:
	return "renderer: %s, %s" % [String(r.get("method", "?")), String(r.get("api", "?"))]

## Why a workload failed, in words: the fatal frame, the early stop or the p95.
static func _why_slow(m: Dictionary, bar: float) -> String:
	if bool(m.get("aborted", false)) and m.has("worst"):
		return "one frame of its test view took %s ms (the limit is %.0f ms)" % [_ms(float(m["worst"])), FAIL_FRAME_MS]
	if not m.has("p95"):
		return "its test view did not run"
	return "a test view of %s triangles took %s ms per frame; it needs %s ms or less" % [
		_grouped(int(m.get("triangles", 0))), _ms(float(m["p95"])), String.num(bar)]

## **§4.3's disclosure: the tier and its measured cause, one or two sentences.**
## `r` is a stored result (empty when never tested); `forced` is the user's
## forced tier or -1; `note` is a forced-tier drop message from
## `check_forced()`, shown first when present. Wording drafted in §4.3 and
## settled with the `design:ux-copy` skill: what happened, why (with the
## number), and what the user can do.
static func disclosure(r: Dictionary, forced: int = -1, note: String = "") -> String:
	if r.is_empty():
		return "3D has not been tested on this device yet. Opening the 3D view runs a short test first (about 3 seconds)."
	var c := classify(r)
	var tier: int = c["tier"]
	var cause: String = c["cause"]
	var w: Dictionary = r.get("workloads", {})
	var sk := standard_key(bool(r.get("phone", false)))
	var text := ""
	match cause:
		"could_not_run":
			text = "3D is off: the test could not run here (%s)." % String(r.get("why", "no display to draw into"))
		"vtf":
			text = "3D is off on this device: drawing terrain from height textures did not work here (%s)." % _renderer(r)
		"basic_slow":
			text = "3D is off on this device: %s." % _why_slow(w.get("basic", {}), bar_for("basic"))
		"array":
			text = "3D runs in Basic mode here: updating one layer of a terrain texture array did not change the picture, and Standard needs it (%s)." % _renderer(r)
		"standard_slow":
			text = "3D runs in Basic mode here: for Standard, %s. Re-test, or force Standard." % _why_slow(w.get(sk, {}), bar_for(sk))
		"standard_memory":
			text = "3D runs in Basic mode here: the %d terrain layers Standard keeps could not be stored and read back on the graphics card." % int(LAYERS[sk])
		"full_slow":
			text = "3D Standard. For Full, %s." % _why_slow(w.get("full", {}), bar_for("full"))
		"full_memory":
			text = "3D Standard. Full keeps %d terrain layers, and they could not be stored and read back on the graphics card." % int(LAYERS["full"])
		_:
			var f: Dictionary = w.get("full", {})
			text = "3D Full: its test view of %s triangles took %s ms per frame, within %s ms." % [
				_grouped(int(f.get("triangles", 0))), _ms(float(f.get("p95", 0.0))), String.num(bar_for("full"))]
	if forced >= 0 and forced != tier:
		text = "3D %s, forced. The test chose %s: %s" % [TIER_NAMES[forced], TIER_NAMES[tier], text]
	if note != "":
		text = note + " " + text
	return text

## The tier the 3D view runs at: the user's forced tier if one is set, else
## the measured one. Off when never measured (an untested device gets no 3D
## until the test has run, which entering 3D does first).
static func effective_tier(r: Dictionary, forced: int) -> int:
	if forced >= TIER_OFF and forced <= TIER_FULL:
		return forced
	if r.is_empty():
		return TIER_OFF
	return int(classify(r)["tier"])

## The identity a stored result was measured against (§4.2: adapter name,
## driver string, app version). Kept separate so `stale_reason()` can be driven
## with a hand-made identity in the acceptance probe.
static func device_identity() -> Dictionary:
	return {
		"adapter": RenderingServer.get_video_adapter_name(),
		"driver": RenderingServer.get_video_adapter_api_version(),
		"godot": String(Engine.get_version_info().get("string", "")),
		"probe_version": PROBE_VERSION,
	}

## Why a stored result must be re-measured, or "" when it still stands. An
## empty result is stale by definition ("never tested").
static func stale_reason(r: Dictionary, ident: Dictionary) -> String:
	if r.is_empty():
		return "never tested"
	for k in ["adapter", "driver", "godot", "probe_version"]:
		if str(r.get(k, "")) != str(ident.get(k, "")):
			return "%s changed" % k
	return ""

# -- rendering --------------------------------------------------------------------

## The probe shader. One unit patch on [0,1]^2 in x/z per `MultiMesh` instance;
## `INSTANCE_CUSTOM.x` picks the height layer of a `Texture2DArray`
## (`FORMAT_RF`) read in `vertex()` -- the two capabilities §2.2 measured and
## §3.2 relies on. Unshaded: the gate measures geometry throughput, not
## lighting. With `stub` the height read becomes a constant, which removes the
## vertex-texture fetch entirely (3D-1 acceptance: "positive controls fail when
## VTF is stubbed out").
static func shader_code(stub: bool) -> String:
	var read := "0.0" if stub else "textureLod(hmap, vec3(VERTEX.xz, INSTANCE_CUSTOM.x), 0.0).r"
	return """shader_type spatial;
render_mode unshaded, cull_disabled;
uniform sampler2DArray hmap : filter_linear, repeat_disable;
uniform sampler2DArray cmap : filter_nearest, repeat_disable;
uniform float amp = 0.0;
uniform float use_cmap = 0.0;
varying float v_h;
varying vec2 v_uv;
varying flat float v_layer;
void vertex() {
	v_uv = VERTEX.xz;
	v_layer = INSTANCE_CUSTOM.x;
	float h = %s;
	v_h = h;
	VERTEX.y = h * amp;
}
void fragment() {
	vec3 ramp = vec3(0.15 + 0.7 * v_h, 0.45 + 0.3 * v_h, 0.75 - 0.5 * v_h);
	ALBEDO = mix(ramp, texture(cmap, vec3(v_uv, v_layer)).rgb, use_cmap);
}
""" % read

## One N x N unit patch on [0,1]^2 in x/z (§3.2's shared patch).
static func make_patch(n: int) -> ArrayMesh:
	var verts := PackedVector3Array()
	verts.resize(n * n)
	for j in n:
		for i in n:
			verts[j * n + i] = Vector3(float(i) / (n - 1), 0.0, float(j) / (n - 1))
	var idx := PackedInt32Array()
	idx.resize((n - 1) * (n - 1) * 6)
	var k := 0
	for j in n - 1:
		for i in n - 1:
			var a := j * n + i
			idx[k] = a; idx[k + 1] = a + 1; idx[k + 2] = a + n
			idx[k + 3] = a + 1; idx[k + 4] = a + n + 1; idx[k + 5] = a + n
			k += 6
	var arr := []
	arr.resize(Mesh.ARRAY_MAX)
	arr[Mesh.ARRAY_VERTEX] = verts
	arr[Mesh.ARRAY_INDEX] = idx
	var m := ArrayMesh.new()
	m.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arr)
	return m

## A `FORMAT_RF` height image: a smooth bump field whose phase is `seed`, so two
## layers built with different seeds differ everywhere that matters.
static func height_image(px: int, seed: float) -> Image:
	var img := Image.create_empty(px, px, false, Image.FORMAT_RF)
	for y in px:
		for x in px:
			var u := float(x) / (px - 1)
			var v := float(y) / (px - 1)
			img.set_pixel(x, y, Color(0.5 + 0.5 * sin(6.2832 * (2.0 * u + seed)) * cos(6.2832 * (2.0 * v - seed)), 0, 0))
	return img

## A `Texture2DArray` over `imgs` (all one size and format).
## `create_from_images` takes a typed `Array[Image]`, so the literal is copied
## into one rather than passed as a plain `Array`.
static func _heights(imgs: Array) -> Texture2DArray:
	var typed: Array[Image] = []
	for i in imgs:
		typed.append(i)
	var t := Texture2DArray.new()
	t.create_from_images(typed)
	return t

## Starts measuring against `sv`. Builds a fresh 3D root and camera under it;
## `_teardown()` frees them. The caller (`view3d.gd`) owns `sv` and keeps it
## visible, so every frame is drawn into the real window (§4.2: "an off-screen
## render cannot be timed under Compatibility the same way").
func _setup(sv: SubViewport) -> void:
	_sv = sv
	_root = Node3D.new()
	_root.name = "GateProbe"
	sv.add_child(_root)
	_cam = Camera3D.new()
	_cam.fov = 50.0
	_root.add_child(_cam)
	_cam.current = true

func _teardown() -> void:
	if _root != null:
		_root.queue_free()
	_root = null
	_cam = null

## A `MultiMesh` of `count` patches of `n` vertices a side, laid out in a
## near-square grid of unit cells and framed whole by the camera (every vertex
## in view: §2.4's worst case). Heights come from `harr`; instance i reads
## layer `layer_of.call(i)`.
func _workload_node(count: int, n: int, harr: Texture2DArray, layer_of: Callable,
		carr: Texture2DArray = null) -> MultiMeshInstance3D:
	var cols := int(ceil(sqrt(float(count))))
	var rows := int(ceil(float(count) / cols))
	var mat := ShaderMaterial.new()
	var sh := Shader.new()
	sh.code = shader_code(debug_stub_vtf)
	mat.shader = sh
	mat.set_shader_parameter("hmap", harr)
	if carr != null:
		mat.set_shader_parameter("cmap", carr)
		mat.set_shader_parameter("use_cmap", 1.0)
	var mm := MultiMesh.new()
	mm.transform_format = MultiMesh.TRANSFORM_3D
	mm.use_custom_data = true
	mm.mesh = make_patch(n)
	mm.instance_count = count
	## Camera-relative from the start (§3.11): the camera stays at the origin
	## and the grid is placed in front of it, centred on its view axis.
	for i in count:
		var cx := float(i % cols) - cols * 0.5
		var cz := float(i / cols) - rows * 0.5
		mm.set_instance_transform(i, Transform3D(Basis(), Vector3(cx, 0.0, cz)))
		mm.set_instance_custom_data(i, Color(float(layer_of.call(i)), 0.0, 0.0, 0.0))
	var mmi := MultiMeshInstance3D.new()
	mmi.multimesh = mm
	mmi.material_override = mat
	## Displacement happens in the vertex shader, so the CPU-side box is too
	## small; a generous box keeps Godot from culling a displaced patch.
	mmi.custom_aabb = AABB(Vector3(-1e4, -1e3, -1e4), Vector3(2e4, 2e3, 2e4))
	## The grid sits in front of the camera, looking down at 55 degrees from a
	## distance that frames its larger side (labelled judgement).
	var span := float(maxi(cols, rows))
	var d := span * 1.25 + 1.0
	var pitch := deg_to_rad(55.0)
	mmi.position = Vector3(0.0, -d * sin(pitch), -d * cos(pitch))
	_cam.rotation = Vector3(-pitch, 0.0, 0.0)
	_cam.near = 0.05
	_cam.far = d * 4.0 + VTF_AMP * 2.0
	_root.add_child(mmi)
	return mmi

## The SubViewport's frame, after `frames` drawn frames. A readback stall,
## acceptable inside the probe and nowhere else (§3.10).
func _grab(frames: int = 3) -> Image:
	for i in frames:
		await RenderingServer.frame_post_draw
	return _sv.get_texture().get_image()

## Share of sampled pixels that differ between two frames of one size.
static func moved_fraction(a: Image, b: Image) -> float:
	if a == null or b == null or a.get_size() != b.get_size() or a.get_width() == 0:
		return 0.0
	var n := 0
	var tot := 0
	for y in range(0, a.get_height(), DIFF_STRIDE):
		for x in range(0, a.get_width(), DIFF_STRIDE):
			tot += 1
			if not a.get_pixel(x, y).is_equal_approx(b.get_pixel(x, y)):
				n += 1
	return float(n) / maxi(tot, 1)

## §4.2 step 1: the 257^2 patch at amp 0, then at `VTF_AMP`. Some pixels must move.
func _vtf_control() -> Dictionary:
	var h := _heights([height_image(64, 0.13)])
	var node := _workload_node(1, 257, h, func(_i): return 0)
	var mat: ShaderMaterial = node.material_override
	mat.set_shader_parameter("amp", 0.0)
	var a := await _grab()
	mat.set_shader_parameter("amp", VTF_AMP)
	var b := await _grab()
	node.queue_free()
	var f := moved_fraction(a, b)
	return {"moved": f, "pass": f >= MIN_MOVED_FRACTION}

## §4.2 step 2: two patches on layers 0 and 1; rewrite layer 1; pixels must move.
func _array_control() -> Dictionary:
	var h := _heights([height_image(64, 0.13), height_image(64, 0.13)])
	var node := _workload_node(2, 65, h, func(i): return i)
	(node.material_override as ShaderMaterial).set_shader_parameter("amp", VTF_AMP)
	var a := await _grab()
	h.update_layer(height_image(64, 0.61), 1)
	var b := await _grab()
	node.queue_free()
	var f := moved_fraction(a, b)
	return {"moved": f, "pass": f >= MIN_MOVED_FRACTION}

## Times `timed` frames after `warm` warm-up frames: the wall-clock delta
## between process frames (§2.4's method), sorted for median and p95. Stops
## early on a fatal frame (`FAIL_FRAME_MS`) or a hopeless running median.
func _time(warm: int, timed: int, bar: float, inject: bool) -> Dictionary:
	for i in warm:
		await get_tree().process_frame
	var ts := PackedFloat64Array()
	var last := Time.get_ticks_usec()
	var aborted := false
	var worst := 0.0
	for i in timed:
		await get_tree().process_frame
		var now := Time.get_ticks_usec()
		var ms := (now - last) / 1000.0
		last = now
		if inject:
			ms = INJECTED_FRAME_MS
		ts.append(ms)
		worst = maxf(worst, ms)
		if ms > FAIL_FRAME_MS:
			aborted = true
			break
		if ts.size() == EARLY_STOP_AFTER:
			var s := ts.duplicate()
			s.sort()
			if s[s.size() / 2] > EARLY_STOP_FACTOR * bar:
				break
	ts.sort()
	var out := {"frames": ts.size(), "bar": bar, "aborted": aborted, "worst": worst,
		"median": ts[ts.size() / 2], "p95": ts[mini(ts.size() - 1, int(ts.size() * 0.95))],
		"min": ts[0], "max": ts[ts.size() - 1]}
	if inject:
		out["injected"] = true
	return out

## One workload: build it, time it, free it. `tier` only decides whether the
## `debug_fail_from_tier` seam applies.
func _workload(key: String, tier: int, warm: int, timed: int) -> Dictionary:
	var w: Dictionary = WORKLOADS[key]
	var h := _heights([height_image(64, 0.13)])
	var node := _workload_node(int(w["instances"]), int(w["n"]), h, func(_i): return 0)
	(node.material_override as ShaderMaterial).set_shader_parameter("amp", VTF_AMP)
	var inject := debug_fail_from_tier >= 0 and tier >= debug_fail_from_tier
	var m := await _time(warm, timed, bar_for(key), inject)
	var rid := _sv.get_viewport_rid()
	m["triangles"] = triangles(key)
	m["drawn_primitives"] = RenderingServer.viewport_get_render_info(rid,
		RenderingServer.VIEWPORT_RENDER_INFO_TYPE_VISIBLE, RenderingServer.VIEWPORT_RENDER_INFO_PRIMITIVES_IN_FRAME)
	m["draw_calls"] = RenderingServer.viewport_get_render_info(rid,
		RenderingServer.VIEWPORT_RENDER_INFO_TYPE_VISIBLE, RenderingServer.VIEWPORT_RENDER_INFO_DRAW_CALLS_IN_FRAME)
	node.queue_free()
	return m

## §4.2 step 4: allocate `layers` colour + height layers (§3.12's geometry) and
## confirm by drawing the LAST layer, which alone is magenta: if the array did
## not allocate, the draw shows black or nothing, never magenta. Records
## `RENDER_VIDEO_MEM_USED` before and after as "this app's usage", not as a
## device ceiling (research §"Capability detection").
##
## `mark_layer` (default: the last) is which layer gets the magenta; the check
## always DRAWS the last. Passing another layer is the negative control the
## acceptance probe uses: the draw must then not see magenta.
func _memory(layers: int, mark_layer: int = -1) -> Dictionary:
	if mark_layer < 0:
		mark_layer = layers - 1
	var before := Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED)
	var black := Image.create_empty(TILE_PX, TILE_PX, false, Image.FORMAT_RGBA8)
	black.fill(Color.BLACK)
	var mark := Image.create_empty(TILE_PX, TILE_PX, false, Image.FORMAT_RGBA8)
	mark.fill(Color.MAGENTA)
	var hp := TILE_PX + 2 * HEIGHT_PAD
	var flat := Image.create_empty(hp, hp, false, Image.FORMAT_RF)
	var cimgs: Array[Image] = []
	var himgs: Array[Image] = []
	for i in layers:
		cimgs.append(mark if i == mark_layer else black)
		himgs.append(flat)
	var carr := Texture2DArray.new()
	var harr := Texture2DArray.new()
	var ok := carr.create_from_images(cimgs) == OK and harr.create_from_images(himgs) == OK
	var seen := false
	if ok:
		var node := _workload_node(1, 2, harr, func(_i): return layers - 1, carr)
		var img := await _grab()
		node.queue_free()
		var c := img.get_pixel(img.get_width() / 2, img.get_height() / 2)
		seen = c.r > 0.9 and c.g < 0.1 and c.b > 0.9
	var after := Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED)
	return {"layers": layers, "ok": ok and seen, "allocated": ok, "read_back": seen,
		"video_mem_before_mb": before / 1048576.0, "video_mem_after_mb": after / 1048576.0}

## The layer check alone, set up and torn down around itself: the acceptance
## probe's positive and negative controls for step 4 (`_memory`).
func memory_check(sv: SubViewport, layers: int, mark_layer: int = -1) -> Dictionary:
	_setup(sv)
	var m := await _memory(layers, mark_layer)
	_teardown()
	return m

## **The §4.2 probe.** Renders into `sv` for a few seconds and returns the
## measured result (the dictionary `classify()` reads), stamped with the
## device identity and the classified tier. Never throws; a run that cannot
## measure returns `could_not_run` with its reason.
##
## Vsync and low-processor mode are switched off for the run and restored
## after: with vsync on every frame reads ~16.7 ms whatever the GPU does, and
## low-processor mode sleeps between frames (`app.gd::_enable_idle_mode`).
func run(sv: SubViewport) -> Dictionary:
	var r := device_identity()
	r["os"] = OS.get_name()
	r["phone"] = phone_class(OS.get_name())
	r["method"] = String(ProjectSettings.get_setting("rendering/renderer/rendering_method", "?"))
	r["api"] = "%s %s" % [RenderingServer.get_video_adapter_vendor(), RenderingServer.get_video_adapter_api_version()]
	r["measured_at"] = Time.get_datetime_string_from_system()
	if DisplayServer.get_name() == "headless":
		r["could_not_run"] = true
		r["why"] = "headless has no rasteriser"
		return _stamp(r)
	var vsync := DisplayServer.window_get_vsync_mode()
	var lowp := OS.low_processor_usage_mode
	var max_fps := Engine.max_fps
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	OS.low_processor_usage_mode = false
	Engine.max_fps = 0
	_setup(sv)
	r["window"] = "%dx%d" % [sv.size.x, sv.size.y]
	r["vtf"] = await _vtf_control()
	r["array"] = await _array_control()
	var w := {}
	var mem := {}
	var sk := standard_key(bool(r["phone"]))
	## Lighter tiers first; a heavier one is only timed when every lighter one
	## passed, because `classify()` could not grant it anyway.
	if bool(r["vtf"]["pass"]):
		w["basic"] = await _workload("basic", TIER_BASIC, WARM_FRAMES, TIMED_FRAMES)
		if workload_passes(w["basic"]) and bool(r["array"]["pass"]):
			w[sk] = await _workload(sk, TIER_STANDARD, WARM_FRAMES, TIMED_FRAMES)
			if workload_passes(w[sk]):
				mem[sk] = await _memory(int(LAYERS[sk]))
				if bool(mem[sk]["ok"]):
					w["full"] = await _workload("full", TIER_FULL, WARM_FRAMES, TIMED_FRAMES)
					if workload_passes(w["full"]):
						mem["full"] = await _memory(int(LAYERS["full"]))
	r["workloads"] = w
	r["memory"] = mem
	_teardown()
	DisplayServer.window_set_vsync_mode(vsync)
	OS.low_processor_usage_mode = lowp
	Engine.max_fps = max_fps
	return _stamp(r)

func _stamp(r: Dictionary) -> Dictionary:
	var c := classify(r)
	r["tier"] = c["tier"]
	r["cause"] = c["cause"]
	return r

## **§4.4: a forced tier that fails drops one tier and says so.** Starting at
## `tier`, runs that tier's workload (shortened) and layer allocation; on a
## fatal frame or a failed allocation it drops one tier and checks again, until
## a tier holds or it reaches `floor_tier` (the measured tier, which has
## already passed) or Off. Returns `{tier, note}`; `note` names every drop with
## its measured cause, or is "" when the forced tier held. Never throws.
func check_forced(sv: SubViewport, tier: int, floor_tier: int) -> Dictionary:
	var notes: Array[String] = []
	var phone := phone_class(OS.get_name())
	var vsync := DisplayServer.window_get_vsync_mode()
	var lowp := OS.low_processor_usage_mode
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	OS.low_processor_usage_mode = false
	_setup(sv)
	var t := tier
	while t > maxi(floor_tier, TIER_OFF):
		var key := "basic"
		if t == TIER_STANDARD:
			key = standard_key(phone)
		elif t == TIER_FULL:
			key = "full"
		var m := await _workload(key, t, FORCE_WARM_FRAMES, FORCE_TIMED_FRAMES)
		var cause := ""
		if bool(m.get("aborted", false)):
			cause = "one frame took %s ms, over the %.0f ms limit" % [_ms(float(m["worst"])), FAIL_FRAME_MS]
		elif LAYERS.has(key):
			var mm := await _memory(int(LAYERS[key]))
			if not bool(mm["ok"]):
				cause = "its %d terrain layers could not be stored and read back" % int(LAYERS[key])
		if cause == "":
			break
		notes.append("Forced %s did not hold: %s. 3D dropped to %s." % [TIER_NAMES[t], cause, TIER_NAMES[t - 1]])
		t -= 1
	_teardown()
	DisplayServer.window_set_vsync_mode(vsync)
	OS.low_processor_usage_mode = lowp
	return {"tier": t, "note": " ".join(notes)}

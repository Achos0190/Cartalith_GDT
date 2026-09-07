extends Node
## **Lane PROBE-SIGHT, 2026-09-07 -- one capture harness that can reach all
## three compositions, and that refuses to claim one it did not reach.**
##
## The defect this replaces: `_ds03shot_probe.gd` boots a hardcoded 1920x1080
## `SubViewport`. Under `--force-touch` that aspect is 0.5625, which is under
## `DccShell._PHONE_ASPECT_MAX` (0.60), so the shell it gets is the **phone**;
## without `--force-touch` it is the pointer desktop. **The tablet composition
## is unreachable from it at any flag combination**, so every "verified on
## tablet" claim that cited it rested on tree state, not on a picture.
##
##   Godot_v4.7.1 --path . _sight3_probe.tscn -- --comp pc
##   Godot_v4.7.1 --path . _sight3_probe.tscn -- --comp tablet --force-touch
##   Godot_v4.7.1 --path . _sight3_probe.tscn -- --comp tablet --vp 1600x1000 --force-touch
##   Godot_v4.7.1 --path . _sight3_probe.tscn -- --comp phone  --force-touch
##
## **Headed, never `--headless`.** A `SubViewport` texture is null under the
## dummy rasteriser; `_pcreach_shot.gd` records the same finding.
##
## **One composition per process, and that is not a convenience.** `DccTheme`'s
## `_touch` / `_phone_mode` / `_portrait` are `static var`s, so a second shell
## booted in the same run would overwrite the first one's published density and
## every static widget factory with it. The harness takes exactly one `--comp`.
##
## ## Why the composition is asserted rather than assumed
##
## `--vp` is an *input*. Proving the shell that came back is the one asked for
## needs a reading the other two compositions cannot produce. Three are taken,
## and `_identify()` classifies on them alone -- it never looks at `--vp`,
## `--comp`, or `DccTheme.is_tablet()`:
##
##   reading                                  pc      tablet   phone
##   rail_column.size.x, laid out             39      47       full width
##   _phone_tab_cells.size()                  0       0        PHONE_TABS.size()
##   node_added -> _on_tablet_node_added      no      YES      no
##
## * **The rail width is DRAWN, not set.** `_build_rail()` gives the rail's
##   `PanelContainer` `_scaled(DccTheme.W_RAIL_COLLAPSED)` -- 40 pointer, 48 via
##   `DccTheme.TABLET[40]` -- and the panel's 1 px right border is taken out of
##   the `VBoxContainer` inside it, so the two land one px under. It is read off
##   the laid-out `size`, so a stylebox that failed to apply moves it.
## * **`_phone_tab_cells` is populated by the phone bottom bar and by nothing
##   else**, one entry per `PHONE_TABS` row. `rail_column` itself is
##   *reassigned* by that builder, which is exactly why node presence is no
##   discriminator here and a count is.
## * **The `node_added` connection is made in one place in the tree** -- the
##   `if DccTheme.is_tablet():` tail of `_build_desktop_shell()`. Pointer never
##   connects it; the phone composition never runs that function.
##
## A leg whose classification disagrees with its `--comp` **exits 3 and saves
## no PNG**, because the failure this whole file exists to stop is a capture
## filed under a name that claims a composition it is not.
##
## Read-only apart from the PNGs and the `--out` directory.

## `_build_rail()`: `_scaled(DccTheme.W_RAIL_COLLAPSED)` on the `PanelContainer`,
## less the 1 px right border `DccTheme.panel("panel", {"right": 1})` draws.
## **Pinned as literals on purpose** (`MISTAKES.md`, "never assert a constant
## against itself"): re-deriving these from `DccTheme` would make the assertion
## agree with the table whatever the table said. Measured, `--force-touch` for
## the tablet leg, 2026-09-07. Both mutate: +/-1 on either fails its own leg.
const RAIL_W_PC := 39.0
const RAIL_W_TABLET := 47.0
## `tool_options_row`'s `MarginContainer`: `margin_left` 14 + `margin_right` 14
## (`_build_tool_options()`). Pinned rather than read back off the container for
## the same reason as the two above.
const TOOL_ROW_MARGINS := 28.0

const SEED := 483920
const COMPS: Array[String] = ["pc", "tablet", "phone"]
## Default frames. Tablet portrait is the frame the shipped shell overflowed at
## (`DccTheme.TABLET_PORTRAIT`'s header); phone is the 412 canvas's own.
const DEFAULT_VP := {
	"pc": Vector2i(1920, 1080),
	"tablet": Vector2i(800, 1280),
	"phone": Vector2i(412, 915),
}
## Does the leg need `--force-touch` on the command line? `DccShell._ready()`
## reads that flag off `OS.get_cmdline_user_args()`, which this scene cannot
## write to -- so the harness checks the caller passed it rather than silently
## capturing the pointer shell under a touch name.
const NEEDS_TOUCH := {"pc": false, "tablet": true, "phone": true}

var app: Node
var _vp: SubViewport
var _dir := "user://sight"


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _arg(nm: String, dflt: String) -> String:
	var a := OS.get_cmdline_user_args()
	var i := a.find(nm)
	return String(a[i + 1]) if (i >= 0 and i + 1 < a.size()) else dflt


func _ctl(nm: String) -> Control:
	var v: Variant = app.get(nm)
	return v as Control if v != null else null


## The classification, off the three readings above and nothing else.
## Returns `[name, note]`, where name is `"?"` when it cannot decide.
func _identify() -> Array:
	var cells_v: Variant = app.get("_phone_tab_cells")
	var cells: int = (cells_v as Dictionary).size() if cells_v != null else 0
	var hook: bool = get_tree().node_added.is_connected(
		Callable(app, "_on_tablet_node_added"))
	var rail := _ctl("rail_column")
	if rail == null:
		return ["?", "rail_column absent"]
	var rw: float = rail.size.x
	var note := "rail=%.1f cells=%d tablet_hook=%s" % [rw, cells, str(hook)]
	if cells > 0:
		if cells != DccShell.PHONE_TABS.size():
			return ["?", "%s -- phone nav drew %d cells, PHONE_TABS has %d"
				% [note, cells, DccShell.PHONE_TABS.size()]]
		if hook:
			return ["?", note + " -- phone nav AND the tablet node_added hook"]
		return ["phone", note]
	if hook:
		if not is_equal_approx(rw, RAIL_W_TABLET):
			return ["?", "%s -- tablet hook, but the rail is not %.0f"
				% [note, RAIL_W_TABLET]]
		return ["tablet", note]
	if not is_equal_approx(rw, RAIL_W_PC):
		return ["?", "%s -- no phone nav, no tablet hook, and the rail is not %.0f"
			% [note, RAIL_W_PC]]
	return ["pc", note]


func _band(label: String, c: Control) -> void:
	if c == null:
		print("  %-18s ABSENT" % label)
		return
	print("  %-18s pos=%s size=%s min=%s vis=%s" % [label,
		str(c.global_position.round()), str(c.size.round()),
		str(c.get_combined_minimum_size().round()), str(c.is_visible_in_tree())])


func _ready() -> void:
	var comp := _arg("--comp", "")
	if not (comp in COMPS):
		print("[FATAL] --comp must be one of %s (got '%s')" % [", ".join(COMPS), comp])
		get_tree().quit(2)
		return
	var touch_flag := "--force-touch" in OS.get_cmdline_user_args()
	if touch_flag != bool(NEEDS_TOUCH[comp]):
		print("[FATAL] --comp %s %s --force-touch on the command line; DccShell._ready() reads it and this scene cannot supply it."
			% [comp, "needs" if NEEDS_TOUCH[comp] else "must NOT have"])
		get_tree().quit(2)
		return
	var vp: Vector2i = DEFAULT_VP[comp]
	var raw := _arg("--vp", "")
	if raw != "":
		var p := raw.split("x")
		vp = Vector2i(int(p[0]), int(p[1]))
	_dir = _arg("--out", _dir)
	DirAccess.make_dir_recursive_absolute(_dir)

	_vp = SubViewport.new()
	_vp.size = vp
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(40)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	if app.get("phone_project_picker") != null:
		app.phone_project_picker.hide()
	await _frames(10)

	if "--generate" in OS.get_cmdline_user_args():
		var bridge: Variant = app.bridge
		bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512,
			"grid_h": 384, "archetype": "", "villages": true, "sea_level": 0.42})
		while bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await get_tree().create_timer(1.2).timeout
	await _frames(20)

	var got: Array = _identify()
	print("=== comp=%s vp=%dx%d ===" % [comp, vp.x, vp.y])
	print("[IDENT] intrinsic=%s  (%s)" % [String(got[0]), String(got[1])])
	print("[FLAGS] DccTheme touch=%s tablet=%s phone=%s  (NOT the assertion)" % [
		str(DccTheme.is_touch()), str(DccTheme.is_tablet()), str(DccTheme.is_phone())])
	if String(got[0]) != comp:
		print("[FATAL] asked for '%s', the shell that booted is '%s'. No PNG saved."
			% [comp, String(got[0])])
		get_tree().quit(3)
		return

	print("[BANDS]")
	for nm in ["menu_bar_row", "tool_options_row", "rail_column", "left_dock",
			"viewport_area", "right_dock", "timeline_bar", "status_row"]:
		_band(nm, _ctl(nm))

	## The horizontal fit, and where the floor comes from. Every figure is read
	## off the live tree: nothing here is quoted from a table.
	var tr := _ctl("tool_options_row")
	if tr != null and tr.get_parent() is Control:
		var pad := tr.get_parent() as Control
		var band := pad.get_parent() as Control
		## `TOOL_ROW_MARGINS` is checked against the live container rather than
		## only printed, so mutating it fails visibly -- a pinned literal that
		## nothing compares is not pinned, it is decorative.
		var row_min: float = tr.get_combined_minimum_size().x
		var pad_min: float = pad.get_combined_minimum_size().x
		print("[FLOOR] tool_options_row min.x=%.1f + margins %.0f = %.1f   pad min.x=%.1f (%s)   band min.x=%s" % [
			row_min, TOOL_ROW_MARGINS, row_min + TOOL_ROW_MARGINS, pad_min,
			"OK" if is_equal_approx(row_min + TOOL_ROW_MARGINS, pad_min)
				else "MISMATCH, margins are %.0f" % (pad_min - row_min),
			str(band.get_combined_minimum_size().x) if band != null else "?"])
	var rail := _ctl("rail_column")
	var ld := _ctl("left_dock")
	var va := _ctl("viewport_area")
	var rd := _ctl("right_dock")
	if rail != null and ld != null and va != null and rd != null:
		var sum: float = rail.size.x + ld.size.x + va.size.x + rd.size.x
		print("[ROW]   rail %.0f + left %.0f + vp %.0f + right %.0f = %.0f  frame %d  overflow %.0f" % [
			rail.size.x, ld.size.x, va.size.x, rd.size.x, sum, vp.x,
			maxf(0.0, rd.global_position.x + rd.size.x - float(vp.x))])
	if app is Control:
		print("[SHELL] root min.x=%.1f  frame %d" % [
			(app as Control).get_combined_minimum_size().x, vp.x])

	## **What the frame cannot show.** The capture answers "it renders"; this
	## answers "it can be FOUND". Every visible, text-bearing `Control` whose
	## laid-out right edge is past the frame is unreachable by any gesture --
	## there is no horizontal scroller over the shell's own bands -- and a
	## screenshot only shows the ones that happen to be sliced mid-glyph.
	##
	## The same walk collects `[STRAY]`: visible text drawn in the strip left of
	## the left dock and below the rail, which is chrome nothing owns. Two
	## two-digit labels showed up there in every desktop-composition capture
	## this pass took, and a screenshot cannot name a node.
	var clipped: Array[String] = []
	var stray: Array[String] = []
	var ld_x: float = _ctl("left_dock").global_position.x if _ctl("left_dock") != null else 48.0
	var rail_bot: float = 0.0
	var rl := _ctl("rail_column")
	if rl != null:
		rail_bot = rl.global_position.y + rl.size.y
	var stack: Array[Node] = [app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for ch in n.get_children():
			stack.append(ch)
		if not (n is Control):
			continue
		var c := n as Control
		if not c.is_visible_in_tree() or c.size.x <= 0.0:
			continue
		var t: Variant = c.get("text")
		if t == null or String(t).strip_edges() == "":
			continue
		var txt := String(t).strip_edges().left(38)
		var right: float = c.global_position.x + c.size.x
		if right > float(vp.x):
			clipped.append("%s '%s' x=%.0f..%.0f" % [
				c.get_class(), txt, c.global_position.x, right])
		if right <= ld_x and c.global_position.y > rail_bot:
			stray.append("%s '%s' at (%.0f, %.0f) path=%s" % [
				c.get_class(), txt, c.global_position.x, c.global_position.y,
				String(app.get_path_to(c))])
	clipped.sort()
	stray.sort()
	print("[CLIPPED] %d text controls reach past x=%d" % [clipped.size(), vp.x])
	for s in clipped:
		print("  ", s)
	print("[STRAY]   %d text controls left of x=%.0f and below y=%.0f" % [
		stray.size(), ld_x, rail_bot])
	for s in stray:
		print("  ", s)

	var img := _vp.get_texture().get_image()
	if img == null:
		print("[FATAL] null SubViewport texture -- was this run --headless?")
		get_tree().quit(4)
		return
	var out := "%s/%s_%dx%d.png" % [_dir, comp, vp.x, vp.y]
	img.save_png(out)
	print("[SHOT] ", ProjectSettings.globalize_path(out), "  ",
		img.get_width(), "x", img.get_height())
	print("### DONE ###")
	get_tree().quit(0)

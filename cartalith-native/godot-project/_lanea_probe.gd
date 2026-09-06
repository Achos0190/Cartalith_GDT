extends Node
## **Lane A's measurement harness — the three read-only windows nobody drew.**
##
## `world_data_window.gd`, `performance_window.gd`, `gen_info_dialog.gd` are
## fixed-size `AcceptDialog`s (760x620 / 560x420 / 560x480) that no design
## canvas draws. **`performance_window.gd` no longer exists** -- that no
## canvas drew it is exactly why: `LARGE_ITEM_RULINGS.md` ruling 19
## (2026-09-06) folded it away and moved its rows into `Preferences`. Its
## numbers below are kept as the history that produced the ruling, not as a
## description of the tree. Nothing was measuring whether their content fits those boxes
## at a density other than the one they were authored at.
##
## The hazard is `MISTAKES.md`'s *"read a layout that overflows the screen"*
## row: `world_data_window._build_tab()` sets
## `horizontal_scroll_mode = SCROLL_MODE_DISABLED`, and a disabled axis **folds
## the child's minimum size into the container's own** on that axis. A six
## column table that grows past 760 px therefore does not gain a scrollbar —
## it pushes the dialog wider, with nothing on screen to say so.
##
## Three densities, and each is a **separate process**: `DccTheme._touch` is
## latched for the life of one (`_ds03fit_probe.gd`'s own note). Read the flag
## in the body below before trusting this header — `--force-touch` is consumed
## by `dcc_shell.gd`, `--resolution` by Godot itself, and this probe reads
## **neither**: it sizes its own `SubViewport` from `-- --vp WxH` (parsed in
## `_ready()`), because a headless `--resolution` does not reach a SubViewport.
##
##   Godot_v4.7.1 --headless --path . _lanea_probe.tscn -- --vp 1920x1080
##   Godot_v4.7.1 --headless --path . _lanea_probe.tscn -- --vp 1600x900
##   Godot_v4.7.1 --headless --path . _lanea_probe.tscn -- --vp 2560x1600 --force-touch
##   Godot_v4.7.1 --headless --path . _lanea_probe.tscn -- --vp 1080x2400 --force-touch
##
## Leg 1 is **desktop** (pointer, base tokens). Leg 2 is **laptop**
## (`DccTheme.is_laptop()`, width < `W_LAPTOP_MAX` 1920). Leg 3 is **tablet**
## (`is_tablet()`, where `role_px("fs_prose")` is 14 not 11 and every note in
## these windows grows). Those are the three densities the pass was measured at.
## Leg 4 is the **phone**, a regression leg rather than a fourth density: that
## composition was already measured and declared in `world_data_window.gd`, and
## this pass was not to re-derive it.
##
## No pixels are read anywhere here, so `--headless` is sound — the headless
## no-op `MISTAKES.md` warns about is `ImageTexture.update()`, and every figure
## here is Control geometry (`size`, `global_position`,
## `get_combined_minimum_size()`) plus one theme-colour read. The one place the
## palette shows through is the `text_bright` check, and it resolves its
## expectation through `DccTheme.c()` in the same run, so it is correct on
## either palette (this machine boots light: the check prints `111210`).

var _fail := 0
var _vp: SubViewport
var _vpw := 1920
var _vph := 1080

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _le(name: String, got: float, cap: float, detail: String = "") -> void:
	var good := got <= cap + 0.5
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name,
		"   got=%.0f cap=%.0f" % [got, cap], ("  " + detail) if detail != "" else "")

func _note(name: String, value) -> void:
	print("  --   ", name, "   ", value)

## Every `Control` under `root`, hidden ones included. A collapsed body
## contributes nothing to its parent's minimum *while closed*, so a walk that
## skipped it would pass today and fail on the first click.
func _walk(root: Node, out: Array) -> void:
	if root == null:
		return
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n is Control:
			out.append(n)

## The widest single leaf. A leaf is a `Control` with no `Control` child, which
## is what actually sets a row's floor — a container's minimum is derived.
func _widest_leaf(root: Node) -> Array:
	var all: Array = []
	_walk(root, all)
	var best: Control = null
	var bw := 0.0
	for c in all:
		var ctl: Control = c
		var kids := 0
		for k in ctl.get_children(true):
			if k is Control:
				kids += 1
		if kids > 0:
			continue
		var w: float = ctl.get_combined_minimum_size().x
		if w > bw:
			bw = w
			best = ctl
	var what := "(none)"
	if best != null:
		what = best.get_class()
		if best is Label:
			what += " \"" + (best as Label).text.substr(0, 42) + "\""
	return [bw, what]

## Controls under the tap floor. `DccWidgets.action()` ships 39 px and
## `role_px("chip_min_h")` is 34 — both are filed standing issues, so this
## counts rather than asserts, and names what it counted.
func _under_tap(root: Node) -> Array:
	var all: Array = []
	_walk(root, all)
	var out: Array = []
	for c in all:
		if c is BaseButton and (c as Control).is_visible_in_tree():
			var h: float = (c as Control).size.y
			if h > 0.0 and h < float(DccTheme.PHONE_TAP_MIN):
				out.append("%s %.0f" % [(c as BaseButton).get_class(), h])
	return out

## The bottom edge of the lowest laid-out visible `Control`, in window-local
## coordinates. **This, not `get_contents_minimum_size()`, is the real answer**:
## an autowrapping `Label` reports its minimum height at its minimum *width*
## (`DccWidgets.note()` pins `custom_minimum_size.x = 190`), so a contents
## minimum for a column of notes is an upper bound measured at a width the
## window does not have. `size.y` after a forced layout is the height on screen.
## A `ScrollContainer`'s own content is *supposed* to run past its box — that
## is what a scrollbar is for — so the walk stops there and takes the scroll's
## own rectangle. Without that cut, `world_data_window`'s 240-row table reports
## its whole 993 px column as overflow and the one window that actually cannot
## scroll (`performance_window` on desktop -- deleted 2026-09-06, see the
## header) is lost in the noise.
func _extent(root: Node) -> Vector2:
	var ext := Vector2.ZERO
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		var stop := n is ScrollContainer
		if not stop:
			for c in n.get_children(true):
				stack.append(c)
		if n is Control:
			var ctl: Control = n
			if ctl.is_visible_in_tree():
				var r: Rect2 = Rect2(ctl.global_position, ctl.size)
				ext.x = maxf(ext.x, r.end.x)
				ext.y = maxf(ext.y, r.end.y)
	return ext

func _measure(tag: String, dlg: Window, declared: Vector2i) -> void:
	print("[", tag, "]")
	var cmin: Vector2 = dlg.get_contents_minimum_size()
	_note("dialog", "declared=%s runtime_size=%s visible=%s min_size=%s wrap_controls=%s contents_min=%s"
		% [declared, dlg.size, dlg.visible, dlg.min_size, dlg.wrap_controls, cmin])
	## Forced to the size the FILE declares, then laid out. `dlg.size` inside an
	## embedded `SubViewport` is not the size a real OS window takes, so the box
	## is asserted rather than read.
	dlg.size = declared
	await _frames(8)
	var ext := _extent(dlg)
	_le("%s: laid-out content fits the declared width" % tag, ext.x, float(declared.x),
		"extent=%s" % ext)
	_le("%s: laid-out content fits the declared height" % tag, ext.y, float(declared.y),
		"extent=%s" % ext)
	var wl := _widest_leaf(dlg)
	_note("widest leaf", "%.0f px  %s" % [wl[0], wl[1]])
	var taps := _under_tap(dlg)
	_note("visible buttons under the %d px tap floor" % DccTheme.PHONE_TAP_MIN,
		"%d %s" % [taps.size(), taps])

## The disabled-axis check, stated as its own item because it is the one that
## fails silently: a `ScrollContainer` whose horizontal mode is DISABLED adopts
## its child's minimum width, so the overflow leaves the scroll and reaches the
## dialog.
func _scroll_check(tag: String, root: Node, cap: float) -> void:
	var all: Array = []
	_walk(root, all)
	for c in all:
		if not (c is ScrollContainer):
			continue
		var sc: ScrollContainer = c
		if sc.horizontal_scroll_mode != ScrollContainer.SCROLL_MODE_DISABLED:
			continue
		_le("%s  h-disabled scroll \"%s\" does not fold past the dialog"
			% [tag, sc.name], sc.get_combined_minimum_size().x, cap)

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--vp" and i + 1 < args.size():
			var parts: PackedStringArray = String(args[i + 1]).split("x")
			if parts.size() != 2:
				print("LANEA  !! --vp wants WxH, got '", args[i + 1], "'")
				get_tree().quit(2)
				return
			_vpw = int(parts[0])
			_vph = int(parts[1])
			i += 1
		elif args[i] == "--force-touch":
			pass   ## Read by `dcc_shell.gd`, not here. Accepted so the harness
			       ## does not reject its own tablet leg.
		else:
			print("LANEA  !! unknown argument '", args[i], "'")
			get_tree().quit(2)
			return
		i += 1

	_vp = SubViewport.new()
	_vp.size = Vector2i(_vpw, _vph)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(30)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(20)

	print("LANEA viewport=%dx%d  touch=%s tablet=%s laptop=%s phone=%s  fs_prose=%d"
		% [_vpw, _vph, DccTheme.is_touch(), DccTheme.is_tablet(),
			DccTheme.is_laptop(), DccTheme.is_phone(),
			DccTheme.role_px("fs_prose")])

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 2400:
		await get_tree().process_frame
		waited += 1
	print("LANEA world generated: has_world=%s (%d frames)  settlements=%d"
		% [app.bridge.has_world, waited, app.bridge.settlements().size()])
	await _frames(10)
	if not app.bridge.has_world:
		print("LANEA  !! generate failed -- an empty table measures nothing")
		get_tree().quit(1)
		return

	app.world_data_window.open()
	await _frames(10)
	await _measure("world data", app.world_data_window, Vector2i(760, 620))
	_scroll_check("world data", app.world_data_window, 760.0)
	for tab in ["Settlements", "Provinces", "Economy"]:
		app.world_data_window.open(tab)
		await _frames(6)
		var wl := _widest_leaf(app.world_data_window)
		_note("world data / " + tab + " widest leaf", "%.0f px  %s" % [wl[0], wl[1]])

	## **The ink actually reached a `Label`.** `_cells()` now draws column 0 in
	## `text_bright` and the rest in `text`; asserting the source line would only
	## restate it, so this reads the colour override back off the first data row
	## of the Settlements table. A settlement name is the string being looked
	## for, so the row is found by matching one -- not by index, which would hit
	## the count row or the header.
	if not DccTheme.is_phone():
		var want := DccTheme.c("text_bright")
		var first_name := String((app.bridge.settlements()[0] as Dictionary).get("name", ""))
		var hit: Label = null
		var all: Array = []
		_walk(app.world_data_window, all)
		for c in all:
			if c is Label and String((c as Label).text) == first_name:
				hit = c
				break
		if hit == null:
			_fail += 1
			print("  FAIL world data: no Label carries the first settlement's name '%s'" % first_name)
		else:
			var got: Color = hit.get_theme_color("font_color")
			var good := got.is_equal_approx(want)
			if not good:
				_fail += 1
			print("  ", "ok  " if good else "FAIL",
				" world data: the name column is drawn in text_bright   got=%s want=%s (%s)"
				% [got.to_html(false), want.to_html(false), first_name])
	app.world_data_window.hide()
	await _frames(4)

	## The `performance` case that stood here is gone: `LARGE_ITEM_RULINGS.md` ruling 19 (2026-09-06) folded `performance_window.gd`
## away -- no diagnostics window exists in this design language.
	## Its measurement stays in this file's header as history.

	app.gen_info_dialog.open()
	await _frames(10)
	await _measure("generation info", app.gen_info_dialog, Vector2i(560, 480))
	_scroll_check("generation info", app.gen_info_dialog, 560.0)
	## The counter-check to the two "fits" items above: a control can always
	## be made to fit by collapsing it. Dropping the `TextEdit`'s 360 px floor
	## is only correct if `SIZE_EXPAND_FILL` really hands it the slack, so the
	## dump's own height is asserted rather than inferred. 250 is well under
	## the 345 / 312 measured and well over a collapsed control.
	##
	## **Not asserted on the phone, because there it is 44 px and that is not
	## this pass's doing.** Checked by running this same leg against
	## `git show HEAD:...gen_info_dialog.gd` put back in place: HEAD measures
	## the identical 44 (the `phone_fit()` tap floor, with nothing above it
	## expanding), so the phone dump has always drawn as a one-line strip in a
	## 903 dp window. Reported, not silently folded into a pointer-density
	## assertion that would then be failing for someone else's reason.
	var te: Control = app.gen_info_dialog._text
	_note("generation info: TextEdit height", "%.0f px" % te.size.y)
	if not DccTheme.is_phone() and te.size.y < 250.0:
		_fail += 1
		print("  FAIL generation info: the dump collapsed instead of filling   got=%.0f floor=250" % te.size.y)
	app.gen_info_dialog.hide()
	await _frames(4)

	print("LANEA %s  (%d failed)" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)

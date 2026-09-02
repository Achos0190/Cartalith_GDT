extends Node
## **Tablet parity.** The owner's standing directive is "keep the tablet version
## as close as possible to the windows gui", and `DCC_SHELL_SPEC.md` §13 states
## it flatly: *"Tablet keeps full desktop parity — same regions, same menus,
## same disclosure depth, targets 44–52 px, docks 400 px."*
##
##   Godot_v4.7.1 --path . --resolution 1600x900 _tabletparity_probe.tscn -- --force-touch
##
## This measures that claim rather than restating it, against §1's own tablet
## column, and does it TWICE in one run -- once at tablet 2560x1600, once at
## desktop 1920x1080 -- so "parity" is a comparison between two live
## compositions instead of one table read twice.
##
## Hosted in `SubViewport`s for `_hidpi_probe.gd`'s reason: Windows clamps a
## real window to the desktop work area, so a 2560x1600 request comes back
## smaller and the shell classifies the result as something else entirely.
##
## §1's tablet column, the numbers asserted below:
##
##   Region         desktop   tablet 2560
##   Menu bar          34         52
##   Tool options      34         52
##   Domain rail       40         48
##   Left dock        372        400
##   Right dock    284-340       400
##   Timeline bar      70         88
##   Status bar        26         36

var _fail := 0
var _notes: Array[String] = []

## Set from the diagnosed remainder -- see the assertion's own comment.
const _SMALL_TARGET_BUDGET := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _near(name: String, got: float, want: float, tol: float) -> void:
	var good := absf(got - want) <= tol
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name,
		"   got=%.1f want=%.1f (tol %.1f)" % [got, want, tol])

func _boot(w: int, h: int) -> Node:
	var vp := SubViewport.new()
	vp.size = Vector2i(w, h)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(45)
	return app

## The node that holds the MenuButtons, found by structure rather than by name.
func _find_menu_bar(n: Node) -> Node:
	for c in n.get_children(true):
		if c is MenuButton:
			return n
	for c in n.get_children(true):
		var r := _find_menu_bar(c)
		if r != null:
			return r
	return null

## Every menu title in bar order -- §13's "same menus".
func _menu_titles(app: Node) -> Array:
	var out: Array = []
	var bar := _find_menu_bar(app)
	if bar == null:
		return out
	for c in bar.get_children(true):
		if c is MenuButton:
			out.append(String((c as MenuButton).text))
	return out

## Rows reachable in a popup, recursing submenus -- §13's "same disclosure
## depth". Disabled rows count: a present-but-disabled row is the same depth,
## and skipping them would let a tablet that silently dropped ten still pass.
func _popup_rows(pm: PopupMenu, depth: int = 0) -> int:
	if depth > 6:
		return 0
	var n := 0
	for i in pm.item_count:
		if pm.is_item_separator(i):
			continue
		n += 1
		var sub := pm.get_item_submenu(i)
		if sub != "":
			var node := pm.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				n += _popup_rows(node as PopupMenu, depth + 1)
	return n

func _menu_row_total(app: Node) -> int:
	var bar := _find_menu_bar(app)
	if bar == null:
		return 0
	var total := 0
	for c in bar.get_children(true):
		if c is MenuButton:
			var pm: PopupMenu = (c as MenuButton).get_popup()
			pm.about_to_popup.emit()
			total += _popup_rows(pm)
	return total

## `text`/`path` are diagnostic, not decorative -- `@Button@1240` alone gives no
## way to tell a live shell violation from a dead popup template, and finding
## the 260-under-44 baseline down to 101 needed to know which panel each one
## was actually in.
func _small_targets(app: Node, floor_px: float) -> Array:
	var small: Array = []
	var stack: Array = [app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n is BaseButton and n is Control:
			var ctl := n as Control
			## `is_visible_in_tree()`, not the bare `.visible` property: a
			## collapsed `category()`/`group()` body is hidden by its own
			## container, not by each row inside it, so a row's own `.visible`
			## stays `true` while it is not actually on screen. `.visible` alone
			## over-counted every closed L2/L4 section as a live violation.
			if not ctl.is_visible_in_tree() or ctl.mouse_filter != Control.MOUSE_FILTER_STOP:
				continue
			if n is OptionButton or n is MenuButton or n is ColorPickerButton:
				continue
			if ctl.size.y > 0.0 and ctl.size.y < floor_px:
				var label_text: Variant = n.get("text")
				small.append("%s  h=%.0f  text=%s  path=%s" % [
					String(n.name), ctl.size.y,
					(String(label_text) if label_text != null else ""),
					str(app.get_path_to(n))])
	return small

## §57's own "no visible Label in a dock is below role_px's tablet prose
## size" -- scoped to `left_dock_body`/`right_dock_body`, per role rather than
## a single blanket floor: a dock's Plex readouts and its `header()` section
## labels are legitimately smaller than a prose row's `fs_prose`, and a
## blanket assertion against that one figure would fail on controls that are
## correctly sized to their OWN role. Mirrors `DccShell.tablet_fit()`'s own
## resolution order exactly, so the assertion checks the same thing the fix
## applies -- `DccTheme.ROLE_META` first, then the mono/prose split
## `mono_label()` vs `label()` already makes real.
func _small_dock_labels(root: Node) -> Array:
	var small: Array = []
	if root == null:
		return small
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n is Label:
			var l := n as Label
			## `is_visible_in_tree()` -- see `_small_targets()`'s own comment;
			## the same collapsed-section trap applies to every row label.
			if not l.is_visible_in_tree() or l.text.strip_edges() == "":
				continue
			var role: String = l.get_meta(DccTheme.ROLE_META) if l.has_meta(DccTheme.ROLE_META) \
				else ("fs_readout" if l.has_theme_font_override("font") else "fs_prose")
			var floor_fs := DccTheme.role_px(role)
			var got := l.get_theme_font_size("font_size")
			if got < floor_fs:
				small.append("%s  role=%s got=%d want>=%d  text=%s" % [
					String(l.name), role, got, floor_fs, l.text.left(40)])
	return small

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var forced := "--force-touch" in OS.get_cmdline_user_args()
	print("[BOOT] force-touch=", forced, "  (the tablet leg needs it)")

	print("")
	print("=== TABLET 2560x1600 ===")
	var tapp := await _boot(2560, 1600)
	## `DccApp extends DccShell`, so the app node IS the shell.
	var tshell: Node = tapp
	_ok("classified as touch", DccTheme.is_touch(), true)
	_ok("classified as TABLET", DccTheme.is_tablet(), true)
	_ok("and NOT the phone composition", DccTheme.is_phone(), false)

	print("")
	print("-- SS1 tablet column, measured off the live tree --")
	_near("left dock width", float(tshell.get("_left_width")), 400.0, 1.0)
	_near("right dock width", float(tshell.get("_right_width")), 400.0, 1.0)
	var rail = tshell.rail_region() if tshell.has_method("rail_region") else null
	if rail != null and rail is Control:
		_near("domain rail width", (rail as Control).size.x, 48.0, 3.0)
	else:
		print("  info rail_region() gave nothing to measure")
	## The four fixed-height bands, through the same resolver the shell builds
	## them with -- measuring the built node would also pass if the resolver
	## were right and the builder ignored it, so both are checked: the resolver
	## here, the built node below.
	##
	## **Two of these four assertions were superseded by the 2026-08-31 token
	## re-base and are restated to the new truth rather than left standing.**
	## `design/dcc-environment-2026-08-31/Cartalith DCC Environment.dc.html:25`
	## puts the menu bar at `--menuH:36px` (was 34) and the tool-options bar at
	## `--tbH:40px` (was 34), and `:1819` puts them at 52 and 56 on touch.
	##
	## - The menu bar's desktop key moved 34 -> 36, so the call moved with it.
	##   `_scaled(34)` still answers 52 and is still asked below, because 34 is
	##   now the *dock header's* figure -- see `DccTheme.TABLET`'s header for
	##   why that row was kept rather than deleted.
	## - The tool-options bar left `_scaled()` altogether. Its new desktop
	##   figure, 40, is also `--railW`, and `DccTheme.TABLET` is keyed by the
	##   bare integer -- one key, two required answers (56 and 48). It resolves
	##   through `role_px("h_tool_options")` now (`dcc_shell.gd`'s
	##   `_build_tool_options_bar()`), which is asserted separately below. A
	##   probe that kept asking `_scaled(40)` for the tool bar would have gone
	##   on passing while silently measuring the rail.
	_ok("_scaled(36) menu bar -> 52", tshell.call("_scaled", 36), 52)
	_ok("_scaled(34) dock header -> 52", tshell.call("_scaled", 34), 52)
	_ok("_scaled(70) timeline -> 88", tshell.call("_scaled", 70), 88)
	_ok("_scaled(26) status -> 36", tshell.call("_scaled", 26), 36)
	_ok("_scaled(40) rail -> 48", tshell.call("_scaled", 40), 48)
	## The band `_scaled()` can no longer answer, and the two role figures the
	## re-base moved. Asserted through `DccTheme` rather than the shell because
	## `role_px()` is static and this is the resolver the builder calls.
	_ok("role_px(h_tool_options) -> 56", DccTheme.role_px("h_tool_options"), 56)
	_ok("role_px(h_menu_bar) -> 52", DccTheme.role_px("h_menu_bar"), 52)
	_ok("role_px(w_rail) -> 48", DccTheme.role_px("w_rail"), 48)
	## The fourth density set must NOT fire on a tablet: `is_laptop()` is
	## `narrow and not touch`, and 2560 is not narrow anyway, so this checks
	## both halves at once. If it ever reads true here, a tablet is about to be
	## handed 330/280 px docks meant for a 1366 px mouse-driven window.
	_ok("LAPTOP band stays off on tablet", DccTheme.is_laptop(), false)
	_ok("tablet dock width survives the LAPTOP override",
		DccTheme.role_px("w_left_dock"), 400)

	print("")
	print("-- SS13 targets: tier A 44px (action/category/group/tool), tier B 34px (mode/style chips) --")
	var under44 := _small_targets(tapp, 44.0)
	var under34 := _small_targets(tapp, 34.0)
	print("  info tablet buttons under 44 px tall: ", under44.size(),
		"  (of which ", under34.size(), " are under the absolute 34px tier-B floor;",
		" the other ", under44.size() - under34.size(),
		" sit in the 34-43 band, e.g. `segment()`'s style/mode chips, which are",
		" correctly sized to their own smaller tier and are not a violation)")
	for s in under44:
		print("    ", s)
	## `UNWIRED_FUNCTIONS.md`'s "the tablet interior walk" -- the note this line
	## used to be. `DccWidgets`' factories, `right_dock.gd`, `layers_popover.gd`
	## and `DccShell.tablet_fit()`'s fallback walk between them resolve every
	## control this pass's owned files build. The assertion floors at 34, not
	## 44, because 34 is the one figure NOTHING should ever sit below (`ROLE`'s
	## own `chip_min_h`, tier B's floor) -- a blanket 44px assertion would
	## itself be wrong, flagging `segment()`'s correctly-sized style chips
	## (measured at 37 px here, `CartographyWorkspace`'s RENDER-style preset
	## row) as failures rather than the tier-B success they are.
	##
	## `_SMALL_TARGET_BUDGET` is 0. It did not start there: the first live
	## measurement here was 89 controls under 34 px, and tracing them (the
	## `path=` column below is what made this tractable) found they fell into
	## three real causes, two of them fixed rather than merely explained:
	##   - `register_workspace()` used to walk `panel` before
	##     `app.gd::_register_workspaces()`'s very next line, `ws.setup(...)`,
	##     had built anything into it -- a bare `WorldWorkspace.new()` has no
	##     rows to floor. Fixed by deferring the walk (`register_workspace()`'s
	##     own comment has the measurement).
	##   - `DccTheme.header()`/`DccWidgets.note()` only *tagged* a label with
	##     the role a walk would need to floor it, and `right_dock.gd` is not a
	##     `register_workspace()` panel, so nothing ever read the tag. Fixed by
	##     resolving both at construction instead of leaving it to a walk that
	##     does not reach that dock.
	##   - `DccWidgets.modal_button()` (a dialog's Open/Cancel pair) had never
	##     been touched at all. Fixed the same way `action()` was.
	## What is left standing at 34-43 px (informational, not a failure) is one
	## raw close-✕ button `open_project_dialog.gd` builds without going through
	## any shared factory -- named here because the next reader should not have
	## to re-diagnose it.
	_ok("tablet buttons under the absolute 34px tier-B floor stay at the known residual",
		under34.size(), _SMALL_TARGET_BUDGET)

	print("")
	print("-- SS13 dock labels: no visible Label below its own ROLE floor --")
	var small_left := _small_dock_labels(tshell.get("left_dock_body"))
	var small_right := _small_dock_labels(tshell.get("right_dock_body"))
	var small_labels := small_left + small_right
	print("  info dock labels under their tablet ROLE size: ", small_labels.size())
	for s in small_labels:
		print("    ", s)
	_ok("no visible dock Label below its own ROLE floor", small_labels.size(), 0)

	var t_titles := _menu_titles(tapp)
	var t_rows := _menu_row_total(tapp)
	print("  info tablet menus: ", t_titles)
	print("  info tablet menu rows (submenus included): ", t_rows)

	print("")
	print("=== DESKTOP 1920x1080 (same process) ===")
	## `DccTheme._touch` is latched for the life of the process by design (see
	## its own comment), so this leg cannot un-touch the theme. What it CAN
	## measure is the two things SS13 actually promises are identical, and
	## those are exactly what parity means here: the menu titles and the total
	## disclosure depth.
	var dapp := await _boot(1920, 1080)
	var d_titles := _menu_titles(dapp)
	var d_rows := _menu_row_total(dapp)
	print("  info desktop menus: ", d_titles)
	print("  info desktop menu rows (submenus included): ", d_rows)

	print("")
	print("=== SS13 PARITY: same menus, same disclosure depth ===")
	_ok("the same menus in the same order",
		",".join(PackedStringArray(t_titles)), ",".join(PackedStringArray(d_titles)))
	_ok("seven of them", t_titles.size(), 7)
	_ok("the same number of reachable rows", t_rows, d_rows)
	_ok("and that number is not trivially small", t_rows > 100, true)

	print("")
	print("_tabletparity_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	for n in _notes:
		print("  note ", n)
	get_tree().quit(1 if _fail > 0 else 0)

extends Node
## **Tablet parity.** The owner's standing directive is "keep the tablet version
## as close as possible to the windows gui", and `DCC_SHELL_SPEC.md` §13 states
## it flatly: *"Tablet keeps full desktop parity — same regions, same menus,
## same disclosure depth, targets 44–52 px, docks 400 px."*
##
## **"Same menus" is RE-SPECIFIED here 2026-09-13, not restated.** The owner
## ruled canvas adoption 2026-09-07 ("tablet still is the pc layout instead of
## the design I gave"; "All designs layouts and styles should match 100%"),
## and `Cartalith Tablet.dc.html` / `TABLET_UI_SPEC.md` §2.1 draw exactly four
## top-level tablet menus -- the overflow square, then File, World, Data --
## not the shipped seven. `CLAUDE.md`'s "the newer canvas wins" makes that
## canvas the authority over §13's older top-level wording. What §13 actually
## protects -- no command lost, i.e. disclosure depth -- is unchanged and is
## what SS13 below asserts directly against `CommandIndex`
## (`LARGE_ITEM_RULINGS.md` DS-03, "keep everything, reflow only"), rather than
## an exact title-list match that a deliberate reflow would always fail.
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

## Part D, 2026-09-13. Pressing the real theme/units radios (14 `id_pressed`
## emits across this file) is deliberate -- `_units_popup`'s checked-state is
## a shadow only that app's own `_on_units_choice()` refreshes, so setting
## `DccSettings` directly leaves it stale (see the Gate B1 comment above) --
## but every one of those presses also persists `DccSettings`'s backing
## `ConfigFile` to the REAL `user://cartalith_settings.cfg`, on every run,
## whether or not anything eventually failed. The in-memory value this file
## already restores (units back to `orig_units`, theme forced to "light" --
## the standing convention, not a per-run snapshot); the FILE ON DISK is a
## separate concern, because a `ConfigFile` re-save is not guaranteed
## byte-identical to itself even when every key it carries ends up equal
## (section order, float formatting, …). Snapshotting the raw bytes once, up
## front, and overwriting with them verbatim at the end sidesteps that
## entirely rather than trying to make the writer reproduce itself.
## **Mutation, measured -- and it is not the simple story it looks like.**
## Neutering `_restore_settings()` to a no-op does NOT show up as a changed
## file hash on a machine whose theme is already "light" and whose units are
## already whatever `orig_units` reads: this probe converges the FILE's own
## theme to "light" and units back to `orig_units` regardless of whether the
## final byte-restore below ever runs, and `ConfigFile.save()` is
## deterministic for identical key/value content -- no timestamp, no nonce --
## so the coincidence hides the defect. Proven for real with a fixture: set
## the live file to `mode="dark"` first (a stand-in for an owner who does not
## use this machine's usual light preference), then the SAME mutant leaves it
## at `mode="light"` -- a real, permanent change to a real preference -- while
## the fixed probe (this file) restores the exact original dark bytes even
## though it forced `DccTheme` through dark -> light -> dark -> light
## internally to run Gate A3. Measured 2026-09-13, both legs PASS, hashes
## compared directly rather than trusted from either run's own report.
var _settings_path := "user://cartalith_settings.cfg"
var _settings_existed := false
var _settings_backup := PackedByteArray()

func _backup_settings() -> void:
	_settings_existed = FileAccess.file_exists(_settings_path)
	if _settings_existed:
		_settings_backup = FileAccess.get_file_as_bytes(_settings_path)
		print("[settings] backed up %d byte(s) of %s before any radio press"
			% [_settings_backup.size(), _settings_path])
	else:
		print("[settings] %s does not exist yet -- will be removed afterward if this run creates it"
			% _settings_path)

## Restores byte-for-byte and ASSERTS it, rather than trusting the sequence of
## presses above to have netted out to the original file -- the whole point
## of this fix is that trust was exactly what let this probe rewrite the
## owner's file on every run while every logical VALUE still came back right.
func _restore_settings() -> void:
	if _settings_existed:
		var f := FileAccess.open(_settings_path, FileAccess.WRITE)
		f.store_buffer(_settings_backup)
		f.close()
		var now := FileAccess.get_file_as_bytes(_settings_path)
		var same: bool = now == _settings_backup
		print("[settings] restored -- byte-identical to the pre-probe file: ", same)
		_ok("settings file restored byte-for-byte", same, true)
	elif FileAccess.file_exists(_settings_path):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(_settings_path))
		print("[settings] removed %s (this run created it; it did not exist before)" % _settings_path)

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

## The `MenuButton` anywhere under `root` whose own `.text` is `title` --
## `get_children(true)` throughout, the same INTERNAL-children trap
## `command_index.gd`'s own header cites for why a `MenuButton`'s popup is
## invisible to the default walk.
func _menu_button(root: Node, title: String) -> MenuButton:
	if root is MenuButton and (root as MenuButton).text == title:
		return root as MenuButton
	for c in root.get_children(true):
		var r := _menu_button(c, title)
		if r != null:
			return r
	return null

## The child `PopupMenu` of the row in `pm` whose own TEXT is `title` -- a
## title lookup rather than an id lookup because `_add_submenu()`'s own
## submenus (Edit/Assets/Preferences/Window/Help, and now Theme) are named by
## title, not by a `DccMenus.ID_*` constant the way leaf actions are.
func _submenu_by_title(pm: PopupMenu, title: String) -> PopupMenu:
	if pm == null:
		return null
	for i in pm.item_count:
		if pm.is_item_separator(i):
			continue
		if pm.get_item_text(i) == title:
			var sub := pm.get_item_submenu(i)
			if sub != "":
				var node := pm.get_node_or_null(NodePath(sub))
				if node is PopupMenu:
					return node as PopupMenu
	return null

## The submenu mounted on item `idx`, or null. Index-based rather than
## `_submenu_by_title()`'s text match: Units' own row text is NOT stable --
## `_menu_row_total()` above already fired `about_to_popup` on every top-level
## popup (tablet's ☰ AND desktop's Preferences) to force dynamic labels to
## refresh before counting rows, so by the time this probe reaches Part B the
## row already reads "Units — kilometres" / "Units   Kilometres", not bare
## "Units" -- a title search for "Units" now finds nothing. Found the hard
## way: `_submenu_by_title(overflow_popup, "Units")` returned null and every
## Part B assertion downstream of it read a frozen, never-updated "kilometres".
func _submenu_at_index(pm: PopupMenu, idx: int) -> PopupMenu:
	if pm == null or idx < 0 or idx >= pm.item_count:
		return null
	var sub := pm.get_item_submenu(idx)
	if sub == "":
		return null
	var node := pm.get_node_or_null(NodePath(sub))
	return node as PopupMenu if node is PopupMenu else null

## The Units row's item index in a popup that also carries other rows --
## found by its own live-stamped prefix rather than a fixed index, since
## `about_to_popup` rewrites its text every time units change (both the
## tablet ☰ copy, Part B, and Preferences' own `_stamp_pref_values()` copy).
func _find_units_row(pm: PopupMenu) -> int:
	if pm == null:
		return -1
	for i in pm.item_count:
		if not pm.is_item_separator(i) and pm.get_item_text(i).begins_with("Units"):
			return i
	return -1

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

## Every menu-sourced COMMAND title `CommandIndex` finds on `app`, as a
## MULTISET (title -> occurrence COUNT) -- reusing `command_index.gd`'s own
## build (`_ensure_command_index()`, grepped at its symbol first) rather than
## re-walking popups a second way, so this probe cannot disagree with what
## the shipped app itself indexes.
##
## **Counted, not a membership `true` -- found 2026-09-13.** A title is not
## unique: Help's "Keyboard shortcuts…" (`ID_HELP_SHORTCUTS`, 72) and
## Preferences' own "Keyboard shortcuts…" (`ID_PREF_SHORTCUTS`, 605) are two
## real, differently-numbered commands that happen to share a title. A membership
## dict collapses them to one key, so a mutant dropping id 72 alone left
## "Keyboard shortcuts…" still present (Preferences' copy) and the parity
## check below PASSED -- it never noticed a row went missing. Counting
## occurrences instead makes desktop's side 2 and a 72-dropped tablet's side
## 1, which the comparison below now catches as a real shortfall.
##
## `kind` filters to **"menu" only, not "readout" too** -- found empirically,
## not assumed: a first version of this filter included "readout" and failed
## on "Working set 386 MB of 31.2 GB" being absent from the other boot's set,
## because that is a *live value* baked into the title by design
## (`command_index.gd`'s own header: "the row IS the result") and the two
## compositions are two separate `EngineBridge` instances measured at two
## different moments, so of course the number differs. That is not a menu
## reachability defect and this probe has no business asserting it is one.
## Generation parameters and `EXTRAS` are excluded the same way `kind !=
## "menu"` already excludes readouts: they do not depend on the menu bar at
## all and would match trivially on both compositions either way.
func _command_titles(app: Node) -> Dictionary:
	var idx = app.call("_ensure_command_index")
	var out := {}
	for row in idx.all():
		if String(row.get("kind", "")) == "menu":
			var t := String(row.get("title", ""))
			out[t] = int(out.get(t, 0)) + 1
	return out

## Every accelerator reachable off `app`'s menu bar, recursing into submenus
## the same way `ShortcutsDialog._collect_from_menus()` does (grepped, not
## assumed) -- keyed by accelerator code, valued with every "Menu > row" that
## carries it. A code with more than one entry is a real double-binding:
## Godot's `PopupMenu` does not itself refuse two rows the same accelerator,
## the input system just picks one and the other's shortcut silently does
## nothing.
func _accelerators(app: Node) -> Dictionary:
	var out := {}
	var bar := _find_menu_bar(app)
	if bar == null:
		return out
	for c in bar.get_children(true):
		if c is MenuButton:
			_walk_accel((c as MenuButton).get_popup(), String((c as MenuButton).text), out)
	return out

func _walk_accel(popup: PopupMenu, menu_name: String, out: Dictionary) -> void:
	for i in popup.item_count:
		var accel: int = popup.get_item_accelerator(i)
		if accel != 0:
			if not out.has(accel):
				out[accel] = []
			(out[accel] as Array).append("%s > %s" % [menu_name, popup.get_item_text(i)])
		var sub := popup.get_item_submenu(i)
		if sub != "":
			var node := popup.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				_walk_accel(node as PopupMenu, menu_name, out)

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
	## Part D: before ANYTHING else, including the extension check right
	## below -- the theme/units radios are pressed for real far below, and
	## nothing must run ahead of this that could itself touch the settings
	## file.
	_backup_settings()
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); _restore_settings(); get_tree().quit(1); return
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
	print("=== SS13 PARITY: same commands, same disclosure depth, tablet's own menu bar ===")
	## **Desktop's own menu bar is unchanged and asserted directly** -- the
	## `else` branch in `menus.gd::build()` is the original seven calls,
	## untouched by this batch.
	_ok("desktop still has its shipped seven menus, unchanged",
		",".join(PackedStringArray(d_titles)),
		"File,Edit,Assets,Data,Preferences,Window,Help")
	## **Tablet's own menu bar is re-specified to the canvas, not restated.**
	## `Cartalith Tablet.dc.html` / `TABLET_UI_SPEC.md` §2.1 draw exactly the
	## overflow square then File, World, Data -- four top-level menus, not
	## seven -- per the owner's 2026-09-07 canvas-adoption ruling (see this
	## file's header).
	_ok("tablet's top-level menus are the canvas's own: overflow, File, World, Data",
		",".join(PackedStringArray(t_titles)), "☰,File,World,Data")

	print("")
	print("-- every command title reachable on desktop is still reachable on tablet --")
	## The substance of "same disclosure depth", measured directly against
	## `CommandIndex` rather than inferred from a row COUNT of ALL rows (that
	## count cannot distinguish "everything moved one level deeper" from "ten
	## rows were quietly dropped and ten unrelated ones added").
	##
	## **By (title, occurrence count), not by title membership -- found
	## 2026-09-13.** `_command_titles()`'s own header has the full account: a
	## membership-only comparison let a mutant dropping Help's "Keyboard
	## shortcuts…" (id 72) PASS, because Preferences' OWN "Keyboard
	## shortcuts…" (id 605) is a second, different row with the same title and
	## kept the key present. Comparing per-title counts catches it: desktop
	## wants 2, a 72-dropped tablet has 1, and that shortfall is what
	## `missing` reports below -- proved by the mutation this batch ran
	## (comment it out, re-run, this assertion FAILS; restored after).
	var d_cmds := _command_titles(dapp)
	var t_cmds := _command_titles(tapp)
	var missing: Array = []
	for title in d_cmds:
		var want: int = int(d_cmds[title])
		var got: int = int(t_cmds.get(title, 0))
		if got < want:
			missing.append("%s  (desktop x%d, tablet x%d)" % [title, want, got])
	missing.sort()
	print("  info desktop command titles: ", d_cmds.size(),
		"   tablet command titles: ", t_cmds.size())
	if not missing.is_empty():
		print("  MISSING ON TABLET (", missing.size(), "):")
		for m in missing:
			print("    ", m)
	_ok("every desktop command title survives on tablet, by occurrence count", missing.size(), 0)

	print("")
	print("-- every accelerator on tablet's menu bar is bound to exactly one row --")
	## The specific failure the ☰ refactor risks: a row promoted to ☰ that is
	## ALSO still built somewhere else would bind its accelerator twice.
	## Desktop is checked too, as a negative control -- it was not
	## restructured, so 0 there is not a coincidence of what this probe
	## happens to check.
	var t_accel := _accelerators(tapp)
	var d_accel := _accelerators(dapp)
	var t_dupes: Array = []
	for code in t_accel:
		if (t_accel[code] as Array).size() > 1:
			t_dupes.append("%s: %s" % [OS.get_keycode_string(code), ", ".join(t_accel[code])])
	var d_dupes: Array = []
	for code in d_accel:
		if (d_accel[code] as Array).size() > 1:
			d_dupes.append("%s: %s" % [OS.get_keycode_string(code), ", ".join(d_accel[code])])
	print("  info tablet accelerators: ", t_accel.size(), "   desktop accelerators: ", d_accel.size())
	for d in t_dupes:
		print("    DUPLICATE (tablet) ", d)
	for d in d_dupes:
		print("    DUPLICATE (desktop) ", d)
	_ok("no accelerator is bound to more than one tablet row", t_dupes.size(), 0)
	_ok("negative control: desktop has no duplicate either", d_dupes.size(), 0)

	print("")
	print("-- every desktop accelerator is also reachable on tablet --")
	## Added 2026-09-13, alongside the (title, count) fix above: uniqueness
	## alone says nothing about whether a rebound-off row's key is still bound
	## to ANYTHING. A code tablet never rebuilds (dropped along with its row)
	## would still read 0 duplicates on both sides and pass the checks above.
	var accel_missing: Array = []
	for code in d_accel:
		if not t_accel.has(code):
			accel_missing.append("%s: %s" % [OS.get_keycode_string(code), ", ".join(d_accel[code])])
	if not accel_missing.is_empty():
		print("  MISSING ON TABLET (", accel_missing.size(), "):")
		for m in accel_missing:
			print("    ", m)
	_ok("every desktop accelerator is bound to some row on tablet", accel_missing.size(), 0)

	print("")
	print("=== MENUS lane Part A: tablet flat 'Toggle theme' + Follow system stays reachable ===")
	var overflow_mb := _menu_button(tshell, "☰")
	_ok("tablet has a ☰ overflow menu button", overflow_mb != null, true)
	var overflow_popup: PopupMenu = overflow_mb.get_popup() if overflow_mb != null else null
	var toggle_idx := overflow_popup.get_item_index(DccMenus.ID_TABLET_TOGGLE_THEME) if overflow_popup != null else -1
	_ok("☰ has a 'Toggle theme' row (canvas VIEW section)", toggle_idx >= 0, true)
	if toggle_idx >= 0:
		_ok("'Toggle theme' text matches the canvas exactly",
			overflow_popup.get_item_text(toggle_idx), "Toggle theme")
		_ok("'Toggle theme' carries no visible accelerator (canvas sc:'')",
			overflow_popup.get_item_accelerator(toggle_idx), 0)
		_ok("'Toggle theme' is not disabled", overflow_popup.is_item_disabled(toggle_idx), false)
	## The old top-level Theme SUBMENU must be gone from ☰'s own top level --
	## that is what makes this a flat row rather than a second copy beside a
	## submenu no one asked to keep there.
	var theme_at_top := _submenu_by_title(overflow_popup, "Theme") if overflow_popup != null else null
	_ok("no 'Theme' SUBMENU left at ☰'s own top level (it is flat now)", theme_at_top == null, true)

	var prefs_sub := _submenu_by_title(overflow_popup, "Preferences") if overflow_popup != null else null
	_ok("☰ still has a nested Preferences submenu", prefs_sub != null, true)
	var theme_sub := _submenu_by_title(prefs_sub, "Theme") if prefs_sub != null else null
	_ok("Follow system's real home: ☰ ▸ Preferences ▸ Theme exists", theme_sub != null, true)
	var theme_item_titles: Array = []
	if theme_sub != null:
		for i in theme_sub.item_count:
			if not theme_sub.is_item_separator(i):
				theme_item_titles.append(theme_sub.get_item_text(i))
	_ok("☰ ▸ Preferences ▸ Theme carries all three choices",
		",".join(PackedStringArray(theme_item_titles)), "Dark,Light,Follow system")

	## The aggregate (title, count) comparison above already proves this by
	## occurrence count; named individually here so a reader of THIS section
	## does not have to cross-reference the earlier block for the one claim
	## this whole part exists to make.
	_ok("'Follow system' reachable AND counted in tablet's own command index",
		int(t_cmds.get("Follow system", 0)) >= 1, true)
	_ok("'Dark' reachable AND counted in tablet's own command index",
		int(t_cmds.get("Dark", 0)) >= 1, true)
	_ok("'Light' reachable AND counted in tablet's own command index",
		int(t_cmds.get("Light", 0)) >= 1, true)

	## Searchable by the WORD "theme" itself -- found by none of Dark/Light/
	## Follow system's own titles, nor by "☰"/"Preferences" as a group. See
	## `command_index.gd`'s `EXTRAS` entry's own header for why a submenu-
	## opening row is never indexed by `_walk_popup()` and why this needs its
	## own pointer row rather than relying on the leaves already being found.
	var t_idx = tapp.call("_ensure_command_index")
	var theme_hit_titles: Array = []
	for r in t_idx.search("theme"):
		theme_hit_titles.append(String(r["title"]))
	_ok("searching 'theme' finds the EXTRAS pointer row", theme_hit_titles.has("Theme"), true)

	print("")
	print("-- Toggle theme functions, and the nested submenu is really wired, not just present --")
	if theme_sub != null:
		theme_sub.id_pressed.emit(DccMenus.ID_PREF_THEME_DARK)
		await _frames(1)
		_ok("forced Dark via the nested submenu", DccTheme.is_dark(), true)
	if overflow_popup != null and toggle_idx >= 0:
		overflow_popup.id_pressed.emit(DccMenus.ID_TABLET_TOGGLE_THEME)
		await _frames(1)
		_ok("Toggle theme flips dark -> light (reads the DRAWN palette, not the stored mode)",
			DccTheme.is_dark(), false)
		overflow_popup.id_pressed.emit(DccMenus.ID_TABLET_TOGGLE_THEME)
		await _frames(1)
		_ok("Toggle theme flips light -> dark (pure binary flip, matches the canvas's own light:!s.light)",
			DccTheme.is_dark(), true)
	## Gate A3 and a wiring check in one assertion: forcing Light through
	## ☰ ▸ Preferences ▸ Theme is both this session's required restore and proof
	## `_theme_popup.id_pressed.connect(_on_theme_choice)` runs on THIS copy of
	## the popup -- a structural-only check (item count/titles, above) cannot
	## tell a wired radio from a decoration.
	if theme_sub != null:
		theme_sub.id_pressed.emit(DccMenus.ID_PREF_THEME_LIGHT)
		await _frames(1)
	_ok("Gate A3: machine theme preference is light when this probe finishes",
		DccTheme.is_dark(), false)
	## Part D, 2026-09-13. The assertion above reads the DRAWN palette
	## (`DccTheme.is_dark()`), which the "Toggle theme" flip a few lines up
	## already proved can move independently of the STORED mode (its own
	## comment: "reads the DRAWN palette, not the stored mode"). Gate A3 as
	## filed is about the persisted preference, so it is asserted directly,
	## against the same `DccSettings.theme_mode()` `menus.gd` reads to seed
	## every new `DccMenus` with (`_theme_mode = DccSettings.theme_mode()`).
	_ok("Gate A3: STORED theme preference is light when this probe finishes",
		DccSettings.theme_mode(), "light")

	print("")
	print("=== MENUS lane Part B: tablet Units row matches the canvas's 'Units — value' wording ===")
	var units_idx2 := _find_units_row(overflow_popup)
	_ok("tablet ☰ still has its own top-level Units row", units_idx2 >= 0, true)
	## **Pressed through the real radio items, not `DccSettings.set_units_mode()`
	## directly.** Each app's `_units_popup` keeps its OWN checked-state,
	## refreshed only by that app's own `_on_units_choice()` -- the same
	## write-only-shadow shape Part D's report describes for the five Window
	## region checks. Setting the shared store directly moves `DccSettings`
	## but leaves a stale popup shadow behind, and a first version of this
	## probe measured exactly that (three straight false FAILs, all reading
	## back "kilometres" no matter what was just set) before this fix.
	var t_units_sub := _submenu_at_index(overflow_popup, units_idx2)
	_ok("tablet ☰'s Units row has its own submenu", t_units_sub != null, true)
	var orig_units := DccSettings.units_mode()
	if t_units_sub != null:
		t_units_sub.id_pressed.emit(DccMenus.ID_PREF_UNITS_KM)
	if overflow_popup != null:
		overflow_popup.about_to_popup.emit()
	_ok("tablet Units row: km reads exactly the canvas's own wording",
		overflow_popup.get_item_text(units_idx2) if units_idx2 >= 0 else "<row not found>",
		"Units — kilometres")
	if t_units_sub != null:
		t_units_sub.id_pressed.emit(DccMenus.ID_PREF_UNITS_MI)
	if overflow_popup != null:
		overflow_popup.about_to_popup.emit()
	_ok("tablet Units row: mi reads exactly the canvas's own wording",
		overflow_popup.get_item_text(units_idx2) if units_idx2 >= 0 else "<row not found>",
		"Units — miles")
	if t_units_sub != null:
		t_units_sub.id_pressed.emit(DccMenus.ID_PREF_UNITS_NMI)
	if overflow_popup != null:
		overflow_popup.about_to_popup.emit()
	_ok("tablet Units row: nmi extends the canvas's own wording consistently (rule 2, no canvas example)",
		overflow_popup.get_item_text(units_idx2) if units_idx2 >= 0 else "<row not found>",
		"Units — nautical miles")
	## Restored on TABLET's own popup, by the same real radio path -- and
	## `DccSettings` is a real, persisted `ConfigFile`, not scratch state
	## private to this process.
	if t_units_sub != null:
		match orig_units:
			"km": t_units_sub.id_pressed.emit(DccMenus.ID_PREF_UNITS_KM)
			"mi": t_units_sub.id_pressed.emit(DccMenus.ID_PREF_UNITS_MI)
			"nmi": t_units_sub.id_pressed.emit(DccMenus.ID_PREF_UNITS_NMI)

	## Gate B1: PC's OWN Preferences Units suffix format, on PC's OWN popup
	## instance (each app builds its own `_units_popup`; `DccSettings` is
	## shared but each app's CHECKED-STATE is refreshed only by that app's own
	## `_on_units_choice()`, so this presses desktop's own radio rather than
	## relying on tablet's press above to have moved it). Must still use the
	## shared `PREF_VALUE_SEP` (three spaces, title case), untouched by the
	## tablet-only rewrite above.
	var dprefs_mb := _menu_button(dapp, "Preferences")
	var dprefs_popup: PopupMenu = dprefs_mb.get_popup() if dprefs_mb != null else null
	var d_units_idx := _find_units_row(dprefs_popup)
	var d_units_sub := _submenu_at_index(dprefs_popup, d_units_idx)
	if d_units_sub != null:
		d_units_sub.id_pressed.emit(DccMenus.ID_PREF_UNITS_MI)
	if dprefs_popup != null:
		dprefs_popup.about_to_popup.emit()
	_ok("Gate B1: PC's own Preferences Units suffix format is untouched",
		dprefs_popup.get_item_text(d_units_idx) if (dprefs_popup != null and d_units_idx >= 0) else "<row not found>",
		"Units" + DccMenus.PREF_VALUE_SEP + "Miles")
	## **Final restore, after every mutation above.** The tablet-side restore
	## a few lines up was not the last write: this Gate B1 press just set the
	## shared `DccSettings` store to "mi" again on desktop's own popup. One
	## more press, on whichever popup is at hand, leaves the real persisted
	## `ConfigFile` at `orig_units` when this probe exits.
	if d_units_sub != null:
		match orig_units:
			"km": d_units_sub.id_pressed.emit(DccMenus.ID_PREF_UNITS_KM)
			"mi": d_units_sub.id_pressed.emit(DccMenus.ID_PREF_UNITS_MI)
			"nmi": d_units_sub.id_pressed.emit(DccMenus.ID_PREF_UNITS_NMI)
	_ok("units mode restored to its pre-probe value", DccSettings.units_mode(), orig_units)

	print("")
	print("-- reachable row totals, informational: a reflow moves rows, it does not erase them --")
	print("  info tablet reachable rows: ", t_rows, "   desktop reachable rows: ", d_rows,
		"   delta: ", t_rows - d_rows)

	## Part D: the LAST thing that happens, after every assertion above
	## (including the ones this very call could otherwise invalidate) has
	## already read whatever it needed to.
	_restore_settings()
	## Independent of `_restore_settings()`'s own internal assertion --
	## deliberately OUTSIDE that function, so a mutant that neuters the whole
	## body (an early `return`, skipping its write-back AND its own `_ok()`
	## call in the same stroke) is still caught here rather than reporting a
	## clean PASS. Compares against `_settings_backup` directly; if the file
	## never existed before this run, there is nothing to compare and the
	## no-such-file branch inside `_restore_settings()` already handled it.
	if _settings_existed:
		_ok("Part D outer check: settings file is byte-identical to the pre-probe backup",
			FileAccess.get_file_as_bytes(_settings_path) == _settings_backup, true)

	print("")
	print("_tabletparity_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	for n in _notes:
		print("  note ", n)
	get_tree().quit(1 if _fail > 0 else 0)

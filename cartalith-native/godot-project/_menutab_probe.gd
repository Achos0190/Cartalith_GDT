extends Node
## MENUS lane probe, 2026-09-12. Measures the tablet menu-bar restructuring
## (ceiling: canvas draws OVERFLOW/File/World/Data) against whatever the
## booted mode actually ships, using the REAL production classes rather than
## a reimplementation:
##
##   - `CommandIndex` (shell/command_index.gd), exactly as the phone search
##     field builds it, for the searchable-title multiset.
##   - a recursive accelerator walk over the live `menu_bar_row`, mirroring
##     `ShortcutsDialog._collect_from_menus()`'s own algorithm (duplicated
##     rather than instantiating a Window dialog headless, since AcceptDialog
##     lifecycle off-tree is untested territory this probe does not need).
##
## Run once per mode and diff the two dumps -- this prints, it does not
## compare:
##
##   godot --path . _menutab_probe.tscn                                    (desktop/phone)
##   godot --path . _menutab_probe.tscn -- --force-touch --vp 1600x1000    (tablet)
##
## Windowed, per this batch's brief ("Measure menu_bar_row's combined minimum
## width on tablet before and after, windowed") -- not `--headless`, which
## never delivers a texture for a pixel probe and is not tested here for a
## plain layout query, so the instruction is followed literally rather than
## re-derived. Reads nothing; writes nothing; changes no behaviour.

var app: Node
var _vp: SubViewport

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _ready() -> void:
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	var vw := int(parts[0])
	var vh := int(parts[1])

	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(6)

	print("=== vp=%dx%d touch=%s tablet=%s phone=%s ===" % [
		vw, vh, str(DccTheme.is_touch()), str(DccTheme.is_tablet()), str(DccTheme.is_phone())])

	## Top-level menu titles, the same walk `_tabspec_probe.gd` uses.
	var titles: Array[String] = []
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			titles.append((ch as MenuButton).text)
	print("[menus] count=%d %s" % [titles.size(), ", ".join(titles)])

	## The one figure this batch's brief names explicitly.
	print("[menu_bar_row] min_w=%.1f" % app.menu_bar_row.get_combined_minimum_size().x)
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			print("  %-8s size=%s min=%s" % [(ch as MenuButton).text,
				str((ch as MenuButton).size), str((ch as MenuButton).get_combined_minimum_size())])

	## The searchable catalogue, built exactly as production code builds it.
	var idx := CommandIndex.new()
	idx.build(app, app.bridge)
	var cmd_titles: Array[String] = []
	for r in idx.all():
		cmd_titles.append(String(r["title"]))
	cmd_titles.sort()
	print("[command_index] count=%d" % cmd_titles.size())
	for t in cmd_titles:
		print("T|%s" % t)

	## Accelerators: (item text, accelerator) for every bound row, at any
	## submenu depth -- the same recursion `command_index.gd::_walk_popup`
	## and `shortcuts_dialog.gd::_walk_popup` both already do, duplicated here
	## as a plain array walk so this probe carries no dependency on either
	## file's own correctness (it is measuring both from outside).
	var accels: Array = []
	_walk_accels(app.menu_bar_row, accels)
	accels.sort_custom(func(a, b): return String(a[0]) < String(b[0]))
	print("[accelerators] count=%d" % accels.size())
	var seen := {}
	var dupes := 0
	for a in accels:
		var key: int = a[1]
		if seen.has(key):
			dupes += 1
			print("DUP_ACCEL|%s|%s|also:%s" % [OS.get_keycode_string(key), a[0], seen[key]])
		seen[key] = a[0]
		print("A|%s|%s" % [OS.get_keycode_string(key), a[0]])
	print("[dupe_accelerators] count=%d" % dupes)

	print("=== end ===")
	get_tree().quit()

func _walk_accels(root: Node, out: Array) -> void:
	var buttons: Array = []
	_gather_menu_buttons(root, buttons)
	for mb in buttons:
		var popup: PopupMenu = (mb as MenuButton).get_popup()
		if popup != null:
			_walk_popup_accels(popup, out)

## `get_children(true)` -- a `MenuButton` keeps its popup as an INTERNAL
## child, the same trap `command_index.gd`'s own header cites.
func _gather_menu_buttons(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather_menu_buttons(c, out)

func _walk_popup_accels(popup: PopupMenu, out: Array) -> void:
	for i in popup.item_count:
		var accel: int = popup.get_item_accelerator(i)
		if accel != 0:
			out.append([popup.get_item_text(i), accel])
		var sub := popup.get_item_submenu(i)
		if sub != "":
			var node := popup.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				_walk_popup_accels(node as PopupMenu, out)

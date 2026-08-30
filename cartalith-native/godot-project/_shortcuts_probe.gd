extends Node
## `Help ▸ Keyboard shortcuts` probe.
##
##   godot --headless --path . --resolution 1600x900 _shortcuts_probe.tscn
##
## The dialog's whole design claim is that it reads accelerators back off the
## LIVE menus rather than carrying a written table, so the only test that means
## anything is one that boots the real shell and checks the rows it produces
## against the accelerators `menus.gd` actually registered.
##
## A dialog that lists nothing would satisfy every structural check, which is
## the silently-empty-output trap this repository has been bitten by four
## times -- so the row count is asserted non-zero and specific shortcuts are
## named.

var _vp: SubViewport
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _texts(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	for c in n.get_children(true):
		_texts(c, out)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(50)
	print("[BOOT] shell up")

	var dlg = app.get("shortcuts_dialog")
	print("\n=== 1: the dialog exists and is wired ===")
	_ok("shortcuts_dialog constructed", dlg != null, true)
	if dlg == null:
		get_tree().quit(1); return
	_ok("app exposes open_shortcuts()", app.has_method("open_shortcuts"), true)

	# Build its contents the way the menu row does.
	dlg.open()
	await _frames(4)

	var rows: Array = []
	_texts(dlg, rows)
	print("\n=== 2: it produced real rows, not an empty list ===")
	print("  info label count in dialog: ", rows.size())
	_ok("more than the header labels", rows.size() > 8, true)

	var joined := "\n".join(rows)
	_ok("no 'no menu accelerators found' fallback fired",
		joined.findn("No menu accelerators found") < 0, true)

	# Accelerators menus.gd really registers -- if the walk works, these appear.
	print("\n=== 3: the accelerators menus.gd registers are present ===")
	for want in ["Ctrl+N", "Ctrl+O", "Ctrl+S", "Ctrl+W", "Ctrl+Z"]:
		_ok("lists %s" % want, joined.find(want) >= 0, true)

	print("\n=== 4: the non-menu shortcuts are listed and marked as such ===")
	_ok("lists the layer digits", joined.find("1 – 8") >= 0, true)
	_ok("lists space-to-pan", joined.find("Space") >= 0, true)
	_ok("has the 'Not on a menu' group", joined.findn("NOT ON A MENU") >= 0, true)
	_ok("names the file that owns a non-menu shortcut",
		joined.find("layers_popover.gd") >= 0, true)

	print("\n=== 5: reopening re-reads rather than accumulating ===")
	var first := rows.size()
	dlg.open()
	await _frames(4)
	var again: Array = []
	_texts(dlg, again)
	_ok("row count stable across reopen", again.size(), first)

	print("\n_shortcuts_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

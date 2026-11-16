extends Node
## Key-hint probe (2026-11-16, the phone welcome's "touch forms drop the desktop shortcut hint" row). A keyboard
## accelerator ("Ctrl+Z", "V") is drawn on the desktop and never on a tablet or a phone, wherever it is produced:
## the ring's four cardinals (`GlobalTools.ring_cardinals`) and `DccQuickBar.accelerator_text` (the quick bar chip's
## tooltip, the ring dock's caption, the card rows). It also checks that the phone's two welcome buttons carry no
## hint in their text. Every expected value is a literal, never read back from the code under test.
##
## Run (windowed, a wiped APPDATA, the runner holds `C:/cw/.win_lock`):
##   godot --path . --resolution 1600x900 _keyhint_probe.tscn -- --expect-hint yes
##   godot --path . --resolution 540x1170 _keyhint_probe.tscn -- --force-touch --density 2.625 --expect-hint no
## `--expect-hint yes|no`; `--force-touch` and `--density` are read by the shell. Exit 0 pass, 1 fail, 2 bad argument.

var _fails := 0
var _checks := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	return String(args[i + 1]) if i >= 0 and i + 1 < args.size() else dflt

func _ready() -> void:
	var want := _arg("--expect-hint", "")
	if want not in ["yes", "no"]:
		print("ABORT: needs --expect-hint yes|no")
		get_tree().quit(2)
		return
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.8).timeout
	var yes := want == "yes"
	## The form the probe is in is what the runner says it is, not what the code answers for itself.
	_ok(DccTheme.is_touch() == (not yes), "the form is %s" % ("desktop" if yes else "touch"))
	var ring: Dictionary = GlobalTools.ring_cardinals(app, {"domain": "WORLD", "armed_tool": "", "finalized": false})
	_ok(String((ring["N"] as Dictionary)["shortcut"]) == ("V" if yes else ""), "ring N (Inspect) hint is %s" % ("V" if yes else "empty"))
	_ok(String((ring["E"] as Dictionary)["shortcut"]) == ("M" if yes else ""), "ring E (Measure) hint is %s" % ("M" if yes else "empty"))
	_ok(String((ring["W"] as Dictionary)["shortcut"]) == ("R" if yes else ""), "ring W (Region) hint is %s" % ("R" if yes else "empty"))
	_ok(String((ring["S"] as Dictionary)["shortcut"]) == ("Ctrl+Z" if yes else ""), "ring S (Undo) hint is %s" % ("Ctrl+Z" if yes else "empty"))
	## The quick bar's reader, on a real menu row that carries Ctrl+O (File > Open project...).
	var popup: PopupMenu = null
	for b in app.menu_bar_row.get_children():
		if b is MenuButton and (b as MenuButton).text == "File":
			popup = (b as MenuButton).get_popup()
	_ok(popup != null, "the File menu exists")
	if popup != null:
		var got := DccQuickBar.accelerator_text({"_popup": popup, "_id": 11})
		_ok(got == ("Ctrl+O" if yes else ""), "accelerator_text of File > Open project is %s (is '%s')" % [("Ctrl+O" if yes else "empty"), got])
	## The phone welcome: its action buttons carry no hint text (the label is the whole text).
	var picker = app.get("phone_project_picker")
	if not yes and picker != null:
		picker.show()
		await get_tree().process_frame
		var texts: Array = []
		for n in picker.find_children("*", "Button", true, false):
			texts.append((n as Button).text)
		print("  welcome buttons: ", texts)
		_ok("OPEN PROJECT .CTL…" in texts.map(func(t): return String(t).to_upper()), "the phone welcome's Open project button reads exactly 'Open project .ctl…' (the theme upper-cases it)")
		var shot := _arg("--shot", "")
		if shot != "":
			await RenderingServer.frame_post_draw
			get_viewport().get_texture().get_image().save_png(shot)
		for t in texts:
			_ok(not String(t).contains("Ctrl"), "welcome button '%s' carries no Ctrl hint" % String(t))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

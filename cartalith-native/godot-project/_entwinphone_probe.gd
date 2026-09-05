extends Node
## The phone half of `_entwin_probe.gd`: the derivations in the three entity
## windows that only exist when `DccWidgets.phone_window()` says phone —
## §6.6's `_moreTitle()` subtitle voice, and the folded roster bar built against
## §6.6's `nav` row (52 dp, gap 12, a `var(--dim)` sub, a `var(--faint)`
## chevron).
##
## **Run windowed, not `--headless`.** Nothing here samples a pixel, so the
## headless `ImageTexture` trap does not apply — but `is_phone()` is decided
## from the real window size, and a dummy display server is not a handset. The
## probe refuses to assert anything if the resize did not take, rather than
## measuring a desktop and calling it a phone.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _entwinphone_probe.tscn -- --force-touch
##
## **`--force-touch` is mandatory and this probe does not read it** — grepped
## before it was written into this header: the only reader is `dcc_shell.gd`'s
## `_touch = … or "--force-touch" in OS.get_cmdline_user_args()`. It must sit
## after the bare `--`, because `get_cmdline_user_args()` returns only what
## follows one. Without it `_compute_layout_mode()` leaves `_phone` false on
## this machine no matter what size the window is (there is no touch hardware
## and `OS.has_feature("mobile")` is false), and the probe below quits rather
## than measure a desktop and call it a handset.
##
## The size is fixed at 1080 x 2400 in `_ready()` — a handset where
## `phone_scale()` is 2.62, not 1.0. A 393 x 852 reference box is exactly 1.0
## and could not discriminate a scaled row from an unscaled one.

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("EWP %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _labels(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append(c)
		_labels(c, out)

func _label_nodes(root: Node) -> Array:
	var out: Array = []
	_labels(root, out)
	return out

func _texts(root: Node) -> Array:
	var out: Array = []
	for l in _label_nodes(root):
		out.append(String((l as Label).text))
	return out

func _label_with(root: Node, text: String) -> Label:
	for l in _label_nodes(root):
		if String((l as Label).text) == text:
			return l
	return null


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 180.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("EWP WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var want := Vector2i(1080, 2400)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	print("EWP === phone=%s scale=%s screen=%s ===" % [app.is_phone(), app.phone_scale(),
		app.get_viewport_rect().size])
	if not app.is_phone():
		print("EWP  !! NOT PHONE -- the resize did not register; nothing asserted")
		get_tree().quit(1)
		return

	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	print("EWP world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])
	await _frames(8)
	if not app.bridge.has_world:
		print("EWP  !! generate failed")
		get_tree().quit(1)
		return

	# -- Faction roster: §6.6's `nav` row, folded ------------------------------
	var fr = app.faction_roster_window
	fr.open()
	await _frames(8)
	_check("FR0: the roster is running its phone treatment", fr._phone,
		"_phone=%s" % fr._phone)
	_check("FR1: the header subtitle is in §6.6 `_moreTitle()`'s voice",
		_texts(fr).has("roster · identity · territory · military"),
		"labels=%s" % [_texts(fr).slice(0, 8)])

	## Fold the list -- the bar only exists on the detail side.
	fr._set_phone_list_open(false)
	await _frames(4)
	var bar: PanelContainer = fr._phone_list_bar
	_check("FR2: the folded bar is on screen and the list pane is not",
		bar.visible and not fr._phone_list_pane.visible,
		"bar=%s pane=%s" % [bar.visible, fr._phone_list_pane.visible])
	_check("FR3: the bar keeps §6.6's 52 dp `nav` min-height",
		bar.custom_minimum_size.y == 52, "y=%s" % bar.custom_minimum_size.y)

	var sub: Label = fr._phone_bar_sub
	_check("FR4: the `nav` sub the comment has always promised is drawn",
		sub != null and sub.visible and String(sub.text) != "",
		"sub='%s' visible=%s" % [String(sub.text) if sub != null else "<null>",
			sub.visible if sub != null else false])
	_check("FR5: it is `var(--dim)`, the colour §6.6 gives a `nav` sub",
		sub != null and sub.get_theme_color("font_color").is_equal_approx(DccTheme.c("text_dim")),
		"col=%s want=%s" % [sub.get_theme_color("font_color") if sub != null else Color.BLACK,
			DccTheme.c("text_dim")])
	## §6.6 says the `nav` sub is "ellipsised". `clip_text` alone is a hard cut
	## AND collapses the label's minimum width to 1; the ellipsis behaviour is
	## what makes it a trim. `DccShell.phone_fit` sets both on every expanding
	## non-wrapping `Label` it walks, so the assertion is on the *pair* — which
	## is the thing that has to hold — not on this file having set either.
	_check("FR6: the sub trims with an ellipsis rather than a hard cut",
		sub != null and sub.text_overrun_behavior == TextServer.OVERRUN_TRIM_ELLIPSIS,
		"overrun=%d clip=%s" % [sub.text_overrun_behavior if sub != null else -1,
			sub.clip_text if sub != null else false])
	_check("FR6b: and it is not autowrapping, which would defeat the trim",
		sub != null and sub.autowrap_mode == TextServer.AUTOWRAP_OFF,
		"autowrap=%d" % (sub.autowrap_mode if sub != null else -1))

	var chev := _label_with(bar, DccIcons.SYMBOLS["expand"])
	_check("FR7: the chevron is `var(--faint)`, not the disabled ink it was",
		chev != null and chev.get_theme_color("font_color").is_equal_approx(DccTheme.c("text_faint")),
		"col=%s faint=%s ghost=%s" % [
			chev.get_theme_color("font_color") if chev != null else Color.BLACK,
			DccTheme.c("text_faint"), DccTheme.c("text_ghost")])
	_check("FR8: the two are actually different colours, so FR7 can fail",
		not DccTheme.c("text_faint").is_equal_approx(DccTheme.c("text_ghost")),
		"faint=%s ghost=%s" % [DccTheme.c("text_faint"), DccTheme.c("text_ghost")])
	fr.hide()
	await _frames(3)

	# -- Place editor and City viewer: the subtitle voice ----------------------
	app.open_place_editor(0)
	await _frames(8)
	_check("PE1: the header subtitle is a contents list, not the window's name again",
		_texts(app.place_editor_window).has("identity · economy · trade · traits · fabric"),
		"labels=%s" % [_texts(app.place_editor_window).slice(0, 8)])
	app.place_editor_window.hide()
	await _frames(3)

	var cv_index := -1
	var places: Array = app.bridge.settlements()
	for i in mini(places.size(), 12):
		if app.bridge.urban_layouts(PackedInt32Array([i])).size() > 0:
			cv_index = i
			break
	if cv_index >= 0:
		app.open_city_viewer(cv_index)
		await _frames(8)
		_check("CV1: the header subtitle is a contents list",
			_texts(app.city_viewer_window).has("plan · legend · stages"),
			"labels=%s" % [_texts(app.city_viewer_window).slice(0, 8)])
		app.city_viewer_window.hide()

	print("EWP %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

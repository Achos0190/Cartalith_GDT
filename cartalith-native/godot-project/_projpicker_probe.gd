extends Node
## Verifies the phone entry screen (`shell/phone_project_picker.gd`): it
## exists and is visible on phone with no world, it does not exist at all on
## desktop, every action button meets the phone tap floor, and it is gone
## once a world loads.
##
## `DccShell._touch` is read once from `OS.get_cmdline_user_args()`
## per process and cannot be toggled mid-run, so unlike `_ph412_probe.gd`
## (which only varies `--vp` within one process) this one needs **two
## separate invocations**, one with `--force-touch` and one without:
##
##   godot --headless --path . _projpicker_probe.tscn -- --force-touch --vp 1080x2340 --tag phone
##   godot --headless --path . _projpicker_probe.tscn -- --vp 1600x900 --tag desktop
##
## Hosted in a `SubViewport` sized from `--vp`, exactly like `_ph412_probe.gd`
## -- `--resolution WxH` alone clamps to the dev monitor's work area and never
## reaches a real phone aspect. Modelled on `_cmdindex_probe.gd`'s boot/assert
## shape otherwise.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them.

var _fail := 0
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

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _find_buttons(node: Node, out: Array) -> void:
	if node is Button:
		out.append(node)
	for c in node.get_children():
		_find_buttons(c, out)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return

	var tag := _arg("--tag", "run")
	var vp_arg := _arg("--vp", "1600x900")
	var parts := vp_arg.split("x")
	var w := int(parts[0]) if parts.size() == 2 else 1600
	var h := int(parts[1]) if parts.size() == 2 else 900

	_vp = SubViewport.new()
	_vp.size = Vector2i(w, h)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(60)

	var is_phone: bool = app.is_phone()
	print("[BOOT] tag=%s  %dx%d  is_phone()=%s" % [tag, w, h, is_phone])

	var picker: Node = app.get("phone_project_picker")
	var by_name: Node = app.find_child("PhoneProjectPicker", true, false)

	print("\n=== 1: existence follows is_phone() exactly ===")
	if is_phone:
		_ok("app.phone_project_picker was constructed", picker != null, true)
		_ok("find_child(\"PhoneProjectPicker\") also finds it", by_name != null, true)
	else:
		_ok("app.phone_project_picker is null (never constructed)", picker == null, true)
		_ok("find_child(\"PhoneProjectPicker\") finds nothing", by_name == null, true)

	if is_phone and picker != null:
		print("\n=== 2: visible on phone with no world ===")
		_ok("bridge reports no world yet", app.bridge.has_world, false)
		_ok("picker is visible", (picker as Window).visible, true)

		print("\n=== 3: every action button meets DccTheme.PHONE_TAP_MIN ===")
		var buttons: Array = []
		_find_buttons(picker, buttons)
		print("  info PHONE_TAP_MIN = %d" % DccTheme.PHONE_TAP_MIN)
		_ok("found the New world / Open .zip action buttons", buttons.size() >= 2, true)
		for b in buttons:
			var btn := b as Button
			_ok("'%s' height (%.1f) >= tap floor" % [btn.text, btn.size.y],
				btn.size.y >= float(DccTheme.PHONE_TAP_MIN) - 0.5, true)

		print("\n=== 4: gone once a world loads, and stays gone ===")
		## A real generation is slow and unnecessary here -- the picker only
		## listens for the same signal `app.gd::_wire_status()` already treats
		## as "a world now exists" (`phone_project_picker.gd::setup()`), so
		## emitting it directly exercises the real dismissal path without
		## paying for a multi-second pipeline run.
		app.bridge.world_loaded.emit()
		await _frames(3)
		_ok("picker hid itself", (picker as Window).visible, false)
		## "must not come back on its own" -- nothing should re-show it just
		## because more frames pass.
		await _frames(10)
		_ok("...and stayed hidden", (picker as Window).visible, false)
	elif not is_phone:
		print("\n=== 2-4: skipped (this boot is the desktop/non-phone control) ===")

	print("\n_projpicker_probe (%s): " % tag, "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

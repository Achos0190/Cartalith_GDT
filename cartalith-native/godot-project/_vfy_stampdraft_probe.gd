extends Node
## VERIFIER (batch 20): does an uncommitted sculpt draft keep its Commit /
## Discard controls when another WORLD tool arms?
##
## Lane B's `TOOL_STAMPS` doc gives the draft clause this reason: *"Under the
## tool clause alone, Escape would take the Commit and Discard controls away
## from a draft that is still uncommitted."* `_tool_section()` returns ONE
## section, and its `match` on `app.armed_tool` answers `paint`, `territory`,
## `label` and `icon` before the draft clause is ever reached. This measures
## what the dock actually shows in each of those states over a live draft.
##
## At `686cd2a` the stack was `CTX_SCULPT`, a context: `_dispatch()` drew it
## and `_append_tool()` appended the tool's section under it, so both were on
## screen at once. This probe does not assert the old behaviour -- it prints
## the new one per tool and fails only if a live draft has no Commit control
## anywhere on screen.

var app: Node
var fail := 0

func _chk(name: String, ok: bool, detail: String = "") -> void:
	print("SD %s  %s%s" % ["ok  " if ok else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not ok:
		fail += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label or c is Button or c is CheckBox:
			var t := String(c.text)
			if not t.is_empty():
				out.append(t)
		_collect(c, out)

func _texts() -> Array:
	var out: Array = []
	_collect(app.right_dock_body, out)
	return out

func _headers() -> Array:
	var out: Array = []
	for t in _texts():
		var s := String(t)
		if s.begins_with("§ "):
			out.append(s.substr(2))
	return out

func _has(frag: String) -> bool:
	for t in _texts():
		if String(t).find(frag) >= 0:
			return true
	return false

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 480.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("SD WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge = app.bridge
	bridge.generate({
		"seed": 4242, "width_km": 800.0, "grid_w": 128, "grid_h": 96,
		"archetype": "", "villages": false, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.5).timeout

	app.select_domain("world")
	await _frames(4)
	var made := false
	if bridge.sculpt_begin_stroke():
		for i in 6:
			bridge.sculpt_add_point(40.0 + i * 4.0, 30.0 + i * 3.0)
		made = bridge.sculpt_end_stroke() >= 0
	_chk("S0 premise: a real uncommitted draft exists",
		made and bridge.sculpt_stamp_count() > 0,
		"stamps=%d" % bridge.sculpt_stamp_count())
	if not made or bridge.sculpt_stamp_count() == 0:
		print("SD RESULT: PREMISE FAILED fail=%d" % fail)
		get_tree().quit(1)
		return

	app.arm_tool("inspect")
	await _frames(4)
	print("SD   state: domain=%s armed=%s stamps=%d section=%s"
		% [app.active_domain(), app.armed_tool, bridge.sculpt_stamp_count(),
			app.right_dock_ctrl._tool_section()])
	_chk("S1 with Inspect armed the live draft keeps its Stamp stack",
		_headers().has("STAMP STACK"), "headers=%s" % [_headers()])

	for t in ["paint", "territory", "label", "icon", "sculpt"]:
		app.arm_tool(t)
		await _frames(4)
		var keeps := _headers().has("STAMP STACK")
		print("SD   armed=%-10s domain=%s section=%s headers=%s stack=%s"
			% [t, app.active_domain(), app.right_dock_ctrl._tool_section(),
				_headers(), keeps])
		_chk("S2/%s a live draft still shows its Stamp stack while %s is armed"
			% [t, t], keeps, "headers=%s stamps=%d"
			% [_headers(), bridge.sculpt_stamp_count()])

	bridge.sculpt_discard()
	print("SD RESULT fail=%d" % fail)
	get_tree().quit(1 if fail > 0 else 0)

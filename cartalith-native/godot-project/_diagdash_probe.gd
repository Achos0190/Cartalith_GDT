extends Node
## Measures the settlement-diagnostics card's dash-with-a-reason condition
## over a real world, by calling `CivilizationWorkspace._diag_card` itself --
## not by re-implementing its formatting here, which is exactly how a
## verification pass talks itself into agreeing with the bug.
##
## Run: godot4 --headless --path . _diagdash_probe.tscn

var _app: Node

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge
	bridge.generate({
		"seed": 77021, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout

	var places: Array = bridge.settlements()
	var idx := PackedInt32Array()
	for i in range(places.size()):
		idx.append(i)
	var cards: Array = bridge.settlement_diagnostics(idx)
	print("PROBE settlements=%d cards=%d" % [places.size(), cards.size()])

	var ws: Control = null
	for w in _app._workspaces:
		if w.name == "CivilizationWorkspace":
			ws = w
	if ws == null:
		print("PROBE FAIL: no CivilizationWorkspace")
		get_tree().quit(2)
		return

	var host := VBoxContainer.new()
	add_child(host)
	var lines: Array[String] = []
	for c in cards:
		var d: Dictionary = c
		var i := int(d.get("index", -1))
		var s: Dictionary = places[i] if i >= 0 and i < places.size() else {}
		var before := host.get_child_count()
		ws._diag_card(host, s, d, i)
		## One card == one VBox; harvest every Label/Button text under it.
		var card: Control = host.get_child(before)
		var txt: Array[String] = []
		for n in card.get_children():
			if n is Label or n is Button:
				txt.append(String(n.text))
		lines.append(" | ".join(txt))

	## The refuted condition, counted straight off the rendered text.
	var bare_zero := 0
	var dashed_reach := 0
	var blank_field := 0
	for l in lines:
		if l.find("Strahler order 0") >= 0:
			bare_zero += 1
		if l.find("order — beyond this site's water reach") >= 0:
			dashed_reach += 1
		## Any field rendered as an empty run between separators.
		if l.find("|  |") >= 0 or l.begins_with(" |") or l.ends_with("| "):
			blank_field += 1
	print("PROBE rendered=%d  bare 'Strahler order 0'=%d  dashed-with-reason=%d  blank fields=%d"
		% [lines.size(), bare_zero, dashed_reach, blank_field])
	var shown := 0
	for l in lines:
		if l.find("order — beyond") >= 0 and shown < 3:
			print("PROBE sample: " + l)
			shown += 1
	## -- Preferences ▸ CPU worker threads (lane A1's UI half) ----------------
	## `DccMenus` is a RefCounted held on the app, not a node in the tree.
	var menus = _app.menus
	if menus == null:
		print("PROBE CPU: no menus controller found")
	else:
		var pop = menus._cpu_threads_popup
		if pop == null:
			print("PROBE CPU: submenu was never built")
		else:
			var boot_configured := DccSettings.cpu_thread_count()
			print("PROBE CPU boot: stored=%d engine_configured=%d pool_active=%d" % [
				boot_configured, bridge.cpu_thread_count(),
				bridge.cpu_thread_count_active()])
			menus._refresh_cpu_threads_menu()
			var rungs: Array[String] = []
			for i in pop.item_count:
				rungs.append("%s%s" % [pop.get_item_text(i),
					"*" if pop.is_item_checked(i) else ""])
			print("PROBE CPU rungs: " + " / ".join(rungs))
			print("PROBE CPU cores=%d configured=%d active=%d" % [
				bridge.cpu_logical_core_count(), bridge.cpu_thread_count(),
				bridge.cpu_thread_count_active()])
			var applied: bool = bridge.cpu_set_thread_count(2)
			print("PROBE CPU set(2) applied=%s active_after=%d stored_after=%d" % [
				applied, bridge.cpu_thread_count_active(), DccSettings.cpu_thread_count()])
			menus._refresh_cpu_threads_menu()
			print("PROBE CPU readout after: " + pop.get_item_text(pop.item_count - 1))
			## Two-run check of `_restore_cpu_pref()`: the first run leaves a
			## real preference behind, the second boots with it and prints
			## what the pool actually came up at, then clears it so a probe
			## run does not silently reconfigure the developer's install.
			## `boot_configured` is read before any set above.
			if boot_configured == 0:
				DccSettings.set_cpu_thread_count(4)
				print("PROBE CPU armed: stored 4 -- run again to see the startup restore")
			else:
				DccSettings.set_cpu_thread_count(0)
				print("PROBE CPU disarmed: stored back to 0 (automatic)")

	get_tree().quit(0 if bare_zero == 0 and blank_field == 0 else 1)

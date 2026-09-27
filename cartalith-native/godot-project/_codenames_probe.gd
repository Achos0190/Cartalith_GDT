extends Node
## Windowed census of developer code names shown to users
## (`OUTSTANDING_WORK.md` §2.10, "Developer code names are shown to users";
## Owner Ruling AQ, `LARGE_ITEM_RULINGS.md`, 2026-09-24: rewrite ALL of them).
##
## Two passes:
##  1. VISIBLE label/button text on each domain's default panels (the
##     original 2026-09-24 census -- unchanged).
##  2. TOOLTIP text (`tooltip_text` on every live Control, plus every
##     `PopupMenu` item's tooltip via `get_item_tooltip`) across WORLD, CIVIL
##     and CARTO's default panels, the reachable menu bar, and the Shortcuts
##     dialog -- the 2026-09-27 tooltip sweep this probe was extended for.
## PASS when neither pass finds a hit, except in a file this sweep explicitly
## skipped (listed below, and printed as SKIPPED rather than counted).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _codenames_probe.tscn

## Files this 2026-09-27 tooltip sweep could not touch -- owned by the
## concurrent per-style-rivers lane (`engine_bridge.gd`, `data_manager_window.gd`,
## `workspaces/cartography_workspace.gd`, `workspaces/render_workspace.gd`,
## `viewport_host.gd`, `map_overlay.gd`). Any hit whose source is one of these
## is reported as SKIPPED, not FAIL, and does not fail the probe.
const SKIPPED_SOURCES := [
	"engine_bridge.gd", "data_manager_window.gd", "cartography_workspace.gd",
	"render_workspace.gd", "viewport_host.gd", "map_overlay.gd",
]

var _re := RegEx.new()
var _hits: Array = []
var _tip_hits: Array = []

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _walk(n: Node, where: String) -> void:
	for c in n.get_children():
		if c is Label and (c as Label).is_visible_in_tree():
			var tx := (c as Label).text
			if _re.search(tx) != null:
				_hits.append("%s | %s" % [where, tx.replace("\n", " ").substr(0, 140)])
		if c is Button and (c as Button).is_visible_in_tree():
			var bt := (c as Button).text
			if _re.search(bt) != null:
				_hits.append("%s | [button] %s" % [where, bt.substr(0, 140)])
		_walk(c, where)

## `Control.tooltip_text` (or `PopupMenu`'s per-item tooltip, which is not a
## property on the item but a separate call) is read regardless of visibility
## -- a hidden tab's tooltip is still a shipped string, and `is_visible_in_tree()`
## would silently exempt every panel not the current tab.
func _walk_tooltips(n: Node, where: String) -> void:
	for c in n.get_children():
		if c is Control:
			var tt := (c as Control).tooltip_text
			if tt != "" and _re.search(tt) != null:
				_tip_hits.append("%s | [%s] %s" % [where, c.get_class(), tt.replace("\n", " ").substr(0, 160)])
		if c is PopupMenu:
			var p := c as PopupMenu
			for i in p.item_count:
				var it := p.get_item_tooltip(i)
				if it != "" and _re.search(it) != null:
					_tip_hits.append("%s | [PopupMenu item %d %s] %s" % [
						where, i, p.get_item_text(i), it.replace("\n", " ").substr(0, 160)])
		_walk_tooltips(c, where)

func _record_tip(where: String, node: Node) -> void:
	_walk_tooltips(node, where)

func _ready() -> void:
	## `.gd`/`.rs` file names, `::` paths, `cartalith_`/`cartalith-` crate
	## names, and snake_case identifiers followed by `(` -- the shapes a
	## developer name takes in prose.
	_re.compile(r"\.gd\b|\.rs\b|::|\bcartalith[_-][a-z0-9_-]*|\b[a-z][a-z0-9]*_[a-z0-9_]+\(")
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	bridge.generate({"seed": 9137, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)
	var shell: Node = app  ## app.gd extends DccShell
	for dom in ["world", "civilization", "cartography"]:
		if shell != null:
			shell.select_domain(dom)
		await _frames(10)
		_walk(app, dom)
		_record_tip(dom, app)

	## The menu bar: every PopupMenu the app owns is a child of the app tree
	## already, so `_walk_tooltips` above reached it -- recorded again here
	## under its own `where` for a readable report.
	_record_tip("menubar", app)

	## The Shortcuts dialog: not open by default, so `_walk_tooltips` from the
	## app root would miss any tooltip that only exists once it is built.
	if app.has_method("open_shortcuts") or ("shortcuts_dialog" in app and app.shortcuts_dialog != null):
		var sd = app.shortcuts_dialog
		if sd != null:
			sd.open()
			await _frames(4)
			_record_tip("shortcuts_dialog", sd)
			sd.hide()

	var seen := {}
	for h in _hits:
		if not seen.has(h):
			seen[h] = true
			print("CODENAME ", h)
	print("CODENAME total distinct %d" % seen.size())
	## 2026-09-24: 4 before the fix (the world progress note, the CIVIL and
	## CARTO default tool-option lines, the right dock's ramp note), 0 after.
	print("CODENAME %s" % ("PASS" if seen.is_empty() else "FAIL"))

	## A hit's tooltip TEXT does not reliably name its own source .gd file (the
	## sweep rewrote most tooltips to plain words that no longer cite the file
	## they live in), so a substring match of `SKIPPED_SOURCES` against the
	## text cannot attribute a hit to its source -- tried, and it scored 0/4
	## correct on this file's actual remaining hits, which is why this probe
	## does not attempt to auto-classify SKIPPED vs FAIL. Every remaining hit
	## is printed and counted as a plain hit; whoever reads the run
	## cross-references it against the current SKIPPED_SOURCES list (and the
	## known "visible DccWidgets.note text, not a tooltip -- out of the
	## tooltip sweep's own brief" cases) by hand, the way the 2026-09-27 sweep
	## did: `grep -rn "<distinctive phrase from the hit>" cartalith-native/godot-project/shell`.
	var tip_seen := {}
	for h in _tip_hits:
		if not tip_seen.has(h):
			tip_seen[h] = true
			print("TOOLTIP-CODENAME ", h)
	print("TOOLTIP-CODENAME total distinct %d" % tip_seen.size())
	print("TOOLTIP-CODENAME %s" % ("PASS" if tip_seen.is_empty() else "FAIL (cross-reference each against SKIPPED_SOURCES / known out-of-brief visible text by hand)"))

	get_tree().quit(0 if (seen.is_empty() and tip_seen.is_empty()) else 1)

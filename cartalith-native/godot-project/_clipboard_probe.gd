extends Node
## `Edit ▸ Cut / Copy / Paste / Select all` (`DCC_SHELL_SPEC.md` §2.2), steps
## two and three of the owner's `LARGE_ITEM_RULINGS.md` ruling — driven through
## the real menu rows rather than by calling `DccMenus`' methods, so a row still
## wired as a `_todo`, or wired to the wrong id, fails here.
##
##   Godot_v4.7.1 --headless --path . _clipboard_probe.tscn
##
## What this cannot cover, said rather than skipped silently: **the icon leg.**
## `icon_arm()` refuses without an asset pack (`lib.rs`'s `has_asset_pack()`
## guard) and none ships — `_deselect_probe.gd` records the same limitation as
## CA-12. Section 7 asserts the icon half that does not need a pack (the
## `slot` → `variant` reverse map every icon paste goes through) against the
## same literals `icon_bridge.rs`'s own `resolve_variant` tests pin, and
## section 8 states plainly which leg went unexercised.

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _find_menu_item(app: Node, id: int) -> PopupMenu:
	var stack: Array = [app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
			if c is PopupMenu and (c as PopupMenu).get_item_index(id) >= 0:
				return c as PopupMenu
	return null

const ID_CUT := 84
const ID_COPY := 85
const ID_PASTE := 86
const ID_SELECT_ALL := 87

func _row(p: PopupMenu, id: int) -> Dictionary:
	var i := p.get_item_index(id)
	if i < 0:
		return {}
	return {"text": p.get_item_text(i), "disabled": p.is_item_disabled(i),
		"tip": p.get_item_tooltip(i)}

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await _frames(40)
	var bridge = app.get("bridge")
	var menus = app.get("menus")
	print("[BOOT] shell up")

	print("")
	print("=== 0: all four rows are LIVE commands, not disabled _todo rows ===")
	var pop := _find_menu_item(app, ID_COPY)
	_ok("a menu carries the Copy row", pop != null, true)
	if pop == null:
		get_tree().quit(1); return
	for pair in [[ID_CUT, "Cut"], [ID_COPY, "Copy"], [ID_PASTE, "Paste"], [ID_SELECT_ALL, "Select all"]]:
		var i := pop.get_item_index(int(pair[0]))
		_ok("%s is on the Edit menu" % pair[1], i >= 0, true)
		_ok("%s carries an accelerator" % pair[1], pop.get_item_accelerator(i) != 0, true)
	## A `_todo` row is `add_item` with no id at all, so `get_item_index(ID)`
	## returning >= 0 already proves these are not `_todo`s. What it does not
	## prove is that they are not `_readout`/`_signpost` rows, which carry
	## metadata; assert the metadata is absent so `command_index.gd` classes
	## them as real commands.
	for id in [ID_CUT, ID_COPY, ID_PASTE, ID_SELECT_ALL]:
		_ok("row %d carries no readout/signpost metadata" % id,
			pop.get_item_metadata(pop.get_item_index(id)), null)

	print("")
	print("=== 1: before any world, every row is dark with a reason ===")
	pop.about_to_popup.emit()
	await _frames(2)
	for id in [ID_CUT, ID_COPY, ID_PASTE, ID_SELECT_ALL]:
		var r := _row(pop, id)
		_ok("row %d disabled with no world" % id, r["disabled"], true)
		_ok("row %d says why" % id, String(r["tip"]).length() > 10, true)
	_ok("Paste's label is bare when the clipboard is empty", _row(pop, ID_PASTE)["text"], "Paste")

	print("")
	print("=== 2: a world, and CARTO active ===")
	bridge.generate({"seed": 4242, "width_km": 600.0, "grid_w": 256, "grid_h": 192,
		"sea_level": 0.5, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	_ok("a world landed", bridge.has_world, true)
	app.select_domain("cartography")
	await _frames(6)
	_ok("CARTO is the active domain", app.active_domain(), "cartography")

	pop.about_to_popup.emit()
	await _frames(2)
	_ok("Copy is dark with nothing selected", _row(pop, ID_COPY)["disabled"], true)
	_ok("...and says so rather than claiming no clipboard exists",
		String(_row(pop, ID_COPY)["tip"]).begins_with("Nothing selected"), true)
	_ok("Select all is LIVE in CARTO with a world", _row(pop, ID_SELECT_ALL)["disabled"], false)
	_ok("Paste is still dark: nothing has been copied", _row(pop, ID_PASTE)["disabled"], true)

	print("")
	print("=== 3: three styled labels, two of them selected ===")
	var made: Array[int] = []
	for i in 3:
		var idx: int = bridge.label_create(20.0 + 10.0 * i, 30.0, "Clip %d" % i)
		_ok("label %d created" % i, idx >= 0, true)
		made.append(idx)
	## Distinct, non-default style on the two that will travel, so a paste that
	## silently dropped `label_set`'s fields cannot pass by accident.
	var style := {"size": 21.0, "angle": 17.0, "arc": 0.25, "size_mode": "fixed"}
	for i in [0, 1]:
		var res: Dictionary = bridge.label_set(made[i], style)
		_ok("style applied to label %d with nothing rejected" % i,
			(res.get("rejected", PackedStringArray()) as PackedStringArray).size(), 0)
	var before_style: Array[Dictionary] = [bridge.label_get(made[0]), bridge.label_get(made[1])]
	_ok("the fixture really is non-default (size moved)",
		float(before_style[0]["size"]) != float(bridge.label_get(made[2])["size"]), true)

	bridge.label_select_set(PackedInt64Array([made[0], made[1]]))
	_ok("two labels selected", bridge.label_get_selection().size(), 2)

	print("")
	print("=== 4: Copy — through the menu row ===")
	pop.about_to_popup.emit()
	await _frames(2)
	_ok("Copy is live now", _row(pop, ID_COPY)["disabled"], false)
	pop.id_pressed.emit(ID_COPY)
	await _frames(4)
	var clip: Dictionary = menus._clipboard
	_ok("the clipboard holds a labels key", clip.has("labels"), true)
	_ok("...with both labels", (clip["labels"] as Array).size(), 2)
	## The omit-the-key rule (`MISTAKES.md`): a kind with nothing in it is
	## ABSENT, never an empty Array. No icon was copied, so there must be no
	## `icons` key at all — an empty Array here would read as "icons were
	## considered and there were none", which is a different claim.
	_ok("...and NO icons key, because no icon was copied", clip.has("icons"), false)
	_ok("Copy did not delete anything", bridge.label_list().size(), 3)

	pop.about_to_popup.emit()
	await _frames(2)
	_ok("Paste is live and names what it holds", _row(pop, ID_PASTE)["text"], "Paste 2 labels")

	print("")
	print("=== 5: Paste — count, offset, style round-trip, selection ===")
	pop.id_pressed.emit(ID_PASTE)
	await _frames(6)
	_ok("two labels were added", bridge.label_list().size(), 5)
	var pasted: PackedInt64Array = bridge.label_get_selection()
	_ok("the paste owns the selection", pasted.size(), 2)
	if pasted.size() < 2:
		## Bail rather than index past the end. A mutation that stops Paste
		## selecting what it added used to take the probe out on an
		## out-of-bounds read after the FAIL line -- detected, but reported as a
		## crash instead of as a failure count, which a harness reads as
		## "no result" rather than as "killed".
		print("_clipboard_probe: ", str(_fail) + " FAILURE(S)")
		get_tree().quit(1); return
	var p0: Dictionary = bridge.label_get(int(pasted[0]))
	var p1: Dictionary = bridge.label_get(int(pasted[1]))
	## The offset is asserted as the LITERAL 4.0, not as
	## `DccMenus.PASTE_OFFSET_CELLS` — a constant compared against itself holds
	## for every value of it (`MISTAKES.md`, made twice in this tree).
	_ok("x moved by exactly 4 cells", p0["x"], float(before_style[0]["x"]) + 4.0)
	_ok("y moved by exactly 4 cells", p0["y"], float(before_style[0]["y"]) + 4.0)
	_ok("text survived", p0["text"], before_style[0]["text"])
	for k in ["size", "angle", "arc", "size_mode", "font", "color"]:
		_ok("field %s round-tripped" % k, p0[k], before_style[0][k])
	_ok("the second label came too", p1["text"], before_style[1]["text"])
	_ok("...at its own offset", p1["x"], float(before_style[1]["x"]) + 4.0)

	print("")
	print("=== 6: Cut — the buffer is real and the entities go ===")
	pop.about_to_popup.emit()
	await _frames(2)
	_ok("Cut is live over the pasted pair", _row(pop, ID_CUT)["disabled"], false)
	pop.id_pressed.emit(ID_CUT)
	await _frames(6)
	_ok("both cut labels are gone", bridge.label_list().size(), 3)
	_ok("the clipboard holds them", (menus._clipboard["labels"] as Array).size(), 2)
	## Descending deletion is the whole reason `_delete_descending` exists: an
	## ascending pass over {3,4} removes 3, slides 4 down to 3, then removes
	## what is now a DIFFERENT label. Three survivors and the right three names
	## is what proves the order.
	var names: Array[String] = []
	for row in bridge.label_list():
		names.append(String(row["text"]))
	names.sort()
	_ok("the three originals survived, unshifted", ", ".join(names), "Clip 0, Clip 1, Clip 2")

	print("")
	print("=== 7: paste again from the cut buffer, and the grid clamp ===")
	pop.id_pressed.emit(ID_PASTE)
	await _frames(6)
	_ok("cut's buffer pastes", bridge.label_list().size(), 5)

	var g: Vector2i = bridge.grid_size()
	var edge: int = bridge.label_create(float(g.x) - 1.0, float(g.y) - 1.0, "Edge")
	_ok("an edge label was created", edge >= 0, true)
	bridge.label_select_set(PackedInt64Array([edge]))
	pop.id_pressed.emit(ID_COPY)
	await _frames(2)
	pop.id_pressed.emit(ID_PASTE)
	await _frames(6)
	var clamped: Dictionary = bridge.label_get(int(bridge.label_get_selection()[0]))
	_ok("x clamped inside the grid, not pushed past it", clamped["x"], float(g.x) - 1.0)
	_ok("y clamped inside the grid, not pushed past it", clamped["y"], float(g.y) - 1.0)

	print("")
	print("=== 8: Select all, scoped to the active domain ===")
	var total: int = bridge.label_list().size()
	bridge.label_select_set(PackedInt64Array())
	_ok("selection cleared first", bridge.label_get_selection().size(), 0)
	pop.id_pressed.emit(ID_SELECT_ALL)
	await _frames(4)
	_ok("every label is selected", bridge.label_get_selection().size(), total)

	app.select_domain("civilization")
	await _frames(6)
	pop.about_to_popup.emit()
	await _frames(2)
	_ok("Select all is dark in CIVIL", _row(pop, ID_SELECT_ALL)["disabled"], true)
	_ok("...naming the real gap, a settlement selection set",
		String(_row(pop, ID_SELECT_ALL)["tip"]).contains("selection set"), true)
	_ok("Copy is dark in CIVIL too", _row(pop, ID_COPY)["disabled"], true)
	_ok("...and does not claim the clipboard is missing",
		String(_row(pop, ID_COPY)["tip"]).contains("no clipboard"), false)

	app.select_domain("world")
	await _frames(6)
	pop.about_to_popup.emit()
	await _frames(2)
	var stamps: int = bridge.sculpt_stamp_count()
	print("  info sculpt stamps in the open draft: ", stamps)
	_ok("Select all in WORLD tracks the draft's stamp count",
		_row(pop, ID_SELECT_ALL)["disabled"], stamps == 0)
	_ok("Copy in WORLD says stamps cannot be serialised, not that nothing exists",
		String(_row(pop, ID_COPY)["tip"]).contains("point COUNT"), true)
	app.select_domain("cartography")
	await _frames(4)

	print("")
	print("=== 9: the slot -> variant reverse map every icon paste goes through ===")
	## Asserted against the same literals `icon_bridge.rs`'s own
	## `resolve_variant_indexes_the_familys_own_frozen_slots` pins, which is an
	## independent statement of the mapping rather than a restatement of the
	## table this reads.
	_ok("feature/mountain", menus._icon_variant_of("feature", "mountain"), 0)
	_ok("feature/hill", menus._icon_variant_of("feature", "hill"), 1)
	_ok("settlement/hamlet", menus._icon_variant_of("settlement", "hamlet"), 0)
	_ok("poi/ruin", menus._icon_variant_of("poi", "ruin"), 0)
	_ok("an unknown slot is -1, not 0", menus._icon_variant_of("feature", "nope"), -1)
	_ok("custom is unaddressable, matching resolve_variant", menus._icon_variant_of("custom", "x"), -1)

	print("")
	print("=== 10: the icon leg, exercised only if a pack is loaded ===")
	if bridge.icon_arm("feature", 0, 1.3, 0.0, 0.0):
		var placed: int = bridge.icon_place(50.0, 50.0)
		_ok("an icon was placed", placed >= 0, true)
		bridge.icon_select_set(PackedInt64Array([placed]))
		var icons_before: int = bridge.icon_list().size()
		pop.id_pressed.emit(ID_COPY)
		await _frames(2)
		_ok("the clipboard has an icons key", (menus._clipboard as Dictionary).has("icons"), true)
		pop.id_pressed.emit(ID_PASTE)
		await _frames(6)
		_ok("an icon was pasted", bridge.icon_list().size(), icons_before + 1)
		var src: Dictionary = bridge.icon_get(placed)
		var dst: Dictionary = bridge.icon_get(int(bridge.icon_get_selection()[0]))
		_ok("family survived", dst["family"], src["family"])
		_ok("slot survived", dst["slot"], src["slot"])
		_ok("scale survived", dst["scale"], src["scale"])
		_ok("x moved by exactly 4 cells", dst["x"], float(src["x"]) + 4.0)
	else:
		print("  info icon_arm refused: no asset pack ships (CA-12).")
		print("  info UNEXERCISED: icon copy/cut/paste and _restore_armed.")
		print("  info Exercised without a pack: the slot->variant map above, and")
		print("  info the Paste tooltip's own has_asset_pack() disclosure below.")
		_ok("has_asset_pack() is indeed false, so the skip is honest",
			bridge.has_asset_pack(), false)

	print("")
	print("_clipboard_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

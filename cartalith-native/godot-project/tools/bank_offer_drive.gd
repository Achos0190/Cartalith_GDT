extends RefCounted
## Ruling HN (owner, 2026-11-23): opening a world saved before `civ.bank_sites` offers, once, to turn it on.
## Driven through the REAL shell on one form; run once per form (desktop, tablet, phone):
##   python -I docs/evidence/town-on-bank-hn-2026-11-23/run_drive.py FORM OUT_DIR TIMEOUT_S res://tools/bank_offer_drive.gd HN_SAVE=<file>
## Inputs are environment variables, since the shell reads its own user arguments:
##   HN_OUT   the directory the frames and the scratch saves go to (required);
##   HN_SAVE  a REAL save written before the switch existed, with its substrate (required; the repository's
##            `crates/cartalith-godot/tests/fixtures/project_384cap_512x328_2026-09-28.zip`). It is copied to HN_OUT
##            and the copy is opened, so the fixture is never written.
## Steps and checks (each prints `ok` or `FAIL`; the launcher exits 1 on any FAIL):
##   1. THE OFFER: opening the save shows one card, with the ruling's sentence as a literal, inside the window, that
##      is no window of its own (nothing exclusive is open, and the map still takes a wheel step under it); on a touch
##      form both buttons are at least 44 dp tall. The switch is still off.
##   2. NO: a real click on "Keep as it is" removes the card, leaves the switch off, writes no parameter (the world is
##      not marked changed and a New World would still be made with the switch on), and records "no" for this world.
##   3. NEVER AGAIN: the same save opened again shows no card. A COPY of it under another name shows none either (the
##      record is the world's, not the file's).
##   4. YES: with the record cleared the card returns; a real click on "Update" turns the switch on, marks the world
##      changed, and the towns move: at least one town is laid round a point that is not its cell, while a town frozen
##      before the click is served unchanged, building for building, and its check names the change.
##   5. A NEW SAVE IS SILENT (the control): the world saved now and reopened, with the record cleared, shows no card
##      and loads the switch on; and a save with an explicit `false` shows none either.
## The frames `<form>_offer.png` (the card over the map) and `<form>_after_yes.png` are for looking at.

const SENTENCE := "This world was made before towns sat on the bank — update?"
const TOUCH_MIN_DP := 44.0

var _fails := 0
var app: Node
var bridge
var _form := "desktop"
var _out := ""


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _wait_for(ctx, t: String, secs: float = 20.0) -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		for e in DriveCtx.dump_ui(ctx, t):
			return e
		await ctx.wait(0.25)
	return {}


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


## Past the boot card: the phone's worlds list, or the desktop and tablet welcome (`drive_proof.gd`).
func _past_welcome(ctx) -> void:
	if app.get_viewport().get_visible_rect().size.x < 700.0:
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		if not nw.is_empty():
			await _click_entry(ctx, nw)
			var cancel := await _wait_for(ctx, "CANCEL")
			if not cancel.is_empty():
				await _click_entry(ctx, cancel)
		await ctx.wait(1.0)
		return
	var cont := await _wait_for(ctx, "Continue without a world")
	if not cont.is_empty():
		await _click_entry(ctx, cont)
		await ctx.wait(0.6)


func _card() -> Control:
	return app.get_node_or_null("BankSitesOffer") as Control


## Opens `path` the way both pickers do and waits for the deferred offer and the first frames.
func _open(ctx, path: String) -> bool:
	var ok: bool = await app.open_project_busy(path)
	await ctx.wait(1.5)
	if app.get("open_project_dialog") != null and app.open_project_dialog:
		app.open_project_dialog.hide()
	return ok


func _shot(ctx, name: String) -> void:
	await ctx.wait(0.4)
	var img: Image = app.get_viewport().get_texture().get_image()
	img.save_png(_out.path_join("%s_%s.png" % [_form, name]))


func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not ("bridge" in app):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	_out = OS.get_environment("HN_OUT")
	var src := OS.get_environment("HN_SAVE")
	if _out == "" or src == "":
		ctx.error("HN_OUT and HN_SAVE are required")
		return
	DirAccess.make_dir_recursive_absolute(_out)
	var size: Vector2 = app.get_viewport().get_visible_rect().size
	_form = "phone" if size.x < 700.0 else ("tablet" if bool(DccTheme.is_tablet()) else "desktop")
	ctx.log("form %s, window %s" % [_form, str(size)])
	await _past_welcome(ctx)
	var old := _out.path_join("old_%s.zip" % _form)
	var copy := _out.path_join("old_renamed_%s.zip" % _form)
	_check(ctx, "the old save is copied for the run", DirAccess.copy_absolute(src, old) == OK and DirAccess.copy_absolute(src, copy) == OK, src)

	## 1. the offer
	_check(ctx, "the old save opens", await _open(ctx, old))
	var key := String(bridge.world_gen.bank_sites_offer())
	_check(ctx, "the engine offers this world the update (a save without the key, a world that lays towns beside a drawn river)", key.length() == 16, "key '%s'" % key)
	_check(ctx, "the switch loaded off", bridge.param_get("civ.bank_sites") == false)
	var card := _card()
	_check(ctx, "one offer card is shown", card != null and card.visible)
	if card == null:
		ctx.log("RESULT FAIL (%d failed)" % _fails)
		return
	var text: Label = card.find_child("BankSitesOfferText", true, false)
	_check(ctx, "it asks the ruling's question, word for word", text != null and text.text == SENTENCE, text.text if text != null else "no label")
	var r := card.get_global_rect()
	_check(ctx, "the card is inside the window, 16 dp or more from its sides", r.position.x >= 15.5 and r.end.x <= size.x - 15.5 and r.position.y >= 0.0 and r.end.y <= size.y, "%s in %s" % [str(r), str(size)])
	_check(ctx, "it is a card on the shell, not a window", card is PanelContainer and not (card.get_parent() is Window and card.get_parent() != app.get_window()) and card.get_viewport() == app.get_viewport())
	var exclusive := 0
	for w in app.get_tree().root.find_children("*", "Window", true, false):
		exclusive += int((w as Window).visible and (w as Window).exclusive)
	_check(ctx, "no exclusive window is open with it", exclusive == 0, "%d" % exclusive)
	var yes: Button = card.find_child("BankSitesOfferYes", true, false)
	var no: Button = card.find_child("BankSitesOfferNo", true, false)
	_check(ctx, "it has its two buttons", yes != null and no != null and yes.text == "Update" and no.text == "Keep as it is")
	if _form != "desktop":
		## The phone draws at its own scale: 44 dp is 44 times it.
		var floor_px: float = TOUCH_MIN_DP * (float(app._pscale(1000)) / 1000.0 if _form == "phone" else 1.0)
		_check(ctx, "both buttons are 44 dp tall or more", yes.size.y >= floor_px - 0.5 and no.size.y >= floor_px - 0.5, "%.1f and %.1f px against %.1f" % [yes.size.y, no.size.y, floor_px])
	## The map under it still answers: a wheel step at the middle of the map's lower half changes the zoom.
	var host = app.viewport
	var z0: float = host.zoom()
	var at := Vector2(size.x * 0.5, size.y * 0.7)
	for pressed in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_WHEEL_UP
		ev.pressed = pressed
		ev.position = at
		ev.global_position = at
		ev.factor = 1.0
		app.get_viewport().push_input(ev)
		await ctx.wait(0.1)
	await ctx.wait(0.5)
	var z1: float = host.zoom()
	_check(ctx, "the map zooms under the card (it blocks nothing)", z1 > z0, "zoom %.3f to %.3f" % [z0, z1])
	await _shot(ctx, "offer")

	## 2. no
	var dirty0: bool = bridge.world_dirty
	await DriveCtx.click(ctx, DriveCtx.rect_of(no).get_center())
	await ctx.wait(0.6)
	_check(ctx, "Keep closes the card", _card() == null)
	_check(ctx, "Keep leaves the switch off", bridge.param_get("civ.bank_sites") == false)
	_check(ctx, "Keep changes nothing in the world", bridge.world_dirty == dirty0 and not bridge.world_dirty)
	_check(ctx, "Keep is not the user's rule for a new world", bridge.world_gen.get_new_world_params()["civ.bank_sites"] == true)
	_check(ctx, "the answer is recorded for this world", DccSettings.bank_offer_answer(key) == "no", DccSettings.bank_offer_answer(key))

	## 3. never again
	_check(ctx, "the same save opens again", await _open(ctx, old))
	_check(ctx, "and is not asked again", _card() == null and String(bridge.world_gen.bank_sites_offer()) == key)
	_check(ctx, "a renamed copy opens", await _open(ctx, copy))
	_check(ctx, "and is not asked either (the record is the world's)", _card() == null and String(bridge.world_gen.bank_sites_offer()) == key)

	## 4. yes
	DccSettings.set_bank_offer_answer(key, "")
	_check(ctx, "with the record cleared the save opens", await _open(ctx, old))
	card = _card()
	_check(ctx, "and is asked again (the control for step 3)", card != null)
	if card == null:
		ctx.log("RESULT FAIL (%d failed)" % _fails)
		return
	var places: Array = bridge.settlements()
	var all := PackedInt32Array()
	for k in places.size():
		all.append(k)
	_check(ctx, "no town is moved before the answer", bridge.world_gen.urban_layout_boxes().is_empty() or not bridge.world_gen.town_bank_census(all, 1150.0, 800.0).any(func(row): return row.has("origin_m")))
	## The largest settlement is frozen before the click.
	var fk := 0
	for k in places.size():
		if int(places[k].get("pop", 0)) > int(places[fk].get("pop", 0)):
			fk = k
	var froze: bool = bridge.town_freeze(fk)
	var before: Array = bridge.urban_layouts(PackedInt32Array([fk]))
	_check(ctx, "#%d is frozen before the answer" % fk, froze and before.size() == 1 and bool(before[0].get("frozen", false)))
	yes = card.find_child("BankSitesOfferYes", true, false)
	await DriveCtx.click(ctx, DriveCtx.rect_of(yes).get_center())
	await ctx.wait(1.0)
	_check(ctx, "Update closes the card", _card() == null)
	_check(ctx, "Update turns the switch on", bridge.param_get("civ.bank_sites") == true)
	_check(ctx, "Update marks the world changed", bool(bridge.world_dirty))
	_check(ctx, "the answer is recorded", DccSettings.bank_offer_answer(key) == "yes")
	var rows: Array = bridge.world_gen.town_bank_census(all, 1150.0, 800.0)
	var moved := 0
	for row in rows:
		moved += int((row as Dictionary).has("origin_m"))
	_check(ctx, "unfrozen towns are laid out again on the bank", moved > 0, "%d of %d laid round a point that is not their cell" % [moved, rows.size()])
	var after: Array = bridge.urban_layouts(PackedInt32Array([fk]))
	_check(ctx, "the frozen town #%d is served unchanged (pin, market and every building)" % fk, after.size() == 1 and bool(after[0].get("frozen", false))
		and after[0]["pin"] == before[0]["pin"] and after[0]["market"] == before[0]["market"] and str(after[0]["buildings"]) == str(before[0]["buildings"]))
	var why := str(bridge.town_state(fk, true).get("reasons", ""))
	_check(ctx, "and its check reports the rules change", why.find("civ bank origin is now on") >= 0 or why.find("the ground or roads under this town changed") >= 0 or why.find("rules") >= 0, why.left(240))
	await _shot(ctx, "after_yes")

	## 5. a new save is silent
	var fresh := _out.path_join("new_%s.cartalith" % _form)
	var sv: Dictionary = bridge.project_save_with_documents(fresh, app._project_documents())
	_check(ctx, "the updated world saves", bool(sv.get("ok", false)), str(sv).left(200))
	DccSettings.set_bank_offer_answer(key, "")
	_check(ctx, "the new save opens", await _open(ctx, fresh))
	_check(ctx, "with the switch on and no card (its file carries the key)", _card() == null and bridge.param_get("civ.bank_sites") == true and String(bridge.world_gen.bank_sites_offer()) == "")
	bridge.param_set("civ.bank_sites", false)
	var off_save := _out.path_join("new_off_%s.cartalith" % _form)
	sv = bridge.project_save_with_documents(off_save, app._project_documents())
	_check(ctx, "a save with the switch turned off by hand opens", bool(sv.get("ok", false)) and await _open(ctx, off_save))
	_check(ctx, "with the switch off and no card (an explicit off is an answer)", _card() == null and bridge.param_get("civ.bank_sites") == false and String(bridge.world_gen.bank_sites_offer()) == "")
	ctx.log("RESULT %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])

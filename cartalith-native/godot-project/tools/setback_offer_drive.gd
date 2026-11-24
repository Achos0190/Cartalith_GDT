extends RefCounted
## Ruling HS (owner, 2026-11-23): a world saved after towns sat on the bank but before the coastal and floodplain
## setback is offered, once, to turn the two setback switches on, on Ruling HN's card (`app.gd::_offer_bank_sites`,
## `WorldGen.town_rules_offer`). Driven through the REAL shell, desktop:
##   CS_SCRIPT=res://tools/setback_offer_drive.gd python -I docs/evidence/coastal-setback-2026-11-23/run_frames.py ROOT OUT 900 HS_OUT=<dir> HS_SAVE=<file>
## Inputs (environment): HS_OUT, the directory the copy and the frame go to; HS_SAVE, a real save written by a build
## before the setback (with `civ.bank_sites` in it and its substrate), copied to HS_OUT so it is never written.
## Checks (each prints `ok` or `FAIL`; the launcher exits 1 on any FAIL):
##   1. the engine offers exactly the two setback switches, under a key of its own (`-hs`, so an answered HN offer
##      does not hide it), and the card shows the setback sentence, as a literal;
##   2. a real click on "Update" turns both switches on, leaves `civ.bank_sites` as it was, marks the world changed,
##      and records "yes" for that key;
##   3. the same save opened again (the answer recorded) shows no card.

const SENTENCE := "This world was made before coastal and floodplain markets were placed by their hazards — update?"

var _fails := 0
var app: Node
var bridge


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _open(ctx, path: String) -> bool:
	var ok: bool = await app.open_project_busy(path)
	await ctx.wait(1.5)
	if app.get("open_project_dialog") != null and app.open_project_dialog:
		app.open_project_dialog.hide()
	return ok


func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not ("bridge" in app):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	var out := OS.get_environment("HS_OUT")
	var src := OS.get_environment("HS_SAVE")
	if out == "" or src == "":
		ctx.error("HS_OUT and HS_SAVE are required")
		return
	await ctx.wait(1.0)
	for e in DriveCtx.dump_ui(ctx, "Continue without a world"):
		var r: Array = e["rect"]
		await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))
		await ctx.wait(0.6)
		break
	var old := out.path_join("pre_hs_copy.cartalith")
	_check(ctx, "the older save is copied", DirAccess.copy_absolute(src, old) == OK, src)
	_check(ctx, "it opens", await _open(ctx, old))
	var keys: PackedStringArray = bridge.world_gen.town_rules_offer()
	var key := String(bridge.world_gen.bank_sites_offer())
	_check(ctx, "the engine offers exactly the two setback switches", keys == PackedStringArray(["urban_coastal_setback", "urban_floodplain_setback"]), str(keys))
	_check(ctx, "under a key of their own", key.length() == 19 and key.ends_with("-hs"), "key '%s'" % key)
	var bank0 = bridge.param_get("civ.bank_sites")
	_check(ctx, "both loaded off", bridge.param_get("urban_coastal_setback") == false and bridge.param_get("urban_floodplain_setback") == false)
	var card := app.get_node_or_null("BankSitesOffer") as Control
	_check(ctx, "one card is shown", card != null and card.visible)
	if card == null:
		ctx.log("RESULT FAIL (%d failed)" % _fails)
		return
	var text: Label = card.find_child("BankSitesOfferText", true, false)
	_check(ctx, "with the setback sentence, word for word", text != null and text.text == SENTENCE, text.text if text != null else "no label")
	await ctx.wait(0.4)
	app.get_viewport().get_texture().get_image().save_png(out.path_join("desktop_setback_offer.png"))
	var yes: Button = card.find_child("BankSitesOfferYes", true, false)
	await DriveCtx.click(ctx, DriveCtx.rect_of(yes).get_center())
	await ctx.wait(1.0)
	_check(ctx, "Update closes the card", app.get_node_or_null("BankSitesOffer") == null)
	_check(ctx, "Update turns both setback switches on", bridge.param_get("urban_coastal_setback") == true and bridge.param_get("urban_floodplain_setback") == true)
	_check(ctx, "and leaves civ.bank_sites as it was", bridge.param_get("civ.bank_sites") == bank0, str(bank0))
	_check(ctx, "Update marks the world changed", bool(bridge.world_dirty))
	_check(ctx, "the answer is recorded", DccSettings.bank_offer_answer(key) == "yes")
	_check(ctx, "the same save opens again", await _open(ctx, old))
	_check(ctx, "and is not asked again", app.get_node_or_null("BankSitesOffer") == null)
	DccSettings.set_bank_offer_answer(key, "")
	ctx.log("RESULT %s (%d failed)" % ["PASS" if _fails == 0 else "FAIL", _fails])

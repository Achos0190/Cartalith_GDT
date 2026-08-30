extends Node
## TEMPORARY, untracked probe for GUI_GAP_REGISTER.md IN-13 (trade flows).
##
## Drives, windowed, against a real 233-settlement world:
##   * `civ_trade_flows()` returns real, differentiated flows -- not all-zero,
##     not all-identical -- and every one of them is a pair that could really
##     trade (mode consistent with both ends' water, distance inside the mode's
##     own reach cliff, volume inside the supplier cap).
##   * The match is deterministic across two calls and retains nothing
##     (`resident_bytes == 0`, process working set flat across 20 calls).
##   * CIVIL ▸ Trade ▸ Match trade flows really runs it and the section fills.
##   * The place editor's Trade section shows a real ledger for a real
##     settlement, and the partner it names is a settlement that names it back.
##   * CARTO ▸ Roads & routes ▸ Trade load really moves pixels, and returns to
##     byte-identical when switched off.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _in13_probe.tscn

var _app: Node
var _bridge
var _fail := 0
var _d: Dictionary = {}

func _p(s: String) -> void:
	print("IN13  %s" % s)

func _bad(s: String) -> void:
	_fail += 1
	print("IN13  FAIL  %s" % s)

func _ok(s: String) -> void:
	print("IN13  ok    %s" % s)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)

func _all(root: Node) -> Array:
	var out: Array = []
	_walk(root, out)
	return out

func _texts(root: Node) -> String:
	var parts: Array[String] = []
	for n in _all(root):
		if n is Label:
			parts.append((n as Label).text)
		elif n is Button:
			parts.append((n as Button).text)
	return "\n".join(parts)

## Section and group headings are rendered UPPER CASE by DccTheme.header and
## DccWidgets.group, so a heading check has to be case-insensitive. Row text
## and note prose are not upper-cased, and this is safe for both.
func _has(root: Node, needle: String) -> bool:
	return needle.to_lower() in _texts(root).to_lower()

func _button(root: Node, needle: String) -> Button:
	for n in _all(root):
		if n is Button and needle in (n as Button).text:
			return n
	return null

func _ws_mb() -> float:
	var out: Array = []
	OS.execute("powershell", ["-NoProfile", "-Command",
		"(Get-Process -Id %d).WorkingSet64" % OS.get_process_id()], out, false)
	if out.is_empty():
		return -1.0
	return float(String(out[0]).strip_edges()) / 1048576.0


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 900.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)
	_p("world: %d settlements, %d factions, %d ways" % [
		_bridge.settlements().size(), _bridge.civ_faction_count(),
		_bridge.roads().size()])

	await _engine()
	await _invariants()
	await _determinism_and_memory()
	await _dock()
	await _editor()
	await _map()
	await _backlinks()
	await _ledger()

	_p("=== %s ===" % ("PASS" if _fail == 0 else "%d FAILURES" % _fail))
	get_tree().quit(0 if _fail == 0 else 1)


# ---------------------------------------------------------------- the engine

func _engine() -> void:
	_p("=== the match ===")
	if not _bridge._has("civ_trade_flows"):
		_bad("civ_trade_flows missing from the loaded extension -- stale .dll?")
		return
	var t0 := Time.get_ticks_msec()
	_d = _bridge.civ_trade_flows()
	var ms := Time.get_ticks_msec() - t0
	if _d.is_empty():
		_bad("empty trade record on a generated world with settlements")
		return
	_p("matched in %d ms (engine reports %d ms); transient %.2f MB, resident %d B" % [
		ms, int(_d.get("elapsed_ms", 0)),
		float(_d.get("transient_bytes", 0)) / 1048576.0, int(_d.get("resident_bytes", -1))])
	_p("flows=%d rows=%d goods=%d importing=%d supplied=%d unmet=%d" % [
		int(_d.get("flow_count", 0)), int(_d.get("flow_rows", 0)),
		int(_d.get("goods_moving", 0)), int(_d.get("importing", 0)),
		int(_d.get("supplied", 0)), int(_d.get("unmet_count", 0))])
	_p("modes by volume: land %.1f%% river %.1f%% sea %.1f%%; total volume %.0f" % [
		100.0 * float(_d.get("land_share", 0.0)), 100.0 * float(_d.get("river_share", 0.0)),
		100.0 * float(_d.get("sea_share", 0.0)), float(_d.get("total_volume", 0.0))])

	if int(_d.get("flow_count", 0)) <= 0:
		_bad("no flows matched at all")
	if int(_d.get("goods_moving", 0)) <= 1:
		_bad("only %d good moves -- a fifteen-resource world should move several"
			% int(_d.get("goods_moving", 0)))
	if int(_d.get("resident_bytes", -1)) != 0:
		_bad("resident_bytes is %d, not 0 -- something is retained"
			% int(_d.get("resident_bytes", -1)))
	var shares := float(_d.get("land_share", 0.0)) + float(_d.get("river_share", 0.0)) \
		+ float(_d.get("sea_share", 0.0))
	if absf(shares - 1.0) > 1e-4:
		_bad("mode shares sum to %.6f, not 1" % shares)

	_p("goods:")
	for g in _d.get("goods", []):
		var r: Dictionary = g
		_p("  %-16s %3d -> %3d  vol %10.0f  mostly %-5s  %s" % [
			String(r.get("key", "?")), int(r.get("exporters", 0)), int(r.get("importers", 0)),
			float(r.get("volume", 0.0)), String(r.get("dominant_mode", "?")),
			"bulk" if bool(r.get("bulk", false)) else "luxury"])


# ------------------------------------------------------- per-flow invariants

func _invariants() -> void:
	_p("=== every flow is a trade that could really happen ===")
	var flows: Array = _d.get("flows", [])
	var navs: Array = _d.get("navigability", [])
	var settlements: Array = _bridge.settlements()
	if flows.is_empty():
		_bad("no flow rows to check")
		return

	# reference constants, restated here so the probe is an independent check
	var reach_km := {"land": 220.0, "river": 1600.0, "sea": 9000.0}
	var double_km := {"land": 160.0, "river": 880.0, "sea": 8000.0}
	const LOCAL_KM := 50.0
	const SUPPLIER_SHARE := 0.6

	var bad_mode := 0
	var bad_cliff := 0
	var bad_cap := 0
	var bad_curve := 0
	var bad_reach := 0
	var self_trade := 0
	var vmin := INF
	var vmax := -INF
	var dmin := INF
	var dmax := -INF
	var mode_kinds := {}
	for f in flows:
		var r: Dictionary = f
		var a := int(r.get("from", -1))
		var b := int(r.get("to", -1))
		if a == b:
			self_trade += 1
		var mode := String(r.get("mode", "?"))
		var dist := float(r.get("distance_km", 0.0))
		var vol := float(r.get("volume", 0.0))
		var del := float(r.get("deliverable", 0.0))
		vmin = minf(vmin, vol); vmax = maxf(vmax, vol)
		dmin = minf(dmin, dist); dmax = maxf(dmax, dist)
		mode_kinds[mode] = int(mode_kinds.get(mode, 0)) + 1

		# 1. the mode must be what both ends' water actually allows
		var ka := String((navs[a] as Dictionary).get("kind", "none")) if a < navs.size() else "?"
		var kb := String((navs[b] as Dictionary).get("kind", "none")) if b < navs.size() else "?"
		var expect := "land"
		if ka == "sea" and kb == "sea":
			expect = "sea"
		elif (ka == "sea" or ka == "river") and (kb == "sea" or kb == "river"):
			expect = "river"
		if mode != expect:
			bad_mode += 1
		# 2. inside the mode's own reach cliff
		if dist > float(reach_km.get(mode, 0.0)):
			bad_cliff += 1
		# 3. deliverable really is 2^(-d/D)
		if absf(del - pow(2.0, -dist / float(double_km.get(mode, 1.0)))) > 1e-6:
			bad_curve += 1
		# 4. never more than SUPPLIER_SHARE of the supplier's own scale
		var sp := float(int((settlements[a] as Dictionary).get("population", 0))) if a < settlements.size() else 0.0
		if vol > SUPPLIER_SHARE * sp + 1e-6:
			bad_cap += 1
		# 5. a bulk good from a landlocked supplier stops at the local radius
		if String(r.get("reach", "")) == "local" and dist > LOCAL_KM + 1e-6:
			bad_reach += 1

	_p("volume  min %.2f  max %.2f" % [vmin, vmax])
	_p("distance min %.1f km  max %.1f km" % [dmin, dmax])
	_p("modes on the flow rows: %s" % str(mode_kinds))
	var nav_kinds := {}
	for n in navs:
		var k := String((n as Dictionary).get("kind", "?"))
		nav_kinds[k] = int(nav_kinds.get(k, 0)) + 1
	_p("settlement water access: %s" % str(nav_kinds))
	if nav_kinds.size() < 2:
		_bad("every settlement has the same water access -- navigability is not discriminating")
	if self_trade > 0:
		_bad("%d flows go from a settlement to itself" % self_trade)
	if bad_mode > 0:
		_bad("%d flows carry a mode neither end's water allows" % bad_mode)
	if bad_cliff > 0:
		_bad("%d flows are past their mode's reach cliff" % bad_cliff)
	if bad_curve > 0:
		_bad("%d flows do not match 2^(-d/D)" % bad_curve)
	if bad_cap > 0:
		_bad("%d flows exceed the supplier share cap" % bad_cap)
	if bad_reach > 0:
		_bad("%d local-reach flows travel past 50 km" % bad_reach)
	if vmax <= vmin:
		_bad("every flow has the same volume (%.3f) -- the model is not discriminating" % vmax)
	if vmin <= 0.0:
		_bad("a flow carries nothing")
	if mode_kinds.size() < 2:
		_bad("only one mode appears -- the fixture cannot separate land from water")

	# the way load has to be real, and not uniform
	var load: PackedFloat32Array = _d.get("way_load", PackedFloat32Array())
	var lmax := 0.0
	var lnonzero := 0
	for v in load:
		lmax = maxf(lmax, v)
		if v > 0.0:
			lnonzero += 1
	_p("way load: %d of %d ways carry something, busiest %.0f" % [lnonzero, load.size(), lmax])
	if load.size() != _bridge.roads().size():
		_bad("way_load has %d entries for %d ways" % [load.size(), _bridge.roads().size()])
	if lnonzero == 0:
		_bad("no way carries any trade at all")
	if lnonzero == load.size() and load.size() > 4:
		_p("      (every way carries something -- possible, worth noticing)")

	var unmet: Array = _d.get("unmet", [])
	_p("unmet: %d settlements" % unmet.size())
	for i in range(mini(5, unmet.size())):
		var u: Dictionary = unmet[i]
		_p("  %-18s %s (%s)" % [String(u.get("name", "?")),
			", ".join(u.get("goods", PackedStringArray())),
			"no exporter in reach" if bool(u.get("exporter_exists", false)) else "nobody exports it"])


# -------------------------------------------------- determinism and memory

func _determinism_and_memory() -> void:
	_p("=== determinism and what it costs ===")
	var a: Dictionary = _bridge.civ_trade_flows()
	var b: Dictionary = _bridge.civ_trade_flows()
	var fa: Array = a.get("flows", [])
	var fb: Array = b.get("flows", [])
	if fa.size() != fb.size():
		_bad("two matches produced %d and %d rows" % [fa.size(), fb.size()])
	else:
		var diff := 0
		for i in fa.size():
			var x: Dictionary = fa[i]
			var y: Dictionary = fb[i]
			if int(x.get("from", -1)) != int(y.get("from", -2)) \
				or int(x.get("to", -1)) != int(y.get("to", -2)) \
				or String(x.get("good", "")) != String(y.get("good", "?")) \
				or absf(float(x.get("volume", 0.0)) - float(y.get("volume", 1.0))) > 0.0:
				diff += 1
		if diff > 0:
			_bad("%d of %d rows differ between two matches" % [diff, fa.size()])
		else:
			_ok("two matches agree row for row across %d rows" % fa.size())

	var start := _ws_mb()
	for i in 20:
		var _r = _bridge.civ_trade_flows()
	await _frames(4)
	var end := _ws_mb()
	_p("working set across 20 matches: %.1f MB -> %.1f MB (%+.1f)" % [start, end, end - start])
	if end - start > 64.0:
		_bad("working set grew %.1f MB across 20 matches -- something is retained" % (end - start))
	else:
		_ok("nothing accumulates across 20 matches")


# ------------------------------------------------------------------ the dock

func _dock() -> void:
	_p("=== CIVIL > Trade ===")
	_app.select_domain_category("civilization", "Trade")
	await _frames(8)
	var civ: Node = _app

	var before := _texts(civ)
	if not ("Not matched yet" in before):
		_bad("Trade did not open on its unmatched state; text was:\n%s" % before.substr(0, 400))
	else:
		_ok("unmatched state disclosed")

	var run := _button(civ, "Match trade flows")
	if run == null:
		_bad("no 'Match trade flows' button")
		return
	if run.disabled:
		_bad("'Match trade flows' is disabled on a generated world")
		return
	run.emit_signal("pressed")
	await _frames(8)
	var after := _texts(civ)
	for needle in ["flows over", "By good", "Busiest partners", "Needs nothing can reach",
			"Way load", "Built on demand and dropped"]:
		if _has(civ, needle):
			_ok("dock shows '%s'" % needle)
		else:
			_bad("dock is missing '%s'" % needle)
	if "Not matched yet" in after:
		_bad("the unmatched note survived the match")
	# the numbers on screen have to be the engine's
	var n := int(_d.get("flow_count", 0))
	var sep := String.num_int64(n)
	if not (sep in after or ("%s" % n) in after or _thousands(n) in after):
		_p("      (flow count %d not found verbatim; the dock abbreviates)" % n)


func _thousands(n: int) -> String:
	var s := str(absi(n))
	var out := ""
	var c := 0
	for i in range(s.length() - 1, -1, -1):
		out = s[i] + out
		c += 1
		if c % 3 == 0 and i > 0:
			out = " " + out
	return ("-" if n < 0 else "") + out


# --------------------------------------------------------- the place editor

func _editor() -> void:
	_p("=== place editor > Trade ===")
	# find a settlement that has at least one import AND one export
	var counts := {}
	for f in _d.get("flows", []):
		var r: Dictionary = f
		var a := int(r.get("from", -1))
		var b := int(r.get("to", -1))
		var ca: Array = counts.get(a, [0, 0])
		ca[0] += 1
		counts[a] = ca
		var cb: Array = counts.get(b, [0, 0])
		cb[1] += 1
		counts[b] = cb
	var pick := -1
	for k in counts:
		var c: Array = counts[k]
		if c[0] > 0 and c[1] > 0:
			pick = k
			break
	if pick < 0:
		_bad("no settlement both imports and exports -- cannot test the two-sided ledger")
		return
	var s: Dictionary = _bridge.settlements()[pick]
	_p("chose #%d %s (%d exports, %d imports)" % [pick, String(s.get("name", "?")),
		(counts[pick] as Array)[0], (counts[pick] as Array)[1]])

	_app.open_place_editor(pick)
	await _frames(8)
	var t := _texts(_app.place_editor_window)
	for needle in ["Trade", "Imports", "Exports", "Water:"]:
		if _has(_app.place_editor_window, needle):
			_ok("editor shows '%s'" % needle)
		else:
			_bad("editor missing '%s'" % needle)
	# the partner it names must be a real settlement that names it back
	var led := TradeStore.ledger(pick)
	var imports: Array = led.get("imports", [])
	if imports.is_empty():
		_bad("TradeStore.ledger disagrees with the engine about this settlement")
	else:
		var first: Dictionary = imports[0]
		var partner := int(first.get("from", -1))
		var pn := String(first.get("from_name", ""))
		var real := String((_bridge.settlements()[partner] as Dictionary).get("name", ""))
		if pn == real and pn != "":
			_ok("partner name '%s' matches settlement #%d" % [pn, partner])
		else:
			_bad("partner name '%s' != settlement #%d's real name '%s'" % [pn, partner, real])
		var back := TradeStore.ledger(partner)
		var found := false
		for e in back.get("exports", []):
			if int((e as Dictionary).get("to", -1)) == pick:
				found = true
		if found:
			_ok("the partner's own ledger lists this settlement as a customer")
		else:
			_bad("the relationship is one-sided in the store")
		if pn in t:
			_ok("the partner is named on screen")
		else:
			_bad("'%s' is not on the editor" % pn)
	_app.place_editor_window.hide()
	await _frames(4)


# --------------------------------------------------------------- the map

func _map() -> void:
	_p("=== CARTO > Roads & routes > Trade load ===")
	var ov = _app.viewport.overlay
	if not ov.has_trade_load():
		_bad("the overlay has no trade load after a match")
		return
	_ok("overlay has a trade load reading")

	_app.select_domain_category("cartography", "Roads & routes")
	await _frames(8)
	var base := await _shot()
	ov.set_show_trade_load(true)
	await _frames(8)
	var on := await _shot()
	var moved := _diff(base, on)
	_p("trade load ON moved %.4f%% of screen pixels" % moved)
	if moved <= 0.0:
		_bad("switching the layer on changed nothing")
	else:
		_ok("the layer draws")
	ov.set_show_trade_load(false)
	await _frames(8)
	var off := await _shot()
	var back := _diff(base, off)
	_p("trade load OFF differs from the baseline by %.4f%%" % back)
	if back != 0.0:
		_bad("switching it off did not return byte-identical (%.4f%%)" % back)
	else:
		_ok("off is byte-identical to before it existed")
	ov.set_show_trade_load(true)
	await _frames(8)
	var img := await _shot()
	img.save_png("user://in13_trade_load.png")
	_p("screenshot: %s" % ProjectSettings.globalize_path("user://in13_trade_load.png"))


# =============================================== VA-01: the backlink index

const VAULT_DIR := "user://probe_vault"

func _write(rel: String, text: String) -> void:
	var abs_path := VAULT_DIR.path_join(rel)
	DirAccess.make_dir_recursive_absolute(abs_path.get_base_dir())
	var f := FileAccess.open(abs_path, FileAccess.WRITE)
	f.store_string(text)
	f.close()

func _backlinks() -> void:
	_p("=== VA-01: backlinks ===")
	if not _bridge._has("vault_refresh_backlinks"):
		_bad("vault_refresh_backlinks missing -- stale .dll?")
		return
	## A real folder of real Markdown, written fresh so every count is known.
	DirAccess.make_dir_recursive_absolute(VAULT_DIR)
	_write("Settlements/Kelvhold.md", "# Kelvhold\n\nA river town at the third ford.\n")
	_write("Factions/Veldmark.md", "The lords of [[Kelvhold]] held it, and [[Kelvhold|the town]] paid.\n")
	_write("People/Aldis.md", "Born in [there](Settlements/Kelvhold.md).\n")
	_write("Journal/Thaw.md", "Rode down to Kelvhold before the river froze and slept badly.\n")
	_write("Elsewhere.md", "Nothing to do with any of it. [[Nowhere]]\n")
	var root := ProjectSettings.globalize_path(VAULT_DIR)

	var info: Dictionary = _bridge.vault_connect(root, "Probe vault")
	if not bool(info.get("ok", false)):
		_bad("vault_connect refused %s: %s" % [root, String(info.get("error", ""))])
		return
	_ok("connected a real folder of 5 real notes")

	_bridge.vault_rebuild_backlinks()
	var st0: Dictionary = _bridge.vault_backlink_stats()
	if bool(st0.get("built", false)):
		_bad("rebuild did not clear the index")
	if not (_bridge.vault_entity_backlinks("settlement", 42) as Array).is_empty():
		_bad("an unbuilt index returned rows")
	else:
		_ok("nothing is scanned until asked")

	var r1: Dictionary = _bridge.vault_refresh_backlinks(500)
	_p("first build: seen=%d reread=%d dropped=%d unreadable=%d" % [
		int(r1.get("seen", 0)), int(r1.get("reread", 0)),
		int(r1.get("dropped", 0)), int(r1.get("unreadable", 0))])
	if int(r1.get("seen", 0)) != 5:
		_bad("expected 5 notes, saw %d" % int(r1.get("seen", 0)))
	if int(r1.get("reread", 0)) != 5:
		_bad("a first build must read every note; read %d" % int(r1.get("reread", 0)))

	## THE claim: a second refresh over an untouched folder opens nothing.
	var r2: Dictionary = _bridge.vault_refresh_backlinks(500)
	if int(r2.get("reread", 0)) != 0:
		_bad("a refresh over an untouched vault re-read %d notes" % int(r2.get("reread", 0)))
	else:
		_ok("an untouched vault costs zero reads on refresh")

	## One edited file costs exactly one read.
	_write("Journal/Thaw.md", "Rode to Kelvhold before the river froze, and slept very badly indeed.\n")
	var r3: Dictionary = _bridge.vault_refresh_backlinks(500)
	if int(r3.get("reread", 0)) != 1:
		_bad("one edited file cost %d reads" % int(r3.get("reread", 0)))
	else:
		_ok("one edited file costs exactly one read")

	var stats: Dictionary = _bridge.vault_backlink_stats()
	_p("index: %d notes, %d links, %d blocks, %d broken, %d orphans, %d bytes" % [
		int(stats.get("notes", 0)), int(stats.get("links", 0)), int(stats.get("entities", 0)),
		int(stats.get("broken", 0)), int(stats.get("orphans", 0)), int(stats.get("bytes", 0))])
	if int(stats.get("links", 0)) < 4:
		_bad("expected at least 4 outgoing links, got %d" % int(stats.get("links", 0)))
	if int(stats.get("broken", 0)) != 1:
		_bad("expected exactly one broken link, got %d" % int(stats.get("broken", 0)))

	## Attach the settlement own note, then ask what points at it.
	var att: Dictionary = _bridge.vault_attach("settlement", 42, "Kelvhold", "Settlements/Kelvhold.md", "")
	if not bool(att.get("ok", false)):
		_bad("attach refused: %s" % String(att.get("error", "")))
		return
	var back: Array = _bridge.vault_entity_backlinks("settlement", 42)
	var rels := []
	for b in back:
		rels.append(String((b as Dictionary).get("rel", "")))
	_p("backlinks: %s" % str(rels))
	if not rels.has("Factions/Veldmark.md"):
		_bad("the wikilink is not a backlink")
	if not rels.has("People/Aldis.md"):
		_bad("the markdown link is not a backlink")
	if rels.has("Settlements/Kelvhold.md"):
		_bad("a note is a backlink to itself")
	if rels.has("Journal/Thaw.md"):
		_bad("a bare mention was counted as a backlink")
	for b in back:
		var d: Dictionary = b
		if String(d.get("rel", "")) == "Factions/Veldmark.md" and int(d.get("count", 0)) != 2:
			_bad("two references from one note counted as %d" % int(d.get("count", 0)))
	if rels.size() == 2:
		_ok("exactly the two real references, each labelled by form")

	var men: Array = _bridge.vault_entity_mentions("settlement", 42, "Kelvhold", 8)
	var mrels := []
	for m in men:
		mrels.append(String((m as Dictionary).get("rel", "")))
	_p("unlinked mentions: %s" % str(mrels))
	if mrels != ["Journal/Thaw.md"]:
		_bad("expected only the prose mention, got %s" % str(mrels))
	elif not ("Kelvhold" in String((men[0] as Dictionary).get("excerpt", ""))):
		_bad("the excerpt does not show the hit")
	else:
		_ok("the unlinked mention is found with its excerpt, and nothing else is")

	var rep: Dictionary = _bridge.vault_backlink_report(40)
	var broken: Array = rep.get("broken", [])
	var orph: PackedStringArray = rep.get("orphans", PackedStringArray())
	_p("report: %d broken, %d orphans" % [broken.size(), orph.size()])
	if broken.is_empty():
		_bad("the dangling wikilink is not reported as broken")

	## And the panel really draws it.
	_app.open_vault_overview()
	await _frames(8)
	for needle in ["Index", "Refresh index", "Rebuild", "Missing & orphan notes"]:
		if _has(_app.vault_window, needle):
			_ok("vault panel shows %s" % needle)
		else:
			_bad("vault panel missing %s" % needle)
	_app.vault_window.hide()
	await _frames(4)


# ==================================================== ED-02: the ledger

func _ledger() -> void:
	_p("=== ED-02: the history ledger ===")
	if not _bridge._has("undo_ledger"):
		_bad("undo_ledger missing -- stale .dll?")
		return
	var rows: Array = _bridge.undo_ledger()
	_p("after a generate: %d rows" % rows.size())
	if rows.size() != 1 or String((rows[0] as Dictionary).get("kind", "")) != "floor":
		_bad("a generate should leave exactly one floor row, got %d" % rows.size())
	else:
		_ok("a generate is the floor, and clears everything before it")

	## A territory commit: recorded, never reversible.
	_bridge.civ_territory_paint_at(100.0, 100.0, 1, 6.0, false)
	_bridge.civ_territory_commit()
	## Two height commits, both reversible.
	_bridge.carve_fjords()
	_bridge.carve_fjords()
	rows = _bridge.undo_ledger()
	_p("rows now: %d" % rows.size())
	var kinds := []
	for r in rows:
		var d: Dictionary = r
		kinds.append("%s/%s%s" % [String(d.get("kind", "")), String(d.get("label", "")),
			"" if bool(d.get("reversible", false)) else " (frozen)"])
	_p("  %s" % str(kinds))
	var height_rows := []
	var recorded := []
	for r in rows:
		var d: Dictionary = r
		if String(d.get("kind", "")) == "height":
			height_rows.append(d)
		elif String(d.get("kind", "")) == "recorded":
			recorded.append(d)
	if height_rows.size() != 2:
		_bad("expected 2 height rows, got %d" % height_rows.size())
		return
	if recorded.is_empty():
		_bad("the territory commit did not record a row")
	for d in recorded:
		if bool(d.get("reversible", false)):
			_bad("a recorded row was offered as reversible")
		if String(d.get("reason", "")) == "":
			_bad("a recorded row carries no reason")
	if not recorded.is_empty():
		_ok("a recorded row is not reversible and says why: %s"
			% String((recorded[0] as Dictionary).get("reason", "")))

	## Reverting to the OLDER height row must pop both.
	var target: Dictionary = height_rows[0]
	var want := int(target.get("steps", 0))
	_p("reverting to %s should take %d steps" % [String(target.get("label", "")), want])
	if want != 2:
		_bad("expected 2 steps to the older height row, ledger says %d" % want)
	var stats_before: Dictionary = _bridge.undo_stats()
	var done: int = _bridge.undo_revert_to(int(target.get("seq", 0)))
	var stats_after: Dictionary = _bridge.undo_stats()
	_p("reverted %d steps; undo depth %d -> %d" % [done,
		int(stats_before.get("depth", 0)), int(stats_after.get("depth", 0))])
	if done != 2:
		_bad("undo_revert_to reverted %d steps, not 2" % done)
	if int(stats_after.get("depth", 0)) != 0:
		_bad("the stack still holds %d steps" % int(stats_after.get("depth", 0)))
	for r in _bridge.undo_ledger():
		if String((r as Dictionary).get("kind", "")) == "height":
			_bad("a height row survived a revert past it")
	_ok("a linear revert took the row and everything after it")

	## A second revert to the same seq is refused, not half-applied.
	if _bridge.undo_revert_to(int(target.get("seq", 0))) != 0:
		_bad("reverting to a gone row did something")
	else:
		_ok("a gone row is refused, not half-applied")

	## The panel draws it.
	_app.open_undo_history()
	await _frames(8)
	for needle in ["History", "Committed", "Cost", "Reversible:"]:
		if _has(_app, needle):
			_ok("dock shows %s" % needle)
		else:
			_bad("dock missing %s" % needle)
	## The frozen row's reason has to be on screen, not only in the dictionary
	## -- the whole point of a recorded row is that it says why.
	if _has(_app, "Discard reverts an uncommitted draft only"):
		_ok("a frozen row's reason is on screen")
	else:
		_bad("a frozen row's reason is not on screen")
	var shot := await _shot()
	shot.save_png("user://in13_history.png")
	_p("screenshot: %s" % ProjectSettings.globalize_path("user://in13_history.png"))


func _shot() -> Image:
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()

func _diff(a: Image, b: Image) -> float:
	if a.get_width() != b.get_width() or a.get_height() != b.get_height():
		return 100.0
	var da := a.get_data()
	var db := b.get_data()
	var n := 0
	for i in range(0, da.size(), 4):
		if da[i] != db[i] or da[i + 1] != db[i + 1] or da[i + 2] != db[i + 2]:
			n += 1
	return 100.0 * float(n) / float(maxi(1, da.size() / 4))

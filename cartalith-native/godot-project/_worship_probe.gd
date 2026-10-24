extends Node
## FH-R2 S3 probe (Ruling CY, 2026-10-25): places of worship drawn in the City
## Viewer in their rite's own built form, and the rite following the ruling
## faction's religion.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _worship_probe.tscn -- --out <dir>
##   ... _worship_probe.tscn -- --out <dir> --force-touch --vp 2560x1600   (tablet)
##   ... _worship_probe.tscn -- --out <dir> --force-touch --vp 1080x2340   (phone)
##
## Windowed, never --headless: it reads rendered pixels, and headless never
## fires `frame_post_draw` (MISTAKES.md). Navigation is by function call
## (`app.open_city_viewer`), so this measures "it renders", not "it can be
## found" on a phone.
##
## On seed 483920 (2400 km, 384 x 288, the faction-hub probe's world) it picks
## five towns that each have a place of worship, in five different factions,
## and sets those factions' religions to `sun_cult` (temple), `earth_mother`
## (shrine), `revealed_law` (mosque), `one_god` (church) and `none`
## (unaffiliated: the culture's default rite). Then, per town:
##  (a) the layout carries `"worship_sites"` in the expected rite, with the
##      same site count, centres and churchyard parcels as before the religion
##      edit, and byte-identical parcels;
##  (b) every house that appeared or vanished has its centroid inside a site's
##      building or precinct, before or after (the change is explained by the
##      building);
##  (c) the City Viewer draws it: removing `"worship_sites"` from the shown
##      layout moves pixels (the positive control), and the legend and the
##      info column name the form;
##  (d) after save and reopen, the five layouts are byte-identical;
##  (e) a religion edit through the faction hub's own setter (`_set_field`)
##      empties the map overlay's town cache, so the map redraws the new rite.
## Checks (a)'s rite and (b) need the rite switched on (`urban_bridge.rs`'s
## `faith`); on a tree without that they fail, which is the point.
## Screenshots: `<out>/<form>_<rite>_<religion>_fit.png`, `..._zoom.png` and `..._zoom_without.png`.
## Result: `WORSHIP ok|FAIL <name>` lines, `WORSHIP RESULT PASS|FAIL checks=<n>
## fails=<n>`; exit 0 on PASS, 1 on a failed check, 2 on a bad argument or a
## world that cannot answer (fewer than five usable towns).

var _app: Node
var _vp: SubViewport
var _out := ""
var _fail := 0
var _checks := 0
var _form := "desktop"

## Religion key -> the rite the table maps it to (`religion_profile::
## religion_rite`); `none` expects the medieval culture's default, `church`.
const PLAN := [["sun_cult", "temple"], ["earth_mother", "shrine"],
	["revealed_law", "mosque"], ["one_god", "church"], ["none", "church"]]


func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("WORSHIP %s %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _finish() -> void:
	print("WORSHIP RESULT %s checks=%d fails=%d" % ["PASS" if _fail == 0 else "FAIL", _checks, _fail])
	get_tree().quit(0 if _fail == 0 else 1)


func _ready() -> void:
	var vp_size := Vector2i.ZERO
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--vp" and i + 1 < args.size():
			var wh := args[i + 1].split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a == "--out" and i + 1 < args.size():
			_out = args[i + 1]
			i += 2
		elif a == "--force-touch":
			i += 1
		else:
			print("WORSHIP ABORT: unknown argument '%s'" % a)
			get_tree().quit(2)
			return
	if _out == "":
		print("WORSHIP ABORT: --out <dir> is required")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	if vp_size != Vector2i.ZERO:
		_vp = SubViewport.new()
		_vp.size = vp_size
		_vp.transparent_bg = false
		_vp.gui_embed_subwindows = true
		_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(_vp)
		_app = load("res://shell/app.tscn").instantiate()
		_vp.add_child(_app)
	else:
		_app = load("res://shell/app.tscn").instantiate()
		add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge
	bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)
	_form = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("WORSHIP form=%s" % _form)

	# -- pick five towns with a site, in five different factions --------------
	var places: Array = bridge.settlements()
	var by_faction := {}
	var with_site := 0
	for k in places.size():
		var got: Array = bridge.urban_layouts(PackedInt32Array([k]))
		if got.is_empty():
			continue
		var l: Dictionary = got[0]
		if not l.has("worship_sites"):
			continue
		if (l["worship_sites"] as Array).is_empty():
			continue
		with_site += 1
		var f := int((places[k] as Dictionary).get("faction", 0))
		if f <= 0:
			continue
		## The largest site-bearing town of each faction: most to look at.
		var pop := int((places[k] as Dictionary).get("population", 0))
		if not by_faction.has(f) or pop > int(by_faction[f][1]):
			by_faction[f] = [k, pop]
	print("WORSHIP world: %d settlements, %d with a place of worship, %d factions hold one"
		% [places.size(), with_site, by_faction.size()])
	_check("the layout dictionary carries worship_sites", with_site > 0)
	if by_faction.size() < PLAN.size():
		print("WORSHIP ABORT: need %d factions with a site-bearing town" % PLAN.size())
		get_tree().quit(2)
		return
	var fids: Array = by_faction.keys()
	fids.sort()
	var targets: Array = []
	for j in PLAN.size():
		targets.append({"faction": int(fids[j]), "index": int(by_faction[fids[j]][0]),
			"religion": PLAN[j][0], "rite": PLAN[j][1]})

	# -- before the religion edit -------------------------------------------
	for t in targets:
		t["before"] = bridge.urban_layouts(PackedInt32Array([t["index"]]))[0]
	for t in targets:
		_check("religion %s set on faction %d" % [t["religion"], t["faction"]],
			bridge.civ_set_faction_field(t["faction"], "religion", t["religion"]))

	# -- (a) and (b): the layout after the edit -----------------------------
	for t in targets:
		var a: Dictionary = t["before"]
		var b: Dictionary = bridge.urban_layouts(PackedInt32Array([t["index"]]))[0]
		t["after"] = b
		var tag := "%s town #%d %s" % [t["rite"], t["index"], places[t["index"]].get("name", "?")]
		var sa: Array = a["worship_sites"]
		var sb: Array = b["worship_sites"]
		var rites := []
		for s in sb:
			rites.append(String(s.get("faith", "")))
		_check("(a) %s: every site is the %s rite" % [tag, t["rite"]],
			rites.size() > 0 and rites.count(t["rite"]) == rites.size(), str(rites))
		var same_sites := sa.size() == sb.size()
		if same_sites:
			for q in sa.size():
				if (sa[q]["center"] as Vector2) != (sb[q]["center"] as Vector2) or String(sa[q]["id"]) != String(sb[q]["id"]):
					same_sites = false
		_check("(a) %s: site count and centres unchanged" % tag, same_sites,
			"%d -> %d sites" % [sa.size(), sb.size()])
		_check("(a) %s: parcels byte-identical" % tag,
			var_to_bytes(a.get("parcels", [])) == var_to_bytes(b.get("parcels", [])))
		var moved := _house_diff(a, b)
		var unexplained := 0
		for c in moved:
			if not _in_any_site(c, sa) and not _in_any_site(c, sb):
				unexplained += 1
		print("WORSHIP %s: %d sites, buildings %d -> %d, %d appeared or vanished, %d unexplained"
			% [tag, sb.size(), (a["buildings"] as Array).size(), (b["buildings"] as Array).size(),
				moved.size(), unexplained])
		_check("(b) %s: every moved house sits under a site, before or after" % tag, unexplained == 0,
			"%d of %d" % [unexplained, moved.size()])

	# -- (c): drawn in the City Viewer --------------------------------------
	for t in targets:
		_app.open_city_viewer(t["index"])
		await _frames(10)
		var cv = _app.city_viewer_window
		var tag := "%s town #%d" % [t["rite"], t["index"]]
		var shown: Dictionary = cv._layout
		_check("(c) %s: the viewer shows the edited layout" % tag,
			var_to_bytes(shown) == var_to_bytes(t["after"]))
		var legend_ok := _find_text(cv._legend, "Place of worship")
		var site0: Dictionary = (shown.get("worship_sites", []) as Array)[0] if not (shown.get("worship_sites", []) as Array).is_empty() else {}
		var want_name := String(site0.get("name", ""))
		var info_ok := want_name != "" and _find_text(cv._info, want_name)
		_check("(c) %s: legend row and info name the form (%s)" % [tag, want_name], legend_ok and info_ok)
		var fit_img := await _shot(cv, "%s_%s_%s_fit" % [_form, t["rite"], t["religion"]])
		_zoom_on(cv, site0.get("center", Vector2.ZERO), 7.0)
		await _frames(4)
		var zoom_img := await _shot(cv, "%s_%s_%s_zoom" % [_form, t["rite"], t["religion"]])
		var keep = shown["worship_sites"]
		cv._layout.erase("worship_sites")
		cv._canvas.queue_redraw()
		await _frames(4)
		var bare := await _shot(cv, "%s_%s_%s_zoom_without" % [_form, t["rite"], t["religion"]])
		cv._layout["worship_sites"] = keep
		cv._canvas.queue_redraw()
		var moved_px := _diff(zoom_img, bare)
		_check("(c) %s: drawing the site moves pixels (positive control)" % tag, moved_px > 50,
			"%d px" % moved_px)
		_check("(c) %s: the fit view rendered" % tag, fit_img != null and fit_img.get_width() > 0)
		cv.hide()
		await _frames(3)

	# -- (e): a religion edit through the faction hub drops the map's towns --
	## `sky_pantheon` maps to the same rite as `sun_cult`, so target 0's layout
	## (and (d)'s comparison) is unchanged by this edit.
	var ov = _app.viewport.overlay
	var idx := PackedInt32Array()
	for t in targets:
		idx.append(t["index"])
	ov.set_urban_layouts(idx, bridge.urban_layouts(idx))
	var fr = _app.faction_roster_window
	var seeded: int = ov._urban_layouts.size()
	if fr != null and fr.has_method("_set_field"):
		fr._selected = targets[0]["faction"]
		fr._set_field("religion", "sky_pantheon")
	_check("(e) a faction-hub religion edit drops the map's cached towns",
		seeded == targets.size() and ov._urban_layouts.is_empty(),
		"cache %d -> %d" % [seeded, ov._urban_layouts.size()])

	# -- (d): save and reopen -------------------------------------------------
	var path := OS.get_user_data_dir().path_join("_worship_probe.zip")
	_check("(d) save_project accepted", bridge.save_project(path, {}))
	var b2: EngineBridge = EngineBridge.new()
	add_child(b2)
	_check("(d) load_save accepted", b2.load_save(path))
	var same := 0
	for t in targets:
		var r: Array = b2.urban_layouts(PackedInt32Array([t["index"]]))
		if not r.is_empty() and var_to_bytes(r[0]) == var_to_bytes(t["after"]):
			same += 1
	_check("(d) the five towns reopen byte-identical", same == targets.size(), "%d of %d" % [same, targets.size()])
	_finish()


## Centroids of the buildings present in exactly one of `a` and `b`, compared
## by exact polygon, so a house the rite cleared or freed counts once.
func _house_diff(a: Dictionary, b: Dictionary) -> Array:
	var seen := {}
	for bp in a.get("buildings", []):
		seen[var_to_bytes(bp)] = bp
	var out := []
	var in_b := {}
	for bp in b.get("buildings", []):
		var key := var_to_bytes(bp)
		in_b[key] = true
		if not seen.has(key):
			out.append(_centroid(bp))
	for key in seen.keys():
		if not in_b.has(key):
			out.append(_centroid(seen[key]))
	return out


## The AREA centroid, as `build_faith_sites` tests a house by
## (`cartalith-urban`'s `poly_centroid`). A vertex average differs for an
## irregular footprint, and on the first run it called one house straddling a
## nave edge unexplained that the generator's own test had cleared.
## Coordinates are taken relative to the first vertex so the shoelace terms
## stay small in f32 (a layout box runs to ~2 km).
func _centroid(p: PackedVector2Array) -> Vector2:
	if p.is_empty():
		return Vector2.ZERO
	var o := p[0]
	var a := 0.0
	var c := Vector2.ZERO
	for i in p.size():
		var u := p[i] - o
		var v := p[(i + 1) % p.size()] - o
		var k := u.x * v.y - v.x * u.y
		a += k
		c += (u + v) * k
	if absf(a) < 1e-9:
		var m := Vector2.ZERO
		for q in p:
			m += q
		return m / p.size()
	return o + c / (3.0 * a)


## True when `c` is inside a building or precinct polygon of any site, the two
## things `build_faith_sites` clears houses under.
func _in_any_site(c: Vector2, sites: Array) -> bool:
	for s in sites:
		for p in (s.get("polys", []) as Array) + (s.get("open", []) as Array):
			if Geometry2D.is_point_in_polygon(c, p):
				return true
	return false


func _find_text(n: Node, text: String) -> bool:
	for c in n.get_children(true):
		if (c is Label or c is RichTextLabel) and String(c.text).find(text) >= 0:
			return true
		if _find_text(c, text):
			return true
	return false


## Centres the canvas on model point `p` at `zoom` times the fit scale, with
## the viewer's own transform (`_draw_canvas`).
func _zoom_on(cv, p: Vector2, zoom: float) -> void:
	cv._zoom = zoom
	cv._pan = Vector2.ZERO
	var box: Rect2 = cv._fit_box()
	var sz: Vector2 = cv._canvas.size
	var fit: float = minf((sz.x - 32.0) / maxf(1.0, box.size.x), (sz.y - 32.0) / maxf(1.0, box.size.y))
	var vs := fit * zoom
	var origin := (sz - box.size * vs) * 0.5 - box.position * vs
	cv._pan = sz * 0.5 - (origin + p * vs)
	cv._canvas.queue_redraw()


func _shot(cv, name: String) -> Image:
	await RenderingServer.frame_post_draw
	var img: Image = (_vp.get_texture() if _vp != null else cv.get_texture()).get_image()
	var path := _out.path_join(name + ".png")
	img.save_png(path)
	print("WORSHIP shot %s -> %s (%dx%d)" % [name, path, img.get_width(), img.get_height()])
	return img


func _diff(a: Image, b: Image) -> int:
	if a == null or b == null or a.get_size() != b.get_size():
		return -1
	var n := 0
	for y in range(0, a.get_height(), 2):
		for x in range(0, a.get_width(), 2):
			if not a.get_pixel(x, y).is_equal_approx(b.get_pixel(x, y)):
				n += 1
	return n

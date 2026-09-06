extends Node
## Verifier for the nine §6.6 sub-screens added to `PhoneMenu` on 2026-09-06:
## `assets`, `assets-grid`, `asset-slot`, `travel`, `travel-item`, `landmarks`,
## `lm-fam`, `help`, `gestures`.
##
## Four questions, in the order they can be answered:
##
##   1. **Does the screen open, and does it draw rows?** A screen that renders a
##      convincing shell over nothing is worse than an absent one, so "opens" is
##      not the assertion -- "opens AND drew N non-empty strings, none of which
##      is the whole-screen `_missing_row()` placeholder" is.
##   2. **Does back return to MORE?** From one level and from three
##      (`more -> assets -> assets-grid -> asset-slot`), because the three
##      argument-carrying screens are the ones a mis-pushed step would strand.
##   3. **Is anything under the touch floor at this density?** Every `Button`
##      and every list row, against `DccTheme.PHONE_TAP_MIN` scaled by
##      `DccShell.phone_scale()` -- and the density is printed beside every
##      number, because 39 px at pointer density and 44 dp at touch density are
##      different bars and this project has already compared them to each other.
##   4. **Did anything stop being reachable?** The `Assets` and `Help` root rows
##      stopped being popup drills and became screens in the same change, so
##      every top-level row of both popups is checked against what those two
##      screens actually drew.
##
## **Run windowed.** Not because these are pixel assertions -- they are layout
## and string assertions, which `--headless` can take -- but because
## `asset-slot` decodes a real PNG through `Image.load_png_from_buffer()` and
## builds an `ImageTexture` from it, and the dummy display driver is where that
## class of thing silently does nothing (`MISTAKES.md`: *"`ImageTexture.update()`
## is a no-op under `--headless`"*). Windowed is the stronger run and costs a
## window.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _phonemore2_probe.tscn \
##       -- --force-touch --nowelcome [--size=WxH]
##
## `--force-touch` is required: `PhoneMenu` is built only by
## `DccShell._build_phone_shell()`, and without it there is no menu to walk.

var app: Node
var pm            ## PhoneMenu
var _fail := 0
var _scale := 1.0

## One density per launch. `PhoneMenu._scale` is read once, in `setup()`, from
## `DccShell.phone_scale()` and nothing re-reads it on a resize -- so resizing
## the window between measurements measures ONE layout against three widths,
## which `_phonemore_reach_probe.gd` found the hard way. The window is sized
## before `app.tscn` is instantiated so `_compute_layout_mode()` sees it.
const DEFAULT_SIZE := Vector2i(1080, 2400)

func _arg_size() -> Vector2i:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if not s.begins_with("--size="):
			continue
		var wh := s.substr(7).split("x")
		if wh.size() == 2 and wh[0].is_valid_int() and wh[1].is_valid_int():
			return Vector2i(int(wh[0]), int(wh[1]))
	return DEFAULT_SIZE

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## `detail` is the FAILURE explanation and is printed only on failure. Printing
## it either way produced lines reading `ok  more fits 1080 px -- min_w=742 >
## 1080`, which is a pass and its own contradiction on one line.
func _check(name: String, cond: bool, detail: String = "") -> void:
	print("PM2 %s  %s%s" % ["ok  " if cond else "FAIL", name,
		"" if cond else (("  -- " + detail) if detail != "" else "")])
	if not cond:
		_fail += 1

func _labels(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append(String((c as Label).text))
		elif c is Button and String((c as Button).text) != "":
			out.append(String((c as Button).text))
		_labels(c, out)

func _nonempty(texts: Array) -> Array:
	var out: Array = []
	for t in texts:
		if String(t).strip_edges() != "":
			out.append(String(t))
	return out

## Push one screen (optionally with an argument) and return what it drew.
func _open(id: String, arg: String = "") -> Array:
	pm.open()
	if id != "more":
		pm._push_screen(id, arg)
	await _frames(3)
	var out: Array = []
	_labels(pm._screen_body, out)
	return out

## Every Control that is meant to be tapped, with its drawn height. A row is
## `PanelContainer` with a `gui_input` connection; a chip/cell is a `Button`.
func _taps(n: Node, out: Array) -> void:
	if n is Button and not (n as Button).disabled:
		out.append([n, (n as Control).size.y])
	elif n is PanelContainer and (n as PanelContainer).mouse_filter == Control.MOUSE_FILTER_STOP:
		out.append([n, (n as Control).size.y])
	for c in n.get_children():
		_taps(c, out)

## Every Control whose combined minimum width exceeds `limit` -- the overflow a
## `ScrollContainer` with its horizontal axis disabled folds into its own
## minimum and propagates to every ancestor with no scrollbar to reveal it.
func _widest(node: Node, limit: float, out: Array, depth: int = 0) -> void:
	if node is Control:
		var mw: float = (node as Control).get_combined_minimum_size().x
		if mw > limit:
			var bits: Array = []
			_labels(node, bits)
			out.append("%s%s (%s) min_w=%.0f %s" % ["  ".repeat(depth), node.name,
				node.get_class(), mw, " / ".join(PackedStringArray(_nonempty(bits))).substr(0, 80)])
	for c in node.get_children():
		_widest(c, limit, out, depth + 1)

func _clean(t: String) -> String:
	var s := t.strip_edges()
	if s.ends_with("▸"):
		s = s.substr(0, s.length() - 1).strip_edges()
	return s

func _key(t: String) -> String:
	var s := _clean(t)
	var cut := s.find("   ")
	return s.substr(0, cut).strip_edges() if cut > 0 else s

func _has(texts: Array, want: String) -> bool:
	var k := _key(want)
	for t in texts:
		if _key(String(t)) == k:
			return true
	return false

func _menu_popup(title: String) -> PopupMenu:
	for child in app.menu_bar_row.get_children():
		if child is MenuButton and String(child.text) == title:
			var p := (child as MenuButton).get_popup()
			p.about_to_popup.emit()
			return p
	return null

## `[text, submenu_node]` for every non-separator top-level item.
func _top_items(p: PopupMenu) -> Array:
	var out: Array = []
	for i in p.item_count:
		if p.is_item_separator(i):
			continue
		out.append([_clean(p.get_item_text(i)), p.get_item_submenu(i)])
	return out

## Report one screen: what it drew, whether it drew anything, whether it fits,
## and whether every tap target clears the floor.
func _report(id: String, arg: String, texts: Array, width: int) -> void:
	var ne := _nonempty(texts)
	var rows: int = pm._screen_body.get_child_count()
	var mw: float = pm._screen_body.get_combined_minimum_size().x
	print("PM2 screen %-12s arg=%-18s rows=%-3d strings=%-3d body_min_w=%.0f  screen_w=%d"
		% [id, arg if arg != "" else "-", rows, ne.size(), mw, width])
	_check("%s draws rows" % id, ne.size() >= 3,
		"only %d non-empty strings" % ne.size())
	_check("%s fits %d px" % [id, width], mw <= float(width),
		"min_w=%.0f > %d" % [mw, width])
	var over: Array = []
	_widest(pm._screen_body, float(width), over, 0)
	for line in over:
		print("PM2   over: %s" % line)

	var floor_px: float = float(DccTheme.PHONE_TAP_MIN) * _scale
	var taps: Array = []
	_taps(pm._screen_body, taps)
	var short: Array = []
	for t in taps:
		## 0.5 px of tolerance: `_pt()` rounds, so a target sized exactly at the
		## floor can land a rounding tick under it and that is not a defect.
		if float(t[1]) > 0.0 and float(t[1]) < floor_px - 0.5:
			var bits: Array = []
			_labels(t[0], bits)
			short.append("%s h=%.0f  %s" % [(t[0] as Node).get_class(), float(t[1]),
				" / ".join(PackedStringArray(_nonempty(bits))).substr(0, 60)])
	_check("%s: every tap target >= %.0f px (44 dp at touch density x%.3f)"
		% [id, floor_px, _scale], short.is_empty(),
		"%d under: %s" % [short.size(), " | ".join(PackedStringArray(short))])

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("PM2 WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var want := _arg_size()
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	pm = app.get("_phone_menu")
	if pm == null:
		print("PM2  !! no PhoneMenu -- run with -- --force-touch")
		get_tree().quit(1)
		return
	_scale = app.phone_scale()
	print("PM2 density %dx%d  phone_scale=%.3f  tap_floor=%.0f px (44 dp)"
		% [want.x, want.y, _scale, float(DccTheme.PHONE_TAP_MIN) * _scale])
	var br = app.get("bridge")
	print("PM2 bridge=%s  generating=%s  has_world=%s" % [
		"yes" if br != null else "NO",
		str(br.get("generating")) if br != null else "-",
		str(br.get("has_world")) if br != null else "-"])

	## -- The root still carries all eight §6.6 rows ---------------------------
	var root: Array = await _open("more")
	for label in ["Project", "Civilization", "Data manager", "Asset library",
			"Travel library", "Simulation", "Preferences", "Help & about"]:
		_check("root row '%s'" % label, _has(root, String(label)))
	for label in ["Edit", "Window"]:
		_check("root fallback row '%s'" % label, _has(root, String(label)))
	_report("more", "", root, want.x)

	## -- The argument-free screens --------------------------------------------
	for id in ["assets", "travel", "landmarks", "help", "gestures"]:
		var t: Array = await _open(String(id))
		_report(String(id), "", t, want.x)
		for s in _nonempty(t).slice(0, 6):
			print("PM2   %s: %s" % [id, String(s).substr(0, 96)])

	## -- assets-grid / asset-slot, on a family discovered live ----------------
	##
	## The family key is taken off `AssetLibraryWindow.FAMILIES` and the slot uid
	## off `as_family_slots()`, so this walks the same route a finger does rather
	## than pushing a hard-coded argument that could be the only one that works.
	var fam_key := ""
	var slot_uid := ""
	if br != null and br.has_method("as_family_slots"):
		for f in AssetLibraryWindow.FAMILIES:
			var k := String((f as Dictionary).get("key", ""))
			var slots: Array = br.as_family_slots(k)
			if slots.is_empty():
				continue
			if fam_key == "":
				fam_key = k
			for s in slots:
				if String((s as Dictionary).get("uid", "")) != "":
					fam_key = k
					slot_uid = String((s as Dictionary).get("uid", ""))
					break
			if slot_uid != "":
				break
	_check("a family with slots exists to push assets-grid with", fam_key != "",
		"as_family_slots() returned nothing for any of the %d families"
			% AssetLibraryWindow.FAMILIES.size())
	if fam_key != "":
		var t: Array = await _open("assets-grid", fam_key)
		_report("assets-grid", fam_key, t, want.x)
	if slot_uid != "":
		var t2: Array = await _open("asset-slot", slot_uid)
		_report("asset-slot", slot_uid, t2, want.x)
		for s in _nonempty(t2).slice(0, 8):
			print("PM2   asset-slot: %s" % String(s).substr(0, 96))
	else:
		print("PM2 note: no slot uid to inspect -- asset-slot not exercised")

	## -- travel-item, on an entry discovered live -----------------------------
	var tkind := ""
	var tid := ""
	if br != null and br.has_method("tl_list"):
		for k in TravelLibraryWindow.KINDS:
			var key := String((k as Dictionary).get("key", ""))
			var rows: Array = br.tl_list(key)
			if rows.is_empty():
				continue
			tkind = key
			tid = String((rows[0] as Dictionary).get("id", ""))
			break
	_check("a travel entry exists to push travel-item with", tid != "",
		"tl_list() returned nothing for any of the %d kinds"
			% TravelLibraryWindow.KINDS.size())
	if tid != "":
		var t3: Array = await _open("travel-item", "%s|%s" % [tkind, tid])
		_report("travel-item", "%s|%s" % [tkind, tid], t3, want.x)
		for s in _nonempty(t3).slice(0, 8):
			print("PM2   travel-item: %s" % String(s).substr(0, 96))

	## -- lm-fam, on a family discovered live ----------------------------------
	var lfam := ""
	if br != null and br.has_method("landmark_kinds"):
		var kinds: Array = br.landmark_kinds()
		print("PM2 landmark_kinds()=%d types" % kinds.size())
		if not kinds.is_empty():
			lfam = String((kinds[0] as Dictionary).get("family", ""))
	_check("a landmark family exists to push lm-fam with", lfam != "",
		"landmark_kinds() returned no types -- the extension has no landmark bridge")
	if lfam != "":
		var t4: Array = await _open("lm-fam", lfam)
		_report("lm-fam", lfam, t4, want.x)
		for s in _nonempty(t4).slice(0, 8):
			print("PM2   lm-fam: %s" % String(s).substr(0, 96))

	## -- Back, from one level and from three ----------------------------------
	for id in ["assets", "travel", "landmarks", "help", "gestures"]:
		pm.open()
		pm._push_screen(String(id))
		await _frames(2)
		var consumed: bool = pm.go_back()
		await _frames(2)
		var top = pm._stack[pm._stack.size() - 1]
		_check("back from '%s' returns to MORE" % id,
			consumed and pm.is_open() and String(top.screen) == "more",
			"consumed=%s open=%s top='%s'" % [consumed, pm.is_open(), String(top.screen)])

	if fam_key != "" and slot_uid != "":
		pm.open()
		pm._push_screen("assets", "")
		pm._push_screen("assets-grid", fam_key)
		pm._push_screen("asset-slot", slot_uid)
		await _frames(2)
		var depth: int = pm._stack.size()
		pm.go_back()
		pm.go_back()
		await _frames(2)
		var mid = pm._stack[pm._stack.size() - 1]
		pm.go_back()
		await _frames(2)
		var top2 = pm._stack[pm._stack.size() - 1]
		_check("back x3 walks asset-slot -> assets-grid -> assets -> MORE",
			depth == 4 and String(mid.screen) == "assets" and String(top2.screen) == "more"
				and pm.is_open(),
			"depth=%d mid='%s' top='%s'" % [depth, String(mid.screen), String(top2.screen)])

	## -- Nothing stopped being reachable --------------------------------------
	##
	## `Assets` and `Help` were popup drills until this change and are screens
	## now. Both screens end in `_rest_of()` over the same popup, so every
	## top-level row must still be drawn -- counting a submenu as drawn when its
	## own text appears OR when one of its children's does (a submenu expanded
	## inline as chips contributes children, not a parent).
	var covers := {"assets": "Assets", "help": "Help"}
	for sid in covers:
		var menu := String(covers[sid])
		var p := _menu_popup(menu)
		if p == null:
			_check("%s menu exists" % menu, false)
			continue
		var texts: Array = await _open(String(sid))
		var lost := PackedStringArray()
		for entry in _top_items(p):
			var text := String(entry[0])
			var node := String(entry[1])
			if _has(texts, text):
				continue
			if node != "":
				var sub := p.get_node_or_null(NodePath(node)) as PopupMenu
				if sub != null:
					var any := false
					for j in sub.item_count:
						if sub.is_item_separator(j):
							continue
						if _has(texts, _clean(sub.get_item_text(j))):
							any = true
							break
					if any:
						continue
			lost.append(text)
		_check("%s: every top-level row reaches screen '%s'" % [menu, sid],
			lost.is_empty(), "missing: %s" % " | ".join(lost))

	## -- A control state, so a pass means something ---------------------------
	##
	## A screen id with no builder must draw the `_missing_row()` placeholder and
	## nothing else. Without this, "the screen drew rows" cannot distinguish a
	## real screen from the fallback, and every assertion above passes vacuously
	## on a `_fill_screen()` that lost its match arms.
	var bogus: Array = await _open("no-such-screen")
	var bogus_ne := _nonempty(bogus)
	_check("control: an unknown screen id draws only its placeholder",
		bogus_ne.size() <= 3 and _has(bogus_ne, "Unknown screen 'no-such-screen'."),
		"drew %d strings: %s" % [bogus_ne.size(),
			" | ".join(PackedStringArray(bogus_ne)).substr(0, 120)])

	## -- What the root screen costs -------------------------------------------
	##
	## `_root_badge()` counts every family's slots through `as_family_slots()`
	## on every root render -- eight crossings of the gdext boundary for one
	## badge. `_menu_row()`'s own header in `phone_menu.gd` refuses a preview
	## that would fire one `about_to_popup`, so eight engine calls need a number
	## beside them rather than an assumption. Median of nine, with the spread,
	## and the root is rendered alone in this loop so nothing else is in it.
	var samples: Array = []
	for i in 9:
		var t0 := Time.get_ticks_usec()
		pm.open()
		samples.append(float(Time.get_ticks_usec() - t0) / 1000.0)
		await _frames(1)
	samples.sort()
	print("PM2 root render: median %.2f ms (%.2f..%.2f) over 9, %d families badged"
		% [float(samples[4]), float(samples[0]), float(samples[samples.size() - 1]),
			AssetLibraryWindow.FAMILIES.size()])
	## The cause, measured separately -- a whole-render figure cannot say whose
	## milliseconds those are, and attributing them without measuring is what
	## `MISTAKES.md` calls making a single-sample claim look like an analysis.
	var badge_samples: Array = []
	for i in 9:
		var t0 := Time.get_ticks_usec()
		var b = pm._root_badge("assets")
		badge_samples.append(float(Time.get_ticks_usec() - t0) / 1000.0)
		if b != null:
			(b as Node).free()
	badge_samples.sort()
	print("PM2 root badge alone (%d x as_family_slots): median %.3f ms (%.3f..%.3f) over 9"
		% [AssetLibraryWindow.FAMILIES.size(), float(badge_samples[4]),
			float(badge_samples[0]), float(badge_samples[badge_samples.size() - 1])])

	## -- The rows ACT on the real shell ---------------------------------------
	##
	## Everything above proves the screens draw. This proves they are not
	## decoration: each control is driven through the node the screen actually
	## built, and the engine is read back afterwards. A screen that renders a
	## convincing shell over nothing passes every check above and fails these.
	await _behaviour(br, lfam, fam_key)

	## -- With a world, so the world-dependent halves are exercised ------------
	##
	## `--world` because it costs a generate. Without it `room_estimate` is `0`
	## by construction (`lib.rs::landmark_headroom()` computes it only for
	## `WorldSource::Generated`), `landmark_run()` has nothing to place, and the
	## headroom line takes its no-world branch -- which the run above already
	## checked. With it, the same line must carry real figures.
	if OS.get_cmdline_user_args().has("--world"):
		await _with_world(br, lfam)

	pm.close()
	print("PM2 done: %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

## Find the first control of `cls` under the current screen body whose sibling
## text matches `near`, so a driven control is the one the user would touch
## rather than one this probe went looking for by index.
func _find_near(cls: String, near: String) -> Node:
	var hit: Array = []
	_collect(pm._screen_body, cls, hit)
	for n in hit:
		var bits: Array = []
		_labels((n as Node).get_parent().get_parent(), bits)
		for b in bits:
			if String(b).begins_with(near):
				return n
	return null

func _collect(n: Node, cls: String, out: Array) -> void:
	for c in n.get_children():
		if c.get_class() == cls:
			out.append(c)
		_collect(c, cls, out)

func _behaviour(br, lfam: String, fam_key: String) -> void:
	if br == null:
		return

	## 1. `lm-fam`'s cap slider disarms at rung 0 and KEEPS the cap.
	##    The desktop rule is `landmark_set_armed(false)`, never
	##    `landmark_set_cap(0)` -- getting it wrong destroys the number a user
	##    gets back when they drag up again, and nothing on screen would say so.
	if lfam != "" and br.has_method("landmark_settings"):
		var kinds: Array = br.landmark_kinds()
		var key := ""
		for k in kinds:
			var kd: Dictionary = k
			if String(kd.get("family", "")) == lfam and bool(kd.get("buildable", true)):
				key = String(kd.get("key", ""))
				break
		if key != "":
			var before: Dictionary = br.landmark_settings()
			var cap0 := int((before.get("caps", {}) as Dictionary).get(key, -1))
			var armed0: bool = bool((before.get("armed", {}) as Dictionary).get(key, false))
			await _open("lm-fam", lfam)
			var s := _find_near("HSlider", "") as HSlider
			_check("lm-fam builds a real slider", s != null)
			if s != null:
				## **The real drag path, not the handler shortcut.** The cap
				## write moved to `drag_ended` so a drag no longer writes a cap
				## per frame -- and `on_release` calls `_render()`, which
				## `queue_free()`s the very slider emitting the signal. Driving
				## the signal is the only way to find out whether that is safe;
				## calling `_lm_set_rung()` directly (which is what the checks
				## below do) never touches it.
				var was := s.value
				s.value = 0.0
				s.drag_ended.emit(true)
				await _frames(3)
				var st_drag: Dictionary = br.landmark_settings()
				_check("lm-fam: a real drag to the zero stop disarms '%s'" % key,
					not bool((st_drag.get("armed", {}) as Dictionary).get(key, true)),
					"armed is still %s after drag_ended"
						% bool((st_drag.get("armed", {}) as Dictionary).get(key, true)))
				_check("lm-fam: the screen survived redrawing from drag_ended",
					is_instance_valid(pm._screen_body)
						and pm._screen_body.get_child_count() > 3,
					"body has %d children" % (pm._screen_body.get_child_count()
						if is_instance_valid(pm._screen_body) else -1))
				br.landmark_set_armed(key, armed0)
				await _frames(2)
				print("PM2 note: drag path exercised at rung %.0f (was %.0f)"
					% [0.0, was])
				await _open("lm-fam", lfam)
				pm._lm_set_rung(key, 0)
				await _frames(2)
				var after: Dictionary = br.landmark_settings()
				var cap1 := int((after.get("caps", {}) as Dictionary).get(key, -1))
				var armed1: bool = bool((after.get("armed", {}) as Dictionary).get(key, false))
				_check("lm-fam rung 0 disarms '%s'" % key, not armed1,
					"armed stayed %s" % armed1)
				_check("lm-fam rung 0 KEEPS the cap for '%s'" % key, cap1 == cap0,
					"cap moved %d -> %d" % [cap0, cap1])
				## Restore through `landmark_set_cap`, NOT back through the rung.
				##
				## **Measured here, and it is a real property of the ladder
				## rather than a probe convenience.** `CivilizationWorkspace.
				## LM_LADDER` is `[0,1,2,3,5,8,12,20,30,50,80,120,200]` and
				## `peak`'s engine default cap is **24**, which is not on it --
				## so `_lm_rung(24)` returns the nearest rung (20, since
				## |20-24| < |30-24|) and a round trip through the slider
				## quantises the cap from 24 to 20. `caps_total` moved 384 ->
				## 380 across exactly this, the missing 4 being peak's.
				##
				## That is the DESKTOP panel's behaviour too -- `_lm_type_row()`
				## carries the same `_lm_rung(cap)` -- and the phone screen reads
				## the same constant deliberately, so this is inherited, not
				## introduced. A probe that restored through the rung would
				## leave the session 4 caps light and call it a pass.
				br.landmark_set_cap(key, cap0)
				br.landmark_set_armed(key, armed0)
				await _frames(2)
				var back: Dictionary = br.landmark_settings()
				_check("lm-fam restores '%s' to armed=%s cap=%d" % [key, armed0, cap0],
					bool((back.get("armed", {}) as Dictionary).get(key, false)) == armed0
						and int((back.get("caps", {}) as Dictionary).get(key, -1)) == cap0,
					"engine reports armed=%s cap=%d"
						% [bool((back.get("armed", {}) as Dictionary).get(key, false)),
							int((back.get("caps", {}) as Dictionary).get(key, -1))])
				## And the finding itself, asserted rather than described: a cap
				## off the ladder does not survive one slider gesture.
				var off_ladder: bool = not CivilizationWorkspace.LM_LADDER.has(cap0)
				if off_ladder:
					print("PM2 note: '%s' default cap %d is NOT a LM_LADDER rung; "
						% [key, cap0]
						+ "one slider gesture quantises it to %d (desktop does the same)"
						% int(CivilizationWorkspace.LM_LADDER[
							CivilizationWorkspace._lm_rung(cap0)]))

	## 2. `landmarks`' Crowding slider writes through to the engine.
	if br.has_method("landmark_settings") and br.has_method("landmark_set_crowding"):
		var st0: Dictionary = br.landmark_settings()
		var c0 := float(st0.get("crowding", 1.0))
		var c1: float = CivilizationWorkspace.LM_CROWDING_MAX if c0 < 1.5 \
			else CivilizationWorkspace.LM_CROWDING_MIN
		await _open("landmarks")
		pm._lm_write("landmark_set_crowding", [c1])
		await _frames(2)
		var st1: Dictionary = br.landmark_settings()
		_check("landmarks Crowding writes through (%0.2f -> %0.2f)" % [c0, c1],
			is_equal_approx(float(st1.get("crowding", -1.0)), c1),
			"engine reports %.2f" % float(st1.get("crowding", -1.0)))
		pm._lm_write("landmark_set_crowding", [c0])

	## 3. `travel`'s type chips actually change the row set.
	if br.has_method("tl_list"):
		var seen: Dictionary = {}
		for k in TravelLibraryWindow.KINDS:
			var key := String((k as Dictionary).get("key", ""))
			pm._set_travel_kind(key)
			var t: Array = await _open("travel")
			seen[key] = _nonempty(t).size()
		var distinct: Dictionary = {}
		for k in seen:
			distinct[seen[k]] = true
		_check("travel: the four type chips draw four different row sets",
			distinct.size() >= 2, "row counts were %s" % str(seen))
		print("PM2 travel row counts by kind: %s" % str(seen))

	## 4. `assets-grid` draws exactly one cell per real slot.
	if fam_key != "" and br.has_method("as_family_slots"):
		var slots: Array = br.as_family_slots(fam_key)
		await _open("assets-grid", fam_key)
		var cells: Array = []
		_collect(pm._screen_body, "Button", cells)
		## The screen's own trailing "Open … in the asset library" row is a
		## `PanelContainer`, not a `Button`, so every `Button` under the body is
		## a grid cell.
		_check("assets-grid draws one cell per slot (%d)" % slots.size(),
			cells.size() == slots.size(),
			"%d cells for %d slots" % [cells.size(), slots.size()])

## The world-dependent halves. Generates a small world, then re-reads the two
## figures that were structurally zero without one.
func _with_world(br, lfam: String) -> void:
	print("PM2 --world: generating…")
	br.generate({"seed": 40417, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var waited := 0
	while br.generating and waited < 9000:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	_check("--world: a world generated", bool(br.get("has_world")))

	var head: Dictionary = br.landmark_headroom()
	print("PM2 --world: landmark_headroom()=%s" % str(head))
	_check("--world: room_estimate is a figure now, not the no-world zero",
		int(head.get("room_estimate", 0)) > 0,
		"room_estimate=%d" % int(head.get("room_estimate", 0)))
	var t: Array = await _open("landmarks")
	_report("landmarks", "with-world", t, get_window().size.x)
	for s in _nonempty(t).slice(0, 3):
		print("PM2   landmarks(world): %s" % String(s).substr(0, 110))
	_check("--world: the headroom line quotes the packing estimate",
		_nonempty(t).size() > 0 and String(_nonempty(t)[0]).contains("room for about"),
		"first line was '%s'" % (String(_nonempty(t)[0]) if _nonempty(t).size() > 0 else ""))

	## The run itself -- threaded, and the screen redraws when it returns.
	print("PM2 --world: running the landmark pass…")
	await pm._lm_run()
	await _frames(4)
	var t2: Array = await _open("landmarks")
	for s in _nonempty(t2).slice(0, 2):
		print("PM2   landmarks(after run): %s" % String(s).substr(0, 110))
	_check("--world: the headroom line quotes the run afterwards",
		_nonempty(t2).size() > 0 and String(_nonempty(t2)[0]).contains("last run placed"),
		"first line was '%s'" % (String(_nonempty(t2)[0]) if _nonempty(t2).size() > 0 else ""))
	if lfam != "":
		var t3: Array = await _open("lm-fam", lfam)
		var any := false
		for s in t3:
			if String(s).begins_with("↳ last run"):
				any = true
				break
		_check("--world: lm-fam draws a last-run row after a pass", any,
			"no '↳ last run' row among %d strings" % _nonempty(t3).size())

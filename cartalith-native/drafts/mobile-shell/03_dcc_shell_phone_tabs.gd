## DRAFT, not loaded by Godot. MS-2: the phone's bottom bar `MAP · <domain> · STORY · MENU`, and the domain
## tab's tap, hold and swipe (`MOBILE_SHELL_PROPOSAL.md` §2.1, §4; Rulings DT Q1 to Q3, DU). Target:
## `shell/dcc_shell.gd`. It replaces `PHONE_TABS`, `_pick_phone_tab()`, `_phone_tab_for_domain()` and
## `_phone_tab_drives_sheet()`, adds the domain tab's gesture handling, and touches the bar builder and
## `_refresh_phone_tabs()` at the two places named below.
##
## Before pasting, re-read every `for t in PHONE_TABS` in the shell (six at `5bd7af8`, found with
## `grep -n "for t in PHONE_TABS" shell/dcc_shell.gd`): they read `t.domain`, which these rows no longer carry.
## The documents that name the old tabs `MAP · GENERATE · PLAN · MORE` are corrected in the same change
## (Ruling DT: `ANDROID_UI_SPEC.md` §0.3, `DESIGN_HANDOFF.md`).
##
## **So is one string the app draws** (found 2026-11-01 by the menu parity check,
## `cartalith-native/docs/evidence/menu-parity-2026-11-01/`, with
## `grep -rn "GENERATE · PLAN" --include=*.gd cartalith-native/godot-project`):
##   H2       `shell/dcc_shell.gd`'s `_COACH_MARKS` row `bottombar_tabs` ("MAP · GENERATE · PLAN switch tasks here
##            — MORE reaches everything else."). The mockup has no coach marks, so its new wording is MS-2's to
##            write. `_coachcoldboot_probe.gd` and `_pandraghint_probe.gd` pin the old text (`BOTTOMBAR_TEXT`).
##   H1       (withdrawn the same day, Ruling DX Q13) was `phone_menu.gd::_fill_help()`'s closing line. MENU ▸ Help
##            now opens the Help popup row for row (draft 05), so that screen and its line are no longer drawn;
##            the mockup carries the sentence in its About window ("…MAP · DOMAIN · STORY · MENU.").
## The same grep finds the words in five comments (`phone_menu.gd` twice, both in code draft 05 leaves unreachable;
## `dcc_icons.gd`; `dcc_shell.gd`'s `PHONE_TABS` note; `_phonemore_reach_probe.gd`'s header). Correct those too.
##
## NOT parse-checked (no Godot binary in the session that wrote it). It depends on draft 04
## (`shell/phone_domain_fan.gd`) and on draft 05's `PhoneMenu.open_story()`.

## The four tabs (Ruling DT Q1 and Q2). The domain tab's caption and glyph follow the current domain
## (`DOMAIN_TAB`), so its row carries neither.
const PHONE_TABS: Array = [
	{"id": "map", "caption": "MAP", "glyph": "▤", "tip": "View · fields · layers"},
	{"id": "domain", "caption": "", "glyph": "",
		"tip": "The current domain. Hold, or swipe sideways, to switch WORLD · CIVIL · CARTO"},
	{"id": "story", "caption": "STORY", "glyph": "➔", "tip": "Time · Journeys · Conflicts · Notes"},
	{"id": "menu", "caption": "MENU", "glyph": "☰", "tip": "The eight desktop menus, Quick and Status"},
]
## What the domain tab says for each domain (Ruling DT Q3: GENERATE while WORLD is the domain). `fan` is
## the caption in the domain fan and the sheet title. The glyphs are ones the shell already draws (`⌗` the
## old GENERATE tab, `◍` the old Civilization row, `▦` the asset library row), so none is new to the font.
const DOMAIN_TAB: Dictionary = {
	"world": {"caption": "GENERATE", "glyph": "⌗", "fan": "WORLD · GENERATE"},
	"civilization": {"caption": "CIVIL", "glyph": "◍", "fan": "CIVIL"},
	"cartography": {"caption": "CARTO", "glyph": "▦", "fan": "CARTO"},
}
## Swipe order: left goes to the next, right to the previous. The ids are `DOMAINS`' own.
const DOMAIN_ORDER: Array = ["world", "civilization", "cartography"]
## The hold that opens the domain fan. It is the mockup's `NAV_HOLD_MS` (`design/mobile-shell-2026-10-31`), a
## design figure, revisited on glass in MS-10. The tool fan's own hold is 260 ms (`ToolFan.FAN_HOLD_MS`).
const DOMAIN_HOLD_MS := 380
## Sideways travel that counts as a swipe, in phone units (scaled by `_pscale`). A labelled judgement: past
## the 12-unit tap slop the tool fan uses, short of anything a sheet drag would need.
const DOMAIN_SWIPE_UNITS := 48.0

## Draft 04, loaded by path so no new `class_name` (and no import pass) is needed.
const _PhoneDomainFan := preload("res://shell/phone_domain_fan.gd")
var _domain_fan: Control
## The domain tab's gesture: `{g0, token, fan, used}` while a press is down, `{}` otherwise. `used` means the
## press became a swipe or a fan pick, so the Button's own `pressed` (a release inside it) must be skipped.
var _dom_gesture: Dictionary = {}
var _dom_token := 0

## Builds one bar cell. The bar builder's loop calls this instead of `_phone_bar_cell()` directly:
##     for t in PHONE_TABS:
##         var cell := _build_phone_tab_cell(t)
##         var key := String(t.id)
##         _phone_tab_cells[key] = cell
##         (cells if key == "menu" else dests).add_child(cell["button"] as Control)
## MENU takes the place MORE had (`cells`, which `Window ▸ Domain rail` never hides). The domain cell is
## no longer registered in `_domain_buttons` / `_domain_marks`: one cell stands for three domains, so it is
## lit and captioned by `_refresh_phone_tabs()` instead (see the addition below).
func _build_phone_tab_cell(t: Dictionary) -> Dictionary:
	var id := String(t.id)
	if id != "domain":
		return _phone_bar_cell(String(t.caption), String(t.glyph), String(t.tip),
			_pick_phone_tab.bind(id))
	var dt: Dictionary = DOMAIN_TAB.get(_active_domain, DOMAIN_TAB["world"])
	var cell := _phone_bar_cell(String(dt["caption"]), String(dt["glyph"]), String(t.tip),
		_on_domain_cell_pressed)
	var b := cell["button"] as Button
	b.gui_input.connect(_on_domain_cell_input.bind(b))
	return cell

## Every tab. `toggle` is false when a domain switch re-picks the domain tab: the sheet then stays open
## (or opens to half from peek) instead of folding to peek as a second tap on the same tab does.
func _pick_phone_tab(id: String, toggle: bool = true) -> void:
	if id == "domain" and _session_refuses(_active_domain):
		return
	var was_tab := _phone_tab
	_phone_tab = id
	match id:
		"menu":
			_toggle_overflow()
		"story":
			if _phone_menu != null:
				_phone_menu.open_story()
		"map":
			if _phone_menu != null and _phone_menu.is_open():
				_phone_menu.close()
			## MS-3 builds the MAP sheet: view, fields and layers (proposal §5). Until it lands, MAP keeps
			## the shipped behaviour of selecting CARTOGRAPHY, which the old `map` row did.
			_pick_bar_domain("cartography")
		"domain":
			if _phone_menu != null and _phone_menu.is_open():
				_phone_menu.close()
			_pick_bar_domain(_active_domain)
	if _phone_tab_drives_sheet(id):
		if toggle and was_tab == id and _phone_detent != "peek":
			_set_phone_detent("peek")
		elif _phone_detent == "peek":
			_set_phone_detent("half")
	_sync_phone_quick_row()
	_refresh_phone_tabs()
	_refresh_phone_sheet_header()
	_refresh_phone_gen_panel()

## Every domain lives on the one domain tab.
func _phone_tab_for_domain(_id: String) -> String:
	return "domain"

## MAP and the domain tab drive the bottom sheet; STORY and MENU open PhoneMenu over it.
func _phone_tab_drives_sheet(id: String) -> bool:
	return id == "map" or id == "domain"

## Add to the end of `_refresh_phone_tabs()`: the domain cell's caption and glyph follow `_active_domain`.
## Its lit state comes from the existing loop, because its key is "domain" and `_phone_tab` is "domain"
## whenever a domain sheet is up.
func _refresh_domain_tab_caption() -> void:
	var dcell: Dictionary = _phone_tab_cells.get("domain", {})
	if dcell.is_empty():
		return
	var dt: Dictionary = DOMAIN_TAB.get(_active_domain, DOMAIN_TAB["world"])
	var lbl := dcell.get("label") as Label
	var ic := dcell.get("icon") as Label
	if lbl != null and is_instance_valid(lbl):
		lbl.text = String(dt["caption"])
	if ic != null and is_instance_valid(ic):
		ic.text = String(dt["glyph"])

## A plain tap on the domain tab. Skipped when the same press already became a swipe or a fan pick: the
## Button emits `pressed` on a release inside it, after `gui_input` has seen that release.
func _on_domain_cell_pressed() -> void:
	var skip := bool(_dom_gesture.get("used", false))
	_dom_gesture.clear()
	if skip:
		return
	_pick_phone_tab("domain")

## The domain tab's gestures, from the same mouse and touch events the tool fan's pill reads
## (`tool_fan.gd`), converted to canvas coordinates the same way. With mouse emulation on (the project's
## default), one finger arrives twice, so a second press while a gesture is live is ignored.
func _on_domain_cell_input(event: InputEvent, b: Button) -> void:
	var pos := Vector2.ZERO
	var kind := ""
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index != MOUSE_BUTTON_LEFT:
			return
		pos = mb.position
		kind = "press" if mb.pressed else "release"
	elif event is InputEventScreenTouch:
		var st := event as InputEventScreenTouch
		pos = st.position
		kind = "press" if st.pressed else "release"
	elif event is InputEventMouseMotion:
		pos = (event as InputEventMouseMotion).position
		kind = "move"
	elif event is InputEventScreenDrag:
		pos = (event as InputEventScreenDrag).position
		kind = "move"
	else:
		return
	var g: Vector2 = b.get_global_transform_with_canvas() * pos
	match kind:
		"press":
			if not _dom_gesture.is_empty():
				return
			_dom_token += 1
			_dom_gesture = {"g0": g, "token": _dom_token, "fan": false, "used": false}
			get_tree().create_timer(DOMAIN_HOLD_MS / 1000.0).timeout.connect(
				_on_domain_hold.bind(_dom_token, b))
		"move":
			if _dom_gesture.is_empty():
				return
			if bool(_dom_gesture["fan"]):
				_domain_fan.call("pointer", g)
			elif not bool(_dom_gesture["used"]):
				var dx: float = g.x - (_dom_gesture["g0"] as Vector2).x
				if absf(dx) >= float(_pscale(DOMAIN_SWIPE_UNITS)):
					_dom_gesture["used"] = true
					_cycle_domain(1 if dx < 0.0 else -1)
		"release":
			if _dom_gesture.is_empty():
				return
			if bool(_dom_gesture["fan"]):
				_dom_gesture["used"] = true
				var picked := String(_domain_fan.call("release_at", g))
				if picked != "":
					_switch_domain_from_tab(picked)
			if bool(_dom_gesture["used"]):
				## The Button may or may not emit `pressed` (only when the release is inside it), so the
				## gesture is cleared after this frame's input either way.
				_clear_dom_gesture.call_deferred(int(_dom_gesture["token"]))
			else:
				_dom_gesture.clear()

## Clears the gesture left by a swipe or a fan pick, unless a newer press has started since.
func _clear_dom_gesture(token: int) -> void:
	if not _dom_gesture.is_empty() and int(_dom_gesture["token"]) == token:
		_dom_gesture.clear()

## The hold timer. It only acts on the press that started it, and only if that press is still down and has
## not become a swipe.
func _on_domain_hold(token: int, b: Button) -> void:
	if _dom_gesture.is_empty() or int(_dom_gesture["token"]) != token or bool(_dom_gesture["used"]):
		return
	if _domain_fan == null:
		_domain_fan = _PhoneDomainFan.new()
		_domain_fan.call("setup", self)
		_phone_root.add_child(_domain_fan)
	_dom_gesture["fan"] = true
	_haptic("sample")
	var entries: Array = []
	for id in DOMAIN_ORDER:
		var dt: Dictionary = DOMAIN_TAB[id]
		entries.append({"id": id, "caption": String(dt["fan"]), "glyph": String(dt["glyph"]),
			"refused": _session_refuses(String(id))})
	_domain_fan.call("open_above", b.get_global_rect(), _active_domain, entries)

## Swipe: the next or previous domain in `DOMAIN_ORDER`.
func _cycle_domain(step: int) -> void:
	var i := DOMAIN_ORDER.find(_active_domain)
	if i < 0:
		i = 0
	_switch_domain_from_tab(String(DOMAIN_ORDER[posmod(i + step, DOMAIN_ORDER.size())]))

## The one way the domain tab changes the domain: through `_pick_bar_domain()`, as the old GENERATE and MAP
## tabs did, then re-picking the domain tab without the second-tap fold. Session mode's refusal is honoured.
func _switch_domain_from_tab(id: String) -> void:
	if _session_refuses(id):
		return
	_haptic("tool_arm")
	_pick_bar_domain(id)
	_pick_phone_tab("domain", false)

## Add to the phone's system-back chain, before the sheet levels: an open domain fan closes first
## (proposal §4's order: window, fan, quick drawer, cartwheel, …).
func _close_domain_fan_for_back() -> bool:
	if _domain_fan != null and bool(_domain_fan.call("is_open")):
		_domain_fan.call("close")
		return true
	return false

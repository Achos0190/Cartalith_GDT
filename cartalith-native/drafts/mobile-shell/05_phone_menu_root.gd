## DRAFT, not loaded by Godot. MS-4 (MENU) and MS-6 (STORY).
## - MENU (Ruling DT Q2, and Ruling DX Q13 "the phone's MENU follows the mockup"): the eight desktop menus, row
##   for row, in the menu bar's order and under its names, with Quick first and Status last. Each root row opens
##   its menu's own popup, drawn row for row by `_fill_popup()`, so MENU shows exactly the rows the desktop builds.
## - STORY (Ruling DT Q1): the four bands TIME · JOURNEYS · CONFLICTS & CAMPAIGNS · NOTES, opened from the
##   STORY tab.
## Target: `shell/phone_menu.gd`. It replaces `_fill_root()`, `_fill_story_section()` and `_fill_popup()`, adds
## `_menu_root_row()`, `open_story()` and `open_screen()`, adds the `story-root` screen, and retires `ROOT_ROWS`,
## `ROOT_REST`, `_named_in_root()` and the "Not on the MORE list" band.
##
## **Rewritten 2026-11-01 for Ruling DX Q13.** The first version opened bespoke phone screens for File, Assets,
## World, Story, Preferences and Help (`_fill_project()`, `_fill_assets()`, a new `_fill_world()` with an "On the
## phone" band, `_fill_prefs()`, `_fill_help()`). They reordered the rows, added phone-only rows and titled
## themselves Project, Asset library and Help & about. The menu parity check measured 64 such rows against the
## mockup, which draws the menus row for row; the owner chose the mockup. Every phone-only row those screens
## carried has a stated home or a stated drop in this folder's README ("Where each phone-only row goes"). The one
## that needed a new row, Gesture reference, gets it in Help (draft 09, the mockup's own row).
##
## **Kept from the first version** (each measured by the same check): `_fill_story_section()` draws the Story
## screen's four band rows without the old root's stray "Story" and "World" heads; `_fill_popup()` keeps a leading
## captioned separator as a band, so File ▸ Export keeps FOR THE TABLE, World ▸ Sources & validation SOURCES and
## Assets ▸ Asset pack ACTIVE PACK.
##
## **What this leaves unreachable** (`MISTAKES.md`, "Delete a surface"). After this draft no row pushes the screens
## `project`, `civ`, `data`, `data-tiles`, `sim`, `prefs`, `assets`, `assets-grid`, `asset-slot`, `travel`,
## `travel-item`, `landmarks`, `lm-fam` or `help`: their only doors were the retired root and each other
## (`grep -n '_push_screen("' shell/phone_menu.gd`). `gestures` keeps a door (Help ▸ Gesture reference…, draft 09),
## and `story` keeps the STORY tab's. Delete the unreachable builders, their `SCREEN_TITLES` entries and their
## helpers in the same change, with the probes the README lists, or the next reader maintains dead screens.
##
## NOT parse-checked (no Godot binary in the session that wrote it).

## The subtitle under each menu's root row: the mockup's own, `menuTree()`'s `sub` for that menu
## (`design/mobile-shell-2026-10-31/Shell.dc.html`). A label, not a list: the rows below it are the menu's own.
const MENU_SUBS: Dictionary = {
	"File": "new · open · save · export · storage",
	"Edit": "undo · clipboard · find · reset",
	"Assets": "library · packs · families · landmark types",
	"World": "data manager · records · technical",
	"Story": "time · journeys · conflicts · notes",
	"Preferences": "performance · graphics · LOD · memory · app",
	"Window": "regions · session · 3D · workspace · layouts",
	"Help": "documentation · shortcuts · credits · about",
}
## Lead glyphs. `_glyph()` already falls back to the submenu arrow for a glyph the mono font lacks.
const MENU_GLYPHS: Dictionary = {
	"File": "⧉", "Edit": "✎", "Assets": "▦", "World": "◍", "Story": "➔",
	"Preferences": "⚙", "Window": "▢", "Help": "?",
}

## Add this to `SCREEN_TITLES`, and rename its `more` entry for MENU (Ruling DT Q2; the key stays `more`, so
## `open()` and every probe that pushes it keep working):
##     "more": ["Menu", "the desktop menus · quick · status"],
##     "story-root": ["Story", "time · journeys · conflicts · notes"],
## and this arm to `_fill_screen()`'s match:
##     "story-root": _fill_story_section(body)

## MENU's root: Quick, the MENUS band, the eight menus in the menu bar's order, then Status. Each menu row opens
## that menu's popup row for row (`_menu_root_row()`), the way `rowsMenuTab()` draws the mockup's MENU.
## Must never route a menu through a bespoke screen (Ruling DX Q13): a phone-only row belongs in the menus,
## marked phone or tablet only, or is dropped by a stated decision.
func _fill_root(body: VBoxContainer) -> void:
	_fill_quick(body)
	_head(body, "Menus")
	for mb in _menu_buttons():
		_add(body, _menu_root_row(mb as MenuButton))
	_fill_status(body)

## One root row: the menu's name, the mockup's subtitle for it, the count of its rows and its glyph. Pressing
## it pushes the menu's popup, which `_fill_popup()` draws row for row with every live refresh the desktop runs
## (`about_to_popup`). The same push and count as the shipped `_menu_row()`, which drew the old fallback band;
## only the subtitle and the glyph differ. Never refreshes the popup itself: the root only names the menus.
func _menu_root_row(mb: MenuButton) -> Control:
	var popup := mb.get_popup()
	var title := String(mb.text)
	var count := 0
	for i in popup.item_count:
		if not popup.is_item_separator(i):
			count += 1
	return _row(title, String(MENU_SUBS.get(title, "")), _trail_label("%d" % count), _chevron(),
		func(): _push(popup, title, SCREEN_LEVEL), false, _glyph(String(MENU_GLYPHS.get(title, "▸"))))

## The Status band, moved out of the old `_fill_root()` without change: the readouts that hold a real value,
## and "Recompute stale stages" when a recompute is possible. Copied verbatim from `_fill_root()` at `5bd7af8`.
## Diff it against the shipped block before pasting. The help text's path "Civilization ▸ Settlements ▸ …" is
## prose to re-check once Civilization's door moves (MS-3).
func _fill_status(body: VBoxContainer) -> void:
	var status := _status_rows()
	if not status.is_empty():
		_head(body, "Status")
		for entry in status:
			_add(body, _note_row(String(entry[0]), String(entry[1])) if bool(entry[2])
				else _value_row(String(entry[0]), String(entry[1])))
		if _can_recompute_stale():
			_add(body, _row("Recompute stale stages",
				"Re-runs only the stages the graph reports stale. The civilisation "
					+ "layer is deliberately not cascaded per edit, so \"civ\" usually "
					+ "stays -- Civilization ▸ Settlements ▸ Recompute civilisation is "
					+ "the one that clears it.",
				null, null, _go_recompute_stale, false))

## The Story screen's body (`story-root`, the STORY tab through `open_story()`): the four band rows of
## `STORY_ROWS`, each opening its band (`_fill_story()`). MENU ▸ Story is the Story popup itself, row for row
## (Ruling DX Q13); this screen is the STORY tab's. Replaces the shipped `_fill_story_section()`, which drew these
## rows as a section of the old MORE root: it opened with a "Story" head and closed with a "World" head for the
## band that followed it there. On a screen of its own the screen title already says Story, and the "World" head
## headed nothing (measured by the menu parity check, 2026-11-01). Its only shipped caller was the old
## `_fill_root()`, which this draft replaces.
func _fill_story_section(body: VBoxContainer) -> void:
	for r in STORY_ROWS:
		var band := String(r["band"])
		_add(body, _row(String(r["label"]), String(r["sub"]), null, _chevron(),
			func(): _push_screen("story", band), false, _glyph(String(r["glyph"]))))

## Re-presents a desktop `PopupMenu` (`step.popup`) as rows: emits its `about_to_popup` first so
## dynamic menus (recent worlds, GPU devices, open windows) fill, then draws each item via `_popup_row`
## with hairlines between them and a titled band for any labelled separator. A leading separator is skipped
## only when it has no caption. Since Ruling DX Q13 this draws all eight of MENU's menus and every submenu
## drilled from them.
##
## **Replaces the shipped `_fill_popup()`, with one test changed** (2026-11-01). It skipped every leading
## separator, captioned or not, and its comment said every separator `menus.gd` writes is unlabelled; many are
## labelled now (RECORDS, FOR THE TABLE, SOURCES, ACTIVE PACK), and a leading one was dropped, caption and all.
## A leading plain separator is still skipped: it would draw a gap above the first row.
func _fill_popup(body: VBoxContainer, step: _Step) -> void:
	var p := step.popup
	## Live exactly as the desktop menu is live: Recent worlds, GPU devices,
	## Open windows and the Preferences busy-lock all populate here.
	p.about_to_popup.emit()

	var first := true
	for i in p.item_count:
		if p.is_item_separator(i):
			## Canvas: "L3 stays a titled band, never a disclosure." A labelled
			## separator is a caption, wherever it sits; an unlabelled one is the
			## rule-and-gap the desktop draws, except at the very top.
			if first and _clean(p.get_item_text(i)) == "":
				continue
			body.add_child(_band(_clean(p.get_item_text(i))))
			continue
		if not first:
			body.add_child(DccTheme.rule())
		first = false
		body.add_child(_popup_row(p, i))

## The STORY tab (Ruling DT Q1). Opens this sheet on the Story screen, or straight on one band
## ("TIME", "JOURNEYS", "CONFLICTS & CAMPAIGNS" or "NOTES", the `STORY_ROWS` bands). System back from a band
## returns to the Story screen, and from there closes the sheet.
func open_story(band: String = "") -> void:
	_stack.clear()
	_stack.append(_Step.new(null, _screen_title("story-root"), "", 2, "story-root"))
	visible = true
	if band != "":
		_push_screen("story", band)
	else:
		_render()

## Opens MENU on its root and pushes screen `id` over it, so system back returns to MENU. For a row a program
## menu carries whose content is a phone screen: Help ▸ Gesture reference… (`gestures`, draft 09), reached through
## `DccShell.phone_open_menu_screen()`. Must not be given a screen this draft leaves unreachable: those are being
## deleted (this file's header).
func open_screen(id: String) -> void:
	_stack.clear()
	_stack.append(_Step.new(null, _screen_title("more"), "", 2, "more"))
	visible = true
	_push_screen(id)

## DRAFT, not loaded by Godot. MS-4 (MENU) and MS-6 (STORY).
## - MENU (Ruling DT Q2): the eight desktop menus, row for row, in the menu bar's order and under its names, with
##   Quick first and Status last.
## - STORY (Ruling DT Q1): the four bands TIME · JOURNEYS · CONFLICTS & CAMPAIGNS · NOTES, opened from the
##   STORY tab.
## Target: `shell/phone_menu.gd`. It replaces `_fill_root()`, `_fill_story_section()` and `_fill_popup()`, adds the
## `world` and `story-root` screens and `open_story()`, and retires `ROOT_ROWS`, `ROOT_REST`, `_named_in_root()` and
## the "Not on the MORE list" band.
##
## **Corrected 2026-11-01 (menu parity check, `cartalith-native/docs/evidence/menu-parity-2026-11-01/`).** Three
## faults the check measured: (1) `story-root` drew `_fill_story_section()`, which was written as a section of the
## old MORE root, so the Story screen opened with a "Story" head and ended with an empty "World" head; it is
## replaced below. (2) The comment on `MENU_SCREENS` said `_fill_project()` draws File's rows in File's order; it
## draws Save, Save as, New world and Open project first. Corrected there; whether MENU keeps the phone screens'
## own order is owner question P1 in that folder's REPORT.md. (3) `_fill_popup()`, which draws MENU ▸ Edit,
## MENU ▸ Window and every submenu drilled from MENU, skipped a leading separator even when it carried a caption,
## so File ▸ Export lost FOR THE TABLE, World ▸ Sources & validation lost SOURCES and Assets ▸ Asset pack lost
## ACTIVE PACK. Ruling DT Q2 wants the menus row for row; it is replaced below with only that one test changed.
## Not corrected here: `_fill_help()` ends with an `_info` line naming the phone tabs
## `MAP · GENERATE · PLAN · MORE`, which Ruling DT renames. The rename is MS-2's, so draft 03 carries that edit
## (its EDIT H1), to land with the tabs it names.
##
## Almost everything here is existing PhoneMenu machinery, re-ordered: `_fill_quick()`, `_menu_buttons()`,
## `_menu_row()`, `_push_screen()`, `_rest_of()`, `_fill_story_section()`, `_fill_story()`, and the Status block.
## No row is new; every retired root row has a stated home (the table in this folder's README, which the
## `MISTAKES.md` "Delete a surface" row requires).
##
## NOT parse-checked (no Godot binary in the session that wrote it).

## A menu with a phone screen opens that screen, which draws the menu's rows together with its phone-only parts.
## **Not in the menu's own order everywhere** (corrected 2026-11-01): `_fill_project()` draws Save, Save as, New
## world and Open project first, Recent worlds inline under its own head, Autosave and its interval chips under
## theirs, then the rest of File in File's order; `_fill_assets()` puts the nine families and the pack's values
## above the Assets rows; `_fill_prefs()` puts Theme and Units first; `_fill_help()` adds Runtime, Engine and
## Gesture reference. The other two, Edit and Window, open their whole popup through `_menu_row()`, as the
## retired band did.
const MENU_SCREENS: Dictionary = {
	"File": "project", "Assets": "assets", "World": "world", "Story": "story-root",
	"Preferences": "prefs", "Help": "help",
}
## Subtitles for the screen-backed rows. Each is the retired `ROOT_ROWS` subtitle where one existed.
const MENU_SUBS: Dictionary = {
	"File": "save · recent · autosave · export",
	"Assets": "families · slots · packs · landmark types",
	"World": "data manager · records · technical",
	"Story": "time · journeys · conflicts · notes",
	"Preferences": "theme · units · performance · graphics",
	"Help": "documentation · shortcuts · credits · about",
}
## Lead glyphs. `_glyph()` already falls back to the submenu arrow for a glyph the mono font lacks.
const MENU_GLYPHS: Dictionary = {
	"File": "⧉", "Edit": "✎", "Assets": "▦", "World": "◍", "Story": "➔",
	"Preferences": "⚙", "Window": "▢", "Help": "?",
}
## MENU ▸ World's "On the phone" band: the phone screens the retired root listed under its World band. Each
## keeps a door here. Civilization's lasting home is the CIVIL domain sheet (MS-3); it stays here until then.
const WORLD_SCREENS: Array = [
	{"id": "civ", "glyph": "◍", "label": "Civilization", "sub": "settlement · POI · way tools"},
	{"id": "data", "glyph": "⇅", "label": "Data manager", "sub": "import · export · sources · validation"},
	{"id": "travel", "glyph": "≋", "label": "Travel library", "sub": "animals · vehicles · vessels · parties"},
	{"id": "sim", "glyph": "◷", "label": "Simulation", "sub": ""},
]

## Add these to `SCREEN_TITLES`. The `more` entry is renamed for MENU (Ruling DT Q2); the key stays `more`, so
## `open()` and every probe that pushes it keep working.
##     "more": ["Menu", "the desktop menus · quick · status"],
##     "world": ["World", "data manager · records · technical"],
##     "story-root": ["Story", "time · journeys · conflicts · notes"],
## and these two arms to `_fill_screen()`'s match:
##     "world": _fill_world(body)
##     "story-root": _fill_story_section(body)

## MENU's root: Quick, the eight menus, then Status.
func _fill_root(body: VBoxContainer) -> void:
	_fill_quick(body)
	_head(body, "Menus")
	for mb in _menu_buttons():
		var title := String((mb as MenuButton).text)
		var screen := String(MENU_SCREENS.get(title, ""))
		if screen == "":
			_add(body, _menu_row(mb as MenuButton))
			continue
		var target := screen
		_add(body, _row(title, String(MENU_SUBS.get(title, "")), _root_badge(target), _chevron(),
			func(): _push_screen(target), false, _glyph(String(MENU_GLYPHS.get(title, "▸")))))
	_fill_status(body)

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

## MENU ▸ World: the desktop World menu row for row, then the "On the phone" band.
func _fill_world(body: VBoxContainer) -> void:
	var p := _menu_popup("World")
	if p == null:
		_add(body, _missing_row("World", "The World menu is not on this build's menu bar."))
	else:
		_rest_of(body, p, {})
	_head(body, "On the phone")
	for spec in WORLD_SCREENS:
		var target := String(spec["id"])
		var sub := String(spec["sub"])
		if sub == "":
			sub = _live_root_sub(target)
		_add(body, _row(String(spec["label"]), sub, _root_badge(target), _chevron(),
			func(): _push_screen(target), false, _glyph(String(spec["glyph"]))))

## The Story screen's body (`story-root`, MENU ▸ Story and the STORY tab through `open_story()`): the four band
## rows of `STORY_ROWS`, each opening its band (`_fill_story()`). Replaces the shipped `_fill_story_section()`,
## which drew these rows as a section of the old MORE root: it opened with a "Story" head and closed with a
## "World" head for the band that followed it there. On a screen of its own the screen title already says Story,
## and the "World" head headed nothing (measured by the menu parity check, 2026-11-01). Its only shipped caller
## was the old `_fill_root()`, which this draft replaces.
func _fill_story_section(body: VBoxContainer) -> void:
	for r in STORY_ROWS:
		var band := String(r["band"])
		_add(body, _row(String(r["label"]), String(r["sub"]), null, _chevron(),
			func(): _push_screen("story", band), false, _glyph(String(r["glyph"]))))

## Re-presents a desktop `PopupMenu` (`step.popup`) as rows: emits its `about_to_popup` first so
## dynamic menus (recent worlds, GPU devices, open windows) fill, then draws each item via `_popup_row`
## with hairlines between them and a titled band for any labelled separator. A leading separator is skipped
## only when it has no caption.
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

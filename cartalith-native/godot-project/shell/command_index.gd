extends RefCounted
class_name CommandIndex

## Every action, parameter and export the app has, as one searchable list.
##
## Owner, 2026-08-30: *"I'm still not happy with how functions and options are
## made known to the user on android the interface is hard to understand and
## muddled."* This is Direction A of the phone redesign: one search field that
## answers "what can this thing do?" without the user already knowing the
## vocabulary.
##
## ## It is BUILT, not written down
##
## The same principle `ShortcutsDialog` uses, for the same reason. Three real
## tables are read at build time:
##
## - **Generation parameters** — `EngineBridge.param_keys()` and
##   `param_info(key)`, which are `cartalith-godot`'s own `ParamSpec` rows
##   (`params.rs`): key, group, label, unit, type, range, and the reference
##   control id. That is where the wording already lives, and
##   `world_workspace.gd`'s own comment says of it: *"nothing about the range,
##   step, label or unit is guessed here."* Neither is anything here.
## - **Menu commands** — walked off the live `MenuBar`, exactly as
##   `ShortcutsDialog` walks it, so a row added to a menu appears here with no
##   edit to this file.
## - **Declared extras** — [`EXTRAS`], the few real actions that are neither a
##   parameter nor a menu row.
##
## A hand-maintained catalogue of an app's own features is the most reliably
## stale document a project can own. This one cannot disagree with the app,
## because it is assembled from the app.
##
## ## Availability is part of the entry, not a surprise
##
## Every entry carries `available` and, when false, `why`. The old shell had
## ten disabled rows that looked identical to live ones — the exact defect
## `PARITY_AUDIT.md` §23 has been finding all week, and the reason the Edit
## menu spent months telling users deletion was impossible while the Delete key
## deleted. A row that cannot run says so where it is read.
##
## ## Three states, not two
##
## A disabled menu row is one of **three** things, and this file read it as two
## until 2026-08-31:
##
## - **chrome** — a signpost ("imports live under Data ▸ Import", the six
##   "Rows here read state" lines under Assets ▸ Landmark types) or a
##   placeholder ("— loading —"). Skipped entirely. `menus.gd::_signpost()`
##   marks the first kind with `DccMenus.META_SIGNPOST`; the rest are still
##   recognised the old way, by being disabled and silent.
## - **a readout** — disabled, tooltipped, and carrying a live *value*:
##   Working set, the pack schema line, the VRAM estimate, the two GPU memory
##   lines, "No recent projects". `menus.gd::_readout()` marks these with
##   `DccMenus.META_READOUT`.
## - **an unavailable command** — `menus.gd::_todo()`, and only that. A thing
##   the app will one day do and cannot yet.
##
## Only the third belongs in the list as `available: false`. Indexing the five
## readouts that way put entries in the list that a user could search for, tap,
## and get nothing from — the exact defect this header opens by describing,
## reproduced by the fix for it. They are kept, as `kind: "readout"`, because
## searching "memory" and being shown `Working set 1.6 GB of 12 GB` is the
## answer, not a dead end: the row IS the result.
##
## ## What this reads is the BUILT state, not the opened state
##
## [`_walk_popup`] reads each row exactly as `menus.gd` left it, plus whatever
## that file refreshes at build time — `_refresh_reset_stage_menu()`,
## `_refresh_atlas_cache_menu()` and `_refresh_gpu_devices_menu()` all run
## inside their own `_build_*`, so their state does reach here. Rows whose
## availability is decided **only** in an `about_to_popup` handler do not:
## `Edit ▸ Undo`, `Redo` and `Reset generation parameters` are indexed as
## available because that is how `_live()` leaves them, even when nothing is on
## the undo stack yet.
##
## **Do not close that by emitting `about_to_popup` from `build()`.** Those
## handlers are not observers. `_refresh_atlas_cache_menu()` calls
## `_enforce_atlas_cap()`, which *evicts baked chunks*; the GPU devices handler
## enumerates adapters, which `menus.gd::_build_gpu_devices_menu` documents as
## the crash it was restructured to avoid. Firing them to tidy up an index
## would make opening a search field destroy the user's bake. Recorded
## 2026-09-03 during the unavailable-row audit so the next reader does not
## re-derive it and then do it anyway.

## Actions that are neither a generation parameter nor a menu row. Deliberately
## short: anything that belongs in a menu should BE in a menu, where the walk
## finds it for free.
## **Two rows added 2026-09-05, and the reason is the point of this table.**
## `_add_menu_commands()` walks the live `MenuBar`, so a command that stops being
## a menu row stops being searchable — silently, with nothing failing. The owner's
## 2026-09-05 ruling moved `Journey planner` to the CIVIL rail and
## `Refine detail for the current view` to the WORLD tool-options bar; a verifier
## then measured **0 title matches out of 361 indexed rows** for both. Neither
## move was authorised to remove a command from search.
##
## A rail node and a bar button are exactly what `EXTRAS` is for: real actions
## that are not menu rows. **Any future move off the menu bar owes this table a
## row in the same change.**
##
## **`Refine detail for the current view`'s row came back out 2026-09-21,
## Ruling U (`LARGE_ITEM_RULINGS.md`).** The move that put it here reversed --
## it is a real `Preferences ▸ Tiles & LOD ▸ Atlas cache` `PopupMenu` row again
## (`menus.gd::_build_atlas_cache_menu()`), so `_add_menu_commands()`'s own walk
## finds it for free and an `EXTRAS` entry would be a second, driftable copy of
## a title the walk already owns. `Journey planner` stayed for the same
## reason in reverse until **SM-2 (2026-10-18, Ruling CK)** gave it a real row
## again, `Story ▸ JOURNEYS ▸ Journey planner` (`menus.gd::_story()`): the walk
## now finds it, so its `EXTRAS` entry came out for the same reason
## `Refine detail`'s did. The rail node stays as well (Ruling 4 is not reversed;
## the menu row is an additional entry point).
##
## **The Asset-pack flattening (same day) is the mirror case, decided the other
## way -- zero rows, checked rather than assumed.** Its fifteen dropped rows
## were never rail nodes or bar buttons; they were shortcuts into the
## `⧉ Asset library` window, and the nine carrying real per-slot/batch
## behaviour are each a labelled control already visible inside that window --
## `asset_library_window.gd::_build_slot_grid()`'s batch-verb row
## (`Tag…`/`Collect…`/`Rename…`/`Duplicate`/`Delete`, live the moment any slot
## is selected) and `_build_inspector()`'s
## `Scale`/`Fit`/`Reset`/`Replace`/`+Variant`/`bg`/tags row (live the moment one
## slot is focused). An `EXTRAS` pointer for each would only ever open the same
## window `⧉ Asset library` already opens -- recreating, in this table, the
## nine-shortcuts-to-one-window shape the owner's ruling flattened out of the
## menu. `menus.gd::_build_asset_pack_submenu()` carries the re-derived
## title-match measurement and the full per-command accounting.
## **`Theme` added 2026-09-13, and it is a POINTER row, not a moved command.**
## `Dark` / `Light` / `Follow system` never left the menu bar -- they are real
## `PopupMenu` rows on every composition (`menus.gd::_build_theme_submenu()`)
## and `_add_menu_commands()`'s own walk already indexes each of them by its
## own name, `kind: "menu"`, with no help from this table. What the walk
## cannot do is answer the word "theme" itself: `_walk_popup()` never indexes
## a row that owns a submenu, only its leaves, so "Dark"/"Light"/"Follow
## system" (their titles) and "☰"/"Preferences" (their group, `menu_name`)
## carry no occurrence of the word a searcher is most likely to type. This row
## exists only to answer THAT search, and only for tablet's own flat
## `ID_TABLET_TOGGLE_THEME` row, `Toggle theme`, which is a real, separately-
## indexed "menu" row on tablet already and needs no pointer of its own -- the
## gap is the WORD, not the command.
##
## **Two POINTER rows added by Ruling EH item 2** (`Landmark icons`, `Landmark label style`): the cascade's two fixed destination rows became buttons in the Landmark types window, so a menu walk no longer finds them. Pointer rows (not aliases): no live row carries these titles, and they are counted by TITLE (`_lmtypes_probe.gd`), per `MISTAKES.md` "Move a command off the menu bar". `Placed by hand` needs none: it was a count readout, and the window carries it.
##
## **Four ALIAS rows added by PC-9 (`PC_SHELL_PROPOSAL.md` §3.8), the `alias`
## key that came with them, and one POINTER row from Ruling DY.** "Import image" left Assets (it opened the Asset
## library under a second name). "Notes for this place" becomes "Note for the
## selection" when the Context panel gains its Note tab (PC-5), and "Right dock"
## becomes "Context panel" when the panel exists (PC-4; on the phone too, where
## it names the selection sheet, Ruling DX PQ13). "Run pipeline" became "Generate
## world" on the narrow tablet (Ruling DX PQ11: one label on every form factor).
## After each, a search for the old words finds nothing by title -- the defect
## this table is for. `alias: true` makes `build()` skip the row while a live
## menu row still carries that exact title (before the change that renames it),
## so a pointer never sits beside the row it points away from. Renames whose old words still match the
## new title ("political history", "assistant", "keyboard") and titles still on
## the bar elsewhere ("Timeline", "Places without notes") need no alias. Counted
## by TITLE, never by `search()`, which matches blurbs (`MISTAKES.md`, "Move a
## command off the menu bar").
## "Records (was World)" is a pointer row like `Theme`, not an alias: Ruling DY
## (2026-11-01) renamed the World menu Records, and the index files each row under
## its menu's name (`group`), so the WORD "world" stopped reaching the records
## rows. It cannot be an alias titled "World": Window ▸ Workspace ▸ World is a
## live row with that exact title, so `_has_menu_title()` would hide it forever.
const EXTRAS: Array = [
	{"title": "Zoom to fit", "blurb": "Frame the whole world in the viewport", "group": "View"},
	{"title": "Point sample", "blurb": "Read elevation, biome and climate under one cell", "group": "View"},
	{"title": "Theme", "blurb": "Dark, Light or Follow system. Preferences ▸ Theme -- ☰ ▸ Preferences ▸ Theme on tablet, which also has a one-tap dark/light flip at ☰ ▸ Toggle theme", "group": "Preferences"},
	{"title": "Import image", "blurb": "Moved: Assets ▸ ⧉ Asset library (Shift+A; ☰ on a tablet) opens the library, and its own Import image… in the rail foot imports into the focused slot.", "group": "Assets", "alias": true},
	{"title": "Notes for this place", "blurb": "Now Story ▸ Note for the selection (Ctrl+Shift+N): the Context panel's Note tab, for the selected place or faction (any other subject says why it has no note).", "group": "Story", "alias": true},
	{"title": "Right dock", "blurb": "Now the Context panel: Window ▸ Context panel (☰ ▸ Window on a tablet; on a phone, the selection sheet) shows or hides it, and the submenu under it opens Inspect, Note and Ask (Ctrl+1, Ctrl+2, Ctrl+3).", "group": "Window", "alias": true},
	{"title": "Run pipeline", "blurb": "Now Generate world (Ctrl+R): File ▸ Generate world, or World ▸ Generate world on a tablet whose bar has a World menu. The same command under one label (Ruling DX).", "group": "File", "alias": true},
	{"title": "Records (was World)", "blurb": "The menu called World is Records now (Ruling DY): ⧉ Data manager, Factions… (Ctrl+Shift+F), Places…, Markdown vault… (Ctrl+Shift+V), Browse & edit a note…, Places without notes…, World data tables…, Import and Sources & validation. Generate world (Ctrl+R) is in File. On a tablet whose bar has a World menu, that menu holds the WORLD domain's own rows.", "group": "Records"},
	{"title": "Landmark icons", "blurb": "Moved into Assets ▸ Landmark types… (Ruling EH): its foot button opens the Asset library on the points-of-interest family, whose slots landmark types draw with.", "group": "Assets"},
	{"title": "Landmark label style", "blurb": "Moved into Assets ▸ Landmark types… (Ruling EH): its foot button opens Cartography ▸ Labels, where a landmark's label style lives.", "group": "Assets"},
]

## The index: one dictionary per entry (`title`, `blurb`, `group`, `kind`, `key`,
## `available`, `why`). Rebuilt from scratch by every `build()`; never edited
## in place by callers (`all()` hands out a copy).
var _rows: Array = []
## MS-9: set by `build(app, bridge, true)`. When true `_walk_popup()` runs a popup's open refreshers before reading
## it, but only for a popup holding a check or radio that registered no state source (the one kind of row whose
## state is read off the item). Off for every quick-bar caller: a chip press must not run refreshers.
var _refresh_first := false
## The engine bridge, kept from `build()`; read only for the parameter table.
## Untyped so a harness can pass a stub.
var _bridge
## The shell, kept from `build()`; its live `MenuButton`s are walked for the menu
## commands. Null yields an index of parameters and extras only.
var _app

## (Re)builds the whole index: generation parameters, then every live menu row,
## then `EXTRAS`. Clears first, so calling it again never duplicates rows. Must
## not emit `about_to_popup` on any menu (see the header: those handlers evict
## baked chunks and enumerate GPU adapters).
func build(app, bridge, refresh_first: bool = false) -> void:
	_app = app
	_bridge = bridge
	_refresh_first = refresh_first
	_rows.clear()
	_add_parameters()
	_add_menu_commands()
	_add_tools()
	_disambiguate_pin_keys()
	for e in EXTRAS:
		if bool(e.get("alias", false)) and _has_menu_title(String(e.get("title", ""))):
			continue
		_rows.append({
			"title": String(e.get("title", "")),
			"blurb": String(e.get("blurb", "")),
			"group": String(e.get("group", "Other")),
			"kind": "action",
			"available": true,
			"why": "",
			"key": "",
			"pin_key": "extra:" + _slug(String(e.get("title", ""))),
			"pinnable": false,
			"why_not": "This is a search result, not a command, so it cannot be pinned.",
		})

## Whether a menu row titled `title` is in the index right now, compared the way `_walk_popup()` stores
## titles (the ellipsis dropped, edges stripped) and without regard to case. Counts `menu` and
## `readout` rows, and a `_todo` row too (it is a menu row whose command is not built): any of them
## means the title is still on the bar, so an alias for it would sit beside it.
##
## Must run after `_add_menu_commands()`, which is the only thing that adds menu rows; `build()` calls
## it in the `EXTRAS` loop, after that walk. Reads the index only.
func _has_menu_title(title: String) -> bool:
	var want := title.replace("…", "").strip_edges().to_lower()
	for r in _rows:
		var kind := String(r.get("kind", ""))
		if (kind == "menu" or kind == "readout") and String(r.get("title", "")).to_lower() == want:
			return true
	return false

## Lower-case ASCII slug of a title: the stable half of an `extra:` pin key.
## Never used for a `menu:` key, which follows the persisted action id.
static func _slug(t: String) -> String:
	var out := ""
	for ch in t.to_lower():
		out += ch if (ch >= "a" and ch <= "z") or (ch >= "0" and ch <= "9") else "_"
	return out.strip_edges()

## Generation parameters, straight off the engine's own spec table.
func _add_parameters() -> void:
	if _bridge == null or not _bridge.has_method("param_keys"):
		return
	var has_world: bool = _bridge.has_world
	for key in _bridge.param_keys():
		var info: Dictionary = _bridge.param_info(String(key))
		if info.is_empty():
			continue
		var unit := String(info.get("unit", ""))
		var group := String(info.get("group", "")).capitalize()
		var ref_ctrl := String(info.get("reference_control", ""))
		## The blurb says what the thing IS, using the engine's own wording
		## plus the one fact a searcher wants: is this a switch or a value,
		## and in what units.
		var kind := String(info.get("type", "float"))
		var blurb := ("A switch" if kind == "bool" else "A value")
		if unit != "":
			blurb += ", in " + unit
		if ref_ctrl != "":
			blurb += " · reference control #" + ref_ctrl
		_rows.append({
			"title": String(info.get("label", key)),
			"blurb": blurb,
			"group": group if group != "" else "Generation",
			"kind": "param",
			"key": String(key),
			## A parameter is always readable; it only *applies* on a
			## generate, which is a different statement from "unavailable".
			"available": true,
			"why": "" if has_world else "Takes effect on the next Generate",
			"pin_key": "param:" + String(key),
			"pinnable": false,
			"why_not": "A generation parameter applies on a Generate and has its own dock control; a chip would only duplicate it.",
		})

## CQ-8: the state getter `DccMenus.state_source(id)` registered for action `id`, or an invalid `Callable`
## (the row then stays unpinnable, with its reason). Reads the live menus object off the app, so a stub app
## without one has no toggles.
func _state_source_for(id: int) -> Callable:
	if _app == null:
		return Callable()
	var menus = _app.get("menus")
	if menus == null or not menus.has_method("state_source"):
		return Callable()
	return menus.state_source(id)

## CQ-8 (Ruling EH item 3, scope Q10): the palette's global tools as pinnable `tool` rows -- Inspect, Measure
## and Region select (`DccWidgets.GLOBAL_TOOL_ENTRIES`, the same table the palette draws, so a tool added there
## is pinnable here with no second list). A domain's own tools are not offered: they exist only in their
## domain's palette and their controls live in that domain's dock, so a chip that armed one from another
## domain would arm a tool with nothing to control it. A tool row is a toggle whose state is "this tool is
## armed" (`armed_tool`, the shell's own variable, read live).
func _add_tools() -> void:
	if _app == null or not _app.has_method("arm_tool"):
		return
	for e in DccWidgets.GLOBAL_TOOL_ENTRIES:
		if (e as Dictionary).has("legend"):
			continue
		var tid := String(e["id"])
		## "Measure (M)" -> "Measure": the key letter is the palette's tooltip, not the command's name.
		var title := String(e["label"])
		var paren := title.find(" (")
		if paren > 0:
			title = title.substr(0, paren)
		var app = _app
		_rows.append({
			"title": title,
			"blurb": "Arms the %s tool, exactly as its palette button does" % title,
			"group": "Tools",
			"kind": "tool",
			"key": "",
			"available": true,
			"why": "",
			"pin_key": "tool:" + tid,
			"pinnable": true,
			"why_not": "",
			"_tool": tid,
			"toggle": true,
			"_state": func() -> bool: return String(app.armed_tool) == tid,
			"refreshes_on_open": false,
		})

## Every menu row, walked off the live MenuBar. A `_todo` row is disabled and
## carries its reason as a tooltip -- both are read here, so the index reports
## the same truth the menu does rather than a second opinion.
func _add_menu_commands() -> void:
	if _app == null:
		return
	var buttons: Array = []
	_gather_menu_buttons(_app, buttons)
	for mb in buttons:
		var popup: PopupMenu = (mb as MenuButton).get_popup()
		## MS-9: a MenuButton with no text is an embedded control's own popup (a ColorPicker's mode and
		## shape presets: 264 of 722 rows on 2026-11-12, every one titled like a command and none a command of
		## this app). Search now RUNS what it lists, and running one would only change an invisible picker, so they
		## are not indexed at all rather than listed as rows that cannot be pressed.
		if popup != null and String((mb as MenuButton).text).strip_edges() != "":
			_walk_popup(popup, String((mb as MenuButton).text))

## Appends one row per leaf item of `popup`, recursing into submenus under the same
## `menu_name`. Skips separators, blank items, marked signposts and silent disabled
## rows (chrome); keeps marked readouts as `kind: "readout"` and unavailable
## commands as `available: false` with their tooltip as `why`. Reads the state the
## menu was BUILT in, not a state an `about_to_popup` handler would give it.
func _walk_popup(popup: PopupMenu, menu_name: String) -> void:
	if _refresh_first and _needs_item_state(popup):
		_refresh_popup(popup)
	for i in popup.item_count:
		var text := popup.get_item_text(i)
		if text.strip_edges() == "" or popup.is_item_separator(i):
			continue
		var sub := popup.get_item_submenu(i)
		if sub != "":
			var node := popup.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				_walk_popup(node as PopupMenu, menu_name)
			continue
		var disabled := popup.is_item_disabled(i)
		var tip := popup.get_item_tooltip(i)
		## The readout marker, set by `menus.gd::_readout()`. Read off the
		## metadata rather than guessed from the text, because several of these
		## rows are rewritten wholesale on every `about_to_popup` and any test
		## over their wording would be a test over a moving string.
		var meta = popup.get_item_metadata(i)
		var marker := String(meta) if typeof(meta) == TYPE_STRING else ""
		var is_readout := marker == DccMenus.META_READOUT
		## **A signpost is prose, not a command.** Marked at the source rather
		## than inferred here, so a reworded sentence cannot turn one back into
		## an indexed action.
		if marker == DccMenus.META_SIGNPOST:
			continue
		## **A disabled row with no tooltip is chrome, not a command** -- unless
		## it is a marked readout, exempted below.
		## `menus.gd::_todo(p, text, why)` ALWAYS sets a tooltip -- that is its
		## whole signature -- so a real not-built-yet row is never silent. What
		## is disabled and silent is a placeholder -- the Assets menu's
		## "-- loading --", which is the pack list mid-fetch. (The signposts
		## that used to be caught here are caught by the marker above; this
		## rule stays because a placeholder is not authored one row at a time.)
		## Indexing those as commands put entries in the list that a user could
		## search for, tap, and get nothing from.
		##
		## Found by the probe rather than by reading: it asserted every
		## unavailable row carries a reason and reported 26 of 29.
		##
		## **`is_readout` is exempted from this skip, found 2026-09-05.** A
		## readout's tooltip is its explanation, not its availability gate --
		## the marker already answers that (`_readout()`'s own header: "the
		## marker is the metadata, not the tooltip") -- and one can be genuinely
		## empty for a frame. `menus.gd`'s lighting-rig provenance row
		## (`_light_source_idx`) is built via
		## `_readout(_lighting_popup, "— loading —", "")` and, unlike its
		## sibling `_atlas_popup` readout, had no eager `_refresh_*` call after
		## construction, so `about_to_popup` was the only path that ever gave it
		## a tooltip. A `CommandIndex` built before that submenu was ever opened
		## -- every index a cold search builds -- hit this exact branch and
		## dropped a real, correctly-marked row out of the index entirely: not
		## "unavailable with no reason", gone. Fixed on both sides:
		## `_build_lighting_menu()` now calls `_refresh_lighting_menu()` at
		## build time too (matching the atlas pattern it already cited as its
		## model), and this line no longer trusts that every future readout's
		## author will remember to add one.
		if disabled and tip.strip_edges() == "" and not is_readout:
			continue
		## A readout is available: it has already told the searcher what they
		## came for, in its own title. `why` stays empty for the same reason —
		## there is nothing being withheld to explain.
		var id := popup.get_item_id(i)
		## CQ-8 (Ruling EH item 3, scope Q8): a check or radio row is pinnable as a TOGGLE only when the menu
		## owner registered where its on/off state really lives (`DccMenus.state_source`); every other one
		## keeps its reason. `_state` is that getter itself, never a copy of its answer.
		var checkable := popup.is_item_checkable(i) and not is_readout
		var state_fn := _state_source_for(id) if checkable else Callable()
		var why_not := _why_not_pinnable(popup, i, id, is_readout, menu_name, state_fn.is_valid())
		_rows.append({
			"title": text.replace("…", "").strip_edges(),
			"blurb": tip if (disabled and tip != "") else (menu_name + " menu"),
			"group": menu_name,
			"kind": "readout" if is_readout else "menu",
			"key": "",
			"available": is_readout or not disabled,
			"why": tip if (disabled and not is_readout) else "",
			## CQ-0 (CUSTOM_QUICK_BAR_SCOPE.md s.3.3). `pin_key` follows the action id
			## `shortcuts_dialog.gd` already persists, so a reword of the title
			## cannot orphan a pin. Ids are dispatched per PopupMenu, so a collision
			## between two popups is possible; `_disambiguate_pin_keys()` resolves it.
			"pin_key": "menu:%d" % id,
			"pinnable": why_not == "",
			"why_not": why_not,
			## Not stored anywhere: the owning popup and id, so `run_row()` can emit
			## that popup's own `id_pressed`. Object refs never reach a config.
			"_popup": popup,
			"_id": id,
			## MS-9: the item's own text and its check/radio shape, so a search press can find the item again
			## (`_item_of`), refuse when the menu has changed since this row was built, and read the state of a
			## check or radio that registered no state source.
			"_text": text,
			"checkable": checkable,
			"radio": checkable and popup.is_item_radio_checkable(i),
			## True when the popup refreshes row state on open, so `available` here
			## is the BUILT state and a chip must not copy it (scope s.3.3).
			"refreshes_on_open": not _open_handlers(popup).is_empty(),
			## CQ-8: present only on a toggle row. `toggle` says the chip shows on/off; `_state` is the
			## getter a chip polls (no key at all on any other row: omit rather than default).
			"toggle": state_fn.is_valid(),
			"_state": state_fn,
		})

## Id spans that are generated per item (a ladder, a list), never a fixed command.
## A pin on one would point at whatever happens to sit at that index tomorrow.
## Names are DccMenus constants so a renumber cannot silently desynchronise. The
## span lengths (9, 14, 32, 10) are labelled judgements rounded up from the
## constants' own comments; `_cmdindex_probe.gd` section 7 lists every pinnable
## row so an over- or under-wide span is visible, not assumed.
func _is_dynamic_id(id: int) -> bool:
	return (id >= DccMenus.ID_AUTOSAVE_FIRST and id < DccMenus.ID_AUTOSAVE_FIRST + 9) 		or (id >= DccMenus.ID_DATA_ROUTE_FIRST and id < DccMenus.ID_DATA_ROUTE_FIRST + 14) 		or (id >= DccMenus.ID_WIN_LAYOUT_FIRST and id < DccMenus.ID_WIN_LAYOUT_FIRST + 32) 		or (id >= DccMenus.ID_RESET_STAGE_FIRST and id < DccMenus.ID_RESET_STAGE_FIRST + 10)

## True when a popup numbers its items from 0. That is the signature of a list built
## from data (the icon families, the texture sets, the retired landmark cascade's per-family types):
## the id means "the n-th entry today", so it is not a stable command identity and
## the same small ids repeat in every such popup. No named `DccMenus.ID_*` command
## is 0 (checked by grep 2026-10-25), so an item with id 0 marks an index popup.
## Measured by `_cmdindex_probe.gd` section 7: without this rule 41 pinnable rows
## shared 9 small ids across 8 such popups. (Not "id == position": a landmark
## family's popup has gaps, so that test missed it on the first attempt.)
func _is_index_popup(popup: PopupMenu) -> bool:
	for i in popup.item_count:
		if not popup.is_item_separator(i) and popup.get_item_id(i) == 0:
			return true
	return false

## `about_to_popup` handlers (by method name) that DO things beyond refreshing a
## row's enabled state, so a chip must never stand in for opening their menu.
## Both are named in this file's header: `_refresh_atlas_cache_menu` calls
## `_enforce_atlas_cap()` (evicts baked chunks) and `_on_gpu_devices_about_to_popup`
## enumerates GPU adapters (the crash `menus.gd::_build_gpu_devices_menu`
## documents). Labelled judgement from reading the handlers, 2026-10-25; a lambda
## handler cannot be read by name and is counted by `_open_handlers()` instead.
const UNSAFE_OPEN_HANDLERS: Array = ["_refresh_atlas_cache_menu", "_on_gpu_devices_about_to_popup"]

## Names of the `about_to_popup` handlers that a popup carries beyond the ones every
## popup has. Dropped as baseline: Godot's own `MenuButton::_popup_visibility_changed`
## and `ColorPicker::_block_input_on_popup_show`, and ONE non-capturing anonymous
## lambda, which is `dcc_widgets.gd::style_popup`'s re-style connection (line
## `popup.about_to_popup.connect(func() -> void: style_popup(popup))`) that every
## themed popup gets. Anything left is a menu-specific handler. Limitation: a
## second non-capturing lambda would be miscounted as baseline; the probe lists
## the handlers per popup so that is visible, not assumed.
func _open_handlers(popup: PopupMenu) -> Array:
	var out: Array = []
	var dropped_style := false
	for c in popup.about_to_popup.get_connections():
		var cb: Callable = c["callable"]
		var nm := String(cb.get_method())
		var txt := str(cb)
		if nm == "_popup_visibility_changed" or nm == "_block_input_on_popup_show":
			continue
		if not dropped_style and txt.ends_with("(lambda)"):
			dropped_style = true
			continue
		out.append(nm if nm != "" else txt)
	return out

## Why a menu row cannot be pinned in v1, or "" when it can. Reads only state
## the menu was BUILT in (never emits `about_to_popup`; see the header). A popup
## whose only open-time handlers just refresh enabled state (Story, World, Edit...)
## stays pinnable: the chip then stays enabled and the shared handler reports
## "nothing to do" itself (CUSTOM_QUICK_BAR_SCOPE.md s.3.3, availability).
func _why_not_pinnable(popup: PopupMenu, i: int, id: int, is_readout: bool, menu_name: String, has_state: bool = false) -> String:
	if menu_name == "":
		return "Not an app menu: an embedded control's own popup (a ColorPicker's presets)."
	if is_readout:
		return "A readout shows a value; there is nothing to run."
	if popup.is_item_disabled(i):
		return "Not available yet: " + popup.get_item_tooltip(i)
	if popup.is_item_checkable(i) and not has_state:
		return "A check or radio row mirrors state a chip cannot read without opening the menu, and no live state source is registered for this one."
	if _is_dynamic_id(id) or _is_index_popup(popup):
		return "A generated row (a ladder or list); its meaning changes with the data."
	for h in _open_handlers(popup):
		if UNSAFE_OPEN_HANDLERS.has(String(h)):
			return "Its menu runs " + String(h) + " when opened (it evicts or enumerates); a chip cannot repeat that safely."
	return ""

## Makes `pin_key` unique among pinnable rows. First, a later row with the SAME id
## and the SAME title as an earlier one is the same command shown in two menus
## (`Places without notes` is in both World and Story): it becomes an unpinnable
## alias of the first so the Customise list shows the command once. Then any
## remaining `menu:<id>` shared by rows with different titles becomes
## `menu:<group>/<id>`. Returns that second count (0 on the tree measured by
## `_cmdindex_probe.gd`; the disambiguator exists for the day it is not).
func _disambiguate_pin_keys() -> int:
	var first := {}
	for r in _rows:
		if not bool(r.get("pinnable", false)):
			continue
		var sig := "%s|%s" % [String(r["pin_key"]), String(r["title"])]
		if first.has(sig):
			r["pinnable"] = false
			r["why_not"] = "The same command as %s ▸ %s (same id, same title)." % [String(first[sig]["group"]), String(r["title"])]
		else:
			first[sig] = r
	var seen := {}
	for r in _rows:
		if bool(r.get("pinnable", false)):
			var k := String(r["pin_key"])
			seen[k] = int(seen.get(k, 0)) + 1
	var collisions := 0
	for r in _rows:
		if bool(r.get("pinnable", false)) and int(seen[String(r["pin_key"])]) > 1:
			collisions += 1
			r["pin_key"] = "menu:%s/%d" % [String(r["group"]).to_lower(), int(r["_id"])]
	return collisions

## Runs a pinnable row by emitting the owning popup's own `id_pressed`, the
## signal its real handler is connected to, so a chip and a click cannot diverge.
## Returns false (and does nothing) for an unpinnable row or a freed popup.
func run_row(row: Dictionary) -> bool:
	if not bool(row.get("pinnable", false)):
		return false
	return _fire(row)

## The one place a menu or tool row is performed: a TOOL row arms through `DccApp.arm_tool`, the very call
## the palette's button makes (CQ-8, scope Q10: never a second Measure); any other emits the owning popup's
## `id_pressed`. `run_row()` (a chip) and `run_search()` (a result) both end here.
func _fire(row: Dictionary) -> bool:
	if String(row.get("kind", "")) == "tool":
		if _app == null or not _app.has_method("arm_tool"):
			return false
		_app.arm_tool(String(row["_tool"]))
		return true
	var popup = row.get("_popup")
	if popup == null or not is_instance_valid(popup):
		return false
	(popup as PopupMenu).id_pressed.emit(int(row["_id"]))
	return true

# -- MS-9: search runs every command ---------------------------------------------------------------
#
# `MOBILE_SHELL_PROPOSAL.md` MS-9, "Search runs every command -- a check, radio or parameter result does what
# its row does". A quick-bar chip may only repeat a command whose effect it can show without opening a menu
# (`pinnable`); a search row has no such limit, because the list is rebuilt when the dialog opens and a press
# can show the new state in the row itself. So the search path has its own gate, `search_block()`, and its own
# entry, `run_search()`, and both end in the SAME `_fire()` as `run_row()`: the owning popup's `id_pressed`,
# the signal the menu's handler is connected to. Nothing here implements a command a second time.

## `row_state()`'s answer for a row with no on/off state (a plain command, a parameter, a readout). A value
## outside {0, 1} on purpose: a "no state" encoded as 0 would read as "off" (`MISTAKES.md`, "Return, marshal or
## default a value that can be absent").
const STATE_NONE := -1

## 1 when the check or radio is on, 0 when off, `STATE_NONE` when the row has no such state. A registered
## `_state` source wins (it reads the app's own value); otherwise the owning popup's item bit, which
## `refresh_states()` and `run_search()` keep current by running the menu's own open handlers.
func row_state(row: Dictionary) -> int:
	var src = row.get("_state")
	if src is Callable and (src as Callable).is_valid():
		return 1 if bool((src as Callable).call()) else 0
	if not bool(row.get("checkable", false)):
		return STATE_NONE
	var popup = row.get("_popup")
	if popup == null or not is_instance_valid(popup):
		return STATE_NONE
	var i := _item_of(row)
	if i < 0:
		return STATE_NONE
	return 1 if (popup as PopupMenu).is_item_checked(i) else 0

## The words a search row shows for `row_state()`: "On"/"Off" for a check, "Selected"/"Not selected" for a
## radio, "" for a row with no state (so the caller adds nothing rather than a guess).
func state_word(row: Dictionary) -> String:
	var st := row_state(row)
	if st == STATE_NONE:
		return ""
	if bool(row.get("radio", false)):
		return "Selected" if st == 1 else "Not selected"
	return "On" if st == 1 else "Off"

## The index of the popup item this row was built from, or -1 when the menu no longer has an item with the
## row's id AND text (a generated list was rebuilt since the index was). Id alone is not enough: the
## index-popups number their items from 0, so an id outlives the row it named.
func _item_of(row: Dictionary) -> int:
	var popup = row.get("_popup")
	if popup == null or not is_instance_valid(popup):
		return -1
	var p := popup as PopupMenu
	var want_text := String(row.get("_text", ""))
	for i in p.item_count:
		if p.get_item_id(i) == int(row.get("_id", -1)) and p.get_item_text(i) == want_text:
			return i
	return -1

## True when `popup` holds a check or radio whose state lives only on the item (no `state_source` registered, not a
## readout), i.e. the one kind of row `row_state()` reads off the item bit and so needs the menu's refreshers run first.
func _needs_item_state(popup: PopupMenu) -> bool:
	for i in popup.item_count:
		if popup.is_item_separator(i) or not popup.is_item_checkable(i):
			continue
		var meta = popup.get_item_metadata(i)
		if typeof(meta) == TYPE_STRING and String(meta) == DccMenus.META_READOUT:
			continue
		if not _state_source_for(popup.get_item_id(i)).is_valid():
			return true
	return false

## Runs the menu's own about-to-open refreshers for every popup that holds a check or radio with no state
## source, so the item bits `row_state()` reads are what the menu would show if it were opened now. Without
## it a check changed from somewhere else (a window closed by its own X) reads as it was last drawn.
## `build(app, bridge, true)` does this before reading each popup; this is the same step on an index already
## built (a probe re-reads state after a press). Skips what
## `_open_handlers()` skips plus the lambdas (anonymous handlers carry no name to vet) and the handlers
## `UNSAFE_OPEN_HANDLERS` names (they evict or enumerate). Returns the number of popups refreshed.
func refresh_states() -> int:
	var seen := {}
	for r in _rows:
		if not bool(r.get("checkable", false)) or (r.get("_state") is Callable and (r["_state"] as Callable).is_valid()):
			continue
		var popup = r.get("_popup")
		if popup == null or not is_instance_valid(popup) or seen.has((popup as Object).get_instance_id()):
			continue
		seen[(popup as Object).get_instance_id()] = true
		_refresh_popup(popup as PopupMenu)
	return seen.size()

func _refresh_popup(popup: PopupMenu) -> void:
	for c in popup.about_to_popup.get_connections():
		var cb: Callable = c["callable"]
		var nm := String(cb.get_method())
		if nm == "_popup_visibility_changed" or nm == "_block_input_on_popup_show" or UNSAFE_OPEN_HANDLERS.has(nm):
			continue
		if str(cb).ends_with("(lambda)") or nm.begins_with("<anonymous") or not cb.is_valid():
			continue
		cb.call()

## Why a search press on `row` would not run it, or "" when it would. The reasons, in order:
## a pointer row (`EXTRAS`) is not a command; a readout has nothing to run; an unavailable row says the menu's
## own reason (`why`, the item's tooltip) -- read from the LIVE item when there is one, since the world may have
## changed since the list was built; and a menu row whose item is gone says so. A parameter's `why` ("Takes
## effect on the next Generate") is a note, not a block, so availability decides, never `why` alone.
func search_block(row: Dictionary) -> String:
	var kind := String(row.get("kind", ""))
	if kind == "action":
		return "%s (This row points to where the command lives; it does not run.)" % String(row.get("blurb", "This is a pointer, not a command."))
	if kind == "readout":
		return "%s is a readout: it shows a value and there is nothing to run." % String(row.get("title", "This row"))
	if not bool(row.get("available", false)):
		var why := String(row.get("why", ""))
		return why if why != "" else "This command is not available right now."
	if kind == "menu":
		var i := _item_of(row)
		if i < 0:
			return "That menu changed since this list was built. Search again."
		var p := row["_popup"] as PopupMenu
		if p.is_item_disabled(i):
			var tip := p.get_item_tooltip(i)
			return tip if tip.strip_edges() != "" else "This command is not available right now."
	return ""

## The search press: "" when `row` ran, else the reason it did not. A menu or tool row goes through `_fire()`
## (`run_row()`'s own tail); a checkable one is then re-read with its menu's refreshers so `row_state()` shows
## the result; a parameter row is handed to the app's `reveal_param()`, which opens and focuses its control.
func run_search(row: Dictionary) -> String:
	var why := search_block(row)
	if why != "":
		return why
	match String(row.get("kind", "")):
		"param":
			if _app == null or not _app.has_method("reveal_param"):
				return "This build has no parameter dock to open."
			return String(_app.call("reveal_param", String(row.get("key", ""))))
		"tool":
			return "" if _fire(row) else "Could not arm that tool."
		"menu":
			if not _fire(row):
				return "Could not run it."
			if bool(row.get("checkable", false)):
				_refresh_popup(row["_popup"] as PopupMenu)
			return ""
	return "This row cannot be run from search."

## The live row for a stored `pin_key`, or an empty Dictionary when the command
## is gone from this build (callers show "not in this build"; never delete it).
func find_pin(pin_key: String) -> Dictionary:
	for r in _rows:
		if String(r.get("pin_key", "")) == pin_key:
			return r
	return {}

## `get_children(true)` -- a MenuButton keeps its popup as an INTERNAL child,
## so the default walk finds none of them. `_loddbg_probe.gd` cost a run
## learning that.
func _gather_menu_buttons(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather_menu_buttons(c, out)

## Number of indexed rows.
func size() -> int:
	return _rows.size()

## A copy of every row in build order, so a caller cannot mutate the index.
func all() -> Array:
	return _rows.duplicate()

## Substring match over title, blurb and group, case-insensitive.
##
## Deliberately not fuzzy. A searcher typing "riv" wants every river row, and a
## fuzzy matcher that also returns "Drive" and "Arrival" makes the list *less*
## trustworthy -- the whole point here is that what you see is what matched.
## Ordered: title matches first, then group, then blurb, each alphabetically,
## so the strongest reason a row is present is the reason it sorts.
func search(q: String) -> Array:
	var needle := q.strip_edges().to_lower()
	if needle == "":
		return _rows.duplicate()
	var t: Array = []
	var g: Array = []
	var b: Array = []
	for r in _rows:
		if String(r["title"]).to_lower().find(needle) >= 0:
			t.append(r)
		elif String(r["group"]).to_lower().find(needle) >= 0:
			g.append(r)
		elif String(r["blurb"]).to_lower().find(needle) >= 0:
			b.append(r)
	var by_title := func(a, c): return String(a["title"]) < String(c["title"])
	t.sort_custom(by_title)
	g.sort_custom(by_title)
	b.sort_custom(by_title)
	return t + g + b

## Distinct group names in the order they first appear, so a caller can band
## the list the way the phone canvas bands its own.
func groups() -> Array:
	var seen: Array = []
	for r in _rows:
		if not seen.has(r["group"]):
			seen.append(r["group"])
	return seen

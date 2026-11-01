## DRAFT, not loaded by Godot. NOT parse-checked (no Godot binary in the session that wrote it).
##
## PC_SHELL_PROPOSAL.md §3.3, §3.6 and §3.8, the Window rows (milestone PC-9):
##   - "Right dock" becomes "Context panel", with a submenu under it: Inspect, Note and Ask (Ctrl+1,
##     Ctrl+2, Ctrl+3), Pin to the current subject, Follow the selection. Drawn as the mockup draws it:
##     the region's check row AND a submenu of the same name directly under it
##     (`design/mobile-shell-2026-10-31/Shell.dc.html`, the desktop Window menu).
##   - Open windows lists everything open, the Context panel and pop-outs included (finding W5).
##   - **Ruling DY (owner, 2026-11-01; PQ1 "1, but it should be modular", then "Tabs you arrange"):** the
##     submenu gains a MODULES band of ten check rows, one per module the owner chose (History; Economy;
##     Military; Faith, culture & relations; Territory; City layout; Here; Routes & journeys; Links; Recent &
##     bookmarks), checked when the module is in the panel, then Reset panel (back to Inspect · Note · Ask).
##     Ctrl+1/2/3 stay Inspect, Note and Ask whatever order the tabs are in; an added module gets no default
##     key. The band and Reset panel are not built on the phone, which has no panel (§3.11; Ruling DX PQ13's
##     five rows stay). The tab strip's own + button, drag to reorder and a tab's Remove and Open in a window
##     are the panel's (PC-4), not the menu's.
##
## Target: `shell/menus.gd`, the Window section. Depends on `01_menus_ids_helpers.gd`.
## Applies to: every form factor: the PC, the tablet (☰ ▸ Window below 1 180 dp) and the phone (MENU ▸
## Window). **The phone was excluded until Ruling DX PQ13** (2026-11-01, decided under the owner's
## delegation): its Window menu follows the mockup, with the "Context panel" check and its submenu and no
## separate "Right dock" row. On the phone the rows open what the phone has, never a panel (§3.11, "the
## model, not the panel"): the check shows or hides the selection sheet (the right dock as a sheet, which
## `app.gd::toggle_region(ID_WIN_RIGHT)` already does there); Inspect opens that sheet; Note opens the
## vault at the selection's note; Ask opens the assistant. `90_pc_milestone_stubs.gd` states those routes
## as `open_context_tab()`'s phone contract, so the rows switch on there when PC-4/5/6 write them.
##
## Forms:
##   before PC-4  the region row keeps "Right dock"; the submenu exists with five `_todo` rows, each
##                with its reason (the menu is the map of what the port still owes: this file's
##                header). Ctrl+1/2/3 are not bound.
##   from PC-4    (`context_tabs()` has "inspect") the region row reads "Context panel" (here and in
##                the Save layout modal, through `_region_label()`); Inspect, Pin and Follow are live,
##                and Open windows lists the panel while it is shown.
##   from PC-5    Note is live (Ctrl+2).        from PC-6: Ask is live (Ctrl+3).
##   modules      each MODULES row is a `_todo` naming where that content lives today, until the host's
##                `context_module_registry()` lists the module; then a check row, ticked from
##                `context_panel_modules()` on every popup. Reset panel is a `_todo` until the host has
##                `reset_context_panel()` (PC-4). Not built on the phone (Ruling DY).
##   always       Open windows walks every visible window, docked or popped out (no dependency).
##
## What to do. Each EDIT quotes its old text exactly as it stands at `a38e0ac`.
##   EDIT N1  `_window()`: replace the three lines
##                for rid in WIN_REGION_IDS:
##                    p.add_check_item(String(WIN_REGION_LABELS[rid]), rid)
##                    p.set_item_checked(p.item_count - 1, true)
##            with
##                for rid in WIN_REGION_IDS:
##                    p.add_check_item(_region_label(rid), rid)
##                    p.set_item_checked(p.item_count - 1, true)
##                    ## PC-9 (§3.3, §3.8): the Context panel's submenu sits directly under
##                    ## its region row, as the mockup draws it, on every form factor (Ruling DX PQ13).
##                    if rid == ID_WIN_RIGHT:
##                        _build_context_panel_menu(p)
##            (one tab before `for`, two inside, three under the `if`). Every row index `_window()`
##            captures later (`sess_idx`, `diag_idx`, the 3D rows) is taken after this loop, and every
##            region lookup is by id (`get_item_index`), so the extra row moves nothing.
##   EDIT N2  `_layout_readout()`: replace
##                var label := String(WIN_REGION_LABELS[rid])
##            with
##                var label := _region_label(rid)
##   EDIT N3  the declaration `var _open_windows: Array = []`: replace its trailing comment
##                ## the live `AcceptDialog`s behind `_windows_popup`, index-parallel to its items
##            with
##                ## what each `_windows_popup` row raises, index-parallel to its items: an open `Window`, or `CONTEXT_PANEL_LABEL` (a String) for the Context panel
##   REPLACE  `_refresh_open_windows()` and `_on_open_window()`, each with its doc, with the versions below.
##   ADD      the rest below, beside `WIN_REGION_LABELS`.
##   PROSE    `ID_WIN_RIGHT`'s comment ("Window region toggle: Right dock."): add "titled Context
##            panel on desktop and tablet from PC-4 (`_region_label()`)". `WIN_REGION_LABELS`' doc ("The
##            wording of those five rows, in one place"): the one exception is `_region_label()`. The
##            orphaned doc above the 3D-1 block ("Fills the Window menu `p`: … (labels from
##            `WIN_REGION_LABELS`)"): labels from `_region_label()`.
##
## Verify: the parse check on `res://shell/menus.gd`; `check_symbols.sh`. Probes: `_seedlayout_probe.gd`
## (the readout's five labels; unchanged until PC-4, when "Right dock" becomes "Context panel"),
## `_appbar20_probe.gd` (the phone's "Right dock" row: unchanged until PC-4, then "Context panel" there
## too), `_sessionmode_probe.gd` and
## `_leftdockcheck_probe.gd` (region checks by id: unchanged), `_menuwire_probe.gd` (every disabled row
## states a reason: the sixteen `_todo` rows do). By hand, desktop: Window shows Left dock, Right dock,
## Context panel ▸ (five dark rows with reasons, then MODULES with ten dark rows, each naming where its
## content lives today, then a dark Reset panel), Timeline…; pop out the faction roster (Preferences ▸
## Open panels in their own window), then Window ▸ Open windows lists "Factions   own window" and
## picking it focuses that window without moving it. Phone: MENU ▸ Window ▸ Context panel has the five
## rows and no MODULES band.

## The name the right dock takes once it is the Context panel (`PC_SHELL_PROPOSAL.md` §3.3, PC-4),
## used for its region row, its submenu and its Open windows row. One string, so the three cannot
## drift apart.
const CONTEXT_PANEL_LABEL := "Context panel"

## Why Window ▸ Context panel ▸ Inspect is disabled before PC-4. Checked 2026-11-01 at `a38e0ac`:
## `right_dock.gd` holds one `_context` at a time and builds no tabs (`grep -c 'TabBar\|TabContainer'
## shell/right_dock.gd` is 0).
const CTX_INSPECT_TODO := "Not built yet: the right dock shows one thing at a time and has no tabs. Inspect, Note and Ask arrive when it becomes the Context panel (PC_SHELL_PROPOSAL.md §3.3, milestone PC-4)."
## Why Note is disabled before PC-5. Points at the row that does the nearest thing today, which is
## Story's shipped form until PC-5 (`04_menus_story.gd`).
const CTX_NOTE_TODO := "Not built yet: the Note tab arrives with milestone PC-5 (PC_SHELL_PROPOSAL.md §3.4). Meanwhile Story ▸ Notes for this place opens a settlement's note in the vault window."
## Why Ask is disabled before PC-6. Points at Story's shipped assistant row.
const CTX_ASK_TODO := "Not built yet: the Ask tab arrives with milestone PC-6 (PC_SHELL_PROPOSAL.md §3.5). Meanwhile Story ▸ Assistant panel… opens the assistant in its own window."
## Why Pin is disabled before PC-4. `right_dock.gd` has no pinned state of its own: its "pin" words
## are map pins and the settlement it shows, which every pick replaces.
const CTX_PIN_TODO := "Not built yet: the right dock always shows the latest pick and cannot hold one subject while you select another. Pinning arrives with the Context panel (PC_SHELL_PROPOSAL.md §3.3, milestone PC-4)."
## Why Follow is disabled before PC-4: with no pin there is nothing to release.
const CTX_FOLLOW_TODO := "Not built yet: there is no pin to release, because the right dock always follows the latest pick. It arrives with the Context panel (PC_SHELL_PROPOSAL.md §3.3, milestone PC-4)."

## The five rows' tooltips once each is live, from §3.3 and §3.5. `CTX_NOTE_TIP` follows Ruling DY's VQ4
## (2026-11-01, "Open in Write"): a note opens in Write, with Preview a toggle. It said "in Preview", the
## proposal's default before that ruling; if the Note tab is ruled to differ from the vault editor, this
## string changes with it.
const CTX_INSPECT_TIP := "Shows the Context panel's Inspect page: the selection, or what the panel is pinned to, with its facts and history. Every name there selects it; every year moves the timeline."
const CTX_NOTE_TIP := "Shows the Context panel's Note tab: the subject's own note, open in Write as every note opens. Preview (Ctrl+E) renders it."
const CTX_ASK_TIP := "Shows the Context panel's Ask tab with the cursor in its input. Off until you turn the assistant on in Preferences ▸ Assistant."
const CTX_PIN_TIP := "Keeps the panel on what it shows now while you select other things. A banner then offers to pin it to the new selection or to follow again."
const CTX_FOLLOW_TIP := "Releases a pin: the panel shows the selection again and follows each new one."
## Status line when Pin finds nothing to hold (`pin_context_panel()` answered false).
const CTX_PIN_NOTHING := "Nothing to pin: the panel is empty. Select something first."

## **Ruling DY's modules** (owner, 2026-11-01): `[key, title, id]` for each of the ten the owner chose, in the
## order the ruling lists them. The key is the module's id in the panel's registry and in the user's stored
## arrangement (`DccApp.context_module_registry()`, `context_panel_modules()`, `90_pc_milestone_stubs.gd`),
## the title is its row's and its tab's label, and the id is its row's (`01_menus_ids_helpers.gd`).
## Append-only: a key, an id and a title's meaning are a module's identity (a rebind is keyed by the id, a
## saved arrangement by the key), so none is ever reordered, reused or renamed in place. The registry must
## use these keys and titles, never a second spelling of them. "Compare" was offered and not chosen.
const CTX_MODULES: Array = [
	["history", "History", ID_WIN_CTX_MOD_HISTORY],
	["economy", "Economy", ID_WIN_CTX_MOD_ECONOMY],
	["military", "Military", ID_WIN_CTX_MOD_MILITARY],
	["faith", "Faith, culture & relations", ID_WIN_CTX_MOD_FAITH],
	["territory", "Territory", ID_WIN_CTX_MOD_TERRITORY],
	["city", "City layout", ID_WIN_CTX_MOD_CITY],
	["here", "Here", ID_WIN_CTX_MOD_HERE],
	["routes", "Routes & journeys", ID_WIN_CTX_MOD_ROUTES],
	["links", "Links", ID_WIN_CTX_MOD_LINKS],
	["recent", "Recent & bookmarks", ID_WIN_CTX_MOD_RECENT],
]

## Why a MODULES row is disabled before its module is built: `%s` is the module's title, then where that
## content lives today (`CTX_MODULE_TODAY`), so the reason is a door, not only a "not yet".
const CTX_MODULE_TODO := "Not built yet: %s arrives as a tab of the Context panel (PC_SHELL_PROPOSAL.md §3.3, Ruling DY; milestone PC-4). %s"

## Where each module's content lives today, each read at its builder on 2026-11-01 at `3099309`
## (`90_pc_milestone_stubs.gd::context_module_registry()` names the symbols). Recent & bookmarks has no
## home today and says so; a "Meanwhile" that pointed nowhere would be a dead control's promise.
const CTX_MODULE_TODAY: Dictionary = {
	"history": "Meanwhile the Place editor's Timeline and Political history tabs, and the Factions window's History tab.",
	"economy": "Meanwhile the Place editor's Economy & notables tab, and the Factions window's Economy tab.",
	"military": "Meanwhile the Factions window's Military tab; a selected place's garrison is in the right dock.",
	"faith": "Meanwhile the Factions window's Relations tab (relations and faith) and its Identity tab (culture).",
	"territory": "Meanwhile the Factions window's Territory tab.",
	"city": "Meanwhile the Place editor's Layout tab, and the City viewer.",
	"here": "Meanwhile the right dock's cursor sample, its default view, reads the terrain under the cursor.",
	"routes": "Meanwhile the right dock shows a picked route, and Story ▸ Journey planner and Saved journeys… plan and list journeys.",
	"links": "Meanwhile the Place editor's Vault notes tab and the Markdown vault window list a note's backlinks.",
	"recent": "Nothing shows the selection history yet: it arrives with one selection (milestone PC-1), and bookmarks are new.",
}

## Each MODULES row's tooltip once its module is built: what the tab shows, in the words of the owner's
## options (Ruling DY), so the row says what ticking it adds.
const CTX_MODULE_TIPS: Dictionary = {
	"history": "Adds a History tab: who held a place over time and its timeline, a faction's history, and the dated entries in the subject's note. Every year moves the timeline.",
	"economy": "Adds an Economy tab: a place's economy and notables, or a faction's economy, currency and tariffs.",
	"military": "Adds a Military tab: a faction's armies, garrisons and conflicts, or whether a place is fortified and its garrison.",
	"faith": "Adds a Faith, culture & relations tab: a faction's relations, its faith and its culture.",
	"territory": "Adds a Territory tab: a faction's claims, provinces and influence.",
	"city": "Adds a City layout tab: a settlement's layout, as the City viewer draws it.",
	"here": "Adds a Here tab: the terrain under the subject, or under the cursor while nothing is selected (elevation, climate, biome).",
	"routes": "Adds a Routes & journeys tab: the roads from the subject, saved journeys through it, and Plan a journey from here.",
	"links": "Adds a Links tab: the notes that link to the subject, and unlinked mentions with a one-click Link.",
	"recent": "Adds a Recent & bookmarks tab: the selection history as a list, and the subjects you bookmark.",
}

## Why Reset panel is disabled before PC-4: there is no arrangement to reset.
const CTX_RESET_TODO := "Not built yet: there are no tabs to rearrange. It arrives with the Context panel (PC_SHELL_PROPOSAL.md §3.3, Ruling DY; milestone PC-4)."
## Reset panel's tooltip once it is live.
const CTX_RESET_TIP := "Puts the Context panel back to its three tabs, Inspect · Note · Ask, in that order. Added modules are removed; their content is not touched."
## Status line when a module could not be added or removed (`set_context_module_shown()` answered false):
## the one refusal its contract allows for a listed module, which is leaving the panel with no tab.
const CTX_MODULE_REFUSED := "The Context panel keeps at least one tab. Add another before removing this one."

## The suffix an Open windows row carries when its window is popped out into its own OS window
## (`Window.is_embedded()` false). The `PREF_VALUE_SEP` spacing, the house way of adding a second
## column a `PopupMenu` cannot draw.
const OPEN_WINDOW_NATIVE_SUFFIX := "   own window"
## Open windows' empty state. The shipped readout said "none of the dialogs this menu tracks is
## open"; it tracks every window now.
const OPEN_WINDOWS_NONE_TIP := "Nothing to raise: no window is open. It fills in as windows are opened, popped-out ones included, and picking a row brings that window to the front."

## The label of region row `rid`: `WIN_REGION_LABELS`' wording, except that the right dock is called
## the Context panel once it is one (PC-4), on every form factor: on the phone the row then names the
## selection sheet (Ruling DX PQ13; it read "Right dock" there until that ruling). `_window()` and
## `_layout_readout()` both read this, so the menu and the Save layout modal cannot name the region
## differently. Must never rename a region the build does not have: the test is the panel's own Inspect
## tab (on the phone, its route to the sheet), not the proposal.
func _region_label(rid: int) -> String:
	if rid == ID_WIN_RIGHT and _context_tab_ready("inspect"):
		return CONTEXT_PANEL_LABEL
	return String(WIN_REGION_LABELS[rid])

## Window ▸ Context panel ▸ (§3.3, §3.8): Inspect, Note, Ask (assistant) with Ctrl+1/2/3, then Pin to
## the current subject and Follow the selection, in the mockup's order and words. A submenu of its
## own (node `ContextPanelMenu`, free in `shell/` at a38e0ac), so its ids never reach the Window popup's
## `toggle_region` handler.
##
## Each row is live only when the panel has what it calls (`_context_tab_ready()`; Pin and Follow also
## need the panel itself, so they test Inspect, the panel's first tab), and a `_todo` with its reason
## otherwise. Ctrl+1, Ctrl+2 and Ctrl+3 were free: no row binds a digit, and the Layers popover's 1-8
## answer only while it is open (`layers_popover.gd::_input` returns while hidden). **While the popover
## is open, Ctrl+1-3 pick a layer**: its `is_action_pressed()` uses the default `exact_match = false`, so
## an unmodified action matches a Ctrl chord too. That is the popover's own scope and is left alone.
##
## Built on every form factor. On the tablet below 1 180 dp it is ☰ ▸ Window ▸ Context panel ▸. On the
## phone (Ruling DX PQ13) it is MENU ▸ Window ▸ Context panel ▸, its rows reaching the selection sheet, the
## vault and the assistant through `open_context_tab()`'s phone contract, and its Ctrl+1/2/3 never fire
## there (the phone's menu bar is a hidden model, so no accelerator does).
##
## **Ruling DY (2026-11-01): then a MODULES band** of ten rows (`CTX_MODULES`, `_build_context_module_row()`)
## and Reset panel, on the desktop and the tablet only. Ctrl+1/2/3 stay on Inspect, Note and Ask whatever the
## order of the tabs (the owner's "tabs you arrange" moves tabs, not keys); a module gets no default key, and
## its row is rebindable by id like any other. The ticks are read on every popup
## (`_refresh_context_modules()`), because the tab strip's own + and Remove change them while this menu is shut.
func _build_context_panel_menu(p: PopupMenu) -> void:
	var sub := PopupMenu.new()
	sub.name = "ContextPanelMenu"
	_shell.style_popup(sub)
	p.add_child(sub)
	p.add_submenu_item(CONTEXT_PANEL_LABEL, sub.name)
	if _context_tab_ready("inspect"):
		_live(sub, "Inspect", ID_WIN_CTX_INSPECT, KEY_MASK_CTRL | KEY_1)
		sub.set_item_tooltip(sub.item_count - 1, CTX_INSPECT_TIP)
	else:
		_todo(sub, "Inspect", CTX_INSPECT_TODO)
	if _context_tab_ready("note"):
		_live(sub, "Note", ID_WIN_CTX_NOTE, KEY_MASK_CTRL | KEY_2)
		sub.set_item_tooltip(sub.item_count - 1, CTX_NOTE_TIP)
	else:
		_todo(sub, "Note", CTX_NOTE_TODO)
	if _context_tab_ready("ask"):
		_live(sub, "Ask (assistant)", ID_WIN_CTX_ASK, KEY_MASK_CTRL | KEY_3)
		sub.set_item_tooltip(sub.item_count - 1, CTX_ASK_TIP)
	else:
		_todo(sub, "Ask (assistant)", CTX_ASK_TODO)
	sub.add_separator()
	if _context_tab_ready("inspect") and _host_has("pin_context_panel"):
		_live(sub, "Pin to the current subject", ID_WIN_CTX_PIN)
		sub.set_item_tooltip(sub.item_count - 1, CTX_PIN_TIP)
	else:
		_todo(sub, "Pin to the current subject", CTX_PIN_TODO)
	if _context_tab_ready("inspect") and _host_has("follow_selection"):
		_live(sub, "Follow the selection", ID_WIN_CTX_FOLLOW)
		sub.set_item_tooltip(sub.item_count - 1, CTX_FOLLOW_TIP)
	else:
		_todo(sub, "Follow the selection", CTX_FOLLOW_TODO)
	## Ruling DY: the modules, on the desktop and the tablet. Not on the phone, which has no panel to add a
	## tab to (§3.11, "the model, not the panel"); its five rows above are Ruling DX PQ13's and stay.
	if not DccTheme.is_phone():
		sub.add_separator("MODULES")
		for m in CTX_MODULES:
			_build_context_module_row(sub, m)
		sub.add_separator()
		if _host_has("reset_context_panel"):
			_live(sub, "Reset panel", ID_WIN_CTX_RESET)
			sub.set_item_tooltip(sub.item_count - 1, CTX_RESET_TIP)
		else:
			_todo(sub, "Reset panel", CTX_RESET_TODO)
		sub.about_to_popup.connect(_refresh_context_modules.bind(sub))
		_refresh_context_modules(sub)
	sub.id_pressed.connect(_on_context_panel)

## Whether module `key` is built in this host: `DccApp.context_module_registry()` lists an entry whose
## "key" is `key`. Asked per module, as `_context_tab_ready()` asks per tab, because the ten modules are
## extracted from their windows one by one and a row must stay a `_todo` until its own module exists.
## Must never infer a module from the panel's other methods: a registry with three entries says nothing
## about the other seven.
func _context_module_ready(key: String) -> bool:
	if not _host_has("context_module_registry"):
		return false
	for e in _host.context_module_registry():
		if typeof(e) == TYPE_DICTIONARY and String((e as Dictionary).get("key", "")) == key:
			return true
	return false

## One MODULES row, `m` being a `CTX_MODULES` entry: a check row with its tooltip once the module is built
## (`_context_module_ready()`), otherwise a `_todo` whose reason names where that content lives today
## (`CTX_MODULE_TODAY`). Its tick is not set here: `_refresh_context_modules()` reads it.
func _build_context_module_row(sub: PopupMenu, m: Array) -> void:
	var key := String(m[0])
	var title := String(m[1])
	if _context_module_ready(key):
		sub.add_check_item(title, int(m[2]))
		sub.set_item_tooltip(sub.item_count - 1, String(CTX_MODULE_TIPS[key]))
	else:
		_todo(sub, title, CTX_MODULE_TODO % [title, String(CTX_MODULE_TODAY[key])])

## Ticks each built module's row when that module is in the user's panel (`context_panel_modules()`), on
## every popup of the submenu and once at build. A row still a `_todo` has no item with the module's id
## (`_todo()` adds by index), so `get_item_index()` answers -1 and it is skipped. Reads only; never adds or
## removes a tab.
func _refresh_context_modules(sub: PopupMenu) -> void:
	if not _host_has("context_panel_modules"):
		return
	var shown: PackedStringArray = _host.context_panel_modules()
	for m in CTX_MODULES:
		var i := sub.get_item_index(int(m[2]))
		if i >= 0 and sub.is_item_checkable(i):
			sub.set_item_checked(i, shown.has(String(m[0])))

## The `CTX_MODULES` key of row `id`, or "" when `id` is not a module row's.
func _context_module_key(id: int) -> String:
	for m in CTX_MODULES:
		if int(m[2]) == id:
			return String(m[0])
	return ""

## Window ▸ Context panel ▸'s dispatcher. Its ids are on live rows only, so the host has each method
## whenever one arrives (a module row is live only once the registry lists it, and
## `90_pc_milestone_stubs.gd` makes the registry, the arrangement and its setter land together). Pin's `false`
## (nothing to hold) says so in the status line. A module row (Ruling DY) flips its module in or out of the
## panel, reading the arrangement at the press, never a tick cached at build; a refusal (the panel's last
## tab) says why in the status line.
func _on_context_panel(id: int) -> void:
	match id:
		ID_WIN_CTX_INSPECT: _host.open_context_tab("inspect")
		ID_WIN_CTX_NOTE: _host.open_context_tab("note")
		ID_WIN_CTX_ASK: _host.open_context_tab("ask")
		ID_WIN_CTX_PIN:
			if not bool(_host.pin_context_panel()):
				_host.set_status("hint", CTX_PIN_NOTHING, "text_ghost")
		ID_WIN_CTX_FOLLOW: _host.follow_selection()
		ID_WIN_CTX_RESET: _host.reset_context_panel()
		_:
			var key := _context_module_key(id)
			if key == "":
				return
			var shown: PackedStringArray = _host.context_panel_modules()
			if not bool(_host.set_context_module_shown(key, not shown.has(key))):
				_host.set_status("hint", CTX_MODULE_REFUSED, "text_ghost")

## Every window open right now: each visible `AcceptDialog` anywhere under `DccApp`, docked or popped
## out, in tree order. Walked rather than listed (finding W5): the six fields this menu named by hand
## missed the faction roster, the Place editor, the vault, the City viewer, the assistant, Conflicts,
## Saved journeys and every pop-out, and a hand list falls behind the next window someone adds.
## `owned = false`, because windows built in code have no owner and the default would find none. A
## pop-out is still a child of `DccApp` (`PopoutHost.prepare()` only sets `force_native`), so the walk
## finds it like any other window.
func _visible_windows() -> Array:
	var out: Array = []
	if _host == null:
		return out
	for n in (_host as Node).find_children("*", "AcceptDialog", true, false):
		if (n as Window).visible:
			out.append(n)
	return out

## The Open windows row for window `w`: its own title (most windows put their subject there,
## "Place — Korrath", "Markdown vault — …"), or its `PopoutHost.key_for()` key made readable when the
## title is empty (a borderless phone window draws its title in its own header instead), plus
## `OPEN_WINDOW_NATIVE_SUFFIX` when it is popped out.
func _open_window_label(w: Window) -> String:
	var t := w.title.strip_edges()
	if t == "":
		t = PopoutHost.key_for(w).replace("_", " ").capitalize()
	if not w.is_embedded():
		t += OPEN_WINDOW_NATIVE_SUFFIX
	return t

## Window ▸ Open windows (WI-03), rebuilt on every popup because it changes between opens: the
## Context panel first while it is shown (from PC-4; §3.6: "lists every open window, the Context panel
## and pop-outs included"), then every window `_visible_windows()` finds. Rows are checked, as they
## always were: an Open windows row is an open thing. Its empty state is a readout, never a silent dark
## row (`_menuwire_probe.gd` checks that every disabled row states a reason).
##
## While an exclusive dialog is open the menu bar takes no input, so in practice this lists pop-outs
## until the context windows stop being exclusive (PC-7, §3.6); the walk needs no change when they do.
## The same test on every form factor (Ruling DX PQ13). On the phone it lists the selection sheet only
## while that sheet is shown, which MENU's own opening ends (`DccShell._close_all_phone_overlays()`), so
## the phone's list shows windows in practice.
func _refresh_open_windows() -> void:
	_windows_popup.clear()
	_open_windows.clear()
	if _context_tab_ready("inspect") and _host.is_region_shown(ID_WIN_RIGHT):
		_windows_popup.add_check_item(CONTEXT_PANEL_LABEL, _open_windows.size())
		_windows_popup.set_item_checked(_windows_popup.item_count - 1, true)
		_open_windows.append(CONTEXT_PANEL_LABEL)
	for w in _visible_windows():
		_windows_popup.add_check_item(_open_window_label(w as Window), _open_windows.size())
		_windows_popup.set_item_checked(_windows_popup.item_count - 1, true)
		_open_windows.append(w)
	if _open_windows.is_empty():
		_readout(_windows_popup, "No windows open", OPEN_WINDOWS_NONE_TIP)

## Brings the picked entry to the front. The Context panel: `open_context_tab("")`, which shows it on
## the tab it already has. A docked window: `popup_centered()`, as shipped (on a visible
## `AcceptDialog` it re-centres and raises it). A popped-out window: `grab_focus()` only, because a
## window the user placed on a second screen must not be dragged back to the centre of the first.
## Ignores an index past the list or a window freed since the menu was built.
func _on_open_window(id: int) -> void:
	if id < 0 or id >= _open_windows.size():
		return
	var entry = _open_windows[id]
	if typeof(entry) == TYPE_STRING:
		_host.open_context_tab("")
		return
	if not is_instance_valid(entry):
		return
	var w := entry as Window
	if w.is_embedded():
		w.popup_centered()
	else:
		w.grab_focus()

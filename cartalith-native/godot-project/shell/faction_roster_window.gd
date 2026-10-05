extends AcceptDialog
class_name FactionRosterWindow

## `civOpenFactionsBtn` → `civFactionsModal` (`_civOpenFactionsModal`,
## reference 16177; `_civRenderFactionList`; `_civPopulateFactionEditor`,
## 16247) -- `PARITY_AUDIT.md` §5 items 9 and 10, `GUI_GAP_REGISTER.md`
## CV-07 / MS-13.
##
## The register said add/remove faction was absent and `CIV_FACTION_COUNT`
## "a compile-time constant … so there is no roster to add to or remove
## from -- get_factions() enumerates a fixed set, it does not own one." That
## is no longer true: `CivData::faction_roster` owns one, and this window is
## the reference's three-part modal over it -- world overview, faction list,
## and the Inspector drawer with its five editable fields, its procedural
## banner, its Territory-fit verdict and its settlement sublist (each now on its
## own tab, see above) -- plus, since
## 2026-09-27, the faction's currency (Ruling AU; `_build_currency`).
##
## ## The Faction hub's tabbed shell (FH-0, `FACTION_HUB_DESIGN.md`)
##
## The inspector used to be one long scroll of eight sections. It is now seven
## tabs over the selected faction -- Identity, Territory, Settlements,
## Economy, Military, Relations, History (`TAB_IDS`) -- and the sections moved
## **unchanged**, only to the tab that owns them. Relations and History are
## honest placeholders: nothing in the engine backs them yet, so they say so in
## words and carry no control.
##
## **Lazy by tab.** Only the active tab is built, on its first show, and the two
## O(cells) engine passes now run on the tab that reads them
## (`_ensure_fits()` for Territory, `_ensure_military()` for Military) instead of
## in `open()`. A hidden tab is a built pane that is merely not visible; any
## change that could make it stale (a different faction, a new world, an edit to
## culture/government/ag. tech, a roster add/remove) frees every pane but the
## visible one (`_drop_panes`) and the next show rebuilds it.
##
## **FR-02 holds across tab switches.** `_select_tab()` flushes the focused
## field *before* the tab changes and every teardown goes through `_clear()` /
## `_drop_panes()`, both of which raise `_rebuilding` so a dying focused field
## cannot write its text into whichever faction is current by then.
##
## **Phone.** A phone gets a segmented chooser (a 4 + 3 grid of 44 px cells)
## rather than a `TabContainer` or a scrolling strip: seven labels do not fit
## one row at 393 dp, a horizontally scrolling strip hides tabs behind a drag,
## and the window keeps exactly one scroller (below).
##
## **The Culture profiles window is folded in (FH-2, owner decision 2,
## 2026-10-05).** `culture_profiles_window.gd` is retired: its profile, "Name
## pool -- real settlements" (with the reroll chips) and "Assigned factions" now
## hang off the Culture picker on the Identity tab (`_build_culture_card`). The
## card follows the faction's current culture; the one thing it gives up is
## browsing a culture NO faction has -- the dock's Culture category still lists
## all seven read-only.
##
## ## What is real, and what is not
##
## Real: name/culture/religion/government/ag-tech editing (all five persist
## and all five are validated against the engine's own vocabularies), the
## currency's name/symbol/rate (user-set, display only -- `_build_currency`),
## add/remove faction (with the reference's own revert-to-Unclaimed side
## effect), the procedural banner (a port of `_civFactionBannerCanvas`'s
## actual composition, not a redesign), Territory fit (a real
## `civ_culture_terrain_fit` verdict over a real `civ_faction_aggregates`
## terrain mix), settlement count / population / territory km² / capital.
##
## Not built, and said so in-window rather than only here: the reference's
## **Power breakdown** (five axes) and **Economy** block (food production,
## tax, trade, exports/imports, strategic resources, craft share). Both come
## from `_civFactionAggregates`' resource- and density-fed half, and
## `compute_civilisation` frees the resource rasters and never retains a
## population-density field for this -- surfacing them means a memory
## decision (`MEMORY_OPTIMIZATION_SCOPE.md`) and an `ECONOMY_SCOPE.md`
## milestone, not a widget. **Diplomacy** has no model at all, in either
## codebase; the reference's own inspector says "not yet implemented" there
## and so does this.
##
## ## Nothing draws this window; here is what it derives from instead
##
## Stated because it was got wrong here once (see `setup()`'s corrected
## nesting comment). `design/dcc-environment-2026-08-31/README.md`, verbatim:
## the two DCC files specify the shell frame and its information architecture
## and *"do not specify the dedicated windows: Faction roster, Place editor,
## City viewer, Data manager, Vault, Travel library, Asset library."* So every
## choice below is derived from the canvases' own vocabulary, and named:
##
## - **`§`-headed sections** — `DccTheme.header()`'s `§ TITLE`, the L3 sigil the
##   DCC Environment markup uses for its own panels (`§ TOOLS`, `§ LAYERS`,
##   `§ RESULTS`, `§ BRUSH · GLOBAL`). "Not built" keeps its wording because it
##   is the shell's own settled title for that block — **ten** other
##   `DccWidgets.section(…, "Not built")` call sites across four workspace files
##   (cartography 3, civilization 5, infrastructure 1, world 1; counted at HEAD,
##   not in a working tree other lanes are editing) — and matching them beats
##   matching a canvas that draws no such block at all.
## - **The phone header's title/subtitle voice** — `06-phone.md` §6.6's
##   `_moreTitle()` table: an upper-cased title beside a lower-case
##   `·`-separated list of what the screen holds (`files · autosave · storage`;
##   `import · export · sources · conversion · validation`).
## - **One scroller per phone screen** — §5.5 (`flex:1; overflow-y:auto;
##   overscroll-behavior:contain`).
##
## **Open, and deliberately not resolved here.** On a phone, is this a window
## or a `moreStack` sub-screen? §6.6's `_moreTitle()` answers that for **three**
## of those seven windows and no more: the Data manager, the Travel library and
## the Asset library each get a sub-screen (`data`, `travel`, `assets`, plus
## their own leaves), reached by a `nav` row on the `root` screen. There is no
## row for the Vault, and none for the faction roster, the place editor or the
## city viewer — and §6.6's own `civ` screen, which is where a CIVIL surface
## would sit, lists the three place tools, Landmark generation and the Journey
## Planner and nothing else. The stack exists and these three are not in it.
## That silence is not a licence to rebuild them as sheets, so the window shape
## is unchanged and the question is recorded instead of guessed.

## The one reason every claim-derived figure here can be missing once a world
## exists: a project opened from an archive with no `rasters/territory.i32`
## (`project_bridge.rs::civ_from_project`, which also warns on open). The
## engine omits those figures rather than report the zero an empty claim grid
## sums to; this is what the dash beside each one says.
const NO_CLAIM_GRID := "Not known: this project was opened without its territory map. Paint territory, or run Recompute civilisation, to rebuild it."

## The hub's tabs, in strip order (`FACTION_HUB_DESIGN.md` FH-0). These ids are
## the public vocabulary: `open(select_faction, tab)` and
## `DccApp.open_faction_roster(faction, tab)` take one, and an id not in this
## list is treated as "no tab asked for", never as an error and never as a
## blank pane.
const TAB_IDS: Array[String] = ["identity", "territory", "settlements", "economy",
		"military", "relations", "history"]

## The strip's captions. Mixed case, because a segmented cell at `FS_MICRO` is
## narrow and capitals cost a quarter more width; the phone chooser uses the
## same strings.
const TAB_LABELS := {
	"identity": "Identity", "territory": "Territory", "settlements": "Settlements",
	"economy": "Economy", "military": "Military", "relations": "Relations",
	"history": "History",
}

## Smallest strip cell height off a phone. 28 is a labelled judgement: one
## `FS_SMALL` line plus the button's own padding, level with the roster list's
## 30 px rows. A tablet takes `DccTheme.role_px("btn_min_h")` instead (44, its
## finger target) and a phone `DccTheme.PHONE_TAP_MIN` -- see
## `_build_tab_strip()`.
const TAB_MIN_H_POINTER := 28

var app                       ## `DccApp`
var bridge: EngineBridge

var _selected := 1
var _list_body: VBoxContainer
var _inspector_body: VBoxContainer
var _overview: Label
## `civ_faction_terrain_fits()`. Fetched by `_ensure_fits()` the first time the
## Territory tab is built after an `open()`, a new world or an edit that moves
## it -- **not** in `open()`, because the pass is O(cells).
var _fits: Array = []
var _fits_ready := false
## `civ_military_summary()`, fetched by `_ensure_military()` on the Military
## tab's first show for the same reason `_fits` is lazy: one call carries
## `power.military`, the fortification counts and the whole of
## `cartalith_civ::manpower`'s answer for every faction at once, and it is O(cells).
var _military: Dictionary = {}
var _military_ready := false
## `civ_territory_influence()` for EVERY faction, fetched by the Territory tab's
## "Analyse influence" button and never otherwise (FH-4). One Dijkstra per
## capital over the whole map, so it is behind a click exactly as
## `Territories > Borders & influence` is. Cached across selection changes
## (the answer is whole-world; only the filter by faction differs) and dropped
## by `_mark_data_stale()`. `_influence_ready` is the "was it run" flag, because
## `{}` is also a real answer (a loaded save, or a world with no capital) and
## must read as "ran, and the engine had nothing", not as "not run".
var _influence: Dictionary = {}
var _influence_ready := false
## The container the influence readout is (re)filled into, freed with its pane.
var _influence_body: VBoxContainer = null

## The tab on screen. Deliberately a member that **survives `open()`**: an
## unknown or empty `tab` argument means "the last tab used this session", so
## reopening the hub after looking at Economy returns to Economy. The first
## ever open is Identity.
var _tab := "identity"
## tab id -> its built pane (a `VBoxContainer` child of `_inspector_body`).
## Holds only the tabs built since the last invalidation; the visible one is
## always present after `_show_active_tab()`.
var _tab_panes: Dictionary = {}
## tab id -> its strip/chooser `Button`, named `Tab_<id>` for the probes.
var _tab_buttons: Dictionary = {}

## Phone (§13). The window's own treatment is
## `DccWidgets.phone_window()`'s; what is specific to *this* window is that
## master-detail does not survive the width. The list pane is 250 px and the
## inspector wants the rest, which at a 393 dp reference leaves the inspector
## 140 -- narrower than a single one of its own vocabulary pickers. So phone
## runs the classic master-*then*-detail: the list is a screen until a faction
## is picked, after which it folds into a 52 dp bar carrying that faction's
## banner, its name and its place in the roster, and the bar is what reopens it.
var _phone := false
var _phone_list_pane: Control
var _phone_list_bar: PanelContainer
var _phone_bar_name: Label
var _phone_bar_sub: Label
var _phone_bar_banner: Control
## The inspector head's own banner, held so the colour picker can repaint it
## live without rebuilding the inspector under the open picker (CV-21).
var _head_banner: FactionBanner

## Emitted whenever the roster changes in a way that moves map data (a
## removed faction reverts settlements and territory to Unclaimed).
signal roster_changed

## Emitted after a tariff edit (Ruling AE). Deliberately its own signal, not
## folded into `roster_changed`: `civilization_workspace.gd::_on_roster_changed`
## documents that signal as "a rate changes no flow" and only re-labels the
## held trade match -- true for currency and colour, false for a tariff, which
## changes real matched volume. This one re-runs the match when one is held.
signal tariff_changed


func setup(a, b: EngineBridge) -> void:
	app = a
	bridge = b
	title = "Factions"
	size = Vector2i(880, 620)
	min_size = Vector2i(620, 420)
	## No `max_size` (2026-09-24, the vault window's `3736fe7` fix repeated). The
	## cap treated `wrap_controls` growing the window to its content, which
	## `phone_window()` below turns off at the cause; all the cap still did was
	## stop a user from making the window bigger.
	_phone = DccWidgets.phone_window(self, a)

	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 6)
	## Phone: **one** scrolling column, not two nested scrolling panes inside a
	## third expanding one. Desktop's shape -- a `SIZE_EXPAND_FILL` split holding
	## two `SIZE_EXPAND_FILL` `ScrollContainer`s -- does not survive an
	## `AcceptDialog` on a phone: the dialog laid this column out at 377 x 2 619
	## inside a 393 x 852 window, so the panes had nothing to scroll and their
	## lower two thirds were simply off the screen. The place editor, whose body
	## is a single scroll with no nesting, comes out at 377 x 797 in the same
	## window from the same helper -- which is what identified the nesting as the
	## cause rather than the window size. Phone follows that shape.
	##
	## It is also what the phone canvas settles for every screen it *does* draw:
	## `06-phone.md` §5.5 gives a sheet exactly one scroller, `flex:1;
	## overflow-y:auto; overscroll-behavior:contain`, and §6.0's screen graph
	## has no screen with two. Nested scroll regions on a touch screen also make
	## every drag ambiguous, which is the reason behind the rule.
	##
	## **Corrected 2026-09-05.** This clause used to read "the design canvas's
	## own roster artboard is one column from the overview to the settlement
	## list". There is no roster artboard. `design/` was grepped whole: the only
	## hits for "roster" are menu rows naming `#civOpenFactionsBtn` in
	## `Cartalith Menu Structure v2/v3.dc.html` and `cartalith-menu-structure.md`
	## -- a menu entry, not a drawing -- and
	## `design/dcc-environment-2026-08-31/README.md` says the opposite in as
	## many words: *"They do not specify the dedicated windows: Faction roster,
	## Place editor, City viewer, Data manager, Vault, Travel library, Asset
	## library."* The conclusion held; the authority cited for it did not exist.
	if _phone:
		var root := ScrollContainer.new()
		root.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		root.size_flags_vertical = Control.SIZE_EXPAND_FILL
		add_child(root)
		outer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		root.add_child(outer)
	else:
		add_child(outer)

	_overview = DccTheme.label("", "text_dim", DccTheme.FS_SMALL)
	_overview.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	outer.add_child(_overview)
	outer.add_child(DccTheme.rule())

	if _phone:
		_phone_list_bar = _build_phone_list_bar()
		outer.add_child(_phone_list_bar)

	## Side by side on a pointer; stacked on a phone, where only one of the two
	## panes is ever visible at a time.
	var split: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	split.add_theme_constant_override("separation", 10)
	if not _phone:
		split.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(split)

	var left := VBoxContainer.new()
	if not _phone:
		left.custom_minimum_size.x = 250
	left.add_theme_constant_override("separation", 4)
	split.add_child(left)
	_phone_list_pane = left
	var list_host: Control = left
	if not _phone:
		var list_scroll := ScrollContainer.new()
		list_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		list_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
		left.add_child(list_scroll)
		list_host = list_scroll
	_list_body = VBoxContainer.new()
	_list_body.add_theme_constant_override("separation", 2)
	_list_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	list_host.add_child(_list_body)

	var roster_row := HBoxContainer.new()
	roster_row.add_theme_constant_override("separation", 6)
	DccWidgets.action(roster_row, "+ Add faction", _add_faction)
	DccWidgets.action(roster_row, "− Remove last", _confirm_remove)
	## Stays under the list it acts on, on both form factors. Pinning it to the
	## window foot on the phone was tried first and is what found the layout
	## trap underneath: an `AcceptDialog` sizes its content child from a resize
	## notification, so anything laid out *after* a `SIZE_EXPAND_FILL` pane can
	## be pushed past the bottom of a window that was resized while hidden --
	## measured at 2 611 px of content in an 852 px window. Below the list is
	## also where these two belong: they change the roster, and the roster is
	## what the list is.
	left.add_child(roster_row)
	if not _phone:
		split.add_child(DccTheme.rule(true))

	## The detail column: the tab strip over the inspector. On a pointer it is a
	## plain `VBoxContainer` holding the strip and the one scroller (still the
	## only `SIZE_EXPAND_FILL` scrolling pane on that side of the split, so the
	## `AcceptDialog` sizing trap above is not re-opened). On a phone the strip
	## is simply the next row of the one root scroller.
	var inspector_host: Control = split
	if not _phone:
		var right := VBoxContainer.new()
		right.add_theme_constant_override("separation", 6)
		right.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		right.size_flags_vertical = Control.SIZE_EXPAND_FILL
		split.add_child(right)
		right.add_child(_build_tab_strip())
		var right_scroll := ScrollContainer.new()
		right_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		right_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		right_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
		right.add_child(right_scroll)
		inspector_host = right_scroll
	else:
		split.add_child(_build_tab_strip())
	_inspector_body = VBoxContainer.new()
	_inspector_body.add_theme_constant_override("separation", 4)
	_inspector_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	inspector_host.add_child(_inspector_body)

	if _phone:
		## Subtitle in §6.6 `_moreTitle()`'s voice — lower case, `·`-separated,
		## naming what the screen holds rather than describing it. It read
		## "world politics", which is a topic; the canvas's own subtitles are
		## contents lists (`files · autosave · storage`).
		DccWidgets.phone_head(outer, "Factions", "roster · identity · territory · economy · military")
		_set_phone_list_open(true)

	## `GUI_GAP_REGISTER.md` **RF-03**. §23 asked "what re-runs this, and on
	## which signal?" of every panel built at launch; this window is built on
	## `open()` instead, which is correct only if nothing can change while it is
	## up. A world can. Left open across a generate the roster went on showing
	## the previous world's factions -- measured Aurelia:27 / Veldmark:49 /
	## Mirelle:57 against a live engine reading Aurelia:57 / Veldmark:27 /
	## Mirelle:7 -- and every editable control in it writes by faction **id**,
	## which the new world reuses. So a rename or a culture change committed
	## from that stale pane lands on a different faction than the one on screen:
	## FR-02's data-corruption mode with a generate as the trigger instead of a
	## click.
	bridge.generation_finished.connect(func(ok: bool): if ok and visible: _on_world_changed())
	bridge.world_loaded.connect(func(): if visible: _on_world_changed())


## Both O(cells) caches are per-world, so both are marked stale here and
## re-taken lazily by the tab that reads them -- the rebuild below builds the
## **active tab only**, so a world change while on Identity runs neither pass,
## and every other tab is invalidated and rebuilt on its next show rather than
## showing the previous world's numbers.
## `_selected` is reset because it is an id into a roster the new world has
## replaced -- the same reason `civilization_workspace._on_world_changed()`
## resets its own `_selected_index`.
##
## Nothing commits on the way out: `_rebuild()` goes through `_clear()`, whose
## `_rebuilding` guard is exactly what stops a dying focused field writing its
## text into the world that just replaced the one it was typed for (FR-02).
func _on_world_changed() -> void:
	_mark_data_stale()
	_selected = 1
	_rebuild()


# -- Phone: the folded master pane -------------------------------------------

## The bar the list folds into: the selected faction's own banner, its name,
## its position in the roster, and a chevron. It *is* a list row -- the one row
## still worth showing once a choice has been made -- so it is built as one,
## against `06-phone.md` §6.6's `nav` row, which is the only list row the phone
## canvas specifies:
##
## | canvas (`nav`) | here |
## |---|---|
## | `min-height:52px` | `custom_minimum_size.y = 52` |
## | `gap:12px` | separation 12 |
## | glyph, `width:18px` | the faction banner, 22 px (it carries the identity colour, which a glyph cannot) |
## | label `12.5px`, `var(--ink)` | `text_bright`, `FS_SMALL` |
## | sub `9.5px mono`, `var(--dim)`, ellipsised | `_phone_bar_sub` |
## | chevron `›`, `var(--faint)` | `DccIcons.SYMBOLS["expand"]` is `›`, `text_faint` |
##
## Two deviations, both deliberate. The canvas's `padding:0 12px` sits *inside*
## §5.5's scroller, which already pays `padding:2px 14px`; this bar has no
## scroller around it, so its own 14 is that outer inset and not a second one.
## And the canvas's `nav` row ends in a badge slot, which has nothing to carry
## here -- an empty badge is a column of air, so there is none.
##
## **The sub line is new, 2026-09-05, and the comment above is why.** It has
## claimed "its position in the roster" since the bar shipped, and the bar drew
## a banner, a name and a chevron. The canvas settles what a `nav` sub looks
## like, so the fix is to draw the thing the comment promised rather than to
## delete the promise.
func _build_phone_list_bar() -> PanelContainer:
	var bar := PanelContainer.new()
	bar.add_theme_stylebox_override("panel", DccTheme.panel("raised", {"bottom": 1}))
	bar.custom_minimum_size.y = 52
	bar.mouse_filter = Control.MOUSE_FILTER_STOP
	bar.gui_input.connect(func(ev: InputEvent):
		var tapped: bool = (ev is InputEventMouseButton and (ev as InputEventMouseButton).pressed) \
			or (ev is InputEventScreenTouch and (ev as InputEventScreenTouch).pressed)
		if tapped:
			_set_phone_list_open(true))

	var m := MarginContainer.new()
	m.add_theme_constant_override("margin_left", 14)
	m.add_theme_constant_override("margin_right", 14)
	m.mouse_filter = Control.MOUSE_FILTER_IGNORE
	bar.add_child(m)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 12)
	row.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	m.add_child(row)

	_phone_bar_banner = FactionBanner.new()
	_phone_bar_banner.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(_phone_bar_banner)

	## Label over sub, which is the `nav` row's own stack. The column takes the
	## expand so the chevron stays pinned right.
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 1)
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	col.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(col)
	_phone_bar_name = DccTheme.mono_label("", "text_bright", DccTheme.FS_SMALL, 0)
	_phone_bar_name.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_phone_bar_name.clip_text = true
	_phone_bar_name.mouse_filter = Control.MOUSE_FILTER_IGNORE
	col.add_child(_phone_bar_name)
	## `9.5px mono, var(--dim), ellipsised` — the canvas's own sub, at the
	## nearest whole pixel this theme carries. `OVERRUN_TRIM_ELLIPSIS` is set
	## here for the "ellipsised", not left to `DccShell.phone_fit`, which sets
	## the same pair on every expanding non-wrapping `Label` it walks: the
	## property belongs with the row it was derived for rather than arriving as
	## a side effect of a walk.
	##
	## `clip_text` is *not* set here and is expected to be true at runtime,
	## because that walk sets it — measured, `_entwinphone_probe.gd` FR6 reads
	## `clip=true` on a bar this file never clipped. That is safe in a
	## `VBoxContainer`: `clip_text` does collapse a `Label`'s minimum width to 1
	## (MISTAKES), which strands a label beside an expanding *horizontal*
	## sibling, and this one stacks under the name rather than beside it, so the
	## column hands it the full width regardless.
	_phone_bar_sub = DccTheme.mono_label("", "text_dim", DccTheme.FS_MICRO, 0)
	_phone_bar_sub.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_phone_bar_sub.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	_phone_bar_sub.mouse_filter = Control.MOUSE_FILTER_IGNORE
	col.add_child(_phone_bar_sub)

	## `text_faint` (`--faint`), the colour §6.6 gives a `nav` chevron. It was
	## `text_ghost`, which is `--dis` -- this shell's disabled ink, on a bar
	## whose entire purpose is that it is tappable.
	var chev := DccTheme.mono_label(DccIcons.SYMBOLS["expand"], "text_faint", DccTheme.FS_SMALL, 0)
	chev.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(chev)
	return bar

## Exactly one of the two panes is on screen. The bar exists only while the
## list is folded, so it is never a second copy of a row already visible.
func _set_phone_list_open(open: bool) -> void:
	if not _phone:
		return
	_phone_list_pane.visible = open
	_phone_list_bar.visible = not open
	if not open:
		var d := _faction(_selected)
		_phone_bar_name.text = String(d.get("name", "?")) if not d.is_empty() else "—"
		if not d.is_empty():
			(_phone_bar_banner as FactionBanner).configure(_selected, _color_of(d), 22)
		_phone_bar_sub.text = _phone_bar_sub_text(d)
		## Hidden, not blank, when there is nothing to say. A `Label` holding ""
		## still claims a line of the 52 dp row and would push the name off
		## centre for no information.
		_phone_bar_sub.visible = _phone_bar_sub.text != ""


## The `nav` sub for the folded bar: where this faction sits in the roster, and
## how much of the world it holds. `·`-separated in §6.6's own voice
## (`129384 · stages 01–07 · 5 d ago`).
##
## **The position is looked up, not assumed.** `_selected` is a faction *id* and
## the roster is an array; `civ_remove_faction` can leave the two out of step,
## and a bar that printed the id as an ordinal would be wrong exactly when the
## roster has been edited -- which is the only time anyone reads this window.
## When the id is not in the roster there is no position, so the clause is
## omitted rather than defaulted to a plausible `1 of n`.
func _phone_bar_sub_text(d: Dictionary) -> String:
	if d.is_empty():
		return ""
	var roster := bridge.get_factions()
	var parts: Array[String] = []
	for i in roster.size():
		if int((roster[i] as Dictionary).get("id", -1)) == _selected:
			parts.append("%d of %d" % [i + 1, roster.size()])
			break
	var n := int(d.get("settlement_count", 0))
	parts.append("%d settlement%s" % [n, "" if n == 1 else "s"])
	return " · ".join(parts)

## `select_faction`: the faction id to land on, or `-1` (the default) to reopen
## on whichever faction was last selected, exactly as every caller did before
## this parameter existed. It exists for the context card's CIVIL "Open <name>
## in roster" row (`MAP_CONTEXT_SCOPE.md` §4.3, CM-2 follow-ups), which already
## knows which faction controls the right-clicked cell (`sample_cell`'s
## `controlling_faction`). An id not in the roster is **ignored, not clamped**:
## selecting a neighbour would show the wrong faction as if it were the one asked
## for, so the window just opens where it last was. On the phone a landed-on
## faction opens its inspector directly -- the pick IS the navigation there, the
## same rule the list rows' own press follows -- rather than the master list.
##
## `tab`: one of `TAB_IDS`, or `""` (the default) for the tab last used this
## session -- Identity on the very first open. An id that is not a tab is
## treated exactly like `""`: it falls back rather than leaving the hub on a
## tab that does not exist. A faction id and a tab id are independent, so
## either may be given alone.
##
## **Nothing O(cells) runs here any more.** Both engine passes used to be
## fetched on every open, in front of a window that might then be shown on
## Identity and never read either; they are now marked stale
## (`_mark_data_stale()`, so a roster that changed while the hub was hidden is
## re-read) and fetched by the tab that needs them.
func open(select_faction: int = -1, tab: String = "") -> void:
	_mark_data_stale()
	var landed := select_faction > 0 and not _faction(select_faction).is_empty()
	if landed:
		## FR-02: flush a half-typed field against the faction it was typed for
		## before `_selected` moves, in case the hub was already up with a field
		## focused (the context card can re-open it onto another faction).
		_commit_focused_field()
		## Before `_rebuild()`, so the list highlight and the inspector are
		## built for the faction asked for, not rebuilt after.
		_selected = select_faction
	if TAB_IDS.has(tab):
		_tab = tab
	_rebuild()
	## Reopens on the master, the way a phone list screen does -- picking up
	## mid-inspector on a faction chosen in a previous session would hide the
	## only control that says which faction this is. A faction asked for BY ID
	## (`landed`) is not a previous session's choice: the caller named it, so
	## the phone shows its inspector.
	_set_phone_list_open(not landed)
	if not DccWidgets.phone_present(self, app):
		popup_centered()
		## `AcceptDialog` sizes its content child once, at popup, from the
		## child's minimum -- and at that moment `_overview` (autowrap) has no
		## width yet, so it reports one glyph per line and the column came out
		## 2 602 px tall in a 620 px window: "+ Add faction" / "− Remove last"
		## sat 2 000 px below the frame and OK floated over the inspector.
		## Nothing re-runs that sizing when the label shrinks, so ask once
		## more after the first layout pass (measured: 2 602 -> 581).
		child_controls_changed.call_deferred()


## Set for the duration of a pane teardown. The inspector's name field commits
## on `focus_exited`, and removing a focused `Control` from the tree fires that
## signal **synchronously** -- so a rebuild was itself an "edit"
## (`GUI_GAP_REGISTER.md` FR-02). Measured before it was believed: with
## Aurelia's name field focused, clicking Veldmark in the list left the roster
## reading `1:Aurelia, 2:Aurelia` -- `_selected` is reassigned before
## `_rebuild_inspector()`, so the dying field's text was written to the faction
## the user had just switched TO.
var _rebuilding := false

func _clear(node: Control) -> void:
	var was := _rebuilding
	_rebuilding = true
	for c in node.get_children():
		node.remove_child(c)
		c.queue_free()
	_rebuilding = was

## Commit whatever the inspector's focused field holds **before** the caller
## changes `_selected`, so a half-typed rename lands on the faction it was
## typed for rather than being dropped by the guard above. Returns having
## released focus, which is what makes the `focus_exited` commit fire here,
## against the right id, instead of during the teardown.
func _commit_focused_field() -> void:
	var vp := _inspector_body.get_viewport()
	if vp == null:
		return
	var fo := vp.gui_get_focus_owner()
	if fo != null and _inspector_body.is_ancestor_of(fo):
		fo.release_focus()


func _rebuild() -> void:
	_rebuild_overview()
	_rebuild_list()
	_rebuild_inspector()

## FH-5: an edit made OUTSIDE this window moved what it shows -- the Place
## editor's Polity picker repaints claims and moves a settlement between
## factions (`civilization_workspace.gd::_on_polity_moved`). Unlike
## `_on_world_changed` it keeps `_selected` and the active tab: the roster is
## the same roster, only its numbers moved. The O(cells) caches are marked
## stale so the Territory tab re-takes them; any focused field is committed
## against the faction it was typed for first (FR-02), and `_rebuild()`'s
## `_clear()` teardown runs under the `_rebuilding` guard as every rebuild does.
func refresh_after_edit() -> void:
	_commit_focused_field()
	_mark_data_stale()
	_rebuild()


# -- World overview (`_civRenderFactionsWorldOverview`) ----------------------

func _rebuild_overview() -> void:
	var factions := bridge.get_factions()
	if factions.is_empty():
		_overview.text = "Generate a world to see a faction summary here."
		return
	var total_pop := 0
	var total_settle := 0
	var total_cells := 0
	## `claimed_cells` is omitted, not `0`, while the claim grid is unknown
	## (`get_factions()`' own doc) -- so the total is a dash, not a sum of the
	## rows that happen to carry it.
	var cells_known := true
	var top_name := ""
	var top_pop := -1
	for f in factions:
		var d: Dictionary = f
		var pop := int(d.get("population", 0))
		total_pop += pop
		total_settle += int(d.get("settlement_count", 0))
		if d.has("claimed_cells"):
			total_cells += int(d["claimed_cells"])
		else:
			cells_known = false
		if pop > top_pop:
			top_pop = pop
			top_name = String(d.get("name", "?"))
	var agr := bridge.civ_agrarian_regional_total()
	var land_line := ""
	if not agr.is_empty():
		land_line = "  ·  Land sustains ≈ %s across %s km²" % [
			_thousands(int(agr.get("sustains", 0))), _thousands(int(agr.get("land_km2", 0)))]
	var cells_text := ("%d claimed cells" % total_cells) if cells_known else "claimed cells —"
	_overview.text = "%d factions  ·  %s total settled population  ·  %d settlements  ·  %s%s\nLargest by population: %s (%s)" % [
		factions.size(), _thousands(total_pop), total_settle, cells_text, land_line,
		top_name, _thousands(top_pop)]
	if not cells_known:
		_overview.text += "\n" + NO_CLAIM_GRID


# -- Faction list (`_civRenderFactionList`) ---------------------------------

func _rebuild_list() -> void:
	_clear(_list_body)
	var factions := bridge.get_factions()
	if factions.is_empty():
		DccWidgets.note(_list_body, "No world generated -- %s to begin."
			% DccShell.new_world_route())
		return
	for f in factions:
		var d: Dictionary = f
		var fid := int(d.get("id", 1))
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)
		## The banner is a `Control` with its own `_draw()`, so it goes
		## beside the button rather than into `Button.icon` (which wants a
		## `Texture2D` and would need a `SubViewport` round trip to get one).
		var banner := FactionBanner.new()
		banner.configure(fid, _color_of(d), 22)
		row.add_child(banner)
		var b := Button.new()
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.custom_minimum_size.y = 30
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.text = "%s — %d settlements, %s pop" % [
			String(d.get("name", "?")), int(d.get("settlement_count", 0)),
			_thousands(int(d.get("population", 0)))]
		if fid == _selected:
			## `flat` has to come off for the same reason `layers_popover.gd`'s own
			## active row does: a flat `Button` skips `normal`/`hover`/`pressed`
			## entirely, so this override -- `DccTheme.flat(c("sunken"))`, alone on
			## "normal" -- had never drawn. This window has no canvas of its own
			## (`design/dcc-environment-2026-08-31/README.md` names it undrawn), so
			## the derived treatment is `DccTheme.outline()`'s own documented
			## "selected row" shape -- accent-outlined, `accent_wash` fill -- the
			## same one `browse_dialog.gd`'s `_paint_row()` already uses for its
			## selected folder row, rather than the layers popover's solid-accent
			## slab, which `GUI_GAP_REGISTER.md` §48 (DS-02) found to be the
			## canvas's *one* deliberate full-fill surface, not a general pattern.
			b.flat = false
			var slab := DccTheme.outline("accent", "accent_wash")
			for sb_name in ["normal", "hover", "pressed"]:
				b.add_theme_stylebox_override(sb_name, slab)
			for color_key in ["font_color", "font_hover_color", "font_pressed_color"]:
				b.add_theme_color_override(color_key, DccTheme.c("text_bright"))
		b.pressed.connect(func():
			## FR-02: flush the inspector's pending edit against the faction it
			## was typed for, before `_selected` moves. These list rows are
			## `FOCUS_NONE`, so without this the name field keeps focus right up
			## until `_rebuild_inspector()` frees it -- under the new id.
			_commit_focused_field()
			_selected = fid
			_rebuild_list()
			_rebuild_inspector()
			## Phone: the pick IS the navigation. Desktop leaves both panes up.
			_set_phone_list_open(false))
		row.add_child(b)
		_list_body.add_child(row)
	## `_set_field("name")` rebuilds this list on its own, so it needs its own
	## fit rather than relying on the inspector's.
	if _phone:
		app.phone_fit(_list_body, 1.0)


# -- Inspector (`_civPopulateFactionEditor`) --------------------------------

## Rebuilds the **visible tab** for the current faction and invalidates every
## other tab. This is the whole teardown, so it runs under `_clear()`'s
## `_rebuilding` guard (FR-02) and nothing built here is reachable from a
## previous faction's pane. It is also what a probe or a caller means by "redraw
## the inspector"; it never builds a tab that is not on screen.
func _rebuild_inspector() -> void:
	_clear(_inspector_body)
	_tab_panes.clear()
	_show_active_tab()


## Makes `_tab`'s pane the visible one, building it first if it has not been
## built since the last invalidation, and restyles the strip. Hiding a pane is
## safe against FR-02 because `_select_tab()` has already released any focus
## inside it; this function must never be the first to do so.
func _show_active_tab() -> void:
	var d := _faction(_selected)
	if d.is_empty():
		## No world (or an id the roster no longer holds): one note, whatever the
		## tab. Cleared with the guard so a stale pane cannot commit into it.
		_clear(_inspector_body)
		_tab_panes.clear()
		DccWidgets.note(_inspector_body, "Select a faction.")
		_style_tab_buttons()
		return
	for id in _tab_panes:
		(_tab_panes[id] as Control).visible = (id == _tab)
	if not _tab_panes.has(_tab):
		_tab_panes[_tab] = _build_tab_pane(_tab, d)
	_style_tab_buttons()
	## The panes are rebuilt from scratch on a switch, so the touch fit is
	## re-applied over the window each time; idempotent, per `DccShell.phone_fit`.
	if _phone:
		app.phone_fit(self, 1.0)


## Builds one tab's pane and adds it to `_inspector_body`. Every tab is
## guaranteed a non-empty pane: a builder that has nothing to say for this
## faction (the Military block returns nothing for a faction with no row) gets
## a plain note instead of a blank tab, because a blank page reads as a broken
## one. Never builds more than the tab it is asked for.
func _build_tab_pane(id: String, d: Dictionary) -> VBoxContainer:
	var pane := VBoxContainer.new()
	pane.name = "Pane_" + id
	pane.add_theme_constant_override("separation", 4)
	pane.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_inspector_body.add_child(pane)
	match id:
		"identity":
			_build_tab_identity(pane, d)
		"territory":
			_build_tab_territory(pane, d)
		"settlements":
			_build_settlement_sublist(pane)
		"economy":
			_build_currency(pane)
			_build_tariffs(pane)
			_build_gaps(pane)
		"military":
			_build_military_block(pane)
		"relations":
			_build_tab_relations(pane)
		"history":
			_build_tab_history(pane)
	if pane.get_child_count() == 0:
		DccWidgets.note(pane, "Nothing to show for this faction on this tab.")
	return pane


## The Identity tab: the faction's banner and name, then the five identity
## rows, with the **Culture profile card** (FH-2, `_build_culture_card`) hung
## directly under the Culture picker, and the **Default settlement type** picker
## (FH-3, `_default_type_choice`) as the last row. The first five moved verbatim
## out of the old single-scroll inspector; the head's name field still commits on
## `focus_exited` unless `_rebuilding` (FR-02).
func _build_tab_identity(pane: VBoxContainer, d: Dictionary) -> void:
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 8)
	_head_banner = FactionBanner.new()
	_head_banner.configure(_selected, _color_of(d), 48)
	head.add_child(_head_banner)
	var name_edit := LineEdit.new()
	name_edit.text = String(d.get("name", ""))
	name_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_edit.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	name_edit.text_submitted.connect(func(t: String): _set_field("name", t))
	## Guarded against its own teardown -- see `_rebuilding` (FR-02).
	name_edit.focus_exited.connect(func():
		if _rebuilding:
			return
		_set_field("name", name_edit.text))
	head.add_child(name_edit)
	pane.add_child(head)

	var sec := DccWidgets.section(pane, "Identity")
	_colour_row(sec, d)
	_vocab_choice(sec, "Government", bridge.civ_government_vocabulary(),
		String(d.get("government", "monarchy")), "government",
		"Live since 2026-08-25, and this is its first consumer in either codebase — the reference's own comment says no simulation reads it there. It sets how much of the surplus this faction's state can actually capture, which drives the standing army and half of the mobilization reach (see the Military tab).")
	_culture_choice(sec, String(d.get("culture", "common")))
	_build_culture_card(sec, String(d.get("culture", "common")))
	_vocab_choice(sec, "Religion", bridge.civ_religion_vocabulary(),
		String(d.get("religion", "none")), "religion")
	_ag_tech_choice(sec, String(d.get("ag_tech", "traditionalAgrarian")))
	_default_type_choice(sec)


## The Relations tab -- a placeholder, on purpose. The only relations model is
## the derived per-pair table under Civilization ▸ Relationships; nothing about
## a faction's own relations is stored, so there is nothing to edit here. It
## says what exists and what does not, in words. **It must stay control-free
## and treaty-free until a model backs it** (`FACTION_HUB_DESIGN.md`): a row
## for a treaty that cannot be signed is a lie the shell would be telling.
func _build_tab_relations(pane: VBoxContainer) -> void:
	var sec := DccWidgets.section(pane, "Relations")
	DccWidgets.note(sec,
		"Not built in this window. Relations between factions are under Civilization ▸ "
		+ "Relationships: a derived value per faction pair, recomputed rather than stored. "
		+ "What is still absent anywhere is anything that acts -- treaties, vassalage, war "
		+ "declarations, change over time.")


## The History tab -- a placeholder, on purpose, for the same reason as
## `_build_tab_relations()`: the engine keeps no per-faction history, so this
## says so instead of drawing an empty table. Must stay control-free.
func _build_tab_history(pane: VBoxContainer) -> void:
	var sec := DccWidgets.section(pane, "History")
	DccWidgets.note(sec,
		"Not built. The engine keeps no per-faction history -- no founding, wars, rulers or "
		+ "collapses are recorded against a faction -- so nothing is drawn here rather than "
		+ "an empty log. The world-level timeline is under Civilization ▸ Timeline.")


# -- Tabs: strip, chooser, switching -----------------------------------------

## The strip (pointer and tablet) or the segmented chooser (phone): one
## `Button` per `TAB_IDS`, in the segmented shape the retired Culture profiles window's
## phone switcher already uses -- an accent-wash fill on the active cell, a
## `FS_MICRO` mono caption, no focus ring (`FOCUS_NONE`, so a tab press never
## steals the focus FR-02 reasons about).
##
## **Phone:** a 4 + 3 `GridContainer` of cells each at least
## `DccTheme.PHONE_TAP_MIN` (44) tall. Not a `TabContainer`, and not a
## horizontally scrolling strip: at 393 dp seven captions do not fit one row,
## and the hub must stay a single vertical scroller. **Elsewhere:** one
## `HBoxContainer` row whose buttons clip their caption rather than demand its
## width -- a button's minimum width is its whole label, and seven of them
## would otherwise push the `AcceptDialog` wider than its window (the
## disabled-axis trap `DccWidgets.action` documents). A tablet's cell height is
## `DccTheme.role_px("btn_min_h")`, not a literal.
func _build_tab_strip() -> Control:
	var wrap := PanelContainer.new()
	wrap.name = "TabStrip"
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"bottom": 1}))
	var row: Container
	if _phone:
		var grid := GridContainer.new()
		grid.columns = 4
		grid.add_theme_constant_override("h_separation", 0)
		grid.add_theme_constant_override("v_separation", 0)
		row = grid
	else:
		var hb := HBoxContainer.new()
		hb.add_theme_constant_override("separation", 0)
		row = hb
	row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	DccWidgets.pad(wrap, 4, 2, 4, 2).add_child(row)
	var cell_h: int = DccTheme.PHONE_TAP_MIN if _phone \
		else maxi(DccTheme.role_px("btn_min_h"), TAB_MIN_H_POINTER)
	for id in TAB_IDS:
		var key: String = id
		var b := Button.new()
		b.name = "Tab_" + key
		b.text = String(TAB_LABELS[key])
		b.tooltip_text = String(TAB_LABELS[key])
		b.focus_mode = Control.FOCUS_NONE
		b.clip_text = true
		b.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		b.custom_minimum_size = Vector2(0, cell_h)
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.add_theme_font_override("font", DccTheme.mono(0))
		b.add_theme_font_size_override("font_size", DccTheme.FS_MICRO)
		b.pressed.connect(func(): _select_tab(key))
		row.add_child(b)
		_tab_buttons[key] = b
	return wrap


## Restyles every cell: accent-wash fill and accent ink on the active one,
## nothing and `text_dim` on the rest. Idempotent; called from
## `_show_active_tab()` so the strip can never disagree with the pane.
func _style_tab_buttons() -> void:
	for id in _tab_buttons:
		var b: Button = _tab_buttons[id]
		var on: bool = id == _tab
		var fill: StyleBox = DccTheme.flat(DccTheme.c("accent_wash")) if on else DccTheme.empty()
		for sb_name in ["normal", "hover", "pressed"]:
			b.add_theme_stylebox_override(sb_name, fill)
		b.add_theme_stylebox_override("focus", DccTheme.empty())
		b.add_theme_color_override("font_color",
			DccTheme.c("accent") if on else DccTheme.c("text_dim"))
		b.add_theme_color_override("font_hover_color",
			DccTheme.c("accent") if on else DccTheme.c("text"))
		b.add_theme_color_override("font_pressed_color", DccTheme.c("accent"))


## Switch to `id`. **Order matters (FR-02):** the focused field is committed
## first, against the faction and tab it was typed for; only then does `_tab`
## change and the old pane get hidden. An unknown id is ignored. Never frees a
## pane -- panes are only freed by `_drop_panes()` / `_clear()`, which carry the
## `_rebuilding` guard.
func _select_tab(id: String) -> void:
	if not TAB_IDS.has(id):
		return
	_commit_focused_field()
	if id == _tab and _tab_panes.has(id):
		return
	_tab = id
	_show_active_tab()


## Frees every built tab pane except `keep` (or all of them when `keep` is
## empty). The invalidation half of the lazy build: used when something the
## hidden panes show may have changed. Freed under `_rebuilding` so a pane's
## dying field cannot commit (FR-02); the kept pane is never touched, so an
## edit in progress in it survives.
func _drop_panes(keep: String = "") -> void:
	var was := _rebuilding
	_rebuilding = true
	for id in _tab_panes.keys():
		if id == keep:
			continue
		var pane: Control = _tab_panes[id]
		if is_instance_valid(pane):
			_inspector_body.remove_child(pane)
			pane.queue_free()
		_tab_panes.erase(id)
	_rebuilding = was


## Marks both O(cells) caches stale so the next tab that reads one re-fetches.
## Cheap by construction -- it fetches nothing.
func _mark_data_stale() -> void:
	_fits_ready = false
	_military_ready = false
	_influence_ready = false
	_influence = {}


## Fetches `civ_faction_terrain_fits()` once per staleness. O(cells): it
## rebuilds a biome raster and an ocean-distance field (see the Rust doc
## comment on the call), which is why only the Territory tab asks.
func _ensure_fits() -> void:
	if _fits_ready:
		return
	_fits = bridge.civ_faction_terrain_fits()
	_fits_ready = true


## Fetches `civ_military_summary()` once per staleness. O(cells): one call
## rebuilds the biome/lithology/resource passes `civ_faction_aggregates` needs,
## and every faction's row comes out of that one answer. Only the Military tab
## asks.
func _ensure_military() -> void:
	if _military_ready:
		return
	_military = bridge.civ_military_summary()
	_military_ready = true


## The faction's identity colour — `GUI_GAP_REGISTER.md` **CV-21**, and v3's
## "CIVIL owns the colour, CARTO owns the paint" split at the CIVIL end.
##
## Registered as unbacked during the v3 pass, on the reading that
## "`FactionRoster` stores no colour field". It stored one already; what it
## had no way to do was let anyone *set* it, and nothing read it — the
## renderers went to `FACTION_RGB` by index. `civ_set_faction_color` writes
## the override and the three surfaces that draw a faction (territory wash,
## Political-control field, this banner) all read `CivData::faction_rgb`.
##
## `color_changed` rather than `popup_closed`: the map updates while the
## wheel is dragged, which is the whole point of a colour picker over a map.
func _colour_row(parent: Control, d: Dictionary) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = 24
	row.tooltip_text = "The colour this faction is drawn in: its territory wash on the map, the Political control analysis field, and its banner here. Unset, it takes the palette's own colour for this index."
	var l := DccTheme.mono_label("Colour", "text_dim", DccTheme.FS_SMALL, 0)
	l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
	l.clip_text = true
	row.add_child(l)

	var picker := ColorPickerButton.new()
	picker.custom_minimum_size = Vector2(64, 20)
	picker.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	picker.color = _color_of(d)
	picker.edit_alpha = false
	## Live, not on close: `roster_changed` repaints the territory wash, so
	## dragging the wheel drags the map's colour with it.
	picker.color_changed.connect(func(c: Color):
		if _rebuilding:
			return
		if bridge.civ_set_faction_color(_selected, c):
			_repaint_banners(c)
			roster_changed.emit())
	row.add_child(picker)

	var reset := DccWidgets.text_button(row, "Reset", func():
		if bridge.civ_clear_faction_color(_selected):
			roster_changed.emit()
			_rebuild())
	reset.disabled = not bool(d.get("color_custom", false))
	reset.tooltip_text = ("Back to the palette colour for faction %d." % _selected) if not reset.disabled \
		else "Already on the palette colour — nothing to reset."
	parent.add_child(row)


## The two banners on screen for the selected faction (the inspector head and,
## on phone, the bar) repainted in place. Rebuilding the whole inspector on
## every wheel movement would tear the picker down mid-drag.
func _repaint_banners(c: Color) -> void:
	if _head_banner != null and is_instance_valid(_head_banner):
		_head_banner.configure(_selected, c, 48)
	if _phone_bar_banner != null and is_instance_valid(_phone_bar_banner):
		(_phone_bar_banner as FactionBanner).configure(_selected, c, 22)


func _vocab_choice(parent: Control, label_text: String, vocab: Array, current: String,
		key: String, tip: String = "") -> void:
	if vocab.is_empty():
		return
	var keys: Array = []
	var labels: Array = []
	for e in vocab:
		var d: Dictionary = e
		keys.append(String(d.get("key", "")))
		labels.append(String(d.get("label", "?")))
	DccWidgets.choice(parent, label_text, labels, maxi(0, keys.find(current)),
		func(i: int): _set_field(key, keys[i]), tip)


## Cultures come back as bare keys -- the reference's own `CIV_CULTURES`
## carries no display label, so capitalising the key is the honest render,
## not an invented label table.
func _culture_choice(parent: Control, current: String) -> void:
	var keys := bridge.civ_culture_vocabulary()
	if keys.is_empty():
		return
	var labels: Array = []
	for k in keys:
		labels.append(String(k).capitalize())
	DccWidgets.choice(parent, "Culture", labels, maxi(0, Array(keys).find(current)),
		func(i: int): _set_field("culture", String(keys[i])),
		"Naming culture -- the pool _civSettleName draws this faction's settlement names from. Also what the Territory tab's fit verdict judges the land against.")


# -- Culture profile card (FH-2, `FACTION_HUB_DESIGN.md` §3.5 Identity) ------

## Most chips the Name-pool block draws. 8 is the retired Culture profiles
## window's own `mini(8, matches.size())` (last at commit `a61bdd8f`), carried over
## unchanged: a labelled judgement -- enough to read a pool's flavour, few enough
## that the card does not outgrow the Identity rows it sits among on a phone.
const NAME_POOL_MAX := 8

## The card's container, a child of the Identity section directly under the
## Culture picker. Held so a reroll can refill the card in place instead of
## rebuilding the whole inspector (which would tear down a focused field).
## Freed with its pane; `is_instance_valid()` guards every use.
var _culture_card_host: VBoxContainer


## Hangs the Culture profile card under the Culture picker. It folds the retired
## Culture profiles window's detail pane (profile), "Name pool -- real
## settlements" and "Assigned factions" into the Identity tab (owner decision 2,
## 2026-10-05): the faction's CURRENT culture is the one shown, so editing the
## picker above and reading what that culture means are one surface.
##
## Read-only except the reroll chips. **Never writes the culture itself** --
## that stays `_culture_choice()` -> `_set_field("culture")`, whose rebuild also
## refills this card (`_rebuild_inspector()`), so there is one write path.
func _build_culture_card(parent: Control, key: String) -> void:
	_culture_card_host = VBoxContainer.new()
	_culture_card_host.name = "CultureCard"
	_culture_card_host.add_theme_constant_override("separation", 2)
	_culture_card_host.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(_culture_card_host)
	_fill_culture_card(key)


## The `get_cultures()` row whose `key` is `key`, or `{}` -- the seven rows exist
## before any world does (compile-time `CIV_CULTURES`), so `{}` means a key the
## engine does not know or a library older than the binding, never "no world".
func _culture_row(key: String) -> Dictionary:
	for c in bridge.get_cultures():
		if String((c as Dictionary).get("key", "")) == key:
			return c
	return {}


## Real settlements belonging to factions CURRENTLY assigned culture `key`, as
## `{"index": i, "data": settlement}` (`index` is into `bridge.settlements()`,
## which is what `civ_reroll_settlement_name` takes). Ported from the retired
## window's `_settlements_for_culture`; "currently assigned" is the point -- see
## `_fill_culture_card` for why the pool is real settlements.
func _settlements_for_culture(key: String) -> Array:
	var out: Array = []
	var faction_ids := {}
	for f in bridge.get_factions():
		var fd: Dictionary = f
		if String(fd.get("culture", "")) == key:
			faction_ids[int(fd.get("id", -1))] = true
	if faction_ids.is_empty():
		return out
	var all := bridge.settlements()
	for i in all.size():
		var s: Dictionary = all[i]
		if faction_ids.has(int(s.get("faction", -1))):
			out.append({"index": i, "data": s})
	return out


## (Re)fills `_culture_card_host` for culture `key`: the profile (terrain theme
## from `terrain_affinity`, `faction_count` and the culture's reach), the
## **Name pool** chips, and **Also used by**.
##
## ## The name pool is real settlements, not a fabricated roll
##
## There is no bound function that draws a free sample from a culture's
## syllable/suffix pool: `cartalith_civ::civ_settle_name` is not `#[func]`-exposed,
## and a hand-copied syllable table here would be exactly the second source of
## truth `get_cultures()`'s own doc comment refuses. So the chips are the REAL
## names of settlements belonging to factions currently assigned this culture
## (`bridge.settlements()` filtered through `bridge.get_factions()`'s `culture`),
## and pressing one rerolls it through `civ_reroll_settlement_name`, the call the
## Place editor's own dice button uses. That reroll draws from the faction's
## STORED culture (`cartalith_civ::civ_faction_culture` over the roster), the very
## field the picker above writes, so every chip rerolls from the pool on screen.
##
## Rebuilt under `_clear()`'s `_rebuilding` guard (FR-02), though nothing in the
## card holds a text field.
func _fill_culture_card(key: String) -> void:
	var host := _culture_card_host
	if host == null or not is_instance_valid(host):
		return
	_clear(host)
	var row := _culture_row(key)
	if row.is_empty():
		DccWidgets.note(host,
			"No profile for the culture \"%s\": this build's engine has no get_cultures() row for it." % key)
		return
	var cname := String(row.get("name", key.capitalize()))
	var affinity := String(row.get("terrain_affinity", ""))
	var fc := int(row.get("faction_count", 0))

	host.add_child(DccTheme.label("Culture profile — %s" % cname, "text_bright", DccTheme.FS_SMALL))
	## The terrain sentence is about the verdict on the Territory tab, which
	## judges this faction's own land against `terrain_affinity`; `common` and
	## `imperial` carry none by design and `civ_culture_terrain_fit` returns no
	## verdict for them rather than invent one.
	DccWidgets.note(host,
		("Terrain theme: %s. The Territory tab compares this faction's own land against it." % affinity.capitalize())
		if affinity != "" else
		"No terrain theme: Common and Imperial are identity-flavoured, and civ_culture_terrain_fit gives no verdict for either rather than fabricating one.")
	var reach := "%d faction%s use%s this culture" % [fc, "" if fc == 1 else "s", "s" if fc == 1 else ""]
	if fc > 0:
		reach += " · %d settlements · %s people" % [
			int(row.get("settlement_count", 0)), _thousands(int(row.get("population", 0)))]
	DccWidgets.note(host, reach)

	var pool := DccWidgets.group(host, "Name pool — real settlements")
	pool.name = "NamePool"
	var matches := _settlements_for_culture(key)
	if matches.is_empty():
		DccWidgets.note(pool,
			"No settlements currently use this culture. Give it to a faction that holds settlements to see the names it actually produces.")
	else:
		matches.sort_custom(func(a, b): return int((a.data as Dictionary).get("population", 0)) > int((b.data as Dictionary).get("population", 0)))
		var shown: Array = matches.slice(0, mini(NAME_POOL_MAX, matches.size()))
		var flow := HFlowContainer.new()
		flow.add_theme_constant_override("h_separation", 6)
		flow.add_theme_constant_override("v_separation", 6)
		pool.add_child(flow)
		for e in shown:
			var s: Dictionary = e.data
			var idx: int = e.index
			## Every row is a settlement of a faction assigned this culture, and
			## the reroll draws from that faction's assigned culture, so each one
			## rerolls from the pool on screen.
			var chip := DccWidgets.chip(flow, String(s.get("name", "?")),
				func(): _reroll_name(idx), false, 9, 4)
			chip.name = "Reroll_%d" % idx
			chip.tooltip_text = "%s — %s, faction %d. Click to reroll a fresh name from this pool." % [
				String(s.get("name", "?")), String(s.get("kind", "?")).capitalize(),
				int(s.get("faction", 0))]
		if matches.size() > shown.size():
			DccWidgets.note(pool, "+%d more, sorted by population." % (matches.size() - shown.size()))
		DccWidgets.note(pool,
			"Names are drawn from the faction's assigned culture -- by Auto-populate, by a Settlement drop left unnamed, and by a reroll. Changing a faction's culture changes the pool those draw from next, and its Territory-fit reading; it does not rename settlements already placed. A new world starts every faction on its default culture.")

	## Everyone on this culture but the faction being edited: "also" is relative
	## to the selection, and the selected faction is already the page's subject.
	var others: Array = []
	for f in bridge.get_factions():
		var fd: Dictionary = f
		if String(fd.get("culture", "")) == key and int(fd.get("id", -1)) != _selected:
			others.append(fd)
	var also := DccWidgets.group(host, "Also used by (%d)" % others.size())
	also.name = "AlsoUsedBy"
	if others.is_empty():
		DccWidgets.note(also, "No other faction uses this culture.")
	for fd in others:
		var frow := HBoxContainer.new()
		frow.add_theme_constant_override("separation", 8)
		var banner := FactionBanner.new()
		banner.configure(int(fd.get("id", 0)), _color_of(fd), 18)
		frow.add_child(banner)
		var lbl := DccTheme.label("%s — %d settlements, %s pop" % [
			String(fd.get("name", "?")), int(fd.get("settlement_count", 0)),
			_thousands(int(fd.get("population", 0)))], "text", DccTheme.FS_SMALL)
		lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		frow.add_child(lbl)
		also.add_child(frow)

	## Carried over from the retired window's "Not built" block. Verified
	## 2026-10-05 against `cartalith-urban/src/rules.rs`: it resolves exactly two
	## named profiles, "medieval" and "venus".
	DccWidgets.note(host,
		"Settlement Style is not part of culture: a settlement's built form (streets, parcels, walls) is a separate axis from naming, and is not assignable per faction. cartalith-urban resolves two named profiles, \"medieval\" and \"venus\", chosen per generation call and stored on no faction or settlement.")


## Reroll settlement `idx`'s name from its faction's culture pool, then bring
## every surface that shows it into line: `roster_changed` (the map's pins and
## labels and the dock, via `civilization_workspace.gd::_on_roster_changed` ->
## `_on_civ_edited`), the hidden panes (the Settlements tab lists the names, so
## they are dropped to rebuild on their next show) and the card itself.
##
## **Emits after the engine call**, never before (MISTAKES: emit after). The
## retired window emitted nothing here, which left map labels on the old name
## until something else repainted them; this is the one behaviour change in the
## fold, made on purpose. Must never write the culture.
func _reroll_name(idx: int) -> void:
	var new_name := bridge.civ_reroll_settlement_name(idx)
	if new_name == "":
		app.set_status("hint", "Couldn't reroll that name: the engine returned none.", "accent")
		return
	app.set_status("hint", "Rerolled — %s" % new_name, "text_ghost")
	roster_changed.emit()
	_drop_panes(_tab)
	var d := _faction(_selected)
	if d.is_empty():
		return
	_fill_culture_card(String(d.get("culture", "common")))
	if _phone:
		app.phone_fit(self, 1.0)  # idempotent; tags only the new chips


func _ag_tech_choice(parent: Control, current: String) -> void:
	var vocab := bridge.civ_ag_tech_vocabulary()
	if vocab.is_empty():
		return
	var keys: Array = []
	var labels: Array = []
	var hint := ""
	for e in vocab:
		var d: Dictionary = e
		keys.append(String(d.get("key", "")))
		labels.append(String(d.get("label", "?")))
		if String(d.get("key", "")) == current:
			hint = String(d.get("hint", ""))
	DccWidgets.choice(parent, "Ag. technology", labels, maxi(0, keys.find(current)),
		func(i: int): _set_field("ag_tech", keys[i]),
		"Live since 2026-08-25: farmersPerUrbanite is the agricultural labour ratio the manpower model runs on (the Military tab), so changing this moves this faction's standing army, field army, emergency levy and war duration. It is deliberately NOT the driver — it enters as one of five variables, and government, roads, water and the land itself move the answer as much.")
	if hint != "":
		DccWidgets.note(parent, hint)


# -- Default settlement type (FH-3, `FACTION_HUB_DESIGN.md` §3.5 Identity) ---

## The "Default settlement type" picker: which `SettlementTypeStore` type the
## settlement tool applies when a place is dropped in this faction's territory
## (`SettlementTypeStore.apply_default_to_settlement`, called from
## `civilization_workspace.gd::_settlement_click`). It moved here from the
## Settlement types window's per-faction column (FH-3); that window is now only
## the library and links back to this row.
##
## **"None" is entry 0 and means today's unmodified behaviour** -- the store's
## own `""` ("no default", `faction_default`'s contract), the same entry the old
## column offered. A faction whose default type was deleted already reads
## `""` (`SettlementTypeStore.delete_type` clears the id out of every default),
## and a stored id naming no type at all (a hand-edited project file) is shown as
## None too: `ids.find()` misses and `maxi(0, ...)` lands on entry 0. The store
## is not rewritten for that case, so no read of it can lose a document.
##
## **Persistence mirrors the old column exactly:** a pick calls
## `SettlementTypeStore.set_faction_default` and nothing else. The store is a
## static with no change signal and no dirty flag; `app.gd::_project_documents()`
## serialises `SettlementTypeStore.document()` at every save, so there is nothing
## to mark. Not touching the engine, it needs no `roster_changed` either -- no
## label, list row or other tab reads a default type.
##
## The faction id is captured at build time, not read from `_selected` in the
## callback: the pane is rebuilt on every faction switch, and a callback that
## re-read `_selected` could write one faction's pick into another's row (the
## FR-02 hazard in a new place). Like the other pickers it holds no focus, so
## `_commit_focused_field()` has no text field of its own to flush here.
##
## The option list is read from the store when the pane is built; the library is
## edited in another window, so `open()` (which rebuilds every pane) is what
## refreshes it. With no type authored the picker still shows "None", plus a note
## saying where types are made, rather than an inert-looking empty control.
func _default_type_choice(parent: Control) -> void:
	var fid := _selected
	var labels: Array = ["None"]
	var ids: Array = [""]
	for t in SettlementTypeStore.types():
		var td: Dictionary = t
		labels.append(String(td.get("name", "Type")))
		ids.append(String(td.get("id", "")))
	var ob := DccWidgets.choice(parent, "Default settlement type", labels,
		maxi(0, ids.find(SettlementTypeStore.faction_default(fid))),
		func(i: int): SettlementTypeStore.set_faction_default(fid, String(ids[i])),
		"The settlement tool applies this type's bundle (kind, specialisation, traits, walls, age policy) to a place dropped in this faction's territory. None behaves exactly as before. Types are authored in the Settlement types window.")
	ob.name = "DefaultType"
	if ids.size() == 1:
		DccWidgets.note(parent, "No settlement types yet -- create them with Settlement types… in the Civilization dock's Factions category.")


# -- Currency (Ruling R, kept by AR; Ruling AU) -----------------------------

## Held so a committed edit can refresh the example line without rebuilding
## the inspector under the field the user is typing in.
var _currency_example: Label

## The faction's own money -- Ruling AU: *"Each faction carries its own
## currency; the user types its rate in the faction roster and the engine only
## converts. No rate is derived from the economy."* Three free fields in the
## Identity rows' own shape (the `_colour_row` label column).
##
## **An unset member is shown as a placeholder, never as a value.** The field
## stays empty and its ghost text says what stands in -- "<faction> currency",
## "¤", "1 (at par, default)" -- so nothing reads as a choice nobody made.
## Clearing a field is how you get the default back.
##
## The rate is sent as typed text: `civ_set_faction_currency` parses it, so
## "abc" is refused rather than read as 0 by `to_float()`, and a refusal
## puts the stored value back.
func _build_currency(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Currency")
	var cur := bridge.civ_faction_currency(_selected)
	if cur.is_empty():
		DccWidgets.note(sec, "Unclaimed land issues no currency: trade into it reads in world price-index units.")
		return
	_currency_field(sec, "Name", "name", cur, "The currency's name, shown beside this faction's trade values.")
	_currency_field(sec, "Symbol", "symbol", cur, "Drawn beside every amount in this currency. Up to six characters.")
	_currency_field(sec, "Rate", "rate", cur,
		"Units of this currency per one unit of the world price index -- the scarcity price the trade match gives every good, 1 for a good whose world demand and supply balance. At 12, a balanced good costs 12 of this faction's money. You set it; the engine only converts, and no trade flow changes with it.")
	_currency_example = DccWidgets.note(sec, "")
	_refresh_currency_example()

func _currency_field(parent: Control, label_text: String, key: String, cur: Dictionary, tip: String) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = 24
	row.tooltip_text = tip
	var l := DccTheme.mono_label(label_text, "text_dim", DccTheme.FS_SMALL, 0)
	l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
	l.clip_text = true
	row.add_child(l)
	var le := LineEdit.new()
	le.name = "Currency_" + key
	le.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	le.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	le.tooltip_text = tip
	le.text = _currency_text(cur, key)
	le.placeholder_text = _currency_placeholder(cur, key)
	DccWidgets.well(le)
	le.text_submitted.connect(func(t: String): _set_currency(key, t, le))
	## Guarded against its own teardown -- see `_rebuilding` (FR-02).
	le.focus_exited.connect(func():
		if _rebuilding:
			return
		_set_currency(key, le.text, le))
	row.add_child(le)
	parent.add_child(row)

## The stored value, or "" when the member is unset (the placeholder shows the
## default then).
static func _currency_text(cur: Dictionary, key: String) -> String:
	if bool(cur.get(key + "_default", true)):
		return ""
	if key == "rate":
		return _rate_text(float(cur.get("rate", 0.0)))
	return String(cur.get(key, ""))

static func _currency_placeholder(cur: Dictionary, key: String) -> String:
	if key == "rate":
		return "1 (at par, default)"
	return "%s (default)" % String(cur.get(key, ""))

## `%g`-style: 12.5 not 12.500000, and 0.001 not 0.0.
static func _rate_text(r: float) -> String:
	return String.num(r, 6)

## Writes one member, then says what happened. The engine call comes first
## and `roster_changed` after it, so a listener re-reading the currency sees
## the new value (`MISTAKES.md`, "Emit a change signal").
func _set_currency(key: String, value: String, le: LineEdit) -> void:
	var before := bridge.civ_faction_currency(_selected)
	if value.strip_edges() == _currency_text(before, key):
		return
	if not bridge.civ_set_faction_currency(_selected, key, value):
		var why := "a number above zero" if key == "rate" else "at most six characters"
		app.set_status("hint", "Rejected -- a currency %s must be %s. Kept %s." % [
			key, why, _currency_text(before, key) if _currency_text(before, key) != "" else "the default"], "accent")
		if is_instance_valid(le):
			le.text = _currency_text(before, key)
		return
	var cur := bridge.civ_faction_currency(_selected)
	if is_instance_valid(le):
		le.text = _currency_text(cur, key)
		le.placeholder_text = _currency_placeholder(cur, key)
	_refresh_currency_example()
	roster_changed.emit()

func _refresh_currency_example() -> void:
	if _currency_example == null or not is_instance_valid(_currency_example):
		return
	var p := bridge.civ_price_in_currency(1.0, _selected)
	if p.is_empty():
		_currency_example.text = ""
		return
	_currency_example.text = "A good at world price 1 costs %s %s here%s. Trade values under Civilization ▸ Economy ▸ Trade flows are shown in the importer's currency." % [
		_rate_text(float(p.get("amount", 0.0))), String(p.get("symbol", "")),
		" (at par: no rate set)" if bool(p.get("rate_default", true)) else ""]


# -- Tariffs (Ruling AE) ------------------------------------------------------

## `OUTSTANDING_WORK.md`'s "Tariffs have no control": the rate THIS faction,
## as importer, levies on goods arriving from every other faction --
## `cartalith_civ::trade::Tariff` is directional and stored on the
## *importer's* row (`civ_roster_bridge.rs::FactionEntry::tariffs`), so the
## importer's own inspector pane is the natural home: one row per possible
## exporter, in the Currency block's own label-column shape, right below it
## -- both are a faction's own economic policy over the trade match.
##
## Unclaimed (`_selected == 0`) cannot levy one -- `FactionRoster::set_tariff`
## refuses it outright ("Unclaimed... has no government to levy with") -- so
## the section says that rather than drawing rows nobody could fill in.
##
## A rate is typed as a percentage (0-100) and stored as the engine's `0..=1`
## fraction. Blank is "no tariff", which is also the engine's own encoding --
## `set_tariff`'s own doc: "rate 0.0 removes the row, so 'no tariff' has
## exactly one encoding" -- so an empty field is the real absent state, never
## a fake zero standing in for one (`MISTAKES.md`).
func _build_tariffs(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Tariffs")
	if _selected <= 0:
		DccWidgets.note(sec, "Unclaimed levies no tariffs: it has no government to levy with.")
		return
	var others: Array = []
	for f in bridge.get_factions():
		var d: Dictionary = f
		if int(d.get("id", -1)) != _selected:
			others.append(d)
	if others.is_empty():
		DccWidgets.note(sec, "No other faction to tax yet -- add one first.")
		return
	for d in others:
		_tariff_row(sec, int(d.get("id", -1)), String(d.get("name", "?")))
	DccWidgets.note(sec,
		"The rate this faction charges on goods crossing in FROM the faction named, as importer -- "
		+ "the reverse direction is that faction's own row, not this one. Applied by the trade match "
		+ "(Civilization ▸ Economy ▸ Trade flows); the busiest-partners readout below re-matches "
		+ "itself on an edit when a match is already held, so its numbers move without a manual re-run.")

func _tariff_row(parent: Control, exporter_id: int, exporter_name: String) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = 24
	row.tooltip_text = ("The fraction of every flow's volume this faction's customs removes when it "
		+ "arrives from %s. Blank or 0 is no tariff; 100 is an embargo -- the flow's volume reaches "
		+ "zero and it drops out of the match entirely.") % exporter_name
	var l := DccTheme.mono_label(exporter_name, "text_dim", DccTheme.FS_SMALL, 0)
	l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
	l.clip_text = true
	row.add_child(l)
	var le := LineEdit.new()
	le.name = "Tariff_%d" % exporter_id
	le.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	le.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	le.tooltip_text = row.tooltip_text
	le.placeholder_text = "0 (none)"
	le.text = _tariff_text(bridge.civ_trade_tariff(_selected, exporter_id))
	DccWidgets.well(le)
	le.text_submitted.connect(func(t: String): _set_tariff(exporter_id, t, le))
	## Guarded against its own teardown -- see `_rebuilding` (FR-02), the same
	## guard `_currency_field`'s `focus_exited` uses.
	le.focus_exited.connect(func():
		if _rebuilding:
			return
		_set_tariff(exporter_id, le.text, le))
	row.add_child(le)
	parent.add_child(row)

## `""` for no tariff (the engine's own zero, and the field's own blank
## state), else the percentage with no trailing zeros -- `12` not
## `12.000000`, `0.5` not `0.005`.
static func _tariff_text(rate: float) -> String:
	if rate <= 0.0:
		return ""
	return String.num(rate * 100.0, 4)

## Writes one tariff row, then says what happened. Percentage in, `0..=1`
## fraction out to the engine; a blank field means 0 (no tariff), the
## engine's own encoding for "unset" -- never a fake number standing in for
## it. The engine call comes first and `tariff_changed` after it, so a
## listener re-reading the tariff sees the new value (`MISTAKES.md`, "Emit a
## change signal").
func _set_tariff(exporter_id: int, value: String, le: LineEdit) -> void:
	var before := bridge.civ_trade_tariff(_selected, exporter_id)
	var text := value.strip_edges()
	if text == _tariff_text(before):
		return
	var pct := 0.0
	if text != "":
		if not text.is_valid_float():
			app.set_status("hint",
				"Rejected -- a tariff must be a number from 0 to 100. Kept %s." % _tariff_display(before),
				"accent")
			if is_instance_valid(le):
				le.text = _tariff_text(before)
			return
		pct = text.to_float()
	var rate := pct / 100.0
	if not bridge.civ_set_trade_tariff(_selected, exporter_id, rate):
		app.set_status("hint",
			"Rejected -- a tariff must be 0 to 100%%. Kept %s." % _tariff_display(before), "accent")
		if is_instance_valid(le):
			le.text = _tariff_text(before)
		return
	var after := bridge.civ_trade_tariff(_selected, exporter_id)
	if is_instance_valid(le):
		le.text = _tariff_text(after)
	tariff_changed.emit()

static func _tariff_display(rate: float) -> String:
	return _tariff_text(rate) if rate > 0.0 else "0"


# -- Territory fit (`_civTerrainFitHtml`) -----------------------------------

func _build_terrain_fit(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Territory fit")
	_ensure_fits()
	var fit := _fit_for(_selected)
	if fit.is_empty():
		DccWidgets.note(sec, "Reopen the roster to recompute terrain composition.")
		return
	if String(fit.get("absent", "")) == "no_claim_grid":
		## Composition is a share of the faction's claimed cells; with no claim
		## grid there is no mix, and an all-zero one would read "no river".
		DccWidgets.note(sec, "Composition: —   " + NO_CLAIM_GRID)
		return
	var mix: Dictionary = fit.get("mix", {})
	var parts: Array[String] = []
	for k in ["river", "coast", "arid", "forest", "hills"]:
		parts.append("%s %d%%" % [String(k).capitalize(), int(round(float(mix.get(k, 0.0)) * 100.0))])
	if bool(fit.get("has_verdict", false)):
		var verdict := String(fit.get("verdict", "typical"))
		var word := "a strong match" if verdict == "match" else ("a mismatch" if verdict == "mismatch" else "roughly typical")
		var line := DccTheme.label("%s territory: %d%% vs. world average %d%% — %s for a %s culture." % [
			String(fit.get("key", "?")).capitalize(),
			int(round(float(fit.get("value", 0.0)) * 100.0)),
			int(round(float(fit.get("world_mean", 0.0)) * 100.0)),
			word, String(fit.get("culture", "?")).capitalize()], "text", DccTheme.FS_SMALL)
		line.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		## `c("good")` since 2026-09-03, and this row is the reason that token
		## now exists. It was the literal `Color(0.48, 0.78, 0.49)` -- #7ac77d,
		## the same colour in both themes, which measures **1.96:1** against the
		## light `panel` #fbfaf7 and is unreadable there. It was also the one raw
		## chrome colour left in any of the nine windows, and therefore the one
		## thing in this file a theme flip could not repaint: `DccTheme.remap()`
		## matches a baked colour back to the token that produced it, and a
		## literal matches no token.
		##
		## The `mismatch` half stays `accent` deliberately. A mismatch is a
		## remark, not a failure -- for common/imperial the engine returns no
		## verdict at all rather than fabricating one, which is the `else` branch
		## below -- so `block`, which this shell spends on conflicts and
		## destructive actions, would overstate it.
		if verdict == "match":
			line.add_theme_color_override("font_color", DccTheme.c("good"))
		elif verdict == "mismatch":
			line.add_theme_color_override("font_color", DccTheme.c("accent"))
		sec.add_child(line)
	else:
		DccWidgets.note(sec,
			"%s culture has no terrain theme — composition shown for reference only. The engine "
			% String(fit.get("culture", "?")).capitalize()
			+ "returns no verdict for common/imperial rather than fabricating one, which is the "
			+ "reference's own discipline.")
	DccWidgets.note(sec, " · ".join(parts))


# -- Territory tab (FACTION_HUB_DESIGN §3.5, FH-4) ---------------------------

## The Territory tab: what the selected faction holds and how it holds it.
## Order is the design's: the claim figures first (they are what the verdict
## and the mix are fractions of), then the fit, the provinces, and last the
## on-demand influence reading. Builds only this tab; `_ensure_fits()` is the
## sole O(cells) call it makes unprompted, and `civ_territory_influence()` is
## never called here -- only by its button. Never offers to clear a faction's
## claims (the design's §3.5 leaves that to the Territories tool, deliberately).
func _build_tab_territory(parent: Control, d: Dictionary) -> void:
	_build_claims_block(parent, d)
	_build_terrain_fit(parent)
	_build_provinces_block(parent)
	_build_influence_block(parent)


## The claim figures and the two actions. The "Capital / Settlements / Settled
## population" line stays here, as it did in the Overview block this replaces,
## until the Settlements tab carries it (`_claimsabsent_probe` and
## `_factionhub_probe` still read "Capital:" on this tab).
##
## Absent data is honest: `civ_faction_territory_stats` answers `{}` when there
## is no claim grid, and every number then reads "—" with `NO_CLAIM_GRID`, never
## a zero (`MISTAKES.md`: never encode "no value" as a plausible value).
func _build_claims_block(parent: Control, d: Dictionary) -> void:
	var sec := DccWidgets.section(parent, "Claims")
	var stats := bridge.civ_faction_territory_stats(_selected)
	var cap := _capital_of(_selected)
	var cap_name: String = String(cap.get("name", "")) if not cap.is_empty() else "none"
	DccWidgets.note(sec, "Capital: %s   ·   Settlements: %d   ·   Settled population: %s" % [
		cap_name, int(d.get("settlement_count", 0)), _thousands(int(d.get("population", 0)))])
	var line: Label
	if not stats.is_empty():
		line = DccWidgets.note(sec, "Territory: %s km² over %d claimed cells (%d contested)" % [
			_thousands(int(float(stats.get("area_km2", 0.0)))),
			int(stats.get("claimed_cells", 0)), int(stats.get("contested_cells", 0))])
	else:
		## With a faction on screen there is a world, so an empty answer has
		## one cause: `civ_faction_territory_stats`' "unknown" (its Rust doc).
		line = DccWidgets.note(sec, "Territory: —   " + NO_CLAIM_GRID)
	line.name = "TerritoryClaims"
	_build_stale_chip(sec)
	var claim := DccWidgets.action(sec, "Claim cells for this faction", func():
		## Hidden first so the map is reachable to paint on, the same as the
		## focus button below. The handler is the context card's own
		## `civ.faction_claim` action (`CivilizationWorkspace.claim_for_faction`):
		## it arms the Territory tool with this faction picked. Nothing is painted.
		var fid := _selected
		hide()
		app.claim_cells_for_faction(fid))
	claim.name = "ClaimCells"
	claim.tooltip_text = "Arms the Territory tool with this faction picked, as the map context card's 'Claim for …' row does. Nothing is painted until you brush."
	if not cap.is_empty():
		var focus := DccWidgets.action(sec, "Focus on capital", func():
			app.viewport.move_view_to(float(int(cap.get("x", 0))), float(int(cap.get("y", 0))))
			hide())
		focus.name = "FocusCapital"


## The stale-claims chip. Driven by the engine's own staleness graph:
## `bridge.stale_stages()` (cheap, recomputes nothing) carries a `civ` entry when
## the settlements were edited after the civilisation layer was last computed,
## which is exactly when the capital positions and the claim grid under these
## figures may disagree with the map. Hand-painted claims survive a recompute.
## Shown ONLY when the entry is present: `{}` also means "generating" or "an
## older binary", and neither is evidence of staleness, so absence draws nothing
## rather than a reassuring "up to date".
func _build_stale_chip(parent: Control) -> void:
	var stale: Dictionary = bridge.stale_stages()
	if not stale.has("civ"):
		return
	var e: Dictionary = stale["civ"] if stale["civ"] is Dictionary else {}
	var why := String(e.get("reason", ""))
	if why.is_empty():
		why = String(e.get("origin", "an earlier edit"))
	var chip := Label.new()
	chip.name = "StaleClaimsChip"
	chip.text = "Claims may be stale — %s. Civilization ▸ Settlements ▸ Recompute civilisation refreshes them; hand-painted claims survive." % why.replace("_", " ")
	chip.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	chip.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	chip.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	chip.add_theme_color_override("font_color", DccTheme.c("stale"))
	var pc := PanelContainer.new()
	pc.name = "StaleClaimsPanel"
	pc.add_theme_stylebox_override("panel", DccWidgets.box("stale", "", 8, 4))
	pc.add_child(chip)
	parent.add_child(pc)


## The faction's provinces: name and capital. `bridge.provinces()` rows carry
## `{id, faction, name, capital_settlement_index}` and NO per-province cell
## count (the design's "cells" column needs a Rust `get_provinces` addition), so
## the column is omitted rather than shown as a plausible number. The capital is
## looked up in `bridge.settlements()` by index; an index outside it reads "—".
func _build_provinces_block(parent: Control) -> void:
	var rows: Array = []
	for p in bridge.provinces():
		if int((p as Dictionary).get("faction", -1)) == _selected:
			rows.append(p)
	var sec := DccWidgets.section(parent, "Provinces (%d)" % rows.size())
	sec.name = "ProvincesSection"
	if rows.is_empty():
		DccWidgets.note(sec, "This faction holds no province.")
		return
	var settlements := bridge.settlements()
	for p in rows:
		var pd: Dictionary = p
		var ci := int(pd.get("capital_settlement_index", -1))
		var cname := "—"
		if ci >= 0 and ci < settlements.size():
			cname = String((settlements[ci] as Dictionary).get("name", "—"))
		var l := DccWidgets.note(sec, "%s   ·   capital %s" % [String(pd.get("name", "?")), cname])
		l.name = "Province_%d" % int(pd.get("id", 0))
	DccWidgets.note(sec, "Cells per province are not exposed by the engine yet, so none is shown.")


## "Influence by neighbour": this faction's row and the borders it takes part
## in, from `civ_territory_influence()` -- the same call as
## `Territories ▸ Borders & influence ▸ By faction`, filtered to the selection.
## Behind a button and cached (see `_influence`): the field is rebuilt from the
## capitals by one Dijkstra each, and is NOT the painted claims, which the
## caption says so the two are never read as one.
func _build_influence_block(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Influence by neighbour")
	var run := DccWidgets.action(sec, "Analyse influence", func():
		_influence = bridge.civ_territory_influence()
		_influence_ready = true
		_fill_influence())
	run.name = "AnalyseInfluence"
	run.tooltip_text = "Rebuilds the cost-distance influence field from the capitals and reports this faction's reach and the borders it shares. Computed on demand and dropped; the same reading as Territories ▸ Borders & influence."
	_influence_body = VBoxContainer.new()
	_influence_body.name = "InfluenceBody"
	_influence_body.add_theme_constant_override("separation", 4)
	_influence_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	sec.add_child(_influence_body)
	_fill_influence()


## Fills `_influence_body` from the cache, or says it has not been run. Pure
## presentation over `_influence`; never calls the engine.
func _fill_influence() -> void:
	if _influence_body == null or not is_instance_valid(_influence_body):
		return
	_clear(_influence_body)
	if not _influence_ready:
		DccWidgets.note(_influence_body,
			"Not run yet. Reach is computed from capitals, not read from painted claims.")
		return
	if _influence.is_empty():
		## `{}` is the engine's "nothing to analyse": a world with no capital, or
		## one reopened from a save without its terrain rasters.
		DccWidgets.note(_influence_body,
			"—   No influence to report: this world has no capitals to project territory from, or it was opened from a save without its terrain rasters. Regenerate to use it.")
		return
	var mine: Dictionary = {}
	for r in (_influence.get("factions", []) as Array):
		if int((r as Dictionary).get("id", -1)) == _selected:
			mine = r
	if mine.is_empty():
		DccWidgets.note(_influence_body, "—   This faction has no capital reach on this map.")
	else:
		DccWidgets.note(_influence_body, "Reach: %s cells, %s on a frontier; mean reach %.1f, mean contest %.3f" % [
			_thousands(int(mine.get("cells", 0))), _thousands(int(mine.get("frontier_cells", 0))),
			float(mine.get("mean_influence", 0.0)), float(mine.get("mean_contested", 0.0))])
	var shown := 0
	for r in (_influence.get("borders", []) as Array):
		var b: Dictionary = r
		var a_is := int(b.get("a", -1)) == _selected
		if not a_is and int(b.get("b", -1)) != _selected:
			continue
		var other := String(b.get("b_name", "?")) if a_is else String(b.get("a_name", "?"))
		var l := DccWidgets.note(_influence_body, "%s -- %s frontier cells, mean contest %.3f" % [
			other, _thousands(int(b.get("cells", 0))), float(b.get("mean_contested", 0.0))])
		l.name = "InfluenceNeighbour_%d" % shown
		shown += 1
	if shown == 0:
		DccWidgets.note(_influence_body, "No neighbour meets this faction on a frontier.")
	DccWidgets.note(_influence_body, "Computed from capital reach, not from painted claims.")


# -- Military (`GUI_GAP_REGISTER.md` CV-25 / `MILITARY_MANPOWER_SCOPE.md`) --
#
# This is the block the roster's own "Not built" note used to disclaim. The
# reference's Power breakdown had no reader anywhere in this port until CV-25
# landed, and its manpower half had no model in either codebase until this
# pass built one.
#
# **The two numbers here answer different questions and are labelled as such.**
# `power.military` is the reference's own 0-100 heuristic -- a comparison
# against the other factions on THIS map, explicitly derived and never
# presented as simulated. The four headcounts are absolute and have no
# reference at all. Neither is a rescaling of the other, and presenting one as
# the other would be the easiest wrong thing to do here.

func _military_row(faction_id: int) -> Dictionary:
	for r in (_military.get("factions", []) as Array):
		if int((r as Dictionary).get("faction", -1)) == faction_id:
			return r
	return {}

func _build_military_block(parent: Control) -> void:
	_ensure_military()
	var row := _military_row(_selected)
	if row.is_empty():
		return
	var sec := DccWidgets.section(parent, "Military")
	DccWidgets.note(sec, "Power: %d/100 relative to the other factions   ·   %d of %d settlements fortified (%d stone, %d palisade, %d ditch)" % [
		int(round(float(row.get("military", 0.0)))), int(row.get("fortified_count", 0)),
		int(row.get("settlement_count", 0)), int(row.get("walled_stone", 0)),
		int(row.get("walled_palisade", 0)), int(row.get("walled_ditch", 0))])

	var m: Dictionary = row.get("manpower", {})
	if String(row.get("absent", "")) == "no_claim_grid":
		## `civ_military_summary`'s own doc: the manpower model's land capacity
		## and road density are claimed-cell integrals, so it is withheld.
		DccWidgets.note(sec, "Standing army, field army and levy: —   " + NO_CLAIM_GRID)
		return
	if m.is_empty():
		return
	DccWidgets.note(sec, "Standing army %s (professional core %s)   ·   sustainable field army %s   ·   emergency levy %s" % [
		_thousands(int(round(float(m.get("standing_army", 0.0))))),
		_thousands(int(round(float(m.get("professional_core", 0.0))))),
		_thousands(int(round(float(m.get("field_army", 0.0))))),
		_thousands(int(round(float(m.get("emergency_mobilization", 0.0)))))])
	DccWidgets.note(sec, "Out of a total population of %s (%.0f%% in farming), of whom %s are of military age. A field army stays out %d days; a full levy %d." % [
		_thousands(int(round(float(m.get("total_population", 0.0))))),
		100.0 * float(m.get("agricultural_labour_ratio", 0.0)),
		_thousands(int(round(float(m.get("mobilization_pool", 0.0))))),
		int(round(float(m.get("field_duration_days", 0.0)))),
		int(round(float(m.get("emergency_duration_days", 0.0))))])
	## The era bands are shares of the CITIZEN / FREE population, not of the
	## total (owner ruling, 2026-08-25 -- the supplied specification's own
	## Republican Rome figure is quoted as "17-29 % of its citizen
	## population"). So that body is named on the line above the verdict
	## rather than being an invisible divisor: a reader has to be able to see
	## what the percentage is a percentage of, and how the government produced
	## it.
	DccWidgets.note(sec, "Citizen / free population %s — %.0f%% of the total, the share a %s confers. This is what the era bands below are measured against, not the whole population." % [
		_thousands(int(round(float(m.get("citizen_population", 0.0))))),
		100.0 * float(m.get("citizen_fraction", 0.0)),
		String(m.get("government", "?")).replace("_", " ")])
	DccWidgets.note(sec, "Reads as a %s (%s). Standing %.2f%% of citizens — %s that era's %.1f–%.1f%% band; mobilization %.1f%% — %s its %.0f–%.0f%%." % [
		String(m.get("era", "?")), String(m.get("era_constraint", "")),
		100.0 * float(m.get("standing_citizen_share", 0.0)),
		String(m.get("era_standing_verdict", "?")),
		100.0 * float(m.get("era_standing_lo", 0.0)), 100.0 * float(m.get("era_standing_hi", 0.0)),
		100.0 * float(m.get("emergency_citizen_share", 0.0)),
		String(m.get("era_mobilization_verdict", "?")),
		100.0 * float(m.get("era_mobilization_lo", 0.0)),
		100.0 * float(m.get("era_mobilization_hi", 0.0))])
	## Closes the modal on the way, the same shape `_build_settlement_sublist`
	## already uses -- leaving a roster window open over the category it just
	## navigated to would hide the thing it sent the reader to look at.
	var go := DccWidgets.action(sec, "Full breakdown → Civilization ▸ Military",
		func():
			hide()
			app.select_domain_category("civilization", "Military"))
	go.alignment = HORIZONTAL_ALIGNMENT_LEFT


# -- Settlement sublist (`_civRenderFactionSettlementSublist`) --------------

func _build_settlement_sublist(parent: Control) -> void:
	var all := bridge.settlements()
	var mine: Array = []
	for i in all.size():
		var s: Dictionary = all[i]
		if int(s.get("faction", 0)) == _selected:
			mine.append({"index": i, "data": s})
	var grp := DccWidgets.group(parent, "Settlements (%d)" % mine.size(), false)
	if mine.is_empty():
		DccWidgets.note(grp, "No settlements yet. Paint territory or drop one with the Settlement tool.")
		return
	mine.sort_custom(func(a, b): return int(a.data.population) > int(b.data.population))
	for e in mine:
		var s: Dictionary = e.data
		var idx: int = e.index
		var b := DccWidgets.action(grp, "%s — %s, %s" % [
			String(s.get("name", "?")), String(s.get("kind", "?")).capitalize(),
			_thousands(int(s.get("population", 0)))],
			func():
				app.viewport.move_view_to(float(int(s.get("x", 0))), float(int(s.get("y", 0))))
				hide()
				app.open_place_editor(idx))
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.tooltip_text = "Centre the map on it and open its editor -- the reference's own sublist row action."


## The "Not built" block, now at the foot of the Economy tab because what it
## lists is economic: the Power breakdown's non-military axes, tax and trade
## income, craft share. (Its old second paragraph, on relations, moved to the
## Relations placeholder, where it belongs.)
##
## Corrected 2026-09-24 (audit B18): it listed food, exports/imports and
## strategic resources as unbuilt, but `civilization_workspace.gd::
## _fill_faction_economy` draws them (Economy ▸ By faction, off
## `civ_faction_economy`). Still undrawn anywhere: `FactionPower`'s four
## non-military axes, `tax_income`, trade income, `craft_share` -- all from
## `_civFactionAggregates`.
func _build_gaps(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Not built")
	DccWidgets.note(sec,
		"Not shown in this window: the Power breakdown's four remaining axes (economic, "
		+ "political, cultural, religious), tax income, trade income and craft share. The "
		+ "military axis and the manpower model are live, on the Military tab. A faction's food "
		+ "capacity and surplus, exports, imports and strategic resources are under Civilization "
		+ "▸ Economy ▸ By faction.")


# -- Roster mutation --------------------------------------------------------

func _add_faction() -> void:
	var id := bridge.civ_add_faction()
	if id < 0:
		app.set_status("hint", "Generate a world before adding a faction.", "accent")
		return
	_selected = id
	_mark_data_stale()
	_rebuild()
	app.set_status("hint",
		"Faction %d added — it owns nothing until you paint territory or reassign a settlement." % id,
		"text_ghost")


## The reference confirms first, and names the consequence in the prompt
## ("Any settlements/territory using it will revert to Unclaimed") -- both
## kept, because both are the point.
func _confirm_remove() -> void:
	if bridge.civ_faction_count() <= 1:
		app.set_status("hint", "One faction plus Unclaimed is the floor — nothing to remove.", "accent")
		return
	## `DccWidgets.confirm()` -- see `right_dock.gd::_confirm_revert` for what a
	## hand-built `ConfirmationDialog` measures as on a phone. `get_ok_button()
	## .text` and `ok_button_text` are the same property by two routes; the
	## helper takes the text and sets it before the window is presented, which
	## the button-object route cannot guarantee.
	DccWidgets.confirm(app, "Remove the last faction?",
		"Remove faction %d?\n\nAny settlements and territory using it will revert to Unclaimed." % bridge.civ_faction_count(),
		"Remove",
		func():
			if bridge.civ_remove_faction():
				_selected = mini(_selected, bridge.civ_faction_count())
				_mark_data_stale()
				roster_changed.emit()
				_rebuild())


func _set_field(key: String, value: String) -> void:
	if not bridge.civ_set_faction_field(_selected, key, value):
		app.set_status("hint", "Rejected — %s is not a value the engine recognises." % value, "accent")
	if key == "culture":
		## The Territory verdict is a function of culture, and the manpower model
		## reads it too, so both caches go stale and every tab is rebuilt on its
		## next show. The visible pane is rebuilt now, as it always was.
		_mark_data_stale()
		## FR-02 (FH-2): the Culture card now sits in the same pane as the name
		## field, so a culture pick is the likeliest thing to tear down a
		## half-typed rename. Flush it against this faction first, or the
		## teardown guard drops it unwritten.
		_commit_focused_field()
		_rebuild_inspector()
	elif key == "government" or key == "ag_tech":
		## Neither is shown outside Identity, but the Military tab's whole answer
		## moves with both (they are two of the manpower model's five variables):
		## it used to keep showing the pre-edit numbers until the hub was reopened.
		## The visible Identity pane keeps its focused control; only the other,
		## hidden panes are dropped, to be rebuilt on their next show.
		_mark_data_stale()
		_drop_panes(_tab)
	elif key == "name":
		_rebuild_list()
		_rebuild_overview()
		roster_changed.emit()


# -- Helpers ----------------------------------------------------------------

func _faction(fid: int) -> Dictionary:
	for f in bridge.get_factions():
		if int((f as Dictionary).get("id", -1)) == fid:
			return f
	return {}


func _fit_for(fid: int) -> Dictionary:
	for f in _fits:
		if int((f as Dictionary).get("faction", -1)) == fid:
			return f
	return {}


func _capital_of(fid: int) -> Dictionary:
	## `_civFactionCapital`: the highest-pop capital/metropolis, else the
	## highest-pop settlement of any kind -- fully derived from kind/pop, no
	## override field, exactly as the reference derives it.
	var best := {}
	var best_pop := -1
	var best_seat := false
	for s in bridge.settlements():
		var d: Dictionary = s
		if int(d.get("faction", 0)) != fid:
			continue
		var kind := String(d.get("kind", ""))
		var seat := kind == "capital" or kind == "metropolis"
		var pop := int(d.get("population", 0))
		if (seat and not best_seat) or (seat == best_seat and pop > best_pop):
			best = d
			best_pop = pop
			best_seat = seat
	return best


func _color_of(d: Dictionary) -> Color:
	return Color8(int(d.get("color_r", 150)), int(d.get("color_g", 150)), int(d.get("color_b", 150)))


static func _thousands(n: int) -> String:
	var s := str(absi(n))
	var out := ""
	var c := 0
	for i in range(s.length() - 1, -1, -1):
		out = s[i] + out
		c += 1
		if c % 3 == 0 and i > 0:
			out = " " + out
	return ("-" if n < 0 else "") + out

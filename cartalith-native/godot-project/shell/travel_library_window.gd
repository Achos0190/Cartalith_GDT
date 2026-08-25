extends AcceptDialog
class_name TravelLibraryWindow

## `TRAVEL_LIBRARY_SPEC.md`'s Data ▸ Travel library… window (⇧L) --
## `design/Journey Planner DCC.dc.html`'s `2a` (menu item, already wired in
## `menus.gd`) and `2b` (this window: list + inspector). An ADDITION to the
## original DCC shell, not part of `DCC_SHELL_SPEC.md` -- §1 of the spec:
## "an information layer, nothing more... no route is stored, no plan is
## computed in this window."
##
## Structure per `2b`: tabbed by definition type (Animals & mounts /
## Vehicles / Vessels / Party set-ups), each tab an entries rail (Custom
## section + Stock section, counts per section, filter box) plus an
## inspector pane for the selected entry, grouped exactly as
## `TRAVEL_LIBRARY_SPEC.md` §3 groups each type's fields.
##
## **What is real vs. disclosed, control by control**: everything this file
## shows is real, live `#[func]` data (`lib.rs`'s `tl_*` block, wired this
## dispatch) -- list/get/duplicate/add-blank/delete/edit/reset-to-stock/
## capture-from-planner all round-trip through the actual GDExtension
## boundary, not client-side mock state. The one honest gap: wholly-new
## species (the stock Ox/Yak/Reindeer) and every vehicle/vessel definition
## are real, validated, inspectable data with **no live effect on a computed
## journey yet** -- `species_key` empty means no `JpParty` slot exists for
## it (`travel_library.rs`'s own module doc). Said plainly in the inspector
## rather than implied away.
##
## Edits are staged locally (`_draft`) and committed with the footer's own
## "save definition" button, matching `2b`'s own footer row exactly
## (save / duplicate / revert, "changes apply at next plan"). Stock entries
## are read-only everywhere in this file -- every field control for one is
## simply not built; "duplicate" is the only path to an editable copy,
## per §3.

const KINDS: Array[Dictionary] = [
	{"key": "animal", "label": "Animals & mounts"},
	{"key": "vehicle", "label": "Vehicles"},
	{"key": "vessel", "label": "Vessels"},
	{"key": "preset", "label": "Party set-ups"},
]

# ---------------------------------------------------------------------------
# Field specs -- TRAVEL_LIBRARY_SPEC.md §3's own groupings, data-driven so
# the inspector builder below is one generic walker rather than four
# hand-written forms that could drift from each other.
# ---------------------------------------------------------------------------
#
# Each field: {key, label, type, unit?, min?, max?, step?, options?}.
# type is one of: "text" (LineEdit), "number" (SpinBox), "toggle" (CheckBox),
# "choice" (OptionButton over `options`, an Array of [wire_value, display]
# pairs), "affinity" (a multiplier SpinBox + a Blocked CheckBox, used only
# by the vehicle off_road/ford pair -- the ten terrain rows have their own
# dedicated builder since they share one slug table and one header).

const ANIMAL_GROUPS: Array[Dictionary] = [
	{"title": "Classification", "fields": [
		{"key": "name", "label": "Name", "type": "text"},
		{"key": "roles", "label": "Role", "type": "roles"},
		{"key": "substitutes_for", "label": "Substitutes for", "type": "text"},
		{"key": "size_class", "label": "Size class", "type": "text"},
		{"key": "availability_kind", "label": "Availability", "type": "choice",
			"options": [["", "(unset)"], ["global", "Global"], ["regional", "Regional"]]},
		{"key": "availability_region", "label": "Region", "type": "text"},
	]},
	{"title": "Capacity & speed", "fields": [
		{"key": "load_capacity_kg", "label": "Load capacity", "type": "number", "unit": "kg", "min": 0, "max": 2000, "step": 1},
		{"key": "draft_pull_kg", "label": "Draft pull", "type": "number", "unit": "kg towed", "min": 0, "max": 3000, "step": 1},
		{"key": "base_speed_kmh", "label": "Base speed", "type": "number", "unit": "km/h", "min": 0, "max": 30, "step": 0.1},
		{"key": "sustainable_hours_day", "label": "Sustainable hours", "type": "number", "unit": "h/day", "min": 0, "max": 24, "step": 0.5},
		{"key": "forced_pace_cap", "label": "Forced-pace cap", "type": "number", "unit": "× base", "min": 1, "max": 3, "step": 0.05},
	]},
	{"title": "Sustenance", "fields": [
		{"key": "fodder_need_kg_day", "label": "Fodder need", "type": "number", "unit": "kg/day", "min": 0, "max": 50, "step": 0.5},
		{"key": "water_need_l_day", "label": "Water need", "type": "number", "unit": "L/day", "min": 0, "max": 100, "step": 1},
		{"key": "grazing_tolerance", "label": "Grazing tolerance", "type": "choice",
			"options": [["", "(unset)"], ["unrestricted", "Unrestricted"], ["grassland_only", "Full — grasslands only"], ["none", "None"]]},
		{"key": "waterless_limit_days", "label": "Waterless limit", "type": "number", "unit": "days", "min": 0, "max": 30, "step": 0.5},
	]},
	{"title": "Requirements & prohibitions", "fields": [
		{"key": "yokeable_to_wheeled", "label": "Yokeable to wheeled vehicles", "type": "toggle"},
		{"key": "requires_road_to_tow", "label": "Requires road/track to tow", "type": "toggle"},
		{"key": "blocked_by_seasonal_closures", "label": "Blocked by seasonal closures", "type": "toggle"},
		{"key": "carryable_aboard_vessel", "label": "Carryable aboard a vessel", "type": "toggle"},
		{"key": "usable_as_mount", "label": "Usable as a mount", "type": "toggle"},
		{"key": "handlers_required_per_n_head", "label": "Handlers per N head", "type": "number", "unit": "head", "min": 0, "max": 20, "step": 1},
	]},
	{"title": "Cost", "fields": [
		{"key": "upkeep_sp_day_head", "label": "Upkeep", "type": "number", "unit": "sp/day/head", "min": 0, "max": 50, "step": 0.1},
	]},
]

const VEHICLE_GROUPS: Array[Dictionary] = [
	{"title": "Classification", "fields": [
		{"key": "name", "label": "Name", "type": "text"},
		{"key": "class", "label": "Class", "type": "choice",
			"options": [["", "(unset)"], ["wheeled", "Wheeled"], ["dragged", "Dragged"]]},
	]},
	{"title": "Capacity & draft", "fields": [
		{"key": "load_kg", "label": "Load", "type": "number", "unit": "kg", "min": 0, "max": 5000, "step": 10},
		{"key": "draft_count", "label": "Draft head required", "type": "number", "unit": "head", "min": 0, "max": 12, "step": 1},
		{"key": "draft_role", "label": "Draft role", "type": "text"},
		{"key": "speed_mult", "label": "Speed", "type": "number", "unit": "× pace", "min": 0, "max": 2, "step": 0.05},
	]},
	{"title": "Road & terrain", "fields": [
		{"key": "road_requirement", "label": "Road requirement", "type": "choice",
			"options": [["", "(unset)"], ["none", "None"], ["track", "Track"], ["road", "Road"]]},
		{"key": "off_road", "label": "Off-road", "type": "affinity"},
		{"key": "ford", "label": "Ford", "type": "affinity"},
	]},
	{"title": "Other", "fields": [
		{"key": "carryable_aboard_vessel", "label": "Carryable aboard a vessel", "type": "toggle"},
	]},
]

const VESSEL_GROUPS: Array[Dictionary] = [
	{"title": "Classification", "fields": [
		{"key": "name", "label": "Name", "type": "text"},
		{"key": "modes", "label": "Mode", "type": "modes"},
	]},
	{"title": "Capacity & crew", "fields": [
		{"key": "hold_kg", "label": "Hold", "type": "number", "unit": "kg", "min": 0, "max": 400000, "step": 100},
		{"key": "crew_required", "label": "Crew required", "type": "number", "unit": "", "min": 0, "max": 300, "step": 1},
		{"key": "base_speed_kmh", "label": "Base speed", "type": "number", "unit": "km/h", "min": 0, "max": 30, "step": 0.5},
	]},
	{"title": "Water & sailing", "fields": [
		{"key": "water_rating", "label": "Water rating", "type": "choice",
			"options": [["", "(unset)"], ["sheltered", "Sheltered"], ["coastal", "Coastal"], ["open", "Open"]]},
		{"key": "sailing_window", "label": "Sailing window", "type": "choice",
			"options": [["", "(unset)"], ["daylight", "Daylight"], ["continuous", "Continuous"]]},
		{"key": "portage_capable", "label": "Portage-capable", "type": "toggle"},
	]},
]

const PRESET_GROUPS: Array[Dictionary] = [
	{"title": "Transport & pace", "fields": [
		{"key": "name", "label": "Name", "type": "text"},
		{"key": "transport", "label": "Transport", "type": "text"},
		{"key": "mount_animal", "label": "Mount/draft animal", "type": "text"},
		{"key": "vessel", "label": "Vessel", "type": "text"},
		{"key": "hours", "label": "Hours", "type": "number", "unit": "h/day", "min": 0, "max": 24, "step": 0.5},
		{"key": "pace", "label": "Pace", "type": "text"},
		{"key": "season", "label": "Season", "type": "text"},
	]},
	{"title": "Supply", "fields": [
		{"key": "supply_days", "label": "Supplies carried", "type": "number", "unit": "days", "min": 0, "max": 90, "step": 1},
		{"key": "carry_food", "label": "Carry food", "type": "toggle"},
		{"key": "grazing", "label": "Grazing", "type": "text"},
		{"key": "foraging", "label": "Foraging", "type": "text"},
	]},
	{"title": "Party composition", "fields": [
		{"key": "group_size", "label": "Group size", "type": "number", "unit": "people", "min": 0, "max": 500, "step": 1},
		{"key": "cargo_kg", "label": "Cargo", "type": "number", "unit": "kg", "min": 0, "max": 20000, "step": 10},
		{"key": "donkey", "label": "Donkeys", "type": "number", "unit": "", "min": 0, "max": 100, "step": 1},
		{"key": "mule", "label": "Mules", "type": "number", "unit": "", "min": 0, "max": 100, "step": 1},
		{"key": "camel", "label": "Camels", "type": "number", "unit": "", "min": 0, "max": 100, "step": 1},
		{"key": "horse", "label": "Horses", "type": "number", "unit": "", "min": 0, "max": 100, "step": 1},
		{"key": "carts", "label": "Carts", "type": "number", "unit": "", "min": 0, "max": 50, "step": 1},
		{"key": "wagons", "label": "Wagons", "type": "number", "unit": "", "min": 0, "max": 50, "step": 1},
		{"key": "sleds", "label": "Sleds", "type": "number", "unit": "", "min": 0, "max": 50, "step": 1},
		{"key": "travois", "label": "Travois", "type": "number", "unit": "", "min": 0, "max": 50, "step": 1},
	]},
]

const GROUPS_BY_KIND := {
	"animal": ANIMAL_GROUPS, "vehicle": VEHICLE_GROUPS, "vessel": VESSEL_GROUPS, "preset": PRESET_GROUPS,
}

## `travel_library.rs::TL_TERRAIN_KEYS` order, paired with the wire slug
## `travel_bridge.rs::terrain_slug` uses (`"terrain.<slug>"` dictionary keys).
const TERRAIN_ROWS: Array[Array] = [
	["plains", "Plains"], ["steppe", "Steppe"], ["forest", "Forest"], ["hills", "Hills"],
	["mountain", "Mountain"], ["marsh", "Marsh"], ["desert", "Desert"],
	["high_pass", "High Pass"], ["snowfield", "Snowfield"], ["river_ford", "River Ford"],
]

var _host: DccApp
var _bridge: EngineBridge

var _current_kind := "animal"
var _current_id := ""
var _filter_text := ""
var _draft: Dictionary = {}       ## staged, uncommitted field edits
var _entry: Dictionary = {}       ## last tl_get(_current_kind, _current_id) result

var _tab_buttons: Dictionary = {} ## kind -> Button
var _rail_body: VBoxContainer
var _rail_header: Label
var _inspector_body: VBoxContainer
var _status_label: Label
var _rail_wrap: Control        ## PH-07: held so the phone pane switch can hide it
var _inspector_wrap: Control
var _phone_title: Label
var _head_row: HBoxContainer    ## PH-07: the bar Reset moves into on a phone

## Phone (§13) -- PH-07. Measured before the fix (parallel device sweep,
## 2026-08-25, 1440x3168): **17 of 29 tappable controls under the 44 dp floor**
## -- every animal/vehicle rail row at 26 physical px, the category tabs at 29
## -- inside a 1180x780 desktop card drawn at native resolution.
##
## Two things beyond the shared three calls. The rail (286 px) beside the
## inspector does not fit 393 dp, so they become two panes behind the tab strip
## the window already has: picking an entry moves to it, and a Back chip in the
## inspector's own header returns. And the tab strip is six category buttons
## plus a Reset action in one row -- ~700 dp of minimum -- so on a phone it
## wraps.
var _phone := false
var _phone_showing_entry := false
var _phone_back_btn: Button

func setup(host: DccApp, bridge: EngineBridge) -> void:
	_host = host
	_bridge = bridge
	title = "⧉ TRAVEL LIBRARY"
	get_ok_button().hide()
	size = Vector2i(1180, 780)
	min_size = Vector2i(940, 620)
	_phone = DccWidgets.phone_window(self, host)
	_build()
	if _phone:
		_host.phone_fit(self, 1.0)

## `kind`, if given, selects that tab; empty keeps whatever tab was last
## active (Animals & mounts the first time).
func open(kind: String = "") -> void:
	if not DccWidgets.phone_present(self, _host):
		popup_centered()
	if kind != "" and GROUPS_BY_KIND.has(kind):
		_select_kind(kind)
	else:
		_refresh_rail()
		_refresh_inspector()
	_show_phone_entry(false)

## PH-07's two-pane switch. The rail is the list, the inspector is the entry;
## on a phone exactly one is visible, and the tab strip above them stays put,
## so switching category from inside an entry lands back on the list -- which
## is what `_select_kind()` already does to the selection anyway.
func _show_phone_entry(on: bool) -> void:
	if not _phone:
		return
	_phone_showing_entry = on
	_rail_wrap.visible = not on
	_inspector_wrap.visible = on
	if _phone_back_btn != null:
		_phone_back_btn.visible = on

## PH-07: the rail rows and the whole inspector are rebuilt on every refresh, so
## the one-shot fit in `setup()` never sees them. Idempotent by meta-flag.
##
## **Deferred**, and that is the whole point of the indirection: the callers are
## rebuild functions with early returns in the middle of them, so a direct call
## at the top would fit the nodes that are about to be freed and a call at the
## bottom would be skipped on exactly the paths that return early. One deferred
## pass runs after the rebuild has finished, whichever way it finished.
func _phone_refit() -> void:
	if _phone and _host != null:
		_do_phone_refit.call_deferred()

func _do_phone_refit() -> void:
	if _phone and _host != null and is_instance_valid(self):
		_host.phone_fit(self, 1.0)

# ---------------------------------------------------------------------------
# Layout
# ---------------------------------------------------------------------------

func _build() -> void:
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)

	var head_row := HBoxContainer.new()
	_head_row = head_row
	head_row.add_theme_constant_override("separation", 12)
	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 12)
	head_pad.add_theme_constant_override("margin_top", 6)
	head_pad.add_theme_constant_override("margin_right", 12)
	head_pad.add_theme_constant_override("margin_bottom", 6)
	head_pad.add_child(head_row)
	outer.add_child(head_pad)
	## PH-07: the title, the menu path and the "definitions only" caption are
	## three captions in one row that already has to hold a Close button. On a
	## phone `phone_head()` carries the first and third below, so the bar keeps
	## the two controls -- Back (into the entry list) and Close.
	if _phone:
		_phone_back_btn = Button.new()
		_phone_back_btn.text = "‹ Entries"
		_phone_back_btn.focus_mode = Control.FOCUS_NONE
		_phone_back_btn.visible = false
		_phone_back_btn.pressed.connect(func(): _show_phone_entry(false))
		head_row.add_child(_phone_back_btn)
		head_row.add_child(DccTheme.spacer())
	else:
		head_row.add_child(DccTheme.mono_label("⧉ TRAVEL LIBRARY", "accent", DccTheme.FS_HEADER, 2, true))
		head_row.add_child(DccTheme.label("Data ▸ Travel library", "text_ghost", DccTheme.FS_TINY))
		head_row.add_child(DccTheme.spacer())
		head_row.add_child(DccTheme.label("definitions only · read by the planner at plan time", "text_ghost", DccTheme.FS_TINY))
	var close_btn := Button.new()
	close_btn.text = "Close"
	close_btn.focus_mode = Control.FOCUS_NONE
	close_btn.pressed.connect(func(): hide())
	head_row.add_child(close_btn)
	outer.add_child(DccTheme.rule())

	outer.add_child(_build_tab_strip())
	outer.add_child(DccTheme.rule())

	_status_label = DccTheme.label("", "text_ghost", DccTheme.FS_MICRO)
	var status_pad := MarginContainer.new()
	status_pad.add_theme_constant_override("margin_left", 12)
	status_pad.add_theme_constant_override("margin_top", 2)
	status_pad.add_theme_constant_override("margin_bottom", 2)
	status_pad.add_child(_status_label)
	outer.add_child(status_pad)

	var main: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	main.add_theme_constant_override("separation", 0)
	main.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(main)
	_rail_wrap = _build_rail()
	main.add_child(_rail_wrap)
	if not _phone:
		main.add_child(DccTheme.rule(true))
	_inspector_wrap = _build_inspector()
	main.add_child(_inspector_wrap)

	if _phone:
		_phone_title = DccWidgets.phone_head(outer, "Travel library",
			"definitions read by the planner at plan time")

	_refresh_rail()
	_refresh_inspector()

func _build_tab_strip() -> Control:
	## PH-07: six category tabs plus `Reset tab to stock…` is ~700 dp of
	## minimum width, and a `BoxContainer` handed more minimum than it has
	## overlaps rather than clipping. An `HFlowContainer` wraps instead, and the
	## number of rows follows the labels rather than a count fixed here.
	var row: Container
	if _phone:
		var flow := HFlowContainer.new()
		flow.add_theme_constant_override("h_separation", 4)
		flow.add_theme_constant_override("v_separation", 4)
		row = flow
	else:
		var box := HBoxContainer.new()
		box.add_theme_constant_override("separation", 0)
		row = box
	for k in KINDS:
		var kind_info: Dictionary = k
		var btn := Button.new()
		btn.flat = true
		btn.focus_mode = Control.FOCUS_NONE
		btn.custom_minimum_size = Vector2(0, 27)
		btn.add_theme_font_override("font", DccTheme.mono(1))
		btn.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
		btn.pressed.connect(_select_kind.bind(String(kind_info["key"])))
		_tab_buttons[String(kind_info["key"])] = btn
		row.add_child(btn)
	if not _phone:
		row.add_child(DccTheme.spacer())   ## no far end in a wrapping row
	var reset_btn := Button.new()
	reset_btn.text = "Reset tab to stock…"
	reset_btn.focus_mode = Control.FOCUS_NONE
	reset_btn.tooltip_text = "Discards every custom entry of the current tab, restoring the stock-only bootstrap. Cannot be undone."
	reset_btn.pressed.connect(func():
		_bridge.tl_reset_to_stock(_current_kind)
		_current_id = ""
		_draft.clear()
		_refresh_rail()
		_refresh_inspector())
	## PH-07: `Reset tab to stock…` is a destructive action, and in a WRAPPING
	## tab strip it lands mid-flow between two category chips -- it reads as a
	## sixth tab and sits a thumb-width from the fifth. On a phone it goes up to
	## the head row instead, beside Close, where the window's other two
	## non-category controls already are.
	if _phone:
		_head_row.add_child(reset_btn)
	else:
		row.add_child(reset_btn)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_right", 10)
	pad.add_child(row)
	return pad

func _select_kind(kind: String) -> void:
	_current_kind = kind
	_current_id = ""
	_draft.clear()
	_refresh_tab_labels()
	_refresh_rail()
	_refresh_inspector()

func _refresh_tab_labels() -> void:
	var counts: Dictionary = _bridge.tl_counts()
	for k in KINDS:
		var kind_info: Dictionary = k
		var key := String(kind_info["key"])
		var btn: Button = _tab_buttons[key]
		var c: Dictionary = counts.get(key, {})
		btn.text = " %s · %d " % [String(kind_info["label"]).to_upper(), int(c.get("total", 0))]
		var active := key == _current_kind
		btn.add_theme_color_override("font_color", DccTheme.c("accent") if active else DccTheme.c("text_dim"))
		btn.add_theme_stylebox_override("normal", DccTheme.active_row(true) if active else DccTheme.empty())
		btn.add_theme_stylebox_override("hover", DccTheme.active_row(true) if active else DccTheme.flat(DccTheme.c("line_soft")))

func _build_rail() -> Control:
	var wrap := PanelContainer.new()
	## PH-07: 286 px is 73% of a phone's 393 dp; stacked, it is the whole pane.
	if _phone:
		wrap.size_flags_vertical = Control.SIZE_EXPAND_FILL
	else:
		wrap.custom_minimum_size.x = 286
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("panel_alt", {"right": 1}))
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	wrap.add_child(col)

	_rail_header = DccTheme.mono_label("ENTRIES", "text_dim", DccTheme.FS_HEADER, 2, true)
	var head_row := HBoxContainer.new()
	head_row.add_theme_constant_override("separation", 4)
	var head_pad := MarginContainer.new()
	for m in ["margin_left", "margin_top", "margin_right", "margin_bottom"]:
		head_pad.add_theme_constant_override(m, 6)
	head_pad.add_child(head_row)
	head_row.add_child(_rail_header)
	head_row.add_child(DccTheme.spacer())
	var add_btn := Button.new()
	add_btn.text = "＋"
	add_btn.tooltip_text = "New blank definition…"
	add_btn.flat = true
	add_btn.focus_mode = Control.FOCUS_NONE
	add_btn.pressed.connect(_on_add_blank)
	head_row.add_child(add_btn)
	var dup_btn := Button.new()
	dup_btn.text = "⧉"
	dup_btn.tooltip_text = "Duplicate the selected entry (the only way to edit a stock one)"
	dup_btn.flat = true
	dup_btn.focus_mode = Control.FOCUS_NONE
	dup_btn.pressed.connect(_on_duplicate)
	head_row.add_child(dup_btn)
	var del_btn := Button.new()
	del_btn.text = "✕"
	del_btn.tooltip_text = "Delete the selected custom entry"
	del_btn.flat = true
	del_btn.focus_mode = Control.FOCUS_NONE
	del_btn.pressed.connect(_on_delete)
	head_row.add_child(del_btn)
	col.add_child(head_pad)
	col.add_child(DccTheme.rule())

	var filter := LineEdit.new()
	filter.placeholder_text = "filter…"
	var filter_pad := MarginContainer.new()
	for m in ["margin_left", "margin_top", "margin_right", "margin_bottom"]:
		filter_pad.add_theme_constant_override(m, 6)
	filter_pad.add_child(filter)
	filter.text_changed.connect(func(t: String): _filter_text = t; _refresh_rail())
	col.add_child(filter_pad)
	col.add_child(DccTheme.rule())

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(scroll)
	_rail_body = VBoxContainer.new()
	_rail_body.add_theme_constant_override("separation", 0)
	_rail_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_rail_body)

	col.add_child(DccTheme.rule())
	var foot := DccTheme.label("stock entries are read-only — duplicate to edit", "text_ghost", DccTheme.FS_MICRO)
	foot.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	var foot_pad := MarginContainer.new()
	for m in ["margin_left", "margin_top", "margin_right", "margin_bottom"]:
		foot_pad.add_theme_constant_override(m, 8)
	foot_pad.add_child(foot)
	col.add_child(foot_pad)

	return wrap

func _build_inspector() -> Control:
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var pad := MarginContainer.new()
	for m in ["margin_left", "margin_top", "margin_right", "margin_bottom"]:
		pad.add_theme_constant_override(m, 14)
	scroll.add_child(pad)
	_inspector_body = VBoxContainer.new()
	_inspector_body.add_theme_constant_override("separation", 4)
	_inspector_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_inspector_body)
	return scroll

# ---------------------------------------------------------------------------
# Rail
# ---------------------------------------------------------------------------

func _refresh_rail() -> void:
	_phone_refit()   ## PH-07: every rail row below is a fresh node.
	_refresh_tab_labels()
	for c in _rail_body.get_children():
		_rail_body.remove_child(c)
		c.queue_free()

	var entries: Array = _bridge.tl_list(_current_kind)
	var q := _filter_text.to_lower()
	var custom: Array = []
	var stock: Array = []
	for e in entries:
		var row: Dictionary = e
		if q != "" and String(row.get("name", "")).to_lower().find(q) < 0:
			continue
		if String(row.get("origin", "")) == "custom":
			custom.append(row)
		else:
			stock.append(row)

	_rail_body.add_child(DccTheme.mono_label("CUSTOM · %d" % custom.size(), "text_faint", DccTheme.FS_MICRO, 2))
	for row in custom:
		_rail_body.add_child(_build_rail_row(row))
	_rail_body.add_child(DccTheme.rule())
	_rail_body.add_child(DccTheme.mono_label("STOCK · %d" % stock.size(), "text_faint", DccTheme.FS_MICRO, 2))
	for row in stock:
		_rail_body.add_child(_build_rail_row(row))

func _build_rail_row(row: Dictionary) -> Control:
	var id := String(row.get("id", ""))
	var selected := id == _current_id
	var btn := Button.new()
	btn.flat = true
	btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
	btn.focus_mode = Control.FOCUS_NONE
	btn.custom_minimum_size.y = 24
	var state := String(row.get("validation_state", "ok"))
	var mark := ""
	if state == "incomplete":
		mark = "  ⚠"
	elif state == "conflicting":
		mark = "  ⚠⚠"
	var usage_note := ""
	if int(row.get("usage_presets", 0)) > 0:
		usage_note = " · in use"
	btn.text = "%s%s   %s%s" % [String(row.get("name", "")), mark, String(row.get("subtitle", "")), usage_note]
	var color := "text"
	if selected:
		color = "accent"
	elif state == "incomplete":
		color = "warn"
	elif state == "conflicting":
		color = "block"
	elif String(row.get("origin", "")) == "stock":
		color = "text_dim"
	btn.add_theme_color_override("font_color", DccTheme.c(color))
	btn.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	if selected:
		btn.add_theme_stylebox_override("normal", DccTheme.flat(DccTheme.c("accent_wash")))
	btn.pressed.connect(_select_entry.bind(id))
	return btn

# ---------------------------------------------------------------------------
# Selection / CRUD
# ---------------------------------------------------------------------------

func _select_entry(id: String) -> void:
	if _draft.size() > 0 and id != _current_id:
		# Unsaved edits are discarded on switching entries -- the mockup's
		# own "revert" affordance is exactly this, just triggered implicitly
		# rather than requiring a confirm dialog for a project-state tool.
		_draft.clear()
	_current_id = id
	_refresh_rail()
	_refresh_inspector()
	## PH-07: on the desktop composition the inspector is the column beside the
	## list and a tap fills it in place; stacked, it is the pane behind, so the
	## same tap has to move there or it looks like nothing happened.
	_show_phone_entry(true)

func _on_add_blank() -> void:
	var kind_label := ""
	for k in KINDS:
		if String(k["key"]) == _current_kind:
			kind_label = String(k["label"])
	var result: Dictionary = _bridge.tl_add_blank(_current_kind, "New %s" % kind_label.trim_suffix("s"))
	if bool(result.get("ok", false)):
		_select_entry(String(result.get("id", "")))

func _on_duplicate() -> void:
	if _current_id == "":
		return
	var result: Dictionary = _bridge.tl_duplicate(_current_kind, _current_id)
	if bool(result.get("ok", false)):
		_select_entry(String(result.get("id", "")))
	else:
		_status_label.text = String(result.get("error", "duplicate failed"))

func _on_delete() -> void:
	if _current_id == "" or String(_entry.get("origin", "")) != "custom":
		return
	_bridge.tl_delete(_current_kind, _current_id)
	_current_id = ""
	_draft.clear()
	_refresh_rail()
	_refresh_inspector()

func _on_save() -> void:
	if _current_id == "" or _draft.is_empty():
		return
	var result: Dictionary = _bridge.tl_edit(_current_kind, _current_id, _draft)
	if bool(result.get("ok", false)):
		_draft.clear()
	else:
		_status_label.text = String(result.get("error", "save failed"))
	_refresh_rail()
	_refresh_inspector()

func _on_revert() -> void:
	_draft.clear()
	_refresh_inspector()

# ---------------------------------------------------------------------------
# Inspector
# ---------------------------------------------------------------------------

## The value the inspector shows for `key`: the staged draft if present,
## otherwise the last-loaded entry's own value, otherwise `default`.
func _val(key: String, default):
	if _draft.has(key):
		return _draft[key]
	if _entry.has(key):
		return _entry[key]
	return default

func _refresh_inspector() -> void:
	_phone_refit()   ## PH-07: this rebuilds the whole pane from fresh nodes.
	for c in _inspector_body.get_children():
		_inspector_body.remove_child(c)
		c.queue_free()

	if _current_id == "":
		DccWidgets.note(_inspector_body, "Select an entry, or ＋ New blank definition…")
		return

	_entry = _bridge.tl_get(_current_kind, _current_id)
	if not bool(_entry.get("ok", false)):
		DccWidgets.note(_inspector_body, "This entry no longer exists.")
		_current_id = ""
		return

	var editable := bool(_entry.get("editable", false))
	var origin := String(_entry.get("origin", "stock"))

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 10)
	_inspector_body.add_child(head)
	head.add_child(DccTheme.mono_label(String(_entry.get("name", "")).to_upper(), "accent", DccTheme.FS_HEADER, 2, true))
	head.add_child(DccTheme.label(origin if editable else "stock · read-only", "text_dim", DccTheme.FS_TINY))
	head.add_child(DccTheme.spacer())
	var usage_p := int(_entry.get("usage_presets", 0))
	var usage_j := int(_entry.get("usage_journeys", 0))
	if usage_p > 0 or usage_j > 0:
		head.add_child(DccTheme.label("in use by %d party set-up(s) · %d journey(s)" % [usage_p, usage_j], "text_ghost", DccTheme.FS_TINY))
	_inspector_body.add_child(DccTheme.rule())

	if not editable:
		DccWidgets.note(_inspector_body, "Stock entries are read-only. Duplicate this entry (⧉ above) to create an editable custom copy.")

	if _current_kind == "animal" and String(_entry.get("species_key", "")) == "":
		DccWidgets.note(_inspector_body,
			"No live effect yet: this is not one of the four built-in party-form species (donkey/mule/camel/horse). It is real, validated, inspectable data, but the party form's JpParty shape has no slot for a new species to occupy, so it does not change a computed journey (TRAVEL_LIBRARY_SPEC.md §6). Duplicating one of the four built-in animals below DOES affect computed journeys.")
	elif _current_kind == "vehicle" or _current_kind == "vessel":
		DccWidgets.note(_inspector_body,
			"No live effect yet: vehicles and vessels are real, validated, inspectable data, but no resolver hook exists yet for jp_capacity's vehicle constants or jp_ship_stats' vessel table (TRAVEL_LIBRARY_SPEC.md §6).")

	for group in GROUPS_BY_KIND.get(_current_kind, []):
		var g: Dictionary = group
		_inspector_body.add_child(DccTheme.header(String(g["title"]), ""))
		for f in g["fields"]:
			_build_field(_inspector_body, f, editable)

	if _current_kind == "animal":
		_inspector_body.add_child(DccTheme.header("Terrain constraints · multiplier, or blocked", ""))
		for row in TERRAIN_ROWS:
			_build_terrain_row(_inspector_body, String(row[0]), String(row[1]), editable)

	_build_validation_banners(_inspector_body)

	_inspector_body.add_child(DccTheme.rule())
	var footer := HBoxContainer.new()
	footer.add_theme_constant_override("separation", 8)
	_inspector_body.add_child(footer)
	var save_btn := DccWidgets.action(footer, "save definition", _on_save, true)
	save_btn.disabled = not editable or _draft.is_empty()
	var dup_btn := DccWidgets.action(footer, "duplicate", _on_duplicate)
	dup_btn.disabled = _current_id == ""
	var revert_btn := DccWidgets.action(footer, "revert", _on_revert)
	revert_btn.disabled = _draft.is_empty()
	footer.add_child(DccTheme.spacer())
	footer.add_child(DccTheme.label("changes apply at next plan", "text_ghost", DccTheme.FS_MICRO))

func _build_validation_banners(parent: Control) -> void:
	var state := String(_entry.get("validation_state", "ok"))
	if state == "ok" and _draft.is_empty():
		return
	parent.add_child(DccTheme.rule())
	parent.add_child(DccTheme.header("Validation", ""))
	if not _draft.is_empty():
		_banner(parent, "water",
			"%d unsaved change(s) staged. Validation below reflects the last saved state -- press save definition to re-check." % _draft.size())
	if state == "incomplete":
		var missing: PackedStringArray = _entry.get("validation_missing", PackedStringArray())
		_banner(parent, "warn", "Incomplete -- unset: %s. The planner falls back to this entry's declared substitute and flags the stage." % ", ".join(missing))
	elif state == "conflicting":
		var conflicts: PackedStringArray = _entry.get("validation_conflicts", PackedStringArray())
		for c in conflicts:
			_banner(parent, "block", String(c))
	elif state == "ok":
		_banner(parent, "water", "Selectable in the party form. Changing capacity, fodder or a constraint re-plans %d saved set-up(s)/journey(s)." % (int(_entry.get("usage_presets", 0)) + int(_entry.get("usage_journeys", 0))))

## One banner: a coloured left rule plus washed background, matching `2b`'s
## own amber/blue inline styling -- `DccTheme`'s `warn` (`#e0a840`) and
## `water` (`#7d9dae`) tokens are the exact same hex the mockup's own
## incomplete/info banners use, and `block` (`#b55950`) for a conflict.
func _banner(parent: Control, token: String, text: String) -> void:
	var box := PanelContainer.new()
	var sb := StyleBoxFlat.new()
	sb.bg_color = Color(DccTheme.c(token), 0.09)
	sb.border_width_left = 2
	sb.border_color = DccTheme.c(token)
	sb.content_margin_left = 8
	sb.content_margin_right = 8
	sb.content_margin_top = 4
	sb.content_margin_bottom = 4
	box.add_theme_stylebox_override("panel", sb)
	var l := DccTheme.label(text, token, DccTheme.FS_SMALL)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(l)
	parent.add_child(box)

func _build_field(parent: Control, f: Dictionary, editable: bool) -> void:
	var key := String(f["key"])
	var label := String(f["label"])
	var kind := String(f["type"])
	match kind:
		"text":
			var row := HBoxContainer.new()
			row.add_theme_constant_override("separation", 8)
			row.custom_minimum_size.y = 24
			var l := DccTheme.mono_label(label, "text_dim", DccTheme.FS_SMALL)
			l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
			row.add_child(l)
			var le := LineEdit.new()
			le.text = String(_val(key, ""))
			le.editable = editable
			le.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			le.text_submitted.connect(func(t: String): _draft[key] = t; _refresh_footer_state())
			le.focus_exited.connect(func(): _draft[key] = le.text; _refresh_footer_state())
			row.add_child(le)
			parent.add_child(row)
		"number":
			var mn: float = f.get("min", 0.0)
			var mx: float = f.get("max", 1000.0)
			var st: float = f.get("step", 1.0)
			var unit := String(f.get("unit", ""))
			var full_label := "%s (%s)" % [label, unit] if unit != "" else label
			var sb := DccWidgets.number(parent, full_label, mn, mx, st, float(_val(key, mn)),
				func(v: float): _draft[key] = v; _refresh_footer_state())
			sb.editable = editable
		"toggle":
			var cb := DccWidgets.toggle(parent, label, bool(_val(key, false)),
				func(v: bool): _draft[key] = v; _refresh_footer_state())
			cb.disabled = not editable
		"choice":
			var options: Array = f["options"]
			var current := String(_val(key, ""))
			var sel := 0
			for i in options.size():
				if String(options[i][0]) == current:
					sel = i
			var labels: Array = []
			for o in options:
				labels.append(String(o[1]))
			var ob := DccWidgets.choice(parent, label, labels, sel,
				func(i: int): _draft[key] = String(options[i][0]); _refresh_footer_state())
			ob.disabled = not editable
		"roles":
			var row := HBoxContainer.new()
			row.add_theme_constant_override("separation", 8)
			row.custom_minimum_size.y = 24
			var l := DccTheme.mono_label(label, "text_dim", DccTheme.FS_SMALL)
			l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
			row.add_child(l)
			var current: Array = String(_val(key, "")).split(",", false)
			for role in ["pack", "mount", "draft"]:
				var cb := CheckBox.new()
				cb.text = role
				cb.button_pressed = current.has(role)
				cb.disabled = not editable
				cb.focus_mode = Control.FOCUS_NONE
				cb.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
				cb.toggled.connect(func(_v: bool):
					var picked: Array = []
					for c in row.get_children():
						if c is CheckBox and (c as CheckBox).button_pressed:
							picked.append((c as CheckBox).text)
					_draft[key] = ",".join(picked)
					_refresh_footer_state())
				row.add_child(cb)
			parent.add_child(row)
		"modes":
			var row := HBoxContainer.new()
			row.add_theme_constant_override("separation", 8)
			row.custom_minimum_size.y = 24
			var l := DccTheme.mono_label(label, "text_dim", DccTheme.FS_SMALL)
			l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
			row.add_child(l)
			var current: Array = String(_val(key, "")).split(",", false)
			for m in ["river", "sea"]:
				var cb := CheckBox.new()
				cb.text = m
				cb.button_pressed = current.has(m)
				cb.disabled = not editable
				cb.focus_mode = Control.FOCUS_NONE
				cb.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
				cb.toggled.connect(func(_v: bool):
					var picked: Array = []
					for c in row.get_children():
						if c is CheckBox and (c as CheckBox).button_pressed:
							picked.append((c as CheckBox).text)
					_draft[key] = ",".join(picked)
					_refresh_footer_state())
				row.add_child(cb)
			parent.add_child(row)
		"affinity":
			var row := HBoxContainer.new()
			row.add_theme_constant_override("separation", 8)
			row.custom_minimum_size.y = 24
			var l := DccTheme.mono_label(label, "text_dim", DccTheme.FS_SMALL)
			l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
			row.add_child(l)
			var raw = _val(key, 1.0)
			var blocked := typeof(raw) == TYPE_STRING and String(raw) == "blocked"
			var sb := SpinBox.new()
			sb.min_value = 0.0
			sb.max_value = 3.0
			sb.step = 0.05
			sb.value = 0.0 if blocked else float(raw)
			sb.editable = editable and not blocked
			sb.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			row.add_child(sb)
			var blocked_cb := CheckBox.new()
			blocked_cb.text = "blocked"
			blocked_cb.button_pressed = blocked
			blocked_cb.disabled = not editable
			blocked_cb.focus_mode = Control.FOCUS_NONE
			row.add_child(blocked_cb)
			sb.value_changed.connect(func(v: float): _draft[key] = v; _refresh_footer_state())
			blocked_cb.toggled.connect(func(v: bool):
				sb.editable = editable and not v
				_draft[key] = "blocked" if v else sb.value
				_refresh_footer_state())
			parent.add_child(row)

func _build_terrain_row(parent: Control, slug: String, label: String, editable: bool) -> void:
	var key := "terrain.%s" % slug
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = 22
	var l := DccTheme.mono_label(label.to_lower(), "text_dim", DccTheme.FS_TINY)
	l.custom_minimum_size.x = 90
	row.add_child(l)
	var raw = _val(key, 1.0)
	var blocked := typeof(raw) == TYPE_STRING and String(raw) == "blocked"
	var sb := SpinBox.new()
	sb.min_value = 0.0
	sb.max_value = 3.0
	sb.step = 0.05
	sb.value = 0.0 if blocked else float(raw)
	sb.editable = editable and not blocked
	sb.custom_minimum_size.x = 90
	row.add_child(sb)
	var blocked_cb := CheckBox.new()
	blocked_cb.text = "blocked"
	blocked_cb.button_pressed = blocked
	blocked_cb.disabled = not editable
	blocked_cb.focus_mode = Control.FOCUS_NONE
	blocked_cb.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	row.add_child(blocked_cb)
	sb.value_changed.connect(func(v: float): _draft[key] = v; _refresh_footer_state())
	blocked_cb.toggled.connect(func(v: bool):
		sb.editable = editable and not v
		_draft[key] = "blocked" if v else sb.value
		_refresh_footer_state())
	parent.add_child(row)

## Enables/disables the save/revert footer buttons in place, without a full
## `_refresh_inspector()` rebuild that would steal keyboard focus from
## whatever control the user is mid-edit in.
func _refresh_footer_state() -> void:
	for c in _inspector_body.get_children():
		if c is HBoxContainer and c.get_child_count() >= 3:
			var maybe_save := c.get_child(0)
			if maybe_save is Button and String((maybe_save as Button).text) == "save definition":
				(maybe_save as Button).disabled = _draft.is_empty()
				var maybe_revert := c.get_child(2)
				if maybe_revert is Button:
					(maybe_revert as Button).disabled = _draft.is_empty()

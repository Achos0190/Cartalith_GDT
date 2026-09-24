extends AcceptDialog
class_name CultureProfilesWindow

## `GUI_GAP_REGISTER.md` **CV-02** ("Culture ▸ Profiles ... CLOSED 2026-08-25
## (engine), UI open ... Still open: no panel draws either"). The engine side
## was already real when this window was built -- `civ_culture_vocabulary()`,
## `get_cultures()`, `civ_set_faction_field(fid, "culture", key)` and
## `get_factions()`'s own `culture` field all shipped in the CV-02 pass this
## row cites. What did not exist was a dedicated window: the only surface
## that let anyone SEE the seven cultures as rows, or CHANGE which one a
## faction uses, was `civilization_workspace.gd`'s read-only "Culture"
## category (a summary list) and `faction_roster_window.gd`'s single
## per-faction Culture picker buried in its Identity block. This window is
## the CV-02 mockup's actual destination: a culture list, a selected-culture
## detail with a real name-pool sample, and a faction roster with a
## per-faction culture control, in one place.
##
## No new Rust. Every read and write here goes through the four `#[func]`s
## this task was scoped to: `civ_culture_vocabulary`, `get_cultures`,
## `get_factions`, `civ_set_faction_field`. `CIV_CULTURES` (`cartalith-civ`,
## around line 5100) is inert lookup data -- naming flavour only, it does not
## touch settlement layout.
##
## ## The "live name-pool preview" is real settlement names, not a fabricated
## roll
##
## The mockup's middle column shows tokens like "Marcora, Novium, Auropolis"
## captioned "re-rolls on every open". There is no bound function that draws
## a free sample from a culture's syllable/suffix pool -- `civ_settle_name`
## (`cartalith_civ::civ_settle_name`) is not `#[func]`-exposed at all, and
## fabricating one from a hand-copied syllable table here would be exactly
## the second-source-of-truth `get_cultures()`'s own doc comment refuses.
##
## What this window does instead: shows the REAL names of settlements
## belonging to factions currently assigned that culture
## (`bridge.settlements()` filtered by `bridge.get_factions()`'s `culture`
## field), and offers a reroll on each via the same
## `civ_reroll_settlement_name` the Place editor's own dice button uses. That
## reroll draws from the faction's stored `culture` -- the field this window
## writes (`cartalith_civ::civ_faction_culture` over the roster, since
## 2026-09-24; before that it read the id-derived default and this window had
## to disable every row whose faction had been reassigned). See
## `_rebuild_detail()`.
##
## ## Chrome: derived from §8's real convention, not invented
##
## `DCC_SHELL_SPEC.md` §8 describes the Asset library window's own chrome --
## a custom-drawn "⧉ TITLE" head row and a Close control inside the dialog,
## rather than relying on the OS/embedded window decoration alone. That
## convention is what `asset_library_window.gd`, `data_manager_window.gd` and
## `travel_library_window.gd` all draw, and this window matches it: a mono,
## accent, tracked "⧉ CULTURE & SETTLEMENT STYLE" label plus an explicit Close
## button. Sizing and popup behaviour follow `travel_library_window.gd`
## (`popup_centered()`, a fixed `size`/`min_size`) rather than
## `asset_library_window.gd`/`data_manager_window.gd`'s full-viewport
## `borderless`/"map hidden while open" treatment -- this window's content is
## a modest three-pane inspector, the same scale as Travel library and
## Faction roster, not a whole workspace.
##
## ## Menu placement -- a judgment call, stated per this task's own brief
##
## `design/cartalith-menu-structure.md` line 104's "Politics" row lists
## "recalculate territories, clear territory, generate provinces, show
## provinces, add/remove faction, faction roster" and names no Culture slot.
## In the actual shell, v3 split that old Politics grouping into two left-dock
## categories -- `civilization_workspace.gd`'s "Factions" (where the Faction
## roster button itself lives, `_fill_factions()`) and its "Culture" (a
## read-only summary, `_fill_culture()`). "Beside Faction roster" is therefore
## the Factions category, where this window's opener was added next to
## `roster_btn`; the Culture category's own "which faction has which culture"
## link now also points here, since that is a more specific destination than
## the whole roster window for that one question.
##
## Culture ids are compile-time-stable (`get_cultures()`'s own doc: "the one
## entity id in this port that survives both a regenerate and a save/load"),
## so unlike `FactionRosterWindow._on_world_changed()`, `_selected_id` is
## never reset on a new world -- only re-derived data changes under it.

var app                       ## `DccApp`
var bridge: EngineBridge

var _cultures: Array = []     ## cached `get_cultures()` rows, refreshed per rebuild
var _selected_id := 0         ## `CIV_CULTURES` index, 0..6, stable across worlds

var _list_body: VBoxContainer
var _detail_body: VBoxContainer
var _roster_body: VBoxContainer
var _status_label: Label

## Phone (§13), PH-12's three calls generalised to three stacked panes
## (list / detail / roster) instead of Data manager's two, switched by the
## same kind of segmented row rather than a `TabContainer` -- see
## `data_manager_window.gd::_build_phone_switcher()` for why.
var _phone := false
var _phone_pane := "list"
var _phone_panes: Dictionary = {}       ## key -> Control
var _phone_pane_buttons: Dictionary = {}


func setup(a, b: EngineBridge) -> void:
	app = a
	bridge = b
	title = "⧉ CULTURE & SETTLEMENT STYLE"
	get_ok_button().hide()
	## `data_manager_window.gd`'s own discipline: `AcceptDialog.wrap_controls`
	## defaults true and grows the window to fit its content on every
	## `child_controls_changed()`, never shrinking back -- explicit off rather
	## than relying on a `max_size` side effect.
	wrap_controls = false
	size = Vector2i(1040, 720)
	min_size = Vector2i(780, 560)
	## No `max_size` (2026-09-24, the vault window's `3736fe7` fix repeated). The
	## cap treated `wrap_controls` growing the window to its content, which
	## `phone_window()` below turns off at the cause; all the cap still did was
	## stop a user from making the window bigger.
	_phone = DccWidgets.phone_window(self, a)
	_build()
	if _phone:
		app.phone_fit(self, 1.0)
	## `FactionRosterWindow`'s RF-03 lesson: a world can change while this is
	## up, and every control here writes by faction id, which a new world
	## reuses -- so it must re-derive, not just sit on stale rows.
	bridge.generation_finished.connect(func(ok: bool): if ok and visible: _rebuild())
	bridge.world_loaded.connect(func(): if visible: _rebuild())


func open() -> void:
	if not DccWidgets.phone_present(self, app):
		popup_centered()
	_rebuild()


func _clear(node: Control) -> void:
	for c in node.get_children():
		node.remove_child(c)
		c.queue_free()


func _rebuild() -> void:
	_cultures = bridge.get_cultures()
	if _selected_id < 0 or _selected_id >= _cultures.size():
		_selected_id = 0
	_rebuild_list()
	_rebuild_detail()
	_rebuild_roster()
	_rebuild_status()


# ---------------------------------------------------------------------------
# Layout
# ---------------------------------------------------------------------------

func _build() -> void:
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)

	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 12)
	head_pad.add_theme_constant_override("margin_top", 6)
	head_pad.add_theme_constant_override("margin_right", 12)
	head_pad.add_theme_constant_override("margin_bottom", 6)
	outer.add_child(head_pad)
	var head_row := HBoxContainer.new()
	head_row.add_theme_constant_override("separation", 12)
	head_pad.add_child(head_row)
	if not _phone:
		head_row.add_child(DccTheme.mono_label(
			"⧉ CULTURE & SETTLEMENT STYLE", "accent", DccTheme.FS_HEADER, 2, true))
		var sub := DccTheme.label("Civilization ▸ Factions", "text_ghost", DccTheme.FS_TINY)
		head_row.add_child(sub)
		head_row.add_child(DccTheme.spacer())
		var caption := DccTheme.label(
			"naming pools · per-faction assignment", "text_ghost", DccTheme.FS_TINY)
		head_row.add_child(caption)
	else:
		head_row.add_child(DccTheme.spacer())
	var close_btn := Button.new()
	close_btn.text = "Close"
	close_btn.focus_mode = Control.FOCUS_NONE
	close_btn.pressed.connect(func(): hide())
	head_row.add_child(close_btn)
	outer.add_child(DccTheme.rule())

	if _phone:
		outer.add_child(_build_phone_switcher())

	var main: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	main.add_theme_constant_override("separation", 0)
	main.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(main)

	var list_pane := _build_list_pane()
	main.add_child(list_pane)
	_phone_panes["list"] = list_pane
	if not _phone:
		main.add_child(DccTheme.rule(true))
	var detail_pane := _build_detail_pane()
	main.add_child(detail_pane)
	_phone_panes["detail"] = detail_pane
	if not _phone:
		main.add_child(DccTheme.rule(true))
	var roster_pane := _build_roster_pane()
	main.add_child(roster_pane)
	_phone_panes["roster"] = roster_pane

	outer.add_child(DccTheme.rule())
	_status_label = DccTheme.label("", "text_ghost", DccTheme.FS_MICRO)
	var status_pad := MarginContainer.new()
	status_pad.add_theme_constant_override("margin_left", 12)
	status_pad.add_theme_constant_override("margin_top", 3)
	status_pad.add_theme_constant_override("margin_bottom", 3)
	status_pad.add_child(_status_label)
	outer.add_child(status_pad)

	if _phone:
		DccWidgets.phone_head(outer, "Culture & style", "naming pools · faction assignment")
		_show_phone_pane("list")


func _build_phone_switcher() -> Control:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccTheme.panel("bg", {"bottom": 1}))
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 0)
	DccWidgets.pad(wrap, 8, 4, 8, 4).add_child(row)
	for spec in [["list", "CULTURES"], ["detail", "DETAIL"], ["roster", "FACTIONS"]]:
		var key := String(spec[0])
		var b := Button.new()
		b.text = String(spec[1])
		b.focus_mode = Control.FOCUS_NONE
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.add_theme_font_override("font", DccTheme.mono(0))
		b.add_theme_font_size_override("font_size", DccTheme.FS_MICRO)
		b.pressed.connect(func(): _show_phone_pane(key))
		row.add_child(b)
		_phone_pane_buttons[key] = b
	return wrap


func _show_phone_pane(pane: String) -> void:
	if not _phone:
		return
	_phone_pane = pane
	for key in _phone_panes:
		(_phone_panes[key] as Control).visible = key == pane
	for key in _phone_pane_buttons:
		var b: Button = _phone_pane_buttons[key]
		var on: bool = key == pane
		var fill: StyleBox = DccTheme.flat(DccTheme.c("accent_wash")) if on else DccTheme.empty()
		for sb_name in ["normal", "hover", "pressed"]:
			b.add_theme_stylebox_override(sb_name, fill)
		b.add_theme_stylebox_override("focus", DccTheme.empty())
		b.add_theme_color_override("font_color",
			DccTheme.c("accent") if on else DccTheme.c("text_dim"))


func _phone_refit() -> void:
	if _phone and app != null:
		_do_phone_refit.call_deferred()

func _do_phone_refit() -> void:
	if _phone and app != null and is_instance_valid(self):
		app.phone_fit(self, 1.0)


# -- Column 1: culture list ---------------------------------------------------

func _build_list_pane() -> Control:
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	if not _phone:
		col.custom_minimum_size.x = 300
	col.size_flags_vertical = Control.SIZE_EXPAND_FILL

	## Not `DccWidgets.section()` for the whole pane: its returned body has no
	## VERTICAL expand flag (only horizontal), which is correct for a section
	## that sits inside an already-scrolling parent (the inspector pattern
	## `_rebuild_detail()` uses) and wrong here, where THIS ScrollContainer is
	## the one that has to claim the pane's remaining height. Measured before
	## the fix, windowed: the section's body sized to its own minimum and the
	## scroll folded down to a sliver, showing one row's top few pixels and no
	## more -- the same disabled/unset-axis trap `MISTAKES.md` already has
	## three entries for, a fourth instance. The header below is `section()`'s
	## own head-row construction, copied rather than reused, so the two
	## columns still look identical.
	var head := DccTheme.header("Cultures — naming flavour", "§", true)
	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 14)
	head_pad.add_theme_constant_override("margin_top", 10)
	head_pad.add_theme_constant_override("margin_bottom", 4)
	head_pad.add_child(head)
	col.add_child(head_pad)

	## The ScrollContainer must be `col`'s DIRECT child, not nested in a
	## padding `MarginContainer` -- a container's `SIZE_EXPAND_FILL` only
	## claims bonus space from its own immediate parent's layout pass
	## (`VBoxContainer`, here). Wrapping it in a margin pad for the L/R
	## padding this pane wants moved the flag onto a grandchild of `col` and
	## nothing distributed space to it; `_list_body`'s own left/right margin
	## below carries the padding instead. Faction roster's own list pane
	## avoids the trap by never padding the scroll region at all -- this
	## pane keeps a small inset because the roster column beside it (a plain
	## `card` per row) reads as flush-left otherwise.
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(scroll)
	var list_pad := MarginContainer.new()
	list_pad.add_theme_constant_override("margin_left", 14)
	list_pad.add_theme_constant_override("margin_right", 12)
	list_pad.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.add_child(list_pad)
	_list_body = VBoxContainer.new()
	_list_body.add_theme_constant_override("separation", 2)
	_list_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	list_pad.add_child(_list_body)

	var foot_pad := MarginContainer.new()
	foot_pad.add_theme_constant_override("margin_left", 14)
	foot_pad.add_theme_constant_override("margin_right", 12)
	foot_pad.add_theme_constant_override("margin_top", 6)
	foot_pad.add_theme_constant_override("margin_bottom", 6)
	col.add_child(foot_pad)
	DccWidgets.note(foot_pad, "civ_culture_vocabulary() / get_cultures() — all seven exist before any world is generated; the counts below are zero until one is.")
	return col


func _rebuild_list() -> void:
	_clear(_list_body)
	for c in _cultures:
		var d: Dictionary = c
		var cid := int(d.get("id", 0))
		var name := String(d.get("name", "?"))
		var key := String(d.get("key", ""))
		var affinity := String(d.get("terrain_affinity", ""))
		var terrain_txt := affinity.capitalize() if affinity != "" else "no terrain theme"
		var fc := int(d.get("faction_count", 0))

		var b := Button.new()
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.custom_minimum_size.y = 32
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.text = "%s — %d faction%s" % [name, fc, "" if fc == 1 else "s"]
		if fc > 0:
			b.text += ", %s pop" % FactionRosterWindow._thousands(int(d.get("population", 0)))
		b.tooltip_text = "%s · %s" % [key, terrain_txt]
		if cid == _selected_id:
			## Selected-row treatment lifted from `faction_roster_window.gd`'s
			## own list rows -- this window has no design canvas of its own
			## either, so the derived shape is `DccTheme.outline()`'s
			## documented "selected row" (accent-outlined, `accent_wash`
			## fill), not a full-fill slab (`GUI_GAP_REGISTER.md` §48 / DS-02).
			b.flat = false
			var slab := DccTheme.outline("accent", "accent_wash")
			for sb_name in ["normal", "hover", "pressed"]:
				b.add_theme_stylebox_override(sb_name, slab)
			for color_key in ["font_color", "font_hover_color", "font_pressed_color"]:
				b.add_theme_color_override(color_key, DccTheme.c("text_bright"))
		b.pressed.connect(func():
			_selected_id = cid
			_rebuild_list()
			_rebuild_detail()
			if _phone:
				_show_phone_pane("detail"))
		_list_body.add_child(b)


# -- Column 2: selected culture detail ---------------------------------------

func _build_detail_pane() -> Control:
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	if not _phone:
		col.custom_minimum_size.x = 340
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col.size_flags_vertical = Control.SIZE_EXPAND_FILL

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(scroll)
	_detail_body = VBoxContainer.new()
	_detail_body.add_theme_constant_override("separation", 4)
	_detail_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_detail_body)
	return col


func _culture_row(cid: int) -> Dictionary:
	if cid < 0 or cid >= _cultures.size():
		return {}
	return _cultures[cid]


## Real settlements belonging to factions CURRENTLY assigned culture `key` --
## see this file's header for why "currently assigned" and "what the pool
## would produce right now" are not always the same faction.
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


func _rebuild_detail() -> void:
	_clear(_detail_body)
	var d := _culture_row(_selected_id)
	if d.is_empty():
		DccWidgets.note(_detail_body, "Select a culture.")
		return
	var key := String(d.get("key", ""))
	var name := String(d.get("name", "?"))

	var title_pad := MarginContainer.new()
	title_pad.add_theme_constant_override("margin_top", 8)
	title_pad.add_theme_constant_override("margin_bottom", 2)
	_detail_body.add_child(title_pad)
	var title_row := HBoxContainer.new()
	title_row.add_theme_constant_override("separation", 8)
	title_pad.add_child(title_row)
	title_row.add_child(DccTheme.label(name, "text_bright", DccTheme.FS_MODAL_TITLE))
	title_row.add_child(DccTheme.mono_label(key, "text_faint", DccTheme.FS_TINY, 1))

	var affinity := String(d.get("terrain_affinity", ""))
	DccWidgets.note(_detail_body,
		("Terrain theme: %s. Territory fit (Faction roster) compares a faction's own land against this." % affinity.capitalize())
		if affinity != "" else
		"No terrain theme -- identity-flavoured, like Common. civ_culture_terrain_fit gives no verdict for either, rather than fabricating one.")

	var pool_sec := DccWidgets.section(_detail_body, "Name pool — real settlements")
	var matches := _settlements_for_culture(key)
	if matches.is_empty():
		DccWidgets.note(pool_sec,
			"No settlements currently use this culture. Assign it to a faction (right) to see the names it actually produces.")
	else:
		matches.sort_custom(func(a, b): return int((a.data as Dictionary).get("population", 0)) > int((b.data as Dictionary).get("population", 0)))
		var shown: Array = matches.slice(0, mini(8, matches.size()))
		var flow := HFlowContainer.new()
		flow.add_theme_constant_override("h_separation", 6)
		flow.add_theme_constant_override("v_separation", 6)
		pool_sec.add_child(flow)
		for e in shown:
			var s: Dictionary = e.data
			var idx: int = e.index
			var fid := int(s.get("faction", 0))
			# Every row is a settlement of a faction assigned this culture, and
			# the reroll draws from that faction's assigned culture, so each
			# one rerolls from the pool on screen.
			var chip := DccWidgets.chip(flow, String(s.get("name", "?")),
				func():
					var new_name := bridge.civ_reroll_settlement_name(idx)
					if new_name != "":
						app.set_status("hint", "Rerolled — %s" % new_name, "text_ghost")
					_rebuild_detail(),
				false, 9, 4)
			chip.tooltip_text = "%s — %s, faction %d. Click to reroll a fresh name from this pool." % [
				String(s.get("name", "?")), String(s.get("kind", "?")).capitalize(), fid]
		if matches.size() > shown.size():
			DccWidgets.note(pool_sec, "+%d more, sorted by population." % (matches.size() - shown.size()))
		DccWidgets.note(pool_sec,
			"Names are drawn from the faction's assigned culture -- by Auto-populate, by a Settlement drop left unnamed, and by a reroll. Reassigning a faction's culture here changes the pool those draw from next, and its Territory-fit reading; it does not rename settlements already placed. The same is true of the roster window's own Culture picker. A new world starts every faction on its default culture.")

	var af_sec := DccWidgets.section(_detail_body, "Assigned factions (%d)" % int(d.get("faction_count", 0)))
	var factions := bridge.get_factions()
	if factions.is_empty():
		DccWidgets.note(af_sec, "No factions — generate a world first.")
	elif int(d.get("faction_count", 0)) == 0:
		DccWidgets.note(af_sec, "No faction is currently using this culture.")
	else:
		for f in factions:
			var fd: Dictionary = f
			if String(fd.get("culture", "")) != key:
				continue
			var row := HBoxContainer.new()
			row.add_theme_constant_override("separation", 8)
			var banner := FactionBanner.new()
			banner.configure(int(fd.get("id", 0)), _faction_color(fd), 18)
			row.add_child(banner)
			var lbl := DccTheme.label("%s — %d settlements, %s pop" % [
				String(fd.get("name", "?")), int(fd.get("settlement_count", 0)),
				FactionRosterWindow._thousands(int(fd.get("population", 0)))], "text", DccTheme.FS_SMALL)
			lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			row.add_child(lbl)
			af_sec.add_child(row)

	var gaps := DccWidgets.section(_detail_body, "Not built")
	DccWidgets.note(gaps,
		"Settlement Style — a settlement's built form (streets, parcels, walls) is a separate axis from culture (naming) and is not assignable per faction in either the port or the reference today. cartalith-urban::rules.rs resolves exactly two named profiles, \"medieval\" and \"venus\", chosen per generation call, not stored per faction or settlement. See OUTSTANDING_WORK.md §2.9 (the v2.66 row) before building it.")


func _faction_color(fd: Dictionary) -> Color:
	return Color8(int(fd.get("color_r", 150)), int(fd.get("color_g", 150)), int(fd.get("color_b", 150)))


# -- Column 3: faction roster, assign culture --------------------------------

func _build_roster_pane() -> Control:
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	if not _phone:
		col.custom_minimum_size.x = 340
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col.size_flags_vertical = Control.SIZE_EXPAND_FILL

	## See `_build_list_pane()`'s own comment: not `DccWidgets.section()` for
	## the whole pane, for the same reason -- this ScrollContainer, not the
	## section body, has to claim the pane's remaining vertical space.
	var head := DccTheme.header("Faction roster — assign culture", "§", true)
	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 14)
	head_pad.add_theme_constant_override("margin_top", 10)
	head_pad.add_theme_constant_override("margin_bottom", 4)
	head_pad.add_child(head)
	col.add_child(head_pad)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(scroll)
	var roster_pad := MarginContainer.new()
	roster_pad.add_theme_constant_override("margin_left", 14)
	roster_pad.add_theme_constant_override("margin_right", 12)
	roster_pad.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.add_child(roster_pad)
	_roster_body = VBoxContainer.new()
	_roster_body.add_theme_constant_override("separation", 8)
	_roster_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	roster_pad.add_child(_roster_body)

	var foot_pad := MarginContainer.new()
	foot_pad.add_theme_constant_override("margin_left", 14)
	foot_pad.add_theme_constant_override("margin_right", 12)
	foot_pad.add_theme_constant_override("margin_top", 6)
	foot_pad.add_theme_constant_override("margin_bottom", 6)
	col.add_child(foot_pad)
	DccWidgets.note(foot_pad, "Writes via civ_set_faction_field(fid, \"culture\", key) — the same call the Faction roster's own Culture picker uses.")
	return col


func _rebuild_roster() -> void:
	_clear(_roster_body)
	var factions := bridge.get_factions()
	if factions.is_empty():
		DccWidgets.note(_roster_body, "No factions — generate a world first.")
		return
	var keys := bridge.civ_culture_vocabulary()
	var labels: Array = []
	for k in keys:
		labels.append(String(k).capitalize())
	for f in factions:
		var fd: Dictionary = f
		var fid := int(fd.get("id", 0))
		var card := VBoxContainer.new()
		card.add_theme_constant_override("separation", 2)
		var head := HBoxContainer.new()
		head.add_theme_constant_override("separation", 8)
		var banner := FactionBanner.new()
		banner.configure(fid, _faction_color(fd), 18)
		head.add_child(banner)
		var name_col := VBoxContainer.new()
		name_col.add_theme_constant_override("separation", 0)
		name_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		head.add_child(name_col)
		name_col.add_child(DccTheme.label(String(fd.get("name", "?")), "text_bright", DccTheme.FS_SMALL))
		name_col.add_child(DccTheme.mono_label("%d settlements · %s pop" % [
			int(fd.get("settlement_count", 0)), FactionRosterWindow._thousands(int(fd.get("population", 0)))],
			"text_faint", DccTheme.FS_TINY))
		card.add_child(head)
		var cur := String(fd.get("culture", "common"))
		DccWidgets.choice(card, "Culture", labels, maxi(0, Array(keys).find(cur)),
			func(i: int): _set_faction_culture(fid, String(keys[i])))
		_roster_body.add_child(card)
		_roster_body.add_child(DccTheme.rule())


func _set_faction_culture(fid: int, key: String) -> void:
	if bridge.civ_set_faction_field(fid, "culture", key):
		app.set_status("hint", "Faction %d's naming culture set to %s." % [fid, key.capitalize()], "text_ghost")
	else:
		app.set_status("hint", "Rejected — %s is not a culture the engine recognises." % key, "accent")
	## Full rebuild: the list column's per-culture counts and the detail
	## column's name pool / assigned-factions blocks both move under either
	## the OLD or the NEW culture, not just the roster row that was edited.
	_rebuild()


# -- Status line --------------------------------------------------------------

func _rebuild_status() -> void:
	var total_factions := bridge.get_factions().size()
	var placed := 0
	for c in _cultures:
		placed += int((c as Dictionary).get("settlement_count", 0))
	_status_label.text = "%d cultures · %d factions · %d settlements carry one through their faction — closes GUI_GAP_REGISTER.md CV-02" % [
		_cultures.size(), total_factions, placed]

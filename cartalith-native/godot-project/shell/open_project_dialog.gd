extends AcceptDialog
class_name OpenProjectDialog

## File ▸ Open project…, drawn from the "Open project dialog 1920" screen in
## `design/Cartalith DCC Shell.dc.html`.
##
## The screen is emphatically **not** a file browser. Its own inline comment
## says so -- *"gallery grid — thumbnails, not a tree list"* -- and every part
## of it is world-shaped rather than disk-shaped: a search well that offers to
## match *"by name, seed or region"*, three scope chips (`Recent`, `All
## worlds`, `Shared`), tiles captioned with a seed and a relative edit time, a
## `CURRENT` badge on the world already open, and a foot that names the folder
## projects are read from. A `.zip` on some other volume is reached through
## the one dashed tile that is an action rather than a row: *"Drop a `.zip`
## save or click to browse a folder"*, which hands off to `DccBrowseDialog`.
##
## | mockup element | here |
## |---|---|
## | modal 1180 x 760 | `size` / `min_size` |
## | title + `choose a world to continue, or bring one in from disk` + `✕` | `_build_head()` |
## | `⌕` search well | `_search` |
## | `Recent` / `All worlds` / `Shared` chips, active one accent-outlined | `_build_scopes()` |
## | 4-column tile grid, `16/11.5` tiles | `_grid` |
## | dashed import tile, first | `_build_import_tile()` |
## | `CURRENT` badge, name, `seed · edited 4 min ago` | `_build_tile()` |
## | foot: `projects read from …`, `Cancel`, `Open selected` | `_build_foot()` |
##
## **What is real and what is disclosed.** Following this shell's own habit of
## saying where an affordance has nothing behind it rather than drawing chrome
## that implies one:
##
## - **Recent** is `DccSettings.recent_projects()`, filtered to paths that
##   still exist -- the same list `Data ▸ Recent worlds` reads.
## - **All worlds** lists `*.zip` directly inside `DccSettings.storage_root
##   ("projects")`. Not recursive: the storage root is a flat worlds folder by
##   construction (`dcc_settings.gd`'s `_default_root`), and walking a tree
##   the design never draws would be inventing a capability.
## - **Shared** is a disclosed gap. Nothing in this port has any notion of a
##   shared, multi-user or remote project; the chip is drawn as the mockup
##   draws it, disabled, saying so on hover.
## - **Thumbnails are generated, not stored.** A `.zip` save carries
##   `params.json` plus raw fields (`SAVEFILE_COMPAT.md`) and no preview
##   image, so there is nothing to show. Rather than four identical grey
##   rectangles, each tile takes a radial gradient hued from a hash of its own
##   path -- stable per world, distinct between worlds, and honest about being
##   an identicon rather than a render. When the port grows a thumbnail on
##   save, this is the one function to replace.
## - **Seed and edit time are real.** The time is the file's own mtime; the
##   seed is read out of the save's `params.json` (`state.tect.seed`), which
##   is display metadata, not a computation -- nothing downstream reads it.
##
## ## Welcome mode
##
## `open_welcome()` is the same dialog with the cold-start framing: a
## different head, two extra action tiles ahead of the gallery, and a foot
## that says leaving is allowed. `app.gd`'s `_ready` opens it once when no
## world exists.
##
## **Why this and not a separate welcome screen.** The reference's own setup
## gate (reference HTML lines 657-666) is one card offering three peer
## choices -- generate, load a `.zip`, import a heightmap. This dialog is
## already two of those three: it *is* the load-a-project surface, and it
## already carries one action tile that is not a project (the dashed
## `.zip`-from-disk tile). Adding "create" and "import a heightmap" beside it
## gives the reference's exact three choices on one screen, in the visual
## language this shell already has, and reuses the recents list, the search
## well, the drop handling and the theme scaffolding. A third dialog would
## have had to duplicate all of that to say the same thing, and would have
## put a chooser in front of a gallery that is itself a chooser.
##
## The two extra tiles appear **only** in welcome mode. `File ▸ Open
## project…` is unchanged, because it answers a narrower question ("which of
## my worlds?") and answering it with two tiles about making a new one would
## be noise. The heightmap route stays reachable afterwards through
## `Data ▸ Import ▸ Heightmaps`.

const TILE_MIN := Vector2(232, 186)
const GRID_COLUMNS := 4

var _host: DccApp

var _search: LineEdit
var _grid: GridContainer
var _foot_note: Label
var _open_btn: Button
var _scope := "recent"
var _scope_buttons: Dictionary = {}   ## scope id -> Button
var _selected := ""
var _tiles: Dictionary = {}           ## path -> PanelContainer

## Cold-start framing (see this file's header). Set by `open_welcome()`,
## cleared by `open()`, read by `_build_head`'s labels and by `_refresh`.
var _welcome := false
var _title_label: Label
var _subtitle_label: Label
var _cancel_btn: Button

## Phone (§13). `DccWidgets.phone_window()`'s header comment carries the whole
## treatment and why; here it decides whether the toolbar stacks and whether
## the composition is re-fitted for touch.
var _phone := false
var _toolbar_row: BoxContainer   ## Held so `_apply_phone_toolbar()` can turn it
	## on its side -- see there for why the search well and the scope chips
	## cannot share a row at 393 dp.

## path -> {seed, modified, size}. Keyed by path *and* mtime so a re-saved
## world re-reads rather than showing a stale seed; opening a `.zip` per tile
## is cheap but not free, and the gallery rebuilds on every keystroke in the
## search well.
static var _meta_cache: Dictionary = {}

# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------

func setup(host: DccApp) -> void:
	_host = host
	title = "Open project"
	get_ok_button().hide()   ## the mockup's own foot row replaces it.
	## The mockup's card is a single branded header with a single `✕`
	## (`_build_head()`). An `AcceptDialog` also draws the host `Window`'s own
	## title bar and close button, so the shipped dialog stacked two headers and
	## two close buttons -- reported from the device pass, but wrong on every
	## platform, not just phone. The content header is the one the design draws,
	## so the window chrome is the one that goes.
	borderless = true
	size = Vector2i(1180, 760)
	min_size = Vector2i(880, 560)
	## PH-06's shared treatment, which this dialog wrote the *precedent* for
	## and then never took: `_present()` below was the original fill-the-screen
	## reasoning, and `new_world_dialog.gd` / `browse_dialog.gd` were fitted to
	## the generalised version of it while this file kept the hand-rolled half.
	## What it was missing is the other half -- `phone_fit()`, the touch-target
	## and stacking pass -- plus `wrap_controls = false`. Also turns the
	## rotation relay into the guarded, self-disconnecting one, so the manual
	## `phone_insets_changed` connection this file used to make is gone: the
	## shared relay re-presents the *window*, and the one kept below re-fits
	## only what is specific to this screen.
	_phone = DccWidgets.phone_window(self, host)
	_build()
	## The dashed tile is a drop target, and a drop lands on the *window*, not
	## on the control under the cursor -- Godot reports files at window level.
	## Guarded on visibility so a drop onto the shell while this dialog is
	## closed is not silently swallowed by a hidden dialog.
	files_dropped.connect(_on_files_dropped)
	if _phone:
		## Rotation changes how many tiles fit across the gallery; the window
		## geometry itself is `phone_window()`'s own relay's business.
		_host.phone_insets_changed.connect(func():
			if visible:
				_fit_phone_content())
		## `1.0`, not `phone_scale()`: `phone_present()` applies the scale once
		## as the window's `content_scale_factor`, and applying it again here
		## would square it. The composition is built once, so one pass does.
		_apply_phone_toolbar()
		_host.phone_fit(self, 1.0)

func open() -> void:
	_selected = ""
	_welcome = false
	_present()
	_refresh()

## The cold-start prompt: the same gallery, framed as "start here" and
## carrying the two actions that are not "open one of these". See this
## file's header for why welcome is a mode rather than its own dialog.
##
## Closing it -- Escape, the ✕, or the foot's own opt-out -- leaves the shell
## exactly as it was. Nothing about this is a gate.
func open_welcome() -> void:
	_selected = ""
	_welcome = true
	_present()
	_refresh()

## §13's region table sends docks to "full-screen sheets" on a phone; a modal
## gallery is the same case, and the shipped dialog instead kept its desktop
## 1180x760 inside a 393-px-wide shell -- most of it simply off-screen, with
## 10-12 px type on the part that wasn't (device pass, 2026-08-19).
##
## Rather than re-author every constant in this file at phone sizes, the window
## fills the screen and `content_scale_factor` scales the whole desktop-authored
## composition by the same factor the shell uses for its own chrome. That keeps
## one layout for both form factors -- the mockup's own phone reference is
## 393 px wide, which is exactly what `size / _host.phone_scale()` comes to on
## a real handset, so the desktop numbers land on the phone reference by
## construction instead of by a second set of constants.
##
## Re-run on every open (and on rotation, via the relay `phone_window()`
## installs) because the viewport it measures changes with both. The geometry
## itself is now `DccWidgets.phone_present()`, which is this reasoning
## generalised -- and which also fixed a bug this file's hand-rolled version
## had: `popup_centered()` first and `size = screen` after produced no resize
## notification on a *hidden* window, so the body kept its desktop rect and
## overflowed instead of scrolling. `Window.popup(rect)` sizes as part of
## showing. See `dcc_widgets.gd`'s own header for the measurement.
func _present() -> void:
	if not DccWidgets.phone_present(self, _host):
		popup_centered()
		return
	_fit_phone_content()

## The two things about *this* screen that a generic phone presentation cannot
## know: which head text does not fit, and how many tiles do.
func _fit_phone_content() -> void:
	## A `Window` cannot shrink below its content minimum, so full-screen only
	## takes effect once the widest row can actually fit the column. The head is
	## that row by a wide margin: the subtitle is a single unwrapped `Label`
	## whose text alone is ~420 px, more than the entire 393 px phone reference.
	## It is explanatory prose that the three action tiles underneath already
	## say in full, so phone drops it -- everything else (the search well, the
	## clipped foot note) is already shrinkable.
	_subtitle_label.visible = false
	_fit_columns(_host.get_viewport_rect().size.x / _host.phone_scale())
	## **An `AcceptDialog` sizes its content child on resize, and on nothing
	## else.** Hiding the subtitle a line above is a minimum-size change, not a
	## resize, so the body kept the 497 dp width it was measured at *with* the
	## subtitle in it -- inside a 393 dp window. Measured: the search well ran
	## 82 dp off the right edge, taking the gallery tiles and the "Open
	## selected" button with it. `child_controls_changed()` is the engine's own
	## "re-measure me" for exactly this, and it brings the body back to 380 dp
	## (its real minimum, three over the 377 available, which is nothing).
	## Called last, so it sees the finished composition.
	child_controls_changed()

## The toolbar is a search well that expands beside a three-chip scope row.
## That is one row too many for 393 dp: the chips' own minimum is ~230 dp, and
## a `BoxContainer` handed more minimum width than it has does not clip, it
## **overlaps** -- so the well's outlined panel drew straight over `Recent /
## All worlds / Shared`, and the `LineEdit` inside it got the ~110 dp left
## over, which is where "Search wo…" came from. Both symptoms measured on the
## handset; both are the same fault. Stacking is the only fix that keeps every
## control at full size, and it is exactly what `phone_window()` returns a
## boolean for.
func _apply_phone_toolbar() -> void:
	if _toolbar_row == null:
		return
	var stacked := VBoxContainer.new()
	stacked.add_theme_constant_override("separation", 10)
	var parent := _toolbar_row.get_parent()
	var index := _toolbar_row.get_index()
	parent.remove_child(_toolbar_row)
	## The children move to the new column rather than the row being reparented
	## into it -- an `HBoxContainer` nested in a `VBoxContainer` would lay its
	## own children out horizontally again, which is the arrangement being
	## undone.
	for c in _toolbar_row.get_children():
		_toolbar_row.remove_child(c)
		(c as Control).size_flags_horizontal = Control.SIZE_EXPAND_FILL
		stacked.add_child(c)
	_toolbar_row.queue_free()
	_toolbar_row = null
	parent.add_child(stacked)
	parent.move_child(stacked, index)

## The gallery is a 4-column grid at 1180 px. At the phone reference width it
## fits one tile, and two in landscape -- computed from the tile's own minimum
## rather than hard-coded per orientation.
func _fit_columns(layout_width: float) -> void:
	if _grid == null:
		return
	var usable := layout_width - 60.0   ## `_build()`'s left+right grid padding.
	_grid.columns = maxi(1, int(floor((usable + 18.0) / (TILE_MIN.x + 18.0))))

# ---------------------------------------------------------------------------
# Layout
# ---------------------------------------------------------------------------

func _build() -> void:
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)

	outer.add_child(_build_head())
	outer.add_child(DccTheme.rule())
	outer.add_child(_build_toolbar())

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 30)
	pad.add_theme_constant_override("margin_top", 22)
	pad.add_theme_constant_override("margin_right", 30)
	pad.add_theme_constant_override("margin_bottom", 8)
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(pad)
	_grid = GridContainer.new()
	_grid.columns = GRID_COLUMNS
	_grid.add_theme_constant_override("h_separation", 18)
	_grid.add_theme_constant_override("v_separation", 18)
	_grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_grid)
	outer.add_child(scroll)

	outer.add_child(DccTheme.rule())
	outer.add_child(_build_foot())

func _build_head() -> Control:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 14)
	## Both are re-worded in welcome mode (`_paint_head`) rather than built
	## twice -- the row's layout is identical either way.
	_title_label = DccTheme.label("Open project", "text_bright", DccTheme.FS_MODAL_TITLE)
	row.add_child(_title_label)
	_subtitle_label = DccTheme.label("choose a world to continue, or bring one in from disk",
		"text_ghost", DccTheme.FS_SMALL)
	row.add_child(_subtitle_label)
	row.add_child(DccTheme.spacer())
	var close := Button.new()
	close.text = DccIcons.SYMBOLS["cross"]
	close.flat = true
	close.focus_mode = Control.FOCUS_NONE
	close.add_theme_font_override("font", DccTheme.mono())
	close.add_theme_font_size_override("font_size", DccTheme.FS_MODAL_TITLE)
	close.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
	close.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	close.pressed.connect(func(): hide())
	row.add_child(close)
	return _pad(row, 30, 22, 30, 16)

func _build_toolbar() -> Control:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 12)
	_toolbar_row = row   ## `_apply_phone_toolbar()` stacks it on a handset.

	var well := PanelContainer.new()
	well.add_theme_stylebox_override("panel", DccTheme.outline("line"))
	well.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var well_row := HBoxContainer.new()
	well_row.add_theme_constant_override("separation", 9)
	var well_pad := MarginContainer.new()
	well_pad.add_theme_constant_override("margin_left", 12)
	well_pad.add_theme_constant_override("margin_right", 12)
	well_pad.add_theme_constant_override("margin_top", 8)
	well_pad.add_theme_constant_override("margin_bottom", 8)
	well_pad.add_child(well_row)
	well.add_child(well_pad)
	well_row.add_child(DccIcons.rect("search", 12, "text_ghost"))
	_search = LineEdit.new()
	_search.placeholder_text = "Search worlds by name, seed or region…"
	_search.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_search.add_theme_font_size_override("font_size", DccTheme.FS_BODY)
	_search.add_theme_stylebox_override("normal", DccTheme.empty())
	_search.add_theme_stylebox_override("focus", DccTheme.empty())
	_search.text_changed.connect(func(_t: String): _refresh())
	well_row.add_child(_search)
	row.add_child(well)

	row.add_child(_build_scopes())
	return _pad(row, 30, 16, 30, 0)

func _build_scopes() -> Control:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 2)
	for entry in [
		{"id": "recent", "label": "Recent", "reason": ""},
		{"id": "all", "label": "All worlds", "reason": ""},
		## §-less disclosed gap: no sharing, sync or remote-project concept
		## exists anywhere in the workspace, so the chip is present (the
		## design draws it) and inert (nothing can answer it).
		{"id": "shared", "label": "Shared",
			"reason": "No shared or remote project concept exists in this port — projects are local .zip saves only."},
	]:
		var scope: Dictionary = entry
		var b := Button.new()
		b.text = String(scope["label"])
		b.focus_mode = Control.FOCUS_NONE
		b.add_theme_font_override("font", DccTheme.mono())
		b.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
		if String(scope["reason"]) != "":
			b.disabled = true
			b.tooltip_text = String(scope["reason"])
		else:
			b.pressed.connect(func(): _set_scope(String(scope["id"])))
		_scope_buttons[String(scope["id"])] = b
		row.add_child(b)
		_paint_scope(String(scope["id"]))
	return row

func _paint_scope(id: String) -> void:
	var b: Button = _scope_buttons[id]
	var on := _scope == id
	var box := DccTheme.outline("accent" if on else "line")
	box.content_margin_left = 12
	box.content_margin_right = 12
	box.content_margin_top = 6
	box.content_margin_bottom = 6
	for state in ["normal", "hover", "pressed", "disabled"]:
		b.add_theme_stylebox_override(state, box)
	b.add_theme_color_override("font_color",
		DccTheme.c("accent") if on else DccTheme.c("text_dim"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))

func _set_scope(id: String) -> void:
	if _scope == id:
		return
	_scope = id
	for key in _scope_buttons:
		_paint_scope(String(key))
	_refresh()

func _build_foot() -> Control:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	_foot_note = DccTheme.mono_label("", "text_ghost", DccTheme.FS_TINY)
	_foot_note.clip_text = true
	row.add_child(_foot_note)
	row.add_child(DccTheme.spacer())
	_cancel_btn = DccWidgets.modal_button(row, "Cancel", func(): hide())
	_open_btn = DccWidgets.modal_button(row, "Open selected", _confirm, true)
	_open_btn.disabled = true
	return _pad(row, 30, 14, 30, 14)

## Welcome mode re-words the head and the opt-out; everything else is the
## same screen. Called from `_refresh`, so it tracks whichever `open*` ran.
func _paint_head() -> void:
	if _welcome:
		_title_label.text = "Cartalith"
		_subtitle_label.text = "start a world, continue one, or bring a heightmap in from disk"
		## Not "Cancel": in welcome mode nothing is being cancelled, and the
		## wording has to say that closing is a real, supported outcome --
		## the one place this port deliberately parts company with the
		## reference's mandatory gate (see `app.gd`'s `_ready`).
		_cancel_btn.text = "Continue without a world"
	else:
		_title_label.text = "Open project"
		_subtitle_label.text = "choose a world to continue, or bring one in from disk"
		_cancel_btn.text = "Cancel"

# ---------------------------------------------------------------------------
# Content
# ---------------------------------------------------------------------------

## The paths the active scope offers, newest first. `Recent` keeps the recency
## order `DccSettings` already maintains; `All worlds` has no such order of its
## own, so it sorts by mtime -- the same "most recently touched first" the
## recents list means.
func _paths() -> Array:
	var out: Array = []
	if _scope == "recent":
		for p in DccSettings.recent_projects():
			if FileAccess.file_exists(String(p)):
				out.append(String(p))
	elif _scope == "all":
		var root := DccSettings.storage_root("projects")
		for f in DirAccess.get_files_at(root):
			if String(f).get_extension().to_lower() == "zip":
				out.append(root.path_join(String(f)))
		out.sort_custom(func(a: String, b: String):
			return FileAccess.get_modified_time(a) > FileAccess.get_modified_time(b))
	return out

func _refresh() -> void:
	## `remove_child` before `queue_free`: freeing alone is deferred to the end
	## of the frame, so two refreshes inside one frame (opening the dialog and
	## the first keystroke in the search well) would rebuild the gallery on top
	## of tiles that are still parented.
	for c in _grid.get_children():
		_grid.remove_child(c)
		c.queue_free()
	_tiles.clear()
	_paint_head()

	## Welcome mode leads with the two actions the gallery cannot express --
	## make a world, bring a heightmap in -- so the reference's three choices
	## read left to right across the first row before any project tile.
	if _welcome:
		_grid.add_child(_build_action_tile(
			"domain_world", "Create a new world",
			"seed, size and world shape",
			true, func():
				hide()
				_host.open_new_world()))
		## Hidden outright, not disabled, when the loaded extension has no
		## import binding: an affordance that cannot work is worse than one
		## that is absent, and unlike the `Shared` chip above there is no
		## design element here it would be dishonest to drop.
		if _host != null and _host.bridge.import_api:
			_grid.add_child(_build_action_tile(
				"mountains", "Import a heightmap",
				"a PNG image, white = high —\ntectonics inferred from it",
				false, func():
					hide()
					_host.open_heightmap_import()))

	_grid.add_child(_build_import_tile())

	var query := _search.text.strip_edges().to_lower()
	var shown := 0
	for path in _paths():
		var meta := project_meta(String(path))
		if query != "" and not _matches(String(path), meta, query):
			continue
		_grid.add_child(_build_tile(String(path), meta))
		shown += 1

	var root := DccSettings.storage_root("projects")
	if _scope == "shared":
		_foot_note.text = "Shared projects are not a concept in this port"
	elif shown == 0 and query != "":
		_foot_note.text = "no match · projects read from %s" % root
	elif shown == 0 and _welcome:
		## The first-ever launch. "nothing here yet" alone reads as a fault;
		## the two tiles above are the answer, so the foot names them.
		_foot_note.text = "no saved worlds yet — start one above · projects read from %s" % root
	elif shown == 0:
		_foot_note.text = "nothing here yet · projects read from %s" % root
	else:
		_foot_note.text = "projects read from %s" % root
	_refresh_open_button()
	## The gallery just changed, and on a phone its width is what the window
	## has to be re-measured against -- see `_fit_phone_content()` for why that
	## does not happen on its own. Runs on every keystroke in the search well,
	## which is what `child_controls_changed()` is cheap enough for.
	if _phone:
		_fit_phone_content()

## The search well offers "name, seed or region". Name and seed are real
## fields; "region" has no equivalent -- a save carries no region name -- so
## the path itself stands in for it, which is what a folder-per-region layout
## would make the query mean anyway.
static func _matches(path: String, meta: Dictionary, query: String) -> bool:
	if path.to_lower().contains(query):
		return true
	return String(meta.get("seed", "")).to_lower().contains(query)

## A welcome-mode action tile: same footprint and caption rhythm as a
## project tile, but a solid outline rather than the dashed-import one and a
## glyph rather than an identicon, so it reads as an action and not as a
## world you could select. `primary` gives the accent treatment the mockup's
## own primary button carries -- exactly one tile has it, matching the
## reference gate's single `class="accent"` button.
func _build_action_tile(icon: String, title: String, note: String, primary: bool, on_click: Callable) -> Control:
	var wrap := PanelContainer.new()
	wrap.custom_minimum_size = TILE_MIN
	wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	wrap.add_theme_stylebox_override("panel",
		DccTheme.outline("accent" if primary else "line", "raised" if primary else "panel"))

	var col := VBoxContainer.new()
	col.alignment = BoxContainer.ALIGNMENT_CENTER
	col.add_theme_constant_override("separation", 10)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 16)
	pad.add_theme_constant_override("margin_right", 16)
	pad.add_child(col)
	wrap.add_child(pad)

	var glyph := DccIcons.rect(icon, 30, "accent" if primary else "text")
	glyph.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	col.add_child(glyph)
	var title_label := DccTheme.label(title, "accent" if primary else "text_bright", DccTheme.FS_BODY)
	title_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	col.add_child(title_label)
	var note_label := DccTheme.mono_label(note, "text_faint", DccTheme.FS_TINY)
	note_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	col.add_child(note_label)

	_ignore_mouse(wrap)
	wrap.gui_input.connect(func(event: InputEvent):
		if event is InputEventMouseButton and (event as InputEventMouseButton).pressed:
			on_click.call())
	return wrap

func _build_import_tile() -> Control:
	var wrap := PanelContainer.new()
	wrap.custom_minimum_size = TILE_MIN
	wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## The mockup's `1px dashed rgba(224,163,74,.5)`. `StyleBoxFlat` has no
	## dash pattern and a custom `_draw` for one tile is not worth the second
	## drawing path, so this keeps the colour and weight and loses the dashes.
	var box := DccTheme.outline("accent")
	box.border_color = Color(DccTheme.c("accent"), 0.5)
	wrap.add_theme_stylebox_override("panel", box)

	var col := VBoxContainer.new()
	col.alignment = BoxContainer.ALIGNMENT_CENTER
	col.add_theme_constant_override("separation", 10)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 16)
	pad.add_theme_constant_override("margin_right", 16)
	pad.add_child(col)
	wrap.add_child(pad)

	var glyph := DccIcons.rect("import", 26, "accent")
	glyph.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	col.add_child(glyph)
	var line := DccTheme.label("Drop a .zip save\nor click to browse a folder",
		"accent", DccTheme.FS_BODY)
	line.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	col.add_child(line)

	_ignore_mouse(wrap)
	wrap.gui_input.connect(func(event: InputEvent):
		if event is InputEventMouseButton and (event as InputEventMouseButton).pressed:
			_browse_from_disk())
	return wrap

func _build_tile(path: String, meta: Dictionary) -> Control:
	var current := _host != null and _host.current_project_path == path
	var wrap := PanelContainer.new()
	wrap.custom_minimum_size = TILE_MIN
	wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	wrap.add_theme_stylebox_override("panel",
		DccTheme.outline("accent" if current else "line", "panel"))

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	wrap.add_child(col)

	## The generated identicon. See this file's header for why there is no
	## real thumbnail to draw.
	var thumb := Control.new()
	thumb.size_flags_vertical = Control.SIZE_EXPAND_FILL
	thumb.custom_minimum_size.y = 128
	thumb.clip_contents = true
	var tex := TextureRect.new()
	tex.texture = identicon(path)
	tex.set_anchors_preset(Control.PRESET_FULL_RECT)
	tex.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	tex.stretch_mode = TextureRect.STRETCH_SCALE
	tex.mouse_filter = Control.MOUSE_FILTER_IGNORE
	thumb.add_child(tex)
	if current:
		var badge := PanelContainer.new()
		badge.position = Vector2(8, 8)
		badge.add_theme_stylebox_override("panel", DccTheme.flat(Color(0, 0, 0, 0.4)))
		badge.mouse_filter = Control.MOUSE_FILTER_IGNORE
		var badge_pad := MarginContainer.new()
		badge_pad.add_theme_constant_override("margin_left", 6)
		badge_pad.add_theme_constant_override("margin_right", 6)
		badge_pad.add_theme_constant_override("margin_top", 2)
		badge_pad.add_theme_constant_override("margin_bottom", 2)
		badge_pad.add_child(DccTheme.mono_label("CURRENT", "accent", DccTheme.FS_MICRO, 1))
		badge.add_child(badge_pad)
		thumb.add_child(badge)
	col.add_child(thumb)
	col.add_child(DccTheme.rule())

	var caption := VBoxContainer.new()
	caption.add_theme_constant_override("separation", 2)
	var cap_pad := MarginContainer.new()
	cap_pad.add_theme_constant_override("margin_left", 11)
	cap_pad.add_theme_constant_override("margin_right", 11)
	cap_pad.add_theme_constant_override("margin_top", 9)
	cap_pad.add_theme_constant_override("margin_bottom", 9)
	cap_pad.add_child(caption)
	col.add_child(cap_pad)

	var title_label := DccTheme.label(path.get_file().get_basename(),
		"text_bright" if current else "text", DccTheme.FS_BODY)
	title_label.name = "Title"
	title_label.clip_text = true
	caption.add_child(title_label)
	caption.add_child(DccTheme.mono_label(
		"%s · %s" % [meta.get("seed", "seed unread"), meta.get("edited", "")],
		"text_faint", DccTheme.FS_TINY))

	_ignore_mouse(wrap)
	wrap.gui_input.connect(func(event: InputEvent):
		if not (event is InputEventMouseButton):
			return
		var mb := event as InputEventMouseButton
		if not (mb.pressed and mb.button_index == MOUSE_BUTTON_LEFT):
			return
		_select(path)
		if mb.double_click:
			_confirm())
	_tiles[path] = wrap
	if _selected == path:
		_paint_tile(path, true)
	return wrap

func _select(path: String) -> void:
	var previous := _selected
	_selected = path
	if previous != path and _tiles.has(previous):
		_paint_tile(previous, false)
	if _tiles.has(path):
		_paint_tile(path, true)
	_refresh_open_button()

func _paint_tile(path: String, on: bool) -> void:
	var wrap: PanelContainer = _tiles[path]
	var current := _host != null and _host.current_project_path == path
	wrap.add_theme_stylebox_override("panel", DccTheme.outline(
		"accent" if (on or current) else "line", "raised" if on else "panel"))
	var title_label := wrap.find_child("Title", true, false) as Label
	if title_label != null:
		title_label.add_theme_color_override("font_color",
			DccTheme.c("text_bright") if (on or current) else DccTheme.c("text"))

func _refresh_open_button() -> void:
	_open_btn.disabled = _selected == "" or not FileAccess.file_exists(_selected)
	## Every disabled control in this shell states its reason on hover -- this
	## one had no tooltip at all, so the primary action of the welcome screen
	## was greyed out and silent about why (found by the 2026-08-25 sweep's
	## disabled-without-a-reason scan, not by a user).
	_open_btn.tooltip_text = "" if not _open_btn.disabled \
		else ("Pick a world above first." if _selected == "" \
			else "%s is no longer on disk." % _selected.get_file())

func _confirm() -> void:
	if _selected == "" or not FileAccess.file_exists(_selected):
		return
	var path := _selected
	hide()
	_host.open_recent_project(path)

# ---------------------------------------------------------------------------
# Bringing one in from disk
# ---------------------------------------------------------------------------

## The dashed tile's click half. A file browse, not a folder browse, despite
## the tile's own "click to browse a folder" wording -- what it returns has to
## be a `.zip` save, and `DccBrowseDialog` browses folders on the way to one.
func _browse_from_disk() -> void:
	DccBrowseDialog.choose_file(self, "Open project — browse", PackedStringArray(["zip"]),
		DccSettings.storage_root("projects"),
		"Cartalith projects are .zip saves", func(path: String):
			hide()
			_host.open_recent_project(path))

func _on_files_dropped(files: PackedStringArray) -> void:
	if not visible:
		return
	for f in files:
		if String(f).get_extension().to_lower() == "zip":
			hide()
			_host.open_recent_project(String(f))
			return
	_foot_note.text = "that is not a .zip save"

# ---------------------------------------------------------------------------
# Per-project metadata
# ---------------------------------------------------------------------------

## `{seed, edited}` for one save. Both are display strings; nothing reads them
## back. The seed comes from the save's own `params.json` (`SAVEFILE_COMPAT.md`
## §`params.json`: `state.tect.seed`) via `ZIPReader`, which is a read of a
## stored value rather than a re-derivation of one -- the distinction the
## `godot-shell` skill's "keep logic out of GDScript" rule turns on.
##
## Public (not `_project_meta`) since `phone_project_picker.gd`'s own recents
## list reads the identical real per-save facts for its cards rather than a
## second implementation of the same `ZIPReader` walk -- the phone screen
## replaces this dialog's *gallery chrome* (§ header, search well, scope
## chips), not its data layer.
static func project_meta(path: String) -> Dictionary:
	var modified := FileAccess.get_modified_time(path)
	var key := "%s@%d" % [path, modified]
	if _meta_cache.has(key):
		return _meta_cache[key]
	var meta := {"seed": "seed unread", "edited": _relative_time(modified)}
	var zip := ZIPReader.new()
	if zip.open(path) == OK:
		if zip.file_exists("params.json"):
			var parsed = JSON.parse_string(zip.read_file("params.json").get_string_from_utf8())
			if parsed is Dictionary:
				var state = (parsed as Dictionary).get("state", {})
				if state is Dictionary:
					var tect = (state as Dictionary).get("tect", {})
					if tect is Dictionary and (tect as Dictionary).has("seed"):
						meta["seed"] = _plain_number((tect as Dictionary)["seed"])
		zip.close()
	_meta_cache[key] = meta
	return meta

## JSON has one number type, so `JSON.parse_string` hands back every seed as a
## float and `str()` renders it `483920.0`. The mockup's caption reads
## `483920 · edited 4 min ago`, and a seed is an integer everywhere else in
## this port, so an integral value prints without the tail. A non-integral one
## is left exactly as it came, rather than being rounded into a lie.
static func _plain_number(value) -> String:
	if value is float and is_equal_approx(value, floor(value)):
		return "%d" % int(value)
	return str(value)

## "edited 4 min ago" -- the mockup's own phrasing, in its own units.
static func _relative_time(unix: int) -> String:
	if unix <= 0:
		return "never opened"
	var delta := int(Time.get_unix_time_from_system()) - unix
	if delta < 60:
		return "edited just now"
	if delta < 3600:
		return "edited %d min ago" % int(delta / 60.0)
	if delta < 86400:
		return "edited %d h ago" % int(delta / 3600.0)
	if delta < 172800:
		return "edited yesterday"
	if delta < 604800:
		return "edited %d days ago" % int(delta / 86400.0)
	if delta < 2419200:
		return "edited %d weeks ago" % int(delta / 604800.0)
	return "edited " + Time.get_date_string_from_unix_time(unix)

## A stable, per-world radial wash. Hue from the path's hash so the same world
## always reads the same colour and two worlds rarely collide; saturation and
## value are fixed low, because these tiles sit behind an accent selection
## border and must never compete with it.
##
## Public alongside `project_meta()` above, for the same reason: the phone
## picker's cards want the identical stable-per-world art, not a second hash
## scheme that would tag the same world with two different colours depending
## on which screen opened it.
static func identicon(path: String) -> Texture2D:
	var hue := float(abs(path.hash()) % 360) / 360.0
	var g := Gradient.new()
	g.set_color(0, Color.from_hsv(hue, 0.30, 0.22))
	g.set_color(1, Color.from_hsv(hue, 0.22, 0.11))
	var t := GradientTexture2D.new()
	t.gradient = g
	t.fill = GradientTexture2D.FILL_RADIAL
	t.fill_from = Vector2(0.46, 0.42)
	t.fill_to = Vector2(1.0, 1.0)
	t.width = 96
	t.height = 72
	return t

## A whole tile is one click target, so nothing inside it may eat the event.
## Godot's default `mouse_filter` is `STOP` on every `Control`, containers
## included, which means an unattended `VBoxContainer` silently swallows the
## click the tile around it is listening for. Every clickable composite in this
## shell that is built from containers rather than from a `Button` needs this.
static func _ignore_mouse(node: Node) -> void:
	for child in node.get_children():
		if child is Control:
			(child as Control).mouse_filter = Control.MOUSE_FILTER_IGNORE
		_ignore_mouse(child)

func _pad(child: Control, l: int, t: int, r: int, b: int) -> MarginContainer:
	var m := MarginContainer.new()
	m.add_theme_constant_override("margin_left", l)
	m.add_theme_constant_override("margin_top", t)
	m.add_theme_constant_override("margin_right", r)
	m.add_theme_constant_override("margin_bottom", b)
	m.add_child(child)
	return m

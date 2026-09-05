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
## | `CURRENT` badge, name, `seed · fmt N · edited 4 min ago` | `_build_tile()` |
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
## ## Welcome mode -- a second composition, not a re-titled gallery
##
## `open_welcome()` shows the **cold-start picker** the 2026-08-31 environment
## canvas boots into (`design/dcc-environment-2026-08-31/Cartalith DCC
## Environment.dc.html`, `state.scr = 'picker'`): a vertically centred column
## of a wordmark, up to three world cards, a row of peer action buttons and one
## foot line. `app.gd`'s `_ready` opens it once when no world exists; phone
## goes to `phone_project_picker.gd` instead and never reaches this.
##
## **It used to be the gallery with different words in the head**, and that was
## the finding: `_paint_head()` re-lettered the modal title to `Cartalith`,
## added two action tiles ahead of the grid, and shipped the search well, the
## three scope chips, the dashed import tile and an `Open selected` button on
## the first screen a user ever sees. None of those are in the drawn picker.
## The two compositions now exist side by side under `_build()` -- `_gallery`
## (`File ▸ Open project…`, unchanged, and it matches its own "Open project
## dialog 1920" artboard almost line for line) and `_picker` -- with exactly
## one visible, chosen in `_refresh()`.
##
## **Where the three routes went.** The reference's own setup gate (reference
## HTML lines 657-666) offers three peer choices -- generate, load a `.zip`,
## import a heightmap -- and the picker canvas draws only the first two, as
## `＋ New world…` and `Open project .zip…`. There is no drawn home for the
## heightmap route, so rather than drop the only cold-start way in for a
## heightmap it takes a third button in the same row, in the row's own
## secondary treatment. That is the one element on this screen the canvas does
## not draw, it is derived from the canvas's own vocabulary, and it is
## reported as a gap rather than presented as conformance. Hidden (not
## disabled) when the loaded extension has no import binding -- an affordance
## that cannot work is worse than one that is absent, and unlike the `Shared`
## chip in the gallery there is no design element here it would be dishonest
## to drop.
##
## **What the picker deliberately does not carry**, all of it still one
## dismissal away through `File ▸ Open project…` (Ctrl+O), which opens the
## gallery: the search well, the `Recent`/`All worlds`/`Shared` chips, and any
## world past the third. The canvas draws three cards and no chrome around
## them; the cards are the shortcut, not the index.
##
## **One drawn element has no data behind it and is therefore not drawn**: the
## canvas's cards carry a pill in the thumbnail's top-left reading `ATLAS
## BAKED` / `IN PROGRESS` / `DRAFT`, over a `status` of `stages 01-10
## resolved`. Nothing in this port records how far a *saved* world got --
## `project.json` carries a format and `params.json` a seed, and the stage
## ledger is live state that is not serialised (`SAVEFILE_COMPAT.md`) -- so
## there is no honest value to put in that pill, and inventing one would label
## every world `DRAFT` or every world `ATLAS BAKED`. The slot is not empty:
## `CURRENT` already occupies it, at the same 8 px inset, and that badge is
## real. If a save ever records its own stage ledger, this is where it goes.
##
## **Where each figure comes from.** Stated by the canvas: the 40 px frame
## padding, the 34 px inter-block gap, `CARTALITH` at `500 20px` mono with
## `.34em`, the tagline at `--m1`/`.2em`/`--faint`, the 16 px card gap, the
## 252 px card, its 130 px thumbnail, its `500 13px`/`.12em` name and `--m2`
## `--dim` meta, the 10 px action gap and the `6px 18px` radius-8 pill. Derived
## from the shell's vocabulary because the canvas states nothing: the 44 px
## action height (the canvas's `--btnH` is 28, below §13's target floor, and
## this is the one screen a tablet user meets first), the foot's `--dim`
## instead of `--dis` (the prototype's foot only says its file dialogs are
## mocked; this one carries the empty-state instruction and has to be legible
## -- `--dis` measures 2.64:1 on the light panel), the `Continue without a
## world` opt-out (the canvas's picker is a gate and this port's is not), and
## the cards' `--ins` fill, which keeps the canvas's *relationship* -- a card
## lifted off the ground behind it -- where taking its literal `--pan` would
## paint the card the same colour as the modal it now sits in.

const TILE_MIN := Vector2(232, 186)
const GRID_COLUMNS := 4

## The picker card: `width:252px` with a `height:130px` thumbnail. Width only --
## the height is whatever the thumbnail plus the two caption lines come to, the
## same way the canvas's card is sized by its content.
const PICKER_TILE_W := 252
const PICKER_THUMB_H := 130
## `flex-wrap:wrap` over `width:850px` fits three 252 px cards and their two
## 16 px gaps (788 px) and no fourth, so the canvas's own row holds three. The
## cap is that figure, not a taste call; the 850 px box is not reproduced
## because three cards are inside it at every density this dialog opens at.
const PICKER_MAX_TILES := 3
## `--btnH` is 28 and `DCC_SHELL_SPEC.md` §13's floor is 44. See the header.
const PICKER_BTN_H := 44

var _host: DccApp

var _search: LineEdit
var _grid: GridContainer
var _foot_note: Label
var _open_btn: Button
var _scope := "recent"
var _scope_buttons: Dictionary = {}   ## scope id -> Button
var _selected := ""
var _tiles: Dictionary = {}           ## path -> PanelContainer

var _subtitle_label: Label

## Cold-start framing (see this file's header). Set by `open_welcome()`,
## cleared by `open()`, and read by `_refresh()`, which is the **only** place
## that decides which of the two compositions below is on screen.
var _welcome := false
var _gallery: VBoxContainer      ## `File ▸ Open project…` -- the 08-23 artboard.
var _picker: Control             ## Cold start -- the 08-31 canvas's `scr:'picker'`.
var _picker_tiles: HFlowContainer
var _picker_note: Label
var _picker_import_btn: Button   ## Held so `_refresh()` can re-ask the bridge.

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

## The cold-start prompt. **Rewritten 2026-09-05 and this doc with it:** it is no
## longer "the same gallery framed as start here" — it is the 08-31 canvas's
## `state.scr = 'picker'` centred column, and it carries THREE routes out
## (Create / Open / Import), not two. The old sentence survived the rewrite that
## falsified it and a verifier caught it; see this file's header for why welcome
## is a mode rather than its own dialog.
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

## An `AcceptDialog` sizes **one** content child, so both compositions hang off
## a single `outer` column and swap by `visible`. A `BoxContainer` skips hidden
## children when it computes its own minimum, so the hidden half costs the
## dialog no width and no height -- which is what lets a 252 px picker and an
## 880 px gallery share one window without either widening the other.
func _build() -> void:
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)
	_gallery = _build_gallery()
	outer.add_child(_gallery)
	_picker = _build_picker()
	## `_refresh()` sets both every time; this is the state before the first
	## one runs, and `_welcome` starts false.
	_picker.visible = false
	outer.add_child(_picker)

func _build_gallery() -> VBoxContainer:
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	outer.size_flags_vertical = Control.SIZE_EXPAND_FILL

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
	return outer

func _build_head() -> Control:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 14)
	## One wording, not two. Both labels used to be re-lettered by a
	## `_paint_head()` that turned this head into the welcome screen's; the
	## welcome screen is `_build_picker()` now and never touches this row.
	row.add_child(DccTheme.label("Open project", "text_bright", DccTheme.FS_MODAL_TITLE))
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
	DccWidgets.modal_button(row, "Cancel", func(): hide())
	_open_btn = DccWidgets.modal_button(row, "Open selected", _confirm, true)
	_open_btn.disabled = true
	return _pad(row, 30, 14, 30, 14)

# ---------------------------------------------------------------------------
# The cold-start picker (08-31 canvas, `state.scr = 'picker'`)
# ---------------------------------------------------------------------------

## `padding:40px` around a column that is `justify-content:center` (a
## `BoxContainer` with `ALIGNMENT_CENTER`, which centres its children along its
## own axis when there is spare room) and `align-items:center` -- which in
## Godot is not one property but one per child: a `Label` centres its text, a
## `FlowContainer` centres its line.
##
## `gap:34px` separates four blocks, and the second of them disappears
## entirely on a fresh profile. That is why the tile row's visibility is
## toggled rather than its contents merely emptied: an empty `HFlowContainer`
## still takes a full 34 px gap on each side of nothing, which is the "looks
## broken on first run" the empty state has to avoid.
func _build_picker() -> Control:
	## **There is deliberately no `ScrollContainer` here, and that was measured
	## rather than assumed.** One was fitted first, because `wrap_controls` is
	## false on this dialog (`DccWidgets.phone_window()` sets it) so the window
	## does not grow to fit its content and an overflow clips instead of
	## scrolling. It made things worse: a `ScrollContainer` reserves 20 px for
	## its vertical bar, which leaves 780 px of row at the dialog's 880 x 560
	## `min_size` floor -- eight short of the 788 three cards need -- so the row
	## wrapped to two lines and the column measured 761 px tall where it had
	## been 552. A guard that creates the overflow it exists to absorb is worse
	## than the eight pixels of headroom it was protecting.
	##
	## What bounds the height instead is the foot's `max_lines_visible` below.
	## Every other block on this screen is fixed: the wordmark is two lines, the
	## card row is capped at `PICKER_MAX_TILES` on one row, and the actions are
	## one row of 44 px pills. The foot was the only thing that could grow, and
	## it now cannot grow past two lines.
	var frame := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		frame.add_theme_constant_override("margin_" + side, 40)
	frame.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	frame.size_flags_vertical = Control.SIZE_EXPAND_FILL

	var col := VBoxContainer.new()
	col.alignment = BoxContainer.ALIGNMENT_CENTER
	col.add_theme_constant_override("separation", 34)
	frame.add_child(col)

	## `font:500 20px 'IBM Plex Mono';letter-spacing:.34em` over
	## `var(--m1)`/`.2em`/`var(--faint)`, `gap:8px`. Tracking is whole pixels
	## in Godot (`FontVariation.spacing_glyph`), so `.34em` at 20 px is 6.8 ->
	## 7 and `.2em` at 10 px is exactly 2.
	var mark := VBoxContainer.new()
	mark.add_theme_constant_override("separation", 8)
	mark.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var word := DccTheme.mono_label("CARTALITH", "text_bright", 20, 7, true)
	word.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	mark.add_child(word)
	## The canvas reads `WORLD CONSTRUCTION · 2.11 DESKTOP`. The version half is
	## dropped: the prototype is naming its own artboard, and "2.11" here would
	## be this shell asserting parity with `reference/Cartalith Gen1 v2.11.html`
	## -- a claim no code checks and nothing would update.
	var tag := DccTheme.mono_label("WORLD CONSTRUCTION", "text_faint", DccTheme.FS_TINY, 2)
	tag.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	mark.add_child(tag)
	col.add_child(mark)

	## `display:flex;gap:16px;flex-wrap:wrap;justify-content:center`.
	_picker_tiles = HFlowContainer.new()
	_picker_tiles.alignment = FlowContainer.ALIGNMENT_CENTER
	_picker_tiles.add_theme_constant_override("h_separation", 16)
	_picker_tiles.add_theme_constant_override("v_separation", 16)
	_picker_tiles.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col.add_child(_picker_tiles)

	## `display:flex;gap:10px`. An `HFlowContainer` rather than an `HBox` so the
	## three buttons wrap instead of overlapping when the dialog is dragged
	## narrow -- a `BoxContainer` handed less width than its minimum overlaps,
	## which is the fault `_apply_phone_toolbar()` exists to undo one screen up.
	var actions := HFlowContainer.new()
	actions.alignment = FlowContainer.ALIGNMENT_CENTER
	actions.add_theme_constant_override("h_separation", 10)
	actions.add_theme_constant_override("v_separation", 10)
	actions.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_picker_button(actions, "%s New world…" % DccIcons.SYMBOLS["add"], true, func():
		hide()
		_host.open_new_world())
	_picker_button(actions, "Open project .zip…", false, _browse_from_disk)
	## The reference gate's third choice, which the canvas does not draw. See
	## this file's header; visibility is re-asked every `_refresh()`, because
	## `setup()` runs while `app.gd` is still standing the bridge up.
	_picker_import_btn = _picker_button(actions, "Import a heightmap…", false, func():
		hide()
		_host.open_heightmap_import())
	col.add_child(actions)

	## The canvas's one foot line, plus the opt-out it has no need for: its
	## picker is a gate and this one is not (`app.gd`'s `_ready`). `--dim`, not
	## the canvas's `--dis`: this line carries the empty-state instruction.
	var foot := VBoxContainer.new()
	foot.add_theme_constant_override("separation", 10)
	foot.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_picker_note = DccTheme.mono_label("", "text_dim", DccTheme.FS_MICRO)
	_picker_note.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	## Wrapped, not clipped. A storage root is an absolute path and the column
	## is only as wide as the dialog; `clip_text` would collapse this label's
	## minimum width to 1 px (`MISTAKES.md`) and hide the instruction rather
	## than the path.
	##
	## Two lines is the cap, and it is what makes this composition's height a
	## known quantity -- see `_build_picker()`. The measured worst case already
	## needs both: at the 880 px floor the truncation notice plus a real
	## `user://` storage root wraps once. A third line would put the opt-out
	## past the bottom edge, so a longer root ellipsises the path instead.
	_picker_note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_picker_note.max_lines_visible = 2
	_picker_note.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	foot.add_child(_picker_note)
	var out := DccWidgets.text_button(foot, "Continue without a world", func(): hide())
	out.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	## `maxf`, not an assignment: `DccWidgets.text_button()` already floors this
	## at `PHONE_TAP_MIN * phone_scale()` on a handset, which is larger than 44,
	## and overwriting it would shrink the one target this factory already
	## sizes correctly. Desktop and tablet get nothing from it, hence the raise.
	out.custom_minimum_size.y = maxf(out.custom_minimum_size.y, PICKER_BTN_H)
	col.add_child(foot)
	return frame

## The canvas's action pill: `min-height:var(--btnH);padding:6px 18px;
## border-radius:8px`, `background:var(--acc);color:var(--accInk)` on the
## primary and `background:var(--ins);color:var(--sec)` on the rest. Radius 8
## overrides §11's "radius 0 everywhere", which is a rule about the desktop
## *shell*; this screen's own canvas answers the question differently and is
## the newer of the two.
##
## Height is the one figure not taken from the canvas -- see the header. The
## hover is derived too: the canvas gives these buttons none, so the secondary
## borrows the accent border its own sibling card hovers with
## (`style-hover="border-color:var(--acc)"`), which is the only hover
## vocabulary this screen has.
##
## Measured pairs, both palettes: primary ink on accent 8.60:1 dark / 4.30:1
## light; secondary `--sec` on `--ins` 7.58:1 / 8.87:1; secondary hover ink
## 14.29:1 / 15.62:1. The light primary is the shell-wide `accent_ink`-on-
## `accent` pair, below 4.5:1 and reported rather than locally patched.
func _picker_button(parent: Control, text: String, primary: bool, on_press: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_NONE
	b.custom_minimum_size = Vector2(0, PICKER_BTN_H)
	b.add_theme_font_size_override("font_size", DccTheme.FS_BODY)
	b.add_theme_color_override("font_color",
		DccTheme.c("accent_ink") if primary else DccTheme.c("text_secondary"))
	b.add_theme_color_override("font_hover_color",
		DccTheme.c("accent_ink") if primary else DccTheme.c("text_bright"))
	var rest := DccTheme.pill(primary, 8, 18, 6)
	var hover := DccTheme.pill(primary, 8, 18, 6)
	if primary:
		hover.bg_color = DccTheme.c("accent_hover")
	else:
		rest.bg_color = DccTheme.c("sunken")
		rest.set_border_width_all(0)
		hover.bg_color = DccTheme.c("sunken")
		hover.border_color = DccTheme.c("accent")
	for state in ["normal", "pressed", "disabled"]:
		b.add_theme_stylebox_override(state, rest)
	b.add_theme_stylebox_override("hover", hover)
	b.pressed.connect(on_press)
	parent.add_child(b)
	return b

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

## The worlds the picker offers: recents first, in the order `DccSettings`
## maintains, then every other `.zip` in the projects root. That union is the
## two scope chips the picker does not draw, and it is the honest answer for
## the case `open_welcome()` is *for* -- a re-install or a wiped config keeps
## the worlds folder and loses the recents list, so a "recent"-only picker
## would tell a user with twelve saved worlds that they have none.
##
## Written by swapping `_scope` around `_paths()` rather than restating either
## branch, so a change to how a scope is read reaches both screens.
func _welcome_paths() -> Array:
	var keep := _scope
	_scope = "recent"
	var out := _paths()
	_scope = "all"
	for p in _paths():
		if not out.has(p):
			out.append(p)
	_scope = keep
	return out

## The one place that decides which composition is on screen. Both holders are
## emptied every time, not just the one about to be filled: the two screens
## read the same worlds folder, so a world deleted between one open and the
## next would otherwise still be drawn in whichever half was not rebuilt, and
## `_tiles` -- which only the gallery writes -- would name tiles that are no
## longer the ones on screen.
##
## `remove_child` before `queue_free`: freeing alone is deferred to the end of
## the frame, so two refreshes inside one frame (opening the dialog and the
## first keystroke in the search well) would rebuild on top of tiles that are
## still parented.
func _refresh() -> void:
	for holder in [_grid, _picker_tiles]:
		for c in holder.get_children():
			holder.remove_child(c)
			c.queue_free()
	_tiles.clear()
	_gallery.visible = not _welcome
	_picker.visible = _welcome
	if _welcome:
		_refresh_picker()
	else:
		_refresh_gallery()
	## The composition just changed, and on a phone its width is what the window
	## has to be re-measured against -- see `_fit_phone_content()` for why that
	## does not happen on its own. Runs on every keystroke in the search well,
	## which is what `child_controls_changed()` is cheap enough for.
	if _phone:
		_fit_phone_content()

func _refresh_gallery() -> void:
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
	elif shown == 0:
		_foot_note.text = "nothing here yet · projects read from %s" % root
	else:
		_foot_note.text = "projects read from %s" % root
	_refresh_open_button()

## Three cards at most (`PICKER_MAX_TILES`), and none at all on a fresh
## profile, where the row is hidden outright so the 34 px gaps around it close
## up and the wordmark sits directly over the three actions -- which is then
## exactly the reference's own setup gate.
func _refresh_picker() -> void:
	var paths := _welcome_paths()
	var shown: int = mini(paths.size(), PICKER_MAX_TILES)
	for i in shown:
		_picker_tiles.add_child(_build_tile(String(paths[i]),
			project_meta(String(paths[i])), true))
	_picker_tiles.visible = shown > 0
	## Re-asked here, not at build time: `setup()` runs from `app.gd`'s
	## `_ready` while the bridge is still resolving which `#[func]`s the loaded
	## extension actually exports.
	_picker_import_btn.visible = _host != null and _host.bridge.import_api

	var root := DccSettings.storage_root("projects")
	if paths.is_empty():
		## The first-ever launch. "nothing here yet" alone reads as a fault;
		## the actions below are the answer, so the foot names them.
		_picker_note.text = "no saved worlds yet — start one below · projects read from %s" % root
	elif paths.size() > shown:
		## Counted, not asserted: `paths` is the union `_welcome_paths()` just
		## walked, and the rest of it is reached through the gallery.
		_picker_note.text = "%d of %d worlds · the rest are in File ▸ Open project… · projects read from %s" \
			% [shown, paths.size(), root]
	else:
		_picker_note.text = "projects read from %s" % root

## The search well offers "name, seed or region". Name and seed are real
## fields; "region" has no equivalent -- a save carries no region name -- so
## the path itself stands in for it, which is what a folder-per-region layout
## would make the query mean anyway.
static func _matches(path: String, meta: Dictionary, query: String) -> bool:
	if path.to_lower().contains(query):
		return true
	return String(meta.get("seed", "")).to_lower().contains(query)

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

## `picker` draws the same world as the 08-31 canvas's picker card rather than
## the 08-23 gallery's tile: 252 px wide and content-tall instead of the 232 x
## 186 grid cell, a 130 px thumbnail, and the name in `500 13px` mono tracked
## `.12em` (1.56 px -> 2) where the gallery sets it in prose at 12. The two
## canvases disagree about the name's face and the newer one wins for its own
## screen.
##
## The fill is `--ins`, not the canvas's `--pan`. The canvas card sits on
## `--sur`; here it sits inside a modal whose own panel is already `--pan`
## (`dcc_shell.gd`'s `AcceptDialog` stylebox), so taking the literal token
## would paint the card the colour of the surface behind it and leave only the
## hairline. `--ins` is the same *relationship* -- one surface step above the
## ground -- which is the property the card was drawn for.
func _build_tile(path: String, meta: Dictionary, picker: bool = false) -> Control:
	var current := _host != null and _host.current_project_path == path
	var wrap := PanelContainer.new()
	wrap.custom_minimum_size = Vector2(PICKER_TILE_W, 0) if picker else TILE_MIN
	if not picker:
		wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	wrap.add_theme_stylebox_override("panel",
		DccTheme.outline("accent" if current else "line", "sunken" if picker else "panel"))

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	wrap.add_child(col)

	## The generated identicon. See this file's header for why there is no
	## real thumbnail to draw.
	var thumb := Control.new()
	thumb.size_flags_vertical = Control.SIZE_EXPAND_FILL
	thumb.custom_minimum_size.y = PICKER_THUMB_H if picker else 128
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

	## `padding:12px 14px;gap:4px` on the picker card; `9px 11px` and 2 on the
	## gallery tile.
	var caption := VBoxContainer.new()
	caption.add_theme_constant_override("separation", 4 if picker else 2)
	var cap_pad := MarginContainer.new()
	cap_pad.add_theme_constant_override("margin_left", 14 if picker else 11)
	cap_pad.add_theme_constant_override("margin_right", 14 if picker else 11)
	cap_pad.add_theme_constant_override("margin_top", 12 if picker else 9)
	cap_pad.add_theme_constant_override("margin_bottom", 12 if picker else 9)
	cap_pad.add_child(caption)
	col.add_child(cap_pad)

	var name_token := "text_bright" if (current or picker) else "text"
	var title_label := DccTheme.mono_label(path.get_file().get_basename(),
			name_token, 13, 2, true) if picker \
		else DccTheme.label(path.get_file().get_basename(), name_token, DccTheme.FS_BODY)
	title_label.name = "Title"
	title_label.clip_text = true
	caption.add_child(title_label)
	## `seed · fmt N · edited 4 min ago`. The format number is this *save's*
	## own (read above), not `EngineBridge.project_format_version()`, which is
	## the version this build writes -- printing the build constant on every
	## tile would say nothing about the file the tile opens.
	var fmt := int(meta.get("format", 0))
	var fmt_part := ("fmt %d · " % fmt) if fmt > 0 else ""
	## `var(--m2)`/`var(--dim)` on the picker card, `10px`/`--faint` on the
	## gallery tile -- both stated by their own canvas.
	caption.add_child(DccTheme.mono_label(
		"%s · %s%s" % [meta.get("seed", "seed unread"), fmt_part, meta.get("edited", "")],
		"text_dim" if picker else "text_faint",
		DccTheme.FS_MICRO if picker else DccTheme.FS_TINY))

	_ignore_mouse(wrap)
	if picker:
		## The canvas card is `onClick="{{ hPickWorld }}"` straight to the shell
		## -- there is no selection on this screen and no `Open selected` button
		## for one to feed, so one click opens. `style-hover="border-color:
		## var(--acc)"` is the only affordance saying it is clickable, and it is
		## load-bearing here in a way it is not in the gallery.
		var rest := DccTheme.outline("accent" if current else "line", "sunken")
		var hot := DccTheme.outline("accent", "sunken")
		wrap.mouse_entered.connect(func(): wrap.add_theme_stylebox_override("panel", hot))
		wrap.mouse_exited.connect(func(): wrap.add_theme_stylebox_override("panel", rest))
		wrap.gui_input.connect(func(event: InputEvent):
			if not (event is InputEventMouseButton):
				return
			var mb := event as InputEventMouseButton
			if not (mb.pressed and mb.button_index == MOUSE_BUTTON_LEFT):
				return
			## The same guard `_confirm()` has, in the same words as
			## `_refresh_open_button()`'s tooltip: a card is built from a
			## directory listing, and the file behind it can be gone by the
			## time it is clicked. Rebuild first, then say why -- `_refresh()`
			## writes the foot itself, so the message has to land after it.
			if not FileAccess.file_exists(path):
				_refresh()
				_say("%s is no longer on disk." % path.get_file())
				return
			hide()
			_host.open_recent_project(path))
		return wrap
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
	_say("that is not a .zip save")

## `files_dropped` is a *window* signal, so a drop lands on whichever
## composition is up -- and the gallery's foot is not on screen in welcome
## mode. Writing to `_foot_note` alone put the only feedback for a bad drop
## into a hidden control the moment welcome stopped being the gallery.
func _say(text: String) -> void:
	if _welcome:
		_picker_note.text = text
	else:
		_foot_note.text = text

# ---------------------------------------------------------------------------
# Per-project metadata
# ---------------------------------------------------------------------------

## `project.json`'s own `format` member, which identifies the archive as this
## application's. `cartalith-io/src/project.rs` declares it as
## `pub const PROJECT_FORMAT: &str = "cartalith-project"` and refuses any
## archive whose manifest says something else or nothing at all. Duplicated as
## a literal only because no binding exposes it; if one is ever added,
## `project_meta()` should read it instead of this constant.
const PROJECT_FORMAT := "cartalith-project"

## `{seed, edited, format}` for one save. The first two are display strings and
## nothing reads them back; `format` is the save's own `format_version`
## (`SAVEFILE_COMPAT.md` §4) as an integer, 0 when unread.
## The seed comes from the save's own `params.json` (`SAVEFILE_COMPAT.md`
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
	var meta := {"seed": "seed unread", "edited": _relative_time(modified), "format": 0}
	var zip := ZIPReader.new()
	if zip.open(path) == OK:
		## `project.json`'s `format_version` (`SAVEFILE_COMPAT.md` §4), which a
		## reader MUST check and which nothing in this shell ever showed. It is
		## the first number asked for when an old save misbehaves, so the tile
		## that offers to open it is where it belongs. 0 means "not read" --
		## an absent or unparsable header, which §4 says a reader refuses -- and
		## the caption omits the field rather than printing a version this file
		## never claimed.
		##
		## **The `format` identity test comes first, and did not used to exist.**
		## The engine refuses an archive whose `project.json` carries a different
		## `format`, or none at all (`cartalith-io/src/project.rs`, twice: `match
		## manifest.get("format") { Some(PROJECT_FORMAT) => {}, Some(other) =>
		## Err(NotAProject(other)), None => Err(NotAProject("")) }`). Reading only
		## `format_version` here meant any unrelated `.zip` that happened to
		## contain a `project.json` with that key got a confident `format 1`
		## caption on a tile offering to open it -- a caption the loader behind
		## the tile would then refuse. `phone_project_picker.gd` reads this same
		## static, so both pickers gain the test together. `PROJECT_FORMAT`'s value
		## is spelled again here only because no binding exposes it: the way to end
		## that duplication is a `project_format()` wrapper in `engine_bridge.gd`
		## beside the existing `project_format_version()`, returning
		## `cartalith_io::PROJECT_FORMAT` from the engine.
		if zip.file_exists("project.json"):
			var head = JSON.parse_string(zip.read_file("project.json").get_string_from_utf8())
			if head is Dictionary \
					and String((head as Dictionary).get("format", "")) == PROJECT_FORMAT \
					and (head as Dictionary).has("format_version"):
				meta["format"] = int((head as Dictionary)["format_version"])
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

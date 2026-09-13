extends AcceptDialog
class_name OpenProjectDialog

## File ▸ Open project…, drawn from the "Open project dialog 1920" screen in
## `design/Cartalith DCC Shell.dc.html`.
##
## **That canvas is superseded, and nothing in the current set redraws this
## screen.** `design/mcp-2026-09-07/` is the live canvas set (PC, Tablet,
## Android); grepped 2026-09-07, `All worlds` appears **0** times in the PC
## and Tablet canvases and the string `gallery grid — thumbnails, not a
## tree list` only in `Cartalith DCC Shell.dc.html`. So the citation is kept
## rather than moved or deleted: it names the last canvas that actually drew
## the gallery, which is the honest source for every figure in the table
## below, and a figure with no source is worse than one with a dated source.
## What the current set *does* draw is the cold-start picker in
## §"Welcome mode" below -- a different composition, cited separately
## there. **If the gallery is ever redrawn, this comment and the table under
## it are what a conformance pass must re-measure**; until then, treat every
## gallery figure here as last-verified against the 08-23 artboard and not
## as current-canvas conformance.
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
## - **Thumbnails are generated, not stored -- and since 2026-09-07 they are a
##   render of the world, not an identicon.** The owner's words: *"Currently the
##   tiles are with a color gradient, I'd like that to be a smaller version of
##   the map."* This bullet used to say a `.zip` carries *"no preview image, so
##   there is nothing to show"*, and **that conclusion was wrong from the
##   premise up**: a save needs no preview image, because it already carries
##   everything needed to draw its own coastline. `SAVEFILE_COMPAT.md` makes
##   `rasters/heightmap.f32` a MUST (§8, refuse-if-absent) and
##   `world.grid_width`, `world.grid_height` and `world.sea_level` MUSTs (§7,
##   all three refuse-if-absent). Every conforming archive is therefore
##   self-drawing, with no new zip entry and no `format_version` bump.
##   `thumbnail()` is that render; `identicon()` stays, unchanged, as the
##   fallback for an archive that cannot supply the four (§6.4a's damage
##   ladder, a mid-write file, or a foreign `.zip` the gallery still lists).
## - **Seed and edit time are real.** The time is the file's own mtime; the
##   seed is read out of the save's `params.json` (`state.tect.seed`), which
##   is display metadata, not a computation -- nothing downstream reads it.
##
## ## Welcome mode -- a second composition, not a re-titled gallery
##
## `open_welcome()` shows the **cold-start picker** the environment canvas
## boots into (`design/mcp-2026-09-07/Cartalith DCC Environment.dc.html`
## lines 28-53, `state.scr = 'picker'`): a vertically centred column
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
## `＋ New world…` and `Open project .ctl…`. There is no drawn home for the
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
## **Re-anchored 2026-09-07, and the figures did not move.** This section
## used to cite `design/dcc-environment-2026-08-31/`, which the
## `design/mcp-2026-09-07/` set supersedes. `diff` of the two files' picker
## blocks (both lines 28-53) is **empty** -- byte-identical -- so every
## measurement below is re-anchored to the current canvas unchanged, and
## none of them needed correcting. The one figure this port *did* move on
## 2026-09-07 is the action row's corner radius, which is no longer the
## canvas's literal 8 on a tablet; see `_picker_button()`.
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
## The extensions a Cartalith project may carry, newest first.
##
## **`.ctl` is the extension; PKZIP is still the container.** Owner decision,
## 2026-09-07 -- a distinct extension stops this picker offering archives the
## reader will refuse, which is what happened to a 2024 `Werk.zip`.
## `SAVEFILE_COMPAT.md` §3 constrains the container, the entry names, the
## compression methods and zip64, and **says nothing about the archive's own
## filename** -- so this is conformant today and needs no `format_version`
## bump.
##
## **`zip` is not legacy support to be dropped later.** Every world saved
## before today carries it, and the HTML app reads `.zip`; removing it would
## empty this picker on upgrade.
## A plain `Array`, because `PackedStringArray(...)` is not a constant
## expression in GDScript -- the call sites that need the packed form wrap it.
const PROJECT_EXTENSIONS := ["ctl", "zip"]

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
	row.add_child(_head_close())
	return _pad(row, 30, 22, 30, 16)

## **The fourth button path in this file, found the way `_picker_button()` was.**
##
## `_wincensus_probe.gd` walks the built tree and reports every control carrying
## no theme override of its own. On this dialog it returned, at both densities:
##
##   '✕'  flat=true min=0x0 size=34x35  box-:["normal","hover","pressed",
##                                            "disabled"]
##                                      ink-:["font_pressed_color",
##                                            "font_disabled_color"]
##
## **Four styleboxes missing, not one** -- `flat = true` suppresses stylebox
## drawing outright (`data_manager_window.gd::_rail_row()` records the same
## measurement from the other side: *"a flat Button draws no stylebox at all"*),
## so this control had no rest ground, **no hover ground**, and no pressed
## ground, while the two sibling windows' way out is a `DccWidgets.chip()` with
## all four. And `font_pressed_color` fell through to Godot's stock `Button`
## theme -- a near-white that lands on the light palette's own panel, so the
## glyph vanished for the duration of the press **on the theme this project
## prefers**. Exactly the pair of defects `_picker_button()` was fixed for last
## batch, on the other composition of the same dialog.
##
## **No factory covers a window-header closer**, which is why the two passes
## that repointed `action()`, `modal_button()` and `_picker_button()` all went
## past this one. The vocabulary it now matches is `DccWidgets.modal_card()`'s
## own inline closer, the shell's single existing answer for this control:
## a `MODAL_CTL` box, `empty` at rest and pressed, and a `MODAL_INSET_RADIUS`
## `line_soft` ground on hover. Every symbol it needs is public.
##
## **Two figures are deliberately NOT modal_card's**, and both are geometry
## this pass is not allowed to move:
##
## * `FS_MODAL_TITLE`, not `FS_SMALL`. The glyph sits beside a
##   `FS_MODAL_TITLE` window title here, where `modal_card()`'s sits beside a
##   `FS_MICRO` tracked caps line. Measured before this change at 34 x 35;
##   `MODAL_CTL` (24) is a *minimum* under that, so it floors without growing.
## * `text_ghost` at rest, not `text_faint`. No canvas in the current set draws
##   this screen's head (see this file's own header), so the rest ink has no
##   source to move it to and stays where it was.
func _head_close() -> Button:
	var close := Button.new()
	close.text = DccIcons.SYMBOLS["cross"]
	close.focus_mode = Control.FOCUS_NONE
	close.custom_minimum_size = Vector2(DccWidgets.MODAL_CTL, DccWidgets.MODAL_CTL)
	close.add_theme_font_override("font", DccTheme.mono())
	close.add_theme_font_size_override("font_size", DccTheme.FS_MODAL_TITLE)
	close.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
	close.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	close.add_theme_color_override("font_pressed_color", DccTheme.c("text_bright"))
	close.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	for state in ["normal", "pressed", "disabled"]:
		close.add_theme_stylebox_override(state, DccTheme.empty())
	close.add_theme_stylebox_override("hover",
		DccTheme.flat(DccTheme.c("line_soft"), DccWidgets.MODAL_INSET_RADIUS))
	close.pressed.connect(func(): hide())
	return close

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
			"reason": "No shared or remote project concept exists in this port — projects are local .ctl saves only."},
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
	## **Which of the two variants each button is, and why.** The canvas
	## states it for the first two and they are not a judgement call:
	## `hPickNew` is `background:var(--acc);color:var(--accInk);font-weight:
	## 600` -- the one filled slab on the screen -- and `hPickZip` is
	## `background:var(--ins);color:var(--sec)`. Creating a world is what
	## this screen is *for*; opening one that already exists is the peer
	## route, and the cards above it are the faster way to do the same
	## thing, so the canvas gives it the quieter chip. One primary per
	## screen is also the rule the rest of this shell keeps.
	_picker_button(actions, "%s New world…" % DccIcons.SYMBOLS["add"], true, func():
		hide()
		_host.open_new_world())
	## **The label names the extension this build WRITES.** It read
	## "Open project .zip…" until 2026-09-07 and was caught on glass one
	## screenshot after the rename landed -- a button still advertising the old
	## extension while Save-as produced the new one. The picker opens BOTH
	## (`PROJECT_EXTENSIONS`), so the label names the one a person will have
	## just made rather than listing the pair.
	_picker_button(actions, "Open project .ctl…", false, _browse_from_disk)
	## The reference gate's third choice, which the canvas does not draw. See
	## this file's header; visibility is re-asked every `_refresh()`, because
	## `setup()` runs while `app.gd` is still standing the bridge up.
	##
	## Secondary, and the variant is the *reason* this button is allowed to
	## exist at all: it is the one element here the canvas does not draw, so
	## it takes the treatment its drawn sibling already has rather than a
	## third one invented for it. A second amber slab would give a derived
	## affordance more weight than the two the design actually specifies.
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

## The canvas's action row, drawn by the same `DccTheme.button_box()` factory
## `DccWidgets.action()` and `DccWidgets.modal_button()` take. This is the
## **third** button path the owner's 2026-09-07 *"all buttons and inputs are
## visually not the same"* covers: it goes through neither of those two, so
## repointing them left it behind -- on the first screen a user ever sees,
## beside nothing else to compare it against.
##
## **Measured in `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html`
## lines 46-49**, the current canvas for this screen (`scr:'picker'`):
##
## * primary   `min-height:var(--btnH);padding:6px 18px;border-radius:8px;
##   background:var(--acc);color:var(--accInk);font-weight:600`
## * secondary `min-height:var(--btnH);padding:6px 18px;border-radius:8px;
##   background:var(--ins);color:var(--sec)`
##
## **Neither carries a `border:` declaration**, which is what the block that
## stood here had to stop drawing.
##
## `6px 18px` stays a literal, because this screen's own canvas states its
## padding. The radius does not: it takes `button_box()`'s default,
## `role_px("btn_radius")` = 8 pointer / 12 tablet, because the tablet canvas
## writes every control corner as `--rCtl:12px` and contains no 8 at all. That
## is the owner's 2026-09-07 *"the radius should follow the newest designs"*,
## and it is the one figure this change moves on a tablet.
##
## ### What sharing the factory repairs, beyond looking the same
##
## The block that stood here built **two** boxes for four states -- `normal ==
## pressed == disabled` -- so a disabled route was pixel-identical to a live
## one, and it overrode `font_color` and `font_hover_color` only, leaving
## `pressed` and `disabled` ink to Godot's stock `Button` theme (a near-white
## that lands on the primary's amber ground). Both are the defect `action()`
## was fixed for last; `button_box()` supplies all four boxes and the three
## overrides below complete the ink.
##
## The secondary hover also stops inventing a border. It borrowed a 1 px accent
## edge from the sibling *card*'s `style-hover="border-color:var(--acc)"`; the
## canvas's own `--ins` chip hovers `style-hover="color:var(--acc)"` -- fill
## held, ink to accent -- which is the nearer vocabulary and is what the rest
## of the shell now does.
##
## Height is still the one figure not taken from the canvas: `--btnH` is 28 and
## `PICKER_BTN_H` is the 44 px target floor, for the reason the header gives.
##
## Contrast, computed over the canvas's own two palettes (`--ins` `#191c1e` /
## `#eceae4`, `--acc` `#e0a34a` / `#a4650f`, `--sec` `#a9adb0` / `#3d3f39`):
## primary ink on accent **8.60:1** dark / **4.30:1** light; secondary rest
## **7.58:1** / **8.87:1**; secondary hover and pressed ink **7.75:1** /
## **3.93:1**; disabled **2.86:1** / **2.29:1**. Two are reported rather than
## locally patched: the light primary and the light accent hover are both the
## shell-wide `accent`/`accent_ink` pairs `DccWidgets.action()` already uses
## everywhere, so a local override here would make this screen the odd one out
## in the direction the owner just asked us to stop. The disabled pair is
## deliberate -- WCAG exempts inactive controls, and `button_box()`'s own
## header explains why the drop is the point.
func _picker_button(parent: Control, text: String, primary: bool, on_press: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.focus_mode = Control.FOCUS_NONE
	b.custom_minimum_size = Vector2(0, PICKER_BTN_H)
	b.add_theme_font_size_override("font_size", DccTheme.FS_BODY)
	b.add_theme_color_override("font_color",
		DccTheme.c("accent_ink") if primary else DccTheme.c("text_secondary"))
	b.add_theme_color_override("font_hover_color",
		DccTheme.c("accent_ink") if primary else DccTheme.c("accent"))
	b.add_theme_color_override("font_pressed_color",
		DccTheme.c("accent_ink") if primary else DccTheme.c("accent"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	for state in ["normal", "hover", "pressed", "disabled"]:
		b.add_theme_stylebox_override(state,
			DccTheme.button_box(primary, state, 18, 6))
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
			## **Both, and this comparison is what makes a save visible at all.**
			## The project extension became `.ctl` on 2026-09-07 (owner: a distinct
			## extension stops the picker offering archives the reader will refuse
			## -- `Werk.zip`, a 2024 archive, was offered and then rejected with
			## "missing zip entry: params.json"). The CONTAINER is unchanged: still
			## a standard PKZIP, and `SAVEFILE_COMPAT.md` §3 never mandated a file
			## extension -- the only extensions it constrains are ENTRY names.
			## **`zip` stays in the list or every existing world disappears from
			## this picker on upgrade.**
			if String(f).get_extension().to_lower() in PROJECT_EXTENSIONS:
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
	## Names the extension this build writes; the handler accepts both.
	var line := DccTheme.label("Drop a .ctl save\nor click to browse a folder",
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

	## The world's own map, or the identicon behind it -- see `thumbnail()`.
	var thumb := Control.new()
	thumb.size_flags_vertical = Control.SIZE_EXPAND_FILL
	thumb.custom_minimum_size.y = PICKER_THUMB_H if picker else 128
	thumb.clip_contents = true
	var tex := TextureRect.new()
	tex.texture = thumbnail(path)
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
			_pick_path(path))
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

## Lane GATE, 2026-09-13 -- the one place every route that hands a chosen path
## to `open_recent_project()` (the picker tile above, `_confirm()`, the disk
## browser and a file drop, all four below) ends up. **Hides only on success.**
## Until this fix all four hid first and opened second: a corrupt or unreadable
## archive still refused (`app.gd::_load_project()`'s own `refusal` branch,
## already unchanged by this fix) but this dialog was already gone by the time
## that ran, so the failure's `set_status("hint", ...)`/`_show_phone_toast()`
## landed on a bare, world-less main shell instead of on the screen the user
## was still looking at. `open_recent_project()` (`app.gd`) is a synchronous
## call -- `EngineBridge.load_save()` underneath it returns before this
## function's own next line runs -- so checking its return here rather than
## hiding unconditionally beforehand changes nothing about a *successful*
## open's sequence, only a failed one's. Filed as Lane GATE part B; reproduced
## with a corrupt `.ctl` through the welcome recent tile, this dialog's own
## Open button and the disk browser (`_openfailgate_probe.gd`).
func _pick_path(path: String) -> void:
	if _host.open_recent_project(path):
		hide()

func _confirm() -> void:
	if _selected == "" or not FileAccess.file_exists(_selected):
		return
	_pick_path(_selected)

# ---------------------------------------------------------------------------
# Bringing one in from disk
# ---------------------------------------------------------------------------

## The dashed tile's click half. A file browse, not a folder browse, despite
## the tile's own "click to browse a folder" wording -- what it returns has to
## be a `.zip` save, and `DccBrowseDialog` browses folders on the way to one.
func _browse_from_disk() -> void:
	DccBrowseDialog.choose_file(self, "Open project — browse",
		PackedStringArray(PROJECT_EXTENSIONS),
		DccSettings.storage_root("projects"),
		"Cartalith projects are .ctl saves (.zip still opens)", func(path: String):
			_pick_path(path))

func _on_files_dropped(files: PackedStringArray) -> void:
	if not visible:
		return
	for f in files:
		if String(f).get_extension().to_lower() in PROJECT_EXTENSIONS:
			_pick_path(String(f))
			return
	## **Names both, because both open.** A message saying ".ctl" alone would
	## be a false rejection reason for someone dropping a pre-2026-09-07 save,
	## which is exactly the "dash a field with a reason" trap in `MISTAKES.md`:
	## the reason is a claim and gets verified like any other.
	_say("that is not a .ctl or .zip save")

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

# ---------------------------------------------------------------------------
# Tile art -- the world's own coastline, and the identicon behind it
# ---------------------------------------------------------------------------

## The tile art's pixel size. Deliberately the same 96x72 `identicon()` has
## always produced, so swapping a render in behind the fallback moves **no**
## laid-out size: both call sites hand the texture to a `TextureRect` with
## `EXPAND_IGNORE_SIZE` + `STRETCH_SCALE`, which scales whatever it is given to
## the slot the tile gives it. That also makes the source aspect a non-question
## -- the slot's aspect is what shows either way -- so the whole map is sampled
## into the tile rather than letterboxed, which is mildly anamorphic for a
## non-4:3 world and is what fills the tile the way the mockup draws it filled.
const THUMB_W := 96
const THUMB_H := 72

## `cartalith_terrain::tile_render::SEA` (reference 8330), deepest first,
## carried over as 0-1 `Color` so `set_pixel` and `Color.lerp()` can be used
## directly. The 0-255 source numbers are left visible in the expression so
## this reads against the Rust constant rather than against a rounded copy.
const HYPSO_SEA: Array[Color] = [
	Color(10 / 255.0, 28 / 255.0, 46 / 255.0),
	Color(26 / 255.0, 86 / 255.0, 140 / 255.0),
	Color(70 / 255.0, 140 / 255.0, 196 / 255.0),
]

## `cartalith_terrain::tile_render::LAND` (reference 8331), split into its two
## halves because GDScript has no tuple: `HYPSO_LAND_AT[i]` is the stop's
## normalised height and `HYPSO_LAND_RGB[i]` its colour.
const HYPSO_LAND_AT: Array[float] = [0.0, 0.18, 0.38, 0.58, 0.78, 1.0]
const HYPSO_LAND_RGB: Array[Color] = [
	Color(47 / 255.0, 122 / 255.0, 68 / 255.0),
	Color(111 / 255.0, 154 / 255.0, 58 / 255.0),
	Color(201 / 255.0, 178 / 255.0, 74 / 255.0),
	Color(150 / 255.0, 112 / 255.0, 72 / 255.0),
	Color(140 / 255.0, 140 / 255.0, 140 / 255.0),
	Color(248 / 255.0, 248 / 255.0, 250 / 255.0),
]

## Where a rendered tile is kept between sessions. See `thumbnail()`.
const THUMB_CACHE_DIR := "user://thumbs"

## `"<path-hash>-<mtime>"` -> the texture for it, for the length of the
## session. Same shape and same reasoning as `_meta_cache` above: the gallery
## rebuilds every tile on every keystroke in the search well, and re-reading a
## world's heightmap per keystroke is not something a memory of one texture per
## world should be traded for.
static var _thumb_cache: Dictionary = {}

## The tile's art: a render of the world the save actually contains, falling
## back to `identicon()` for any save that cannot supply one.
##
## **Why this is buildable without touching the save format.** Every conforming
## archive already carries its own coastline: `SAVEFILE_COMPAT.md` §8 makes
## `rasters/heightmap.f32` a MUST and §7 makes `world.grid_width`,
## `world.grid_height` and `world.sea_level` MUSTs, all four refuse-if-absent.
## Checked against a real save on 2026-09-07, not just against the document.
##
## **Why it is not `build_color_texture()`.** That is the map renderer, and it
## needs the whole engine state loaded -- a full project load per tile, for a
## gallery of them. This takes the cheap path the LOD tiler already takes:
## `render_height_tile_rgba`'s hypsometric ramp (`_hypso()` below), which is
## the reference's own *Relief* colouring, run against the save's own
## `sea_level`. Land/sea and the elevation ramp are what make a tile read as
## *this* world; the material palettes are not, and they are what costs a load.
##
## **Two levels of cache, because the raw read is the expensive part.**
## Inflating `rasters/heightmap.f32` costs `GW x GH x 4` bytes -- ~10 MB for a
## 2048x1311 world -- to draw 96x72 pixels, and doing that per tile per gallery
## open is the thing that would make this a regression instead of a feature.
## `_thumb_cache` covers the session; `user://thumbs` covers the restart. Both
## are keyed on the path **and its mtime**, which is the same invalidation
## `project_meta()` uses and which the tile already pays for -- `_relative_time`
## reads the identical `FileAccess.get_modified_time()` for its "edited 4 min
## ago" caption -- so a re-saved world re-renders and nothing else does.
static func thumbnail(path: String) -> Texture2D:
	var key := "%s-%d" % [path.sha256_text().substr(0, 16), FileAccess.get_modified_time(path)]
	if _thumb_cache.has(key):
		return _thumb_cache[key]
	var tex: Texture2D = null
	var file := THUMB_CACHE_DIR.path_join(key + ".png")
	## `FileAccess.file_exists()` first, and not as a micro-optimisation:
	## `Image.load_from_file()` on an absent path pushes two engine errors of
	## its own before returning `null`, so the miss path -- which is every tile
	## of the first gallery open -- would otherwise write 30 stack traces into
	## `user://logs/godot.log`, the file `diagnostic_report.gd` collects.
	var cached := Image.load_from_file(file) if FileAccess.file_exists(file) else null
	if cached != null and cached.get_width() == THUMB_W and cached.get_height() == THUMB_H:
		tex = ImageTexture.create_from_image(cached)
	else:
		var img := _render_thumbnail(path)
		if img != null:
			tex = ImageTexture.create_from_image(img)
			_store_thumbnail(key, img)
	## The fallback, and it is cached under the same key on purpose: a damaged
	## or foreign archive must not be re-opened and re-rejected on every
	## keystroke either. A **re-save** changes the mtime, so the key changes and
	## a world that was mid-write when it was first seen gets another chance.
	if tex == null:
		tex = identicon(path)
	_thumb_cache[key] = tex
	return tex

## The render itself, or `null` for anything that is not a world this can draw.
##
## Both archive layouts are read, because the gallery lists both: the tree
## layout `project.json` names (`SAVEFILE_COMPAT.md` §7/§8) and the **flat**
## layout of §15, whose grid lives in `params.json` as `GW`/`GH`/
## `state.seaLevel` with the raster at the archive root. §15's own mapping
## table is where those four names come from.
##
## Every refusal below is a refusal `SAVEFILE_COMPAT.md` already states -- an
## unreadable zip, a missing or unparsable manifest, a non-positive grid (§7:
## *"Refuse. Zero or negative is refused too"*), an absent heightmap, or one
## whose length is not `GW x GH x 4` (§8.1: *"the reader MUST refuse the
## archive"*). None of them is an error here, because a picker that throws
## instead of drawing a tile is worse than one drawing a gradient.
static func _render_thumbnail(path: String) -> Image:
	var zip := ZIPReader.new()
	if zip.open(path) != OK:
		return null
	var gw := 0
	var gh := 0
	## `NAN`, not `0.0`. A `sea_level` of 0.0 is a **conforming** value (§7:
	## *"number in [0,1]"*) that paints an all-land tile, so defaulting to it
	## would answer an archive that never said where its coastline is with a
	## confident, wrong coastline -- `MISTAKES.md`: never encode "no value" as a
	## plausible value. §7's own table says `sea_level`: *"Refuse. It is the
	## coastline. A default would silently redraw it."* `NAN` is the one float
	## that fails the range test below, so the refusal is the range test.
	var sea := NAN
	var entry := ""
	if zip.file_exists("project.json"):
		var head = JSON.parse_string(zip.read_file("project.json").get_string_from_utf8())
		if head is Dictionary:
			var world = (head as Dictionary).get("world", {})
			if world is Dictionary:
				gw = int((world as Dictionary).get("grid_width", 0))
				gh = int((world as Dictionary).get("grid_height", 0))
				sea = float((world as Dictionary).get("sea_level", NAN))
				entry = "rasters/heightmap.f32"
	elif zip.file_exists("params.json"):
		var flat = JSON.parse_string(zip.read_file("params.json").get_string_from_utf8())
		if flat is Dictionary:
			gw = int((flat as Dictionary).get("GW", 0))
			gh = int((flat as Dictionary).get("GH", 0))
			var state = (flat as Dictionary).get("state", {})
			if state is Dictionary:
				sea = float((state as Dictionary).get("seaLevel", NAN))
			entry = "heightmap.f32"
	## `not (sea >= 0.0 and sea <= 1.0)`, written that way round on purpose:
	## NAN fails both comparisons, so the absent case and an out-of-range case
	## are refused by the same clause.
	if gw < 1 or gh < 1 or entry == "" or not (sea >= 0.0 and sea <= 1.0) \
			or not zip.file_exists(entry):
		zip.close()
		return null
	var bytes := zip.read_file(entry)
	zip.close()
	if bytes.size() != gw * gh * 4:
		return null

	## Nearest-neighbour, not an average of the cells a tile pixel covers.
	## The field being reduced is read through a threshold at `sea` -- averaging
	## a coast's cells lands the mean between the two sides of that threshold
	## and paints shoreline that is neither, which is exactly the detail the
	## owner is asking to see.
	##
	## **The two `mini()` bounds never bind, and are kept anyway.** The largest
	## index either axis produces is `int((THUMB_H - 1) * gh / THUMB_H)`, which
	## is below `gh` for every `gh >= 1` -- so raising the ceiling to `gh` is a
	## mutation nothing can detect, and the mutation pass reports it as the one
	## survivor of 45 rather than pretending otherwise. Lowering it to `gh - 2`
	## *is* caught, by a world exactly 96x72 where the index does reach the
	## bound. They stay because what they guard is an index into a raw byte
	## buffer: `decode_float()` past the end returns a silent `0.0`, which would
	## paint a wrong pixel rather than fail, and the cost of the guard is two
	## tokens on a loop that runs 6 912 times per world, once.
	var img := Image.create(THUMB_W, THUMB_H, false, Image.FORMAT_RGB8)
	for ty in THUMB_H:
		var sy: int = mini(int(float(ty) * gh / THUMB_H), gh - 1)
		var row: int = sy * gw
		for tx in THUMB_W:
			var sx: int = mini(int(float(tx) * gw / THUMB_W), gw - 1)
			img.set_pixel(tx, ty, _hypso(bytes.decode_float((row + sx) * 4), sea))
	return img

## `cartalith_terrain::tile_render::hypso` (reference 8332): a normalised
## height to its map colour. Below `sea`, a two-segment ramp through
## `HYPSO_SEA` driven by relative depth; above it, the `HYPSO_LAND_*` stops
## interpolated on height renormalised into `[0,1]`.
##
## The three guards are the reference's, kept rather than tidied: `sea <= 0`
## reads the shallowest sea colour, `1 - sea <= 0` reads the lowest land
## colour, and a zero-width land interval divides by 1 instead of by 0.
##
## **Which of them a conforming save can actually reach, measured rather than
## assumed** -- the first version of this comment claimed all three and a probe
## falsified it. §7 allows `sea_level` anywhere in `[0,1]`, so `1 - sea <= 0`
## is reachable: at `sea_level` 1.0 a cell of exactly 1.0 takes the land branch
## with a zero-width range and must read `HYPSO_LAND_RGB[0]`. `sea <= 0` is
## **not** reachable from a conforming raster, because it is only consulted on
## the `v < sea` branch and §8 makes the heightmap `[0,1]`; it is kept because
## that range is a MUST a length-correct but corrupt raster can still break,
## and because dropping a reference guard is not this port's call to make.
##
## **The hillshade half of `render_height_tile_rgba` is deliberately not here,
## and that is a decision rather than an omission.** Its `exag` (3.4) scales a
## finite difference between **adjacent cells**; this samples every `GW/96`th
## cell, so the same difference is taken across ~21 cells on a 2048-wide world
## and the shade would come out over-driven by that stride -- differently per
## world, since the stride is the world's own width. Correcting it would mean
## dividing `exag` by a stride nothing in the reference divides it by. The ramp
## alone is what carries the coastline and the elevation banding at 96x72;
## relief detail at that size is sub-pixel.
##
## Clamped on the way out because the depth ramp is deliberately unclamped
## upstream -- `hypso_extrapolates_below_the_palette_rather_than_clamping` in
## `tile_render.rs` pins that a deep enough `v` drives it past `SEA[0]` into
## negative channels, and it is the clamped **store** that makes it harmless.
static func _hypso(v: float, sea: float) -> Color:
	if v < sea:
		var d := 0.0 if sea <= 0.0 else (sea - v) / sea
		var c := HYPSO_SEA[2].lerp(HYPSO_SEA[1], d / 0.5) if d < 0.5 \
			else HYPSO_SEA[1].lerp(HYPSO_SEA[0], (d - 0.5) / 0.5)
		return c.clamp()
	var r := 0.0 if (1.0 - sea) <= 0.0 else (v - sea) / (1.0 - sea)
	for i in HYPSO_LAND_AT.size() - 1:
		if r <= HYPSO_LAND_AT[i + 1]:
			var span: float = HYPSO_LAND_AT[i + 1] - HYPSO_LAND_AT[i]
			var t: float = (r - HYPSO_LAND_AT[i]) / (1.0 if span == 0.0 else span)
			return HYPSO_LAND_RGB[i].lerp(HYPSO_LAND_RGB[i + 1], t).clamp()
	return HYPSO_LAND_RGB[HYPSO_LAND_RGB.size() - 1]

## Write one rendered tile to `user://thumbs`, and drop the same world's older
## ones while there.
##
## The key's first half is the path's SHA-256 prefix rather than
## `String.hash()`: a 32-bit hash collision between two worlds would put one
## world's map on the other's tile, which is the quiet kind of wrong this
## project has paid for before, and 64 bits of digest costs the same nothing.
## The second half is the mtime, so a re-save writes a **new** file rather than
## overwriting the old one -- hence the prune, which is what keeps the cache
## bounded by the number of worlds rather than by the number of saves.
##
## Every failure here is ignored on purpose. A read-only or full `user://` must
## cost the next open its render time, not its tile.
static func _store_thumbnail(key: String, img: Image) -> void:
	DirAccess.make_dir_recursive_absolute(THUMB_CACHE_DIR)
	if img.save_png(THUMB_CACHE_DIR.path_join(key + ".png")) != OK:
		return
	var prefix := key.split("-")[0] + "-"
	for f in DirAccess.get_files_at(THUMB_CACHE_DIR):
		if String(f).begins_with(prefix) and String(f) != key + ".png":
			DirAccess.remove_absolute(THUMB_CACHE_DIR.path_join(String(f)))

## A stable, per-world radial wash. Hue from the path's hash so the same world
## always reads the same colour and two worlds rarely collide; saturation and
## value are fixed low, because these tiles sit behind an accent selection
## border and must never compete with it.
##
## **Since 2026-09-07 this is the fallback, not the tile art.** `thumbnail()`
## above draws the world's own coastline and calls this only when the archive
## cannot supply one. It is kept, rather than deleted, because that case is
## real: §6.4a's damage ladder, a file caught mid-write, and any foreign `.zip`
## the projects folder happens to contain all still get a tile.
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

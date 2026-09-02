extends AcceptDialog
class_name PhoneProjectPicker

## The Android phone's entry screen. The owner's locked `docs/ANDROID_UI_SPEC
## .md` (`design/android-2026-08-30/README.md` points at it; the file itself
## lives in the owner's Claude Design project, not this repo) states its first
## decision verbatim: *"Entry: project picker (recent worlds grid: Eldra + 2
## others, New world, Open .zip)."* Built from the `scrPicker` screen in
## `design/android-2026-08-30/Cartalith Android.dc.html` (lines 34-60): a
## header reading `worlds on this device · ~/Cartalith/Worlds`, a vertical
## stack of world cards, a `+ NEW WORLD` action, an `OPEN PROJECT .ZIP…`
## action and a build-line footer.
##
## **Phone only, by construction, not by a runtime check.** `app.gd` builds
## this node at all only inside `if is_phone():` -- `dcc_shell.gd`'s own
## public accessor for its `_phone = _touch and short/long < 0.6` aspect test
## (`dcc_shell.gd:1278`, `is_phone()`) -- so a desktop or tablet session never
## has one in its tree to show, hide or race against. `open()` also refuses to
## present on anything `is_phone()` denies, as a second, cheap guard against a
## future call site reaching it by mistake. `_projpicker_probe.gd` asserts
## both directions: `find_child("PhoneProjectPicker", ...)` comes back null on
## a 1600x900 boot and non-null, visible, with every action button at or above
## `DccTheme.PHONE_TAP_MIN`, on a 1080x2340 one.
##
## **Why a second dialog and not `OpenProjectDialog.open_welcome()`.**
## `open_project_dialog.gd`'s welcome mode already reaches every world on disk
## and already takes the phone treatment (`_phone = DccWidgets.phone_window
## (self, host)`, that file's own `setup()`) -- but what it takes the phone
## treatment *of* is the desktop "Open project dialog 1920" gallery: a search
## well, `Recent` / `All worlds` / `Shared` scope chips, a 4-column tile grid,
## scaled and stacked to fit. The locked spec draws a different, phone-native
## composition -- one column of full-width cards, no search, no scopes -- and
## `DCC_SHELL_SCOPE.md`'s standing rule (`CLAUDE.md`'s own "Working rules"
## section) is that the newer canvas wins where the two disagree. This file is
## that screen. `OpenProjectDialog` keeps `File ▸ Open project…` on desktop
## and tablet, completely unchanged -- see `app.gd::open_project_picker()`
## for the one-line branch that sends a phone session here instead.
##
## **Recents are the real store, not the mockup's `Eldra + 2 others`.** Those
## three rows are `Cartalith Android.dc.html`'s own hard-coded `pickerWorlds`
## array (its own `renderVals()`), placeholder content for an interactive
## prototype, complete with a `state` chip -- FINALIZED / IN PROGRESS / DRAFT
## -- this port has no per-save concept of and cannot honestly compute.
## `DccSettings.recent_projects()` is the real list: the same one `Data ▸
## Recent worlds` (`menus.gd`'s `_recent_popup`) and `open_project_dialog.gd`'s
## own `Recent` scope both already read, written once per successful load by
## `DccSettings.remember_project()` (`app.gd::_load_project`). `_refresh()`
## below filters it to paths still on disk, exactly like `open_project_dialog
## .gd::_paths()`'s `"recent"` branch, and shows an honest "no saved worlds
## yet" row rather than three worlds nobody generated. Per-card seed and
## edited-time are `OpenProjectDialog.project_meta()` / `.identicon()`, made
## public on that file rather than re-read here a second way -- the same real
## `params.json` seed and file mtime the desktop gallery's own tiles show. The
## mockup's per-world state chip and its `2 048² · 1.6 GB` figures are
## deliberately not reproduced: nothing this port tracks answers "is this
## world finalized" for a save that is not the one currently open, and
## inventing an answer would be exactly the fabricated row this task's own
## brief says not to build.
##
## **New world and Open .zip call the shell's existing flows.** `+ New world`
## calls `DccApp.open_new_world()`, which already phone-presents
## `new_world_dialog.gd` (PH-06, that function's own doc comment in `app.gd`)
## -- nothing here reimplements the setup form. `Open project .zip…` calls the
## same `DccBrowseDialog.choose_file()` that `open_project_dialog.gd
## ::_browse_from_disk()` calls: a custom in-shell `DirAccess` browser rooted
## at `DccSettings.storage_root("projects")` (under `OS.get_user_data_dir()`,
## the app's own private/scoped storage on Android). Grepped for
## `FileDialog`/`ACCESS_FILESYSTEM` across `godot-project/shell/` before
## writing this: **nothing in this shell uses Godot's stock `FileDialog`
## anywhere.** `browse_dialog.gd`'s own header explains why -- the OS tree
## picker was replaced project-wide by this custom browser -- so there is no
## native Android file-picker behaviour for this screen to branch on; it
## inherits the same in-app-storage browsing every other "open a file" call
## site in this shell already uses, unchanged.
##
## **Dismissal.** Shown once at cold start by `app.gd::_ready()` when
## `is_phone() and not bridge.has_world` (`open_welcome()`'s phone branch),
## exactly parallel to the existing desktop/tablet path through
## `open_project_dialog.open_welcome()`. `setup()` below connects
## `bridge.world_loaded` and `bridge.generation_finished(ok)` -- the same two
## signals `app.gd::_wire_status()` already uses to know a world exists -- to
## hide this window the instant either fires with success, and nothing here
## re-opens it on its own. The way back is the existing entry point, not a
## second one: `DccApp.open_project_picker()` (`menus.gd`'s File ▸ Open
## project…, `data_manager_window.gd`'s Import ▸ World Data footer) now
## branches on `is_phone()` before choosing which of the two dialogs to open.

var _host: DccApp
var _list: VBoxContainer
var _phone := false

func setup(host: DccApp) -> void:
	_host = host
	## Explicit rather than the engine's default class-derived name, so a
	## probe (or anything else) can find this one node by name without
	## reaching into `DccApp`'s own field -- `_projpicker_probe.gd` checks
	## both and expects them to agree.
	name = "PhoneProjectPicker"
	title = "Cartalith"
	get_ok_button().hide()   ## this screen's own actions replace it, exactly
		## like every other full-screen phone dialog in this shell.
	_phone = DccWidgets.phone_window(self, host)
	_build()
	if _phone:
		host.phone_fit(self, 1.0)
	## §"Dismissal" above. `if visible` on both so a signal that fires while
	## some *other* screen caused the world change (a menu action taken from
	## the map, long after this dialog last closed) does not re-show it.
	host.bridge.world_loaded.connect(func(): if visible: hide())
	host.bridge.generation_finished.connect(func(ok: bool): if ok and visible: hide())

func open() -> void:
	## The cheap second guard this file's header describes -- `app.gd` never
	## calls this except behind its own `is_phone()` check, but a dialog that
	## silently no-ops on a bad call is safer than one that trusts every
	## caller forever.
	if _host == null or not _host.is_phone():
		return
	_refresh()
	if not DccWidgets.phone_present(self, _host):
		popup_centered()

# ---------------------------------------------------------------------------
# Layout
# ---------------------------------------------------------------------------

func _build() -> void:
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)

	## `DccWidgets.phone_head()` is the 44 dp keep-clear + 56 dp app-bar header
	## every other full-screen phone dialog in this shell already draws its
	## title in (`gen_info_dialog.gd`, `world_data_window.gd`, …) -- reused
	## rather than a fifth hand-rolled header.
	DccWidgets.phone_head(outer, "Cartalith",
		"worlds on this device · %s" % _short_root())

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var pad := DccWidgets.pad(scroll, 14, 14, 14, 20)
	## **`EXPAND`, not just `FILL`, and this is the whole reason the picker
	## drew its column at a third of the screen.**
	##
	## A `ScrollContainer` lays a child out at the child's MINIMUM size along
	## an axis unless that child asks to expand -- `SIZE_FILL` alone is not
	## enough there, which is the opposite of how it behaves inside a plain
	## `BoxContainer` and is why this looked correct in every reading of the
	## code. `_list` already carried `EXPAND_FILL`; the `MarginContainer`
	## between it and the scroll did not, so the expansion had nothing to
	## expand inside.
	##
	## Measured by `_pickerwidth_probe.gd` on the real tree rather than
	## reasoned about: `ScrollContainer` 736 wide, its `MarginContainer` child
	## **220**, the `VBoxContainer` under that 192. Found on the OnePlus 6T,
	## where the screenshot showed the actions ending at about half the screen
	## with the window itself full width.
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 10)
	_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_list)
	outer.add_child(scroll)

## The projects root, short enough to be a header subtitle.
##
## `DccSettings.storage_root("projects")` is an absolute path, and on Android it
## is the app-private one -- `/data/data/org.cartalith.walkingskeleton/files/…`
## on the 6T, which is 50-odd characters of scaffolding around one useful word
## and which overran the right edge of the screen. The prototype's own header
## reads `worlds on this device · ~/Cartalith/Worlds`: a short, recognisable
## tail, not a filesystem address.
##
## So: the last two segments, which is `Cartalith/Worlds` on desktop and
## `files/Worlds` on Android, prefixed to show it is a tail. The full path is
## still one tap away and unabbreviated in `File ▸ Storage locations`, which is
## the row that exists to answer "where exactly".
func _short_root() -> String:
	var root := DccSettings.storage_root("projects")
	var parts := root.replace("\\", "/").split("/", false)
	if parts.size() <= 2:
		return root
	return "…/%s/%s" % [parts[parts.size() - 2], parts[parts.size() - 1]]

func _refresh() -> void:
	for c in _list.get_children():
		_list.remove_child(c)
		c.queue_free()

	var shown := 0
	for p in DccSettings.recent_projects():
		if FileAccess.file_exists(String(p)):
			_list.add_child(_build_world_row(String(p)))
			shown += 1
	if shown == 0:
		## The genuine first-launch state -- not a fake `Eldra + 2 others`.
		## `DccSettings.recent_projects()` is real and simply empty until a
		## world is created or opened once.
		DccWidgets.note(_list, "no saved worlds yet — create one below")

	DccWidgets.action(_list, "+ New world", _on_new_world, true)
	DccWidgets.action(_list, "Open project .zip…", _on_open_zip, false)

	var foot := DccTheme.mono_label(
		"Cartalith · build %s" % DccShell.build_id(), "text_ghost", DccTheme.FS_MICRO, 1)
	foot.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	foot.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_list.add_child(foot)

	## The list just changed length -- `_fit_phone_content()`'s reasoning in
	## `open_project_dialog.gd` applies here too: a `Window` only re-measures
	## its content on a resize notification, and `phone_fit()`'s own tap-floor
	## and pill conversion have to run on whatever nodes are new this pass.
	if _phone:
		_host.phone_fit(self, 1.0)

## One world card: the same stable per-world identicon and the same real
## seed/edited-time `OpenProjectDialog.project_meta()` already reads for the
## desktop gallery, in the locked spec's one-column card shape rather than
## that dialog's 4-column grid.
func _build_world_row(path: String) -> Control:
	var meta := OpenProjectDialog.project_meta(path)
	var current := _host != null and _host.current_project_path == path

	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel",
		DccTheme.outline("accent" if current else "border", "panel_alt"))
	## PH-05 (`browse_dialog.gd:434-445`'s own citation of the same fault):
	## this row is a `PanelContainer` with its own `gui_input`, which
	## `DccShell.phone_fit()` deliberately leaves `STOP` because several rows
	## in this shell must keep stopping the event they consume. Here the row
	## sits inside a vertically scrolling column, so a flick that starts on a
	## card has to still reach the `ScrollContainer` above it -- `PASS` keeps
	## delivering the press to `gui_input` below while letting an unclaimed
	## drag continue past this node, the same trade `browse_dialog.gd` makes
	## for its own live rows.
	wrap.mouse_filter = Control.MOUSE_FILTER_PASS

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	wrap.add_child(col)

	var thumb := TextureRect.new()
	thumb.texture = OpenProjectDialog.identicon(path)
	thumb.custom_minimum_size.y = 92
	thumb.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	thumb.stretch_mode = TextureRect.STRETCH_SCALE
	thumb.mouse_filter = Control.MOUSE_FILTER_IGNORE
	col.add_child(thumb)

	var cap := DccWidgets.pad(col, 14, 10, 14, 10)
	var cap_row := HBoxContainer.new()
	cap_row.add_theme_constant_override("separation", 10)
	cap.add_child(cap_row)

	var name_col := VBoxContainer.new()
	name_col.add_theme_constant_override("separation", 3)
	name_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cap_row.add_child(name_col)
	var name_label := DccTheme.mono_label(
		path.get_file().get_basename(), "text_bright" if current else "text", DccTheme.FS_SMALL, 2, true)
	name_label.clip_text = true
	name_col.add_child(name_label)
	name_col.add_child(DccTheme.mono_label(
		"%s · %s" % [meta.get("seed", "seed unread"), meta.get("edited", "")],
		"text_faint", DccTheme.FS_TINY))

	cap_row.add_child(DccTheme.mono_label(
		DccIcons.SYMBOLS["expand"], "text_ghost", DccTheme.FS_BODY))

	_ignore_mouse(wrap)
	wrap.gui_input.connect(func(event: InputEvent):
		if not (event is InputEventMouseButton):
			return
		var mb := event as InputEventMouseButton
		if mb.pressed and mb.button_index == MOUSE_BUTTON_LEFT:
			_pick_world(path))
	return wrap

func _pick_world(path: String) -> void:
	hide()
	_host.open_recent_project(path)

# ---------------------------------------------------------------------------
# Actions
# ---------------------------------------------------------------------------

## Hides first, matching `open_project_dialog.gd`'s own welcome-mode "Create a
## new world" tile (`hide(); _host.open_new_world()`) -- the new-world sheet
## phone-presents itself as its own full-screen window
## (`DccWidgets.phone_present(new_world_dialog, self)`, `app.gd
## ::open_new_world()`), and leaving this one visible underneath a second
## full-screen window it can never be seen through again is pointless state
## to keep live. If the sheet is cancelled the shell is left exactly as
## `open_project_dialog.gd`'s own header describes for its equivalent tile --
## not a gate, and `File ▸ Open project…` (routed to this dialog on phone,
## `app.gd::open_project_picker()`) is the way back.
func _on_new_world() -> void:
	hide()
	_host.open_new_world()

## Deliberately does NOT hide first -- mirrors `open_project_dialog.gd
## ::_browse_from_disk()` exactly, whose own callback hides only once a path
## is actually chosen. `DccBrowseDialog` phone-presents itself as its own
## full-screen window on top of this one; cancelling it should return to this
## picker, not to the empty map behind it.
func _on_open_zip() -> void:
	DccBrowseDialog.choose_file(_host, "Open project — browse", PackedStringArray(["zip"]),
		DccSettings.storage_root("projects"),
		"Cartalith projects are .zip saves", func(path: String):
			hide()
			_host.open_recent_project(path))

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

## Same duplicated-by-design micro-helper `open_project_dialog.gd` and
## `browse_dialog.gd` each already carry their own copy of (their own doc
## comments on it are identical to this one): Godot's default `mouse_filter`
## is `STOP` on every `Control`, containers included, so an unattended
## `VBoxContainer`/`HBoxContainer` inside a clickable wrapper silently
## swallows the click the wrapper is listening for.
static func _ignore_mouse(node: Node) -> void:
	for child in node.get_children():
		if child is Control:
			(child as Control).mouse_filter = Control.MOUSE_FILTER_IGNORE
		_ignore_mouse(child)

extends AcceptDialog
class_name DccBrowseDialog

## The in-shell replacement for Godot's stock `FileDialog`, built from the
## "Select folder dialog 1920" screen in `design/Cartalith DCC Shell.dc.html`.
##
## The mockup's own comment states the intent: *"open breadcrumb browser,
## replaces the stock OS tree picker"*. Everything it draws is here, in the
## order it draws it:
##
## | mockup element | here |
## |---|---|
## | modal 760 x 640, `#121314` on a dimmed shell | `size` / `min_size` |
## | title `Select markdown vault folder` + `✕` | `_build_head()` |
## | breadcrumb `Home › Documents › Cartalith`, last segment accent, `⌂ Home` at the right | `_build_breadcrumb()` |
## | a typeable current-folder path well | `_path_edit` |
## | flat, generous rows: `▸ name` + a right-hand `14 items` meta | `_build_row()` |
## | the selected row outlined accent, washed, with a `selected` tag | same |
## | files `dimmed, not selectable` | `_build_row()`'s `live` flag |
## | a dashed `＋ New folder…` row | `_build_new_folder_row()` |
## | foot: a hint, `Cancel`, `Use this folder` | `_build_foot()` |
##
## **Two modes, one screen.** The mockup only draws folder-picking, but the
## shell's other stock-`FileDialog` call sites are *file* picks (a project
## `.zip`, an asset pack `.zip`). Rather than leave a second generic OS dialog
## in a shell whose whole premise is that it draws its own chrome, this dialog
## takes a `PickKind` (not `Mode` -- `Window` already owns that name, and a
## clashing enum shadows it): `FOLDERS` is the mockup exactly, `FILES` keeps every pixel
## and flips which rows are live -- matching files become selectable and
## non-matching ones take the dimming the mockup gives *all* files. The
## breadcrumb, the path well, the row rhythm and the foot are identical
## because the design draws one browser, not two.
##
## **No logic beyond navigation lives here.** Listing, filtering by extension
## and reporting a chosen path is presentation-side plumbing; nothing here
## computes a value the engine also computes (`godot-shell` skill's own rule).

enum PickKind { FOLDERS, FILES }

## Counting `14 items` costs one directory open per row. That is nothing in a
## project folder and noticeable in `C:/Windows/System32`, so the count is
## skipped wholesale once a directory is wider than this -- the meta column
## simply goes blank rather than the dialog stalling on open. Chosen as "more
## rows than anyone scrolls", not measured.
const COUNT_ROWS_MAX := 120

var _mode: PickKind = PickKind.FOLDERS
var _extensions: PackedStringArray = PackedStringArray()  ## lower-case, no dot.
var _on_choose := Callable()
var _cwd := ""
## The row the user clicked, or "" for "no row -- the current folder itself".
## Folder mode confirms `_selected` if set and `_cwd` otherwise, which is what
## makes the mockup's "Use this folder" button read correctly whether or not a
## child row is highlighted. File mode has no such fallback: with nothing
## selected there is no file to return, so the primary button is disabled.
var _selected := ""

var _crumb_row: HBoxContainer
var _path_edit: LineEdit
var _list: VBoxContainer
var _foot_note: Label
var _primary: Button
var _rows: Dictionary = {}   ## absolute path -> PanelContainer

# ---------------------------------------------------------------------------
# Entry points
# ---------------------------------------------------------------------------

## Pick a folder. `on_choose` is called with one absolute path, exactly like
## `FileDialog.dir_selected` did, so a caller swaps constructors and nothing
## else. The dialog frees itself on close.
static func choose_folder(host: Node, dialog_title: String, start_dir: String,
		footnote: String, on_choose: Callable) -> DccBrowseDialog:
	return _spawn(host, dialog_title, PickKind.FOLDERS, PackedStringArray(),
		start_dir, footnote, on_choose)

## Pick a file whose extension is in `extensions` (`["zip"]`, no dot).
## `on_choose` matches `FileDialog.file_selected`.
static func choose_file(host: Node, dialog_title: String, extensions: PackedStringArray,
		start_dir: String, footnote: String, on_choose: Callable) -> DccBrowseDialog:
	return _spawn(host, dialog_title, PickKind.FILES, extensions, start_dir,
		footnote, on_choose)

static func _spawn(host: Node, dialog_title: String, mode: PickKind,
		extensions: PackedStringArray, start_dir: String, footnote: String,
		on_choose: Callable) -> DccBrowseDialog:
	var d := DccBrowseDialog.new()
	host.add_child(d)
	d.setup(dialog_title, mode, extensions, footnote, on_choose)
	## One dialog per invocation, freed when it closes -- the same lifetime the
	## `FileDialog` it replaces had. Windows this shell keeps alive (the asset
	## library, the data manager) are long-lived because they hold state worth
	## keeping; a browser holds a directory, and re-listing is instant.
	d.visibility_changed.connect(func():
		if not d.visible:
			d.queue_free())
	d.navigate(start_dir)
	d.popup_centered()
	return d

func setup(dialog_title: String, mode: PickKind, extensions: PackedStringArray,
		footnote: String, on_choose: Callable) -> void:
	_mode = mode
	_on_choose = on_choose
	for e in extensions:
		_extensions.append(String(e).to_lower().trim_prefix("."))
	title = dialog_title
	get_ok_button().hide()   ## the mockup's own foot row replaces it.
	size = Vector2i(760, 640)
	min_size = Vector2i(560, 460)
	_build(dialog_title, footnote)

# ---------------------------------------------------------------------------
# Layout
# ---------------------------------------------------------------------------

func _build(dialog_title: String, footnote: String) -> void:
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)

	outer.add_child(_build_head(dialog_title))
	outer.add_child(DccTheme.rule())
	outer.add_child(_build_breadcrumb())
	outer.add_child(_build_path_well())

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 28)
	pad.add_theme_constant_override("margin_top", 16)
	pad.add_theme_constant_override("margin_right", 28)
	pad.add_theme_constant_override("margin_bottom", 8)
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(pad)
	_list = VBoxContainer.new()
	## The mockup's rows sit 2 px apart -- close enough to read as one list,
	## far enough that an outlined selected row never touches its neighbour.
	_list.add_theme_constant_override("separation", 2)
	_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_list)
	outer.add_child(scroll)

	outer.add_child(DccTheme.rule())
	outer.add_child(_build_foot(footnote))

func _build_head(dialog_title: String) -> Control:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 14)
	row.add_child(DccTheme.label(dialog_title, "text_bright", DccTheme.FS_MODAL_TITLE))
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
	return _pad(row, 28, 22, 28, 16)

## `Home › Documents › Cartalith`, every segment a jump target, the last one
## accent. A breadcrumb is also the whole of this dialog's "go up": the parent
## of the current folder is always one of the segments, so there is no `..`
## row and none of the nested-tree disclosure the mockup's comment rejects.
func _build_breadcrumb() -> Control:
	_crumb_row = HBoxContainer.new()
	_crumb_row.add_theme_constant_override("separation", 6)
	return _pad(_crumb_row, 28, 14, 28, 0)

func _build_path_well() -> Control:
	_path_edit = LineEdit.new()
	_path_edit.add_theme_font_override("font", DccTheme.mono())
	_path_edit.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	_path_edit.add_theme_color_override("font_color", DccTheme.c("text"))
	_path_edit.add_theme_stylebox_override("normal", _well())
	_path_edit.add_theme_stylebox_override("focus", DccTheme.outline("accent"))
	_path_edit.add_theme_stylebox_override("read_only", _well())
	## "typeable, not required" (the mockup's own comment): typing a directory
	## navigates there, typing a matching file in file mode picks it, and
	## anything else leaves the well alone rather than erroring at the user.
	_path_edit.text_submitted.connect(_on_path_submitted)
	return _pad(_path_edit, 28, 12, 28, 0)

func _build_foot(footnote: String) -> Control:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	_foot_note = DccTheme.mono_label(footnote, "text_ghost", DccTheme.FS_TINY)
	_foot_note.clip_text = true
	row.add_child(_foot_note)
	row.add_child(DccTheme.spacer())
	DccWidgets.modal_button(row, "Cancel", func(): hide())
	_primary = DccWidgets.modal_button(row,
		"Use this folder" if _mode == PickKind.FOLDERS else "Open", _confirm, true)
	return _pad(row, 28, 14, 28, 14)

# ---------------------------------------------------------------------------
# Navigation
# ---------------------------------------------------------------------------

## Godot exposes no home-directory call. `USERPROFILE` is the Windows
## variable, `HOME` the POSIX one; the user data dir is the fallback that
## always exists (Android in particular has neither variable).
static func home_dir() -> String:
	for key in ["USERPROFILE", "HOME"]:
		var v := OS.get_environment(key)
		if v != "" and DirAccess.dir_exists_absolute(v):
			return v.simplify_path()
	return OS.get_user_data_dir()

func navigate(dir: String) -> void:
	var target := dir.simplify_path()
	if target == "" or not DirAccess.dir_exists_absolute(target):
		target = home_dir()
	_cwd = target
	_selected = ""
	_path_edit.text = _cwd
	_refresh_crumbs()
	_refresh_list()
	_refresh_primary()

func _on_path_submitted(text: String) -> void:
	var p := text.strip_edges().simplify_path()
	if DirAccess.dir_exists_absolute(p):
		navigate(p)
	elif _mode == PickKind.FILES and FileAccess.file_exists(p) and _extension_ok(p):
		navigate(p.get_base_dir())
		_select(p)
	else:
		_path_edit.text = _cwd

func _refresh_crumbs() -> void:
	## `remove_child` before `queue_free`: freeing alone is deferred to the end
	## of the frame, so two navigations inside one frame would draw the new
	## breadcrumb after the old one instead of in place of it.
	for c in _crumb_row.get_children():
		_crumb_row.remove_child(c)
		c.queue_free()
	var parts := _cwd.split("/", false)
	var walked := ""
	## An absolute POSIX path starts with the separator the split just ate;
	## a Windows path starts with its drive, which *is* the first part.
	var posix_root := _cwd.begins_with("/")
	for i in parts.size():
		var seg := String(parts[i])
		walked = ("/" + seg) if (posix_root and i == 0) else (
			seg if walked == "" else walked.path_join(seg))
		var jump := walked + ("/" if i == 0 and not posix_root else "")
		if i > 0:
			_crumb_row.add_child(DccTheme.mono_label(
				DccIcons.SYMBOLS["expand"], "text_ghost", DccTheme.FS_SMALL))
		var last := i == parts.size() - 1
		var b := Button.new()
		b.text = seg
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.add_theme_font_override("font", DccTheme.mono())
		b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
		b.add_theme_color_override("font_color",
			DccTheme.c("accent") if last else DccTheme.c("text_dim"))
		b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
		b.add_theme_stylebox_override("normal", DccTheme.empty())
		b.add_theme_stylebox_override("hover", DccTheme.empty())
		b.add_theme_stylebox_override("pressed", DccTheme.empty())
		b.pressed.connect(func(): navigate(jump))
		_crumb_row.add_child(b)

	_crumb_row.add_child(DccTheme.spacer())
	var home := Button.new()
	home.text = "⌂ Home"
	home.flat = true
	home.focus_mode = Control.FOCUS_NONE
	home.add_theme_font_override("font", DccTheme.mono())
	home.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	home.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
	home.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	home.add_theme_stylebox_override("normal", DccTheme.empty())
	home.add_theme_stylebox_override("hover", DccTheme.empty())
	home.pressed.connect(func(): navigate(home_dir()))
	_crumb_row.add_child(home)

func _refresh_list() -> void:
	for c in _list.get_children():
		_list.remove_child(c)
		c.queue_free()
	_rows.clear()

	var da := DirAccess.open(_cwd)
	if da == null:
		_list.add_child(DccTheme.label(
			"This folder cannot be read (%s)." % error_string(DirAccess.get_open_error()),
			"text_ghost", DccTheme.FS_SMALL))
		return

	var dirs := da.get_directories()
	var files := da.get_files()
	var count_children := dirs.size() + files.size() <= COUNT_ROWS_MAX

	for entry in dirs:
		var path := _cwd.path_join(entry)
		var meta := ""
		if count_children:
			var child := DirAccess.open(path)
			if child != null:
				var n := child.get_directories().size() + child.get_files().size()
				meta = "%d item%s" % [n, "" if n == 1 else "s"]
		_list.add_child(_build_row(path, entry, true, meta))

	for entry in files:
		var path := _cwd.path_join(entry)
		var ok := _mode == PickKind.FILES and _extension_ok(path)
		var meta := _size_text(path) if ok else (
			"file" if _mode == PickKind.FOLDERS else "file · not a .%s" % ", .".join(_extensions))
		_list.add_child(_build_row(path, entry, ok, meta))

	if dirs.is_empty() and files.is_empty():
		_list.add_child(DccTheme.label("This folder is empty.", "text_ghost", DccTheme.FS_SMALL))

	_list.add_child(_build_new_folder_row())

## One list row. `live` is the mockup's own distinction between a row that can
## be chosen and one that is "dimmed, not selectable" -- a dim row still shows,
## because knowing a folder holds a `worldgen.log` is part of recognising it,
## and hiding non-matching files is what makes a stock file dialog feel like
## it is lying about what is on disk.
func _build_row(path: String, entry_name: String, live: bool, meta: String) -> Control:
	var is_dir := DirAccess.dir_exists_absolute(path)
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccTheme.empty())
	wrap.mouse_filter = Control.MOUSE_FILTER_STOP if (live or is_dir) else Control.MOUSE_FILTER_IGNORE

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 12)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_theme_constant_override("margin_top", 11)
	pad.add_theme_constant_override("margin_bottom", 11)
	pad.add_child(row)
	wrap.add_child(pad)

	var dim: String = "text" if live else "text_ghost"
	var glyph := DccTheme.mono_label(
		DccIcons.SYMBOLS["submenu"] if is_dir else "◆",
		"text_ghost" if not live else "text_dim", DccTheme.FS_BODY)
	glyph.name = "Glyph"
	row.add_child(glyph)
	var label := DccTheme.label(entry_name, dim, DccTheme.FS_BODY)
	label.name = "Name"
	label.clip_text = true
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(label)
	var meta_label := DccTheme.mono_label(meta, "text_ghost", DccTheme.FS_TINY)
	meta_label.name = "Meta"
	row.add_child(meta_label)

	_ignore_mouse(wrap)
	if live or is_dir:
		## Click selects, double-click enters (a folder) or confirms (a file) --
		## the two gestures every file browser has, and the reason the row is a
		## `PanelContainer` with a `gui_input` handler rather than a `Button`:
		## `Button.pressed` cannot tell them apart.
		wrap.gui_input.connect(func(event: InputEvent):
			if not (event is InputEventMouseButton):
				return
			var mb := event as InputEventMouseButton
			if not (mb.pressed and mb.button_index == MOUSE_BUTTON_LEFT):
				return
			if mb.double_click:
				if is_dir:
					navigate(path)
				elif live:
					_select(path)
					_confirm()
			elif live:
				_select(path))
		wrap.mouse_entered.connect(func():
			if _selected != path:
				wrap.add_theme_stylebox_override("panel", DccTheme.flat(DccTheme.c("line_soft"))))
		wrap.mouse_exited.connect(func():
			if _selected != path:
				wrap.add_theme_stylebox_override("panel", DccTheme.empty()))

	_rows[path] = wrap
	return wrap

## The mockup's dashed `＋ New folder…`. Godot's `StyleBoxFlat` has no dash
## pattern and drawing one would mean a custom `_draw` for a single row, so
## this takes the closest honest approximation: a full hairline at the same
## ghost weight the dash reads as. Clicking swaps the row for an inline field
## rather than stacking a second modal on a modal.
func _build_new_folder_row() -> Control:
	var wrap := PanelContainer.new()
	wrap.add_theme_stylebox_override("panel", DccTheme.outline("line"))
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_theme_constant_override("margin_top", 9)
	pad.add_theme_constant_override("margin_bottom", 9)
	wrap.add_child(pad)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 12)
	pad.add_child(row)
	row.add_child(DccTheme.mono_label(DccIcons.SYMBOLS["add"], "accent", DccTheme.FS_BODY))
	var label := DccTheme.label("New folder…", "text_dim", DccTheme.FS_SMALL)
	row.add_child(label)

	var field := LineEdit.new()
	field.visible = false
	field.placeholder_text = "folder name"
	field.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	field.add_theme_font_override("font", DccTheme.mono())
	field.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	field.add_theme_stylebox_override("normal", DccTheme.empty())
	field.add_theme_stylebox_override("focus", DccTheme.empty())
	field.text_submitted.connect(func(text: String):
		var clean := text.strip_edges()
		if clean == "":
			return
		var da := DirAccess.open(_cwd)
		if da == null or da.make_dir(clean) != OK:
			_foot_note.text = "could not create '%s' here" % clean
			return
		navigate(_cwd.path_join(clean)))
	row.add_child(field)

	## Everything in the row is inert so the row itself catches the click --
	## except the field, which has to keep its own caret once it appears.
	_ignore_mouse(wrap)
	field.mouse_filter = Control.MOUSE_FILTER_STOP
	wrap.gui_input.connect(func(event: InputEvent):
		if event is InputEventMouseButton and (event as InputEventMouseButton).pressed:
			label.visible = false
			field.visible = true
			field.grab_focus())
	return wrap

# ---------------------------------------------------------------------------
# Selection
# ---------------------------------------------------------------------------

func _select(path: String) -> void:
	var previous := _selected
	_selected = path
	if _rows.has(previous):
		_paint_row(previous, false)
	if _rows.has(path):
		_paint_row(path, true)
	if _mode == PickKind.FILES:
		_path_edit.text = path
	_refresh_primary()

## The mockup's selected row: accent hairline, an 8 % accent wash, the name in
## `text_bright` and a mono `selected` tag where the item count was.
func _paint_row(path: String, on: bool) -> void:
	var wrap: PanelContainer = _rows[path]
	wrap.add_theme_stylebox_override("panel",
		DccTheme.outline("accent", "accent_wash") if on else DccTheme.empty())
	var glyph := wrap.find_child("Glyph", true, false) as Label
	var name_label := wrap.find_child("Name", true, false) as Label
	var meta := wrap.find_child("Meta", true, false) as Label
	if glyph != null:
		glyph.add_theme_color_override("font_color",
			DccTheme.c("accent") if on else DccTheme.c("text_dim"))
	if name_label != null:
		name_label.add_theme_color_override("font_color",
			DccTheme.c("text_bright") if on else DccTheme.c("text"))
	if meta != null:
		meta.set_meta("rest_text", meta.get_meta("rest_text", meta.text))
		meta.text = "selected" if on else String(meta.get_meta("rest_text"))
		meta.add_theme_color_override("font_color",
			DccTheme.c("accent") if on else DccTheme.c("text_ghost"))

func _refresh_primary() -> void:
	## Folder mode always has an answer -- the current folder, if no child row
	## is highlighted. File mode needs a file.
	_primary.disabled = _mode == PickKind.FILES and _selected == ""

func _confirm() -> void:
	var path := _selected if _selected != "" else _cwd
	if _mode == PickKind.FILES and (_selected == "" or not FileAccess.file_exists(path)):
		return
	hide()
	if _on_choose.is_valid():
		_on_choose.call(path)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

func _extension_ok(path: String) -> bool:
	if _extensions.is_empty():
		return true
	return _extensions.has(path.get_extension().to_lower())

## Presentation only -- a row's right-hand meta column, never a value anything
## downstream reads.
static func _size_text(path: String) -> String:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		return ""
	var bytes := f.get_length()
	f.close()
	if bytes < 1024:
		return "%d B" % bytes
	if bytes < 1024 * 1024:
		return "%.0f KB" % (bytes / 1024.0)
	return "%.1f MB" % (bytes / 1048576.0)

## A whole row is one click target, so nothing inside it may eat the event.
## Godot's default `mouse_filter` is `STOP` on every `Control`, containers
## included, which means an unattended `HBoxContainer` silently swallows the
## click the row around it is listening for.
static func _ignore_mouse(node: Node) -> void:
	for child in node.get_children():
		if child is Control:
			(child as Control).mouse_filter = Control.MOUSE_FILTER_IGNORE
		_ignore_mouse(child)

func _well() -> StyleBoxFlat:
	var sb := DccTheme.outline("line")
	sb.content_margin_left = 12
	sb.content_margin_right = 12
	sb.content_margin_top = 8
	sb.content_margin_bottom = 8
	return sb

func _pad(child: Control, l: int, t: int, r: int, b: int) -> MarginContainer:
	var m := MarginContainer.new()
	m.add_theme_constant_override("margin_left", l)
	m.add_theme_constant_override("margin_top", t)
	m.add_theme_constant_override("margin_right", r)
	m.add_theme_constant_override("margin_bottom", b)
	m.add_child(child)
	return m

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

## `SAVE` is the third mode, added with `File ▸ Save as…` (`GUI_GAP_REGISTER.md`
## FI-01). It is `FILES` with one extra control -- a name field in the foot --
## because that is the entire difference between "which of these?" and "where
## shall I put this?". The rows stay live so clicking an existing save fills
## the name in, which is how every save dialog spells "overwrite that one".
enum PickKind { FOLDERS, FILES, SAVE }

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
## The viewport `_crumb_row` scrolls inside. Held because `_refresh_crumbs()`
## has to re-pin it to its tail after every rebuild -- see `_pin_crumbs()`.
var _crumb_scroll: ScrollContainer
var _path_edit: LineEdit
var _list: VBoxContainer
var _foot_note: Label
var _primary: Button
var _rows: Dictionary = {}   ## absolute path -> PanelContainer
## SAVE mode only: the foot's file-name field and the name it opens with.
var _name_edit: LineEdit
var _default_name := ""
## Owner rulings 28/29's save-time LOD-tile control -- a plain,
## non-canvas-backed checkbox the owner authorised for this one row (see
## `app.gd`'s `save_project_as()` for the citation). Empty means "this save
## flow carries no such option", which is every call site except the
## project-save one, so every other `choose_save_path()` caller draws its
## foot exactly as before. `lod_tiles_checked` is read by the caller AFTER
## `on_choose` fires (the dialog instance is still alive then; see
## `app.gd`'s forward-declared `dlg` pattern), never passed as a second
## argument to `on_choose` -- that signature is shared with every other
## save flow and changing it would ripple into all of them.
var _lod_option_text := ""
var lod_tiles_checked := false
## Phone (§13, PH-06). `_shell` is the `DccShell` this dialog's treatment is
## measured against; it is **not** always `host`, because two call sites hand
## `_spawn()` a `Window` (`open_project_dialog.gd`) rather than the shell, and
## a `Window` answers neither `is_phone()` nor `phone_scale()`.
var _phone := false
var _shell: Node = null

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
##
## **On Android this is not this dialog at all** -- owner ruling A, 2026-09-07
## (`LARGE_ITEM_RULINGS.md`): *file*-picking goes through the OS document
## picker, and *folder*-picking keeps this browser. The split is measured
## rather than stylistic, and the two mechanisms are exactly complementary
## with **zero permissions granted**:
##
##   * `DirAccess` lists shared storage's **directories** fine and cannot see
##     a **file another uid wrote**. Verified on glass 2026-09-07 before this
##     change: `Open project -- browse` landed in `/storage/emulated/0/Documents`
##     and drew `Werk` and `the Shattered Realm` and **no rows for files**,
##     with a real 3 493 626-byte `Werk.zip` sitting beside them and `Open`
##     disabled. That is the owner's original complaint, exactly.
##   * SAF sees every one of them -- and its `OPEN_DIR` returns a tree URI
##     `DirAccess` refuses with **error 31**, which is why folders did *not*
##     move and why `MANAGE_EXTERNAL_STORAGE` was offered and declined.
##
## Returns `null` on that path: there is no `DccBrowseDialog` to hand back.
## Checked repo-wide 2026-09-07 -- every non-probe `choose_file` call site
## discards the return (`app.gd` x2, `asset_library_window.gd` x2, `menus.gd`,
## `open_project_dialog.gd`, `phone_project_picker.gd`), and the probes that
## do read it run on desktop, where this branch is not taken.
static func choose_file(host: Node, dialog_title: String, extensions: PackedStringArray,
		start_dir: String, footnote: String, on_choose: Callable) -> DccBrowseDialog:
	if _saf_file_picking():
		_saf_choose_file(host, dialog_title, extensions, start_dir, on_choose)
		return null
	return _spawn(host, dialog_title, PickKind.FILES, extensions, start_dir,
		footnote, on_choose)

# ---------------------------------------------------------------------------
# Android: the OS document picker (owner ruling A)
# ---------------------------------------------------------------------------

## **Branches on the platform, deliberately -- not on the feature flag.**
## `FEATURE_NATIVE_DIALOG_FILE` is true on Windows and on Linux too, and this
## shell's whole premise is that it draws its own chrome: keying off the flag
## alone would silently replace the desktop picker with the OS one, which is
## the regression the ruling's "desktop must not regress" is about.
##
## The flag is still consulted, and only as an **availability** check on the
## side that has already been chosen: an Android build whose `DisplayServer`
## cannot raise the picker falls back to the in-shell browser rather than to
## nothing at all.
static func _saf_file_picking() -> bool:
	return OS.get_name() == "Android" \
		and DisplayServer.has_feature(DisplayServer.FEATURE_NATIVE_DIALOG_FILE)

## Raise Android's own document picker and hand the caller a path it can use.
##
## `start_dir` is passed through as the picker's opening hint **and** used as
## the copy destination -- see `_saf_materialise()` for why a copy happens at
## all. Android ignores the hint (SAF opens wherever the user last was); the
## destination half is what the argument is really doing here.
static func _saf_choose_file(host: Node, dialog_title: String,
		extensions: PackedStringArray, start_dir: String, on_choose: Callable) -> void:
	var exts := PackedStringArray()
	var globs := PackedStringArray()
	for e in extensions:
		var clean := String(e).to_lower().trim_prefix(".")
		if clean == "":
			continue
		exts.append(clean)
		globs.append("*.%s" % clean)
	var filters := PackedStringArray()
	if not globs.is_empty():
		filters.append("%s ; %s" % [", ".join(globs), ", ".join(exts).to_upper()])
	## Second, never first. Godot turns each filter into a MIME type for
	## `EXTRA_MIME_TYPES`, and a provider that reports an unexpected type for a
	## file would otherwise hide it with no way for the user to say "show me
	## anyway". The typed filter still opens selected, so the default view is
	## the narrow one.
	filters.append("*.* ; All files")
	var err := DisplayServer.file_dialog_show(dialog_title, start_dir, "", false,
		DisplayServer.FILE_DIALOG_MODE_OPEN_FILE, filters,
		func(ok: bool, picked: PackedStringArray, _filter_index: int):
			print("[saf] callback ok=%s n=%d %s" % [ok, picked.size(),
				String(picked[0]) if not picked.is_empty() else ""])
			if not ok or picked.is_empty():
				return
			var local := _saf_materialise(String(picked[0]), exts, start_dir)
			if local == "":
				_saf_say(host, "could not read the file that was picked")
				return
			if on_choose.is_valid():
				on_choose.call(local))
	if err != OK:
		## Not silence: the picker failing to raise is indistinguishable from a
		## dead button otherwise, and the in-shell browser is still a real
		## answer for anything inside the app's own storage.
		print("[saf] file_dialog_show refused (err %d) -- falling back" % err)
		_spawn(host, dialog_title, PickKind.FILES, extensions, start_dir,
			"the device picker was unavailable", on_choose)

## Copy a `content://` document to a real file and return that path.
##
## **This is the whole reason ruling A is a caller audit.** Godot's own
## `FileAccess` and `ZIPReader` read a document URI (measured: `err == 0` and
## the exact byte length), but **not one consumer on the other side of
## `on_choose` is Godot**. All four readers a `choose_file` result reaches are
## Rust, and every one of them opens with `std::fs`, which knows nothing about
## Android's content resolver:
##
## | reader | opens with |
## |---|---|
## | `project_bridge.rs::project_open` | `std::fs::File::open` |
## | `lib.rs::load_save` | `std::fs::File::open` |
## | `lib.rs::load_asset_pack` | `std::fs::read` |
## | `lib.rs::import_heightmap` | `std::fs::read` |
##
## Handing any of them a URI is the "picker whose result is dropped" the
## ruling names. Copying first is the only fix that does not reach into the
## engine crate, and it is also what a native Android app does with a SAF
## pick, so the caller-side contract is unchanged: **`on_choose` still
## receives an absolute filesystem path**, and `get_base_dir()`,
## `path_join()`, `get_file()`, `file_exists()` all keep working on it.
##
## The copy lands in `dest_dir` -- the caller's own `start_dir`, which is
## already the right home for the kind of file being picked
## (`storage_root("projects")` for a save, `storage_root("asset_packs")` for a
## pack). That is deliberate and it is the difference between *importing* the
## file and *hiding* it: the app's own gallery, recent list, Save and the
## in-shell browser all find it afterwards. **What it is NOT is a link back to
## the original** -- edits are saved to the copy, because the writers are the
## same `std::fs` list above and Android grants no filesystem write to another
## uid's file.
static func _saf_materialise(uri: String, extensions: PackedStringArray,
		dest_dir: String) -> String:
	var src := FileAccess.open(uri, FileAccess.READ)
	if src == null:
		print("[saf] FileAccess refused %s (err %d)" % [uri, FileAccess.get_open_error()])
		return ""
	var size := src.get_length()
	var dir := dest_dir
	if dir == "" or not DirAccess.dir_exists_absolute(dir):
		## `storage_root("projects")` does not exist until the first save, and
		## on a fresh install that is exactly when this runs. Creating it is
		## what the first save would have done anyway.
		if dir != "" and DirAccess.make_dir_recursive_absolute(dir) != OK:
			dir = ""
		if dir == "":
			dir = OS.get_user_data_dir()
	var name := _saf_local_name(uri, extensions)
	var dest := dir.path_join(name)
	## Never clobber, and never litter either. Re-picking the same document is
	## the common case (it is how a person reopens a project), so a name that is
	## taken by a file of the **same length** is taken as that same document and
	## reused; anything else gets a suffix. Bounded, because a `while` around a
	## filesystem predicate is how a picker hangs.
	var n := 2
	while FileAccess.file_exists(dest) and n < 100:
		var seen := FileAccess.open(dest, FileAccess.READ)
		## Length was the whole test here until the verifier measured what that
		## costs: a document with the **same name and the same byte length but
		## different content** was reused, and the caller received the OLD file.
		## Source first byte `0x5A`, delivered first byte `0x41` -- the person
		## opens a different document than the one they picked, silently, and
		## every downstream error message is then about the wrong file.
		##
		## So length is now the cheap pre-filter it should always have been, and
		## the bytes decide. `_saf_same_bytes()` only runs when the lengths
		## already match, which is the re-pick case this reuse exists for.
		if seen != null and seen.get_length() == size \
				and _saf_same_bytes(uri, dest, size):
			print("[saf] reusing %s (%d bytes)" % [dest, size])
			return dest
		var ext := name.get_extension()
		dest = dir.path_join("%s-%d%s" % [name.get_basename(), n,
			("." + ext) if ext != "" else ""])
		n += 1
	var out := FileAccess.open(dest, FileAccess.WRITE)
	if out == null:
		## Not a failure -- a wrong destination, and the distinction is the
		## whole bug. `dest_dir` is the caller's own `start_dir`, and two of the
		## seven callers pass `_picker_start_dir()` -> `home_dir()`, which on
		## Android resolves into **shared storage**. Shared storage EXISTS, so
		## the `dir_exists_absolute` test above passes it, and it is **not
		## writable at zero permissions** -- which is this whole feature's
		## premise. The open then failed and the person was told "could not read
		## the file that was picked" about a file that had read perfectly.
		##
		## Existence was the wrong question. The materialised copy is a cache of
		## a picked document, so app-private storage is always a correct home
		## for it; fall back there rather than refusing.
		var fallback := OS.get_user_data_dir().path_join(name)
		if fallback != dest:
			print("[saf] %s is not writable (err %d) -- falling back to %s" % [
				dest, FileAccess.get_open_error(), fallback])
			dest = fallback
			out = FileAccess.open(dest, FileAccess.WRITE)
	if out == null:
		print("[saf] cannot write %s (err %d)" % [dest, FileAccess.get_open_error()])
		return ""
	while not src.eof_reached():
		out.store_buffer(src.get_buffer(1 << 20))
	out.close()
	## Length, not existence. A short read on a content stream produces a file
	## that exists and is truncated, and `ZIPReader` would report that as "not
	## a Cartalith save" -- a wrong sentence about a correct file.
	var check := FileAccess.open(dest, FileAccess.READ)
	if check == null or check.get_length() != size:
		print("[saf] copy is %d of %d bytes -- refusing" % [
			0 if check == null else check.get_length(), size])
		return ""
	print("[saf] copied %s -> %s (%d bytes)" % [uri, dest, size])
	return dest

## Do a picked document and an existing local copy hold the same bytes?
##
## Called only when the two already agree on length, so the common case this
## guards -- reopening the same project -- is the only one that pays for it.
## Compares in 1 MiB chunks rather than hashing, because that uses exactly the
## two calls already proven to work on a `content://` URI (`FileAccess.open`
## plus `get_buffer`); `get_sha256()` on a document URI is untested here and a
## wrong answer from it would reintroduce the defect it exists to prevent.
##
## **Fails closed.** Anything unexpected -- either handle refusing to open, a
## short read, a length that moved between the two opens -- returns `false`, so
## the caller copies afresh. A needless copy costs disk; a wrong reuse hands
## someone the wrong document.
static func _saf_same_bytes(uri: String, dest: String, size: int) -> bool:
	var a := FileAccess.open(uri, FileAccess.READ)
	var b := FileAccess.open(dest, FileAccess.READ)
	if a == null or b == null:
		return false
	if a.get_length() != size or b.get_length() != size:
		return false
	while not a.eof_reached():
		if a.get_buffer(1 << 20) != b.get_buffer(1 << 20):
			return false
	## `b` must be spent too: a trailing tail on the local copy with an
	## identical declared length would mean one of the two lied about its size.
	return b.eof_reached()

## The local file name for a picked document.
##
## A document URI is not a path and its tail is not always a name:
## `ExternalStorageProvider` spells one document
## `content://.../document/primary%3ADocuments%2FWerk.zip`, whose decoded tail
## really is `Werk.zip`, while `DownloadsProvider` spells another
## `content://.../document/msf%3A1000000123`, whose tail is an opaque row id.
## The extension is the test: a tail that does not end in one of the
## extensions the caller asked for is not a name, and a stamped one is used
## instead of a plausible-looking `msf:1000000123`.
static func _saf_local_name(uri: String, extensions: PackedStringArray) -> String:
	var tail := uri.uri_decode().replace("\\", "/").get_file()
	if tail != "" and (extensions.is_empty()
			or extensions.has(tail.get_extension().to_lower())):
		return tail
	var ext := String(extensions[0]) if not extensions.is_empty() else "bin"
	return "imported-%d.%s" % [Time.get_unix_time_from_system(), ext]

## Say a failure where the person can see it. The phone hides the status
## region, which is why `app.gd`'s own failure reports pair `set_status` with
## a toast -- the same pairing, for the same reason.
static func _saf_say(host: Node, line: String) -> void:
	push_warning("Cartalith: " + line)
	var shell := _shell_of(host)
	if shell == null:
		return
	if shell.has_method("set_status"):
		shell.set_status("hint", line, "accent")
	if shell.has_method("is_phone") and shell.is_phone() \
			and shell.has_method("_show_phone_toast"):
		shell._show_phone_toast(line, null, 4.0)

## Choose *where to write* a new file. `default_name` fills the foot's name
## field (with `extension` appended if it carries none); `on_choose` is called
## with one absolute path, which may or may not already exist -- overwrite
## confirmation belongs to the caller, which is the only side that knows what
## is about to be overwritten.
##
## `lod_option_text`, left empty by every call site but the project-save one:
## when non-empty, the foot draws one extra checkbox with this exact text,
## unchecked by default (owner rulings 28/29 -- see `_lod_option_text`'s own
## field comment).
static func choose_save_path(host: Node, dialog_title: String, extension: String,
		start_dir: String, footnote: String, default_name: String,
		on_choose: Callable, lod_option_text: String = "") -> DccBrowseDialog:
	return _spawn(host, dialog_title, PickKind.SAVE, PackedStringArray([extension]),
		start_dir, footnote, on_choose, default_name, lod_option_text)

static func _spawn(host: Node, dialog_title: String, mode: PickKind,
		extensions: PackedStringArray, start_dir: String, footnote: String,
		on_choose: Callable, default_name: String = "",
		lod_option_text: String = "") -> DccBrowseDialog:
	var d := DccBrowseDialog.new()
	host.add_child(d)
	d._default_name = default_name
	d._lod_option_text = lod_option_text
	d._shell = _shell_of(host)
	d.setup(dialog_title, mode, extensions, footnote, on_choose)
	## One dialog per invocation, freed when it closes -- the same lifetime the
	## `FileDialog` it replaces had. Windows this shell keeps alive (the asset
	## library, the data manager) are long-lived because they hold state worth
	## keeping; a browser holds a directory, and re-listing is instant.
	d.visibility_changed.connect(func():
		if not d.visible:
			d.queue_free())
	d.navigate(start_dir)
	## PH-06: on a phone this fills the screen, and it is `phone_present()`
	## that opens it -- see that function's own doc comment for why sizing a
	## hidden `AcceptDialog` and popping it afterwards leaves the body at its
	## desktop rect (the list then overflows instead of scrolling).
	if not DccWidgets.phone_present(d, d._shell):
		d.popup_centered()
	return d

## The nearest ancestor that is a `DccShell`. `host` is whatever node the
## caller had to hand: `DccApp` itself from most call sites, but the Open
## project dialog passes `self`, a `Window` whose parent is the shell. Walking
## up is what makes one treatment cover both without either call site having
## to know about phones.
static func _shell_of(host: Node) -> Node:
	var n := host
	while n != null and not n.has_method("is_phone"):
		n = n.get_parent()
	return n

func setup(dialog_title: String, mode: PickKind, extensions: PackedStringArray,
		footnote: String, on_choose: Callable) -> void:
	_mode = mode
	_on_choose = on_choose
	for e in extensions:
		_extensions.append(String(e).to_lower().trim_prefix("."))
	title = dialog_title
	## PH-06. The dialog draws its own head row with a ✕ and its own foot with
	## Cancel, so dropping the title bar on a phone costs it nothing -- unlike
	## `new_world_dialog.gd`, this one needed no way-out button adding.
	_phone = DccWidgets.phone_window(self, _shell)
	get_ok_button().hide()   ## the mockup's own foot row replaces it.
	size = Vector2i(760, 640)
	min_size = Vector2i(560, 460)
	_build(dialog_title, footnote)
	## `1.0`: `phone_present()` has already applied the scale once as the
	## window's `content_scale_factor`. `navigate()` re-fits after every
	## listing, since the crumbs and the rows are both rebuilt there.
	if _phone:
		_shell.phone_fit(self, 1.0)

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
	var places := _build_places()
	if places != null:
		outer.add_child(places)

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
## **A breadcrumb is as wide as the path it describes, and that is not a phone
## problem.** `Button.get_minimum_size().x` is the width of its own text, so the
## crumb row's minimum is the whole path plus its separators, and a
## `VBoxContainer` hands the widest child's minimum to the window. Measured
## 2026-09-05 at 4.7.1, three densities -- DESKTOP 1920x1080, LAPTOP 1366x768,
## TABLET 1600x1000 (`--force-touch`) -- with `_lb2browse_probe.gd`, and the
## three agree to the pixel because nothing on this screen reads `ROLE`:
##
## | start directory | crumb row min | dialog `contents_min.x` | window |
## |---|---|---|---|
## | `C:/Users/Vincent` (3) | 187 | 265 (the foot) | 760 |
## | 9 short segments | 541 | **597** | 760 |
## | 9 real segments (`…/Cartalith Projects`) | 640 | **696** | 760 |
## | 13 real segments (`…/heightmaps/2026-09-05`) | 988 | **1044** | 760 |
##
## So a deep path overflowed its own window by **284 px** and there was no
## scrollbar anywhere to reveal it. Note which half of that is the variable:
## 13 one-letter segments measure 729 and fit, 9 real ones measure 696 and
## nearly do not. **Segment count is half of it; segment text is the other
## half**, and a path is unbounded in both.
##
## **Scrolled, and pinned to the tail.** The three candidates, and why this one:
##
##   * *Widen the window to fit.* There is no width that fits: the next folder
##     down makes the path longer. Sizing for the deepest plausible path would
##     also make every shallow browse enormous -- shallow needs 265.
##   * *Elide with `clip_text`.* It collapses a `Label`'s minimum width to 1
##     (`MISTAKES.md`; this dialog's own foot hint shipped at 1 px for exactly
##     that reason), and an elided segment stops being a jump target -- the
##     breadcrumb *is* this dialog's "go up", per the comment above.
##   * *Scroll it.* A `ScrollContainer` contributes no minimum on the axis it
##     scrolls, so the row can no longer widen the window, and every segment
##     stays clickable. Pinned to the **tail** by `_pin_crumbs()` because the
##     last segments are where the user is -- the accent one is the current
##     folder -- and the root is one drag away.
##
## Two details the phone-only version got wrong for the general case. `⌂ Home`
## now sits *outside* the scroll, pinned right where the mockup draws it, so
## the one absolute jump can never be scrolled off; it used to ride at the end
## of `_crumb_row` behind a spacer. And the bar is `SHOW_NEVER` only on the
## phone, where the crumbs drag-scroll (`phone_fit()` leaves them
## `MOUSE_FILTER_PASS`, PH-05); a pointer density gets `AUTO`, which is
## `journey_planner_view.gd`'s own remedy for this trap and the only visible
## affordance a mouse has that the row goes on.
##
## `vertical_scroll_mode` is DISABLED on purpose and is **not** a fifth
## instance of `MISTAKES.md`'s disabled-axis trap: the folded axis is the one
## with nothing to overflow, and folding it is what makes the row exactly as
## tall as one crumb.
func _build_breadcrumb() -> Control:
	_crumb_row = HBoxContainer.new()
	_crumb_row.add_theme_constant_override("separation", 6)

	_crumb_scroll = ScrollContainer.new()
	_crumb_scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_crumb_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_NEVER \
		if _phone else ScrollContainer.SCROLL_MODE_AUTO
	_crumb_scroll.add_theme_stylebox_override("panel", DccTheme.empty())
	_crumb_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_crumb_scroll.add_child(_crumb_row)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	row.add_child(_crumb_scroll)
	row.add_child(_build_home_button())
	return _pad(row, 28, 14, 28, 0)

## `⌂ Home` -- "at the right" in the mockup, and built once here rather than
## per navigation, because it is the one crumb that never changes and the one
## that must survive the scroll.
func _build_home_button() -> Button:
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
	return home

## Put the crumb view at its right-hand end, so the current folder and its
## nearest parents are what is on screen.
##
## Two frames, not `call_deferred()`: the rebuild only *queues* a sort on the
## `HBoxContainer`, whose new minimum then queues a second sort on the
## `ScrollContainer`, and `max_value` is not written until that second one
## runs. A deferred call is flushed inside the first. `scroll_horizontal`
## clamps itself, so this is a no-op on a path that already fits.
##
## Awaits on the tree rather than on a node signal, so the guard below is
## needed: the dialog frees itself on close (`_spawn()`), and a navigation on
## the frame before that would otherwise resume into a freed node.
func _pin_crumbs() -> void:
	for i in 2:
		await get_tree().process_frame
		if not is_inside_tree() or _crumb_scroll == null:
			return
	_crumb_scroll.scroll_horizontal = int(_crumb_scroll.get_h_scroll_bar().max_value)

## The places strip: one tap to each root the process can actually open.
##
## This exists because the breadcrumb is not a way *out* of anywhere -- it only
## walks the ancestors of where you already are, and on Android those ancestors
## are unreadable two segments up. Before this row the only route from the
## landing folder to the user's own files was to know an absolute path and type
## it into the well by hand, which is precisely the "it just accepts a path"
## the browser was supposed to have replaced.
##
## Returns `null` rather than an empty bar when there is nothing worth showing
## (one place is the folder you are already in), so no shell draws a strip that
## cannot take it anywhere.
func _build_places() -> Control:
	var places := _places()
	if places.size() < 2:
		return null
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)
	for pl in places:
		var b := Button.new()
		b.text = String(pl["label"])
		b.tooltip_text = String(pl["path"])
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.add_theme_font_override("font", DccTheme.mono())
		b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
		b.add_theme_color_override("font_color", DccTheme.c("text_ghost"))
		b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
		b.add_theme_stylebox_override("normal", DccTheme.empty())
		b.add_theme_stylebox_override("hover", DccTheme.empty())
		var dest := String(pl["path"])
		b.pressed.connect(func(): navigate(dest))
		row.add_child(b)
	## Scrolls for the same reason the breadcrumb does: five places do not fit
	## a 1080 px phone at this font, and a clipped row that cannot be reached
	## is the defect this whole strip is here to remove.
	var scroll := ScrollContainer.new()
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	scroll.get_h_scroll_bar().visible = false
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.custom_minimum_size.y = 22
	## Flat. A `ScrollContainer` takes the theme's panel, which here is the same
	## bordered well the path field draws -- shipped once and it read as a second
	## typeable field sitting under the real one.
	scroll.add_theme_stylebox_override("panel", DccTheme.empty())
	scroll.add_child(row)
	return _pad(scroll, 28, 2, 28, 10)

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

## The mockup's foot is *"a hint, `Cancel`, `Use this folder`"*, and until this
## rewrite no call site of this dialog had ever put a hint on screen. Two
## separate defects, both measured rather than reasoned about, 2026-09-05 at
## 4.7.1:
##
##   * **In FOLDERS and FILES the hint rendered one pixel wide.** `clip_text`
##     collapses a `Label`'s minimum width to 1 (the hazard `MISTAKES.md`
##     already records), and the `DccTheme.spacer()` beside it is
##     `SIZE_EXPAND_FILL`, so the spacer took every free pixel and the note
##     took its minimum. Three note lengths -- 11, 33 and 76 characters -- all
##     measured `_foot_note.size.x == 1.0`. Fixed by making the note itself
##     the row's expanding child: it *is* the spacer now, so there is nothing
##     left to lose the width to. The invariant that fix buys is that the three
##     lengths all measure the *same* width, because a filling label's width
##     stops depending on its text; **the width itself is the dialog's laid-out
##     width minus the buttons, and that used to be path-dependent.** Before
##     `_build_breadcrumb()` was made to scroll, an overflowing crumb row laid
##     the whole body out at its own minimum and the note rode along:
##     `_lanebpickers_probe.gd`, same machine, 2026-09-05 -- FILES at a 9-segment
##     start directory measured a 823 px body and a **609 px** note, and the
##     same probe after the fix measures a 760 px body and a **546 px** note.
##     Re-measured with `_lb2browse_probe.gd` across 11, 33 and 76 characters,
##     at both a 3-segment and a 13-segment path, all six agreeing: **496 px in
##     FOLDERS, 546 in FILES, 704 in SAVE**. The pair this comment used to carry
##     (460 and 510) does not reproduce in either harness; what produced it was
##     not established, so it is replaced rather than explained.
##   * **SAVE mode had no hint at all.** `choose_save_path` accepts a
##     `footnote` and this function dropped it on the floor -- `menus.gd
##     ::_export_atlas()` passes a full sentence that has never been on
##     screen. A name field, a hint and two buttons do not fit one row, so
##     SAVE keeps the hint on the foot's first line and puts the field with
##     the buttons on the second.
##
## The note exists in every mode, so `_build_new_folder_row()` always has
## somewhere to report a failure; in SAVE it is hidden while the footnote is
## empty, which is `app.gd::save_project_as()`'s case, because a hidden child
## contributes no height to a `VBoxContainer` and an empty one would reserve a
## blank line.
func _build_foot(footnote: String) -> Control:
	var stack := VBoxContainer.new()
	stack.add_theme_constant_override("separation", 8)
	_foot_note = DccTheme.mono_label(footnote, "text_ghost", DccTheme.FS_TINY)
	_foot_note.clip_text = true
	_foot_note.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	if _mode == PickKind.SAVE:
		_foot_note.visible = footnote != ""
		stack.add_child(_foot_note)
		## Owner rulings 28/29 -- a plain, non-canvas-backed control the owner
		## authorised for this one row (`app.gd::save_project_as()`'s own
		## comment carries the full citation). Only drawn when the caller
		## asked for it (`_lod_option_text != ""`), so every other
		## `choose_save_path()` flow (asset pack export, raster export, tile
		## export, GeoJSON export, atlas cache export) is unchanged.
		##
		## A bare `CheckBox` with its own native `.text`, not
		## `DccWidgets.toggle()`: that helper's row fixes its label at
		## `ROW_LABEL_W` for a docked "label: value" property row, which would
		## clip a full sentence like this one's size readout. Unstyled is the
		## point here -- this is the explicitly-authorised undesigned
		## exception, not a widget this shell's canvases define.
		if _lod_option_text != "":
			var lod_check := CheckBox.new()
			lod_check.text = _lod_option_text
			lod_check.button_pressed = false  ## ruling 28: off by default.
			lod_check.focus_mode = Control.FOCUS_NONE
			lod_check.toggled.connect(func(v: bool): lod_tiles_checked = v)
			stack.add_child(lod_check)
		## The one control the mockup's browser does not draw, because the
		## mockup never had a save flow. Kept in the foot beside the primary
		## button -- the "what shall it be called" question belongs next to
		## the "do it" button, not above the folder list.
		row.add_child(DccTheme.mono_label("name", "text_ghost", DccTheme.FS_TINY))
		_name_edit = LineEdit.new()
		_name_edit.text = _default_name
		## 260 is the desktop field width, and on a phone it is a hard *floor*:
		## with the label, both buttons, the separations and the 28 px padding
		## either side it drives the foot past a 1080 px screen. Measured on the
		## handset -- `Cancel` clipped at the right edge and `Save`, the primary
		## action, off-screen entirely, so the save flow could be opened and not
		## completed. Phones get a floor small enough to fit and `EXPAND_FILL` to
		## claim what is left, which is the job the spacer does on desktop; the
		## desktop branch is untouched, so that layout is unchanged.
		_name_edit.custom_minimum_size.x = 120 if _phone else 260
		if _phone:
			_name_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		_name_edit.add_theme_font_size_override("font_size", DccTheme.FS_BODY)
		_name_edit.add_theme_stylebox_override("normal", _well())
		_name_edit.add_theme_stylebox_override("focus", DccTheme.outline("accent"))
		_name_edit.text_changed.connect(func(_t: String): _refresh_primary())
		_name_edit.text_submitted.connect(func(_t: String): _confirm())
		row.add_child(_name_edit)
		if not _phone:
			row.add_child(DccTheme.spacer())
	else:
		row.add_child(_foot_note)
	DccWidgets.modal_button(row, "Cancel", func(): hide())
	var primary_text := "Open"
	if _mode == PickKind.FOLDERS:
		primary_text = "Use this folder"
	elif _mode == PickKind.SAVE:
		primary_text = "Save"
	_primary = DccWidgets.modal_button(row, primary_text, _confirm, true)
	stack.add_child(row)
	return _pad(stack, 28, 14, 28, 14)

# ---------------------------------------------------------------------------
# Navigation
# ---------------------------------------------------------------------------

## Godot exposes no home-directory call. `USERPROFILE` is the Windows
## variable, `HOME` the POSIX one; Android has neither.
##
## **Android does not fall back to the user data dir any more, and the reason
## is measured.** That dir is the app-private sandbox
## (`/data/data/<pkg>/files`), and opening a picker there strands the user:
## on a OnePlus 6T, Android 15, with this APK's *zero* declared permissions,
## it lists only `Cache/`, `shader_cache/` and the settings file, and the
## breadcrumb cannot climb out because `/data` and `/data/data` both fail to
## open with error 31. From the outside that is indistinguishable from "there
## is no file browser" -- which is exactly how it was reported.
##
## Shared storage needs no permission for what this dialog does. Same device,
## same run: `/storage/emulated/0` listed 16 directories, `Documents` and
## `Download` both listed their subfolders, and both accepted `make_dir` plus
## a `FileAccess.WRITE` that read back byte-identical. So Documents is the
## landing, not the sandbox -- and the sandbox stays one tap away in
## `_places()`, because the four storage roots still default into it.
##
## Documents rather than the storage root itself: `/storage/emulated/0`
## refuses `make_dir` at its top level (error 1, Android reserves it), so
## landing there would put `＋ New folder…` one tap from a guaranteed failure.
static func home_dir() -> String:
	for key in ["USERPROFILE", "HOME"]:
		var v := OS.get_environment(key)
		if v != "" and DirAccess.dir_exists_absolute(v):
			return v.simplify_path()
	if OS.get_name() == "Android":
		var docs := OS.get_system_dir(OS.SYSTEM_DIR_DOCUMENTS)
		if docs != "" and DirAccess.open(docs) != null:
			return docs
	return OS.get_user_data_dir()

## The one-tap jump targets under the path well.
##
## **Every entry is verified openable before it is offered.** `OS.get_system_dir`
## happily reports a path the process cannot read -- and a dead row in a picker
## is worse than a missing one, because the user cannot tell a permission wall
## from a bug. `DirAccess.open() != null` is the test; anything that fails it
## is dropped silently.
##
## `/storage` is the concrete case that made this a rule rather than a nicety:
## it looks like the obvious root of the two paths above it and it is *not*
## openable (error 31 on the test device), so deriving "device storage" by
## trimming a path would have offered exactly one broken row.
static func _places() -> Array:
	var out: Array = []
	var seen := {}
	var add := func(label: String, path: String):
		if path == "" or seen.has(path):
			return
		if DirAccess.open(path) == null:
			return
		seen[path] = true
		out.append({"label": label, "path": path})
	if OS.get_name() == "Android":
		var docs := OS.get_system_dir(OS.SYSTEM_DIR_DOCUMENTS)
		## The volume root is Documents' parent, not a literal: the emulated-0
		## spelling is not guaranteed and `/storage` itself cannot be opened.
		add.call("⌂ Device", docs.get_base_dir() if docs != "" else "")
		add.call("Documents", docs)
		add.call("Downloads", OS.get_system_dir(OS.SYSTEM_DIR_DOWNLOADS))
		add.call("Pictures", OS.get_system_dir(OS.SYSTEM_DIR_PICTURES))
		## Last, and named for what it is. The storage roots still default here,
		## so a user who has not moved them must be able to get back.
		add.call("App storage", OS.get_user_data_dir())
	else:
		add.call("⌂ Home", home_dir())
		add.call("Documents", OS.get_system_dir(OS.SYSTEM_DIR_DOCUMENTS))
		add.call("Downloads", OS.get_system_dir(OS.SYSTEM_DIR_DOWNLOADS))
		add.call("Desktop", OS.get_system_dir(OS.SYSTEM_DIR_DESKTOP))
		add.call("App storage", OS.get_user_data_dir())
	return out

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
	## Both refreshes above build fresh nodes -- the breadcrumb segments as
	## well as the rows -- so the touch fit belongs here rather than on either
	## one. Idempotent by meta-flag, so re-walking the whole dialog per
	## navigation only *touches* what the last one did not.
	if _phone:
		_shell.phone_fit(self, 1.0)

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

	## `⌂ Home` is not rebuilt here -- it lives outside the scroll, so it is
	## built once in `_build_breadcrumb()` and survives every clear above. The
	## `DccTheme.spacer()` that used to push it right went with it: the scroll
	## viewport is the expanding child now.
	_pin_crumbs()

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
		var ok := _mode != PickKind.FOLDERS and _extension_ok(path)
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
	## PH-05's rule, in the one place `phone_fit()` cannot reach it: this row
	## is a `PanelContainer` with its own `gui_input` (see below -- a `Button`
	## cannot tell a click from a double-click), and the shared walk excludes
	## exactly that shape because several rows in this shell must keep
	## stopping the event they consume. Here the row *can* forward: `PASS`
	## still delivers the press to the handler below, so click-to-select and
	## double-click-to-enter are untouched, and it also lets a flick reach the
	## `ScrollContainer` above -- without which a phone could not scroll a
	## directory listing at all.
	if _phone and wrap.mouse_filter == Control.MOUSE_FILTER_STOP:
		wrap.mouse_filter = Control.MOUSE_FILTER_PASS

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
			## Every mode has a foot note to write into since `_build_foot()`
			## gave SAVE one of its own -- it used to hand the whole foot to
			## the name field, and this branch went silent there. `show()`
			## because a SAVE dialog opened without a footnote hides it.
			_foot_note.text = "could not create '%s' here" % clean
			_foot_note.show()
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
	elif _mode == PickKind.SAVE and _name_edit != null:
		## Clicking an existing save means "that one" -- the standard
		## overwrite gesture. The folder is already `_cwd`, so only the name
		## has to move.
		_name_edit.text = path.get_file()
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
	## is highlighted. File mode needs a file; save mode needs a name.
	if _mode == PickKind.FILES:
		_primary.disabled = _selected == ""
	elif _mode == PickKind.SAVE:
		_primary.disabled = _save_path() == ""
	else:
		_primary.disabled = false

## SAVE mode's answer: the current folder plus the typed name, with the
## extension appended when the user did not type one. `""` when there is no
## usable name -- an empty field, or one that is only a directory separator.
func _save_path() -> String:
	if _name_edit == null:
		return ""
	var name := _name_edit.text.strip_edges()
	if name == "" or name.ends_with("/") or name.ends_with("\\"):
		return ""
	if not _extensions.is_empty() and not _extension_ok(name):
		name += "." + _extensions[0]
	return _cwd.path_join(name)

func _confirm() -> void:
	var path := _selected if _selected != "" else _cwd
	if _mode == PickKind.SAVE:
		path = _save_path()
		if path == "":
			return
	elif _mode == PickKind.FILES and (_selected == "" or not FileAccess.file_exists(path)):
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

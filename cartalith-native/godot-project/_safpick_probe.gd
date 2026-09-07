extends Node

## Owner ruling A (`LARGE_ITEM_RULINGS.md`, 2026-09-07): Android file-picking
## goes through SAF; folder-picking keeps `DccBrowseDialog`. This probe holds
## the **desktop** half of that ruling and the **URI-shaped** half of the
## materialiser, both of which can be measured off the handset.
##
## What it deliberately does NOT claim: that the picker works. That is an
## on-glass question and it was answered on glass -- SAF cannot be raised
## under any desktop harness, and a probe that cannot answer must not report a
## pass.
##
## Run **windowed**, not `--headless`. The single most important assertion
## here is that `FEATURE_NATIVE_DIALOG_FILE` is **true on this desktop** and
## `_saf_file_picking()` is nonetheless **false** -- which is the difference
## between branching on the platform and branching on the flag. Headless
## reports the feature absent, so the assertion would pass vacuously there and
## prove nothing.

var _fails := 0
var _checks := 0

func _ok(label: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if cond:
		print("SAF ok    %s%s" % [label, "" if detail == "" else "  -- " + detail])
	else:
		_fails += 1
		print("SAF FAIL  %s%s" % [label, "" if detail == "" else "  -- " + detail])

func _ready() -> void:
	call_deferred("_run")

func _run() -> void:
	_platform_branch()
	_names()
	_materialise()
	print("SAF ---- %d check(s), %d failure(s)" % [_checks, _fails])
	get_tree().quit(1 if _fails > 0 else 0)

# ---------------------------------------------------------------------------
# The platform branch
# ---------------------------------------------------------------------------

## The whole point of the guard, asserted the way it can actually fail.
##
## `has_feature(FEATURE_NATIVE_DIALOG_FILE)` is **true here**, on Windows.
## If `_saf_file_picking()` were written against the flag rather than against
## `OS.get_name()`, every desktop `choose_file` would silently become the OS
## picker and the shell would stop drawing its own chrome -- the regression
## the ruling names in one line ("Desktop must not regress"). So the two are
## asserted **apart**: the flag true, the decision false.
func _platform_branch() -> void:
	var native: bool = DisplayServer.has_feature(DisplayServer.FEATURE_NATIVE_DIALOG_FILE)
	var android := OS.get_name() == "Android"
	print("SAF env   OS=%s  FEATURE_NATIVE_DIALOG_FILE=%s" % [OS.get_name(), native])
	_ok("this harness is not Android", not android, OS.get_name())
	## The positive control. Without it a `false` below could mean "the guard
	## reads the platform" or "this build has no native dialog at all", and
	## those are not the same claim.
	_ok("the desktop DOES advertise a native file dialog", native,
		"has_feature=%s" % native)
	_ok("...and file-picking still does NOT route to SAF",
		not DccBrowseDialog._saf_file_picking(),
		"_saf_file_picking()=%s" % DccBrowseDialog._saf_file_picking())

	## And the desktop dialog is still the one that comes back -- the return
	## value, not just the absence of an OS picker. `null` here is what a
	## caller reading the return would trip over first.
	##
	## **Gated on the decision above, and that gate is not decoration.**
	## Mutating the guard to read the flag alone was run for real: on Windows
	## `choose_file` then reached `DisplayServer.file_dialog_show`, which
	## raised a **live modal OS file dialog on the owner's desktop** and hung
	## the harness until it was killed. A probe that has already measured
	## "this platform must not raise the OS picker" must not then raise it.
	if DccBrowseDialog._saf_file_picking():
		return
	var holder := Node.new()
	add_child(holder)
	var d := DccBrowseDialog.choose_file(holder, "probe", PackedStringArray(["zip"]),
		DccBrowseDialog.home_dir(), "hint", Callable())
	_ok("choose_file still returns a DccBrowseDialog on desktop", d != null,
		"null" if d == null else d.get_class())
	if d != null:
		_ok("...in FILES mode", d._mode == DccBrowseDialog.PickKind.FILES, str(d._mode))
		_ok("...with the caller's extensions", str(d._extensions) == '["zip"]',
			str(d._extensions))
		d.hide()
	holder.queue_free()

# ---------------------------------------------------------------------------
# Naming a document that has no name
# ---------------------------------------------------------------------------

## The two document-URI shapes Android actually produces, plus the one that
## matters most: a tail that looks like a name and is not one.
func _names() -> void:
	var zip := PackedStringArray(["zip"])
	var ext_storage := "content://com.android.externalstorage.documents/document/primary%3ADocuments%2FWerk.zip"
	_ok("ExternalStorageProvider tail is a real name",
		DccBrowseDialog._saf_local_name(ext_storage, zip) == "Werk.zip",
		DccBrowseDialog._saf_local_name(ext_storage, zip))

	## `msf:1000000123` has no extension, so it is not a name, and calling the
	## copy that would be the bug: the user would get a file called
	## `msf:1000000123` in their Worlds folder and no way to tell what it was.
	var downloads := "content://com.android.providers.downloads.documents/document/msf%3A1000000123"
	var made := DccBrowseDialog._saf_local_name(downloads, zip)
	_ok("opaque DownloadsProvider id is NOT used as a name",
		not made.contains("msf"), made)
	_ok("...and the stamped name carries the caller's extension",
		made.get_extension() == "zip", made)

	## The direction that would survive a careless "just take the tail":
	## a tail with the WRONG extension. `.txt` is a name, and it is not a name
	## for a `.zip` pick.
	var wrong := "content://x/document/primary%3ADocuments%2Fnotes.txt"
	_ok("a tail with the wrong extension is refused",
		not DccBrowseDialog._saf_local_name(wrong, zip).ends_with("notes.txt"),
		DccBrowseDialog._saf_local_name(wrong, zip))
	## ...and the same tail IS taken when the caller asked for it.
	_ok("...and taken when the caller asked for .txt",
		DccBrowseDialog._saf_local_name(wrong, PackedStringArray(["txt"])) == "notes.txt",
		DccBrowseDialog._saf_local_name(wrong, PackedStringArray(["txt"])))

# ---------------------------------------------------------------------------
# The copy
# ---------------------------------------------------------------------------

## `_saf_materialise()` reads its source with `FileAccess`, which on Android
## is handed a `content://` URI and on a desktop is happy with a plain path.
## That is what makes the copy, the verify, the reuse and the anti-clobber
## suffix measurable here rather than only on glass.
func _materialise() -> void:
	var root := OS.get_user_data_dir().path_join("_safprobe")
	## Recursively, and before anything else. A previous run's `Werk.zip`
	## left in `dst/` makes "left exactly one file behind" fail on the second
	## run and pass on the first, which is a probe reporting its own state
	## rather than the code's.
	_wipe(root)
	var src_dir := root.path_join("src")
	var dst_dir := root.path_join("dst")
	DirAccess.make_dir_recursive_absolute(src_dir)
	DirAccess.make_dir_recursive_absolute(dst_dir)

	## Not a token payload: 2.5 MB crosses the 1 MiB chunk `_saf_materialise`
	## copies with, so a loop that only ever ran once would pass a small
	## fixture and truncate a real 3.5 MB project.
	var payload := PackedByteArray()
	payload.resize(2_500_000)
	for i in range(0, payload.size(), 9973):
		payload[i] = (i / 9973) % 251
	var src := src_dir.path_join("Werk.zip")
	var f := FileAccess.open(src, FileAccess.WRITE)
	f.store_buffer(payload)
	f.close()

	var got := DccBrowseDialog._saf_materialise(src, PackedStringArray(["zip"]), dst_dir)
	_ok("materialise returned a path", got != "", got)
	_ok("...inside the caller's own start_dir", got.get_base_dir() == dst_dir,
		got.get_base_dir())
	_ok("...named for the document", got.get_file() == "Werk.zip", got.get_file())
	var back := FileAccess.open(got, FileAccess.READ)
	_ok("...and the copy is byte-complete",
		back != null and back.get_length() == payload.size(),
		"%d of %d" % [0 if back == null else back.get_length(), payload.size()])
	_ok("...byte-identical, not merely the right length",
		back != null and back.get_buffer(back.get_length()) == payload)

	## Second pick of the same document must reuse, not litter. This is the
	## common case -- it is how a person reopens a project.
	var again := DccBrowseDialog._saf_materialise(src, PackedStringArray(["zip"]), dst_dir)
	_ok("re-picking the same document reuses the copy", again == got, again)
	_ok("...and left exactly one file behind",
		DirAccess.get_files_at(dst_dir).size() == 1,
		str(DirAccess.get_files_at(dst_dir)))

	## A different document of the same name must NOT overwrite the first --
	## the direction that loses a user's file rather than merely annoying them.
	var other := src_dir.path_join("other").path_join("Werk.zip")
	DirAccess.make_dir_recursive_absolute(other.get_base_dir())
	var g := FileAccess.open(other, FileAccess.WRITE)
	g.store_buffer(payload.slice(0, 1_000_000))
	g.close()
	var third := DccBrowseDialog._saf_materialise(other, PackedStringArray(["zip"]), dst_dir)
	_ok("a different document of the same name gets its own file", third != got, third)
	_ok("...suffixed, extension intact", third.get_file() == "Werk-2.zip",
		third.get_file())
	var first_again := FileAccess.open(got, FileAccess.READ)
	_ok("...and the first copy is untouched",
		first_again != null and first_again.get_length() == payload.size(),
		"%d of %d" % [0 if first_again == null else first_again.get_length(),
			payload.size()])

	## A destination that does not exist yet is the fresh-install case:
	## `storage_root("projects")` is not created until the first save.
	var fresh := root.path_join("never_made")
	var into_fresh := DccBrowseDialog._saf_materialise(src, PackedStringArray(["zip"]), fresh)
	_ok("a start_dir that does not exist yet is created, not abandoned",
		into_fresh.get_base_dir() == fresh, into_fresh)

	## An unreadable source returns "" rather than a plausible path -- the
	## `MISTAKES.md` rule about never encoding "no value" as a value.
	_ok("an unreadable document returns \"\", not a path",
		DccBrowseDialog._saf_materialise(src_dir.path_join("nothing.zip"),
			PackedStringArray(["zip"]), dst_dir) == "")

## `DirAccess` has no recursive remove. Depth-first, files then directories.
static func _wipe(path: String) -> void:
	if not DirAccess.dir_exists_absolute(path):
		return
	for f in DirAccess.get_files_at(path):
		DirAccess.remove_absolute(path.path_join(f))
	for d in DirAccess.get_directories_at(path):
		_wipe(path.path_join(d))
	DirAccess.remove_absolute(path)

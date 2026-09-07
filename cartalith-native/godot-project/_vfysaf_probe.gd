extends Node
## VERIFIER probe -- `DccBrowseDialog._saf_materialise()` exercised directly.
## It is platform-agnostic GDScript (FileAccess + DirAccess), so every branch
## except the `content://` scheme itself is testable on a desktop.
##
##   ... --headless _vfysaf_probe.tscn
##
## Legs: exact-length copy; reuse on same-name-same-length; `-2` suffix on
## same-name-different-length; SAME LENGTH BUT DIFFERENT BYTES (the silent
## case); a dest_dir that does not exist; a dest_dir that exists and CANNOT BE
## WRITTEN -- which is what `_picker_start_dir()` -> `home_dir()` resolves to on
## the handset this feature ships for.

var _fail := 0
func _log(s: String) -> void: print("[vfysaf] %s" % s)
func _chk(ok: bool, w: String) -> void:
	if not ok: _fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", w])

func _write(path: String, n: int, fill: int) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	var b := PackedByteArray(); b.resize(n); b.fill(fill)
	f.store_buffer(b); f.close()

func _count(dir: String) -> int:
	var d := DirAccess.open(dir)
	if d == null: return -1
	return d.get_files().size()

func _ready() -> void:
	var root := OS.get_user_data_dir().path_join("_vfysaf")
	DirAccess.make_dir_recursive_absolute(root)
	var src := root.path_join("src"); DirAccess.make_dir_recursive_absolute(src)
	var dst := root.path_join("dst"); DirAccess.make_dir_recursive_absolute(dst)
	var exts := PackedStringArray(["zip"])

	var a := src.path_join("Werk.zip"); _write(a, 4096, 0x41)
	var r1: String = DccBrowseDialog._saf_materialise(a, exts, dst)
	_chk(r1 == dst.path_join("Werk.zip") and FileAccess.file_exists(r1)
		and FileAccess.open(r1, FileAccess.READ).get_length() == 4096,
		"copy lands at dest/Werk.zip at the exact length (got '%s')" % r1)

	var r2: String = DccBrowseDialog._saf_materialise(a, exts, dst)
	_chk(r2 == r1 and _count(dst) == 1, "re-picking the same document REUSES (files in dest = %d)" % _count(dst))

	## Different length, same name -> a suffix, and the first copy survives.
	var b := src.path_join("other").path_join("Werk.zip")
	DirAccess.make_dir_recursive_absolute(src.path_join("other"))
	_write(b, 8192, 0x42)
	var r3: String = DccBrowseDialog._saf_materialise(b, exts, dst)
	_chk(r3 == dst.path_join("Werk-2.zip") and _count(dst) == 2
		and FileAccess.open(r1, FileAccess.READ).get_length() == 4096,
		"a different document of the same NAME gets -2 and leaves the first alone (got '%s')" % r3)

	## THE SILENT CASE: same name, same length, DIFFERENT BYTES.
	var c := src.path_join("same").path_join("Werk.zip")
	DirAccess.make_dir_recursive_absolute(src.path_join("same"))
	_write(c, 4096, 0x5A)
	var r4: String = DccBrowseDialog._saf_materialise(c, exts, dst)
	var got := FileAccess.open(r4, FileAccess.READ).get_buffer(1)[0]
	_log("  same-name-same-length-DIFFERENT-BYTES -> '%s', first byte 0x%02X (source was 0x5A, stale copy is 0x41)"
		% [r4, got])
	_chk(got == 0x5A, "the caller receives the document the person actually picked, not a stale same-length copy")

	## dest_dir absent -> created.
	var fresh := root.path_join("made/up/deep")
	var r5: String = DccBrowseDialog._saf_materialise(a, exts, fresh)
	_chk(r5 == fresh.path_join("Werk.zip") and FileAccess.file_exists(r5),
		"a dest_dir that does not exist is created (got '%s')" % r5)

	## dest_dir EXISTS and CANNOT BE WRITTEN -- `home_dir()`'s Android answer.
	var ro := "C:/Windows/System32" if OS.get_name() == "Windows" else "/proc"
	_log("  unwritable dest probe: %s  dir_exists=%s" % [ro, DirAccess.dir_exists_absolute(ro)])
	var r6: String = DccBrowseDialog._saf_materialise(a, exts, ro)
	_log("  -> returned '%s'" % r6)
	_chk(r6 == "", "an unwritable dest_dir returns '' (the caller then reports 'could not read the file that was picked')")

	_log("RESULT vfysaf fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

extends SceneTree
## `DccSettings` autosave-default migration probe.
##
##   godot --headless --path . --script _autodefault_probe.gd
##
## Two claims, and they are the two halves of the absent-vs-set rule:
##
##   1. An **absent** key takes the new design default -- on, every 5 min.
##   2. An **explicitly stored** value survives the change untouched, off and
##      15 min included, because `ConfigFile.get_value` never consults its
##      default argument for a key that exists.
##
## The live `user://cartalith_settings.cfg` on this machine has no `[autosave]`
## section at all, which is exactly case 1; case 2 is produced by writing one
## through the same setters the File menu calls. The file's original bytes are
## captured before anything is written and restored at the end, so running this
## leaves the install as it was found.

var _fail := 0

func _ok(name: String, got: Variant, want: Variant) -> void:
	var pass_ := str(got) == str(want)
	print("AUTO %s  %-34s got=%s want=%s" % ["ok  " if pass_ else "FAIL", name, str(got), str(want)])
	if not pass_:
		_fail += 1

func _init() -> void:
	var path := DccSettings.CONFIG_PATH
	var before := FileAccess.get_file_as_bytes(path)
	print("AUTO cfg=%s  bytes=%d  had_autosave_section=%s" % [
		ProjectSettings.globalize_path(path), before.size(),
		str(FileAccess.get_file_as_string(path).contains("[autosave]"))])

	# 1 -- absent key.
	_ok("absent enabled -> design default", DccSettings.autosave_enabled(), true)
	_ok("absent minutes -> design default", DccSettings.autosave_minutes(), 5)
	_ok("5 is on the interval ladder", DccMenus.AUTOSAVE_MINUTES.has(DccSettings.autosave_minutes()), true)

	# 2 -- explicitly stored values, written the way the File menu writes them.
	DccSettings.set_autosave_enabled(false)
	DccSettings.set_autosave_minutes(15)
	_ok("stored false survives new default", DccSettings.autosave_enabled(), false)
	_ok("stored 15 survives new default", DccSettings.autosave_minutes(), 15)

	## Restore the exact bytes read above, then prove it: an install that was
	## absent-key before this ran must be absent-key after it.
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_buffer(before)
	f.close()
	_ok("config restored byte-for-byte",
		FileAccess.get_file_as_bytes(path) == before, true)
	print("AUTO %s (%d failed)" % ["PASS" if _fail == 0 else "FAIL", _fail])
	quit(_fail)

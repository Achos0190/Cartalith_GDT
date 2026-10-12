extends SceneTree

## VERIFIER probe (untracked, delete after use). Reads only -- never writes
## `user://cartalith_settings.cfg`. The config is staged by the harness before
## each run so each leg is a fresh process with its own `DccSettings._loaded`.
##
## Run:    Godot_v4.7.1-stable_win64_console.exe --headless --path . --script _vfyautosave_probe.gd
##         (a SceneTree script, no .tscn; one process per leg -- stage `user://
##         cartalith_settings.cfg` first, the probe only reads it)
## Result: a single reading, no PASS/FAIL: `RESULT file_bytes= has_section= has_enabled=
##         has_minutes= enabled= minutes= ladder=[...] ladder_has_5= ladder_has_10=`; exit 0.

func _init() -> void:
	var f := FileAccess.open(DccSettings.CONFIG_PATH, FileAccess.READ)
	var bytes := 0
	var raw := ""
	if f != null:
		raw = f.get_as_text()
		bytes = raw.length()
		f.close()
	var probe := ConfigFile.new()
	probe.load(DccSettings.CONFIG_PATH)
	var has_sec := probe.has_section("autosave")
	var has_en := has_sec and probe.has_section_key("autosave", "enabled")
	var has_mn := has_sec and probe.has_section_key("autosave", "minutes")
	print("RESULT file_bytes=%d has_section=%s has_enabled=%s has_minutes=%s enabled=%s minutes=%d ladder=%s ladder_has_5=%s ladder_has_10=%s" % [
		bytes, has_sec, has_en, has_mn,
		DccSettings.autosave_enabled(), DccSettings.autosave_minutes(),
		str(DccMenus.AUTOSAVE_MINUTES),
		5 in DccMenus.AUTOSAVE_MINUTES, 10 in DccMenus.AUTOSAVE_MINUTES])
	quit(0)

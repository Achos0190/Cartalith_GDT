extends "res://_pcl_base.gd"
## The seventeen `vault_*` `#[func]`s (`_vefuncs_checks.gd`) run INSIDE the real shell on each form factor: the shell
## boots, generates a world and has its bridge, and the shared checks call the functions through that live bridge,
## while a world is loaded and the shell's own vault code is wired (the headless `_vefuncs_probe.gd` has neither).
## The functions answer the same on every form; this proves it where the owner asked (desktop 1600x900, tablet
## 1700x1000 `--force-touch --density=1.0`, phone 1080x2340 `--force-touch --density 2.625`) and that none of them
## changes because a world is loaded, the form is touch, or the shell is running.
##
##   godot --path . --resolution 1600x900 _vefuncs_form_probe.tscn -- [--force-touch] [--density 2.625] [--shots <dir>]
##
## Windowed, under `C:/cw/.win_lock`, scratch APPDATA (`MISTAKES.md`). Also checks that the form took the shape its
## name says, so a run on the wrong window size cannot pass as the form it was meant to be.

func _run() -> void:
	_prefix = "vef"
	var want := _expected_mode()
	_ok(want == "" or _mode == want, "the shell booted as the %s form (mode %s)" % [want, _mode])
	var checks = load("res://_vefuncs_checks.gd").new()
	var code: int = checks.run(_bridge)
	_checks += int(checks.checks)
	_fails += int(checks.fails)
	_ok(code == 0, "the seventeen functions' shared checks passed (%d checks, exit %d)" % [checks.checks, code])
	await _settle(4)
	await _shot("after")


## The form the command line is meant to give, by the owner's three recipes: a plain window is the desktop,
## `--force-touch --density=1.0` the tablet, `--force-touch --density 2.625` (the value after a space) the phone. A window
## size is not used: this display clamps a 1080x2340 window to about 1 030 px tall, so its aspect says nothing.
## "" when the flags match none of the three (no assertion).
func _expected_mode() -> String:
	var args := OS.get_cmdline_user_args()
	if not "--force-touch" in args:
		return "desktop"
	if "--density" in args:
		return "phone"
	for a in args:
		if String(a).begins_with("--density="):
			return "tablet"
	return ""

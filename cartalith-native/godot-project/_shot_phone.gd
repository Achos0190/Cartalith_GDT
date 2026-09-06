extends Node
## Phone-chrome capture harness, copied from `_shot.gd` rather than editing it
## in place (that file is git-tracked and other work may be mid-flight on it).
## The only real difference: `--force-touch` (read by `DccShell._ready()`)
## makes the phone/tablet composition reachable at all in this headless dev
## environment, which has no real touchscreen for `DisplayServer
## .is_touchscreen_available()` to find, and the output filename is distinct
## so a portrait and a landscape capture (or a desktop capture from `_shot.gd`
## itself) never clobber each other.
##
## Run:
##   godot --path . --resolution 393x852 _shot_phone.tscn -- --force-touch
##   godot --path . --resolution 852x393 _shot_phone.tscn -- --force-touch
## **`--generate` is effectively mandatory on the phone, not an extra.**
## Measured 2026-09-06 at 393x852: with `--nowelcome` and no world, the phone
## composition sits on its full-screen project picker, so every overlay flag
## below opens its overlay *behind* that picker and the saved PNG is of the
## picker. Three runs, three md5s -- `--search` and `--phoneoverflow` came out
## byte-identical to each other and only the control differed, which is the tell.
## The same three flags with `--generate` give three distinct frames.
##
## One of `--search` / `--phoneoverflow` / `--overflow` / `--leftsheet` /
## `--rightsheet` force-opens that phone overlay before the capture, since
## none of them are reachable by a script driving no real input.
##
## **Every flag in that list is grepped out of the body below, not remembered.**
## `ARGS` is the one list, `_ready()` reads it, and an argument that is not in it
## **aborts with exit 2** rather than being ignored -- `MISTAKES.md`'s "write a
## probe's usage header" row, applied to the file that needed it: this harness
## carried `--drawer` and `--picker` in its header and its body for months after
## both surfaces were deleted, and a run passing either got a silent
## shell-at-rest capture saved under the ordinary filename.

## The complete accepted set. `--force-touch` is not consumed here -- `DccShell
## ._ready()` reads it off the same `OS.get_cmdline_user_args()` -- but it is
## still a legal argument to this scene, so it is listed rather than rejected.
const ARGS: Array[String] = [
	"--force-touch", "--generate", "--nowelcome", "--rotate",
	"--search", "--phoneoverflow", "--overflow", "--leftsheet", "--rightsheet",
]

func _ready() -> void:
	## Loudly, and before the shell is even instantiated: an unrecognised flag
	## means the caller is asking for a state this harness cannot produce, and
	## the worst outcome is a capture that looks like the requested one.
	var unknown := PackedStringArray()
	for a in OS.get_cmdline_user_args():
		if not (a in ARGS):
			unknown.append(a)
	if unknown.size() > 0:
		push_error("_shot_phone: unknown argument(s) %s -- accepted: %s"
			% [", ".join(unknown), ", ".join(ARGS)])
		print("_shot_phone: unknown argument(s) ", ", ".join(unknown),
			"  -- accepted: ", ", ".join(ARGS))
		get_tree().quit(2)
		return

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(0.8).timeout

	if "--generate" in OS.get_cmdline_user_args():
		var bridge = app.bridge
		bridge.generate({
			"seed": 483920, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
			"archetype": "", "villages": true, "sea_level": 0.42,
		})
		while bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await get_tree().create_timer(0.6).timeout

	## `app.gd` opens the welcome dialog whenever no world exists, so without
	## this every capture is of that dialog rather than of the shell behind it.
	if "--nowelcome" in OS.get_cmdline_user_args():
		app.open_project_dialog.hide()
		await get_tree().process_frame

	## Simulates a device rotation the only way this environment can: resize the
	## real window to the transposed resolution and let `root.size_changed` fire
	## exactly as Android's own rotation makes it fire. This is what verifies
	## `_on_window_resized()` -> `_apply_phone_orientation()` actually re-lays the
	## shell, as opposed to `--resolution` merely *booting* into an orientation.
	if "--rotate" in OS.get_cmdline_user_args():
		var w := get_window()
		w.size = Vector2i(w.size.y, w.size.x)
		await get_tree().process_frame
		await get_tree().process_frame
		await get_tree().create_timer(0.4).timeout

	## **`--drawer` and `--picker` are gone, not renamed.** `☰`'s side drawer went
	## with the 412 dp migration (`_set_drawer_open()` deleted 2026-08-25, see
	## `_hidpi_probe.gd`'s own post-mortem) and `▤`'s panel picker with owner
	## ruling 20; `dcc_shell.gd`'s ruling-20 block records that the picker was a
	## *router* whose only two destinations were the left and right dock sheets,
	## and `--leftsheet` / `--rightsheet` below open those directly. So nothing
	## replaces either flag: there is no surface left for a capture to be of.
	##
	## The two cells ruling 20 **kept** had no flag at all, which is why the `⌕`
	## overlay and the `⋮` popover have never been captured by this harness.
	## `--phoneoverflow` is deliberately not spelled `--overflow`:
	## `_set_overflow_open()` opens `PhoneMenu`'s L2 root and
	## `_set_phone_overflow_open()` opens the `⋮` popover -- two functions eleven
	## characters apart, and `dcc_shell.gd`'s comment on the first records what
	## confusing them already cost (three popover rows that no warm-up in the
	## tree had ever laid out).
	##
	## `open_find_on_map()` rather than `_set_search_open()` so the capture goes
	## through the same guard the `⌕` cell draws itself behind; it is checked
	## afterwards so a refusal prints instead of saving a shell-at-rest frame
	## under a name that claims otherwise.
	if "--search" in OS.get_cmdline_user_args():
		app.open_find_on_map()
		await get_tree().process_frame
		var ov: Control = app.get("_phone_search_overlay") as Control
		if ov == null or not ov.visible:
			print("_shot_phone: --search did not open the overlay (",
				"absent" if ov == null else "built but hidden", ")")
	if "--phoneoverflow" in OS.get_cmdline_user_args():
		app._set_phone_overflow_open(true)
	if "--overflow" in OS.get_cmdline_user_args():
		app._set_overflow_open(true)
	if "--leftsheet" in OS.get_cmdline_user_args():
		app._set_sheet_open("left", true)
	if "--rightsheet" in OS.get_cmdline_user_args():
		app._set_sheet_open("right", true)
	await get_tree().process_frame
	await get_tree().process_frame

	var size := get_viewport().get_visible_rect().size
	var orientation := "landscape" if size.x > size.y else "portrait"
	var img := get_viewport().get_texture().get_image()
	var out := "user://shell_shot_phone_%s.png" % orientation
	img.save_png(out)
	print("saved ", ProjectSettings.globalize_path(out))
	get_tree().quit()

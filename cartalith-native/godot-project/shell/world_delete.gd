extends RefCounted
## The red trash can on every world card, and the confirmation that stands between a tap on it and
## the project file (owner, 2026-11-14: "a trash can in the bottom right of the card line art, red.
## And with a confirmation splash.").
##
## Shared by `open_project_dialog.gd` (desktop and tablet welcome cards) and `phone_project_picker.gd`
## so the glyph, the refusal rules and the delete itself exist once (Ruling BK: one implementation).
##
## **Must never:** open the world (the button is a `Button`, which stops the press before the card's
## own `gui_input` sees it); delete the world that is currently open; delete without the splash;
## default-focus the Delete answer (a stray Enter or tap must land on Cancel); claim success when
## `DirAccess.remove_absolute` failed.

## Glyph box in px. The touch target is `TARGET` (44, the project's tap floor), the drawn can is smaller.
const TARGET := 44.0
const GLYPH := 18.0

## A flat, transparent `Button` with a red line-art can drawn on it. `on_press` runs on release only.
static func make_button(on_press: Callable) -> Button:
	var b := Button.new()
	b.name = "TrashButton"
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.custom_minimum_size = Vector2(TARGET, TARGET)
	b.size_flags_horizontal = Control.SIZE_SHRINK_END
	b.size_flags_vertical = Control.SIZE_SHRINK_END
	b.tooltip_text = "Delete this world"
	b.set_meta("trash_red", DccTheme.c("block"))
	b.draw.connect(func():
		var col: Color = b.get_meta("trash_red")
		if b.is_hovered() or b.is_pressed():
			col = col.lightened(0.25)
		var s := b.size
		var c := s * 0.5
		var h := GLYPH * 0.5
		var w := 1.6
		## Lid line and handle, then the tapering body and two ribs: a bin in outline.
		b.draw_line(c + Vector2(-h, -h * 0.6), c + Vector2(h, -h * 0.6), col, w, true)
		b.draw_polyline(PackedVector2Array([c + Vector2(-h * 0.35, -h * 0.6), c + Vector2(-h * 0.35, -h),
			c + Vector2(h * 0.35, -h), c + Vector2(h * 0.35, -h * 0.6)]), col, w, true)
		b.draw_polyline(PackedVector2Array([c + Vector2(-h * 0.8, -h * 0.6), c + Vector2(-h * 0.65, h),
			c + Vector2(h * 0.65, h), c + Vector2(h * 0.8, -h * 0.6)]), col, w, true)
		b.draw_line(c + Vector2(-h * 0.25, -h * 0.25), c + Vector2(-h * 0.25, h * 0.65), col, w, true)
		b.draw_line(c + Vector2(h * 0.25, -h * 0.25), c + Vector2(h * 0.25, h * 0.65), col, w, true))
	b.pressed.connect(on_press)
	return b

## True when a press at `pos` (in `wrap`'s local space) lands on `trash`. The phone shell's `phone_fit()` turns
## every `BaseButton` into `MOUSE_FILTER_PASS` so a flick that starts on a button still scrolls, and a PASS
## button lets the press continue up to the card's own `gui_input`, which would open the world under the
## confirmation. The card's handler calls this first and returns on a hit (measured 2026-11-14, phone probe).
static func hits(trash: Control, wrap: Control, pos: Vector2) -> bool:
	return trash != null and trash.get_global_rect().has_point(wrap.get_global_transform() * pos)

## Opens the confirmation for `path`. `say(text)` reports a refusal or failure on the picker; `done()`
## runs after a real deletion so the picker rebuilds. Refuses (with a reason) for the open world.
static func ask(host: DccApp, path: String, title: String, say: Callable, done: Callable) -> void:
	if host != null and host.current_project_path == path:
		say.call("%s is open -- close it (File > New world or open another) before deleting it." % title)
		return
	if not FileAccess.file_exists(path):
		say.call("%s is no longer on disk." % path.get_file())
		done.call()
		return
	var card := DccWidgets.modal_card(host, "Delete %s?" % title, DccWidgets.MODAL_DESTRUCTIVE)
	DccWidgets.modal_prose(card["body"], "The world's file is removed from this device. This cannot be undone.")
	DccWidgets.modal_foot(card["body"], path)
	DccWidgets.modal_choices(card, {
		"safe": {"text": "Cancel", "on": func(): pass},
		"destructive": {"text": "Delete", "on": func(): _delete(path, title, say, done)},
	})
	DccWidgets.modal_present(card, host)

## The removal. Reports the engine's error code rather than assuming; Android's shared Documents
## path is the same one the picker lists, so `remove_absolute` is the right call there too.
static func _delete(path: String, title: String, say: Callable, done: Callable) -> void:
	var err := DirAccess.remove_absolute(path)
	if err != OK or FileAccess.file_exists(path):
		say.call("Could not delete %s (error %d: %s)." % [path.get_file(), err, error_string(err)])
		return
	DccSettings.forget_project(path)
	done.call()

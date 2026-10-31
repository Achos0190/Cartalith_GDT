## DRAFT, not loaded by Godot: this folder is outside `godot-project/`. MS-1 stage S1
## (`MOBILE_SHELL_MS1_SPEC.md` §3.1). Target: `shell/dcc_theme.gd`, beside the form-factor flags
## (`set_touch()` … `is_tablet()`).
##
## Paste the block below into DccTheme. Then:
##   1. delete `_PHONE_ASPECT_MAX` and `_TABLET_MIN_DP` from `dcc_shell.gd` and point their readers at
##      `DccTheme.PHONE_ASPECT_MAX` / `DccTheme.TABLET_MIN_DP`. Move their long comments here with them, and
##      correct `_TABLET_MIN_DP`'s worked table while moving it: its rows `TABLET 800 -> phone` and
##      `2560x1600 @ 288 dpi -> phone` are false for the code (both are aspect 0.625, so both are tablets;
##      MS-1 spec §2.2);
##   2. no new `class_name` is introduced, so the import pass (which strips `project.godot`'s `;`
##      comments, `MISTAKES.md`) is not needed.
##
## NOT parse-checked: the session that wrote it had no Godot binary. Before anything else, run
##   godot --headless --path cartalith-native/godot-project --check-only --script res://shell/dcc_theme.gd
## and fix what it reports.

# -- Size classes in dp (MS-1) ---------------------------------------------------

## The phone/tablet line's aspect half: `min(w, h) / max(w, h)` below this is a narrow screen (16:9 or
## narrower). Moved unchanged from `DccShell._PHONE_ASPECT_MAX`, whose comment carries the device table.
const PHONE_ASPECT_MAX := 0.6
## The phone/tablet line's size half. Owner answer, `design/dcc-environment-2026-08-31/BUILD_ANSWERS.md`
## §2.3 (2026-08-31): a shortest side under 900 dp is a phone. Today's code applies it only below
## `PHONE_ASPECT_MAX`, and MS-1 keeps exactly that (MQ-1's default). Where the line ends up is MQ-1.
const TABLET_MIN_DP := 900.0
## Hysteresis (MS-1 spec §3.1): a class is left only when the size is past its line by this much, so a
## window parked on a line cannot flap. Labelled judgements, revisited with stage S3's numbers.
const BAND_DP := 16.0
const BAND_ASPECT := 0.01
## The proposal's width classes (`MOBILE_SHELL_PROPOSAL.md` §3), as [name, lower bound in dp]. MS-7 (the
## docking rule) is the first real consumer; in MS-1 only `_ms1class_probe` reads them.
const WIDTH_CLASSES: Array = [
	["compact", 0.0], ["medium", 600.0], ["expanded", 840.0], ["large", 1200.0], ["xl", 1600.0],
]

## Classifies a viewport. Pure (no node, no DisplayServer), so a probe can call it with any numbers.
##
## `px` is the hosting viewport's size in PHYSICAL px: `Window.size` for the root, `SubViewport.size` in a
## probe. Never `get_viewport_rect()`, which turns into dp once a content scale is applied.
## `density_known` is false only for the legacy figure (`DccShell._density()`); the dp half of the line is
## then skipped, exactly as today's off-device rule skips it. `current` is the previous result, for
## hysteresis; pass `{}` at boot.
##
## Returns `{dp: Vector2, short_dp: float, aspect: float, phone: bool, landscape: bool, width_class: String}`.
## Never a partial dictionary. A size or density of zero or less keeps `current` when there is one, and
## otherwise answers not-phone: the same direction `_is_tablet_sized()` falls back to today.
static func classify(px: Vector2, density: float, ui_scale: float, touch: bool,
		density_known: bool, current: Dictionary = {}) -> Dictionary:
	var k := density * ui_scale
	if px.x <= 0.0 or px.y <= 0.0 or k <= 0.0:
		if not current.is_empty():
			return current
		return {"dp": Vector2.ZERO, "short_dp": 0.0, "aspect": 0.0, "phone": false,
			"landscape": px.x > px.y, "width_class": "xl"}
	var dp := px / k
	var short_dp := minf(dp.x, dp.y)
	var aspect := minf(px.x, px.y) / maxf(px.x, px.y)
	## Both lines sit exactly at their constants at boot. Afterwards each moves by its band in the
	## direction that keeps the current class: a phone stays a phone until it is past the line by the band,
	## and so does a tablet.
	var aspect_line := PHONE_ASPECT_MAX
	var size_line := TABLET_MIN_DP
	if not current.is_empty():
		if bool(current.get("phone", false)):
			aspect_line += BAND_ASPECT
			size_line += BAND_DP
		else:
			aspect_line -= BAND_ASPECT
			size_line -= BAND_DP
	var phone := touch and aspect < aspect_line and (not density_known or short_dp < size_line)
	return {"dp": dp, "short_dp": short_dp, "aspect": aspect, "phone": phone,
		"landscape": px.x > px.y, "width_class": width_class_of(dp.x)}

## The proposal's width class for a width in dp. Pure; the lowest class for anything below 600.
static func width_class_of(width_dp: float) -> String:
	var out := "compact"
	for row in WIDTH_CLASSES:
		if width_dp >= float(row[1]):
			out = String(row[0])
	return out

## The last classification, published by `DccShell` (its single writer) for the static widget factories,
## which have no node to reach the shell through. `{}` until the shell has booted; readers must treat an
## empty dictionary as "not classified yet", never as a phone or a tablet.
static var _layout: Dictionary = {}

## Published by `DccShell._compute_layout_mode()` after every classification. Never call it from a widget or
## a probe without restoring the old value: it changes every static factory's answer for the process.
static func set_layout(d: Dictionary) -> void:
	_layout = d

## The last classification, or `{}` before the first one (see `_layout`).
static func layout() -> Dictionary:
	return _layout

## The current width class (`compact` … `xl`), or "" before the first classification. "" is not a class:
## a caller that needs one decides what to do before boot instead of reading a guess.
static func width_class() -> String:
	return String(_layout.get("width_class", ""))

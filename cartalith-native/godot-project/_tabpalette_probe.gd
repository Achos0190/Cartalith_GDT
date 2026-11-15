extends Node
## Guards the 2026-11-15 decision NOT to apply `Cartalith Tablet.dc.html`'s
## (2026-09-07) palette on the tablet: the newest tablet authority,
## `design/mobile-shell-2026-10-31/Shell.dc.html` (MOBILE_SHELL_PROPOSAL.md; Rulings
## DT, DU, DX, DY), draws the tablet with the desktop's alpha hairlines
## (`--hair rgba(255,255,255,.10)`, `--bord .16`), so no tablet palette exists.
##   godot --headless --path . _tabpalette_probe.tscn
## Asserts LITERALS read off that file and the shell's DARK table, at desktop,
## tablet and phone: every form returns the same colours, none is the old
## canvas's opaque #232628 / #1e2123 / #2c3033. Prints `[RESULT] failures=N`.
var _fail := 0
func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)
func _ready() -> void:
	var forms := {"desktop": [false, false], "tablet": [true, false], "phone": [true, true]}
	for f in forms:
		DccTheme.set_touch(forms[f][0])
		DccTheme.set_phone(forms[f][1])
		DccTheme.set_narrow(false)
		_ok(f + " line (--hair)", DccTheme.c("line"), Color(1, 1, 1, 0.10))
		_ok(f + " line_soft (--div)", DccTheme.c("line_soft"), Color(1, 1, 1, 0.07))
		_ok(f + " border (--bor)", DccTheme.c("border"), Color(1, 1, 1, 0.16))
		_ok(f + " bg (--sur)", DccTheme.c("bg"), Color("#0d0e0f"))
		_ok(f + " panel (--pan)", DccTheme.c("panel"), Color("#121314"))
		_ok(f + " line is not the canvas #232628", DccTheme.c("line") == Color("#232628"), false)
	_ok("planner entry point exists", JourneyPlannerView.new().has_method("open"), true)
	print("[RESULT] failures=", _fail)
	get_tree().quit(1 if _fail > 0 else 0)

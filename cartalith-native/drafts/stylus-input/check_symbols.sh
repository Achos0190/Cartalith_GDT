#!/usr/bin/env bash
# Greps the shipped Godot tree for every SHIPPED symbol the stylus-input drafts
# call. The drafts are not parse-checked (no Godot in the session that wrote
# them), so this is the only automatic check: a missing symbol here means a draft
# calls something that no longer exists under its name. A found symbol does not
# prove the draft is correct, only that the names it depends on are real.
#
# Run from this folder: bash check_symbols.sh
set -u
ROOT="$(cd "$(dirname "$0")/../../godot-project" && pwd)"
if [ ! -d "$ROOT/shell" ]; then
	echo "FAIL: shipped tree not found at $ROOT"
	exit 2
fi

fail=0
# Each line: "grep-pattern<TAB>what it is". The pattern is an extended regex.
check() {
	local pat="$1" what="$2"
	if grep -rqE "$pat" "$ROOT/shell" "$ROOT/project.godot" 2>/dev/null; then
		echo "ok   - $what"
	else
		echo "FAIL - $what  (pattern: $pat)"
		fail=1
	fi
}

# -- 00_stylus_tester.gd --
check 'class_name DccTheme'            'DccTheme (class_name)'
check 'static func c\('                'DccTheme.c(token) — colour accessor'
check '"text":'                        'palette token "text"'
check '"text_secondary":'              'palette token "text_secondary"'
check '"accent":'                      'palette token "accent"'
check '"panel":'                       'palette token "panel"'

echo
echo "Godot built-ins the tester uses (InputEventMouseMotion.pressure/.tilt/.pen_inverted,"
echo "InputEventScreenDrag, InputEventScreenTouch, MOUSE_BUTTON_MASK_LEFT, Time.get_ticks_msec)"
echo "are engine API, not project symbols, and are not grep-checked here."
echo
if [ "$fail" -eq 0 ]; then
	echo "all shipped symbols found"
else
	echo "one or more shipped symbols MISSING — a draft needs updating"
fi
exit "$fail"

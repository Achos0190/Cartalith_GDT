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

# -- 01_dcc_settings_stylus.gd (pastes into dcc_settings.gd; mirrors its pattern) --
check 'class_name DccSettings'         'DccSettings (class_name)'
check 'static func _ensure_loaded\('   'DccSettings._ensure_loaded()'
check 'static func _save\('            'DccSettings._save()'
check 'const _SEC_HANDEDNESS'          'the _SEC_* section pattern (handedness, the model)'
check 'func set_dominant_hand\('       'the validated getter/setter pattern (handedness)'

# -- 10_pressure_curve.gd (edits world_workspace.gd at the two pressure chokepoints) --
check 'func _sculpt_take_pressure\('   'world_workspace _sculpt_take_pressure (sculpt chokepoint)'
check 'func _paint_drag\('             'world_workspace _paint_drag (paint chokepoint)'
check '_sculpt_pressure_on'            'world_workspace _sculpt_pressure_on (the Pressure toggle)'
check 'func sculpt_add_point_p\(|sculpt_add_point_p\('  'bridge.sculpt_add_point_p (pressure-aware sculpt add)'
check 'paint_stroke_sample\('          'bridge.paint_stroke_sample (paint add)'
check '_paint_pending_sample'          'world_workspace _paint_pending_sample (the matched sample)'

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

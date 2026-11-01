#!/usr/bin/env bash
# Re-checks the drafts against the shipped shell before anything is pasted (MISTAKES.md: re-open a row at its
# symbol before believing it; grep `func <name>` before calling a helper).
#   1. every shipped symbol a draft calls still exists (88 at 5bd7af8, 2026-10-31; 93 since the 2026-11-01 menu
#      parity fixes to drafts 05 and 08, none missing at a38e0ac);
#   2. no new name a draft adds is already taken in its target file (none at 5bd7af8);
#   3. the two drawn strings draft 03 replaces are each still there once (added 2026-11-01).
# Run from the repository root:  bash cartalith-native/drafts/mobile-shell/check_symbols.sh
# Prints only problems, then a final line. Works in Git Bash on Windows.
set -u
cd "$(dirname "$0")/../../godot-project/shell" || exit 1
has() { grep -qE "^(static )?(func|var|const|signal) $2\b|^class $2\b" "$1"; }
need() { f=$1; shift; for n in "$@"; do has "$f" "$n" || echo "MISSING in $f: $n"; done; }
free() { f=$1; shift; for n in "$@"; do has "$f" "$n" && echo "TAKEN in $f: $n"; done; }

need dcc_shell.gd _touch _phone _landscape _phone_scale _pscale _phone_bar_cell _pick_bar_domain _session_refuses \
  _toggle_overflow _phone_menu _phone_tab _phone_detent _set_phone_detent _sync_phone_quick_row _refresh_phone_tabs \
  _refresh_phone_sheet_header _refresh_phone_gen_panel _phone_tab_cells _active_domain _phone_root _haptic \
  _apply_phone_orientation _on_tablet_resized phone_scale phone_content_insets phone_insets_changed \
  refresh_phone_quick viewport_content _pfont
need app.gd open_quick_bar_window phone_quick
need dcc_theme.gd PHONE_REF_SHORT c mono mono_label FS_SMALL is_tablet
need dcc_widgets.gd box
need dcc_settings.gd dominant_hand theme_mode
need phone_menu.gd _fill_quick _head _menu_buttons _menu_row _row _root_badge _chevron _push_screen _glyph \
  _status_rows _note_row _value_row _can_recompute_stale _go_recompute_stale _menu_popup _add _missing_row _rest_of \
  _live_root_sub _stack _Step _screen_title _render _fill_story_section is_open close \
  _fill_popup _band _popup_row _clean STORY_ROWS
need quick_bar_phone.gd entries press _list _app
need quick_bar_phone_list.gd _sub_for show_entries
need menus.gd _shell _bridge _host _theme_mode _overflow _file _tablet_world _story _edit _assets _data _preferences \
  _window _help _apply_theme_mode _connect_session_gate
# Draft 08 also calls `_pc_world`, which drafts/pc-shell/02 adds (not shipped, so not checked here;
# drafts/pc-shell/check_symbols.sh checks that the name is still free).

free dcc_theme.gd PHONE_ASPECT_MAX TABLET_MIN_DP BAND_DP BAND_ASPECT WIDTH_CLASSES classify width_class_of _layout \
  set_layout layout width_class
free dcc_shell.gd _layout _user_arg_value _host_px _legacy_density _density _ui_scale _phone_scale_for _apply_unit \
  _unit_factor _booted RESIZE_DEBOUNCE_S _resize_timer _on_resize_settled _request_class_swap DOMAIN_TAB DOMAIN_ORDER \
  DOMAIN_HOLD_MS DOMAIN_SWIPE_UNITS _PhoneDomainFan _domain_fan _dom_gesture _dom_token _build_phone_tab_cell \
  _refresh_domain_tab_caption _on_domain_cell_pressed _on_domain_cell_input _clear_dom_gesture _on_domain_hold \
  _cycle_domain _switch_domain_from_tab _close_domain_fan_for_back set_phone_quick_drawer _close_quick_drawer_for_back
free phone_menu.gd MENU_SCREENS MENU_SUBS MENU_GLYPHS WORLD_SCREENS _fill_status _fill_world open_story
free quick_bar_phone.gd _QuickDrawer _drawer drawer
free menus.gd TABLET_WHOLE_BAR_DP _tablet_bar_whole

# 3. The drawn strings draft 03's EDIT H1 and H2 replace (added 2026-11-01): each must be there exactly once, or the
#    edit has nothing to land on.
anchor() { c=$(grep -cF -- "$2" "$1"); [ "$c" = 1 ] || echo "ANCHOR in $1: found $c times: $2"; }
anchor phone_menu.gd '+ "function is reachable through MAP · GENERATE · PLAN · MORE.")'
anchor dcc_shell.gd '"text": "MAP · GENERATE · PLAN switch tasks here — MORE reaches everything else."},'
echo "check_symbols: done"

#!/usr/bin/env bash
# Re-checks the PC-shell drafts against the shipped shell before anything is pasted (MISTAKES.md:
# re-open a row at its symbol before believing it; grep `func <name>` before calling a helper; never
# quote a count you did not measure). Six checks, the first five pass/fail:
#   1. need    every shipped symbol a draft calls, or names as the thing it relies on, still exists in
#              the file the draft expects (the list was cross-checked against an extraction of every
#              identifier in the drafts' code, comments and strings removed, on 2026-11-01);
#   2. free    no new name a draft adds is taken in its target file (or, for app.gd, its base class);
#   3. anchor  every EDIT's old text is still in its file exactly once, as a whole line;
#   4. base    every block a draft REPLACES or DELETES is still the text it was derived from (a fingerprint
#              of each, taken at a38e0ac on 2026-11-01, `_tablet_world()`'s at 3099309 for Ruling DY); a
#              moved block means re-derive that replacement;
#   5. stub    no `STUB(PC-` marker from 90_pc_milestone_stubs.gd is in the shipped tree;
#   6. report  which milestone methods the host has (so which form each gated row takes), and the
#              key-collision table: every chord menus.gd binds, then each chord PC-9 adds.
# Ruling DY (2026-11-01) added: the names the Records rename, File's Generate world, the Context panel's
# modules and the planner window rely on (1), their new names and ids 920-930 (2), the EDIT W1-W14 anchors
# (3), and the fingerprint of `_tablet_world()`, which 02 deletes (4).
# Run from the repository root:  bash cartalith-native/drafts/pc-shell/check_symbols.sh
# Works in Git Bash on Windows: every read strips CR first, so a CRLF checkout compares like LF.
# Exits 1 when a check in 1-5 failed, else 0. It is a PRE-paste check: after the drafts are applied,
# 2 and 3 report what was pasted (TAKEN, ANCHOR x0) by design, and 4 reports the replaced blocks moved.
set -u
cd "$(dirname "$0")/../../godot-project" || exit 2
fail=0
T=$'\t'
txt() { tr -d '\r' < "$1"; }
n_need=0; n_free=0; n_anchor=0; n_base=0
has() { txt "$1" | grep -qE "^(static )?(func|var|const|signal) $2\b|^class $2\b"; }
need() { f=$1; shift; for n in "$@"; do n_need=$((n_need + 1)); has "$f" "$n" || { echo "MISSING in $f: $n"; fail=1; }; done; }
free() { f=$1; shift; for n in "$@"; do n_free=$((n_free + 1)); has "$f" "$n" && { echo "TAKEN in $f: $n"; fail=1; }; done; }
anchor() { n_anchor=$((n_anchor + 1)); n=$(txt "$1" | grep -cxF -- "$2" || true); [ "$n" = 1 ] || { echo "ANCHOR x$n in $1: $2"; fail=1; }; }
anchor_sub() { n_anchor=$((n_anchor + 1)); n=$(txt "$1" | grep -cF -- "$2" || true); [ "$n" = 1 ] || { echo "ANCHOR x$n in $1 ($3): $2"; fail=1; }; }
block() { txt "$1" | awk -v s="$2" -v e="$3" 'on && index($0, e) == 1 {exit} index($0, s) == 1 {on = 1} on {print}' | cksum | cut -d' ' -f1-2; }
base() { n_base=$((n_base + 1)); got=$(block "$1" "$2" "$3"); [ "$got" = "$4" ] || { echo "BASE MOVED in $1 from '$2': cksum $got, drafted against $4 -- re-read it and re-derive that replacement"; fail=1; }; }

echo "-- 1. need ------------------------------------------------------------------------------------"
need shell/menus.gd build _shell _bridge _host _live _todo _readout _signpost _bind_accelerator shortcut_default \
  _session_gate_row _engine_has _world _tablet_world _on_world _data _on_data _data_route_ids _route_submenu \
  _build_vault_rows _open_timeline _story _story_place _refresh_story_menu _on_story _story_popup \
  _story_political_idx _story_notes_idx _story_campaigns_idx _edit _build_edit_actions _on_edit _window \
  _window_popup _windows_popup _open_windows _refresh_open_windows _on_open_window _layout_readout \
  _build_help_shortcuts_row _help _assets _on_assets WIN_REGION_LABELS WIN_REGION_IDS TIMELINE_TIP \
  PLACES_WITHOUT_NOTES_TIP STORY_POLITICAL_TIP STORY_NOTES_TIP STORY_NO_PLACE STORY_BUSY STORY_NO_WORLD \
  SESSION_MODE_TIP PREF_VALUE_SEP ID_WORLD_RUN_PIPELINE ID_DATA_MANAGER ID_FACTIONS ID_PLACES \
  ID_PLACES_WITHOUT_NOTES ID_WORLD_TIMELINE ID_VAULT ID_VAULT_BROWSE ID_WORLD_DATA ID_EXPORT_MAP_IMAGE \
  ID_DATA_ROUTE_FIRST ID_STORY_TIMELINE ID_STORY_POLITICAL ID_STORY_FACTION_HISTORY ID_STORY_JOURNEY_PLANNER \
  ID_SAVED_JOURNEYS ID_TRAVEL_LIBRARY ID_CONFLICTS_LIST ID_STORY_DRAW_CONFLICT ID_STORY_NOTES_HERE \
  ID_STORY_ASSISTANT ID_WIN_RIGHT ID_HELP_SHORTCUTS ID_PREF_SHORTCUTS ID_IMPORT_IMAGE ID_ASSET_LIBRARY \
  ID_DESELECT ID_FIND_ON_MAP _file _on_file _overflow ID_NEW_WORLD ID_NEW_WORLD_FROM_SELECTION \
  ID_WORLD_MODE_PIPELINE ID_WORLD_MODE_SCULPT ID_WORLD_SEED_SIZE
need shell/app.gd menus viewport context_broker shortcuts_dialog place_editor_window right_dock_ctrl \
  open_faction_roster open_vault open_vault_overview open_vault_browse open_places_without_notes \
  open_assistant_panel open_shortcuts open_travel_library open_journey_planner open_saved_journeys \
  open_conflicts_list open_data_manager open_data_manager_route open_world_data open_export_dialog \
  open_asset_library arm_tool clear_selection is_region_shown toggle_region _world_workspace \
  _open_context_card_from_keyboard _unhandled_key_input redo_last delete_selection _escape_action \
  _backspace_handlers armed_tool open_journey_planner_with_route journey_planner_view
need shell/dcc_shell.gd add_menu style_popup set_status select_domain select_domain_category \
  is_session_mode _touch
need shell/dcc_theme.gd is_phone is_tablet
need shell/dcc_settings.gd shortcut_binding SHORTCUT_CONTEXT_MENU
need shell/dcc_widgets.gd phone_head
need shell/context_broker.gd ring_key_press ring_key_release ring_is_open ring_close ring_click
need shell/engine_bridge.gd generating has_world conflict_campaigns get_civ_year vault_entity_chronos vault_links_for \
  vault_refresh_backlinks
need shell/place_editor_window.gd open_for_tab
need shell/radial_ring.gd release q_release
need shell/popout_host.gd key_for native_windows GROUP
need shell/command_index.gd EXTRAS build _rows _slug _add_menu_commands _disambiguate_pin_keys _walk_popup find_pin \
  _why_not_pinnable _is_dynamic_id search
need shell/shortcuts_dialog.gd UNLISTED setup open open_editable _rebuild _editable _app
need shell/right_dock.gd current_settlement_index _build_settlement _build_sample _build_route CTX_SAMPLE CTX_ROUTE \
  SAMPLE_FIELDS
# Ruling DY: what the module registry (90) and EDITs W8-W14 (02) name, each opened at its symbol on 2026-11-01.
need shell/phone_menu.gd _fill_data _named_in_root _menu_popup
need shell/data_manager_window.gd VAULT_NOTE
need shell/place_editor_window.gd TAB_ORDER _build_timeline _build_political _build_trade _build_economy \
  _build_layout _build_journey_passes _build_backlinks
need shell/faction_roster_window.gd _build_tab_history _build_economy_readout _build_currency _build_tariffs \
  _build_military_block _build_tab_relations _build_faith_group _build_culture_card _build_tab_territory
need shell/workspaces/civilization_workspace.gd garrison_row
need shell/workspaces/world_workspace.gd _regenerate_live
need shell/city_viewer_window.gd open
need shell/vault_window.gd _build_browse_backlinks
need shell/journey_planner_view.gd _show
need shell/dcc_settings.gd _SEC_QUICK_BAR quick_bar_value set_quick_bar_value
need shell/popout_host.gd LIVE_KEYS add_toggle_button prepare supported
for f in shell/saved_journeys_window.gd; do n_need=$((n_need + 1)); [ -f "$f" ] || { echo "MISSING file: $f"; fail=1; }; done
need map_overlay.gd last_mouse_pos request_at

# Ruling DZ (2026-11-01), draft 11: the tool-options bar holds tool settings only. The names it calls or
# deletes, and the WORLD dock's homes for every control it takes off the bar.
need shell/app.gd _tool_options_simple _tool_options_generate _tool_options_stale _tool_options_bake \
  set_bake_shortcut _new_seed _center_landmasses _wire_status set_tool_options
need shell/dcc_theme.gd is_touch
need shell/workspaces/world_workspace.gd _refresh_finalize bake_and_finalize _on_bake_all on_world_changed
echo "-- 2. free ------------------------------------------------------------------------------------"
free shell/menus.gd ID_EDIT_SELECT_BACK ID_EDIT_SELECT_FORWARD ID_EDIT_CONTEXT_CARD ID_EDIT_CARTWHEEL \
  ID_WIN_CTX_INSPECT ID_WIN_CTX_NOTE ID_WIN_CTX_ASK ID_WIN_CTX_PIN ID_WIN_CTX_FOLLOW _host_has \
  _context_tab_ready _typed_into_field cartwheel_keycode RUN_PIPELINE_TIP FACTIONS_TIP FACTIONS_CURRENT_TIP \
  TIMELINE_MOVED_SIGNPOST _open_factions DESELECT_TIP DESELECT_ONE_TIP SELECTION_HISTORY_TODO \
  SELECT_BACK_TIP SELECT_FORWARD_TIP SELECT_BACK_NONE SELECT_FORWARD_NONE CONTEXT_CARD_TIP CARTWHEEL_TIP \
  _deselect_tip _build_selection_rows _build_map_surface_rows _on_edit_selection STORY_NO_SELECTION \
  STORY_BUSY_SELECTION STORY_POLITICAL_SEL_TIP STORY_NOTE_SEL_TIP STORY_ASK_TIP STORY_ASSISTANT_TIP \
  FACTION_HISTORY_TIP FACTION_HISTORY_CURRENT_TIP PLACES_WITHOUT_NOTES_SIGNPOST _story_forms _story_selection \
  _story_political_tip _story_note_tip CONTEXT_PANEL_LABEL CTX_INSPECT_TODO CTX_NOTE_TODO CTX_ASK_TODO \
  CTX_PIN_TODO CTX_FOLLOW_TODO CTX_INSPECT_TIP CTX_NOTE_TIP CTX_ASK_TIP CTX_PIN_TIP CTX_FOLLOW_TIP \
  CTX_PIN_NOTHING OPEN_WINDOW_NATIVE_SUFFIX OPEN_WINDOWS_NONE_TIP _region_label _build_context_panel_menu \
  _on_context_panel _visible_windows _open_window_label HELP_KEYBOARD_REFERENCE_TIP \
  ID_WIN_CTX_MOD_HISTORY ID_WIN_CTX_MOD_ECONOMY ID_WIN_CTX_MOD_MILITARY ID_WIN_CTX_MOD_FAITH \
  ID_WIN_CTX_MOD_TERRITORY ID_WIN_CTX_MOD_CITY ID_WIN_CTX_MOD_HERE ID_WIN_CTX_MOD_ROUTES ID_WIN_CTX_MOD_LINKS \
  ID_WIN_CTX_MOD_RECENT ID_WIN_CTX_RESET _generate_in_world _build_generate_world_row CTX_MODULES \
  CTX_MODULE_TODO CTX_MODULE_TODAY CTX_MODULE_TIPS CTX_RESET_TODO CTX_RESET_TIP CTX_MODULE_REFUSED \
  _context_module_ready _build_context_module_row _refresh_context_modules _context_module_key \
  STORY_PLANNER_TIP STORY_PLANNER_WINDOW_TIP
for f in shell/app.gd shell/dcc_shell.gd; do
  free "$f" open_context_card open_cartwheel_at_pointer selection selection_changed select select_back \
    select_forward current_faction context_tabs open_context_tab pin_context_panel follow_selection \
    context_module_registry context_panel_modules set_context_module_shown reset_context_panel planner_in_window
done
free shell/command_index.gd _has_menu_title
free shell/shortcuts_dialog.gd TITLE_REFERENCE TITLE_EDITOR _phone_title _set_mode_title
n=$(grep -rn --include='*.gd' '"ContextPanelMenu"' shell | wc -l | tr -d ' ')
[ "$n" = 0 ] || { echo "TAKEN: the node name ContextPanelMenu ($n uses in shell/)"; fail=1; }
n=$(grep -rn --include='*.gd' -E ':= (92[0-9]|930)\b' shell | wc -l | tr -d ' ')
[ "$n" = 0 ] || { echo "TAKEN: an id in 920-930, Ruling DY's module rows ($n definitions in shell/)"; fail=1; }

# Ruling DZ, draft 11's two new constants.
free shell/app.gd WORLD_IDLE_HINT_POINTER WORLD_IDLE_HINT_TOUCH
echo "-- 3. anchor ----------------------------------------------------------------------------------"
anchor shell/menus.gd "${T}${T}shell.add_menu(\"World\", _data)"
anchor shell/menus.gd "${T}${T}shell.add_menu(\"☰\", _overflow)"
anchor shell/menus.gd "${T}${T}shell.add_menu(\"World\", _tablet_world)"
anchor shell/menus.gd "${T}_live(p, \"New world…\", ID_NEW_WORLD, KEY_MASK_CTRL | KEY_N)"
anchor shell/menus.gd "${T}## \"World\", not \"Data\", since SM-1 renamed the menu (Ruling CK)."
anchor shell/menus.gd "${T}for line in [\"imports live under World ▸ Import\","
anchor shell/phone_menu.gd "${T}var p := _menu_popup(\"World\")"
anchor shell/phone_menu.gd "${T}if [\"File\", \"World\", \"Story\", \"Preferences\", \"Assets\", \"Help\"].has(menu_title):"
anchor shell/phone_menu.gd "${T}## \"World\" was \"Data\" until SM-1 renamed the menu (Ruling CK); the \`data\`"
anchor_sub shell/phone_menu.gd '"The World menu is not on this build'"'"'s menu bar."' "_fill_data's missing-row text, EDIT W8"
anchor_sub shell/data_manager_window.gd '(World ▸ Markdown vault)' "VAULT_NOTE, EDIT W9"
anchor_sub shell/data_manager_window.gd 'the panel that owns those links is World ▸ Markdown vault.' "EDIT W10"
anchor_sub shell/data_manager_window.gd '"Opens World ▸ Markdown vault, which owns' "EDIT W11"
anchor_sub shell/place_editor_window.gd '"World ▸ Markdown vault… ▸ Index builds it.' "EDIT W12"
anchor_sub shell/workspaces/world_workspace.gd 'Opens World ▸ Import, whose Heightmaps route' "EDIT W13"
anchor_sub shell/journey_planner_view.gd 'party set-ups live in World ▸ Travel library (⇧L)' "EDIT W14"
anchor_sub shell/data_manager_window.gd '"resolution": "World ▸ Travel library ⇧L ▸ %s"' "EDIT W14"
anchor shell/menus.gd "${T}_live(p, \"Markdown vault…\", ID_VAULT)"
anchor shell/menus.gd "${T}${T}ID_FACTIONS: _host.open_faction_roster()"
anchor shell/menus.gd "${T}${T}ID_WORLD_TIMELINE: _open_timeline()"
anchor shell/menus.gd "${T}_live(p, \"Deselect\", ID_DESELECT, KEY_MASK_CTRL | KEY_D)"
anchor shell/menus.gd "${T}${T}\"Clears the settlement, label or icon selection in the active domain. Not the same as Escape, which puts the tool down and deliberately leaves the selection alone.\")"
anchor shell/menus.gd "${T}_live(p, \"Find on map…\", ID_FIND_ON_MAP, KEY_MASK_CTRL | KEY_F)"
anchor shell/menus.gd "${T}${T}_build_reset_generation(p)"
anchor shell/menus.gd "${T}${T}p.add_check_item(String(WIN_REGION_LABELS[rid]), rid)"
anchor shell/menus.gd "${T}${T}var label := String(WIN_REGION_LABELS[rid])"
anchor shell/menus.gd 'var _open_windows: Array = []  ## the live `AcceptDialog`s behind `_windows_popup`, index-parallel to its items'
anchor shell/menus.gd 'const ID_HELP_SHORTCUTS := 72  ## Help ▸ Keyboard shortcuts… (read-only list).'
anchor shell/menus.gd "${T}_live(p, \"Import image…\", ID_IMPORT_IMAGE)"
anchor shell/menus.gd "${T}${T}ID_IMPORT_IMAGE: _host.open_asset_library()"
anchor shell/menus.gd 'const ID_IMPORT_IMAGE := 32  ## Assets ▸ Import image….'
anchor_sub shell/menus.gd 'Window ▸ Workspace ▸ World, and on a tablet World ▸ Run pipeline,' "SESSION_MODE_TIP's clause, EDIT W4"
anchor_sub shell/menus.gd '"Runs the whole generation pipeline -- the same path the WORLD dock' "the Run pipeline tooltip, EDIT W3"
anchor shell/menus.gd "${T}_live(p, \"Run pipeline\", ID_WORLD_RUN_PIPELINE, KEY_MASK_CTRL | KEY_R)"
anchor shell/command_index.gd "${T}for e in EXTRAS:"
anchor shell/command_index.gd 'const EXTRAS: Array = ['
anchor_sub shell/command_index.gd "${T}{\"title\": \"Theme\", \"blurb\": \"Dark, Light or Follow system." "the Theme row, EDIT X1"
anchor shell/shortcuts_dialog.gd "${T}[\"Map view\", \"Delete / Backspace\", \"Remove the selected item\", \"the main window\"],"
anchor shell/shortcuts_dialog.gd "${T}${T}DccWidgets.phone_head(body, \"Keyboard shortcuts\", \"read from the live menus\")"
anchor shell/shortcuts_dialog.gd "${T}_editable = false"
anchor shell/shortcuts_dialog.gd "${T}_editable = true"
anchor shell/dcc_shell.gd "${T}mb.focus_mode = Control.FOCUS_NONE"
anchor shell/app.gd 'func _open_context_card_from_keyboard() -> void:'

# Ruling DZ, draft 11's EDITs T1, T2, T3 and T5 (T4 and T6 delete whole blocks, fingerprinted in 4).
anchor shell/app.gd "## The tool-options bar's staleness label; text is \`generating…\` while the engine is busy and empty afterwards."
anchor shell/app.gd 'var _tool_options_stale: Label'
anchor shell/app.gd 'var _tool_options_bake: Button'
anchor shell/app.gd "${T}${T}${T}_tool_options_stale.text = \"generating…\")"
anchor shell/app.gd "${T}${T}${T}_tool_options_stale.text = \"\""
anchor shell/workspaces/world_workspace.gd "${T}## The tool-options bar's copy mirrors this one rather than recomputing it."
anchor shell/workspaces/world_workspace.gd "## tool bar's bake shortcut. Safe before \`_build_finalize\` ran (returns when"
echo "-- 4. base ------------------------------------------------------------------------------------"
base shell/menus.gd 'func _data(p: PopupMenu) -> void:' '## Tooltip of both Timeline rows' '687853559 2392'
base shell/menus.gd "## The World menu's dispatcher, shared by the popup and its three route" "## Both Timeline rows' action:" '1605145114 1286'
base shell/menus.gd "## The tablet's single World menu since SM-1" "# -- Story (SM-2, Ruling CK)" '1819389796 544'
base shell/menus.gd '## **The Story menu** (`STORY_MENUS_PROPOSAL.md` §3.3, Ruling CK, owner' '## Tooltip of Political history of this place… while a place is current.' '1034846636 6199'
base shell/menus.gd "## Refreshes the Story menu's live rows:" "## The Story menu's dispatcher." '3278626836 1981'
base shell/menus.gd "## The Story menu's dispatcher." "## **The Markdown vault's program-scope entry point**" '4187067950 1357'
base shell/menus.gd '## `_host` is `DccApp` (`app.gd`); these **six** fields are its own public' '# -- §2.7 Help' '3689053380 1988'
base shell/menus.gd '## The one-line "Keyboard shortcuts…" row' '## `full=false` on the tablet omits Keyboard shortcuts and About' '2144072409 381'
base shell/app.gd '## §4.5.6: "Escape commits an in-progress multi-click tool... and otherwise' '## Delete whatever is selected, in whichever workspace owns a selection.' '1473528590 6963'

# Ruling DZ, draft 11: the blocks it replaces or deletes, fingerprinted at 4f3d904 (2026-11-01).
base shell/app.gd "## §4's Generation Pipeline row, in its specified order" '## Installs a minimal tool-options row' '3897334673 3647'
base shell/app.gd '## Called by `world_workspace._refresh_finalize()`, the single owner of whether' "## Rewrites the status bar's" '20495316 464'
base shell/workspaces/world_workspace.gd "## The tool-options bar's copy of this control presses exactly this" '## Re-syncs the Finalize section from' '123075827 363'
echo "-- 5. stub ------------------------------------------------------------------------------------"
n=$(grep -rln --include='*.gd' 'STUB(PC-' . | grep -v '^\./\.godot/' | wc -l | tr -d ' ')
[ "$n" = 0 ] || { echo "STUB marker in $n shipped file(s): a 90_pc_milestone_stubs.gd placeholder was pasted, and the rows it gates are live over it"; fail=1; }

echo "-- 6a. forms (which form each gated row will build on the next boot) --------------------------"
for m in select_back select_forward current_faction context_tabs open_context_tab pin_context_panel follow_selection \
    context_module_registry context_panel_modules set_context_module_shown reset_context_panel planner_in_window; do
  if has shell/app.gd "$m" || has shell/dcc_shell.gd "$m"; then s=present; else s=absent; fi
  printf '  %-24s %s\n' "$m" "$s"
done
echo "  (context_tabs present says nothing about WHICH tabs: read its body for inspect / note / ask;"
echo "   context_module_registry present says nothing about WHICH modules: read it for the ten keys)"

echo "-- 6b. keys: every chord menus.gd binds (code lines only), then each chord PC-9 adds -----------"
chords() {
  txt shell/menus.gd | grep -n -E '(_live|_bind_accelerator)\(.*KEY_[A-Z0-9_]+' \
    | grep -v -E '^[0-9]+:[[:space:]]*##' | grep -v 'accel: Key = KEY_NONE' \
    | awk -F: '{ line = $1; rest = substr($0, length($1) + 2)
        id = ""; if (match(rest, /ID_[A-Z0-9_]+/)) id = substr(rest, RSTART, RLENGTH)
        acc = rest; sub(/^.*, */, "", acc); sub(/\).*$/, "", acc)
        m = ""; if (acc ~ /KEY_MASK_CTRL/) m = m "Ctrl+"; if (acc ~ /KEY_MASK_SHIFT/) m = m "Shift+"; if (acc ~ /KEY_MASK_ALT/) m = m "Alt+"
        k = acc; sub(/^.*KEY_/, "", k); gsub(/[[:space:]]/, "", k)
        printf "%s\t%s\tmenus.gd:%s\n", m k, id, line }'
}
chords | sort | awk -F'\t' '{ printf "  %-14s %-28s %s\n", $1, $2, $3 }'
echo "  ($(chords | wc -l | tr -d ' ') bindings)"
echo "  new chord      intended command              state"
while read -r chord want; do
  got=$(chords | awk -F'\t' -v c="$chord" '$1 == c { print $2 }' | sort -u | tr '\n' ' ')
  if [ -z "$got" ]; then state="free"
  elif [ "$got" = "$want " ]; then state="bound to its own command (expected once its row is pasted)"
  else state="COLLISION: bound to $got"; fail=1; fi
  printf '  %-14s %-28s %s\n' "$chord" "$want" "$state"
done <<'EOF'
Ctrl+R ID_WORLD_RUN_PIPELINE
Ctrl+Shift+F ID_FACTIONS
Ctrl+Shift+V ID_VAULT
Alt+LEFT ID_EDIT_SELECT_BACK
Alt+RIGHT ID_EDIT_SELECT_FORWARD
Ctrl+1 ID_WIN_CTX_INSPECT
Ctrl+2 ID_WIN_CTX_NOTE
Ctrl+3 ID_WIN_CTX_ASK
Ctrl+J ID_STORY_ASSISTANT
Shift+F10 ID_EDIT_CONTEXT_CARD
Q ID_EDIT_CARTWHEEL
EOF
echo "  every other match of those keys in a shipped script (not menus.gd's bindings, not comments):"
find . -name '*.gd' -not -path './.godot/*' -not -name '_*' -print0 \
  | xargs -0 grep -n -E 'KEY_(R|F|V|J|Q|1|2|3|LEFT|RIGHT|F10|MENU)\b' \
  | grep -v -E '^[^:]+:[0-9]+:[[:space:]]*##' | grep -v -E '^\./shell/menus\.gd:[0-9]+:.*(_live|_bind_accelerator)\(' \
  | sed 's/^/    /' | cut -c1-150
echo "  tool-button letters (exact-match Shortcuts, dcc_widgets.gd::_tool_shortcut): $(find . -name '*.gd' -not -path './.godot/*' -not -name '_*' -print0 | xargs -0 grep -o -h -E '"label": *"[^"]* \((⇧)?[A-Z]\)' | sed -E 's/.*\(((⇧)?[A-Z])\)$/\1/' | sort -u | tr '\n' ' ')+ Freehand's F (world_workspace.gd)"
echo "  project.godot [input] actions: $(grep -c '^\[input\]' project.godot || true) (0 means only Godot's ui_* defaults and the Layers popover's runtime 1-8)"

echo "check_symbols: need $n_need, free $n_free, anchor $n_anchor, base $n_base checked; $([ $fail = 0 ] && echo 'all passed' || echo 'FAILURES above')"
exit $fail

## DRAFT, not loaded by Godot (this folder is outside `godot-project/`). NOT parse-checked: the session
## that wrote it had no Godot binary.
##
## PC_SHELL_PROPOSAL.md §3.8 and §3.9, milestone PC-9 (menus and keys): the ids and the four helpers
## every later draft in this folder calls. Nothing here draws a row, so pasting it changes nothing on
## screen.
##
## Target: `shell/menus.gd` (`DccMenus`). ADD, do not replace:
##   - the twenty `ID_*` constants: beside `ID_PREF_POPOUT` (the last id block in the file): 910-918 for
##     PC-9's rows, and 920-930 for the Context panel's modules and its reset (Ruling DY, 2026-11-01);
##   - the four functions: beside `_engine_has()`, after `_signpost()`.
##
## Depends on: nothing. Apply it first.
##
## Forms (how every later draft decides which row to draw). A row whose behaviour needs a milestone that
## has not landed is built with `_todo()` and its reason, and a row the shipped build already honours
## keeps its shipped form, until the host has grown the method the new form calls. The question is
## asked once, at build time, through `_host_has()` (PC-1's methods) and `_context_tab_ready()` (the
## Context panel's tabs, PC-4/5/6). Menus are built once per boot, so a milestone that lands shows its
## rows from the next start. The contract for every method asked about is
## `90_pc_milestone_stubs.gd`, which must not be pasted before its milestone (its header says why).
##
## Verify after pasting: `godot --headless --path cartalith-native/godot-project --check-only --script
## res://shell/menus.gd` (from the repository root; `MISTAKES.md`, "Run --check-only"), and the same for
## `res://shell/app.gd`.

## PC-9's rows (`PC_SHELL_PROPOSAL.md` §3.8, §3.9; mockup `design/mobile-shell-2026-10-31/Shell.dc.html`'s
## desktop menus). 910-918 were unused, measured 2026-11-01 at `a38e0ac`: no `ID_` constant in this
## file is above 903 (`grep -n -E '^const ID_[A-Z_0-9]+ := 9[0-9]{2}' shell/menus.gd` lists 900-903
## only), `grep -rn -E ':= 91[0-9]\b' shell/` finds nothing, and none of the generated spans that
## `command_index.gd::_is_dynamic_id()` names reaches 900 (the highest is the layouts' 640-671).
## Each is a command identity: a rebind (`DccSettings.shortcut_binding`) and a quick-bar pin
## (`menu:<id>`) are keyed by it, so none may ever be reissued to a different command.
const ID_EDIT_SELECT_BACK := 910  ## Edit ▸ Previous selection (Alt+←); live from PC-1.
const ID_EDIT_SELECT_FORWARD := 911  ## Edit ▸ Next selection (Alt+→); live from PC-1.
const ID_EDIT_CONTEXT_CARD := 912  ## Edit ▸ Context card (Shift+F10); the key moved here from `app.gd` (§3.9, M8).
const ID_EDIT_CARTWHEEL := 913  ## Edit ▸ Cartwheel at the pointer (Q); the key moved here from `app.gd` (§3.9, M8).
const ID_WIN_CTX_INSPECT := 914  ## Window ▸ Context panel ▸ Inspect (Ctrl+1); live from PC-4.
const ID_WIN_CTX_NOTE := 915  ## Window ▸ Context panel ▸ Note (Ctrl+2); live from PC-5.
const ID_WIN_CTX_ASK := 916  ## Window ▸ Context panel ▸ Ask (assistant) (Ctrl+3); live from PC-6.
const ID_WIN_CTX_PIN := 917  ## Window ▸ Context panel ▸ Pin to the current subject; live from PC-4.
const ID_WIN_CTX_FOLLOW := 918  ## Window ▸ Context panel ▸ Follow the selection; live from PC-4.

## Ruling DY (owner, 2026-11-01; `PC_SHELL_PROPOSAL.md` PQ1 and its follow-up): the Context panel is modular,
## "tabs you arrange", and these are the rows of Window ▸ Context panel ▸ MODULES, one per module the owner
## chose, plus Reset panel (`05_menus_window.gd::CTX_MODULES`). 919 is mobile-shell draft 09's
## (`ID_HELP_GESTURES`). 920-930 were unused, measured 2026-11-01 at `3099309`: `grep -n -E '^const
## ID_[A-Z_0-9]+ := 9[0-9]{2}' shell/menus.gd` still lists 900-903 only, `grep -rn -E ':= 9[2-3][0-9]\b'
## shell/ drafts/` finds nothing, and `command_index.gd::_is_dynamic_id()`'s spans end below 700. The same
## rule as 910-918: a command identity, never reissued. Each module row is a check row once its module is built,
## so `command_index.gd::_why_not_pinnable()` leaves it unpinnable ("A check or radio row …"); the ids are
## still what a rebind is keyed by.
const ID_WIN_CTX_MOD_HISTORY := 920  ## Window ▸ Context panel ▸ MODULES ▸ History (check); live with its module.
const ID_WIN_CTX_MOD_ECONOMY := 921  ## … ▸ Economy (check); live with its module.
const ID_WIN_CTX_MOD_MILITARY := 922  ## … ▸ Military (check); live with its module.
const ID_WIN_CTX_MOD_FAITH := 923  ## … ▸ Faith, culture & relations (check); live with its module.
const ID_WIN_CTX_MOD_TERRITORY := 924  ## … ▸ Territory (check); live with its module.
const ID_WIN_CTX_MOD_CITY := 925  ## … ▸ City layout (check); live with its module.
const ID_WIN_CTX_MOD_HERE := 926  ## … ▸ Here (check); live with its module.
const ID_WIN_CTX_MOD_ROUTES := 927  ## … ▸ Routes & journeys (check); live with its module.
const ID_WIN_CTX_MOD_LINKS := 928  ## … ▸ Links (check); live with its module.
const ID_WIN_CTX_MOD_RECENT := 929  ## … ▸ Recent & bookmarks (check); live with its module.
## Shipped as 934 (2026-11-07): MS-1 took 930-933 for Preferences ▸ UI scale first.
const ID_WIN_CTX_RESET := 934  ## Window ▸ Context panel ▸ Reset panel (back to Inspect · Note · Ask); live from PC-4.

## Whether the host (`DccApp`) has `method`, which is how a row tells whether the milestone that adds
## it has landed. The same question `_on_world()` and `_on_edit()` already ask before calling a host
## method another pass owns (`_host.has_method("open_find_on_map")`), named so that every form
## decision in PC-9 reads alike.
##
## It answers for the METHOD, not for the work behind it: a placeholder body pasted early makes it
## true. That is why `90_pc_milestone_stubs.gd` is a contract that is not pasted, and why
## `check_symbols.sh` fails while any `STUB(PC-` marker is in the shipped tree. Must never be used to
## hide a row: a row whose method is missing is built with `_todo()` and its reason, never left out.
func _host_has(method: String) -> bool:
	return _host != null and _host.has_method(method)

## Whether the Context panel has the tab `tab` ("inspect", "note" or "ask") in this build:
## `DccApp.context_tabs()` lists the tabs that exist, and PC-4, PC-5 and PC-6 each add one. Asked
## per tab rather than "has the panel landed", because the three tabs land in three milestones and a
## row for the Note tab must stay a `_todo` while only Inspect exists.
##
## Must never infer a tab from the panel's other methods: `open_context_tab()` existing says nothing
## about which tabs it can open.
func _context_tab_ready(tab: String) -> bool:
	if not _host_has("context_tabs"):
		return false
	var tabs: PackedStringArray = _host.context_tabs()
	return tabs.has(tab)

## True when a text field has focus AND row `id`'s own key is down, i.e. this press came from the
## row's accelerator while the user was typing, not from a click on the row.
##
## Why: Q and Shift+F10 used to be matched in `app.gd::_unhandled_key_input`, behind a guard that
## ignored them while a `LineEdit`, `TextEdit` or `SpinBox` had focus. As menu accelerators they run
## from `_shortcut_input`, after the focused control has had the key. A field consumes a printable Q
## itself, but nothing guarantees it consumes Shift+F10, so the guard moves into the handler. Alt+← and
## Alt+→ (Edit ▸ Previous / Next selection) take the same guard: Ruling DX, PQ12, says they must not act
## while a text field has focus, and on Windows and Linux a field lets them through. Asking
## the focus alone would also swallow a mouse click on the row while a field keeps focus (`MenuButton`
## takes no focus), hence the key test. `Input.is_key_pressed` is true for the press being dispatched.
##
## Reads the binding through `DccSettings.shortcut_binding`, so a rebound key is the one tested.
## Must never be used on a row whose key a text field needs (Ctrl+C, Delete): those rely on the field
## consuming the key first, which this does not replace.
##
## **A probe cannot reach the guard through `Viewport.push_input()`**: `Input` learns of a key only
## from `Input.parse_input_event()`, which real key presses go through and `push_input()` skips, so a
## pushed press reads as "not held" and the row acts. A probe of the guard must use
## `Input.parse_input_event()`.
func _typed_into_field(id: int) -> bool:
	if _host == null:
		return false
	var focus := (_host as Node).get_viewport().gui_get_focus_owner()
	if not (focus is LineEdit or focus is TextEdit or focus is SpinBox):
		return false
	var accel := DccSettings.shortcut_binding(DccSettings.SHORTCUT_CONTEXT_MENU, id, shortcut_default(id))
	if accel == KEY_NONE:
		return false
	return Input.is_key_pressed((accel & KEY_CODE_MASK) as Key)

## The key the cartwheel answers to, without modifiers: Edit ▸ Cartwheel at the pointer's binding (Q
## until the user rebinds it). `app.gd::_unhandled_key_input` matches the key-UP of the cartwheel's
## hold-aim-release gesture against it, because a menu accelerator fires on the press only.
##
## The default comes from `shortcut_default()`, which only `_bind_accelerator` fills, so the
## `KEY_Q` written on the row is the single source. Returns `KEY_NONE` where the row is not built
## (the phone, `03_menus_edit.gd`), which matches no real key-up.
func cartwheel_keycode() -> Key:
	var accel := DccSettings.shortcut_binding(DccSettings.SHORTCUT_CONTEXT_MENU,
		ID_EDIT_CARTWHEEL, shortcut_default(ID_EDIT_CARTWHEEL))
	return (accel & KEY_CODE_MASK) as Key

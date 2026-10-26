extends RefCounted
class_name QuickBarPhone

## The phone's quick bar: where a pinned command lives on a handset
## (`CUSTOM_QUICK_BAR_SCOPE.md` s.3.2, milestone CQ-6, Ruling DH Q9: "both" -- a chip row in the MAP sheet AND
## a Quick section in MORE). The phone has no tool palette bar and no cartwheel, so neither of CQ-2's and
## CQ-4's surfaces exists there; this is the whole of the phone's form, and it reads the SAME `[quick_bar]`
## store (`DccSettings`, key `bar`) and the same Assistant rule (`AssistantChip`) the other two forms do.
##
## ## A pin is still a pointer (scope s.0 rule 1)
##
## Nothing here knows what any command does. A press calls `DccQuickBar.run_pin_on()`, which resolves the
## stored `pin_key` against a fresh `CommandIndex` and emits the owning popup's own `id_pressed` -- the
## signal a menu click emits. A pin whose command is gone is kept and shown as unavailable with a stated
## reason; it is never dropped and never re-pointed.
##
## ## The three surfaces this controller feeds
##
##  * [`QuickBarPhoneRow`] -- the chip row at the top of the MAP sheet's content, shown from the HALF
##    detent up (the 66 dp peek carries only what it carried before). Mounted into the sheet by
##    `DccShell.set_phone_quick_row`.
##  * MORE's root screen -- a `Quick` section first in the list (`phone_menu.gd::_fill_quick`), built from
##    the same [`entries`].
##  * [`QuickBarPhoneList`] -- the full-height list the row's last chip, `All...`, opens: every pin, 52 dp
##    rows, with the reason on every unavailable one and a `Customise...` row.
##
## ## Nothing exists until it is needed (the pixel-identity bar)
##
## With an EMPTY quick bar and the assistant off, [`entries`] is empty and **no node is created, no row is
## mounted, no section is drawn**: the phone's screens are pixel-identical to the tree before this
## milestone (`_quickbar_phoneid_probe.gd`, measured). This is deliberate and not only tidy: CQ-4 and CQ-7
## each found that creating controls, or connecting a signal, in a `setup()` that ran at boot moved 26 014
## pixels of the phone's Cartography sheet, for a window the phone never opened (scope s.14 and s.15; the
## cause was never traced). So this controller's `setup()` only remembers the app; the row, the list
## sheet and the hub connection are created on first need.
##
## The one way to a FIRST pin on an empty bar is therefore `MORE > Preferences > Quick bar...`
## (`menus.gd`, no longer hidden from the phone), or a star on a command in the phone's search. An empty
## row with only a `+` chip would be the scope's "one small + chip", but the owner's CQ-6 bar is that the
## phone be pixel-identical with nothing pinned, which a permanent chip cannot be.

## A chip's height and minimum width in dp: Android's 48 dp (`DESIGN_HANDOFF.md`: "take the larger of the
## two platform minimums"). The hard floor under it is `DccTheme.PHONE_TAP_MIN` (44), enforced by `_ptap`.
const CHIP_DP := 48.0
## The chip row's inset and gaps, in dp (the MAP sheet's own 14 dp side padding; a labelled judgement for
## the rest: 6 above and 8 below keep the row clear of the header and the tool options under it).
const ROW_SIDE_DP := 14.0
const ROW_TOP_DP := 6.0
const ROW_BOTTOM_DP := 8.0
const ROW_GAP_DP := 8.0
## The label of the row's last chip. It opens [`QuickBarPhoneList`].
const ALL_LABEL := "All…"

var _app
var _row: QuickBarPhoneRow
var _list: QuickBarPhoneList

## Remembers the app and follows the Assistant rule's inputs (module header). No node is created.
func setup(app) -> void:
	_app = app
	_ensure_hub()

## What the phone should draw right now, in order: the Assistant chip when its rule shows it, then the
## stored pins. Each entry is `{kind: "assistant" | "pin", key, title, label, live, reason, group}`.
## `live` is whether a tap runs it; `reason` is the sentence for a tap that cannot (empty when live);
## `group` is the menu the command lives in (empty for the Assistant). A fresh `CommandIndex` is built
## only when there are pins to resolve. Static so MORE's section and the probes read the same answer.
static func entries(app) -> Array:
	var out: Array = []
	if app == null:
		return out
	var st := AssistantChip.state_for_app(app)
	if String(st["state"]) != AssistantChip.HIDDEN:
		out.append({"kind": "assistant", "key": "", "title": AssistantChip.LABEL,
			"label": AssistantChip.LABEL, "live": String(st["state"]) == AssistantChip.LIVE,
			"reason": "" if String(st["state"]) == AssistantChip.LIVE else AssistantChip.tooltip_for(st),
			"group": ""})
	var pins: Array = DccSettings.quick_bar_value("bar")
	if pins.is_empty():
		return out
	var idx := CommandIndex.new()
	idx.build(app, app.get("bridge"))
	for p in pins:
		out.append(pin_entry(idx.find_pin(String(p["k"])), String(p["k"]), String(p["t"])))
	return out

## The entry for one stored pin against its live index row (`{}` when the command is gone).
## Mirrors `DccQuickBar._chip_for`: live exactly when the index calls the row pinnable now; a missing row
## keeps the title it had when pinned and says it is not in this build.
static func pin_entry(row: Dictionary, pin_key: String, stored_title: String) -> Dictionary:
	var title := String(row.get("title", "")) if not row.is_empty() else stored_title
	if title == "":
		title = pin_key
	var live := not row.is_empty() and bool(row.get("pinnable", false))
	var reason := ""
	if row.is_empty():
		reason = "Not in this build. The pin is kept; remove it in Customise."
	elif not live:
		reason = "Unavailable: " + String(row.get("why_not", ""))
	return {"kind": "pin", "key": pin_key, "title": title, "label": DccQuickBar.label_for(title),
		"live": live, "reason": reason, "group": String(row.get("group", ""))}

## One tap on an entry. A live Assistant entry opens the panel; a live pin runs its command; anything
## else (a pin that cannot run, a greyed Assistant) says why in a toast, so a tap is never silent
## (`near` is where the toast appears; the status bar is parked on the phone, so a status line would
## never be seen). Static so MORE and the list sheet press through the same code as the row.
static func press(app, entry: Dictionary, near: Control = null) -> void:
	if app == null:
		return
	if not bool(entry.get("live", false)):
		say(app, String(entry.get("reason", "")), near)
		return
	if String(entry.get("kind", "")) == "assistant":
		if not AssistantChip.open_panel(app):
			say(app, AssistantChip.tooltip_for(AssistantChip.state_for_app(app)), near)
		return
	var why := DccQuickBar.run_pin_on(app, String(entry.get("key", "")))
	if why != "":
		say(app, why, near)

## A short sentence on the phone: the shell's toast, or the status line where there is no toast.
static func say(app, text: String, near: Control = null) -> void:
	if text == "":
		return
	if app != null and app.has_method("phone_toast"):
		app.phone_toast(text, near)
	else:
		DccQuickBar.say_on(app, text)

# -- the controller --------------------------------------------------------------------

## The mounted chip row, or null while nothing is wanted (for tests).
func row() -> QuickBarPhoneRow:
	return _row

## The `All...` list sheet, or null until it was first opened (for tests).
func list_sheet() -> QuickBarPhoneList:
	return _list

## Brings the phone's chip row up to date with the store and the Assistant rule. Does NOTHING when there is
## nothing to show and nothing is mounted (the identity bar); mounts the row on the first need; unmounts and
## frees it when the last entry goes. Called after a world load, a generation, a pin change and a change to
## any setting the Assistant chip reads. Safe on a non-phone app: it returns at once.
func refresh() -> void:
	if _app == null or not _app.is_phone():
		return
	var es := entries(_app)
	if es.is_empty():
		if _row != null:
			_app.set_phone_quick_row(null)
			_row.queue_free()
			_row = null
		return
	if _row == null:
		_row = QuickBarPhoneRow.new()
		_row.setup(_app, self)
		_app.set_phone_quick_row(_row)
	_row.set_entries(es)
	if _list != null and _list.visible:
		_list.show_entries(es)
	_app.refresh_phone_quick()

## Connects [`refresh`] to `DccSettings.assistant_hub`, once. A bare signal connection to a RefCounted creates
## no node; whether even that moves the phone's pixels at boot is measured by `_quickbar_phoneid_probe.gd`.
func _ensure_hub() -> void:
	if not DccSettings.assistant_hub.changed.is_connected(refresh):
		DccSettings.assistant_hub.changed.connect(refresh)

## Opens the `All...` list sheet on the current entries.
func open_list() -> void:
	if _app == null:
		return
	if _list == null:
		_list = QuickBarPhoneList.new()
		_app.add_child(_list)
		_list.setup(_app)
	_list.open(entries(_app))

extends RefCounted
class_name QuickBarPhone

## The phone's quick bar: where a pinned command lives on a handset
## (`CUSTOM_QUICK_BAR_SCOPE.md` s.3.2, milestone CQ-6, Ruling DH Q9, and MS-11 of `MOBILE_SHELL_PROPOSAL.md` s.2.4,
## Ruling DY Q10 to Q12). The phone has no tool palette bar and no cartwheel, so neither of CQ-2's and CQ-4's
## surfaces exists there; this is the whole of the phone's form, and it reads the SAME `[quick_bar]` store
## (`DccSettings`, key `bar`) and the same Assistant rule (`AssistantChip`) the other two forms do.
##
## ## A pin is still a pointer (scope s.0 rule 1)
##
## Nothing here knows what any command does. A press calls `DccQuickBar.run_pin_on()`, which resolves the
## stored `pin_key` against a fresh `CommandIndex` and emits the owning popup's own `id_pressed` -- the
## signal a menu click emits. A pin whose command is gone is kept and shown as unavailable with a stated
## reason; it is never dropped and never re-pointed.
##
## ## The surfaces this controller feeds
##
##  * the side drawer (`quick_drawer.gd`, MS-11) -- a grip at the map band's edge on the side away from the
##    dominant hand that opens a list of every entry. It REPLACED CQ-6's chip row in the MAP sheet
##    (`QuickBarPhoneRow`, deleted with its `set_phone_quick_row` hooks in the shell). Mounted over the map by
##    `DccShell.set_phone_quick_drawer`.
##  * MENU's root screen -- a `Quick` section first in the list (`phone_menu.gd::_fill_quick`), built from
##    the same [`entries`].
##  * [`QuickBarPhoneList`] -- the full-height list sheet. Nothing on the phone opens it now that the chip row's
##    `All...` chip is gone (the drawer lists every pin itself); it is kept for the class-swap probe and is a
##    candidate for deletion (`OUTSTANDING_WORK.md`).
##
## ## Nothing exists until it is needed (the pixel-identity bar)
##
## With an EMPTY quick bar and the assistant off, [`entries`] is empty and **no node is created, no drawer is
## mounted, no section is drawn, and no grip is drawn**: the phone's screens are pixel-identical to the tree
## before this milestone (`_quickbar_phoneid_probe.gd`, measured). This is deliberate and not only tidy: CQ-4 and
## CQ-7 each found that creating controls, or connecting a signal, in a `setup()` that ran at boot moved 26 014
## pixels of the phone's Cartography sheet, for a window the phone never opened (scope s.14 and s.15; the
## cause was never traced). So this controller's `setup()` only remembers the app; the drawer, the list sheet
## and the hub connection are created on first need.
##
## The one way to a FIRST pin on an empty bar is therefore `MENU > Preferences > Quick bar...`
## (`menus.gd`, no longer hidden from the phone), or a star on a command in the phone's search.
##
## ## Across a class swap (MS-1 S4)
##
## The drawer is a child of the phone's composition, which a swap frees. [`represent_capture`] reads whether it is
## open and lets go of it (giving back the world-pause depth it holds) BEFORE the unmount; [`represent_restore`]
## builds it again for the new class from the stored pins and, if it was open, opens it again. This controller
## itself is the app's, so it survives.

var _app
## A phone pin control's size in dp: Android's 48 dp (`DESIGN_HANDOFF.md`: "take the larger of the two platform
## minimums"); the hard floor under it is `DccTheme.PHONE_TAP_MIN` (44), enforced by `_ptap`. Today the one reader is the
## search result's pin star (`dcc_shell.gd`); the chip row that gave it its name is gone (MS-11).
const CHIP_DP := 48.0

## The drawer, loaded by path (no `class_name`, so no class-cache entry is needed).
const _QuickDrawer := preload("res://shell/quick_drawer.gd")
## The mounted drawer, or null while there is nothing to show (that null is what keeps an empty bar drawing nothing).
var _drawer: Control
var _list: QuickBarPhoneList

## Remembers the app and follows the Assistant rule's inputs (module header). No node is created.
func setup(app) -> void:
	_app = app
	_ensure_hub()
	## CQ-8: the row follows the active domain. A bare signal connection creates no node (the identity bar
	## above); `refresh()` itself does nothing while nothing is pinned and nothing is mounted.
	if app != null and app.has_signal("workspace_changed") and not app.workspace_changed.is_connected(_on_workspace):
		app.workspace_changed.connect(_on_workspace)

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
	## CQ-8: the pins shown in the active domain; no known domain, no filter.
	var dom := String(app.active_domain()) if app.has_method("active_domain") else ""
	var pins: Array = DccSettings.quick_bar_pins_for("bar", dom)
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
	var label := DccQuickBar.label_for(title)
	## CQ-8: a toggle pin shows its state as the desktop chip does, read from the row's getter when the entry is
	## built (the phone redraws on every refresh: a pin change, a workspace change, a world load).
	var toggle := live and bool(row.get("toggle", false)) and (row.get("_state") as Callable).is_valid()
	if toggle:
		label = (DccQuickBar.MARK_ON if bool((row["_state"] as Callable).call()) else DccQuickBar.MARK_OFF) + label
	return {"kind": "pin", "key": pin_key, "title": title, "label": label,
		"live": live, "reason": reason, "group": String(row.get("group", "")), "toggle": toggle}

## One tap on an entry. A live Assistant entry opens the panel; a live pin runs its command; anything
## else (a pin that cannot run, a greyed Assistant) says why in a toast, so a tap is never silent
## (`near` is where the toast appears; the status bar is parked on the phone, so a status line would
## never be seen). Static so MORE and the list sheet press through the same code as the row.
static func press(app, entry: Dictionary, near: Control = null) -> void:
	if app == null:
		return
	if not bool(entry.get("live", false)):
		## Owner decision 2026-10-26 (OnePlus 6T pass): the phone's greyed Assistant chip is a way IN, not a dead
		## button. When the gate refuses it takes the user to where the missing piece is fixed and says what
		## that is: the panel (which presents the consent card) for `consent_needed`, the Assistant settings for
		## everything else. Desktop and tablet chips keep CQ-7's rule (disabled, with the reason as tooltip):
		## they have a tooltip and a status bar to explain a dead control, the phone has neither.
		if String(entry.get("kind", "")) == "assistant":
			var st := AssistantChip.state_for_app(app)
			if String(st["state"]) == AssistantChip.DISABLED:
				if String(st.get("code", "")) == "consent_needed" and app.has_method("open_assistant_panel"):
					app.open_assistant_panel()
				elif app.has_method("open_assistant_settings"):
					app.open_assistant_settings()
		say(app, String(entry.get("reason", "")), near)
		return
	if String(entry.get("kind", "")) == "assistant":
		if not AssistantChip.open_panel(app):
			say(app, AssistantChip.tooltip_for(AssistantChip.state_for_app(app)), near)
		return
	var why := DccQuickBar.run_pin_on(app, String(entry.get("key", "")))
	if why != "":
		say(app, why, near)
	elif bool(entry.get("toggle", false)):
		## CQ-8: a toggle's chip must show the new state at once; the phone redraws on refresh, not on a poll.
		var q = app.get("phone_quick")
		if q != null:
			q.refresh.call_deferred()

## A short sentence on the phone: the shell's toast, or the status line where there is no toast.
static func say(app, text: String, near: Control = null) -> void:
	if text == "":
		return
	if app != null and app.has_method("phone_toast"):
		app.phone_toast(text, near)
	else:
		DccQuickBar.say_on(app, text)

# -- the controller --------------------------------------------------------------------

## The mounted drawer, or null while nothing is wanted (for tests).
func drawer() -> Control:
	return _drawer

## The `All...` list sheet, or null until it was first opened (for tests).
func list_sheet() -> QuickBarPhoneList:
	return _list

## Brings the phone's drawer up to date with the store and the Assistant rule. Does NOTHING when there is
## nothing to show and nothing is mounted (the identity bar); mounts the drawer on the first need; frees it when
## the last entry goes (releasing the world-pause depth first if it was open). Called after a world load, a
## generation, a pin change and a change to any setting the Assistant chip reads. Safe on a non-phone app: it
## returns at once.
func refresh() -> void:
	if _app == null or not _app.is_phone():
		return
	var es := entries(_app)
	if es.is_empty():
		_free_drawer()
		return
	if _drawer == null or not is_instance_valid(_drawer):
		_drawer = _QuickDrawer.new()
		_drawer.call("setup", _app)
		_app.set_phone_quick_drawer(_drawer)
	_drawer.call("set_entries", es)
	_drawer.call("relayout")
	if _list != null and _list.visible:
		_list.show_entries(es)

## Gives back the drawer's pause depth and frees it. Idempotent.
func _free_drawer() -> void:
	if _drawer != null and is_instance_valid(_drawer):
		_drawer.call("release_surface")
		if _drawer.get_parent() != null:
			_drawer.get_parent().remove_child(_drawer)
		_drawer.queue_free()
	_drawer = null

## MS-1 S4: before the unmount. Returns `{open: bool}` and frees the drawer, so the pause depth it holds is given
## back and nothing is left pointing into the composition about to be freed.
func represent_capture() -> Dictionary:
	var was_open := _drawer != null and is_instance_valid(_drawer) and bool(_drawer.call("is_open"))
	_free_drawer()
	return {"open": was_open}

## MS-1 S4: after the new composition is mounted. Builds the drawer for the new class from the stored pins (a
## tablet or desktop has none: `refresh()` returns at once) and, if it was open, opens it again without a slide.
func represent_restore(bag: Dictionary) -> void:
	refresh()
	if bool(bag.get("open", false)) and _drawer != null and is_instance_valid(_drawer):
		_drawer.call("open_now")

## Connects [`refresh`] to `DccSettings.assistant_hub`, once. A bare signal connection to a RefCounted creates
## no node; whether even that moves the phone's pixels at boot is measured by `_quickbar_phoneid_probe.gd`.
func _ensure_hub() -> void:
	if not DccSettings.assistant_hub.changed.is_connected(refresh):
		DccSettings.assistant_hub.changed.connect(refresh)

## CQ-8: a domain change re-derives the row (per-domain pins).
func _on_workspace(_id: String) -> void:
	refresh()

## Opens the `All...` list sheet on the current entries.
func open_list() -> void:
	if _app == null:
		return
	if _list == null:
		_list = QuickBarPhoneList.new()
		_app.add_child(_list)
		_list.setup(_app)
	_list.open(entries(_app))

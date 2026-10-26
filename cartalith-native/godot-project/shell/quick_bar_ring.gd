extends RefCounted
class_name QuickBarRing

## What the user's pins become ON THE CARTWHEEL (`CUSTOM_QUICK_BAR_SCOPE.md` s.4, milestone CQ-4,
## Ruling DH Q3 and Q5): the ring DOCK (up to 4 chips drawn beneath the ring) and the pins
## APPENDED to a diagonal's existing sub-ring (so that built-in plus user children never exceed 9).
##
## ## What this file is, and is not
##
## Pure resolution, no drawing and no input. It turns the stored pin lists (`DccSettings`
## `[quick_bar]`: `dock` and `share`, CQ-1) into the item rows `radial_ring.gd` already understands
## (`label`, `callable`, `enabled`, `reason`), against a FRESH `CommandIndex` so a pin whose
## command was removed, renamed or disabled is shown honestly. `context_broker.gd` calls it when a ring is armed;
## `quick_bar_window.gd` calls the read-only helpers to show a diagonal's built-in children.
##
## **A pin is a pointer, never a second implementation** (scope s.0 rule 1): the row's callable
## runs `DccQuickBar.run_pin_on()`, which emits the owning menu's own `id_pressed`, exactly as the
## quick cluster's chip and the menu click do.
##
## **Nothing is built when nothing is pinned.** Both entry points return `[]` before touching the
## command index (the Assistant chip, CQ-7, reads settings only and is hidden by default), so the
## shipped empty store costs the ring zero work and draws zero pixels more than it did before this
## milestone (the pixel-identity bar, `_quickbar_ringid_probe.gd`).
##
## ## The 9-child ceiling (scope s.1, s.4.3)
##
## A sub-ring of `n` slots at radius 84 with 56 px slots clears its neighbours only up to n = 9
## (chord `2*84*sin(pi/9)` = 57.5 px; n = 10 is 51.9). `allowance()` is how many user pins a diagonal
## with `b` built-in children may add; it is applied on READ here (a hand-edited file, or a build
## that gained a built-in child, can never overfill a sub-ring) and on WRITE by
## `DccSettings.quick_bar_share_add()`.

## The domains that own a ring, as `ContextBroker.ring_domain_req()["domain"]` and each workspace's
## `ring_slots(req)` name them. Not discovered from the app because the window must list a domain's
## diagonals without that domain being the active one; a workspace added later is one more row here.
const DOMAINS := ["world", "civilization", "cartography"]
## The four domain-owned slots; the cardinals are global and fixed (scope s.4.2).
const DIAGONALS := ["NW", "NE", "SE", "SW"]
## Human names for the window.
const DOMAIN_NAMES := {"world": "World", "civilization": "Civilization", "cartography": "Cartography"}

## The `share` key for a diagonal: `<domain>.<DIR>`, the shape `DccSettings` accepts.
static func share_key(domain: String, dir: String) -> String:
	return "%s.%s" % [domain, dir]

## How many user pins a sub-ring with `builtin_count` built-in children may carry: 9 less the
## built-ins, never negative. The arithmetic lives once, in `DccSettings` beside the cap it uses.
static func allowance(builtin_count: int) -> int:
	return DccSettings.quick_bar_share_allowance(builtin_count)

# -- resolving pins ------------------------------------------------------------------------

## One ring row for stored pin `pin` (`{k, t}`) against `idx`. `enabled` is exactly "the index calls
## the row pinnable now" (the same rule `DccQuickBar._chip_for` uses); a disabled row carries its
## `reason` and NO callable, so a release over it runs nothing. The row never lacks a title: a vanished
## command shows the title it had when pinned.
static func _item(app, idx: CommandIndex, pin: Dictionary) -> Dictionary:
	var key := String(pin["k"])
	var row := idx.find_pin(key)
	var title := String(row.get("title", "")) if not row.is_empty() else String(pin.get("t", ""))
	if title == "":
		title = key
	var out := {"pin_key": key, "title": title, "label": DccQuickBar.label_for(title), "shortcut": ""}
	if row.is_empty():
		out["enabled"] = false
		out["reason"] = "Not in this build"
		return out
	out["shortcut"] = DccQuickBar.accelerator_text(row)
	if not bool(row.get("pinnable", false)):
		out["enabled"] = false
		out["reason"] = "Unavailable: " + String(row.get("why_not", "this command cannot be run from a chip."))
		return out
	out["enabled"] = true
	out["callable"] = _run.bind(app, key)
	return out

## Runs pin `key` and says in the status line why it did not run (the same sentence the quick cluster
## says). Static and bound with the app, so a ring row owns no reference to this class.
static func _run(app, key: String) -> void:
	var why := DccQuickBar.run_pin_on(app, key)
	if why != "":
		DccQuickBar.say_on(app, why)

## The ring DOCK's items: the conditional Assistant chip first (CQ-7) when it is shown, then the
## stored pins in order (at most `QUICK_BAR_MAX_DOCK`, enforced by the store on read). `[]` when the
## Assistant chip is hidden and nothing is docked -- the shipped state, which builds no index and
## draws nothing (the pixel-identity bar, `_quickbar_ringid_probe.gd`, in its assistant-off default).
static func dock_items(app) -> Array:
	var out: Array = []
	var chip := assistant_item(app)
	if not chip.is_empty():
		out.append(chip)
	var pins: Array = DccSettings.quick_bar_value("dock")
	if pins.is_empty():
		return out
	var idx := CommandIndex.new()
	idx.build(app, app.bridge)
	for p in pins:
		out.append(_item(app, idx, p))
	return out

## The Assistant chip as a ring row, or `{}` while it is hidden (`AssistantChip.state_for_app`). It is
## NOT a pin: it has no `pin_key`, is not stored, and is not counted against `QUICK_BAR_MAX_DOCK`, so a
## full dock of four pins plus this chip is five items (Ruling DH Q5). A refused gate leaves the row
## disabled with the gate's own sentence as its `reason`, which `radial_ring.gd` already puts in the
## caption of a hovered disabled dock chip and draws dimmed. Marked `assistant` so a caller or a test
## can tell it from a pin.
static func assistant_item(app) -> Dictionary:
	var st := AssistantChip.state_for_app(app)
	var state := String(st["state"])
	if state == AssistantChip.HIDDEN:
		return {}
	var out := {"title": AssistantChip.LABEL, "label": AssistantChip.LABEL, "shortcut": "", "assistant": true}
	if state == AssistantChip.LIVE:
		out["enabled"] = true
		out["callable"] = AssistantChip.open_panel.bind(app)
	else:
		out["enabled"] = false
		out["reason"] = String(st["reason"])
	return out

## The user children to append AFTER a diagonal's built-in children: the stored pins for
## `<domain>.<dir>`, cut to `allowance(builtin_count)`, as sub-ring rows marked `shared`. `[]` when
## there are none (no index is built). The built-in children are never touched by the caller.
## `idx` is an index the caller already built (one build serves all four diagonals of a ring, about
## 8 ms each); pass null to have one built here.
static func shared_children(app, domain: String, dir: String, builtin_count: int, idx: CommandIndex = null) -> Array:
	var share: Dictionary = DccSettings.quick_bar_value("share")
	var pins: Array = share.get(share_key(domain, dir), [])
	var room := allowance(builtin_count)
	if pins.is_empty() or room == 0:
		return []
	if idx == null:
		idx = CommandIndex.new()
		idx.build(app, app.bridge)
	var out: Array = []
	for p in pins:
		if out.size() >= room:
			break
		var it := _item(app, idx, p)
		it["shared"] = true
		out.append(it)
	return out

# -- reading a diagonal's built-in children (for the window) -------------------------------

## For `domain`, the built-in state of each diagonal as `ring_slots()` reports it right now:
## `dir -> {label: String, children: int}` for a `▸` diagonal (an Array under `children`, even an
## empty one), `dir -> {label: String, children: -1}` for a leaf tool. `{}` when no world is loaded
## (every slot is then a disabled placeholder with no children and the real count is unknown --
## the window says so rather than guessing a plausible number). Read-only: `ring_slots` only builds
## rows; nothing it returns is run.
static func builtin_diagonals(app, domain: String) -> Dictionary:
	if app == null or app.bridge == null or not app.bridge.has_world:
		return {}
	var req := {"domain": domain, "armed_tool": "", "finalized": false}
	var out := {}
	for ws in app._workspaces:
		if not ws.has_method("ring_slots"):
			continue
		var diag: Dictionary = ws.ring_slots(req)
		for dir in DIAGONALS:
			if not diag.has(dir):
				continue
			var row: Dictionary = diag[dir]
			var kids: int = (row["children"] as Array).size() if row.has("children") else -1
			out[dir] = {"label": String(row.get("label", dir)), "children": kids}
	return out

## The titles of a diagonal's built-in children, for the window's read-only list. `[]` for a leaf.
static func builtin_child_labels(app, domain: String, dir: String) -> Array:
	var out: Array = []
	if app == null or app.bridge == null or not app.bridge.has_world:
		return out
	var req := {"domain": domain, "armed_tool": "", "finalized": false}
	for ws in app._workspaces:
		if not ws.has_method("ring_slots"):
			continue
		var diag: Dictionary = ws.ring_slots(req)
		if diag.has(dir) and (diag[dir] as Dictionary).has("children"):
			for c in diag[dir]["children"]:
				out.append(String((c as Dictionary).get("label", "")))
	return out

extends RefCounted
## AC-7 (`ASSISTANT_ACTIONS_PROPOSAL.md` milestone AC-7; `PC_SHELL_PROPOSAL.md` s.3.5 "Cards"): the WORDS of an
## action card, one section per KIND of action: setting, view, select, edit, window.
##
## `propose_actions` returns one card holding up to five actions that Apply runs as ONE step (AQ3). A person
## cannot judge a flat list of ids, so [`sections`] turns the card's validated actions into one section per
## kind, each with the kind's own wording: a setting reads `was -> will be`, a view change says what it shows,
## a selection names the thing selected (never just a number), an edit says it arms a tool and draws nothing,
## a window says it opens for the person to use. Each section ends with what Undo covers for that kind.
##
## **This file builds no widgets and runs nothing.** It reads the app (the settlement list, the year cursor,
## the undo label) and returns plain data; `llm_assistant_window.gd::_on_action_proposal` draws it, and Apply
## is still `llm_actions.gd::run_card`, the only runner (`MISTAKES.md`: never a second execution path).
##
## MUST NEVER: change the app; invent a value when the app cannot say (a missing "was" is left out, a settlement
## number outside the list says so: no `0`, no "unknown" standing in for a name); put the model's own words
## anywhere but a row's `reason` (they are inert already, `action_tools.rs`).

## The order sections appear in: what changes the world's settings first, what only moves the person's view
## last, so the reading order is the weight of the change.
const KIND_ORDER := ["setting", "edit", "view", "select", "window", "query"]

## The heading of each kind. A kind this table does not know is shown as its own wire name, upper-cased, never
## hidden (a new kind must not silently vanish from a card).
const KIND_LABEL := {"setting": "SETTING", "edit": "EDIT", "view": "VIEW", "select": "SELECT", "window": "WINDOW", "query": "LOOK UP"}


## The heading text of `kind`.
static func kind_label(kind: String) -> String:
	if KIND_LABEL.has(kind):
		return String(KIND_LABEL[kind])
	return kind.to_upper() if kind != "" else "ACTION"


## True when Apply puts this action on the undo stack: a setting, and the style preset (`run_card`'s one group).
static func is_undoable(cd: Dictionary) -> bool:
	return String(cd.get("route", "")) == "param" or String(cd.get("id", "")) == "view.style_preset"


## What the settlement `index` is called, or "" when this world has no such settlement.
static func settlement_name(bridge, index: int) -> String:
	if bridge == null or index < 0:
		return ""
	var all: Array = bridge.settlements()
	if index >= all.size():
		return ""
	return String((all[index] as Dictionary).get("name", ""))


## The one sentence for action `cd` (one entry of the card's `valid`), in its kind's own words.
static func row_text(cd: Dictionary, app, bridge) -> String:
	var id := String(cd.get("id", ""))
	var a: Dictionary = cd.get("args", {})
	var title := String(cd.get("title", id))
	match String(cd.get("kind", "")):
		"setting":
			var s := "%s: " % title
			if cd.has("from"):
				s += "%s → " % String(cd["from"])
			return s + ("%s" % str(a.get("value", "")) if a.has("value") else "(no value)")
		"view":
			if id.begins_with("view.layer."):
				## The catalogue titles a layer "<label> layer" (`actions.rs::view`), so the title already names it.
				return "%s the %s" % ["Show" if bool(a.get("visible", false)) else "Hide", title]
			if id == "view.style_preset":
				return "Style: %s%s" % [("%s → " % String(cd["from"])) if cd.has("from") else "", String(a.get("preset", ""))]
			if id == "view.zoom_to":
				return "Zoom the map to (%s, %s) at ×%s" % [str(a.get("x", "?")), str(a.get("y", "?")), str(a.get("zoom", "?"))]
			if id == "view.landmark_filter":
				return "%s %s landmarks" % ["Show" if bool(a.get("visible", false)) else "Hide", String(a.get("kind", ""))]
			return title
		"select":
			if id == "select.settlement":
				var i := int(a.get("id", -1))
				var n := settlement_name(bridge, i)
				return ("Select %s (settlement %d)" % [n, i]) if n != "" else ("Select settlement %d (this world has no such settlement)" % i)
			if id == "select.none":
				return "Clear the selection"
			if id == "select.back":
				return "Go back to the previous selection"
			if id == "timeline.set_year":
				var y := "Move the year cursor to %d" % int(a.get("year", 0)) if a.has("year") else "Move the year cursor"
				if app != null and app.has_method("tl_year"):
					y += " (it is at %d now)" % int(app.tl_year())
				return y
			return title
		"edit":
			if id.begins_with("tool."):
				return "Arm the %s tool: you then draw on the map yourself, this card draws nothing" % id.substr("tool.".length())
			if id == "edit.undo":
				var lbl := String(app.undo_target_label()) if app != null and app.has_method("undo_target_label") else ""
				return ("Undo the last step (%s)" % lbl) if lbl != "" else "Undo the last step"
			return title
		"window":
			return "Open %s: it opens for you to use, the assistant does not work inside it" % title.trim_prefix("Open ")
	return title


## What Undo covers for the actions of one section, as the card words it.
static func section_note(kind: String, undoable: int, count: int) -> String:
	match kind:
		"setting":
			return "Edit ▸ Undo reverts these in one step. The world is not rebuilt; press Generate to see the effect."
		"view":
			if undoable > 0:
				return "Applies at once. The style is one Undo step; layers and the camera are session state and stay."
			return "Applies at once. Layers and the camera are session state, not on the undo stack."
		"select":
			return "Session state: the selection and the year cursor are not on the undo stack."
		"edit":
			return "Arming a tool changes nothing in the world until you use it." if count > 0 else ""
		"window":
			return "Opened and left to you; closing it is yours."
	return ""


## The card as sections: `[{kind, label, rows: [{text, reason?, undoable}], note, undoable}]` in [`KIND_ORDER`]
## (kinds not in the list follow, in the order they appear). Empty `valid` gives `[]`.
static func sections(valid: Array, app, bridge) -> Array:
	var by_kind := {}
	var order: Array = []
	for k in KIND_ORDER:
		by_kind[k] = []
	for c in valid:
		var cd: Dictionary = c
		var k := String(cd.get("kind", ""))
		if not by_kind.has(k):
			by_kind[k] = []
			order.append(k)
		(by_kind[k] as Array).append(cd)
	var out: Array = []
	for k in KIND_ORDER + order:
		var items: Array = by_kind[k]
		if items.is_empty():
			continue
		var rows: Array = []
		var n_undo := 0
		for cd in items:
			var d: Dictionary = cd
			var r := {"text": row_text(d, app, bridge), "undoable": is_undoable(d)}
			if d.has("reason"):
				r["reason"] = String(d["reason"])
			if bool(r["undoable"]):
				n_undo += 1
			rows.append(r)
		out.append({"kind": k, "label": kind_label(k), "rows": rows, "note": section_note(k, n_undo, items.size()), "undoable": n_undo})
	return out


## The card's chip: "ACTIONS" for a mixed card, else the one kind (`SETTING`, `VIEW`, ...).
static func chip_text(secs: Array) -> String:
	if secs.size() == 1:
		return "MODEL PROPOSAL · %s" % String((secs[0] as Dictionary)["label"])
	return "MODEL PROPOSAL · ACTIONS"

extends RefCounted
## `MAP_CONTEXT_SCOPE.md` §3 and §9.1: **one request, many providers, one
## presenter** -- CM-1's broker. Owned by `app.gd`, like its other brokers.
##
## `map_overlay.gd` reports a context gesture as `context_requested(req)`: the
## point, every pick under it (`hits`), and which pointer made it. This file
## adds what only the shell knows, asks every provider for rows, merges them
## into §4.1's section order and presents them.
##
## ## The request (`build_request`)
##
## The overlay's `{gx, gy, screen_pos, hits, source}` plus:
##
##   `domain`      `app.active_domain()`
##   `armed_tool`  `app.armed_tool`
##   `form`        `"phone"` / `"tablet"` / `"desktop"` -- `DccTheme.is_phone()`
##                 and `is_tablet()`, never re-derived (§9.3)
##
## §3's `selection`, `draft` and `finalized` are **absent, not defaulted**: no
## CM-1 provider reads them, and a `draft: {count: 0}` written before anything
## computes it would read as "there is no draft" when the truth is "nobody
## asked". They arrive with the rows that need them (CM-2's Draft section).
##
## ## The provider contract (`context_actions`)
##
## `GlobalTools.context_actions(app, req)` (static -- that class has no
## instance) and `context_actions(req)` on every entry of `app._workspaces`
## that has one. Each returns an `Array` of Action dictionaries, §3's shape:
##
##   `id`        String, stable -- `"civ.edit"`, not a menu position
##   `label`     String, what the row says
##   `section`   one of `SECTIONS` below (§4.1's order)
##   `enabled`   bool
##   `reason`    String, **required when `enabled` is false** (§4.2)
##   `callable`  Callable, run when the row is chosen
##   `danger`    bool, optional -- Delete-class; carried for CM-2's card
##   `header`    String, optional -- the hit this row acts on, by name; the
##               first one found titles the phone sheet (else `"Here"`)
##
## A provider returns rows for its own domain's verbs and nothing else, and
## returns `[]` rather than a placeholder when it has none. An empty merge
## opens nothing, which is what a right-click in WORLD has always done.
##
## ## The merge
##
## Providers are asked in a fixed order (GlobalTools, then `app._workspaces`
## in registration order), their rows concatenated, then **stably** sorted by
## section. Within a section, a provider's own order is kept -- which is what
## keeps CX-01's Edit · Move viewer to · Delete in that order. A separator
## falls between two rows whose sections differ, which reproduces CX-01's two
## separators exactly.
##
## ## The presenter (CM-2)
##
## **Desktop and tablet: `context_card.gd`** -- §4.1's sectioned card with
## its header readout, Select ▸, disabled-with-reason rows, inline Tool rows,
## keyboard navigation and a filter. Rows may carry, beyond the above:
##
##   `children`  Array of Action rows -- a one-level submenu in the card
##   `param`     `{"value": Callable -> String, "step": Callable(dir)}` -- an
##               inline parameter row (§4.1's Tool section)
##   `shortcut`  String, drawn right-aligned
##
## **Phone: unchanged since CM-1** -- the same styled `PopupMenu`
## (`DccWidgets.style_popup`) re-presented by `app.phone_present_popup()` as
## the L4 sheet. The phone's own noun surface is CM-5. A `PopupMenu` can show
## neither a submenu row nor a param row, so rows carrying `children` or
## `param` are left out of it, and the providers keep their CM-2 rows off the
## phone altogether (`card_form()`), so the sheet a phone user sees is the one
## CM-1 shipped.
##
## **Select ▸.** Picking one object re-resolves the request with `hits`
## narrowed to that one object and `all_hits` keeping the full list, so every
## provider's rows are about the picked object and the card can still list the
## rest. Nothing above this section had to change for it: a provider reads
## `hits` as it always did.

## §4.1's sections after the header, in their fixed order.
const SECTIONS: Array = ["draft", "tool", "object", "place", "go", "info"]
const ContextCard := preload("res://shell/context_card.gd")

var app
## The phone presenter's popup, rebuilt per request (its rows depend on what
## was hit and what it is called, so a cached list would be stale). Null until
## a phone first needs it.
var popup: PopupMenu
## The desktop and tablet presenter. Null until first needed.
var card: ContextCard
## What the last `resolve()` asked and got -- read by `_on_id`, and by probes.
var last_request: Dictionary = {}
var last_actions: Array = []


func _init(p_app) -> void:
	app = p_app


## `map_overlay.gd`'s `_context_pick`: the engine's read-only label and icon
## picks, as `{kind, id, label?, x, y}` rows. Labels first, then icons, each
## topmost first (`label_pick_all` / `icon_pick_all`); `hits_at()` sorts the
## lot by distance.
func engine_picks(gx: float, gy: float, px_per_cell: float) -> Array:
	var out: Array = []
	var bridge = app.bridge
	if bridge == null or not bridge.has_world:
		return out
	var labels: Array = []
	var li: PackedInt64Array = bridge.label_pick_all(gx, gy, px_per_cell)
	if not li.is_empty():
		labels = bridge.label_list()
	for i in li:
		if i < labels.size():
			var lb: Dictionary = labels[i]
			out.append({"kind": "label", "id": int(i), "label": lb.get("text"),
				"x": float(lb.get("x", gx)), "y": float(lb.get("y", gy))})
	for i in bridge.icon_pick_all(gx, gy):
		var ic: Dictionary = bridge.icon_get(int(i))
		if ic.is_empty():
			continue
		out.append({"kind": "icon", "id": int(i), "label": ic.get("slot"),
			"x": float(ic.get("x", gx)), "y": float(ic.get("y", gy))})
	return out


func build_request(raw: Dictionary) -> Dictionary:
	var req := raw.duplicate()
	req["domain"] = app.active_domain()
	req["armed_tool"] = app.armed_tool
	req["form"] = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	return req


## Every provider's rows, merged. Public so a probe can check the merge
## without opening a popup.
func collect(req: Dictionary) -> Array:
	var rows: Array = []
	rows.append_array(GlobalTools.context_actions(app, req))
	for ws in app._workspaces:
		if ws.has_method("context_actions"):
			rows.append_array(ws.context_actions(req))
	return merge(rows)


## Stable sort by §4.1 section; provider order kept within a section. An
## unknown section sorts last rather than vanishing, so a typo shows up.
static func merge(rows: Array) -> Array:
	var keyed: Array = []
	for n in rows.size():
		var s: int = SECTIONS.find(String(rows[n].get("section", "")))
		keyed.append([s if s >= 0 else SECTIONS.size(), n, rows[n]])
	keyed.sort_custom(func(a: Array, b: Array) -> bool:
		return a[0] < b[0] if a[0] != b[0] else a[1] < b[1])
	var out: Array = []
	for k in keyed:
		out.append(k[2])
	return out


func resolve(raw: Dictionary) -> void:
	if raw.is_empty():
		return
	last_request = build_request(raw)
	last_actions = collect(last_request)
	present(last_request, last_actions)


## Whether this request is presented by the card (desktop, tablet) rather
## than the phone's `PopupMenu` sheet. Providers gate their CM-2 rows on it.
static func card_form(req: Dictionary) -> bool:
	return String(req.get("form", "")) != "phone"


## Select ▸: the same request, about one object. `all_hits` keeps what the
## pointer was over, so the card's list survives the narrowing.
func reselect(hit: Dictionary) -> void:
	if last_request.is_empty():
		return
	var req := last_request.duplicate()
	req["all_hits"] = last_request.get("all_hits", last_request.get("hits", []))
	req["hits"] = [hit]
	last_request = req
	last_actions = collect(req)
	present(req, last_actions)


func present(req: Dictionary, actions: Array) -> void:
	if actions.is_empty():
		return
	if card_form(req):
		if card == null:
			card = ContextCard.new()
			card.setup(app)
			app.add_child(card)
		## `screen_pos` is `map_overlay`'s local space, under the map camera's
		## zoom; the card pops in the viewport `app` is in (the measured
		## correction CX-01's menu made, 2026-09-23).
		var at: Vector2 = app.viewport.overlay.get_global_transform_with_canvas() * Vector2(req["screen_pos"])
		card.open(req, actions, at, reselect)
		return
	_present_phone(req, actions)


func _present_phone(req: Dictionary, all_actions: Array) -> void:
	var actions: Array = all_actions.filter(func(a): return not (a.has("children") or a.has("param")))
	last_actions = actions
	if actions.is_empty():
		return
	if popup == null:
		popup = PopupMenu.new()
		DccWidgets.style_popup(popup)
		popup.id_pressed.connect(_on_id)
		app.add_child(popup)
	popup.clear()
	var prev := ""
	for n in actions.size():
		var a: Dictionary = actions[n]
		var sec := String(a.get("section", ""))
		if n > 0 and sec != prev:
			popup.add_separator()
		prev = sec
		popup.add_item(String(a["label"]), n)
		var idx := popup.get_item_index(n)
		if not bool(a.get("enabled", true)):
			popup.set_item_disabled(idx, true)
			popup.set_item_tooltip(idx, String(a.get("reason", "")))
	var title := "Here"
	for a in actions:
		if (a as Dictionary).has("header"):
			title = String(a["header"])
			break
	if app.phone_present_popup(popup, title,
			"Map · cell %d, %d" % [int(req["gx"]), int(req["gy"])]):
		return
	## `screen_pos` is `map_overlay`'s local space, under the map camera's
	## zoom; an embedded `PopupMenu` pops in the main viewport's space
	## (`civilization_workspace.gd`'s own measured correction, 2026-09-23).
	popup.position = Vector2i(app.viewport.overlay.get_global_transform_with_canvas() * Vector2(req["screen_pos"]))
	popup.reset_size()
	popup.popup()


func _on_id(id: int) -> void:
	if id < 0 or id >= last_actions.size():
		return
	var a: Dictionary = last_actions[id]
	if not bool(a.get("enabled", true)):
		return
	(a["callable"] as Callable).call()

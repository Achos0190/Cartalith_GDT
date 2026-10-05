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
## **Phone, since CM-5** (`MAP_CONTEXT_SCOPE.md` §8.1): a peek/half sheet
## (`app.phone_present_peek_card()` -> `PhoneMenu.peek_card()`), not the
## `PopupMenu` CM-1 shipped -- see this file's own 2026-09-25 history for why
## that PopupMenu is still built as a fallback below (a harness with no phone
## chrome at all). Rows carrying `children` or `param` are still left out
## (peek_card()'s rows are plain buttons, exactly what `PopupMenu` could draw,
## same reason as before: the providers keep their CM-2-only rows off the
## phone via `card_form()`). **Not built here:** §8.1.4's multi-hit "Select ▸"
## chip -- `reselect` is passed through to `peek_card()` but nothing there
## calls it yet.
##
## **Select ▸** (desktop/tablet card only, for now). Picking one object
## re-resolves the request with `hits`
## narrowed to that one object and `all_hits` keeping the full list, so every
## provider's rows are about the picked object and the card can still list the
## rest. Nothing above this section had to change for it: a provider reads
## `hits` as it always did.

## §4.1's sections after the header, in their fixed order.
const SECTIONS: Array = ["draft", "tool", "object", "place", "go", "info"]
const ContextCard := preload("res://shell/context_card.gd")
const RadialRing := preload("res://shell/radial_ring.gd")

var app
## The phone presenter's popup, rebuilt per request (its rows depend on what
## was hit and what it is called, so a cached list would be stale). Null until
## a phone first needs it.
var popup: PopupMenu
## The desktop and tablet presenter. Null until first needed.
var card: ContextCard
## CM-3's ring, desktop only. Null until the first RMB-drag, RMB-hold or Q
## press -- see the "The ring (CM-3)" section below for its whole driver.
var ring: RadialRing
var _ring_press_local: Vector2 = Vector2.ZERO
## What the last `resolve()` asked and got -- read by `_on_id`, and by probes.
var last_request: Dictionary = {}
var last_actions: Array = []


func _init(p_app) -> void:
	app = p_app


## `map_overlay.gd`'s `_context_pick`: the engine's read-only label and icon
## picks, as `{kind, id, label?, x, y}` rows. Labels first, then icons, each
## topmost first (`label_pick_all` / `icon_pick_all`); `hits_at()` sorts the
## lot by distance.
func engine_picks(gx: float, gy: float, px_per_cell: float, zoom: float = 1.0) -> Array:
	var out: Array = []
	var bridge = app.bridge
	if bridge == null or not bridge.has_world:
		return out
	var labels: Array = []
	var li: PackedInt64Array = bridge.label_pick_all(gx, gy, px_per_cell, zoom)
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
	## CM-7's two engine picks (Ruling BA). Each is asked only in the domain
	## whose card carries its verbs -- ways in CIVIL, rivers in WORLD, the
	## domains `MAP_CONTEXT_SCOPE.md` §4.3 puts their rows in -- because a hit
	## with no row of its own on this card would only take the header and a
	## Select ▸ slot from the things that do have rows. `x`/`y` are the nearest
	## point on the way / the picked cell's centre, so `hits_at()`'s distance
	## sort measures to the line, not to an arbitrary anchor.
	var tol := PICK_LINE_PX / px_per_cell if px_per_cell > 0.0 else PICK_LINE_FALLBACK_CELLS
	var domain := String(app.active_domain()) if app.has_method("active_domain") else ""
	if domain == "civilization":
		var w: Dictionary = bridge.way_pick(gx, gy, tol)
		if not w.is_empty():
			var store := String(w.get("store", ""))
			var widx := int(w.get("index", -1))
			var title := CivilizationWorkspaceScript.way_title(w)
			out.append({"kind": "way", "id": WAY_ID_BASE.get(store, 0) + widx, "label": title,
				"store": store, "index": widx,
				"x": float(w.get("x", gx)), "y": float(w.get("y", gy))})
	elif domain == "world":
		var r: Dictionary = bridge.river_pick(gx, gy, tol)
		if not r.is_empty():
			out.append({"kind": "river", "id": int(r["cell"]), "label": river_title(r),
				"x": float(r.get("x", gx)), "y": float(r.get("y", gy))})
	return out


## The pick tolerance for a line (a way, a river), in this control's local
## pixels -- `civilization_workspace.gd::ROUTE_HIT_PX`'s own 6 px, so a way and
## a route are equally easy to hit. With no world on screen to measure a
## pixel-per-cell ratio from, a fixed 1.5 cells, the same fallback.
const PICK_LINE_PX := 6.0
const PICK_LINE_FALLBACK_CELLS := 1.5
const CivilizationWorkspaceScript := preload("res://shell/workspaces/civilization_workspace.gd")
## A way's hit `id` must be unique across its three stores (the Select ▸ list
## and `context_card.gd::_same_hit` compare by kind and id), so each store gets
## its own range; the real `(store, index)` ride on the hit beside it.
const WAY_ID_BASE := {"generated": 0, "sea_lane": 1000000, "manual": 2000000}


## A river described, never named -- rivers are unnamed in this engine
## (`right_dock.gd`'s River context gives the reason). Order is the picked
## cell's own Strahler order; the length is Ruling BA's branch, clicked cell to
## where the walk stopped, and the words say where that was.
static func river_title(r: Dictionary) -> String:
	var km := DccUnits.format(float(r.get("km", 0.0)))
	var where := ""
	match String(r.get("terminus", "")):
		"sea":
			where = "%s to the sea" % km
		"lake":
			where = "%s to a lake" % km
		"edge":
			where = "%s, then off the map edge" % km
		_:
			where = "%s, then ends inland" % km
	return "River (order %d, %s)" % [int(r.get("order", 0)), where]


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
		## CM-4 (`MAP_CONTEXT_SCOPE.md` §7.1): on the tablet's touch-hold
		## gesture only, the card docks on the side away from the dominant
		## hand rather than always trying the pointer's right side first --
		## `DccSettings.dominant_hand()` defaults "right", so the default dock
		## is "left". Every other presentation (desktop RMB, Q, a tablet's
		## own mouse/pen hover) keeps `_target_rect()`'s plain pointer-relative
		## placement, which still flips at a screen edge either way.
		var dock := ""
		if DccTheme.is_tablet() and String(req.get("source", "")) == "touch":
			dock = "right" if DccSettings.dominant_hand() == "left" else "left"
		## **Found by coordinator review, 2026-09-27.** CM-3's desktop RMB-hold
		## and CM-4's tablet touch-hold open the ring and the card SIDE BY SIDE
		## at the same anchor, but `_target_rect()`'s own offset was a bare
		## 10/6 px -- right for the plain-click, no-ring case (CM-2), and never
		## widened for the two "ring and card together" paths. On the CM-4
		## residual's 96 dp touch ring this put the card overlapping the
		## ring's own lower slots and centre caption (screenshotted); desktop's
		## smaller 60 px ring has the identical defect, just less visibly.
		##
		## `ring_clear` is read off the ring's own LIVE geometry --
		## `debug_state()`'s `radius`/`slot`, never a re-declared constant --
		## and stays 0.0 whenever no ring is open beside this card, which keeps
		## CM-2's plain-click placement byte-for-byte unchanged. `radius +
		## slot * 0.5` is "the farthest any slot's own bounding square reaches
		## from centre", the same first two terms `radial_ring.gd::
		## _clamp_centre()` already computes into its own `pad` -- reused
		## rather than a new formula, without that function's trailing
		## `+ 40.0` (which clears the ring/caption of the SCREEN edge, a
		## different concern from clearing the card).
		##
		## Overlap follow-up, same review (`OUTSTANDING_WORK.md`: "the context
		## card can overlap an open sub-ring"): a sub-ring re-centres on its own
		## parent slot (`radial_ring.gd::_open_sub()`), reaching past this
		## top-level footprint toward the card -- `_ring_clearance()` below
		## also reads its live centre-x and `radius + slot * 0.5` whenever one
		## is open, so the card can be opened with a sub-ring already showing.
		## The sub-ring can also open LATER, after this card is already on
		## screen (a hold-then-drag-to-a-children-slot never reopens the card) --
		## `_sync_card_ring_clear()` is what reacts to that; it is called from
		## every ring path that can call `radial_ring.gd::_open_sub()`
		## (`ring_release()`, `ring_click()`, `ring_key_release()`, and now
		## `ring_pointer()` too -- the continuous nested flick lets a drag alone
		## reach `_open_sub()`, CM-3 residual), not from here.
		var cc := _ring_clearance()
		card.open(req, actions, at, reselect, dock,
			float(cc["ring_clear"]), float(cc["sub_x"]), float(cc["sub_clear"]))
		return
	_present_phone(req, actions)


func _present_phone(req: Dictionary, all_actions: Array) -> void:
	var actions: Array = all_actions.filter(func(a): return not (a.has("children") or a.has("param")))
	last_actions = actions
	if actions.is_empty():
		return
	## CM-5: the peek/half sheet, the phone's own noun surface since Ruling
	## AX F1. `on_dismiss` clears `map_overlay.gd`'s sample pin whenever the
	## sheet closes by the user's own action (never on a mere re-resolve --
	## `PhoneMenu.close()` skips the callback for exactly that reason).
	if app.has_method("phone_present_peek_card"):
		var overlay = app.viewport.overlay if app.viewport != null else null
		var on_dismiss := func() -> void:
			if overlay != null:
				overlay.clear_sample_pin()
		if bool(app.phone_present_peek_card(req, actions, reselect, on_dismiss)):
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


## ── The ring (CM-3, `MAP_CONTEXT_SCOPE.md` §5, §6, §9.1) ─────────────────────
##
## The ring needs none of `resolve()`'s machinery above: it never touches
## `hits`, never picks an object, and has no phone form (desktop-only, §6).
## Its request is exactly the three fields §5.1's table actually reads --
## `ring_domain_req()` -- and its rows come from a parallel provider contract,
## `ring_slots(req) -> Dictionary` (workspaces) and `GlobalTools.
## ring_cardinals(app, req) -> Dictionary`, merged by direction (`ring_
## collect()`) rather than by section (there are no sections on a compass).
##
## `map_overlay.gd` never imports this file or `radial_ring.gd`: it is handed
## five plain `Callable`s (`set_ring_callbacks`, mirroring the existing
## `set_context_pick_resolver` pattern) and calls whichever one applies to the
## raw pointer/button event it just saw. That keeps every timing decision
## (§6's flick/hold/sticky table) inside `radial_ring.gd`'s own state machine,
## reachable from exactly one file, rather than re-derived at each call site.

## The card's own obstacle geometry to clear, read off the ring's LIVE state
## (never a re-declared constant, `MISTAKES.md`'s preflight rule) -- `{}`'s
## worth of zeros when no ring is open, so `present()`'s plain-click path is
## unaffected. `sub_x`/`sub_clear` stay 0.0 unless a sub-ring is ALSO open
## (`debug_state()`'s own `sub_open`); `context_card.gd::_target_rect()`'s own
## doc has the "x-range clears both obstacles' shapes" proof this feeds.
func _ring_clearance() -> Dictionary:
	if ring == null or not ring.is_open():
		return {"ring_clear": 0.0, "sub_x": 0.0, "sub_clear": 0.0}
	var rs := ring.debug_state()
	var out := {
		"ring_clear": float(rs.get("radius", 0.0)) + float(rs.get("slot", 0.0)) * 0.5,
		"sub_x": 0.0, "sub_clear": 0.0,
	}
	if bool(rs.get("sub_open", false)):
		var sc: Vector2 = rs.get("sub_centre", Vector2.ZERO)
		out["sub_x"] = sc.x
		out["sub_clear"] = float(rs.get("sub_radius", 0.0)) + float(rs.get("sub_slot_size", 0.0)) * 0.5
	return out


## Called from every ring path that can reach `radial_ring.gd::_open_sub()`
## (`release()`, via `ring_release()`/`ring_click()`; `q_release()`, via
## `ring_key_release()`; `pointer()`, via `ring_pointer()` -- the continuous
## nested flick, CM-3 residual) -- the only transition the card needs to react to
## once it is already open (`radial_ring.gd` never closes a sub-ring without
## closing the whole ring, so there is no "sub closed, ring still open" case).
## A no-op when the card is not showing, or when nothing actually changed
## (`context_card.gd::set_ring_clear()`'s own early return).
func _sync_card_ring_clear() -> void:
	if card == null or not card.visible:
		return
	var cc := _ring_clearance()
	card.set_ring_clear(float(cc["ring_clear"]), float(cc["sub_x"]), float(cc["sub_clear"]))


func _ensure_ring() -> RadialRing:
	if ring == null:
		ring = RadialRing.new()
		ring.setup(app)
		ring.hold_fired.connect(_on_ring_hold_fired)
		## §7.1's own table: "a `tool_arm` pulse on crossing into a slot" --
		## fired for every hover change, desktop or touch alike; `app._haptic()`
		## is itself a no-op off Android/iOS; no separate desktop guard needed.
		ring.hover_entered.connect(_on_ring_hover_entered)
		app.add_child(ring)
	return ring

## No `hits`, no `gx`/`gy`: the ring arms a tool, it never acts on a picked
## object (§5.1's own table has no per-object row anywhere on it).
func ring_domain_req() -> Dictionary:
	return {
		"domain": app.active_domain(),
		"armed_tool": app.armed_tool,
		"finalized": app.bridge != null and app.bridge.has_world and app.bridge.is_finalized(),
	}

## Every provider's ring rows, merged into the 8-slot compass: `GlobalTools.
## ring_cardinals` first (N/E/S/W, identical in every domain, §5.1), then
## each workspace's own `ring_slots(req)` (its NW/NE/SE/SW) over it. Public so
## a probe can ask what a domain's compass looks like without opening a ring.
func ring_collect(req: Dictionary) -> Dictionary:
	var slots := GlobalTools.ring_cardinals(app, req)
	for ws in app._workspaces:
		if ws.has_method("ring_slots"):
			var diag: Dictionary = ws.ring_slots(req)
			for dir in diag:
				slots[dir] = diag[dir]
	return slots

## `map_overlay.gd`'s RMB press: local space, converted the same way
## `present()` converts a card's `screen_pos` above.
func ring_press(local_pos: Vector2) -> void:
	_ring_press_local = local_pos
	var at: Vector2 = app.viewport.overlay.get_global_transform_with_canvas() * local_pos
	_ensure_ring().arm(at, ring_collect(ring_domain_req()), false)

## CM-3 residual, continuous nested flick: `radial_ring.gd::pointer()` can now
## call its own `_open_sub()` directly (dragging PAST a `▸` slot, §5.2 rule 2),
## not only through a release -- so this is now a FOURTH path that can reach
## `_open_sub()`, beside the three `_sync_card_ring_clear()`'s own doc comment
## already names. Synced unconditionally, same as those three: a card open
## beside a still-hold ring (§6's third row) must not keep clearing space for
## a top-level ring once a drag has re-centred it onto a sub-ring, and this is
## the only call site that can see that transition happen mid-gesture.
func ring_pointer(local_pos: Vector2) -> void:
	if ring == null:
		return
	ring.pointer(app.viewport.overlay.get_global_transform_with_canvas() * local_pos)
	_sync_card_ring_clear()

## Returns whether the ring was open for this release (so `map_overlay.gd`
## knows not to fall through to the card's own click path) -- runs the picked
## callable, if any, before returning.
func ring_release() -> bool:
	if ring == null:
		return false
	var was_open := ring.is_open()
	var cb: Callable = ring.release()
	## `ring.release()` is one of the two paths that can call `radial_ring.gd::
	## _open_sub()` (a `children` slot's release opens a sub-ring rather than
	## returning a valid `cb`) -- sync unconditionally, not only in the
	## `cb.is_valid()` branch below.
	_sync_card_ring_clear()
	if cb.is_valid():
		cb.call()
	return was_open

## The sticky-ring click path (a plain LMB press while the ring is open and
## sticky). Returns true unconditionally: `map_overlay.gd` only calls this
## once `ring_is_open()` already said yes, so the click is always the ring's
## to consume rather than the armed tool's.
func ring_click(local_pos: Vector2) -> bool:
	if ring == null:
		return false
	var at: Vector2 = app.viewport.overlay.get_global_transform_with_canvas() * local_pos
	var cb: Callable = ring.click_at(at)
	## `click_at()` re-aims and calls `release()` -- the same `_open_sub()` path
	## as `ring_release()` above, same unconditional sync.
	_sync_card_ring_clear()
	if cb.is_valid():
		cb.call()
	return true

func ring_is_open() -> bool:
	return ring != null and ring.is_open()

## Esc, while the ring is open (`app.gd`'s `_unhandled_key_input`): closes the
## ring only, same rule §9.4 gives the card ("An open surface never commits or
## discards a draft by being dismissed").
func ring_close() -> void:
	if ring != null:
		ring.close()

## Q key-down (`app.gd`). `pos` is `map_overlay.gd`'s own last-tracked mouse
## position, local space -- "Ring at the cursor" (§6), not at whatever the
## RMB press point happened to be.
func ring_key_press(local_pos: Vector2) -> void:
	if ring != null and ring.is_open():
		return
	var at: Vector2 = app.viewport.overlay.get_global_transform_with_canvas() * local_pos
	_ring_press_local = local_pos
	_ensure_ring().arm(at, ring_collect(ring_domain_req()), true)

## Q key-up. Returns true when the key event should count as handled: either
## it picked something, or the ring is (still) open, sticky, waiting for a
## click -- see `radial_ring.gd::q_release()`'s own doc for why a second
## key-up while sticky is a no-op rather than a second resolve.
func ring_key_release() -> bool:
	if ring == null:
		return false
	var cb: Callable = ring.q_release()
	## `q_release()` calls `release()` -- the same `_open_sub()` path, same
	## unconditional sync, before either return below.
	_sync_card_ring_clear()
	if cb.is_valid():
		cb.call()
		return true
	return ring.is_open()

## The still-hold firing (`radial_ring.gd`'s own `hold_fired` signal, §6's
## third row): the ring is now visible and sticky; open the card beside it,
## through the exact same `resolve()` the click path uses -- never a second
## card-opening path. Needs the real pick-driven request (`hits` included),
## which only `map_overlay.gd::request_at()` can build, so this is the one
## place the ring reaches back into the overlay's own picking machinery.
func _on_ring_hold_fired() -> void:
	var req: Dictionary = app.viewport.overlay.request_at(_ring_press_local, "mouse")
	if not req.is_empty():
		resolve(req)

## CM-4 (`MAP_CONTEXT_SCOPE.md` §7.1, CM-4): the tablet's touch-hold, called
## by `map_overlay.gd` once ITS OWN 500 ms still-hold timer fires (that timer
## already did the waiting §7.1 asks for, so this opens full and immediately --
## `arm(..., true)` is the same "no flick delay" branch Q's press uses, not a
## second hold timer stacked on top of the first). The card opens alongside it
## through the same `resolve()` every other presenter uses -- never a second
## card-opening path, same rule `_on_ring_hold_fired()` above follows for the
## desktop RMB hold. `local_pos` is `map_overlay.gd`'s own local space, the
## same conversion `ring_press()` uses.
func ring_touch_open(local_pos: Vector2) -> void:
	_ring_press_local = local_pos
	var at: Vector2 = app.viewport.overlay.get_global_transform_with_canvas() * local_pos
	_ensure_ring().arm(at, ring_collect(ring_domain_req()), true)
	app._haptic("sample")   ## §7.1's own table: "sample haptic pulse (the table has it)"
	var req: Dictionary = app.viewport.overlay.request_at(local_pos, "touch")
	if not req.is_empty():
		resolve(req)

func _on_ring_hover_entered() -> void:
	app._haptic("tool_arm")

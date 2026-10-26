extends RefCounted
## LLM-6d (`LLM_ASSISTANT_SCOPE.md` s.5b.8 item 12, s.5b.10; Rulings DD and DI): the shell half of
## the assistant's VIEW AND LAYER ACTIONS. The model may PROPOSE five kinds of action over the
## map's session state; `cartalith_llm::view_actions` validates them; this file does two jobs and
## nothing else:
##
##   1. [`facts`] reads the live app into the FACTS the validator and the tool schema are built
##      from, every request (and every turn of a request): the layers `ViewportHost` can switch with
##      their state, the style presets, the landmark types the map carries, the grid and zoom range,
##      and the windows. The windows are the **CommandIndex rows** (`command_index.gd`) that the
##      index marks `pinnable` and `available` AND that the curated [`WINDOWS`] table names, so a
##      menu row that leaves the build leaves the enum, and the unpinnable and side-effecting rows
##      (Save, Generate, Export, a check or radio row, a generated list) can never appear.
##   2. [`apply`] does what a valid card says, after Apply was clicked, by the app's own setters:
##      `ViewportHost.set_layer_visible` / `set_landmarks_shown_by_user`, `set_landmark_kind_visible`,
##      `move_view_to` + `zoom_step`, `RenderWorkspace._apply_preset` and `CommandIndex.run_row`. It
##      adds no setter of its own. It re-checks every value against fresh facts first, because the
##      card may have sat open while the app changed.
##
## ## Undo, defined (the owner asked for it)
##
## Only the style preset is on the undo stack: [`apply`] runs it inside the LLM-5 grouped undo
## (`undo_group.gd`), so Edit > Undo reverts it in ONE step. Layer visibility, landmark filters, the
## camera and an opened window are **session state: instant and NOT undoable**; the card says so
## beside Apply and lists each row's old value so the user can reverse it by hand. See the header of
## `cartalith-llm/src/view_actions.rs` for why (a camera move must not share Ctrl+Z with a sculpt).
##
## No `class_name` (preloaded by `llm_assistant_window.gd`), so adding it needs no `godot --import`
## pass, which strips `project.godot`'s comments (`CLAUDE.md`).
##
## MUST NEVER: generate, open or close or save a project, delete, export, import or touch the
## network; run a menu row that is not in the curated table AND pinnable AND available; apply
## anything but the arguments Rust validated; touch the undo stack outside `apply`'s one group.

## The curated read-only windows: the id the model names, and the `DccMenus` command id of the menu
## row that opens it. A labelled design choice (2026-10-26): each opens a window or panel and
## changes no world data. Deliberately NOT here: the Data manager (it hosts deletions), anything
## under File, Export, Import, Preferences ▸ Memory, Generate, the pipeline, undo history.
const WINDOWS: Array = [
	{"id": "asset_library", "menu": DccMenus.ID_ASSET_LIBRARY},
	{"id": "factions", "menu": DccMenus.ID_FACTIONS},
	{"id": "world_data", "menu": DccMenus.ID_WORLD_DATA},
	{"id": "vault", "menu": DccMenus.ID_VAULT},
	{"id": "timeline", "menu": DccMenus.ID_STORY_TIMELINE},
	{"id": "journey_planner", "menu": DccMenus.ID_STORY_JOURNEY_PLANNER},
	{"id": "journeys_list", "menu": DccMenus.ID_SAVED_JOURNEYS},
	{"id": "travel_library", "menu": DccMenus.ID_TRAVEL_LIBRARY},
	{"id": "conflicts", "menu": DccMenus.ID_CONFLICTS_LIST},
	{"id": "shortcuts", "menu": DccMenus.ID_HELP_SHORTCUTS},
]


## Lower-case `[a-z0-9_]` id of a display name, runs of other characters collapsed to one `_`
## ("Cel / Toon" -> `cel_toon`). The preset ids the model names are these.
static func slug(name: String) -> String:
	var out := ""
	for ch in name.to_lower():
		var ok: bool = (ch >= "a" and ch <= "z") or (ch >= "0" and ch <= "9")
		if ok:
			out += ch
		elif not out.ends_with("_"):
			out += "_"
	return out.trim_prefix("_").trim_suffix("_")


## The CommandIndex row for menu command `menu_id` that is pinnable and available, or `{}`.
## `idx` is a freshly built index; the row is the SAME dictionary the index holds, so
## `idx.run_row(row)` runs it. Disambiguated duplicates (`pinnable` false) are skipped on purpose.
static func window_row(idx: CommandIndex, menu_id: int) -> Dictionary:
	for r in idx.all():
		var row: Dictionary = r
		if String(row.get("kind", "")) == "menu" and int(row.get("_id", -1)) == menu_id \
				and bool(row.get("pinnable", false)) and bool(row.get("available", false)):
			return row
	return {}


## The index of preset `id` in `RenderWorkspace.STYLE_PRESETS`, or -1.
static func preset_index(id: String) -> int:
	for i in RenderWorkspace.STYLE_PRESETS.size():
		if slug(String(RenderWorkspace.STYLE_PRESETS[i][0])) == id:
			return i
	return -1


## The facts Rust builds the view catalogue from (`ViewCatalogue::from_json`), read from the live
## app NOW. A group is ABSENT when its action cannot run, with the reason in `withheld`, so the
## model is never offered what Apply would refuse. Reads only; writes nothing and builds the index
## without opening a menu (`CommandIndex.build` never emits `about_to_popup`).
static func facts(app, bridge) -> Dictionary:
	var f := {}
	var withheld := {}
	var vp = app.viewport
	if vp == null:
		return {"withheld": {"set_layer_visible": "The map is not built yet.", "set_style_preset": "The map is not built yet.",
			"zoom_to": "The map is not built yet.", "set_landmark_filter": "The map is not built yet.", "open_window": "The map is not built yet."}}
	# -- layers: the ten ids `ViewportHost.set_layer_visible` takes, in the Layers popover's order.
	var layers: Array = []
	for l in CartographyWorkspace.LIVE_LAYERS:
		var ld: Dictionary = l
		var id := String(ld["id"])
		layers.append({"id": id, "label": String(ld["label"]), "on": bool(vp.layer_visible(id))})
	f["layers"] = layers
	# -- style presets (the base looks; variants are not offered).
	var rw = app._render_workspace()
	if rw == null:
		withheld["set_style_preset"] = "The style gallery is not available."
	elif not bridge.has_world:
		withheld["set_style_preset"] = "No world is open."
	elif bridge.generating:
		withheld["set_style_preset"] = "A world is generating; the style cannot change until it finishes."
	else:
		var presets: Array = []
		for i in RenderWorkspace.STYLE_PRESETS.size():
			var name := String(RenderWorkspace.STYLE_PRESETS[i][0])
			var row := {"id": slug(name), "label": name, "on": i == int(rw._active_preset) and String(rw._active_variant) == ""}
			if RenderWorkspace.PRESET_LAYERS_OFF.has(name):
				row["note"] = "Also hides these layers: %s. Edit > Undo restores the look, not those layers." % ", ".join(
					PackedStringArray(RenderWorkspace.PRESET_LAYERS_OFF[name]))
			presets.append(row)
		f["presets"] = presets
	# -- camera and landmark types need a world.
	if not bridge.has_world:
		withheld["zoom_to"] = "No world is open."
		withheld["set_landmark_filter"] = "No world is open."
	else:
		var g: Vector2i = bridge.grid_size()
		if g.x > 0 and g.y > 0:
			f["camera"] = {"w": g.x, "h": g.y, "zoom_min": ViewportHost.ZOOM_MIN, "zoom_max": float(vp._zoom_max), "zoom_now": float(vp.zoom())}
		else:
			withheld["zoom_to"] = "The world has no grid yet."
		var counts: Dictionary = vp.landmark_kind_counts()
		var kinds: Array = []
		var seen := {}
		for k in bridge.landmark_kinds():
			var spec: Dictionary = k
			var key := String(spec.get("key", ""))
			if key != "" and int(counts.get(key, 0)) > 0 and not seen.has(key):
				seen[key] = true
				kinds.append({"id": key, "label": String(spec.get("label", key.capitalize())), "on": bool(vp.landmark_kind_visible(key))})
		if int(counts.get("_manual_poi", 0)) > 0:
			kinds.append({"id": "_manual_poi", "label": "Hand-placed POI icons", "on": bool(vp.landmark_kind_visible("_manual_poi"))})
		if kinds.is_empty():
			withheld["set_landmark_filter"] = "This world carries no landmarks to filter."
		else:
			f["kinds"] = kinds
	# -- windows: the curated table, intersected with the live index.
	var idx := CommandIndex.new()
	idx.build(app, bridge)
	var windows: Array = []
	for w in WINDOWS:
		var wd: Dictionary = w
		var row := window_row(idx, int(wd["menu"]))
		if not row.is_empty():
			windows.append({"id": String(wd["id"]), "label": String(row["title"])})
	if windows.is_empty():
		withheld["open_window"] = "No window is available from the command index."
	else:
		f["windows"] = windows
	f["withheld"] = withheld
	return f


## [`facts`] as the JSON text `LlmStream.prepare` takes in `args.view`.
static func facts_json(app, bridge) -> String:
	return JSON.stringify(facts(app, bridge))


## Applies the validated changes of one view card. `valid` is the card's `valid` array (Rust's
## `view_verdict_dict`: `{action, args, label, undoable, ...}`); `ug` is `app.undo_groups`.
##
## Order: (1) re-check every change against FRESH facts and refuse the whole card if any value is
## no longer offered (nothing changed); (2) the undoable change (the preset) inside one undo group;
## (3) the instant changes. Returns `{ok, reason, applied, instant, group_id, failed}`: `ok` false
## means NOTHING was changed (a refusal in step 1 or 2); `ok` true with a non-empty `failed` means
## the group committed and a later instant change did not land, each named in `failed` (the card
## says so rather than claiming success). `group_id` is the undo group's id, or 0 when the card
## had no undoable member.
static func apply(app, bridge, ug, valid: Array) -> Dictionary:
	var fresh := facts(app, bridge)
	var gone := _stale(fresh, valid)
	if gone != "":
		return {"ok": false, "reason": "%s is no longer available, so nothing was applied." % gone, "applied": 0, "instant": 0, "group_id": 0, "failed": []}
	var vp = app.viewport
	var idx := CommandIndex.new()
	idx.build(app, bridge)
	var group_id := 0
	var applied := 0
	var instant := 0
	var failed: Array = []
	# (2) the undoable members, in one group.
	var undoable: Array = valid.filter(func(c): return bool(c.get("undoable", false)))
	if not undoable.is_empty():
		var rw = app._render_workspace()
		var refused: String = ug.begin("Assistant: style %s" % String(undoable[0]["to"]).left(40))
		if refused != "":
			return {"ok": false, "reason": "Not applied: %s." % refused, "applied": 0, "instant": 0, "group_id": 0, "failed": []}
		for c in undoable:
			var i := preset_index(String(c["args"]["preset"]))
			if i < 0 or rw == null:
				var r0: Dictionary = ug.abort("the style preset is not available")
				return {"ok": false, "reason": "Not applied: %s." % String(r0.get("reason", "")), "applied": 0, "instant": 0, "group_id": 0, "failed": []}
			rw._apply_preset(i)
			applied += 1
		var r: Dictionary = ug.commit()
		if not bool(r.get("ok", false)):
			return {"ok": false, "reason": "Not applied: %s. Nothing was changed." % String(r.get("reason", "")), "applied": 0, "instant": 0, "group_id": 0, "failed": []}
		group_id = int(r.get("id", 0))
	# (3) the instant members.
	for c in valid:
		if bool(c.get("undoable", false)):
			continue
		var a: Dictionary = c["args"]
		var why := ""
		match String(c["action"]):
			"set_layer_visible":
				var layer := String(a["layer"])
				var want := bool(a["visible"])
				if layer == "landmarks":
					vp.set_landmarks_shown_by_user(want)
				else:
					vp.set_layer_visible(layer, want)
				if bool(vp.layer_visible(layer)) != want:
					why = "%s did not change" % String(c["label"])
			"set_landmark_filter":
				var kind := String(a["kind"])
				var want_k := bool(a["visible"])
				vp.set_landmark_kind_visible(kind, want_k)
				if bool(vp.landmark_kind_visible(kind)) != want_k:
					why = "%s did not change" % String(c["label"])
			"zoom_to":
				if a.has("zoom"):
					var cur := float(vp.zoom())
					if cur > 0.0:
						vp.zoom_step(float(a["zoom"]) / cur)
				vp.move_view_to(float(a["x"]), float(a["y"]))
				if a.has("zoom") and absf(float(vp.zoom()) - float(a["zoom"])) > 0.02 * maxf(float(a["zoom"]), 1.0):
					why = "the view reached zoom %s, not %s" % [str(snappedf(float(vp.zoom()), 0.01)), str(a["zoom"])]
			"open_window":
				var menu := _window_menu(String(a["window"]))
				var row := window_row(idx, menu)
				if row.is_empty() or not idx.run_row(row):
					why = "%s could not be opened" % String(c["label"])
			_:
				why = "unknown action %s" % String(c["action"])
		if why == "":
			instant += 1
		else:
			failed.append(why)
	applied += instant
	return {"ok": true, "reason": "", "applied": applied, "instant": instant, "group_id": group_id, "failed": failed}


## The `DccMenus` command id behind curated window `id`, or -1.
static func _window_menu(id: String) -> int:
	for w in WINDOWS:
		if String((w as Dictionary)["id"]) == id:
			return int((w as Dictionary)["menu"])
	return -1


## The label of the first change whose value is not in `fresh` any more, or "" when all are.
## Checks only what Rust checked (membership and the camera's bounds), against facts read now.
static func _stale(fresh: Dictionary, valid: Array) -> String:
	var has := func(list: String, id: String) -> bool:
		for r in fresh.get(list, []):
			if String((r as Dictionary).get("id", "")) == id:
				return true
		return false
	for c in valid:
		var a: Dictionary = c["args"]
		var ok := true
		match String(c["action"]):
			"set_layer_visible": ok = has.call("layers", String(a["layer"]))
			"set_style_preset": ok = has.call("presets", String(a["preset"]))
			"set_landmark_filter": ok = has.call("kinds", String(a["kind"]))
			"open_window": ok = has.call("windows", String(a["window"]))
			"zoom_to":
				var cam: Dictionary = fresh.get("camera", {})
				ok = not cam.is_empty() and float(a["x"]) <= float(int(cam["w"]) - 1) and float(a["y"]) <= float(int(cam["h"]) - 1) \
					and (not a.has("zoom") or (float(a["zoom"]) >= float(cam["zoom_min"]) and float(a["zoom"]) <= float(cam["zoom_max"])))
			_: ok = false
		if not ok:
			return String(c.get("label", c.get("action", "A change")))
	return ""

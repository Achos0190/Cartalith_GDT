extends Node
## Protects: the mid-generation sweep of 2026-10-12 -- every shell entry point that can reach a `WorldGen`
## `#[func]` while the worker thread owns it (`bridge.generating`) WITHOUT a user click (a timer, a window
## notification) or by a direct call, and that the 2026-10-11 arming sweep left open. Each must neither panic
## (`Gd<T>::bind() failed`) nor drop the user's intent silently: it is refused with the shell's "Not done" line,
## held, or replayed when the run finishes. Run WINDOWED from a scratch copy, never headless, never from the live
## project. Legs, all fired during ONE real `bridge.generate` (512 x 320) after a small world exists:
##   R  resource_overlay._refresh (direct + its 0.5 s timer)      -> held text, redrawn after the run
##   T  dcc_shell._tl_tick (direct + the 0.6 s playback timer)    -> skipped; advances again after the run
##   C  civilization_workspace._tl_on_play_tick (direct)          -> skipped
##   B  WM_GO_BACK_REQUEST notification                            -> refused with a hint; works after the run
##   X  confirm_unsaved_world (the OS-close card)                  -> seed dashed with its reason; real seed after
##   U  menus._on_undo_budget rung + Clear                         -> rung deferred and replayed; Clear refused
##   D  Edit > Redo (menus._on_edit(ID_REDO) -> app.redo_last)     -> refused with a hint
##   W  the bridge read wrappers (settlements, roads, label_*, paint_*, icon_*, civ year ...) -> empty default, 0 panics
##   Y  callers of the deliberately-ungated reads (batch 2026-10-12 follow-up): viewport _apply_river_paint /
##      visible_grid_rect / layer_visible("rivers") / _lod_tile_rect, the cartography label drag, the atlas tile-size
##      menu, the Bake button, the ring's finalized flag and the vault info / backlink reads -> no panic, last-seen
##      values (never invented ones) where the bridge caches
##   P  menus._paste_clipboard / _cut / _copy / _select_all        -> refused, clipboard kept, paste replays after
##   K  right_dock paint commit / discard                          -> refused; commit works after
## Panics are counted by an in-process Logger here AND by grepping stderr afterwards (the caller's job).
## usage: godot --path <scratch copy> res://_gsw_probe.tscn -- [--tag T]
## Result: an assertion (CRLF file), windowed only, run from a scratch copy: `godot --path <scratch copy>
## res://_gsw_probe.tscn -- [--tag T]` (only `--tag` is read). Every line is prefixed `[<tag>] ` and it ends `RESULT
## <tag> checks=<n> fail=<n> engine-panic-hits(total)=<n>`. Exit 0 when no check failed, 1 otherwise, 2 when not windowed
## (`ABORT run WINDOWED`).

var app: Node
var bridge: Node
var _vp: SubViewport
var _tag := "gsw"
var _checks := 0
var _fail := 0

class PanicLogger extends Logger:
	var hits := 0
	var lines: PackedStringArray = PackedStringArray()
	func _log_error(function: String, _file: String, _line: int, code: String, rationale: String, _editor_notify: bool, _error_type: int, bt: Array[ScriptBacktrace]) -> void:
		var s := "%s %s %s" % [function, code, rationale]
		if "bind() failed" in s or "cannot borrow" in s or "panicked" in s or "already bound" in s or "already mutably" in s:
			hits += 1
			var frame := ""
			for b in bt:
				if b != null and b.get_frame_count() > 0:
					frame = "%s:%d %s" % [b.get_frame_file(0).get_file(), b.get_frame_line(0), b.get_frame_function(0)]
					break
			if lines.size() < 60:
				lines.append("%s | %s" % [s.substr(0, 80), frame])
	func _log_message(_m: String, _e: bool) -> void:
		pass

var _pl: PanicLogger


func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])


func _check(ok: bool, what: String) -> void:
	_checks += 1
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _guard() -> bool:
	var img := Image.create_empty(2, 2, false, Image.FORMAT_RGBA8)
	img.fill(Color(0, 0, 0, 0))
	var t := ImageTexture.create_from_image(img)
	img.set_pixel(1, 1, Color(1, 0, 0, 1))
	t.update(img)
	return t.get_image().get_pixel(1, 1).a > 0.0


func _hint() -> String:
	return String((app._status_labels["hint"] as Label).text)


## Runs `c` and checks it raised no engine panic; returns the panic delta.
func _fire(name: String, c: Callable) -> int:
	var before := _pl.hits
	c.call()
	var d := _pl.hits - before
	_check(d == 0, "%s: 0 engine panics (got %d)" % [name, d])
	return d


func _texts(root: Node) -> String:
	var out := PackedStringArray()
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Label:
			out.append(String((n as Label).text))
		elif n is Button:
			out.append(String((n as Button).text))
		if n is Control and String((n as Control).tooltip_text) != "":
			out.append(String((n as Control).tooltip_text))
		for c in n.get_children(true):
			stack.append(c)
	return " | ".join(out)


## The bridge's pending undo budget; -999 on a shell without the field (the HEAD negative control).
func _pending() -> int:
	var v = bridge.get("_pending_undo_budget_mb")
	return -999 if v == null else int(v)


## Waits for the run to finish, then lets the finish handlers settle.
func _wait_done() -> void:
	var spins := 0
	while bridge.generating and spins < 1200:
		await _frames(5)
		spins += 1
	await _frames(20)


func _civ_ws() -> Node:
	for ws in app._workspaces:
		if ws != null and ws.has_method("_tl_on_play_tick"):
			return ws
	return null


## Every visible embedded window under the probe viewport whose text contains `needle`.
func _windows_with(needle: String) -> Array:
	var out: Array = []
	for w in _vp.get_embedded_subwindows():
		if w is Window and (w as Window).visible and needle in _texts(w):
			out.append(w)
	return out


## Leg W (backlog row "unguarded bridge read wrappers"): while `bridge.generating` the worker holds `WorldGen`,
## so each read wrapper must answer its empty default WITHOUT reaching the engine. Each call is fired inside
## `_fire` (0 new panics) and its value compared with the documented default. Protects the gates added to
## `engine_bridge.gd`; mutating any one `generating or` back to the bare `not _has(` makes its row fail.
func _leg_reads(primed_tex: Texture2D, primed_tier: String) -> void:
	var rows: Array = [
		["settlements", func(): return bridge.settlements(), []],
		["roads", func(): return bridge.roads(), []],
		["sea_routes", func(): return bridge.sea_routes(), []],
		["provinces", func(): return bridge.provinces(), []],
		["province_cell_counts", func(): return bridge.province_cell_counts(), []],
		["trade_balances", func(): return bridge.trade_balances(), []],
		["label_list", func(): return bridge.label_list(), []],
		["labels_render_list", func(): return bridge.labels_render_list(), []],
		["label_get", func(): return bridge.label_get(0), {}],
		["label_get_selected", func(): return bridge.label_get_selected(), -1],
		["label_hit_test", func(): return bridge.label_hit_test(10.0, 10.0, 4.0), -1],
		["label_get_selection", func(): return bridge.label_get_selection(), PackedInt64Array()],
		["icon_list", func(): return bridge.icon_list(), []],
		["icon_armed", func(): return bridge.icon_armed(), {}],
		["icon_hit_test", func(): return bridge.icon_hit_test(10.0, 10.0), -1],
		["icon_get_selection", func(): return bridge.icon_get_selection(), PackedInt64Array()],
		["get_paint_layers", func(): return bridge.get_paint_layers(), PackedStringArray()],
		["paint_painted_counts", func(): return bridge.paint_painted_counts(), {}],
		["paint_draft_count", func(): return bridge.paint_draft_count(), 0],
		["build_paint_preview_texture", func(): return bridge.build_paint_preview_texture(), null],
		["get_civ_year", func(): return bridge.get_civ_year(), 0],
		["get_civ_date", func(): return bridge.get_civ_date(), {}],
		["get_civ_timeline_years", func(): return bridge.get_civ_timeline_years(), PackedInt64Array()],
		["sample_cell", func(): return bridge.sample_cell(5, 5), {}],
		["route_count", func(): return bridge.route_count(), 0],
		["sculpt_stamp_count", func(): return bridge.sculpt_stamp_count(), -1],
		["sculpt_list_stamps", func(): return bridge.sculpt_list_stamps(), []],
		["sculpt_can_undo", func(): return bridge.sculpt_can_undo(), false],
		["conflict_list", func(): return bridge.conflict_list(), []],
		["journey_list", func(): return bridge.journey_list(), []],
		["gpu_stages_used", func(): return bridge.gpu_stages_used(), PackedStringArray()],
		# Second sweep (read wrappers gated 2026-10-12: vault, as_*, atlas, urban rules, lod_*, info_*, jp_*,
		# measure_*, appearance/ramp/colour-space reads, sculpt_get_*). Rows generated from each gate's own
		# fallback, so a row goes red when its gate is removed (the call then reaches the borrowed WorldGen).
		# as_slice_preview is checked on `ok` alone, below.
		["territory_opacity_default", func(): return bridge.territory_opacity_default(), 82.0 / 255.0],
		["urban_layouts", func(): return bridge.urban_layouts(PackedInt32Array()), []],
		["settlement_diagnostics", func(): return bridge.settlement_diagnostics(PackedInt32Array()), []],
		["explain_settlement", func(): return bridge.explain_settlement(0), {}],
		["get_active_urban_rules", func(): return bridge.get_active_urban_rules(), {}],
		["urban_town_plan_options", func(): return bridge.urban_town_plan_options(), {}],
		["quality_tiers", func(): return bridge.quality_tiers(), PackedStringArray()],
		["looks", func(): return bridge.looks(), []],
		["look", func(): return bridge.look(), ""],
		["npr_settings", func(): return bridge.npr_settings(), {}],
		["appearance", func(): return bridge.appearance(), {}],
		["appearance_tunables", func(): return bridge.appearance_tunables(), []],
		["ramp_presets", func(): return bridge.ramp_presets(), []],
		["color_ramp", func(): return bridge.color_ramp(), []],
		["ramp_modes", func(): return bridge.ramp_modes(), []],
		["ramp_mode", func(): return bridge.ramp_mode(), ""],
		["color_spaces", func(): return bridge.color_spaces(), []],
		["color_space", func(): return bridge.color_space(), ""],
		["color_print_gamut", func(): return bridge.color_print_gamut(Color()), {}],
		["layer_stack", func(): return bridge.layer_stack(), []],
		["blend_modes", func(): return bridge.blend_modes(), []],
		["get_sculpt_features", func(): return bridge.get_sculpt_features(), []],
		["get_sculpt_presets", func(): return bridge.get_sculpt_presets(), []],
		["get_sculpt_globals_info", func(): return bridge.get_sculpt_globals_info(), []],
		["get_sculpt_freehand_modes", func(): return bridge.get_sculpt_freehand_modes(), PackedStringArray()],
		["sculpt_get_globals", func(): return bridge.sculpt_get_globals(), {}],
		["sculpt_get_feature", func(): return bridge.sculpt_get_feature(), ""],
		["sculpt_get_feature_params", func(): return bridge.sculpt_get_feature_params(), {}],
		["sculpt_get_freehand_mode", func(): return bridge.sculpt_get_freehand_mode(), ""],
		["sculpt_get_seed", func(): return bridge.sculpt_get_seed(), -1],
		["get_sculpt_grid_snap_steps", func(): return bridge.get_sculpt_grid_snap_steps(), PackedFloat64Array()],
		["sculpt_get_grid_snap", func(): return bridge.sculpt_get_grid_snap(), {}],
		["sculpt_stroke_point_count", func(): return bridge.sculpt_stroke_point_count(), -1],
		["sculpt_get_selection", func(): return bridge.sculpt_get_selection(), PackedInt64Array()],
		["atlas_status", func(): return bridge.atlas_status(), {}],
		["bake_estimate", func(): return bridge.bake_estimate(0), {}],
		["atlas_is_covered", func(): return bridge.atlas_is_covered(0, 0, 0), false],
		["atlas_export_zip", func(): return bridge.atlas_export_zip(false), PackedByteArray()],
		["atlas_tile_png", func(): return bridge.atlas_tile_png(0, 0, 0), PackedByteArray()],
		["finalize_check", func(): return bridge.finalize_check(""), ""],
		["measure_result", func(): return bridge.measure_result(), {}],
		["measure_section", func(): return bridge.measure_section(0.0, 0.0, 0.0, 0.0, 0), {}],
		["measure_area", func(): return bridge.measure_area(PackedVector2Array()), {}],
		["measure_radius", func(): return bridge.measure_radius(0.0, 0.0, 0.0, 0.0), {}],
		["measure_vertical", func(): return bridge.measure_vertical(0.0, 0.0, 0.0, 0.0), {}],
		["jp_options", func(): return bridge.jp_options(), {}],
		["jp_default_plan", func(): return bridge.jp_default_plan(), {}],
		["jp_plan_for_route", func(): return bridge.jp_plan_for_route(0), {}],
		["as_family_slots", func(): return bridge.as_family_slots(""), []],
		["as_slot_summary", func(): return bridge.as_slot_summary(""), {"ok": false}],
		["as_item_summary", func(): return bridge.as_item_summary("", 0), {"ok": false}],
		["as_thumbnail_png", func(): return bridge.as_thumbnail_png("", 0, 0), PackedByteArray()],
		["as_pack_info", func(): return bridge.as_pack_info(), {"name": "", "author": "", "license": "", "total_items": 0}],
		["as_validate", func(): return bridge.as_validate(), PackedStringArray()],
		["as_export_pack_bytes", func(): return bridge.as_export_pack_bytes(), {"ok": false, "error": "as_export_pack_bytes not available on this binary"}],
		["as_slicer_move_line", func(): return bridge.as_slicer_move_line(PackedFloat64Array([0.5]), 0, 0.0), PackedFloat64Array([0.5])],
		["as_uniform_lines", func(): return bridge.as_uniform_lines(0), PackedFloat64Array()],
		["continents", func(): return bridge.continents(), []],
		["vault_list_files", func(): return bridge.vault_list_files(0), PackedStringArray()],
		["vault_templates", func(): return bridge.vault_templates(), []],
		["vault_template_source", func(): return bridge.vault_template_source(), {"ok": false, "error": "this engine build predates Ruling BF"}],
		["vault_suggested_path", func(): return bridge.vault_suggested_path("", ""), ""],
		["vault_file_headings", func(): return bridge.vault_file_headings(""), []],
		["vault_read_file", func(): return bridge.vault_read_file(""), ""],
		["vault_read_file_for_edit", func(): return bridge.vault_read_file_for_edit(""), bridge._VAULT_UNAVAILABLE],
		["vault_links_for", func(): return bridge.vault_links_for("", 0), []],
		["vault_all_links", func(): return bridge.vault_all_links(), []],
		["vault_entity_summary", func(): return bridge.vault_entity_summary("", 0), {"link_count": 0, "status": ""}],
		["vault_link_text", func(): return bridge.vault_link_text(""), ""],
		["vault_preview_section_write", func(): return bridge.vault_preview_section_write(""), bridge._VAULT_UNAVAILABLE],
		["vault_entity_kinds", func(): return bridge.vault_entity_kinds(), PackedStringArray(["settlement", "province", "continent", "faction", "culture"])],
		["vault_export_fields", func(): return bridge.vault_export_fields("", 0), []],
		["vault_entity_values", func(): return bridge.vault_entity_values("", 0), {}],
		["vault_block_body", func(): return bridge.vault_block_body("", 0, PackedStringArray()), ""],
		["vault_snapshot_radii", func(): return bridge.vault_snapshot_radii("", 0), []],
		["vault_preview_block", func(): return bridge.vault_preview_block("", "", 0, ""), bridge._VAULT_UNAVAILABLE],
		["vault_preview_field_fill", func(): return bridge.vault_preview_field_fill("", "", 0, false), bridge._VAULT_UNAVAILABLE],
		["vault_entity_backlinks", func(): return bridge.vault_entity_backlinks("", 0), []],
		["vault_entity_mentions", func(): return bridge.vault_entity_mentions("", 0, "", 0), []],
		["vault_file_backlinks", func(): return bridge.vault_file_backlinks(""), []],
		["vault_file_mentions", func(): return bridge.vault_file_mentions("", 0), []],
		["vault_backlink_index_json", func(): return bridge.vault_backlink_index_json(), ""],
		["vault_state_json", func(): return bridge.vault_state_json(), ""],
		["vault_search", func(): return bridge.vault_search("", 0, 0), {"ok": false, "error": "vault_search not available on this binary", "hits": []}],
		["vault_file_data", func(): return bridge.vault_file_data(""), {"ok": false, "error": "vault_file_data not available on this binary"}],
		["vault_entity_data", func(): return bridge.vault_entity_data("", 0), []],
		["vault_entity_chronos", func(): return bridge.vault_entity_chronos("", 0), {"ok": false, "error": "vault_entity_chronos not available on this binary"}],
		["vault_link_data", func(): return bridge.vault_link_data(""), {"ok": false, "error": "vault_link_data not available on this binary"}],
		["vault_write_prefs", func(): return bridge.vault_write_prefs(), {}],
		["vault_prefs_json", func(): return bridge.vault_prefs_json(), ""],
		["lod_worker_stats", func(): return bridge.lod_worker_stats(), {}],
		["lod_budget", func(): return bridge.lod_budget(), {}],
		["info_tile_views", func(): return bridge.info_tile_views(), {}],
		["info_tile_tag", func(): return bridge.info_tile_tag(""), ""],
		["info_worker_stats", func(): return bridge.info_worker_stats(), {}],
		["jp_pack_range", func(): return bridge.jp_pack_range({}, false), {}],
	]
	for r in rows:
		var holder: Array = [null]
		var d := _fire("W " + String(r[0]), func(): holder[0] = (r[1] as Callable).call())
		_check(d == 0 and typeof(holder[0]) == typeof(r[2]) and holder[0] == r[2],
			"W: %s answers its default with no panic (got %s)" % [r[0], str(holder[0]).substr(0, 30)])
	var sp_holder: Array = [null]
	var sp_d := _fire("W as_slice_preview", func(): sp_holder[0] = bridge.as_slice_preview({}))
	_check(sp_d == 0 and sp_holder[0] is Dictionary and sp_holder[0].get("ok", true) == false,
		"W: as_slice_preview answers {ok:false} with no panic")
	var tex_holder: Array = [null]
	_fire("W color_texture", func(): tex_holder[0] = bridge.color_texture())
	_check(primed_tex != null and tex_holder[0] == primed_tex, "W: color_texture serves the last built texture, no panic")
	var tier_holder: Array = [null]
	_fire("W quality_tier", func(): tier_holder[0] = bridge.quality_tier())
	_check(tier_holder[0] == primed_tier and primed_tier != "", "W: quality_tier serves the last read name ('%s')" % str(tier_holder[0]))
	_check(not bridge.set_quality_tier("quality"), "W: set_quality_tier is refused mid-run")


## Values the bridge's last-seen caches must serve mid-run, read BEFORE the run (see `_leg_callers`).
var _primed_vault_info: Dictionary = {}
var _primed_vault_stats: Dictionary = {}


## Leg M (caller-gate audit, 2026-10-12): every MUTATOR and every `civ_*` / `tl_*` / `conflict_*` wrapper
## that was gated only by its callers is refused by the bridge itself while `generating`. One row per
## wrapper: a mutator answers its fallback default (the `_BUSY_TEXT` reason where the default carries one),
## a `void` mutator is only required not to panic. The panic count is the proof: remove a gate and the call
## reaches the borrowed `WorldGen` (`Gd<T>::bind() failed`), so the row goes red. Rows are generated from the
## gates themselves (`scratchpad/genrows.py`), arguments are each type's neutral literal, so no row can
## mutate a world if its gate regresses (the engine refuses before it can).
func _leg_mutators() -> void:
	var rows: Array = [
		["param_groups", func(): return bridge.param_groups(), PackedStringArray(), false],
		["apply_archetype", func(): return bridge.apply_archetype(""), false, false],
		["set_territory_opacity", func(): bridge.set_territory_opacity(0.0), null, true],
		["set_active_urban_rules", func(): return bridge.set_active_urban_rules({}), {}, false],
		["apply_urban_wildness", func(): return bridge.apply_urban_wildness(0.0), {}, false],
		["apply_urban_plot_chaos", func(): return bridge.apply_urban_plot_chaos(0.0), {}, false],
		["reset_active_urban_rules", func(): bridge.reset_active_urban_rules(), null, true],
		["apply_urban_rules_preset", func(): return bridge.apply_urban_rules_preset(""), {}, false],
		["civ_set_town_plan", func(): return bridge.civ_set_town_plan(0, {}), false, false],
		["recommended_quality_tier", func(): return bridge.recommended_quality_tier(), "", false],
		["set_look", func(): return bridge.set_look(""), false, false],
		["set_npr", func(): return bridge.set_npr({}), 0, false],
		["set_appearance", func(): return bridge.set_appearance({}), 0, false],
		["drop_appearance_overrides", func(): return bridge.drop_appearance_overrides(PackedStringArray()), 0, false],
		["set_color_ramp", func(): return bridge.set_color_ramp([]), 0, false],
		["load_ramp_preset", func(): return bridge.load_ramp_preset(""), false, false],
		["set_color_space", func(): return bridge.set_color_space(""), false, false],
		["set_layer_stack", func(): return bridge.set_layer_stack([]), 0, false],
		["cpu_logical_core_count", func(): return bridge.cpu_logical_core_count(), 0, false],
		["disk_free_bytes", func(): return bridge.disk_free_bytes(""), -1, false],
		["load_save", func(): return bridge.load_save(""), false, false],
		["save_project", func(): return bridge.save_project(""), false, false],
		["load_asset_pack", func(): return bridge.load_asset_pack(""), false, false],
		["has_asset_pack", func(): return bridge.has_asset_pack(), false, false],
		["civ_trait_badge_row", func(): return bridge.civ_trait_badge_row(0.0, 0.0, PackedStringArray(), 0.0, 0.0), [], false],
		["center_landmasses", func(): return bridge.center_landmasses(), {"ok": false, "reason": bridge._BUSY_TEXT}, false],
		["carve_fjords", func(): return bridge.carve_fjords(), {"ok": false, "reason": bridge._BUSY_TEXT}, false],
		["sculpt_set_globals", func(): return bridge.sculpt_set_globals({}), {}, false],
		["sculpt_set_feature", func(): return bridge.sculpt_set_feature(""), false, false],
		["sculpt_set_feature_params", func(): return bridge.sculpt_set_feature_params({}), {}, false],
		["sculpt_apply_preset", func(): return bridge.sculpt_apply_preset(0), false, false],
		["sculpt_set_freehand_mode", func(): return bridge.sculpt_set_freehand_mode(""), false, false],
		["sculpt_set_seed", func(): bridge.sculpt_set_seed(0), null, true],
		["sculpt_begin_stroke", func(): return bridge.sculpt_begin_stroke(), false, false],
		["sculpt_begin_stroke_with", func(): return bridge.sculpt_begin_stroke_with(), false, false],
		["sculpt_add_point", func(): return bridge.sculpt_add_point(0.0, 0.0), -1, false],
		["sculpt_add_point_p", func(): return bridge.sculpt_add_point_p(0.0, 0.0, null), -1, false],
		["sculpt_set_grid_snap", func(): return bridge.sculpt_set_grid_snap(0.0), 0.0, false],
		["sculpt_snap", func(): return bridge.sculpt_snap(0.0, 0.0), Vector2(0.0, 0.0), false],
		["sculpt_cancel_stroke", func(): bridge.sculpt_cancel_stroke(), null, true],
		["sculpt_select_stamp", func(): return bridge.sculpt_select_stamp(0), false, false],
		["sculpt_select_set", func(): return bridge.sculpt_select_set(PackedInt64Array()), false, false],
		["sculpt_select_all_stamps", func(): return bridge.sculpt_select_all_stamps(), 0, false],
		["sculpt_set_stamp_hidden", func(): return bridge.sculpt_set_stamp_hidden(0, false), false, false],
		["sculpt_move_stamp_up", func(): return bridge.sculpt_move_stamp_up(0), false, false],
		["sculpt_move_stamp_down", func(): return bridge.sculpt_move_stamp_down(0), false, false],
		["sculpt_undo", func(): return bridge.sculpt_undo(), false, false],
		["sculpt_redo", func(): return bridge.sculpt_redo(), false, false],
		["build_sculpt_preview_texture", func(): return bridge.build_sculpt_preview_texture(), null, false],
		["bake_visible", func(): return bridge.bake_visible(0, 0.0, 0.0, 0.0, 0.0), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["export_heightmap_png", func(): return bridge.export_heightmap_png("", 0), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["atlas_clear", func(): return bridge.atlas_clear(), 0, false],
		["atlas_evict_to", func(): return bridge.atlas_evict_to(0), 0, false],
		["atlas_import_zip", func(): return bridge.atlas_import_zip(PackedByteArray()), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["set_finalized", func(): return bridge.set_finalized(false), false, false],
		["icon_arm", func(): return bridge.icon_arm("", 0, 0.0, 0.0, 0.0), false, false],
		["icon_disarm", func(): bridge.icon_disarm(), null, true],
		["icon_place", func(): return bridge.icon_place(0.0, 0.0), -1, false],
		["icon_brush_stamp", func(): return bridge.icon_brush_stamp(0.0, 0.0), 0, false],
		["icon_select_set", func(): return bridge.icon_select_set(PackedInt64Array()), false, false],
		["icon_generate", func(): return bridge.icon_generate({}), {"ok": false, "reason": bridge._BUSY_TEXT}, false],
		["civ_pick_place_at", func(): return bridge.civ_pick_place_at(0.0, 0.0), -1, false],
		["civ_drop_settlement", func(): return bridge.civ_drop_settlement(0.0, 0.0, "", 0, "", false), -1, false],
		["civ_territory_paint_at", func(): return bridge.civ_territory_paint_at(0.0, 0.0, 0, 0.0, false), false, false],
		["civ_territory_paint_polygon", func(): return bridge.civ_territory_paint_polygon(PackedVector2Array(), 0, false), 0, false],
		["civ_territory_commit", func(): return bridge.civ_territory_commit(), false, false],
		["civ_territory_discard", func(): bridge.civ_territory_discard(), null, true],
		["civ_settlement_details", func(): return bridge.civ_settlement_details(0), {}, false],
		["civ_edit_settlement", func(): return bridge.civ_edit_settlement(0, {}), false, false],
		["civ_polity_reassign_preview", func(): return bridge.civ_polity_reassign_preview(0, 0), {}, false],
		["civ_polity_reassign", func(): return bridge.civ_polity_reassign(0, 0), {}, false],
		["civ_settlement_toggle_trait", func(): return bridge.civ_settlement_toggle_trait(0, ""), false, false],
		["civ_reroll_settlement_name", func(): return bridge.civ_reroll_settlement_name(0), "", false],
		["civ_delete_settlement", func(): return bridge.civ_delete_settlement(0), false, false],
		["civ_faction_count", func(): return bridge.civ_faction_count(), 0, false],
		["civ_add_faction", func(): return bridge.civ_add_faction(), -1, false],
		["civ_remove_faction", func(): return bridge.civ_remove_faction(), false, false],
		["civ_set_faction_field", func(): return bridge.civ_set_faction_field(0, "", ""), false, false],
		["civ_set_faction_color", func(): return bridge.civ_set_faction_color(0, Color()), false, false],
		["civ_clear_faction_color", func(): return bridge.civ_clear_faction_color(0), false, false],
		["civ_set_faction_currency", func(): return bridge.civ_set_faction_currency(0, "", ""), false, false],
		["civ_faction_currency", func(): return bridge.civ_faction_currency(0), {}, false],
		["civ_price_in_currency", func(): return bridge.civ_price_in_currency(0.0, 0), {}, false],
		["civ_set_trade_tariff", func(): return bridge.civ_set_trade_tariff(0, 0, 0.0), false, false],
		["civ_trade_tariff", func(): return bridge.civ_trade_tariff(0, 0), 0.0, false],
		["civ_has_faction_colors", func(): return bridge.civ_has_faction_colors(), false, false],
		["civ_faction_terrain_fits", func(): return bridge.civ_faction_terrain_fits(), [], false],
		["civ_faction_economy", func(): return bridge.civ_faction_economy(), [], false],
		["civ_military_summary", func(): return bridge.civ_military_summary(), {}, false],
		["civ_military_summary_at", func(): return bridge.civ_military_summary_at(0), {}, false],
		["civ_year_in_force", func(): return bridge.civ_year_in_force(0), {}, false],
		["civ_settlement_garrison", func(): return bridge.civ_settlement_garrison(0, 0), {}, false],
		["get_garrison_exposure_scale", func(): return bridge.get_garrison_exposure_scale(), 1.0, false],
		["set_garrison_exposure_scale", func(): bridge.set_garrison_exposure_scale(0.0), null, true],
		["conflict_kinds", func(): return bridge.conflict_kinds(), PackedStringArray(), false],
		["conflict_add", func(): return bridge.conflict_add({}), {"ok": false, "id": 0, "error": bridge._BUSY_TEXT}, false],
		["conflict_update", func(): return bridge.conflict_update(0, {}), {"ok": false, "id": 0, "error": bridge._BUSY_TEXT}, false],
		["conflicts_attached_to", func(): return bridge.conflicts_attached_to("", 0), PackedInt64Array(), false],
		["conflicts_touching_settlement", func(): return bridge.conflicts_touching_settlement(0), [], false],
		["conflict_sides_manpower", func(): return bridge.conflict_sides_manpower(0), [], false],
		["civ_faction_relations", func(): return bridge.civ_faction_relations(), [], false],
		["civ_agrarian_regional_total", func(): return bridge.civ_agrarian_regional_total(), {}, false],
		["civ_trait_vocabulary", func(): return bridge.civ_trait_vocabulary(), [], false],
		["civ_specialisation_vocabulary", func(): return bridge.civ_specialisation_vocabulary(), [], false],
		["civ_religion_vocabulary", func(): return bridge.civ_religion_vocabulary(), [], false],
		["civ_custom_religions", func(): return bridge.civ_custom_religions(), [], false],
		["civ_add_religion", func(): return bridge.civ_add_religion("", "", Color(), ""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["civ_edit_religion", func(): return bridge.civ_edit_religion(0, {}), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["civ_delete_religion", func(): return bridge.civ_delete_religion(0), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["civ_belief_run", func(): return bridge.civ_belief_run(0), {}, false],
		["civ_government_vocabulary", func(): return bridge.civ_government_vocabulary(), [], false],
		["civ_ag_tech_vocabulary", func(): return bridge.civ_ag_tech_vocabulary(), [], false],
		["civ_culture_vocabulary", func(): return bridge.civ_culture_vocabulary(), PackedStringArray(), false],
		["get_cultures", func(): return bridge.get_cultures(), [], false],
		["set_biome_k_enabled", func(): bridge.set_biome_k_enabled(false), null, true],
		["paint_set_layer", func(): return bridge.paint_set_layer(""), false, false],
		["paint_set_brush", func(): return bridge.paint_set_brush(0, 0.0, 0.0, 0.0, false, false), {}, false],
		["paint_stroke_at", func(): bridge.paint_stroke_at(0.0, 0.0), null, true],
		["paint_stroke_begin", func(): return bridge.paint_stroke_begin(), false, false],
		["paint_stroke_sample", func(): return bridge.paint_stroke_sample(0.0, 0.0, null, null, 0), {}, false],
		["paint_commit", func(): return bridge.paint_commit(), {}, false],
		["paint_discard", func(): return bridge.paint_discard(), -1, false],
		["way_begin", func(): return bridge.way_begin(""), false, false],
		["way_append_point", func(): return bridge.way_append_point(0.0, 0.0), false, false],
		["way_commit", func(): return bridge.way_commit(), -1, false],
		["way_discard", func(): bridge.way_discard(), null, true],
		["route_begin", func(): return bridge.route_begin(""), false, false],
		["route_append_stop", func(): return bridge.route_append_stop(0.0, 0.0), false, false],
		["route_commit", func(): return bridge.route_commit(), -1, false],
		["river_catchment", func(): return bridge.river_catchment(0), PackedByteArray(), false],
		["route_set_name", func(): return bridge.route_set_name(0, ""), false, false],
		["journey_set_resupply", func(): return bridge.journey_set_resupply(0, []), false, false],
		["journey_delete", func(): return bridge.journey_delete(0), false, false],
		["journey_positions", func(): return bridge.journey_positions(), [], false],
		["civ_settlement_journey_passes", func(): return bridge.civ_settlement_journey_passes(0), [], false],
		["measure_begin", func(): bridge.measure_begin(), null, true],
		["measure_add_point", func(): bridge.measure_add_point(0.0, 0.0), null, true],
		["measure_clear", func(): bridge.measure_clear(), null, true],
		["region_clear", func(): bridge.region_clear(), null, true],
		["region_export_tiles", func(): return bridge.region_export_tiles({}), PackedByteArray(), false],
		["export_geojson", func(): return bridge.export_geojson(), "", false],
		["label_set_class", func(): return bridge.label_set_class(0, ""), false, false],
		["label_create", func(): return bridge.label_create(0.0, 0.0, ""), -1, false],
		["label_move", func(): return bridge.label_move(0, 0.0, 0.0), false, false],
		["label_confirm_edit", func(): bridge.label_confirm_edit(), null, true],
		["label_set", func(): return bridge.label_set(0, {}), {}, false],
		["label_delete", func(): return bridge.label_delete(0), false, false],
		["label_select_set", func(): return bridge.label_select_set(PackedInt64Array()), false, false],
		["debug_layers", func(): return bridge.debug_layers(), [], false],
		["debug_texture", func(): return bridge.debug_texture(""), null, false],
		["wildlife_region_at", func(): return bridge.wildlife_region_at(0.0, 0.0), {}, false],
		["civ_territory_influence", func(): return bridge.civ_territory_influence(), {}, false],
		["civ_trade_flows", func(): return bridge.civ_trade_flows(), {}, false],
		["civ_food_shed", func(): return bridge.civ_food_shed(), {}, false],
		["civ_place_smelting", func(): return bridge.civ_place_smelting(), {}, false],
		["civ_salt_access", func(): return bridge.civ_salt_access(), {}, false],
		["world_crs", func(): return bridge.world_crs(), {}, false],
		["ecology_summary", func(): return bridge.ecology_summary(), {}, false],
		["civ_add_year", func(): bridge.civ_add_year(0), null, true],
		["civ_resync_next_tid_with_timeline", func(): return bridge.civ_resync_next_tid_with_timeline(), 0, false],
		["civ_year_diff", func(): return bridge.civ_year_diff(0), {}, false],
		["civ_year_diff_removed", func(): return bridge.civ_year_diff_removed(0), [], false],
		["civ_year_diff_removed_ways", func(): return bridge.civ_year_diff_removed_ways(0), [], false],
		["civ_settlement_ownership_periods", func(): return bridge.civ_settlement_ownership_periods(0), [], false],
		["civ_settlement_population_trajectory", func(): return bridge.civ_settlement_population_trajectory(0), [], false],
		["civ_run_collapse_simulation", func(): return bridge.civ_run_collapse_simulation({}), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["tl_counts", func(): return bridge.tl_counts(), {}, false],
		["tl_list", func(): return bridge.tl_list(""), [], false],
		["tl_get", func(): return bridge.tl_get("", ""), {"ok": false}, false],
		["tl_duplicate", func(): return bridge.tl_duplicate("", ""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["tl_add_blank", func(): return bridge.tl_add_blank("", ""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["tl_delete", func(): return bridge.tl_delete("", ""), {"ok": false}, false],
		["tl_reset_to_stock", func(): return bridge.tl_reset_to_stock(""), {"ok": false}, false],
		["tl_edit", func(): return bridge.tl_edit("", "", {}), {"ok": false, "error": bridge._BUSY_TEXT, "rejected": []}, false],
		["tl_capture_preset_from_plan", func(): return bridge.tl_capture_preset_from_plan("", {}), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["as_import_item", func(): return bridge.as_import_item("", "", PackedByteArray()), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["as_add_custom_slot", func(): return bridge.as_add_custom_slot("", ""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["as_set_pack_info", func(): return bridge.as_set_pack_info("", "", ""), false, false],
		["as_remove_item", func(): return bridge.as_remove_item("", 0), false, false],
		["as_clear_library", func(): return bridge.as_clear_library(), false, false],
		["as_apply_to_map", func(): return bridge.as_apply_to_map(), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["as_batch_tag", func(): return bridge.as_batch_tag(PackedStringArray(), ""), {"ok": false}, false],
		["as_batch_collect", func(): return bridge.as_batch_collect(PackedStringArray(), ""), {"ok": false}, false],
		["as_batch_rename", func(): return bridge.as_batch_rename(PackedStringArray(), ""), {"ok": false, "renamed": 0, "remap": {}}, false],
		["as_batch_duplicate", func(): return bridge.as_batch_duplicate(PackedStringArray()), {"ok": false, "made": 0}, false],
		["as_batch_delete", func(): return bridge.as_batch_delete(PackedStringArray()), {"ok": false, "deleted": 0}, false],
		["as_load_sheet", func(): return bridge.as_load_sheet("", PackedByteArray()), {"ok": false, "error": bridge._BUSY_TEXT, "w": 0, "h": 0}, false],
		["as_clear_sheet", func(): return bridge.as_clear_sheet(), false, false],
		["as_slice_apply", func(): return bridge.as_slice_apply({}), {"ok": false, "error": bridge._BUSY_TEXT, "added": 0, "skipped_blank": 0, "unplaced": 0, "uids": PackedStringArray()}, false],
		["as_set_item_transform", func(): return bridge.as_set_item_transform("", 0, 0.0, 0.0, 0.0), false, false],
		["as_reset_item_transform", func(): return bridge.as_reset_item_transform("", 0, false), {"ok": false}, false],
		["vault_connect", func(): return bridge.vault_connect(""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["vault_disconnect", func(): bridge.vault_disconnect(), null, true],
		["vault_connect_saf", func(): return bridge.vault_connect_saf("", "", Callable()), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["vault_insert_template", func(): return bridge.vault_insert_template("", "", 0, ""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["vault_create_from_template", func(): return bridge.vault_create_from_template("", "", ""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["vault_write_file", func(): return bridge.vault_write_file("", "", ""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["vault_attach", func(): return bridge.vault_attach("", 0, "", "", ""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["vault_detach", func(): return bridge.vault_detach(""), false, false],
		["vault_set_link_text", func(): return bridge.vault_set_link_text("", ""), false, false],
		["vault_reload_link", func(): return bridge.vault_reload_link(""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["vault_write_section", func(): return bridge.vault_write_section("", ""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["vault_snapshot", func(): return bridge.vault_snapshot("", 0, "", "", 0), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["vault_write_block", func(): return bridge.vault_write_block("", "", 0, "", ""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["vault_refresh_backlinks", func(): return bridge.vault_refresh_backlinks(), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["vault_rebuild_backlinks", func(): bridge.vault_rebuild_backlinks(), null, true],
		["vault_restore_backlink_index", func(): return bridge.vault_restore_backlink_index(""), false, false],
		["vault_restore_state", func(): return bridge.vault_restore_state(""), false, false],
		["vault_add_chronos_event", func(): return bridge.vault_add_chronos_event("", "", {}), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["vault_set_write_pref", func(): return bridge.vault_set_write_pref("", false), false, false],
		["vault_restore_prefs", func(): return bridge.vault_restore_prefs(""), false, false],
		["vault_remove_block", func(): return bridge.vault_remove_block("", "", 0, ""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["project_save_with_documents", func(): return bridge.project_save_with_documents("", {}), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["project_engine_built_documents", func(): return bridge.project_engine_built_documents(), {}, false],
		["sculpt_restore_document", func(): return bridge.sculpt_restore_document(""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["paint_restore_document", func(): return bridge.paint_restore_document(""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["asset_library_restore_document", func(): return bridge.asset_library_restore_document(""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["travel_library_restore_document", func(): return bridge.travel_library_restore_document(""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
		["project_document_slots", func(): return bridge.project_document_slots(), PackedStringArray(), false],
		["project_engine_owned_slots", func(): return bridge.project_engine_owned_slots(), PackedStringArray(), false],
		["project_format_version", func(): return bridge.project_format_version(), 0, false],
		["get_villages_enabled", func(): return bridge.get_villages_enabled(), true, false],
		["get_metropolis_enabled", func(): return bridge.get_metropolis_enabled(), false, false],
		["get_recovery_phase", func(): return bridge.get_recovery_phase(), 0, false],
		["civ_regional_population", func(): return bridge.civ_regional_population(), {}, false],
		["as_drop_collection", func(): return bridge.as_drop_collection(""), {"ok": false, "error": bridge._BUSY_TEXT}, false],
	]
	var bad := 0
	for r in rows:
		var holder: Array = [null]
		var before := _pl.hits
		(r[1] as Callable).call() if r[3] else (func(): holder[0] = (r[1] as Callable).call()).call()
		var d := _pl.hits - before
		var ok: bool = d == 0 and (r[3] or (typeof(holder[0]) == typeof(r[2]) and holder[0] == r[2]))
		if not ok:
			bad += 1
			_check(false, "M: %s answers its default with no panic (panics %d, got %s)" % [r[0], d, str(holder[0]).substr(0, 40)])
	_check(bad == 0, "M: all %d mutator and civ/tl/conflict wrappers refuse mid-run with no panic and their default" % rows.size())
	_check(rows.size() >= 200, "M: the table is not silently short (%d rows)" % rows.size())
	## atlas_ready is gated by hand (a cached flag answers first): with no root set it must not reach the engine.
	var ar_before := _pl.hits
	var ar: bool = bridge.atlas_ready()
	_check(_pl.hits == ar_before, "M: atlas_ready makes no engine call mid-run (answered %s)" % str(ar))


func _carto_ws() -> Node:
	for ws in app._workspaces:
		if ws != null and ws is CartographyWorkspace:
			return ws
	return null


func _world_ws() -> Node:
	for ws in app._workspaces:
		if ws != null and ws is WorldWorkspace:
			return ws
	return null


## Leg Y: every caller of a `#[func]` read that has no honest empty default (so the bridge either serves a
## last-seen value or the CALLER refuses). Protects `engine_bridge.gd`'s `_*_seen` caches, `bake_all` /
## `set_atlas_tile_size` gates and the caller gates in viewport_host, cartography_workspace, menus and
## world_workspace. Removing any of them reaches the borrowed `WorldGen` and counts a panic here. The
## journey planner's `_sync_lod` (an inner class needing a route) is covered by reading only.
func _leg_callers() -> void:
	var vp = app.viewport
	_fire("Y viewport._apply_river_paint", func(): vp._apply_river_paint())
	_check(vp.visible_grid_rect().get("ok", true) == false, "Y: visible_grid_rect answers {ok:false} mid-run")
	_fire("Y viewport.layer_visible(rivers)", func(): vp.layer_visible("rivers"))
	var img := Image.create_empty(4, 4, false, Image.FORMAT_RGBA8)
	var small := ImageTexture.create_from_image(img)
	var holder: Array = ["unset"]
	_fire("Y viewport._lod_tile_rect", func(): holder[0] = vp._lod_tile_rect(Vector3i(1, 0, 0), small, Vector2i(192, 124), Vector2.ZERO, Vector2(100, 100)))
	_check(holder[0] == null, "Y: _lod_tile_rect places nothing mid-run")
	var carto = _carto_ws()
	_check(carto != null, "Y: cartography workspace found")
	if carto != null:
		for mode in [carto.DragMode.RESIZE, carto.DragMode.ROTATE, carto.DragMode.ARC]:
			carto._label_drag_index = 0
			carto._label_drag_mode = mode
			_fire("Y label drag mode %d" % mode, func(): carto._on_label_drag(5.0, 5.0))
		carto._label_drag_index = -1
	var menus = app.menus
	_fire("Y menus._refresh_tile_size_menu", func(): menus._refresh_tile_size_menu())
	_check(bridge.atlas_tile_size() == 512, "Y: atlas_tile_size serves the last-seen 512, not the 1024 default (got %d)" % bridge.atlas_tile_size())
	_fire("Y menus._on_tile_size", func(): menus._on_tile_size(DccMenus.ID_LOD_TILE_FIRST))
	_check("Not done: Atlas tile size" in _hint(), "Y: tile-size change refused with the hint ('%s')" % _hint().substr(0, 40))
	_check(bridge.atlas_tile_size() == 512, "Y: refused change left the size at 512")
	var world = _world_ws()
	_check(world != null, "Y: world workspace found")
	if world != null:
		_fire("Y world._on_bake_all", func(): world._on_bake_all())
		_check("Not done: Bake ALL" in _hint(), "Y: Bake ALL refused with the hint")
	_fire("Y bake_all", func(): holder[0] = bridge.bake_all(0))
	_check(holder[0] is Dictionary and holder[0].get("ok", true) == false, "Y: bake_all answers {ok:false} mid-run")
	_fire("Y context_broker.ring_domain_req", func(): holder[0] = app.context_broker.ring_domain_req())
	_check(holder[0] is Dictionary and holder[0].has("finalized"), "Y: ring request still built mid-run")
	_fire("Y is_finalized", func(): bridge.is_finalized())
	_fire("Y vault_info", func(): holder[0] = bridge.vault_info())
	_check(holder[0] == _primed_vault_info and not (holder[0] as Dictionary).is_empty(), "Y: vault_info serves the last-seen dictionary (%d keys)" % (holder[0] as Dictionary).size())
	_fire("Y vault_backlink_stats", func(): holder[0] = bridge.vault_backlink_stats())
	_check(holder[0] == _primed_vault_stats and not (holder[0] as Dictionary).is_empty(), "Y: vault_backlink_stats serves the last-seen dictionary")
	_fire("Y vault_backlink_report", func(): holder[0] = bridge.vault_backlink_report(40))
	_check(holder[0] is Dictionary and holder[0].get("built", true) == false, "Y: vault_backlink_report answers built:false")


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var ti := args.find("--tag")
	if ti >= 0 and ti + 1 < args.size():
		_tag = String(args[ti + 1])
	if not _guard():
		_log("ABORT run WINDOWED")
		get_tree().quit(2)
		return
	_pl = PanicLogger.new()
	OS.add_logger(_pl)
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	bridge = app.bridge
	_log("user dir %s" % OS.get_user_data_dir())
	_log("DLL mtime (unix) %d" % FileAccess.get_modified_time(ProjectSettings.globalize_path("res://").path_join("../target/debug/cartalith_godot.dll")))

	# -- a world first, so every path has something to read --------------------------------------
	bridge.generate({"seed": 4242, "width_km": 800.0, "grid_w": 192, "grid_h": 124, "sea_level": 0.5, "villages": true})
	await bridge.generation_finished
	await _frames(12)
	_check(bridge.has_world, "setup: a world exists")
	app.arm_tool("inspect")
	await _frames(4)
	var menus = app.menus
	var civ := _civ_ws()
	_check(civ != null, "setup: civilisation workspace found")
	app.resource_overlay.visible = true
	await _frames(2)
	var tl_ok: bool = app.tl_available()
	if tl_ok and not app.tl_playing:
		app.tl_toggle_play()
	_log("timeline available=%s playing=%s" % [tl_ok, app.tl_playing])
	menus._clipboard = {"labels": [{"x": 20.0, "y": 20.0, "text": "GSW"}]}
	## Primed BEFORE the run: `color_texture()` / `quality_tier()` answer the last value they read while
	## `generating`, so there must be one to answer.
	var primed_tex: Texture2D = bridge.color_texture()
	var primed_tier: String = bridge.quality_tier()
	## Leg Y priming: a non-default tile size and the vault reads, so a cache is told apart from a default.
	bridge.set_atlas_tile_size(512)
	bridge.atlas_tile_size()
	_primed_vault_info = bridge.vault_info()
	_primed_vault_stats = bridge.vault_backlink_stats()
	var hits_idle := _pl.hits
	_check(hits_idle == 0, "setup: 0 panics before the run (got %d)" % hits_idle)

	# -- the run ----------------------------------------------------------------------------------
	bridge.generate({"seed": 777, "width_km": 1600.0, "grid_w": 512, "grid_h": 320, "sea_level": 0.5, "villages": true})
	await _frames(2)
	_check(bridge.generating, "run: bridge.generating is true when the legs fire")
	var t0 := Time.get_ticks_msec()

	_fire("R resource_overlay._refresh direct", func(): app.resource_overlay._refresh())
	_check(String(app.resource_overlay._label.text).begins_with("Generating"), "R: overlay text says it is held ('%s')" % String(app.resource_overlay._label.text).substr(0, 40))
	_fire("T dcc_shell._tl_tick direct x3", func():
		app._tl_tick()
		app._tl_tick()
		app._tl_tick())
	_fire("C civ._tl_on_play_tick direct x2", func():
		civ._tl_on_play_tick()
		civ._tl_on_play_tick())
	var wins_before := _vp.get_embedded_subwindows().size()
	_fire("B WM_GO_BACK_REQUEST", func(): app.notification(NOTIFICATION_WM_GO_BACK_REQUEST))
	_check("Not done: Back" in _hint(), "B: refused with the hint ('%s')" % _hint().substr(0, 50))
	_check(_vp.get_embedded_subwindows().size() == wins_before, "B: no exit card opened mid-run")
	var holder: Array = [null]
	_fire("X confirm_unsaved_world (OS close card)", func():
		holder[0] = app.confirm_unsaved_world("Exit Cartalith", "Exit the app?", "Discard and exit", "Save and exit", func(): pass))
	var card_mid: AcceptDialog = holder[0]
	_check(card_mid != null and "the world is generating; its seed is readable" in _texts(card_mid), "X: seed dashed with its reason")
	if card_mid != null:
		card_mid.hide()
	_fire("U undo budget rung 128 MB", func(): menus._on_undo_budget(1))
	_check("applied when the run finishes" in _hint(), "U: rung says it is deferred ('%s')" % _hint().substr(0, 50))
	_check(_pending() == 128, "U: 128 MB pending (got %d)" % _pending())
	_fire("U undo Clear", func(): menus._on_undo_budget(DccMenus.ID_PREF_UNDO_CLEAR))
	_check("Not done: Clear undo history" in _hint(), "U: Clear refused with the hint")
	_fire("D Edit > Redo", func(): menus._on_edit(DccMenus.ID_REDO))
	_check("Not done: Redo" in _hint(), "D: refused with the hint")
	_leg_reads(primed_tex, primed_tier)
	_leg_mutators()
	_leg_callers()
	_fire("P menus._paste_clipboard", func(): menus._paste_clipboard())
	_check("Not done: Paste" in _hint(), "P: paste refused with the hint")
	_check(not menus._clipboard.is_empty(), "P: clipboard kept for after the run")
	_fire("P menus._cut_selection", func(): menus._cut_selection())
	_check("Not done: Cut" in _hint(), "P: cut refused")
	_fire("P menus._copy_selection", func(): menus._copy_selection())
	_check("Not done: Copy" in _hint(), "P: copy refused")
	_fire("P menus._select_all", func(): menus._select_all())
	_check("Not done: Select all" in _hint(), "P: select all refused")
	_fire("K right_dock paint commit", func(): app.right_dock_ctrl._on_paint_commit_from_dock())
	_check("Not done: Commit paint" in _hint(), "K: commit refused")
	_fire("K right_dock paint discard", func(): app.right_dock_ctrl._on_paint_discard_from_dock())
	_check("Not done: Discard paint draft" in _hint(), "K: discard refused")

	_log("direct legs fired in %d ms" % (Time.get_ticks_msec() - t0))
	await _wait_done()

	# -- the timer legs: a second, larger run (1024 x 640) long enough for the 0.5 s overlay timer and
	# the 0.6 s playback timer to fire while it holds the engine; each timeout is counted only while
	# `generating`, so a pass cannot come from timers that fired after the run.
	if not app.tl_playing and app.tl_available():
		app.tl_toggle_play()
	var fired := {"overlay": 0, "playback": 0}
	app.resource_overlay._timer.timeout.connect(func(): if bridge.generating: fired["overlay"] += 1)
	if app._tl_timer != null:
		app._tl_timer.timeout.connect(func(): if bridge.generating: fired["playback"] += 1)
	bridge.generate({"seed": 778, "width_km": 1600.0, "grid_w": 1024, "grid_h": 640, "sea_level": 0.5, "villages": true})
	await _frames(2)
	var before_timers := _pl.hits
	var t1 := Time.get_ticks_msec()
	while bridge.generating and Time.get_ticks_msec() - t1 < 1600:
		await _frames(1)
	var held_text := String(app.resource_overlay._label.text)
	var still := bool(bridge.generating)
	_check(still, "timers: still generating after %d ms" % (Time.get_ticks_msec() - t1))
	_check(int(fired["overlay"]) >= 2 and int(fired["playback"]) >= 2, "timers: overlay fired %d, playback fired %d times under the run" % [int(fired["overlay"]), int(fired["playback"])])
	_check(_pl.hits - before_timers == 0, "R/T timers: 0 engine panics while ticking under the run (got %d)" % (_pl.hits - before_timers))
	_check(held_text.begins_with("Generating"), "R: timer kept the held text")
	await _wait_done()
	_check(not bridge.generating, "after: run finished")
	if app.tl_playing:
		app.tl_toggle_play()
	_check(not String(app.resource_overlay._label.text).begins_with("Generating"), "R after: overlay redrawn ('%s')" % String(app.resource_overlay._label.text).substr(0, 30))
	_check(_pending() == -1, "U after: nothing pending")
	var us: Dictionary = bridge.undo_stats()
	_check(int(us.get("budget_bytes", -1)) == 128 * 1048576, "U after: engine budget is 128 MB (budget_bytes %d)" % int(us.get("budget_bytes", -1)))
	var y0: int = app.tl_year()
	app._tl_tick()
	var y1: int = app.tl_year()
	_check(y1 == mini(y0 + int(app.tl_speed), DccShell.TL_YEAR_MAX), "T after: a tick advances the year (%d -> %d)" % [y0, y1])
	menus._paste_clipboard()
	var pass_text := String((app._status_labels["pass"] as Label).text)
	_check(pass_text.begins_with("pasted 1"), "P after: the kept paste lands ('%s')" % pass_text.substr(0, 40))
	menus._on_edit(DccMenus.ID_REDO)
	_check(not ("Not done" in _hint()), "D after: redo reaches the engine ('%s')" % _hint().substr(0, 40))
	app.right_dock_ctrl._on_paint_commit_from_dock()
	_check(_hint().begins_with("painted"), "K after: commit runs ('%s')" % _hint().substr(0, 30))
	var card_after: AcceptDialog = app.confirm_unsaved_world("Exit Cartalith", "Exit the app?", "Discard and exit", "Save and exit", func(): pass)
	_check(card_after != null and str(bridge.world_gen.get_seed()) in _texts(card_after), "X after: the card shows the real seed")
	if card_after != null:
		card_after.hide()
	await _frames(4)
	app.notification(NOTIFICATION_WM_GO_BACK_REQUEST)
	await _frames(4)
	var exit_cards := _windows_with("Exit the app?")
	_check(exit_cards.size() >= 1, "B after: back reaches the exit card (%d open)" % exit_cards.size())
	for w in exit_cards:
		(w as Window).hide()
	await _frames(4)

	_log("panic frames (first %d):" % _pl.lines.size())
	for l in _pl.lines:
		_log("    " + l)
	_log("RESULT %s checks=%d fail=%d engine-panic-hits(total)=%d" % [_tag, _checks, _fail, _pl.hits])
	get_tree().quit(1 if _fail > 0 else 0)

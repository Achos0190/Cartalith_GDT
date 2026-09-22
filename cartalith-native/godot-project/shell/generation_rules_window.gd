extends AcceptDialog
class_name GenerationRulesWindow

## Design canvas artboard `1h` ("Generation rules"), `lazy-riding-piglet.md`
## Batch F -- the last batch of the Settlement Editor plan, and the one that
## needed a real Rust signature change first: `urban_adapter::
## settlement_layout_with` took no `rules` argument at all, so nothing built
## here could ever have reached the generator. That change landed the same
## batch as this window (`cartalith-urban/src/rules.rs`'s `Rules::to_patch`,
## `urban_adapter.rs`'s `run_layout`/`settlement_layout_with`, `urban_bridge.rs`'s
## new `#[func]`s) -- see those files' own doc comments for the golden-parity
## proof that threading the parameter changed no existing town.
##
## World-level, matching the canvas's own framing and the reference tool's own
## scope -- not per-faction. The canvas's closing panel names the per-faction
## fork explicitly as a later owner decision, not one to make speculatively
## here.
##
## ## The two sliders call the real engine formulas, not a GDScript copy
##
## The canvas's own script (`Cartalith Settlement Editor.dc.html`, the
## `renderVals()` block) computes `ruleStreet`/`ruleParcels` with
## `cl(0.26*w,0.15,0.70)` and its nine siblings -- bit-for-bit
## `cartalith_urban::apply_wildness`/`apply_plot_chaos`'s own formulas, which
## are golden-verified against the reference HTML. Rather than re-deriving
## that arithmetic here a second time (a second copy that could drift from
## the first), `apply_urban_wildness`/`apply_urban_plot_chaos` below call
## `EngineBridge.apply_urban_wildness`/`apply_urban_plot_chaos`, which run the
## real Rust functions on the current active rules and hand back the whole
## resulting rule set. What this window shows is what `generate()` actually
## uses, not a parallel computation of it.
##
## ## Presets are this window's own invention, not a Rust table
##
## `cartalith-urban/src/rules.rs` has exactly one axis of named bundles --
## `CultureProfile` ("medieval"/"venus"), which governs street-planning MODE,
## building grammar and civic naming, not the jitter/variance dials this
## window edits. It carries no "Planned Grid"/"Classical Town"/"Medina"/"Wild
## Frontier" table at all (grepped). The five presets the canvas draws are
## therefore new, GDScript-only (wildness, plot chaos) coordinate pairs
## defined in `_PRESETS` below -- a convenience starting point on the same
## two-slider space, not a reverse-engineered Rust constant.
##
## **The exception is one chip that is a Rust rule set, not a slider pair**:
## "Walled Market Town" (`"rust": "market_town"`) is
## `cartalith_urban::MARKET_TOWN_RULES`, shaped on the owner's town plan under
## Ruling H. It sets the fields directly through
## `EngineBridge.apply_urban_rules_preset`, so the values live only in Rust. That
## constant's doc explains each value.
##
## ## The wall-generation block
##
## `grow` reads all five `rules.settlement` fields (`cartalith-urban/src/
## growth.rs`, the wall-generation branch; `urban_adapter::run_layout` passes
## `wall_generations: true`). This window has no editor for them and keeps the
## canvas's dashed panel, but it shows the active values, which the market-town
## preset changes.
##
## ## Parameter tables are read-only
##
## The canvas draws a computed-value column and a default column, not an
## editable field per row -- the two sliders and the five presets are the
## only writers this window offers. `EngineBridge.set_active_urban_rules`
## (the direct per-field write path `urban_bridge.rs` also exposes) has no
## caller here; a future per-field editor would use it, not a new bridge
## call.

var app                       ## `DccApp`
var bridge: EngineBridge

var _rules: Dictionary = {}   ## last `get_active_urban_rules()` read
var _defaults: Dictionary = {}
var _active_preset := ""      ## "" once a slider/table diverges from every preset

var _presets_row: HBoxContainer
var _street_body: VBoxContainer
var _parcels_body: VBoxContainer
var _settlement_body: VBoxContainer
var _wild_readout: Label
var _chaos_readout: Label
var _status_label: Label

var _phone := false

## New, GDScript-only bundles -- see this file's own header. `(wildness,
## plot_chaos)` pairs on the reference's own `[0, 2]` slider domain; "Organic
## Medieval" is `(1.0, 1.0)`, the exact `DEFAULT_RULES` baseline, matching the
## canvas's own default-selected preset.
const _PRESETS := [
	{"label": "Planned Grid", "w": 0.3, "c": 0.3},
	{"label": "Classical Town", "w": 0.7, "c": 0.6},
	{"label": "Organic Medieval", "w": 1.0, "c": 1.0},
	{"label": "Medina", "w": 1.4, "c": 1.3},
	{"label": "Wild Frontier", "w": 2.0, "c": 1.6},
	{"label": "Walled Market Town", "rust": "market_town"},
]

## label, dotted key, unit -- the canvas's own `ruleStreet` rows, in its own
## order.
const _STREET_ROWS := [
	["Branch angle jitter (σ rad)", "street.branch_angle_jitter", ""],
	["Continuation jitter (σ rad)", "street.continuation_jitter", ""],
	["Exploration share (start)", "street.exploration_start", ""],
	["Exploration decay / epoch", "street.exploration_decay", ""],
	["Exploration minimum", "street.exploration_minimum", ""],
	["Segment length median (m)", "street.segment_length_median", ""],
	["Segment length variance (σ)", "street.segment_length_variance", ""],
	["Pierce chance", "street.pierce_chance", ""],
	["Junction angle limit (rad)", "street.junction_angle_limit", ""],
	["Market gradient decay", "street.market_gradient_decay", ""],
	["Parallel street spacing (m)", "street.parallel_street_spacing", ""],
	["Dead-end bias", "street.dead_end_bias", ""],
	["Bridgehead distance (m)", "street.bridgehead_distance", ""],
	["Bridgehead probability", "street.bridgehead_probability", ""],
]
const _PARCEL_ROWS := [
	["Frontage width variance (σ)", "parcels.frontage_width_variance", ""],
	["Plot depth variance (σ)", "parcels.plot_depth_variance", ""],
	["Subdivision cap", "parcels.subdivision_cap", ""],
]
const _SETTLEMENT_ROWS := [
	["Wall-generation threshold", "settlement.wall_generation_threshold"],
	["Min. years between circuits", "settlement.wall_generation_min_age_gap"],
	["Min. extramural share", "settlement.wall_generation_extramural_share"],
	["Max wall generations", "settlement.max_wall_generations"],
	["Carrying-capacity weight", "settlement.carrying_capacity_weight"],
]


func setup(a, b: EngineBridge) -> void:
	app = a
	bridge = b
	title = "⌧ GENERATION RULES"
	get_ok_button().hide()
	wrap_controls = false
	size = Vector2i(980, 660)
	min_size = Vector2i(760, 520)
	max_size = Vector2i(1180, 900)
	_phone = DccWidgets.phone_window(self, a)
	_build()
	if _phone:
		app.phone_fit(self, 1.0)


func open() -> void:
	if not DccWidgets.phone_present(self, app):
		popup_centered()
	_rebuild()


func _clear(node: Control) -> void:
	for c in node.get_children():
		node.remove_child(c)
		c.queue_free()


func _rebuild() -> void:
	_defaults = bridge.get_default_urban_rules()
	_rules = bridge.get_active_urban_rules()
	_rebuild_presets()
	_rebuild_sliders()
	_rebuild_tables()
	_rebuild_status()


# ---------------------------------------------------------------------------
# Layout
# ---------------------------------------------------------------------------

func _build() -> void:
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)

	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 12)
	head_pad.add_theme_constant_override("margin_top", 6)
	head_pad.add_theme_constant_override("margin_right", 12)
	head_pad.add_theme_constant_override("margin_bottom", 6)
	outer.add_child(head_pad)
	var head_row := HBoxContainer.new()
	head_row.add_theme_constant_override("separation", 12)
	head_pad.add_child(head_row)
	if not _phone:
		head_row.add_child(DccTheme.mono_label(
			"⌧ GENERATION RULES", "accent", DccTheme.FS_HEADER, 2, true))
		var sub := DccTheme.label("civ ▸ urban ▸ rules", "text_ghost", DccTheme.FS_TINY)
		head_row.add_child(sub)
		head_row.add_child(DccTheme.spacer())
	else:
		head_row.add_child(DccTheme.spacer())
	var close_btn := Button.new()
	close_btn.text = "Close"
	close_btn.focus_mode = Control.FOCUS_NONE
	close_btn.pressed.connect(func(): hide())
	head_row.add_child(close_btn)
	outer.add_child(DccTheme.rule())

	DccWidgets.note(outer, "World-level: one active rule set feeds every settlement's layout, "
		+ "same as the reference tool. Move a slider and the parameters it drives recompute "
		+ "through the real clamp formulas, so you can see which parameters a slider actually "
		+ "owns and where each one pins.")

	_presets_row = HBoxContainer.new()
	_presets_row.add_theme_constant_override("separation", 4)
	var presets_pad := MarginContainer.new()
	presets_pad.add_theme_constant_override("margin_left", 12)
	presets_pad.add_theme_constant_override("margin_right", 12)
	presets_pad.add_theme_constant_override("margin_bottom", 6)
	presets_pad.add_child(_presets_row)
	outer.add_child(presets_pad)

	var main: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	main.add_theme_constant_override("separation", 0)
	main.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(main)

	main.add_child(_build_sliders_pane())
	if not _phone:
		main.add_child(DccTheme.rule(true))
	main.add_child(_build_tables_pane())

	outer.add_child(DccTheme.rule())
	_status_label = DccTheme.label("", "text_ghost", DccTheme.FS_MICRO)
	var status_pad := MarginContainer.new()
	status_pad.add_theme_constant_override("margin_left", 12)
	status_pad.add_theme_constant_override("margin_top", 3)
	status_pad.add_theme_constant_override("margin_bottom", 3)
	status_pad.add_child(_status_label)
	outer.add_child(status_pad)


func _apply_preset(p: Dictionary) -> void:
	_active_preset = String(p.label)
	if p.has("rust"):
		bridge.apply_urban_rules_preset(String(p.rust))
	else:
		bridge.apply_urban_wildness(float(p.w))
		bridge.apply_urban_plot_chaos(float(p.c))
	_rebuild()


func _do_reset() -> void:
	_active_preset = "Organic Medieval"
	bridge.reset_active_urban_rules()
	_rebuild()


func _rebuild_presets() -> void:
	_clear(_presets_row)
	for p in _PRESETS:
		var on := _active_preset == String(p.label)
		var on_press := func(): _apply_preset(p)
		var chip := DccWidgets.chip(_presets_row, String(p.label), on_press, on, 10, 4)
		if p.has("rust"):
			chip.tooltip_text = ("Rule set shaped on the owner's walled market-town plan: wider "
				+ "street spacing for legible blocks and one wall circuit (MARKET_TOWN_RULES).")
		else:
			chip.tooltip_text = "wildness %.2f · plot chaos %.2f" % [float(p.w), float(p.c)]
	var reset := DccWidgets.chip(_presets_row, "Reset", _do_reset)
	reset.tooltip_text = "Back to DEFAULT_RULES -- the same set every existing golden-parity test is pinned against."


# -- Left pane: the two convenience sliders ----------------------------------

func _build_sliders_pane() -> Control:
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	if not _phone:
		col.custom_minimum_size.x = 300
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_top", 10)
	pad.add_theme_constant_override("margin_right", 12)
	pad.add_theme_constant_override("margin_bottom", 10)
	col.add_child(pad)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 14)
	pad.add_child(body)

	var wild := DccWidgets.slider(body, "Wildness", 0.0, 2.0, 0.05, 1.0, "", func(v: float):
		_active_preset = ""
		bridge.apply_urban_wildness(v)
		_rebuild_tables()
		_rebuild_status())
	_wild_readout = wild["readout"]
	DccWidgets.note(body, "The primary organic-versus-planned control. Drives ten street "
		+ "parameters; 1.00 is the baseline that reproduces DEFAULT_RULES exactly.")

	var chaos := DccWidgets.slider(body, "Plot chaos", 0.0, 2.0, 0.05, 1.0, "", func(v: float):
		_active_preset = ""
		bridge.apply_urban_plot_chaos(v)
		_rebuild_tables()
		_rebuild_status())
	_chaos_readout = chaos["readout"]
	DccWidgets.note(body, "Parcel-level irregularity, independent of the street network. "
		+ "Drives the three parcel parameters.")

	var why := DccWidgets.section(body, "Why two layers")
	DccWidgets.note(why, "The sliders are a convenience that computes values for the "
		+ "individual fields; the fields stay the single source of truth the generator "
		+ "reads (cartalith_urban::apply_wildness/apply_plot_chaos, called through the "
		+ "bridge -- not reimplemented here).")
	return col


# -- Right pane: the parameter tables -----------------------------------------

func _build_tables_pane() -> Control:
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col.size_flags_vertical = Control.SIZE_EXPAND_FILL

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(scroll)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_top", 10)
	pad.add_theme_constant_override("margin_right", 12)
	pad.add_theme_constant_override("margin_bottom", 10)
	scroll.add_child(pad)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 10)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(body)

	var street_head := HBoxContainer.new()
	street_head.add_theme_constant_override("separation", 10)
	street_head.add_child(DccTheme.mono_label("STREET", "text_dim", DccTheme.FS_TINY, 2))
	street_head.add_child(DccTheme.label("14 parameters · read by the ported growth stage",
		"text_ghost", DccTheme.FS_TINY))
	body.add_child(street_head)
	_street_body = VBoxContainer.new()
	_street_body.add_theme_constant_override("separation", 2)
	body.add_child(_street_body)

	var parcels_head := HBoxContainer.new()
	parcels_head.add_theme_constant_override("separation", 10)
	parcels_head.add_child(DccTheme.mono_label("PARCELS", "text_dim", DccTheme.FS_TINY, 2))
	parcels_head.add_child(DccTheme.label("3 · read by the platting stage",
		"text_ghost", DccTheme.FS_TINY))
	body.add_child(parcels_head)
	_parcels_body = VBoxContainer.new()
	_parcels_body.add_theme_constant_override("separation", 2)
	body.add_child(_parcels_body)

	var settlement_head := HBoxContainer.new()
	settlement_head.add_theme_constant_override("separation", 10)
	settlement_head.add_child(DccTheme.mono_label(
		"SETTLEMENT · WALL GENERATIONS", "text_ghost", DccTheme.FS_TINY, 2))
	settlement_head.add_child(DccTheme.mono_label("5 · READ BY GROW · NO EDITOR", "text_ghost", DccTheme.FS_TINY))
	body.add_child(settlement_head)
	var dashed := PanelContainer.new()
	dashed.add_theme_stylebox_override("panel", DccWidgets.box("line_soft", "", 9, 7))
	body.add_child(dashed)
	_settlement_body = VBoxContainer.new()
	_settlement_body.add_theme_constant_override("separation", 2)
	dashed.add_child(_settlement_body)
	DccWidgets.note(body, "These five govern successive wall circuits, and the growth stage "
		+ "reads them. The window has no editor for them; a preset can set them, and this "
		+ "shows the active values.")
	return col


func _param_row(parent: Control, label_text: String, val: String, def: String, driven: bool) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 9)
	var lbl := DccTheme.label(label_text, "text", DccTheme.FS_SMALL)
	lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(lbl)
	var val_lbl := DccTheme.mono_label(val, "accent" if driven else "text", DccTheme.FS_SMALL)
	val_lbl.custom_minimum_size.x = 54
	val_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	row.add_child(val_lbl)
	var def_lbl := DccTheme.mono_label(def, "text_ghost", DccTheme.FS_TINY)
	def_lbl.custom_minimum_size.x = 44
	def_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	row.add_child(def_lbl)
	parent.add_child(row)
	parent.add_child(DccTheme.rule())


static func _fmt(v: float) -> String:
	return "%.2f" % v


func _rebuild_tables() -> void:
	_rules = bridge.get_active_urban_rules()
	_clear(_street_body)
	for r in _STREET_ROWS:
		var key := String(r[1])
		var v := float(_rules.get(key, 0.0))
		var d := float(_defaults.get(key, 0.0))
		_param_row(_street_body, String(r[0]), _fmt(v), _fmt(d), not is_equal_approx(v, d))
	_clear(_parcels_body)
	for r in _PARCEL_ROWS:
		var key := String(r[1])
		var v := float(_rules.get(key, 0.0))
		var d := float(_defaults.get(key, 0.0))
		_param_row(_parcels_body, String(r[0]), _fmt(v), _fmt(d), not is_equal_approx(v, d))
	_clear(_settlement_body)
	for r in _SETTLEMENT_ROWS:
		var key := String(r[1])
		var v := float(_rules.get(key, _defaults.get(key, 0.0)))
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 9)
		var lbl := DccTheme.label(String(r[0]), "text_ghost", DccTheme.FS_SMALL)
		lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		row.add_child(lbl)
		var val_lbl := DccTheme.mono_label(_fmt(v), "text_ghost", DccTheme.FS_SMALL)
		val_lbl.custom_minimum_size.x = 54
		val_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		row.add_child(val_lbl)
		_settlement_body.add_child(row)
	if _wild_readout != null:
		_wild_readout.text = _fmt(float(_rules.get("meta.wildness", 1.0)))
	if _chaos_readout != null:
		_chaos_readout.text = _fmt(float(_rules.get("meta.plot_chaos", 1.0)))


func _rebuild_sliders() -> void:
	# Sliders own their own readouts (`DccWidgets.slider`'s own contract); this
	# rebuild only needs to happen once, in `_rebuild()`, since the sliders'
	# starting position is read straight from `_rules` at build time via
	# `_rebuild_tables()`'s readout sync above -- no separate slider redraw is
	# needed on every parameter change, only its readout.
	pass


func _rebuild_status() -> void:
	var edited := not _rules.is_empty() and _rules != _defaults
	_status_label.text = (
		"Active rules %s DEFAULT_RULES -- urban_layouts() reads this set on every call."
		% ("differ from" if edited else "match")
	)

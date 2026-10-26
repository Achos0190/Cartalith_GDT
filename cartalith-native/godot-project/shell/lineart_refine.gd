extends RefCounted
## AP-10 (Ruling CR, 2026-10-18): the Asset library inspector's line-art
## block, under REFINE, for sprite families.
##
## **What it does.** Converts the previewed variant's drawing into a clean,
## solid, one-ink sprite and adds it as a NEW variant (Ruling CQ Q7: the
## original is never touched). All image work runs in Rust
## (`WorldGen.as_line_art` -> `cartalith_assets::lineart`); this file only
## lays out the choices and words the result.
##
## **The choices, each from a ruling:**
## * **Ink** -- one colour from the current Look (Ruling CQ Q4): the live
##   overlay treatment's `icon_ink`, `label_ink` and `border_ink`, read from
##   `MapOverlay.style_treatment()` each time the block refreshes, so a Look
##   change is followed. A look with no treatment (Default) has no ink
##   palette at all, so it offers three labelled fallbacks (`DEFAULT_INKS`).
## * **Line weight at map scale** -- Ruling CR Q-L1's per-variant control,
##   0.5-1.5 px, default 1.0, as five segments rather than a slider (a slider
##   in the inspector's scroller is a gesture hazard on phones: `MISTAKES.md`,
##   "Raise a touch target to the 44 dp floor").
## * **Gaps** -- Ruling CR's "scans-only 1 px gap closing": Auto (the engine
##   detects a scan), Off, or Close 1 px.
##
## **Built once.** Every control is created in `build()` and only re-labelled
## in `refresh()`, so the window's one-shot phone and tablet fitters reach all
## of them (`MISTAKES.md`, "Floor a control that a rebuild creates").
##
## **Must never** write the original variant, run image work in GDScript, or
## claim a measurement the engine did not return.
##
## Loaded with `preload` by `asset_library_window.gd` (no `class_name`, so no
## import pass and no `project.godot` change is needed).

## Ruling CR Q-L1's line weights, output pixels.
const WEIGHTS := [0.5, 0.75, 1.0, 1.25, 1.5]
## Ruling CR Q-L1's default.
const DEFAULT_WEIGHT := 1.0
## The gap-closing segments: label, `as_line_art`'s `closing` code, tooltip.
const GAPS := [
	["Auto", 0, "Close 1 px gaps only when the image reads as a scan (uneven light or paper grain)."],
	["Off", 1, "Never close gaps: clean digital art, where closing would bridge close parallel lines."],
	["Close 1 px", 2, "Always close 1 px gaps: a scan the detector missed."],
]
## The inks offered when the live overlay has no treatment (the Default look
## draws every label and icon in its own colour, so there is no Look ink to
## read). **Labelled judgements:** `sepia` is the research's ink (34, 28, 22,
## `lineart_lab.py`'s `INK`); `black` a near-black that is not pure black;
## `brown` the Antique look's `icon_ink` / `border_ink` (0.30, 0.19, 0.10).
const DEFAULT_INKS := [
	["sepia", Color8(34, 28, 22)],
	["black", Color8(20, 20, 20)],
	["brown", Color(0.30, 0.19, 0.10)],
]
## The treatment keys read for the three swatches, in order.
const INK_KEYS := [["icons", "icon_ink"], ["labels", "label_ink"], ["borders", "border_ink"]]

## The owning `AssetLibraryWindow` (untyped: the window preloads this file).
var _win = null
## The block's root, shown only for sprite families.
var box: VBoxContainer
var _readout: Label
var _ink_swatches: Array = []
var _ink_buttons: Array = []
var _inks: Array = []
var _ink_index := 0
var _weight := DEFAULT_WEIGHT
var _weight_buttons: Dictionary = {}
var _closing := 0
var _gap_buttons: Dictionary = {}
var _convert_btn: Button
var _undo_btn: Button
## The last conversion, for Undo: `{uid, index, hash}`; Undo removes it only
## while that variant's content hash still matches.
var _undo: Dictionary = {}


## Builds every control into `parent` (the inspector's REFINE column).
func build(parent: Control, win) -> void:
	_win = win
	box = VBoxContainer.new()
	box.add_theme_constant_override("separation", 6)
	parent.add_child(box)
	_readout = DccTheme.label("", "text_dim", DccTheme.FS_MICRO)
	_readout.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_readout.custom_minimum_size.x = win.W_INSPECTOR - 52
	box.add_child(_readout)

	var ink_row := HFlowContainer.new()
	ink_row.add_theme_constant_override("h_separation", 6)
	ink_row.add_theme_constant_override("v_separation", 4)
	box.add_child(ink_row)
	var ink_head := DccTheme.label("Ink", "text_ghost", DccTheme.FS_MICRO)
	ink_head.tooltip_text = "The one colour the converted sprite is drawn in, from the current Look. No paper colour survives, so there is no halo."
	ink_head.mouse_filter = Control.MOUSE_FILTER_STOP
	ink_row.add_child(ink_head)
	for i in INK_KEYS.size():
		var pair := HBoxContainer.new()
		pair.add_theme_constant_override("separation", 3)
		ink_row.add_child(pair)
		## A bordered swatch: the inks are dark, and a bare ColorRect of one
		## vanished into the dark palette's panel (seen in the first desktop
		## capture, 2026-10-19). The border is the theme's dim text token so it
		## reads on both palettes.
		var sw := Panel.new()
		sw.custom_minimum_size = Vector2(12, 12)
		sw.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		var st := StyleBoxFlat.new()
		st.set_border_width_all(1)
		st.border_color = DccTheme.c("text_dim")
		sw.add_theme_stylebox_override("panel", st)
		pair.add_child(sw)
		var idx := i
		var b: Button = win._segment(pair, String(INK_KEYS[i][0]), func(): _pick_ink(idx))
		_ink_swatches.append(sw)
		_ink_buttons.append(b)

	var w_row := HFlowContainer.new()
	w_row.add_theme_constant_override("h_separation", 4)
	w_row.add_theme_constant_override("v_separation", 4)
	box.add_child(w_row)
	var w_head := DccTheme.label("Line weight", "text_ghost", DccTheme.FS_MICRO)
	w_head.tooltip_text = "The thinnest line at map scale, in screen pixels. 1 px keeps every line solid; 0.5 keeps light art light, at the cost of faint lines."
	w_head.mouse_filter = Control.MOUSE_FILTER_STOP
	w_row.add_child(w_head)
	for wv in WEIGHTS:
		var v := float(wv)
		var b: Button = win._segment(w_row, ("%.2f" % v).rstrip("0").rstrip(".") + " px", func(): _pick_weight(v))
		b.tooltip_text = "Lines at least %s px wide at map scale." % ("%.2f" % v).rstrip("0").rstrip(".")
		_weight_buttons[v] = b

	var g_row := HFlowContainer.new()
	g_row.add_theme_constant_override("h_separation", 4)
	g_row.add_theme_constant_override("v_separation", 4)
	box.add_child(g_row)
	var g_head := DccTheme.label("Gaps", "text_ghost", DccTheme.FS_MICRO)
	g_head.tooltip_text = "Whether 1 px breaks in a line are closed before converting. Only scans need it."
	g_head.mouse_filter = Control.MOUSE_FILTER_STOP
	g_row.add_child(g_head)
	for g in GAPS:
		var code := int(g[1])
		var b: Button = win._segment(g_row, String(g[0]), func(): _pick_gap(code))
		b.tooltip_text = String(g[2])
		_gap_buttons[code] = b

	var act := HBoxContainer.new()
	act.add_theme_constant_override("separation", 5)
	box.add_child(act)
	_convert_btn = win._chip(act, "Convert line art", func(): _on_convert())
	_convert_btn.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	_undo_btn = win._chip(act, "Undo", func(): _on_undo())
	_undo_btn.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	_undo_btn.tooltip_text = "Remove the line-art variant just made. The original was never changed."
	_undo_btn.visible = false
	_light_segments()


## The current Look's three inks as `[[name, Color], ...]`: the live overlay
## treatment's `INK_KEYS` when it has all three, else `DEFAULT_INKS` (never a
## mix, so a name always says where its colour came from).
func look_inks() -> Array:
	var t: Dictionary = {}
	var host = _win._host if _win != null else null
	if host != null and host.get("viewport") != null and host.viewport.get("overlay") != null \
			and host.viewport.overlay.has_method("style_treatment"):
		t = host.viewport.overlay.style_treatment()
	var out: Array = []
	for i in INK_KEYS.size():
		var key := String(INK_KEYS[i][1])
		if not (t.get(key) is Color):
			return DEFAULT_INKS.duplicate(true)
		var c: Color = t[key]
		out.append(["Look " + String(INK_KEYS[i][0]), Color(c.r, c.g, c.b, 1.0)])
	return out


## Re-reads the Look's inks and words the block for the focused variant.
## `fam` is the window's family dictionary; the block shows only for sprites.
func refresh(uid: String, index: int, fam: Dictionary, item_count: int) -> void:
	var sprite := not bool(fam.get("texture", false)) and uid != ""
	box.visible = sprite
	if not _undo.is_empty() and (String(_undo.get("uid", "")) != uid or int(_undo.get("index", -1)) >= item_count):
		_undo = {}
	_undo_btn.visible = not _undo.is_empty()
	if not sprite:
		return
	_inks = look_inks()
	for i in _inks.size():
		swatch_set(_ink_swatches[i], _inks[i][1])
		_ink_buttons[i].text = String(_inks[i][0])
		_ink_buttons[i].tooltip_text = "Draw in %s, #%s." % [String(_inks[i][0]), Color(_inks[i][1]).to_html(false)]
	_light_segments()
	if item_count == 0 or index < 0 or index >= item_count:
		_readout.text = "Line art — no art in this slot yet."
		_convert_btn.disabled = true
		_convert_btn.tooltip_text = "Nothing to convert: this slot has no image."
		return
	if _undo.is_empty():
		_readout.text = "Line art — clean the paper away and redraw the lines solid, in one ink, as a new variant."
	_convert_btn.disabled = false
	_convert_btn.tooltip_text = "Convert variant %d: remove the paper (scans too), keep every line at least %s px at map scale, and draw it in the chosen ink. The original stays as it is." % [index + 1, ("%.2f" % _weight).rstrip("0").rstrip(".")]


## Paints one swatch (`build()`'s bordered Panel) in `c`.
static func swatch_set(sw: Panel, c: Color) -> void:
	(sw.get_theme_stylebox("panel") as StyleBoxFlat).bg_color = c


## The colour a swatch is showing.
static func swatch_colour(sw: Panel) -> Color:
	return (sw.get_theme_stylebox("panel") as StyleBoxFlat).bg_color


## Lights the segment matching each current choice.
func _light_segments() -> void:
	for i in _ink_buttons.size():
		_win._set_segment_on(_ink_buttons[i], i == _ink_index)
	for k in _weight_buttons:
		_win._set_segment_on(_weight_buttons[k], is_equal_approx(float(k), _weight))
	for k in _gap_buttons:
		_win._set_segment_on(_gap_buttons[k], int(k) == _closing)


func _pick_ink(i: int) -> void:
	_ink_index = i
	_light_segments()


func _pick_weight(v: float) -> void:
	_weight = v
	_light_segments()
	_win._refresh_inspector()


func _pick_gap(code: int) -> void:
	_closing = code
	_light_segments()


## Runs the conversion on the previewed variant and previews the result.
func _on_convert() -> void:
	var uid: String = _win._focused_uid
	if uid == "":
		return
	if _inks.is_empty():
		_inks = look_inks()
	var ink: Color = _inks[clampi(_ink_index, 0, _inks.size() - 1)][1]
	var r: Dictionary = _win._bridge.as_line_art(uid, _win._preview_index, ink, _weight, _closing)
	if not bool(r.get("ok", false)):
		_win._host.set_status("hint", "line art failed — %s" % String(r.get("error", "unknown error")), "warn")
		_readout.text = "Line art — not converted: %s." % String(r.get("error", "unknown error"))
		return
	_win._dirty = true
	var idx := int(r["index"])
	var made: Dictionary = _win._bridge.as_item_summary(uid, idx)
	_undo = {"uid": uid, "index": idx, "hash": String(made.get("hash", ""))}
	_readout.text = words(r)
	_win._preview_index = idx
	_win._host.set_status("hint", "line-art variant %d added — the original is unchanged" % (idx + 1), "accent")
	_win._refresh_grid()
	_win._refresh_inspector()
	_win._refresh_rail_counts()
	_win._refresh_status_line()


## The readout for one `as_line_art` result, in words and the engine's own
## numbers (nothing here is computed in GDScript).
static func words(r: Dictionary) -> String:
	var parts: Array = []
	parts.append("%d ink part%s" % [int(r["components"]), "" if int(r["components"]) == 1 else "s"])
	if bool(r["closed"]):
		parts.append("read as a scan: 1 px gaps closed" if bool(r["scan"]) else "1 px gaps closed (forced)")
	elif bool(r["scan"]):
		parts.append("read as a scan, gaps left open (Off)")
	if int(r["specks"]) > 0:
		parts.append("%d speck%s removed" % [int(r["specks"]), "" if int(r["specks"]) == 1 else "s"])
	if int(r["pinholes"]) > 0:
		parts.append("%d pinhole%s filled" % [int(r["pinholes"]), "" if int(r["pinholes"]) == 1 else "s"])
	if int(r["shading"]) > 0:
		parts.append("%d soft shadow%s ignored" % [int(r["shading"]), "" if int(r["shading"]) == 1 else "s"])
	var floor_px := float(r["floor_px"])
	var lines := ("thinnest line %.2f px at 256, widened by %.2f px a side" % [float(r["stroke_px"]), floor_px]) if floor_px > 0.0 \
		else ("thinnest line %.2f px at 256, no widening needed" % float(r["stroke_px"]))
	parts.append(lines)
	return "Line art — " + ", ".join(parts) + "."


## AP-10's Undo: removes the variant the last conversion added, only while
## it is still the same image.
func _on_undo() -> void:
	if _undo.is_empty():
		return
	var uid := String(_undo["uid"])
	var idx := int(_undo["index"])
	var want := String(_undo.get("hash", ""))
	_undo = {}
	var now: Dictionary = _win._bridge.as_item_summary(uid, idx)
	if want == "" or String(now.get("hash", "")) != want:
		_win._host.set_status("hint", "undo skipped — that variant has moved or changed since", "warn")
		_win._refresh_inspector()
		return
	if _win._bridge.as_remove_item(uid, idx):
		_win._dirty = true
		_win._preview_index = maxi(0, idx - 1)
		_win._host.set_status("hint", "line-art variant removed", "accent")
	else:
		_win._host.set_status("hint", "undo failed — that variant is gone", "warn")
	_win._refresh_grid()
	_win._refresh_inspector()
	_win._refresh_rail_counts()
	_win._refresh_status_line()

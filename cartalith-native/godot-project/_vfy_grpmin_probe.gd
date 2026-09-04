extends SceneTree
## VERIFIER. Widget-level, no shell: what `DccWidgets.group()`'s header
## minimum actually becomes, and whether the guard is really answering
## "does a sibling compete for my width" rather than "is my parent a BoxContainer".
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##       --script _vfy_grpmin_probe.gd

const TITLE := "who the bands are measured against"

var _fail := 0

func _chk(cond: bool, what: String) -> void:
	print(("  PASS  " if cond else "  FAIL  ") + what)
	if not cond:
		_fail += 1


func _hdr(body: VBoxContainer) -> Button:
	var p := body.get_parent().get_parent()
	for c in p.get_children():
		if c is Button:
			return c as Button
	return null


func _in(parent: Control) -> Button:
	var host := Control.new()
	host.size = Vector2(1000, 800)
	root.add_child(host)
	host.add_child(parent)
	var body := DccWidgets.group(parent, TITLE)
	return _hdr(body)


func _init() -> void:
	print("")
	print("GUARD -- one real child of each container kind")
	var vb := VBoxContainer.new()
	var hb := HBoxContainer.new()
	var mc := MarginContainer.new()
	var pc := PanelContainer.new()
	var hf := HFlowContainer.new()
	var g1 := GridContainer.new(); g1.columns = 1
	var g3 := GridContainer.new(); g3.columns = 3
	var bv := BoxContainer.new(); bv.vertical = true
	var bh := BoxContainer.new(); bh.vertical = false

	var cases := [
		["VBoxContainer", vb, true], ["MarginContainer", mc, true],
		["PanelContainer", pc, true], ["BoxContainer(vertical)", bv, true],
		["HBoxContainer", hb, false], ["HFlowContainer", hf, false],
		["GridContainer(columns=3)", g3, false],
		["GridContainer(columns=1)", g1, false],
		["BoxContainer(horizontal)", bh, false],
	]
	for c in cases:
		var b: Button = _in(c[1])
		var wraps: bool = b.autowrap_mode == TextServer.AUTOWRAP_WORD_SMART
		_chk(wraps == bool(c[2]), "%s -> autowrap %s (expected %s)"
			% [c[0], "ON" if wraps else "OFF", "ON" if bool(c[2]) else "OFF"])

	print("")
	print("MINIMUM -- what autowrap lowers the header's minimum TO")
	var col := VBoxContainer.new()
	var b2: Button = _in(col)
	col.size = Vector2(330, 400)
	var min_on: float = b2.get_minimum_size().x
	b2.autowrap_mode = TextServer.AUTOWRAP_OFF
	var min_off: float = b2.get_minimum_size().x
	b2.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	# The widest single word, for comparison -- the claim under test is that
	# autowrap does NOT stop there.
	var probe := Label.new()
	probe.add_theme_font_override("font", DccTheme.mono(2, true))
	probe.add_theme_font_size_override("font_size", DccTheme.FS_HEADER)
	root.add_child(probe)
	var widest := 0.0
	for w in String(b2.text).split(" "):
		probe.text = w
		widest = maxf(widest, probe.get_minimum_size().x)
	print("    autowrap OFF min.x = %.1f   ON min.x = %.1f   widest word = %.1f"
		% [min_off, min_on, widest])
	_chk(min_off > 200.0, "the un-wrapped header minimum is the whole label (>200 px)")
	_chk(min_on < 2.0, "autowrap drops the minimum to ~0, NOT to the widest word")
	_chk(not b2.clip_text, "clip_text stays false (DS-03 reflow, not ellipsis)")
	_chk(String(b2.text).to_lower().ends_with(TITLE), "the label text is intact")

	print("")
	print("VERIFIER RESULT: %d FAIL" % _fail)
	quit(1 if _fail > 0 else 0)

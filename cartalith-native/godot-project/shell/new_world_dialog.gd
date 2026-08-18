extends AcceptDialog
class_name NewWorldDialog

## File ▸ New world… (`DCC_SHELL_SPEC.md` §2.1): name, seed, extent
## (region/world) and working resolution.
##
## The non-square-map logic this replaces is real and hard-won -- cells are
## square in km and grid height is derived, `gh = gw` was the single line
## that made every map square -- so it is ported from `main.gd`'s
## `_build_world_setup` rather than rewritten.

var bridge: EngineBridge

func setup(b: EngineBridge) -> void:
	bridge = b
	title = "New world"
	size = Vector2i(620, 560)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 6)
	add_child(body)
	_build(body)

func _build(body: VBoxContainer) -> void:
	var l := DccTheme.label("Being ported from main.gd's world-setup dialog: extent, size preset, aspect ratio, exact grid width and height, and the derived readout that reports cells, km across and km per cell.", "text_dim", DccTheme.FS_SMALL)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.custom_minimum_size.x = 520
	body.add_child(l)

func open() -> void:
	popup_centered()

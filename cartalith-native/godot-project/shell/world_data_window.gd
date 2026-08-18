extends AcceptDialog
class_name WorldDataWindow

## Data ▸ World data tables (`DCC_SHELL_SPEC.md` §2.4/§9): the settlement,
## province and economy tables, sortable and filterable.
##
## §9's Data manager is a five-route window this is one route of; the rest
## (import, export, sources, conversion, validation) has no engine behind it
## yet and is not faked here.

var bridge: EngineBridge

func setup(b: EngineBridge) -> void:
	bridge = b
	title = "World data"
	size = Vector2i(940, 660)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 6)
	add_child(body)
	_build(body)

func _build(body: VBoxContainer) -> void:
	var l := DccTheme.label("Being ported from main.gd's world-data dialog: three tabs (settlements, provinces, economy), column sorting, and text filtering.", "text_dim", DccTheme.FS_SMALL)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.custom_minimum_size.x = 520
	body.add_child(l)

func open() -> void:
	popup_centered()

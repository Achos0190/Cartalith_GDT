extends AcceptDialog
class_name PerformanceWindow

## Preferences ▸ Performance readout (`DCC_SHELL_SPEC.md` §2.5).
##
## Reports which stages actually ran on the GPU (`get_gpu_stages_used`)
## rather than which ones could. The device checklist and multi-GPU mode
## §2.5 specifies need engine support that does not exist.

var bridge: EngineBridge

func setup(b: EngineBridge) -> void:
	bridge = b
	title = "Performance"
	size = Vector2i(620, 780)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 6)
	add_child(body)
	_build(body)

func _build(body: VBoxContainer) -> void:
	var l := DccTheme.label("Being ported from main.gd's performance dialog: per-stage GPU eligibility, which stages the last generate actually dispatched, and run timings.", "text_dim", DccTheme.FS_SMALL)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.custom_minimum_size.x = 520
	body.add_child(l)

func open() -> void:
	popup_centered()

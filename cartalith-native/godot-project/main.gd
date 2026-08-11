extends Control
## Phase 0 walking skeleton UI (ROADMAP.md): a triangle, a button, a printed
## line. Confirms the gdext-backed WalkingSkeleton node loaded and can be
## called into from GDScript — nothing here computes anything.

@onready var result_label: Label = $VBox/ResultLabel
@onready var skeleton: Node = $Skeleton


func _ready() -> void:
	$VBox/PingButton.pressed.connect(_on_ping_pressed)


func _on_ping_pressed() -> void:
	result_label.text = skeleton.ping()

extends Workspace
class_name WorldWorkspace

## WORLD domain (`DCC_SHELL_SPEC.md` §5): a two-button switch between the
## ten-stage Generation Pipeline (§5.1) and the Sculpt panel (§5.2).
##
## Built out by the pipeline agent. The Sculpt half cannot be wired at all
## until `cartalith-godot` binds `SculptStamp` -- see `STRANDED_TOOLS.md`.

func _build() -> void:
	_not_built("Generation pipeline",
		"Ten stages, ordered by dependency (spec §5.1). Being built.")

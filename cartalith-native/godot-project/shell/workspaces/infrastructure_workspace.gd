extends Workspace
class_name InfrastructureWorkspace

## INFRA domain (§3): roads, rivers, ports, trade, logistics.
##
## Roads and sea routes are read from the engine today. Drawing a new one
## has an engine (`ManualWay`, `RouteContext`) and no surface --
## `STRANDED_TOOLS.md` row 11.

func _build() -> void:
	_not_built("Infrastructure",
		"Roads and sea routes draw on the map; the route inspector is being built.")

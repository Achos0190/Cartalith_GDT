extends RefCounted
## The one name a hand-committed route carries on every surface (Ruling FO item 2, owner 2026-11-16;
## `PC_SHELL_PROPOSAL.md` finding M2: the context card said "Journey 1" while the planner said "Route #0").
##
## A route's engine index is 0-based; the person-facing number counts from 1, so index 0 is "Route 1".
## The fallback is computed here and never stored (a delete renumbers the later routes, and the label
## follows). A name the person typed (`route_set_name`) always wins. It must never be used for a
## SAVED JOURNEY (a plan with a party, which carries its own typed name) nor for a generated road, whose
## absent name stays absent ("never a made-up name", `civilization_workspace.gd::way_title`).
## Loaded by `preload` rather than `class_name`, so no project.godot import pass is needed.

## "Route N" for engine index `index` (0-based in, 1-based out).
static func default_name(index: int) -> String:
	return "Route %d" % (index + 1)

## The typed name when there is one, else `default_name(index)`. `route` is a `route_get()` dictionary,
## whose `name` key is always present and blank until `route_set_name` is called.
static func display(route: Dictionary, index: int) -> String:
	var n := String(route.get("name", ""))
	return n if not n.is_empty() else default_name(index)

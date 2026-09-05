extends VBoxContainer
class_name Workspace

## Base for the five domain workspaces (`DCC_SHELL_SPEC.md` §3).
##
## A workspace owns one domain's left-dock panel and, where §6 gives it one, a
## right-dock contribution. It reads world state through `bridge` and never
## holds a `WorldGen`. It draws using `DccWidgets` only, which is what keeps
## the five-level disclosure grammar honest -- there is nothing deeper to call.
##
## Subclasses override `_build`. `setup` is called once, after the panel is in
## the tree, so `_build` may safely add children.

var app: DccApp
var bridge: EngineBridge

## L2 categories are accordion siblings; this array ties them together so
## opening one closes the rest. One array per workspace, per §3's rule that L2
## state persists *per domain*.
var categories: Array = []

func setup(a: DccApp, b: EngineBridge) -> void:
	app = a
	bridge = b
	add_theme_constant_override("separation", 0)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_build()

func _build() -> void:
	pass

## Open the L2 category called `title`, closing its siblings the way a real
## click on its header would. Returns false when this workspace has no such
## category, which is the caller's cue that a cross-domain pointer has gone
## stale -- the whole point of routing these through one lookup.
##
## Every "→ Civilization ▸ Territories"-style button in the shell used to do
## half of this: switch domain and stop, leaving the user on a rail of
## collapsed categories with no indication which one was meant. Survivable when
## CIVIL had six; v3 gave it fourteen and CARTO ten, and an accordion opens one
## at a time, so the odds of landing on the right one by guessing went from
## poor to negligible.
func open_category(title: String) -> bool:
	for e in categories:
		if String(e["title"]) == title:
			if not (e["body"] as Control).visible:
				(e["button"] as Button).pressed.emit()
			return true
	return false

## Write the collapsed left dock's one line (`DCC_SHELL_SPEC.md` §1: *"A
## collapsed dock keeps its primary readout visible"*).
##
## **Called on every domain switch, by `DccApp._on_workspace_changed()`, for
## whichever workspace is now active.** Until 2026-09-05 the only writer of
## `set_dock_readout("left", …)` anywhere in the shell was
## `WorldWorkspace.push_dock_readout()`, so collapsing the dock in CIVIL or
## CARTO showed whatever WORLD had last written -- `resolved`, over a dock with
## no pipeline in it. `world_workspace.gd` overrides this with its stage state;
## the two domains that have no number of their own take the default below.
##
## The default is the domain's own rail word -- `WORLD` / `CIVIL` / `CARTO`,
## `DccShell.DOMAINS`' `rail` column -- which is the vocabulary
## `_refresh_viewport_context()` already names a domain with, and it is read
## from that table rather than restated so a rename cannot leave two spellings.
##
## **Stated rather than implied: the design does not settle this string.**
## `04-left-dock.md` §9.1 lists `ldCollapsedLabel` among the bindings lost to
## the prototype's truncation -- *"Unknown; may or may not vary by domain"* --
## so the rail word is a decision taken here, not a quotation. What the design
## does settle is that the strip is not blank and belongs to the dock beside it.
func push_dock_readout() -> void:
	if app == null:
		return
	var id := app.active_domain()
	for d in DccShell.DOMAINS:
		if String(d["id"]) == id:
			app.set_dock_readout("left", String(d["rail"]))
			return

## Draw the honest placeholder a workspace shows while its engine binding does
## not exist. `STRANDED_TOOLS.md` is the standing record of which those are;
## this is that record made visible in the product rather than only in a
## document, so nobody mistakes an empty panel for a finished one.
func _not_built(what: String, why: String) -> void:
	var body := DccWidgets.section(self, what)
	DccWidgets.note(body, why)

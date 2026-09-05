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

## Which domain's dock this panel is -- `"world"` / `"civilization"` /
## `"cartography"`. Written by `DccShell.register_workspace()`, which is the one
## call that knows the pairing; empty until then, and `apply_mode()` is a no-op
## while it is, so a workspace instantiated bare by a probe is unaffected.
##
## Deliberately not `app.active_domain()`: that answers *which dock is on
## screen*, and each of the three is off screen two thirds of the time.
var domain_id := ""

func bind_domain(id: String) -> void:
	domain_id = id

## -- `04-left-dock.md` §3's gate ----------------------------------------------
##
## Hide every category header this mode does not render, show the rest, and make
## sure one of the survivors is open.
##
## `DccShell.RAIL_NODES`' `shows` key is the whole table and its header block is
## the reasoning; the short version is that §3 gates exactly one node --
## `world/b`, the Sculpt block -- and every other node returns `[]`, which means
## *ungated*, which means this function shows all of them. **`[]` is not "hide
## everything":** nine nodes of ten carry no `shows` at all, and reading an
## absent allow-list as an empty one would blank nine docks.
##
## The header, its body and the rule under it are one `VBoxContainer` --
## `DccWidgets.category()` builds `wrap` and puts all three in it -- so hiding
## `body.get_parent()` takes the row and its divider together. Hiding the body
## alone would leave a header that cannot be opened and a hairline under it.
##
## Reached from `DccShell._select_domain()` and from nowhere else, which is the
## point: that is the single choke point `select_domain()`,
## `select_domain_mode()`, `select_domain_category()`, `Window ▸ Workspace`, the
## phone tabs and every in-shell jump button all pass through, so there is no
## transition into a gated mode that can arrive without the gate having run.
func apply_mode(mode: String) -> void:
	if domain_id.is_empty():
		return
	var shows: Array = DccShell.mode_shows(domain_id, mode)
	for e: Dictionary in categories:
		var wrap := _category_wrap(e)
		if wrap != null:
			wrap.visible = shows.is_empty() or shows.has(String(e.get("title", "")))
	_enforce_open_floor()

## The `VBoxContainer` holding one category's header, body and divider.
func _category_wrap(e: Dictionary) -> Control:
	var body: Control = e.get("body")
	if body == null or not is_instance_valid(body):
		return null
	return body.get_parent() as Control

## Whether this mode currently renders `title` at all. `open_category()` uses it
## to tell "no such category" (a stale pointer, and a warning) apart from "that
## category is behind a mode switch" (navigation, and a mode write).
func category_visible(title: String) -> bool:
	for e: Dictionary in categories:
		if String(e.get("title", "")) == title:
			var wrap := _category_wrap(e)
			return wrap != null and wrap.visible
	return false

## One visible category is always open. A gated dock can reach zero two ways --
## the mode's own body was closed when the user left it, or `_toggle_category()`
## re-collapsed the one open header -- and both leave a dock of headings with
## nothing under them, which reads as an empty domain rather than a closed
## accordion.
##
## The floor is `floor_category()` when that category is currently rendered, and
## *the first visible category* otherwise. Both halves are needed and neither is
## redundant: CIVIL names `Landmarks`, which is `04-left-dock.md` §6's own rule
## (*"Landmarks is the floor -- one category is always open, and it is never
## zero"*) and is **not** the first CIVIL category built (`Civilizations` is), so
## a pure build-order floor would open the wrong one; while WORLD `b` renders one
## category and names no floor, so a pure named floor would have nothing to open.
##
## The named floor is skipped when the mode hides it, which is why this cannot
## re-open a gated category behind the gate's back.
func _enforce_open_floor() -> void:
	var first: Dictionary = {}
	var named: Dictionary = {}
	var want := floor_category()
	for e: Dictionary in categories:
		var wrap := _category_wrap(e)
		if wrap == null or not wrap.visible:
			continue
		if first.is_empty():
			first = e
		if not want.is_empty() and String(e.get("title", "")) == want:
			named = e
		if (e["body"] as Control).visible:
			return
	var pick: Dictionary = named if not named.is_empty() else first
	if not pick.is_empty():
		DccWidgets._toggle_category(pick, categories)

## The category this dock falls back to when every header is closed, or `""` for
## *"whichever is rendered first"*. Overridden by CIVIL alone, per §6.
func floor_category() -> String:
	return ""

func setup(a: DccApp, b: EngineBridge) -> void:
	app = a
	bridge = b
	add_theme_constant_override("separation", 0)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_build()
	## **A gated dock needs the floor wired to its headers, not only to its mode
	## changes.** `apply_mode()` calls `_enforce_open_floor()` on every mode
	## switch, which covers arriving in a mode with everything closed; it does
	## not cover the user closing the one open header *while already there*.
	## `DccWidgets._toggle_category()` has no floor of its own — re-clicking the
	## open header always leaves the whole group closed — and in WORLD ▸ Sculpt
	## that group is one category, so the dock becomes a single heading with
	## nothing under it. CIVIL has had a floor since §6 asked for one and wires
	## it itself (`_lm_enforce_floor()`); this is the same wiring for the domains
	## the §3 gate reaches.
	##
	## Scoped to gating domains rather than applied to all three, because a
	## floor is a behaviour change to a dock that does not need one: CARTO has
	## ten headers and closing them all is a legible state there, not an empty
	## dock. `domain_gates()` is asked rather than `"world"` written down, so a
	## domain that later gains a gate gains the floor with it.
	if not domain_id.is_empty() and DccShell.domain_gates(domain_id):
		for e: Dictionary in categories:
			var cat_btn: Button = e.get("button")
			if cat_btn != null and is_instance_valid(cat_btn):
				cat_btn.pressed.connect(_enforce_open_floor)

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
## **A gated category is opened by first un-gating it** (`04-left-dock.md` §3;
## `DccShell.RAIL_NODES`' `shows`). `world/b` renders `Terrain` and hides the
## other eight, so a caller that jumps straight to `→ World ▸ Climate` while
## Sculpt is up would otherwise get `true` back, a body marked visible, and a
## wrap still hidden above it -- navigation that reports success and moves
## nothing, which is the exact failure this shell has already shipped once from
## the other end (a rail node that selected a mode and opened nothing).
##
## The mode is written through `DccShell.apply_domain_mode()`, which repaints the
## rail and re-runs the gate but does **not** call back into here -- so this
## cannot recurse, and `select_domain_mode()` (which does call here, after
## writing the mode itself) finds the wrap already visible and takes the plain
## path. Guarded on `app`, since `_railfold_probe.gd` and friends construct
## workspaces before `setup()`.
func open_category(title: String) -> bool:
	for e in categories:
		if String(e["title"]) == title:
			var wrap := _category_wrap(e)
			if wrap != null and not wrap.visible and app != null \
					and app.has_method("apply_domain_mode"):
				var m := DccShell.mode_for_category(domain_id, title)
				if not m.is_empty():
					app.call("apply_domain_mode", domain_id, m)
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

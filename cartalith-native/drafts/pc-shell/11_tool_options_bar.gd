## DRAFT, never loaded by Godot (it sits outside `godot-project/`). Ruling DZ (owner, 2026-11-01): the
## tool-options bar holds tool settings only.
##
## The owner, verbatim: "The buttons in the top rail new seed, generate and bake can be removed. Please see to
## only keep the paint/sculpt brush settings here." Asked what the bar should hold from then on, the owner chose
## "Tool settings only":
##   - the armed tool's settings, and nothing else: brush size, strength and mode for sculpt and paint, and
##     likewise each other tool's own settings (Measure, Way, Territory, Settlement, Icon and the rest);
##   - with no tool armed, only the context name (GENERATE · WORLD) and a one-line hint;
##   - the quick bar's pins at the bar's right end stay.
##
## What leaves the bar, and where each keeps a home (MISTAKES.md, "Delete a surface"):
##   Generate world       the WORLD dock's Generate world (`world_workspace.gd`), File ▸ Generate world and
##                        Ctrl+R (`pc-shell/02`), and on the narrow touch tablet World ▸ Generate world
##   New seed             the WORLD dock's RUN section (`world_workspace.gd`, `app._new_seed()`)
##   Center landmasses    the WORLD dock's RUN section (`world_workspace.gd`, `app._center_landmasses()`)
##   "generating…"        the busy overlay's stage line and the status bar's `pass` cell, which the same
##                        `generation_started` handler already writes
##   Bake ALL & finalize  the WORLD dock's Finalize foot (`_bake_button`, `_on_bake_all()`), the control the
##                        bar's copy only ever pressed
##
## Applies to: every composition that builds the tool-options bar (the PC and the tablet). The phone's tool
## strip is `MOBILE_SHELL_PROPOSAL.md`'s and follows the same rule there.
## Order: lands on its own; no other draft needs it, and it needs none. It does not touch `menus.gd`, so the
## menu parity check (`cartalith-native/docs/evidence/menu-parity-2026-11-01/`) is unaffected.
##
##   REPLACE  `app.gd::_tool_options_generate()`, with its doc comment (from "## §4's Generation Pipeline row"
##            to the line before "## Installs a minimal tool-options row"), by the two constants and the
##            function below.
##   EDIT T1  `app.gd`, the field block: delete these six lines.
##                ## The tool-options bar's staleness label; text is `generating…` while the engine is busy and empty afterwards.
##                var _tool_options_stale: Label
##                ## The tool-options bar's copy of the WORLD dock's bake button. Rebuilt every
##                ## time `_tool_options_generate()` runs, so always reached through
##                ## `is_instance_valid`.
##                var _tool_options_bake: Button
##   EDIT T2  `app.gd`, the `generation_started` handler in `_wire_status()`: delete the two lines (two and three tabs)
##                if is_instance_valid(_tool_options_stale):
##                    _tool_options_stale.text = "generating…")
##            and close the lambda on the line before them: `_refresh_status_mid()` becomes
##            `_refresh_status_mid())`.
##   EDIT T3  `app.gd`, the `generation_finished` handler in `_wire_status()`: delete the two lines (two and three tabs)
##                if is_instance_valid(_tool_options_stale):
##                    _tool_options_stale.text = ""
##   EDIT T4  `app.gd`: delete `set_bake_shortcut()` with its three doc lines (from "## Called by
##            `world_workspace._refresh_finalize()`" to the line before "## Rewrites the status bar's").
##   EDIT T5  `workspaces/world_workspace.gd::_refresh_finalize()`: delete these five lines (one or two tabs)
##                ## The tool-options bar's copy mirrors this one rather than recomputing it.
##                if app != null and app.has_method("set_bake_shortcut"):
##                    app.set_bake_shortcut(not finalized, not has_world,
##                        _bake_button.tooltip_text if has_world
##                        else "Generate a world before baking: the atlas is keyed to one.")
##            and, in its doc comment, replace "tile-count notes, and the" + newline + "## tool bar's bake
##            shortcut." with "and tile-count notes." (MISTAKES.md, "Change behaviour": the prose goes with
##            the code).
##   EDIT T6  `workspaces/world_workspace.gd`: delete `bake_and_finalize()` with its four doc lines (from "## The
##            tool-options bar's copy of this control presses exactly this" to the line before "## Re-syncs the
##            Finalize section from"). Its only caller was the bar's copy: `grep -rn bake_and_finalize shell/`
##            finds `app.gd` alone, inside the function this draft replaces.
##   CHECK    four comments cite `_tool_options_generate()`. Read each one after the paste:
##            - `dcc_widgets.gd`, the guard on `DccWidgets.action()`'s wrapping. It says that function "builds
##              five buttons straight into `row`". After this draft it builds none, but the tool rows still put
##              `✓ Commit` and `Discard` there (Way, Route, Conflict, Territory, Lasso), so the guard stays. Reword
##              the example to one of those rows.
##            - `menus.gd`, at `ID_LOD_REFINE_VIEW` and at the atlas-cache refine row. Both are history: Refine
##              detail sat on this bar from 2026-09-05 until Ruling U moved it back. They stay true.
##            - `_ds03fit_probe.gd`. Its band-height check was written against those five buttons. With WORLD's
##              idle row holding none, it must arm a tool whose row has `DccWidgets.action()` buttons before it
##              measures, or the check passes without exercising the case.
##   NOTE     `journey_planner_view.gd::_tool_options_journey()` puts the planner's journey picker, a carriage
##            readout and three actions on this bar: re-route…, save journey and export table. Under this ruling
##            and Ruling DY's PQ4 (the planner in its own window) they belong in the planner's window. They move
##            with PC-8, not in this draft (`90_pc_milestone_stubs.gd::planner_in_window()`).
##
## Probes whose expectation changes, to update in the same change:
##   `_structmove_probe.gd`     "2b control: Bake ALL & finalize is still on the WORLD bar". It becomes: the bar
##                              holds no such button, and the WORLD dock's Finalize foot holds one.
##   `_vfy_batch0905_probe.gd`  "A2 control: Bake ALL & finalize is still on the WORLD tool-options bar". The same.
## Unchanged, checked: `_worldcensus_probe.gd` counts the dock's New seed, Center landmasses and "Bake ALL levels &
## finalize", which stay; `_vfy_seedroll_probe.gd` calls `app._new_seed()`, which stays.
##
## Verify: `godot --headless --check-only` on `app.gd` and `world_workspace.gd`. Then a windowed probe that arms
## Inspect in WORLD and reads the bar: the context label, the hint and the quick bar's pins, and no Button of
## the bar's own. The WORLD dock's Generate world, New seed, Center landmasses and Bake ALL levels & finalize
## must each still act, and a generate must still write the status bar's `pass` cell.


## WORLD's idle hint in the tool-options bar, with a mouse and keyboard. A labelled judgement (Ruling DZ): with
## no actions left on the bar, its one line says what a click and the two context gestures do. Q is the
## cartwheel at the pointer and right-click the context card (`PC_SHELL_PROPOSAL.md` §3.9).
## Must never name an action that is not on the map itself; the bar holds settings, not commands.
const WORLD_IDLE_HINT_POINTER := "Click selects · right-click opens the card · Q opens the cartwheel"

## The same line on a touch tablet. A 500 ms hold opens the cartwheel together with the context card (CM-3,
## CM-4, kept as shipped by Ruling DY Q12).
const WORLD_IDLE_HINT_TOUCH := "Tap selects · hold opens the cartwheel and the card"


## WORLD's idle tool-options row (Ruling DZ, owner, 2026-11-01). The bar holds the armed tool's settings
## only. So with no tool armed, WORLD shows its context name and one hint, built the way CIVIL's and CARTO's
## idle rows already are (`_tool_options_simple()`). A sculpt or paint tool reclaims the bar with its brush
## settings when it arms, as every workspace's tools already do.
## Must never: build an action into this row. Generate world, New seed, Center landmasses and Bake ALL &
## finalize live in the WORLD dock (`world_workspace.gd`), and Generate world also in File (Ctrl+R). The owner
## removed this second copy. Keeping it in step had also needed a pushed-state shortcut (`set_bake_shortcut`,
## deleted by EDIT T4).
func _tool_options_generate() -> void:
	_tool_options_simple("GENERATE · WORLD",
		WORLD_IDLE_HINT_TOUCH if DccTheme.is_touch() else WORLD_IDLE_HINT_POINTER)

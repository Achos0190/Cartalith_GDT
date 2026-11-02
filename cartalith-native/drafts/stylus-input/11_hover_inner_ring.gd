## DRAFT, never loaded by Godot (it sits outside `godot-project/`). SH-6: the pen
## **hover footprint** — a second, inner brush ring at the minimum-pressure radius,
## so a hovering pen previews the span its pressure will sweep, not just the
## full-pressure outer edge. Standard pointer input; drawing only; no engine, no
## permission, no golden risk.
##
## WHY ADDITIVE. The brush ring already follows the cursor (and so already follows
## a hovering pen, since Godot delivers hover as plain motion): `tool_overlay.gd`'s
## `set_brush_cursor` / `_draw`, read 2026-11-02. This draft adds ONE optional
## field and ONE arc. Every existing caller passes nothing, gets `0`, and draws no
## inner ring — byte-for-byte today's cursor. Only a caller that opts in (the armed
## brush tool, passing its minimum-pressure radius) gains the footprint preview.
##
## HOW TO APPLY (three edits in `shell/tool_overlay.gd`):
##
##   ADD, in the fields block by `brush_radius_cells`:
##       ## SH-6: the inner (minimum-pressure) brush radius in grid cells. 0 draws
##       ## no inner ring, so a caller that does not set it keeps today's single
##       ## ring. Always < `brush_radius_cells` when set.
##       var brush_inner_cells := 0.0
##
##   EDIT C1 — `set_brush_cursor`'s signature and body, to carry it:
##       - func set_brush_cursor(visible_: bool, gx: float = 0.0, gy: float = 0.0, radius_cells: float = 0.0) -> void:
##       + func set_brush_cursor(visible_: bool, gx: float = 0.0, gy: float = 0.0, radius_cells: float = 0.0, inner_cells: float = 0.0) -> void:
##             brush_visible = visible_
##             brush_center = Vector2(gx, gy)
##             brush_radius_cells = radius_cells
##       +     brush_inner_cells = inner_cells
##             queue_redraw()
##     (the new arg is LAST and defaults to 0, so every current call compiles and
##      behaves as before — the "move, never a second surface" spirit: one ring
##      primitive, now with an optional inner edge, not a parallel one.)
##
##   EDIT C2 — in `_draw`, right AFTER the existing outer brush `draw_arc`
##     (the `draw_arc(center_screen, px_radius, 0, TAU, 48, MEASURE_COLOR, 1.2, true)`
##     line), add the inner ring, reusing the SAME `center_screen` and `cell_px`:
##       +     if brush_inner_cells > 0.0 and brush_inner_cells < brush_radius_cells:
##       +         ## SH-6: the minimum-pressure edge, dimmer and thinner than the
##       +         ## full-pressure ring, so the gap between the two reads as the
##       +         ## range a press will sweep. Same fit scaling as the outer ring.
##       +         draw_arc(center_screen, brush_inner_cells * cell_px, 0, TAU, 40,
##       +             Color(MEASURE_COLOR, 0.5), 1.0, true)
##
## WHO OPTS IN. The armed brush tool that already computes `radius_cells` for
## `set_brush_cursor` (the sculpt and paint hover paths in `world_workspace.gd`)
## passes `radius_cells * p_min` as `inner_cells`, where `p_min` is the pressure
## floor the brush maps to its smallest footprint (the engine's
## `paint_stroke_begin` `pressure_size` table, or the sculpt equivalent). That is a
## one-argument addition at each `set_brush_cursor(true, ...)` call for a brush
## tool; a non-brush tool (Measure, Way) passes nothing and is unchanged.
##
## OPTIONAL REFINEMENT, NOT IN THIS DRAFT: SH-6 also asks the ring to hide ~150 ms
## after the last hover. That is a timer on `_process`, and it MUST fire only while
## hovering (no button down): hiding the ring 150 ms after the last motion during
## an ACTIVE stroke would blink the cursor off mid-drag. Because that touches a
## shared primitive's visibility and cannot be verified without the editor, it is
## left to the implementer with that guard stated, rather than baked in here.
##
## MUST NEVER: draw the inner ring when it is not smaller than the outer (the
## guard), change any existing caller's result (the default 0), or scale the inner
## radius by anything but `cell_px` (both rings are real map distances; a fixed
## pixel inner ring would drift from the terrain on zoom, the bug the outer ring's
## own comment warns about).
##
## VERIFY: `godot --headless --check-only shell/tool_overlay.gd`. Then a windowed
## probe: a brush tool armed, `set_brush_cursor(true, gx, gy, r, r*0.3)` draws two
## concentric rings; `set_brush_cursor(true, gx, gy, r)` (no inner) draws exactly
## one, pixel-identical to today. On the device, hover the pen and see the pair.

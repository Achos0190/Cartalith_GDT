extends Workspace
class_name RenderWorkspace

## Terrain appearance groups; right dock preview & quality -- formerly the
## standalone RENDER rail domain (§3).
##
## `render.rs` carries a full `TerrainAppearance` structure that is settable
## in Rust and bound to nothing -- its own doc comment says so. Until it is
## bound, quality tier (Preferences) is the only live control.
##
## **Domain merge (2026-08-20, owner instruction: "And render into carto").**
## RENDER no longer has its own rail button; this one-subject class is now
## composed into `CartographyWorkspace` (`cartography_workspace.gd`'s own
## `_render` field) as a nested `VBoxContainer` appended after CARTO's own
## three categories. `_nested` (set true by `cartography_workspace.gd` before
## `setup()`) skips this file's own TOOLS block -- CARTO's own already covers
## the three global tools (plus Icon/Label), and RENDER never had a
## domain-specific tool of its own to add to it. This also directly resolves
## the CA-01/RN-01 ambiguity `GUI_GAP_REGISTER.md` §8.6 flagged: CARTO and
## RENDER were both proposing to own the same future `set_appearance()`
## binding; merging the domains removes the split.
var _nested := false

func _build() -> void:
	## §4.5.1's three global tools -- RENDER owns no tool of its own (no
	## §4.5.x section names one), so this was always the whole TOOLS block.
	## Skipped when nested -- see this file's own class doc.
	if not _nested:
		DccWidgets.tools_block(self, app, app.tool_group)
	_not_built("Terrain appearance",
		"render.rs's TerrainAppearance is real but unbound to Godot; until it is, Preferences ▸ Render quality is the live control.")
	_build_owed_inventory()

## What the one umbrella note above actually covers, enumerated rather than
## left as "terrain appearance" -- a reader of the dock could not otherwise
## learn that ~60 designed controls sit behind that single sentence.
## `design/Cartalith Menu Structure v2.dc.html` (MAP ▸ MAP STYLE and MAP ▸
## TERRAIN APPEARANCE) is the authoritative content list; `TERRAIN_APPEARANCE_
## SCOPE.md` carries the milestones. Prose, not disabled rows: sixty disabled
## sliders would be clutter implying sixty separate gaps, when there is one --
## the missing `set_appearance()`-shaped #[func].
func _build_owed_inventory() -> void:
	var sec := DccWidgets.section(self, "What that covers")
	DccWidgets.note(sec,
		"Preset (16 named ramps, save preset / save as theme / reset) · Colour relief "
		+ "(gradient editor with draggable stops, add/delete/duplicate/reverse, "
		+ "per-stop elevation + hex/RGBA, interpolation mode, elevation domain, "
		+ "Auto Fit / Auto Breakpoints) · Colour (vibrancy, saturation, contrast, "
		+ "brightness, gamma, temperature, tint, and the four field-influence "
		+ "weights) · Material (vegetation / rock / soil exposure and their slope, "
		+ "curvature and wetness modulation).")
	DccWidgets.note(sec,
		"Relief (multidirectional 8-light hillshade, strength / directionality / "
		+ "softness, the light rig, AO strength / radius / contrast, slope, "
		+ "curvature and local contrast) · Detail & atmosphere (macro / meso / "
		+ "micro intensity, distance and elevation haze) · Preview & quality "
		+ "(preview on-off, Compare current / previous / split, the four quality "
		+ "tiers -- of which only the tiers are live, in Preferences).")
	DccWidgets.note(sec,
		"And the reference's own Rendering-advanced and Painter-styles (NPR) "
		+ "blocks, which fold into this same subsystem rather than staying a "
		+ "separate menu: parchment, surface texture, sky view factor, ridge "
		+ "crests, ridged relief, slope rock, geology materials, cast shadows, "
		+ "curvature shading, minor channels, wetness, season, contour interval, "
		+ "SDF coastlines / river bands / biome blend; contour veins, ink "
		+ "linework, hachure, watercolor, cel/toon, engraving, stipple, sepia, "
		+ "risograph, pointillism, stylized icons, coastal wave lines, animate "
		+ "water, multi-sun lighting.")
	DccWidgets.note(sec,
		"None of it is presentation-unsafe: every control here updates visible "
		+ "tiles only and never marks a generation stage stale.")

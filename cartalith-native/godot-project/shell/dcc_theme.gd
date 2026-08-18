extends RefCounted
class_name DccTheme

## Colour tokens and StyleBox factories for the DCC shell.
##
## Every value here is read off the design mockup
## (`design/Cartalith DCC Shell.dc.html`), not invented. The mockup ships a dark
## and a light screen of the same layout, so the tokens come in pairs and the
## shell swaps `ACTIVE` between them rather than restyling anything.
##
## `DCC_SHELL_SPEC.md` §1 owns the geometry; this file owns only colour, type
## and the borders that separate regions.

# ── Palette ──────────────────────────────────────────────────────────────────

const DARK := {
	"bg": Color("#0d0e0f"),          ## Application ground, and the viewport letterbox.
	"panel": Color("#121314"),       ## Docks, menu bar, tool options bar.
	"panel_alt": Color("#111210"),   ## Rows that need to sit a shade back from `panel`.
	"raised": Color("#17191a"),      ## Menus, popovers, modals -- anything floating.
	"sunken": Color("#101112"),      ## Input wells and list bodies.
	"line": Color(1, 1, 1, 0.10),    ## The hairline every region is separated by.
	"line_soft": Color(1, 1, 1, 0.06),
	"text": Color("#c8cbcd"),        ## Body text.
	"text_bright": Color("#e8ebec"), ## Headers, active rows, the wordmark.
	"text_dim": Color("#8d9296"),    ## Secondary values.
	"text_faint": Color("#6f7478"),  ## Units, hints, the status bar's quiet half.
	"text_ghost": Color("#5f6468"),  ## Disabled.
	"accent": Color("#e0a34a"),
	"accent_dim": Color("#a4650f"),
	"accent_wash": Color("#e0a34a14"), ## 8% -- the active menu/tool background.
	"stale": Color("#b9a878"),       ## "downstream is stale" marks.
	"stale_wash": Color("#3d3226"),
}

const LIGHT := {
	"bg": Color("#fbfaf7"),
	"panel": Color("#f2f0ec"),
	"panel_alt": Color("#eeece7"),
	"raised": Color("#ffffff"),
	"sunken": Color("#e7e5e0"),
	"line": Color(0, 0, 0, 0.12),
	"line_soft": Color(0, 0, 0, 0.07),
	"text": Color("#23241f"),
	"text_bright": Color("#111210"),
	"text_dim": Color("#6b6f6a"),
	"text_faint": Color("#7c807a"),
	"text_ghost": Color("#9a9d95"),
	"accent": Color("#a4650f"),
	"accent_dim": Color("#7a6a4a"),
	"accent_wash": Color("#a4650f1a"),
	"stale": Color("#7a6a4a"),
	"stale_wash": Color("#e2d7bd"),
}

# ── Type ─────────────────────────────────────────────────────────────────────
#
# The mockup runs one sans stack for prose and IBM Plex Mono for anything
# numeric -- readouts, coordinates, the wordmark. Godot ships no Plex, so the
# mono role falls back to the editor's own mono font at runtime; the *roles*
# are what matter, and they are what the rest of the shell asks for.

const FS_MENU := 12
const FS_BODY := 12
const FS_SMALL := 11
const FS_TINY := 10
const FS_HEADER := 11   ## Section headers, letter-spaced and uppercased.
const FS_READOUT := 11  ## Mono.

# ── Geometry (§1) ────────────────────────────────────────────────────────────

const H_MENU_BAR := 34
const H_TOOL_OPTIONS := 34
const H_TIMELINE := 70
const H_STATUS := 26
const W_RAIL_COLLAPSED := 40
const W_RAIL_EXPANDED := 200
const W_LEFT_DOCK := 372
const W_LEFT_DOCK_MIN := 300
const W_LEFT_DOCK_MAX := 520
const W_RIGHT_DOCK := 300
const W_RIGHT_DOCK_MIN := 260
const W_RIGHT_DOCK_MAX := 460

## Touch scale (§13). The shell multiplies the heights above by this and
## enforces a 44 px floor on every hit box when the platform is not pointer-first.
const TOUCH_SCALE := 1.53  ## 34 -> 52, 26 -> 40, matching the tablet column.

# ── Active palette ───────────────────────────────────────────────────────────

static var _dark := true

static func set_dark(dark: bool) -> void:
	_dark = dark

static func is_dark() -> bool:
	return _dark

static func c(token: String) -> Color:
	var pal: Dictionary = DARK if _dark else LIGHT
	if not pal.has(token):
		push_error("DccTheme: unknown colour token '%s'" % token)
		return Color.MAGENTA
	return pal[token]

# ── StyleBox factories ───────────────────────────────────────────────────────
#
# Borders are asymmetric on purpose: a region draws only the edge that faces
# its neighbour, so two adjacent regions never stack two hairlines into one
# 2 px line.

static func panel(token: String = "panel", border: Dictionary = {}) -> StyleBoxFlat:
	var sb := StyleBoxFlat.new()
	sb.bg_color = c(token)
	sb.border_color = c("line")
	sb.border_width_left = border.get("left", 0)
	sb.border_width_right = border.get("right", 0)
	sb.border_width_top = border.get("top", 0)
	sb.border_width_bottom = border.get("bottom", 0)
	return sb

static func flat(color: Color, radius: int = 0) -> StyleBoxFlat:
	var sb := StyleBoxFlat.new()
	sb.bg_color = color
	sb.set_corner_radius_all(radius)
	return sb

static func empty() -> StyleBoxEmpty:
	return StyleBoxEmpty.new()

static func inset(l: int, t: int, r: int, b: int) -> StyleBoxEmpty:
	var sb := StyleBoxEmpty.new()
	sb.content_margin_left = l
	sb.content_margin_top = t
	sb.content_margin_right = r
	sb.content_margin_bottom = b
	return sb

## A row that lights up when active: transparent at rest, accent-washed with a
## 1 px accent underline when on. Used by the menu bar, the domain rail and
## every L2 category header.
static func active_row(bottom_rule: bool = true) -> StyleBoxFlat:
	var sb := StyleBoxFlat.new()
	sb.bg_color = c("accent_wash")
	if bottom_rule:
		sb.border_width_bottom = 1
		sb.border_color = c("accent")
	return sb

# ── Label helpers ────────────────────────────────────────────────────────────

## §7's section header: uppercase, letter-spaced, faint. Godot has no
## letter-spacing property on Label, so the spacing is baked into the string.
static func spaced(text: String) -> String:
	return " ".join(text.to_upper().split(""))

static func label(text: String, token: String = "text", size: int = FS_BODY) -> Label:
	var l := Label.new()
	l.text = text
	l.add_theme_color_override("font_color", c(token))
	l.add_theme_font_size_override("font_size", size)
	return l

static func header(text: String) -> Label:
	var l := label(text.to_upper(), "text_faint", FS_HEADER)
	l.add_theme_constant_override("line_spacing", 0)
	return l

static func rule(vertical: bool = false) -> Control:
	var r := ColorRect.new()
	r.color = c("line")
	if vertical:
		r.custom_minimum_size = Vector2(1, 0)
		r.size_flags_vertical = Control.SIZE_EXPAND_FILL
	else:
		r.custom_minimum_size = Vector2(0, 1)
		r.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	return r

static func spacer() -> Control:
	var s := Control.new()
	s.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	s.size_flags_vertical = Control.SIZE_EXPAND_FILL
	return s

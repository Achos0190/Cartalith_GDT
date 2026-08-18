extends RefCounted
class_name DccIcons

## The drawn glyph set (`DCC_SHELL_SPEC.md` §12).
##
## No emoji anywhere in the product. Every glyph is a bespoke inline SVG on a
## 16 x 16 viewBox, `fill:none`, `stroke-width:1.2`, round caps and joins, one
## weight only. The spec asks for `currentColor`; Godot's SVG rasteriser has no
## notion of an inherited colour, so every glyph is drawn in pure white and the
## host `modulate`s it. That is `currentColor` by another route -- one asset,
## inherits the accent when its row is active, inverts with the light theme.
##
## Rendered at 12 px in panels and 14-17 px on canvas buttons. Rasterising at
## the display size rather than scaling a larger bitmap is what keeps a 1.2 px
## hairline from turning into a grey smear, so `get()` takes the size and
## caches per (name, size).

const STROKE := 'fill="none" stroke="#ffffff" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"'

## The thirteen sculpt features, drawn as terrain cross-sections so they read as
## one family. These are the only glyphs that carry meaning rather than
## decoration, and their order matches `cartalith-terrain/src/sculpt.rs`.
const PATHS := {
	# ── Sculpt features (§12's table, in registry order) ──────────────────────
	"mountains": '<path d="M1 13.2 L6.2 4.4 L9.6 9.4"/><path d="M6.8 13.2 L10.6 6.2 L15 13.2"/>',
	"hills": '<path d="M1 13.2 C3.2 8.6 5.4 8.6 7.4 12 C9 14.6 11 9.4 12.6 8.2 C13.7 7.4 14.4 8.2 15 9.2"/><path d="M1 15 H15"/>',
	"ridge": '<path d="M1.4 13.4 L8 4 L14.6 13.4"/><path d="M8 4 V13"/>',
	"plateau": '<path d="M1.4 13.4 L4.4 6 H11.6 L14.6 13.4"/><path d="M4 6 H12"/>',
	"cliff": '<path d="M1 5.4 H7.2 V13.4 H15"/>',
	"canyon": '<path d="M1 4 L5.6 11.6 H10.4 L15 4"/><path d="M5.4 11.8 H10.6"/>',
	"valley": '<path d="M1.6 4 C3 11.4 5 13.4 8 13.4 C11 13.4 13 11.4 14.4 4"/>',
	"river": '<path d="M2 3.4 C5 5.4 4 8.2 6.4 10 C8.8 11.8 8 13 9.6 14.6"/><path d="M6.2 3 C8.6 5.2 7.6 7.4 9.6 9 C11.6 10.6 11.4 12.4 12.6 14"/>',
	"lake": '<path d="M2.6 8.8 C2.6 5.8 5.2 4 8 4 C10.8 4 13.4 5.8 13.4 8.8 C13.4 11 11 12.8 8 12.8 C5 12.8 2.6 11 2.6 8.8 Z"/><path d="M5 10.4 H8.4"/>',
	"basin": '<path d="M1.4 5.6 C3.4 11.6 12.6 11.6 14.6 5.6"/><path d="M4.4 6.8 C5.6 10 10.4 10 11.6 6.8"/>',
	"coastline": '<path d="M1 9.6 L3.4 6.6 L5.4 8.4 L7.8 5.2 L10 7.8 L12.4 5.6 L15 8.6"/><path d="M1 12.6 H15"/>',
	"volcano": '<path d="M1.4 13.4 L5.6 5.6 H10.4 L14.6 13.4"/><path d="M5.4 5.8 L6.8 7 L8 5.4 L9.2 7 L10.6 5.8"/>',
	"freehand": '<path d="M3 13 L3.6 10.4 L11 3 L13 5 L5.6 12.4 Z"/><path d="M10.6 3.4 L12.6 5.4"/>',

	# ── Everything else the spec names ────────────────────────────────────────
	"layers": '<path d="M8 2 L14.4 5.4 L8 8.8 L1.6 5.4 Z"/><path d="M1.6 8.2 L8 11.6 L14.4 8.2"/><path d="M1.6 11 L8 14.4 L14.4 11"/>',
	"dice": '<rect x="2.4" y="2.4" width="11.2" height="11.2" rx="2"/><circle cx="5.6" cy="5.6" r="0.7" fill="#ffffff" stroke="none"/><circle cx="10.4" cy="10.4" r="0.7" fill="#ffffff" stroke="none"/><circle cx="8" cy="8" r="0.7" fill="#ffffff" stroke="none"/>',
	"window": '<rect x="1.6" y="4" width="9.6" height="9.6" rx="1"/><path d="M5.2 4 V2.4 H14.4 V11.6 H12.8"/>',

	# ── Domain rail (§3). Not in §12's table; drawn to the same rules. ─────────
	"domain_world": '<circle cx="8" cy="8" r="6.2"/><path d="M1.8 8 H14.2"/><path d="M8 1.8 C10.4 4 10.4 12 8 14.2 C5.6 12 5.6 4 8 1.8 Z"/>',
	"domain_civ": '<path d="M2 13.6 V7.4 L5.4 5 L8.8 7.4 V13.6"/><path d="M8.8 13.6 V9 L11.8 7 L14 8.6 V13.6"/><path d="M1.2 13.6 H14.8"/>',
	"domain_infra": '<path d="M2 14 C5.6 10.4 4 6.4 8 2.4"/><path d="M14 14 C10.4 10.8 12 6.8 8.4 2.6"/><path d="M6 10 H10"/><path d="M5 13 H11"/>',
	"domain_carto": '<path d="M1.6 4 L5.8 2.4 L10.2 4.6 L14.4 3 V12 L10.2 13.6 L5.8 11.4 L1.6 13 Z"/><path d="M5.8 2.4 V11.4"/><path d="M10.2 4.6 V13.6"/>',
	"domain_render": '<circle cx="8" cy="8" r="3.2"/><path d="M8 1.4 V3.2"/><path d="M8 12.8 V14.6"/><path d="M1.4 8 H3.2"/><path d="M12.8 8 H14.6"/><path d="M3.4 3.4 L4.6 4.6"/><path d="M11.4 11.4 L12.6 12.6"/>',
}

static var _cache: Dictionary = {}  ## "name@size" -> ImageTexture

## Rasterise `name` at `px` and return a texture drawn in white. Tint it with
## `modulate` on whatever displays it -- never bake a colour in, or the light
## theme needs a second copy of every glyph.
static func get_icon(name: String, px: int = 12) -> Texture2D:
	var key := "%s@%d" % [name, px]
	if _cache.has(key):
		return _cache[key]
	if not PATHS.has(name):
		push_error("DccIcons: no glyph named '%s'" % name)
		return null
	var svg := '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16" %s>%s</svg>' % [STROKE, PATHS[name]]
	var img := Image.new()
	# The rasteriser's `scale` is relative to the declared 16 px box, so this
	# renders natively at the display size rather than resampling.
	var err := img.load_svg_from_string(svg, float(px) / 16.0)
	if err != OK:
		push_error("DccIcons: '%s' failed to rasterise (%d)" % [name, err])
		return null
	var tex := ImageTexture.create_from_image(img)
	_cache[key] = tex
	return tex

## A TextureRect sized to the glyph and tinted to a theme token, ready to drop
## into a row.
static func rect(name: String, px: int = 12, token: String = "text_dim") -> TextureRect:
	var t := TextureRect.new()
	t.texture = get_icon(name, px)
	t.custom_minimum_size = Vector2(px, px)
	t.stretch_mode = TextureRect.STRETCH_KEEP_CENTERED
	t.modulate = DccTheme.c(token)
	t.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return t

## Text symbols stay text (§12): they are typographic, inherit type metrics and
## need no drawing. Listed here so nobody is tempted to draw them.
const SYMBOLS := {
	"submenu": "\u25B8", "collapse": "\u2039", "expand": "\u203A", "caret": "\u25BE",
	"chevron": "\u2304", "on": "\u25CF", "off": "\u25CB", "checked": "\u2611",
	"unchecked": "\u2610", "tick": "\u2713", "cross": "\u2715", "add": "\uFF0B",
	"delete": "\u232B", "undo": "\u21B6", "redo": "\u21B7", "play": "\u25B6",
	"pause": "\u23F8", "drawer": "\u2630", "panels": "\u25A4", "overflow": "\u22EF",
	"locked": "\U0001F512",
}

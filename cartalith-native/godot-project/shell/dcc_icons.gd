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
##
## **`px` is the size the glyph is *drawn* at, not the size it is *rasterised*
## at, and on a phone those differ** (`GUI_GAP_REGISTER.md` HD-02). A window
## presented by `DccWidgets.phone_present()` sets `content_scale_factor` to the
## handset's own scale and lays its content out in 393 dp; a 12 px glyph in it
## is 12 *dp* and reaches the panel as 44 physical pixels on a 1440-wide
## device. Rasterising 12 texels and letting the canvas transform blow them up
## is exactly the "grey smear" the paragraph above exists to prevent -- it was
## simply invisible while every phone measured was 1080 wide, where the same
## fault is 2.75x rather than 3.66x.
##
## So `get_icon()` takes a second number: `magnify`, what this glyph will be
## multiplied by between the layout that sizes it and the pixels that show it.
## The texture is rasterised at `px * magnify` and **presents** at `px` (via
## `ImageTexture.set_size_override`), so nothing a caller lays out moves and no
## call site has to change. `magnify` defaults to 1 -- the unscaled main
## viewport, where a caller already sizes in real pixels and a larger raster
## would only be minified back down through a 1.2 px hairline. `rect()` below
## works the real number out for itself, from the canvas transform at draw
## time; `DccShell._phone_fit_tool_button()` passes its own, because a `Button`
## icon is not a node and has no transform to ask.

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

	## The touch navpad (`GUI_GAP_REGISTER.md` SH-14), the port's answer to the
	## reference's mobile `#zoomOverlay`. Drawn rather than left as the
	## reference's own `+` / `-` / `✋` / `⟳` text: the four sit in one column
	## and have to read as one family, which a type glyph beside a 1.2 px
	## stroke never does -- and `⟳` (U+27F3) is missing from Plex Mono and the
	## whole fallback chain anyway, the same tofu case `search`/`import` were
	## drawn for. The hand is `tool_pan` above, reused unchanged.
	##
	## `view_fill` is a *frame with the content pushed out to it* -- the cover
	## reset, not a fit: the two diagonals run outward to opposite corners.
	"zoom_in": '<circle cx="7" cy="7" r="4.4"/><path d="M10.2 10.2 L14.2 14.2"/><path d="M4.9 7 H9.1"/><path d="M7 4.9 V9.1"/>',
	"zoom_out": '<circle cx="7" cy="7" r="4.4"/><path d="M10.2 10.2 L14.2 14.2"/><path d="M4.9 7 H9.1"/>',
	"view_fill": '<rect x="1.8" y="3.2" width="12.4" height="9.6" rx="1"/><path d="M5.8 6.2 H4.2 V7.8"/><path d="M4.4 6.4 L6.9 8.9"/><path d="M10.2 9.8 H11.8 V8.2"/><path d="M11.6 9.6 L9.1 7.1"/>',

	## The two file-dialog screens' own marks (`design/Cartalith DCC Shell.
	## dc.html`, "Open project dialog 1920"): `⌕` on the search well and `⤓` on
	## the import/drop tile. Drawn rather than left as text -- unlike the
	## symbols in `SYMBOLS` below, U+2315 and U+2913 are missing from Plex Mono
	## *and* from the Segoe UI Symbol / DejaVu fallback chain `mono()` installs,
	## so as text they render as tofu on at least one target platform. §12's
	## "typographic symbols stay text" premise holds only for glyphs that
	## actually exist somewhere in the chain.
	"search": '<circle cx="7" cy="7" r="4.4"/><path d="M10.2 10.2 L14.2 14.2"/>',
	"import": '<path d="M8 2.4 V10.4"/><path d="M4.8 7.4 L8 10.6 L11.2 7.4"/><path d="M2.6 13.4 H13.4"/>',

	# ── Tool palette (§4.5), glyphs described in §12's second table ───────────
	# One tool is armed at a time, globally; these are what the TOOLS block at
	# the head of every left dock draws.
	"tool_inspect": '<path d="M4 2.4 L4 13.2 L6.8 10.6 L8.8 14.4 L10.6 13.4 L8.7 9.9 L12.4 9.4 Z"/>',
	"tool_measure": '<path d="M1.6 6 H14.4 V10 H1.6 Z"/><path d="M4.8 6 V8.2"/><path d="M8 6 V8.8"/><path d="M11.2 6 V8.2"/>',
	"tool_region": '<path d="M2 2.6 H5" /><path d="M7 2.6 H9"/><path d="M11 2.6 H14 V5"/><path d="M14 7 V9"/><path d="M14 11 V13.4 H11"/><path d="M9 13.4 H7"/><path d="M5 13.4 H2 V11"/><path d="M2 9 V7"/><path d="M2 5 V2.6"/>',
	"tool_pan": '<path d="M5.4 8.4 V4.2 A1 1 0 0 1 7.4 4.2 V7.6"/><path d="M7.4 7.4 V3.2 A1 1 0 0 1 9.4 3.2 V7.6"/><path d="M9.4 7.6 V4.4 A1 1 0 0 1 11.4 4.4 V9.6"/><path d="M5.4 8.4 V6.6 A1 1 0 0 0 3.4 6.6 V10 C3.4 12.6 5.4 14.4 8 14.4 C10.6 14.4 11.4 12.4 11.4 10"/>',
	"tool_paint": '<path d="M5.4 2.6 H10.6 V6.4 A2 2 0 0 1 8.6 8.4 H7.4 A2 2 0 0 1 5.4 6.4 Z"/><path d="M8 8.6 V11.2"/><path d="M8 11.4 C8 11.4 9.2 12.6 9.2 13.3 A1.2 1.2 0 0 1 6.8 13.3 C6.8 12.6 8 11.4 8 11.4 Z"/>',
	"tool_settlement": '<path d="M2.4 7.6 L8 2.8 L13.6 7.6"/><path d="M4 7 V13.4 H12 V7"/><path d="M6.8 13.4 V9.8 H9.2 V13.4"/>',
	"tool_poi": '<path d="M8 2.2 L13.8 8 L8 13.8 L2.2 8 Z"/><circle cx="8" cy="8" r="0.7" fill="#ffffff" stroke="none"/>',
	"tool_territory": '<path d="M2.2 5 L7.6 2.6 L13.8 5.4 L11 13.2 L4 12.4 Z"/><path d="M4 6.6 L6.4 11.8"/><path d="M6.8 5.4 L9.4 12.2"/><path d="M9.6 5.2 L11.4 10.4"/>',
	"tool_way": '<path d="M4 2.2 V13.8"/><path d="M12 2.2 V13.8"/><path d="M2.6 4.6 H13.4"/><path d="M2.6 8 H13.4"/><path d="M2.6 11.4 H13.4"/>',
	"tool_route": '<path d="M2 12.6 C2 8.6 4.6 6 8 6"/><path d="M8 6 C10.2 6 11.6 5 12.6 3.6"/><path d="M11 2.6 L13.6 3.4 L12.4 5.8"/>',
	## A distance-spine glyph -- a path with elevation ticks along it, matching
	## `JOURNEY_PLANNER_SPEC.md` §3's own "route map and terrain profile share
	## one distance axis" (`DCC_SHELL_SPEC.md` §4.5.4's 2026-08-19 addition).
	"tool_journey": '<path d="M1.6 12.4 C4.4 8.4 6 8.4 8 11 C10 13.6 11.6 6 14.4 3.6"/><path d="M4.4 12.4 V9.8"/><path d="M8.2 12.4 V10.2"/><path d="M11.8 12.4 V7.6"/>',
	"tool_label": '<path d="M2.2 6.4 L7.4 2.4 H13 A0.8 0.8 0 0 1 13.8 3.2 V9.2 A0.8 0.8 0 0 1 13 10 H7.4 Z"/><circle cx="10.6" cy="6.2" r="0.8"/>',
	"tool_icon": '<path d="M8 2.2 L13.8 8 L8 13.8 L2.2 8 Z"/>',

	# ── Domain rail (§3). Not in §12's table; drawn to the same rules. ─────────
	"domain_world": '<circle cx="8" cy="8" r="6.2"/><path d="M1.8 8 H14.2"/><path d="M8 1.8 C10.4 4 10.4 12 8 14.2 C5.6 12 5.6 4 8 1.8 Z"/>',
	"domain_civ": '<path d="M2 13.6 V7.4 L5.4 5 L8.8 7.4 V13.6"/><path d="M8.8 13.6 V9 L11.8 7 L14 8.6 V13.6"/><path d="M1.2 13.6 H14.8"/>',
	"domain_infra": '<path d="M2 14 C5.6 10.4 4 6.4 8 2.4"/><path d="M14 14 C10.4 10.8 12 6.8 8.4 2.6"/><path d="M6 10 H10"/><path d="M5 13 H11"/>',
	"domain_carto": '<path d="M1.6 4 L5.8 2.4 L10.2 4.6 L14.4 3 V12 L10.2 13.6 L5.8 11.4 L1.6 13 Z"/><path d="M5.8 2.4 V11.4"/><path d="M10.2 4.6 V13.6"/>',
	"domain_render": '<circle cx="8" cy="8" r="3.2"/><path d="M8 1.4 V3.2"/><path d="M8 12.8 V14.6"/><path d="M1.4 8 H3.2"/><path d="M12.8 8 H14.6"/><path d="M3.4 3.4 L4.6 4.6"/><path d="M11.4 11.4 L12.6 12.6"/>',
}

static var _cache: Dictionary = {}  ## "name@drawn@raster" -> ImageTexture
	## Keyed on the *rasterisation* size as well as the drawn one, because the
	## same 12 px glyph is a 12-texel bitmap in a dock and a 44-texel one in a
	## content-scaled window, and the two must not share a cache entry.

## What the glyph cache actually holds, for the Performance window's Memory
## group. HD-02's finer raster is the one hi-DPI cost that could plausibly have
## been large, so it is reported rather than argued about: measured 2026-08-25
## on the OnePlus 6T at `_phone_scale` 2.748, **389.4 KiB with a world up**,
## against 500.9 MiB of canvas vertex buffers in the same frame. See
## `MEMORY_OPTIMIZATION_SCOPE.md`'s hi-DPI section for the full bisection.
static func cache_stats() -> Dictionary:
	var bytes := 0
	for k in _cache.keys():
		var t := _cache[k] as ImageTexture
		if t != null:
			var im := t.get_image()
			if im != null:
				bytes += im.get_width() * im.get_height() * 4
	return {"entries": _cache.size(), "bytes": bytes}

## Rasterise `name` and return a texture drawn in white, presenting at `px`.
## Tint it with `modulate` on whatever displays it -- never bake a colour in, or
## the light theme needs a second copy of every glyph.
##
## `magnify` is what the drawing surface will scale this glyph by before it
## reaches a physical pixel: 1 in the main viewport (which has no content
## scale), the handset scale inside a `phone_present()`ed window. See the file
## header. It never changes the size the glyph *reports*, so it can be raised
## on an existing call site without moving any layout.
static func get_icon(name: String, px: int = 12, magnify: float = 1.0) -> Texture2D:
	var raster := maxi(1, int(round(float(px) * maxf(1.0, magnify))))
	var key := "%s@%d@%d" % [name, px, raster]
	if _cache.has(key):
		return _cache[key]
	if not PATHS.has(name):
		push_error("DccIcons: no glyph named '%s'" % name)
		return null
	var svg := '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" width="16" height="16" %s>%s</svg>' % [STROKE, PATHS[name]]
	var img := Image.new()
	# The rasteriser's `scale` is relative to the declared 16 px box, so this
	# renders natively at the display size rather than resampling.
	var err := img.load_svg_from_string(svg, float(raster) / 16.0)
	if err != OK:
		push_error("DccIcons: '%s' failed to rasterise (%d)" % [name, err])
		return null
	var tex := ImageTexture.create_from_image(img)
	## The half that keeps 66 call sites untouched: `get_width()`/`get_height()`
	## -- which is what a `TextureRect`, a `Button` icon and every other consumer
	## lays out against -- report `px`, while the pixels behind them are the
	## finer raster. Godot draws an `ImageTexture` into `get_size()`, so the
	## downscale-on-draw is exactly cancelled by the surface's own magnification.
	if raster != px:
		tex.set_size_override(Vector2i(px, px))
	_cache[key] = tex
	return tex

## The applied magnification, so `_refit_glyph()` below is a no-op on every
## draw after the one that settled it. Also the marker that says "this
## TextureRect is a `DccIcons` glyph", which nothing else needs today and the
## next phone pass will.
const ICON_META := "dcc_icon_magnify"

## A TextureRect sized to the glyph and tinted to a theme token, ready to drop
## into a row.
##
## **The magnification is read off the canvas transform at draw time, not
## passed in** (`GUI_GAP_REGISTER.md` HD-02). Three earlier shapes of this fix
## were tried and each was wrong for a real call site:
##
##   - a static "device scale" set once by the shell -- wrong in the main
##     viewport, which has no content scale, where it would rasterise 3.664x
##     too fine and then minify a 1.2 px hairline back down with no mipmap;
##   - re-rasterising from `DccShell.phone_fit()`, which knows the number
##     exactly -- but only reaches a subtree that exists when it runs, and the
##     open-project dialog builds its action tiles and its import tile on
##     `navigate()`, long after its one `phone_fit(self, 1.0)` call. Measured:
##     the search glyph was fixed and the other three were not;
##   - re-rasterising on `tree_entered`, which fires before `phone_present()`
##     has set `content_scale_factor` for every glyph built during `setup()`.
##
## `get_screen_transform()` is the call, and **not**
## `get_global_transform_with_canvas()`, which is the one that reads like the
## right answer: a `CanvasLayer` transform is not a viewport's *final*
## transform, and a content scale lives in the latter. Measured side by side on
## all four glyphs of the welcome screen at 1440x3168 -- `gtwc` scale (1.0,
## 1.0), `screen` scale (3.664122, 3.664122). With the wrong one the fix is
## silently inert, which is exactly how it first measured.
##
## It composes the viewport's final transform with the node's own, so it is the
## true answer for any surface -- dock, content-scaled window, embedded popup --
## with no host reference, no call-site change and nothing to keep in sync.
## Assigning `texture` from inside `draw` queues one more redraw; the meta guard
## makes the next one a comparison and stops there.
static func rect(name: String, px: int = 12, token: String = "text_dim") -> TextureRect:
	var t := TextureRect.new()
	t.texture = get_icon(name, px)
	t.custom_minimum_size = Vector2(px, px)
	t.stretch_mode = TextureRect.STRETCH_KEEP_CENTERED
	t.modulate = DccTheme.c(token)
	t.mouse_filter = Control.MOUSE_FILTER_IGNORE
	t.set_meta(ICON_META, 1.0)
	t.draw.connect(_refit_glyph.bind(t, name, px))
	return t

## Quantised to 1/16, so a transform that jitters in the last decimal (a
## float32 `content_scale_factor` is 3.66412210464478, not 3.66412213740458)
## cannot re-rasterise and re-cache a glyph on every frame.
static func _refit_glyph(t: TextureRect, name: String, px: int) -> void:
	var mag: float = maxf(1.0, t.get_screen_transform().get_scale().x)
	mag = round(mag * 16.0) / 16.0
	if is_equal_approx(float(t.get_meta(ICON_META, 1.0)), mag):
		return
	t.set_meta(ICON_META, mag)
	var tex := get_icon(name, px, mag)
	if tex != null:
		t.texture = tex

## Text symbols stay text (§12): they are typographic, inherit type metrics and
## need no drawing. Listed here so nobody is tempted to draw them.
const SYMBOLS := {
	"submenu": "\u25B8", "collapse": "\u2039", "expand": "\u203A", "caret": "\u25BE",
	"chevron": "\u2304", "on": "\u25CF", "off": "\u25CB", "checked": "\u2611",
	"unchecked": "\u2610", "tick": "\u2713", "cross": "\u2715", "add": "+",
	"delete": "\u232B", "undo": "\u21B6", "redo": "\u21B7", "play": "\u25B6",
	"pause": "\u23F8", "drawer": "\u2630", "panels": "\u25A4", "overflow": "\u22EF",
	"locked": "\U0001F512",
	## The journey planner mockup's own three severity glyphs (`design/Journey
	## Planner DCC.dc.html`), used verbatim -- dingbat-class Unicode symbols
	## like the rest of this table, not colour emoji.
	"blocked": "\u26D4", "warn_tri": "\u26A0", "bolt": "\u26A1",
}

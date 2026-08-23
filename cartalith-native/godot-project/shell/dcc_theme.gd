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
	## Read off `design/Journey Planner DCC.dc.html`'s own inline styles
	## (`JOURNEY_PLANNER_SPEC.md` §3: "warn #e0a840, block #b55950, water
	## #7d9dae"), added here rather than left as hard-coded hex in
	## `journey_planner_view.gd` because §6's disclosure grammar and this
	## file's own header both say colour is a shell-wide token, not a
	## per-feature constant. Not yet used anywhere else in the shell -- the
	## journey planner is the first feature to need "strained"/"blocked"/
	## "water leg" as distinct from the existing `accent`/`stale` pair.
	"warn": Color("#e0a840"),
	"block": Color("#b55950"),
	"water": Color("#7d9dae"),
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
	## The dark-mode journey planner hex values, unchanged. `JOURNEY_PLANNER_
	## SPEC.md` §10 lists light theme as still to build for this feature --
	## these three tokens exist so `c()` never errors under light mode, not
	## because they were tuned against a light mockup that doesn't exist yet.
	"warn": Color("#e0a840"),
	"block": Color("#b55950"),
	"water": Color("#7d9dae"),
}

# ── Type ─────────────────────────────────────────────────────────────────────
#
# The mockup runs two faces and leans hard on the second: a plain UI sans for
# prose, and **IBM Plex Mono for every numeric readout, code, shortcut and
# section label**, at 9-11 px with 0.12-0.22 em letter-spacing. That mono-with-
# tracking texture is most of what makes the reference look like a DCC tool
# rather than a form, so it is not optional styling -- it is the design.
#
# Plex is bundled (`fonts/`, SIL OFL 1.1) rather than taken from the system:
# Android has no Plex, and a silent fallback to Roboto would quietly undo the
# thing this block exists to achieve. The prose face is Fira Sans, same
# reasoning, same bundling -- `fonts/FiraSans-{Regular,Medium,Bold,Italic}.ttf`
# (SIL OFL 1.1, `fonts/FiraSans-OFL.txt`), wired as `dark_theme.tres`'s
# `default_font` rather than duplicated as a helper here, since (unlike Plex)
# every Control gets it for free with no per-Label override needed. This
# closes the "Fira Sans/Fira Code, sourcing deferred" note CHANGELOG.md has
# carried since the original design-system match -- Fira Code itself is
# sourced too (`fonts/FiraCode-{Regular,Medium}.ttf`) but stays unwired since
# Plex Mono already fills that exact role, shipped and tested; see
# `dark_theme.tres`'s own header for the full reasoning.

const FONT_MONO := preload("res://fonts/IBMPlexMono-Regular.ttf")
const FONT_MONO_MED := preload("res://fonts/IBMPlexMono-Medium.ttf")

const FS_MENU := 12
const FS_BODY := 12
const FS_SMALL := 11
const FS_TINY := 10
const FS_MICRO := 9    ## Section labels and the smallest readouts.
const FS_HEADER := 9   ## §-prefixed section headers, tracked wide.
const FS_READOUT := 11 ## Mono numerics.
const FS_HERO := 26    ## The one big accent readout per context (§6's elevation).
## The one size the shell's *modal* screens set their own title in -- both the
## "Open project dialog 1920" and "Select folder dialog 1920" cards in
## `design/Cartalith DCC Shell.dc.html` open with `font:500 16px` prose, a
## step above anything a dock ever draws. A modal title is the only place in
## the design where 16 px appears, which is why it is its own token rather
## than an off-by-one reuse of `FS_HERO`.
const FS_MODAL_TITLE := 16

static var _tracked: Dictionary = {}  ## spacing px -> FontVariation

## Godot has no letter-spacing property on Label, but `FontVariation` carries
## `spacing_glyph` -- extra pixels after every glyph, which is exactly tracking.
## Cached per spacing because a FontVariation is a Resource, not a value.
static func mono(spacing: int = 0, medium: bool = false) -> Font:
	var key := "%d/%s" % [spacing, medium]
	if _tracked.has(key):
		return _tracked[key]
	var fv := FontVariation.new()
	fv.base_font = FONT_MONO_MED if medium else FONT_MONO
	if spacing != 0:
		fv.spacing_glyph = spacing
	## §12 asserts the text symbols "are typographic, inherit type metrics, and
	## need no drawing". That premise does not hold for Plex Mono, which is
	## missing seven of them -- checked against the font's own cmap, not
	## assumed: ✕ (2715), ● (25CF), ○ (25CB), ▾ (25BE), ▸ (25B8), ▶ (25B6) and
	## ＋ (FF0B). ✓, →, §, ‹, ›, ↶, ↷, ·, • and × are all present.
	##
	## A fallback keeps the missing seven rendering in the system face rather
	## than as tofu. They lose Plex's metrics, which is the cost of §12's
	## premise being wrong; drawing them instead would be the alternative, and
	## is recorded in DCC_SHELL_SPEC.md's header as a question for the design.
	var sys := SystemFont.new()
	sys.font_names = PackedStringArray(["Segoe UI Symbol", "Segoe UI", "DejaVu Sans"])
	fv.fallbacks = [sys]
	_tracked[key] = fv
	return fv

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

## Phone geometry (§13, `design/…dc.html`'s "DCC shell android phone" and
## "Phone inset rules" cards). Tablet reuses the desktop constants above
## through `TOUCH_SCALE`; phone is a distinct composition with its own fixed
## pixel budget, read directly off the 393×852 mockup rather than derived
## from the desktop numbers, because none of the desktop regions survive
## phone width unchanged.
const PHONE_REF_SHORT := 393.0   ## The mockup's own short-side width -- the
	## scale of "1 phone pixel" that every constant below is authored at.
	## `DccShell._phone_scale` maps it onto the real device's short side.
const H_PHONE_TOP_SAFE := 44     ## Keep-clear status row: glyphs only.
const H_PHONE_TOP_SCRIM := 96    ## The gradient reaches past the safe area
	## itself so the fade reads as atmosphere, not a hard edge at 44 px.
const W_PHONE_CUTOUT := 108      ## Centre lane reserved for a notch/punch-hole.
const H_PHONE_APP_BAR := 52      ## ☰ / title+seed / ▤ / ⋯.
const W_PHONE_RAIL := 44         ## Domain rail column width == its hit height.
const H_PHONE_GESTURE := 26      ## Bottom gesture inset -- no tappable target.
const PHONE_TAP_MIN := 44        ## §13's floor, with no exceptions.

# ── Active palette ───────────────────────────────────────────────────────────

static var _dark := true

## PR-13/PR-14: flips the active token set. This alone repaints nothing --
## every node that already called `c()` baked a `Color` value into its own
## override, not a live reference to this dictionary. `DccShell.rebuild_theme()`
## is the other half: it walks the tree and repaints what this call only
## re-pointed.
static func apply_theme(is_dark: bool) -> void:
	_dark = is_dark

static func is_dark() -> bool:
	return _dark

## The reverse of `c()`. Given a colour some node already has (painted under
## `old_pal`, the palette that was active when it was built) and that same
## `old_pal`, returns the colour the *token that produced it* now resolves to
## under the palette active this instant -- or `null` if `value` matches no
## token at all (a literal, non-token colour, e.g. a phone overlay's plain
## dim scrim). Tried as an exact RGBA match first, then RGB-only so an
## alpha-blended derivative (`Color(c("bg"), 0.9)`) keeps its own alpha
## rather than inheriting the token's.
static func remap(value: Color, old_pal: Dictionary) -> Variant:
	for token in old_pal:
		if (old_pal[token] as Color).is_equal_approx(value):
			return c(token)
	for token in old_pal:
		var tv: Color = old_pal[token]
		if is_equal_approx(tv.r, value.r) and is_equal_approx(tv.g, value.g) \
				and is_equal_approx(tv.b, value.b):
			var nc: Color = c(token)
			return Color(nc.r, nc.g, nc.b, value.a)
	return null

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

## A hairline *outline* box: border on all four sides, optional fill.
##
## `panel()` above draws a filled region with the one edge that faces its
## neighbour; this draws the other thing the mockup uses constantly -- a chip,
## a path well, a gallery tile, a selected row -- where the whole rectangle is
## outlined and the fill is either nothing or a wash. `border_token` is a
## palette token so the accent-outlined variants (the mockup's active filter
## chip, its selected folder row, its "Open selected" button) go through the
## same call as the quiet `line` ones instead of hand-rolling a StyleBoxFlat
## per site. Radius stays 0 per §11.
static func outline(border_token: String = "line", bg_token: String = "",
		width: int = 1) -> StyleBoxFlat:
	var sb := StyleBoxFlat.new()
	sb.bg_color = c(bg_token) if bg_token != "" else Color(0, 0, 0, 0)
	sb.border_color = c(border_token)
	sb.set_border_width_all(width)
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

static func label(text: String, token: String = "text", size: int = FS_BODY) -> Label:
	var l := Label.new()
	l.text = text
	l.add_theme_color_override("font_color", c(token))
	l.add_theme_font_size_override("font_size", size)
	return l

## A numeric readout, code, shortcut or anything else the mockup sets in Plex.
## `spacing` is the tracking in whole pixels -- 1 reads as roughly .12 em at
## these sizes, 2 as roughly .22 em.
static func mono_label(text: String, token: String = "text", size: int = FS_READOUT,
		spacing: int = 0, medium: bool = false) -> Label:
	var l := label(text, token, size)
	l.add_theme_font_override("font", mono(spacing, medium))
	return l

## §11's section header: uppercase Plex Mono, widely tracked, faint. The `§`
## marker is the disclosure grammar's L3 sigil and is drawn, not implied.
static func header(text: String, sigil: String = "§") -> Label:
	var body := text.to_upper()
	var l := mono_label(("%s %s" % [sigil, body]) if sigil != "" else body,
		"text_faint", FS_HEADER, 2, true)
	return l

## The one large accent number a context is collapsed down to (§6's elevation).
static func hero(text: String) -> Label:
	return mono_label(text, "accent", FS_HERO, 0, true)

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

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
	"line_soft": Color(1, 1, 1, 0.07),
	## §11's third rule weight, distinct from `line` and absent here until
	## 2026-08-25. The canvas outlines every *control* -- chip, action button,
	## input well, modal footer button -- at .16 and separates every *region*
	## at .10. Drawing both at .10 is why the shell's chips read as suggestions
	## rather than as edges.
	"border": Color(1, 1, 1, 0.16),
	"text": Color("#c8cbcd"),        ## Body text.
	"text_bright": Color("#e8ebec"), ## Headers, active rows, the wordmark.
	## §11's "Ink secondary", missing from this file until 2026-08-25 and the
	## most-used ink in the canvas after body text: 76 occurrences in
	## `DCC shell 1920` alone. It is the colour of a *menu bar item* and of a
	## *parameter row's label* -- the two highest-traffic labels in the shell,
	## both of which were being drawn in `text_dim`, one step too quiet.
	"text_secondary": Color("#a9adb0"),
	"text_dim": Color("#8d9296"),    ## Secondary values.
	"text_faint": Color("#6f7478"),  ## Units, hints, the status bar's quiet half.
	"text_ghost": Color("#5f6468"),  ## Disabled.
	"accent": Color("#e0a34a"),
	"accent_hover": Color("#f0bd72"), ## §11's own row, never added here before.
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

## Re-read off `DCC shell 1920 light` 2026-08-25 rather than off §11's table.
## Four values were wrong and one was inverted: the canvas's *ground* is
## `#f4f2ee` and its *floating* surfaces are `#fbfaf7`, where this file had the
## ground at `#fbfaf7` and floated onto pure white -- so light mode raised a
## menu onto a surface brighter than anything the design draws, over a ground
## a shade too bright to sit under it. `#ffffff` appears nowhere in either
## light canvas.
const LIGHT := {
	"bg": Color("#f4f2ee"),
	"panel": Color("#f2f0ec"),
	"panel_alt": Color("#eeece7"),
	"raised": Color("#fbfaf7"),
	"sunken": Color("#e7e5e0"),
	"line": Color(0, 0, 0, 0.14),
	"line_soft": Color(0, 0, 0, 0.08),
	"border": Color(0, 0, 0, 0.20),
	"text": Color("#23241f"),
	"text_bright": Color("#111210"),
	"text_secondary": Color("#3d3f39"),
	"text_dim": Color("#6b6f6a"),
	"text_faint": Color("#8d9088"),
	"text_ghost": Color("#9a9d95"),
	"accent": Color("#a4650f"),
	"accent_hover": Color("#8a5309"),
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

## The wordmark, and only the wordmark: `font:500 12px 'IBM Plex Mono'` with
## `.26em` in every artboard that draws it. Menu *titles* are not this -- see
## `FS_MENU_ITEM`.
const FS_MENU := 12
## Menu bar titles and menu item labels. The canvas sets these in the prose
## face at `11.5px`, not in Plex: 76 spans of `font-size:11.5px` across
## `DCC shell 1920`, none of them monospaced. Godot font sizes are integers and
## the canvas's other prose size is 11, so 11 it is. Until 2026-08-25 the whole
## menu system was drawn in mono at 12, which is most of why the top of the
## shell did not read like the reference.
const FS_MENU_ITEM := 11
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
	## missing most of them. The count in this comment used to read "seven" and
	## was stale by more than a factor of two -- re-parsed from the shipped
	## `IBMPlexMono-Regular.ttf` cmap on 2026-08-25, **19 of the 24 entries in
	## `DccIcons.SYMBOLS` have no glyph**: ▸ ▾ ⌄ ● ○ ☑ ☐ ✕ ⌫ ▶ ⏸ ☰ ▤ ⋯ 🔒 ⛔ ⚠ ⚡
	## and (until this pass) ＋. `FiraSans-Regular.ttf` is worse, missing ✓ ↶ ↷
	## as well. Only ‹ › § · • × ✓ are native. Recount before quoting this.
	##
	## A fallback keeps the missing ones rendering in the system face rather
	## than as tofu. They lose Plex's metrics, which is the cost of §12's
	## premise being wrong; drawing them instead would be the alternative, and
	## is recorded in DCC_SHELL_SPEC.md's header as a question for the design.
	##
	## **The three names below are all desktop faces and none of them exists on
	## Android, and that turned out not to matter.** Checked on a OnePlus 6T
	## (LineageOS 22.2 / Android 15) rather than reasoned about: `▸ ✕ ● ○ ☰ ▤`
	## and the rest all rasterise correctly there, because
	## `SystemFont.allow_system_fallback` defaults to `true` and Godot's Android
	## backend walks `/system/fonts` on its own when no listed name resolves. So
	## the list being Windows-only is a cosmetic wart, not the bug it looks
	## like -- recorded because the next reader will otherwise "fix" it.
	##
	## Exactly **one** codepoint had no glyph anywhere on that handset:
	## `DccIcons.SYMBOLS["add"]` was `＋` U+FF0B, a *fullwidth* plus, which lives
	## in the CJK compatibility block and is therefore carried by Noto Sans CJK
	## rather than by any of the Noto Symbols faces a non-CJK build installs. It
	## drew as a `FF 0B` tofu box in the Travel Library's ENTRIES header. It is
	## an ASCII `+` now: Plex Mono has that natively, so it needs no fallback at
	## all and loses no metrics -- the one glyph in the table that could be
	## fixed rather than fallen back.
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
## §1's rail width, and also what a *collapsed dock* narrows to. §1's companion
## `W_RAIL_EXPANDED := 200` is gone with the rail expansion itself (2026-08-24):
## the design canvas draws the rail at 40 px in every artboard and never draws
## an expanded one -- see `dcc_shell.gd::_build_rail()`'s header.
const W_RAIL_COLLAPSED := 40
const W_LEFT_DOCK := 372
const W_LEFT_DOCK_MIN := 300
const W_LEFT_DOCK_MAX := 520
const W_RIGHT_DOCK := 300
const W_RIGHT_DOCK_MIN := 260
const W_RIGHT_DOCK_MAX := 460

# ── Menu geometry (§2) ───────────────────────────────────────────────────────
#
# Read off the two artboards that actually draw an open menu, not off §2's
# prose: `DCC shell 1920` (Assets open, with its Asset pack submenu),
# `DCC Cartography style 1920` (File open) and `DCC shell tablet 2560`
# (Data open). The desktop and tablet columns are two *drawn* menus, so the
# tablet figures below are measured, not a multiplier applied to the desktop
# ones -- the same reason `TABLET` above is a table.
#
# | Part | Desktop | Tablet |
# |---|---|---|
# | panel | `padding:5px 0` | `padding:6px 0` |
# | item | `padding:6px 14px` | `padding:9px 18px 9px 30px;min-height:44px` |
# | item text | `font-size:11.5px` prose | `font-size:14px` prose |
# | trailing/shortcut | `10.5px 'IBM Plex Mono' #6f7478` | `13px` |
# | group label | `padding:9px 14px 4px;font:9px Plex;.18em;#5f6468` | `11px`, `padding:11px 18px 5px` |
# | rule | `height:1px;background:rgba(255,255,255,.09);margin:5px 0` | `margin:6px 0` |
# | highlight | `background:rgba(224,163,74,.10);color:#e8ebec` | same |
# | bar title | `padding:9px 11px` at 11.5px | `padding:15px 15px` at 14px |
#
# `pitch` is the whole row box the canvas's padding produces -- 11.5 x 1.45
# line plus 6 + 6 is 28.7 px, and the tablet row states its own 44 px minimum.
# Godot has no per-item height on `PopupMenu`, so `dcc_shell.style_popup()`
# reaches it as `v_separation` and grows the hover box by the same amount.
const MENU := {
	"fs_bar": 11,       "fs_bar_t": 14,
	"fs_item": 11,      "fs_item_t": 14,
	"fs_group": 9,      "fs_group_t": 11,
	"pad_x": 14,        "pad_x_t": 18,
	"pad_y": 5,         "pad_y_t": 6,
	"pitch": 28,        "pitch_t": 44,
	"bar_pad_x": 11,    "bar_pad_x_t": 15,
	"bar_pad_y": 9,     "bar_pad_y_t": 15,
}

## The open/hovered menu item's wash. **Not** `accent_wash`: the canvas draws
## the menu *bar's* open title at `rgba(224,163,74,.08)` and the *item* inside
## the dropdown at `rgba(224,163,74,.10)`, two literals a few lines apart in the
## same artboard. `GUI_GAP_REGISTER.md` §48 (DS-05) matched the item to
## `accent_wash`, which is the .08 one.
##
## Derived from `accent` rather than stored as its own token so a theme switch
## repaints it: `remap()`'s RGB-only pass matches an alpha derivative back to
## the token that produced it and keeps the alpha, which is the same mechanism
## `phone_menu.gd`'s scrim already relies on.
const MENU_HIGHLIGHT_ALPHA := 0.10

static func menu_highlight() -> Color:
	return Color(c("accent"), MENU_HIGHLIGHT_ALPHA)

## One menu figure, desktop or tablet. `touch` is `DccShell._touch`; the phone
## never draws a `PopupMenu` at all (§13 -- `phone_menu.gd` re-presents them as
## rows), so there is no third column.
static func menu(key: String, touch: bool) -> int:
	return int(MENU[key + "_t"] if touch else MENU[key])

## Touch scale (§13) -- the fallback for any figure the table below does not
## name. It is a fallback and not the rule, because §1's tablet column is not a
## single multiplier and never was: 34 -> 52 is x1.53, but 26 -> 36 is x1.38,
## 40 -> 48 is x1.20 and 29 -> 34 is x1.17. Applying 1.53-with-a-44-floor to
## all of them (which is what happened until 2026-08-25) gives a 61 px rail
## where the canvas draws 48, and a 44 px status bar where it draws 36 -- the
## floor firing on chrome that is not tappable and has no business being
## floored.
const TOUCH_SCALE := 1.53
## §1's tablet column, verified figure-by-figure against
## `design/Cartalith DCC Shell.dc.html`'s own `DCC shell tablet 2560`
## artboard: `height:52px` twice (menu bar, tool options), `width:48px` (rail),
## `height:34px` (rail head), `height:88px` (timeline), `height:36px` (status),
## `width:400px` (right dock). Keyed by the desktop figure, which is unique per
## region here.
const TABLET := {
	34: 52,   ## Menu bar, tool options bar, dock header.
	29: 34,   ## The rail's own head cell.
	26: 36,   ## Status bar.
	40: 48,   ## Domain rail width.
	70: 88,   ## Timeline.
}
## Tablet dock width. §1: "400 px" for both, so the desktop 372/300 pair
## converges rather than scaling.
const W_DOCK_TABLET := 400

# ── Tablet interior · role-keyed (§13, DS-03) ────────────────────────────────
#
# `TABLET` above is keyed by the bare desktop integer, and that key space is
# **exhausted**. `GUI_GAP_REGISTER.md` §57 found the tablet artboard maps one
# desktop figure onto two different tablet figures in at least five places; all
# five below were re-measured element-by-element off `DCC shell tablet 2560`
# against `DCC shell 1920` on 2026-08-30 rather than taken on §57's word:
#
# | desktop | tablet, in one place | tablet, in another |
# |---|---|---|
# | `14` | **22** — bar `padding:0 14px` → `0 22px` | **18** — sample grid `gap:6px 14px` → `9px 18px` |
# | `11` | **13** — tool-options mono `font:11px` → `13px` | **14** — frame prose `font:11px/1.4` → `14px/1.4` |
# | `9`  | **15** — menu-bar title `padding:9px 11px` → `15px 15px` | **11** — layer row `gap:9px` → `11px` |
# | `70` | **88** — timeline `height:70px` → `88px` | **90** — slider track `width:70px` → `90px` |
# | `6`  | **9**  — sample grid row gap `6px` → `9px` | **6**, pinned — layers-FAB column `gap:6px` |
#
# So the resolution has to be keyed by **what a figure is for**, not by what it
# happens to equal on the desktop. That is the shape `MENU` above already uses
# (`fs_bar`/`fs_bar_t`); `ROLE` is the same idea generalised to the interior,
# and deliberately does **not** restate any key `MENU` already owns — menu
# figures stay there, so there is one source of truth per figure.
#
# `[desktop, tablet]` per role. Every pair is a *drawn* value from each
# artboard, never a multiplier: the ratios here run ×1.00 (the FAB, hairlines,
# every letter-spacing) to ×3.00 (a chip's vertical padding), and §57 measured
# the full spread as ×1.00–×2.06 with no centre. There is no unit to scale by,
# which is why this is a table.
#
# **This table is tokens only.** Nothing in this file applies it — the walk that
# would push these into live Controls lives in `dcc_shell.gd`, which this pass
# does not own. See `role_px()`'s header for the predicate it must use.
const ROLE := {
	# — Type. Sans and mono take different multipliers off the same 11 px rung.
	"fs_prose": [11, 14],        ## Frame body, dock rows: `font:11px/1.4` → `14px/1.4`.
	"fs_readout": [11, 13],      ## Tool-options bar and the sample grid, Plex.
	"fs_shortcut": [10, 13],     ## Menu trailing text, `10.5px` → `13px` (Godot
		## sizes are integers, so the desktop 10.5 rounds down to the 10 the rest
		## of the desktop canvas's small mono already uses).
	"fs_timeline": [10, 13],     ## Timeline bar, `10.5px` → `13px`.
	"fs_status": [10, 12],       ## Status bar, `10.5px` → `12px`. **Not** the
		## same tablet figure as `fs_timeline` despite the same desktop figure —
		## this pair is the whole argument for a role key in one line.
	"fs_viewport": [10, 13],     ## Viewport furniture: map labels, coordinate
		## readout, projection/zoom block.
	"fs_dock_header": [9, 11],   ## `font:500 9px` .22em → `500 11px` .22em.
	"fs_rail": [10, 12],         ## Rail domain labels, .12em, vertical.
	"fs_rail_head": [11, 13],    ## The rail head's `›` chevron.
	"fs_wordmark": [12, 15],     ## CARTALITH, `font:500 12px` .26em → `500 15px`.
		## `FS_MENU` is the desktop half of this and stays as it is.

	# — Region boxes. These duplicate `TABLET`'s five rows on purpose: `TABLET`
	#   answers `_scaled(px)` for a caller that only has an integer, this answers
	#   a caller that knows which region it is building. Same figures, and if one
	#   ever moves the other must move with it.
	"h_menu_bar": [34, 52],
	"h_tool_options": [34, 52],
	"h_status": [26, 36],
	"h_timeline": [70, 88],
	"h_rail_head": [29, 34],
	"w_rail": [40, 48],

	# — Bar interiors.
	"bar_pad_x": [14, 22],       ## All four bars: `padding:0 14px` → `0 22px`.
	"bar_gap": [18, 22],         ## Tool-options bar's inter-group gap.
	"readout_gap": [22, 26],     ## Menu-bar right readouts and the status bar.
	"timeline_gap": [22, 24],    ## The timeline's transport row — a *third*
		## tablet figure off the same desktop 22 as `readout_gap`.
	"timeline_track_h": [12, 20],  ## Scrub track box.
	"timeline_row_gap": [10, 14],  ## Between the scrub row and the transport row.
	"rail_label_gap": [14, 18],    ## Between the rail's vertical domain labels.

	# — Dock interiors.
	"dock_pad_x": [13, 18],
	"dock_row_pad_y": [4, 8],      ## Layer/list row: `padding:4px 13px` → `8px 18px`.
	"dock_header_pad_y": [8, 12],  ## Section header: `8px 13px` → `12px 18px`.
	"dock_body_pad_y": [10, 14],   ## Grid/body block: `10px 13px` → `14px 18px`.
	"dock_row_gap": [9, 11],       ## Within a row, dot → label → value.
	"grid_gap_y": [6, 9],          ## Sample grid, `gap:6px 14px` → `9px 18px`.
	"grid_gap_x": [14, 18],

	# — Controls.
	"slider_track_w": [70, 90],
	"slider_track_h": [2, 3],
	"chip_pad_x": [9, 16],         ## Mode segment: `padding:3px 9px` → `9px 16px`.
	"chip_pad_y": [3, 9],
	"btn_pad_x": [11, 18],         ## Action button: `padding:3px 11px` → `9px 18px`.
	"btn_pad_y": [3, 9],

	# — Pinned. Listed rather than omitted, because "the canvas draws this the
	#   same at both sizes" is a measured fact and a caller that guesses will
	#   scale it. §11's letter-spacings are pinned too and are not figures this
	#   table carries — `mono()` takes them as an argument.
	"w_fab": [36, 36],             ## The layers button, `36×36` in both artboards.
	"hairline": [1, 1],
	"active_underline": [1, 2],    ## The one border that is *not* pinned: the
		## open menu-bar title's underline, `1px` → `2px solid #e0a34a`.

	# — The touch constraint layer, which is not a scaled property at all.
	#   §57 counted **29** `min-height:44px` and **3** `min-height:34px` in the
	#   tablet artboard against **zero** `min-height` declarations of any kind in
	#   the desktop one. A desktop `0` here means "the design states no
	#   constraint", not "the constraint is zero px" — a consumer must read it as
	#   "leave the control at its content height".
	"chip_min_h": [0, 34],         ## Mode chips (raise/lower/smooth), tier B.
	"btn_min_h": [0, 44],          ## Commit/discard, transport, speed. Tier A.
	"row_min_h": [0, 44],          ## Dock list rows and menu items.
}

## Phone geometry -- **`design/Cartalith Android Phone.dc.html`, 412 dp**.
##
## Owner ruling, 2026-08-25: that eight-screen canvas is the phone authority.
## `DCC_SHELL_SPEC.md` §13's phone column and the 393 dp `DCC shell android
## phone` artboard are superseded, and every figure below was re-read off the
## 412 canvas's own literal inline styles rather than rescaled from the 393 set.
## `DCC_SHELL_SCOPE.md`'s "WHICH CANVAS WINS" header carries the ruling itself.
##
## **Changing `PHONE_REF_SHORT` alone would have been wrong.** `_phone_scale` is
## `short_side / PHONE_REF_SHORT`, so 393 → 412 *drops* it (3.664 → 3.495 at
## 1440, 2.748 → 2.621 at 1080) and shrinks every derived figure by 4.6 % --
## while the authored dp below move independently, some up (the app bar) and
## some hard down (the status row, the gesture inset). The two do not cancel;
## the constants are re-authored, not converted.
##
## Tablet reuses the desktop constants above — its *frame* through `TABLET` and
## `TOUCH_SCALE`, its *interior* through `ROLE`; phone is a distinct
## composition, because none of the desktop regions survive phone width
## unchanged.
const PHONE_REF_SHORT := 412.0   ## The canvas's own short-side width -- the
	## scale of "1 phone dp" that every constant below is authored at.
	## `DccShell._phone_scale` maps it onto the real device's short side.
## `height:28px;padding:0 16px;font:10px 'IBM Plex Mono';color:#8d9296`, clock
## left and signal/battery right -- a *status row*, not the 44 dp keep-clear
## reserve §13 reserved. The 412 canvas draws no gradient scrim over the map
## either: the screen ground is solid above the app bar, which is why
## `H_PHONE_TOP_SCRIM` is gone rather than merely resized.
const H_PHONE_TOP_SAFE := 28
## Landscape only, and the one phone figure with no canvas behind it -- all
## eight 412 screens are portrait. Kept from §13 and derived under
## `DCC_SHELL_SCOPE.md`'s rule 2: the portrait row's "left/right pockets with
## nothing centred" rotated onto the side edge. Portrait reserves no centre
## lane any more; the canvas's status row runs edge to edge.
const W_PHONE_CUTOUT := 108
const H_PHONE_APP_BAR := 56      ## `height:56px;gap:14px;padding:0 12px`,
	## `border-bottom:1px rgba(255,255,255,.09)`. ☰ / title+seed / ⌕.
## `height:64px;background:#131516;border-top:1px rgba(255,255,255,.09)`, five
## equal cells, each a `14px` glyph over a `9.5px/.1em` caption with `gap:4px`.
## `#131516` is one hair off `panel` (`#121314`) and is drawn with the token
## rather than a second near-identical literal -- see `GUI_GAP_REGISTER.md` §48,
## which exists because eleven such literals had accumulated.
const H_PHONE_BOTTOM_NAV := 64
const H_PHONE_GESTURE := 20      ## `height:20px`, handle `112x4` radius 2 at
	## `rgba(255,255,255,.22)`.
const W_PHONE_GESTURE_HANDLE := 112
const PHONE_TAP_MIN := 44        ## The canvas's own TARGETS card: "44 dp icon
	## buttons". Unchanged, and it is a *dp* figure in 412 units now.
const H_PHONE_ROW := 52          ## TARGETS: "52 dp list rows".
const H_PHONE_PILL := 48         ## TARGETS: "48 dp buttons". `border-radius:24px`
	## -- the one place in this design system with a radius, and the reason
	## `pill()` below exists beside §11's radius-0 rule.
const PHONE_ICON_BOX := 40       ## The app bar's own glyph cell. A *layout* box,
	## not a visible one (these buttons draw no background), so the hit target
	## still floors at `PHONE_TAP_MIN` per the TARGETS card.
## `height:32px` row, `height:3px` track, `22x22` round thumb -- the phone's
## slider, which unlike the dock's radius-0 2 px rule has a grabber at all.
const PHONE_SLIDER_ROW := 32
const PHONE_SLIDER_TRACK := 3
const PHONE_SLIDER_THUMB := 22

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

## Whether this session is running touch geometry (§13's tablet/phone column).
## `DccShell` owns the decision and publishes it here once, in `_ready`, so the
## static widget factories -- which have no node and therefore no way to reach
## the shell -- can size a menu the same way the shell's own chrome does.
## Menus are the reason this exists: `DccWidgets.style_popup()` is called from
## `dropdown()`, which is static.
static var _touch := false

static func set_touch(v: bool) -> void:
	_touch = v

static func is_touch() -> bool:
	return _touch

## The narrower question: is this the **phone** composition, not merely a touch
## one? Published the same way and for the same reason as `_touch` -- the widget
## factories are static and cannot reach the shell -- and needed separately
## because the 412 canvas asks for things a tablet must not get. The L2 drill
## row's control count is the first: the phone canvas puts one on every category
## row ("the count is the number of controls inside, so depth is legible before
## the tap"), and no desktop or tablet artboard draws one.
static var _phone_mode := false

static func set_phone(v: bool) -> void:
	_phone_mode = v

static func is_phone() -> bool:
	return _phone_mode

## The phone composition's scale factor, published for the same reason and by
## the same line as `_phone_mode`: a static widget factory has no node, so it
## cannot reach `DccShell._pscale()` and would otherwise have to floor a tap
## target in reference units against an already-scaled control. That is exactly
## the mistake `DccShell._ptap()` carried until 2026-08-30 -- a 44 compared
## against a scaled value never fires past a factor of about 1.1.
##
## `1.0` on desktop and tablet, which is correct: neither composition is drawn
## through `_pscale` at all.
static var _phone_scale := 1.0

static func set_phone_scale(v: float) -> void:
	_phone_scale = maxf(1.0, v)

static func phone_scale() -> float:
	return _phone_scale

## **Tablet and only tablet.** `is_touch()` is true on a phone too — `_phone`
## requires `_touch` (`dcc_shell.gd:335`) — so any tablet-only resolution that
## reaches for `is_touch()` silently fires on phones as well. `GUI_GAP_REGISTER.md`
## §57 refuted a proposed role resolver on exactly that ground, and line 249 of
## `dcc_shell.gd` already records the lesson in the other direction: "the 412
## canvas asks for things a tablet must not get." The converse holds here.
##
## This is the predicate `role_px()` uses and the one anything reading `ROLE`
## must use. It costs one `and`, and it is the difference between a tablet pass
## and a tablet pass that also re-sizes the phone.
static func is_tablet() -> bool:
	return _touch and not _phone_mode

## One `ROLE` figure, resolved for the device this session is running on.
##
## Tablet gets the tablet column; **desktop and phone both get the desktop
## column**, which is not a fallback but the right answer for each: the desktop
## column is what the desktop canvas draws, and the phone never consumes `ROLE`
## at all — `design/Cartalith Android Phone.dc.html` is a separate composition
## with its own `PHONE_*` constants above and its own walk. A phone that somehow
## reaches this call gets the unscaled desktop figure, then `phone_fit()`'s own
## unit applies on top, which is what happens today and is unchanged by this
## table existing.
##
## Unknown role errors rather than guessing, the same way `c()` does: a typo'd
## key that silently returned 0 would collapse a padding or a height to nothing
## and look like a layout bug rather than a spelling one.
static func role_px(role: String) -> int:
	if not ROLE.has(role):
		push_error("DccTheme: unknown role '%s'" % role)
		return 0
	var pair: Array = ROLE[role]
	return int(pair[1] if is_tablet() else pair[0])

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

## The 412 phone canvas's action button, and the **only** rounded surface in
## this design system.
##
## §11's "radius 0 everywhere" is a rule about the *desktop* artboards, and
## `design/Cartalith Android Phone.dc.html` overrides it in every one of its
## eight screens: `flex:1;height:48px;border-radius:24px;background:#e0a34a;
## color:#141617;font:500 11px 'IBM Plex Mono';letter-spacing:.16em` on the
## primary, and the same box `border:1px solid rgba(255,255,255,.16)` with no
## fill on the secondary. A fully-rounded 48 dp target is a phone convention the
## desktop canvas never had to have an opinion about; taking radius 0 to the
## phone would be applying a rule the newer canvas already answered.
##
## Reversed ink is `c("bg")` rather than the literal `#141617`, so a theme
## switch repaints it -- the same choice `DccWidgets.set_mode_segment_on()`
## made for the one filled accent surface on the desktop.
static func pill(primary: bool, radius: int, pad_x: int, pad_y: int) -> StyleBoxFlat:
	var sb := StyleBoxFlat.new()
	sb.bg_color = c("accent") if primary else Color(0, 0, 0, 0)
	if not primary:
		sb.border_color = c("border")
		sb.set_border_width_all(1)
	sb.set_corner_radius_all(radius)
	sb.content_margin_left = pad_x
	sb.content_margin_right = pad_x
	sb.content_margin_top = pad_y
	sb.content_margin_bottom = pad_y
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

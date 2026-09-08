extends RefCounted
class_name DccTheme

## Colour tokens and StyleBox factories for the DCC shell.
##
## **Re-based 2026-08-31 onto `design/dcc-environment-2026-08-31/Cartalith DCC
## Environment.dc.html`** -- cited throughout as `ENV:<line>`, its companion
## `cartalith-dcc-parts.js` as `PARTS:<line>`, and the phone prototype
## `Cartalith Android.dc.html` as `AND:<line>`. That prototype arrived with the
## owner's instruction "Replace the current GUI, do not upgrade. Fully replace."
## Every value below is read off its own two token blocks -- the dark root at
## `ENV:25` and the light override string `themeStr` at `ENV:1818` -- and not
## off `design/Cartalith DCC Shell.dc.html`, which this file was written against
## until today and which the new prototype supersedes.
##
## The *shape* is unchanged and deliberately so: the prototype ships one dark
## block and one light block of the same custom-property names, so the tokens
## still come in pairs and the shell still swaps `ACTIVE` between them rather
## than restyling anything.
##
## **This is stage 1 of the replacement: token values only.** Nothing here
## re-assigns which token a region draws with, and no accessor is renamed --
## `DccTheme` is consumed by ~30 files and a rename is 30 files of churn for no
## design reason. Where the prototype moves a *region* onto a different token,
## the value is retuned here and the re-assignment is left to the structural
## stage, named at the site. The three the next reader will look for:
##
## - the menu bar's open title is `var(--ins)` (`ENV:1821`), not an accent wash;
## - the menu bar, the tool-options bar and the status bar have **no background
##   of their own** and sit straight on `--sur`, separated by one `--hair` rule
##   (`ENV:56`, `ENV:109`, `ENV:1218`) -- this shell draws all three on
##   `panel`/`panel_alt`;
## - a menu row's OFF toggle track is `var(--ins)` (`PARTS:180`) where a dock
##   row's is `var(--sur)` (`ENV:1838`). The prototype carries both idioms; the
##   audit logged it as an open design defect and it is not resolved here.
##
## `DCC_SHELL_SPEC.md` §1 owns the geometry; this file owns only colour, type
## and the borders that separate regions.

# ── Palette ──────────────────────────────────────────────────────────────────

## `ENV:25` is the dark root: 34 custom properties on one line. The mapping onto
## the names this file already publishes, so the next reader can diff the two
## without re-deriving it:
##
## | prototype | here | prototype | here |
## |---|---|---|---|
## | `--sur`   | `bg`             | `--acc`    | `accent`         |
## | `--pan`   | `panel`          | `--accH`   | `accent_hover`   |
## | `--ins`   | `sunken`         | `--accInk` | `accent_ink`     |
## | `--ink`   | `text_bright`    | `--wash`   | `accent_wash`    |
## | `--body`  | `text`           | `--wash2`  | `accent_wash_2`  |
## | `--sec`   | `text_secondary` | `--hair`   | `line`           |
## | `--dim`   | `text_dim`       | `--div`    | `line_soft`      |
## | `--faint` | `text_faint`     | `--bor`    | `border`         |
## | `--dis`   | `text_ghost`     | `--block`  | `block`          |
## |           |                  | `--water`  | `water`          |
## |           |                  | `--good`   | `good`           |
##
## Five tokens here have **no** counterpart in the prototype and keep the values
## they had -- `panel_alt`, `raised`, `accent_dim`, `stale`, `stale_wash` --
## each annotated in place with what the prototype does instead.
##
## One prototype property is deliberately **not** imported: `--g`, a 10 px /
## 12 px gap unit that `grep -c "var(--g)"` over `ENV` plus `PARTS` returns
## **0** for. Importing it would reproduce the prototype's own dead-token defect
## in a file whose consumers would then have to guess what it meant.
##
## **`--good` was in that sentence until 2026-09-03 and is not any more.** It is
## dead in the prototype on the same grep, but it is not dead *here*: the
## faction roster's terrain-fit verdict needs the positive half of a pair whose
## negative half is `block`, and was carrying a raw literal to get it. See
## `good`'s own comment in `DARK` for the measurement that settled it. `--shadow`
## is skipped too, but for a third reason: `DccWidgets.style_popup():473`
## already draws exactly `0 14px 34px rgba(0,0,0,.55)` / `rgba(35,36,31,.16)`
## as literals, so a token would be a second source of truth for a value that
## is already correct.
const DARK := {
	"bg": Color("#0d0e0f"),          ## `--sur`. Application ground, and the
		## viewport letterbox. Unchanged by the re-base.
	"panel": Color("#121314"),       ## `--pan`. Docks, and *also* every
		## floating surface: the prototype raises the menu popup (`ENV:62`) and
		## the layers popover (`ENV:902`) on `var(--pan)` with `--bor` and
		## `--shadow`, never onto a brighter third grey.
		## `DccWidgets.style_popup()` already does precisely that, which is why
		## the collapse costs this shell nothing. Unchanged in value.
	## No prototype counterpart. `ENV:56`, `ENV:109` and `ENV:1218` draw the
	## menu bar, the tool-options bar and the status bar with **no background at
	## all** -- they sit on `--sur` and are separated by one `--hair` rule. This
	## file's `panel_alt` consumers are two of those three bars
	## (`dcc_shell.gd:1006`, `:2090`) plus the planner totals column
	## (`journey_planner_view.gd:1315`), so the token survives stage 1 at its
	## old value and the re-assignment belongs to the structural stage. Kept
	## rather than deleted: deleting it now would silently repaint three regions
	## in a pass whose whole contract is "look different, behave identically".
	"panel_alt": Color("#111210"),
	## No prototype counterpart either, and for the opposite reason -- the
	## prototype has no surface *above* `--pan`. Its three dark greys are
	## `--sur` #0d0e0f (ground) < `--pan` #121314 (panel and float) < `--ins`
	## #191c1e (inset), and #17191a falls between the last two. Kept for the
	## consumers that are not menus and that the prototype does not draw.
	##
	## **Three today**, by `grep -rn 'c("raised")'` rather than from memory: a
	## hover stylebox and a `Window` title outline (`dcc_shell.gd`), and one
	## stylebox override in `civilization_workspace.gd`.
	##
	## *This listed **four**, at four line numbers, until 2026-09-03 — and by
	## then two of them (an image-preview backdrop in `asset_library_window.gd`,
	## a swatch well in `place_editor_window.gd`) had been deleted by the very
	## pass that left this comment standing, while the other two had drifted to
	## different lines. Counted here, not cited: a consumer list with line
	## numbers goes stale faster than the tokens it describes.*
	##
	## **Note the inversion this pass creates.** `raised` (#17191a) is now
	## *darker* than `sunken` (#191c1e). Both are independently right against
	## their own sources; the pair is no longer a ramp and must not be read as
	## one. If that ever matters visually, the fix is to move `raised` onto
	## `panel` the way the prototype does, not to un-do `sunken`.
	"raised": Color("#17191a"),
	## `--ins`, and the single largest visible change in this pass: **#101112 ->
	## #191c1e**, from a shade *darker* than `bg` to a shade *lighter* than
	## `panel`. The old canvas sank an input well below the ground; the new one
	## lifts it above the panel. It is the prototype's most-used surface after
	## `--pan` -- 130 `var(--ins)` occurrences across `ENV` and `PARTS` -- and
	## it now carries three distinct jobs: input wells, the **open menu-bar
	## title** (`ENV:1821`, where this shell still paints an accent wash), and
	## the OFF track of a menu-row toggle (`PARTS:180`).
	"sunken": Color("#191c1e"),
	"line": Color(1, 1, 1, 0.10),      ## `--hair`. The rule every region is
		## separated by. Unchanged.
	"line_soft": Color(1, 1, 1, 0.07), ## `--div`. The lighter rule, used inside
		## a surface rather than between two: menu separators (`ENV:65`).
		## Unchanged.
	## `--bor`. Control outlines -- chip, action button, input well, modal
	## footer button -- as against `line`'s region separators. Unchanged, and
	## the prototype confirms the distinction this file added on 2026-08-25:
	## `ENV:62` outlines the menu popup at `--bor` while `ENV:56` separates the
	## menu bar at `--hair`, six lines apart.
	"border": Color(1, 1, 1, 0.16),
	"text": Color("#c8cbcd"),           ## `--body`. Unchanged.
	"text_bright": Color("#e8ebec"),    ## `--ink`. Unchanged.
	"text_secondary": Color("#a9adb0"), ## `--sec`. Unchanged.
	"text_dim": Color("#8d9296"),       ## `--dim`. Unchanged.
	"text_faint": Color("#6f7478"),     ## `--faint`. Unchanged.
	"text_ghost": Color("#5f6468"),     ## `--dis`. Unchanged.
	"accent": Color("#e0a34a"),         ## `--acc`. Unchanged.
	## `--accH`. Unchanged, and worth stating that it is dead **in the
	## prototype**: `grep -c "var(--accH)"` over `ENV` plus `PARTS` is 0, no
	## hover rule reaches for it. It is not imported *from* there -- it survives
	## because this shell has two real consumers (`dcc_shell.gd:3661` and
	## `dcc_widgets.gd:662`) that predate the re-base.
	"accent_hover": Color("#f0bd72"),
	## **New in this pass**, `--accInk` `ENV:25`, and it exists to enforce one
	## rule: nothing may render near-black on a FILLED accent surface. The six
	## reversed-ink sites in this shell used `c("bg")` until today -- #0d0e0f on
	## #e0a34a -- and now use this.
	##
	## Read the light half before assuming what this token is. It is #f7f4ee, a
	## *light* ink, because the light accent #a4650f is dark. `accent_ink` is
	## not "the dark ink"; it is "whatever reads on top of `accent`", and a
	## theme switch flips it end for end.
	"accent_ink": Color("#141005"),
	## `--wash`, retuned **.08 -> .09**. The old canvas wrote it as the hex
	## `#e0a34a14` (alpha 20/255 = .078); the prototype writes
	## `rgba(224,163,74,.09)` at `ENV:25` and uses it as the menu row's hover
	## (`ENV:67`) and the active domain cell's fill (`ENV:1823`). Written as
	## floats rather than an 8-digit hex because .09 has no exact byte.
	"accent_wash": Color(0.878431, 0.639216, 0.290196, 0.09),
	## **New in this pass**: `--wash2`, `rgba(224,163,74,.16)`, the *armed*
	## weight at almost twice `--wash`. The prototype uses it 44 times and
	## always for the same distinction -- a control that is armed and will act
	## on the next click, as against one that is merely current: `inspChips`
	## (`ENV:1910`), `measSegBg` (`ENV:1912`), the timeline speed pills
	## (`ENV:1978`), `ldSwABg` (`ENV:1941`), `layersBtnBg` (`ENV:1959`).
	##
	## ~~**Nothing in this port reads it yet** … the one token here whose
	## consumer is in the future rather than the past.~~ **Both halves of that
	## went stale, and on different dates.** It has consumers --
	## `grep -rn '"accent_wash_2"' --include=*.gd . | grep -v dcc_theme | grep
	## -v "^./_"` returned **nine** on 2026-09-06 before this token pass and ten
	## after it: the data manager's route-segment track, its carried-format row
	## and its bound-column block; the phone transport's play button on hover
	## and on press; the right dock's pressed history chip and its accent stance
	## chip; the viewport's armed-tool fill; `modal_destructive()`, which
	## borrows the alpha and not the hue; and now `set_segment_on()`.
	##
	## The second half went with them. The paragraph's real claim was that
	## "armed vs. merely current" could not be drawn without deciding, control
	## by control, which of some forty is which -- and the prototype turns out
	## to make that decision itself, cleanly, along a line the shell already
	## has a widget for. `var(--wash2)` is 36 uses and every one is a **segment
	## or toggle on-state**; `var(--wash)` is 13 and every one is a **hover or
	## a list-row selection**. `DccWidgets.set_segment_on()` is exactly the
	## first set, so the whole class moved through one call on 2026-09-06 --
	## see that function's own header for the two greps, the owner ruling that
	## names this token (`LARGE_ITEM_RULINGS.md` §6/§7) and the windowed
	## measurement that had to come first.
	"accent_wash_2": Color(0.878431, 0.639216, 0.290196, 0.16),
	## No prototype counterpart: besides the two washes, `--accH` is the only
	## amber derivative the prototype declares. Kept at the old canvas's value
	## for its existing consumers.
	"accent_dim": Color("#a4650f"),
	## No prototype counterpart. The prototype expresses "downstream is stale"
	## as a **literal em dash** in the readout (`ENV:1835` turns a stale field
	## into `label · —`; the twelve `sampleRows` gate the same way at
	## `ENV:1866`) plus `pipeNoteCol:'var(--acc)'` on the pipeline note
	## (`ENV:1924`) -- never as a colour of its own. Both tokens keep their old
	## values until the structural stage decides whether this shell adopts the
	## em-dash idiom or keeps a stale tint.
	"stale": Color("#b9a878"),
	"stale_wash": Color("#3d3226"),
	## The semantic pair, and **the one place the two delivered prototypes
	## contradict each other outright**:
	##
	## |       | `ENV:25` / `ENV:1818` | `AND:31` / `AND:1469` |
	## |---|---|---|
	## | good  | `#6fae7d` / `#2c7a44` | `#8fae7d` / `#4e6f3f` |
	## | block | `#c96a5a` / `#a03d2e` | `#c26a60` / `#a04437` |
	## | water | `#6a9bc4` / `#2e6a9e` | `#7d9dae` / `#3f6675` |
	##
	## Both files are dated 2026-08-31, so `CLAUDE.md`'s "the newer canvas wins"
	## does not separate them. `ENV` is taken here on two grounds: it is the
	## prototype this file's desktop half is being re-based onto, and it is the
	## only one of the two that states a value for all three in **both** themes
	## from one source. The divergence is recorded, not averaged -- if the phone
	## composition ever needs its own semantic triple it needs a second palette,
	## not a compromise in this one.
	##
	## **`good` IS imported, since 2026-09-03**, reversing this block's own
	## "not imported at all" and the header table's reasoning with it. That
	## reasoning was *"`grep -c "var(--good)"` over `ENV` plus `PARTS` returns
	## **0**; importing it would reproduce the prototype's own dead-token defect
	## in a file whose consumers would then have to guess what it meant."* The
	## grep is still 0 -- re-run 2026-09-03 -- but the premise behind it was that
	## this shell had no consumer either, and it does:
	## `faction_roster_window.gd::_build_terrain_fit()` paints its "strong match"
	## verdict green and its "mismatch" verdict in `accent`, and until stage 7 the
	## green half was the raw literal `Color(0.48, 0.78, 0.49)` (#7ac77d) -- a
	## *fixed* colour in both themes, measured at **1.96:1** against the light
	## `panel` #fbfaf7, which is unreadable. The value below measures **5.06:1**
	## there and 7.20:1 on the dark panel.
	##
	## So the consumer does not have to guess what it means: it is the positive
	## half of the same verdict pair whose negative half is already a token.
	## Values are `ENV`'s own, on exactly the grounds the table above gives for
	## taking `ENV` over `AND` for `block` and `water` -- one source, both themes.
	"good": Color("#6fae7d"),
	"block": Color("#c96a5a"),
	"water": Color("#6a9bc4"),
	## `--warn` exists in **neither** `ENV` block: `grep -c -- "--warn:"` over
	## the whole file returns 0. Its only source is the phone prototype
	## (`AND:31` dark, `AND:1469` light), which BUILD_ANSWERS §4 confirms as a
	## deliberate addition there. Held at the phone's dark value, which is what
	## this file already carried; the light half below is the one that moves.
	"warn": Color("#e0a840"),
	## **New in this pass**: `scrimBg` `ENV:1963`, and BUILD_ANSWERS §2.6
	## confirms the two halves differ deliberately rather than being one alpha
	## over two grounds.
	##
	## Named for the HUD and not "scrim" because it is **not** a modal dim: it
	## is the pill drawn behind each of the three viewport HUD readouts
	## (`ENV:911` context, `ENV:913` projection/zoom, `ENV:918` coordinates).
	## The phone sheet scrim (`phone_menu.gd:224`) is a different thing and
	## stays `Color(c("bg"), 0.72)` off its own canvas.
	##
	## **Nothing consumes it yet** -- this port has no desktop viewport HUD.
	## Imported because the value is settled and the HUD is a later stage's
	## work; unlike `accent_wash_2` it needs no design decision to wire, only
	## the widget to exist.
	##
	## One trap for whoever does wire it: in **both** themes this token's RGB is
	## `bg`'s RGB exactly -- `rgba(13,14,15,.62)` against #0d0e0f here, and
	## `rgba(244,242,238,.72)` against #f4f2ee below. `remap()` still resolves
	## it correctly, because its **exact-RGBA** pass runs first and `bg` carries
	## alpha 1.0; but a further alpha derivative (`Color(c("hud_scrim"), 0.5)`)
	## would fall through to the RGB-only pass and be matched back to `bg`,
	## which comes first in this dictionary. Paint with the token as it stands;
	## do not derive from it.
	"hud_scrim": Color(0.050980, 0.054902, 0.058824, 0.62),
}

## The light half is `themeStr` at `ENV:1818` -- a string of the same custom
## properties, concatenated ahead of `densStr` and applied over the dark root
## (`fvars:themeStr+densStr`, `ENV:1896`). It is empty when dark, so the root at
## `ENV:25` **is** the dark theme rather than a base both themes override.
##
## The 2026-08-25 re-read off `DCC shell 1920 light` turns out to have been
## right about six of its eight inks: `text`, `text_bright`, `text_secondary`,
## `text_dim`, `text_faint` and `text_ghost` all match the new prototype
## character for character, as do `bg`, `accent`, `accent_hover` and all three
## rules. Three values move -- `panel`, `sunken` and `warn` -- and two are new.
const LIGHT := {
	"bg": Color("#f4f2ee"),          ## `--sur`. Unchanged.
	## `--pan`, **#f2f0ec -> #fbfaf7**. The prototype's light theme has one
	## surface for panels and floats where the old canvas had two, and it is the
	## *brighter* of the old pair: what this file called `raised` is what the
	## prototype calls `--pan`. So light mode's docks brighten by three steps
	## and its menus stay exactly where they were. `#ffffff` still appears
	## nowhere in either theme.
	"panel": Color("#fbfaf7"),
	## No prototype counterpart -- see the dark half. Unchanged, and now only
	## one step below `panel` rather than four, which is the correct reading of
	## a token whose whole job is "a shade back".
	"panel_alt": Color("#eeece7"),
	## No prototype counterpart, and in light it is already *equal* to the new
	## `panel` -- the collapse the dark half only describes has in effect
	## already happened here. Left as its own entry rather than aliased, so the
	## two halves stay symmetrical and a future divergence has somewhere to go.
	"raised": Color("#fbfaf7"),
	## `--ins`, **#e7e5e0 -> #eceae4**. Lighter, and in the same direction as
	## the dark half's much larger move: an inset well is a step *toward* the
	## panel in this design system, not away from it.
	"sunken": Color("#eceae4"),
	"line": Color(0, 0, 0, 0.14),      ## `--hair`. Unchanged.
	"line_soft": Color(0, 0, 0, 0.08), ## `--div`. Unchanged.
	"border": Color(0, 0, 0, 0.20),    ## `--bor`. Unchanged.
	"text": Color("#23241f"),           ## `--body`. Unchanged.
	"text_bright": Color("#111210"),    ## `--ink`. Unchanged.
	"text_secondary": Color("#3d3f39"), ## `--sec`. Unchanged.
	"text_dim": Color("#6b6f6a"),       ## `--dim`. Unchanged.
	"text_faint": Color("#8d9088"),     ## `--faint`. Unchanged.
	"text_ghost": Color("#9a9d95"),     ## `--dis`. Unchanged.
	"accent": Color("#a4650f"),         ## `--acc`. Unchanged.
	"accent_hover": Color("#8a5309"),   ## `--accH`. Unchanged. Darker than
		## `accent`, where the dark theme's is lighter -- both move *away* from
		## the ground, which is why this is a token pair and not one colour.
	## **New**, `--accInk`. #f7f4ee: paper, not ink. See the dark half -- this
	## is the token that makes "reversed on accent" survive a theme switch, and
	## it is the reason `c("bg")` was the wrong thing to reverse onto. On the
	## light theme `c("bg")` is #f4f2ee, which is *nearly* right by accident and
	## fails by a shade; on the dark theme it was wrong outright.
	"accent_ink": Color("#f7f4ee"),
	## `--wash`, retuned **.102 -> .09**. The old canvas wrote `#a4650f1a`
	## (26/255); the prototype writes `rgba(164,101,15,.09)`, the same alpha as
	## the dark half, so the two themes now agree on the wash weight where they
	## used to differ by a quarter.
	"accent_wash": Color(0.643137, 0.396078, 0.058824, 0.09),
	## **New**: `--wash2`, `rgba(164,101,15,.16)`. ~~Unconsumed for now~~ --
	## consumed since, on this half as much as the dark one: the light palette
	## is what this machine boots, and the segment on-state's .09 -> .16 step
	## measures **15-16 / 255** here against the dark half's 14-15. See the
	## dark half for the whole correction.
	"accent_wash_2": Color(0.643137, 0.396078, 0.058824, 0.16),
	"accent_dim": Color("#7a6a4a"),  ## No prototype counterpart. Unchanged.
	## No prototype counterpart. Unchanged; see the dark half.
	"stale": Color("#7a6a4a"),
	"stale_wash": Color("#e2d7bd"),
	## `ENV:1818`'s `--block` and `--water`. These were the *dark* values until
	## today -- `JOURNEY_PLANNER_SPEC.md` §10 listed light theme as unbuilt for
	## that feature and the three tokens existed only so `c()` would not error.
	## They are real light values now, and they are markedly more saturated than
	## the phone's (`#a03d2e` against `AND:1469`'s `#a04437`, `#2e6a9e` against
	## `#3f6675`). See the dark half's table for the full contradiction.
	##
	## `good` joins them 2026-09-03 -- `ENV:1818`'s own `--good`, and the whole
	## reason this token now exists: the dark half of the pair was legible as a
	## raw literal and the light half was not. See the dark half's comment.
	"good": Color("#2c7a44"),
	"block": Color("#a03d2e"),
	"water": Color("#2e6a9e"),
	## The one token here sourced from the phone prototype rather than `ENV`,
	## because `ENV` declares no `--warn` in either theme. `AND:1469`, confirmed
	## by BUILD_ANSWERS §4 as one of the four semantic colours that gained a
	## light value in this delivery. **This is a real change**: light mode was
	## drawing `warn` at the dark #e0a840, an amber that has no contrast against
	## a #f4f2ee ground.
	"warn": Color("#9a6a12"),
	## **New**: `scrimBg` light, `ENV:1963`. Note the alpha differs from the
	## dark half -- .72 against .62 -- which BUILD_ANSWERS §2.6 states is
	## deliberate. A light HUD pill has to work harder to separate its text from
	## a map beneath it, because the map does not lighten with the chrome
	## (BUILD_ANSWERS §4: "light chrome over a dark map is the intended
	## pairing"). Unconsumed; see the dark half.
	"hud_scrim": Color(0.956863, 0.949020, 0.933333, 0.72),
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
## The one big accent readout per context (§6's elevation). `--hero` at
## `ENV:25`, unchanged at this density -- but it is a *pair* now: 30 px on
## touch (`ENV:1819`), which BUILD_ANSWERS §2.4 added because the readout used
## to be a bare literal that did not scale with the 11 px mono around it. This
## constant is the desktop half and keeps its name; the pair lives in `ROLE`
## as `fs_hero`, beside `fs_hero_2` for `--hero2` (22 / 26), which this file
## had no name for at all. **`hero()` reads the pair, not this constant** --
## it hard-coded `26` until 2026-08-31, which is how the token stayed
## unconsumed while looking done. What is left here is the desktop figure's
## documented origin and `right_dock.gd`'s own citation of it.
const FS_HERO := 26
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
#
# Re-read off the new prototype's own token block (`ENV:25`) for the base set
# and `densStr` (`ENV:1819`) for the touch set. Four of the eight figures below
# move; the ones that do not are stated as unchanged rather than left silent,
# because "still 372" is a measurement too.

const H_MENU_BAR := 36      ## `--menuH`, **34 -> 36** (`ENV:25`). Touch 52,
	## unchanged, so the touch/desktop ratio tightens from x1.53 to x1.44.
const H_TOOL_OPTIONS := 40  ## `--tbH`, **34 -> 40** (`ENV:25`), touch **52 ->
	## 56** (`ENV:1819`). The tool-options bar is the one bar that grew at both
	## densities: it now carries the run/finalize block (`ENV:1921`-`1928`) and
	## the four mutually-exclusive tool rows, where the old canvas drew only
	## chips. **This figure collides with `W_RAIL_COLLAPSED` in `TABLET`'s key
	## space** -- see `TABLET`'s header for what that forced.
## No prototype counterpart any more, and this is a real gap rather than an
## unchanged value. The new prototype's timeline has two states and neither is
## 70 px: collapsed is authored at `calc(var(--sbH) - 2px)` = 24 px (`ENV:1187`)
## and expanded is content-driven -- `flex:none;...;gap:6px;padding:8px var(--pad)`
## (`ENV:1195`) with no authored height at all. It is also domain-gated now
## (`tlShow:s.domain==='CIVIL'&&this.cc()!=='planner'`, `ENV:1973`), which this
## shell does not do. Held at 70 so stage 1 moves no structure; the timeline is
## a structural-stage rewrite, not a token retune.
const H_TIMELINE := 70
const H_STATUS := 26        ## `--sbH`. Unchanged, touch 36 unchanged.
## `--railW` (`ENV:25`). Unchanged, and still also what a *collapsed dock*
## narrows to.
##
## **`W_RAIL_EXPANDED` is coming back.** It was deleted on 2026-08-24 with the
## reasoning "the design canvas draws the rail at 40 px in every artboard and
## never draws an expanded one" -- true of the old canvas, false of this one.
## `ENV:294` draws the expansion column at `var(--railExpW)`, `ENV:1929` binds
## its chevron and `ENV:1934` its node clicks, and BUILD_ANSWERS §2.5 rules on
## how it opens. The width lives in `ROLE` as `w_rail_expanded` rather than as a
## const here, because unlike `W_RAIL_COLLAPSED` it is density-varying
## (200 / 264) and has no consumer yet -- `dcc_shell.gd::_build_rail()` still
## builds the collapsed-only rail. Rebuilding the rail is stage 2's work.
const W_RAIL_COLLAPSED := 40
const W_LEFT_DOCK := 372    ## `--ldW`. Unchanged at the base density; 330 on
	## the new LAPTOP band and 400 on touch -- see `LAPTOP` and `ROLE`.
const W_LEFT_DOCK_MIN := 300
const W_LEFT_DOCK_MAX := 520
const W_RIGHT_DOCK := 304   ## `--rdW`, **300 -> 304** (`ENV:25`). Four pixels,
	## and they are not noise: the right dock's sample grid is a two-column
	## key/value table (`ENV:1866`'s twelve rows) whose widest value,
	## `SLOPE · ASPECT`'s `'18° · 243°'`, sets the column. 280 on LAPTOP,
	## 400 on touch.
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

## The hovered menu item's wash, retuned **.10 -> .09**, at which point it is
## no longer distinct from `accent_wash`.
##
## The old canvas drew two amber washes a few lines apart -- the menu *bar's*
## open title at `rgba(224,163,74,.08)` and the *item* inside the dropdown at
## `.10` -- and this constant existed to hold the second. The new prototype
## keeps neither pairing: its item hover is `style-hover="background:var(--wash)"`
## (`ENV:67`), the same .09 wash as everything else, and its open bar title is
## not a wash at all but `var(--ins)` (`ENV:1821`).
##
## So the constant survives with one value instead of two. It is **not** deleted
## in favour of `c("accent_wash")` directly, for the same reason `panel_alt`
## survives above: the open-title half of the pairing is a structural
## re-assignment this stage does not make, and when it is made this is the hook
## that half will move through.
##
## Still derived from `accent` rather than stored as its own token so a theme
## switch repaints it: `remap()`'s RGB-only pass matches an alpha derivative
## back to the token that produced it and keeps the alpha, the same mechanism
## `phone_menu.gd`'s scrim relies on. Note that it now produces a colour
## *exactly* equal to `accent_wash`, so `remap()`'s prior exact-RGBA pass
## resolves it through that token -- which is correct, and is why the two must
## not be allowed to drift apart again without a reason in the prototype.
const MENU_HIGHLIGHT_ALPHA := 0.09

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
## The tablet column, re-read off `densStr` (`ENV:1819`) rather than off the old
## `DCC shell tablet 2560` artboard: the prototype states its touch density as
## one string of the same custom properties the base set uses, so every figure
## here is a *declared* token value rather than an element measured off a
## drawing.
##
## **Keyed by the desktop figure, and that key space has now collided.** The new
## `--tbH` is 40, which is also `--railW`, and the two want different tablet
## answers -- 56 and 48. A dictionary keyed by a bare integer cannot serve both,
## so the tool-options bar was moved off `DccShell._scaled()` and onto
## `role_px("h_tool_options")` (`dcc_shell.gd:1004`). That is the resolver
## `ROLE` exists for and the same remedy `GUI_GAP_REGISTER.md` §57 prescribed
## when this key space first ran short. `40` below therefore means the rail and
## only the rail.
##
## `34` is kept alongside the new `36` rather than replaced. 36 is the menu bar;
## 34 is the *dock header*, which `dcc_shell.gd:1835`/`:1902` still ask for and
## for which the prototype authors no height at all (`ENV:305` is
## `padding:8px var(--pad) 0` around a `var(--ctl)` button). Both resolve to 52,
## which is exactly why dropping 34 would have looked harmless: it would have
## fallen through to `TOUCH_SCALE` and landed on 52 by arithmetic coincidence.
## Keeping the row makes the dock header's tablet height a stated figure rather
## than a lucky one.
const TABLET := {
	36: 52,   ## Menu bar -- `--menuH` 36 -> 52 (`ENV:1819`).
	34: 52,   ## Dock header. Not a prototype figure; see above.
	30: 44,   ## The rail's own head cell -- `var(--tool)` (`ENV:284`), 30 px
		## pointer and 44 px touch. **This row was `29: 34` until stage 2.**
		## Stage 1 measured the prototype's figure, found the shipped one stale,
		## and left it: "changing it moves the rail head's box, and the rail is
		## stage 2's rebuild." `dcc_shell.gd::_build_rail()` is that rebuild, and
		## it now asks for `_scaled(30)`; it was the only caller of `_scaled(29)`
		## in the project, so the old key had no other consumer to strand. The
		## touch figure is a real gain and not cosmetic: 34 px was below `ROLE`'s
		## own 34 px tier-B floor's intent for a control that is now a `Button`
		## rather than the inert `Label` it was when 34 was written.
	26: 36,   ## Status bar -- `--sbH` (`ENV:1819`).
	40: 48,   ## Domain rail width -- `--railW`. **Not** the tool-options bar;
		## see the header above.
	70: 88,   ## Timeline. No prototype counterpart -- see `H_TIMELINE`.
}
## Tablet dock width. `--ldW:400px;--rdW:400px` (`ENV:1819`) -- the desktop
## 372/304 pair still converges rather than scaling, exactly as the old canvas
## had it, so this constant survives the re-base unchanged.
const W_DOCK_TABLET := 400

# ── The fourth density: LAPTOP 1366 ──────────────────────────────────────────
#
# There were three density sets and they were three *device classes* -- desktop,
# tablet, phone -- resolved by `is_touch()` and `is_phone()`. The prototype adds
# a fourth that is not a device class at all:
#
#   densStr = (touch ? <touch tokens> : <pointer tokens>)
#           + (!touch && frame==='w1366' ? '--ldW:330px;--rdW:280px;--pop:280px;' : '')
#                                                                    -- ENV:1819
#
# Read the *shape* of that expression rather than its content. LAPTOP is not a
# fourth branch of a four-way switch; it is an **override layer applied on top
# of the pointer set, and only while the pointer set is the one in play**. Three
# tokens change and every other figure is inherited. Modelling it as its own
# column would mean restating two dozen inherited values and inviting the copies
# to drift -- which is the failure `ROLE`'s own header describes when `TABLET`'s
# five rows had to be duplicated.
#
# So it is a dictionary of overrides keyed by the same role names `ROLE` uses,
# consulted by `role_px()` *before* `ROLE`, and gated on `is_laptop()`, which is
# `narrow and not touch`. That `not touch` is the literal `!touch` above and it
# is load-bearing: without it a 1366-wide tablet would take dock widths sized
# for a mouse. It also means the four sets are resolved by two independent
# questions -- "is this touch?" and "is this narrow?" -- rather than by one
# four-way classification, which is what makes phone (touch, and narrow, and
# neither answer consulted because it has its own `PHONE_*` composition) fall
# out without a special case.
#
# **The threshold is derived rather than read, and that is disclosed.** The
# prototype names two pointer frames -- `w1920` (1920x1080) and `w1366`
# (1366x768), `ENV:1675` -- and no boundary between them, because an artboard is
# discrete where a real window is continuous. `W_LAPTOP_MAX` is set to the base
# set's own frame width, so what selects the override is "narrower than the
# width the base set was authored at". That is the only line the two named
# frames support without inventing a figure, and it puts every common panel
# below 1920 (1680, 1600, 1440) on the narrow set and everything at or above it
# on the base set. BUILD_ANSWERS §2.3 argues a threshold for the phone/desktop
# split from chrome-versus-map arithmetic and says nothing about this one; if
# the owner ever states a number, it replaces this constant and nothing else.
const W_LAPTOP_MAX := 1920
## `--ldW` / `--rdW` / `--pop` at `ENV:1819`. Three overrides, no more: the
## prototype leaves `--railExpW`, `--popW`, `--hero` and every type rung at the
## base value on `w1366`.
const LAPTOP := {
	"w_left_dock": 330,
	"w_right_dock": 280,
	"w_menu_popup": 280,
}

# ── The fifth density: TABLET PORTRAIT ───────────────────────────────────────
#
# **This is a defect fix, not adoption of the tablet canvas.** Measured on the
# shipped shell at 800 x 1280 `--force-touch`: rail 47 + left 400 + viewport 237
# + right 400 = 1084 against a frame of 800, the right dock's right edge landing
# at x = 1085 -- **overflow +285 px**, with the menu bar cut mid-word and no map
# visible at all. At 900 x 1440 the overflow is +185. Landscape has no defect
# (1280 x 800 sums 1279, overflow 0), and that is the whole reason only one
# figure moves below.
#
# The prototype states the pair the same way `LAPTOP` states its own, in the
# same `densStr` position (`Cartalith Tablet.dc.html`, `valsShell()`):
#
#   dens = port ? '--ldW:232px;--rdW:232px;...' : '--ldW:320px;--rdW:320px;...'
#
# **Only the portrait half is taken, and that is a decision rather than an
# oversight.** The landscape half would move a surface that measures correct
# today, and `TABLET_UI_SPEC.md` §4.3 names the dock split "the only item that
# buys a defect fix rather than a resemblance" -- the rest of that spec, this
# 320 included, waits on an owner decision that has not been made. Landscape
# therefore keeps `ROLE`'s shipped 400/400 (= `W_DOCK_TABLET`). When the owner
# rules on canvas adoption, the landscape figure moves in `ROLE` and this table
# does not change at all.
#
# **Shape: an override layer, not a third `ROLE` column.** `ROLE`'s own header
# (below) says a 232/320 pair "could not be expressed as a `ROLE` row even if it
# were ruled on; `ROLE` is a two-column table and this is a third column" -- and
# that is right, but it is only an argument against *widening `ROLE`*, not
# against expressing the pair at all. `LAPTOP` above already answers exactly
# this shape of question: two tokens change, three dozen are inherited, and
# modelling it as its own column would mean restating the inherited values and
# inviting the copies to drift. Orientation is that case again, so it gets that
# answer again -- a dictionary of overrides keyed by the same role names,
# consulted by `role_px()`, gated on a predicate. The cost of the next tablet
# orientation row is then one line, in one place, and no caller changes.
#
# `is_laptop()` is `narrow and not touch` and `is_tablet_portrait()` is
# `touch and not phone and portrait`, so the two override layers are mutually
# exclusive by predicate and their order in `role_px()` carries no meaning.
const TABLET_PORTRAIT := {
	"w_left_dock": 232,
	"w_right_dock": 232,
}

# ── Tablet interior · role-keyed (§13, DS-03) ────────────────────────────────
#
# `TABLET` above is keyed by the bare desktop integer, and that key space is
# **exhausted**. `GUI_GAP_REGISTER.md` §57 found the tablet artboard maps one
# desktop figure onto two different tablet figures in at least five places; all
# five below were re-measured element-by-element off `DCC shell tablet 2560`
# against `DCC shell 1920` on 2026-08-30 rather than taken on §57's word:
#
# **"At least five" was a floor, and the count is now measured: this table holds
# 12 colliding desktop integers** (2026-09-03, over 49 rows). Three of them are
# *three*-way — desktop `10` carries {12, 13, 14} and desktop `11` carries
# {13, 14, 18}, which no two-column value table of any keying could express. The
# full enumeration is printed by `_roleresolve_probe.tscn`, which also pins each
# one; the five §57 named are the subset drawn below.
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
# **This table was "tokens only" until stage 2 wired it, and that sentence stood
# here after it stopped being true.** Counted 2026-09-03 rather than recalled:
# Counting `role_px("<key>")` **occurrences** outside comments (not matching
# lines — a line can carry two) gives **87 live call sites in shell code**,
# spread over eight files rather than the three a `dcc_shell.gd`/`dcc_widgets.gd`
# grep suggests: `dcc_widgets.gd` 32, **`right_dock.gd` 26**, `app.gd` 9,
# `workspaces/render_workspace.gd` 6, `dcc_shell.gd` 5, `layers_popover.gd` 5,
# this file 3 (`header()`, `hero()`, `hero2()`), `civilization_workspace.gd` 1.
# Plus 4 in `_tabletparity_probe.gd`. The eight shell files read **22 of the 49 rows
# below**; the other **25** are carried tokens with no consumer yet, and each
# says so where the reason is interesting. See `role_px()`'s header for the
# predicate every reader must use.
#
# **Where the rows come from, and the one disagreement that follows.** 16 rows
# are backed by the governing canvas's own `densStr` token block (`ENV:1819`);
# the other 33 are element-level figures off `DCC shell tablet 2560` against
# `DCC shell 1920`, measured 2026-08-30 — one day *before* the re-base that made
# `ENV` the newer authority, and that re-base declared itself "stage 1: token
# values only". So the two canvases now disagree in at least one place a reader
# will hit: `ENV`'s `--pad` is 14 → **16**, while `bar_pad_x` below is 14 → 22
# from the older artboard, and `ENV:109`/`:149`/`:1187` draw the bars at
# `padding:0 var(--pad)`. **Not resolved here**, deliberately: `ENV` has one
# `--pad` where this table has three separate desktop-14 roles (`bar_pad_x`,
# `grid_gap_x`, `rail_label_gap`), so adopting the token wholesale would
# *destroy* the desktop-14 collision — the exact distinction DS-03 exists to
# hold. It needs an owner ruling on which canvas governs the interior, not a
# lane's edit. Four `densStr` tokens likewise have no row at all: `--ctl`
# (24 → 36), `--btnH` (28 → 44), `--row` (28 → 44) and `--g` (10 → 12, which is
# dead in the prototype and deliberately not imported — see the palette header).
#
# **A SECOND, LARGER DISAGREEMENT, measured 2026-09-07: there is a whole
# tablet canvas this table has never been read against.**
#
# Every tablet figure above is homed to `ENV:1819`'s `densStr` -- the *PC*
# canvas's touch branch. `design/mcp-2026-09-07/Cartalith Tablet.dc.html`, which
# the owner named the same day ("Implement: Tablet"), draws its own and they are
# not the same tablet. Its `valsShell()` writes
# `--menuH:52px;--railH:56px;--railW:52px;--sbH:28px;--tap:44px;--ctl:36px;
# --row:48px;--rowD:44px;--rCtl:12px;--rPan:16px` plus a portrait/landscape
# split `--ldW/--rdW: 232px | 320px` and `--m0/--m1/--m2: 12/11/9.5 | 12.5/11.5/10`.
# Against `ENV`'s touch column, token by token:
#
# | token | `ENV:1819` (this table's source) | `Cartalith Tablet.dc.html` |
# |---|---|---|
# | `--railW` | 48 | **52** |
# | `--sbH` | 36 | **28** |
# | `--row` | 44 | **48** (plus a second `--rowD:44`) |
# | `--ldW` / `--rdW` | 400 / 400 | **232 portrait, 320 landscape** |
# | `--m1` / `--m2` | 12 / 11 | **11 / 9.5** portrait, **11.5 / 10** landscape |
# | — | no counterpart | `--railH:56` (a **horizontal** rail under the menu bar) |
# | — | no counterpart | `--rCtl:12px` / `--rPan:16px` (radii, against §11's radius-0 rule) |
# | `--tbH` 56, `--btnH` 44, `--tool` 44, `--pad` 16, `--fs` 14 | present | **absent** |
#
# The palette disagrees too, and by more than the metrics. Counted rather than
# estimated -- both token blocks parsed and compared this session: of the tokens
# both canvases declare, **5 agree and 12 differ**, with 7 more declared by only
# one side. The three that cannot be reconciled by a value are the hairlines:
# `--hair #232628`, `--div #1e2123` and `--bor #2c3033` are **opaque** where this
# file's `line`/`line_soft`/`border` are `Color(1, 1, 1, .10/.07/.16)`. A white
# alpha composites differently over `--pan` than over `--sur`; an opaque hex does
# not. **No alpha value reproduces them**, so this is a structural change to
# `DARK`, not a re-tune of three numbers. Also differing: `--sur`, `--pan`,
# `--ink`, `--dis`, `--ins`, `--accInk`, `--good`, `--wash2` and `--shadow`; and
# `--map` and `--tst` have no token here at all.
#
# **Not resolved here, and not resolvable by one lane.** Three reasons, stated
# so the next reader does not re-derive them:
#
#   1. It needs an owner ruling on which canvas governs a tablet. `CLAUDE.md`'s
#      "the newer canvas wins" does not decide it -- both were imported from the
#      owner's project on the same day, and they are not two drafts of one
#      drawing but two drawings of different form factors that overlap.
#   2. A third palette is not a `c()` branch. `dcc_shell.gd`'s `rebuild_theme()`
#      remaps live colours through `old_pal = DccTheme.DARK if was_dark else
#      LIGHT`; a tablet palette that `c()` returned but that call did not know
#      about would leave every repainted node reading the desktop values.
#   3. The horizontal `--railH:56` rail is **layout**, not tokens, and is still
#      unbuilt. **The dock half of this item is fixed as of 2026-09-07** and the
#      paragraph that stood here is corrected rather than deleted, because its
#      conclusion was right and its premise has moved. It read: "`DccTheme`
#      itself has no orientation predicate at all -- `is_touch()`,
#      `is_tablet()` and `is_phone()` are the whole vocabulary here -- so a
#      232/320 pair could not be expressed as a `ROLE` row even if it were
#      ruled on; `ROLE` is a two-column table and this is a third column."
#      The last clause still holds and is why `TABLET_PORTRAIT` above is an
#      override layer and not a widened `ROLE`. The first no longer does:
#      `is_tablet_portrait()` below is that predicate, fed by
#      `DccShell._compute_layout_mode()`'s existing `_landscape`, which is the
#      "machinery already built and simply not reached from the tablet
#      composition" this paragraph correctly identified.
#
# **What guards this table:** `_roleresolve_probe.tscn`, added 2026-09-03. Until
# then every property below was carried by this prose alone. It pins the
# predicate (an `is_touch()` "simplification" would hand the phone the tablet
# column — §57's own refutation), the collision counts, the `TABLET`↔`ROLE`
# relationship in **both** directions, and each homed figure against `ENV`'s own
# literals. Run it after touching any row here.
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
	"h_menu_bar": [36, 52],        ## `--menuH` (`ENV:25`, `ENV:1819`).
	## `--tbH`. **The one row here that is not a duplicate of `TABLET`** -- it
	## cannot be, because `TABLET`'s integer key space maps 40 to the rail. This
	## is the authoritative tool-options height and `dcc_shell.gd:1004` reads
	## it directly; see `TABLET`'s header.
	"h_tool_options": [40, 56],
	"h_status": [26, 36],          ## `--sbH`.
	"h_timeline": [70, 88],        ## No prototype counterpart; see `H_TIMELINE`.
	## `--tool` (`ENV:25` / `ENV:1819`) -- the rail-head cell `ENV:284` draws at
	## `height:var(--tool)`, 30 px pointer and 44 px touch.
	##
	## **Was `[29, 34]`, self-labelled "Stale", until 2026-09-03**, and its own
	## cross-reference pointed at a `TABLET` row (`29`) that no longer existed:
	## stage 2's `_build_rail()` rebuild moved the shipped cell to `_scaled(30)`
	## -> `TABLET[30]` = 44 and retired the 29 key, and this row was not moved
	## with it. It had no consumer -- `grep -rn h_rail_head` over the whole
	## project returned this line and nothing else -- so correcting it moves no
	## pixel at either density. What it removes is a resolver that answered 34
	## where the shell and the canvas both say 44, waiting for the first caller
	## to read it.
	"h_rail_head": [30, 44],
	"w_rail": [40, 48],            ## `--railW`.

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
	## **The `3px 11px` this comment used to cite is a PILL, not a button** --
	## measured 2026-09-07: every instance in the PC canvas carries
	## `border-radius:999px` (the `add`/`subtract` mode chips at `ENV:255-256`,
	## the breakpoint targets at `ENV:202`). It was attributed to the action
	## button and is not one.
	##
	## What the canvas actually gives a control of the button's own shape
	## (`border-radius:8px`), as a census rather than a single pick: **8 of 18
	## use `padding:2px 12px`**, then 5 at `2px 9px`, 2 at `3px 10px`, 2 at
	## `0 14px`, 1 at `2px 13px`. `padding:4px 12px` -- the figure the backlog
	## row quoted -- occurs 4 times in the file and on none of them.
	##
	## These values are NOT applied yet and this comment does not claim they
	## are: `action()` resolves 10/4 on pointer and this key is read only on
	## tablet. Moving them is a layout change with a named regression history
	## (the 265 px tool bar, DS-03's eight over-wide minimums), so it needs
	## `_ds03fit_probe` and `_ds03shot_probe` as guards, not a sweep.
	## **The census above asks the wrong question, and this is the answer.**
	## Ruled `2px 12px` on 2026-09-08 and **reversed the same hour** — the
	## reversal is the useful part, so it is recorded rather than tidied away.
	##
	## **The canvas DOES disambiguate the action button**, which the row above
	## says it does not. It tags every radius-8 node with a height role, and
	## that role is the population:
	##
	##   `--btnH` = 28px -- **the action button**, N=14
	##   `--ctl`  = 24px -- the small inline chip, N=15
	##   `--tool` = 30px -- the tool bar, N=1
	##
	## **All eight `2px 12px` nodes are `--ctl` (7) or `--tool` (1). Not one
	## is `--btnH`.** So the ruling took the inline chip's padding and put it
	## in the button's slot. `role_px("btn_pad_x"/"btn_pad_y")` feeds exactly
	## `action()` and `modal_button()`, both `--btnH`-class.
	##
	## **What `--btnH` actually draws**, counted independently twice (the
	## verifier's script and a fourth census over all 888 `style=` attributes,
	## agreeing node for node):
	##
	##   `4px 14px` x3, `4px 15px` x3, `4px 13px` x3, `6px 18px` x2,
	##   `0 14px` x2, `4px 12px` x1
	##
	## **y=4 on 10 of 14; x=14 on 5 of 14**, the only x with a plurality.
	## (The verifier said y=4 on 7 of 14 -- recounted here as 10, since three
	## groups of three plus the single `4px 12px` all carry y=4. The
	## disagreement does not move the conclusion, and the smaller figure is
	## the one that was published, so it is named rather than quietly fixed.)
	##
	## **So `action()`'s live literal already has y right**: it bypasses this
	## row with its own `10`/`4`, and 4 is the canvas figure. **x is the open
	## half** -- 10 against a canvas plurality of 14. Not changed here: the
	## literal lives in `dcc_widgets.gd`, which this pass does not own, and
	## widening x is the change with the named regression history (the 265 px
	## tool bar, DS-03's eight over-wide minimums). **It needs `_ds03fit_probe`
	## and `_ds03shot_probe` around it, not a sweep.**
	##
	## **The lesson, since this row has now carried four wrong figures:** a
	## census is only as good as its population, and "the canvas does not say"
	## was never checked against the canvas's own grouping variable.
	"btn_pad_x": [11, 18],         ## Tablet only; see the census above.
	"btn_pad_y": [3, 9],
	## The action button's corner radius, and **the row that retires §11's
	## radius-0 rule for buttons.** Both figures are canvas literals, measured
	## rather than reasoned:
	##
	## * **8** -- `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html`
	##   writes `border-radius:8px` inline on every action chip it draws
	##   (`background:var(--acc);color:var(--accInk)` on the primary,
	##   `background:var(--ins);color:var(--sec)` on the secondary). 76 of them.
	## * **12** -- `Cartalith Tablet.dc.html` does not use 8 anywhere. Its
	##   buttons are `border-radius:var(--rCtl)`, and that canvas defines
	##   `--rCtl:12px`. Its 2 px radii are all slider *tracks*
	##   (`width:{{ ...Pct }}`), not controls, so they are not this figure.
	##
	## **A phone deliberately gets the desktop 8**, per `role_px()`'s own header:
	## the phone consumes `PHONE_*`, not `ROLE`, and `pill()` already carries the
	## 412 canvas's 24 px fully-rounded target. This row is not that one.
	##
	## §11's "radius 0 everywhere" is cited in `outline()` above and was true of
	## the artboards it was written against. Both current canvases have moved off
	## it -- the PC canvas alone draws 81 `border-radius:999px` pills and 76
	## `8px` corners -- so the shell was faithfully implementing a rule the design
	## had abandoned. Every conformance pass missed it because they ranked
	## palette tokens and dock widths, and a corner radius is neither.
	"btn_radius": [8, 12],

	# — Density-varying widths and the two hero type rungs, all new with the
	#   2026-08-31 re-base. They live here rather than as top-level constants
	#   because each is a *pair* the prototype states in `densStr` (`ENV:1819`),
	#   and because `ROLE` is by its own header a table of tokens rather than of
	#   wired values -- three of the five have no consumer in this shell yet and
	#   say so below.
	#
	#   `w_left_dock` / `w_right_dock` restate `W_LEFT_DOCK` / `W_RIGHT_DOCK` /
	#   `W_DOCK_TABLET` on purpose, the same way the region-box rows restate
	#   `TABLET`: those constants answer a caller that has only a number, this
	#   answers one that knows which dock it is building and can therefore also
	#   pick up the `LAPTOP` override. If one moves the other must move with it.
	"w_left_dock": [372, 400],
	"w_right_dock": [304, 400],
	## `--railExpW`, 200 -> **264**. BUILD_ANSWERS §2.4 records this as one of
	## three values that did not scale to touch and should have -- "they were
	## oversights". No consumer: `dcc_shell.gd::_build_rail()` builds the
	## collapsed rail only, and the expansion column (`ENV:294`) is stage 2.
	"w_rail_expanded": [200, 264],
	## `--popW`, 238 -> **300**. The layers popover (`ENV:902`), which is its
	## one consumer: `layers_popover.gd` reads it for the outer column, the
	## scroll and the `popup()` rect. It sized itself from its content, and
	## from a bare `228`, until 2026-08-31.
	"w_popover": [238, 300],
	## `--pop`, 300 -> **380**, and 280 on the LAPTOP band. The menu dropdown
	## (`ENV:62`). Unconsumed because Godot's `PopupMenu` widths are
	## content-driven; carried so the structural stage can pin them.
	"w_menu_popup": [300, 380],
	## `--hero` / `--hero2` (`ENV:1819`), the two large accent readouts, read by
	## `hero()` and `hero2()` below. `FS_HERO` above is the desktop half of the
	## first and keeps its name; this pair adds the touch halves that §2.4 says
	## were missing. `--hero` is the sample elevation, the measure
	## total and the paint count (`ENV:957`, `:973`, `:1041`); `--hero2` is the
	## planner verdict (`ENV:1144`), which is smaller because it sits inside a
	## dock rather than at the top of one.
	"fs_hero": [26, 30],
	"fs_hero_2": [22, 26],

	## `--tool`, 30 -> **44** (`ENV:25` / `ENV:1819`). The viewport's Layers
	## button (`ENV:900`), sized `width:var(--tool);height:var(--tool)` and read
	## by `viewport_host.gd`'s `_layers_btn` since 2026-09-07.
	##
	## **This row read `[36, 36]` and sat under the "Pinned" heading below,
	## annotated "the layers button, `36×36` in both artboards".** Both halves
	## were false against the canvas the rest of this table is homed to, and the
	## heading made the error look deliberate -- "pinned" asserts that the two
	## columns agreeing is a measured fact, and here it was an artefact of a
	## retired artboard. Nothing consumed it, so nothing contradicted it:
	## `_layers_btn` carried its own `44 if _touch else 26` literal pair, which
	## agreed with neither column nor with `--tool`. `grep -rn 'w_fab' --include=*.gd`
	## over the whole project on 2026-09-07 returned this row, that button and
	## `_roleresolve_probe.gd` -- three lines, no third opinion.
	##
	## `--tool` is the *same* token as `h_rail_head` below is homed to
	## (`ENV:284`'s rail chevron cell), and both now read `[30, 44]`. That is
	## the canvas agreeing with itself, not a duplicate: the two rows answer
	## different callers, exactly as this table's own header describes for the
	## region-box rows.
	"w_fab": [30, 44],
	## The **graphical** scale bar under the viewport's km label, which this
	## port did not draw at all until 2026-09-07 -- `viewport_host.gd` built the
	## text and stopped. Both canvases draw the rule and they draw it
	## differently, so this is a measured pair and not a scaled one:
	##
	##   - `ENV:916`: `width:120px;height:1px;background:var(--sec)`, with two
	##     `width:1px;height:5px` end ticks in the same ink, `bottom:0` inside a
	##     1 px box -- so they rise *above* the rule.
	##   - `TAB:451`: `width:84px;height:4px` drawn as `border-top` +
	##     `border-left` + `border-right`, 1 px, in `var(--dim)` -- the same
	##     figure mirrored, ticks hanging *below*.
	##
	## The phone canvas draws no scale bar at all (`grep` for a `scaleLab` ref
	## over `Cartalith Android.dc.html` returns nothing), which is why
	## `viewport_host.gd` gates the rule on `is_phone()` rather than reading a
	## third column that would have to be invented.
	## **The authority question these two raised is settled.** They were flagged
	## as an inconsistency because this table homes every touch figure to
	## `ENV:1819` — the PC canvas's own touch branch — while the touch column
	## here comes from `TAB:451`, in a file that (at the time) declined the
	## tablet canvas pending an owner ruling.
	##
	## **That ruling came on 2026-09-07: all three canvases are the target, and
	## `TABLET_UI_SPEC.md`'s "adoption has not been decided" was corrected the
	## same evening.** So a tablet figure sourced from the tablet canvas is now
	## the CORRECT authority rather than an exception to one — and it was
	## already disclosed above, with both line citations, which is what the
	## backlog row asked for.
	##
	## `ENV:916` stays the pointer source because it writes `width:120px` as a
	## literal, so the PC canvas draws 120×1 at both densities and reading it
	## for touch would have been the real error.
	"w_scale_bar": [120, 84],
	"h_scale_tick": [5, 4],

	# — Pinned. Listed rather than omitted, because "the canvas draws this the
	#   same at both sizes" is a measured fact and a caller that guesses will
	#   scale it. §11's letter-spacings are pinned too and are not figures this
	#   table carries — `mono()` takes them as an argument.
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
## ─────────────────────────────────────────────────────────────────────────
## **SUPERSESSION NOTICE, 2026-09-07. Every figure below is a true citation to
## a canvas the owner has since replaced. Do not "correct" them one at a time;
## the whole block needs one owner ruling.**
##
## `design/Cartalith Android Phone.dc.html` is what the paragraphs below quote,
## and it says what they say -- checked literal by literal this session, not
## assumed. On 2026-09-07 the owner named a *different* phone canvas,
## `design/mcp-2026-09-07/Cartalith Android.dc.html`, imported live from the
## project the same day ("Implement: Smartphone"). The two draw different
## phones, and these are the differences that reach a constant here:
##
## | figure | 412 canvas (the 2026-08-25 ruling) | `Cartalith Android.dc.html` |
## |---|---|---|
## | status row | `height:28px;padding:0 16px` | `height:max(env(safe-area-inset-top),30px);padding:0 18px` (`AND:78`) |
## | app bar | `height:56px;gap:14px;padding:0 12px;border-bottom:1px rgba(255,255,255,.09)` | **no fixed height**, `gap:8px;padding:4px 10px 0`, no border (`AND:79`) |
## | app-bar buttons | `40x40`, no background, leading ☰ | `44x44;border-radius:22px`, filled, `border:1px solid var(--hair)`, **both trailing**, ⌕ then ⋮ (`AND:87-88`) |
## | bottom nav | `height:64px;background:#131516;border-top .09`, **five** `flex:1` cells, `gap:4px`, caption `9.5px/.1em` | 84 px total, `background:var(--pan2)`, `border-top:var(--hair2)` (.07), **four** cells, `gap:3px`, caption `9.5px/.12em`, pill `padding:4px 16px;border-radius:13px` (`AND:596-604`) |
## | gesture row | `height:20px`, handle `rgba(255,255,255,.22)` | `max(env(safe-area-inset-bottom),18px)`, handle `var(--bord)` = `.16` (`AND:604`) |
## | landscape | **none exists** -- `grep -c "landscape\|portrait\|rotate"` over the 412 canvas is **0** | an explicit branch: 72 px left rail, `--pan2`, 1 px `--hair2` right border, `gap:6`, `padding-top:40`, cells `60x56` r16, glyph 14, caption `8.5px/.08em` (`AND:607-617`) |
## | slider | row `height:32px`, thumb `22x22` r11 | `input[type=range]{height:22px}`, thumb `20x20` r10 (`AND:10`) |
##
## **This is a canvas-supersession question, not a defect list.** A 2026-09-07
## diff pass read six of these constants as "undisclosed departures dressed as
## citations" -- a comment naming a canvas that no longer says what it claims.
## That grading is wrong on the mechanism and it matters, because the two
## conclusions send a lane to opposite places: nothing here drifted from its
## source, and a lane told the comments are stale would rewrite prose that is
## accurate while leaving the values, which are the half that actually moved.
##
## Values are left alone deliberately. Six of them are consumed by
## `dcc_shell.gd`'s phone composition, adopting the new canvas is a layout
## change and not a token change (four cells against five, an app bar with no
## height, a landscape rail that does not exist here), and `CLAUDE.md`'s "the
## newer canvas wins" is a tie-break between *drawings* -- it does not by
## itself overturn a dated owner ruling that named a file. Ask.
## ─────────────────────────────────────────────────────────────────────────
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

## The **fourth density set's** own predicate. See `LAPTOP`'s header for the
## whole argument; the short version is that "narrow" and "touch" are two
## independent questions, and this is the one `_touch` cannot answer.
##
## Published the same way as `_touch` and `_phone_mode`, and for the same
## reason: the widget factories are static and cannot reach `DccShell`.
## `DccShell._compute_layout_mode()` is the single writer.
##
## Defaults to `false`, which is the safe direction: a shell that never calls
## `set_narrow()` gets the base 1920 set, which is what it got before this
## constant existed. The failure mode of the other default would be a 1920
## desktop silently running 330 px docks.
static var _narrow := false

static func set_narrow(v: bool) -> void:
	_narrow = v

## `narrow and not touch` -- the `!touch` in `ENV:1819`'s own expression, which
## keeps a 1366-wide tablet on the touch set rather than handing it dock widths
## sized for a mouse. A phone answers `false` here too (it is `_touch`), and
## would ignore the answer regardless: it consumes `PHONE_*`, not `ROLE`.
static func is_laptop() -> bool:
	return _narrow and not _touch

## The **fifth density set's** state, published the same way as `_touch`,
## `_phone_mode` and `_narrow`, and for the same reason: the widget factories
## are static and cannot reach `DccShell`.
##
## `DccShell._compute_layout_mode()` is the single writer, and it already had
## the answer -- `_landscape = size.x > size.y`, computed on every resize since
## the phone composition needed it. Nothing new measures orientation; this is
## the same fact, published one level down so `role_px()` can see it.
##
## Defaults to `false` = landscape, which is the safe direction twice over: it
## is the orientation that has no defect, and it is what every caller got
## before this variable existed, so a shell that never calls `set_portrait()`
## is byte-identical to the one that shipped.
static var _portrait := false

static func set_portrait(v: bool) -> void:
	_portrait = v

## `tablet and portrait` -- see `TABLET_PORTRAIT`'s header for the shape
## argument and for why only the portrait half of the canvas pair is taken.
##
## It is `is_tablet()` and not `is_touch()` deliberately, and that is the same
## trap `is_tablet()`'s own header documents: a phone is `_touch` too, and a
## portrait phone reaching a 232 px dock would be a `GUI_GAP_REGISTER.md` §57
## repeat. A phone would ignore the answer anyway -- it consumes `PHONE_*`, not
## `ROLE` -- but the predicate should not depend on that being true forever.
static func is_tablet_portrait() -> bool:
	return is_tablet() and _portrait

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
	## `LAPTOP` is consulted first and only ever narrows the *pointer* answer:
	## `is_laptop()` already excludes touch, so this can never shadow a tablet
	## figure. The lookup is `LAPTOP.has()` rather than a parallel three-element
	## array in `ROLE` because the override covers three roles out of forty --
	## see `LAPTOP`'s header for why an override layer and not a fourth column.
	if is_laptop() and LAPTOP.has(role):
		return int(LAPTOP[role])
	## The second override layer, same shape and same reasoning as `LAPTOP`'s.
	## Order between the two is arbitrary and says nothing: `is_laptop()` is
	## `narrow and not touch` and `is_tablet_portrait()` is `tablet and
	## portrait`, so no device can satisfy both.
	if is_tablet_portrait() and TABLET_PORTRAIT.has(role):
		return int(TABLET_PORTRAIT[role])
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
		## **An OPAQUE value never matches a translucent token.** Without this
		## line the RGB-only pass reads pure white as `line` -- `Color(1,1,1,
		## .10)` in `DARK` -- and hands back the LIGHT `line`'s RGB at the
		## original alpha, so an opaque white flips to opaque **black**.
		## Measured by `_cklight_probe`'s `[4]` block, which calls this
		## function and printed `#000000` until this guard landed. The reverse
		## holds on the light palette, where `line` is `Color(0,0,0,.14)` and
		## an opaque black would flip to white.
		##
		## That is not a hypothetical trap: `dcc_shell.gd` carries three
		## separate comments (at `_measure_ink`'s white, the phone scrim, and
		## the export overlay) explaining that a literal white "would break
		## `remap()`" and deriving from a token instead. The workaround was
		## right; the false positive it worked around is fixed here.
		##
		## **The legitimate case is the other direction and it still runs.**
		## The pass exists so a resource entry that used a token's hue at a
		## different opacity follows the palette -- `LineEdit/selection_color`
		## is `accent` at `a=0.35` against an opaque `accent` token, and
		## `dark_theme.tres` holds four such accent derivatives plus three
		## white hairlines at .05/.06/.14/.20 against `line`'s .10. Every one
		## of those has a **translucent value**, so none is touched by this
		## guard; only an opaque value claiming a translucent token is.
		if value.a >= 1.0 and tv.a < 1.0:
			continue
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
## Reversed ink is `c("accent_ink")` -- `c("bg")` until the 2026-08-31 token
## re-base, and the literal `#141617` before that. `pill()` sets only the fill;
## its five callers set the ink, and all five moved together. See
## `accent_ink`'s own comment above for why "dark" was the wrong property to
## reverse onto and "reads on amber" is the right one.
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

## The **desktop and tablet action button**, filled, at the canvas's own
## corner radius. `pill()`'s sibling: same job, other composition.
##
## ### Why this is a new factory and not a change to `outline()`
##
## `outline()` has **30 call sites across nine files** (`grep -rn
## "DccTheme.outline("`), and only four of them are buttons. The rest are path
## wells, gallery tiles, asset cards, a `LineEdit` focus ring, the right dock's
## bar background, folder rows and modal cards. Giving `outline()` a fill and a
## radius to fix the buttons would have repainted every one of them -- rounding
## the dock furniture and filling boxes whose whole purpose is that they are
## hairline-on-nothing. That is a blanket edit this project has had to revert
## before, so the change lives here instead, where only buttons can reach it.
##
## ### What the canvas actually draws
##
## Measured in `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html`, and
## every figure below is a literal from that file:
##
## | | primary | secondary |
## |---|---|---|
## | fill | `var(--acc)` = `accent` | `var(--ins)` = `sunken` |
## | ink | `var(--accInk)` = `accent_ink` | `var(--sec)` = `text_secondary` |
## | border | **none** | **none** (0 of the `--ins` chips carry `border:`) |
## | radius | `8px` | `8px` |
##
## The token mapping is exact in both halves of the palette, checked value for
## value: `--ins` is `#191c1e`/`#eceae4` = `sunken`, `--sec` is
## `#a9adb0`/`#3d3f39` = `text_secondary`. Nothing here needed a new colour.
##
## ### The four states, and where each came from
##
## The shipped button had **two** boxes for four states (`normal`==`disabled`,
## `hover`==`pressed`), which is why a disabled button read as an enabled one.
##
## * **normal** -- measured, the table above.
## * **hover** -- measured. The canvas carries `style-hover` attributes: an
##   `--ins`-filled chip's is `style-hover="color:var(--acc)"`, so a secondary
##   button hovers by moving its *ink* to accent and holding its fill. The
##   primary lifts its fill to `accent_hover`, which is the same move the
##   canvas's own stylesheet makes (`a:hover{color:#f0bd72}`, `--accH`), and it
##   is directional per theme: lighter in dark, darker in light.
## * **pressed** -- **derived**, and said so: the canvas has no pressed state
##   for a filled button. The primary drops back from `accent_hover` to
##   `accent` (a press is always preceded by a hover on a pointer, so the
##   pull-back is the visible event), and the secondary takes
##   `accent_wash_2` -- the canvas's own `--wash2`, its "this control is
##   engaged" ground -- *blended onto* `sunken` rather than used raw, because
##   `--wash2` is 16 % alpha and a translucent box would lose the `--ins`
##   ground over whatever panel the button happens to sit on.
## * **disabled** -- measured, from the canvas's undo/redo pair, which is the
##   one control it draws in both states: `background:var(--ins)` is held and
##   the ink alone drops to `var(--dis)` = `text_ghost`. Extending that to a
##   *primary* disabled button (fill falls to `sunken` too, rather than staying
##   amber) is the derivation, and it is the one that matters: a disabled
##   button keeping a full accent fill would stay the loudest thing on screen.
##   Measured, the drop is real -- `text_ghost` on `sunken` is 2.86:1 dark and
##   2.29:1 light, against 7.58:1 and 8.87:1 for the enabled secondary on the
##   same ground. WCAG exempts inactive components from a contrast floor, so
##   failing AA here is the intent rather than a defect.
##
## Ink is **not** set here, exactly as `pill()` does not set it: Godot takes
## font colours as per-state overrides on the `Button` itself, so the caller
## owns them and the two must be read together. See `DccWidgets.action()`.
static func button_box(primary: bool, state: String, pad_x: int, pad_y: int,
		radius: int = -1) -> StyleBoxFlat:
	var sb := StyleBoxFlat.new()
	if state == "disabled":
		sb.bg_color = c("sunken")
	elif primary:
		sb.bg_color = c("accent_hover") if state == "hover" else c("accent")
	elif state == "pressed":
		sb.bg_color = c("sunken").blend(c("accent_wash_2"))
	else:
		sb.bg_color = c("sunken")
	sb.set_corner_radius_all(role_px("btn_radius") if radius < 0 else radius)
	sb.content_margin_left = pad_x
	sb.content_margin_right = pad_x
	sb.content_margin_top = pad_y
	sb.content_margin_bottom = pad_y
	return sb

## The **input field's** well: a dropdown's closed box, a spin box's number
## field, a search field. Its ground is `button_box(false, ...)` above, and
## that is a fact about the design rather than a shortcut taken here -- the PC
## canvas draws its `<input>` (`ENV:222`, `ENV:498`, `ENV:1101`, `ENV:1118`)
## and its dropdown chip (`ENV:522`) as
##
##   background:var(--ins); border:none; border-radius:8px; outline:none
##
## which is, declaration for declaration, what it writes on a *secondary
## button*. So there is exactly one place the `--ins` chip is defined and this
## is not it. Extending `button_box()` with a `field` flag would have been the
## same four colour lines behind a branch that never diverges; a second copy of
## them would have been worse again, and is how `DccTheme` grows a palette that
## disagrees with itself.
##
## ### What a field has that a button does not
##
## **`focus`.** Both canvases write `outline:none` on every input, which
## suppresses a browser default rather than designing a state, so a keyboard
## user is left with a caret and nothing else. That is survivable in a browser
## and not here: every `Button` this shell builds is `FOCUS_NONE`, so fields
## are the only focusable controls in it. The derivation, said out loud -- hold
## the `--ins` ground and add the shell's own engaged edge, a 1 px `accent`
## border. It costs **no layout**, because `button_box()` sets all four content
## margins explicitly and a `StyleBoxFlat`'s border only grows a control when
## they are left unset.
##
## **`read_only`.** Measured, not derived: the canvas holds the `--ins` ground
## and drops the ink alone, which is the same move `button_box("disabled")`
## already makes, so it maps onto that state instead of getting one of its own.
static func field_box(state: String, pad_x: int, pad_y: int,
		radius: int = -1) -> StyleBoxFlat:
	var ground := "disabled" if state == "read_only" else state
	var sb := button_box(false, ground, pad_x, pad_y, radius)
	if state == "focus":
		sb.border_color = c("accent")
		sb.set_border_width_all(1)
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

## The meta key `DccShell.tablet_fit()`'s fallback walk reads to resolve a
## `Label`'s `ROLE` figure precisely instead of guessing from its font alone --
## every Plex label would otherwise read as `fs_readout`, which is wrong for a
## section header (`fs_dock_header`, a smaller pair). Set only where a factory
## here already knows the role; the walk falls back to a mono/prose guess for
## everything else. Mirrors `DccWidgets.ACTION_META`'s own pattern.
const ROLE_META := "dcc_role"

## §11's section header: uppercase Plex Mono, widely tracked, faint. The `§`
## marker is the disclosure grammar's L3 sigil and is drawn, not implied.
##
## Resolved here, at construction, rather than left to `DccShell.tablet_fit()`'s
## fallback walk -- `right_dock.gd` builds `§ SAMPLE` and every other section
## title through `DccWidgets.section()` -> this function, and that dock is
## never walked (it is not a `register_workspace()` panel). `ROLE_META` is
## still stamped below, for the walk's OWN precision on whatever label it does
## reach, but a caller of `header()` must not depend on being walked at all.
static func header(text: String, sigil: String = "§") -> Label:
	var body := text.to_upper()
	var size := role_px("fs_dock_header") if is_tablet() else FS_HEADER
	var l := mono_label(("%s %s" % [sigil, body]) if sigil != "" else body,
		"text_faint", size, 2, true)
	l.set_meta(ROLE_META, "fs_dock_header")
	return l

## The one large accent number a context is collapsed down to (§6's elevation).
##
## `--hero` through `ROLE`, not the bare `FS_HERO` it hard-coded until
## 2026-08-31: BUILD_ANSWERS §2.4 makes the readout one of "three unscaled
## values -- all three now scale. They were oversights." Nothing moves on a
## pointer, since `role_px()` returns the same 26 there and `LAPTOP` leaves
## `--hero` at base on purpose; a tablet finally gets the 30 the token has
## carried, unread, since it landed.
static func hero(text: String) -> Label:
	return mono_label(text, "accent", role_px("fs_hero"), 0, true)

## `--hero2` (22 / 26): the second, smaller accent readout -- the planner
## verdict (`ENV:1144`), which is a rung down from `hero()` because it sits
## inside a dock rather than at the top of one. Built here rather than left to
## each caller's own literal, for the reason `hero()` above just had to be
## fixed for: a size written at the call site is a size no density table can
## reach.
static func hero2(text: String) -> Label:
	return mono_label(text, "accent", role_px("fs_hero_2"), 0, true)

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

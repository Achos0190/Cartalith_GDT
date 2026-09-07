# TABLET_UI_SPEC.md — the tablet canvas, inventoried against the shipped shell

**Authority:** `design/mcp-2026-09-07/Cartalith Tablet.dc.html` (131 206 bytes,
imported 2026-09-07 by `DesignSync` from the owner's "UI mockups planning"
project; the owner's word for it, per `design/mcp-2026-09-07/CAPTURE.md`:
*"Implement: Tablet"*). It needs `support.js`, `cartalith-dcc-parts.js` and
`landmark-glyphs.js` beside it to render.

**This file is an inventory, not an implementation plan, and no shell code was
written by the pass that produced it.** The tablet canvas describes a
*different shell* from the one this project ships.

> **CORRECTION, 2026-09-07 (evening). This paragraph used to end "and adopting
> it is an owner decision that has not been made", and that was wrong when it
> was written.** The decision had been given the same day, by the instruction
> that delivered this canvas: *"All designs layouts and styles should match
> 100%. Check all designs, pc, tablet and phone."*
>
> **The consequence was measurable and the owner found it: the tablet still
> ships the PC layout.** `ANDROID_UI_SPEC.md` drove a full day of phone work;
> this file drove none, because it declared itself inventory-only and parked
> adoption behind a question nobody had been asked.
>
> **What the shell implements instead is ruling DS-03** (2026-09-03, *"keep
> everything; reflow only"*) — a **content** answer about which controls leave,
> made four days before this canvas existed. Read as settling the tablet's
> *shape*, it yields the PC composition, reflowed.
>
> **Two items in §4.2 remain genuinely not adoptable** — per-stage recompute and
> the canvas's ten stage names — so "100% on tablet" has named exceptions that
> do need a ruling. **That is a much smaller question than the one this
> paragraph asked, and it should have been put to the owner on the day.** This file exists so that decision can
be made from measured facts. Where it says "cost", that is what a change would
touch — not a recommendation that it be spent.

**Method, stated so the next reader can check it (`MISTAKES.md`, "Enumerate a
surface for an audit"):** the canvas template was walked top to bottom, `sc-if`
block by `sc-if` block — 49 of them, beside 49 `sc-for` loops. Then each
block's engine counterpart was looked for **in the code** by symbol; only then
was the shipped side's geometry measured. Every number attributed to the canvas
is quoted from the canvas's own source. Every number attributed to this build
was either read off a named
constant or produced by `_tabspec_probe.tscn`, which this pass added and which
§0.5 describes.

**Grep the symbol, never seek an offset.** Line numbers below are for
orientation only, in a file of 1 344 lines that will drift if it is re-vendored.

---

## 0. How to read the canvas

One interactive prototype, not a set of artboards: the `<x-dc>` template runs
lines 9–656 and `class Component extends DCLogic` runs 658–1344. Three
top-level screens, selected by `s.scr`:

| `s.scr` | `sc-if` | What it is |
|---|---|---|
| `picker` | `scrPicker` (line 29) | "Open a world" grid |
| `app` | `scrApp` (line 56) | the shell |
| `planner` | `scrPlanner` (line 555) | **a third top-level screen** — the Journey Planner |

### 0.1 Palette — the load-bearing table

Both palettes are one CSS string in `valsShell()` (line ~897), a ternary on
`s.light`. **19 colour variables, plus `--shadow`.** Re-derived for this file by
parsing both blocks and `dcc_theme.gd`'s `DARK` / `LIGHT` dictionaries in the
same pass — not copied from the note already in `dcc_theme.gd`.

#### DARK — tablet canvas against `DccTheme.DARK`

| Canvas var | Canvas value | `DccTheme` token | Token value | Verdict |
|---|---|---|---|---|
| `--body` | `#c8cbcd` | `text` | `#c8cbcd` | **exact** |
| `--sec` | `#a9adb0` | `text_secondary` | `#a9adb0` | **exact** |
| `--dim` | `#8d9296` | `text_dim` | `#8d9296` | **exact** |
| `--faint` | `#6f7478` | `text_faint` | `#6f7478` | **exact** |
| `--acc` | `#e0a34a` | `accent` | `#e0a34a` | **exact** |
| `--sur` | `#0f1011` | `bg` | `#0d0e0f` | differs |
| `--pan` | `#141618` | `panel` | `#121314` | differs |
| `--ink` | `#f0f2f3` | `text_bright` | `#e8ebec` | differs |
| `--dis` | `#565b5f` | `text_ghost` | `#5f6468` | differs |
| `--ins` | `#1c1f21` | `sunken` | `#191c1e` | differs |
| `--accInk` | `#12140f` | `accent_ink` | `#141005` | differs |
| `--warn` | `#e0a34a` | `warn` | `#e0a840` | differs (5 blue) |
| `--good` | `#7ea86a` | `good` | `#6fae7d` | differs |
| `--hair` | `#232628` | `line` | `rgba(255,255,255,.10)` | **differs structurally** |
| `--div` | `#1e2123` | `line_soft` | `rgba(255,255,255,.07)` | **differs structurally** |
| `--bor` | `#2c3033` | `border` | `rgba(255,255,255,.16)` | **differs structurally** |
| `--map` | `#0c0d0e` | — | — | no token |
| `--wash2` | `rgba(224,163,74,.15)` | — | `accent_wash` is `--wash`, `.09` | no token |
| `--tst` | `#1c1f21` | — | — | no token |
| `--shadow` | `0 14px 40px rgba(0,0,0,.5)` | — | not a colour token | no token |

**16 shared, 5 exact, 11 differ, 3 canvas vars with no token at all.**

#### LIGHT — tablet canvas against `DccTheme.LIGHT`

**16 shared, 2 exact (`--acc` `#a4650f`, `--warn` `#9a6a12`), 14 differ.** The
light palette is not a re-tune of the shipped one; it is a warmer paper. Six
representative rows:

| Canvas var | Canvas | `DccTheme.LIGHT` |
|---|---|---|
| `--sur` | `#efece6` | `bg` `#f4f2ee` |
| `--pan` | `#f6f4f0` | `panel` `#fbfaf7` |
| `--body` | `#2b2d2a` | `text` `#23241f` |
| `--hair` | `#d9d5cc` | `line` `rgba(0,0,0,.14)` |
| `--bor` | `#cdc8bd` | `border` `rgba(0,0,0,.20)` |
| `--accInk` | `#ffffff` | `accent_ink` `#f7f4ee` |

#### Do I agree with `dcc_theme.gd`'s existing count? Yes, with one wording fix.

`dcc_theme.gd`'s palette header (the block above `const ROLE`) says: *"of the
tokens both canvases declare, **5 agree and 12 differ**, with **7 more declared
by only one side**"*. That comparison is **tablet canvas against the PC canvas
(`ENV`)**, not against `DccTheme`, and re-run independently here it reproduces
exactly:

* shared by both canvases: `sur pan ins ink body sec dim faint dis acc accInk
  hair div bor wash2 good` (16) **plus `--shadow`** = 17;
* agree: `body sec dim faint acc` = **5**;
* differ: the other 11 **plus `--shadow`** (`0 14px 34px rgba(0,0,0,.55)` →
  `0 14px 40px rgba(0,0,0,.5)`) = **12**;
* declared by one side only: `ENV` has `accH wash block water` (4), the tablet
  has `map tst warn` (3) = **7**.

**The fix:** eleven of the twelve are colours; the twelfth is a box-shadow
string. Read as "12 palette *colours* differ" the figure is one too many —
which is the reading my own brief for this lane carried ("12 of 19 palette
tokens different"). Against `DccTheme` rather than against `ENV` the colour
count is **11 of 16 shared**, because `--wash2` loses its partner (`DccTheme`'s
`accent_wash` is `--wash`, not `--wash2`) and `--warn` gains one.

`dcc_theme.gd`'s three other claims in that block hold as written:

* **The hairlines cannot be reconciled by a value.** Confirmed by composition
  rather than by inspection: `DccTheme`'s `line` is white at α .10, which
  resolves to `#2a2b2c` over `panel` and `#252627` over `bg`. The canvas's
  `--hair #232628` is one colour everywhere. **No alpha reproduces both**, so
  this is a structural change to `DARK`, not three re-tuned numbers.
* **`--map` and `--tst` have no token here.** Confirmed. `--wash2` has none
  either, which that block does not say.
* **A third palette is not a `c()` branch**, because `dcc_shell.gd`'s
  `rebuild_theme()` remaps live colours through `DccTheme.DARK if was_dark else
  LIGHT`. Confirmed at the symbol.

#### Two contrast consequences of adoption, computed rather than asserted

Re-basing a palette silently moves every relationship built on it
(`MISTAKES.md`, "Re-base a shared token / constant set"). Two pairs move in a
direction worth knowing about; WCAG 2.x ratios:

| Pair | Shipped | Tablet canvas |
|---|---|---|
| ghost/disabled ink on panel | `#5f6468` on `#121314` = **3.11:1** | `#565b5f` on `#141618` = **2.64:1** |
| light body ink on panel | `#23241f` on `#fbfaf7` = **14.97:1** | `#2b2d2a` on `#f6f4f0` = **12.65:1** |

The first crosses below 3:1. Everything else measured moves by less than 0.5
(dark body 11.41 → 11.13, dark accent on ground 8.75 → 8.62, dark reversed ink
on accent 8.60 → 8.39); light reversed ink on accent *improves*, 4.30 → 4.72.

### 0.2 Frames, orientation and zoom

`FRAMES()` (line ~700), six of them; the canvas's default is `t1600`:

| id | label | w × h | orientation |
|---|---|---|---|
| `t1280` | `10″ 1280×800` | 1280 × 800 | landscape |
| `t1440` | `11″ 1440×900` | 1440 × 900 | landscape |
| `t1600` | `13″ 1600×1000` | 1600 × 1000 | landscape (default) |
| `t800` | `10″ ▯ 800×1280` | 800 × 1280 | **portrait** |
| `t900` | `11″ ▯ 900×1440` | 900 × 1440 | **portrait** |
| `t2560` | `2560×1600` | 2560 × 1600 | landscape |

`isPortrait(){return f.h>f.w}` — aspect, nothing else. That is the same
predicate `DccShell._compute_layout_mode()` already uses for `_landscape`
(`_landscape = size.x > size.y`), so the *test* exists; §5 covers what does not.

`_zoom()` is `min(1, max(560, innerWidth−40)/fw, max(520, innerHeight−150)/fh)`.
**`CAPTURE.md` quotes this without the two `max()` floors.** Harmless for
capture (a floor only ever raises a numerator, so the true zoom is never
*smaller* than the quoted form), but the recipe as written computes a different
value on a small window. Recorded, not fixed: `CAPTURE.md` is not this lane's
file.

### 0.3 Metric tokens — `themeVars`, one string, line 914

```
--menuH:52px --railH:56px --railW:52px --sbH:28px
--tap:44px --ctl:36px --row:48px --rowD:44px
--rCtl:12px --rPan:16px
```

plus a portrait/landscape density pair:

| | portrait | landscape |
|---|---|---|
| `--ldW` / `--rdW` | **232px / 232px** | **320px / 320px** |
| `--m0` / `--m1` / `--m2` | 12 / 11 / 9.5 px | 12.5 / 11.5 / 10 px |

`--m1` is the frame's base font (`font:var(--m1)` on the shell root).

**Against `DccTheme`'s tablet column, live** — read by `_tabspec_probe`, not
copied out of the table:

| Canvas token | Canvas | `DccTheme.role_px` (tablet) | Measured, laid out |
|---|---|---|---|
| `--menuH` | 52 | `h_menu_bar` **52** | `menu_bar_row` 51 px |
| `--railH` | 56 | *(no role — see §2.2)* | `tool_options_row` 55 px |
| `--railW` | 52 | `w_rail` **48** | `rail_column` 47 px |
| `--sbH` | 28 | `h_status` **36** | `status_row` 35 px |
| `--ldW` | 232 ▯ / 320 ▭ | `w_left_dock` **232** in portrait | `left_dock` **331 px** — the role resolves to 232 and a CHILD forces 331; see the correction below |
| `--rdW` | 232 ▯ / 320 ▭ | `w_right_dock` **232** in portrait | `right_dock` **232 px** — canvas-exact |

> **CORRECTED 2026-09-07 (evening). These two rows read "400 px, every frame"**
> and that is not what the build does. `_tabspec_probe` at `--force-touch --vp
> 800x1280` reports `w_left_dock=232  w_right_dock=232`, `right_dock w=232.0`
> and `left_dock w=331.0` with `min.x=331.0`.
>
> **So the portrait machinery is not missing — it works.** `TABLET_PORTRAIT`
> and `DccShell.set_portrait(not _landscape)` deliver the canvas figure, the
> right dock is canvas-exact, and only the left dock is over — by 99 px, forced
> by a child rather than by its role.
>
> **That matters for scoping the tablet work:** the dock split does not need to
> be built. The role, the cached width and the dock's own floor all read 232
> (`[floor] _left_width=232.0 role_left=232 ld_cms=232.0`), so the 331 is
> `get_minimum_size()` — the panel's CONTENT — and one contributor has to be
> found. **99 of the 285 px portrait overflow is this one dock.**
>
> **Two diagnoses were filed and retracted before this one, both by me, both
> within an hour**: that the role was not applied (it is), and that hidden
> workspace panels were sizing it (they are `own_vis=false` and cannot). The
> second came from a probe printing `is_visible_in_tree()` where the container
> minimum uses the child's own `visible` — the mirror of the planner defect.
> `_tabspec_probe` now prints both flags.
| `--tap` | 44 | `_ttap()` floors to `max(44, round(px × TOUCH_SCALE))`, `TOUCH_SCALE = 1.53` | — |
| `--ctl` | 36 | `ROLE`'s control rung, 24 → 36 | — |
| `--row` / `--rowD` | 48 / 44 | `ROLE`'s row, 28 → **44** (one figure, not two) | — |
| `--rCtl` / `--rPan` | 12 / 16 | no radius tokens exist | — |

Only `--menuH` agrees. `--tbH 56`, `--btnH 44`, `--tool 44`, `--pad 16` and
`--fs 14` — five tokens the PC canvas's touch branch declares — are **absent**
from the tablet canvas.

**A correction to the record on `tablet_fit()`.** The brief this lane ran from
said it *"has ONE call site (`dcc_shell.gd`, tool_options_row); 21 of 22 grep
hits are prose in comments."* It has **two**, and the second is the more
consequential of the pair:

* `tablet_fit(tool_options_row)` inside the `DccTheme.is_tablet()` branch of the
  tool-options rebuild, which its own comment explains — the row is rebuilt on
  every tool-mode switch, outside the one-time walk, and `tool_bar.gd`'s
  `_tool_segment()` sets a raw `custom_minimum_size.y` after the factory has
  already sized the button.
* `call_deferred("tablet_fit", panel)` at the end of `register_workspace()` —
  deferred rather than synchronous because `ws.setup()` has not filled the panel
  yet when `register_workspace()` returns. **This is the one that floors every
  dock control in every workspace.**

So the 44 dp floor reaches the docks through `register_workspace()`, not only
through the tool bar. Anything that re-homes or re-widths a dock passes through
that call.

**On radii:** `dcc_theme.gd`'s `ROLE` header (grep `--rCtl`) files
`--rCtl`/`--rPan` as "against §11's radius-0 rule". That rule is about an
*older desktop artboard*. Counted this
pass, the current PC canvas draws **81 `border-radius:999px` pills and 76 `8px`
corners**; the tablet canvas draws 32 `var(--rCtl)`, 11 pills and 14 circles.
Both current canvases have moved off radius-0. The tablet is not the outlier —
it is 12 where the PC is 8.

### 0.4 Composition — the region stack

```
scrApp
├─ menu bar          --menuH 52   ☰ overflow · File · World · Data · ⟨spacer⟩ · project · meta · ☀
├─ horizontal rail   --railH 56   [tool icons] │ [tool options, scrolls] │ [↶ ↷]
├─ body (flex:1)
│  ├─ domain rail    --railW 52   WOR / CIV / CAR, vertical foot label
│  ├─ left dock      --ldW        breadcrumb row + scroller
│  ├─ viewport       flex:1       canvas + scale bar + armed chip + toast
│  └─ right dock     --rdW        title row + scroller (SAMPLE or MEASURE)
├─ timeline          --sbH 28 collapsed / 118 px open
└─ status bar        --sbH 28     left · centre · right
```

**Both rails exist in both places.** The canvas has a horizontal rail *and* a
vertical domain rail; so does this shell (`tool_options_row` and
`rail_column`). What differs is **what the horizontal one carries** — §2.2.

### 0.5 The probe this pass added

`cartalith-native/godot-project/_tabspec_probe.tscn` / `.gd`. Boots
`shell/app.tscn` inside a `SubViewport`, so the frames larger than this
machine's 1680×1050 screen are reachable:

```
godot --path . _tabspec_probe.tscn -- --force-touch --vp 1600x1000 --tag land
godot --path . _tabspec_probe.tscn -- --force-touch --vp 800x1280  --tag port
```

It reports each region's laid-out size against the canvas's figure for the same
region, the live `role_px` answers, the menu-bar titles, and the fit arithmetic
of §5. It reads; it writes nothing and changes no behaviour.

`--force-touch` is required. `_is_tablet_sized()` returns false off a real
device, so aspect alone would put every one of these frames on the desktop
composition.

Six frames were measured, not one (`MISTAKES.md`, "Report a layout
measurement"): 1280×800, 1440×900, 1600×1000, 2560×1600, 800×1280, 900×1440.

---

## 1. `scrPicker` — "Open a world"

Template lines 29–54.

| Element | Canvas spec | Exists today |
|---|---|---|
| page | `padding:34px 40px`, column, `overflow-y:auto` | — |
| wordmark | `CARTALITH`, `500 var(--m2)`, `letter-spacing:.3em`, `var(--acc)` | `open_project_dialog.gd` draws a welcome, not a screen |
| title | `Open a world`, `400 26px`, `var(--ink)` | " |
| subtitle | `pickerSub` = `'6 tablet frames · ' + f.label + ' · ' + (port ? 'portrait, docks 232 dp' : 'landscape, docks 320 dp')` | prototype-only string — it names the dock rule out loud |
| grid | `repeat(pickCols,1fr)`, `gap:14px`; `pickCols = port ? 2 : 3` | — |
| NEW WORLD card | `1px dashed var(--bor)`, `radius:16px`, `min-height:186px`, `+` at `300 30px`, caption `NEW WORLD`, sub `seed · size · plates` | `menus.gd` File ▸ `New world…` (Ctrl+N) → `new_world_dialog.gd` |
| world card | `1px solid var(--hair)`, `radius:16px`, `min-height:186px`; 96 px gradient thumb; state pill `9px/.14em` on `rgba(0,0,0,.42)`; name `500 var(--m0)/.1em`; meta; status | `open_project_dialog.gd::open_welcome()` lists real projects |
| card data | 5 hard-coded worlds (`ELDRA`, `KESSA`, `VHAL SERAI`, `THORNWOOD`, `HIGH SADDLE`), meta `seed · N² · edited … · NNN MB`, state `LIVE`/`DRAFT`/`ARCHIVE` | the project list is real; **`LIVE`/`DRAFT`/`ARCHIVE` has no engine counterpart** |

**Engine cannot back — the three-state badge.** There is no project-state
enumeration in `engine_bridge.gd` or `open_project_dialog.gd`. The nearest live
signal is the stale model (`world_workspace.gd`'s `_stale_from_stage`), which is
per-*stage* inside an open world, not per-world on disk; the card's
`resolved · stage 10` / `draft · stage 03` / `stale from 07` line has the same
shape and the same problem. If this screen is built, the badge is a new
persisted field, not a read.

**What already exists and where:** the launcher is a `Window`
(`open_project_dialog.gd`), and last batch's capture index records the same
thing from the other side — *"design only — tablet gets the Open-project
dialog, not a launcher screen"*. Converting it to a screen is presentation only;
the card grid, `pickCols` and the thumbnails are new.

---

## 2. `scrApp` — the shell

### 2.1 Menu bar — `--menuH:52`

Template lines 56–79. Row: `gap:4px`, `padding:0 10px 0 6px`,
`border-bottom:1px solid var(--hair)`, `background:var(--pan)`, `z-index:60`.

| Element | Canvas spec |
|---|---|
| overflow ☰ | `var(--tap)` square (44), `radius:var(--rCtl)`, `font-size:15px`, `data-id="OVERFLOW"` |
| menu title | `min-height:var(--tap)`, `padding:0 14px`, `radius:var(--rCtl)`, `var(--m1)`, `letter-spacing:.1em`; open = `background:var(--ins)`, ink `var(--ink)` |
| menus | **exactly three**: `['File','World','Data']` (line ~930) |
| project name | `var(--m1)`, `letter-spacing:.16em`, `var(--sec)` |
| project meta | `var(--m2)`, `var(--dis)`; `'4096² · seed 118402'` |
| theme toggle | `var(--tap)` square, glyph `☀` / `☾` |
| popup | `top:calc(100% + 4px)`, `left:menuX`, `min-width:300px`, `max-width:390px`, `max-height:560px`, `radius:var(--rPan)`, `1px solid var(--bor)`, `box-shadow:var(--shadow)`, `padding:7px 0` |
| `menuX` | `OVERFLOW→6`, `File→56`, `World→132`, `Data→216` — **hardcoded**, not measured off the button |
| popup row | `min-height:var(--tap)`, `padding:0 16px 0 {ind}px` (`ind` 16, or **36 for a submenu child**); label `flex:1`, shortcut `var(--m2) var(--dis)`, trailing glyph `var(--faint)`; danger ink `#c05a4a` |
| separator | `height:1px`, `background:var(--div)`, `margin:5px 0` |
| section head | `padding:7px 16px 3px`, `var(--m2)`, `letter-spacing:.2em`, `var(--dis)` |

**Shipped: seven menus, measured.** `_tabspec_probe` reads `menu_bar_row`'s
children as `File, Edit, Assets, Data, Preferences, Window, Help` — seven
`MenuButton`s, not a `MenuBar`, built by `menus.gd`'s seven `shell.add_menu()`
calls (grep `shell.add_menu(`).

`_menuRows()` (line 984) in full, and where each row lives today:

| Canvas menu | Canvas row | Shipped |
|---|---|---|
| File | `New world…` | File ▸ `New world…` (Ctrl+N) |
| | `Open…` | File ▸ `Open project…` (Ctrl+O) |
| | `Recent ▸` (4 names, `ind:36`) | File ▸ Recent submenu |
| | `Save` `⌘S` | File ▸ `Save project` (Ctrl+S) |
| | `Save as…` | File ▸ `Save as…` |
| | `Revert to saved` | File ▸ `Revert to last save` |
| | `Export map…` `⇧⌘E` | **no menu row of that name.** The export surface is `EXPORT_SCOPE.md`, shelved by the owner 2026-08-25 |
| | `Export data…` | Data ▸ `World data tables…` is the nearest; a plain "export data" row does not exist |
| | `Close world` (danger) | File ▸ `Close project` (Ctrl+W) |
| World | `Run pipeline` `⌘R` | `world_workspace.gd::_regenerate_live()`, on the RUN button — **not a menu row** |
| | `Run from stale` | the same button relabels; there is no separate command |
| | `Reset to seed…` (danger) | Edit ▸ `Reset generation parameters` is the nearest |
| | `Generation pipeline` / `Sculpt` (radio) | `DccShell.select_domain_mode("world", …)` — real, reached from the dock's mode pill |
| | `Seed & size…` | `new_world_dialog.gd::request()`; creation-time |
| | `Plate settings…` | `tect.plates` and siblings are live parameters in the WORLD dock |
| Data | `Travel library…` `⇧L` | Data ▸ `Travel library…` (Shift+L) — **exact, accelerator included** |
| | `Data manager…` | Data ▸ `⧉ Data manager` |
| | `Export CSV…` / `Export GeoJSON…` | `journey_planner_view.gd::_export_stage_table()` exports a table; no menu rows by these names |
| | `Journey planner` | exists as a **tool takeover**, not a menu row — §3 |
| OVERFLOW | `Undo` `⌘Z` / `Redo` `⇧⌘Z` | Edit ▸ Undo / Redo, same accelerators |
| | `Undo history — N steps` | Edit ▸ `Undo history…` |
| | `Asset library…` | Assets ▸ `⧉ Asset library` (Shift+A) |
| | `Landmark types ▸` (family rows, `armed · placed`) | Assets ▸ `Landmark types` submenu (grep `add_submenu_item("Landmark types"`) — **same shape, same counts** |
| | `Toggle theme` | light/dark exists; `DccShell.rebuild_theme()` |
| | `Units — kilometres/miles` | `dcc_units.gd`; Preferences carries units |
| | `Stylus pressure — on/off` | **no counterpart** — `grep "pressure" menus.gd` returns nothing |
| | `Palm rejection — on` | **no counterpart**; the canvas's own row is a toast, not a setting |
| | `Shortcuts…` | Help ▸ `Keyboard shortcuts…`, and Preferences ▸ the same |
| | `About Cartalith` | Help ▸ `About` |

**Cost of the 7 → 3 + overflow collapse.** `menus.gd` is the single builder, and
`command_index.gd` builds the searchable index by **walking the live menu bar**.
`MISTAKES.md`'s "Move a command off the menu bar" is exactly this hazard: a
command that stops being a menu row stops being findable, silently, with nothing
failing. Any re-homing needs `command_index.gd`'s `EXTRAS` and
`shortcuts_dialog.gd`'s `UNLISTED` rows written **in the same change**.

### 2.2 Horizontal rail — `--railH:56`

Template lines 81–128. `gap:8px`, `padding:0 8px`, `background:var(--pan)`,
`border-bottom:1px solid var(--hair)`, `z-index:50`. Three zones:

1. **Tool picker**, `flex:none`, `gap:3px`: one `var(--tap)` square per tool,
   `radius:var(--rCtl)`, a 15 × 15 stroked SVG at `stroke-width:1.25`; selected
   = `background:var(--acc)` with ink `var(--accInk)`. `_toolsFor(domain)`
   (line 799):
   * `WORLD` — Inspect, Pan, Measure, Sculpt, Biome paint (**5**)
   * `CIVIL` — Inspect, Pan, Measure, Settlement, POI, Territory, Way (**7**)
   * `CARTO` — Inspect, Pan, Measure, Label, Icon (**5**)
2. **Options**, `flex:1;min-width:0;overflow-x:auto` — a horizontally scrolling
   strip in four mutually exclusive forms.
3. **Undo / redo**, `flex:none`: two `var(--tap)` squares, `↶ ↷`, ink
   `var(--sec)` when the stack is non-empty and `var(--dis)` when it is empty.

Option forms, with every number the canvas states:

| Form | Contents |
|---|---|
| `optBrush` (`sculpt` or `biome`) | `size` slider **120 px** wide in a `var(--tap)`-tall hit area; track `height:4px`, radius 2, `background:var(--ins)`, fill `var(--acc)`, thumb **14 × 14** circle in `var(--ink)`; readout `width:80px`. Then `hardness`: slider **84 px**, readout `width:34px`. Then a rule, then 8 shape chips (`round soft flat ridge noise square fan chisel`) at `min-height:var(--ctl)` = 36, `padding:0 11px`, `radius:999px` |
| `optFeat` (`sculpt` only) | a rule, then 6 feature chips — Cliff, Ridge, Valley, Plateau, Basin, Dune — at `padding:0 12px` |
| `optMeas` | 5 chips — Distance, Bearing, Area, Radius, Cross-section — at `padding:0 13px`; when Cross-section is live: a rule, a `field` label, and 5 field chips (Elevation, Terrain, Climate, Hydrology, Geology) at `padding:0 11px`; then `measHint` in `var(--dim)` |
| `optPlain` | one hint string in `var(--dis)`. `inspect` → `'tap samples · drag pans · two fingers or wheel zooms · long-press = right-click'`; `pan` → `'drag pans · pinch zooms'`; anything else → `'tap the map to place · hold a dock chip and tap to split-tap place'` |

Brush values: `size` 6–200 quantised to even numbers
(`Math.round((6+p*194)/2)*2`), displayed as `NN px · <km>`; `hard` 0–1 to two
decimals.

**Exists today, and where.** `tool_options_row` is the shipped counterpart, and
it measures **55 px** at every tablet frame. `tool_bar.gd`'s own header — the
"Where the canvas's two bars land in this shell" block — records that the canvas
`DccToolBar` was built from draws *two* bars where this shell has one, so it
puts a two-row `VBoxContainer` inside the single bar: row 1 the tools, row 2 the
options. **So the horizontal rail is not new:** the shipped one is
1 px shorter than `--railH` and carries both of the canvas's zones.

Three real differences, in decreasing size:

* **Tool set.** Shipped `MODES` is `["sculpt", "paint", "measure"]` plus a
  globally-registered `inspect`; `pan` is a viewport gesture, not a registered
  tool. The canvas's CIVIL set (Settlement, POI, Territory, Way) exists in this
  shell as **left-dock categories**, not rail tools —
  `civilization_workspace.gd` builds 15 of them (`Routes & ways`, `Travel`,
  `Trade`, `Civilizations`, `Factions`, `Territories`, `Settlements`, `Economy`,
  `Culture`, `Religion`, `Politics`, `Simulation`, `Landmarks`, `Military`,
  `Relationships`).
* **Measure.** Every one of the canvas's five sub-modes is live:
  `global_tools.gd::MEASURE_GROUP_POINT = ["distance","bearing","area","radius"]`
  plus `"section"`, with `SECTION_CHANNELS` carrying the five fields.
  `tool_bar.gd::_build_measure_tools()` already draws them as one chip row with
  a `CROSS-SECTION` group — **the same composition the canvas draws** — and the
  disabled states are already reasoned against `bridge.measure_api`.
* **Undo/redo in the rail.** Shipped, these are Edit-menu rows with
  accelerators; the rail has no pair of buttons. This is the cheapest single
  item in the whole canvas: two buttons over commands that already exist.

### 2.3 Domain rail — `--railW:52`

Template lines 130–138. `border-right:1px solid var(--hair)`,
`background:var(--pan)`, `padding:7px 0`, `gap:4px`. Each domain cell is
**38 × 44**, `radius:var(--rCtl)`, a 15 px SVG over an **8 px** `.08em`
three-letter caption (`WOR`, `CIV`, `CAR`); selected = `background:var(--wash2)`
with ink `var(--acc)`, otherwise ink `var(--dim)`. Then `flex:1`, then a
`writing-mode:vertical-rl` foot label in `var(--dis)`:

```
railFoot = WORLD ? (worldMode==='sculpt' ? 'SCULPT' : '10 / 10')
         : CIVIL ? civCat.toUpperCase() : cartoCat.toUpperCase()
```

**Exists today:** `rail_column`, measured **47 px** at every frame (`w_rail`
role 48). `_rail_foot_stack` is the foot slot and is already a rotated label.
The three domains are the same three. The cell is a different shape (38 × 44
with a caption under a glyph), and the selected state differs by wash alpha —
`--wash2` at .15 against `accent_wash` at .09.

### 2.4 Left dock — `--ldW`, four bodies

A breadcrumb row (`min-height:var(--tap)`, `padding:5px 10px`, `flex-wrap`,
crumbs at `var(--m2)/.14em` with the last in `var(--acc)`, `›` separators in
`var(--dis)`) over `flex:1;overflow-y:auto` holding one of four bodies.

**(a) `ldPipe` — WORLD ▸ PIPELINE** (lines 146–188). Ten accordion rows from
`STG()`: `Plates & tectonics`, `Uplift & relief`, `Erosion`, `Sea level`,
`Rivers & basins`, `Prevailing wind`, `Ocean currents`, `Climate`, `Soils`,
`Ecology & biomes`. Row: `min-height:var(--row)` = 48, `padding:6px 12px`,
`gap:9px` — a 20 px number in `var(--m2) var(--dis)`, a **7 px** dot, the name
(`flex:1`, ellipsised), a state word (`resolved` / `running` / `stale`), and a
`▸` rotated 90° when open. Dot: `var(--good)` resolved, `var(--acc)` running,
`var(--dis)` stale. Open body: `padding:4px 12px 12px`, a needs note in
`var(--m2)/1.6 var(--dis)`, a **2-column grid** of `min-height:var(--rowD)` = 44
cells, sliders, then `Run stage NN` (`var(--acc)` on `var(--accInk)`) beside
`reset`. Footer: `Run all 10 stages` with an inline progress fill
(`rgba(0,0,0,.2)`), and a note.

**This is the single largest divergence from the engine in the whole canvas.**
The canvas's stage list is not this engine's. `world_workspace.gd::STAGES` and
`cartalith-engine/src/progress.rs::STAGE_NAMES` are `Planet`, `Extent & scale`,
`World structure`, `Tectonics`, `Volcanism & impacts`, `Erosion`, `Hydrology`,
`Climate`, `Ecology & biomes`, `Resources & soils` — ten, but a different ten,
and `_assert_stage_names()` guards that pairing. Three names are shared
(`Erosion`, `Climate`, `Ecology & biomes`), three are the same subject renamed
(`Plates & tectonics`→`Tectonics`, `Rivers & basins`→`Hydrology`,
`Soils`→`Resources & soils`), and **four have no stage at all** — `Uplift &
relief`, `Sea level`, `Prevailing wind`, `Ocean currents`. In the other
direction, `Planet`, `Extent & scale`, `World structure` and `Volcanism &
impacts` are engine stages the canvas does not draw.

More importantly, **`Run stage NN` cannot be backed.** `ANDROID_UI_SPEC.md` §1.3
already records it and the shipped WORLD dock says it in prose: *"there is no
partial recompute in this engine or in the app it ports"*. One `generate()`
resolves all ten. A per-stage Run button is a claim this engine cannot honour.

The one-slider-per-stage model (`stageFields`, `SF`) is likewise a *reduction*:
the shipped WORLD dock draws the engine's own 92 parameters in 9 groups
(measured live last batch by `_genphone_probe.gd`).

**(b) `ldSculpt` — WORLD ▸ SCULPT** (lines 189–244). Root form: a
`FEATURE · PARAMETERS` title and two to three sliders per feature, from `FP` —
`cliff` (height 0–1200 m, run 0–400 px, plus a **left/right side** pair),
`ridge` (height, length), `valley` (depth, width), `plateau` (height, radius),
`basin` (depth 0–900 m, falloff 0–1), `dune` (height 0–120 m, spacing 0–200 px).
Then a stamp list, or an empty-state note. A drill state (`scDrill`) edits one
stamp.

**Exists today:** the mode is real (`DccShell.select_domain_mode("world", …)`),
and `world_workspace.gd` builds `_sculpt_body` from
`bridge.get_sculpt_features()` / `get_sculpt_presets()` / `sculpt_get_globals()`.
The engine's own freehand list is `raise, lower, smooth, cliff, ridge, canyon,
mesa, volcano` (`tool_bar.gd`'s header, read against the registry) — so `cliff`
and `ridge` land directly, and `valley` / `plateau` / `basin` / `dune` are
canvas names whose mapping has to be *stated* rather than assumed.

**(c) `ldCivCats` — CIVIL** (lines 245–353). **Four** collapsible categories:
`LANDMARKS`, `FACTIONS & SETTLEMENTS`, `WAYS & ROUTES`, `JOURNEY PLANNER`.
LANDMARKS has two levels — a family list (`LMFAMS()`, glyph plus
`N of M armed · P placed`), a crowding slider (0.25–2.0, displayed `× N.NN`,
with a live "a regional landmark keeps <km> clear" readout computed from
`radii.REG × crowd`), a `compete` toggle, a `Run landmark pass` button with a
percentage fill, and a per-family drill whose per-type caps step along a
**13-rung ladder** `[0,1,2,3,5,8,12,20,30,50,80,120,200]`. `JOURNEY PLANNER` is
a teaser card plus an `hOpenPlanner` that sets `s.scr = 'planner'`.

**Exists today:** all four subjects exist, as **15** categories rather than 4.
Landmarks in particular — `civilization_workspace.gd` opens `Landmarks` by
default (`open_category("Landmarks")`), and `menus.gd`'s `Landmark types`
submenu already prints the same `armed · placed` pair per family.

**(d) `ldCartoCats` — CARTO** (lines 354–443). Four categories: `LAYERS &
STYLE` (a visibility list with per-layer opacity), `LABELS`, `ICONS`,
`TERRAIN APPEARANCE`. Shipped: **10** categories (`Map style`,
`Terrain appearance`, `Colours`, `Layers`, `Roads & routes`, `Labels`,
`Assets & landmarks`, `Political display`, `Visibility / zoom`, `Map presets`)
plus `layers_popover.gd`.

### 2.5 Viewport

`flex:1;position:relative;min-width:0;background:var(--map);overflow:hidden`, a
full-bleed `<canvas>` with `touch-action:none`, and three overlays:

| Overlay | Spec |
|---|---|
| scale bar | `bottom:12px;left:12px`; label `var(--m2) var(--dim)`; bar **84 × 4** with three 1 px `var(--dim)` borders (no bottom); label text = `fmtKm(84 / view.s * 2.5)` |
| armed chip | `top:12px`, centred; `var(--acc)` on `var(--accInk)`; `padding:7px 15px`, `radius:999px`, `pointer-events:none` |
| toast | `bottom:16px`, centred; `background:var(--tst)`, `radius:var(--rCtl)`, `padding:9px 17px`, `max-width:80%`; 2 600 ms |

Gestures, from `_pd` / `_pm` / `_pu`: **two pointers → pinch zoom**, clamped
`0.05..3`; wheel → `exp(-deltaY * 0.0016)`; **long-press 420 ms → sample**;
`contextmenu` → sample plus the toast *"Long-press = right-click — sampled
here"*. Haptics are declared and quantified: `navigator.vibrate` at
`sample:12, arm:10, detent:8, place:14` ms.

**Exists today:** `viewport_host.gd` and the `tool_overlay.gd` family. The armed
chip's counterpart is the tool-armed state drawn in `tool_options_row`; there is
no centred floating chip over the map. Haptics have no counterpart in GDScript
at all — `Input.vibrate_handheld()` is the Godot call and nothing references it.

### 2.6 Right dock — `--rdW`, two contexts

Title row: `min-height:var(--tap)`, `padding:5px 12px`; title in
`var(--m2)/.18em var(--acc)`; a right-aligned sub in `var(--m2) var(--dis)`.

**`rdSample`** — with nothing sampled, one paragraph at `padding:16px 13px`,
`var(--m2)/1.7 var(--dis)`: *"tap anywhere on the map to sample it — elevation,
terrain, climate, hydrology, geology and biome for that cell. Long-press does
the same as a right-click."* Otherwise `sampleGroups`: each a
`border-bottom:1px solid var(--div)` block with a `.2em var(--faint)` title and
rows of `min-height:22px` — key in `var(--m2) var(--faint)` at `flex:1`, value in
`var(--m1)`, right-aligned.

**Exists today, and this is the closest match in the whole canvas.**
`right_dock.gd`'s `CTX_SAMPLE` is the default context, `SAMPLE_FIELDS` is the
field list in spec order, every row is backed by `bridge.sample_cell()`, and
`SAMPLE_STAGE` already dashes a row **with its stale reason** — the same
"omit, do not fake" rule this project runs on. The canvas's `sampleData()`
returns elevation, slope, aspect and a biome string; the shipped panel returns
more (the 800×1280 capture shows Position, Cell, Elevation, Slope, Aspect,
Plate + type, Boundary + distance, Resistance, Lithology, Temperature,
Precipitation, Drainage, Biome, Soil, Control, Nearest, Route cost, E–W profile).

**`rdMeasure`** — a headline block (`mBigLabel` in `.16em var(--faint)`, `mBig`
in `500 26px var(--acc)`, `mBigSub`), then for Cross-section a **300 px**
profile: a 44 px A/mid/B gutter, an SVG `viewBox="0 0 100 400"` with two
0.6-width grid lines at x = 33 and x = 66 and a `vector-effect="non-scaling-
stroke"` path, and a min/max row. Then per-segment rows, a stats block,
`subtract water` (area only), `clear`, and a footnote. The profile is computed
at **n = 120** samples along A→B.

**Exists today:** `global_tools.gd`'s measure API, and
`journey_planner_view.gd::_rebuild_profile()` — a profile drawn in the strip
under the map, not in the right dock. `tool_bar.gd`'s own note states the
shipped placement: *"Cross-section is one line read in the strip below."* So the
canvas **moves** the profile from the strip into the right dock.

### 2.7 Timeline

Collapsed (`tlShow`): a `--sbH` 28 bar — `▸` chevron, `TIMELINE` in
`var(--m2)/.18em var(--sec)`, and `tlSummary` = `'yr ' + year + ' · N of 6
layers armed'`. Open (`tlOpen`): **118 px** — title in `var(--acc)`, a
disclosure (*"not wired — these record which simulation layer you want; no layer
renders yet"*), six chips at `min-height:var(--ctl)` = 36, `padding:0 12px`,
`radius:999px`, then a `yr NNN` label of `width:52px` and a scrubber in a
`var(--tap)`-tall hit area.

**Exists today, and the layer list matches exactly.** `dcc_shell.gd`'s
timeline-layer array (grep `["Climate", true]`) carries
`["Climate", true], ["Population", true], ["Economy", false],
["Politics", true], ["Infrastructure", false], ["Warfare", false]` — the
canvas's six, in the canvas's order. `_tl_load_layers()` runs before either
composition builds a view of them. Defaults differ: the canvas arms Climate and
Population, this shell also arms Politics. `timeline_bar` measures **17 px**
collapsed and is `visible = false` on a fresh boot, against `--sbH` 28.

### 2.8 Status bar — `--sbH:28`

`gap:14px`, `padding:0 12px`, `border-top:1px solid var(--hair)`,
`background:var(--pan)`, `font:var(--m2)`:

| Slot | Canvas | Ink |
|---|---|---|
| left | `proj + ' · ' + tool + ' · ' + w + '×' + h + ' dp'` | `var(--sec)` |
| centre | `'last pass 10 Ecology · 108 ms · repaint 84 ms · autosave ' + savedAt` | `var(--faint)` |
| right | `N landmarks · N stamps` | `var(--dis)` |

**Exists today:** `status_row`, measured **35 px** (`h_status` role 36 — 8 px
taller than the canvas). The centre slot's timings are prototype constants; the
shipped shell has real ones. Note that the canvas prints the **frame size in
dp** in the left slot, which is a prototype affordance rather than a product
feature.

---

## 3. `scrPlanner` — the Journey Planner as a third screen

Template lines 555–654. Chrome: a `--menuH` bar with a `‹ SHELL` back chip
(`background:var(--ins)`, `padding:0 13px 0 9px`), `JOURNEY PLANNER` in
`var(--m1)/.18em var(--acc)`, a sub, and a right-aligned verdict
(`planVerdict:'STRAINED'`, ink `var(--warn)`). Then a three-column body that
reuses `--ldW` and `--rdW`.

| Region | Canvas |
|---|---|
| left dock | 4 accordion groups — `PARTY` (6 cells plus `pace` and `cargo load` sliders), `SUPPLY` (4 cells), `SEASON & WEATHER` (4 cells), `STAGE OVERRIDES` (4 cells). Cells `min-height:var(--rowD)` in a 2-column grid |
| spine | a `DISTANCE SPINE` header, then a **172 px** SVG at `viewBox="0 0 1000 160"`, `preserveAspectRatio="none"`, fill α .12, stroke 1.4 non-scaling, with one vertical mark per stage boundary at x = 200 / 400 / 600 / 800 |
| stage strip | one `flex:1` chip per stage, `min-height:var(--rowD)`, name over meta; selected = `var(--wash2)` / `var(--acc)` |
| matrix | header `min-height:var(--row)` on `var(--pan)`; 7 columns at widths `22% 13% 13% 13% 13% 13% 13%` — `STAGE TERRAIN WEATHER CARGO SUPPLY KM/DAY DAYS`, with columns 1–3 inked `var(--acc)`; rows `min-height:var(--rowD)`, ellipsised, `padding-right:6px`; footnote *"lit columns are overrides you set · blank inherits the model · problem stages pin above route order"* |
| right dock | a `TOTAL` block (label, `500 26px var(--acc)`, sub) then 4 result groups — `TIME`, `LOAD`, `SUPPLY`, `COST` — as key/value rows |

**Exists today, and it is bigger than the canvas's version.**
`journey_planner_view.gd` (`class_name JourneyPlannerView`, ~4 700 lines) is
already a **whole-domain takeover** built to `JOURNEY_PLANNER_SPEC.md`'s
"direction 1a (distance spine)". It carries `_rebuild_profile()`,
`_rebuild_stops()`, `_rebuild_matrix()` with `_matrix_header()`,
`_matrix_clear_column()` and `_matrix_fill_down()`, `_build_verdict_card()`, and
`_build_time_group()` / `_build_load_group()` / `_build_supply_group()` /
`_build_cost_group()` — the canvas's four result groups, by name. It also has
vessels, a trace group, presets, save/load journeys and stage overrides, none of
which the canvas draws.

**So the delta is the framing, not the content.** The canvas makes it a third
top-level screen reached from `Data ▸ Journey planner`; this shell arms it as a
tool inside CIVIL and swaps the domain region. Converting it means giving
`DccShell` a screen-level state *above* the domain layer — §4.

---

## 4. The three columns

### 4.1 What the desktop shell already provides — "re-home this", not "build this"

| Canvas item | Shipped counterpart |
|---|---|
| horizontal tool rail (`--railH:56`) | `tool_options_row`, **55 px**, a two-row `VBox` from `tool_bar.gd` |
| vertical domain rail (`--railW:52`) | `rail_column`, **47 px**, the same three domains, `_rail_foot_stack` for the foot label |
| measure sub-modes and cross-section fields | `global_tools.gd::MEASURE_MODES` / `SECTION_CHANNELS`, drawn by `_build_measure_tools()` in the canvas's own composition |
| brush size / hardness / shape | `tool_bar.gd::_build_sculpt_options()`, writing through `bridge.sculpt_*` |
| right-dock Sample | `right_dock.gd::CTX_SAMPLE`, `SAMPLE_FIELDS`, `bridge.sample_cell()`, with per-row stale reasons |
| timeline, six layers | `dcc_shell.gd::_build_timeline()` plus the six layers verbatim |
| status bar, three slots | `status_row` |
| Journey Planner (spine, stages, matrix, totals) | `journey_planner_view.gd`, a superset |
| landmark families, `armed · placed`, run pass | `civilization_workspace.gd` ▸ `Landmarks`; `menus.gd`'s `Landmark types` submenu |
| pipeline / sculpt mode switch | `DccShell.select_domain_mode("world", …)` |
| menu rows | of the canvas's **32** `it()` rows, about 20 exist under the same or a near name (§2.1) |
| project picker cards | `open_project_dialog.gd::open_welcome()` — a `Window`, not a screen |

### 4.2 What is genuinely new

| New thing | Why it has no counterpart | What it touches |
|---|---|---|
| **Portrait / landscape dock split (232 / 320)** | `DccTheme` has no orientation predicate — `is_touch()`, `is_tablet()`, `is_phone()` is the whole vocabulary, and `ROLE` is a two-column table. `_landscape` exists on `DccShell`, but every consumer of it is a `_phone_*` surface | `dcc_theme.gd` (`ROLE`'s shape, or a third lookup), `dcc_shell.gd`'s desktop composition, `_roleresolve_probe.gd` |
| **`scrPlanner` as a top-level screen** | `DccShell` has no state above the domain layer; the planner is armed as a tool | `dcc_shell.gd`, `app.gd::_on_workspace_changed`, `journey_planner_view.gd::_show()/_hide()` |
| **`scrPicker` as a screen** | the launcher is a `Window` | `open_project_dialog.gd`, `app.gd::open_welcome()` |
| **The tablet palette** | 11 dark and 14 light tokens differ, three of them structurally (opaque hairlines) | `dcc_theme.gd::DARK` + `LIGHT`, and `rebuild_theme()`'s two-palette remap |
| **`--rCtl:12` / `--rPan:16` radii** | no radius tokens exist outside `pill()` | `dcc_theme.gd`, the `StyleBoxFlat` factories in `dcc_widgets.gd` |
| **Per-stage `Run stage NN`** | **the engine has no partial recompute** — one `generate()` resolves all ten stages | not buildable; it would need an engine change |
| **The canvas's ten stage names** | they are not `progress.rs::STAGE_NAMES`, which `_assert_stage_names()` guards | not adoptable without renaming the engine's stages |
| **`LIVE` / `DRAFT` / `ARCHIVE` per world** | no project-state field exists on disk | `open_project_dialog.gd` plus the save format |
| **Haptics** (`vibrate`, 8–14 ms) | nothing calls `Input.vibrate_handheld()` | small; per interaction site |
| **Stylus pressure / palm rejection rows** | no counterpart in `menus.gd`; the canvas reads `e.pressure` into `sc.brush` | `tool_bar.gd`, input handling |
| **Centred armed-tool chip over the map** | the armed state lives in the tool bar | `viewport_host.gd` |
| **7 → 3 menus plus overflow** | see the `command_index.gd` hazard in §2.1 | `menus.gd`, `command_index.gd`, `shortcuts_dialog.gd` |

### 4.3 The cost of each, in the order a reader would spend it

| Item | Touches | Notes |
|---|---|---|
| Undo/redo buttons in the rail | `tool_bar.gd` only | two buttons over commands that exist |
| **Dock width 232 / 320** | `dcc_theme.gd`, `dcc_shell.gd`, `_roleresolve_probe.gd` | **this is the fix for a measured defect** — §5 |
| Armed chip, scale-bar geometry, toast | `viewport_host.gd` | presentation only |
| Cross-section profile into the right dock | `right_dock.gd`, `tool_bar.gd`'s note, `journey_planner_view.gd`'s strip | it *moves* a surface; both ends must move together |
| Radii `--rCtl` / `--rPan` | `dcc_theme.gd`, `dcc_widgets.gd` | one constant, many call sites |
| Picker as a screen | `open_project_dialog.gd`, `app.gd` | plus a new per-world state field for the badge |
| Planner as a screen | `dcc_shell.gd`, `app.gd`, `journey_planner_view.gd` | a new state layer above domains |
| Menu collapse to 3 plus overflow | `menus.gd`, `command_index.gd`, `shortcuts_dialog.gd` | **searchability regresses silently** if the index rows are not written in the same change |
| The palette | `dcc_theme.gd` (both dicts), `rebuild_theme()`, every contrast pair | a re-base; §0.1's ghost-ink pair drops below 3:1 |
| Per-stage run, and the canvas's stage names | the engine | not adoptable as drawn |

---

## 5. Portrait and landscape — the verified absence, and what it is hiding

**Verified.** `grep -rn "tablet_portrait\|TABLET_PORTRAIT" --include=*.gd` over
`cartalith-native/godot-project` returns **nothing**. The claim holds.

The narrower true statement: `DccShell` *does* track orientation.
`_compute_layout_mode()` sets `_landscape = size.x > size.y`, and
`_on_window_resized()` compares it against `was_landscape` and re-lays the shell
when it flips. **Every other `_landscape` reference in the tree is a `_phone_*`
surface.** Counted this pass rather than estimated — `grep -c "_landscape"
dcc_shell.gd` is **31**: one declaration, three comment lines, the three
orientation-machinery sites above, and **24 consumer references spread over
exactly seven functions**, every one of them phone:
`_apply_phone_nav_orientation` (11), `_apply_phone_orientation` (7),
`phone_content_insets` (2), `_phone_bottom_reserve`, `_show_next_coach_mark`,
`_on_phone_sheet_grab_input`, `_snap_phone_sheet` (1 each). So the machinery
exists and the tablet composition simply never reaches it. `DccTheme` itself has
no orientation predicate at all.

*(No `dcc_shell.gd` line numbers are quoted anywhere in this file. It drifted by
88 lines during the single pass that wrote this — `_landscape = size.x > size.y`
moved from 813 to 822 and the timeline layer array from 4397 to 4485 while other
lanes worked in it. `MISTAKES.md`, "Re-resolve a citation late in a long pass":
grep the string.)*

**And here is why that matters, measured rather than argued.**
`_tabspec_probe` at each frame, `--force-touch`:

| Frame | rail | left dock | viewport | right dock | sum | frame w | overflow |
|---|---|---|---|---|---|---|---|
| 1280×800 | 47 | 400 | 432 | 400 | 1279 | 1280 | 0 |
| 1440×900 | 47 | 400 | 592 | 400 | 1439 | 1440 | 0 |
| 1600×1000 | 47 | 400 | 752 | 400 | 1599 | 1600 | 0 |
| 2560×1600 | 47 | 400 | 1712 | 400 | 2559 | 2560 | 0 |
| **800×1280** | 47 | 400 | **237** | 400 | **1084** | 800 | **+285 px** |
| **900×1440** | 47 | 400 | **237** | 400 | **1084** | 900 | **+185 px** |

At both portrait frames the right dock's right edge lands at **x = 1085**. Both
docks report `get_combined_minimum_size().x = 400`, so the row's floor is
47 + 400 + 400 = 847 plus the map's own floor, and the composition cannot fit.
Last batch's shipped 800×1280 capture shows it directly: the SAMPLE column's
values are cut off at the frame edge, the menu bar's last item is sliced
mid-word, and the map is not visible at all.

The canvas's portrait figure is the fix, and it fits with room:
52 + 232 + 232 = 516, leaving **284 px** of map at 800 wide and **384** at 900.

**So `--ldW/--rdW: 232 | 320` is not a styling preference.** It is the one line
in this canvas that repairs something currently broken, and the one item in §4.3
whose cost buys a defect fix rather than a resemblance.

---

## 6. What could not be measured, and why

* **Whether the owner wants this canvas adopted at all.** `CLAUDE.md`'s "the
  newer canvas wins" does not decide it: the PC and tablet canvases were
  imported from the owner's project on the *same day* and are two drawings of
  different form factors that overlap, not two drafts of one drawing.
* **Whether `Cartalith Tablet.dc.html` is that project's current head.** The
  same open question `CLAUDE.md` records for the v2.11 reference. It was
  imported by `DesignSync` on 2026-09-07 and has not been re-fetched since.
* **The canvas's own *rendered* layout.** Everything here is read from the
  source, not from a live DOM. A rendered measurement would need the CDP recipe
  in `CAPTURE.md`; the figures above are the canvas's declared values, which is
  what a shell would be built against in any case.
* **What `--map` and `--tst` should map to.** Both are declared and drawn;
  neither has a `DccTheme` token, so that is a decision, not a measurement.
* **Light-theme shipped captures.** `_ph412_probe` and `_tabspec_probe` both
  boot dark, so the tablet light palette (§0.1) is compared value-to-value only.
* **How the canvas behaves at a size that is not one of its six frames.** The
  prototype only ever renders `FRAMES()`; a real tablet at, say, 1194 × 834 is
  outside every one of them, and `isPortrait()` is the only rule that would
  apply.

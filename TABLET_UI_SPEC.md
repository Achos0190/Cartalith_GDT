# TABLET_UI_SPEC.md — the tablet canvas, inventoried against the shell

**What this is:** every element of the tablet design canvas, with the canvas's
own figures and the shell or engine symbol each maps onto — the reference a
tablet conformance pass builds from. **What it is not:** a status record. How
far the tablet is built is `cartalith-native/docs/STATUS.md`'s answer; open
tablet work is routed from `OUTSTANDING_WORK.md` to this file. The phone's
counterpart is `ANDROID_UI_SPEC.md`, and the rules the two share (gesture
arbitration, the touch floor) are written there once.

**Authority:** `design/mcp-2026-09-07/Cartalith Tablet.dc.html` (131 206 bytes,
imported 2026-09-07 by `DesignSync` from the owner's "UI mockups planning"
project; `CAPTURE.md` in the same folder records the owner's *"Implement:
Tablet"*). It needs `support.js`, `cartalith-dcc-parts.js` and
`landmark-glyphs.js` beside it to render.

**Adoption is decided.** Owner, 2026-09-07: *"All designs layouts and styles
should match 100%. Check all designs, pc, tablet and phone."* "100% on tablet"
means every layout and style with **two named exceptions**, both ruled the same
evening (`LARGE_ITEM_RULINGS.md`):

- **Ruling D** — `Run stage NN` is **drawn**, wired to a **full** run, and each
  button carries a tooltip saying it recomputes the whole pipeline. Partial
  recompute was declined as Phase-scale; do not re-propose it in a tablet pass.
- **Ruling E** — the tablet shows the **engine's** stage names
  (`progress.rs::STAGE_NAMES`, guarded by `_assert_stage_names()`), not the
  canvas's.

Ruling DS-03 (2026-09-03, *"keep everything; reflow only"*) still binds the
*content*: no desktop control leaves the tablet. It is not an answer about the
tablet's *shape* — an earlier version of this file read it that way and parked
adoption behind a question the owner had already answered.

**Method:** all 49 `sc-if` blocks walked in order, each counterpart then found
**in the code, by symbol**; every canvas number is quoted from the canvas's
source. Canvas line numbers are orientation only, and no shell line numbers are
quoted (`dcc_shell.gd` moved 88 lines during the pass that first wrote this) —
grep the symbol.

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

### 0.1 Palette

Both palettes are one CSS string in `valsShell()`, a ternary on `s.light`:
19 colour variables plus `--shadow`. Compared value-by-value against
`dcc_theme.gd`'s `DARK` / `LIGHT` dictionaries.

**DARK — 17 shared, 5 exact, 12 differ; `--map` and `--tst` have no token;
`--shadow` is not a colour token.**

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
| `--warn` | `#e0a34a` | `warn` | `#e0a840` | differs (the canvas reuses its accent) |
| `--good` | `#7ea86a` | `good` | `#6fae7d` | differs |
| `--wash2` | `rgba(224,163,74,.15)` | `accent_wash_2` | α `.16` | differs |
| `--hair` | `#232628` | `line` | `rgba(255,255,255,.10)` | **differs structurally** |
| `--div` | `#1e2123` | `line_soft` | `rgba(255,255,255,.07)` | **differs structurally** |
| `--bor` | `#2c3033` | `border` | `rgba(255,255,255,.16)` | **differs structurally** |
| `--map` | `#0c0d0e` | — | — | no token |
| `--tst` | `#1c1f21` | — | — | no token |
| `--shadow` | `0 14px 40px rgba(0,0,0,.5)` | — | — | not a colour token |

The five that agree are the whole text ramp plus the accent: the tablet canvas
is less a different palette than a different **ground**.

**LIGHT — 17 shared, 2 exact (`--acc` `#a4650f`, `--warn` `#9a6a12`), 15
differ.** Not a re-tune of the shipped light palette; a warmer paper.
Representative rows:

| Canvas var | Canvas | `DccTheme.LIGHT` |
|---|---|---|
| `--sur` | `#efece6` | `bg` `#f4f2ee` |
| `--pan` | `#f6f4f0` | `panel` `#fbfaf7` |
| `--body` | `#2b2d2a` | `text` `#23241f` |
| `--hair` | `#d9d5cc` | `line` `rgba(0,0,0,.14)` |
| `--bor` | `#cdc8bd` | `border` `rgba(0,0,0,.20)` |
| `--wash2` | `rgba(164,101,15,.13)` | `accent_wash_2` α `.16` |
| `--accInk` | `#ffffff` | `accent_ink` `#f7f4ee` |

**Three consequences for anyone adopting it:**

- **The hairlines cannot be reconciled by a value.** `line` is white at α .10,
  which composites to `#2a2b2c` over `panel` and `#252627` over `bg`; the
  canvas's `--hair #232628` is one opaque colour everywhere. No alpha
  reproduces both, so this is a change to how `DARK` is built, not three
  re-tuned numbers.
- **A third palette is not a `c()` branch**: `DccShell.rebuild_theme()` remaps
  live colours through `DccTheme.DARK if was_dark else LIGHT`, a two-way remap.
- **Re-basing moves every contrast pair** (`MISTAKES.md`, "Re-base a shared
  token / constant set"). Computed, WCAG 2.x:

  | Pair | Shipped | Tablet canvas |
  |---|---|---|
  | ghost/disabled ink on panel | `#5f6468` on `#121314` = **3.11:1** | `#565b5f` on `#141618` = **2.64:1** |
  | light body ink on panel | `#23241f` on `#fbfaf7` = **14.97:1** | `#2b2d2a` on `#f6f4f0` = **12.65:1** |

  The first crosses below 3:1. Everything else computed moves by less than 0.5
  (dark body 11.41 → 11.13, dark accent on ground 8.75 → 8.62, dark reversed
  ink on accent 8.60 → 8.39); light reversed ink on accent *improves*,
  4.30 → 4.72.

### 0.2 Frames, orientation and zoom

`FRAMES()`, six of them; the default is `t1600`:

| id | label | w × h | orientation |
|---|---|---|---|
| `t1280` | `10″ 1280×800` | 1280 × 800 | landscape |
| `t1440` | `11″ 1440×900` | 1440 × 900 | landscape |
| `t1600` | `13″ 1600×1000` | 1600 × 1000 | landscape (default) |
| `t800` | `10″ ▯ 800×1280` | 800 × 1280 | **portrait** |
| `t900` | `11″ ▯ 900×1440` | 900 × 1440 | **portrait** |
| `t2560` | `2560×1600` | 2560 × 1600 | landscape |

`isPortrait(){return f.h>f.w}` — aspect, nothing else. The shell's predicate is
the same test: `DccShell._compute_layout_mode()` sets `_landscape = size.x >
size.y`, `_on_window_resized()` re-lays the shell when it flips, and
`DccTheme.set_portrait()` / `is_tablet_portrait()` (`touch and not phone and
portrait`) carry it into `role_px()`.

`_zoom()` is `min(1, max(560, innerWidth−40)/fw, max(520, innerHeight−150)/fh)`.
`CAPTURE.md` quotes it without the two `max()` floors — harmless for capture (a
floor only raises a numerator), wrong for a small window.

### 0.3 Metric tokens

`themeVars`, one string:

```
--menuH:52px --railH:56px --railW:52px --sbH:28px
--tap:44px --ctl:36px --row:48px --rowD:44px
--rCtl:12px --rPan:16px
```

plus a portrait/landscape pair in `valsShell()`:

| | portrait | landscape |
|---|---|---|
| `--ldW` / `--rdW` | **232px / 232px** | **320px / 320px** |
| `--m0` / `--m1` / `--m2` | 12 / 11 / 9.5 px | 12.5 / 11.5 / 10 px |

`--m1` is the frame's base font (`font:var(--m1)` on the shell root).

Against the shell's tablet figures (`DccTheme.role_px()` on a tablet):

| Canvas token | Canvas | Shell | |
|---|---|---|---|
| `--menuH` | 52 | `ROLE["h_menu_bar"]` **52** | exact |
| `--railH` | 56 | `ROLE["h_tool_options"]` **56** | exact |
| `--railW` | 52 | `ROLE["w_rail"]` **48** | differs |
| `--sbH` | 28 | `ROLE["h_status"]` **36** | differs |
| `--ldW` / `--rdW` portrait | 232 | `DccTheme.TABLET_PORTRAIT` **232 / 232** | exact |
| `--ldW` / `--rdW` landscape | 320 | `ROLE["w_left_dock"]` / `["w_right_dock"]` **400 / 400** | **held deliberately** — see below |
| `--tap` | 44 | `ROLE["btn_min_h"]` 44; `DccShell._scaled()` floors any figure `DccTheme.TABLET` does not name at `max(44, round(px × TOUCH_SCALE))`, `TOUCH_SCALE = 1.53` | exact floor |
| `--ctl` | 36 | no role reads 36; mode chips floor at `ROLE["chip_min_h"]` 34 | differs |
| `--row` / `--rowD` | 48 / 44 | `ROLE["row_min_h"]` **44** — one figure, not two | differs |
| `--rCtl` | 12 | `ROLE["btn_radius"]` **12** on tablet (8 on desktop) | exact |
| `--rPan` | 16 | no role | no token |

Five tokens the PC canvas's touch branch declares — `--tbH 56`, `--btnH 44`,
`--tool 44`, `--pad 16`, `--fs 14` — are **absent** from the tablet canvas.

**Landscape docks stay at 400 until their content reflows.** A same-day change
moved them to 320 and was reverted 2026-09-13: at 320, 8 of 10 rail nodes' dock
content still drew 331–382 px (`_ds03fit_probe.gd --force-touch` at 2560×1600),
so the docks forced themselves back open — a declared width is not a drawn one.
`dcc_theme.gd`'s `W_DOCK_TABLET` comment carries the measurement; the width
moves in the same change that reflows the content. **The same holds in
portrait:** a role sets a dock's floor, and a child's minimum still wins
(`MISTAKES.md`, "Read a layout that overflows the screen").

**The touch floor reaches tablet docks through `DccShell.tablet_fit()`**, which
floors **height only** (`BaseButton`/`LineEdit`/`TextEdit` to
`ROLE["btn_min_h"]`, `Label` fonts to their role) and never shrinks. Three call
sites: the `tool_options_row` rebuild inside the `is_tablet()` branch; the
deferred call at the end of `register_workspace()`; and
`_run_tablet_dock_arbitrate()`, which re-fits both docks whenever a node is
added under them — the one that reaches controls a later `_rebuild_*` creates.
Width stays a per-call-site fix (`MISTAKES.md`, "Floor a control that a rebuild
creates").

**Radii.** The "§11 radius-0" rule described an older desktop artboard; both
current canvases left it (PC: 81 `999px` pills, 76 `8px` corners; tablet: 32
`var(--rCtl)`, 11 pills, 14 circles). The tablet is 12 where the PC is 8.

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

Both rails exist in the shell too: the horizontal one is `tool_options_row`
under `DccShell`'s bar, the vertical one `rail_column`. What differs is what the
horizontal one carries (§2.2).

### 0.5 The probe

`cartalith-native/godot-project/_tabspec_probe.tscn` / `.gd` boots
`shell/app.tscn` in a `SubViewport` (so frames larger than the screen are
reachable) and reports each region's laid-out size against the canvas figure,
the live `role_px` answers, the menu-bar titles and the dock-row fit. Read-only.

```
godot --path . _tabspec_probe.tscn -- --force-touch --vp 1600x1000 --tag land
godot --path . _tabspec_probe.tscn -- --force-touch --vp 800x1280  --tag port
```

`--force-touch` is required: without it `_touch` is false off a device and every
frame gets the desktop composition. With it, all six frames (short/long ≥ 0.6)
classify as tablet by aspect — `DccShell._is_tablet_sized()`, the size test that
keeps a 16:9 tablet off the phone composition, answers only on a real mobile
device. Measure at several frames, not one (`MISTAKES.md`, "Report a layout
measurement").

---

## 1. `scrPicker` — "Open a world"

Template lines 29–54.

| Element | Canvas spec | Shell counterpart |
|---|---|---|
| page | `padding:34px 40px`, column, `overflow-y:auto` | — |
| wordmark | `CARTALITH`, `500 var(--m2)`, `letter-spacing:.3em`, `var(--acc)` | `open_project_dialog.gd` draws a welcome `Window`, not a screen |
| title | `Open a world`, `400 26px`, `var(--ink)` | " |
| subtitle | `pickerSub` = `'6 tablet frames · ' + f.label + ' · ' + (port ? 'portrait, docks 232 dp' : 'landscape, docks 320 dp')` | prototype-only string |
| grid | `repeat(pickCols,1fr)`, `gap:14px`; `pickCols = port ? 2 : 3` | — |
| NEW WORLD card | `1px dashed var(--bor)`, `radius:16px`, `min-height:186px`, `+` at `300 30px`, caption `NEW WORLD`, sub `seed · size · plates` | `menus.gd` File ▸ `New world…` (Ctrl+N) → `new_world_dialog.gd` |
| world card | `1px solid var(--hair)`, `radius:16px`, `min-height:186px`; 96 px gradient thumb; state pill `9px/.14em` on `rgba(0,0,0,.42)`; name `500 var(--m0)/.1em`; meta; status | `open_project_dialog.gd::open_welcome()` lists real projects |
| card data | 5 hard-coded worlds (`ELDRA`, `KESSA`, `VHAL SERAI`, `THORNWOOD`, `HIGH SADDLE`), meta `seed · N² · edited … · NNN MB`, state `LIVE`/`DRAFT`/`ARCHIVE` | the project list is real; **`LIVE`/`DRAFT`/`ARCHIVE` has no engine counterpart** |

**The three-state badge cannot be read from anything.** No project-state field
exists in `engine_bridge.gd`, `open_project_dialog.gd` or the save format. The
nearest signal, `world_workspace.gd`'s `_stale_from_stage`, is per-*stage*
inside an open world, not per-world on disk — and the card's `resolved · stage
10` / `draft · stage 03` / `stale from 07` line has the same problem. Building
the badge means a new persisted field, not a read. Converting the launcher to a
screen is otherwise presentation; the card grid, `pickCols` and thumbnails are
new.

---

## 2. `scrApp` — the shell

### 2.1 Menu bar — `--menuH:52`

Template lines 56–79. Row: `gap:4px`, `padding:0 10px 0 6px`,
`border-bottom:1px solid var(--hair)`, `background:var(--pan)`, `z-index:60`.

| Element | Canvas spec |
|---|---|
| overflow ☰ | `var(--tap)` square (44), `radius:var(--rCtl)`, `font-size:15px`, `data-id="OVERFLOW"` |
| menu title | `min-height:var(--tap)`, `padding:0 14px`, `radius:var(--rCtl)`, `var(--m1)`, `letter-spacing:.1em`; open = `background:var(--ins)`, ink `var(--ink)` |
| menus | **exactly three**: `['File','World','Data']` |
| project name | `var(--m1)`, `letter-spacing:.16em`, `var(--sec)` |
| project meta | `var(--m2)`, `var(--dis)`; `'4096² · seed 118402'` |
| theme toggle | `var(--tap)` square, glyph `☀` / `☾` |
| popup | `top:calc(100% + 4px)`, `left:menuX`, `min-width:300px`, `max-width:390px`, `max-height:560px`, `radius:var(--rPan)`, `1px solid var(--bor)`, `box-shadow:var(--shadow)`, `padding:7px 0` |
| `menuX` | `OVERFLOW→6`, `File→56`, `World→132`, `Data→216` — **hardcoded**, not measured off the button |
| popup row | `min-height:var(--tap)`, `padding:0 16px 0 {ind}px` (`ind` 16, or **36 for a submenu child**); label `flex:1`, shortcut `var(--m2) var(--dis)`, trailing glyph `var(--faint)`; danger ink `#c05a4a` |
| separator | `height:1px`, `background:var(--div)`, `margin:5px 0` |
| section head | `padding:7px 16px 3px`, `var(--m2)`, `letter-spacing:.2em`, `var(--dis)` |

**How the shell builds it.** `menus.gd`'s tablet branch builds the canvas's
three menus plus ☰ (its "Tablet only: the ☰ overflow and the World menu"
block): File and Data are the shipped menus; World is built from canvas rows
that have a real action; ☰ holds the canvas's overflow rows, moved (not copied)
from Edit/Assets/Preferences/Help, then those menus' remaining rows as
submenus named for their shipped menu — so under DS-03 every PC row stays
reachable, most one level deeper. Rows keep being real `PopupMenu` items, so
`command_index.gd` and `shortcuts_dialog.gd`, which walk the live menu bar
recursively, still find them; a command that stopped being a menu row at all
would need `EXTRAS` / `UNLISTED` rows in the same change (`MISTAKES.md`, "Move a
command off the menu bar").

`_menuRows()` in full, and the command each row reaches:

| Canvas menu | Canvas row | Command it maps to |
|---|---|---|
| File | `New world…` | `New world…` (Ctrl+N) |
| | `Open…` | `Open project…` (Ctrl+O) |
| | `Recent ▸` (4 names, `ind:36`) | the Recent submenu |
| | `Save` `⌘S` | `Save project` (Ctrl+S) |
| | `Save as…` | `Save as…` |
| | `Revert to saved` | `Revert to last save` |
| | `Export map…` `⇧⌘E` | the Data manager's Export routes (`data_manager_window.gd`: Export ▸ Maps / GIS / World data); no menu row of this name. The 16K/32K single-image export is `EXPORT_SCOPE.md`'s (un-shelved by ruling 15, resumed by Ruling AP) |
| | `Export data…` | nearest is Data ▸ `World data tables…` |
| | `Close world` (danger) | `Close project` (Ctrl+W) |
| World | `Run pipeline` `⌘R` | `Run pipeline` (Ctrl+R) — the WORLD dock's RUN button's action (`world_workspace.gd::_regenerate_live()`) |
| | `Run from stale` | not a separate command — the RUN button relabels; omitted rather than fabricated |
| | `Reset to seed…` (danger) | nearest is Edit ▸ `Reset generation parameters` |
| | `Generation pipeline` / `Sculpt` (radio) | `DccShell.select_domain_mode("world", …)` |
| | `Seed & size…` | `new_world_dialog.gd::request()`; creation-time |
| | `Plate settings…` | no single target — `tect.plates` and siblings are live WORLD-dock parameters; omitted |
| Data | `Travel library…` `⇧L` | `Travel library…` (Shift+L) — exact, accelerator included |
| | `Data manager…` | `⧉ Data manager` |
| | `Export CSV…` / `Export GeoJSON…` | `journey_planner_view.gd::_export_stage_table()` exports a table; no rows by these names |
| | `Journey planner` | a **tool takeover**, not a menu row — §3 |
| OVERFLOW | `Undo` `⌘Z` / `Redo` `⇧⌘Z` | Undo / Redo, same accelerators |
| | `Undo history — N steps` | `Undo history…` |
| | `Asset library…` | `⧉ Asset library` (Shift+A) |
| | `Landmark types ▸` (family rows, `armed · placed`) | the `Landmark types` submenu — same shape, same counts |
| | `Toggle theme` | a flat toggle on tablet; the three-way choice stays at ☰ ▸ Preferences ▸ Theme |
| | `Units — kilometres/miles` | the Units submenu (km / mi / nmi, `dcc_units.gd`) |
| | `Stylus pressure — on/off` | **no counterpart** anywhere in the shell; omitted |
| | `Palm rejection — on` | **no counterpart**; the canvas's own row is a toast, not a setting; omitted |
| | `Shortcuts…` | Help ▸ `Keyboard shortcuts…` |
| | `About Cartalith` | `About` |

### 2.2 Horizontal rail — `--railH:56`

Template lines 81–128. `gap:8px`, `padding:0 8px`, `background:var(--pan)`,
`border-bottom:1px solid var(--hair)`, `z-index:50`. Three zones:

1. **Tool picker**, `flex:none`, `gap:3px`: one `var(--tap)` square per tool,
   `radius:var(--rCtl)`, a 15 × 15 stroked SVG at `stroke-width:1.25`; selected
   = `background:var(--acc)` with ink `var(--accInk)`. `_toolsFor(domain)`:
   * `WORLD` — Inspect, Pan, Measure, Sculpt, Biome paint (**5**)
   * `CIVIL` — Inspect, Pan, Measure, Settlement, POI, Territory, Way (**7**)
   * `CARTO` — Inspect, Pan, Measure, Label, Icon (**5**)
2. **Options**, `flex:1;min-width:0;overflow-x:auto` — a horizontally scrolling
   strip in four mutually exclusive forms.
3. **Undo / redo**, `flex:none`: two `var(--tap)` squares, `↶ ↷`, ink
   `var(--sec)` when the stack is non-empty and `var(--dis)` when it is empty.

| Form | Contents |
|---|---|
| `optBrush` (`sculpt` or `biome`) | `size` slider **120 px** wide in a `var(--tap)`-tall hit area; track `height:4px`, radius 2, `background:var(--ins)`, fill `var(--acc)`, thumb **14 × 14** circle in `var(--ink)`; readout `width:80px`. Then `hardness`: slider **84 px**, readout `width:34px`. Then a rule, then 8 shape chips (`round soft flat ridge noise square fan chisel`) at `min-height:var(--ctl)` = 36, `padding:0 11px`, `radius:999px` |
| `optFeat` (`sculpt` only) | a rule, then 6 feature chips — Cliff, Ridge, Valley, Plateau, Basin, Dune — at `padding:0 12px` |
| `optMeas` | 5 chips — Distance, Bearing, Area, Radius, Cross-section — at `padding:0 13px`; when Cross-section is live: a rule, a `field` label, and 5 field chips (Elevation, Terrain, Climate, Hydrology, Geology) at `padding:0 11px`; then `measHint` in `var(--dim)` |
| `optPlain` | one hint string in `var(--dis)`. `inspect` → `'tap samples · drag pans · two fingers or wheel zooms · long-press = right-click'`; `pan` → `'drag pans · pinch zooms'`; anything else → `'tap the map to place · hold a dock chip and tap to split-tap place'` |

Brush values: `size` 6–200 quantised to even numbers
(`Math.round((6+p*194)/2)*2`), displayed as `NN px · <km>`; `hard` 0–1 to two
decimals.

**Shell counterparts.**

- **Tool picker** ↔ the top tool bar's strip: `DccWidgets.tool_strip()`, drawn
  by `app.gd::_rebuild_tool_strip()` for the active domain on desktop and
  tablet (owner Ruling AK, 2026-09-23 — the phone keeps its dock `TOOLS`
  block). Tool set: `tool_bar.gd`'s `MODES` is `["sculpt", "paint",
  "measure"]` plus a globally registered `inspect`; `pan` is a viewport gesture,
  not a registered tool. The canvas's CIVIL tools (Settlement, POI, Territory,
  Way) are left-dock **categories** here (`civilization_workspace.gd`), not
  rail tools.
- **Options** ↔ `tool_options_row`, filled by each workspace's tool-options
  callback and, on tablet, wrapped in a `ScrollContainer` — the canvas's own
  `overflow-x:auto`, and the fix for the portrait overflow in §5.
- **Measure** is complete: `global_tools.gd::MEASURE_MODES` holds the canvas's
  five sub-modes (Distance, Bearing, Area, Radius, Cross-section) and
  `SECTION_CHANNELS` its five fields (Elevation, Terrain, Climate, Hydrology,
  Geology); `tool_bar.gd::_build_measure_tools()` draws them as one chip row —
  `MEASURE_GROUP_POINT` (`tool_bar.gd`) for the first four, then a
  `CROSS-SECTION` group, the canvas's own composition — and reasons its disabled
  states against `bridge.measure_api`.
- **Brush** ↔ `tool_bar.gd::_build_sculpt_options()`, writing through
  `bridge.sculpt_*`.
- **Undo / redo** exist as the menu bar's `↶`/`↷` squares
  (`DccShell._menu_undo_btn` / `_menu_redo_btn`), not in the rail.

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

**Shell:** `rail_column` (`ROLE["w_rail"]` 48 on tablet); `_rail_foot_stack` is
the foot slot and is already a rotated label. The three domains are the same
three. The cell differs in shape (38 × 44, caption under the glyph) and in the
selected wash (`--wash2` .15 against `accent_wash_2` .16).

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
`var(--m2)/1.6 var(--dis)`, then the stage's fields, then `Run stage NN`
(`var(--acc)` on `var(--accInk)`) beside `reset`. Footer: `Run all 10 stages`
with an inline progress fill (`rgba(0,0,0,.2)`), and a note.

**The field layout, read off the markup (grep `st.grid` and `st.sliders`):** the
2-column grid holds **tap-to-cycle toggle / segment pill cells**
(`min-height:var(--rowD)` = 44) and never a slider; **sliders are full-width
two-line cells** — label and value on one line, the track below. An earlier
brief read the grid as a grid of sliders and a build from it was reverted; brief
this body from the markup, not from a summary (`MISTAKES.md`, "Brief a layout
change from a spec document's summary").

**Stage names — Ruling E.** The canvas's ten are not the engine's.
`world_workspace.gd::STAGES` and `cartalith-engine/src/progress.rs::STAGE_NAMES`
are `Planet`, `Extent & scale`, `World structure`, `Tectonics`, `Volcanism &
impacts`, `Erosion`, `Hydrology`, `Climate`, `Ecology & biomes`, `Resources &
soils`. Three names are shared (`Erosion`, `Climate`, `Ecology & biomes`), three
are the same subject renamed (`Plates & tectonics`→`Tectonics`, `Rivers &
basins`→`Hydrology`, `Soils`→`Resources & soils`), four have no stage (`Uplift
& relief`, `Sea level`, `Prevailing wind`, `Ocean currents`), and four engine
stages are not drawn (`Planet`, `Extent & scale`, `World structure`, `Volcanism
& impacts`). **Per Ruling E, the tablet draws the engine's ten.**

**`Run stage NN` — Ruling D.** The engine has no partial recompute: one
`generate()` resolves all ten (`world_workspace.gd`'s own note above its
per-stage rows says the same). The button is drawn, runs the whole pipeline, and
says so in its tooltip.

The canvas's one-slider-per-stage model (`stageFields`, `SF`) is a
*reduction*: the WORLD dock draws the engine's own parameter table
(`crates/cartalith-godot/src/params.rs`, `PARAMS`), grouped by stage, and under
DS-03 none of it leaves.

**(b) `ldSculpt` — WORLD ▸ SCULPT** (lines 189–244). Root form: a
`FEATURE · PARAMETERS` title and two to three sliders per feature, from `FP` —
`cliff` (height 0–1200 m, run 0–400 px, plus a **left/right side** pair),
`ridge` (height, length), `valley` (depth, width), `plateau` (height, radius),
`basin` (depth 0–900 m, falloff 0–1), `dune` (height 0–120 m, spacing 0–200 px).
Then a stamp list, or an empty-state note. A drill state (`scDrill`) edits one
stamp.

**Shell:** the mode is `DccShell.select_domain_mode("world", …)`, and
`world_workspace.gd` builds `_sculpt_body` from `bridge.get_sculpt_features()` /
`get_sculpt_presets()` / `sculpt_get_globals()`. The engine's stamp registry
(`cartalith-terrain/src/sculpt.rs::Feature`, 13 keys: Mountains, Hills, Ridge,
Plateau, Cliff, Canyon, Valley, River, Lake, Basin, Coastline, Volcano,
Freehand) has five of the canvas's six by name; **`dune` has no engine
feature**, and each feature's sliders map onto that feature's own parameters,
not assumed ones. (Compare against this registry, not the freehand brush list.)

**(c) `ldCivCats` — CIVIL** (lines 245–353). **Four** collapsible categories:
`LANDMARKS`, `FACTIONS & SETTLEMENTS`, `WAYS & ROUTES`, `JOURNEY PLANNER`.
LANDMARKS has two levels — a family list (`LMFAMS()`, glyph plus
`N of M armed · P placed`), a crowding slider (0.25–2.0, displayed `× N.NN`,
with a live "a regional landmark keeps <km> clear" readout computed from
`radii.REG × crowd`), a `compete` toggle, a `Run landmark pass` button with a
percentage fill, and a per-family drill whose per-type caps step along a
**13-rung ladder** `[0,1,2,3,5,8,12,20,30,50,80,120,200]`. `JOURNEY PLANNER` is
a teaser card plus an `hOpenPlanner` that sets `s.scr = 'planner'`.

**Shell:** all four subjects exist, as a larger set of `civilization_workspace.gd`
categories built with `DccWidgets.category()`; `menus.gd`'s `Landmark types`
submenu prints the same `armed · placed` pair per family. DS-03 keeps every
category; the canvas's four are a grouping, not a cut.

**(d) `ldCartoCats` — CARTO** (lines 354–443). Four categories: `LAYERS &
STYLE` (a visibility list with per-layer opacity), `LABELS`, `ICONS`,
`TERRAIN APPEARANCE`. **Shell:** `cartography_workspace.gd`'s categories plus
`layers_popover.gd`.

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
here"*. Haptics: `navigator.vibrate` at `sample:12, arm:10, detent:8, place:14`
ms.

**Shell:** `viewport_host.gd` and the `tool_overlay.gd` family. The armed state
is drawn in the tool bar; the canvas's centred floating chip over the map has
no counterpart. **Haptics exist**: `DccShell._haptic(kind)` over `_HAPTIC_MS`
(transcribed from `design/dcc-environment-2026-08-31/BUILD_ANSWERS.md` §4:
sample 12, detent 8, tool arm 10,
verdict, back, blocked), gated on `OS.has_feature("mobile")` — so an Android
tablet gets them. The canvas's `place:14` has no `_HAPTIC_MS` entry.

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

**Shell — the closest match in the canvas.** `right_dock.gd`'s `CTX_SAMPLE` is
the default context, `SAMPLE_FIELDS` is the field list, every row is backed by
`bridge.sample_cell()`, and `SAMPLE_STAGE` dashes a row **with its stale
reason**. The canvas's `sampleData()` returns elevation, slope, aspect and a
biome string; the shell's sample is a superset.

**`rdMeasure`** — a headline block (`mBigLabel` in `.16em var(--faint)`, `mBig`
in `500 26px var(--acc)`, `mBigSub`), then for Cross-section a **300 px**
profile: a 44 px A/mid/B gutter, an SVG `viewBox="0 0 100 400"` with two
0.6-width grid lines at x = 33 and x = 66 and a `vector-effect="non-scaling-
stroke"` path, and a min/max row. Then per-segment rows, a stats block,
`subtract water` (area only), `clear`, and a footnote. The profile is computed
at **n = 120** samples along A→B.

**Shell:** `global_tools.gd`'s measure API; the cross-section profile is drawn
in the strip under the map (`section_strip.gd`), not in the right dock. The
canvas **moves** it into the right dock, so both ends move together.

### 2.7 Timeline

Collapsed (`tlShow`): a `--sbH` 28 bar — `▸` chevron, `TIMELINE` in
`var(--m2)/.18em var(--sec)`, and `tlSummary` = `'yr ' + year + ' · N of 6
layers armed'`. Open (`tlOpen`): **118 px** — title in `var(--acc)`, a
disclosure (*"not wired — these record which simulation layer you want; no layer
renders yet"*), six chips at `min-height:var(--ctl)` = 36, `padding:0 12px`,
`radius:999px`, then a `yr NNN` label of `width:52px` and a scrubber in a
`var(--tap)`-tall hit area.

**Shell — the layer list matches exactly.** `dcc_shell.gd`'s timeline-layer
array (grep `["Climate", true]`) carries `["Climate", true], ["Population",
true], ["Economy", false], ["Politics", true], ["Infrastructure", false],
["Warfare", false]` — the canvas's six, in the canvas's order;
`_tl_load_layers()` restores them before either composition builds a view.
Defaults differ: the canvas arms Climate and Population, the shell also arms
Politics. The shell's height role, `ROLE["h_timeline"]` (88 on tablet), was set
with no prototype figure to match; this canvas supplies two — 28 collapsed, 118
open.

### 2.8 Status bar — `--sbH:28`

`gap:14px`, `padding:0 12px`, `border-top:1px solid var(--hair)`,
`background:var(--pan)`, `font:var(--m2)`:

| Slot | Canvas | Ink |
|---|---|---|
| left | `proj + ' · ' + tool + ' · ' + w + '×' + h + ' dp'` | `var(--sec)` |
| centre | `'last pass 10 Ecology · 108 ms · repaint 84 ms · autosave ' + savedAt` | `var(--faint)` |
| right | `N landmarks · N stamps` | `var(--dis)` |

**Shell:** `status_row` (`ROLE["h_status"]` 36 — 8 px taller than the canvas).
The centre slot's timings are prototype constants; the shell's are measured.
The frame size in dp in the left slot is a prototype affordance, not a product
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

**Shell — a superset.** `journey_planner_view.gd` (`class_name
JourneyPlannerView`) is a whole-domain takeover built to
`JOURNEY_PLANNER_SPEC.md`'s "direction 1a (distance spine)": `_rebuild_profile()`,
`_rebuild_stops()`, `_rebuild_matrix()` with `_matrix_header()`,
`_matrix_clear_column()` and `_matrix_fill_down()`, `_build_verdict_card()`, and
`_build_time_group()` / `_build_load_group()` / `_build_supply_group()` /
`_build_cost_group()` — the canvas's four result groups, by name — plus vessels,
a trace group, presets, saved journeys and stage overrides the canvas does not
draw.

**The delta is the framing.** The canvas makes the planner a third top-level
screen reached from `Data ▸ Journey planner`; the shell arms it as a tool inside
CIVIL and swaps the domain region. Converting it means giving `DccShell` a
screen-level state **above** the domain layer, which it does not have.

---

## 4. Adoption: what each canvas item touches

### 4.1 Re-home, don't rebuild

Most of the canvas already has a counterpart, named under **Shell** in each §1–§3
entry: both rails, the measure and brush options, the right-dock Sample, the
timeline's six layers, the status bar, the planner (a superset), the landmark
families, the pipeline/sculpt switch, the menu rows (§2.1), the portrait dock
widths and haptics. Adopting those is layout and style work over existing code.

### 4.2 New work, and the constraint on each

| Item | Why the shell has no counterpart | What it touches |
|---|---|---|
| **Landscape docks at 320** | held at 400 until the dock content reflows to fit (§0.3) | the content first, then `ROLE`'s `w_left_dock` / `w_right_dock` together |
| **Portrait field layout** | the canvas's two-line slider cells and pill grid (§2.4a), at 232 px | `world_workspace.gd` and `DccWidgets`' row factories; the layout must be re-chosen on rotation, not only at build |
| **`scrPlanner` as a top-level screen** | `DccShell` has no state above the domain layer | `dcc_shell.gd`, `app.gd::_on_workspace_changed`, `journey_planner_view.gd`'s show/hide |
| **`scrPicker` as a screen** | the launcher is a `Window` | `open_project_dialog.gd`, `app.gd::open_welcome()` |
| **`LIVE` / `DRAFT` / `ARCHIVE` per world** | no project-state field exists on disk | `open_project_dialog.gd` plus the save format |
| **The tablet palette** | 12 dark and 15 light tokens differ, three structurally (opaque hairlines) | `dcc_theme.gd::DARK` + `LIGHT`, `rebuild_theme()`'s two-way remap, every contrast pair (§0.1) |
| **`--rPan:16`** | no panel-radius role (`--rCtl` is `ROLE["btn_radius"]`) | `dcc_theme.gd`, the `StyleBoxFlat` factories in `dcc_widgets.gd` |
| **Per-stage `Run stage NN`** | the engine has no partial recompute | Ruling D: draw it, run the whole pipeline, say so in the tooltip |
| **The canvas's stage names** | not `progress.rs::STAGE_NAMES` | Ruling E: not adopted |
| **Stylus pressure / palm rejection rows** | nothing in the shell reads pen pressure; the canvas reads `e.pressure` into `sc.brush` | `tool_bar.gd`, input handling |
| **Centred armed-tool chip over the map** | the armed state lives in the tool bar | `viewport_host.gd` |
| **Undo / redo in the rail** | they are menu-bar squares | `tool_bar.gd` or the rail; two buttons over commands that exist |
| **Cross-section profile in the right dock** | it is drawn in the strip under the map | `right_dock.gd` and `section_strip.gd` together |

### 4.3 The one item that was a defect fix, not a resemblance

The portrait dock split (232 / 232) was the only canvas figure that repaired
something broken rather than matching a drawing — and even it did not repair
the whole of it (§5).

---

## 5. Portrait: the overflow and its real cause

**Measured 2026-09-07**, `_tabspec_probe --force-touch`, docks at the then
tablet width of 400:

| Frame | rail | left dock | viewport | right dock | sum | frame w | overflow |
|---|---|---|---|---|---|---|---|
| 1280×800 | 47 | 400 | 432 | 400 | 1279 | 1280 | 0 |
| 1440×900 | 47 | 400 | 592 | 400 | 1439 | 1440 | 0 |
| 1600×1000 | 47 | 400 | 752 | 400 | 1599 | 1600 | 0 |
| 2560×1600 | 47 | 400 | 1712 | 400 | 2559 | 2560 | 0 |
| **800×1280** | 47 | 400 | **237** | 400 | **1084** | 800 | **+285 px** |
| **900×1440** | 47 | 400 | **237** | 400 | **1084** | 900 | **+185 px** |

At 800×1280 the SAMPLE column was cut at the frame edge, the menu bar's last
item sliced mid-word, and no map visible.

**The docks were not the cause.** That table reads the *result* of the layout.
`viewport_area` has no minimum of its own (`get_combined_minimum_size().x =
0.0`) and expands into whatever the row is given; **the binding constraint was
`tool_options_row` at a combined minimum of 1 057 px** — six touch-sized text
buttons — plus 28 px of bar margins, a shell floor of **1 085**. The canvas's
232 / 232 docks shipped (`TABLET_PORTRAIT`) and were canvas-correct, and the
shell was still 1 085 wide. **What cleared it was the canvas's own
`overflow-x:auto`**: the tablet options row now scrolls inside a
`ScrollContainer`. And landscape was fine only by being wide enough — a
1024×768 landscape tablet overflowed by 61 px on the same floor.

The rule this produced is `MISTAKES.md`'s "Read a layout that overflows the
screen": walk the tree for the ancestor whose minimum exceeds the frame, never
reason from the region sizes. Current figures are the probe's to report.

---

## 6. What could not be measured, and why

* **Whether `Cartalith Tablet.dc.html` is the design project's current head.**
  Imported by `DesignSync` on 2026-09-07 and not re-fetched since.
* **The canvas's rendered layout.** Everything here is read from the source,
  not from a live DOM; a rendered measurement needs `CAPTURE.md`'s CDP recipe.
  The declared values are what a shell is built against in any case.
* **What `--map` and `--tst` should map to.** Both are declared and drawn;
  neither has a `DccTheme` token. That is a decision, not a measurement.
* **The canvas at a size outside its six frames.** A real tablet at, say,
  1194 × 834 is outside every one; `isPortrait()` is the only rule that would
  apply.

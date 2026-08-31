# Cartalith DCC Environment — Desktop shell specification
## Right dock · Tool options bar · Status bar · Timeline bar · Viewport furniture · Tool palette

Source: `C:\Users\Vincent\Cartalith_GDT\design\dcc-environment-2026-08-31\Cartalith DCC Environment.dc.html`

---

## 0. BLOCKING DEFECT — the delivered desktop file is truncated

**The desktop prototype is exactly 262 144 bytes (2^18 = 256 KiB) and ends mid-statement.** The final byte sequence is:

```
      measRows.push({i:('0'+i).slice(-2),len:this.fmtKm(km),be
```

There is no closing `}`, no `</script>`, no `</body>`, and **no `renderVals()`**. The file ends partway through `valsCore()`, which begins at file line 2060.

What survives intact:
- The **entire markup** (file lines 1–1162) — every element, every inline style, every literal string, every binding name. Geometry and colour for my whole region are therefore fully recoverable.
- Logic classes `vals2()` … `vals6()` and the hook overrides — complete.
- `valsCore()` up to and including `statusMsg`, `keyHints`, and the first two lines of the `measRows` loop.

What is lost — **every value and handler in the tail of `valsCore()`**. Verified by grepping each binding name against the script region (file lines 1163→EOF); the following are referenced by the markup and defined nowhere in the delivered file:

| Region | Missing bindings |
|---|---|
| Right dock frame | `rdTitle`, `rdClosed`, `rdCollapsedLabel`, `hRdClose`, `hRdOpen` |
| Right dock · Sample | `rdSample`, `sampleOn`, `sampleOff`, `sampleCoords`, `sampleElev`, `sampleRows` |
| Right dock · Measure | `rdMeasure`, `measRows` (partially visible, see §1.6) |
| Right dock · Region | `rdRegion`, `regionRows` |
| Tool options bar | `tbLabel`, `tbInspect`, `inspChips`, `hInspChip`, `tbMeasure`, `measSegCol`, `measSegBg`, `measPathCol`, `measPathBg`, `hMeasMode`, `hMeasClear`, `tbRegion`, `regionReadout`, `hRegionExport`, `tbPipe`, `runStageLabel`, `hRunStage`, `runChainLabel`, `hRunChain`, `hDice`, `pipeNote`, `pipeNoteCol`, `genRunning`, `progTitle`, `progW`, `progPct`, `hFinalize`, `finLabel`, `finBord`, `finCol`, `finBg` |
| Status bar | `statusMid`, `statusKeys` |
| Timeline bar | `tlShow`, `tlCollapsed`, `tlExpanded`, `tlYearLabel`, `tlPlayGlyph`, `tlSpeeds`, `tlState`, `tlToggles`, `tlPct`, `hTlExpand`, `hTlCollapse`, `hTlPlay`, `hTlStep`, `hTlSpeed`, `hTlTog`, `hTlScrub` |
| Viewport furniture | `vpContext`, `vpField`, `scrimBg`, `mapCursor`, `layersBtnBg`, `layersBtnCol`, `hLayersBtn`, `masterOpPct`, `masterOpDisp`, `hMasterOp`, `layerRows`, `hLayerPick` |
| Frame chrome | `frames`, `hFrame`, `hTheme`, `themeLabel`, `fw`, `fh`, `scale`, `mb`, `fvars`, `footLabel`, plus every left-dock binding (`ldTitle`, `ldSwitch`, …) |

Three hooks are **defined and never called** — their only call sites were in the truncated tail: `tbToolMode(t)` (file line 1694), `tbLabelExtra(tb)` (1695), `vpCtxExtra()` (1697, returns `''`).

Everything below marked `UNSPECIFIED:` is a consequence of this truncation, not of a designer omission. **The file should be re-exported before build.** Where the markup, the state defaults, or the surviving hooks pin a value down anyway, I give it and say so.

Cross-check: `Cartalith Android.dc.html` in the same folder is **complete** (ends `</html>`) and shares **zero** binding names with the desktop file — `tlSpeeds`, `layerRows`, `sampleRows`, `inspChips`, `regionRows`, `statusMid`, `statusKeys`, `vpContext`, `vpField`, `scrimBg`, `masterOpPct`, `rdTitle`, `tlToggles` all return 0 matches. The phone build cannot fill these gaps.

---

## 0.1 Design tokens (complete — file line 25 and line 2063–2064)

Every measurement below is expressed in these tokens. **Dark is the default theme**; light is a full token swap.

**Dark (`:root` default)**

| Token | Value | Token | Value |
|---|---|---|---|
| `--sur` | `#0d0e0f` | `--hair` | `rgba(255,255,255,.10)` |
| `--pan` | `#121314` | `--div` | `rgba(255,255,255,.07)` |
| `--ins` | `#191c1e` | `--bor` | `rgba(255,255,255,.16)` |
| `--ink` | `#e8ebec` | `--wash` | `rgba(224,163,74,.09)` |
| `--body` | `#c8cbcd` | `--wash2` | `rgba(224,163,74,.16)` |
| `--sec` | `#a9adb0` | `--shadow` | `0 14px 34px rgba(0,0,0,.55)` |
| `--dim` | `#8d9296` | `--good` | `#6fae7d` |
| `--faint` | `#6f7478` | `--block` | `#c96a5a` |
| `--dis` | `#5f6468` | `--water` | `#6a9bc4` |
| `--acc` | `#e0a34a` | `--accH` | `#f0bd72` |
| `--accInk` | `#141005` | | |

**Light (applied as an override string when `state.light === true`)**

`--sur:#f4f2ee` `--pan:#fbfaf7` `--ins:#eceae4` `--ink:#111210` `--body:#23241f` `--sec:#3d3f39` `--dim:#6b6f6a` `--faint:#8d9088` `--dis:#9a9d95` `--acc:#a4650f` `--accH:#8a5309` `--accInk:#f7f4ee` `--hair:rgba(0,0,0,.14)` `--div:rgba(0,0,0,.08)` `--bor:rgba(0,0,0,.20)` `--wash:rgba(164,101,15,.09)` `--wash2:rgba(164,101,15,.16)` `--shadow:0 14px 34px rgba(35,36,31,.16)` `--good:#2c7a44` `--block:#a03d2e` `--water:#2e6a9e`

**Metrics**

| Token | 1920 / default | 1366 override | Tablet (touch) |
|---|---|---|---|
| `--fs` | `11.5px` | — | `14px` |
| `--m0` | `10.5px` | — | `12.5px` |
| `--m1` | `10px` | — | `12px` |
| `--m2` | `9px` | — | `11px` |
| `--menuH` | `36px` | — | `52px` |
| `--tbH` | `40px` | — | `56px` |
| `--railW` | `40px` | — | `48px` |
| `--ctl` | `24px` | — | `36px` |
| `--btnH` | `28px` | — | `44px` |
| `--row` | `28px` | — | `44px` |
| `--tool` | `30px` | — | `44px` |
| `--ldW` | `372px` | `330px` | `400px` |
| `--rdW` | **`304px`** | **`280px`** | **`400px`** |
| `--sbH` | **`26px`** | — | **`36px`** |
| `--pad` | `14px` | — | `16px` |
| `--g` | `10px` | — | `12px` |
| `--pop` | `300px` | `280px` | `380px` |

Frames: `w1920` 1920×1080, `w1366` 1366×768, `tabL` 2560×1600, `tabP` 1600×2560. `isTouch()` is true for `tabL`/`tabP` only.

**Typography.** Body text: `'Helvetica Neue', Helvetica, Arial, sans-serif` at `var(--fs)`, line-height `1.45`. All numerals, labels, section titles and readouts: `'IBM Plex Mono', monospace` at `var(--m0)`, `var(--m1)` or `var(--m2)`. Loaded from Google Fonts, weights 400 and 500.

**Repeated control geometry** (used verbatim throughout my region):

| Control | Spec |
|---|---|
| Slider hit area | `height:var(--ctl)`, `touch-action:none`, width given per instance |
| Slider track | `height:4px; border-radius:2px; background:var(--ins)` |
| Slider fill | `background:var(--acc)`, `width:{pct}` |
| Slider thumb | `12×12px; border-radius:50%; background:var(--ink); top:-4px; margin-left:-6px; left:{pct}` |
| Pill chip | `min-height:var(--ctl); padding:2px 12px; border-radius:999px; font:var(--m1) mono` |
| Segmented group | outer `background:var(--ins); border-radius:999px; padding:2px`; items `padding:3px 10–12px; border-radius:999px` |
| Primary button | `min-height:var(--btnH); padding:4px 15px; border-radius:8px; background:var(--acc); color:var(--accInk); font-weight:600` |
| Secondary button | `min-height:var(--btnH); padding:4px 13px; border-radius:8px; background:var(--ins); color:var(--sec)` |
| Square icon button | `width:height:var(--ctl); border-radius:8px; background:var(--ins); display:grid; place-items:center` |
| Toggle switch | track `background: on ? var(--acc) : var(--sur)`, knob `translateX: on ? 15 : 2` (px) |

**World constants.** Map is 4096 × 4096 cells; **1 cell = 2.5 km** (every `*2.5` in the file). `view.s` default `0.34`, centred `cx:2048, cy:2048`; clamped `0.12 … 4`.

---

## 1. RIGHT DOCK

### 1.1 Frame (markup file lines 869–880, 1096–1105)

| Property | Value |
|---|---|
| Width | `var(--rdW)` — 304px @1920, 280px @1366, 400px tablet |
| Border | `border-left: 1px solid var(--hair)` |
| Background | `var(--pan)` |
| Layout | `display:flex; flex-direction:column; min-height:0` |
| Header | `flex:none; display:flex; align-items:center; gap:8px; padding:8px var(--pad) 6px` |
| Header — close button | glyph **`›`**, `var(--ctl)` square, `border-radius:8px`, `color:var(--faint)`, hover `background:var(--ins)`. Handler `hRdClose`. |
| Header — title | `{{ rdTitle }}`, `font:var(--m2) 'IBM Plex Mono'`, `letter-spacing:.2em`, `color:var(--faint)`, `flex:1`, **`text-align:right`** |
| Body | `flex:1; overflow-y:auto; min-height:0; padding:0 var(--pad) 14px` |

**Collapsed state** (`rdClosed`): replaced by a `var(--railW)` (40px) strip, `border-left:1px solid var(--hair)`, `background:var(--pan)`, `align-items:center; gap:12px; padding:8px 0`, cursor pointer, handler `hRdOpen`. Contents: glyph **`‹`** in `var(--faint)`, then `{{ rdCollapsedLabel }}` rendered `writing-mode:vertical-rl; transform:rotate(180deg); font:var(--m2) mono; letter-spacing:.18em; color:var(--acc)`.

`state.rdOpen` default `true`. Window ▸ *Reset layout* sets `rdOpen: frame !== 'tabP'`.

`UNSPECIFIED:` `rdTitle` for the three core contexts (Sample, Measure, Region), and `rdCollapsedLabel` in every context. The extra-mode titles **are** specified — see §1.3.

### 1.2 How the context is chosen

Two independent mechanisms, both partly recoverable.

**(a) Core contexts** — flags `rdSample`, `rdMeasure`, `rdRegion`. `UNSPECIFIED:` their conditions. From the markup's `hint-placeholder-val` defaults, `rdSample` is the default-true branch and the other two default false; from the tap handlers (file lines 1928–1932) `sample` is written by the Inspect tool and `measure.pts` by the Measure tool, and both call `openRd()`.

**(b) Extra contexts** — a fully specified three-level fall-through, `rdExtraMode()` (file line 1696) → `rdExtraMode3()` (1650) → `rdMode4()` (1495). **First match wins:**

| Order | Condition | Mode |
|---|---|---|
| 1 | `tool` is `sculpt` or `freehand` | `stamps` |
| 2 | `tool === 'biome'` | `paint` |
| 3 | `tool` is `label` or `icon` | `anno` |
| 4 | `tool === 'territory'` | `terr` |
| 5 | `tool === 'settlement'` **or** (`domain==='CIVIL'` && `cv.sel >= 0` && `tool==='inspect'`) | `place` |
| 6 | `domain === 'CARTO'` && `tool === 'inspect'` | `stops` |
| 7 | (`tool==='way'` \|\| `tool==='route'`) && `wy().draft.length > 0` | `way` |
| 8 | `domain==='CIVIL'` && `civCat==='planner'` && `tool` ∈ {`inspect`,`pan`} | `plan` |
| — | otherwise | `null` |

**Auto-open triggers** (`openRd()` sets `rdOpen:true`; on frame `tabP` it also closes the left dock):
- Inspect tap on map, when `rdOpen` is false
- Measure tap on map, when `rdOpen` is false
- Label tool: tap on empty ground (creates) **or** tap within `30 / view.s` world units of an existing label (selects)
- Settlement tool: tap drops a place
- Clicking a row in the CIVIL settlement list (`hCivPlaceSel`)

**Defect:** completing a Region marquee drag sets `rdOpenForce: 1` (file line 1923). `rdOpenForce` appears exactly once in the whole file — **nothing reads it**. The region drag therefore does not open the dock.

### 1.3 Right dock titles for the extra contexts (`rdTitleExtra`, file line 1696)

| Mode | Title string |
|---|---|
| `stamps` | `STAMP STACK` |
| `paint` | `PAINT · ` + `bp().target.toUpperCase()` → `PAINT · BIOME` / `PAINT · SOIL` / `PAINT · VEGETATION` |
| `anno` | `ANNOTATION` |
| `place` | `SETTLEMENT` |
| `terr` | `TERRITORY` |
| `stops` | `RAMP · STOPS` |
| `way` | `WAY` |
| `route` | `ROUTE` |
| `plan` | `JOURNEY — RESULTS` |

**Dead entry:** `route` is in the title map but `rdMode4()` returns `'way'` for both the Way and Route tools, so `ROUTE` is unreachable.

### 1.4 Context — SAMPLE (`rdSample`) · markup 876–892

Two sub-states.

**`sampleOn`** — header block: `display:flex; flex-direction:column; gap:2px; padding-bottom:10px; border-bottom:1px solid var(--div); margin-bottom:9px`

| Element | Binding | Style |
|---|---|---|
| Coordinate line | `{{ sampleCoords }}` | `font:var(--m2) mono; color:var(--faint)` |
| Elevation hero | `{{ sampleElev }}` | `font:500 26px 'IBM Plex Mono'; color:var(--acc)` |

Then a repeated key/value list, `hint-placeholder-count="12"` (≈12 rows expected):
`display:flex; align-items:baseline; gap:10px; min-height:calc(var(--row) - 6px)`
- key `{{ r.k }}` — `flex:1; font:var(--m2) mono; letter-spacing:.1em; color:var(--faint)`
- value `{{ r.v }}` — `font:var(--m1) mono; color:{{ r.col }}; text-align:right`

Footnote, verbatim: **`fields owned by stale stages read —`** at `padding-top:9px; font:var(--m2)/1.6 mono; color:var(--dis)`

**`sampleOff`** — `padding-top:26px; text-align:center; font:var(--m1)/1.8 mono; color:var(--dis)`, two lines:
```
no sample
click the map with Inspect armed
```

`UNSPECIFIED:` the row labels, order, formatting and per-row colour of `sampleRows`, and the exact composition of `sampleCoords` / `sampleElev`. The **data available** is fully pinned by `sampleData(x,y)` (file line 1904), which returns exactly these fields — a builder must choose labels for them, or the file must be re-exported:

| Field | Type / formatting in the source |
|---|---|
| `elev` | integer metres, range `−410 … 4210` |
| `slope` | string, one decimal, `0.0 … 32.0` (degrees) |
| `aspect` | integer `0 … 360` |
| `plate` | `P-1` … `P-14` |
| `ptype` | `convergent` \| `divergent` |
| `bdist` | integer `0 … 900` (distance to plate boundary) |
| `resist` | string, two decimals, `0.20 … 0.90` |
| `lith` | `granite` \| `basalt` \| `sediment` \| `schist` |
| `temp` | integer °C, latitude-driven |
| `precip` | integer `0 … 2200` mm |
| `drain` | `ocean` \| `endorheic` \| `river 3` \| `river 5` |
| `biome` | `steppe` \| `temperate forest` \| `taiga` \| `desert` \| `tundra` \| `rainforest`, or `ocean` when `elev <= 0` |
| `soil` | string, one decimal, `0.0 … 3.2` |
| `land` | boolean, `elev > 0` |

### 1.5 Context — MEASURE (`rdMeasure`) · markup 893–909

Header block (same box model as Sample):

| Element | Binding | Style |
|---|---|---|
| Label | `{{ measBigLabel }}` | `font:var(--m2) mono; color:var(--faint)` |
| Hero value | `{{ measTotal }}` | `font:500 26px mono; color:var(--acc)` |
| Sub-line | `{{ measBigSub }}` | `font:var(--m2) mono; color:var(--faint)` |

**All three are fully specified in `vals5()` (file line 1356 ff.):**

| `measure.sub` | `measBigLabel` | `measTotal` | `measBigSub` |
|---|---|---|---|
| `distance` | `TOTAL LENGTH` | `fmtKm(Σ segment km)` | `{n−1} segments · {n} points · great circle` |
| `bearing` | `BEARING A → B` | `NNN°` zero-padded to 3, else `—` | `` (empty) |
| `area` | `AREA · PROJECTED` | `{n} km²` `toLocaleString('en-US')`, × `0.88` when *subtract water* is on; `—` with < 3 points | `water subtracted` or `projected` |
| `radius` | `RADIUS` | `fmtKm(|AB|)`, else `—` | `` (empty) |
| `section` | `SECTION · ` + field name upper-cased | `fmtKm(section length)`, else `—` | `A → B locked · {field lower-cased}` |

**Segment list** — shown when `measShowSegs` (true for `distance` and `bearing`). Row: `display:flex; align-items:baseline; gap:10px; min-height:calc(var(--row) - 6px)`
- `{{ r.i }}` — `width:22px; font:var(--m2) mono; color:var(--faint)`
- `{{ r.len }}` — `flex:1; font:var(--m1) mono; color:var(--sec)`
- `{{ r.bear }}` — `font:var(--m1) mono; color:var(--dim)`

Partially recoverable from the truncated tail: `i` = `('0'+i).slice(-2)` (two-digit, 1-based); `len` = `this.fmtKm(km)` where `km = hypot(Δx,Δy) * 2.5`. `bear` is computed one line earlier as `Math.round((Math.atan2(b.x−a.x, −(b.y−a.y)) * 180/π + 360) % 360)`. `UNSPECIFIED:` how `bear` is formatted into the row (the file cuts off at `be`).

**Stat rows** (`measStatRows`) — same row geometry as Sample rows. Fully specified in `vals5`; key strings verbatim, all values `var(--sec)`:

| Sub-mode | Condition | Rows (key → value) |
|---|---|---|
| `bearing` | ≥ 2 pts | `BACK-BEARING` → `↺ NNN°` · `LENGTH` → `fmtKm` · `Δ ELEVATION` → `{elevB − elevA} m` |
| `area` | ≥ 3 pts | `TRUE SURFACE` → `area × 1.036 km²` · `WATER SUBTRACTED` → `−{area × 0.12} km²` *(only when water toggle on)* · `PERIMETER` → `fmtKm(path + closing segment)` · `VERTICES` → point count |
| `radius` | ≥ 2 pts | `DIAMETER` → `fmtKm(r×2)` · `CIRCUMFERENCE` → `fmtKm(2πr)` · `ENCLOSED AREA` → `πr² km²` |
| `section` | profile exists | `MIN · MAX` → `{min} · {max}{unit}` · `MEAN` → `{mean}{unit}` · `BEARING` → `NNN°` · `SAMPLES` → `120 · 1 per {fmtKm(len/120)}` |

**Delta line** (`measDelta`), `padding-top:9px; font:var(--m2)/1.7 mono; color:var(--faint)`. Only for `distance` with ≥ 3 points:
`straight line {fmtKm} — along path +{N}%`

**Hint line** (`measRdHint`), `padding-top:6px; font:var(--m2)/1.6 mono; color:var(--dis)`. Verbatim per sub-mode:

| Sub-mode | Hint |
|---|---|
| `distance` | `click drops points · double-click or Esc ends` |
| `bearing` | `two points · A then B` |
| `area` | `click vertices · double-click closes the ring` |
| `radius` | `click the centre, then the edge` |
| `section` | `click A, then B — profile reads below the map` |

Point caps (`measCap`): `bearing`, `radius`, `section` → **2 points**; `distance`, `area` → **99**. On tap, if the run is `done` or at cap, the point list restarts from the new point.

### 1.6 Context — REGION (`rdRegion`) · markup 910–918

Column, `gap:6px`. Rows: `display:flex; align-items:baseline; gap:10px`, key `flex:1; font:var(--m2) mono; color:var(--faint)`, value `font:var(--m1) mono; color:var(--sec)`. `hint-placeholder-count="5"`.

Then a **full-width primary button**, `margin-top:6px; min-height:var(--btnH); border-radius:8px; background:var(--acc); color:var(--accInk); font-weight:600; justify-content:center`, label verbatim:

> **`Send to Data ▸ Export`**

Handler `hRegionExport` (shared with the tool options bar's *Use as export extent*).

Footnote, verbatim, `font:var(--m2)/1.6 mono; color:var(--dis)`:

> `the marquee and the export route are two views of one rect`

`UNSPECIFIED:` the five `regionRows` key/value pairs. Available data: `state.region = {x, y, w, h}` in world cells; 1 cell = 2.5 km.

### 1.7 Context — STAMP STACK (`rdStamps`) · markup 919–960

Header row: `display:flex; align-items:center; gap:6px; padding-bottom:10px; border-bottom:1px solid var(--div); margin-bottom:9px`
- `{{ stampCount }}` — `font:500 20px mono; color:var(--acc)`
- literal **`DRAFT STAMPS`** — `font:var(--m2) mono; color:var(--faint)`
- spacer
- undo **`↶`** and redo **`↷`**, each `var(--ctl)` square, radius 8, `background:var(--ins)`, `color:var(--sec)` → `hScUndo` / `hScRedo`

**Empty state** (`stampEmpty`), `padding:18px 0; text-align:center; font:var(--m1)/1.8 mono; color:var(--dis)`, three lines verbatim:
```
no stamps yet
stroke the map — nothing touches the
heightfield until commit
```

**Rows** — newest first (`stampRows` is `.reverse()`d), each wrapped in `border-bottom:1px solid var(--div)`.
Row: `min-height:var(--row); gap:8px; padding:5px 0`, background `var(--wash)` when selected.

| Slot | Content | Style |
|---|---|---|
| Index | `('0'+(count−i)).slice(-2)` — two-digit, counts down | `width:20px; font:var(--m2) mono; color:var(--faint); text-align:right` |
| Visibility dot | click toggles `vis` | `9×9px; border-radius:50%; background: vis ? var(--acc) : transparent; border:1px solid var(--bor)` |
| Label | stamp `type` (e.g. `mountains`, `cliff`, `freehand · raise`) | `font:var(--m1) mono; color: vis ? var(--body) : var(--dis)` |
| Meta | radial: `r {round(r*2.5)} km`; stroke: `fmtKm(len)` | `font:var(--m2) mono; color:var(--dim)` |

**Selected-row drawer** (`st.sel`), `padding:2px 0 9px 28px; gap:6px`:
- Detail line: `brush {bs} px · {k} {v} · {k} {v} …` (every feature parameter) — `font:var(--m2)/1.6 mono; color:var(--faint)`
- Button strip, `gap:4px; flex-wrap:wrap`, each `padding:2px 9px; border-radius:8px; background:var(--ins); font:var(--m1) mono`:

| Label (verbatim) | Colour | Action |
|---|---|---|
| `▲ up` | `var(--sec)` | swap with index −1 |
| `▼ down` | `var(--sec)` | swap with index +1 |
| `hide` / `show` | `var(--sec)` | toggle `vis` |
| `⇅ flip high side · {left\|right}` | `var(--acc)` | **cliff stamps only**; toasts `High side flipped — now {side} of the stroke` |
| `✕ delete` | `var(--block)` | removes stamp, pushes undo `delete stamp` |

**Footer**, `display:flex; gap:6px; padding-top:12px`:
- **`✓ Commit to map`** — `flex:1`, primary button, centred. On empty: toast `Nothing to commit — the draft is empty`. On success: clears the stack and toasts `Committed {n} stamp(s) — erosion, hydrology, climate re-run once`.
- **`Discard`** — `min-height:var(--btnH); padding:0 14px; border-radius:8px; background:var(--ins); color:var(--sec)`. Pushes undo `discard draft`, toasts `Draft discarded — {n} stamp(s) dropped`.

Footnote, verbatim: `commit bakes the stack in one pass and re-runs erosion, hydrology and climate once`

### 1.8 Context — PAINT (`rdPaint`) · markup 961–979

Header: label **`PAINTED · UNCOMMITTED`** (`var(--m2)` mono, `var(--faint)`), hero `{{ paintCount }}` (`font:500 26px mono; color:var(--acc)`), formatted `{n} cells` with `en-US` grouping.

**Legend rows** (`hint-placeholder-count="6"`), `min-height:var(--row); gap:9px; padding:2px 4px`, background `var(--wash)` when it is the armed value. Click arms that value.
- swatch `10×10px; border-radius:3px`
- value name, `color: armed ? var(--acc) : var(--body)`
- spacer
- per-value painted count, `font:var(--m2) mono; color:var(--dim)`

Legend contents by target (exact names and hexes, `bpLegend()` file line 1692):

| `biome` | `soil` | `vegetation` |
|---|---|---|
| `temperate forest` `#5d8a5f` | `shallow` `#b0977a` | `dense` `#4f7a52` |
| `steppe` `#a8a06a` | `loam` `#8a7355` | `open` `#7a9a6a` |
| `taiga` `#4f7a6a` | `deep` `#6b5a42` | `sparse` `#a8ad8a` |
| `desert` `#c2a36b` | `alluvial` `#9a8a6a` | `barren` `#b8b0a0` |
| `tundra` `#8fa3a8` | | |
| `rainforest` `#3f7a52` | | |

Footer: **`✓ Commit`** (primary, `flex:1`) + **`Discard`**. Commit on empty toasts `Nothing painted yet`; on success marks stage 9 stale and toasts `Committed {n} cells — stages 09 and 10 stale`.

Footnote, verbatim: `painted biome overrides classification for covered cells — commit marks stages 09 and 10 stale`

### 1.9 Context — RAMP · STOPS (`rdStops`) · markup 980–1019

Column, `gap:10px`.

1. **Ramp bar** — `height:26px; border-radius:8px; background:{{ caRampBg }}` (a CSS `linear-gradient`, see the ramp table below). Handler `hStopBar` is a **no-op** (`e=>{}`) — clicking the bar itself does nothing.
2. **Stop handles**, absolutely positioned at `left:{pct}`, `top:-4px; bottom:-4px; width:10px; margin-left:-5px`. Inner swatch `border-radius:3px; background:{stop colour}; border:2px solid {selected ? var(--acc) : var(--pan)}`. Position `pct = (e − eMin)/(eMax − eMin)` with **`eMin = −410`, `eMax = 4210`** metres.
3. **Stop buttons**, `gap:4px`, each `padding:3px 11px; border-radius:8px; background:var(--ins); color:var(--sec); font:var(--m1) mono`:
   - **`＋ add`** — inserts a stop midway to the next one (or to `eMax`), colour `#8a8a72`, interp `linear`, and selects it
   - **`delete`** — removes the selected stop; refuses below 2 stops with toast `A ramp needs at least two stops`
   - **`reverse`** — mirrors every stop's elevation about the range and reverses order
4. **Selected-stop editor**, `border-top:1px solid var(--div); padding-top:9px; gap:8px`:
   - Swatch `18×18px; border-radius:5px; border:1px solid var(--bor)` + hex `{{ selStopHex }}` (upper-cased) `font:500 var(--m0) mono; color:var(--ink)` + spacer + elevation `{{ selStopElev }}` (`{n} m`, `en-US` grouped) `font:var(--m1) mono; color:var(--acc)`
   - Row **`elevation`** — label `width:72px; color:var(--sec)`, slider `flex:1`, maps 0→1 onto `−410 … 4210` m
   - Row **`hue`** — label `width:72px`, track is a fixed rainbow `linear-gradient(90deg,#c33,#cc3,#3c3,#3cc,#33c,#c3c,#c33)`, `height:8px; border-radius:4px`; thumb `12×14px; border-radius:4px; background:var(--ink); top:-3px`. Thumb position is hard-wired `selStopHuePct: '50%'` — **it never tracks the stop's actual hue.** Dragging writes a colour via an HSL→hex conversion at fixed S/L (`c = 0.62 − 0.34·…`).
   - Row **`interp`** — label `width:72px`, segmented pill with exactly three options: **`linear`**, **`ease`**, **`step`**
5. **Footer**, `border-top:1px solid var(--div); padding-top:10px; gap:6px`, both `flex:1`, `min-height:var(--btnH)`, centred:
   - **`Compare`** — secondary (`background:var(--ins); color:var(--sec)`). Toast: `Compare — holds the previous ramp for A/B (mock)`
   - **`Apply`** — primary. Toast: `Ramp applied — repaint 84 ms`

Default stops (5): `−410 #1d3140 linear` · `120 #2e5a4a linear` · `2640 #B9A878 ease` · `3600 #d8cdb0 linear` · `4210 #efe9dd step`. Selected index defaults to `2`.

Ramp gradients available (`RAMPS()`, file line 1523) — the same nine feed the left dock's ramp picker and this bar's background:

| Name | `linear-gradient(90deg, …)` |
|---|---|
| `Earth` | `#1d3140,#2e5a4a,#7a8a55,#b9a878,#d8cdb0,#efe9dd` |
| `Elevation` | `#22304a,#2d6a58,#a8a06a,#b06a42,#8a4a3a,#f0ece4` |
| `Atlas` | `#2a3140,#4a6a5a,#9aa571,#c9b789,#e5dcc4` |
| `Mono` | `#14161a,#3a3f44,#7a8188,#b8bdc2,#eceff2` |
| `Imhof` | `#5a6a7a,#8a9a8a,#c2b494,#e0cfa4,#f2e8cf` |
| `Ice` | `#2a4a6a,#5a8aaa,#9ac2d8,#d0e5ef,#f4fafd` |
| `Dark ice` | `#10202f,#26445c,#4a7690,#84aec4,#c6dfec` |
| `Desert` | `#5a4432,#8a6a44,#b8905e,#d8b884,#efdcb4` |
| `Dark atlas` | `#10141a,#243230,#4a5540,#7a744f,#a89a6a` |

### 1.10 Context — ANNOTATION (`rdAnno`) · markup 1020–1038

Column, `gap:9px`.

**When a label is selected** (`annoSelOn`):
- Section title **`SELECTED LABEL`** — `font:var(--m2) mono; letter-spacing:.14em; color:var(--faint)`
- Text `<input>` — `width:100%; box-sizing:border-box; background:var(--ins); border:none; border-radius:8px; min-height:var(--ctl); padding:4px 11px; color:var(--ink); font:var(--fs) 'Helvetica Neue'; outline:none`. Edits the label live.
- Row **`arc`** — label `width:72px; color:var(--sec)`, slider `flex:1`, readout `width:36px; text-align:right; font:var(--m1) mono; color:var(--ink)`. **Range −50 … +50**, integer; display is the raw integer.
- **`✕ delete label`** — `min-height:var(--ctl); padding:2px 12px; border-radius:8px; background:var(--ins); color:var(--block); align-self:flex-start`. Pushes undo `delete label`.

**Always:**
- Section title **`PLACED · THIS SESSION`** — same style, plus `border-top:1px solid var(--div); padding-top:9px`
- Row **`LABELS`** → `{{ annoLabelCount }}`
- Row **`ICONS`** → `{{ annoIconCount }}`
  (both: key `flex:1; var(--m2) mono; var(--faint)`, value `var(--m1) mono; var(--sec)`)
- **`clear all`** — `min-height:var(--ctl); padding:2px 12px; border-radius:8px; background:var(--ins); color:var(--sec); align-self:flex-start`. Pushes undo `clear annotation`, toasts `Labels and icons cleared`.
- Footnote, verbatim: `labels and icons are presentation — they add nothing to and take nothing from the world model`

### 1.11 Context — SETTLEMENT (`rdPlace`) · markup 1039–1052

Column, `gap:9px`.

- Name `<input>` — full width, `background:var(--ins); border-radius:8px; min-height:var(--ctl); padding:4px 11px; color:var(--ink); font:500 var(--fs) 'Helvetica Neue'`
- Eight read-only rows (`hint-placeholder-count="8"`), key `flex:1; var(--m2) mono; var(--faint)`, value `var(--m1) mono; var(--sec); text-align:right`. **Exact keys and values** (`placeRows`, file line 1701):

| Key | Value |
|---|---|
| `CLASS` | the settlement's class |
| `FACTION` | the owning faction name |
| `POPULATION` | `metropolis` → `120 000`, `city` → `38 000`, `town` → `6 400`, `village` → `820`, `hamlet` → `160` (fallback `6 400`) |
| `GOVERNMENT` | `council` |
| `AGRICULTURE` | `mixed arable` |
| `WATER ACCESS` | `river · good` |
| `DEFENSIBILITY` | `0.62` |
| `ROUTES` | `0 connected` |

*(the last six are static in the prototype)*
- Three chips, `gap:4px; flex-wrap:wrap`, each `padding:3px 11px; border-radius:999px; background:var(--ins); color:var(--sec); font:var(--m1) mono`: **`Economy`**, **`Politics`**, **`Logistics`**. Each toasts `{name} inspector — desktop-parity mock`.
- **`✕ delete`** — `background:var(--ins); color:var(--block); align-self:flex-start`. Pushes undo `delete settlement`.

Factions (`FACTIONS()`, file line 1589): `Vhal Serai Compact` `#6a9bc4` · `Kessan League` `#c96a5a` · `Free Marches` `#6fae7d`.

### 1.12 Context — TERRITORY (`rdTerr`) · markup 1053–1060

Column, `gap:9px`.
- Faction line: `12×12px; border-radius:3px` swatch + `{{ terrFaction }}` at `font:500 var(--m0) mono; color:var(--ink)`
- Row **`CLAIMED CELLS`** → `{{ terrCells }}` at `font:500 20px mono; color:var(--acc)`, `en-US` grouped
- Row **`AREA`** → `{{ terrArea }}` at `font:var(--m1) mono; color:var(--sec)`. Computed `fmtKm(cells * 2.5 * 2.5 / 1000)` with a trailing `²` appended — **so the unit renders as `km²` but the arithmetic divides by 1000, i.e. the figure is in thousands of km². Flag as a probable unit bug.**
- Footnote, verbatim: `contested cells: 0 · paint takes pointer capture and is LOD-aware`

### 1.13 Context — JOURNEY — RESULTS (`rdPlan`) · markup 1061–1086

Column, `gap:10px`.

**Verdict card** — `border:1px solid {{ planVerdictBord }}; border-radius:8px; padding:11px 12px; gap:4px`
- Verdict word `{{ planVerdict }}` — `font:var(--m2) mono; letter-spacing:.2em`, colour `planVerdictCol`
- Day count `{{ planDays }}` — `font:500 22px mono`, same colour, right-aligned
- `{{ planSplit }}` — `font:var(--m2) mono; color:var(--dim)`
- `{{ planReason }}` — `font:var(--m2)/1.6 mono; color:var(--sec); padding-top:2px`
- Action chips (`planActs`), `gap:4px; flex-wrap:wrap; padding-top:4px`, each `padding:3px 10px; border-radius:999px; border:1px solid var(--bor); color:var(--sec); font:var(--m2) mono`; hover `color:var(--acc); border-color:var(--acc)`

Two verdict states, both fully specified (`vals4()`, file line 1442). `blocked` = `season === 'winter' && closures === true`:

| | blocked | not blocked |
|---|---|---|
| `planVerdict` | `IMPOSSIBLE` | `STRAINED` |
| colour | `var(--block)` | `var(--acc)` |
| `planDays` | `— days` | `38 days` |
| `planSplit` | `no feasible calendar under current closures` | `29 travel · 6 rest · 3 layover` |
| `planReason` | `High Saddle is closed by seasonal closures in winter. Depart in another season, ignore closures, or re-route land-only.` | `Supply gap Thornwood → Lakemouth runs 9.4 effective days against 6 carried. Heavier foraging or +4 supply days resolves it.` |
| chips | `DEPART IN AUTUMN` · `IGNORE CLOSURES` | `+4 SUPPLY DAYS` · `HEAVY FORAGING` |

Each chip mutates the party form (`season='autumn'` / `closures=false` / `supplies+=4` capped at 30 / `forage='heavy'`) and toasts `Applied — full recompute is desktop-parity; verdict text is static here`.

**Result cards** — one per entry, `gap:5px; border-bottom:1px solid var(--div); padding-bottom:9px`
- Head: key `{{ c.k }}` (`var(--m2)` mono, `letter-spacing:.16em`, `var(--faint)`) + spacer + value `{{ c.v }}` (`font:500 var(--m0)` mono, colour `c.vCol`)
- Optional bar (`c.hasBar`): `height:4px; border-radius:2px; background:var(--ins)`, fill `background:{{ c.barCol }}; width:{{ c.barW }}`
- Sub-line `{{ c.sub }}` — `font:var(--m2)/1.55 mono; color:var(--dim)`

Exact card contents:

| `k` | `v` | bar | `sub` |
|---|---|---|---|
| `TIME` | `620 km · 21.4 km/d` (blocked: `—`) | no | `arrives day 38 · slowest stage High Saddle → Grey Vale` (blocked: `resolve the closure to schedule`) |
| `LOAD` | `1 840 / 2 210 kg` | yes, `83%`, `var(--acc)` | `capacity from 4 mules · 2 horses · 1 cart · party carry` |
| `SUPPLY REACH` | `9.4 d gap vs 6 carried` (blocked: `—`), colour `var(--block)` | yes, `100%`, `var(--block)` (blocked: `var(--ins)`) | `resupply at Kess Ford · Thornwood · Lakemouth` |
| `COST` | `412 cr` | no | `food 148 · fodder 96 · wages 118 · ferry 32 · tolls 12 · upkeep 6` |
| `VESSELS` | `river barge` | no | `last leg on water · capacity ok · auto-selected` |

Footnote, verbatim: `results are static in this prototype — the full v2.10 compute runs on desktop parity`

### 1.14 Context — WAY (`rdWay`) · markup 1087–1095

Four rows (key `flex:1; var(--m2)` mono `var(--faint)`; value right):

| Key | Value | Value style |
|---|---|---|
| `WAYPOINTS` | `{{ wayPts }}` — draft point count | `font:500 20px mono; color:var(--acc)` |
| `LENGTH` | `{{ wayLen }}` — `fmtKm(Σ|Δ| × 2.5)` | `var(--m1) mono; var(--sec)` |
| `GRADE · MAX` | **`4.2%`** — hard-coded literal in the markup | `var(--m1) mono; var(--sec)` |
| `SURFACE` | `{{ wayType }}` — `road` \| `track` \| `trail` \| `bridge` | `var(--m1) mono; var(--sec)` |

Footnote, verbatim: `Esc commits the way · hovering shows the live snap preview`

---

## 2. TOOL OPTIONS BAR

Two stacked bars. Both are `height:var(--tbH)` (40px desktop / 56px touch), `flex:none`, `display:flex; align-items:center; gap:10px; padding:0 var(--pad)`, `border-bottom:1px solid var(--hair)`.

### 2.1 Bar A — the horizontal rail (conditional, sits **above** bar B) · markup 109–148

`z-index:51`. Shown when `railShow` = `screen === 'app' && (tool ∈ {sculpt, freehand, biome} || tool === 'measure')`.

**Group switcher** (always, `display:flex; gap:3px`) — three pills, `min-height:var(--ctl); padding:2px 13px; border-radius:8px; gap:6px; font:var(--m1) mono; letter-spacing:.1em`. Active: `background:var(--acc); color:var(--accInk)`; inactive: `background:var(--ins); color:var(--sec)`. Each carries a 12×12 inline SVG (`viewBox 0 0 14 14`, `stroke-width 1.2`, round caps/joins) followed by its id text:

| Id (displayed verbatim) | Active when | Click arms | SVG path data |
|---|---|---|---|
| `SCULPT` | tool is `sculpt` or `freehand` | `sculpt` | `M1.5 11.5 8 5M9 2l3 3-2.6 2.6-3-3z` |
| `PAINT` | tool is `biome` | `biome` | `M2 12h4l6-6-3-3-6 6z` |
| `MEASURE` | tool is `measure` | `measure` | `M1.5 12.5 12.5 1.5` + two `r=1.3` filled circles at `(2,12)` and `(12,2)` |

Then a `1px × 20px` divider in `var(--div)`.

**Measure branch** (`railMeasure`, i.e. `tool === 'measure'`):

1. Sub-mode chips, `gap:3px`, each `min-height:var(--ctl); padding:2px 12px; border-radius:999px; font:var(--m1) mono`. Selected `background:var(--wash2); color:var(--acc)`; else `background:var(--ins); color:var(--sec)`. Exact `id → label`:
   `distance → Distance` · `bearing → Bearing` · `area → Area` · `radius → Radius`
   Clicking clears the point list and `done` flag.
2. Divider.
3. Literal label **`CROSS-SECTION`** — `font:var(--m2) mono; letter-spacing:.14em`, colour `var(--acc)` when `sub === 'section'`, else `var(--faint)`.
4. Field chips, `gap:3px`, each `padding:2px 11px; border-radius:999px`, same colour rule as above but only lit when `sub === 'section'` **and** it is the current field. Five, displayed verbatim: **`Elevation`**, **`Terrain`**, **`Climate`**, **`Hydrology`**, **`Geology`**. Clicking any one *switches the sub-mode to `section`* and clears the points unless already in section.
   Field → sampled quantity and unit: `Elevation` → `max(elev,0)` ` m` · `Terrain` → `slope` `°` · `Climate` → `temp` ` °C` · `Hydrology` → `precip` ` mm` · `Geology` → `round(resist*100)` (no unit).
5. Divider.
6. A permanently **disabled** pill: `min-height:var(--ctl); padding:2px 11px; border-radius:999px; background:var(--ins); color:var(--dis)`, text verbatim **`Δ vertical · 3D only`**. No handler.
7. Spacer (`flex:1`).
8. Right-aligned note, `font:var(--m2) mono; color:var(--dis)`, verbatim:
   **`measurements answer how far · Inspect answers what is here`**

**Brush branch** (`railBrush`, i.e. `tool ∈ {sculpt, freehand, biome}`):

1. **`size`** — label `color:var(--faint); font:var(--m1) mono`; slider hit area **130px**; readout `font:var(--m1) mono; color:var(--ink); width:88px`.
   - Biome: fill `= (radius − 2)/28`, display `{radius} cells`, drag sets `radius = round(2 + p·28)` → **2 … 30**
   - Sculpt/freehand: fill `= (size − 6)/194`, display `{size} px · {fmtKm(size × 2.5)}`, drag sets `size = round((6 + p·194)/2) × 2` → **6 … 200, even values only**
2. **`hardness`** — present only when `tool !== 'biome'` (`railHardOn`). Slider **90px**, fill `hard × 100%`, readout `hard.toFixed(2)`. Range **0 … 1, step 0.01**.
3. Divider.
4. Literal label **`SHAPE`** — `font:var(--m2) mono; letter-spacing:.14em; color:var(--faint)`
5. Eight shape chips, `gap:3px; flex-wrap:wrap`, each `min-height:var(--ctl); padding:2px 10px; border-radius:999px`, selected `background:var(--wash2); color:var(--acc)` else `background:var(--ins); color:var(--sec)`. Verbatim, in order:
   **`circle`**, **`directional`**, **`spatter`**, **`spiral`**, **`dots`**, **`cloud`**, **`checker`**, **`hatch`**
6. Spacer.
7. Note, `font:var(--m2) mono; color:var(--dis)`:
   - biome: **`shape is shared with sculpt · ⇧ erases`**
   - else: **`shape + falloff detail in the dock below`**

### 2.2 Bar B — the main tool options bar · markup 149–277

`z-index:50`. Always present on the app screen.

**Leading element, in every context:** `{{ tbLabel }}` — `font:var(--m1) 'IBM Plex Mono'; letter-spacing:.16em; color:var(--acc); flex:none`.

`UNSPECIFIED:` the `tbLabel` string for most contexts. Only the sculpt/paint forms survive, via `tbLabelExtra()` (file line 1695):
- sculpt: `SCULPT · ` + (freehand tool → `sc.free.toUpperCase()`, e.g. `SCULPT · RAISE`; else `sc.feature.toUpperCase()`, e.g. `SCULPT · MOUNTAINS`)
- biome: `PAINT · ` + `bp.target.toUpperCase()` → `PAINT · BIOME` / `PAINT · SOIL` / `PAINT · VEGETATION`

Contexts render as sibling `sc-if` blocks in this markup order. Conditions marked ✔ are defined in the delivered file; ✖ are truncated.

| # | Flag | Condition |
|---|---|---|
| 1 | `tbInspect` | ✖ |
| 2 | `tbMeasure` | ✖ |
| 3 | `tbRegion` | ✖ |
| 4 | `tbPipe` | ✖ |
| 5 | `tbSculpt` | ✔ `tool === 'sculpt' \|\| tool === 'freehand'` |
| 6 | `tbBiome` | ✔ `tool === 'biome'` |
| 7 | `tbCarto` | ✔ `domain === 'CARTO' && tool === 'inspect'` |
| 8 | `tbLabelRow` | ✔ `tool === 'label'` |
| 9 | `tbIconRow` | ✔ `tool === 'icon'` |
| 10 | `tbSettle` | ✔ `tool === 'settlement'` |
| 11 | `tbPoi` | ✔ `tool === 'poi'` |
| 12 | `tbTerr` | ✔ `tool === 'territory'` |
| 13 | `tbWay` | ✔ `tool === 'way' \|\| tool === 'route'` |
| 14 | `tbPlan` | ✔ `domain === 'CIVIL' && civCat === 'planner' && tool === 'inspect'` |

**Collision risk to resolve at build time:** with `tool === 'inspect'` and `domain === 'CARTO'`, both `tbCarto` (7) and — presumably — `tbInspect` (1) are true, and the markup would render both rows into one 40px bar. The same applies to `tbPlan` (14) in CIVIL. `UNSPECIFIED:` whether `tbInspect` excludes those two domains. **A builder must not guess this; re-export or ask.**

---

#### 2.2.1 INSPECT (`tbInspect`) — markup 151–157

| Slot | Content |
|---|---|
| Literal label | **`hit`** — `color:var(--faint); font:var(--m1) mono` |
| Chip row | `display:flex; gap:5px`, four chips (`hint-placeholder-count="4"`), each `min-height:var(--ctl); padding:2px 12px; border-radius:999px; font:var(--m1) mono`; text is the chip's own key. From `state.inspFilter` (file line 1203) the four keys are, verbatim: **`places`**, **`labels`**, **`icons`**, **`ways`** — all defaulting to `true`. Handler `hInspChip` (`data-k`). |
| Trailing hint | **`click selects · drag pans · wheel zooms`** — `color:var(--faint); font:var(--m1) mono` |

`UNSPECIFIED:` the on/off `bg`/`col` pair for these chips (`c.bg`, `c.col`). Every comparable chip in the file uses on = `var(--wash2)`/`var(--acc)`, off = `var(--ins)`/`var(--sec)` or `transparent`/`var(--dim)` — but which of the two idioms applies here is not stated. No commit/discard affordance in this row.

#### 2.2.2 MEASURE (`tbMeasure`) — markup 158–175

| Slot | Shown when | Content |
|---|---|---|
| Mode switch | `measIsDist` (`sub === 'distance'`) | Segmented pill `background:var(--ins); border-radius:999px; padding:2px`, two options `padding:3px 12px`: **`multi-segment`** (`data-v="segment"`) and **`point-to-point`** (`data-v="path"`). Handler `hMeasMode`. |
| Water toggle | `measIsArea` | Pill **`subtract water`**, `min-height:var(--ctl); padding:2px 12px; border-radius:999px`. On: `background:var(--wash2); color:var(--acc)`; off: `background:var(--ins); color:var(--dim)`. **Multiplies the reported area by 0.88 and adds the `WATER SUBTRACTED` stat row.** |
| Section info | `measIsSection` | `field · {{ measField }}` (`var(--m1)` mono, `var(--sec)`) then the literal **`120 samples · resample on zoom`** (`var(--faint)`) |
| Total | always | `{{ measTotal }}` — `font:500 var(--m0) mono; color:var(--ink)` |
| Clear | always | **`clear`** — `min-height:var(--ctl); padding:2px 12px; border-radius:8px; background:var(--ins); color:var(--sec)`. Handler `hMeasClear`. **This is the discard affordance; there is no commit.** |
| Hint | always | `{{ measHint }}` — `color:var(--faint); font:var(--m1) mono`; strings as in §1.5 |

`UNSPECIFIED:` `measSegCol`/`measSegBg`/`measPathCol`/`measPathBg` (which of the two mode options is lit and how) and the body of `hMeasMode` and `hMeasClear`. Note `state.measure.mode` defaults to **`'path'`** (file line 1202) — but the mode value is written nowhere else in the delivered file and `secSample()`/`vals5()` never read it, so **the multi-segment / point-to-point switch has no observable effect in the surviving code.**

#### 2.2.3 REGION (`tbRegion`) — markup 176–180

| Slot | Content |
|---|---|
| Readout | `{{ regionReadout }}` — `font:var(--m1) mono; color:var(--sec)` |
| Button | **`Use as export extent`** — `min-height:var(--ctl); padding:2px 12px; border-radius:8px; background:var(--ins); color:var(--sec)`. Handler `hRegionExport`, shared with the right dock's *Send to Data ▸ Export*. |
| Hint | **`drag a marquee · Esc clears`** — `color:var(--faint); font:var(--m1) mono` |

`UNSPECIFIED:` the composition of `regionReadout`.

#### 2.2.4 GENERATION PIPELINE (`tbPipe`) — markup 181–191

| Slot | Content |
|---|---|
| Primary button | `{{ runStageLabel }}` — `min-height:var(--btnH); padding:4px 14px; border-radius:8px; background:var(--acc); color:var(--accInk); font-weight:600`. Handler `hRunStage`. |
| Secondary button | `{{ runChainLabel }}` — same box, `background:var(--ins); color:var(--sec)`. Handler `hRunChain`. |
| Seed button | Dice SVG (13×13, `viewBox 0 0 16 16`, `rect x=2.5 y=2.5 w=11 h=11 rx=2` + four `r=0.7` filled dots at `(5.6,5.6) (10.4,10.4) (10.4,5.6) (5.6,10.4)`), then the literal text **`New seed`** (`white-space:nowrap`). Box: `min-height:var(--btnH); padding:4px 12px; border-radius:8px; background:var(--ins); gap:6px; color:var(--sec)`. Handler `hDice`. |
| Note | `{{ pipeNote }}` — `font:var(--m1) mono`, colour `{{ pipeNoteCol }}` |
| Spacer | `flex:1` |
| Progress (`genRunning`) | `gap:8px`: `{{ progTitle }}` (`var(--m1)` mono, `var(--acc)`), a **120 × 4px** track `border-radius:2px; background:var(--ins); overflow:hidden` with fill `background:var(--acc); width:{{ progW }}`, then `{{ progPct }}` (`var(--m1)` mono, `var(--sec)`) |
| Finalize | `{{ finLabel }}` — `min-height:var(--ctl); padding:2px 12px; border-radius:999px; gap:6px; font:var(--m1) mono; border:1px solid {{ finBord }}; color:{{ finCol }}; background:{{ finBg }}`. Handler `hFinalize`. |

`UNSPECIFIED:` every string and colour above (`runStageLabel`, `runChainLabel`, `pipeNote`, `pipeNoteCol`, `progTitle`, `progW`, `progPct`, `finLabel`, `finBord`, `finCol`, `finBg`, and the `hFinalize` action).

**Recoverable behaviour** from the surviving `runStages(from,to)` (file line 1949) and `state`:
- `state.run = {i, pct, to}`; ticks every **220 ms**, `pct += 16 + random()·24`
- Per-stage completion appends a log line `"{NN} {StageName} — resolved · {0.4–2.5} s"` in `var(--dim)`, log capped at 3 entries
- On chain completion: `lastRun` set to `HH:MM`; when `to >= 10`, `staleFrom` clears and `world.status` becomes `stages 01–10 resolved`
- Completion toast: `Stages {NN} → {NN} resolved` (chain) or `Stage {NN} resolved` (single)
- Default run origin: `from = staleFrom || openStage`; stale count = `10 − staleFrom + 1`
- Finalize semantics are only visible indirectly: `state.finalized` blocks `setField`, `runStages` and `armTool` for `biome, sculpt, freehand, settlement, poi, territory, way, route` (toast `World is finalized — {tool} is locked`), and the status bar shows `finalized — stages locked · atlas L0–L3 baked`

#### 2.2.5 SCULPT / FREEHAND (`tbSculpt`) — markup 192–199

| Slot | Content |
|---|---|
| `size` | Label `var(--faint)`; slider **110px**; fill `= (size − 6)/194`; readout `{size} px` at `var(--m1)` mono `var(--ink)`. Handler `hBrSlide` `data-k="size"`. |
| `intensity` | Label; slider **90px**; fill `= inten / 1.5`; readout `inten.toFixed(2)`. Range **0 … 1.5, step 0.05**. `data-k="inten"`. |
| Hint | `{{ tbSculptHint }}` — `color:var(--faint)`. Exactly **`tap places it`** for radial features (lake, volcano), **`drag strokes · draft only`** otherwise. |
| Spacer | `flex:1` |
| **Commit** | **`✓ Commit to map`** — primary, `padding:4px 15px`, `white-space:nowrap` |
| **Discard** | **`Discard draft`** — secondary, `padding:4px 13px`, `white-space:nowrap` |

Both buttons share their handlers with the right dock's Stamp Stack footer (§1.7).

#### 2.2.6 BIOME PAINT (`tbBiome`) — markup 200–211

| Slot | Content |
|---|---|
| Target switch | Segmented pill, three options `padding:3px 11px`: **`biome`**, **`soil`**, **`vegetation`**. Selecting one resets the value to that target's first legend entry (`biome → temperate forest`, `soil → loam`, `vegetation → open`). |
| Value cycler | Pill `min-height:var(--ctl); padding:2px 12px; border-radius:999px; gap:7px; background:var(--ins)` containing a `9×9px; border-radius:3px` swatch and the value name at `var(--m1)` mono `var(--ink)`. **Clicking cycles to the next legend entry** (`hPaintCycle`). |
| `radius` | Label; slider **90px**; fill `= (radius − 2)/28`; readout `{radius} c`. Range **2 … 30 cells**. |
| Erase | Pill **`erase ⇧`** — on `background:var(--wash2); color:var(--acc)`, off `background:var(--ins); color:var(--dim)` |
| Land mask | Pill **`land only`** — same on/off pair |
| Spacer | `flex:1` |
| **Commit** | **`✓ Commit`** — primary, `padding:4px 15px`, `white-space:nowrap` |
| **Discard** | **`Discard`** — secondary, `padding:4px 13px` |

#### 2.2.7 CARTOGRAPHY STYLE (`tbCarto`) — markup 212–220

| Slot | Content |
|---|---|
| Preset chips | `gap:5px`, four chips `min-height:var(--ctl); padding:2px 13px; border-radius:999px; font:var(--m1) mono`. Verbatim: **`Atlas`**, **`Parchment`**, **`Physical`**, **`Ink`**. Lit (`var(--wash2)`/`var(--acc)`) only when it is the current preset **and** `edited === false`; else `var(--ins)`/`var(--sec)`. Toast: `Preset {name} applied — presentation only`. |
| Note | `{{ presetNote }}` — `custom — edited since preset {name}` in `var(--acc)` when edited, else `preset {name}` in `var(--faint)` |
| Spacer | `flex:1` |
| **Reset** | **`Reset`** — `min-height:var(--ctl); padding:2px 12px; border-radius:8px; background:var(--ins); color:var(--sec)`. Drops the whole cartography state; toast `Style reset to preset Atlas`. |
| **Save** | **`Save preset`** — primary, `padding:4px 14px`, `white-space:nowrap`. Toast `Saved as preset — appears in the preset row (mock)`. |

#### 2.2.8 LABEL (`tbLabelRow`) — markup 221–230

| Slot | Content |
|---|---|
| Text field | `<input placeholder="label text">` — `width:150px; background:var(--ins); border:none; border-radius:8px; min-height:var(--ctl); padding:2px 11px; color:var(--ink); font:var(--fs) 'Helvetica Neue'; outline:none`. Default value **`New label`**. |
| Size mode | Segmented pill, two options `padding:3px 10px`: **`fixed`**, **`scale`**. Default `scale`. |
| Anchor | Segmented pill, three options: **`start`**, **`centre`**, **`end`**. Default `centre`. |
| Hint | **`click empty spot creates · click a label selects`** — `color:var(--faint)` |

No commit/discard — placement is immediate and pushes an undo entry `place label`.

#### 2.2.9 ICON (`tbIconRow`) — markup 231–240

| Slot | Content |
|---|---|
| Family | Segmented pill, three options `padding:3px 10px`: **`Places`**, **`Trees`**, **`Sea marks`**. Switching family resets the variant to that family's first. |
| Variant | Pill, `background:var(--wash2); color:var(--acc)`, text **`◆ {variant} · armed`**. **Clicking cycles the variant.** Variants per family: `Places` → `capital-star`, `city-ring`, `town-dot`, `keep`; `Trees` → `pine-3`, `oak`, `palm`, `scrub`; `Sea marks` → `serpent`, `waves`, `anchor`, `compass`. |
| `scale` | Label; slider **90px**; fill `= (scale − 0.5)/1.5`; readout `{round(scale × 100)}%`. Range **0.5 … 2.0**. |
| Hint | **`click stamps the armed icon`** |

Placement pushes undo `stamp icon`. No commit/discard.

#### 2.2.10 SETTLEMENT (`tbSettle`) — markup 241–248

| Slot | Content |
|---|---|
| Class | Segmented pill (`flex-wrap:wrap`), five options `padding:3px 10px`: **`metropolis`**, **`city`**, **`town`**, **`village`**, **`hamlet`**. Default `town`. |
| Faction | Pill `background:var(--ins)`, `9×9px` swatch + faction name. **Click cycles** through the three factions. |
| Hint | **`click drops a place · blank name = generated`** |

Drop pushes undo `drop settlement`, selects the new place, opens the right dock, and toasts `Settlement dropped — staleness: none (civil data, not generation)`. Names cycle through `Ostvel, Carn Rua, Miradel, Thess, Vael Hold, Bruma, Kal Serat, Ond`.

#### 2.2.11 POI (`tbPoi`) — markup 249–254

| Slot | Content |
|---|---|
| Kind | Segmented pill (`flex-wrap:wrap`), five options: **`shrine`**, **`ruin`**, **`mine`**, **`ford`**, **`beacon`**. Default `shrine`. |
| Hint | **`click drops a point of interest · snap to way on`** |

Drop pushes undo `drop POI`, toasts `POI dropped · {kind}`.

#### 2.2.12 TERRITORY (`tbTerr`) — markup 255–263

| Slot | Content |
|---|---|
| Faction | Same cycling swatch pill as Settlement |
| Mode | Segmented pill, two options `padding:3px 11px`: **`add`** and **`subtract ⇧`**. `data-v` values are `add` / `subtract`. On: `var(--wash2)`/`var(--acc)`; off: `transparent`/`var(--dim)`. |
| Hint | **`drag paints the armed faction's claim · respects coastlines`** |

Each paint dab adds/subtracts `round(π · terrRadius²)` cells (`terrRadius` default 10). Drag re-samples once the cursor moves more than `terrRadius × 6` world units.

#### 2.2.13 WAY / ROUTE (`tbWay`) — markup 264–274

| Slot | Content |
|---|---|
| Type | Segmented pill, four options `padding:3px 10px`: **`road`**, **`track`**, **`trail`**, **`bridge`**. Default `road`. |
| Routing | Segmented pill, three options: **`freehand`**, **`snap`**, **`least-cost`**. Default `snap`. |
| Snap toggle | Pill **`snap to places`** — on `var(--wash2)`/`var(--acc)`, off `var(--ins)`/`var(--dim)`. Default on. |
| Spacer | `flex:1` |
| **Commit** | **`✓ Commit · Esc`** — primary, `padding:4px 15px`, `white-space:nowrap`. Refuses with toast `A way needs at least two waypoints` and clears the draft. On success pushes undo `commit way` and toasts `Way committed — staleness: routing graph rebuilt`. |
| **Discard** | **`Discard`** — secondary, `padding:4px 13px`. Clears the draft. |

#### 2.2.14 JOURNEY PLANNER (`tbPlan`) — markup 275–280

| Slot | Content |
|---|---|
| Party line | **`party of {n} · winter · respect closures`** — `font:var(--m1) mono; color:var(--sec)`. *(The season and closure words are literal markup, not bound — they do not follow the party form.)* |
| Button | **`Show route on map`** — `min-height:var(--ctl); padding:2px 12px; border-radius:8px; background:var(--ins); color:var(--sec); white-space:nowrap`. Sets `view = {cx:2048, cy:2048, s:0.5}` and toasts `Route framed — demo route geometry is schematic`. |
| Spacer | `flex:1` |
| Verdict | `{{ planVerdict }} · {{ planDays }}` — `font:var(--m1) mono`, colour `planVerdictCol` (see §1.13) |

No commit/discard.

---

## 3. STATUS BAR

Markup 1139–1148. Rendered only when `showSB` (default `true`; toggled from Window ▸ *Status bar*).

| Property | Value |
|---|---|
| Height | `var(--sbH)` — **26px** desktop, 36px touch |
| Layout | `flex:none; display:flex; align-items:center; gap:18px; padding:0 var(--pad)` |
| Border | `border-top: 1px solid var(--hair)` |
| Font | `var(--m2) 'IBM Plex Mono', monospace` — 9px desktop, 11px touch |

Three regions, left → right:

| Region | Binding | Colour | Extra |
|---|---|---|---|
| **Left — status message** | `{{ statusMsg }}` | **`var(--acc)`** | `letter-spacing:.06em` |
| *(spacer)* | — | — | `flex:1` |
| **Middle-right — secondary** | `{{ statusMid }}` | **`var(--dis)`** | — |
| **Right — key hints** | `{{ statusKeys }}` | **`var(--faint)`** | — |

### 3.1 `statusMsg` — fully specified (file lines 2094–2100)

Strict first-match precedence:

| # | Condition | String |
|---|---|---|
| 1 | a generation run is active | `running {NN} {StageName} — {pct}%` |
| 2 | `statusExtra()` returns non-null | see 2a–2d |
| 2a | sculpt/freehand with ≥ 1 draft stamp | `draft — {n} stamp{s} uncommitted` |
| 2b | biome tool with a non-zero painted count | `painting {target} — {n} cells uncommitted` (`en-US` grouped) |
| 2c | `domain === 'CARTO'` and the style is edited | `style edited — layers differ from preset {name}` |
| 2d | way/route tool with a non-empty draft | `drawing {tool} — {n} points · Esc commits` |
| 3 | `staleFrom != null` | `stage {NN} edited — {10 − staleFrom} downstream stages stale` |
| 4 | measure tool with ≥ 1 point | `measuring — {n} points` |
| 5 | `finalized` | `finalized — stages locked · atlas L0–L3 baked` |
| 6 | otherwise | `stages 01–10 resolved · {world.name lower-cased} · seed {world.seed}` |

Stage names for #1 (`STG[].name`, file lines 1173–1191): `Planet`, `Extent & scale`, `World structure`, `Tectonics`, `Volcanism & impacts`, `Erosion`, `Hydrology`, `Climate`, `Ecology & biomes`, `Resources & soils`.

### 3.2 `statusKeys`

`UNSPECIFIED:` the binding. `valsCore` computes `keyHints` immediately after `statusMsg` (file line 2101) with exactly the right shape, and the mapping `keyHints → statusKeys` is one of the truncated lines. Its value:

- touch frames: **`long-press = sample · pinch zooms`**
- desktop: `V M R ` + (`WORLD` → `B F`; `CARTO` → `L I`; otherwise → `S P T W`) + ` · ⌘Z · Esc`
  → e.g. `V M R B F · ⌘Z · Esc`

### 3.3 `statusMid`

`UNSPECIFIED:` **no candidate exists anywhere in the delivered file.** Nothing in the surviving code computes a middle status string. Candidates present in state that a builder would plausibly need: `savedAt` (default `'14:02'`), `lastRun` (default `'—'`), `prefs.autosave`, `prefs.units`. **Do not guess — re-export or ask.**

---

## 4. TIMELINE BAR

Markup 1108–1138. Sits **between** the main content row and the status bar. Gated by `tlShow`.

`UNSPECIFIED:` `tlShow`. The Window menu names the item **`Timeline (CIVIL · INFRA)`** (file line 2015) and toggles `state.tlOpen`, which defaults to `false` — so the intended gate is *the Timeline is off by default and is meaningful in the CIVIL / infrastructure context*, but the exact predicate is truncated.

### 4.1 Collapsed (`tlCollapsed`)

Whole strip is clickable → `hTlExpand`.

| Property | Value |
|---|---|
| Height | `calc(var(--sbH) - 2px)` — **24px** desktop, 34px touch |
| Layout | `flex:none; display:flex; align-items:center; gap:12px; padding:0 var(--pad)` |
| Border | `border-top: 1px solid var(--hair)` |

| Slot | Content | Style |
|---|---|---|
| 1 | literal **`TIMELINE`** | `font:var(--m2) mono; letter-spacing:.16em; color:var(--faint)` |
| 2 | `{{ tlYearLabel }}` | `font:var(--m1) mono; color:var(--sec)` |
| 3 | spacer | `flex:1` |
| 4 | literal **`▴ expand`** | `font:var(--m2) mono; color:var(--faint)` |

### 4.2 Expanded (`tlExpanded`)

Container: `flex:none; display:flex; flex-direction:column; gap:6px; padding:8px var(--pad); border-top:1px solid var(--hair)`.

**Row 1 — transport** (`display:flex; align-items:center; gap:10px`)

| Order | Control | Geometry | Colour | Handler |
|---|---|---|---|---|
| 1 | Play/pause, glyph `{{ tlPlayGlyph }}` | `var(--ctl)` square, `border-radius:8px`, `background:var(--ins)` | `var(--acc)` | `hTlPlay` |
| 2 | Step back, glyph **`◀`** | same | `var(--sec)` | `hTlStep` `data-d="-1"` |
| 3 | Step forward, glyph **`▶`** | same | `var(--sec)` | `hTlStep` `data-d="1"` |
| 4 | Speed segmented pill | outer `background:var(--ins); border-radius:999px; padding:2px`; items `padding:2px 10px; border-radius:999px; font:var(--m1) mono` | per-item `col`/`bg` | `hTlSpeed` `data-v` |
| 5 | `{{ tlState }}` | — | `var(--acc)`, `font:var(--m1) mono` | — |
| 6 | spacer | `flex:1` | — | — |
| 7 | Layer toggles | each `padding:3px 10px; border-radius:999px; font:var(--m2) mono` | per-item `bg`/`col` | `hTlTog` `data-id` |
| 8 | Collapse, glyph **`⌄`** | `padding:0 4px` | `var(--faint)` | `hTlCollapse` |

**Row 2 — scrub track** (`height:16px; display:flex; align-items:center; cursor:pointer; touch-action:none`), handler `hTlScrub` (`onPointerDown`):
- Rail: `flex:1; height:3px; background:var(--ins); border-radius:2px`
- Playhead: `position:absolute; left:{{ tlPct }}; top:-5px; width:2px; height:13px; background:var(--acc)`

**Row 3 — track scale** (`display:flex; justify-content:space-between; font:var(--m2) mono; color:var(--dis)`), three items:
- left: literal **`YEAR −400`**
- centre: `{{ tlYearLabel }}` in `var(--sec)`
- right: literal **`YEAR 1200`**

**The scrub range is therefore fixed at year −400 … year 1200 (1600 years).**

### 4.3 Timeline state (file line 1204 — complete)

| Key | Default | Meaning |
|---|---|---|
| `tlOpen` | `false` | expanded / collapsed |
| `tlYear` | `412` | year cursor — 50.75 % of the −400…1200 track |
| `tlRun` | `false` | playing |
| `tlSpeed` | `'×10'` | current speed |
| `tlTog` | see below | six layer toggles |

**Layer toggles — exactly six, in this order, with these defaults.** The markup renders `{{ t.id }}` as the pill's text, so these strings are the visible labels verbatim:

| Label | Default |
|---|---|
| `Climate` | **on** |
| `Population` | **on** |
| `Economy` | off |
| `Politics` | **on** |
| `Infrastructure` | off |
| `Warfare` | off |

`hint-placeholder-count="6"` in the markup confirms six.

`UNSPECIFIED:`
- **The speed ladder.** `tlSpeeds` is truncated; only the default member `×10` survives. The markup's `hint-placeholder-count="3"` implies **three** options, one of which is `×10`. The other two are unknown.
- `tlPlayGlyph` (both play and pause glyphs), `tlState` (the running/paused string), `tlYearLabel` format (`412` alone? `YEAR 412`? `412 AR`?), `tlPct` formula (very likely `(tlYear + 400) / 1600`, but not stated), the step size for `hTlStep`, the tick interval for playback (`clearInterval(this._tlInt)` is called in `componentWillUnmount` — an interval exists but is never created in the delivered file), and the on/off colour pair for the six toggles.

---

## 5. VIEWPORT FURNITURE

The viewport is the flex-`1` cell between the left and right docks: `flex:1; min-width:0; position:relative; overflow:hidden`. Everything below is absolutely positioned inside it, so the furniture never overlaps the docks.

### 5.1 The canvas

`position:absolute; inset:0; width:100%; height:100%; touch-action:none; cursor:{{ mapCursor }}`.
Events: `onPointerDown/Move/Up/Cancel/Leave`, `onWheel`, `onDoubleClick`.
Device-pixel ratio is clamped: `dpr = min(window.devicePixelRatio || 1, 1.5)`.

`UNSPECIFIED:` `mapCursor` — no cursor string per tool is given anywhere.

**What the canvas draws** (`draw()`, file lines 1845–1897 — complete):

| Element | Spec |
|---|---|
| Background | radial gradient centred `(W/2, H·0.4)`, radius `max(W,H)·0.7`. Dark: `#17191a` → `#101112` @0.6 → `#0d0e0f` @1. Light: `#faf9f5` → `#f1efe9` → `#eae7e0`. |
| Layer tint | full-canvas fill, per active layer: `biome` `rgba(110,165,95,.05)`, `political` `rgba(150,115,205,.055)`, `elevation` `rgba(224,163,74,.045)`, `slope` `rgba(120,140,170,.05)`, `flow` `rgba(90,140,190,.05)`, `temp` `rgba(210,110,80,.05)`, `rain` `rgba(80,130,200,.05)`. **`relief` has no tint.** |
| Minor grid | step **64** world units when `s > 0.8`, **128** when `s > 0.3`, else **256**; multiples of 512 skipped; `rgba(ink,.05)`, `lineWidth 1` |
| Major grid | every **512** units; `rgba(ink,.13)` |
| World bounds | `strokeRect(0,0,4096,4096)` at `rgba(ink,.22)` |
| Grid labels | every 512, excluding 0 and 4096; `10px 'IBM Plex Mono'` at `rgba(ink,.3)`. X labels at `(wx(x)+4, 14)`; Y labels at `(4, wy(y)−4)`. |
| Accent | `#a4650f` in light, `#e0a34a` in dark |
| Region marquee | dashed `[5,4]` stroke in accent + four `6×6px` filled accent corner handles |
| Measure geometry | polyline / closed filled ring (accent at `globalAlpha 0.1`) / circle+radius, `lineWidth 1.4`; vertices are `r=3` filled dots in `#e8ebec` (dark) or `#111210` (light); per-segment `fmtKm` labels in accent `10px` mono offset `(+6, −6)` from the midpoint; section mode draws literal `A` and `B` at `(+7, −7)` plus four perpendicular 12px tick marks at t = 0.25/0.5/0.75, `globalAlpha 0.55` |
| Sample marker | filled `r=4` disc + `r=9` ring, both accent |
| Brush cursor ring | for `biome`/`sculpt`/`freehand`: circle of radius `brushSize() × view.s`, accent, `globalAlpha 0.8`. `brushSize()` = `bp.radius × 8` for biome, else `sc.brush.size`. |
| Stamps, ways, landmarks, places, POIs, labels, icons, territory | drawn by `drawExtra`, `drawExtra4`, `drawExtra6` — outside my region |

**Navigation** (complete):
- Drag with 1 pointer pans (`pan` mode) unless the armed tool acts on drag (`region`, `biome`, `sculpt`, `freehand`, `territory`), in which case it is `act` mode. A drag is only "moved" past **4 px**.
- 2 pointers → pinch; scale = `s0 × d/d0`, clamped `0.12 … 4`.
- Wheel: `s ×= exp(−deltaY × 0.0012)`, clamped; zooms about the cursor.
- Double-click: `s ×= 1.6` about the cursor, clamped to 4. **Exception:** with Measure armed and ≥ 1 point, double-click instead ends the measurement (`done = true`) and does not zoom.
- Taps outside `0 … 4096` in either axis are ignored entirely.

### 5.2 Top-left cluster (`position:absolute; top:10px; left:10px; display:flex; align-items:center; gap:8px`)

#### Layers button

| Property | Value |
|---|---|
| Size | `var(--tool)` square — **30 × 30px** desktop, 44 × 44px touch |
| Radius | `8px` |
| Background | `{{ layersBtnBg }}` |
| Border | `1px solid var(--hair)` |
| Colour | `{{ layersBtnCol }}` |
| Glyph | inline SVG `15 × 15`, `viewBox "0 0 16 16"`, `fill:none; stroke:currentColor; stroke-width:1.2; stroke-linecap:round; stroke-linejoin:round`, three paths — a stacked-sheets icon: `M8 2.5 14 5.5 8 8.5 2 5.5Z` · `M2 8.5 8 11.5 14 8.5` · `M2 11 8 14 14 11` |
| Handler | `hLayersBtn` |

The button's wrapper carries `data-menupop="1"`, so a pointer-down anywhere outside it closes the popover (shared with the menu-bar dismissal logic). `Escape` closes it too, at priority 2 (after the menu bar, before every tool action).

`UNSPECIFIED:` `layersBtnBg` and `layersBtnCol` in both open and closed states.

#### Layers popover (`layersOpen`)

| Property | Value |
|---|---|
| Position | `absolute; top: calc(100% + 6px); left: 0` |
| Width | **238px** (a fixed literal — *not* `var(--pop)`) |
| Background | `var(--pan)` |
| Border | `1px solid var(--bor)` |
| Shadow | `var(--shadow)` |
| Padding | `6px 0` |
| z-index | `60` |

**Opacity row** — `display:flex; align-items:center; gap:8px; padding:6px 12px`:
- literal **`OPACITY`** — `font:var(--m2) mono; color:var(--faint)`
- slider `flex:1`, standard geometry, handler `hMasterOp`
- readout `{{ masterOpDisp }}` — `font:var(--m1) mono; color:var(--sec); width:32px; text-align:right`

`state.masterOp` defaults to `1`. `UNSPECIFIED:` `masterOpPct`/`masterOpDisp` formatting (almost certainly `%` and `0–100`, but not stated) and the `hMasterOp` write target.

**Layer rows** — driven by `this.LAYERS` (file line 1191, complete). Headers and rows interleave in exactly this order:

| Kind | Label | `id` | Shortcut key |
|---|---|---|---|
| header | `SURFACE` | — | — |
| row | `Relief` | `relief` | **`1`** |
| row | `Biome` | `biome` | **`2`** |
| row | `Political` | `political` | **`3`** |
| header | `TERRAIN FIELDS` | — | — |
| row | `Elevation` | `elevation` | **`4`** |
| row | `Slope` | `slope` | **`5`** |
| row | `Flow accumulation` | `flow` | **`6`** |
| header | `CLIMATE` | — | — |
| row | `Temperature` | `temp` | **`7`** |
| row | `Rainfall` | `rain` | **`8`** |

Header style: `padding:8px 12px 2px; font:var(--m2) mono; letter-spacing:.2em; color:var(--faint)`
Row style: `min-height:var(--row); display:flex; align-items:center; gap:9px; padding:2px 12px; cursor:pointer; background:{{ l.bg }}; color:{{ l.col }}`
Key badge: `width:14px; font:var(--m2) mono; text-align:center; border:1px solid var(--div); border-radius:4px; color:{{ l.keyCol }}`

Default active layer: **`relief`**. Pressing `1`–`8` anywhere (outside a text field, on the app screen) sets the layer directly — the popover need not be open.

`UNSPECIFIED:` `l.bg`, `l.col`, `l.keyCol` for the selected vs. unselected states, and whether `hLayerPick` closes the popover.

#### Viewport context chip

`{{ vpContext }}` — `font:var(--m2) mono; letter-spacing:.14em; color:var(--dim); background:{{ scrimBg }}; padding:4px 9px; border-radius:6px`.

`UNSPECIFIED:` `vpContext` (its string in every context) and `scrimBg`. The one surviving hook, `vpCtxExtra()` (file line 1697), **returns the empty string** — so no subsystem contributes to it in the delivered code.

### 5.3 Top-right readout

`position:absolute; top:12px; right:12px; font:var(--m2) mono; color:var(--dim); background:{{ scrimBg }}; padding:4px 9px; border-radius:6px`.

Composed of three parts, two literal and one live:

> `equirect · ` + `<span ref=zoomRef>zoom 100%</span>` + ` · ` + `{{ vpField }}`

- **`equirect`** is a hard-coded literal in the markup.
- The zoom span is written imperatively by `_updateHud()` (file line 1898), **not** by a binding: `'zoom ' + Math.round(view.s * 100) + '%'`. Initial DOM text is `zoom 100%`; the real initial value is `zoom 34%` (`s = 0.34`).
- `UNSPECIFIED:` `vpField`. Presumably the active layer's display name (the `LAYERS` labels above), but nothing in the delivered file says so.

### 5.4 Bottom-left scale bar

`position:absolute; bottom:{{ hudBottom }}; left:12px; display:flex; flex-direction:column; gap:3px`.

| Part | Spec |
|---|---|
| Label | `font:var(--m2) mono; color:var(--dim)`. Initial DOM text `250 km`. Written imperatively: **`fmtKm(120 / view.s * 2.5)`** — i.e. the bar is exactly 120 screen px wide and the label is what 120 px measures. |
| Rule | `width:120px; height:1px; background:var(--sec); display:block; position:relative` |
| End ticks | two `1 × 5px` bars in `var(--sec)`, at `left:0; bottom:0` and `right:0; bottom:0` |

`fmtKm(km)` (file line 1885, complete): converts to miles (`× 0.621371`) when `prefs.units === 'mi'`; formats `>= 100` as `Math.round(v).toLocaleString('en-US')`, otherwise `v.toFixed(1)`; always suffixed with ` ` + the unit string (`km` or `mi`).

### 5.5 Bottom-right coordinate readout

`position:absolute; bottom:{{ hudBottom }}; right:12px; font:var(--m2) mono; color:var(--dim); background:{{ scrimBg }}; padding:4px 9px; border-radius:6px`.

Written imperatively (file line 1900):
- cursor over the map: **`{fmtKm(x × 2.5)} E · {fmtKm(y × 2.5)} N · {elev} m`** (elevation `en-US` grouped)
- cursor off the map (pointer leave): **`— · —`**

### 5.6 `hudBottom` — the only fully specified HUD binding

From `vals5()` (file line 1425): `'180px'` when the cross-section strip is showing (measure tool, section sub-mode, valid profile), otherwise `'12px'`. Both the scale bar and the coordinate readout use it, so both lift together above the strip.

### 5.7 Cross-section strip (`secStrip`) — the one overlay the prototype draws

Shown when `tool === 'measure'`, `measure.sub === 'section'`, and a profile exists.

| Property | Value |
|---|---|
| Position | `absolute; left:0; right:0; bottom:0` — spans the viewport only, not the docks |
| Height | **168px** |
| Background | `var(--pan)` |
| Border | `border-top: 1px solid var(--hair)` |
| z-index | `15` |

**Header row** — `flex:none; display:flex; align-items:center; gap:14px; padding:6px var(--pad) 2px; font:var(--m2) mono`:

| Slot | Content | Colour |
|---|---|---|
| 1 | literal **`SECTION A → B`**, `letter-spacing:.16em` | `var(--acc)` |
| 2 | `{{ secLen }}` = `fmtKm(|AB| × 2.5)` | `var(--dim)` |
| 3 | `{{ secField }}` = the field name, lower-cased | `var(--sec)` |
| 4 | literal **`120 samples · ×4 exaggeration`** | `var(--faint)` |
| 5 | spacer `flex:1` | — |
| 6 | `{{ secMinMax }}` = `min {n}{unit} · max {n}{unit}` | `var(--faint)` |
| 7 | **`✕`**, clickable, `padding:0 4px`, handler `hMeasClear` | `var(--faint)` |

**Body** — `flex:1; display:flex; min-height:0; padding:2px var(--pad) 4px; gap:10px`:

- **Y-axis column**: `width:52px; flex:none; display:flex; flex-direction:column; justify-content:space-between; text-align:right; font:var(--m2) mono; color:var(--dis); padding:2px 0 14px`. Three values top→bottom: `{{ secTop }}` = max, `{{ secMid }}` = `round((min+max)/2)`, `{{ secBot }}` = min.
- **Chart**: `flex:1; position:relative; min-width:0; border-left:1px solid var(--div)`
  - SVG `viewBox="0 0 1000 130" preserveAspectRatio="none"`, `position:absolute; inset:0; width:100%; height:calc(100% - 14px)`
  - Two gridlines, `x1=0 x2=1000`, at **`y=43`** and **`y=86`**, `stroke:{{ secGridCol }}`, `stroke-width:1`
    - `secGridCol` = `rgba(0,0,0,.07)` light / `rgba(255,255,255,.06)` dark
  - Profile path `d="{{ secArea }}"`, `fill:{{ secFillCol }}`, `stroke:{{ secLineCol }}`, `stroke-width:1.6`
    - `secFillCol` = `rgba(164,101,15,.12)` light / `rgba(224,163,74,.13)` dark
    - `secLineCol` = `#a4650f` light / `#e0a34a` dark
  - Path construction (`secSample()`, file line 1354): starts `M0,130`, then one `L` per sample at `x = i/n × 1000` and `y = 126 − ((v − min)/range × 118)`, closing `L1000,130 Z`. Range floor is `max(max − min, 1)`.
  - **X-axis strip**: `position:absolute; left:0; right:0; bottom:0; height:14px; display:flex; justify-content:space-between; font:var(--m2) mono; color:var(--dis)`, three labels: **`A · 0`**, `{{ secHalf }}` = `fmtKm(len/2)`, **`B · {{ secLen }}`**

**Two discrepancies in this block, both in the delivered code:**
1. The header claims **`×4 exaggeration`**, but `secSample()` normalises the profile to `min…max` across 118 units of a 130-unit box. There is no ×4 factor anywhere. Decide which is authoritative before building.
2. The header and the `SAMPLES` stat row both say **120 samples**, but the loop is `for (let i = 0; i <= n; i++)` with `n = 120` — **121 samples**. The `1 per {fmtKm(len/120)}` spacing figure is consistent with 120 *intervals*.

### 5.8 Toast stack

Not strictly viewport-local — it is positioned against the shell, but it overlays the viewport.

| Property | Value |
|---|---|
| Position | `absolute; left:0; right:0; bottom: calc(var(--sbH) + 14px)` |
| Layout | `display:flex; flex-direction:column; align-items:center; gap:6px` |
| Pointer events | `none` |
| z-index | **90** |
| Pill | `background:var(--ins); border:1px solid var(--bor); color:var(--body); padding:7px 16px; border-radius:999px; font:var(--m1) mono; box-shadow:var(--shadow)` |
| Entry animation | `tIn .18s ease` — `@keyframes tIn { from { opacity:0; transform:translateY(6px) } to { opacity:1; transform:none } }` |
| Lifetime | **2600 ms**, then removed |
| Stack cap | `toasts.slice(-2)` plus the new one → **at most 3 visible** |

---

## 6. TOOL PALETTE

The palette lives at the **top of the left dock**, in its own block between the dock's mode switcher and the scrollable body (markup 317–328). It is not a floating viewport palette.

| Property | Value |
|---|---|
| Block | `flex:none; padding:10px var(--pad); border-bottom:1px solid var(--div)` |
| Header | literal **`TOOLS`** — `font:var(--m2) 'IBM Plex Mono'; letter-spacing:.2em; color:var(--faint); margin-bottom:7px` |
| Row | `display:flex; gap:5px; flex-wrap:wrap` |
| Global tool button | `width: height: var(--tool)` (**30 × 30px** desktop, 44 × 44px touch), `border-radius:8px; display:grid; place-items:center`, `title="{tip}"` |
| Divider | `width:1px; height:var(--tool); background:var(--div); margin:0 3px` — sits between the global and domain groups |
| Domain tool button | `min-height:var(--tool); padding:2px 12px; border-radius:8px; display:flex; align-items:center; gap:6px; font:var(--m1) 'IBM Plex Mono'` — label, then the shortcut key in `var(--faint)` |
| Armed state (both kinds) | `background:var(--wash2); color:var(--acc)` |
| Idle state | `background:var(--ins); color:var(--sec)` |

### 6.1 Global tools — always present, in this order (file line 2072)

| Id | Glyph (icon-only) | `title` tooltip, verbatim | Key | Notes |
|---|---|---|---|---|
| `inspect` | arrow-cursor | `Inspect · V` | **`V`** | default armed tool |
| `measure` | ruler | `Measure · M` | **`M`** | |
| `region` | dashed crop marks | `Region select · R` | **`R`** | |
| `pan` | hand | `Pan / zoom — always available` | *(none)* | **hard-coded to `background:var(--ins); color:var(--dis)` — it never lights up, even when armed** |

All four SVGs are `14 × 14`, `viewBox "0 0 16 16"`, `fill:none; stroke:currentColor; stroke-width:1.2; stroke-linecap:round`; the first, second and fourth add `stroke-linejoin:round`. Exact path data (file lines 1168–1171):

| Id | Path data |
|---|---|
| `inspect` | `M4 2.2 12.2 9.4 8.6 9.8 10.6 13.8 8.8 14.6 6.9 10.6 4 13Z` |
| `measure` | `<rect x="1.8" y="9.2" width="12.4" height="4.4" rx="0.8" transform="rotate(-30 8 11.4)">` + `M6.2 8.4 7 9.8M8.8 6.9 9.6 8.3M11.4 5.4 12.2 6.8` |
| `region` | `M5 2.5H2.5V5M11 2.5H13.5V5M5 13.5H2.5V11M11 13.5H13.5V11` + `M6.8 2.5H9.2M6.8 13.5H9.2M2.5 6.8V9.2M13.5 6.8V9.2` with `stroke-dasharray="1.6 1.8"` |
| `pan` | `M5.4 7.4V4.2a1 1 0 0 1 2 0V7M7.4 6.8V3.4a1 1 0 0 1 2 0V7M9.4 7V4.4a1 1 0 0 1 2 0V8.6c0 2.6-1.2 4.6-3.6 4.6-1.9 0-2.8-.9-3.6-2.5L3 8.3a.94.94 0 0 1 1.6-1l.8 1.3` |

### 6.2 Domain tools — swap with the active domain (file line 2073)

Rendered as **text pills with a trailing key badge**, not icons. The label and key strings below are the verbatim rendered text.

**WORLD**

| Id | Label | Key badge |
|---|---|---|
| `sculpt` | `Sculpt` | *(empty string — renders no key)* |
| `freehand` | `Freehand` | `F` |
| `biome` | `Biome paint` | `B` |

**CIVIL**

| Id | Label | Key badge |
|---|---|---|
| `settlement` | `Settlement` | `S` |
| `poi` | `POI` | `P` |
| `territory` | `Territory` | `T` |
| `way` | `Way` | `W` |
| `route` | `Route` | `⇧R` |

**CARTO**

| Id | Label | Key badge |
|---|---|---|
| `label` | `Label` | `L` |
| `icon` | `Icon` | `I` |

### 6.3 Keyboard shortcuts — what is actually bound

The complete key map (file line 1832): `{ v: 'inspect', m: 'measure', r: 'region', b: 'biome', l: 'label', i: 'icon', f: 'freehand' }`, matched case-insensitively and skipped when `metaKey`/`ctrlKey` is held or focus is in an `input`/`textarea`.

**Discrepancy to resolve:** the palette *advertises* `S`, `P`, `T`, `W` and `⇧R` for the five CIVIL tools, but **none of them is in the key map** — pressing them does nothing. `Sculpt` advertises no key and has none. The Help menu's own summary agrees with the map, not the palette: `V M R B L I arm tools · ⌘Z undo · Esc commits or disarms` (file line 2020). The status bar's key hints, however, print `S P T W` in CIVIL (§3.2) — so **two of the three places that name the shortcuts are wrong.**

Also note `r` maps to `region`, and layer key `1`–`8` handling runs in the *same* handler after the tool map, so both can fire from one keypress if a key ever appeared in both tables (none currently does).

### 6.4 Arming behaviour (`armTool`, file line 1939 — complete)

1. If `state.finalized` and the id is one of `biome, sculpt, freehand, settlement, poi, territory, way, route` → refuse with toast **`World is finalized — {id} is locked`**.
2. `sculpt` or `freehand` additionally force `domain = 'WORLD'` and `worldMode = 'b'` (the Sculpt panel).
3. `measure` additionally clears `measure.done`.
4. Otherwise just sets `tool`.

Choosing a sculpt *feature* chip in the left dock also arms the tool: it arms `freehand` for the `freehand` feature and `sculpt` for every other, but only when the current tool is neither already.

### 6.5 Escape — the disarm ladder (file line 1831, complete)

Strict order; the first applicable branch runs and stops:

1. A menu is open → close menu and submenu
2. The layers popover is open → close it
3. `escExtra()`: way/route tool with a non-empty draft → **commit the way**; sculpt/freehand with a selected stamp → deselect it
4. Measure tool with ≥ 1 point → mark the measurement `done`
5. Region tool with a marquee → clear it
6. Tool is not `inspect` → **arm `inspect`**

### 6.6 Undo / redo

`⌘Z` / `Ctrl+Z` undoes; `⇧⌘Z` / `Ctrl+Shift+Z` redoes. Depth is `prefs.undoDepth`, default **5** (Preferences offers `5` / `15` / `50`). A new push clears the redo stack. Undo toasts `Undo — {label}`, redo toasts `Redo — {label}`; empty stacks toast `Nothing to undo` / `Nothing to redo`.

Undo labels recorded in my region and its neighbours, verbatim: `commit way`, `place label`, `delete label`, `clear annotation`, `stamp icon`, `drop settlement`, `delete settlement`, `drop POI`, `add {type} stamp`, `delete stamp`, `discard draft`, `edit stage {NN}`.

Parameter edits coalesce: `setField` only pushes an undo entry if no edit has happened in the last **900 ms**.

---

## 7. Complete `UNSPECIFIED` register

Consolidated, in build order. Every item is a consequence of the 256 KiB truncation described in §0 unless noted.

**Blocking — no value or plausible reconstruction exists anywhere in the file**

1. `statusMid` — the status bar's middle region. Nothing in the surviving code computes any candidate string.
2. `tlSpeeds` — the timeline speed ladder. Only `×10` is known (the state default); the markup implies three options.
3. `tlPlayGlyph`, `tlState`, `tlYearLabel` format, `tlPct` formula, `hTlStep` step size, playback tick interval.
4. `tlShow` — the predicate that shows the timeline at all.
5. `sampleRows` — the Inspect readout's row labels, order, formatting and per-row colour. The 14 available `sampleData` fields are listed in §1.4; the presentation is not.
6. `regionRows` — the five Region readout rows, and `regionReadout` in the tool options bar.
7. `vpContext` — the viewport context chip's string in every context. Its one extension hook returns `''`.
8. `vpField` — the top-right field name.
9. `mapCursor` — no per-tool cursor is specified.
10. `scrimBg` — the shared chip/readout scrim behind three separate HUD elements.
11. The entire GENERATION PIPELINE tool-options row's strings and colours: `runStageLabel`, `runChainLabel`, `pipeNote`, `pipeNoteCol`, `progTitle`, `progW`, `progPct`, `finLabel`, `finBord`, `finCol`, `finBg`, and what `hFinalize` does.
12. `rdTitle` for the Sample / Measure / Region contexts, and `rdCollapsedLabel` in every context. *(The nine extra-mode titles are specified — §1.3.)*
13. `tbLabel` for every context except sculpt and biome.
14. Whether `tbInspect` excludes CARTO and the CIVIL planner. As written, two tool-options rows can render into one 40px bar. **Do not guess.**

**Recoverable-with-a-decision — the shape is pinned, only the styling choice is missing**

15. `inspChips` on/off `bg`/`col`. The four chip texts (`places`, `labels`, `icons`, `ways`) and their all-true defaults are known.
16. `measSegCol` / `measSegBg` / `measPathCol` / `measPathBg`, plus the bodies of `hMeasMode` and `hMeasClear`. Also: `measure.mode` is set but read nowhere — the multi-segment / point-to-point switch is inert in the delivered code.
17. `layerRows`' `bg` / `col` / `keyCol` for selected vs. unselected, and whether `hLayerPick` closes the popover. The eleven rows, their labels and their `1`–`8` keys are exact.
18. `masterOpPct` / `masterOpDisp` formatting; the write target of `hMasterOp`.
19. `layersBtnBg` / `layersBtnCol` in open and closed states.
20. The six timeline toggles' on/off colour pair. The six labels and their defaults are exact.
21. `measRows`' third field: `bear` is computed (formula in §1.5) but its formatting is cut mid-token.

**Contradictions in the delivered design, to be adjudicated rather than guessed**

22. §5.7 — the section strip header says `×4 exaggeration`; the profile maths applies no exaggeration.
23. §5.7 — `120 samples` is stated twice; the loop produces 121.
24. §6.3 — the palette advertises `S P T W ⇧R` and the status bar prints them, but none is bound; the Help menu's list is the only accurate one.
25. §1.12 — the Territory `AREA` readout labels its output `km²` while dividing by 1000.
26. §1.9 — the ramp hue slider's thumb is pinned at `50%` and never reflects the selected stop.
27. §1.2 — the Region marquee sets `rdOpenForce`, which nothing reads; the right dock does not open on region completion.
28. §1.3 — the `ROUTE` right-dock title is unreachable.
29. §2.2.14 — the Journey planner's tool-options line hard-codes `winter · respect closures` instead of binding the party form.
30. §1.14 — the Way dock's `GRADE · MAX` is the literal `4.2%` in markup, not a binding.

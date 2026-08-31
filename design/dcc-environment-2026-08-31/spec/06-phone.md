# Cartalith Android — Phone Specification

Source: `C:\Users\Vincent\Cartalith_GDT\design\dcc-environment-2026-08-31\Cartalith Android.dc.html`
Everything below is read out of that file's `<script data-dc-script>` `Component` class and its markup. All lengths are CSS px = Android dp.

---

## 0. Global foundations

### 0.1 Component props (host-configurable)

| Prop | Editor | Options | Default | Effect |
|---|---|---|---|---|
| `deviceStrip` | boolean | — | `true` | Shows/hides the device-picker strip above the frame |
| `startIn` | enum | `picker`, `map` | `picker` | `map` boots straight into the app screen with world `ELDRA` |
| `coachMarks` | boolean | — | `true` | Enables the two coach-mark steps |

### 0.2 Fonts

| Role | Stack |
|---|---|
| Frame body default | `13px/1.45 'Helvetica Neue', Helvetica, Arial, sans-serif`, colour `var(--body)` |
| Everything labelled | `'IBM Plex Mono', monospace` at explicit sizes, weights **400** and **500** only |

Google Fonts import: `family=IBM+Plex+Mono:wght@400;500`.
Only four places use the sans stack: list-row primary labels (`font-size:12px` / `12.5px`), the sheet layer-row label, and the plan stage name.

### 0.3 Colour tokens

Declared on the frame's inner div; the light set is applied as an inline override string (`themeVars`) on a child.

| Token | Dark (default) | Light |
|---|---|---|
| `--sur` | `#0d0e0f` | `#f4f2ee` |
| `--pan` | `#15171a` | `#fbfaf7` |
| `--pan2` | `#121314` | `#eceae4` |
| `--ink` | `#e8ebec` | `#111210` |
| `--body` | `#c8cbcd` | `#23241f` |
| `--sec` | `#8d9296` | `#3d3f39` |
| `--dim` | `#6f7478` | `#6b6f6a` |
| `--faint` | `#5f6468` | `#8d9088` |
| `--acc` | `#e0a34a` | `#a4650f` |
| `--accInk` | `#16130c` | `#f7f4ee` |
| `--hair` | `rgba(255,255,255,.10)` | `rgba(0,0,0,.14)` |
| `--hair2` | `rgba(255,255,255,.07)` | `rgba(0,0,0,.08)` |
| `--bord` | `rgba(255,255,255,.16)` | `rgba(0,0,0,.20)` |
| `--wash` | `rgba(224,163,74,.14)` | `rgba(164,101,15,.10)` |
| `--chip` | `rgba(255,255,255,.05)` | `rgba(0,0,0,.05)` |
| `--chipOn` | `rgba(255,255,255,.10)` | `rgba(0,0,0,.10)` — **declared, never used** |
| `--warn` | `#e0a840` | *not overridden* |
| `--block` | `#c26a60` | *not overridden* |
| `--good` | `#8fae7d` | *not overridden* |
| `--water` | `#7d9dae` | *not overridden* |

Derived, theme-switched inline values:

| Name | Dark | Light |
|---|---|---|
| `pillBg` (all floating chrome) | `rgba(18,20,21,.92)` | `rgba(251,250,247,.92)` |
| `scrimCol` (map top gradient) | `rgba(10,11,12,.72)` | `rgba(244,242,238,.85)` |
| `statusCol` (status text) | `#a9adb0` | `#3d3f39` |

### 0.4 Shared control primitives

| Control | Spec |
|---|---|
| **Slider** (`input[type=range]`) | element height 22; track height 3, radius 2, background `var(--hair)`; thumb 20×20, radius 10, background `var(--acc)`, `margin-top:-8.5px`, no border |
| **Toggle (small)** — GENERATE, PLAN, stage-override | track 40×22 radius 11; knob 18×18 radius 9 at `top:2px`, `left:2px` off / `right:2px` on. On: track `var(--acc)`, knob `var(--accInk)`. Off: track `var(--chip)`, knob `var(--sec)`. GENERATE toggles only: `transition:background .15s` on track and `left .15s, right .15s` on knob |
| **Toggle (large)** — MORE rows | track 44×24 radius 12; knob 20×20 radius 10 at `top:2px`, same colours, **no transition declared** |
| **Selected-chip formula** `chip(on)` | on → `border-color:var(--acc)`, `color:var(--acc)`, `background:var(--wash)`; off → `border-color:var(--hair)`, `color:var(--sec)`, `background:transparent` (MAP tool/icon chips use `var(--chip)` for the off background instead of transparent) |
| **Stepper button** (− / ＋) | 38×38, radius 14, background `var(--chip)`, colour `var(--sec)`, glyphs `−` and `＋` at 14px mono |

### 0.5 Animations

| Name | Definition | Used by |
|---|---|---|
| `tIn` | `opacity 0 → 1`, `translateY(8px) → none` | toasts (`.25s ease`), coach mark (`.3s ease`) |
| `pIn` | `opacity 0 → 1`, `translateX(40px) → none` | inspector panel (`.24s ease`) |
| Sheet | `height .28s cubic-bezier(.3,.9,.3,1)` (portrait) / `width .28s cubic-bezier(.3,.9,.3,1)` (landscape) | bottom sheet / side drawer |
| Double-tap zoom | 240 ms, easing `1-(1-k)³`, factor **×1.7** | map |

---

## 1. Device frames

`devices=[…]` in `renderVals()`:

| Label | w × h (portrait) | Landscape (w↔h) | Sheet: peek / half / full | Landscape drawer width | Inspector width |
|---|---|---|---|---|---|
| `360` | 360 × 800 | 800 × 360 | 66 / 329 / 620 | 368 | 295 (P) / 340 (L) |
| `412` | 412 × 892 **(default)** | 892 × 412 | 66 / 372 / 712 | 410 | 338 (P) / 340 (L) |
| `480` | 480 × 1040 | 1040 × 480 | 66 / 440 / 860 | 440 | 340 |
| `TABLET 800` | 800 × 1280 | 1280 × 800 | 66 / 550 / 1100 | 440 | 340 |

Frame chrome: `border-radius:30px`, `border:1px solid rgba(255,255,255,.16)`, `overflow:hidden`, `background:#101112`, `flex:none`, size overridden inline to `width:{fw}px;height:{fh}px`.

Device strip (host chrome, not part of the app):
- Chip: `padding:8px 14px`, `border-radius:16px`, `font:10px 'IBM Plex Mono'`, `letter-spacing:.12em`. Active `border #e0a34a`, `color #e0a34a`, `background rgba(224,163,74,.12)`; inactive `border rgba(255,255,255,.16)`, `color #8d9296`, `background transparent`.
- Divider `1px × 20px`, `rgba(255,255,255,.14)`.
- Orientation chip: label `PORTRAIT ⤾` / `LANDSCAPE ⤾`, `border 1px rgba(255,255,255,.16)`, `color #c8cbcd`.
- Dimension readout: `{fw} × {fh} dp`, `font:10px 'IBM Plex Mono'`, `#5f6468`, centred.
- Page background `#0a0b0c`. Header `CARTALITH · ANDROID ENVIRONMENT` (`500 12px` mono, `letter-spacing:.24em`, `#e8ebec`) + `interactive — drag · pinch · rotate · long-press · edge-swipe` (`10.5px` mono, `#6f7478`).
- Footer: `Deep flows: map gestures · generator run · journey planner. All other menus navigable with mock actions. Chrome direction B · DCC palette · sharp at any dp size.`

Derived helpers: `frameW() = land ? d.h : d.w`, `frameH() = land ? d.w : d.h`.

---

## 2. The four tabs

`tabs=[…]`:

| id | Glyph | Label | Sheet title | Sheet subtitle |
|---|---|---|---|---|
| `map` | `▤` | `MAP` | `MAP` | `layers · style · annotation` |
| `gen` | `⌗` | `GENERATE` | `GENERATE` | `pipeline · seed {seed}` (pipe) / `sculpt · draft stamps` (sculpt) |
| `plan` | `➔` | `PLAN` | `PLAN` or `PLAN · STAGE {n}` | `journey · Vhal Serai → Port Amre` |
| `more` | `⋯` | `MORE` | from `_moreTitle()` (§6.5) | from `_moreTitle()` |

Tab colour: active `var(--acc)`, inactive `var(--sec)`. Active pill background `var(--wash)`, inactive `transparent`.

**Tab behaviour (`hTab`):** tapping the already-active tab sets `tab = null` (closes the sheet). Tapping a different tab sets it, and if `detent === 'peek'` bumps it to `'half'`. Then `_snapSheet()` and a canvas redraw.

Default state: `tab: null` — the app opens with **no sheet**, map full-bleed.

---

## 3. The bottom bar (portrait only)

Rendered when `scr === 'app' && !land`.

| Property | Value |
|---|---|
| Position | `absolute; left:0; right:0; bottom:0` |
| Height | **84 px** total |
| z-index | `14` |
| Background | `var(--pan2)` |
| Top border | `1px solid var(--hair2)` |
| Layout | column: tab row (`flex:1`, i.e. **66 px**) + gesture inset (**18 px**, `flex:none`) |
| Tab row padding | `4px 6px 0` |
| Tab cell | `flex:1` (⇒ frameW ÷ 4; 103 px on the 412 frame), column, centred, `gap:3px` |
| Active pill | `padding:4px 16px`, `border-radius:13px`, `background:var(--wash)` |
| Glyph | `font:14px 'IBM Plex Mono'` |
| Label | `font:9.5px 'IBM Plex Mono'`, `letter-spacing:.12em` |
| Gesture inset | 18 px band, centred pill **112 × 4**, `border-radius:2px`, `background:var(--bord)` |

The map is inset by exactly this bar (`bottom:84px`); the sheet sits *above* the bar (`bottom:84px`) and overlays the map.

**Landscape replaces the bar with a rail** — see §8.

---

## 4. The app bar

A single `pointer-events:none` container at `left:{chromeLeft}px; right:0; top:0; z-index:8` (`chromeLeft` = 0 portrait, 72 landscape). Interactive children re-enable `pointer-events:auto`.

### 4.1 Status band
`height:30px`, `padding:0 18px`, `font:10px 'IBM Plex Mono'`, colour `statusCol`.
Left: `9:41`. Right: `LTE ▮▮ 84%` with `letter-spacing:.14em`.

### 4.2 Control row
`display:flex; align-items:center; gap:8px; padding:4px 10px 0` → occupies y = 34…78.

| Element | Spec |
|---|---|
| **World pill** | `flex:1`, `padding:8px 14px`, `border-radius:20px`, `background:pillBg`, `border:1px solid var(--hair)`. Line 1 = world name, `font:500 11.5px mono`, `letter-spacing:.2em`, `var(--ink)`, ellipsised. Line 2 = `{seed} · {status}`, `font:9.5px mono`, `var(--dim)`, ellipsised. Not tappable. |
| **Search** | `44 × 44`, `border-radius:22px`, `background:pillBg`, `border:1px solid var(--hair)`, glyph `⌕` at `15px mono`, colour `var(--body)` |
| **Overflow** | identical, glyph `⋮` |

### 4.3 Overflow menu (`menuOpen`)
`position:absolute; right:10px; top:86px; width:230px; border-radius:18px; background:var(--pan); border:1px solid var(--bord); box-shadow:0 14px 34px rgba(0,0,0,.45); padding:6px 0`.

| Row (min-height 44, padding `0 16px`) | Right-hand value | Action |
|---|---|---|
| `Save project` | `{savedAt}` — `9.5px mono`, `var(--faint)` | sets `savedAt` to current `HH:MM`, closes menu, toast `Project saved · {HH:MM}` |
| `Theme` | `{dark\|light}` — `9.5px mono`, `var(--acc)` | flips `light`, closes menu |
| `Close world` | — | closes menu, `scr='picker'`, `tab=null` |

The menu also closes on any `pointerdown` on the map.

---

## 5. The sheet

Container: `z-index:12`, `display:flex; flex-direction:column`, `background:var(--pan)`, `box-shadow:0 -10px 30px rgba(0,0,0,.35)`.

### 5.1 Geometry

**Portrait:**
```
position:absolute; left:0; right:0; bottom:84px;
height: {open ? _detH(detent) : 0}px;
border-radius:22px 22px 0 0;
transition:height .28s cubic-bezier(.3,.9,.3,1)
```

**Landscape:**
```
position:absolute; top:0; bottom:0; right:0;
width: {open ? min(440, round(frameW()*0.46)) : 0}px;
border-left:1px solid var(--hair); border-radius:0;
transition:width .28s cubic-bezier(.3,.9,.3,1)
```

### 5.2 Detents — `_detH(det)`

Let `fh = land ? frameH() : frameH() - 84`.

| Detent | Formula | On 412×892 portrait |
|---|---|---|
| `peek` | **66** (constant) | 66 |
| `half` | `round(fh × 0.46)` | 372 |
| `full` | `fh − 96` | 712 |

Default detent: `'half'`.

### 5.3 Drag handle

The grab region is the whole header block above the scroller: `flex:none; touch-action:none; cursor:grab`.
- Handle row: `height:20px`, centred pill **42 × 4**, `border-radius:2px`, `background:var(--bord)`.
- Title row: `padding:0 14px 10px`, `gap:10px`, containing the optional back button, the title block, and the close button.

**Drag mechanics:**
| Phase | Behaviour |
|---|---|
| `pointerdown` | Disabled entirely in landscape (`_sd` returns). Otherwise: `preventDefault`, `setPointerCapture`, records `y0` and `h0 = _detH(detent)`, sets `transition:none` |
| `pointermove` | `h = clamp(h0 − (clientY − y0), 40, frameH() − 90)`; applied directly to `style.height` |
| `pointerup` | Restores `transition:height .28s cubic-bezier(.3,.9,.3,1)`. If `h < 44` → `tab = null` (sheet closes). Otherwise snap to whichever of `peek / half / full` has the smallest `|value − h|`, and write that height |

### 5.4 Header contents

| Element | Spec |
|---|---|
| Back `←` | `38×38`, radius 19, `background:var(--chip)`, `15px mono`, `var(--body)`. Visible when `(tab==='more' && moreStack.length>1) \|\| (tab==='plan' && planView!=='root')`. Pops `moreStack` by one, or returns `planView` to `'root'` |
| Title | `font:500 11px mono`, `letter-spacing:.2em`, `var(--acc)`, ellipsised |
| Subtitle | `font:9.5px mono`, `var(--dim)`, ellipsised |
| Close `✕` | `38×38`, radius 19, `background:var(--chip)`, `13px mono`, `var(--sec)` → `tab = null` |

### 5.5 Scroller
`flex:1; overflow-y:auto; overscroll-behavior:contain; padding:2px 14px 24px`.

---

## 6. Every reachable screen

### 6.0 Screen graph

```
picker ──(tap world | CREATE WORLD)──> app
app ──(⋮ ▸ Close world)──> picker
app + sheet: map | gen(pipe|sculpt) | plan(root|stage) | more(17 sub-screens)
overlays on app: search · inspector · new-world modal · overflow menu ·
                 undo-history popover · coach · toasts · measure strip ·
                 label bar · way card · sim strip · sample chip
```

---

### 6.1 World picker (`scr === 'picker'`)

Full-bleed column; **no bottom bar, no app bar**.

| Block | Spec |
|---|---|
| Status band | `height:30px`, `padding:0 20px`, `font:10px mono`, `var(--dim)`. `9:41` / `LTE ▮▮ 84%` (`letter-spacing:.14em`) |
| Header | `padding:22px 20px 14px`. Title `CARTALITH` — `font:500 15px mono`, `letter-spacing:.3em`, `var(--ink)`. Sub `worlds on this device · ~/Cartalith/Worlds` — `padding-top:5px`, `font:10.5px mono`, `var(--dim)` |
| List | `flex:1; overflow-y:auto; padding:0 14px 20px; gap:10px` |

**World card** — `border:1px solid var(--bord)`, `border-radius:20px`, `overflow:hidden`, `background:var(--pan2)`.
- Thumbnail: `height:92px`, gradient per world, `display:flex; align-items:flex-end; padding:10px 14px`.
- State chip: `font:9.5px mono`, `letter-spacing:.14em`, `padding:4px 9px`, `border-radius:11px`.
- Info row: `padding:12px 14px`, `gap:12px`. Name `font:500 12.5px mono`, `letter-spacing:.18em`, `var(--ink)`. Meta `padding-top:3px`, `font:10px mono`, `var(--dim)`. Chevron `›` `14px mono`, `var(--faint)`.

| # | name | seed | status (becomes `world.status`) | meta | state chip | thumb gradient | chip bg / col |
|---|---|---|---|---|---|---|---|
| 1 | `ELDRA` | `483920` | `finalized · atlas` | `483920 · 2 048² · edited 2 d ago · 1.6 GB` | `FINALIZED` | `linear-gradient(140deg,#2a3140 0%,#1a2230 52%,#141a24 100%)` | `rgba(224,163,74,.16)` / `#e0a34a` |
| 2 | `VHAREN REACH` | `129384` | `stages 01–07 resolved` | `129384 · 1 024² · edited 5 d ago · 410 MB` | `IN PROGRESS` | `linear-gradient(140deg,#31402a 0%,#22301a 52%,#1a2414 100%)` | `rgba(255,255,255,.10)` / `#c8cbcd` |
| 3 | `KESSA` | `774201` | `draft · stage 03` | `774201 · 512² · edited 3 w ago · 88 MB` | `DRAFT` | `linear-gradient(140deg,#402a31 0%,#301a22 52%,#24141a 100%)` | `rgba(255,255,255,.10)` / `#c8cbcd` |

Below the list:
| Row | Spec | Action |
|---|---|---|
| `＋ NEW WORLD` | `min-height:58px`, `border:1px dashed var(--bord)`, radius 20, `var(--acc)`, `font:500 11px mono`, `letter-spacing:.18em` | opens the New World modal with a fresh 6-digit seed and `extent:'region'` |
| `OPEN PROJECT .ZIP…` | `min-height:50px`, radius 20, `var(--sec)`, `font:10.5px mono`, `letter-spacing:.14em`, `background:var(--chip)` | toast `File picker — mock. Archives load from ~/Cartalith/Worlds.` |
| Footer | `Cartalith Mobile 0.9 · build 2611`, `font:9.5px mono`, `var(--faint)`, centred, `padding-top:6px` | — |

Tapping a card: `scr='app'`, adopts name/seed/status, and **resets the coach sequence to step 0** unless it is already `'done'`.

---

### 6.2 App screen — map layer

Map host: `position:absolute; top:0; {mapInset}; overflow:hidden; touch-action:none; cursor:grab; background:#12161b`
where `mapInset` = `left:0;right:0;bottom:84px` (portrait) / `left:72px;right:0;bottom:0` (landscape).
A full-size `<canvas>` fills it (`width:100%;height:100%;display:block`), sized to `clientWidth/Height × devicePixelRatio` by a `ResizeObserver`.

Top scrim (`pointer-events:none`): `left:0;right:0;top:0;height:110px`, `linear-gradient(180deg, {scrimCol} 0%, transparent 100%)`.

#### View model
`view = {cx, cy, s, r}`; initial `{cx:1035, cy:2270, s:0.42, r:0}`. World square is **0…4096 on both axes**; every distance is formatted as km, so **1 world unit = 1 km**. Scale clamp **[0.07, 14]**. Rotation unclamped, no snap.

#### Canvas draw order

| Layer | Spec (`lw(k) = k / s`) |
|---|---|
| Background fill | `P.bg` |
| Base tint | `biome` → `rgba(110,165,95,.055)` over the whole viewport; `political` → `rgba(150,115,205,.06)`; `relief` → none |
| Minor grid | every **128** units, `lineWidth lw(1)`, `P.minor` |
| Major grid | every **512** units, `lineWidth lw(1.4)`, `P.major` |
| World border | `strokeRect(0,0,4096,4096)`, `lineWidth lw(2)`, `P.major` |
| Overlay hatch (any overlay active) | diagonals every **96** along `x+y`, `lineWidth lw(1)`, `rgba(224,163,74,.10)` |
| Coordinate labels | X value every 512, `font ${11/s}px mono`, `P.ink`, offset `+6/s`, `+16/s`. **No Y labels.** |
| Route | polyline through `PLACES[0..6]`, `lineWidth lw(2.4)`, `P.acc`, `globalAlpha .85` |
| Selected stage (when `tab==='plan'`) | segment `sel → sel+1`, `lineWidth lw(5)`, `P.acc` |
| Committed ways | dashed `[10/s, 7/s]`, `lineWidth lw(1.6)`, `P.ink` |
| Way draft | same dash, `P.acc` |
| Places | radius `city 5.5/s`, `poi 3.6/s`, else `4.4/s`. User-placed fill `P.acc`, else `P.place`. POI = square rotated 45°; others circles. Name at `+9/s, +4/s`, `font ${11/s}px mono`, `P.ink` |
| Route stop diamonds | 7 stops, `r 2.6/s`, rotated 45°, filled `P.acc` |
| Labels | `font 500 ${13/s}px mono`, `P.acc` |
| Icons | `r 6/s`, stroke `P.acc`, `lineWidth lw(1.5)`; `diamond` rotated square, `circle`, `triangle` (apex up), `square` |
| Landmarks | `window.LM_GLYPHS[type]` path list via `Path2D`, stroke `P.place`, `globalAlpha .85`, translate `(x − 8·gk, y − 8·gk)`, `scale(gk)` where `gk = 1.05/s`, `lineWidth 1.2`, round caps/joins |
| Measure | dashed `[6/s, 5/s]`, `lineWidth lw(1.8)`, `P.acc`; vertex dots `r 3.4/s`; per-segment distance label at midpoint `+6/s` |
| Sample pin | square `r 7/s` rotated 45°, stroke `P.acc`, `lineWidth lw(2)` |
| Scale bar (screen space) | ladder `[10,20,50,100,200,500,1000,2000]`; picks the first `n` where `n·s ≥ 64` px, else 2000. Bar at `x:14, y:H−26`, `height 2`, width `n·s`; end caps `2×7` at `y:H−31`; label `fmtKm(n)` at `(14, H−34)`, `font 10px mono`, `P.ink` |
| Readout (screen space) | right-aligned at `(W−12, H−16)`: `{round(cx)} E · {round(cy)} N · {round(s*100)}% · {base}[ + {overlay}]` |

#### Style palettes — `_styleP()`

| Preset | `bg` | `minor` | `major` | `ink` | `acc` | `place` |
|---|---|---|---|---|---|---|
| `atlas` (default) | `#12161b` | `rgba(185,205,230,.09)` | `rgba(185,205,230,.17)` | `rgba(195,210,230,.55)` | `#e0a34a` | `#e6ebf2` |
| `parchment` | `#d9cfb6` | `rgba(80,60,30,.13)` | `rgba(80,60,30,.26)` | `rgba(60,45,22,.75)` | `#8a5a18` | `#3d2f16` |
| `physical` | `#16211b` | `rgba(170,215,180,.08)` | `rgba(170,215,180,.16)` | `rgba(190,220,195,.55)` | `#9ec27a` | `#d9e8d2` |
| `ink` | `#0b0c0d` | `rgba(255,255,255,.10)` | `rgba(255,255,255,.20)` | `rgba(255,255,255,.6)` | `#e8ebec` | `#f0f2f3` |

#### Seed data — `PLACES` (world coordinates)

| n | x | y | c |
|---|---|---|---|
| `Vhal Serai` | 620 | 2620 | `city` |
| `Kess Ford` | 760 | 2470 | `town` |
| `Thornwood` | 915 | 2350 | `town` |
| `High Saddle` | 1050 | 2210 | `waypost` |
| `Grey Vale` | 1215 | 2135 | `village` |
| `Lakemouth` | 1370 | 2050 | `town` |
| `Port Amre` | 1450 | 1930 | `city` |
| `Qet Oasis` | 2300 | 3100 | `oasis` |

The route uses the first **seven** only; `Qet Oasis` is searchable but off-route.

#### Distance formatting — `fmtKm(km)`
`val = units==='mi' ? km × 0.621371 : km`; then `val ≥ 100 → round(val)`, else `round(val×10)/10`; rendered `${val.toLocaleString('en-US')} ${units}`.

#### Terrain sample — `sampleData(w)` (deterministic mock)
`h(x,y,k) = (sin(x·.013 + k·7.1)·cos(y·.011 + k·3.7) + sin((x+y)·.006 + k)) / 2`

| Field | Formula |
|---|---|
| `elev` | `max(−410, round(900 + 1400·h(x,y,1) + 600·h(2.3x, 1.7y, 2)))` |
| `slope` | `abs(round(14·h(x,y,3)·10)/10)` |
| `aspect` | `round(360·abs(h(x,y,8)))` |
| `temp` | `round((22 − elev·0.0065 − abs(y−2048)·0.004)·10)/10` |
| `rain` | `max(60, round(900 + 700·h(x,y,5)))` |
| `biome` | `elev<0 → 'coastal water'`; `elev>1600 → 'montane'`; `rain>1100 → 'temperate forest'`; `rain>700 → 'steppe'`; else `'dry shrubland'` |
| `plate` | `['P-04 continental','P-07 continental','P-11 oceanic'][abs(round(h(x,y,4)·10)) % 3]` |
| `lith` | `['granite','sandstone','basalt','schist'][abs(round(h(x,y,6)·10)) % 4]` |
| `drain` | `elev<0 ? '—' : 'Kess basin'` |
| `soil` | `elev<0 ? '—' : 'loam · 1.2 m'` |
| `near` / `nearD` | nearest of `PLACES` by Euclidean world distance, rounded |

#### Sample chip (DOM, follows the pin)
Positioned at `toScreen(pin)`; `z-index:6`; inner `transform:translate(14px,-50%)`, `padding:9px 13px`, `border-radius:14px`, `background:var(--pan)`, `border:1px solid var(--bord)`, `box-shadow:0 6px 18px rgba(0,0,0,.4)`, `white-space:nowrap`.
- Line 1 (`10.5px mono`, `var(--ink)`): `{elev} m · {biome}`
- Line 2 (`9.5px mono`, `var(--dim)`): `{temp} °C · slope {slope}° · tap for inspector ›`
- Tap → opens the inspector.

#### Floating map furniture

| Element | Portrait | Landscape | Spec |
|---|---|---|---|
| FAB column | `right:12px; bottom:104px` | `right:12px; bottom:18px` | `z:9`, column, `gap:10px` |
| ↳ 3D/2D FAB | — | — | `48×48`, radius 24, `background:pillBg`, `border:1px solid var(--hair)`, label `2D`/`3D` at `12.5px mono`, `var(--acc)`. Toggles a label only; toast `3D viewport — mock · relief exaggeration in Preferences ▸ Graphics` / `2D viewport` |
| ↳ Recenter FAB | — | — | same box, glyph `⌖` at `16px mono`, `var(--body)`. Resets `view` to `{1035, 2270, 0.42, 0}` |
| Undo chip | `left:12px; bottom:104px` | `left:86px; bottom:18px` | `z:9`, `padding:11px 15px`, radius 22, `background:pillBg`, `border:1px solid var(--hair)`, `user-select:none`. Glyph `↶` `13px`, `var(--acc)`; label = last undo entry, `10px mono`, `var(--body)`. Shown only when `undoStack.length > 0` |
| Measure strip | `left:{chromeLeft}px; top:92px; margin:0 10px` | same | `z:9`, `padding:9px 13px`, radius 16, `background:pillBg`, `border:1px solid var(--hair)` |
| Label / Way bar | `bottom:98px` | `bottom:14px` | `z:11`, `padding:0 12px`; inner `background:var(--pan)`, `border:1px solid var(--bord)`, radius 18, `box-shadow:0 8px 22px rgba(0,0,0,.35)` |
| Sim strip | `bottom:98px` | `bottom:14px` | `z:10`, `padding:0 10px`; inner `padding:8px 12px`, radius 18, `background:pillBg`, `border:1px solid var(--hair)` |
| Coach mark | `bottom:100px` | `bottom:16px` | `z:9`, centred, `padding:0 20px`; inner `padding:10px 16px`, radius 18, `background:pillBg`, `border:1px solid var(--hair)`, `animation:tIn .3s ease` |
| Toast stack | `bottom:162px` | `bottom:78px` | `z:22`, column, centred, `gap:8px`, `pointer-events:none`. Each: `max-width:86%`, `padding:10px 16px`, radius 16, `background:pillBg`, `border:1px solid var(--bord)`, `font:10.5px/1.5 mono`, `var(--ink)`, centred, `animation:tIn .25s ease`, auto-removed after **2600 ms** |

**Undo history popover** — opens on a **520 ms** hold of the undo chip. `position:absolute; left:0; bottom:52px; width:220px`, radius 16, `background:var(--pan)`, `border:1px solid var(--bord)`, `box-shadow:0 12px 30px rgba(0,0,0,.45)`, `padding:4px 0`.
- Header `EDIT HISTORY · TAP TO ROLL BACK` — `padding:8px 14px`, `font:9px mono`, `letter-spacing:.18em`, `var(--dim)`.
- Rows: `min-height:40px`, `padding:0 14px`, `font:10.5px mono`, `var(--body)`. Label `{index+1} · {action}`. Newest first, **max 6 rows**.
- Tapping row `i` reverts every entry above index `i` (pops until `length === i`), then closes.
- A short tap (no hold) on the chip undoes one step; if the popover is open, a short tap closes it.

**Measure strip contents** (left→right): `MEASURE` (`10px mono`, `letter-spacing:.14em`, `var(--acc)`) · total (`11px mono`, `var(--ink)`) · `{n} pts · tap map to add` (`9.5px mono`, `var(--dim)`) · `CLEAR` (`var(--sec)`) · `DONE` (`var(--acc)`), both `padding:6px 4px`.
`CLEAR` empties the point list. `DONE` sets `tool='inspect'`, empties the list, and if >1 point toasts `Measured {total} along {n−1} segments`.

**Label bar contents:** `LABEL` (`9.5px mono`, `.14em`, `var(--acc)`) · text field (`flex:1`, `background:var(--chip)`, `border:1px solid var(--hair)`, radius 12, `padding:10px 12px`, `12px mono`, `var(--ink)`, placeholder `label text…`) · `✕` (`var(--sec)`, `padding:8px 2px`) · `ADD` (`font:500 10px mono`, `.12em`, `color:var(--accInk)`, `background:var(--acc)`, radius 14, `padding:9px 13px`).
`ADD` with empty/whitespace text just cancels. Otherwise commits the label and pushes undo entry `label · {text}`.

**Way card contents:** `WAY` (`9.5px mono`, `.14em`, `var(--acc)`) · `{n} pts · {length}` (`11px mono`, `var(--ink)`) · spacer · `CANCEL` (`var(--sec)`) · `COMMIT` (`font:500 10px mono`, `.12em`, `var(--accInk)` on `var(--acc)`, radius 14, `padding:9px 13px`).
Visible only when `tool==='way' && wayDraft.length > 0`. `COMMIT` with <2 points silently discards; otherwise commits, pushes undo `way · {length}`, toasts `Way committed · {length} — routes can now use it`.

**Sim strip contents:** play/pause `▶`/`⏸` in a `38×38` radius-19 `var(--wash)` circle (`13px mono`, `var(--acc)`) · `YEAR {n}` (`11px mono`, `var(--ink)`) · slider `min=-400 max=1200 step=1` (`flex:1`) · three speed labels `×1 ×10 ×100` (`9.5px mono`, `padding:6px 3px`; active `var(--acc)`, inactive `var(--faint)`) · `✕` (`11px mono`, `var(--sec)`).
Playing advances the year by `speed` every **600 ms**, capped at 1200. `✕` stops playback and hides the strip.

**Coach marks** — two steps, gated by prop `coachMarks` and `scr==='app'`:
| Step | Text |
|---|---|
| 0 | `Drag to pan · pinch to zoom · two fingers to rotate` |
| 1 | `Long-press samples the terrain · swipe in from the right edge for the inspector` |

Step 0 → 1 automatically after **more than 6** pan/pinch move events. Step 1 → done on a completed long-press. The `✕` advances 0→1 and 1→done. Completion writes `localStorage['cartalith.coach'] = 'done'` and is read back on construction.

---

### 6.3 Sheet — MAP tab

Column, `gap:14px`.

**§ TOOLS** — header `TOOLS · ARMING DROPS THE SHEET TO A PEEK` (`9.5px mono`, `letter-spacing:.2em`, `var(--dim)`, `padding:6px 2px 8px`).
Chips: `min-height:44px`, `padding:0 16px`, `border-radius:22px`, `gap:8px`, `font:10px mono`, `letter-spacing:.12em`. Off-background is `var(--chip)`.

| id | Glyph | Label | Tap-on-map does |
|---|---|---|---|
| `inspect` | `➤` | `INSPECT` | clears the sample pin if one exists |
| `measure` | `⟟` | `MEASURE` | appends a measure vertex |
| `label` | `⌖` | `LABEL` | opens the label draft bar at that point |
| `icon` | `◇` | `ICON` | stamps the current icon variant; undo entry `icon {variant}` |

Selecting any tool other than `inspect` forces `detent = 'peek'`.

**Icon variant row** (only while `tool==='icon'`), `padding-top:10px`, `gap:8px`: four cells `48 × 44`, radius 16, `font:15px mono` — `diamond ◇`, `circle ○`, `triangle △`, `square □`. Trailing hint `tap the map to stamp` (`9.5px mono`, `var(--faint)`).

**§ LAYERS** — header `LAYERS`. Group captions `font:9px mono`, `letter-spacing:.18em`, `var(--faint)`, `padding:6px 2px 4px`.
Row: `min-height:46px`, `padding:0 12px`, `border-radius:14px`, `gap:12px`; dot `●`/`○` at `11px mono` (`var(--acc)`/`var(--faint)`); label `flex:1` (`var(--ink)` on, `var(--body)` off); background `var(--wash)` when on. Trailing note column exists but every note is empty.

| Group | Rows | Kind |
|---|---|---|
| `SURFACE · BASE` | `Relief` (default), `Biome`, `Political` | radio — sets `base` |
| `TERRAIN FIELDS · OVERLAY` | `Elevation`, `Slope`, `Flow accumulation` | single-select toggle — tapping the active one clears `overlay` |
| `CLIMATE · OVERLAY` | `Temperature`, `Rainfall` | same `overlay` slot |

**§ STYLE** — header row: `STYLE` plus, when `styleCustom`, `custom — edited since preset` (`9px mono`, `var(--warn)`).
Preset chips: `min-height:42px`, `padding:0 16px`, radius 21 — `ATLAS` (default), `PARCHMENT`, `PHYSICAL`, `INK`. Selecting one clears `styleCustom`.

Sub-header `COLOUR RAMP · TERRAIN` (`9px mono`, `.18em`, `var(--faint)`).
Ramp row: `min-height:44px`, `padding:0 10px`, radius 14, `gap:12px`, `border:1px solid {acc|hair}`, background `var(--wash)` when selected. Swatch `flex:1; height:14px; border-radius:7px`. Name column `width:76px`, `10px mono`. Selecting a ramp sets `styleCustom = true`.

| Ramp | CSS gradient |
|---|---|
| `Earth` (default) | `linear-gradient(90deg,#2c4a5e,#4a7a52,#8aa05a,#c9b57a,#e8e0c8)` |
| `Elevation` | `linear-gradient(90deg,#1d3557,#457b9d,#a8dadc,#f1faee,#e63946)` |
| `Atlas` | `linear-gradient(90deg,#33415c,#5c677d,#979dac,#d9dcd6,#f5f3ef)` |
| `Mono` | `linear-gradient(90deg,#111,#555,#999,#ddd)` |
| `Imhof` | `linear-gradient(90deg,#5a7d8c,#8fa98a,#c9c489,#e6d3a3,#f2ead9)` |
| `Ice` | `linear-gradient(90deg,#274060,#5a7fa8,#a8c6de,#e8f1f8)` |
| `Dark ice` | `linear-gradient(90deg,#0d1b2a,#1b3a5c,#3c6e91,#89b6d5)` |
| `Desert` | `linear-gradient(90deg,#7a4a2b,#b07d46,#d9b380,#f0e0bd)` |
| `Dark atlas` | `linear-gradient(90deg,#0e1116,#232a36,#48526a,#8892aa)` |

Footer: `Presentation only — nothing here alters world data or marks a generation stage stale.` (`9.5px/1.6 mono`, `var(--faint)`)

---

### 6.4 Sheet — GENERATE tab

**Mode switch** — two cells, `flex:1`, `min-height:44px`, radius 22, `font:500 10px mono`, `letter-spacing:.16em`: `PIPELINE` (default) and `SCULPT`.

#### 6.4.1 PIPELINE

**Seed row** — `padding:10px 12px`, radius 16, `background:var(--chip)`, `gap:10px`. `SEED` (`9.5px mono`, `.16em`, `var(--dim)`) · value (`font:500 13px mono`, `var(--ink)`) · dice button `42 × 40`, radius 14, `background:var(--wash)`, `var(--acc)`, glyph `⚄`.
Dice → random integer in `[100000, 999999]` as a string; marks the pipeline stale **from stage 1**.

**Progress panel** (while running) — `border:1px solid var(--acc)`, radius 18, `padding:13px 14px`, `gap:9px`.
- Title `{NN} · {stage name}` (`font:500 10.5px mono`, `.14em`, `var(--acc)`); percentage right (`10px mono`, `var(--dim)`) = `round((i·100 + min(pct,100)) / 10)%`.
- Bar: `height:5px`, radius 3, track `var(--chip)`, fill `var(--acc)` at `min(pct,100)%`.
- Log: last **3** lines, `9.5px mono`, `var(--dim)`; format `{NN} {stage} — resolved · {0.4–2.6}s`.
- `CANCEL`: `align-self:flex-end`, `padding:8px 14px`, radius 16, `background:var(--chip)`, `var(--sec)`, `10px mono`, `.12em`. Toast `Generation cancelled — nothing was written`.

Run mock: `setInterval` at **260 ms**, `pct += 14 + rand·22`; on ≥100 emit a log line and advance. Completes after stage index 10, sets `lastRun = HH:MM`, `world.status = 'stages 01–10 resolved'`, toast `World regenerated — stages {NN} → 10 resolved`.

**Idle block**
- Stale note (when stale): `padding:11px 14px`, `border:1px solid rgba(224,168,64,.4)`, radius 14, `font:10px/1.6 mono`, `var(--warn)`. Text: `Stage {NN} edited — stages {NN} → 10 are stale. Fields owned by stale stages read — until re-run.`
- Primary button: `min-height:52px`, radius 26, `background:var(--acc)`, `color:var(--accInk)`, `font:500 11px mono`, `.18em`. Label = `REGENERATE {NN} → 10` when stale **and** `lastRun !== '—'`, else `GENERATE WORLD`.
- Meta row (`9.5px mono`, `var(--faint)`, `padding:0 4px`): left `last run · {lastRun}`, right `10 stages · dependency order`.

**Stage names** — `GENSTAGES`:
`Planet`, `Extent & scale`, `World structure`, `Tectonics`, `Volcanism & impacts`, `Erosion`, `Hydrology`, `Climate`, `Ecology & biomes`, `Resources & soils`.

**Accordion groups** — `border:1px solid var(--hair2)`, radius 18, `overflow:hidden`.
Header: `min-height:50px`, `padding:0 14px`, `gap:10px`. Number span `width:22px`, `9.5px mono`, `var(--warn)` when stale else `var(--faint)`. Name `flex:1`, `12.5px`, `var(--ink)`. State `9px mono`, `var(--faint)`: `stale` / `resolved`. Chevron `⌄` open / `›` closed.
Body: `border-top:1px solid var(--hair2)`, `padding:6px 14px 12px`, `gap:2px`.

Only **eight** groups exist — stages 05 and 10 are deliberately absent.

| Group | num | name | default open | key | Label | Type | min | max | step | unit | Default |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `g1` | `01` | `Planet` | **yes** | `gravity` | `Gravity` | range | 0.5 | 2 | 0.01 | ` g` | 1.0 |
| | | | | `dayLen` | `Day length` | range | 6 | 48 | 1 | ` h` | 24 |
| | | | | `tilt` | `Axial tilt` | range | 0 | 45 | 0.1 | `°` | 23.4 |
| `g2` | `02` | `Extent & scale` | no | `res` | `Working resolution` | seg `512 · 1024 · 2048 · 4096` | | | | | `2048` |
| | | | | `seaLevel` | `Sea level` | range | 0 | 100 | 1 | ` %` | 42 |
| | | | | `peakAlt` | `Peak altitude` | range | 1000 | 9000 | 100 | ` m` | 4000 |
| `g3` | `03` | `World structure` | no | `archetype` | `Archetype` | seg `Earthlike · Supercontinent · Islands · Rift` | | | | | `Earthlike` |
| | | | | `continentality` | `Continentality` | range | 0 | 1 | 0.01 | — | 0.30 |
| | | | | `fragmentation` | `Fragmentation` | range | 0 | 1 | 0.01 | — | 0.50 |
| `g4` | `04` | `Tectonics` | no | `plates` | `Plates` | range | 4 | 30 | 1 | — | 14 |
| | | | | `drift` | `Drift` | range | 0.2 | 3 | 0.05 | ` ×` | 1.0 |
| | | | | `energy` | `Tectonic energy` | range | 0 | 1 | 0.01 | — | 0.60 |
| `g6` | `06` | `Erosion` | no | `eroStrength` | `Erosion strength` | range | 0 | 1 | 0.01 | — | 0.60 |
| | | | | `streamPower` | `Stream-power carve` | range | 0 | 1 | 0.01 | — | 0.45 |
| `g7` | `07` | `Hydrology` | no | `riverDensity` | `River density` | range | 0 | 1 | 0.01 | — | 0.50 |
| | | | | `streamOrder` | `Min stream order` | range | 1 | 8 | 1 | — | 3 |
| `g8` | `08` | `Climate` | no | `eqTemp` | `Equator temp` | range | 10 | 40 | 1 | ` °C` | 28 |
| | | | | `poleTemp` | `Pole temp` | range | −40 | 5 | 1 | ` °C` | −18 |
| | | | | `rainfall` | `Rainfall` | range | 0.2 | 2 | 0.05 | ` ×` | 1.0 |
| `g9` | `09` | `Ecology & biomes` | no | `ecotone` | `Ecotone sharpness` | range | 0 | 1 | 0.01 | — | 0.50 |
| | | | | `riversBiome` | `Rivers in biome view` | toggle | | | | | on |

**Field rendering:**
- *Segmented*: label `10px mono`, `var(--sec)`, `padding-bottom:7px`; option chips `min-height:38px`, `padding:0 13px`, radius 19, `gap:6px`, `font:10px mono`.
- *Range*: label left (`10px mono`, `var(--sec)`), value right (`11px mono`, `var(--ink)`), then `[− button][slider][＋ button]` with `gap:8px`. Value display = `step<1 ? val.toFixed(2) : round(val).toLocaleString('en-US')`, then the unit suffix. Stepper result is rounded to 2 decimals and clamped to `[min,max]`.
- *Toggle*: `min-height:44px`, label `flex:1` `10px mono` `var(--sec)`, small toggle right.

**Every edit marks stale from that stage** (`stale = min(existing, stage)`).

Footer: `Editing a stage marks everything downstream stale. Volcanism (05) and Resources (10) run with defaults. GPU, LOD and render quality live under MORE ▸ Preferences.`

#### 6.4.2 SCULPT

**§ GEOLOGICAL FEATURE** — header `9.5px mono`, `.2em`, `var(--dim)`.
Chip: `min-height:42px`, `padding:0 13px`, radius 21, `gap:7px`, `font:10px mono`; contains a `14×14` SVG (`viewBox 0 0 16 16`, `fill:none`, `stroke:currentColor`, `stroke-width:1.2`, round caps/joins) plus the id text. Wrapper `gap:7px; flex-wrap:wrap`.

| id | SVG path `d` | Hint line (shown under the chip row) |
|---|---|---|
| `Mountains` (default) | `M2 12 L6 5 L9 9 L12 3 L14 12` | `stroke — Height 0.42 · Peak sharpness 1.5 · Ruggedness 0.55` |
| `Hills` | `M2 11 Q5 7 8 11 T14 11` | `stroke — Amplitude 0.11 · Rolling freq 1.4 · Softness 0.7` |
| `Ridge` | `M2 12 L8 4 L14 12 M8 4 L8 12` | `stroke — Height 0.15 · Width 0.28 · Detail 1.5` |
| `Plateau` | `M2 12 L5 6 L11 6 L14 12 M5 6 L11 6` | `stroke · set — Rise 0.26 · Terraces 4 · never lowers terrain` |
| `Cliff` | `M2 5 L8 5 L8 12 L14 12` | `stroke — Rise 0.22 · Steepness 0.75 · high side left` |
| `Canyon` | `M2 4 L6 4 L7 11 L9 11 L10 4 L14 4` | `stroke · negative — Depth 0.18 · Walls 0.7 · Meander 0.35` |
| `Valley` | `M2 5 Q8 14 14 5` | `stroke · negative — Depth 0.14 · Width 0.85 · Meander 0.3` |
| `River` | `M2 8 Q5 5 8 8 T14 8` | `stroke · set — Width 7 px · Depth 0.09 · writes riverMask` |
| `Lake` | `M3 9 Q8 13 13 9 Q8 6 3 9` | `radial — Depth 0.13 · Shore 0.25 · fills lakeMask` |
| `Basin` | `M3 7 Q8 12 13 7 M5 6 Q8 9 11 6` | `stroke · negative — Depth 0.1 · endorheic, no outlet` |
| `Coastline` | `M2 10 L4 8 L6 10 L8 7 L10 10 L12 8 L14 10` | `stroke · set — Amount 0.85 · pulls toward sea level` |
| `Volcano` | `M4 12 L7 5 L9 5 L12 12 M7 5 L8 7 L9 5` | `radial — Cone 0.45 · Crater 0.5 · Radius 110 px` |
| `Freehand` | `M3 13 L11 4 L13 6 L5 13 Z` | `continuous drag — Raise · Lower · Smooth · Amount 0.12` |

**§ PRESETS** — chips `min-height:38px`, `padding:0 12px`, radius 19, `font:9.5px mono`, `gap:7px`.

| Preset | Seeds feature |
|---|---|
| `Rolling Hills` | `Hills` |
| `Alps` | `Mountains` |
| `Rockies` | `Mountains` |
| `Badlands` | `Canyon` |
| `Volcanic Isle` | `Volcano` |
| `Mesa` | `Plateau` |
| `Karst` | `Hills` |
| `Glacial Valley` | `Valley` |

Tapping a preset sets the feature and toasts `{preset} — parameters seeded for {feature} (preset never paints)`. Tapping a feature directly clears the active preset.

**§ BRUSH · GLOBAL**

| key | Label | min | max | step | Default | Display |
|---|---|---|---|---|---|---|
| `size` | `Brush size` | 6 | 200 | 1 | 64 | `{size} px · {fmtKm(size×2)}` |
| `hard` | `Hardness` | 0 | 1 | 0.01 | 0.35 | `{v.toFixed(2)}` |
| `inten` | `Intensity` | 0 | 1.5 | 0.01 | 1.0 | `{v.toFixed(2)} ×` |

(These sliders have no stepper buttons; `width:100%`.)

**Stamp block**
- `＋ ADD DRAFT STAMP (MOCK STROKE)` — `min-height:48px`, radius 24, `border:1px dashed var(--acc)`, `var(--acc)`, `font:500 10px mono`, `.16em`. Increments the stamp count and pushes undo `stamp · {feature}`.
- Status row: note `flex:1` `10px mono` — `no draft stamps` (`var(--faint)`) or `{n} draft stamp(s) — uncommitted` (`var(--acc)`); `DISCARD` (`padding:11px 14px`, radius 18, `var(--chip)`, `var(--sec)`); `✓ COMMIT` (`padding:11px 16px`, radius 18, `var(--acc)`, `var(--accInk)`, `font:500 10px`, `.12em`).
- `COMMIT` with 0 stamps → toast `Nothing to commit — add a stroke first`. Otherwise clears stamps, **marks stale from stage 6**, toast `Baked {n} stamp(s) — erosion → hydrology → climate re-run once`.
- `DISCARD` → clears stamps, toast `Draft discarded — heightfield untouched`.
- Footer: `Strokes accumulate as live procedural stamps; nothing touches the heightfield until commit, which re-runs erosion → hydrology → climate once. Locked while finalized.`

---

### 6.5 Sheet — PLAN tab

Two views: `planRoot` (`planView==='root'`, default) and `planStage`.

#### Fixed route — `STAGES`

| i | name | terr | biome | km | ascent m | water |
|---|---|---|---|---|---|---|
| 0 | `Vhal Serai → Kess Ford` | `plains` | `steppe` | 118 | 240 | no |
| 1 | `Kess Ford → Thornwood` | `forest` | `temperate` | 92 | 410 | no |
| 2 | `Thornwood → High Saddle` | `mountain` | `montane` | 64 | 1180 | no |
| 3 | `High Saddle → Grey Vale` | `hills` | `montane` | 77 | 0 | no |
| 4 | `Grey Vale → Lakemouth` | `plains` | `temperate` | 103 | 0 | no |
| 5 | `Lakemouth → Port Amre` | `water` | `lake` | 46 | 0 | **yes** |

Total 500 km; `routeTotal` renders `{fmtKm(500)} · 7 stops`.

#### 6.5.1 Journey model (`_journey()`)

Effective value per stage = `ov[i][key] ?? party[key]`.

| Table | Values |
|---|---|
| `paceBase` (km/day at 8 h) | `Easy 18`, `Steady 24`, `Forced 30` |
| `terrF` | `plains 1`, `forest .8`, `hills .75`, `mountain .55`, `water 1` |
| `seasonW` (auto weather) | `spring .85`, `summer 1`, `autumn .9`, `winter .7` |
| `wF` (explicit weather) | `clear 1`, `rain .85`, `storm .6`, `snow .5` |
| `roadF` | `paved 1.15`, `tracks 1`, `trail .9`, `none .8` |
| `forageMul` | `none 1`, `modest .85`, `active .7` |
| `RESUPPLY` | `Kess Ford`, `Thornwood`, `Lakemouth` |

Per stage:
- water: `kmd = 60`; `days = km/60 + 0.5`
- land: `kmd = paceBase[pace] × (hours/8) × terrF[terr] × wf × roadF[road]`; `days = km / max(4, kmd)`; then `days ×= 1 + ascent/2000`
- `block = 'seasonal closure'` when land **and** `terr==='mountain'` **and** `closures` **and** `season==='winter'`

Load and penalty:
```
suppliesKg = carryFood ? supplies × groupSize × 1.8 : 0
load       = cargo + suppliesKg
animals    = donkeys + mules + camels + horses
cap        = donkeys·60 + mules·80 + camels·140 + horses·90
           + carts·400 + wagons·800 + travois·40 + sleds·120
capPct     = cap > 0 ? load/cap : 9
speedPen   = 1;  capPct > 0.90 → 0.85;  capPct > 1.15 && !promote → 0.70
non-blocked stage days /= speedPen
```

Totals: `travel = Σ days`; `restEvery = rest==='auto' ? 7 : parseInt(digits(rest)) || 7`; `rest = floor(travel/restEvery)`; `lay = Σ layover days`; `total = travel + rest + lay`.

Supply gap: walking the stages accumulating days, closing the run at every stage whose destination is in `RESUPPLY` and at the final stage; the longest run wins. `effGap = gap × forageMul[foraging]`; `carried = carryFood ? supplies : 2`.

Cost (currency suffix ` cr`):
```
food   = round(0.4 × groupSize × total)
fodder = round(0.3 × animals   × total)
wages  = round(0.5 × groupSize × total)
ferry  = round(Σ water-stage km × 0.6)     // = round(46 × 0.6) = 28
tolls  = road === 'paved' ? 26 : 12
upkeep = round(0.12 × animals × total)
cost   = food + fodder + wages + ferry + tolls + upkeep
```

Verdict, first match wins:

| # | Condition | `state` | Reason text | Action chips |
|---|---|---|---|---|
| 1 | any stage blocked | `impossible` | `High Saddle is closed by seasonal closures in winter. Depart in another season, ignore closures, or re-route land-only.` | `DEPART AUTUMN` (`seasonAutumn`), `IGNORE CLOSURES` (`closuresOff`) |
| 2 | `capPct > 1.15` | `impossible` | `Load {load} kg exceeds carry capacity {cap} kg by more than 15%. Shed cargo or add carriers.` | `CARGO −25%` (`cargoDown`), `+4 MULES` (`mulesUp`) |
| 3 | `effGap > carried` | `feasible — strained` | `Supply gap {where} runs {effGap} effective days but only {carried} days are carried. Add supplies, forage harder, or plan a resupply layover.` | `SUPPLIES +6 D` (`suppliesUp`), `FORAGING ACTIVE` (`forageUp`) |
| 4 | `capPct > 0.9` | `feasible — strained` | `Load sits at {pct}% of capacity — speed penalty applied to every land stage.` | `CARGO −25%`, `+4 MULES` |
| 5 | otherwise | `feasible` | `All stages passable. Longest supply run {where} at {effGap} effective days against {carried} carried.` | none |

Action semantics: `seasonAutumn` → `season='autumn'`; `closuresOff` → `closures=false`; `suppliesUp` → `min(60, supplies+6)`; `forageUp` → `foraging='active'`; `cargoDown` → `round(cargo×0.75)`; `mulesUp` → `mules+4`. All then toast `Applied — verdict recomputed`.

Verdict colour: `impossible → var(--block)`, `feasible → var(--good)`, strained → `var(--warn)`.
Verdict card background: `impossible → rgba(194,106,96,.07)`, `feasible → transparent`, strained → `rgba(224,168,64,.06)`.

Number formatter `fd(v,d)`: `v ≥ 100 → round(v).toLocaleString('en-US')`, else `(round(v×10)/10).toFixed(d ?? 1)`.

#### 6.5.2 PLAN root layout

**Verdict card** — `border:1px solid {verCol}`, radius 20, `padding:14px 15px`, `gap:8px`.
- State chip: `font:500 9.5px mono`, `.16em`, `border:1px solid {verCol}`, radius 11, `padding:4px 10px`. Text = uppercased state (`FEASIBLE`, `FEASIBLE — STRAINED`, `IMPOSSIBLE`).
- Right: `confidence ± {fd(total×0.11, 0)} d` — `9.5px mono`, `var(--faint)`.
- Number: `font:500 34px mono`, `var(--ink)` (`—` when impossible) + ` calendar days` at `12px mono`, `var(--dim)`.
- Split line: `{travel} travel · {rest} rest · {lay} layover` — `10px mono`, `var(--sec)`.
- Reason: `10.5px/1.6 mono` in `verCol`.
- Action chips: `min-height:38px`, `padding:0 13px`, radius 19, `font:9.5px mono`, `.08em`, `border:1px solid var(--bord)`, `color:var(--ink)`, `background:var(--chip)`.

**Route section** — header `ROUTE · VHAL SERAI → PORT AMRE` (`9.5px mono`, `.2em`, `var(--dim)`) and right `{500 km} · 7 stops` (`9.5px mono`, `var(--faint)`).
Stage row: `min-height:52px`, `padding:0 12px`, radius 16, `gap:11px`, `border:1px solid {block?--block : sel?--acc : --hair2}`, background `var(--wash)` when selected.

| Column | Content |
|---|---|
| dot (`9.5px mono`) | `✕` if blocked (`var(--block)`), `≈` if water (`var(--water)`), else `●` (`var(--acc)` selected / `var(--faint)`) |
| name (`12px`, `var(--ink)`) | stage name, ellipsised |
| sub (`9.5px mono`, `var(--dim)`) | `{terr} · {biome} · {km} · +{asc} m` (ascent segment omitted when 0) |
| days (`11px mono`, `var(--ink)`) | `{fd(days)} d`, or `—` when blocked |
| override note (`9px mono`) | `{n} override(s)` in `var(--acc)`, or `inherits` in `var(--faint)` |
| chevron | `›` `var(--faint)` |

Tapping a row selects the stage and switches to the stage detail view.

**§ PARTY FORM** — accordions, `border:1px solid var(--hair2)`, radius 18. Header `min-height:50px`, `padding:0 14px`; name `flex:1`, `12.5px`, `var(--ink)`; note `9px mono`, `var(--faint)`; chevron `⌄`/`›`.

| Group id | Title | Default open | Header note |
|---|---|---|---|
| `traveler` | `Traveler` | **yes** | `{groupSize} · {pace lowercased}` |
| `season` | `Season & weather` | no | `{season} · {weather}` |
| `carriage` | `Carriage` | no | `auto · {animals} animals` or `manual` |
| `cond` | `Route conditions` | no | `{road}` |
| `stops` | `Stops · layover days` | no | `{lay} layover d` |

| Group | key | Label | Type / options | min–max/step | Default | Note |
|---|---|---|---|---|---|---|
| traveler | `groupSize` | `Group size` | range ` people` | 1–200 / 1 | 12 | |
| | `pace` | `Pace` | seg `Easy · Steady · Forced` | | `Steady` | |
| | `hours` | `Hours per day (land)` | range ` h` | 4–12 / 0.5 | 8 | |
| | `cargo` | `Trade cargo` | range ` kg` | 0–5000 / 25 | 900 | |
| | `supplies` | `Supplies carried` | range ` days` | 0–60 / 1 | 10 | `fodder ceiling: a mule carries ~9 days of its own fodder at this grazing` |
| | `carryFood` | `Carry food` | toggle | | on | `off = live off the land` |
| | `grazing` | `Grazing` | seg `none · sparse · normal · rich` | | `normal` | |
| | `foraging` | `Foraging` | seg `none · modest · active` | | `modest` | |
| season | `season` | `Season at departure` | seg `spring · summer · autumn · winter` | | `autumn` | |
| | `weather` | `Weather` | seg `auto · clear · rain · storm · snow` | | `auto` | `auto = weighted by the season` |
| | `drift` | `Season drift during journey` | toggle | | on | |
| | `rest` | `Rest days` | seg `auto · 1 in 5 · 1 in 7 · 1 in 10` | | `auto` | |
| carriage | `carriage` | `Carriage` | seg `auto · manual` | | `auto` | `auto computes counts terrain × biome, km-weighted` |
| | `mode` | `Transport mode` | seg `walking · baggage train · wagon train · riders` | | `baggage train` | |
| | `mount` | `Mount` | seg `horse · pony · camel · mule` | | `horse` | |
| | `vessel` | `Vessel` | seg `river barge · coaster · raft` | | `river barge` | |
| | `donkeys` | `Donkeys` | range **auto-dimmed** | 0–60 / 1 | 0 | |
| | `mules` | `Mules` | range **auto-dimmed** | 0–60 / 1 | 6 | |
| | `camels` | `Camels` | range **auto-dimmed** | 0–60 / 1 | 0 | |
| | `horses` | `Horses` | range **auto-dimmed** | 0–60 / 1 | 8 | |
| | `carts` | `Carts` | range **auto-dimmed** | 0–20 / 1 | 2 | |
| | `wagons` | `Wagons` | range **auto-dimmed** | 0–20 / 1 | 0 | |
| | `travois` | `Travois` | range **auto-dimmed** | 0–20 / 1 | 0 | |
| | `sleds` | `Sleds` | range **auto-dimmed** | 0–20 / 1 | 0 | |
| | `promote` | `Auto-promote Walking → Baggage train when overloaded` | toggle | | on | |
| cond | `road` | `Road quality` | seg `paved · tracks · trail · none` | | `tracks` | |
| | `infra` | `Infrastructure` | seg `none · fords · fords+ferries · full` | | `fords+ferries` | |
| | `dWater` | `Desert water` | seg `auto · oasis chain · carry all` | | `auto` | |
| | `closures` | `Respect seasonal closures` | toggle | | on | |
| stops | `lay:Kess Ford` | `Kess Ford` | range ` d` | 0–14 / 1 | 0 | |
| | `lay:Thornwood` | `Thornwood` | range ` d` | 0–14 / 1 | 1 | |
| | `lay:Grey Vale` | `Grey Vale` | range ` d` | 0–14 / 1 | 0 | |
| | `lay:Lakemouth` | `Lakemouth` | range ` d` | 0–14 / 1 | 2 | |

**Auto-dimmed** = when `carriage === 'auto'`, the row gets `opacity:.55; pointer-events:none` and its value renders as `auto · {n}{unit}`.
Range display in this form is always `round(val).toLocaleString('en-US') + unit` (no decimals — so `hours` at 8.5 renders `9 h`). Only the `supplies` row renders its note.

**§ RESULTS** — accordions, `border:1px solid var(--hair2)`, radius 18; header `min-height:48px`, `padding:0 14px`, name `12px` `var(--ink)`, summary `10px mono`, chevron. Body rows `min-height:34px`, key `9.5px mono` `var(--dim)`, value `10.5px mono` right, default `var(--ink)`.

| id | Title | Default open | Summary | Rows |
|---|---|---|---|---|
| `rTime` | `Time` | **yes** | `{fd(total,0)} d` | `TRAVEL` `{fd(travel)} d` · `REST` `{rest} d` · `LAYOVERS` `{lay} d` · `MEAN · BEST · WORST` `{total} · {total×0.92} · {total×1.14}` · `ARRIVAL SEASON` (`drift && total>80 → 'winter'`, else the departure season) |
| `rLoad` | `Load` | no | `{round(capPct×100)}%` | `CARGO` `{cargo} kg` · `SUPPLIES` `{suppliesKg} kg` · `CAPACITY` `{cap} kg` · `CARRIERS` `{animals} animals · {carts+wagons} vehicles` · `SPEED PENALTY` `× 0.85` in `var(--warn)` when `capPct>0.9`, else `none` |
| `rSupply` | `Supply reach` | no | `gap!` (`var(--warn)`) / `ok` (`var(--good)`) | `CARRIED` `{carried} d` · `LONGEST GAP` `{effGap} d · {where}` · `FORAGING OFFSET` `× {1\|.85\|.7}` · `RESUPPLY AT` `Kess Ford · Thornwood · Lakemouth` |
| `rCost` | `Cost` | no | `{cost} cr` | `FOOD · FODDER` · `WAGES` · `TOLLS · FERRY` · `ANIMAL UPKEEP` · `PER KM · PER DAY` `{cost/500} · {cost/total} cr` (1 dp) |
| `rVessel` | `Vessels` | no | `1 leg` | `LAKEMOUTH → PORT AMRE` `{vessel}` · `HOLD USED` `{round(load/40)}%` · `SAILING WINDOW` `closed — winter` in `var(--block)` when winter, else `open · {season}` |

Load bar (only in `rLoad`, only while open): `height:6px`, radius 3, track `var(--chip)`; fill width `min(100, round(capPct×100))%`, colour `var(--warn)` when `capPct>0.9` else `var(--acc)`; a `1.5px` `var(--warn)` tick at `left:90%`; caption `overload threshold at 90%` (`8.5px mono`, `var(--faint)`).

Footer: `Auto fields show auto · resolved value. The route line and the selected stage are highlighted on the map behind this sheet.`

#### 6.5.3 PLAN stage detail (`planView === 'stage'`)

Sheet title becomes `PLAN · STAGE {sel+1}`; the back button returns to root.

**Header card** — `border:1px solid var(--hair)`, radius 18, `padding:12px 14px`.
- Name (`font:500 12.5px mono`, `var(--ink)`): `STAGE {i+1} · {stage name}`
- Sub (`10px mono`, `var(--dim)`): `{terr} · {biome} · {km} · ascent +{asc} m · {n} override(s)`
- Note (`10px mono`): `BLOCKED — {reason}` in `var(--block)`, else `{days} days · {kmd} km/day · arrives day {cumulative}` in `var(--sec)`

**Bulk chips** — `min-height:38px`, `padding:0 13px`, radius 19, `font:9.5px mono`, `border:1px solid var(--hair)`, `var(--sec)`, `background:var(--chip)`:
- `CLEAR OVERRIDES` → empties this stage's overrides, toast `Overrides cleared — stage inherits the party form`
- `COPY TO ALL LAND STAGES` → replaces every non-water stage's override set with a copy of this one; toast `Copied {n} override(s) to all land stages`

**Override field rows** — `border-left:2px solid {set ? var(--acc) : var(--hair2)}`, `padding-left:12px`, `margin:3px 0`.
Header line: label left (`10px mono`, `var(--sec)`); right group (`gap:10px`) = state note (`9px mono`) plus, when set, a `CLEAR` link (`9px mono`, `var(--sec)`, `padding:4px 2px`).

State note precedence:
1. not applicable → `— · {why}` in `var(--faint)`, and the whole row gets `opacity:.45; pointer-events:none`
2. override set → `override set` in `var(--acc)`
3. `weather` and value `auto` → `Auto (snow-weighted)` when season is winter, else `Auto ({season})`
4. otherwise → `Inherit ({party value})` in `var(--faint)`

`why` strings: `vessel → 'land stage'`; `dWater → 'not a desert stage'`; everything else → `'water leg'`.

Controls in this view: segmented chips `min-height:36px`, `padding:0 11px`, radius 18, `font:9.5px mono`; ranges are slider + right-aligned `84px` value (`10.5px mono`, `var(--ink)`) with **no stepper buttons**; toggles show `{label} · on|off` at `10.5px mono` `var(--ink)`, `min-height:40px`.

The 15 override fields, in order:

| key | Label | Type | Range | Disabled when |
|---|---|---|---|---|
| `mode` | `Travel mode` | seg `walking · baggage train · wagon train · riders` | | water stage |
| `groupSize` | `Group size` | range ` people` | 1–200 / 1 | — |
| `cargo` | `Cargo` | range ` kg` | 0–5000 / 25 | — |
| `pace` | `Pace` | seg `Easy · Steady · Forced` | | water stage |
| `hours` | `Hours per day` | range ` h` | 4–12 / 0.5 | water stage |
| `weather` | `Weather` | seg `auto · clear · rain · storm · snow` | | — |
| `carryFood` | `Carry food` | toggle | | — |
| `supplies` | `Supplies days` | range ` d` | 0–60 / 1 | — |
| `grazing` | `Grazing` | seg `none · sparse · normal · rich` | | water stage |
| `foraging` | `Foraging` | seg `none · modest · active` | | water stage |
| `road` | `Road quality` | seg `paved · tracks · trail · none` | | water stage |
| `infra` | `Infrastructure` | seg `none · fords · fords+ferries · full` | | water stage |
| `mount` | `Mount` | seg `horse · pony · camel · mule` | | water stage |
| `dWater` | `Desert water` | seg `auto · oasis chain · carry all` | | `biome !== 'desert'` (always true here) |
| `vessel` | `Vessel` | seg `river barge · coaster · raft` | | **land** stage |

Footer: `A blank field inherits the party form · an auto field shows its resolved value · a field that cannot apply to this stage is disabled with the reason.`

---

### 6.6 Sheet — MORE tab

A navigation stack (`moreStack`, starts `['root']`). The back button pops one level.

**Row-type styling**

| Type | Spec |
|---|---|
| `head` | `font:9px mono`, `letter-spacing:.18em`, `var(--faint)`, `padding:14px 4px 6px` |
| `nav` | `min-height:52px`, `padding:0 12px`, radius 16, `gap:12px`. Glyph `12px mono`, `width:18px`, centred, `var(--faint)` (or `var(--acc)` when the badge is `ARMED`). Label `12.5px`, `var(--ink)`. Sub `9.5px mono`, `var(--dim)`, ellipsised. Badge `9px mono` at its own colour. Chevron `›`, `var(--faint)` |
| `act` | `min-height:48px`, `margin:5px 0`, `padding:0 16px`, radius 24, centred, `font:500 10px mono`, `letter-spacing:.14em`. Primary: `background:var(--acc)`, `color:var(--accInk)`. Secondary: `background:var(--chip)`, `color:var(--sec)` |
| `tog` | `min-height:50px`, `padding:0 12px`, `gap:12px`. Label `12px`, `var(--body)`; sub `9.5px mono`, `var(--dim)`. Large toggle (44×24) right |
| `seg` | `padding:8px 4px 6px`. Label `10px mono`, `var(--sec)`, `padding-bottom:7px`. Chips `min-height:40px`, `padding:0 13px`, radius 20, `gap:6px`, `font:10px mono` |
| `range` | `padding:8px 4px 4px`. Label left `10px mono` `var(--sec)`; display right `11px mono` `var(--ink)`; full-width slider, no steppers |
| `read` | `min-height:42px`, `padding:0 12px`, `border-bottom:1px solid var(--hair2)`. Key `9.5px mono`, `letter-spacing:.1em`, `var(--dim)`; value `10.5px mono`, right-aligned, `var(--ink)` unless overridden |
| `info` | `font:9.5px/1.6 mono`, `var(--faint)`, `padding:8px 4px` |

**`_moreTitle()` — title / subtitle per screen**

| Screen id | Title | Subtitle |
|---|---|---|
| `root` | `MORE` | `program · data · preferences` |
| `project` | `PROJECT` | `files · autosave · storage` |
| `civ` | `CIVILIZATION` | `settlement · POI · way tools` |
| `data` | `DATA MANAGER` | `import · export · sources · conversion · validation` |
| `data-tiles` | `EXPORT ▸ MAPS ▸ TILE PYRAMID` | `leaflet · XYZ · baked atlas L0–L3` |
| `data-io` | `DATA ROUTE` | `{dir} ▸ {what}` — always `IMPORT ▸ Maps` |
| `assets` | `ASSET LIBRARY` | `24 families · 72 of 113 filled` |
| `assets-grid` | `ASSET LIBRARY` | `family · {assetFam}` |
| `asset-slot` | `SLOT INSPECTOR` | `{assetFam} · slot {n+1}` |
| `travel` | `TRAVEL LIBRARY` | `classifications & constraints — feeds the planner` |
| `travel-item` | `TRAVEL LIBRARY` | `{selected entry}` |
| `landmarks` | `LANDMARKS` | `caps · spacing · one run` |
| `lm-fam` | `{family label}` | `zero = off · a cap is a ceiling, not a quota` |
| `sim` | `SIMULATION` | `timeline · layers` |
| `prefs` | `PREFERENCES` | `application · performance · graphics` |
| `help` | `HELP & ABOUT` | `Cartalith Mobile 0.9` |
| `gestures` | `GESTURE REFERENCE` | `the whole touch vocabulary` |

#### `root`

| Type | Glyph | Label | Sub | Target |
|---|---|---|---|---|
| nav | `⧉` | `Project` | `save · recent · storage` | `project` |
| nav | `◍` | `Civilization` | `settlement · POI · way tools` | `civ` |
| nav | `⇅` | `Data manager` | `import · export · sources · validation` | `data` |
| nav | `▦` | `Asset library` | `24 families` — badge `72 / 113` in `var(--acc)` | `assets` |
| nav | `≋` | `Travel library` | `animals · vehicles · vessels · parties` | `travel` |
| nav | `◷` | `Simulation` | `year {n} · running\|paused` | `sim` |
| nav | `⚙` | `Preferences` | `theme · units · performance · graphics` | `prefs` |
| nav | `?` | `Help & about` | `0.9 · build 2611` | `help` |
| head | | `STATUS` | | |
| read | | `WORLD` | `{name} · {seed}` | |
| read | | `STATE` | `{world.status}` | |
| read | | `LAST AUTOSAVE` | `{savedAt} · every {autoInt}` | |
| read | | `UNDO DEPTH` | `{undoDepth}` | |

#### `project`

| Type | Content | Action |
|---|---|---|
| act (primary) | `SAVE PROJECT` | sets `savedAt = HH:MM`, toast `Project saved · {HH:MM}` |
| act | `SAVE AS…` | toast `Save as… — the new path becomes the project path (mock)` |
| act | `＋ NEW WORLD…` | opens the New World modal |
| act | `OPEN PROJECT .ZIP…` | toast `File picker — mock. Archives load from ~/Cartalith/Worlds.` |
| head | `RECENT WORLDS` | |
| nav `◍` | `VHAREN REACH` / `129384 · stages 01–07 · 5 d ago` | loads it (seed `129384`, status `stages 01–07 resolved`), resets `moreStack` to root, closes the sheet, toast `VHAREN REACH loaded` |
| nav `◍` | `KESSA` / `774201 · draft · 3 w ago` | same, seed `774201`, status `draft · stage 03` |
| head | `AUTOSAVE` | |
| tog | `Autosave` / `status bar reports the last write` | default **on** |
| seg | `Interval` — `off · 1 min · 5 min · 15 min` | default `5 min` |
| head | `STORAGE LOCATIONS` | |
| read | `PROJECTS` `~/Cartalith/Worlds` | |
| read | `TILE ATLAS` `~/Cartalith/Cache/atlas` | |
| read | `ASSET PACKS` `~/Cartalith/Packs` | |
| read | `EXPORTS` `~/Cartalith/Exports` | |
| info | `Imports live under Data ▸ Import · asset packs under Asset library. Moving the atlas root invalidates the cache.` | |

#### `civ`

| Type | Content | Action |
|---|---|---|
| info | `Arming a tool closes this sheet — tap the map to place. Esc or a new tool disarms.` | |
| nav `⌂` | `Settlement` / `tap drops a place · class town · snap to water` — badge `ARMED` in `var(--acc)` when active | arms tool `settlement`, `tab=null`, toast `Tool armed — tap the map to place. Undo chip appears after each drop.` |
| nav `◇` | `Point of interest` / `tap drops a POI · diamond marker` | arms `poi`, same toast |
| nav `∥` | `Way` / `taps append waypoints · commit from the floating card` | arms `way`, same toast |
| read | `TERRITORY PAINT` = `desktop parity — not in this prototype` in `var(--faint)` | |
| nav `◈` | `Landmark generation` / `49 types · 6 families · cap + spacing` — badge `{n} placed` in `var(--acc)` | → `landmarks` |
| act (primary) | `OPEN JOURNEY PLANNER ➔` | `tab='plan'`, `planView='root'` |
| head | `PLACED THIS SESSION` | |
| read | `SETTLEMENTS · POI` `{userPlaces.length}` | |
| read | `WAYS` `{ways.length}` | |
| read | `LABELS · ICONS` `{labels} · {icons}` | |
| act | `CLEAR PLACED` | clears labels/icons/ways/userPlaces, pushes undo `clear placed`, toast `Placed annotation cleared` |

On-map naming when armed: settlement → `New town {userPlaces.length+1}` (class `town`); POI → `POI {userPlaces.length+1}` (class `poi`). Both toast `{name} placed — staleness: none (annotation)` and push an undo entry `{tool} · {name}`.

#### `data`

| Section | Rows |
|---|---|
| `IMPORT` | nav `Maps` / `georeferenced image`; nav `Heightmaps` / `PNG · TIFF · 16-bit`; nav `GIS / GeoJSON` / `features → layers`; nav `World data` / `.zip · fields` — all glyph `▸`, all → `data-io` |
| `EXPORT` | nav `Maps · tile pyramid` / `leaflet · XYZ · retina` — badge `{zmax} levels` in `var(--acc)` → `data-tiles`; nav `GIS / GeoJSON` / `places · ways · territories` → `data-io`; nav `World data` / `full archive` → `data-io`; act `EXPORT ASSET PACK .ZIP` → toast `pack.json schema 2 + PNGs → STORED zip (mock)` |
| `SOURCES` | read `EXTERNAL SOURCES` `2 linked`; read `CONNECTED SOURCES` `Markdown vault · 412 notes`; read `SOURCE REGISTRY` `14 entries` |
| `CONVERSION` | nav `Coordinate systems` / `EPSG:3857 · 4326 · custom` → `data-io`; nav `Format conversion` / `tables · rasters` → `data-io`; act `IMPORT TABLE .CSV…` → toast `Import table .csv — column mapping opens on desktop (mock)` |
| `VALIDATION` | act (primary) `CHECK DATA` → toast `Check data — 0 errors · 8 warnings (6 missing icon slots, 2 unnamed rivers)`; act `REPAIR / NORMALIZE` → toast `Repair / normalize — 2 fixes applied, report in Source Registry` |

#### `data-tiles`

State defaults: `scheme:'XYZ'`, `size:'256'`, `retina:true`, `skip:true`, `zmax:5`.

| Row | Spec |
|---|---|
| seg `Scheme` | `XYZ · TMS · WMTS` |
| seg `Tile size` | `256 · 512` |
| range `Zoom levels 0 → N` | 1–8 / 1, display `0 – {zmax}` |
| tog `Retina @2x` | sub `doubles render cost` |
| tog `Skip all-ocean tiles` | sub `−34% of the pyramid` |
| head | `ESTIMATE` |
| read `TILES` | `{tiles}` + ` after ocean skip` when skipping |
| read `SIZE` | `~{sizeMB} MB` |
| read `RENDER TIME` | `~{mins} min · source: baked atlas L0–L3` |
| act (primary) | `EXPORT {tiles} TILES` → toast `Rendering tile pyramid — baked atlas L0–L3, no re-gen…`, then after **1800 ms** toast `Export complete · leaflet-preview.html + style.json emitted` |
| info | `Destination ~/Cartalith/Exports · emits leaflet-preview.html + style.json + attribution.` |

Estimator:
```
tilesBase = round(4^zmax × 1.37)
tiles     = skip ? round(tilesBase × 0.66) : tilesBase
sizeMB    = round(tiles × (size==='512' ? 0.11 : 0.059) × (retina ? 1.9 : 1))
mins      = max(1, round(tiles / 1650 × (retina ? 1.8 : 1)))
```
Defaults yield `tiles 926`, `size ~104 MB`, `time ~1 min`.

#### `data-io`

| Row | Content |
|---|---|
| info | `Route configuration is desktop-parity mock in this prototype. The route exists so nothing on the phone is unreachable.` |
| read | `CRS` `EPSG:3857` |
| read | `WORLD BOUNDS` `region select tool writes these fields` |
| act (primary) | `CHOOSE FILE…` → toast `File picker — mock. Archives load from ~/Cartalith/Worlds.` |

#### `assets`

Eleven nav rows (glyph `▦`, badge = filled / total), then `COLLECTIONS`:

| Family | Badge |
|---|---|
| `Places` | `10 / 12` |
| `Buildings` | `7 / 16` |
| `Walls & gates` | `4 / 8` |
| `Trees & cover` | `14 / 16` |
| `Rock & scree` | `6 / 10` |
| `Textures` | `8 / 12` |
| `Hachure & hatch` | `3 / 6` |
| `Compass & frame` | `5 / 6` |
| `Label plaques` | `4 / 8` |
| `Ship & sea marks` | `6 / 9` |
| `Map furniture` | `5 / 10` |

Then: head `COLLECTIONS`; read `UNASSIGNED IMPORTS` `3` in `var(--acc)`; info `Import image… and pack .zip live here on desktop; slots compile to the live pack with Apply.`

#### `assets-grid`

**No list rows** — this screen renders only the slot grid.
- Header row: `padding:10px 4px 8px`; left `{FAMILY} · SLOTS` (uppercased), `9.5px mono`, `.16em`, `var(--dim)`; right `tap a slot`, `9px mono`, `var(--faint)`.
- Grid: `grid-template-columns:repeat(4,1fr)`, `gap:8px`, **12 cells**.
- Cell: `aspect-ratio:1`, radius 14, column, centred, `gap:4px`, `border:1px solid {sel?--acc:--hair}`.
  - Filled background `var(--chip)`; empty background `repeating-linear-gradient(45deg,var(--chip),var(--chip) 6px,transparent 6px,transparent 12px)`.
  - Glyph `17px mono`, cycling `['◆','▲','●','▪','◈','✦']` by `i % 6`; blank when empty.
  - Code `8.5px mono`, `var(--faint)`: `{first letter of family, uppercased}-{(i+1) zero-padded to 2}`.
- Filled count: `Places → 10`, `Trees & cover → 11`, every other family → `7`.
- Tapping a cell selects it and pushes `asset-slot`.

#### `asset-slot`

Fixed placeholder content for every slot:
`FILE` `capital-star.png · 512×512 · 84 KB` · `SCALE` `118% · fit · reset` · `ANCHOR` `base` · `VARIANTS` `×3` · `TAGS` `capital · star · settlement` · act `REPLACE…` · act `+ VARIANT` · info `Preview backgrounds and batch actions are desktop-parity.`

#### `travel`

seg `Type` — `ANIMALS · VEHICLES · VESSELS · PARTIES` (default `ANIMALS`), then one nav row (glyph `≋`) per entry → `travel-item`, then info `An information layer only — entries become selectable options in the journey planner.`

| Tab | Entry | Sub |
|---|---|---|
| ANIMALS | `Horse` | `mount · pack 90 kg · 24 km/d base · grazing normal` |
| | `Pony` | `mount · pack 70 kg · sure-footed · montane ok` |
| | `Mule` | `pack 80 kg · fodder ceiling ~9 d · stubborn` |
| | `Camel` | `pack 140 kg · arid specialist · water every 5 d` |
| | `Ox` | `draft only · 2 per cart · slow` |
| VEHICLES | `Cart` | `2 draft · 400 kg · needs track or better` |
| | `Wagon` | `4 draft · 800 kg · needs road` |
| | `Travois` | `1 animal · 40 kg · any terrain` |
| | `Sled` | `snow only · 120 kg` |
| VESSELS | `River barge` | `hold 4 t · river & lake · crew 3` |
| | `Coaster` | `hold 12 t · coastal · crew 6 · sailing windows` |
| | `Raft` | `hold 0.8 t · downstream only` |
| PARTIES | `Merchant caravan` | `12 · steady · baggage train · 900 kg` |
| | `Royal courier` | `3 · forced · riders · 40 kg` |
| | `Pilgrim band` | `28 · easy · 300 kg · 14 d supplies` |

#### `travel-item`

`ENTRY` `{name}` · `CLASS` `party set-up` (parties) or `classification` · `CONSTRAINTS` `{sub text}` · `SOURCE` `Travel library · project data`.
Parties additionally get act (primary) `LOAD INTO PLANNER ➔`; other entries get info `Selectable as mount / vehicle / vessel in the planner party form.`

Party presets written into the planner:

| Preset | groupSize | pace | mode | cargo | supplies | horses | mules | carts |
|---|---|---|---|---|---|---|---|---|
| `Merchant caravan` | 12 | `Steady` | `baggage train` | 900 | 10 | 8 | 6 | 2 |
| `Royal courier` | 3 | `Forced` | `riders` | 40 | 5 | 6 | 0 | 0 |
| `Pilgrim band` | 28 | `Easy` | `baggage train` | 300 | 14 | 0 | 2 | 1 |

Loading also switches to the PLAN tab at root and toasts `{name} loaded into the planner`.

#### `sim`

| Row | Spec |
|---|---|
| tog `Transport strip on map` / `mini scrub above the nav bar` | turning on toasts `Transport strip on the map — close this sheet to scrub` |
| range `Year` | −400…1200 / 1, default 412, display `YEAR {n}` |
| seg `Speed` | `×1 · ×10 · ×100`, default `×1` |
| head | `SIMULATION LAYERS` |
| tog ×6 | `Climate` **on**, `Population` **on**, `Economy` off, `Politics` **on**, `Infrastructure` off, `Warfare` off |
| info | `Generation is not time-based — the timeline drives simulation layers only.` |

#### `landmarks`

`LMFAMS()` — 6 families, **49 types**. `cls` is one of `REG` (regional), `LOC` (local), `CON` (continental), `CUL` (cultural).

| Family | Label | Glyph | Types (`name` · `cls` · cap · base · candidates · flags) |
|---|---|---|---|
| `physical` | `PHYSICAL` | `▲` | `Peak` REG cap 12 base 14 cand 640 *no viewshed*; `Ridge` REG cap 0 (was 8); `Saddle` LOC cap 0 (was 5); `Cliff` LOC cap 0 (was 20); `Gorge` REG cap 8 base 6 cand 410 *fixed: no terrain*; `Cave` LOC cap 0 (was 12); `Waterfall` REG cap 40 base 11 cand 1284; `Spring` LOC cap 0 (was 30); `Lake` REG cap 20 base 12 cand 520; `Delta` REG cap 0 (was 3); `River confluence` LOC cap 30 base 24 cand 960; `Volcanic feature` CON cap 0 (was 2) *no viewshed*; `Rock formation` LOC cap 0 (was 16); `Glacial feature` REG cap 0 (was 6); `Ancient forest` REG cap 16 base 9 cand 300 *fixed: candidates* |
| `transportation` | `TRANSPORTATION` | `⌒` | `Mountain pass` REG cap 12 base 9 cand 88; `River crossing` LOC cap 0 (was 20); `Ford` LOC cap 20 base 8 cand 340 *no terrain*; `Bridge site` LOC cap 0 (was 12); `Road junction` LOC cap 0 (was 8); `Caravan station` LOC cap 0 (was 6); `Portage` LOC cap 0 (was 4); `Harbour` REG cap 8 base 4 cand 60 *candidates* |
| `economic` | `ECONOMIC` | `▦` | `Mine` LOC cap 12 base 6 cand 520; `Quarry` LOC cap 5 base 3 cand 280 *no terrain*; `Salt works` LOC cap 0 (was 4); `Resource extraction site` LOC cap 0 (was 8); `Market site` LOC cap 0 (was 6); `Trade depot` LOC cap 0 (was 5) |
| `military` | `MILITARY` | `▥` | `Fort` REG cap 12 base 10 cand 240 *no viewshed*; `Watchtower` LOC cap 30 base 15 cand 680 *no viewshed*; `Fortified pass` REG cap 5 base 8 cand 44; `Fortified crossing` LOC cap 0 (was 6); `Battlefield` CUL cap 0 (was 10); `Border marker` LOC cap 8 base 3 cand 90 *candidates, no viewshed* |
| `religious` | `RELIGIOUS · CULTURAL` | `∩` | `Shrine` CUL cap 50 base 18 cand 1420; `Temple` CUL cap 8 base 7 cand 120; `Sacred grove` CUL cap 12 base 8 cand 380 *no terrain*; `Sacred mountain` CON cap 3 base 5 cand 22 *no viewshed*; `Pilgrimage site` CUL cap 0 (was 6); `Tomb` CUL cap 8 base 5 cand 210 *candidates*; `Monument` CUL cap 0 (was 8); `Ceremonial site` CUL cap 0 (was 7) |
| `historical` | `HISTORICAL` | `‖` | `Ruin` CUL cap 20 base 5 cand 600; `Abandoned settlement` CUL cap 8 base 3 cand 150; `Ancient road` CUL cap 3 base 1 cand 40 *candidates*; `Historic battlefield` CUL cap 0 (was 6); `Destroyed fortress` CUL cap 0 (was 4); `Historic crossing` CUL cap 0 (was 5) |

Initial per-type state: `armed = cap > 0`; when disarmed the remembered cap is `was` (or 8).
Global defaults: `crowd = 1`, `compete = true`, `lastRun = '—'`, `edited = false`.

Placement model:
```
base  = t.base ?? max(1, round((t.was ?? 6) × 0.6))
room  = max(1, round(base / crowd^1.6 × (compete ? 1 : 1.35)))
fixed 'no terrain'  → placed = min(cap, base); reason 'at cap' | 'no terrain'
fixed 'candidates'  → placed = min(cap, base); reason 'at cap' | 'candidates'
otherwise           → placed = min(cap, room); reason 'at cap' | 'spacing'
```

Screen rows:

| Row | Content |
|---|---|
| info | `caps total {Σ armed caps} · room for about {round(210/crowd^1.6 × (compete?1:1.35))} at this spacing · last run placed {Σ placed}` |
| range `Crowding` | 0.25–2 / 0.05, display `× {crowd.toFixed(2)} · REG keeps {fmtKm(34 × crowd)}` |
| tog `Types compete with each other` | sub `off lets a shrine sit beside a waterfall · on keeps every landmark clear of every other one` |
| head | `FAMILIES` |
| nav ×6 | label = family label, sub `{armed} of {total} armed · {placed} placed`, glyph = family glyph, → `lm-fam` |
| act (primary) | `RUN LANDMARK PASS`, or `PLACING… {min(99, round(pct))}%` while running |
| info | `caps edited since the last run — results are stale until you run · ` (when edited) or `last run {HH:MM} · `, then `a cap is a ceiling, not a quota — the spacing calculation gives the restraint` |

Run mock: `setInterval` at **130 ms**, `pct += 9 + rand·14`; on completion recomputes results and marks, sets `lastRun = HH:MM`, `edited = false`, redraws the map, toast `Landmark pass — {total} placed · spacing gave the restraint`.

Marks are placed at `x = 200 + h(k,i+1)·3700`, `y = 200 + h(i+2,k)·3700` where `h(a,b) = frac(sin(a·127.1 + b·311.7) × 43758.5453)`.

#### `lm-fam`

Per type in the selected family:
- range, label `{name} · {cls}[ · no viewshed]`, **0–12 step 1**, value = index into the cap ladder `[0,1,2,3,5,8,12,20,30,50,80,120,200]` nearest the current cap (0 when disarmed). Display: `{cap} max` when armed, `off · was {cap}` when disarmed. Dragging to **0 disarms the type and remembers its number**.
- when armed, a `read` row: key `↳ LAST RUN`, value `{placed} placed · {reason}`, value colour `var(--acc)` when reason is `at cap`, else `var(--dim)`.

Then: act `ARM ALL` (arms every type in this family), act `ALL OFF`, and info `the slider is one gesture — zero disarms the type and remembers its number; drag up and it resumes.` plus, when the family contains a `no viewshed` type, `no viewshed = scores without the visibility term; the engine has no viewshed analysis yet.`

#### `prefs`

| Row | Spec / default |
|---|---|
| seg `Theme` | `dark · light` — default `dark` |
| seg `Units` | `km · mi` — default `km`; changing it re-renders the map |
| head | `PERFORMANCE` |
| tog `GPU acceleration` | sub `WebGPU · on` / `WebGPU · off — CPU tile passes`; default **on** |
| range `CPU worker threads` | 1–16 / 1, default 12, display `{n} of 16` |
| head | `GRAPHICS` |
| seg `Render quality` | `performance · balanced · quality · ultra` — default `balanced` |
| seg `Anti-aliasing` | `off · MSAA 2× · MSAA 4× · MSAA 8×` — default `MSAA 4×` |
| head | `TILES & LOD` |
| tog `Tiled LOD` | sub `auto on zoom`; default **on** |
| read `ATLAS CACHE` | `{cache} · 85 tiles`, default cache `1.2 GB` |
| act `CLEAR CACHES…` | sets cache to `0.0 GB`, toast `Atlas + field caches cleared — project data untouched` |
| head | `MEMORY` |
| range `Undo history` | 1–50 / 1, default 5, display `{n} steps` |
| head | `TOUCH` |
| nav `☰` `Gesture reference` | sub `the whole touch vocabulary` → `gestures` |

#### `help`

read `VERSION` `Cartalith Mobile 0.9 · build 2611` · read `ENGINE` `shared with desktop · WebGPU` · nav `☰` `Gesture reference` / `pan · pinch · rotate · sample` → `gestures` · act `CREDITS & ACADEMIC PRINCIPLES` (toast `Credits & academic principles — full text in the desktop build`) · act `REPORT AN ISSUE` (toast `Report an issue — opens the tracker (mock)`) · info `The phone reorganises rather than truncates: every desktop function is reachable through MAP · GENERATE · PLAN · MORE.`

#### `gestures`

Nine `read` rows, verbatim:

| Key | Value |
|---|---|
| `DRAG` | `pan the map` |
| `PINCH` | `zoom` |
| `TWO FINGERS` | `rotate` |
| `DOUBLE-TAP` | `zoom in` |
| `LONG-PRESS` | `sample terrain → pin + chip` |
| `EDGE-SWIPE ←` | `inspector panel` |
| `SHEET HANDLE` | `drag between peek · half · full` |
| `TAB RE-TAP` | `close the sheet` |
| `↶ CHIP` | `tap undo · hold history` |

---

### 6.7 Overlays

#### Search (`⌕`)
- Scrim: `inset:0`, `z:24`, `rgba(0,0,0,.35)` — tap closes.
- Panel: `left:{chromeLeft}px; right:0; top:0; z:25; padding:34px 10px 0`. Card radius 20, `background:var(--pan)`, `border:1px solid var(--bord)`, `box-shadow:0 16px 40px rgba(0,0,0,.5)`.
- Input row: `padding:10px 12px`, `border-bottom:1px solid var(--hair2)`, `gap:10px`. Glyph `⌕` `14px mono` `var(--dim)`; field transparent, `13px mono`, `var(--ink)`, placeholder `find places · labels · routes…`; `✕` `11px mono` `var(--sec)`.
- Result row: `min-height:48px`, `padding:0 16px`, `gap:12px`, `border-bottom:1px solid var(--hair2)`. Glyph `11px mono` `var(--acc)`; name `flex:1` `var(--body)`; kind right `9.5px mono` `var(--faint)`.
- Result set: the 8 `PLACES` (glyph `◍`, kind = class) + user places (`◍`) + labels (glyph `⌖`, kind `label`) + one fixed route row `Vhal Serai → Port Amre` at `(1035, 2270)`, glyph `➔`, kind `route`. Case-insensitive substring filter, **max 8 rows**.
- Picking: pans to `(x, y)`, sets scale to `max(current, 1.1)`, drops a sample pin, closes, toast `Panned to {name}`.

#### Inspector (right drawer)
- Scrim `inset:0`, `z:26`, `rgba(0,0,0,.3)` — tap closes.
- Panel: `top:0; bottom:0; right:0; width:{min(340, round(frameW()×0.82))}px; z:27`, `background:var(--pan)`, `border-left:1px solid var(--bord)`, `box-shadow:-14px 0 34px rgba(0,0,0,.4)`, `animation:pIn .24s ease`.
- Header (`padding:16px 16px 8px`): `SAMPLE · INSPECTOR` (`font:500 10.5px mono`, `.2em`, `var(--acc)`); coords `9.5px mono` `var(--dim)` = `{x} km E · {y} km N` (thousands-separated); close `38×38` radius 19 `var(--chip)`.
- Elevation block (`padding:6px 16px 14px`, `border-bottom:1px solid var(--hair2)`): value `font:500 30px mono` `var(--ink)` + ` m elevation` `12px mono` `var(--dim)`.
- Rows: `min-height:40px`, `padding:0 16px`, `border-bottom:1px solid var(--hair2)`. Key `9.5px mono`, `.14em`, `var(--dim)`; value `11px mono`, `var(--ink)`, right.

| Key | Value |
|---|---|
| `SLOPE · ASPECT` | `{slope}° · {aspect}°` |
| `PLATE` | plate id |
| `LITHOLOGY` | lithology |
| `TEMPERATURE` | `{temp} °C` |
| `PRECIPITATION` | `{rain} mm/yr` |
| `DRAINAGE` | `Kess basin` or `—` |
| `BIOME` | biome |
| `SOIL` | `loam · 1.2 m` or `—` |
| `NEAREST SETTLEMENT` | `{name} · {distance}` |

- Footer: `swipe in from the right map edge to reopen · fields from stale stages read —` (`padding:14px 16px`, `9.5px/1.6 mono`, `var(--faint)`).
- Opened by: the right edge-swipe, or a tap on the sample chip. Samples the pin if one exists, otherwise the view centre.

#### New World modal
- Scrim `inset:0`, `z:30`, `rgba(0,0,0,.45)`, `display:flex; align-items:center; justify-content:center; padding:22px`.
- Card: `width:100%; max-width:360px`, radius 22, `background:var(--pan)`, `border:1px solid var(--bord)`, `box-shadow:0 20px 50px rgba(0,0,0,.55)`, `padding:18px 16px 16px`.
- Title `NEW WORLD` — `font:500 11px mono`, `.2em`, `var(--acc)`, `padding-bottom:12px`.
- Field labels `9.5px mono`, `var(--dim)`: `NAME` (`padding-bottom:5px`), `SEED` (`padding:12px 0 5px`), `EXTENT` (`padding:12px 0 6px`).
- Text inputs: `background:var(--chip)`, `border:1px solid var(--hair)`, radius 12, `padding:11px 12px`, `12px mono`, `var(--ink)`. Name placeholder `world name…`.
- Dice button: `44 × 42`, radius 12, `background:var(--chip)`, `border:1px solid var(--hair)`, `var(--body)`, glyph `⚄`.
- Extent chips: two, `flex:1`, `min-height:42px`, radius 14, `font:10.5px mono`, `.1em` — `REGION` (default) and `WORLD`.
- Buttons (`padding-top:16px`, `gap:10px`): `CANCEL` `flex:1`, `min-height:46px`, radius 23, `background:var(--chip)`, `var(--sec)`, `font:500 10.5px mono`, `.14em`; `CREATE WORLD` `flex:1.4`, `background:var(--acc)`, `var(--accInk)`.
- Create: name uppercased (`UNTITLED` if blank), status `empty · generate to begin`, clears all annotation and the sample pin, sets `gen.stale = 1` and `lastRun = '—'`, sets the view to `{cx:2048, cy:2048, s:0.18, r:0}`, toast `World created — open GENERATE to run stages 01–10`.

---

## 7. Gestures

All map gestures are bound directly on the map host (`touch-action:none`), not on the canvas.

| Gesture | Trigger | Effect |
|---|---|---|
| **Pan** | one pointer, total displacement from origin > **10 px** | `cx -= (dx·cos(−r) − dy·sin(−r))/s`; `cy -= (dx·sin(−r) + dy·cos(−r))/s`. Counts toward the coach step-0 counter |
| **Pinch zoom** | two pointers | `s = clamp(s₀ × d / max(20, d₀), 0.07, 14)`, anchored so the world point under the two-finger midpoint stays put |
| **Rotate** | the same two pointers | `r = r₀ + (angle − angle₀)`, unclamped, no snapping, applied simultaneously with the pinch |
| **Double tap** | second tap within **300 ms** and **34 px** of the first | animated zoom **×1.7** over **240 ms**, easing `1−(1−k)³`, anchored at the tap point |
| **Single tap** | `duration < 280 ms` **and** movement `< 10 px`, resolved after a **240 ms** delay (to let a double tap win) | dispatches to the armed tool — see §6.3 / §6.6 `civ` |
| **Long press** | one pointer held **480 ms** without moving | samples the terrain, drops the pin, shows the chip, clears the pointer set, completes coach step 1 |
| **Right edge-swipe** | pointer-down within **28 px** of the map's right edge, then dragged **≥ 60 px** left | opens the Inspector |
| **Wheel** (mouse/trackpad) | `wheel` with `preventDefault` | zoom factor `1.0016^(−deltaY)` at the cursor |
| **Sheet drag** | pointer on the handle/header block (portrait only) | live height, snap to nearest detent, close below 44 px — §5.3 |
| **Undo chip hold** | **520 ms** | opens the history popover; a short tap undoes one step |
| **Tab re-tap** | tap the active tab | closes the sheet |
| **Escape key** | `keydown` | closes, in order: modal → search → overflow menu → inspector → sheet |

Any pointer-down on the map also closes the overflow menu.

---

## 8. Safe areas, cutout, landscape

### 8.1 What the prototype actually draws

| Region | Portrait | Landscape |
|---|---|---|
| Status band | **30 px** at the top of the chrome layer (mock text only), `padding:0 18px`, starting at `x = chromeLeft` | same, starting at `x = 72` |
| Bottom gesture inset | **18 px** inside the 84 px nav bar, with a **112 × 4** pill in `var(--bord)` | **none** |
| Nav | 84 px bottom bar | 72 px left rail |
| Rail top pad | — | `padding-top:40px` (the only landscape allowance for the status area) |
| Map inset | `bottom:84px` | `left:72px` |
| Chrome origin | `chromeLeft = 0` | `chromeLeft = 72` |

### 8.2 Landscape rail

`position:absolute; left:0; top:0; bottom:0; width:72px; z-index:14`, `background:var(--pan2)`, `border-right:1px solid var(--hair2)`, column, `align-items:center`, `gap:6px`, `padding-top:40px`.
Cell: `60 × 56`, `border-radius:16px`, column, centred, `gap:3px`, background `var(--wash)` when active. Glyph `14px mono`; label `8.5px mono`, `letter-spacing:.08em`. Same four tabs, same colours.

### 8.3 Landscape behaviour differences

| Behaviour | Portrait | Landscape |
|---|---|---|
| Sheet | bottom sheet, three detents, draggable | right drawer, `width = min(440, round(frameW×0.46))`, **drag disabled** (the pointerdown handler returns immediately), only the close ✕ and back arrow work |
| Sheet corner radius | `22px 22px 0 0` | `0`, with a `1px solid var(--hair)` left border |
| FAB column bottom | 104 | 18 |
| Undo chip left | 12 | 86 |
| Dock (label / way / sim) bottom | 98 | 14 |
| Coach bottom | 100 | 16 |
| Toast bottom | 162 | 78 |
| Home-indicator pill | present | absent |

Orientation is toggled only by the host device strip; there is no in-app orientation control and no defined behaviour for rotating while a sheet or overlay is open.

---

## 9. `UNSPECIFIED:` — what a builder needs and cannot find

**Platform integration**

1. `UNSPECIFIED:` **Real safe-area / display-cutout insets.** The 30 px status band and 18 px gesture inset are hard-coded decorations; no `env(safe-area-inset-*)` equivalent appears anywhere. Landscape has no bottom inset at all and only a `padding-top:40px` on the rail. The builder needs the actual inset policy for status bar, navigation bar, and camera cutout in both orientations.
2. `UNSPECIFIED:` **Android system-back behaviour.** Only `Escape` is bound (modal → search → menu → inspector → sheet). Whether system back follows that chain, and what it does at the root of the app screen and on the picker, is not stated.
3. `UNSPECIFIED:` **On-screen keyboard avoidance.** The LABEL bar sits at `bottom:98px` and would be covered by an IME; the search field and New World modal have no stated keyboard behaviour.
4. `UNSPECIFIED:` **Haptics.** No feedback is defined for long-press sample, detent snap, tool arm, or verdict change.
5. `UNSPECIFIED:` **Accessibility.** No focus states, no content descriptions, no dynamic-type policy, no contrast statement. Several targets fall below the 48 dp Android minimum: stage-override segmented chips at **36 px**, GENERATE/PLAN segment chips and sculpt presets at **38 px**, stepper buttons at **38 × 38**, icon-variant cells at **48 × 44**.
6. `UNSPECIFIED:` **Rotation while a sheet or overlay is open.** `_snapSheet()` re-applies the portrait height, but the landscape drawer has no equivalent and there is no stated transition.
7. `UNSPECIFIED:` **Tablet treatment.** The `TABLET 800` frame (800 × 1280) runs the identical phone layout — same 84 px bar, same 4-tab row, same full-width sheet. No two-pane or wider variant is given.

**Design gaps inside the prototype**

8. `UNSPECIFIED:` **Light-theme semantic colours.** `--warn`, `--block`, `--good` and `--water` are not redefined in the light palette; they keep their dark values against a light surface.
9. `UNSPECIFIED:` **Map rendering in light theme.** `_styleP()` reads `stylePreset` only; the canvas stays dark in light mode unless the user separately picks `PARCHMENT`. Whether the theme should drive the map palette is not stated.
10. `UNSPECIFIED:` **What the 9 colour ramps do.** Selecting a ramp only sets `styleCustom = true`; nothing in the draw path reads `ramp`. Their effect on terrain rendering is undefined.
11. `UNSPECIFIED:` **Bottom-bar active-pill height.** Given only as `padding:4px 16px` around a 14 px glyph — no explicit height, so it is content-derived.
12. `UNSPECIFIED:` **The 3D viewport.** The FAB flips a `2D`/`3D` label and toasts `relief exaggeration in Preferences ▸ Graphics`, but Preferences ▸ Graphics contains no relief-exaggeration control and no 3D mode is drawn.
13. `UNSPECIFIED:` **Undo depth enforcement.** Preferences offers `Undo history` (1–50, default 5), but `undoStack` is never trimmed. Whether the cap drops oldest entries, blocks new ones, or is advisory is undefined.
14. `UNSPECIFIED:` **Simulation layer effects.** The six layer toggles (`Climate`, `Population`, `Economy`, `Politics`, `Infrastructure`, `Warfare`) change no rendering and no readout.
15. `UNSPECIFIED:` **Stale-field rendering.** The stale note promises `Fields owned by stale stages read — until re-run`, but no field in the prototype ever renders `—` for that reason.
16. `UNSPECIFIED:` **The 8 Data-manager routes.** All eight `data-io` nav rows land on a single screen; `this.state.dataRoute` is declared in the title lookup but never assigned, so the title is always `IMPORT ▸ Maps`. The per-route screens do not exist.
17. `UNSPECIFIED:` **13 of the 24 asset families.** The header claims `24 families · 72 of 113 filled`; the badges do sum to 72/113, but only **11** families are listed.
18. `UNSPECIFIED:` **Asset slot grids.** Every family shows exactly **12** cells regardless of its declared total (`Buildings` is `7 / 16` but gets 12 slots), and fill counts are `Places → 10`, `Trees & cover → 11` (its badge says 14), everything else → 7.
19. `UNSPECIFIED:` **Asset badge colour rule.** Computed as `badge[0] === badge.slice(-2,-1)` — a character comparison with no semantic meaning, which makes only `Places` and `Trees & cover` render faint. The intended rule (e.g. "complete = faint") is not stated.
20. `UNSPECIFIED:` **Per-slot asset data.** `asset-slot` shows the same fixed placeholder (`capital-star.png · 512×512 · 84 KB`, `118% · fit · reset`, anchor `base`, `×3` variants) for every slot in every family.
21. `UNSPECIFIED:` **Empty and error states.** No empty world list, no "no search results" row, no generation-failure state, no offline or storage-full state anywhere.
22. `UNSPECIFIED:` **World units vs. working resolution.** The canvas draws a fixed 0–4096 square and every distance is formatted in km (so 1 unit = 1 km), while the world cards say `2 048²` / `1 024²` / `512²` and GENERATE offers `Working resolution 512…4096`. The relationship between resolution and physical extent is not defined.
23. `UNSPECIFIED:` **Landmark viewshed.** Eight types are flagged `no viewshed` with the note `the engine has no viewshed analysis yet` — the intended scoring once it exists is not given.
24. `UNSPECIFIED:` **Which persistence beyond coach marks.** The only persisted value is `localStorage['cartalith.coach']`. Whether device/theme/units/preferences persist is not stated.

**Defects in the source that a builder must resolve before porting**

25. **`restEvery` parse bug.** `String(rest).replace(/\D/g,'')` turns `'1 in 5'` → `15`, `'1 in 7'` → `17`, `'1 in 10'` → `110`. The label promises a rest day every 5 / 7 / 10 days; the code computes one every 15 / 17 / 110. `UNSPECIFIED:` which is intended (almost certainly 5 / 7 / 10).
26. **Blocked-verdict text hardcodes `High Saddle`** regardless of which stage actually blocked. With the fixed six-stage route only stage 2 can block, so it happens to be correct here, but the general rule is unstated.
27. **`Desert water` can never be enabled.** Its `na` test is `stage.biome !== 'desert'`, and no stage in `STAGES` has biome `desert`, so the row is permanently `— · not a desert stage`. Its enabled appearance is never rendered.
28. **`COPY TO ALL LAND STAGES` replaces rather than merges** — it overwrites each land stage's entire override map with a copy of the current stage's, discarding any existing per-stage overrides. Not stated as intentional.
29. **`--chipOn` is declared in both palettes and never referenced.**
30. **`landmark-glyphs.js` is not present** in the imported folder, so `window.LM_GLYPHS` is undefined and landmark marks never draw in the prototype. (Per the folder's own `README.md`, this is not a real gap: the same 49 glyph paths are already committed to `cartalith-native/godot-project/shell/dcc_icons.gd`, and the design was built on top of them.)
31. **All Preferences values are inert.** `gpu`, `workers`, `quality`, `aa`, `lod`, `autosave`, `autoInt`, `undoDepth` are stored and displayed but drive nothing.

# Cartalith DCC Environment — Frame Geometry & Token System

**Source:** `C:\Users\Vincent\Cartalith_GDT\design\dcc-environment-2026-08-31\Cartalith DCC Environment.dc.html`
Markup: lines 1–1162. Logic class `Component extends DCLogic`: lines 1163–2105.

---

## 0. BLOCKING DEFECT — the desktop prototype file is truncated

**The file is exactly 262 144 bytes (256 KiB) and ends mid-identifier.** Its last bytes are:

```
      measRows.push({i:('0'+i).slice(-2),len:this.fmtKm(km),be
```

There is no closing `}` for `valsCore()`, no closing `}` for the class, and no `</script></body></html>`. The companion `Cartalith Android.dc.html` in the same folder (166 424 bytes) **is** complete — it ends `...};\n  }\n}\n</script>\n</body>\n</html>`. So this is a per-file import truncation at a 2^18 boundary, not an authoring omission.

The committed blob is truncated too (`git log` → `fd2b6fd Import the new DCC Environment and Android prototypes`; working tree clean for that path). **Re-import this file from the design project before any builder starts.**

### What the truncation costs

`valsCore()`'s entire `return{…}` object is gone. A placeholder-vs-definition diff (481 `{{ }}` bindings in the markup against every `name:` in the script) finds **158 unresolved bindings**. Many are locals computed *before* the cut and merely not yet mapped into the return (`menus`, `frames`, `stages`, `globalTools`, `domainTools`, `railNodes`, `statusMsg`, `keyHints`, `doms`) — a builder can recover those. The rest have **no definition anywhere in the file**:

| Binding | Region it belongs to | What is unknown |
|---|---|---|
| `fw`, `fh`, `scale`, `mb` | artboard wrapper | frame px → style plumbing (recoverable, §1) |
| `fvars` | token root | how `themeStr` and `densStr` are concatenated (§2.4) |
| `scrimBg` | viewport HUD | the translucent plate behind the three HUD chips, light + dark |
| `railFoot` | domain rail footer | the vertical footer string |
| `ldCollapsedLabel`, `rdCollapsedLabel` | collapsed dock strips | the vertical strip labels |
| `ldTitle`, `rdTitle` | dock headers | per-context titles |
| `tlShow`, `tlCollapsed`, `tlExpanded`, `tlPct`, `tlYearLabel`, `tlState`, `tlSpeeds`, `tlToggles`, `tlPlayGlyph` | timeline | the whole timeline binding set |
| `worldLabel`, `themeLabel`, `footLabel` | menu bar / chrome | header strings |
| `statusMid`, `statusKeys` | status bar | the two right-hand status fields |
| `mapCursor`, `vpContext`, `vpField`, `layersBtnBg`, `layersBtnCol` | viewport | cursor + HUD |
| `undoCol`, `redoCol` | menu bar | enabled/disabled colours for `↶` `↷` |
| `ldSwitch`, `ldSwA`, `ldSwB`, `ldSwABg`, `ldSwACol`, `ldSwBBg`, `ldSwBCol` | left dock | the A/B mode switcher labels and states |
| `tbLabel`, `tbInspect`, `tbMeasure`, `tbRegion`, `tbPipe` | tool-options bar | 5 of the 14 content branches |
| `scrPicker`, `scrShell`, `pickerWorlds` | screens | screen gate + world cards |

Everything below is what the surviving 2 105 lines **do** specify, stated exactly.

---

## 1. The four frames

`frameDef()` — line 1816:

```
{w1920:{w:1920,h:1080}, w1366:{w:1366,h:768}, tabL:{w:2560,h:1600}, tabP:{w:1600,h:2560}}
```

`isTouch()` — line 1817: `return this.state.frame==='tabL'||this.state.frame==='tabP'`

Labels — line 2062, in switcher order:

| `id` | Label (exact) | Width | Height | Touch? | Density source |
|---|---|---|---|---|---|
| `w1920` | `WINDOWS 1920` | 1920 px | 1080 px | no | base only (no override) |
| `w1366` | `LAPTOP 1366` | 1366 px | 768 px | no | base + 3-token laptop override |
| `tabL` | `TABLET 2560` | 2560 px | 1600 px | **yes** | base + full 17-token touch override |
| `tabP` | `TABLET PORTRAIT` | 1600 px | 2560 px | **yes** | base + full 17-token touch override |

Default frame: `w1920` (state, line 1197). Frame is prototype-only chrome — a Godot build picks one branch by real window size.

### 1.1 Artboard wrapper (prototype chrome, line 24)

```
flex:none; width:{{fw}}px; height:{{fh}}px;
transform:scale({{scale}}); transform-origin:top center;
margin-bottom:{{mb}}px;
border:1px solid rgba(255,255,255,.16);
position:relative; overflow:hidden; background:#0d0e0f
```

`scale` is set in `componentDidMount` (line 1827): `Math.min(1,(window.innerWidth-36)/d.w)`, recomputed on `resize`. `fw`/`fh` are `frameDef().w/.h`.

`UNSPECIFIED:` `mb` has no definition. It compensates the `transform:scale` (which does not shrink layout box height), so it is presumably `-(1-scale)*fh` plus the 14 px column gap — but the file does not say. Irrelevant to a Godot build; note only if reproducing the canvas page.

The shell itself is the child at line 25, `position:absolute;inset:0` — so **the shell occupies exactly `fw` × `fh`**, unaffected by the wrapper's 1 px border.

### 1.2 Frame-conditional behaviour outside the token strings

| Where | Rule |
|---|---|
| `openRd()` line 1938 | `rdOpen:true`; **if `frame==='tabP'` and `ldOpen`, force `ldOpen:false`.** The two docks are mutually exclusive in portrait. |
| `Window ▸ Reset layout` line 2052 | `{ldOpen:true, rdOpen: frame!=='tabP', showRail:true, showSB:true, railExp:false, menu:null, sub:null}` — portrait resets with the right dock **closed**. |
| Status-bar key hints line 2101 | touch → `long-press = sample · pinch zooms`; non-touch → `V M R ` + (`B F` \| `L I` \| `S P T W`) + ` · ⌘Z · Esc` |
| `vals2()` line 1746 | declares `const touch=this.isTouch()` and **never uses it**. No sculpt/paint dock differs by frame. |

**`UNSPECIFIED:`** no other frame branch exists. In particular there is **no** touch override for: the 200 px rail-expansion column, the 238 px layers popover, the 168 px cross-section strip, the 12 px slider thumb, the 30×17 px toggle, or the 4 px slider track. All are literal px at every frame.

---

## 2. The token system

### 2.1 Base declaration (line 25, inline on the shell root)

Declared in this order, then `{{ fvars }}` appended last so overrides win.

**Colour tokens — dark (21):**

| Token | Value | Used for |
|---|---|---|
| `--sur` | `#0d0e0f` | shell background (`background:var(--sur)`, line 26); **also the OFF track of every toggle switch** (6 uses) |
| `--pan` | `#121314` | panel fill: left dock, right dock, collapsed dock strips, rail-expansion column, every popover, section strip, picker card |
| `--ins` | `#191c1e` | inset fill: slider tracks, chip/segment backgrounds, unarmed tool buttons, `↶ ↷ ◐` buttons, toast fill, text-input fill, dock-close hover |
| `--ink` | `#e8ebec` | strongest text; **every slider thumb and toggle knob** |
| `--body` | `#c8cbcd` | default shell text colour (line 26); menu item labels; toast text |
| `--sec` | `#a9adb0` | secondary text; unarmed tool glyphs; scale-bar rule |
| `--dim` | `#8d9296` | tertiary text; HUD chips; segment options when off |
| `--faint` | `#6f7478` | section headers (`.2em` tracked caps), field labels, chevrons, hints |
| `--dis` | `#5f6468` | disabled/least: the Pan tool, `Δ vertical · 3D only`, axis ticks, footnotes |
| `--acc` | `#e0a34a` | accent: wordmark, active domain, armed state, slider fill, primary-button fill, status message, section trace |
| `--accH` | `#f0bd72` | **declared, never referenced in this file** (0 uses). Page-level `a:hover` uses the literal `#f0bd72`. |
| `--accInk` | `#141005` | text/icon on `--acc` fill (the 11 primary buttons) |
| `--hair` | `rgba(255,255,255,.10)` | **structural** 1 px separators: every region border, dock borders, rail border |
| `--div` | `rgba(255,255,255,.07)` | **in-panel** 1 px separators: list-row rules, menu separators, vertical pips |
| `--bor` | `rgba(255,255,255,.16)` | 1 px outline of anything that floats: popovers, toasts, picker card |
| `--wash` | `rgba(224,163,74,.09)` | active-domain cell fill; menu/list hover |
| `--wash2` | `rgba(224,163,74,.16)` | armed-chip / armed-tool fill; selected segment option |
| `--shadow` | `0 14px 34px rgba(0,0,0,.55)` | every floating surface (5 uses) |
| `--good` | `#6fae7d` | **declared, never referenced** (0 uses) |
| `--block` | `#c96a5a` | blocked/danger (10 uses, incl. `col:r.danger?'var(--block)'` in menu rows) |
| `--water` | `#6a9bc4` | one use: journey-stage dot when the leg is water |

**Metric tokens (17):**

| Token | Base | Used for |
|---|---|---|
| `--fs` | `11.5px` | shell root `font-size` (line 26); the two `<input>` elements |
| `--m0` | `10.5px` | **always** `font:500 var(--m0) 'IBM Plex Mono'` — emphasised numeric readouts (6 uses) |
| `--m1` | `10px` | the standard mono size — 146 uses |
| `--m2` | `9px` | the small mono size — 142 uses (section caps, captions, status bar) |
| `--menuH` | `36px` | menu bar height (1 use) |
| `--tbH` | `40px` | horizontal tool rail **and** tool-options bar (2 uses) |
| `--railW` | `40px` | domain rail width; **and both collapsed dock strips** (3 uses) |
| `--ctl` | `24px` | universal control height: slider hit-rows, chips (`min-height`), `↶ ↷ ◐`, dock close buttons, inputs — 96 uses |
| `--btnH` | `28px` | button `min-height` — 20 uses |
| `--row` | `28px` | list-row `min-height` — 22 uses |
| `--tool` | `30px` | tool button box, layers button, rail chevron row, tool separator height — 7 uses |
| `--ldW` | `372px` | left dock width (1 use) |
| `--rdW` | `304px` | right dock width (1 use) |
| `--sbH` | `26px` | status bar height; toast bottom offset `calc(var(--sbH) + 14px)`; collapsed timeline `calc(var(--sbH) - 2px)` |
| `--pad` | `14px` | horizontal padding of every band and dock — 48 uses |
| `--g` | `10px` | **declared, never referenced** (0 uses) |
| `--pop` | `300px` | menu popover width (1 use) |

### 2.2 Full per-frame metric table

`densStr` — line 2064. Touch (`tabL`/`tabP`) replaces **all 17**; `w1366` replaces **3**; `w1920` replaces none.

| Token | `w1920` | `w1366` | `tabL` | `tabP` |
|---|---|---|---|---|
| `--fs` | 11.5px | 11.5px | **14px** | **14px** |
| `--m0` | 10.5px | 10.5px | **12.5px** | **12.5px** |
| `--m1` | 10px | 10px | **12px** | **12px** |
| `--m2` | 9px | 9px | **11px** | **11px** |
| `--menuH` | 36px | 36px | **52px** | **52px** |
| `--tbH` | 40px | 40px | **56px** | **56px** |
| `--railW` | 40px | 40px | **48px** | **48px** |
| `--ctl` | 24px | 24px | **36px** | **36px** |
| `--btnH` | 28px | 28px | **44px** | **44px** |
| `--row` | 28px | 28px | **44px** | **44px** |
| `--tool` | 30px | 30px | **44px** | **44px** |
| `--ldW` | 372px | **330px** | **400px** | **400px** |
| `--rdW` | 304px | **280px** | **400px** | **400px** |
| `--sbH` | 26px | 26px | **36px** | **36px** |
| `--pad` | 14px | 14px | **16px** | **16px** |
| `--g` | 10px | 10px | **12px** | **12px** |
| `--pop` | 300px | **280px** | **380px** | **380px** |

Colour tokens are **identical across all four frames** — `themeStr` and `densStr` sets are disjoint.

### 2.3 Light theme — complete override

`themeStr` — line 2063, applied when `state.light === true`. Overrides **all 21** colour tokens; **no metric token changes**.

| Token | Dark | Light |
|---|---|---|
| `--sur` | `#0d0e0f` | `#f4f2ee` |
| `--pan` | `#121314` | `#fbfaf7` |
| `--ins` | `#191c1e` | `#eceae4` |
| `--ink` | `#e8ebec` | `#111210` |
| `--body` | `#c8cbcd` | `#23241f` |
| `--sec` | `#a9adb0` | `#3d3f39` |
| `--dim` | `#8d9296` | `#6b6f6a` |
| `--faint` | `#6f7478` | `#8d9088` |
| `--dis` | `#5f6468` | `#9a9d95` |
| `--acc` | `#e0a34a` | `#a4650f` |
| `--accH` | `#f0bd72` | `#8a5309` |
| `--accInk` | `#141005` | `#f7f4ee` |
| `--hair` | `rgba(255,255,255,.10)` | `rgba(0,0,0,.14)` |
| `--div` | `rgba(255,255,255,.07)` | `rgba(0,0,0,.08)` |
| `--bor` | `rgba(255,255,255,.16)` | `rgba(0,0,0,.20)` |
| `--wash` | `rgba(224,163,74,.09)` | `rgba(164,101,15,.09)` |
| `--wash2` | `rgba(224,163,74,.16)` | `rgba(164,101,15,.16)` |
| `--shadow` | `0 14px 34px rgba(0,0,0,.55)` | `0 14px 34px rgba(35,36,31,.16)` |
| `--good` | `#6fae7d` | `#2c7a44` |
| `--block` | `#c96a5a` | `#a03d2e` |
| `--water` | `#6a9bc4` | `#2e6a9e` |

Note the accent **inverts direction**: dark `--accH` (`#f0bd72`) is *lighter* than `--acc`; light `--accH` (`#8a5309`) is *darker* than `--acc`. A hover state must not assume "hover = lighter".

**Non-tokenised colours that also switch on `light`** (`const L=s.light`, lines 1409–1410):

| Binding | Dark | Light | Used by |
|---|---|---|---|
| `secGridCol` | `rgba(255,255,255,.06)` | `rgba(0,0,0,.07)` | cross-section gridlines |
| `secFillCol` | `rgba(224,163,74,.13)` | `rgba(164,101,15,.12)` | cross-section area fill |
| `secLineCol` | `#e0a34a` | `#a4650f` | cross-section trace |
| `drawExtra6` stroke (line 1266) | `#a9adb0` | `#3d3f39` | landmark glyphs on the map canvas |

`UNSPECIFIED:` `scrimBg` — the plate behind the three viewport HUD chips (`vpContext`, `equirect · zoom …`, coords) — has no definition in either theme. It is required for HUD legibility over the map canvas.

**Theme is set two ways:**
1. `Preferences ▸ APPLICATION ▸ theme` segment `[dark | light]` → action `theme=dark` / `theme=light` → `setState({light: …})` (line 2043).
2. The `◐` button at the far right of the menu bar (`hTheme`, line 105) — **`UNSPECIFIED:` handler is in the truncated block**, but the same `hTheme` also drives the prototype chrome's theme pill (line 20), so it is a plain toggle.

### 2.4 How the strings compose — `UNSPECIFIED`

The root's `style` ends `…--pop:300px;{{ fvars }}`. `fvars` is not defined. Both `themeStr` (line 2063) and `densStr` (line 2064) are built as local consts and neither is otherwise consumed, so `fvars` must be their concatenation.

**Order is provably immaterial:** `themeStr`'s 21 keys and `densStr`'s 17 keys do not intersect. A builder can apply base → theme → density (or the reverse) with identical results. State this rather than guess a syntax.

---

## 3. The six regions

`box-sizing` is **`content-box`** — `support.js` carries no `*{box-sizing:border-box}` reset (it appears only inside `.sc-placeholder`/`.sc-interp` helper CSS, and once as the `.bbox` atomic class). **Each region's 1 px border therefore adds to its token height.** Two `<input>` elements set `box-sizing:border-box` explicitly (lines 1024, 1041); nothing else does.

The shell (line 26) is:
```
position:absolute; inset:0;
background:var(--sur); color:var(--body);
font-size:var(--fs); line-height:1.45;
display:flex; flex-direction:column
```

Two screens gate on `{{ scrPicker }}` / `{{ scrShell }}` (`state.scr`, default `'picker'`). Regions below are the `scrShell` branch.

### 3.1 DOM order

| # | Region | Line | Condition | Height | Border | Fill |
|---|---|---|---|---|---|---|
| 1 | Menu bar | 56 | always | `var(--menuH)`, `flex:none` | `border-bottom:1px solid var(--hair)` | none (`--sur` shows through) |
| 2 | Horizontal tool rail | 109 | `{{ railShow }}` | `var(--tbH)`, `flex:none` | `border-bottom:1px solid var(--hair)` | none |
| 3 | Tool-options bar | 149 | always | `var(--tbH)`, `flex:none` | `border-bottom:1px solid var(--hair)` | none |
| 4 | Middle band | 281 | always | **`flex:1; min-height:0`** | none | none |
| 5 | Timeline | 1108 | `{{ tlShow }}` | collapsed `calc(var(--sbH) - 2px)`; expanded auto | `border-top:1px solid var(--hair)` | none |
| 6 | Status bar | 1140 | `{{ showSB }}` | `var(--sbH)`, `flex:none` | `border-top:1px solid var(--hair)` | none |
| — | Toast layer | 1150 | always | absolute overlay, `pointer-events:none` | — | — |

Region 4 is the **only** flexible child. Regions 1, 3, 6 are always present; 2 and 5 are conditional. Composition is **flex column**, not grid.

### 3.2 Z-index stack

| z | Element | Line |
|---|---|---|
| 90 | toast layer | 1150 |
| 80 | menu popover | 62 |
| 70 | menu bar | 56 |
| 60 | layers popover | 825 |
| 51 | horizontal tool rail | 109 |
| 50 | tool-options bar | 149 |
| 40 | dock-internal popovers (2) | 543, 640 |
| 15 | cross-section strip | 843 |

Regions 2 and 3 carry `position:relative` so their `z-index` applies; the rail (51) sits above the tool-options bar (50) so its popovers overlap downward.

### 3.3 Region 1 — Menu bar (line 56)

```
height:var(--menuH); flex:none; display:flex; align-items:center;
gap:2px; padding:0 10px;
border-bottom:1px solid var(--hair); position:relative; z-index:70
```
Note: horizontal padding is a literal **`10px`**, *not* `var(--pad)` — the only band that does this.

Children, in order:

| Child | Geometry |
|---|---|
| Wordmark `CARTALITH` | `font:var(--m1) 'IBM Plex Mono'; letter-spacing:.18em; color:var(--acc); margin:0 12px 0 4px` |
| 7 menu buttons (`sc-for {{ menus }}`) | `padding:5px 11px; border-radius:8px`; on → `color:var(--ink)`, `background:var(--ins)`; off → `color:var(--sec)`, `background:transparent`. Ids/labels line 2065: `file`/`File`, `edit`/`Edit`, `assets`/`Assets`, `data`/`Data`, `prefs`/`Preferences`, `window`/`Window`, `help`/`Help` |
| separator | `width:1px; height:16px; background:var(--div); margin:0 6px` |
| `↶` undo | `var(--ctl)` square, `border-radius:8px`, `background:var(--ins)`, `display:grid;place-items:center` |
| `↷` redo | same |
| spacer | `flex:1` |
| `{{ worldLabel }}` | `font:var(--m1) 'IBM Plex Mono'; color:var(--dim)` |
| `◐` theme | `var(--ctl)` square, `background:var(--ins)`, `color:var(--sec)`, `margin-left:8px` |

Menu popover (line 62): `position:absolute; top:calc(100% + 4px); left:0; width:var(--pop); max-height:72vh; overflow-y:auto; background:var(--pan); border:1px solid var(--bor); box-shadow:var(--shadow); padding:5px 0`.
**`72vh` is measured against the real browser viewport, not the artboard** — a Godot build must clamp to the shell height instead.

Menu row types and their metrics:

| Type | Metrics |
|---|---|
| head | `padding:9px 14px 4px; font:var(--m2) mono; letter-spacing:.2em; color:var(--faint)` |
| sep | `height:1px; background:var(--div); margin:5px 0` |
| item | `min-height:var(--row); gap:8px; padding:3px 14px; padding-left:{{r.ind}}px` (default `14`); glyph column `width:14px`, `color:var(--dim)`, `font:var(--m1) mono`; label `flex:1`; shortcut `font:var(--m1) mono; color:var(--faint)`; hover `background:var(--wash)` |
| toggle | `min-height:var(--row); padding:3px 14px`; label `padding-left:22px`; switch `30×17px` radius `999px`, knob `13×13px` radius `50%` `background:var(--ink)` at `top:2px`, `left:2px` off / `left:15px` on; track `var(--ins)` off, `var(--acc)` on |
| seg | `padding:5px 14px 5px 36px`; label `flex:1; color:var(--sec)`; group `background:var(--ins); border-radius:999px; padding:2px`; option `padding:3px 10px; border-radius:999px; font:var(--m1) mono`; selected `color:var(--acc); background:var(--wash2)`, else `color:var(--dim); background:transparent` |
| read | `padding:4px 14px 4px 36px`; key `flex:1; font:var(--m2) mono; letter-spacing:.08em; color:var(--faint)`; value `font:var(--m1) mono; color:var(--sec); text-align:right` |
| note | `padding:6px 14px; font:var(--m2)/1.6 mono; color:var(--faint); border-top:1px solid var(--div); margin-top:4px` |

Item colour: `r.danger ? var(--block) : r.dim ? var(--dis) : var(--body)`.

### 3.4 Region 2 — Horizontal tool rail (line 109)

```
height:var(--tbH); flex:none; display:flex; align-items:center;
gap:10px; padding:0 var(--pad);
border-bottom:1px solid var(--hair); position:relative; z-index:51
```
Shown when `railShow` (line 1377): `s.scr==='app' && (isBrush || isMeas)` where `isBrush = tool ∈ {sculpt, freehand, biome}` and `isMeas = tool==='measure'`.

Leftmost is always the 3-group switch (`SCULPT` / `PAINT` / `MEASURE`): `min-height:var(--ctl); padding:2px 13px; border-radius:8px; gap:6px; font:var(--m1) mono; letter-spacing:.1em`; on → `background:var(--acc)`, `color:var(--accInk)`; off → `background:var(--ins)`, `color:var(--sec)`. Then `width:1px;height:20px;background:var(--div)`, then one of two branches (`railMeasure` / `railBrush`), each ending with `flex:1` spacer + a `var(--m2)` `var(--dis)` note.

The gap is a literal `10px`, not `var(--g)`.

### 3.5 Region 3 — Tool-options bar (line 149)

Identical box to region 2 but `z-index:50`. Begins with `{{ tbLabel }}`: `font:var(--m1) mono; letter-spacing:.16em; color:var(--acc); flex:none`. Then **14 mutually-exclusive `sc-if` branches** plus `<!--ANCHOR_TB-->`:

| Branch | Condition | Source |
|---|---|---|
| `tbInspect` | — | **`UNSPECIFIED:` truncated** |
| `tbMeasure` | — | **`UNSPECIFIED:` truncated** |
| `tbRegion` | — | **`UNSPECIFIED:` truncated** |
| `tbPipe` | — | **`UNSPECIFIED:` truncated** |
| `tbSculpt` | `tool==='sculpt' \|\| tool==='freehand'` | line 1784 |
| `tbBiome` | `tool==='biome'` | line 1784 |
| `tbCarto` | `domain==='CARTO' && tool==='inspect'` | line 1583 |
| `tbLabelRow` | `tool==='label'` | line 1604 |
| `tbIconRow` | `tool==='icon'` | line 1604 |
| `tbSettle` | `tool==='settlement'` | line 1632 |
| `tbPoi` | `tool==='poi'` | line 1632 |
| `tbTerr` | `tool==='territory'` | line 1632 |
| `tbWay` | `tool==='way' \|\| tool==='route'` | line 1507 |
| `tbPlan` | `domain==='CIVIL' && cc()==='planner' && tool==='inspect'` | line 1507 |

### 3.6 Region 4 — Middle band (line 281)

```
flex:1; display:flex; min-height:0
```
Four to six flex children left→right:

**(a) Domain rail** — `{{ showRail }}`, default `true`, line 283:
```
width:var(--railW); flex:none;
border-right:1px solid var(--hair);
display:flex; flex-direction:column; align-items:stretch
```
- expand chevron row: `height:var(--tool); display:grid; place-items:center; color:var(--faint)`
- 3 domain cells (`WORLD`, `CIVIL`, `CARTO` — line 2066): `flex:1; max-height:112px; display:grid; place-items:center`; label `writing-mode:vertical-rl; transform:rotate(180deg); font:var(--m1) mono; letter-spacing:.26em`. Active → `color:var(--acc)`, `background:var(--wash)`, `box-shadow:inset -2px 0 var(--acc)`; inactive → `color:var(--dim)`, `background:transparent`, `box-shadow:none`.
- `flex:1` spacer
- footer: `padding:10px 0`; `{{ railFoot }}` vertical, `font:var(--m2) mono; letter-spacing:.2em; color:var(--faint)` — **`UNSPECIFIED:` string missing**

> **Stale markup hint:** line 285 still reads `hint-placeholder-count="5"` although `doms` (line 2066) is three. Cosmetic — placeholder-render only — but it is the fossil of the five-domain rail the README says was replaced.

**(b) Rail-expansion column** — `{{ railExp }}`, default `false`, line 294:
```
width:200px; flex:none; border-right:1px solid var(--hair);
background:var(--pan); overflow-y:auto; padding:8px 0
```
**200 px is a literal, with no touch override.** Rows: header `padding:10px 14px 3px; font:var(--m2) mono; letter-spacing:.2em; color:var(--faint)`; node `min-height:var(--row); padding:2px 14px`, hover `background:var(--wash)`.

Node tree (line 2068) — 3 headers + 10 nodes:

| Header | Nodes (`label`, `dom`, `mode`) |
|---|---|
| `WORLD` | `Generation pipeline` (WORLD, `a`) · `Sculpt` (WORLD, `b`) |
| `CIVIL` | `Landmarks` (CIVIL, `landmarks`) · `Factions & settlements` (CIVIL, `factions`) · `Ways & routes` (CIVIL, `infra`) · `Journey planner` (CIVIL, `planner`) |
| `CARTO` | `Layers & style` · `Labels` · `Icons` · `Terrain appearance` (all CARTO, no mode) |

Node colour: `var(--acc)` when `n.dom===s.domain` and (no mode, or mode matches `worldMode`/`cc()`); else `var(--body)`. Background always `transparent`. `setDomain()` (line 2054) forces `railExp:false`.

**(c) Left dock** — `{{ ldOpen }}`, default `true`, line 304:
```
width:var(--ldW); flex:none; border-right:1px solid var(--hair);
background:var(--pan); display:flex; flex-direction:column; min-height:0
```
| Sub-region | Geometry |
|---|---|
| header | `flex:none; gap:8px; padding:8px var(--pad) 0`; title `font:var(--m2) mono; letter-spacing:.2em; color:var(--faint); flex:1`; close `‹` `var(--ctl)` square, radius 8px, `color:var(--faint)`, hover `background:var(--ins)` |
| A/B switch (`{{ ldSwitch }}`) | `flex:none; padding:8px var(--pad) 2px`; pill `background:var(--ins); border-radius:999px; padding:3px`; halves `flex:1; text-align:center; padding:5px 0; border-radius:999px; font:var(--m2) mono; letter-spacing:.12em` |
| TOOLS block | `flex:none; padding:10px var(--pad); border-bottom:1px solid var(--div)`; label `TOOLS` `font:var(--m2) mono; letter-spacing:.2em; color:var(--faint); margin-bottom:7px`; grid `display:flex; gap:5px; flex-wrap:wrap` |
| body | `flex:1; overflow-y:auto; min-height:0` |

Global tools (line 2072): 4 icon squares `var(--tool)` × `var(--tool)`, radius 8px — `inspect` (tip `Inspect · V`), `measure` (`Measure · M`), `region` (`Region select · R`), `pan` (`Pan / zoom — always available`, permanently `background:var(--ins)`, `color:var(--dis)`). Then `width:1px; height:var(--tool); background:var(--div); margin:0 3px`. Then domain tools (line 2073) as pills `min-height:var(--tool); padding:2px 12px; border-radius:8px; gap:6px; font:var(--m1) mono`, key suffix `color:var(--faint)`:

| Domain | `[id, label, key]` |
|---|---|
| WORLD | `sculpt`/`Sculpt`/`` · `freehand`/`Freehand`/`F` · `biome`/`Biome paint`/`B` |
| CIVIL | `settlement`/`Settlement`/`S` · `poi`/`POI`/`P` · `territory`/`Territory`/`T` · `way`/`Way`/`W` · `route`/`Route`/`⇧R` |
| CARTO | `label`/`Label`/`L` · `icon`/`Icon`/`I` |

Armed tool → `background:var(--wash2)`, `color:var(--acc)`; else `background:var(--ins)`, `color:var(--sec)`.

**(d) Collapsed left strip** — `{{ ldClosed }}`, line 813:
```
width:var(--railW); flex:none; border-right:1px solid var(--hair);
background:var(--pan); display:flex; flex-direction:column;
align-items:center; gap:12px; padding:8px 0; cursor:pointer
```
`›` in `var(--faint)`, then vertical `{{ ldCollapsedLabel }}` `font:var(--m2) mono; letter-spacing:.2em; color:var(--dim)`.

**(e) Viewport** — line 819, always present:
```
flex:1; min-width:0; position:relative; overflow:hidden
```
Contents:
- `<canvas>` `position:absolute; inset:0; width:100%; height:100%; touch-action:none; cursor:{{mapCursor}}`
- top-left cluster `top:10px; left:10px; gap:8px` — layers button `var(--tool)` square radius 8px `border:1px solid var(--hair)`; then `{{ vpContext }}` chip `font:var(--m2) mono; letter-spacing:.14em; color:var(--dim); background:{{scrimBg}}; padding:4px 9px; border-radius:6px`
- layers popover: `top:calc(100% + 6px); left:0; width:238px; background:var(--pan); border:1px solid var(--bor); box-shadow:var(--shadow); padding:6px 0` — **238 px literal, no touch override**
- top-right chip `top:12px; right:12px` — `equirect · zoom 100% · {{ vpField }}`
- scale bar `bottom:{{hudBottom}}; left:12px; gap:3px` — label default text `250 km` (`font:var(--m2) mono; color:var(--dim)`), rule `width:120px; height:1px; background:var(--sec)` with two `1×5px` end ticks
- coords `bottom:{{hudBottom}}; right:12px`, default text `— · —`
- `hudBottom` (line 1402) = `'180px'` when the cross-section strip is up, else `'12px'`
- cross-section strip `{{ secStrip }}` (line 843): `position:absolute; left:0; right:0; bottom:0; height:168px; background:var(--pan); border-top:1px solid var(--hair); z-index:15`. Header row `padding:6px var(--pad) 2px`; title `SECTION A → B` (`letter-spacing:.16em; color:var(--acc)`), fixed caption `120 samples · ×4 exaggeration` (`color:var(--faint)`). Body `padding:2px var(--pad) 4px; gap:10px`; axis gutter `width:52px; text-align:right; color:var(--dis); padding:2px 0 14px`; plot `border-left:1px solid var(--div)`; `<svg viewBox="0 0 1000 130" preserveAspectRatio="none"` at `height:calc(100% - 14px)`, gridlines at `y=43` and `y=86`, trace `stroke-width:1.6`; axis strip `height:14px`, `A · 0` / `{{secHalf}}` / `B · {{secLen}}`.
- `<!--ANCHOR_VP-->`

**(f) Right dock** — `{{ rdOpen }}`, default `true`, line 870:
```
width:var(--rdW); flex:none; border-left:1px solid var(--hair);
background:var(--pan); display:flex; flex-direction:column; min-height:0
```
Header is **mirrored**: close `›` first (`var(--ctl)` square), then title `flex:1; text-align:right`, `padding:8px var(--pad) 6px`. Body `flex:1; overflow-y:auto; min-height:0; padding:0 var(--pad) 14px` (the left dock body has **no** padding — each panel pads itself).

**(g) Collapsed right strip** — `{{ rdClosed }}`, line 1101: same as (d) but `border-left`, `‹`, `letter-spacing:.18em`, **`color:var(--acc)`** (the left strip uses `var(--dim)`).

### 3.7 Region 5 — Timeline (line 1108)

Collapsed (line 1110): `flex:none; height:calc(var(--sbH) - 2px); gap:12px; padding:0 var(--pad); border-top:1px solid var(--hair); cursor:pointer`. Strings: `TIMELINE` (`font:var(--m2) mono; letter-spacing:.16em; color:var(--faint)`), `{{ tlYearLabel }}` (`var(--m1)`, `color:var(--sec)`), `flex:1` spacer, `▴ expand` (`var(--m2)`, `color:var(--faint)`).

Expanded (line 1118): `flex:none; display:flex; flex-direction:column; gap:6px; padding:8px var(--pad); border-top:1px solid var(--hair)` — **auto height**.
- transport row `gap:10px`: play glyph, `◀`, `▶` — each `var(--ctl)` square radius 8px `background:var(--ins)` (play `color:var(--acc)`, steps `color:var(--sec)`); speed pill group `background:var(--ins); border-radius:999px; padding:2px`, options `padding:2px 10px`; `{{ tlState }}` `var(--m1)`, `color:var(--acc)`; `flex:1`; 6 layer toggles `padding:3px 10px; border-radius:999px; font:var(--m2) mono`; collapse `⌄` `color:var(--faint); padding:0 4px`
- scrub `height:16px`: track `flex:1; height:3px; background:var(--ins); border-radius:2px`; playhead `width:2px; height:13px; top:-5px; background:var(--acc)`
- footer row `justify-content:space-between; font:var(--m2) mono; color:var(--dis)`: `YEAR −400` · `{{ tlYearLabel }}` (`color:var(--sec)`) · `YEAR 1200`

State defaults (line 1204): `tlOpen:false`, `tlYear:412`, `tlRun:false`, `tlSpeed:'×10'`, `tlTog:{Climate:true, Population:true, Economy:false, Politics:true, Infrastructure:false, Warfare:false}`.
`Window ▸ Timeline (CIVIL · INFRA)` toggles `tlOpen`.
`UNSPECIFIED:` `tlShow`, `tlCollapsed`, `tlExpanded` are not defined — whether the timeline is additionally gated on domain (the menu label implies it) and how collapsed/expanded is stored (there is no `tlExpandedState` key) cannot be recovered.

### 3.8 Region 6 — Status bar (line 1140)

```
height:var(--sbH); flex:none; display:flex; align-items:center;
gap:18px; padding:0 var(--pad);
border-top:1px solid var(--hair);
font:var(--m2) 'IBM Plex Mono',monospace
```
Three fields: `{{ statusMsg }}` (`color:var(--acc); letter-spacing:.06em`), `flex:1` spacer, `{{ statusMid }}` (`color:var(--dis)`), `{{ statusKeys }}` (`color:var(--faint)`).

`statusMsg` is built at lines 2094–2100 in strict priority order — exact strings:

| Order | Condition | Message |
|---|---|---|
| 1 | `s.run` | `running NN <stage name> — P%` (NN zero-padded, `&` unescaped) |
| 2 | `this.statusExtra()` truthy | that value |
| 3 | `s.staleFrom != null` | `stage NN edited — D downstream stages stale` where `D = 10 - staleFrom` |
| 4 | `tool==='measure' && measure.pts.length` | `measuring — N points` |
| 5 | `s.finalized` | `finalized — stages locked · atlas L0–L3 baked` |
| 6 | else | `stages 01–10 resolved · <world.name lowercased> · seed <world.seed>` |

`keyHints` (line 2101), touch: `long-press = sample · pinch zooms`; non-touch: `V M R ` + `B F` (WORLD) / `L I` (CARTO) / `S P T W` (CIVIL) + ` · ⌘Z · Esc`.
`UNSPECIFIED:` `statusKeys` is not mapped in the return; `keyHints` is almost certainly it, but `statusMid` has no candidate at all.

### 3.9 Toast layer (line 1150)

```
position:absolute; left:0; right:0; bottom:calc(var(--sbH) + 14px);
display:flex; flex-direction:column; align-items:center; gap:6px;
pointer-events:none; z-index:90
```
Toast: `background:var(--ins); border:1px solid var(--bor); color:var(--body); padding:7px 16px; border-radius:999px; font:var(--m1) 'IBM Plex Mono'; box-shadow:var(--shadow); animation:tIn .18s ease`.
`@keyframes tIn` (line 10): `from{opacity:0; transform:translateY(6px)} to{opacity:1; transform:none}`.
`toast()` (line 1819): lifetime **2600 ms**; queue `[...s.toasts.slice(-2), new]` → **at most 3 stacked**.

### 3.10 Derived viewport dimensions

Widths (subtracting each 1 px separator):

| Frame | Both docks open | Both collapsed | `railExp` open + docks open |
|---|---|---|---|
| `w1920` | 1920−41−373−305 = **1201** | 1920−41−41−41 = **1797** | **1000** |
| `w1366` | 1366−41−331−281 = **713** | 1366−41−41−41 = **1243** | **512** |
| `tabL` | 2560−49−401−401 = **1709** | 2560−49−49−49 = **2413** | **1508** |
| `tabP` | 1600−49−401−401 = **749** | 1600−49−49−49 = **1453** | **548** |

`tabP` with exactly one dock open (the enforced state after `openRd()`): 1600−49−401−49 = **1101**.

Heights:

| Frame | Bands consumed | Default viewport | + rail (region 2) | + timeline collapsed | + both |
|---|---|---|---|---|---|
| `w1920` (1080) | 37 + 41 + 27 = 105 | **975** | 934 | 950 | 909 |
| `w1366` (768) | 105 | **663** | 622 | 638 | 597 |
| `tabL` (1600) | 53 + 57 + 37 = 147 | **1453** | 1396 | 1418 | 1361 |
| `tabP` (2560) | 147 | **2413** | 2356 | 2378 | 2321 |

`UNSPECIFIED:` the expanded timeline's height. Its `flex:none` column is content-driven (`8px + var(--ctl) + 6 + 16 + 6 + one --m2 line + 8`, ≈ 82 px at desktop, ≈ 104 px at touch) but no fixed value is authored.

---

## 4. Typography

**Two stacks only.**

| Stack | Where declared | Loaded? |
|---|---|---|
| `'Helvetica Neue', Helvetica, Arial, sans-serif` | page root, line 11 — inherited by the whole shell | **system stack, not webfont-loaded** |
| `'IBM Plex Mono', monospace` | every explicit `font:` shorthand | webfont, line 10 |

Webfont link (line 10, verbatim):
`https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&display=swap`
**Only weights 400 and 500 are loaded.** No mono weight above 500 is used, so nothing synthesises.

Base text (line 26): `font-size:var(--fs); line-height:1.45; color:var(--body)` on the sans stack. This is what menu labels, dock body copy, node labels, tool pill labels and button text inherit.

### Roles

| Role | Declaration | Size @desktop / @touch | Tracking |
|---|---|---|---|
| Shell body text | inherited sans | 11.5 / 14 px, `line-height:1.45` | — |
| Emphasised value | `font:500 var(--m0) mono` (6×) | 10.5 / 12.5 px | — |
| Standard mono (values, chips, shortcuts, tool labels) | `font:var(--m1) mono` (146×) | 10 / 12 px | usually none |
| Small mono (section caps, captions, status, hints) | `font:var(--m2) mono` (142×) | 9 / 11 px | usually `.2em` |
| Prose in mono | `font:var(--m2)/1.6 mono` (17×), `/1.7` (7×), `/1.5` (5×), `/1.55` (1×) | 9 / 11 px | — |
| Mono at `--m1` with leading | `font:var(--m1)/1.8 mono` (3×), `/1.7` (2×) | 10 / 12 px | — |
| Emphasised standard mono | `font:500 var(--m1) mono` (2×) | 10 / 12 px | — |
| Primary button label | inherited sans + `font-weight:600` (11×) | 11.5 / 14 px | — |
| Stage name | inherited sans, `font-weight:{{st.nameW}}` = `400` (resolved) or `600` | 11.5 / 14 px | — |
| Text inputs | `font:var(--fs) 'Helvetica Neue',sans-serif` (line 222, 1024) and `font:500 var(--fs) …` (line 1041) | 11.5 / 14 px | — |

### Fixed-px type (does **not** scale with frame density)

| Declaration | Uses | Where |
|---|---|---|
| `font:500 26px 'IBM Plex Mono'` | 3 | large numeric readout — e.g. right-dock `{{ sampleElev }}` (line 880), `color:var(--acc)` |
| `font:500 22px 'IBM Plex Mono'` | 1 | large readout |
| `font:500 20px 'IBM Plex Mono'; letter-spacing:.34em` | 4 | picker wordmark `CARTALITH` (line 31), `color:var(--ink)` |
| `font:500 13px 'IBM Plex Mono'; letter-spacing:.12em` | 1 | picker card world name (line 41), `color:var(--ink)` |
| `font:500 12px 'IBM Plex Mono'; letter-spacing:.24em` | 1 | prototype chrome header `CARTALITH · DCC ENVIRONMENT` (line 13) |
| `font:10.5px 'IBM Plex Mono'` | 1 | prototype chrome subtitle (line 14) |
| `font:10px 'IBM Plex Mono'` | 3 | frame chips + theme pill (17–20), footer (1160) |
| `font:8.5px 'IBM Plex Mono'` | 2 | smallest in-shell caption |

**`UNSPECIFIED:`** the 26 px / 22 px readouts have no touch equivalent. On `tabL`/`tabP` the surrounding mono grows 9→11 px while these stay 26 px, so the ratio changes from ~2.9× to ~2.4×. Either that is intended or it is an oversight; the file does not say.

### Letter-spacing inventory

| Value | Uses | Role |
|---|---|---|
| `.2em` | 29 | **the section-header standard** — every `var(--m2)` caps label (menu heads, TOOLS, dock titles, rail heads, collapsed-left strip) |
| `.14em` | 8 | viewport context chip, `CROSS-SECTION` label, picker state chip |
| `.18em` | 6 | menu-bar wordmark; collapsed-**right** strip |
| `.16em` | 5 | tool-options bar `{{tbLabel}}`; `SECTION A → B`; `TIMELINE` |
| `.12em` | 5 | left-dock A/B switch halves; picker card name; frame chips |
| `.1em` | 2 | rail group buttons (line 112); sample-row keys (line 884) |
| `.34em` | 1 | picker wordmark |
| `.26em` | 1 | domain-rail vertical labels |
| `.24em` | 1 | prototype chrome header |
| `.08em` | 1 | menu `read` row key |
| `.06em` | 1 | status-bar message |

Only one `line-height` is declared anywhere: `1.45` on the shell root. All other leading comes from the `font:` shorthand's `/N` form.

---

## 5. Radii, borders, shadows

### Radii

| Value | Uses | Role |
|---|---|---|
| `999px` | 79 | **pill** — chips, segment groups and their options, toggle tracks, toasts, picker state chip, timeline speed/layer pills |
| `8px` | 76 | **square control** — buttons, tool squares, `↶ ↷ ◐`, dock close, layers button, primary buttons, text inputs, rail group buttons |
| `2px` | 43 | **slider track and its fill** (`height:4px`), timeline scrub track (`height:3px`) |
| `50%` | 28 | **circles** — slider thumbs (12 px), toggle knobs (13 px), 7 px stage dots |
| `3px` | 7 | tiny swatches |
| `6px` | 5 | **viewport HUD chips** (`padding:4px 9px`) |
| `4px` | 4 | menu-row key caps (`width:14px`, `border:1px solid var(--div)`); hue slider track (`height:8px`) |
| `1px` | 2 | hairline swatch |
| `16px` | 2 | prototype chrome frame chips / theme pill (`padding:8px 14px`) |
| `14px` | 1 | — |
| `5px` | 1 | — |

**Popovers, panels, docks and the section strip have radius 0.**

### Borders — every declaration in the file

| Declaration | Uses | Role |
|---|---|---|
| `border-bottom:1px solid var(--div)` | 31 | list-row rules inside docks |
| `border:1px solid var(--bor)` | 9 | every floating surface: menu popover, layers popover, 2 dock popovers, toast, picker card border-on-hover partner |
| `border-top:1px solid var(--div)` | 6 | in-panel summary rules |
| `border-top:1px solid var(--hair)` | 4 | **status bar**, **timeline collapsed**, **timeline expanded**, section strip |
| `border-right:1px solid var(--hair)` | 4 | **domain rail**, **rail-expansion column**, **left dock**, **collapsed left strip** |
| `border-bottom:1px solid var(--hair)` | 3 | **menu bar**, **horizontal tool rail**, **tool-options bar** |
| `border-left:1px solid var(--hair)` | 2 | **right dock**, **collapsed right strip** |
| `border:1px solid var(--hair)` | 2 | layers button; picker card |
| `border:1px dashed var(--bor)` | 3 | drop/empty zones |
| `border:1px solid rgba(255,255,255,.16)` | 2 | prototype chrome only (artboard wrapper, theme pill) |
| `border:1px solid var(--div)` | 1 | menu-row key cap |
| `border-left:1px solid var(--div)` | 1 | cross-section plot gutter |
| `border:2px solid {{ m.bord }}` | 1 | selected map marker |
| `border:1px solid {{ st.dotBord }}` | 1 | 7 px stage dot |
| `border:1px solid {{ planVerdictBord }}` / `{{ finBord }}` / `{{ f.bord }}` | 3 | state-driven |
| `border:none` | 4 | text inputs (3) + one reset |

**The rule:** `--hair` = structural chrome edges; `--div` = rules inside a panel; `--bor` = the outline of anything that floats. All are 1 px solid; the only 2 px border is a selected map marker.

Vertical separator pips are `<span>`s, not borders: `width:1px; height:16px` (menu bar, `margin:0 6px`), `height:20px` (tool bars), `height:var(--tool)` (`margin:0 3px`, tool grid) — all `background:var(--div)`.

### Shadows

Exactly **one** shadow token, used 5×: `--shadow` = `0 14px 34px rgba(0,0,0,.55)` dark / `0 14px 34px rgba(35,36,31,.16)` light. Applied to: menu popover, layers popover, the two dock popovers, toast.

The one other `box-shadow` is not a shadow: `box-shadow:{{ d.inset }}` on the active domain cell = `inset -2px 0 var(--acc)` (a 2 px inner edge marker on the cell's right side), else `'none'`.

### Motion

| Declaration | Uses | Where |
|---|---|---|
| `transition:transform .15s` | **1** | the pipeline stage chevron only (line 338) |
| `animation:tIn .18s ease` | 1 | toast entry |

All other chevrons (`{{f.chev}}`, `{{g.chev}}`, `{{lmCatChev}}`, `{{facCatChev}}`, `{{infCatChev}}`, `{{plnCatChev}}`, `{{lmAdvChev}}`) rotate `0deg`↔`90deg` with **no transition**. `UNSPECIFIED:` whether that asymmetry is intentional.

### Reusable control geometry (literal at every frame)

| Control | Spec |
|---|---|
| Slider | hit row `height:var(--ctl)`, `cursor:pointer`, `touch-action:none`; track `flex:1; height:4px; border-radius:2px; background:var(--ins)`; fill `position:absolute; left:0; top:0; bottom:0; border-radius:2px; background:var(--acc); width:{pct}`; thumb `position:absolute; top:-4px; margin-left:-6px; width:12px; height:12px; border-radius:50%; background:var(--ink); left:{pct}`. Percentages are strings like `"63%"`. |
| Toggle | track `width:30px; height:17px; border-radius:999px; flex:none`; knob `position:absolute; top:2px; width:13px; height:13px; border-radius:50%; background:var(--ink)`; `left:2px` off, `left:15px` on. Track `var(--acc)` on; OFF track is **`var(--ins)`** in menu rows but **`var(--sur)`** in dock rows (lines 2081, 1459, and the 4 named toggles). |
| Segment group | wrapper `display:flex; background:var(--ins); border-radius:999px; padding:2px` (menu) or `padding:3px` (dock A/B); option `padding:3px 10px; border-radius:999px; font:var(--m1) mono`; selected `color:var(--acc); background:var(--wash2)`; unselected `color:var(--dim); background:transparent` |
| Chip | `min-height:var(--ctl); padding:2px 10–13px; border-radius:999px; font:var(--m1) mono`; on `background:var(--wash2); color:var(--acc)`; off `background:var(--ins); color:var(--sec)` or `var(--dim)` |
| Primary button | `min-height:var(--btnH); border-radius:8px; background:var(--acc); color:var(--accInk); font-weight:600` (inherited sans); padding varies `0 14px` / `4px 14px` / `4px 15px` / `6px 18px`, or `flex:1` + `justify-content:center` |
| Secondary button | `min-height:var(--btnH); border-radius:8px; background:var(--ins); color:var(--sec)`, no weight override |
| Icon button | `width:var(--ctl); height:var(--ctl); border-radius:8px; background:var(--ins); display:grid; place-items:center` |
| Tool button | `width:var(--tool); height:var(--tool); border-radius:8px; display:grid; place-items:center` |
| Hover styles (all 5 in the file) | `background:var(--wash)` (4 — menu items, node rows); `background:var(--ins)` (2 — dock close buttons); `color:var(--acc)` (1); `border-color:var(--acc)` (1 — picker card); `color:var(--acc);border-color:var(--acc)` (1) |

**Slider thumbs stay 12 px and toggles stay 30×17 px at touch density**, while `--ctl` grows 24→36 px. The hit area scales; the visual affordance does not. Compare the 2026-08-30 Android brief in `design/android-2026-08-30/`, whose TARGETS note reads *"Slider thumbs grow to 19 px"* — the desktop file has no equivalent.

---

## 6. Consolidated `UNSPECIFIED` list

**Caused by the 256 KiB truncation** (fix by re-import, not by invention):

1. `fvars` composition — inferable as `themeStr + densStr`; provably order-independent (disjoint key sets), but not authored.
2. `scrimBg` — the viewport HUD plate, both themes. Needed for legibility over the canvas.
3. `railFoot`, `ldCollapsedLabel`, `rdCollapsedLabel`, `ldTitle`, `rdTitle`, `worldLabel`, `themeLabel`, `footLabel` — all chrome strings.
4. `statusMid`, `statusKeys` — the two right-hand status fields. `keyHints` (line 2101) is the likely `statusKeys`; nothing matches `statusMid`.
5. `mapCursor`, `vpContext`, `vpField`, `layersBtnBg`, `layersBtnCol`.
6. `undoCol` / `redoCol` — the enabled/disabled colours for `↶` `↷`.
7. The whole timeline binding set: `tlShow`, `tlCollapsed`, `tlExpanded`, `tlPct`, `tlYearLabel`, `tlState`, `tlSpeeds`, `tlToggles`, `tlPlayGlyph`. Whether the timeline is domain-gated is unrecoverable.
8. `ldSwitch`, `ldSwA`, `ldSwB` and their four colour bindings — the left-dock A/B switcher.
9. `tbLabel` and the conditions `tbInspect`, `tbMeasure`, `tbRegion`, `tbPipe` (the other 9 branches survive).
10. `scrPicker`, `scrShell`, `pickerWorlds`.
11. `mb`, and the `fw`/`fh`/`scale` mapping (recoverable from `frameDef()` and `_onResize`).

**Genuine design gaps** (present even in a complete file):

12. `--g` (`10px` / `12px`) is declared at both densities and **never used**. Either wire it to the band gaps that are currently literal `10px` (regions 2 and 3), or drop it.
13. `--good` (`#6fae7d` / `#2c7a44`) and `--accH` (`#f0bd72` / `#8a5309`) are declared and never used. `--accH` has no hover rule anywhere; the file's only accent hover is `color:var(--acc)`.
14. No touch override for the 200 px rail-expansion column, the 238 px layers popover, or the 168 px cross-section strip.
15. No touch override for the 12 px slider thumb, the 30×17 px toggle, or the 4 px slider track.
16. No touch override for the fixed-px display type (26 px, 22 px, 20 px, 13 px).
17. The expanded timeline has no authored height.
18. The menu popover's `max-height:72vh` is measured against the browser viewport, not the shell. A native build needs a shell-relative clamp.
19. The OFF colour of a toggle track is inconsistent: `var(--ins)` in menu rows, `var(--sur)` in dock rows. Pick one.
20. Only the pipeline stage chevron animates; six other chevrons rotate instantly.
21. `box-sizing` is `content-box` throughout (no reset in `support.js`), so every band's 1 px border sits *outside* its token height. Any Godot port that treats `--menuH` as total height will be 1 px short per band — 3 px at the top, 1 px at the bottom, per screen.
22. Line 285 still declares `hint-placeholder-count="5"` for the domain rail that now holds three domains — a fossil of the five-domain rail the README says this release replaces. Cosmetic, but it is the one place the markup still remembers the old IA.

# Cartalith DCC Environment — Desktop Left Dock Specification

Source: `C:\Users\Vincent\Cartalith_GDT\design\dcc-environment-2026-08-31\Cartalith DCC Environment.dc.html`
Markup lines 303–818; logic class `Component extends DCLogic`, lines 1164–2105.

---

## 0. CRITICAL — the source file is truncated

**The desktop `.dc.html` is exactly 262 144 bytes (256 KiB) and ends mid-token**, inside `valsCore()`:

```
      measRows.push({i:('0'+i).slice(-2),len:this.fmtKm(km),be
```

There is no closing `}`, no `render()`, and no `</script>`/`</body>`/`</html>`. The Android file in the same folder (166 424 bytes) *is* complete, so this is a per-file truncation, not a convention.

Everything in `valsCore()` after that byte is lost. That accounts for **every `UNSPECIFIED` in §9** — the dock's own title strings, the WORLD a/b mode-switch labels, the collapsed-rail label, and roughly a dozen handler bindings the markup references. The structure, controls, ranges, defaults and copy below are all recovered from the markup (lines 1–1162) plus the six `vals2()`–`vals6()` methods, which are intact.

**Get an untruncated copy of this file before building.** The gap list is short and mechanical, but it is real.

---

## 1. Design tokens (defined at markup line 25; light overrides at line 2063)

### 1.1 Colour — dark (base `:root`-equivalent, inline on the frame div)

| Token | Dark | Light (`themeStr`) |
|---|---|---|
| `--sur` surface | `#0d0e0f` | `#f4f2ee` |
| `--pan` panel (**the dock's background**) | `#121314` | `#fbfaf7` |
| `--ins` inset (track / chip / button fill) | `#191c1e` | `#eceae4` |
| `--ink` strongest text, slider knob | `#e8ebec` | `#111210` |
| `--body` body text | `#c8cbcd` | `#23241f` |
| `--sec` secondary — control labels | `#a9adb0` | `#3d3f39` |
| `--dim` dimmed — inactive chip text | `#8d9296` | `#6b6f6a` |
| `--faint` faint — section headings | `#6f7478` | `#8d9088` |
| `--dis` disabled / footnotes | `#5f6468` | `#9a9d95` |
| `--acc` accent | `#e0a34a` | `#a4650f` |
| `--accH` accent hover | `#f0bd72` | `#8a5309` |
| `--accInk` ink on accent | `#141005` | `#f7f4ee` |
| `--hair` hairline (dock's right border) | `rgba(255,255,255,.10)` | `rgba(0,0,0,.14)` |
| `--div` divider (between dock sections) | `rgba(255,255,255,.07)` | `rgba(0,0,0,.08)` |
| `--bor` border | `rgba(255,255,255,.16)` | `rgba(0,0,0,.20)` |
| `--wash` accent wash (selected row bg) | `rgba(224,163,74,.09)` | `rgba(164,101,15,.09)` |
| `--wash2` accent wash 2 (active chip bg) | `rgba(224,163,74,.16)` | `rgba(164,101,15,.16)` |
| `--shadow` | `0 14px 34px rgba(0,0,0,.55)` | `0 14px 34px rgba(35,36,31,.16)` |
| `--good` | `#6fae7d` | `#2c7a44` |
| `--block` blocked/error | `#c96a5a` | `#a03d2e` |
| `--water` | `#6a9bc4` | `#2e6a9e` |

### 1.2 Metrics — three density sets

| Token | Desktop 1920 (base) | Laptop 1366 | Tablet (2560 / 1600×2560) |
|---|---|---|---|
| `--fs` base font | 11.5px | 11.5px | 14px |
| `--m0` mono 0 | 10.5px | 10.5px | 12.5px |
| `--m1` mono 1 | 10px | 10px | 12px |
| `--m2` mono 2 (headings) | 9px | 9px | 11px |
| `--ctl` control height | 24px | 24px | 36px |
| `--btnH` button height | 28px | 28px | 44px |
| `--row` row height | 28px | 28px | 44px |
| `--tool` tool button | 30px | 30px | 44px |
| **`--ldW` left dock width** | **372px** | **330px** | **400px** |
| `--rdW` right dock | 304px | 280px | 400px |
| `--railW` domain rail / collapsed dock | 40px | 40px | 48px |
| `--pad` dock horizontal padding | 14px | 14px | 16px |
| `--g` gap | 10px | 10px | 12px |
| `--pop` popover width | 300px | 280px | 380px |
| `--menuH` / `--tbH` / `--sbH` | 36 / 40 / 26px | same | 52 / 56 / 36px |

Touch density (`isTouch()`) is `frame === 'tabL' || frame === 'tabP'`. Frame sizes: `w1920` 1920×1080, `w1366` 1366×768, `tabL` 2560×1600, `tabP` 1600×2560.

### 1.3 Typography

Two families only:
- Body/UI: `'Helvetica Neue', Helvetica, Arial, sans-serif` at `var(--fs)`, line-height 1.45.
- Mono: `'IBM Plex Mono', monospace` (Google Fonts, weights 400 & 500) — used for **every** section heading, numeric readout, footnote and chip label.

Heading recipe, used for all uppercase section headings in the dock:
`font: var(--m2) 'IBM Plex Mono', monospace; letter-spacing: .2em; color: var(--faint)`
(Landmark/CIVIL category headers use `.18em`; the `CLASS RADII · ADVANCED` fold uses `.14em`; landmark family labels use `.14em`.)

---

## 2. Dock chrome

The dock sits inside `<div style="flex:1;display:flex;min-height:0">`, immediately right of the domain rail (and of the optional 200px expanded rail-node list).

### 2.1 Open state (markup 303–811)

```
width: var(--ldW); flex: none;
border-right: 1px solid var(--hair);
background: var(--pan);
display: flex; flex-direction: column; min-height: 0;
```

Four stacked bands, top to bottom:

| # | Band | Box | Content |
|---|---|---|---|
| 1 | **Header** | `flex:none; display:flex; align-items:center; gap:8px; padding:8px var(--pad) 0` | Title `{{ ldTitle }}` — mono `var(--m2)`, letter-spacing `.2em`, `var(--faint)`, `flex:1`. Then the collapse control. |
| 2 | **Mode switch** (conditional, `{{ ldSwitch }}`) | `flex:none; padding:8px var(--pad) 2px` | See §2.3. |
| 3 | **TOOLS** (always) | `flex:none; padding:10px var(--pad); border-bottom:1px solid var(--div)` | See §2.4. |
| 4 | **Scroll body** | `flex:1; overflow-y:auto; min-height:0` | The per-mode content, §4–§8. |

**Only band 4 scrolls.** The header, mode switch and TOOLS block are pinned. There is no dock footer — the last section of each mode's body is its footnote, and it scrolls with the content. The markup ends the scroll body with an extension point `<!--ANCHOR_LD-->` (line 808).

### 2.2 Collapse chevron

- Glyph: **`‹`** (U+2039). Handler `hLdClose`.
- Box: `width: var(--ctl); height: var(--ctl); border-radius: 8px; display:grid; place-items:center; cursor:pointer; color: var(--faint)`; hover `background: var(--ins)`.

Collapsed state (markup 812–817, gated on `{{ ldClosed }}`), the whole strip is one click target (`hLdOpen`):

```
width: var(--railW); flex: none;
border-right: 1px solid var(--hair); background: var(--pan);
display:flex; flex-direction:column; align-items:center; gap:12px; padding:8px 0; cursor:pointer;
```
- Glyph **`›`** (U+203A), `var(--faint)`.
- Vertical label `{{ ldCollapsedLabel }}` — `writing-mode: vertical-rl; transform: rotate(180deg)`, mono `var(--m2)`, letter-spacing `.2em`, `var(--dim)`.

**Also toggled from the menu bar:** `Window ▸ Left dock` is a checkbox row bound to `win:ld`, which flips `state.ldOpen`. `Window ▸ Reset layout` sets `ldOpen: true` (plus `rdOpen: frame !== 'tabP'`, `showRail: true`, `showSB: true`, `railExp: false`) and toasts `Layout reset`.
`openRd()` closes the left dock when the right dock is opened **on the `tabP` frame only** — never on desktop.

### 2.3 Mode switch (`ldSwitch`)

A two-segment pill, shown only when `ldSwitch` is true. From the markup and state, this is the WORLD **a / b** switch (`state.worldMode`, default `'a'`); CIVIL uses its own four category headers instead (§6) and CARTO has no switch.

```
container: display:flex; background: var(--ins); border-radius:999px; padding:3px
segment  : flex:1; text-align:center; padding:5px 0; border-radius:999px; cursor:pointer;
           font: var(--m2) 'IBM Plex Mono', monospace; letter-spacing:.12em
```
Segment A: `data-v="a"`, text `{{ ldSwA }}`, colour `{{ ldSwACol }}`, fill `{{ ldSwABg }}`.
Segment B: `data-v="b"`, text `{{ ldSwB }}`, colour `{{ ldSwBCol }}`, fill `{{ ldSwBBg }}`.
Handler `hLdMode` reads `dataset.v`.

`UNSPECIFIED:` the literal strings `ldSwA` / `ldSwB` and the condition `ldSwitch`. Truncated. The domain-rail node labels for WORLD are `Generation pipeline` (mode `a`) and `Sculpt` (mode `b`) — those are the mode names, but the switch's own two labels are not recoverable.

### 2.4 TOOLS block

Heading text: **`TOOLS`**, `margin-bottom: 7px`.
Row: `display:flex; gap:5px; flex-wrap:wrap`.

**Global tools** — four icon buttons, always present in every domain. Each `var(--tool)` square, `border-radius: 8px`, `display:grid; place-items:center`. Active: `background: var(--wash2)`, `color: var(--acc)`. Inactive: `background: var(--ins)`, `color: var(--sec)`.

| id | `title` tooltip | Icon (14×14 SVG, stroke `currentColor`, width 1.2, round caps) |
|---|---|---|
| `inspect` | `Inspect · V` | arrow cursor — `M4 2.2 12.2 9.4 8.6 9.8 10.6 13.8 8.8 14.6 6.9 10.6 4 13Z` |
| `measure` | `Measure · M` | ruler — rotated `rect x1.8 y9.2 w12.4 h4.4 rx.8 rotate(-30 8 11.4)` + three tick strokes |
| `region` | `Region select · R` | dashed marquee corners |
| `pan` | `Pan / zoom — always available` | hand |

`pan` is **permanently styled inert**: `bg: var(--ins)`, `col: var(--dis)`, overriding the active-state computation. It never highlights.

**Divider**: `width:1px; height:var(--tool); background:var(--div); margin:0 3px`.

**Domain tools** — pill buttons, `min-height: var(--tool)`, `padding: 2px 12px`, `border-radius: 8px`, `gap: 6px`, mono `var(--m1)`; label then key hint in `var(--faint)`. Same active/inactive colours as above.

| Domain | Pills (label / key hint) |
|---|---|
| WORLD | `Sculpt` (no key) · `Freehand` **F** · `Biome paint` **B** |
| CIVIL | `Settlement` **S** · `POI` **P** · `Territory` **T** · `Way` **W** · `Route` **⇧R** |
| CARTO | `Label` **L** · `Icon` **I** |

Handler `hTool` → `armTool(dataset.id)`.

**`armTool(id)` state effects (line 1939):**
- If `state.finalized` and id ∈ `{biome, sculpt, freehand, settlement, poi, territory, way, route}` → refuse, toast `World is finalized — <id> is locked`.
- Sets `tool: id`.
- **If id is `sculpt` or `freehand`, it also forces `domain: 'WORLD'` and `worldMode: 'b'`** — arming a sculpt tool from anywhere jumps the dock to WORLD·b.
- If id is `measure`, resets `measure.done = false`.

**Keyboard (line 1833):** the global `keydown` map is `{v:inspect, m:measure, r:region, b:biome, l:label, i:icon, f:freehand}`.
> **Design defect:** the CIVIL pills advertise **S P T W ⇧R** and the status bar hint prints `S P T W` for CIVIL, but **none of `s p t w r⇧` is in the key map.** `r` is bound to `region`, not `route`. The five CIVIL keyboard shortcuts drawn in the dock do not exist.

### 2.5 Shared control primitives

Every slider, toggle, stepper and chip in the dock is one of these five. Build them once.

**Slider** (all sliders are identical geometry; only the label width and readout width change):
```
hit area : flex:1; height:var(--ctl); display:flex; align-items:center; cursor:pointer; touch-action:none
track    : flex:1; height:4px; border-radius:2px; background:var(--ins); position:relative
fill     : absolute; left:0; top:0; bottom:0; border-radius:2px; background:var(--acc); width:{pct}
knob     : absolute; top:-4px; margin-left:-6px; 12×12; border-radius:50%; background:var(--ink); left:{pct}
```
Drag model — `startSlide(e, cb)` (line 2056): captures the pointer, and on pointerdown *and* every pointermove computes `p = clamp((clientX − rect.left) / rect.width, 0, 1)`, then calls back. **The slider jumps to the click position on press** (it calls `move(e)` immediately). Release on pointerup or pointercancel. `pct` is always `Math.round(normalised × 100) + '%'`.

**Stepper buttons** flank the slider on pipeline / sculpt / brush / planner rows:
`width/height: var(--ctl); border-radius:8px; background:var(--ins); color:var(--sec); display:grid; place-items:center`
Glyphs **`−`** (U+2212, `data-d="-1"`) and **`＋`** (U+FF0B fullwidth, `data-d="1"`). One `step` per click, clamped to `[min, max]`.

**Toggle:**
`30 × 17px, border-radius:999px`; on `background: var(--acc)`, off `background: var(--sur)`.
Knob: `absolute; top:2px; 13×13; border-radius:50%; background: var(--ink); left: 15px` when on, `2px` when off.

**Segmented control:** container `display:flex; background:var(--ins); border-radius:999px; padding:2px` (planner uses `border-radius:14px`, `flex-wrap:wrap`). Options `padding:3px 9…10px; border-radius:999px; font:var(--m1) mono`. Selected `color:var(--acc); background:var(--wash2)`; unselected `color:var(--dim); background:transparent`.

**Chip (free-standing, not in a segment container):** `padding:3px 9…10px; border-radius:999px` (or `8px` for square-ish action chips). Selected `background:var(--wash2); color:var(--acc)`; unselected `background:var(--ins); color:var(--sec)`.

---

## 3. Which body renders, per domain and mode

The scroll body is a flat sequence of `sc-if` blocks in this markup order. More than one can be true at once.

| Order | Block | Condition | Defined at |
|---|---|---|---|
| 1 | Generation pipeline | `ldPipe` | **UNSPECIFIED** (truncated) — implied `domain==='WORLD' && worldMode==='a'` |
| 2 | Sculpt | `ldSculpt` = `domain==='WORLD' && worldMode==='b'` | 1757 |
| 3 | CARTO layers & style | `ldCarto` = `domain==='CARTO'` | 1566 |
| 4 | CIVIL header — LANDMARKS | `ldCivilDock` = `domain==='CIVIL'` | 1300 |
| 5 | Landmarks body | `ldLandmarks` = `CIVIL && civCat==='landmarks'` | 1307 |
| 6 | CIVIL header — FACTIONS & SETTLEMENTS | `ldCivilDock` | |
| 7 | Factions body | `ldCivil` = `CIVIL && civCat==='factions'` | 1566 |
| 8 | Terrain appearance | `ldRender` = `domain==='CARTO'` | 1566 |
| 9 | CIVIL header — WAYS & ROUTES | `ldCivilDock` | |
| 10 | Ways body | `ldRoutes` = `CIVIL && civCat==='infra'` | 1477 |
| 11 | CIVIL header — JOURNEY PLANNER | `ldCivilDock` | |
| 12 | Planner body | `ldPlanner` = `CIVIL && civCat==='planner'` | 1477 |

**Consequences a builder must honour:**

1. **CARTO renders blocks 3 and 8 together**, and block 8 sits *after* the (hidden) CIVIL factions block in source order. The CARTO dock is therefore always: search → layer list → selected-layer properties → `TERRAIN APPEARANCE · MOCK`.
2. **The four CARTO rail nodes do not switch the dock.** `Layers & style`, `Labels`, `Icons` and `Terrain appearance` are declared with `mode: ''` (line 2069), so all four highlight simultaneously whenever `domain === 'CARTO'`, and all four show the same dock. Labels and Icons have **no left-dock body at all** — their controls (`sizeModeChips`, `anchorChips`, `iconFamChips`, `iconVar`, `iconScale`) live in the top toolbar and right dock, not here.
3. **CIVIL always shows all four category headers**, with exactly one body expanded between them. The headers are interleaved with the bodies, so the layout is accordion-shaped, not tabbed.
4. `civCats` is computed in `vals6()` (line 1300) as `[['landmarks','LANDMARKS'], ['factions','FACTIONS & SETTLEMENTS'], ['infra','WAYS & ROUTES'], ['planner','JOURNEY PLANNER']]` but **is never consumed by the markup** — the four headers are hand-written. Dead binding; the labels below are the hand-written ones, which match.

---

## 4. WORLD · a — Generation pipeline

Data source: `this.STG` (line 1176) — 10 stages. Defaults: `this.DEFAULTS` (line 1196), copied into `state.params` at construction.

### 4.1 Stage header row (repeated ×10)

Container `border-bottom: 1px solid var(--div)`. Header `min-height:var(--row); display:flex; align-items:center; gap:8px; padding:7px var(--pad); cursor:pointer`; handler `hStageOpen` with `data-n`.

| Element | Spec |
|---|---|
| Number | `width:18px`, mono `var(--m1)`, zero-padded 2 digits: `01`…`10` |
| State dot | `7×7px; border-radius:50%; border:1px solid` |
| Name | weight 600 normally, 400 when resolved |
| Spacer | `flex:1` |
| State label | mono `var(--m2)` |
| Chevron | **`▸`**, `var(--faint)`, `transform: rotate(0deg)` closed / `rotate(90deg)` open, `transition: transform .15s` |

**Four stage states** — `stageState(n)` (line 1963): `running` if `run.i === n`; else if `staleFrom != null`: `editing` when `n === staleFrom`, `stale` when `n > staleFrom`; else `resolved`.

| State | State label | Number colour | Dot fill | Dot border | Name colour / weight | Label colour |
|---|---|---|---|---|---|---|
| resolved | `✓ resolved` | `--faint` | `--dis` | `--dis` | `--sec` / 400 | `--dis` |
| editing | `● editing` | `--acc` | `--acc` | `--acc` | `--ink` / 600 | `--acc` |
| stale | `○ stale` | `--acc` | transparent | `--acc` | `--ink` / 600 | `--acc` |
| running | `running NN%` (`Math.round(run.pct)`) | `--acc` | `--acc` | `--acc` | `--ink` / 600 | `--acc` |

Only **one** stage is open at a time (`state.openStage`, **default 4**).

### 4.2 Stage body

`padding: 2px var(--pad) 12px; display:flex; flex-direction:column; gap:8px`.

First: a two-line provenance block, mono `var(--m2)`, line-height 1.7, `var(--faint)`, literally
`needs {needs}` `<br>` `produces {produces}`.

Then the fields, then a two-button run row.

**Field row geometry** — label column `width: 96px; flex:none; color: var(--sec)`; slider rows are `label · − · slider · ＋ · readout`, readout `width: 52px; text-align:right; mono var(--m1); color: var(--ink)`.

**Value formatting** (`fmtV`, line 2078): `dec = step >= 1 ? 0 : step >= 0.1 ? 1 : 2`; display `value.toFixed(dec) + (unit || '')`. Segmented fields display their raw string.

**Run row:** `display:flex; gap:5px`
- `Run stage {NN}` — `padding:4px 12px; border-radius:8px; background:var(--acc); color:var(--accInk); font:500 var(--m1) mono`. Handler `hRunOne`.
- `Run {NN} → 10` — same box, `background:var(--ins); color:var(--sec)`, weight 400. Handler `hRunFrom`.

### 4.3 The ten stages, in full

| # | Name | `needs` | `produces` |
|---|---|---|---|
| 01 | `Planet` | `—` | `gravity, rotation, tilt, geoid, tides → 02 08` |
| 02 | `Extent & scale` | `01` | `land/sea split, all distances` |
| 03 | `World structure` | `01` | `continentality.f32` |
| 04 | `Tectonics` | `01, 03` | `elevation, plate_id, boundary_type, resistance` |
| 05 | `Volcanism & impacts` | `04` | `cones, provinces, craters` |
| 06 | `Erosion` | `04, 08` | `final surface` |
| 07 | `Hydrology` | `06` | `rivers, lakes, drainage, flow accumulation` |
| 08 | `Climate` | `01, 02, 06` | `temperature, rainfall, wind, currents` |
| 09 | `Ecology & biomes` | `07, 08` | `biome classification, ecotones` |
| 10 | `Resources & soils` | `04, 08, 09` | `soil depth, ore, fertility` |

Note stage 06 needs 08 and stage 08 needs 06 — the declared dependency graph is cyclic as written.

#### 01 Planet
| key | label | type | min | max | step | unit | default |
|---|---|---|---|---|---|---|---|
| `gravity` | `gravity` | slider | 0.5 | 2 | 0.01 | ` g` | 1 → `1.00 g` |
| `day` | `day length` | slider | 6 | 48 | 1 | ` h` | 24 → `24 h` |
| `tilt` | `axial tilt` | slider | 0 | 45 | 0.1 | `°` | 23.4 → `23.4°` |

#### 02 Extent & scale
| key | label | type | options / range | default |
|---|---|---|---|---|
| `res` | `working res` | **segmented** | `512` `1024` `2048` `4096` `8K` | `2048` |
| `sea` | `sea level` | slider 0–100 step 1, unit ` %` | | 42 → `42 %` |
| `peak` | `peak altitude` | slider 1000–9000 step 100, unit ` m` | | 4000 → `4000 m` |

#### 03 World structure
| key | label | type | range | default |
|---|---|---|---|---|
| `arch` | `archetype` | **segmented** | `Earth` `Super` `Islands` `Volcanic` `Rift` | `Earth` |
| `cont` | `continentality` | slider | 0–1 / 0.01 | 0.30 |
| `frag` | `fragmentation` | slider | 0–1 / 0.01 | 0.50 |
| `ten` | `tectonic energy` | slider | 0–1 / 0.01 | 0.60 |
| `od` | `ocean depth` | slider | 0–1 / 0.01 | 0.60 |
| `hot` | `hotspot density` | slider | 0–1 / 0.01 | 0.20 |

#### 04 Tectonics — nine sliders
| key | label | min | max | step | unit | default |
|---|---|---|---|---|---|---|
| `plates` | `plates` | 4 | 40 | 1 | — | 14 |
| `drift` | `drift` | 0.1 | 3 | 0.05 | `×` | 1 → `1.00×` |
| `warp` | `warp` | 0 | 1 | 0.01 | — | 0.45 |
| `uplift` | `uplift spread` | 2 | 60 | 1 | ` px` | 18 → `18 px` |
| `alpha` | `α uplift` | 0 | 2 | 0.01 | — | 0.85 |
| `beta` | `β decay` | 0 | 1 | 0.01 | — | 0.22 |
| `flex` | `flexure F` | 0 | 1 | 0.01 | — | 0.20 |
| `het` | `heterogeneity C` | 0 | 0.5 | 0.01 | — | 0.08 |
| `rock` | `rock resistance` | 0 | 1 | 0.01 | — | 0.50 |

#### 05 Volcanism & impacts
| key | label | type | range | default |
|---|---|---|---|---|
| `vol` | `volcanoes` | slider | 0–100 / 1 | 20 |
| `vage` | `volcano age` | slider | 0–1 / 0.01 | 0.40 |
| `prov` | `igneous provinces` | **toggle** | — | **on** |
| `crat` | `craters` | slider | 0–400 / 10 | 100 |

#### 06 Erosion — six sliders, all 0–1 step 0.01
| key | label | default |
|---|---|---|
| `drop` | `droplet` | 0.55 |
| `hill` | `hillslope diffuse` | 0.40 |
| `stream` | `stream power` | 0.62 |
| `velo` | `velocity field` | 0.35 |
| `glac` | `glacial` | 0.25 |
| `coast` | `coastal` | 0.45 |

#### 07 Hydrology
| key | label | type | range | default |
|---|---|---|---|---|
| `rdens` | `river density` | slider | 0–1 / 0.01 | 0.50 |
| `order` | `min stream order` | slider | 1–8 / 1 | 3 |
| `lakes` | `lakes as water` | **toggle** | — | **on** |

#### 08 Climate
| key | label | type | range | unit | default |
|---|---|---|---|---|---|
| `eq` | `equator °C` | slider | 10–45 / 1 | `°` | 30 → `30°` |
| `pole` | `pole °C` | slider | −60–10 / 1 | `°` | −18 → `-18°` |
| `lapse` | `lapse rate` | slider | 3–10 / 0.1 | `°/km` | 6.5 → `6.5°/km` |
| `rain` | `rainfall` | slider | 0.2–2 / 0.05 | `×` | 1 → `1.00×` |
| `koppen` | `seasons & Köppen` | **toggle** | | | **on** |
| `cur` | `ocean currents` | **toggle** | | | **on** |

#### 09 Ecology & biomes
| key | label | type | range | default |
|---|---|---|---|---|
| `eco` | `ecotone sharpness` | slider | 0–1 / 0.01 | 0.50 |
| `riv` | `rivers in biome view` | **toggle** | — | **on** |

#### 10 Resources & soils
No controls. One read-only line, mono `var(--m2)`, line-height 1.6, `var(--faint)`:
> `derived only — soil depth, ore bodies, fertility. no direct controls.`

The `Run stage 10` / `Run 10 → 10` buttons still render.

> `state.params` also carries `rdens2: 0`, which no field references. Dead default.

### 4.4 Staleness and the run machine

**`setField(n, k, v)` (line 1946):**
- If `state.finalized`, refuse and toast `Finalized — unlock to edit stages`.
- Sets `params[k] = v` and `staleFrom = (staleFrom == null) ? n : min(staleFrom, n)`.
- Pushes an undo entry labelled `edit stage NN`, **debounced at 900 ms** — a drag produces one undo entry, not one per sample.

`fieldMeta(n, k)` returns the field descriptor, i.e. the clamp bounds and step.

**`runStages(from, to)` (line 1953):**
- Refuses if finalized (toast `Finalized — unlock to run`) or if a run is already active.
- Sets `run = {i: from, pct: 0, to}` and clears `log`.
- Ticks every **220 ms**, `pct += 16 + random()*24`.
- At each 100%, appends a log line `{t: 'NN Name — resolved · X.X s', col: 'var(--dim)'}` where `X.X = (0.4 + random()*2.1).toFixed(1)`, then advances `i` and resets `pct`.
- On finishing stage `to`: `run = null`; `lastRun = HH:MM`; `staleFrom` becomes `null` if `to >= 10`, else `to + 1` if the old `staleFrom <= to`, else unchanged; `world.status` becomes `stages 01–10 resolved` when `to >= 10`.
- Toast: `Stages NN → NN resolved` when `to > from`, else `Stage NN resolved`.
- `log` retains at most 3 lines (`[...log.slice(-2), line]`).

### 4.5 Generation log block (conditional, `{{ genLog }}`)

`padding: 9px var(--pad); border-bottom: 1px solid var(--div); display:flex; flex-direction:column; gap:3px`.
Each line from `genLogLines` is one span, mono `var(--m2)`, colour from the line's own `col`. Shape matches `state.log` entries `{t, col}`.

`UNSPECIFIED:` the bindings `genLog` and `genLogLines` (truncated). The producing array `state.log` and its `{t, col}` shape are specified above.

### 4.6 Finalize block (always last in the pipeline body)

`padding: 11px var(--pad); display:flex; flex-direction:column; gap:8px`. Not inside a stage.

Row 1: heading **`FINALIZE · LOD 0–3 · 85 TILES`** (mono `var(--m2)`, `var(--faint)`) · spacer · button `{{ bakeLabel }}` (`padding:4px 12px; border-radius:8px; font:500 var(--m1) mono`, fill `{{ bakeBg }}`, ink `{{ bakeCol }}`, handler `hFinalize`).

Row 2, footnote — mono `var(--m2)`, line-height 1.7, `var(--dis)`, verbatim:
> `NOT A GENERATION STAGE — gpu & multi-gpu → Preferences ▸ Performance · render quality, lighting, 3D → Preferences ▸ Graphics · tiled LOD, atlas → Preferences ▸ Tiles & LOD · appearance, ramps → CARTO · settlements, routes → CIVIL`

`UNSPECIFIED:` `bakeLabel`, `bakeBg`, `bakeCol` and what `hFinalize` does (truncated). What *is* specified: it drives `state.finalized` (default `false`), and finalization locks stage editing, stage runs, and the eight tools listed in §2.4; the status bar then reads `finalized — stages locked · atlas L0–L3 baked`, and the sculpt footnote flips to `world is finalized — sculpting locked; unlock from the toolbar` — so an **unlock** affordance exists in the toolbar, not in the dock.

---

## 5. WORLD · b — Sculpt

Five sections, each `padding: 11px var(--pad)` with `border-bottom: 1px solid var(--div)` — except the last, which has no bottom border.

State: `SCD()` (line 1674). Defaults: `feature:'mountains'`, `brush:{size:64, hard:0.35, inten:1, noise:6, oct:5, pers:0.52, lac:2, edge:0.45}`, `seed:'483920'`, `shape:'circle'`, `op:'feature'`, `fall:'smooth'`, `mirror:false`, `free:'raise'`, `stamps:[]`, `sel:-1`, `redo:[]`. Per-feature parameter values are seeded from each param's `v`.

### 5.1 `GEOLOGICAL FEATURE`

Chip grid, `display:flex; flex-wrap:wrap; gap:4px`. Each chip: `min-height:var(--ctl); padding:3px 9px; border-radius:8px; display:flex; align-items:center; gap:5px; font:var(--m1) mono`, a 12×12 inline SVG then the label. Selected `bg var(--wash2)`, `col var(--acc)`; else `bg var(--ins)`, `col var(--sec)`.

13 chips, in order: `Mountains` `Hills` `Ridge` `Plateau` `Cliff` `Canyon` `Valley` `River` `Lake` `Basin` `Coastline` `Volcano` `Freehand`.

Below the grid, `padding-top:7px`, mono `var(--m2)`, line-height 1.5, `var(--faint)`: the selected feature's `hint`.

`hScFeat` sets `sc.feature`, **and** if the current tool is neither `sculpt` nor `freehand` it arms `freehand` (for the Freehand chip) or `sculpt` (for all others) — which in turn forces `domain: WORLD, worldMode: b`.

**Feature parameter table** — for each feature, its `hint` and its params (`label`, min–max / step, default). Radial features are marked; they place on tap rather than on drag.

| Feature | Hint | Params |
|---|---|---|
| **Mountains** | `stroke · add — height, peak sharpness, ridge frequency, ruggedness` | `height` 0.1–0.55/0.01 = **0.42** · `peak sharpness` 0.6–3/0.1 = **1.5** · `ridge freq` 0.6–5/0.1 = **1.6** · `ruggedness` 0–1/0.01 = **0.55** |
| **Hills** | `stroke · add — soft rolling amplitude` | `amplitude` 0.02–0.3/0.01 = **0.11** · `rolling freq` 0.5–4/0.1 = **1.4** · `softness` 0–1/0.01 = **0.7** |
| **Ridge** | `stroke · add — one crest along the stroke axis` | `height` 0.02–0.35/0.01 = **0.15** · `width frac` 0.1–0.6/0.01 = **0.28** · `detail freq` 0.5–4/0.1 = **1.5** |
| **Plateau** | `stroke · set — never lowers existing terrain` | `rise` 0.03–0.45/0.01 = **0.26** · `terraces` 1–8/1 = **4** · `detail freq` 0.4–3/0.1 = **1.1** |
| **Cliff** | `stroke · add — direction-sensitive, high side left of the stroke` | `rise` 0.05–0.45/0.01 = **0.22** · `steepness` 0.2–1/0.01 = **0.75** |
| **Canyon** | `stroke · add negative — walls closing to a flat floor` | `depth` 0.03–0.35/0.01 = **0.18** · `wall steepness` 0–1/0.01 = **0.7** · `meander` 0–0.8/0.01 = **0.35** |
| **Valley** | `stroke · add negative — U-shaped trough` | `depth` 0.03–0.3/0.01 = **0.14** · `width frac` 0.3–1/0.01 = **0.85** · `meander` 0–0.8/0.01 = **0.3** |
| **River** | `stroke · set — writes riverMask and riverFloor on commit` | `width` 2–26/1 ` px` = **7** · `depth` 0.02–0.22/0.01 = **0.09** · `meander` 0–0.6/0.01 = **0.28** · `branch noise` 0–1/0.01 = **0.5** |
| **Lake** *(radial)* | `radial · set — tap places it, brush size is the radius; fills lakeMask on commit` | `depth` 0.03–0.3/0.01 = **0.13** · `shore` 0.05–0.6/0.01 = **0.25** |
| **Basin** | `stroke · add negative — endorheic, no outlet` | `depth` 0.02–0.25/0.01 = **0.1** · `floor rough` 0–1/0.01 = **0.4** |
| **Coastline** | `stroke · set — pulls terrain toward sea level` | `amount` 0.1–1/0.01 = **0.85** · `raggedness` 0.4–4/0.1 = **1.6** |
| **Volcano** *(radial)* | `radial · add — tap places the cone; crater notch at the summit` | `cone height` 0.15–0.6/0.01 = **0.45** · `crater depth` 0–0.9/0.01 = **0.5** · `radius` 30–200/5 ` px` = **110** · `flank rough` 0–1/0.01 = **0.6** |
| **Freehand** | `continuous drag or tap — sub-mode below; a one-point stroke degenerates to radial` | `amount` 0.02–0.3/0.01 = **0.12** |

### 5.2 `PRESETS`

Pill row, `flex-wrap:wrap; gap:4px`; each `min-height:var(--ctl); padding:3px 10px; border-radius:999px; background:var(--ins); color:var(--sec); font:var(--m1) mono`; hover `color:var(--acc)`. **No selected state** — presets are one-shot applies.

Eight, in order, each switching feature and overwriting that feature's params:

| Label | Feature | Parameter overrides |
|---|---|---|
| `Rolling Hills` | hills | amp 0.09, freq 1.1, soft 0.85 |
| `Alps` | mountains | h 0.5, sharp 2.4, rfreq 2.2, rug 0.7 |
| `Rockies` | mountains | h 0.44, sharp 1.8, rfreq 1.4, rug 0.6 |
| `Badlands` | canyon | depth 0.22, wall 0.85, mea 0.55 |
| `Volcanic Isle` | volcano | cone 0.52, crat 0.6, rad 140, flank 0.7 |
| `Mesa` | plateau | rise 0.3, terr 5, det 1.4 |
| `Karst` | hills | amp 0.16, freq 2.8, soft 0.3 |
| `Glacial Valley` | valley | depth 0.2, w 0.95, mea 0.15 |

Toast on apply: `{Label} — seeds {feature}, never paints`.

### 5.3 `FEATURE PARAMETERS · {FEATURE}`

Heading is dynamic: literal `FEATURE PARAMETERS · ` + the feature label uppercased (e.g. `FEATURE PARAMETERS · MOUNTAINS`). Section `gap: 8px`.

**Freehand sub-mode row** — shown when `sc.feature === 'freehand' || tool === 'freehand'`. Eight chips, `padding:3px 10px; border-radius:999px; font:var(--m1) mono`, selected `wash2`/`acc`:
`raise` `lower` `smooth` `cliff` `ridge` `canyon` `mesa` `volcano` — default **`raise`**.

**High-side row** — shown when `sc.feature === 'cliff' && tool !== 'freehand'`. Label column 96px, text `high side`; segmented pair with the literal option texts **`left of stroke`** and **`right of stroke`**; default `left`. Followed by a footnote, mono `var(--m2)`, line-height 1.5, `var(--dis)`:
> `relative to the direction you draw — ticks on the map mark the high side; flip it per stamp in the stack`

**Parameter sliders** — one row per param of the current feature (`label · − · slider · ＋ · readout`), label 96px, readout **52px**. Same `dec` rule as §4.2. Step buttons clamp; the slider quantises to `Math.round(raw/step)*step` rounded to 4 dp.

### 5.4 `BRUSH & NOISE · GLOBAL`

Eight sliders, label 96px, readout **60px**:

| key | label | min | max | step | default | readout |
|---|---|---|---|---|---|---|
| `size` | `brush size` | 6 | 200 | 2 | **64** | `64 px · 160.0 km` — special format: `v + ' px · ' + fmtKm(v*2.5)` |
| `hard` | `hardness` | 0 | 1 | 0.01 | **0.35** | `0.35` |
| `inten` | `intensity` | 0 | 1.5 | 0.05 | **1** | `1.00` |
| `noise` | `noise scale` | 1 | 20 | 0.5 | **6** | `6.0` |
| `oct` | `octaves` | 1 | 8 | 1 | **5** | `5` |
| `pers` | `persistence` | 0.2 | 0.9 | 0.01 | **0.52** | `0.52` |
| `lac` | `lacunarity` | 1.4 | 3.2 | 0.05 | **2** | `2.00` |
| `edge` | `edge noise` | 0 | 1 | 0.01 | **0.45** | `0.45` |

Slider quantises to `Math.round(raw/step)*step` at 3 dp; steppers clamp to `[min,max]` at 3 dp.

**Seed row** (last): label column 96px `seed` · value `{{ scSeed }}` mono `var(--m1)` `var(--ink)` · dice button `var(--ctl)` square, radius 8, `background:var(--ins)`, containing a 13×13 die SVG (rounded rect + 3 pips). Handler `hScDice` → `String(floor(random()*900000 + 100000))`, i.e. a 6-digit seed in **100000–999999**.

### 5.5 `BRUSH SHAPE`

**Shape chips** (`flex-wrap; gap:4px`, `padding:3px 10px; border-radius:999px`), 8 + 1:
`circle` `directional` `spatter` `spiral` `dots` `cloud` `checker` `hatch` — default **`circle`**.
Then `import…` — `border: 1px dashed var(--bor); color: var(--faint)`, no fill. Toast: `Import brush — greyscale height stamp, alpha respected (mock)`.

**`operation`** — label 96px, segmented (`flex-wrap:wrap`), 6 options:
`feature` `add` `subtract` `multiply` `min` `max` — default **`feature`**.

**`falloff`** — label 96px, segmented, 4 options:
`smooth` `linear` `sharp` `constant` — default **`smooth`**.

**Toggle row** — `mirror across stroke axis`, label `flex:1` `var(--sec)`, standard toggle, default **off**.

### 5.6 `STROKE & GRID · SELECTED STAMP` (no bottom border)

Two chip rows, `padding:3px 10px; border-radius:8px; background:var(--ins); font:var(--m1) mono`. Colour is `var(--sec)` when a stamp is selected (`sc.sel >= 0`) and **`var(--dis)` when none is** — the rows read as disabled with an empty stack.

Row 1 (`strokeOps`, 8): `add point` `duplicate` `rotate` `scale` `tilt` `push` `pull` `align`
Row 2 (`actOps`, 5): `flip x` `flip y` `rot left` `rot right` `flatten`

Clicking with no selection toasts `Select a stamp in the stack first`. With a selection, every one of the 13 is a mock: toast `{id} — edits the stamp control points, not the heightfield (mock)`.

Footnote, mono `var(--m2)`, line-height 1.6, `var(--dis)` — two variants:
- normal: `every stroke becomes a live procedural stamp · commit bakes the stack in one pass`
- when `state.finalized`: `world is finalized — sculpting locked; unlock from the toolbar`

> The stamp stack itself (`STAMP STACK`, commit/discard) lives in the **right dock**, not here.

---

## 6. CIVIL — four category accordions

Each of the four headers is an identical row, always visible whenever `domain === 'CIVIL'`:

```
min-height: var(--row); display:flex; align-items:center; gap:8px;
padding: 6px var(--pad); cursor:pointer; border-bottom: 1px solid var(--div)
```
- Chevron **`▸`**, `transform: rotate(0deg)` closed / `rotate(90deg)` open, coloured `var(--acc)` when open and `var(--faint)` when closed.
- Label, mono `var(--m2)`, letter-spacing `.18em`, same colour as the chevron.

| `data-id` | Label (verbatim) | Right-hand count |
|---|---|---|
| `landmarks` | `LANDMARKS` | `{{ lmCatCount }}` — see below |
| `factions` | `FACTIONS & SETTLEMENTS` | none |
| `infra` | `WAYS & ROUTES` | none |
| `planner` | `JOURNEY PLANNER` | none |

`lmCatCount` = `` `${armedCount} armed · ${placedTotal} on the map` `` (mono `var(--m2)`, `var(--faint)`), pushed right by a `flex:1` spacer.

**`hCivCat` (line 1301):** `setCc(cc === id && id !== 'landmarks' ? 'landmarks' : id)`.
So: clicking a closed category opens it; clicking the open **Landmarks** header does nothing; clicking any other open header collapses back to Landmarks. **Landmarks is the floor — one category is always open, and it is never zero.** Default `civCat = 'landmarks'`.

`setDomain(d)` (line 2054) sets `domain` and `railExp: false` only — **it does not reset `civCat` or `worldMode`**, so leaving and re-entering CIVIL returns to the category you left.
`Edit`-menu jump `lmjump:{familyId}` sets `domain:'CIVIL', civCat:'landmarks'` and opens that landmark family.

---

## 6a. CIVIL · Landmarks

The whole body sits inside `<div style="position:relative; border-bottom:1px solid var(--div)">` so the funnel popover can be absolutely positioned inside it.

### 6a.1 `PLACEMENT`

`padding: 10px var(--pad); gap: 8px; border-bottom: 1px solid var(--div)`.

**Headroom line** (mono `var(--m2)`, `var(--sec)`) — `lmHeadroom`:
> `caps total {capsTotal} · room for about {room} at this spacing · last run placed {placedTotal}`

where `capsTotal` = Σ `cap` over **armed** types; `placedTotal` = Σ `placed` over **all** types; `room = Math.round(210 / crowd^1.6 × (compete ? 1 : 1.35))`.

**`crowding` slider** — label column **78px**, readout **52px**.
Value = `+(0.25 + p × 1.75).toFixed(2)` → **range 0.25–2.00**, effectively step 0.01. Default **1**.
`pct = round((crowd − 0.25) / 1.75 × 100)`. Readout `× 1.00` (literal `'× ' + crowd.toFixed(2)`).
Sets `edited: true`.

**Sub-line** — `padding-left: 85px`, mono `var(--m2)`, `var(--faint)`:
`a regional landmark keeps {fmtKm(radii.REG × crowd)}` + literal ` clear` → at defaults, `a regional landmark keeps 34.0 km clear`.

**Toggle — `types compete with each other`**, default **on**. Toggle sits left (`align-items:flex-start`, `margin-top:1px`), label right in a column: title in `var(--body)`, then a sub-line mono `var(--m2)`, line-height 1.5, `var(--dis)`:
> `off lets a shrine sit beside a waterfall · on keeps every landmark clear of every other one`

Sets `edited: true`.

**Advanced fold** — a clickable row, `gap:6px`, `color: var(--faint)`: chevron `▸` (rotates to 90deg) + label mono `var(--m2)`, letter-spacing `.14em`:
> **`CLASS RADII · ADVANCED`**

Default **closed** (`lm.adv = false`). Toggling it does **not** set `edited`.

**Inside the fold** — four sliders, label column **78px**, readout **58px**:

| key | label | default radius | value on slide | displayed |
|---|---|---|---|---|
| `CON` | `Continental` | **120** | `round(2 + p × 158)` → 2–160 | `fmtKm(radius × crowd)` |
| `REG` | `Regional` | **34** | same | same |
| `LOC` | `Local` | **12** | same | same |
| `CUL` | `Cultural` | **8** | same | same |

Fill percentage is `round(radius / 160 × 100)` — note it is **not** normalised over `2..160`, so a radius of 2 shows a 1% fill, not 0%. Sets `edited: true`.

### 6a.2 `TYPES`

`padding: 10px var(--pad); gap: 7px; border-bottom: 1px solid var(--div)`.

Header row: heading `TYPES` · spacer · four class-filter chips, right-aligned, `padding:2px 9px; border-radius:999px; font:var(--m2) mono`:
`CON` `REG` `LOC` `CUL` — selected `bg var(--wash2)`, `col var(--acc)`; unselected `bg var(--ins)`, `col var(--dim)`. Click toggles; clicking the active one clears the filter. Default **none**.

The filter applies **only to the rows inside the open family** — family header counts are computed unfiltered.

Note line, mono `var(--m2)`, line-height 1.5, `var(--dis)`:
> `6 types score without viewshed — the engine has no visibility analysis yet; those rows say so in place`

(the `6` is computed as the count of `noview` types, listed in §6a.4.)

### 6a.3 Family groups

Six groups, each `border-bottom: 1px solid var(--div)`. Only one family is open at a time (`lm.openFam`, default **`physical`**); clicking the open one closes it (`openFam: null`).

**Family header** — `min-height:var(--row); gap:8px; padding:5px var(--pad)`:
chevron `▸` (`var(--faint)`, rotates) · 12×12 glyph SVG (`var(--sec)`) · label mono `var(--m2)` letter-spacing `.14em` (`var(--sec)`) · spacer · count mono `var(--m2)` (`var(--faint)`) · **`arm all`** chip · **`off`** chip.

Count string: `` `${armedInFamily} of ${typesInFamily} armed · ${placedInFamily} placed` ``.

Bulk chips: `padding:1px 8px; border-radius:999px; font:var(--m2) mono; background:var(--ins)`; `arm all` in `var(--sec)`, `off` in `var(--dim)`. They set `armed` for every type in the family (**keeping each type's existing `cap`**) and set `edited: true`. They `stopPropagation`, so they do not open/close the family.

| id | Label | Glyph |
|---|---|---|
| `physical` | `PHYSICAL` | mountain profile + ground curve |
| `transportation` | `TRANSPORTATION` | winding path + dashed line |
| `economic` | `ECONOMIC` | anvil/trough + strike |
| `military` | `MILITARY` | crenellated tower on a baseline |
| `religious` | `RELIGIOUS · CULTURAL` | arched shrine with inner arch |
| `historical` | `HISTORICAL` | broken columns |

### 6a.4 Type row

`padding: 5px var(--pad) 7px; display:flex; flex-direction:column; gap:3px; border-top: 1px solid var(--div)`.

**Line 1** — `gap: 7px`:

| Element | Spec |
|---|---|
| Class badge | `width:26px; font:8.5px 'IBM Plex Mono'; color:var(--dis)` — `CON` / `REG` / `LOC` / `CUL` |
| Type glyph | 12×12 SVG, coloured `var(--acc)` if armed else `var(--dis)` — **sourced from `window.LM_GLYPHS[t.name]`** |
| Name | `flex:1; min-width:40px; white-space:nowrap; overflow:hidden; text-overflow:ellipsis`; `var(--body)` armed / `var(--dis)` off |
| `no viewshed` badge | only when `t.noview`. `font:8.5px mono; color:var(--dis); border:1px dashed var(--bor); border-radius:4px; padding:0 4px; white-space:nowrap` |
| Cap slider | **fixed `width:118px`**, otherwise the standard slider |
| Cap readout | `width:56px; text-align:right; mono var(--m1)`; armed → `{cap} max`, off → `off` |

**Line 2 — placed bar**, only when armed. `padding-left:26px`, spacer, then a 118px × **2px** bar (`border-radius:1px`, transparent track, fill `var(--dim)`, width `placedPct`), then a **120px** right-aligned clickable readout, mono `var(--m2)`, `title="why fewer?"`.
Readout text: `` `${placed} placed · ${reason}` ``. Colour: `var(--acc)` when `reason === 'at cap'`, else `var(--dim)`. Click opens the funnel popover (§6a.5); clicking the open one closes it.
`placedPct = min(100, round(placed / cap × 100))`, or `0%` when unarmed or `cap === 0`.

**Line 2b — off note**, only when not armed. `padding-left:26px`, mono `var(--m2)`, `var(--dis)`: `` `was ${cap}` ``.

**Cap slider is a 13-stop ladder, not a linear range.** `LM_LADDER = [0, 1, 2, 3, 5, 8, 12, 20, 30, 50, 80, 120, 200]`.
- On drag: `i = round(p × 12)`; `v = LADDER[i]`. **`v === 0` disarms the type** (keeping its `cap`); any other value sets `armed: true, cap: v`. Sets `edited: true`.
- Fill percentage: the ladder index **nearest to the current cap**, as `round(idx / 12 × 100)`. An unarmed row always shows `0%`.

**Initial arming rule** (`LMD()`, line 1231): `armed = cap > 0`; stored `cap = cap > 0 ? cap : (was || 8)`.

**The 47 landmark types.** `id` = name lowercased with every non-`a–z` run replaced by `-`.

| Family | Type | Class | Initial cap | armed | `base` | `cand` | `fixed` | `noview` |
|---|---|---|---|---|---|---|---|---|
| PHYSICAL | Peak | REG | 12 | ✓ | 14 | 640 | — | ✓ |
| | Ridge | REG | 8 (`was`) | — | — | — | — | — |
| | Saddle | LOC | 5 (`was`) | — | — | — | — | — |
| | Cliff | LOC | 20 (`was`) | — | — | — | — | — |
| | Gorge | REG | 8 | ✓ | 6 | 410 | `no terrain` | — |
| | Cave | LOC | 12 (`was`) | — | — | — | — | — |
| | Waterfall | REG | 40 | ✓ | 11 | 1284 | — | — |
| | Spring | LOC | 30 (`was`) | — | — | — | — | — |
| | Lake | REG | 20 | ✓ | 12 | 520 | — | — |
| | Delta | REG | 3 (`was`) | — | — | — | — | — |
| | River confluence | LOC | 30 | ✓ | 24 | 960 | — | — |
| | Volcanic feature | CON | 2 (`was`) | — | — | — | — | ✓ |
| | Rock formation | LOC | 16 (`was`) | — | — | — | — | — |
| | Glacial feature | REG | 6 (`was`) | — | — | — | — | — |
| | Ancient forest | REG | 16 | ✓ | 9 | 300 | `candidates` | — |
| TRANSPORTATION | Mountain pass | REG | 12 | ✓ | 9 | 88 | — | — |
| | River crossing | LOC | 20 (`was`) | — | — | — | — | — |
| | Ford | LOC | 20 | ✓ | 8 | 340 | `no terrain` | — |
| | Bridge site | LOC | 12 (`was`) | — | — | — | — | — |
| | Road junction | LOC | 8 (`was`) | — | — | — | — | — |
| | Caravan station | LOC | 6 (`was`) | — | — | — | — | — |
| | Portage | LOC | 4 (`was`) | — | — | — | — | — |
| | Harbour | REG | 8 | ✓ | 4 | 60 | `candidates` | — |
| ECONOMIC | Mine | LOC | 12 | ✓ | 6 | 520 | — | — |
| | Quarry | LOC | 5 | ✓ | 3 | 280 | `no terrain` | — |
| | Salt works | LOC | 4 (`was`) | — | — | — | — | — |
| | Resource extraction site | LOC | 8 (`was`) | — | — | — | — | — |
| | Market site | LOC | 6 (`was`) | — | — | — | — | — |
| | Trade depot | LOC | 5 (`was`) | — | — | — | — | — |
| MILITARY | Fort | REG | 12 | ✓ | 10 | 240 | — | ✓ |
| | Watchtower | LOC | 30 | ✓ | 15 | 680 | — | ✓ |
| | Fortified pass | REG | 5 | ✓ | 8 | 44 | — | — |
| | Fortified crossing | LOC | 6 (`was`) | — | — | — | — | — |
| | Battlefield | CUL | 10 (`was`) | — | — | — | — | — |
| | Border marker | LOC | 8 | ✓ | 3 | 90 | `candidates` | ✓ |
| RELIGIOUS · CULTURAL | Shrine | CUL | 50 | ✓ | 18 | 1420 | — | — |
| | Temple | CUL | 8 | ✓ | 7 | 120 | — | — |
| | Sacred grove | CUL | 12 | ✓ | 8 | 380 | `no terrain` | — |
| | Sacred mountain | CON | 3 | ✓ | 5 | 22 | — | ✓ |
| | Pilgrimage site | CUL | 6 (`was`) | — | — | — | — | — |
| | Tomb | CUL | 8 | ✓ | 5 | 210 | `candidates` | — |
| | Monument | CUL | 8 (`was`) | — | — | — | — | — |
| | Ceremonial site | CUL | 7 (`was`) | — | — | — | — | — |
| HISTORICAL | Ruin | CUL | 20 | ✓ | 5 | 600 | — | — |
| | Abandoned settlement | CUL | 8 | ✓ | 3 | 150 | — | — |
| | Ancient road | CUL | 3 | ✓ | 1 | 40 | `candidates` | — |
| | Historic battlefield | CUL | 6 (`was`) | — | — | — | — | — |
| | Destroyed fortress | CUL | 4 (`was`) | — | — | — | — | — |
| | Historic crossing | CUL | 5 (`was`) | — | — | — | — | — |

Armed at load: **24 of 47.** The six `noview` types are Peak, Volcanic feature, Fort, Watchtower, Border marker, Sacred mountain.

**Placement model** (`lmCompute`, line 1238) — the shell's own simulation, which a builder should replace with the real engine but must match in the readouts:
```
base = t.base ?? max(1, round((t.was ?? 6) × 0.6))
room = max(1, round(base / crowd^1.6 × (compete ? 1 : 1.35)))
if !armed                  → placed 0, reason ''
else if fixed=='no terrain'→ placed = min(cap, base); reason = placed>=cap ? 'at cap' : 'no terrain'
else if fixed=='candidates'→ placed = min(cap, base); reason = placed>=cap ? 'at cap' : 'candidates'
else                       → placed = min(cap, room); reason = placed>=cap ? 'at cap' : 'spacing'
```

### 6a.5 Run block

`padding: 10px var(--pad) 14px; gap: 8px`.

**Stale banner** — only when `lm.edited`, mono `var(--m2)`, `var(--acc)`:
> `caps edited since the last run — results below are stale until you run`

**Run button** — full width, `min-height: var(--btnH); border-radius:8px; background: var(--acc); color: var(--accInk); font-weight:600; position:relative; overflow:hidden`.
While running, a progress overlay `background: rgba(0,0,0,.18)` fills from the left to `lmRunPct`.
Label: idle → **`Run landmark pass`**; running → `` `placing… ${min(99, round(pct))}%` ``.

`lmRun()` (line 1253): 130 ms interval, `pct += 9 + random()×14`. On completion it recomputes `res` and `marks`, stamps `lastRun` as `HH:MM`, clears `edited`, and toasts:
> `Landmark pass — {total} placed across {armedCount} armed types`

**Footnote** — mono `var(--m2)`, line-height 1.6, `var(--dis)`:
`` `last run ${lastRun} · ${placedTotal} placed · results below are that run` `` + literal ` · a cap is a ceiling, not a quota — spacing gives the restraint`
Initial `lastRun` is `—`.

### 6a.6 Funnel popover

Opened by clicking a row's `{placed} placed · {reason}` readout.

```
position:absolute; left:10px; right:10px; top:120px; z-index:40;
background: var(--pan); border: 1px solid var(--bor); box-shadow: var(--shadow);
padding: 12px 14px; display:flex; flex-direction:column; gap:6px;
data-menupop="1"   (so the document click handler does not close it)
```
Title: `{TYPE NAME uppercased} · LAST RUN`, mono `var(--m2)`, letter-spacing `.16em`, `var(--acc)`; spacer; close **`✕`** in `var(--faint)`, `padding: 0 4px`.

Five rows, `display:flex; align-items:baseline; gap:10px`; key left (`flex:1`, mono `var(--m1)`, `var(--faint)`), value right (mono `var(--m1)`, `white-space:pre`):

Let `cand = t.cand ?? 400`, `f1 = round(cand×0.7)`, `f2 = round((cand−f1)×0.62)`, `sp = max(0, cand−f1−f2−placed)`.

| Key | Value | Colour |
|---|---|---|
| `candidates evaluated` | `cand` with `en-US` thousands separators | `var(--sec)` |
| `failed min flow accumulation` (family `physical`) / `failed physical constraints` (all others) | `− {f1}   {cand−f1} left` (three spaces) | `var(--sec)` |
| `failed type constraints` | `− {f2}   {cand−f1−f2} left` | `var(--sec)` |
| `rejected by spacing` | `− {sp}   {placed} left` | `var(--sec)` |
| `cap {cap}` | `reached` / `not reached` | `var(--acc)` / `var(--faint)` |

Footer, above a `border-top: 1px solid var(--div)`, `padding-top:6px`, `font:500 var(--m0) mono; color:var(--ink)`: `` `${placed} placed` ``.
Then, mono `var(--m2)`, line-height 1.5, `var(--dis)`:
> `facts, not prose — the engine returns the five integers; the shell writes the labels`

---

## 6b. CIVIL · Factions & settlements

Three sections. State from `CVD()` (line 1534): `places:[]`, `pois:[]`, `sel:-1`, `cls:'town'`, `kind:'shrine'`, `faction:'Vhal Serai Compact'`, `terrMode:'add'`, `terrRadius:10`, `cells:{}`.

### `FACTIONS`
`padding: 11px var(--pad); border-bottom: 1px solid var(--div)`; heading `margin-bottom: 7px`.
Three fixed rows, `min-height:var(--row); gap:9px; padding:2px 4px`, selected row `background: var(--wash)` and name in `var(--acc)`:

| Name (verbatim) | Swatch (10×10, `border-radius:3px`) | Right-hand meta |
|---|---|---|
| `Vhal Serai Compact` | `#6a9bc4` | `0k cells` |
| `Kessan League` | `#c96a5a` | `0k cells` |
| `Free Marches` | `#6fae7d` | `0k cells` |

Meta = `` `${Math.round((cells[id] ?? 0)/100)/10}k cells` ``. Click sets `cv.faction` — the paint target for the Territory tool.

### `SETTLEMENTS · {count}`
Heading is literal `SETTLEMENTS · ` + `cv.places.length`.
Empty state (mono `var(--m1)`, line-height 1.7, `var(--dis)`):
> `none placed — arm Settlement and click the map`

Rows: glyph **`◆`** (mono `var(--m2)`, `var(--faint)`) · name · spacer · class (mono `var(--m2)`, `var(--dim)`). Selected row `background: var(--wash)`, name `var(--acc)`.
Click (`hCivPlaceSel`): selects the place, **arms `inspect`**, and **opens the right dock**.

### `POPULATION · ECONOMY · MOCK`
`padding: 11px var(--pad); gap: 5px`. Entirely static text, mono `var(--m1)`, line-height 1.8, `var(--dim)`, four `<br>`-separated lines:
```
population 1.24 M · urban 9%
trade routes 0 active · treasury —
politics: 3 factions · 0 wars
full civilization model is desktop-parity mock in this prototype
```

> Settlement class, POI kind and territory add/subtract chips are **not** in the left dock — they render in the toolbar/right dock (`settleClassChips` = `metropolis city town village hamlet`; `poiKindChips` = `shrine ruin mine ford beacon`).

---

## 6c. CIVIL · Ways & routes

Three sections. State from `WYD()` (line 1424): `draft:[]`, `type:'road'`, `routing:'snap'`, `snap:true`, `ways:[]`.

### `WAYS · {count}`
`padding: 11px var(--pad); border-bottom: 1px solid var(--div)`; heading `margin-bottom: 7px`, text `WAYS · ` + `ways.length`.
Empty state:
> `none drawn — arm Way and click waypoints; Esc commits`

Rows (`min-height:var(--row); gap:9px; padding:2px 4px`): glyph **`══`** (mono `var(--m2)`, `var(--faint)`) · way type (`var(--body)`) · spacer · `fmtKm(len)` (mono `var(--m2)`, `var(--dim)`). Read-only — no click handler.

Length is computed as `Σ hypot(Δx,Δy) × 2.5` — **2.5 km per world unit** is the prototype's global scale constant.

### `ROUTES`
Heading `ROUTES`, `margin-bottom: 7px`. One hard-coded row, `min-height:var(--row); gap:9px; padding:2px 4px; cursor:pointer`, hover `background: var(--wash)`:

| Glyph | Text | Right |
|---|---|---|
| **`➔`** (mono `var(--m2)`, `var(--acc)`) | `Eldra route · Vhal Serai → Port Amre` | `7 stages` |

Click (`hOpenPlanner`) sets `civCat: 'planner'`.

> **Inconsistency:** this row says `7 stages`; `PSTAGES()` and the planner both have **6**.

Sub-note, `padding-top:6px`, mono `var(--m2)`, line-height 1.7, `var(--dis)`:
> `open in the PLANNER tab for party, overrides and feasibility`

### Section footnote
`padding: 11px var(--pad)`, mono `var(--m2)`, line-height 1.7, `var(--dis)`:
> `a way is durable geometry others route over · a route is a journey along existing geometry — two tools, two records`

> The Way tool's own chips are in the toolbar/right dock, not here: `wayTypeChips` = `road track trail bridge` (default `road`); `wayRouteChips` = `freehand snap least-cost` (default `snap`); a `snap` toggle (default on).
> Drawing: tap adds a waypoint; **Esc commits** (`commitWay`). Fewer than two waypoints → toast `A way needs at least two waypoints` and the draft is dropped. On commit: toast `Way committed — staleness: routing graph rebuilt`.

---

## 6d. CIVIL · Journey planner

State from `PLD()` (line 1420). This mode reuses **exactly the pipeline's disclosure pattern** (§4.1–4.2): a collapsible group header with a chevron, then a body of slider / segmented / toggle / read rows with the same 96px label column and the same `− slider ＋` geometry. Two differences: the readout is **56px** (not 52px), and segmented containers use `border-radius: 14px` with `flex-wrap: wrap` and a `padding-top:5px` on the label.

### Route header
`padding: 10px var(--pad); border-bottom: 1px solid var(--div)`.
Line 1, `font: 500 var(--m0) mono; color: var(--ink)`:
> `ELDRA ROUTE · VHAL SERAI → PORT AMRE`

Line 2, `padding-top:3px`, mono `var(--m2)`, `var(--dim)`:
> `620 km · 6 stages · preloaded demo route · winter departure`

### The five parameter groups

Header row: `min-height:var(--row); gap:8px; padding:6px var(--pad)` — label (mono `var(--m2)`, letter-spacing `.2em`, `var(--acc)` open / `var(--faint)` closed) · spacer · summary (mono `var(--m2)`, `var(--faint)`) · chevron `▸`.
Body: `padding: 2px var(--pad) 12px; gap: 8px`.

**Open by default: `TRAVELER` only.** Groups open independently — this is a multi-open accordion, unlike the pipeline's single-open one.

Value format: `step >= 1` → `String(v) + unit`; otherwise `v.toFixed(1) + unit`.

#### `TRAVELER` — summary `12 · steady · 8 h`
| key | label | type | range / options | default |
|---|---|---|---|---|
| `group` | `group size` | slider | 1–200 / 1 | **12** |
| `pace` | `pace` | segmented | `Easy` `Steady` `Forced` | **`Steady`** |
| `hours` | `hours / day` | slider | 4–14 / 0.5, unit ` h` | **8** → `8.0 h` |
| `cargo` | `cargo` | slider | 0–2000 / 20, unit ` kg` | **240** |
| `supplies` | `supplies` | slider | 0–30 / 1, unit ` d` | **6** |
| `carry` | `carry food` | toggle | | **on** |
| `graze` | `grazing` | toggle | | **on** |
| `forage` | `foraging` | segmented | `none` `light` `heavy` | **`light`** |

#### `SEASON` — summary `winter · auto`
| key | label | type | options | default |
|---|---|---|---|---|
| `season` | `season` | segmented | `spring` `summer` `autumn` `winter` | **`winter`** |
| `weather` | `weather` | segmented | `auto` `clear` `rain` `storm` | **`auto`** |
| `drift` | `season drift` | toggle | | **off** |
| `rest` | `rest days` | segmented | `none` `1 per 6` `1 per 4` | **`1 per 6`** |

#### `CARRIAGE` — summary `auto · baggage train`
| key | label | type | options / range | default |
|---|---|---|---|---|
| `carriage` | `carriage` | segmented | `auto` `manual` | **`auto`** |
| `mode` | `transport mode` | segmented | `walking` `baggage train` `wagon train` `riders` | **`baggage train`** |
| `mount` | `mount` | segmented | `none` `horse` `pony` | **`horse`** |
| — | `vessel` | **read-only** | value `river barge · auto` | — |
| `mules` | `mules` | slider | 0–20 / 1 | **4** |
| `horses` | `horses` | slider | 0–20 / 1 | **2** |
| `carts` | `carts` | slider | 0–10 / 1 | **1** |
| `wagons` | `wagons` | slider | 0–10 / 1 | **0** |
| `promote` | `auto-promote` | toggle | | **on** |

#### `ROUTE` — summary `road · bridges`
| key | label | type | options | default |
|---|---|---|---|---|
| `road` | `road quality` | segmented | `track` `road` `paved` | **`road`** |
| `infra` | `infrastructure` | segmented | `none` `fords` `bridges` | **`bridges`** |
| `desert` | `desert water` | toggle | | **off** |
| `closures` | `respect seasonal closures` | toggle | | **on** |

#### `STOPS · LAYOVERS` — summary `3 stops`
Four read-only rows (label mono `var(--m2)` `var(--faint)` left, value mono `var(--m1)` `var(--sec)` right):

| Label | Value |
|---|---|
| `KESS FORD` | `1 d` |
| `THORNWOOD` | `2 d` |
| `LAKEMOUTH` | `1 d` |
| `AUTO FIELDS` | `display auto · resolved` |

### Stage list

Heading, `padding: 10px var(--pad) 4px`, mono `var(--m2)`, letter-spacing `.2em`, `var(--faint)`:
> **`STAGES · TAP FOR OVERRIDES`**

Six rows, each `border-bottom: 1px solid var(--div)`. Row `min-height:var(--row); gap:8px; padding:6px var(--pad)`; selected `background: var(--wash)`, name `var(--acc)`.
Layout: status dot · two-line name block (name, then `sub` mono `var(--m2)` `var(--faint)`) · spacer · days (mono `var(--m1)`).

| i | Name | `terr` | km | flags |
|---|---|---|---|---|
| 0 | `Vhal Serai → Kess Ford` | `plains · steppe` | 118 | |
| 1 | `Kess Ford → Thornwood` | `forest · temperate` | 92 | |
| 2 | `Thornwood → High Saddle` | `mountain · alpine` | 76 | **`closed`** |
| 3 | `High Saddle → Grey Vale` | `mountain · alpine` | 64 | |
| 4 | `Grey Vale → Lakemouth` | `hills · steppe` | 88 | |
| 5 | `Lakemouth → Port Amre` | `river · water leg` | 102 | **`water`** |

- `sub` = `` `${terr} · ${fmtKm(km)}` ``.
- **Blocked** = `season === 'winter' && closures === true`, applied only to stages carrying `closed` — i.e. stage 2 at defaults.
- Dot: blocked → **`✕`** `var(--block)`; water → **`≈`** `var(--water)`; else **`●`** `var(--faint)`.
- Days: blocked → **`—`**; else `(km / 24).toFixed(1) + ' d'` (`var(--sec)`) — a fixed **24 km/day**.

Tapping a stage toggles selection (`pl.sel`, default `-1`; tapping the selected one deselects).

### Stage override panel (expanded on selection)

`padding: 2px var(--pad) 12px; gap: 5px`.

**Note line**, mono `var(--m2)`, line-height 1.6:
- blocked (`var(--block)`): `BLOCKED — High Saddle is closed by seasonal closures in winter`
- otherwise (`var(--faint)`): `` `${n} override${n===1?'':'s'} · ${fmtKm(km)} · arrives day ${6 + i*5}` ``

**Fifteen override rows**, each `display:flex; gap:8px; min-height: calc(var(--ctl) + 2px)`: label (`flex:1`) then one chip (`min-height:var(--ctl); padding:2px 11px; border-radius:999px; font:var(--m1) mono; border:1px solid`).

Order and labels, verbatim:

| # | key | label |
|---|---|---|
| 1 | `mode` | `travel mode` |
| 2 | `group` | `group size` |
| 3 | `cargo` | `cargo` |
| 4 | `pace` | `pace` |
| 5 | `hours` | `hours / day` |
| 6 | `weather` | `weather` |
| 7 | `carry` | `carry food` |
| 8 | `supplies` | `supplies` |
| 9 | `graze` | `grazing` |
| 10 | `forage` | `foraging` |
| 11 | `road` | `road quality` |
| 12 | `infra` | `infrastructure` |
| 13 | `mount` | `mount` |
| 14 | `desert` | `desert water` |
| 15 | `vessel` | `vessel` |

Chip text and styling:

| Condition | Chip text | bg | text colour | border |
|---|---|---|---|---|
| **N/A** — stage has `water` and key ∈ {`mode`,`mount`,`road`,`infra`,`desert`} | `N/A · water leg` (label also `var(--dis)`) | transparent | `var(--dis)` | `var(--div)` |
| **set** | `set · {inherited}` | `var(--wash)` | `var(--acc)` | `var(--acc)` |
| **unset** | `Inherit ({inherited})` | transparent | `var(--dim)` | `var(--div)` |

`inherited` = `river barge` for `vessel`; `p.mode` for `mode`; otherwise `String(p[key])` (which stringifies booleans as `true`/`false`), falling back to `auto`.

> **Design gap:** `hOvToggle` only flips a boolean flag. There is no editor for the override's *value* — a "set" override still displays the inherited value. This is a placeholder, not a finished control.

Two action chips, `padding-top:4px; gap:5px`, `padding:3px 11px; border-radius:8px; background:var(--ins); color:var(--sec); font:var(--m1) mono`:
- **`clear all`** — empties this stage's overrides; toast `Overrides cleared — stage inherits the party form`
- **`copy to land stages`** — copies this stage's override set to every non-water stage; toast `Copied to all land stages`

**Section footnote**, `padding: 11px var(--pad)`, mono `var(--m2)`, line-height 1.7, `var(--dis)`:
> `blank = inherit from the party form · N/A disabled with reason · set = accent border`

> The verdict card, cost/load/supply cards and the `+4 SUPPLY DAYS` / `HEAVY FORAGING` / `DEPART IN AUTUMN` / `IGNORE CLOSURES` action buttons are in the **right dock** (`JOURNEY — RESULTS`) and toolbar, not the left dock.

---

## 7. CARTO — Layers & style, and Terrain appearance

One dock for all four CARTO rail nodes (§3, point 2). Three sections plus a popover.

State from `CAD()` (line 1524): `sel:'terrain'`, `search:''`, `ramp:'Earth'`, `domain:'World'`, `preset:'Atlas'`, `edited:false`, `rampOpen:false`, `az:315`, `el:45`, `strength:0.62`, `multi:true`, `selStop:2`.

### 7.1 Layer search

`padding: 10px var(--pad); border-bottom: 1px solid var(--div)`.
A text input, `width:100%; box-sizing:border-box; background:var(--ins); border:none; border-radius:8px; min-height:var(--ctl); padding:4px 11px; color:var(--ink); font:var(--m1) mono; outline:none`.
Placeholder: **`search layers`**. Filters the list below by case-insensitive substring on the layer label. Does **not** set `edited`.

### 7.2 Layer list

`border-bottom: 1px solid var(--div)`. Each row: `min-height:var(--row); display:flex; align-items:center; gap:9px; padding:2px var(--pad); padding-left:{ind}px`.
`ind = 14 + (child ? 16 : 0)` → 14px for top-level, **30px for children**.

Row contents: visibility dot (`9×9px; border-radius:50%; border:1px solid var(--bor)`, fill `var(--acc)` when visible / `transparent` when hidden — click `hCaVis`, `stopPropagation`) · label · spacer · a mini opacity bar (`36×3px; border-radius:2px; background:var(--ins)`, fill `var(--dim)` at `round(op×100)%`).

Row colour: selected → `var(--acc)`; else visible → `var(--body)`; hidden → `var(--dis)`. Selected row background `var(--wash)`.

Twelve layers, top to bottom (paint order is bottom-up):

| # | id | Label (verbatim) | Visible | Opacity | Indent |
|---|---|---|---|---|---|
| 1 | `labels` | `Labels & annotation` | ✓ | 1.00 | — |
| 2 | `settlements` | `Settlements` | ✓ | 1.00 | — |
| 3 | `ways` | `Ways & routes` | ✓ | 0.90 | — |
| 4 | `political` | `Political` | ✗ | 0.60 | — |
| 5 | `water` | `Water` | ✓ | 1.00 | — |
| 6 | `veg` | `Vegetation` | ✓ | 0.80 | — |
| 7 | `terrain` | `Terrain` | ✓ | 0.78 | — |
| 8 | `hand` | `Hand-drawn hillshade` | ✗ | 0.50 | **child** |
| 9 | `hillshade` | `Hillshade` | ✓ | 0.62 | **child** |
| 10 | `relief` | `Colour relief` | ✓ | 1.00 | **child** |
| 11 | `land` | `Land` | ✓ | 1.00 | — |
| 12 | `bg` | `Background` | ✓ | 1.00 | — |

Default selection: **`terrain`** (row 7). If the selected id is missing, the fallback is `layers[6]` — also `terrain`.

`hCaSel` selects (no `edited`). `hCaVis` toggles visibility **and sets `edited: true`**, which flips the status bar to `style edited — layers differ from preset Atlas`.

> The layer tree is flat data with a boolean `ind` flag — there is no parent/child model, no collapse, and no drag-reorder.

### 7.3 Selected-layer properties

`padding: 11px var(--pad); display:flex; flex-direction:column; gap:9px; position:relative` (relative for the ramp popover).

**Title row**: layer label **uppercased** (mono `var(--m2)`, letter-spacing `.2em`, `var(--acc)`) · spacer · viz descriptor (mono `var(--m2)`, `var(--faint)`).
Viz descriptor: `colour relief` for `relief`; `hillshade · multidirectional` for `hillshade`; **`raster`** for all other ten.

**`opacity`** — label column 96px, slider (continuous 0–1, value = `+p.toFixed(2)`), readout **36px**, right-aligned, mono `var(--m1)`, `var(--ink)`. Readout is the **integer percentage with no `%` sign** (`Math.round(op × 100)` → `78`). Sets `edited: true`.

**Heading `FILL`** (`padding-top: 2px`).

**Ramp button** — `min-height:var(--ctl); border-radius:8px; background:var(--ins); display:flex; gap:9px; padding:3px 9px`:
gradient swatch (`flex:1; height:12px; border-radius:6px`, painted with the ramp's CSS gradient) · ramp name (mono `var(--m1)`, `var(--ink)`) · **`▾`** (`var(--faint)`). Click toggles the popover.

**`domain` row** — label 96px, segmented `World` `View` `Abs` (default **`World`**), then a spacer, then a **hard-coded static readout** (mono `var(--m2)`, `var(--faint)`):
> `−410 → 4 210 m`

(Note the U+2212 minus and the narrow-space thousands separator; this string is literal in the markup and does not respond to the domain segment or to the ramp stops.) The segment sets `edited: true`.

**Heading `LIGHT`** (`padding-top: 2px`). Three sliders, label 96px, readout **44px**:

| key | label | min | max | step | default | readout |
|---|---|---|---|---|---|---|
| `az` | `azimuth` | 0 | 360 | 5 | **315** | `315°` |
| `el` | `elevation` | 0 | 90 | 1 | **45** | `45°` |
| `strength` | `strength` | 0 | 1 | 0.01 | **0.62** | `0.62` (no unit) |

Readout rule: `step >= 1` → `Math.round(v) + unit`; else `v.toFixed(2)`. All three set `edited: true`.

**Toggle** — `multidirectional · 8 lights`, default **on**. Sets `edited: true`.

**Footnote**, mono `var(--m2)`, line-height 1.6, `var(--dis)`:
> `presentation only — nothing here alters world data or marks a stage stale`

### 7.4 Ramp popover

```
position:absolute; top:0; left:8px; right:8px; z-index:40;
background: var(--pan); border: 1px solid var(--bor); box-shadow: var(--shadow);
padding: 6px; display:flex; flex-direction:column; gap:4px;
data-menupop="1"
```
Nine rows, `min-height:var(--ctl); border-radius:8px; display:flex; gap:9px; padding:3px 8px`: gradient swatch (`flex:1; height:12px; border-radius:6px`) then the name (mono `var(--m1)`). Selected: `background: var(--wash2)`, `outline: 1px solid var(--acc)`, name `var(--acc)`; else transparent, name `var(--sec)`.

| Name (verbatim) | CSS gradient |
|---|---|
| `Earth` *(default)* | `linear-gradient(90deg,#1d3140,#2e5a4a,#7a8a55,#b9a878,#d8cdb0,#efe9dd)` |
| `Elevation` | `linear-gradient(90deg,#22304a,#2d6a58,#a8a06a,#b06a42,#8a4a3a,#f0ece4)` |
| `Atlas` | `linear-gradient(90deg,#2a3140,#4a6a5a,#9aa571,#c9b789,#e5dcc4)` |
| `Mono` | `linear-gradient(90deg,#14161a,#3a3f44,#7a8188,#b8bdc2,#eceff2)` |
| `Imhof` | `linear-gradient(90deg,#5a6a7a,#8a9a8a,#c2b494,#e0cfa4,#f2e8cf)` |
| `Ice` | `linear-gradient(90deg,#2a4a6a,#5a8aaa,#9ac2d8,#d0e5ef,#f4fafd)` |
| `Dark ice` | `linear-gradient(90deg,#10202f,#26445c,#4a7690,#84aec4,#c6dfec)` |
| `Desert` | `linear-gradient(90deg,#5a4432,#8a6a44,#b8905e,#d8b884,#efdcb4)` |
| `Dark atlas` | `linear-gradient(90deg,#10141a,#243230,#4a5540,#7a744f,#a89a6a)` |

Tenth row: **`create custom ramp…`** — `min-height:var(--ctl); border-radius:8px; justify-content:center; border:1px dashed var(--bor); color:var(--faint); font:var(--m1) mono`. Closes the popover and toasts `Create custom ramp — edit stops in the right dock`.

Picking a ramp closes the popover, sets `edited: true`, and toasts `Ramp — {name}`.

> The stop editor (`RAMP · STOPS`: 5 stops, elevation/hue sliders, `linear`/`ease`/`step` interpolation, add/delete/reverse, `−410 → 4 210 m` domain) lives in the **right dock**. Left-dock builders need only the ramp *name* and its gradient.
> The style presets `Atlas` `Parchment` `Physical` `Ink`, the `preset Atlas` / `custom — edited since preset Atlas` note, `Reset` and `Save preset` are in the **top toolbar** (markup 214–219), not the dock.

### 7.5 `TERRAIN APPEARANCE · MOCK`

`padding: 11px var(--pad); display:flex; flex-direction:column; gap:9px`. **No bottom border** — this is the last block in the CARTO dock.

Four sliders, label column **110px**, readout **40px**. All are 0.00–1.00, value = `+p.toFixed(2)`, fill = `round(v×100)%`, readout = `v.toFixed(2)`.

| key | label | default |
|---|---|---|
| `tint` | `hypsometric tint` | **0.70** |
| `hs` | `hillshade strength` | **0.62** |
| `water` | `water depth cue` | **0.50** |
| `atmo` | `atmosphere haze` | **0.30** |

Stored in `state.rnd`. There are **no stepper buttons** on these four — slider only.

Footnote, mono `var(--m2)`, line-height 1.7, `var(--dis)`:
> `drives the appearance pipeline only — sample counts live in Preferences ▸ Graphics; style presets and ramps live in CARTO`

---

## 8. Domain rail (immediately left of the dock — context, not the dock itself)

Included because it selects which dock body renders, and its node labels are the only surviving statement of the mode names.

`width: var(--railW); flex:none; border-right: 1px solid var(--hair)`. Gated on `state.showRail` (default true; `Window ▸ Domain rail`).

Top: an expand control, `height: var(--tool); display:grid; place-items:center; color:var(--faint)`, glyph `{{ railChev }}` (**UNSPECIFIED**, truncated), handler `hRailExp` → toggles `state.railExp`.
Then three domain cells, each `flex:1; max-height:112px; display:grid; place-items:center`, label vertical (`writing-mode: vertical-rl; transform: rotate(180deg)`, mono `var(--m1)`, letter-spacing `.26em`): **`WORLD`** · **`CIVIL`** · **`CARTO`**.
Active cell: `color: var(--acc)`, `background: var(--wash)`, `box-shadow: inset -2px 0 var(--acc)`. Inactive: `color: var(--dim)`, transparent, no inset.
Then `flex:1` spacer and a vertical foot label `{{ railFoot }}` (**UNSPECIFIED**), mono `var(--m2)`, letter-spacing `.2em`, `var(--faint)`.

**Expanded node list** (`railExp`): a second 200px column, `border-right: 1px solid var(--hair); background: var(--pan); overflow-y:auto; padding: 8px 0`. Headings `padding:10px 14px 3px`, mono `var(--m2)`, letter-spacing `.2em`, `var(--faint)`. Nodes `min-height:var(--row); padding:2px 14px`, hover `background: var(--wash)`; `var(--acc)` when active, else `var(--body)`.

| Heading | Node label | `dom` | `mode` |
|---|---|---|---|
| `WORLD` | `Generation pipeline` | WORLD | `a` |
| | `Sculpt` | WORLD | `b` |
| `CIVIL` | `Landmarks` | CIVIL | `landmarks` |
| | `Factions & settlements` | CIVIL | `factions` |
| | `Ways & routes` | CIVIL | `infra` |
| | `Journey planner` | CIVIL | `planner` |
| `CARTO` | `Layers & style` | CARTO | *(empty)* |
| | `Labels` | CARTO | *(empty)* |
| | `Icons` | CARTO | *(empty)* |
| | `Terrain appearance` | CARTO | *(empty)* |

Active test: `n.dom === domain && (!n.mode || n.mode === (dom === 'WORLD' ? worldMode : civCat))`. Because the four CARTO nodes have an empty mode, **all four highlight together.**

`setDomain(d)` also sets `railExp: false`, collapsing the node list on any domain change. `Window ▸ Workspaces` offers `World` / `Civilization` / `Cartography`, calling the same function.

---

## 9. `UNSPECIFIED:` — what a builder cannot get from this file

All of §9.1 is a consequence of the 256 KiB truncation described in §0 and is recoverable from an intact copy. §9.2 is not.

### 9.1 Lost to truncation (all inside `valsCore()`)

| Missing binding | What is missing | Consequence |
|---|---|---|
| `ldTitle` | The dock header title string, per domain and mode | **No dock has a title.** 6+ distinct strings unknown. |
| `ldSwitch` | The condition that shows the mode-switch pill | Assumed WORLD-only; unconfirmed |
| `ldSwA`, `ldSwB` (+ `…Col`, `…Bg`) | The two WORLD mode-switch segment labels and their state colours | The switch cannot be labelled. Rail node names `Generation pipeline` / `Sculpt` are the closest evidence but are 19 and 6 characters — almost certainly not the pill text. |
| `ldPipe` | The condition that shows the Generation pipeline body | Inferred `domain==='WORLD' && worldMode==='a'` from `ldSculpt`'s complement |
| `ldClosed` | The condition for the collapsed strip | Inferred `!ldOpen` |
| `ldCollapsedLabel` | The vertical text on the collapsed strip | Unknown; may or may not vary by domain |
| `hLdClose`, `hLdOpen` | The two collapse handlers | Effect is clear (`ldOpen` flips) but not the exact call |
| `hRailNode` | What clicking an expanded rail node does | Must set `domain` from `data-dom` and `worldMode`/`civCat` from `data-mode`, but the CARTO nodes' empty `mode` means the intended behaviour for `Labels`, `Icons`, `Terrain appearance` is genuinely undetermined |
| `hDomain`, `hRailExp`, `railChev`, `railFoot` | Domain-rail handlers and its two glyph/label strings | Rail chevron glyph and foot label unknown |
| `hTool` | The tool-button click handler | Effect specified via `armTool()` |
| `hStageOpen`, `hStep`, `hSlide`, `hSegField`, `hTogField` | The five pipeline field handlers | Effects specified via `setField(n,k,v)` and `fieldMeta(n,k)`; the wiring (clamp, quantise, single-open enforcement) is not literally present |
| `hRunOne`, `hRunFrom` | The two run buttons | Effects specified via `runStages(from,to)`; that `hRunOne` = `runStages(n,n)` and `hRunFrom` = `runStages(n,10)` is inference from the labels |
| `genLog`, `genLogLines` | Whether/when the log block shows, and its line shape | `state.log` entries are `{t, col}` and capped at 3 — that much is specified |
| `hFinalize`, `bakeLabel`, `bakeBg`, `bakeCol` | The Finalize button's label, colours and action | The two-state nature (`finalized` true/false) is certain from `armTool`, `setField`, `runStages` and the sculpt footnote, but **neither label string exists** |
| `globalTools`/`domainTools` `key` values as rendered | The key hint strings are in `valsCore` line 2074-5 and **are** present | — no gap |

### 9.2 Genuinely unspecified by the design

1. **`window.LM_GLYPHS` is not in the repository.** The landmark type rows and the map marks both read `window.LM_GLYPHS[typeName].inner` / `.ds`, loaded from `<script src="./landmark-glyphs.js">`. That file does not exist beside the `.dc.html`. **47 type icons have no artwork.** The row degrades to an empty `<span>` — it does not fall back to the family glyph.
2. **`./support.js` is likewise absent** — the `sc-for` / `sc-if` / `{{ }}` runtime. Not needed to build GDScript, but the prototype cannot be run to check anything against.
3. **No scroll-position memory.** Switching domain or category re-renders the body with no `scrollTop` restoration. Whether a builder should preserve it is not stated.
4. **No dock resize.** `--ldW` is a token with three fixed values. There is no drag handle, no min/max width, no persisted user width.
5. **No empty/error state for the pipeline, sculpt or CARTO bodies.** Only CIVIL's settlements and ways sections have empty-state copy.
6. **The override chips have no value editor** (§6d). `hOvToggle` sets a boolean only; a "set" override displays the inherited value. What a set override should let you *enter* is not designed.
7. **Five CIVIL keyboard shortcuts are advertised but unbound** (§2.4): `S`, `P`, `T`, `W`, `⇧R`. `R` is taken by `region`.
8. **`Labels` and `Icons` (CARTO) have no left-dock content**, yet they are first-class rail nodes. Whether this is intentional (their controls being toolbar-resident) or an omission is not stated anywhere.
9. **The `−410 → 4 210 m` domain readout is hard-coded** in the markup and does not derive from the ramp stops (which do span −410…4210) or respond to the `World`/`View`/`Abs` segment. What `View` and `Abs` should display is undefined.
10. **The dock has no footer.** If a persistent footer (apply/revert, status) is wanted, none is drawn.
11. **`7 stages` vs 6** in the Ways ▸ Routes row (§6c).
12. **The stage dependency graph is cyclic** (06 needs 08, 08 needs 06) — §4.3. `Run 06 → 10` is well-defined; `Run stage 06` alone against a stale 08 is not.
13. **Sculpt `Sculpt` pill has no key hint** while `Freehand` and `Biome paint` do (§2.4). Deliberate or omitted is not stated.
14. **`state.infraMode: 'a'`** is initialised and never read. Dead state; if it was meant to drive a WAYS/ROUTES sub-switch, that switch was not drawn.
15. **`civCats` (line 1300) is computed and never rendered** (§3, point 4) — the headers are hand-written. Harmless, but it means the four category rows are not data-driven and a builder should not assume they are extensible.
16. **`state.params.rdens2: 0`** is a default with no field (§4.3). Dead.

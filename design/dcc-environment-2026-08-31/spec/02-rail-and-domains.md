# Domain Rail & Node Tree — Desktop Prototype Specification

**Source:** `C:\Users\Vincent\Cartalith_GDT\design\dcc-environment-2026-08-31\Cartalith DCC Environment.dc.html`
**Logic class:** `class Component extends DCLogic`, script block opens at line 1163.
**Rail markup:** lines 282–303. **Rail data:** `valsCore()`, lines 2060–2071.

---

## 0. CRITICAL — the source file is truncated

The desktop `.dc.html` is **exactly 262 144 bytes (256 KiB)** and ends mid-token inside `valsCore()`, at line 2105:

```
      measRows.push({i:('0'+i).slice(-2),len:this.fmtKm(km),be
```

`valsCore()` never closes and **never returns**. Every binding the rail markup consumes that was defined in the returned object is therefore absent from the file. `renderVals()` — the method that merges `valsCore()`, `vals5()`, `vals6()` etc. — does not exist in the file either.

Bindings referenced by rail markup but **not defined anywhere in the delivered file** (each occurs exactly once, in markup only):

| Binding | Where used | Status |
|---|---|---|
| `domains` | `<sc-for list="{{ domains }}">`, line 285 | value missing — the array `doms` is built at line 2067 but never assigned to a key |
| `railNodes` | `<sc-for list="{{ railNodes }}">`, line 295 | array built at line 2070, never assigned to a key |
| `hDomain` | domain cell `onClick`, line 286 | handler missing |
| `hRailNode` | node row `onClick`, line 297 | handler missing |
| `hRailExp` | chevron `onClick`, line 284 | handler missing |
| `railChev` | chevron glyph, line 284 | glyph string missing |
| `railFoot` | rail footer text, line 291 | string missing |
| `railExp` | `<sc-if value="{{ railExp }}">`, line 293 | reads `state.railExp` (exists, line 1199); the vals key that exposes it is missing |
| `showRail` | `<sc-if value="{{ showRail }}">`, line 282 | reads `state.showRail` (exists, line 1199); vals key missing |
| `ldPipe`, `ldTitle`, `ldSwitch`, `ldSwA`, `ldSwB`, `ldSwACol`, `ldSwABg`, `ldSwBCol`, `ldSwBBg`, `hLdMode` | left dock, lines 306–330 | all missing — these are the mode→dock selectors |

Everything below is either (a) directly present in the file, or (b) marked `UNSPECIFIED:` where it is not. No inference is presented as fact.

Grep confirms no other file in `design/` defines `railFoot`, so there is no sibling source to recover these from.

---

## 1. Naming collision — two things are called "rail"

The builder **must not** conflate these.

| | Vertical domain rail (this spec) | Horizontal contextual tool rail |
|---|---|---|
| Markup | lines 282–303 | lines 108–146 |
| Gate | `{{ showRail }}` (state `showRail`) | `{{ railShow }}` (computed in `vals5()`, line 1391) |
| Bindings | `railNodes`, `railExp`, `railChev`, `railFoot`, `railW` | `railGroups`, `railMeasure`, `railBrush`, `railSizePct`, `railHardPct`, `hRailGroup` |
| Position | left edge of the main content row | a full-width `--tbH` bar above the domain toolbar |

`railShow` = `s.scr==='app' && (tool is sculpt/freehand/biome, or tool is measure)`. Unrelated to the domain rail.

---

## 2. The domain set

```js
const doms = ['WORLD','CIVIL','CARTO'].map(id => ({
  id,
  on:    s.domain === id,
  col:   s.domain === id ? 'var(--acc)'  : 'var(--dim)',
  bg:    s.domain === id ? 'var(--wash)' : 'transparent',
  inset: s.domain === id ? 'inset -2px 0 var(--acc)' : 'none'
}));
```
*(line 2067)*

Three domains. Displayed string is the id verbatim, already uppercase: `WORLD`, `CIVIL`, `CARTO`.

**Stale-hint evidence of the reduction from five:** the `<sc-for>` at line 285 carries `hint-placeholder-count="5"` while the array has 3 members. The node loop at line 295 carries `hint-placeholder-count="10"` while its array has 13 members (3 headers + 10 nodes).

---

## 3. Complete node tree

Built at line 2069:

```js
const nodes=[]; const nd=(label,dom,mode)=>({t:'n',label,dom,mode:mode||''});
[['WORLD',[nd('Generation pipeline','WORLD','a'),nd('Sculpt','WORLD','b')]],
 ['CIVIL',[nd('Landmarks','CIVIL','landmarks'),nd('Factions & settlements','CIVIL','factions'),nd('Ways & routes','CIVIL','infra'),nd('Journey planner','CIVIL','planner')]],
 ['CARTO',[nd('Layers & style','CARTO'),nd('Labels','CARTO'),nd('Icons','CARTO'),nd('Terrain appearance','CARTO')]]
].forEach(([h,list])=>{nodes.push({t:'h',label:h}); nodes.push(...list)});
```

Flat render order, index = `key`:

| idx | kind | exact label | `dom` | `mode` |
|---:|---|---|---|---|
| 0 | header | `WORLD` | — (undefined → `''`) | — |
| 1 | node | `Generation pipeline` | `WORLD` | `a` |
| 2 | node | `Sculpt` | `WORLD` | `b` |
| 3 | header | `CIVIL` | — | — |
| 4 | node | `Landmarks` | `CIVIL` | `landmarks` |
| 5 | node | `Factions & settlements` | `CIVIL` | `factions` |
| 6 | node | `Ways & routes` | `CIVIL` | `infra` |
| 7 | node | `Journey planner` | `CIVIL` | `planner` |
| 8 | header | `CARTO` | — | — |
| 9 | node | `Layers & style` | `CARTO` | `''` |
| 10 | node | `Labels` | `CARTO` | `''` |
| 11 | node | `Icons` | `CARTO` | `''` |
| 12 | node | `Terrain appearance` | `CARTO` | `''` |

Labels contain a literal ampersand (`&`, not `&amp;`) — verified against the raw bytes. Sentence case for nodes; ALL-CAPS only because the header labels are the domain ids themselves.

### 3a. The four CARTO nodes carry no mode

`nd('Layers & style','CARTO')` passes no third argument, so `mode` is `''` for all four. Consequences, both provable from the delivered code:

1. **They are not distinguishable in selection state.** The highlight test (§5) is `!n.mode || …`, so with `mode===''` the guard short-circuits true. **All four CARTO nodes render accent simultaneously** the moment `domain==='CARTO'`.
2. **They cannot select different dock content.** The two CARTO dock blocks are gated on domain alone (`ldCarto` and `ldRender`, line 1566), not on any per-node mode. Clicking `Labels` versus `Icons` has no state to write.

`UNSPECIFIED:` what `Layers & style`, `Labels`, `Icons` and `Terrain appearance` are each supposed to select. The design gives four rows and one destination. A builder needs either four distinct dock panels or a ruling that these four collapse to one.

---

## 4. What each mode DOES — mode → left-dock content

The left dock (`--ldW`, lines 304–811) renders, top to bottom: a title row, an optional a/b segmented switch, an always-present `TOOLS` block, then a scrolling body of mutually-gated blocks.

| Order | `sc-if` gate | Line | Gate expression (where it survives) | Content shown |
|---:|---|---:|---|---|
| — | *(none)* | 318 | always | `TOOLS` — `globalTools` (4 icon buttons) + divider + `domainTools` |
| 1 | `ldPipe` | 330 | **missing** (truncated) | Ten-stage generation pipeline accordion, `stages` from `this.STG` |
| 2 | `ldSculpt` | 394 | `s.domain==='WORLD' && s.worldMode==='b'` (line 1757) | `GEOLOGICAL FEATURE` + sculpt presets/brush params |
| 3 | `ldCarto` | 490 | `s.domain==='CARTO'` (line 1566) | layer search input, `caLayers` tree, `caDomains`, `caLight`, ramp editor |
| 4 | `ldCivilDock` | 555 | `s.domain==='CIVIL'` (line 1300) | accordion header `LANDMARKS` + `{{ lmCatCount }}` |
| 5 | `ldLandmarks` | 561 | `s.domain==='CIVIL' && cc()==='landmarks'` (line 1307) | `PLACEMENT`, crowding, radii, class chips, `famGroups`, funnel |
| 6 | `ldCivilDock` | 651 | as above | accordion header `FACTIONS & SETTLEMENTS` |
| 7 | `ldCivil` | 656 | `s.domain==='CIVIL' && cc()==='factions'` (line 1566) | `FACTIONS` list, `civPlaces` |
| 8 | `ldRender` | 685 | `s.domain==='CARTO'` (line 1566) | `TERRAIN APPEARANCE · MOCK`, `rndParams` |
| 9 | `ldCivilDock` | 698 | as above | accordion header `WAYS & ROUTES` |
| 10 | `ldRoutes` | 703 | `s.domain==='CIVIL' && cc()==='infra'` (line 1477) | `WAYS · {{ wayCount }}`, `wayRows` |
| 11 | `ldCivilDock` | 728 | as above | accordion header `JOURNEY PLANNER` |
| 12 | `ldPlanner` | 733 | `s.domain==='CIVIL' && cc()==='planner'` (line 1477) | `ELDRA ROUTE · VHAL SERAI → PORT AMRE`, `planGroups`, `planStages` |

Reading the table against the node tree:

| Domain | mode | State field written | Dock block selected |
|---|---|---|---|
| WORLD | `a` | `state.worldMode = 'a'` | block 1 (`ldPipe`) |
| WORLD | `b` | `state.worldMode = 'b'` | block 2 (`ldSculpt`) |
| CIVIL | `landmarks` | `state.civCat = 'landmarks'` | blocks 4–12 headers + block 5 body |
| CIVIL | `factions` | `state.civCat = 'factions'` | headers + block 7 body |
| CIVIL | `infra` | `state.civCat = 'infra'` | headers + block 10 body |
| CIVIL | `planner` | `state.civCat = 'planner'` | headers + block 12 body |
| CARTO | `''` ×4 | nothing | blocks 3 **and** 8, both, always |

`UNSPECIFIED:` `ldPipe`'s definition. Every other `ld*` gate survives; this one was in the truncated return. All surviving evidence (state field `worldMode:'a'` at line 1199, `nd('Generation pipeline','WORLD','a')`, and `ldSculpt` being the exact complement) points to `s.domain==='WORLD' && s.worldMode==='a'`, but the file does not say so.

### 4a. The two mode-writing accessors that DO survive

**CIVIL mode** — `state.civCat`, defaulted, never null:
```js
cc(){ return this.state.civCat || 'landmarks' }        // line 1211
setCc(v,cb){ this.setState({civCat:v}, cb) }           // line 1212
```
Initial state does **not** contain `civCat` (line 1199 lists `domain`, `worldMode`, `infraMode`, `railExp`, `showRail`, `showSB` — no `civCat`), so CIVIL opens on `landmarks` by fallback.

**WORLD mode** — `state.worldMode`, initial `'a'` (line 1199).

### 4b. The rail's node click is duplicated by an in-dock accordion

The four CIVIL categories are *also* clickable headers inside the left dock (lines 556, 652, 699, 729), each `onClick="{{ hCivCat }}" data-id="…"`. That handler survives (line 1301):

```js
hCivCat: e => { const id = e.currentTarget.dataset.id;
  this.setCc(this.cc()===id && id!=='landmarks' ? 'landmarks' : id) }
```

Toggle rule: clicking the already-open category **falls back to `landmarks`**, except `landmarks` itself, which is sticky. Header ids map exactly to the CIVIL node modes: `landmarks`, `factions`, `infra`, `planner`.

A parallel `civCats` array is built at line 1300 from
`[['landmarks','LANDMARKS'],['factions','FACTIONS & SETTLEMENTS'],['infra','WAYS & ROUTES'],['planner','JOURNEY PLANNER']]`
and is **never consumed by the markup** (the four headers are hard-coded). It is dead data, but it is the authoritative uppercase label list for the dock headers.

`UNSPECIFIED:` whether `hRailNode` applies the same toggle-back-to-landmarks rule, or is a plain set. The rail node rows have no visible "collapse" affordance, which argues for a plain set, but the file does not say.

### 4c. Other writers of domain / mode

| Source | Line | Effect |
|---|---:|---|
| `setDomain(d)` | 2054 | `{domain:d, railExp:false}`, then `this.dirty=true` |
| `act('dom:X')` | 2027 | `this.setDomain(a.slice(4))`, then `{menu:null,sub:null}` |
| `act('lmjump:<fam>')` | 2028 | `{domain:'CIVIL', civCat:'landmarks', menu:null, sub:null}` + `setLm({openFam:…})` |
| `armTool('sculpt'\|'freehand')` | 1941 | patch includes `domain:'WORLD', worldMode:'b'` — **arming a sculpt tool re-navigates the rail** |
| `hOpenPlanner` | 1480 | `{civCat:'planner'}` — changes CIVIL mode without touching `domain` |
| `resetlayout` | 2052 | `{ldOpen:true, rdOpen:s.frame!=='tabP', showRail:true, showSB:true, railExp:false, menu:null, sub:null}` |

`Window` menu (line 2016) offers a third route, under the header `WORKSPACES`:

| Menu item label | action | resolves to |
|---|---|---|
| `World` | `dom:WORLD` | `setDomain('WORLD')` |
| `Civilization` | `dom:CIVIL` | `setDomain('CIVIL')` |
| `Cartography` | `dom:CARTO` | `setDomain('CARTO')` |

Note the menu labels differ from the rail labels: menu says `Civilization` / `Cartography`, rail says `CIVIL` / `CARTO`.

**No keyboard shortcut switches domain.** The full keymap (line 1832) is `{v:'inspect', m:'measure', r:'region', b:'biome', l:'label', i:'icon', f:'freehand'}` plus `this.LAYERS` keys `1`–`8`, plus `⌘Z`/`⌘⇧Z` and `Escape`.

### 4d. The domain also drives the tool palette and the key hint line

`domToolDefs`, line 2073 — `[id, label, key]`:

| Domain | Tools |
|---|---|
| `WORLD` | `sculpt` `Sculpt` *(no key)* · `freehand` `Freehand` `F` · `biome` `Biome paint` `B` |
| `CIVIL` | `settlement` `Settlement` `S` · `poi` `POI` `P` · `territory` `Territory` `T` · `way` `Way` `W` · `route` `Route` `⇧R` |
| `CARTO` | `label` `Label` `L` · `icon` `Icon` `I` |

Global tools, always present (line 2072), tooltip strings verbatim: `Inspect · V`, `Measure · M`, `Region select · R`, `Pan / zoom — always available` (the pan button is permanently `bg:var(--ins)` / `col:var(--dis)` — it is a legend, not a button).

Status-bar key hint (line 2101):
```js
keyHints = touch ? 'long-press = sample · pinch zooms'
  : 'V M R ' + (s.domain==='WORLD' ? 'B F' : s.domain==='CARTO' ? 'L I' : 'S P T W') + ' · ⌘Z · Esc';
```
Note the CIVIL branch omits `⇧R` even though `Route` declares it.

---

## 5. Geometry

The rail is the first child of the main content row `<div style="flex:1;display:flex;min-height:0">` (line 281), i.e. it spans from the bottom of the toolbar stack to the top of the timeline strip / status bar, and sits **outside and left of** the left dock.

### 5a. Rail column — line 283

```
width: var(--railW);
flex: none;
border-right: 1px solid var(--hair);
display: flex; flex-direction: column; align-items: stretch;
```
No background is set — it shows `--sur`, unlike the left dock, which sets `background:var(--pan)`.

### 5b. Children, in order

| # | Element | Exact style | Content |
|---:|---|---|---|
| 1 | collapse chevron | `height:var(--tool); display:grid; place-items:center; cursor:pointer; color:var(--faint)` | `{{ railChev }}` |
| 2 | domain cells ×3 | `flex:1; max-height:112px; display:grid; place-items:center; cursor:pointer; color:{{d.col}}; background:{{d.bg}}; box-shadow:{{d.inset}}` | vertical label span |
| 2a | label span | `writing-mode:vertical-rl; transform:rotate(180deg); font:var(--m1) 'IBM Plex Mono',monospace; letter-spacing:.26em` | `{{ d.id }}` |
| 3 | spacer | `flex:1` (a bare `<span>`) | — |
| 4 | footer wrapper | `padding:10px 0; display:grid; place-items:center` | — |
| 4a | footer span | `writing-mode:vertical-rl; transform:rotate(180deg); font:var(--m2) 'IBM Plex Mono',monospace; letter-spacing:.2em; color:var(--faint)` | `{{ railFoot }}` |

**Height distribution.** Four siblings carry `flex:1` (= `1 1 0%`): the three domain cells and the spacer. The domain cells are capped at `max-height:112px`; the spacer is not. On any realistic desktop height each domain cell therefore settles at exactly **112 px** and the spacer absorbs the remainder. Worked example, `w1920` (1920×1080), status bar on, no timeline, no horizontal tool rail:

```
1080 − 36 (--menuH) − 40 (--tbH) − 26 (--sbH) = 978 px available
  − 30 (chevron, --tool)
  − 336 (3 × 112)
  − footer (20 px padding + vertical text length)
  = spacer height
```

`writing-mode:vertical-rl` + `rotate(180deg)` = text reads **bottom-to-top**.

### 5c. Expanded node panel — line 294

Rendered only inside `<sc-if value="{{ railExp }}">`, as a **separate sibling column to the right of the rail**, not an overlay:

```
width: 200px;        ← literal, not a token; does not scale with touch density
flex: none;
border-right: 1px solid var(--hair);
background: var(--pan);
overflow-y: auto;
padding: 8px 0;
```

**Header row** (`n.isHead`), line 296:
```
padding: 10px 14px 3px;
font: var(--m2) 'IBM Plex Mono', monospace;
letter-spacing: .2em;
color: var(--faint);          ← HARD-CODED. n.col is computed for headers and then ignored.
```

**Node row** (`n.isNode`), line 297:
```
min-height: var(--row);
display: flex; align-items: center;
padding: 2px 14px;
cursor: pointer;
color: {{ n.col }};
background: {{ n.bg }};        ← always 'transparent' (see §6)
style-hover: background: var(--wash);
data-dom="{{ n.dom }}" data-mode="{{ n.mode }}"
```
No `font` is declared on the node row, so it inherits the shell body font: `--fs` at `line-height:1.45` in `'Helvetica Neue', Helvetica, Arial, sans-serif` (set on the outer page wrapper, line 11). **Node labels are sans-serif; domain labels and section headers are IBM Plex Mono.** No indent distinguishes a node from a header — both sit at `14px` left padding; the header's extra `10px` top padding is the only separation.

### 5d. Header vs node — the exact visual difference

| | Header | Node |
|---|---|---|
| Font family | IBM Plex Mono | Helvetica Neue / Helvetica / Arial / sans-serif |
| Font size | `var(--m2)` = 9 px (11 px touch) | `var(--fs)` = 11.5 px (14 px touch) |
| Letter-spacing | `.2em` | normal |
| Colour | `var(--faint)`, fixed | `var(--acc)` when selected, else `var(--body)` |
| Case | ALL CAPS (source literal) | Sentence case (source literal) |
| Box | `padding:10px 14px 3px`, auto height | `padding:2px 14px`, `min-height:var(--row)` |
| Interaction | none | `cursor:pointer`, hover `background:var(--wash)` |

### 5e. Tokens

Desktop / dark baseline, line 25:

| Token | Value | Token | Value |
|---|---|---|---|
| `--railW` | `40px` | `--tool` | `30px` |
| `--row` | `28px` | `--ctl` | `24px` |
| `--fs` | `11.5px` | `--m0` | `10.5px` |
| `--m1` | `10px` | `--m2` | `9px` |
| `--menuH` | `36px` | `--tbH` | `40px` |
| `--sbH` | `26px` | `--pad` | `14px` |
| `--ldW` | `372px` | `--rdW` | `304px` |

Touch override (`densStr`, line 2064; applies when `frame` is `tabL` or `tabP` — `isTouch()`, line 1817):
```
--fs:14px; --m0:12.5px; --m1:12px; --m2:11px; --menuH:52px; --tbH:56px;
--railW:48px; --ctl:36px; --btnH:44px; --row:44px; --tool:44px;
--ldW:400px; --rdW:400px; --sbH:36px; --pad:16px; --g:12px; --pop:380px;
```
The `w1366` override changes only `--ldW:330px; --rdW:280px; --pop:280px` — **`--railW` stays 40 px on the 1366 laptop frame**, and the 200 px node panel is unchanged at every frame.

Frames (`frameDef()`, line 1816): `w1920` 1920×1080 · `w1366` 1366×768 · `tabL` 2560×1600 · `tabP` 1600×2560.

### 5f. Colours used by the rail

| Var | Dark (line 25) | Light (`themeStr`, line 2063) |
|---|---|---|
| `--acc` | `#e0a34a` | `#a4650f` |
| `--body` | `#c8cbcd` | `#23241f` |
| `--dim` | `#8d9296` | `#6b6f6a` |
| `--faint` | `#6f7478` | `#8d9088` |
| `--wash` | `rgba(224,163,74,.09)` | `rgba(164,101,15,.09)` |
| `--pan` | `#121314` | `#fbfaf7` |
| `--sur` | `#0d0e0f` | `#f4f2ee` |
| `--hair` | `rgba(255,255,255,.10)` | `rgba(0,0,0,.14)` |

Theme is a runtime toggle: `state.light`, flipped by `act('theme=light'|'theme=dark')` (line 2043) and the `◐` button at line 105.

---

## 6. Selection state — exact rules

### 6a. Active domain — three simultaneous marks

From `doms`, line 2067:

| Property | Active | Inactive |
|---|---|---|
| `color` | `var(--acc)` | `var(--dim)` |
| `background` | `var(--wash)` | `transparent` |
| `box-shadow` | `inset -2px 0 var(--acc)` | `none` |

**The `inset -2px 0` marker.** An inset box-shadow with `offset-x: -2px, offset-y: 0, blur: 0, spread: 0` shifts the shadow's inner hole 2 px to the left, leaving the rightmost 2 px of the cell's padding box painted in `--acc`. Rendered result: **a 2 px full-height accent bar flush against the right edge of the active domain cell**, i.e. against the `1px solid var(--hair)` right border of the rail — the bar sits inside that border, not replacing it. Height = the cell height (112 px at desktop sizes), not the full rail.

Inactive domain text is `--dim` (`#8d9296`), not `--body` — the resting state of the rail is deliberately quieter than dock body text.

### 6b. Active node — text colour only

Line 2070, verbatim:

```js
const railNodes = nodes.map((n,i)=>({
  key:i, isHead:n.t==='h', isNode:n.t==='n',
  label:n.label, dom:n.dom||'', mode:n.mode||'',
  col: n.dom===s.domain && (!n.mode || n.mode===(n.dom==='WORLD' ? s.worldMode : this.cc()))
       ? 'var(--acc)' : 'var(--body)',
  bg: 'transparent'
}));
```

The predicate, in words:

1. **Domain must match** — `n.dom === s.domain`. A CIVIL node is never accent while WORLD is the domain, regardless of `civCat`.
2. **Then either**: the node declares no mode (`!n.mode` — all four CARTO nodes), **or** the node's mode equals the domain's current mode. The mode read is `s.worldMode` for `dom==='WORLD'` and `this.cc()` for everything else.

`bg` is the literal string `'transparent'` for **every** node, always. There is no selected-row fill, no left indicator bar, no bold weight, no chevron on a node row. **Node selection is expressed exclusively as `var(--acc)` text.** The only background a node row ever gets is the hover `var(--wash)`.

Headers get `col` computed too (`n.dom||''` is `''`, so it never matches `s.domain` and always resolves to `var(--body)`), but the header markup hard-codes `color:var(--faint)` and discards it. **Headers never change appearance with selection.**

Exactly-one-selected holds for WORLD (2 modes, mutually exclusive) and CIVIL (4 modes, `cc()` returns exactly one). It does **not** hold for CARTO: all four CARTO nodes are accent together.

---

## 7. Collapse / expand

The rail has two independent collapse mechanisms.

### 7a. `railExp` — the node panel (default collapsed)

- State: `railExp: false` at line 1199. **The prototype's default is the collapsed rail** — the 40 px strip only.
- Toggle affordance: the chevron cell at the very top of the rail, `height:var(--tool)`, `color:var(--faint)`, `onClick="{{ hRailExp }}"`, glyph `{{ railChev }}`.
- Collapsed appearance: the 40 px rail alone — chevron, three vertical domain labels, spacer, footer. The 200 px node panel is not in the DOM (`<sc-if>`, not a width transition).
- Expanded appearance: rail **plus** the 200 px panel to its right, pushing the left dock right by 200 px. Total left chrome expanded = 40 + 200 + 372 = **612 px** at `w1920`.
- **Selecting a domain force-collapses the panel.** `setDomain(d)` writes `{domain:d, railExp:false}` (line 2054). So the click that navigates to CARTO also closes the node list — the domain cells are the collapsed rail's own navigation, and the node panel is a transient drill-down.
- `resetlayout` sets `railExp:false` (line 2052).

`UNSPECIFIED:` the `railChev` glyph in both states. Sibling collapse affordances in the same file use `›` for "open me" (left dock collapsed, line 814) and `‹` for "close me" (left dock header, line 307; right dock collapsed, line 1102), and the dock accordions use `▸` rotated `0deg`/`90deg`. Which of these the rail chevron uses, and whether it rotates, is not in the delivered file.

`UNSPECIFIED:` whether `hRailNode` also closes the panel after a node click, the way `setDomain` does. Given `setDomain` collapses and `hRailNode` is missing, this is a genuine coin-flip for the builder.

`UNSPECIFIED:` whether `hDomain` on the **already-active** domain toggles `railExp` open (a common DCC pattern that would make `setDomain`'s `railExp:false` reset coherent) or is a no-op. Not in the file.

### 7b. `showRail` — hide the rail entirely

- State: `showRail: true` at line 1199.
- Toggle: `Window` menu, first section — `this.tog('Domain rail','win:rail',s.showRail)` (line 2015). Menu item text is exactly `Domain rail`; it renders as a pill switch (`togBg: r.on ? 'var(--acc)' : 'var(--ins)'`, `togX: r.on ? 15 : 2`).
- Action: `if(w==='rail'){ this.setState(x=>({showRail:!x.showRail})); return }` (line 2032). Note this branch does **not** close the menu — unlike `dom:` and `mock:`, it leaves `menu` set, so the Window menu stays open while toggling.
- When false, both the rail and (being nested inside the same `<sc-if>`) the node panel are removed from the DOM; the left dock becomes the leftmost element.
- `resetlayout` restores `showRail:true`.
- **There is no keyboard shortcut and no in-rail affordance to re-show a hidden rail** — only the Window menu.

Order of the Window menu's first section, verbatim, for the builder implementing it: `Left dock`, `Right dock`, `Domain rail`, `Status bar`, `Timeline (CIVIL · INFRA)`, separator, `Reset layout`, `Save layout as…`, separator, header `WORKSPACES`, `World`, `Civilization`, `Cartography`.

### 7c. Not to be confused with

Both docks have their own 40 px (`--railW`) collapsed strips that look superficially like the domain rail:

- Left dock collapsed (line 813): `width:var(--railW); border-right:1px solid var(--hair); background:var(--pan); align-items:center; gap:12px; padding:8px 0; cursor:pointer`, chevron `›` in `--faint`, then `{{ ldCollapsedLabel }}` vertical in `var(--m2)`, `letter-spacing:.2em`, `color:var(--dim)`.
- Right dock collapsed (line 1101): same, mirrored (`border-left`), chevron `‹`, `{{ rdCollapsedLabel }}` at `letter-spacing:.18em`, `color:var(--acc)`.

The domain rail differs from both: **no `background:var(--pan)`** (it sits on `--sur`), `align-items:stretch` rather than `center`, no `gap`, no `padding`, and the whole strip is not one click target — the chevron and each domain cell are separate.

---

## 8. `UNSPECIFIED:` — consolidated list

Ordered by how much it blocks a build.

**Blocking — the design gives a row but no destination**

1. **What the four CARTO nodes select.** `Layers & style`, `Labels`, `Icons`, `Terrain appearance` all carry `mode:''` and all four highlight together. The CARTO dock shows `ldCarto` (layer tree) and `ldRender` (`TERRAIN APPEARANCE · MOCK`) unconditionally and together. There is no `Labels` panel and no `Icons` panel anywhere in the left dock. Needed: four panel definitions, or a decision to reduce CARTO to one node.

**Blocking — lost to truncation**

2. **`hRailNode` semantics.** Which state fields it writes (presumably `domain` from `data-dom` and `worldMode`/`civCat` from `data-mode`), whether it toggles back to a default the way `hCivCat` does, and whether it collapses `railExp`.
3. **`hDomain` semantics.** Whether it is exactly `setDomain(d.id)`, and what clicking the already-active domain does.
4. **`hRailExp` semantics.** Presumably `railExp = !railExp`; not stated.
5. **`ldPipe`'s gate expression.** The one `ld*` flag whose definition did not survive.
6. **`ldSwitch` / `ldSwA` / `ldSwB` label strings and gate.** The left dock's a/b segmented control at lines 309–314 is the second route to the WORLD mode; its two button captions and the domains for which it appears are unknown. `data-v="a"` / `data-v="b"` and `hLdMode` are all that survive.
7. **`ldTitle`.** The left dock's own title string per domain/mode.

**Non-blocking but visible**

8. **`railChev`** — the chevron glyph, and whether it changes or rotates between collapsed and expanded.
9. **`railFoot`** — the vertical text at the bottom of the rail. Content, and whether it is static or state-derived, are both unknown. Style is fully specified (§5b row 4a).
10. **Header colour when its domain is active.** `n.col` is computed for header rows and then thrown away by hard-coded `color:var(--faint)`. Whether that discard is intentional or a defect is not recoverable from the file.

**Behaviour the design simply does not address**

11. **Keyboard access.** No shortcut for domain switching, no focus ring, no tab order, no `role`/`aria` on rail or node rows. `title` attributes exist on the tool buttons (line 321) but on nothing in the rail.
12. **Node-panel scroll affordance.** `overflow-y:auto` at 200 px wide with 13 rows; whether a scrollbar is expected to be visible, and its styling, are unstated.
13. **Transition/animation.** No `transition` property anywhere on the rail. Expand/collapse is an instantaneous DOM swap.
14. **Touch behaviour of the 200 px panel.** It does not scale with `densStr` while `--railW` and `--row` do, so at `tabL`/`tabP` the panel holds 44 px rows in a column still 200 px wide. Whether that is intended is unstated.
15. **Disabled / finalized states.** `armTool` refuses to arm editing tools when `state.finalized` (line 1940, toast `'World is finalized — <id> is locked'`), but no rail node or domain is ever disabled or dimmed by `finalized`.

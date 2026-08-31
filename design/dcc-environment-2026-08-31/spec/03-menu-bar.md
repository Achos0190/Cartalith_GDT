# Menu bar — `Cartalith DCC Environment.dc.html` (desktop/tablet prototype)

Source: `C:\Users\Vincent\Cartalith_GDT\design\dcc-environment-2026-08-31\Cartalith DCC Environment.dc.html`
Markup: lines 56–106. Data: `menuRowsFor(id)` lines 1974–2023. Dispatch: `act(a)` lines 2023–2053. Bar values: `valsCore()` lines 2065–2066. Design tokens: line 25 (dark) and line 2063 (light).

> **File integrity — read this first.** The file is truncated at exactly **262144 bytes (2^18)**, mid-statement inside `valsCore()` at line 2105. The `data-dc-script` block is never closed (`grep -c "</script>"` = 2, both from `<head>`/`<helmet>`). The truncated blob is what is committed (`git cat-file -s HEAD:…` = 262144). Everything below line 2105 — including the `vals()` return object that binds `menuRows`, `hMenu`, `hAct`, `hUndo`, `hRedo`, `hTheme`, `undoCol`, `redoCol`, `worldLabel`, `fvars`, `scrShell` — **does not exist in the file.** Those are listed under UNSPECIFIED at the end.

---

## 1. Design tokens used by the bar

Dark is the default (`state.light = false`). Light values come from `themeStr` (line 2063), which redefines only the colour tokens.

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
| `--hair` | `rgba(255,255,255,.10)` | `rgba(0,0,0,.14)` |
| `--div` | `rgba(255,255,255,.07)` | `rgba(0,0,0,.08)` |
| `--bor` | `rgba(255,255,255,.16)` | `rgba(0,0,0,.20)` |
| `--wash` | `rgba(224,163,74,.09)` | `rgba(164,101,15,.09)` |
| `--wash2` | `rgba(224,163,74,.16)` | `rgba(164,101,15,.16)` |
| `--shadow` | `0 14px 34px rgba(0,0,0,.55)` | `0 14px 34px rgba(35,36,31,.16)` |
| `--block` (danger) | `#c96a5a` | `#a03d2e` |

Metric tokens, by frame. `densStr` (line 2064) overrides them; `isTouch()` is true for `tabL` and `tabP` only.

| Token | `w1920` (base) | `w1366` | `tabL` / `tabP` (touch) |
|---|---|---|---|
| `--fs` (bar font-size) | `11.5px` | `11.5px` | `14px` |
| `--m1` (mono small) | `10px` | `10px` | `12px` |
| `--m2` (mono tiny) | `9px` | `9px` | `11px` |
| `--menuH` (bar height) | `36px` | `36px` | `52px` |
| `--ctl` (square buttons) | `24px` | `24px` | `36px` |
| `--row` (menu item min-height) | `28px` | `28px` | `44px` |
| `--pop` (popup width) | `300px` | `280px` | `380px` |

Type families: the artboard inherits `font-family:'Helvetica Neue',Helvetica,Arial,sans-serif` from the page wrapper (line 11) and `line-height:1.45` from the shell (line 26). **Only elements that declare `font:… 'IBM Plex Mono',monospace` are monospace** — the wordmark, all shortcut text, all glyph cells, all section headings, all read-row labels/values, all segmented pills, and all note text. Menu titles and item labels are the sans stack at `--fs`.

---

## 2. The bar container

Line 56:

```
height: var(--menuH);  flex: none;
display: flex;  align-items: center;  gap: 2px;
padding: 0 10px;
border-bottom: 1px solid var(--hair);
position: relative;  z-index: 70;
```

The bar paints **no background of its own** — it sits on the shell's `background:var(--sur)` (line 26). Z-index stack: bar `70`, menu popup `80`, toasts `90`.

Children, left to right:

| # | Element | Exact spec |
|---|---|---|
| 1 | Wordmark `CARTALITH` | `font:var(--m1) 'IBM Plex Mono',monospace; letter-spacing:.18em; color:var(--acc); margin:0 12px 0 4px` |
| 2 | Seven menu titles | one `<div data-menupop="1" style="position:relative">` wrapper each; see §3 |
| 3 | Divider | `width:1px; height:16px; background:var(--div); margin:0 6px` |
| 4 | Undo button | `width/height:var(--ctl); border-radius:8px; background:var(--ins); display:grid; place-items:center; cursor:pointer; color:{{ undoCol }}` — glyph `↶` |
| 5 | Redo button | same box; glyph `↷`; `color:{{ redoCol }}` |
| 6 | Spacer | `<span style="flex:1">` |
| 7 | World readout | `font:var(--m1) 'IBM Plex Mono',monospace; color:var(--dim)`; text = `{{ worldLabel }}` |
| 8 | Theme button | `width/height:var(--ctl); border-radius:8px; background:var(--ins); display:grid; place-items:center; cursor:pointer; color:var(--sec); margin-left:8px` — glyph `◐` |

The bar's `gap:2px` applies between all eight, so the divider sits at 2+6 = 8px clear on each side, and the theme button at 2+8 = 10px after the world readout.

---

## 3. Menu titles

Line 60, one per entry of `menusDef`:

```
padding: 5px 11px;  border-radius: 8px;  cursor: pointer;
color: {{ m.col }};  background: {{ m.bg }};
```

| State | `col` | `bg` |
|---|---|---|
| closed | `var(--sec)` | `transparent` |
| open (`state.menu === id`) | `var(--ink)` | `var(--ins)` |

Order and exact labels (line 2065):

| id | label |
|---|---|
| `file` | `File` |
| `edit` | `Edit` |
| `assets` | `Assets` |
| `data` | `Data` |
| `prefs` | `Preferences` |
| `window` | `Window` |
| `help` | `Help` |

Note the id/label mismatch on `prefs` → `Preferences`.

---

## 4. Popup positioning, opening and closing

**Popup box** (line 62), rendered only when `m.on` (i.e. `state.menu === m.id`):

```
position: absolute;
top: calc(100% + 4px);        /* 4px below the title chip, not the bar */
left: 0;                       /* left-aligned to the title chip's left edge */
width: var(--pop);
max-height: 72vh;  overflow-y: auto;
background: var(--pan);
border: 1px solid var(--bor);
box-shadow: var(--shadow);
padding: 5px 0;
z-index: 80;
```

No border-radius. No open/close animation. The offset parent is the per-title `position:relative` wrapper, so the popup hangs from the **title chip**, whose top is `(menuH − chipHeight)/2` — the popup's top edge is therefore *below* the bar's bottom border by roughly `(menuH − chipH)/2 + 4px`, not flush with the bar.

**Opening:** click the title → `onClick="{{ hMenu }}" data-id="{{ m.id }}"`. `hMenu` itself is in the truncated region (UNSPECIFIED); the only observable contract is that the popup renders iff `state.menu === id`.

**Closing** — three routes, all present in the surviving code:

1. **Escape** (`componentDidMount`, line 1831). First branch of the Esc chain, ahead of `layersOpen`, measure-commit, region-clear and tool-disarm:
   `if(s.menu) this.setState({menu:null, sub:null})`. Only fires while `s.scr === 'app'` and the event target is not an `input`/`textarea`.
2. **Outside pointerdown** (line 1836), registered on `document` in the **capture** phase:
   `if(!e.target.closest('[data-menupop]')) { if(menu||layersOpen) setState({menu:null, sub:null, layersOpen:false}) }`.
   Because the title and its popup share one `data-menupop="1"` wrapper, clicks anywhere inside the open popup do **not** dismiss it.
3. **The clicked action itself**, if it sets `menu:null`. See the table in §6.

**Not present:** hover-to-switch between menus while one is open, arrow-key navigation, Enter/Space activation, Alt-accelerators, right-edge flip or clamp (`left:0` is unconditional).

---

## 5. Popup row types

`menuRowsFor(id)` returns rows tagged `t`; the tail of the function (lines 2018–2023) maps them to render flags and colours. Seven types.

Shared colour rule for **item** rows (`isIt`):

```
col = r.danger ? var(--block) : r.dim ? var(--dis) : var(--body)
```

| Type | Flag | Exact CSS |
|---|---|---|
| **Section heading** | `isHead` | `padding:9px 14px 4px; font:var(--m2) 'IBM Plex Mono',monospace; letter-spacing:.2em; color:var(--faint)` |
| **Separator** | `isSep` | `height:1px; background:var(--div); margin:5px 0` (full popup width, no inset) |
| **Item** | `isIt` | `min-height:var(--row); display:flex; align-items:center; gap:8px; padding:3px 14px; cursor:pointer; color:{{r.col}}; padding-left:{{r.ind}}px` — hover `background:var(--wash)` |
| **Toggle** | `isTog` | `min-height:var(--row); display:flex; align-items:center; gap:8px; padding:3px 14px; cursor:pointer` — hover `background:var(--wash)` |
| **Segmented** | `isSeg` | `display:flex; align-items:center; gap:8px; padding:5px 14px 5px 36px` — **no hover** |
| **Read-only** | `isRead` | `display:flex; align-items:baseline; gap:8px; padding:4px 14px 4px 36px` — **no hover, not clickable** |
| **Note** | `isNote` | `padding:6px 14px; font:var(--m2)/1.6 'IBM Plex Mono',monospace; color:var(--faint); border-top:1px solid var(--div); margin-top:4px` |

### Item row internals (three cells, `gap:8px`)

| Cell | Spec | Content |
|---|---|---|
| glyph | `width:14px; flex:none; color:var(--dim); font:var(--m1) 'IBM Plex Mono',monospace` | `r.glyph`, default `''` — **the 14px cell is always rendered**, so every label aligns whether or not a glyph exists |
| label | `flex:1` | `r.label` |
| shortcut | `font:var(--m1) 'IBM Plex Mono',monospace; color:var(--faint)` | `r.sc`, default `''` |

Indent: `r.ind || 14`. Two values only — **14px** (top level) and **36px** (submenu child / secondary). `padding-left` is written after the `padding` shorthand, so it wins; right padding stays 14px, vertical stays 3px.

Two glyph characters are in use: **`▸`** (has a submenu) and **`⧉`** (opens a window). One row uses **`▾`** (expanded submenu family — see §6.3).

### Toggle row internals

| Cell | Spec |
|---|---|
| label | `flex:1; padding-left:22px` (so the label starts at 14+22 = 36px, matching submenu indent) |
| switch track | `width:30px; height:17px; border-radius:999px; flex:none; position:relative; background:{{r.togBg}}` — `var(--acc)` when on, `var(--ins)` when off |
| switch knob | `position:absolute; top:2px; width:13px; height:13px; border-radius:50%; background:var(--ink); left:{{r.togX}}px` — `15px` when on, `2px` when off |

No transition is declared on the knob.

### Segmented row internals

| Cell | Spec |
|---|---|
| label | `flex:1; color:var(--sec)` (sans, `--fs`) |
| pill group | `display:flex; background:var(--ins); border-radius:999px; padding:2px` |
| each option | `padding:3px 10px; border-radius:999px; cursor:pointer; font:var(--m1) 'IBM Plex Mono',monospace; color:{{o.col}}; background:{{o.bg}}` |

Option state: selected → `color:var(--acc)`, `background:var(--wash2)`; unselected → `color:var(--dim)`, `background:transparent`. Each option's action is `r.act + '=' + optionValue` (e.g. `theme=light`, `autoInt=15 min`, `aa=4×`).

### Read row internals

| Cell | Spec |
|---|---|
| label | `flex:1; color:var(--faint); font:var(--m2) 'IBM Plex Mono',monospace; letter-spacing:.08em` |
| value | `font:var(--m1) 'IBM Plex Mono',monospace; color:var(--sec); text-align:right` |

---

## 6. Menu contents — every row, in order

Rows are listed exactly as `menuRowsFor` pushes them. `act` is the `data-act` string. Toast text for `mock:` actions is the substring after `mock:` with ` (mock)` appended (line 2024).

### 6.1 File

| # | Type | Label | Value / options | Shortcut | Glyph | act |
|---|---|---|---|---|---|---|
| 1 | item | `New world…` | | `⌘N` | | `mock:New world modal — name, seed, extent, working resolution` |
| 2 | item | `Open project…` | | `⌘O` | | `mock:Open project — .zip archive picker` |
| 3 | item | `Recent worlds` | | | `▸` | `sub:recent` |
| — | *(only while `sub==='recent'`, indent 36px)* | | | | | |
| 3a | item | `VHAREN REACH — 129384 · 5 d ago` | | | | `world:VHAREN REACH:129384` |
| 3b | item | `KESSA — 774201 · 3 w ago` | | | | `world:KESSA:774201` |
| 3c | item | `ELDRA — 483920 · 2 d ago` | | | | `world:ELDRA:483920` |
| 4 | sep | | | | | |
| 5 | item | `Save project` | | `⌘S` | | `save` |
| 6 | item | `Save as…` | | `⌘⇧S` | | `mock:Save as — the new path becomes the project path` |
| 7 | toggle | `Autosave` | on = `prefs.autosave` (**default `true`**) | | | `autosave` |
| 8 | segmented | `interval` | `off` · `1 min` · `5 min` · `15 min` — selected = `prefs.autoInt` (**default `5 min`**) | | | `autoInt` |
| 9 | item | `Revert to last save` | | | | `mock:Revert — discards in-memory changes including sculpt drafts` |
| 10 | item | `Close project` | | `⌘W` | | `pick` |
| 11 | sep | | | | | |
| 12 | head | `STORAGE LOCATIONS` | | | | |
| 13 | read | `PROJECTS` | `~/Cartalith/Worlds` | | | |
| 14 | read | `TILE ATLAS` | `~/Cartalith/Cache/atlas` | | | |
| 15 | read | `ASSET PACKS` | `~/Cartalith/Packs` | | | |
| 16 | read | `EXPORTS` | `~/Cartalith/Exports` | | | |
| 17 | item | `Change locations…` | | | | `mock:Change locations — one folder picker per root; moving the atlas root invalidates the cache` |
| 18 | item | `Show project on disk` | | | | `mock:Revealed in the OS file manager` |
| 19 | note | `imports live under Data ▸ Import · asset packs under Assets` | | | | |

State effects:
- `save` → reads the clock, formats `HH:MM` zero-padded, sets `state.savedAt`, toasts `Project saved · HH:MM`. **Menu stays open.**
- `world:NAME:SEED` → sets `world = {name, seed, status:'stages 01–10 resolved'}`, clears `sample` and `measure`, closes the menu (`menu:null, sub:null`), marks canvas dirty, toasts `Opened NAME`.
- `pick` → `scr:'picker'`, closes the menu. (Returns to the world-picker screen.)

### 6.2 Edit

`un = undoStack.length`, `re = redoStack.length`. Initial state: both stacks empty, `sample` null — so **every row except the last three is dim on first open.**

| # | Type | Label | Shortcut | Glyph | act | dim when |
|---|---|---|---|---|---|---|
| 1 | item | `Undo` + (`un` ? ` — ` + `undoStack[un-1].label` : `''`) | `⌘Z` | | `undo` | `un === 0` |
| 2 | item | `Redo` + (`re` ? ` — ` + `redoStack[re-1].label` : `''`) | `⌘⇧Z` | | `redo` | `re === 0` |
| 3 | item | `Undo history` | | `▸` | `sub:hist` | `un === 0` |
| — | *(only while `sub==='hist'`)* — `undoStack.slice().reverse().forEach((e,i) => …)`, indent 36px | | | | | |
| 3a…| item | `e.label` (newest first) | | | `histjump:` + `i` | |
| 4 | sep | | | | | |
| 5 | item | `Cut` | `⌘X` | | `mock:Cut — operates on the current selection` | no `sample` |
| 6 | item | `Copy` | `⌘C` | | `mock:Copy` | no `sample` |
| 7 | item | `Paste` | `⌘V` | | `mock:Paste` | **always** (`dim:true`) |
| 8 | item | `Delete` | `⌫` | | `delsel` | no `sample` |
| 9 | sep | | | | | |
| 10 | item | `Select all` | `⌘A` | | `mock:Select all — scoped to the active layer` | never |
| 11 | item | `Deselect` | `⌘D` | | `desel` | never |
| 12 | item | `Find on map…` | `⌘F` | | `mock:Find — places, labels, factions, routes; result pans the viewport` | never |

State effects:
- `undo`/`redo` → `doUndo()`/`doRedo()` (lines 1821–1824). Empty stack toasts `Nothing to undo` / `Nothing to redo`; otherwise applies `e.before`/`e.after`, moves the entry between stacks, toasts `Undo — LABEL` / `Redo — LABEL`. **Menu stays open.**
- `histjump:i` → `n = i+1`; calls `doUndo()` `n` times in a loop; then closes the menu.
- `delsel` → `sample:null`, closes menu, toasts `Selection deleted`.
- `desel` → `sample:null`, closes menu, **no toast**.
- Undo depth is `prefs.undoDepth` (default `5`), enforced in `pushUndo` (line 1820): `undoStack.slice(-(undoDepth-1))`. Any push clears `redoStack`.
- Observed `pushUndo` labels that can appear in rows 1–3a: `commit way`, `place label`, `stamp icon`, `drop settlement`, `drop POI`, `delete label`, `clear annotation`, `delete settlement`, `add <type> stamp`, `delete stamp`, `discard draft`, `edit stage NN` (two-digit zero-padded).

### 6.3 Assets

| # | Type | Label | Shortcut | Glyph | act |
|---|---|---|---|---|---|
| 1 | item | `Asset library` | `⇧A` | `⧉` | `win:assets` |
| 2 | item | `Sprite sheet slicer` | | `⧉` | `win:slicer` |
| 3 | sep | | | | |
| 4 | item | `Import image…` | | | `mock:Import image — lands in Unassigned imports` |
| 5 | item | `Import asset pack .zip…` | | | `mock:Import pack — loads into the library for editing` |
| 6 | item | `Asset pack` | | `▸` | `sub:pack` |
| 7 | item | `Icon families` | | `▸` | `sub:fam` |
| 8 | item | `Texture sets` | | `▸` | `win:assets` |
| 9 | sep | | | | |
| 10 | item | `Landmark types` | | `▸` | `sub:lmt` |
| 11 | sep | | | | |
| 12 | item | `Apply library to map` | | | `mock:Library compiled and loaded as the live pack` |
| 13 | item | `Clear library…` | | | `mock:Clear library — destructive, confirmation required` — **`danger:1` → `color:var(--block)`** |

> Row 8 `Texture sets` carries the submenu glyph `▸` but its action is `win:assets`, not `sub:…`. It opens the Asset library window; it never expands. Flagged as-is, not corrected.

**6.3a — `Asset pack` expansion** (inserted between rows 6 and 7 while `sub==='pack'`):

| Type | Label | Value | Shortcut | Indent |
|---|---|---|---|---|
| head | `ACTIVE PACK` | | | |
| read | `NAME` | `Eldra Atlas Pack` | | |
| read | `AUTHOR` | `A. Chos` | | |
| read | `LICENSE` | `CC-BY 4.0` | | |
| read | `SCHEMA` | `2 · STORED zip` | | |
| read | `FILLED` | `148 of 212 · 26 MB` | | |
| item | `Pack metadata…` | act `mock:Pack metadata — name, author, license` | | 36px |
| item | `Validate pack` | act `mock:Validate — 8 warnings` | | 36px |
| item | `Export pack .zip…` | act `mock:Export — pack.json schema 2 + PNGs, STORED zip` | `⌘⇧P` | 36px |

**6.3b — `Icon families` expansion** (between rows 7 and 8 while `sub==='fam'`). All four items have act `win:assets`, indent 36px:

| Label |
|---|
| `P · Places — 10 of 12` |
| `B · Buildings — 18 of 24` |
| `T · Trees & cover — 22 of 22` |
| `C · Compass & frame — 6 of 8` |

Then a **note** row: `24 families — full list in the Asset library`.

**6.3c — `Landmark types` expansion** (between rows 10 and 11, while `sub` starts with `lmt`). Two nesting levels, built from `LMFAMS()` (lines 1213–1229) and `lmCompute()` (lines 1241–1252).

Level 1 — one **item** per family, indent 36px, act `sub:lmt:<famId>`, glyph `▾` when that family is open else `▸`, shortcut cell = `<armed> of <typeCount> · <placedSum> placed`. With the default landmark state (`types[id].armed = cap>0`, `crowd=1`, `compete=true`) the six rows read exactly:

| Order | Label | Shortcut cell | act |
|---|---|---|---|
| 1 | `PHYSICAL` | `6 of 15 · 74 placed` | `sub:lmt:physical` |
| 2 | `TRANSPORTATION` | `3 of 8 · 21 placed` | `sub:lmt:transportation` |
| 3 | `ECONOMIC` | `2 of 6 · 9 placed` | `sub:lmt:economic` |
| 4 | `MILITARY` | `4 of 6 · 33 placed` | `sub:lmt:military` |
| 5 | `RELIGIOUS · CULTURAL` | `5 of 8 · 41 placed` | `sub:lmt:religious` |
| 6 | `HISTORICAL` | `3 of 6 · 9 placed` | `sub:lmt:historical` |

Level 2 — when a family is open, one **read** row per type (padding-left fixed at 36px by the read-row style):

- label = `'● '` if armed else `'○ '`, then the type name;
- value = armed ? `<cap> max · <placed> placed · <reason>` : `off · was <cap>`.

Default values, computed from the source data (`placed = min(cap, base)` when armed; `reason = 'at cap'` if `placed >= cap`, else the type's `fixed` string — `'no terrain'` / `'candidates'` — else `'spacing'`):

| Family | Label | Value |
|---|---|---|
| PHYSICAL | `● Peak` | `12 max · 12 placed · at cap` |
| | `○ Ridge` | `off · was 8` |
| | `○ Saddle` | `off · was 5` |
| | `○ Cliff` | `off · was 20` |
| | `● Gorge` | `8 max · 6 placed · no terrain` |
| | `○ Cave` | `off · was 12` |
| | `● Waterfall` | `40 max · 11 placed · spacing` |
| | `○ Spring` | `off · was 30` |
| | `● Lake` | `20 max · 12 placed · spacing` |
| | `○ Delta` | `off · was 3` |
| | `● River confluence` | `30 max · 24 placed · spacing` |
| | `○ Volcanic feature` | `off · was 2` |
| | `○ Rock formation` | `off · was 16` |
| | `○ Glacial feature` | `off · was 6` |
| | `● Ancient forest` | `16 max · 9 placed · candidates` |
| TRANSPORTATION | `● Mountain pass` | `12 max · 9 placed · spacing` |
| | `○ River crossing` | `off · was 20` |
| | `● Ford` | `20 max · 8 placed · no terrain` |
| | `○ Bridge site` | `off · was 12` |
| | `○ Road junction` | `off · was 8` |
| | `○ Caravan station` | `off · was 6` |
| | `○ Portage` | `off · was 4` |
| | `● Harbour` | `8 max · 4 placed · candidates` |
| ECONOMIC | `● Mine` | `12 max · 6 placed · spacing` |
| | `● Quarry` | `5 max · 3 placed · no terrain` |
| | `○ Salt works` | `off · was 4` |
| | `○ Resource extraction site` | `off · was 8` |
| | `○ Market site` | `off · was 6` |
| | `○ Trade depot` | `off · was 5` |
| MILITARY | `● Fort` | `12 max · 10 placed · spacing` |
| | `● Watchtower` | `30 max · 15 placed · spacing` |
| | `● Fortified pass` | `5 max · 5 placed · at cap` |
| | `○ Fortified crossing` | `off · was 6` |
| | `○ Battlefield` | `off · was 10` |
| | `● Border marker` | `8 max · 3 placed · candidates` |
| RELIGIOUS · CULTURAL | `● Shrine` | `50 max · 18 placed · spacing` |
| | `● Temple` | `8 max · 7 placed · spacing` |
| | `● Sacred grove` | `12 max · 8 placed · no terrain` |
| | `● Sacred mountain` | `3 max · 3 placed · at cap` |
| | `○ Pilgrimage site` | `off · was 6` |
| | `● Tomb` | `8 max · 5 placed · candidates` |
| | `○ Monument` | `off · was 8` |
| | `○ Ceremonial site` | `off · was 7` |
| HISTORICAL | `● Ruin` | `20 max · 5 placed · spacing` |
| | `● Abandoned settlement` | `8 max · 3 placed · spacing` |
| | `● Ancient road` | `3 max · 1 placed · candidates` |
| | `○ Historic battlefield` | `off · was 6` |
| | `○ Destroyed fortress` | `off · was 4` |
| | `○ Historic crossing` | `off · was 5` |

Immediately after the open family's leaves, one **item** row, indent 36px:
`Open in CIVIL ▸ Landmarks` → act `lmjump:<famId>` → sets `domain:'CIVIL'`, `civCat:'landmarks'`, closes the menu, and calls `setLm({openFam: famId})`.

Then, once, after all six families (still inside the `lmt` block):

| Type | Content |
|---|---|
| note | `leaves are read-only — the dropdown is a shortcut into the panel, never a second implementation of it` |
| read | `LANDMARK ICONS` → `poi · 10 slots` |
| item (indent 36px) | `Landmark label style… → Cartography` → act `dom:CARTO` |

**Submenu toggle semantics** (`act`, line 2025): `sub:V` → `sub = (state.sub === V) ? (V.startsWith('lmt:') ? 'lmt' : null) : V`.
- Clicking an already-open family (`sub:lmt:physical` while `sub==='lmt:physical'`) collapses to `'lmt'` — the family *list* stays open.
- Clicking `Landmark types` while `sub==='lmt'` closes the whole block; clicking it while a family is open collapses back to `'lmt'`.
- `sub` is a **single shared slot for the entire app** — there is one `state.sub`, not one per menu, so at most one submenu anywhere is expanded at a time.

### 6.4 Data

No separators; five heads. Every item glyph is `⧉`, no shortcuts.

| Type | Label | act |
|---|---|---|
| head | `IMPORT` | |
| item | `Maps · heightmaps (PNG · TIFF)` | `win:data:imp-maps` |
| item | `GIS / GeoJSON` | `win:data:imp-gis` |
| item | `World data (.zip · fields)` | `win:data:imp-world` |
| head | `EXPORT` | |
| item | `Maps (image · tiles)` | `win:data:exp-maps` |
| item | `GIS / GeoJSON` | `win:data:exp-gis` |
| item | `World data` | `win:data:exp-world` |
| item | `Assets (pack .zip)` | `win:data:exp-assets` |
| head | `SOURCES` | |
| item | `External sources` | `win:data:sources` |
| item | `Source registry` | `win:data:registry` |
| head | `CONVERSION` | |
| item | `Coordinate systems (EPSG)` | `win:data:crs` |
| item | `Format conversion` | `win:data:convert` |
| head | `VALIDATION` | |
| item | `Check data — 3 warnings` | `win:data:check` |
| item | `Repair / normalize` | `win:data:repair` |

### 6.5 Preferences

| Type | Label | Value / options | act |
|---|---|---|---|
| head | `PERFORMANCE` | | |
| toggle | `GPU acceleration — WebGPU` | on = `prefs.gpu` (**default `true`**) | `gpu` |
| read | `DEVICES` | `GPU 0 · 16 GB · 71% / GPU 1 · 64%` | |
| segmented | `multi-gpu` | `split tiles` · `alt frames` · `single` — selected = `prefs.mgpu` (**default `split tiles`**) | `mgpu` |
| read | `CPU WORKERS` | `12 of 16` | |
| read | `VRAM BUDGET` | `12 GB · fallback CPU tile pass` | |
| head | `GRAPHICS` | | |
| segmented | `quality` | `perf` · `balanced` · `quality` · `ultra` — selected = `prefs.quality` (**default `balanced`**) | `quality` |
| segmented | `anti-aliasing` | `off` · `2×` · `4×` · `8×` — selected = `prefs.aa === 'MSAA 4×' ? '4×' : prefs.aa` (**default `MSAA 4×` → renders `4×` selected**) | `aa` |
| read | `COLOUR` | `sRGB · anisotropy 8` | |
| head | `TILES & LOD` | | |
| read | `TILED LOD` | `auto on zoom · 512 px · L0–L8` | |
| read | `ATLAS CACHE` | `6.2 of 24 GB` | |
| item | `Clear caches…` | | `mock:Cleared atlas + field caches — never project data` |
| head | `MEMORY` | | |
| segmented | `undo depth` | `5` · `15` · `50` — selected = `String(prefs.undoDepth)` (**default `5`**) | `undoDepth` |
| read | `WORKING SET` | `1.6 GB of 12 GB` | |
| head | `APPLICATION` | | |
| segmented | `theme` | `dark` · `light` — selected = `state.light ? 'light' : 'dark'` (**default `dark`**) | `theme` |
| segmented | `units` | `km` · `mi` — selected = `prefs.units` (**default `km`**) | `units` |
| item | `Keyboard shortcuts…` | | `mock:Editable shortcut table — per context` |
| item | `Storage locations…` | | `mock:Same modal as File ▸ Change locations` |

Setter effects (lines 2036–2042) — **none of these close the menu:**

| act prefix | Effect |
|---|---|
| `autoInt=` | `prefs.autoInt = value` |
| `mgpu=` | `prefs.mgpu = value` |
| `quality=` | `prefs.quality = value` |
| `aa=` | `prefs.aa = value` |
| `undoDepth=` | `prefs.undoDepth = +value` (number) |
| `theme=` | `state.light = (value === 'light')`, canvas dirty |
| `units=` | `prefs.units = value`, canvas dirty, calls `_updateHud()` |
| `gpu` | flips `prefs.gpu` |
| `autosave` | flips `prefs.autosave` |

> `anti-aliasing` stores a different format than it seeds with: the initial `prefs.aa` is `'MSAA 4×'` and is displayed via a one-off remap to `'4×'`; the first click on any pill writes the bare pill string (`'off'`, `'2×'`, `'4×'`, `'8×'`), after which the remap is identity. Recorded as found.

### 6.6 Window

| Type | Label | On-state | act |
|---|---|---|---|
| toggle | `Left dock` | `state.ldOpen` (**default `true`**) | `win:ld` |
| toggle | `Right dock` | `state.rdOpen` (**default `true`**) | `win:rd` |
| toggle | `Domain rail` | `state.showRail` (**default `true`**) | `win:rail` |
| toggle | `Status bar` | `state.showSB` (**default `true`**) | `win:sb` |
| toggle | `Timeline (CIVIL · INFRA)` | `state.tlOpen` (**default `false`**) | `win:tl` |
| sep | | | |
| item | `Reset layout` | | `resetlayout` |
| item | `Save layout as…` | | `mock:Layout saved as preset` |
| sep | | | |
| head | `WORKSPACES` | | |
| item | `World` | | `dom:WORLD` |
| item | `Civilization` | | `dom:CIVIL` |
| item | `Cartography` | | `dom:CARTO` |

State effects:
- The five `win:ld|rd|rail|sb|tl` actions flip their boolean and **return before any menu close** — the menu stays open so several can be toggled in one visit.
- `resetlayout` → `ldOpen:true`, `rdOpen: (frame !== 'tabP')`, `showRail:true`, `showSB:true`, `railExp:false`, closes menu, toasts `Layout reset`. (Right dock is deliberately left closed on the tablet-portrait frame.)
- `dom:X` → `setDomain(X)` → `{domain:X, railExp:false}`, canvas dirty; then closes the menu.

### 6.7 Help

| Type | Label | act / value |
|---|---|---|
| item | `Documentation` | `mock:Documentation opens in the OS browser` |
| item | `Keyboard shortcuts` | `mock:V M R B L I arm tools · ⌘Z undo · Esc commits or disarms` |
| item | `Credits & academic principles` | `mock:Credits — generation follows published geomorphology` |
| item | `Report an issue` | `mock:Issue reporter` |
| sep | | |
| read | `VERSION` | `2.11 · build 4183` |

---

## 7. Does an action close the menu?

| act pattern | Closes menu (`menu:null, sub:null`)? |
|---|---|
| `mock:…` | **Yes** — plus toast `<text> (mock)` |
| `sub:…` | No — toggles `state.sub` only |
| `world:N:S` | **Yes** |
| `dom:X` | **Yes** |
| `lmjump:F` | **Yes** |
| `win:ld` / `win:rd` / `win:rail` / `win:sb` / `win:tl` | No |
| `win:<anything else>` | **Yes if `this.openWindow` exists**; otherwise no close, toast `⧉ window — coming in this build`. **`openWindow` is not defined anywhere in the surviving 2105 lines**, and the window mount point is the empty comment `<!--ANCHOR_WIN-->` (line 1148) — so as the file stands, every `⧉` row toasts and leaves the menu open. |
| `autoInt=` `mgpu=` `quality=` `aa=` `undoDepth=` `theme=` `units=` | No |
| `histjump:i` | **Yes** (after `i+1` undos) |
| `save` `undo` `redo` `autosave` `gpu` | No |
| `pick` `delsel` `desel` `resetlayout` | **Yes** |
| any act not matched above | No — toast `<act> (mock)` |

**`dim` does not disable.** `r.dim` only recolours to `var(--dis)`; the `onClick` is still bound and the action still fires. `Paste` is permanently dim and still toasts `Paste (mock)`.

**Toast presentation** (lines 1150–1153, referenced because most menu actions produce one): stacked bottom-centre at `bottom:calc(var(--sbH) + 14px)`, `gap:6px`, `pointer-events:none`, `z-index:90`; each pill `background:var(--ins); border:1px solid var(--bor); color:var(--body); padding:7px 16px; border-radius:999px; font:var(--m1) 'IBM Plex Mono',monospace; box-shadow:var(--shadow); animation:tIn .18s ease` (`tIn` = opacity 0→1 with `translateY(6px)→none`). At most 3 visible (`toasts.slice(-2)` + new); each auto-removes after **2600 ms**.

---

## 8. Keyboard, as it actually is

`componentDidMount` (lines 1828–1835) is the only key handler. It is skipped when the event target is an `input` or `textarea`, and when `state.scr !== 'app'`.

| Key | Effect |
|---|---|
| `⌘Z` / `Ctrl+Z` | `doUndo()`, `preventDefault` |
| `⌘⇧Z` / `Ctrl+Shift+Z` | `doRedo()` |
| `Esc` | first branch closes the menu; then layers panel; then `escExtra()`; then commit measure; then clear region; then disarm tool to `inspect` |
| `v m r b l i f` (no ⌘/Ctrl) | arm tool: `inspect` `measure` `region` `biome` `label` `icon` `freehand` |
| `1`–`8` | set layer (`LAYERS[].key`): 1 Relief, 2 Biome, 3 Political, 4 Elevation, 5 Slope, 6 Flow accumulation, 7 Temperature, 8 Rainfall |

Consequence for the menu: **there is no focus trap.** Every letter shortcut printed in the popups other than `⌘Z`/`⌘⇧Z` is decorative — `⌘N ⌘O ⌘S ⌘⇧S ⌘W ⌘X ⌘C ⌘V ⌫ ⌘A ⌘D ⌘F ⇧A ⌘⇧P` are rendered but bound to nothing. Typing `f` while the File menu is open arms the Freehand tool rather than doing anything to the menu.

---

## 9. UNSPECIFIED

**A. The truncated tail.** The file ends at byte 262144, mid-expression inside `valsCore()` (line 2105, `measRows.push({i:('0'+i).slice(-2),len:this.fmtKm(km),be` …). Everything the template binds but the surviving code never defines:

| Binding | Where used | What is missing |
|---|---|---|
| `hMenu` | line 60 | Whether clicking an open menu's title closes it (toggle) or re-opens it; whether it clears `state.sub`; whether it reads `e.currentTarget.dataset.id`. |
| `hAct` | lines 67, 74, 84 | Confirmation that it dispatches `dataset.act` into `act()`; whether it stops propagation (relevant because the document capture-phase handler runs first). |
| `menuRows` | line 63 | Confirmation that it is `this.menuRowsFor(state.menu)`. |
| `hUndo` / `hRedo` | lines 101–102 | Almost certainly `doUndo`/`doRedo`, but not stated. |
| `undoCol` / `redoCol` | lines 101–102 | The enabled and disabled colours for the two bar buttons. Not derivable — nothing in the surviving file computes them. |
| `hTheme` | line 105 | The bar's `◐` button behaviour (flip `state.light`, presumably) and whether it differs from the artboard-chrome `hTheme` at line 21. |
| `worldLabel` | line 104 | **The entire right-hand readout text.** Available state it could draw on: `world.name`, `world.seed`, `world.status`, `savedAt` (default `'14:02'`), `finalized`. Which of these, in what order, with what separators — unknown. |
| `fvars` | line 25 | The token-override injection point. `valsCore` computes `themeStr` (the full light palette, line 2063) and `densStr` (the touch / `w1366` metric overrides, line 2064) but the file is cut before either is placed into a return object. Concatenation order and whether anything else is appended are unknown. |
| `scrShell` | line 55 | Presumed `state.scr === 'app'` (the key handler uses that test) but never written. |

**B. Interaction gaps the design does not answer:**

1. **No hover state on the menu-bar titles.** Only `color`/`background` for open vs. closed; `style-hover` is absent on the title chip (present on item and toggle rows only).
2. **No hover-to-switch.** With one menu open, moving the pointer over another title does nothing; a second click is required. Not stated either way.
3. **No keyboard navigation inside a popup** — no arrow keys, no Home/End, no Enter/Space, no type-ahead, no focus ring. No element in the bar or popup is focusable (all `div`/`span`, no `tabindex`, no ARIA roles).
4. **No edge handling.** The popup is unconditionally `left:0` at `width:var(--pop)`; there is no flip, clamp, or shift for a menu near the right edge, and no logic for `max-height:72vh` interacting with the artboard's `transform:scale()`.
5. **Scrollbar appearance** for the `overflow-y:auto` popup is unstyled — the platform default. Nothing says what it should look like at `--pan` on a 300px popup.
6. **No pressed/active state** on any row — only hover `var(--wash)`. No ripple, no flash, no delay before the menu closes on activation.
7. **Toggle knob motion** — `left` flips between `2px` and `15px` with no `transition` declared. Whether it should animate is not stated.
8. **Segmented pills have no hover state at all** (no `style-hover`), unlike item and toggle rows.
9. **Separator insets** — separators span the full popup width (no left inset to 14px), while every row is inset. Deliberate or not is not stated.
10. **`Texture sets`** shows the `▸` submenu affordance but performs `win:assets`. Whether the intent is a submenu that was never written, or the glyph is wrong, is not stated.
11. **`Data` menu has no separators** while File, Edit, Assets, Window and Help do — Data relies on section heads alone. Not stated as a rule.
12. **Window targets are unimplemented.** All fifteen `⧉` rows (Assets ×2, Data ×13) resolve to `openWindow`, which does not exist; `<!--ANCHOR_WIN-->` is an empty placeholder. The prototype specifies the menu row and its id string, and nothing about what the window contains.
13. **Menu bar in the touch frames.** `--menuH` becomes `52px` and `--pop` `380px` on `tabL`/`tabP`, but the bar markup is unconditional — there is no alternate touch layout, no long-press, and the seven menus plus wordmark plus four controls are laid out identically. Whether the desktop menu bar is meant to survive onto tablet unchanged is not stated.
14. **`state.sub` is global.** Only one submenu can be expanded across the whole application, and nothing resets it when the menu changes. Whether that is intended is not stated.
15. **The bar has no background of its own** — it relies on `var(--sur)` from the shell. If the menu bar is built as a standalone container, the background must be supplied.

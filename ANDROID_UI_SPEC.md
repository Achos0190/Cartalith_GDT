# ANDROID_UI_SPEC.md — the phone canvas, inventoried against the shell

**What this is:** the phone design canvas walked screen by screen, with the
canvas's own figures, the engine or shell symbol each element maps onto, and
every place this build deliberately departs from the canvas and why. It also
holds the touch-gesture rules the phone and tablet share (§1.12);
`TABLET_UI_SPEC.md` points here for them. **What it is not:** a status
record — how far the phone is built is `cartalith-native/docs/STATUS.md`'s
answer, and open phone work is routed from `OUTSTANDING_WORK.md` (its conformance
row, *"THE PHONE SHELL MUST MATCH `Cartalith Android.dc.html` 100%
FAITHFULLY"*, is the umbrella).

**Authority:** `design/Cartalith-Android-2026-09-07.dc.html` (170 327 bytes,
vendored 2026-09-07). Owner, 2026-09-07: *"The app only faintly resembles this
version … where it should resemble it 100% faithfully."* The same content sits
at `design/mcp-2026-09-07/Cartalith Android.dc.html` (168 836 bytes); the two
differ only in line endings, so **line numbers agree between them and byte
offsets do not**. Owner decisions newer than the canvas win over it: the New
World card (§6), Ruling Z (MAP's half detent, §0.2), and the 2026-09-05 ruling
that builds phone MORE from `design/dcc-environment-2026-08-31/spec/06-phone.md`
§6.6 (§4).

**Not the owner's `docs/ANDROID_UI_SPEC.md`.** Shell comments in
`dcc_shell.gd`, `phone_menu.gd`, `phone_project_picker.gd` and `dcc_icons.gd`
cite `docs/ANDROID_UI_SPEC.md` and its "Locked decisions". That is the owner's
own document in the Claude Design project (`design/android-2026-08-30/README.md`
points at it), never committed here. This file is a different document; do not
repoint those citations at it. (An earlier version of this file claimed to be
the file they meant, and called their `docs/` prefix a path error.)

**Coverage:** §0 conventions, §1 GENERATE, §2 MAP and §6 the NEW WORLD modal are
inventoried; §3 PLAN, §4 MORE and §5 Overlays are stubs — extend this file, do
not replace it. **Grep the symbol, never seek an offset** (`MISTAKES.md`,
"Re-resolve a citation late in a long pass"); every canvas location below is a
symbol or a line number, and a byte offset only where the canvas offers nothing
else.

---

## 0. How to read the canvas

One interactive prototype, not a set of artboards: an `<x-dc>` template (roughly
the first 1 000 lines) followed by the component's state and handlers. Each tab's
content is an `sc-if` block (`tabIsGen`, `tabIsMap`, …) inside the sheet;
floating chrome over the map is a sibling of the sheet, not a child of it (§2.0).

### 0.1 Palette — the canvas's variables against `DccTheme`

Read off the frame's inline `style` (dark: the `--sur:#0d0e0f;…` string on line
31; light: line 1469) and compared value-by-value with `dcc_theme.gd`'s `DARK`.

| Canvas var | Canvas value | `DccTheme.DARK` | Verdict |
|---|---|---|---|
| `--sur` | `#0d0e0f` | `bg` `#0d0e0f` | exact |
| `--pan` | `#15171a` | `raised` `#17191a` (`panel` is `#121314`) | near |
| `--pan2` | `#121314` | `panel` | exact |
| `--ink` | `#e8ebec` | `text_bright` | exact |
| `--body` | `#c8cbcd` | `text` | exact |
| `--sec` | `#8d9296` | `text_secondary` is `#a9adb0`; `#8d9296` is `text_dim` | **one rung dimmer** |
| `--dim` | `#6f7478` | `text_dim` is `#8d9296`; `#6f7478` is `text_faint` | **one rung dimmer** |
| `--faint` | `#5f6468` | `text_faint` is `#6f7478`; `#5f6468` is `text_ghost` | **one rung dimmer** |
| `--acc` | `#e0a34a` | `accent` | exact |
| `--accInk` | `#16130c` | `accent_ink` `#141005` | differs |
| `--hair` | `rgba(255,255,255,.10)` | `line` | exact |
| `--hair2` | `rgba(255,255,255,.07)` | `line_soft` | exact |
| `--bord` | `rgba(255,255,255,.16)` | `border` | exact |
| `--wash` | `rgba(224,163,74,.14)` | `accent_wash` α .09, `accent_wash_2` α .16 | no exact token |
| `--chip` | `rgba(255,255,255,.05)` | — | no token; drawn as `Color(DccTheme.c("text_bright"), 0.05)` |
| `--chipOn` | `rgba(255,255,255,.10)` | — | no token |
| `--warn` | `#e0a840` | `warn` `#e0a840` | exact |
| `--block` | `#c26a60` | `block` `#c96a5a` | differs |
| `--good` | `#8fae7d` | `good` `#6fae7d` | differs |
| `--water` | `#7d9dae` | `water` `#6a9bc4` | differs |

**The dark secondary ramp is shifted one rung against the PC canvas.** The
phone's `--sec` / `--dim` / `--faint` carry the values the PC canvas and
`DccTheme` name `--dim` / `--faint` / `--dis`. A build that maps by **name**
draws the phone's secondary text one rung brighter than the canvas — §6.6's
CANCEL ink is the measured instance. The light half's ramp (line 1469) does
match `DccTheme.LIGHT` by name (`--sec #3d3f39` = `text_secondary`, `--dim
#6b6f6a` = `text_dim`, `--faint #8d9088` = `text_faint`); its `--wash` is α .10
against `accent_wash`'s .09. An earlier version of this table called the three
dark ramp rows, `--accInk` and `--wash` exact — wrong on the day, as the
same-day tablet inventory's quoted token values show — and `--warn` tokenless,
which is not true of the code now.

`--good` / `--block` / `--water` differ on purpose: the PC and phone canvases
contradict each other outright on the semantic triple, and `dcc_theme.gd`
(the block above `"good"`) takes the PC canvas's and records why. `--chip` has
no token and is built off `text_bright` so `DccTheme.remap()` can trace it — on
light it becomes a 5 % black wash, as a light theme wants.

### 0.2 Frame and detents

- Frame `412 × 892`, `border-radius:30px`. `DccTheme.PHONE_REF_SHORT = 412.0` is
  the same number.
- `_detH(det)`: `fh = frameH() − 84` portrait; `peek → 66`, `half → round(fh ×
  0.46)`, `full → fh − 96`. `DccShell._phone_detent_height()` transcribes it
  (`PHONE_DETENT_PEEK`, `PHONE_DETENT_HALF_FRAC`, `PHONE_DETENT_FULL_GAP`) —
  **except MAP's `half`**, which is sized to MAP's real content
  (`_phone_map_sheet_content_height()`, never above the flat 0.46), by **Ruling
  Z** (2026-09-21): the canvas's proportion left that sheet ~92 % blank.
- `navH = 84` portrait (bottom nav `height:84px`), `0` landscape.
- Landscape sheet: right-docked, `width = min(440, round(fw × 0.46))`,
  `border-radius:0`, `border-left:1px solid var(--hair)`.
- The detent tween is `PHONE_DETENT_ANIM` = 0.28 s on `custom_minimum_size:y`.
  **Await it before reading a sheet height** — a mid-tween read once reported
  609 px for a sheet that settles at 975 (`MISTAKES.md`, "Read a size, position
  or detent from an animating control").

### 0.3 Bottom nav — the tab row

```
tabs = [{id:'map',g:'▤',label:'MAP'}, {id:'gen',g:'⌗',label:'GENERATE'},
        {id:'plan',g:'➔',label:'PLAN'}, {id:'more',g:'⋯',label:'MORE'}]
```

Row `height:84px`, `background:var(--pan2)`, `border-top:1px solid var(--hair2)`,
`padding:4px 6px 0`. Each cell: glyph in `padding:4px 16px; border-radius:13px`
with `background: var(--wash)` when on, caption
`font:9.5px 'IBM Plex Mono'; letter-spacing:.12em`, ink `var(--acc)` on /
`var(--sec)` off. Gesture bar `height:max(env(safe-area-inset-bottom),18px)`
with a `112 × 4` radius-2 pill.

`hTab` verbatim:

```
hTab: e => { const t = e.currentTarget.dataset.tab;
  this.setState(s => { if (s.tab === t) return {tab:null};
                       return {tab:t, detent: s.detent==='peek' ? 'half' : s.detent} }) }
```

**Shell:** `DccShell.PHONE_TABS` is this list one-for-one (glyphs included), and
`_pick_phone_tab()` implements the same detent rule.

### 0.4 Sheet header

A `42 × 4` handle row, then a title row `padding:0 14px 10px` carrying an
optional `38 × 38` back circle, a title (`font:500 11px mono;
letter-spacing:.2em; color:var(--acc)`), a subtitle (`9.5px mono;
color:var(--dim)`) and a `38 × 38` close ✕. `titles.gen = ['GENERATE',
genMode==='pipe' ? 'pipeline · seed '+seed : 'sculpt · draft stamps']`;
`titles.map = ['MAP', 'layers · style · annotation']`. `sheetBack` is
`(tab==='more' && moreDepth) || (tab==='plan' && planDepth)`.

**Shell:** `_phone_sheet_title` / `_phone_sheet_subtitle`, refreshed per tab by
`DccShell._refresh_phone_sheet_header()`; every path that changes a subtitle's
source (a recompute, a stage isolate) must call it.

---

## 1. GENERATE — `tabIsGen`

Everything below is the canvas's own number unless it says otherwise.

### 1.0 Column

`display:flex; flex-direction:column; gap:12px` inside the sheet scroller, which
is `padding:2px 14px 24px`.

### 1.1 Mode segment — `hGenModePipe` / `hGenModeSculpt`

Two cells, `display:flex; gap:8px`, each `flex:1`:

| Property | Value |
|---|---|
| min-height | `44px` |
| border-radius | `22px` |
| font | `500 10px 'IBM Plex Mono'`, `letter-spacing:.16em` |
| captions | `PIPELINE`, `SCULPT` |
| on | border `var(--acc)`, ink `var(--acc)`, bg `var(--wash)` |
| off | border `var(--hair)`, ink `var(--sec)`, bg `transparent` |

**Engine:** `DccShell.select_domain_mode("world", …)` carries the per-domain
mode; `world_workspace.gd` builds `_sculpt_body` for Sculpt from
`bridge.get_sculpt_features()` / `get_sculpt_presets()` / `sculpt_get_globals()`.

### 1.2 SEED row — `genSeed`, `hDice`

Row: `padding:10px 12px`, `border-radius:16px`, `background:var(--chip)`,
`align-items:center`, `gap:10px`.

| Element | Spec |
|---|---|
| caption | `SEED`, `9.5px mono`, `letter-spacing:.16em`, `var(--dim)` |
| value | `flex:1`, `500 13px mono`, `var(--ink)` |
| dice | `42 × 40`, `border-radius:14px`, `background:var(--wash)`, ink `var(--acc)`, glyph `⚄`, `14px mono` |

`hDice` sets a fresh 6-digit seed **and calls `_markStale(1)`** — a new seed
makes stage 01 onward stale.

**Engine:** the seed is `new_world_dialog.gd::request()["seed"]`; the dice is
`app._new_seed()` → `new_world_dialog.randomise_seed()`. The canvas's `42 × 40`
dice is under the 44 dp floor on both axes (§1.11).

### 1.3 Progress card — `genRunning`, `progLog`, `hCancelGen`

Shown only while running. `border:1px solid var(--acc)`, `border-radius:18px`,
`padding:13px 14px`, `gap:9px`.

| Element | Spec |
|---|---|
| title `progTitle` | `500 10.5px mono`, `letter-spacing:.14em`, `var(--acc)`; text is `"NN · <stage name>"` |
| pct `progPct` | `10px mono`, `var(--dim)`; `round((i*100 + min(pct,100)) / 10) + '%'` |
| bar | `height:5px`, `border-radius:3px`, track `var(--chip)`, fill `var(--acc)` |
| log rows `progLog` | `9.5px mono`, per-row `l.col`; **last 3 only** (`log.slice(-2)` plus the new line) |
| log text | `"NN <stage> — resolved · N.N s"` |
| CANCEL | `align-self:flex-end`, `padding:8px 14px`, `radius:16px`, bg `var(--chip)`, ink `var(--sec)`, `10px mono`, `letter-spacing:.12em` |

`GENSTAGES`, all ten, in order: `Planet`, `Extent & scale`, `World structure`,
`Tectonics`, `Volcanism & impacts`, `Erosion`, `Hydrology`, `Climate`, `Ecology
& biomes`, `Resources & soils` — **identical, in order, to
`world_workspace.gd::STAGES[*].name`** and to
`cartalith-engine/src/progress.rs::STAGE_NAMES`, which `_assert_stage_names()`
guards. (The phone canvas uses the engine's names; the tablet canvas does not —
`TABLET_UI_SPEC.md` §2.4, Ruling E.)

**Engine:** `EngineBridge.generation_stage(index, name, total)` is a real
per-stage signal off `GenerationProgress`, plus `generation_started` /
`generation_finished`; per-stage elapsed time is `world_workspace.gd`'s
`_stage_elapsed_ms`. **CANCEL cannot be backed:** `engine_bridge.gd` has
`generate()` and no cancellation entry point (its only `cancel` functions are
`sculpt_cancel_stroke()` and `label_cancel_edit()`). The prototype's cancel is
`clearInterval` over a simulated timer, not a claim about this engine.

### 1.4 Idle block — `staleOn`, `hGenerate`

| Element | Spec |
|---|---|
| stale note | `padding:11px 14px`, `border:1px solid rgba(224,168,64,.4)`, `radius:14px`, `font:10px/1.6 mono`, ink `var(--warn)` |
| stale text | `"Stage NN edited — stages NN → 10 are stale. Fields owned by stale stages read — until re-run."` |
| GENERATE button | `min-height:52px`, `border-radius:26px`, bg `var(--acc)`, ink `var(--accInk)`, `font:500 11px mono`, `letter-spacing:.18em` |
| label | `"GENERATE WORLD"`, or `"REGENERATE NN → 10"` when stale **and** a run has happened |
| footer | `9.5px mono`, `var(--faint)`, `padding:0 4px`, `justify-content:space-between` |
| footer left | `"last run · " + genLastRun` (`—` when never) |
| footer right | `"10 stages · dependency order"` |

The canvas's own absence convention is the em-dash: `genLastRun` starts `'—'`.

**Engine:** `world_workspace.gd::_regenerate_live()` is the guarded Generate (it
prompts before discarding hand-authored work); `_stale_from_stage` and
`_stale_note_text()` are the stale model; `_mark_stale_from(stage_index)` is
called by every parameter row's release handler.

### 1.5 `genGroups` — the collapsible parameter groups

Data `_genFieldDefs()`, handler `hGenGroup`.

Box: `border:1px solid var(--hair2)`, `border-radius:18px`, `overflow:hidden`.

Header row — `min-height:50px`, `padding:0 14px`, `gap:10px`:

| Element | Spec |
|---|---|
| `g.num` | `9.5px mono`, `width:22px`, ink `g.numCol` = `var(--warn)` when stale else `var(--faint)` |
| `g.name` | `flex:1`, `font-size:12.5px`, ink `var(--ink)` |
| `g.state` | `9px mono`, `var(--faint)`; `'stale'` or `'resolved'` |
| `g.chev` | `11px mono`, `var(--faint)`; `'⌄'` open, `'›'` closed |

Body — `border-top:1px solid var(--hair2)`, `padding:6px 14px 12px`, `gap:2px`.

**Segment (`f.isSeg`)** — label `10px mono var(--sec)`, `padding-bottom:7px`;
options `min-height:38px`, `padding:0 13px`, `border-radius:19px`, `10px mono`;
selected border/ink `var(--acc)` on `var(--wash)`, else border `var(--hair)`,
ink `var(--sec)`, transparent.

**Range (`f.isRange`)** — label `10px mono var(--sec)` left, `f.disp`
`11px mono var(--ink)` right; then `−` and `＋` steppers each `38 × 38`,
`border-radius:14px`, bg `var(--chip)`, ink `var(--sec)`, `14px mono`, with an
`<input type=range>` between them at `flex:1`. Track `height:3px`,
`border-radius:2px`, `background:var(--hair)`; thumb `20 × 20`, radius 10,
`background:var(--acc)`. `f.disp` = `step<1 ? val.toFixed(2)
: Math.round(val).toLocaleString('en-US')`, then `f.unit`.

**Toggle (`f.isTog`)** — row `min-height:44px`, `gap:12px`; label
`10px mono var(--sec)` at `flex:1`; track `40 × 22` radius 11, `var(--acc)` on /
`var(--chip)` off; knob `18 × 18` radius 9, `top:2px`, `left:2px` off /
`right:2px` on, fill `var(--accInk)` on / `var(--sec)` off.

The steppers (38 × 38), segment chips (38 high) and group header (50) are the
canvas's figures, not this build's — the build floors them (§1.11).

### 1.6 The canvas's fields against this engine's parameter table

`_genFieldDefs()` in full, against `crates/cartalith-godot/src/params.rs`'s
`PARAMS` (99 rows in 9 groups, counted 2026-09-23 — `grep -c 'key: "'`).

| Canvas group | Canvas field | Canvas range | Engine key | Engine range | Verdict |
|---|---|---|---|---|---|
| 01 Planet | Gravity | 0.5–2 / .01 / ` g` | `planet.g` | 0.3–2.5 / .05 | live, **engine range is wider** |
| | Day length | 6–48 / 1 / ` h` | `planet.rotation_hours` | 6–96 / 1 | live, wider |
| | Axial tilt | 0–45 / .1 / `°` | `planet.axial_tilt_deg` | 0–45 / .5 | live, coarser step |
| 02 Extent & scale | Working resolution | seg 512/1024/2048/4096 | — | — | **no parameter** |
| | Sea level | 0–100 / 1 / ` %` | `sea_level` | 0–1 / .01 | live (unit differs) |
| | Peak altitude | 1000–9000 / 100 / ` m` | `peak_m` | 1–30000 / 50 | live, wider |
| 03 World structure | Archetype | seg Earthlike/Supercontinent/Islands/Rift | — | — | **creation-time** |
| | Continentality | 0–1 / .01 | `world_structure.continentality` | 0.01–0.9 / .01 | live |
| | Fragmentation | 0–1 / .01 | `world_structure.fragmentation` | 0–1 / .01 | live, exact |
| 04 Tectonics | Plates | 4–30 / 1 | `tect.plates` | 4–40 / 1 | live, wider |
| | Drift | 0.2–3 / .05 / ` ×` | `tect.vel` | 0–2 / .02 | live, different |
| | Tectonic energy | 0–1 / .01 | `world_structure.tectonic_energy` | 0–1 / .01 | live, exact |
| 06 Erosion | Erosion strength | 0–1 / .01 | — | — | **no single key** |
| | Stream-power carve | 0–1 / .01 | `stream.k` | 0–0.03 / .0003 | live under another name |
| 07 Hydrology | River density | 0–1 / .01 | `river_density` | 0.3–3.0 / .05 | live, different |
| | Min stream order | 1–8 / 1 | — | — | **not settable** |
| 08 Climate | Equator temp | 10–40 / 1 / ` °C` | `climate.equator_temp` | 0–45 / 1 | live, wider |
| | Pole temp | −40–5 / 1 / ` °C` | `climate.pole_temp` | −50–10 / 1 | live, wider |
| | Rainfall | 0.2–2 / .05 / ` ×` | `climate.rain_k` | 0–2 / .01 | live, wider |
| 09 Ecology & biomes | Ecotone sharpness | 0–1 / .01 | — | — | **not parameterised** |
| | Rivers in biome view | toggle | — | — | **a layer switch, not a parameter** |

**The absences, each opened at the symbol:**

1. **Working resolution.** `params.rs`' `world` group is `world`, `sea_level`,
   `peak_m`, `carve_rivers`, `river_density`, `integrate_drainage`, `use_gpu` —
   no resolution key anywhere in the table. `STAGES[1]["gap"]` states the rule
   (*"GRID HEIGHT IS A CALL ARGUMENT, NOT A PARAMETER"*, `main.gd`'s own): it is
   set in File ▸ New world.
2. **Archetype.** `bridge.archetypes()` / `apply_archetype(name)` work, but
   `new_world_dialog.gd::NOTE_CREATION_ONLY` is explicit that extent,
   resolution and archetype reallocate every field, and `request()["archetype"]`
   decides which generation call runs. So the row is a **route, not a dash**:
   it names the current archetype and opens the New World card, which carries
   the control on a phone (§6).
3. **Erosion strength.** No key means it; stage 06's `erosion` group holds 28
   rows (`stream.*`, `passes.*`). One synthesised 0–1 dial over several would be
   a second parameter table that can drift from the desktop's, so the group
   draws the engine's own rows.
4. **Min stream order.** Strahler order is real (`get_rivers(min_order)`), and
   since 2026-09-22 the drawn rivers *are* those polylines — but the viewport
   always asks for order 1 (`viewport_host.gd`: `overlay.set_rivers(_bridge.rivers(1))`)
   and no control changes it (`STAGES[6]["gap"]`). A filter nothing exposes, on
   either density.
5. **Ecotone sharpness.** Ecology has no parameter group (`STAGES[8]["gap"]`:
   *"Not parameterised"*).
6. **Rivers in biome view.** Not a generation parameter: it is CARTO ▸ Layers'
   `Rivers` row, one switch for the canvas's `#showRivers` and rivers-as-ways
   (`cartography_workspace.gd`'s note beside that row).

### 1.7 The group set — ten, where the canvas draws eight

The canvas draws `01 02 03 04 06 07 08 09` and footnotes *"Volcanism (05) and
Resources (10) run with defaults."* **True of the prototype, false of this
engine:** `params.rs` files 9 live parameters under `group: "volcanism"`
(`volc.*`, `crater.*`), all on the desktop's stage-05 body. Hiding them would
remove nine working controls.

**Decision, reversible in one place:** the phone draws `world_workspace.gd::STAGES`
— all ten — in the canvas's `genGroups` chrome, so 05 Volcanism & impacts is a
group of nine rows and 10 is a group whose body carries the engine's own `gap`
prose. The footnote is rewritten to say what is true here. Everything about the
*drawing* — box, radii, number column, state caption, chevron, field forms — is
the canvas's.

### 1.8 Where the fields come from — one source, not two

Every row is built from the **desktop's own** parameter source:
`EngineBridge.param_keys()` / `param_info(key)` (`{group, label, unit, type,
min, max, step, reference_control}`) / `param_get` / `param_set` /
`param_default`, with membership by `STAGES[i]["groups"]` and `STAGES[i]["keys"]`
— the predicate `world_workspace.gd::_build_group_section()` uses. Adding a
parameter is still one row in `params.rs` and no GDScript change.
`ADVANCED_KEYS` (the keys the desktop folds behind a disclosure) are drawn
inline at the foot of their group: the canvas has no disclosure form, and
dropping them would lose live controls.

### 1.9 SCULPT mode — `isSculpt`

In order: a `GEOLOGICAL FEATURE` caption (`9.5px mono`, `letter-spacing:.2em`,
`var(--dim)`), a wrap of feature chips (`min-height:42px`, `padding:0 13px`,
`radius:21px`, a `14 × 14` SVG glyph then the id), a hint line, a `PRESETS`
caption and chip wrap (`min-height:38px`, `radius:19px`, `9.5px mono`), a
`BRUSH · GLOBAL` caption and range rows, a dashed-border `＋ ADD DRAFT STAMP
(MOCK STROKE)` button (`min-height:48px`, `radius:24px`, `1px dashed
var(--acc)`), then a stamp note with `DISCARD` (`padding:11px 14px`,
`radius:18px`, `var(--chip)`) and `✓ COMMIT` (`padding:11px 16px`,
`radius:18px`, `var(--acc)` on `var(--accInk)`), and a closing prose line.

**Engine:** all live — `bridge.get_sculpt_features()`, `get_sculpt_presets()`,
`sculpt_get_globals()`, `get_sculpt_globals_info()`, `sculpt_stamp_count()`,
`sculpt_set_feature()`, `sculpt_apply_preset()`; `world_workspace.gd::_build_sculpt()`
builds the same on the desktop. The canvas's `ADD DRAFT STAMP (MOCK STROKE)` is
the prototype standing in for a map stroke; this build's button arms the sculpt
tool (`app.arm_tool("sculpt")`) and drops the sheet to `peek`.

### 1.10 Tap path — launch to a generation slider

"It renders" and "it can be operated" are not "it can be found" (`MISTAKES.md`,
"Claim a screen 'works' on a phone"):

1. Launch → project picker (`phone_project_picker.gd`) → pick or create a world.
2. The app screen shows the map, the app bar and the bottom nav.
3. **Tap `GENERATE`** — always on screen, second of four cells.
4. `_pick_phone_tab("gen")` selects the `world` domain and lifts the sheet from
   `peek` to `half`.
5. The sheet is the GENERATE column: `PIPELINE | SCULPT`, `SEED`,
   `GENERATE WORLD`, then the ten groups.
6. **Tap a group header** and its fields open under it; drag the handle up for
   `full` if the group is long.

Nothing in that path is a gesture, a long-press or an off-screen affordance.

### 1.11 Every place the GENERATE column departs from the canvas

Built in `dcc_shell.gd` (`_build_phone_tool_sheet()`,
`_refresh_phone_gen_panel()`) and `workspaces/world_workspace.gd`
(`build_phone_generate()` and the `_pg_*` block). Probe:
`_genphone_probe.gd`, which prints its own check count.

| Canvas says | This build | Why |
|---|---|---|
| 8 groups (`01-04, 06-09`) | **10**, the whole `STAGES` table | 05 Volcanism is 9 live parameters here; §1.7 |
| footnote "Volcanism (05) and Resources (10) run with defaults" | rewritten | false of this engine; §1.7 |
| `input[type=range]{height:22px}` | slider row floored to **44 dp** | the ± pair is already 44 |
| dice glyph `⚄` (U+2684) | `↻` (U+21BB) | U+2684 is absent from the shipped `IBMPlexMono-Regular.ttf` (cmap parsed); U+21BB is present |
| stepper `＋` (U+FF0B) | `+` (U+002B) | U+FF0B is absent from the same font. The minus is **not** a substitution: U+2212 is present and drawn |
| dice cell `42 x 40` | floored by `_ptap()` | under 44 dp on both axes |
| segment chips `38`, steppers `38 x 38`, group header `50` | floored by `_ptap()` | same |
| `--warn #e0a840` | `accent` `#e0a34a`, via `world_workspace.gd::_pg_warn()` | **The stated reason has expired.** It was "a token can be remapped, a literal cannot" — but `DccTheme` now has `warn` `#e0a840` / light `#9a6a12`, exact to the canvas, so `c("warn")` would be both exact and remappable |
| `--chip rgba(255,255,255,.05)` | `Color(c("text_bright"), 0.05)` (`_pg_chip()`) | traces to a token, so light gets a 5 % black wash instead of an invisible white one |
| `+ ADD DRAFT STAMP (MOCK STROKE)`, `1px dashed` | `+ ARM SCULPT · DRAW ON THE MAP`, solid border | the prototype has no map to stroke; `StyleBoxFlat` has no dashed border |
| CANCEL during a run | **dashed, with its reason** | no cancellation entry point (§1.3) |
| `progPct` interpolated within a stage | whole stages | `GenerationProgress` reports a stage index and a count, not a fraction; a smooth bar over a value the engine does not produce is a fake measurement |
| group state `stale` / `resolved` | plus a third, **`no world`** | before the first generate neither is true |
| `last run · HH:MM` | `last run · N.N s`, `—` before a run | nothing records a wall-clock time; the elapsed total is measured (`_stage_elapsed_ms`) |
| — | `ADVANCED` sub-caption inside a group | the desktop's `ADVANCED_KEYS` fold has no canvas form; drawn inline, not dropped |
| `Min stream order`, `Ecotone sharpness`, `Rivers in biome view`, `Erosion strength`, `Working resolution`, `Archetype` | dashed or routed, each with a reason (`world_workspace.gd::PHONE_GEN_ABSENT`) | §1.6. **Two of those shipped reasons are stale:** `Min stream order` and `Rivers in biome view` still say the drawn rivers are a flow-area tint baked into the terrain raster, which stopped being true 2026-09-22; §1.6 items 4 and 6 give the current truth, and `Working resolution`'s reason quotes a "93-row table" that is now 99 |

### 1.12 Touch-gesture arbitration — the rule for phone and tablet

**Three control classes swallow a vertical scroll that starts on them**, each for
a different engine reason, so each has its own lever. All three classify at an
**8 dp slop** (Android's `ViewConfiguration.getScaledTouchSlop()`), then give the
gesture to the axis it travelled furthest along; a vertical verdict drives the
nearest vertical `ScrollContainer` (`DccWidgets.vertical_scroller_above()`).

| Class | Why it swallows | Lever | Where |
|---|---|---|---|
| `Slider` | `Slider::gui_input` writes the value on **touch-DOWN** (`set_as_ratio()` from the press position), before any motion exists to classify | `DccWidgets.PgSlider`, attached by `DccWidgets.touch_slider()` via `set_script()`: withholds the press in `_gui_input` (which runs before the C++ `gui_input`; `accept_event()` ends the chain), resolves at the slop; a tap keeps jump-to-the-tap, applied at release | `DccShell._touch_arbitrate()`, reached from `phone_fit()`; `phone_menu.gd::_slider_row()` (the phone menu is parented to `_phone_root`, not a dock, so `phone_fit()` never reaches it); `world_workspace.gd`'s `_pg_range_field()` / `_pg_sculpt_slider()` construct `PgSlider` directly |
| `BaseButton` (dropdowns, `DccWidgets.choice()`) | the press opens the popup under the finger; the drag travels its list and the release picks whatever is beneath — silent, exactly the slider defect | `DccWidgets.touch_release_button()`: `action_mode = ACTION_MODE_BUTTON_RELEASE` **and** `mouse_filter = PASS` — both load-bearing | `DccShell._touch_arbitrate()` |
| `LineEdit`, including a `SpinBox`'s internal field | `Viewport::_gui_input_event` grabs focus **before** `_gui_call_input`, so neither `accept_event()` nor `PASS` can stop it — the swipe focuses the field and raises the keyboard | `DccWidgets.PgField` via `touch_focus_field()`: parks `focus_mode` at `FOCUS_NONE`, classifies, takes focus at release on a tap, re-parks on `focus_exited` | `DccShell._touch_arbitrate()`, twice — a `SpinBox`'s field is an **internal** child that `get_children()` never returns |

**The tablet reaches the same `_touch_arbitrate()`** through
`DccShell.tablet_arbitrate()`, run once over the desktop shell and then by a
`node_added` hook
(`_on_tablet_node_added()` → `_run_tablet_dock_arbitrate()`) on anything
attached under either dock — a boot-time walk alone sees only chrome, since
every workspace panel arrives later. Gated on `is_tablet()`, with a unit of 1.0,
so the phone is never walked twice at two scales.

**Three gates on `PgSlider`**, found by widening it from three sliders to every
slider: `editable == false` falls back to stock (otherwise a disabled slider
becomes drag-writable); no vertical-scrolling ancestor falls back to stock
(nothing to arbitrate against); and `drag_started` is re-emitted when the
verdict resolves to *slider*, because the class suppresses the press Godot
emits it from and `civilization_workspace.gd`'s landmark-cap row reads it.

**Two costs, stated:** a fling that begins on a slider carries no inertia (the
scroll is written to `scroll_vertical`, since the `ScreenTouch` press that would
arm the scroller's own drag has been swallowed); and a gesture ending with the
value unchanged emits no `drag_ended`, so it cannot mark a stage stale. A field
tap puts the caret at end-of-text (4.7.1 exposes no pixel-to-column call) and a
phone field has no drag-select.

**`_family`** exists because `project.godot` leaves
`input_devices/pointing/emulate_mouse_from_touch` at its default `true`: one
finger delivers `ScreenTouch`/`ScreenDrag` **and** an emulated mouse pair, and
latching to the family that opened the gesture stops every delta counting twice.

**The slop is pinned from both directions.** `_nwsize_probe.gd` swipes a
perfectly vertical path, which covers the arbitration but cannot hold the slop
from below; `_rangeswipe_probe.gd::_jitter()` pivots first — `(3,1) (6,2)
(5,4)` px, further sideways than down — then runs 468 px down. Mutated
2026-09-07: slop `0` → 5 FAIL on `_jitter()` (reproducing the original defect,
`0.3 → 0.13`), `_nwsize_probe` still green; slop `400` → 5 FAIL on both.

**The census is a lower bound, and says which state it walked.**
`_rangeswipe_probe.gd --census-only` boots the phone shell, walks
`get_tree().root`, and reports every `Range` with its nearest scroller. (A
census of any class with internal children — a `SpinBox`'s field — must walk
`get_children(true)`; `MISTAKES.md`, "Walk a Godot scene tree to count
anything".) Measured 2026-09-07 at 1080×2340 on a **world-less** boot:
247 `Range`, 245 writable inside a live vertical scroller, 3 arbitrated — 242
hazards, 214 of them in the left-dock sheet — and one `phone_fit()` seam
converted all 218 sliders among them. A world-less boot hides the MORE ▸
Simulation sliders (`phone_menu.gd::_fill_sim()` draws `_missing_row("Year")`
when `tl_available()` is false), so walk named states, including a
world-loaded one: the dropdown census ran 22 → 38 → 44 across boot, PLAN and a
generate. The tablet measured 224 unarbitrated sliders at 800×1280 before its
hook existed.

**A fourth mechanism is separate and not solved by any of the above:** a
`MOUSE_FILTER_STOP` control (a `ColorPickerButton`, a `PanelContainer` —
`PanelContainer` defaults to `STOP` where other containers default to `PASS`)
ends the event walk before the scroller sees the press. `phone_fit()`'s PH-05
conversion excludes `PanelContainer` deliberately, because several carry their
own `gui_input`; triage the rest case by case (§6.6 has one).

**Observed on glass, cause not proven (2026-09-07):** a drag starting on the last
~30 physical px of a slider's drawn thumb at the 100 % stop moved nothing —
`center_grabber` lets the grabber overhang the `custom_minimum_size` that
`phone_slider()` sets.

**Invariants the GENERATE column relies on** (each found as a defect a desktop
probe could not see):

1. **Assert the boot state before tapping anything.** `_phone_tab` starts at
   `"gen"`, so GENERATE is lit with `_pick_phone_tab()` never called; the
   column is filled by a deferred `_refresh_phone_gen_panel()` at the foot of
   `register_workspace()`. A probe that taps the tab first hides a broken boot.
2. **The column's `mouse_filter` rule is reasserted on every rebuild**
   (`_pg_open_gestures()`): `IGNORE` for anything inert, `PASS` for
   `BaseButton`, `STOP` for `Range` (which then arbitrates as above). Group
   boxes and their padding, not the buttons, were what ate the drag.
3. **Every writer marks the stage stale** through
   `_pg_after_param_write(stage_index)` — slider, stepper and toggle alike.

---

## 2. MAP — `tabIsMap`

Template lines 188–241; state and handlers in `valsMap()`.

### 2.0 Which blocks are MAP's

The template is consistently indented, so `sc-if` nesting reads off it. Of the
nine blocks once filed under MAP, **two** are inside `tabIsMap` (`iconOn`,
`styleCustom`); the rest float over the map as siblings of the sheet, and split
by which `vals*` function feeds them:

| Block | Line | Indent | Where it lives | Fed by |
|---|---|---|---|---|
| `measureOn` | 102 | 12 | map canvas, outside the sheet | `valsMap()` — MAP's, §2.5 |
| `labelDraftOn` | 111 | 12 | map canvas, outside the sheet | `valsMap()` — MAP's, §2.5 |
| `wayOn` | 121 | 12 | map canvas, outside the sheet | `valsMap()` — MAP's, §2.5 |
| `undoOn` | 132 | 12 | map canvas, outside the sheet | `valsOver()` — §5 |
| `histOpen` | 138 | 16 | inside `undoOn` | `valsOver()` — §5 |
| `simStrip` | 148 | 12 | map canvas, outside the sheet | `valsOver()` — §5 |
| `coachOn` | 161 | 12 | map canvas, outside the sheet | `valsOver()` — §5 |
| **`tabIsMap`** | **188** | **16** | inside the sheet body | — |
| `iconOn` | 196 | 20 | inside `tabIsMap` | `valsMap()` |
| `styleCustom` | 223 | 22 | inside `tabIsMap` | `valsMap()` |

The three MAP overlays live outside the sheet because arming a MAP tool drops
the sheet to a peek (§2.2), so the tool's readout needs somewhere the sheet is
not. A byte offset taken with `grep -bo <symbol>` lands 16 bytes past the
`sc-if value="{{ ` that opens the block — another reason to cite by symbol.

### 2.1 Column

`tabIsMap`'s wrapper is `display:flex; flex-direction:column; gap:14px` inside
the sheet scroller (`flex:1; overflow-y:auto; overscroll-behavior:contain;
padding:2px 14px 24px`) — **`gap:14px`, not GENERATE's 12**. `sheetBack` is
always false for MAP, so the header draws only the title and the ✕. Four
top-level children, in order: TOOLS, LAYERS, STYLE, the closing footnote.

### 2.2 TOOLS — `mapTools`, `hTool`, `iconOn`

Caption: `TOOLS · ARMING DROPS THE SHEET TO A PEEK`, `9.5px mono`,
`letter-spacing:.2em`, `var(--dim)`, `padding:6px 2px 8px`. Chip row is
`display:flex; gap:8px; flex-wrap:wrap`.

`mapTools` is four entries and no more:

| id | label | glyph |
|---|---|---|
| `inspect` | `INSPECT` | `➤` |
| `measure` | `MEASURE` | `⟟` |
| `label` | `LABEL` | `⌖` |
| `icon` | `ICON` | `◇` |

Chip: `min-height:44px`, `padding:0 16px`, `border-radius:22px`, `gap:8px`,
`font:10px 'IBM Plex Mono'`, `letter-spacing:.12em`; text `"{glyph} {label}"`.
On/off from `chip(on)`: border `var(--acc)`/`var(--hair)`, ink
`var(--acc)`/`var(--sec)`, background `var(--wash)`/`var(--chip)`. `s.tool`
starts `'inspect'`.

`hTool` verbatim:

```
hTool: e => { const t = e.currentTarget.dataset.tool; this._haptic('arm');
  this.setState({tool:t, detent: t==='inspect' ? this.state.detent : 'peek'},
                () => this._snapSheet()) }
```

Three behaviours: a 10 ms haptic on arm; arming anything but Inspect
**collapses the sheet to `peek`**; Inspect leaves the detent alone.

`iconOn` = `s.tool === 'icon'`. When on, a variant strip appends under the
chips: `display:flex; gap:8px; padding-top:10px`, four cells each `48 × 44`,
`border-radius:16px`, `font:15px mono`, same `chip()` on/off, then a `flex:1`
hint `tap the map to stamp` in `9.5px mono var(--faint)`. `iconVars` is
`diamond ◇`, `circle ○`, `triangle △`, `square □`.

**Engine — all four tools are registered:** `inspect` (`app.arm_tool("inspect")`,
the default); `measure` (`global_tools.gd`'s handlers over
`EngineBridge.measure_begin` / `measure_add_point` / `measure_result` /
`measure_clear`, plus `measure_section` / `measure_area` / `measure_radius` /
`measure_vertical` for `GlobalTools.MEASURE_MODES`); `label`
(`cartography_workspace.gd::_on_label_click` / `_drag` / `_release`;
`EngineBridge.label_create(gx, gy, text)` is the canvas's `hLabelAdd`); `icon`
(`_on_icon_click` / `_drag` / `_release`; `EngineBridge.icon_arm(family,
variant, scale, rotation, jitter)`). Haptics: `DccShell._haptic("tool_arm")`.

**The four `iconVars` cannot be backed as drawn.** The engine's vocabulary is
`CartographyWorkspace.ICON_FAMILIES` — a family key plus a **positional** variant
index mirroring `cartalith-assets/src/slots.rs`'s `PACK_SETTLEMENT_SLOTS` /
`PACK_ICON_SLOTS` / `PACK_POI_SLOTS` (order is load-bearing:
`icon_bridge::resolve_variant` indexes by position). There is no
diamond/circle/triangle/square set, and `_arm_icon_from_ui()` returns early
unless `bridge.has_asset_pack()`. A four-cell strip is buildable as *the armed
family's first four slots*; with no asset pack loaded it has nothing to draw
and is dashed with that reason — a reason that must be true on the handset.

### 2.3 LAYERS — `layerGroups`, `hLayer`

Caption `LAYERS`, `9.5px mono`, `letter-spacing:.2em`, `var(--dim)`,
`padding:2px 2px 6px`. Each group is a `padding-bottom:6px` block with a name
row (`9px mono`, `letter-spacing:.18em`, `var(--faint)`, `padding:6px 2px 4px`)
over its rows.

Row: `display:flex; align-items:center; gap:12px; min-height:46px;
padding:0 12px; border-radius:14px`. Three spans — dot (`11px mono`), label
(`flex:1`), note (`9.5px mono var(--faint)`). On/off from `row()`: dot `●`/`○`,
dot ink `var(--acc)`/`var(--faint)`, label ink `var(--ink)`/`var(--body)`, row
background `var(--wash)`/`transparent`. **The note slot is always empty**:
`row(kind,id,label,on,note)` takes a fifth argument and all eight call sites
pass four. It is a slot, not content.

| Group | Row | `kind` | Engine id (`sample_bridge.rs::LAYER_GROUPS`) |
|---|---|---|---|
| `SURFACE · BASE` | Relief | `base` | `off` — "No overlay (base map)" |
| | Biome | `base` | `bclass` — "Biomes" |
| | Political | `base` | `control` — "Political control" |
| `TERRAIN FIELDS · OVERLAY` | Elevation | `ov` | `elevation` |
| | Slope | `ov` | `slope` |
| | Flow accumulation | `ov` | `flow` — "River flow" |
| `CLIMATE · OVERLAY` | Temperature | `ov` | `temp` |
| | Rainfall | `ov` | `rain` |

All eight exist, as a curated subset of `LayersPopover`'s full list. `hLayer`:
`kind==='base'` **sets** `s.base` (a radio); anything else **toggles**
`s.overlay` to the id or `null`.

**Where the models differ.** The canvas holds two slots — a base radio and a
nullable overlay; the engine has **one** overlay switch whose `off` value *is*
the base map. Relief / Biome / Political as a three-way radio over that switch
is faithful. It must not become a radio over `ViewportHost.set_layer_visible()`:
`provinces` and `territory` (`CartographyWorkspace.POLITICAL_LAYERS`) are
visibility flags on drawn furniture, a different mechanism, and the canvas's
"Political" is the `control` overlay.

**`Relief` is a name collision: Relief → `off`, never `relief`.** The engine
also has a row literally called `relief` ("Local relief",
`analysis::local_relief()`, max−min height over a 25 km window, in the Surface
group). Wiring by id would draw shaded local relief instead of the base map —
plausibly wrong rather than obviously broken, the expensive kind.

### 2.4 STYLE — `stylePresets`, `ramps`, `styleCustom`

Header row: `display:flex; align-items:baseline; gap:10px; padding:2px 2px 8px`;
`STYLE` in `9.5px mono`, `letter-spacing:.2em`, `var(--dim)`; when
`styleCustom` is set, `custom — edited since preset` in `9px mono var(--warn)`.

**Presets** — `display:flex; gap:8px; flex-wrap:wrap; padding-bottom:12px`; each
`min-height:42px`, `padding:0 16px`, `border-radius:21px`, `font:10px mono`,
`letter-spacing:.12em`, same `chip()` colours, lit only when
`s.stylePreset===p.id && !s.styleCustom`. Four ids: `atlas ATLAS`,
`parchment PARCHMENT`, `physical PHYSICAL`, `ink INK`.

**Ramps** — caption `COLOUR RAMP · TERRAIN`, `9px mono`, `letter-spacing:.18em`,
`var(--faint)`, `padding:0 2px 6px`. Column `gap:7px`. Row `min-height:44px`,
`padding:0 10px`, `border-radius:14px`, `gap:12px`, `border:1px solid`
`var(--acc)`/`var(--hair)`, background `var(--wash)`/`transparent`; swatch
`flex:1; height:14px; border-radius:7px`; name `10px mono`, `width:76px`, ink
`var(--acc)`/`var(--sec)`.

`hPreset` sets the preset and clears `styleCustom`; `hRamp` sets the ramp and
**sets** `styleCustom` — that is the whole "custom" model.

**Ramps: exact, nine for nine.** `render.rs::RAMP_PRESETS` is `Earth,
Elevation, Atlas, Mono, Imhof, Ice, Dark ice, Desert, Dark atlas` — the canvas's
`RAMPS` keys, in the same order (`EngineBridge.ramp_presets()`;
`load_ramp_preset(name)` applies one, `color_ramp()` returns its stops). **The
gradients are not the canvas's:** the canvas swatches are illustrative CSS hex
(`Earth` starts `#2c4a5e`; the engine's `Earth` starts `(152,168,116)`). Draw the
swatch from `color_ramp()`, never from the canvas's hex.

**Style presets: the shell ships named presets.**
`render_workspace.gd::STYLE_PRESETS` is `Natural Vibrant`, `Default`,
`Antique`, `Ink`, `Watercolor`, `Print`, `Village` — each an engine *look*
(`render.rs::LOOK_PRESETS`: `Quality tier`, `Natural Vibrant`, `Antique
Parchment`) plus an NPR bundle. Against the canvas's four: **INK → `Ink`**
(exact name); **PARCHMENT → `Antique`** (look `Antique Parchment`); **PHYSICAL →
`Natural Vibrant`** is a judgement, not a match; **ATLAS has no preset** — `Atlas`
exists only as a colour ramp. (An earlier version of this section said no named
style existed to select; `STYLE_PRESETS` already did.)

`styleCustom` is derivable: `appearance()` returns what the engine renders with,
and `reset_appearance()` returns **how many overrides it dropped** — exactly
"has this been edited since the preset".

### 2.4a The closing footnote

Canvas line 240, verbatim — the fourth and last top-level child of `tabIsMap`:

```html
<div style="font:9.5px/1.6 'IBM Plex Mono',monospace;color:var(--faint);padding:0 2px">Presentation only — nothing here alters world data or marks a generation stage stale.</div>
```

**Its `1.6` is the only line-height in the whole MAP block** — the kind of
single-instance value an inventory exists to carry. It was once missed because
the section that should have compared it quoted the *shipped* caption instead
of the canvas: compare the build to the canvas, never to itself.

### 2.5 The three MAP overlays drawn outside the sheet

Anchors: `chromeLeft = land ? 72 : 0`; `dockBottom = (land ? 14 : 98) + s.kb`;
`fabBottom = land ? 18 : 104`; `undoLeft = land ? 86 : 12`.

**`measureOn`** = `s.tool === 'measure'`. Pill at `top:92px`, `z-index:9`,
`margin:0 10px`, `gap:10px`, `padding:9px 13px`, `border-radius:16px`,
`background:{{pillBg}}`, `border:1px solid var(--hair)`. Contents: `MEASURE`
(`10px mono`, `.14em`, `var(--acc)`); `measTotal` (`11px mono var(--ink)`);
`{measN} pts · tap map to add` (`9.5px mono var(--dim)`); `CLEAR` (`10px mono
var(--sec)`, `padding:6px 4px`); `DONE` (`10px mono var(--acc)`, same padding).
`hMeasDone` disarms to `inspect`, clears the points and toasts `Measured X along
N segments`. The prototype's total is a straight `Math.hypot` sum through
`fmtKm()`.

**`labelDraftOn`** = `!!s.labelDraft`. Card at `bottom:{{dockBottom}}`,
`z-index:11`, `padding:0 12px`; inner `gap:10px`, `padding:10px 12px`,
`border-radius:18px`, `background:var(--pan)`, `border:1px solid var(--bord)`,
`box-shadow:0 8px 22px rgba(0,0,0,.35)`. `LABEL` (`9.5px mono .14em
var(--acc)`); an `<input>` at `flex:1`, `border-radius:12px`,
`padding:10px 12px`, `12px mono`, placeholder `label text…`; `✕` (`10px mono
var(--sec)`, `padding:8px 2px`); `ADD` (`500 10px mono .12em`, ink
`var(--accInk)` on `var(--acc)`, `border-radius:14px`, `padding:9px 13px`).
`hLabelAdd` trims, pushes an undo entry `label · <text>`, and drops the draft.

**`wayOn`** = `s.tool==='way' && s.wayDraft.length > 0`. Same anchor and card,
`gap:12px`, `padding:10px 14px`. `WAY`; `{wayN} pts · {wayLen}` (`11px mono
var(--ink)`); a `flex:1` spacer; `CANCEL` (`10px mono var(--sec)`); `COMMIT`
(styled as `ADD`). `hWayCommit` refuses under 2 points, pushes undo `way ·
<len>`, and toasts `Way committed · X — routes can now use it`.

**`way` is not one of the four `mapTools`.** `_tapAct` handles five tools —
`measure`, `label`, `icon`, `settlement`/`poi`, `way` — and MAP offers four; in
this build `way`, `settlement` and `poi` are CIVIL's. Do not add a fifth chip to
MAP on the strength of `_tapAct`.

**Engine:** `measure_*` as above; labels `label_create` / `label_move` /
`label_select` / `label_delete` / `label_clear_all` / `label_list`; ways
`way_begin(way_type)` / `way_append_point(gx, gy)` / `way_commit()` /
`way_discard()` — one-for-one with COMMIT and CANCEL. All three readouts are
backed.

### 2.6 Where each MAP surface's capability lives in the shell

A map from canvas surface to the code that already does the work — not a
statement of what the MAP sheet draws today, which is `STATUS.md`'s and
`_mapinv_probe.gd`'s to report.

| Canvas surface | Capability in the shell |
|---|---|
| TOOLS chips | each dock's TOOLS block, `DccWidgets.tools_block()`: `GLOBAL_TOOL_ENTRIES` (Inspect, Measure, Region select, Pan) plus the domain's own (CARTO: Icon, Label). On a phone the block stays in the dock — the top tool bar of Ruling AK is desktop and tablet only, and `06-phone.md` §6.3 has no top bar |
| `iconOn` variant strip | `cartography_workspace.gd::_build_icon_tool_options_row()` / `_build_icon_brush_controls()` |
| LAYERS | `LayersPopover`, opened from `ViewportHost._layers_btn` |
| STYLE presets | `render_workspace.gd::STYLE_PRESETS` (§2.4) |
| Colour ramp | the CARTO dock, over `bridge.ramp_presets()` |
| `styleCustom` note | derivable from `reset_appearance()` (§2.4) |
| Footnote | the MAP sheet's own caption line, written by `app.gd::_tool_options_cartography_default()` |
| `measureOn` pill | `right_dock.gd`'s `CTX_MEASURE` plus `section_strip.gd` |
| `labelDraftOn` card | `right_dock.gd`'s appended `TOOL_ANNO` section |
| `wayOn` card | `right_dock.gd`'s appended `TOOL_WAY` section; the way tool is armed from CIVIL |

The owner's report that prompted this inventory — *"Nothing from map, generate,
plan or more really leads to a deeper menu"* — is a findability claim. Measure
it by tapping only what is visible from launch, with a hit test at each
control's own rect, not by reading `.visible` (`_mapinv_probe.gd`, run as its
header prescribes; it aborts if its MAP tap does not land rather than
describing the wrong screen). A raised sheet can also cover
`ViewportHost._navpad`; the canvas resolves the same collision by anchoring its
FAB column at `fabBottom` above `dockBottom`, both of which move with the sheet.

### 2.7 Out of scope here

- **`undoOn`, `histOpen`, `simStrip`, `coachOn`** — `valsOver()`'s (§2.0), so
  §5's. Shell counterparts, for that lane: `DccShell._phone_undo_chip` (shown
  while `bridge.can_undo()`), `_phone_undo_pop` behind a
  `PHONE_UNDO_HOLD_SEC = 0.45` long-press, `_phone_sim_strip` with its speeds
  and transport, and `_maybe_show_coach_marks()` at boot. The canvas's
  `simStrip` slider is `min=-400 max=1200 step=1` with `×1 / ×10 / ×100` and a
  600 ms play tick.
- **The sample chip** (`chipRef` / `hChipTap` / `chipLine1` / `chipLine2`, line
  69) opens the inspector — §5's `inspOpen`.
- **The map canvas itself** — pan/pinch/rotate, the scale bar, `sampleData()`.
  The prototype's is a mock; comparing it with the real one is a different pass.

---

## 3. PLAN — `tabIsPlan` *(stub)*

Not yet inventoried. One correction the stub carries: **`stageRows` and
`resGroups` are PLAN's, not GENERATE's.** `stageRows` is the journey route's leg
list under `ROUTE · VHAL SERAI → PORT AMRE` (each row `st.dot`, `st.name`,
`st.sub`, `st.days`, `st.ovNote`), and `resGroups` is a PLAN sub-screen. The
GENERATE pipeline's list is `GENSTAGES` plus `progLog`.

## 4. MORE — `tabIsMore` *(stub)*

Not yet inventoried. The phone's MORE screens are built from
`06-phone.md` §6.6, not from this canvas, by the owner's 2026-09-05 ruling
(`LARGE_ITEM_RULINGS.md`, "Phone MORE — build the bespoke screens"): five
purpose-built screens — Project, Civilization, Data, Simulation, Preferences —
in `phone_menu.gd`, each row still resolving to a real `menus.gd` item. Read
that ruling before inventorying this block against the canvas.

## 5. Overlays *(stub)*

Not yet inventoried: `menuOpen`, `searchOpen`, `searchEmpty`, `inspOpen`,
`gridOn`, the `scrPicker` project picker (built as `phone_project_picker.gd`),
and the four `valsOver()` blocks §2.0 and §2.7 route here.

---

## 6. NEW WORLD modal — `modalOpen`

**The one place this file records the canvas being overruled, on the owner's
word.**

### 6.1 What the canvas draws

`NAME` (a text input), `SEED` with a `44 × 42` dice at radius 12, `EXTENT` as two
`min-height:42px` radius-14 chips, then `CANCEL` and `CREATE WORLD` at `46 dp` /
radius 23 with the create at `flex:1.4`. **No width field, no resolution, no
archetype.** Card `max-width:360px`, radius 22, `padding:18px 16px 16px`, over a
`rgba(0,0,0,.45)` scrim padded `22px`.

### 6.2 Why this build draws more

**Owner, 2026-09-07, holding the APK: *"even the initial or new map setup
doesn't allow for a km/size input"*, reported as a defect.** The canvas modal
and the card this shell then shipped agreed with each other; the owner overruled
both. `CLAUDE.md`'s standing rule settles it — an owner decision is newer than
any canvas. Recorded here and in `new_world_dialog.gd::_build()` so nobody
diffs it against the canvas and "fixes" it back.

### 6.3 Per control

| Control | On the card | Reason |
|---|---|---|
| **Map width** (preset) | **yes** | what the owner named. `width_km / grid_w` is the quotient every distance, grade, route length and settlement spacing derives from; creation-time only |
| **Width (km)** (free entry) | **yes** | the km face of the same value; `_refresh_dimensions()` keeps preset ↔ km in step both ways, so one value, not two sources |
| **Resolution** | yes | the other half of that quotient; `DCC_SHELL_SPEC.md` §2.1 names it for this command even though the canvas does not draw it |
| **Archetype** | **yes** | `request()["archetype"]` decides **which generation call runs**, which no GENERATE dial can express — and the phone had no other route to it (§1.6) |
| Grid columns | no | the same number as Resolution (`_on_resolution_selected` writes it, `_refresh_dimensions` writes the dropdown back); two views of one value in 360 dp is `MISTAKES.md`'s "second route to a control the user already has one to" |
| Aspect | no | the extent chips already pick the reference's two aspects (2:1 whole-world, 1.5625:1 otherwise; `_update_extent_state()`); the other five ratios are this port's addition, and a wrong one yields a differently shaped map, not a broken one |
| Grid rows | no | derived from aspect; editable only as a hand-typed override |
| Derived readout (Grid / Extent / Cell size / Aspect) | **yes** | what makes the pair legible: `Width (km) 40 075` over `Resolution 512` is 78 km per cell, and nothing else on the card says so |
| NAME | **no** | `request()` carries no name and `EngineBridge.generate()` takes none, so a name field would be a control the engine cannot back |

The card keeps **both** advisories: a smaller form is a smaller set of
controls; dropping a warning because the artboard is narrower is a different
thing.

### 6.4 Measured fit

`_nwsize_probe.gd`, four viewports, before the action row (§6.6) existed:

| Viewport | scale | screen | card | window content min |
|---|---|---|---|---|
| 1080×2340 | 2.621 | 412 dp | **360 × 688 dp** | 400 dp |
| 1440×3168 | 3.495 | 412 dp | 360 × 688 dp | 400 dp |
| 720×1600 | 1.748 | 412 dp | 360 × 688 dp | 400 dp |
| 380×800 | 1.000 | 380 dp | 336 × 689 dp | 384 dp |

Every tappable control cleared 44 dp **on both axes** at all four. **The action
row made the card 752 dp** against a 702 dp form viewport at 1080×2340 (716 at
1440×3168, 725 at 720×1600), so the card scrolls (§6.6).

At a 380 dp screen the window's content minimum is **4 dp over** (384 against
380). The card's own content wants 331 dp of a 336 dp card, so the cause is the
dialog chrome: `PHONE_CARD_INSET` counts the scrim padding but not the
`AcceptDialog`'s own margins (40 dp at 412, 48 at 380 — not a constant, so no
constant fixes it). Below any mainstream handset; left, because shrinking the
card on every device for a sub-380 dp case is the worse trade.

### 6.5 On glass

Driven on the OnePlus 6T (`9608b26b`, 1080×2340 @ 450 dpi) on 2026-09-07, every
act an `adb shell input` at coordinates read off a screencap: the card opens
with every control of §6.3 on it; a map width and `Archipelago` produced a world
of small islands whose `03 World structure` dials read the Archipelago preset —
the lifted control reaching `generate_world_structure_sized`, not merely
drawing; and the GENERATE sheet's `Archetype  New world ›` row opens the card
with the archetype on it. The full log is in commit `77f9194`. **A tap on a
button inside a phone-presented `AcceptDialog` is a claim only the handset can
make** — synthetic input cannot reach it on the desktop (§6.6).

### 6.6 The action row

Read off the canvas — the last `<div>` of the `modalOpen` card:

```
<div style="display:flex;gap:10px;padding-top:16px">
  <div … style="flex:1;min-height:46px;border-radius:23px;…
                color:var(--sec);background:var(--chip)">CANCEL</div>
  <div … style="flex:1.4;min-height:46px;border-radius:23px;…
                color:var(--accInk);background:var(--acc)">CREATE WORLD</div>
</div>
```

Both `font:500 10.5px 'IBM Plex Mono'`, `letter-spacing:.14em`.

**Built inside the card**, by `new_world_dialog.gd::_build_phone_actions()`,
with `AcceptDialog`'s own footer hidden (`get_ok_button().visible = false`, no
`add_cancel_button()`). `AcceptDialog` would otherwise put its primary first,
whatever the design says. `confirmed` is still the signal `_on_create()` hangs
off, so the desktop and tablet path is unchanged. Not `DccWidgets.phone_pill()`:
that is the 412 canvas's *other* button (48 dp, accent-filled or outlined), and
this row draws neither outline.

**Measured**, `_nwaction_probe.gd`, `fail=0` at 1080×2340 (2.621), 1440×3168
(3.495) and 720×1600 (1.748) — identical in dp, because `_fit_phone_card()`
clamps the card to `max-width:360px`:

| | canvas | measured |
|---|---|---|
| order | CANCEL left | CANCEL at x 39, CREATE WORLD at x 180 |
| `flex:1` : `flex:1.4` | 1.40 | 131 dp : 185 dp = **1.41** |
| `min-height` | 46 | **46.0 dp** both |
| `border-radius` | 23 | **23** both |
| `gap` | 10 px | **10.0 dp** |
| spans the card | — | 326 dp of a 360 dp card (16 dp padding each side) |
| footer | — | `AcceptDialog`'s OK button not in the visible set; no stray `Create`/`Cancel`/`Close` |

**Departures:**

| Canvas says | This build | Why |
|---|---|---|
| `font: 500 10.5px 'IBM Plex Mono'` | **11 px**, `DccTheme.mono(1, true)` | `add_theme_font_size_override()` takes an **int**; `.14em` of 10.5 px is 1.47 px of tracking, taken as a whole pixel |
| CANCEL ink `--sec` `#8d9296` | `#a9adb0` | **Not deliberate** — the name-for-name mapping §0.1 describes |
| CREATE WORLD ink `--accInk` `#16130c` | `#141005` | same root: the token map, not the row |

**The row surfaced a `PanelContainer` that was eating every drag.** The card
went 688 → 752 dp against a 702 dp viewport, putting CREATE WORLD 50 dp below
the fold, and the first on-glass swipe on the card moved **zero pixels**. The
`mouse_filter` chain from the card's last `Label` up to the scroller read
`Label=2 → VBoxContainer=1 → MarginContainer=1 → PanelContainer=0 →
CenterContainer=1 → VBoxContainer=1 → ScrollContainer=1` — the `0` (`STOP`) is
`_phone_card()`'s own `PanelContainer` (§1.12's fourth mechanism). Fixed at the
card (`_card.mouse_filter = MOUSE_FILTER_PASS`) and asserted as a chain with no
`=0`. Pinning the row *outside* the scroller was refused — it would sit below the
card's rounded panel, a bigger departure than 50 dp of scroll.

**What the probe cannot say.** Synthetic input cannot reach a control inside a
phone-presented `AcceptDialog` — `gui_get_hovered_control()` stays null at
`content_scale_factor` 2.62. So the probe opens the dialog by a staging call
and asserts the buttons on geometry and on having a `pressed` connection, never
on a finger pressing them; that is §6.5's, on glass.

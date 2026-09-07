# ANDROID_UI_SPEC.md — the phone's per-screen inventory

**Authority:** `design/Cartalith-Android-2026-09-07.dc.html` (170 327 bytes,
vendored 2026-09-07). Owner, 2026-09-07: *"The app only faintly resembles this
version … where it should resemble it 100% faithfully."*

**This file did not exist until 2026-09-07, and `.gd` comments were already
citing it.** Counted in the same edit that writes this line —
`grep -rn "ANDROID_UI_SPEC" --include=*.gd .` returns **14 occurrences in 4
files**: 11 in `dcc_shell.gd`, and one each in `phone_menu.gd`,
`phone_project_picker.gd` and `workspaces/world_workspace.gd` (the last is this
pass's own). Every one of them writes `docs/ANDROID_UI_SPEC.md`;
`DESIGN_HANDOFF.md`'s table puts the file at the repository root. Nothing was at
either path — `find . -name ANDROID_UI_SPEC.md` returned nothing and
`git log -- ANDROID_UI_SPEC.md` is empty, so those citations were pointing at a
file that had never been written. This is that file, at the root. The
`docs/`-prefixed citations are therefore **still wrong by one path segment**;
they are left for whoever owns those lines rather than mass-edited from a lane
that owns two of the four files.

**Scope of this pass:** the **GENERATE** surface only. Sections for MAP, PLAN
and MORE are stubs to be filled by the lanes that own them — extend this file,
do not replace it.

---

## 0. How to read the canvas

The canvas is one interactive prototype, not a set of artboards: a `<x-dc>`
document whose template lives at roughly bytes 3 500–75 000 and whose state and
handlers live at 75 000–170 000. Byte offsets below are from
`grep -bo <symbol> design/Cartalith-Android-2026-09-07.dc.html` on 2026-09-07 and
will drift if the file is re-vendored — **grep the symbol, never seek the
offset** (`MISTAKES.md`, "Re-resolve a citation late in a long pass").

### 0.1 Palette — the canvas's CSS variables against this shell's tokens

Read off the frame's inline `style` at byte 3 043. Every one verified against
`dcc_theme.gd`'s `DARK` dictionary in the same pass.

| Canvas var | Value | `DccTheme` token | Match |
|---|---|---|---|
| `--sur` | `#0d0e0f` | `bg` | exact |
| `--pan` | `#15171a` | `raised` `#17191a` | near; `panel` is `#121314` |
| `--pan2` | `#121314` | `panel` | exact |
| `--ink` | `#e8ebec` | `text_bright` | exact |
| `--body` | `#c8cbcd` | `text` | exact |
| `--sec` | `#8d9296` | `text_secondary` | exact |
| `--dim` | `#6f7478` | `text_dim` | exact |
| `--faint` | `#5f6468` | `text_faint` | exact |
| `--acc` | `#e0a34a` | `accent` | exact |
| `--accInk` | `#16130c` | `accent_ink` | exact |
| `--hair` | `rgba(255,255,255,.10)` | `line` | exact |
| `--hair2` | `rgba(255,255,255,.07)` | `line_soft` | exact |
| `--bord` | `rgba(255,255,255,.16)` | `border` | exact |
| `--wash` | `rgba(224,163,74,.14)` | `accent_wash` | exact |
| `--chip` | `rgba(255,255,255,.05)` | **no token** — used inline | — |
| `--chipOn` | `rgba(255,255,255,.10)` | **no token** | — |
| `--warn` | `#e0a840` | **no token** — `accent` is `#e0a34a`, 5 blue apart | — |
| `--block` | `#c26a60` | — | — |
| `--good` | `#8fae7d` | — | — |
| `--water` | `#7d9dae` | — | — |

`--chip` and `--warn` have no token. `--chip` is drawn as
`Color(DccTheme.c("text_bright"), 0.05)` and `--warn` as a literal, both stated
at their call sites. A literal colour cannot be remapped by `DccTheme.remap()`
for the light palette — recorded as an open item, not silently accepted.

### 0.2 Frame and detents

- Frame `412 × 892`, `border-radius:30px` (byte 2 852). `PHONE_REF_SHORT = 412.0`
  in `dcc_theme.gd` is the same number.
- `_detH(det)` (byte 89 252):
  `fh = frameH() − 84` portrait; `peek → 66`, `half → round(fh × 0.46)`,
  `full → fh − 96`. `DccShell._phone_detent_height()` already transcribes this.
- `navH = 84` portrait (bottom nav `height:84px`), `0` landscape.
- Landscape sheet: right-docked, `width = min(440, round(fw × 0.46))`,
  `border-radius:0`, `border-left:1px solid var(--hair)`.

### 0.3 Bottom nav — the tab row

Template at byte 63 887; values at byte 167 182.

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

`hTab` (byte 98 442) verbatim:

```
hTab: e => { const t = e.currentTarget.dataset.tab;
  this.setState(s => { if (s.tab === t) return {tab:null};
                       return {tab:t, detent: s.detent==='peek' ? 'half' : s.detent} }) }
```

`DccShell.PHONE_TABS` matches this list one-for-one and
`_pick_phone_tab()` implements the same detent rule.

### 0.4 Sheet header — and why GENERATE has none today

The prototype's sheet header (byte 19 700) is a `42 × 4` handle row, then a title
row `padding:0 14px 10px` carrying an optional `38 × 38` back circle, a title
(`font:500 11px mono; letter-spacing:.2em; color:var(--acc)`), a subtitle
(`9.5px mono; color:var(--dim)`) and a `38 × 38` close ✕.

`titles.gen = ['GENERATE', genMode==='pipe' ? 'pipeline · seed '+seed
: 'sculpt · draft stamps']` (byte 167 780).

**This shell's phone sheet has no header at all** — `_build_phone_tool_sheet()`
is the desktop tool-options bar with a grab handle and nothing else, and its own
comment says so at length. Building the header is a separate item; it is listed
in §1.9 below as owed, not claimed.

---

## 1. GENERATE — `tabIsGen`

Template bytes 26 332 – 39 559. State/handlers bytes 104 500 – 112 000.
Everything below is the canvas's own number.

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

**Engine:** `DccShell.select_domain_mode("world", …)` already carries a
per-domain mode, and `world_workspace.gd` builds `_sculpt_body` for the Sculpt
mode from `bridge.get_sculpt_features()` / `get_sculpt_presets()` /
`sculpt_get_globals()`. Both modes are real.

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

**Engine:** seed is `app.new_world_dialog.request()["seed"]`
(`new_world_dialog.gd::request()`), and the dice is `app._new_seed()` →
`new_world_dialog.randomise_seed()`. Both live. Note the dice button is
`42 × 40` in the canvas — **below the 44 dp touch floor on both axes**; this
build floors it with `_ptap()` and says so.

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

`GENSTAGES` (byte 75 717), all ten, in order: `Planet`, `Extent & scale`,
`World structure`, `Tectonics`, `Volcanism & impacts`, `Erosion`, `Hydrology`,
`Climate`, `Ecology & biomes`, `Resources & soils`. **Identical, in the same
order, to `world_workspace.gd::STAGES[*].name`** and to
`cartalith-engine/src/progress.rs::STAGE_NAMES`, which `_assert_stage_names()`
already guards.

**Engine:** `EngineBridge.generation_stage(index, name, total)` is a real
per-stage signal off `GenerationProgress`, plus `generation_started` /
`generation_finished`. Per-stage *elapsed seconds* are already tracked by
`world_workspace.gd`'s `_stage_elapsed_ms`.

**Engine cannot back — CANCEL.** `EngineBridge` has `generate()` and no
cancellation entry point; grep for `cancel` in `engine_bridge.gd` returns
nothing that stops a run. Drawn dashed with that reason. The prototype's own
cancel is `clearInterval` over a simulated timer, which is not a claim about
this engine.

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

Note the canvas's own em-dash convention for absence: `genLastRun` starts `'—'`.
That is the same "omit, do not fake" rule this project runs on.

**Engine:** `world_workspace.gd::_regenerate_live()` is the guarded Generate
(it prompts before discarding hand-authored work); `_stale_from_stage` and
`_stale_note_text()` are the stale model; `_mark_stale_from(stage_index)` is
already called by every parameter row's release handler.

### 1.5 `genGroups` — the collapsible parameter groups

Template byte 30 310, handler `hGenGroup` byte 110 661, data `_genFieldDefs()`
byte 104 800.

Box: `border:1px solid var(--hair2)`, `border-radius:18px`, `overflow:hidden`.

Header row — `min-height:50px`, `padding:0 14px`, `gap:10px`:

| Element | Spec |
|---|---|
| `g.num` | `9.5px mono`, `width:22px`, ink `g.numCol` = `var(--warn)` when stale else `var(--faint)` |
| `g.name` | `flex:1`, `font-size:12.5px`, ink `var(--ink)` |
| `g.state` | `9px mono`, `var(--faint)`; `'stale'` or `'resolved'` |
| `g.chev` | `11px mono`, `var(--faint)`; `'⌄'` open, `'›'` closed |

Body — `border-top:1px solid var(--hair2)`, `padding:6px 14px 12px`, `gap:2px`.

Field forms:

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

**Both steppers (38 × 38) and the segment chips (38 high) are below the
44 dp touch floor.** This build floors them and says so; the canvas figure is
recorded here as the canvas's, not as this build's.

### 1.6 The canvas's eight groups against this engine's 92 parameters

`_genFieldDefs()` in full, against `bridge.param_info()` read live on 2026-09-07
by `_genphone_probe.gd` (`PARAMS available=true count=92`, 9 groups).

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
| | Min stream order | 1–8 / 1 | — | — | **render filter** |
| 08 Climate | Equator temp | 10–40 / 1 / ` °C` | `climate.equator_temp` | 0–45 / 1 | live, wider |
| | Pole temp | −40–5 / 1 / ` °C` | `climate.pole_temp` | −50–10 / 1 | live, wider |
| | Rainfall | 0.2–2 / .05 / ` ×` | `climate.rain_k` | 0–2 / .01 | live, wider |
| 09 Ecology & biomes | Ecotone sharpness | 0–1 / .01 | — | — | **not parameterised** |
| | Rivers in biome view | toggle | — | — | **render toggle** |

**The four true absences, each opened at the symbol before it was written here.**

1. **Working resolution.** `params.rs`' `world` group holds exactly
   `world`, `sea_level`, `peak_m`, `carve_rivers`, `river_density`, `use_gpu` —
   no resolution key, and the live dump above confirms 92 keys with none.
   `world_workspace.gd::STAGES[1]["gap"]` states the rule: *"GRID HEIGHT IS A
   CALL ARGUMENT, NOT A PARAMETER"* (`main.gd`'s own rule). It is set in
   File ▸ New world, which is `app.open_new_world()`.
2. **Archetype.** `bridge.archetypes()` and `bridge.apply_archetype(name)` both
   exist and both work — but `new_world_dialog.gd::NOTE_CREATION_ONLY` is
   explicit that *"Extent, resolution and archetype reallocate every field in
   the pipeline, so changing them later means a fresh Create"*, and
   `request()["archetype"]` is what decides which generation call runs. So this
   is a **route, not a dash**: the row names the current archetype and opens
   File ▸ New world.
3. **Min stream order.** `STAGES[6]["gap"]`: *"Min stream order and lakes-as-water
   are reference render filters, not generation parameters — Cartography's
   map-mode work."*
4. **Ecotone sharpness / Rivers in biome view.** `STAGES[8]["gap"]`: *"Not
   parameterised. Biome classification runs off the finished
   elevation/temperature/rainfall fields with no dials of its own in
   cartalith-engine."* "Rivers in biome view" is a *render* toggle and belongs
   to Cartography's layer list.

**"Erosion strength" is a fifth, and it is subtler.** There is no engine key of
that name and no single key that means it. The engine's stage-06 group holds 28
parameters (`stream.*` and `passes.*`). Inventing one 0–1 dial and mapping it
onto several would be a second parameter table that can drift from the desktop's
— the exact defect this project keeps finding. So the group draws the engine's
own rows instead, and this line is recorded as a canvas field with no
counterpart rather than synthesised.

### 1.7 The group set — a stated deviation, not a silent one

The canvas draws **eight** groups (`01 02 03 04 06 07 08 09`) and footnotes:
*"Volcanism (05) and Resources (10) run with defaults."*

**That footnote is true of the prototype and false of this engine.** `params.rs`
files **9 parameters** under `group: "volcanism"` (`volc.count`, `volc.age`,
`volc.provinces`, `volc.exclude_transform`, `volc.edifice_model`,
`crater.count`, `crater.age`, `crater.physical_model`,
`crater.surface_age_myr`), every one of them live and every one of them already
on the desktop's stage-05 body. Hiding them to match the drawing would remove
nine working controls from the one screen the owner opened this batch to fix.

**Decision, stated so it can be reversed in one place:** this build draws
`world_workspace.gd::STAGES` — all ten — in the canvas's `genGroups` chrome, so
05 Volcanism & impacts is a group with nine rows and 09/10 are groups whose
bodies carry the engine's own `gap` prose instead of controls. The footnote is
rewritten to say what is true here. Everything about the *drawing* — the box,
the radii, the number column, the state caption, the chevron, the field forms —
is the canvas's.

Consequence, said plainly: the phone group list is **10 rows where the canvas
draws 8**, and the numbers `05` and `10` appear where the canvas has none.

### 1.8 Where the fields come from — one source, not two

Every row is built from the **desktop's own** parameter source:

- `EngineBridge.param_keys()` → the 92 dotted keys
- `EngineBridge.param_info(key)` → `{group, label, unit, type, min, max, step,
  reference_control}`
- `EngineBridge.param_get(key)` / `param_set(key, v)` / `param_default(key)`
- membership by `STAGES[i]["groups"]` (matched against `info.group`) and
  `STAGES[i]["keys"]` — the identical predicate
  `world_workspace.gd::_build_group_section()` uses

There is no second table. Adding a parameter is still one row in
`crates/cartalith-godot/src/params.rs` and no GDScript change, which is that
file's own stated contract.

`ADVANCED_KEYS` (9 keys the desktop tucks behind a disclosure) are drawn inline
at the foot of their group on the phone — the canvas has no disclosure form, and
dropping them would lose nine live controls.

### 1.9 SCULPT mode — `isSculpt`

Template byte 35 517. Contents, in order: a `GEOLOGICAL FEATURE` caption
(`9.5px mono`, `letter-spacing:.2em`, `var(--dim)`), a wrap of feature chips
(`min-height:42px`, `padding:0 13px`, `radius:21px`, a `14 × 14` SVG glyph then
the id), a hint line, a `PRESETS` caption and chip wrap (`min-height:38px`,
`radius:19px`, `9.5px mono`), a `BRUSH · GLOBAL` caption and range rows, a
dashed-border `＋ ADD DRAFT STAMP (MOCK STROKE)` button (`min-height:48px`,
`radius:24px`, `1px dashed var(--acc)`), then a stamp note with `DISCARD`
(`padding:11px 14px`, `radius:18px`, `var(--chip)`) and `✓ COMMIT`
(`padding:11px 16px`, `radius:18px`, `var(--acc)` on `var(--accInk)`), and a
closing prose line.

**Engine:** all of it is live — `bridge.get_sculpt_features()`,
`get_sculpt_presets()`, `sculpt_get_globals()`, `get_sculpt_globals_info()`,
`sculpt_stamp_count()`, `sculpt_set_feature()`, `sculpt_apply_preset()`, and
`world_workspace.gd::_build_sculpt()` builds exactly this on the desktop. The
canvas's `ADD DRAFT STAMP (MOCK STROKE)` is the prototype's stand-in for a map
stroke; this build arms the sculpt tool and closes the sheet instead, which is
what `phone_menu.gd::_go_civ_tool()` already does for the CIVIL tools.

**Owed, not claimed:** the phone sheet still has **no header row** (§0.4), so
`titles.gen`'s `GENERATE / pipeline · seed NNNNNN` subtitle is not drawn. That
is `_build_phone_tool_sheet()`'s own documented gap and is a separate item.

### 1.10 Tap path — launch to a generation slider

Stated because "it renders" and "it can be operated" are not "it can be found"
(`MISTAKES.md`, and the owner's own report):

1. Launch → project picker (`phone_project_picker.gd`) → pick or create a world.
2. The app screen shows the map, the app bar, and the bottom nav.
3. **Tap `GENERATE` in the bottom nav** — always on screen, `84 dp` tall,
   second of four cells.
4. `_pick_phone_tab("gen")` selects the `world` domain and lifts the sheet from
   `peek` (66 dp) to `half` (0.46 × usable height).
5. The sheet is now the GENERATE column: `PIPELINE | SCULPT`, `SEED`,
   `GENERATE WORLD`, then the ten parameter groups.
6. **Tap a group header** — e.g. `04 Tectonics` — and its sliders open under it.
7. Drag the sheet handle up for `full` if the group is long.

Nothing in that path is a gesture, a long-press, or an off-screen affordance.

### 1.11 What shipped, and every place it departs from the canvas

Built in `dcc_shell.gd` (`_build_phone_tool_sheet()`'s second column,
`_refresh_phone_gen_panel()`) and `workspaces/world_workspace.gd`
(`build_phone_generate()` and the `_pg_*` block). Verified by
`_genphone_probe.gd` at 720x1600 / 1080x2340 / 1440x3168 and 800x1280, and on
the handset `9608b26b` (ONEPLUS_A6013, 1080x2340 @ 450 dpi).

| Canvas says | This build | Why |
|---|---|---|
| 8 groups (`01-04, 06-09`) | **10**, the whole `STAGES` table | 05 Volcanism is 9 live parameters here; §1.7 |
| footnote "Volcanism (05) and Resources (10) run with defaults" | rewritten | false of this engine; §1.7 |
| `input[type=range]{height:22px}` | slider row floored to **44 dp** | probe measured 248.0 x 32.0 dp against the floor; the ± pair is already 44 |
| dice glyph `⚄` (U+2684) | `↻` (U+21BB) | U+2684 is absent from the shipped `IBMPlexMono-Regular.ttf` — cmap parsed 2026-09-07. U+21BB is present |
| stepper `＋` (U+FF0B) | `+` (U+002B) | same cmap check: U+FF0B is absent from the shipped font, so the ASCII plus stands in. **The minus is NOT a substitution and this row said it was, in the direction that hid a real defect** — U+2212 is present in the font and the canvas draws it, but the code shipped ASCII `-` for one revision while this table asserted U+2212 *"present and used"*. On glass that renders short and high beside a full-height `+`. Corrected 2026-09-07: the code now draws U+2212 and there is no minus deviation left to record |
| dice cell `42 x 40` | floored by `_ptap()` | under 44 dp on both axes |
| segment chips `38`, steppers `38 x 38`, group header `50` | floored by `_ptap()` | same |
| `--warn #e0a840` | `accent` `#e0a34a` | 5/6 units apart; a token can be remapped for the light palette, a literal cannot |
| `--chip rgba(255,255,255,.05)` | `Color(c("text_bright"), 0.05)` | traces to a token, so it becomes a 5% black wash on light instead of an invisible white one |
| `+ ADD DRAFT STAMP (MOCK STROKE)`, `1px dashed` | `+ ARM SCULPT · DRAW ON THE MAP`, solid border | the prototype has no map to stroke; `StyleBoxFlat` has no dashed border |
| CANCEL during a run | **dashed, with its reason** | `EngineBridge` exposes no cancellation entry point (grep `cancel`, 2026-09-07: `sculpt_cancel_stroke`, `label_cancel_edit`, nothing that stops a run) |
| `progPct` interpolated within a stage | whole stages | `GenerationProgress` reports a stage index and a count, not a fraction. A smooth bar over a value the engine does not produce is a fake measurement |
| group state `stale` / `resolved` | plus a third, **`no world`** | before the first generate neither of the two is true |
| `last run · HH:MM` | `last run · N.N s`, `—` before a run | nothing records a wall-clock time for the last generate; the elapsed total IS measured (`_stage_elapsed_ms`) |
| — (no equivalent) | `ADVANCED` sub-caption inside a group | the desktop's `ADVANCED_KEYS` fold has no canvas form; the 9 keys are drawn inline rather than dropped |

### 1.12 Three defects the desktop probe could not see

Recorded because they are the whole reason the on-glass half of this pass
exists, and because two of them were **invisible** to a probe that passed.

1. **The app booted to the old one-line strip.** `_phone_tab` starts at `"gen"`,
   so GENERATE is lit with `_pick_phone_tab()` never called, and
   `_select_domain()`'s refresh runs during shell build before any workspace is
   registered. The probe taps the tab — the exact act that hides this. Fixed by
   a deferred `_refresh_phone_gen_panel()` at the foot of `register_workspace()`,
   and the probe now asserts the boot state *before* it taps anything.
2. **A finger drag on the column scrolled nothing.** Only the ~14 dp gutter
   either side worked, because that x misses every child and lands on the
   `ScrollContainer`. The first fix set `MOUSE_FILTER_PASS` on the buttons and
   **changed nothing on glass** — the buttons were never the blocker; the group
   boxes and their padding were. `_pg_open_gestures()` now reasserts the rule
   over the whole subtree on every rebuild: `IGNORE` for anything inert, `PASS`
   for `BaseButton`, `STOP` for `Range`. The probe reached groups with
   `ensure_control_visible()` — a programmatic scroll that could never have
   caught this.
3. **A stepper wrote the engine without marking the stage stale.** Found by the
   probe, not on glass: `tect.plates` moved 14 → 15 with `_stale_from_stage`
   still `-1`. The slider's release handler had the stage index and the
   stepper's closure did not. All three writers now go through
   `_pg_after_param_write(stage_index)`.

### 1.13 On-glass evidence

Handset `9608b26b` (ONEPLUS_A6013), 1080x2340 @ 450, `--export-debug "Android"`
against the existing `target/aarch64-linux-android/android-dev/
libcartalith_godot.so` (no Rust changed this pass). `project.godot` md5
`ccba27c9280cf8373412e2ba87ed4054` before and after all three Godot invocations.
Every act below is `adb shell input tap/swipe` at coordinates read off an
`adb exec-out screencap`, never a call into the app.

- launch → project picker → `+ NEW WORLD` → `Create` → map, GENERATE lit,
  **sheet at peek already showing `PIPELINE | SCULPT`** (the boot fix)
- tap GENERATE → sheet at half: SEED `661593` + `↻`, `GENERATE WORLD`,
  `last run · 32.2 s`, `10 stages · dependency order`, `01 Planet · resolved`
  open with `Gravity 1.00g` and its `−`/`+` pair
- drag a slider → `1.40g`, stale note *"Stale from 01 Planet -- edited since the
  last Generate."*, button relabelled `REGENERATE 01 → 10`, the `01` number and
  every group caption gone stale, map badge `WORLD · EDITED`
- swipe from a group header → scrolls to `07 / 08 / 09 / 10` and the footnote
- tap `08 Climate` → opens with `North edge 55°`, `South edge 5°`,
  `Equator temperature 30°C`, `Pole temperature -25°C` — the engine's own
  labels and values
- tap `SCULPT` → 13 engine feature chips, the live registry hint, PRESETS;
  map badge `SCULPT · DRAFT`
- `adb logcat -d -s godot | grep -i "SCRIPT ERROR"` — none

---

## 2. MAP — `tabIsMap`  *(stub)*

Template bytes 21 103 – 26 332. Sub-blocks: `measureOn` (11 480),
`labelDraftOn` (12 499), `wayOn` (13 779), `undoOn` (14 929), `histOpen`
(15 632), `simStrip` (16 543), `coachOn` (17 986), `iconOn` (22 017),
`styleCustom` (24 392). Not inventoried by this pass.

## 3. PLAN — `tabIsPlan`  *(stub)*

Template bytes 39 559 – 57 003.

**Correction to the brief that dispatched this lane:** `stageRows` (byte 42 190)
and `resGroups` (byte 49 369) were described as GENERATE's pipeline list and
results. They are **not**. Both are inside `tabIsPlan`: `stageRows` is the
journey route's leg list under the heading `ROUTE · VHAL SERAI → PORT AMRE`,
each row carrying `st.dot`, `st.name`, `st.sub`, `st.days` and `st.ovNote`; and
`resGroups` is a PLAN sub-screen. The GENERATE pipeline's list is `GENSTAGES`
plus `progLog`, not `stageRows`. Not inventoried by this pass.

## 4. MORE — `tabIsMore`  *(stub)*

Template bytes 57 003 – 63 400. Not inventoried by this pass.

## 5. Overlays  *(stub)*

`menuOpen` (9 568), `searchOpen` (65 711), `searchEmpty` (66 863), `inspOpen`
(68 006), `modalOpen` (70 413), `gridOn` (62 182), plus the `scrPicker` project
picker (3 525 – 6 537). Not inventoried by this pass.

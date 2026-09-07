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

**Extended 2026-09-07 (second pass):** §1.14 (the slider gesture arbitration) and §6 (the NEW WORLD modal, and the one place this build deliberately departs from the canvas on an owner ruling). MAP, PLAN and MORE are still stubs.

**Extended 2026-09-07 (MAP lane):** §2 in full — `tabIsMap`'s four sections and
the three tool overlays it drives outside the sheet, each measured against the
engine symbol that would back it, plus a hit-tested reachability pass
(`_mapinv_probe.gd`). It corrects the stub it replaces on two counts: every byte
offset on this page anchors the `sc-if` attribute and so runs **16 low**, and
four of the nine blocks the stub filed under MAP are `valsOver()`'s and belong
to §5. **No shell code was written by that lane.** PLAN and MORE are still
stubs.

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
| `Min stream order` dashed as *"belongs to Cartography's map modes"* | dashed with the true reason | **the reason named a home that does not exist.** Corrected 2026-09-07: the drawn rivers are a flow-area tint inside the terrain raster (`render.rs`, `WET_AREA_LO`/`WET_AREA_HI` over upstream drainage area), not the polylines `get_rivers(min_order)` returns, so there is no order in the image to filter on. `STAGES[6]["gap"]` carried the same wrong home and was corrected with it |
| `Rivers in biome view` dashed as *"a Cartography layer option"* | dashed with the true reason | same class: `cartography_workspace.gd`'s own note says the biome view's rivers are that same baked tint *"with no parameter to switch it off"*. The row asserted the opposite of the file it pointed at |
| `Archetype` routed to *"File ▸ New world"* | the route now leads somewhere | **the reason was true of the desktop and false of the phone** — `archetype_input` was built into `new_world_dialog.gd`'s `struct_sec`, parented to the `rest` container that file hides on a handset. Closed by lifting the control onto the phone card (§6), not by rewording the row |

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

### 1.14 The slider gesture, and the floor that made it worse (2026-09-07)

**Found on glass on a verifier's first swipe: Ocean depth `0.60` → `0.14` in one
vertical gesture, stage 03 marked stale, and nothing on screen saying so.**

Two mechanisms, and the first is the one that surprises:

1. **Godot's `Slider` writes the value on touch-DOWN.** `Slider::gui_input`
   calls `set_as_ratio()` from the press position and only then arms its grab,
   so the jump had already happened before there was any motion to classify. A
   fix aimed only at the drag would have left it exactly as it was.
2. `_pg_open_gestures()` gives every `Range` `MOUSE_FILTER_STOP` — right for the
   horizontal drag, wrong for the vertical one — and **§1.11's own 44 dp floor
   widened the band that consumes it from 32 dp to 44 dp.** A correct change
   that made an existing hazard measurably worse. `MISTAKES.md`: *a floor is a
   hit area, and a bigger hit area catches more than you meant.*

**The fix is arbitration, not a smaller target.** `DccWidgets.PgSlider` —
written as an inner class of `world_workspace.gd` on 2026-09-07 and **moved to
`dcc_widgets.gd` the same day, once the census in §1.15 showed how narrow its
reach was** — holds the gesture until it has travelled `_pg_px(8)`, Android's own
`ViewConfiguration.getScaledTouchSlop()`, then gives it to the axis it travelled
furthest along. Vertical drives the ancestor `ScrollContainer.scroll_vertical`
and **never touches `value`**; horizontal writes exactly as before; a tap that
never resolves keeps Godot's jump-to-the-tap, applied at release once it is
known to be a tap. `_gui_input` is the seam that makes withholding possible:
`Control::_call_gui_input` runs the script's `_gui_input` **before** the C++
`Slider::gui_input`, and `accept_event()` aborts the rest of that chain.

Two costs, stated rather than hidden:

- **A fling that begins on a slider carries no inertia.** The scroll is driven
  by writing `scroll_vertical` rather than by re-propagating the event, because
  `ScrollContainer`'s own touch drag arms on the `InputEventScreenTouch` press —
  which by classification time has been swallowed. Every other pixel of the
  sheet keeps the native fling.
- **A gesture that ends with the value unchanged emits no `drag_ended`**, so it
  cannot mark a stage stale. A deliberate tightening, and half of what the
  report was about.

`_family` exists because `project.godot` leaves
`input_devices/pointing/emulate_mouse_from_touch` at its default `true`, so one
finger delivers `InputEventScreenTouch`/`ScreenDrag` **and** an emulated
`InputEventMouseButton`/`MouseMotion`; latching to the family that opened the
gesture is what stops every delta being counted twice.

Both slider sites in the `_pg_*` block were converted — `_pg_range_field()`'s
parameter rows and `_pg_sculpt_slider()`'s brush globals. **Named as symbols,
and not as a grep, deliberately:** this paragraph originally pasted
`grep -n "HSlider.new()" workspaces/world_workspace.gd` with a quoted count
of two, and that command stopped reproducing the moment it was written — the
substitution it documents removed both matches, so the only line it returns
today is the comment quoting its own string. A symbol survives the edit that
a command measuring the file cannot.

**The slop is now pinned from both directions, and it was not before.**
`_nwsize_probe.gd` swipes a *perfectly* vertical path (`delta.x == 0`), which is
a gesture no thumb makes; that leg **covers** the arbitration and cannot hold
the constant from below. `_rangeswipe_probe.gd::_jitter()` closes it: a path
whose first three samples travel further sideways than down —
`(3,1) (6,2) (5,4)` px, the pivot a thumb makes before it slides — then 468 px
straight down. Measured 2026-09-07 with a Python harness that replaces exact
literals at all four slop sites, restores in a `finally` and hashes every file
before and after (all four `SAME`, zero residue):

| slop | `_rangeswipe_probe` | `_nwsize_probe` |
|---|---|---|
| `8` (shipped) | **GREEN** | GREEN |
| mutated **down to `0`** (`touch_slider`'s own `maxf(1.0, …)` floor mutated to `0.0` with it, so it really is zero) | **5 FAIL** — `left dock: … leaves the value byte-identical (0.3 vs 0.13)`, no `drag_ended`, does scroll; and the GENERATE sheet's two | **GREEN** — reproducing exactly the gap this closes |
| mutated **up to `400`** | **5 FAIL** — the same five, now because nothing ever resolves and the sheet does not scroll | 5 FAIL |

`0.3 → 0.13` in the DOWN column is the original defect, reproduced by
measurement on a *different* surface from the one the owner hit. §1.14's earlier
sentence — *"covers, not pins"* — is discharged rather than restated. What
`_nwsize_probe.gd` still establishes on its own, at all four densities:
a vertical swipe starting **on** a slider leaves `planet.g` byte-identical,
scrolls, emits **zero** `drag_ended` and leaves `_stale_from_stage` at `-1`; a
horizontal drag on the same slider writes and emits **exactly one**. The engine
value is what is asserted, not the node's — a first parameter write legitimately
rebuilds the column and frees the slider under test.

### 1.15 The same hazard everywhere it lives — the census (2026-09-07)

§1.14 fixed **three sliders**. This is what a walk of the live tree found.

**How the inventory was taken**, because `MISTAKES.md` says an enumeration walks
the code and then asks the designs, never the other way round:

1. Every `Range` subclass constructed anywhere in `shell/`
   (`grep -n 'HSlider.new()\|VSlider.new()\|SpinBox.new()\|ProgressBar.new()'`),
   **plus the three factories a grep for `.new()` cannot see** —
   `DccWidgets.slider()`, `DccWidgets.number()` and
   `phone_menu.gd::_slider_row()`. Counted in the same edit that writes it:
   `grep -rn 'DccWidgets.slider(' shell/ --include=*.gd | grep -v
   ':[0-9]*:[[:space:]]*#' | wc -l` is **41**, and the same for `number(` is
   **13**, 2026-09-07.
2. Then `_rangeswipe_probe.gd --census-only`, which boots the real phone shell,
   walks `get_tree().root`, and for every `Range` reports its nearest
   `ScrollContainer` ancestor, that scroller's `vertical_scroll_mode`, whether
   the node carries the arbitration script, and which surface it is in. The
   probe navigates by tapping visible captions; the two exceptions
   (`_set_sheet_open`, `layers_popover.open()`) are staging calls and say so in
   the log.

**Before, at 1080×2340: 247 `Range` nodes. 245 of them are writable and sit
inside a live vertical scroller, and exactly 3 of those 245 arbitrate the
gesture — so 242 do not.** (The other two `Range`s have no scroller above them
at all and are not a hazard; the count is 247 once PLAN has built the planner's
form, 235 at boot before it exists.)

**247 is a LOWER BOUND taken in ONE state, and this paragraph did not say so
until a verifier measured it.** The census walks a **world-less** boot. On that
boot `tl_available()` is `false`, so `phone_menu.gd::_fill_sim()` draws
`_missing_row("Year")` and **all four `_slider_row()` sites contribute nothing**
— the probe reaches MORE ▸ Simulation and finds no `Range` there at all. Its own
census label used to read *"the Year slider's screen"*, which is a claim about
what was walked; it now names what was found. **So the four `_slider_row()`
sliders are converted on the strength of a code walk and are exercised by no
probe**, and a world-loaded boot would raise every number here. The 242 is what
was observed, not what exists.

| Surface | Count | Class | Hazard? |
|---|---|---|---|
| left-dock sheet | **214** | `HSlider` (`DccWidgets.slider()`) | **yes** — this is where the owner would hit it next |
| left-dock sheet (PLAN) | 12 | `SpinBox` (`DccWidgets.number()`) | no — measured, below |
| GENERATE sheet | 3 | `DccWidgets.PgSlider` | already fixed, §1.14 |
| `new_world_dialog.gd` | 2 + 5 | `HSlider` + `SpinBox` | **yes** (the sliders) |
| `asset_library_window.gd` | 1 + 2 | `HSlider` + `SpinBox` | **yes** (the scrolled slider) |
| `asset_library_window.gd` | 1 | `HSlider`, **no scroller** | **no** — nothing to arbitrate against |
| `layers_popover.gd` | 1 | `HSlider` | **yes** |
| a phone-presented `AcceptDialog` | 5 | `SpinBox` | no |
| phone root (`_phone_sim_slider`) | 1 | `HSlider`, **no scroller** | **no** |

**After: 24, and all 24 are `SpinBox`.** Every `Slider` inside a live vertical
scroller on the phone now arbitrates. Identical at every phone density measured
— 1080×2340 (`_phone_scale` 2.621), 1440×3168 (3.495) and 720×1600 (1.748) all
report 13 unarbitrated at boot, 25 once PLAN has built the planner form, 24 once
the Layers popover has been opened, and `fail=0` on every swipe leg.

**Three fixes, not one**, because no single seam reaches all of them:

- `DccShell.phone_fit()` calls `DccWidgets.touch_slider(ctl, 8.0 * unit)` beside
  the `phone_slider()` call it already made. **Of the 242, exactly 218 are
  `Slider`s** (214 in the left-dock sheet, 2 in the New World card, 1 in the
  asset library, 1 in the Layers popover) **and this one call converted all
  218**, because `_on_phone_node_added()` already routes every dock descendant
  through this walk and every phone-presented window is fitted by it. The other
  24 are `SpinBox`es — measured below, and not a hazard. `touch_slider()` attaches by `set_script()` and **skips any
  slider that already carries one**, so the `_pg_*` three are untouched.
- `phone_menu.gd::_slider_row()` attaches its own. `PhoneMenu` is parented to
  `_phone_root`, not to a dock, so `phone_fit()` never reaches it — checked, not
  assumed: `grep -n phone_fit shell/phone_menu.gd` is empty. Its Year cursor,
  Crowding dial and landmark caps all sit in `_screen_scroll`
  (`vertical_scroll_mode = AUTO`).
- `world_workspace.gd`'s two `_pg_*` builders keep constructing `PgSlider`
  directly; that sheet is not one `phone_fit()` walks either.

**Three gates were added to the class in the same edit**, because widening it
from 3 sliders to every slider on the phone raised questions the narrow version
never had to answer:

| Gate | Why | Would have broken |
|---|---|---|
| `editable == false` → stock behaviour | the original checked nothing | four surfaces disable a slider (`cartography_workspace.gd:2632`, `civilization_workspace.gd:4589`, `world_workspace.gd:1471`/`:1555`); a *disabled* slider would have become writable by a drag |
| no vertical-scrolling ancestor → stock behaviour | there is nothing to arbitrate against | the two `scroller=none` sliders in the table above would have lost their vertical drag for no gain |
| `drag_started` re-emitted when the verdict resolves to *slider* | the class suppresses the press that Godot emits it from | `civilization_workspace.gd:4590` is the shell's only `drag_started` consumer — it records whether a landmark-cap drag began at the `off` stop, and §2.1's *"drag up from `off` and the slider resumes at 40"* would have stopped working silently |

**`SpinBox` is a `Range` and is NOT a hazard here — measured, with a control.**
Godot's `SpinBox::gui_input` does step the value on press and drag it after, but
it only ever receives what its `LineEdit` child does not, and in this shell the
child covers the control. `_rangeswipe_probe.gd::_spinbox_leg()` measures the child's
rect and then drives a real press-and-jittered-swipe at 3, 8 and 18 px in from
the right edge of a live planner `SpinBox`.

**The strip is not zero-width** — `its LineEdit covers 899x154 of 917x154 —
uncovered strip 18 px wide` at 1440×3168, and `409x77 of 427x77`, again 18, at
720×1600 — so the taps at inset 3 and 8 land inside it. And still: `4.0 → 4.0`
on the swipe at all three insets, **and the plain tap at the same x does not
step the value either**. That tap is the control, and it is what turns "the
drag did nothing" into "the gesture does not reach `SpinBox::gui_input` on this
build". Reported as a finding, with its measurement; not as coverage, and not
with a cause this pass did not establish.

**On glass** — handset `9608b26b` (ONEPLUS_A6013), 1080×2340 @ 450 dpi,
`--export-debug "Android"` → `builds/android/Cartalith-slop.apk`, installed
sha256 `8839d339…3de8ada` matching the file (first build; the second build
after the `PanelContainer` fix was installed the same way). Boot state asserted
**before any input**: the project picker with `no saved worlds yet — create one
below`, `+ NEW WORLD`, `OPEN PROJECT .ZIP…`, `build b1d30162c8e9`. Every act
below is `adb shell input` at a coordinate read off an `adb exec-out screencap`;
nothing calls into the app.

The surface driven is the **Layers popover's `Opacity` slider** — a
`DccWidgets.slider()` row, i.e. one of the 218 this pass converted, not one of
the three §1.14 had already done.

| Gesture | Result |
|---|---|
| plain vertical swipe starting **on the track at 24% of its length**, `input swipe 720 1869 720 2280` | the popover **scrolls** and `Opacity` stays **100%**, thumb still at the right stop. Before this pass Godot's touch-DOWN `set_as_ratio()` would have written ~24% |
| **jittered** vertical swipe, `input motionevent DOWN/MOVE×12/UP` along `(3,1) (6,2) (5,4) … (-4,380)` — the on-glass form of `_jitter()` | same: scrolls, `Opacity` **100%** |
| horizontal drag on the same slider, `input swipe 820 1869 700 1869 600` | **100% → 5%**, and the diff bounding box is `(645,1840)-(1002,1898)` — the slider row and nothing else |
| horizontal drag back, `700 → 900` | **5% → 100%** |

**One observation from that pass, reported and not fixed**, because it is
`phone_slider()`'s and not this hazard's: a drag beginning at x ≥ **870**
physical px moved **zero pixels** (`ImageChops.difference` bbox `None`) while
one at 820 wrote. The drawn track+thumb spans 663–900 px at the 100% stop, so
the last ~30 px of the drawn thumb is outside the control's own 78 dp hit area —
`center_grabber` lets the grabber overhang the `custom_minimum_size` the factory
sets. Measured, cause not proven; recorded here rather than attributed.

**Not fixed, and why:**

- **Tablet — and it is not a small number.** The same probe at 800×1280
  reports `phone=false tablet=true` and **224 unarbitrated writable `Range`
  nodes inside a live vertical scroller**, before and after. `tablet_fit()` is
  the tablet walk and it has exactly two call sites (`dcc_shell.gd`'s
  `tool_options_row`, and one in `app.gd`), so it is not the seam `phone_fit()`
  is — nothing routes a tablet's dock content through it. Attaching at
  `DccWidgets.slider()` on `is_touch()` would reach most of them in one line,
  and is deliberately **not** done here: it would change tablet behaviour this
  pass has measured the hazard on but not the fix, and `GUI_GAP_REGISTER.md` §57
  refuted an earlier `is_touch()` reach-across on exactly that ground. Filed
  with its number so the next pass has one.
- **The 24 `SpinBox`es**, on the measurement above.
- **`layers_popover.gd`'s slider needed nothing of its own.** The boot census
  showed it unarbitrated, which is a real state and a transient one: that file's
  `_phone_fit()` returns early until `phone_present()` has run inside `open()`.
  Opening the popover in the probe drops the count 25 → 24. Recorded because
  "the census saw it red" would otherwise read as a gap.

---

## 2. MAP — `tabIsMap`

**Inventoried 2026-09-07.** Template lines 188–241 of
`design/Cartalith-Android-2026-09-07.dc.html`; state and handlers in
`valsMap()`, line 905. Every number below is the canvas's own, read off the
element it is written on.

### 2.0 The offsets in this section's own stub were wrong by a constant, and four of the nine blocks are not MAP's

The stub this section replaces listed `tabIsMap` at bytes 21 103–26 332 with
nine sub-blocks. Both halves of that needed correcting, and the corrections are
different in kind.

**The number.** Every stub offset is exactly **16 bytes low**, uniformly, from
`menuOpen` (9 568) to `modalOpen` (70 413). It is not a unit error — both
figures are byte offsets in the same UTF-8 file — it is a different **anchor**:
the stub reports the start of the `sc-if value="{{ ` attribute, and
`grep -bo <symbol>` reports the start of the symbol token, and
`sc-if value="{{ ` is 16 characters. Verified by
`dd bs=1 skip=21103 count=45`, which reads `sc-if value="{{ tabIsMap }}" hint-`.
So `tabIsMap`'s guard opens at byte 21 103 and its symbol at 21 119; **this
section cites line numbers instead**, per §0's own "grep the symbol, never seek
the offset".

**The structure, which the numbers were hiding.** Seven of the nine listed
sub-blocks sit at byte offsets *below* `tabIsMap`'s own start, and that is
because they are **not inside it**. Indentation resolves it — the template is
indented consistently and `sc-if` nesting reads straight off it:

| Block | Line | Indent | Where it really lives |
|---|---|---|---|
| `measureOn` | 102 | 12 | map canvas, **outside** the sheet |
| `labelDraftOn` | 111 | 12 | map canvas, outside the sheet |
| `wayOn` | 121 | 12 | map canvas, outside the sheet |
| `undoOn` | 132 | 12 | map canvas, outside the sheet |
| `histOpen` | 138 | 16 | inside `undoOn` |
| `simStrip` | 148 | 12 | map canvas, outside the sheet |
| `coachOn` | 161 | 12 | map canvas, outside the sheet |
| **`tabIsMap`** | **188** | **16** | inside the sheet body |
| `iconOn` | 196 | 20 | **inside `tabIsMap`** |
| `styleCustom` | 223 | 22 | **inside `tabIsMap`** |

So only **two** of the nine are `tabIsMap` sub-blocks. The other seven are
floating chrome over the map surface, siblings of the sheet, and they split
again by *which* `vals*` function feeds them:

- **`measureOn`, `labelDraftOn`, `wayOn` are MAP's** — `valsMap()` computes all
  three from `s.tool` and `s.labelDraft` / `s.wayDraft`. They are drawn outside
  the sheet because arming a MAP tool drops the sheet to a peek (below), so the
  tool's own readout has to live somewhere the sheet is not. **Inventoried
  here**, §2.5, because MAP owns their state even though MAP does not contain
  them.
- **`undoOn`, `histOpen`, `simStrip`, `coachOn` are not MAP's at all** —
  `valsOver()` computes them from `s.undoStack`, `s.sim` and `s.coach`, none of
  which MAP touches. They belong in **§5 Overlays** with `menuOpen`,
  `searchOpen` and `inspOpen`, and are **out of scope here**; §2.7 records only
  what this build already has for them, so the §5 lane starts from a fact.

### 2.1 Column and header

`tabIsMap`'s own wrapper is `display:flex; flex-direction:column; gap:14px`
inside the sheet scroller (`flex:1; overflow-y:auto; overscroll-behavior:contain;
padding:2px 14px 24px`). **Note `gap:14px`, not GENERATE's `12px`.**

Sheet header (§0.4's shared block, line 176): `titles.map = ['MAP',
'layers · style · annotation']` (line 1466). `sheetBack` is
`(tab==='more' && moreDepth) || (tab==='plan' && planDepth)` — **false for MAP
always**, so MAP never draws a back circle, only the title and the ✕.

### 2.2 TOOLS — `mapTools`, `hTool`, `iconOn`

Caption: `TOOLS · ARMING DROPS THE SHEET TO A PEEK`, `9.5px mono`,
`letter-spacing:.2em`, `var(--dim)`, `padding:6px 2px 8px`. Chip row is
`display:flex; gap:8px; flex-wrap:wrap`.

`mapTools` (line 907) is four entries and no more:

| id | label | glyph |
|---|---|---|
| `inspect` | `INSPECT` | `➤` |
| `measure` | `MEASURE` | `⟟` |
| `label` | `LABEL` | `⌖` |
| `icon` | `ICON` | `◇` |

Chip: `min-height:44px`, `padding:0 16px`, `border-radius:22px`, `gap:8px`,
`font:10px 'IBM Plex Mono'`, `letter-spacing:.12em`; text is `"{glyph} {label}"`.
On/off from `chip(on)` (line 906): border `var(--acc)`/`var(--hair)`, ink
`var(--acc)`/`var(--sec)`, background `var(--wash)`/`var(--chip)`. `s.tool`
starts `'inspect'` (line 714).

`hTool` verbatim (line 922):

```
hTool: e => { const t = e.currentTarget.dataset.tool; this._haptic('arm');
  this.setState({tool:t, detent: t==='inspect' ? this.state.detent : 'peek'},
                () => this._snapSheet()) }
```

**Three behaviours in one line, and the caption names the middle one:** a
haptic on arm (`_haptic('arm')` = 10 ms, line 749); arming anything but Inspect
**collapses the sheet to `peek`**; and Inspect leaves the detent alone.

`iconOn` = `s.tool === 'icon'` (line 919). When on, a variant strip appends
under the chips: `display:flex; gap:8px; padding-top:10px`, four cells each
`48 × 44`, `border-radius:16px`, `font:15px mono`, same `chip()` on/off, then
a `flex:1` hint reading `tap the map to stamp` in `9.5px mono var(--faint)`.
`iconVars` (line 908) is `diamond ◇`, `circle ○`, `triangle △`, `square □`.

**Engine.** All four tools are real and all four are registered:

- `inspect` — `app.gd`'s default; `app.arm_tool("inspect")`.
- `measure` — `global_tools.gd` registers click/escape/backspace handlers for
  it; `EngineBridge.measure_begin` / `measure_add_point` / `measure_result` /
  `measure_clear`, plus `measure_section` / `measure_area` / `measure_radius` /
  `measure_vertical` for the four modes in `GlobalTools.MEASURE_MODES`.
- `label` — `cartography_workspace.gd::_on_label_click` / `_on_label_drag` /
  `_on_label_release`; `EngineBridge.label_create(gx, gy, text)` is exactly the
  canvas's `hLabelAdd`.
- `icon` — same file's `_on_icon_click` / `_on_icon_drag` / `_on_icon_release`;
  `EngineBridge.icon_arm(family, variant, scale, rotation, jitter)`.

**Engine cannot back the four `iconVars` as drawn.** The canvas's vocabulary is
four abstract shapes; this engine's is
`CartographyWorkspace.ICON_FAMILIES` — a family key plus a **positional**
variant index, mirroring `cartalith-assets/src/slots.rs`'s
`PACK_SETTLEMENT_SLOTS` / `PACK_ICON_SLOTS` / `PACK_POI_SLOTS`, whose order is
load-bearing (`icon_bridge::resolve_variant` indexes by position). There is no
diamond/circle/triangle/square set anywhere in it, and
`_arm_icon_from_ui()` returns early unless `bridge.has_asset_pack()`. A
four-cell strip is buildable as *the armed family's first four slots*; it is
not buildable as the canvas's four shapes. **With no asset pack loaded the
strip has nothing to draw at all** — dash it with that reason, and the reason
has to be true on the handset, which is the case §0's own rule was written for.

### 2.3 LAYERS — `layerGroups`, `hLayer`

Caption `LAYERS`, `9.5px mono`, `letter-spacing:.2em`, `var(--dim)`,
`padding:2px 2px 6px`. Each group is a `padding-bottom:6px` block with a name
row (`9px mono`, `letter-spacing:.18em`, `var(--faint)`, `padding:6px 2px 4px`)
over its rows.

Row: `display:flex; align-items:center; gap:12px; min-height:46px;
padding:0 12px; border-radius:14px`. Three spans — dot (`11px mono`), label
(`flex:1`), note (`9.5px mono var(--faint)`). On/off from `row()` (line 909):
dot `●`/`○`, dot ink `var(--acc)`/`var(--faint)`, label ink
`var(--ink)`/`var(--body)`, row background `var(--wash)`/`transparent`.

**The note slot is drawn and always empty.** `row(kind,id,label,on,note)` takes
a fifth argument and all eight call sites pass four, so `r.note` is `''`
everywhere in the prototype. It is a slot, not content.

`layerGroups` (line 910), and what each row resolves to in this engine —
**every one measured live** by `_mapinv_probe.gd` §6 against
`LayersPopover._rows`:

| Group | Row | `kind` | Engine id in `sample_bridge.rs::LAYER_GROUPS` | Present |
|---|---|---|---|---|
| `SURFACE · BASE` | Relief | `base` | `off` — "No overlay (base map)" | yes |
| | Biome | `base` | `bclass` — "Biomes", `CART_BIOME_COLS` | yes |
| | Political | `base` | `control` — "Political control" | yes |
| `TERRAIN FIELDS · OVERLAY` | Elevation | `ov` | `elevation` | yes |
| | Slope | `ov` | `slope` | yes |
| | Flow accumulation | `ov` | `flow` — "River flow" | yes |
| `CLIMATE · OVERLAY` | Temperature | `ov` | `temp` | yes |
| | Rainfall | `ov` | `rain` | yes |

Eight for eight. The popover carries **43** rows in total, so the canvas's
eight are a curated subset, not the whole picker.

`hLayer` (line 925): `kind==='base'` **sets** `s.base` (a radio); anything else
**toggles** `s.overlay` to the id or to `null`. Both set `this.dirty`.

**The one place the canvas's model and the engine's do not line up.** The canvas
holds *two independent slots* — a base radio and a nullable overlay — and this
engine has **one** overlay switch whose `off` value *is* the base map. Relief /
Biome / Political as a three-way radio over that single switch is faithful.
What it must not become is a radio over `ViewportHost.set_layer_visible()`:
`provinces` and `territory` (`CartographyWorkspace.POLITICAL_LAYERS`) are
*visibility flags on drawn furniture*, a different mechanism from an overlay,
and the canvas's "Political" is the `control` overlay, not those two.

### 2.4 STYLE — `stylePresets`, `ramps`, `styleCustom`

Header row: `display:flex; align-items:baseline; gap:10px; padding:2px 2px 8px`;
`STYLE` in `9.5px mono`, `letter-spacing:.2em`, `var(--dim)`; and when
`styleCustom` is set, `custom — edited since preset` in `9px mono var(--warn)`.

**Presets** — `display:flex; gap:8px; flex-wrap:wrap; padding-bottom:12px`;
each `min-height:42px`, `padding:0 16px`, `border-radius:21px`,
`font:10px mono`, `letter-spacing:.12em`, same `chip()` colours, and lit only
when `s.stylePreset===p.id && !s.styleCustom`. Four ids: `atlas ATLAS`,
`parchment PARCHMENT`, `physical PHYSICAL`, `ink INK`.

**Ramps** — caption `COLOUR RAMP · TERRAIN`, `9px mono`,
`letter-spacing:.18em`, `var(--faint)`, `padding:0 2px 6px`. Column
`gap:7px`. Row `min-height:44px`, `padding:0 10px`, `border-radius:14px`,
`gap:12px`, `border:1px solid` `var(--acc)`/`var(--hair)`, background
`var(--wash)`/`transparent`; swatch `flex:1; height:14px; border-radius:7px`;
name `10px mono`, `width:76px`, ink `var(--acc)`/`var(--sec)`.

`hPreset` sets the preset and clears `styleCustom`; `hRamp` sets the ramp and
**sets** `styleCustom` — that is the whole of the "custom" model.

**Engine — ramps: exact, nine for nine.** `render.rs::RAMP_PRESETS` is
`Earth, Elevation, Atlas, Mono, Imhof, Ice, Dark ice, Desert, Dark atlas` and
the canvas's `RAMPS` keys are the same nine strings **in the same order**.
Read back live through `EngineBridge.ramp_presets()` on 2026-09-07:
`["Earth", "Elevation", "Atlas", "Mono", "Imhof", "Ice", "Dark ice", "Desert",
"Dark atlas"]`. `load_ramp_preset(name)` applies one; `color_ramp()` returns the
stops. **The gradients are not.** The canvas's swatches are illustrative CSS
hex; the engine's stops are different values (`Earth` starts `(152,168,116)`,
the canvas `#2c4a5e`). Draw the swatch from `color_ramp()` — never from the
canvas's hex, which would put a picture of a ramp above the ramp it selects.

**Engine — style presets: two of four have a name, and no build ships any.**
The engine's named looks are `render.rs::LOOK_TIER` / `LOOK_VIBRANT` /
`LOOK_ANTIQUE` — three, read back live as
`["Quality tier", "Natural Vibrant", "Antique Parchment"]`
(`EngineBridge.looks()` / `look()` / `set_look()`). PARCHMENT maps to
`Antique Parchment`; PHYSICAL is plausibly `Natural Vibrant`, and that is a
judgement, not a match. **ATLAS and INK have no engine look, and NPR is not
one either** — `set_npr()` takes a key/value `Dictionary`, not a named style,
so there is no `"Ink"` to select. The user-preset API does exist
(`save_appearance_preset` / `load_appearance_preset` / `peek_appearance_preset`
over `user://appearance_presets`, `preset_api == true` measured), so four named
presets are *shippable* — but nothing ships them today, so as of this pass two
of the four chips have nothing to select and must be dashed with that reason.

`styleCustom` is derivable rather than absent: `appearance()` returns what the
engine is rendering with, and `reset_appearance()` returns **how many overrides
it dropped**, which is exactly "has this been edited since the preset".

### 2.5 The three MAP overlays drawn outside the sheet

All three are `valsMap()`'s, all three sit over the map canvas, and all three
exist because `hTool` drops the sheet to a peek when a tool arms. Anchors:
`chromeLeft = land ? 72 : 0`; `dockBottom = (land ? 14 : 98) + s.kb`;
`fabBottom = land ? 18 : 104`; `undoLeft = land ? 86 : 12`.

**`measureOn`** = `s.tool === 'measure'` (line 102). Pill at `top:92px`,
`z-index:9`, `margin:0 10px`, `gap:10px`, `padding:9px 13px`,
`border-radius:16px`, `background:{{pillBg}}`, `border:1px solid var(--hair)`.
Contents: `MEASURE` (`10px mono`, `.14em`, `var(--acc)`); `measTotal`
(`11px mono var(--ink)`); `{measN} pts · tap map to add` (`9.5px mono
var(--dim)`); `CLEAR` (`10px mono var(--sec)`, `padding:6px 4px`); `DONE`
(`10px mono var(--acc)`, same padding). `hMeasDone` disarms to `inspect`,
clears the points and toasts `Measured X along N segments`. The prototype's
total is a straight `Math.hypot` sum in world units through `fmtKm()`.

**`labelDraftOn`** = `!!s.labelDraft` (line 111). Card at `bottom:{{dockBottom}}`,
`z-index:11`, `padding:0 12px`; inner `gap:10px`, `padding:10px 12px`,
`border-radius:18px`, `background:var(--pan)`, `border:1px solid var(--bord)`,
`box-shadow:0 8px 22px rgba(0,0,0,.35)`. `LABEL` (`9.5px mono .14em
var(--acc)`); an `<input>` at `flex:1`, `border-radius:12px`, `padding:10px 12px`,
`12px mono`, placeholder `label text…`; `✕` (`10px mono var(--sec)`,
`padding:8px 2px`); `ADD` (`500 10px mono .12em`, ink `var(--accInk)` on
`var(--acc)`, `border-radius:14px`, `padding:9px 13px`). `hLabelAdd` trims,
pushes an undo entry `label · <text>`, and drops the draft.

**`wayOn`** = `s.tool==='way' && s.wayDraft.length > 0` (line 121). Same anchor
and card, `gap:12px`, `padding:10px 14px`. `WAY`; `{wayN} pts · {wayLen}`
(`11px mono var(--ink)`); a `flex:1` spacer; `CANCEL` (`10px mono var(--sec)`);
`COMMIT` (styled as `ADD`). `hWayCommit` refuses under 2 points, pushes undo
`way · <len>`, and toasts `Way committed · X — routes can now use it`.

**Note `way` is not one of the four `mapTools`.** `_tapAct` (line 813) handles
five tools — `measure`, `label`, `icon`, `settlement`/`poi`, `way` — and the MAP
chip row offers four of them. In the prototype `way`, `settlement` and `poi`
are armed from elsewhere; in this build they are CIVIL's, which is where MORE
already puts them. Do not add a fifth chip to MAP on the strength of `_tapAct`.

**Engine.** `measure_*` as above. Labels: `label_create` / `label_move` /
`label_select` / `label_delete` / `label_clear_all` / `label_list`. Ways:
`way_begin(way_type)` / `way_append_point(gx, gy)` / `way_commit()` /
`way_discard()` — a one-for-one match for COMMIT and CANCEL. All three
readouts are backed; **none of the three floating cards exists**, because this
build puts every one of those readouts in the right dock instead (§2.6).

### 2.6 What this build has today, and where — measured, not read off the code

`_mapinv_probe.gd` / `_mapinv_probe.tscn` (this lane's own, added at the
`godot-project` root), run headless at 1080 × 2340 with `--force-touch`.
Reachability is measured by **hit-testing the GUI tree** at each control's own
centre, not by reading `.visible`: a control can be visible and still sit under
an opaque sibling.

Boot state: tab `gen`, detent `peek`, tool `inspect`, domain `world`. 31
pressables in the tree, **13 finger-reachable and enabled**.

Tapping MAP (through the real GUI tree, on the tab cell's own rect): tab `map`,
detent `half`, domain `cartography`, tool still `inspect`, sheet **609.0 px =
232.32 dp**, tool scroller shown, GENERATE column hidden. And then:

> **The MAP sheet contains two `Label`s and three `Control`s. That is the whole
> of it.** `CARTOGRAPHY · STYLE`, and a paragraph beginning *"presentation only
> — no control here marks a generation stage stale…"*.

That text is `app.gd::_on_workspace_changed()`'s `"cartography"` arm —
`_tool_options_simple("CARTOGRAPHY · " + active_mode("cartography").to_upper(),
…)` — matched against the probe's captured strings, not
`cartography_workspace.gd::_show_style_tool_options()`, which writes a *shorter*
near-duplicate and fires from `_on_any_tool_armed`'s default branch instead. Two
writers, one row; tapping MAP is a domain switch, so `app.gd`'s wins. Either
way it is the canvas's closing footnote **with the four sections above it
missing**, and it is verbatim the owner's *"one line that barely scrolls
properly and a lot of white space"* — the same defect §1 found on GENERATE, in
the same sheet, from the same cause.

Per canvas surface:

| Surface | State | Where it is instead |
|---|---|---|
| TOOLS chip row | **does not exist** | Icon and Label are `DccWidgets.tools_block` rows in the **cartography left dock** (line 294); Measure is a `MODES` segment inside `tool_bar.gd`, which only draws once Measure is already armed |
| `iconOn` variant strip | **does not exist** | `_build_icon_tool_options_row()` / `_build_icon_brush_controls()`, same dock |
| LAYERS | **built, elsewhere** | `LayersPopover`, opened from `ViewportHost._layers_btn`; all 8 canvas ids present among 43 rows |
| STYLE presets | **does not exist** | nothing selects a named look on the phone |
| Colour ramp | **built, elsewhere** | cartography left dock (CA-02), over `bridge.ramp_presets()` |
| `styleCustom` note | **does not exist** | — |
| Footnote | **built** | the only thing in the sheet |
| `measureOn` pill | **does not exist** | right dock measure context + `section_strip.gd` |
| `labelDraftOn` card | **does not exist** | right dock `CTX_ANNO` |
| `wayOn` card | **does not exist** | CIVIL dock — MORE's own row says *"taps append waypoints · commit from the dock"* |

### 2.7 Reachability — the owner's actual complaint, measured

*"Nothing from map, generate, plan or more really leads to a deeper menu."*
For MAP that is not an impression; it is what the tree does.

**With MAP lit, a finger can press seven things, and four of them are the tab
bar.** Measured list: `Layers`, `Search`, `⋮`, and the four tab cells.

| Canvas surface | Reachable from launch by tapping only what is visible? |
|---|---|
| INSPECT / MEASURE / LABEL / ICON | **No — none of the four.** `_mapinv_probe.gd` §5 finds no hit-testable control whose text or tooltip names any of them |
| Icon variants | No — behind Icon |
| Layer rows | **Yes** — `Layers` button, top-left of the viewport, hit-tested true. Not where the canvas draws them |
| Colour ramp | **No** — cartography left dock, and on a phone the left dock is a hidden sheet |
| Style presets | No — nothing selects a look |
| Measure / label / way readouts | No — behind tools that cannot be armed |

**The paths that do exist, and how long they are.** Icon and Label live in the
cartography left dock. `PhoneMenu._open_left_sheet()` is called from exactly two
rows — `_go_civilization()` and `_go_simulation()` — so **no MORE row opens the
*cartography* dock**. What remains is `MORE ▸ Window ▸ Left dock`, a
`PopupMenu` check item named after a desktop region
(`menus.gd::WIN_REGION_LABELS[ID_WIN_LEFT]`). Three taps, and the row is named
for furniture rather than for the tools behind it.

**Measure is worse, and this one was measured rather than reasoned.**
`app.arm_tool("measure")` has **three** call sites, not one, and the
conclusion below survives that while this sentence did not. The generic call
in `tool_bar.gd::_select_mode()`, plus two literal ones in `global_tools.gd` —
`set_measure_mode()` at :195, called from `tool_bar.gd` :559/:593 and so still
inside the same circle, and `recall_measurement()` at :147, called from
`right_dock.gd:3332::_on_measure_recall`, which **is** a genuinely different
entry point. **It cannot be a FIRST entry to Measure** — `_on_measure_recall`
returns early unless `_saved_measurements` is non-empty, and nothing can be
saved before the tool has been armed once. So the trap holds; the count did
not. **Corrected 2026-09-07 after a re-check, because a reader who verifies
"only from" finds three sites and stops trusting the paragraph** — and the
paragraph is right. (The same passage cited `grep -in measure shell/menus.gd`
as finding one comment; it finds **20** lines, all prose or an unrelated
readout.)

Reached from `tool_bar.gd::_select_mode()`,
whose `SCULPT / PAINT / MEASURE` segment `DccToolBar._build()` draws — and
`_build` fills `tool_options_row` only while one of those three is *already
armed* (`_on_tool_armed` returns early otherwise). So the one control that arms
Measure sits inside the panel that only appears once Measure is armed. Probe §8
tried to break that circle from the phone and could not; **no visible `MEASURE`
chip exists in any of the three states**:

| State | `MEASURE` chip visible | armed tool | tool scroller |
|---|---|---|---|
| MAP lit | no | `inspect` | shown |
| after `arm_tool("paint")` | no | `paint` | **hidden** |
| …then re-tap MAP | no | `paint` | shown |

The middle row is the mechanism. Arming Paint — the nearest thing to a phone
entry point, since Paint *is* in the WORLD dock's own TOOLS block — selects the
WORLD domain, which runs `_phone_tab_for_domain("world")` → `gen`, which runs
`_refresh_phone_gen_panel()`, which **hides the tool scroller** the segment was
just built into. Re-tapping MAP shows the scroller again and
`_on_workspace_changed("cartography")` overwrites the segment with the style
caption on the way. It is in no menu either: `grep -in measure shell/menus.gd`
finds one comment and no item.

**And raising the sheet buries the navigation.** Before the MAP tap, `Zoom out`,
`Zoom in`, `Pan mode` and `Reset view` all hit-test true. After it — with the
sheet at `half`, 609 px — all four are **not hit-testable**: the sheet covers
`ViewportHost._navpad`. The canvas has the same collision and resolves it by
anchoring the FAB column at `fabBottom` above `dockBottom`, both of which move
with the sheet. Nothing here moves.

### 2.8 Out of scope for this pass, deliberately

- **`undoOn`, `histOpen`, `simStrip`, `coachOn`** — `valsOver()`'s, not MAP's
  (§2.0). Recorded for §5's lane so it does not start from the stub's wrong
  filing: all four already exist in this shell — `DccShell._phone_undo_chip`
  (visible only while `bridge.can_undo()`), `_phone_undo_pop` behind a
  `PHONE_UNDO_HOLD_SEC = 0.45` long-press, `_phone_sim_strip` with
  `_phone_sim_speeds` and `_phone_sim_transport`, and
  `_maybe_show_coach_marks()` at boot. The canvas's `simStrip` slider is
  `min=-400 max=1200 step=1` with `×1 / ×10 / ×100` and a 600 ms play tick.
- **The sample chip** (`chipRef` / `hChipTap` / `chipLine1` / `chipLine2`,
  line 69) — inside `mapHostRef`, opens the inspector; that is §5's `inspOpen`.
- **The map canvas itself** — pan/pinch/rotate, the scale bar, `sampleData()`.
  The prototype's is a mock; this build's is real, and comparing them is a
  different pass.
- **No shell code was written.** This lane's deliverable is this section plus
  `_mapinv_probe.gd`; `godot-project/shell/` was not touched.

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
(68 006), `gridOn` (62 182), plus the `scrPicker` project
picker (3 525 – 6 537). Not inventoried by this pass.

**Four more belong here and were mis-filed under §2**, resolved 2026-09-07 —
`undoOn`, `histOpen` (inside it), `simStrip` and `coachOn` are `valsOver()`'s,
not `valsMap()`'s, and none of the four is inside `tabIsMap`. §2.0 has the
resolution and §2.8 records what this shell already has for each, so this
section's lane does not start from the wrong filing. **Every byte offset on
this page is 16 low** — it anchors the `sc-if value="{{ ` attribute, not the
symbol; see §2.0.

`modalOpen` (70 413) IS inventoried — §6 below.

---

## 6. NEW WORLD modal — `modalOpen`

Template bytes 70 413 onward. **This is the one place in this file where the
canvas is deliberately not followed, and the reason is a standing rule rather
than a judgement call.**

### 6.1 What the canvas draws

`NAME` (a text input), `SEED` with a `44 × 42` dice at radius 12, `EXTENT` as
two `min-height:42px` radius-14 chips, then `CANCEL` and `CREATE WORLD` at
`46 dp` / radius 23 with the create at `flex:1.4`. **No width field, no
resolution, no archetype.** Card `max-width:360px`, radius 22, `padding:18px
16px 16px`, over a `rgba(0,0,0,.45)` scrim padded `22px`.

### 6.2 Why this build draws more, and on whose authority

**Owner, 2026-09-07, holding the APK: *"even the initial or new map setup
doesn't allow for a km/size input"*, reported as a defect.** Verified before
acting: the canvas modal and the card this shell shipped **agreed with each
other**, and the owner overruled both. `CLAUDE.md`'s standing rule settles it —
*an owner decision is newer than any canvas.* Recorded here and in
`new_world_dialog.gd::_build()` so the next reader does not diff it against the
canvas and "fix" it back.

### 6.3 Per control, what came across and what did not

| Control | On the card | Reason |
|---|---|---|
| **Map width** (preset) | **yes** | the thing the owner named. `width_km / grid_w` is the one quotient every distance, grade, route length and settlement spacing is derived from; creation-time only; 800 km was silently the only answer a handset could give |
| **Width (km)** (free entry) | **yes** | the km half of the same value. `_refresh_dimensions()` already keeps preset ↔ km in step both ways, so they are one value with two faces, not two sources of truth |
| **Resolution** | yes (already was) | the other half of that quotient; `DCC_SHELL_SPEC.md` §2.1 names it for this command even though §6.7 does not draw it |
| **Archetype** | **yes** | creation-time in the strongest sense — `request()["archetype"]` decides **which generation call runs**, which no dial on the GENERATE sheet can express. It also had **no other route on a phone at all**, which is why that sheet's row was a dead end (§1.11) |
| Grid columns | no | it is the **same number** as Resolution (`_on_resolution_selected` writes it, `_refresh_dimensions` writes the dropdown back). Two views of one value inside 360 dp is `MISTAKES.md`'s "second route to a control the user already has one to" |
| Aspect | no | the extent chips already pick both of the reference's own aspects — it hardcodes 2:1 in world mode and 1.5625:1 otherwise, and `_update_extent_state()` moves between exactly those two. The other five ratios are this port's addition, and a wrong one yields a differently-shaped map rather than a broken one |
| Grid rows | no | derived from aspect; only editable as a hand-typed override |
| Derived readout (Grid / Extent / Cell size / Aspect) | **yes** | it is what makes the pair legible: `Width (km) 40 075` over `Resolution 512` is 78 km per cell, and nothing else on the card would say so |
| NAME | **no, and not because of this pass** | the canvas draws it; `request()` carries no name and `EngineBridge.generate()` takes none, so a name field would be a control the engine cannot back. Unchanged by this pass, and left as a gap for whoever adds world naming |

### 6.4 Measured

`_nwsize_probe.gd`, four viewports, `fail=0` at each:

| Viewport | scale | screen | card | window content min |
|---|---|---|---|---|
| 1080×2340 | 2.621 | 412 dp | **360 × 688 dp** | 400 dp |
| 1440×3168 | 3.495 | 412 dp | 360 × 688 dp | 400 dp |
| 720×1600 | 1.748 | 412 dp | 360 × 688 dp | 400 dp |
| 380×800 | 1.000 | 380 dp | 336 × 689 dp | 384 dp |

688 dp of card on 893 dp of usable height at 1080×2340 — it fitted with no
scrolling, confirmed on glass. Every tappable control on the card clears 44 dp
**on both axes** at all four.

**That fit claim expired on 2026-09-07 and the row below records what
replaced it.** §6.7's action row adds 46 dp plus 16 dp of `padding-top`, so the
card is **752 dp** and the form's `ScrollContainer` viewport is 702 dp at
1080×2340 (716 at 1440×3168, 725 at 720×1600). The card now scrolls — see
§6.6 — and it could not, until the same pass fixed the `PanelContainer` that
was eating every drag.

**One pre-existing measurement, with its attribution.** At a 380 dp screen the
window's content minimum lands **4 dp over** (384 against 380). The card's own
content wants **331 dp** against a 336 dp card, so the lifted controls are not
the cause: the excess is the dialog chrome around the card, and
`PHONE_CARD_INSET` counts §6.7's scrim padding without counting the
`AcceptDialog`'s own margins (chrome measures 40 dp at a 412 dp screen and 48 at
380 — not a constant, which is why no constant fixes it). Below any mainstream
handset; recorded, not patched, because shrinking the card on every device to
fix a sub-380 dp case is the worse trade.

### 6.5 On-glass evidence

Handset `9608b26b` (ONEPLUS_A6013), 1080×2340 @ 450 dpi, `--export-debug
"Android"` → `builds/android/Cartalith-0907b.apk`, `adb install -r`. Every act
is `adb shell input tap/swipe` at coordinates read off an `adb exec-out
screencap`; nothing calls into the app.

- **boot state asserted before anything was touched**: project picker, `no
  saved worlds yet — create one below`, `+ NEW WORLD` / `OPEN PROJECT .ZIP…`
- tap `+ NEW WORLD` → the card: `§ SEED 53996` + dice, `§ EXTENT REGION|WORLD`,
  `§ SIZE & RESOLUTION` with `Map width Province · 800 km`, `Width (km) 800`,
  `Resolution 2K`, the derived block `2048 × 1311 cells (2.68 M cells)` /
  `800 km × 512 km` / `0.391 km per cell, square` / `1.562 : 1 · landscape`,
  then `§ WORLD STRUCTURE Archetype Classic` and both advisories — all on one
  screen, no scrolling
- tap `Map width` → the 7-row popup at the tap floor; tap `Continent · 12 000
  km` → `Width (km) 12000`, derived `12000 km × 7682 km`, `5.9 km per cell`
- tap `Archetype` → `Classic / Earth-like / Supercontinent / Archipelago /
  Volcanic / Rift`; tap `Archipelago` → it takes
- set `Local · 200 km` + `512`, tap `Create` → a world of small islands, and
  `03 World structure`'s five dials read the Archipelago preset
  (`Continentality 0.15`, `Fragmentation 0.90`, `Tectonic energy 0.80`,
  `Ocean depth 0.30`, `Hotspot density 0.50`) — the lifted control reaching
  `generate_world_structure_sized`, not merely drawing
- **the swipe, on the control the report named**: a vertical swipe starting
  **on** the Gravity slider scrolls the sheet and leaves `1.00g`,
  `01 Planet · resolved` and `GENERATE WORLD` untouched; a horizontal drag on
  the same slider gives `2.00g`, `01 Planet · stale` and `REGENERATE 01 → 10`;
  a vertical swipe starting on **Ocean depth** leaves it at `0.30`; a swipe the
  other way (down) scrolls back and leaves it at `0.30` still
- `07 Hydrology` → the corrected `Min stream order` dash reads on glass
- tap the GENERATE sheet's `Archetype  New world ›` row → the card opens with
  `Archetype Archipelago` on it. **The route no longer dead-ends.**
- `adb logcat -d -s godot | grep -c "SCRIPT ERROR"` → **0**

**2026-09-07, the action row (§6.6), same handset, build `b1d30162c8e9`.** Boot
state asserted before any input (picker, `no saved worlds yet — create one
below`):

- tap `+ NEW WORLD` (538, 393) → the card, with `CANCEL` and `CREATE WORLD`
  **clipped at the card's bottom edge** — the 50 dp of overflow §6.6 measures
- swipe on the card, `input swipe 540 1700 540 1250 400` → the card scrolls
  (diff bbox `(57,246)-(1028,2086)`) and the row comes fully into view:
  **CANCEL** on the left as a chip wash, **CREATE WORLD** on the right filled
  accent and visibly wider, both radius-23 pills spanning the card. Against the
  **first** build of the same pass the identical swipe moved **zero pixels**
  (bbox `None`), which is the `PanelContainer=0` §6.6 records
- tap `CANCEL` (273, 1966) → the modal dismisses to the shell. **The touch path
  through a phone-presented `AcceptDialog` is a claim only the handset can
  make**, and this is it: the desktop probe cannot press these buttons at all

`project.godot` md5 `ccba27c9280cf8373412e2ba87ed4054` before and after every
Godot invocation, and `git diff -- project.godot` empty. No Rust changed this
pass (`git status --porcelain -- '*.rs'` empty); the APK carries the existing
`target/aarch64-linux-android/android-dev/libcartalith_godot.so`, mtime
2026-09-02 03:38.


### 6.6 The action row — built, and the departures left (2026-09-07)

**Read off the canvas rather than taken from a brief**
(`design/Cartalith-Android-2026-09-07.dc.html`, the `modalOpen` block, the last
`<div>` of the card):

```
<div style="display:flex;gap:10px;padding-top:16px">
  <div … style="flex:1;min-height:46px;border-radius:23px;…
                color:var(--sec);background:var(--chip)">CANCEL</div>
  <div … style="flex:1.4;min-height:46px;border-radius:23px;…
                color:var(--accInk);background:var(--acc)">CREATE WORLD</div>
</div>
```

Both `font:500 10.5px 'IBM Plex Mono'`, `letter-spacing:.14em`.

**What shipped until this pass was `AcceptDialog`'s own footer**:
`get_ok_button().text = "Create"` plus `add_cancel_button("Cancel")`, laid out
`[Create] [Cancel]` — primary first, **reported as** `59 × 44` and `60 × 44` dp
(**carried from an earlier pass and NOT re-measured here; a verifier could not
reproduce them — with only this file at HEAD it found no visible `Create` or
`Cancel` in the dialog tree at 1080×2340, so treat the two figures as an
unverified quotation rather than a measurement**) against a 360 dp
card. Three departures at once: the order inverted, about a third of the width
the artboard spans, and 44 dp where §6.7 says 46.

**The order was Godot's, and that is a mechanism rather than a reason.**
`AcceptDialog` owns its OK button and `add_cancel_button()` appends beside it,
so on this platform the primary comes first whatever the design says. Opened
before choosing, per the brief: honouring the artboard means not using that
convention. The row is built inside the card by
`new_world_dialog.gd::_build_phone_actions()` and `AcceptDialog`'s footer is
hidden — `get_ok_button().visible = false`, and the `add_cancel_button()` call
is gone. `confirmed` is still the signal `_on_create()` hangs off, so the
desktop and tablet path is byte-identical; the OK button simply never shows on a
phone. A full-width primary is also the better finger target, which is the
independent reason the same choice would have been right anyway.

**Not `DccWidgets.phone_pill()`.** That is the 412 canvas's *other* button
treatment — a 48 dp pill, accent-filled or outlined. §6.7 draws neither outline:
CANCEL is a `--chip` wash with `--sec` ink, CREATE WORLD is `--acc` with
`--accInk`, and both are 46 dp, not 48. Building it here leaves the shell's own
pill treatment untouched.

**Measured** — `_nwaction_probe.gd`, `fail=0` at each of 1080×2340 (scale
2.621), 1440×3168 (3.495) and 720×1600 (1.748). Identical dp at all three,
because `_fit_phone_card()` clamps the card to §6.7's `max-width:360px` on every
mainstream handset:

| | canvas | measured |
|---|---|---|
| order | CANCEL left | CANCEL at x 39, CREATE WORLD at x 180 |
| `flex:1` : `flex:1.4` | 1.40 | 131 dp : 185 dp = **1.41** |
| `min-height` | 46 | **46.0 dp** both |
| `border-radius` | 23 | **23** both |
| `gap` | 10 px | **10.0 dp** |
| spans the card | — | 326 dp of a 360 dp card (16 dp padding each side) |
| footer | — | `AcceptDialog`'s OK button not in the tree's visible set; no stray `Create`/`Cancel`/`Close` |

**The departures**, recorded rather than left silent. This table said *"the one
departure"* and a verifier found two more by reading the colours off the canvas
rather than off §0.1's palette map, whose "exact" row for these two inks is
itself wrong and predates this batch:

| Canvas says | This build | Why |
|---|---|---|
| `font: 500 10.5px 'IBM Plex Mono'` | **11 px**, `DccTheme.mono(1, true)` | `Control.add_theme_font_size_override()` takes an **int**; a half pixel is not expressible. `.14em` of 10.5 px is 1.47 px of tracking, which the same API takes as a whole pixel — so the tracking is 1 |
| CANCEL ink `--sec` `#8d9296` | `#a9adb0` | **Not deliberate.** §0.1's palette map calls this pairing exact and it is not. Pre-existing, so not a regression of this batch — but the "one departure" sentence above was, and it hid this one |
| CREATE WORLD ink `--accInk` `#16130c` | `#141005` | Same root as the row above: the token map, not the action row. Fix both at §0.1 and this table loses two rows |

**The row pushed the card past its scroller, and that surfaced a bug that was
already there.** `MISTAKES.md`: *content added below the fold evicts content
that was above it* — so the card was measured before and after. It went **688 →
752 dp** against a 702 dp viewport, putting CREATE WORLD's bottom edge 50 dp
below the fold. The first on-glass check was therefore a swipe on the card,
and it moved **zero pixels**: `ImageChops.difference` between the screencap
before and after an `adb shell input swipe 540 1700 540 1300 350` returned
`bbox = None`.

The cause is not the row. `_nwaction_probe.gd` prints the `mouse_filter` chain
from the card's last advisory `Label` up to the scroller, and it read

```
Label=2 -> VBoxContainer=1 -> MarginContainer=1 -> PanelContainer=0
       -> CenterContainer=1 -> VBoxContainer=1 -> ScrollContainer=1
```

— `0` is `MOUSE_FILTER_STOP`, and it is `_phone_card()`'s own `PanelContainer`.
**A `PanelContainer` defaults to `STOP` where every other `Container` defaults
to `PASS`**, and `DccShell.phone_fit()`'s PH-05 conversion deliberately excludes
`PanelContainer` because several in this shell carry their own `gui_input`. This
one does not. It cost nothing while 688 fitted inside 702; from this pass on it
would have made the primary action unreachable. Fixed at the card
(`_card.mouse_filter = MOUSE_FILTER_PASS`), asserted by that chain having no
`=0` in it, at all three densities.

**Two candidates were weighed and one is recorded as refused.** Pinning the
action row *outside* the scroller would put it below the card's rounded panel,
which is a bigger departure from the drawing than 50 dp of scroll; the artboard
draws the row as the card's last child. Shrinking the card back under 702 dp
means dropping one of the two advisories, and §6.3 keeps both deliberately —
*"a smaller form is a smaller set of controls; dropping a warning because the
artboard is narrower is a different thing."*

**What the probe cannot say.** `MISTAKES.md`: synthetic input cannot reach a
control inside a phone-presented `AcceptDialog` — `gui_get_hovered_control()`
stays null at `content_scale_factor` 2.62. So the dialog is opened by a staging
call and the two buttons are asserted on geometry and on having a `pressed`
connection, never on "a finger can press them". That claim is §6.5's, on glass.

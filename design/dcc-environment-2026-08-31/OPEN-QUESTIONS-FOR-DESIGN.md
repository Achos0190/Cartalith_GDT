# Prompt for Claude Design — resolving what the build needs

Paste everything below the line into Claude Design, in the **UI mockups
planning** project.

---

I'm implementing `Cartalith DCC Environment.dc.html` and `Cartalith Android.dc.html`
as the replacement GUI for the Cartalith app — a full replacement of the shipped
shell, not an upgrade. Six specification passes over both files raised 84 items
the build can't resolve from the files themselves.

Most of those collapse into one transport problem. The rest are genuine design
decisions I need you to make, and I'd rather you make them **in the files** than
answer in prose, so the prototype stays the source of truth.

## 1 · Blocker: the desktop file exceeds the export limit and arrives truncated

`Cartalith DCC Environment.dc.html` comes back through the design MCP as exactly
262,144 bytes — a hard 256 KiB cap — and it ends mid-word inside the logic class:

```js
measRows.push({i:('0'+i).slice(-2),len:this.fmtKm(km),be
```

cut inside `bear`, part-way through `valsCore()`. `Cartalith Android.dc.html` is
162 KiB and complete, so only the desktop file is affected.

The markup survived, so layout, labels, structure and the token sets are all
readable. What's lost is the tail of `valsCore()`'s return — the bindings that
say which string or colour fills each hole. Specifically I can't see: `ldPipe`;
the pipeline run block (`runStageLabel`, `runChainLabel`, `pipeNote`,
`progTitle`, `progPct`, `finLabel`, `bakeLabel`, `hFinalize`); `rdTitle` and
`rdCollapsedLabel`; `sampleRows`; `regionRows` and `regionReadout`; `tbLabel`
for most tool contexts; `measSegCol` / `measPathCol` / `hMeasMode` / `hMeasClear`;
`vpContext`, `vpField`, `scrimBg`, `mapCursor`; `layersBtnBg` / `layersBtnCol`;
`l.bg` / `l.col` in the layers popover; `masterOpPct`; and `tlShow` /
`tlCollapsed` / `tlExpanded`.

**Please get it under 256 KiB.** Either of these works and I don't mind which:

- **Split it in two** — e.g. `DCC Environment Frame.dc.html` (chrome, rail,
  menus, bars, viewport furniture) and `DCC Environment Docks.dc.html` (the left
  and right dock bodies for every mode), sharing the token block.
- **Strip the embedded map-drawing code.** The procedural terrain/coastline
  drawing is a large block and the GUI port doesn't need it — a flat placeholder
  in the viewport is fine for my purposes.

Everything in §2 and §3 stands regardless of the re-export; those are real
decisions, not missing bytes.

## 2 · Design decisions I need

### 2.1 CARTO's four rail nodes point at one destination

The rail tree is:

```
CARTO  →  Layers & style   mode ''
          Labels           mode ''
          Icons            mode ''
          Terrain appearance  mode ''
```

All four carry an empty `mode`, where WORLD's two and CIVIL's four each carry a
distinct one (`a`/`b`, `landmarks`/`factions`/`infra`/`planner`). So the design
gives four rows and one destination.

**Should these be four distinct left-dock panels, or one panel the four rows
scroll to?** If four, please give each a mode string and draw its dock. If one,
the rail rows want a different treatment so they don't read as four destinations.

### 2.2 `statusMid` — the middle of the status bar

The status bar has three regions. Left and right are specified. Nothing anywhere
in the delivered file computes a middle string, and no candidate exists in state.

**What does the middle of the status bar show?** In the shipped app it's the
last heavy pass, repaint time and autosave.

### 2.3 The tablet contradiction between the two files

This is the one I'd most like resolved, because the two prototypes disagree.

- `Cartalith DCC Environment.dc.html` has `TABLET 2560` and `TABLET PORTRAIT`
  frames that run **full desktop parity** — same regions, same seven menus, same
  disclosure depth, at 52/56 px bars, 48 px rail and 400 px docks.
- `Cartalith Android.dc.html` has a `TABLET 800` frame (800 × 1280) that runs
  the **identical phone layout** — same 84 px bottom bar, same four tabs, same
  full-width sheet.

Your standing directive to me has been "keep the tablet version as close as
possible to the Windows GUI", which matches the first and not the second.

**Where's the line?** A specific dp threshold is what I need — Android's own is
`sw600dp`, which would make an 800 dp tablet a desktop-parity device and put
`TABLET 800` on the wrong side of it. If you want 800 dp to stay phone-shaped,
give me the number.

### 2.4 Touch overrides that are missing

The touch density set covers most things but not these three, which keep their
desktop values on a tablet:

- the **200 px rail-expansion column**
- the **238 px layers popover**
- the **26 px and 22 px hero readouts** — everything around them grows from
  9 px to 11 px mono, so the ratio changes on tablet

**Are those intentional, or should they scale?**

### 2.5 Rail interaction details

- Does clicking the **already-active domain** toggle the rail expansion open, or
  is it a no-op?
- Does clicking a **node** close the panel the way `setDomain` does, or leave it?
- What glyph is the rail chevron in each state? Siblings in the same file use
  `›` for "open me", `‹` for "close me", and `▸` rotated 0°/90° in the dock
  accordions.

### 2.6 `scrimBg`

The plate behind the three viewport HUD chips has no definition in either theme.
**What colour, and does it differ light vs dark?**

## 3 · Things the prototype declares but doesn't do

Each of these is drawn and reachable but inert. For each I need to know: **is it
intended and simply not wired in the prototype, or is it an oversight?** The
answer decides whether I build it live or ship it visibly disabled with its
reason — this app's standing rule is that a control with nothing behind it must
say so rather than look live.

| What | Prototype behaviour |
|---|---|
| The 9 colour ramps | Selecting one sets `styleCustom = true`; nothing in the draw path reads `ramp` |
| The 6 simulation layer toggles | Climate / Population / Economy / Politics / Infrastructure / Warfare change no rendering |
| The 8 Data-manager routes | All eight nav rows land on the same screen; `dataRoute` is declared but never read |
| Stale-field rendering | The note promises "fields owned by stale stages read —" but no field ever renders that way |
| Undo depth | Preferences offers 1–50 (default 5); `undoStack` is never trimmed |
| The 3D viewport toggle | The FAB flips a 2D/3D label and toasts "relief exaggeration in Preferences ▸ Graphics" — but Preferences ▸ Graphics has no such control |
| 13 of 24 asset families | The header claims 24 families and 72 of 113 filled; only 11 are listed |
| Asset slot grids | Every family shows 12 cells regardless of its declared total (Buildings is 7/16 but gets 12) |
| Per-slot asset data | Every slot shows the same fixed placeholder — `capital-star.png · 512×512 · 84 KB`, `118% · fit · reset`, anchor `base` |
| Landmark viewshed | 8 types flagged "no viewshed"; the intended scoring once it exists isn't given |

## 4 · Phone gaps worth a ruling

The Android file is complete, so these are genuine omissions rather than lost
bytes. They matter because this ships on a real handset.

1. **Safe areas.** The 30 px status band and 18 px gesture inset are hard-coded
   decorations — there's no `env(safe-area-inset-*)` equivalent. On a device
   with a punch-hole or a gesture bar, what should actually reserve space?
2. **System back.** Only `Escape` is bound (modal → search → menu → inspector →
   sheet). Should Android's system back follow that same chain?
3. **Keyboard avoidance.** The LABEL bar sits at `bottom: 98px` and an IME would
   cover it; the search field and New World modal have no scroll-into-view.
4. **Accessibility.** No focus states, no content descriptions, no dynamic-type
   policy, no contrast statement — and **several targets fall below Android's
   48 dp minimum**. Which of those do you want fixed in the design?
5. **Rotation with a sheet open.** Portrait re-snaps the sheet; the landscape
   drawer has no equivalent.
6. **Light theme.** `--warn`, `--block`, `--good` and `--water` aren't redefined
   for light and keep their dark values; and the map canvas stays dark in light
   mode unless the user separately picks a light style preset.
7. **Empty and error states.** There's no empty world list, no "no results" row,
   no generation-failure state, no storage-full state anywhere in the file.
8. **Persistence.** The only persisted value is `localStorage['cartalith.coach']`.
   Should device, theme, units and preferences persist too?
9. **Haptics.** Nothing is defined for long-press sample, detent snap, tool arm
   or verdict change.

## 5 · How I'd like it back

In priority order:

1. The re-exported desktop file (§1) — this unblocks the largest share.
2. §2.3, the tablet threshold, as a number.
3. §2.1 and §2.2, in the file.
4. §3 as a simple intended / oversight call per row — one word each is enough.
5. §4 as whatever you want changed in the design, with the rest noted as
   deliberate so I can disclose them in the UI rather than silently omit them.

Anything you'd rather decide later, say so and I'll ship it visibly disabled with
the reason on the control, which is this app's convention for a gap.

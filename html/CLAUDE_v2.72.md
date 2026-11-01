# CLAUDE.md additions for v2.72 (paste into Cartalith_RC's root `CLAUDE.md`)

Two pastes. Neither changes anything else in the file.

## 1 · The file table

**Replace** the opening of the `Cartalith v2.71 DCC test.html` row,
`**The DCC shell line's current head, not a mainline version.**`, with `Previous DCC-line file.` (the rest of
that row stays as it is — the convention every older row follows), and **insert this row above it**:

```
| `Cartalith v2.72 DCC test.html` | **The DCC shell line's current head, not a mainline version.** Owner: *"Can we upgrade the layout in the html with the new design?"*, then *"A working copy in this repo"* and *"Everything, staged"*. **STAGE 1 OF 3: THE DESKTOP SHELL ONLY** (wider than 860px; a phone keeps v2.71's layout until stage 2, the Context panel's modules and the planner window are stage 3), from Cartalith_GDT's `PC_SHELL_PROPOSAL.md` §2–§3.12 and Rulings DY/DZ. A **menu bar** — CARTALITH, then File · Edit · Assets · Records · Story · Preferences · Window · Help, then Undo, Redo, Find and a document readout of only what the file knows (`seed · grid · extent`, `no world` behind the start card) — where **every row presses an existing control or calls the function its handler calls** (`_DSK_CMDS`) and a design row with nothing behind it is left out, not drawn disabled. **Ctrl+R is File ▸ Generate world in every domain**; Ctrl+O, Ctrl+S, Ctrl+Shift+F and Shift+A too, each checked free and none a key browsers keep. **The tool-options bar holds the armed tool's settings only (Ruling DZ)**: the context name and one hint, Sculpt's picker, mode and brush when armed, and the pins (Save, Settlements, Assets — still the way in and out of the library — Planner); Seed, ⟳, Center landmasses and Generate moved to **WORLD › Generate › Run**, View into the Base map chip, and `_stampGenLock` still locks all four (it never used containment). The rail is leftmost with vertical labels; the right dock is the **Context panel** (Inspect only — no note editor, no assistant); a **timeline strip above the status bar in every domain** (CV-24) carries Explore's own slider, years and filters, and no reader of the year tests the domain. **ONE CONTROL, TWO HOMES**: the markup keeps every control where v2.71 put it (its phone home) and `_dskPlace()` MOVES 26 elements into 13 desktop slots across the 860px line, marking each home with a comment node — never a clone; a phone measures **to the pixel** as v2.71. Found and left: `#assetsPanel` has been unreachable in Assets mode since v2.24. `tests/run.sh` {{RUN}}, `run_um.sh` {{UM}}, `hash_gen1.js` vs v2.71 **{{HASH}}**. Verify with `tests/perf/probe_desktopshell.js` ({{PN}} assertions; pass v2.71 second for the phone comparison; {{P71}} fail on v2.71). |
```

## 2 · The section

**Insert above** `### Nothing the settlement draws sits on water, and woodland is an area (v2.71, DCC-line file only)`
(DCC-line sections run newest first):

```markdown
### The desktop shell, stage 1: menus, tool settings only, the Context panel, the year everywhere (v2.72, DCC-line file only)

Owner: *"Can we upgrade the layout in the html with the new design?"*, then *"A working copy in this
repo"* and *"Everything, staged"*. Stage 1 is the desktop (wider than 860px); a phone keeps v2.71's
layout until stage 2; the Context panel's modules and the planner window are stage 3. Design:
Cartalith_GDT `PC_SHELL_PROPOSAL.md` §2–§3.12 and Rulings DY/DZ. `tests/run.sh` **{{RUN}}**,
`run_um.sh` **{{UM}}**, `hash_gen1.js` vs v2.71 **{{HASH}}**; verification is
`tests/perf/probe_desktopshell.js` ({{PN}} assertions; pass v2.71 as a second argument and the phone half
compares against its geometry; run alone on v2.71 it fails {{P71}}).

- **ONE CONTROL, TWO HOMES — `_DSK_SLOTS` and `_dskPlace()`.** When a control sits in a different band on
  the desktop than on a phone, it is still ONE element. The markup keeps it where v2.71 put it (its phone
  home); its desktop home is an empty `.shell-slot` (`display:contents`); `_dskPlace()` moves it across the
  860px line. The first move writes an empty comment node right before the element, and the way back is
  "insert after that marker", which restores the exact place whatever order the rows run in. A slot fills
  in TABLE order. **Never clone a control to "appear" in both places** — v1.57's defect. A new control that
  lives in different places by form factor gets a row in `_DSK_SLOTS`, not a second copy.
- **A slot is `display:contents`, so `.parent > .child` across it never matches.** The first cut wrote
  `.map-br > .viewdim` and the 3D square and zoom column silently kept their phone positions. Write
  descendant selectors for anything that sits in a slot.
- **The desktop query is `not all and (max-width:860px)`, never `(min-width:861px)`.** It is the exact
  complement of the phone's query; a fractional window width between 860 and 861 would otherwise get
  neither layout. `.dk-only`/`.ph-only` carry `!important` on purpose: a form-factor gate must beat the
  inline display a control's own JS writes.
- **One controller owns all eight menus (`_dskMenuInit`) — File included, on a phone too**, where it is
  v0.87's File ▾ with its contract kept (the five single-shot rows close it; the export form does not). Two
  controllers on one panel cannot agree on which menu is open, and hover-switching must know. Esc is taken
  at WINDOW CAPTURE while a menu is open, so it cannot also commit a half-drawn route or way.
- **A menu row presses what exists.** `_DSK_CMDS` entries click the real button or call the function that
  button's handler calls; `enabled`/`checked` READ the real control each time a menu opens — a row never
  stores a copy of a state. A design row with nothing behind it is left OUT (the CHANGELOG lists each with
  its checked reason), not drawn disabled. A command whose control is disabled says why when a key fires it
  (`refused` — Ctrl+R on a finalized world raises the button's own alert).
- **Accelerators (`_DSK_KEYS`) were checked free against every key the file binds**, and against the keys a
  browser keeps (Ctrl+N, Ctrl+T, Ctrl+W, Ctrl+1–9 — nothing is drawn against those). Their
  `preventDefault` is what stops Ctrl+R reloading, so a held key is swallowed, not ignored. Under a modal or
  the setup gate they do not fire and the browser keeps its default.
- **The tool-options bar holds the armed tool's settings only (Ruling DZ).** `_dskToolOptsSync()` writes the
  context name and hint and shows the sculpt group; it runs from `_applyDomainUI`, `_civSetTool`,
  `sculptSyncFeatureSeg`, `applyFinalizedUI`, Assets mode's enter/exit and the paint toggle. **Any new way
  to arm a tool must call it**, or the bar names the previous tool. The bar stays drawn in Assets mode on
  the desktop — its pins hold the Assets toggle, the only way back out (v2.46).
- **Find must own every container a control can sit in.** The search lists `label[for]` controls and reveals
  one by clicking each panel owner that contains it. A labelled control moved into a new container stays
  LISTED and becomes UNREVEALABLE until `_findPanelOwners` has an owner for that container — v2.71 listed
  the export form's "Image size" and the Sculpt category's sliders and could reveal neither. v2.72 adds
  WORLD's two categories, the bar's sculpt group and the menus with their submenus.
- **Block-1 functions that block 2 calls read block-2 `let`s through `_dskV` (try/catch), never `typeof`**,
  which throws on a binding still in its dead zone (v2.15). `var _dskReady` gates every sync function until
  `_dskInit` has run, so the boot render and `syncUI` cannot reach this section's own consts early.
- **The document readout must never show the boot seed as a world's.** The boot render writes it before the
  start card opens, so `_setupOpen`/`_setupHide` refresh it; it reads `_hasLiveWorld()`.
- **The Context panel's subject is `_civRenderInspector()`'s own precedence** — place, else label, else icon
  — refreshed from `_civOpenPlacePopup`/`_civClosePlacePopup`, the two functions every selection change ends
  in. It only reads the selection. The popup's × re-renders the inspector now: hiding only the popup left Inspect
  reading "Editing in the popup" beside a header saying "Nothing selected".
- **No reader of the year tests the domain, and it must stay that way.** The strip moves `civGotoYear →
  civSnapshotLoad → drawCivLayerAuto`, and `drawCivLayer`'s `_tlActive` is `civTimeline.length>0`. A reader
  gated on `_domain==='explore'` would make the strip lie everywhere else.
- **Found, not fixed (since v2.24):** `#assetsPanel` sits in the left dock, which Assets mode hides, so its
  pack metadata, stats and Validate output cannot be reached by a click. The Assets menu reaches Export,
  Open pack, Apply and Clear by opening the library and pressing their buttons; Validate and the metadata
  are open.
```

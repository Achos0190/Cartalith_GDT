## v2.72 (DCC line) — the desktop shell, stage 1: a menu bar, a tool-options bar that holds tool settings only, the Context panel, and the year in every domain

Owner: *"Can we upgrade the layout in the html with the new design?"* — then, asked how, *"A working copy
in this repo"* and *"Everything, staged"*. Stage 1 is the **desktop** shell (windows wider than 860px);
stage 2 is the phone and tablet shells; stage 3 is the Context panel's modules and the journey planner in
its own window. The design is Cartalith_GDT's `PC_SHELL_PROPOSAL.md` §2–§3.12 with Rulings DY and DZ, and
its mockup `design/mobile-shell-2026-10-31/Desktop.dc.html`. `tests/run.sh` **1280/0**, `run_um.sh`
**882/0**, `hash_gen1.js` vs v2.71 **ALL IDENTICAL**. Verification is `tests/perf/probe_desktopshell.js`
(**174 assertions**; give it v2.71 as a second argument and the phone half compares against v2.71's own
geometry; run alone on v2.71 it fails **63** — a section that cannot run there fails once, by name — and
passes 20: the start card, File ▾'s own picker, unique ids, Find's Seed, Settlements, the phone half, which
v2.72 leaves as v2.71 has it, no page errors, and the busy-key check, which v2.71 meets by having no Ctrl+R).

- **ONE CONTROL, TWO HOMES — and still one element.** Most of what moved on the desktop sits somewhere
  else on a phone, and a phone must keep v2.71's layout until stage 2. So the markup keeps every control
  where v2.71 put it (its phone home), each desktop home is an empty `display:contents` slot, and
  `_dskPlace()` MOVES the element between the two whenever the 860px line is crossed (`_DSK_SLOTS`: 26
  elements into 13 slots). The first move writes an empty comment node right before the element, so
  going back to a phone returns it to exactly that place whatever the order. Never a clone: one node, one
  id, one set of listeners — v1.57's rule, carried one step past v2.31's `order` trick, which only works
  inside one flex line. Measured: at 390×760 and 412×915 the map, the drawer and its rail sit **to the
  pixel** where v2.71 puts them, and the header and document bar draw the same controls in the same order.
- **The menu bar replaces the header band (PC §3.8).** CARTALITH in mono and the accent, then **File ·
  Edit · Assets · Records · Story · Preferences · Window · Help**, then Undo, Redo, Find (same ids, ⌃K
  kept) and a document readout that shows only what this file knows: `seed · grid · extent`, `no world`
  behind the start card or after Skip for now (the boot render writes it before the gate opens, so the gate
  refreshes it — it was showing the boot seed as if it were a world's), and `#phaseChip` beside it once
  finalized. One controller (`_dskMenuInit`) owns all eight menus, **File included, on a phone too**,
  where it is v2.71's File ▾ with v0.87's contract intact: the five single-shot rows close it, the export
  form does not. One line per row, accelerators right-aligned, submenus beside their row (lifted, never
  above the top, when they would run off the bottom), hover switches menus while one is open, Esc or an
  outside click closes, arrows walk rows and menus, and focus is outlined. Esc is taken at window
  CAPTURE while a menu is open, so it cannot also commit a half-drawn route or way underneath. In File ▸
  Export the form's two checkbox labels that carry a `<code>` path (`atlas/`, `layers/`) now read as one
  paragraph: as `label.chk`'s flex row every text run became a column of its own, and v2.71's 300px
  dropdown clipped the last one. Desktop only; a phone's File ▾ measures as v2.71's.
- **Every row presses what this file already has** (`_DSK_CMDS`): the real button, or the function that
  button's handler calls. `enabled`/`checked` are READ from the real control each time a menu opens; no
  row keeps a copy of a state. A design row with nothing behind it is left out, not drawn disabled —
  table below, each reason checked by grep. File keeps File ▾'s own rows, as the stage-1 brief asked,
  so the phone's File ▾ loses nothing; the design files heightmap import under Records ▸ Import and
  asset packs under Assets, and both now carry a signpost to File instead.
- **Keys, each checked free** against every key the file binds (Space, Shift+D, Delete, Enter, Escape,
  ⌃K, ⌃Z and the bare layer letters, whose handler bails on any modifier): **Ctrl+R** File ▸ Generate
  world, in every domain — v2.24 hoisted Generate into the document bar so it was one click from
  anywhere, and this keeps that while the button moves; its `preventDefault` is what stops a reload, a
  held key neither reloads nor queues a stack of generates, and on a finalized world it gives the same
  alert the button's own guard gives instead of doing nothing. **Ctrl+O** opens a project, **Ctrl+S**
  saves (the project .zip — "a project ZIP genuinely IS a full save", v0.88), **Ctrl+Shift+F** Factions,
  **Shift+A** the Asset Library (never while typing). Ctrl+N, Ctrl+T and Ctrl+W are kept by browsers, so
  nothing is drawn against them. Under a modal or the setup gate none of these fire and the browser keeps
  its default: the menu bar is covered there, and its keys with it. **While the busy overlay is up they
  are swallowed**: the first cut let Ctrl+R through mid-operation, and `withBusy`'s chain queued a second
  generate behind the running one (measured). Now nothing is queued and nothing reloads. The overlay
  covers only the map, so a click on the dock's own Generate can still queue one, as in v2.71.
- **The tool-options bar holds the armed tool's settings only (Ruling DZ).** Owner: *"The buttons in the
  top rail new seed, generate and bake can be removed. Please see to only keep the paint/sculpt brush
  settings here."* With no tool armed it carries the context name (GENERATE · WORLD, CIVIL · WAY …) and one
  hint (`_dskToolOptsSync`), then the pins. Seed, ⟳ (now "⟳ New seed") and Center landmasses moved into
  **WORLD › Generate › Run**, first in the dock under the finalized banner, with GENERATE WORLD at full
  width in the accent. View (`#modeSeg`) is not a tool setting and moved into the Base map chip. The four
  keep their handlers, and `_stampGenLock()` never used containment — measured: finalize disables all
  four in Run, File ▸ Generate world reads disabled, un-finalize releases them.
- **Sculpt's settings are the sculpt tool's.** Armed, the bar reads SCULPT · <feature> and holds
  `#toolRail`'s feature picker, Freehand's brush mode (`#sculptModeSeg`), brush size with its real-km
  radius, hardness and intensity, and the feature's own hint — the same elements, found by id as
  always — on two rows, the way v2.24's tool rail added a band (the map refits). The rail band itself no
  longer draws on the desktop. The noise rows, the stamp stack and Commit/Discard stay in the dock this
  stage.
- **Pins (HTML-backed only, no "+")**: Save (presses `#exportBtn`), Settlements and Assets — the v2.46 and
  v2.63 header toggles THEMSELVES, so Assets is still the visible way into and out of the library ("←
  Map"; the bar stays drawn in Assets mode for exactly that reason, where v2.24 hid it), and Planner (the
  action behind `#jpOpenEditorBtn`; with no journey selected it shows Explore ▸ Journeys rather than doing
  nothing).
- **The rail** is the leftmost band (order −2, the DCC frame's own order and the mockup's; v2.30–v2.31 drew
  it between the dock and the map), with vertical mono labels and an accent bar on the active domain.
  Buttons, ids, `data-*` literals and handlers are untouched. **EXPLORE stays a domain** until stage 3
  rehomes its journeys and planner.
- **The left dock** has a header naming where you are (WORLD › GENERATE, WORLD › SCULPT, CIVIL …) with a
  chevron that is Window ▸ Left dock; collapsed, the dock is a 30px strip that keeps the chevron, so the
  way back is where it was. Section labels are mono and uppercase; WORLD's category control is drawn as the
  design's GENERATE | SCULPT ("Terrain" on a phone). Domain bodies keep their inline displays (v2.24).
- **The right dock is the Context panel**: CONTEXT · kind, the subject's name or "Nothing selected", one
  line of facts, and an INSPECT tab whose page is `#inspector`, unchanged. The subject is read from
  exactly what `_civRenderInspector()` renders from, in its order — the place, else the label, else the
  icon — refreshed from `_civOpenPlacePopup`/`_civClosePlacePopup`, the two paths every selection change
  ends in. Absent facts are left out, never shown as zero. Note and Ask are not drawn: no note editor
  (v2.11 shows a native-app save's vault links read-only in the place and faction editors) and no
  assistant. Collapses to a 30px strip, like the left dock. **One stale state this exposed is fixed**: the
  place popup's × hid the popup and left Inspect reading "Editing in the popup on the map" with nothing
  selected — unseen while the dock was plain Properties, a contradiction beside a header that says
  "Nothing selected". The × now re-renders the inspector, which owns the empty state and closes the popup
  itself.
- **The timeline strip — the year in every domain** (`GUI_GAP_REGISTER.md` CV-24, approved as Ruling DY
  PQ2). Collapsed: TIMELINE · active year · ▶ Animate · the slider with a tick per recorded year · min/max;
  the chevron adds the recorded years (pills, Add year) and the three filters. Every control is Explore's
  own element, moved, so `_civWireYearSlider`/`civGotoYear` drive it unchanged. **Every reader checked**:
  `civGotoYear → civSnapshotLoad → drawCivLayerAuto`, and `drawCivLayer`'s `_tlActive = civTimeline.length>0`
  — none tests the domain, so nothing needed un-gating. Measured: in all four domains the slider moves
  `civYear`, the painted territory and the civ layer's pixels. With fewer than two recorded years the
  strip says which reason applies instead of leaving a gap. The simulator and Journeys stay in Explore.
  Show, hide and expand all call `_scheduleViewportRefit()`.
- **The status bar** is mono and small as before; `#verTag` (v2.72) sits at its right end. `updateReadout`
  is unchanged except one class on its version chip (`.r-ver`), so the desktop can drop that duplicate —
  a phone, whose header holds `#verTag`, still shows it.
- **Map controls**: top-left the Base map chip (`#layersFab`, View inside its popover) with the Explore
  filter beside it; bottom-right one column — the 2D/3D switch drawn as the one square it will switch to,
  over ⌖ recentre, + and − (`#zoomReset`/`#zoomIn`/`#zoomOut`, the existing buttons v2.71 drew on touch
  devices only; ✋ stays for coarse pointers); bottom-left the scale bar with the legend above it.
  Measured at four sizes and two seeds: no two overlap and all sit inside the map.
- **Find still reaches what moved.** Seed, Brush size, Hardness, Intensity and Image size are listed and
  revealed on screen from another domain. `_findPanelOwners` gained three owners for that: WORLD's two
  categories (a Run hit must also switch off Sculpt), the sculpt tool's place in the bar, and the menus and
  their submenus. v2.71 listed the export form's and the Sculpt category's labelled controls and could
  reveal neither.
- **One extraction, no copies**: the Delete key's body is now `_civDeleteSelectedPlace()`, so Edit ▸ Delete
  selected place runs the same code (no confirm, no undo — exactly as the key).
- **Found, not fixed (pre-existing since v2.24, measured on v2.71)**: the Asset Library's dock panel
  `#assetsPanel` — pack name, author and licence, `#alStats`, Validate and its output, Export/Apply/Import/
  Clear — is inside the left dock, which Assets mode hides, so none of it can be reached by a real click.
  The Assets menu now gives Export, Open pack, Apply and Clear a reachable path (each opens the library and
  presses its button, whose toast shows there); Validate and the pack metadata stay unreachable, and are
  listed below.
- **Unchanged by construction**: no engine line moved, so the headless suites and the render hashes are as
  v2.71's. Known cuts: a phone and tablet keep v2.71 (stage 2); Context modules, back, forward and pin
  (stage 3); the planner window (stage 3); Explore dissolving (stage 3); customisable pins; the paint brush
  and the CIVIL/CARTO/EXPLORE tools' settings in the bar (they stay in the dock's Tools block, and the bar's
  hint says so).
- **Seen, not changed**: the floating place editor (v0.90's popup, `max-height: min(80vh, 660px)`, fixed to
  the window) can reach over the timeline strip at 1366×768 when the strip is expanded — measured, its
  bottom at 697px against the expanded strip's top at 664px (collapsed, the strip starts at 707px and
  the popup clears it). It is placed against the window, not the map, and its × closes it; fitting it to
  the map's own box would change v2.71's popup placement on every form factor, which is not stage 1's to
  change.
- **RC's shell and UI probes, run on both files** (18: `shellui`, `domainrail`, `cogmodal`, `assetsbtn`,
  `geninfo`, `worldreset`, `settleparams`, `typepreview`, `savetree`, `civbusy`, `manpower`, `trade`,
  `landmass`, `landsurface`, `roadconnect`, `foodshed`, `placement`, `water_gap`). **Four assertions change,
  each the design moving a control a probe pinned where an earlier version put it, none a regression**:
  `probe_shellui`'s "…in that left-to-right order" (v2.27's Find · Undo · Redo · File · cog — File is now
  the bar's first menu, Undo · Redo · Find sit at its right end, the cog is not drawn on the desktop; at
  390×720 all five measure where v2.71 puts them); `probe_domainrail`'s "…still between the tools dock and
  the map" (the rail is the leftmost band, the DCC frame's order); and `probe_assetsbtn`'s and
  `probe_settleparams`'s "it is in the header" (both toggles are pins at the tool-options bar's right end —
  still one visible click, never behind the cog, as each probe's other assertions show). Every other
  assertion matches v2.71's result, including four failures both files share and this version does not
  touch: `settleparams`' "three panes" (written before v2.66 added Types), `civbusy`'s "the same world"
  (a v2.38 pin), and `foodshed`'s and `placement`'s urbanisation band (21.21% on both). Two needed the
  harness, not the file: `probe_civbusy` hard-codes Cartalith_RC's own folder, so it ran from a copy
  pointed at the files' folder, and `probe_savetree` exhausts node's default 2 GB heap on BOTH files here
  and ran with `--max-old-space-size=8192` (31/0 on each). One pass is weaker than it reads:
  `probe_shellui`'s "…share one line" counts the cog, which the desktop does not draw, so its empty box
  passes by sitting at the top.

**What moved where** (desktop = wider than 860px; a phone keeps every v2.71 home)

| Control (ids kept) | v2.71 | v2.72 desktop |
|---|---|---|
| File ▾ `#fileWrap`/`#fileMenuBtn`/`#fileMenu` — Load heightmap…, Infer tectonics, Load project .zip…, Import asset pack… | header, between Redo and the cog | the menu bar's File, plus Generate world, New world from selection…, Save project, Autosave |
| export form `#exportSec` (`#bakeRes`, `#bakeTiles`, `#chanAtlasChk`, `#layersPreviewChk`, `#exportBtn`, `#exportGeoBtn`) | inside File ▾ | File ▸ Export submenu (inline in File ▾ on a phone) |
| `#settlementsHeaderBtn`, `#assetsHeaderBtn` | header, left | pins at the tool-options bar's right end |
| `h1` | "CARTALITH GEN1" | wordmark CARTALITH ("Gen1" only on a phone) |
| `#verTag` | header | status bar, right end |
| tagline `.tag` | header (phone hid it) | not drawn |
| `#undoBtn`, `#redoBtn`, `#findWrap`/`#findInput`/`#findResults`, `#phaseChip` | header, right | menu bar, right: ↩ ↪ Find, phase, then the new `#docReadout` |
| `#cogBtn` | header, right end | not drawn; Preferences ▸ Performance & tiled LOD…, Storage & data…, Appearance & 3D view… and Help ▸ Generation info… open the same `#settingsModal` |
| `#gpuToggle`, `#themeToggleBtn`, `#autosaveChk`, `#creditsBtn` | Settings pages | stay there; Preferences ▸ GPU acceleration, Theme ▸ Dark/Light, File ▸ Autosave, Help ▸ Credits press them |
| `#seedN` + label, `#reseedBtn`, `#centerBtn`, `#genBtn` | document bar | WORLD › Generate › Run (Generate also File ▸ Generate world, Ctrl+R) |
| `#modeSeg` (View) | document bar | Base map chip's popover |
| `#toolRail` band: `#sculptFeatureSeg`, `#sculptFeatHint` | band under the document bar | tool-options bar while Sculpt is armed (band not drawn) |
| `#sBrush`/`#sHard`/`#sInten` rows, `#sBrushKm`; `#sculptModeSeg` | WORLD › Sculpt (Brush & noise; Feature parameters) | tool-options bar while Sculpt is armed |
| `#domainRail` | between the left dock and the map | far left, vertical labels |
| `#layersFab`/`#layersBtn`, `#explFilterFab` | map top-left (54px, 10px) | map top-left: Base map chip, the filter beside it |
| `#viewDimSeg` | map top centre | map bottom-right column, top |
| `#zoomOverlay` (`#zoomIn`, `#zoomOut`, `#zoomReset`, `#panBtn`) | bottom-right, touch devices only | bottom-right column on every desktop (✋ on coarse pointers only) |
| `#scaleBar`, `#legend` | bottom-right (right 64px); bottom-left | bottom-left, the legend above the scale bar |
| `#explTimelineSliderRow` (slider, ticks, min/max, active year, `#explTlPlayBtn`) | Explore ▸ Timeline | timeline strip, collapsed row |
| `#civTimelinePanel`, `#civTlYear`, `#civTlAddYearBtn`, the three filters (now in `#tlFilters`) | Explore ▸ Timeline | timeline strip, second row |
| `civSim*`, `#civJourneyList`, `#civPlannerSec` | Explore | unchanged (stage 3) |
| `#inspector`/`#inspectorBody`, `#civInfoSec` | right dock | Context panel ▸ Inspect |
| `#statusBar`, `#readout`, `#genInfoBtn`, chips | status bar | unchanged (version chip yields to `#verTag`) |

**Not in this build** — design rows with nothing behind them in this file (each checked by grep, by concept
rather than by one string), or with a reason they cannot be wired safely yet

| Menu ▸ row | Why it is left out |
|---|---|
| File ▸ New world… | The New World card (`_setupOpen('generate')`) is the boot gate and is not built to be re-entered over a live world: its Back leads to the intro, whose Skip for now sets `_setupSkipped`, which makes `_hasLiveWorld()` false and so switches off the beforeunload guard and autosave for the world still open; `_suGenCommit` calls `allocate()` without `confirmRegenerate()`, so on a finalized world `generate()` then refuses and leaves the field as `allocate()` zeroed it, and an imported heightmap is discarded without the confirm Generate asks. Ctrl+N is also the browser's. Regenerating with the current settings is File ▸ Generate world |
| File ▸ Recent worlds | No recent-files list; a file:// page cannot reopen a file by path. Autosave snapshots are Settings ▸ Storage & data |
| File ▸ Save as… | Every save is a browser download of the project .zip; there is no second path to choose |
| File ▸ Autosave interval | The interval is a free 1–120 minute field (`#autosaveMins`) in Preferences ▸ Storage & data…; fixed radio values would be a second, narrower editor of one number |
| File ▸ Revert to last save | Saves are downloads, so the page holds no handle to the last one; snapshots restore from Storage & data |
| File ▸ Close project | No close operation exists (the app always holds a world or Skip's empty state); Ctrl+W is the browser's |
| File ▸ Export ▸ Player map…, DM / full map…, Export map image… | No player/DM split, and no standalone image: the rendered map is written only inside the project .zip (`exportZip`) — File ▸ Export carries it. Tiled output of a region is Region export (Storage & data) |
| File ▸ Export ▸ Print sheet, VTT battle map | Not built in the design either |
| File ▸ Storage locations, Change locations…, Show project on disk | A browser page has no file-system locations; Storage & data explains where snapshots live |
| Edit ▸ Undo history… | No history list; `#undoMem`'s step/MB readout rides on the Undo button's tooltip (v2.27) |
| Edit ▸ Cut, Copy, Paste, Select all | No clipboard or multi-selection for map entities (the only clipboard use is Generation info's copy) |
| Edit ▸ Deselect, Previous/Next selection | Selection is three variables (place, label, icon), cleared by the popup's ×, Inspect on empty ground or a domain switch; one Deselect and a history over them is the one-selection model of PC §3.1 — stage 3 |
| Edit ▸ Context card, Cartwheel at the pointer | The map menu opens at the pointer on a right-click only, with no keyboard path; no cartwheel exists |
| Edit ▸ Reset generation parameters, Reset one stage | No reset-to-defaults and no ten-stage pipeline; `#resetBtn` is erosion's "Reset", which regenerates with the current parameters |
| Assets ▸ Pack report…, Import folder of art…, Compare pack with library… | None exist |
| Assets ▸ Asset pack (readouts, Pack metadata…, Validate pack) | The controls exist (`#alPackName`/`Author`/`License`, `#alStats`, `#alValidateBtn` → `#alValidation`) but sit in `#assetsPanel`, which Assets mode hides (above): a row would run a control whose result has nowhere visible to land. Open issue |
| Assets ▸ Icon families, Texture sets, Landmark types | The library's own left rail lists its families with counts; no command opens it at one, and no landmark-type system exists |
| Records ▸ Data manager, World data tables…, Sources & validation | None exist |
| Records ▸ Markdown vault…, Browse & edit a note…, Places without notes… | No note editor (v2.11 shows imported vault links read-only, per place and faction) |
| Records ▸ Import ▸ Maps tiles, GIS / GeoJSON, World Data .ctl | No importer for any of them (GeoJSON is export-only); Heightmaps is File ▸ Import here (signpost) |
| Story ▸ Political history of the selection, Faction history… | The recorded years are world-level snapshots (`civTimeline`); there is no per-entity history view |
| Story ▸ Travel library…, Conflicts…, Draw a conflict, Campaigns this year | None exist (the only "conflict" here is a collapse-simulator character) |
| Story ▸ Note for the selection, Ask the assistant | No note editor, no assistant |
| Preferences ▸ Devices, Multi-GPU mode, CPU worker threads, VRAM budget, Fallback when VRAM full, Try the GPU again | One GPU checkbox (`#gpuToggle`, a row here) and a worker pool that sizes itself; none of these settings exist |
| Preferences ▸ Render quality, Anti-aliasing, 3D viewport defaults, Lighting rig defaults, Relief exaggeration | The 3D dials are Appearance & 3D view…; no render-quality or lighting-rig preference exists |
| Preferences ▸ Tiled LOD, Tile size · LOD levels, Atlas cache, Chunk debug overlay | They exist, in Settings ▸ Performance and Storage & data, and are reached through those two rows rather than redrawn as menu rows — one editor per setting |
| Preferences ▸ Undo history size, Working set, Clear caches… | The undo depth is a constant (`MAX_UNDO`); no cache-clearing command exists |
| Preferences ▸ Assistant…, Quick bar…, Open panels in their own window, Handedness, Keyboard shortcuts… | None exist |
| Preferences ▸ Theme ▸ Follow system; Units ▸ Nautical miles | The theme switch has two states and the unit toggle two (km, mi) |
| Window ▸ Context panel ▸ (Inspect/Note/Ask on Ctrl+1/2/3, Pin, Follow, modules, Reset panel) | One tab (Inspect); pin and follow need a selection history; no modules (stage 3). Ctrl+1–3 are the browser's tab keys |
| Window ▸ Session mode, 3D capability | None exist |
| Window ▸ Open windows, Layouts | Every window here is modal, and there are no saved layouts |
| Help ▸ Documentation | Not built in the design either |
| Help ▸ Keyboard reference…, Save diagnostic report… | No shortcut list (the menus print their keys) and no report writer (Generation info… is the parameter dump; Window ▸ Diagnostics overlay is Shift+D) |

**Rows added that the design does not list**: File ▸ Import's four v2.71 rows (kept in File, above); Story ▸
Simulate collapse / recovery… (reveals the simulator, which stays in Explore — PQ7 keeps both of its entry
points); Window ▸ Workspace ▸ Explore (the fourth domain survives until stage 3); Help's version readout.

# Presented but not wired — the standing table

Owner, 2026-08-30: *"Make sure all menu's are created and all presented
functions are wired or the ones that have no code behind them get listed in a
table with a proper proposal (inferred from the menu name and highest probable
explanation/design spec of the named function.)"*

This is that table. Every proposal is drawn from `docs/DCC_SHELL_SPEC.md`'s own
behaviour column, from `docs/ANDROID_UI_SPEC.md`, or from a binding that already
exists in the engine — none is invented.

**Every row here is honest in the product too.** `menus.gd::_todo(p, text, why)`
always sets a tooltip, so each appears disabled *with its reason* where a user
meets it; none is enabled-and-inert. `CommandIndex` reports the same reasons in
the phone's search results.

---

## This document was itself stale, and that is the point of re-verifying it

The first cut had **44 rows**. A read-only re-verification pass on 2026-08-30
checked every one against the code and found **12 of 44 wrong**: eleven had
shipped since the table was written, and one described a real gap inaccurately.
Four more were duplicates of each other.

That is the exact defect class this repository has spent the week fixing — a
disclosure describing something the code no longer does — and a *gap register*
is the worst place for it, because its whole value is being trustworthy about
what is missing. Recorded here rather than quietly corrected.

The rows closed since the first cut, with what closed them:

| Was listed as a gap | Actually |
|---|---|
| Clear caches… has no confirmation | `menus.gd::_clear_caches()` builds a real `ConfirmationDialog`, freed bytes in the OK button |
| Autosave interval submenu ABSENT | `_autosave_popup` exists and writes `DccSettings.set_autosave_minutes()`; the status bar reports it |
| GPU backend readout absent | rewritten in `about_to_popup` to `"GPU acceleration   %s · %s"` via `_active_backend()` |
| Working set opens a dialog instead of a row | inline disabled row, refreshed in `about_to_popup`; the dialog is kept as well |
| Icon families / Texture sets have no filled counts | `_refresh_family_counts()` computes filled/capacity live |
| Pack metadata… wired to the wrong thing | a real three-field modal writing `as_set_pack_info()` |
| Tile size · LOD levels ABSENT | a real radio submenu calling `set_atlas_tile_size()` |
| Documentation (listed twice) | live via `OS.shell_open` when a source checkout resolves |
| **Slot transform — scale · fit · reset** | **the row's reason was FALSE for a week**: `as_set_item_transform` is at `cartalith-godot/src/lib.rs:10811` and `asset_library_window.gd:1703` has been calling it on every slider tick, with Fit and Reset at `:2485`/`:2488`. Same shape as `Show tile borders` below: not stale wiring but a stale *reason*, which `audit_wiring.py` structurally cannot see, because every `#[func]` involved IS called and it is the tooltip that lies |

And the rows closed by building them, same day:

| Row | What closed it |
|---|---|
| Find on map… (⌘F) | `shell/place_search.gd` — 162 entities on a 192×144 world, off five live getters |
| Refine detail for the current view | `ViewportHost.visible_grid_rect()`; measured, 20 chunks to LOD 5 in 0.63 s |
| Show tile borders | live — **and its stated reason was factually wrong.** It is not a fourth chunk-debug toggle: the reference's `#lodShowGrid` (line 1281) sets `_showExportGrid`, draws `drawExportTileGrid()` (line 9602) as a dashed `refCols × refRows` split of the whole map, and its call site is `if(_showExportGrid && !_lodOn)` (line 8658) — it draws when the pyramid is DOWN, the opposite of the three toggles it was filed beside |
| Deselect (⌘D) | `DccApp.clear_selection()` + the `icon_deselect()` binding icons never had |
| LOD levels 0–8 | `DccSettings.bake_depth()` — the blocker was a private field, so it stopped being one |
| Tiled LOD · auto / manual | three functions on `ViewportHost`, shipped **with** `Enter deep detail now`, because a suppressor with no way in is worse than no choice |
| Lighting rig defaults | `DccSettings.lighting_defaults()` — the row's own reason named this fix and left it unbuilt |

**Counts now: 21 rows genuinely open** — 3 trivial, 6 small, 6 medium, 6 large —
after deduplication. Every one below was re-verified against the code on
2026-08-30.

---

## Trivial

| Item | Where | Proposal |
|---|---|---|
| "Arming a tool drops the sheet to a peek" | MAP tab tool chips. The prototype states it on the label itself (`TOOLS · ARMING DROPS THE SHEET TO A PEEK`) and implements it in `hTool` (`detent: t==='inspect' ? current : 'peek'`). `_set_phone_detent()` has no caller outside `dcc_shell.gd` | Expose it as a small public `phone_peek_sheet()` and call it from whichever function arms a tool, for every tool except Inspect — arming a tool means the user is about to touch the map, and the sheet is what covers it. One line at the arming site; the detent machinery is already in place |
| Letter-spacing is pinned but `ROLE` cannot carry it | `dcc_theme.gd` — `mono(spacing, medium)` takes tracking as an argument | **No change needed**, recorded so the next reader does not add it. The tablet artboard draws .12/.14/.18/.20/.22/.26 em exactly as the desktop one does; tracking is genuinely pinned at ×1.00, so a `ROLE` row would be two identical numbers behind a resolver that can only return one. Tracking already comes from an explicit argument at every call site and cannot be scaled by accident |
| `TABLET` and `ROLE` state the six region boxes twice | `dcc_theme.gd:286` (`TABLET`, consumed by `dcc_shell.gd:413` `_scaled()`) and `ROLE`'s six region rows | Deliberate and flagged in the code. `TABLET` answers `_scaled(px)` for a caller holding only an integer; `ROLE` answers a caller that knows which region it is building. The figures are identical and the comment says they must move together. The single-source fix is to rewrite `_scaled()`'s call sites to name their region and derive `TABLET` from `ROLE` — until then it is two names for one measured value, not two competing values |

## Small

| Item | Where | Proposal |
|---|---|---|
| Deselect's sibling: **Select all** | `menus.gd:526` (`_todo`) | Every selection here holds exactly one item — `icon_get_selected` and `label_get_selected` each return a single index — so there is nothing to select *into*. Needs a multi-selection model first. Deliberately NOT closed alongside Deselect: clearing one item needs no such model |
| Documentation (in-app) | `menus.gd:1995` — live when a source checkout resolves, disabled with its reason otherwise | §2.7 lists it first. It opens the repository's documents via `OS.shell_open`, which is a real behaviour, but there is no in-app manual and §2.7 names no URL. An exported build ships `res://` inside the `.pck`, so the row is honestly disabled there. Closing it needs a published documentation URL from the owner, or docs bundled into the export |
| Fully-dismissed sheet state (prototype `tab:null`) | `dcc_shell.gd:2607` and `:3002` — both collapse to `peek` where the prototype removes the sheet | Declined on purpose and recorded in both functions. This sheet is the desktop's tool options bar (§13: "tool options become a bottom sheet"), the one row always on screen on desktop and tablet, so a state where it is absent has no counterpart to keep parity with. If the owner wants true dismissal it should come with a way back that is not a tab — the prototype has none, its bar just re-lights one |
| LOD levels ladder ▸ its **other half** | `menus.gd:1601` disabled row inside `_build_atlas_tiles_menu()` | The 0–8 ladder now ships and writes `DccSettings.bake_depth()`. What remains disabled here is the pointer row naming its owner, kept so a reader who opens the tiles submenu learns where the number lives rather than finding two ladders |
| Report an issue | `menus.gd:2017` (`_todo`) | §2.7 lists it. Pairs naturally with the already-live `Generation info…`, which dumps every generation parameter as plain text: an issue route that pre-fills that dump plus version and build is the useful shape. **Blocked only on the owner naming a destination** |
| Sheet header row (title · subtitle · ← back · ✕ close) | Phone tool sheet, under the grab handle. The prototype draws it inside the same `grabRef` block (`Cartalith Android.dc.html` line 176ff: `sheetTitle` 11px/.2em accent, `sheetSub` 9.5px dim, two 38px round chips). `dcc_shell.gd:2759` goes straight from handle to scroll | A 38 dp row under the handle: accent 11 px/.2em title and 9.5 px dim subtitle fed from the active tab (the prototype's own `titles`: MAP · "layers · style · annotation"; GENERATE · "pipeline · seed NNNN"; PLAN · "journey · A → B"), a leading ← chip shown only when a depth exists (`PhoneMenu` already tracks it with `moreStack`), and a trailing ✕ that collapses to peek rather than closing, matching the re-tap already wired. The title text is the only new state |

## Medium

| Item | Where | Proposal |
|---|---|---|
| Atlas cache — size cap in GB | `menus.gd:1654` (`_todo`) | §2.5: "Size cap in GB + Clear." Clear is live; the cap is not, and **the blocker is eviction, not measurement** — `bake_bridge` writes chunks and only `atlas_clear()` removes them, all at once, with no access order or level priority to evict by. `atlas_status()` already reports bytes. The GB ladder itself should mirror Performance ▸ VRAM budget, which solves the presentation half; the policy question (which chunk goes when the cap is hit) is the real work. Filesystem mtime is available as a least-recently-written proxy and is worth costing before inventing a record |
| CPU worker threads | `menus.gd:1103` (`_todo`) | §2.5: "Integer, 1…logical cores, default cores − 4 (`12 of 16`)." Rayon sizes its own pool today. Wire as a ladder (1 · ¼ · ½ · cores−4 · all) calling `ThreadPoolBuilder::num_threads` at pool init. The spec's own default, cores−4, is a real number to honour |
| Preferences ▸ Units (km / mi) | `menus.gd:1231` (`_todo`), re-presented on the phone under MORE ▸ System | The reason is the honest one: the shell is km-only, and a setting with five call sites still printing km is worse than no setting. The work is one formatter (`DccSettings.distance(km) -> String`) plus its five consumers — the status bar's cursor coordinates, the scale bar, Sculpt's brush km equivalent (`#sBrushKm`), Measure's running total and per-segment lengths, and Region select's km column — after which this becomes a two-value radio beside Theme, and the phone gets it free because `phone_menu.gd` re-presents the same popup. Theme dark/light, the other half of `ANDROID_UI_SPEC.md`'s clause, IS wired |
| Redo (⌘⇧Z) | `menus.gd:462` (`_todo`) | §2.2 lists Redo as the mirror of Undo. The reason is correct: `undo_last()` pops the snapshot rather than moving a cursor, so global redo needs `undo.rs` turned from a stack into a cursor over history. The Sculpt draft's own Redo already exists and the tooltip points at it |
| Save layout as… | `menus.gd:1921` (`_todo`) | §2.6 lists it beside Reset layout. Needs a layout store: dock widths, collapsed state, visible regions, active domain. `DccSettings` already persists window state, so this is a named-preset section over data the shell already has |
| Simulation — the "mini transport strip overlay" | `phone_menu.gd:669` routes MORE ▸ Simulation to CIVIL ▸ Simulation, which is real. The overlay itself exists nowhere | The row is wired to a real destination, so nothing is dead — but §10 gives the overlay's full desktop form (a scrub track across the year range with the current year in accent, ▶ ⏸ step ◀ ▶, ×1/×10/×100, a run-state readout, and six layer toggles) and it is not built on either platform. For the phone: a peek-detent overlay above the bottom bar reusing `timeline_bar`, carrying year · ▶/⏸ · step · speed, with the six toggles behind the drag handle at half detent. **It must drive the same year cursor `civilization_workspace.gd`'s Politics category already owns**, not a second clock. `app.gd`'s own strip comment says the controls "deliberately live in the CIVIL dock's Timeline category instead" — this revisits that, so it needs the owner's word first |
| Which controls leave the tablet — the content decision | Owner call, not a file. `GUI_GAP_REGISTER.md` §57; affects `_build_right_dock` and `layers_popover.gd` | **Take it to the owner before building.** The tablet artboard DELETES roughly 30 % of the desktop's content — the entire PROPERTIES section, the six per-layer opacity minis, the scale bar, the layers popover, 2 of 6 layer rows and 4 of 13 sample fields — and that deletion is what buys room for 44 px rows. No table can express a deletion, so `ROLE` does not attempt it. Concretely: a `TABLET_OMITS` set of section ids that `_build_right_dock` skips under `DccTheme.is_tablet()`. The membership list is not derivable from the spec — the four sample fields it drops (aspect, drainage, soil, control) versus the four it keeps record an answer somebody already gave |

## Large

| Item | Where | Proposal |
|---|---|---|
| Cut · Copy · Paste (⌘X ⌘C ⌘V) | `menus.gd:505`, `:509`, `:510` (three `_todo`s) | §2.2: "Operate on the current selection (labels, icons, places, stamps)." All three share one prerequisite — a clipboard representation for each entity kind — which is why they are one row here rather than three. `PARITY_AUDIT.md` §20 records them as "three separate pieces, not one selection abstraction" |
| 3D viewport defaults | `menus.gd:1161` (`_todo`) | §2.5's four parameters verbatim — relief exaggeration, detail, light, flatten oceans — replacing `#genV3dSec`, exempt from the finalize lock. There is no 3D viewport; `DECISIONS.md` §4 defers it to Phase 3. Honour the four names and the lock exemption verbatim when it lands |
| Anti-aliasing · anisotropy | `menus.gd:1126` (`_todo`) | §2.5: "off · MSAA 2× · 4× · 8×; anisotropy 1–16." Both are 3D-viewport settings, deferred with it. Do **not** bolt MSAA onto the 2D map path, which composites whole rasters and where a sample count means nothing |
| Colour management | `menus.gd:1128` (`_todo`) | §2.5: "sRGB · Display P3 · linear." The renderer is sRGB-only end to end — `render.rs` writes 8-bit sRGB bytes and nothing carries a colour space to the texture. Blocked until it does; a three-row radio that always resolves to sRGB is the enabled-and-inert shape this menu forbids |
| Check Data — current warning count | `data_manager_window.gd:148` — the badge column exists and is drawn for other routes | §2.4: "Check Data (shows current warning count)." Blocked on a validation pass to count: `load_save()` reports pass/fail only. The badge should stay **empty rather than faked**, and lands with the validation engine work |
| Keyboard shortcuts… (editable, per-context) | `menus.gd:1233` | §2.5 asks for an editable table. The dialog is live and lists every shortcut by walking the real MenuBar; what is missing is *rebinding* — a per-context store and an input-remap layer. (This row's own earlier note, that its tooltip was stale, is itself now stale: that fix already shipped) |

---

## Two footnotes, so the `_todo` census reconciles

**Four `_todo` calls in `menus.gd` have no row here on purpose.** `Devices`,
`Multi-GPU mode`, `VRAM budget` and `Fallback when VRAM full` (`:1101`, `:1102`,
`:1109`, `:1110`) fire **only** when `_bridge.gpu_api` is false — a GDExtension
older than the multi-GPU API. `gpu_enumerate_devices` is bound in the current
engine (`cartalith-godot/src/lib.rs:2638`), so against any current build those
four are unreachable and the live builders run instead. A defensive fallback for
a stale binary, not a standing gap.

**Two disclosed gaps bypass the `_todo` helper** and so do not appear in a
grep-based census: `Alternate frames` (`menus.gd:1401`) and `Reduce working res`
(`:1487`) are radio options disabled directly, each with an honest tooltip.

---

## Not gaps — recorded so they are not re-listed

- **`Data ▸ Conversion` is absent by owner decision** (2026-08-20). The spec's
  §2.4 still draws it, so the **spec** is the stale party there, not the shell.
  It must not be re-added.
- **The nine colour ramps are exact and complete**: Earth, Elevation, Atlas,
  Mono, Imhof, Ice, Dark ice, Desert, Dark atlas (`render.rs:610-619`), matching
  §7 one for one.
- **The planner's 15 per-stage overrides are all present**
  (`journey_planner_view.gd`, 18 `_override_*_row` calls): transport,
  group_size, cargo_kg, pace, hours, weather_override, carry_food, supply_days,
  grazing, foraging, route_cond, infra, mount_animal, desert_water, vessel —
  exactly `ANDROID_UI_SPEC.md`'s fifteen.
- **The MORE list is built to the spec line verbatim** (`phone_menu.gd:80-82`).
- **Tablet parity is measured, not asserted**: `_tabletparity_probe` boots
  2560×1600 touch and 1920×1080 in one run — docks 400/400, rail 48, bands
  52/88/36, and the same seven menus with **223 reachable rows on both**.

---

## One question for the owner: the style presets are named differently

`DCC_SHELL_SPEC.md` §4 and `ANDROID_UI_SPEC.md`'s MAP line both ask for preset
chips **Atlas / Parchment / Physical / Ink**. `render_workspace.gd:111` ships
the reference's own six — **Natural Vibrant · Default · Antique · Ink ·
Watercolor · Print** — which are the looks the renderer actually has.

Mapping: Ink = Ink. Antique ≈ Parchment (its own label is literally "Antique
Parchment"). **Atlas and Physical have no counterpart at all**, and the spec's
*Atlas* may be a conflation with the colour RAMP of that name, where Atlas and
Dark atlas are both real.

This is a vocabulary decision, not a code gap: rename the shipped presets to the
spec's words, or keep the reference's? Inventing an "Atlas" and a "Physical"
preset would be two chips with no look behind them, which is the one thing this
table exists to prevent.

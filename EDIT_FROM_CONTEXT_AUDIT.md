# Edit any map element from its context -- step 1 audit (2026-11-11)

Owner request (`OUTSTANDING_WORK.md` §2.12, row "Edit any map element directly from its context"). **Findings and
plan only; no status column** (`STATUS.md` owns status). Read at the symbols on origin/main `4a9d5bcd`:
`map_overlay.gd::hits_at`, `_pick_mark`, `context_broker.gd::engine_picks`, and every `context_actions(req)`
provider (`global_tools.gd`, `civilization_workspace.gd`, `cartography_workspace.gd`, `world_workspace.gd`).
Not yet measured by a windowed probe: this document says what the code does; step 2's probes prove the fixes.

## How a hit works today

`hits_at(mouse)` returns every pick under the pointer, nearest first. Its kinds are exactly six: `settlement` and
`landmark` (the overlay's own pin and ring tests), `label` and `icon` (the engine's `label_pick_all` /
`icon_pick_all`, through `engine_picks`), `way` (CIVIL only, `way_pick`) and `river` (WORLD only, `river_pick`).
The broker merges every provider's rows. **Rows come from the active domain's provider only**, so a hit with no
row in that domain's provider only titles the card and fills Select ▸. A route is not a hit kind: CIVIL's provider
picks it locally (`_route_hit`).

## The table

| Element | Hit on the map? | Card offers today | Missing, and the path to use |
|---|---|---|---|
| Settlement | Yes, `settlement` (`_hit_test_settlement`; a tap selects) | **CIVIL only**: Edit, Move viewer to, Open in city viewer, Delete (`civ.delete`), vault note (`civ.vault_note`, attach/open), Select culture. WORLD: nothing. CARTO: only "Label this" | Rename, Move, Delete, Note in WORLD and CARTO. Paths: `place_editor_window.gd`, the `_run_ctx` arms, `app.open_vault("settlement", tid, name)` |
| Settlement name beside a pin | Not separate: it is the settlement | as above | nothing beyond the settlement's rows |
| Label (the engine's label store: feature, region, custom) | Yes, `label` (`label_pick_all`); only labels in that store | CARTO only: Edit text (`carto.label_edit`), Delete (`label_delete`) | Move (`bridge.label_move` exists, no row), inline Rename, Note; every row in WORLD and CIVIL; one undo step each |
| Icon | Yes, `icon` | CARTO only: Delete (`icon_delete`) | Move (no `icon_move` in `engine_bridge.gd`; the Icon tool drags), Note |
| Landmark | Yes, `landmark` | CIVIL only: Inspect, Why here? | Rename (user name, Ruling EL), Open or create note (`context_note.gd` already binds `landmark` through `vault_landmark_entity_id`), Delete/Move: generated, none by design. WORLD and CARTO: nothing |
| Way (road, track) | Yes, CIVIL only | CIVIL: Inspect, Delete (`way_delete`) | Rename (only if a way has a name), hit in WORLD/CARTO |
| River | Yes, WORLD only | Inspect, Trace, Catchment, Clear | Nothing to edit by design (generated, unnamed). Another lane owns river drawing: stay off it |
| Region (export region, territory draft) | **No hit**; drawn by their tools | CARTO: "Start export region here"; CIVIL: Commit/Discard territory draft | A pick for a finished region is a new hit source: a design, not a row |
| Conflict (marker, front, arrow) | **No hit** (`set_selected_conflict` is driven from the list) | CIVIL: "Add event to timeline here" only | A pick over conflict geometry, then Select, Delete, Move; conflicts are not note-bindable (`context_note.gd::BINDABLE`) |
| Journey / route | Picked locally in CIVIL (`_route_hit`, 6 px) | CIVIL: Open in Journey Planner, Delete (`route_delete`) | Rename (`route_set_name` exists, no row), Select |
| Journey markers (planner) | **No hit** (`set_journey_markers`) | none | decide whether they are elements or part of the route |
| Sample pin | Own touch state machine (`_sample_pin`) | none | not a context target; has its own drag and drop |
| Colour-paint mark | **No hit.** `colour_paint_tool.gd` is a layer list (select, rename, delete, clear); marks have no identity in the UI | none | layer-level only; a mark pick needs mark geometry: a `COLOUR_PAINT_SCOPE.md` row, not a card row |
| Notes | Not drawn as pins; they hang off settlement, faction and landmark | Note row only on a CIVIL settlement | one "Note" row for settlement, faction and landmark in every domain |
| Culture / faction | No hit of its own | CIVIL's settlement card: Select culture, Open faction | reachable through the settlement |
| Sculpt stamps | via the sculpt draft's list | WORLD: Select, Move up/down, Delete | complete |

## Structural gaps

1. **Rows are per-domain, hits are not.** A settlement, landmark, label or icon is hit in every domain but has verbs
   in one. Fix: one shared "element" provider (in `GlobalTools`) that every domain asks. Each row only calls the
   existing path; nothing is reimplemented.
2. **Landmark Rename does not exist** although `landmark_set_user_name` does, session-only. Ruling EL item 1 makes
   it a saved optional slot (`SAVEFILE_COMPAT.md`, `cartalith-project`, `cartalith-godot` state), old saves load
   unchanged, round-trip byte-exact, tested. Label Move (`label_move`) has no row either.
3. **Note rows exist only on a CIVIL settlement**, while `context_note.gd` already creates and binds notes for
   settlement, faction and landmark by `KnowledgeLink`.

## Plan for step 2, one verified commit per family

1. Landmark: Rename (with the save slot), Note, Select, in every domain.
2. Settlement: Rename, Move, Delete, Note in WORLD and CARTO through CIVIL's own calls.
3. Label: Move, Rename, Note; icon: Move, Note.
4. Route and way: Rename, Select in every domain.
5. Touch, **Ruling EO**: a tap selects and shows the element's chips (Move, Delete, Rename, Note); a long-press keeps
   the full card. The chips are the card's own rows (`chip` key, `lead_chip`).
6. Separate designs, not rows: region, conflict and journey-marker picks, colour-paint mark picks.

Destructive rows confirm or are one undo step; every action uses its own existing path and undo group.

## Bars for step 2

Windowed probes with real right-clicks (desktop 1600x1000) and real touches (tablet 2388x1668@2; phone 1080x2340,
`--density 2.8125 --force-touch`) per family and action including undo; the landmark-name save round trip; literal
mutants; and a re-run of `_ctxcard`, `_ctxring`, `_ctxfan`, `_ms5`, `_ms5b`, `_pc4context`, `_pc5note`, `_ve6`,
`_ve9b`, `_culturesel`, `_scrollfix` against origin/main.

## Build notes, step 2 (2026-11-11)

Built as one change, not one commit per family (the families share `global_tools.gd`). The shared provider is
`GlobalTools.element_rows`. Deviations from the plan above, each for a reason read at the symbol: **settlement Move**
is not built (no engine path relocates a settlement); **label and icon Note** are not built (`SAVEFILE_COMPAT.md`
13.3.4 fixes the bindable kinds); **way Rename** is not built (no setter; only a route has `route_set_name`);
**Select** rows exist for landmarks only (a way, label, icon and route are not selection kinds); **icon Move** needed
a new engine call, `icon_move`. Undo: labels, icons and names are on no undo history, so Delete confirms and Rename /
Move are reversed by repeating them. The touch chips are `shell/element_chips.gd`; the strip is placed at the tapped
point, because the host fills the window and the bars are drawn over it. Probe: `_edfc_probe.gd`.

## Build notes, step 3: the phone form and the Note tab (2026-11-12)

Findings and what was built. Status is `STATUS.md`'s ("Edit any map element from its context: step 3"). Read at the
symbols: `map_overlay.gd::_release_touch_press` / `request_at` / `element_tapped`, `DccApp._on_element_tapped`,
`DccApp.select`, `ElementChips`, `RightDock.show_subject` / `shown_subject` / `_build_label` / `_element_actions`,
`ContextNote.BINDABLE` / `resolve`, `ContextBroker.engine_picks`, `GlobalTools.element_rows`.

### What the phone's tap was doing (three defects the first phone probe pass found)

1. **The chips sat under the right sheet.** A tap selected the element, and on the phone a selection ends in
   `RightDock._show_on_phone()`, which opens the right sheet over the map; the chips are children of the map host, so a
   finger on the Rename chip pressed the sheet's "Why it is here" button. The existing phone legs passed only because
   they asked `chips.visible`. Fixed: with chips up the sheet is closed (`DccShell.phone_hide_right_sheet`); a tap that
   made no chips keeps the sheet it opened.
2. **The chips were not phone-sized.** The strip is not scaled by the phone's UI unit (`_phone_scale`, short side /
   412), so 44 "dp" measured 55 px on a 1080 px screen (about 20 dp). `ElementChips.show_for` takes the unit; the strip,
   its type and its offset from the fingertip are multiplied by it. The Context page's action buttons floor to
   `_ptap(44)` for the same reason.
3. **One tap pressed a chip twice.** A touch reaches a `Button` as a ScreenTouch and as its emulated mouse button, and the
   button answered both: two Rename prompts. A press on a hidden strip is now ignored.

A fourth, smaller one: renaming a landmark that was already selected called `select()`, which re-opened the sheet; it
redraws through `RightDock.refresh_shown()` now.

### The Note tab: label and icon are selection kinds

| | |
|---|---|
| `DccApp.selection` kinds | settlement, landmark, faction, culture, conflict, note, **label, icon** |
| id | the index in the engine's store (`label_get` / `icon_get`); an index the store does not hold selects nothing (never a neighbour) |
| producers | a desktop **left click** on a label or icon (`map_overlay.gd::element_clicked`, Inspect armed, nothing else under the pointer), a touch **tap** (`_on_element_tapped`, before its chips), an icon pin the overlay already picked (`icon_selected`) |
| Context panel | Inspect page for a label (`_build_label`: text, cell, size, the card's own verbs as buttons) and for an icon (`_build_icon`, now with the same buttons); header, Note tab and the pin follow `shown_subject()` |
| Note tab | `ContextNote.BINDABLE` gains label and icon; `resolve` asks the engine for the vault entity id (`vault_label_entity_id` / `vault_icon_entity_id`: the item's cell, never the index) and says why when two items share a cell |
| renumbering | `EngineBridge.annotations_removed(kind)` (after `label_delete`, `icon_delete`, `label_clear_all`, `icon_clear_all`) drops a selection of that kind, because every later index shifted |
| phone | the sheet has no tab strip, so its equivalent is the label or icon page's Note row (`Act_el_label_note`, at least 44 dp) and the Note chip, both through `app.open_vault` |

Not done, deliberately: `annotations_removed` is emitted by the four calls above only. A pass that replaces the icon list
wholesale (a regeneration; not read at its symbol, so a suspicion) is not wired to it, and a selected icon could then name a
neighbour until the next pick; and
Alt+Left can step back to a label or icon subject whose index has since shifted (a settlement's has the same hazard).

### Per element, on the phone (tap = chips, long press = the card)

| Element | Tap shows | Notes |
|---|---|---|
| Label | Move, Rename, Note, Delete | Delete confirms (no undo history) |
| Icon | Move, Note, Delete | Rename: an icon has no text |
| Place (settlement) | Rename, Note, Delete (CIVIL, WORLD, CARTO) | Delete confirms; **Move not offered**: the engine has no relocation |
| Landmark | Rename, Note | generated, so no Move or Delete by design |
| Way (CIVIL) | Delete, only when the way is the nearest hit | undoable (Ruling BA), no confirm. **Rename not offered**: no setter. A finger's pick is 22 px (`PICK_LINE_TOUCH_PX`), the mouse's stays 6 |
| River (WORLD) | no chips | generated and unnamed: nothing to edit; Inspect and Trace are on the card |
| Region | no hit | still a design, not a row |
| Route | no chips | not a hit kind (`_route_hit` is local to CIVIL's provider); its Rename and Delete are on the long-press card |

### Undo for label, icon and name edits: not built; what it takes

Checked at `undo.rs::EntryKind`, `context_pick_bridge::choose_undo`, `lib.rs::undo_last` and `shell/undo_group.gd`: the
engine's history is a ledger with one entry kind per undoable thing (height snapshot, way delete, way drag), and
`undo_last` picks the newest of the height and way stacks. Labels, icons and landmark names are in none of them. A real
undo needs (1) a new `EntryKind` for an annotation edit holding the before-state of the label store, the icon store or
the landmark name map (a clone of a small Vec, so the memory is not the cost); (2) `choose_undo` growing a third arm so
"newest wins" holds across three stacks, and a redo tail for it; (3) `undo_group.gd`'s member kinds and
`edit_state_digest` taking it, so the assistant's grouped undo and its "world hashes equal" bar stay true; (4) the shell's
Edit menu label and the history ledger row. It is a Rust change with a golden-neutral surface but five touch points, not
a small step, so it is filed here and not built. Until then Delete confirms and Rename and Move are reversed by repeating
them.

### Probe

`_edfc_probe.gd` now runs on the phone (1080x2340, `--density 2.8125 --force-touch`). The earlier phone failures were the
probe's: its domain helper used the MAP tab (which selects CARTOGRAPHY), it emulated a prompt's OK without hiding the
dialog, it ran row callables without closing the peek card, it picked pins under the layers button, and it read a
destination cell after the view had re-fitted. New legs: WY (the finger's way pick and the way chip, deleted and undone)
and NT (selection kinds and the Note tab).

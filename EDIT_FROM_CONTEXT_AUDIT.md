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

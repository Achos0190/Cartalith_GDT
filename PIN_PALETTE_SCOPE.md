# Pin palette — placing pins by hand in CARTO (Ruling FD, 2026-11-15)

Defines, does not track: status is `cartalith-native/docs/STATUS.md`'s.

## Why
The owner wants to place any pin from the full icon sets by hand (Ruling FD). Today the CARTO Point of interest
tool (CL-11, `CITY_MAP_LOOK_SCOPE.md`; `poi_bridge.rs`, `map_overlay.gd::_draw_poi_marks`) offers twelve fixed
`PoiKind`s (`cartalith-civ/src/poi.rs`). The icon sets (`cartalith-native/godot-project/icons/pins/`) now carry
about sixty kinds, and `PinMarker.KIND_ICON` / `icon_name_for` already map a kind name to an icon.

## Design
- **Tool and palette.** A Pins tool in CARTO's tool strip (desktop and tablet; the phone's CARTO tool sheet). While
  active, the right dock shows the palette: a search field, a recents row, and collapsible categories matching the
  icon colours (Settlements; Buildings & trade; Military; Sacred & historic; Water; Terrain; Land & hazards; Other).
  Icons render in the current marker style; with Circles or drawn Pins the palette shows the painted icons with the
  style's name beside them.
- **Placing.** Pick an icon, click the map to drop; the tool stays armed until Esc or re-picking the tool; dragging an
  icon onto the map also drops. After a drop the pin's card opens with Name focused and its Note section (vault
  folder choice as in "Where a new note goes"). Every drop, move, rename, change of kind and delete is one undo step.
- **Editing.** Drag a placed pin to move it; its card (right-click / long-press) has Change kind (reopens the palette),
  Rename, Delete. Generated landmarks use the same card and palette (Ruling FD item 3).
- **Kinds.** A user pin stores a kind **name** (any icon name, or a free-text label) instead of a fixed enum; old
  saves load their `PoiKind` as its name. A free-text kind draws the question-mark (`other`) or exclamation-mark
  (`notice`) pin, the user's choice. Saves: `annotations/pois.json` gains `kind_name` and `free_label`; a save
  without them loads as today.
- **Visibility.** User pins inside a town follow Ruling FC (hidden below the town-layout zoom, always in the City
  Viewer). Tilting follows Ruling FA.
- **Phone.** The palette is a bottom sheet with category tabs and search; the recents row sits in the sheet's peek.
  Tap an icon, then tap the map. 44 dp targets. (As built, PP-6: the sheet's body is hidden at its peek detent, so the recents
  row is the tool strip's second row, the surface that stays on screen above the sheet.)

## Milestones
- **PP-1** `notice` icons (glyph and painted) drawn to the sets' standard.
- **PP-2** Engine: user pins with a kind name and optional free label; save and load, old saves unchanged; tests.
- **PP-3** Desktop palette in the right dock: search, recents, categories, icon grid in the current style; arm and drop.
- **PP-4** Card: Change kind via the palette, free-text kind with ?/! choice; generated landmarks share the card.
- **PP-5** Drag-and-drop from the palette; drag to move; undo for each step.
- **PP-6** Tablet dock and phone sheet forms.
- **PP-7** Inn as a kind (with the routed `PoiKind::Inn` row) and a glyph `inn` icon.

## Bars
Probes on desktop, tablet and phone forms: a drop lands at the clicked point (tip within 1 px), the palette lists
every icon in both sets, search finds by substring, recents keep the last 8, undo restores each step, an old save
loads unchanged, a free-label pin round-trips, user town pins hide under Ruling FC's threshold. Mutants on each.

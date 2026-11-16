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

## Custom pins (Ruling FP, 2026-11-16)

Owner: *"under assets we should maybe create a category for the pins. So a user can add their own they so please."* Answers (`LARGE_ITEM_RULINGS.md`, Ruling FP): each added icon is a new placeable kind under a *Custom* category with a name and colour the user chooses; custom pins travel in asset packs; they live in the user's asset library and work in every world, and each world saves which custom pins it uses so it opens and draws elsewhere.

### Design (one kind registry, not a parallel path)
- **A custom kind id** is `custom-<slug>-<8 hex>`. A hyphen never occurs in a built-in kind name (`cartalith_civ::poi::KIND_NAMES`, all `[a-z_]`), so an id can never equal a built-in; `is_valid_kind_name` accepts the shape, so a custom pin is an ordinary `Poi::kind` through the same `poi_add`, `poi_set_kind`, undo and save as every other kind. The hash is the library's, over slot id, name, colour and the normalised icon: **editing a pin gives a new id**, so a pin already placed keeps the copy its world saved (Ruling FP item 3) and two different pictures can never share an id.
- **Library.** The *Pins* category of the asset library is **not a new engine family**: it is the open Custom family's reserved set `Pins` (`cartalith_assets::pins::PINS_SET`), so it is stored, listed, sliced, tagged, exported and merged by the machinery every custom icon already uses. `SlotMeta` gains `color`. The window shows `pins` and `custom` as two categories over one engine list (`_family_slots`), so no icon shows in both. *Add pin...* takes a PNG or an SVG (Godot rasterises the SVG at 256 px; the engine reads PNG only) and asks for a name and a colour; the inspector's *Pin* section edits both; the ordinary Replace, Delete and the slicer (opened from Pins it targets the set) work as for any custom icon.
- **Icon.** The picture is the slot's first item, normalised (`asset_bridge::normalise_pin_icon`): trimmed to its opaque bounds, scaled so the longer side is 128, centred and **bottom-aligned** on a 128 x 128 canvas, because the shell draws every pin picture with its bottom-centre pixel at the pin's point. Idempotent, which is what lets a pack carry the normalised picture and re-import to the same id.
- **Packs.** `pack.json` gains an additive `pins` section, `{"<slot id>": {"name", "color"}}`, next to `custom.Pins`; absent when the library has no pin, ignored by a reader that does not know it. A bad colour, empty name or missing art is refused per entry with a warning, never repaired.
- **World.** The open world holds a registry of the custom pins its points use (`WorldGen::custom_pins`), filled by `poi_register_custom_pin` before a pin is placed or re-kinded, saved as `annotations/pois.json`'s optional `custom_pins` (id, name, colour, base64 PNG; only the ids still in use; omitted when none; `SAVEFILE_COMPAT.md` 11.2a). The registry is not in the undo snapshot (an entry is content-addressed, so a stale one is harmless) and is pruned at save. A point whose custom id has no picture is **kept** and draws `PinMarker`'s missing-icon marker (a grey pin, its head crossed out), never a built-in; its words say *Custom pin (icon missing)*.
- **The library outlives a world opening and a restart.** The asset library is a session store (its images are in no world archive), and opening a world replaces it with that archive's pixel-less record, which would have wiped every pin. The restore (`asset_library_restore_document`) therefore snapshots the pins and puts them back (`pin_snapshot` / `pin_restore`), and the pins alone are written to `user://custom_pins.zip` as an ordinary asset pack when they change and read at start (`as_pins_store_bytes` / `as_pins_store_load`; `app.gd::_save_pin_store`, `_load_pin_store`).
- **Shell.** `PinMarker` is the one drawing path: `shape()`, `draw()`, `draw_icon()` and `draw_ring()` route a custom kind to its registry entry in **every** marker style (Icons, Painted and Pins draw the user's picture, Circles a disc in the user's colour holding it), so the footprint that is drawn is the footprint that is picked. `PinPalette` gains a dynamic ninth category, *Custom*, from `load_catalog(EngineBridge.as_pin_catalog())`: it exists only while the library holds a pin, so the eight-category table and its probe are unchanged; tiles carry the pin's colour as their frame; search matches the name; the phone gets a *Custom* chip; 44 dp floors as for every tile. Landmark change mode never shows it.

### Build note (2026-11-16)
Rust: `cartalith-civ` `poi.rs` (id shape, colour rule, `CustomPin`, base64), `cartalith-assets` `pins.rs`, `manifest.rs` (`pins`), `pack_import.rs` (`PlannedSlot::pin`), `library.rs` (`SlotMeta::color`), `cartalith-godot` `asset_bridge.rs` (`pin_catalog`, `pin_add`, `pin_update`, `pin_snapshot`, `pin_restore`, `pins_store_bytes`, `normalise_pin_icon`, `pin_kind_id`, export and import branches), `poi_bridge.rs` (`poi_register_custom_pin`, `poi_custom_pins`, the document), bound in `lib.rs` (`as_pin_catalog`, `as_pin_add`, `as_pin_update`, `as_pins_store_bytes`, `as_pins_store_load`); `project_bridge.rs` (the restore keeps the pins). Shell: `pin_marker.gd`, `pin_palette.gd`, `asset_library_window.gd`, `cartography_workspace.gd` (`ensure_custom_pin`, `reload_pin_catalog`), `viewport_host.gd`, `app.gd`, `engine_bridge.gd`. Probe: `_custompins_probe.gd`.

**Left, stated:** (1) only the pins have a per-install store; the rest of the asset library is still a session store (its images are in no world archive, `SAVEFILE_COMPAT.md` 16.5), which is unchanged. (2) The colour frames the palette tile and fills the Circles disc; the Icons, Painted and Pins styles draw the user's picture as it is (no tint). (3) A pin renamed in the library is a new id: pins placed earlier stay on their saved copy and are not relinked to it. (4) A device pass (the Pad and the 6T) was not done.

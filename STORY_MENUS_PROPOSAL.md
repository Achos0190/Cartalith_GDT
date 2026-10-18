# Story menus: proposal

**A proposal of 2026-10-17, from an owner request; scheduled by Ruling CK
(2026-10-18, Q1–Q7 at their defaults).** It defines milestones (SM-1…SM-7) and
the reasoning behind them. It has no status column: status lives in
`cartalith-native/docs/STATUS.md` and nowhere else.

**The request, as briefed (2026-10-17).** Reorganise the menus and the workflow
so that Cartalith serves worldbuilders, writers and dungeon masters who want to
plot stories and keep track of their world's structure. Draw it in the current
design scheme, and audit every control against the engine.

**Mockup:** `design/story-menus-2026-10-17/mockup.html`. It is one static page
with no external files and a *Dark / Light* switch. It has eight artboards and
an inventory table:

- A1: the current menus
- A2: the proposed menus
- A3: the rail
- A4: the context card
- A5: Places without notes
- A6: the workflow
- A7: tablet
- A8: phone

The inventory table is generated from the same row data the artboards draw.
Hover a row to see its tooltip or disabled reason, and the symbol it was checked
against. PNG captures of every artboard sit beside it: `a1.png` … `a8.png`, plus
`a2-light.png`. They were rendered with headless Chrome and read back to check
them against the shipped shell's look.

**Evidence base.** Each fact about the code names the symbol I opened today.
Nothing here was measured. No Rust or GDScript was changed, and no probe was
run. The map, place names and counts in the mockup are placeholders.

---

## 1. The workflow it serves

Eight stages. A first session runs through them in order, and any stage can be
revisited. Artboard A6 maps each stage to the proposed entries.

| # | Stage | Question | Served by (proposed) | Gap it exposes |
|---|---|---|---|---|
| 1 | Premise | What kind of world, at what scale? | File ▸ New world…, WORLD ▸ Generate, Help ▸ Generation info… | No place for a premise note: the vault is per entity (Ruling L item 4) |
| 2 | Land | Coasts, mountains, rivers? | WORLD ▸ Generate · Sculpt, World ▸ World data tables…, World ▸ Import ▸ Heightmaps | — |
| 3 | Peoples | Who lives here, who rules? | CIVIL ▸ Factions (Populate), World ▸ Factions…, CIVIL ▸ Settlements node (**not built**) | — |
| 4 | Places that matter | Which places carry the story? | World ▸ Places…, card ▸ Open note, Places without notes…, card ▸ Open in city viewer… | **Nothing lists the places that have no note** |
| 5 | Time | What happened, and when? | Story ▸ Timeline, Political history…, Faction history…, card ▸ Add event | **The editor timeline is thin**: year snapshots and conflicts, with no free events |
| 6 | Movement | Who travels where, and how long does it take? | Story ▸ Journey planner, Saved journeys…, Travel library…, card ▸ Plan journey from here | Saved journeys have no list window |
| 7 | Presentation | What do players or readers see? | File ▸ Export ▸ Player map / DM map / Share | **No item-level "DM only" mark**; no print or VTT writer |
| 8 | Run & revise | At the table, or mid-draft | Window ▸ Session mode, Story ▸ Conflicts…, Draw a conflict, Edit ▸ Undo history… | **Conflicts are not presented as a story tool** |

## 2. What exists today (opened 2026-10-17)

- **Menus.** `menus.gd::build()` adds seven menus on desktop and phone: File,
  Edit, Assets, Data, Preferences, Window, Help. Tablet gets four: ☰, File,
  World, Data (DS-03). **Tablet already has a `World` menu**, `_world()`, which
  holds *Run pipeline* (Ctrl+R) and *Seed & size…*.
- **Data menu.** `menus.gd::_data()` holds four rows: ⧉ Data manager, World data
  tables…, Factions…, and Travel library… (Shift+L). Under them sit the four
  bands of `DataManagerWindow.ROUTES` (IMPORT, EXPORT, SOURCES, VALIDATION, in
  `GROUP_ORDER`). The EXPORT band also holds *Export map image…*
  (`ID_EXPORT_MAP_IMAGE`). After the bands, `_build_vault_rows()` adds *Markdown
  vault…* and *Browse & edit a note…*.
- **There is no File ▸ Export today.** Export lives in Data ▸ EXPORT. The brief
  said "split File ▸ Export"; this proposal creates that submenu instead (§3.1).
- **Timeline.** It has no menu row. It exists as a Window region
  (`WIN_REGION_LABELS[ID_WIN_TIMELINE]`), as the CIVIL dock category
  `Timeline` (`civilization_workspace.gd`), and as year snapshots
  (`get_civ_timeline_years`, `civ_add_year`, `civ_goto_year`,
  `civ_year_diff`). Per-place history is the Place editor's Political history
  tab (`place_editor_window.gd::_build_timeline_strip`, SP-3, with a conflicts
  lane). Per-faction history is the Factions window's History tab (FH-8).
- **Conflicts.** They are built in the engine (`conflict_bridge.rs`: add,
  update, delete, list, get, `conflicts_touching_settlement`,
  `conflict_sides_manpower`). Campaigns come from `campaign_bridge.rs::
  conflict_campaigns`. The shell reaches them in four ways:
  - the CIVIL Conflict tool (`_active_civ_tool == "conflict"`);
  - the CARTO `conflict` layer;
  - the right dock's `CTX_CONFLICT`;
  - the Factions window's Conflicts section.

  **No list of every conflict exists**, and no menu row reaches them.
- **Journeys.** The planner is a rail node, plus Shift+J in
  `app.gd::_unhandled_input`. Its menu row was retired on 2026-09-05. Saved
  journeys exist as a store (`journey_save`, `journey_list`, `journey_get`,
  `journey_delete`), but no window lists them.
- **Rail.** `dcc_shell.gd::RAIL_NODES` has three groups:
  - WORLD: Generate, Sculpt;
  - CIVIL: Landmarks, Factions, Routes & ways, Journey planner;
  - CARTO: Style, Layers, Labels, Icons.

  Ruling L's Settlements node is **not built**. It needs a fifth CIVIL mode id,
  according to the comment under `RAIL_NODES`.
- **Context card.** `civilization_workspace.gd::context_actions` already ships
  *Open city layout for X…* (`civ.open_city_layout`, which calls
  `app.open_city_viewer`). It also ships *Open vault note for X* or *Attach
  vault note to X…* (`civ.vault_note`, which calls `app.open_vault`), plus
  *Start route here*. Two of the four requested verbs are therefore live today.
- **Vault.** `vault_window.gd`'s index lists *Missing & orphan notes* (VA-01):
  links that point at no note, and notes that nothing points at. **It does not
  list places that have no note.** The data for that list exists one entity at
  a time, as `vault_entity_summary(kind, tid).link_count`.
- **Export dialog.** `export_dialog.gd::LAYERS` holds six switches: rivers,
  ways, labels, icons, territory, routes. There is a look override as well.
  `export_options.rs::FORMAT_NAMES` writes PNG and BigTIFF only. There is no
  `dm_only`, VTT or print symbol anywhere in the shell or the crates.

## 3. The menu changes, row by row

Marks: **B** means built: the binding exists, and only the row is new or moved.
**P** means partly built: some backing exists, and the gap is named. **N** means
new: there is no backing, so the row is drawn disabled with its reason, following
`_todo()`. The mockup's inventory table carries every tooltip verbatim.

### 3.1 File: a new `Export ▸` submenu (A2)

| Row | Key | Mark | Backing / gap |
|---|---|---|---|
| Export ▸ (submenu) | — | P | New submenu (derived). It gathers Data's EXPORT band by audience |
| ▸ Player map… | Ctrl+Shift+E | P | Opens `export_dialog.gd` with a players preset: routes, conflict and diagnostic layers off. **Gap:** no per-item DM-only mark exists, so a secret place still exports if its layer is on |
| ▸ DM / full map… | — | P | The same dialog with every switch on. The preset store is new |
| ▸ Export map image… | — | B | `ID_EXPORT_MAP_IMAGE` → `app.open_export_dialog()`, moved unchanged |
| ▸ Print sheet (PDF)… | — | N | Disabled: *"No print or PDF writer exists. The streaming writer has PNG and BigTIFF only (export_options.rs FORMAT_NAMES)."* |
| ▸ VTT battle map (grid + walls)… | — | N | Disabled: *"No virtual-tabletop format exists in this build: no grid calibration, wall or light data is computed or written."* |
| ▸ Maps / GIS / World data / Asset pack | — | B | The existing `ROUTES` export ids, unchanged |

The rest of File is unchanged.

### 3.2 Data renamed to World (A2)

| Row | Mark | Backing |
|---|---|---|
| ⧉ Data manager | B | unchanged |
| **RECORDS** band | | |
| Factions… | B | `ID_FACTIONS`, moved |
| Places… | B | new row → `DccShell.select_domain_category("civilization", "Settlements")` |
| Markdown vault… · Browse & edit a note… | B | `ID_VAULT`, `ID_VAULT_BROWSE`, moved |
| Places without notes… | P | new view (§5) over `vault_entity_summary` |
| Timeline | B | new row → `select_domain_category("civilization", "Timeline")` |
| **TECHNICAL** band | | |
| World data tables… | B | unchanged |
| Import ▸ | B | the IMPORT band as a submenu, same routes |
| Sources & validation ▸ | B | SOURCES and VALIDATION as a submenu. Five of the six routes are still gap pages, as today |
| *Export is in File ▸ Export* | — | `_signpost()` row |

Travel library… moves to Story.

### 3.3 Story: a new menu (A2)

| Row | Key | Mark | Backing / gap |
|---|---|---|---|
| **TIME** Timeline | Ctrl+Shift+T | B | `select_domain_category` + shows the Timeline region |
| Political history of this place… | — | P | Disabled until a settlement is current (`right_dock.gd::_settlement_index`). It then opens the Place editor on Political history |
| Faction history… | — | B | `app.open_faction_roster(-1, "history")` |
| **JOURNEYS** Journey planner | Shift+J | B | `app.open_journey_planner()`. Shift+J stays an `app.gd` key, so it is not rebindable |
| Saved journeys… | — | P | `journey_list()` is built; the list window is new (derived) |
| Travel library… | Shift+L | B | `ID_TRAVEL_LIBRARY`, moved from Data |
| **CONFLICTS & CAMPAIGNS** Conflicts… | — | P | `conflict_list()` is built; a world-wide list window is new (derived) |
| Draw a conflict | — | B | arms the CIVIL Conflict tool |
| Campaigns this year | — | B | `conflict_campaigns(year)`, the right dock readout |
| **NOTES** Notes for this place | Ctrl+Shift+N | B | `app.open_vault("settlement", tid, name)`, the card's own call. Disabled with a reason when nothing is current |
| Places without notes… | — | P | the same row as in World |

### 3.4 Window: Session mode (A2)

| Row | Key | Mark | Backing / gap |
|---|---|---|---|
| Session mode (check) | Ctrl+Shift+M | P | It would hide WORLD generation and sculpt controls and the pipeline readouts. It would keep the map, notes, timeline bar and planner. **Built:** Layouts already snapshot the region toggles, the domain and the rail (`_build_layouts_menu`, including the seeded *Journey* layout). **New:** a gate that hides controls inside a dock |

### 3.5 Shortcuts

The four new accelerators are Ctrl+Shift+T, Ctrl+Shift+N, Ctrl+Shift+E and
Ctrl+Shift+M. I checked them against:

- every `KEY_MASK` accelerator in `menus.gd`;
- `app.gd`'s keys (Ctrl+Y, Shift+J, Menu, Shift+F10);
- `shortcuts_dialog.gd`'s `UNLISTED` rows;
- `markdown_editor.gd`'s local Ctrl+E.

None collides. All four would go through `_bind_accelerator()`, so they are
rebindable.

## 4. Rail and context card

**Rail (A3).** STORY is drawn as a **section of the node list, not a fourth
domain**. Ruling L defines the node list as a shortcut layer in which every
target also exists in the dock:

- **Timeline** points to CIVIL ▸ Timeline (P).
- **Journeys** points to the planner plus Saved journeys (P).
- **Notes** opens the vault window (P).

No workspace or dock changes. The 40 px domain rail keeps its three marks.
Notes opens a window, not a dock category, because Ruling L item 4 says linked
notes stay per entity under CIVIL and are never a standalone category. CIVIL ▸
**Settlements** is drawn where Ruling L put it and marked **N, proposal**.

**Context card (A4).** No new section is added: §4.1 of `MAP_CONTEXT_SCOPE.md`
fixes the section order.

| Row | Section | Mark | Backing |
|---|---|---|---|
| Open in city viewer… | Object | B | renames `civ.open_city_layout` |
| Add event to timeline here ▸ (Battle · Siege · Front · Arrow) | Place here | P | `conflict_add` with the kind and the cursor year. **Gap:** non-conflict events have no store. One submenu level only (§4.2) |
| Plan journey from here | Go | P | `route_begin` + `open_journey_planner_with_route`. This is §4.3's "Journey from here", not yet written |
| Journey to here | Go | P | drawn disabled with its reason until an origin is pending |
| Open note for X | Info | B | renames `civ.vault_note` |

## 5. Places without notes (A5)

A new group in the vault panel, placed above *Missing & orphan notes*.

- **Readout:** "N of M places have no linked note".
- **Filter chips:** class (All, Capitals … Hamlets) and kind (Settlements,
  Provinces, Continents, Factions), plus *On a journey*
  (`civ_settlement_journey_passes`) and *In a conflict*
  (`conflicts_touching_settlement`).
- **Each row:** the place's name, class and faction, with *New from template…*
  (the existing VA-02 flow) and *Attach existing…*.

Four states are drawn:

- **No vault connected:** offers the connect button. The list would otherwise
  be the whole world.
- **All linked.**
- **No world.**
- **Generating:** dashed, not emptied, matching the 2026-10-12 bridge read gate.

**Cost to measure before building:** opening the group makes one
`vault_entity_summary` call per entity, and the largest world should be timed
before choosing between a live list and a cached one.

## 6. Tablet and phone

- **Tablet (A7).** The bar becomes ☰ · File · World · Story, with 44 px rows
  and 13–14 type. The tablet's existing `World` menu (`_world()`) keeps its
  pipeline rows on top; Data's rows join below them under RECORDS / TECHNICAL.
  That is a departure from DS-03's "match the canvas 100%", so it is question
  Q2. `_tabletparity_probe.gd`'s reach rule still holds, because Story adds
  titles on both form factors.
- **Phone (A8).** The MORE screen gains a STORY section (derived) with four rows:
  - Timeline (B);
  - Journeys (P);
  - Conflicts & campaigns (P);
  - Notes (P).

  The bottom nav stays `MAP · GENERATE · PLAN · MORE`. The long-press peek
  sheet's four chips are *Open note* (B), *Add event* (P), *Journey from here*
  (P) and *City viewer* (B), on 48 dp pills. This assumes `MAP_CONTEXT_SCOPE.md`
  fork F1 answer (a): the long-press drops the pin **and** opens the verbs, at
  480 ms.

## 7. Risks

1. **Eight menus.** `DCC_SHELL_SPEC.md` §2 says "Seven menus … program scope
   only". On 2026-08-24 the owner refused a top-level Vault menu as bar growth.
   Story is the same kind of addition. The alternative is to drop the Story menu
   and keep only the rail section and the card verbs (Q1).
2. **Name collision on tablet.** `World` already names a tablet menu (§6, Q2).
   On desktop, *World* also names the rail's WORLD domain, which is generation.
   A "World" menu holding factions and notes is a second meaning (Q3 offers
   *Records* instead).
3. **Rail churn and Ruling L.** Settlements needs a fifth CIVIL mode id. A STORY
   section adds three nodes that point outside their own section. Notes must
   not become a dock category (Ruling L item 4).
4. **Moving commands.** `MISTAKES.md` has three rules for this:
   - "Move a command off the menu bar": every moved row needs its
     `command_index.gd` `EXTRAS` and `shortcuts_dialog.gd` `UNLISTED` homes
     checked, and asserted by title.
   - "Delete a surface": inventory the Data menu item by item. §3.2 homes every
     row: 4 head rows, 15 route rows, Export map image… and 2 vault rows.
   - Ruling 4 (2026-09-05) already moved the planner off the menu bar, and it
     cannot be silently reversed. Story's *Journey planner* row is an
     additional entry point; the rail node stays.
5. **Probes that pin today's contents** (found by grep; they would need
   updating with the change):
   - **Data menu by title:** `_adaudit`, `_adpress`, `_factionhub`,
     `_phonemore_reach`, `_structmove`, `_v3menu`, `_vaultclick`,
     `_vfy_batch0905`, `_vfyb13`.
   - **Moved row titles:** `_exportdialog`, `_factionhub`, `_v3menu`,
     `_vaultclick`.
   - **`RAIL_NODES` or the node labels:** `_cartocensus`, `_civilcensus`,
     `_detent`, `_dockfit`, `_ds03fit`, `_ds03reflow`, `_ds03shot`,
     `_grpfloor`, `_idxfind`, `_jump`, `_ldwidth`, `_leftdock12`,
     `_numglass`, `_pcreach`, `_railfind`, `_railfold`, `_seedlayout`,
     `_structmove`, `_v3menu`, `_vfy_batch0905`, `_vfy_ld32`, `_vfyb13c`,
     `_worldcensus`.
   - **Card labels:** `_ctxbroker`, `_ctxcard`, `_ctxpicks`, `_pechronos`,
     `_pechronos_add`.
   - **General menu probes:** `_menuconf`, `_menudefect`, `_menupop`,
     `_menutab`, `_menutreedump`, `_menuwire`, `_vfy_r2_menus`.
   - **Tablet reach:** `_tabletparity`.

   All of these are `cartalith-native/godot-project/_<name>_probe.gd`.
6. **Session mode can hide too much.** If it hides controls that a running
   draft needs, such as Commit and Discard, the draft is stranded. The gate must
   enumerate every transition into the mode (the `MISTAKES.md` row on converting
   contexts).
7. **The Player map preset can leak.** Without a per-item DM-only mark, the
   preset only switches whole layers off. The row's tooltip says so; do not
   relabel it "safe for players".

## 8. Milestones

| Id | What | Size | Needs |
|---|---|---|---|
| SM-1 | Rename Data → World with RECORDS / TECHNICAL bands; Import ▸ and Sources & validation ▸ submenus; Places… and Timeline rows; probe updates | small | Q1–Q3 |
| SM-2 | Story menu over built calls only (the B rows); four accelerators via `_bind_accelerator` | small | Q1 |
| SM-3 | File ▸ Export ▸ submenu; Player / DM presets over `export_dialog.gd` | small | Q4 |
| SM-4 | Context card: two renames, *Add event to timeline* (conflict kinds), *Plan journey from here* / *to here* | small | — |
| SM-5 | Places without notes view in the vault panel, with the four states; time it on the largest world first | medium | — |
| SM-6 | Saved journeys and Conflicts list windows | medium | — |
| SM-7 | Session mode: the dock-content gate plus a seeded layout | medium | Q5 |

Later rows, not scheduled by this proposal: a per-item DM-only mark (Rust and
the save format); free timeline events (a new store); print and VTT writers; and
the CIVIL ▸ Settlements node (a fifth mode id, plan C1).

## 9. Owner questions, with recommended defaults

| # | Question | Recommended default | Why |
|---|---|---|---|
| Q1 | Add an eighth top-level menu, Story, against §2's "seven"? | **Yes**, with the rail section and the card verbs as well | Story tools are scattered across a rail node, a key, a window region, a dock category and three windows; one menu is the only single place a writer can find them |
| Q2 | On tablet, merge Data's rows into the existing tablet World menu, below its pipeline rows? | **Yes** | Keeps four top-level menus on tablet; nothing already in the tablet World menu moves |
| Q3 | Call the renamed Data menu *World* or *Records*? | **World**, as briefed | *Records* avoids the clash with the WORLD domain but hides the Technical band; the owner's word was World |
| Q4 | Move export from Data to File ▸ Export (DCC_SHELL_SPEC §2.1 keeps imports out of File)? | **Yes for export, imports stay** | Export is an act on the open project, the File menu's scope; leave a signpost row in World |
| Q5 | Should Session mode be a layout (switchable in Layouts) or a toggle? | **A toggle in Window that also seeds a layout** | A toggle is what a DM reaches for at the table; the layout makes it restorable |
| Q6 | Should STORY be a node-list section (as drawn) or a fourth domain? | **Section** | A domain swaps both docks and needs a workspace class; a section reuses CIVIL's |
| Q7 | Should *Add event* stay limited to the four conflict kinds until free events have a store? | **Yes** | Draws only what the engine stores; the tooltip names the gap |

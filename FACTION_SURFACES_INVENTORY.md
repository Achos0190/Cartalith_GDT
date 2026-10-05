# Faction surfaces inventory

Read-only audit, 2026-10-05. Scope: every menu, window, panel, category, row and rail item in
`cartalith-native/godot-project/shell/**` and `shell/workspaces/**` that touches a faction.
Each row was verified by opening the named file and symbol. Design documents were consulted only
to flag planned-but-unbuilt surfaces (marked NOT BUILT). This is a snapshot, not a status record;
`cartalith-native/docs/STATUS.md` owns progress.

Paths are relative to `cartalith-native/godot-project/shell/`. "R/E" = read-only or editable.

## 1. Surface table

### 1a. Navigation, menus, global

| Surface | Where (user path · file/symbol) | Faction data shown or edited | R/E | State |
|---|---|---|---|---|
| Rail node "Factions" | left rail · `dcc_shell.gd` `RAIL_NODES` (id `factions`); owns 10 CIVIL dock categories | opens Populate, Settlements, Factions, Territories, Relationships, Military, Culture, Religion, Economy, Timeline | nav | built |
| Search scope `f` | search box, prefix `f` · `dcc_shell.gd` scope `{prefix:"f", entity:"faction"}`; `place_search.gd` | faction name; result positioned on capital | R | built |
| Timeline layer "Politics" | timeline bar · `dcc_shell.gd` `TL_LAYERS` | toggle only; `TL_LAYER_NOTE` says layer toggles record intent, no layer renders yet | E (flag) | partial |
| Data ▸ Markdown vault… / Browse | `menus.gd` `_data` (`ID_VAULT`, `ID_VAULT_BROWSE`) → `vault_window.gd` | faction is an entity kind; notes attach to faction | E (notes) | built |
| Data ▸ World data tables… | `menus.gd` → `world_data_window.gd` | settlements table "Faction" col, provinces table "Faction" col; bare ids ("faction %d") | R | partial (ids, no names) |
| Data manager export/import | `data_manager_window.gd` export group `factions` (`civ_faction_count`) | one territory outline per faction holding cells; GeoJSON import creates unknown factions by name and paints territory | E (import) | built |
| New world dialog ▸ Factions | `new_world_dialog.gd` number row `civ.factions` | seed count for placement only | E (count) | built |
| Layers popover ▸ Political / Contested | `layers_popover.gd`; `viewport_host.gd` `refresh_faction_colors()` | territory fill in roster colours, contested cells | R | built |
| Map search/pick context card | `civilization_workspace.gd` `context_actions`: `civ.faction_open` "Open %s in roster…", `civ.faction_claim` "Claim for %s" (driven by `sample_cell` `controlling_faction`; absent on object hit) | controlling faction of clicked cell | open / E (claim) | built |

### 1b. CARTOGRAPHY workspace (`workspaces/cartography_workspace.gd`)

| Surface | Where | Faction data | R/E | State |
|---|---|---|---|---|
| Live layers `provinces`, `territory`, `conflict` | LIVE_LAYERS rows | visibility of faction-derived layers | E (toggle) | built |
| Political layers | `_build_political_layers`, section "Political layers" | two toggles over territory/province display | E (toggle) | built |
| Feature style ▸ Territories | `_build_feature_style`, section "Territories" | Fill opacity (`set_territory_opacity`), Reset; jumps to "Civilization ▸ Factions" (colours) and "Civilization ▸ Territories" | E (opacity), jump | built |
| Border line width / style, claim-hatching controls | same section, listed under "Not built" | n/a | n/a | NOT BUILT |

### 1c. CIVIL dock categories (`workspaces/civilization_workspace.gd`)

| Surface | Where | Faction data | R/E | State |
|---|---|---|---|---|
| Factions ▸ Roster | `_build_factions` | buttons: Faction roster…, Culture profiles…, Settlement types…; group "By province count" | R + open windows | built |
| Factions ▸ Identity colour | `_build_factions` | jump to Cartography Feature style | jump | built |
| Factions ▸ Linked notes | `_build_factions` `_knowledge_row("faction", …)` | per-faction vault notes, "N settlements · culture" | R + open | built |
| Factions ▸ emblem | `_build_factions` "Not built" (CV-21, GUI_GAP_REGISTER) | n/a | n/a | NOT BUILT |
| Populate | rail category | placement seeds by faction (faction picker via `_faction_choice`) | E | built |
| Settlements ▸ Totals ▸ By faction | `_build_settlements` | settlement count/population per faction | R | built |
| Settlements ▸ Roster | `_build_settlements` | settlement list; faction via place editor | R + open | built |
| Territories ▸ Recompute | `_build_territories` | Recalculate, Generate provinces, Clear territory (`civ_clear_territory`) | E | built |
| Territories ▸ Borders & influence | `_build_territories` (`civ_territory_influence`) | groups "By faction", "Contested borders" | R | built |
| Territories ▸ Linked notes | same | province / continent notes | R + open | built |
| Relationships ▸ Standing | `_build_relationships` / `_fill_relationships` (`civ_faction_relations`) | group "Every pair": stance, score, tooltip (border, culture, faith, trade, rivalry); rows open `show_faction(a, other)` | R | built |
| Relationships ▸ treaties, vassalage, diplomacy actions, change over time | "Not built" block (CV-26) | n/a | n/a | NOT BUILT |
| Military ▸ Conflicts | `_fill_military` → `_fill_conflicts` | conflict list, sides | R + open | built |
| Military ▸ Faction strength | `_fill_military` group "By military power" | per-faction power; rows open `show_faction(f)` | R | built |
| Military ▸ Manpower | `_fill_manpower` (`civ_military_summary_at`) | standing/field/emergency armies per faction | R | built |
| Military ▸ Garrisons / Fortifications / Campaigns | `_fill_garrisons`, "Strongest places", campaigns | garrison and fort strength by faction/place | R | built |
| Military ▸ unit movement/battles; per-year ag-tech/government | "Not built" | n/a | n/a | NOT BUILT |
| Culture ▸ Profiles | `_build_culture` | the 7 fixed culture profiles | R | built |
| Religion | `_build_religion` (`civ_belief_run`) | "N faiths" or "— not run"; ADHERENCE/DIFFUSION/DIVERGENCE segments; Run diffusion; groups "where each faith leads", "%d diverged", "%d factions" (By-faction block) | R (derived) | partial (manual run; not year-indexed, not saved) |
| Religion ▸ create / edit a custom religion | none found | n/a | n/a | NOT BUILT |
| Economy ▸ Trade balance / By-faction expander | `_build_economy` (`civ_faction_economy`) | per-faction food/resources/trade | R | built |
| Economy ▸ Trade flows | `infrastructure_workspace.gd` | tariff- and currency-aware prices | R | built |
| Timeline | `_build_timeline` (years, scrub, playback, filters, sim "Simulate collapse / recovery") | year cursor that Military/Relationships follow | E (cursor) | built |
| Timeline ▸ Politics gaps | `_build_politics_gaps` "Not built": vassalage, alliances, rivalries over time (CV-26) | n/a | n/a | NOT BUILT |

### 1d. Tools (CIVIL tool palette)

| Surface | Where | Faction data | R/E | State |
|---|---|---|---|---|
| Settlement tool | `_settlement_faction`, `_faction_choice` | faction a new settlement is founded for | E | built |
| Territory tool | `_territory_faction`; brush radius; commit / discard; Lasso territory (`civ_territory_paint_at`, `_polygon`, `_commit`, `_discard`) | claim grid, per-faction | E | built |
| Conflict tool | `_tool_options_conflict`; `_commit_conflict`, `refresh_conflicts`, `select_conflict`; kinds front/arrow/siege/battle | conflict drawing between factions | E | built |

### 1e. Windows

| Surface | Where | Faction data | R/E | State |
|---|---|---|---|---|
| Faction roster window | `faction_roster_window.gd` `FactionRosterWindow`; opened by Factions ▸ Roster, `app.gd` `open_faction_roster(faction)`, context card | list (banner, name); Overview; **Identity** (colour `civ_set_faction_color`; Government, Culture, **Religion**, Ag. technology via `civ_set_faction_field`); **Currency** (`_build_currency`); **Tariffs** (`_build_tariffs`); Territory fit (`_build_terrain_fit`); Military block; Settlements (N) sublist; Add faction; Remove last (confirm, floor of one) | E | built |
| Roster ▸ "Not built" | `_build_gaps` | Power breakdown's economic/political/cultural/religious axes, tax income, trade income, craft share | n/a | NOT BUILT |
| Culture profiles window | `culture_profiles_window.gd` | culture list, name-pool sample, per-faction culture picker (writes the roster's `culture`) | E | built (duplicate edit path) |
| Settlement types window | `settlement_types_window.gd`, `settlement_type_store.gd` `_build_faction_defaults` "Default type per faction", `set_faction_default`, `document()` / `restore_document()` | per-faction default settlement type | E | built |
| Place editor ▸ Overview | `place_editor_window.gd` "Polity" picker (`{"faction": id}`), "Re-roll name" (faction's culture) | settlement's owning faction | E | built |
| Place editor ▸ Political history | `_build_political`; stacked ownership bar, "Ownership periods" (`civ_settlement_ownership_periods`), "Population & tier trajectory" | ownership by year | R | built |
| Place editor ▸ timeline strip (SP-3) | lanes: ownership, population, events, journeys, conflicts | ownership lane | R | built |
| Place editor ▸ Manual political entries | NOT BUILT (no precedence rule decided) | n/a | n/a | NOT BUILT |
| Vault window | `vault_window.gd` `_entity()`, `_faction_name`; attach kinds settlement, province, continent, faction | faction notes; capital as position | E (notes) | built |
| City viewer | `city_viewer_window.gd` | culture profile only | R | partial (no faction edit; not a faction surface proper) |

### 1f. Right dock (`right_dock.gd`)

| Surface | Where | Faction data | R/E | State |
|---|---|---|---|---|
| Faction context | `CTX_FACTION`, `show_faction(faction_id, pair_with)`, `_build_faction` | id, Culture, Government, Ag. technology, Colour, Settlements, Territory (`civ_faction_territory_stats`), Provinces, State religion | R | built |
| Faction ▸ Relations | `_build_faction_relations` | "Relations · N", stance chip per pair, marked pair, BALANCE bar (same call as Relationships, filtered) | R | built |
| Territory context | `show_territory` / `_build_territory` | armed faction's Claimed cells, Area, Contested | R | built |
| Settlement context | `_settlement_faction_row`, "Politics" action → `show_faction`, standing-army share (MM-6/8) | owning faction | R + nav | built |
| Sample context "Control" row | sample context | which faction controls the cell (`CivData::territory`) | R | built |
| Conflict context | `_build_conflict` | Name, Start/End year, Ongoing, Outcome (authored), Attached to, "Sides" (toggle per faction), "Manpower by side" (`conflict_sides_manpower`), siege garrison, Delete | E | built |

### 1g. Counts

Built 44 rows, partial 4 rows (Timeline Politics layer, World data tables, Religion diffusion, City viewer),
NOT BUILT 9 (custom religion, emblem, Roster power axes/income block, diplomacy/treaties/vassalage,
Timeline politics gaps, manual political entries, border style/hatching controls, per-year ag-tech/government,
unit movement/battles). Counted by table row; rows that bundle several controls count once.

## 2. Faction data model touched by these surfaces

- **Roster** (`CivData::faction_roster`): name, culture, religion, government, ag_tech, optional colour override,
  currency (name, symbol, rate). Ids are 1-based. `get_factions()` returns id, name, culture, religion,
  government, ag_tech, color_r/g/b, settlement_count, population.
- **Claim grid** (`CivData::territory`): owning faction id per cell, 0 = unowned. Saved as `rasters/territory.i32`;
  roster as `entities/factions.json` (SAVEFILE_COMPAT.md).
- **Bridge (`engine_bridge.gd`)**
  - Roster and territory: `get_factions`, `civ_faction_count`, `civ_add_faction`, `civ_remove_faction`,
    `civ_set_faction_field`, `civ_set_faction_color`, `civ_clear_faction_color`, `civ_has_faction_colors`,
    `civ_faction_territory_stats`, `civ_faction_terrain_fits`, `civ_territory_paint_at`, `civ_territory_paint_polygon`,
    `civ_territory_commit`, `civ_territory_discard`, `civ_clear_territory`, `civ_territory_influence`,
    `territory_opacity` / `set_territory_opacity`.
  - Currency and tariffs: `civ_set_faction_currency`, `civ_faction_currency`, `civ_price_in_currency`,
    `civ_set_trade_tariff`, `civ_trade_tariff`.
  - Derived: `civ_faction_economy`, `civ_military_summary`, `civ_military_summary_at`, `civ_faction_relations`,
    `civ_belief_run`.
  - Vocabularies: `civ_religion_vocabulary` (fixed 8 keys, `CIV_RELIGIONS`), `civ_culture_vocabulary` (7 fixed
    profiles); government and ag-tech are fixed lists too.
  - Conflicts: `conflict_kinds`, `conflict_add`, `conflict_update`, `conflict_delete`, `conflict_list`,
    `conflict_campaigns`, `conflict_get`, `conflict_sides_manpower`.
  - History: `civ_settlement_ownership_periods`.
- **Stored vs derived**
  - Stored: roster fields above, claim grid, tariffs, currency, conflicts (with side lists), per-faction default
    settlement type (travels in `SettlementTypeStore.document()`), vault notes.
  - Derived on each call, stored nowhere: relations (culture +30, faith ±20, trade +25, border friction −55, banded
    to a stance), economy and military summaries, territory stats and influence.
  - Derived on demand and **not** year-indexed or saved: religion diffusion (`civ_belief_run`).
- **Engine consumers of roster fields**: culture feeds settlement naming, the relations culture term and Territory
  fit; government and ag_tech feed `manpower.rs`; religion feeds the relations faith term and the religious power
  axis. State religion is a hand-set categorical (`DCC_CONTROL_INDEX.md:527`, `RELIGION_DIFFUSION_SCOPE.md`).
- **Religion is not extensible**: no surface creates or edits a religion; the only religion write is the roster's
  Identity ▸ Religion picker over the fixed 8 keys. The expansion (archetypes, government forms, religious-policy
  axis, S1–S7/G1) is recorded in `OUTSTANDING_WORK.md` as researched, not built.

## 3. Overlaps and duplicates

| Data | Edited in | Also read/linked in | Note |
|---|---|---|---|
| Culture | Roster ▸ Identity; Culture profiles window per-faction picker | Faction dock, Factions ▸ Linked notes sub-line, Territory fit | two write paths to one field |
| Settlement's faction | Place editor ▸ Overview "Polity"; reassigning territory (Territory tool, Lasso, Recompute, "Claim for", GeoJSON import) | Settlements ▸ By faction, World data tables | five ways to change who owns land; Polity picker does not move the claim grid (verify before consolidating) |
| Territory claims | Territory tool, Lasso, Recompute, context-card "Claim for", Data manager import | Territories ▸ Borders & influence, dock Territory, Layers popover, CARTO Political layers | four entry points, three read views |
| Faction colour | Roster ▸ Identity only | Factions ▸ Identity colour, CARTO Feature style ▸ Territories only jump there | links, not duplicates, but three labels for one control |
| Tariffs, currency | Roster only | Economy ▸ Trade flows | edit and effect are in different workspaces |
| Conflict sides | right-dock Conflict context only | Military ▸ Conflicts / Campaigns, Place editor strip | no faction-side edit from the Faction view |
| Military numbers | none (derived) | Roster ▸ Military, Military ▸ Faction strength / Manpower, Settlement context army share | same call, four views |
| Relations | none (derived) | Relationships ▸ Every pair; dock Faction ▸ Relations | same call, filtered |
| Economy | none (derived) | Economy ▸ By faction, Roster ▸ Overview | two views |
| Per-faction notes | Vault window; Factions ▸ Linked notes | — | one store, two doors |
| State religion | Roster ▸ Identity ▸ Religion | dock "State religion", Religion ▸ "%d factions" | Religion category cannot edit |
| Faction identity summary | — | Roster inspector, dock Faction section, Factions ▸ By province count | three overlapping summaries |
| Per-faction default settlement type | Settlement types window only | applied by `apply_default_to_settlement` | lives away from the roster |

## 4. Proposal: one consolidated Faction view

**PROPOSAL ONLY. Nothing here is built, and none of it has been agreed with the owner.**

Make the Faction roster window the single hub (it already holds the most editable data), widened to tabs; keep the
right dock's Faction context as the compact read-only card, and keep the category pages as quick lists that open it.

Tabs:
1. **Identity**: name, colour, culture (absorbs the Culture profiles picker), government, ag. technology, religion
   (state religion; the future custom-religion list and the Religion diffusion readout for that faith belong here
   or in tab 6), emblem (NOT BUILT slot), default settlement type (absorbs the Settlement types per-faction row).
2. **Territory**: claimed cells, area, contested, terrain fit, provinces, influence by neighbour; actions Claim /
   Clear / Open territory tool.
3. **Settlements**: sublist, capital focus, population totals, polity reassign.
4. **Economy**: economy summary, currency, tariffs, trade flows summary.
5. **Military**: strength, manpower, garrisons, conflicts the faction is a side of (with side toggle).
6. **Relations & religion**: relation pairs, treaties/vassalage slot (NOT BUILT), belief diffusion by-faction block.
7. **History & notes**: ownership periods rolled up per faction (no faction-level view exists today), vault notes.

Menu entries that open it:
- Rail ▸ Factions (and the Factions category "Faction roster…" button).
- Data ▸ a new "Factions…" row beside Markdown vault and World data tables.
- Context card "Open %s in roster…", dock Faction "open" action, search result `f`, Military/Relationships rows
  (which today open the dock card; they could open the hub on the matching tab).
- Cartography ▸ Feature style ▸ Territories "Faction identity colours" jump (retargeted to the Identity tab).

Needs an owner decision before building: whether the Polity picker should also move the claim grid; whether Culture
profiles stays a separate library window (culture is a vocabulary, not a faction field); whether tariffs stay on the
Economy tab or on Relations.

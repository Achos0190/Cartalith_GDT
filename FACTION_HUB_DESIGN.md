# Faction hub — design

**Design only, 2026-10-05. Nothing here is built. No code was edited to produce it.**
Progress belongs in `cartalith-native/docs/STATUS.md` and the routed backlog row in
`OUTSTANDING_WORK.md`; this file defines the shape and the milestones, it does not track them.

Inputs: `FACTION_SURFACES_INVENTORY.md` (the listing this design answers) and the owner's three
answers of 2026-10-05, recorded in §1. Paths are relative to
`cartalith-native/godot-project/shell/` unless they start `cartalith-native/` or `crates/`.

What was opened to write this, so a reader can check it: `faction_roster_window.gd` (whole file),
`culture_profiles_window.gd` (header, `_build_phone_switcher`, `_rebuild_detail`, `_rebuild_roster`),
`settlement_types_window.gd` (header, `_build_faction_defaults`), `app.gd` (`open_faction_roster`,
`open_world_data`), `right_dock.gd` (`show_faction`, `CTX_FACTION`), `place_search.gd`
(`_add_factions`), `dcc_shell.gd` (`RAIL_NODES`, `SEARCH_SCOPES`), `workspaces/civilization_workspace.gd`
(`_fill_factions`, the `civ.faction_open` / `civ.faction_claim` rows, the roster signal hookup),
`place_editor_window.gd` (the Polity picker, `_apply`), `crates/cartalith-godot/src/lib.rs`
(`civ_edit_settlement`, `civ_territory_commit`, `civ_reprovince`, `civ_rebase_territory_paint`,
`civ_commit_territory_paint`, `civ_settle_staleness`, `get_factions`), `crates/cartalith-godot/src/civ_tools_bridge.rs`
(`CivTools`, `paint_at`, `commit`, `recompose`, `rebase`), `crates/cartalith-civ/src/lib.rs`
(`assign_territory`, `territory_sweep`, `civ_generate_provinces`), `crates/cartalith-civ/src/roster.rs`
(religion defaults). The religion-expansion research artifact linked from `OUTSTANDING_WORK.md` was **not**
opened; §6 relies on `OUTSTANDING_WORK.md`'s own summary of it.

---

## 1. Goal and the three owner decisions (binding)

**Goal.** One comprehensive Faction view: every fact about a faction, readable and (where it is editable
today) editable, in one window with tabs, reachable from every place a faction already appears.

**Owner decisions, 2026-10-05 — binding on this design:**

1. **The Place editor's Polity picker also moves the claim grid.** Changing a settlement's polity changes who
   owns the land, not only the settlement's label. The rule is §5.
2. **Culture is consolidated.** The separate *Culture profiles* window stops being a faction-editing surface;
   its content (profile detail, name-pool sample, per-faction picker) folds into the hub's Identity tab.
3. **Tariffs live on the Economy tab, with a mention on Relations.** Edit once, on Economy; Relations shows a
   read-only summary and a link back.

Two consequences of those answers that the owner did not state and that this design assumes (flagged in §9):
the *Settlement types* window stays as a **library** (a type is a library entry, not a faction field) but its
per-faction default column moves into the hub; and the hub is the existing `FactionRosterWindow` grown in
place, not a new window class (§3.0).

## 2. What exists today (short; the inventory has the table)

- **One editing window:** `faction_roster_window.gd` `FactionRosterWindow` — list pane (banner + name + counts,
  Add / Remove last), world-overview line, and an inspector holding Identity (colour, government, culture,
  religion, ag. technology), Currency, Tariffs, Territory fit, Overview, Military, Settlements sublist and a
  "Not built" note. 880 × 620, phone = master-then-detail with a 52 dp folded bar.
- **Duplicate culture write path:** `culture_profiles_window.gd` (three panes; phone uses a segmented
  `CULTURES / DETAIL / FACTIONS` switcher) writes the same `civ_set_faction_field("culture")`.
- **Per-faction default settlement type** lives only in `settlement_types_window.gd` (`_build_faction_defaults`),
  stored in `SettlementTypeStore`.
- **Read-only views of the same derived calls** are scattered: right-dock Faction card (`show_faction`),
  Military / Relationships / Economy / Religion / Territories dock categories.
- **Claim grid** is edited only by the Territory tool, Lasso, Recompute, the context card's "Claim for", and
  GeoJSON import. The Polity picker does not touch it (`civ_edit_settlement` sets `placement.faction` and
  `civ_dirty` only).
- **Not built anywhere:** custom religion, emblem, Power axes / tax / trade income / craft share, diplomacy
  actions, faction-level history.

Correction to the inventory: it lists a dock Faction card "open" action as an entry point. `right_dock.gd` has no
such action today; §4 adds it.

## 3. The hub

### 3.0 Shape decision: grow `FactionRosterWindow`, do not add a class

The hub is `FactionRosterWindow` with a tab strip. Reasons, all checked in the code:

- `app.faction_roster_window` is referenced by 31 files; `civilization_workspace.gd` connects its
  `roster_changed` and `tariff_changed` signals (lines ~407-408) and `_on_roster_changed` repaints the map.
- Seven probes find its nodes by name (`Currency_<key>`, `Tariff_<id>`) or class (`_tariff_probe`,
  `_currency_probe`, `_claimsabsent_probe`, `_pelayout_probe`, `_wingrip_probe`, `_peptraj_probe`,
  `_lanedfoodshed_probe`).
- A new `class_name` needs the `godot --headless --import` pass that strips `project.godot` comments
  (`CLAUDE.md` working rules). Not adding one avoids the hazard entirely.
- The window already carries the FR-02 `_rebuilding` focus guard, the RF-03 `_on_world_changed` reset and the
  phone master-detail machinery. A rewrite would re-earn each.

The window title becomes **"Factions"**. The method name `open_faction_roster` is kept and gains a parameter
(§4) so that no probe or caller has to be renamed.

### 3.1 Tab list

Seven tabs, as the inventory proposed, with these placement corrections that the code reading justifies:

| Tab | Id | Why it is here |
|---|---|---|
| Identity | `identity` | name, colour, government, ag. tech, culture (+ profile), religion, default settlement type |
| Territory | `territory` | claims, area, contested, terrain fit, provinces, influence. **Territory fit moves here from Identity's neighbourhood** — it is a verdict on land, and Identity's culture picker only *drives* it |
| Settlements | `settlements` | sublist, capital, totals |
| Economy | `economy` | economy summary, **currency, tariffs (edit)**, trade-flow summary |
| Military | `military` | power, manpower, fortifications, conflicts this faction is a side of |
| Relations | `relations` | pair standing, faith term, **tariff mention (read-only)**, absent-diplomacy note |
| History | `history` | ownership periods per faction, vault notes |

**Departure from the inventory:** it labelled tab 6 "Relations & religion". Religion is split instead. *State
religion* and the future custom-religion library sit in Identity (they are a faction's own attribute and will
need room to grow, §6); the *diffusion readout* (`civ_belief_run`, by-faction block) is a derived view of faiths
across settlements, so it sits at the foot of Relations under a "Faith" group, because the relations score has a
faith term and the two are read together. Tab count stays seven.

**Build lazily.** Only the active tab is built, on first show; the rest are built when first selected.
Today `open()` computes `civ_faction_terrain_fits()` and `civ_military_summary()` eagerly (both O(cells)); under
tabs each moves to its own tab's first show. This is the same lever as `DccWidgets.fill_when_visible`
(commit `ebdf6fdb`) and is the reason the hub should not cost more to open than the roster does now.

### 3.2 Desktop wireframe (window 960 × 640, min 680 × 440)

```
┌─ Factions ──────────────────────────────────────────────────────────── ✕ ┐
│ 5 factions · 412 300 settled pop · 61 settlements · 18 204 claimed cells │  ← _overview (kept)
│ Largest by population: Aurelia (141 000)                                 │
├──────────────────────┬───────────────────────────────────────────────────┤
│ ▣ Aurelia — 14, 141k │ ▣ [ Aurelia                        ]  (name field)│
│ ▣ Veldmark — 9, 98k  │ ┌IDENTITY┬LAND┬PLACES┬ECONOMY┬MILITARY┬RELATIONS┬HISTORY┐
│ ▣ Korrath  — 7, 61k  │ │                                               │
│ ▣ Mirelle  — 6, 40k  │ │ § IDENTITY                                    │
│ ▣ Dunmere  — 5, 33k  │ │  Colour        [■ ▾]  [Reset]                 │
│                      │ │  Government    [monarchy ▾]                   │
│ [+ Add] [− Remove]   │ │  Ag. technology[traditionalAgrarian ▾]        │
│                      │ │  Culture       [highland ▾]                   │
│  (250 px, scrolls)   │ │   ┌ Highland profile ───────────────────────┐ │
│                      │ │   │ Terrain theme: hills · fit shown on LAND │ │
│                      │ │   │ Names drawn: [Dunmar] [Caerlin] [+3] 🎲  │ │
│                      │ │   │ Also used by: ▣ Korrath                  │ │
│                      │ │   └──────────────────────────────────────────┘ │
│                      │ │  Religion      [sky_pantheon ▾]               │
│                      │ │  Default settlement type  [Walled town ▾]     │
│                      │ │ § NOTES  (faction vault notes → HISTORY)      │
│                      │ └───────────────────────────────────────────────┘
│                      │                                         [ OK ]   │
└──────────────────────┴───────────────────────────────────────────────────┘
```

Tab labels on desktop are the short upper-case mono forms shown (`LAND`, `PLACES`, `ARMS` if width is short),
the full word in the tooltip. Style follows `DccTheme.header()`'s `§` sigil and the `culture_profiles_window.gd`
segmented-button look (accent-wash fill on the active tab).

### 3.3 Tablet wireframe

Same two-pane structure and tab strip; `DccTheme.is_tablet()` swaps `role_px("row_min_h")` / `fs_prose`
metrics through `DccWidgets` as every window already does. List pane narrows to 220; tab strip buttons take the
44 px `btn_min_h`; the strip is one row of seven at ≥ 800 px content width, otherwise it wraps to two rows
(`HFlowContainer`, not a scroller — §5.5 allows one scroller per sheet).

### 3.4 Phone wireframe (393 × 852 dp reference)

Phone keeps the window's present master-then-detail and adds one control. Seven segments cannot fit at 393 dp
(56 dp each is narrower than the 44 px hit cell plus a label), so the tab strip becomes a **section chooser**:

```
 list screen                  detail screen
┌──────────────────────┐    ┌──────────────────────────┐
│ FACTIONS             │    │ ▣ Aurelia   1 of 5     › │  ← 52 dp folded bar (kept)
│ factions · identity… │    ├──────────────────────────┤
├──────────────────────┤    │ SECTION  [ Identity   ▾ ]│  ← DccWidgets.choice, 44 px
│ ▣ Aurelia — 14, 141k │    ├──────────────────────────┤
│ ▣ Veldmark — 9, 98k  │    │  (one scroller)          │
│ …                    │    │  § IDENTITY              │
│ [+ Add] [− Remove]   │    │  Colour …                │
└──────────────────────┘    └──────────────────────────┘
```

`DccWidgets.choice` is an existing, probed popup picker (`_pickerpopup_probe`). A horizontally scrolling chip
strip was rejected: §5.5 gives a sheet exactly one scroller and nested scroll regions make touch drags
ambiguous. A faction chosen by id lands on the detail screen directly, exactly as `open(select_faction)` does
now (`_set_phone_list_open(not landed)`).

### 3.5 Per tab: every control and readout

"Source" is the bridge call or engine symbol. R = read-only, E = editable. "Absorbs" names the surface that goes
away or becomes a link.

**Identity**

| Control / readout | Source | R/E | Absorbs |
|---|---|---|---|
| Name field | `civ_set_faction_field(id,"name")` | E | roster inspector head (kept) |
| Colour picker + Reset | `civ_set_faction_color` / `civ_clear_faction_color` | E | `Factions ▸ Identity colour` note (becomes a one-line link here); Cartography "Faction identity colours" jump retargets here |
| Government | `civ_government_vocabulary`, `civ_set_faction_field("government")` | E | right-dock Faction card's read of it stays |
| Ag. technology (+ hint) | `civ_ag_tech_vocabulary`, `civ_set_faction_field("ag_tech")` | E | — |
| Culture picker | `civ_culture_vocabulary`, `civ_set_faction_field("culture")` | E | **Culture profiles window's per-faction picker** |
| Culture profile card (below the picker) | `get_cultures()` row for the selected key: terrain theme (`terrain_affinity`), `faction_count` | R | **Culture profiles window's detail pane** |
| Name-pool chips with reroll | `bridge.settlements()` filtered to factions on that culture; `civ_reroll_settlement_name(idx)` | E (reroll) | **Culture profiles window's "Name pool — real settlements"** (its comment explains why it is real settlements, not a synthetic roll: `civ_settle_name` is not `#[func]`-exposed) |
| "Also used by" list | `get_factions()` filtered by culture | R | **Culture profiles window's "Assigned factions"** |
| Religion picker | `civ_religion_vocabulary`, `civ_set_faction_field("religion")` | E | — (custom religions add to this picker, §6) |
| Default settlement type | `SettlementTypeStore.faction_default(id)` / `set_faction_default` | E | **Settlement types window's "Default type per faction" column** |
| Emblem | none | — | the `Factions` category's "Not built" emblem slot; stays an honest disabled slot |

**Territory**

| Control / readout | Source | R/E | Absorbs |
|---|---|---|---|
| Claimed cells · km² · contested | `civ_faction_territory_stats(id)` | R | roster Overview line; right-dock Territory section |
| Territory fit verdict + mix | `civ_faction_terrain_fits()` / `_fit_for` | R | roster "Territory fit" |
| Provinces (name, capital, cells) | `bridge.provinces()` filtered by `faction` | R | `Factions ▸ By province count` row (becomes a link to this tab) |
| Influence by neighbour | `civ_territory_influence()` filtered to this faction | R | `Territories ▸ Borders & influence ▸ By faction` (that list stays; this is its per-faction view) |
| "Claim cells for this faction" | the same handler as context-card `civ.faction_claim` (`civilization_workspace.gd`, arms the Territory tool) | E (arm) | — |
| "Focus on capital" | `viewport.move_view_to` (existing) | nav | roster Overview button |
| Stale-claims chip | `civ_dirty` + `civ_settle_staleness` state | R | tooltip text on the Polity picker (§5) |

There is deliberately **no** "Clear this faction's claims" button: `civ_clear_territory` clears every faction and
there is no per-faction clear in the engine. Inventing one is out of scope.

**Settlements**

| Control / readout | Source | R/E | Absorbs |
|---|---|---|---|
| Totals (count, settled pop, capital) | `get_factions()`, `_capital_of` | R | roster Overview |
| Sublist, sorted by population | `bridge.settlements()` filtered by `faction` | R + open | roster "Settlements (N)" group; row opens the Place editor (existing) |
| "Transfer…" per row (later, FH-9) | the §5 rule | E | none — new convenience over the same call as the Polity picker |

**Economy**

| Control / readout | Source | R/E | Absorbs |
|---|---|---|---|
| Per-faction economy (food, surplus, exports, imports, strategic resources) | `civ_faction_economy()` row | R | `Economy ▸ By faction` expander (kept; this is its detail view) |
| Currency name / symbol / rate (+ example line) | `civ_faction_currency`, `civ_set_faction_currency`, `civ_price_in_currency` | E | roster "Currency" section, moved verbatim |
| **Tariffs** — one row per exporter | `civ_trade_tariff`, `civ_set_trade_tariff`; `tariff_changed` signal | **E** | roster "Tariffs" section, moved verbatim |
| "Trade flows →" | `select_domain_category("civilization","Economy")` | nav | — |

Node names `Currency_<key>` and `Tariff_<id>` and the `tariff_changed` / `roster_changed` signals are preserved
exactly (§8).

**Military**

| Control / readout | Source | R/E | Absorbs |
|---|---|---|---|
| Power /100, fortification counts | `civ_military_summary()` row | R | roster "Military" |
| Standing / field / levy, era verdict, citizen share | same, `manpower` dict | R | roster "Military" |
| Garrison and fortification strength of this faction's places | `civ_military_summary_at` / `_fill_garrisons` inputs | R | `Military ▸ Garrisons` (kept; per-faction cut) |
| Conflicts this faction is a side of | `conflict_list()` filtered by sides | R + open | `Military ▸ Conflicts`; each row opens the right-dock Conflict context where sides are edited. **Side editing is not duplicated here.** |
| "Full breakdown →" | existing | nav | — |

**Relations**

| Control / readout | Source | R/E | Absorbs |
|---|---|---|---|
| Pairs involving this faction: stance chip, score, term tooltip | `civ_faction_relations()` filtered | R | dock Faction ▸ Relations; `Relationships ▸ Every pair` (kept; this is its per-faction view) |
| **Tariff mention (decision 3)**: "Charges X% on goods from B · B charges Y% on yours · edit on Economy" per pair | `civ_trade_tariff(a,b)` and `(b,a)` | **R** + link | none — new; the edit stays on Economy |
| Faith group: faiths in this faction's settlements, diffusion readout | `civ_belief_run` by-faction block | R | `Religion ▸ "%d factions"` block |
| "Diplomacy: not built" | — | — | CV-26's "Not built" notes, consolidated into one |

**History**

| Control / readout | Source | R/E | Absorbs |
|---|---|---|---|
| Ownership periods rolled up across this faction's settlements | `civ_settlement_ownership_periods` per settlement (aggregate in script first; a `#[func]` only if measured slow, FH-8) | R | none exists today at faction level |
| Linked vault notes | `_knowledge_row("faction", …)` in `civilization_workspace.gd`; `open_vault("faction", id, name)` | R + open | `Factions ▸ Linked notes` |
| Manual political entries | none | — | stays NOT BUILT (no precedence rule decided; inventory §1e) |

### 3.6 What is deliberately left out of the hub

The Territory, Settlement and Conflict **tools** stay in the tool palette: they are map interactions, not
faction fields. The Place editor stays the place for a settlement's own data. The Vault stays a window. The
right dock's Faction card stays as the compact read-only card (it gains an "Open in Factions…" action, §4).

## 4. Entry points and navigation

**Signature.** `FactionRosterWindow.open(select_faction: int = -1, tab: String = "")` and
`DccApp.open_faction_roster(faction: int = -1, tab: String = "")` (`app.gd`, currently `open_faction_roster(faction: int = -1)`).
`tab` is one of `identity territory settlements economy military relations history`; an unknown or empty value
means "the last tab used this session, else `identity`". A faction id not in the roster is ignored exactly as
`open()` does today ("ignored, not clamped"). This is the same shape as `open_world_data(tab: String = "")`
(`app.gd`), so no new convention is introduced. Existing zero- and one-argument callers (about a dozen probes
and the dock buttons) keep working unchanged.

| Entry point | Today | After |
|---|---|---|
| Rail ▸ Factions | selects the CIVIL dock's Factions category (`RAIL_NODES`) | unchanged; the category's first row becomes the **"Open factions…"** button → `open_faction_roster()` |
| Factions category "Faction roster…" button | `app.open_faction_roster()` | same call, relabelled; the *Culture profiles…* and *Settlement types…* buttons are removed from this section (Settlement types survives as a library entry under the new Data row below) |
| Data menu | Markdown vault, World data tables (`menus.gd` `_data`) | **new row "Factions…"** → `open_faction_roster()`; no new `ID` collides: it is added next to `ID_VAULT` |
| Context card "Open %s in roster…" (`civ.faction_open`) | `open_faction_roster(cf_id)` | `open_faction_roster(cf_id, "territory")` — the user clicked land, so land is the relevant tab; row label becomes "Open %s…" |
| Context card "Claim for %s" (`civ.faction_claim`) | arms the Territory tool | unchanged |
| Right-dock Faction card | no open action | **new action "Open in Factions…"** → `(faction_id, "identity")`; when the card was opened with `pair_with` it opens `"relations"` |
| Right-dock Settlement ▸ "Politics" | `show_faction(faction)` (dock card) | unchanged (dock card, then the action above) |
| Search scope `f` | pans the map to the capital (`place_search.gd` row carries `x`,`y`,`entity`,`id`) | unchanged in FH-1; a faction row's secondary activation (Shift+Enter / long-press) opening `(id,"identity")` is an FH-9 option. A faction with no settlements has no row today (documented decline in `place_search.gd`) and the hub is how to reach it |
| Military ▸ Faction strength rows | `show_faction(f)` | `open_faction_roster(f, "military")` |
| Relationships ▸ Every pair rows | `show_faction(a, other)` | **FH-6 re-routed them** to `open_faction_roster(a, "relations", other)`: the Relations tab lists this faction's pairs and marks `other` (FH-1 had left them on `show_faction(a, other)` while the tab was a placeholder) |
| Culture category "Which faction has which culture →" | `app.open_culture_profiles()` | `open_faction_roster(-1, "identity")` |
| Cartography ▸ Feature style ▸ Territories "Faction identity colours" | `select_domain_category("civilization","Factions")` | `open_faction_roster(-1, "identity")` |
| Roster window's own "Full breakdown → Civilization ▸ Military" | closes window, selects category | unchanged |

On a phone, every row above that passes a faction id lands on the detail screen (the pick is the navigation), as
`open()` already does for the context-card row.

## 5. The Polity-picker rule (owner decision 1)

### 5.1 What the code does today (verified)

- `civ_edit_settlement` (`crates/cartalith-godot/src/lib.rs`) is all-or-nothing: it validates `faction` with
  `faction_roster.is_assignable`, sets `placement.faction`, sets `civ_dirty = true`, and touches **nothing
  else** — not `CivData::territory`, not `provinces`, not `CivTools`. It records no ledger entry, so it has no
  undo.
- `CivData::territory` is `CivTools::territory_base` (from `assign_territory`) merged with
  `CivTools::territory_paint` (u8 values, `0` = fall through to the base). `commit` → `recompose`; `rebase`
  re-anchors the base on a Recompute and **re-merges the paint, so hand paint survives Recompute**.
- `assign_territory` / `territory_sweep` project territory from **capitals only** (`placement.capital`): a
  cost-distance Voronoi where effective distance = cost ÷ `territory_weight(pop)`. Non-capitals sit in whichever
  capital's zone they fall in.
- `civ_territory_commit` = `civ_commit_territory_paint` (`tools.commit` + `civ_reprovince`) and the ledger
  records "Territory commit" as `EntryKind::Recorded`: the pre-commit grid is not retained, so a committed
  territory change is **not undoable**.

So today the picker relabels a settlement and nothing else; the borders move only on the next Recompute, and
only if the settlement is a capital.

### 5.2 Recommended rule — "move the settlement's land through the hand-paint layer"

**Changing a settlement's polity from A to B repaints, as hand paint committed in one step, the connected cells
A owns that belong to that settlement — its whole province if it is a province seed (Metropolis, Capital or City,
per `civ_generate_provinces`), otherwise only its own cell — then re-runs `civ_reprovince`.**
**Going through `CivTools::territory_paint` means the change survives Recompute with no new state and no new
saved format; the settlement's `faction` is set in the same call, and `civ_dirty` stays true because economy and
routes also derive from faction.**

Detail the implementation must settle (each is a test in FH-5):

1. **New engine surface, two `#[func]`s.** `civ_polity_reassign_preview(index, new_faction) -> Dictionary`
   (cell count, km², `from`, `to`, `loses_capital`, `loses_last_cell`, `ok`/`reason`) and
   `civ_polity_reassign(index, new_faction) -> Dictionary`. Validation stays `is_assignable`. Both live beside
   `civ_edit_settlement` and share one body so the preview cannot disagree with the apply.
2. **Cells.** A flood fill (4-neighbour) from the settlement's cell over cells with `territory == A` and, for a
   province seed, `provinces == its province id`. A non-seed settlement takes only its own cell and the status
   line says so ("one cell moved — it is an enclave of B"). **The "own cell only" choice for small places is a
   labelled judgement, not a measurement** (§9 Q1).
3. **Paint value.** `paint_value` is u8: a faction id above 255 cannot be painted. The call refuses with a named
   reason; the roster is nowhere near that today.
4. **No accumulating paint.** Where the target faction already owns the cell in the *base*, write the fall-through
   value (`0`) instead of an explicit paint, so moving a settlement back does not leave a stack of redundant
   paint. FH-5 must verify this against `CivTools::paint_at`'s subtract semantics before relying on it
   (`civ_rebase_territory_paint`'s own comment records a past bug where a subtract restored paint instead of the
   computed owner).
5. **A capital moving.** If the settlement has `placement.capital` and A has no other capital, A's base territory
   disappears at the next Recompute (only capitals project). The preview reports `loses_capital` and the picker
   asks for confirmation naming the consequence; it does not refuse. B gains the capital's zone on the next
   Recompute whether or not paint exists, so the paint and the recompute agree.
6. **A loses its last cell.** Allowed, reported in the preview. A faction with settlements and no land is a state
   the roster already permits (a freshly added faction owns nothing).
7. **Confirmation.** The picker calls the preview, then shows `DccWidgets.confirm` with "N cells (X km²) will move
   from A to B" for any move of more than the settlement's own cell; a one-cell move applies directly.
8. **Undo.** None in v1, **consistent with the Territory commit it reuses** (`Recorded`, pre-commit grid not
   retained). The confirm dialog states "not undoable; use the Territory tool to repaint". An undo entry needs a
   sparse (cell → old value) snapshot in the ledger and is FH-5's stated follow-up, not part of it.
9. **Saved data.** `rasters/territory.i32` changes content, not schema. Nothing new is written to
   `entities/factions.json`. An old build opening a new save sees an ordinary painted grid.
10. **UI text.** The Place editor's Polity tooltip, which today says changing it "does not repaint the borders by
    itself; Recalculate territories re-derives them", is rewritten to describe the new rule in the same change
    (Ruling BK: a change fixes the prose it makes stale).

Costs, stated plainly: the claim grid now has a second writer that is not the Territory tool; a polity change
becomes a hard-to-reverse edit; a province seed's whole province moving is a large visible change from a small
control; and the picker now needs a world with a whole claim grid (a project reopened without
`rasters/territory.i32` has none; there the picker falls back to alternative B and says why).

### 5.3 Alternative B — "label now, borders on Recompute, say so"

Keep `civ_edit_settlement` as it is; add a **stale-claims chip** (Territory tab, Settlements tab, Place editor)
that appears when a settlement's faction differs from the owner of its own cell, and a one-click "Recalculate
territories" beside it. Smallest change, no new Rust, nothing irreversible, and it fixes the real defect (the
user cannot see that borders disagree). Its cost is that the borders *do not* move for a non-capital even after
Recompute (non-capitals sit in a capital's zone), so the chip would be permanent for them — which is exactly why
the owner asked for the grid to move. **Recommended only as the fallback inside §5.2**, not as the rule.

## 6. Custom religion (later phase; the picker exists now)

**State today.** Religion is a fixed list of eight keys (`CIV_RELIGIONS`, `civ_religion_vocabulary`,
`roster.rs` "categorical state religion"). A faction's `religion` is one of them. Consumers read the *key*: the
relations faith term, `belief::compat` / `CIV_RELIGION_DOMAIN` (a bijection from themed cultures), and the
religious power axis via `faction_has_religion`. No surface creates or edits a religion.
`OUTSTANDING_WORK.md` records the owner's rejection of a small list extension and the published research
(intra-religion pantheon mechanism, inter-religion coexistence, seven new archetypes, six-plus-two government
forms, a religious-policy axis, stages S1-S7/G1, eight owner decisions, S3 needing its own ruling because it
changes generated output).

**Where it lands in the hub.** Identity tab, under the Religion picker: a **Religions library** group — list,
"New religion…", edit, delete-if-unused — and the faction's picker lists built-ins first, custom ones after.
The Faith group on Relations shows the diffusion readout for custom religions too, once the engine can compute
one.

**Phasing — separable versus blocked:**

| Phase | Content | Engine change | Depends on the researched expansion? |
|---|---|---|---|
| **R0 (in FH-0..FH-3)** | Picker over the eight fixed keys, in Identity | none | no |
| **R1 — named variants** | A custom religion is a record `{id, name, base_key, colour, notes}` whose engine behaviour **is its `base_key`'s** (the eight built-ins). The shell shows the custom name; the engine keeps reading the base key. Library UI, create/rename/delete, per-faction pick | `CivData::religions: Vec<ReligionDef>`, saved as a new `entities/religions.json` (additive: an old reader ignores it); `FactionEntry.religion` may hold `custom:<id>`, resolved to `base_key` at the engine boundary so `belief.rs`, `roster.rs` and the power axis see only built-in keys | **No.** Honest about its limit: the UI must say a custom religion currently behaves as its base |
| **R2 — real attributes** | Custom religions carry axes (exclusivity, locus, institutional form per the research) that drive coexistence, retention and the siting rule | `belief.rs` dynamics, `SettlementReligionState` use, civ→urban faith wiring (S3) | **Yes — hard dependency** on S1-S7 and the S3 ruling; changes generated output, so it needs a ruling and a golden/pin re-baseline plan |

R1 is the only part that can ship without owner rulings on the expansion, and it is the part the hub's layout
has to reserve space for. R2 is not designed here.

**R1 shipped 2026-10-05** (see `STATUS.md`). Deviations from the table: the store is a `ReligionLibrary { next_id, defs }` rather than a bare `Vec<ReligionDef>` (ids are never reused); `get_factions().religion` is now the engine key, with `religion_choice`/`religion_name`/`religion_missing` added for custom choices; the library lives inline on Identity, no window; the slot is `SAVEFILE_COMPAT.md` §9.9.

## 7. Milestones

Each is small and independently shippable; the app builds and every probe listed in §8 passes after each. Model
tiers follow the standing "cheapest reliable model per task" rule; they are suggestions. Every milestone
carries Ruling BK annotation, a "Protects:" line on each new test, and the `MISTAKES.md` preflight in its brief.
Max two agents per batch.

**Migration order — no surface is ever homeless.** A source surface is removed only in the milestone *after*
its destination is live and probed.

| # | Milestone | Scope | Files | Probe | Rust / saved data | Tier |
|---|---|---|---|---|---|---|
| **FH-0** | Tabbed shell | Tab strip (desktop/tablet) and phone section chooser inside `FactionRosterWindow`; `open(select_faction, tab)`; existing sections redistributed into their tabs **unchanged** (Identity, Economy = currency + tariffs, Territory = fit + overview, Settlements = sublist, Military = military block); Relations and History are placeholders; lazy per-tab build; `_rebuilding` guard held across tab switches | `faction_roster_window.gd` | new `_factionhub_probe` (every tab builds non-empty; switching tabs with a focused currency field commits to the right faction — FR-02; phone chooser reaches all 7) | none / none | Sonnet 5 max |
| **FH-1** | Entry points | `open_faction_roster(faction, tab)`; context-card row → Territory; Data ▸ Factions…; dock Faction "Open in Factions…"; Military and Relationships rows; Cartography jump; retitle "Factions" | `app.gd`, `menus.gd`, `right_dock.gd`, `civilization_workspace.gd`, `cartography_workspace.gd` | extend `_ctxcard`, `_ctxphone`, `_factionhub` | none / none | Sonnet 5 max |
| **FH-2** | Culture fold (decision 2) | Culture profile card, name-pool chips with reroll and "also used by" under the Identity culture picker; **retire** `culture_profiles_window.gd`, its `app` field/opener, and the Factions category button; Culture category link → `(−1,"identity")`. The 7-profile list stays in the Culture dock category (read-only list of a vocabulary) | `faction_roster_window.gd`, `app.gd`, `civilization_workspace.gd`, delete `culture_profiles_window.gd` | **port** `_cultureprofiles_probe`'s assertions into `_factionhub` before deleting it | none / none | Sonnet 5 max |
| **FH-3** | Default settlement type | "Default settlement type" choice in Identity; `settlement_types_window.gd` **kept** as the library but its per-faction column becomes a link to Factions ▸ Identity | `faction_roster_window.gd`, `settlement_types_window.gd` | adjust `_settlementtypes_probe`; `SettlementTypeStore.document()` round-trip unchanged | none / none (store unchanged) | Sonnet 5 |
| **FH-4** | Territory tab | claims, area, contested, fit, provinces, influence by neighbour, "Claim cells" (reuses the `civ.faction_claim` handler), stale-claims chip | `faction_roster_window.gd`, `civilization_workspace.gd` (shared handler) | `_factionhub` Territory assertions; `_claimsabsent` still reads "—" with no grid | none / none | Sonnet 5 max |
| **FH-5** | Polity rule | §5.2: `civ_polity_reassign_preview` / `civ_polity_reassign`; Place editor Polity picker calls them with confirm; tooltip rewritten | `crates/cartalith-godot/src/lib.rs` (+ a unit-testable free function like `civ_commit_territory_paint`), `engine_bridge.gd`, `place_editor_window.gd` | Rust tests: province move, one-cell enclave, paint survives `civ_rebuild(Replace)`, capital warning, u8 refusal, no-accumulating-paint, no-grid fallback; new `_polityreassign_probe` in the shell | **Rust yes; saved data: `territory.i32` content only, no schema change** | Opus (touches generated/saved data) — verifier Opus |
| **FH-6** | Economy and Relations content | Economy: `civ_faction_economy` row, trade summary + link; Relations: pair list, **read-only tariff mention**, Faith group (`civ_belief_run`) | `faction_roster_window.gd` | extend `_factionhub`; `_tariff_probe` and `_currency_probe` unchanged | none / none | Sonnet 5 |
| **FH-7** | Military tab completion | garrison/fortification cut and "conflicts this faction is a side of" with open-to-dock rows | `faction_roster_window.gd` | extend `_factionhub`; `_mm8garrison` baseline recorded first (4 pre-existing literal failures) | none / none | Sonnet 5 |
| **FH-8** | History tab | ownership periods rolled up per faction; vault notes row | `faction_roster_window.gd` | extend `_factionhub` | none in script; a `#[func] civ_faction_history` only if the per-settlement loop measures slow | Sonnet 5 |
| **FH-9** | Tidy | Factions category collapses to one "Open factions…" button plus the By-province list; dock Faction card trimmed to the compact summary; per-row "Transfer…" in Settlements; optional search-result secondary action; World data tables show faction names not ids (the inventory's "partial") | `civilization_workspace.gd`, `right_dock.gd`, `world_data_window.gd`, `place_search.gd` | `_menuconf`, `_rdconform`, `_railfold` | none / none | Sonnet 5 |
| **FH-R1** | Custom religion, named variants | §6 R1: library UI + `civ_*_religion` funcs + `entities/religions.json` | `faction_roster_window.gd`, `crates/cartalith-civ`, `cartalith-godot`, `project_bridge.rs`, `SAVEFILE_COMPAT.md` | Rust round-trip + save/load; `_factionhub` | **Rust yes; new saved file (additive)** | Opus |
| **FH-R2** | Custom religion, real attributes | blocked on the researched expansion S1-S7 and the S3 ruling | — | — | changes generated output | not scheduled |

Windows after the migration: **retired** — Culture profiles. **Kept, narrowed** — Settlement types (library only).
**Kept, unchanged** — Place editor, Vault, City viewer, Data manager, World data tables. **Grown** — Faction roster
becomes "Factions". **Kept as thin entry points** — the right-dock Faction card and the CIVIL dock's category pages.

FH-0 to FH-4 and FH-6 to FH-8 have **no Rust and no saved-data change**; they are safe to run before the owner
has weighed in on anything in §9 except Q4.

## 8. Risks and what must not regress

Existing probes that read this surface, to be run (windowed, scratch project copy, distinct `config/name`, under
`timeout`; they print `PROBE-FAIL` but exit 0) before and after every milestone: `_tariff_probe`,
`_currency_probe`, `_claimsabsent_probe`, `_pelayout_probe`, `_wingrip_probe`, `_peptraj_probe`,
`_lanedfoodshed_probe` (all find `FactionRosterWindow` nodes — **node names `Currency_<key>` and `Tariff_<id>`
must not change**); `_winstale_probe`, `_winsweep_probe`, `_pickerpopup_probe`, `_phonesweep_probe`,
`_widthfloor_probe`, `_uxreview_probe`, `_vfy_filldiff_probe`, `_entwinphone_probe`, `_exitbuttons_probe`,
`_gap37_probe` (window sweeps, phone fit, popup pickers, exit buttons); `_ctxcard_probe`, `_ctxphone_probe`
(the context-card rows and their phone shape); `_cultureprofiles_probe` and `_settlementtypes_probe` (FH-2/FH-3
retire or adjust them deliberately); `_lazyfill_probe` (the `fill_when_visible` contract); `_railfold_probe`,
`_menuconf_probe`, `_rdconform_probe` (rail, menu and right-dock conformance).
Pre-existing failures to record as the baseline, not to chase: `_mm8garrison_probe` (4 literal failures at HEAD)
and `_ctxpicks_probe` (2, routed in `OUTSTANDING_WORK.md`).

Hazards, each already paid for once:

- **FR-02 (focus-exit commit into the wrong faction).** A tab switch tears down a pane that may hold a focused
  currency or tariff field. Every teardown goes through `_clear()` and its `_rebuilding` flag, and
  `_commit_focused_field()` runs before `_selected` or the tab changes.
- **RF-03 (stale roster after a generate).** `_on_world_changed` must rebuild the *active* tab and invalidate the
  lazily built others; `_selected` still resets to 1.
- **`AcceptDialog` sizing.** The window is sized once at popup from its content minimum; the existing
  `child_controls_changed.call_deferred()` and the single-scroller phone shape are load-bearing. A tab strip must
  not add a second `SIZE_EXPAND_FILL` pane (the roster measured 2 619 px of content in an 852 px phone window the
  last time that happened).
- **Phone: one scroller per screen, 44 px (`PHONE_TAP_MIN`) hit cells,** and no horizontally scrolling strip
  (§3.4). Tablet: `role_px` metrics, not literals.
- **O(grid) work on open.** `civ_faction_terrain_fits()` and `civ_military_summary()` are O(cells). Build per
  tab on first show; never in `open()`. A 4096-wide world already blocks ~5.8 s on generate; the hub must not
  add a second block.
- **No new `class_name`.** `project.godot` import-pass hazard (`CLAUDE.md`). This design adds none.
- **Signals.** `roster_changed` (a rate changes no flow) and `tariff_changed` (a tariff changes matched volume)
  stay distinct, and `civilization_workspace.gd`'s two connections (`_on_roster_changed`, `_on_tariff_changed`)
  keep working. A culture write still emits nothing extra beyond today's behaviour.
- **Dead controls.** `DESIGN_HANDOFF.md` §9 rule 1: a control with nothing behind it is a defect. Emblem,
  diplomacy and Power axes stay as honest "Not built" text; the Relations tab must not draw a treaty row.
- **Retiring before replacing.** Do not delete `culture_profiles_window.gd` until FH-2's ported assertions pass;
  its probe protects behaviour (reroll draws from the faction's stored culture) that is easy to lose.
- **Generated output.** FH-0..FH-4 and FH-6..FH-9 change no generated output. FH-5 changes the saved claim grid
  for a world a user edits (not a generation result); FH-R2 would change generated output and needs its own
  ruling.
- **Docs.** `STATUS.md` and `OUTSTANDING_WORK.md` updated in the same change as each milestone; the inventory's
  entry-point list is a snapshot and is not edited.

## 9. Open owner questions (each with a default)

Only what really needs the owner.

1. **How much land does a small settlement take with it?** Default: a province seed (Metropolis/Capital/City)
   takes its whole province; a Town/Village/Hamlet takes **its own cell only** (an enclave). Alternative: a
   fixed hinterland radius, which needs a number nobody has measured.
2. **Moving a capital: warn or refuse?** Default: **warn and confirm**, naming that the old faction will lose its
   territory at the next Recompute if it has no other capital.
3. **Should a polity move be undoable?** Default: **no in v1**, matching the Territory commit the move reuses;
   the dialog says so. Undo is a follow-up that needs a sparse snapshot in the ledger.
4. **Does the Settlement types library stay a separate window?** Default: **yes** — it is a library of types, and
   only its per-faction column moves. (Decision 2 covered Culture, not this.)
5. **Custom religion: ship named variants (R1) before the researched expansion, or wait for it?** Default:
   **R1 first**, labelled in the UI as behaving as its base religion until R2, because it is the only part that
   needs no engine rulings.

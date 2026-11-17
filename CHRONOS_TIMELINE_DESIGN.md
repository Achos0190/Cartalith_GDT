# CHRONOS_TIMELINE_DESIGN.md

**A design for Ruling GD (owner, 2026-11-17), written before any build.** It defines; it does not track (`STATUS.md` answers how far it has got). Everything below was read at its symbol on `f69b3e6b`; where a claim comes from a document and not from code, it says so. No product code was written and no test was run.

The owner's words (`LARGE_ITEM_RULINGS.md`, Ruling GD): *"I want that to be the timeline data per item consolidated on the cartalith timeline. If any timelines are found in the vault the program should recognise it and ask what to do with it."* And the alternative the same day: *"Or make it the primary cartalith timeline."*

**Where it sits.** It replaces VE-7's unspecified "drawn chronos track" in `VAULT_EDITOR_PROPOSAL.md` (s.3.4: *"A note's history and the map's timeline stay one thing, as Ruling AM intended"*). It builds on `TIMELINE_SCOPE.md` (the snapshot simulation, unchanged) and `STORY_PLANNING_SCOPE.md` (SP-2 journeys, SP-3 chronos events, SP-4 conflicts, whose entities it only *reads*).

---

## 1. Inventory: every kind of dated data today

Precision legend: **Y** = integer year; **D** = year plus month/day in the 365-day no-leap calendar (`cartalith_vault::chronos::MONTH_DAYS`); **R** = range (start, optional end; `None` = open); **derived** = computed from other data, never stored.

| # | Data | Lives at (symbol) | Precision | Save slot |
|---|---|---|---|---|
| 1 | **The year cursor** | `CivData::year` (engine); the shell's one writer `tl_set_year()` in `dcc_shell.gd`; signal `timeline_changed` | Y | `history/timeline.json` member `year` |
| 2 | **The cursor's day** | `WorldGen::civ_day`, `tl_day()` / `tl_set_day()` | D (0-364) | **not saved** (no member in `timeline.json`; `SAVEFILE_COMPAT.md` 10.1 lists `year` only) |
| 3 | **Recorded snapshots** | `cartalith_civ::timeline::TimelineSnapshot` (`year`, `territory`, `settlements`, `ways`, `collapse_flags`, `record`) | Y (a point per recorded year) | `history/timeline.json` `years[]`; claims raster `history/territory/*` (10.2) |
| 4 | **Per-year records** | `YearRecord` (`institutions` per faction: `ag_tech`, `government`; `place_overrides` per `tid`: walls, age, traits, specialisation) | Y | inside `years[]` (`institutions`, `place_overrides`) |
| 5 | **Settlement founding year** | `PlaceExtras::founded: Option<i64>` (`civ_roster_bridge.rs`); stamped by `generation_founding_year` when `civ.founding_year` is on (default off in `WorldParams::defaults`) | Y, `None` = unknown | `settlements[].extras.founded` (`entities/settlements.json`, 9.1) |
| 6 | **Settlement age** | `PlaceExtras::age: Option<u32>`, `YearPlaceOverride.age` | a duration, not a date | `extras.age`; `years[].place_overrides[].age` |
| 7 | **Ownership spans** | `OwnershipSpan {start_year, end_year: Option, faction_id}` from `civ_settlement_ownership_periods` over every snapshot | R, **derived** | none (recomputed) |
| 8 | **Population trajectory** | `PopulationPoint {year, pop, kind, flags}` | Y, **derived** | none |
| 9 | **Collapse / ruins flags** | `CollapseFlags {fortified, ruins}` per `tid` per snapshot | Y | `years[]` (collapse flags member, 10.1) |
| 10 | **Faction span** | no authored faction dates exist. A faction is "present" when any snapshot has a settlement it owns | R, **derived** from 3/7 | none |
| 11 | **Conflicts** | `cartalith_civ::conflict::Conflict {start_year, end_year: Option, sides, outcome, kind, anchor}`; read by the strip via `bridge.conflict_list()` / `tl_conflict_spans()`; `Conflict::active_in(year)` | Y, R | `entities/conflicts.json` (9.7), engine-owned |
| 12 | **Journeys** | `travel_library::Journey {start_year, start_day: Option<u16>, route, plan}`; length = `JourneyTimeline.calendar_days` (derived from the plan) | D start; the end is **derived** (`start + calendar_days`) | `entities/journeys.json` (9.6) |
| 13 | **Campaign reading** | `CampaignReading {territory_year, baseline_year}` | Y, **derived** | none |
| 14 | **Landmark-derived marks** | `BattleMark`, `FormerSettlement.last_year`, `FormerWay` (landmark generation reads the above) | Y, **derived** | none |
| 15 | **Simulation inputs** | `timeline_bridge.rs` `start_year`, `step_years`, `end_year` (the collapse/recovery run) | Y | not saved (run parameters); its *output* is item 3 |
| 16 | **Note chronos events** | `cartalith_vault::chronos::parse` -> `Event {kind: Event/Period/Point/Marker, start, end, start_md, end_md, color, group, name, description}`; `VaultSession::entity_chronos(kind, id)` reads every note linked to the entity, **live from disk, else the link's saved copy**; written back only by `add_chronos_event` (hash-guarded, appends to the note's last block) | Y, D, R; negative years valid | **the note itself** (vault); the link is in `vault.json` `links[]` |
| 17 | **Other note dates** | frontmatter keys, dated lists and tables: **not read by anything today** (`frontmatter.rs` reads properties generically; `template.rs` has `{{date}}` which is the *real-world* date) | - | the note |
| 18 | **Timeline view state** | `TL_LAYERS` (six toggles, "they record which layer you want; no layer renders yet"), `_tl_filter_exist_only` / `_ghost` / `_highlight` in `CivilizationWorkspace` | - | layers persist per device (`_tl_save_layers`); the three filters do not |

What the strip already draws (`tl_range`, `tl_conflict_spans`, `tl_selection_events` in `dcc_shell.gd`; `_fill_timeline_strip` / `_build_timeline_expanded` in `app.gd`): the recorded years, conflict spans, and **the selected settlement's or faction's note events** as ticks plus a list. So one partial consolidation exists (items 3, 11, 16), selection-bound and with no kind or faction filter; settlement founding years (5), journeys (12) and ownership (7) are not on it. `tl_range()` widens its track from recorded years and conflict years only.

**Three facts the design must respect, found in the inventory.**
1. **There is no per-item timeline store.** "Per-item timeline data" is a *join* over items 3-16, not a table. Consolidation is therefore a read model, not a migration.
2. **Precision is mixed.** The strip is year-grained (`tl_year`), the day cursor is not saved, and journeys and chronos events carry months/days. The consolidated model keeps `(year, Option<MonthDay>)` per end, exactly as `chronos::Event` does (Ruling AO: the finer grain is *this* date, not a second date system).
3. **Year 0 is the generated world** and `founded` is in `-1000..=-60` (`generation_founding_year`); a note's years are the author's own calendar. Whether a note's year 1204 means the engine's year 1204 is an **assumption the user must confirm per timeline** (s.4, the "epoch offset" on import; owner question Q5).

---

## 2. The consolidated timeline

### 2.1 One read model, `TimelineItem`

A pure function in a new `cartalith-civ` module (`timeline_items.rs`), stateless like the rest of that crate, building one list from what exists:

```
TimelineItem {
  key:        stable string  ("founded:settlement:42", "conflict:7", "journey:3",
                              "span:faction:2:120", "note:<link_id>:<line_key>")
  source:     Founding | Snapshot | Ownership | Conflict | Journey | NoteEvent | Imported
  start:      (i64, Option<MonthDay>)
  end:        Option<(i64, Option<MonthDay>)>      // None = point or still open
  open_end:   bool                                  // distinguishes "open" from "a point"
  title:      String
  entity:     Option<(EntityKind, i64)>             // the map item it belongs to
  factions:   Vec<i32>                              // owner(s) / sides
  precision:  Year | Date | Range
  origin:     { vault_id, relative_path, line_key } // NoteEvent / Imported only
  private:    bool                                  // from the note (s.4.5)
}
```

`open_end` is a separate flag on purpose (`MISTAKES.md`: never encode "no value" as a plausible value): a point and an open range must not both read `end: None`.

Sources and what each emits:

| Source | Emits | Entity | Factions |
|---|---|---|---|
| Founding (5) | one point per settlement with `founded: Some` (none for `None`; the filter shows a dash with the reason, not a year 0) | the settlement | its owner now |
| Snapshot (3/4/9) | a tick per recorded year; collapse/ruins/walls changes as points on the item they touch | the `tid` | the faction in that snapshot |
| Ownership (7) | one range per `OwnershipSpan` | the settlement | `faction_id` |
| Conflict (11) | a range (`end_year` open = `open_end`) | its anchor, if any | `sides` |
| Journey (12) | a range, `start + calendar_days` | none (route) | none |
| NoteEvent (16) | one per parsed line, **live** | via the note's `KnowledgeLink` | from the linked entity's owner |
| Imported (s.4) | one per imported line, from the map's own copy | the entity chosen at import | as NoteEvent |

Cost note: items 7 and 8 are derived from every snapshot per call; `civ_settlement_ownership_periods` carries its own cost note and the model asks for one settlement's or one faction's spans at a time, never the whole world at once. A world-wide ownership lane is opt-in (kind filter), built lazily, and measured in CT-2.

### 2.2 Filters

Three independent filters, all multi-select, combined with AND across groups and OR inside one:

- **Item**: a map item (settlement, faction, conflict, journey) or "selection follows" (today's `tl_selection_events` behaviour, kept as the default so nothing the strip does today disappears).
- **Kind**: Founding, Snapshot, Ownership, Conflict, Journey, Note events, Imported. (The six `TL_LAYERS` pills stay: they are *layer* intent, "Climate/Population/..." and say so; the Kind filter is a different axis. They are not merged here.)
- **Faction**: by faction id; matches `factions` of an item.

Rows beyond the filter are not drawn and are not counted; the strip says "N hidden by filters" so an empty strip is never read as "no data".

Filter state persists per device beside `_tl_save_layers`, not in the world save (the three `CivilizationWorkspace` filters today are not saved at all).

### 2.3 What drawing it moves

Only `app.gd`'s `_build_timeline_expanded` rows and the CIVIL > Timeline body read the model; the model is queued on the existing `timeline_records_changed` signal (PC-3) and rebuilt deferred and coalesced as `_queue_tl_rows` already does. **No new year writer and no new signal.** A click on any row calls `tl_set_year()` (and `tl_set_day()` when the row has a day), as VE-7's chronos row already does through `_on_chronos_year`. `tl_range()` additionally widens to include `Founding` and visible `Imported` years so a founding at year -340 is reachable; today's track would clamp it out.

---

## 3. Option (a) against option (b)

**(a) A consolidated view on today's timeline, beside the year strip.** The year strip, the snapshot simulation and the saves are unchanged; the read model of s.2 feeds extra rows (and a full "Items" list in CIVIL > Timeline). Chronos syntax stays the note's format.

**(b) The chronos timeline *as* Cartalith's primary timeline,** possibly with Chronos syntax as storage or interchange.

What each moves:

| Concern | (a) view beside the strip | (b) chronos as primary |
|---|---|---|
| **Year strip** | Unchanged cursor; gains rows from the read model | Its meaning changes: the cursor is a *time on a drawn track of items*, not an index into recorded snapshots. `tl_set_year`, `tl_range`, `tl_year_neighbours`, the phone sim strip, `timeline_changed` listeners (journey markers, conflicts, Place editor, right dock, `_pc2year_probe`) all assume the engine's `CivData::year` is the one value. They would have to be re-based on a new owner of time |
| **Timeline simulation** (`TIMELINE_SCOPE.md`) | Untouched | Its product is snapshots (`TimelineSnapshot`: whole settlement and way lists per year), which Chronos syntax **cannot express**: a Chronos line is `[date] name | description`, with no population, no ways, no claim raster, no `tid`. Snapshots would remain as a second store the primary timeline merely points at, so "primary" would be a skin |
| **Story** (`STORY_PLANNING_SCOPE.md`) | Conflicts and journeys read as they do; SP-3's note events stay in notes | Conflicts (anchor geometry, sides, outcome) and journeys (route, plan, `start_day`) cannot be stored as Chronos lines either; they would stay engine entities and be *mirrored* into chronos text, with two writers per fact |
| **Saves** (`SAVEFILE_COMPAT.md`) | No change to 10.1, 9.6, 9.7, 9.1. One new optional slot for imports (s.5) and one optional `vault.json` member (s.4.4); old files read byte-identically, as the append-only rules require | 10.1's `years[]` gains a chronos representation or is replaced; every reader of the format (a second implementation is the point of the compat file) must accept it; the engine-owned slots (`ENGINE_OWNED_SLOTS`, `cartalith-io/src/project.rs`) change owner. A format migration to carry a view |
| **Vault notes** | Untouched until the user asks; an import copies, a link reads | Making Chronos syntax the storage means Cartalith writes lines into notes, or keeps text blocks outside notes that Obsidian cannot see. Notes are hand-edited free text (the module doc of `chronos.rs`): a machine-owned block replaced wholesale is exactly what `add_chronos_event` was written *not* to do (`write_block` vs the free-text body) |
| **Undo / round-trips** | Imports are one grouped undo step (same grouping as the assistant's actions, Ruling DW); a note is never written, so no cross-store undo | Every timeline edit is a text edit of a note or a block: undo must span the note, the engine and the hash guard (`SourceChanged`). Round trip: `to_line` "refuses any value that would not read back identically", so every unrepresentable field (population, `tid`, faction sides, route) is a lossy edge |
| **Chronos' strength it would borrow** | Reads it live (as `entity_chronos` does) and shows it; keeps Obsidian compatibility for free | Interchange with the plugin; groups, colours, `ORDERBY`/`DEFAULTVIEW` flags (view, not data) |
| **Cost / risk** | CT-0..CT-9 below, mostly one read model and rows; no golden or save moves | Rewrites the cursor's owner, the saves and three crates' contracts at once; a re-baseline of 10.1 needs an owner ruling |

**Recommendation: (a), with two pieces of (b) taken as opt-in edges.**

Reasons, each at a symbol:
1. The year cursor is `CivData::year`, written by one function and read by a dozen listeners (`timeline_changed`'s own doc says so); (b) would move time's owner for a display benefit.
2. The simulation's output is `TimelineSnapshot`, whose content has no Chronos form. A Chronos-primary timeline would carry the snapshots as a hidden second store.
3. Ruling AM's reason for Chronos syntax was *interchange with the note*, and it already delivers that: the note stays the source for its own events and `entity_chronos` reads it live. (a) completes that into the strip; it does not need to own the syntax.
4. The owner's wish is *"timeline data per item consolidated on the cartalith timeline"*, which (a) delivers literally.

The two edges taken from (b), both optional and later (CT-8): **(i) Export** a filtered consolidated view as a Chronos block the user pastes into a note (a generated, read-only snapshot with a header comment `# generated by Cartalith; edit freely, it is not read back`); **(ii) Chronos as the import format** for vault timelines (that is s.4 anyway). Neither makes Chronos the store of anything Cartalith owns. If the owner still wants (b), Q1 below names what it costs; the design for (a) leaves it possible, because the read model's `TimelineItem` is the seam either could sit behind.

---

## 4. Recognising vault timelines

### 4.1 What counts as a timeline (and why each form)

A **recognised timeline** is a set of at least one dated event found in one note, carrying a *form*, a *location in the note* and a *line key*. Forms, in order of confidence:

| Form | Rule | Why | Confidence |
|---|---|---|---|
| **F1 Chronos block** | a fenced ` ```chronos ` block; `chronos::parse`; lines in `skipped` are listed, not dropped | the owner's own syntax (Ruling AM); exact reuse, zero new grammar | certain |
| **F2 Frontmatter dates, closed key list** | only the keys `founded`, `founding_year`, `established`, `born`, `died`, `start_year`, `end_year`, `year` with an integer (negative allowed) or `YYYY[-MM[-DD]]` | a settlement's `founded` is a first-class engine field (`PlaceExtras::founded`) so this is the one frontmatter form that has a *destination*. **`date`, `created`, `modified`, `updated` are excluded on purpose**: they are real-world note metadata (`template.rs`'s `{{date}}` is today's real date) and would flood the strip with 2024 | high for the key list |
| **F3 Year-led list under a timeline heading** | a bullet list whose items start with a year (`- 1204: ...`, `- **1204** - ...`, `- 1204-1210 ...`), under a heading named *Timeline, History, Chronology, Events, Annals, Dates* (case-insensitive), with >= 2 items; or, under any heading, >= 4 items whose years are strictly ascending | the commonest hand-written form in a worldbuilding vault; the heading or the ascending run is what separates it from `- 3 cats` | medium |
| **F4 Dated table** | a Markdown table whose header has a `Year` or `Date` column and >= 3 rows each parsing as a year in that column | the second most common form; the header column name is the guard | medium |
| **F5 The Obsidian *Timelines* plugin's HTML spans** (`ob-timelines`, `data-date`) | **recalled, not read from its source here; verify before building** | a different plugin's data; deferred (CT-9, only if the owner's vault uses it) | unverified |

Not recognised, deliberately: bare years in prose, `date:` frontmatter, daily-note filenames, and any line the reader cannot place with certainty. A false positive costs a prompt; a silent one costs trust. Everything recognised is labelled with its form, so the user sees *why* it was found.

### 4.2 When detection runs

- **On vault open / bind:** a background scan over the vault's Markdown, **read-budgeted** like `private::private_marks` (it takes `max_reads` and "a later call with the leftovers continues"): same cadence, same "not looked at is not decided" rule. It never blocks the UI and never runs while the busy overlay is up.
- **On a note's save** (Cartalith's editor or an external change picked up by the provider's modified time and `provider::content_hash`): that note only.
- **On link/unlink** of a note to a map item: that note only.
- **Never on a keystroke**, and never for a note the user has told us to ignore (s.4.4).
- The result is a **candidate list**, not a prompt storm: one passive badge ("3 timelines found in the vault") in the vault side panel and the strip's Items filter; the prompt (s.4.3) opens when the user clicks it, or once at first detection per vault, whichever the user chooses (Q3).

### 4.3 The prompt, per timeline

One row per timeline (note + form + block ordinal): the note's name, the form, the count of events, the year range, and a 3-line preview. The four choices:

| Choice | What it does | Writes where |
|---|---|---|
| **Link to a map item** | binds the *note* to a settlement / faction / province (the existing `KnowledgeLink` flow); the note's events then show on the strip for that item **live**, through `entity_chronos`, exactly as today | `vault.json` `links[]` (existing) |
| **Import onto the timeline** | copies the events to the map as `Imported` items, bound to an entity (asks which) and with an **epoch offset** (default 0, shown) | a new optional slot, s.5 |
| **Keep as the note's own** | the timeline stays in the note, shown only in the note's Preview (VE-7) and not on the strip | the decision record only |
| **Ignore** | never asked again for this block | the decision record only |

**Defaults, and why.**
- A note **already linked to an entity** (any `KnowledgeLink`): default **Link**, i.e. *no action*: the strip already shows it live (`tl_selection_events`); the prompt only confirms. Nothing is written.
- A note **not linked**: default **Keep as the note's own**. It is the one choice that changes nothing and loses nothing; the user can choose Import or Link any time from the same list. A default that wrote the map or the note on a click-through would be the regression this ruling is worded against ("ask what to do").
- **Never a bulk default of Import.** A "Apply to all" exists only for **Keep** and **Ignore**.
- F2's `founded` offers a fifth row-specific action, **"Set the settlement's founding year to Y"**, for a linked settlement whose `founded` is `None` or differs; it goes through the existing `set_founded` (one undo step), shows the current value, and is never applied silently.

### 4.4 How the choice is remembered

A new optional `timelines` member in the project's `vault.json` (13.3; a writer MAY omit it; an unknown member is preserved, per 13.3.5's tolerance), keyed by what the *project* knows, not the machine:

```
"timelines": [
  { "vault_id": "...", "relative_path": "Kelv.md", "form": "chronos", "ordinal": 0,
    "block_hash": "<hash of the block text>",
    "decision": "keep" | "ignore" | "link" | "import",
    "entity_kind": "settlement", "entity_id": 42,          // link / import
    "epoch_offset": 0,                                      // import
    "decided": <unix seconds> }
]
```

- The key is `(vault_id, relative_path, form, ordinal)`; `block_hash` detects *change*. A changed block of a `keep`/`link` timeline re-surfaces as **"changed since you decided"** (a badge, no prompt); an `ignore` stays ignored **until the block's hash changes**, then it is a candidate again with the previous decision shown.
- It travels with the map file (the same logical vault lands on the same `vault_id`), and survives a re-bind of the vault on another machine. A relative path that no longer resolves leaves the row dangling and listed, never deleted (13.3.5's "missing" rule).
- No per-machine state is needed. Detection results themselves (the candidate list) are recomputed and never saved.

### 4.5 Conflicts and duplicates

- **Duplicate against Cartalith's own data.** Dedup key for an incoming event: `(entity, kind, normalised name, start, end)`. An `Imported` item whose key equals an existing one (same settlement, same year, same name) is flagged **"already on the timeline"** and offered as *skip*, not added twice. A note event on a linked entity whose year equals that entity's `founded`, with a name containing "found", is shown as **"agrees with the founding year"**; a different year is shown as a **conflict row** ("note says 1204, founding year says 1210"), with both visible and a button to set one from the other. Nothing is auto-resolved: picking a winner is a guess (`entity_data`'s own doc makes the same refusal for two notes disagreeing about population).
- **Duplicate across notes.** The same event in two linked notes is shown once with both origins (path list), the way `entity_data` carries `relative_path` on every row rather than merging.
- **Re-import after the note changed.** Line identity is the hash of `(kind, start, end, name)`; a re-sync shows **added / removed / changed** counts and lets the user take each; it never rewrites silently. An imported event whose source line vanished is kept and marked "source removed".
- **The reverse direction (map -> note).** The existing `add_chronos_event` (hash-guarded, with an explicit event dialog) is the only writer into a note, and is **unchanged**. **An import never edits the note under any choice.** The note is read only (`VaultSession::read`); the one note write in this whole design stays the one the user already invokes by name.
- **Private notes** (`cartalith_vault::private::is_private` / `private_marker`, Ruling DP, the one rule both LLM features share): detection **runs** (the user's own app, the user's own notes) but a private note's rows carry the private badge (`PrivateMarker::label`, which is a fixed vocabulary and never leaks note text), the **default is always Keep**, **Import and Link show a one-line warning** ("this note is private; its events will be saved in the project file"), and an imported event inherits `private: true`: it is **hidden from the assistant's tools and from the retrieval index** (the same function the LLM features already call) and **omitted from any export** until the user says otherwise. Whether `private` events should also be left out of a saved `.zip` is Q6, because the save's `vault.json` already stores `imported_text` for linked notes (13.3.3), so there is precedent either way. An unreadable note is "not decided", never "public" (the `private_marks` rule).

---

## 5. Saves and compat (what changes, exactly)

Additive only; every old file reads byte-identically, per `SAVEFILE_COMPAT.md`'s own rule for new members.

| Slot | Change | Notes |
|---|---|---|
| `history/timeline.json` (10.1) | **none** | the simulation and snapshots are untouched |
| `entities/conflicts.json`, `journeys.json`, `settlements.json` | **none** | read by the model, never written by it |
| `vault.json` (13.3) | optional `timelines[]` (s.4.4) | MAY; absent = no decisions made |
| **new** `history/events.json` (a `MAY` slot, engine-owned, in `cartalith-io::DOCUMENT_SLOTS`; section **10.3** of the compat file) | the `Imported` events: `{ id, entity_kind, entity_id, kind, start, start_md?, end?, end_md?, color?, group?, title, description?, origin {vault_id, relative_path, line_key}, epoch_offset, private? }` | ids issued by the writer and never reissued (the 9.x rule); a malformed row costs that row and is reported (the 10.1 tolerance); the `Imported` list **never** overrides a snapshot |
| device settings | filters (s.2.2), "ask once on first detection" | not in the world save |

Not stored: candidate detection, the read model, `TimelineItem`s from live sources. `SAVEFILE_COMPAT.md` gets section 10.3 and the `vault.json` member in the same change that builds them (CT-4, CT-5), not in this one.

---

## 6. Forms

Sizes: desktop uses the strip's expanded rows (`_build_timeline_expanded`); tablet the same in the bottom dock; phone the phone sim strip and a sheet, per `MOBILE_SHELL_PROPOSAL.md`. Touch targets 44 px.

**Desktop: the expanded strip with consolidated rows.**
```
+--------------------------------------------------------------------------------+
| |<  <  >  >|  x1   1204-03-14   [ Items v ]  [ Kind v ]  [ Faction v ]  3 hidden |
| -400 ----- -200 ----- 0 ----- 200 ----- 400 ----- 600 ----- 800 ----- 1000      |
| Founding   *Kelv(-340)    *Orsa(-210)         *Dun(-90)                        |
| Ownership  [====== Red League ======][== Duchy ==][= Red League =]  Kelv       |
| Conflict   [Siege of Kelv 1198-1204]        [War of Two Rivers 640- ...>       |
| Journey    [Envoy to Orsa 1203-03-02..1203-05-11]                              |
| Note       o 1204 coronation (Kelv.md)   [Imported] o 1190 flood (Orsa.md)     |
| ! 2 timelines found in the vault - Review...                                   |
+--------------------------------------------------------------------------------+
```
**Desktop prompt (a window, not blocking the map, PC_SHELL's context-window rule):**
```
+- Timelines found in your vault ---------------------------- [x] -+
| Kelv.md   Chronos block   6 events   -340 to 1204   [linked]      |
|   o Link (already: shown live)  o Import  (*) Keep  o Ignore      |
| Dun.md    Year list under "History"   4 events  310-388           |
|   o Link to map item...  o Import...  (*) Keep  o Ignore          |
| Plot.md   [private] Chronos block 3 events                        |
|   ! private note: default Keep. Import stores it in the project.  |
| [ Apply Keep to all ]  [ Ignore all ]            [ Done ]         |
+-------------------------------------------------------------------+
```

**Tablet: the same rows in the bottom dock, filters as a segmented bar; prompt as a right-edge sheet (520 dp docking rule).**
```
+--------------------------------------------------+
| |< < > >|  x1   1204-03-14  [Items][Kind][Fac]   |
| Founding  *Kelv  *Orsa            *Dun           |
| Conflict  [Siege of Kelv][War of ...>            |
| Note      o 1204 coronation                      |
| ! 2 timelines found - Review                     |
+--------------------------------------------------+
```

**Phone: the existing collapsed strip is untouched; "Items" is a bottom sheet from the strip's row, rows stacked, one tap moves the year.**
```
+-------------------------+     +-----------------------------+
| 1204-03-14  |< < > >|   |     | Timeline items        [Filter]|
| [ Items (14) ]  ! 2     |     | Kelv founded        -340      |
+-------------------------+     | Siege of Kelv   1198-1204     |
                                | Envoy to Orsa   1203-03..05   |
 prompt = full-height sheet:    | 1204 coronation (Kelv.md)     |
 one card per timeline,         | [ Review 2 vault timelines ]  |
 four 44 px buttons each        +-----------------------------+
```

---

## 7. Milestones

CT-0 is read-only and gates the rest. Bars are literal and each carries a mutant (`MISTAKES.md`: assert a literal; mutate the constant). Builds are ordered by the owner's GUI-first priority; none starts until Ruling GD's questions below are answered. Every Godot-visible step is probed windowed on desktop, tablet and phone with frames looked at.

| ID | What | Bars |
|---|---|---|
| **CT-0** | Corpus pass: run the four forms over the owner's own vault (read-only, no network) and a synthetic set; count candidates per form and false positives | F1 precision 1.00 on blocks (it is the parser); F2 key list false positives 0 on the owner's vault; F3/F4 precision >= 0.90 on a hand-labelled sample **before** they ship; report recalled Timelines-plugin use (F5) |
| **CT-1** | Read model `TimelineItem` in `cartalith-civ` (s.2.1) from existing sources only | one test per source with literal rows; `founded: None` emits **no** item; open end vs point distinguished; no engine output, golden or save changes (`git diff` of `history/*` slots empty) |
| **CT-2** | Strip rows + CIVIL > Timeline "Items" list read the model; Item/Kind/Faction filters; "N hidden" | desktop/tablet/phone probes: row counts equal the model's; every row click lands `tl_year()` on the row's year and `timeline_changed` fires exactly once; ownership lane lazy, **median of 5 with min..max** under a stated budget on a 2 048 world; selection-follows reproduces today's `tl_selection_events` ticks exactly |
| **CT-3** | Detection: F1+F2 in Rust (`cartalith_vault::timelines`), read-budgeted scan, on-save re-scan | scan of a 5 000-note vault yields to the UI (no frame over the budget, measured); a note not looked at is "undecided" never "no timeline"; F2 never reads `date`/`created`; private notes detected with the private marker |
| **CT-4** | The prompt, choices **Link / Keep / Ignore** + the decision record in `vault.json` `timelines[]`; compat section written | no note byte changes under any choice (hash of every note before/after identical); decisions survive save/load/re-bind; changed block resurfaces; old save loads identically |
| **CT-5** | **Import**: `history/events.json`, epoch offset, entity binding, dedup and conflict rows, grouped undo; compat 10.3 | import then undo = byte-identical project; duplicate import adds 0 rows; a mismatch with `founded` renders both; round-trip save/load equal; malformed row costs one row |
| **CT-6** | F3 (year list) and F4 (table) recognition, shipped only if CT-0 met its precision bar | the CT-0 sample's precision/recall as literals; a mutant that loosens the heading rule fails |
| **CT-7** | Founding action from F2 (`set_founded`), re-sync of changed imports (added/removed/changed) | each is one undo step; "source removed" kept, not deleted |
| **CT-8** | Optional Chronos **export** of a filtered view as a pasteable block (the one piece of option (b) taken) | exported text parses with `chronos::parse` with `skipped` empty; each event round-trips (`to_line` then `parse_line`) |
| **CT-9** | F5 (Timelines plugin), only on the owner's word and after reading that plugin's real syntax | n/a until asked |

Out of scope everywhere: replacing the year cursor's owner, re-baselining `history/timeline.json`, new simulation behaviour, generation changes (so no Ruling FM/§7p golden moves).

---

## 8. Owner questions, with defaults

| # | Question | Default |
|---|---|---|
| **Q1** | (a) or (b)? Option (b) means re-owning the year cursor, a new storage for snapshots, conflicts and journeys, and a save-format migration | **(a), with Chronos export (CT-8) and Chronos as the import format.** If (b), say so and I scope the migration as its own design |
| **Q2** | Is the strip's selection-bound behaviour kept as the default view (rows for the selected item) or is "all items, filtered" the default? | **Selection follows** (today's behaviour), "all items" one click away |
| **Q3** | When does the prompt open: once automatically at first detection per vault, or only from the passive badge? | **Passive badge only**; the prompt opens when the user clicks it |
| **Q4** | Default for an unlinked note's timeline | **Keep as the note's own** (changes nothing) |
| **Q5** | Is a note's year the engine's year? Import asks an epoch offset | **Offset 0, shown, editable**; year 0 = the generated world, as `founded` already assumes |
| **Q6** | Private notes: may their imported events be saved in the project file and in a shared `.zip`? | **Saved in the project (like `imported_text`), flagged `private`, hidden from the assistant and from exports** |
| **Q7** | Should F2's `founded` be offered to set `PlaceExtras::founded`, and what about `born`/`died`/`reign`? | **`founded` only** (it has a destination); the rest import as events |
| **Q8** | Which extra forms matter in the owner's vault (year lists, tables, the Timelines plugin)? CT-0 measures; this picks what ships | **F1+F2 first; F3/F4 only if CT-0 passes; F5 on request** |
| **Q9** | Should `TL_LAYERS`' six "no layer renders yet" pills now drive rows (Warfare = conflicts, Politics = ownership, ...)? | **No**, keep them as stated intent; the Kind filter is the working control |
| **Q10** | Should the day cursor (`tl_day`, unsaved today) be saved so a journey position survives a reload? | **Yes, as an optional `day` member of `history/timeline.json`** (additive); noted here because the Journey rows depend on it, not built by CT |

---

## 9. What I did not verify

`civ_settlement_ownership_periods` and `civ_settlement_population_trajectory` cost notes were read from `OwnershipSpan`'s and `PopulationPoint`'s doc comments, not benchmarked; no timing here is measured. The Timelines plugin's syntax (F5) is recalled, not read. Whether any export path currently drops `vault.json` link text for private notes was not traced (Q6). No test, probe or build was run; this is a design.

## 10. Build notes (Ruling GE build, 2026-11-17)

**Commit 1: the Rust layer (CT-0, CT-1, CT-3, CT-6, CT-8, and the data halves of CT-4, CT-5, CT-7).**

- **Where things live (a disclosed deviation).** The design put the read model in `cartalith-civ` and said nothing of where imports live. `cartalith-civ` now depends on `cartalith-vault` (acyclic; vault has no engine dependency) for `MonthDay` and `EntityKind`, so `TimelineItem` can name them as the design shows. Detection, decisions and the imported-events document live in `cartalith_vault::timelines` because the rows name vault notes and vault already owns serde. The bridge fills `ItemSources` from the real stores.
- **CT-0, the corpus pass, was run on a synthetic hand-labelled sample only.** The owner's vault was out of bounds for the build session, so F2's "0 false positives on the owner's vault" and F5's recalled-plugin count are **not measured**. The real pass is `#[ignore]` test `ct0_corpus_pass_over_a_real_vault` (set `CARTALITH_CT0_VAULT`; it reads Markdown only and prints per-form counts). On the sample: F1 is the parser (precision 1.00 by construction); **F3 precision 12/13 = 0.923 (recall 12/12), one acknowledged false positive** (a ranked list `- 1: gold ... - 4: wood` under any heading passes the ascending-run rule); **F4 first measured 6/7 = 0.857, below the 0.90 gate**, because a meeting log of `2024-05-01` dates read as a timeline. The fix is a principled guard, not a tuned threshold: a table whose year cells are all full `YYYY-MM-DD` dates in 1900-2100 is real-world metadata and is refused (the same reason F2 excludes `date`/`created`); F4 is then 6/6 with no false positive. **The sample is the author's, so this gate is self-fulfilling to the extent the negatives are the author's; treat the shipped state of F3/F4 as provisional until the real-vault pass.** `SHIP_YEAR_LIST` and `SHIP_TABLE` switch each off.
- **Bars and mutants.** 19 tests in `timelines`, 8 in `timeline_items`; 17 literal mutants, each named in the commit message, all killed (one survivor, M5 "heading match loosened", was a weak test and a bad mutant: the test now asserts `## Eventsourcing` is not a timeline heading, and the mutant dies).
- **Not done in this commit:** shell rows, filters UI, badge, prompt, bridge functions, the `history/events.json` slot wiring and its project save/load, journey lengths (a journey is a point until the bridge can supply `calendar_days`).

**Commit 2: the bridge and the saves (CT-4/5/7 data halves wired).** `history/events.json` is registered in `cartalith-io::DOCUMENT_SLOTS` and `project_bridge.rs::ENGINE_OWNED_SLOTS`, written only when `VaultSession::events` is non-empty and restored on open (a bad row is reported, never fatal); `LinkStore::timelines` is restored with the links and both are cleared where `links` is cleared. `chronos_bridge.rs` exposes the `#[func]`s listed in `STATUS.md`. **Checkpoint commit; the shell work follows.** Remaining, in order: the shell rows and filters (CT-2; a new `chronos_items.gd` hosted by `_build_timeline_expanded` and the phone strip, filters persisted beside `_tl_save_layers`), the badge and prompt window (CT-4; `AcceptDialog`, `exclusive = false`, defaults Link when linked else Keep, no bulk Import), the Import/re-sync/founding buttons (CT-5, CT-7), then windowed probes at desktop 1600x900, tablet 1700x1000 and phone 450x975 with frames looked at, and literal mutants for the GDScript. CT-9 only on the owner's word and after reading the plugin's real syntax. Note for the next builder: `cartalith-native/target` must point at the build's target dir for the `.gdextension` to find the DLL, and Python edits on Windows need `encoding='utf-8'`.

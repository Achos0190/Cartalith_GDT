# Markdown Vault: investigation, entity audit, and the milestones

> **This document defines milestones 0-6 and records what each pass found; it
> does not track them.** Where any milestone stands, and whether a blocker
> named below still holds, is recorded only in
> `cartalith-native/docs/STATUS.md`. Read this file for what a milestone *is*,
> what it must not break, and why §2's seven disagreements were resolved the
> way they were.

`ROADMAP.md` carried the Markdown Vault under "Options kept open, not
scheduled" and required this document before any code, for a specific reason
it stated: the owner-supplied design's own V1 acceptance criteria
(`MARKDOWN_VAULT_INTEGRATION.md` §35) assume entity concepts this port might
not have. **The owner asked for the work to start on 2026-08-24**, naming
three entity kinds: **continents, provinces and settlements** — and explicitly
*not* POIs, which are a deliberate absence in this port.

So the first job was not to write code. It was to find out whether those three
entities exist.

**This is new scope, not a port.** Nothing in `reference/Cartalith Gen1
v2.10.html` links anything to an external Markdown corpus —
`reference/FUNCTION_INDEX.md` was searched for `markdown`, `vault`, `note`,
`obsidian`, `wiki`, `link` and `knowledge` and the only hits are unrelated
(`_civLinkPlaces`, road topology). It therefore sits outside `DECISIONS.md`
§7d's parity contract entirely: there is nothing to preserve and nothing to
modernise over. What replaces golden-parity discipline here is stated in
`cartalith-vault`'s own crate doc and enforced by its tests — *a write that
changes nothing produces a byte-identical document, and a write that changes
one section leaves every other byte alone.*

---

## 1. The entity audit, and what it found

Checked against the real code, not against the scope documents.

### Settlements — real, and the strongest of the three

`WorldGen::get_settlements()` returns one dictionary per settlement with a
`tid`: `NamedSettlement::tid`, this port's own stable id, assigned in
`compute_civilisation` via `cartalith_civ::timeline::civ_assign_tid` and
deliberately designed (`TIMELINE_SCOPE.md` milestone 1) to survive a rename, a
move and a neighbouring deletion. `place_editor_window.gd` already edits a
settlement by index and displays its `tid`.

**Knowledge links key on `tid`, never on the array index.** An index shifts
every time an earlier settlement is deleted; a link that followed the index
would silently re-point at a different town.

### Provinces — real, with a weaker id

`WorldGen::get_provinces()` returns `Province { id, faction, name,
capital_settlement_index }`, produced by `cartalith_civ::civ_generate_provinces`
— a settlement-seeded Voronoi partition of `assign_territory`'s output, plus a
per-cell `Vec<i32>` raster.

`Province::id` is **sequential over the seed order**, not a persistent
identity. `civ_recompute()` rebuilds the partition, and a faction that gains or
loses a city-tier settlement can renumber its provinces. Real, and recorded
rather than smoothed over — see §4.

### Continents — **did not exist**, and now do

This is the finding the audit was for.

There is no continent, landmass or region entity in `cartalith-terrain`,
`cartalith-engine` or `cartalith-civ`. What the roadmap audit called "world
structure archetypes" is `cartalith_terrain::generate_continentality_field` — a
**per-cell scalar field**, `continentality`/`fragmentation` knobs producing a
`Vec<f32>` that biases height. It has no per-instance identity, no name and no
boundary. It is a generation-time classification exactly as suspected.

What *does* exist, one layer along, is a genuine connected-component labelling
of land. `cartalith_civ::build_landmass_quality` (reference line 5970,
golden-verified) runs an 8-neighbour flood fill over every land cell and
returns `LandmassQuality { quality, comp, sizes, count }` — where `comp` is a
per-cell component id and `sizes` is each component's cell count. Its own doc
comment says those three fields are *"not consumed by this milestone, kept for
parity with the reference's real shape and for later milestones"*, and
`compute_civilisation` has computed and discarded them on every generate since
Phase 2.

**So a minimal addressable continent was small, and it is milestone 0 below**:
retain that labelling, rank it, name it, and give each landmass a boundary. The
partition itself is not new code — it is the same golden-verified flood fill,
with its bookkeeping kept instead of dropped.

**The honest caveat, stated once and repeated at the binding**: a continent's
id is its **rank by area**, derived from the height field on every generate. It
is stable under anything that does not change the size ordering, and it is not
stable across a terrain edit that merges or splits a landmass. A settlement's
`tid` is a real persistent id; a continent's is not, and no amount of wanting
one makes the data carry one. §4 says what the design does about that.

### POIs — confirmed absent, and not built

`civ_tools_bridge.rs`'s module doc, `GUI_GAP_REGISTER.md` CV-01 and
`place_editor_window.gd`'s own footer all record POI as an unported concept.
`cartalith_vault::links::EntityKind` therefore has three variants and no `Poi`,
and the enum's doc comment says why. §3 of the design lists POIs and region
labels; §35's criteria 6 and 7 ("attach a specific section to a POI", "attach a
region document to a region") are **not satisfiable in this port** and are
listed as such in §6 below.

---

## 2. What the design asks for that this port cannot give it

Read `MARKDOWN_VAULT_INTEGRATION.md` in full, including its owner-amended
header. These are the places where the design and this port disagree, each
resolved rather than left open.

| Design | This port | Resolution |
|---|---|---|
| §3 entity scope includes **POIs** and **region labels** | Neither is a ported concept | Not built. `EntityKind` covers settlement/province/continent, and **faction** (CV-22) and **culture** (CV-02) were both added on 2026-08-25 — §3's own "add a kind later without redesigning the storage model" requirement is therefore not a claim but a measured one: each was a variant, an `as_str` arm, a `parse` arm and an `entity_values` arm. |
| §26 puts `knowledgeLinks` **inside the Cartalith project save** | *When this audit was taken*, the only save path was the reference HTML app's own `.zip` (`SAVEFILE_COMPAT.md`), which carries **no civ layer at all** — `WorldGen::load_save`'s own doc says `get_settlements()` comes back empty | Links live in `user://markdown_vault.json` (`vault_store.gd`). A link written into a save that carries no civ layer comes back pointing at a `tid` that no longer exists. **Milestone 3** below is the change that makes §26 possible — and its precondition is a save format that carries the layer, which is a property of whichever format is current, not of the one recorded here. |
| §23 rule 2: *"user content outside the block is immutable"* | The owner's 2026-08-18 amendment adds field population into the author's own template | Both mechanisms exist and are separated by policy, not by hope: the delimited block is machine-owned and regenerated unattended; author-field population is `FieldFill::OnlyIfEmpty` by default, previewed, confirmed, and **reports "skipped, you had already filled it"** rather than overwriting. `markdown::fill_field` is the one place that can write outside the block, and it refuses an occupied field. This is the reconciliation the design's header asked whoever wrote this document to make. |
| §11 offers `TextRange` and `MarkdownBlock` selections | — | Not built, and this is a correctness decision rather than a scope cut. A byte offset stops pointing at the right paragraph the moment the author edits the text above it, and a block reference (`^abc123`) is an Obsidian construct the owner's clarification put out of core. V1 ships the two selections §11 itself prioritises: whole document and heading section. |
| §19's Geography group wants a **continent** field on a settlement | Answering "which landmass is this cell on" needs the per-cell component raster | Not offered. `civ_continents` deliberately keeps no raster — 268 MB at this port's 8192² ceiling for a lookup nothing else performs (`MEMORY_OPTIMIZATION_SCOPE.md`'s standing objection to exactly that shape). Filling it from bounding-box containment would be a guess, and a wrong one wherever two boxes overlap. |
| §21 map snapshot, §19's *Open-in-Cartalith link* | — | **Milestone 2.** The snapshot needs a crop of the live renderer at three radii; the open-in link is URL-scheme registration, which the owner's clarification put outside the core. |
| §6 Android via the Storage Access Framework | `std::fs` cannot reach a `content://` tree URI | **Milestone 4.** The seam is kept honest now: nothing above `provider.rs` takes a `PathBuf`, so a SAF provider slots in beside `FsVault` rather than through it. |
| `DCC_SHELL_SPEC.md` §9's vault block in the Data manager | Assumes `obsidian://` links in exported tiles, note links in exported GeoJSON, and a **two-way sync toggle** | **Left alone**, per `ROADMAP.md` and `MARKDOWN_VAULT_INTEGRATION.md`'s own header. Two-way sync is §33's explicit V1 non-goal; the `obsidian://` scheme is out of core. Nothing in this pass touches the Data manager. |

---

## 3. Where the code lives

| Crate / file | Owns |
|---|---|
| `cartalith-civ::civ_continents` / `Continent` | Milestone 0: the addressable landmass |
| **`cartalith-vault`** (new crate) | Every Markdown, section, block, link and provider decision. Depends on `serde`/`serde_json` and nothing else in the workspace — no engine crate, no `gdext`. |
| `cartalith-vault::markdown` | Section spans, section replacement, author-template field lines |
| `cartalith-vault::block` | The machine-owned `CARTALITH:BEGIN/END` block |
| `cartalith-vault::links` | `KnowledgeLink`, `LinkStore`, the six status states |
| `cartalith-vault::provider` | `FsVault`: bounded listing, path containment, atomic writes |
| `cartalith-vault::export` | The exportable-field registry and the block renderer |
| `cartalith-vault::links::ImportedData` | Milestone 6: the note's information copied into Cartalith's own JSON |
| `cartalith-vault::WritePrefs` | Milestone 6: the *confirm always* choices, device-scoped |
| `cartalith-godot/src/vault_bridge.rs` | The `#[func]` surface, and turning a Cartalith entity into values |
| `godot-project/shell/vault_window.gd` | The panel: connect, browse, attach, read/edit, preview, write |
| `godot-project/shell/vault_store.gd` | `user://markdown_vault.json` |
| `place_editor_window.gd` §KNOWLEDGE, `civilization_workspace.gd` §Linked notes | §28's requirement that the vault live in the entity's own panel |

`cartalith-vault` is a new crate rather than a module of `cartalith-io`
because it is not the save format, and rather than a module of
`cartalith-godot` because it must be testable without a Godot runtime — which
is what let the round-trip guarantee be asserted 41 times before any UI
existed. This follows `ARCHITECTURE.md`'s ladder: it is not generation logic,
it does not orchestrate subsystem crates, it is not the HTML app's save format,
and it touches no Godot type.

---

## 4. The identity problem, and what the design does about it

Three entity kinds, three different strengths of id:

| Entity | Key | Survives a rename/move | Survives `civ_recompute()` | Survives a regenerate | Survives save/load |
|---|---|---|---|---|---|
| Settlement | `NamedSettlement::tid` | **Yes** | **Yes** (kept settlements keep their tid) | No | No — civ is not saved |
| Province | `Province::id` | Yes | Only if the seed set is unchanged | No | No |
| Continent | rank by area | Yes | Yes (terrain unchanged) | No | No |
| Faction | roster row index | Yes | Yes | No | No |
| Culture | `CIV_CULTURES` index | **Yes** | **Yes** | **Yes** | **Yes** |

**The last column was answered against the save path of the day** — the
reference HTML app's `.zip`, which carries no civ layer, so nothing keyed on a
generated entity could survive it. That column is a property of whatever save
format is current rather than of the id designs beside it, and it is the one
column here to re-read against the code instead of against this page. The other
four are properties of the ids themselves and do not move.

The last row is the exception that proves the rule: a culture's id is an index
into a **compile-time** table of seven, identical in every world, so a culture
link is the only one here that a regenerate and a save/load both leave intact.
That is a consequence of cultures not being generated, not a design achievement
— and the cost is stated with it: reorder `CIV_CULTURES` and every existing
culture link silently re-points. A test asserts its length and order for that
reason.

Rather than pretend, every `KnowledgeLink` also stores `entity_label` — the
entity's **name at link time**. Nothing resolves by it and it is never a
fallback key. It exists so that when an id goes stale the panel can say *"this
note was linked to Nareth"* and let a person re-bind, which is §32's "stop and
ask the user rather than guessing" applied to identity rather than to content.

---

## 5. Milestones

### Milestone 0 — the addressable continent

`cartalith_civ::Continent` and `civ_continents()`. `build_landmass_quality`'s
existing golden-verified 8-neighbour flood fill, with its component
bookkeeping kept instead of discarded:

- `id` — 1-based **rank by area**, largest first. Chosen over the raw component
  index, which is scan order and would renumber every landmass when an island
  appears in the top-left.
- `name` — `civ_settle_name` in the naming culture of the faction holding the
  most cells on that landmass. There is no separate continent-name vocabulary
  in the reference and inventing one is out of scope, so this reuses the one
  syllable generator the world already speaks in.
- boundary — inclusive cell bbox, cell-space centroid, cell count, plurality
  faction.
- `CONTINENT_MIN_CELLS = 64` is a floor on what gets **listed**, not a
  definition of "continent". An archipelago world legitimately has none, which
  is a real outcome rather than missing data.

No new per-cell memory: `CivData` gains a `Vec<Continent>` (metadata only) and
no raster. Exposed as `WorldGen::get_continents()`.

**One bug this milestone shipped and the end-to-end run caught**: naming
continents from `civ_name_rng` gave continent 1 and settlement 1 the *same
name* in a real generated world, because that stream's seed is a fixed
reference quirk (`state.seed||12345` — see its own doc comment) and both were
drawing its first value. Continents now have their own stream,
`civ_continent_name_rng`, and a test named after the failure.

### Milestone 1 — link, read, section-aware write-back

The `cartalith-vault` crate, its bridge, and the panels. Specifically:

- **Connect** a vault folder (§6, §7) — any folder of `.md` files. Idempotent
  by display name, so §7's "Connect Existing Vault" on a second device is the
  same call with a different path.
- **Browse** (§9) — bounded, sorted, dot-directories skipped, no file opened by
  the listing. §31's "do not load the entire vault into memory" is a property
  of the walk, not a promise.
- **Attach** (§11-§13) — whole document or one heading section, validated at
  attach time: a section that does not exist, or whose title is duplicated in
  the file, is refused rather than becoming a link that can never be read.
- **Status** (§27) — Unbound / Missing / Stale / Cached / LocalChanges /
  Connected, with the content hash outranking the timestamp when both are
  known (a file touched by a sync client is *not* stale, and calling it stale
  would train the user to ignore the warning).
- **Edit and write back** (§15, §16) — the working copy diverges locally;
  `Insert updated section` replaces **only that section**. Every write takes an
  `expect_hash` the caller can only have from a preview, so a source edited in
  the user's own editor in between refuses and writes nothing.
- **The Cartalith block** (§18, §23, §24) — `<!-- CARTALITH:BEGIN entity="…"
  version="1" -->` … `<!-- CARTALITH:END -->`. Insert below frontmatter and
  title; update replaces only the block; remove restores the document exactly.
  A `BEGIN` with no `END`, or two blocks for one entity, refuses outright
  (§23 rule 4).
- **Author-field population** — the owner's amendment, `OnlyIfEmpty` by
  default, previewed with a per-field outcome report.
- **The export registry** (§19, §20) — data-driven, filtered twice: by what an
  entity kind *can* have, and by what this entity *does* have. A field with no
  value never reaches a checkbox and therefore never reaches a note as a blank
  row.

### Milestone 2 — the map snapshot (§21, §22)

Reuse the current renderer at immediate/local/regional radii; store under a
user-accepted location; reference it from the block. It depends on nothing
milestone 1 did not already have — `export_raster.rs` crops — and was split
out because it is a separate, self-contained piece of work whose inclusion
would have made milestone 1 unreviewable.

### Milestone 3 — project-scoped links (§26)

**The requirement**: the save format has to carry the civ layer, because a
link written into a save that does not is a link pointing at settlements the
loaded world will not have. When this document was written the only save path
was the reference HTML app's `.zip`, which carries no civ layer at all — that
is the reason recorded in §2's table, and it is the condition to re-check
against the current save format rather than to assume still holds.
`vault_store.gd` is the one file that has to move when this lands: it is what
owns `user://markdown_vault.json` as the live store.

### Milestone 4 — the Android provider (§6)

Storage Access Framework: a tree URI, a persisted permission grant, and a
provider implementation beside `FsVault`. Cross-device vault identity (§35
criterion 2) is designed for and unverified until this exists.

### Milestone 6 — search, the note as data, culture, and "confirm always"

The owner's direction, verbatim:

> *"Let's be certain if the user links a vault the user can add cultural data
> or settlement data to the respective entity they want to add it to. Search
> for it it in the vault etc. The information then gets copied to a json. When
> the user ever wants to change this or add information to the markdown file
> it's an explicit action with a prompt confirmation (the prompt should have an
> option to confirm always)"*

Four requirements. This pass is the **Rust half**: the engine, the
`cartalith-vault` crate and the `#[func]` surface. The panel work — a search
field, a culture picker, a "note says" readout and the *don't ask again*
checkbox — is a separate pass.

Still outside `DECISIONS.md` §7d's contract, for the reason §0 of this
document already gives: nothing in `reference/Cartalith Gen1 v2.10.html` links
anything to a Markdown corpus, so **there is no golden target for any of it**.
What replaces golden parity is unchanged — round trip and non-destruction —
and the CPU pipeline's numeric behaviour was not touched. Workspace: 138
binaries, **2,204 → 2,216 passed**, 0 failed, 8 ignored.

#### What was already there, and one finding that was wrong before it was built

Checked against the code first, because this repository's most common defect
is registering something as missing that already exists:

| # | Requirement | What was actually there |
|---|---|---|
| 1 | Attach to the right entity | **Complete.** `EntityKind` already covered settlement/province/continent/faction and `attach` already validated the kind and the section at attach time. |
| 2 | Cultural data | **Half there, and the missing half was not the half it looked like.** Culture was *not* unexposed: `civ_culture_vocabulary()` has shipped the seven keys as a `#[func]`, `get_factions()` reports each faction's `culture`, and `civ_set_faction_field` validates and sets it. What was missing was that a **culture was not addressable** — no `EntityKind`, so no note could be attached to one. |
| 3 | Search the vault | **Genuinely absent.** Browsing listed files, `vault_file_headings` listed one file's headings, and `entity_mentions` searched for *one entity's own name*. Nothing let a person type a word. |
| 4 | Copied into a JSON | **Half there, and this is the correction that matters most.** `attach` has copied a note's **prose** into `KnowledgeLink::imported_text` since milestone 1, and `LinkStore::to_json` writes it out — §35 criterion 8 was already ticked. What did not exist was a copy a *program* can read: a note saying `**Size / Population:** 8,420` left Cartalith holding a paragraph, not a population. |
| 5 | Explicit confirmed write-back | **Complete except the option the owner asked for.** All three write paths preview first, confirm through a `ConfirmationDialog`, and carry an `expect_hash` from the preview so a file edited in between refuses. No "confirm always". |

**This is not the model inversion it was scoped as.** The reference model is
intact: a link still points at a file and the file is still the source of
truth. What changed is that the copy taken at attach time is now *structured*
as well as textual. Nothing was removed and no existing behaviour moved.

#### What was built

- **`EntityKind::Culture`** — a culture is `CIV_CULTURES[id]`, the seven
  compile-time rows at reference line 14607, addressed by its **0-based
  index**. `get_cultures()` is the binding `GUI_GAP_REGISTER.md` CV-02 asked
  for ("a fuller `get_cultures()` is one binding"), and it is non-empty before
  any `generate()` — the Riverlands' name pool does not depend on a height
  field, and only its aggregates do.
- **`VaultSession::search`** — names always, content when the backlink index
  has been built.
- **`ImportedData`** on every link — the note's YAML frontmatter and its
  `**Name:**` template lines, captured in the same read as `imported_text`.
- **`WritePrefs`** — three independent *don't ask again* flags.

#### The five design questions, and how each was resolved

**1. Where does the copied JSON live?** In `LinkStore`'s existing JSON, on the
link it belongs to. That is deliberately **not** a new persistence path: it
rides whatever carries the link store, so when the save-format restructure
lands (milestone 3, unblocked by the owner's 2026-08-25 ruling that saving is
strictly the new format) the copy moves with the links and needs no separate
home. Nothing was parked in a private file and nothing was invented for the
device sidecar to hold.

**2. Every copied value is a string, and that is load-bearing.**
`population: 8420` is stored as the five characters `"8420"`. Two reasons, one
of them already paid for: **KV-04**, the same day, was Godot's `JSON` floating
`entity_id` to `1.0` and `source_modified` to `1787605785.0` — serde refused
both and every link was discarded on every boot. The owner intends the new save
format to be implemented in the HTML app too, and JavaScript has exactly the
same defect: every JSON number is a double and integers above 2^53 cannot round
trip at all. A map of strings cannot be corrupted by a layer that floats
numbers. It is also simply what the note said — `8,420`, `~8000` and `8420` are
three different things a person wrote, and parsing them into one number would be
Cartalith deciding what the author meant. `source_hash` has always been hex text
rather than a `u64` for the same reason; this is the precedent, not a new idea.

**3. What happens when the note changes after a copy?** Nothing new is
invented. The copy is taken in the same read as `imported_text`, from the same
bytes, under the same `source_hash` — so §27's existing vocabulary already
answers it: a link that reports **Stale** has a stale copy, and *Reload source*
moves text, data and hash together. There is deliberately **no**
refresh-the-data-only call, because it would let the fields be current while
the prose was not, under a status that could describe only one of them.

One hole this closed on the way: `write_section` is the single write path that
re-syncs a link's own hash, so it is the only one that could leave a stale copy
sitting under a **Connected** status — an edited section carrying a
`**Population:**` line is exactly that case. It now re-reads the copy from the
document it just wrote, and a test is named after the hole.

**4. What is copied?** The smallest thing that satisfies the owner's sentence:

- the leading YAML block's **flat scalars only**. An indented line (a nested
  map or a `- list` item), a `#` comment, a line with no colon and a key with
  an empty value are skipped rather than half-parsed. A **duplicated key is
  omitted entirely**, not resolved last-wins — the same refusal-to-guess
  `find_section` applies to duplicate headings.
- the author's `**Name:** value` lines, minus any still holding the template's
  own `[bracketed prompt]`. Copying `[City / Town]` in as this settlement's
  type would be importing the question as though it were the answer.
- **as two maps, not one.** `type: town` in the frontmatter and `**Type:**
  City` in the body are two authoring surfaces that can legitimately disagree;
  merging them needs a precedence rule nobody asked for.
- **whole-document scope, always**, even for a heading selection. Frontmatter
  is document metadata by definition, and a settlement note's population line
  commonly sits under a `### General Info` heading the user did not attach. One
  rule rather than a selection-dependent one that surprises.
- **not deduplicated across notes.** Two notes on one settlement may disagree,
  and every row carries the path it came from so the disagreement is visible
  and attributable instead of silently resolved.

This is not a YAML parser and must not become one — that would drag a
dependency and an error type into a crate whose entire contract is that it
never rewrites what it does not understand.

**5. What may "confirm always" skip?** The dialog, never the guard. A caller
with the preference set still has to call the matching `vault_preview_*` —
that is where `expect_hash` comes from — and simply not display it. A note
edited between the preview and the write still refuses, whether or not anyone
was asked. Three independent flags rather than one, because replacing a
section, regenerating a machine-owned block and writing into the author's own
template lines are three different risks.

`always_field_fill` sits against `MARKDOWN_VAULT_INTEGRATION.md`'s own header
(*"offered and explicitly confirmed, never silent"*). It is honoured because
the owner asked for it and it is safe because `FieldFill::OnlyIfEmpty` still
refuses an occupied field whether or not anyone is watching. Recorded here
rather than left as a silent contradiction.

The preferences are **device state**: kept off `LinkStore` because one
person's "stop asking me" must not travel into another person's copy of a
project (§5). Same split `BacklinkIndex` already makes — its own JSON, its own
file.

#### Verification

Twelve new tests, each shaped to reach the code rather than to pass:

- `search_finds_notes_by_name_and_by_content_and_says_when_it_could_not_look`
  — including the failure paths: no matches, an empty query, a two-character
  query, the cap, and **the un-built index, where an empty answer must be
  reported as "did not look" rather than "nothing there"**.
- `a_notes_information_is_copied_into_the_json_and_reads_back_without_the_vault`
  — the owner's sentence end to end, including reading it back on a device that
  has never seen the folder.
- `a_note_with_malformed_frontmatter_still_attaches_and_copies_nothing_wrong`
  — an unterminated block, a line with no colon, an indented line and a
  duplicated key in one fixture; the note still attaches and the copy is empty.
- `a_note_that_changed_after_the_copy_is_stale_until_reload_moves_both_halves`.
- `writing_a_section_back_refreshes_the_copy_it_just_invalidated`.
- `the_confirm_always_preference_persists_and_never_travels_with_a_project`.
- `a_culture_is_an_addressable_entity_with_a_permanent_id` and
  `a_culture_is_addressed_by_its_compile_time_index`, the second of which
  **asserts `CIV_CULTURES`' length and order**: a culture link's id is an index
  into a compile-time table, so changing that table silently re-points every
  existing culture link in every user's sidecar.
- `a_culture_is_offered_its_own_fields_and_no_places_fields` — from both sides,
  because the wrong answer available here is offering a culture a coordinate.
- `frontmatter_reads_flat_scalars_and_declines_everything_else`,
  `malformed_frontmatter_yields_nothing_rather_than_a_wrong_answer`,
  `field_values_import_answers_and_never_the_templates_own_questions`.

#### Known limitations of this milestone

1. **Content search needs the backlink index.** Names are always searchable;
   bodies are not until *Refresh* has run once. The alternative is opening
   every note on every keystroke, which §31 forbids outright.
2. **A culture's id is a compile-time index**, so it is the *most* stable id in
   this port — it survives a regenerate and a save/load, unlike every other
   kind — but it moves if `CIV_CULTURES` is ever reordered. A test asserts the
   order for exactly that reason.
3. **A link made before 2026-08-25 has an empty copy** until *Reload source*
   runs on it. Deliberately a `#[serde(default)]` rather than a format-version
   bump: an old sidecar loads and simply has no data yet.
4. **The copy is not fed back into the engine.** Cartalith holds what the note
   said and shows it; nothing sets a settlement's population from a note. That
   is a much larger question — it would make the vault a second source of truth
   for world state, which §36 explicitly forbids ("neither side should silently
   become the other") — and it is not what the owner asked for.
5. **This pass built the Rust half only.** The panel work — a search field, a
   culture picker, a "note says" readout and the *don't ask again* checkbox —
   was scoped as a separate pass, so nothing this milestone added was reachable
   by a user when it landed. The `#[func]` list it produced is in the retired
   `cartalith-native/docs/CHANGELOG.md`.

### Milestone 5 — the conflict UI (§14's *Compare*)

§14's three-way prompt is *Compare*, *Reload source* and *Keep current copy*.
Milestone 1 built the latter two and deliberately left Compare here, because
this shell has no diff widget to build it on. The two it built are the two
that cannot lose work, which is the right subset to have first.

---

## 6. `MARKDOWN_VAULT_INTEGRATION.md` §35, criterion by criterion

**Which milestone owns which acceptance criterion**, and the two the design
asks for that this port cannot satisfy at all. This is the map, not a
scoreboard — whether a milestone has met its criteria is in
`cartalith-native/docs/STATUS.md`.

| # | Criterion | Owned by |
|---|---|---|
| 1 | Connect a vault on Windows | Milestone 1 |
| 2 | Connect the same logical vault on Android | **Milestone 4.** Vault identity is display-name-derived and portable; the provider is not, and cannot be verified until a SAF provider exists |
| 3 | Browse Markdown files | Milestone 1 |
| 4 | Open a Markdown file | Milestone 1 |
| 5 | Attach a complete file to a settlement | Milestone 1 |
| 6 | Attach a specific section to a **POI** | **Not satisfiable in this port** — POI is not a ported concept (§1). Sections attach to settlements, provinces, continents, factions and cultures instead |
| 7 | Attach a region document to a **region** | **Not satisfiable as written** — there is no "region" entity. Provinces and continents are this port's nearest real equivalents and both are addressable |
| 8 | Import text into Cartalith | Milestone 1 for the prose; **milestone 6** widens it so the note's frontmatter and template fields import as *data* too, not only as prose |
| 9 | Edit the imported text locally | Milestone 1 |
| 10 | Detect a changed source by timestamp | Milestone 1 — with a content hash outranking the timestamp, per §27 |
| 11 | Compare **or** reload changed source | Reload and Keep: milestone 1. **Compare: milestone 5** |
| 12 | Explicitly insert an edited section back | Milestone 1 |
| 13 | Select information groups for export | Milestone 1 |
| 14 | Generate a snapshot with the existing renderer | **Milestone 2** |
| 15 | Preview the generated block | Milestone 1 |
| 16 | Explicitly write it | Milestone 1 |
| 17 | Update a block without altering surrounding content | Milestone 1 — the crate's central invariant, asserted three ways (§7) |
| 18 | Open the project when the vault is unavailable | Milestone 1 — every link reports Unbound, cached text stays readable, the map is untouched |
| 19 | Show what is cached/stale/missing/connected | Milestone 1 — the six status states of §27 |
| 20 | Keep vault data and world data logically separated | Milestone 1 — separate crate, separate store, separate file |

---

## 7. Verification

Because there is no golden fixture to match, the evidence is round-trip and
non-destruction, at three levels.

**`cartalith-vault`, 41 unit and integration tests.** The load-bearing ones:

- `replacing_a_section_with_its_own_text_is_a_byte_identical_round_trip` —
  including the last section, which has no trailing sibling to bound it.
- `insert_then_update_leaves_every_hand_written_byte_alone` — and
  `remove` returns the document to `HAND` **exactly**, which is what caught the
  blank-line-accretion bug: taking only the block's own terminator back left
  one of its two pads behind, so an add/remove cycle widened the gap by a line
  each time.
- `a_source_that_changed_since_the_preview_is_not_overwritten` — the author
  edits the note in between; the write refuses, not one byte moves, and a
  re-preview then writes correctly *with the author's concurrent edit intact*.
- `fill_field_never_clobbers_an_author_filled_value` — asserts the document is
  `==` unchanged, not merely that the value survived.
- `duplicate_headings_refuse_rather_than_guess`,
  `an_unterminated_marker_refuses_rather_than_overwriting`.
- `renaming_a_heading_in_the_working_copy_moves_the_link_with_it` — §32's
  "heading renamed" from the one direction V1 controls. Without it Cartalith
  would refuse to read a section it had just written itself.
- `a_hash_inside_a_fence_is_not_a_heading`, `a_tag_is_not_a_heading`,
  `crlf_documents_round_trip`.
- `resolve_refuses_every_way_out_of_the_vault` — `..`, `a/../../b.md`,
  absolute, drive-qualified, empty.

**`cartalith-civ`, 4 tests** for milestone 0, on a hand-built three-landmass
fixture whose every number the fixture states: rank order, exact bounding
boxes, exact centroid, plurality-faction naming, determinism, the empty-ocean
case, and `a_continent_is_not_named_after_the_first_settlement`.

**`_vault_probe.gd`, 54 end-to-end checks** — the real app, the real shell, a
real generated world, and a **real folder of real Markdown files on disk**.
Headless and windowed, both green. It generates a world, asserts continents are
ranked and named and bounded, writes a hand-authored note with frontmatter and
four sections, connects, attaches a real settlement **by tid**, edits, previews,
writes back, then asserts on disk that the section changed and that each of
seven hand-authored fragments is still there; writes and updates the Cartalith
block and asserts the file is otherwise unchanged; fills the author's template
fields and asserts the filled one was skipped; edits the file behind
Cartalith's back and asserts the write refuses and changes nothing; and opens
both panels and asserts they say what they should.

The probe is what found the continent-naming collision and a `String(int)` call
in the Civilization dock that no unit test could have reached.

---

## 8. Limitations, and which of them a milestone closes

Two kinds, kept apart because they behave differently. The first four are
**decisions**: they are the shape V1 was designed to have, and no amount of
further work closes them. The rest are **milestone-shaped**: each names the
milestone that closes it, and whether it is still open is in
`cartalith-native/docs/STATUS.md`, not here.

**Decided, not deferred:**

1. **Continent and province ids are derived**, not persistent (§4). No amount
   of wanting one makes the data carry a persistent id; `entity_label` is what
   the design does about it instead.
2. **No `continent` field on a settlement's export block** (§2's table) —
   refused because filling it from bounding-box containment would be a guess,
   and a wrong one wherever two boxes overlap.
3. **Setext headings (`Title\n====`) are not recognised.** ATX only, because
   that is what all four of the owner's real templates use.
4. **No `TextRange`/`MarkdownBlock` selections** (§2's table) — a byte offset
   stops pointing at the right paragraph the moment the author edits above it.
   A correctness decision, not a scope cut.

**Owned by a milestone:**

5. **Links profile-scoped rather than project-scoped**, and with it whether a
   link into a saved world resolves at all — **milestone 3**, whose
   precondition is a save format that carries the civ layer.
6. **No Compare view** for a stale source. Reload and Keep are milestone 1's
   two, chosen because they are the two that cannot lose work —
   **milestone 5**.
7. **No map snapshot**; the block is text — **milestone 2**.
8. **Android**, and with it the cross-device vault identity §35 criterion 2
   asks for, which is designed-for and unverifiable until a provider exists —
   **milestone 4**.

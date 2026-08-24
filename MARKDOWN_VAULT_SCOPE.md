# Markdown Vault: investigation, entity audit, and milestone 1

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
| §3 entity scope includes **POIs** and **region labels** | Neither is a ported concept | Not built. `EntityKind` covers settlement/province/continent; §3's own "add a kind later without redesigning the storage model" requirement is met — a new variant plus one `match` arm is the whole change. |
| §26 puts `knowledgeLinks` **inside the Cartalith project save** | `cartalith-io` writes the reference HTML app's own `.zip` (`SAVEFILE_COMPAT.md`), which carries **no civ layer at all** — `WorldGen::load_save`'s own doc says `get_settlements()` comes back empty | Links live in `user://markdown_vault.json` (`vault_store.gd`). A link written into a save would come back pointing at a `tid` that no longer exists. **Milestone 3** below is the change that makes §26 possible. |
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

Rather than pretend, every `KnowledgeLink` also stores `entity_label` — the
entity's **name at link time**. Nothing resolves by it and it is never a
fallback key. It exists so that when an id goes stale the panel can say *"this
note was linked to Nareth"* and let a person re-bind, which is §32's "stop and
ask the user rather than guessing" applied to identity rather than to content.

---

## 5. Milestones

### Milestone 0 — the addressable continent · **done, 2026-08-24**

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

### Milestone 1 — link, read, section-aware write-back · **done, 2026-08-24**

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

### Milestone 2 — the map snapshot (§21, §22) · **not started**

Reuse the current renderer at immediate/local/regional radii; store under a
user-accepted location; reference it from the block. Blocked on nothing —
`export_raster.rs` already crops — but it is a separate, self-contained piece
of work and bundling it would have made milestone 1 unreviewable.

### Milestone 3 — project-scoped links (§26) · **blocked**

Requires the save format to carry the civ layer. Until it does, a link inside
a save points at settlements a loaded world does not have. `vault_store.gd` is
the one file that has to move when this lands.

### Milestone 4 — the Android provider (§6) · **not started**

Storage Access Framework: a tree URI, a persisted permission grant, and a
provider implementation beside `FsVault`. Cross-device vault identity (§35
criterion 2) is designed for and unverified until this exists.

### Milestone 5 — the conflict UI (§14's *Compare*) · **not started**

`Reload source` and `Keep current copy` both ship; a diff view does not,
because this shell has no diff widget. The two shipped actions are the two
that cannot lose work, which is the right subset to ship first.

---

## 6. `MARKDOWN_VAULT_INTEGRATION.md` §35, criterion by criterion

| # | Criterion | Status |
|---|---|---|
| 1 | Connect a vault on Windows | **Done** |
| 2 | Connect the same logical vault on Android | **Milestone 4.** Vault identity is display-name-derived and portable; the provider is not. |
| 3 | Browse Markdown files | **Done** |
| 4 | Open a Markdown file | **Done** |
| 5 | Attach a complete file to a settlement | **Done** |
| 6 | Attach a specific section to a **POI** | **Not possible** — POI is not a ported concept. Sections attach to settlements, provinces and continents. |
| 7 | Attach a region document to a **region** | **Not possible as written** — no "region" entity. Provinces and continents are this port's nearest real equivalents and both are supported. |
| 8 | Import text into Cartalith | **Done** |
| 9 | Edit the imported text locally | **Done** |
| 10 | Detect a changed source by timestamp | **Done**, with a content hash outranking it |
| 11 | Compare **or** reload changed source | **Reload done; Compare is milestone 5** |
| 12 | Explicitly insert an edited section back | **Done** |
| 13 | Select information groups for export | **Done** |
| 14 | Generate a snapshot with the existing renderer | **Milestone 2** |
| 15 | Preview the generated block | **Done** |
| 16 | Explicitly write it | **Done** |
| 17 | Update a block without altering surrounding content | **Done**, and asserted three ways |
| 18 | Open the project when the vault is unavailable | **Done** — every link reports Unbound, cached text stays readable, the map is untouched |
| 19 | Show what is cached/stale/missing/connected | **Done** |
| 20 | Keep vault data and world data logically separated | **Done** — separate crate, separate store, separate file |

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

## 8. Known limitations, stated rather than discovered

1. **Links are profile-scoped, not project-scoped** (§2's table, milestone 3).
2. **Continent and province ids are derived**, not persistent (§4).
3. **A settlement's `tid` does not survive save/load**, because the save format
   carries no civ layer. Every link to a loaded world's settlement is Unbound
   in practice.
4. **No `continent` field on a settlement's export block** (§2's table).
5. **No Compare view** for a stale source; Reload and Keep only.
6. **No map snapshot**; the block is text.
7. **Android is unimplemented**, and cross-device vault identity is therefore
   designed-for and unverified.
8. **Setext headings (`Title\n====`) are not recognised.** ATX only, because
   that is what all four of the owner's real templates use.

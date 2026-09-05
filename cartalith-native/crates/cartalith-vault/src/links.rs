//! Knowledge links (`MARKDOWN_VAULT_INTEGRATION.md` §11, §26, §27).
//!
//! A [`KnowledgeLink`] is the whole relationship between one Cartalith entity
//! and one Markdown document. §36's engineering principle — "neither side
//! should silently become the other" — is why this type holds *references*
//! and a working copy, never the vault's own content as world data.
//!
//! ## Where this is stored (corrected 2026-09-02)
//!
//! §25 and §26 ask for a separate JSON layer, and this is it: [`LinkStore`]
//! is one JSON document, and `cartalith-godot`'s `project_bridge.rs` writes
//! it into the project archive's `vault.json` slot and reads it back through
//! [`LinkStore::from_json`].
//!
//! **The paragraph that used to stand here said the opposite, and it was
//! stale.** It read: *"`cartalith-io`'s save format is the reference HTML
//! app's own `.zip` … it carries no civ data at all … There is therefore no
//! entity in a loaded save for a link inside that save to point at."* That
//! stopped being true on 2026-08-25, when `DECISIONS.md` §7h replaced the
//! flat reference archive with the project tree — `cartalith_io::
//! DOCUMENT_SLOTS` lists `entities/settlements.json` and `vault.json` side by
//! side, so the entity a link points at and the link itself now travel in one
//! file. `MARKDOWN_VAULT_SCOPE.md` milestone 3 is that move; it does not
//! change one line of this module, which is the point of the crate boundary.
//!
//! What is still true and still worth stating: this crate holds *references*,
//! not the vault's content as world data, and a vault's **location** is never
//! in here (§5) — which is also what makes the vault block in
//! `DCC_SHELL_SPEC.md` §9 (note links written into exported GeoJSON and
//! tiles) something to leave alone rather than build.
//!
//! ## Entity identity is only as stable as the entity
//!
//! Recorded here rather than discovered later:
//!
//! | Entity | Key | Stability |
//! |---|---|---|
//! | Settlement | `NamedSettlement::tid` | Stable across edits, renames and moves within a session. **Not** across save/load — civ is not saved — nor across a fresh `generate()`. |
//! | Province | `Province::id` | Re-derived by every `civ_recompute()`; a province that gains or loses a seed settlement can change id. |
//! | Continent | landmass component index | Re-derived from the height field; any terrain edit that merges or splits a landmass renumbers it. |
//! | Faction | `faction_roster` row index | Stable while the roster is; `civ_remove_faction` renumbers the rows above the one removed. |
//! | Culture | `CIV_CULTURES` index | **Stable absolutely.** Seven compile-time rows, the same seven in every world, so a culture link survives a regenerate and a save/load — the only kind here that does. |
//!
//! So every link also stores [`KnowledgeLink::entity_label`] — the entity's
//! name at link time. It is not the key and nothing resolves by it; it exists
//! so that when a key goes stale the UI can say *"this note was linked to
//! Nareth"* and let a person re-bind, which is §32's "stop and ask the user
//! rather than guessing" applied to identity rather than to content.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The three entity kinds this port can genuinely address
/// (`MARKDOWN_VAULT_SCOPE.md` milestone 0's verification).
///
/// §3 of the design also lists POIs and region labels. **POIs are not a
/// ported concept** in this port at all — `civ_tools_bridge.rs`'s module doc
/// and `GUI_GAP_REGISTER.md` CV-01 record that as a deliberate decision, and
/// `place_editor_window.gd` already tells the user so — so there is no
/// `Poi` variant here and building one would be inventing an entity to hang
/// a feature on. The enum is `#[non_exhaustive]`-in-spirit: §3's own
/// requirement is that "additional Cartalith entity types can be added later
/// without redesigning the storage model", and adding a variant here plus a
/// `key()` case is that whole change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Settlement,
    Province,
    Continent,
    /// `GUI_GAP_REGISTER.md` **CV-22**, added 2026-08-25 — and the proof of
    /// the paragraph above: one variant, one `as_str` arm, one `parse` arm.
    /// A faction is `CivData::faction_roster`'s own row, addressed by its
    /// 1-based id exactly as a province is.
    Faction,
    /// `GUI_GAP_REGISTER.md` **CV-02**, added 2026-08-25 for the owner's
    /// *"the user can add cultural data … to the respective entity"*.
    ///
    /// A culture is `cartalith_civ::CIV_CULTURES[id]` — one of **seven**
    /// compile-time rows (reference line 14607), addressed by its **0-based
    /// index**. It is not generated, so unlike every other kind here its id
    /// is not derived from a world: see the stability table in this module's
    /// own doc, where it is the only row that survives a regenerate *and* a
    /// save/load. That is a property worth having rather than an accident —
    /// a person's essay on the Riverlands stays attached to the Riverlands.
    Culture,
}

impl EntityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EntityKind::Settlement => "settlement",
            EntityKind::Province => "province",
            EntityKind::Continent => "continent",
            EntityKind::Faction => "faction",
            EntityKind::Culture => "culture",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "settlement" => Some(EntityKind::Settlement),
            "province" => Some(EntityKind::Province),
            "continent" => Some(EntityKind::Continent),
            "faction" => Some(EntityKind::Faction),
            "culture" => Some(EntityKind::Culture),
            _ => None,
        }
    }
}

/// The stable string an entity is addressed by, in Cartalith and in the
/// Markdown block's own `entity="…"` attribute: `settlement:42`.
pub fn entity_key(kind: EntityKind, id: i64) -> String {
    format!("{}:{id}", kind.as_str())
}

/// What part of the document a link points at (§11).
///
/// §11 also lists `TextRange` and `MarkdownBlock`. Both need a source anchor
/// that survives the author editing the text above it — a byte offset does
/// not, and a block reference (`^abc123`) is an Obsidian construct the
/// owner's clarification put out of core. V1 ships the two selections §11
/// itself prioritises and says so, rather than shipping a `TextRange` that
/// silently points at the wrong paragraph after one edit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Selection {
    WholeDocument,
    Heading { value: String },
}

/// `SAVEFILE_COMPAT.md` §13.3.6: *"An unrecognised `type` MUST be read as
/// `whole_document` and reported. It MUST NOT cost the link, and it MUST NOT
/// cost the document."*
///
/// The derived `Deserialize` did the opposite. A `selection` shape a newer
/// writer added made this one link fail, which made `LinkStore::from_json`
/// fail, which made `project_open`'s `if let Ok(store)` skip the vault
/// entirely — **every link in the project discarded to protect against one
/// narrower selection than this build expected.** That is `GUI_GAP_REGISTER.md`
/// KV-04's failure shape exactly, one layer up: a whole knowledge layer lost
/// silently, on open, with `ok == true`.
///
/// Written by hand rather than with `#[serde(other)]` for one reason: `other`
/// needs a third variant, and a third variant would be re-serialised on the
/// next save as its own name, writing a selection type no reader knows. Folding
/// straight onto `WholeDocument` means what this build read is exactly what it
/// writes back — which is the honest record of what it understood.
impl<'de> Deserialize<'de> for Selection {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        match v.get("type").and_then(|t| t.as_str()) {
            Some("heading") => Ok(Selection::Heading {
                // A `heading` with no `value` is a heading selecting nothing,
                // which resolves to nothing — not a reason to lose the link.
                value: v.get("value").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
            }),
            _ => Ok(Selection::WholeDocument),
        }
    }
}

impl Selection {
    pub fn label(&self) -> String {
        match self {
            Selection::WholeDocument => "Whole document".to_string(),
            Selection::Heading { value } => format!("## {value}"),
        }
    }
}

/// §27's five states, plus the local-edit state §15 adds.
///
/// Ordered by how much the user needs to know: [`LinkStatus::Connected`] is
/// the quiet one, everything above it wants saying out loud.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkStatus {
    /// The project knows about the vault, but this device has not bound it.
    Unbound,
    /// Bound, but the file is not there (§32 "file deleted"/"file moved").
    Missing,
    /// The file's timestamp or hash differs from what was imported.
    Stale,
    /// The source is unreachable but the imported text is held locally.
    Cached,
    /// The user has edited the working copy and not written it back.
    LocalChanges,
    /// Source present and matching.
    Connected,
}

impl LinkStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            LinkStatus::Unbound => "unbound",
            LinkStatus::Missing => "missing",
            LinkStatus::Stale => "stale",
            LinkStatus::Cached => "cached",
            LinkStatus::LocalChanges => "local_changes",
            LinkStatus::Connected => "connected",
        }
    }
}

/// The note's own **structured** information, copied into Cartalith's JSON.
///
/// ## Why this exists, and what changed
///
/// Milestone 1 already copied a note's *prose*: [`KnowledgeLink::imported_text`]
/// holds what was read and [`LinkStore::to_json`] writes it out, so the
/// design's §35 criterion 8 ("import text into Cartalith") has been satisfied
/// since 2026-08-24. What it did **not** copy was anything a program can read.
/// The owner's 2026-08-25 direction — *"the user can add cultural data or
/// settlement data to the respective entity … The information then gets copied
/// to a json"* — is about data, not paragraphs: a note that says
/// `**Size / Population:** 8,420` should leave Cartalith holding
/// `population = 8,420`, not a string containing that sentence.
///
/// So a link now carries two small maps taken from the same read, and
/// `MARKDOWN_VAULT_SCOPE.md` milestone 6 records the reasoning.
///
/// ## Two maps, not one merged map
///
/// A note's YAML frontmatter and its `**Name:**` template lines are different
/// authoring surfaces and can legitimately disagree — `type: town` in the
/// frontmatter and `**Type:** City` in the body. Merging them needs a
/// precedence rule nobody asked for, and picking one silently is exactly the
/// guess §32 forbids. They are kept apart and the consumer decides.
///
/// ## Freshness is the link's, not its own
///
/// **There is deliberately no second staleness idea here.** This is captured
/// in the same read as [`KnowledgeLink::imported_text`], from the same bytes,
/// under the same [`KnowledgeLink::source_hash`] — so §27's existing
/// vocabulary already answers "is this copy current": a link that reports
/// [`LinkStatus::Stale`] has a stale copy, and *Reload source* refreshes both
/// halves together. Adding a per-map timestamp would let the two disagree,
/// which is a state the UI would then have to explain.
///
/// ## Scoped to the whole document, always
///
/// Even for a `Heading` selection. Frontmatter is document metadata by
/// definition, and a settlement note's `**Population:**` line commonly lives
/// under a `### General Info` heading the user did not attach. One rule,
/// stated once, rather than a selection-dependent one that surprises.
///
/// ## Every value is a string, and that is a decision
///
/// `population: 8420` in a note's frontmatter is stored here as the **five
/// characters** `"8420"`, never as a number. Two reasons, one of which this
/// project has already paid for:
///
/// 1. **KV-04, 2026-08-25.** Godot's `JSON` has a single number type and it is
///    `f64`, so a round trip floated `entity_id` to `1.0` and
///    `source_modified` to `1787605785.0`; serde refused both and the shell
///    discarded every link on every boot. The owner has since said the new
///    save format will also be implemented in the HTML app, and JavaScript has
///    exactly the same defect — every JSON number is a double, and an integer
///    above 2^53 cannot round-trip at all. A map of strings cannot be
///    corrupted by a layer that floats numbers, which is why this one is.
/// 2. It is also simply **what the note said**. `8,420`, `~8000` and `8420`
///    are three different things a person wrote, and parsing them into one
///    number would be Cartalith deciding what the author meant. Whoever
///    consumes a value decides how to read it; the copy preserves it.
///
/// The same rule is why [`KnowledgeLink::source_hash`] has always been hex
/// text rather than a `u64`. It is the precedent, not a new idea.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportedData {
    /// Flat scalar keys from the leading YAML block.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub frontmatter: BTreeMap<String, String>,
    /// `**Name:** value` lines from the author's own template.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub fields: BTreeMap<String, String>,
}

impl ImportedData {
    /// Everything readable as data in one document. Never fails: a note with
    /// no frontmatter and no field lines yields an empty value, which is a
    /// legitimate answer and not an error.
    pub fn from_document(text: &str) -> Self {
        ImportedData {
            frontmatter: crate::markdown::frontmatter_fields(text).into_iter().collect(),
            fields: crate::markdown::field_values(text).into_iter().collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.frontmatter.is_empty() && self.fields.is_empty()
    }

    pub fn len(&self) -> usize {
        self.frontmatter.len() + self.fields.len()
    }

    /// `(origin, key, value)` rows in a stable order — frontmatter first,
    /// then template fields. `origin` is `"frontmatter"` or `"field"`, and it
    /// is carried rather than dropped because a consumer that cannot say
    /// where a value came from cannot let a person correct it.
    pub fn rows(&self) -> Vec<(&'static str, &str, &str)> {
        self.frontmatter
            .iter()
            .map(|(k, v)| ("frontmatter", k.as_str(), v.as_str()))
            .chain(self.fields.iter().map(|(k, v)| ("field", k.as_str(), v.as_str())))
            .collect()
    }
}

/// A vault as the *project* knows it — an id and a name, no path (§5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultRef {
    pub id: String,
    pub display_name: String,
}

impl VaultRef {
    /// A vault id derived from the display name, so the same logical vault
    /// connected on a second device under a different path lands on the same
    /// id (§35 criterion 2) without either device having to have seen the
    /// other's project file first.
    pub fn new(display_name: &str) -> Self {
        VaultRef {
            id: format!("vault_{}", crate::provider::content_hash(display_name.trim())),
            display_name: display_name.trim().to_string(),
        }
    }
}

/// One entity-to-document relationship.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeLink {
    pub link_id: String,
    pub entity_kind: EntityKind,
    /// The entity's own id (`tid` for a settlement). See the module doc's
    /// stability table.
    pub entity_id: i64,
    /// The entity's name when the link was made — for re-binding a stale
    /// key, never for resolving one.
    pub entity_label: String,
    pub vault_id: String,
    pub relative_path: String,
    pub selection: Selection,
    /// Seconds since the epoch, as of the last import or refresh.
    #[serde(default)]
    pub source_modified: u64,
    #[serde(default)]
    pub source_hash: String,
    /// What was read from the vault. `None` means "linked but never
    /// imported" — a legitimate state (§27 distinguishes it from Cached).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imported_text: Option<String>,
    /// The Cartalith-side working copy, present only once it diverges (§15).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_text: Option<String>,
    /// The note's structured information, copied into Cartalith's own JSON —
    /// the owner's 2026-08-25 direction. Captured with `imported_text`, from
    /// the same read, under the same `source_hash`. See [`ImportedData`].
    ///
    /// Empty for a link written by a build before 2026-08-25; *Reload source*
    /// fills it. That is why it is `#[serde(default)]` rather than a
    /// format-version bump — an old sidecar loads and simply has no data yet.
    #[serde(default, skip_serializing_if = "ImportedData::is_empty")]
    pub imported_data: ImportedData,
}

impl KnowledgeLink {
    pub fn entity_key(&self) -> String {
        entity_key(self.entity_kind, self.entity_id)
    }

    /// The text the UI should show and the write-back should send: the
    /// working copy if there is one, otherwise what was imported.
    pub fn working_text(&self) -> &str {
        self.edited_text.as_deref().or(self.imported_text.as_deref()).unwrap_or("")
    }

    pub fn has_local_changes(&self) -> bool {
        match (&self.edited_text, &self.imported_text) {
            (Some(e), Some(i)) => e != i,
            (Some(_), None) => true,
            _ => false,
        }
    }

    /// This link's state given what the provider currently reports.
    ///
    /// `current` is `None` when the vault is not bound on this device or the
    /// file is not there; the caller distinguishes those two because only it
    /// knows whether a binding exists.
    pub fn status(&self, bound: bool, current: Option<crate::provider::FileMeta>, current_hash: Option<&str>) -> LinkStatus {
        if !bound {
            return LinkStatus::Unbound;
        }
        let Some(meta) = current else {
            return if self.imported_text.is_some() { LinkStatus::Cached } else { LinkStatus::Missing };
        };
        // The hash is authoritative when both sides have one: a file touched
        // by a sync client has a new mtime and identical bytes, and calling
        // that "stale" would train the user to ignore the warning. The
        // timestamp is the fallback, which is §14's own ordering.
        let changed = match current_hash {
            Some(h) if !self.source_hash.is_empty() => h != self.source_hash,
            _ => meta.modified != self.source_modified,
        };
        if changed {
            return LinkStatus::Stale;
        }
        if self.has_local_changes() {
            return LinkStatus::LocalChanges;
        }
        LinkStatus::Connected
    }
}

/// Every vault reference and every link a project holds — §26's save-file
/// model, as its own JSON document.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkStore {
    #[serde(default)]
    pub vaults: Vec<VaultRef>,
    #[serde(default)]
    pub links: Vec<KnowledgeLink>,
    /// §21's map snapshots, keyed [`snapshot_key`] and valued with the
    /// image's path **relative to the vault root** — the same convention
    /// [`KnowledgeLink::relative_path`] uses and for the same reason (§5: a
    /// device path is not project data).
    ///
    /// Keyed by *entity*, not by link. A snapshot is a picture of a place and
    /// exists whether or not anyone has attached a note to it; a settlement
    /// with three notes has one immediate map, not three.
    ///
    /// `#[serde(default)]` plus `skip_serializing_if`, not a
    /// [`STORE_VERSION`] bump — the same call milestone 6 made for
    /// `imported_data`, and for the same reason: an older store simply has no
    /// snapshots yet, and a document with none writes no member at all rather
    /// than an empty object every reader then has to ignore.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub snapshots: BTreeMap<String, String>,
}

/// The key one entity's snapshot at one radius is filed under:
/// `settlement:42|local`.
///
/// `|` because [`entity_key`] already spends the `:` and a radius name is
/// lowercase ASCII — so the key splits unambiguously at the last `|` and
/// there is nothing to escape.
pub fn snapshot_key(entity_key: &str, radius: &str) -> String {
    format!("{entity_key}|{radius}")
}

/// The `version` written into the store, so a later format can recognise an
/// earlier one rather than mis-parsing it.
pub const STORE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct Envelope {
    version: u32,
    #[serde(flatten)]
    store: LinkStore,
}

impl LinkStore {
    /// Whether this store has nothing worth filing — the question a caller
    /// deciding *"do I write a `vault.json` into this archive at all?"* is
    /// actually asking.
    ///
    /// It exists because that caller was asking a narrower one, and it is
    /// **the gate `project_bridge.rs` uses as of `52666b9`** — the call site
    /// reads `if !self.vault.store.is_empty()`. Before that commit it read
    /// `!self.vault.store.links.is_empty()`, one member of a three-member
    /// store, and [`LinkStore::snapshots`] is keyed by **entity, not by
    /// link**: its own doc says a snapshot "exists whether or not anyone has
    /// attached a note to it", and `vault_bridge.rs`'s `vault_snapshot`
    /// requires no link to file one. A project whose only vault state was a
    /// generated map therefore had `links.is_empty() == true`, wrote no
    /// document, and lost the snapshot map on save — with no second copy
    /// anywhere, since `shell/vault_store.gd` stops writing the `store` half
    /// of the sidecar for exactly the sessions that have a project open.
    ///
    /// Three members, three ways to have something to say, so the predicate
    /// belongs to the store rather than to whichever caller last guessed at
    /// it. [`ImportedData::is_empty`] is the same shape one layer down.
    ///
    /// **A new member of [`LinkStore`] has to be added to this conjunction in
    /// the same change that adds it.** Nothing structural forces that — the
    /// gate asks this one function, so a member missing from here is a member
    /// that silently does not survive a save, which is precisely the defect
    /// above. `a_store_holding_only_a_snapshot_is_not_an_empty_store` asserts
    /// each member alone for that reason.
    ///
    /// Note what it costs to be right here: a store holding only a
    /// [`VaultRef`] is **not** empty, so connecting a vault and linking
    /// nothing does write a small document. That is §26's own save-file
    /// model — `markdownVault: {vaultId, displayName}` is listed there
    /// beside `knowledgeLinks` — and it is the portable half of the split in
    /// `vault_store.gd`'s header table, not the device binding, which is
    /// never in here (§5).
    pub fn is_empty(&self) -> bool {
        self.vaults.is_empty() && self.links.is_empty() && self.snapshots.is_empty()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&Envelope { version: STORE_VERSION, store: self.clone() })
            .expect("a LinkStore always serializes")
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        // An empty file is an empty store, not a parse failure -- the shell
        // creates the sidecar before anything has been linked.
        if s.trim().is_empty() {
            return Ok(LinkStore::default());
        }
        Ok(serde_json::from_str::<Envelope>(s)?.store)
    }

    pub fn vault(&self, id: &str) -> Option<&VaultRef> {
        self.vaults.iter().find(|v| v.id == id)
    }

    /// The vault-relative path of one entity's snapshot at one radius, if one
    /// has been generated. `None` is the normal state and is what keeps the
    /// Map fields out of `export::offer`'s list until there is an image to
    /// point at — §20's "must not expose information that the entity does not
    /// possess", enforced by the data rather than by the panel.
    pub fn snapshot(&self, entity_key: &str, radius: &str) -> Option<&str> {
        self.snapshots.get(&snapshot_key(entity_key, radius)).map(String::as_str)
    }

    /// Files a freshly written snapshot. Overwrites, deliberately: a second
    /// snapshot at the same radius is a *newer picture of the same place*,
    /// and keeping the first would leave the note pointing at a stale map.
    pub fn set_snapshot(&mut self, entity_key: &str, radius: &str, rel: &str) {
        self.snapshots.insert(snapshot_key(entity_key, radius), rel.to_string());
    }

    /// Registers a vault, returning its id. Re-connecting a vault with the
    /// same display name is idempotent.
    pub fn add_vault(&mut self, display_name: &str) -> String {
        let v = VaultRef::new(display_name);
        if self.vault(&v.id).is_none() {
            self.vaults.push(v.clone());
        }
        v.id
    }

    pub fn links_for(&self, kind: EntityKind, id: i64) -> Vec<&KnowledgeLink> {
        self.links.iter().filter(|l| l.entity_kind == kind && l.entity_id == id).collect()
    }

    pub fn get(&self, link_id: &str) -> Option<&KnowledgeLink> {
        self.links.iter().find(|l| l.link_id == link_id)
    }

    pub fn get_mut(&mut self, link_id: &str) -> Option<&mut KnowledgeLink> {
        self.links.iter_mut().find(|l| l.link_id == link_id)
    }

    /// Adds a link, replacing any existing link from the same entity to the
    /// same file and selection — attaching the same section twice is a
    /// mis-click, not a second relationship.
    pub fn attach(&mut self, mut link: KnowledgeLink) -> String {
        // Drop the superseded link *before* minting, or `mint_id` collides
        // with the very link it is about to replace and re-attaching the same
        // section would walk the id up `_2`, `_3`, … on every click.
        self.links.retain(|l| {
            !(l.entity_kind == link.entity_kind
                && l.entity_id == link.entity_id
                && l.relative_path == link.relative_path
                && l.selection == link.selection)
        });
        if link.link_id.is_empty() {
            link.link_id = self.mint_id(&link);
        }
        let id = link.link_id.clone();
        self.links.push(link);
        id
    }

    pub fn detach(&mut self, link_id: &str) -> bool {
        let before = self.links.len();
        self.links.retain(|l| l.link_id != link_id);
        self.links.len() != before
    }

    /// A deterministic id from the link's own coordinates, with a counter
    /// suffix only if that collides. No RNG and no clock: two devices
    /// attaching the same section to the same settlement produce the same
    /// link id, which is what makes a project file mergeable by hand.
    fn mint_id(&self, link: &KnowledgeLink) -> String {
        let seed = format!(
            "{}:{}:{}:{}",
            link.entity_kind.as_str(),
            link.entity_id,
            link.relative_path,
            link.selection.label()
        );
        let base = format!("link_{}", crate::provider::content_hash(&seed));
        if self.get(&base).is_none() {
            return base;
        }
        (2..)
            .map(|n| format!("{base}_{n}"))
            .find(|c| self.get(c).is_none())
            .expect("an unbounded counter always finds a free id")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::FileMeta;

    fn link() -> KnowledgeLink {
        KnowledgeLink {
            link_id: String::new(),
            entity_kind: EntityKind::Settlement,
            entity_id: 42,
            entity_label: "Nareth".into(),
            vault_id: "vault_x".into(),
            relative_path: "Locations/Nareth.md".into(),
            selection: Selection::Heading { value: "The Old Quarter".into() },
            source_modified: 1000,
            source_hash: "aaaa".into(),
            imported_text: Some("## The Old Quarter\n\nNarrow streets.\n".into()),
            edited_text: None,
            imported_data: ImportedData::default(),
        }
    }

    #[test]
    fn the_store_round_trips_through_json() {
        let mut s = LinkStore::default();
        let vid = s.add_vault("Elaris");
        assert_eq!(s.add_vault("Elaris"), vid, "re-connecting is idempotent");
        let mut l = link();
        l.vault_id = vid;
        let id = s.attach(l);
        let back = LinkStore::from_json(&s.to_json()).unwrap();
        assert_eq!(back, s);
        assert_eq!(back.get(&id).unwrap().entity_key(), "settlement:42");
        assert_eq!(LinkStore::from_json("").unwrap(), LinkStore::default());
    }

    /// Milestone 2's snapshots ride the store that milestone 3 puts in the
    /// project archive, so this asserts both halves at once: the map survives
    /// `to_json`/`from_json` (which is what `project_bridge.rs` writes into
    /// and reads out of `vault.json`), a store with no snapshots writes **no
    /// member at all** rather than an empty object, and a document written
    /// before this member existed still parses.
    #[test]
    fn snapshots_round_trip_and_an_older_store_without_them_still_parses() {
        let mut s = LinkStore::default();
        assert!(!s.to_json().contains("snapshots"), "an empty map writes no member");

        s.set_snapshot("settlement:42", "local", ".cartalith/maps/settlement_42_local.png");
        s.set_snapshot("continent:1", "regional", ".cartalith/maps/continent_1_regional.png");
        // A second snapshot at the same radius is a newer picture of the same
        // place, not a second entry.
        s.set_snapshot("settlement:42", "local", ".cartalith/maps/settlement_42_local_v2.png");
        assert_eq!(s.snapshots.len(), 2);

        let back = LinkStore::from_json(&s.to_json()).unwrap();
        assert_eq!(back, s);
        assert_eq!(back.snapshot("settlement:42", "local"), Some(".cartalith/maps/settlement_42_local_v2.png"));
        assert_eq!(back.snapshot("settlement:42", "immediate"), None, "an ungenerated radius is absent, not empty");
        assert_eq!(back.snapshot("settlement:43", "local"), None);

        // An older store parses, and so does an empty one. The *installed
        // base* — a pre-snapshot document that actually has something in it —
        // is `LEGACY_STORE` and its own test below; two empty arrays would
        // pass whatever `from_json` did to the members.
        assert_eq!(LinkStore::from_json(r#"{"version":1,"vaults":[],"links":[]}"#).unwrap(), LinkStore::default());
    }

    /// A `vault.json` as a build before 2026-09-02 wrote it — `version`,
    /// `vaults`, `links`, and **no `snapshots` member**, because
    /// [`LinkStore::snapshots`] did not exist (`git show
    /// 2972689:…/links.rs`, where `LinkStore` is two fields).
    ///
    /// Verbatim `to_json` output, which is what makes it a fixture rather
    /// than a guess — `a_pre_snapshot_document_still_opens_and_still_resolves`
    /// re-derives it and asserts byte equality, so a change to the wire shape
    /// of any member fails there rather than silently rewriting the meaning of
    /// this string.
    const LEGACY_STORE: &str = r###"{
  "version": 1,
  "vaults": [
    {
      "id": "vault_f4dce017eb51232b",
      "display_name": "Elaris"
    }
  ],
  "links": [
    {
      "link_id": "link_7a79cb0395536f7e",
      "entity_kind": "settlement",
      "entity_id": 42,
      "entity_label": "Nareth",
      "vault_id": "vault_f4dce017eb51232b",
      "relative_path": "Locations/Nareth.md",
      "selection": {
        "type": "heading",
        "value": "The Old Quarter"
      },
      "source_modified": 1000,
      "source_hash": "aaaa",
      "imported_text": "## The Old Quarter\n\nNarrow streets.\n"
    }
  ]
}"###;

    /// Backward compatibility, measured on the installed base rather than on
    /// a new save round-tripped against itself.
    ///
    /// The three things a person with a pre-milestone-2 project cares about,
    /// in order: the document still opens; every link in it still resolves
    /// exactly what it resolved (id, entity, vault, selection, imported text,
    /// status); and opening it does **not** silently rewrite its shape — an
    /// archive with no snapshots must still be written back with no
    /// `snapshots` member, or every old project gains a diff on first open.
    #[test]
    fn a_pre_snapshot_document_still_opens_and_still_resolves() {
        let old = LinkStore::from_json(LEGACY_STORE).expect("a pre-snapshot vault.json still opens");

        // Still resolves what it resolved.
        assert_eq!(old.vault("vault_f4dce017eb51232b").map(|v| v.display_name.as_str()), Some("Elaris"));
        let found = old.links_for(EntityKind::Settlement, 42);
        assert_eq!(found.len(), 1, "the one link in the document");
        let l = found[0];
        assert_eq!(l.link_id, "link_7a79cb0395536f7e");
        assert_eq!(l.entity_key(), "settlement:42");
        assert_eq!(l.vault_id, "vault_f4dce017eb51232b");
        assert_eq!(l.relative_path, "Locations/Nareth.md");
        assert_eq!(l.selection, Selection::Heading { value: "The Old Quarter".into() });
        assert_eq!(l.working_text(), "## The Old Quarter\n\nNarrow streets.\n");
        assert!(!l.has_local_changes());
        assert!(old.get("link_7a79cb0395536f7e").is_some());
        assert_eq!(
            l.status(true, Some(FileMeta { modified: 1000, len: 36 }), Some("aaaa")),
            LinkStatus::Connected,
            "an unchanged source is still Connected under the new store"
        );

        // The new member is absent, not empty-valued: nothing to show, and
        // `snapshot()` says so with `None` rather than an empty path.
        assert!(old.snapshots.is_empty());
        assert_eq!(old.snapshot("settlement:42", "local"), None);
        // ...and the document is worth writing, so opening and saving does
        // not drop it (`is_empty` gates that write).
        assert!(!old.is_empty());

        // Byte-identical both ways: this build re-derives the fixture, so the
        // fixture is what an older build wrote; and re-saving an opened
        // legacy project adds no member to its `vault.json`.
        let mut rebuilt = LinkStore::default();
        let vid = rebuilt.add_vault("Elaris");
        assert_eq!(vid, "vault_f4dce017eb51232b");
        let mut l = link();
        l.vault_id = vid;
        rebuilt.attach(l);
        assert_eq!(rebuilt.to_json(), LEGACY_STORE, "the wire shape of a snapshot-less store has not moved");
        assert_eq!(old.to_json(), LEGACY_STORE, "opening a legacy store and saving it writes the same bytes");
        assert!(!old.to_json().contains("snapshots"));
    }

    /// The predicate `project_bridge.rs`'s `vault.json` gate calls, wired at
    /// `52666b9`.
    ///
    /// Each of the three members is asserted **alone**, so dropping any one
    /// conjunct from [`LinkStore::is_empty`] turns exactly one case red. The
    /// snapshot-only case is the live one: it is the state a person reaches
    /// by generating a map for a settlement they have not linked a note to,
    /// and the superseded gate (`!store.links.is_empty()`) called it nothing
    /// to write and dropped the snapshot on save.
    #[test]
    fn a_store_holding_only_a_snapshot_is_not_an_empty_store() {
        assert!(LinkStore::default().is_empty(), "nothing filed is empty");

        let mut snap_only = LinkStore::default();
        snap_only.set_snapshot("settlement:42", "local", ".cartalith/maps/settlement_42_local.png");
        assert!(snap_only.links.is_empty(), "the superseded gate read this store as empty");
        assert!(!snap_only.is_empty(), "and a generated map is not nothing");

        let mut vault_only = LinkStore::default();
        vault_only.add_vault("Elaris");
        assert!(!vault_only.is_empty(), "a connected vault is §26's markdownVault member");

        let mut link_only = LinkStore::default();
        link_only.attach(link());
        assert!(!link_only.is_empty());

        // And the snapshot-only store survives the document it would be
        // written as, which is the whole point of writing it: a project saved
        // in this state and reopened still knows where its map is.
        let back = LinkStore::from_json(&snap_only.to_json()).expect("re-reads");
        assert_eq!(
            back.snapshot("settlement:42", "local"),
            Some(".cartalith/maps/settlement_42_local.png")
        );
        assert!(!back.is_empty());
    }

    #[test]
    fn attaching_the_same_section_twice_replaces_rather_than_duplicates() {
        let mut s = LinkStore::default();
        let a = s.attach(link());
        let mut second = link();
        second.entity_label = "Nareth (renamed)".into();
        let b = s.attach(second);
        assert_eq!(a, b, "the id is a function of the link's own coordinates");
        assert_eq!(s.links.len(), 1);
        assert_eq!(s.links[0].entity_label, "Nareth (renamed)");
        assert!(s.detach(&a));
        assert!(!s.detach(&a));
    }

    #[test]
    fn a_different_section_of_the_same_file_is_a_different_link() {
        let mut s = LinkStore::default();
        s.attach(link());
        let mut other = link();
        other.selection = Selection::WholeDocument;
        s.attach(other);
        assert_eq!(s.links_for(EntityKind::Settlement, 42).len(), 2);
        assert_eq!(s.links_for(EntityKind::Province, 42).len(), 0);
    }

    #[test]
    fn status_covers_every_state_the_design_names() {
        let l = link();
        let same = FileMeta { modified: 1000, len: 40 };
        assert_eq!(l.status(false, Some(same), Some("aaaa")), LinkStatus::Unbound);
        assert_eq!(l.status(true, None, None), LinkStatus::Cached);
        let mut never_imported = link();
        never_imported.imported_text = None;
        assert_eq!(never_imported.status(true, None, None), LinkStatus::Missing);
        assert_eq!(l.status(true, Some(same), Some("aaaa")), LinkStatus::Connected);
        assert_eq!(l.status(true, Some(same), Some("bbbb")), LinkStatus::Stale);
        // A touched-but-identical file is Connected, not Stale -- the hash
        // outranks the timestamp when both are known.
        let touched = FileMeta { modified: 2000, len: 40 };
        assert_eq!(l.status(true, Some(touched), Some("aaaa")), LinkStatus::Connected);
        // With no hash on either side the timestamp decides.
        let mut no_hash = link();
        no_hash.source_hash = String::new();
        assert_eq!(no_hash.status(true, Some(touched), None), LinkStatus::Stale);

        let mut edited = link();
        edited.edited_text = Some("## The Old Quarter\n\nRebuilt.\n".into());
        assert!(edited.has_local_changes());
        assert_eq!(edited.status(true, Some(same), Some("aaaa")), LinkStatus::LocalChanges);
        assert_eq!(edited.status(true, Some(same), Some("bbbb")), LinkStatus::Stale, "a changed source outranks a local edit");
    }

    #[test]
    fn entity_keys_and_kinds_round_trip() {
        for k in [EntityKind::Settlement, EntityKind::Province, EntityKind::Continent] {
            assert_eq!(EntityKind::parse(k.as_str()), Some(k));
        }
        assert_eq!(EntityKind::parse("poi"), None, "POI is not a ported concept");
        assert_eq!(entity_key(EntityKind::Continent, 2), "continent:2");
    }
    /// `SAVEFILE_COMPAT.md` §13.3.6, and the reason it is a MUST.
    ///
    /// Before 2026-08-26 an unrecognised `selection.type` failed that link,
    /// which failed `from_json`, which made `project_open`'s `if let Ok(..)`
    /// skip the vault entirely. One selection shape from a newer writer cost
    /// **every link in the project**, silently, on open. KV-04's exact shape
    /// one layer up.
    #[test]
    fn an_unrecognised_selection_costs_neither_the_link_nor_the_store() {
        let json = r#"{"version":1,"vaults":[],"links":[
            {"link_id":"a","vault_id":"v","relative_path":"a.md","entity_kind":"settlement",
             "entity_id":1,"entity_label":"Nareth","selection":{"type":"whole_document"}},
            {"link_id":"b","vault_id":"v","relative_path":"b.md","entity_kind":"settlement",
             "entity_id":2,"entity_label":"Eldra","selection":{"type":"text_range","from":10,"to":20}},
            {"link_id":"c","vault_id":"v","relative_path":"c.md","entity_kind":"settlement",
             "entity_id":3,"entity_label":"Farsahspan","selection":{"type":"heading","value":"History"}}
        ]}"#;
        let store = LinkStore::from_json(json).expect("a newer writer's selection must not fail the store");
        assert_eq!(store.links.len(), 3, "every link survives, including the unrecognised one");
        // The unrecognised one is READ AS whole_document, per §13.3.6 -- not
        // dropped, and not guessed into a heading.
        assert_eq!(store.links[1].selection, Selection::WholeDocument);
        // The two this build does know are untouched.
        assert_eq!(store.links[0].selection, Selection::WholeDocument);
        assert_eq!(store.links[2].selection, Selection::Heading { value: "History".to_string() });

        // And what it writes back is what it understood -- no invented type
        // string escapes into a file another reader has to cope with.
        let round = LinkStore::from_json(&store.to_json()).expect("re-reads");
        assert_eq!(round.links.len(), 3);
        assert!(!store.to_json().contains("text_range"), "the shape we could not read is not echoed back");

        // A `heading` missing its value is a heading selecting nothing, which
        // resolves to nothing -- still not a reason to lose the link.
        let partial = r#"{"version":1,"vaults":[],"links":[
            {"link_id":"d","vault_id":"v","relative_path":"d.md","entity_kind":"faction",
             "entity_id":1,"entity_label":"House Vare","selection":{"type":"heading"}}]}"#;
        let store = LinkStore::from_json(partial).expect("still parses");
        assert_eq!(store.links[0].selection, Selection::Heading { value: String::new() });
    }
}

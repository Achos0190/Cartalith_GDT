//! Knowledge links (`MARKDOWN_VAULT_INTEGRATION.md` §11, §26, §27).
//!
//! A [`KnowledgeLink`] is the whole relationship between one Cartalith entity
//! and one Markdown document. §36's engineering principle — "neither side
//! should silently become the other" — is why this type holds *references*
//! and a working copy, never the vault's own content as world data.
//!
//! ## Where this is stored, and why not in the save
//!
//! §25 and §26 ask for a separate JSON layer, and this port has a second,
//! harder reason to obey that: `cartalith-io`'s save format is the reference
//! HTML app's own `.zip` (`SAVEFILE_COMPAT.md`), it carries **no civ data at
//! all**, and `WorldGen::load_save` documents that `get_settlements()` comes
//! back empty after a load. There is therefore no entity in a loaded save for
//! a link inside that save to point at. [`LinkStore`] is its own JSON
//! document beside the project instead — which is also what makes the vault
//! block in `DCC_SHELL_SPEC.md` §9 (note links written into exported GeoJSON
//! and tiles) something to leave alone rather than build.
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
//!
//! So every link also stores [`KnowledgeLink::entity_label`] — the entity's
//! name at link time. It is not the key and nothing resolves by it; it exists
//! so that when a key goes stale the UI can say *"this note was linked to
//! Nareth"* and let a person re-bind, which is §32's "stop and ask the user
//! rather than guessing" applied to identity rather than to content.

use serde::{Deserialize, Serialize};

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
}

impl EntityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EntityKind::Settlement => "settlement",
            EntityKind::Province => "province",
            EntityKind::Continent => "continent",
            EntityKind::Faction => "faction",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "settlement" => Some(EntityKind::Settlement),
            "province" => Some(EntityKind::Province),
            "continent" => Some(EntityKind::Continent),
            "faction" => Some(EntityKind::Faction),
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Selection {
    WholeDocument,
    Heading { value: String },
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
}

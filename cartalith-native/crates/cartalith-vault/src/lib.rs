//! Markdown Vault integration — the engine half.
//!
//! `MARKDOWN_VAULT_SCOPE.md` is this crate's scope document and
//! `MARKDOWN_VAULT_INTEGRATION.md` is the owner-supplied design it implements.
//! This is a **new feature, not a port**: nothing in `reference/Cartalith Gen1
//! v2.10.html` does any of it, so it sits outside `DECISIONS.md` §7d's parity
//! contract entirely and there is no golden fixture to match. What replaces
//! golden parity here is the round-trip discipline the tests enforce
//! throughout: *a write that changes nothing must produce a byte-identical
//! document, and a write that changes one section must leave every other byte
//! alone.*
//!
//! ## The four modules
//!
//! | Module | Owns |
//! |---|---|
//! | [`markdown`] | Section spans, section replacement, author-template field lines |
//! | [`block`] | The machine-owned `CARTALITH:BEGIN/END` block (§23, §24) |
//! | [`links`] | [`links::KnowledgeLink`], [`links::LinkStore`], the five status states (§11, §26, §27) |
//! | [`provider`] | The desktop filesystem vault (§6), path containment, atomic writes |
//! | [`export`] | The exportable-field registry and the block renderer (§19, §20) |
//!
//! [`VaultSession`] is the one type that puts them together, and the only one
//! `cartalith-godot`'s `vault_bridge.rs` needs to hold.
//!
//! ## The V1 promise, stated once
//!
//! §17: *"All writes to the Markdown Vault are explicit user actions… Reading
//! can be automatic/on-demand. Writing cannot."* Every read method on
//! [`VaultSession`] is safe to call from a panel rebuild. Every write method
//! takes an `expect_hash` argument, which the caller can only have obtained
//! from a preview of the current file — so a source that changed between the
//! preview and the confirmation cannot be overwritten (§16 step 3).
//!
//! ## Not Obsidian
//!
//! Per the owner's 2026-08-18 clarification: the target is a generic Markdown
//! vault. There is no `obsidian://` scheme here, no wikilink *generation*, no
//! block references, and no vault-config directory is read. Obsidian
//! constructs a user has written are preserved as opaque bytes, which is §10's
//! actual requirement.

pub mod backlinks;
pub mod block;
pub mod export;
pub mod links;
pub mod markdown;
pub mod provider;
pub mod template;

pub use backlinks::{Backlink, BacklinkIndex, LinkForm, RefreshStats, excerpt};
pub use block::{BlockAction, BlockError};
pub use links::{EntityKind, ImportedData, KnowledgeLink, LinkStatus, LinkStore, Selection, VaultRef};
pub use markdown::{FieldFill, FieldOutcome, Section, SectionError};
pub use provider::{FileMeta, FsVault, VaultError};
pub use template::Template;

/// Why an operation refused. Every variant maps to something §32 requires be
/// handled explicitly, and none of them has a destructive fallback.
#[derive(Debug)]
pub enum Error {
    /// No vault is bound on this device (§27 "Unbound").
    NotBound,
    NoSuchLink(String),
    Vault(VaultError),
    Section(SectionError),
    Block(BlockError),
    /// §16 step 3: the file on disk is not the file the preview was taken
    /// from. Carries both hashes so the UI can say so precisely.
    SourceChanged { expected: String, actual: String },
    /// The link has no imported or edited text to write back.
    NothingToWrite,
    /// `create_from_template` will not write over a note that already
    /// exists (`GUI_GAP_REGISTER.md` VA-02). Creating a file is safe
    /// precisely because it cannot destroy one.
    AlreadyExists(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotBound => write!(f, "no Markdown vault is connected on this device"),
            Error::NoSuchLink(id) => write!(f, "no such link: {id}"),
            Error::Vault(e) => write!(f, "{e}"),
            Error::Section(e) => write!(f, "{e}"),
            Error::Block(e) => write!(f, "{e}"),
            Error::SourceChanged { .. } => write!(
                f,
                "the source file changed since this preview was taken; re-open it and check before writing"
            ),
            Error::NothingToWrite => write!(f, "nothing has been imported for this link yet"),
            Error::AlreadyExists(p) => write!(f, "\"{p}\" already exists -- attach to it instead of creating it"),
        }
    }
}

impl std::error::Error for Error {}

impl From<VaultError> for Error {
    fn from(e: VaultError) -> Self {
        Error::Vault(e)
    }
}
impl From<SectionError> for Error {
    fn from(e: SectionError) -> Self {
        Error::Section(e)
    }
}
impl From<BlockError> for Error {
    fn from(e: BlockError) -> Self {
        Error::Block(e)
    }
}

/// What a field-fill preview carries: the document as it would be, the hash
/// of the source it was computed from, and one `(template field, outcome)`
/// row per field considered — which is the part the user has to see before
/// confirming, since "skipped, you had already filled it" is the answer as
/// often as "written".
pub type FieldFillPreview = (String, String, Vec<(String, FieldOutcome)>);

/// The bound on any single vault walk, shared by browsing, the backlink
/// refresh and [`VaultSession::search`] so §31's "do not load the entire
/// vault" is one number rather than three.
pub const LIST_CAP: usize = 2000;

// ---------------------------------------------------------------- searching

/// One note a search matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub rel: String,
    /// The query is in this note's **path**. A certain match, found without
    /// opening the file.
    pub in_name: bool,
    /// One line of context around the first occurrence in the body, or `""`
    /// for a name-only hit.
    pub excerpt: String,
}

/// What [`VaultSession::search`] found, and what it was able to look at.
///
/// `indexed` is the field a caller must not skip. Content search runs off the
/// backlink index's word fingerprints, so with no index there is no content
/// search — and *"nothing in the vault says this"* and *"I did not look"* are
/// opposite statements to put in front of a person. The same distinction
/// [`BacklinkIndex::is_built`] already forces on every other reader.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchResults {
    pub hits: Vec<SearchHit>,
    /// Whether the body of any note could be searched at all.
    pub indexed: bool,
    /// How many notes were actually opened. Zero for a pure name search.
    pub scanned: usize,
    /// `limit` or `max_reads` cut the answer short, so there may be more.
    pub truncated: bool,
}

/// Which confirmation dialogs the user has said *"don't ask me again"* to.
///
/// The owner's 2026-08-25 direction: *"it's an explicit action with a prompt
/// confirmation (the prompt should have an option to confirm always)"*.
///
/// ## What "always" may and may not skip
///
/// It suppresses **the dialog**, never the guard. Every write still takes an
/// `expect_hash` obtained from a preview computed moments earlier, so a note
/// edited between the two still refuses. Removing the prompt removes a
/// keystroke; it cannot remove the thing that stops Cartalith overwriting
/// somebody's evening's work. A caller with a preference set must therefore
/// still call the preview — it is where the hash comes from — and simply not
/// show it.
///
/// ## Three flags, not one
///
/// Replacing a section the user just typed, regenerating a machine-owned
/// block, and writing into the author's own template lines are three
/// different risks, and one switch that turned off all three would be a
/// worse offer than three that are each obviously scoped. `always_field_fill`
/// in particular sits against `MARKDOWN_VAULT_INTEGRATION.md`'s own header
/// (*"offered and explicitly confirmed, never silent"*); it is honoured
/// because the owner asked for it, and it is safe because
/// [`FieldFill::OnlyIfEmpty`] still refuses an occupied field whether anyone
/// is watching or not. The reconciliation is recorded in
/// `MARKDOWN_VAULT_SCOPE.md` milestone 6 rather than left implicit.
///
/// ## Device state, not project data
///
/// Kept off [`LinkStore`] deliberately. The store is portable project data
/// (§5) and one person's "stop asking me" must not travel into another
/// person's copy of a project. This follows the split
/// [`BacklinkIndex`] already established: its own JSON, its own file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WritePrefs {
    /// Skip the confirmation on *Insert updated section into source*.
    #[serde(default)]
    pub always_section: bool,
    /// Skip the confirmation on *Write Cartalith block*.
    #[serde(default)]
    pub always_block: bool,
    /// Skip the confirmation on *Fill the note's own fields*.
    #[serde(default)]
    pub always_field_fill: bool,
}

impl WritePrefs {
    /// The three names a UI passes: `"section"`, `"block"`, `"field_fill"`.
    /// An unknown name is `false` and setting one returns `false`, never a
    /// default — a typo must not silently disarm a confirmation.
    pub fn get(&self, path: &str) -> bool {
        match path {
            "section" => self.always_section,
            "block" => self.always_block,
            "field_fill" => self.always_field_fill,
            _ => false,
        }
    }

    pub fn set(&mut self, path: &str, value: bool) -> bool {
        match path {
            "section" => self.always_section = value,
            "block" => self.always_block = value,
            "field_fill" => self.always_field_fill = value,
            _ => return false,
        }
        true
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// An empty or blank document is the default (ask every time), which is
    /// the safe direction to fail in.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        if s.trim().is_empty() {
            return Ok(WritePrefs::default());
        }
        serde_json::from_str(s)
    }
}

/// A project's links plus, when this device has one, the binding that makes
/// them readable.
///
/// The split is §5's: [`LinkStore`] is portable project data, `binding` is a
/// device-local path this type never writes into the store. Opening a project
/// on a machine that has never seen the vault yields a session with links and
/// no binding — every link reports [`LinkStatus::Unbound`], every read
/// refuses, and the map still works (§27's requirement that Cartalith remain
/// usable without the vault).
pub struct VaultSession {
    pub store: LinkStore,
    binding: Option<FsVault>,
    /// `GUI_GAP_REGISTER.md` **VA-01**'s reverse index. Built only when asked
    /// for, invalidated per file by `(modified, len)`, and persisted beside
    /// the link store rather than inside it -- see
    /// [`backlinks`](crate::backlinks)' module doc for why that is not the
    /// "second store to keep in step" the register warned about.
    ///
    /// Empty until [`VaultSession::refresh_backlinks`] runs. Every reader
    /// below distinguishes "not built" from "nothing found", because on
    /// screen those are opposite statements.
    pub backlinks: BacklinkIndex,
    /// The user's *"don't ask me again"* choices. Device state — see
    /// [`WritePrefs`] for why it is not in `store`.
    pub prefs: WritePrefs,
}

impl Default for VaultSession {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultSession {
    pub fn new() -> Self {
        VaultSession {
            store: LinkStore::default(),
            binding: None,
            backlinks: BacklinkIndex::new(),
            prefs: WritePrefs::default(),
        }
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        Ok(VaultSession {
            store: LinkStore::from_json(s)?,
            binding: None,
            backlinks: BacklinkIndex::new(),
            prefs: WritePrefs::default(),
        })
    }

    pub fn to_json(&self) -> String {
        self.store.to_json()
    }

    /// Binds a directory on this device and registers the vault in the store,
    /// returning its `vaultId`. `display_name` defaults to the directory's
    /// own name, which is what a user calls their vault.
    pub fn connect(&mut self, root: &str, display_name: Option<&str>) -> Result<String, Error> {
        let vault = FsVault::new(root);
        if !vault.available() {
            return Err(Error::Vault(VaultError::RootUnavailable(vault.root().to_path_buf())));
        }
        let name = display_name
            .map(str::to_string)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| {
                vault
                    .root()
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "Vault".to_string())
            });
        let id = self.store.add_vault(&name);
        self.binding = Some(vault);
        Ok(id)
    }

    pub fn disconnect(&mut self) {
        self.binding = None;
    }

    pub fn is_bound(&self) -> bool {
        self.binding.as_ref().is_some_and(|v| v.available())
    }

    pub fn vault(&self) -> Option<&FsVault> {
        self.binding.as_ref()
    }

    fn bound(&self) -> Result<&FsVault, Error> {
        self.binding.as_ref().filter(|v| v.available()).ok_or(Error::NotBound)
    }

    /// Markdown files in the bound vault (§9, bounded — see
    /// [`FsVault::list_markdown`]).
    pub fn list(&self, limit: usize) -> Result<Vec<String>, Error> {
        Ok(self.bound()?.list_markdown(limit)?)
    }

    pub fn read(&self, rel: &str) -> Result<String, Error> {
        Ok(self.bound()?.read(rel)?)
    }

    // ---------------------------------------------------- backlinks (VA-01)

    /// Bring the backlink index up to date, reading only the notes whose
    /// `(modified, len)` moved since the last refresh.
    ///
    /// The only thing that ever writes the index. There is no watcher and no
    /// background pass — a person presses **Refresh**, and this is what that
    /// does.
    pub fn refresh_backlinks(&mut self, limit: usize) -> Result<RefreshStats, Error> {
        let vault = self.binding.as_ref().filter(|v| v.available()).ok_or(Error::NotBound)?;
        Ok(self.backlinks.refresh(vault, limit)?)
    }

    /// Throw the index away so the next refresh re-reads everything —
    /// **Rebuild**.
    pub fn rebuild_backlinks(&mut self) {
        self.backlinks.clear();
    }

    /// Every note that references this entity, by either route.
    ///
    /// Two routes, and the second is the one a note-to-note index alone would
    /// miss:
    ///
    /// 1. the entity's **own** linked notes (from [`LinkStore`]) are looked
    ///    up in the reverse index, giving every note that links to them;
    /// 2. every note carrying a `CARTALITH:BEGIN entity="settlement:42"`
    ///    block for this entity — which finds it even when the entity has no
    ///    note of its own, because a province's note can describe a
    ///    settlement nobody has written a page for.
    ///
    /// Returns `(relative_path, form, count)` with `form` `None` for an
    /// entity-block reference, which is not a link and should not be drawn as
    /// one. Sorted by path, deduplicated.
    pub fn entity_backlinks(
        &self,
        kind: EntityKind,
        id: i64,
    ) -> Vec<(String, Option<LinkForm>, usize)> {
        let mut by_path: std::collections::BTreeMap<String, (Option<LinkForm>, usize)> =
            Default::default();
        for l in self.store.links_for(kind, id) {
            for b in self.backlinks.backlinks_to(&l.relative_path) {
                let e = by_path.entry(b.source).or_insert((Some(b.form), 0));
                e.1 += b.count;
                if e.0.is_none() {
                    e.0 = Some(b.form);
                }
            }
        }
        for rel in self.backlinks.notes_referencing_entity(&links::entity_key(kind, id)) {
            by_path.entry(rel).or_insert((None, 1));
        }
        // The entity's own note is not a backlink to itself.
        for l in self.store.links_for(kind, id) {
            by_path.remove(&l.relative_path);
        }
        by_path.into_iter().map(|(k, (f, c))| (k, f, c)).collect()
    }

    /// Notes that name `name` in prose and do not link to this entity.
    ///
    /// The index narrows the vault to a handful of **candidates** by word
    /// fingerprint; this then opens only those and confirms with a real
    /// case-insensitive search, returning one excerpt per hit. A candidate
    /// that does not really contain the name is dropped silently — that is
    /// the Bloom filter's expected false positive and the reason the read
    /// exists.
    ///
    /// `max` bounds the reads, so even a name that filters badly cannot open
    /// the whole vault. Returns `(relative_path, excerpt)`.
    pub fn entity_mentions(
        &self,
        kind: EntityKind,
        id: i64,
        name: &str,
        max: usize,
    ) -> Vec<(String, String)> {
        if name.trim().len() < 3 || !self.backlinks.is_built() {
            return Vec::new();
        }
        // Everything already linked is excluded before any file is opened.
        let mut exclude: Vec<String> =
            self.store.links_for(kind, id).into_iter().map(|l| l.relative_path.clone()).collect();
        for (rel, _, _) in self.entity_backlinks(kind, id) {
            exclude.push(rel);
        }
        let needle = name.to_lowercase();
        let mut out = Vec::new();
        for rel in self.backlinks.mention_candidates(name, &exclude) {
            if out.len() >= max {
                break;
            }
            let Ok(text) = self.read(&rel) else { continue };
            let hay = text.to_lowercase();
            let Some(at) = hay.find(&needle) else { continue };
            out.push((rel, excerpt(&text, at, needle.len())));
        }
        out
    }

    // ------------------------------------------------------------ searching

    /// Find notes by name and, where the index allows it, by content.
    ///
    /// The owner's 2026-08-25 direction — *"Search for it in the vault"* —
    /// and the piece milestone 1 genuinely did not have: browsing listed
    /// every note, [`Self::headings`] listed one note's headings, and
    /// [`Self::entity_mentions`] searched **for one entity's own name**.
    /// Nothing let a person type a word.
    ///
    /// ## Two halves, and they are not equally certain
    ///
    /// 1. **Name.** A case-insensitive substring of the vault-relative path.
    ///    Certain, and costs no file read at all — it filters the same
    ///    bounded listing the file picker already uses.
    /// 2. **Content.** Narrowed by the backlink index's word fingerprints to
    ///    a handful of candidates, then confirmed by actually opening at most
    ///    `max_reads` of them. False positives from the Bloom filter are
    ///    dropped silently; false negatives are impossible, which is the
    ///    property that makes this a search rather than a suggestion.
    ///
    /// Content search **requires the index**. With no index
    /// [`SearchResults::indexed`] is `false` and the caller must say so
    /// rather than report an empty result as an absence — the alternative is
    /// opening every note in the vault on every keystroke, which is precisely
    /// what §31 forbids and what the index exists to avoid.
    ///
    /// A query under three characters is name-only for the same reason
    /// [`Self::entity_mentions`] declines one: every note would be a
    /// candidate, so confirming would mean reading the whole vault.
    ///
    /// Name hits come first and each note appears once.
    pub fn search(&self, query: &str, limit: usize, max_reads: usize) -> Result<SearchResults, Error> {
        let files = self.list(LIST_CAP)?;
        let q = query.trim();
        let mut out = SearchResults { indexed: self.backlinks.is_built(), ..Default::default() };
        if q.is_empty() || limit == 0 {
            return Ok(out);
        }
        let needle = q.to_lowercase();

        let mut named: Vec<String> = Vec::new();
        for rel in files {
            if rel.to_lowercase().contains(&needle) {
                named.push(rel);
            }
        }
        for rel in named.iter() {
            if out.hits.len() >= limit {
                out.truncated = true;
                return Ok(out);
            }
            out.hits.push(SearchHit { rel: rel.clone(), in_name: true, excerpt: String::new() });
        }

        if !out.indexed || q.chars().count() < 3 {
            return Ok(out);
        }
        for rel in self.backlinks.mention_candidates(q, &named) {
            if out.hits.len() >= limit || out.scanned >= max_reads {
                out.truncated = true;
                break;
            }
            let Ok(text) = self.read(&rel) else { continue };
            out.scanned += 1;
            let Some(at) = text.to_lowercase().find(&needle) else { continue };
            out.hits.push(SearchHit { rel, in_name: false, excerpt: excerpt(&text, at, needle.len()) });
        }
        Ok(out)
    }

    // -------------------------------------- the note's information, as data

    /// One note's frontmatter and template fields, read now and stored
    /// nowhere — what a search result or an attach dialog shows before
    /// anything is committed to.
    pub fn document_data(&self, rel: &str) -> Result<ImportedData, Error> {
        Ok(ImportedData::from_document(&self.read(rel)?))
    }

    /// Everything this entity's linked notes said about it, as
    /// `(relative_path, origin, key, value)`.
    ///
    /// **This is the owner's sentence, read back.** The user attached a note
    /// to a settlement; its information was copied into Cartalith's JSON at
    /// that moment; this is where it comes out. It reads the *copy*, never
    /// the disk, so it answers with the vault disconnected — which is §27's
    /// whole point and the reason copying was worth doing.
    ///
    /// Deliberately **not** deduplicated or merged across notes. Two notes on
    /// one settlement may disagree about its population, and picking a winner
    /// is a guess §32 forbids; `relative_path` is carried on every row so the
    /// disagreement is visible and attributable instead.
    pub fn entity_data(&self, kind: EntityKind, id: i64) -> Vec<(String, &'static str, String, String)> {
        let mut out = Vec::new();
        for l in self.store.links_for(kind, id) {
            for (origin, key, value) in l.imported_data.rows() {
                out.push((l.relative_path.clone(), origin, key.to_string(), value.to_string()));
            }
        }
        out
    }

    /// The templates in the bound vault (`GUI_GAP_REGISTER.md` **VA-02**),
    /// filtered out of the same bounded listing the file picker uses -- no
    /// second walk, and still no file opened.
    pub fn templates(&self, limit: usize) -> Result<Vec<template::Template>, Error> {
        Ok(template::discover(&self.list(limit)?))
    }

    /// Creates `rel` from the template at `template_rel`, with `name`
    /// substituted for the template's own name placeholders and **nothing
    /// else touched** (`template::fill_title`).
    ///
    /// Refuses rather than overwrites: an existing `rel` is
    /// [`Error::AlreadyExists`], because the one thing that makes creating a
    /// note safe -- unlike editing one -- is that it cannot destroy an
    /// author's work. Returns the text written, so the caller can show it.
    ///
    /// Deliberately does **not** attach the new note. Attaching is a
    /// separate, already-previewed act with its own validation, and folding
    /// it in here would make one button do two writes.
    pub fn create_from_template(&self, template_rel: &str, rel: &str, name: &str) -> Result<String, Error> {
        let v = self.bound()?;
        if v.exists(rel) {
            return Err(Error::AlreadyExists(rel.to_string()));
        }
        let body = template::fill_title(&v.read(template_rel)?, name);
        v.write(rel, &body)?;
        Ok(body)
    }

    /// The heading titles in one file, for the attach dialog's section list.
    pub fn headings(&self, rel: &str) -> Result<Vec<(u8, String)>, Error> {
        let text = self.read(rel)?;
        Ok(markdown::sections(&text).into_iter().map(|s| (s.level, s.title)).collect())
    }

    /// This link's state right now (§27). Never fails: an unreadable file is
    /// a *status*, not an error, because this is called on every panel
    /// rebuild.
    pub fn status(&self, link_id: &str) -> LinkStatus {
        let Some(l) = self.store.get(link_id) else { return LinkStatus::Missing };
        let Some(v) = self.binding.as_ref().filter(|v| v.available()) else {
            return l.status(false, None, None);
        };
        let meta = v.meta(&l.relative_path).ok();
        let hash = v.read(&l.relative_path).ok().map(|t| provider::content_hash(&t));
        l.status(true, meta, hash.as_deref())
    }

    /// Links an entity to a document (or one section of it), importing the
    /// text and recording the source's timestamp and hash.
    ///
    /// The selection is validated before the link is created: attaching a
    /// section that does not exist, or whose title is duplicated in the file,
    /// fails here rather than becoming a link that can never be read.
    ///
    /// The note's **structured** information is copied at the same moment,
    /// from the same bytes ([`ImportedData`]) — the owner's 2026-08-25
    /// direction. It is scoped to the whole document even for a heading
    /// selection, and its freshness is this link's own `source_hash`, so
    /// there is one staleness idea here and not two.
    pub fn attach(
        &mut self,
        kind: EntityKind,
        entity_id: i64,
        entity_label: &str,
        rel: &str,
        selection: Selection,
    ) -> Result<String, Error> {
        let v = self.bound()?;
        let text = v.read(rel)?;
        let imported = match &selection {
            Selection::WholeDocument => text.clone(),
            Selection::Heading { value } => markdown::section_text(&text, value)?,
        };
        let meta = v.meta(rel)?;
        let vault_id = self.store.vaults.first().map(|x| x.id.clone()).unwrap_or_default();
        Ok(self.store.attach(KnowledgeLink {
            link_id: String::new(),
            entity_kind: kind,
            entity_id,
            entity_label: entity_label.to_string(),
            vault_id,
            relative_path: rel.to_string(),
            selection,
            source_modified: meta.modified,
            source_hash: provider::content_hash(&text),
            imported_data: ImportedData::from_document(&text),
            imported_text: Some(imported),
            edited_text: None,
        }))
    }

    /// §14's "Reload source": re-reads the selection, replacing the imported
    /// text and dropping any local edit. Destructive to the *working copy*
    /// only, which is why the UI offers it beside "Keep current copy".
    ///
    /// **The one path that refreshes the copied [`ImportedData`]**, and
    /// deliberately so: text, data and hash move together or not at all. A
    /// refresh-the-data-only call would let the fields be current while the
    /// prose was not, under a status that could only describe one of them.
    /// It is also what fills the copy on a link made before 2026-08-25.
    pub fn reload(&mut self, link_id: &str) -> Result<(), Error> {
        let v = self.binding.as_ref().filter(|x| x.available()).ok_or(Error::NotBound)?;
        let l = self.store.get(link_id).ok_or_else(|| Error::NoSuchLink(link_id.into()))?;
        let text = v.read(&l.relative_path)?;
        let imported = match &l.selection {
            Selection::WholeDocument => text.clone(),
            Selection::Heading { value } => markdown::section_text(&text, value)?,
        };
        let meta = v.meta(&l.relative_path)?;
        let hash = provider::content_hash(&text);
        let data = ImportedData::from_document(&text);
        let l = self.store.get_mut(link_id).expect("looked up above");
        l.imported_text = Some(imported);
        l.imported_data = data;
        l.edited_text = None;
        l.source_modified = meta.modified;
        l.source_hash = hash;
        Ok(())
    }

    /// Records the Cartalith-side working copy (§15). Setting it back to the
    /// imported text clears the divergence rather than storing a redundant
    /// copy.
    pub fn set_working_text(&mut self, link_id: &str, text: &str) -> Result<(), Error> {
        let l = self.store.get_mut(link_id).ok_or_else(|| Error::NoSuchLink(link_id.into()))?;
        l.edited_text = if l.imported_text.as_deref() == Some(text) { None } else { Some(text.to_string()) };
        Ok(())
    }

    /// The document as it *would* be after [`Self::write_section`], plus the
    /// hash of the source it was computed from.
    ///
    /// §16 steps 1-4 in one call. The hash comes back so the caller can hand
    /// it to the write as `expect_hash` — that pairing is what makes the
    /// preview and the write see the same file.
    pub fn preview_section_write(&self, link_id: &str) -> Result<(String, String), Error> {
        let v = self.bound()?;
        let l = self.store.get(link_id).ok_or_else(|| Error::NoSuchLink(link_id.into()))?;
        let working = l.working_text();
        if working.is_empty() {
            return Err(Error::NothingToWrite);
        }
        let text = v.read(&l.relative_path)?;
        let hash = provider::content_hash(&text);
        let next = match &l.selection {
            Selection::WholeDocument => working.to_string(),
            Selection::Heading { value } => markdown::replace_section(&text, value, working)?,
        };
        Ok((next, hash))
    }

    /// §15's "Insert Updated Text into Source" — the only V1 path that writes
    /// an edited section back.
    ///
    /// `expect_hash` must be the hash [`Self::preview_section_write`]
    /// returned. If the file changed in between, this refuses with
    /// [`Error::SourceChanged`] and writes nothing (§16: *"If the source
    /// changed in the meantime, Cartalith must not blindly overwrite it."*).
    ///
    /// On success the link's recorded timestamp and hash are refreshed and
    /// the working copy becomes the imported text, so the link is
    /// [`LinkStatus::Connected`] immediately after rather than reporting the
    /// change it just made as someone else's.
    pub fn write_section(&mut self, link_id: &str, expect_hash: &str) -> Result<(), Error> {
        let (next, actual) = self.preview_section_write(link_id)?;
        if actual != expect_hash {
            return Err(Error::SourceChanged { expected: expect_hash.to_string(), actual });
        }
        let v = self.bound()?;
        let l = self.store.get(link_id).ok_or_else(|| Error::NoSuchLink(link_id.into()))?;
        let rel = l.relative_path.clone();
        v.write(&rel, &next)?;
        let meta = v.meta(&rel)?;
        let new_hash = provider::content_hash(&next);
        // Re-read the copy from the document that was just written. This is
        // the one write path that re-syncs the link's own hash, so it is the
        // one path that could otherwise leave a *stale copy under a
        // Connected status* — an edited section carrying a `**Population:**`
        // line is exactly that case.
        let new_data = ImportedData::from_document(&next);
        let l = self.store.get_mut(link_id).expect("looked up above");
        l.imported_data = new_data;
        // A rename *inside the edited text* moves the link with it (§32
        // "heading renamed"). The alternative is a link that was valid a
        // second ago and is broken by the user's own successful write —
        // Cartalith would then refuse to read a section it had just written.
        // Read from the working copy rather than from `next`, because `next`
        // is the whole document and its first heading is the document title.
        let renamed = match &l.selection {
            Selection::Heading { value } => markdown::sections(l.working_text())
                .into_iter()
                .next()
                .map(|s| s.title)
                .filter(|t| t != value),
            Selection::WholeDocument => None,
        };
        if let Some(title) = renamed {
            l.selection = Selection::Heading { value: title };
        }
        l.imported_text = Some(l.working_text().to_string());
        l.edited_text = None;
        l.source_modified = meta.modified;
        l.source_hash = new_hash;
        Ok(())
    }

    /// The document as it would be after [`Self::write_block`], plus the hash
    /// of the source and what the write would do (§23 rule 5, §24).
    pub fn preview_block_write(&self, rel: &str, entity_key: &str, body: &str) -> Result<(String, String, BlockAction), Error> {
        let v = self.bound()?;
        let text = if v.exists(rel) { v.read(rel)? } else { String::new() };
        let hash = provider::content_hash(&text);
        let (next, action) = block::upsert(&text, entity_key, body)?;
        Ok((next, hash, action))
    }

    /// Writes the machine-owned Cartalith block, creating the file if it does
    /// not exist. Same `expect_hash` contract as [`Self::write_section`].
    pub fn write_block(&mut self, rel: &str, entity_key: &str, body: &str, expect_hash: &str) -> Result<BlockAction, Error> {
        let (next, actual, action) = self.preview_block_write(rel, entity_key, body)?;
        if actual != expect_hash {
            return Err(Error::SourceChanged { expected: expect_hash.to_string(), actual });
        }
        self.bound()?.write(rel, &next)?;
        Ok(action)
    }

    /// Removes an entity's block from a document (§32's "stale Cartalith
    /// block"), leaving every other byte alone.
    pub fn remove_block(&mut self, rel: &str, entity_key: &str, expect_hash: &str) -> Result<bool, Error> {
        let v = self.bound()?;
        let text = v.read(rel)?;
        let actual = provider::content_hash(&text);
        if actual != expect_hash {
            return Err(Error::SourceChanged { expected: expect_hash.to_string(), actual });
        }
        let (next, removed) = block::remove(&text, entity_key)?;
        if removed {
            v.write(rel, &next)?;
        }
        Ok(removed)
    }

    /// Fills author-owned template fields (the owner's 2026-08-18 amendment
    /// to §23), reporting what each one did.
    ///
    /// `values` is keyed by [`export`] field key; [`export::AUTHOR_FIELDS`]
    /// maps those onto the template's own field names. Under
    /// [`FieldFill::OnlyIfEmpty`] an author-filled field is skipped, never
    /// overwritten — the constraint the amendment came with.
    pub fn preview_field_fill(
        &self,
        rel: &str,
        values: &dyn Fn(&str) -> Option<String>,
        policy: FieldFill,
    ) -> Result<FieldFillPreview, Error> {
        let v = self.bound()?;
        let text = v.read(rel)?;
        let hash = provider::content_hash(&text);
        let mut next = text;
        let mut report = Vec::new();
        for (key, author_name) in export::AUTHOR_FIELDS {
            let Some(value) = values(key).filter(|s| !s.trim().is_empty()) else { continue };
            let (t, outcome) = markdown::fill_field(&next, author_name, &value, policy);
            next = t;
            report.push(((*author_name).to_string(), outcome));
        }
        Ok((next, hash, report))
    }

    pub fn write_field_fill(
        &mut self,
        rel: &str,
        values: &dyn Fn(&str) -> Option<String>,
        policy: FieldFill,
        expect_hash: &str,
    ) -> Result<Vec<(String, FieldOutcome)>, Error> {
        let (next, actual, report) = self.preview_field_fill(rel, values, policy)?;
        if actual != expect_hash {
            return Err(Error::SourceChanged { expected: expect_hash.to_string(), actual });
        }
        self.bound()?.write(rel, &next)?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole point of the subsystem, as one test: a hand-authored note
    /// survives every write Cartalith can make to it.
    const HAND: &str = "---\ntags: [worldbuilding]\n---\n\n# Nareth\n\nA river town at the third ford. The author wrote this sentence.\n\n## History\n\nFounded in the third age by the Ashfall clans.\n\n## The Old Quarter\n\nNarrow streets, older than the walls.\n\n## Trade\n\nGrain downriver, salt up.\n";

    fn scratch(tag: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("cartalith-vault-it-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(p.join("Locations")).unwrap();
        std::fs::write(p.join("Locations/Nareth.md"), HAND).unwrap();
        p
    }

    /// `GUI_GAP_REGISTER.md` **VA-01**, end to end against a real folder:
    /// a settlement's note gains a backlink from a note that links to it, a
    /// backlink from a note that carries its *entity block* and nothing else,
    /// and an unlinked mention from a note that names it in prose -- with the
    /// three kept apart, because on screen they mean different things.
    #[test]
    fn an_entity_finds_its_incoming_notes_by_link_by_block_and_by_mention() {
        let root = scratch("backlinks");
        std::fs::write(root.join("Chronicle.md"), "The lords of [[Nareth]] held the ford.
").unwrap();
        std::fs::write(
            root.join("Province.md"),
            format!(
                "# Vale

{} entity=\"settlement:42\" version=\"1\" -->
rows
{}
",
                block::BEGIN_PREFIX,
                block::END_MARKER
            ),
        )
        .unwrap();
        std::fs::write(root.join("Journal.md"), "Rode to Nareth before the thaw and slept badly.
")
            .unwrap();
        std::fs::write(root.join("Elsewhere.md"), "Nothing to do with any of it.
").unwrap();

        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), None).unwrap();
        let link_id = s
            .attach(EntityKind::Settlement, 42, "Nareth", "Locations/Nareth.md",
                Selection::WholeDocument)
            .unwrap();
        assert!(!link_id.is_empty());

        // Nothing is built until somebody asks.
        assert!(!s.backlinks.is_built());
        assert!(s.entity_backlinks(EntityKind::Settlement, 42).is_empty());
        assert!(s.entity_mentions(EntityKind::Settlement, 42, "Nareth", 8).is_empty());

        let stats = s.refresh_backlinks(500).unwrap();
        assert_eq!(stats.seen, 5, "five notes in the folder");
        assert_eq!(stats.reread, 5, "a first build reads them all");

        let back = s.entity_backlinks(EntityKind::Settlement, 42);
        let paths: Vec<&str> = back.iter().map(|(p, _, _)| p.as_str()).collect();
        assert!(paths.contains(&"Chronicle.md"), "the wikilink is a backlink: {paths:?}");
        assert!(paths.contains(&"Province.md"), "the entity block is a backlink: {paths:?}");
        assert!(!paths.contains(&"Locations/Nareth.md"), "a note is not a backlink to itself");
        assert!(!paths.contains(&"Journal.md"), "a bare mention is not a backlink");
        // and the two are distinguishable: a block reference carries no form
        let chron = back.iter().find(|(p, _, _)| p == "Chronicle.md").unwrap();
        assert_eq!(chron.1, Some(LinkForm::Wiki));
        let prov = back.iter().find(|(p, _, _)| p == "Province.md").unwrap();
        assert_eq!(prov.1, None, "an entity block is not a link and must not be drawn as one");

        let mentions = s.entity_mentions(EntityKind::Settlement, 42, "Nareth", 8);
        let mpaths: Vec<&str> = mentions.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(mpaths, vec!["Journal.md"], "only the unlinked prose mention");
        assert!(
            mentions[0].1.contains("Nareth"),
            "the excerpt must show the hit: {:?}",
            mentions[0].1
        );

        // A second refresh over an untouched folder opens nothing.
        assert_eq!(s.refresh_backlinks(500).unwrap().reread, 0);

        let _ = std::fs::remove_dir_all(&root);
    }

    /// `GUI_GAP_REGISTER.md` **VA-02**, end to end against a real folder:
    /// a template is found, a note is created from it at v3's own path, the
    /// author's prompts survive, an existing note is refused, and the new
    /// note is attachable by the ordinary path -- which is the proof that
    /// this creates a *real* note rather than a special one.
    #[test]
    fn a_note_is_created_from_a_template_and_never_over_one() {
        let root = scratch("template");
        std::fs::write(
            root.join("Settlement Template.md"),
            "## Settlement Profile: [Name]

**Former Names:** [If applicable]

### History

[Key events.]
",
        )
        .unwrap();
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), None).unwrap();

        let ts = s.templates(100).unwrap();
        assert_eq!(ts.len(), 1, "the hand-authored note is not a template");
        assert_eq!(ts[0].rel, "Settlement Template.md");

        let rel = template::suggested_path(EntityKind::Settlement, "Kel Var");
        assert_eq!(rel, "Settlements/Kel Var.md");
        let body = s.create_from_template(&ts[0].rel, &rel, "Kel Var").unwrap();
        assert!(body.starts_with("## Settlement Profile: Kel Var"));
        assert!(body.contains("[If applicable]"), "the author's own prompt is untouched");
        assert_eq!(s.read(&rel).unwrap(), body, "what was returned is what is on disk");

        // Refused, not overwritten -- and the file is byte-identical after.
        assert!(matches!(s.create_from_template(&ts[0].rel, &rel, "Someone Else"), Err(Error::AlreadyExists(_))));
        assert_eq!(s.read(&rel).unwrap(), body);
        // The template itself is untouched too.
        assert!(s.read("Settlement Template.md").unwrap().contains("[Name]"));

        // An ordinary attach works on it, heading and all.
        let id = s
            .attach(EntityKind::Settlement, 7, "Kel Var", &rel, Selection::Heading { value: "History".into() })
            .unwrap();
        assert_eq!(s.status(&id), LinkStatus::Connected);
        assert!(s.store.get(&id).unwrap().working_text().contains("[Key events.]"));

        // And a faction, the kind CV-22 added, goes to its own folder.
        let frel = template::suggested_path(EntityKind::Faction, "Draumr League");
        s.create_from_template(&ts[0].rel, &frel, "Draumr League").unwrap();
        assert!(s.read(&frel).unwrap().contains("Draumr League"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn creating_a_note_needs_a_bound_vault() {
        let s = VaultSession::new();
        assert!(matches!(s.create_from_template("T.md", "Settlements/X.md", "X"), Err(Error::NotBound)));
        assert!(matches!(s.templates(10), Err(Error::NotBound)));
    }

    #[test]
    fn attach_edit_and_write_back_one_section_leaves_the_rest_of_the_note_alone() {
        let root = scratch("section");
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), Some("Elaris")).unwrap();
        assert!(s.is_bound());
        assert_eq!(s.list(100).unwrap(), ["Locations/Nareth.md"]);

        let id = s
            .attach(
                EntityKind::Settlement,
                42,
                "Nareth",
                "Locations/Nareth.md",
                Selection::Heading { value: "The Old Quarter".into() },
            )
            .unwrap();
        assert_eq!(s.status(&id), LinkStatus::Connected);
        assert!(s.store.get(&id).unwrap().working_text().contains("Narrow streets"));

        s.set_working_text(&id, "## The Old Quarter\n\nRebuilt after the fire of 812.\n").unwrap();
        assert_eq!(s.status(&id), LinkStatus::LocalChanges);

        let (preview, hash) = s.preview_section_write(&id).unwrap();
        assert!(preview.contains("Rebuilt after the fire of 812."));
        s.write_section(&id, &hash).unwrap();
        assert_eq!(s.status(&id), LinkStatus::Connected, "the write is not reported as someone else's change");

        let on_disk = std::fs::read_to_string(root.join("Locations/Nareth.md")).unwrap();
        assert_eq!(on_disk, preview, "what was previewed is what landed");
        assert!(on_disk.contains("Rebuilt after the fire of 812."));
        assert!(!on_disk.contains("Narrow streets"));
        for kept in [
            "tags: [worldbuilding]",
            "A river town at the third ford. The author wrote this sentence.",
            "Founded in the third age by the Ashfall clans.",
            "Grain downriver, salt up.",
        ] {
            assert!(on_disk.contains(kept), "hand-authored content survived: {kept}");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_source_that_changed_since_the_preview_is_not_overwritten() {
        let root = scratch("conflict");
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), None).unwrap();
        let id = s
            .attach(EntityKind::Settlement, 42, "Nareth", "Locations/Nareth.md", Selection::Heading { value: "History".into() })
            .unwrap();
        s.set_working_text(&id, "## History\n\nMine.\n").unwrap();
        let (_, hash) = s.preview_section_write(&id).unwrap();

        // The author edits the note in their own editor, in between.
        let theirs = HAND.replace("Grain downriver, salt up.", "Grain downriver, salt up, and wool.");
        std::fs::write(root.join("Locations/Nareth.md"), &theirs).unwrap();
        assert_eq!(s.status(&id), LinkStatus::Stale);

        assert!(matches!(s.write_section(&id, &hash), Err(Error::SourceChanged { .. })));
        assert_eq!(std::fs::read_to_string(root.join("Locations/Nareth.md")).unwrap(), theirs, "not one byte written");

        // Re-previewing against the new file succeeds, and still only
        // touches the section.
        let (_, hash2) = s.preview_section_write(&id).unwrap();
        s.write_section(&id, &hash2).unwrap();
        let after = std::fs::read_to_string(root.join("Locations/Nareth.md")).unwrap();
        assert!(after.contains("Mine."));
        assert!(after.contains("and wool."), "the author's concurrent edit survived");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// §32's "heading renamed", from the one direction V1 can actually
    /// control: the user renames the heading *in Cartalith* and writes back.
    /// The link has to follow, or Cartalith immediately refuses to read a
    /// section it just wrote itself.
    #[test]
    fn renaming_a_heading_in_the_working_copy_moves_the_link_with_it() {
        let root = scratch("rename");
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), None).unwrap();
        let id = s
            .attach(EntityKind::Settlement, 7, "Nareth", "Locations/Nareth.md", Selection::Heading { value: "History".into() })
            .unwrap();
        s.set_working_text(&id, "## Chronicle\n\nRewritten.\n").unwrap();
        let (_, hash) = s.preview_section_write(&id).unwrap();
        s.write_section(&id, &hash).unwrap();

        assert_eq!(s.store.get(&id).unwrap().selection, Selection::Heading { value: "Chronicle".into() });
        assert_eq!(s.status(&id), LinkStatus::Connected);
        // And the link still reads: a reload finds the renamed section.
        s.reload(&id).unwrap();
        assert!(s.store.get(&id).unwrap().working_text().contains("Rewritten."));
        let on_disk = std::fs::read_to_string(root.join("Locations/Nareth.md")).unwrap();
        assert!(on_disk.contains("## Chronicle"));
        assert!(!on_disk.contains("## History"));
        assert!(on_disk.contains("Narrow streets, older than the walls."), "the neighbouring section is untouched");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn reload_discards_the_working_copy_and_re_syncs() {
        let root = scratch("reload");
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), None).unwrap();
        let id = s
            .attach(EntityKind::Province, 3, "Lower Nareth", "Locations/Nareth.md", Selection::WholeDocument)
            .unwrap();
        s.set_working_text(&id, "clobbered").unwrap();
        assert!(s.store.get(&id).unwrap().has_local_changes());
        s.reload(&id).unwrap();
        assert!(!s.store.get(&id).unwrap().has_local_changes());
        assert_eq!(s.store.get(&id).unwrap().working_text(), HAND);
        assert_eq!(s.status(&id), LinkStatus::Connected);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_cartalith_block_is_written_updated_and_removed_without_disturbing_the_note() {
        let root = scratch("block");
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), None).unwrap();
        let selected: Vec<String> = ["name", "settlement_type", "population"].iter().map(|x| x.to_string()).collect();
        let body = export::render_body("Cartalith", &selected, &|k| match k {
            "name" => Some("Nareth".into()),
            "settlement_type" => Some("Town".into()),
            "population" => Some("8,420".into()),
            _ => None,
        });

        let (_, hash, action) = s.preview_block_write("Locations/Nareth.md", "settlement:42", &body).unwrap();
        assert!(matches!(action, BlockAction::Inserted(_)));
        s.write_block("Locations/Nareth.md", "settlement:42", &body, &hash).unwrap();
        let v1 = std::fs::read_to_string(root.join("Locations/Nareth.md")).unwrap();
        assert!(v1.contains("- Population: 8,420"));
        assert!(v1.contains("Founded in the third age by the Ashfall clans."));

        let body2 = body.replace("8,420", "9,001");
        let (_, hash2, action2) = s.preview_block_write("Locations/Nareth.md", "settlement:42", &body2).unwrap();
        assert!(matches!(action2, BlockAction::Replaced(_)));
        s.write_block("Locations/Nareth.md", "settlement:42", &body2, &hash2).unwrap();
        let v2 = std::fs::read_to_string(root.join("Locations/Nareth.md")).unwrap();
        assert!(v2.contains("9,001") && !v2.contains("8,420"));
        assert!(v2.contains("Narrow streets, older than the walls."));

        let h3 = provider::content_hash(&v2);
        assert!(s.remove_block("Locations/Nareth.md", "settlement:42", &h3).unwrap());
        assert_eq!(std::fs::read_to_string(root.join("Locations/Nareth.md")).unwrap(), HAND, "the note is exactly what it was");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn author_field_population_is_offered_and_never_silent() {
        let root = scratch("fields");
        std::fs::write(
            root.join("Locations/Template.md"),
            "## Settlement Profile: [Name]\n\n**Type:** [City / Town]  \n**Location:** Riverbend  \n\n- **Size / Population:**\n",
        )
        .unwrap();
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), None).unwrap();
        let values = |k: &str| match k {
            "population" => Some("8,420".to_string()),
            "settlement_type" => Some("Town".to_string()),
            "region" => Some("Lower Nareth".to_string()),
            _ => None,
        };
        let (preview, hash, report) = s.preview_field_fill("Locations/Template.md", &values, FieldFill::OnlyIfEmpty).unwrap();
        let by_name: std::collections::BTreeMap<&str, FieldOutcome> =
            report.iter().map(|(n, o)| (n.as_str(), *o)).collect();
        assert_eq!(by_name["Size / Population"], FieldOutcome::Written);
        assert_eq!(by_name["Type"], FieldOutcome::Written, "a [placeholder] counts as empty");
        assert_eq!(by_name["Location"], FieldOutcome::SkippedOccupied, "the author already filled it");
        assert!(preview.contains("**Location:** Riverbend"), "and it is untouched");

        s.write_field_fill("Locations/Template.md", &values, FieldFill::OnlyIfEmpty, &hash).unwrap();
        let on_disk = std::fs::read_to_string(root.join("Locations/Template.md")).unwrap();
        assert_eq!(on_disk, preview);
        assert!(on_disk.contains("- **Size / Population:** 8,420"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_project_opens_and_reports_honestly_with_no_vault_bound() {
        let root = scratch("unbound");
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), Some("Elaris")).unwrap();
        let id = s
            .attach(EntityKind::Continent, 1, "Vantharis", "Locations/Nareth.md", Selection::WholeDocument)
            .unwrap();
        let json = s.to_json();

        // Another device: the project's links, none of its paths.
        let elsewhere = VaultSession::from_json(&json).unwrap();
        assert!(!elsewhere.is_bound());
        assert_eq!(elsewhere.status(&id), LinkStatus::Unbound);
        assert_eq!(elsewhere.store.links.len(), 1);
        assert_eq!(elsewhere.store.vaults[0].display_name, "Elaris");
        assert!(matches!(elsewhere.list(10), Err(Error::NotBound)));
        assert!(elsewhere.store.get(&id).unwrap().working_text().contains("Ashfall clans"), "cached text is still readable");
        let _ = std::fs::remove_dir_all(&root);
    }

    // ---- the owner's 2026-08-25 direction, requirement by requirement ----

    /// **Requirement 3.** A person types a word; the vault answers by name
    /// and by content, and says which of the two it could actually do.
    #[test]
    fn search_finds_notes_by_name_and_by_content_and_says_when_it_could_not_look() {
        let root = scratch("search");
        std::fs::write(root.join("Locations/Kelvhold.md"), "# Kelvhold\n\nA hill fort.\n").unwrap();
        std::fs::write(root.join("Customs.md"), "# Customs\n\nThe riverlands wed at the ford.\n").unwrap();
        std::fs::write(root.join("Elsewhere.md"), "# Elsewhere\n\nNothing relevant whatsoever.\n").unwrap();
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), None).unwrap();

        // Before the index exists, only names can be searched -- and that
        // must be *reported*, or an empty answer reads as an absence.
        let before = s.search("riverlands", 20, 20).unwrap();
        assert!(!before.indexed, "no index, so the body of no note was read");
        assert!(before.hits.is_empty());
        assert_eq!(before.scanned, 0, "and nothing was opened to find that out");

        // A name search works with no index at all, and opens nothing.
        let by_name = s.search("nareth", 20, 20).unwrap();
        assert_eq!(by_name.hits.len(), 1);
        assert_eq!(by_name.hits[0].rel, "Locations/Nareth.md");
        assert!(by_name.hits[0].in_name);
        assert_eq!(by_name.scanned, 0, "a name hit costs no file read");

        s.refresh_backlinks(500).unwrap();
        let hit = s.search("riverlands", 20, 20).unwrap();
        assert!(hit.indexed);
        assert_eq!(hit.hits.len(), 1, "{:?}", hit.hits);
        assert_eq!(hit.hits[0].rel, "Customs.md");
        assert!(!hit.hits[0].in_name);
        assert!(hit.hits[0].excerpt.to_lowercase().contains("riverlands"), "{:?}", hit.hits[0]);

        // A note matching by name is not also listed as a content hit.
        let both = s.search("kelvhold", 20, 20).unwrap();
        assert_eq!(both.hits.len(), 1, "one note, one row: {:?}", both.hits);
        assert!(both.hits[0].in_name);

        // No matches is empty and not an error -- the failure path.
        let none = s.search("zzzznotintheworld", 20, 20).unwrap();
        assert!(none.hits.is_empty());
        assert!(!none.truncated, "an honest empty answer is not a truncated one");
        // An empty query is empty, not "everything".
        assert!(s.search("   ", 20, 20).unwrap().hits.is_empty());
        // Two characters is name-only: confirming it would mean reading the
        // whole vault, which is what the index exists to avoid.
        let short = s.search("ri", 20, 20).unwrap();
        assert!(short.hits.iter().all(|h| h.in_name));
        assert_eq!(short.scanned, 0);
        // The cap is real.
        let capped = s.search("md", 1, 20).unwrap();
        assert!(capped.hits.len() <= 1);

        // And with no vault bound at all it refuses rather than lying.
        let elsewhere = VaultSession::from_json(&s.to_json()).unwrap();
        assert!(matches!(elsewhere.search("x", 10, 10), Err(Error::NotBound)));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// **Requirement 4.** Attaching a note copies its information into
    /// Cartalith's own JSON, it survives the round trip, and it reads back
    /// per entity with the vault disconnected.
    #[test]
    fn a_notes_information_is_copied_into_the_json_and_reads_back_without_the_vault() {
        let root = scratch("copy");
        std::fs::write(
            root.join("Locations/Kelvhold.md"),
            "---\ntype: hill fort\nculture: highland\n---\n\n# Kelvhold\n\n**Type:** Fort  \n**Ruling authority:** [Who?]\n\n## History\n\nOld.\n",
        )
        .unwrap();
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), Some("Elaris")).unwrap();

        // Read a file's data before committing to anything -- what a search
        // result shows.
        let peek = s.document_data("Locations/Kelvhold.md").unwrap();
        assert_eq!(peek.frontmatter.get("culture").map(String::as_str), Some("highland"));

        // Attaching *one section* still copies the whole document's data,
        // because the fields live outside the section the user picked.
        let id = s
            .attach(EntityKind::Settlement, 42, "Kelvhold", "Locations/Kelvhold.md", Selection::Heading { value: "History".into() })
            .unwrap();
        let d = &s.store.get(&id).unwrap().imported_data;
        assert_eq!(d.frontmatter.get("type").map(String::as_str), Some("hill fort"));
        assert_eq!(d.fields.get("Type").map(String::as_str), Some("Fort"));
        assert!(!d.fields.contains_key("Ruling authority"), "an unanswered prompt is not data");
        assert_eq!(d.len(), 3);
        // Two surfaces, kept apart: the same word means different things.
        assert_ne!(d.frontmatter.get("type"), d.fields.get("Type"));

        // It is in the JSON -- which is the owner's sentence, literally.
        let json = s.to_json();
        assert!(json.contains("\"imported_data\""), "{json}");
        assert!(json.contains("hill fort"));

        // And it reads back on a device that has never seen the folder.
        let elsewhere = VaultSession::from_json(&json).unwrap();
        assert!(!elsewhere.is_bound());
        let rows = elsewhere.entity_data(EntityKind::Settlement, 42);
        assert_eq!(rows.len(), 3, "{rows:?}");
        assert!(rows.iter().all(|(rel, _, _, _)| rel == "Locations/Kelvhold.md"), "provenance is carried");
        assert!(rows.iter().any(|(_, o, k, v)| *o == "frontmatter" && k == "culture" && v == "highland"));
        assert!(rows.iter().any(|(_, o, k, v)| *o == "field" && k == "Type" && v == "Fort"));
        assert!(elsewhere.entity_data(EntityKind::Settlement, 999).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The failure path the copy has to survive: a note whose frontmatter is
    /// malformed must still attach, and must contribute nothing rather than
    /// something wrong.
    #[test]
    fn a_note_with_malformed_frontmatter_still_attaches_and_copies_nothing_wrong() {
        let root = scratch("malformed");
        std::fs::write(
            root.join("Locations/Broken.md"),
            "---\ntype: town\nthis line has no colon\n  indented: value\ntype: city\n\n# Broken\n\nThe block above was never closed.\n",
        )
        .unwrap();
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), None).unwrap();
        let id = s
            .attach(EntityKind::Settlement, 1, "Broken", "Locations/Broken.md", Selection::WholeDocument)
            .unwrap();
        let d = &s.store.get(&id).unwrap().imported_data;
        assert!(d.is_empty(), "an unterminated block is not frontmatter: {d:?}");
        assert!(s.store.get(&id).unwrap().working_text().contains("never closed"), "the prose came through regardless");
        assert_eq!(s.status(&id), LinkStatus::Connected, "a malformed note is not a broken link");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The copy tracks the link's own staleness and nothing else: a note that
    /// changed reports Stale, the copy is still the old one until *Reload*,
    /// and reload moves text, data and hash together.
    #[test]
    fn a_note_that_changed_after_the_copy_is_stale_until_reload_moves_both_halves() {
        let root = scratch("copystale");
        let note = "---\npopulation: 8420\n---\n\n# Nareth\n\n## History\n\nOld.\n";
        std::fs::write(root.join("Locations/Nareth.md"), note).unwrap();
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), None).unwrap();
        let id = s
            .attach(EntityKind::Settlement, 42, "Nareth", "Locations/Nareth.md", Selection::WholeDocument)
            .unwrap();
        assert_eq!(s.store.get(&id).unwrap().imported_data.frontmatter["population"], "8420");

        std::fs::write(root.join("Locations/Nareth.md"), note.replace("8420", "9001")).unwrap();
        assert_eq!(s.status(&id), LinkStatus::Stale, "one staleness idea, and it already covers the copy");
        assert_eq!(
            s.store.get(&id).unwrap().imported_data.frontmatter["population"], "8420",
            "the copy is Cartalith's until the user asks for the new one"
        );

        s.reload(&id).unwrap();
        assert_eq!(s.status(&id), LinkStatus::Connected);
        assert_eq!(s.store.get(&id).unwrap().imported_data.frontmatter["population"], "9001");
        assert!(s.store.get(&id).unwrap().working_text().contains("9001"), "text and data moved together");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Writing a section back re-syncs the link's hash, so it must re-sync
    /// the copy too — otherwise a field edited inside that section leaves a
    /// stale copy sitting under a Connected status.
    #[test]
    fn writing_a_section_back_refreshes_the_copy_it_just_invalidated() {
        let root = scratch("copywrite");
        std::fs::write(
            root.join("Locations/Nareth.md"),
            "# Nareth\n\n## Facts\n\n**Size / Population:** 8,420\n\n## Trade\n\nGrain.\n",
        )
        .unwrap();
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), None).unwrap();
        let id = s
            .attach(EntityKind::Settlement, 42, "Nareth", "Locations/Nareth.md", Selection::Heading { value: "Facts".into() })
            .unwrap();
        assert_eq!(s.store.get(&id).unwrap().imported_data.fields["Size / Population"], "8,420");

        s.set_working_text(&id, "## Facts\n\n**Size / Population:** 9,001\n").unwrap();
        let (_, hash) = s.preview_section_write(&id).unwrap();
        s.write_section(&id, &hash).unwrap();

        assert_eq!(s.status(&id), LinkStatus::Connected);
        assert_eq!(
            s.store.get(&id).unwrap().imported_data.fields["Size / Population"], "9,001",
            "no stale copy under a Connected status"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// **Requirement 5's missing half.** The "always" preference is real
    /// state, it round-trips, it is device state rather than project data,
    /// and a name nobody defined does not quietly disarm a confirmation.
    #[test]
    fn the_confirm_always_preference_persists_and_never_travels_with_a_project() {
        let root = scratch("prefs");
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), Some("Elaris")).unwrap();
        s.attach(EntityKind::Settlement, 42, "Nareth", "Locations/Nareth.md", Selection::WholeDocument).unwrap();

        assert!(!s.prefs.get("section"), "every dialog is asked for by default");
        assert!(s.prefs.set("section", true));
        assert!(s.prefs.get("section"));
        assert!(!s.prefs.get("block"), "one switch, one dialog");
        assert!(!s.prefs.set("everything", true), "an undefined name changes nothing");
        assert!(!s.prefs.get("everything"));

        let round = WritePrefs::from_json(&s.prefs.to_json()).unwrap();
        assert_eq!(round, s.prefs);
        assert_eq!(WritePrefs::from_json("").unwrap(), WritePrefs::default(), "a blank file asks every time");
        assert!(WritePrefs::from_json("{\"nonsense\"").is_err());

        // It is not in the portable store, and does not come back with it.
        let json = s.to_json();
        assert!(!json.contains("always_section"), "prefs are device state: {json}");
        assert!(!VaultSession::from_json(&json).unwrap().prefs.get("section"));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// **Requirement 2.** A culture is attachable, and its id is the only one
    /// in this system that a regenerate cannot move.
    #[test]
    fn a_culture_is_an_addressable_entity_with_a_permanent_id() {
        let root = scratch("culture");
        std::fs::write(root.join("Riverlands.md"), "---\nkind: culture\n---\n\n# The Riverlands\n\nThey wed at the ford.\n").unwrap();
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), None).unwrap();
        let id = s.attach(EntityKind::Culture, 4, "riverlands", "Riverlands.md", Selection::WholeDocument).unwrap();
        assert_eq!(s.store.get(&id).unwrap().entity_key(), "culture:4");
        assert_eq!(EntityKind::parse("culture"), Some(EntityKind::Culture));
        assert_eq!(s.entity_data(EntityKind::Culture, 4).len(), 1);
        // A culture is not a settlement, however alike the ids look.
        assert!(s.store.links_for(EntityKind::Settlement, 4).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn attaching_a_section_that_does_not_exist_fails_at_attach_time() {
        let root = scratch("badsec");
        let mut s = VaultSession::new();
        s.connect(root.to_str().unwrap(), None).unwrap();
        assert!(matches!(
            s.attach(EntityKind::Settlement, 1, "x", "Locations/Nareth.md", Selection::Heading { value: "Nope".into() }),
            Err(Error::Section(SectionError::NotFound(_)))
        ));
        assert!(s.store.links.is_empty(), "no half-made link left behind");
        let _ = std::fs::remove_dir_all(&root);
    }
}

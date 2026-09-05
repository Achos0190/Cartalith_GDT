//! The Markdown Vault's Godot surface (`MARKDOWN_VAULT_SCOPE.md` milestone 1).
//!
//! `cartalith-vault` owns every decision about Markdown, sections, blocks and
//! links; this file owns exactly two things it cannot:
//!
//! 1. **Turning a Cartalith entity into values.** [`WorldGen::entity_values`]
//!    reads `CivData` and answers `cartalith_vault::export`'s registry keys.
//!    The registry says what a settlement *can* have; this says what this
//!    settlement *does* have, which is §20's "must not expose information that
//!    the entity does not possess".
//! 2. **Not panicking.** Every method here returns a `Dictionary` with an
//!    `ok` flag and an `error` string rather than unwrapping, because a panic
//!    crossing the gdext boundary takes the Godot process down
//!    (`cartalith-rust-conventions`). A vault lives on a user's disk: the
//!    file *will* be missing, the permission *will* be revoked, and none of
//!    that may be a crash.
//!
//! ## Where the links live (rewritten 2026-09-02 — the old text was stale)
//!
//! **The links are project-scoped.** `project_bridge.rs` writes
//! `self.vault.store.to_json()` into the archive's `SLOT_VAULT`
//! (`vault.json`) and reads it back through `LinkStore::from_json`;
//! `WorldGen::load_save` clears `self.vault.store.links` first, so one
//! project's notes cannot follow the user into the next.
//! `SAVEFILE_COMPAT.md` §13.3 is the document's shape.
//!
//! `vault_state_json()`/`vault_restore_state()` still hand the whole store to
//! GDScript as text, and `shell/vault_store.gd` still writes
//! `user://markdown_vault.json` — but **only the device binding, and the
//! links only while no project is open**. That file's own header states the
//! rule and what happens to a sidecar written before it.
//!
//! **What used to stand here, and why it was wrong.** This paragraph read:
//! *"That is profile-scoped, not project-scoped … `cartalith-io`'s save
//! format is the reference HTML app's own `.zip` and carries no civ data at
//! all … Until the save format carries the civ layer, there is no project for
//! a project-scoped link store to belong to."* `DECISIONS.md` §7h replaced
//! that flat archive with the project tree on 2026-08-25;
//! `cartalith_io::DOCUMENT_SLOTS` has listed `entities/settlements.json` and
//! `vault.json` side by side ever since. The claim outlived its blocker by a
//! week in two source files (this one and `cartalith-vault/src/links.rs`),
//! which is the exact hazard `CLAUDE.md`'s *"a document's claim about itself
//! is a claim, not evidence"* names.

use crate::WorldGen;
use cartalith_vault::{export, links::entity_key, BlockAction, EntityKind, FieldFill, FieldOutcome, LinkStatus, Selection};
use godot::prelude::*;
use std::collections::BTreeMap;

/// `{"ok": false, "error": …}` — the single failure shape every method here
/// returns, so GDScript has one thing to check. `pub(crate)` so
/// `vault_saf.rs`'s `#[func]` returns the identical shape rather than
/// inventing a second one for the same boundary.
pub(crate) fn err(message: impl std::fmt::Display) -> VarDictionary {
    vdict! { "ok" => false, "error" => message.to_string() }
}

pub(crate) fn ok() -> VarDictionary {
    vdict! { "ok" => true, "error" => "" }
}

/// `"settlement"`/`"province"`/`"continent"` from GDScript into the enum.
/// An unknown string is `None` and every caller turns that into an `error`,
/// never a default — silently treating a typo as "settlement" would attach a
/// note to the wrong thing.
fn kind_of(s: &GString) -> Option<EntityKind> {
    EntityKind::parse(&s.to_string())
}

#[godot_api(secondary)]
impl WorldGen {
    // -- connection (§6, §7) ------------------------------------------------

    /// Binds a directory as this device's Markdown Vault and registers it in
    /// the link store. `display_name` may be empty, in which case the
    /// directory's own name is used.
    ///
    /// Returns `{ok, error, vault_id}`. Idempotent for a vault of the same
    /// display name — §7's "Connect Existing Vault" on a second device is
    /// this same call with a different path.
    #[func]
    fn vault_connect(&mut self, path: GString, display_name: GString) -> VarDictionary {
        let name = display_name.to_string();
        let name = if name.trim().is_empty() { None } else { Some(name) };
        match self.vault.connect(&path.to_string(), name.as_deref()) {
            Ok(id) => {
                let mut d = ok();
                d.set("vault_id", id);
                d
            }
            Err(e) => err(e),
        }
    }

    /// Drops the device-local binding. The links survive — that is the
    /// difference between §27's Unbound and detaching them.
    #[func]
    fn vault_disconnect(&mut self) {
        self.vault.disconnect();
    }

    /// `{bound, root, display_name, vault_id, link_count}`. `root` is
    /// whatever [`VaultProvider::describe`] says — a filesystem path for a
    /// desktop vault, a tree URI for a Storage-Access-Framework one — and is
    /// for showing the user where they pointed Cartalith, never for storing
    /// in project data (§5).
    #[func]
    fn vault_info(&self) -> VarDictionary {
        let v = self.vault.store.vaults.first();
        vdict! {
            "bound" => self.vault.is_bound(),
            "root" => self.vault.vault().map(|v| v.describe()).unwrap_or_default(),
            "display_name" => v.map(|v| v.display_name.clone()).unwrap_or_default(),
            "vault_id" => v.map(|v| v.id.clone()).unwrap_or_default(),
            "link_count" => self.vault.store.links.len() as i64,
        }
    }

    // -- browsing (§9) ------------------------------------------------------

    /// Markdown files in the bound vault, `/`-separated and sorted, capped at
    /// `limit` (§31: the vault is never walked exhaustively and no file is
    /// opened by this call). A non-positive `limit` uses 2000.
    #[func]
    fn vault_list_files(&self, limit: i64) -> PackedStringArray {
        let limit = if limit > 0 { limit as usize } else { 2000 };
        self.vault.list(limit).unwrap_or_default().into_iter().map(|p| GString::from(p.as_str())).collect()
    }

    /// One file's headings, as `{level, title}` — the attach dialog's section
    /// list. Empty for an unreadable file, which the caller sees as "whole
    /// document only".
    #[func]
    fn vault_file_headings(&self, rel: GString) -> Array<VarDictionary> {
        self.vault
            .headings(&rel.to_string())
            .unwrap_or_default()
            .into_iter()
            .map(|(level, title)| vdict! { "level" => level as i64, "title" => title })
            .collect()
    }

    /// A whole file's raw text, or `""`. Reading is the automatic half of
    /// §17's principle, so this needs no confirmation and no `ok` flag.
    #[func]
    fn vault_read_file(&self, rel: GString) -> GString {
        GString::from(self.vault.read(&rel.to_string()).unwrap_or_default().as_str())
    }

    /// Every entity kind this build can address in a vault, in the order the
    /// docks list them (`cartalith_vault::EntityKind`) — so GDScript passes a
    /// string this engine actually parses rather than a transcribed literal.
    ///
    /// `"faction"` joined the list on 2026-08-25 (`GUI_GAP_REGISTER.md`
    /// **CV-22**) and `"culture"` the same day (**CV-02**). `"poi"` is
    /// deliberately absent and always will be while this port has no
    /// point-of-interest entity (CV-01).
    #[func]
    fn vault_entity_kinds(&self) -> PackedStringArray {
        [
            EntityKind::Settlement,
            EntityKind::Province,
            EntityKind::Continent,
            EntityKind::Faction,
            EntityKind::Culture,
        ]
        .iter()
        .map(|k| GString::from(k.as_str()))
        .collect()
    }

    // -- searching (§9, the owner's 2026-08-25 direction) ------------------

    /// Find notes by name and, where the backlink index allows it, by
    /// content.
    ///
    /// `{ok, error, indexed, scanned, truncated, hits: [{rel, in_name,
    /// excerpt}]}`.
    ///
    /// **`indexed` is not optional to read.** Content search runs off the
    /// backlink index's word fingerprints; with no index only *names* were
    /// searched, and a panel that reports that as "no results" is telling the
    /// user their vault does not contain a word nobody looked for. When
    /// `indexed` is false the right message offers **Refresh index**.
    ///
    /// `in_name` separates the certain half from the narrowed one: a name hit
    /// cost no file read, a content hit was confirmed by opening the file.
    /// `scanned` says how many were opened, and `truncated` that `limit` or
    /// `max_reads` cut the answer short.
    ///
    /// A query under three characters is name-only, deliberately: confirming
    /// a two-letter query would mean reading the whole vault, which is what
    /// §31 forbids. Non-positive `limit`/`max_reads` use 50 and 40.
    #[func]
    fn vault_search(&self, query: GString, limit: i64, max_reads: i64) -> VarDictionary {
        let limit = if limit > 0 { limit as usize } else { 50 };
        let max_reads = if max_reads > 0 { max_reads as usize } else { 40 };
        match self.vault.search(&query.to_string(), limit, max_reads) {
            Ok(r) => {
                let hits: Array<VarDictionary> = r
                    .hits
                    .iter()
                    .map(|h| {
                        vdict! {
                            "rel" => h.rel.as_str(),
                            "in_name" => h.in_name,
                            "excerpt" => h.excerpt.as_str(),
                        }
                    })
                    .collect();
                let mut d = ok();
                d.set("indexed", r.indexed);
                d.set("scanned", r.scanned as i64);
                d.set("truncated", r.truncated);
                d.set("hits", &hits);
                d
            }
            Err(e) => err(e),
        }
    }

    // -- the note's information, copied into Cartalith's JSON --------------

    /// One note's frontmatter and template fields, read from disk right now
    /// and stored nowhere: `{ok, error, frontmatter: {…}, fields: {…}}`.
    ///
    /// What a search result or an attach dialog shows *before* the user
    /// commits to anything. Two maps rather than one merged map, because
    /// `type: town` in the frontmatter and `**Type:** City` in the body are
    /// two authoring surfaces that can legitimately disagree, and picking a
    /// winner is a guess.
    #[func]
    fn vault_file_data(&self, rel: GString) -> VarDictionary {
        match self.vault.document_data(&rel.to_string()) {
            Ok(data) => {
                let mut d = ok();
                d.set("frontmatter", &map_of(&data.frontmatter));
                d.set("fields", &map_of(&data.fields));
                d
            }
            Err(e) => err(e),
        }
    }

    /// The information Cartalith **holds** for this entity, copied out of the
    /// notes attached to it: `[{rel, origin, key, value}]`, `origin` being
    /// `"frontmatter"` or `"field"`.
    ///
    /// The owner's sentence read back: the user attached a note, its
    /// information was copied into the JSON at that moment, and this is where
    /// it comes out. It reads the **copy**, never the disk, so it still
    /// answers with the vault disconnected — which is why copying was worth
    /// doing at all (§27).
    ///
    /// Not deduplicated across notes. Two notes on one settlement may
    /// disagree, and `rel` is on every row so the disagreement is visible and
    /// attributable rather than silently resolved.
    ///
    /// Empty for a link made before 2026-08-25; *Reload source* fills it.
    #[func]
    fn vault_entity_data(&self, kind: GString, entity_id: i64) -> Array<VarDictionary> {
        let Some(k) = kind_of(&kind) else { return Array::new() };
        self.vault
            .entity_data(k, entity_id)
            .into_iter()
            .map(|(rel, origin, key, value)| {
                vdict! {
                    "rel" => rel.as_str(),
                    "origin" => origin,
                    "key" => key.as_str(),
                    "value" => value.as_str(),
                }
            })
            .collect()
    }

    /// One link's copied information, as two maps: `{ok, error, frontmatter,
    /// fields}`. The per-link view of `vault_entity_data`, for the reader
    /// panel that is already showing one note.
    #[func]
    fn vault_link_data(&self, link_id: GString) -> VarDictionary {
        let Some(l) = self.vault.store.get(&link_id.to_string()) else {
            return err(format!("no such link: {link_id}"));
        };
        let mut d = ok();
        d.set("frontmatter", &map_of(&l.imported_data.frontmatter));
        d.set("fields", &map_of(&l.imported_data.fields));
        d
    }

    // -- "confirm always" (the owner's 2026-08-25 direction) ---------------

    /// Which confirmations the user has switched off:
    /// `{section, block, field_fill}`, all booleans.
    ///
    /// **These suppress the dialog, never the guard.** A caller with a
    /// preference set must still call the matching `vault_preview_*` — that
    /// is where `expect_hash` comes from — and simply not show it before
    /// calling the write. A note edited between the preview and the write
    /// still refuses, whether or not anyone was asked.
    #[func]
    fn vault_write_prefs(&self) -> VarDictionary {
        let p = &self.vault.prefs;
        vdict! {
            "section" => p.always_section,
            "block" => p.always_block,
            "field_fill" => p.always_field_fill,
        }
    }

    /// Sets one of them. `path` is `"section"`, `"block"` or `"field_fill"`;
    /// anything else returns `false` and changes nothing, because a typo must
    /// not quietly disarm a confirmation.
    #[func]
    fn vault_set_write_pref(&mut self, path: GString, value: bool) -> bool {
        self.vault.prefs.set(&path.to_string(), value)
    }

    /// The preferences as JSON, for the shell to store **beside** the link
    /// store rather than inside it — one person's "stop asking me" is device
    /// state and must not travel into another person's copy of a project
    /// (§5). Same split `vault_backlink_index_json` already makes.
    #[func]
    fn vault_prefs_json(&self) -> GString {
        GString::from(self.vault.prefs.to_json().as_str())
    }

    /// Restores them. Malformed JSON returns `false` and leaves the
    /// preferences at their defaults, which is *ask every time* — the safe
    /// direction for a corrupt file to fail in.
    #[func]
    fn vault_restore_prefs(&mut self, json: GString) -> bool {
        match cartalith_vault::WritePrefs::from_json(&json.to_string()) {
            Ok(p) => {
                self.vault.prefs = p;
                true
            }
            Err(_) => false,
        }
    }

    // -- creating a note (§16/§17, `GUI_GAP_REGISTER.md` VA-02) ------------

    /// The templates in the bound vault, `{rel, label}` each, or an empty
    /// `Array` when no vault is connected.
    ///
    /// A template is a `.md` file with "template" in its path — the way the
    /// owner's own corpus names them (`design/vault-templates/`), and the
    /// only convention that needs no registry compiled into the binary. See
    /// `cartalith_vault::template`'s module doc for why that matters.
    #[func]
    fn vault_templates(&self) -> Array<VarDictionary> {
        self.vault
            .templates(2000)
            .unwrap_or_default()
            .into_iter()
            .map(|t| vdict! { "rel" => t.rel.as_str(), "label" => t.label.as_str() })
            .collect()
    }

    /// Where a new note for this entity goes — v3's `Settlements/{name}.md`
    /// convention, generalised to every kind `vault_entity_kinds` lists.
    /// The caller may edit it; this is a suggestion, not a rule.
    #[func]
    fn vault_suggested_path(&self, kind: GString, name: GString) -> GString {
        let Some(k) = kind_of(&kind) else { return GString::new() };
        GString::from(cartalith_vault::template::suggested_path(k, &name.to_string()).as_str())
    }

    /// Creates `rel` from `template_rel`, substituting `name` for the
    /// template's own name placeholders and touching nothing else.
    ///
    /// `{ok, path, text}` or `{ok: false, error}`. **Refuses an existing
    /// path** rather than overwriting it — the one thing that makes creating
    /// a note safe, where editing one needs a preview and a hash.
    ///
    /// Does not attach: attaching is its own validated act, and one button
    /// doing two writes is how a "create" quietly becomes an "overwrite".
    #[func]
    fn vault_create_from_template(&mut self, template_rel: GString, rel: GString, name: GString) -> VarDictionary {
        match self.vault.create_from_template(&template_rel.to_string(), &rel.to_string(), &name.to_string()) {
            Ok(text) => vdict! { "ok" => true, "path" => &rel, "text" => text.as_str() },
            Err(e) => err(e.to_string()),
        }
    }

    // -- links (§11, §12, §13) ---------------------------------------------

    /// Attaches an entity to a document, or to one heading section of it.
    ///
    /// `kind` is one of [`Self::vault_entity_kinds`]; `entity_id` is that
    /// kind's own id (a settlement's `tid`, a province's `id`, a faction's
    /// 1-based roster index, a continent's rank — see `get_continents()` on
    /// what that last one means).
    /// An empty `heading` attaches the whole document.
    ///
    /// Fails rather than creating a link that can never be read: a heading
    /// that does not exist, or whose title is duplicated in the file, is
    /// refused here.
    #[func]
    fn vault_attach(&mut self, kind: GString, entity_id: i64, label: GString, rel: GString, heading: GString) -> VarDictionary {
        let Some(k) = kind_of(&kind) else { return err(format!("unknown entity kind \"{kind}\"")) };
        let h = heading.to_string();
        let selection = if h.trim().is_empty() { Selection::WholeDocument } else { Selection::Heading { value: h } };
        match self.vault.attach(k, entity_id, &label.to_string(), &rel.to_string(), selection) {
            Ok(id) => {
                let mut d = ok();
                d.set("link_id", id);
                d
            }
            Err(e) => err(e),
        }
    }

    #[func]
    fn vault_detach(&mut self, link_id: GString) -> bool {
        self.vault.store.detach(&link_id.to_string())
    }

    /// Every link on one entity, each with its live §27 status:
    /// `{link_id, path, selection, selection_label, status, entity_label,
    /// local_changes}`.
    ///
    /// `status` is one of `unbound`/`missing`/`stale`/`cached`/
    /// `local_changes`/`connected`. Safe to call on every panel rebuild —
    /// an unreadable file is a status, not an error.
    #[func]
    fn vault_links_for(&self, kind: GString, entity_id: i64) -> Array<VarDictionary> {
        let Some(k) = kind_of(&kind) else { return Array::new() };
        self.vault
            .store
            .links_for(k, entity_id)
            .into_iter()
            .map(|l| {
                vdict! {
                    "link_id" => l.link_id.as_str(),
                    "path" => l.relative_path.as_str(),
                    "selection" => match &l.selection { Selection::Heading { value } => value.as_str(), Selection::WholeDocument => "" },
                    "selection_label" => l.selection.label(),
                    "status" => self.vault.status(&l.link_id).as_str(),
                    "entity_label" => l.entity_label.as_str(),
                    "local_changes" => l.has_local_changes(),
                }
            })
            .collect()
    }

    /// Every link in the store, for the vault window's own overview. Same
    /// keys as `vault_links_for` plus `entity_kind`/`entity_id`.
    #[func]
    fn vault_all_links(&self) -> Array<VarDictionary> {
        self.vault
            .store
            .links
            .iter()
            .map(|l| {
                vdict! {
                    "link_id" => l.link_id.as_str(),
                    "entity_kind" => l.entity_kind.as_str(),
                    "entity_id" => l.entity_id,
                    "entity_label" => l.entity_label.as_str(),
                    "path" => l.relative_path.as_str(),
                    "selection_label" => l.selection.label(),
                    "status" => self.vault.status(&l.link_id).as_str(),
                    "local_changes" => l.has_local_changes(),
                }
            })
            .collect()
    }

    /// The Cartalith-side working copy: the edited text if there is one,
    /// otherwise what was imported (§15's two states).
    #[func]
    fn vault_link_text(&self, link_id: GString) -> GString {
        GString::from(self.vault.store.get(&link_id.to_string()).map(|l| l.working_text()).unwrap_or(""))
    }

    /// Records an edit to the working copy. Writing the imported text back
    /// clears the divergence rather than storing a redundant copy, so
    /// "undo my edit by retyping it" really does return to Connected.
    #[func]
    fn vault_set_link_text(&mut self, link_id: GString, text: GString) -> bool {
        self.vault.set_working_text(&link_id.to_string(), &text.to_string()).is_ok()
    }

    /// §14's "Reload source": re-reads the selection and discards the local
    /// working copy.
    #[func]
    fn vault_reload_link(&mut self, link_id: GString) -> VarDictionary {
        match self.vault.reload(&link_id.to_string()) {
            Ok(()) => ok(),
            Err(e) => err(e),
        }
    }

    // -- write-back (§15, §16, §17) ----------------------------------------

    /// §16 steps 1-4: the document as it *would* be, plus the `hash` of the
    /// source it was computed from. `{ok, error, preview, hash}`.
    ///
    /// The `hash` must be handed straight back to `vault_write_section` —
    /// that pairing is the whole guard. A source edited between the preview
    /// and the confirmation produces a different hash and the write refuses.
    #[func]
    fn vault_preview_section_write(&self, link_id: GString) -> VarDictionary {
        match self.vault.preview_section_write(&link_id.to_string()) {
            Ok((preview, hash)) => {
                let mut d = ok();
                d.set("preview", preview);
                d.set("hash", hash);
                d
            }
            Err(e) => err(e),
        }
    }

    /// §15's "Insert Updated Text into Source" — **the only V1 path that
    /// writes an edited section back**, and it writes only that section.
    #[func]
    fn vault_write_section(&mut self, link_id: GString, expect_hash: GString) -> VarDictionary {
        match self.vault.write_section(&link_id.to_string(), &expect_hash.to_string()) {
            Ok(()) => ok(),
            Err(e) => err(e),
        }
    }

    // -- the Cartalith block (§18-§20, §23, §24) ---------------------------

    /// The fields offerable for one entity — `{key, group, label}`, in §19's
    /// own group order, already filtered to the ones this entity actually has
    /// a value for.
    #[func]
    fn vault_export_fields(&self, kind: GString, entity_id: i64) -> Array<VarDictionary> {
        let Some(k) = kind_of(&kind) else { return Array::new() };
        let values = self.entity_values(k, entity_id);
        export::offer(k, &|key| values.get(key).is_some_and(|v| !v.trim().is_empty()))
            .into_iter()
            .map(|f| vdict! { "key" => f.key, "group" => f.group, "label" => f.label })
            .collect()
    }

    /// Every value Cartalith holds for one entity, keyed by export-field key
    /// — what the preview shows beside each checkbox.
    #[func]
    fn vault_entity_values(&self, kind: GString, entity_id: i64) -> VarDictionary {
        let Some(k) = kind_of(&kind) else { return VarDictionary::new() };
        let mut d = VarDictionary::new();
        for (key, value) in self.entity_values(k, entity_id) {
            d.set(key, value);
        }
        d
    }

    // -- the map snapshot (§21, §22) ---------------------------------------

    /// §21's three radii for a UI that would otherwise hardcode them:
    /// `{key, radius, label, km, cells, path}` per row, where `cells` is this
    /// world's own conversion and `path` is the snapshot already generated
    /// for `kind`/`entity_id` (empty when there is none).
    ///
    /// `cells` is `0` before a world exists or when the world declares no
    /// width in km, which is the caller's cue that a snapshot cannot be
    /// scaled honestly — not a cue to substitute a cell count.
    ///
    /// # `missing`, and the three states a row can be in
    ///
    /// A seventh key, **`missing`, is set only when a path is filed and
    /// [`Self::snapshot_on_disk`] confirms the image is not there** — so
    /// three states stay apart in the row rather than collapsing into one:
    ///
    /// | `path` | `missing` | what happened |
    /// |---|---|---|
    /// | `""` | absent | never generated |
    /// | a path | absent | generated, and the image is there (or this device cannot check — see [`Self::snapshot_on_disk`]) |
    /// | a path | `true` | generated, and the image has since been deleted or moved |
    ///
    /// The third row is why the key exists. `entity_values` drops the Map
    /// field entirely in that state, which is §20 — but a panel that only
    /// knew "not offered" could not tell the user whether to press *Generate*
    /// for the first time or because their file manager ate the last one, and
    /// "never generated" and "generated then deleted" are different things a
    /// user may want told apart. Absent rather than `false` so a caller reads
    /// it with `has()`: there is no value of `missing` that means *unknown*.
    #[func]
    fn vault_snapshot_radii(&self, kind: GString, entity_id: i64) -> Array<VarDictionary> {
        let key = kind_of(&kind).map(|k| entity_key(k, entity_id)).unwrap_or_default();
        export::MAP_RADII
            .iter()
            .map(|(field, radius, km)| {
                let filed = self.vault.store.snapshot(&key, radius);
                let mut row = vdict! {
                    "key" => *field,
                    "radius" => *radius,
                    "label" => export::field(field).map(|f| f.label).unwrap_or(*radius),
                    "km" => *km,
                    "cells" => self.snapshot_radius_cells(*km),
                    "path" => filed.unwrap_or_default(),
                };
                if filed.is_some_and(|rel| self.snapshot_on_disk(rel) == Some(false)) {
                    row.set("missing", true);
                }
                row
            })
            .collect()
    }

    /// Writes one entity's map snapshot into the bound vault and files its
    /// path on the link store — `MARKDOWN_VAULT_SCOPE.md` milestone 2.
    ///
    /// `radius` is one of [`export::MAP_RADII`]' names. `subdir` is the
    /// folder **inside the vault** the user accepted (§22's *"the user must
    /// explicitly accept the proposed structure or choose another
    /// location"*); empty means `.cartalith/maps`, the structure §22 itself
    /// proposes.
    ///
    /// # Inside the vault, deliberately
    ///
    /// §22 offers "user-selected location" *or* "project-local generated
    /// assets" as two concepts. V1 here is the first, narrowed to a folder
    /// inside the connected vault, and the narrowing is what makes the note
    /// portable: the block carries a **vault-relative** path, so a vault
    /// copied to another machine still renders its own maps. An absolute path
    /// to somewhere else on this disk would be a §5 violation written into
    /// the user's own note, where it outlives anything Cartalith could later
    /// correct. `FsVault::resolve` is what enforces it — the same containment
    /// check that refuses `..` for a note.
    ///
    /// Returns `{ok, error, path, rel, width, height, bytes, cells_across}`.
    #[func]
    fn vault_snapshot(&mut self, kind: GString, entity_id: i64, radius: GString, subdir: GString, size: i64) -> VarDictionary {
        let Some(k) = kind_of(&kind) else { return err(format!("unknown entity kind \"{kind}\"")) };
        let radius = radius.to_string();
        let Some((_, _, km)) = export::MAP_RADII.iter().find(|(_, r, _)| *r == radius) else {
            let names: Vec<&str> = export::MAP_RADII.iter().map(|(_, r, _)| *r).collect();
            return err(format!("unknown snapshot radius \"{radius}\" -- offered: {}", names.join(", ")));
        };
        let Some(binding) = self.vault.vault() else {
            return err("no vault is connected on this device, so there is nowhere inside it to put a map");
        };
        // A snapshot is written through `export_snapshot_png`, which needs a
        // real filesystem path -- `FsVault` has one; a Storage-Access-
        // Framework-backed vault does not (`VaultProvider::as_fs_vault`'s own
        // doc comment). Milestone 4 built the SAF *provider*; writing an
        // image through a `content://` grant is separate, unbuilt work this
        // says plainly rather than silently mis-writing to nowhere.
        let Some(vault) = binding.as_fs_vault() else {
            return err("this vault is not on this device's filesystem, so a map snapshot cannot be written into it yet");
        };
        let Some((cx, cy)) = self.entity_cell(k, entity_id) else {
            return err("this entity has no position on the map, so there is nothing to centre a snapshot on");
        };
        let cells = self.snapshot_radius_cells(*km);
        if cells < 1 {
            return err("this world does not say how wide it is in km, so a radius cannot be scaled to it");
        }

        // `<subdir>/<entity_key>_<radius>.png`, with the `:` of the entity
        // key spent -- it is not a filename character on Windows. Stable, so
        // regenerating a snapshot replaces the file the note already points
        // at rather than accumulating a folder of orphans.
        let subdir = subdir.to_string();
        let subdir = subdir.trim().trim_matches('/');
        let subdir = if subdir.is_empty() { ".cartalith/maps" } else { subdir };
        let key = entity_key(k, entity_id);
        let rel = format!("{subdir}/{}_{radius}.png", key.replace(':', "_"));
        let full = match vault.resolve(&rel) {
            Ok(p) => p,
            Err(e) => return err(format!("{rel} is not a path inside the vault ({e})")),
        };

        let r = self.export_snapshot_png(GString::from(full.display().to_string().as_str()), cx, cy, cells, size);
        if !r.get("ok").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false) {
            return err(r.get("error").map(|v| v.to_string()).unwrap_or_else(|| "the snapshot could not be rendered".into()));
        }
        // Filed only after the bytes are on disk, so a failed render can
        // never leave the note pointing at an image that was not written.
        self.vault.store.set_snapshot(&key, &radius, &rel);

        let mut d = ok();
        d.set("rel", rel);
        for pass in ["path", "width", "height", "bytes", "cells_across"] {
            if let Some(v) = r.get(pass) {
                d.set(pass, &v);
            }
        }
        d
    }

    /// The Markdown body of the Cartalith block for one entity and one
    /// checkbox selection (§18's shape, `export::render_body`). Pure — the
    /// preview and the write both call it, so they cannot disagree.
    #[func]
    fn vault_block_body(&self, kind: GString, entity_id: i64, selected: PackedStringArray) -> GString {
        let Some(k) = kind_of(&kind) else { return GString::new() };
        let values = self.entity_values(k, entity_id);
        let keys: Vec<String> = selected.as_slice().iter().map(|s| s.to_string()).collect();
        let heading = self.vault.store.vaults.first().map(|_| "Cartalith").unwrap_or("Cartalith");
        GString::from(export::render_body(heading, &keys, &|key| values.get(key).cloned()).as_str())
    }

    /// `{ok, error, preview, hash, action, entity_key}` — §23 rule 5's
    /// preview. `action` is `"inserted"` or `"replaced"`, which is what §24's
    /// "preview insertion location" needs to say out loud.
    #[func]
    fn vault_preview_block(&self, rel: GString, kind: GString, entity_id: i64, body: GString) -> VarDictionary {
        let Some(k) = kind_of(&kind) else { return err(format!("unknown entity kind \"{kind}\"")) };
        let key = entity_key(k, entity_id);
        match self.vault.preview_block_write(&rel.to_string(), &key, &body.to_string()) {
            Ok((preview, hash, action)) => {
                let mut d = ok();
                d.set("preview", preview);
                d.set("hash", hash);
                d.set("action", match action { BlockAction::Inserted(_) => "inserted", BlockAction::Replaced(_) => "replaced" });
                d.set("entity_key", key);
                d
            }
            Err(e) => err(e),
        }
    }

    /// Writes the machine-owned block. Same `expect_hash` contract as
    /// `vault_write_section`; everything outside the two markers is
    /// untouched, by construction rather than by care.
    #[func]
    fn vault_write_block(&mut self, rel: GString, kind: GString, entity_id: i64, body: GString, expect_hash: GString) -> VarDictionary {
        let Some(k) = kind_of(&kind) else { return err(format!("unknown entity kind \"{kind}\"")) };
        let key = entity_key(k, entity_id);
        match self.vault.write_block(&rel.to_string(), &key, &body.to_string(), &expect_hash.to_string()) {
            Ok(action) => {
                let mut d = ok();
                d.set("action", match action { BlockAction::Inserted(_) => "inserted", BlockAction::Replaced(_) => "replaced" });
                d
            }
            Err(e) => err(e),
        }
    }

    /// Removes an entity's block (§32's "stale Cartalith block"). `{ok,
    /// error, removed}`.
    #[func]
    fn vault_remove_block(&mut self, rel: GString, kind: GString, entity_id: i64, expect_hash: GString) -> VarDictionary {
        let Some(k) = kind_of(&kind) else { return err(format!("unknown entity kind \"{kind}\"")) };
        let key = entity_key(k, entity_id);
        match self.vault.remove_block(&rel.to_string(), &key, &expect_hash.to_string()) {
            Ok(removed) => {
                let mut d = ok();
                d.set("removed", removed);
                d
            }
            Err(e) => err(e),
        }
    }

    // -- author-field population (owner's amendment to §23) ----------------

    /// Previews filling the author's *own* template fields.
    ///
    /// `{ok, error, preview, hash, report}` where `report` is one
    /// `{field, outcome}` per field considered — `written`, `skipped_occupied`
    /// or `not_found`. `overwrite` false is the owner's constraint as a flag:
    /// a field the author already filled is skipped, never clobbered.
    #[func]
    fn vault_preview_field_fill(&self, rel: GString, kind: GString, entity_id: i64, overwrite: bool) -> VarDictionary {
        let Some(k) = kind_of(&kind) else { return err(format!("unknown entity kind \"{kind}\"")) };
        let values = self.entity_values(k, entity_id);
        let policy = if overwrite { FieldFill::Overwrite } else { FieldFill::OnlyIfEmpty };
        match self.vault.preview_field_fill(&rel.to_string(), &|key| values.get(key).cloned(), policy) {
            Ok((preview, hash, report)) => {
                let mut d = ok();
                d.set("preview", preview);
                d.set("hash", hash);
                d.set("report", &fill_report(&report));
                d
            }
            Err(e) => err(e),
        }
    }

    #[func]
    fn vault_write_field_fill(&mut self, rel: GString, kind: GString, entity_id: i64, overwrite: bool, expect_hash: GString) -> VarDictionary {
        let Some(k) = kind_of(&kind) else { return err(format!("unknown entity kind \"{kind}\"")) };
        let values = self.entity_values(k, entity_id);
        let policy = if overwrite { FieldFill::Overwrite } else { FieldFill::OnlyIfEmpty };
        match self.vault.write_field_fill(&rel.to_string(), &|key| values.get(key).cloned(), policy, &expect_hash.to_string()) {
            Ok(report) => {
                let mut d = ok();
                d.set("report", &fill_report(&report));
                d
            }
            Err(e) => err(e),
        }
    }

    // -- backlinks (`GUI_GAP_REGISTER.md` VA-01) ---------------------------

    /// Bring the backlink index up to date and report what it cost.
    ///
    /// **This is the only thing that ever scans.** There is no watcher and no
    /// background pass: a person presses Refresh, and only the notes whose
    /// `(modified, len)` moved are opened. A first build reads everything
    /// once, which the panel says before it starts.
    ///
    /// `{ok, seen, reread, dropped, unreadable, notes, links, entities,
    /// bytes, refreshed_at}`, or `{ok:false, error}` with no vault bound.
    #[func]
    fn vault_refresh_backlinks(&mut self, limit: i64) -> VarDictionary {
        let limit = if limit > 0 { limit as usize } else { 2000 };
        match self.vault.refresh_backlinks(limit) {
            Ok(s) => {
                let mut d = vdict! {
                    "ok" => true,
                    "seen" => s.seen as i64,
                    "reread" => s.reread as i64,
                    "dropped" => s.dropped as i64,
                    "unreadable" => s.unreadable as i64,
                };
                d.set("notes", self.vault.backlinks.note_count() as i64);
                d.set("links", self.vault.backlinks.link_count() as i64);
                d.set("entities", self.vault.backlinks.entity_block_count() as i64);
                d.set("bytes", self.vault.backlinks.approx_bytes() as i64);
                d.set("refreshed_at", self.vault.backlinks.refreshed_at as i64);
                d
            }
            Err(e) => err(e),
        }
    }

    /// Throw the index away, so the next refresh re-reads every note —
    /// **Rebuild**, and the only remedy for an index a previous build wrote
    /// with a different parser.
    #[func]
    fn vault_rebuild_backlinks(&mut self) {
        self.vault.rebuild_backlinks();
    }

    /// What the index currently knows, for the panel header: `{built, notes,
    /// links, entities, broken, orphans, bytes, refreshed_at}`.
    ///
    /// `built` is the field every reader has to branch on: an index that has
    /// never been built reports zero notes, and *"no notes"* and *"nothing
    /// indexed"* are opposite statements on screen.
    #[func]
    fn vault_backlink_stats(&self) -> VarDictionary {
        let b = &self.vault.backlinks;
        vdict! {
            "built" => b.is_built(),
            "notes" => b.note_count() as i64,
            "links" => b.link_count() as i64,
            "entities" => b.entity_block_count() as i64,
            "broken" => b.broken_links().len() as i64,
            "orphans" => b.orphans().len() as i64,
            "bytes" => b.approx_bytes() as i64,
            "refreshed_at" => b.refreshed_at as i64,
        }
    }

    /// Every note that references this entity — `{rel, form, count}`, where
    /// `form` is `"wiki"`, `"markdown"` or `"block"`.
    ///
    /// `"block"` is the row a note-to-note index alone would miss: a note
    /// carrying `entity="settlement:42"` references the settlement directly,
    /// so it is found even when that settlement has no note of its own.
    #[func]
    fn vault_entity_backlinks(&self, kind: GString, entity_id: i64) -> Array<VarDictionary> {
        let Some(k) = kind_of(&kind) else { return Array::new() };
        self.vault
            .entity_backlinks(k, entity_id)
            .into_iter()
            .map(|(rel, form, count)| {
                vdict! {
                    "rel" => rel.as_str(),
                    "form" => match form {
                        Some(cartalith_vault::LinkForm::Wiki) => "wiki",
                        Some(cartalith_vault::LinkForm::Markdown) => "markdown",
                        None => "block",
                    },
                    "count" => count as i64,
                }
            })
            .collect()
    }

    /// Notes that name this entity in prose and do not link to it —
    /// `{rel, excerpt}`.
    ///
    /// **A guess, and drawn as one.** The index narrows the vault to
    /// candidates by word fingerprint and only those files are opened; a
    /// candidate that turns out not to contain the name is dropped. `max`
    /// bounds the reads so even a name that filters badly cannot open the
    /// whole vault. Empty when the index has not been built, when the name is
    /// under three characters, or when there is genuinely nothing.
    #[func]
    fn vault_entity_mentions(
        &self,
        kind: GString,
        entity_id: i64,
        name: GString,
        max: i64,
    ) -> Array<VarDictionary> {
        let Some(k) = kind_of(&kind) else { return Array::new() };
        let max = if max > 0 { max as usize } else { 12 };
        self.vault
            .entity_mentions(k, entity_id, &name.to_string(), max)
            .into_iter()
            .map(|(rel, excerpt)| vdict! { "rel" => rel.as_str(), "excerpt" => excerpt.as_str() })
            .collect()
    }

    /// Links that resolve to no note in this vault, and notes nothing links
    /// to — the two halves of `Data ▸ Missing & orphan notes report…`, from
    /// the one index rather than from a second walk.
    ///
    /// `{broken: [{source, target}], orphans: [rel]}`, both capped at `limit`
    /// rows with the full counts in `vault_backlink_stats`.
    #[func]
    fn vault_backlink_report(&self, limit: i64) -> VarDictionary {
        let limit = if limit > 0 { limit as usize } else { 200 };
        let b = &self.vault.backlinks;
        let broken: Array<VarDictionary> = b
            .broken_links()
            .into_iter()
            .take(limit)
            .map(|(s, t)| vdict! { "source" => s.as_str(), "target" => t.as_str() })
            .collect();
        let orphans: PackedStringArray =
            b.orphans().into_iter().take(limit).map(|r| GString::from(r.as_str())).collect();
        let mut d = vdict! { "built" => b.is_built() };
        d.set("broken", &broken);
        d.set("orphans", &orphans);
        d
    }

    /// The backlink index as JSON, for the shell to write beside the link
    /// store. Separate from `vault_state_json` on purpose: the link store is
    /// **portable project data** (§5) and this is a *cache of somebody's
    /// folder*, which does not travel with a project and is rebuilt in one
    /// press if it is lost.
    #[func]
    fn vault_backlink_index_json(&self) -> GString {
        GString::from(self.vault.backlinks.to_json().as_str())
    }

    /// Restore a saved index. Malformed JSON returns `false` and leaves the
    /// in-memory index alone — a corrupt cache must never be the thing that
    /// loses the links.
    #[func]
    fn vault_restore_backlink_index(&mut self, json: GString) -> bool {
        match cartalith_vault::BacklinkIndex::from_json(&json.to_string()) {
            Ok(idx) => {
                self.vault.backlinks = idx;
                true
            }
            Err(_) => false,
        }
    }

    // -- persistence (§25, §26) --------------------------------------------

    /// The whole link store as JSON, for the shell to write to disk.
    #[func]
    fn vault_state_json(&self) -> GString {
        GString::from(self.vault.to_json().as_str())
    }

    /// Restores a link store, keeping any binding this session already has.
    /// Malformed JSON returns `false` and changes nothing — a corrupt sidecar
    /// must not take the links that are in memory with it.
    #[func]
    fn vault_restore_state(&mut self, json: GString) -> bool {
        match cartalith_vault::LinkStore::from_json(&json.to_string()) {
            Ok(store) => {
                self.vault.store = store;
                true
            }
            Err(_) => false,
        }
    }

    /// One entity's status summary for a panel header: `{link_count,
    /// status}`, where `status` is the *worst* status among its links
    /// (`""` when it has none). Cheap enough for a rebuild, and saves the
    /// panel from ranking six strings itself.
    #[func]
    fn vault_entity_summary(&self, kind: GString, entity_id: i64) -> VarDictionary {
        let Some(k) = kind_of(&kind) else { return vdict! { "link_count" => 0, "status" => "" } };
        let links = self.vault.store.links_for(k, entity_id);
        // `LinkStatus`' declaration order is worst-first, so `min` by rank is
        // the loudest thing to say. Connected is last and therefore only
        // reported when nothing else applies.
        let worst = links
            .iter()
            .map(|l| self.vault.status(&l.link_id))
            .min_by_key(|s| status_rank(*s))
            .map(|s| s.as_str())
            .unwrap_or("");
        vdict! { "link_count" => links.len() as i64, "status" => worst }
    }
}

fn status_rank(s: LinkStatus) -> u8 {
    match s {
        LinkStatus::Unbound => 0,
        LinkStatus::Missing => 1,
        LinkStatus::Stale => 2,
        LinkStatus::Cached => 3,
        LinkStatus::LocalChanges => 4,
        LinkStatus::Connected => 5,
    }
}

/// A `BTreeMap` of copied note information as a Godot `Dictionary`.
fn map_of(m: &BTreeMap<String, String>) -> VarDictionary {
    let mut d = VarDictionary::new();
    for (k, v) in m {
        d.set(k.as_str(), v.as_str());
    }
    d
}

fn fill_report(report: &[(String, FieldOutcome)]) -> Array<VarDictionary> {
    report
        .iter()
        .map(|(field, outcome)| {
            vdict! {
                "field" => field.as_str(),
                "outcome" => match outcome {
                    FieldOutcome::Written => "written",
                    FieldOutcome::SkippedOccupied => "skipped_occupied",
                    FieldOutcome::NotFound => "not_found",
                },
            }
        })
        .collect()
}

/// Rust-internal, so outside the `#[godot_api]` block.
impl WorldGen {
    /// Everything Cartalith knows about one entity, keyed by
    /// `cartalith_vault::export` field key.
    ///
    /// **A key is absent when the value is unknown, never blank.** That is
    /// §20's rule ("must not expose information that the entity does not
    /// possess") enforced at the source: `export::offer` filters on presence
    /// here, so a field this map cannot fill never reaches a checkbox and
    /// therefore never reaches a user's note as an empty row.
    ///
    /// Returns an empty map before any `generate()`, for a loaded save (no
    /// civ layer — `CivData`'s own doc comment), or for an id that no longer
    /// resolves. An entity that has gone away is not an error here; the panel
    /// shows the link's stored `entity_label` and offers a re-bind.
    pub(crate) fn entity_values(&self, kind: EntityKind, entity_id: i64) -> BTreeMap<&'static str, String> {
        let mut out: BTreeMap<&'static str, String> = BTreeMap::new();

        // A culture is answered **before** the `civ` guard below, and that is
        // the point of it: `CIV_CULTURES` is seven compile-time rows, so a
        // culture's name and terrain theme are real with no world generated
        // and stay real across a regenerate. Only its aggregates need a world,
        // and those are filled further down if there is one.
        if kind == EntityKind::Culture {
            let Some(c) = cartalith_civ::CIV_CULTURES.get(entity_id.max(0) as usize).filter(|_| entity_id >= 0)
            else {
                return out;
            };
            out.insert("entity_type", kind.as_str().to_string());
            out.insert("name", capitalise(c.key));
            // `common` and `imperial` are identity-flavoured rather than
            // terrain-themed, and `civ_culture_terrain_fit` deliberately gives
            // them no verdict rather than a fabricated one. So neither gets
            // this field at all.
            if let Some((_, terrain)) =
                cartalith_civ::CIV_CULTURE_TERRAIN_KEY.iter().find(|(k, _)| *k == c.key)
            {
                out.insert("terrain_affinity", capitalise(terrain));
            }
            let Some(civ) = self.civ.as_ref() else { return out };
            let mut factions: Vec<&str> = Vec::new();
            let mut names: Vec<&str> = Vec::new();
            let mut pop: i64 = 0;
            for (fid, e) in civ.faction_roster.0.iter().enumerate().skip(1) {
                if e.culture != c.key {
                    continue;
                }
                factions.push(&e.name);
                for s in civ.settlements.iter().filter(|s| s.placement.faction == fid as i32) {
                    names.push(&s.name);
                    pop += s.pop as i64;
                }
            }
            if !factions.is_empty() {
                out.insert("factions", factions.join(", "));
            }
            if !names.is_empty() {
                out.insert("settlements", names.join(", "));
                out.insert("population", thousands(pop));
            }
            return out;
        }

        let Some(civ) = self.civ.as_ref() else { return out };
        let gw = self.gw.max(0) as usize;
        out.insert("entity_type", kind.as_str().to_string());

        match kind {
            EntityKind::Settlement => {
                // Keyed by `tid`, not by index: an index shifts when another
                // settlement is deleted, and a link must survive that.
                let Some(i) = civ.settlements.iter().position(|s| s.tid as i64 == entity_id) else {
                    out.clear();
                    return out;
                };
                let s = &civ.settlements[i];
                out.insert("name", s.name.clone());
                out.insert("coordinates", format!("{}, {}", s.placement.x, s.placement.y));
                out.insert("population", thousands(s.pop as i64));
                out.insert("settlement_type", capitalise(crate::journey_bridge::settlement_kind_key(s.placement.kind)));
                out.insert("faction", self.faction_label(s.placement.faction));
                out.insert("capital", yes_no(s.placement.capital));
                out.insert("coastal", yes_no(s.placement.coastal));
                if let Some(e) = civ.explanations.get(i) {
                    out.insert("elevation", format!("{:.3}", e.elevation));
                    out.insert("biome", capitalise(crate::sample_bridge::biome_name(e.biome)));
                    if e.river_order > 0 {
                        out.insert("river_order", e.river_order.to_string());
                    }
                }
                if gw > 0 {
                    let cell = s.placement.y * gw + s.placement.x;
                    let province = civ
                        .provinces
                        .get(cell)
                        .copied()
                        .filter(|p| *p > 0)
                        .and_then(|pid| civ.province_list.iter().find(|p| p.id == pid));
                    if let Some(p) = province {
                        out.insert("region", p.name.clone());
                    }
                }
                if let Some(t) = civ.trade_balances.get(i) {
                    if !t.exports.is_empty() {
                        out.insert("exports", t.exports.join(", "));
                    }
                    if !t.imports.is_empty() {
                        out.insert("imports", t.imports.join(", "));
                    }
                }
                let spec = civ.place_extras.get(s.tid).specialisation;
                if !spec.is_empty() && spec != "none" {
                    out.insert("specialisation", capitalise(&spec));
                }
            }
            EntityKind::Province => {
                let Some(p) = civ.province_list.iter().find(|p| p.id as i64 == entity_id) else {
                    out.clear();
                    return out;
                };
                out.insert("name", p.name.clone());
                out.insert("faction", self.faction_label(p.faction));
                if let Some(cap) = civ.settlements.get(p.capital_settlement_index) {
                    out.insert("coordinates", format!("{}, {}", cap.placement.x, cap.placement.y));
                }
                // Which settlements sit on this province's cells -- the one
                // aggregate the per-cell `provinces` raster makes cheap, and
                // the only place `population` for a province can come from.
                if gw > 0 {
                    let mut names: Vec<&str> = Vec::new();
                    let mut pop: i64 = 0;
                    for s in &civ.settlements {
                        let cell = s.placement.y * gw + s.placement.x;
                        if civ.provinces.get(cell).copied() == Some(p.id) {
                            names.push(&s.name);
                            pop += s.pop as i64;
                        }
                    }
                    if !names.is_empty() {
                        out.insert("settlements", names.join(", "));
                        out.insert("population", thousands(pop));
                    }
                }
            }
            EntityKind::Continent => {
                let Some(c) = civ.continents.iter().find(|c| c.id as i64 == entity_id) else {
                    out.clear();
                    return out;
                };
                out.insert("name", c.name.clone());
                out.insert("coordinates", format!("{:.0}, {:.0}", c.cx, c.cy));
                out.insert("faction", self.faction_label(c.faction));
                // Real km², from this world's own cell size rather than a
                // cell count dressed up as an area.
                let km = self.cell_km_side();
                if km > 0.0 {
                    out.insert("area", format!("{} km²", thousands((c.cells as f64 * km * km).round() as i64)));
                } else {
                    out.insert("area", format!("{} cells", thousands(c.cells as i64)));
                }
            }
            // `GUI_GAP_REGISTER.md` **CV-22**. A faction is `faction_roster`'s
            // own row, addressed by its 1-based id. Everything below is real
            // roster or real aggregate -- the three vocabulary fields
            // (culture/government/religion) are author-set values this port
            // stores and validates, and `ECONOMY_SCOPE.md`'s finding that
            // *nothing simulates* them is exactly why they belong in a note
            // rather than only in a dropdown.
            EntityKind::Faction => {
                if entity_id < 1 || entity_id as usize >= civ.faction_roster.0.len() {
                    out.clear();
                    return out;
                }
                let fid = entity_id as i32;
                let e = &civ.faction_roster.0[entity_id as usize];
                out.insert("name", e.name.clone());
                out.insert("culture", capitalise(&e.culture));
                out.insert("government", self.vocab_label(&cartalith_civ::roster::CIV_GOVERNMENTS, &e.government));
                if e.religion != "none" {
                    out.insert("religion", self.vocab_label(&cartalith_civ::roster::CIV_RELIGIONS, &e.religion));
                }
                let mut names: Vec<&str> = Vec::new();
                let mut pop: i64 = 0;
                for s in &civ.settlements {
                    if s.placement.faction == fid {
                        names.push(&s.name);
                        pop += s.pop as i64;
                    }
                }
                if !names.is_empty() {
                    out.insert("settlements", names.join(", "));
                    out.insert("population", thousands(pop));
                }
                let cells = civ.territory.iter().filter(|&&t| t == fid).count();
                let km = self.cell_km_side();
                if cells > 0 {
                    out.insert(
                        "area",
                        if km > 0.0 {
                            format!("{} km²", thousands((cells as f64 * km * km).round() as i64))
                        } else {
                            format!("{} cells", thousands(cells as i64))
                        },
                    );
                }
                if let Some(cap) = civ.settlements.iter().find(|s| s.placement.faction == fid && s.placement.capital) {
                    out.insert("coordinates", format!("{}, {}", cap.placement.x, cap.placement.y));
                }
            }
            // Answered above, before the `civ` guard, and unreachable here.
            // An empty arm rather than an `unreachable!()`: this function is
            // one `#[func]` away from the gdext boundary, where a panic takes
            // the Godot process down with it
            // (`cartalith-rust-conventions`). An impossible branch is not
            // worth a way to crash.
            EntityKind::Culture => {}
        }
        // §19's Map group — the one set of values that is not read out of
        // `CivData` at all, because a snapshot is a file somebody generated
        // rather than a property of the world. Filed on the link store by
        // `vault_snapshot`, and absent until one exists, which is what keeps
        // `export::offer` from putting a Map checkbox in front of a user with
        // no image behind it.
        //
        // The value is the Markdown image `export.rs`'s module doc specifies:
        // a relative path, never a base64 payload (§22).
        //
        // **A filed path is a record that a snapshot was written, not proof
        // it is still there** (2026-09-05). The store is the only thing this
        // loop used to consult, so deleting or moving `.cartalith/maps/*.png`
        // from outside Cartalith left the Map checkbox offered and let
        // `vault_block_body` write `![](…)` into the user's own note pointing
        // at nothing — §20's "must not expose information that the entity
        // does not possess" failing on the one field whose value lives
        // outside the process. `snapshot_on_disk` closes it, and only ever
        // *removes* a field: `None` (this device cannot check) keeps the old
        // behaviour rather than hiding a snapshot nobody verified was gone.
        let key = entity_key(kind, entity_id);
        for (field, radius, _) in export::MAP_RADII {
            if let Some(rel) = self.vault.store.snapshot(&key, radius) {
                if self.snapshot_on_disk(rel) == Some(false) {
                    continue;
                }
                out.insert(*field, format!("![]({rel})"));
            }
        }
        out
    }

    /// Whether the image a filed snapshot points at is still on disk.
    /// `Some(false)` is *gone*; `None` is **cannot answer**, which is a
    /// different thing and is why this is not a `bool`.
    ///
    /// # Scoped to the filesystem provider, deliberately
    ///
    /// [`cartalith_vault::VaultProvider::exists`] would also answer for the
    /// Storage-Access-Framework provider, and asking it would be a `Callable`
    /// round trip into GDScript per radius instead of one `is_file()` — but
    /// the stronger reason is that nothing in `shell/` connects a SAF vault
    /// at all today (`vault_connect_saf` has an `engine_bridge.gd` wrapper
    /// and no caller but `_vaultsaf_probe.gd`), so the `"exists"` op's real
    /// behaviour on a device is unverified, and a handler that answered
    /// `false` would hide every snapshot on Android. `vault_snapshot` refuses
    /// a non-filesystem provider outright, so the only way a store holds
    /// snapshots this cannot check is a project written on a desktop and
    /// opened on a device — where `project_open` assigns `store.snapshots`
    /// wholesale. Unknown, there, is the honest answer.
    ///
    /// # Cost
    ///
    /// One `Path::is_file()` per **filed** radius, and none at all otherwise:
    /// callers ask only after [`cartalith_vault::LinkStore::snapshot`] has
    /// already returned a path, so an entity with no snapshot pays nothing
    /// and an entity with all three pays three.
    ///
    /// Measured 2026-09-05, this machine, warm NTFS directory, harness linked
    /// against the built `cartalith-vault` rlib and run alone — median of 7
    /// runs of 200 000 calls each: **~10 µs** per call when the file is there,
    /// **~4 µs** when it is not.
    ///
    /// **The brackets are deliberately not quoted (2026-09-05).** This read
    /// "9.65 µs (9.63..9.66)" and "4.11 µs (4.11..4.14)"; an independent
    /// harness of the same shape, also run alone, measured 10.02 µs
    /// (9.95..10.04) — the magnitude holds and the bracket does not overlap.
    /// A three-hundredths-of-a-microsecond spread across 7 runs was measuring
    /// this run's cache state, not the call. Order of magnitude is what this
    /// number is for. Why the
    /// present case is the dearer one was not measured and is not claimed
    /// here. Worst case for one entity is three of the former, ~29 µs, once
    /// per panel rebuild — against a rebuild that also crosses gdext and
    /// constructs Godot `Control`s. It is **not** cheap enough to call per
    /// frame, and nothing here does.
    fn snapshot_on_disk(&self, rel: &str) -> Option<bool> {
        Some(self.vault.vault()?.as_fs_vault()?.exists(rel))
    }

    /// One `(key, label)` vocabulary row's display label, falling back to the
    /// stored key. Used only by the Faction arm above, whose three vocabulary
    /// fields store keys and must not export them raw.
    fn vocab_label(&self, vocab: &[(&str, &str)], key: &str) -> String {
        vocab.iter().find(|(k, _)| *k == key).map(|(_, l)| (*l).to_string()).unwrap_or_else(|| capitalise(key))
    }

    /// A faction's roster name, falling back to its number. `0` is
    /// "Unclaimed", which is a real answer rather than a missing one.
    fn faction_label(&self, faction: i32) -> String {
        if faction <= 0 {
            return "Unclaimed".to_string();
        }
        self.civ
            .as_ref()
            .and_then(|c| c.faction_roster.0.get(faction as usize))
            .map(|e| e.name.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| format!("Faction {faction}"))
    }

    /// The grid cell one entity sits on, for §21's snapshot — `None` for an
    /// entity this world does not have and for a **culture**, which has no
    /// position at all (`export::PLACED`'s own reasoning: a culture is a
    /// naming vocabulary several factions share, so any point offered for it
    /// would be a fabrication).
    ///
    /// The three lookups mirror [`WorldGen::entity_values`]' own arms — a
    /// settlement by `tid`, a province by its capital, a continent by its
    /// centroid — deliberately rather than parsing the `coordinates` string
    /// that function formats. A snapshot centred on a re-parsed display
    /// string would be one rounding decision away from a different cell.
    pub(crate) fn entity_cell(&self, kind: EntityKind, entity_id: i64) -> Option<(i64, i64)> {
        let civ = self.civ.as_ref()?;
        let (gw, gh) = (self.gw.max(0) as i64, self.gh.max(0) as i64);
        let (x, y) = match kind {
            EntityKind::Settlement => {
                let s = civ.settlements.iter().find(|s| s.tid as i64 == entity_id)?;
                (s.placement.x as i64, s.placement.y as i64)
            }
            EntityKind::Province => {
                let p = civ.province_list.iter().find(|p| p.id as i64 == entity_id)?;
                let cap = civ.settlements.get(p.capital_settlement_index)?;
                (cap.placement.x as i64, cap.placement.y as i64)
            }
            EntityKind::Continent => {
                let c = civ.continents.iter().find(|c| c.id as i64 == entity_id)?;
                (c.cx.round() as i64, c.cy.round() as i64)
            }
            EntityKind::Faction => {
                // The seat of power, and **only** that -- exactly as
                // `entity_values` answers `coordinates` for a faction, which
                // has no centroid of its own. Deliberately without a
                // fall-back to any other settlement of the faction: a note
                // that shows no coordinate must not show a map centred on
                // one, and a faction with no capital is a real state
                // (`civ_recompute` can leave one) rather than a lookup to
                // paper over.
                if entity_id < 1 || entity_id as usize >= civ.faction_roster.0.len() {
                    return None;
                }
                let fid = entity_id as i32;
                let cap = civ.settlements.iter().find(|s| s.placement.faction == fid && s.placement.capital)?;
                (cap.placement.x as i64, cap.placement.y as i64)
            }
            EntityKind::Culture => return None,
        };
        (x >= 0 && y >= 0 && x < gw && y < gh).then_some((x, y))
    }

    /// One of §21's radii, in cells of *this* world. `0` when the world does
    /// not say how wide it is, which the caller reports rather than guesses.
    ///
    /// Floored at 4 cells — below that `bake_rect` is magnifying nine samples
    /// across a whole image, and the result is a picture of the interpolator
    /// rather than of the place.
    fn snapshot_radius_cells(&self, km: f64) -> i64 {
        let side = self.cell_km_side();
        if side <= 0.0 {
            return 0;
        }
        ((km / side).round() as i64).max(4)
    }

    /// One grid cell's side in km, or `0.0` before a world exists.
    fn cell_km_side(&self) -> f64 {
        if self.gw <= 0 || self.map_width_km <= 0.0 {
            0.0
        } else {
            self.map_width_km / self.gw as f64
        }
    }
}

/// `8420` -> `8,420`. The one place the vault formats a number, so a note
/// reads like prose rather than like a dump.
fn thousands(n: i64) -> String {
    let neg = n < 0;
    let digits = n.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if neg { format!("-{out}") } else { out }
}

fn yes_no(b: bool) -> String {
    if b { "Yes".to_string() } else { "No".to_string() }
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_groups_from_the_right() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(8420), "8,420");
        assert_eq!(thousands(100), "100");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(1234567), "1,234,567");
        assert_eq!(thousands(-8420), "-8,420");
    }

    #[test]
    fn status_rank_puts_the_loudest_state_first() {
        let mut all = [
            LinkStatus::Connected,
            LinkStatus::Cached,
            LinkStatus::Unbound,
            LinkStatus::Stale,
            LinkStatus::LocalChanges,
            LinkStatus::Missing,
        ];
        all.sort_by_key(|s| status_rank(*s));
        assert_eq!(all[0], LinkStatus::Unbound);
        assert_eq!(all[all.len() - 1], LinkStatus::Connected, "Connected is only reported when nothing else applies");
    }

    /// `kind_of` itself cannot run here — constructing a `GString` needs a
    /// live Godot runtime — so this asserts the decision it delegates,
    /// which is the part with a wrong answer available: an unrecognised kind
    /// must be `None`, never a default that attaches a note to the wrong
    /// entity.
    #[test]
    fn an_unknown_entity_kind_is_none_not_a_default() {
        assert_eq!(EntityKind::parse("settlement"), Some(EntityKind::Settlement));
        assert_eq!(EntityKind::parse("continent"), Some(EntityKind::Continent));
        assert_eq!(EntityKind::parse("poi"), None, "POI is not a ported concept");
        assert_eq!(EntityKind::parse("Settlement"), None);
        assert_eq!(EntityKind::parse(""), None);
    }

    /// `GUI_GAP_REGISTER.md` **CV-02**: the kind the vault gained on
    /// 2026-08-25, and the id it is addressed by.
    ///
    /// The assertion that matters is the second one. A culture's id is an
    /// index into a **compile-time** table, so if that table ever changes
    /// length or order every culture link in every user's sidecar silently
    /// re-points at a different culture — the exact failure `entity_label`
    /// exists to make visible and this test exists to make loud first.
    #[test]
    fn a_culture_is_addressed_by_its_compile_time_index() {
        assert_eq!(EntityKind::parse("culture"), Some(EntityKind::Culture));
        assert_eq!(EntityKind::Culture.as_str(), "culture");
        assert_eq!(entity_key(EntityKind::Culture, 4), "culture:4");
        assert_eq!(
            cartalith_civ::CIV_CULTURES.len(),
            7,
            "a culture link's id is this table's index; changing its length re-points every existing link"
        );
        let keys: Vec<&str> = cartalith_civ::CIV_CULTURES.iter().map(|c| c.key).collect();
        assert_eq!(
            keys,
            ["common", "imperial", "highland", "desert", "riverlands", "sylvan", "maritime"],
            "and changing its order re-points them too"
        );
        // The two identity-flavoured cultures get no terrain verdict, which
        // is why `terrain_affinity` is absent for them rather than blank.
        for identity in ["common", "imperial"] {
            assert!(cartalith_civ::CIV_CULTURE_TERRAIN_KEY.iter().all(|(k, _)| *k != identity));
        }
    }
}
